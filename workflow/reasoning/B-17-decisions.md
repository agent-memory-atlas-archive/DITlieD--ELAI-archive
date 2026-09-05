B-17 — Ground the Vacuous Gates: Decisions
==========================================

Author session: s-28972
Phase: B-17 (TESTING_FIRST -> EXECUTING)
Date: 2026-04-26
Status: ALL 6 TASKS SHIPPED. Task 1 (gate_liveness.rs) s-28972,
        Task 2 (dashboard frequency panel — inline OrchestrationDrawer
        VERDICT COUNTS block, NOT GatesPanel.svelte) silent commit eacd440,
        Task 3 (.github/workflows/gate-liveness.yml CI) s-91480.
        Backend + dashboard + CI all live. Remaining: B-2/B-4 wiring stubs
        (2 ignored tests) + W4 FSM-transition perf test (DECISION-D).

DECISION-A — Test file location
-------------------------------
.elai_cc/crates/orchestrator/tests/gate_liveness.rs (sibling to
safety_gate_integration.rs).

Rationale: the integration test must reach symbols across three crates
simultaneously — elai_orchestrator (gates, fsm, task_runner shapes),
elai_reasoning (engine, GrcError, FeasibilityOutcome), elai_knowledge
(DegradationCascade, SearchMode, EmbeddingCircuitBreaker). Only the
orchestrator integration test crate has all three as in-scope dependencies
(verified: crates/orchestrator/Cargo.toml [dependencies]). Putting the file
under crates/contracts/tests/ would have required adding elai-knowledge to
the contracts dev-deps list — a wider blast radius than B-17 needs.

Spec wording said "tests/contracts/gate_liveness.rs"; the project layout has
no tests/contracts/ dir. We follow the B-8 precedent (safety_gate_integration
.rs lives at .elai_cc/crates/orchestrator/tests/, NOT contracts/) so
gate_liveness.rs sits as its sibling. This is a project-layout correction,
not a semantic change.

DECISION-B — ENV_LOCK reuse
---------------------------
We DUPLICATE the
    static ENV_LOCK: Mutex<()> = Mutex::new(());
pattern by name (NOT promote it to a shared module). safety_gate_integration
.rs:42 keeps its private static; gate_liveness.rs:62 declares an identical
one with the same identifier.

Rationale: cargo's integration test isolation makes each test file its own
binary; sharing a mutex across binaries is impossible without a real
test-support crate. The same name keeps grep-based refactor in lockstep with
safety_gate_integration.rs and provider_signals_contract.rs (the two
prior-art callers). The risk of duplicate-name confusion is bounded — both
files explicitly comment on the cross-file naming invariant.

DECISION-C — Naming for wiring-blocked stubs
--------------------------------------------
Suffix `_pending_b<N>_wiring` + `#[ignore = "..."]` attribute.

We use BOTH because:
  - the suffix makes future grep-find easy
    (`rg pending_b._wiring` returns every wiring follow-up across the suite)
  - `#[ignore]` keeps the test in the compiled inventory (caught by `cargo
    test -- --list`) without contributing to pass/fail counts
  - removing the `#[ignore]` is a one-line edit when the wiring B-N lands;
    no rename required

The three stubs as shipped:
  search_mode_cascade_pending_b4_wiring        (B-4 / pipeline.rs:520 hardcoded Hybrid)
  supervision_manager_escalation_pending_b6_wiring (B-6 / SupervisionManager not wired into task_runner)
  cli_interactive_gate_pending_b2_wiring       (B-2 / CliInteractiveGate defined cli/main.rs:7753, never instantiated)

Each test asserts library-level behaviour (DegradationCascade.evaluate,
SupervisionManager.handle_death, run_interactive_gate) so removing the
`#[ignore]` once production wiring lands turns it into a real liveness check.

DECISION-D — Latency measurement scope
--------------------------------------
The performance test
    gate_chain_under_400ms_with_individual_under_100ms
measures sequential gate-evaluation work using std::time::Instant. The four
wired gate calls are timed individually and as a chain:

  Gate                                Per-gate budget   Reference (Phase 2B)
  PipelineGrc.ground_plan             100ms             p99 = 188 us
  GroundingSweepGate.check            100ms             vector op, sub-us
  PipelineGrc.generate_csq            100ms             regex scan, ms-class
  PipelineGrc.check_inline_feasibility 100ms            engine call, ms-class
  Full chain                           400ms

