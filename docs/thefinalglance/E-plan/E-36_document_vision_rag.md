# E-36: Document + vision RAG — ColModernVBERT, PaddleOCR-VL pre-parse, 8th GroundingMethod::VisualArtifactCheck
Objective: Add a vision-RAG path to ELAI's Cosmos artifact tier for visually-rich inputs (architecture PNGs, scanned PDFs, screenshots, slide decks). Stage ColModernVBERT (or ColQwen2.5-v0.2) page-image multi-vectors alongside existing BM25+text-vector. Pre-parse mixed-modality PDFs via PaddleOCR-VL 0.9B. Introduce the 8th GRC method `VisualArtifactCheck` so the designer / scope-planner can verify visual-spatial claims against ingested diagrams. The 7 existing methods (FILE / SOURCE / TEST / WEB / API / API_REGISTRY / CosmosCheck per B-42 SHIPPED) plus the planned ArtifactCheck (B-35) gain a vision sibling here.
Estimated effort: 3 sessions, ~10-14 h (largest E-N in this assigned set; pre-parse pipeline is heaviest piece)
Pre-requisites: B-35 PLANNED (Context Artifacts Pipeline + GroundingMethod::ArtifactCheck; 7th method), B-42 SHIPPED (Cosmos tier + VerificationMethod::CosmosCheck + cosmos_worker.rs ~700 LOC), B-37 SHIPPED (PerRoleRuntime — vision sidecar backend slots in here), E-17 main shipped (RRF infrastructure + reranker pipeline)
Parallel-safe with: E-17 (text RAG only; this report is vision-side), E-37 (different surface), E-39 (cost gates may also touch vision), E-40 (different concern)
Source report: workflow/improvement-sweep-2026-05-19/reports/36-document-vision-rag.md

## Context

ELAI's Cosmos worker today (crates/knowledge/src/cosmos_worker.rs, ~700 LOC shipped per B-42, per elai.map lines 424-428 / 4136 / 4515) handles structured artifacts — SQL DDL, OpenAPI specs, k8s YAML, ADR.md, glossary.md, Terraform — via text chunking and the existing 8th-method `VerificationMethod::CosmosCheck`. That covers the cheap-and-clean text path. The report's TL;DR is precise: text-RAG stays primary where text is already grep-able, and vision-RAG is ADDITIVE for the visually-rich subset where OCR loses structure (charts, scanned contracts, screenshots, architecture PNGs without source). VisRAG ICLR 2025 (openreview.net/pdf/b13eed287ec8fb31700414fded06f07db85891cd.pdf) measures a 20-40% lift over pure-text RAG on visually-rich PDFs — but only on that subset.

The retrieval stack is grounded. ColQwen2.5-v0.2 hits 0.597 nDCG@5 on ViDoRe v2 (huggingface.co/blog/manu/vidore-v2) vs 0.546 for ColPali-v1.3 and 0.530 for ColSmolVLM-v0.1; ColModernVBERT (huggingface.co/ModernVBERT/colmodernvbert) matches models 10x larger and is the right "small for huge corpus" pick for ELAI's 12GB tier (per memory:project_gpu_context — real free VRAM ~1.8GB, sidecar fits 1.92GB Qwen 3B today). PaddleOCR-VL 0.9B (arxiv:2510.14528) tops OmniDocBench v1.5 at 92.56 overall, beating MinerU2.5 at 90.67. Dolphin-v2 3B (arxiv:2505.14059) handles photographed/handwritten input. ColPali multi-vec is storage-heavy (~1030 vectors/page); the Visual RAG Toolkit (arxiv:2602.12510 — verify this arXiv ID via WebSearch; report-cited but the year-tag is suspicious) shows training-free pooling to ~100 vectors with minimal recall loss.

The 8th GRC method `VisualArtifactCheck` is the structural delta. The designer and scope-planner's plan-improvement sweep currently grounds claims via the 7 existing methods (FILE / SOURCE / TEST / WEB / API / API_REGISTRY plus the ArtifactCheck planned in B-35 and the CosmosCheck shipped in B-42). Claims like "the auth flow has 3 redirects" or "users sits between orders and accounts in the ERD" are visual-spatial and unverifiable via text retrieval alone. The new method routes such claims to a ColPali / ColModernVBERT page-image retriever + a VLM verifier (Qwen2.5-VL via the sidecar's PerRoleRuntime, B-37 SHIPPED). Anti-pattern flag from the report: VLM caption of a PNG is LOW reliability per ELAI's source-grounding HARD RULE — use as retrieval signal only, not as authoritative ArtifactCheck output. The VisualArtifactCheck verdict carries a `reliability: VlmCaption | OcrParse | DiagramSource` enum so downstream gates can downweight VlmCaption.

