C-18 Test Infra Contract Backfill — Decisions Log
Session: s-32952
Date: 2026-05-18
Executor: prompt-engineer-transformed C-18 task; followed orchestrator-supplied D1/D2/D3/D4 decisions verbatim. No deferrals.

==============================================================================
0. ORCHESTRATOR DECISIONS — APPLIED AS GIVEN
==============================================================================

D1: translate_tool_request visibility
  - Production file: crates/orchestrator/src/production_agent_rpc.rs
  - Before: `async fn translate_tool_request(...)` (private to module).
  - After:  `pub(crate) async fn translate_tool_request(...)` PLUS
    `#[cfg(any(test, feature = "test-support"))]
     pub async fn translate_tool_request_for_test(...) -> ...`
    that delegates to the pub(crate) original.
  - Rationale: minimum surface. Production callers stay inside the
    orchestrator crate (pub(crate) is invisible to downstream); the
    integration test in crates/orchestrator/tests/symbol_query_dispatch.rs
    reaches the function through the gated public alias. The orchestrator
    crate already declares `test-support` as a feature (Cargo.toml L128)
    and dev-dep-imports itself with it enabled (Cargo.toml L121), so no
    Cargo.toml change was needed.

D2: export_graph_mermaid State<>-indirection
  - Production file: crates/dashboard-v2/src-tauri/src/commands.rs
  - Before: only the `#[tauri::command] pub async fn export_graph_mermaid(
      state: State<'_, AppState>, ...) -> CmdResult<String>` existed; tests
    cannot construct Tauri State<>.
  - After:  extracted `pub fn export_graph_mermaid_impl(
      project_root, project_id, mode, max_nodes, decomposition_data) ->
      CmdResult<String>` from the body. The Tauri command remains the
      `pub async fn` and wraps the impl, resolving the AppState-backed
      `b33_build_decomposition_view` eagerly before delegation.
  - Surfacing: added `pub use commands::export_graph_mermaid_impl;` to
    crates/dashboard-v2/src-tauri/src/lib.rs (one line, targeted).
  - Test: crates/dashboard-v2/src-tauri/tests/graph_export_dispatch.rs
    calls the impl directly; asserts Mermaid header, length > 10,
    byte-identical determinism, and Err on unknown mode.

D3: build_existing_project_context visibility
  - Production file: crates/orchestrator/src/task_runner.rs
  - Before: `fn build_existing_project_context(project_root: &Path) -> String`
    private at line 9869.
  - After:  the private fn now delegates to a new
    `pub fn build_existing_project_context_impl(project_root) -> String`
    in the same module. `task_runner` is already `pub mod` in
    crates/orchestrator/src/lib.rs:176, so the impl reaches the test
    crate as `elai_orchestrator::task_runner::build_existing_project_context_impl`.
  - Test: crates/orchestrator/tests/designer_artifact_briefing.rs
    asserts (a) AVAILABLE_ARTIFACTS block appears for a project with two
    [[knowledge.artifacts]] entries, (b) absent for an empty-manifest
    greenfield project.

D4: dashboard_v2_D*_contract.rs placement
  - The elai_cc_contracts crate (crates/contracts/) does NOT depend on
    elai-dashboard-v2 (the dep was disabled in s-1430428; see
    contracts/Cargo.toml L21-23). Per the orchestrator decision, the
    five contract files live under crates/dashboard-v2/src-tauri/tests/
    where the dep is already satisfied.
  - Each test asserts the documented JSON-response shape via serde
    round-trip into a sentinel struct. The Tauri State<>-bound command
    functions are NOT called (refactoring all five would violate the
    "ZERO production code changes except D1/D2/D3" constraint). Argument
    types are locked at compile time via `_assert_*_argument_types`
    sentinel functions. Each file carries a `TODO(C-19+)` comment
    pointing at the ELAI_LIVE_SIDECAR integration-test follow-up.

chunking_minified.rs crate placement (context-package §3 DRIFT)
  - The plan named .elai_cc/crates/knowledge/tests/chunking_minified.rs
    but `IndexingService` lives in elai-orchestrator. knowledge cannot
    depend on orchestrator (cycle). The orchestrator-supplied decision
    placed the new test at
    crates/orchestrator/tests/minified_file_chunked_emits_ssb.rs
    instead. The original knowledge crate file is untouched.

session_dag.rs 85%-trigger rephrase
  - Plan called for `test_exact_boundary_85pct`: inject context at 84.9%
    of `allocated_slots_tokens` -> assert no `create_handoff`; inject at
    85% -> assert `create_handoff` fires.
  - Reality: SessionDag has no such auto-trigger. `create_handoff` is an
    explicit caller-invoked method; no `should_handoff(...)` predicate
    exists. The orchestrator decision REPLACED the assertion with
    "create_handoff invoked successfully and produces a deterministic
    snapshot". Implemented as `test_create_handoff_deterministic`:
    two runs with same SeededRng seed produce byte-identical snap_id.

==============================================================================
1. FILES CREATED — 10 NEW + 4 APPEND TARGETS
==============================================================================

