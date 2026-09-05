# I-plan — Summary Index of All 54 sessions (Tracks 12-20)
Generated: 2026-06-10 (Tracks 12-18 authored in one wave from the docs/prefableresearch corpus; Track 19
I-41..I-44 added the same day from workflow/research/grok-elai-trust/ + the token-efficiency plan — the
genuinely-uncovered residue after cross-referencing those three docs against B..I; see I-0_INDEX Track 19 note;
Track 20 I-45..I-54 added the same day from docs/.source/STEAL-LIST.md — the Tier-1/Tier-2 competitor-steal
residue, grounded via _grounding/I-STEAL-GROUNDING-MAP.md; see I-0_INDEX Track 20 note + declines).
Mirrors H-SUMMARY_INDEX format.

Each entry preserves the track + lever, the grounding strength, the nature (ACTIVATION / BUILD /
BUILD-on-shipped / EXTENSION / REFACTOR), the wire-dark or shipped ELAI substrate it activates (or the absent
code it builds, by grounding-map ID), the prior-plan artifact it COMPOSES-WITH (GD-1), the production
integration point, and the observable_effect that is the wiring done-gate (W-1). Doctrine + waves + the
ID-1..ID-6 delta family: I-0_INDEX.md. Grounded symbol map (W-7): _grounding/I-GROUNDING-MAP.md (IS/IN/IC
families; live baseline 221/532 at 2026-06-10 — re-read at edit).

## Premise
Track 12 moves enforcement OUT of the generator's reach and gives parallel work isolated workspaces; Track 13
turns the LLM into a stateless hypothesis generator under an orchestrator-owned, receipts-ranked search
ladder; Track 14 makes the cloud lane cache-invariant and the middleware journaled so deep local validation
stops paying the cache/rate-limit tax; Track 15 composes the shipped verification substrate into an in-loop,
anti-gaming gate stack cheap enough to veto per hunk; Track 16 routes/budgets/measures on ELAI's own ledger;
Track 17 distills search compute into local weights behind blocking ledger gates; Track 18 closes the
remaining prompt-shaped control planes with deterministic mechanisms the 2025-26 literature shows prompts
cannot deliver; Track 20 ports the short list of genuine gaps three competitor repos expose (one goal-hash
trust steal, dispatch/tool-plane discipline, three provider protocol shapes, and three shipped-substrate
activations) after rejecting the wholesale provider-layer port. Differentiator vs B..H: the I-plan is the first letter authored AFTER the wiring bedrock was
reached (wire-remediation 2026-06-07: no clean flips remain), so every session is genuine feature engineering
over a substrate whose wiring discipline is already enforced — and three binding correction layers (IC-1..18)
from an adversarial critique + two competitive reviews are baked into the scopes.

================================================================================
==== TRACK 12 — FAULT-ISOLATION + ENFORCEMENT-PLANE ====
================================================================================

### I-1 — directive supervisor enforcement plane
- Track 12 | lever CANT-WRECK | grounding STRONG | nature BUILD | effort L | WAVE 2
- Steal/build: Erlang/Jido actor discipline + the 2026 incident record (an agent disabling its own sandbox;
  in-process PreToolUse is a convenience layer, not a boundary — infrastructure dossier §4.1). The generator
  emits PURE Intent->Directive structs; a process-separated Supervisor is the ONLY executor of side effects,
  runs every gate first, holds the credentials, and the generator cannot reach gate code or the egress path.
  Saga-style compensation on crash restores pre-tool state. Gap: no Directive type, no supervisor boundary
  (IN: hooks run in-process; IS-19 doctrine assumes it).
- COMPOSES-WITH: Phase 4A IPC (the process seam EXISTS — reuse, do not re-invent), F-1 VibeGate + E-30 argv +
  G-3 blast-radius (they become supervisor-plane gates), E-44 saga compensation. Restates none.
- Integration: task_runner dispatch + hook_dispatcher seams; new crates/orchestrator/src/directive.rs;
  the "generator cannot author/reach gate code" invariant gets a contract test (mirrors E-33 stub_ban shape).
  frontend_pair: supervisor verdict rows in an existing drawer.
- observable_effect: an injected OOM/panic during a tool call restores the exact pre-tool state with a
  DirectiveExecuted(CompensationFired) row; a generator-authored attempt to modify gate code or call a sink
  directly is REFUSED at the supervisor with the full directive logged; missing row = FAIL (R1).

### I-2 — cow workspace isolation
- Track 12 | lever CONCURRENCY | grounding STRONG | nature BUILD | effort M-L | WAVE 2 | ID-2
- Steal/build: Pregel-style branch isolation. Parallel branches/agents get isolated workspaces — Linux
  overlayfs/btrfs reflink; Windows/macOS pooled WARM git worktrees with shared target-dir (IC-9 corrected:
  no cross-platform CoW assumption); merges happen ONLY at an explicit deterministic barrier, ordered by SSB
  seq. A branch escaping its workspace is a BLOCK. Cloud-sandbox adapter (E2B-class) optional behind the same
  trait, never a dep.
- COMPOSES-WITH: Phase 3D BSP barriers + IS-9 recover_from_checkpoint, E-15 Firecracker pool plan, C-11
  sandbox, C-23 branch UI, H-49 RV-1 divergence guard (restore semantics).
- Integration: new crates/orchestrator/src/sandbox/branch_workspace.rs (trait + overlayfs + worktree-pool
  impls); BSP aggregator merge site; frontend_pair: branch workspace badges on BranchTree.
- observable_effect: two parallel branches touching adjacent code run in disjoint paths (verified by
  per-branch root prefixes in the journal), the barrier merge resolves a seeded conflict deterministically
  (BranchWorkspaceEvent rows Spawned/Merged), and an escape fixture (absolute-path write outside the root) is
  BLOCKED with EscapeBlocked; same-seed re-run merges byte-identically (GD-2).

### I-3 — ssb throughput + atomic checkpoint saves
- Track 12 | lever PERF | grounding STRONG | nature EXTENSION | effort S | WAVE 1
- Steal/build: harden the single-writer for search-era write rates: confirm/extend pragmas (busy_timeout,
  cache_size, wal_autocheckpoint per Phase 3B spec), add .tmp+fsync+rename ATOMIC writes at every checkpoint
  site, and MEASURE insert throughput into a metrics TOML (no paper number claimed — ID-5). io_uring is a
  Linux-only opt-in experiment, allowlisted, never the default path.
- COMPOSES-WITH: Phase 3B SSB (pragmas exist — verify, don't re-declare), 3C checkpoint atomic-write spec
  (some sites already atomic — close the rest), H-44 durability. Restates none.
- Integration: ssb.rs write loop (pragma block), checkpoint write sites (orchestrator + session_dag);
  frontend_pair: N/A (throughput lands in metrics TOML; W-6 justified — no live surface).
- observable_effect: a kill -9 mid-checkpoint leaves either the old or the new complete file (never a torn
  one) on a seeded crash fixture; measured inserts/sec recorded in metrics/phase_I3.toml from a real run;
  the no-atomic broken arm produces a torn file and FAILS (R2).

## Track-12 cross-cutting
The supervisor boundary (I-1) is the precondition every later "the generator cannot game X" claim leans on;
I-35's sequence rules and I-31's reward endpoint both live on the supervisor side. ID-2 makes wide search
(I-4) safe to parallelize.

================================================================================
==== TRACK 13 — EVIDENCE-RANKED SEARCH ====
================================================================================

### I-4 — evidence-ranked search ladder [TRACK 13 KEYSTONE]
- Track 13 | lever SMARTER-ORCH | grounding STRONG | nature BUILD | effort XL (tiered) | WAVE 4 | BLOCKING
  for I-22/I-28/I-30 | depends I-2 + I-17 | ID-1/ID-2/ID-4
- Steal/build: nothing public ships tree search — what ships is parallel agents + an LLM-judge selector;
  ELAI selects BY RECEIPTS (IC-6). Ladder: v1 best-of-N (N candidates in isolated workspaces -> tri-layer
  veto + budgeted mutation + progress score produce a RANKING; the SSB records WHY a branch won); v2
  tournament; v3 PUCT tree over SessionDag with virtual loss for parallel expansion. The generator proposes
  K divergent candidates (C-26 temperature schedules); the orchestrator owns the tree. Gap: no search module
  (IN-1).
- COMPOSES-WITH: SessionDag/H-49 (fork/undo-tree = the branch store), I-17 stack (the ranker), I-20
  (mutation receipts), I-2 (workspaces), C-26 temps, D-3 verifier consensus. Restates none.
- Integration: new crates/orchestrator/src/search/ (ladder.rs now; mcts.rs at v3); replaces the linear
  retry at the task_runner baby-step dispatch ONLY behind the ladder's measured win (GD-4); frontend_pair:
  branch board (H15-1 comparison view consumes SearchNodeEvent rows).
- observable_effect: a real task fans to 3 candidates; one fails compile (HardVeto -> Pruned row), survivors
  are RANKED with receipts ("branch 2: 9/9 mutants killed, progress 0.71") in SearchNodeEvent(Promoted); the
  winner merges; an --llm-judge-only broken arm picks a vetoed branch and FAILS (R2/ID-4); v1 ships before
  any tree code lands.

### I-5 — pristine context injector
- Track 13 | lever TRUST-STEERING | grounding STRONG | nature BUILD | effort M | WAVE 2 | ID-1 instrument
- Steal/build: vividness/attention-sink discipline: when expanding a branch, context is rebuilt from the
  parent chain; failed siblings + raw tracebacks are HARD-EXCLUDED; the negative signal survives only as a
  compressed contrastive constraint ("attempt A failed on line-42 type X — avoid X"). AND the claim is
  MEASURED: a sampled A/B (pristine vs accumulated context on the same decision) via the E-19
  reasoning-quality probe, delta ledgered — the first quantified vividness tax (review 1.4-4).
- COMPOSES-WITH: H-6 masking + H-50 regime policy (shared machinery — extends, does not re-mask), E-19
  probe, I-4 (per-branch prompts). Restates none.
- Integration: context assembly seam (context_budget + the prompt builder); pristine_path_filter; frontend_pair:
  vividness-tax tile in the ReasoningDrawer.
- observable_effect: a retry branch's outbound payload contains ZERO bytes of the prior 5K-line traceback
  while carrying the <=30-token contrastive constraint (verified from the journaled request); ContextPruned
  rows land; the A/B probe writes a measured delta recomputable under ELAI_SEED (GD-2); the
  --carry-tracebacks broken arm reinflates and FAILS (R2).

### I-6 — anchor pane slot
- Track 13 | lever STEERING | grounding STRONG | nature BUILD | effort S | WAVE 2
- Steal/build: lost-in-the-middle defense: an immutable PrivilegedAnchor slot appended LAST (after the final
  cache breakpoint — HD-2-safe) carrying the TaskSpec, unresolved claims, and active constraints; the model
  must emit a short acknowledgement tag reflecting the active constraint before generating (recency-bias
  grounding). Gap: no anchor slot (IN-3; ContextSlot has 9 slots, IS-6).
- COMPOSES-WITH: Phase 3E/H-2 ContextSlot allocator (adds the slot, does not rebalance others), H-31 Memory
  slot precedent (the lazy-ALTER + slot-add pattern). Restates none.
- Integration: context_budget.rs ContextSlot (+AnchorPane LAST) + assembly; reuse ContextReduction rows
  (NO new action_type — I-0 table); frontend_pair: N/A (prompt-internal; W-6 justified).
- observable_effect: on a 25K+ token task the assembled prompt's final block is the anchor (verified from the
  journaled request bytes) and the reply opens with the acknowledgement tag; constraint-adherence on a seeded
  long-context fixture improves vs the --no-anchor arm (measured, not asserted — ID-5); cache prefix stability
  assert still holds (HD-2).

