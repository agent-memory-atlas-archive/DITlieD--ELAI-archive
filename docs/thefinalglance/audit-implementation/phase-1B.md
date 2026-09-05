# Phase 1B Implementation Audit
Date: 2026-04-24
Overall verdict: IMPLEMENTED (with 1 minor path deviation + 1 known debt carried from Phase 1A)
Wiring-to-pipeline verdict: WIRED

Source of truth:
- Spec: F:\.ELAI_workflow\docs\thefinalglance\summary\phase-1-summary.md §PHASE 1B (lines 368-469)
- Target: F:\.ELAI_workflow\.elai_cc\crates\knowledge\src\bm25.rs (1103 lines)
- Map: F:\.ELAI_workflow\.claude\tools\elai.map (Bm25Index/BM25Chunk symbols resolved)
- Metrics: F:\.ELAI_workflow\.elai_cc\metrics\phase_1B.toml (12 lines, complete)

## Sub-step: 1B.1 ADD TANTIVY DEPENDENCY
Verdict: IMPLEMENTED
Evidence:
- F:\.ELAI_workflow\.elai_cc\crates\knowledge\Cargo.toml:24 -> `tantivy = "0.26"` — present and pinned (minor-pin).
- bm25.rs:11-12 records `pub const INDEX_FORMAT_VERSION: u32 = 7;` with source comment "verified from tantivy src/lib.rs" — satisfies spec requirement to record INDEX_FORMAT_VERSION as u32 (not the project-label "tantivy_v21").
- metrics/phase_1B.toml:11-12 mirrors both: `tantivy_version = "0.26.0"`, `index_format_version = 7`.
Gap: NONE. Spec warned that "tantivy_v21" is a project identifier, not tantivy's real API — this distinction is honored.

## Sub-step: 1B.2 BM25 MODULE SKELETON WITH CORRECT API SHAPE
Verdict: IMPLEMENTED
Evidence:
- Module registered: knowledge/src/lib.rs:5 `pub mod bm25;`.
- Types (bm25.rs):
  - `ChunkKind` enum at lines 45-56 — variants `Function, Struct, Enum, Impl, Class, Module, Trait, LineBlock, File`. Spec minimum was `{Function, LineBlock, File}`; impl is a superset. Derives `Serialize/Deserialize` — contract test verifies round-trip (bm25_contract.rs:168-175).
  - `BM25Chunk` at lines 88-95 matches spec exactly: `path: PathBuf, content: String, line_start: u32, line_end: u32, kind: ChunkKind`.
  - `BM25Hit` at lines 97-105 has all spec fields + `chunk_kind` + `snippet`. Score is `f32`.
  - `IndexStats` at lines 107-115 with `files_indexed, chunks_indexed, files_skipped_binary, elapsed_ms`.