## Existing state (B/C/D coverage)

- B-42 SHIPPED (Cosmos Artifact Tier — `cosmos_worker.rs`, `VerificationMethod::CosmosCheck`, SCHEMA 44, CosmosDrawer): per memory:project_b42_shipped + B-PLAN-SUMMARY line 920 + elai.map line 4136. EXTENSION: E-36 adds vision-side artifact types and a vision worker that composes with the existing cosmos_worker. NO BREAKING CHANGE to the existing text-only path.
- B-35 PLANNED (Context Artifacts Pipeline): per B-PLAN-SUMMARY line 707; defines `GroundingMethod::ArtifactCheck` (7th method) + `[[knowledge.artifacts]]` TOML. EXTENSION: E-36 adds an 8th `VisualArtifactCheck` method that dispatches when the claim type is visual-spatial; coordinate the ArtifactCheck and VisualArtifactCheck dispatchers so they share a registry surface.
- B-37 SHIPPED (PerRoleRuntime + 11 backends — commit fd5c2e5): per memory:project_b37_shipped. EXTENSION: register a new role `doc_parser` with backends `paddleocr_vl_local`, `mineru_local`, `dolphin_local`, plus optional cloud fallbacks `mistral_ocr_cloud`, `reducto_cloud`, `llamaparse_cloud`. Reuses the RuntimeAdapter trait wholesale.
- E-17 main (shipped or in-flight): cAST chunker + Jina-rerank-v3 + 3-ranker RRF + Anthropic Contextual Retrieval. EXTENSION: E-36 extends RRF from 3 lists (BM25 / dense-text / code_vec) to 4 lists when a vision multi-vec index is present (BM25 / dense-text / code_vec / colpali-image). No conflict — the RRF surface from E-17 is the integration hook.
- C-7 PLANNED (Cosmos Worker Wiring): per C-SUMMARY_INDEX lines 200-208. DELTA: E-36 sequences AFTER C-7 lands the cosmos SSB action types (CosmosRefreshStarted / CosmosArtifactUpdated / CosmosArtifactDecayed); E-36 adds 5 NEW vision-specific action types alongside (see WIRING_CONTRACT).
- Per memory:project_b40_shipped — Idea-Framer + Research Presets + Critic Loop ship 5 SSB variants and SCHEMA_VERSION 41. E-36 increments SCHEMA_VERSION (pin TBD; coordinate with whatever pin is current at execution time — likely 45+ given B-41/B-42 land in 43/44).

## Reads (mandatory before coding)

- `F:\.ELAI_workflow\workflow\improvement-sweep-2026-05-19\reports\36-document-vision-rag.md` (full — 112 lines)
- `F:\.ELAI_workflow\.elai_cc\crates\knowledge\src\cosmos_worker.rs` (existing B-42 shipped impl; per elai.map lines 424-428 lines 195/247/391/425/466/493/619/640/695/806 for the key extension points: `refresh_one`, `record_grounding_used`, `manifest`, `CosmosError`)
- `F:\.ELAI_workflow\.elai_cc\crates\reasoning\src\verification.rs` (existing VerificationMethod enum at line 452; add `VisualArtifactCheck` variant alongside `CosmosCheck`/`FileCheck`/`WebCheck`)
- `F:\.ELAI_workflow\.elai_cc\crates\contracts\tests\artifact_grounding_contract.rs` (existing per elai.map line 2559; assert pattern for new variant)
- `F:\.ELAI_workflow\docs\thefinalglance\B-plan\B-PLAN-SUMMARY.md` lines 707-740 (B-35 ArtifactCheck design; 7th method)
- `F:\.ELAI_workflow\docs\thefinalglance\B-plan\B-PLAN-SUMMARY.md` lines 920-960 (B-42 Cosmos shipped surface)
- `F:\.ELAI_workflow\workflow\improvement-sweep-2026-05-19\E-plan\E-17_rag_retrieval_quality.md` (RRF integration hook + reranker pattern)
- arxiv:2510.14528 PaddleOCR-VL paper — WebFetch via paper-ingestor mode=paper
- arxiv:2505.14059 Dolphin-v2 paper — WebFetch
- arxiv:2407.01449 ColPali ICLR 2025 paper — already cached per memory; verify path
- huggingface.co/ModernVBERT/colmodernvbert model card — WebFetch
- huggingface.co/blog/manu/vidore-v2 — WebFetch (ViDoRe v2 leaderboard numbers)
- VisRAG ICLR 2025 openreview PDF — WebFetch

