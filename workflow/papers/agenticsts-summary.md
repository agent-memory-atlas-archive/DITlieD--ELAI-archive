AgenticSTS — repo + paper analysis (source-grounded)
Repo: github.com/AlayaLab/AgenticSTS  Paper: arXiv:2607.02255 (EMNLP 2026 ARR under review)
Clone read: C:\Users\user\AppData\Local\Temp\claude\agenticsts-clone (shallow, commit e93a7b1)
Grounding: file:line refs are into the clone's AgenticSTS/ (Python agent) unless noted.

PURPOSE
A testbed for long-horizon LLM agents that reframes agent memory as an explicit, bounded, ablatable
CONTRACT instead of the default "append everything to every prompt". The default transcript-append
contract grows context without bound, lets stale traces re-enter decisions, and makes it impossible to
say which memory component caused an improvement. AgenticSTS builds the opposite: every decision prompt
is a FRESH user message assembled by typed retrieval from five named knowledge layers, so prompt size is
independent of run length and any one layer can be switched off in isolation. It is instantiated in Slay
the Spire 2 (closed-rule stochastic deckbuilder, hundreds of decisions/run) purely as a hard, unsaturated
long-horizon task; the game itself is the benchmark, not the point. The point is the memory-contract design
+ a reproducible evaluation surface.

CORE MECHANISM
The bounded memory contract: each decision prompt u_d = pi(L1, L2(s), L3(s), L4(s), L5(s)).
  L1 operator prompts (role/protocol, immutable), L2 state-typed schemas (immutable/strictness toggle),
  L3 game knowledge (static, filterable), L4 episodic postrun summaries (writable/postrun), L5 skill
  library = triggered strategic guides w/ explicit trigger + prose policy (writable/gated).
Four static system prompts are cacheable; ALL per-run state lives in the (bounded) user message.
Raw logs are deliberately NOT used as similarity-RAG (near-identical states can be strategically opposite);
retrieval returns summaries + triggered guides, not nearest-neighbor log snippets.

1. Per-decision typed retrieval (the bounded prompt assembly).
   src/memory/retriever.py::query_for_decision (retriever.py:444) assembles a WorkingContext from typed
   stores (combat/route/card-build/guide/event) in a priority order that depends on decision type
   (combat: Guide>Episodes>Rules>ShortTerm; route: Guide>RouteMem>Rules; deck: Guide>Build>Rules) with a
   per-decision-type token budget: _classify_decision_type (retriever.py:114) -> _token_budget
   (retriever.py:129), then _trim_hints / _truncate_on_boundary / _trim_working_context (retriever.py:147,
   183, 713) enforce the cap. Retrieval keyed by typed tags (enemy_key, character, decision_type), capped
   top-k (limit=5 on combat_store.query, retriever.py:507). This is what keeps the prompt flat across an
   arbitrarily long run.

2. Agent loop: observe -> retrieve -> decide -> act. src/agent/loop.py::AgentLoop (loop.py:431), async,
   driven by GameStateMachine (src/agent/state_machine.py). A dispatcher routes each decision to one of
   four model tiers rather than one call per in-game action (median ~67 strategic calls/run). Combat keeps
   at most 3 messages/round; earlier rounds are summarized THROUGH typed state, not appended raw.

3. Four-tier health-aware model router with circuit breaker. src/brain/llm_router.py (777 lines).
   CallClass groups: gameplay_fast / gameplay_strategic / postrun_analysis / evolution, each with its own
   fallback chain (llm_router.py:197, chains built ~301-362). classify_failure (llm_router.py:~90) splits
   HARD (timeout/5xx/429/empty/unparseable -> switch model immediately) vs SOFT (parse/format anomaly ->
   small retry budget); a per-(relay_profile,provider,model) CircuitBreaker (CLOSED/OPEN/HALF_OPEN) bypasses
   proven-unhealthy models; relay_profile dimension isolates gameplay creds from postrun creds so one
   endpoint's failure doesn't pollute the other. select_model (llm_router.py:439). Fallback prefers peer
   model same tier, cross-tier only as last resort.

