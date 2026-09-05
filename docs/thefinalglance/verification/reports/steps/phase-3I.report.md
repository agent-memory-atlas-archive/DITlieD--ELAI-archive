# Verification Report: phase-3I
Source: F:\.ELAI_workflow\docs\specs\guide\steps\phase-3I.txt
Date: 2026-05-17
Overall status: MOSTLY_DONE
Confidence: HIGH

## Summary
Phase 3I DST infrastructure is substantially complete. The core skeleton (SimulatorRuntime, FsmReferenceModel, FsmEvent with all 10 variants, ReferenceStateMachine + StateMachineTest impls, proptest-state-machine PBT, BSP determinism tests, IPC fault simulation, adversarial state space tests, and metrics file) are all present and implemented. The principal gaps are: (1) madsim is absent entirely — replaced by a custom DstIpc stub layer without `#[cfg(madsim)]` conditional compilation, and no madsim spike test or Windows-documented alternative exists; (2) the VCR cassette pattern (HTTP intercept, blake2s hash, msgpack cassette file), TraceOps trajectory layer, and SQLite custom VFS are not implemented; (3) `sim.with_seed()` builder method is missing (only `::new(seed)` exists); (4) `check_determinism` (madsim static function pattern) is absent from tests — replaced by manual seed replay.

## Deliverables verified DONE

