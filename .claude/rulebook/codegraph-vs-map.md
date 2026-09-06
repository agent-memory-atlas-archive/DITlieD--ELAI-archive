codegraph vs elai.map -- split of duties + the reconciler (load when locating symbols / tracing call chains / pre-edit impact)

Two code-graphs cover this repo. They are complementary along a live-syntactic vs static-semantic axis, NOT redundant. Use the right one per intent; use the reconciler when the answer must be trusted.

codegraph  (MCP server codegraph-elai, scoped to .elai_cc; CLI `codegraph` on PATH)
  WHAT: tree-sitter -> SQLite call-graph + FTS5, file-watched (ReadDirectoryChangesW), incremental, never stale. 57k nodes / 192k edges over .elai_cc (rust 1765, python 212, svelte 199, ts 69 files). Recognizes axum/actix/Rocket, FastAPI/Flask/Django, SvelteKit route -> handler edges.
  EDGES ARE HEURISTIC: tree-sitter name-match. It can surface a dynamic-dispatch / trait-object hop SCIP misses, but can also emit a same-name false positive, and `codegraph_callers <name>` disambiguates an overloaded name to ONE node (so it under-reports refs for ambiguous names).
  USE FOR: live "how does X work" / "how does X reach Y" exploration, call-flow walking (callers/callees), pre-edit blast radius, fast symbol search, and `codegraph affected <changed-files>` -> the test files those changes reach (feeds scoped-test selection). Low-token: one codegraph_explore replaces a grep/read sweep.
  MCP tools (auto-allowed, live after a Claude restart): codegraph_explore (primary), codegraph_search, codegraph_callers, codegraph_callees, codegraph_impact, codegraph_node, codegraph_files, codegraph_status.
  Regen: none needed (auto-syncs). Full re-index if ever required: `codegraph index "/workspace/ELAI\.elai_cc" --force`.

elai.map  (.claude/tools/elai.map, 166MB TSV; twin elai_graph.json)
  WHAT: static, regenerated on demand by .claude/tools/codebase_mapper.py. 6-col TSV TAG<TAB>SYMBOL<TAB>FILE:LINE<TAB>LANG<TAB>EXTRA<TAB>SNIPPET, sorted by symbol (grep returns all rows for a symbol contiguously).
  STRENGTH: SCIP_REF / SCIP_DEF rows are type-accurate canonical references resolved by rust-analyzer / scip-python / scip-typescript -- the semantic ground truth codegraph's heuristics cannot match. Plus ELAI-specific tags (IMPL_TRAIT) and the whole harness built on it (impact_radius.py, wiring_contract_check.py, cross_lang_diff.py, canvas).
  WEAKNESS: goes stale between regens (check with .claude/tools/elai_map_freshness.py; regen per the CLAUDE.md command). 166MB -- grep it, never read it.
  USE FOR: SCIP-resolved canonical refs ("where is this symbol REALLY used", type-accurate), ELAI wiring tags, and any harness tool that already consumes the TSV/JSON.

graph_reconcile.py  (.claude/tools/graph_reconcile.py) -- the merge, "better than either alone"
  Unions BOTH graphs for a symbol and tags every referencing file by confidence:
    HIGH       in codegraph (live) AND elai.map SCIP (resolved) -- trust it.
    LIVE_ONLY  codegraph sees it, SCIP does not -- dynamic-dispatch hop? stale map? heuristic FP? investigate.
    SCIP_ONLY  SCIP resolved it, codegraph did not surface it -- map drift the other way? ambiguous-name disambiguation? scope? investigate.
    MAP_SYN    only a syntactic CALL row in elai.map, weakest signal.
  Test-file refs (tests/, contracts/tests/, *_test.*, *.spec.*) are tagged TEST and reported apart from production refs, per wiring-enforcement.md R7 (a test caller does not satisfy production reachability).
  Modes: callers (default), impact, affected. Examples:
    python .claude/tools/graph_reconcile.py CloudModeRouter --mode callers
    python .claude/tools/graph_reconcile.py route --mode impact --depth 2
    python .claude/tools/graph_reconcile.py crates/orchestrator/src/cloud_mode.rs --mode affected
  --json for machine output. Fail-soft: if either leg is unavailable it says so and reports the other.
  NOTE: `affected` is import-graph based -- strongest on TS/JS/Python; Rust integration-test reachability via contracts/ may not be import-linked, so a Rust file can legitimately return no affected tests.

ROUTING RULE OF THUMB
  - exploring / understanding a flow, fast and cheap                 -> codegraph_explore (MCP)
  - walking callers/callees live, never-stale                        -> codegraph_callers / _callees
  - pre-edit blast radius, quick                                     -> codegraph_impact, or impact_radius.py
  - canonical/semantic "real" references, type-accurate              -> elai.map SCIP_REF grep
  - ELAI wiring tags / cross-lang pairing / contract checks          -> elai.map + the harness tools
  - a verdict you must TRUST (wiring done-gate, risky refactor)      -> graph_reconcile.py (both, disagreements flagged)
  - which tests a change reaches (scoped-test selection)             -> graph_reconcile.py --mode affected
