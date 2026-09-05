# DEEP-H-005 exhaustive feature discovery

## Scope, method, and evidence classes

- Processed exactly the `DEEP-H-005` manifest chunk: 9/9 files, 1,451/1,451 logical lines, 149,987/149,987 bytes. No numbered/index original outside this chunk was read.
- Orientation-only input: `08-H-SUMMARY_INDEX.discovery.md`. It is used only for the delta ledger; it is not treated as original evidence.
- Every source was read end-to-end with physical line numbering. Repeated requirements appearing in objective, task, wiring, acceptance, and test sections are collapsed into one atom with every material anchor retained.
- `E-*` = explicit source statement. `A-*` = explicit ambiguity/open choice. `C-*` = internal or cross-source conflict. `I-*` = bounded inference from two or more explicit atoms; inferred rows never silently upgrade into requirements.
- Domain tags: `CAP` capability/action/surface; `STATE` present/absent/planned state; `FLOW` workflow/state transition; `DATA` schema/data/lifecycle; `CFG` flag/default/config; `INT` integration/dependency/event; `SEC` security/privacy/compliance; `FAIL` failure/fallback/recovery; `DET` determinism/idempotency/concurrency; `OBS` observability/evidence; `TEST` acceptance/done-gate; `LIMIT` constraint/non-goal; `DEFER` deferred/status.
- Atomic IDs are unique within this document. Summary deltas and interactions point back to these atoms and do not create duplicate feature atoms.

## Exact processed manifest and verification

Manifest anchor: `DEEP-SOURCE-MANIFEST.md:726-740`. Family `H`; target status `under 1,800`; manifest totals 9 files / 1,451 lines / 149,987 bytes.

| File | Manifest lines | Actual lines | Manifest bytes | Actual bytes | SHA-256 (manifest = actual) |
|---|---:|---:|---:|---:|---|
| `docs/thefinalglance/H-plan/H-4_tool_search_beta_header.md` | 183 | 183 | 15,181 | 15,181 | `6631f0ce3c130430c00ce6a22433f1ec6ceccdde97a42843d8e84f51f4a4bc45` |
| `docs/thefinalglance/H-plan/H-10_canvas_shell_code_graph.md` | 196 | 196 | 18,067 | 18,067 | `fae2d1681c50081ec150f7e0725eb3f94750dcbd797f52909e8f83583bfb2e56` |
| `docs/thefinalglance/H-plan/H-21_runtime_lookahead_differential.md` | 157 | 157 | 12,529 | 12,529 | `c68d739a8f6915a3cd3c8c6cb1fa03deb63fc2c525f03ee09d9f5f26a8a03624` |
| `docs/thefinalglance/H-plan/H-25_result_manifest_rederivation.md` | 170 | 170 | 13,988 | 13,988 | `91839bc23fac6a22efe287e9a74e69d414dd1af30c389dcf71e48be246ba7e5d` |
| `docs/thefinalglance/H-plan/H-31_user_decision_bitemporal_ledger.md` | 100 | 100 | 21,375 | 21,375 | `567916ee25ae528058dd394f4a0896d5a56022385d9e6d936fc0a3e54ce1c300` |
| `docs/thefinalglance/H-plan/H-44_behavioral_trap_detection_recall.md` | 187 | 187 | 16,346 | 16,346 | `6a11647451537b53e832889dd9b30ad0616d36e0bac449029c4d7f3670c0ded7` |
| `docs/thefinalglance/H-plan/H-47_dgm_eval_firewall_dategate_templateregen.md` | 222 | 222 | 19,667 | 19,667 | `dd78d5f76058527483d81c9d574bced024b374d3a2d1c985fcc1fab86c259b2c` |
| `docs/thefinalglance/H-plan/H-48_wire_bandit_on_real_reward.md` | 177 | 177 | 15,522 | 15,522 | `55976febf487ec58ebe6ff6c959fed04702e68780689c9861f1381eec57eebaa` |
| `docs/thefinalglance/H-plan/H-52_deterministic_finding_anchoring_cockpit.md` | 59 | 59 | 17,312 | 17,312 | `f71167566b0e3ea92643000783d6f06c7fb54e17f7e00d216ba7b5849cbbd6f2` |
| **Total** | **1,451** | **1,451** | **149,987** | **149,987** | **9/9 exact** |

## Per-file exhaustive ledgers

### H-4 — Tool Search Tool + advanced-tool-use beta header

Source: `H-4_tool_search_beta_header.md` (183 lines). Status: planned, Wave 1, in-process build.

| Atomic ID | Class/domain | Exact source anchor(s) | Explicit feature or behavior |
|---|---|---|---|
| H005-04-001 | E-STATUS/STATE | L3 | Work is `PLANNED (WAVE 1; in-process; BUILD)` and estimated as one 5–7 hour session. |
| H005-04-002 | E-STATE | L5-L12 | The live request sends no advanced-tool-use header, includes a 16-tool catalog every request, leaves four `dkg_*` tools catalog-only, and has a stale `len==14` contract. |
| H005-04-003 | E-STATE/INT | L7-L10, L41-L45 | OAuth bearer requests emit only `oauth-2025-04-20`; configured/default profile betas reach only the API-key branch, making normal `with_beta` registration dead on the production OAuth path. |
| H005-04-004 | E-CAP/CFG | L59-L62 | Join `advanced-tool-use-2025-11-20` with `OAUTH_BETA_HEADER` on the OAuth branch and register it in `AnthropicRequestProfile` defaults so both OAuth and API-key paths carry it. |
| H005-04-005 | E-DATA/CFG | L41-L44 | Existing profile defaults are `claude-code-20250219` and `prompt-caching-scope-2026-01-05`; `header_pairs` joins them. |
| H005-04-006 | E-CAP/DATA | L23-L25, L64-L68 | Add always-present `tool_search_tool_regex_20251119`; bulk tool definitions gain `defer_loading:true` only above a catalog-size threshold. |
| H005-04-007 | E-CFG/FAIL | L64-L68, L97, L124-L125 | Below the size gate, keep the small catalog inline; the search tool itself must never be deferred; behavior is default-on above the gate with no terminal default-off enable flag. |
| H005-04-008 | E-CAP/LIMIT | L23-L25, L162-L164 | Verified Tool Search protocol also names `tool_search_tool_bm25_20251119`; regex patterns are capped at 200 characters, service scale is up to 10,000 tools, and typical search returns 3–5 definitions. |
| H005-04-009 | E-FAIL/INT | L27-L32, L66-L68, L98 | A custom/non-Anthropic `ANTHROPIC_BASE_URL` disables server-side expansion, so ELAI must route to its own ToolSearch emulation and may not silently rely on Anthropic deferral. |
| H005-04-010 | E-FAIL/OBS | L172-L174 | A gateway that drops `tool_search_tool_result`/`tool_reference` expansion can return HTTP 400 `Tool reference not found`; compatibility must be verified. |
| H005-04-011 | E-CAP/INT | L35-L40, L70-L72 | Repair `dkg_callers`, `dkg_dependents`, `dkg_data_flow`, and `dkg_subgraph` with all three coordinated arms: catalog, JSON-to-postcard conversion, and dispatch. |
| H005-04-012 | E-DATA/INT | L70-L72 | Each repaired DKG tool gets a `TOOL_<X>` constant and `<X>Request` in `elai_ipc_types`, with fields matching `input_schema` verbatim. |
| H005-04-013 | E-FAIL/LIMIT | L95-L100, L126 | A tool with fewer than the three arms is forbidden; the implementation must mirror the shipped E-40 `symbol_dsl_query`/`symbol_dataflow_impact` pattern rather than create a second dispatcher. |
| H005-04-014 | E-STATE | L35-L45 | `build_tool_catalog` is shipped with 16 entries; conversion and dispatch cover 12 catalog tools; six artifact dispatch tools are agent-invisible because they are absent from the catalog. |
| H005-04-015 | E-DATA/TEST | L74-L75, L118, L127 | Update `production_agent_rpc_tool_catalog_contract.rs` away from stale `len==14` so it matches the post-change live catalog count. |
| H005-04-016 | E-OBS/LIMIT | L5-L7, L26, L128, L160-L161 | The corrected external reduction is about 77K→8.7K (85%), not 72K; the 79.5%→88.1% accuracy and token figures are literature evidence, not ELAI measurements. |
| H005-04-017 | E-OBS/INT | L14-L15, L101-L102, L106-L107, L128, L138, L142 | H-9, not a new H-4 UI/action, measures and surfaces per-request tool-definition tokens; live reduction is not a gate-time network dependency. |
| H005-04-018 | E-DET/TEST | L112-L120 | Deterministic checks run `wiring_contract_check.py --run-trigger`, grep both provider files for the beta, grep production RPC for search/defer and DKG arms, validate catalog count, and require `H-4-decisions.md`. |
| H005-04-019 | E-OBS/TEST | L103-L110, L122-L130 | A real OAuth inference with full catalog plus a DKG call must show the beta on the wire, on-demand definition loading/token reduction, successful DKG dispatch, and inline behavior for a tiny catalog. |
| H005-04-020 | E-TEST | L145-L148 | Contract tests cover OAuth header presence, bulk deferral/search non-deferral, small-catalog inline behavior, all four DKG round trips, and live catalog-count agreement. |
| H005-04-021 | E-TEST | L122-L130 | Completion requires run-trigger, live/default path, reuse of three-arm precedent, measured H-9 reduction, workspace build, targeted auditor pass, and no new dependency; cargo-green alone is insufficient. |
| H005-04-022 | E-OBS/DATA | L141-L142 | H-4 adds no SSB `action_type`; static header/catalog assertions are Tier 1 while H-9 supplies the live token metric. |
| H005-04-023 | E-INT | L82-L94 | Produced surfaces are the OAuth beta join, search entry/gate, and four conversion/dispatch arms; consumers are `build_tool_catalog`, `convert_json_input_to_postcard`, `translate_tool_request`, `AnthropicClient::with_beta`, and `AnthropicRequestProfile`. |
| H005-04-024 | E-FLOW/INT | L92-L94, L133-L140 | Production reachability is `handle_infer_request -> build_tool_catalog -> request tools` and the OAuth header emit at `anthropic.rs:695-701`; produced symbols must be consumed on those paths. |
| H005-04-025 | E-DATA/OBS | L77-L79 | `workflow/reasoning/H-4-decisions.md` records the OAuth blocker, corrected 77K figure, custom-base-URL behavior, size gate, DKG repair, and contract drift. |
| H005-04-026 | E-LIMIT/SEC | L95-L100, L130 | Use verified protocol strings, blake3, and no production `.unwrap()`/`.expect()`; no package is introduced merely for beta identifiers. |
| H005-04-027 | E-STATE/DEP | L16-L19 | Prerequisites are shipped B-37/E-18 catalog and E-40 three-arm precedent; H-1 shares the provider header edit and requires coordination. |
| H005-04-028 | E-CFG/OBS | L169-L171 | The catalog threshold is a tuning parameter: start conservatively and tune from H-9 token measurements because too low adds a retrieval hop and too high retains prefix bloat. |
| H005-04-029 | E-FAIL/INT | L175-L176 | Tool-list changes can invalidate the tools→system→messages cache; coordinate with H-2 so deferral stabilizes tool-definition bytes. |
| H005-04-030 | E-DEP | L178-L181 | H-9 consumes token counts, H-2 benefits from a smaller stable prefix, and H-13 must reuse the same three-arm pattern for future `canvas_*` tools. |
| H005-04-031 | E-DATA/INT | L150-L157 | Implementation inventory modifies Anthropic provider/profile, production RPC, IPC types, and catalog contract; creates the H-4 contract test and decision record. |
| H005-04-032 | E-LIMIT | L14-L15, L164 | Gate-time work is in-process/no-network; Tool Search supports batches at standard pricing, but live reduction is measured separately. |
| H005-04-033 | A-CFG | L49, L59-L62, L162-L164 | The source verifies a second scoped beta `tool-search-tool-2025-10-19`, but executable tasks and acceptance explicitly require only `advanced-tool-use-2025-11-20`; whether both must be emitted is not settled. |
| H005-04-034 | A-DATA | L74-L75, L145-L148 | The final catalog count is intentionally phrased “16 + search tool + 4 newly-dispatchable … as applicable”; because the four DKG tools already belong to the 16, the exact post-change integer must be derived from live construction, not inferred as 21. |

