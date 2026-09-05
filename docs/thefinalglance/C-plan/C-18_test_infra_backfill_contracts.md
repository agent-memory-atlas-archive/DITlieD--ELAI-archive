# C-18: Test Infra Backfill — Contract Tests Specified but Never Created
Objective: create all contract and integration test files that B-plan and phase-step
specs required but whose absence was confirmed by verification reports. Zero production
code changes. Only new test files under .elai_cc/crates/.
Estimated effort: 1 session.
Pre-requisites: C-1 (path unification stable), C-2 (SSB types stable, SCHEMA_VERSION
pinned), C-13 (IPC dispatcher swap stable — symbol_* and graph tool entries present).
Parallel-safe with: C-19, C-20 (different test categories; no shared file ownership).

## Context

Verification reports for B-2, B-29, B-32, B-33, B-35, B-36, phase-3B, phase-3D,
and phase-3H each list one or more contract/integration tests in the MISSING section.
These are not implementation gaps — the production symbols exist and are correct. The
tests that were specified at plan-authoring time simply never got written.

The effect: CI does not enforce the interface contracts at the crate boundary level,
so regressions in type shapes, dispatcher wiring, SSB payload serde, or DAG
determinism can go undetected until a late auditor run.

C-18 fills every named missing test in one session. It touches only test files;
the cargo workspace compiles each under the appropriate crate's dev-dependencies.

## Reads

- F:\.ELAI_workflow\docs\thefinalglance\verification\reports\B-plan\B-2_fsm_phasetransition_wiring.report.md
- F:\.ELAI_workflow\docs\thefinalglance\verification\reports\B-plan\B-29_symbol_level_impact_api.report.md
- F:\.ELAI_workflow\docs\thefinalglance\verification\reports\B-plan\B-32_minified_file_chunking.report.md
- F:\.ELAI_workflow\docs\thefinalglance\verification\reports\B-plan\B-33_interactive_graph_viewer.report.md
- F:\.ELAI_workflow\docs\thefinalglance\verification\reports\B-plan\B-35_context_artifacts_pipeline.report.md
- F:\.ELAI_workflow\docs\thefinalglance\verification\reports\B-plan\B-36_dashboard_v2_src_tauri_wiring.report.md
- F:\.ELAI_workflow\docs\thefinalglance\verification\reports\steps\phase-3B.report.md
- F:\.ELAI_workflow\docs\thefinalglance\verification\reports\steps\phase-3D.report.md
- F:\.ELAI_workflow\docs\thefinalglance\verification\reports\steps\phase-3H.report.md

## Tasks

1. **fsm_phase_transition_contract.rs** (B-2 MISSING)
   File: .elai_cc/crates/contracts/tests/fsm_phase_transition_contract.rs
   Construct Fsm with a real in-memory Ssb (tmp dir), call transition_with_metrics
   IDLE -> PLANNING, read latest_fsm_state() via SsbViewer. Assert returned state
   string equals "PLANNING". Assert PhaseTransitionPayload fields from_state="IDLE"
   and to_state="PLANNING" are present in the most-recent SSB row. Assert Fsm::reset()
   after transition does NOT emit a second PhaseTransition row (documents the known
   deliberate exclusion from report line 123).

2. **symbol_query_dispatch.rs** (B-29 MISSING)
   File: .elai_cc/crates/orchestrator/tests/symbol_query_dispatch.rs
   Build a minimal SymbolGraph fixture (3 nodes, 2 edges) using the knowledge crate's
   build_pipeline helpers. Construct a production_agent_rpc::translate_tool_request
   call for each of the four symbol_ tools (symbol_impact, symbol_flow, symbol_context,
   symbol_list) with a valid target. Assert each returns Ok (not Err). After each call
   assert an SsbActionType::SymbolQuery row is present in the Ssb. Run with
   ELAI_LIVE_SIDECAR absent (no network needed — graph is in-memory SQLite).

3. **chunking_minified.rs addition** (B-32 MISSING)
   File: .elai_cc/crates/knowledge/tests/chunking_minified.rs (append test function)
   Add test_minified_file_emits_ssb_action: construct an IndexingService with a
   real Ssb, call build_index_resumable on a fixture directory containing
   bundle.min.js (from tests/fixtures/minified_project/). Assert that after indexing
   at least one SsbActionType::MinifiedFileChunked row exists in the SSB with
   relative_path containing "bundle.min.js" and chunk_count > 0.