### I-7 — cascading escalation + offline batch
- Track 13 | lever CHEAPER-SPEED | grounding STRONG | nature BUILD | effort M | WAVE 4 | depends I-11 | IC-1
- Steal/build: the CORRECTED I-7 (the draft's speculative racing vs the Batch API is impossible — IC-1):
  a strict waterfall — wide cheap branches on the LOCAL lane first; deterministic checks locally; only on
  local failure does the orchestrator CONDENSE the failure set into the I-11 BeliefState and escalate
  SYNCHRONOUSLY to the cloud architect. The Batch API stays where H-5 put it (offline lanes).
- COMPOSES-WITH: H-5 batch (offline; restates none), H-7 architect/editor routing (the escalation target),
  I-11 BeliefState, E-39 cost gate. 
- Integration: the routing seam (cloud_mode/escalation_policy); EscalationCascade rows; frontend_pair:
  cascade trace in the cost cockpit (H-9 tile family).
- observable_effect: a fixture task the local lane can solve never leaves the machine (LocalWin row, zero
  cloud spend in the journal); a local-fail fixture escalates ONCE with a <=2K-token condensed payload
  (Condensed -> Escalated rows; payload size from the journal); an interactive-path batch route is
  structurally impossible (the broken arm asserting batch-on-interactive FAILS — R2).

### I-8 — aletheia voluntary abort
- Track 13 | lever CANT-LIE | grounding STRONG | nature BUILD | effort M | WAVE 1
- Steal/build: an explicit acknowledge_infeasible/abort_trajectory tool in the tool schema; calling it is
  CREDITED by the abstention-aware score (H-39/RD-1 — composes, does not re-score); the orchestrator rolls
  back to the last good trunk node (H-49). Dead-end hashing: the AST/approach hash of an aborted path is
  cached; an isomorphic future branch is pruned BEFORE an LLM call. Plus the empirical halting bound
  (deterministic checks spinning >15s = abort + penalize) pulled forward from I-17's spec.
- COMPOSES-WITH: D-16/D-17 refusal + SLA (the abstention substrate), H-39 AbstentionScored, H-49 rollback,
  E-35 tool schema. Restates none.
- Integration: tool catalog 3-arm add (IS-14); search/retry loop hook; dead-end cache keyed blake3(AST shape);
  frontend_pair: abort/dead-end strip in ReasoningDrawer.
- observable_effect: a structurally-impossible fixture task yields TrajectoryAborted(Voluntary) + a rollback
  to trunk (journal-verified) instead of a spiral; a re-attempt of the isomorphic approach is pruned with
  DeadEndHash and ZERO model calls (token count from the journal); the halting bound fires on a seeded
  spin fixture (HaltingBound row within budget).

### I-9 — error amplification telemetry
- Track 13 | lever PROVABLE | grounding STRONG | nature BUILD | effort M | WAVE 4
- Steal/build: measure alpha_t (does a corrupted step amplify or contract downstream error?): an
  ErrorPropagationHarness corrupts step t on a frozen fixture pipeline (C-28 mutation operators as the
  corruption source) and measures recovery; active annealing (drop temperature + tighten grammar) when the
  measured alpha exceeds 1 mid-run. Per ID-5: ELAI's OWN alpha baseline is measured before any "contraction"
  claim; no literature number is a target.
- COMPOSES-WITH: C-28 mutation operators, C-26 temperature schedule, DriftComparator first-divergence shape
  (H-23 cite), I-4 (the trajectories measured). Restates none.
- Integration: crates/orchestrator/src/benchmarks/error_amplification_runner.rs (seccodeplt-runner shape,
  include_str! fixtures); `elai bench error-amplification`; frontend_pair: alpha trend tile (cockpit).
- observable_effect: the bench emits ErrorAmplificationMeasured with alpha computed from fixtures,
  byte-recomputable under ELAI_SEED (GD-2); the --no-repair broken arm yields alpha>1 while the repaired arm
  contracts (proves the metric discriminates — R2); annealing fires on a live alpha>1 fixture run and the
  journal shows the temperature drop.

## Track-13 cross-cutting
ID-1 and ID-4 are the spine: the generator proposes, the orchestrator owns state, receipts rank. I-5
instruments the doctrine before the tree exists; the ladder (I-4) never ships a rung whose selection isn't
journal-explained.

================================================================================
==== TRACK 14 — CACHE-INVARIANT ECONOMICS MIDDLEWARE ====
================================================================================

### I-10 — prefix invariance + ttl tiering + cache-hit hard guard [TRACK 14 KEYSTONE]
- Track 14 | lever CHEAPER | grounding STRONG | nature BUILD-on-shipped | effort M | WAVE 1 | IC-4/IC-5
- Steal/build: byte-frozen prefix discipline end-to-end: no timestamps/UUIDs/mutable endpoints in the prefix;
  dynamic tool restriction via tool_choice/allowed_tools payload attributes, NEVER prompt-text mutation; the
  tool-array question (inline-frozen vs H-4 deferral) is decided per catalog size by MEASURED cache-hit x
  token cost (IC-5). TTL tiering: 1h on the frozen prefix breakpoint, 5m on volatile tails, chosen by a
  cache-economics planner reading inter-call gap stats from the journal (2026-03-06 landmine #1). The
  hit-rate HARD GUARD: unexplained miss-rate >10% on identical task classes HALTS and dumps a blake3 prefix
  diff naming the leaking mutation. Burst-amortized scheduling for long local validation loops (IC-4);
  heartbeat demoted to last-resort.
- COMPOSES-WITH: H-1 markers + H-2 breakpoints/bp.valid + H-9 hit-rate instrument (IS-5/IS-6 — if H-1/H-2
  shipped first, I-10 is the TTL/planner/guard layer ONLY; re-grep at edit), H-4 tool search, E-39 cost gate.
- Integration: providers prompt assembly + anthropic_api marker sites (IS-5); new cache_economics.rs
  (planner + guard); frontend_pair: TTL/economics fields on the H-9 cost tile.
- observable_effect: a real two-turn cloud task shows turn-2 cache_read covering the frozen prefix with the
  planner-chosen TTL recorded (CacheEconomicsEvent TtlPlanned, recomputable GD-2); a deliberately injected
  prefix timestamp trips the guard within ONE session (HitRateGuardTripped + PrefixDiffDumped naming the
  byte range) — the March-6-class silent economics drift is caught by the harness, not the invoice.

### I-11 — pabu belief-state handoff
- Track 14 | lever CHEAPER-ORCH | grounding STRONG | nature BUILD | effort M | WAVE 2 | ID-3 | IC-8
- Steal/build: a TYPED, versioned BeliefState (tried/failed-because/current-AST-state/constraints, DKG-keyed
  entities) replacing raw history on every cross-model or cross-tier handoff (local->cloud escalation,
  architect->editor, session resume). Differentiator vs commodity memory layers (IC-8): typed + DKG-linked +
  journaled (ID-3 receipt) + HONESTLY MEASURED — periodically re-run a task from full history vs BeliefState
  and ledger the success delta ("we know what our compression costs").
- COMPOSES-WITH: H-7 routing (the handoff sites), H-51 digest (cross-session sibling — shares the digest
  engine, scopes differ), H-6 recover_stubs (detail re-expansion), DKG. Restates none.
- Integration: new crates/orchestrator/src/belief_state.rs (schema versioned); escalation + handoff call
  sites; frontend_pair: BeliefState inspector row in SessionsDrawer.
- observable_effect: a local->cloud escalation carries ONLY prefix+BeliefState (journal-verified payload
  <=~2K tokens vs the >=20K raw history control); BeliefStateEvent(Condensed/Handoff) rows land with the
  schema version; the honesty job emits HonestyDelta with the measured success-rate difference under
  ELAI_SEED (GD-2); a schema-version mismatch on read fails CLOSED (typed error, not silent parse).

### I-12 — evidence-gated playbook curator
- Track 14 | lever STEERING | grounding STRONG | nature BUILD | effort M | WAVE 3 | ID-6
- Steal/build: ACE-shape playbook curation with RECEIPTS (the field shipped ACE + "Dreaming"; ELAI's edge is
  evidence-gating — review 1.5): a playbook/rule entry PROMOTES only when linked to >=k SSB-verified
  successful outcomes, carries its measured lift (cheap interleaved on/off injection), DEMOTES by the same
  ledger, and merged rules must NOT regress the C-28 mutation suite (DSPy-style metric-driven validation).
  Export verified entries to the formats the world consumes (CLAUDE.md/AGENTS.md/skills) — learned content
  interop.
- COMPOSES-WITH: Phase 3F rules lifecycle (PROPOSED->SHADOW->ACTIVE — the promotion machinery EXISTS; the
  curator feeds it), C-25 TemplateCrystallizer (success-count precedent), E-8 GEPA (the optimizer lane),
  C-21 self-mod data tables, F-5 staleness demotion. Restates none.
- Integration: new crates/orchestrator/src/rule_curator.rs; offline tick (Layer-1A schedule); skills export
  script; frontend_pair: playbook panel rows (RulesDrawer).
- observable_effect: a rule with >=k verified-outcome links promotes with PlaybookEvent(Promoted) carrying
  the outcome row ids (recomputable GD-2); a rule whose merged form regresses the mutation suite is REFUSED
  (ValidationRun fail); a stale entry demotes; `elai playbook export` writes an AGENTS.md block whose every
  entry cites its evidence rows; zero-evidence promotion attempt FAILS (R1).

### I-13 — memory guard + dual serialization
- Track 14 | lever SECURITY-EFFICIENCY | grounding STRONG | nature BUILD | effort M | WAVE 2 | IC-10
- Steal/build: (a) a deterministic write-time scan on ALL Recall/Archival/playbook memory writes (injection
  patterns, invisible-Unicode/multi-byte tricks, secrets) — regex/aho-corasick first, a YARA engine only as
  a user-approved dep (IC-10); (b) OpenHands-style dual serialization: to_ui_dict (full, dashboard) vs
  to_llm_content (condensed, context) on tool results; (c) a TraceQuery tool so the model can expand exact
  line ranges from disk on demand (prevents condensation starvation).
- COMPOSES-WITH: memory crate MINJA defenses + H-31/H-32 auditor-objectivity firewall (extends; never
  re-filters), E-37 PromptArmor (data-plane classify), G-9 redaction (write-time precedent), H-6 masking.
- Integration: memory write choke-points (mem0_dispatcher/store seam) + crates/tools tool_result.rs;
  TraceQuery via the 3-arm catalog pattern; frontend_pair: MemoryFactsDrawer guard badges.
- observable_effect: a seeded malicious memory write (invisible-Unicode injection fixture) is BLOCKED with
  MemoryGuardEvent(PoisonBlocked) naming the pattern class; a 500-file directory listing reaches the
  dashboard complete but enters context as the condensed form (journal shows the token delta,
  DualSerialized); a TraceQuery expands the exact requested range (TraceQueryExpanded); the guard's FP rate
  is bounded on >=3 real runs before default-ON (GD-4).

### I-14 — deterministic context compression
- Track 14 | lever CHEAPER | grounding STRONG | nature BUILD | effort M | WAVE 2 | ID-3 | IC-2/IC-3
- Steal/build: the corrected compression stack: Tier-1 DETERMINISTIC — headless-LSP slicing (extract exactly
  the AST node/declaration/failing line a compiler error references) + tree-sitter AST folding (bodies of
  irrelevant functions -> "/* elided */", signatures/types intact — BPE priors preserved); Tier-2 (gated,
  prose-only payloads) LLMLingua-2-class token classification with a VRAM budget check; cipher/abbreviation
  substitution BANNED (IC-3). EVERY transform lands a reversible ContextTransformReceipt (ID-3) so replay
  reconstructs what the model saw.
- COMPOSES-WITH: E-52 LSP verifier lane (the rust-analyzer headless substrate — shares the client), knowledge
  tree-sitter grammars (IS for folding), H-6 masking (different layer: H-6 drops turns; I-14 compresses
  payloads). Restates none.
- Integration: new crates/orchestrator/src/context_transform.rs hooked at the tool-result -> context seam;
  frontend_pair: per-transform receipt rows in the Transcript drawer.
- observable_effect: a 10K-token compiler panic enters context as a <=500-token slice (journal-verified
  before/after token counts) with a receipt row whose reverse pointer re-expands to the original bytes
  (blake3 match); pass-rate on a fixed fixture set is unchanged vs uncompressed (measured, ID-5); a Tier-2
  attempt without VRAM headroom is REFUSED (fail-closed); zero receipts while transforms ran = FAIL (R1/ID-3).

### I-15 — semantic cache + blake3 fast path
- Track 14 | lever CHEAPER-SPEED | grounding STRONG | nature EXTENSION | effort S | WAVE 1
- Steal/build: cache INTERMEDIATE summaries (DKG chunk summaries, research extracts), not just final answers;
  serve them on semantically-similar queries; an exact blake3 fast path runs BEFORE any vector math (O(1) on
  exact repeats). Confined to IDEMPOTENT grounding content (H-8's confinement rule inherited verbatim).
- COMPOSES-WITH: C-30/H-8 OracleAnswerCache (tier TTLs + persistence — extends the same store; restates
  none), Phase 1E A-RAG (the summarize site), 1G reranker (the vector path it short-circuits).
- Integration: knowledge/src/arag.rs interceptor + oracle_cache extension; reuse OracleCacheHit/Miss rows
  (I-0 table; prefer reuse); frontend_pair: cache-source badge on research answers.
- observable_effect: the same deep-research query twice: run-2 serves the chunk summaries from cache with
  RAG input tokens reduced (journal-measured delta — the % is MEASURED, not the paper's 50-60%, ID-5); an
  exact repeat hits the blake3 path with zero ONNX calls (latency + call-count from the trace); a code-gen
  query NEVER hits the durable cache (confinement guard, inherited test).

### I-16 — kv-cache pruning + quant eval
- Track 14 | lever LOCAL-VRAM | grounding MODERATE | nature ACTIVATION+EVAL | effort M | WAVE 5 | RunPod-gated | ID-5
- Steal/build: sidecar VRAM levers as an EVALUATION, not an adoption: sink-protected KV pruning
  (SnapKV/ThinK-class, first-4-tokens pinned) + Int4 KV quantization, measured on ELAI's own fixtures
  (pass-rate + VRAM + throughput) against the E-27 AWQ baseline. The MoE sidecar retarget
  (Qwen3-Coder-Next-class on 24GB) is a SEPARATE evaluation checkbox — prior model swaps were rejected on
  measurement (memory: baseline_findings), so no swap is assumed (IC-7 note).
- COMPOSES-WITH: E-26/E-27/E-28 (the serving stack + FA2 mandate + LogitsProcessor chain preserved), E-32
  context lift, B-37 sidecar adapter. Restates none.
- Integration: python/elai_inference/server.py flags + an eval harness mirroring E-27's; frontend_pair: N/A
  (bench TOML; W-6 justified).
- observable_effect: `elai bench kv-tune` (RunPod checkbox) writes KvCacheTuned with measured
  {vram_gb, tok_s, fixture_pass_rate} per config; ADOPT only if pass-rate is within the band AND VRAM drops
  — recorded as a decision row; the deterministic fixture harness (no GPU) ships now and exercises the real
  config path (W-10).

## Track-14 cross-cutting
Every number here is HD-1-measured per config and GD-2-recomputable; ID-3 makes the middleware itself
auditable — the replay story survives compression. The cloud lane gets cache-stability; the local lane gets
window-minimization (H-50's split inherited, never re-fought).

================================================================================
==== TRACK 15 — DETERMINISTIC GATES + ANTI-GAMING VERIFICATION ====
================================================================================

### I-17 — tri-layer evaluation stack [BLOCKING for I-4]
- Track 15 | lever TRUST-PROVABLE | grounding STRONG | nature BUILD (composition) | effort M | WAVE 3 | ID-4
- Steal/build: the search-facing verdict stack: L1 HARD VETO (compile/test/F-1 VibeGate/F-9 dep-gate/D-11
  checkers — any fail = score 0.0, supreme, never overridable); L2 calibrated scorer on SURVIVORS only (D-2
  PPM seam when live; PRM-lite/E-19 until); L3 disagreement escalation to the cross-family adversary (D-13).
  Plus: the 15s empirical halting bound on L1 (slow-deterministic = abort branch, I-8 shares it); FLAKE
  QUARANTINE — an oracle that flips on n-rerun under seed variation can neither veto nor vouch until
  rehabilitated (ledgered); a cross-language verifier-pack seam (TS/Python packs behind the D-3 adapter
  trait) noted Tier-2.
- COMPOSES-WITH: D-3 verifier_adapters + consensus, D-11 checker registry, D-13 adversary, F-1/F-9 gates,
  C-28 (via I-20), E-19 PRM-lite. Restates none — this is the composition layer.
- Integration: new crates/orchestrator/src/verifier_stack.rs consumed by I-4's ranker + the baby-step audit
  seam; frontend_pair: per-branch verdict chips (cockpit family).
- observable_effect: a fluent candidate with a hallucinated import scores 0.0 via L1 BEFORE any L2 call
  (TriLayerVerdict HardVeto; L2 call-count 0 in the journal); a survivors-only L2 ordering lands
  (ScoredSurvivor); a seeded flaky test is quarantined (FlakeQuarantined) and its vote excluded; the
  --l2-overrides-l1 broken arm resurrects a vetoed branch and FAILS (R2/ID-4).

### I-18 — upip chain + sigstore + ai-bom
- Track 15 | lever AUDITABLE | grounding STRONG | nature EXTENSION | effort M | WAVE 3 | IC-11
- Steal/build: extend the shipped RRA/attestation surface (IS-10) with: a per-row chain_prev_hash on the SSB
  single-writer (tamper-evident sequencing), a 5-layer hierarchical stack hash (STATE/DEPS/PROCESS/RESULT/
  VERIFY) sealed into the RRA, OPT-IN Rekor transparency-log push (external, offline-verifiable; cosign
  keyless as the opt-in dep — anti-slopsquat + approval), and an AI-BOM/CycloneDX-ML entry alongside the
  attestation so the artifact speaks OWASP AISVS AC.8 (trust-review B7). Local blake3-Merkle stays the
  offline default.
- COMPOSES-WITH: G-2 RRA + G-8 attestation.rs (EXTENDS; restates none), F-4 stdout_hash (a chained field),
  ssb.rs append-only discipline.
- Integration: ssb.rs write loop (chain field — coordinate the golden lock), rra.rs/attestation.rs extension,
  `elai verify-attestation` gains --chain + --rekor; frontend_pair: chain badge on the RRA viewer.
- observable_effect: `elai run --emit-rra` seals a chained bundle; editing ANY mid-run row breaks
  verification at exactly that row (RowChainSealed(ChainBroken) names the seq); the AI-BOM file validates
  against the CycloneDX schema; a Rekor push (opt-in checkbox) yields an inclusion proof verifiable WITHOUT
  re-running ELAI; Cargo.lock mutation mid-run invalidates the DEPS layer (hard halt).

### I-19 — tdd enforcer preflight
- Track 15 | lever CANT-WRECK | grounding STRONG | nature EXTENSION | effort S-M | WAVE 3 | IC-12
- Steal/build: the symbol-level PreToolUse leg ONLY (TESTING_FIRST FSM + E-13 anti-over-mock already exist —
  IC-12): a src-file edit during EXECUTING requires a corresponding test-symbol delta (tree-sitter AST: the
  modified fn has a test referencing it) + a syntax-only dry-run (cargo check --tests / pytest
  --collect-only) before the write lands; pure-structural refactors exempt via a coverage-delta check.
  FP-bounded on >=3 real runs before default-ON (GD-4 — this gate is friction-prone; the bound is the gate).
- COMPOSES-WITH: F-1 VibeGate compound evaluator (the hook seam — adds an axis, does not fork the hook),
  E-13, Phase 4C hook tiers, knowledge grammars.
- Integration: the run_pre_tool_use compound check; frontend_pair: block reasons in the existing gate panel.
- observable_effect: an Edit adding a new pub fn with no test-symbol delta is REFUSED pre-disk
  (TddPreflightBlocked MissingTestDelta); the same edit with a matching failing test lands; a rename-only
  refactor passes via the exemption (RefactorExempted); measured FP rate on 3 real orchestrate runs is in
  the recorded bound before the default flips ON.

### I-20 — budgeted mutation scheduler [TRACK 15 KEYSTONE]
- Track 15 | lever VERIFIED | grounding STRONG | nature EXTENSION | effort M-L | WAVE 3 | ID-5 | IC-14
- Steal/build: make mutation testing live INSIDE the loop: select mutants ONLY on the diff's data-flow cone
  (reachability via the symbol graph/impact_radius), time-box per hunk, SAMPLE mutants and report a
  CONFIDENCE-INTERVAL score with early stop once the CI clears/fails the threshold ("0.82 +/- 0.06" beats an
  exhaustive number that arrives too late); cache results in the SSB keyed (fn content-hash, bound-test-set
  hash) so unchanged code never re-pays (extends the shipped EquivalentMutantCache — IC-14); SURVIVORS
  become structured counterexample Directives fed back to the generator ("your tests do not distinguish >=
  from > at x.rs:412") with bounded retries; property-based relation adapters (proptest/Hypothesis) behind
  the D-3 trait as a parallel lane.
- COMPOSES-WITH: C-28 MutationRunner + cache (IS-8), C-36 MUCOCO operators, impact_radius.py / elai_graph
  reachability, I-17 (consumes the score), I-4 (receipts). Restates none.
- Integration: new crates/orchestrator/src/mutation_budget.rs over mutation_testing.rs; the audit + search
  seams; frontend_pair: per-hunk mutation chip (cockpit).
- observable_effect: a real hunk gets a sampled score with CI within the time-box (MutationBudgetedRun
  CiCleared, params + sample recomputable GD-2); an unchanged fn re-audit is a CacheHit (zero mutant runs,
  journal-verified); a surviving mutant emits SurvivorDirective whose counterexample text names the operator
  + line; thresholds are ELAI-measured baselines, never paper constants (ID-5).

### I-21 — sprint contracts + ralph loop
- Track 15 | lever TRUST-ANTI-BIAS | grounding STRONG | nature BUILD | effort M | WAVE 3
- Steal/build: generator/evaluator separation with ZERO shared context (the +0.12 self-review bias): the
  generator proposes a structured Sprint Contract (goal, files, acceptance, bounded scope); an isolated
  evaluator approves/rejects BEFORE execution; on exit, failed checks REINJECT the trace (bounded Ralph
  loop, <=3); the 3rd failure escalates to the D-13 cross-family adversary for a concrete counter-example.
  Evaluator runs tau=0 with a hostile-critic role — the persona research SUPPORTS personas exactly here
  (alignment-dependent task; IC-17 consistency).
- COMPOSES-WITH: H-37 plan-approval gate seam (run_interactive_gate machinery — the contract rides it),
  H-38 stop gate, C-32 skeleton-first, D-13 adversary, I-1 supervisor (the isolation substrate).
- Integration: new crates/orchestrator/src/sprint_contract.rs + the dispatch seam; frontend_pair: contract
  card (PlanApprovalCard sibling).
- observable_effect: a vague fixture goal is REJECTED by the isolated evaluator pre-execution
  (SprintContractEvent Rejected, with the evaluator's context PROVABLY free of generator turns —
  journal-verified zero shared rows); a failing exit reinjects once (RalphReinjected) and the 3rd loop
  escalates (AdversaryEscalated) producing a counter-example row; approval and execution rows recompute
  under ELAI_SEED (GD-2).

### I-22 — progress-shaped rewards
- Track 15 | lever ANTI-GAMING | grounding STRONG | nature BUILD | effort M | WAVE 4 | depends I-4 | ID-4
- Steal/build: score trajectories by DELTA progress (phi(h_t) - phi(h_t-1), phi from deterministic
  observables: tests passing, compile errors falling, requirements covered) instead of absolute scores that
  inflate; FIRST-ERROR localization truncates the branch at the failing step, salvaging the valid prefix
  (H-49 fork at the truncation point); reward DECAY on no-progress loops breaks reasoning spirals
  (anti-spiral's search-native sibling).
- COMPOSES-WITH: D-2 PpmClient seam (the L2 scorer this shapes), H-49 undo-tree (truncation), I-4 (PUCT
  uses the shaped value), anti-spiral skill (the CLI-side cousin — does not replace it).
- Integration: search/score.rs in the I-4 module; frontend_pair: progress sparkline per branch (branch board).
- observable_effect: a branch that loops on isomorphic states sees its reward decay (DecayApplied rows) and
  the search backtracks (journal shows sibling expansion); a branch failing at step 8/10 truncates to step 7
  with the prefix preserved (FirstErrorTruncated + an H-49 fork node); phi components are all deterministic
  observables enumerable from the journal (ID-4) — an LLM-opinion phi component is structurally absent.

### I-23 — unified conformal calibration
- Track 15 | lever SMARTER-ORCH | grounding STRONG | nature EXTENSION | effort S-M | WAVE 1 | IC-13
- Steal/build: ONE calibration implementation, three consumers: D-16 refusal (shipped), G-7 claim_score
  (planned producer), and the I-4 search prior/uncertainty fork (new). High epistemic uncertainty forces
  exploration forks instead of confident exploitation. No new math — unification of the shipped
  conformal_calibration + calibration_bootstrap (IS-7) behind one module boundary so tau/coverage semantics
  cannot drift between consumers.
- COMPOSES-WITH: D-16/D-30 (the impl), G-7 (the producer spec), H-39 abstention scoring (the credited
  outcome), I-4. Restates none.
- Integration: a calibration facade module re-exporting one impl; call-site migration; frontend_pair: N/A
  (internal unification; W-6 justified — existing refusal surfaces unchanged).
- observable_effect: refusal decisions and search uncertainty forks in one run trace to the SAME tau row
  (journal join on the calibration id — drift structurally impossible); a synthetic miscalibrated arm
  (--split-calibrations) lets the two drift and FAILS the new contract test (R2); D-16's existing
  observable effects keep passing untouched (regression guard).

### I-24 — isomorphic canary anti-gaming
- Track 15 | lever ANTI-GAMING | grounding STRONG | nature EXTENSION | effort S-M | WAVE 4
- Steal/build: inject AST-preserving obfuscated canaries (for<->while, variable renames — semantics
  identical, style perturbed) into the L2 scoring stream; a scorer whose rating drops under perturbation is
  STYLE-BIASED — detected, logged, its weight neutralized until recalibrated. This is the PRM-side
  anti-gaming complement to H-44's candidate-side behavioral traps (composes, never re-implements its
  detector).
- COMPOSES-WITH: C-29 metamorphic runner (the perturbation engine — adds AST-preserving relations),
  H-22/H-44 (perturbation + trap precedents), I-17 L2, D-2 PPM.
- Integration: extend metamorphic_testing.rs relations + a canary injector in the I-17 L2 path;
  frontend_pair: scorer-health badge.
- observable_effect: a seeded style-biased scorer fixture rates the canary pair divergently ->
  PrmCanaryEvent(StyleBiasDetected) + the scorer's weight zeroed for the run (ScorerNeutralized), with the
  pair + scores recomputable (GD-2); an unbiased scorer passes (CanaryInjected only); the canary set is
  FROZEN out-of-band (R8 — the scored model never authors its own canaries).

## Track-15 cross-cutting
The stack exists to make I-4's receipts trustworthy: veto supremacy (ID-4), sampled-CI mutation that fits
inside a loop, contracts that remove self-review bias, canaries that keep the neural scorer honest, and a
chain that makes the whole record tamper-evident.

================================================================================
==== TRACK 16 — ROUTING + BUDGETS + MEASUREMENT ====
================================================================================

### I-25 — ledger-trained router
- Track 16 | lever CHEAPER-SPEED | grounding STRONG | nature EXTENSION | effort M | WAVE 4 | ID-5/ID-6
- Steal/build: routing products train on proxy signals; ELAI's journal holds ground-truth verifier outcomes
  per (task features, lane, cost). Train a SMALL router (features: task class, file count, DKG centrality,
  historical cost shape) on OWN outcomes; execution-cost profiling pre-routes known-expensive AST shapes to
  the deep lane before a cheap-lane failure burns tokens. Router promotions pass the ID-6 ledger (it is a
  learned artifact).
- COMPOSES-WITH: H-7 calibrated routing (the seams + RouteDecisionEvent — wires the dark emitter further;
  GD-1), E-31 tier matrix, C-46 selective delegation, E-20 privacy routing (its mask composes).
- Integration: escalation_policy/cloud_mode seams; offline trainer in workflow/training/router/;
  frontend_pair: route-explanation row on the cost cockpit.
- observable_effect: a real mixed batch routes with RouteDecisionEvent rows carrying the feature vector +
  predicted-vs-actual outcome; measured over-escalation + mis-route rates on >=3 runs beat (or honestly
  fail to beat) the H-7 heuristic baseline — the comparison number is ELAI-measured (ID-5), and a losing
  router is NOT promoted (ledger row shows the block).

### I-26 — dynamic thinking budgets
- Track 16 | lever STEERING | grounding STRONG | nature EXTENSION | effort M | WAVE 4
- Steal/build: budget the SEARCH the way H-42 budgets GRC: serial-depth-scaled compute per node, a collapse
  rule at 85% budget (halt exploration, score frontier, synthesize best), plateau early-stop (L2 scores flat
  across 5 supersteps = collapse early); map cloud reasoning to the EFFORT parameter (Opus 4.7+ rejects
  manual budget_tokens — landmine #6) and never force-JSON a reasoning role (HD-3).
- COMPOSES-WITH: H-42 ReasoningBudgetScaled (REUSES the row; extends payload only if a field is missing),
  E-39 hard cost ceiling (supreme), E-19 confidence gate, I-4.
- Integration: search budget module + the cloud request builder (effort field); frontend_pair: budget bar on
  the branch board.
- observable_effect: a deep fixture task receives a larger node budget than a shallow one (rows show depth +
  budget, same role); an 85% collapse fires on a seeded over-budget run (frontier scored, best synthesized,
  journal shows NO post-collapse expansions); a plateau fixture collapses EARLY saving measured tokens
  (GD-2-recomputable delta); a budget_tokens field on an Opus 4.7+ request is structurally absent (contract
  test greps the builder).

### I-27 — budgeted convergence instrumentation
- Track 16 | lever PROVABLE | grounding STRONG | nature BUILD | effort S-M | WAVE 4
- Steal/build: the public instrument for "more compute -> more verified outcomes": plot verified-outcome
  rate (W-1-grade, not pass@k claims) against total test-time compute (nodes expanded / tokens / $) per
  config; the Pareto cost/accuracy view lets an operator pick an operating point. Monotonicity is MEASURED;
  any non-monotonic region is reported, not smoothed (GD-5).
- COMPOSES-WITH: E-49 metrics tuple ($/CTS, STABLE-Pass), E-9 bench harnesses, I-4 telemetry, H-9 cost
  cockpit. Restates none.
- Integration: crates/cli/src/bench_convergence.rs + a cockpit tile; frontend_pair: the curve tile.
- observable_effect: `elai bench convergence --suite <fixture>` emits ConvergenceCurveMeasured whose curve
  points recompute byte-identically from the journal (GD-2); the report names the knee + the
  cost-per-additional-verified-task; a --skip-verification broken arm inflates the curve and FAILS (R2).

### I-28 — dag credit assignment
- Track 16 | lever SELF-MOD | grounding STRONG | nature BUILD | effort M-L | WAVE 5 | depends I-4
- Steal/build: walk backward from COMPLETE assigning causal credit to DAG nodes (retrieval chunks,
  sub-plans, template choices) on winning trajectories; extract DPO pairs as a byproduct (winner vs the
  highest-L2-scoring VETOED sibling — the perfect "fluent but wrong" negative); demote retrieval chunks that
  consistently precede aborts. Shapley/SHARP credit DECLINED (research-grade; revisit with RL infra).
- COMPOSES-WITH: Phase 4H archive + parent selection (IS-4), H-46 clade metaproductivity (the lineage-aware
  scorer — credit feeds it), I-4 (the trajectories), I-30 (consumes the pairs).
- Integration: new crates/orchestrator/src/dag_credit.rs run at PhaseComplete; frontend_pair: credit
  heat on the plan view (optional, W-6 justified if deferred).
- observable_effect: a completed fixture run emits DagCreditAssigned rows whose credit walk re-derives from
  the journal (GD-2); a DPO pair file lands with winner/loser hashes matching sealed RRA entries; a chunk
  with 3 consecutive abort-preceding appearances is demoted (ChunkDemoted + the retrieval layer's next-run
  ranking provably changed under seed).

### I-29 — diffusion draft lane
- Track 16 | lever GENERATIVE | grounding MODERATE | nature BUILD | effort L | WAVE 5 | new-dep + network
  gated (user approval)
- Steal/build: a dLLM (Mercury-class) as a FAST UNTRUSTED DRAFT GENERATOR behind the full veto stack — the
  trust-preserving consumption of the diffusion speed story (review 1.3-5): drafts/apply-edits stream at
  dLLM speed, then pass the IDENTICAL I-17/I-20 gates as any generator output; CRDT merge semantics
  (Automerge-class or hand-rolled deterministic merge) let parallel sub-agents patch a shared draft without
  races; semantic checkpoints (C-32 MiniCritic per merge). TTD-DR's full denoising-research loop is
  DECLINED; this is the narrow steal.
- COMPOSES-WITH: C-32 MiniCritic, E-48 research orchestrator (the long-form consumer), I-17 (the gates),
  B-37 adapter pattern (the dLLM client is just another RuntimeAdapter).
- Integration: a new RuntimeAdapter variant (user-approved dep + anti-slopsquat first); draft_merge.rs;
  frontend_pair: draft-lane badge on generation cards.
- observable_effect: a draft produced by the dLLM lane is VETOED by the same L1 gate that would veto the
  main generator (DiffusionDraftEvent DraftVetoed — proves gate parity); two parallel patches to one draft
  merge deterministically (CrdtMerged, byte-identical under seed); the lane OFF (flag absent) leaves all
  existing paths untouched; W-2: ships default-ON for drafts only when its corruption rate is bounded
  (GD-4), else stays an allowlisted experiment.

## Track-16 cross-cutting
Everything routes/budgets/measures on the journal: the router learns from receipts (ID-6), budgets follow
measured depth, the convergence curve is the believe-the-number instrument for the whole search bet.

================================================================================
==== TRACK 17 — OPEN-WEIGHT FLYWHEEL ====
================================================================================

### I-30 — verifier-ranked sft exhaust
- Track 17 | lever ALIGNMENT | grounding STRONG | nature BUILD | effort L | WAVE 5 | depends I-4 | ID-6 | IC-7
- Steal/build: the search compute already paid for produces the dataset: extract winning paths (root ->
  verified COMPLETE) from SEALED RRAs into SFT JSONL; record negative pairs (the highest-L2-scoring branch
  that FAILED a deterministic check — the canonical fluent-but-wrong); rejection-sampling SFT (LoRA on the
  canonical local model) FIRST — most of the distillation gain, none of the RL instability (IC-7). Dataset
  rows carry provenance hashes back to their RRA.
- COMPOSES-WITH: G-2 RRA (the sealed source — only sealed runs are eligible), I-4/I-28 (trajectories +
  pairs), E-55 SFT-on-rollouts (the plan this implements the substrate for), D-6 held-out (eval), H-17/H-18
  ledger (promotion).
- Integration: crates/cli/src/sft_exhaust.rs (`elai dataset export --format sft`); training scripts under
  workflow/training/ (offline, RunPod for the run); frontend_pair: dataset card (counts + provenance) in a
  drawer.
- observable_effect: `elai dataset export` emits SftExhaustEvent rows + a JSONL whose every record's
  provenance hash verifies against its RRA (a tampered record fails verification — R2); negative pairs
  reference real vetoed branches; the LoRA training run itself is a RunPod checkbox; any resulting
  checkpoint enters PROMOTION ONLY through the ID-6 ledger gates (the promotion row cites trial-N +
  deflation + held-out results).

### I-31 — grpo flywheel [LAST; BLOCKING GUARDS]
- Track 17 | lever SELF-IMPROVEMENT | grounding STRONG | nature BUILD | effort XL | WAVE 5 | LAST | ID-6 |
  IC-7 | depends I-30 + H-43/H-44/H-28 + I-24
- Steal/build: GRPO on EXECUTION-GROUNDED rewards (compile/test/mutation/null-test gates; format penalties
  via XGrammar conformance) served over a typed local reward endpoint; KL penalty against mode collapse;
  negative-trajectory masking from I-30's pairs. BLOCKING prerequisites (not advisories): H-43
  execution-grounded fitness LIVE (no byte-hash anywhere in the loop — IS-2 retired), H-44 behavioral traps
  LIVE, I-24 canaries in the scoring stream, H-28 reward-tampering gate + the evaluator/harness/thresholds
  in the R8 frozen set, promotions through H-17 trial-N + H-18 deflation + D-6 held-out + D-7 canary. The
  whole lane stays behind the existing darwinian-evolve kill-switched allowlisted gate (W-2 satisfied by
  the SHIPPED allowlist row — deliberate, documented).
- COMPOSES-WITH: H-43..H-48 (the verification filter Track 7 built — the loop's spine), E-55 (KTO/RLVR
  notes inherited; DPO HARD-AVOID stands), D-5/D-29 bandit, I-30 (SFT init). Restates none.
- Integration: crates/training/grpo_harness.rs bridging the RL stack (NEW DEPS — user approval +
  anti-slopsquat, named in-file) to a /rl_reward endpoint ON THE SUPERVISOR side (I-1 — the trained policy
  cannot reach its reward code); frontend_pair: flywheel status panel (admits/blocks/KL trend).
- observable_effect: a fixture training episode receives rewards ONLY from execution gates
  (GrpoRewardComputed RewardServed rows, each citing the gate verdict row); a seeded reward-hack candidate
  (harness-edit fixture) is HARD-BLOCKED by H-28 (PromotionBlocked); a candidate checkpoint's promotion row
  cites the full ledger chain (PromotionLedgered) or does not promote; the live multi-night run is a
  separately-gated RunPod checkbox (W-10) — the deterministic fixture harness ships first.

## Track-17 cross-cutting
The convergence theorem governs (H-plan Track 7): without an un-gameable stronger-than-generator filter the
loop collapses; Track 17 only runs ON that filter. ID-6 makes every weight a config: same ledger, same
freezes, same honesty.

================================================================================
==== TRACK 18 — DETERMINISTIC-OVER-PROMPT HARDENING ====
================================================================================

### I-32 — structural role scoping
- Track 18 | lever STEERING-TRUST | grounding STRONG | nature REFACTOR | effort M | WAVE 2 | IC-17
- Steal/build: the persona evidence is split and ELAI should split with it: narrative personas measurably
  DAMAGE factual/code roles (MMLU -3.6..-5.3pp; coding -0.65) and HELP alignment-dependent ones. Refactor:
  executor/context-compiler/researcher prompts lose narrative personas in favor of schema-first role
  contracts (typed I/O, explicit constraints); auditor/critic/adversary KEEP hostile-critic framing;
  anti-spiral's 5 lateral personas KEEP (creative perturbation is alignment-dependent — the research's own
  carve-out). Per-role A/B measured on fixtures before the flip lands (ID-5).
- COMPOSES-WITH: Phase 2C per-role GRC configs (the role substrate), .claude/agents/* + agent role_prompts,
  C-41 skills routing, I-21 (evaluator persona kept). Restates none.
- Integration: role prompt templates + agent dispatch configs; the A/B harness reuses E-19's probe;
  frontend_pair: N/A (prompt-internal; W-6 justified).
- observable_effect: the executor's outbound system prompt contains the schema contract and ZERO persona
  narrative (journal-grepped); the per-role A/B writes RoleContractApplied(AbDelta) with the measured
  fixture deltas under seed (GD-2); auditor prompts still carry the critic framing (PersonaRetained); a
  regression in any role's fixture pass-rate blocks the flip (the refactor is evidence-gated, not aesthetic).

### I-33 — mechanical decision governance
- Track 18 | lever TRUST | grounding STRONG | nature BUILD | effort M | WAVE 2
- Steal/build: the Santander mechanical-enforcement primitives for ELAI's decision-type LLM calls (mode
  picks, gate advisories, classification calls): CEFL — the orchestrator EXTERNALIZES the candidate set and
  the model only scores/ranks it (selection-by-negation structurally impossible); I6Q — deterministic
  rationale-quality validation (min length + lexical diversity) rejecting vacuous compliant-looking
  rationales; commit-reveal entropy integrity — stochastic gate choices commit a blake3 of (seed, choice)
  before reveal so the model cannot exploit ordering. Governance metrics (cosmetic-deadlock rate, deferral
  information utilisation, framing-success rate) ledgered. The dossier's measured decoupling (governance
  invariant under task stress) is the design target, MEASURED locally (ID-5).
- COMPOSES-WITH: Phase 3F rules engine (the verdict plumbing), D-16 refusal (structured deferral target),
  B-22/H-37 gate decisions (the call sites), C-26 constrained decoding (the scoring schema).
- Integration: new crates/orchestrator/src/decision_governance.rs wrapped around the LLM-decision seams
  (prompt_decision primitive per E-25 — composes it); frontend_pair: governance metrics tile.
- observable_effect: a decision call's candidate set is orchestrator-authored (the journaled request shows
  the externalized list; a model-invented option is structurally unparseable); a vacuous rationale fixture
  is REJECTED (GovernancePrimitiveFired I6qRejected); a framing-inversion fixture (reversed field order)
  produces the SAME decision (framing-success rate ~0 on the fixture set, measured); commit precedes reveal
  in row order for every stochastic choice (CommitRevealSealed; an out-of-order arm FAILS — R2).

### I-34 — crane hybrid + grammar pinning
- Track 18 | lever TRUST-CANT-LIE | grounding STRONG | nature BUILD | effort M | WAVE 1 | IC-18
- Steal/build: three constrained-decoding hardenings on the shipped substrate: (1) CRANE-style grammar
  augmentation on the local lane — free-form thinking tokens permitted BETWEEN structural commitments
  (recovers the measured 10-15pp reasoning tax; the shipped crane_adaptive flag gets its runtime — IS-12);
  reflection/self-correction loops get constraints on the FINAL output only (structure-snowballing
  defense, C-37's pilot consumes this); (2) grammars become PINNED control-plane artifacts — blake3-pinned
  like MCP schemas, no third-party grammar ever loaded, enum-bearing schemas from untrusted sources
  rejected (CDA defense); (3) post-parse schema RE-VALIDATION on every constrained output (engines emit
  silently non-compliant outputs — landmine #7).
- COMPOSES-WITH: E-35/E-18 XGrammar substrate + C-26 DecodingMode (the flag + config), C-37 re-evaluation
  (the pilot harness), E-24 blake3 pin precedent (the MCP pattern mirrored), D-16 (an unparseable-after-
  revalidation output routes to refusal, never silent acceptance).
- Integration: providers/src/decoding.rs runtime leg + python sidecar mask layer; grammar store with pin
  manifest (R8-frozen); frontend_pair: grammar-pin status in the existing decode panel.
- observable_effect: a reasoning-heavy fixture under CRANE mode emits thinking tokens then a valid
  structure with pass-rate >= the FreeText arm on the fixture set (measured — ID-5; the tight-grammar arm's
  measured tax is recorded); a mutated grammar file is REFUSED at load (GrammarGateEvent PinViolation); a
  seeded silently-non-compliant output (fixture engine stub) is caught by re-validation
  (PostParseRejected) instead of corrupting downstream; the --trust-mask broken arm ships the bad output
  and FAILS (R2).

### I-35 — temporal tool-sequence enforcement
- Track 18 | lever CANT-WRECK | grounding STRONG | nature BUILD | effort M | WAVE 2 | depends I-1
- Steal/build: AgentSpec/Agent-C-shape runtime rules over the tool-call HISTORY, evaluated at the
  SUPERVISOR plane (I-1): trigger (a tool class fires) + predicate over the ordered session history
  ("tests ran since last src write", "no destructive verb after an untrusted ingest without a taint check",
  "git push requires a prior auditor PASS row") + enforcement (halt / require-confirmation / substitute).
  Deterministic, token-level-attack immune (CDA-immune by construction — action-level, not token-level).
  Pro2Guard-style probabilistic reachability = Tier-2 DEFERRED (needs a trace corpus).
- COMPOSES-WITH: Phase 3A FSM tool masks (per-state allow — I-35 adds ORDER), E-30 argv allowlist (per-call
  shape), F-3 committed tool sets (per-task scope), G-1 taint (the predicate's richest input), I-1
  supervisor. Restates none — this is the fourth, missing dimension (sequence).
- Integration: new crates/orchestrator/src/tool_sequence.rs with a TOML rule set (R8-frozen, mutable=0 for
  the safety core); evaluated in the supervisor dispatch; frontend_pair: sequence-verdict rows in the gate
  panel.
- observable_effect: a fixture agent attempting git-push with no auditor-PASS row in-session is HALTED
  (ToolSequenceVerdict OrderViolation naming the unsatisfied predicate); the compliant order proceeds
  (Allowed); the rules file's hash is in the frozen manifest (a run that edits its own sequence rules is
  BLOCKED — R8); replay of the session re-derives identical verdicts (GD-2).

### I-36 — cross-model prompt compilation
- Track 18 | lever CHEAPER-PORTABLE | grounding STRONG | nature BUILD | effort M | WAVE 4 | ID-3/ID-6
- Steal/build: ELAI routes across model families (B-37: 11 backends; local Qwen + cloud Sonnet/Opus), and
  prompts measurably do NOT transfer (the transfer-gap evidence). Build a compiled-prompt artifact store:
  per (role, backend) compiled prompt variants produced OFFLINE by the cloud-lane optimizer (MIPROv2-class
  search over instructions+exemplars; promptolution-style plain-string output so call sites do not
  refactor), VALIDATED on ELAI fixtures with the measured transfer gap ledgered, promoted through the ID-6
  ledger, journaled as ID-3 receipts (replay knows which compiled variant served). Deliberative-lane
  hygiene baked in: no explicit step-by-step scaffolds under extended thinking; temperature defaults
  respected (the temperature trap).
- COMPOSES-WITH: B-37 PerRoleRuntime (the routing key), E-8 GEPA (the optimizer lane — shares it), I-12
  (the same ledger-gated promotion machinery), IS-13 store precedent. Restates none.
- Integration: new crates/orchestrator/src/prompt_artifacts.rs + workflow/optimizer extensions;
  frontend_pair: per-role compiled-variant badge in SettingsWindow.
- observable_effect: a role served on two backends uses two compiled variants (PromptCompiled rows name
  variant hash + backend); the measured fixture delta between source-prompt-on-target vs compiled-on-target
  is ledgered (TransferGapMeasured, recomputable GD-2); a variant that regresses its fixture floor
  auto-rolls-back (Rolledback row); a cloud extended-thinking request provably carries no CoT scaffold
  (contract grep on the builder).

### I-37 — ears requirement objects
- Track 18 | lever VERIFIED-INTENT | grounding STRONG | nature BUILD | effort M-L | WAVE 3
- Steal/build: the field's named open problem (the behavior harness): atomic EARS-pattern requirements as
  TYPED records — content hash, bound tests, mutation coverage on those tests (I-20's score), metamorphic
  relations that bind, and a per-requirement VERIFIED/UNVERIFIED status surfaced per hunk in the cockpit.
  Import interop with Spec-Kit/Kiro-style spec files (they capture intent; ELAI proves satisfaction).
  "Satisfied = covered by T1,T2 at 88% sampled mutation + MR-3 holds" — or honestly UNVERIFIED.
- COMPOSES-WITH: C-29 IntentPair/IntentPairsGate (IS-15 — the substrate this generalizes; an IntentPair
  becomes the simplest EARS record), C-50 ContractSpec, H-54 obligation closure (formal-lane sibling), G-5
  cockpit + H-52 anchoring (the render), I-20 (coverage numbers).
- Integration: new crates/orchestrator/src/requirements.rs + the PLANNING gate extension (requirements
  authored/imported before TESTING_FIRST); frontend_pair: per-requirement chips in the cockpit (G-5).
- observable_effect: a fixture task with 3 EARS requirements renders 3 per-requirement statuses whose
  evidence pointers resolve to real test+mutation rows (RequirementVerified Satisfied with the join
  recomputable — GD-2); deleting a bound test flips the status to Unverified on re-audit (Regressed row);
  an imported spec-kit file round-trips into typed records; an unverifiable requirement is SHOWN as
  unverified, never silently dropped (R1).

### I-38 — qdelta + statistical floor
- Track 18 | lever PROVABLE | grounding STRONG | nature BUILD | effort M | WAVE 1 | ID-5 instrument
- Steal/build: QDELTA — the quality sibling of SAFEDELTA: defective diffs reaching a reviewer per 100
  tasks, gates-ON vs gates-OFF, where defective = held-out test failures + post-merge surviving mutants +
  regression count, with bootstrap CIs clustered by repo. THE STATISTICAL FLOOR AS POLICY (enforced by the
  bench harness, not culture): paired-task designs, >=5 seeds per condition with SEM, pass^k for
  reliability claims, $/task reported next to accuracy, date-stamped task cuts + the D-20 memorization
  probe surfaced as the contamination check. Ablation framing ONLY (GD-3) — never a leaderboard rank.
- COMPOSES-WITH: G-6 safedelta_runner (the harness SHAPE — mirrored, not forked), D-6/E-54 held-out corpus
  (+ its 13-gram filter), D-20 memorization probe (surfaced into the methodology), E-49 metrics, I-20
  (the mutant leg). Restates none.
- Integration: crates/cli/src/bench_qdelta.rs + a methodology doc auto-generated from machine-readable
  config; frontend_pair: QDELTA tile next to the SAFEDELTA tile.
- observable_effect: `elai bench qdelta` (fixture tier now; full corpus RunPod checkbox) emits
  QdeltaMeasured with the ON/OFF delta + CI, byte-recomputable (GD-2); a run violating the floor (3 seeds,
  unpaired) is REFUSED by the harness itself (the floor is mechanical); the methodology doc regenerates
  deterministically and names the D-20 contamination result.

### I-39 — plan state from repo
- Track 18 | lever ANTI-DRIFT | grounding STRONG | nature BUILD | effort S-M | WAVE 1
- Steal/build: kill hand-maintained status drift (five drifting sources of truth; every index's SCHEMA
  snapshot went stale within days): `elai plan status` derives per-plan state from the journal +
  plan-status fragments + wiring_contract_check results, with a "verified against <commit/map-date>"
  anchor per line; hand-tallies demote to build artifacts. PLUS the journal-rot guard: ONE migration test
  that replays an archived old-schema ssb.db through current code (composing F-13's archived copies) —
  old RRAs are the trust product; they must not rot silently.
- COMPOSES-WITH: SSB viewer (read paths), the s-plan-audit fragments + wiring_contract_check.py (the
  evidence), F-13 session archive (the old-db source), E-44 migration machinery. Restates none.
- Integration: crates/cli/src/plan_status_cmd.rs + a contracts test replaying the archived db;
  frontend_pair: N/A (CLI + doc artifact; W-6 justified).
- observable_effect: `elai plan status --letter I` prints per-session states each carrying its evidence
  pointer (fragment path + check verdict + journal row), regenerated byte-identically from the same inputs
  (GD-2); the migration replay test opens a REAL archived ssb.db (F-13 artifact) through current code and
  decodes N rows (PlanStateDerived beacon); a corrupted archive FAILS loudly (R2), never skips.

### I-40 — trust-layer wrapper + rra spec
- Track 18 | lever WEDGE | grounding STRONG (strategy: both reviews converge) | nature BUILD | effort L |
  WAVE 5
- Steal/build: unbundle the agent-agnostic trust layer as the wedge: (a) publish the .rra format as an
  open, versioned spec (the replayable-evidence artifact nobody else ships — spec doc + a conformance
  checker); (b) a minimal proxy/MCP wrapper that runs an EXTERNAL agent's tool traffic through ELAI's
  taint tagging + write-gates + RRA emission ("the trust layer for whatever agent you already use").
  Two-tier strategy recorded: wrapper = wedge + standard; the deep orchestrator stays the premium
  destination. G-9 redaction is the publication precondition (an exported RRA must not leak secrets).
- COMPOSES-WITH: G-1 taint + F-1/F-9/G-3 gates + G-2 RRA (the wrapped surface — composes, never forks),
  B-37 adapter patterns, E-24 MCP bridge hardening, G-9 redaction (precondition), I-18 (the chained
  format the spec documents).
- Integration: docs/spec/rra-spec-v1.md (in-repo, versioned) + crates/cli wrapper subcommand (`elai wrap --
  <agent cmd>`) proxying tool I/O; frontend_pair: wrapped-run rows render in the existing RRA viewer.
- observable_effect: an external fixture agent run under `elai wrap` produces a spec-conformant .rra whose
  conformance checker passes, with taint rows on its untrusted ingests and a write-gate BLOCK on a seeded
  malicious write (the wrapper proves the gates bite outside ELAI's own loop); the published spec's
  conformance suite rejects a tampered artifact; redaction verified on the exported bundle (no raw secret
  bytes — G-9 composed).

## Track-18 cross-cutting
Every session converts a prompt-shaped control into a mechanical one and MEASURES the conversion (ID-5):
roles, decisions, grammars, sequences, prompts, intent, the release number, the plan ledger, and finally
the wedge that ships the whole evidence stack around other agents.

================================================================================
==== TRACK 19 — DESIGN-PHASE CRITIQUE + TRUST-CLAIM DISCIPLINE ====
================================================================================
Authored 2026-06-10 from workflow/research/grok-elai-trust/{ELAI-CRITIQUE-LENSES.md, ELAI-IMPROVEMENT-PLAN.md}
+ workflow/reasoning/ELAI-TOKEN-EFFICIENCY-PLAN-2026-06-08.md. Track 19 is the genuinely-uncovered residue after
cross-referencing those three docs against B..I: the token-efficiency doc is FULLY covered (it restates H-plan
Track 1 H-1..H-9 + I-11 — no new file); the improvement-plan's agent-agnostic-verifier WEDGE is ALREADY covered
by I-40 (no duplicate authored — GD-1). Track 19 adds only what those plans do not. HARD differentiator: every
Track-19 LLM-judged surface is ADVISORY and NEVER load-bearing (the §6 ban-LLM-judge rule); the off-agent spine
alone decides DONE. GTM/willingness-to-pay (F0.3) and SOC 2 are non-engineering and deliberately NOT sessions.

### I-41 — spec-time critique lenses (L1-L8)
- Track 19 | lever PLURAL-DOUBT | grounding MODERATE | nature BUILD | effort M | WAVE 2 | binds ID-4 + §6 ban-LLM-judge
- Steal/build: 8 re-authored design-phase critique lenses (Assumption Excavator / Implied Completeness / Ambiguity
  Mapper / Gap Analyzer / Pre-Implementation Architect / Public-Interface Validator / Documentation Validator /
  Anxiety Reader) run on PROSE/SPEC before any deterministic gate has an artifact to hash. LLM-judged = noisy/
  bypassable (MSR-2026 60.2% 0-30% signal; NAACL all-8-bypassed), so ADVISORY ONLY — a finding with no concrete
  anchor (spec heading / file:line / symbol / quoted phrase) is DROPPED; no lens output ever sets accept/block.
  Bounded (<=8 findings/lens, deduped); lens count scales to artifact risk. Native re-authored (E-43 mcp-scan
  gate avoided). Closes ELAI's comparatively-thin DESIGN-phase critique (strength is post-impl/wiring).
- COMPOSES-WITH: B-38 AmbiguityScorer (L3's gate number — lens is the human map), E-45 golden lock + cross_lang_diff
  (L6's design-time mirror), Phase 2B grounding-sweep (L1 names assumptions, sweep verifies), B-40 idea-critic loop
  (the advisory wire-in). Restates none.
- Integration: crates/orchestrator/src/critique/lenses.rs + `elai critique <spec>` CLI + idea_framing advisory hook;
  frontend_pair: advisory findings list in ReasoningDrawer / the B-40 framing panel (non-blocking; BLOCKER-CANDIDATE
  routes to human, never auto-blocks).
- observable_effect: `elai critique <fixture-spec>` emits CritiqueLensFinding(AdvisoryFinding) rows anchored at a
  planted assumption (L1) + a planted gap (L4); an anchorless finding emits DroppedNoAnchor (never surfaced); the
  contract test proves the run emits ZERO gate-verdict rows (the load-bearing path is provably untouched — R1).

### I-42 — critique-only operating profile
- Track 19 | lever HUMAN-AS-VERIFIER | grounding MODERATE | nature BUILD | effort M | WAVE 3 | depends I-41
- Steal/build: a SECOND selectable profile alongside autonomous orchestrate — RunProfile{Orchestrate, CritiqueOnly}.
  In CritiqueOnly the OPERATOR writes the code, spawned agents run ONLY for critique (I-41 lenses at design + the
  shipped auditor/off-agent spine at ship), and the executor/fixer spawn path is STRUCTURALLY gated off (not a
  prompt instruction). The human-as-verifier mode R5 calls durable; orchestrate stays default. One engine, one
  spine across both — only WHO writes + whether executors spawn differs. INVARIANT: in CritiqueOnly the set of
  executor/fixer-attributed file_write rows is EMPTY.
- COMPOSES-WITH: I-41 (the design-phase critique engine), the shipped auditor + off-agent spine (F-1/F-4/G-2/B-41 —
  unchanged), B-22 GateMode flag plumbing (the pattern; RunProfile is orthogonal — do NOT overload), I-1 supervisor
  (the structural seam to reject executor side-effect Directives). Restates none.
- Integration: crates/orchestrator/src/run_profile.rs + `elai run --profile critique-only` + the spawn-site gate;
  frontend_pair: active profile in the StatusBar mode cluster (B-22 patch-4G.1 badge pattern; no new component).
- observable_effect: a critique-only run produces I-41 findings + auditor/spine verdicts but ZERO executor/fixer
  file_write rows (the spawn-gate fired); the SAME fixture under --profile orchestrate DOES produce executor writes
  (the gate is the differentiator, not a no-op — R2); the off-agent spine verdicts are byte-identical across both
  profiles (one spine — GD-2).

### I-43 — claim scoper + Phase-0 kill-gate
- Track 19 | lever FALSIFY-FIRST / PROVABLE | grounding STRONG | nature BUILD | effort M | WAVE 1 | binds ID-5
- Steal/build: the cheap falsifiers grounded-by-default demands BEFORE the G-6 XL build. (a) PILOT — the shipped
  seccodeplt/safedelta runner at reduced N (50-100), gate-ON vs OFF, dynamic-exploit scored, reporting the
  residual-insecure-rate delta SIGN + magnitude AND the reviewer-reaching rate (the block-everything-trap detector:
  a gate that wins only by blocking nearly everything FAILS). (b) DIVERGENCE SCOPER — ~20 identical tasks local-Qwen
  vs cloud-Claude under fixed seed, measuring action-stream divergence on security paths to decide the
  reproducibility CLAIM WORDING (cloud = trace-authentication, NOT deterministic replay — GD-5). Output is a go/no-go
  memo, never an external claim. Distinct from H-23 (same-env determinism) — this measures CROSS-LANE divergence to
  scope the claim. No paper constant is a threshold (ID-5).
- COMPOSES-WITH: I-38 QDELTA harness (the full instrument run at reduced N — compose, never re-implement), G-6
  SAFEDELTA + the seccodeplt/safedelta runner (the gate-ON/OFF + git-strip + dynamic-exploit path), H-23
  DriftComparator shape (first-divergence reporting), G-2 rra.rs (the action-stream traces diffed across lanes),
  B-37 lanes. Restates none.
- Integration: crates/cli/src/bench_claim_scope.rs (`elai bench claim-scope`) + a go/no-go memo writer; frontend_pair:
  a Phase-0 go/no-go panel in the H-9 cost cockpit (or W-6-justified absence — the memo is the load-bearing artifact).
- observable_effect: `elai bench claim-scope --n 50` (fixture now, LIVE leg gated — W-10) emits ClaimScopeMeasured
  (PilotDelta with sign+magnitude+reviewer-reaching rate; CrossLaneDivergence %; ClaimScoped honest wording) +
  workflow/reasoning/I-43-phase0-memo.md; a block-everything fixture trips the trap detector (pilot FAILS — R2); a
  >5% cross-lane divergence emits trace-authentication-only wording, never replay (GD-5); all numbers journal-recompute (GD-2).

### I-44 — zero-retention-after-verification export mode
- Track 19 | lever AUDITABLE / TABLE-STAKES | grounding STRONG | nature BUILD | effort S | WAVE 3 | depends G-9
- Steal/build: zero-retention is the step past G-9 redaction — once the sealed verdict + evidence bundle exist (G-2
  RRA + G-8 attestation), PURGE the raw inputs (prompts, raw tool I/O, raw model I/O) at file granularity (reusing
  F-13's archive-then-purge primitive; the SSB WAL is append-only so retention is whole-session-DB purge, the G-9
  shape), keeping ONLY redacted, hash-checkable evidence. INVARIANT: post-purge a grep over the exported bundle for
  raw-input markers is EMPTY AND `elai replay --verify` STILL PASSES on the hash-checkable steps (the verdict survives
  the purge). G-9 redaction is the precondition (mask retained summaries first). Opt-in, default unchanged (GD-4 —
  the purge is destructive/irreversible). Self-host is already satisfied by the I-40 local wrapper; SOC 2 is process,
  out of scope.
- COMPOSES-WITH: G-9 redaction (precondition + the file-granularity purge shape), F-13 session_archive (the
  archive-then-purge primitive), G-2 RRA + G-8 attestation (the evidence that SURVIVES the purge), I-40 wrapper export
  (the surface this policy governs). Restates none.
- Integration: crates/orchestrator/src/retention.rs (RetentionPolicy{Full, ZeroAfterVerify}) + `elai run
  --zero-retention` + the seal-point retention check; frontend_pair: RetentionPurged rows in the RRA viewer / a
  session-retention drawer (the purge receipt is the operator-visible artifact).
- observable_effect: `elai run --zero-retention <fixture w/ planted secret + raw tool I/O>` emits RetentionPurged
  (RawInputsPurged + EvidenceRetained{blake3-of-retained} + PurgeVerified); a grep for raw-prompt/raw-tool markers in
  the export is EMPTY; `elai replay --verify` STILL PASSES on hash-checkable steps; the same fixture under default Full
  differs ONLY in raw-byte presence (evidence + verdict byte-identical — GD-2, R2).

## Track-19 cross-cutting
The whole track is FRONT-of-pipe + claim-discipline, and it earns its place by NOT duplicating the back: the lenses
and the critique-only profile thicken design-phase doubt where deterministic gates cannot reach (advisory only, §6);
I-43 falsifies the trust number cheaply before the G-6 build commits; I-44 makes the evidence bundle shippable to a
client without a retention liability. None of the four ever sets an accept/block verdict — the off-agent spine (F/G
gates) remains the sole arbiter of DONE, identically in both operating profiles.

================================================================================
==== TRACK 20 — COMPETITOR-STEAL PORTS (lazycodex / gajae-code / roach-pi) ====
================================================================================
Authored 2026-06-10 from docs/.source/STEAL-LIST.md (three competitor repos cloned + mapped + summarized under
docs/.source/). Track 20 is the Tier-1/Tier-2 residue: the gajae provider-layer wholesale port was REJECTED
(B-37 broader, subscription-riding philosophy better), Tier-3 items stay reference material for already-planned
sessions, and the declines (B7/C3/C4/D1/E4/F1 etc.) are recorded in I-0_INDEX so they are not re-litigated.
Grounding map: _grounding/I-STEAL-GROUNDING-MAP.md (SL-1..SL-11, SLC-1..SLC-3). BINDING per SL-10/SL-11: every
port session's executor reads the named competitor source file before EXECUTING — per-repo SUMMARYs are MEDIUM
confidence and a SUMMARY-only port is a W-3 violation.

### I-45 — objective-hash sealed receipts
- Track 20 | lever CANT-LIE | grounding STRONG (SL-5) / MODERATE (roach, SL-11) | nature EXTENSION | effort S-M | WAVE 1
- Steal/build: roach-pi E1, the STEAL-LIST's sharpest single steal: blake3(canonical objective bytes) stamped
  into every sealed PASS (rra.rs seal + attestation), staleness-checked on every receipt READ path — mutating
  the goal/acceptance text after a PASS makes the receipt detectably stale instead of silently still-valid.
  F-4 stdout_hash hashes the artifact; I-45 hashes the GOAL. Legacy receipts verify as explicit LEGACY, never
  silently fresh. Substrate confirmed SHIPPED (SL-5 corrects the stale "rra.rs absent" memory).
- COMPOSES-WITH: G-2 rra.rs + G-8 attestation.rs (extends in place), F-4 stdout_hash (sibling field), I-18
  chain (same record — coordinate layout), I-37 EARS (future objective source), I-39 (a re-checking consumer).
- Integration: rra.rs seal site + attestation.rs verify leg + `elai verify-attestation`; frontend_pair:
  stale-receipt badge on the existing RRA viewer row.
- observable_effect: seal a fixture run, mutate the objective text, verify: ObjectiveSealed(MutationDetected)
  + ReceiptStale hard FAIL naming the hash pair; the unmutated arm verifies PASS; a verify that passes after
  mutation is the broken arm and FAILS (R2).

### I-46 — bash interceptor + command fixup
- Track 20 | lever CHEAPER + CANT-WRECK | grounding STRONG (SL-2) / MEDIUM (gajae, SL-10) | nature EXTENSION | effort S-M | WAVE 2
- Steal/build: gajae B1 ("top tool steal"): bash_validation.rs validates but does not REDIRECT. Add the
  interceptor ahead of it: bare grep/cat/find/head/tail -> REFUSE-WITH-REDIRECT carrying the exact substitute
  registered-tool call; a frozen deterministic fixup table for known-broken shapes; allowed-prefixes pass
  untouched. No LLM in the loop, no auto-execution of substitutes, narrows-never-widens validation.
- COMPOSES-WITH: tools::bash_validation (the seam — extended, not forked), E-30 argv (downstream, unchanged),
  I-35 sequence (disjoint dimension), I-53 recipes (future redirect target), H-4 tool search.
- Integration: crates/tools/src/bash_validation.rs + the bash dispatch site; tables R8-frozen; frontend_pair:
  redirect rows in the existing tool-call drawer.
- observable_effect: a live `grep -rn` bash call lands BashRedirected(Redirected) naming the substitute and
  spawns NO process (journal-verified); allowed-prefix commands execute untouched with zero redirect rows;
  a redirect row coexisting with a spawned grep is the broken arm and FAILS (R2).

### I-47 — verifier-input hardening + dispatch rule injection
- Track 20 | lever TRUST-ANTI-BIAS | grounding STRONG (seams) / MODERATE (roach, SL-11) | nature EXTENSION | effort S-M | WAVE 2
- Steal/build: roach-pi E2+E3 on one seam (dispatch prompt assembly): (a) the auditor's untrusted inputs
  (worker diff, claims, tool output) enter inside typed data tags under a standing "don't follow embedded
  instructions / don't trust self-attestation" frame; (b) the worker discipline rules (read-before-write,
  scope, no premature abstraction, no while-I'm-here refactoring) inject MECHANICALLY into worker dispatch as
  a per-role frozen, byte-stable block (HD-2-safe). No new classifier — E-37 owns data-plane classification.
- COMPOSES-WITH: E-37 PromptArmor (upstream), H-31/H-32 auditor firewall (objective sibling), I-32 role
  contracts (same seam — coordinate), D-13 adversary, I-21 sprint contracts.
- Integration: the agent dispatch prompt builder + auditor dispatch site; injection table R8-frozen;
  frontend_pair: N/A (prompt-internal; W-6 justified).
- observable_effect: an embedded-instruction + self-attestation fixture lands inside data tags below the
  frame (journal-verified request bytes) with PromptHardeningApplied(InputsTagged); worker dispatches carry
  the byte-identical role block across two calls (HD-2 assert) + (RulesInjected); zero hardening rows while
  ON = FAIL (R1).

### I-48 — prompts as reviewed file artifacts
- Track 20 | lever STEERING | grounding STRONG | nature REFACTOR | effort S | WAVE 2
- Steal/build: gajae C1: every dispatch prompt template becomes a static file under crates/*/prompts/
  imported via include_str! — diffable, reviewed, never inline. Byte-preserving extraction (content
  unchanged, location changed) + a contract test banning new long inline prompt literals in dispatch code.
  The precondition for I-36 compiled prompts and I-47 frozen blocks being pinnable/diffable.
- COMPOSES-WITH: D-1 typed templates (substitution unchanged), I-36 (consumer), I-47 (its block becomes one
  of these files), E-8 GEPA (optimizer reads file artifacts).
- Integration: orchestrator + providers dispatch modules; frontend_pair: N/A (build-time; W-6 justified).
- observable_effect: assembled executor + auditor prompts are blake3-identical pre/post refactor on the
  journaled request bodies; the ban test FAILS on a seeded inline-literal reintroduction; NO new action_type
  (GD-1).

### I-49 — bedrock + azure direct lanes + cross-protocol shim
- Track 20 | lever REACH | grounding STRONG (gap SL-8) / MEDIUM (gajae, SL-10) | nature BUILD (reference
  port) | effort L | WAVE 4 | live legs network-gated (W-10) | FULL-FORM expansion required before EXECUTING
- Steal/build: the only provider steal that survived the wholesale-port rejection: (1) Bedrock direct
  (SigV4 signing + binary EventStream decode), (2) Azure OpenAI Responses shape, (3) the OpenAI-as-Anthropic
  shim as the normalization-seam pattern (extend genai_adapter if it fits). Reference ports of protocol
  logic, TS source read first; MANDATORY pre-task: WebSearch current AWS/Azure protocol docs +
  anti-slopsquat any aws-* crate candidate + user approval on any new dep. Deterministic replay-fixture
  harness ships now (recorded frames decoded byte-exactly); live calls are separately-gated checkboxes.
- COMPOSES-WITH: B-37 RuntimeAdapter/PerRoleRuntime (anthropic_api.rs is the pattern file), genai_adapter,
  oauth.rs + CLI bridges (untouched KEEP list), I-50 (credential supply), I-51 (model metadata).
- Integration: providers/src/runtime/{bedrock.rs, azure_openai.rs} + shim module + registry; frontend_pair:
  the two backends appear in the existing RuntimePanel list.
- observable_effect: `elai runtime list` shows both; replay fixtures decode byte-exactly with a corrupted
  frame FAILING typed (R2); the SigV4 fixture reproduces the documented known-good signature; NO new
  action_type — existing runtime/route rows name the backend ids (GD-1).

### I-50 — subscription auth broker
- Track 20 | lever REACH | grounding STRONG (SL-3) / MEDIUM (gajae, SL-10) | nature EXTENSION | effort M | WAVE 4
- Steal/build: gajae A2: generalize the Anthropic-only oauth.rs into an AuthProvider trait (acquire/refresh/
  revoke/status) + a broker owning token lifecycle/storage. The existing Claude-plan flow becomes the first
  impl BEHIND the trait, byte-compatible (token file format + refresh semantics regression-guarded). No new
  provider flows this session; no new storage backend; token material NEVER journaled.
- COMPOSES-WITH: oauth.rs (wrapped, preserved), B-37 adapters (consumers), I-49 (future credential impls),
  I-18 (rows join the chain).
- Integration: providers/src/auth_broker.rs + oauth.rs refactor + adapter call sites; frontend_pair: auth
  status rows in existing RuntimePanel/Settings.
- observable_effect: a real authenticated cloud request lands AuthBrokerEvent(TokenAcquired|Refreshed)
  provider=anthropic; expired-token fixture refreshes then succeeds; refresh-failure fixture lands
  RefreshFailed + typed error; journal grep for token material is EMPTY; auth without a broker row = FAIL (R1).

### I-51 — unified model capability catalog
- Track 20 | lever SMARTER-ORCH | grounding STRONG (SL-4) | nature EXTENSION (consolidation) | effort S-M | WAVE 1
- Steal/build: gajae A1: one models.toml catalog ({context_window, pricing in/out/cache, thinking support +
  effort mapping, max_tokens defaults, vram tier} per model id) over the CONFIRMED scatter — pricing.rs,
  vram_tier.rs duplicated across providers AND core, pricing_rollup.rs (SL-4). Existing modules become VIEWS
  with preserved APIs; per-entry staleness date (landmine 4); SLC-2: the measured VRAM tier plan is WRAPPED,
  never deleted. No live price fetching; behavior-neutral by construction.
- COMPOSES-WITH: E-39 cost gate, E-31 tier matrix, H-9 cockpit (consumers, unchanged), I-10 economics
  planner + I-25 router (future consumers — their clean input), B-37 defaults (deferred leg).
- Integration: providers/src/model_catalog.rs + models.toml; pricing.rs/vram_tier.rs re-sourced in place;
  frontend_pair: N/A (behind preserved APIs; W-6 justified).
- observable_effect: cost + tier outputs byte-identical pre/post on a fixture set; the drift contract test
  FAILS when a seeded models.toml edit diverges a view (R2); price-constant grep outside models.toml in
  migrated modules returns 0; NO new action_type (GD-1).

### I-52 — checkpoint/rewind agent tool
- Track 20 | lever CONCURRENCY-SAFETY | grounding STRONG (SL-6) | nature ACTIVATION | effort S-M | WAVE 3
- Steal/build: gajae B2 corrected by SLC-1: the snapshot machinery is SHIPPED (edit_checkpoints.rs +
  checkpoint_cmd.rs + two e25 contracts) but no agent can call it. Ship the 3-arm tool (save/rewind/diff)
  over the EXISTING store: session-scoped (no cross-session rewind), divergence-refusing (files changed
  outside the checkpointed set = refuse, named). H-49 coordination clause: if H-49 ships first, re-scope to
  the tool surface over its store + REUSE EditHistoryEvent. Never a second snapshot store. In-branch
  single-agent speculation; I-2 stays the parallel-branch tool.
- COMPOSES-WITH: E-25 edit_checkpoints (activated), H-49 (clause; RV-1 semantics inherited when shipped),
  I-2 (disjoint scope), the E-35 3-arm tool catalog pattern.
- Integration: tool catalog + edit_checkpoints.rs label/scope lookups; gate posture per W-2 if the substrate
  is still flagged OFF; frontend_pair: checkpoint rows in the existing session drawer.
- observable_effect: save A, edit, rewind A: post-rewind bytes blake3-match the pre-images
  (CheckpointToolEvent Saved/Rewound); a diverged-file fixture is REFUSED naming the path; cross-session
  rewind REFUSED; a silent overwrite of a diverged file is the broken arm and FAILS (R2).

### I-53 — recipe runners + utility tool pack
- Track 20 | lever TOOLS | grounding STRONG (gap SL-9) / MEDIUM (gajae, SL-10) | nature BUILD | effort S-M | WAVE 2
- Steal/build: gajae B4 + the two worthwhile B9 cherry-picks: (a) recipe tool — discover Justfile/Makefile
  targets + cargo aliases, run the NAMED recipe through the existing bash_validation + E-30 gates (the
  owner-sanctioned entrypoints become first-class, journaled invocations); (b) read-only archive reader (no
  extraction side effects); (c) capped JSON tree explorer (the token-discipline tool). cron/monitor/
  background-job tools DECLINED — the orchestrator + Layer-1A schedules own those concerns.
- COMPOSES-WITH: bash_validation + E-30 (the gates — composed, never bypassed), I-46 (future redirect
  target), the tool catalog, H-4 tool search.
- Integration: crates/tools/src/{recipe.rs, archive_reader.rs, json_tree.rs} + registrations; any archive
  dep verified against the existing workspace tree first (anti-slopsquat + approval if new); frontend_pair:
  rows in the existing tool-call drawer.
- observable_effect: recipe list names the fixture Justfile targets; the run's journaled command carries a
  bash_validation verdict row (a run without one = FAIL, R1); archive read writes ZERO files to disk
  (fixture dir diff empty); json_tree output capped with truncation marked; NO new action_type (GD-1).

### I-54 — structural AST edit tool
- Track 20 | lever TOOLS-VERIFIED | grounding STRONG (SL-7) / MEDIUM (gajae, SL-10) | nature BUILD | effort M | WAVE 3
- Steal/build: gajae B3 as an IDEA steal (SLC-3: any ast-grep dep is a BLOCK — gajae vendors ast-grep-core,
  ELAI rejected it): structural search/preview/apply built on the in-tree tree-sitter 0.26 + 9 grammars
  (knowledge/Cargo.toml:13-22). tree-sitter query syntax as the pattern language (no new DSL); preview is
  MANDATORY before apply (the match set + diff hash journal first); apply is per-file atomic, staleness-
  refusing, and rides the SAME PreToolUse edit gates as Edit. Syntactic only — semantic rename is E-52's
  LSP lane, named in-file. Rust/Python/TS grammars first.
- COMPOSES-WITH: knowledge grammars (one version source — never a second pin), F-1/PreToolUse edit gates,
  E-52 (semantic sibling), I-20 (survivor Directives may cite it), impact-mapper (harness-side sibling).
- Integration: crates/tools/src/structural_edit.rs consuming a knowledge pub parser API + catalog
  registration; frontend_pair: preview/apply rows in the existing tool-call drawer.
- observable_effect: a fixture-crate rename via search>preview>apply: post-apply byte-diff equals the
  preview diff and the fixture compiles (StructuralEditApplied Matched/Previewed/Applied); a broken rewrite
  template fails at preview (ParseFailed, ZERO bytes written); apply with no preceding preview row = FAIL (R1).

## Track-20 cross-cutting (what makes it different)
Track 20 is the only track sourced from COMPETITOR CODE rather than research corpora, so its discipline is
provenance: every session names its source files under docs/.source/, carries the SL-10/SL-11 binding read
rule (SUMMARY-only ports are W-3 violations), and the three SLC corrections (activation-not-build,
wrap-never-delete, idea-not-dep) keep the ports from regressing ELAI decisions already made. Six of ten
sessions are EXTENSION/ACTIVATION/REFACTOR on shipped substrate; the wholesale-port temptation is recorded as
REJECTED in I-0 so it is never re-litigated. ADDS no new delta family (R-j).

End of I-SUMMARY_INDEX.md.