### H-10 — Canvas shell + CODE-GRAPH

Source: `H-10_canvas_shell_code_graph.md` (196 lines). Status: planned Wave 2 build; B0 blocks rendering.

| Atomic ID | Class/domain | Exact source anchor(s) | Explicit feature or behavior |
|---|---|---|---|
| H005-10-001 | E-STATUS/TEST | L3, L62-L67 | H-10 is planned Wave 2; B0 is blocking and no renderer code may ship after a failed measurement gate. |
| H005-10-002 | E-CAP/STATE | L5-L13 | Add a 2D generative canvas WindowHost type for code topology and disconnected dead-code islands, while correcting the premise that the symbol call graph itself is absent. |
| H005-10-003 | E-STATE | L7-L11, L37-L46 | `build_symbol_view -> get_graph_view_data(mode=symbol) -> GraphWindow.svelte` already renders outgoing-call edges; `dkg_edges` itself contains no call edges and call-edge adapter queries are structurally empty. |
| H005-10-004 | E-STATE/DATA | L9-L11, L43-L44 | PageRank is producer-dark: readers query `dkg_nodes_pagerank`, but no producer materializes it; `compute_repo_map` is only in-memory. |
| H005-10-005 | E-TEST/CFG | L62-L67 | B0 measures worst-case real-project DKG node and edge counts, rechecks state-store isolation, records results/renderer choice, and verifies dependencies before package changes. |
| H005-10-006 | E-CFG/LIMIT | L25-L30, L62-L67 | Select Cytoscape WebGL 3.34 for roughly <2,500 nodes at measured density; otherwise select `sigma@3.0.3`; sigma v4 alpha is blocked. |
| H005-10-007 | E-LIMIT/OBS | L25-L31 | Published renderer figures (~100fps at 1,200/16,000 and ~10fps at 3,200/68,000) are planning evidence showing edge density dominates; B0 real-corpus measurement owns the decision. |
| H005-10-008 | E-FAIL/CFG | L25-L30, L69-L71, L141 | Cytoscape WebGL is provisional and must gracefully fall back to canvas if its flag/API disappears. |
| H005-10-009 | E-DEP/CFG | L28-L30, L66-L67, L110-L114 | `cytoscape>=3.31`, real-but-stale `cytoscape-layout-utilities@1.1.1`, and optionally `sigma@3.0.3` must be registry-verified; no dependency may come from memory alone. |
| H005-10-010 | E-CAP/INT | L69-L71 | `Canvas.svelte` mounts the chosen renderer in the real WindowHost tab bar and reads symbol graph plus PageRank with pan/zoom and viewport culling keyed by `cy.extent()`. |
| H005-10-011 | E-CAP/DATA | L73-L75 | Persistently materialize `dkg_nodes_pagerank` on `elai symbol_graph build` so `get_pagerank_top` and the PageRank panel are non-empty. |
| H005-10-012 | E-FLOW/DATA | L31-L35, L77-L81 | Run dynamic-entrypoint injection before orphan detection; sources include Tauri command handlers, `dyn Trait` sites, and SSB action-type consumers. |
| H005-10-013 | E-LIMIT/FAIL | L31-L35, L109-L113 | Topological isolation is necessary but insufficient; a static orphan may never be labeled confirmed dead without coverage evidence. |
| H005-10-014 | E-DATA/CAP | L77-L81, L119-L122 | Add `is_dead_code` and `coverage_count`; coverage-confirmed dead is text-ghost/dashed, while topology-only suspected dead gets a distinct lighter amber-neutral treatment. |
| H005-10-015 | E-FAIL | L188-L190 | If Layer-3 coverage production is unavailable, degrade to suspected-only and never produce a false confirmed-dead state. |
| H005-10-016 | E-CAP/CFG | L77-L86 | Use fcose component packing, and register layout utilities because omission makes packing a silent no-op. |
| H005-10-017 | E-CAP/CFG | L83-L86 | Semantic LOD is pixel-based: far `<8px` folder supernodes/bundled edges/no text; mid `8–40px` per-file with labels above ~24px; near `>40px` per-symbol card chrome. |
| H005-10-018 | E-LIMIT/OBS | L30-L31, L83-L86, L113 | Hide labels/chrome below pixel thresholds and bound drawn elements irrespective of graph size; this is the stated 60fps lever. |
| H005-10-019 | E-CAP/DATA | L47-L48, L83-L86 | Add a net-new SCC pass over import/call graph so real cycles replace hardcoded `VizFileEdge.cyclic=false`. |
| H005-10-020 | E-STATE/CFG | L47-L49 | Existing viz limits are 20,000 symbols, 60,000 edges, and 200 symbols/file; `level`/`status` are wire-dark; state stores currently pass the no-three.js-import isolation gate. |
| H005-10-021 | E-INT | L93-L107 | Produced surfaces are `Canvas.svelte`, `entrypoints_map.ts`, PageRank materializer, dead/coverage/SCC fields, and decision record; they consume existing graph, PageRank, impact, entrypoint, and WindowHost surfaces. |
| H005-10-022 | E-FLOW/INT | L105-L107 | Production paths are WindowHost→Canvas→`get_graph_view_data` and `elai symbol_graph build`→materializer→non-empty `get_pagerank_top`. |
| H005-10-023 | E-INT/OBS | L115-L122 | Frontend/backend pair is Canvas plus existing Tauri commands; evidence is the showFps HUD, materialized table, Layer-3 coverage, known orphan, and non-dead Tauri-only function. |
| H005-10-024 | E-TEST | L124-L134 | Deterministic checks run the wiring trigger, require Canvas/decision files, verify chosen renderer, layout registration, PageRank writes, coverage/dead fields, and empty forbidden store imports. |
| H005-10-025 | E-TEST | L136-L143 | Acceptance requires B0 record, ≥60fps worst-case render, real tab/build reachability, honest dead-code distinction, verified/fallback renderer, cross-language diff, dashboard E2E, workspace build, Svelte check, and auditor pass. |
| H005-10-026 | E-TEST | L158-L164 | Contracts cover non-empty PageRank, true orphan, dynamic-only non-orphan, suspected-only unknown coverage, SCC cycle, and a dashboard fixture asserting showFps ≥60 and retained `webgl:true`. |
| H005-10-027 | E-OBS/DATA | L154-L155 | H-10 adds no SSB action type; PageRank table production and dashboard render are the artifacts. |
| H005-10-028 | E-DATA/INT | L166-L174 | Implementation inventory creates Canvas/entrypoint map/tests/decision record and modifies WindowHost, package metadata, visualization fields, and PageRank producer. |
| H005-10-029 | E-DATA/OBS | L88-L90 | The decision record must capture B0 numbers/renderer, inverted call-graph premise, dead-code honesty boundary, entrypoint/coverage backstops, and dependency blocks. |
| H005-10-030 | E-LIMIT/DEP | L185-L187 | H-10 is independently shippable; do not begin H-13/H-14 before H-10–H-12 are default. |
| H005-10-031 | E-CFG/TEST | L188 | Re-run B0 whenever the target codebase becomes materially larger; renderer choice is data-bound rather than permanent. |
| H005-10-032 | E-OBS/TEST | L189 | Measure and surface the dead-code false-positive rate on ELAI’s actual dispatch patterns rather than assume it is small. |
| H005-10-033 | E-DEP | L17-L21, L192-L194 | H-10 depends on shipped symbol graph, graph commands, entrypoint detection, and project intelligence; H-11/H-12 extend it, and H-16 retires orbit only after a full default-canvas release. |
| H005-10-034 | E-LIMIT | L109-L114 | Stub bans also prohibit sigma v4, unregistered fcose packing, below-threshold labels/chrome, and unverified packages. |
| H005-10-035 | E-DET | L155-L156 | Runtime rendering is in-process; B0 is a one-time measurement rather than a per-run network dependency; `coverage_count` is authoritative. |
| H005-10-036 | A-CFG | L28-L30, L62-L67 | The ~2,500-node switch is provisional and explicitly subordinate to measured edge density; the plan does not give a deterministic density formula for choosing Cytoscape versus Sigma. |
| H005-10-037 | A-DATA | L95-L97, L171-L172 | The exact crate owning the PageRank writer remains a choice (`ssb_viewer.rs OR knowledge/...`; modify/create knowledge or orchestrator). |

### H-21 — Runtime look-ahead / causality differential gate