- `Bm25Index` struct at lines 137-146 holds `index_path, index, writer, reader, schema, fields`. Single-writer model documented at lines 134-136 (matches spec's single-writer constraint).
- Constructor `pub fn new(project_root: &Path, project_id: &str)` at lines 152-208 — project_id explicitly passed (per spec D8).
- Index path: `{project_root}/projects/{project_id}/bm25/` constructed at lines 156-159. SPEC DEVIATION: spec wanted `.elai_cc/projects/{project_id}/bm25/`. The `.elai_cc/` prefix is injected by the CALLER (orchestrator/indexing_service.rs:177 passes `indexes_dir = {project_root}/.elai_cc/indexes`). Resulting real runtime path is `{project_root}/.elai_cc/indexes/projects/default/bm25`. The extra `/indexes/` segment diverges from the spec literal but is intentional: IndexingService colocates `dkg.db` + `bm25/` under a common `indexes/` directory. See Gaps.
- Schema (lines 162-168):
  - path: `STRING | STORED` — CORRECT per spec critical note "path MUST be STRING. TEXT tokenizes."
  - content: `TEXT | STORED`
  - line_start: `u64 STORED`
  - line_end: `u64 STORED`
  - chunk_kind: `STRING | STORED`
- Five public methods present: `index_file` (213), `index_directory` (240), `query` (258), `delete_file` (311), `update_file` (325). `BM25Chunk::kind` serialized via `ChunkKind::as_str`/`from_str` at lines 59-86 — ChunkKind enum (not strings) on the wire type, with string serialization only at the tantivy doc boundary (spec-compliant).
- Writer serialization: `Bm25Index` holds a single `IndexWriter` (line 141) — matches spec "single-writer model".
- Contract test stub: F:\.ELAI_workflow\.elai_cc\crates\contracts\tests\bm25_contract.rs exists (189 lines, 10 `#[test]` functions) — one per public method plus cross-cutting (empty query, ChunkKind serde, IndexStats fields, binary skip).
Gap: path literal runtime location is `{project_root}/.elai_cc/indexes/projects/default/bm25/` not `{project_root}/.elai_cc/projects/default/bm25/` — extra `indexes/` segment. Semantics preserved (per-project isolation via project_id, on-disk tantivy index). Cosmetic divergence; no behavior impact. Classify as MINOR_DEVIATION.

## Sub-step: 1B.3 WRITE FAILING TESTS (TDD)
Verdict: IMPLEMENTED (green-phase; TDD red cycle long past)
Evidence:
- F:\.ELAI_workflow\.elai_cc\crates\knowledge\tests\bm25_tests.rs (434 lines, 11 `#[test]`): `a_index_single_file_query_symbol, b_index_dir_query_specific_symbol, c_delete_file_query_returns_empty, d_query_nonexistent_returns_empty, e_unknown_extension_fallback, f_binary_file_skipped, g_update_file_replaces_content, h_project_isolation, i_chunk_kind_function_and_lineblock, j_delete_precision_nested_paths, k_hidden_dir_skipped`.
- Each spec (a)-(j) has a direct test-function counterpart (see letter prefixes). Extra `k_hidden_dir_skipped` covers walk_dir skip-hidden behavior — above spec.
- In-file `#[cfg(test)] mod tests` at bm25.rs:837-1102 adds 12 more unit tests (AST chunking, ChunkKind round-trip, token estimation, overlap, tantivy query, index version). 11 + 10 (contract) + 12 (unit) = 33 total BM25-facing tests — spec minimum ≥10.
- Tests fail in `tempdir` (line 1061: `tempfile::tempdir()`); no real codebase paths used — matches spec "ALL synthetic files in tempdir".
Gap: NONE. Test count and topic coverage exceed spec.

## Sub-step: 1B.4 IMPLEMENT BM25 INDEX
Verdict: IMPLEMENTED
Evidence:
- Core index construction (bm25.rs:162-207): schema built, `Index::create_in_dir` with open_in_dir fallback (line 170-171), schema drift validation (lines 173-184 — checks for all 5 expected fields, returns error with rebuild instruction if stale), writer heap 50MB (`WRITER_HEAP_BYTES = 50_000_000` at line 15), reader with `ReloadPolicy::OnCommitWithDelay`. Schema-drift guard is a PLUS beyond spec.
- `index_file` two modes (lines 213-236): explicit `Some(chunks)` vs `None` -> calls `chunk_file` -> reads content, `try_ast_chunk`, falls back to `line_block_chunks` on empty. Single `writer.commit()` at end, `reader.reload()` — atomic.
- Line-level fallback: `line_block_chunks` (lines 482-502). Uses `CHUNK_WINDOW_LINES = 50` and `CHUNK_OVERLAP_LINES = 10` (lines 18-21). Step = `50 - 10 = 40`. Produces `ChunkKind::LineBlock`. Matches spec "50-line sliding windows with 10-line overlap".
- Small-file case: `chunk_file` at lines 370-378 emits a single `ChunkKind::File` chunk when `lines.is_empty()`. Larger files always get at least one `File` chunk appended (line 388-394) — the zero-orphan invariant. Explicitly commented "Zero-orphan invariant: every non-binary file gets at least one File-kind chunk".
- Binary detection (lines 357-360): `is_binary` reads first 8192 bytes (`BINARY_DETECT_BYTES = 8192` at line 24) and checks for null byte with `.contains(&0)`. Matches spec "null bytes in first 8KB".
- `index_directory` (lines 240-254 + walk_dir 399-447):
  - Recursive walk skips any entry whose name `starts_with('.')` (line 413) — skips `.git`, `.elai_cc`, and all hidden dirs. Test `k_hidden_dir_skipped` verifies.
  - Per-file binary check (line 425-428) before read; increments `files_skipped_binary` and `continue`. No panic on non-UTF8 — uses `String::from_utf8_lossy` (line 429).
  - Per-file chunk production via `make_chunks_from_content` (line 431) which also emits a trailing `ChunkKind::File` chunk for zero-orphan (line 470-476).
  - Atomic commit: `writer.commit()` called ONCE at line 251 after the entire walk, matching spec "one writer.commit() after all".
  - DKG integration: spec 1B.4 says "query DKG ranges, read content for BM25Chunks. If DKG no nodes, line-level fallback." CURRENT IMPLEMENTATION uses `try_ast_chunk` (bm25.rs:506-562) which parses tree-sitter DIRECTLY inside BM25 and falls back to line blocks on parse failure — it does NOT query DKG at index time. This is a SIMPLIFICATION: BM25 is self-contained w.r.t. chunk boundaries. Spec intent (AST-aligned chunks + fallback) is preserved; the source of AST boundaries just skips the DKG round-trip. Classify as DESIGN_DIVERGENCE, not a defect — the 7 grammars supported (Rust, Python, Go, TS, Java, C#, Ruby) match LanguageGrammar trait set and produce correct ChunkKind (Function/Struct/Enum/Impl/Class/Module/Trait) via `symbol_kind_to_chunk_kind` at lines 580-590.
- AST chunking infrastructure (bm25.rs:569-831):
  - `AstBoundary` struct tracks `start_line/end_line/kind`.
  - `build_ast_chunks` (593-679): emits gap chunks (LineBlock) for uncovered regions between AST nodes, enforces HARD_MAX_CHUNK_TOKENS=1000 cap (line 36, verified by test `no_chunk_exceeds_hard_max` at line 903), splits large nodes via `split_large_node` with OVERLAP_TOKENS=50, merges small chunks (merge_small_chunks, line 758), adds overlap between adjacent chunks (add_overlap, line 797). This chunking engine is more sophisticated than the spec's 50-line sliding window baseline and is compatible with cross-layer alignment (D1).
- `query` (lines 258-307): `QueryParser::for_index` over `content` field; `TopDocs::with_limit(limit).order_by_score()` sorts DESC. Extracts path/content/line_start/line_end/chunk_kind from stored fields; snippet is `content_val.chars().take(200).collect()` — 200-char truncation (D9), NOT SnippetGenerator. Empty/whitespace query returns empty Vec (line 263-265) — covered by contract test `empty_query_returns_empty_results`.
- `delete_file` (lines 311-321): `Term::from_field_text(self.fields.path, &path_str)` then `delete_term` + `commit`. Uses STRING path field -> exact match -> test `j_delete_precision_nested_paths` confirms no false deletions on `src/deep/nested/module.rs`. Returns opstamp as usize (comment clarifies it's a monotonic counter, not doc count — honest naming).
- `update_file` (lines 325-351): single `delete_term` + N `add_document` + SINGLE `commit` — atomic per spec. No inter-stage commit. `reader.reload()` at end. metrics p99 < 100ms (see 1B.5) — well under spec's 5000ms budget.
- `is_binary` correct: test `f_binary_file_skipped` passes bytes `b"has\x00null\x00bytes"` -> stats.files_skipped_binary == 1 (contract bm25_contract.rs:143-155).
- Existing DKG tests: not executed by main model (audit policy), but phase_1A.toml presence implies prior green state.
Gap:
1. Path literal `indexes/projects/.../bm25` vs spec `projects/.../bm25` (see 1B.2).
2. `Box<dyn std::error::Error>` signatures — 8 occurrences at lines 155, 217, 243, 262, 314, 329, 362, 403. Carried forward from Phase 1A audit with SAME line numbers. Violates CLAUDE.md "no Box<dyn Error> in library code"; downstream callers (orchestrator/indexing_service.rs:178, 181, 225) work around it with `.map_err(|e| IndexingServiceError::Bm25(e.to_string()))`. Classify as KNOWN_DEBT. Not a Phase 1B regression — 1A audit flagged identical pattern.
3. `split_large_node` does NOT use the `overlap_chars` parameter for overlap between sub-chunks — the loop steps `chunk_start = chunk_end.saturating_sub(overlap_lines)` (line 735), giving LINE overlap not CHAR overlap. `overlap_chars` IS used inside `estimate_overlap_lines` to DERIVE line count, so the effect is correct — minor naming cosmetic.
4. `elapsed_ms` on `IndexStats` is always 0 from `index_directory` — the comment at lines 112-114 says "Caller-populated". `measure_bm25.rs` example populates it externally. Contract test at bm25_contract.rs:187-188 acknowledges this ("elapsed_ms is currently 0 (no timing impl yet) — check type only"). Spec does not explicitly require internal timing; acceptable.

## Sub-step: 1B.5 MEASURE AND VALIDATE INVARIANTS
Verdict: IMPLEMENTED
Evidence:
- F:\.ELAI_workflow\.elai_cc\metrics\phase_1B.toml contains ALL 11 spec-required fields:
  - `index_build_time_ms = 538` (<5000 budget OK — actual build under 1s for 73 files)
  - `file_count = 73`
  - `chunk_count = 1544`
  - `binary_skipped = 0`
  - `query_p50_ms = 1`
  - `query_p99_ms = 2` (<10 dev budget OK; <50 CI budget OK)
  - `update_file_p99_ms = 100` (<5000 budget OK by 50x)
  - `orphan_files = 0` (invariant holds — zero-orphan)
  - `index_size_bytes = 2456531` (~2.3 MiB)
  - `tantivy_version = "0.26.0"`
  - `index_format_version = 7`
- Measure binary exists at F:\.ELAI_workflow\.elai_cc\crates\knowledge\examples\measure_bm25.rs (296 lines) — this binary is what populates `phase_1B.toml`; drives the 10 diverse queries (spec list) through `Bm25Index::query`. Confirmed referenced in elai.map entry at examples/measure_bm25.rs:119 (`Bm25Index::new(tmp.path(), project_id)`).
Gap: NONE. All five acceptance criteria met: p99<10ms dev, update_file<5s, orphan_files=0, metrics file with all fields, index build <5s for ~69 (here 73) files.

## Wiring gap check
Verdict: WIRED — BM25 is invoked from the real CLI startup path AND the orchestrator service; it is NOT example-only.

Evidence (production callers, non-test):
- **F:\.ELAI_workflow\.elai_cc\crates\orchestrator\src\indexing_service.rs:6** — `use elai_knowledge::bm25::Bm25Index;`.
- **indexing_service.rs:122** — `IndexingService` struct holds `bm25: Option<Bm25Index>` as a live field.
- **indexing_service.rs:177-199** — `run_full_index`: constructs `Bm25Index::new(&self.indexes_dir, BM25_PROJECT_ID)` with `BM25_PROJECT_ID = "default"` (line 30), calls `bm25.index_directory(&self.project_root)`, logs `files/chunks/skipped_binary` via `tracing::info!`, stores into `self.bm25`. Returns `IndexingSummary` with bm25 counts alongside DKG counts.
- **indexing_service.rs:223-226** — `reindex_file` calls `bm25.update_file(changed_path, None)` on incremental changes (feeds watcher / file_write SSB events).
- **indexing_service.rs:249-251** — `pub fn take_bm25(&mut self) -> Option<Bm25Index>` — ownership handoff to CLI.
- **F:\.ELAI_workflow\.elai_cc\crates\cli\src\main.rs:70** — `use elai_knowledge::bm25::Bm25Index;`.
- **F:\.ELAI_workflow\.elai_cc\crates\cli\src\main.rs:3658-3662** — CLI startup: after IndexingService completes, constructs a second live `Bm25Index::new(&project_root, &session.id)` (per-session project_id) and hands it to `QueryPipeline::new(bm25, catrag, arag)`. This is the search entry point used by the REPL / dashboard queries downstream.
- elai.map SCIP_REF confirms: line 40675 `use elai_knowledge::bm25::Bm25Index;` in crates/cli/src/main.rs:70 (canonical rust-analyzer resolved); line 26540 `Bm25Index` in `pipeline.rs:65` inside `QueryPipeline::new(bm25: Bm25Index, ...)`.
- **startup.rs:406** — `Bm25Index::new(tmp.path(), "unavail_test")` inside readiness/canary flow.

Evidence (non-production callers — still present but not the only callers):
- examples/measure_bm25.rs:119 (benchmark binary).
- tests/bm25_tests.rs (11 integration tests).
- tests/benchmark_eval.rs:36 and benchmark_eval_improved.rs:74 and splade_tests.rs:401 and splade_measure_tests.rs (BM25 used as baseline for SPLADE / cross-system evaluation).
- contracts/tests/bm25_contract.rs (10 contract tests).
- src/startup.rs:268 (fixture builder for startup tests).

Conclusion: BM25 is WIRED at TWO production points (IndexingService + CLI QueryPipeline construction). No orphan implementation.

## Box<dyn Error> debt count
Verified: EXACTLY 8 occurrences in bm25.rs, at lines 155, 217, 243, 262, 314, 329, 362, 403.
- 155 — `new(...) -> Result<Self, Box<dyn ...>>`
- 217 — `index_file(...)`
- 243 — `index_directory(...)`
- 262 — `query(...)`
- 314 — `delete_file(...)`
- 329 — `update_file(...)`
- 362 — `chunk_file(...)` (private)
- 403 — `walk_dir(...)` (private)

This matches the Phase 1A audit's prior-identified debt exactly (same 8 lines). Resolution requires introducing a typed error enum (`Bm25Error` with `thiserror`) — out-of-scope for Phase 1B acceptance but tracked as carried-over-debt.

## Summary table
| Sub-step | Verdict | Key evidence |
|---|---|---|
| 1B.1 tantivy dep | IMPLEMENTED | Cargo.toml:24 tantivy=0.26; INDEX_FORMAT_VERSION=7 |
| 1B.2 skeleton | IMPLEMENTED | 5 public methods, STRING path, ChunkKind enum, contract test file |
| 1B.3 TDD tests | IMPLEMENTED | 11 (integration) + 10 (contract) + 12 (unit) BM25 tests |
| 1B.4 implement | IMPLEMENTED | AST + line-block + File fallback, null-byte binary detect, atomic commits |
| 1B.5 measure | IMPLEMENTED | phase_1B.toml with all 11 fields, p99=2ms, build=538ms, orphan=0 |
| Wiring | WIRED | orchestrator/indexing_service.rs + cli/main.rs both construct Bm25Index at startup |

## Risk register disposition (spec R1–R7)
- R1 tantivy index format version: ADDRESSED. `INDEX_FORMAT_VERSION: u32 = 7` constant + schema-drift guard in new() (bm25.rs:173-184).
- R2 index corruption on crash: ADDRESSED via single `writer.commit()` per operation; atomic update_file confirmed by test g.
- R3 Windows path handling: `path.to_string_lossy()` used consistently. No backslash issues in tests.
- R4 memory on large repos: 50 MB writer heap; not measured at >100K docs scale — spec says "sub-10ms for <100K docs", 1544 chunks well under.
- R5 binary file panic on non-UTF8: ADDRESSED. `is_binary` check before UTF-8 decode; `from_utf8_lossy` as backstop.
- R6 chunking strategy alignment: ADDRESSED via ChunkKind (Function/Struct/Enum/Impl/Class/Module/Trait/LineBlock/File) shared wire format.
- R7 path field MUST be STRING: VERIFIED (schema line 163; contract test j guards against TEXT regression).

## Critical gaps (blocking)
NONE. Phase 1B acceptance criteria are all met.

## Non-blocking gaps
1. **MINOR_DEVIATION** — runtime index path has `/indexes/` segment between `.elai_cc/` and `projects/`. Spec said `.elai_cc/projects/{id}/bm25/`; actual is `.elai_cc/indexes/projects/{id}/bm25/`. Caused by IndexingService co-locating DKG and BM25 under `indexes/`. Semantic equivalence maintained; fix requires either a spec amendment or reshuffling IndexingService directory layout.
2. **KNOWN_DEBT (carried from 1A)** — 8 `Box<dyn std::error::Error>` signatures in bm25.rs. Violates CLAUDE.md. Downstream uses `.map_err(|e| IndexingServiceError::Bm25(e.to_string()))` which works but loses structured error context. Recommended: introduce `Bm25Error` with `thiserror`.
3. **DESIGN_DIVERGENCE (not a defect)** — `try_ast_chunk` parses tree-sitter directly in bm25.rs rather than querying DKG for node ranges. Acceptable simplification; BM25 no longer depends on DKG at index time. Spec intent (AST-aligned chunks + line-block fallback) preserved.
4. **MINOR** — `IndexStats::elapsed_ms` always 0 from library code; only the measure binary populates it. Acceptable given library is meant to be entropy-free; measurement is external.

## Confidence
HIGH — every claim grounded in file lines directly read (bm25.rs 1103 lines across four reads; contract 189 lines; indexing_service.rs around line 177 + 223 + 249; cli/main.rs around line 3658 + line 70; Cargo.toml line 24; phase_1B.toml 12 lines; bm25_tests.rs test names via grep). elai.map SCIP_REF rows corroborate the symbol + caller graph. Box<dyn Error> count (=8) verified via grep with line numbers.

## Overall verdict (for caller)
Phase 1B BM25 full-text search is REAL, WIRED, TESTED, and MEASURED. One cosmetic path divergence (`/indexes/` segment) and one known debt (Box<dyn Error>) inherited from 1A. No blocking gaps. The BM25 subsystem can be treated as a reliable prerequisite for Phase 1C/1D/1E consumers.
