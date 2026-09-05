# Verification Report: phase-1H.txt
Source: F:\.ELAI_workflow\docs\specs\guide\steps\phase-1H.txt
Date: 2026-05-17
Overall status: MOSTLY_DONE
Confidence: HIGH

## Summary
Phase 1H (SPLADE keyword expansion) is substantially implemented. The splade module exists, is correctly feature-gated, and contains SpladeExpander, rrf_fuse, vocab_coverage, wilcoxon_signed_rank, keep_drop_decision, and all measurement utilities. Test suites for steps 2 and 4 are comprehensive. The measurement was executed and a DROP decision was recorded with metrics. One gap: the cli crate unconditionally enables the splade feature rather than keeping it optional, meaning the live_cli SPLADE integration is always compiled rather than being a true opt-in gate.

## Deliverables verified DONE

- splade.rs module created, feature-gated behind cfg(feature="splade") in lib.rs — evidence: .elai_cc/crates/knowledge/src/lib.rs:36-37
- SpladeExpander struct with new(config, clock) -> Result<Self, SpladeError> — evidence: .elai_cc/crates/knowledge/src/splade.rs:109-150
- expand_query(&self, query: &str) -> Vec<ExpandedTerm> returning empty vec for empty input — evidence: .elai_cc/crates/knowledge/src/splade.rs:227-268
- ModelVariant enum (PrePooled, RawMlm) dispatching pooling logic — evidence: .elai_cc/crates/knowledge/src/splade.rs:72-76
- ort dependency in knowledge Cargo.toml (unconditional, serves GraphCodeBERT/Granite) — evidence: .elai_cc/crates/knowledge/Cargo.toml:46
- tokenizers dependency in knowledge Cargo.toml — evidence: .elai_cc/crates/knowledge/Cargo.toml:47
- splade feature flag defined in knowledge/Cargo.toml [features] — evidence: .elai_cc/crates/knowledge/Cargo.toml:58
- Circuit breaker integration via EmbeddingCircuitBreaker wrapping ONNX calls — evidence: .elai_cc/crates/knowledge/src/splade.rs:16, 253-267
- Fallback on SPLADE failure: returns empty expansion (BM25-only path) — evidence: .elai_cc/crates/knowledge/src/splade.rs:258-267
- panic-safe model load via catch_unwind — evidence: .elai_cc/crates/knowledge/src/splade.rs:166-193
- Missing model file returns Err (not panic) — evidence: .elai_cc/crates/knowledge/src/splade.rs:158-163
- Missing tokenizer file returns Err (not panic) — evidence: .elai_cc/crates/knowledge/src/splade.rs:197-203
- vocab_coverage(&self, corpus_tokens: &[String]) -> f64 — evidence: .elai_cc/crates/knowledge/src/splade.rs:423-443
- rrf_fuse(bm25, splade, k, w_bm25, w_splade) with k clamped to min 1 — evidence: .elai_cc/crates/knowledge/src/splade.rs:465-494
- jaccard_similarity, recall_at_k, wilcoxon_signed_rank, keep_drop_decision utility functions — evidence: .elai_cc/crates/knowledge/src/splade.rs:504-719
- SPARSE_ONLY cascade: with splade disabled, degrades to BM25_ONLY (SearchMode::Bm25Only) — evidence: .elai_cc/crates/knowledge/src/degradation.rs:122-123, 317-328
- splade_tests.rs: feature-gated, 5+ unit tests + proptest + integration test — evidence: .elai_cc/crates/knowledge/tests/splade_tests.rs:16, 84-381
- proptest weight invariants: all weights finite and >= 0.0, Unicode no-panic — evidence: .elai_cc/crates/knowledge/tests/splade_tests.rs:310-381
- splade_measure_tests.rs: 4a vocab fitness, 4b model variant selection, 4c 50-query recall measurement, 4d keep/drop decision — evidence: .elai_cc/crates/knowledge/tests/splade_measure_tests.rs:1-1350
- Synthetic corpus: 100 files, 50 queries (15 vocab-mismatch, 15 distractor, 10 keyword, 10 code-syntax) — evidence: .elai_cc/crates/knowledge/tests/splade_measure_tests.rs:619-990
- Wilcoxon signed-rank with normal approximation implemented and applied — evidence: .elai_cc/crates/knowledge/src/splade.rs:544-612
- Measurement executed; results recorded in metrics/phase_1H.toml (decision=DROP, wilcoxon_p=1.0, vocab_coverage=0.58) — evidence: .elai_cc/metrics/phase_1H.toml
- workflow/reasoning/phase-1H-measurements.md logged with per-query breakdown, model table, decision rationale — evidence: F:\.ELAI_workflow\workflow\reasoning\phase-1H-measurements.md
- If DROP: feature flag disabled by default in knowledge/Cargo.toml (default = []) — evidence: .elai_cc/crates/knowledge/Cargo.toml:56
- SPLADE pipeline integration in live_cli with graceful degradation on load failure — evidence: .elai_cc/crates/cli/src/live_cli.rs:1024-1041

