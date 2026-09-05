# E-17: RAG retrieval quality — stage ONNX, contextual ingest, modern reranker, DEEP fix

<!--
COMPLETION-DRIVE STATUS (s-25120 / 2026-05-23 rollup)

SHIPPED-ON-V2 (15 surfaces):
  SP1 doctor probe `elai doctor knowledge-models` (cli/src/doctor.rs:725)
  SP2 DEEP cap + RRF floor + named consts (crates/knowledge/src/deep.rs)
  SP2 HyDE NL-only gate + ELAI_HYDE_NL_ONLY env (crates/knowledge/src/hyde.rs)
  SP2 SsbActionType::DeepPoolCapped (ssb.rs:2315 + as_sql_str arm)
  SP2 SsbActionType::RrfQualityGateFired (ssb.rs:2319 + as_sql_str arm)
  SP2 SsbActionType::HydeSkippedCodeQuery (ssb.rs:2323 + as_sql_str arm)
  SP2 SsbActionType::RrfComponentWeights (ssb.rs:2327 + as_sql_str arm)
  SP2 ReRankerModel enum + with_project_root_and_variant (crates/knowledge/src/reranker.rs)
  SP2 code_vec wired into 3-ranker RRF (crates/knowledge/src/pipeline.rs:506-553)
  SP2 [knowledge.reranker]/[knowledge.deep]/[knowledge.rrf]/[knowledge.code_vec]/[knowledge.hyde] config schema (elai_cc.toml:811-851)
  SP4 Tauri commands: get_deep_pool_capped_recent / get_rrf_quality_gate_recent / get_hyde_skipped_code_query_recent / get_rrf_component_weights_recent (dashboard-v2/src-tauri/src/commands.rs:1237-1386)
  SP4 KnowledgeDrawer.svelte panels: DEEP POOL CAPPED / RRF QUALITY GATE / HYDE SKIPPED / RRF COMPONENT WEIGHTS (dashboard-v2/src/lib/drawers/KnowledgeDrawer.svelte:357-502)
  SP4 invoke_handler! registration (dashboard-v2/src-tauri/src/lib.rs:301-304)
  SP6 .elai_cc/eval/rag/ harness (queries.toml + run.py + README.md)
  SP6 tests: deep_pool_cap_contract.rs / hyde_code_gate_contract.rs / contracts::rag_monotonicity_contract.rs

SHIPPED-THIS-SESSION (5 surfaces):
  M-A SP4 contextual_retrieval module — heuristic ground-floor + LLM template scaffold
     (crates/knowledge/src/contextual_retrieval.rs, ~480 LOC incl. 13 tests)
  M-B SP5 cAST chunker — recursive-split + sibling-merge over tree-sitter
     (crates/knowledge/src/chunking/cast.rs, ~510 LOC incl. 10 tests, 4 lang adapters)
  M-C SP7 [knowledge.late_interaction] enabled = false flag (elai_cc.toml)
  3 missing tests: contextual_chunk_emission_contract.rs (6/6) + jina_reranker_load_and_score.rs (5/5)
     + cast_chunker_recall_smoke.rs (10/10) — all green
  Doctor probe optional-rerankers advisory footer (cli/src/doctor.rs)
     + stage_onnx_models.py bge-reranker-v2-m3 recipe (anti-slopsquatting verified)

JINA-MODEL-PENDING (D-1 drift):
  ReRankerModel::default() stays MsMarcoMiniLmL12. Flipping prematurely silently
  degrades every install (load() returns Err -> unranked). jina-reranker-v3 has
  NO official ONNX as of 2026-05-23 (WebSearch verified: PyTorch/MLX/GGUF only).
  bge-reranker-v2-m3 ONNX (onnx-community) is opt-in stageable via
  `python scripts/stage_onnx_models.py --only bge-reranker-v2-m3`.
  See workflow/reasoning/E-17-decisions.md SECTION 2.

SCHEMA-DEFERRED (SCHEMA_VERSION 103 LOCKED):
  SsbActionType::ContextualChunkEmitted, SsbActionType::ChunkRegenSkipped
  Producer code (contextual_retrieval.rs) is READY; variants defer to next batch.
  See workflow/reasoning/E-17-decisions.md SECTION 1.

SPEC_DRIFT-RESOLVED (3 grounding items):
  1. Default flip — resolved as JINA-MODEL-PENDING above.
  2. Missing cAST/contextual modules — both shipped this session.
  3. Missing [knowledge.late_interaction] flag — schema doc added (producer DEFERRED).
