# C-24: Symbol Impact Dashboard Surface
Objective: close all MISSING and PARTIAL gaps from B-29 and B-33 that concern
symbol-level impact API agent exposure, incremental graph update wiring, CLI
graph export, dispatcher tool registration, cytoscape decomposition styling,
and ActivityWindow blast-radius cross-link.
Estimated effort: 2 sessions.
Pre-requisites: C-17 (knowledge indexing resilience: incremental update path
stable, LockScope::Watch watcher service present).
Parallel-safe with: C-21, C-22, C-23, C-25.

## Context

B-29 shipped symbol_impact, symbol_flow, symbol_context, and symbol_list
dispatcher arms in production_agent_rpc.rs:807-971, but none of the four
appear in build_tool_catalog (production_agent_rpc.rs:312). The catalog
currently returns 6 tools (edit, file_read, validate, compile_check,
classify, git_commit). Because agents receive no ToolDefinition for the
symbol_* tools, the LLM cannot self-select them during task planning — the
core value proposition of B-29 Phase 5 is therefore unreachable in practice.

B-29 also shipped update_changed_files_symbol_graph in
knowledge/src/symbol_graph/build_pipeline.rs:242, but the orchestrator
indexing_service.rs incremental-update path never calls it. The full-rebuild
path (build_symbol_graph) fires on every index cycle regardless of change
count, discarding the INCREMENTAL_SYMBOL_THRESHOLD=50 constant defined in
build_pipeline.rs:44-46.

B-33 shipped the Mermaid generator, all three Tauri commands, and GraphWindow
in full. Two wiring seams remain open: (1) no /graph mermaid CLI slash command
exists in crates/cli/src/commands.rs, and (2) export_graph_mermaid is only
reachable as a Tauri command — no dispatcher tool entry exists in
crates/orchestrator/src/dispatcher.rs, blocking REPL/IPC use. The cytoscape
style array in GraphWindow.svelte has language-based node colors but no
decomposition-level shape selectors or status-color classes (pending=gray,
running=blue pulse, passed=green, failed=red, blocked=orange). ActivityWindow
has no "Show blast radius" cross-link on file-write SSB rows.

Key source facts (from verification reports, HIGH confidence):
- build_tool_catalog at production_agent_rpc.rs:312 returns only 6 tools;
  symbol_* dispatcher arms at :807-971 unreachable by agent self-selection
  (B-29 PARTIAL).
- update_changed_files_symbol_graph at build_pipeline.rs:242 exists;
  indexing_service.rs never calls it (B-29 PARTIAL).
- KnowledgeDrawer.svelte:289-340 IMPACT ANALYSIS panel has hop band table
  but no submit_task input form or Tauri wiring (B-29 PARTIAL).
- export_graph_mermaid Tauri command at commands.rs:1484-1508; no dispatcher
  entry in orchestrator (B-33 MISSING).
- /graph mermaid absent from crates/cli/src/commands.rs (B-33 MISSING).
- GraphWindow.svelte:124-193 cytoscape style array has no decomposition-level
  shape selectors or status color classes (B-33 PARTIAL).
- ActivityWindow.svelte has no "Show blast radius" cross-link on file-write
  rows (B-33 PARTIAL).
- crates/dashboard-v2/src-tauri/tests/ empty; graph_export_dispatch.rs
  absent from orchestrator/tests/ (B-33 MISSING).

## Reads

- F:\.ELAI_workflow\docs\thefinalglance\verification\reports\B-plan\B-29_symbol_level_impact_api.report.md
- F:\.ELAI_workflow\docs\thefinalglance\verification\reports\B-plan\B-33_interactive_graph_viewer.report.md
- .elai_cc/crates/orchestrator/src/production_agent_rpc.rs (build_tool_catalog:312, symbol_* arms:807-971)
- .elai_cc/crates/orchestrator/src/dispatcher.rs (existing tool dispatch pattern)
- .elai_cc/crates/orchestrator/src/indexing_service.rs (incremental-update path)
- .elai_cc/crates/knowledge/src/symbol_graph/build_pipeline.rs (update_changed_files_symbol_graph:242, INCREMENTAL_SYMBOL_THRESHOLD:44-46)
- .elai_cc/crates/cli/src/commands.rs (SlashCommand enum, existing /runtime or /model pattern)
- .elai_cc/crates/dashboard-v2/src/lib/drawers/KnowledgeDrawer.svelte (IMPACT ANALYSIS panel:289-340)
- .elai_cc/crates/dashboard-v2/src/lib/windows/GraphWindow.svelte (cytoscape style array:124-193)
- .elai_cc/crates/dashboard-v2/src/lib/windows/ActivityWindow.svelte (file-write SSB row pattern)
- .elai_cc/crates/dashboard-v2/src-tauri/src/commands.rs (export_graph_mermaid:1484-1508)

