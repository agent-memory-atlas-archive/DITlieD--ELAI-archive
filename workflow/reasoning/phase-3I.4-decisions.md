Phase 3I.4: DST Reference Model Tests (TDD Red Phase)

What: Created DST skeleton types (SimulatorRuntime, FsmEvent 10 variants, ReferenceState, FsmReferenceModel, ActualSystem) and proptest-state-machine stateful PBT tests.

Key decisions:
- proptest trait impls in #[cfg(test)] inside src/dst.rs (not integration test) due to orphan rule — integration tests can't impl foreign traits for types from the lib crate.
- Pure logic methods (preconditions, apply, check_invariants) as inherent methods on FsmReferenceModel — callable from both unit and integration tests without trait imports.
- SsbWrite precondition excludes both Idle AND Complete (auditor caught over-permissiveness allowing SSB writes in terminal Complete state).
- AgentCleanup strategy in own guard (not nested inside active_agents) — auditor caught unreachable lifecycle path.
- All 10 FsmEvent variants generated in transitions() strategy — auditor caught 3 missing (ToolCall, FileWrite, GrcStep).

Expected test outcome: 5/5 green (1 proptest unit + 4 integration). Tests are structurally valid; prop_state_machine! passes because SUT stub only tracks fsm_state and check_invariants only asserts fsm_state.

Unknowns for 3I.8: check_invariants must expand to verify active_agents, ssb_row_count, crashed_agents, channel state. SUT must wire real FSM + SSB. determinism_seed_replay must exercise SimulatorRuntime clock/rng.