-->

Objective: ship the four highest-leverage RAG fixes that the improvement-sweep flagged as ELAI's #1 cross-cutting priority: (1) stage the dual-vector ONNX models on disk so Layer 3 stops silently degrading to BM25-only, (2) add Anthropic Contextual Retrieval at ingest, (3) replace ms-marco-MiniLM-L12 reranker with Jina-rerank-v3 or BGE-reranker-v2-m3, (4) cap DEEP candidate pool + add an RRF quality gate so DEEP MRR stops collapsing under STANDARD MRR. The "stage ONNX models" claim is the highest-leverage fix in the entire sweep — verify and ship FIRST before any new technique is added.
Estimated effort: 3 sessions, ~12-16 h (largest E-N in this wave)
Pre-requisites: B-1 SHIPPED (path unification), B-4 SHIPPED via code_vec ELAI_CODE_VEC=1 opt-in, C-1 (SSB action types for query trace).
Parallel-safe with: E-16, E-18, C-8 (Viewer Stub Eradication).
Source report: workflow/improvement-sweep-2026-05-19/reports/17-rag-retrieval-quality.md

## Context

ELAI's measured DEEP MRR 0.206 vs STANDARD MRR 0.575 violates spec monotonicity (DEEP should be >= STANDARD). The sweep traces this to three concurrent failure modes documented in the literature: (a) reranker dilution beyond candidate-quality ceiling — Drowning in Documents (arxiv:2411.11767) shows "testing top-250 candidates found almost no improvement over top-100, meaning the retriever exhausts useful candidates well before rank 250" and "an improved candidate list sometimes causes inferior reranking"; (b) ONNX dual-vector models NOT staged on disk so Layer 3 silently degrades to BM25-only at runtime, while code_vec is populated but never queried; (c) HyDE introduces hallucinated-figure noise on precise code-symbol queries (finance-RAG benchmark cited recall@5 0.544 HyDE vs 0.587 vanilla; CodeRAG-Bench arxiv:2406.14497 explicitly flags low-lexical-overlap weakness).

The highest-leverage single fix is Anthropic Contextual Retrieval at ingest (anthropic.com/news/contextual-retrieval, Sep 2024): -49% top-20 failure on Anthropic's own internal codebase eval, -67% with rerank stacked. Cost is paid once at ingest (cache via prompt caching), not per query, so it dominates HyDE on cost-quality Pareto. Replacing 2020-era ms-marco-MiniLM-L12 with Jina-rerank-v3 (61.94 NDCG@10 BEIR, +4.88% vs v2) is the next-cheapest win. PLAID/FastPlaid late-interaction is a longer-tail addition; it's deferred to E-17.B as an optional 8th retriever once tasks 1-4 land.

Cited grounding: anthropic.com/news/contextual-retrieval, arxiv:2411.11767 (Drowning in Documents), arxiv:2406.14497 (CodeRAG-Bench), arxiv:2510.20609 (Practical Code RAG at Scale; PL-PL BM25 word-level beats dense at 10x lower latency), arxiv:2506.15655 (cAST AST-aware chunking, +4.3 Recall@5), jina.ai/models/jina-reranker-v3, arxiv:2212.10496 (HyDE original), qdrant.tech/articles/bm42 (BM42 contested; do NOT adopt).

## Existing state (B/C/D coverage)

- B-1 (DONE s-59964): path unification — DKG/BM25/vectors paths now go through `core::paths::*`. Pre-req satisfied.
- B-4 (DONE; opt-in ELAI_CODE_VEC=1): producer wiring for knowledge layer; code_vec populated but never queried per sweep audit. EXTENSION needed — fix the query path so code_vec actually participates in RRF.
- B-4 staging spec for 3 ONNX models (cross-encoder-l12, GraphCodeBERT, Granite) is in the B-4 plan; sweep confirms they are NOT staged on disk at runtime → degraded retrieval. REPLACEMENT — verify staging, fix if absent.
- C-8 Viewer Stub Eradication (PLANNED) covers 7 viewer stubs in ssb_viewer.rs incl. `freshness_state`, `query_tier_stats`, `recent_query_traces`. E-17 produces the underlying rows C-8 will surface.
- Phase 3K rag-hardening (per sweep) targets HyDE / dual-vector hardening; E-17 supersedes Layer A HyDE (drop for code queries) and Layer 7 reranker (upgrade).
- D-11 checker registry (PLANNED) consumes API/code checkers but does NOT touch retrieval; orthogonal.