Source: `H-21_runtime_lookahead_differential.md` (157 lines). Status: planned Track 3, Wave 3 Cluster B.

| Atomic ID | Class/domain | Exact source anchor(s) | Explicit feature or behavior |
|---|---|---|---|
| H005-21-001 | E-STATUS/DEFER | L3, L13-L15 | Tier 1 is an in-process fail-closed column diff plus SSB row; Tier 2 is a frozen twice-run sidecar with fixture now and separately gated live checkbox. |
| H005-21-002 | E-CAP/FLOW | L5-L11, L47-L55 | Execute a feature/signal pipeline on full data and future-truncated/time-shifted data, then compare overlapping output columns. |
| H005-21-003 | E-DET | L7-L10, L52-L55 | Equality uses configured ULP/relative tolerance; any beyond-band change is `BLOCK/FutureLeak`, while zero-within-tolerance is `ALLOW/Causal`. |
| H005-21-004 | E-LIMIT/FAIL | L23-L30, L85-L88 | Runtime dataframe behavior—not name grep—is authoritative for dynamic paths; H-21 backstops but does not replace cheap early H-20 static analysis. |
| H005-21-005 | E-OBS | L10-L11, L57-L59 | Persist the dataframe diff as off-agent evidence; it carries the first changing column, index, and tolerance. |
| H005-21-006 | E-STATE | L32-L36 | `MetricGate` and `DriftComparator`/first-divergence reporting are shipped; a perturb-and-diff feature runner is absent. |
| H005-21-007 | E-INT/LIMIT | L33-L36, L78-L80 | Reuse `CompiledPredicate`/MetricGate and the `DriftComparator` reporting shape rather than reimplement them. |
| H005-21-008 | E-CAP/DET | L47-L50 | The frozen sidecar emits two dataframes, is seeded by `ELAI_SEED`, uses no wall clock, belongs in the frozen gate set, and should reuse a REGRET-style runner. |
| H005-21-009 | E-FAIL | L52-L55, L85 | Missing either rerun output is `FAIL/BLOCK`, never `SKIP` or implicit allow. |
| H005-21-010 | E-DATA/INT | L57-L59, L73-L83 | Add `diff_future_truncation`, `run_pipeline_twice`, and Pattern-2 `LookaheadDifferential` with kinds `Causal`/`FutureLeak`, on the production pre-promotion path. |
| H005-21-011 | E-DATA/STATE | L57-L59, L120 | Event planning uses “baseline+1 from live 207/484” and requires rereading/bumping the live schema/e45 pin at edit time. |
| H005-21-012 | E-TEST | L61-L63 | Frozen fixtures include `iloc[t+1]` that must fail and a causal feature that must diff to zero; live strategy execution is deferred behind its own checkbox. |
| H005-21-013 | E-DET/SEC | L65, L84-L88 | Freeze runner and tolerance in `gate-manifest.txt`; use blake3, avoid production `.unwrap()`, and never advertise a fail-closed Tier-2 sidecar if it is fail-open. |
| H005-21-014 | E-LIMIT/DET | L68-L70, L88, L110 | Cross-hardware bit identity is not asserted; relative tolerance and a stable verdict band are the honest boundary. |
| H005-21-015 | E-OBS/CAP | L89-L94 | Leakage drawer shows first changing column/index; a real fixture produces a recomputable `FutureLeak` row and visible block, while the causal fixture passes. |
| H005-21-016 | E-FLOW/INT | L81-L83, L108-L109 | `diff_future_truncation` runs from the live pre-promotion path for strategies with feature/signal pipelines, not from test-only callers. |
| H005-21-017 | E-CFG | L104-L109, L116 | Gate is default-on with no enable override; only a kill switch is permitted. |
| H005-21-018 | E-TEST | L96-L102 | Deterministic checks run wiring trigger, confirm reuse, reject blake2/sha2 in the new module, and require the decision record. |
| H005-21-019 | E-TEST | L104-L112 | Acceptance requires broken/clean fixtures, live entrypoint, Tier split, FP honesty, reuse, same-environment recomputation, workspace green, auditor pass, and no dependency. |
| H005-21-020 | E-TEST | L123-L127 | Contract cases cover FutureLeak with first column, causal allow, missing rerun block, relative tolerance, and identical same-seed diff. |
| H005-21-021 | E-DATA/OBS | L67-L70 | Decision record documents dynamic-static boundary, truncation/shift method, tolerance/FP residue, and H-20 complementarity. |
| H005-21-022 | E-DATA/INT | L129-L136 | Implementation inventory creates Rust diff, Python runner, fixtures/tests/decision record; modifies SSB locks, pre-promotion path, and gate manifest. |
| H005-21-023 | E-FAIL/LIMIT | L144-L146 | Truncate only future observations, never required past lookback; verify the truncation boundary for each data shape. |
| H005-21-024 | E-OBS/TEST | L147-L148 | Before default-on, bound FP rate on at least three real causal pipelines while tuning tolerance. |
| H005-21-025 | E-LIMIT/CFG | L149-L150 | Twice-running costs roughly 2× a backtest, so schedule H-21 pre-promotion rather than per-edit. |
| H005-21-026 | E-DEP | L16-L19, L152-L155 | Requires shipped rules MetricGate, DriftComparator, and H-20 pairing; H-22 covers perturbation/null cases truncation may miss. |
| H005-21-027 | E-DET | L93-L94, L111 | Journal result must recompute under `ELAI_SEED` in the same environment. |
| H005-21-028 | A-CFG | L47-L55, L67-L70 | The source requires a chosen truncation/shift method and tolerance but leaves their concrete values/algorithm to the decision record and real-pipeline calibration. |

### H-25 — Result manifest + re-derivation verifier

Source: `H-25_result_manifest_rederivation.md` (170 lines). Status: planned Track 3, Wave 3 Cluster E, Tier 1.

| Atomic ID | Class/domain | Exact source anchor(s) | Explicit feature or behavior |
|---|---|---|---|
| H005-25-001 | E-STATUS | L3, L16 | Planned in-process Tier-1 build, extending shipped F-4, with roughly 60% of primitives already present. |
| H005-25-002 | E-STATE/LIMIT | L5-L14, L25-L32 | F-4 proves a test-pass stdout hash but does not bind an empirical value; H-25 extends that discipline rather than restating it. |
| H005-25-003 | E-DATA/CAP | L8-L14, L56-L59 | Every reported empirical number requires typed `ResultManifest {value,captured_output_blake3,parser_expr,data_hash,code_commit,env_lock_hash,elai_seed,run_id,n_trials,dsr_pvalue,pbo}`. |
| H005-25-004 | E-FAIL | L56-L59, L95-L99 | A number without a manifest is `BLOCK`; no production result may bypass it. |
| H005-25-005 | E-CAP/DET | L61-L64 | `verify_result` reparses captured output with `parser_expr` and asserts equality with `value`. |
| H005-25-006 | E-CAP/DET | L61-L64 | The verifier rehashes pinned data, environment lock, and code commit and asserts digest agreement. |
| H005-25-007 | E-INT/DATA | L61-L64, L90-L91 | Verify `n_trials`, `dsr_pvalue`, and `pbo` against H-17/H-18 outputs; bind them without recomputation. |
| H005-25-008 | E-FAIL | L61-L64 | A hand-edited value, digest mismatch, or missing captured output fails closed. |
| H005-25-009 | E-SEC/CAP | L66-L68 | Add `AST_PATTERN` blocking strategy import, redefinition, or monkeypatch of metric/backtest harness. |
| H005-25-010 | E-SEC/DET | L66-L75 | Metric/backtest harness remains gate-frozen; AST enforcement prevents strategy-local replacement. |
| H005-25-011 | E-DATA/INT | L70-L72, L82-L94 | Emit Pattern-2 `ResultManifestVerified` with kinds `ReDerived`, `FabricationDetected`, and `MonkeypatchBlocked` from live result reporting. |
| H005-25-012 | E-OBS | L70-L72 | Orchestrator-captured output plus independently re-derived value are the off-agent artifacts. |
| H005-25-013 | E-STATE | L34-L43 | F-4 stdout discipline, RRA manifest/blake3, H-17/H-18 fields, and rules predicates exist; result schema/reparser is absent. |
| H005-25-014 | E-INT | L83-L94 | New `ResultManifest`, `verify_result`, and locked `finance/result_manifest.toml` consume F-4, RRA `RunArtifact`, trial ledger/H-18, and shipped rules primitives. |
| H005-25-015 | E-CFG/DATA | L85, L142 | `result_manifest.toml` is `mutable=0` and combines SchemaRequired with the monkeypatch AST ban. |
| H005-25-016 | E-FLOW | L92-L94, L119 | Every live result-reporting path routes through `verify_result` before a number lands; a test-only caller is insufficient. |
| H005-25-017 | E-CFG | L116-L120, L126 | Verification is default-on for every result, with no enable override and only a kill switch. |
| H005-25-018 | E-CAP/OBS | L100-L105 | Reviewer cockpit shows reported value, re-derived value, and provenance, composed with G-5; fabrication is dashboard-visible. |
| H005-25-019 | E-TEST | L103-L105 | Real hand-edited fixture must produce `FabricationDetected` and fail; clean manifest yields `ReDerived`; metric monkeypatch blocks. |
| H005-25-020 | E-TEST | L107-L114 | Deterministic checks run wiring trigger, confirm F-4 extension/rule file, reject blake2/sha2, and require decision record. |
| H005-25-021 | E-TEST | L116-L122 | Acceptance requires live/default path, frozen harness, bound H-17/H-18 values, one reused stdout discipline, same-seed recomputation, green build, auditor, and no dependency. |
| H005-25-022 | E-TEST | L133-L138 | Contracts cover edited/clean/missing-manifest/monkeypatch cases, binding-not-recompute, data-hash mismatch, and same-seed identical value. |
| H005-25-023 | E-DET/SEC | L95-L99, L112 | Only blake3 is allowed and production code has no `.unwrap()`/`.expect()`; do not duplicate F-4 or H-17/H-18. |
| H005-25-024 | E-DATA/OBS | L77-L79 | Decision record distinguishes F-4 test honesty from empirical honesty and records schema, reparse/rehash, bound trust values, and harness protection. |
| H005-25-025 | E-DATA/INT | L140-L147 | Implementation inventory creates verifier/rules/tests/fixtures/decision doc and modifies SSB locks, result path, and gate manifest. |
| H005-25-026 | E-SEC/DET | L156-L159 | `parser_expr` is restricted grammar, frozen, and sandboxed—not arbitrary executable code—so it cannot “derive” any chosen value. |
| H005-25-027 | E-SEC/OBS | L160-L161 | Captured output must come from the orchestrator’s F-4 capture path, never from agent-supplied pseudo-evidence. |
| H005-25-028 | E-LIMIT/DEFER | L162-L163 | Initial scope is numeric results; plots/visual results go to shipped `media_validator` VLM and remain valid-not-verified rather than numeric proof. |
| H005-25-029 | E-DEP | L17-L21, L165-L168 | Depends on shipped F-4/RRA/rules plus H-17/H-18; H-26 later checks wrong numeric behavior rather than fabricated provenance. |
| H005-25-030 | E-DATA/STATE | L70-L72, L130 | Event planning says baseline+1 from live 207/484 and requires live schema/e45 reconciliation at edit time. |
| H005-25-031 | A-DATA | L56-L64, L156-L159 | The manifest stores a `parser_expr`, but its concrete restricted grammar and sandbox implementation are deliberately unspecified and must be settled before safe execution. |

