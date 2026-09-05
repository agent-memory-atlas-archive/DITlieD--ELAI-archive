Phase 1H Grounding Results
Session: s-913064
Date: 2026-04-06

SPEC VERIFICATION
  docs/specs/crates/knowledge.txt SPLADE section (L57-62)    | EXISTS | Non-empty, contains model name, size, fusion details
  docs/specs/crates/knowledge.txt query pipeline (L88)        | EXISTS | Shows SPLADE position in pipeline
  docs/specs/architecture-overview.txt Section 10 deps (L272) | EXISTS | Confirms ort dependency
  docs/specs/crates/knowledge.txt circuit breaker (L133-151)  | EXISTS | SPLADE-specific fallback at L148
  docs/specs/crates/knowledge.txt startup readiness (L157)    | EXISTS | SPLADE optional load confirmed

CLAIM GROUNDING
  "prithvida/Splade_PP_en_v1"              | VERIFIED  | HuggingFace: correct name is "prithivida" (not "prithvida"). Model exists. ONNX = 532MB (main), 436MB (onnx-models variant). TYPO in knowledge.txt noted (missing 'i' in prithivida).
  "~500MB ONNX weight"                     | VERIFIED  | Actual: 532MB main model. Approximately correct. Corrected to 532MB in phase file.
  "ort crate for ONNX Runtime"             | VERIFIED  | pyke/ort on crates.io (github.com/pykeio/ort). Active. Wraps Microsoft ONNX Runtime. Download strategy configurable.
  "RRF fusion with BM25"                   | GROUNDED  | knowledge.txt L61 and L88 confirm weighted RRF before PageRank. Standard IR technique.
  "5% recall improvement threshold"        | GROUNDED  | knowledge.txt L62: "Drop if <5% gain over BM25 alone." Design decision, not empirical claim.
  "SPLADE improves recall over BM25"       | VERIFIED  | Qdrant benchmarks: 29% nDCG@10 improvement on Amazon ESCI dataset. Competitive with dense retrievers on TREC DL.
  "error handling matches try/catch"       | GROUNDED  | knowledge.txt L60 states vocabulary expansion catches semantic synonyms.
  "fastembed-rs alternative to ort"        | GROUNDED  | architecture-overview.txt L280 marks as EVALUATE (not confirmed dependency).
  SPLADE recall on CODE SEARCH specifically | UNGROUNDED | All published benchmarks are NL retrieval (ESCI, TREC DL, MS MARCO). No code search benchmarks found. Assumed useful because: vocabulary expansion should help with programming synonyms, but magnitude is unknown. This is exactly why the phase measures and applies a keep/drop gate.

SUBSTANTIVE IMPROVEMENTS (evidence-backed, s-913064)
  1. Added SPLADE-v3 (naver/splade-v3) as comparison candidate.
     Evidence: HuggingFace model exists. arxiv 2403.06789 shows it outperforms SPLADE++ on TREC DL and BEIR via ensemble distillation + fused losses. Provably beneficial: better model = higher chance of passing keep/drop gate.
  2. Added BERT vocabulary coverage pre-check with 0.3 early-drop threshold.
     Evidence: HuggingFace blog "Vocabulary is the most important element of Sparse Retrieval" (huggingface.co/blog/yjoonjang/) demonstrates vocab mismatch destroys SPLADE representations in specialized domains. BERT WordPiece (30522 tokens) was trained on Wikipedia/BookCorpus, not code. Provably beneficial: avoids wasting an entire recall measurement cycle on a fundamentally broken tokenizer.
  3. Increased evaluation from 10 queries / 50 files to 50 queries / 100 files.
     Evidence: Sakai 2016 SIGIR/TOIS systematic review on statistical power shows 10 queries cannot detect effect sizes below ~15%. TREC standard is 50 topics. With 50 queries + Wilcoxon signed-rank, ~5% differences detectable at p<0.05 with ~80% power. Provably beneficial: prevents false DROP decisions on real but small improvements.
  4. Added quantized ONNX variant (436MB int8) as evaluation option.
     Evidence: onnx-models/Splade_PP_en_v1-onnx exists at 436MB. sentence-transformers docs report ~3x CPU inference speedup for int8 quantization with typically <1% nDCG loss. Provably beneficial: if quality loss < 1%, saves 96MB disk + 3x speed for free.
  5. Added Wilcoxon signed-rank statistical test to keep/drop decision.
     Evidence: Sakai 2016 + Urbano et al. confirm Wilcoxon + Benjamini-Hochberg yields correct Type I error rates and best power for typical IR sample sizes. Provably beneficial: prevents acting on noise. Delta >= 5% but p >= 0.05 correctly classified as inconclusive instead of falsely kept.