## Reads (mandatory before coding)

- workflow/improvement-sweep-2026-05-19/reports/17-rag-retrieval-quality.md (full)
- F:\.ELAI_workflow\.elai_cc\crates\knowledge\src\reranker.rs (existing MiniLM-L12 ONNX path; lines 551-626 the inference site, lines 1059-1208 the staging-fixture tests, line 392 the degradation-emit site)
- F:\.ELAI_workflow\.elai_cc\crates\knowledge\src\pipeline.rs (BM25 + rerank stage wiring; lines 949-977 canary tests confirm bm25_ok / rerank_ok plumbing)
- F:\.ELAI_workflow\.elai_cc\crates\knowledge\src\degradation.rs (search-mode cascade HYBRID → CodeOnly/TextOnly → SparseOnly → Bm25Only)
- F:\.ELAI_workflow\.elai_cc\crates\knowledge\src\query_trace.rs (mode/stage emission for SSB)
- F:\.ELAI_workflow\.elai_cc\crates\knowledge\src\hyde.rs (if present; B-4 noted hyde_query SSB type)
- F:\.ELAI_workflow\.elai_cc\crates\knowledge\src\chunking\ (existing chunking; cAST replacement target)
- anthropic.com/news/contextual-retrieval (the cookbook + system-prompt spec)
- arxiv:2411.11767 PDF + arxiv:2406.14497 PDF + arxiv:2506.15655 PDF (cite in tests)

## Tasks

1. VERIFY-FIRST PROBE: before writing any new code, write a runtime probe that loads the knowledge engine and reports actual on-disk file existence for the three ONNX models (cross-encoder-l12, GraphCodeBERT, Granite) under `core::paths::models_dir_path`. If any are missing, stage them; if present, log file size + tokenizer hash. The probe is a CLI subcommand `elai doctor knowledge-models`. This is the highest-ROI single check in the whole sweep — confirm BEFORE adding new techniques.

2. Add Anthropic Contextual Retrieval to the DKG ingest pipeline. At chunk emission, call the local sidecar with a 50-100-token "this chunk is from file X, in class Y, doing Z" generation. Use prompt caching at the sidecar layer to amortise the per-file system prompt. Prepend the generated context to the chunk body BEFORE BM25 indexing and BEFORE dense vector embedding. Cache the contextualized chunk in the existing chunk store; do NOT re-generate on re-index unless source-file hash changed. Track per-chunk Blake3 content hash to detect regen need.

3. Replace ms-marco-MiniLM-L12 with Jina-rerank-v3 (preferred) or BGE-reranker-v2-m3 (fallback). Both are ONNX-friendly so the existing `reranker.rs` ORT session pipeline is reusable. Stage the model at `core::paths::models_dir_path().join("jina-reranker-v3")` with tokenizer + ONNX file pair. Add a config key `[knowledge.reranker] model = "jina-rerank-v3" | "bge-reranker-v2-m3" | "ms-marco-minilm-l12"` so the swap is testable per-task. Default to jina-rerank-v3 once staging confirmed.

4. Fix DEEP MRR regression. Two-part fix: (a) hard-cap DEEP candidate pool to 100 (per arxiv:2411.11767 saturation finding); (b) add RRF-score quality gate — drop candidates below `RRF_FLOOR = 0.005` (k=60 baseline) before reranker input. Surface both gates as config keys `[knowledge.deep] max_candidates = 100, rrf_floor = 0.005`. Emit `deep_pool_capped` and `rrf_quality_gate_fired` SSB action types per query.

5. Drop HyDE from the code-query path. Keep HyDE as opt-in for natural-language queries only (`ELAI_HYDE_NL_ONLY=1` default true). At Layer A query-type classifier, route NL → HyDE-enabled, PL/code-symbol → HyDE-disabled. Cite arxiv:2212.10496 + finance-RAG recall@5 numbers in the decision doc. Emit `hyde_skipped_code_query` SSB row when the gate fires.

6. Replace ELAI's chunking with cAST (arxiv:2506.15655) recursive-split + sibling-merge algorithm. The tree-sitter parser stays; only the chunk-emission heuristic changes. Measure +4.3 Recall@5 / +2.67 Pass@1 deltas on the private ELAI 135-file eval (per task 10).

