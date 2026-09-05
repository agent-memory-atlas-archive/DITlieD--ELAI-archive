# B-29: Symbol-Level Impact Analysis API
Objective: expose blast-radius / call-flow / symbol-context / symbol-list as
four agent-callable tools backed by a sharded-index, LRU-bounded
SymbolGraphCache so the Auditor can pre-check edits, the Designer can ground
decompositions in real entry points, and the Researcher can answer "what
breaks if I change X" without loading the whole DKG.
Estimated effort: 2-3 sessions (~12-18 hours).
Pre-requisites: B-1 (path unification — the DKG must read from one place).
Recommended sibling: B-34 (entry-point detection consumes the same cache).
Parallel-safe with: B-30, B-31, B-32, B-35.

## Context

Per A-44 (Project Indexing Wiring DONE) the DKG is built and persisted at
`.elai_cc/.elai_cc/indexes/dkg.db` with 8075 nodes / 4148 edges on the
ELAI repo itself. Per A-48 (DKG Data Flow Edges DONE) the schema includes
`dkg_data_flow` with PARAMETER / RETURN / ASSIGNMENT / FIELD_ACCESS edges
as the foundation for blast-radius computation.

What's missing: the four caller-facing tools that turn graph data into
agent-actionable answers. Today the Auditor cannot answer "what breaks if
this Edit lands". The Designer cannot answer "where does this codebase
actually start". The Researcher cannot answer "who calls bcryptCompare".
The DKG is a database without a query layer for symbol-level traversal —
PageRank uses node-level data, but no production code path exposes
hop-banded reverse closure or forward call flow.

SocratiCode (F:\.ELAI_workflow\docs\.source\SocratiCode-main\SocratiCode-main)
demonstrates the pattern at production quality. Their
`src/services/graph-impact.ts` (270 LOC) implements all four tools backed by
an LRU-bounded `SymbolGraphCache` (`src/services/symbol-graph-cache.ts`,
LOC 1-200) with a 27-shard name index (a-z + `_`), a 256-shard
reverse-call file index (single-byte SHA1 prefix), and a 500-file LRU for
per-file payloads. Critical invariant: no query loads every symbol or
every edge into memory. Their cross-file resolver
(`src/services/graph-symbol-resolution.ts:26-94`) populates each call
edge with `calleeCandidates` and a confidence tier — local / unique /
multiple-candidates / unresolved — that the Auditor uses to decide
whether to escalate.

This B-step ports the algorithm and the four tools, NOT the source code
(SocratiCode is AGPL-3.0). Reimplementation in Rust with ELAI's existing
DKG SQLite schema as the source of truth.

Outcome: Auditor calls `compute_impact_radius` automatically before any
PostToolUse Write/Edit. Designer's Recursive Decomposition (4D.7-4D.10)
gets `compute_call_flow` for grounding. Researcher's /research command
exposes `symbol_context` to the Oracle pipeline. The dashboard
KnowledgeDrawer renders blast-radius hops live.

## External grounding (top-k local-focus principle)

The "BFS over reverse-call index, hop-banded, depth-clamped" approach is
empirically validated outside the dev-tooling domain by FocalAD's
Ego-Local-Agents Interactor (Liu et al., arXiv:2506.11419, Jun 2025).
FocalAD argues — and measures, on nuScenes / Adv-nuScenes / Bench2Drive —
that focusing model attention on the Top-k decision-critical local
neighbors strictly outperforms globally-aggregated features for
downstream decision-making (planning in their case, edit auditing in
ours). Their network scores each neighbor's *interaction influence* and
weights attention by the score; B-29's design maps onto this with the
four `Confidence` tiers (Local / Unique / MultipleCandidates /
Unresolved). This citation is the academic-grounding anchor for the
design choice: "don't dump every transitive caller into the Auditor —
hop-band them and let the Auditor walk only the top of the band."

Future refinement (NOT this B-step, see Side benefits): the
hop-banded output is currently uniform-weighted within a band. FocalAD
suggests an attention-weighted fusion where each caller in a band is
weighted by call-edge confidence — a path B-23 (self-modification) can
explore once B-29's deterministic baseline is shipped.

## Reads (mandatory before coding)

- F:\.ELAI_workflow\docs\.source\SocratiCode-main\SocratiCode-main\src\services\graph-impact.ts
  (FULL — 270 LOC; the four traversal algorithms are here)