## Tasks

1. Stage ColModernVBERT (default) and ColQwen2.5-v0.2 (alternate) under `core::paths::models_dir_path().join("colmodernvbert")` and `.../colqwen2_5_v02`. Use the same pattern as E-17's `elai doctor knowledge-models` probe: extend that probe with a `--vision` flag that checks the vision retriever model presence + tokenizer hash + size, reporting via `vision_models_probe` SSB row. Default in `[knowledge.vision_retriever] model = "colmodernvbert"`. Reference: huggingface.co/ModernVBERT/colmodernvbert "matches 10x larger models, open-weight".

2. Implement `ImageArtifactChunker` at `crates/knowledge/src/chunking/image_artifact.rs` (~250 LOC). Inputs: a path to PNG/SVG/PDF-page-render. Outputs: a `VisionChunk { source_blake3, page_idx, multi_vec: Vec<[f16; 128]>, pooled: bool, vlm_caption: Option<String>, parser_confidence: f32 }`. Pooling: training-free reduction to ~100 vectors per page (target 10x storage savings per the report's Visual RAG Toolkit cite; VERIFY arXiv ID `2602.12510` via WebSearch — report's date-tag is anomalous). Storage: per-project SQLite table `vision_chunks` keyed by blake3 of the source file (matches existing cosmos_worker stale-check pattern; CLAUDE.md HARD RULE blake3 only — NEVER blake2/sha2).

3. Add the `PdfStructuredChunker` variant for mixed-modality PDFs (~280 LOC at `crates/knowledge/src/chunking/pdf_structured.rs`). Invokes PaddleOCR-VL 0.9B via a Python sidecar role `doc_parser` (B-37 RuntimeAdapter — register backend `paddleocr_vl_local`). Output: `{ markdown: String, page_html_tables: Vec<String>, figure_crops: Vec<PathBuf>, page_confidences: Vec<f32>, fallback_recommended: bool }`. The markdown → existing E-17 text chunker; figure crops → ImageArtifactChunker. The `fallback_recommended` bit fires (per report Task 8) when any page_confidence < 0.85 → force vision-multi-vec indexing for that page as well. Reference: arxiv:2510.14528 (OmniDocBench v1.5 92.56) and arxiv:2412.07626 (per-doc-type variance).

4. Register 4 new artifact types in the Cosmos tier registry (extend `crates/knowledge/src/cosmos_worker.rs`): `Diagram { source_lang: Mermaid | PlantUML | D2 | Excalidraw, source_text: String, rendered_png_blake3: Option<Blake3> }`, `ArchImage { image_path: PathBuf, vlm_caption: Option<String>, image_blake3: Blake3 }`, `Screenshot { image_path, captured_url: Option<String>, image_blake3 }`, `ScannedDoc { pdf_path: PathBuf, parser: ParserBackend, parser_confidence: f32, page_count: u32 }`. Stale-check policy per type: Diagram fires on either source change OR rendered-png change; ArchImage / Screenshot / ScannedDoc fire on blake3 mismatch of the source file. Reference: dev.to/akari_iku token-efficiency analysis — diagram source is 24x more token-efficient than rendered JSON.