## Tasks

1. **Add ToolDefinition entries for symbol_* tools** — in
   production_agent_rpc.rs build_tool_catalog (~line 312): append four
   ToolDefinition entries: symbol_impact (target: string, depth: u8 default 2),
   symbol_flow (target: string, depth: u8 default 5), symbol_context (target:
   string), symbol_list (prefix: string). Match the JSON schema shape used by
   the six existing tools in the catalog. Impact check: grep build_tool_catalog
   call sites before edit.

2. **Wire update_changed_files_symbol_graph into indexing_service.rs** — in
   the incremental-update path (identify the branch where changed_file_count is
   computed): if changed_file_count <= INCREMENTAL_SYMBOL_THRESHOLD call
   update_changed_files_symbol_graph(changed_paths); else call full
   build_symbol_graph. Import INCREMENTAL_SYMBOL_THRESHOLD from build_pipeline.
   Emit SsbActionType::SymbolGraphUpdated with is_incremental=true on the
   incremental path. Impact check: grep update_changed_files_symbol_graph and
   build_symbol_graph call sites before edit.

3. **Add /symbol_impact input form to KnowledgeDrawer.svelte** — in the
   IMPACT ANALYSIS panel (KnowledgeDrawer.svelte:289-340): add a target input
   field and a "Run" button that calls invoke("submit_task", {task:
   "/symbol_impact target=" + target}) via Tauri invoke. Display the returned
   blast_radius list below the existing hop band table. Follow the Svelte 5
   runes pattern ($state/$effect) already used in the component.

4. **Add /graph mermaid slash command to CLI** — in
   crates/cli/src/commands.rs: add SlashCommand::GraphMermaid { mode:
   GraphMode, max_nodes: usize } variant. In the command dispatch match arm,
   call invoke_export_graph_mermaid (IPC call to the orchestrator dispatcher
   export_graph_mermaid tool). Print the returned Mermaid string to stdout.
   Mode values: "file" | "symbol" | "decomposition". Default max_nodes: 200.
   Match the slash-command dispatch pattern of existing /runtime or /model
   commands.

5. **Wire export_graph_mermaid as dispatcher tool** — in
   crates/orchestrator/src/dispatcher.rs: add a DispatchTool::ExportGraphMermaid
   arm. The tool handler calls export_graph_mermaid from the knowledge crate's
   mermaid module (the same function the Tauri command delegates to). Input
   schema: { project_id: string, mode: string, max_nodes: usize }. Output:
   { mermaid: string }. This enables REPL and IPC callers to request Mermaid
   export without the Tauri layer. Impact check: grep DispatchTool enum before
   edit.

