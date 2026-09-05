# ELAI Phase 1 — Knowledge Engine — Summary Part 3 (1G, 1H, 1I)

(See phase-1-summary-part1.md for index and parts 1A/1B/1C. See part2 for 1D/1E/1F.)

=====================================================================
PHASE 1G — CROSS-ENCODER RERANKER
=====================================================================

Prerequisites: Phase 1E complete.
Status NOTE (2026-04-07): Phase 1F.3-1F.5 DROPPED (ollama no spec decode); no impact on 1G
(1G depends on 1E pipeline, not 1F inference speed).

Goal: cross-encoder reranker (ms-marco-MiniLM-L12-v2) after A-RAG, before token-budget fit.
CPU-only, target <50ms for 50 candidates. Concurrency-safe, bounded queue, request batching,
circuit breaker integrated. ORT-optimized with graph optimization + optional INT8.

GROUNDING:
- cross-encoder/ms-marco-MiniLM-L-12-v2 VERIFIED HF: 33.4M params, 12 layers, hidden=384,
  base=microsoft/MiniLM-L12-H384-uncased, WordPiece/BertTokenizer, requires tokenizer.json.
- ~120MB ONNX fp32 VERIFIED (Xenova/ms-marco-MiniLM-L-12-v2).
- ~30MB ONNX int8 VERIFIED (~4x via dynamic quant, no calibration).
- ort CONDITIONAL pykeio/ort v2.0.0-rc.12 (RC, not stable; API changed rc.8->rc.10; pin exact).
- tokenizers (HF Rust crate) VERIFIED.
- <50ms CPU CONDITIONAL (FlashRank Python sub-20ms; empirical gate — 3 optimization levers).
- ORT_ENABLE_ALL VERIFIED (onnxruntime.ai), NCHWc CPU layout, BERT SQuAD F1 87.05 vs 87.03.
- INT8 up to 6x Xeon VNNI, ~2.9x AVX2 (MS Azure blog).
- intra-op thread scaling CONDITIONAL (Milvus cites 1->8=4.2x, AI-generated source).
- L-6 fallback VERIFIED (22.7M params, 6 layers, MRR@10=36.81 vs L-12 39.02). SPEC AMENDMENT
  REQUIRED — current spec L149 only "skip reranking"; L-6 intermediate tier filed at
  workflow/reasoning/phase-1G-spec-amendment.md.
- skip<=10 GROUNDED knowledge.txt L96.
- Semaphore(1) + depth 4 GROUNDED L219-220.
- batching L221; CB L135/L149; SSB degradation L124 action_type='knowledge_degradation';
  canary includes rerank L159-160.
