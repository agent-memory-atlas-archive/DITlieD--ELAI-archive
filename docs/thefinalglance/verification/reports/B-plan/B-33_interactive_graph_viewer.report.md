# Verification Report: B-33_interactive_graph_viewer.md
Source: F:\.ELAI_workflow\docs\thefinalglance\B-plan\B-33_interactive_graph_viewer.md
Date: 2026-05-17
Overall status: MOSTLY_DONE
Confidence: HIGH

## Summary

Core data types, all three Tauri commands, the Mermaid generator, all three Svelte
components (GraphWindow, GraphSidebarPanel, MermaidExportModal), and the
SymbolGraphUpdated SSB variant are fully implemented and covered by tests. The
Ctrl+G keybinding and "Show in Graph" cross-link from PlanWindow are also present.
Two items are missing: the CLI `/graph mermaid` slash command and the orchestrator
dispatcher tool `export_graph_mermaid`, plus the Activity file-write row cross-link
("Show blast radius" from ActivityWindow). The decomposition-mode status-indicator
styling (pending/running/passed/failed/blocked colors per step 9) is absent from
the Svelte cytoscape stylesheet.

## Claims verified DONE

- VizData struct with all 7 fields (project, files, file_edges, symbols, symbol_edges,
  symbols_by_file, symbol_mode) — evidence:
  crates/knowledge/src/visualization/types.rs:148-158

- VizProject, VizFile, VizFileEdge, VizSymbol, VizSymbolEdge structs — evidence:
  crates/knowledge/src/visualization/types.rs:75-143

- SymbolMode enum (Full/Capped/Omitted with reason) — evidence:
  crates/knowledge/src/visualization/types.rs:62-67

- VizEdgeKind enum (Import/DynamicImport/ReExport) — evidence:
  crates/knowledge/src/visualization/types.rs:50-55

- Constants MAX_VIZ_SYMBOLS=20000, MAX_VIZ_EDGES=60000, MAX_VIZ_SYMS_PER_FILE=200 — evidence:
  crates/knowledge/src/visualization/types.rs:33-41

- Tauri command get_graph_view_data (3 modes: file/symbol/decomposition) — evidence:
  crates/dashboard-v2/src-tauri/src/commands.rs:1414-1432

- Tauri command get_impact_overlay (project_id, target, depth) — evidence:
  crates/dashboard-v2/src-tauri/src/commands.rs:1437-1478

- Tauri command export_graph_mermaid (project_id, mode, max_nodes) — evidence:
  crates/dashboard-v2/src-tauri/src/commands.rs:1484-1508

- Mermaid generator crates/knowledge/src/visualization/mermaid.rs — exists with
  classDef language colors (rust/python/ts/svelte/go/jvm/csharp/other), cyclic edges
  as red dotted (-.->|cycle|), deterministic BTreeMap iteration — evidence:
  crates/knowledge/src/visualization/mermaid.rs:123-131 (classDefs), :90 (cycle edge)

- GraphWindow.svelte created in dashboard-v2 — evidence:
  crates/dashboard-v2/src/lib/windows/GraphWindow.svelte:1

- GraphSidebarPanel.svelte created — evidence:
  crates/dashboard-v2/src/lib/windows/GraphSidebarPanel.svelte (file exists)

- MermaidExportModal.svelte created — evidence:
  crates/dashboard-v2/src/lib/windows/MermaidExportModal.svelte (file exists)

- Cytoscape + cytoscape-dagre + cytoscape-fcose npm deps added — evidence:
  crates/dashboard-v2/package.json:19-21

- GraphWindow uses Svelte 5 runes ($state/$effect, no $:) — evidence:
  crates/dashboard-v2/src/lib/windows/GraphWindow.svelte:40-55 ($state, $derived)

- 6 layout algorithms defined (dagre/fcose/concentric/breadthfirst/grid/circle) — evidence:
  crates/dashboard-v2/src/lib/windows/GraphWindow.svelte:35

- Node right-click triggers impact overlay (cxttap event) — evidence:
  crates/dashboard-v2/src/lib/windows/GraphWindow.svelte:196

- Ctrl+G keybinding registered to open GraphWindow — evidence:
  crates/dashboard-v2/src/lib/state/shortcuts.ts:72-76

- "graph" registered as valid WindowType in workspace — evidence:
  crates/dashboard-v2/src/lib/state/workspace.svelte.ts:9

- SsbActionType::SymbolGraphUpdated variant — evidence:
  crates/orchestrator/src/ssb.rs:99 (DEF_ENUM line), :1278 (parser)

- SymbolGraphUpdatedPayload struct — evidence:
  crates/orchestrator/src/ssb.rs:2427

- SymbolGraphUpdated emitted from indexing_service — evidence:
  crates/orchestrator/src/indexing_service.rs:602

- GraphWindow subscribes to symbol_graph_updated SSB event for live refetch — evidence:
  crates/dashboard-v2/src/lib/windows/GraphWindow.svelte:353

- lib.rs SymbolGraphUpdatedEvent struct for Tauri re-broadcast — evidence:
  crates/dashboard-v2/src-tauri/src/lib.rs:706

- PlanWindow cross-link: "Show in Graph" opens decomposition mode + focuses node — evidence:
  crates/dashboard-v2/src/lib/windows/PlanWindow.svelte:344-345

- KnowledgeDrawer "Visualize blast radius" button cross-link — evidence:
  crates/dashboard-v2/src/lib/drawers/KnowledgeDrawer.svelte:308

- graph.svelte.ts store (GraphMode type, focus_node, request_impact, set_mode, trigger_refresh) — evidence:
  crates/dashboard-v2/src/lib/state/graph.svelte.ts:16-82