6. **Add decomposition-mode cytoscape style selectors to GraphWindow.svelte**
   — in the cytoscape style array (~line 124-193): append selector blocks for:
   node[level="Concept"] (large rounded rectangle, width 180, border 3px),
   node[level="Phase"] (medium rounded, width 140, border 2px),
   node[level="Step"] (small rounded, width 100, border 1px),
   node[level="BabyStep"] (circle shape, width 60).
   Status color classes: .status-pending (background-color #9e9e9e),
   .status-running (background-color #1976d2 with animation via transition
   property), .status-passed (background-color #388e3c),
   .status-failed (background-color #d32f2f),
   .status-blocked (background-color #f57c00).
   Guard with: apply level and status classes only when graphMode === "decomposition".

7. **Add "Show blast radius" action to ActivityWindow.svelte file-write rows**
   — identify the file-write SSB row rendering pattern in ActivityWindow.svelte.
   Add a "Show blast radius" button/link on each file-write row. On click: call
   graph.svelte.ts store's request_impact(filePath) + set_mode("file") + trigger
   open of GraphWindow via workspace state openWindow("graph"). Mirror the
   pattern already used in KnowledgeDrawer.svelte:308 and
   GraphSidebarPanel.svelte:49.

8. **Create crates/orchestrator/tests/graph_export_dispatch.rs** — end-to-end
   test for the export_graph_mermaid dispatcher tool:
   (a) construct a minimal IPC DispatchRequest with tool="export_graph_mermaid";
   (b) assert response is Ok and mermaid field starts with "graph " or "flowchart";
   (c) assert output is deterministic across two calls with identical input
   (BTreeMap iteration property from mermaid.rs);
   (d) assert VizData::empty() path returns non-empty Mermaid ("%% empty graph"
   header present).

9. **Create crates/dashboard-v2/src-tauri/tests/graph_commands.rs** — Tauri
   command smoke tests with fixture project:
   (a) get_graph_view_data in file mode returns VizData with files.len() >= 1;
   (b) get_impact_overlay for a known symbol returns non-empty overlay_nodes;
   (c) export_graph_mermaid in symbol mode returns string containing "classDef";
   (d) all three commands return Ok (no panic) on an empty project fixture.

10. **Decision doc** — write workflow/reasoning/C-24-decisions.md covering:
    dispatcher tool catalog approach (ToolDefinition shape chosen vs alternatives),
    incremental threshold branch placement in indexing_service.rs, cytoscape
    selector guard strategy for decomposition mode, ActivityWindow cross-link
    store call pattern.

## WIRING_CONTRACT

produces_symbols:
  - ToolDefinition entries for symbol_impact, symbol_flow, symbol_context,
    symbol_list in build_tool_catalog (production_agent_rpc.rs)
  - update_changed_files_symbol_graph call in indexing_service.rs incremental
    path (gated by INCREMENTAL_SYMBOL_THRESHOLD)
  - /symbol_impact input form + invoke("submit_task") in KnowledgeDrawer.svelte
  - SlashCommand::GraphMermaid variant in crates/cli/src/commands.rs
  - DispatchTool::ExportGraphMermaid arm in crates/orchestrator/src/dispatcher.rs
  - decomposition-level cytoscape selectors and status color classes in
    GraphWindow.svelte (Concept/Phase/Step/BabyStep shapes; 5 status classes)
  - "Show blast radius" button on ActivityWindow file-write SSB rows
  - crates/orchestrator/tests/graph_export_dispatch.rs (4 assertions)
  - crates/dashboard-v2/src-tauri/tests/graph_commands.rs (4 smoke tests)
consumes_symbols:
  - symbol_* dispatcher arms in production_agent_rpc.rs:807-971 (already exist)
  - update_changed_files_symbol_graph + INCREMENTAL_SYMBOL_THRESHOLD
    (knowledge::symbol_graph::build_pipeline — already implemented, C-17 stable)
  - export_graph_mermaid Tauri command (commands.rs:1484; function already exists)
  - mermaid generator (knowledge::visualization::mermaid — B-33 shipped)
  - graph.svelte.ts store (request_impact, set_mode, trigger_refresh — B-33 shipped)
  - workspace.svelte.ts openWindow("graph") (B-33 shipped)
  - SsbActionType::SymbolGraphUpdated + SymbolGraphUpdatedPayload (B-33 shipped)
  - VizData::empty() (knowledge::visualization::types — B-33 shipped)
reachable_from:
  - agent tool catalog (build_tool_catalog) — symbol_* tools self-selectable by LLM
  - indexing_service.rs incremental watcher path
  - dashboard KnowledgeDrawer IMPACT ANALYSIS panel (submit_task invoke)
  - CLI /graph mermaid command (elai.bat repl "/graph mermaid --mode file")
  - orchestrator dispatcher IPC (export_graph_mermaid tool)
  - dashboard ActivityWindow file-write SSB rows ("Show blast radius" action)
stub_ban:
  - build_tool_catalog MUST include all four symbol_* ToolDefinitions; partial
    catalog (e.g. only symbol_impact) is not acceptable
  - update_changed_files_symbol_graph MUST be called on the incremental path;
    calling build_symbol_graph unconditionally is forbidden after this C-plan
  - /graph mermaid MUST produce output via the real mermaid generator (same path
    as Tauri command); a stub println is not acceptable
  - DispatchTool::ExportGraphMermaid MUST delegate to the knowledge crate mermaid
    fn, not re-implement generation inline
  - ActivityWindow "Show blast radius" MUST use graph.svelte.ts store (not direct
    Tauri invoke); consistency with KnowledgeDrawer and GraphSidebarPanel required
frontend_pair:
  - GraphWindow.svelte decomposition selectors (Task 6)
  - KnowledgeDrawer.svelte IMPACT ANALYSIS input form (Task 3)
  - ActivityWindow.svelte file-write cross-link (Task 7)
observable_effect:
  trigger: run `elai /task --mode cloud "rename the SsbActionType enum and
           update all callers"` with cloud Sonnet 4.6 against real ELAI codebase
  expect_ssb: trace SSB contains SymbolQuery row with tool="symbol_impact" and
              target="SsbActionType"; blast_radius list is non-empty
  expect_artifact: GraphWindow renders affected nodes with cytoscape decomposition
                   styles; /graph mermaid SsbActionType produces valid Mermaid
                   output (deterministic, zero-LLM); export_graph_mermaid Tauri
                   command returns the same string; KnowledgeDrawer /symbol_impact
                   panel lists >= 10 callers from elai.map evidence

## DETERMINISTIC_CHECK

```bash
# 1. Four symbol_* ToolDefinitions in build_tool_catalog
grep -n "symbol_impact\|symbol_flow\|symbol_context\|symbol_list" \
  .elai_cc/crates/orchestrator/src/production_agent_rpc.rs | grep -i "ToolDefinition"

# 2. update_changed_files_symbol_graph + INCREMENTAL_SYMBOL_THRESHOLD in indexing_service
grep -n "update_changed_files_symbol_graph\|INCREMENTAL_SYMBOL_THRESHOLD" \
  .elai_cc/crates/orchestrator/src/indexing_service.rs

# 3. SlashCommand::GraphMermaid in CLI; ExportGraphMermaid in dispatcher
grep -n "GraphMermaid" .elai_cc/crates/cli/src/commands.rs
grep -n "ExportGraphMermaid\|export_graph_mermaid" \
  .elai_cc/crates/orchestrator/src/dispatcher.rs

# 4. Decomposition cytoscape selectors in GraphWindow
grep -n "Concept\|BabyStep\|status-running\|status-passed" \
  .elai_cc/crates/dashboard-v2/src/lib/windows/GraphWindow.svelte | head -10

# 5. Show blast radius in ActivityWindow
grep -n "blast_radius\|request_impact" \
  .elai_cc/crates/dashboard-v2/src/lib/windows/ActivityWindow.svelte

# 6. Test files exist
test -f .elai_cc/crates/orchestrator/tests/graph_export_dispatch.rs && echo "EXISTS"
test -f .elai_cc/crates/dashboard-v2/src-tauri/tests/graph_commands.rs && echo "EXISTS"

# 7. cargo check + tests
cargo check -p elai-orchestrator -p elai-cli 2>&1 | tail -3
cargo test -p elai-orchestrator --test graph_export_dispatch 2>&1 | grep "test result"
cargo test -p elai-contracts 2>&1 | grep "test result"

# 8. CLI zero-LLM smoke (deterministic)
elai repl "/graph mermaid SsbActionType" 2>&1 | grep -E "^graph |^flowchart "

# 9. svelte-check dashboard
cd .elai_cc/crates/dashboard-v2 && npx svelte-check --output human 2>&1 | tail -5
```

## ACCEPTANCE

- [ ] build_tool_catalog in production_agent_rpc.rs returns >= 10 ToolDefinitions;
      symbol_impact, symbol_flow, symbol_context, symbol_list all present with
      correct JSON schema (target/depth/prefix fields)
- [ ] Agent cloud Sonnet smoke test: `elai /task --mode cloud "rename the
      SsbActionType enum and update all callers"` produces trace SSB with
      SymbolQuery row tool="symbol_impact", target="SsbActionType"; blast_radius
      list is non-empty (>= 1 caller)
- [ ] update_changed_files_symbol_graph called from indexing_service.rs when
      changed_file_count <= INCREMENTAL_SYMBOL_THRESHOLD; build_symbol_graph
      called when changed_file_count > threshold; grep confirms both branches
- [ ] SsbActionType::SymbolGraphUpdated emitted with is_incremental=true on
      incremental path (verifiable via grep on indexing_service.rs)
- [ ] KnowledgeDrawer.svelte IMPACT ANALYSIS panel has target input + "Run"
      button invoking submit_task; running `elai /symbol_impact SsbActionType`
      via the dashboard panel returns >= 10 callers from elai.map evidence
- [ ] SlashCommand::GraphMermaid variant present in crates/cli/src/commands.rs;
      `elai repl "/graph mermaid SsbActionType"` produces Mermaid syntax output
      (starts with "graph " or "flowchart "; zero-LLM, deterministic)
- [ ] DispatchTool::ExportGraphMermaid in dispatcher.rs delegates to knowledge
      mermaid fn; same output as Tauri export_graph_mermaid command for identical
      input (round-trip determinism verified by graph_export_dispatch.rs test)
- [ ] GraphWindow.svelte cytoscape style array includes Concept/Phase/Step/
      BabyStep shape selectors and 5 status color classes; selectors guard-gated
      to decomposition mode only; svelte-check passes
- [ ] ActivityWindow.svelte file-write SSB rows have "Show blast radius" action
      calling graph.svelte.ts request_impact + openWindow("graph"); pattern
      consistent with KnowledgeDrawer.svelte:308 and GraphSidebarPanel.svelte:49
- [ ] crates/orchestrator/tests/graph_export_dispatch.rs exists; all 4 assertions
      pass (Ok response, Mermaid header, determinism, empty-graph stub)
- [ ] crates/dashboard-v2/src-tauri/tests/graph_commands.rs exists; all 4 smoke
      tests pass (file mode VizData, impact overlay, symbol mode classDef, empty
      fixture no-panic)
- [ ] cargo check -p elai-orchestrator -p elai-cli clean after all changes
- [ ] svelte-check on dashboard-v2 clean after GraphWindow and ActivityWindow edits
- [ ] workflow/reasoning/C-24-decisions.md written

## Files touched

- MODIFY: .elai_cc/crates/orchestrator/src/production_agent_rpc.rs
          (build_tool_catalog: +4 ToolDefinition entries for symbol_*)
- MODIFY: .elai_cc/crates/orchestrator/src/indexing_service.rs
          (incremental path: conditional update_changed_files_symbol_graph call)
- MODIFY: .elai_cc/crates/orchestrator/src/dispatcher.rs
          (add DispatchTool::ExportGraphMermaid arm)
- MODIFY: .elai_cc/crates/cli/src/commands.rs
          (add SlashCommand::GraphMermaid variant + dispatch arm)
- MODIFY: .elai_cc/crates/dashboard-v2/src/lib/drawers/KnowledgeDrawer.svelte
          (add target input + submit_task invoke to IMPACT ANALYSIS panel)
- MODIFY: .elai_cc/crates/dashboard-v2/src/lib/windows/GraphWindow.svelte
          (add decomposition-mode cytoscape shape + status selectors)
- MODIFY: .elai_cc/crates/dashboard-v2/src/lib/windows/ActivityWindow.svelte
          (add "Show blast radius" action on file-write SSB rows)
- ADD: .elai_cc/crates/orchestrator/tests/graph_export_dispatch.rs
- ADD: .elai_cc/crates/dashboard-v2/src-tauri/tests/graph_commands.rs
- ADD: workflow/reasoning/C-24-decisions.md

## Next

C-21, C-22, C-23, C-25 (all parallel-safe with this plan). After all C-plan
phases complete, transition to final integration smoke pass and MEMORIZING.