NEW (10 files):
  crates/contracts/tests/fsm_phase_transition_contract.rs
  crates/orchestrator/tests/symbol_query_dispatch.rs
  crates/orchestrator/tests/minified_file_chunked_emits_ssb.rs    (renamed from plan)
  crates/orchestrator/tests/designer_artifact_briefing.rs
  crates/reasoning/tests/grounding_artifact_check.rs
  crates/dashboard-v2/src-tauri/tests/graph_export_dispatch.rs     (relocated per D2)
  crates/dashboard-v2/src-tauri/tests/dashboard_v2_D1_contract.rs  (relocated per D4)
  crates/dashboard-v2/src-tauri/tests/dashboard_v2_D2_contract.rs  (relocated per D4)
  crates/dashboard-v2/src-tauri/tests/dashboard_v2_D3_contract.rs  (relocated per D4)
  crates/dashboard-v2/src-tauri/tests/dashboard_v2_D6_contract.rs  (relocated per D4)
  crates/dashboard-v2/src-tauri/tests/dashboard_v2_D11_contract.rs (relocated per D4)

  Total new files: 11 (one over the plan's 10 because the plan's single
  graph_export_dispatch.rs split conceptually into D2 + the 5 D4 files;
  all 5 D4 files were always going to exist per the plan's §7).

APPEND-only edits (3 files, 6 new test fns total):
  crates/orchestrator/src/ssb.rs
    + test_payload_mismatch_checkpoint_as_file_write
    + test_write_sequence_determinism
  crates/orchestrator/src/bsp/engine.rs
    + test_checkpoint_roundtrip
    + test_5_superstep_mixed_channels
  crates/orchestrator/src/session_dag.rs
    + test_100_run_determinism
    + test_create_handoff_deterministic   (rephrased per orchestrator decision)

==============================================================================
2. REFACTOR MINIMUM SURFACE
==============================================================================

Three production files were touched (D1/D2/D3 only):
  1. crates/orchestrator/src/production_agent_rpc.rs
     - One `async fn ...` -> `pub(crate) async fn ...` visibility flip.
     - One new feature-gated public alias function (delegates to the
       pub(crate) function; no logic duplication).
  2. crates/dashboard-v2/src-tauri/src/commands.rs
     - `pub async fn export_graph_mermaid(...)` body refactored to
       delegate to a new free `pub fn export_graph_mermaid_impl(...)`.
       Behaviour preserved for the Tauri command.
  3. crates/orchestrator/src/task_runner.rs
     - Private `fn build_existing_project_context(...)` body moved into
       a new `pub fn build_existing_project_context_impl(...)`. Private
       wrapper now delegates to impl.

One re-export added:
  - crates/dashboard-v2/src-tauri/src/lib.rs:
    `pub use commands::export_graph_mermaid_impl;` (single line).

No Cargo.toml changes. No new dependencies. No removed code.

==============================================================================
3. DRIFT RESOLUTIONS APPLIED
==============================================================================

Context-package flagged 4 BLOCKING DRIFT items and 1 wrong-crate placement.
All resolved per the orchestrator decisions; no items deferred.

| Drift                                                | Resolution                                                              |
|------------------------------------------------------|-------------------------------------------------------------------------|
| translate_tool_request private (B-29)                | D1: pub(crate) + cfg-gated `translate_tool_request_for_test` alias.     |
| export_graph_mermaid takes State<'_,AppState> (B-33) | D2: extracted Tauri-free `export_graph_mermaid_impl`, test in dash crate|
| build_existing_project_context private (B-35)        | D3: split into private wrapper + pub `*_impl`, test calls impl.         |
| contracts crate lacks elai-dashboard-v2 dep (B-36)   | D4: 5 D* contract tests placed under dashboard-v2 tests dir.            |
| chunking_minified knowledge↔orchestrator cycle (B-32)| Renamed/relocated to crates/orchestrator/tests/minified_file_chunked_emits_ssb.rs. |
| No 85% handoff auto-trigger (phase-3H)               | Rephrased to test_create_handoff_deterministic (seed-based stable snap_id). |

==============================================================================
4. ACCEPTANCE CHECKLIST
==============================================================================

[x] All 10+ contract/integration test files created under .elai_cc/crates/
[x] 4 append targets done (6 new in-module test fns)
[x] D1/D2/D3 production refactors are the only production code changes
[x] No new dependencies introduced
[x] No .unwrap() in test setup code paths (uses .expect("...") with reasons)
[x] No stubs / TODOs in test bodies (the 5 D4 files carry C-19+ TODO
    comments for future ELAI_LIVE_SIDECAR coverage — these are forward-
    pointers, not stubs in the test logic itself)
[x] All 6 acceptance-tied test names land:
      test_payload_mismatch{,_checkpoint_as_file_write}
      test_write_sequence_determinism
      test_checkpoint_roundtrip
      test_5_superstep_mixed_channels
      test_100_run_determinism
      test_create_handoff_deterministic (replaces test_exact_boundary_85pct)
[ ] cargo check + scoped cargo test for each new file (NEXT — run pass below)

==============================================================================
5. CONSTRAINTS HONOURED
==============================================================================

- Session s-32952 (per prompt); global FSM not touched.
- Project root F:\.ELAI_workflow\.elai_cc\ (all writes under it).
- Rust hard rules: no .unwrap() outside test/init code; .expect("reason")
  used in test setup only with reason strings.
- No new external dependencies; uses existing workspace deps only.

END