4. Write gate cascade (admission + dedup for new L4/L5 entries). src/memory/write_gate.py::WriteGate.check
   (write_gate.py:627). Levels: L1 exact name/id match -> update; L3a reject if candidate cosine-overlaps
   an IMMUTABLE L1/L2/L3 span (>=0.70) i.e. don't re-learn what's already in the operator prompt
   (StaticSpanIndex, write_gate.py:323, rebuild_if_stale fingerprinted); L3b compare to existing L4/L5:
   >=0.85 auto-reject duplicate, [0.70,0.85) split on trigger-Jaccard (>=0.60 same context -> MERGE, else
   COEXIST as distinct-trigger variant); ambiguous [0.55,reject) band -> defer_to_judge (batch LLM judge,
   Level 4). Named thresholds write_gate.py:43-52. Embedder-unavailable degrades to lexical-only, embed-fail
   falls back to an LLM DUPLICATE/UNIQUE verdict (never blind-accept on failure). All decisions logged to
   data/evolution/write_gate_log.jsonl.

5. Pre-write A/B counterfactual gate (does a candidate skill actually change behavior before we admit it).
   src/skills/prewrite_ab.py::validate_candidate. For each mistake round: fetch_prompt_a (byte-exact
   original prompt from the run log, prewrite_ab.py:19), inject the candidate skill into it
   (composer.inject_candidate_into_prompt), redecide_b N=3 resamples on the strategic tier, run_judge
   (analysis tier) verdict {skill_helps/unclear/harmful, hit_count 0..3}. aggregate_strict (prewrite_ab.py:244)
   passes iff NO round is skill_harmful AND total hits >= ceil(total_samples * 2/3). A skill is only written
   if it demonstrably flips the original mistake decision.