4. **graph_export_dispatch.rs** (B-33 MISSING)
   File: .elai_cc/crates/orchestrator/tests/graph_export_dispatch.rs
   Test the export_graph_mermaid Tauri command via direct function call (bypass Tauri
   runtime): invoke commands::export_graph_mermaid with a tmp project containing
   the mermaid_generation.rs fixture. Assert returned string contains "flowchart" or
   "graph" (Mermaid header). Assert the string length is > 10 (non-trivial output).
   Assert determinism: two consecutive calls return byte-identical strings.

5. **grounding_artifact_check.rs** (B-35 MISSING)
   File: .elai_cc/crates/reasoning/tests/grounding_artifact_check.rs
   Three test functions covering VerificationMethod::ArtifactCheck dispatch:
   - test_artifact_check_verifies: search returns results above threshold -> Ok(Verified)
   - test_artifact_check_malformed_grounding: artifact_name not in manifest -> MalformedGrounding
   - test_artifact_check_unverified: search returns empty -> Unverified
   Use ArtifactStore backed by an in-memory SQLite db; no filesystem required.

6. **designer_artifact_briefing.rs** (B-35 MISSING)
   File: .elai_cc/crates/orchestrator/tests/designer_artifact_briefing.rs
   Two test functions:
   - test_existing_project_injects_available_artifacts: call build_existing_project_context
     with a manifest containing two ArtifactSpec entries; assert returned briefing
     string contains "AVAILABLE_ARTIFACTS" and both artifact names.
   - test_greenfield_skips_artifact_block: call with empty manifest; assert briefing
     string does NOT contain "AVAILABLE_ARTIFACTS".

7. **dashboard_v2_D1_contract.rs through D5** (B-36 MISSING — 5 D-commands)
   Files: .elai_cc/crates/contracts/tests/dashboard_v2_D1_contract.rs
          .elai_cc/crates/contracts/tests/dashboard_v2_D2_contract.rs
          .elai_cc/crates/contracts/tests/dashboard_v2_D3_contract.rs
          .elai_cc/crates/contracts/tests/dashboard_v2_D6_contract.rs
          .elai_cc/crates/contracts/tests/dashboard_v2_D11_contract.rs
   Each file: one test that constructs the request type, serializes to JSON, deserializes,
   asserts round-trip identity. Also asserts the corresponding command function signature
   compiles (type-level contract). D1=get_sidecar_health, D2=run_planning_task,
   D3=run_research, D6=reindex_now, D11=sidecar_search. No live sidecar required;
   tests exercise type shapes only. ELAI_LIVE_SIDECAR-gated integration path noted
   in each file as a TODO comment for future CI expansion.

8. **ssb.rs test_payload_mismatch** (phase-3B MISSING)
   File: .elai_cc/crates/orchestrator/src/ssb.rs (append to ssb::tests module)
   Add test_payload_mismatch: write a CheckpointPayload to Ssb, then attempt to read
   it back and deserialize as a FileWritePayload. Assert the result is
   Err(SsbError::ActionTypeMismatch { .. }). Also add test_write_sequence_determinism:
   write 5 rows with a SeededClock (fixed epoch), read all back, assert seq values
   are 1..=5 monotonically and payload bytes are byte-identical on a second identical
   write run (same clock, new in-memory db).

9. **checkpoint_roundtrip and 5-superstep integration** (phase-3D MISSING)
   File: .elai_cc/crates/orchestrator/src/bsp/engine.rs (append to tests module)
   - test_checkpoint_roundtrip: build a BspCheckpoint with all three channel types
     (LastValue<String>, Topic<u64>, EphemeralValue<bool>), serialize via rmp_serde,
     deserialize, assert byte-identical to original serialization.
   - test_5_superstep_mixed_channels: run BspEngine for 5 supersteps with agents
     writing LastValue, Topic(accumulate=false), and EphemeralValue each superstep.
     Assert superstep counter == 5, all channel reads match expected last-writer-wins
     and topic-clear semantics, no untracked memory force-clear fires (channels small).

