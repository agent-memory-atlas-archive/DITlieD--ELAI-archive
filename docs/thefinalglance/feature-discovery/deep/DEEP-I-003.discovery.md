# DEEP-I-003 — exhaustive feature discovery

## Scope, method, and classification

This ledger is a line-anchored deep read of exactly the ten originals assigned to manifest chunk
DEEP-I-003. The only non-original used for orientation was
docs/thefinalglance/feature-discovery/09-I-SUMMARY_INDEX.discovery.md.
No original from another deep chunk was read.

Classification:

- **[E] explicit** — stated by the assigned source.
- **[I] inferred** — a bounded interpretation, never promoted to a requirement.
- **[A] ambiguous** — the source leaves a value, seam, or status unresolved.
- **[C] conflicting/corrective** — two source statements differ, or the orientation summary needs a
  status/meaning correction.

Anchors use exact source line numbers in the form filename:line or filename:start-end. Plan-asserted and
live/shipped claims are preserved as claims; they are not independently upgraded by this read.

## Exact processed manifest and integrity verification

Manifest section:
docs/thefinalglance/feature-discovery/DEEP-SOURCE-MANIFEST.md:807-822.

| Chunk | Class | Lines | Bytes | SHA-256 | Exact absolute path | Verification |
|---|---|---:|---:|---|---|---|
| DEEP-I-003 | numbered/index original | 117 | 8213 | c1dcb05555ab190ce87f7daad419eea07a79cf80199d2abb25c50b060d7b44ef | docs/thefinalglance/I-plan/I-1_directive_supervisor_enforcement_plane.md | exact |
| DEEP-I-003 | numbered/index original | 193 | 15323 | 9967be0ae8db28d7564b96b4910b02a65889f5e651524f613c21744a36f4ea2d | docs/thefinalglance/I-plan/I-4_evidence_ranked_search_ladder.md | exact |
| DEEP-I-003 | numbered/index original | 136 | 9856 | 33f08e0127711d345b45fcda45b32dd5d0c98cc1480fcf9c8ec446a066325d2f | docs/thefinalglance/I-plan/I-9_error_amplification_telemetry.md | exact |
| DEEP-I-003 | numbered/index original | 120 | 11650 | c5eeb6d26e1822656af736488960c7e02fb1fac6728bc557539832aaca17c24e | docs/thefinalglance/I-plan/I-20_budgeted_mutation_scheduler.md | exact |
| DEEP-I-003 | numbered/index original | 97 | 7468 | c97b04a168be835f81175ea9972796dd86205e5a11b51ae40900dec4ed70e6e4 | docs/thefinalglance/I-plan/I-32_structural_role_scoping.md | exact |
| DEEP-I-003 | numbered/index original | 106 | 7960 | 9284444b3e35cc869a8a183b20547eccfe4c7be42c35ea18724249db627d1bb0 | docs/thefinalglance/I-plan/I-33_mechanical_decision_governance.md | exact |
| DEEP-I-003 | numbered/index original | 112 | 8484 | 078dee380d1da622cc80fbf155fa2894c31e493641d2ef5c4d5053a30d05eed7 | docs/thefinalglance/I-plan/I-38_qdelta_statistical_floor.md | exact |
| DEEP-I-003 | numbered/index original | 129 | 10568 | 2679407d264ef6dc8976d422826a856f1ab0a05f4f6cada461944cef6e22e212 | docs/thefinalglance/I-plan/I-41_spec_time_critique_lenses.md | exact |
| DEEP-I-003 | numbered/index original | 75 | 5127 | c60669807533228199827b1e5a9edc1dc9133ae18fa161f8e2ac63f5146c9a4b | docs/thefinalglance/I-plan/I-50_subscription_auth_broker.md | exact |
| DEEP-I-003 | numbered/index original | 72 | 4974 | 0f62ced3fcb20fea3fddba8d4f75fe2235c52345bb076584fe77ca8079def6ec | docs/thefinalglance/I-plan/I-52_checkpoint_rewind_tool.md | exact |
| **Total** | 10 originals | **1157** | **89623** | — | — | exact |

Verification used logical-line counts, byte counts, and SHA-256 on the files as read. Every row and the
aggregate line/byte totals match the manifest.

## Per-file exhaustive ledgers

### I-1 — directive supervisor enforcement plane

Source:
docs/thefinalglance/I-plan/I-1_directive_supervisor_enforcement_plane.md

- **DI003-I01-001 [E][status]** Track 12; CANT-WRECK; STRONG; BUILD; effort L; Wave 2; binds ID-2.
  Anchor: I-1_directive_supervisor_enforcement_plane.md:3.
- **DI003-I01-002 [E][gap/security]** Existing PreToolUse enforcement shares the generator process; no
  Directive type or structural supervisor boundary exists. Anchor: I-1_directive_supervisor_enforcement_plane.md:4-9.
- **DI003-I01-003 [E][schema]** Tier-1 Directive is a pure value containing tool_name, args, intent_id, and
  initiator. Anchor: I-1_directive_supervisor_enforcement_plane.md:8-9,35.
- **DI003-I01-004 [E][invariant]** The generator emits Directives and must not directly call a side-effect
  sink. Anchor: I-1_directive_supervisor_enforcement_plane.md:12-16.
- **DI003-I01-005 [E][security]** The target architecture makes a Supervisor the sole side-effect executor
  and owner of credentials and egress paths. Anchor: I-1_directive_supervisor_enforcement_plane.md:8-9.
- **DI003-I01-006 [E][workflow]** Supervisor execution runs every applicable gate before the side effect
  and executes only on PASS. Anchor: I-1_directive_supervisor_enforcement_plane.md:13-16.
- **DI003-I01-007 [E][integration]** The pre-check chain composes F-1 VibeGate, E-30 argv allowlist, and
  G-3 blast-radius rather than reimplementing them. Anchor: I-1_directive_supervisor_enforcement_plane.md:14-15,27-28.
- **DI003-I01-008 [E][recovery]** A Supervisor crash invokes E-44 saga rollback to undo partial state.
  Anchor: I-1_directive_supervisor_enforcement_plane.md:15-16,38.
- **DI003-I01-009 [E][test/security]** A contract must prove that a generator-authored function cannot
  satisfy a gate or reach a sink/egress path. Anchor: I-1_directive_supervisor_enforcement_plane.md:16-17,39.
- **DI003-I01-010 [E][non-goal]** A mere async-task/thread split is declined because it retains the heap
  and fault domain. Anchor: I-1_directive_supervisor_enforcement_plane.md:17-18.
- **DI003-I01-011 [E][non-goal]** Capability passing without OS isolation is declined because the handle
  could leak back to the generator. Anchor: I-1_directive_supervisor_enforcement_plane.md:18-19.
- **DI003-I01-012 [E][dependency]** The existing Phase 4A IPC seam is the required generator/Supervisor
  channel; a second process seam is forbidden. Anchor: I-1_directive_supervisor_enforcement_plane.md:26,66.
- **DI003-I01-013 [E][tier-1]** Ship Directive, a Supervisor trait/process abstraction, and an in-process
  implementation sufficient for tests. Anchor: I-1_directive_supervisor_enforcement_plane.md:34-38,48.
- **DI003-I01-014 [E][observability]** Register DirectiveExecuted with CompensationFired; clean execution
  also produces Succeeded and rejected execution produces Refused. Anchor:
  I-1_directive_supervisor_enforcement_plane.md:40,62-63,88-91.
- **DI003-I01-015 [E][deferred]** A distinct OS PID and IPC channel are Tier-2 and process/network-launch
  gated. Anchor: I-1_directive_supervisor_enforcement_plane.md:42-44.
- **DI003-I01-016 [E][deferred/security]** The credential vault/key-management surface is a separate
  Tier-2 session. Anchor: I-1_directive_supervisor_enforcement_plane.md:42-45.
- **DI003-I01-017 [E][deferred/integration]** A cloud-sandbox adapter may reuse the trait but is optional
  and never a dependency. Anchor: I-1_directive_supervisor_enforcement_plane.md:45.
- **DI003-I01-018 [E][surface]** Create directive.rs, the I-1 contract, and I-1-decisions.md; modify
  task_runner dispatch and hook_dispatcher. Anchor: I-1_directive_supervisor_enforcement_plane.md:47-54.
- **DI003-I01-019 [E][identifiers]** Produced identifiers are Directive, Supervisor,
  DirectiveExecuted, and i1_directive_supervisor_contract.rs. Anchor:
  I-1_directive_supervisor_enforcement_plane.md:58-64.
- **DI003-I01-020 [E][dependency]** The Supervisor consumes E-44 rollback and safety Provenance in addition
  to the three gates. Anchor: I-1_directive_supervisor_enforcement_plane.md:65-71.
- **DI003-I01-021 [E][reachability]** Production reachability must be task_runner → execute_directive and
  hook_dispatcher → run_gates; test-only callers do not satisfy R7. Anchor:
  I-1_directive_supervisor_enforcement_plane.md:72-75.
- **DI003-I01-022 [E][constraint]** Production paths ban unwrap/expect, blake2/sha2, fabricated
  measurements, and invented file/line evidence. Anchor: I-1_directive_supervisor_enforcement_plane.md:76-80.
- **DI003-I01-023 [E][constraint/status]** Tier-1 must not claim OS process separation, and may not
  reimplement F-1/E-30/G-3 logic. Anchor: I-1_directive_supervisor_enforcement_plane.md:81-82.
- **DI003-I01-024 [E][UI]** DirectiveExecuted rows render in an existing SSB-backed drawer; no new Tier-1
  panel is required. Anchor: I-1_directive_supervisor_enforcement_plane.md:55-56,83-86.
- **DI003-I01-025 [E][acceptance]** Injected OOM/panic mid-tool must emit CompensationFired and restore
  pre-tool bytes exactly; clean execution emits Succeeded. Anchor:
  I-1_directive_supervisor_enforcement_plane.md:87-94.
- **DI003-I01-026 [E][failure/evidence]** A direct-sink attempt emits Refused with the full Directive
  logged; a missing required SSB row is failure. Anchor: I-1_directive_supervisor_enforcement_plane.md:89-94.
- **DI003-I01-027 [E][status]** DirectiveExecuted is specified as baseline+1 from LIVE 221/532 on
  2026-06-10 and must be re-read at edit. Anchor: I-1_directive_supervisor_enforcement_plane.md:62-63,102-103.
- **DI003-I01-028 [E][dependency]** I-2 supplies the workspace boundary that the Supervisor enforces.
  Anchor: I-1_directive_supervisor_enforcement_plane.md:113.
- **DI003-I01-029 [E][dependency]** I-1 is the Supervisor-side prerequisite for I-35 sequence rules and
  the I-31 reward endpoint. Anchor: I-1_directive_supervisor_enforcement_plane.md:114-115.
- **DI003-I01-030 [E][done-gate]** Full-form expansion is mandatory before EXECUTING. Anchor:
  I-1_directive_supervisor_enforcement_plane.md:105.
- **DI003-I01-031 [C][tier conflict]** The architectural premise says process-separated Supervisor and
  Supervisor-held credentials, but Revised Scope ships an in-process stub and defers OS separation and the
  credential vault. Tier-1 claims must follow the latter. Anchors:
  I-1_directive_supervisor_enforcement_plane.md:8-9,13,33-45,81.
- **DI003-I01-032 [A][UI]** The exact existing drawer is intentionally unresolved until EXECUTING.
  Anchor: I-1_directive_supervisor_enforcement_plane.md:55-56.
- **DI003-I01-033 [I][privacy]** Because refusal logging includes the full Directive and args, downstream
  redaction requirements may be needed; this source does not specify them. Basis:
  I-1_directive_supervisor_enforcement_plane.md:35,89-91.

### I-4 — evidence-ranked search ladder

Source:
docs/thefinalglance/I-plan/I-4_evidence_ranked_search_ladder.md

