# H-10: Canvas shell + CODE-GRAPH (read-only) — the thesis slice that replaces the orbit

STATUS: PLANNED (WAVE 2; BUILD; the cheap thesis proof; B0 is BLOCKING — nothing renders until measured)

Objective: Stand up a 2D generative canvas as a new WindowHost window type rendering the codebase graph, dead code
visibly disconnected — the mode the user described first ("a big graph where each node is a part of the codebase,
every connection the real plumbing, dead code literally not-connected"). INVERTED premise (C11, grep-confirmed
2026-06-03): the symbol-level CALL graph IS already on canvas in symbol mode (build_symbol_view views.rs:175 emits
VizSymbolEdge from outgoing_calls -> get_graph_view_data mode='symbol' -> GraphWindow.svelte); the producer-DARK
piece is PageRank (get_pagerank_top / SsbViewer::pagerank_top read a dkg_nodes_pagerank table NOTHING materialises)
and dkg_edges carries NO call edges (only contains/imports/implements/staleness). B0 (blocking) measures the real
worst-case DKG node+edge count and picks the renderer; then semantic zoom + LOD + the dead-code island read with the
dynamic-dispatch entrypoints map + the runtime coverage_count backstop (HD-5).

Estimated effort: ~2 sessions (~10-14 h). B0 is a measurement gate; the renderer is a dashboard build; the PageRank
materialiser is a backend build.
Pre-requisites: B-29 symbol-graph SHIPPED (traversal.rs, build_symbol_view); B-33 graph commands SHIPPED
  (get_graph_view_data/VizData commands.rs:2507); B-34 entry-points SHIPPED (detect_entry_points detect.rs:142);
  the canvas-store isolation gate PASSES (C12 — no store imports three.js). H-10 builds on Project-Intelligence/DKG.
Parallel-safe with: H-1..H-9 (disjoint crates). The first Wave-2 session; H-11/H-12 build on its shell.
Source: CANVAS-WORKBENCH-DESIGN.md §4 (codebase graph) + §13 C1; _grounding/H-GROUNDING-MAP.md C11 + C12 + the H-10 table + verify:canvas-deps. Doctrine: H-0_INDEX.md.

## Context

The cytoscape WebGL renderer (renderer:{name:'canvas',webgl:true,showFps:true}, requires cytoscape >=3.31, latest
3.34.0) is PROVISIONAL (preview, not GA — verify:canvas-deps) and gives ~100+fps at 1,200 nodes / 16,000 edges but
~10fps at 3,200 nodes / 68,000 edges — edge DENSITY, not node count, is the frame killer. So B0 measures the real
worst-case DKG (node AND edge count) and decides: under ~2,500 nodes at real density -> cytoscape WebGL + semantic-zoom
LOD; over ~2,500 -> sigma@3.0.3 from C1 (v4 is alpha-only = BLOCK; sigma has no native hierarchical layout, so feed it
dagre/elk coords). Semantic zoom changes WHAT is drawn per zoom tier (pixel-size gates, not zoom-percentage), keeping
the DRAWN element count bounded regardless of graph size (the 60fps lever). Dead code: a node with zero static edges
after fcose component-packing is an ORPHAN CANDIDATE, but the DKG cannot see dynamic dispatch (Tauri invoke() strings,
dyn Trait, the SSB bus), so topological orphan-ness is necessary-but-not-sufficient (HD-5, R5) — the entrypoints map
re-attaches dynamic-dispatch-only nodes before orphan detection, and the runtime coverage_count is the AUTHORITATIVE
backstop. Confirmed-dead (coverage-backed) renders distinct from topology-only-suspected (HD-5 — never launder a guess).

## Existing state (B/C/D/E/F/G coverage)
- build_symbol_view (knowledge/src/visualization/views.rs:175) — SHIPPED. Emits VizSymbolEdge from
  payload.outgoing_calls.callee_candidates -> get_graph_view_data mode='symbol' -> GraphWindow.svelte. THE call graph is on canvas (bounded MAX_VIZ_EDGES=60k).
