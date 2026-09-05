CONTEXT MANAGEMENT DEEP DIVE
date 2026-06-25
author main-llm (opus 4.8 1M)
question how does ELAI manage broad context across small + cloud models so every task (plan, reason, execute) always has the important context but no bloat, no wrong-thing search, no half-blind fixes. trade time for perfection but stay as fast as provider rate limits + concurrency allow.

method read 1 file > reason > write down > repeat. this file accumulates notes as i go, then ends with grounded improvement proposals.

=========================================================
PART 0 - NOTES FROM THE REQUESTED SKILLS (read first)
=========================================================

THROUGHLINE the single most important fact: every orchestration/intake skill is itself a context-management device. The recurring invariant is "main LLM is a STRATEGIST not an executor; it holds only {plan, state, returns, decisions}; workers run in their own context windows that are GARBAGE-COLLECTED on return." Dispatching a worker SAVES main-context, it does not cost it. This is the load-bearing idea of the whole harness.

orchestrate (plan-file walker)
- reads a phase markdown, parses Phase>sub-Phase>Task, walks an FSM cycle PER sub-phase: (context-compiler + integration-explorer in PARALLEL cap=3) > skip prompt-engineer > executor > auditor 1 cycle > memory-writer.
- HARD RULE: no Edit/Write on plan-scope files BEFORE the parallel batch returns. Main LLM never runs tests (auditor only) or writes memory-bank (memory-writer only).
- per-sub-phase isolation is the context unit: each sub-phase gets a fresh context-package + fresh exploration, so the main LLM never accumulates raw source.
- phase-end sweep = 6 generic + stack gates; commit ONCE at end.
- model ladder: haiku=bulk read, sonnet=canonical agents+executor, opus/fable=controller only. parallel_cap=3 HARD (rate limit).
- 4 tripwires only break autonomy (irreversible op, 50-event bubble cap, dep-wait>4h, sweep-fail x3).

auto-orchestrate (general task, not plan files)
- same strategist-never-executor doctrine. run-state.md = structured global state updated after EVERY agent return (OBJECTIVE / TASK GRAPH / DECISIONS / BUDGET / VERIFICATION).
- SCOPED CONTEXT rule (key): a worker receives ONLY what its task needs - the task, relevant paths, the DISTILLED slice of prior results (orchestrator-summarized), its handoff pointer. NEVER the full run-state, never sibling raw output, never failed-sibling tracebacks. <-- this is the anti-bloat mechanism for workers.
- handoff mediation loop: worker BLOCKED > appends entry to ONE topic-keyed handoff file > orchestrator reads the ENTRY (not whole file) > decides > appends DECISION block > next worker prompt points at "read entry N + its DECISION". forks resolved by orchestrator, never workers.
- integration gate R10: per-worker green necessary-not-sufficient; isolated GC'd contexts CREATE seam-blindness; explicit integration smoke (independent agent, real entrypoint, HARD input) after every 2-3 units. vertical-slice decomposition (producer+consumer+smoke together).
- autonomy envelope: 30-dispatch cap before check-in, re-dispatch cap 2 per task id, monotonic-progress check every return.

idea-intake (Layer A: idea > Concepts > Phases > Steps > BabySteps)
- ONE AGENT PER NODE PER LEVEL with fresh per-invocation context. explicit rationale (line 58-60): "a single designer agent doing all 200 nodes in one context loses quality past ~30K tokens of working memory." <-- this is the empirical context-budget number the harness is built around.
- critic-loop + per-level audits catch errors AT THE LEVEL they were introduced (cheaper than cascading 200 baby steps then auditing once).
- parallel batches of 4 (5h rate-limit headroom). user checkpoints at concept-list, concept-tree, final-tree.
- end-session-and-resume option at the heaviest stage (step 7-10) keeps each session working memory clean; cache hits on prior research make resume cheap.
- ~130-180 agent calls per Emil-sized intake; the cost buys per-node grounding the main context never holds.

paper-to-plan
- replaces a 4-session manual flow; the expensive part (12-file ~165K-token context load) is replaced by ONE file: docs/thefinalglance/ELAI-FEATURE-MAP.md (single source of truth for features+composition).
- haiku=bulk repo read, sonnet=ingestor/synth, main(fable/opus)=steal reasoning+plan authoring in-band (source fidelity > token economy here). cap 3 parallel.
- repo sizing: <200K = 1 mapper; >=200K = ceil(tokens/100K) haiku explorers partitioned by subtree + 1 synthesizer; >1M = stop and ask to scope. <-- token-budgeted fan-out.