5. Add `GroundingMethod::VisualArtifactCheck` to the GRC dispatcher (8th method alongside CosmosCheck shipped per B-42, ArtifactCheck planned per B-35). Variant lives in the same enum at `crates/reasoning/src/verification.rs` (extend the existing enum at line 452 per elai.map). Dispatch rule: claims tagged with `claim_class: visual_spatial` (e.g. "the auth flow has N redirects", "X sits between Y and Z in the ERD", "the dashboard shows a histogram") route here. Implementation invokes (a) the vision retriever to fetch top-k page images for the claim, (b) the sidecar VLM (Qwen2.5-VL via PerRoleRuntime role `vision_verifier`) to issue a verification verdict with structured output `{ verdict: Confirm | Refute | Indeterminate, evidence_pages: Vec<PageRef>, reliability: VlmCaption | OcrParse | DiagramSource }`. Report HARD RULE: VlmCaption is LOW reliability (matches ELAI source-grounding rule for decompiled output) — downstream gates MUST downweight.

6. Extend RRF fusion to 4 lists when a vision index is present. The existing E-17 RRF pipeline (`crates/knowledge/src/pipeline.rs`, `[knowledge.rrf] weights`) fuses BM25 + code_vec + text_vec. Add `vision_vec` weight (default 0.6 — slightly lower than text-vector since vision precision degrades on text-heavy queries per REAL-MM-RAG arxiv:2502.12342). Surface as `[knowledge.rrf] weights = {bm25 = 1.0, code_vec = 1.0, text_vec = 0.8, vision_vec = 0.6}`. Emit a `rrf_vision_component_weight` SSB row per query. Skip the vision component when the per-project policy `[knowledge.vision_rag] enabled = false`.