- F:\.ELAI_workflow\docs\.source\SocratiCode-main\SocratiCode-main\src\services\symbol-graph-cache.ts
  (FULL — 200 LOC; LRU + sharded loading strategy)
- F:\.ELAI_workflow\docs\.source\SocratiCode-main\SocratiCode-main\src\services\graph-symbol-resolution.ts
  (FULL — 109 LOC; cross-file candidate resolution, 4 confidence tiers)
- F:\.ELAI_workflow\docs\.source\SocratiCode-main\SocratiCode-main\src\services\graph-symbols.ts
  (FULL — extraction patterns; mirror for languages already supported by
  ELAI tree-sitter grammars)
- F:\.ELAI_workflow\docs\.source\SocratiCode-main\SocratiCode-main\src\constants.ts
  lines 130-150 (SYMBOL_NAME_SHARDS=27, SYMBOL_REVERSE_SHARDS=256,
  SYMBOL_FILE_LRU_SIZE=500, MAX_IMPACT_DEPTH=10, MAX_FLOW_DEPTH=10,
  ENTRY_POINT_NAMES per language)
- F:\.ELAI_workflow\.elai_cc\crates\knowledge\src\dkg\ (FULL crate)
- F:\.ELAI_workflow\.elai_cc\crates\knowledge\src\pagerank\mod.rs
  (DKG node iteration pattern reference)
- F:\.ELAI_workflow\docs\specs\guide\steps\phase-1A.txt
  (DKG schema reference)
- F:\.ELAI_workflow\docs\specs\guide\update-implementation-order-summary.txt
  A-48 (DKG Data Flow Edges)

## Tasks

### Phase 1 — Schema and persistence (Session 1, half)

1. Extend `dkg.db` schema (knowledge crate, schema migration v3):
   ```
   CREATE TABLE IF NOT EXISTS symbol_name_index (
     shard_key TEXT NOT NULL,        -- 'a'..'z' or '_'
     symbol_name TEXT NOT NULL,
     refs_msgpack BLOB NOT NULL,     -- Vec<SymbolRef { file, id }>
     PRIMARY KEY (shard_key, symbol_name)
   );
   CREATE TABLE IF NOT EXISTS symbol_reverse_index (
     shard_bucket INTEGER NOT NULL,  -- 0..255 (single-byte SHA1 prefix of callee_file)
     callee_file TEXT NOT NULL,
     caller_files_msgpack BLOB NOT NULL, -- Vec<String>
     PRIMARY KEY (shard_bucket, callee_file)
   );
   CREATE TABLE IF NOT EXISTS symbol_file_payload (
     file TEXT PRIMARY KEY,
     language TEXT NOT NULL,
     content_hash TEXT NOT NULL,     -- blake3, for staleness detection
     symbols_msgpack BLOB NOT NULL,  -- Vec<SymbolNode>
     edges_msgpack BLOB NOT NULL     -- Vec<SymbolEdge>
   );
   CREATE TABLE IF NOT EXISTS symbol_graph_meta (
     project_id TEXT PRIMARY KEY,
     symbol_count INTEGER NOT NULL,
     edge_count INTEGER NOT NULL,
     file_count INTEGER NOT NULL,
     unresolved_edge_pct REAL NOT NULL,
     built_at INTEGER NOT NULL,      -- ClockProvider::now_ms
     schema_version INTEGER NOT NULL
   );
   ```
   No new SQLite dependency — already in workspace.

2. Define types in `crates/knowledge/src/symbol_graph/types.rs`:
   - `SymbolKind` enum: Function, Method, Class, Struct, Trait, Module, Enum, Interface, Type
   - `SymbolNode { id: String, name: String, qualified_name: String, kind: SymbolKind, file: String, line: u32, end_line: u32, language: String }`
   - `SymbolEdge { caller_id: String, callee_name: String, callee_candidates: Vec<String>, confidence: Confidence, call_site: CallSite }`
   - `Confidence` enum: Local, Unique, MultipleCandidates, Unresolved
   - `SymbolRef { file: String, id: String }`
   - `EntryPoint { id: String, name: String, file: String, line: Option<u32>, reason: String }`
   All `#[derive(Serialize, Deserialize)]` via serde + msgpack.

