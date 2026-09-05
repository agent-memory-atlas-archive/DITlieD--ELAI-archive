# B-32: Adaptive Chunking — Minified File Detection + Universal Char Cap
Objective: prevent vendored bundles, minified JS, packed CSS, and
single-line generated files from blowing up the embedding context window
or producing one giant chunk that fails BM25 indexing. Add a
deterministic minified-content detector and a universal character cap
applied to every chunk regardless of strategy.
Estimated effort: 0.5-1 session (~3-5 hours).
Pre-requisites: B-1 (path unification — chunk persistence reads canonical
paths). Bundles cleanly with B-31's per-batch refactor.
Parallel-safe with: B-29, B-30, B-31, B-33, B-34, B-35.

## Context

Phase 3K (RAG production hardening) added AST-aware chunking via
tree-sitter at fn/impl/struct/class boundaries with 50-line line-window
fallback when AST returns empty. The audit
(DETAILED_SUMMARY_02:697-722) confirms it works on real metric data —
R@5=0.6333, R@10=0.7667 across 30 queries.

What it does NOT handle: source files that are valid syntax but consist
of one or two extremely long lines. These appear in real codebases as:
- `webpack.config.js` output: `dist/main.bundle.js` (single 50K-char line)
- Packed CSS: `bootstrap.min.css` (one ~150K-char line)
- Generated protobuf: `*.pb.go` with multi-thousand-char strings
- Minified vendored libraries committed under `vendor/` or `lib/`

When these hit ELAI's chunker:
- AST tree-sitter parses successfully (it IS valid JS)
- One top-level region = one chunk
- That single chunk is 50K+ chars
- Embedding provider (sidecar / OpenAI / local) hits context-window limit
- Either the embedding fails outright or produces noise
- BM25 indexing succeeds but the chunk is meaningless for retrieval

There is no soft cap or detection layer in current ELAI code. The
character-cap is enforced ONLY at the provider boundary, far too late.

SocratiCode handles this correctly. Two patterns to port:

1. **Pre-strategy detection** (`src/services/indexer.ts:432-439`):
   compute `avg_line_length = content.len() / max(line_count, 1)`. If
   `avg_line_length > MAX_AVG_LINE_LENGTH` (500 chars/line), the file is
   flagged minified and immediately routed to a character-based chunker.
   No AST attempt.

2. **Universal char cap** (`src/services/indexer.ts:344-351, 420`):
   `applyCharCap` runs over EVERY chunk regardless of strategy (AST,
   line-windows, character-based). Any chunk exceeding `MAX_CHUNK_CHARS`
   (2000) is truncated. This is the universal safety net mentioned in
   the inline doc: "Must be kept below CHARS_PER_TOKEN_ESTIMATE ×
   model_context_length so that the provider-level pretruncation always
   fires as a second line of defence."

The character-based chunker
(`src/services/indexer.ts:363-411`) uses backwards-scan from a hard
limit to find a safe split boundary (newline / space / tab / `;` / `,`)
so chunks don't break mid-identifier. Chunk IDs use byte offset (not
line number) since minified files may all be one line.

This is small, contained, and unambiguously correct. The fastest B-step
in Part V; ship it alongside B-31.

## Reads (mandatory before coding)

- F:\.ELAI_workflow\docs\.source\SocratiCode-main\SocratiCode-main\src\services\indexer.ts
  lines 283-465 (full chunking dispatch — AST, line-window, character-based)
- F:\.ELAI_workflow\docs\.source\SocratiCode-main\SocratiCode-main\src\constants.ts
  lines 92-128 (CHUNK_SIZE, CHUNK_OVERLAP, MAX_FILE_BYTES,
  MAX_AVG_LINE_LENGTH=500, MAX_CHUNK_CHARS=2000)
- F:\.ELAI_workflow\.elai_cc\crates\knowledge\src\bm25.rs
  (existing chunker dispatch site — find `chunk_file` and
  `make_chunks_from_content`)
- F:\.ELAI_workflow\docs\specs\guide\steps\phase-3K.txt
  (3K.1 adaptive AST chunking — does not address minified)
- DETAILED_SUMMARY_02_current_state.md lines 697-722 (Phase 3K audit;
  confirms gap)

## Tasks

