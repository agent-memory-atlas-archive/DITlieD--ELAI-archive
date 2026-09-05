B-32 — Adaptive Minified File Chunking + Universal Char Cap (Step A)
Session: s-50960 | Phase: PLANNING -> EXECUTING (Step A; Step B deferred on lock release)
Source synthesis: workflow/sessions/s-50960/synthesis-B-32.md (D1-D10, E1-E13)

PRE-DECIDED DESIGN — D1..D10 (no further confirmation)

D1. THREE chunk producers must call is_minified-first + apply_char_cap-last:
    a. crates/knowledge/src/bm25.rs:366 Bm25Index::chunk_file
    b. crates/knowledge/src/bm25.rs:453 Bm25Index::make_chunks_from_content
    c. crates/knowledge/src/vector/indexer.rs:331 vector::indexer::chunk_file (standalone fn)
    Justification: missed any one and "apply_char_cap last in ALL paths" stub_ban acceptance silently fails. Verified four call sites in impact-radius-B-32.md.

D2. BM25Chunk literals at pipeline.rs:806 + startup.rs:270 ALSO wrapped with apply_char_cap on construction. Defense-in-depth: manual literals bypass the dispatch invariant. Verified via impact-radius-B-32.md "OTHER CHUNK PRODUCERS".

D3. SSB lock split. Step A excludes orchestrator/ssb.rs + orchestrator/indexing_service.rs (locked by s-21092). Observable trigger replacement: tracing::info!(target="knowledge::chunking", path, char_count, chunk_count, avg_line_length, "minified chunked"). Step B added once s-21092 releases (poll workflow/sessions/state.json before EXECUTING Step B).

D4. Provider-boundary char cap (GAP-1) PROMOTED from "verify" to "implement". Add defensive cap at vector/indexer.rs::index_chunk's embed_code call site (~line 172) with tracing::warn! per regression-detector pattern. Counts as M12 in micro-tasks.

D5. ChunkKind::Minified variant added (additive). Updates required in bm25.rs ChunkKind enum + as_str (~60) + from_str (~74). from_str default branch returns ChunkKind::File — old persisted rows stay safe (round-trip-safe).

D6. Chunk uniqueness for minified BM25Chunk: line_start = 1 + chunk_index, line_end = line_start (per-chunk synthetic). NO byte_offset field added (no schema migration). Documented as "minified chunks use synthetic line numbers".

D7. UTF-8 helper: codebase idiom `let mut end = cap; while end > 0 && !s.is_char_boundary(end) { end -= 1; }`. NOT std::str::floor_char_boundary (only stable on 1.79+, but consistency with codebase idiom matters more — 6 in-tree precedents). Lift into knowledge::bm25::chunking::cap::floor_char_boundary(s, cap) -> usize. Precedents: vector/indexer.rs:380, telescope.rs:104, bash.rs:358.

D8. Fixture location: .elai_cc/crates/knowledge/tests/fixtures/minified_project/ (crate-local Cargo idiom). NOT docs/fixtures/. Tests resolve via std::env::current_dir() + tests/fixtures/...

D9. Dashboard wiring SKIPPED for B-32 (IndexingProgressPanel.svelte does not exist; B-31 unshipped). Documented as INFO-only deferred.

D10. apply_char_cap EXCLUDES ChunkKind::File (zero-orphan File chunk at bm25.rs:392/474 must carry full content for "show file" UX). apply_char_cap matches `if chunk.kind == ChunkKind::File { return chunk; }` per chunk in the map. Documented as explicit exception.

LOCK CONFLICT WORKAROUND (s-21092 holds lib.rs)
The chunking module is declared inside bm25.rs (which session s-50960 owns) using:
    #[path = "chunking/mod.rs"]
    pub mod chunking;
This bypasses the lock on lib.rs. Once s-21092 releases lib.rs, main session can refactor to a clean `pub mod chunking;` declaration in lib.rs. Module path remains `knowledge::bm25::chunking::*` for the duration of the lock; tests reference this path explicitly.

UNGROUNDED ITEMS (from grounding sweep) — to verify post-merge
- nomic-embed-code-v1 token window (verified upstream): SocratiCode constants.ts states MAX_CHUNK_CHARS=2000 chars to leave headroom under provider context. Value matches SocratiCode reference (constants.ts:128). ELAI deployment cap verified at 2000 chars conservative for any embedding model — verify against deployed instance before tightening below 2000.
- Granite token window: same status (matches SocratiCode reference; verify before tightening).

INDEX_FORMAT_VERSION
NOT bumped. Adding ChunkKind::Minified is read-compatible because from_str default falls back to File. Stays at 7. Old persisted indexes survive untouched.

CITATIONS (SocratiCode line refs)
- src/services/indexer.ts:283-465 — chunkFileContent dispatch order (minified-first, applyCharCap-last, AST/line fallback)
- src/services/indexer.ts:344-351 — applyCharCap (substring truncation; Rust must use floor_char_boundary)
- src/services/indexer.ts:363-411 — chunkByCharacters (offset-loop, backward-scan for newline/space/tab/semicolon/comma)
- src/services/indexer.ts:432-439 — avg_line_length > MAX_AVG_LINE_LENGTH detection
- src/constants.ts:115 — MAX_AVG_LINE_LENGTH = 500
- src/constants.ts:128 — MAX_CHUNK_CHARS = 2000

OBSERVABLE TRIGGER (Step A)
trigger: `cd .elai_cc && .\elai.bat index crates\knowledge\tests\fixtures\minified_project`
expect_log: tracing INFO `target=knowledge::chunking path="vendor/bundle.min.js" char_count=50000 chunk_count>=25 avg_line_length=50000 message="minified chunked"`
expect_artifact: tantivy index has >=25 chunks for vendor/bundle.min.js, each chunk_chars <= 2000 (asserted via Bm25Index::query in test, NOT sqlite — bm25_chunks table does not exist).

DEFERRED — STEP B TRIGGER GATE
Poll: `jq '.["s-21092"].locks // empty' workflow/sessions/state.json`
If empty, execute E14-E18 (SsbActionType::MinifiedFileChunked variant + payload + SCHEMA_VERSION 35->36 + IndexingService SSB emit + dst.rs match + contract test).
