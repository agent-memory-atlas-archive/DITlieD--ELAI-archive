E-40 — Symbol-graph tooling — decisions
session: s-25120 / session 2 of the 4-way concurrent cluster
date: 2026-05-25
status: SHIPPED (WIRE-DARK avoidance via scip_cmd production caller)
SCHEMA bump: 133 -> 134 (atomic +3 SSB variants)

DECISION-1: orchestrator-rooted symbol-graph module is a DISTINCT
surface, NOT an alias or extension of either existing symbol-graph
surface.
  - elai_knowledge::symbol_graph::cache::SymbolGraphCache is the B-29
    SQLite-rooted cache (27 name-shards + 256 reverse-shards + 500-file
    LRU on rusqlite::Connection). Stays the canonical persistence
    layer for the existing C-24 KnowledgeDrawer + B-29 traversal
    surface.
  - elai_knowledge::pagerank::DkgSnapshot::compute_repo_map is the C-38
    DKG-snapshot-rooted whole-repo PageRank (Aider repo-map style).
    Stays the canonical Stage-2 retrieval signal.
  - NEW elai_orchestrator::symbol_graph::SymbolGraphCache is the
    JSON-rooted (`.claude/tools/elai_graph.json`) read-only view with a
    BTreeMap-backed LRU. Distinct module path; both surfaces coexist.

Rationale: the cross-check report mis-classified the existing pagerank
module as PARTIAL for E-40 — context-compiler verified the surfaces
are structurally orthogonal (DKG-snapshot rooted vs. JSON-graph-rooted),
which the executor honored by creating a sibling module rather than
extending the DKG-rooted one.

DECISION-2: SCHEMA bump 133 -> 134 with 3 SSB variants in one atomic
commit per the SchemaBumpPerVariantPair invariant.
  - SymbolGraphQueried — one row per find_defs/refs/callers/callees
    invocation. Hashes the query kind + symbol; carries result_count
    + ts_ms.
  - SymbolGraphCacheLoaded — one row per SymbolGraphCache::load mount.
    Hashes the graph_path; carries node_count + ts_ms.
  - LspOracleConsulted — one row per oracle_lookup invocation. Hashes
    language + symbol + oracle_kind; on-demand-only discipline (>5/
    session = bug).

Cascade sites: enum decl + canonical_name (as_sql_str) + from_str +
SsbPayload + payload_action_type + SQL CHECK + test_payload + ALL_VARIANTS
+ exhaustive_match_guard + golden file (378 -> 381 entries).

DECISION-3: RuntimeBackend::Lsp variant added rather than a new
PerRoleRuntime AgentRole.
  - Context-compiler flagged the E37_AGENTROLE_CASCADE_PATTERN — adding
    a new AgentRole variant fans out to ~8 match-arm sites (agent_pool
    + bsp + ipc_gateway + routing/selective + per_role_runtime + the C-45
    contract + reasoning_surface_contract). Lsp is a SUBPROCESS adapter
    (rust-analyzer / pylsp / etc.), not an Anthropic-role completion
    surface — it does not belong in the PerRoleRuntime role enum.
  - The RuntimeBackend slot lets `/runtime list` surface the lsp slot +
    gives the cost-model registry a stable id to attribute toolchain
    subprocess time against. `create_runtime(Lsp)` returns
    `NotImplemented` like Ppm — the orchestrator dispatches the actual
    oracle via `symbol_graph::lsp_oracle::oracle_lookup` directly via
    tokio::process::Command, NOT through the RuntimeAdapter trait.

DECISION-4: lsp_oracle::oracle_lookup is RUNPOD-PENDING.
  - Live integration tests against running rust-analyzer / pylsp /
    typescript-language-server / gopls require an installed toolchain
    on the host, which the CI lane does not provide.
  - The function returns a synthetic `OracleAnswer { invoked: false, .. }`
    when the toolchain is absent. Callers can treat this as a soft
    signal (degrade gracefully) rather than a hard error.
  - The shape ships now so the dashboard SymbolGraphWindow + the
    contract test (e40_f7b) can pin the surface; a follow-on patch-N
    completes the JSON-RPC handshake when B-30's LSP client surface
    ships.

