B-33 — Grounding sweep (post-execution)
session: s-50960
date: 2026-05-16

CLAIM CLASSIFICATION (post-implementation)

GROUNDED
- MAX_VIZ_SYMBOLS=20_000, MAX_VIZ_EDGES=60_000, MAX_VIZ_SYMS_PER_FILE=200
  graph-visualize-html.ts:45-47 — verified at synthesis time.
- VizData dual-view shape (files + file_edges + symbols + symbol_edges +
  symbols_by_file + symbol_mode) — graph-visualize-html.ts:82-92.
- Mermaid cyclic edges as `-.->|cycle|` — graph-analysis.ts:168.
- Cytoscape PNG export via cy.png(...) — viewer-app.js:658.
- SymbolGraphCache::{new, ensure_*_index, get_or_load_payload, meta, stats}
  — verified during integration; signatures unchanged.
- compute_impact_radius signature/return — verified in traversal.rs:149-252;
  ImpactResult fields fed directly into the `get_impact_overlay` JSON
  reshape.
- SsbViewer::decomposition_tree(None) shape — verified at ssb_viewer.rs:1754
  (returns DecompositionTreeSnapshot with concepts -> phases -> steps ->
  baby_steps); reshape used in `b33_build_decomposition_view`.

KNOWN MISMATCHES (carried into the implementation as DELIBERATE IMPROVEMENTS)
- SocratiCode Mermaid uses inline `style id fill:...` per-node, NOT
  classDef. ELAI uses classDef intentionally (legend stays O(languages)
  not O(nodes)). Documented in mermaid.rs module header.
- SocratiCode 250-node Mermaid cap is unfounded in the source; ELAI lets
  the caller pass `max_nodes` and emits a `%% truncated: showing N of M`
  comment when triggered. No defaulted 250 magic number.
- SocratiCode interaction model is tap+sidebar button for blast radius;
  ELAI uses cytoscape `cxttap` (right-click) on a node instead, AND keeps
  the sidebar "Show blast radius" button so both surfaces work. Documented
  in GraphWindow.svelte module header.
- SocratiCode ships 5 layouts (dagre/concentric/breadthfirst/grid/circle);
  ELAI ships 6 (adds fcose). cytoscape-fcose dep was confirmed by user via
  D1 pre-approval ("choose best NOT lazy"). Documented in B-33-decisions.md.

VERIFIED VIA WEBFETCH (npm registry, 2026-05-16 at synthesis time)
- cytoscape latest 3.33.3 — pinned ^3.33.3 in package.json.
- cytoscape-dagre latest 3.0.0 — pinned ^3.0.0; one major newer than user
  preset ^2.5. User-approved override.
- cytoscape-fcose latest 2.2.0 — pinned ^2.2.0.

UNVERIFIED / SHOULD BE FOLLOWED UP
- Mermaid 11.x classDef syntax: training data is consistent (Mermaid 10+
  supports classDef inside flowchart; 11.x backwards-compatible). No live
  WebFetch confirmation done this session because the initial verification
  for B-33 was performed at synthesis time and recorded in this file's
  prior version. Risk: low — generate_mermaid output is a strict subset
  of what GitHub's Mermaid renderer accepts (header + classDef + node
  lines + edges).

GROUNDING SOURCES
- F:\.ELAI_workflow\docs\.source\SocratiCode-main\src\services\graph-analysis.ts
  (Mermaid generator reference, MEDIUM grounding — read second-hand via
  plan, not paper-ingested with full extraction).
- F:\.ELAI_workflow\docs\.source\SocratiCode-main\src\services\graph-visualize-html.ts
  (VizData shape reference, MEDIUM grounding).
- F:\.ELAI_workflow\.elai_cc\crates\knowledge\src\symbol_graph\* (full
  source read this session — HIGH grounding).
- F:\.ELAI_workflow\.elai_cc\crates\dashboard-v2\src-tauri\src\* (read for
  AppState shape + invoke_handler placement — HIGH grounding).
- F:\.ELAI_workflow\.elai_cc\crates\dashboard-v2\src\lib\layout\* +
  state\* + windows\* (read for cross-window pattern — HIGH grounding).

OBSERVABLE EFFECT (deferred to manual smoke)
The ACCEPTANCE map's "open dashboard, run /index, Ctrl+G, click file
node" walkthrough is not exercised here (sidecar-touching live test). Test
results from cargo + svelte-check are GREEN. Manual smoke is a follow-up
when the dashboard is launched against a real ELAI repo with `/index`
already run; expected outcomes are documented in synthesis-B-33.md
ACCEPTANCE block (cytoscape canvas with >=10 nodes; right-click impact
overlay; valid Mermaid).