- SimulatorRuntime struct with SeededClock, SeededRng, fault_queue, advance_time, inject_fault, snapshot_state — evidence: crates/orchestrator/src/dst.rs:45-135
- FsmEvent enum with all 10 variants (PhaseTransition, AgentSpawn, AgentCrash, AgentCleanup, ToolCall, FileWrite, GrcStep, SsbWrite, ConfigChange, SuperstepBoundary) — evidence: crates/orchestrator/src/dst.rs:142-153
- FsmReferenceModel struct implementing ReferenceStateMachine trait (init_state, transitions, preconditions, apply) — evidence: crates/orchestrator/src/dst.rs:191, 318-435
- ReferenceState type (FSM state, SSB row counts, agent lifecycle sets, channel maps, ephemeral set) — evidence: crates/orchestrator/src/dst.rs:159-184
- StateMachineTest impl for ActualSystem (init_test, apply, check_invariants) — evidence: crates/orchestrator/src/dst.rs:438-483
- FsmReferenceModel::check_invariants (FSM state match, SSB row counts match, active agent set match) — evidence: crates/orchestrator/src/dst.rs:284-305
- SuperstepBoundary event variant with two-phase ephemeral clear in apply() — evidence: crates/orchestrator/src/dst.rs:152, 276-279
- AgentCleanup variant (crashed->cleaned lifecycle) — evidence: crates/orchestrator/src/dst.rs:146, 265-267
- prop_state_machine! macro invocation (sequential 1..200 => ActualSystem) — evidence: crates/orchestrator/src/dst.rs:485-490
- proptest + proptest-state-machine 0.8 in dev-dependencies — evidence: crates/orchestrator/Cargo.toml:110-111
- Determinism seed-replay test (two runs same seed, assert identical final state) — evidence: crates/orchestrator/tests/dst_reference_model.rs:20-90
- Synthetic regression invariant violation test (#[should_panic]) — evidence: crates/orchestrator/tests/dst_reference_model.rs:96-109
- proptest-regressions/dst.txt exists with one shrunk case — evidence: crates/orchestrator/proptest-regressions/dst.txt:7
- BSP determinism test (100 repetitions identical checkpoint bytes) — evidence: crates/orchestrator/tests/bsp_determinism.rs:20-62
- untracked_never_checkpointed test — evidence: crates/orchestrator/tests/bsp_determinism.rs:65-92
- ephemeral_clears test (two full clear cycles) — evidence: crates/orchestrator/tests/bsp_determinism.rs:94-141
- topic_ordering test (SSB seq-sorted entries, 100 reps) — evidence: crates/orchestrator/tests/bsp_determinism.rs:143-163
- superstep_boundary_two_phase_reset test (snapshot, clear non-accum, execute, clear ephemeral) — evidence: crates/orchestrator/tests/bsp_determinism.rs:184+
- IPC fault simulation tests (ipc_packet_loss, ipc_asymmetric, ipc_agent_crash, ipc_burst_loss) — evidence: crates/orchestrator/tests/ipc_simulation.rs:20-144
- DstIpcGateway/DstIpcMessage/DstIpcError types (DST stub IPC layer) — evidence: crates/orchestrator/src/ipc.rs:12-end
- Adversarial: anti_spiral tests (whitespace normalization, threshold boundary, empty string) — evidence: crates/orchestrator/tests/adversarial_state_space.rs:52-132
- Adversarial: evidence_gate tests (blocks Pending/Failed/Missing, passes Complete/TimedOut, FSM transition blocked) — evidence: crates/orchestrator/tests/adversarial_state_space.rs:138-233
- Adversarial: context_budget tests (BudgetInfeasible cases) — evidence: crates/orchestrator/tests/adversarial_state_space.rs:239-277
- Adversarial: ssb_concurrent write tests (higher seq wins, equal seq no-overwrite) — evidence: crates/orchestrator/tests/adversarial_state_space.rs:283-335
- Adversarial: demonic_ssb_last_writer_wins proptest (256 cases) — evidence: crates/orchestrator/tests/adversarial_state_space.rs:341+
- Metrics file phase_3I.toml with all required fields (state_coverage_pct, bugs_found, determinism_verified, regression_count, ops_per_sec, compile_time_delta_sec, madsim_compile_time_sec) — evidence: .elai_cc/metrics/phase_3I.toml:1-35
- measure_3i example binary for ops/sec measurement — evidence: crates/orchestrator/examples/measure_3i.rs (SCIP refs in elai.map)

## Deliverables PARTIAL

- madsim IPC simulation (step 3I.6)
  - exists: crates/orchestrator/tests/ipc_simulation.rs — all 4 named test functions present (ipc_packet_loss, ipc_asymmetric, ipc_agent_crash, ipc_burst_loss), using DstIpcGateway stub
  - gap: madsim dependency is absent from Cargo.toml; no `#[cfg(madsim)]` gates; no `#[madsim::test]` attributes; simulation uses a custom stub (DstIpcGateway) instead of actual madsim net::Config (packet_loss_rate, send_latency Range<Duration>, clog_link). The spec requires real madsim or a documented alternative with equivalent fault injection — neither is formally documented.

- SimulatorRuntime::with_seed() builder (step 3I.8)
  - exists: `SimulatorRuntime::new(seed: u64)` at crates/orchestrator/src/dst.rs:56
  - gap: step 3I.8 specifies `sim.with_seed(seed)` as a named builder method. Only `::new(seed)` is present; functionally equivalent but does not match the spec signature.

## Deliverables MISSING

- VCR (Video Cassette Recorder) pattern — searched: VCR, cassette, vcr_cassette, CassettePlayer, blake2s, msgpack in elai.map and orchestrator/src; no evidence
- TraceOps layer (agent execution trajectory recording) — searched: TraceOps, trace_ops in elai.map and orchestrator; no evidence
- SQLite custom VFS spike (sqlite3_vfs struct, torn write simulation, WAL recovery) — searched: sqlite.*vfs, SqliteVfs, custom_vfs in elai.map; no evidence. The spike was required to evaluate whether madsim hooks cover write/fsync before implementing.
- madsim spike test (step 3I.1: madsim v0.2.34 dev-dep, minimal node-to-node test, RUSTFLAGS="--cfg madsim" cargo test madsim_spike) — searched: madsim in all Cargo.toml files, madsim_spike in test files; no evidence. The alternative chosen (DstIpcGateway stub) is not formally documented as the madsim replacement per spec requirement.
- madsim dev-dependency in Cargo.toml (step 3I.2 requires madsim v0.2.34 with cfg(madsim) gate) — searched: Cargo.toml for "madsim"; not present
- check_determinism static function call (step 3I.4/3I.8: madsim::runtime::Runtime::check_determinism(seed, config, bare_fn_pointer)) — searched: check_determinism in elai.map and orchestrator; no evidence. Manual seed replay used instead in dst_reference_model.rs.
- PROPTEST_CASES=10000 CI configuration or #[timeout(600_000)] annotation — searched: timeout attribute, PROPTEST_CASES in orchestrator; not found. Current config is ProptestConfig::with_cases(256) hardcoded.

## Deliverables UNVERIFIABLE

- "All Phase 0-3H tests still pass after 3I additions" (step 3I.3/3I.8 exit criterion) — reason: test execution not performed; no auditor test run output available to verify zero regressions
- "Reference model LOC < 30% of real FSM LOC" (step 3I.8 acceptance) — reason: would require counting LOC of FsmReferenceModel methods vs real FSM; possible but not executed as part of static verification
- "State space coverage > 80% of valid transition pairs" (step 3I.9) — reason: metric is recorded as 100% in phase_3I.toml but was not independently measured here; taken from metrics file which is flagged as unverifiable per report instructions

## What must be done (action list)

- [ ] Add madsim v0.2.34 dev-dependency with `[cfg(madsim)]` feature gate to crates/orchestrator/Cargo.toml, OR document in a grounding log that DstIpcGateway stub was formally chosen as the madsim alternative with justification
- [ ] Write madsim spike test (step 3I.1) — minimal two-node message send — OR document Windows incompatibility with specific failure evidence
- [ ] Add `sim.with_seed(seed)` builder method to SimulatorRuntime (crates/orchestrator/src/dst.rs) or update spec reference
- [ ] Implement VCR cassette layer (HTTP transport intercept, blake2s request hash, msgpack storage) per step 3I infrastructure requirements, or document as out-of-scope deferral
- [ ] Implement TraceOps trajectory recording (GRC steps, tool calls, branching decisions) or document as deferred
- [ ] Complete SQLite custom VFS spike (evaluate madsim libc hooks vs sqlite3_vfs implementation) — currently no evidence of evaluation
- [ ] Add `#[timeout(600_000)]` or equivalent to fsm_reference_model_matches test, and document PROPTEST_CASES=10000 CI env var configuration
- [ ] Move prop_state_machine! test to integration test scope (currently in unit test module inside dst.rs) if external access is needed, OR confirm current placement is intentional

## Notes
The IPC simulation tests compile and run against a DstIpcGateway stub that simulates fault injection via SimulatorRuntime fault_queue rather than real madsim network simulation. This is functionally adequate for deterministic testing but does not cover madsim-specific behaviors (MoE routing anomalies, real OS network stack simulation). The phase_3I.toml records madsim_compile_time_sec=83.0 suggesting madsim was evaluated at some point, but no madsim code is present in the current codebase.