- **DI003-I04-001 [E][status]** Track 13; SMARTER-ORCH; STRONG; BUILD; effort XL; Wave 4; depends on I-2
  and I-17; blocks I-22/I-28/I-30; binds ID-1/ID-2/ID-4. Anchor:
  I-4_evidence_ranked_search_ladder.md:3.
- **DI003-I04-002 [E][gap]** No public tree search ships; current behavior is wide parallelism plus an
  LLM-judge selector. Anchor: I-4_evidence_ranked_search_ladder.md:5-9.
- **DI003-I04-003 [E][workflow]** V1 is best-of-N: generate K divergent candidates, isolate them, evaluate
  them, rank by receipts, and return a winner. Anchor: I-4_evidence_ranked_search_ladder.md:11-16,41-48.
- **DI003-I04-004 [E][concurrency]** Each candidate uses an I-2 BranchWorkspace; mutable scratch may never
  be shared across branches. Anchor: I-4_evidence_ranked_search_ladder.md:13,36-37,70,85-86.
- **DI003-I04-005 [E][ranking]** Receipt inputs are the I-17 tri-layer verdict, I-20 mutation evidence,
  and progress score. Anchor: I-4_evidence_ranked_search_ladder.md:13-15,71-72.
- **DI003-I04-006 [E][determinism]** V1 ranking precedence is tri-layer verdict first, mutation kill-rate
  second, progress score third. Anchor: I-4_evidence_ranked_search_ladder.md:47.
- **DI003-I04-007 [E][observability]** The SSB records why each branch won or lost. Anchor:
  I-4_evidence_ranked_search_ladder.md:14-15.
- **DI003-I04-008 [E][workflow/deferred]** V2 is a tournament among ranked survivors. Anchor:
  I-4_evidence_ranked_search_ladder.md:15,49-51.
- **DI003-I04-009 [E][workflow/deferred]** V3 adds PUCT over SessionDag. Anchor:
  I-4_evidence_ranked_search_ladder.md:15-16,49-52.
- **DI003-I04-010 [E][concurrency/deferred]** V3 uses virtual loss for safe parallel expansion. Anchor:
  I-4_evidence_ranked_search_ladder.md:16.
- **DI003-I04-011 [E][generation]** C-26 temperature schedules generate the K divergent hypotheses.
  Anchor: I-4_evidence_ranked_search_ladder.md:16-17,124.
- **DI003-I04-012 [E][ownership]** The generator is stateless with respect to the search tree; the
  orchestrator owns it. Anchor: I-4_evidence_ranked_search_ladder.md:6-8,16-17.
- **DI003-I04-013 [E][hard-veto]** A neural scorer may order survivors but may never resurrect an I-17
  HardVeto branch. Anchor: I-4_evidence_ranked_search_ladder.md:17-19,80-83.
- **DI003-I04-014 [E][non-goal]** LLM-judge-only selection that ignores verifier receipts is banned.
  Anchor: I-4_evidence_ranked_search_ladder.md:18-20,79-81.
- **DI003-I04-015 [E][non-goal]** Batch API parallel expansion is offline-only and banned from live search
  loops. Anchor: I-4_evidence_ranked_search_ladder.md:19-20,84.
- **DI003-I04-016 [E][rollout]** V1 must demonstrate measured value before v3/tree code ships. Anchor:
  I-4_evidence_ranked_search_ladder.md:20-22.
- **DI003-I04-017 [E][status]** No mcts module exists; I-4 is a genuine BUILD. Anchor:
  I-4_evidence_ranked_search_ladder.md:24-26.
- **DI003-I04-018 [E][dependency]** Per-branch context rebuilding must preserve ContextSlot and blake3
  prefix breakpoints through update_breakpoints/adaptive_reduce. Anchor:
  I-4_evidence_ranked_search_ladder.md:27-28,74.
- **DI003-I04-019 [E][dependency]** Existing RelationLibrary, MutationRunner, and the 30-day blake3-keyed
  EquivalentMutantCache supply verifier receipts. Anchor: I-4_evidence_ranked_search_ladder.md:29-31.
- **DI003-I04-020 [E][integration]** The ladder mirrors the production_agent_rpc three-arm dispatch shape
  at baby-step dispatch. Anchor: I-4_evidence_ranked_search_ladder.md:32-33,75.
- **DI003-I04-021 [E][recovery]** Existing recover_from_checkpoint is a live/tested primitive inherited by
  the branch store, not a stub to rebuild. Anchor: I-4_evidence_ranked_search_ladder.md:34-35.
- **DI003-I04-022 [E][dependency/status]** SessionDag/H-49 is the planned branch store and must be
  re-grepped at execution. Anchor: I-4_evidence_ranked_search_ladder.md:38.
- **DI003-I04-023 [E][tier-1/surface]** Ship search/ladder.rs, search/mod.rs, best-of-N dispatch, decision
  doc, and SearchNodeEvent registration. Anchor: I-4_evidence_ranked_search_ladder.md:40-48,54-55.
- **DI003-I04-024 [E][schema/observability]** SearchNodeEvent kinds are Expanded, Pruned, Promoted, and
  BudgetCollapsed. Anchor: I-4_evidence_ranked_search_ladder.md:44,68.
- **DI003-I04-025 [E][rollout]** Linear retry remains until the ladder has a measured first-win on at least
  three real runs. Anchor: I-4_evidence_ranked_search_ladder.md:45-46.
- **DI003-I04-026 [E][deferred]** Tournament code is gated on a measured V1 pass-rate delta on a real
  held-out fixture. Anchor: I-4_evidence_ranked_search_ladder.md:49-51.
- **DI003-I04-027 [E][deferred]** MCTS/PUCT code is gated on the V2 delta and may not block Tracks 14/15.
  Anchor: I-4_evidence_ranked_search_ladder.md:51.
- **DI003-I04-028 [E][deferred]** Cross-language verifier packs are outside V1. Anchor:
  I-4_evidence_ranked_search_ladder.md:52.
- **DI003-I04-029 [E][UI]** H-15 branch-board comparison is the intended SearchNodeEvent consumer, but V1
  may ship without that deferred Svelte surface. Anchor: I-4_evidence_ranked_search_ladder.md:60-62,89-92.
- **DI003-I04-030 [E][identifiers]** Production entry is SearchLadder::run_best_of_n; search/mod.rs
  reexports it. Anchor: I-4_evidence_ranked_search_ladder.md:64-68.
- **DI003-I04-031 [E][dependency]** Concrete calls are BranchWorkspace::spawn,
  TriLayerStack::evaluate, MutationScheduler::score, SessionDag::fork_branch,
  update_breakpoints, and build_tool_catalog. Anchor: I-4_evidence_ranked_search_ladder.md:69-75.
- **DI003-I04-032 [E][reachability]** task_runner baby-step dispatch must reach run_best_of_n; contracts-only
  callers do not satisfy R7. Anchor: I-4_evidence_ranked_search_ladder.md:76-78.
- **DI003-I04-033 [E][constraint]** Production code bans unwrap/expect, blake2/sha2, and fabricated
  measurements. Anchor: I-4_evidence_ranked_search_ladder.md:87-88.
- **DI003-I04-034 [E][acceptance]** The simple_fix fixture must emit at least one Pruned compile-fail branch
  and one Promoted survivor carrying verifier evidence, all seed-recomputable. Anchor:
  I-4_evidence_ranked_search_ladder.md:93-97.
- **DI003-I04-035 [E][acceptance]** The winner merge must be journal-verified as the highest-receipt
  survivor. Anchor: I-4_evidence_ranked_search_ladder.md:98-99.
- **DI003-I04-036 [E][broken-arm]** An LLM-judge-only arm choosing a compile-fail branch must exit nonzero
  with zero Promoted rows. Anchor: I-4_evidence_ranked_search_ladder.md:99-100.
- **DI003-I04-037 [E][done-gate]** V2/V3 code must be absent from the binary before its gates open.
  Anchor: I-4_evidence_ranked_search_ladder.md:100-101.
- **DI003-I04-038 [E][status]** SearchNodeEvent is baseline+1 from LIVE 221/532 on 2026-06-10 and must be
  re-read at edit. Anchor: I-4_evidence_ranked_search_ladder.md:114-115.
- **DI003-I04-039 [E][done-gate]** Full-form expansion is mandatory before EXECUTING. Anchor:
  I-4_evidence_ranked_search_ladder.md:21-22.
- **DI003-I04-040 [E][held-out security]** The merge gate reruns B_test off-agent in a fresh detached
  worktree from a pool separate from the executor worktree. Anchor:
  I-4_evidence_ranked_search_ladder.md:127-141.
- **DI003-I04-041 [E][privacy/anti-gaming]** The executor sees B_dev only and never touches B_test.
  Anchor: I-4_evidence_ranked_search_ladder.md:130-133,139-141.
- **DI003-I04-042 [E][evidence]** LLM-reported scores may be logged but only the independently verified score
  governs merge. Anchor: I-4_evidence_ranked_search_ladder.md:133-134.
- **DI003-I04-043 [E][failure]** No improvement against trunk/baseline rejects the candidate with
  metric-direction awareness. Anchor: I-4_evidence_ranked_search_ladder.md:134-135.
- **DI003-I04-044 [E][merge lifecycle]** Accepted merge uses no-fast-forward; conflicts abort the merge and
  restore the original checkout. Anchor: I-4_evidence_ranked_search_ladder.md:135-136.
- **DI003-I04-045 [E][security]** Merge checks protected-path patterns and required outputs. Anchor:
  I-4_evidence_ranked_search_ladder.md:136-137.
- **DI003-I04-046 [E][fail-closed]** Missing held-out eval command must BLOCK; there is no fallback to an
  LLM-reported test score. Anchor: I-4_evidence_ranked_search_ladder.md:143-146.
- **DI003-I04-047 [E][integrity]** Eval commands belong in a frozen gate manifest and must not be
  agent-writable state. Anchor: I-4_evidence_ranked_search_ladder.md:147-150.
- **DI003-I04-048 [E][determinism]** Eval-score parsing is deterministic only; an LLM parser is not evidence.
  Anchor: I-4_evidence_ranked_search_ladder.md:151-154.
- **DI003-I04-049 [E][branch lifecycle]** A candidate branch starts from current trunk HEAD so it includes
  prior merges. Anchor: I-4_evidence_ranked_search_ladder.md:156-160.
- **DI003-I04-050 [E][artifact lifecycle]** Finalize-commit filters generated artifacts from commit-worthy
  paths. Anchor: I-4_evidence_ranked_search_ladder.md:161-162.
- **DI003-I04-051 [E][branch lifecycle]** Removing a candidate worktree preserves its branch for later merge.
  Anchor: I-4_evidence_ranked_search_ladder.md:163-164.
- **DI003-I04-052 [E][concurrency]** Merge-eval and executor worktrees use distinct pools. Anchor:
  I-4_evidence_ranked_search_ladder.md:165-167.
- **DI003-I04-053 [E][recovery/concurrency]** Stale worktrees are force-removed at creation and naming
  collisions receive a timestamp suffix. Anchor: I-4_evidence_ranked_search_ladder.md:168-169.
- **DI003-I04-054 [E][platform/non-goal]** Arbor has no pool and is Unix-only; ELAI keeps I-2's warm,
  Windows-safe pool instead of copying that implementation. Anchor:
  I-4_evidence_ranked_search_ladder.md:171-173.
- **DI003-I04-055 [E][non-goal]** Do not port persistent ReAct, a 500-turn loop, premature-stop nudges, or
  LLM-selected nodes; ELAI FSM plus deterministic leaf selection replaces them. Anchor:
  I-4_evidence_ranked_search_ladder.md:175-179.
- **DI003-I04-056 [E][advisory]** High-impact/evidenced/recoverable/diverse SELECT guidance may be used only
  as an advisory tie-break; deterministic scoring remains authoritative. Anchor:
  I-4_evidence_ranked_search_ladder.md:180-183.