### H-31 — Tier-B user-preference bi-temporal ledger

Source: `H-31_user_decision_bitemporal_ledger.md` (100 lines). Status: Track 4 activation, Wave 4, depends H-30.

| Atomic ID | Class/domain | Exact source anchor(s) | Explicit feature or behavior |
|---|---|---|---|
| H005-31-001 | E-CAP/FLOW | L1-L3 | Persist explicit durable work preferences across sessions, honor them without re-asking, retire contradicted rules immediately, never narrate dead rules, and never silently learn preferences. |
| H005-31-002 | E-STATUS/LIMIT | L5-L6 | This is activation with exactly three missing production wires and zero new storage: capture, retrieval/prompt slot, and supersession SSB beacon/smoke. |
| H005-31-003 | E-LIMIT/INT | L5-L6, L91 | Reimplement Odysseus relevance (identity floor 0.9), near-dup threshold 0.92, sorted `(id,text,fact_type)` blake3 fingerprint, local headroom clamp, and deferred >50%-deletion guard in Rust against SSB; never port/copy Python. |
| H005-31-004 | E-STATE/DATA | L8-L10 | Source snapshot says live `SCHEMA_VERSION=220` and `TOTAL_VARIANTS=497`; older 191/464 documentation is stale. |
| H005-31-005 | E-DATA | L11-L12 | `FactType` has eight variants including `Preference`; thresholds include Preference 0.5 and Opinion 0.7; `Scope` is Project/Global and promotion is only documented/deferred. |
| H005-31-006 | E-DATA/FLOW | L13 | `FactStore::supersede` atomically closes the current old row and inserts the replacement; the old row remains and `DELETE` is never issued. |
| H005-31-007 | E-DET/DATA | L13, L39, L49 | Caller builds the new row at the same `now_ms`, making `old.valid_to_ms == new.valid_from_ms`; bi-temporal replay history is preserved. |
| H005-31-008 | E-DATA/FAIL | L14 | `soft_close` closes without replacement and returns `NotFound` for an already-closed row; idempotency belongs to its caller. |
| H005-31-009 | E-DATA | L15 | Existing store operations include `update_trust`, `list_current`, `iter_current`, `insert`, and `get`; `list_current` filters to `valid_to_ms IS NULL`. |
| H005-31-010 | E-FLOW/STATE | L17-L18 | Dispatcher behavior is ADD on no match, UPDATE via supersede on changed/non-decreasing-confidence value, DELETE tombstone via soft-close, and NOOP on identical/lower-confidence input. |
| H005-31-011 | E-STATE | L18 | `dispatch_one`/`dispatch_batch` have only test callers; no production turn extractor currently feeds them. |
| H005-31-012 | E-STATE/INT | L19 | Same-key `audit_conflicts` optionally soft-closes by tier and already has its first production live-tick caller. |
| H005-31-013 | E-STATE/INT | L20 | Cross-key semantic contradiction is live for Preference/Opinion/DomainKnowledge, uses default-on offline `DeterministicNliJudge`; `nli-onnx` is only a feature seam and its model is not on disk. |
| H005-31-014 | E-DATA/OBS | L20 | Semantic outcomes reuse one `MemoryConflictDetected` row with `conflict_kind="semantic_contradiction"`; H-31 adds no duplicate conflict event. |
| H005-31-015 | E-STATE/FLOW | L22 | HALO-like per-FactType half-life decay, monotone exponential trust, tick, and scheduler are already production-wired. |
| H005-31-016 | E-DATA/FLOW | L24-L25 | `get_relevant_facts` starts from `list_current`, so closed/superseded rows are excluded in SQL before narration. |
| H005-31-017 | E-SEC/DATA | L25 | Retrieval partitions safety-critical T2/T3 rows out with tracing, applies type trust floor/override, rejects empty evidence pointers, sorts by `last_observed` descending, truncates to `k`, and bumps `last_observed` on returned rows. |
| H005-31-018 | E-STATE | L25 | All `get_relevant_facts` callers are tests; the production retrieval-to-prompt path is absent. |
| H005-31-019 | E-STATE/DATA | L27-L30 | Current slots are SystemPrompt, Rules, PhaseSpec (not “Telescope”), RepoMap, CodeContext, SsbEntries, GrcHistory, OutputReserve (`SLOT_COUNT=8`), with no Memory; conversation has no memory reference and only the Tauri drawer reads `list_current`. |
| H005-31-020 | E-CAP/FLOW | L35-L38 | `/remember <text>` explicitly builds one `ExtractedFact` with type/key/value/confidence and user-turn evidence, then runs dispatcher reconciliation. |
| H005-31-021 | E-CAP/SEC | L36-L38 | Repeated-correction proposal triggers at `N=2`, asks “remember this? Y/N,” emits only the proposal unprompted, and performs no write without confirmation. |
| H005-31-022 | E-DATA | L37 | Capture normally uses `Preference`, with `RiskTol` for routing/spend and `Mistake` for footguns. |
| H005-31-023 | E-FLOW/INT | L39 | Existing deterministic conflict judge handles ordinary writes; a cloud model is invoked only for cross-key residue, avoiding a 7B call. |
| H005-31-024 | E-CAP/DATA | L40 | Add `ContextSlot::Memory`; prompt assembly calls `get_relevant_facts` for `Preference`, `k=8`, against the live user/store clock. |
| H005-31-025 | E-FAIL/LIMIT | L40, L49 | Superseded facts are structurally absent and must never be injected or narrated as `[OUTDATED]`; decay is a backstop, not primary staleness resolution. |
| H005-31-026 | E-CFG/LIMIT | L41 | Memory is retrieval-on-demand, never always-injected; budget is approximately 100–500 tokens, at most eight one-line facts, sized with local-window headroom. |
| H005-31-027 | E-SEC | L42, L52 | Before any insert, hard-reject secrets, API keys, PII, file contents, secret patterns, and values above the entropy floor, reusing the Stage-1 defense seam. |
| H005-31-028 | E-SEC/FLOW | L43 | Superseding a hard rule, dependency choice, or routing choice whose key is in `HIGH_STAKES_KEYS` requires InteractiveGate Y/N; other authorized flips auto-supersede. |
| H005-31-029 | E-SEC/CFG | L44 | Compile-time role branch forces Memory empty for auditor, critic, and adversary; only user-facing/executor roles can receive it, and this is not flag-controlled. |
| H005-31-030 | E-CFG/DEFER | L45, L56-L66 | Feature defaults off behind `ELAI_USER_MEMORY_ENABLE` and requires an allowlist row: experiment, owner `maintainer`, expiry `2026-09-01`, promotion `H-31`, named ON-path smoke. |
| H005-31-031 | E-SEC/CFG | L66 | Registering the frozen flag requires sanctioned out-of-band `gate_freeze_check.py --lock`; an executing run otherwise trips freeze verification. |
| H005-31-032 | E-DEFER | L47 | H-32/Tier 2 defers DKG/rerank ordering, project→global promotion, periodic LLM tidy/audit, and activation of the >50%-deletion guard. |
| H005-31-033 | E-LIMIT/FAIL | L47, L49 | Until tidy exists, there is no bulk deletion; all retirement is per-fact supersede/soft-close and physical row deletion is banned because it breaks replay. |
| H005-31-034 | E-INT | L51-L53 | Capture wire is REPL/CLI→`user_memory_capture.rs`→live `FactStore`→`dispatch_one`; repeated-correction state reuses the FactStore already held by the live tick. |
| H005-31-035 | E-DATA/INT | L53 | Insert Memory before OutputReserve (or otherwise choose lowest churn), raise `SLOT_COUNT` 8→9, and update every per-tier split table. |
| H005-31-036 | E-CAP/INT | L54 | Extend `MemoryFactsDrawer` with a “superseded by” affordance using existing data; no new display Tauri command is planned. |
| H005-31-037 | E-DATA/OBS | L55 | Add exactly one Pattern-2 `UserDecisionSuperseded` action with discriminator `Superseded`, `Captured`, or `ProposalSurfaced`. |
| H005-31-038 | E-SEC/DATA | L55 | Supersede payload contains only `{old_fact_id,new_fact_id,fact_type,key}` and never the value, preventing journal secret leakage. |
| H005-31-039 | E-DATA/STATE | L55 | Planned bump is 220→221 and 497→498 in lockstep, but executor must reread live values and apply baseline+1. |
| H005-31-040 | E-INT/FLOW | L68-L78 | Produced capture/event/Memory slot consume dispatcher, retrieval, store lifecycle, and live conflict ticks; real entrypoints are CLI/REPL remember plus executor `elai run` prompt assembly. |
| H005-31-041 | E-FAIL/SEC | L79 | Stub bans: no silent writes, stale narration, physical delete, protected-role injection, secret storage, or producer `.unwrap()`; propagate `MemoryError`. |
| H005-31-042 | E-OBS/TEST | L80-L82 | `/remember X` then contradictory Y must show the exact temporal boundary in live DB and a supersession row independent of producer return value. |
| H005-31-043 | E-TEST/SEC | L82 | Frozen auditor-authored smoke drives the real flag-on retrieval/assembly path, proves X absent and protected-role memory count zero; missing DB/zero rows fails, never skips. |
| H005-31-044 | E-OBS/LIMIT | L86-L90 | External memory numbers are not ELAI results; a future-dated arXiv ID was unverified and must remain a flagged claim until independently confirmed. |
| H005-31-045 | E-LIMIT/SEC | L93 | Product stores verifiable work facts, not persona; proposals must use work-fact phrasing and never infer personality. |
| H005-31-046 | E-DEP/INT | L95-L100 | Consume H-30’s user/store substrate without reopening a store; H-32 ranks/tidies; H-37 supplies InteractiveGate; F-5 supplies quarantine analogy; existing live contradiction/decay tick is not duplicated. |
| H005-31-047 | C-INT | L5-L6 versus L36-L38, L52, L70-L75 | Capture target conflicts internally: the steal/build paragraph says route into `dispatch_batch`, while revised scope, integration, and produced symbol explicitly call `dispatch_one`; executor must settle on the latter or document batch semantics. |
| H005-31-048 | A-DATA | L35-L38 | The extractor may classify a remembered item as Preference, RiskTol, or Mistake, but the plan does not specify a deterministic classification rule/schema. |
| H005-31-049 | A-DATA | L53 | Context-slot numbering remains an explicit implementation choice: append Memory as variant 8 and move reserve, or insert before reserve; all tables must agree. |
| H005-31-050 | A-CAP | L30, L54 | Drawer currently calls `list_current`, yet the new affordance concerns supersession history; the plan says no new Tauri command but does not specify how closed predecessor rows become available to that current-only read. |

