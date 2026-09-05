C-24 Symbol Impact Dashboard Surface — Decisions Log
=====================================================

Session: s-25672
Plan: docs/thefinalglance/C-plan/C-24_symbol_impact_dashboard_surface.md
Context-package: workflow/sessions/s-25672/context-package-C24.md (16 DRIFT items)
Pre-req: C-17 SHIPPED (resilience + incremental knowledge-graph plumbing)
Pre-shipped overlap: C-18 export_graph_mermaid_impl + dashboard-v2/src-tauri/tests/graph_export_dispatch.rs

DECISION-1 — ToolDefinition schema name pinning (DRIFT-1).
  Plan used invented field names (target, prefix). Wire structs in ipc-types
  require the actual names (project_root, entrypoint_id, name/file_hint, file/
  query/limit). The patch-R3 doc-comment at production_agent_rpc.rs:206-208
  pins schema keys to ipc-types Request struct fields verbatim so
  convert_json_input_to_postcard can serde_json::from_value::<Request>()
  without rename overhead.
  Choice: option (i) — use wire-struct field names verbatim. The four schemas
  match SymbolImpactRequest/SymbolFlowRequest/SymbolContextRequest/
  SymbolListRequest one-for-one.
  Rationale: rejected option (ii) "inject project_root via system prompt
  middleware" — net-new infrastructure, not in C-24 scope. Rejected option
  (iii) "drop project_root from wire structs" — breaks all 4 dispatcher arms
  which use project_root to derive project_id and open SymbolGraphCache.

DECISION-2 — update_changed_files_symbol_graph caller signature (DRIFT-2).
  Plan said "call update_changed_files_symbol_graph(changed_paths); else
  call full build_symbol_graph". The fn already handles the threshold
  branch internally (build_pipeline.rs:255-262).
  Choice: branch on `is_resume` (already computed at indexing_service.rs:417),
  call the appropriate fn. is_resume=true -> incremental + UpdateSummary;
  is_resume=false -> full build_symbol_graph + BuildSummary (preserves
  cold-start behaviour).
  Rationale: the indexing_service's existing partitioning (work_queue +
  stale-prune set) already gives us the change list. Threading it through
  the incremental path on resume is the natural surface; on first cold
  index the entire project is "changed", so a full build is correct.

DECISION-3 — SymbolGraphUpdatedPayload.is_incremental as additive Option (DRIFT-4).
  Existing payload shape pinned by symbol_graph_updated_ssb_contract.rs.
  Adding is_incremental as #[serde(default, skip_serializing_if=
  "Option::is_none")] Option<bool> follows the C-17 DECISION-8 precedent for
  IndexCheckpointPayload.batch_duration_ms — rmp-serde named-map tolerates
  additive Option fields without SCHEMA_VERSION bump.
  Choice: additive Option<bool>. Default None on full builds (preserves
  existing test fixtures), Some(true) on incremental path.
  Rationale: rejected SCHEMA_VERSION bump because no consumer parses
  the SSB row strictly — the dashboard event listener takes whichever
  fields land.

DECISION-4 — wire-ID surface for export_graph_mermaid (DRIFT-5, DRIFT-6).
  No DispatchTool enum exists in dispatcher.rs; the file is pure routing.
  Real tool surfaces are (i) ToolDispatcher trait, (ii) translate_tool_request
  in production_agent_rpc.rs. Plan's "DispatchTool::ExportGraphMermaid arm"
  has no target.
  Choice: option (ii) — add TOOL_EXPORT_GRAPH_MERMAID wire ID +
  ExportGraphMermaidRequest struct to ipc-types; add a translate_tool_request
  arm that delegates to elai_knowledge::visualization::mermaid::generate_mermaid
  via a per-mode VizData build. Adds parallel `symbol_*` style arm.
  Rationale: minimal new infrastructure; reuses the same wire-ID + match-arm
  pattern as B-29 Phase 5 symbol_* tools. Heavier "extend ToolDispatcher
  trait" path would have propagated to 5+ impl sites.

DECISION-5 — /graph mermaid CLI delegation (DRIFT-6).
  No invoke_export_graph_mermaid symbol exists. Two paths:
  (a) call generate_mermaid directly from the CLI on a VizData built from
      the local indexes (no IPC), or
  (b) route via translate_tool_request (IPC path).
  Choice: (a). The CLI lives in the same process as the orchestrator
  inference loop; the indexes are file-rooted (project_root + dkg_db_path).
  Calling the mermaid generator directly is deterministic, zero-LLM, zero-IPC
  and matches the spec's "elai repl /graph mermaid SsbActionType" pattern.
  The wire-ID path (DECISION-4) still exists for IPC callers (agent tool
  self-selection or external REPL).

DECISION-6 — VizFile.level/status Path A (DRIFT-7).
  Add Option<String> level + Option<String> status fields to VizFile
  (knowledge crate). Producers (b33_build_decomposition_view) fill level;
  rebuildCytoscape (GraphWindow.svelte) publishes both to cytoscape data
  attributes. Decomposition selectors guard on graphMode === "decomposition".
  Status source: status fields stay None for now — the spawn_loop dispatcher
  / per-baby-step execution-status pipeline is deferred per B-42 memo
  (cosmos_worker scope, not C-24). The selectors EXIST in cytoscape style
  array so the moment status is sourced upstream, visual feedback ships
  automatically.
  Rationale: rejected Path B (label-prefix parsing in GraphWindow) — couples
  the cytoscape layer to the orchestrator DTO label format. Path A keeps
  the knowledge crate VizFile contract aligned with the producer DTO.

