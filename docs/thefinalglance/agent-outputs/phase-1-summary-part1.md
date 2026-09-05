# ELAI Phase 1 — Knowledge Engine — Lossless Step-File Summary

Source files (all under F:\.ELAI_workflow\docs\specs\guide\steps\):
- phase-1A.txt  (DKG / tree-sitter + SQLite)
- phase-1B.txt  (BM25 / tantivy)
- phase-1C.txt  (Vector search / dual-model ONNX)
- phase-1D.txt  (PageRank repo map)
- phase-1E.txt  (CatRAG + A-RAG pipeline)
- phase-1F.txt  (Speculative decoding evaluation)
- phase-1G.txt  (Cross-encoder reranker)
- phase-1H.txt  (SPLADE keyword expansion, conditional)
- phase-1I.txt  (Resilience: circuit breaker, cascade, readiness, tracing)

Output split:
- part1 = 1A, 1B, 1C
- part2 = 1D, 1E, 1F
- part3 = 1G, 1H, 1I

=====================================================================
PHASE 1A — DKG (tree-sitter -> SQLite graph)
=====================================================================

Canonical path: .elai_cc/crates/knowledge/
Status markers: AUDIT FINDINGS ADDRESSED (v3). NOTES dated 2026-04-06. Phase 1D/1G/1I paths updated 2026-04-06.
Goal: Parse source into a dependency knowledge graph stored in SQLite. Layer 1 of knowledge pipeline.

Downstream consumers declared:
- Phase 1D reads DKG edges for adjacency matrix.
- Phase 1E traverses DKG for evidence chains.
- Phase 1B shares readiness gate.

Risks
- R7 (LOW) tree-sitter grammar gaps. Start Rust, Python, TS, Go.
- R-1A-1 SQLite schema not specified in specs; define here.
- R-1A-2 <15s build time claim UNGROUNDED.
- R-1A-3 Phase 1D PRE_GATE needs edge/adjacency API.
- R-1A-4 parse fallback items 2-4 deferred to 1B/1C/1D; 1A implements items 1+5 only.
- R-1A-5 LanguageGrammar must be object-safe (all methods &self).
- R-1A-6 DkgNodeInfo must include signature + SymbolKind per 1D contract.

COMPLIANCE SCOPE
knowledge.txt Sec 8 tree-sitter fallback:
1. Exclude file from DKG graph                           — IMPLEMENTED here
2. Still BM25-index the file                             — DEFERRED to 1B
3. Still vector-embed the file                           — DEFERRED to 1C
4. PageRank treats file as leaf                          — DEFERRED to 1D
5. Log parse failure, retry on change                    — IMPLEMENTED here
Invariant "every file in at least BM25 index" only achievable after 1B.

STEPS

1A.1 SKELETON
PRE_GATE: knowledge Cargo.toml exists; Phase 0 cargo check finishes; no prior tree-sitter code.
TASK:
- Resolve KNOW_PATH dynamically from knowledge/Cargo.toml.
- Cargo.toml dependencies (verified crates.io 2026-04-06):
  tree-sitter 0.26 (0.26.8 latest), tree-sitter-rust 0.24 (0.24.2 latest),
  rusqlite 0.39 features=["bundled"] (0.39.0 latest), blake2 0.10,
  [dev-dependencies] proptest 1.
  Future: tree-sitter-python 0.25, tree-sitter-go 0.25, tree-sitter-typescript 0.23.
- Define `KnowledgeEngine` trait (lib.rs) matching architecture-overview.txt L94-104:
    fn build_graph(&mut self, project_root: &Path) -> Result<DependencyGraph>
    fn query(&self, query: &str, strategy: QueryStrategy) -> Result<Vec<CodeChunk>>
    fn rerank(&self, query: &str, candidates: Vec<CodeChunk>) -> Result<Vec<CodeChunk>>
    fn update(&mut self, file_changed: &Path) -> Result<()>
    fn readiness_check(&self) -> ReadinessStatus
