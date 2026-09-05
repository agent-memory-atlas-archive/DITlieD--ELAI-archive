# B-33: Interactive Decomposition + Symbol Graph Viewer (with Mermaid export)
Objective: ship a dashboard window that visualises the file-import graph,
the symbol-level call graph, and the DecompositionTree (concept → phase →
step → baby step) interactively, with right-click blast-radius overlay,
live search, layout switcher, PNG export, AND a lightweight Mermaid text
export for cases where the user wants to paste a diagram into a PR
description or render it in Claude web client without webview.
Estimated effort: 2-3 sessions (~12-18 hours).
Pre-requisites: B-7 (DecompositionTree producer wired), B-29 (symbol
graph cache exists), B-36 Session 2 (V2 AppState + engine deps so the
GraphWindow's Tauri commands have a backend handle to the SymbolGraphCache).
Mermaid sub-step can land independently.
Parallel-safe with: B-26 (4K.6-4K.13 dashboard completion — they touch
different windows).

## V2 REBASELINE NOTE (2026-05-07)

Original B-33 path locations were V1-style (`crates/dashboard/svelte/src/`).
Re-resolve every path against V2:
  V1 path                                          V2 path
  ----------------------------------------------------------------------
  crates/dashboard/src/commands.rs              -> crates/dashboard-v2/src-
                                                    tauri/src/lib.rs
  crates/dashboard/svelte/src/windows/          -> crates/dashboard-v2/src/
                                                    lib/windows/
  crates/dashboard/svelte/src/drawers/          -> crates/dashboard-v2/src/
                                                    lib/drawers/
  crates/dashboard/svelte/package.json          -> crates/dashboard-v2/
                                                    package.json
  crates/dashboard/tests/                       -> crates/dashboard-v2/
                                                    src-tauri/tests/

Component-naming reminders for V2:
  - V2 has 7 windows already scaffolded. GraphWindow.svelte would be the 8th
    new window — confirm with user before adding (V2 design ships 7 spec
    windows; an 8th may need DOMAIN-CONTEXT.md update).
  - Cytoscape integration: V2 already has `three.js@0.184` + `troika-three-
    text@0.52` in package.json. Adding `cytoscape` (~80KB gzipped) is a NEW
    npm dep — confirm with user. Alternative: render the graph directly in
    the existing orbit canvas via Mode 2 MAP (DESIGN.md §8) which already
    handles PageRank-ranked symbols in a score-banded ring.
  - Use Svelte 5 runes ($state/$effect) — NO `$:` reactive labels.
  - Mermaid text export: pure-data export, no new dep needed.

Decision the executing session must make:
  Option A: Add cytoscape + new GraphWindow.svelte alongside existing 7
            windows. Heaviest, most flexible.
  Option B: Reuse the orbit's Mode 2 MAP surface. Lightest. Trade-off:
            graph viewer becomes a mode of ActivityWindow instead of its
            own surface.
  Option C: Drawer-only viewer in KnowledgeDrawer.svelte (PageRank + edges
            already feasible with existing three.js). No new window.
            Smallest blast radius.

Recommendation: Option B or C unless the user explicitly wants a
right-click contextual flyout that the orbit cannot host.

## Context

Per dashboard wiring inventory (DETAILED_SUMMARY_03 §3 items 24-27 +
inventory section 22): "DecompositionTree visualizer (Concept → Phase →
Step → BabyStep)", "ExecutablePipeline DAG visualizer", "DAG repair events
log", "Web research per-node FORMULATE/SEARCH/EXTRACT/REFLECT/CONVERGE
log" — all four are listed as INVISIBLE in the audit. The dashboard ships
9 windows + 10 drawers but no graph viewer. PartialPlanTree (the closest
existing) only polls PhaseComplete SSB rows.

When ELAI is solving a real task today, the user can see:
- A flat plan tree of phases
- An activity feed of SSB rows
- A status bar with FSM state (after B-2)