7. Wire code_vec into the RRF stage. The B-4 audit found code_vec populated but never queried; verify the 3-ranker RRF (BM25 + code_vec + text_vec) actually invokes all three retrievers and that RRF weights are tunable per config `[knowledge.rrf] weights = {bm25 = 1.0, code_vec = 1.0, text_vec = 0.8}`. Emit `rrf_component_weights` per query to surface in C-8 viewer.

8. Add PLAID/FastPlaid late-interaction as an optional 8th retriever (deferred default-off). next-plaid Rust crate (CPU) is the lowest-friction integration; FastPlaid Rust+CUDA is the production target. Hide behind `[knowledge.late_interaction] enabled = false` for E-17.B follow-up; the wiring lives behind the flag so a future ablation can flip it on.

9. Surface the new SSB types and config in the CloudPanel/KnowledgeDrawer (V2 dashboard). Per-query: `rrf_component_weights`, `deep_pool_capped`, `rrf_quality_gate_fired`, `hyde_skipped_code_query`, `contextual_chunk_emitted`, `chunk_regen_skipped`. C-8 owns the Tauri commands; E-17 emits the rows.

10. Build a private ELAI-specific eval harness at `.elai_cc/eval/rag/` with the 135-file DKG corpus + 50-query goldset. Report R@5, R@10, MRR per tier (STANDARD, DEEP) before/after each task. Targets per report §Open questions: STANDARD MRR ≥ 0.55, DEEP MRR > STANDARD MRR (monotonicity), HyDE-NL recall@5 ≥ baseline, cAST Recall@5 ≥ +4 points.

## WIRING_CONTRACT

produces_symbols:
  - knowledge::contextual_retrieval::contextualize_chunk (new)
  - knowledge::chunking::cast::CastChunker (new)
  - knowledge::reranker::ReRankerModel (extended enum: JinaV3, BgeV2M3, MsMarcoMiniLmL12-legacy)
  - knowledge::deep::deep_pool_cap (new fn)
  - knowledge::deep::rrf_quality_gate (new fn)
  - knowledge::hyde::should_use_hyde (new gate)
  - cli::doctor::knowledge_models_probe (new subcommand)
  - SsbActionType::ContextualChunkEmitted (new)
  - SsbActionType::ChunkRegenSkipped (new)
  - SsbActionType::DeepPoolCapped (new)
  - SsbActionType::RrfQualityGateFired (new)
  - SsbActionType::HydeSkippedCodeQuery (new)
  - SsbActionType::RrfComponentWeights (new)
consumes_symbols:
  - knowledge::pipeline::run_query (existing entry point; new stages slot in)
  - knowledge::reranker::rerank_with_breaker (existing; model loader swap)
  - knowledge::degradation::SearchMode (existing cascade)
  - knowledge::query_trace::QueryTrace::stages (new stage labels: contextual, deep_cap, rrf_gate)
  - core::paths::models_dir_path (B-1 helper)
reachable_from:
  - crates/cli/src/main.rs::main (via `elai doctor knowledge-models`)
  - crates/orchestrator/src/task_runner.rs::knowledge_query
  - crates/dashboard-v2/src-tauri/src/lib.rs::get_query_trace_recent (after C-8)
stub_ban:
  - no ms-marco-MiniLM-L12 as DEFAULT reranker after task 3 (kept as legacy enum variant only)
  - no HyDE invocation on code-symbol queries (gate enforced)
  - no DEEP pool > 100 candidates without explicit `[knowledge.deep] max_candidates` override
  - no chunk emission without contextual prefix (after task 2 ships, unless source-file hash chain unbroken)
  - zero `.unwrap()` in production knowledge paths (CLAUDE.md)
frontend_pair: crates/dashboard-v2/src/lib/components/KnowledgeDrawer.svelte
observable_effect:
  trigger: "elai /research --mode cloud 'find every caller of RuntimeAdapter::stream_step in the codebase'"
  expect_ssb: "rrf_component_weights with non-null bm25 + code_vec + text_vec; deep_pool_capped with capped_count<=100; hyde_skipped_code_query for the PL query class"
  expect_artifact: "DEEP MRR >= STANDARD MRR on the eval harness 50-query run; query_trace shows 4-stage pipeline (bm25 + code_vector + text_vector + rerank) all populated"

## DETERMINISTIC_CHECK