DECISION-5: CLI scip_cmd is the WIRE-DARK guard.
  - Every new function in `symbol_graph::{cache,query,pagerank,scip_import,lsp_oracle}`
    is reachable from `crates/cli/src/scip_cmd.rs::run` — find_defs /
    find_refs / find_callers / find_callees / personalized_top_k /
    import_scip_indices / oracle_lookup (via oracle_probe helper) +
    OracleKind::canonical_binary (via canonical_oracle_binary helper).
  - Defends against E37_WIRE_DARK_GAP_PATTERN — library surface (6
    modules + 7+ functions) cannot ship without production wiring.
  - The contract test e40_f7 asserts the WIRE-DARK guard structurally
    by importing the same names as scip_cmd.rs and touching each one;
    if either surface drifts, the build breaks.

DECISION-6: 4-paper + 1-SDK cite cluster deferred per the s-25120
DEFERRAL discipline.
  - arXiv:2211.01224 (Stack Graphs), arXiv:2406.07003 (GraphCoder),
    arXiv:2408.03910 (CodexGraph), arXiv:2509.25257 (RANGER), and the
    Glean Angle SDK source-grounding are NOT inline-cited in source.
    They are noted as DESIGN-INTENT motivation only.
  - See workflow/sessions/s-25120/mistakes.md cluster #6 for the
    deferred-not-skipped block + the follow-on paper-ingestor mechanical
    task list. This is the 6th DEFERRAL CLUSTER entry overall.

DECISION-7: dashboard surface = new SymbolGraphWindow rather than a
KnowledgeDrawer extension.
  - C-24 KnowledgeDrawer IMPACT ANALYSIS panel is itself PLANNED per
    the cross-check report. Coupling E-40 to a PLANNED surface risks
    audit BLOCK on a precondition that has not yet shipped.
  - Per the dashboard precedent, complex new operator surfaces ship as
    a sibling Window (mounted in WindowHost via type=`symbol_graph`).
    The Panel.svelte WINDOW_TYPES + WINDOW_LABELS were updated
    simultaneously to avoid the E-12 / C-40 svelte-check exhaustiveness
    BLOCK lesson.

KNOWN-RESIDUAL (intentional, follow-on):
  - oracle_lookup live JSON-RPC handshake — currently does `--version`
    liveness probe only. Full handshake lands in patch-N alongside the
    B-30 LSP client surface.
  - import_scip_indices does NOT yet parse SCIP protobuf (the `scip`
    crate is not pulled in until anti-slopsquatting verification runs
    in a follow-on patch). The signature is stable; the cache argument
    is RESERVED so the follow-on can hydrate nodes without changing
    callers.
  - symbol_graph_vec 4th-list RRF integration is forward-compatible
    with E-36 vision_vec — currently NOT wired into knowledge::pipeline
    (that wiring lives in E-17 SP2 follow-on per E13_LEAN_SCOPE_DISCIPLINE).
  - C-24 KnowledgeDrawer DSL input form — NOT added in this slice.
    SymbolGraphWindow is the canonical operator surface; the
    KnowledgeDrawer extension lands when C-24 ships.