- **DI003-I04-057 [E][prompt]** B_dev iteration/B_test milestone and divergence-investigation guidance may
  enter the executor/eval prompt. Anchor: I-4_evidence_ranked_search_ladder.md:184-185.
- **DI003-I04-058 [E][prompt/dependency]** IDEATE prompt material is cloud-lane-only; executor method text
  is diffed and only deltas are stolen. Anchor: I-4_evidence_ranked_search_ladder.md:186-189.
- **DI003-I04-059 [A][configuration]** The prose mixes best-of-N with K candidates and supplies no fixed
  N/K product value. Anchors: I-4_evidence_ranked_search_ladder.md:12-13,16,42.
- **DI003-I04-060 [A][UI]** H-15 and SessionDag are plan-asserted seams requiring re-grep; their exact live
  APIs/surface cannot be treated as confirmed here. Anchors: I-4_evidence_ranked_search_ladder.md:38,60-62.
- **DI003-I04-061 [I][status]** The three-rung descriptions are architectural capability, but only V1 is
  current ship scope; treating V2/V3 as current features would collapse explicit gates. Basis:
  I-4_evidence_ranked_search_ladder.md:40-52.

### I-9 — error amplification telemetry

Source:
docs/thefinalglance/I-plan/I-9_error_amplification_telemetry.md

- **DI003-I09-001 [E][status]** Track 13; PROVABLE; STRONG; BUILD; effort M; Wave 4; binds ID-5.
  Anchor: I-9_error_amplification_telemetry.md:3.
- **DI003-I09-002 [E][metric]** alpha_t distinguishes amplification above 1 from contraction below 1 after
  a corrupt/low-quality step. Anchor: I-9_error_amplification_telemetry.md:5-10.
- **DI003-I09-003 [E][workflow]** ErrorPropagationHarness corrupts step t in a frozen fixture pipeline.
  Anchor: I-9_error_amplification_telemetry.md:12-15.
- **DI003-I09-004 [E][dependency]** C-28 MutationRunner supplies corruption operators; no new corruption
  primitives are built. Anchor: I-9_error_amplification_telemetry.md:13-14,28-31.
- **DI003-I09-005 [E][measurement]** The harness measures the fraction of downstream steps that recover
  versus propagate error. Anchor: I-9_error_amplification_telemetry.md:14-16.
- **DI003-I09-006 [E][determinism]** ErrorAmplificationMeasured carries alpha_t recomputable under
  ELAI_SEED. Anchor: I-9_error_amplification_telemetry.md:15-16.
- **DI003-I09-007 [E][control]** A mid-run alpha above 1 tightens C-26 temperature toward its lower bound.
  Anchor: I-9_error_amplification_telemetry.md:16-18.
- **DI003-I09-008 [E][control/config]** Temperature tightening is bounded; runaway cooling is forbidden.
  Anchor: I-9_error_amplification_telemetry.md:48-49.
- **DI003-I09-009 [E][control/dependency]** Grammar narrowing is optional through the I-34 seam if/when I-34
  is live. Anchor: I-9_error_amplification_telemetry.md:17-18.
- **DI003-I09-010 [E][evidence policy]** ELAI's own baseline must be recorded before any contraction claim;
  literature alpha values cannot become thresholds. Anchor: I-9_error_amplification_telemetry.md:18-20,25-26.
- **DI003-I09-011 [E][determinism]** The benchmark mirrors the seccodeplt runner and uses embedded
  include_str fixtures. Anchor: I-9_error_amplification_telemetry.md:20.
- **DI003-I09-012 [E][CLI]** The command is elai bench error-amplification. Anchor:
  I-9_error_amplification_telemetry.md:21,47.
- **DI003-I09-013 [E][non-goal]** Always-on per-production-trajectory monitoring is excluded due to latency
  and no current consumer. Anchor: I-9_error_amplification_telemetry.md:21-24.
- **DI003-I09-014 [E][non-goal]** Replacing C-28 operators and publishing a pre-baseline target are
  declined. Anchor: I-9_error_amplification_telemetry.md:23-26.
- **DI003-I09-015 [E][status]** No alpha harness exists; this is a genuine BUILD. Anchor:
  I-9_error_amplification_telemetry.md:32.
- **DI003-I09-016 [E][dependency]** DriftComparator supplies the first-divergence measurement shape and
  must be re-grepped. Anchor: I-9_error_amplification_telemetry.md:35-36.
- **DI003-I09-017 [E][dependency]** I-4 branches are the Wave-4 trajectories/fixture pipeline measured by
  I-9. Anchor: I-9_error_amplification_telemetry.md:37-38,133-134.
- **DI003-I09-018 [E][tier-1/surface]** Ship error_amplification_runner.rs, CLI wiring, the bounded
  temperature hook, action registration, and I-9-decisions.md. Anchor:
  I-9_error_amplification_telemetry.md:42-53.
- **DI003-I09-019 [E][lifecycle]** The first artifact in I-9-decisions.md is the local baseline, preceding
  every contraction claim. Anchor: I-9_error_amplification_telemetry.md:51-53.
- **DI003-I09-020 [E][deferred]** Per-task production alpha estimation is Tier-2 until the baseline shows
  the metric is stable enough for in-loop use. Anchor: I-9_error_amplification_telemetry.md:54-56.
- **DI003-I09-021 [E][deferred/UI]** The cockpit alpha-trend tile waits for at least three consistent
  baseline runs. Anchor: I-9_error_amplification_telemetry.md:57-58.
- **DI003-I09-022 [E][deferred]** V1 annealing is temperature-only; the grammar-tighten arm waits for I-34.
  Anchor: I-9_error_amplification_telemetry.md:59-60,95.
- **DI003-I09-023 [E][identifiers]** Produced identifiers are ErrorPropagationHarness::run,
  ErrorAmplificationMeasured, and the benchmark CLI. Anchor:
  I-9_error_amplification_telemetry.md:71-75.
- **DI003-I09-024 [E][reachability]** CLI dispatch reaches the harness; a task_runner mid-run alpha event
  reaches temperature tightening; tests alone do not satisfy R7. Anchor:
  I-9_error_amplification_telemetry.md:81-86.
- **DI003-I09-025 [E][constraint]** Production paths ban premature literature thresholds, custom mutation
  operators, premature production estimation, unwrap/expect, non-blake3 hashes, and fabricated measurements.
  Anchor: I-9_error_amplification_telemetry.md:87-96.
- **DI003-I09-026 [E][acceptance]** The fixture row and any annealing temperature event must be
  seed-recomputable. Anchor: I-9_error_amplification_telemetry.md:100-105.
- **DI003-I09-027 [E][broken-arm]** A no-repair arm must produce alpha above 1 and a repaired arm below 1.
  Anchor: I-9_error_amplification_telemetry.md:106-109.
- **DI003-I09-028 [E][status]** The contraction proposition remains DESIGN until the local baseline runs.
  Anchor: I-9_error_amplification_telemetry.md:118-120.
- **DI003-I09-029 [E][status]** ErrorAmplificationMeasured is baseline+1 from LIVE 221/532 on 2026-06-10 and
  must be re-read at edit. Anchor: I-9_error_amplification_telemetry.md:123-124.
- **DI003-I09-030 [C][lifecycle]** Revised Scope simultaneously ships a production task_runner
  mid-run-alpha hook and defers per-task live alpha estimation. The source does not explain what supplies the
  Tier-1 live measurement, so implementation status is internally unresolved. Anchors:
  I-9_error_amplification_telemetry.md:48-49,54-56,81-85.
- **DI003-I09-031 [C][summary status]** Grammar tightening and the alpha tile are architectural features but
  explicitly Tier-2, not Tier-1 deliverables. Anchors: I-9_error_amplification_telemetry.md:54-60.
- **DI003-I09-032 [A][configuration]** The temperature lower bound, tightening step, and baseline stability
  criterion are not numerically specified. Anchors: I-9_error_amplification_telemetry.md:17-18,48-49,55-58.

### I-20 — budgeted mutation scheduler

Source:
docs/thefinalglance/I-plan/I-20_budgeted_mutation_scheduler.md

- **DI003-I20-001 [E][status]** Track 15 keystone; VERIFIED; STRONG; EXTENSION; effort M-L; Wave 3; ID-5
  and IC-14. Anchor: I-20_budgeted_mutation_scheduler.md:3.
- **DI003-I20-002 [E][correction/gap]** MutationRunner and EquivalentMutantCache already ship; the real
  gaps are diff-cone selection, sampled CI scoring, and survivor-to-Directive feedback. Anchor:
  I-20_budgeted_mutation_scheduler.md:4-10,43.
- **DI003-I20-003 [E][vertical slice]** Selector, sampler, and Directive emitter ship as one unit so
  mutation can run inside the search loop. Anchor: I-20_budgeted_mutation_scheduler.md:7-10.
- **DI003-I20-004 [E][selection]** Given a hunk, traverse elai_graph reachability or impact_radius to
  enumerate data-flow-reachable functions and mutate only them. Anchor:
  I-20_budgeted_mutation_scheduler.md:12-16.
- **DI003-I20-005 [E][cache]** Diff-cone reachability is cached in the SSB for the session by blake3(diff).
  Anchor: I-20_budgeted_mutation_scheduler.md:15-16,48.
- **DI003-I20-006 [E][sampling]** Sample without replacement from the cone-selected mutants using an adaptive
  sampler. Anchor: I-20_budgeted_mutation_scheduler.md:17-18.
- **DI003-I20-007 [E][statistics]** Compute a Wilson-score confidence interval after each batch and report
  the interval plus sample count, not a raw count. Anchor: I-20_budgeted_mutation_scheduler.md:18-20.
- **DI003-I20-008 [E][budget]** Stop when CI width clears its criterion or the time budget expires.
  Anchor: I-20_budgeted_mutation_scheduler.md:18-20.
- **DI003-I20-009 [E][control]** The design calls for both fail-early and pass-early exits based on CI bounds.
  Anchor: I-20_budgeted_mutation_scheduler.md:21-22,49.
- **DI003-I20-010 [E][evidence policy]** The gate threshold comes from ELAI's own fixed-fixture baseline,
  never a paper kill-rate constant. Anchor: I-20_budgeted_mutation_scheduler.md:19-20,32-34.
- **DI003-I20-011 [E][cache schema]** Mutation results reuse EquivalentMutantCache with key
  blake3(function-content-hash, bound-test-set-hash). Anchor:
  I-20_budgeted_mutation_scheduler.md:23-25.
- **DI003-I20-012 [E][cache migration]** Adding bound-test-set hash is backwards-compatible: old entries
  miss and rerun once. Anchor: I-20_budgeted_mutation_scheduler.md:50.
- **DI003-I20-013 [E][performance]** Unchanged code returns CacheHit and runs zero mutants. Anchor:
  I-20_budgeted_mutation_scheduler.md:24-25.
- **DI003-I20-014 [E][feedback/schema]** A surviving mutant emits SurvivorDirective naming the behavioral
  distinction, operator, source line, and surviving replacement. Anchor:
  I-20_budgeted_mutation_scheduler.md:26-28,51.
- **DI003-I20-015 [E][ownership]** I-20 emits the Directive but does not own the generator retry loop or
  retry count. Anchor: I-20_budgeted_mutation_scheduler.md:27-28.
- **DI003-I20-016 [E][integration]** Proptest and Hypothesis relation adapters register as a parallel
  RelationLibrary lane behind D-3; no new dispatcher is needed. Anchor:
  I-20_budgeted_mutation_scheduler.md:29-31.
- **DI003-I20-017 [E][lifecycle/artifact]** First deploy measures the local baseline and writes
  metrics/mutation_baseline.toml before enabling a threshold. Anchor:
  I-20_budgeted_mutation_scheduler.md:32-34,52.
- **DI003-I20-018 [E][non-goal]** Exhaustive hot-path mutation, paper-derived thresholds, and a second
  mutant cache are excluded. Anchor: I-20_budgeted_mutation_scheduler.md:35-36.