```bash
# 1. ONNX staging probe runs and exits 0 on a healthy install
elai doctor knowledge-models
# Expected: exit 0; reports cross-encoder/graphcodebert/granite all present with non-zero sizes and tokenizer hash.

# 2. Contextual ingest is wired
rg "fn contextualize_chunk|ContextualChunkEmitted" .elai_cc/crates/knowledge --type rust
# Expected: ≥3 hits (impl + dispatch + SSB variant).

# 3. Jina-rerank-v3 (or BGE-v2-m3) is the default reranker
rg "ReRankerModel::JinaV3|reranker.*default.*jina" .elai_cc/crates/knowledge --type rust
# Expected: ≥1 default-binding hit.

# 4. DEEP pool cap + RRF floor present
rg "MAX_DEEP_CANDIDATES|RRF_FLOOR|rrf_quality_gate" .elai_cc/crates/knowledge --type rust
# Expected: const MAX_DEEP_CANDIDATES = 100 + const RRF_FLOOR = 0.005.

# 5. HyDE gate for code queries
rg "hyde_skipped_code_query|should_use_hyde" .elai_cc/crates/knowledge --type rust
# Expected: ≥2 hits.

# 6. cAST chunker present
rg "CastChunker|chunking::cast" .elai_cc/crates/knowledge --type rust
# Expected: ≥3 hits.

# 7. Eval harness exists
ls .elai_cc/eval/rag/queries.toml .elai_cc/eval/rag/run.py
# Expected: both files exist.

# 8. Eval-harness run produces required deltas
python .elai_cc/eval/rag/run.py --tier standard --tier deep --report mrr,r@5,r@10
# Expected: STANDARD MRR >= 0.55, DEEP MRR > STANDARD MRR (monotonicity).
```

## ACCEPTANCE

- [ ] `elai doctor knowledge-models` reports all three ONNX models staged with non-zero file sizes
- [ ] Contextual Retrieval prefix present on every newly indexed chunk; re-index of unchanged source skips regen (Blake3 hash check)
- [ ] Jina-rerank-v3 (or BGE-v2-m3) is the DEFAULT reranker model; ms-marco-MiniLM-L12 remains accessible as legacy
- [ ] DEEP candidate pool capped at 100; RRF floor at 0.005; both surfaced as SSB rows per query
- [ ] HyDE disabled for code-symbol/PL queries by default; opt-in only for NL queries
- [ ] cAST chunking algorithm replacing legacy line-based chunking; tree-sitter parser unchanged
- [ ] code_vec actually queried in 3-ranker RRF; weights configurable; SSB row per query confirms participation
- [ ] STANDARD MRR ≥ 0.55, DEEP MRR > STANDARD MRR on the 50-query eval (monotonicity restored)
- [ ] cAST Recall@5 ≥ +4 points on private eval vs legacy chunker (per arxiv:2506.15655 paper number; conservative target ≥ +2)
- [ ] PLAID/FastPlaid wiring exists behind `[knowledge.late_interaction] enabled = false` flag (deferred default-off)
- [ ] CLAUDE.md anti-stale rule: blake3 for chunk-hash (NEVER blake2/sha2 per token-discipline)

## Tests to add

- `.elai_cc/crates/knowledge/tests/contextual_chunk_emission_contract.rs`: round-trip a synthetic file, assert prefix presence, assert hash-stable regen skip.
- `.elai_cc/crates/knowledge/tests/deep_pool_cap_contract.rs`: synthesize 250-candidate pool, assert cap to 100, assert RRF floor drop count.
- `.elai_cc/crates/knowledge/tests/hyde_code_gate_contract.rs`: route 20 query classes; assert HyDE only fires on NL.
- `.elai_cc/crates/knowledge/tests/jina_reranker_load_and_score.rs`: load ONNX, score a fixed pair, assert reproducible score (3 trials, std-dev < 1e-6 per existing reranker.rs:1101 determinism pattern).
- `.elai_cc/crates/knowledge/tests/cast_chunker_recall_smoke.rs`: 10 file fixtures, assert Recall@5 delta ≥ +2 vs legacy chunker.
- `.elai_cc/crates/contracts/tests/rag_monotonicity_contract.rs`: harness run on synthetic goldset, assert DEEP MRR > STANDARD MRR.

## Files touched