Files touched (net new + edits, all under .elai_cc/ + workflow/):
  + crates/orchestrator/src/symbol_graph/{mod,cache,query,pagerank,scip_import,lsp_oracle}.rs
  + crates/cli/src/scip_cmd.rs
  + crates/dashboard-v2/src-tauri/src/commands_e40.rs
  + crates/dashboard-v2/src/lib/windows/SymbolGraphWindow.svelte
  + crates/contracts/tests/e40_symbol_graph_tools_contract.rs
  M  crates/orchestrator/src/lib.rs            (mod symbol_graph)
  M  crates/orchestrator/src/ssb.rs            (SCHEMA 133->134 + 3 variants + cascade)
  M  crates/orchestrator/tests/action_type_lock.rs + golden
  M  crates/providers/src/runtime/types.rs     (Lsp variant)
  M  crates/providers/src/runtime/mod.rs       (resolve + create_runtime sentinel)
  M  crates/providers/src/runtime/cli_common.rs (Lsp non-cli arm)
  M  crates/contracts/tests/c44_composer2_backend_contract.rs (all.len 15->16)
  M  crates/cli/src/args.rs                    (CliAction::SymbolGraph + parser)
  M  crates/cli/src/main.rs                    (mod scip_cmd + dispatcher arm)
  M  crates/dashboard-v2/src-tauri/src/lib.rs  (mod + invoke_handler entries)
  M  crates/dashboard-v2/src/lib/api/types.ts  (WindowType + 'symbol_graph')
  M  crates/dashboard-v2/src/lib/layout/Panel.svelte (WINDOW_TYPES + WINDOW_LABELS)
  M  crates/dashboard-v2/src/lib/layout/WindowHost.svelte (route + SCOPE_LABEL)
  M  crates/dashboard-v2/src/lib/api/stub.ts   (3 sym graph stub fns + DTOs)
  M  workflow/sessions/s-25120/mistakes.md     (cluster #6 deferral)
  + workflow/reasoning/E-40-decisions.md       (this file)

POST-AUDIT ADDITIONS (Tranche B-3, shipped after s-plan-audit-2026-06-04):
  + crates/orchestrator/src/symbol_graph/scope_graph.rs  (Task 2, 740 lines)
  + crates/orchestrator/src/symbol_graph/mod.rs          (scope_graph_emit_smoke in-crate smoke)
  + .elai_cc/scope_graph_rules/rust.toml
  + .elai_cc/scope_graph_rules/python.toml
  + .elai_cc/scope_graph_rules/typescript.toml
  + .elai_cc/scope_graph_rules/go.toml
  M  wire-dark-allowlist.toml                            (ELAI_SYMBOL_GRAPH_5TH_RRF_ENABLE R9 row)
  + crates/contracts/tests/e40_symbol_graph_contract.rs
  + crates/contracts/tests/e40_symbol_graph_5th_rrf_on_path.rs

CITATION ERRATA (Task 10 deliverable, 2026-06-06):

The original E-40 plan cited arXiv:2506.11419 under "FocalAD" as prior art
for code-change impact analysis. This citation is INCORRECT.

arXiv:2506.11419 is "FocalAD: Ego-Local-Agents Interactor for Autonomous
Driving" — a multi-agent trajectory-prediction paper in the autonomous-driving
domain. It has no relevance to symbol-graph reachability, SCIP import, or
code-change impact analysis. The paper describes local and global interactive
focal agents for predicting vehicle trajectories in traffic, not code
analysis tooling.

The correct references for the scope-graph + SCIP-reachability design space
are:

  arXiv:2406.07003 — GraphCoder (ASE 2024). Code change intention retrieval
    via graph-of-code for repository-level code generation. Directly motivates
    the use of SCIP symbol edges to find change-relevant context; the
    "code-change impact analysis" angle in the E-40 plan refers to this work.

  arXiv:2408.03910 — CodexGraph (NAACL 2025). LLM-database integration
    using a graph database constructed from repository-wide static analysis;
    the SymbolGraphCache BTreeMap-backed design mirrors the queryable-graph
    interface described here.

  arXiv:2509.25257 — RANGER. Retrieval-augmented generation for code
    repositories using graph-structured symbol relationships; grounds the
    5th-list RRF extension (ELAI_SYMBOL_GRAPH_5TH_RRF_ENABLE) and the
    scope-graph ranking signal.

  [SLOT 4 — NO VERIFIED CITATION]: A genuine paper covering repository-level
    code impact analysis via symbol disambiguation was not found during
    WebSearch on 2026-06-06. arXiv:2511.00262 was removed after verification
    revealed it is "LLM-Driven Cost-Effective Requirements Change Impact
    Analysis" (ProReFiCIA) — requirements-text NLP, not code symbol graphs.
    No replacement is cited here until a verified paper with matching abstract
    is found. The ScopeGraphResolved SSB variant and scope_graph.rs
    resolve_name disambiguation stand without this citation; the design is
    grounded by the three verified papers above (GraphCoder, CodexGraph,
    RANGER).

The shipped code contains ZERO references to arXiv:2506.11419 as an impact-
analysis citation. The errata closes DECISION-6's "4-paper cite cluster
deferred" note for the four references above. The Glean Angle SDK source-
grounding from that deferral remains pending (no scip crate pulled in yet).

ANTI-SLOPSQUATTING VERIFICATION RECORD (scip crate, deferred):

The `scip` crate (protobuf decode for SCIP index files) has NOT been added
as a dependency. The scip_import.rs module ships a homegrown byte-level path
reader that decodes only the occurrence and symbol fields it needs (no full
protobuf parse).

Before pulling the `scip` crate, the mandatory verification steps are:
  1. `cargo search scip` or crates.io lookup to confirm exact crate name,
     version, and publish date (must be published in last 24 months).
  2. Verify the crate is the official protobuf-compiled output of the SCIP
     spec (github.com/sourcegraph/scip), not a typo-squatted substitute.
  3. Confirm license compatibility (Apache-2.0 / MIT).
  4. User must explicitly confirm the dep add before the import line is
     added (per CLAUDE.md "no new dependencies without confirmation").

This is documented here per DECISION-6 cluster #6 and the plan-status
REMAINING note. Until the above steps run and the user confirms, the
scip_import.rs homegrown reader is the production path and the `scip`
crate is explicitly NOT a dependency.