1. Add constants in `crates/knowledge/src/chunking/constants.rs` (create
   module if needed):
   ```rust
   /// Average chars per line above which a file is treated as
   /// minified/bundled and switched to character-based chunking.
   pub const MAX_AVG_LINE_LENGTH: usize = 500;

   /// Hard character limit per chunk payload — universal safety net
   /// applied to every chunk regardless of strategy. Must stay below
   /// EMBEDDING_CTX_TOKENS * CHARS_PER_TOKEN so provider pretruncation
   /// always fires as a second line of defence.
   pub const MAX_CHUNK_CHARS: usize = 2000;
   ```

2. Implement `is_minified(content: &str, line_count: usize) -> bool` in
   `crates/knowledge/src/chunking/detect.rs`:
   ```rust
   pub fn is_minified(content: &str, line_count: usize) -> bool {
       if line_count == 0 { return content.len() > MAX_AVG_LINE_LENGTH; }
       (content.len() / line_count) > MAX_AVG_LINE_LENGTH
   }
   ```
   Avoid alloc — use line_count from caller (already computed during
   line splitting).

3. Implement `chunk_by_characters` in
   `crates/knowledge/src/chunking/character.rs`:
   ```rust
   pub fn chunk_by_characters(
       file_path: &str,
       relative_path: &str,
       content: &str,
       language: &str,
   ) -> Vec<FileChunk> { ... }
   ```
   Algorithm (mirror `src/services/indexer.ts:363-411`):
   - Iterate `offset` from 0 to `content.len()`.
   - `end = min(offset + MAX_CHUNK_CHARS, content.len())`.
   - If `end < content.len()`, scan backwards from `end` to `offset`
     looking for a safe boundary char (`\n`, ` `, `\t`, `;`, `,`).
     Update `end = boundary_idx + 1`.
   - Slice `content[offset..end]` as the chunk content.
   - **Critical**: chunk ID uses byte offset, NOT line number. Many
     minified files are one line, so line-number-based IDs would
     collide. Use `chunk_id(relative_path, offset)` with offset as the
     discriminator.
   - Track `current_line` only for `start_line`/`end_line` metadata —
     count newlines in chunk content, increment from previous.
   - Advance `offset = end`.
   Use `&str` slicing carefully — must split at char boundaries since
   Rust strings are UTF-8 (use `floor_char_boundary` or similar).

4. Implement `apply_char_cap(chunks: Vec<FileChunk>) -> Vec<FileChunk>`
   in `crates/knowledge/src/chunking/cap.rs`:
   ```rust
   pub fn apply_char_cap(mut chunks: Vec<FileChunk>) -> Vec<FileChunk> {
       for c in &mut chunks {
           if c.content.len() > MAX_CHUNK_CHARS {
               // Truncate at char boundary.
               let cap = floor_char_boundary(&c.content, MAX_CHUNK_CHARS);
               c.content.truncate(cap);
           }
       }
       chunks
   }
   ```
   Apply to ALL chunker outputs — AST, line-window, character-based —
   as the last step before returning.

5. Refactor `chunk_file_content` in `crates/knowledge/src/bm25.rs` (or
   wherever the dispatch lives) to follow the SocratiCode order:
   ```rust
   pub fn chunk_file_content(
       file_path: &Path,
       relative_path: &Path,
       content: &str,
   ) -> Vec<FileChunk> {
       let lines: Vec<&str> = content.split('\n').collect();
       let language = language_from_extension(file_path);

       // 1. Minified detection FIRST — before any AST attempt.
       if is_minified(content, lines.len()) {
           tracing::debug!(
               path = ?relative_path,
               avg_line = content.len() / lines.len().max(1),
               "minified content detected — using character-based chunking",
           );
           return apply_char_cap(chunk_by_characters(
               file_path.to_str().unwrap_or(""),
               relative_path.to_str().unwrap_or(""),
               content,
               &language,
           ));
       }

       // 2. Tiny files: single chunk.
       if lines.len() <= CHUNK_SIZE {
           return apply_char_cap(vec![single_chunk(...)]);
       }

       // 3. AST-aware chunking (existing path).
       let regions = ast_boundaries(content, &language);
       if !regions.is_empty() {
           return apply_char_cap(chunk_by_ast_regions(...));
       }

       // 4. Line-window fallback.
       apply_char_cap(chunk_by_lines(...))
   }
   ```

6. Add SSB action `MinifiedFileChunked` payload
   `{ relative_path, char_count, chunk_count, avg_line_length }` for
   one-shot logging when a file is detected as minified. Surface in
   ActivityFeed under "Indexing" filter so users see why a vendored
   bundle was treated specially.

