Phase 1H SPLADE Measurement Decision
Date: 2026-04-28 | Session: s-27276

FINAL DECISION: DROP

Measurements
  vocab_coverage_code:     0.5795 (58.0%) — PASS (threshold: >= 0.3)
  model_variant_chosen:    PP_en_v1_full (prithivida/Splade_PP_en_v1)
  model_p50_latency_ms:    15
  splade_query_latency_p95_ms: 16
  model_load_time_s:       ~0.94
  recall_delta_mean_pct:   0.00
  recall_delta_std_pct:    0.00
  wilcoxon_W:              0.00
  wilcoxon_p_value:        1.0000
  wilcoxon_n_pairs:        1
  binary_size_delta_mb:    3.62

Candidates Evaluated
  Candidate A (PP_en_v1_full): EVALUATED — p50=15ms, recall delta=0.0%
  Candidate A-quant (int8):    SKIPPED — models/splade-quant not staged
  Candidate B (splade-v3):     SKIPPED — models/splade-v3 not staged

Decision Rationale (spec phase-1H.txt lines 183-291, section D)
  KEEP threshold: mean_delta >= 5% AND p_value < 0.05
  Measured mean_delta = 0.00% (threshold unmet: 0.00 < 5.0)
  Measured p_value = 1.0000 (threshold unmet: 1.0 >= 0.05)
  Measured n_pairs = 1 (below minimum 20 for valid Wilcoxon test)
  Spec clause: n < 20 pairs defaults to DROP

  The synthetic corpus contains 50 queries. 22 had BM25 recall=0 and were excluded
  from the delta calculation per spec (cannot compute relative improvement from zero).
  Of the 28 remaining, only 1 pair showed a non-zero difference (Q08 "logging": bm25=0,
  splade=1.0 — but this pair was excluded since BM25=0). After exclusion, only 1 valid
  paired difference remained (Q01: 0.0%), yielding n_pairs=1, far below the 20-pair
  minimum required for a valid Wilcoxon signed-rank test.

  All 27 queries where BM25 had recall > 0 showed identical recall for BM25 and SPLADE
  (delta=0.0%), meaning SPLADE expansion did not change retrieval outcomes for any query
  where BM25 could already find results. The model vocabulary coverage of 58% is adequate
  but the expansion terms are not improving retrieval rank on this corpus.

  SPLADE shows one notable win: Q08 "logging" returns 0 BM25 results but SPLADE recalls
  the relevant file. This is a real advantage but one data point is insufficient to
  establish a statistically significant improvement per the defined protocol.

Post-DROP Action (per spec phase-1H.txt section F, bullet 8)
  splade feature flag: REMAINS (graceful degradation — model absence disables runtime expansion)
  splade module (crates/knowledge/src/splade.rs): PRESERVED
  Runtime behavior: SpladeExpander::new returns Err if model absent; pipeline degrades gracefully
  binary_size_delta_mb = 3.62MB is acceptable; no binary action required
  crates/cli/Cargo.toml:16 splade feature remains unconditionally enabled (no change needed)

Test Results
  28/28 integration tests passed (splade_measure_tests.rs)
  All 4 model-dependent tests (4a, 4b, 4c, 4d) ran with real model files
  3 reranker unit tests failed (pre-existing, unrelated to SPLADE)
