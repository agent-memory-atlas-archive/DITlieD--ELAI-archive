Phase 1C Grounding Log
Session: s-1375148
Date: 2026-04-06

CRATE VERIFICATION (web search, crates.io)

  ort: VERIFIED. crates.io/crates/ort. pykeio/ort. ONNX Runtime 1.24 wrapper.
    166 reverse deps. GPU backends (cuda, tensorrt). docs.rs/ort.

  usearch: VERIFIED. crates.io/crates/usearch. v2.24.0. Rust bindings exist (not Python-only).
    HNSW ANN search. SIMD-accelerated. Supports f32, f64, i8, f16, binary.
    docs.rs/usearch. Decision: use usearch, no custom stub needed.

  fastembed: VERIFIED. crates.io/crates/fastembed (slug is "fastembed", NOT "fastembed-rs").
    v5.12.0. 98 reverse deps. Wraps ort internally.
    Decision: do NOT import fastembed alongside ort — version conflict risk. Use ort directly.

  blake2: VERIFIED. crates.io/crates/blake2. 80M+ downloads. Provides Blake2s256 hasher.
    The "blake2s" crate exists but has only 1,958 downloads — not the mainstream choice.
    Decision: use blake2 crate with blake2::Blake2s256 for content_blake2s column.

PAPER VERIFICATION

  TurboQuant: arXiv 2504.19874. ICLR 2026 accepted. INGESTED.
    "Online Vector Quantization with Near-optimal Distortion Rate"
    Interface map: workflow/papers/turboquant-interface.md
    RAG topic: paper-turboquant
    Key verified claims:
      b=2 MSE=0.117, b=3 MSE=0.030, b=4 MSE=0.009 (unit-norm)
      768d @ b=3: ~10.7x compression, InProd error ~0.00023/dim
      Indexing: 0.001s vs PQ 494s at 100k vectors
      Requirements: QR rotation (not Hadamard), unit-norm, codebook per (b,d)
    Implementation deferred to Phase 1G. Phase 1C = raw f32 only.

CORRECTIONS APPLIED TO phase-1C.txt

  1. VERIFY blocks -> VERIFIED-CRATE blocks with actual version data.
  2. "fastembed-rs" -> "fastembed" (correct crate slug).
  3. Step 1 actions: removed cargo search commands (already resolved), added cargo add commands.
  4. Step 1 adversarial: updated usearch block (confirmed Rust, no longer Python-only risk).
  5. Step 1 adversarial: updated blake2 block (confirmed Blake2s256 type).
  6. Step 4: usearch confirmed, replaced conditional HNSW stub with direct usearch usage.
  7. Exit criteria #9: updated to reflect usearch confirmed.
  8. fastembed excluded from direct dependency (wraps ort, conflict risk).
  9. TurboQuant UNGROUNDED -> GROUNDED-PAPER with verified distortion values.
  10. Step 4 PRE_GATE updated: TurboQuant ingested, implementation deferred to Phase 1G.
  11. Exit criteria #10 updated: paper ingested, deferred (not "not yet ingested").
  12. Contract test stubs for TurboQuant saved in interface map for future Phase 1G use.

SUBSTANTIVE IMPROVEMENTS (web-search grounded, 2026-04-06)

  13. Arc<Session> concurrency: ort::Session is Send+Sync (docs.rs/ort/session/struct.Session).
      Added Arc<Session> wrapper for concurrent embed() from multiple tokio tasks.
      Added test_concurrent_embed_arc_session unit test.

  14. Tokenizer max_seq_len=512: GraphCodeBERT has 512 max position embeddings
      (huggingface.co/microsoft/graphcodebert-base). Added truncation with tracing::warn.
      Added test_truncation_warning_on_long_input unit test.

  15. usearch persistence: Index::save/load/view_from_file API (docs.rs/usearch/struct.Index).
      Added save after bulk add, load at startup, rebuild-from-cache on load failure.
      Added test_hnsw_index_persistence_roundtrip unit test.

  16. usearch filtered_search: filtered_search(&q, k, predicate) in Rust API (docs.rs/usearch).
      Replaces post-filtering for project isolation. More efficient (filters during traversal).
      Added test_hnsw_filtered_search_project_isolation unit test.

  17. RRF k=60: Reciprocal Rank Fusion formula from Cormack, Clarke & Buettcher (SIGIR'09).
      score(doc) = sum(1/(60+rank)) across BM25, code_vec, text_vec rankers.
      Replaces unspecified "dual-vector rerank". Added test_rrf_fusion_formula unit test.

  18. ONNX warmup: first inference is slow due to graph optimization (microsoft/onnxruntime#19177).
      Canary query now explicitly documented as warmup. Runs before Ready status reported.
      Added first-query-latency measurement to MEASUREMENTS section.

  19. AST chunking dependency: noted that input chunking strategy (tree-sitter AST-based vs
      line-based) is a prerequisite. Referenced code-sage (github.com/faxioman/code-sage)
      as production Rust implementation using tree-sitter.

  Unit test count: 13 -> 20 (+7 tests: 6 for new features + 1 control chars validation).
  Exit criteria: 10 -> 14 (+4 for RRF, truncation, filtered_search, concurrency).

ADVERSARIAL AUDIT FIXES (2026-04-06)

  5 BLOCKs fixed:
  B1. Arc<Session> -> Arc<Mutex<Session>>. ort::Session::run() takes &mut self (docs.rs/ort/session).
  B2. Removed hallucinated view_from_file. Correct API: save() + load() only (docs.rs/usearch).
  B3. Added Step 3.5 tensor format verification. Satisfies Step 4 PRE_GATE that was orphaned.
  B4. RRF citation corrected: SIGIR'09 not CIKM'09 (dl.acm.org/doi/10.1145/1571941.1572114).
  B5. Added SPLADE EVALUATE decision (deferred to Phase 1G). Documented scope explicitly.

  8 WARNs addressed:
  W1. Split GROUNDING section into "GROUNDED" and "UNGROUNDED" sub-headers.
  W2. Added test_query_validation_control_chars (knowledge.txt Section 5). Count now 20.
  W3. Added note to READ BEFORE IMPLEMENTING about usearch EVALUATE flag being outdated.
  W4. Added SSB knowledge_degradation event emission to Step 4 (knowledge.txt Section 5).
  W5. Fixed RRF test to cover all 3 rankers (BM25 + code_vec + text_vec), not just 2.
  W6. Added physical file isolation note to project isolation contract test.
  W7. Canary scope: cross-encoder canary noted as incomplete (acceptable for Phase 1C scope).
  W8. ClockProvider usage: relies on existing entropy quarantine grep check (sufficient).