6. Two skill-population routes. Mode A mistake-driven discovery: src/skills/mistake_discovery.py — loss_ratio
   (:25), baseline_a/baseline_b HP-loss baselines (:35/:48), is_mistake_episode (:70) flags combats where HP
   loss exceeds baseline, run_critic_parallel proposes candidates, then the pre-write A/B gate (#5) + write
   gate (#4) admit them. Mode B template-filled authoring: src/skills/stub_template.py / stub_filler.py /
   stub_validators.py fill a frozen trigger+prose template (isolated "stub_" namespace the mistake path is
   blind to, write_gate.py:642). Paper: Mode B reaches the same 6/10 point estimate as hand-authored seeds.
   Skill distillation "evolution" phase: src/brain/evolution_engine.py::EvolutionEngine.run_evolution
   (evolution_engine.py:273) — a read-phase then a tool-based author/write phase with a final force-write round.

KEY INTERFACES (data flow: game -> agent -> memory)
- Mod HTTP API (C# / .NET 9 / Godot / Harmony, AgenticSTS-Mod, fork of CharTyr/STS2-Agent): GET /state,
  POST /action, GET /events/stream on localhost:8128. Agent talks to it via src/mcp_client/client.py (async
  httpx) + sse_client.py.
- AgentLoop.__init__ (loop.py:443) wires McpClient + memory stores + brain. State parsed by
  src/state/state_parser.py::parse_state into GameState (src/state/game_state.py).
- query_for_decision(gs, short_term, combat_store, route_store, card_build_store, guide_store, *,
  card_memory_store, event_store, current_round) -> WorkingContext  (the bounded prompt inputs).
- WriteGate.check(candidate: Candidate, existing: Sequence[ExistingEntry]) -> GateDecision(action in
  update/accept/reject/merge/defer_to_judge). Candidate carries name, content, target_layer, trigger_tags.
- llm_router.select_model(call_class, ...) -> chosen (provider, model) honoring circuit-breaker health.
- prewrite_ab.validate_candidate(candidate, episode, log_path, combat_system_prompt) ->
  (passed: bool, per_round_results, total_hits).
- Config surface: STS2_MODEL_FAMILY (gemini|gpt|qwen|claude, all 4 tiers resolve from a registry),
  STS2_{FAST,STRATEGIC,ANALYSIS,EVOLUTION}_MODEL, STS2_POSTRUN_ENABLED, STS2_EVOLUTION_ENABLED,
  STS2_DATA_REPO, per-run --no-skills / --no-memory / --no-llm ablation flags.

EVIDENCE
Numbers verified from README + arXiv:2607.02255 abstract (WebFetched) + CITATION.cff — NOT independently re-run.
- Fixed-A0 five-cell ablation, N=10/cell (Silent character, ascension A0): baseline-strict 3/10 (mean 70.4),
  prompt-only 4/10, mode-a hand-authored skills 6/10 (85.5), mode-b-frozen 6/10 (83.3), full-frozen 6/10 (82.1).
  Largest delta is the L5 skill layer (+2/10). Authors explicitly label it DIRECTIONAL not significant
  (Fisher exact p~=0.37; pooled p~=0.148; CIs overlap). Honest caveating throughout — a notable strength.
- Calibration (not matched baselines): public AGI-Eval frontier LLMs 0 wins at A0 across 5 configs;
  human win rate 16%.
- Cross-backbone probe (frozen Gemini-trained stores applied elsewhere): Qwen 3.6-27B score +84.5%,
  DeepSeek V4-Pro -18.1%, Gemini 3.1-Pro +16.6% — stores are backbone-sensitive (stated as empirical, not premise).
- Efficiency vs two accumulating-context StS2 agents (STS2MCP, CharTyr): ~4x less wall-clock/floor,
  66-90x fewer fresh (non-cached) LLM tokens/score-point (>=7x under a deliberately absurd upper bound).
  Explicitly flagged asymmetric: theirs measured, ours ESTIMATED (strategic calls * ~5k user tokens) and
  partly attributable to decision batching. Competitor 0/5 partly an invalid_action interface bug (disclosed).
- Eval harness IS in the repo: scripts/reproduce/reproduce_table_2, _table_3, recompute_all.sh recompute the
  paper tables from AgenticSTS-Data/runs/history.jsonl (385 rows -> filter victory/defeat = 298 paper games)
  with Wilson CI / bootstrap, no LLM calls. Frozen SHA-anchored L4/L5 snapshots + per-decision prompt records
  released (298 traj in repo, 312-superset + 305 gzipped full logs on HF ShandaAI/AgenticSTS-trajectories).

DEPENDENCIES + LICENSE
License (verified via LICENSE + per-dir files): monorepo, tri-licensed. AgenticSTS/ (the Python agent) =
Apache-2.0. AgenticSTS-Mod/ (C# mod) = AGPL-3.0 (inherited from CharTyr fork). AgenticSTS-Data/ = CC-BY-4.0.
The Apache grant does NOT extend to the mod; running a modified mod triggers AGPL. Verbatim game metadata under
data/knowledge/upstream/ is NOT redistributed (regenerate locally). For a Rust steal, only Apache-2.0
AgenticSTS/ ideas are cleanly reusable; avoid the AGPL mod.
Python deps (pyproject.toml, py>=3.12): httpx>=0.27, pydantic>=2.0, anthropic>=0.40, numpy>=1.26, scipy>=1.14,
rich>=13.0, filelock>=3.12; optional monitor(fastapi/uvicorn/websockets), dev(pytest/pytest-asyncio/ruff),
eval(matplotlib). All lightweight/standard; no exotic or ML-training deps. ruff configured, py312 target.

MATURITY
Substantial, well-structured research codebase, not a toy. ~62k LOC across src/ (brain 18.9k, memory 15k,
agent 10.9k, skills 4.5k), 509 Python files, 212 test files (tests/ = unit + golden-log regression suites
incl tests/regression/test_golden_logs.py, test_log_replay.py; tests/patch/ well covered). Docstrings are
dense and reference design specs (docs/superpowers/specs/*.md) — code was spec-driven. Named constants, graceful
degradation everywhere (embedder-unavailable, embed-fail, circuit-breaker), no obvious stubs in the core paths
read. Single visible commit on the shallow clone (squashed release), so per-commit activity history unknown.
CITATION.cff + NOTICE + tri-license map = release hygiene above typical academic repos. Quality read: HIGH.

PORTABILITY (to ELAI: Rust FSM-gated coding-agent runtime w/ sub-agent dispatch, verification gates,
self-improvement loop, local+cloud backends). Five concrete stealable ideas, each with the reference impl:
1. BOUNDED TYPED-MEMORY CONTRACT for the agent prompt. Instead of appending transcript, compose each
   sub-agent/decision prompt from named typed slots (immutable operator/spec layer + filterable knowledge
   + writable episodic + gated skill layer) with a per-decision-type token budget and capped top-k retrieval,
   so context is flat regardless of session length and any layer is ablatable. This DIRECTLY answers ELAI's
   H-50/H-51 context-lifecycle work (regime-aware in-session mgmt + cross-session handoff) and the local-window
   pressure problem in MEMORY.md. Ref: src/memory/retriever.py::query_for_decision (retriever.py:444),
   _token_budget (retriever.py:129), _trim_working_context (retriever.py:713). Key doctrine: retrieve
   summaries+triggered guides, NOT nearest-neighbor raw-log RAG (retriever design note, README "not similarity RAG").
2. WRITE GATE CASCADE for admitting skills/memory (dedup + overlap-with-immutable check). Before adding any
   learned artifact: exact-match -> update; reject if it duplicates an IMMUTABLE base-prompt span (cosine);
   vs existing store auto-reject/merge/coexist on cosine + trigger-Jaccard; ambiguous band -> LLM judge; log
   every decision. This is a ready-made anti-bloat / anti-duplication admission gate for ELAI's skill library
   and memory-bank, and the "don't re-learn what's already in the base prompt" span index is novel and valuable.
   Ref: src/memory/write_gate.py::WriteGate.check (write_gate.py:627), StaticSpanIndex (write_gate.py:323),
   thresholds (write_gate.py:43-52).
3. PRE-WRITE A/B COUNTERFACTUAL GATE for self-improvement. A newly-discovered skill/rule is admitted ONLY if,
   re-running the original decision N times WITH the candidate injected, an independent judge confirms it flips
   the outcome favorably (strict 2/3 + zero-harmful). This is exactly ELAI's needed "execution-grounded /
   stronger-than-generator fitness filter" for its DGM/self-improvement loop (H-43 keystone, and the
   wire-enforcement doctrine that a self-improvement loop without a real un-gameable filter collapses). Steal
   the counterfactual-replay-then-strict-aggregate shape. Ref: src/skills/prewrite_ab.py::validate_candidate +
   aggregate_strict (prewrite_ab.py:244).
4. FOUR-TIER HEALTH-AWARE MODEL ROUTER w/ circuit breaker + call-class fallback chains + credential-profile
   isolation. Maps onto ELAI's PerRoleRuntime / RuntimeOrchestrator / RuntimeAdapter (B-37) and its
   local+cloud backend switching: classify failures HARD vs SOFT, per-(profile,provider,model) breaker
   CLOSED/OPEN/HALF_OPEN, peer-first fallback, isolate gameplay creds from analysis creds. Ref:
   src/brain/llm_router.py (CallClass:197, select_model:439, classify_failure ~:90, CircuitBreaker).
5. ROLE-TIERED DISPATCH + DECISION BATCHING as a cost lever. One "strategic" LLM call plans several
   downstream mechanical actions (fast tier / no-LLM for trivial), giving the ~4x wall-clock and 66-90x token
   wins. ELAI already has role runtimes; the stealable specifics are (a) a cheap/no-LLM fast tier for trivial
   sub-steps, (b) one strategic call emitting a multi-action plan the runtime executes without re-calling.
   Ref: src/agent/loop.py (AgentLoop dispatch + CombatPlan/PlannedAction), src/brain/planner.py.
Bonus reusable pattern: ablatable named slots + config flags (--no-skills/--no-memory/--no-llm,
STS2_*_ENABLED) make every capability independently switch-off-able for measurement — mirrors ELAI's
wire-dark flag doctrine and gives clean A/B evaluation of each layer's contribution.
NOT portable: the C# StS2 mod (AGPL, game-specific), the 577-card/121-monster L3 game knowledge, all the
combat/deck domain logic. These are ~half the repo and are game glue, not transferable.

UNKNOWNS
- Did not independently re-run the reproduce_table scripts or any game — all quantitative claims are as-reported
  (README/arXiv abstract/CITATION), not re-measured here.
- Full commit history / real dev cadence: the shallow clone shows one squashed release commit; activity unknown.
- The token-efficiency claim is authors' own ESTIMATE for their side (disclosed asymmetry); the true head-to-head
  is deferred to their own Future Work (same-codebase accumulating-context cell not yet run).
- Level-4 batch LLM judge in the write gate is described as "stubbed / observation mode" at that commit
  (write_gate.py:596-599) — the full judge may be partial; the L1-L3 cascade is complete.
- HF full-trajectory dataset (305 gzipped logs) not downloaded; only the in-repo 298-traj snapshot inspected.
- arXiv ID 2607.02255 resolves and abstract matches the repo; did not fetch the full PDF body (§7 token
  accounting, §Limitations read only via README summaries).