- **DI003-I20-019 [E][dependency/status]** MutationRunner, EquivalentMutantCache, and RelationLibrary are
  shipped substrates to extend, not rebuild. Anchor: I-20_budgeted_mutation_scheduler.md:38-41.
- **DI003-I20-020 [A][integration]** Execution must choose between an in-process graph API and an
  impact_radius subprocess after re-grep, preferring in-process if it exists. Anchor:
  I-20_budgeted_mutation_scheduler.md:42.
- **DI003-I20-021 [E][tier-1]** Ship mutation_budget.rs selector/sampler, cache-key extension,
  SurvivorDirective, baseline fixture, MutationBudgetedRun, and PBT adapters. Anchor:
  I-20_budgeted_mutation_scheduler.md:46-54.
- **DI003-I20-022 [E][deferred]** Exhaustive mutation is an offline opt-in CLI flag only. Anchor:
  I-20_budgeted_mutation_scheduler.md:55-57.
- **DI003-I20-023 [E][deferred/dependency]** PITest and cosmic-ray adapters require separate per-language
  approval. Anchor: I-20_budgeted_mutation_scheduler.md:57.
- **DI003-I20-024 [E][surface]** MutationBudget wraps MutationRunner without changing its internals and is
  called by I-17 and the search ladder. Anchor: I-20_budgeted_mutation_scheduler.md:59-64.
- **DI003-I20-025 [E][schema/observability]** MutationBudgetedRun kinds are CiCleared, CiFailed,
  SurvivorDirective, and CacheHit. Anchor: I-20_budgeted_mutation_scheduler.md:65,69-74.
- **DI003-I20-026 [E][UI]** An existing branch-board/audit pane gets a per-hunk chip showing CI, n, verdict,
  or survivor path; no new drawer is added. Anchor: I-20_budgeted_mutation_scheduler.md:66-67,92-95.
- **DI003-I20-027 [E][identifiers]** Produced symbols are MutationBudget, SurvivorDirective,
  MutationBudgetedRun, and mutation_baseline.toml. Anchor: I-20_budgeted_mutation_scheduler.md:69-74.
- **DI003-I20-028 [E][dependency]** Consumers include MutationRunner, EquivalentMutantCache,
  RelationLibrary, C-36 MUCOCO operators, graph reachability, and I-17. Anchor:
  I-20_budgeted_mutation_scheduler.md:75-81.
- **DI003-I20-029 [E][reachability]** Production paths are I-17 run_verdict and I-4 audit expansion;
  contracts-only callers do not satisfy R7. Anchor: I-20_budgeted_mutation_scheduler.md:82-85.
- **DI003-I20-030 [E][constraint]** Hot paths require cone selection and a time box; exhaustive mode is
  offline-only. Anchor: I-20_budgeted_mutation_scheduler.md:86-87.
- **DI003-I20-031 [E][constraint]** A numeric hard-coded threshold is a BLOCK; threshold must read the
  baseline TOML. Anchor: I-20_budgeted_mutation_scheduler.md:88.
- **DI003-I20-032 [E][constraint]** A second cache or MutationRunner-internal modification is forbidden.
  Anchor: I-20_budgeted_mutation_scheduler.md:89-90.
- **DI003-I20-033 [E][constraint]** Production paths ban unwrap/expect, non-blake3 hashes, and fabricated
  CI values. Anchor: I-20_budgeted_mutation_scheduler.md:91.
- **DI003-I20-034 [E][observability]** CiCleared records n_sampled, n_total_cone, ci_lower, ci_upper, and
  time_ms under ELAI_SEED. Anchor: I-20_budgeted_mutation_scheduler.md:96-98.
- **DI003-I20-035 [E][acceptance]** An unchanged rerun must produce CacheHit with zero mutation runs;
  survivors must identify operator and line. Anchor: I-20_budgeted_mutation_scheduler.md:97-98.
- **DI003-I20-036 [E][broken-arm]** A contract must reject a literal paper threshold and prove the value
  comes from mutation_baseline.toml. Anchor: I-20_budgeted_mutation_scheduler.md:99.
- **DI003-I20-037 [E][status]** MutationBudgetedRun is baseline+1 from LIVE 221/532 on 2026-06-10 and must
  be re-read at edit. Anchor: I-20_budgeted_mutation_scheduler.md:109.
- **DI003-I20-038 [E][done-gate]** Full-form expansion is mandatory before EXECUTING. Anchor:
  I-20_budgeted_mutation_scheduler.md:118.
- **DI003-I20-039 [C][statistics]** The prose says fail early when the CI lower bound already fails the
  threshold; conventional decisive failure normally depends on the upper bound being below threshold. The
  intended formula is ambiguous and must be resolved, not silently corrected. Anchor:
  I-20_budgeted_mutation_scheduler.md:21-22.
- **DI003-I20-040 [C][summary ownership]** Bounded retries are a surrounding-generator property; I-20
  explicitly does not own the retry loop. Anchor: I-20_budgeted_mutation_scheduler.md:27-28.
- **DI003-I20-041 [C][summary cache]** Two caches are distinct: SSB session cache for diff-cone reachability
  and EquivalentMutantCache for execution results. Anchors:
  I-20_budgeted_mutation_scheduler.md:15-16,23-25.
- **DI003-I20-042 [A][configuration]** Time-box duration, batch/sample policy, retry bound, CI-width
  criterion, and gate threshold are not numerically fixed. Anchors:
  I-20_budgeted_mutation_scheduler.md:17-22,27-28,32-34.
- **DI003-I20-043 [I][lifecycle]** Because cache-key migration intentionally makes old entries miss,
  the first post-migration audit repays mutation cost once; this is implied by the explicit compatibility
  rule. Basis: I-20_budgeted_mutation_scheduler.md:50.

### I-32 — structural role scoping

Source:
docs/thefinalglance/I-plan/I-32_structural_role_scoping.md

- **DI003-I32-001 [E][status]** Track 18; STEERING-TRUST; STRONG; REFACTOR; effort M; Wave 2; binds IC-17.
  Anchor: I-32_structural_role_scoping.md:3.
- **DI003-I32-002 [E][policy]** Persona removal is evidence-gated with a per-role fixture A/B before any
  production change. Anchor: I-32_structural_role_scoping.md:5-9.
- **DI003-I32-003 [E][scope]** Executor, context-compiler, researcher, and factual/code-generation agents
  lose narrative persona blocks. Anchor: I-32_structural_role_scoping.md:11-16.
- **DI003-I32-004 [E][schema]** Replacement role contracts are schema-first with typed input/output and
  explicit output constraints. Anchor: I-32_structural_role_scoping.md:14-15.
- **DI003-I32-005 [E][prompt constraint]** Affected roles have no “you are X who Y” opener. Anchor:
  I-32_structural_role_scoping.md:14-15.
- **DI003-I32-006 [E][retention]** Auditor, critic, adversary, and anti-spiral's five lateral personas retain
  alignment-dependent framing. Anchor: I-32_structural_role_scoping.md:15-17.
- **DI003-I32-007 [E][config]** Persona blocks are set max-tokens=0 and replaced by typed constraint and I/O
  schema. Anchor: I-32_structural_role_scoping.md:19-21.
- **DI003-I32-008 [E][testing]** The old and new prompt templates run against E-19 fixtures per role.
  Anchor: I-32_structural_role_scoping.md:20-22.
- **DI003-I32-009 [E][promotion]** Each affected role flips only if new pass rate meets or exceeds its old
  baseline. Anchor: I-32_structural_role_scoping.md:21-22.
- **DI003-I32-010 [E][non-goal]** Wholesale persona removal is excluded because alignment-dependent roles
  are the carve-out. Anchor: I-32_structural_role_scoping.md:23-28.
- **DI003-I32-011 [E][status/correction]** IC-17 is binding and corrects the overbroad source-dossier reading.
  Anchor: I-32_structural_role_scoping.md:26-28.
- **DI003-I32-012 [E][dependency/status]** Phase 2C role configs, agent templates, and C-41 routing are
  plan-asserted and require re-grep; streaming role plumbing is live. Anchor:
  I-32_structural_role_scoping.md:30-35.
- **DI003-I32-013 [E][tier-1]** Tier-1 names executor, context-compiler, and researcher templates plus the
  A/B harness and retained-role proof rows. Anchor: I-32_structural_role_scoping.md:37-43.
- **DI003-I32-014 [E][observability]** RoleContractApplied emits seeded AbDelta per role and
  PersonaRetained proof for unaffected roles. Anchor: I-32_structural_role_scoping.md:40-41.
- **DI003-I32-015 [E][deferred]** Cross-role executor→auditor chain effects wait for the I-21 sprint context.
  Anchor: I-32_structural_role_scoping.md:45-47.
- **DI003-I32-016 [E][deferred]** Per-commit regression A/B for new persona blocks waits for a post-Wave-3
  CI harness. Anchor: I-32_structural_role_scoping.md:47.
- **DI003-I32-017 [E][identifiers]** Produce RoleContract, RoleContractStore, and RoleContractApplied with
  PersonaStripped/PersonaRetained/AbDelta kinds. Anchor: I-32_structural_role_scoping.md:49-62.
- **DI003-I32-018 [E][integration]** agent_dispatch prompt building switches affected roles to
  apply_to_system_prompt and reuses E-19 fixtures. Anchor: I-32_structural_role_scoping.md:52-54,63-66.
- **DI003-I32-019 [E][UI]** This is prompt-internal; AbDelta uses the existing governance metrics stream and
  no new Svelte component. Anchor: I-32_structural_role_scoping.md:55-56,75-76.
- **DI003-I32-020 [E][reachability]** dispatch_agent → apply_to_system_prompt → SSB row and the
  role-ab-probe CLI are production paths; tests alone do not satisfy R7. Anchor:
  I-32_structural_role_scoping.md:67-70.
- **DI003-I32-021 [E][constraint]** After flip, affected prompts contain zero narrative, and no flip occurs
  without a non-regressing AbDelta row. Anchor: I-32_structural_role_scoping.md:71-74.
- **DI003-I32-022 [E][constraint]** Production paths ban unwrap/expect, non-blake3 hashes, and fabricated
  measurements. Anchor: I-32_structural_role_scoping.md:74.
- **DI003-I32-023 [E][acceptance]** Seeded probe rows must show stripped affected roles and retained
  auditor/critic/adversary/lateral roles. Anchor: I-32_structural_role_scoping.md:77-80.
- **DI003-I32-024 [E][failure]** Any per-role pass-rate regression blocks production flip; journal evidence
  proves the executor prompt has a schema and no narrative. Anchor: I-32_structural_role_scoping.md:78-80.
- **DI003-I32-025 [E][status]** RoleContractApplied is baseline+1 from LIVE 221/532 and must be re-read at
  edit. Anchor: I-32_structural_role_scoping.md:62,88.
- **DI003-I32-026 [E][dependency]** Compose Phase 2C dispatch, in-place agent templates, E-19 loading, C-41
  routing, and I-21's retained evaluator persona. Anchor: I-32_structural_role_scoping.md:90-95.
- **DI003-I32-027 [A][scope]** The general rule covers all factual/code roles, while the concrete Tier-1
  list names only three; treatment of any additional factual/code roles is not enumerated. Anchors:
  I-32_structural_role_scoping.md:13-16,38-41.
- **DI003-I32-028 [A][measurement]** The local fixture gate is pass-rate ≥ baseline, but fixture composition,
  sample size, and statistical tolerance are unspecified. Anchor: I-32_structural_role_scoping.md:20-22,42.
- **DI003-I32-029 [I][status]** Persona-performance numbers motivate the refactor but are grounding claims,
  not acceptance thresholds; local E-19 results own the flip. Basis:
  I-32_structural_role_scoping.md:5-9,82-87.

### I-33 — mechanical decision governance

Source:
docs/thefinalglance/I-plan/I-33_mechanical_decision_governance.md

- **DI003-I33-001 [E][status]** Track 18; TRUST; STRONG; BUILD; effort M; Wave 2. Anchor:
  I-33_mechanical_decision_governance.md:3.