7. Verify provider-side pretruncation still fires as second line of
   defence. Find the embedding-provider call site and confirm it has
   its own char or token cap; if not, add one with logging "chunk
   pretrunc fired — should not happen if MAX_CHUNK_CHARS configured
   correctly". This is the engineering-fail-safe pattern: the universal
   cap should make this case unreachable, but the code path exists in
   case future changes break the invariant.

## WIRING_CONTRACT

produces_symbols:
  - knowledge::chunking::constants::{MAX_AVG_LINE_LENGTH, MAX_CHUNK_CHARS}
  - knowledge::chunking::detect::is_minified
  - knowledge::chunking::character::chunk_by_characters
  - knowledge::chunking::cap::apply_char_cap
  - core::ssb::SsbActionType::MinifiedFileChunked (new variant)
consumes_symbols:
  - knowledge::bm25::chunk_file_content → calls is_minified first,
    apply_char_cap last
  - knowledge::bm25::make_chunks_from_content → same
  - any other chunk producer in knowledge crate (audit during refactor)
reachable_from:
  - orchestrator::indexing_service::IndexingService::build_index (during
    Phase B chunking)
  - watcher service incremental update path
stub_ban:
  - apply_char_cap is the LAST step before returning from any
    chunk-producing function (regression-tested via inspection)
  - MAX_CHUNK_CHARS is referenced from at least 2 sites: chunking/cap.rs
    and provider boundary (defense-in-depth)
frontend_pair: B-31 (per-batch indexing surfaces minified detection in
  IndexingProgressPanel)
observable_effect:
  trigger: "elai.bat index docs/fixtures/minified_project/" where the
           fixture contains a 50K-char single-line minified.js
  expect_ssb: MinifiedFileChunked action with char_count > 50000,
              chunk_count >= 25 (50K / 2K)
  expect_artifact: tantivy index has multiple chunks for that file with
                   each chunk_chars <= 2000

## DETERMINISTIC_CHECK

```bash
# 1. Unit tests pass
cargo test -p knowledge --test chunking
# Expected: all tests including is_minified, chunk_by_characters,
# apply_char_cap, char-boundary safety green.

# 2. Fixture project: minified file produces multiple chunks
elai.bat index docs/fixtures/minified_project
sqlite3 .elai_cc/.elai_cc/indexes/dkg.db \
  "SELECT COUNT(*) FROM bm25_chunks WHERE relative_path='vendor/bundle.min.js';"
# Expected: >= 20 chunks for a 50K-char file (50000 / 2000 = 25 floor).

# 3. No chunk exceeds MAX_CHUNK_CHARS
sqlite3 .elai_cc/.elai_cc/indexes/dkg.db \
  "SELECT MAX(LENGTH(content)) FROM bm25_chunks;"
# Expected: <= 2000 across the entire fixture.

# 4. Search returns reasonable results on minified content (semantic +
#    BM25 still find substrings)
elai.bat repl "/search 'webpack runtime'" --project docs/fixtures/minified_project
# Expected: returns chunks from the minified file with content snippets
# (truncated but findable).

# 5. SSB MinifiedFileChunked action emitted
sqlite3 .elai_cc/.elai/sessions/ssb.db \
  "SELECT COUNT(*) FROM ssb WHERE action_type='MinifiedFileChunked';"
# Expected: >= 1 per minified file in fixture.

# 6. UTF-8 safety: minified file with non-ASCII content (e.g. emoji
#    inside a string literal) does not panic on chunk boundary
elai.bat index docs/fixtures/minified_utf8/
# Expected: no panics; all chunks valid UTF-8 in dkg.db.
```

## ACCEPTANCE

- [ ] is_minified function added with deterministic threshold (500 char
      avg line length)
- [ ] chunk_by_characters implemented with safe-boundary backwards-scan
      (newline / space / tab / `;` / `,`)
- [ ] apply_char_cap applied as the last step in ALL chunker dispatch
      paths (AST, line-window, character-based)
- [ ] Chunk IDs use byte offset for character-based chunks to avoid
      single-line collision
- [ ] UTF-8 char-boundary safety preserved (no mid-codepoint split panics)
- [ ] MinifiedFileChunked SSB action emitted per detected file
- [ ] Fixture project at `docs/fixtures/minified_project/` includes a
      ~50K-char minified.js, ~150K-char minified.css, and a
      multi-thousand-char `*.pb.go` for regression coverage
- [ ] Provider-boundary pretrunc remains as defense-in-depth with log
      "should not fire if MAX_CHUNK_CHARS correct"
- [ ] Phase 3K metric file unchanged (chunking strategy is internal —
      retrieval quality on the original 30 queries should be
      identical or improved)
- [ ] Zero `.unwrap()` in non-test source