3. Build helpers in `crates/knowledge/src/symbol_graph/sharding.rs`:
   - `name_shard_key(name: &str) -> char` — first lowercase ASCII letter or `_`
   - `reverse_shard_bucket(file: &str) -> u8` — first byte of SHA1(file)
   - `all_name_shard_keys() -> [char; 27]`

### Phase 2 — Extraction and resolution (Session 1, half)

4. Implement `extract_symbols_and_calls(source: &str, language: Language, file: &str) -> ExtractedSymbols` mirroring SocratiCode's
   `graph-symbols.ts` per-language dispatch but using ELAI's existing
   tree-sitter grammars (DO NOT add ast-grep dependency — see B-30).
   Per-language extractors return `ExtractedSymbols { symbols, raw_calls }`
   where raw calls have caller_id + callee_name + call_site only. Module-level
   synthetic `<module>` symbol mandatory so unattributed calls have a caller.

5. Implement cross-file resolver in
   `crates/knowledge/src/symbol_graph/resolution.rs` mirroring SocratiCode's
   four-tier strategy from `graph-symbol-resolution.ts:26-94`:
   - Tier 1 — Local (same file)
   - Tier 2 — Imported (1-hop via DKG file dependencies)
   - Tier 3 — Re-export (2-hop)
   - Confidence assignment: 0 candidates → Unresolved, 1 → Unique,
     >1 → MultipleCandidates
   Mutates `outgoing_calls_by_file` in place.

### Phase 3 — Cache and traversal (Session 2)

6. Implement `SymbolGraphCache` in
   `crates/knowledge/src/symbol_graph/cache.rs`:
   - `meta: SymbolGraphMeta` — eager (tiny)
   - `name_index: OnceCell<BTreeMap<String, Vec<SymbolRef>>>` — eager on
     first symbol-name query, loads all 27 shards in parallel via
     tokio::JoinSet
   - `reverse_file_index: OnceCell<BTreeMap<String, BTreeSet<String>>>` —
     eager on first impact query, loads all 256 shards in parallel
   - `file_payload_lru: LruCache<String, Arc<SymbolGraphFilePayload>>` —
     capacity 500 (mirror SYMBOL_FILE_LRU_SIZE constant)
   - Hand-rolled tiny LRU using `linked_hash_map` or std `BTreeMap` +
     access counter (NO new external dependency without confirmation per
     CLAUDE.md)
   Constructor: `new(project_id: &str, db: &SqliteConnectionPool, lru_capacity: usize) -> Result<Self>`.

