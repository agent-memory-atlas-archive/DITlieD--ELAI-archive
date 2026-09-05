# Verification Report: phase-3K.txt
Source: F:\.ELAI_workflow\docs\specs\guide\steps\phase-3K.txt
Date: 2026-05-17
Overall status: MOSTLY_DONE
Confidence: HIGH

## Summary
Phase 3K covers eight deliverables across RAG production hardening: adaptive chunking (3K.1), query expansion (3K.2), HyDE (3K.3), query-tier routing (3K.4), retrieval quality benchmark (3K.5), A-RAG keyword/semantic/progressive-disclosure (3K.6), CatRAG symbolic anchoring and key-fact enhancement (3K.7), and post-improvement benchmark (3K.8). All eight steps have substantial implementation. Seven are DONE; one is PARTIAL: the semantic_search in A-RAG (3K.6) uses a proxy score (embedding norm) rather than real dual-vector query-document dot-product as specified. Non-code file heading/paragraph chunking (3K.1) is also absent — the fallback is 50-line windows, not heading/blank-line splits as specified.

## Deliverables verified DONE

- 3K.1 try_ast_chunk: AST-aware splitting reusing LanguageGrammar trait (Rust/Python/TS/Go/Java/C#/Ruby) — evidence: crates/knowledge/src/bm25.rs:579
- 3K.1 ChunkKind enum with Function/Struct/Enum/Impl/Class/Module/Trait/LineBlock variants — evidence: crates/knowledge/src/bm25.rs:48-62
- 3K.1 Tantivy schema extended with chunk_kind, line_start, line_end fields — evidence: crates/knowledge/src/bm25.rs:172-176
- 3K.1 SchemaMismatch guard on stale index (schema validation at open) — evidence: crates/knowledge/src/bm25.rs:184-192
- 3K.1 AST parse failure falls back to 50-line line_block chunks — evidence: crates/knowledge/src/bm25.rs:423-425
- 3K.1 50-token overlap via add_overlap + inject_overlap between adjacent chunks — evidence: crates/knowledge/src/bm25.rs:870 (add_overlap), bm25.rs:749 (called from build_ast_chunks)
- 3K.1 Chunk metadata: file_path, start_line, end_line, chunk_type per chunk — evidence: crates/knowledge/src/bm25.rs:89-95 (BM25Chunk struct)
- 3K.2 expand_query with keyword extraction, code_symbol_variants, synonym_expand — evidence: crates/knowledge/src/query_expansion.rs:86
- 3K.2 Short query (<5 words) skips expansion — evidence: crates/knowledge/src/query_expansion.rs:86 (four_word_query_skips_expansion test at line 255)
- 3K.2 Expansion produces 2-3 variants, no duplicates — evidence: crates/knowledge/src/query_expansion.rs:315 (no_duplicate_expansions test)
- 3K.2 Multi-query results merged and passed to reranker — evidence: crates/knowledge/src/pipeline.rs:1352 (query_expansion stage in pipeline tests)
- 3K.3 HydeStage with circuit-breaker, cache, graceful degradation — evidence: crates/knowledge/src/hyde.rs:79-82
- 3K.3 HyDE uses sidecar via HydeClient trait; SidecarHydeClient in CLI — evidence: crates/cli/src/sidecar_hyde.rs:66
- 3K.3 5-min cache for hypothetical embeddings (CachedHypothesis + FxHashMap) — evidence: crates/knowledge/src/hyde.rs:65, 80
- 3K.3 Short query skips HyDE; complex query generates hypothesis — evidence: crates/knowledge/src/hyde.rs:263, 271 (tests)
- 3K.3 Sidecar failure degrades to direct search, no panic — evidence: crates/knowledge/src/hyde.rs:155-180 (call_through_breaker)
- 3K.4 QueryTier enum (Fast/Standard/Deep/Auto) in lib.rs — evidence: crates/knowledge/src/lib.rs:61
- 3K.4 QueryTier::detect auto-routes on word count and keywords — evidence: crates/knowledge/src/lib.rs:82-102
- 3K.4 query_with_tier on QueryPipeline, backwards-compat query() delegates to Auto — evidence: crates/knowledge/src/pipeline.rs:153, 165
- 3K.4 FAST tier skips reranker/vector; DEEP includes HyDE — evidence: crates/knowledge/src/pipeline.rs:1343, 1371 (tier tests)
- 3K.5 benchmark.rs with build_eval_set() returning 30 queries — evidence: crates/knowledge/src/benchmark.rs:120, 646
- 3K.5 metrics/phase_3K_rag.toml written with Recall@5, Recall@10, MRR, latency per tier — evidence: F:\.ELAI_workflow\.elai_cc\metrics\phase_3K_rag.toml (exists, 30 queries, dated 2026-04-15)
- 3K.6 A-RAG keyword_search implements Score_kw formula (SUM(count * len) / sqrt(char_len)) — evidence: crates/knowledge/src/arag.rs:186-238
- 3K.6 A-RAG keyword_search returns non-empty results with snippet field — evidence: crates/knowledge/tests/arag_tests.rs:64-68 (f_execute_tool_keyword_search_returns_ok)
- 3K.6 Progressive disclosure: snippet extracted via extract_keyword_snippet — evidence: crates/knowledge/src/arag.rs:403 (extract_keyword_snippet), arag.rs:228
- 3K.6 semantic_search implements 3x oversampling (SEMANTIC_OVERSAMPLE_FACTOR) — evidence: crates/knowledge/src/arag.rs:263
- 3K.6 RetrievalResult.snippet field for progressive disclosure — evidence: crates/knowledge/src/arag.rs:62
- 3K.6 arag_contract.rs exists and covers interface — evidence: crates/contracts/tests/arag_contract.rs:13-53
- 3K.7 symbolic_anchor implemented in catrag.rs — evidence: crates/knowledge/src/catrag.rs:183
- 3K.7 key_fact_enhance implemented in catrag.rs — evidence: crates/knowledge/src/catrag.rs:221
- 3K.7 epsilon=0.2, beta=2.5 in CatRagConfig::default — evidence: crates/knowledge/src/catrag.rs:142-143
- 3K.7 extract_named_entities (CamelCase/UPPER_SNAKE rule-based) — evidence: crates/knowledge/src/catrag.rs:26
- 3K.7 catrag tests covering symbolic_anchor and key_fact_enhance — evidence: crates/knowledge/tests/catrag_tests.rs:136 (f), 179 (g)
- 3K.8 metrics/phase_3K_rag_improved.toml written with baseline comparison — evidence: F:\.ELAI_workflow\.elai_cc\metrics\phase_3K_rag_improved.toml (exists, references 3K.5 baseline)

## Deliverables PARTIAL

- 3K.1 Non-code file chunking at heading/paragraph boundaries (spec: markdown splits at heading boundaries, non-code at blank-line paragraphs)
  - exists: fallback is line_block_chunks (50-line windows) at crates/knowledge/src/bm25.rs:425; ChunkKind has no Heading or Paragraph variant
  - gap: Heading and Paragraph chunk kinds not present in ChunkKind enum; no heading-boundary or blank-line-paragraph splitter for .md or plain text files; spec states "Non-code files: split at heading boundaries (markdown) or blank-line paragraphs"

- 3K.6 A-RAG semantic_search uses proxy score (embedding norm), not real query-document dot-product
  - exists: crates/knowledge/src/arag.rs:251-298 (semantic_search with SEMANTIC_OVERSAMPLE_FACTOR and 3x oversampling structure correct)
  - gap: line 261-272 comment: "In production this would use real dual-vector embeddings; for now we use chunk-level embeddings directly" — actual dot-product against query embedding not wired; scoring is query-independent (arag.rs:425 comment confirms)

## Deliverables MISSING
None confirmed missing at structural level; all major deliverables have at least partial implementations.

## Deliverables UNVERIFIABLE

- 3K.5 Regression gate enforcement (auto-block if metric regresses >5%) — the TOML files exist and contain numbers, but no runtime gating code confirmed; could be test-only assertion — searched: "regression_gate", "regress", "5%", "block_if_regress" in elai.map; no enforcer found beyond manual inspection of TOML
- 3K.8 Tokens-per-query reduced by >=30% (progressive disclosure effect) — metric file exists but delta_pct values not confirmed to show >=30% reduction; searched benchmark_eval_improved.rs for assertion; present only as format output, not enforced assertion

## What must be done (action list)

- [ ] Add Heading and Paragraph variants to ChunkKind enum in crates/knowledge/src/bm25.rs and implement heading-boundary splitter (# / ## detection) and blank-line-paragraph splitter for non-code files (crates/knowledge/src/bm25.rs chunk_file path)
- [ ] Wire real query embedding into AragOrchestrator::semantic_search: compute query embedding via existing dual-vector ONNX models (Phase 1C) and use dot-product against chunk embeddings instead of embedding-norm proxy (crates/knowledge/src/arag.rs:251)
- [ ] Add regression-gate assertion in benchmark test (crates/knowledge/tests/benchmark_eval_improved.rs) that asserts each metric delta_pct >= -5.0 to enforce the ">5% regression = block" requirement

## Notes
The ChunkKind enum does not include Heading or Paragraph variants (only Function/Struct/Enum/Impl/Class/Module/Trait/LineBlock/File/Minified). For non-.rs/.py/.ts/.go files, try_ast_chunk returns empty and the fallback is 50-line windows — the spec explicitly requires heading-boundary and blank-line-paragraph splits for markdown and plain text. This is the most significant structural gap. The semantic_search proxy is documented in-code as an interim measure and does not affect the pipeline structure, but it means the "3x oversampling with query-document match" specification is not fully satisfied.