- **DI003-I33-002 [E][scope]** CEFL, I6Q, and commit-reveal apply to all orchestrator decision-type LLM
  calls such as mode picks, gate advisories, and classifications. Anchor:
  I-33_mechanical_decision_governance.md:5-11.
- **DI003-I33-003 [E][CEFL]** The orchestrator authors the complete candidate set and the model only scores
  or ranks it. Anchor: I-33_mechanical_decision_governance.md:13-16.
- **DI003-I33-004 [E][schema/security]** The response schema makes model-invented options structurally
  unparseable, eliminating selection-by-negation at the type level. Anchor:
  I-33_mechanical_decision_governance.md:13-16.
- **DI003-I33-005 [E][I6Q]** A deterministic validator enforces minimum rationale length and lexical
  diversity before acceptance. Anchor: I-33_mechanical_decision_governance.md:18-20.
- **DI003-I33-006 [E][failure]** Vacuous rationales are rejected with
  GovernancePrimitiveFired(I6qRejected). Anchor: I-33_mechanical_decision_governance.md:19-21.
- **DI003-I33-007 [E][config/security]** I6Q thresholds are orchestrator-owned configuration and cannot be
  set by the model. Anchor: I-33_mechanical_decision_governance.md:21.
- **DI003-I33-008 [E][integrity]** Every stochastic gate choice commits
  blake3(seed || choice_index) before revealing the choice. Anchor:
  I-33_mechanical_decision_governance.md:23-25.
- **DI003-I33-009 [E][evidence]** SSB ordering proves commit-before-reveal; reveal-before-commit is an R2
  failure. Anchor: I-33_mechanical_decision_governance.md:24-25.
- **DI003-I33-010 [E][non-goal]** Model-authored candidates and reveal-before-commit are explicitly excluded.
  Anchor: I-33_mechanical_decision_governance.md:27-29.
- **DI003-I33-011 [E][metrics]** Each seeded run ledgers CDL cosmetic-deadlock rate, DIU deferral-information
  utilization, and FSR framing-success rate. Anchor: I-33_mechanical_decision_governance.md:31-33.
- **DI003-I33-012 [E][evidence policy]** Governance decoupling under task stress is measured locally before
  it is claimed. Anchor: I-33_mechanical_decision_governance.md:31-33.
- **DI003-I33-013 [E][status]** CEFL/I6Q/commit-reveal are grep-confirmed absent, so I-33 is a BUILD.
  Anchor: I-33_mechanical_decision_governance.md:35-36.
- **DI003-I33-014 [E][dependency/status]** Phase 3F, D-16, B-22/H-37, and C-26 are plan-asserted; E-25
  prompt_decision is present at file level. Anchor: I-33_mechanical_decision_governance.md:37-41.
- **DI003-I33-015 [E][tier-1/identifiers]** Ship CeflRequest, I6qValidator, CommitRevealRecord, wrapped
  call sites, action rows, fixtures, UI tile, and decision doc. Anchor:
  I-33_mechanical_decision_governance.md:43-51.
- **DI003-I33-016 [E][schema/observability]** GovernancePrimitiveFired kinds are CeflExternalized,
  I6qRejected, and CommitRevealSealed. Anchor: I-33_mechanical_decision_governance.md:45-48,65-70.
- **DI003-I33-017 [E][testing]** Fixtures cover framing inversion, vacuous rationale, and out-of-order
  commit. Anchor: I-33_mechanical_decision_governance.md:50.
- **DI003-I33-018 [E][deferred]** Cross-gate DIU correlation requires at least three real runs. Anchor:
  I-33_mechanical_decision_governance.md:53-55.
- **DI003-I33-019 [E][deferred]** Adaptive I6Q calibration waits for local ID-5 measurement. Anchor:
  I-33_mechanical_decision_governance.md:55.
- **DI003-I33-020 [E][integration]** Every gate-advisory/mode/classification prompt_decision call is wrapped
  by CEFL and I6Q; Phase 3F routes rows and D-16 handles refusal. Anchor:
  I-33_mechanical_decision_governance.md:57-63,71-75.
- **DI003-I33-021 [E][UI]** A mandatory new governance metrics chip in MonitorWindow/SettingsWindow renders
  CDL/DIU/FSR through Tauri. Anchor: I-33_mechanical_decision_governance.md:49,62-63,85-86.
- **DI003-I33-022 [E][reachability]** Production prompt_decision and stochastic-gate paths must emit rows;
  tests alone do not satisfy R7. Anchor: I-33_mechanical_decision_governance.md:76-79.
- **DI003-I33-023 [E][constraint]** Parsing a model-invented option or revealing before commit proves the
  wrapper broken. Anchor: I-33_mechanical_decision_governance.md:80-82.
- **DI003-I33-024 [E][constraint]** Production bans unwrap/expect, non-blake3 hashes, and fabricated
  CDL/DIU/FSR. Anchor: I-33_mechanical_decision_governance.md:83-84.
- **DI003-I33-025 [E][acceptance]** Every decision row carries the candidate list; vacuous input produces
  I6qRejected; stochastic choices have commit_row_id < reveal_row_id under the same seed. Anchor:
  I-33_mechanical_decision_governance.md:87-90.
- **DI003-I33-026 [E][acceptance]** Reversing field order must yield the same decision, out-of-order commit
  exits nonzero, and the UI passes cross_lang_diff. Anchor: I-33_mechanical_decision_governance.md:88-90.
- **DI003-I33-027 [E][status]** GovernancePrimitiveFired is baseline+1 from LIVE 221/532 and must be
  re-read at edit. Anchor: I-33_mechanical_decision_governance.md:70,97.
- **DI003-I33-028 [E][dependency]** Compose prompt_decision, Phase 3F routing, D-16 refusal, and C-26 schema;
  do not fork any. Anchor: I-33_mechanical_decision_governance.md:100-104.
- **DI003-I33-029 [A][configuration]** Minimum rationale length, lexical-diversity threshold, and later
  adaptive calibration rule are unspecified. Anchors: I-33_mechanical_decision_governance.md:18-21,53-55.
- **DI003-I33-030 [A][scope]** “All decision-type calls” is categorical, but only three call families and
  plan-asserted sites are enumerated; a complete call-site census is deferred to execution. Anchors:
  I-33_mechanical_decision_governance.md:11,37-41,47.
- **DI003-I33-031 [I][security]** Commit-reveal proves ordering/integrity, not unbiased randomness; no entropy
  source beyond seed is specified. Basis: I-33_mechanical_decision_governance.md:23-25.

### I-38 — QDELTA and statistical floor

Source:
docs/thefinalglance/I-plan/I-38_qdelta_statistical_floor.md

- **DI003-I38-001 [E][status]** Track 18; PROVABLE; STRONG; BUILD; effort M; Wave 1; binds ID-5.
  Anchor: I-38_qdelta_statistical_floor.md:3.
- **DI003-I38-002 [E][gap]** SAFEDELTA exists, but QDELTA and mechanical statistical-floor enforcement do
  not. Anchor: I-38_qdelta_statistical_floor.md:5-9.
- **DI003-I38-003 [E][metric]** QDELTA is defective diffs reaching a reviewer per 100 tasks, compared with
  gates ON versus OFF. Anchor: I-38_qdelta_statistical_floor.md:11-14.
- **DI003-I38-004 [E][schema]** A defective diff includes held-out test failures, post-merge surviving I-20
  mutants, and regressions. Anchor: I-38_qdelta_statistical_floor.md:12-14.
- **DI003-I38-005 [E][statistics]** Confidence intervals are bootstrapped and clustered by repository.
  Anchor: I-38_qdelta_statistical_floor.md:13-14.
- **DI003-I38-006 [E][framing/non-goal]** Reporting is gate ablation only; SWE-bench Verified/leaderboard
  rank is explicitly declined. Anchor: I-38_qdelta_statistical_floor.md:14-16.
- **DI003-I38-007 [E][policy]** Every gates-ON run requires a gates-OFF counterpart on the same tasks.
  Anchor: I-38_qdelta_statistical_floor.md:18-20.
- **DI003-I38-008 [E][policy]** Each condition requires at least five seeds and reports SEM. Anchor:
  I-38_qdelta_statistical_floor.md:20.
- **DI003-I38-009 [E][policy]** Multi-step reliability claims report pass^k and log k explicitly.
  Anchor: I-38_qdelta_statistical_floor.md:21.
- **DI003-I38-010 [E][economics]** Every headline reports dollars per task beside accuracy. Anchor:
  I-38_qdelta_statistical_floor.md:22.
- **DI003-I38-011 [E][corpus/privacy]** Runs use date-stamped task cuts and surface D-20 memorization probe
  results as contamination evidence. Anchor: I-38_qdelta_statistical_floor.md:23.
- **DI003-I38-012 [E][fail-closed]** floor_check validates config at startup and exits nonzero before compute
  on violations. Anchor: I-38_qdelta_statistical_floor.md:25-27.
- **DI003-I38-013 [E][contamination]** QdeltaConfig has contamination_probe and executes D-20 before calling
  a run clean. Anchor: I-38_qdelta_statistical_floor.md:29-30.
- **DI003-I38-014 [E][status]** IN-9 confirms QDELTA absent; SAFEDELTA is only a shape precedent. Anchor:
  I-38_qdelta_statistical_floor.md:32-34.
- **DI003-I38-015 [E][dependency/status]** D-6/E-54 corpus, D-20, E-49, and SAFEDELTA are plan-asserted and
  re-grepped; I-20 supplies mutant data. Anchor: I-38_qdelta_statistical_floor.md:34-39.
- **DI003-I38-016 [E][tier-1/identifiers]** Ship QdeltaConfig, run_qdelta, floor_check, bootstrap_ci,
  deterministic methodology generation, QdeltaMeasured, fixtures, probe field, UI tile, and decision doc.
  Anchor: I-38_qdelta_statistical_floor.md:41-51.
- **DI003-I38-017 [E][schema/observability]** QdeltaMeasured includes ON/OFF delta, CI, seed_count, paired,
  dollars/task, and contamination-probe result. Anchor: I-38_qdelta_statistical_floor.md:45-47.
- **DI003-I38-018 [E][testing]** Ship a valid paired ≥5-seed fixture and a 3-seed unpaired fixture the
  harness refuses. Anchor: I-38_qdelta_statistical_floor.md:48.
- **DI003-I38-019 [E][deferred]** Full-corpus evaluation is RunPod-gated; Tier-1 is fixture-scale.
  Anchor: I-38_qdelta_statistical_floor.md:53-55.
- **DI003-I38-020 [E][deferred]** Cross-repository CI-width analysis waits for at least three repositories
  of real data. Anchor: I-38_qdelta_statistical_floor.md:55.
- **DI003-I38-021 [E][integration]** QdeltaConfig mirrors SafedeltaConfig and shares D-6/E-54 corpus plus
  13-gram filtering. Anchor: I-38_qdelta_statistical_floor.md:57-60.
- **DI003-I38-022 [E][integration]** D-20 supplies contamination, I-20 rows supply surviving mutants, and
  E-49 supplies metrics plumbing. Anchor: I-38_qdelta_statistical_floor.md:61-62,72-77.
- **DI003-I38-023 [E][UI]** A new QDELTA Svelte/Tauri tile sits beside SAFEDELTA and must pass
  cross_lang_diff. Anchor: I-38_qdelta_statistical_floor.md:63,88-89.
- **DI003-I38-024 [E][reachability]** elai bench qdelta reaches run_qdelta; floor_check runs at entry; tests
  alone do not satisfy R7. Anchor: I-38_qdelta_statistical_floor.md:78-81.
- **DI003-I38-025 [E][constraint]** Unpaired or fewer-than-five-seed runs may emit no headline/QdeltaMeasured
  row. Anchor: I-38_qdelta_statistical_floor.md:82-85.