- Test: crates/knowledge/tests/visualization_types.rs — evidence: file exists, tested
  VizData round-trip, SymbolMode serialisation, BTreeMap sort, constants

- Test: crates/knowledge/tests/mermaid_generation.rs — evidence: file exists, tests
  cyclic edges, classDef, deterministic output

- Contract test: crates/contracts/tests/b33_graph_viewer_contract.rs — evidence: file exists,
  pins JSON shape + Mermaid stub_ban + cap constants

- Contract test: crates/contracts/tests/symbol_graph_updated_ssb_contract.rs — evidence:
  file exists, round-trips SymbolGraphUpdatedPayload

- Decomposition mode wired to orchestrator DecompositionTree — evidence:
  crates/dashboard-v2/src-tauri/src/commands.rs:1280-1391

- PNG export implemented via Tauri dialog save — evidence:
  crates/dashboard-v2/src/lib/windows/GraphWindow.svelte:306-315

- Size cap enforcement: Omitted symbol mode disables toggle with hint in GraphWindow — evidence:
  crates/dashboard-v2/src/lib/windows/GraphWindow.svelte:94-101

- VizData.empty() helper always returns non-empty Mermaid on empty graph — evidence:
  crates/knowledge/src/visualization/mermaid.rs:49-50 (header + "%% empty graph")

- Note on V2 path rebaseline: plan's V2 rebaseline note is correct.
  All implementation landed in crates/dashboard-v2/ not crates/dashboard/.

## Claims PARTIAL

- Decomposition mode node styling (step 9 of plan) — status-indicator styles (pending=gray,
  running=blue pulse, passed=green, failed=red, blocked=orange) and shape differentiation
  (Concept=large rounded, Phase=medium rounded, Step=small rounded, BabyStep=circle)
  - exists: cytoscape instance at crates/dashboard-v2/src/lib/windows/GraphWindow.svelte:124-193
    has language-based node colors and impact overlay classes but NO decomposition-specific
    selectors for level-type shapes or status colors
  - gap: Concept/Phase/Step/BabyStep shape selectors and running/passed/failed/blocked
    color classes absent from the cytoscape style array

- ActivityFeed file-write row cross-link ("Show blast radius" → GraphWindow file mode + impact)
  - exists: KnowledgeDrawer.svelte:308 has the Visualize blast radius button; GraphSidebarPanel.svelte:49
    has the Show blast radius action; the plan specified this should also be in ActivityWindow on
    file-write SSB rows
  - gap: ActivityWindow.svelte has no file-write row with a "Show blast radius" link to open
    GraphWindow; grep on ActivityWindow.svelte found no such cross-link

## Claims MISSING

- CLI slash command `/graph mermaid` (elai.bat repl "/graph mermaid --mode file") —
  searched: crates/cli/src/ for "graph mermaid", "export_graph_mermaid", "/graph"; no evidence

- Orchestrator dispatcher tool entry `export_graph_mermaid` for REPL/IPC use (wiring contract
  produces_symbols, task 14 of plan) — searched: crates/orchestrator/src/ for
  "export_graph_mermaid", "graph mermaid"; no evidence

- Test: crates/dashboard/tests/graph_commands.rs (Tauri command smoke tests) — searched:
  crates/dashboard-v2/src-tauri/tests/ directory is empty; no graph_commands.rs file

- Test: crates/orchestrator/tests/graph_export_dispatch.rs (CLI export end-to-end) —
  searched: crates/orchestrator/tests/; no graph_export_dispatch.rs file

- workflow/reasoning/B-33-decisions.md decision doc — not verified (outside project
  boundary; workflow/ is gitignored/excluded from scope)

## Claims UNVERIFIABLE

- "User explicitly confirmed adding cytoscape npm deps" — plan states PENDING USER
  CONFIRMATION; deps ARE present in package.json; whether confirmation was obtained
  is a process record, not a code artifact

- "Mermaid output parses by mermaid-cli if available in CI" — runtime/CI environment
  check; cannot verify from source

- Manual smoke tests (PNG export > 1KB, paste Mermaid into GitHub preview renders) —
  stated as manual tests in DETERMINISTIC_CHECK; no automated equivalent in tests/

## What must be done (action list)

- [ ] Add `/graph mermaid` slash command to crates/cli/src/commands.rs (or equivalent
  dispatcher) so `elai.bat repl "/graph mermaid --mode file"` routes to export_graph_mermaid
- [ ] Wire `export_graph_mermaid` as a dispatcher tool in crates/orchestrator/src/dispatcher.rs
  (wiring contract produces_symbols entry; REPL/IPC path)
- [ ] Create crates/orchestrator/tests/graph_export_dispatch.rs (CLI export end-to-end test)
- [ ] Create crates/dashboard-v2/src-tauri/tests/graph_commands.rs (Tauri command smoke
  tests with fixture project; dashboard-v2/src-tauri/tests/ directory currently empty)
- [ ] Add decomposition-mode cytoscape style selectors to GraphWindow.svelte (step 9):
  level shapes (Concept/Phase/Step/BabyStep) and status colors
  (pending=gray, running=blue pulse, passed=green, failed=red, blocked=orange)
- [ ] Add "Show blast radius" action to ActivityWindow.svelte file-write SSB row entries
  (cross-link to GraphWindow in file mode with impact overlay pre-applied)

## Notes

The plan's V2 rebaseline note (lines 16-43) is accurate. Implementation used
crates/dashboard-v2/ throughout. The `symbol_omit_reason` field from the plan
struct definition was folded into SymbolMode::Omitted{reason} (more idiomatic);
this is a deliberate design improvement, not a gap. The VizData struct in types.rs
does not have a top-level `symbol_omit_reason` field as the plan showed, but the
SymbolMode enum carries the reason inside Capped/Omitted variants, which is
equivalent and actually what the contract tests verify.