- injectable clock entropy quarantine.
- max seq 512.
- enable_deterministic_compute VERIFIED (ORT issue #4611) — multi-thread MATMUL non-deterministic
  float reduction without this.

DECISIONS:
- ort vs fastembed-rs: use raw ort + tokenizers (knowledge already uses ort for GraphCodeBERT/
  Granite; allows ORT_ENABLE_ALL, enable_deterministic, int8; fastembed abstracts too much).
- fp32 vs int8: start fp32; if <50ms met with fp32+ORT_ENABLE_ALL+threads stop; else export int8;
  accept int8 only if NDCG@10 delta <0.5. INT8 trigger when baseline p50 >30ms (60% of 50ms budget).
- skip threshold: default 10; sweep 5/10/15/20/30 using NDCG@10.
- batching mechanism: tokio::sync::mpsc bounded channel capacity 4 (over Notify — atomic
  drain-and-batch via try_recv, no lost-wakeup races). Reranker task drains, acquires semaphore,
  merges candidates, single inference, splits by oneshot channels per caller with offset tracking.

STEPS

1G.1 RERANKER MODULE SKELETON + DEPS
PRE_GATE: 1E tests pass (catrag/arag); KnowledgeEngine::rerank exists (record exact signature);
ort compiles after adding ort+tokenizers; ONNX + tokenizer.json accessible.
TASK:
- Add ort (pinned rc) + tokenizers to knowledge/Cargo.toml.
- Create reranker module stub.
- `CrossEncoderReranker` fields: session ort::Session, tokenizer, semaphore tokio::Semaphore,
  request_channel mpsc::Sender, circuit_breaker, clock Arc<dyn ClockProvider>.
- `RerankerConfig`:
    model_path: PathBuf
    tokenizer_path: PathBuf (shared L-12/L-6 BERT vocab)
    fallback_model_path: Option<PathBuf> (L-6)
    quantized_model_path: Option<PathBuf> (int8, generated 1G.5)
    skip_threshold: usize=10
    max_sequence_length: usize=512
    max_queue_depth: usize=4
    intra_op_threads: usize=1
    graph_optimization_level: GraphOptimizationLevel (ORT_ENABLE_ALL)
    enable_deterministic_compute: bool=true
- Implement `KnowledgeEngine::rerank` trait stub matching exact signature.
- Wire into pipeline between A-RAG and token-fit. In SPARSE_ONLY/BM25_ONLY: skip (L118-119).

DETERMINISTIC_CHECK: cargo check 0; CrossEncoderReranker grep; fn rerank signature matches;
tokenizers/ort in Cargo.toml (ort pinned); graph_optimization_level|GraphOptimizationLevel|
ORT_ENABLE_ALL hit; enable_deterministic_compute hit.
ADVERSARIAL: ort RC yanked/API change; missing default; trait signature mismatch.

1G.2 ONNX MODEL LOADING, TOKENIZER, GRAPH OPTIMIZATION
PRE_GATE: 1G.1 passes; fixture model+tokenizer present; verify ort Session builder API against rc.12.
TASK:
- Load ONNX via ort session builder with ORT_ENABLE_ALL + enable_deterministic_compute + CPU only.
- Configurable intra-op threads (default 1 for deterministic tests).
- Load tokenizer from tokenizer.json via tokenizers::Tokenizer::from_file.
- Configure truncation max_length=512.
- Model path resolution: quantized_model_path (if Some) -> model_path -> local cache.
- L-6 fallback lazy-loaded on L-12 CB open (5-15s cold start acceptable per L165).
- Return Result; map ort errors to KnowledgeError.
DETERMINISTIC_CHECK: load model returns Ok; tokenizer loads; invalid path Err (not panic);
missing tokenizer Err; encode("hello world") non-empty Vec<u32>; truncates >512 to exactly 512;
ORT_ENABLE_ALL produces identical scores to ORT_DISABLE_ALL (score identity);
enable_deterministic_compute=true + 4 threads identical scores across 5 runs.
ADVERSARIAL: 0-byte file -> Err; API changes; non-deterministic multi-thread; wrong tokenizer vocab;
nonexistent L-6 fallback path -> Err not panic.

1G.3 PAIR SCORING, RERANKING, CONCURRENCY
PRE_GATE: 1G.2 passes.
TASK:
- Tokenize (query, chunk) via encode_batch with pair encoding -> input_ids/attention_mask/token_type_ids.
- Batch inference: one session.run() per batch.
- Extract single logit per pair; re-sort desc.
- Skip when <=skip_threshold (default 10); return unchanged.
- Concurrency: tokio::Semaphore(1) guards session.
- Bounded queue via mpsc channel capacity 4; try_send full -> return unranked immediately (L220).
- Batching: reranker task loop drains channel via try_recv post-semaphore; merges candidates;
  single session.run; splits by caller offsets via oneshot (L221).
DETERMINISTIC_CHECK:
- 20 candidates reranked (order differs); 5 passthrough (<skip); 10 boundary passthrough (<=10);
  11 reranked; skip_threshold=5 with 7 candidates -> rerank; empty returns empty; 1000-token chunk
  valid score (truncation); tensors correct shapes; 5 concurrent => 1 at a time (semaphore);
  6th concurrent when 4 queued -> unranked (channel full); 2 queued batched to 1 call;
  2 concurrent with distinct sets -> no cross-contamination (offset tracking).
ADVERSARIAL: >512 tokens need truncation; NaN/Inf score -> filter or lowest; channel overflow
no infinite block; batching correct splits.

1G.4 CIRCUIT BREAKER, DEGRADATION, SSB EVENTS, MODEL TIERING
PRE_GATE: 1G.3 passes; CircuitBreaker exists from 1C/1E; ClockProvider exists; SSB exists;
spec amendment filed at workflow/reasoning/phase-1G-spec-amendment.md.
TASK:
- Independent CB for L-12; injectable ClockProvider.
- Fallback cascade:
  1. L-12 open + fallback_model_path configured -> lazy-load L-6 (own CB instance).
  2. L-6 also fails or unconfigured -> skip reranking, A-RAG order (L149).
  Note: L-6 tier requires amendment approval; if rejected, remove L-6 path.
- SPARSE_ONLY/BM25_ONLY skip reranker (wired 1G.1).
- SSB event action_type='knowledge_degradation' stage='rerank' on CB state change;
  non-fatal if SSB fails (log warn + continue).
- Tracing DEBUG StageTrace for rerank (L238).
- CB config: failure_threshold=3, window=60s, half_open_timeout=30s (L139-142) via ClockProvider.
DETERMINISTIC_CHECK: 3 failures -> OPEN; L-12 open + no L-6 -> passthrough; mock clock half-open
+ probe -> CLOSED; SSB emitted on state changes; SSB failure non-fatal; StageTrace stage="rerank";
SPARSE_ONLY no session.run; L-12 open + L-6 configured -> L-6 lazy-loaded and used;
both open -> skip; nonexistent L-6 path -> graceful skip not panic.
ADVERSARIAL: shared static CB; real clock slow tests; SSB error propagation; SPARSE_ONLY calls
reranker; L-6 15s cold start caller timeout (document in metrics).

1G.5 STARTUP CANARY UPDATE
PRE_GATE: 1G.4 passes; readiness_check exists; existing canary = embed + BM25.
TASK: update canary to embed + BM25 + rerank (L159); synthetic 3-candidate set; <5s (L160);
reranker failure during canary -> DEGRADED (not UNAVAILABLE, consistent with CB fallback);
log rerank latency.
DETERMINISTIC_CHECK: healthy -> READY + rerank in stages; broken rerank -> DEGRADED (not UNAVAILABLE);
within 5s (mock for speed); rerank grep hit.
ADVERSARIAL: >2s rerank overflows 5s total (load model first then measure rerank-only);
empty BM25 canary output -> reranker empty input no error.

1G.6 OPTIMIZATION SWEEP AND PRECISION VALIDATION
PRE_GATE: 1G.1-1G.5 passes; existing knowledge tests green; fixture committed at
tests/fixtures/reranker_benchmark.json (1 query, 50 candidates, binary relevance, >=10 relevant,
real code chunks not lorem ipsum); INT8 toolchain available (if baseline >30ms):
scripts/quantize_cross_encoder.py committed.
TASK:
A. Baseline (fp32, ORT_ENABLE_ALL, deterministic, 1 thread): rerank 50 candidates warm session,
   p50/p99 10 runs; NDCG@10 with/without on fixture; record as baseline.
B. Thread scaling sweep {1,2,4,8} threads, 10 runs each, p50/p99; determinism at each count
   (5 runs identical ranking order; if scores drift accept ranking-order stability — top-5 same
   set and order); optimal_threads = lowest p50 passing determinism.
C. INT8 conditional if baseline p50 >30ms: quantize script; validate load via ort; re-bench at
   optimal_threads; accept only if NDCG@10 delta <0.5; set quantized_model_path.
D. Skip threshold sweep {5,10,15,20,30}: NDCG@10 at N+1 candidates and at 50; optimal = lowest
   where delta >0.02 over unranked; update default if different than 10.
E. Final: all optimizations applied -> final_latency_p50/p99, final_ndcg_at_10, model_variant
   (fp32/int8); if final p50 >50ms FILE spec amendment with data; record cold-start model load;
   queue wait p99 under 4-agent load (per L224 for future session pool eval).
DETERMINISTIC_CHECK: fixture exists + contains code chunks; baseline/thread_sweep/threshold_sweep/
final_latency_p50 all in phase_1G.toml; int8_* (if attempted); int8 model loads+runs;
NDCG@10 improvement >0.0; quantization script committed if int8 evaluated.
ADVERSARIAL: no relevance signal; warm-up; int8 score drift; thread instability; corrupt quant.

ACCEPTANCE:
- compiles + tests pass; reranking changes order for >skip; skip correct at boundary/config;
  CB opens after 3 failures; L-6 fallback if amendment; final skip per L149; SSB degradation
  non-fatal; queue overflow returns unranked; batching via channel drain + offset;
  ORT_ENABLE_ALL + enable_deterministic; threads optimized + deterministic;
  INT8 evaluated if baseline>30ms, NDCG delta <0.5; canary includes rerank, DEGRADED on failure;
  if p50>50ms spec amendment filed with data; NDCG@10 improvement measurable; StageTrace emitted;
  Semaphore prevents concurrent; ClockProvider (no Instant::now); no unwrap/panic.

EXIT CRITERIA: reranker improves NDCG@10; latency optimized (graph opt + deterministic +
threads + optional int8); skip threshold tuned; CB + graceful degradation (with L-6 if
amendment) + SSB events + canary wired and tested; concurrency-safe; tokenizers produces correct
tensors; injectable clock; all tests pass.

MEASUREMENTS: baseline p50/p99; thread sweep (1/2/4/8); int8 p50/p99 + NDCG delta (if evaluated);
skip threshold sweep (5/10/15/20/30); final p50/p99 all optimizations; model variant;
NDCG@10 delta vs unranked; model load cold start (L-12 + L-6 if used); queue wait p99 under 4-agent;
CB recovery time (mock clock); canary total + per-stage.

=====================================================================
PHASE 1H — SPLADE KEYWORD EXPANSION (conditional)
=====================================================================

Prerequisites: Phase 1G complete.
Status: feature-gated behind cfg(feature="splade").
Goal: evaluate SPLADE sparse expansion; compare prithivida/Splade_PP_en_v1 vs naver/splade-v3;
DROP if <5% recall improvement over BM25-only.

CRITICAL RISK — BERT vocabulary mismatch on code: 30522 WordPiece tokens none programming-specific
(fn, impl, async, Vec<T>, unwrap()). HF research: "vocabulary mismatch destroys representation"
in specialized domains (huggingface.co/blog/yjoonjang/vocabulary-is-the-most-important-element-in-splade).
MUST measure vocab coverage before concluding SPLADE helps on code.

Other risks:
- R24 ~532MB ONNX main vs marginal recall gain.
- R25 SPLADE benchmarks NL (ESCI) not code; recall differs significantly.
- R26 ort ONNX Runtime download strategy; air-gapped.
- R27 SPLADE needs BERT tokenization (input_ids/attention_mask/token_type_ids); ort has no
  tokenization -> need tokenizers crate (maybe present from 1G).
- R28 ort is unconditional dep for GraphCodeBERT/Granite; splade feature gates only the
  SPLADE-specific ort::Session init, NOT ort itself.
- R29 BERT vocab mismatch on programming tokens.
- R30 10-query eval underpowered (TREC 50 minimum, Sakai 2016).
- NOTE: knowledge.txt L58 typo "prithvida" should be "prithivida" — file correction needed.

GROUNDING:
- prithivida/Splade_PP_en_v1 VERIFIED HF, ONNX 532MB main / 436MB int8 quantized.
- ort VERIFIED pyke/ort on crates.io.
- RRF + BM25 GROUNDED L61/L88.
- 5% recall threshold GROUNDED L62.
- SPLADE 29% nDCG@10 on ESCI (Qdrant benchmark).
- SPLADE recall on CODE SEARCH UNGROUNDED (all benchmarks NL).
- fastembed-rs as alt GROUNDED arch L280 EVALUATE.
- naver/splade-v3 VERIFIED HF (arxiv 2403.06789, ensemble distillation + fused losses, outperforms
  PP_en_v1 on TREC/BEIR); ONNX via sentence-transformers optimum.
- BERT vocab on code VERIFIED HF blog.
- 10-query power VERIFIED Sakai 2016, Wilcoxon + Benjamini-Hochberg recommended.
- Quantized ONNX variant VERIFIED onnx-models/Splade_PP_en_v1-onnx 436MB int8, ~3x CPU speedup,
  quality <1% nDCG.

STEPS

1. SKELETON (feature-gated module stub)
PRE_GATE: 1G exit met; workspace check 0; ort resolvable (may already exist for other models).
IMPLEMENT:
- Cargo.toml: [dependencies.ort] version="..." optional=true.
- .elai_cc/src/knowledge/splade.rs entire module behind #[cfg(feature="splade")].
- Stub: `pub struct SpladeExpander`; `pub fn expand_query(&self, query: &str) -> Vec<(String, f32)>`
  returning empty vec.
- Re-export behind #[cfg(feature="splade")] in knowledge/mod.rs.
DETERMINISTIC_CHECK: cargo check (no features) 0; cargo check --features splade 0;
no SpladeExpander symbols without feature (ort is unconditional for GraphCodeBERT/Granite).
ADVERSARIAL: leaky feature gate; ungated pub items.

2. TEST (failing tests, feature-gated)
PRE_GATE: Step 1 DETERMINISTIC_CHECK passes.
IMPLEMENT (tests/knowledge/splade_tests.rs with #[cfg(feature="splade")]):
- expand_query("error handling") returns sparse vec with terms like error/exception/try/catch/handling with non-zero weights.
- expand_query("") empty vec.
- expand_query very long input (>10000 chars) no panic.
- RRF fusion SPLADE+BM25 produces ranking differing from BM25-only.
- Property: non-empty query -> all weights finite and >=0.
- Integration: recall on synthetic corpus BM25 vs BM25+SPLADE.
DETERMINISTIC_CHECK: compile; tests FAIL (red).
ADVERSARIAL: stubs pass trivially (require at least one non-empty assertion); Unicode queries in seed corpus.
ACCEPTANCE: >=5 unit + 1 property + 1 integration.

3. IMPLEMENT — SPLADE inference + RRF fusion
PRE_GATE: Step 2 compiles; tokenizers crate available; ONNX model accessible
(.elai_cc/models/splade/model.onnx or SPLADE_MODEL_PATH env).
IMPLEMENT:
- Add tokenizers crate if missing.
- SpladeExpander::new(model_path): load via ort::Session; load BERT tokenizer (vocab.txt +
  tokenizer.json). Model path configurable for candidate swap.
- expand_query: tokenize (input_ids, attention_mask, token_type_ids); run ONNX; apply SPLADE pooling;
  return top-k (token, weight).
  CRITICAL: ONNX output varies by source:
    PP_en_v1 (prithivida ONNX): outputs MLM logits directly; apply max-pool over positions + ReLU +
      log(1+x).
    splade-v3 (optimum export): outputs RAW MLM logits only; SPLADE pooling NOT in graph; MUST apply
      pooling in Rust. Dispatch via model_variant enum (PrePooled, RawMLM); verify output shape:
      [1, seq_len, 30522] RawMLM vs [1, 30522] PrePooled.
- `vocab_coverage(corpus_tokens: &[String]) -> f64`: fraction mapping to single WordPiece token.
  Fitness metric — coverage <0.3 means SPLADE unlikely to help.
- `rrf_fuse(bm25_results, splade_results, k=60, weight_bm25=0.5, weight_splade=0.5) -> merged`.
  Ranks 1-based. Guard: k=max(k,1).
- Circuit breaker wraps ONNX (knowledge.txt Section 6).
- Fallback on failure: empty expansion (BM25 continues).
- Candidate A prithivida (532MB full / 436MB int8); Candidate B naver/splade-v3. Same interface;
  only model_path differs.
DETERMINISTIC_CHECK: unit tests pass; property finite>=0; non-feature build 0 errors;
vocab_coverage returns valid ratio.
ADVERSARIAL:
- vocab_coverage>0.3 but all common English words — measure on code identifiers specifically
  (extract via tree-sitter, not comments/strings); report code-specific coverage separately.
- corrupt ONNX (1-byte) -> Err not panic.
- non-English (Chinese) -> empty or valid, no panic.
- >5s per query on CPU defeats interactive; assert <2s on 100-char (p50 of 10 excluding warmup).
- RRF k=0 -> clamp min 1; empty results one source OK.
- missing tokenizer files -> Err.
ACCEPTANCE: all unit + property + integration; corrupt returns Err; non-feature clean;
<2s CPU p50; missing tokenizer Err; vocab_coverage returns 0-1; both candidates load + produce output.

4. MEASURE — vocab fitness, model comparison, recall, keep/drop decision
PRE_GATE: Step 3 ACCEPTANCE; recall test infrastructure exists.
IMPLEMENT (execute in order):

4a. VOCABULARY FITNESS CHECK (early-exit gate)
- Extract unique code identifiers via tree-sitter (NOT strings/comments).
- Run SpladeExpander::vocab_coverage on identifiers.
- If code-specific coverage <0.3 (UNGROUNDED conservative heuristic — <30% -> >70% split into 2+
  subwords, each diluting): EARLY DROP. Log reason. Skip 4b-4d.
- If >=0.3 log ratio + proceed.
- Recommendation: if 0.2-0.4 proceed to recall anyway to calibrate data-driven threshold.

4b. MODEL VARIANT SELECTION
- PRE_GATE: pre-stage model files locally (OUTSIDE session — 500MB+). Verify:
  .elai_cc/models/splade/model.onnx and .elai_cc/models/splade-v3/model.onnx. If B unavailable,
  proceed with A only.
- Quantization check (Jaccard valid for same-model different precision): run A-full (532MB) and
  A-quant (436MB int8) on 5 queries; Jaccard top-50 expanded terms. If overlap >=0.95 prefer
  A-quant (saves 96MB + ~3x speed).
- Cross-model A vs B requires actual recall (NOT Jaccard): quick recall@10 on 10 of 50 queries for
  both; higher mean wins; within 1% -> pick faster. Preliminary selection; full eval in 4c.
- Log table (candidate, p50_latency_ms, quantization_jaccard, quick_recall) to
  workflow/reasoning/phase-1H-measurements.md.

4c. RECALL MEASUREMENT (statistically powered)
- Synthetic corpus: 100 files, known-relevant matches for 50 queries (TREC standard minimum;
  Wilcoxon detects ~5% at p<0.05 with ~80% power).
- Query design requirements:
  >=15 vocabulary-mismatch ("error handling" matches try/catch; "concurrent access" matches mutex/rwlock/arc).
  >=15 with distractors (keyword-sharing non-relevant files).
  >=10 straightforward keyword (baseline sanity).
- For winning candidate from 4b:
  BM25-only recall@10 per query;
  BM25+SPLADE RRF recall@10 per query.
- Per-query delta: (splade - bm25)/bm25 * 100 if bm25>0; exclude bm25=0 queries with warning.
- Wilcoxon signed-rank test on paired (bm25, splade) per query; p-value.
- Log: mean delta, std, p-value, per-query breakdown.

4d. KEEP/DROP DECISION
- KEEP: mean delta >=5% AND p<0.05.
- DROP: mean delta <5% OR p>=0.05.
- EDGE: delta>=5% but p>=0.05 -> log "promising but inconclusive," default DROP, preserve module.
- Log rationale to workflow/reasoning/phase-1H-measurements.md.

DETERMINISTIC_CHECK: vocab coverage numeric output; model comparison table logged;
per-query recall; decision documented + Wilcoxon/p_value/EARLY DROP keyword.

ADVERSARIAL:
- corpus too easy -> >=15 queries with distractor.
- all NL-style queries -> >=15 use code syntax/identifiers.
- subset measurement -> assert all 50 run; log errors; no silent drops.
- caching between BM25/SPLADE -> random order or clear caches.
- too few non-zero deltas for Wilcoxon (<20) -> log and default DROP.
- quantized model Jaccard <0.80 vs full -> quality loss exceeds typical <1% nDCG estimate.

ACCEPTANCE: vocab measured on code identifiers; EARLY DROP if <0.3; model table if >=0.3;
50 queries + 100 files; Wilcoxon p-value; decision documented; if DROP flag remains but default-off,
module preserved.

EXIT CRITERIA — three outcomes:
1. EARLY DROP (vocab coverage <0.3).
2. DROP (delta <5% OR p>=0.05).
3. KEEP (delta>=5% AND p<0.05).
All outcomes: feature flag works; module behind cfg gate.
KEEP: CB integrated + RRF operational + winning candidate documented.
DROP: flag disabled by default + decision rationale + measurements logged.
Verify SPARSE_ONLY cascade with splade disabled -> degrades to BM25_ONLY (no dead code path).

MEASUREMENTS:
  vocab_coverage_code (0-1, threshold 0.3)
  model_comparison (candidate, p50_latency_ms, top50_term_jaccard)
  recall_delta_percent per query (excl bm25=0); mean + std across 50
  wilcoxon_p_value (threshold <0.05)
  splade_query_latency_ms p50/p95 (N=10 excl warmup)
  model_load_time_s
  binary_size_delta_mb (with vs without splade feature)

=====================================================================
PHASE 1I — RESILIENCE LAYER
=====================================================================

Prerequisites: Phase 1E complete AND Phase 1G complete (1I.5 adds concurrency around 1G cross-encoder).
BLOCKING NOTE: implementation-guide.txt dep graph L141 shows 1E -> 1I only. MUST add 1G -> 1I edge
before this phase begins.

Goal: circuit breakers, degradation cascade, startup readiness, query validation, cross-encoder
concurrency, pipeline tracing, index versioning.

STEPS

1I.1 CIRCUIT BREAKER
PRE_GATE: cargo test --lib (1G baseline); knowledge compiles; cross-encoder exists (1G);
ClockProvider trait exists.
TASK:
- `EmbeddingCircuitBreaker` struct; states CLOSED/OPEN/HALF_OPEN; independent per ONNX model
  (GraphCodeBERT, Granite, cross-encoder, SPLADE).
- Config: failure_threshold=3, failure_window=60s, half_open_timeout_base=30s.
- All time via ClockProvider (entropy quarantine).
- Failure classification (Azure Architecture Center):
  TRANSIENT (timeout, temporary OOM): count toward threshold (3 needed).
  PERMANENT (segfault, model missing, invalid ONNX): open immediately after 1.
  Classification via error type enum, not string match.
- Exponential backoff for successive OPEN (AWS Builders Library):
  first 30s, second 60s, third 120s, cap 120s.
  Deterministic jitter per model (hash of model_id, not random) to prevent thundering herd
  on simultaneous recovery.
- State NOT checkpointed (resets CLOSED on restart).
- On OPEN: skip + fallback immediately.
- On HALF_OPEN: 2 consecutive successes for CLOSED (Azure pattern: success counter); any failure
  -> OPEN, reset timer, increase backoff.
- Fresh ONNX InferenceSession for HALF_OPEN probes (ORT issues #20043 crash / #20354 hang).
DETERMINISTIC_CHECK: circuit_breaker test passes; struct + enum variants grep-hit.
ADVERSARIAL: non-injectable clock; shared state across models; premature CLOSED on flaky;
simultaneous recovery GPU OOM; permanent-as-transient wastes 2 retries; corrupted session reuse.
ACCEPTANCE: 3 transient opens; 1 permanent opens; 3 at 0/61/122s stay CLOSED; HALF_OPEN needs 2
successes; success-then-failure resets counter; backoff 30/60/120; deterministic jitter same
model_id; different models different jitter; fresh session on probe; per-model independent.
GROUNDING: thresholds GROUNDED L139/L141-142/L151/L135; HALF_OPEN counter GROUNDED Azure;
expo backoff GROUNDED AWS; failure classification GROUNDED Azure; fresh session GROUNDED ORT issues.

1I.2 SEARCHMODE CASCADE AND DEGRADATION
PRE_GATE: 1I.1 passes; BM25 exists; vector exists.
TASK:
- `enum SearchMode { HYBRID, SPARSE_ONLY, BM25_ONLY }`.
- Cascade: any failure drops to next.
- Fallback chain per model:
  GraphCodeBERT fail -> TEXT-vector-only (Granite).
  Granite fail -> CODE-vector-only (GraphCodeBERT).
  Both fail -> SPARSE_ONLY (BM25 + SPLADE).
  SPLADE fail -> BM25_ONLY.
  Cross-encoder fail -> skip reranking; use vector similarity.
- Restore automatically via CB HALF_OPEN probe (30s timeout driven); no separate timer.
- Every mode change -> SSB action_type='knowledge_degradation'.
DETERMINISTIC_CHECK: search_mode / degradation tests pass; enum 3 variants; SSB logging hit.
ADVERSARIAL: flapping model (fail-recover-fail <1s) stabilizes; SSB I/O async; with SPLADE disabled
SPARSE_ONLY==BM25_ONLY (cascade skips).
ACCEPTANCE: kill embed -> BM25 within 100ms; auto-restore HYBRID; SSB has event per change;
never total outage (BM25_ONLY always reachable, tantivy has no ONNX dep).

1I.3 QUERY VALIDATOR
PRE_GATE: knowledge compiles.
TASK: QueryValidator before any embedding call.
- Reject: empty (zero tokens after split), >10,000 chars, null byte or control chars.
- <1ms latency (no model invocation).
DETERMINISTIC_CHECK: query_valid tests pass; struct + length constant (10_000 or MAX_QUERY) hit.
ADVERSARIAL: validation after embed start -> 10MB query OOM; whitespace-only passes wastes compute;
\r\n legitimate must pass.
ACCEPTANCE: empty Err; 10_001 Err; null byte Err; control char Err; "fn main()" Ok; "fn\nmain()" Ok;
<1ms.

1I.4 STARTUP READINESS
PRE_GATE: 1I.1 and 1I.2 exist.
TASK:
- Sequence:
  1. Load ONNX models (GraphCodeBERT, Granite, cross-encoder, optional SPLADE).
  2. Open SQLite DBs (DKG graph, vector cache, BM25).
  3. Canary: embed("fn main()") + BM25 search("main") + rerank.
  4. All within 5s.
  5. CANARY DRIFT CHECK: cosine_sim(embed("fn main()"), reference) < 0.85 -> DEGRADED
     (model weights corrupted or wrong version loaded). Reference generated on first successful
     startup, persisted in index_version.toml as canary_reference_vec.
  6. Report ReadinessStatus to orchestrator.
- `enum ReadinessStatus { READY, DEGRADED, UNAVAILABLE }`.
  READY all layers ok.
  DEGRADED ONNX fail but BM25 ok -> BM25_ONLY.
  UNAVAILABLE BM25 also fail -> total outage.
  Invariant: DEGRADED implies BM25_ONLY (not HYBRID or SPARSE_ONLY).
- 3 retries then DEGRADED + WARN.
- BM25 fail -> UNAVAILABLE + ERROR; orchestrator blocks FSM.
- Orchestrator gates FSM: IDLE -> PLANNING blocked until READY or DEGRADED; UNAVAILABLE blocks all.
DETERMINISTIC_CHECK: startup_readiness + canary_query tests pass; enum 3 variants; fn impl hit.
ADVERSARIAL: hanging canary (5s timeout -> DEGRADED); garbage embeddings (swap random-weight ONNX
-> drift <0.85 -> DEGRADED); missing reference (first startup -> skip + persist after); 3 retries
serial without backoff (total startup <20s).
ACCEPTANCE: healthy READY <5s; all ONNX fail -> DEGRADED after 3 retries BM25_ONLY; BM25 fail ->
UNAVAILABLE; UNAVAILABLE blocks IDLE->PLANNING; canary timeout -> DEGRADED within 5s (no hang);
drift cos<0.85 -> DEGRADED; first startup drift skipped + persisted; DEGRADED == BM25_ONLY;
enum exactly 3 variants.
GROUNDING: canary components GROUNDED L159; 5s GROUNDED L160; 3 retries GROUNDED L164;
enum variants GROUNDED arch L99; 5-15s GROUNDED L165; drift check GROUNDED embedding audit blog.

1I.5 CROSS-ENCODER SEMAPHORE
PRE_GATE: cross-encoder exists + compiles (1G); rerank tests pass.
TASK:
- tokio::Semaphore(1) around cross-encoder ONNX.
- Bounded channel capacity 4; max 5 in-flight (1 active + 4 queued).
- 6th overflows -> unranked immediately.
- Batching: multiple queued -> merge + rerank once.
DETERMINISTIC_CHECK: semaphore + rerank_queue tests pass; Semaphore(1) hit.
ADVERSARIAL: missing semaphore -> session corruption on concurrent; overflow silently empty
(must return input-order unranked); merged batch wrong caller order (offset tracking).
ACCEPTANCE: 2 concurrent serialized; 6th overflows returns unranked (not empty); result.len ==
input.len on overflow; batching 2x10 -> 1 call with 20; per-caller segregation.
GROUNDING: Semaphore(1) L219; depth 4 L220; overflow unranked L220; merge L221; <50ms L222.

1I.6 QUERY TRACE
PRE_GATE: 1I.1-1I.5 compile; tracing crate present.
TASK:
- `QueryTrace { query_id: UUID, stages: Vec<StageTrace>, total_latency_ms, mode: SearchMode,
                result_count }`.
- `StageTrace { stage: String, latency_ms, input_count, output_count, error: Option<String>,
                circuit_breaker_state: Option<String> }`.
- Stages (9): "bm25", "splade", "code_vector", "text_vector", "pagerank", "catrag", "arag",
  "rerank", "token_fit".
- Emit via tracing at DEBUG.
- Aggregated per-session stats (p50/p99 per stage) stored in SSB at session end.
- NOT per-query in SSB (too verbose).
DETERMINISTIC_CHECK: query_trace tests pass; QueryTrace + StageTrace + tracing::debug|event hit.
ADVERSARIAL: constructed-but-not-emitted dead code; wall-clock non-determinism (assert populated,
not exact); unpopulated CB state -> blind debugging.
ACCEPTANCE: every query emits one QueryTrace; HYBRID trace has bm25/code_vector/text_vector/rerank
minimum; BM25_ONLY trace has only "bm25"; p50/p99 aggregation correct.
GROUNDING: QueryTrace/StageTrace GROUNDED L230-236/L237-244; DEBUG + session-end SSB L246-247;
9 stages L238.

1I.7 INDEX VERSION SCHEMA
PRE_GATE: knowledge compiles; .elai_cc/projects/ convention.
TASK:
- Read/write index_version.toml at .elai_cc/projects/{id}/index_version.toml.
- Schema: [index] version, dkg_schema_version, vector_dimensions (code=768, text=384), bm25_format,
  created_at.
- On upgrade: compare versions, trigger full rebuild OR BM25-only rebuild OR reuse.
- Archive old index at index_backup_{version}/ BEFORE rebuild.
- User notified (not silent).
DETERMINISTIC_CHECK: index_version test passes; schema handling grep; round-trip write+read test.
ADVERSARIAL: missing file (first run) -> create default v1 no panic; corrupt TOML -> full rebuild
+ warning; archive fails (disk full) -> abort rebuild not destroy old.
ACCEPTANCE: round-trip all fields; missing -> default v1; version mismatch -> rebuild notification;
archive exists after upgrade.
GROUNDING: schema L195-201; rebuild rules L203-207; archive L208; notify L211.

VERIFY:
- cargo check -p elai_knowledge with all new types.
- cargo test --lib no 1G regressions.
- Project isolation under degradation: BM25_ONLY query with project_id=A never returns B results
  (owned by 1A/1B/1C; 1I verifies isolation under degraded SearchMode).
- Vector cache invalidation under degradation: file_write invalidates entries even in SPARSE_ONLY/
  BM25_ONLY (no stale embeddings on recovery).
Note: project isolation invariants (L274-279) owned by 1A (DKG), 1B (BM25), 1C (vectors);
this phase verifies they hold under degradation.

MEASURE:
- degradation latency (failure injection -> SearchMode switch, <100ms target)
- canary latency healthy (<5s)
- CB recovery OPEN -> HALF_OPEN -> CLOSED (<35s typical)
- cross-encoder semaphore p99 queue wait under load
- QueryTrace overhead (<1ms per query)
- index version check latency (<10ms)

EXIT CRITERIA:
- Integration test: full pipeline on elai_cc codebase returns results.
- Pipeline survives ONNX failure without total outage.
- CB transitions through 3 states.
- Cascade reaches BM25_ONLY and auto-recovers.
- Startup readiness gates FSM until READY/DEGRADED.
- Semaphore prevents concurrent ONNX corruption.
- QueryTrace emitted every query at DEBUG.
- Index version migration preserves old as backup.

PHASE 1 OVERALL MEASUREMENTS:
Index build time, query latency per layer, token savings, dual-vector vs single-vector accuracy,
reranker precision gain, SPLADE recall delta, degradation cascade correctness, startup readiness latency.