oracle
- grounded Q&A via researcher agent in oracle mode. the WHOLE point: the heavy web-fetch context (10-15 WebFetches, 50-100K tokens "mostly discarded after extraction") lives in the RESEARCHER's context, not the main LLM's. main LLM gets back only the formatted answer.
- cache via mcp_search_context topic prefix - prior research re-used, not re-fetched.

spec-eval
- evaluation lens (as-specified), read-only. relevant to context mgmt only in that it reads the feature map / summaries as CONTEXT PROVISION and reasons over design. confirms the summaries are "context to reason over, treat as built" - matches the user's instruction here.

pdf
- standalone utility; MinerU extraction lives in an external engine + venv; the heavy model load (15-20GB) is off the main process. read-only.

INTERIM TAKEAWAY (skills layer)
The harness already encodes a strong, consistent context model: thin orchestrator + GC'd workers + scoped-context dispatch + single-source-of-truth feature map + cache-by-topic + per-node fresh context + a hard ~30K working-memory budget assumption + a hard parallel cap of 3 (provider rate limit). The open questions for the deep-dive: (1) how is the WORKER's context actually compiled (context-compiler) - is it scoped well or does it over/under-fetch? (2) how does the RUNTIME (the actual elai_cc product, not the .claude harness) manage context for the local + cloud models during a live task - compaction, KV cache, context window ceilings? (3) where does plan/reason/execute split happen in the PRODUCT vs the harness, and do they share a context model?

=========================================================
PART 1 - NOTES FROM THE FEATURE MAP (read 100%, 2501 lines)
=========================================================

There are TWO context-management systems in ELAI, and they must not be conflated:
  (A) the HARNESS layer (.claude/, the Claude-Code session you talk to): orchestrate / idea-intake / paper-to-plan / auto-orchestrate. This splits PLAN vs REASON vs EXECUTE across agent dispatches. Status: WIRED, mature.
  (B) the PRODUCT runtime layer (.elai_cc/ crates, the thing ELAI ships to its own users running local+cloud models): context_budget / context_policy / compaction ladder / session-dag / sidecar KV. Status: DESIGN excellent, ACTIVATION partial/dark.

THE LOAD-BEARING RUNTIME CONTEXT FEATURES (Section 1.5 + 1.3 + 1.9):

CONTEXT BUDGET ALLOCATOR [WIRED] phase-3E, crates/orchestrator/src/context_budget.rs.
  8 FIXED SLOTS: SystemPrompt / Rules / PhaseSpec / RepoMap / CodeContext / SsbEntries / GrcHistory / OutputReserve.
  floors + steal (a slot under its floor steals from a slot over budget), 2 cache breakpoints (xxhash prefix), 3-level adaptive reduction <50ms. B-10 ModelCapabilityProfile WIRED.
  <-- THIS is the live per-call context manager for the product. The 8 slots ARE the "always have important context, no bloat" answer. Every model call (local or cloud) gets a budgeted assembly.

HIERARCHICAL PLAN MGMT [WIRED] phase-3G. skeleton <=2-4K tokens ALWAYS in Slot3 + per-phase detail loaded on demand, contract extraction, Kahn cycle detection.
  <-- this is the "plan is always present but never the whole 200-node tree" mechanism. The skeleton is the cheap always-resident map; detail is paged.