7. Per-project vision-RAG enable gate. Add `[knowledge.vision_rag] enabled = false` default (HARD opt-in — storage and VRAM cost are non-trivial; report Open Question 1 lists 25KB/page × 1000 pages = 25MB per project as the target ceiling). When disabled: ImageArtifactChunker still runs at ingest IF source is in the visually-rich subset (per the report's Per-document-type table), but the multi-vec index is skipped — only VLM caption + page hash stored. When enabled: full multi-vec index. Emit `vision_rag_policy_decision` SSB row per project with `enabled`, `storage_budget_mb`, `actual_storage_mb`.

8. Cloud-fallback wiring for high-stakes parses. Register `mistral_ocr_cloud`, `reducto_cloud`, `llamaparse_cloud` backends under the B-37 RuntimeAdapter `doc_parser` role. Trigger rule: `parser_confidence < [knowledge.doc_parser] cloud_fallback_threshold` (default 0.7 — below this, the local PaddleOCR-VL output is unreliable per Task 3). Same `--mode cloud|hybrid|local` flag from C-5 Cloud Mode Hardening (PLANNED) gates eligibility; `local` mode REFUSES the cloud fallback and emits `doc_parser_cloud_fallback_blocked_by_mode` SSB row. Cost guard: each cloud parse counts against the C-5 `max_spend_per_task` ceiling (E-39 strengthens this to a HARD cap).

9. Mini-benchmark harness for vision-RAG recall@5 on ELAI's actual artifact corpus. Per report Open Question 3: ViDoRe v2 0.597 is a different corpus distribution; ELAI's own diagrams / screenshots / scanned docs may differ. Build 50 hand-labeled query→page pairs at `.elai_cc/eval/vision_rag/queries.toml` (separate from E-17's `eval/rag/queries.toml` — different evaluator, different metrics). Report nDCG@5 + Recall@5 + MRR per backend (ColModernVBERT vs ColQwen2.5-v0.2). Gate: at least one backend must hit nDCG@5 ≥ 0.50 on the ELAI corpus before vision-RAG default-enables in any policy.

10. Dashboard surface. Extend the existing CosmosDrawer (per B-42 SHIPPED — `crates/dashboard-v2/src/lib/components/CosmosDrawer.svelte`) OR create a sibling `VisionRAGDrawer.svelte` (per the report's Anti-pattern flag about hand-crafted PDF chunkers — keep this surface composable). Show: per-artifact vision-index status (indexed / pending / skipped-by-policy), storage usage, last parse confidence, last VisualArtifactCheck verdict (Confirm / Refute / Indeterminate + reliability tier). The Tauri commands `get_vision_artifact_status` and `get_vision_artifact_check_recent` live in `crates/dashboard-v2/src-tauri/src/commands.rs` (extension pattern matches existing `get_cosmos_*` per memory:project_b42_shipped).

## WIRING_CONTRACT

produces_symbols:
  - knowledge::chunking::image_artifact::ImageArtifactChunker (new)
  - knowledge::chunking::image_artifact::VisionChunk (new struct)
  - knowledge::chunking::pdf_structured::PdfStructuredChunker (new)
  - knowledge::cosmos_worker::ArtifactType::{Diagram, ArchImage, Screenshot, ScannedDoc} (4 new variants)
  - knowledge::cosmos_worker::ParserBackend (new enum: PaddleOcrVl, MinerU, Dolphin, MistralCloud, ReductoCloud, LlamaParseCloud)
  - reasoning::verification::GroundingMethod::VisualArtifactCheck (new variant; 8th method)
  - reasoning::verification::VisualArtifactVerdict { Confirm, Refute, Indeterminate } (new enum)
  - reasoning::verification::ArtifactReliability { VlmCaption, OcrParse, DiagramSource } (new enum)
  - knowledge::pipeline::RrfFusion4 (new — extends E-17's RrfFusion3 with vision_vec)
  - SsbActionType::VisionChunkEmitted (new)
  - SsbActionType::VisionFallbackTriggered (new — fires when parser_confidence < threshold)
  - SsbActionType::VisualArtifactCheckVerdict (new)
  - SsbActionType::VisionRagPolicyDecision (new)
  - SsbActionType::DocParserCloudFallbackBlockedByMode (new)
  - cli::doctor::vision_models_probe (extends `elai doctor knowledge-models --vision`)
consumes_symbols:
  - knowledge::cosmos_worker::CosmosWorker (existing; per elai.map line 4515; extend with vision artifact registration — do NOT replace)
  - knowledge::cosmos_worker::CosmosError (existing; per elai.map line 4136)
  - reasoning::verification::VerificationMethod (existing at verification.rs:452; add VisualArtifactCheck variant)
  - knowledge::pipeline::run_query (existing E-17 entry point; new 4-list RRF stage slots in)
  - orchestrator::runtime_orchestrator::PerRoleRuntime (B-37 SHIPPED; register `doc_parser` + `vision_verifier` roles)
  - core::paths::models_dir_path (B-1 helper)
  - blake3 crate (HARD RULE — never blake2/sha2)
reachable_from:
  - crates/cli/src/main.rs (via `elai doctor knowledge-models --vision`)
  - crates/orchestrator/src/task_runner.rs::knowledge_query (via 4-list RRF when vision enabled)
  - crates/dashboard-v2/src-tauri/src/commands.rs (via `get_vision_artifact_status`, `get_vision_artifact_check_recent`)
  - crates/reasoning/src/verification.rs::dispatch (via VisualArtifactCheck branch)
stub_ban:
  - no .unwrap() in production vision pipelines (CLAUDE.md HARD RULE)
  - no blake2 / sha2 for source-file hashing (CLAUDE.md HARD RULE: blake3 only)
  - no VlmCaption-only output as authoritative VisualArtifactCheck verdict (must carry reliability tier)
  - no full-1030-vector indexing (must pool to ~100/page per Task 2)
  - no auto-enabling vision-RAG default — `[knowledge.vision_rag] enabled = false` is the shipping default
  - no MarkItDown for image-PDFs (report anti-pattern; no built-in OCR)
  - no Nougat (report anti-pattern; unmaintained dep tree)
  - no cloud fallback when `--mode local`
frontend_pair: crates/dashboard-v2/src/lib/components/VisionRAGDrawer.svelte (new) OR extend existing CosmosDrawer
observable_effect:
  trigger: "elai /research --mode cloud 'verify that the auth flow in docs/architecture/auth.png has exactly 3 redirects'"
  expect_ssb: "vision_chunk_emitted for auth.png; vision_artifact_check_verdict with verdict=Confirm|Refute|Indeterminate + reliability tier; rrf_vision_component_weight non-zero"
  expect_artifact: "vision_chunks table has row keyed by blake3(auth.png); evidence_pages list non-empty on Confirm/Refute; query_trace 4-stage pipeline (bm25 + code_vector + text_vector + vision_vector) when vision enabled"

## DETERMINISTIC_CHECK

```bash
# 1. blake3 used for source-file hashing (NOT blake2/sha2)
rg "blake3::hash" .elai_cc/crates/knowledge/src/chunking/image_artifact.rs
# Expected: ≥1 hit
rg "blake2|sha2" .elai_cc/crates/knowledge/src/chunking/image_artifact.rs
# Expected: zero hits

# 2. VisualArtifactCheck variant exists alongside CosmosCheck
rg "VisualArtifactCheck" .elai_cc/crates/reasoning/src/verification.rs
rg "GroundingMethod::VisualArtifactCheck|VerificationMethod::VisualArtifactCheck" .elai_cc/crates --type rust
# Expected: enum variant + ≥1 dispatch site

# 3. 4 new artifact types in cosmos_worker
rg "ArtifactType::(Diagram|ArchImage|Screenshot|ScannedDoc)" .elai_cc/crates/knowledge --type rust
# Expected: ≥4 hits

# 4. PaddleOCR-VL backend registered in PerRoleRuntime
rg "doc_parser|paddleocr_vl_local|PaddleOcrVl" .elai_cc/crates/orchestrator --type rust
# Expected: ≥3 hits (role decl + backend variant + dispatch)

# 5. Pooling to ~100 vectors per page (NOT full 1030)
rg "MAX_VECTORS_PER_PAGE|pool.*vec|VISION_POOLED_DIM" .elai_cc/crates/knowledge/src/chunking/image_artifact.rs
# Expected: pooling constant ≤ 128 visible

# 6. Vision-RAG default-disabled
rg "vision_rag.*enabled.*=.*false" .elai_cc/elai_cc.toml
# Expected: ≥1 hit (HARD opt-in)

# 7. New SSB action variants
rg "VisionChunkEmitted|VisionFallbackTriggered|VisualArtifactCheckVerdict|VisionRagPolicyDecision|DocParserCloudFallbackBlockedByMode" .elai_cc/crates/orchestrator/src/ssb.rs
# Expected: 5 hits

# 8. RRF extended to 4 lists
rg "vision_vec|RrfFusion4" .elai_cc/crates/knowledge/src/pipeline.rs
# Expected: ≥2 hits

# 9. Vision-RAG mini-benchmark exists
test -f .elai_cc/eval/vision_rag/queries.toml
test -f .elai_cc/eval/vision_rag/run.py
# Expected: both files exist

# 10. Probe extension
elai doctor knowledge-models --vision
# Expected: exit 0; reports colmodernvbert OR colqwen2.5-v02 present with non-zero size
```

## ACCEPTANCE

- [ ] `elai doctor knowledge-models --vision` reports the chosen vision retriever model staged with non-zero file size + tokenizer hash
- [ ] PaddleOCR-VL 0.9B sidecar backend boots and processes a fixture PDF, emitting `parser_confidence` per page
- [ ] ImageArtifactChunker pools per-page vectors to ~100 (storage budget per Open Question 1: ≤25KB/page at fp16)
- [ ] 4 new artifact types (Diagram / ArchImage / Screenshot / ScannedDoc) register in cosmos_worker with correct stale-check policy
- [ ] VisualArtifactCheck (8th GRC method) dispatches on `claim_class: visual_spatial`, returns verdict + reliability tier
- [ ] VlmCaption-tier verdicts carry LOW reliability flag and downstream gates downweight per ELAI source-grounding HARD RULE
- [ ] RRF extends to 4 lists when `[knowledge.vision_rag] enabled = true`; SSB row confirms vision_vec component participation
- [ ] Default `[knowledge.vision_rag] enabled = false` — HARD opt-in
- [ ] Cloud parse fallback triggers only when `parser_confidence < cloud_fallback_threshold` AND `--mode != local`
- [ ] Vision-RAG mini-benchmark hits nDCG@5 ≥ 0.50 on the 50-query ELAI corpus for at least one backend (gate per Task 9)
- [ ] No blake2/sha2 anywhere in new code (CLAUDE.md HARD RULE); blake3-only

## Tests to add

- `crates/contracts/tests/vision_artifact_check_dispatch_contract.rs`: claim with `claim_class: visual_spatial` routes to VisualArtifactCheck; claim with `claim_class: file_existence` routes to FileCheck (no regression on existing 7 methods)
- `crates/contracts/tests/vision_chunk_pooling_contract.rs`: 1030-vector input pools to ≤128 vectors output; recall delta vs unpooled within ±2 points on a synthetic fixture
- `crates/contracts/tests/paddleocr_vl_parser_confidence_contract.rs`: low-confidence fixture (synthetic noisy scan) triggers `fallback_recommended=true`; high-confidence fixture does not
- `crates/contracts/tests/visual_artifact_reliability_tier_contract.rs`: VlmCaption-only verdict carries LOW reliability; OcrParse carries MEDIUM; DiagramSource carries HIGH
- `crates/contracts/tests/rrf_4list_fusion_contract.rs`: 4-list RRF with vision_vec produces stable ordering; deterministic across 3 runs
- `crates/knowledge/tests/cosmos_worker_vision_artifact_register.rs`: register Diagram artifact, mutate rendered_png, assert stale fires on rendered_png_blake3 change
- `eval/vision_rag/run.py`: nDCG@5 ≥ 0.50 gate on the 50-query ELAI corpus

## Files touched

- CREATE: `crates/knowledge/src/chunking/image_artifact.rs` (~250 LOC)
- CREATE: `crates/knowledge/src/chunking/pdf_structured.rs` (~280 LOC)
- MODIFY: `crates/knowledge/src/cosmos_worker.rs` (+~120 LOC for 4 new artifact types + stale-check policy)
- MODIFY: `crates/knowledge/src/pipeline.rs` (+~50 LOC for RrfFusion4 with vision_vec slot)
- MODIFY: `crates/reasoning/src/verification.rs` (+~40 LOC for VisualArtifactCheck variant + VisualArtifactVerdict + ArtifactReliability enums)
- CREATE: `crates/orchestrator/src/vision_verifier.rs` (~180 LOC — VLM verifier role wrapper)
- MODIFY: `crates/orchestrator/src/runtime_orchestrator.rs` (+~30 LOC for `doc_parser` + `vision_verifier` role registration)
- MODIFY: `crates/orchestrator/src/ssb.rs` (+5 SsbActionType variants)
- MODIFY: `crates/cli/src/doctor.rs` (+~60 LOC for `--vision` flag in `knowledge-models` probe)
- MODIFY: `crates/dashboard-v2/src-tauri/src/commands.rs` (+~50 LOC for `get_vision_artifact_status` / `get_vision_artifact_check_recent`)
- CREATE: `crates/dashboard-v2/src/lib/components/VisionRAGDrawer.svelte` (~140 LOC) OR extend CosmosDrawer
- MODIFY: `elai_cc.toml` schema docs (`[knowledge.vision_retriever]`, `[knowledge.vision_rag]`, `[knowledge.doc_parser]`)
- CREATE: `eval/vision_rag/queries.toml` (50-query goldset)
- CREATE: `eval/vision_rag/run.py` (~160 LOC eval driver)
- CREATE: `crates/contracts/tests/vision_artifact_check_dispatch_contract.rs`
- CREATE: `crates/contracts/tests/vision_chunk_pooling_contract.rs`
- CREATE: `crates/contracts/tests/paddleocr_vl_parser_confidence_contract.rs`
- CREATE: `crates/contracts/tests/visual_artifact_reliability_tier_contract.rs`
- CREATE: `crates/contracts/tests/rrf_4list_fusion_contract.rs`
- CREATE: `workflow/reasoning/E-36-decisions.md`

## Citations / Grounding

- huggingface.co/blog/manu/vidore-v2 (cited in source) — ColQwen2.5-v0.2 0.597 nDCG@5, ColModernVBERT matches 10x larger
- arxiv:2407.01449 (cited in source) — ColPali ICLR 2025 late-interaction over page images
- arxiv:2510.14528 (cited in source) — PaddleOCR-VL 0.9B; OmniDocBench v1.5 92.56
- arxiv:2505.14059 (cited in source) — Dolphin-v2 ACL 2025 ByteDance; handwriting + photographed docs
- arxiv:2412.07626 (cited in source) — OmniDocBench CVPR 2025; 1651 pages, 10 doc types, 5 layouts
- arxiv:2502.12342 (cited in source) — REAL-MM-RAG ACL 2025 IBM; vision retriever brittle on tables + query-rephrase
- arxiv:2411.04952 (cited in source) — M3DocRAG Bloomberg; multi-page multi-doc VQA SOTA with ColPali + Qwen2-VL 7B
- VisRAG ICLR 2025 (cited in source) — 20-40% lift over text-RAG on visually-rich PDFs
- arxiv:2501.17887 (cited in source) — Docling IBM AAAI 2025; MIT-licensed 16-core 0.6-1.3 pg/s; 97.9% complex-table accuracy
- arxiv:2505.22879 (cited in source) — KubeDiagrams k8s YAML → architecture diagram
- huggingface.co/ModernVBERT/colmodernvbert (cited in source) — open-weight; matches 10x larger
- huggingface.co/learn/cookbook/en/multimodal_rag_using_document_retrieval_and_smol_vlm (cited in source) — quantized ColSmolVLM on T4 16GB
- dev.to/akari_iku/analyzing-the-best-diagramming-tools-for-the-llm-age-based-on-token-efficiency-5891 (cited in source) — text-first diagrams 24x more token-efficient
- mistral.ai/news/mistral-ocr (cited in source) — Mistral OCR API; equation fidelity inconsistent on dense math

## Open questions / risks

- arXiv:2602.12510 Visual RAG Toolkit citation (Task 2 pooling reference): the report's year-tag `2602` is anomalous (today's date is 2026-05-19 per env; arXiv IDs use YYMM format so `2602` = February 2026 is plausible but pre-publication). VERIFY arXiv ID + paper existence via WebSearch before adopting the pooling algorithm; if the paper doesn't exist or pooling claim isn't grounded, fall back to a simpler max-pool or mean-pool with measured recall delta on the eval harness (Task 9).
- Storage budget gating (report Open Question 1): 25KB/page × 1000 pages = 25MB per project; 1000+ pages multi-project = GB-scale. Confirm `[knowledge.vision_rag] storage_budget_mb = 500` default holds for typical projects; surface `actual_storage_mb > storage_budget_mb` SSB row.
- VRAM headroom (report Open Question 2): sidecar runs Qwen2.5-Coder-3B at ~1.92 GB; adding Qwen2.5-VL-3B for caption would push to ~3.8 GB. Per memory:project_gpu_context — real free VRAM ~1.8 GB. Result: cannot run both simultaneously on the user's local box. Mitigation: gate VLM caption pass to ingest-time only (batch once) OR route via the remote sidecar pod (memory:feedback_stop_pod_after_session — billable; stop after each batch).
- Vision-RAG recall ceiling (report Open Question 3): ViDoRe v2 0.597 open-weight is a paper-corpus number; ELAI corpus may be smaller and more technical-skewed. Task 9 mini-benchmark is the gate — do NOT default-enable vision-RAG without nDCG@5 ≥ 0.50 measured on ELAI's actual artifacts.
- VlmCaption authoritativeness: the report explicitly flags VLM-caption-as-ground-truth as a HARD anti-pattern (matches ELAI's source-grounding HARD RULE for decompiled output = LOW reliability). The ArtifactReliability enum is the structural defense — downstream gates (auditor, scope-planner) MUST inspect it. Confirm dispatchers in those crates check the reliability tier.
- Open-license vs closed-license retrieval ceiling (report Open Question 5): closed-license Qwen2.5-VL fine-tunes hit ~0.90 ViDoRe v1; open ceiling ~0.60 v2. A `--cloud doc-rerank` path via Cohere Embed v4 / Voyage multimodal-3 is a future E-N (defer; out of scope here).
- C-7 sequencing: C-7 (Cosmos Worker Wiring, PLANNED) lands the cosmos SSB action types FIRST. E-36 depends on C-7's SSB substrate. Coordinate ordering — if C-7 is in-flight when E-36 starts, scope-planner blocks E-36 entry.

## Next

E-36 unblocks: cross-modal RAG for the designer's plan sweep (visual-spatial claims now groundable); B-35 ArtifactCheck composes with VisualArtifactCheck via the shared `claim_class` dispatcher. E-17 + E-36 together complete the RAG retrieval picture (text + vision in one RRF). Future E-N candidates: (a) cloud doc-rerank via Cohere Embed v4 or Voyage multimodal-3 for the open-vs-closed ceiling gap; (b) Reason-ModernColBERT late-interaction sidecar for "DEEP reasoning" tier (cross-references E-17.C); (c) PageRank over vision-chunks for visually-rich repos (the Aider repomap pattern applied to image neighborhoods). Coordinate with E-39 (cost) for the cloud-fallback spend gates — vision parses are non-trivial cost.