## Tests to add

- `crates/knowledge/tests/chunking_minified.rs`:
  - `test_is_minified_detects_single_line_50k_chars`
  - `test_is_minified_rejects_normal_code`
  - `test_chunk_by_characters_respects_max_chunk_chars`
  - `test_chunk_by_characters_safe_boundary_split`
  - `test_chunk_by_characters_byte_offset_unique_ids`
  - `test_apply_char_cap_truncates_oversize_chunks`
  - `test_apply_char_cap_utf8_boundary_safe` (input contains 4-byte
    UTF-8 codepoint at the cap boundary)
  - `test_minified_file_emits_ssb_action`
- `crates/knowledge/tests/chunking_dispatch.rs`:
  - `test_dispatch_routes_minified_to_char_chunker`
  - `test_dispatch_routes_normal_to_ast_then_lines`
  - `test_apply_char_cap_called_in_every_branch` (via per-branch
    assertion that no chunk exceeds MAX_CHUNK_CHARS even when AST
    produces a giant region)
- Add fixture files under `docs/fixtures/minified_project/`:
  - `vendor/bundle.min.js` (50K-char single line, valid JS)
  - `static/bootstrap.min.css` (150K-char single line, valid CSS)
  - `gen/api.pb.go` (multi-thousand-char string literals)
  - `vendor/bundle_utf8.min.js` (50K-char with 4-byte emoji every 1000 chars)

## Files touched

- CREATE: `crates/knowledge/src/chunking/mod.rs` (if not exists; reuse
  existing `chunking` module path if it does)
- CREATE: `crates/knowledge/src/chunking/constants.rs`
- CREATE: `crates/knowledge/src/chunking/detect.rs`
- CREATE: `crates/knowledge/src/chunking/character.rs`
- CREATE: `crates/knowledge/src/chunking/cap.rs`
- MODIFY: `crates/knowledge/src/bm25.rs` (rewire `chunk_file_content`
  dispatch to call is_minified first, apply_char_cap last)
- MODIFY: `crates/knowledge/src/lib.rs` (export `chunking` module if new)
- MODIFY: `crates/core/src/ssb/action_types.rs` (MinifiedFileChunked
  variant)
- MODIFY: `crates/dashboard-v2/src/lib/components/IndexingProgressPanel
  .svelte` (V2 path — was V1 `crates/dashboard/svelte/src/`. Created in
  B-31; this B-N adds an optional "Detected N minified files; chunked by
  character" line)
- CREATE: `docs/fixtures/minified_project/` (with files listed above)
- CREATE: `workflow/reasoning/B-32-decisions.md`

## Decision doc

Write `workflow/reasoning/B-32-decisions.md` documenting:
- MAX_AVG_LINE_LENGTH=500 — same as SocratiCode. Document why this
  threshold catches real bundlers without flagging legitimate
  long-lined source (e.g. CSV in test fixtures, very long Rust generic
  signatures)
- MAX_CHUNK_CHARS=2000 — verify against ELAI's embedding model context
  window. nomic-embed-text-768d has 8192 token / ~32K char window;
  Granite has 512 tokens. Pick conservative 2000 to fit smallest
- Whether to truncate or split-then-discard the suffix. Default
  truncate (matches SocratiCode); split-then-discard would require
  larger refactor
- Whether the fallback character-based chunker should overlap chunks
  (like CHUNK_OVERLAP=10 lines for line-window) or non-overlapping.
  Default non-overlapping for minified content; overlap doesn't add
  retrieval value when content is opaque
- Whether to skip indexing minified files entirely (configurable via
  env `ELAI_SKIP_MINIFIED=1`). Default off; some users do want vendored
  bundles searchable

## Side benefits unlocked

- Fixed-cost ceiling on chunk size eliminates a class of provider
  errors (token-limit rejection, batch failure, embedding noise)
- Vendored libraries committed under `vendor/`, `lib/`, `dist/` are
  searchable instead of being silently broken
- One-line generated code (protobuf, Thrift, GraphQL codegen) indexes
  correctly
- Faster indexing — character-based chunker is O(n) byte scan vs
  AST parse + traversal which is O(n) but with parser allocations
- Defensive-depth pattern (universal cap as last step) hardens the
  retrieval pipeline against future regressions

## Next

B-31 (Resumable Batched Indexing) — natural sibling, shares chunking
  refactor.
B-29 (Symbol-Level Impact API) — independent, parallel-safe.
B-35 (Context Artifacts Pipeline) — uses the same chunker for
  artifact text.