### H-44 — Behavioral trap detection + measured recall

Source: `H-44_behavioral_trap_detection_recall.md` (187 lines). Status: planned Track 7, Wave 7.

| Atomic ID | Class/domain | Exact source anchor(s) | Explicit feature or behavior |
|---|---|---|---|
| H005-44-001 | E-STATUS/DEFER | L3, L16-L18 | Tier 1 is in-process detector plus event and frozen-corpus recall fail-closed gate; full TRACE/live-generation sweep is a separately gated Tier-2 checkbox. |
| H005-44-002 | E-STATE/FAIL | L5-L14, L39-L45 | Current `score_candidate_for_traps` is self-described unsound substring matching; keyword-free verifier gaming passes and behavioral/TRACE surfaces are absent. |
| H005-44-003 | E-CAP/TEST | L57-L59 | Frozen ImpossibleBench spec/test conflicts classify any impossible pass as `ImpossiblePassExploit` and use a stop/read-only/abort guard recipe. |
| H005-44-004 | E-CAP/DET | L61-L62 | Reverify a flagged win under variable-renamed/structure-preserving isomorph; original-pass/isomorph-fail becomes `IsomorphMismatch`. |
| H005-44-005 | E-CAP/INT | L64-L66 | Execute/classify “compiles/asserts pass but wrong output/exception” as `ExecutionHallucination`, reusing H-43 execution rather than creating another runner. |
| H005-44-006 | E-FLOW/SEC | L68-L70 | Hard flag requires agreement from at least two different-family views; a single view is advisory. |
| H005-44-007 | E-CAP/LIMIT | L68-L70, L91 | Keep old grep registry only as cheap `KeywordFallback` advisory, never sole blocker. |
| H005-44-008 | E-OBS/TEST | L72-L74 | Measure recall and false-positive rate on a frozen, out-of-band TRACE-style honest/hacking corpus; unmeasured recall is fail-closed. |
| H005-44-009 | E-DATA/DET | L72-L79 | Freeze corpus and hash with blake3; results must be seed-reproducible. |
| H005-44-010 | E-DATA/OBS | L76-L79 | Pattern-2 `TrapBehaviorDetected` carries `{kind,recall,fp_rate,n,corpus_hash,seed}` with kinds ImpossiblePassExploit, IsomorphMismatch, ExecutionHallucination, KeywordFallback, mirroring `InsecureRateMeasured`. |
| H005-44-011 | E-DATA/STATE | L76-L79, L144 | Use one baseline+1 SSB variant (source snapshot 224/501); recall roll-up rides the same payload and does not add a second action variant. |
| H005-44-012 | E-STATE/INT | L40-L45 | Trap registry/guard chain/safedelta measured-row exist; D-24 and D-13 are planned compose targets; new detector/harness is absent. |
| H005-44-013 | E-INT | L85-L96 | Produce `detect_trap_behavior`, `TrapRecallReport`, and event; consume/downgrade old scorer, live guard chain, H-43 execution, and safedelta pattern. |
| H005-44-014 | E-FLOW/CFG | L95-L96, L125-L126 | Production path is self-mod guard chain behind allowlisted `ELAI_DARWINIAN_LOOP_ENABLE`; behavioral detection is default within that switch and adds no new flag. |
| H005-44-015 | E-FAIL/LIMIT | L97-L102 | Keyword-only gating, unmeasured recall, and unilateral single-view blocks are forbidden; use blake3 and no production unwrap/expect. |
| H005-44-016 | E-TEST/FAIL | L101, L111-L112 | Broken arm `--trap-keyword-only` must miss the keyword-free hack while the behavioral leg blocks it. |
| H005-44-017 | E-CAP/OBS | L103-L105 | DGM/trust drawer shows recall and per-kind counts; dashboard E2E shows keyword-free hack blocked and clean candidate passing. |
| H005-44-018 | E-OBS/TEST | L106-L112 | Real frozen H1/C1 generation must emit a flag only for H1 and a recomputable recall report, with H1 blocked by the guard. |
| H005-44-019 | E-TEST | L114-L121 | Deterministic checks require wiring trigger, production detector/event/report symbols, retained old scorer, no blake2/sha2, and decision record. |
| H005-44-020 | E-TEST | L123-L133 | Acceptance requires live/default path, measured TRACE recall, two-family agreement, reused substrates, byte-identical seed results, broken arm, Tier split, green build, auditor. |
| H005-44-021 | E-TEST | L148-L154 | Contracts cover all three behavioral kinds, advisory-vs-blocking view count, repeatable recall, and keyword-only miss; frozen fixtures include H1/C1, conflicts, rewriter, and corpus hash. |
| H005-44-022 | E-DATA/OBS | L81-L83 | Decision record explains proxy gaming, cross-view gate, measured-recall mandate, and explicitly reports incomplete recall/eval-awareness residue. |
| H005-44-023 | E-DATA/INT | L156-L163 | Implementation creates detector/tests/fixtures/decision doc; modifies old scorer, guard chain, one SSB variant/locks, and dashboard/types pair. |
| H005-44-024 | E-LIMIT | L173-L176 | Strong candidates may sandbag recognizable canaries; mitigate through H-47 indistinguishability and report residual rather than claim completeness. |
| H005-44-025 | E-FAIL/LIMIT | L177-L178 | Isomorphic transforms may fail semantic preservation, so `IsomorphMismatch` requires second-family confirmation and cannot block alone. |
| H005-44-026 | E-OBS/LIMIT | L179-L180 | Paper’s 63% best recall is a realistic external ceiling, not an ELAI result; frozen corpus is necessary but insufficient. |
| H005-44-027 | E-DEP | L19-L24, L182-L185 | Runs after H-43; composes D-24, D-13, E-54 and later H-45/H-47; keyword registry remains advisory. |
| H005-44-028 | E-LIMIT | L31-L37, L165-L171 | Published hack-rate/detection figures only ground recipes; ELAI must report its own detector performance and never claim 100%. |
| H005-44-029 | E-DET | L129-L132 | Recall and FP rate must recompute byte-identically under `ELAI_SEED` over the frozen corpus. |
| H005-44-030 | A-DATA | L76-L79, L88-L89, L109-L110, L144 | `TrapRecallReport` is defined as a Rust roll-up and observable text calls it a “row,” while W-9 says recall rides `TrapBehaviorDetected` with no second variant; exact row/payload representation needs reconciliation. |

### H-47 — DGM eval firewall

Source: `H-47_dgm_eval_firewall_dategate_templateregen.md` (222 lines). Status: planned Track 7, Wave 7, including J-11/S-8c amendment.

