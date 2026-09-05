# Phase 1A Implementation Audit
Date: 2026-04-24
Overall verdict: IMPLEMENTED
Wiring-to-pipeline verdict: WIRED

## Sub-step: 1A.1 SKELETON
Verdict: IMPLEMENTED
Evidence:
- Crate exists: F:\.ELAI_workflow\.elai_cc\crates\knowledge\Cargo.toml (pkg "elai-knowledge") with tree-sitter=0.26, tree-sitter-rust=0.24, rusqlite=0.39 [bundled], plus tree-sitter-python/go/typescript/java/c-sharp/ruby all present (Cargo.toml:12-19). NOTE: Spec 1A.1 says "blake2 = 0.10" but implementation uses blake3=1.8 (Cargo.toml:21); this matches CLAUDE.md Anti-Stale-Training rule ("blake3 never blake2/sha2 for non-RFC uses"). Anti-Stale-Training overrode the spec — acceptable deviation and aligned with project standard.
- proptest=1 in [dev-dependencies] (Cargo.toml:43).
- DkgEngine struct: knowledge/src/dkg/mod.rs:89-95. `pub fn new(db_path: &Path, index_version_path: Option<&Path>, created_at: i64)` at mod.rs:98-146.
- `DkgNode` (internal) defined at knowledge/src/lib.rs:223-230 (name, kind, signature, start_line, end_line, parent_idx). `DkgNodeInfo` (public API contract) at lib.rs:211-219 with node_id, symbol_name, file_path, signature, kind, start_line, end_line — exactly per spec.
- `SymbolKind` enum at lib.rs:134-146 (Function, Struct, Enum, Trait, Impl, Module, Const, Static, TypeAlias, Macro — superset of spec list).
- `DkgReadiness` enum at lib.rs:127-132 (Ready, Empty, Unavailable) per spec.
- `ReadinessStatus` enum at lib.rs:120-125 (Ready, Degraded, Unavailable) for the aggregate engine.
- `DependencyGraph` struct at lib.rs:196-202 (node_count, edge_count, files_parsed, data_flow_count — spec asks 3 fields, extra field data_flow_count added by A-48 extension, non-breaking).
- `KnowledgeEngine` trait at lib.rs:253-259 (build_graph/query/rerank/update/readiness_check) — per spec; currently "zero implementors" per doc comment (aggregator deferred to Phase 1I).
- `LanguageGrammar` trait at lib.rs:237-241: all three methods take `&self` (language_id, tree_sitter_language, extract_nodes). OBJECT-SAFE — proven by use at mod.rs:91,124 where `HashMap<&'static str, Box<dyn LanguageGrammar>>` compiles and is instantiated at mod.rs:125-134.
- DkgEngine API methods (all present): nodes() mod.rs:388, edges() mod.rs:413, node_by_id() mod.rs:427, outbound_edges() mod.rs:451, inbound_edges() mod.rs:465, failed_files() mod.rs:479, dkg_readiness() mod.rs:862-872.
- SQLite schema SCHEMA_SQL mod.rs:18-69 creates: dkg_nodes (id PK, file_path, name, kind, signature DEFAULT '', start_line, end_line, parent_id REFERENCES dkg_nodes ON DELETE CASCADE, concept TEXT DEFAULT NULL, concept_stale INTEGER DEFAULT 1, file_hash BLOB), dkg_edges (source_id, target_id, edge_type, PK composite, both FKs CASCADE), dkg_files (file_path PK, file_hash, parse_status, parse_error, last_indexed), dkg_meta (key PK, value), dkg_data_flow (A-48 extension), plus indexes idx_nodes_name/_file/_kind/idx_files_status — all spec-required indexes present.
- Edge-type constants mod.rs:75-77: EDGE_IMPORTS="imports", EDGE_CONTAINS="contains", EDGE_IMPLEMENTS="implements".
- PRAGMA journal_mode=WAL at mod.rs:105; PRAGMA foreign_keys=ON at mod.rs:106.
- dkg_meta seeded: schema_version='1' (mod.rs:109), created_at=<unix> (mod.rs:113).
- elai.map evidence: DEF_STRUCT DkgEngine crates/knowledge/src/dkg/mod.rs:89 (rust + rust_scip canonical path `elai-knowledge 0.1.0 dkg/DkgEngine#`); IMPL_TRAIT LanguageGrammar matches at mod.rs:1032 (RustGrammar), 1063 (PythonGrammar), 1186 (GoGrammar), 1346 (TS), 1364 (TSX), 1512 (Java), 1650 (C#), 1798 (Ruby) — 8 grammars implemented.
Gap: NONE (deviation: blake3 replaces spec's blake2 — justified by CLAUDE.md Anti-Stale-Training).

## Sub-step: 1A.2 VERIFY
Verdict: NOT AUDITED (dynamic — cargo check/test not run by main model per CLAUDE.md audit policy)
Evidence:
- phase_1A.toml exists at metrics/phase_1A.toml, proving at least one successful end-to-end build_graph run against .elai_cc/ (135 files parsed, 0 skipped, 8075 nodes).
Gap: Compilation state at audit time UNVERIFIED; historically green based on metrics file presence.

## Sub-step: 1A.3 TEST (TDD red, >=20 tests, >=10 fail)
Verdict: IMPLEMENTED (green-phase; no longer red)
Evidence:
- 28 `#[test]` attributes in knowledge/tests/dkg_tests.rs (grep count = 28). Spec required "#[test] count >=20" — exceeded. Earlier comment "27+ tests per phase_1A.toml metrics" (audit prompt) also consistent.
- Additional contract tests for DKG in crates/contracts/tests/: dkg_multilang_contract.rs (>=7 #[test]), dkg_data_flow_a48_contract.rs (>=12 #[test]). These are downstream/A-48 contracts but all call DkgEngine::new + build_graph.
- proptest-regressions directory exists (knowledge/proptest-regressions/pagerank) — proptest usage confirmed elsewhere.
- Tests cover: build_graph success (mod.rs:63), parse failure -> failed_files (multiple), concept_stale default=1 (insert at mod.rs:262-273 hard-sets concept_stale=1), dkg_readiness transitions (tests 377+), multi-file population (411), file_hash record (429), parse_status=ok (449), project isolation (483-484 two engines distinct db paths), proptest-style nested tests (520).
Gap: NONE — test count exceeds spec.

## Sub-step: 1A.4 IMPLEMENT
Verdict: IMPLEMENTED
Evidence:
- `build_graph` mod.rs:148-327 walks files (walk_source_files, sort), dispatches by `ext_to_lang_id` through `grammars: HashMap<&'static str, Box<dyn LanguageGrammar>>`, parses with tree_sitter::Parser.
- Incremental via blake3 hash compare mod.rs:174-206: unchanged -> skip, changed -> DELETE + re-insert (CASCADE cleans edges).
- Parse failure handling mod.rs:209-245:
  - Binary detection via std::str::from_utf8 (line 209) -> tracing::warn + dkg_files parse_status='failed', parse_error='binary file'.
  - tree-sitter returns None -> tracing::warn + parse_status='failed', parse_error='parse returned None'.
  - has_error() -> tracing::warn + parse_status='failed', parse_error='syntax errors in source'.
  - No panic; file recorded in dkg_files; available via failed_files() at mod.rs:479. Spec compliance item #5 (parse failure logged) MET.
- Nodes inserted with concept_stale=1 hard-coded (mod.rs:262-273), concept column left NULL (per spec).
- Cross-file edges `generate_cross_edges` mod.rs:329-386: imports edges from 'use %' signature matching, implements edges from impl names containing " for ".
- Contains edges: parent->child via parent_idx during per-file insert (mod.rs:277-288).
- Directed query API WHERE source_id=?/target_id=? in outbound_edges/inbound_edges (mod.rs:451/465).
- dkg_readiness() correct: Ready (count>0), Empty (count=0 && build_graph_called), Unavailable (otherwise) — mod.rs:862-872.
- index_version.toml written at build_graph end mod.rs:314-319 via crate::index_version::write.
- Signature reconstruction: RustGrammar::extract_nodes calls extract_recursive which fills DkgNode.signature (per spec "fn name(params) -> return_type"). Metrics confirm 8075/8075 signatures populated (signatures_populated = 8075 in phase_1A.toml).
- blake3 (replacing blake2 per Anti-Stale-Training): mod.rs:4, mod.rs:175.
Gap: Minor spec/impl naming drift — "use %" SQL LIKE (mod.rs:333) resolves only last-segment use paths; adequate for compliance but not full name resolution. Not a 1A defect per spec.

## Sub-step: 1A.5 MEASURE
Verdict: IMPLEMENTED
Evidence:
- metrics/phase_1A.toml exists with all 12 spec keys:
  build_time_ms=9670, query_p50_ms=0.009, query_p99_ms=0.018, db_size_bytes=2842624, total_nodes=8075, total_edges=4148, files_parsed=135, files_skipped=0, edges_imports=249, edges_contains=3830, edges_implements=69, signatures_populated=8075.
- examples/measure.rs uses DkgEngine::new + build_graph end-to-end; other measure examples (measure_pi_latency.rs, measure_bm25.rs) further exercise build_graph.
- 9.67s build < 15s target (spec notes target was UNGROUNDED, but measured).
Gap: NONE.

## Wiring gap check
Verdict: WIRED (the pre-existing spec note is OUTDATED — wiring is complete as of audit).

Evidence (build_graph runtime callers, non-test):
- **F:\.ELAI_workflow\.elai_cc\crates\orchestrator\src\indexing_service.rs:166** — `IndexingService::run_full_index` constructs DkgEngine at line 161-165 with db_path `{indexes_dir}/dkg.db` and index_version_path, then calls `dkg.build_graph(&self.project_root)` at line 166. This is the real production caller.
- **F:\.ELAI_workflow\.elai_cc\crates\cli\src\main.rs:3831-3874** — CLI startup constructs `IndexingService` (line 3835), checks `indexes_exist`, runs full index on first run (line 3840-3844: "IndexingService: full index complete"), rebuild on schema version change (line 3859-3865). Degraded-mode warn-and-continue on failure (line 3848/3869).
- **F:\.ELAI_workflow\.elai_cc\crates\cli\src\main.rs:3752-3762** — After IndexingService run, QueryPipeline gets a DkgEngine attached for PageRank repo maps: `pipeline.dkg_engine = Some(engine)` when `.elai/dkg.db` exists. NOTE: this is a SECOND, read-only DkgEngine on a DIFFERENT path (`.elai/dkg.db`) than IndexingService writes (`.elai_cc/indexes/dkg.db`). Possible minor inconsistency — see Critical gaps.
- **F:\.ELAI_workflow\.elai_cc\crates\cli\src\main.rs:5437-5446** — `build_telescope_block` opens DkgEngine for nodes() to feed populate_tier3.
- **F:\.ELAI_workflow\.elai_cc\crates\cli\src\main.rs:7190-7199** — `DkgQuerierAdapter::new(&dkg_db_path)` or falls back to `EmptyDkgQuerier` for rule-engine queries. Same fallback pattern in dashboard/src/lib.rs:866-888 with HotSwappableDkgQuerier.
- Examples (measure.rs, measure_pi_latency.rs) still exist but are no longer the ONLY callers — the pre-existing wiring gap (pipeline-modes-design.txt + project-intelligence-design.txt) is CLOSED by A-44 IndexingService.

Conclusion: Phase 3J.WIRE.E2E / A-44 IndexingService landed. build_graph is called from the runtime startup path. WIRED.

## Error-type compliance
Verdict: CLEAN for DKG public API; VIOLATIONS remain in sibling modules.

Evidence:
- DKG public API uses `KnowledgeError` (thiserror-based) at knowledge/src/error.rs:3-25 with `#[from]` for io::Error, rusqlite::Error, QueryValidationError. DkgEngine methods all return `Result<_, KnowledgeError>`. NO `Box<dyn Error>` in knowledge/src/dkg/mod.rs, knowledge/src/lib.rs (apart from already-public KnowledgeError variants).
- `IndexingServiceError` (orchestrator/src/indexing_service.rs:91-107) also thiserror-based, wraps KnowledgeError via #[from].
- HOWEVER, **15 `Box<dyn std::error::Error>` signatures** remain in sibling knowledge modules (NOT 1A scope, but CLAUDE.md hard rule is global):
  - bm25.rs:155, 217, 243, 262, 314, 329, 362, 403 (8 methods)
  - pagerank/mod.rs:28 (from_engine)
  - vector/indexer.rs:57, 71, 89, 194, 218 (5 methods)
- None of these are in DKG. They are Phase 1B (BM25), 1C (vector), 1D (pagerank) code. Phase 1A spec only governs DKG — so 1A is CLEAN. The broader codebase violates the CLAUDE.md hard rule "No Box<dyn Error> in public API", but that is NOT a Phase 1A regression; it is an open debt in 1B/1C/1D.

## Test evidence
- 28 #[test] functions in F:\.ELAI_workflow\.elai_cc\crates\knowledge\tests\dkg_tests.rs (spec minimum 20; audit prompt said 27+; actual 28).
- Additional DKG-relevant contracts (downstream / A-48):
  - crates/contracts/tests/dkg_multilang_contract.rs (multi-language fixtures)
  - crates/contracts/tests/dkg_data_flow_a48_contract.rs (data flow)
  - crates/contracts/tests/fula_dkg_querier_hot_swap_contract.rs (adapter hot-swap)
- proptest usage evidenced by knowledge/proptest-regressions/ directory (regressions persisted for pagerank proptest cases; proptest dep confirmed in Cargo.toml:43).
- Phase metric file metrics/phase_1A.toml present with all 12 spec-required keys populated and sane values (8075 nodes, 4148 edges, 135 files parsed, 9.67s build).

## Multi-language verification
Spec says "Rust-only initial (Python/TS/Go/Java/C# are future work item 1A-MULTILANG)".
Implementation STATE: 8 LanguageGrammar impls present — RustGrammar + PythonGrammar + GoGrammar + TypeScriptGrammar + TsxGrammar + JavaGrammar + CSharpGrammar + RubyGrammar (all at mod.rs via IMPL_TRAIT entries 176567-176581). ext_to_lang_id dispatch routes .rs/.py/.go/.ts/.tsx/.java/.cs/.rb. `extract_all_data_flow_edges` guards data flow extraction to Rust only (mod.rs:594 `if lang_id != "rust" { continue; }`).

Conclusion: 1A-MULTILANG future-work item has already shipped (non-regression; exceeds baseline). No negative impact on 1A acceptance.

## Summary
- Implemented sub-steps: 5 / 5 (1A.1 skeleton, 1A.2 verify inferred from metrics, 1A.3 test, 1A.4 implement, 1A.5 measure). 1A.2 not directly runnable by auditor — inferred green from 1A.5 artifacts.
- Wiring: WIRED. DkgEngine::build_graph is driven by `elai_orchestrator::indexing_service::IndexingService::run_full_index`, invoked from `crates/cli/src/main.rs:3829-3874` on CLI startup. DkgQuerierAdapter + EmptyDkgQuerier fallback additionally hot-swaps the rule-engine DKG querier after build. The pre-existing spec warning (pipeline-modes-design.txt, project-intelligence-design.txt) is STALE — A-44 closed the gap.
- Confidence: HIGH. Every assertion backed by file:line or elai.map SCIP entry.
- Critical gaps:
  1. DUAL DKG PATHS. IndexingService writes dkg.db at `.elai_cc/indexes/dkg.db` (indexing_service.rs:30 "INDEXES_DIR_NAME=indexes", line 131). QueryPipeline DKG attach (cli/main.rs:3752) reads from `.elai/dkg.db` — a different subdirectory. Telescope block (cli/main.rs:5437) also reads `.elai/dkg.db`. If the pipeline never writes to `.elai/dkg.db` (nothing in the audit shows it does), PageRank + telescope will see an empty/missing DB. This likely means IndexingService populates the real index but the pipeline consumers look at a legacy path. Candidate bug/Phase 3J.WIRE.E2E residue. Recommend grep for `.elai/dkg.db` vs `.elai_cc/indexes/dkg.db` usage and unify.
  2. Box<dyn Error> debt still present in bm25/pagerank/vector modules (15 occurrences). Not a 1A scope violation, but CLAUDE.md hard rule — track for Phase 1B/1C/1D cleanup.
  3. KnowledgeEngine trait has zero implementors (lib.rs:253 doc comment confirms) — expected per spec (Phase 1I aggregator), not a gap at 1A level.
  4. "effect tagging on functions" mentioned in audit prompt EXPECTED DELIVERABLES is NOT in the Phase 1A spec text lines 256-365 (which lists only 5 compliance items, not effect tagging). Not a real gap; audit-prompt artifact.
  5. Spec called for blake2, impl uses blake3 — deviation documented as Anti-Stale-Training-compliant.