7. Implement four traversal functions in
   `crates/knowledge/src/symbol_graph/traversal.rs`:
   - `compute_impact_radius(cache, target, depth) -> ImpactResult`
     - polymorphic on target (file path vs symbol name; detect via
       `looks_like_file_path` mirroring SocratiCode's algorithm)
     - BFS over reverse_file_index
     - hop-banded output: `files_by_depth: BTreeMap<u32, Vec<String>>`
     - `truncated: bool` flag set when stopped at depth limit with more
       callers known
     - depth clamp: max 10 (MAX_IMPACT_DEPTH)
   - `compute_call_flow(cache, entrypoint_id, depth) -> Option<FlowNode>`
     - DFS with cycle detection (visited set)
     - lazy children — only loads payload for next file when descending
     - depth clamp: max 10 (MAX_FLOW_DEPTH)
     - leaf nodes carry `truncated_reason: Option<TruncationReason>`
       (Depth or Cycle)
   - `compute_symbol_context(cache, name, file_hint) -> Vec<SymbolContext>`
     - returns all matching symbols (caller may pass file_hint to
       disambiguate); for each, includes definition + Vec<Caller> +
       Vec<Callee> with confidence
   - `list_symbols(cache, filter) -> Vec<SymbolNode>` where filter is
     `{ file: Option<String>, query: Option<String>, limit: usize }`

8. Cache invalidation: when watcher detects file change (per B-31 watcher
   path), call `cache.invalidate_file(file)` which evicts file_payload_lru
   entry, marks name_index/reverse_file_index for partial rebuild.
   Full rebuild trigger: when changed_file_count > 50 (mirror SocratiCode's
   INCREMENTAL_SYMBOL_THRESHOLD).

### Phase 4 — Build pipeline integration (Session 2-3)

9. Hook into `IndexingService` (orchestrator/src/indexing_service.rs):
   after DKG build completes, call new `build_symbol_graph(project_id,
   project_root, dkg)` which:
   - Iterates files, runs extract_symbols_and_calls per file
   - Calls resolve_call_sites with the file-import graph
   - Builds shard maps in memory (BTreeMap<shard_key, BTreeMap<name, Vec<ref>>>)
   - Persists shards via msgpack to `symbol_name_index`,
     `symbol_reverse_index`, `symbol_file_payload`, `symbol_graph_meta`
   - Logs summary: `{ files, symbols, edges, unresolvedPct }` like
     SocratiCode's `Symbol graph persisted` info-line

10. Incremental update: in `update_project_index`, after file-import
    graph rebuilds, if `changed_file_count <= INCREMENTAL_SYMBOL_THRESHOLD`,
    call `update_changed_files_symbol_graph(project_id, changed_paths,
    removed_paths)` which patches affected payloads + reshards only the
    affected name buckets and reverse buckets. Otherwise full rebuild.

### Phase 5 — Tool dispatcher exposure (Session 3)

11. Add four tool dispatcher entries in
    `crates/orchestrator/src/dispatcher.rs`:
    - `symbol_impact { target: String, depth: Option<u32> }` →
      `compute_impact_radius`
    - `symbol_flow { entrypoint: Option<String>, file: Option<String>, depth: Option<u32> }`
      → `compute_call_flow` (no entrypoint = list candidates via
      `entry_points` from B-34 if available, else error)
    - `symbol_context { name: String, file: Option<String> }` →
      `compute_symbol_context`
    - `symbol_list { file: Option<String>, query: Option<String>, limit: Option<usize> }`
      → `list_symbols`

12. Auto-inject Auditor pre-edit check in
    `crates/orchestrator/src/task_runner.rs` PostToolUse hook on Write/Edit:
    when `changed_files.len() > 0`, call `compute_impact_radius` for each
    changed file at depth=2; if `total_files > 0`, append summary to
    auditor context as `IMPACT_SUMMARY` block. Auditor renders this in
    its briefing.

13. Wire SSB action types: add `symbol_query` action_type to ssb.rs enum,
    payload `{ tool: String, target: String, hop_count: u32, total_files: u32, latency_ms: u32 }`.
    Append per query for dashboard ActivityFeed.

14. Dashboard: add `KnowledgeDrawer` panel "Impact Analysis" with:
    - Real-time `unresolvedEdgePct` gauge
    - Last impact query: target, depth, total files, hop band table
    - "Run /symbol_impact target=..." input form (delegates to
      `submit_task` Tauri command with synthetic intent)

## WIRING_CONTRACT

produces_symbols:
  - knowledge::symbol_graph::cache::SymbolGraphCache
  - knowledge::symbol_graph::traversal::compute_impact_radius
  - knowledge::symbol_graph::traversal::compute_call_flow
  - knowledge::symbol_graph::traversal::compute_symbol_context
  - knowledge::symbol_graph::traversal::list_symbols
  - knowledge::symbol_graph::resolution::resolve_call_sites
  - knowledge::symbol_graph::extraction::extract_symbols_and_calls
  - knowledge::symbol_graph::sharding::name_shard_key
  - knowledge::symbol_graph::sharding::reverse_shard_bucket
  - orchestrator::ssb::action_type::SsbActionType::SymbolQuery
  - ipc_types::TOOL_SYMBOL_IMPACT
  - ipc_types::TOOL_SYMBOL_FLOW
  - ipc_types::TOOL_SYMBOL_CONTEXT
  - ipc_types::TOOL_SYMBOL_LIST
consumes_symbols:
  - orchestrator::indexing_service::IndexingService::build_index → calls
    build_symbol_graph
  - orchestrator::indexing_service::IndexingService::update_index → calls
    update_changed_files_symbol_graph or full rebuild
  - orchestrator::task_runner::run_post_tool_use_hooks → calls
    compute_impact_radius for changed files (depth=2)
  - dashboard::commands::get_impact_analysis → reads symbol_graph_meta and
    formats hop bands
  - cli::main::handle_research_command → uses compute_symbol_context for
    /research grounding
reachable_from:
  - cli::main::main (via dispatcher tool calls)
  - dashboard::lib::run (via Tauri commands)
  - orchestrator::task_runner::drive_task (auto-injected pre-edit check)
stub_ban:
  - cache.reverse_file_index_loaded becomes true after first impact query
    (no zero-result fallback when meta says edges > 0)
  - SymbolGraphCache.file_payload_lru.capacity == 500 (not 0, not None)
  - SymbolGraphMeta.unresolved_edge_pct must be < 50.0 on the ELAI repo
    itself (sanity check; spec says <30 ideal but MultipleCandidates is
    expected to be elevated on dynamic-call codebases)
frontend_pair: B-33 (Interactive Decomposition Viewer consumes the same
                cache for symbol-mode rendering)
observable_effect:
  trigger: cd /d F:\.ELAI_workflow\.elai_cc && set ELAI_SSB_SMOKE_DB=F:\.ELAI_workflow\.elai_cc\sessions\ssb.db && cargo test --package elai-orchestrator --test symbol_query_dispatch symbol_query_dispatch_writes_ssb_row_for_each_tool -- --nocapture
  expect_ssb: symbol_query
  expect_artifact: stdout contains "Hop 1 (N files):" block followed by
                   real ELAI file paths from .elai_cc/crates/

## DETERMINISTIC_CHECK

```bash
# 1. Schema migration applied
sqlite3 .elai_cc/.elai_cc/indexes/dkg.db ".tables" | grep -E "symbol_name_index|symbol_reverse_index|symbol_file_payload|symbol_graph_meta"
# Expected: 4 lines, one per new table.

# 2. Symbol graph populated for the ELAI repo itself
sqlite3 .elai_cc/.elai_cc/indexes/dkg.db "SELECT symbol_count, edge_count, unresolved_edge_pct FROM symbol_graph_meta;"
# Expected: symbol_count >= 5000 (Rust workspace symbols)
#           edge_count >= 8000
#           unresolved_edge_pct < 50.0 (sanity)

# 3. Reverse index has entries for known multi-caller files
sqlite3 .elai_cc/.elai_cc/indexes/dkg.db "SELECT COUNT(*) FROM symbol_reverse_index WHERE callee_file LIKE '%core/src/clock.rs%';"
# Expected: >= 1 (clock module is called from many places)

# 4. CLI symbol_impact returns real results
elai.bat repl "/symbol_impact target=ClockProvider"
# Expected stdout: Blast radius for symbol: ClockProvider
#                  Depth: 3    Total impacted files: >= 20
#                  Hop 1 (N files): listing real .rs paths

# 5. CLI symbol_flow returns real call tree
elai.bat repl "/symbol_flow entrypoint=main file=crates/cli/src/main.rs"
# Expected stdout: Call flow from main (...:line)
#                  └── ... tree with real callee names

# 6. Auditor auto-includes IMPACT_SUMMARY in briefing
elai.bat repl "/task 'rename function compute_with_profile to compute_with_caps in context_budget.rs'"
# Wait for auditor invocation; check workflow logs:
grep "IMPACT_SUMMARY" workflow/sessions/*/auditor-briefing.log
# Expected: at least one IMPACT_SUMMARY block listing >= 5 dependent files

# 7. Cache size bounded
# After 1000 file_payload accesses, file_payload_lru.size <= 500
# (covered by unit test test_lru_capacity_bounded)
```

## ACCEPTANCE

- [ ] All four traversal functions implemented in
      `crates/knowledge/src/symbol_graph/traversal.rs` with property tests
      covering depth bounds, cycle safety, and target polymorphism
- [ ] `SymbolGraphCache` persists 27 name shards + 256 reverse shards on
      ELAI repo build with `unresolved_edge_pct < 50.0`
- [ ] Schema migration v3 idempotent (run twice = no error)
- [ ] IndexingService builds symbol graph automatically on full index
- [ ] Watcher path triggers incremental update when
      changed_file_count <= 50, full rebuild above
- [ ] Four dispatcher tools registered, callable from CLI / dashboard /
      orchestrator IPC
- [ ] Auditor PostToolUse hook injects IMPACT_SUMMARY for changed files
- [ ] SSB SymbolQuery action emitted per tool call with latency
- [ ] Dashboard KnowledgeDrawer Impact Analysis panel renders real data
- [ ] Zero `.unwrap()` in non-test source per CLAUDE.md hard rule
- [ ] No new external dependencies without explicit user confirmation
      (use std + workspace deps only)

## Tests to add

- `crates/knowledge/tests/symbol_graph_extraction.rs` — per-language
  fixtures (Rust, Python, TS, Go, JVM): assert symbol count, edge count,
  qualified names match expected
- `crates/knowledge/tests/symbol_graph_resolution.rs` — fixture with
  Local + Imported + Re-export call sites; assert correct confidence
  tier per edge
- `crates/knowledge/tests/symbol_graph_cache.rs` — LRU capacity bounded,
  shard-loading parallelism, invalidate_file evicts payload but preserves
  index until rebuild
- `crates/knowledge/tests/symbol_graph_traversal.rs` — proptest:
  - impact_radius depth limit always honored
  - call_flow cycle detection terminates on cyclic graphs
  - symbol_context returns all matches (not first only)
  - list_symbols respects limit
- `crates/orchestrator/tests/symbol_query_dispatch.rs` — end-to-end via
  IPC dispatcher
- `tests/contracts/symbol_graph_contract.rs` — meta sanity bounds, schema
  version match

## Files touched

- CREATE: `crates/knowledge/src/symbol_graph/mod.rs`
- CREATE: `crates/knowledge/src/symbol_graph/types.rs`
- CREATE: `crates/knowledge/src/symbol_graph/sharding.rs`
- CREATE: `crates/knowledge/src/symbol_graph/extraction.rs`
- CREATE: `crates/knowledge/src/symbol_graph/resolution.rs`
- CREATE: `crates/knowledge/src/symbol_graph/cache.rs`
- CREATE: `crates/knowledge/src/symbol_graph/traversal.rs`
- CREATE: `crates/knowledge/src/symbol_graph/store.rs` (msgpack persistence)
- CREATE: `crates/knowledge/src/symbol_graph/migration_v3.sql`
- MODIFY: `crates/knowledge/src/lib.rs` (export `symbol_graph` module)
- MODIFY: `crates/orchestrator/src/indexing_service.rs` (call
  `build_symbol_graph` after DKG build)
- MODIFY: `crates/orchestrator/src/dispatcher.rs` (4 new tool entries)
- MODIFY: `crates/orchestrator/src/task_runner.rs` (PostToolUse impact hook)
- MODIFY: `crates/core/src/ssb/action_types.rs` (SymbolQuery variant)
- MODIFY: `crates/dashboard-v2/src-tauri/src/lib.rs` (V2 — was V1
  `crates/dashboard/src/commands.rs`; add `get_impact_analysis` Tauri
  command)
- MODIFY: `crates/dashboard-v2/src/lib/drawers/KnowledgeDrawer.svelte`
  (V2 path — was V1 `crates/dashboard/svelte/src/drawers/`; Impact
  Analysis panel; consumes new stub.get_impact_analysis once added)
- CREATE: `workflow/reasoning/B-29-decisions.md` (decision doc)

## Decision doc

Write `workflow/reasoning/B-29-decisions.md` documenting:
- Why msgpack-blob shards in SQLite vs flat tables (small read/write
  amortized, mirrors A-12 SSB pattern, avoids 27×N + 256×N row counts)
- Why std-only LRU vs `lru` crate (CLAUDE.md anti-dep policy)
- Why mutate confidence in-place vs returning new edges (memory; same
  pattern as DKG edge enrichment)
- Whether IMPACT_SUMMARY auto-injection should be feature-gated or
  always-on (recommend always-on, opt-out via env)
- Default depth values (B-2 in Auditor pre-check; user-overridable to 10)
- External grounding citation: FocalAD ELAI (arXiv:2506.11419) — record
  it as the academic anchor for the top-k local-focus design so future
  audits don't re-litigate the "why hop-band instead of global" choice

## Side benefits unlocked

- Auditor can answer "did this Edit affect more than the changed file"
  without reading the rest of the codebase — closes a long-standing GRC
  grounding gap
- Designer's Recursive Decomposition gets real call-flow input from
  existing-project mode (TechStackDecision::ExistingProject path)
- Researcher's Oracle Mode (/research) can ground "who calls X" with
  authoritative results, not LLM speculation
- Self-modification (B-23) gains a concrete blast-radius signal for
  proposed config changes that touch multiple call sites
- B-33 viewer gets symbol mode for free (same cache backs the data)
- B-34 entry-point detection runs against the same payloads (zero extra
  load)

## Next

B-30 (Multi-Language DKG Hardening) — independent, parallel-safe.
B-33 (Interactive Decomposition Viewer) — depends on B-29 cache existing.
B-34 (Entry-Point Detection Tool) — depends on B-29 file payloads.