- get_graph_view_data + _impl (dashboard-v2/src-tauri/src/commands.rs:2507/2531) — SHIPPED. Registered lib.rs:716;
  GraphWindow.svelte:87. Dispatches file/symbol/decomposition.
- compute_impact_radius (traversal.rs:149) / detect_entry_points (entry_points/detect.rs:142) — SHIPPED.
- get_pagerank_top (commands.rs:1613) + SsbViewer::pagerank_top (ssb_viewer.rs:2904) + run_symbol_graph_rank
  (symbol_graph.rs:115) — WIRE-DARK. Read dkg_nodes_pagerank, a table NOTHING materialises (RetrievalPipeline::compute_repo_map is in-memory-only).
- dkg_edges edge_type set (dkg/mod.rs:158-166) — only contains/imports/implements/staleness_quarantined; NO call edge.
  dkg_adapter query_callers/callees (dkg_adapter.rs:222) query edge_type IN ('CALLS','calls') -> structurally empty.
- VizFileEdge.cyclic hardcoded false (views.rs:103); VizFile.level/status None (views.rs:147-148) — WIRE-DARK attributes.
- MAX_VIZ caps (visualization/types.rs:33) — SHIPPED (20000 symbols / 60000 edges / 200 per file).
- The canvas-store isolation gate (lib/state/*.svelte.ts) — PASSES (C12; no store imports three.js); three.js confined to lib/scene/* + lib/orbit/node-mandala.ts.

## Reads (mandatory before coding)
- _grounding/H-GROUNDING-MAP.md C11 + C12 + the H-10 symbol table + verify:canvas-deps (the dep table + the CORRECTED sigma-v3 + provisional-WebGL rows).
- CANVAS-WORKBENCH-DESIGN.md §4 (the codebase graph: data source, semantic zoom/LOD, dead-code rule + the false-positive boundary, the perf stack, the verified-against-code corrections) + §10 (aesthetic) + §13 C1.
- F:\.ELAI_workflow\.elai_cc\crates\knowledge\src\visualization\views.rs:40-220 (build_file_view + build_symbol_view + the cyclic/level/status hardcodes)
- F:\.ELAI_workflow\.elai_cc\crates\dashboard-v2\src-tauri\src\commands.rs:1600-1620,2500-2680 (get_pagerank_top + get_graph_view_data + get_impact_overlay + export_graph_mermaid)
- F:\.ELAI_workflow\.elai_cc\crates\orchestrator\src\ssb_viewer.rs:2890-2910 (pagerank_top over the unpopulated table)
- F:\.ELAI_workflow\.elai_cc\crates\dashboard-v2\src\lib\layout\WindowHost.svelte (the window-type registration to extend)
- CLAUDE.md HARD RULES + HD-5 (substrate-honesty) + anti-slopsquatting (the dep BLOCKs: sigma@3.0.3 not v4; cytoscape-layout-utilities@1.1.1 stale-but-real).

## Tasks (WAVE 2 — B0 then the shell)

1. **B0 (BLOCKING; no code ships after a fail) (~1 session of measurement + a decision record).** (a) measure the real
   target ELAI project's worst-case DKG node AND edge count and pick cytoscape-WebGL (3.34) vs sigma@3.0.3 by the §4
   rule; (b) confirm the canvas-store isolation gate (grep every lib/state/*.svelte.ts for three/troika/scene/orbit-three
   imports — already PASSES per C12, re-confirm). Done-gate: the measured numbers + the renderer decision are recorded in
   H-10-decisions.md; the store grep is clean. Anti-slopsquatting: verify cytoscape@>=3.31 + cytoscape-layout-utilities@1.1.1
   (stale-but-real, register) + sigma@3.0.3 (NOT v4) before any package.json add.

2. **Canvas.svelte mounts the chosen renderer as a WindowHost window type (~320 LOC svelte).** Reads the symbol call
   graph (build_symbol_view feed) + the PageRank scores; pan/zoom + viewport culling keyed off cy.extent(); the WebGL
   renderer with showFps (provisional — graceful canvas fallback if the flag is dropped). Reachable from the real WindowHost tab bar.

3. **Materialise the producer-dark PageRank table (~120 LOC).** Build a persistent dkg_nodes_pagerank materialiser
   (today PageRank is in-memory-only via RetrievalPipeline::compute_repo_map; get_pagerank_top reads an empty table).
   Write the table on `elai symbol_graph build` so get_pagerank_top / the canvas PageRank panel are non-empty.

4. **Dead-code island + entrypoints map + coverage backstop (~160 LOC).** fcose component-packing (with
   cytoscape-layout-utilities REGISTERED — the packing is a silent no-op without it); the explicit dynamic-dispatch
   entrypoints map (Tauri command handlers, dyn Trait sites, SSB action-type consumers) injected as edges BEFORE orphan
   detection; is_dead_code + coverage_count node fields fed from the wiring-gate Layer-3 entrypoint coverage; the
   confirmed-dead (coverage-backed, --text-ghost + dashed) vs suspected-dead (topology-only, lighter amber-neutral ring) treatment (HD-5).

5. **Semantic zoom + LOD + the SCC cycle pass (~120 LOC).** Far tier (footprint <8px): folder super-nodes + bundled
   super-edges, no text. Mid tier (8-40px): per-file, labels above ~24px. Near tier (>40px): per-symbol card chrome.
   Labels/chrome hidden below their pixel threshold (the biggest 60fps lever). A net-new SCC pass over the import/call
   graph (VizFileEdge.cyclic is hardcoded false — back-edges do not stand out without it).

6. **Decision doc workflow/reasoning/H-10-decisions.md (~120 LOC).** Records: the B0 measured worst-case + the renderer
   decision (the scope-death-by-optimism kill); INVERTED premise (call graph on canvas, PageRank is the dark build); the
   dead-code false-positive boundary + the entrypoints-map + coverage backstop (HD-5); the dep BLOCKs (sigma v3, provisional WebGL).

## WIRING_CONTRACT
produces_symbols:
  - crates/dashboard-v2/src/lib/windows/Canvas.svelte (the renderer host + pan/zoom + culling + LOD)
  - crates/dashboard-v2/src/lib/canvas/entrypoints_map.ts (the dynamic-dispatch entrypoints injected before orphan detection)
  - crates/orchestrator/src/ssb_viewer.rs OR knowledge/src/... (the persistent dkg_nodes_pagerank materialiser)
  - crates/knowledge/src/visualization/views.rs (is_dead_code + coverage_count node fields + the SCC cycle pass replacing cyclic:false)
  - workflow/reasoning/H-10-decisions.md (the B0 measurement record + renderer decision)
consumes_symbols:
  - crates/knowledge/src/visualization/views.rs::build_symbol_view (the call graph already on canvas)
  - crates/dashboard-v2/src-tauri/src/commands.rs::get_graph_view_data (+ get_pagerank_top, get_impact_overlay)
  - crates/knowledge/src/symbol_graph/traversal.rs::compute_impact_radius
  - crates/knowledge/src/symbol_graph/entry_points/detect.rs::detect_entry_points
  - crates/dashboard-v2/src/lib/layout/WindowHost.svelte (the window-type registration)
reachable_from:
  - crates/dashboard-v2/src/lib/layout/WindowHost.svelte (new canvas window type) -> Canvas.svelte -> get_graph_view_data  [PRODUCTION dashboard, reachable from the tab bar]
  - `elai symbol_graph build` -> the dkg_nodes_pagerank materialiser -> get_pagerank_top non-empty  [PRODUCTION PageRank producer]
stub_ban:
  - NO code ships after a B0 FAIL (the measurement gates the renderer; scope-death-by-optimism)
  - NO sigma v4 (alpha-only = BLOCK); sigma@3.0.3 pinned
  - NO fcose component-packing WITHOUT cytoscape-layout-utilities registered (silent no-op otherwise)
  - NO topological orphan rendered as confirmed-dead without coverage backing (HD-5 — suspected vs confirmed distinct)
  - NO labels/chrome painted below the pixel threshold (the 60fps lever)
  - NO new dep from training memory (anti-slopsquatting: cytoscape/sigma/layout-utilities verified)
frontend_pair:
  Canvas.svelte (svelte) + get_graph_view_data/get_pagerank_top (tauri, existing). cross_lang_diff PASS; W-6 dashboard E2E.
observable_effect:
  trigger: "open the dashboard on a real project, switch to the Canvas window (NO env override); run `elai symbol_graph build`"
  expect_ssb: "the dkg_nodes_pagerank table is materialised (get_pagerank_top returns non-empty); coverage_count is fed from the wiring-gate Layer-3 entrypoint coverage"
  expect_artifact: "the codebase renders on canvas at the measured >=60fps for the B0 worst-case graph (the showFps HUD
    is the off-agent artifact, NOT the agent stdout); a known true orphan islands apart AND a known Tauri-invoked-only
    function does NOT render dead (the entrypoints map attached it); confirmed-dead vs suspected-dead are visually distinct"

## DETERMINISTIC_CHECK
```bash
python .claude/tools/wiring_contract_check.py --bn-file docs/thefinalglance/H-plan/H-10_canvas_shell_code_graph.md --run-trigger
test -f .elai_cc/crates/dashboard-v2/src/lib/windows/Canvas.svelte
rg "webgl: true|showFps|sigma" .elai_cc/crates/dashboard-v2/src/lib/windows/Canvas.svelte   # the chosen renderer
rg "cytoscape-layout-utilities|layoutUtilities" .elai_cc/crates/dashboard-v2/package.json .elai_cc/crates/dashboard-v2/src  # registered (not a silent no-op)
rg "dkg_nodes_pagerank" .elai_cc/crates/knowledge/src .elai_cc/crates/orchestrator/src  # a materialiser now WRITES the table
rg "coverage_count|is_dead_code" .elai_cc/crates/knowledge/src/visualization/views.rs   # the runtime backstop fields exist
grep -rE "from .*scene/three|troika" .elai_cc/crates/dashboard-v2/src/lib/state/        # MUST be empty (C1.B0 gate)
test -f workflow/reasoning/H-10-decisions.md
```

## ACCEPTANCE
- [ ] B0 done-gate: the measured worst-case DKG node+edge count + the cytoscape-vs-sigma decision are recorded; the store-import grep is clean. Nothing renders until this passes.
- [ ] W-1 done-gate: --run-trigger PASS (the canvas renders at >=60fps for the B0 worst-case; the orphan + entrypoints-map behaviors hold; PageRank non-empty).
- [ ] W-3 production entrypoint: the canvas window is reachable from the real WindowHost tab bar; the PageRank materialiser fires from `elai symbol_graph build`.
- [ ] HD-5: confirmed-dead (coverage-backed) renders distinct from topology-only-suspected; a Tauri-invoked-only fn is not a false orphan.
- [ ] anti-slopsquatting: sigma@3.0.3 (not v4); cytoscape-layout-utilities registered; the WebGL renderer has a graceful canvas fallback (provisional).
- [ ] W-4: cross_lang_diff PASS; W-6 dashboard E2E (showFps + the dead-code treatments) via dashboard-smoker.
- [ ] cargo check --workspace GREEN + dashboard svelte-check; auditor PASS.

## WIRING DONE-GATE (clause by clause)
- W-1: --run-trigger is ACCEPTANCE; the canvas (produced) consumes get_graph_view_data (W-8) reachable from the tab bar (W-3); the showFps HUD is the off-agent artifact.
- W-2: the canvas + PageRank materialiser are default behaviors (the canvas is a window, the materialiser runs on build).
- W-3: WindowHost tab bar + `elai symbol_graph build`; not test.
- W-4: Canvas.svelte + the existing tauri commands; cross_lang_diff PASS.
- W-5: a runnable dashboard open + a symbol_graph build -> a real >=60fps render + a materialised table; the showFps HUD is the artifact.
- W-6: the dashboard E2E (render + dead-code treatments) via dashboard-smoker.
- W-7: build_symbol_view / get_graph_view_data / get_pagerank_top / detect_entry_points / the cyclic/level/status hardcodes grep-confirmed 2026-06-03 (C11).
- W-8: Canvas.svelte (produced) consumes get_graph_view_data; the materialiser (produced) feeds get_pagerank_top.
- W-9: NO new SSB action_type (the PageRank table is a producer build).
- W-10: the render is in-process; the B0 measurement is a one-time gate, not a per-run network dep.
- HD-5: coverage_count is the authoritative backstop; the entrypoints map prevents dynamic-dispatch false orphans.

## Tests to add
- crates/contracts/tests/h10_code_graph_contract.rs (~190 LOC): (a) the dkg_nodes_pagerank materialiser writes a
  non-empty table on build and get_pagerank_top returns it; (b) a known true orphan (no static edge, coverage_count==0)
  is islanded + rendered confirmed-dead; (c) a known Tauri-invoked-only fn (zero static edges, coverage_count>0) is
  attached by the entrypoints map and NOT rendered dead; (d) a topology-only orphan with no coverage data renders
  suspected-dead (distinct treatment), not confirmed; (e) the SCC pass marks a real import cycle (cyclic no longer hardcoded false).
- a dashboard-smoker fixture asserting showFps >= the 60fps budget on the B0 worst-case graph + the cytoscape WebGL config object retains webgl:true.

## Files touched
- CREATE: crates/dashboard-v2/src/lib/windows/Canvas.svelte (~320 LOC)
- CREATE: crates/dashboard-v2/src/lib/canvas/entrypoints_map.ts (~120 LOC)
- MODIFY: crates/dashboard-v2/src/lib/layout/WindowHost.svelte (~10 LOC: register the canvas window type)
- MODIFY: crates/dashboard-v2/package.json (~3 LOC: cytoscape >=3.31 [+ sigma@3.0.3 if B0 selects it] + cytoscape-layout-utilities@1.1.1, anti-slopsquatting-verified)
- MODIFY: crates/knowledge/src/visualization/views.rs (~80 LOC: is_dead_code + coverage_count + the SCC cycle pass)
- MODIFY/CREATE: the dkg_nodes_pagerank materialiser (~120 LOC, knowledge or orchestrator)
- CREATE: crates/contracts/tests/h10_code_graph_contract.rs (~190 LOC)
- CREATE: workflow/reasoning/H-10-decisions.md (~120 LOC)

## Citations / Grounding
- CANVAS-WORKBENCH-DESIGN.md §4: cytoscape WebGL budget (blog.js.cytoscape.org/2025/01/13); semantic zoom (GraphMaps
  arXiv:1906.05996); the dead-code false-positive boundary (R5, the entrypoints map + coverage backstop); §13 C1.
- verify:canvas-deps (2026-06-03): cytoscape 3.34.0 MIT (WebGL renderer provisional); cytoscape-layout-utilities 1.1.1
  MIT (stale 2021 but real); sigma 3.0.3 MIT (v4 alpha-only = BLOCK).
- ELAI ground truth (C11/C12): build_symbol_view views.rs:175 (the call graph on canvas); get_pagerank_top commands.rs:1613
  (empty table); dkg_edges no call edge (dkg/mod.rs:158-166); cyclic:false views.rs:103; the store-isolation gate PASSES.
- CLAUDE.md HARD RULES + HD-5 + anti-slopsquatting.

## Open questions / risks
- Scope death: this is the biggest risk across the whole canvas track. C1 (H-10) alone is a complete shippable win
  (the codebase graph the user wanted first); H-11..H-16 are additive. Do not start H-13/H-14 before H-10..H-12 are default.
- B0 is project-dependent: re-run it when ELAI is pointed at a materially larger codebase than the one measured (the renderer decision is data-bound, not permanent).
- The dead-code false-positive RATE for ELAI's specific dispatch patterns is a number to MEASURE on the real codebase and surface (HD-5), not assume small.
- coverage_count depends on the wiring-gate Layer-3 entrypoint coverage existing; if that leg is not yet producing coverage, the dead-code overlay falls back to suspected-only (never confirmed) — honest degradation, not a false confirm.

## Next
H-11 (live layer + file explorer) adds the CanvasActivityEvent overlay + the fs surface on top of the H-10 shell.
H-12 ports PLAN/RESEARCH modes + makes Canvas the default. H-16 retires the orbit only after Canvas-as-default ships a full release.

End of H-10.