Phase 2B reference: docs/thefinalglance/audit-implementation/phase-2B.md
recorded ground_plan p99 = 188 us. The 100ms budget is ~530x the worst
measured case; the chain budget of 400ms is ~2125x. A real regression will
trip the assertion long before pure-CPU code reaches the budget.

WHY NOT a full FSM-transition perf test: a real transition through
TaskRunner requires constructing BspEngine + agent_pool + recovery + indexing
+ ClockProvider + KnowledgeEngine + a populated TransitionContext. The
fixture cost dwarfs the work being measured and would push B-17 well past
the one-session budget. We document the scope split explicitly in the test's
doc comment so a follow-up B-N implementer who wants TaskRunner-level perf
knows exactly what is and isn't covered today.

DECISION-E — INTERACTIVE block default action
---------------------------------------------
When run_interactive_gate is wired in autonomous (no-dashboard) mode and
the gate would BLOCK with no user surface to dismiss, the FSM treats the
verdict as REJECT (fail closed).

  - The gate verdict surfaces as InteractiveGateOutcome::Rejected.
  - Fsm::transition stays at the source state (no state advancement).
  - tracing::warn emits "interactive gate blocked in autonomous mode;
    treating as Reject" so the operator can audit the block in logs.
  - No SSB action_type is added — the existing fsm transition trace and the
    tracing log are sufficient. (Anti-stale: SsbActionType extensions are a
    14-site coordinated change per gotchas.md SSB_ACTION_TYPE_14_SITE_EXTENSION;
    we explicitly avoid one here.)

WHY fail-closed: in autonomous mode the user cannot intervene. If we silently
auto-Approved, every gate becomes vacuous in autonomous runs — exactly the
B-17 anti-goal. Approve is a positive signal we have no reason to manufacture.

WHY NOT log+re-prompt: there is nobody to prompt in backend autonomous mode.
A re-prompt loop converges to either Approve (same vacuous problem) or
infinite spin. Reject is the only stable choice.

The cli_interactive_gate_pending_b2_wiring stub asserts this contract on a
LocalMockGate that returns GateDecision::Reject; the assertion checks that
run_interactive_gate yields InteractiveGateOutcome::Rejected and that the
gate is presented exactly once (no recursion).