## Deliverables PARTIAL

- Feature gate on cli crate usage
  - exists: .elai_cc/crates/cli/Cargo.toml:16 — elai-knowledge = { features = ["splade"] } always on
  - gap: The cli crate unconditionally enables the splade feature for elai-knowledge. The spec says "DROP: feature flag disabled by default, module preserved." The knowledge crate's default feature list is correct (default = []), but cli/Cargo.toml forces splade=on for all cli builds, defeating the optional gate for end-user binaries. The live_cli block still guards on model file existence, so runtime behavior is safe, but the compile-time gate is bypassed.

- Candidate B (naver/splade-v3) model comparison
  - exists: .elai_cc/crates/knowledge/tests/splade_measure_tests.rs:481-567 — v3 path checked, skips gracefully if absent
  - gap: splade-v3 ONNX model was never staged (models/splade-v3 not present). Spec step 4b requires running cross-model recall@10 if Candidate B is available. Per measurements.md: "Candidate B (splade-v3): SKIPPED (models/splade-v3 not staged)". The measurement proceeded with Candidate A only. This is within the spec's allowed fallback ("If splade-v3 export is unavailable, proceed with Candidate A variants only and log"), so it is acceptable per spec but represents incomplete execution of the comparison.

## Deliverables MISSING

None — all spec-required deliverables have implementations or documented skip rationale.

## Deliverables UNVERIFIABLE

- Quantized model (Candidate A-quant) Jaccard >= 0.80 assertion actually ran
  - reason: models/splade-quant was not staged at measurement time. The test code correctly skips. Cannot verify the Jaccard gate executed; only that it would enforce the threshold if staged.
- Single query latency <2s on CPU assertion
  - reason: Phase_1H.toml records p50=15ms, p95=16ms which is well within the 2s limit, but the test itself only eprintln! the latency and does not assert <2000ms in splade_tests.rs (assertion would be in a model-dependent path). The metrics file provides evidence the bound holds in practice.

## What must be done (action list)

- [ ] Fix cli/Cargo.toml to not force splade feature: change `features = ["splade"]` to no features, and gate the SpladeExpander import in live_cli.rs behind `#[cfg(feature = "splade")]`. This restores the compile-time opt-in gate the spec requires.
- [ ] Add a hard assert in splade_tests.rs for latency < 2000ms when a model is present (currently only eprintln — no assertion enforces the acceptance criterion).
- [ ] Stage quantized model (models/splade-quant) or document that A-quant comparison is intentionally deferred, to close the Jaccard-gate verification gap.

## Notes

The DROP decision is well-documented and statistically justified (1 non-zero paired difference, Wilcoxon p=1.0). The module is correctly preserved behind the feature gate in knowledge/, and the SearchMode cascade correctly degrades SPARSE_ONLY to BM25_ONLY when splade is not compiled. The primary actionable gap is the cli crate always enabling the feature, which prevents true opt-in binary builds.