DECISION-7 — ReviewWindow re-target for "Show blast radius" (DRIFT-8).
  ActivityWindow is an OrbitCanvas host; it does not render file-write rows.
  FileChangeRow rows live in ReviewWindow.svelte (loaded via
  stub.get_file_changes). Plan task #7 retargeted to add a "blast radius"
  link/button in the ReviewWindow file-header action bar that calls
  graph.svelte.ts request_impact + set_mode('file') + workspace.openTab(
  'primary', 'graph').
  Rationale: aligns with the plan's intent (cross-link from a file-write
  surface to the GraphWindow) and the existing data model. ActivityWindow
  re-design would have been net-new scope.

DECISION-8 — workspace.openTab not openWindow; no GraphSidebarPanel (DRIFT-9).
  workspace.svelte.ts exports openTab(panelId, type) at L196, not
  openWindow(type). GraphSidebarPanel does not exist in this codebase.
  Use openTab('primary', 'graph') — matches the established
  KnowledgeDrawer.svelte:18 pattern.

DECISION-9 — extend existing C-18 graph_export_dispatch.rs (DRIFT-10).
  C-18 already shipped dashboard-v2/src-tauri/tests/graph_export_dispatch.rs
  with 3 tests (header, determinism, unknown-mode-Err). Adding the 4th
  ("empty-graph mermaid contains classDef header") and 3 new for the
  /graph mermaid CLI surface (file/symbol/decomposition modes through the
  translate_tool_request arm).
  Choice: extend existing file (option i), add 4 new tests for total of 7.
  Rationale: rejected option (ii) "create orchestrator-side file" — the
  orchestrator crate cannot depend on elai-dashboard-v2.

DECISION-10 — extract get_graph_view_data_impl + get_impact_overlay_impl (DRIFT-11).
  Apply the C-18 D2 pattern (State<>-free impl-fn extraction) to both
  Tauri commands. Lets the new graph_commands.rs test file call the
  impl-fn directly without a Tauri runtime.
  Extract surface: (project_root, project_id, mode) -> VizData and
  (project_root, project_id, target, depth) -> impact JSON.

DECISION-11 — convert_json_input_to_postcard 4 symbol_* arms (DRIFT-12).
  Per Group A2 in context-package. Adding the 4 arms in lockstep with
  the catalog so agent self-selection works end-to-end.

DECISION-12 — baby_step_dispatch_config tool_filter 10-name set (DRIFT-13).
  Extended task_runner.rs:1305-1343 to include TOOL_SYMBOL_IMPACT/FLOW/
  CONTEXT/LIST in the allowed_tools vec. Contract test
  baby_step_initial_agent_request_contract.rs updated to expect the 10-name
  set.

DECISION-13 — production_agent_rpc_tool_catalog_contract.rs inversion (DRIFT-15).
  Catalog len from 6 to 10; expected set extended to include the 4 symbol_*
  constants; "forbidden symbol_* assertion" loop trimmed to only the still-
  excluded surfaces (TOOL_ENTRY_POINTS, TOOL_LIST_ARTIFACTS,
  TOOL_SEARCH_ARTIFACTS, TOOL_REMOVE_ARTIFACTS). Note: TOOL_INDEX_ARTIFACTS
  stays in the forbidden set because its dispatcher arm is wired (C-8)
  but the doc-comment + catalog-exclusion contract still considers it
  Err-arm territory (its catalog inclusion can be flipped in a follow-up
  patch with the matching convert_json_input_to_postcard arm).

DECISION-14 — patch-R3 doc-comment refresh (DRIFT-14).
  The block at production_agent_rpc.rs:210-220 enumerating "Excluded
  entirely: Symbol-graph / artifact-search tools" updated to reflect
  that symbol_* are now included.

Test plan
---------
- New: dashboard-v2/src-tauri/tests/graph_commands.rs (4 smoke tests
  using the extracted *_impl functions on a tempdir).
- Extended: dashboard-v2/src-tauri/tests/graph_export_dispatch.rs
  (+1 test for empty-graph classDef header).
- Extended: contracts/tests/production_agent_rpc_tool_catalog_contract.rs
  (catalog len 10, set equality 10-name, forbidden set 4-name).
- Extended: contracts/tests/baby_step_initial_agent_request_contract.rs
  (tool_filter sorted set equality to 10-name list).
- Extended: contracts/tests/symbol_graph_updated_ssb_contract.rs
  (is_incremental field appears in the additive Option<bool> default-None
  state on existing fixtures).

Verification scripts (per plan DETERMINISTIC_CHECK)
---------------------------------------------------
- grep symbol_* ToolDefinition entries (4 hits in build_tool_catalog body).
- grep update_changed_files_symbol_graph + INCREMENTAL_SYMBOL_THRESHOLD
  in indexing_service.rs.
- grep SlashCommand::GraphMermaid + TOOL_EXPORT_GRAPH_MERMAID.
- grep Concept|BabyStep|status-running|status-passed in GraphWindow.svelte.
- grep request_impact|blast_radius in ReviewWindow.svelte.

NO STUBS — every new tool path delegates to real generator/cache code.
NO SCHEMA bump (rmp-serde additive Option<bool> precedent).
NO new dependencies.