10. **session_dag determinism and exact boundary** (phase-3H MISSING)
    File: .elai_cc/crates/orchestrator/src/session_dag.rs (append to tests module)
    - test_100_run_determinism: loop 100 iterations each creating a SessionDag with
      the same SeededRng seed, saving a 10-message snapshot, calling trim_context,
      and recording the resulting snap_id + stub summary_hash values. Assert all 100
      iterations produce byte-identical snap_id strings and summary_hash values.
    - test_exact_boundary_85pct: create a TrimConfig with stub_threshold=100; inject
      context at exactly 84.9% of allocated_slots_tokens; assert create_handoff does
      NOT fire. Inject at exactly 85%; assert create_handoff fires and tagged snapshot
      exists with tag="handoff".

## WIRING_CONTRACT

produces_symbols:
  - fsm_phase_transition_contract (crates/contracts/tests/)
  - symbol_query_dispatch (crates/orchestrator/tests/)
  - chunking_minified.rs test_minified_file_emits_ssb_action (crates/knowledge/tests/)
  - graph_export_dispatch (crates/orchestrator/tests/)
  - grounding_artifact_check (crates/reasoning/tests/)
  - designer_artifact_briefing (crates/orchestrator/tests/)
  - dashboard_v2_D1/D2/D3/D6/D11 contract files (crates/contracts/tests/)
  - ssb.rs test_payload_mismatch + test_write_sequence_determinism
  - engine.rs test_checkpoint_roundtrip + test_5_superstep_mixed_channels
  - session_dag.rs test_100_run_determinism + test_exact_boundary_85pct
consumes_symbols:
  - Fsm::transition_with_metrics, SsbViewer::latest_fsm_state (orchestrator)
  - production_agent_rpc::translate_tool_request, symbol_* tools (orchestrator)
  - IndexingService::build_index_resumable, MinifiedFileChunkedPayload (orchestrator+knowledge)
  - commands::export_graph_mermaid, mermaid::generate (dashboard-v2+knowledge)
  - VerificationMethod::ArtifactCheck dispatch, ArtifactStore (reasoning+knowledge)
  - build_existing_project_context, ArtifactSpec (orchestrator)
  - Tauri command signatures D1/D2/D3/D6/D11 (dashboard-v2)
  - SsbError::ActionTypeMismatch, SeededClock (orchestrator)
  - BspCheckpoint, BspEngine::run (orchestrator bsp)
  - SessionDag::save_snapshot, trim_context, create_handoff (orchestrator)
reachable_from:
  - cargo test -p elai-contracts
  - cargo test -p elai-orchestrator --test '*'
  - cargo test -p elai-knowledge --test '*'
  - cargo test -p elai-reasoning --test '*'
stub_ban:
  - no mock data in test assertions — all symbol graph, SSB, and DAG fixtures must
    use real in-memory SQLite (tmp dir or :memory:), not hard-coded return values
  - no .unwrap() in test setup that would silently swallow construction errors
observable_effect:
  trigger: cargo test -p elai-contracts && cargo test -p elai-orchestrator --test '*'
  expect: all 10+ new test files compile and pass; zero pre-existing tests regress
  trigger2: elai /task --mode cloud "modify symbol X in foo.rs and confirm blast radius"
  expect_ssb: SsbActionType::SymbolQuery row present; symbol_query_dispatch contract
              re-runs green on next cargo test invocation

## DETERMINISTIC_CHECK

