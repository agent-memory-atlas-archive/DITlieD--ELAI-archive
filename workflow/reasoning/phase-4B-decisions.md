Phase 4B Decisions (s-659860, 2026-04-16)

What: 8 IPC tool systems + DKG query tools + 3 bundled RA tasks
  - git_tool (commit/undo/branch), validation, trace, edit_matching
  - file_read (adaptive shortening), compile_check (async incremental)
  - failure_taxonomy (DiagnosticResult/RootCause from RA-1)
  - dkg_tools (callers/dependents/data_flow/subgraph from RA-3)
  - IpcToolRegistry trait (dispatch hub)
  - Typed arg structs in ipc-types (14 request types)

Key decisions:
  1. IpcToolRegistry (NOT ToolRegistry) — avoids collision with tools/src/lib.rs:80
  2. tools/ restructure: new module files in tools/src/, lib.rs facade gets pub mod + pub use
  3. DKG tools query via existing DkgQuerier trait (rules/src/predicate.rs:386)
  4. No new workspace deps: rusqlite already in workspace, DkgQuerier in rules crate
  5. tools/Cargo.toml adds: elai-ipc-types, elai-rules (both workspace-local)
  6. ToolError uses thiserror (library crate rule)
  7. All async tool handlers use tokio::process::Command for subprocesses
  8. ClockProvider for all timestamps (entropy quarantine)
  9. postcard round-trip for all typed arg structs
  10. Phase-aware tool filtering: IpcToolRegistry::available_tools(FsmState) filters write tools

Expected test outcome: >=15 failing tests after 4B.3, all pass after 4B.4
Unknowns: ShortCoder paper interface (ingestor running), SVRepair CFG tier (optional, deferred if complex)
Deferred: 4B.6+4B.7 (IntegrationTestRunner depends on 4E types)