What they cannot see:
- File-import dependency graph of the project being worked on
- Symbol-level call graph (no UI exists for B-29's data)
- DecompositionTree as a node-graph (only as a flat list)
- DAG repair events (REORDER / SUBSTITUTE) over time
- "What will my edit affect" overlay (B-29 has the data, no visualizer)

SocratiCode demonstrates the contract works in practice. Their
`src/services/graph-visualize-html.ts` (340 LOC) builds a single
self-contained HTML file with cytoscape.js + dagre layout, dual file/symbol
view with size-capped toggle (`MAX_SYMBOLS=20_000`,
`MAX_EDGES=60_000`), blast-radius overlay on right-click, live search,
six layout algorithms, PNG export. Their `src/services/graph-analysis.ts:
113-210` (which I missed in the first pass per the agent investigation)
implements **Mermaid diagram generation** with language-coded node colors
and cyclic-edge highlighting in red — far simpler than full cytoscape
and rendered natively in Claude web, GitHub, GitLab, and most editor
preview panes.

This B-step builds a Tauri+Svelte+cytoscape.js implementation of the
interactive view (cytoscape from npm — NOT vendored AGPL JS), plus a
Mermaid export endpoint that returns text suitable for chat / PR /
markdown rendering. Two output modes serve two audiences:
- Interactive HTML for "explore my codebase right now"
- Mermaid text for "paste this into the PR description"

## Reads (mandatory before coding)

- F:\.ELAI_workflow\docs\.source\SocratiCode-main\SocratiCode-main\src\services\graph-visualize-html.ts
  (FULL — 340 LOC; data shape, size caps, asset bundling)
- F:\.ELAI_workflow\docs\.source\SocratiCode-main\SocratiCode-main\src\services\graph-analysis.ts
  (FULL — Mermaid generator at lines 113-210, cycle detection, stats)
- F:\.ELAI_workflow\docs\.source\SocratiCode-main\SocratiCode-main\src\assets\viewer-app.js
  (REFERENCE — interaction model: click = sidebar, right-click =
  blast radius, search box, layout switcher, PNG export)
- F:\.ELAI_workflow\docs\.source\SocratiCode-main\SocratiCode-main\src\assets\viewer-styles.css
  (REFERENCE — visual idiom; ELAI dashboard already has its own design
  system per B-11, do not copy literally)
- F:\.ELAI_workflow\.elai_cc\dashboard-wiring-inventory.txt
  GROUP 13 (Pipeline Factory L2 / Execution Engine — DecompositionTree,
  TreeNavigator, BubbleUpHandler — these are the data sources)
- F:\.ELAI_workflow\.elai_cc\crates\dashboard\svelte\src\
  (existing window components — to mirror style)
- DETAILED_SUMMARY_03_gaps_and_roadmap.md §3 items 24-27

## Tasks

### Phase 1 — Data shape contract (Session 1, half)

1. Define `VizData` types in
   `crates/knowledge/src/visualization.rs` (or `crates/dashboard/src/`,
   wherever it ends up — caller-side):
   ```rust
   pub struct VizData {
       pub project: VizProject,
       pub files: Vec<VizFile>,
       pub file_edges: Vec<VizFileEdge>,
       pub symbols: Vec<VizSymbol>,
       pub symbol_edges: Vec<VizSymbolEdge>,
       pub symbols_by_file: BTreeMap<String, Vec<VizSymbol>>,
       pub symbol_mode: SymbolMode,           // Full | Capped | Omitted
       pub symbol_omit_reason: Option<String>,
   }
   pub struct VizProject {
       pub name: String,
       pub files: usize,
       pub edges: usize,
       pub symbols: Option<usize>,
       pub call_edges: Option<usize>,
       pub built_at: Option<i64>,
   }
   pub struct VizFile {
       pub id: String,        // relative path
       pub label: String,     // basename
       pub language: String,
       pub deps: usize,
       pub dependents: usize,
       pub symbol_count: usize,
   }
   pub struct VizFileEdge {
       pub source: String,
       pub target: String,
       pub kind: VizEdgeKind, // Import | DynamicImport | ReExport
       pub cyclic: bool,
   }
   pub struct VizSymbol {
       pub id: String,
       pub file: String,
       pub name: String,
       pub qualified_name: String,
       pub kind: String,
       pub line: u32,
   }
   pub struct VizSymbolEdge {
       pub source: String,
       pub target: String,
       pub confidence: knowledge::symbol_graph::Confidence,
   }
   ```
   Same shape as SocratiCode's `VizData`
   (`src/services/graph-visualize-html.ts:51-92`) for consistency
   with the proven UX.

2. Constants:
   ```rust
   pub const MAX_VIZ_SYMBOLS: usize = 20_000;
   pub const MAX_VIZ_EDGES: usize = 60_000;
   pub const MAX_VIZ_SYMS_PER_FILE: usize = 200;
   ```
   Above either symbol/edge cap, return `SymbolMode::Capped` or `Omitted`
   with a reason like "symbol graph too large for embed; use
   /symbol_impact target=X for individual queries".

### Phase 2 — Tauri backend (Session 1)

3. Add Tauri command in `crates/dashboard/src/commands.rs`:
   ```rust
   #[tauri::command]
   pub async fn get_graph_view_data(
       project_id: String,
       mode: String, // "file" | "symbol" | "decomposition"
   ) -> Result<VizData, String>
   ```
   - `mode == "file"`: load file-import graph + per-file symbol counts
     + symbol payloads up to `MAX_VIZ_SYMS_PER_FILE` per file (for
     sidebar). Symbol mode `Full`/`Capped`/`Omitted` based on totals.
   - `mode == "symbol"`: same plus full symbol_edges if under caps.
   - `mode == "decomposition"`: load DecompositionTree from
     orchestrator (read from SSB rows or directly from
     pipeline_tree_navigator state). Return as VizFile/VizFileEdge
     where files = nodes (Concept/Phase/Step/BabyStep) and edges =
     parent-child relations + dependency arrows.

4. Add Tauri command for blast-radius overlay:
   ```rust
   #[tauri::command]
   pub async fn get_impact_overlay(
       project_id: String,
       target: String,
       depth: u32,
   ) -> Result<ImpactOverlay, String>
   ```
   Wraps `compute_impact_radius` (B-29) and returns
   `{ files: BTreeSet<String>, by_depth: BTreeMap<u32, Vec<String>> }`
   so the viewer can highlight nodes by hop count.

5. Add Mermaid export command:
   ```rust
   #[tauri::command]
   pub async fn export_graph_mermaid(
       project_id: String,
       mode: String, // "file" | "symbol" | "decomposition"
       max_nodes: Option<usize>,
   ) -> Result<String, String>
   ```
   - Implements Mermaid generation in
     `crates/knowledge/src/visualization/mermaid.rs` mirroring
     SocratiCode's `graph-analysis.ts:113-210` algorithm:
     - Per-language node color via Mermaid `classDef`
     - Cyclic edges in red dotted (`-.->|cycle|`)
     - Compact node IDs (sanitize relative paths to valid Mermaid IDs)
     - Legend block at top
     - Header + footer
   - Default `max_nodes = 200` (Mermaid degrades visually above ~250
     nodes; SocratiCode caps at 250).
   - Output is a `String` — caller decides whether to copy to
     clipboard, save to file, or display inline.

### Phase 3 — Svelte UI (Session 2)

6. Create `crates/dashboard/svelte/src/windows/GraphWindow.svelte`:
   - Layout: header (project name + stats + mode toggle) + main
     cytoscape canvas + right sidebar (selected node details) + footer
     (search box + layout dropdown + Mermaid export button + PNG export
     button).
   - Mode toggle: `[Files] [Symbols] [Decomposition]`. Files always
     enabled; Symbols disabled when `symbol_mode == Omitted` with hint
     "Graph too large; use /symbol_impact for queries"; Decomposition
     enabled when DecompositionTree exists in current task.
   - Cytoscape from npm (`cytoscape`, `cytoscape-dagre`,
     `cytoscape-fcose`) — explicit user confirmation needed for these 3
     deps (CLAUDE.md no-new-deps rule).

7. Sidebar interactions:
   - Click node → show file/symbol details: language badge, line range,
     deps in / deps out, symbols list (capped at 200), action buttons
     (`Trace flow from here`, `Show blast radius`, `Open file`).
   - Right-click node → call `get_impact_overlay`, highlight returned
     files in red gradient by hop depth (1 = bright red, deeper =
     fading), display hop counter as overlay banner.
   - Search box → filter visible nodes live (fuzzy match on label /
     qualified_name / file path).
   - Layout dropdown: dagre / fcose / concentric / breadth-first / grid
     / circle (mirror SocratiCode 6 layouts).
   - PNG export → `cy.png({ output: 'blob', full: true })` → save via
     Tauri dialog API.
   - Mermaid export button → call `export_graph_mermaid` → show modal
     with Mermaid text + copy-to-clipboard + save-as-`.mmd` button.

8. Cytoscape styling — use ELAI's design system (B-11):
   - Node fill by language (via `classDef` equivalent):
     - rust: ELAI brand teal `#1B7A6E`
     - python: blue `#3776AB`
     - typescript / javascript: yellow `#F7DF1E`
     - svelte: orange `#FF3E00`
     - go: cyan `#00ADD8`
     - jvm (java/kotlin/scala): purple `#7C5CBF` (ELAI brand purple)
     - csharp: green `#239120`
     - other: neutral gray
   - Edge styling: import = solid, dynamic-import = dashed,
     re-export = dotted, cyclic = red
   - Selected node halo + dim everything else
   - Right-click overlay = red gradient on impacted nodes

9. Decomposition mode-specific styling:
   - Concept = large rounded rectangle
   - Phase = medium rounded rectangle
   - Step = small rounded rectangle
   - BabyStep = circle
   - Status indicators: pending = gray, running = blue pulse, passed =
     green, failed = red, blocked = orange
   - Edges: parent-child = solid arrow, dependency = dashed arrow

### Phase 4 — Wiring (Session 2-3)

10. Register `GraphWindow` in `WindowRegistry.ts`. Sidebar icon
    (graph/network glyph) + Ctrl+G keybinding to open.
11. Top of GraphWindow shows "Refresh" button that re-fetches
    `get_graph_view_data` (graph rebuilds in background per B-31's
    watcher path).
12. SSB-driven live updates: subscribe to `IndexingComplete` and
    `SymbolGraphUpdated` action types — when received, refetch
    automatically.
13. Add cross-link from existing windows:
    - Plan tree node click → "Show in Graph" → opens GraphWindow in
      decomposition mode, focuses that node
    - Activity feed file-write entry → "Show blast radius" → opens
      GraphWindow in file mode, runs impact overlay on the written file
    - KnowledgeDrawer (post-B-29) gets a "Visualize" button next to
      symbol query results.
14. Mermaid export — also expose via CLI command:
    ```
    elai.bat repl '/graph mermaid > graph.md'
    ```
    Maps to a new dispatcher tool `export_graph_mermaid` so users can
    paste graphs into PR descriptions without opening the dashboard.

## WIRING_CONTRACT

produces_symbols:
  - knowledge::visualization::types::{VizData, VizProject, VizFile,
    VizFileEdge, VizSymbol, VizSymbolEdge, SymbolMode}
  - knowledge::visualization::mermaid::generate_mermaid
  - dashboard::commands::get_graph_view_data (Tauri command)
  - dashboard::commands::get_impact_overlay (Tauri command)
  - dashboard::commands::export_graph_mermaid (Tauri command)
  - dispatcher tool entry: `export_graph_mermaid` for CLI/IPC use
  - Svelte component: GraphWindow.svelte
  - Svelte component: GraphSidebarPanel.svelte (selected node details)
  - Svelte component: MermaidExportModal.svelte
  - core::ssb::SsbActionType::SymbolGraphUpdated (new variant — used
    for live refetch trigger)
consumes_symbols:
  - knowledge::symbol_graph::cache::SymbolGraphCache (B-29) for symbol
    mode + impact overlay
  - knowledge::dkg::engine for file-import graph
  - orchestrator::pipeline_tree_navigator::DecompositionTree for
    decomposition mode
  - dashboard::svelte::WindowRegistry registers GraphWindow
reachable_from:
  - dashboard::lib::run (Tauri window mount)
  - cli::main::handle_graph_command (mermaid export from REPL)
stub_ban:
  - VizData.symbol_mode is always one of Full | Capped | Omitted with a
    populated reason when not Full
  - GraphWindow does not call cytoscape with > MAX_VIZ_EDGES edges
  - export_graph_mermaid returns at least the project header even when
    graph is empty (no `unwrap_or_default` returning empty string)
frontend_pair: B-29 (symbol cache backs symbol mode)
observable_effect:
  trigger: open dashboard, run /index on a real ELAI repo, open
    Ctrl+G GraphWindow, click on a file node
  expect_ssb: SymbolGraphUpdated action when index completes
  expect_artifact: cytoscape canvas renders ~16 nodes (ELAI crates) with
    inter-crate edges; right-click on `orchestrator` shows impact
    overlay highlighting cli, dashboard, contracts in red

## DETERMINISTIC_CHECK

```bash
# 1. Tauri commands compile and register
cargo check -p dashboard
# Expected: GraphWindow + 3 commands compile.

# 2. Mermaid export from CLI
elai.bat repl "/graph mermaid --mode file" > /tmp/elai_graph.md
head -5 /tmp/elai_graph.md
# Expected: starts with "graph TD" or "flowchart TD" + classDef lines.

# 3. Mermaid output renders in upstream tools
# (manual smoke test — paste into github.com/<user>/<repo>/issues
# preview or claude.ai chat; should render).

# 4. PNG export saves a non-empty file
# (manual smoke test — click PNG button, save, verify > 1KB).

# 5. Impact overlay highlights correct files
elai.bat repl "/symbol_impact target=ClockProvider"  # known >20 files
# Then in dashboard: open GraphWindow, right-click ClockProvider node.
# Verify highlighted file count matches CLI output.

# 6. Size cap honored
# Synthetic test: build a fixture with 25K symbols (above cap), open
# GraphWindow in symbol mode. Verify symbol mode is disabled with hint;
# file mode still works.

# 7. Live refetch on indexing complete
elai.bat repl "/index"  # rebuilds graph
# In dashboard with GraphWindow open: verify graph repaints within 2s
# after IndexingComplete SSB row.

# 8. Cytoscape deps approved (after user confirmation)
cat crates/dashboard/svelte/package.json | jq '.dependencies | keys'
# Expected: cytoscape, cytoscape-dagre, cytoscape-fcose all present.
```

## ACCEPTANCE

- [ ] VizData types defined with same shape as SocratiCode contract
- [ ] 3 Tauri commands implemented: get_graph_view_data,
      get_impact_overlay, export_graph_mermaid
- [ ] GraphWindow.svelte renders 3 modes (File / Symbol / Decomposition)
- [ ] Right-click triggers impact overlay; nodes highlighted by hop depth
- [ ] Live search filters visible nodes
- [ ] 6 layout algorithms switchable
- [ ] PNG export produces valid blob
- [ ] Mermaid export produces valid Mermaid 11.x flowchart text with
      classDef language colors and cyclic-edge highlighting
- [ ] CLI `/graph mermaid` command outputs to stdout (paste-ready)
- [ ] Size caps enforced (Symbol mode disabled with reason above
      MAX_VIZ_SYMBOLS or MAX_VIZ_EDGES)
- [ ] Cross-window links: Plan tree → Graph (decomposition mode);
      ActivityFeed file-write → Graph (impact overlay)
- [ ] SSB SymbolGraphUpdated action triggers auto-refetch
- [ ] User explicitly confirmed adding cytoscape + cytoscape-dagre +
      cytoscape-fcose npm deps (CLAUDE.md no-new-deps rule)
- [ ] Zero `.unwrap()` in non-test source

## Tests to add

- `crates/knowledge/tests/visualization_types.rs` — round-trip
  serialization of VizData; size-cap enforcement
- `crates/knowledge/tests/mermaid_generation.rs`:
  - test cyclic edges rendered as red dotted
  - test language color classDef per language
  - test Mermaid output parses by `mermaid-cli` (if available in CI)
  - test deterministic output (same graph = same Mermaid string)
  - test max_nodes truncation honored
- `crates/dashboard/tests/graph_commands.rs` — Tauri command smoke tests
  with fixture project
- `crates/orchestrator/tests/graph_export_dispatch.rs` — CLI export
  command end-to-end
- `tests/contracts/graph_viewer_contract.rs` — VizData shape stable
  across versions (regression for schema drift)

## Files touched

- CREATE: `crates/knowledge/src/visualization/mod.rs`
- CREATE: `crates/knowledge/src/visualization/types.rs`
- CREATE: `crates/knowledge/src/visualization/mermaid.rs`
- CREATE: `crates/dashboard/src/commands.rs` (3 new Tauri commands —
  add to existing file)
- CREATE: `crates/dashboard/svelte/src/windows/GraphWindow.svelte`
- CREATE: `crates/dashboard/svelte/src/windows/GraphSidebarPanel.svelte`
- CREATE: `crates/dashboard/svelte/src/windows/MermaidExportModal.svelte`
- MODIFY: `crates/dashboard/svelte/src/WindowRegistry.ts` (register
  GraphWindow)
- MODIFY: `crates/dashboard/svelte/src/Sidebar.svelte` (graph icon +
  Ctrl+G binding)
- MODIFY: `crates/dashboard/svelte/package.json` (cytoscape deps —
  PENDING USER CONFIRMATION)
- MODIFY: `crates/cli/src/commands.rs` (`/graph mermaid` slash command)
- MODIFY: `crates/orchestrator/src/dispatcher.rs` (export_graph_mermaid
  dispatcher tool)
- MODIFY: `crates/core/src/ssb/action_types.rs` (SymbolGraphUpdated
  variant)
- MODIFY: `crates/dashboard/svelte/src/windows/Activity.svelte`
  (file-write rows get "Show blast radius" link)
- CREATE: `workflow/reasoning/B-33-decisions.md`

## Decision doc

Write `workflow/reasoning/B-33-decisions.md` documenting:
- npm deps: cytoscape (~370KB), cytoscape-dagre (~50KB), cytoscape-fcose
  (~80KB). Total ~500KB added to dashboard webview bundle. Document
  user-confirmed acceptance.
- Why NOT vendor JS like SocratiCode does (we have Vite bundle anyway,
  npm + tree-shake is cleaner)
- Mermaid version: 11.x classDef syntax; document min Mermaid version
  needed by downstream renderers
- Decomposition mode size limits: 4-level trees rarely exceed 200 nodes
  but Concept count × Phase × Step × BabyStep can. Cap at 500 nodes;
  show "use /plan list for full breakdown" beyond
- Should impact overlay be auto-applied on selected node, or only on
  right-click? Default right-click (less visual noise)
- Should the Mermaid export include node line counts / sizes? Yes for
  legibility on >50-node graphs; document the per-node label format

## Side benefits unlocked

- B-7 DecompositionTree visualizer requirement satisfied
- DAG repair events (REORDER / SUBSTITUTE) can be overlaid as
  before/after diff if Decomposition mode is reused for ExecutablePipeline
- User can share dependency graphs in PR descriptions with copy-paste
  Mermaid (no screenshot, no link to a transient HTML file)
- ELAI dashboard gains a "feel" win — visual data was a long-standing
  user complaint per Part II spec
- Mermaid export usable from CI to embed graphs in build artifacts /
  README updates
- Foundation for B-26 4K.6 EXPLORE window (Q&A interface) which can
  reference graph nodes by ID

## Next

B-26 (4K.6-4K.13 Dashboard Completion) — sibling work; some windows
  share Workspace shell. Coordinate.
B-29 (Symbol-Level Impact API) — required dependency.
B-7 (DecompositionTree producer wiring) — required for decomposition
  mode.