```bash
# 1. Count new contract test files (must be >= 10)
find .elai_cc/crates -name "*_contract.rs" -newer .elai_cc/crates/contracts/tests/channel_registry_contract.rs | wc -l

# 2. Contracts crate compiles and all tests pass
cargo test -p elai-contracts 2>&1 | tail -3

# 3. Orchestrator integration tests pass
cargo test -p elai-orchestrator --test '*' 2>&1 | tail -3

# 4. Knowledge tests pass (includes chunking_minified addition)
cargo test -p elai-knowledge --test chunking_minified 2>&1 | tail -3

# 5. Reasoning tests pass
cargo test -p elai-reasoning --test grounding_artifact_check 2>&1 | tail -3

# 6. 100-run determinism test exists and passes
cargo test -p elai-orchestrator test_100_run_determinism -- --nocapture 2>&1 | grep "ok\|FAILED"

# 7. Checkpoint roundtrip test exists and passes
cargo test -p elai-orchestrator test_checkpoint_roundtrip -- --nocapture 2>&1 | grep "ok\|FAILED"

# 8. Payload mismatch test exists and passes
cargo test -p elai-orchestrator test_payload_mismatch -- --nocapture 2>&1 | grep "ok\|FAILED"

# 9. No new .unwrap() in test setup code
rg "\.unwrap\(\)" .elai_cc/crates/contracts/tests/fsm_phase_transition_contract.rs 2>/dev/null | wc -l
# Expected: 0

# 10. Workspace check clean after additions
cargo check --workspace 2>&1 | grep "^error" | wc -l
# Expected: 0
```

## ACCEPTANCE

- [ ] All 10+ contract/integration test files created under .elai_cc/crates/
- [ ] cargo test -p elai-contracts: all tests pass including 5 new dashboard_v2_D*
- [ ] cargo test -p elai-orchestrator --test '*': symbol_query_dispatch, graph_export_dispatch,
      designer_artifact_briefing all pass
- [ ] cargo test -p elai-knowledge --test chunking_minified: test_minified_file_emits_ssb_action passes
- [ ] cargo test -p elai-reasoning --test grounding_artifact_check: all 3 test functions pass
- [ ] ssb.rs test_payload_mismatch returns Err(ActionTypeMismatch) — not panic, not Ok
- [ ] ssb.rs test_write_sequence_determinism: seq 1..=5, byte-identical on repeat run
- [ ] engine.rs test_checkpoint_roundtrip: byte-identical serialize/deserialize round-trip
- [ ] engine.rs test_5_superstep_mixed_channels: superstep_counter == 5, channel semantics correct
- [ ] session_dag.rs test_100_run_determinism: all 100 iterations produce identical snap_id + summary_hash
- [ ] session_dag.rs test_exact_boundary_85pct: handoff fires at 85%, absent at 84.9%
- [ ] Cloud Sonnet smoke: elai /task --mode cloud "modify symbol X in foo.rs and confirm blast radius"
      completes; SSB contains SymbolQuery row; symbol_query_dispatch contract re-runs green
- [ ] find ... | wc -l returns >= 10 for new contract files newer than channel_registry_contract.rs
- [ ] cargo check --workspace exits 0 after all additions
- [ ] No pre-existing tests regressed (auditor runs full workspace suite to confirm)

## Files touched

- ADD: .elai_cc/crates/contracts/tests/fsm_phase_transition_contract.rs
- ADD: .elai_cc/crates/orchestrator/tests/symbol_query_dispatch.rs
- MODIFY: .elai_cc/crates/knowledge/tests/chunking_minified.rs (append test function)
- ADD: .elai_cc/crates/orchestrator/tests/graph_export_dispatch.rs
- ADD: .elai_cc/crates/reasoning/tests/grounding_artifact_check.rs
- ADD: .elai_cc/crates/orchestrator/tests/designer_artifact_briefing.rs
- ADD: .elai_cc/crates/contracts/tests/dashboard_v2_D1_contract.rs
- ADD: .elai_cc/crates/contracts/tests/dashboard_v2_D2_contract.rs
- ADD: .elai_cc/crates/contracts/tests/dashboard_v2_D3_contract.rs
- ADD: .elai_cc/crates/contracts/tests/dashboard_v2_D6_contract.rs
- ADD: .elai_cc/crates/contracts/tests/dashboard_v2_D11_contract.rs
- MODIFY: .elai_cc/crates/orchestrator/src/ssb.rs (append 2 test fns to ssb::tests)
- MODIFY: .elai_cc/crates/orchestrator/src/bsp/engine.rs (append 2 test fns to tests mod)
- MODIFY: .elai_cc/crates/orchestrator/src/session_dag.rs (append 2 test fns to tests mod)
- CREATE: workflow/reasoning/C-18-decisions.md

## Next

C-19 or C-20 (other test categories); or B-43 if new implementation work is prioritized.