REGIME-AWARE CONTEXT POLICY [PARTIAL, uncommitted] H-50, context_policy.rs.
  ContextPolicy selected by BackendMode: CLOUD = big cache-stable prefix + late compaction; LOCAL = window-minimization + unload-early.
  THE KEY FINDING (user's own catch, confirmed): cloud and local need OPPOSITE policies, but ELAI historically ran ONE cloud-shaped policy for both -> local was unprotected. Concrete bugs named: flat-100K auto-compact never fires <32K (conversation.rs:22); 85% gate reads 64K-non-opus ignoring ELAI_LOCAL_MAX_CONTEXT (cli_consts.rs:20); recover_stubs re-inflates the trim back to ~0 (session_dag.rs:1551); local dies at hard context_window_blocked. ALSO local caching is a NO-OP: sidecar PrefixKVCache (server.py:1819) is BYPASSED on the stream:true path the live client uses (runtime_client.rs:547).

DETERMINISTIC OBSERVATION MASKING + READ-BACK [PARTIAL] H-6. masking DROPS trailing observations, never rewrites the cache prefix (HD-2). LLM-resummarize-the-prefix is BANNED (busts cache). This is the cloud-lane compaction primitive.

COMPACTION LADDER [DARK(ELAI_E25_* x9)] E-25. 6-tier compaction ladder + opusplan substate + prompt_decision + edit_checkpoints. Extended by H-6/H-49. Dark = built, gated off.

EMPIRICAL CONTEXT CALIBRATION [PLANNED(J-34)] kill the folklore thresholds: `elai bench context-fill` MEASURES per-(model,lane) fill->verified-outcome degradation curves on deterministically-padded fixtures (gate verdicts, never LLM-judge). ModelCapabilityProfile gains fill_degradation_curve / calibrated_trim_fraction / positional_value_curve. context_policy prefers the measured knee over hardcoded CLOUD_TRIM_FRACTION=0.85 (context_policy.rs:70). E-49 5-position context_rot_probe extended to a positional-value curve driving 3E SLOT ORDERING (per-model-static layout = cache-safe). <-- the single most important PLANNED context fix: replace guesses with measurement.

CONTEXT-ROT CEILING/PLACEMENT + LOCAL COMPACTION RECIPE [PLANNED(J-76)] chunk-count cap + head/tail placement (never middle, "lost in the middle" 30%+) consuming J-34's positional curve; RLM _compact_history recipe for the LOCAL lane (85%-window trigger, summarize PRESERVING intermediate numerics). IMPORT THE METHOD NOT THE NUMBERS (frontier curves != Qwen-7B).

PRISTINE CONTEXT INJECTOR [PLANNED(I-5)] hard-EXCLUDE failed-sibling tracebacks, contrastive constraint only. <-- the runtime twin of the harness's "never give a worker sibling raw output / failed tracebacks" rule. This is exactly the "don't bloat from searching/finding the wrong things" the user named.

CONTEXT-ACCESS SPEC [PLANNED(J-78)] formalize a DECLARED, journaled per-dispatch context-access spec — which prior SSB rows / sibling outputs / memory a dispatch MAY read; orchestrator COMPUTES it deterministically (ELAI DECLARES vs Conductor LEARNS). I-5 hard-exclude + the H-31 auditor firewall become DECLARED cases. OBSERVE-ONLY first.

PABU TYPED BELIEFSTATE HANDOFF [PLANNED(I-11)] replaces RAW history on cross-model/tier handoffs with a typed, DKG-keyed beliefstate. Consumed by I-7 escalation cascade (condense-on-escalate). <-- the "small model gets a distilled belief, not the full transcript" mechanism for local<->cloud handoff.

DETERMINISTIC CONTEXT COMPRESSION [PLANNED(I-14)] LSP slicing + tree-sitter folding; every transform lands a REVERSIBLE ContextTransformReceipt; ciphers BANNED. <-- compress code context structurally (fold bodies, slice by LSP relevance) not by lossy summarization.

INFINITE-CONTEXT CHAT [WIRED] E-1. A-MEM + MemGPT-style digest, NoteStore + ppr symbols in production paths.

USER MEMORY SUBSTRATE [PARTIAL] crates/memory, E-5/H-30..H-33: Mem0 ADD/UPDATE/DELETE/NOOP + Zep bi-temporal + poisoning defense; bi-temporal supersession HARD-FILTERS superseded rows at retrieval (never [OUTDATED] narration — the measured Graphiti +68%-token trap); AUDITOR-OBJECTIVITY FIREWALL: auditor/critic/adversary get ZERO memory injection. dark on capture + retrieval-into-prompt (the activation gap again).

CROSS-SESSION SLEEP-TIME HANDOFF [PARTIAL] H-51. grounded topic-digest auto-authored from the journal, tiered load ~20-27K -> ~1-3K. wires handoff-protocol.md into the runtime.

PLAN/REASON/EXECUTE SPLIT IN THE PRODUCT: the FSM (9 states IDLE/PLANNING/FEASIBILITY_PROBE/TESTING_FIRST/EXECUTING/AUDITING/MEMORIZING/COMPLETE/CONFIGURING) is the split. Each state phase-loads its OWN tools+rules (phase-aware tool/rule loading, 3A). So PLANNING-state context != EXECUTING-state context: different slots, different rules, different tools. That is the product-level "plan and execute never share a context" guarantee. SPRINT CONTRACTS + RALPH LOOP [WIRED, I-21]: generator/evaluator have ZERO shared context (the +0.12 self-review bias). DIRECTIVE SUPERVISOR PLANE [PLANNED(I-1)]: generator emits pure Intent->Directive; a process-separated Supervisor is the ONLY side-effect executor and holds credentials; generator cannot even reach gate code. <-- strongest form of the split.

INFERENCE / ROUTING (1.9): EscalationCascade [WIRED, I-7] local->cloud waterfall, condense-on-escalate via I-11. CALIBRATED ROUTING [PARTIAL, H-7/E-31] architect/editor/verifier role routing over PerRoleRuntime. DYNAMIC THINKING BUDGETS [DARK, I-26] serial-depth scaled (depth not window). PROMPT CACHING cloud [WIRED main loop] markers live; LOCAL prefix-cache is a NO-OP (bypass). MODEL-STRENGTH MODALITY ROUTER [PLANNED(J-55) KEYSTONE]: route the ACTION SURFACE per model strength (CodeAct for capable lanes, constrained single-tool-call + minimal schemas for sub-14B local), Qwen-7B NEVER routed to CodeAct. CONTEXT WINDOW: ELAI_LOCAL_MAX_CONTEXT 25-32K (Qwen native ceiling); cloud cloud_200k.

CONCURRENCY vs RATE LIMITS: parallel_cap=3 HARD across all orchestrators (provider rate limit). FREE-API ROTATION POOL [PLANNED(J-54)] per-key daily quota + 429 header-priority cooldown + multi-key rotation + keyless overflow — the way to get MORE concurrency without tripping limits. BATCH API LANES [PARTIAL, H-5] flat 50% for non-interactive lanes (Cosmos/research/bench), racing REJECTED.

THE HONESTY NOTE (Section 4, superpowers/context-rot verdict, verbatim weight): "ELAI context-mgmt DESIGN is at/beyond both [SOTA] sources, but ACTIVATION is the gap - H-50/H-51 PARTIAL uncommitted, E-25 DARK x9, J-34 PLANNED. Live cloud trim still on the folklore 0.85; H-50 finding had LOCAL lane on a cloud-shaped policy. Highest-leverage = finish/commit H-50/H-51 + build J-34, not a new plan." <-- This is the thesis of the whole deep-dive. The design is done; the wins are activation + measurement, not invention.

OPEN QUESTIONS FOR THE CODE INVESTIGATION (to ground before proposing):
  Q1. context_budget.rs: are the 8 slots + floors + steal + adaptive-reduction actually called on EVERY model dispatch (local AND cloud), or only some paths?
  Q2. context_policy.rs: is the regime split actually selected by live BackendMode, or is the cloud-shaped policy still the only one running (H-50 uncommitted)?
  Q3. the hierarchical skeleton (3G): confirm the <=2-4K skeleton is always-resident and detail is paged — and whether the HARNESS (orchestrate) actually uses it or only the product runtime does.
  Q4. the local sidecar KV bypass (runtime_client.rs:547 / server.py:1819): confirm and scope the cost.
  Q5. does the WORKER context-compiler (the agent) produce a tight scoped package, or can it over-fetch?

=========================================================
PART 2 - CODE INVESTIGATION (grounded against the live tree 2026-06-25)
=========================================================

METHOD: read context_policy.rs in full; grepped callers of the regime functions, adaptive_reduce, BeliefState; read the runtime_client cloud message-build path; confirmed file existence of the planned modules. Several feature-map statuses are now STALE in the right direction (built since the 06-23 map).

WHAT IS ACTUALLY WIRED TODAY (verified, not doc-claimed):
W1. ContextBudget allocator [WIRED, GROWN TO 10 SLOTS]. context_budget.rs now has 10 slots (the original 8 + ContextSlot::Memory [H-31] + ContextSlot::Evidence [H-53]). compute() + update_breakpoints() are called on the live cloud message build (runtime_client.rs:544-577) with the 2 cache breakpoints (after PhaseSpec, after CodeContext) and the HD-2 cache-bust debug_assert. compute_with_profile / compute_repair_context exist. So the per-call budgeted assembly is REAL on the cloud path.
W2. H-50 regime policy [NOW WIRED - was "PARTIAL, uncommitted" at map time]. context_policy.rs (248 LOC) is committed AND called from production: live_cli.rs:1227 effective_context_window(backend_mode); runtime_builder.rs:371 trim_threshold_for_mode(backend); runtime_client.rs:592/850 regime_policy(self.backend). Local=floor(window*0.75), Cloud=floor(window*0.85), kill-switch ELAI_CONTEXT_POLICY_DISABLE, contract test h50_context_policy_contract.rs asserts the wiring. THE BIG "local ran a cloud-shaped policy" BUG IS FIXED. effective_context_window(Local) reads ELAI_LOCAL_MAX_CONTEXT.
W3. I-10 cache-economics TTL tiering [WIRED] on the cloud path: CacheEconomicsPlanner::plan_ttl(gap_stats, regime_policy(backend), cache_head_stable) drives Long(1h)/Short/NoCache cache_control; LOCAL backend -> NoCache (correct: no cross-request KV reuse to cache against). Composes on top of H-1/H-2 markers (live, anthropic_api.rs).
W4. Dual tool-result serialization [WIRED, I-13-shape]. tool_result.rs: to_ui_dict (full, dashboard) vs to_llm_content (condensed, LLM context) at LLM_OUTPUT_CHAR_LIMIT; a directory listing is "always condensed (triggering TraceQuery demand)"; trace_query.rs lets the model expand a specific range on demand. THIS IS A STRONG, LIVE ANTI-BLOAT PRIMITIVE: big tool outputs never hit the model whole; the model pulls detail only when it asks.
W5. FSM phase-aware tool/rule loading [WIRED, 3A] - PLANNING-state context != EXECUTING-state context (different slots/tools/rules). I-21 sprint contracts: generator/evaluator zero shared context. This IS the plan-vs-execute split in the product.
W6. BeliefState substrate [BUILT since map: SCHEMA_VERSION 248, BeliefStateEvent Pattern-2 3-arm Condensed/Handoff/HonestyDelta in action_type_lock]. belief_state.rs = 437 LOC. handoff_digest.rs = 526 LOC (H-51 built).

THE REAL REMAINING GAPS (grounded):
G1 [#1 LEVERAGE] J-34 calibration ABSENT. context_calibration.rs does NOT exist. recalibrate_budget(profile) exists at context_budget.rs:849 as the HOOK, but nothing feeds it a MEASURED curve. So the trim fractions (0.75 local / 0.85 cloud at context_policy.rs:65/70) and the slot ORDER are still FOLKLORE constants, not measured per-(model,lane) knees. Every other context mechanism is downstream of these guessed numbers. Grounding for "guesses are wrong": RULER arXiv:2404.06654 + Chroma context-rot (models degrade well before the nominal window; the knee is model-specific). This is the single highest-leverage build.
G2 adaptive_reduce (H-6 3-level overflow ladder: FilteredByRelevance -> ReducedGranularity -> ReducedScope) has ZERO production callers - only context_budget_integration.rs / context_budget_contract.rs (tests). compute()+update_breakpoints() ARE wired, but the actual RELEVANCE-based reduction ladder is not invoked on a live overflow; overflow falls through to conversation.rs trim/compaction (coarser, less relevance-aware). Wiring this is pure activation of shipped code.
G3 Local sidecar KV bypass: server.py:1819 PrefixKVCache is bypassed on the stream:true path the live client uses -> local prefix caching is a NO-OP. Current mitigation = H-50 Local window-minimization (avoid the cost rather than cache it). The real fix = J-19 vLLM backend (automatic prefix caching, cross-request KV reuse) - PLANNED, the lone "REPLACE" verdict, handled additively.
G4 I-11 BeliefState built but the escalation/handoff CONDENSE call-site is not visible in a production grep (escalation_policy / cloud_mode did not show belief_state.rs callers). So a local->cloud escalation may still carry raw-ish history rather than the typed ~2K belief. Needs the call-site wired (the substrate is done).
G5 I-5 pristine context injector PLANNED in the PRODUCT runtime. The HARNESS already hard-excludes failed-sibling tracebacks (auto-orchestrate scoped-context rule), but the product's own context_budget assembly does not yet hard-exclude failed-path noise. This is exactly the user's "no bloat from searching/finding the wrong things."
G6 E-25 compaction ladder DARK x9; the LOCAL-lane RLM _compact_history recipe (J-76) PLANNED.

CROSS-CUTTING TRUTH: the DESIGN answer to the user's question is essentially complete and, in several places, ahead of SOTA harnesses (superpowers/deer-flow verdicts: CONFIRMATION, ELAI stronger). The remaining work is ACTIVATION + MEASUREMENT + 3-4 call-site wires, NOT new architecture. The one genuinely missing LAYER is measurement (J-34): until the curves are measured, the whole stack optimizes against guessed thresholds.

=========================================================
PART 3 - GROUNDED IMPROVEMENTS (ranked by leverage)
=========================================================
Framing: the user wants (a) plan/reason/execute always split across contexts, (b) every model (small or cloud) always holds the IMPORTANT context with NO bloat, (c) no half-blind fixes (know everything the code touches), (d) trade time for perfection but stay fast within rate limits via concurrency. Each improvement below is grounded in code we read or a verified source, and tagged ACTIVATE (wire shipped code) / MEASURE (build the calibration) / BUILD (genuine net-new) / DESIGN (a structural idea to attack adversarially before acting).

IMP-1 [MEASURE, #1 leverage] Build J-34 context-calibration so trim + slot-order are MEASURED, not folklore.
  Why: G1. Today the 0.75/0.85 trim and the slot ordering are guesses; RULER/Chroma show the real degradation knee is model-specific and usually well below the window. `elai bench context-fill` measures per-(model,lane) fill->verified-outcome curves on deterministically-padded fixtures (gate verdicts, never LLM-judge), writes fill_degradation_curve / calibrated_trim_fraction / positional_value_curve into ModelCapabilityProfile, and feeds recalibrate_budget:849 (the hook is already there). The positional curve also reorders the 3E slots by measured value (layout fixed per model at calibration time = cache-safe, HD-2). This converts the ENTIRE stack from guessed to calibrated. Local Qwen-7B and cloud Sonnet get DIFFERENT measured knees - the regime split finally has real numbers.
  Cost/risk: one bench harness + ~250 LOC module; needs the remote 3090 for local-lane curves; OBSERVE-ONLY first, never auto-tightens without >=3 runs (GD-4).

IMP-2 [ACTIVATE] Wire adaptive_reduce into the live overflow path (close G2).
  Why: when a turn exceeds trim_threshold, run the shipped 3-level RELEVANCE ladder (FilteredByRelevance -> ReducedGranularity -> ReducedScope, file_overlap_score sort, SSB context_reduction) BEFORE falling back to lossy conversation-level compaction. This keeps the IMPORTANT context (high-overlap SSB rows, current-file+direct-deps code) and drops the irrelevant - exactly "important context, no bloat." Pure activation: the code exists and is unit-tested; it just needs the production call-site at the conversation/turn-prep overflow branch. Pairs with the Local regime (window-minimization) so the local lane reduces by RELEVANCE not by blind truncation.

IMP-3 [ACTIVATE] Wire I-11 BeliefState into local<->cloud escalation + cross-tier handoff (close G4).
  Why: directly serves "small models always have the important context, no bloat." On a local fail or a tier handoff, condense {tried / failed-because / current-AST-state / constraints, DKG-keyed} into a typed ~2K BeliefState instead of shipping >=20K of raw transcript (I-7 EscalationCascade condense-on-escalate). The substrate is BUILT (schema 248, belief_state.rs 437 LOC); the gap is the escalation_policy/cloud_mode call-site + the honesty measure (periodically re-run from full history vs BeliefState, journal the delta). This is the single biggest "don't bloat the expensive cloud call / don't starve the cheap local call" win.

IMP-4 [BUILD/ACTIVATE] Pristine context injector (I-5) in the PRODUCT runtime (close G5).
  Why: the user's "no bloat from searching/finding the WRONG things." The harness already hard-excludes failed-sibling tracebacks for workers; bring the same rule into context_budget assembly: a failed exploration / wrong-path tool burst becomes a contrastive CONSTRAINT ("don't do X, it failed because Y") not 5K of raw error text. Compose with J-78 context-access spec (a declared, journaled per-dispatch "what this call MAY read") so what enters each model call is deterministic and auditable, not whatever accumulated.

IMP-5 [ACTIVATE/BUILD] Know-everything-it-touches BEFORE a fix: J-16 blast-radius gate + J-38 mid-task retrieval loop + sufficiency gate.
  Why: the user's explicit "fix smt without knowing all that that code touches." J-16 (user's own #1 priority) makes the SHIPPED B-29 impact API an ENFORCED auditor gate (reverse-dep cone + diff-scope + regression-test selection + silent-deletion flag) - the fix cannot pass if it ignored the blast radius. J-38 exposes the hardened hybrid retrieval as a MID-TASK callable tool the executor loops on (retrieve-as-you-go), plus a deterministic context-sufficiency gate (symbol-coverage over the IntentPair) that refuses to start editing until coverage is met. Net: the model gathers exactly the touched surface, no more (anti-bloat) and no less (anti-half-blind). Grounded: SweRank/SpIDER verified; B-29 impact API shipped.

IMP-6 [DESIGN -> then BUILD] Concurrency within rate limits: role-routed lanes + multi-key rotation + mid-task retrieval, not bigger single calls.
  Why: "as fast as possible considering provider rate limits, use concurrency." The grounded levers, composed: (a) H-7 calibrated role-routing (architect/editor/verifier) so cheap models do cheap context and only hard sub-tasks escalate - parallelism across ROLES, not one giant call; (b) J-54 free-API rotation pool (per-key UTC-day quota + 429 header-priority cooldown + multi-key rotation + keyless overflow) = MORE concurrent lanes without tripping limits, with the verification-lane trust-marker keeping the spine honest; (c) keep parallel_cap=3 as the HARD provider ceiling but fill it with the cheapest-tier-that-fits ladder (haiku bulk-read / sonnet agents); (d) H-5 batch lanes (flat 50%) for non-interactive work (cosmos/research/bench). Batch-RACING stays REJECTED (IC-1). The time-for-perfection trade is bounded by: smaller calibrated context (IMP-1) = faster per call; relevance-reduce (IMP-2) = fewer tokens; belief-condense (IMP-3) = cheaper escalation; so "perfection" does not mean "slow."

IMP-7 [DESIGN] Make the plan/reason/execute split STRUCTURAL via the I-1 directive-supervisor plane.
  Why: today the split is enforced by FSM phase-loading + sprint-contract zero-shared-context (strong, but policy). I-1 makes it physical: the generator emits pure Intent->Directive and CANNOT reach gate/side-effect code; a process-separated Supervisor is the only executor and holds credentials. This is the strongest form of "plan and execute never happen in the same context" - the planning model literally cannot execute. Adversarial check before building: confirm the local lane's latency budget tolerates the extra process hop (Tier-C IPC is 15-msg postcard, <10MB frames - already measured cheap), and that the directive schema is expressive enough for the editor role without leaking execution detail back into the generator context.

IMP-8 [ACTIVATE] Local prefix-cache: prefer J-19 vLLM backend over un-bypassing the HF sidecar cache (close G3).
  Why: HF sidecar PrefixKVCache is bypassed on the streaming path and the H-50 mitigation is to AVOID re-sending (window-min) rather than cache. The durable fix is vLLM as backend #12 (native /v1/messages = zero Rust client change, automatic prefix caching = the cross-request KV reuse HF lacks). Until then, IMP-1's measured local knee + IMP-2 relevance-reduce keep the local window small enough that the missing cache hurts less. Note: vLLM swap stays a USER decision (additive, no HF removal without confirm).

SEQUENCING (cheapest-first, each independently valuable):
  Wave A (pure activation, days): IMP-2 adaptive_reduce wire; IMP-3 BeliefState escalation wire; IMP-5a J-16 gate enforce.
  Wave B (measurement, needs remote GPU): IMP-1 J-34 calibration -> feeds everything.
  Wave C (build): IMP-4 pristine injector + J-78 access spec; IMP-5b J-38 retrieval loop + sufficiency gate; IMP-6b J-54 rotation pool.
  Wave D (structural, design-first): IMP-7 directive-supervisor; IMP-8 vLLM local cache.

HONEST RESIDUE / WHAT I DID NOT VERIFY:
- I did not run anything (main-LLM-never-tests; these are reading-grounded, the auditor is the test authority). Statuses are from reading the live .rs files + greps on 2026-06-25; the feature map (06-23) lags reality (H-50/I-11/H-51 are built since it).
- I did not read the context-compiler AGENT file (house rule: main LLM never reads .claude/agents/*.md); the worker-scoping doctrine is taken from the orchestrate/auto-orchestrate skills, which I did read.
- The "wired vs dark" calls on adaptive_reduce/BeliefState are from caller greps; an auditor wire-check (production_caller_check.py) is the authority before any of these are declared done.