| Atomic ID | Class/domain | Exact source anchor(s) | Explicit feature or behavior |
|---|---|---|---|
| H005-47-001 | E-STATUS/DEFER | L3, L16-L18 | Tier 1 is in-process date gate, template regeneration, DP-budget binding, and event over frozen fixtures; live corpus indistinguishability is a separate Tier-2 checkbox. |
| H005-47-002 | E-LIMIT/INT | L5-L14 | H-47 composes D-6 store/date tags, E-54 filtering/cutoffs/probes, and H-19 budget; rebuilding any of them is a P0 violation. |
| H005-47-003 | E-CAP/FLOW | L27-L36, L57-L60 | `date_gated_sample` allows only items whose D-6 release date is newer than the Darwin generation’s last config-update timestamp; pre-update items become `PreUpdateItemRejected`. |
| H005-47-004 | E-DET/CFG | L57-L60, L103 | Date comparison uses SeededClock C-15, never wall clock. |
| H005-47-005 | E-CAP/DET | L62-L65 | Regenerate each templated task at score time from a gate-frozen template, seeded by generation id, so the exact instance is not scored twice. |
| H005-47-006 | E-INT/DATA | L62-L65 | Regenerated tests compose E-54’s test-triplet schema and emit `TemplateRegenerated`. |
| H005-47-007 | E-CAP/FLOW | L67-L69 | Every DGM score query spends H-19 Thresholdout budget; after `Exhausted`, queries are refused until frozen-path reprovisioning. |
| H005-47-008 | E-DATA/INT | L67-L69, L78 | Reuse `HoldoutBudgetEvent` and H-19 API; do not add a second budget mechanism or budget event. |
| H005-47-009 | E-CAP/OBS | L71-L73 | Run E-54/D-23 evaluation-awareness probe and surface measured `IndistinguishabilityProbe`; high distinguishability is a sandbagging risk, not silently ignored. |
| H005-47-010 | E-DATA/OBS | L75-L78 | Pattern-2 `EvalFirewallEvent` payload is `{kind,items_scored,items_rejected,regen_count,budget_remaining,distinguishability,seed}`. |
| H005-47-011 | E-DATA | L75-L78 | Firewall kinds are `PreUpdateItemRejected`, `TemplateRegenerated`, and `IndistinguishabilityProbe`; budget outcomes stay in H-19’s variant. |
| H005-47-012 | E-DATA/STATE | L75-L78, L144 | Event is baseline+1 from source snapshot 224/501; reread live schema and reconcile e45 if it is first Track-7 bump. |
| H005-47-013 | E-STATE/DEP | L38-L44 | D-6/E-54/H-19 are planned; held-out sampling and generation clock are shipped; loop-specific date/regen/budget wiring is absent. |
| H005-47-014 | E-INT | L84-L96 | Produce date sampler/regenerator/event; consume shipped sampler, D-6/E-54 data, H-19 budget, H-43 scorer, and safedelta measured-row pattern. |
| H005-47-015 | E-FLOW/CFG | L95-L96, L126-L127 | Production path is `run_one_generation -> date_gated_sample/regenerate_instance -> execution_grounded_fitness` under the existing allowlisted Darwinian switch, with no new default-off gate. |
| H005-47-016 | E-FAIL | L97-L103 | Fail on pre-update scoring, exact template reuse, unbounded adaptive queries, or duplicate store/filter/budget implementations. |
| H005-47-017 | E-TEST/FAIL | L102, L112-L113 | Broken arm `--eval-ignore-dates` must score memorized pre-update P and inflate fitness, proving the date gate is load-bearing. |
| H005-47-018 | E-DET/SEC | L103 | Hash items/templates only with blake3, use no production unwrap/expect, and use SeededClock. |
| H005-47-019 | E-CAP/OBS | L104-L106 | DGM/trust drawer shows rejected count, regen count, budget remaining, and distinguishability; dashboard shows P rejected and Q scored. |
| H005-47-020 | E-OBS/TEST | L107-L113 | Frozen P/Q/T generation must score Q plus fresh T, never P, emit firewall rows, spend/refuse budget, and be journal-recomputable. |
| H005-47-021 | E-TEST | L115-L122 | Deterministic checks require wiring trigger, production sampler/regenerator/event, reuse of `HoldoutBudgetEvent`, no blake2/sha2, and decision record. |
| H005-47-022 | E-TEST | L124-L133 | Acceptance requires P/Q/T/budget behavior, default/live path, named sandbagging residue, composition, seed-identical stats, broken arm, Tier split, green build, auditor. |
| H005-47-023 | E-TEST | L148-L153 | Contracts cover dates, fresh instance per generation, budget exhaustion, probe measurement, broken arm, and identical same-seed firewall stats. |
| H005-47-024 | E-DATA/OBS | L80-L82 | Decision record separates composed versus new work and records date, regen, Thresholdout, eval-awareness residue, and non-cure language. |
| H005-47-025 | E-DATA/INT | L155-L161 | Implementation creates firewall/tests/fixture/decision record and modifies loop eval, one SSB variant/locks, and dashboard/types pair. |
| H005-47-026 | E-FAIL/CFG | L170-L172 | A strict newer-than-update window may starve rapid generations; tune on frozen fixture and never substitute wall clock. |
| H005-47-027 | E-LIMIT | L173-L176 | Regeneration covers only templated tasks; free-form tasks rely on date/budget, and indistinguishability surfaces—but cannot eliminate—sandbagging. |
| H005-47-028 | E-DEFER/DEP | L177-L182 | Coordinate shared schema; if D-6/E-54/H-19 land later, ship against fixture store and leave live composition gated until dependencies exist. |
| H005-47-029 | E-SEC/FAIL | L185-L195 | Amendment Mode 1: eval container must not expose `.git` history/reflog/loose objects; destroy and reinitialize Git during fixture hygiene (shared with D-6). |
| H005-47-030 | E-SEC/FAIL | L196-L200 | Amendment Mode 2: sweep all Python environments, site-packages copies, pip wheel caches, and `/tmp` C/Rust extension artifacts for reference implementations. |
| H005-47-031 | E-SEC/FAIL | L202-L206 | Amendment Mode 3: runtime command allowlist blocks source recovery via pip install/download, curl/wget, clones, and uses package-name fuzzy rules for suspicious commands. |
| H005-47-032 | E-SEC/LIMIT | L208-L213 | Amendment Mode 4: LLM trace audit only surfaces suspicious behavior; it cannot block without deterministic rule confirmation because LLM judgment is Tier-3 advisory. |
| H005-47-033 | E-SEC/FLOW | L215-L220 | Enforce two layers: pre-clean local artifacts (Modes 1–2, shared D-6) before execution, then runtime network/command prevention (Mode 3, H-47); Mode 4 overlays both advisory-only. |
| H005-47-034 | E-OBS/LIMIT | L27-L36, L163-L168 | External date/regeneration/DP/awareness results justify mechanisms but are not ELAI guarantees; firewall is necessary-not-sufficient. |
| H005-47-035 | E-DEP | L19-L23, L180-L183 | Requires H-43 and coordinates with D-6/E-54/H-19; together with H-44/H-45/H-43 it forms the verification filter, with E-53 as narrow proof ceiling. |
| H005-47-036 | A-CFG | L57-L60, L170-L172 | Concrete mapping between release date and config-update timestamp, including starvation-window tuning, is unspecified. |
| H005-47-037 | A-SEC | L202-L206 | Command allowlist and fuzzy package detector are required, but exact allowed commands, network policy, and matching threshold are not specified. |
| H005-47-038 | C-LIMIT | L31-L32 | Context paraphrases Thresholdout as bounding adaptive-query overfitting to `O(1)` independent of query count; the plan treats this as grounding, not an implementation acceptance metric, so it must not be promoted into an ELAI guarantee. |
| H005-47-039 | E-STATE/DATA | L39 | Planned D-6 substrate is `.elai_cc/held-out/{date}/{task}.toml` with release tags, SeededClock, and automatic retirement of uniformly passing items older than six months. |
| H005-47-040 | E-STATE/DATA | L40 | Planned E-54 substrate provides 13-gram plus embedding-cosine filtering, per-family shards, model-month cutoffs, test triplets, and eval-awareness probes. |
| H005-47-041 | E-STATE/DATA | L41 | Planned H-19 substrate is Thresholdout/Ladder DP with locked `mutable=0` configuration and `HoldoutBudgetEvent` kinds QuerySpent, Exhausted, and Provisioned. |

### H-48 — Wire BAI bandit on real reward

Source: `H-48_wire_bandit_on_real_reward.md` (177 lines). Status: planned Track 7 P2, sequenced last.

| Atomic ID | Class/domain | Exact source anchor(s) | Explicit feature or behavior |
|---|---|---|---|
| H005-48-001 | E-STATUS/DEFER | L3, L14-L19 | Tier 1 is in-process update/sample plus telemetry over frozen multi-arm fixture; live multi-generation regret is RunPod-pending Tier 2; H-48 runs last. |
| H005-48-002 | E-STATE | L5-L12, L36-L42 | `ArmPosterior`, `bai::sample`, and hierarchical best-arm structures are shipped, but live Darwinian sample/update/run on real reward is scaffold/absent. |
| H005-48-003 | E-DEP/LIMIT | L7-L19, L166-L168 | H-43 execution reward and H-45 worst-case aggregate are hard prerequisites; a regret selector over byte hash is theater and must not ship first. |
| H005-48-004 | E-CAP/FLOW | L55-L57 | Each generation updates candidate `ArmPosterior` from H-45 aggregate as success/failure or a Beta update, never byte hash/placeholder. |
| H005-48-005 | E-CAP/CFG | L55-L57 | Apply Discounted Thompson Sampling so old observations decay under nonstationary archive rewards. |
| H005-48-006 | E-CAP/INT | L59-L61 | Next-arm selection calls shipped `bai.rs::sample` or D-29 hierarchical best-arm on updated posteriors; do not reauthor the bandit. |
| H005-48-007 | E-CAP/CFG | L63-L64 | Under fixed generation budget, run Successive Rejects for best-arm identification. |
| H005-48-008 | E-CAP/DATA | L66-L68 | Seed arm/guard-task curriculum from partial intermediate states of past generations, not only terminal benchmarks, composing H-44 tasks. |
| H005-48-009 | E-OBS/INT | L70-L73 | Prefer D-5’s BAI telemetry; add Pattern-2 `BanditArmSampled` baseline+1 only when no reusable emitter exists. |
| H005-48-010 | E-DATA/STATE | L70-L73, L137 | Conditional new variant uses source snapshot 224/501 and requires live/e45 reconciliation if first Track-7 bump. |
| H005-48-011 | E-DATA/OBS | L75-L77 | Decision record covers sequencing, DTS, Successive Rejects, ExIt, composed/activated split, stationarity residue, and deferred live regret. |
| H005-48-012 | E-INT | L79-L90 | Produce `update_arms_from_reward`/`select_next_arm` and conditional event; consume `sample`, `ArmPosterior`, hierarchy, H-45 aggregate, and H-43 fitness. |
| H005-48-013 | E-FLOW/CFG | L89-L90, L118-L119 | Live path is `run_one_generation -> update -> select` behind existing Darwinian kill switch; it is default within that switch with no new gate. |
| H005-48-014 | E-FAIL/LIMIT | L91-L96 | Fail byte/placeholder reward, unwired scaffold, or bandit reimplementation; use SeededRng and no production unwrap/expect. |
| H005-48-015 | E-TEST/FAIL | L95, L105-L106 | Broken arm `--bandit-reward-from-bytes` must converge to failing hash-favorite B while real reward converges to G. |
| H005-48-016 | E-CAP/OBS | L97-L99 | Show per-arm posterior and selected arm in DGM/archive drawer unless D-5 already supplies it; dashboard shows convergence to genuinely best arm. |
| H005-48-017 | E-OBS/TEST | L100-L106 | Frozen G/B generation must emit reused/new telemetry and show seed-recomputable convergence to high-execution-fitness G. |
| H005-48-018 | E-TEST | L108-L114 | Deterministic checks require wiring trigger, live update/select symbols, proof BAI code reads H-45/H-43, and decision record. |
| H005-48-019 | E-TEST | L116-L126 | Acceptance requires convergence, live/default path, real aggregate reward, activated scaffold, reused bandit/telemetry, seed-identical state, broken arm, Tier split, green build, auditor. |
| H005-48-020 | E-TEST | L141-L146 | Contracts cover posterior source, fixed-budget convergence, Successive Rejects, hash broken arm, partial-history seed, and identical same-seed posteriors/selection. |
| H005-48-021 | E-DATA/INT | L148-L154 | Implementation modifies Darwinian BAI/selection and optionally SSB/dashboard; creates test fixture and decision record. |
| H005-48-022 | E-CFG/DET | L162-L165 | Tune DTS discount on frozen fixture, never at runtime; nonstationarity weakens regret assumptions and live regret remains deferred. |
| H005-48-023 | E-LIMIT/INT | L168 | Minimize schema bumps by reusing D-5 emitter; new event is strictly conditional. |
| H005-48-024 | E-DEP | L170-L175 | H-43–H-48 compose fitness, trap recall, worst-case scoring, clade selection, firewall, and bandit; E-53 is a narrow proof ceiling. |
| H005-48-025 | A-DATA | L55-L57 | Mapping a continuous H-45 aggregate into Beta success/failure/update is left open (“success/failure or a Beta update derived from aggregate”). |
| H005-48-026 | A-CFG | L59-L64 | Source allows flat `bai::sample` or hierarchical best-arm and does not specify which path or how it composes with Successive Rejects. |
| H005-48-027 | A-INT | L70-L73, L97-L99 | Both telemetry variant and frontend work are conditional on D-5 state at authoring; live inspection decides whether either is added. |
| H005-48-028 | C-LIMIT | L162-L168 versus L170-L175 | Risk section says regret assumptions are only stationary-enough and live proof is pending, while “Next” calls the combined filter un-gameable and says it will “PROVABLY CONVERGE”; the latter is an overclaim not supported by this plan’s acceptance evidence. |