- **DI003-I38-026 [E][constraint]** Methodology starts with ablation-only framing; CI is seed-recomputable;
  production bans unwrap/expect and non-blake3 hashes. Anchor: I-38_qdelta_statistical_floor.md:85-87.
- **DI003-I38-027 [E][acceptance]** A valid run emits all required payload fields and is byte-recomputable;
  the underpowered run emits no QdeltaMeasured. Anchor: I-38_qdelta_statistical_floor.md:90-92.
- **DI003-I38-028 [E][acceptance]** Refusal names the floor violation; methodology regenerates
  deterministically and names D-20; the UI passes cross-language checking. Anchor:
  I-38_qdelta_statistical_floor.md:93.
- **DI003-I38-029 [E][status]** QdeltaMeasured is baseline+1 from LIVE 221/532 and must be re-read at edit.
  Anchor: I-38_qdelta_statistical_floor.md:70,103.
- **DI003-I38-030 [E][dependency]** SAFEDELTA, corpus loading, memorization probe, mutation scoring, and
  metrics plumbing must be composed, never reimplemented. Anchor: I-38_qdelta_statistical_floor.md:105-110.
- **DI003-I38-031 [A][statistics]** Bootstrap resample count, CI level, SEM presentation, defect weighting,
  and cost-accounting method are not specified. Anchors: I-38_qdelta_statistical_floor.md:11-23,43-47.
- **DI003-I38-032 [A][contamination]** The source requires D-20 to run but does not define which probe result
  rejects a task cut. Anchor: I-38_qdelta_statistical_floor.md:29-30.
- **DI003-I38-033 [I][lifecycle]** Because floor_check exits before compute, absence of QdeltaMeasured is the
  fail-closed evidence rather than a failure event row. Basis: I-38_qdelta_statistical_floor.md:25-27,91-93.

### I-41 — spec-time critique lenses

Source:
docs/thefinalglance/I-plan/I-41_spec_time_critique_lenses.md

- **DI003-I41-001 [E][status]** Track 19; PLURAL-DOUBT; MODERATE; BUILD; effort M; Wave 2; binds ID-4 and
  the ban on load-bearing LLM judges. Anchor: I-41_spec_time_critique_lenses.md:3.
- **DI003-I41-002 [E][timing]** Lenses run on prose/specifications before deterministic artifacts exist.
  Anchor: I-41_spec_time_critique_lenses.md:4-8.
- **DI003-I41-003 [E][trust]** LLM-judged findings are noisy advisory input only; they never accept or block
  and do not replace off-agent gates. Anchor: I-41_spec_time_critique_lenses.md:8-11.
- **DI003-I41-004 [E][lens]** L1 Assumption Excavator names interface assumptions, classifies
  GROUNDED/VERIFY/INGEST, and verifies nothing itself. Anchor: I-41_spec_time_critique_lenses.md:14-16.
- **DI003-I41-005 [E][lens]** L2 is Implied Completeness Detector. Anchor:
  I-41_spec_time_critique_lenses.md:16.
- **DI003-I41-006 [E][lens]** L3 Ambiguity Mapper composes B-38: the lens provides a human map while the
  scorer supplies the gate-time number. Anchor: I-41_spec_time_critique_lenses.md:16-18.
- **DI003-I41-007 [E][lens]** L4 is Gap Analyzer. Anchor: I-41_spec_time_critique_lenses.md:18.
- **DI003-I41-008 [E][lens]** L5 Pre-Implementation Architect flags a producer lacking an in-plan consumer
  and smoke trigger. Anchor: I-41_spec_time_critique_lenses.md:18-19.
- **DI003-I41-009 [E][lens]** L6 Public-Interface/API-Contract Validator mirrors cross_lang_diff and the
  E-45 action-type golden lock at design time. Anchor: I-41_spec_time_critique_lenses.md:19-20.
- **DI003-I41-010 [E][lens]** L7 is Documentation Validator. Anchor:
  I-41_spec_time_critique_lenses.md:20-21.
- **DI003-I41-011 [E][lens]** L8 Anxiety Reader is the catch-all and runs last. Anchor:
  I-41_spec_time_critique_lenses.md:21.
- **DI003-I41-012 [E][schema]** Finding fields are lens, severity, location, claim, why_it_matters,
  suggested_resolution, and confidence. Anchor: I-41_spec_time_critique_lenses.md:21-23.
- **DI003-I41-013 [E][schema]** Severity values are BLOCKER-CANDIDATE, CONCERN, and NIT. Anchor:
  I-41_spec_time_critique_lenses.md:21-22.
- **DI003-I41-014 [E][grounding]** Location is mandatory and may be a spec heading, file:line, symbol, or
  quoted phrase. Anchor: I-41_spec_time_critique_lenses.md:22-24.
- **DI003-I41-015 [E][fail-safe]** Anchorless findings are dropped rather than surfaced. Anchor:
  I-41_spec_time_critique_lenses.md:23-24.
- **DI003-I41-016 [E][budget]** Each lens emits at most eight findings per pass. Anchor:
  I-41_spec_time_critique_lenses.md:24.
- **DI003-I41-017 [E][dedup]** Findings deduplicate across lenses by location plus claim. Anchor:
  I-41_spec_time_critique_lenses.md:24-25.
- **DI003-I41-018 [E][routing/config]** SIMPLE uses 2–3 lenses; HEAVY uses the full eight. Anchor:
  I-41_spec_time_critique_lenses.md:25-26.
- **DI003-I41-019 [E][non-goal]** Do not import agents-and-pipelines; lenses are clean native reauthors to
  avoid the E-43 import gate. Anchor: I-41_spec_time_critique_lenses.md:26-28.
- **DI003-I41-020 [E][non-goal]** Do not run these prose-time lenses on shipped code. Anchor:
  I-41_spec_time_critique_lenses.md:28.
- **DI003-I41-021 [E][integration/status]** B-40, B-38, E-45/cross_lang_diff, and Phase 2B are shipped or
  plan-asserted composition seams; D-13/E-6/E-10 are distinct critique surfaces. Anchor:
  I-41_spec_time_critique_lenses.md:30-40.
- **DI003-I41-022 [E][status]** No LensSet/spec-time critique module exists; I-41 is net-new. Anchor:
  I-41_spec_time_critique_lenses.md:41-42.
- **DI003-I41-023 [E][tier-1]** Ship lenses.rs with eight prompts, LensSet, Finding, anchor drop, caps, and
  dedup. Anchor: I-41_spec_time_critique_lenses.md:44-47.
- **DI003-I41-024 [E][CLI]** elai critique accepts a spec path, optional lens list, and simple/heavy risk.
  Anchor: I-41_spec_time_critique_lenses.md:48-49.
- **DI003-I41-025 [E][artifact]** CLI output is workflow/reasoning/<artifact>-critique.md plus advisory SSB
  rows. Anchor: I-41_spec_time_critique_lenses.md:48-49.
- **DI003-I41-026 [E][integration]** B-40 idea-critic calls LensSet only as an advisory surfacer and never
  uses it for framing accept/reject. Anchor: I-41_spec_time_critique_lenses.md:50-51.
- **DI003-I41-027 [E][schema/observability]** CritiqueLensFinding is advisory=true with AdvisoryFinding,
  LensRun, and DroppedNoAnchor arms. Anchor: I-41_spec_time_critique_lenses.md:52,72-78.
- **DI003-I41-028 [E][test/invariant]** A contract proves no lens output reaches any gate-verdict path.
  Anchor: I-41_spec_time_critique_lenses.md:53.
- **DI003-I41-029 [E][deferred]** Automatic injection into the nine-step planning skill follows runtime
  proof. Anchor: I-41_spec_time_critique_lenses.md:55-57.
- **DI003-I41-030 [E][deferred]** Per-lens positive/negative eval JSONL waits for accumulated findings.
  Anchor: I-41_spec_time_critique_lenses.md:58.
- **DI003-I41-031 [E][deferred]** GEPA self-optimization waits for a labelled corpus. Anchor:
  I-41_spec_time_critique_lenses.md:59.
- **DI003-I41-032 [E][surface]** Create lenses.rs, critique_cmd.rs, the advisory-only contract, and decision
  doc; modify idea_framing. Anchor: I-41_spec_time_critique_lenses.md:61-67.
- **DI003-I41-033 [E][UI/human-loop]** Existing ReasoningDrawer or framing panel shows nonblocking findings;
  BLOCKER-CANDIDATE routes to a human and never auto-blocks. Anchor:
  I-41_spec_time_critique_lenses.md:68-70,97-100.
- **DI003-I41-034 [E][identifiers]** Produced symbols are LensSet, Finding, critique_cmd::run,
  CritiqueLensFinding, and the advisory-only contract. Anchor:
  I-41_spec_time_critique_lenses.md:72-79.
- **DI003-I41-035 [E][dependency]** LensSet consumes B-38, E-45/cross_lang_diff, Phase 2B
  VerificationMethod, B-40, and B-37 runtime. Anchor: I-41_spec_time_critique_lenses.md:80-86.
- **DI003-I41-036 [E][reachability]** Both CLI and idea-framing production paths emit findings; test-only
  reachability does not satisfy R7. Anchor: I-41_spec_time_critique_lenses.md:87-90.
- **DI003-I41-037 [E][constraint]** No verdict path, anchorless surfacing, primitive reimplementation, or
  upstream repository import is allowed. Anchor: I-41_spec_time_critique_lenses.md:91-96.
- **DI003-I41-038 [E][constraint]** Production paths ban unwrap/expect and non-blake3 hashes. Anchor:
  I-41_spec_time_critique_lenses.md:96.
- **DI003-I41-039 [E][acceptance]** Planted L1 and L4 issues emit anchored AdvisoryFinding rows;
  anchorless output emits DroppedNoAnchor; every row is advisory. Anchor:
  I-41_spec_time_critique_lenses.md:101-105.
- **DI003-I41-040 [E][acceptance]** Markdown omits anchorless findings and the run emits zero gate-verdict
  rows. Anchor: I-41_spec_time_critique_lenses.md:106-108.
- **DI003-I41-041 [E][status]** CritiqueLensFinding is baseline+1 from LIVE 221/532 and must be re-read at
  edit. Anchor: I-41_spec_time_critique_lenses.md:77-78,119.
- **DI003-I41-042 [E][dependency]** I-42 consumes LensSet; L8 remains distinct from trajectory/ship-time
  critics. Anchor: I-41_spec_time_critique_lenses.md:121-127.
- **DI003-I41-043 [A][UI]** The exact existing drawer is unresolved between ReasoningDrawer and B-40's
  framing panel. Anchor: I-41_spec_time_critique_lenses.md:68-70.
- **DI003-I41-044 [A][dedup]** The source gives a location/claim key but no normalization, cross-pass
  persistence, or semantic-duplicate policy. Anchor: I-41_spec_time_critique_lenses.md:24-25.
- **DI003-I41-045 [I][trust]** BLOCKER-CANDIDATE is severity/routing metadata, not a gate outcome; the human
  may act, but the lens itself cannot block. Basis: I-41_spec_time_critique_lenses.md:21-24,68-70.

### I-50 — subscription auth broker

Source:
docs/thefinalglance/I-plan/I-50_subscription_auth_broker.md

- **DI003-I50-001 [E][status]** Track 20; REACH; STRONG for SL-3 and MEDIUM for gajae semantics/SL-10;
  EXTENSION; effort M; Wave 4. Anchor: I-50_subscription_auth_broker.md:3-4.
- **DI003-I50-002 [E][capability]** One broker/gateway/storage abstraction owns token acquisition, refresh,
  storage, and provider linkage. Anchor: I-50_subscription_auth_broker.md:6-11.
- **DI003-I50-003 [E][dependency/done-gate]** Executor must read the named gajae auth-broker,
  auth-gateway, and auth-storage sources before EXECUTING. Anchor:
  I-50_subscription_auth_broker.md:13-15.
- **DI003-I50-004 [E][API]** AuthProvider exposes acquire, refresh, revoke, and status. Anchor:
  I-50_subscription_auth_broker.md:16-17.