PER-GATE STATUS MATRIX (post-B-17)
----------------------------------
GATE                          | PROD WIRING TODAY | LIVENESS COVERAGE
------------------------------|-------------------|-------------------
GroundingSweepGate            | WIRED (task_runner.rs:3337) | grounding_sweep_gate_fires_on_ungrounded_claim (active)
CSQ Critical                  | WIRED (task_runner.rs:3762) | csq_critical_blocks_destructive_call (active)
Inline feasibility            | WIRED (task_runner.rs:3825-3833) | inline_feasibility_blocks_propagate_to_baby_step_outcome (active)
Auditor anti-rubber-stamp     | WIRED (engine.rs:297-308 + agent_pool seed) | auditor_anti_rubber_stamp_rejects_findingless_converge (active)
SearchMode cascade            | NOT WIRED (pipeline.rs:520 hardcodes Hybrid) | search_mode_cascade_pending_b4_wiring (#[ignore], B-4)
SupervisionManager escalation | WIRED (task_runner.rs:4879 emit_baby_step_recovery -> mgr.handle_death, s-91480) | supervision_manager_escalation (active, #[ignore] removed s-91480)
CliInteractiveGate            | NOT WIRED (cli/main.rs:7753 unused struct)   | cli_interactive_gate_pending_b2_wiring (#[ignore], B-2)

SPEC AMENDMENT
--------------
NONE required. B-8 already shipped GroundingSweepGate with the canonical
EvidenceError shape (gate_name="grounding_sweep", missing_artifacts=claims).
No verdict semantics changed during B-17. Task 2 (dashboard frequency panel)
shipped as an INLINE OrchestrationDrawer.svelte VERDICT COUNTS block
(L415-430) — NOT a separate GatesPanel.svelte (that is B-38 AMBIGUITY, a
different concern). Task 3 (.github/workflows/gate-liveness.yml) shipped
s-91480 pinned to ubuntu-24.04.

OUT-OF-SCOPE WORK (remaining follow-ups)
----------------------------------------
- Task 2 dashboard surface: SHIPPED (silent commit eacd440) as inline
  OrchestrationDrawer.svelte VERDICT COUNTS block (L415-430) backed by
  get_gate_verdict_counts (commands.rs:3436) + aggregate_gate_verdict_counts
  (ssb_viewer.rs:3638) + stub.ts:1182 + drawers.svelte.ts gateVerdictCounts
  $state. Coverage = EvidenceGateEvaluated rows only (DECISION-E: broader
  multi-action-type join out of scope).
- Task 3 CI yml: SHIPPED (s-91480) at .github/workflows/gate-liveness.yml,
  ubuntu-24.04, push+PR trigger, runs check_gate_liveness.sh.
- Production-FSM-transition perf test: requires a TaskRunner harness; B-N
  follow-up (W4, DECISION-D).
- B-2 / B-4 wiring: each is a separate batch step; the stubs in
  gate_liveness.rs are pre-positioned to flip live by removing `#[ignore]`.
  B-6 SupervisionManager wiring landed s-91480 (stub activated).

VERIFICATION (this session)
---------------------------
- cargo check -p elai-orchestrator --tests --message-format short: clean
  (only pre-existing warnings: PhaseNode unused, web_research_orchestrator
  clock field — neither introduced by B-17)
- cargo test -p elai-orchestrator --test gate_liveness:
    test result: ok. 5 passed; 0 failed; 3 ignored
- bash .elai_cc/scripts/check_gate_liveness.sh: exits 0 with parsed
  passed=5 / failed=0 / ignored=3
- All five active tests run in <0.01s (well under 100ms per gate budget).

## Session s-91480 (2026-05-09) — B-17 follow-up
WARN closures:
  W1=CLOSED: grounding_sweep test now asserts err.missing_artifacts is non-empty
             (gate_liveness.rs grounding_sweep_gate_fires_on_ungrounded_claim)
  W2=CLOSED: inline_feasibility test now asserts result.claim.contains("CONTRADICTS_PLAN")
             (gate_liveness.rs inline_feasibility_blocks_propagate_to_baby_step_outcome)
  W3=PARTIAL: check_gate_liveness.bat now has a fallback parse path for cases
              where primary positional-token parse yields PASSED=0 on cargo exit 0.
              Primary parse unchanged (fragility remains for non-zero-PASSED false
              zero; documented — fixing properly requires regex extraction which
              cmd.exe cannot do cleanly without external tools)
  W4=DEFERRED: latency test still covers gate-evaluation only. Full FSM-transition
               perf test requires TaskRunner harness; cost exceeds B-17 budget.
               Documented in DECISION-D. No change.

Ignored test reactivations: 1 of 3
  supervision_manager_escalation_pending_b6_wiring: #[ignore] REMOVED.
    B-6 production wiring confirmed at task_runner.rs:4879 (emit_baby_step_recovery
    calls mgr.handle_death under AsyncMutex). Library-level test passes independently.
  search_mode_cascade_pending_b4_wiring: still #[ignore]. pipeline.rs:578 now uses
    self.search_mode (field, not hardcoded Hybrid) but cascade is still evaluated
    only at startup — per-query path does not re-evaluate. B-4 blocker remains.
  cli_interactive_gate_pending_b2_wiring: still #[ignore]. CliInteractiveGate defined
    at runtime_builder.rs:389 and implements InteractiveGate, but is never instantiated
    and passed to the FSM pipeline. B-2 blocker remains.

SearchMode cascade wiring: MINIMUM
  Added `search_mode: SearchMode` field to QueryPipeline (pipeline.rs:62).
  Constructor initializes to SearchMode::Hybrid (same as prior hardcoded value).
  `set_search_mode(&mut self, mode: SearchMode)` builder method added.
  QueryTrace::new now passes `self.search_mode` instead of literal SearchMode::Hybrid.
  Deferred: wiring cascade.evaluate() result into this field at the call sites
  in cli/main.rs and on circuit-breaker callbacks requires B-4 plumbing.

Dashboard get_evidence_gates: stub-NOT-needed
  commands.rs:760 already queries SSB via recent_evidence_gates(50) and maps to JSON.
  stub.ts:497 already uses tryInvoke (tries real Tauri invoke, falls back to mock data).
  OrchestrationDrawer.svelte EVIDENCE GATES section (L144-158) renders gate verdicts.
  No changes needed — already wired in dashboard-v2 Tier 3 Drawers work.

CI workflow: CREATED (candidate)
  .github/workflows/ did not exist; directory created + gate-liveness.yml written.
  Pinned to ubuntu-24.04 (not ubuntu-latest). Uses dtolnay/rust-toolchain@stable.
  Trigger: push + PR on all branches. Runner choice may need refinement for
  Windows-only deps (see note in YAML file). No existing CI files to reference —
  this is the first workflow file in the project.

cargo check: PASS
  elai-knowledge: PASS (SearchMode field addition + set_search_mode)
  elai-orchestrator: PASS (gate_liveness.rs changes)
  elai-dashboard-v2: PASS

Test results (gate_liveness.rs):
  test result: ok. 6 passed; 0 failed; 2 ignored
  (6 = 5 prior active + 1 newly activated supervision_manager stub)

Remaining open:
  - B-4 (SearchMode cascade per-query): pipeline.rs field added, call-site wiring deferred
  - B-2 (CliInteractiveGate instantiation and FSM integration): still pending
  - W3 full fix: cmd.exe positional parse fragility; acceptable for internal tool
  - W4 (FSM-transition perf test): deferred (TaskRunner harness too large for B-17)

## Session docs-drift-repair (2026-07-03) — B-17 Task 2 + Task 3 docs sync
SILENT WIRING RECONCILED: commit eacd440 wired Task 2 (dashboard frequency
panel) without updating this decisions doc or changelog.md. This session
repairs the docs drift; NO code changes (R-3 scope contract — code is
frozen, verify-only).

Task 2 (dashboard frequency panel) — SHIPPED via eacd440, surfaced here:
  - get_gate_verdict_counts (commands.rs:3436) — #[tauri::command] returning
    [{gate_id, pass_count, block_count, total}] sorted by total desc.
  - aggregate_gate_verdict_counts (ssb_viewer.rs:3638) — SQL GROUP BY
    gate_id, verdict over EvidenceGateEvaluated rows, capped at
    DRAWER_QUERY_HARD_CAP=500.
  - Registered in lib.rs:712 invoke_handler alongside get_evidence_gates.
  - stub.ts:1182 get_gate_verdict_counts() -> tryInvoke<GateVerdictCount[]>.
  - types.ts:720 GateVerdictCount interface.
  - drawers.svelte.ts:395 gateVerdictCounts $state + L406-420 Promise.all
    refresh + L438 getter.
  - OrchestrationDrawer.svelte:418-421 {#each d.gateVerdictCounts as cnt}
    inline VERDICT COUNTS block (NOT GatesPanel.svelte — that is B-38).

Task 3 (CI yml) — SHIPPED s-91480:
  .github/workflows/gate-liveness.yml, ubuntu-24.04, push+PR trigger,
  runs check_gate_liveness.sh. Confirmed live.

DOCS-DRIFT REPAIR (this session, markdown-only):
  - L7 status: flipped from "deferred" to "ALL 6 TASKS SHIPPED".
  - L121-131 matrix: SupervisionManager row flipped from NOT WIRED to
    WIRED (task_runner.rs:4879, s-91480); test name updated to
    supervision_manager_escalation (active, #[ignore] removed).
  - L137-150 OUT-OF-SCOPE: rewritten — Task 2 + Task 3 now marked SHIPPED
    with shipped-surface pointers; only B-2/B-4 wiring + W4 perf test
    remain as true follow-ups.
  - changelog.md: additive entry appended for the silent eacd440 Task 2
    wiring (commands.rs + ssb_viewer.rs + OrchestrationDrawer + types.ts
    + stub.ts + drawers.svelte.ts).

VERIFICATION (this session, all green):
  A. cargo check -p elai-orchestrator --tests: clean (only pre-existing
     warnings — PhaseNode unused, web_research_orchestrator clock, etc.).
  B. cargo test -p elai-orchestrator --test gate_liveness -- --nocapture:
     6 passed; 0 failed; 2 ignored (B-2 + B-4 stubs).
  C. cargo check -p elai-dashboard-v2: clean (only pre-existing warnings).
  D. bash .elai_cc/scripts/check_gate_liveness.sh: exit 0, parsed
     passed=6 / failed=0 / ignored=2.
  E. rg get_gate_verdict_counts lib.rs: 1 hit at L712 (registered).
  F. rg gateVerdictCounts OrchestrationDrawer.svelte: hits at L418 + L421
     (rendered).

NO code changes (R-3 scope contract honored). NO files recreated.