### H-52 — Deterministic finding anchoring + line-less schema

Source: `H-52_deterministic_finding_anchoring_cockpit.md` (59 lines). Status: Track 10 build, Wave 10; ELAI anchors are plan-asserted, not live-code verified.

| Atomic ID | Class/domain | Exact source anchor(s) | Explicit feature or behavior |
|---|---|---|---|
| H005-52-001 | E-STATUS/DEP | L1-L3 | Build depends on planned G-5 cockpit/DSR and C-7 hunk space; composes F-4 and defers E-41/C-40 GitHub delivery. |
| H005-52-002 | E-LIMIT/OBS | L3 | Model must not own the load-bearing location; deterministic anchoring prevents fabricated line joins and wrong-hunk vouches. |
| H005-52-003 | E-CAP/DATA | L7-L8 | Model emits verbatim `snippet`/`existing_code`, optional suggestion/vouch handle, and never a line number; deterministic code owns line/hunk. |
| H005-52-004 | E-DET | L7, L23 | Normalize each line by stripping whitespace and leading `+`/`-`, then match a consecutive normalized run. |
| H005-52-005 | E-FLOW | L7, L23 | Search each hunk’s NEW side first, then OLD side, then whole new-file content as fallback. |
| H005-52-006 | E-INT/LIMIT | L10, L23, L33 | Reuse ELAI’s existing diff/hunk parser and C-7 header-hash hunk ID; do not port the external regex/parser or rederive the ID space. |
| H005-52-007 | E-FAIL | L7, L23 | No match becomes `Unanchorable` and file-level fallback; never guess an inline line. |
| H005-52-008 | E-FAIL/DET | L10, L23 | More than one normalized match in scope becomes `Ambiguous` and file fallback; first-occurrence-wins is forbidden. |
| H005-52-009 | E-DET | L10, L23 | Use hunk-proximity preference toward the finding’s touched file-write hunk before resolving locality. |
| H005-52-010 | E-DATA/CAP | L23 | `anchor_finding(snippet,&[FileChange],Option<HunkId>) -> AnchorResult` returns optional hunk/range, kind, and candidate count. |
| H005-52-011 | E-DATA | L23 | `AnchorKind` reserves `Anchored`, `FileFallback`, `Ambiguous`, `Unanchorable`, and `Relocated`. |
| H005-52-012 | E-DATA/SEC | L24 | `ReviewFinding {snippet,claimed_vouch,severity}` has no model-settable line or verdict; deterministic resolver/gate own them. |
| H005-52-013 | E-TEST/SEC | L24, L44 | A stub-ban test asserts the schema exposes no model-settable line/verdict field. |
| H005-52-014 | E-DATA/OBS | L25, L35, L41 | Add one Pattern-2 `FindingAnchored` row on every attempt with finding ID, snippet blake3, optional hunk/start, kind, and candidate count. |
| H005-52-015 | E-DATA/STATE | L19, L25, L35 | Event is baseline+1 from plan-asserted live 224/501; reread cumulative live integers and reconcile stale cross-language pin at edit time. |
| H005-52-016 | E-INT/FLOW | L26, L34 | Extend (do not rewrite) G-5 `score_review_surface` to consume anchor rows and join vouches/findings to C-7 `hunk_id`. |
| H005-52-017 | E-FAIL/DATA | L9, L26 | Diff exclusion reasons are typed (`binary`, `user_exclude`, `unsupported_ext`, `default_path`, `deleted`), and oversized files are skipped/reported rather than silently truncated. |
| H005-52-018 | E-DATA/LIMIT | L26 | FileFallback/Ambiguous/Unanchorable attaches at file granularity and counts as `needs_attention`, conservatively preserving DSR denominator integrity. |
| H005-52-019 | E-CAP/OBS | L27, L37 | Existing cockpit renders Anchored inline and fallback/ambiguous in an un-vouched file band; Tauri joins event to hunk view and cross-language diff must pass. |
| H005-52-020 | E-DEFER | L28-L30 | One-call constrained LLM snippet relocation is Tier 2; `Relocated` is reserved now but model retry is not part of deterministic slice. |
| H005-52-021 | E-DEFER/INT | L18, L28-L30 | GitHub mapping (`0,0`→summary, anchored→inline) waits for C-40/E-41 and a separately gated network boundary. |
| H005-52-022 | E-FLOW/INT | L36, L43 | Insert anchoring before G-5 scoring in existing `elai review --cockpit`; no new subcommand, and dashboard CockpitWindow is the second real entrypoint. |
| H005-52-023 | E-CFG | L38 | Ship default-on with kill switch `ELAI_REVIEW_ANCHOR_DISABLE`, never default-off enable. |
| H005-52-024 | E-FAIL/LIMIT | L44 | Forbid model lines/verdicts, ambiguous guesses, silent truncation, naive line invention, resolver unwrap/expect, and non-blake3 hashing. |
| H005-52-025 | E-OBS/TEST | L45 | Unique snippet must emit Anchored with hunk/range, render inline, and join same C-7 hunk. |
| H005-52-026 | E-OBS/TEST | L45 | Duplicate snippet must emit Ambiguous and file fallback, never guessed line. |
| H005-52-027 | E-OBS/TEST | L45 | Missing snippet must emit Unanchorable and file fallback. |
| H005-52-028 | E-TEST/FAIL | L45 | Broken arm `--trust-model-line-numbers` must misanchor duplicate to wrong hunk and fail. |
| H005-52-029 | E-DET/TEST | L45 | Kind and hunk ID recompute byte-identically from snippet+diff under fixed `ELAI_SEED`. |
| H005-52-030 | E-FLOW/OBS | L45 | No findings is a valid no-op and emits zero `FindingAnchored` rows, distinguishable from a dead path. |
| H005-52-031 | E-OBS/TEST | L45 | Measure and bound Ambiguous+FileFallback over-fallback rate on at least three real reviews before default-on. |
| H005-52-032 | E-STATE/LIMIT | L13-L19 | Cockpit, hunk join, and vouch rows are plan-asserted/unfinished; executor must re-grep live symbols and schema rather than treat anchors as shipped code. |
| H005-52-033 | E-STATE/INT | L15-L18 | Planned consumers/producers are G-5 `HunkVouch`/scorer, C-7 header-hash ID, F-4 stdout hash, F-1 CWE, C-1 grounding sweep, and E-41 reviewer. |
| H005-52-034 | E-LIMIT | L11 | “Smart file bundling” and “reflection module” are grep-verified absent marketing/vaporware and excluded from scope. |
| H005-52-035 | E-OBS/LIMIT | L48 | External SEM-F1 26% vs 15.5% is not same-model isolated and is directional only, never a guarantee. |
| H005-52-036 | E-LIMIT/FAIL | L50 | Deterministic resolver is sound only for verbatim-after-normalization presence; reflow/paraphrase safely falls back or later uses labeled nondeterministic relocation. |
| H005-52-037 | E-LIMIT | L50 | Anchoring proves where a finding lands, not whether it is correct; correctness remains G-5’s deterministic vouch/failing-test responsibility. |
| H005-52-038 | E-DEP/INT | L52-L57 | G-5 consumes anchors, C-7 supplies IDs, F-4/F-1/C-1 supply vouches, E-41/C-40 consume later; no new doctrine/delta family is created. |
| H005-52-039 | E-DATA/INT | L32-L37 | Implementation creates `review_anchor.rs`, registers module/types, adds one full SSB lock/round-trip, adds one G-5 pass, and adds cockpit/Tauri join. |
| H005-52-040 | A-DET | L10, L23 | Duplicate guard says `>1` match becomes Ambiguous, while proximity is also a tie-breaker; ordering (scope/proximity narrowing before or after duplicate count) is not fully formalized. |
| H005-52-041 | A-DATA | L17, L42 | Plan says H-52 anchors F-4/F-1/C-1 rows to hunks but specifies `ReviewFinding.claimed_vouch`; exact deterministic mapping from each existing vouch row to snippet/finding is not defined. |
| H005-52-042 | A-DATA | L9 versus L44 | External typed reasons are listed as binary/user_exclude/unsupported_ext/default_path/deleted, while the stub ban calls oversize a typed `Skipped` ExcludeReason; the exact enum/state representation for oversize is unresolved. |
| H005-52-043 | C-DATA | L23, L25 versus L45 | `AnchorResult` has a full `new_line_range`, but the declared event payload stores only `new_line_start`; observable acceptance nevertheless expects the row to carry `new_line_range`, so end-line persistence must be reconciled. |

## Cross-file interactions and bounded inferences

These rows are deliberately separate from explicit source requirements.