- **DI003-I50-005 [E][lifecycle]** auth_broker.rs owns token lifecycle and storage through a broker/storage
  facade. Anchor: I-50_subscription_auth_broker.md:16-20,45-47.
- **DI003-I50-006 [E][migration]** Existing Anthropic oauth.rs becomes the first implementation behind the
  trait. Anchor: I-50_subscription_auth_broker.md:17-18.
- **DI003-I50-007 [E][compatibility]** Migration preserves token-file bytes and refresh semantics so existing
  users observe no behavior change. Anchor: I-50_subscription_auth_broker.md:18-20.
- **DI003-I50-008 [E][storage/security]** Credentials remain in the current oauth.rs storage location unless
  the user confirms a new backend. Anchor: I-50_subscription_auth_broker.md:19-20.
- **DI003-I50-009 [E][non-goal]** This session adds no AWS/Azure or second-provider flow. Anchor:
  I-50_subscription_auth_broker.md:21-22.
- **DI003-I50-010 [E][non-goal/dependency]** No keychain/credential-manager dependency is added. Anchor:
  I-50_subscription_auth_broker.md:22-23.
- **DI003-I50-011 [E][non-goal]** CLI-bridge authentication remains vendor-CLI-based and unchanged. Anchor:
  I-50_subscription_auth_broker.md:23.
- **DI003-I50-012 [E][status]** oauth.rs is shipped and Anthropic-only; existing runtimes/adapters are kept,
  not replaced. Anchor: I-50_subscription_auth_broker.md:25-27.
- **DI003-I50-013 [E][tier-1]** Ship the trait, broker, and byte-compatible Anthropic migration. Anchor:
  I-50_subscription_auth_broker.md:29-31.
- **DI003-I50-014 [E][deferred]** AWS/Azure credentials and any second subscription provider are deferred
  with I-49 Tier-2. Anchor: I-50_subscription_auth_broker.md:31.
- **DI003-I50-015 [E][integration]** Refactor oauth.rs behind auth_broker, migrate adapter call sites, and
  make runtime adapters read credentials through the broker. Anchor:
  I-50_subscription_auth_broker.md:33-36.
- **DI003-I50-016 [E][UI]** Existing RuntimePanel/SettingsWindow rows show per-provider authentication status;
  no new component is required. Anchor: I-50_subscription_auth_broker.md:35-36.
- **DI003-I50-017 [E][UI/security]** OpenCode may inspire Providers/Connect/verified-status layout only;
  implementation is clean-room and credential replay is forbidden. Anchor:
  I-50_subscription_auth_broker.md:37-43.
- **DI003-I50-018 [E][coordination]** J-30 owns the wider BYOK/custom-endpoint panel sharing the surface.
  Anchor: I-50_subscription_auth_broker.md:42-43.
- **DI003-I50-019 [E][schema/observability]** AuthBrokerEvent kinds are TokenAcquired, Refreshed,
  ProviderLinked, and RefreshFailed. Anchor: I-50_subscription_auth_broker.md:45-47,69-73.
- **DI003-I50-020 [E][reachability]** Every authenticating adapter's production request builder, at minimum
  Anthropic API, must flow through the broker; contracts-only callers do not satisfy R7. Anchor:
  I-50_subscription_auth_broker.md:48-52.
- **DI003-I50-021 [E][privacy]** Journal rows contain provider ID and event kind but never token material.
  Anchor: I-50_subscription_auth_broker.md:53-55.
- **DI003-I50-022 [E][failure]** Refresh failures are typed, emit RefreshFailed, and may not be silent.
  Anchor: I-50_subscription_auth_broker.md:53-55.
- **DI003-I50-023 [E][constraint]** Production paths ban behavior drift, new storage locations, unwrap, and
  expect. Anchor: I-50_subscription_auth_broker.md:53-55.
- **DI003-I50-024 [E][acceptance]** A real Anthropic request emits TokenAcquired/Refreshed; an expired fixture
  refreshes then succeeds; a failed refresh emits a typed error without hanging. Anchor:
  I-50_subscription_auth_broker.md:56-60.
- **DI003-I50-025 [E][acceptance]** Token-store fixtures remain byte-identical, token-material journal grep
  is empty, and successful auth without a broker row fails. Anchor:
  I-50_subscription_auth_broker.md:61-63.
- **DI003-I50-026 [E][dependency]** Auth events join I-18's chained record; I-49 consumes future credential
  implementations. Anchor: I-50_subscription_auth_broker.md:69-71.
- **DI003-I50-027 [E][status]** AuthBrokerEvent is baseline+1 from LIVE 221/532 and must be re-read at edit.
  Anchor: I-50_subscription_auth_broker.md:71-73.
- **DI003-I50-028 [A][API/schema]** AuthProvider method signatures, token status model, storage facade schema,
  and refresh retry policy are not specified. Anchors: I-50_subscription_auth_broker.md:16-20,45-49.
- **DI003-I50-029 [I][scope]** “Multi-provider” describes the extensible seam, not current provider breadth;
  the only shipped implementation in this session is Anthropic. Basis:
  I-50_subscription_auth_broker.md:16-23,29-31.

### I-52 — checkpoint/rewind agent tool

Source:
docs/thefinalglance/I-plan/I-52_checkpoint_rewind_tool.md

- **DI003-I52-001 [E][status]** Track 20; CONCURRENCY-SAFETY; STRONG SL-6; ACTIVATION; effort S-M; Wave 3.
  Anchor: I-52_checkpoint_rewind_tool.md:3-4.
- **DI003-I52-002 [E][activation]** Existing edit-checkpoint machinery and CLI/contracts ship, but agents
  cannot call it; I-52 exposes it rather than building a store. Anchor:
  I-52_checkpoint_rewind_tool.md:6-12.
- **DI003-I52-003 [E][scope]** The tool supports cheap single-agent, in-branch speculation; I-2 remains the
  parallel-branch mechanism. Anchor: I-52_checkpoint_rewind_tool.md:10-12.
- **DI003-I52-004 [E][dependency/done-gate]** Executor reads gajae checkpoint.ts for ergonomics before
  EXECUTING. Anchor: I-52_checkpoint_rewind_tool.md:14-16.
- **DI003-I52-005 [E][tool]** save creates a labelled snapshot through edit_checkpoints. Anchor:
  I-52_checkpoint_rewind_tool.md:16-18.
- **DI003-I52-006 [E][tool]** rewind restores a label subject to divergence and session guards. Anchor:
  I-52_checkpoint_rewind_tool.md:17-19.
- **DI003-I52-007 [E][tool]** diff reports changes since a label. Anchor:
  I-52_checkpoint_rewind_tool.md:18.
- **DI003-I52-008 [E][safety]** Rewind refuses if files outside the checkpointed set diverged. Anchor:
  I-52_checkpoint_rewind_tool.md:17-18.
- **DI003-I52-009 [E][security]** An agent may rewind only checkpoints from its own session. Anchor:
  I-52_checkpoint_rewind_tool.md:18-19.
- **DI003-I52-010 [E][coordination]** If H-49 ships first, I-52 becomes a tool surface over H-49's store and
  reuses EditHistoryEvent. Anchor: I-52_checkpoint_rewind_tool.md:20-23.
- **DI003-I52-011 [E][invariant]** No implementation order permits a second snapshot store. Anchor:
  I-52_checkpoint_rewind_tool.md:21-23.
- **DI003-I52-012 [E][non-goal]** Redo/branching semantics, git-worktree integration, cross-session travel,
  and new UI are excluded. Anchor: I-52_checkpoint_rewind_tool.md:24-25.
- **DI003-I52-013 [E][status]** edit_checkpoints.rs, checkpoint_cmd.rs, and two E-25 contract tests exist.
  Anchor: I-52_checkpoint_rewind_tool.md:27-30.
- **DI003-I52-014 [E][flag/default]** If the substrate is still OFF, activation flips it per W-2:
  default-ON behind an ELAI_*_DISABLE flag or an allowlist row. Anchor:
  I-52_checkpoint_rewind_tool.md:30-32.
- **DI003-I52-015 [E][lifecycle/status]** Executor must recheck whether file_ops still discards preimages and
  whether the gate remains off. Anchor: I-52_checkpoint_rewind_tool.md:30-32.
- **DI003-I52-016 [E][tier-1]** Ship the in-process three-arm tool, session guard, divergence refusal, and
  contract test. Anchor: I-52_checkpoint_rewind_tool.md:34-36.
- **DI003-I52-017 [E][deferred]** H-49 store migration and automatic checkpoints before risky edits are
  deferred. Anchor: I-52_checkpoint_rewind_tool.md:36.
- **DI003-I52-018 [E][integration]** Register through the existing three-arm catalog and extend
  edit_checkpoints only with label/scope lookup. Anchor: I-52_checkpoint_rewind_tool.md:38-40.
- **DI003-I52-019 [E][UI]** Checkpoint rows use the existing session/transcript drawer; no component is
  added. Anchor: I-52_checkpoint_rewind_tool.md:40-41.
- **DI003-I52-020 [E][schema/observability]** Default event is CheckpointToolEvent with Saved, Rewound,
  Diffed, and RefusedDiverged; it is conditional on H-49 event reuse. Anchor:
  I-52_checkpoint_rewind_tool.md:43-45,65-70.
- **DI003-I52-021 [E][dependency]** Consume edit_checkpoints store, catalog registration, and file_ops
  preimage capture; never reimplement the store. Anchor: I-52_checkpoint_rewind_tool.md:46-47.
- **DI003-I52-022 [E][reachability]** A live worker's production tool dispatch must reach the tool;
  contracts-only callers do not satisfy R7. Anchor: I-52_checkpoint_rewind_tool.md:48-49.
- **DI003-I52-023 [E][failure]** Divergence refusal names the complete divergent-path set and may not
  silently overwrite files. Anchor: I-52_checkpoint_rewind_tool.md:50-52,55-59.
- **DI003-I52-024 [E][constraint]** Production paths ban unwrap/expect and claiming activation while the
  substrate remains gated off. Anchor: I-52_checkpoint_rewind_tool.md:50-52.
- **DI003-I52-025 [E][observability]** save then rewind emits Saved then Rewound with the file set; divergence
  emits a refusal naming the path. Anchor: I-52_checkpoint_rewind_tool.md:53-56.
- **DI003-I52-026 [E][acceptance]** Post-rewind bytes must blake3-match checkpoint preimages. Anchor:
  I-52_checkpoint_rewind_tool.md:57.
- **DI003-I52-027 [E][broken-arm]** Cross-session labels are refused; silent overwrite of a diverged file
  fails R2. Anchor: I-52_checkpoint_rewind_tool.md:57-59.
- **DI003-I52-028 [E][dependency]** H-49 supplies future divergence semantics, I-2 is the parallel sibling,
  and I-8 rollback-to-trunk remains outside this tool. Anchor: I-52_checkpoint_rewind_tool.md:65-68.
- **DI003-I52-029 [E][status]** CheckpointToolEvent is baseline+1 from LIVE 221/532 unless H-49 reuse fires;
  action registration must be re-read at edit. Anchor: I-52_checkpoint_rewind_tool.md:69-70.
- **DI003-I52-030 [A][event schema]** Whether I-52 adds CheckpointToolEvent or reuses EditHistoryEvent depends
  on landing order and is explicitly decided at edit. Anchors: I-52_checkpoint_rewind_tool.md:20-23,43-45.
- **DI003-I52-031 [A][flag]** The exact disable-flag name or allowlist-row schema is not specified.
  Anchor: I-52_checkpoint_rewind_tool.md:30-32.

## Cross-source interaction and lifecycle ledger

- **DI003-X-001 [E][enforcement]** I-1 creates the Supervisor-side execution boundary later consumed by I-35
  and I-31; its Tier-1 process isolation remains incomplete. Refs: DI003-I01-013, DI003-I01-015,
  DI003-I01-029.
