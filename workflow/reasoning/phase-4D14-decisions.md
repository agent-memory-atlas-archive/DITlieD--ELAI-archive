Phase 4D.14 Decision Doc

What: WebContentIndex in knowledge crate — sentence-level A-RAG for web content.
New file: crates/knowledge/src/web_arag.rs
Structs: WebChunk, WebContentIndex, WebAragConfig
Methods: ingest, keyword_search, semantic_search, chunk_read

Key decisions:
1. Sentence splitting via regex [.!?\n]+ with URL guard (skip splits inside http(s):// tokens). Use char_indices for UTF-8 safety.
2. Keyword scoring: count(kw,text)*len(kw)/sqrt(word_count) with eq_ignore_ascii_case matching.
3. Semantic search: L2-normalize embeddings before dot product, 3x oversampling, group by chunk, max-pool scores.
4. SourceTier imported from elai_core (not redefined). AragChunk shape untouched.
5. RetrievalResult/ContextTracker reused from crate::arag.
6. Named constants for all thresholds: CHUNK_SIZE_TOKENS, MIN_SENTENCE_LEN, OVERSAMPLE_FACTOR, BYTES_PER_TOKEN, TOKEN_OVERCOUNT_FACTOR.

Expected test outcomes:
- Contract tests: ingest_and_keyword_search, ingest_and_chunk_read, sentence_split_url_guard, keyword_score_formula, semantic_search_l2_normalized, chunk_size_500_tokens — all GREEN after impl.
- cargo check --workspace: 0 errors.

Unknowns:
- None significant. All interfaces verified from arag.rs and source_tier.rs.