- CREATE: `.elai_cc/crates/knowledge/src/contextual_retrieval.rs` (~220 LOC)
- CREATE: `.elai_cc/crates/knowledge/src/chunking/cast.rs` (~280 LOC)
- CREATE: `.elai_cc/crates/knowledge/src/deep.rs` (~120 LOC for pool cap + RRF gate)
- MODIFY: `.elai_cc/crates/knowledge/src/reranker.rs` (+~80 LOC for ReRankerModel enum + Jina path)
- MODIFY: `.elai_cc/crates/knowledge/src/pipeline.rs` (+~60 LOC stage wiring)
- MODIFY: `.elai_cc/crates/knowledge/src/hyde.rs` (+~40 LOC gate; file may need creation if B-4 stubbed)
- MODIFY: `.elai_cc/crates/knowledge/src/query_trace.rs` (+~30 LOC stage labels)
- MODIFY: `.elai_cc/crates/cli/src/doctor.rs` (+~80 LOC `knowledge-models` subcommand)
- MODIFY: `.elai_cc/crates/orchestrator/src/ssb.rs` (+6 SsbActionType variants)
- MODIFY: `.elai_cc/elai_cc.toml` schema docs (`[knowledge.reranker]`, `[knowledge.deep]`, `[knowledge.rrf]`, `[knowledge.late_interaction]`)
- CREATE: `.elai_cc/eval/rag/queries.toml` (50-query goldset)
- CREATE: `.elai_cc/eval/rag/run.py` (~150 LOC eval driver)
- CREATE: `workflow/reasoning/E-17-decisions.md`

## Citations / Grounding

- anthropic.com/news/contextual-retrieval (cited in source) — -49% top-20 failure, -67% w/ rerank
- arXiv:2411.11767 (cited in source) — Drowning in Documents, top-250 saturates at top-100
- arXiv:2406.14497 (cited in source) — CodeRAG-Bench low-lexical-overlap failure mode
- arXiv:2510.20609 (cited in source) — Practical Code RAG; PL-PL BM25 word-level dominant
- arXiv:2506.15655 (cited in source) — cAST AST-aware chunking; +4.3 Recall@5 RepoEval, +2.67 Pass@1 SWE-bench
- arXiv:2212.10496 (cited in source) — HyDE original; precise-query weakness confirmed downstream
- arXiv:2506.21182 (cited in source) — MTEB self-reported vs verified divergence; eval-on-private-corpus rationale
- jina.ai/models/jina-reranker-v3 (cited in source) — 61.94 NDCG@10 BEIR, +4.88% vs v2
- qdrant.tech/articles/bm42 (cited in source) — BM42 contested; do NOT adopt as drop-in
- crates.io/crates/next-plaid + github.com/lightonai/fast-plaid (cited in source) — Rust late-interaction option

## Open questions / risks

- ONNX staging may be a packaging/install bug, not a code bug. The `elai doctor` probe in task 1 confirms; if missing, the fix is to add a model-download step to install / a `models/` git-LFS pointer. Surface this in the decision doc.
- cAST recall delta is paper-reported on RepoEval/SWE-bench; ELAI's 135-file private corpus may not see the same +4.3 — conservative target ≥ +2 hedges. If +2 not met, keep legacy chunker behind a flag.
- Anthropic Contextual Retrieval at ingest costs a sidecar call per chunk; ingest time may grow N-fold. Prompt caching at sidecar mitigates but does not eliminate. Measure ingest-time delta before declaring shippable.
- BM42 is explicitly NOT adopted (Qdrant's own subsequent miniCOIL post signals validity concerns). Do NOT let any reviewer push for it.
- HyDE gate on query-class classification: misclassification rate matters. The Layer A classifier (per B-4 plan) must distinguish code-symbol from NL with ≥95% F1 — measure on the eval goldset; if below, fall back to "HyDE off by default everywhere" until classifier improves.
- PLAID/FastPlaid is wired behind a flag for E-17.B; ELAI may decide to skip it entirely after the eval shows STANDARD/DEEP monotonicity restored via tasks 1-4 alone.

## Next

- E-17.B (follow-up, deferred): enable PLAID late-interaction; ablate against the new baseline. Pre-req: E-17 main shipped.
- E-19 (reasoning quality) consumes the retrieval improvements via L3 Interrogation Tree which queries the knowledge layer for MCQ candidates.
- C-8 Viewer Stub Eradication consumes the new SSB rows for the dashboard surface; sequence E-17 before C-8 finishes so C-8 has real rows to render.
- Reason-ModernColBERT (LightOn 2025) for the "DEEP reasoning" tier is a future addition under E-17.C; needs PyLate sidecar.
- HippoRAG PPR upgrade of the existing PageRank repo-map is a Layer 5 enhancement (deferred E-17.D); seeds from query-extracted concepts.