| Atomic ID | Class/domain | Evidence atoms | Interaction or inference |
|---|---|---|---|
| H005-I-001 | I-INT/OBS | H005-04-017, H005-04-029, H005-10-023 | H-4 and H-10 both depend on off-agent UI evidence owned by another surface (H-9 token cockpit; Canvas HUD/Table); completion instrumentation must be wired before either can honestly claim success. |
| H005-I-002 | I-DET/INT | H005-04-009, H005-04-029 | Local ToolSearch emulation changes the tool-prefix representation, so H-2 cache-prefix validation must test both Anthropic-native and custom-base-URL paths. |
| H005-I-003 | I-SEC/FLOW | H005-21-004, H005-25-016, H005-47-015 | Trust gates form increasing-cost layers: cheap static H-20, pre-promotion twice-run H-21, result-report H-25, and loop-eval H-47; moving any later gate earlier would change stated cost semantics. |
| H005-I-004 | I-DATA/INT | H005-21-011, H005-25-030, H005-31-039, H005-44-011, H005-47-012, H005-48-010, H005-52-015 | Every plan’s schema number is a dated snapshot; ship-order-cumulative reread is mandatory and no literal baseline in this chunk is globally authoritative. |
| H005-I-005 | I-SEC/INT | H005-31-029, H005-31-038, H005-52-012 | Both memory and review anchoring structurally remove model authority over trust-critical data: protected prompts receive no preferences, journal events omit values, and findings expose no line/verdict. |
| H005-I-006 | I-FAIL/INT | H005-44-024, H005-47-009, H005-47-032 | H-47’s indistinguishability and deterministic-confirmation rules are required to keep H-44’s canaries from becoming a recognizable/sandbaggable hard proxy. |
| H005-I-007 | I-FLOW/DEP | H005-48-003, H005-48-004, H005-47-007 | The bandit cannot consume a score until H-47 has admitted a budget-legal eval and H-45/H-43 have produced a real aggregate; reward provenance is a strict upstream dependency chain. |
| H005-I-008 | I-DATA/INT | H005-52-016, H005-52-018, H005-52-037 | H-52 affects G-5’s denominator and vouch attribution but must not alter finding correctness; anchor fallback should conservatively increase attention rather than vouch uncertainty away. |
| H005-I-009 | I-FAIL/DET | H005-10-013, H005-21-014, H005-25-028, H005-52-036 | Four plans encode honest degradation instead of false certainty: suspected-only dead code, tolerance-bounded FP, VLM-valid-not-numeric-proof, and file fallback. |
| H005-I-010 | I-TEST/OBS | H005-21-024, H005-52-031 | Both H-21 and H-52 explicitly require ≥3 real-run error-rate bounds before default-on; default-on code paths cannot be promoted solely from deterministic fixtures. |

## Summary-index additions and corrections

Orientation anchors are the H-4, H-10, H-21, H-25, H-31, H-44, H-47, H-48, and H-52 sections of `08-H-SUMMARY_INDEX.discovery.md`. Each delta below points to deep atoms; it adds no duplicate atom.

### Material additions absent or materially compressed in the summary

| Delta ID | Source | Deep atoms | Addition to summary-level discovery |
|---|---|---|---|
| DELTA-005-001 | H-4 | H005-04-008, H005-04-010, H005-04-022, H005-04-029 | Record Tool Search protocol limits/BM25 identifier, gateway 400 failure, explicit no-new-SSB decision, and cache-invalidation interaction. |
| DELTA-005-002 | H-4 | H005-04-033, H005-04-034 | Record unresolved second scoped beta and the nontrivial final catalog-count calculation. |
| DELTA-005-003 | H-10 | H005-10-009, H005-10-016-H005-10-020 | Add dependency registration, exact LOD thresholds, SCC cycle pass, current caps, and store-isolation state. |
| DELTA-005-004 | H-10 | H005-10-015, H005-10-031, H005-10-032 | Add suspected-only degradation when coverage is absent, re-run B0 on larger corpora, and measure dead-code FP rate. |
| DELTA-005-005 | H-21 | H005-21-009-H005-21-014, H005-21-024-H005-21-025 | Add missing-rerun fail-closed rule, event evidence fields/freeze, deferred live leg, ≥3-pipeline FP bound, and 2× pre-promotion cost placement. |
| DELTA-005-006 | H-25 | H005-25-008-H005-25-010, H005-25-026-H005-25-028 | Add missing-output failure, frozen safe parser grammar, orchestrator-only capture provenance, and numeric-first/VLM visual boundary. |
| DELTA-005-007 | H-31 | H005-31-005-H005-31-019 | Add exact fact thresholds, dispatcher transitions, contradiction/decay state, full retrieval filter behavior, and precise absence of prompt slot. |
| DELTA-005-008 | H-31 | H005-31-021, H005-31-030-H005-31-031, H005-31-044-H005-31-050 | Add N=2 proposal rule, exact allowlist metadata/freeze procedure, unverified citation warning, personal-agent non-goal, API conflict, and drawer/history ambiguity. |
| DELTA-005-009 | H-44 | H005-44-008-H005-44-011, H005-44-024-H005-44-030 | Add FP rate/payload/kinds, same-variant roll-up rule, live-sweep tiering, sandbagging/isomorph caveats, and row representation ambiguity. |
| DELTA-005-010 | H-47 | H005-47-029-H005-47-033 | Add the entire J-11/S-8c amendment: Git-history, environment-cache, network/command, advisory trace-audit bypass modes and two-layer doctrine. |
| DELTA-005-011 | H-47 | H005-47-010-H005-47-012, H005-47-026-H005-47-028 | Add complete event payload/schema accounting, starvation risk, templated-only limit, and fixture/live dependency fallback. |
| DELTA-005-012 | H-48 | H005-48-009-H005-48-010, H005-48-022-H005-48-028 | Add conditional telemetry/UI, fixed-fixture-only discount tuning, open reward-to-Beta mapping/selector choice, and convergence overclaim conflict. |
| DELTA-005-013 | H-52 | H005-52-012-H005-52-018 | Add no-verdict schema, exact event payload/every-attempt lifecycle, typed exclusion reasons, skip-not-truncate, and conservative DSR fallback behavior. |
| DELTA-005-014 | H-52 | H005-52-030-H005-52-043 | Add zero-row no-op, plan-asserted caveat, directional benchmark limit, precision-not-correctness boundary, and unresolved matching/join/exclusion/range semantics. |

### Corrections or qualifications to summary wording

| Delta ID | Source | Deep atoms | Correction/qualification |
|---|---|---|---|
| DELTA-005-015 | H-4 | H005-04-016 | Retain corrected ~77K→8.7K and explicitly keep both savings and accuracy as borrowed, not ELAI-measured. |
| DELTA-005-016 | H-10 | H005-10-003-H005-10-004 | “Code graph” is not wholly new: call graph is already on canvas; persistent PageRank/call-edge substrate and new shell behaviors are the missing pieces. |
| DELTA-005-017 | H-21 | H005-21-001, H005-21-012 | “Run in a frozen Tier-2 runner” is tiered: deterministic fixture now; live strategy checkbox deferred, while Tier-1 diff is the done gate. |
| DELTA-005-018 | H-31 | H005-31-011-H005-31-014, H005-31-047 | Dispatcher capture is wire-dark even though contradiction reconciliation is live; summary should not collapse those distinct write-side paths, and exact `dispatch_one`/`dispatch_batch` API conflicts. |
| DELTA-005-019 | H-44 | H005-44-006, H005-44-025 | Isomorph mismatch is not itself a block; two-family confirmation is mandatory. |
| DELTA-005-020 | H-47 | H005-47-032, H005-47-038 | LLM trace audit is advisory-only and Thresholdout’s literature description is not an implementation guarantee. |
| DELTA-005-021 | H-48 | H005-48-028 | Replace “un-gameable/provably converge” conclusion with measured, assumption-bounded selection language. |
| DELTA-005-022 | H-52 | H005-52-032, H005-52-037 | All ELAI consumer anchors are plan-asserted and anchoring proves location only, never finding truth. |

## Ambiguity/conflict register

All ambiguities/conflicts are already atomic above; this index prevents them from being mistaken for requirements.

- H-4: scoped beta emission and live catalog integer — H005-04-033, H005-04-034.
- H-10: renderer density rule and PageRank owner — H005-10-036, H005-10-037.
- H-21: truncation/tolerance algorithm — H005-21-028.
- H-25: safe parser grammar — H005-25-031.
- H-31: dispatcher API conflict, fact classification, slot numbering, drawer history — H005-31-047 through H005-31-050.
- H-44: report struct versus event-row wording — H005-44-030.
- H-47: date-window semantics, command-policy details, borrowed DP qualification — H005-47-036 through H005-47-038.
- H-48: reward mapping, selector composition, conditional telemetry, convergence overclaim — H005-48-025 through H005-48-028.
- H-52: duplicate/proximity ordering, vouch-to-snippet join, exclusion enum, and event-range conflict — H005-52-040 through H005-52-043.

## Zero-omission / zero-duplicate audit

- Source coverage: 9/9 manifest files; 1,451/1,451 lines; 149,987/149,987 bytes; all nine SHA-256 values exact.
- Structural coverage per source: header/status/objective; context; existing state; tasks/revised scope; wiring/production reachability; flags/defaults; schema/event lifecycle; stub bans/failure behavior; observable effects; deterministic checks; acceptance; tests; implementation inventory; risks/open questions; dependencies/next; and H-47 amendment were inspected and represented.
- Behavioral coverage: every distinct explicit capability, state, data/lifecycle rule, config/default, integration/event, security/privacy rule, failure/fallback, determinism rule, observable artifact, acceptance gate, constraint/non-goal, dependency, and deferral was assigned one atomic ID. Pure bibliographic repetition and LOC estimates were not promoted to features; status/effort that changes execution tier was retained.
- Duplicate policy: repeated source clauses were folded into one atom with combined anchors. Summary deltas and interaction rows refer to atoms rather than restating them as new requirements.
- Classification audit: explicit requirements/states are separated from bounded inference (`H005-I-*`), ambiguity (`A-*`), and conflict (`C-*`). No inferred row is counted as source-mandated behavior.
- Automated ID audit and final line/atom counts are recorded below after generation; canonical output digest excludes only its own final digest line.

## Final counts

- Files processed: **9**.
- Source lines processed: **1,451**.
- Source bytes processed: **149,987**.
- Atomic feature/behavior rows: **332** = 300 explicit + 18 ambiguous + 4 conflicting + 10 inferred; automated uniqueness check: **332/332 unique, 0 duplicate IDs**.
- Output logical lines: **499**.
- Output SHA-256 (canonical content excluding the final digest line): **`25d456a7e60e4320d68b60fa10d1a1366e52556a53ff1dc01b835a4426168a0c`**.