- **DI003-X-002 [E][search chain]** I-4 requires I-2 isolation and I-17 verdicts, consumes I-20 mutation
  receipts, and supplies real trajectories to I-9. Refs: DI003-I04-001, DI003-I04-004, DI003-I04-005,
  DI003-I09-017.
- **DI003-X-003 [E][quality chain]** I-20's sampled-survivor records both rank I-4 branches and populate the
  mutant leg of I-38 QDELTA. Refs: DI003-I20-024, DI003-I38-004, DI003-I38-022.
- **DI003-X-004 [E][gated rollout]** I-4 V1 precedes tournament/PUCT; I-9 baseline precedes production-wide
  live estimation; I-32 local A/B precedes prompt flip; I-38 fixture tier precedes full-corpus RunPod work.
  Refs: DI003-I04-025–027, DI003-I09-019–022, DI003-I32-008–009, DI003-I38-019.
- **DI003-X-005 [E][local-baseline doctrine]** I-9, I-20, I-32, I-33, and I-38 all prohibit promoting
  external/paper claims into policy before local evidence. Refs: DI003-I09-010, DI003-I20-010,
  DI003-I32-002, DI003-I33-012, DI003-I38-016.
- **DI003-X-006 [E][determinism]** I-4 search receipts, I-9 alpha, I-20 mutation CI, I-33 governance metrics,
  and I-38 QDELTA evidence are required to be seed-recomputable. Refs: DI003-I04-034, DI003-I09-006,
  DI003-I20-034, DI003-I33-025, DI003-I38-027.
- **DI003-X-007 [E][SSB lifecycle]** Every source introduces or conditionally reuses an SSB action family;
  all baseline counts are provisional and must be re-read at edit. Refs: DI003-I01-027, DI003-I04-038,
  DI003-I09-029, DI003-I20-037, DI003-I32-025, DI003-I33-027, DI003-I38-029, DI003-I41-041,
  DI003-I50-027, DI003-I52-029.
- **DI003-X-008 [E][advisory boundary]** I-41 findings never decide, while I-4 and I-38 make selection/reporting
  authoritative only through deterministic receipts and floor checks. Refs: DI003-I41-003,
  DI003-I04-013–014, DI003-I38-012.
- **DI003-X-009 [E][provider chain]** I-50 preserves current Anthropic subscription auth while providing the
  future credential seam for I-49; auth events join I-18's chained record. Refs: DI003-I50-006–008,
  DI003-I50-014, DI003-I50-026.
- **DI003-X-010 [E][rewind coordination]** I-52 is single-agent/in-branch, I-2 remains parallel isolation,
  H-49 may own the shared store/events, and I-8 remains rollback-to-trunk. Refs: DI003-I52-003,
  DI003-I52-010–012, DI003-I52-028.
- **DI003-X-011 [E][UI reuse/new surfaces]** I-1, I-20, I-32, I-41, I-50, and I-52 reuse existing surfaces;
  I-33 and I-38 require new metric tiles; I-4 and I-9 defer their principal visualizations. Refs:
  DI003-I01-024, DI003-I04-029, DI003-I09-021, DI003-I20-026, DI003-I32-019, DI003-I33-021,
  DI003-I38-023, DI003-I41-033, DI003-I50-016, DI003-I52-019.
- **DI003-X-012 [E][concurrency]** I-4 uses isolated candidate and detached eval worktrees; I-52 explicitly
  avoids worktree overhead and cannot replace I-2 parallelism. Refs: DI003-I04-004,
  DI003-I04-040, DI003-I04-052, DI003-I52-003, DI003-I52-012.

## Explicit ambiguity and conflict register

No unlisted source contradiction was silently normalized. The bounded issues found are:

- **REG-001 [C]** I-1's target architecture versus Tier-1 process/credential scope. Ref: DI003-I01-031.
- **REG-002 [A]** I-1's exact existing drawer. Ref: DI003-I01-032.
- **REG-003 [A]** I-4's N/K notation and unspecified candidate count. Ref: DI003-I04-059.
- **REG-004 [A]** I-4's plan-asserted H-15/SessionDag seams. Ref: DI003-I04-060.
- **REG-005 [C]** I-9 ships a mid-run hook while deferring live per-task estimation. Ref: DI003-I09-030.
- **REG-006 [C]** I-9 grammar and UI are deferred despite summary-level capability wording. Ref:
  DI003-I09-031.
- **REG-007 [A]** I-9 annealing/stability values are unspecified. Ref: DI003-I09-032.
- **REG-008 [A]** I-20 graph API is selected only at execution. Ref: DI003-I20-020.
- **REG-009 [C]** I-20's fail-early CI-bound wording is statistically ambiguous. Ref: DI003-I20-039.
- **REG-010 [C]** I-20 does not own bounded retries. Ref: DI003-I20-040.
- **REG-011 [C]** I-20 uses two different cache surfaces. Ref: DI003-I20-041.
- **REG-012 [A]** I-20 numeric budgets/thresholds are unspecified. Ref: DI003-I20-042.
- **REG-013 [A]** I-32's broad factual/code rule versus three-role Tier-1 list. Ref: DI003-I32-027.
- **REG-014 [A]** I-32 A/B statistical details are unspecified. Ref: DI003-I32-028.
- **REG-015 [A]** I-33 I6Q numeric configuration and complete call-site census are unspecified. Refs:
  DI003-I33-029, DI003-I33-030.
- **REG-016 [A]** I-38 statistical and contamination rejection details are unspecified. Refs:
  DI003-I38-031, DI003-I38-032.
- **REG-017 [A]** I-41 drawer and dedup normalization are unresolved. Refs: DI003-I41-043,
  DI003-I41-044.
- **REG-018 [A]** I-50 method/schema/retry details are unresolved. Ref: DI003-I50-028.
- **REG-019 [A]** I-52 event-family and flag schema depend on edit-time state. Refs: DI003-I52-030,
  DI003-I52-031.

## Orientation-summary additions and corrections

These deltas compare the deep read to
docs/thefinalglance/feature-discovery/09-I-SUMMARY_INDEX.discovery.md.
Delta entries cross-reference primary atoms rather than duplicating them.

- **SD-I01-01 [CORRECTION]** Summary I-1.02/I-1.05 flatten the target architecture into current scope;
  Tier-1 is an in-process stub and the OS split/credential vault are deferred. Refs: DI003-I01-013,
  DI003-I01-015–016, DI003-I01-031.
- **SD-I01-02 [ADDITION]** Exact Directive fields, event arms, production reachability, Tier-1 UI posture,
  and full acceptance/bans were absent. Refs: DI003-I01-003, DI003-I01-014, DI003-I01-021–026.
- **SD-I04-01 [CORRECTION]** Candidate notation is not a declared N constant, and V2/V3 are gated—not
  current ship scope. Refs: DI003-I04-025–028, DI003-I04-059, DI003-I04-061.
- **SD-I04-02 [ADDITION]** Deterministic V1 ranking precedence, all SearchNodeEvent kinds, exact rollout
  evidence, and binary-absence gate were omitted. Refs: DI003-I04-006, DI003-I04-024–027,
  DI003-I04-034–037.
- **SD-I04-03 [ADDITION]** The full J-11 held-out merge gate—hidden B_test, detached pool, fail-closed
  command, frozen manifest, deterministic parser, merge recovery, path/output guards—was omitted. Refs:
  DI003-I04-040–048.
- **SD-I04-04 [ADDITION]** J-11 worktree lifecycle and prompt do/do-not-port rules were omitted. Refs:
  DI003-I04-049–058.
- **SD-I09-01 [CORRECTION]** Summary I-9.07/I-9.11 present grammar tightening and the tile without their
  Tier-2 gates; V1 is temperature-only and the tile needs three baselines. Refs: DI003-I09-021–022,
  DI003-I09-031.
- **SD-I09-02 [ADDITION]** Embedded-fixture determinism, bounded cooling, baseline-first artifact, explicit
  non-goals, and the live-estimation contradiction were omitted. Refs: DI003-I09-008, DI003-I09-011,
  DI003-I09-013–014, DI003-I09-019, DI003-I09-030.
- **SD-I20-01 [CORRECTION]** Summary I-20.08 conflates reachability and mutant caches; summary I-20.14
  over-assigns retry ownership. Refs: DI003-I20-005, DI003-I20-011, DI003-I20-015,
  DI003-I20-040–041.
- **SD-I20-02 [ADDITION]** Sampling-without-replacement, Wilson CI, cache migration, first-deploy baseline,
  exact event arms, deferred engines, and CI-bound ambiguity were omitted. Refs: DI003-I20-006–012,
  DI003-I20-017, DI003-I20-022–025, DI003-I20-039.
- **SD-I32-01 [ADDITION]** The broad factual/code-role rule versus concrete three-role Tier-1 list,
  max-tokens=0 mechanism, complete retained roles/event kinds, and deferred cross-role/CI work were omitted.
  Refs: DI003-I32-003, DI003-I32-006–007, DI003-I32-013–017, DI003-I32-027.
- **SD-I33-01 [ADDITION]** Orchestrator-owned I6Q config, exact commitment formula/order proof,
  selection-by-negation guarantee, D-16 routing, mandatory UI, and deferred calibration/correlation were
  omitted. Refs: DI003-I33-004, DI003-I33-007–009, DI003-I33-018–021.
- **SD-I38-01 [ADDITION]** Pre-compute refusal/no-row evidence, exact payload/config identifiers, D-20
  pre-clean execution, 13-gram reuse, full-corpus/cross-repo gates, and unresolved statistical parameters
  were omitted. Refs: DI003-I38-012–013, DI003-I38-016–022, DI003-I38-025–032.
- **SD-I41-01 [ADDITION]** Per-lens semantics, complete Finding/severity schema, 2–3 versus 8 risk routing,
  LensRun arm, output artifact, and native-reauthor/code non-goals were omitted. Refs:
  DI003-I41-004–020, DI003-I41-024–027.
- **SD-I41-02 [ADDITION]** Deferred planning-skill injection, eval corpora, GEPA optimization, exact
  production reachability, and UI/dedup ambiguities were omitted. Refs: DI003-I41-029–036,
  DI003-I41-043–044.
- **SD-I50-01 [ADDITION]** No CLI-bridge change/keychain dependency, existing-location rule, AWS/Azure
  deferral, ProviderLinked arm, clean-room UI constraint, all-adapter reachability, and no-hang failure
  evidence were omitted. Refs: DI003-I50-008–011, DI003-I50-014, DI003-I50-017,
  DI003-I50-019–024.
- **SD-I52-01 [ADDITION]** Exact W-2 default posture, preimage/gate recheck, conditional EditHistoryEvent
  reuse, Diffed/RefusedDiverged arms, redo/worktree exclusions, and auto-checkpoint deferral were omitted.
  Refs: DI003-I52-010–017, DI003-I52-020.

## Zero-omission and duplicate audit

- Processed originals: **10/10** manifest rows.
- Verified logical lines: **1157/1157**.
- Verified bytes: **89623/89623**.
- Per-file feature atoms: **367**.
- Cross-source interaction atoms: **12**; total primary atomic count: **379**.
- Classification count across primary atoms: **348 explicit, 8 inferred, 17 ambiguous, 6 conflicting/corrective**.
- Primary atomic IDs are unique; no identifier is reused.
- Repeated source prose (grounding, wiring contract, composes-with, and observable-effect) was normalized to
  one primary atom and referenced elsewhere rather than counted twice.
- Explicit, inferred, ambiguous, and conflicting items are separated by tags and consolidated again in the
  ambiguity/conflict register.
- Orientation-summary deltas are cross-references, not duplicate feature atoms.
- No source line within the assigned originals was skipped; headings/end markers without feature content do
  not receive artificial atoms.

## Output digest

Output SHA-256 (canonical self-hash; compute after replacing the digest value on the next line with
the literal token OUTPUT_SHA256): **26060cb61295d2ef8a09adacc485da483a8f7f938db257cf9f07d9bed48b55e8**
