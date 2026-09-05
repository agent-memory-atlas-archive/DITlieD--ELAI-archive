Phase 3I — Deterministic Simulation Testing (DST) — Hostile Implementation Audit

Audit date: 2026-04-24
Auditor stance: hostile / verification-by-evidence
Spec source: docs/thefinalglance/summary/phase-3-summary.md (## PHASE 3I, lines 897-1000)
Code root: F:\.ELAI_workflow\.elai_cc\crates\orchestrator\

================================================================
SUMMARY VERDICT
================================================================

PARTIAL — strong skeleton, weak fault-injection layer, false metric claim.

The core reference-model + proptest-state-machine plumbing is genuinely
implemented and compiles into a unit-test-scope state machine. BSP determinism
is real (100 reps, byte-identical). The 5 adversarial domains all have
test coverage. However:

1. madsim is COMPLETELY ABSENT. Zero dependency, zero cfg gate, zero
   madsim::test, zero check_determinism. Spec 3I.1 / 3I.6 / 3I.8 explicitly
   require it (or a documented alternative). No alternative is documented in
   code.
2. The 10K+ random sequence claim is NOT enforced. ProptestConfig::with_cases(256)
   is hardcoded; no PROPTEST_CASES env wiring; no CI knob.
3. metrics/phase_3I.toml claims `madsim_compile_time_sec = 83.0` from a
   `RUSTFLAGS="--cfg madsim" cargo check --workspace` run — this would have
   no effect because no source uses cfg(madsim) and no crate depends on madsim.
   Claim is unfalsifiable / fabricated.
4. ipc_simulation.rs uses a homegrown DstIpcGateway with synthetic faults,
   NOT madsim's NetSim. Functionally exercises the SimulatorRuntime fault
   queue but does not provide deterministic packet-level fault injection,
   send_latency Duration ranges, NetSim::clog_link, or buggify_with_prob().

================================================================
PER-SUB-STEP VERDICTS
================================================================

----------------------------------------------------------------
3I.1 MADSIM WINDOWS SPIKE
----------------------------------------------------------------
Spec: Add madsim v0.2.34 dev-dep + cfg(madsim) gate. Document failure /
alternative.

Verdict: FAIL

Evidence:
- crates/orchestrator/Cargo.toml dev-dependencies (lines 51-55) lists
  proptest=1.11, proptest-state-machine=0.8, tempfile=3, tokio test-util.
  NO madsim entry.
- Workspace-wide grep for "madsim" in any Cargo.toml returns nothing.
- Workspace-wide grep for cfg(madsim) in src/ returns nothing.
- No SPIKE-decision document at workflow/reasoning/phase-3I-madsim*.md
  was located by the audit (a doc may exist outside the crate, but no
  artefact in code makes the spike's outcome traceable).
- The metrics file phase_3I.toml line 31-34 claims a 83s compile with
  RUSTFLAGS="--cfg madsim" — since no crate references the cfg flag, this
  value reflects an unrelated baseline cargo check, not a madsim build.

The ALTERNATIVE path (manual deterministic scheduler wrapping
SeededClock+SeededRng) IS implemented as `SimulatorRuntime` in
src/dst.rs:45-135. This satisfies the SPIRIT of "alternative with
equivalent fault injection" but the spec required this be DOCUMENTED as
the chosen branch when madsim fails. No such documentation found in
code, comments, or metrics rationale.

----------------------------------------------------------------
3I.2 SKELETON
----------------------------------------------------------------
Spec: SimulatorRuntime{SeededClock+SeededRng+madsim}, FsmReferenceModel
implementing ReferenceStateMachine with init_state/transitions/
preconditions/apply, FsmEvent enum (10 variants), StateMachineTest impl,
proptest-state-machine v0.8+ + cfg(madsim) IPC modules.

Verdict: PARTIAL PASS (madsim portion missing)

Evidence:
- src/dst.rs:45-53 SimulatorRuntime exists with elai_core::SeededClock
  (line 46), elai_core::SeededRng (line 47), VecDeque<(SimFault,Duration)>
  fault_queue, sim_time, fsm_state, ssb_row_count, active_agents.
  SimulatorRuntime contains NO madsim runtime handle.
- FsmEvent enum at src/dst.rs:141-153 has exactly 10 variants:
  PhaseTransition, AgentSpawn, AgentCrash, AgentCleanup, ToolCall,
  FileWrite, GrcStep, SsbWrite, ConfigChange, SuperstepBoundary. MATCHES SPEC.
- ReferenceState at src/dst.rs:159-184 tracks fsm_state, ssb_row_count,
  active_agents, crashed_agents, cleaned_agents, last_value_channels,
  topic_channels, ephemeral_present. UntrackedValue is structurally
  excluded (no field). MATCHES SPEC INTENT.
- impl ReferenceStateMachine for FsmReferenceModel at src/dst.rs:318-436
  with init_state (line 322), transitions (line 326), preconditions
  (line 429), apply (line 433). Lives inside #[cfg(test)] mod tests, so
  NOT visible to integration tests — those use bare inherent methods on
  src/dst.rs:219-306. Spec footnote in tests/dst_reference_model.rs:1-7
  acknowledges this design choice.
- impl StateMachineTest for ActualSystem at src/dst.rs:438-483 wires
  init_test (returns ActualSystem by value), apply (returns SUT by value
  per spec), check_invariants (delegates to FsmReferenceModel).
- Cargo.toml dev-deps confirm proptest=1.11, proptest-state-machine=0.8.
- AgentCleanup variant correctly transitions crashed -> cleaned
  (preconditions src/dst.rs:228, apply src/dst.rs:265-268).
- SuperstepBoundary apply (src/dst.rs:276-278) only clears
  ephemeral_present — does NOT model the spec's full "two-phase"
  semantics (clear_non_accumulating + execute + clear_ephemeral +
  evidence_gate). This is a SIMPLIFICATION, not a bug, but does not match
  the composite event description in spec 3I.2.

Reference model LOC: 491 lines for src/dst.rs (entire file). Real FSM
crates/orchestrator/src/fsm.rs = 1442 lines. Ratio = 34.0%, exceeds the
"<30% real FSM LOC" adversarial threshold (spec 3I.2 + 3I.8).

----------------------------------------------------------------
3I.3 VERIFY (cargo test --workspace; warnings <= baseline)
----------------------------------------------------------------
Verdict: NOT-VERIFIABLE-IN-AUDIT

Audit cannot run cargo per main-model rules; no audit-trail artefact in
metrics/phase_3I.toml records the workspace test exit code or a warning
delta. proptest-regressions/dst.txt presence indicates at least one
shrunk run occurred. The recorded `bugs_found = 5` claim (metrics line
11) is asserted but unverifiable from code evidence alone.

----------------------------------------------------------------
3I.4 TEST-WRITE: REFERENCE MODEL (PROPTEST_CASES=10000)
----------------------------------------------------------------
Spec: prop_state_machine!{sequential 1..200 => FsmStateMachineTest}.
PROPTEST_CASES=10000 in CI + 10-min timeout, 256 local.
check_determinism(seed, config, bare fn pointer) — STATIC.

Verdict: PARTIAL PASS (10K not enforced; check_determinism missing)

Evidence:
- src/dst.rs:485-490 contains:
    prop_state_machine! {
        #![proptest_config(ProptestConfig::with_cases(256))]
        #[test]
        fn fsm_reference_model_matches(sequential 1..200 => ActualSystem);
    }
  Sequential range 1..200 MATCHES SPEC. The 256-case count is hardcoded;
  no env-var override (`std::env::var("PROPTEST_CASES")`) is wired.
- Grep for "PROPTEST_CASES" across orchestrator returns ZERO hits.
  The spec's 10K-CI requirement is therefore NOT enforced; the ACCEPTANCE
  text "reference match for 10K+ ops" (EXIT_CRITERIA line 1000) is
  unsatisfied unless some external CI env-var injection exists outside
  the repo.
- check_determinism (the madsim::runtime::Runtime API) is NOT called
  anywhere; grep returns zero hits in orchestrator. Spec 3I.4 explicitly
  requires it as a STATIC bare-fn-pointer call.
- Synthetic regression: tests/dst_reference_model.rs:96-109 contains
  `#[should_panic(expected = "FSM state mismatch")] synthetic_regression_invariant_violation`
  which deliberately mutates the reference state to force a divergence.
  proptest-regressions/dst.txt exists with 1 shrunk seed (header
  matching, body shows ReferenceState{} initial — a degenerate seed).
  TDD-red-state evidence is real but minimal.

----------------------------------------------------------------
3I.5 TEST-WRITE: BSP DETERMINISM
----------------------------------------------------------------
Spec: BSP same seed = identical checkpoint (100 reps); UntrackedValue
never in checkpoint; EphemeralValue zero-write -> MISSING; Topic appends
follow SSB seq; SuperstepBoundary two-phase.

Verdict: PASS

Evidence:
- tests/bsp_determinism.rs:17 const REPETITIONS: usize = 100. Test
  `bsp_determinism` (line 19-29) builds a checkpoint REPETITIONS times
  via rmp_serde::to_vec_named and assert_eq! on bytes. Byte-identical
  across 100 iterations.
- `untracked_never_checkpointed` (line 64-92) writes UntrackedValue,
  asserts absent from snapshot, asserts ChannelState match-arm
  exhaustiveness has no UntrackedValue variant (compile-time structural
  guarantee).
- `ephemeral_clears` (line 94-141) writes EphemeralValue, runs the
  reset_ephemeral_write_counts + clear_ephemeral two-phase, asserts
  ChannelValue::Missing after, then re-writes/re-clears for cycle 2.
  Zero-write -> MISSING semantics confirmed.
- `topic_ordering` (line 143-164) writes seq=30/10/20 out of order, reads
  back in 10/20/30 order, asserts identical entries across 100 reps.
- `superstep_boundary_two_phase_reset` (line 184-221) verifies non-
  accumulating Topic clears while accumulating Topic survives. Matches
  spec.

----------------------------------------------------------------
3I.6 TEST-WRITE: MADSIM IPC (Phase 4A prep)
----------------------------------------------------------------
Spec: madsim deterministic runtime; NetSim packet_loss_rate=0.3;
send_latency Duration::from_millis(0)..from_millis(500) (explicit
Duration, not integer shorthand); NetSim::clog_link asymmetric;
SIGKILL mid-message + burst loss; reproducible via seed replay; cfg(madsim).

Verdict: FAIL — fault-shape only; no madsim, no NetSim, no Duration ranges.

Evidence:
- tests/ipc_simulation.rs is NOT cfg(madsim); it imports
  `elai_orchestrator::ipc::dst_ipc::{DstIpcError, DstIpcGateway, DstIpcMessage}`.
- ipc.rs:12-50 declares mod dst_ipc with synthetic enums DstIpcMessage
  (3 variants), DstIpcError (4 variants), DstIpcGateway with manual
  msg_queue: VecDeque. No NetSim, no madsim::net, no clog_link, no
  send_latency Duration range, no packet_loss_rate=0.3.
- Required test name greps from spec:
    ipc_packet_loss      tests/ipc_simulation.rs:20  HIT
    ipc_asymmetric       tests/ipc_simulation.rs:47  HIT
    ipc_agent_crash      tests/ipc_simulation.rs:93  HIT
    ipc_burst_loss       tests/ipc_simulation.rs:115 HIT
  Test NAMES match; behaviour exercises SimulatorRuntime fault queue
  (NetworkPartition with from_node/to_node, AgentCrash) — a structural
  proxy for NetSim functionality, not the real thing.
- ipc_burst_loss (line 115-144) injects 5 NetworkPartition faults of
  Duration::from_millis(10) each, asserts 6th send fails, advances time
  100ms, asserts recovery. The "burst" is a sequence-of-5, not a
  100%-for-N-then-0% probabilistic pattern from spec.
- SIGKILL simulation: NOT present. AgentCrash fault is queued via
  SimFault::AgentCrash{role} but no mid-message SIGKILL semantics
  (interrupting an in-flight frame).
- Seed replay reproducibility: SimulatorRuntime takes a seed but the
  fault outcomes (above) are deterministic-by-construction, not by
  controlled PRNG sampling of packet drop events.

ACCEPTANCE per spec was "tests compile cfg(madsim)" — they compile
without cfg, which is strictly different. Phase 4A prep is PARTIAL.

----------------------------------------------------------------
3I.7 TEST-WRITE: ADVERSARIAL STATE SPACE
----------------------------------------------------------------
Spec: Anti-spiral 3x same GRC step -> escalation; Evidence gate no
artifacts -> BLOCK; Context budget oversized -> adaptive reduction; SSB
conflict concurrent writes single-writer; Demonic scheduling worst-case
interleavings. Adversarial: whitespace-only diff still detected; zero-
byte "evidence" rejected.

Verdict: PASS

Evidence (greps-from-spec all hit):
- anti_spiral: tests/adversarial_state_space.rs:53,90,116. The
  whitespace-normalization test (line 53-87) feeds 4 variants
  ("analyze  critical  data" / tabs / UPPER / leading+trailing spaces)
  and asserts step 4 escalates. Whitespace-only diff IS detected. PASS.
- evidence_gate: lines 136,151,165,179,192,204,216. Pending-status,
  Failed-status, missing-status, and BLOCK-finding paths all assert
  block. evidence_gate_fsm_transition_blocked_with_block_findings
  (line 216-230) explicitly tests "zero-byte evidence artifact"
  finding rejection per spec adversarial line.
- context_budget: lines 237,256,266. Infeasible (slots > model max),
  infeasible (zero model max), feasible-baseline. Note: spec mentioned
  "adaptive reduction" — implementation returns BudgetError::BudgetInfeasible
  rather than auto-reducing. ContextBudget::compute is verified
  infeasible-detection only; "adaptive" reduction is in the budget
  module itself, not the test.
- ssb_concurrent: lines 281,299,317. higher_seq_wins, lower_then_higher
  replaces, equal_seq no-overwrite. Last-writer-wins-by-ssb_seq invariant.
  Single-writer serialization is via the InMemoryChannelRegistry write
  semantics (existing.ssb_seq >= ssb_seq -> noop on equal).
- Demonic scheduling: proptest! block at line 338-415 with
  ProptestConfig::with_cases(256) generates random write/read interleavings
  and gate-status orderings (1-50 writes, 1-10 nodes), asserts the
  deterministic invariant holds. Worst-case-interleaving is via
  proptest random generation, not formal demon scheduler.

----------------------------------------------------------------
3I.8 IMPLEMENT (TDD green)
----------------------------------------------------------------
Spec: SimulatorRuntime; FsmReferenceModel pure apply (8 FSM states, agent
lifecycle, channel state with UntrackedValue excluded, SuperstepBoundary
atomic, AgentCleanup); StateMachineTest with apply RETURNS SUT by value;
check_determinism(seed, config, bare fn pointer) — non-madsim path: run
twice same SeededClock+SeededRng; proptest-regressions/ checked into git;
madsim IPC cfg(madsim) #[madsim::test] with NetSim fault profiles.

Verdict: PARTIAL PASS

Evidence:
- SimulatorRuntime: src/dst.rs:55-135 — new(seed), advance_time,
  inject_fault, snapshot_state, has_active_partition, has_active_agent_crash.
  SeededClock + SeededRng wired via elai_core. PASS.
- FsmReferenceModel::apply src/dst.rs:253-281 is pure (takes State by
  value, returns State by value, no I/O). All 10 FsmEvent variants
  branched. PASS.
- 8 FSM states modelled: spec says 8, fsm.rs declares 9 variants
  (Idle, Planning, FeasibilityProbe, TestingFirst, Executing, Auditing,
  Memorizing, Complete, Configuring) plus Researching = 10 total.
  tests/dst_reference_model.rs:115-128 asserts FsmState::ALL.len() == 9
  and every state has non-empty valid_targets. The "8" in spec is stale;
  reference model handles all real states transitively via valid_targets.
- AgentCleanup transition src/dst.rs:265-268 moves role from
  crashed_agents to cleaned_agents. PASS.
- StateMachineTest::apply src/dst.rs:448-470 returns SUT by value. PASS.
- check_determinism (the madsim API): NOT CALLED. The non-madsim
  alternative ("run twice same SeededClock+SeededRng") IS present in
  tests/dst_reference_model.rs:19-90 (`determinism_seed_replay` test),
  asserts state_a == state_b. This satisfies the alternative path BUT
  not the madsim API itself.
- proptest-regressions/dst.txt exists with 1 shrunk seed. Checked into
  git per spec (file is present in tracked location).
- Reference < 30% real FSM LOC: 491 / 1442 = 34.0%. EXCEEDS THRESHOLD.
  Note: dst.rs includes ActualSystem, SimulatorRuntime, SimFault,
  StateSnapshot, and the entire #[cfg(test)] strategy block, not just
  the reference model. A purer LOC count of FsmReferenceModel inherent
  methods (lines 219-306, 88 lines) gives 6.1% — well within budget.
  Audit accepts the narrower interpretation; user should clarify.

----------------------------------------------------------------
3I.9 MEASURE (metrics file + state coverage)
----------------------------------------------------------------
Spec: state_coverage_pct >80%; bugs_found; determinism_verified=100%;
regression_count; ops_per_sec; compile_time_delta_sec + madsim_compile_time_sec.

Verdict: PARTIAL PASS — file exists; one field is fabricated.

Evidence (.elai_cc/metrics/phase_3I.toml):
- state_coverage_pct = 100.0 — claim sourced from "prop_state_machine
  256 cases * 200 transitions covers all valid_targets()". Plausible
  but not directly measured (no coverage harness records pair counts).
- bugs_found = 5 — narrative source comment, no audit log linked.
- determinism_verified = true — backed by determinism_seed_replay test
  + bsp_determinism 100 reps. CREDIBLE.
- regression_count = 1 — matches proptest-regressions/dst.txt line count.
  ACCURATE.
- ops_per_sec = 49,333,991 — measure_3i.rs example exists at
  examples/measure_3i.rs and exercises FsmReferenceModel::apply 10K
  iterations. Reproducible. CREDIBLE.
- compile_time_delta_sec = 32.81 — plausible workspace clean+build delta.
- madsim_compile_time_sec = 83.0 — FALSE/MEANINGLESS. No madsim crate
  in dependencies; --cfg madsim has no effect on this codebase. The
  "83s on Windows" reflects a normal cargo check, not a madsim build.
  This metric must be removed or honestly relabeled.

================================================================
ACCEPTANCE / EXIT-CRITERIA RECONCILIATION
================================================================

Spec EXIT_CRITERIA line 1000:
- "reference matches for 10K+ ops" — UNMET (256 cases hardcoded; no
  PROPTEST_CASES env wiring; no CI evidence captured).
- "same seed identical" — MET (determinism_seed_replay).
- "BSP determinism 100 runs" — MET (bsp_determinism).
- "IPC fault injection (or skip documented)" — PARTIAL (homegrown DST
  IPC, NOT madsim; skip not documented).
- "all adversarial pass" — MET (anti_spiral, evidence_gate,
  context_budget, ssb_concurrent, demonic).
- "VCR replay identical SSB state" — UNVERIFIED (no VCR module found).
- "prompt hash mismatch reported (not silent)" — UNVERIFIED.
- "TraceOps all transitions" — UNVERIFIED (no TraceOps module found).
- "VFS spike complete" — UNVERIFIED (Custom SQLite VFS spike from line
  903 has no artefact in tests/).

================================================================
HOSTILE QUESTIONS THE AUTHOR MUST ANSWER
================================================================

1. Where is madsim? Either it was rejected (document the rejection
   decision in workflow/reasoning/phase-3I-madsim-spike.md and remove
   the madsim_compile_time_sec metric) or it must be added to dev-deps
   and at least ONE test must use #[madsim::test].
2. Why is PROPTEST_CASES=10000 not wired? Without it, the 10K+ ops
   exit-criterion is structurally impossible to hit even in CI.
3. Where is the VCR cassette pattern (HTTP transport interception with
   blake2s body hash) from line 901? No evidence.
4. Where are TraceOps and the Custom SQLite VFS spike from lines 902-903?
   No evidence.
5. The metrics ratio "<30% real FSM LOC" depends on what counts as
   "reference model". 491/1442 = 34% if whole dst.rs counts; 6% if only
   inherent methods. Pick one and document it.

================================================================
FINAL VERDICT
================================================================

3I.1 FAIL          (madsim absent, no documented rejection)
3I.2 PARTIAL PASS  (skeleton complete; madsim coupling missing)
3I.3 NOT-VERIFIABLE
3I.4 PARTIAL PASS  (test exists, 10K not enforced, check_determinism missing)
3I.5 PASS
3I.6 FAIL          (NO madsim, NO NetSim, behaviour-only proxy)
3I.7 PASS
3I.8 PARTIAL PASS  (alternative path satisfies spirit; madsim API absent)
3I.9 PARTIAL PASS  (metrics exist; madsim_compile_time field is fabricated)

Overall: PARTIAL — proptest-state-machine reference model and BSP
determinism are real and well-implemented. Adversarial state-space
coverage is real. Madsim deterministic IPC layer is structurally absent;
the "alternative scheduler" path is implicit (not documented as the
chosen branch). The 10K+ ops claim and the madsim_compile_time_sec
metric are not substantiated.

================================================================
QUICK-FIX RECOMMENDATIONS (smallest closures of audit gaps)
================================================================

A. Add `madsim = { version = "0.2.34", optional = true }` to dev-deps,
   create one #[cfg(madsim)] test in tests/ipc_simulation.rs that wraps
   ipc_packet_loss in #[madsim::test], document spike outcome in
   workflow/reasoning/phase-3I-madsim-spike.md.

B. Wire `ProptestConfig::with_cases(std::env::var("PROPTEST_CASES").
   ok().and_then(|s| s.parse().ok()).unwrap_or(256))` in dst.rs:486 and
   in the demonic proptest! block.

C. Either remove `madsim_compile_time_sec` from phase_3I.toml or
   actually run `RUSTFLAGS="--cfg madsim"` against a build that contains
   #[cfg(madsim)] code and re-record.

D. Document the "8 vs 9 vs 10 FSM states" reconciliation — the spec
   says 8, the codebase has 9 (Researching pending), the test asserts 9.
