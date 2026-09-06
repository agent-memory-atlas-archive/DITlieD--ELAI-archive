# Document + vision RAG for ELAI Cosmos (May 2026)

## TL;DR
- Add a vision-RAG path to Cosmos for visually-rich artifacts (architecture PNGs, scanned ADRs, screenshots, slide decks). ColQwen2.5-v0.2 / ColModernVBert as the late-interaction retriever; fits ~3B param, runs on 12GB consumer GPU. ViDoRe v2 nDCG@5 ~0.597 for ColQwen2.5 vs 0.546 for ColPali-v1.3 vs 0.530 for ColSmolVLM ([HF/manu](https://huggingface.co/blog/manu/vidore-v2)).
- Keep text-RAG primary for structured artifacts already covered by Cosmos (SQL DDL, OpenAPI, k8s YAML, ADR.md, glossary.md, Terraform). Vision adds cost+latency with no gain when text is already structured and grep-able; VisRAG's 20-40% lift is over a *pure-text* baseline on visually-rich PDFs, not on already-parsed YAML/JSON ([VisRAG ICLR 2025](https://openreview.net/pdf/b13eed287ec8fb31700414fded06f07db85891cd.pdf)).
- For mixed-modality PDFs entering the artifact tier (PRDs, vendor whitepapers, scanned contracts): pre-parse with MinerU2.5 or Dolphin-v2 to extract Markdown + tables + figure crops, then dual-index: text chunks AND page-image multi-vectors. PaddleOCR-VL 0.9B leads OmniDocBench v1.5 at 92.56 overall ([arxiv 2510.14528](https://arxiv.org/abs/2510.14528)).
- Add 4 new artifact types to Cosmos: `Diagram` (Mermaid/PlantUML/D2 text source — preferred), `ArchImage` (PNG/SVG, ingested via VLM caption + ColPali embedding), `Screenshot` (web UI captures), `ScannedDoc` (handwritten / camera-captured). Each gets its own chunker + stale-check policy.
- New 8th GRC grounding method: `VisualArtifactCheck` — designer's plan sweep can verify claims like "the auth flow has 3 redirects" against ingested architecture diagrams via ColPali page-image retrieval + VLM verification.

## Key sources (16, all 2024-2026)
URL | what | takeaway
---|---|---
[arxiv 2407.01449 v6](https://arxiv.org/html/2407.01449v6) | ColPali ICLR 2025 paper (already cached) | Late-interaction over page images. ViDoRe v1 average 81.3 nDCG@5; obsoletes OCR+chunk pipelines for visually-rich docs.
[HF blog manu vidore-v2](https://huggingface.co/blog/manu/vidore-v2) | ViDoRe v2 leaderboard | ColQwen2.5-v0.2 = 0.597 nDCG@5; ColPali-v1.3 = 0.546; ColSmolVLM-v0.1 = 0.530; ColSmol-256M = 0.397. Harder benchmark, scores lower than v1.
[arxiv 2502.12342](https://arxiv.org/abs/2502.12342) | REAL-MM-RAG ACL 2025 (IBM) | Real-world enterprise multi-modal benchmark; reveals weakness on table-heavy docs and query-rephrasing robustness — vision retrievers brittle when query wording shifts.
[arxiv 2411.04952](https://arxiv.org/abs/2411.04952) | M3DocRAG (Bloomberg) | Multi-page multi-doc benchmark M3DocVQA (3K+ PDFs, 40K+ pages); ColPali + Qwen2-VL 7B SOTA on MP-DocVQA. ICCV 2025 workshop accepted.
[VisRAG ICLR 2025](https://openreview.net/pdf/b13eed287ec8fb31700414fded06f07db85891cd.pdf) | VisRAG paper | VLM-retriever + VLM-generator end-to-end vision RAG; 20-40% lift over text RAG on visually-rich docs.
[arxiv 2510.14528](https://arxiv.org/abs/2510.14528) | PaddleOCR-VL 0.9B (Oct 2025) | NaViT + ERNIE-4.5-0.3B; OmniDocBench v1.5 overall 92.56 (beats MinerU2.5 90.67); text edit dist 0.035; tops academic-lit, books, exam papers.
[arxiv 2505.14059](https://arxiv.org/abs/2505.14059) | Dolphin ACL 2025 (ByteDance) | Analyze-then-parse two-stage; layout-first + parallel content parsing; Dolphin-v2 (Dec 2025) is 3B params, 21-element detection, handles handwriting + photographed docs.
[arxiv 2412.07626](https://arxiv.org/html/2412.07626v1) | OmniDocBench CVPR 2025 | 1651 pages, 10 doc types, 5 layouts, 5 languages — current standard for OCR/parser evaluation. MinerU2-VLM, Marker-1.7.1, Dolphin, Nanonets-OCR-s, Qwen2.5-VL-7B benchmarked.
[arxiv 2501.17887](https://arxiv.org/pdf/2501.17887) | Docling AAAI 2025 (IBM) | MIT-licensed, 16-core CPU 0.6-1.3 pages/sec, 97.9% complex-table accuracy; DocLayNet + TableFormer; commodity-hardware focus.
[Mistral OCR Mar 2025](https://mistral.ai/news/mistral-ocr) | Mistral OCR API | Handles interleaved imagery, math, tables, LaTeX; speed-first managed API. Equation fidelity inconsistent on dense notation per [simonw](https://simonwillison.net/2025/Mar/7/mistral-ocr/).
[MarkTechPost vision-RAG vs text-RAG](https://www.marktechpost.com/2025/09/24/vision-rag-vs-text-rag-a-technical-comparison-for-enterprise-search/) | Sep 2025 comparison | Text-RAG cheaper/faster when extraction is clean; vision-RAG wins on charts, scans, complex layouts, multilingual where OCR drops chars.
[arxiv 2505.17166](https://arxiv.org/pdf/2505.17166) | ViDoRe v2 paper | New harder benchmark; current SOTA closed-license Qwen2.5-VL fine-tunes >90 nDCG@5 v1, ~0.60 v2. Open license ceiling ~0.60 on v2.
[arxiv 2308.13418](https://arxiv.org/pdf/2308.13418) | Nougat (Meta) | Academic-PDF LaTeX OCR. Unmaintained repo, dep tree minefield per 2025 evals — avoid for new builds.
[ModernVBERT colmodernvbert](https://huggingface.co/ModernVBERT/colmodernvbert) | ColModernVBERT | Matches models 10x larger on visual doc benchmarks; open-weight; fits the "small model for indexing huge corpus" cell.
[Reducto comparison Q1 2026](https://llms.reducto.ai/document-parser-comparison) | Docling/LlamaParse/Unstructured/Reducto comparison | Reducto +20% parse accuracy on real-world docs over OSS; LlamaParse v2 (Dec 2025) simplifies tier picker; managed services win on enterprise tables + provenance.
[HF cookbook ColSmolVLM + SmolVLM](https://huggingface.co/learn/cookbook/en/multimodal_rag_using_document_retrieval_and_smol_vlm) | Quantized ColSmolVLM | Runs on Colab free-tier T4 16GB; quantized variants fit ELAI's 1.8GB-free 12GB workstation tier.

## Vision-RAG vs text-RAG bake-off

| Doc type | Winner | Numbers | Why |
|---|---|---|---|
| Already-structured SQL DDL / OpenAPI / k8s YAML / ADR.md / Terraform .tf | text-RAG | n/a — vision adds zero info, ~10x cost | Text already grep-able; chunk-by-statement preserves semantics; embedding text faster |
| Scanned PDFs (contracts, vendor whitepapers, regulatory docs) | vision-RAG | VisRAG +20-40% over text-RAG ([VisRAG](https://openreview.net/pdf/b13eed287ec8fb31700414fded06f07db85891cd.pdf)) | OCR loses tables, headers, multi-column reading order |
| Charts, plots, dashboards, screenshots | vision-RAG | Visual-RAG ([2502.16636](https://arxiv.org/abs/2502.16636)) shows MLLMs need image evidence | Text extraction misses spatial/color/encoding info |
| Mathematical / scientific PDFs | hybrid (Marker text + ColQwen image) | Marker 10.5/12 on equation extraction; Nougat unmaintained | Math survives Marker's LaTeX path; layout context needs image fallback |
| Multi-page multi-doc QA | vision-RAG | M3DocRAG SOTA on MP-DocVQA with ColPali + Qwen2-VL 7B ([2411.04952](https://arxiv.org/abs/2411.04952)) | Cross-page evidence stitching needs page-image retrieval |
| Handwritten notes / photographed whiteboards | vision-VLM end-to-end | Dolphin-v2 3B handles photographed/handwritten ([2505.14059](https://arxiv.org/abs/2505.14059)) | Classical OCR <30% on handwriting; VLMs ~80%+ |
| Mermaid / PlantUML / D2 source files | text-RAG | n/a — diagram source IS text | Already structured; LLMs natively write/read syntax ([dev.to token efficiency](https://dev.to/akari_iku/analyzing-the-best-diagramming-tools-for-the-llm-age-based-on-token-efficiency-5891)) |
| PNG / SVG architecture diagrams (no source) | vision-RAG + VLM caption | VLM caption + ColPali page embedding | No text in image → must use vision |
| Excalidraw .excalidraw JSON | text-RAG | Native JSON | Already structured; can render to PNG for fallback |
| Table-heavy enterprise docs | vision-RAG (with caveats) | REAL-MM-RAG ([2502.12342](https://arxiv.org/abs/2502.12342)) shows vision retrievers brittle here | Tables hard for both; fine-tuned vision wins but query-rephrasing hurts |

## Engine landscape table

Tool | open/closed | speed | quality | consumer GPU | rust port | citation
---|---|---|---|---|---|---
PaddleOCR-VL 0.9B | open (Apache) | fast (0.9B params) | OmniDocBench v1.5: 92.56 (#1) | yes (~2GB VRAM) | no — Paddle inference | [arxiv 2510.14528](https://arxiv.org/abs/2510.14528)
MinerU2.5 | open (Apache) | medium | OmniDocBench v1.5: 90.67 | yes (1.2B) | no | [arxiv 2409.18839](https://arxiv.org/pdf/2409.18839)
Dolphin-v2 (ByteDance) | open (MIT) | medium | strong on photographed + handwritten | yes (3B) | no | [arxiv 2505.14059](https://arxiv.org/abs/2505.14059)
Marker | open (GPL) | medium | best OSS for math LaTeX (~10.5/12) | yes | no — Python+Surya | [marker-pdf PyPI](https://pypi.org/project/marker-pdf/)
Surya | open | fast | comparable to Marker; lightweight | yes (low VRAM) | no | [Modal blog 8 OCR](https://modal.com/blog/8-top-open-source-ocr-models-compared)
Docling (IBM) | open (MIT) | fast CPU (0.6-1.3 pg/s 16-core) | 97.9% complex tables | CPU-only viable | no | [arxiv 2501.17887](https://arxiv.org/pdf/2501.17887)
PaddleOCR PP-StructureV3 | open | fast | strong tables + handwriting | yes (low VRAM) | no | [tenorshare guide](https://www.tenorshare.com/ocr/paddleocr.html)
Tesseract | open | fast | weak on layout, OK plain text | CPU-only | yes (tesseract-rs crate) | n/a (legacy)
Nougat (Meta) | open (CC-BY-NC-4.0) | slow | academic LaTeX; UNMAINTAINED | yes | no | [arxiv 2308.13418](https://arxiv.org/pdf/2308.13418)
unstructured.io | open + paid | slow | weak on layout per Modal benchmark | yes | no | [Modal blog](https://modal.com/blog/8-top-open-source-ocr-models-compared)
Mistral OCR API | closed (managed) | very fast | strong, math inconsistent | API | n/a | [Mistral OCR](https://mistral.ai/news/mistral-ocr)
LlamaParse v2 | closed (managed) | fast | tier-picker; LlamaIndex-native | API | n/a | [LlamaIndex insights](https://www.llamaindex.ai/insights/top-document-parsing-apis)
Reducto | closed (managed) | medium | +20% accuracy real-world, provenance | API | n/a | [Reducto comparison](https://llms.reducto.ai/document-parser-comparison)
MarkItDown (MS) | open (MIT) | fast | NO built-in OCR; weak on image PDFs | CPU | no | [adwaitx guide](https://www.adwaitx.com/microsoft-markitdown-document-to-markdown-converter/)
ColPali / ColQwen2.5 / ColSmolVLM | open | retrieval-only | ViDoRe v2: 0.546 / 0.597 / 0.530 nDCG@5 | yes (3B-fp16 ~6GB, 500M for ColSmol) | partial — [Candle](https://github.com/huggingface/candle) inference + [EmbedAnything](https://github.com/StarlightSearch/EmbedAnything) | [arxiv 2407.01449](https://arxiv.org/html/2407.01449v6)
ColModernVBERT | open | retrieval-only | matches 10x larger models | yes | via Candle | [HF/ModernVBERT](https://huggingface.co/ModernVBERT/colmodernvbert)

## Per-document-type recommendation

- **DB schema (SQL DDL)** — text-RAG, chunk-by-CREATE-statement (current Cosmos). No change. Vision adds nothing. ([Cosmos B-42 already implements this])
- **OpenAPI spec (JSON/YAML)** — text-RAG, chunk-per-endpoint with `paths.<route>.<method>` as the chunking boundary. Per [Qdrant Khami talk](https://qdrant.tech/blog/building-search-rag-open-api/) and [arxiv 2411.19804 OpenAPI chunking](https://arxiv.org/pdf/2411.19804), endpoint-level chunks outperform line-window for API retrieval.
- **k8s YAML manifests** — text-RAG, chunk-per-document (`---` boundary) + chunk-per-resource-kind. Optional secondary index: pipe through KubeDiagrams ([arxiv 2505.22879](https://arxiv.org/pdf/2505.22879)) to auto-generate architecture diagram, then dual-index. Saves on retrieval misses where user asks "what services talk to db?" and YAML doesn't surface relationships.
- **ADR (Architecture Decision Record .md)** — text-RAG, semantic-chunk by heading (existing Cosmos line-window OK; upgrade to semantic boundaries per [oneuptime semantic chunking](https://oneuptime.com/blog/post/2026-01-30-semantic-chunking/view)). If ADR contains embedded architecture diagrams (PNG inline), trigger vision pipeline for the image.
- **Domain glossary (.md / .yaml)** — text-RAG, chunk-per-term. No change.
- **Architecture PNG / scanned diagram** — vision-RAG. Run through (a) VLM caption (Qwen2.5-VL or local sidecar) → text chunk; (b) ColModernVBERT / ColQwen2.5 page-image multi-vector. Hybrid (caption-text + image-multi-vec). ([VisRAG ICLR 2025](https://openreview.net/pdf/b13eed287ec8fb31700414fded06f07db85891cd.pdf))
- **Excalidraw (.excalidraw JSON)** — text-RAG primary (the JSON is structured). Secondary: render to PNG via Excalidraw CLI, run VLM caption for natural-language summary. ([excalidraw/mermaid-to-excalidraw](https://github.com/excalidraw/mermaid-to-excalidraw))
- **Mermaid / PlantUML / D2 source** — text-RAG. Diagram source is text. ELAI bonus: convert to PNG render for vision fallback when user asks visual-style questions.
- **Scanned PDF (contracts, whitepapers)** — pre-parse with MinerU2.5 or Dolphin-v2 → Markdown chunks (text-RAG) + ColPali page-image multi-vectors (vision-RAG). Hybrid retrieval, RRF fusion. ([arxiv 2412.07626 OmniDocBench](https://arxiv.org/html/2412.07626v1))
- **Screenshot of web UI / dashboard** — vision-RAG via ColPali; VLM caption stored as alt-text chunk. WebUI dataset ([arxiv 2301.13280](https://arxiv.org/pdf/2301.13280)) and MultiUI 7.3M samples ([arxiv 2410.13824](https://arxiv.org/pdf/2410.13824)) show training-time signal that vision models handle this well.
- **Handwritten / photographed notes** — Dolphin-v2 or PaddleOCR-VL for OCR transcript; store transcript + image multi-vec.

## Best techniques to adopt

Rank | technique | grounding | integration hook
---|---|---|---
1 | Add ColModernVBERT (open, small) or ColQwen2.5-v0.2 as a page-image multi-vector retriever alongside existing BM25+vector text index. Late-interaction MaxSim scoring fuses with text via RRF. | [HF/manu vidore-v2](https://huggingface.co/blog/manu/vidore-v2) ColQwen2.5 0.597 nDCG@5 v2; [HF/ModernVBERT](https://huggingface.co/ModernVBERT/colmodernvbert) matches 10x larger | Cosmos worker: new `ImageArtifactChunker` produces page PNGs + multi-vectors. Stored in per-project collection with blake3 of the source file (same stale-check pattern). Retrieval API gains `mode=vision\|text\|hybrid` flag.
2 | Pre-parse mixed-modality PDFs with PaddleOCR-VL 0.9B (best OmniDocBench v1.5 at 92.56). Output: Markdown + table HTML + figure crops. | [arxiv 2510.14528](https://arxiv.org/abs/2510.14528) | New artifact chunker variant `PdfStructuredChunker` invokes PaddleOCR-VL via local Python sidecar (matches existing remote-sidecar pattern). Markdown → existing text chunker; figure crops → ImageArtifactChunker.
3 | Add 8th GRC method `VisualArtifactCheck`. Designer's plan sweep can verify visual-spatial claims ("auth flow has 3 redirects", "users table is in the middle of the ERD") by retrieving relevant page-images + VLM-verifying. | VisRAG ICLR 2025 end-to-end vision RAG architecture | ArtifactCheck dispatcher in grc-grounding: route by claim-type. Visual claims → vision retriever + VLM verifier (sidecar Qwen2.5-VL); structural claims (existing) stay on text path.
4 | Diagram source files (Mermaid/PlantUML/D2/Excalidraw) get a dedicated `DiagramArtifact` type. Text-RAG primary; auto-render to PNG via [excalidraw/mermaid-to-excalidraw](https://github.com/excalidraw/mermaid-to-excalidraw) for vision fallback. | [dev.to token efficiency analysis](https://dev.to/akari_iku/analyzing-the-best-diagramming-tools-for-the-llm-age-based-on-token-efficiency-5891) — text-first diagrams 24x more token-efficient than JSON | Cosmos artifact type registry: `Diagram { source_lang: Mermaid\|PlantUML\|D2\|Excalidraw, source_text, rendered_png_blake3 }`. Stale-check fires on either source change OR rendered-png change.
5 | Hybrid retrieval pipeline: BM25 + text-vector (existing) + ColPali-image multi-vec, fused via RRF. Per [arxiv 2509.21336 HetaRAG](https://arxiv.org/html/2509.21336v1) and existing Cosmos RRF code, RRF is the right fusion. | M3DocRAG ColPali + Qwen2-VL SOTA on MP-DocVQA; HetaRAG hybrid superior to single-mode | Retrieval API: extend RRF to fuse 3 lists (BM25 / dense-text / colpali-image) per artifact. Per-project policy controls vision-RAG enable flag (cost gate).
6 | Page-level multi-vector pooling for compression: ColPali produces ~1030 vectors/page (heavy). Visual RAG Toolkit ([arxiv 2602.12510](https://arxiv.org/pdf/2602.12510)) shows training-free pooling to ~100 vectors with minimal recall loss. | Visual RAG Toolkit paper | ImageArtifactChunker stores pooled multi-vec (~100 dim-128 vectors per page) instead of full ~1030. 10x storage savings.
7 | Self-hosted vs API: prefer self-hosted for the document-AI critical path (ELAI is a local-first / sidecar tool). Mistral OCR / Reducto / LlamaParse v2 as optional fallbacks via existing `--cloud` flag for cases where self-hosted parse confidence < 0.7. | [arxiv 2511.10297](https://arxiv.org/html/2511.10297v1) local-hybrid-RAG: self-hosting reduces GDPR risk and improves latency | RuntimeAdapter extension (matches B-37 PerRoleRuntime): role `doc_parser` with backends `paddleocr_vl_local`, `mineru_local`, `mistral_ocr_cloud`, `reducto_cloud`, `llamaparse_cloud`.
8 | Confidence scoring on parse output. PaddleOCR-VL exposes per-token confidence; Marker/MinerU expose layout-detection confidence. Route low-confidence pages to vision-RAG fallback. | OmniDocBench reports per-doc-type variance ([arxiv 2412.07626](https://arxiv.org/html/2412.07626v1)); confidence-routed pipelines reduce hallucination | Parser output schema: `{ markdown, page_confidences: [...], fallback_recommended: bool }`. If `fallback_recommended`, force vision-multi-vec indexing as well.

## Anti-patterns

- **Indexing already-structured artifacts as page-images.** Cosmos already chunks SQL DDL by CREATE-statement and YAML line-window — turning these into PNGs to embed via ColPali burns ~10x compute and storage with zero retrieval lift. ([REAL-MM-RAG ACL 2025](https://arxiv.org/abs/2502.12342) shows vision retrievers brittle on tables anyway.)
- **Relying on MarkItDown for image-PDFs.** MarkItDown has no built-in OCR; image-only PDFs come back as empty or noisy text. ([adwaitx MarkItDown guide](https://www.adwaitx.com/microsoft-markitdown-document-to-markdown-converter/))
- **Using Nougat for new pipelines.** Unmaintained; dependency tree is a minefield per 2025 Modal blog. Marker is the maintained successor for academic PDFs. ([Modal blog](https://modal.com/blog/8-top-open-source-ocr-models-compared))
- **Treating vision-RAG as drop-in replacement.** REAL-MM-RAG ([arxiv 2502.12342](https://arxiv.org/abs/2502.12342)) shows current vision retrievers crumble on query-rephrasing and table-heavy docs. Vision-RAG is additive, not substitutive.
- **Embedding ColPali multi-vectors without pooling.** ~1030 vectors/page is storage-prohibitive at corpus scale. Pool via training-free methods per Visual RAG Toolkit.
- **Decompiled or LLM-summarized diagrams as ground truth.** Same rule as ELAI's source-grounding HARD RULE: VLM caption of a PNG is "LOW" reliability. Use as retrieval signal only, not as authoritative source for ArtifactCheck.
- **Hand-crafted PDF chunkers per layout.** OmniDocBench shows even SOTA models drop on reading-order in complex layouts. Spend effort on confidence routing + fallback, not bespoke chunk rules.
- **Mistral OCR for math-dense papers.** Equation fidelity inconsistent per [simonw](https://simonwillison.net/2025/Mar/7/mistral-ocr/) and dev.to evals. Use Marker for math-heavy content.
- **Single-page retrieval on multi-page artifacts.** M3DocRAG paper ([arxiv 2411.04952](https://arxiv.org/abs/2411.04952)) explicitly built because single-page benchmarks miss the actual production failure mode (evidence scattered across pages).

## Open questions

1. **Storage budget gating.** ColPali multi-vec at 100-pooled-vectors-per-page × 128-dim × float16 = ~25KB/page. For a 1000-page artifact, ~25MB per project. Acceptable? Measure: target <500MB per-project Cosmos collection on disk for typical project.
2. **VRAM headroom for VLM caption step.** ELAI sidecar runs Qwen2.5-Coder-3B at ~1.92GB; adding Qwen2.5-VL-3B for caption would push to ~3.8GB. Fits 12GB tier but blocks running both simultaneously. Measure: per-token latency of caption pass; threshold at which batch-once-at-ingest beats on-the-fly.
3. **Vision-RAG retrieval recall@5 on ELAI's actual artifact corpus.** ViDoRe v2 = 0.597 for open-weight; ELAI corpus is smaller and more technical-skewed. Build mini-benchmark: 50 hand-labeled query→page pairs on a real project's Cosmos collection.
4. **Confidence threshold for vision-fallback routing.** What page-confidence triggers vision-multi-vec indexing? Tune on OmniDocBench page-type breakdown — likely <0.85 confidence triggers fallback for non-academic docs.
5. **Open-license vs closed-license retrieval ceiling.** ViDoRe v2 closed (Qwen2.5-VL fine-tunes) ~0.90+ v1; open ceiling ~0.60 v2. Is the gap worth a `--cloud doc-rerank` path that calls Cohere Embed v4 / Voyage multimodal-3? Cost-benefit measure: latency × per-query $ × accuracy lift.