- `DependencyGraph { node_count, edge_count, files_parsed }`.
- `ReadinessStatus { READY, DEGRADED, UNAVAILABLE }` (trait-level, aggregate; full-engine aggregate is Phase 1I's job).
- `LanguageGrammar` trait (object-safe, all &self):
    fn language_id(&self) -> &'static str
    fn tree_sitter_language(&self) -> tree_sitter::Language
    fn extract_nodes(&self, tree: &Tree, source: &[u8]) -> Vec<DkgNode>
- `SymbolKind` enum: Function, Struct, Impl, Trait, Enum, Module, Use, TypeAlias, Other.
- `DkgNodeInfo { node_id: i64, symbol_name: String, file_path: PathBuf, signature: String, kind: SymbolKind, start_line: u32, end_line: u32 }`.
  signature reconstructed from AST:
    function_item -> "fn name(params) -> return_type"
    struct_item -> "struct Name { ... }" (abbreviated)
    trait_item -> "trait Name"
- `DkgEngine::new(db_path: &Path)` — project-scoped.
- SQLite schema:
    dkg_nodes(id PK, file_path TEXT, name TEXT, kind TEXT, signature TEXT DEFAULT '',
              start_line INT, end_line INT, parent_id INT REFERENCES dkg_nodes(id),
              concept TEXT NULL, concept_stale INT DEFAULT 1, file_hash BLOB)
    dkg_edges(source_id INT REF dkg_nodes ON DELETE CASCADE,
              target_id INT REF dkg_nodes ON DELETE CASCADE,
              edge_type TEXT NOT NULL,  -- "imports" | "contains" | "implements"
              PRIMARY KEY (source_id, target_id, edge_type))
    dkg_files(file_path TEXT PK, file_hash BLOB, parse_status TEXT DEFAULT 'pending',
              parse_error TEXT NULL, last_indexed INT)
    dkg_meta(key TEXT PK, value TEXT)
    Indexes: idx_nodes_name, idx_nodes_file, idx_nodes_kind, idx_files_status.
- Edge semantics (directed):
    imports — source module -> target symbol
    contains — parent -> child
    implements — impl block -> trait
- index_version.toml at project-scoped path (5 fields):
    [index] version=1, dkg_schema_version=1, vector_dimensions={code=0,text=0},
    bm25_format="", created_at=<unix>.
- Public DkgEngine API:
    nodes(), edges(), node_by_id(id), outbound_edges(node_id), inbound_edges(node_id),
    failed_files(), dkg_readiness() -> DkgReadiness.
- `DkgReadiness { Ready, Empty, Unavailable }` — NOT same as trait-level ReadinessStatus.

DETERMINISTIC_CHECK (all greps on KNOW_PATH):
- cargo check -p knowledge exits 0
- 1 hit each for: trait KnowledgeEngine, enum ReadinessStatus, enum DkgReadiness,
  fn language_id(&self), enum SymbolKind, pub signature: String, struct DkgEngine,
  dkg_files CREATE TABLE
- 2 hits for outbound_edges + inbound_edges
- >=1 hit for concept_stale, ON DELETE CASCADE, proptest in Cargo.toml
- index_version.toml 5 fields grep count >=5

ADVERSARIAL:
- Typosquat tree-sitter crate name causes unresolved imports.
- Trait signature drift causes compile errors downstream.
- LanguageGrammar methods without &self -> Box<dyn> fails.
- Missing signature/SymbolKind breaks Phase 1D DkgSnapshot::from_engine.

ACCEPTANCE: cargo check 0; all greps satisfied; Box<dyn LanguageGrammar> compiles.

1A.2 VERIFY
PRE_GATE: 1A.1 checks pass; workspace cargo check 0.
TASK: cargo check workspace; cargo test --workspace (0 failed).
ADVERSARIAL: rusqlite bundled feature needs cc compiler in CI.
ACCEPTANCE: workspace compiles + pre-existing tests still pass.

1A.3 TEST (TDD RED)
PRE_GATE: proptest in dev-deps; 1A.2 green.
Tests to write (>=20 #[test]):
Functional: parse rust file extracts function/struct/impl/use/mod/trait/enum/type nodes;
query by name returns correct file+line; signature populated ("fn foo(x: i32) -> bool");
incremental update modifies line numbers, file_hash changes, concept_stale reset;
concept NULL by default; file_hash byte-exact round-trip.
Edge type tests: impl->method contains; use->target imports; impl Trait for Struct -> implements;
outbound != inbound direction.
Downstream API: edges() returns typed edges; outbound_edges/inbound_edges correct;
nodes() iteration; failed_files() empty when all parse.
Parse failure (DKG-layer only): empty file -> 0 nodes no panic; syntax error ->
partial extraction + log + parse_status='failed'; binary -> 0 nodes + failed_files() hit.
Readiness: Unavailable before build_graph; Ready after (>0 nodes); Empty after empty build.
Isolation: two DkgEngine with different db_path don't share state.
Property (proptest): random Rust snippets -> nodes>=0, lines>0, names non-empty, hashes non-empty,
signatures non-empty for function/struct.
DETERMINISTIC_CHECK: cargo test --no-run 0; FAILED count >=10 (stubs may pass 3 trivial tests).
ACCEPTANCE: >=20 tests, compile, >=10 fail.

1A.4 IMPLEMENT
PRE_GATE: 1A.3 red with >=10 failures; cargo check 0 with grammars.
TASK:
- RustGrammar impl LanguageGrammar (all &self):
    Walks AST; extracts function_item, struct_item, impl_item, use_declaration, mod_item,
    trait_item, enum_item, type_item.
    Reconstructs signature per node kind.
- Parser dispatch: HashMap<&str, Box<dyn LanguageGrammar>> (only .rs -> RustGrammar initially).
- SQLite layer: CREATE TABLE IF NOT EXISTS for all; PRAGMA journal_mode=WAL;
  insertion with concept_stale=1 + file_hash; typed directed edges:
    contains (parent->child), imports (use_decl->resolved, log unresolved),
    implements (impl->trait if trait in graph).
- dkg_files tracking (insert/update parse status).
- Directed queries: outbound/inbound/by-id/nodes/edges/failed_files.
- Incremental update: blake2s hash compare; on change DELETE WHERE file_path=?, re-parse,
  reinsert (cascade drops edges); update dkg_files; concept_stale=1.
- build_graph: walk dir, parse each supported, populate SQLite, return DependencyGraph.
- Parse failure: log::warn; insert dkg_files row with parse_status='failed'; no nodes;
  no panic.
- dkg_readiness: Unavailable pre-build, Ready if >0 nodes, Empty if 0.
- index_version.toml write after build_graph (5 fields).
- DkgEngine::new(db_path) explicit Path.
- File I/O via core/ abstractions where available.

DETERMINISTIC_CHECK:
- all tests pass; warnings count recorded (target 0); PRAGMA journal_mode >=1;
  log::warn/tracing::warn >=1; DkgReadiness:: >=3; "imports"/"contains"/"implements" >=3;
  blake2 >=1; concept_stale >=2; index_version fields >=4; signature >=2.

ADVERSARIAL:
- Grammar returns unexpected types -> silent drops (parse lib.rs and compare fn count).
- WAL fails on read-only FS -> graceful error.
- 10k+ AST node file -> indexing >15s.
- Zero "imports" edges (external crate unresolved) -> log warning, document limit.
- Empty signature for function_item/struct_item -> Phase 1D blanks.

ACCEPTANCE: 0 failures; builds from .elai_cc; queries return correct file+line+signature;
edge count >0; directed queries asymmetric; readiness Ready; parse-fail non-rs no panic + in failed_files();
isolation test passes.

1A.5 MEASURE
TASK: measure on .elai_cc codebase:
- index build time (vs <15s claim — first measurement)
- query p50/p99 (10 random lookups)
- SQLite db size
- counts: total_nodes, total_edges by type, files_parsed, files_skipped, signatures_populated
Output: .elai_cc/metrics/phase_1A.toml with 12 required keys:
build_time_ms, query_p50_ms, query_p99_ms, db_size_bytes, total_nodes, total_edges,
files_parsed, files_skipped, edges_imports, edges_contains, edges_implements, signatures_populated.
ADVERSARIAL: validate toml parses.
ACCEPTANCE: file exists, all 12 metrics populated.

EXIT CRITERIA: DKG builds, queries correct; all tests pass; directed edge API exposed;
failed_files() bridges to 1B; dkg_readiness() scope correct; index_version.toml 5 fields;
DkgNodeInfo with signature+SymbolKind; concept_stale; project-scoped db; metrics baseline; no regressions.

GROUNDING table (selected):
- "tree-sitter 0.26.8" VERIFIED crates.io 2026-04-06.
- "<15s build" UNGROUNDED — 1A.5 establishes baseline.
- "Edge types imports/contains/implements" DERIVED from knowledge.txt L67, L79 + 1D contract.
- "LanguageGrammar object safety" VERIFIED via Rust trait rules; audit caught non-&self.
- "7 layers vs 5 LAYERS" RESOLVED — header corrected to "5 major + 2 sub-layers".

AUDIT FINDINGS ADDRESSED v3: BLOCK 1-6, WARN 1-7 (path inconsistency, DkgNodeInfo contract,
readiness scope split, index_version.toml 5 fields, compliance scope, object safety,
concept_stale column, project isolation, proptest dep, neighbors direction,
DependencyGraph return, file_hash round-trip, dkg_files table).

=====================================================================
PHASE 1B — BM25 (tantivy full-text search)
=====================================================================

Canonical path: .elai_cc/crates/knowledge/
Prerequisites: Phase 1A complete.
Goal: full-text search, sub-10ms queries under 100K docs, per-project isolation,
zero orphan text files, binary files (null bytes in first 8KB) only exception.

Risks:
- R1 tantivy INDEX_FORMAT_VERSION is u32 constant, not label; spec "tantivy_v21" is project label.
- R2 index corruption on crash — atomic commit.
- R3 Windows paths.
- R4 memory usage on large repos.
- R5 binary file panic — skip.
- R6 chunking alignment with vectors/reranker.
- R7 path field MUST be STRING (untokenized), not TEXT, else delete_term fails silently.

STEPS

1B.1 ADD TANTIVY DEPENDENCY
PRE_GATE: knowledge/Cargo.toml exists; Phase 1A lib tests pass; no prior tantivy entry.
TASK: verify tantivy on crates.io; add pinned version; record INDEX_FORMAT_VERSION (u32) in decision doc.
DETERMINISTIC_CHECK: cargo check 0; tantivy grep hit.
ADVERSARIAL: version too new breaks format; dep version conflicts break cargo check.
ACCEPTANCE: crate compiles with tantivy.

1B.2 BM25 MODULE SKELETON + CORRECT API SHAPE
TASK: create bm25.rs in knowledge/src/.
- `BM25Chunk { path: PathBuf, content: String, line_start: u32, line_end: u32, kind: ChunkKind }`.
- `enum ChunkKind { Function, LineBlock, File }`.
- `Bm25Index::new(project_root: &Path, project_id: &str) -> Result<Self>`. project_id first-class.
- Index path: `{project_root}/.elai_cc/projects/{project_id}/bm25/`.
- Tantivy schema:
    path: STRING | STORED (untokenized, required for delete_term exact match)
    content: TEXT | STORED (tokenized, BM25 target)
    line_start: u64 STORED
    line_end: u64 STORED
    chunk_kind: STRING | STORED
- Methods:
    `index_file(path, chunks: Option<Vec<BM25Chunk>>) -> Result<usize>` —
      if chunks provided, one doc per chunk; else 50-line sliding window fallback;
      binary (null in first 8KB) skip + warn.
    `index_directory(root, dkg: Option<&DkgIndex>) -> Result<IndexStats>` —
      walk dir, query DKG for node ranges, create BM25Chunks, else line fallback.
      Returns IndexStats { files_indexed, chunks_indexed, files_skipped_binary, elapsed_ms }.
    `query(text, limit) -> Result<Vec<BM25Hit>>`
      BM25Hit { path: String, line_start: u32, line_end: u32, score: f32,
                chunk_kind: ChunkKind, snippet: String (first 200 chars, no SnippetGenerator) }.
    `delete_file(path) -> Result<usize>` via delete_term on STRING path.
    `update_file(path, chunks)` atomic (delete + insert + commit); <5s target.
- Single IndexWriter per Bm25Index; index_directory holds writer entire batch.
- Register mod in lib.rs.
- Contract test stub: .elai_cc/tests/contracts/bm25_contract.rs, one per public method.

DETERMINISTIC_CHECK:
- bm25.rs exists; mod bm25 in lib.rs; cargo check 0; existing tests green;
  score + chunk_kind + IndexStats + BM25Chunk + STRING + contract file all grep-hit.

ADVERSARIAL:
- path field accidentally TEXT -> delete_file silently fails (regression test j).
- hardcoded path separator -> Windows breaks.

ACCEPTANCE: bm25.rs registered; API complete; ChunkKind enum; BM25Chunk content field;
path field STRING; contract stubs.

1B.3 TEST (TDD RED)
Tests in bm25.rs or tests/bm25_tests.rs, all using tempdir synthetic files:
(a) Index rust file with hello_world, query -> correct line range.
(b) Index tempdir with 3 files, query "fn complete" -> correct file.
(c) delete_file then query returns empty.
(d) Nonexistent term -> empty Vec (not error).
(e) Zero-orphan: index .xyz file with chunks=None, query keyword, get result (line fallback).
(f) Binary file detection: tempdir with text+binary, files_indexed=1, files_skipped_binary=1.
(g) update_file replaces content atomically.
(h) Per-project isolation: two project_ids don't cross-contaminate.
(i) Chunking strategy: chunks=Function get ChunkKind::Function; None with >120 lines produces LineBlock.
(j) path STRING regression guard: multi-segment path delete_file is precise.
DETERMINISTIC_CHECK: >=10 tests; all FAILED initially.
ADVERSARIAL: test paths in tempdir; cleanup; test j must fail with TEXT.
ACCEPTANCE: 10+ tests, all fail (red).

1B.4 IMPLEMENT
Core index with schema above.
index_file two modes:
- with BM25Chunks -> one doc per chunk, ChunkKind::Function.
- without chunks -> 50-line sliding windows, 10-line overlap -> LineBlock;
  small files (<50 lines) single doc ChunkKind::File.
Rationale note: 10-line overlap is UNGROUNDED for BM25 specifically (scores terms independently);
carried for cross-layer chunk alignment; ~20% more docs, no proven BM25 benefit.
Binary detection: null byte in first 8KB -> skip + warn + IndexStats count.
index_directory: recursive walk; skip hidden + .elai_cc/; DKG-derived BM25Chunks when available;
atomic batch commit; blocks concurrent update_file.
query: tantivy QueryParser on content; top N; snippet=first 200 chars.
delete_file: delete_term on STRING path, single commit.
update_file: delete+insert single commit, <5s.
DETERMINISTIC_CHECK: all BM25 tests pass; no warnings on bm25.rs; DKG tests still pass.
ADVERSARIAL: file-race skip gracefully; second writer returns Err not deadlock;
overlap ensures function in at least one chunk with context; whitespace-only still indexed.
ACCEPTANCE: 10+ tests green; binary skipped; .xyz indexed (zero orphan); delete exact; atomic update;
no regression; isolation confirmed.

1B.5 MEASURE AND VALIDATE INVARIANTS
Benchmark: index .elai_cc/rust/ (69 .rs files, ~61K LOC).
- Build time, 10 diverse queries, p50/p99, single-file update_file latency.
Invariants: text files on disk == indexed (zero orphan); index path correct; update_file <5s.
Output: .elai_cc/metrics/phase_1B.toml with keys:
  index_build_time_ms, file_count, chunk_count, binary_skipped,
  query_p50_ms, query_p99_ms, update_file_p99_ms,
  orphan_files (==0), index_size_bytes, tantivy_version, index_format_version (u32).
DETERMINISTIC_CHECK:
- CI=true -> query_p99_ms < 50; else < 10.
- update_file_p99_ms < 5000.
- all fields present.
ADVERSARIAL: warm-up discard; CI vs dev thresholds; O(n^2) scaling check (ms per chunk).
ACCEPTANCE: p99 <10ms dev / <50 CI; update <5s; zero orphan text files; all fields;
build <5s for 69 files.

EXIT CRITERIA: sub-10ms p99; 10+ tests; no DKG regression; per-project isolation;
zero orphan text files; binary handled; atomic update <5s; DKG-derived chunking with line fallback;
spec-correct path convention; STRING path field; contract tests; metrics with tantivy_version +
index_format_version.

DECISIONS D1-D9 logged to workflow/reasoning/phase-1B-decisions.md:
D1 50-line sliding window + 10-line overlap (overlap UNGROUNDED for BM25).
D2 null-byte binary detection (git-style, not MIME).
D3 BM25Hit.chunk_kind for downstream filtering.
D4 path convention .elai_cc/projects/{id}/bm25/.
D5 atomic update_file (single-writer blocks index_directory).
D6 path=STRING (exact delete_term).
D7 BM25Chunk owned by BM25 (DKG doesn't store content).
D8 project_id explicit, not derived.
D9 snippet=200-char truncation (not SnippetGenerator).

GROUNDING table:
- tantivy full-text VERIFIED knowledge.txt L36 L103.
- <10ms VERIFIED L36 L255.
- per-project VERIFIED L37.
- "tantivy_v21" CORRECTED — project label not tantivy u32.
- "fn complete" CORRECTED — tests now tempdir-only.
- Section 4 VERIFIED.
- path STRING VERIFIED from tantivy examples.
- project_id first-class VERIFIED L274.
- 10-line overlap UNGROUNDED for BM25 specifically.
- Circuit breaker, SearchMode, query validation, readiness, tracing, versioning SCOPED OUT to 1I.

=====================================================================
PHASE 1C — VECTOR SEARCH (dual-model ONNX)
=====================================================================

Prerequisites: Phase 1B complete.
Goal: dual embedding (GraphCodeBERT 768d code, Granite 384d text) via ONNX Runtime (ort crate);
vector cache in SQLite keyed by content hash; circuit breakers per model with graceful degradation;
startup readiness gate with canary.

Status: SPLADE deferred to Phase 1G/1H (EVALUATE). Circuit breakers only for GraphCodeBERT + Granite.

Risks:
- R22 (MED) GPU contention — embedding GPU1 only if executor/auditor on GPU0; fallback CPU-only;
  ModelPool.can_fit_onnx(model_size) before alloc.
- R26 (MED) single ONNX failure — per-model CB; HYBRID -> SPARSE_ONLY -> BM25_ONLY.
- R27 (LOW) ONNX startup blocking FSM — canary "fn main()" <5s.
- R29 (LOW) cross-encoder concurrency — Semaphore(1) + depth 4.

GROUNDING SECTION
[VERIFIED-SPEC]: code 768d GraphCodeBERT, text 384d Granite; embed()->Vec<f32>;
vector_cache schema (content_blake2s BLOB PK, code_vec BLOB, text_vec BLOB, cached_at INT);
ort ADD; CB threshold 3/60s, half-open 30s; canary <5s; Semaphore(1) depth 4;
cache hit >=90%; DELETE on SSB file_write; degradation cascade; dual-GPU vs CPU-only rule.
[VERIFIED-CRATE]: ort (pykeio) ONNX Runtime 1.24; usearch v2.24.0 HNSW SIMD f32/f64/i8/f16/binary;
fastembed v5.12.0 (wraps ort — don't add both); blake2 crate with Blake2s256 (not blake2s crate).
[GROUNDED-PAPER] TurboQuant arXiv:2504.19874 ICLR 2026 ingested 2026-04-06:
  Distortion (unit-norm): b=2 MSE=0.117, b=3 MSE=0.030, b=4 MSE=0.009.
  Inner product: b=2 0.56/d, b=3 0.18/d, b=4 0.047/d.
  768d@b=3: ~3 bits/dim, ~10.7x vs f32, InProd err ~0.00023.
  384d@b=3: ~3 bits/dim, ~10.7x vs f32, InProd err ~0.00047.
  Indexing: ~0.001s vs PQ ~494s at 100k.
  Requirements: unit-norm input, QR rotation (NOT Hadamard), Lloyd-Max codebook per (b,d);
  d >= 64. Deferred to Phase 1G+ (Phase 1C stores raw f32).
[UNGROUNDED]: GraphCodeBERT/Granite ONNX tensor format; throughput; dual-vector vs BM25 accuracy.

STEPS

STEP 1 — DEPENDENCY VERIFICATION AND CARGO SETUP
PRE_GATE: 1B ok; rusqlite present; no ort entry.
ACTIONS: cargo add ort (pinned, GPU feature if dual-GPU); cargo add usearch; cargo add blake2;
do NOT add fastembed (conflict); record HNSW=usearch in workflow/reasoning/phase-1C-decisions.md.
DETERMINISTIC_CHECK: cargo check 0; ort version-pinned; decisions file exists.
ADVERSARIAL: wrong crate names; usearch API diff; blake2 type name.
ACCEPTANCE: cargo check 0; HNSW decision logged; no floating ort="*".

STEP 2 — SKELETON (module stubs, ort session stub, vector cache table)
Files:
- knowledge/src/vector/mod.rs — VectorModule trait; `DualVector { code_vec: Vec<f32> 768d, text_vec: Vec<f32> 384d }`.
- knowledge/src/vector/cache.rs — SQLite CRUD (insert, lookup by blake2s, invalidate by file_path).
  Schema: content_blake2s BLOB PK, code_vec BLOB, text_vec BLOB, cached_at INT.
- knowledge/src/vector/ort_session.rs — `OrtSession` wrapping `Arc<Mutex<ort::Session>>`
  (ort::Session::run takes &mut self). Models loaded from canonical path
  .elai_cc/models/onnx/ (pre-staged, not auto-downloaded); missing model -> ModelNotFound Err ->
  DEGRADED BM25_ONLY, not crash. Path configurable via [knowledge.onnx_model_path] in elai_cc.toml.
  `embed(text) -> Result<Vec<f32>, OrtError>`. Two instances: code_model GraphCodeBERT 768d,
  text_model Granite 384d. tokenizer max_seq_len=512 for GraphCodeBERT; truncate before tensor;
  log WARNING on truncation; store max_seq_len field.
- knowledge/src/vector/circuit_breaker.rs — `CircuitBreaker` states CLOSED/OPEN/HALF_OPEN;
  threshold 3/60s; half-open after 30s.
- Extend vector_cache SQLite migration in 1B schema runner (or M3).
- SSB file_write handler calls DELETE FROM vector_cache WHERE file_path=?.
- knowledge/src/vector/hnsw_index.rs — VectorIndex trait + UsearchIndex wrapping `usearch::Index`.
  MetricKind::Cos; add(key, vec), search(query, k), filtered_search(query, k, predicate) for
  project isolation; save(path)/load(path) persistence. No view() (read-only).
  Two instances: code_index (768d), text_index (384d). Index files alongside SQLite DB.
- Do NOT write tensor construction before Step 3.5.
- INPUT CHUNKING DEPENDENCY flagged (AST vs line); reference code-sage (github.com/faxioman/code-sage)
  for tree-sitter AST chunking. If chunking not defined earlier, add sub-step here.

DETERMINISTIC_CHECK: cargo check 0; greps for DualVector, CircuitBreaker, vector_cache,
Arc<Mutex<...Session>>, max_seq_len|512, VectorIndex|UsearchIndex, filtered_search, save|load.
No TODO/placeholder (unimplemented!() behind error path acceptable; TODO not).

ADVERSARIAL:
- bad model path panic — unit test asserts Err.
- migration ordering (M3 after M1+M2).
- DualVector dim mismatch — assert code 768, text 384.
- silent tokenizer truncation — assert warning log.
- usearch load corrupt file — rebuild from cache (source of truth).

ACCEPTANCE: cargo check 0; migration runs; DualVector::new validates dims;
OrtSession stores max_seq_len + Arc<Session>; VectorIndex with save/load/filtered_search stubs compile;
no TODO; OrtSession bad path returns Err.

STEP 3 — TEST (TDD, no network, no real models)
Required unit tests (>=22):
- test_dual_vector_dimensions (768/384).
- test_vector_cache_insert_lookup.
- test_vector_cache_invalidation.
- test_circuit_breaker_opens_after_3_failures.
- test_circuit_breaker_half_open_probe (advance 30s).
- test_graceful_degradation_code_model_fail (text_vec only).
- test_graceful_degradation_both_models_fail (SearchMode::SPARSE_ONLY).
- test_query_validation_empty/oversized/control_chars (validation errors).
- test_startup_canary_success (<5s -> Ready).
- test_startup_canary_timeout (>5s -> Degraded(BM25_ONLY)).
- test_onnx_canonical_path (.elai_cc/models/onnx/).
- test_missing_model_degraded (ModelNotFound -> BM25_ONLY, not crash).
- test_semaphore_queue_overflow (5th of >4 concurrent rerank -> unranked).
- test_cache_hit_skips_embed.
- test_truncation_warning_on_long_input (>512 tokens, tracing-test crate).
- test_hnsw_index_persistence_roundtrip (save/load).
- test_hnsw_filtered_search_project_isolation.
- test_concurrent_embed_mutex_session (4 tokio tasks, no deadlock).
- test_rrf_fusion_formula (3-ranker k=60).
Contract tests in tests/contracts/knowledge_vector_contract.rs:
- test_query_returns_dual_vector_results (code_score + text_score).
- test_query_project_isolation (physical separate knowledge.db per project per L13).
DETERMINISTIC_CHECK: #[test] count >=20; named-test greps; cargo test --no-run 0.
ADVERSARIAL: OrtSession must be injectable; blake2s bytes round-trip; std::time FAIL in CB (entropy).
ACCEPTANCE: cargo test --no-run 0; >=22 tests; contract tests for dual-vector + isolation;
no std::time in CB.

STEP 3.5 — TENSOR FORMAT VERIFICATION (gate for Step 4)
TASK: verify GraphCodeBERT input_ids/attention_mask shapes from HF model card + ONNX metadata.
Expected: input_ids (batch, seq_len), attention_mask (batch, seq_len), max_seq_len=512.
Verify Granite tensor format similarly.
Record tensor_format_verified: true + shapes in workflow/reasoning/phase-1C-decisions.md.
Add tokenizer crate (tokenizers or huggingface-hub) if missing.
DETERMINISTIC_CHECK: grep tensor_format_verified: true; grep input_ids|attention_mask.
ACCEPTANCE: shapes recorded or "unknown" = FAIL.

STEP 4 — IMPLEMENT (ort ONNX loading, dual-vector embed, HNSW, cache)
PRE_GATE: Step 3 and 3.5 passed; TurboQuant paper INGESTED (workflow/papers/), impl deferred.
ACTIONS:
- OrtSession::embed: tokenize (truncate 512, warn), build tensor, run session, extract output,
  Vec<f32> with correct dim.
- VectorCache: insert_dual, lookup_by_hash, invalidate_by_file; single-writer SSB channel.
- CircuitBreaker: record_failure/success, is_open, probe via ClockProvider (not std::time).
- HNSW via usearch v2.24.0: Index(MetricKind::Cos, dim=768/384); VectorIndex trait wrapping;
  save after bulk add; load at startup; rebuild from vector_cache if load fails;
  filtered_search for project isolation.
- KnowledgeEngine::readiness_check: load both models, canary embed("fn main()"), <5s both ->
  Ready; else Degraded(mode). Canary doubles as ONNX warmup (first inference slow per
  microsoft/onnxruntime#19177); run BEFORE reporting Ready.
- RRF fusion: BM25 + code_vec + text_vec (3 rankers); `rrf_score = sum(1/(k+rank))` with k=60
  (Cormack/Clarke/Buettcher SIGIR'09). Return CodeChunk with code_score, text_score, bm25_score, rrf_score.
- SSB file_write handler: DELETE FROM vector_cache WHERE file_path=?; registered at startup.
- Cross-encoder guarded by tokio::Semaphore(1), queue depth 4, overflow returns unranked.
- Emit SSB action_type='knowledge_degradation' on every SearchMode change.
- QueryTrace at DEBUG: query_id, stages Vec<StageTrace>, total_latency_ms, mode, result_count.

DETERMINISTIC_CHECK: cargo check 0; std::time 0 lines; unwrap() 0; TODO/placeholder 0;
QueryTrace/StageTrace hit; file_write/invalidate_by_file hit; rrf_score|1.0.*60 hit;
max_seq_len/truncat hit; .save/.load hit; filtered_search hit.

ADVERSARIAL:
- silent CPU fallback on GPU init fail — log provider after load.
- non-deterministic usearch order on ties — sort DESC score, ASC chunk_id tiebreaker.
- wrong RRF k — const k=60 not configurable.
- SSB DELETE race with mid-flight query — DELETE before query() returns.
- wrong output dim — assert in readiness_check.

ACCEPTANCE: cargo check 0; no std::time; no unwrap; SSB handler registered; QueryTrace emitted;
RRF k=60; truncation warning; usearch save/load + filtered_search; canary warmup; no TurboQuant code.

STEP 5 — MEASURE (operational, not unit gate)
PRE_GATE: Step 4 audited; models at .elai_cc/models/graphcodebert/ and granite-embedding-small/.
TASK: embed 100 chunks; 5 representative queries; compare dual-vector recall vs BM25;
re-embed 100 chunks -> 100% cache hits; log measurements.
Output: workflow/reasoning/phase-1C-measurements.md:
  throughput_code_model, throughput_text_model, cache_hit_rate, dual_vector_vs_bm25.
If throughput <10 chunks/sec CPU: WARN; note HNSW impact.
If cache_hit_rate <90%: BLOCK and investigate.
DETERMINISTIC_CHECK: file + 4 keys; cache_hit_rate >=90; no TODO/N/A.
ADVERSARIAL: all ONNX errors Err with path; cache counter explicit (hits/misses).
ACCEPTANCE: file + 4 entries; cache_hit_rate >=90; dual-vector <= BM25 is INFO, not FAIL.

EXIT CRITERIA (14 items):
1. cargo check 0 with ort.
2. >=20 unit tests pass.
3. Contract test dual-vector passes.
4. Canary both models <5s (or Degraded correct); canary = ONNX warmup.
5. Cache insert/lookup/invalidation tests.
6. CB opens after 3/60s.
7. std::time == 0 in vector/.
8. cache_hit_rate >=90%.
9. HNSW=usearch v2.24.0 with save/load.
10. TurboQuant NOT implemented (deferred 1G).
11. RRF k=60 unit-tested.
12. max_seq_len=512 truncation + warning.
13. usearch filtered_search isolation.
14. Arc<Mutex<Session>> concurrent embed test no deadlock.

MEASUREMENTS: throughput chunks/sec per model; p50/p99 latency;
RRF hybrid recall vs BM25/vector-only (5 queries, 3 comparisons);
first-query latency warmed vs cold; cache hit rate; canary <5s;
cross-encoder p99 50 candidates <50ms CPU.