ADVERSARIAL AUDIT (post-improvement, s-913064)
  4 BLOCKs found and fixed:
    1. Missing tokenizers crate in Step 3 — SPLADE needs BERT tokenization, ort does not provide it. FIXED: added PRE_GATE check and IMPLEMENT instruction.
    2. Step 1 false invariant: tested "no ort without feature" but ort is unconditional for GraphCodeBERT/Granite. FIXED: changed to test for SpladeExpander symbol absence instead.
    3. Recall delta division-by-zero when BM25 recall = 0. FIXED: added per-query guard excluding zero-recall queries with warning log.
    4. RRF rank basis unspecified, k=0 risk. FIXED: specified 1-based ranks and k = max(k, 1) guard in IMPLEMENT.
  5 WARNs found and fixed:
    1. knowledge.txt typo "prithvida" -> noted for correction (cannot edit locked file).
    2. Distractor enforcement missing deterministic check. FIXED: added CHECK assertion.
    3. SPARSE_ONLY cascade validation missing from EXIT CRITERIA. FIXED: added verification step.
    4. SPLADE_MODEL_PATH env var had no default. FIXED: added default path .elai_cc/models/splade/.
    5. Latency measurement lacked warm-up exclusion. FIXED: specified p50 of N=10, excluding first call.

ADVERSARIAL AUDIT ROUND 2 (post-substantive-improvements, s-913064)
  2 BLOCKs found and fixed:
    1. splade-v3 ONNX export only outputs raw MLM logits — ReLU+log1p pooling NOT in the ONNX graph. FIXED: added model_variant enum (PrePooled vs RawMLM) and explicit pooling step in expand_query() documentation. Verified via sentence-transformers ONNX export docs.
    2. 0.3 vocab coverage threshold was presented as evidence-backed but is ungrounded heuristic. FIXED: labeled as UNGROUNDED with reasoning (>70% subword splits = fragment-level operation), added soft zone guidance (0.2-0.4 = proceed anyway and calibrate).
  4 WARNs found and fixed:
    1. Wilcoxon non-zero pairs floor too low (15). FIXED: raised to 20 per standard power tables (~80% power at p<0.05 for r~0.3).
    2. Jaccard used as cross-model recall proxy (invalid). FIXED: restricted Jaccard to quantization regression only. Cross-model comparison uses actual recall@10 on 10 queries.
    3. splade-v3 ONNX export has no PRE_GATE. FIXED: added file existence check with graceful fallback to A-only if B unavailable.
    4. Model files may need in-session download (scope creep). FIXED: added explicit note that model staging is a prerequisite outside the session, not in-session work.

EXTERNAL SOURCES
  HuggingFace model: https://huggingface.co/prithivida/Splade_PP_en_v1
  ort crate: https://crates.io/crates/ort (docs: https://ort.pyke.io/)
  SPLADE benchmarks: https://qdrant.tech/articles/sparse-embeddings-ecommerce-part-1/
  SPLADE paper efficiency: https://arxiv.org/html/2511.22263v1
  SPLADE-v3 paper: https://arxiv.org/abs/2403.06789
  SPLADE-v3 model: https://huggingface.co/naver/splade-v3
  Vocab mismatch blog: https://huggingface.co/blog/yjoonjang/vocabulary-is-the-most-important-element-in-splade
  Sentence-transformers ONNX export: https://sbert.net/docs/sparse_encoder/usage/efficiency.html
  IR statistical power (Sakai 2016): https://www.semanticscholar.org/paper/Statistical-Significance-Power-and-Sample-Sizes-Sakai/5a729680c3f2c9ee40b4f5f3a20c24b768c0a1c9
  Wilcoxon significance testing in IR: https://julian-urbano.info/files/publications/076-statistical-significance-testing-information-retrieval-empirical-analysis-type-i-type-ii-type-iii-errors.pdf
