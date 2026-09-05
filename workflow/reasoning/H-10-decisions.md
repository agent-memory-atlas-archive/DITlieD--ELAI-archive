H-10 — Canvas shell + CODE-GRAPH (read-only) — DECISIONS (session s-10612, 2026-06-04)

B0 — WORST-CASE DKG MEASUREMENT (the blocking gate; authoritative, from the real index)
Source: F:\.ELAI_workflow\.elai_cc\indexes\dkg.db (159.67 MB, the live ELAI self-index — the
largest realistic project). Measured via workflow/sessions/s-10612/b0_measure.py:
  - dkg_nodes            = 398,741   (~399K symbols)
  - dkg_edges            = 172,868   (ALL edge_type = 'contains'; ZERO call edges in the snapshot —
                                      confirms CORRECTION C11: dkg_edges carries no call edges)
  - dkg_nodes_pagerank   = TABLE ABSENT (confirms the PageRank producer is dark — H-10 materialises it)
Implication (CANVAS-WORKBENCH-DESIGN §4 renderer rule): ~399K nodes is FAR beyond any canvas/SVG
renderer. WebGL is MANDATORY, and even WebGL must render a BOUNDED subgraph via semantic-zoom + LOD
(top-N by PageRank / the focused-symbol neighborhood), never all 399K at once. The total is the
worst-case ceiling; the rendered set is LOD-bounded.

RENDERER DECISION — sigma@3.0.3 + graphology@0.26.0 (both MIT). cytoscape REJECTED.
Grounded by spec-grounder (workflow/sessions/s-10612/reasoning/H-10-deps.md, anti-slopsquat 2026-06-04):
  - sigma@3.0.3       : latest stable (v4 is 4.0.0-alpha.6 = alpha-only, BLOCK). MIT. WebGL-FIRST core
                        (designed for scale; handles the LOD-bounded subgraph of a 399K-node graph).
                        Co-install graphology@0.26.0 (MIT, ESM/Vite-safe) + graphology-utils (sigma hard-dep).
  - cytoscape@3.34.0  : REJECTED. (1) LICENSE = LGPL-3.0+, a real legal hazard for ELAI's CLOSED-SOURCE
                        engine (GTM: engine stays closed) — sigma's MIT avoids it. (2) Its WebGL renderer
                        is still PROVISIONAL (canvas sprite-sheet addon), weaker than sigma's core WebGL.
  - cytoscape-layout-utilities@1.1.1 : DROPPED. 57 months stale (2021), no maintained successor = BLOCK.
DEVIATION FROM PLAN: H-0_INDEX framed it as "cytoscape-WebGL (provisional) vs sigma@3.0.3, pick by §4".
The 2026-06-04 license + B0 findings make sigma the unambiguous pick on EVERY axis (license + WebGL
maturity + scale). User authorized "add deps as needed, best/newest or mature"; sigma is both, MIT-clean.

H-10 BUILD SCOPE (grounded; B0 now satisfied, build unblocked)
1. deps: add sigma@3.0.3 + graphology@0.26.0 + graphology-utils to crates/dashboard-v2/package.json
   (drop cytoscape + cytoscape-layout-utilities from the plan). pnpm/npm install + verify lockfile.
2. Canvas.svelte (NEW): a sigma/WebGL host as a new WindowHost window type, rendering the EXISTING
   call-graph feed via invoke('get_graph_view_data', {mode:'symbol'}) (build_symbol_view — GD-1 reuse,
   do NOT re-implement); semantic-zoom + LOD bounding the rendered subgraph; showFps HUD (off-agent
   artifact for the Tier-2 visual gate).
3. WindowType 4-SITE invariant (silent FallbackWindow if any missed — gotchas.md:2147): add 'canvas' to
   types.ts:67 (WindowType union), workspace.svelte.ts:9, Panel.svelte (SCOPE_LABEL), WindowHost.svelte
   (import Canvas + the {:else if type === 'canvas'} arm before the FallbackWindow {:else}).
4. PageRank materialiser (backend, producer-dark -> live): persist dkg_nodes_pagerank after
   DkgEngine::generate_cross_edges (dkg/mod.rs:461), computed via compute_repo_map (pagerank/mod.rs:352,
   currently in-memory-per-query). SsbViewer::pagerank_top (ssb_viewer.rs:2904) already reads it
   gracefully when absent — wire the panel to the now-materialised table.
5. Dead-code island + coverage backstop (HD-5): combine detect_entry_points (detect.rs:142,
   topology-suspected-dead) + CoveredFunction.count (wiring_coverage.rs:158, coverage-confirmed-dead)
   into ONE dashboard-visible annotation; entrypoints-map re-attaches dynamic-dispatch-only nodes
   BEFORE orphan detection (so a Tauri-invoked-only fn does NOT render dead). Topology-suspected must
   render VISUALLY DISTINCT from coverage-confirmed-dead.
6. NO new SSB action_type (W-9): H-10 is read-only CODE-GRAPH; PageRank is a producer-dark TABLE, not
   an SSB variant. SCHEMA stays 219/496.

TIER BOUNDARY (cloud-only / defer-local override): the HD-4/HD-5 VISUAL leg (open project -> render at
>=60fps for the 399K worst-case via the showFps HUD; orphan islands observed; pagerank panel non-empty)
needs a LIVE Tauri dashboard render = a local UI op = DEFERRED (Tier-2) per the no-local-tests override.
Tier-1 done-gate = svelte-check + cargo check (dashboard-smoker) + the WindowType 4-site wiring + the
PageRank materialiser unit test + the dead-code annotation logic test + cross_lang_diff. The live render
is a separately-gated Tier-2 checkbox.

FIXER CYCLE (s-10612, 2026-06-04) — 3 BLOCKs resolved:

D2 / WARN-1 coverage_count=None Tier-1 boundary:
coverage_count is intentionally None in this run. The topology-suspected-dead visual leg ships in Tier-1;
the coverage-confirmed-dead data channel (wiring_coverage.rs -> VizFile.coverage_count) is deferred to H-11+.
This is an honest Tier-1 state: the canvas renders suspected nodes with an amber ring (correct branch present)
but no node will ever be coverage-confirmed-dead until H-11 wires the channel. Tier-1 is topology-only. The
branch at CanvasWindow.svelte:153 (coverage_count === 0) is structurally present; data-dark until H-11.

BLOCK-1 PageRank resolution (GD-2 fix):
Chose honest-degrade over uniform-score fabrication. When n > MAX_PAGERANK_NODES (398K symbols > 50K cap),
materialise_pagerank() now returns Ok(()) immediately after creating an empty table, rather than writing
uniform scores that carry no ranking signal. The canvas receives known:false from get_pagerank_top (empty
table), sets pageRankKnown=false, and renders with no COLOR_HUB applied — honest degradation. A tracing::warn
surfaces the degradation state. The 50K cap has an expanded doc comment explaining the GD-2 rationale.
File-level PageRank (tractable at repo scale) is deferred to H-11+ when the file-level graph projection
is available.

WARN-2 saturation guard:
Added total_import_edges guard in build_file_view (views.rs). When the DKG has zero import edges (as the
live ELAI dkg.db does — all 172K edges are 'contains' only), is_dead_code is suppressed to None for all
files, and a tracing::warn is emitted. topology-suspected-dead annotation only fires when there is at least
one import edge to reason over, preventing the amber-ring storm on the real ELAI self-index.
