# ELAI phase-index MASTER — exhaustive feature discovery

## Source coverage

- **Primary feature source:** `docs/specs/guide/steps/phase-index-MASTER.txt`, read end-to-end, lines 1–1056. The file says it was generated 2026-05-17 from end-to-end reads of every then-current phase file and is a context index of goals, key deliverables/steps, dependencies, and grounding anchors; it explicitly delegates full `PRE_GATE`/`TASK`/`DETERMINISTIC_CHECK`/`ADVERSARIAL`/`ACCEPTANCE` blocks to the individual phase files (L1–4).
- **Orientation only:** `docs/thefinalglance/ELAI-SPEC-OVERVIEW.md`, read end-to-end (592 lines). No feature is asserted below solely from the overview; it was used only to understand system vocabulary and topology.
- **Filesystem manifest check:** filenames in `docs/specs/guide/steps/` were enumerated without reading their contents. That directory now has 46 entries and contains several phase-detail files absent from the MASTER's L1039–1055 inventory; these are called out in the deeper-source manifest.
- **Status legend:** **[E]** explicit requirement/capability in the MASTER; **[C]** explicit completion/current-resolution statement; **[D]** explicitly deferred/dropped/conditional; **[I]** implication needed to make explicit entries compose but not directly specified; **[Q]** ambiguity, inconsistency, or deeper-read question. Line anchors (`Lx–y`) refer to the MASTER.
- **Scope caution:** the MASTER describes the design at index granularity, not actual shipped/wired state. Completion annotations below reproduce only completion language present in the MASTER.

## Exhaustive atomic feature inventory

### PRE-PHASE 0 — Windows platform fixes (`L10–22`)

- **[E] P0 goal:** make `claw-code-rust-port` compile and pass all 684 tests on Windows without feature changes (`L10–11`).
- **[E] P0.1:** fix ten compile errors by `cfg`-gating `std::os::unix::PermissionsExt` and `set_mode` in `runtime/mcp_stdio.rs`, `runtime/mcp_tool_bridge.rs`, and `rusty-claude-cli/tests/mock_parity_harness.rs` (`L13–14`).
- **[E] P0.2:** make four plugin-hook failures Windows-compatible by translating `.sh` to `.cmd` for `write_hook_plugin` and making `HookRunner::run_hook_command` dispatch through the platform shell (`L15–16`).
- **[E] P0.3:** fix four tool tests through path-separator normalization, `python3` → `python` on Windows, and poisoned environment-mutex recovery (`L17–18`).
- **[E] P0.4:** remove two unused-import/unused-`mut` warnings (`L19`).
- **[E] P0.5:** persist a clean baseline to `.elai_cc/metrics/pre_phase_0.toml` (`L20`).
- **[E] Exit gate:** `cargo check` must have zero errors and zero warnings and `cargo test` must have zero failures on Windows (`L21`).
- **[E] Recorded baseline:** 69 Rust files, 61,317 LOC, 684 test annotations, and a 26.24-second `cargo check` (`L22`).

### PHASE 0 — fork, strip, restructure, walking skeleton (`L24–51`)

- **[E] Product reduction:** strip approximately 4K+ LOC of known-bad code and restructure the 25K-LOC/38-file `runtime/` into a nine-crate workspace under `.elai_cc/crates/` (`L24–27`).
- **[E] Walking skeleton:** one Ollama model, one `IDLE → EXECUTING → IDLE` FSM loop, one `file_read` tool, workspace root `.elai_cc/Cargo.toml` (`L25–27`).
- **[E] 0.1 migration ledger:** `migration_manifest.toml` must classify every `.rs` file as `KEEP`, `STRIP`, `REPLACE`, or `EXTEND` (`L29`).
- **[E] 0.2 strip set:** remove `telemetry/`, `compat-harness/`, and runtime modules `usage`, `compact`, `summary_compression`, `lsp_client`, `oauth`, and `prompt`; replace stripped token accounting with a structural `TokenUsage` shim while refactoring 61 `Session` occurrences; audit 28+ slash commands into keep/strip sets (`L30–32`).
- **[E] 0.3 strip verification:** classify every failing test as `EXPECTED` or `BUG`, fix all `BUG`s, and record the strip delta (`L33`).
- **[E] 0.4a leaf crates:** create `core/`, `rules/`, and `providers/`; providers absorbs `api/` plus `runtime/sse.rs` (`L34`).
- **[E] 0.4b middle layer:** create `tools/` and `plugins/` plus empty `knowledge/` and `reasoning/` skeletons (`L35`).
- **[E] 0.4c top layer:** create `orchestrator/` from the remaining runtime and `cli/` from `rusty-claude-cli` plus `commands/`; archive `.elai_cc/rust/`; keep `main.rs` below 500 LOC as a thin REPL (`L36–37`).
- **[E] 0.5 restructure verification:** run `cargo test --workspace` and classify/fix `RESTRUCTURE_BUG` and `PATH_UPDATE` failures (`L38`).
- **[E] 0.6 contract TDD:** create `crates/contracts/` with five deliberately failing contract tests as the red phase (`L39`).
- **[E] 0.7a provider/config:** define a provider trait and `OllamaProvider`; configure through `elai_cc.toml`; support TOML and MessagePack (`rmp-serde`) serialization (`L40`).
- **[E] 0.7b runtime base:** implement `IDLE`/`EXECUTING` transition validation, a visibility-filtered `file_read`, and MessagePack session storage (`L41–42`).
- **[E] 0.7c CLI/runtime UX:** commands `/task`, `/config`, `/continue`, `/explore`, `/status`; visibility policy with `allowed_paths`, `hidden_paths`, and allowlist sanitization; `tracing`-based logging (`L43–44`).
- **[E] 0.8 implementation gate:** all five contract tests pass (`L45`).
- **[E] 0.9 entropy quarantine:** `ClockProvider` and `EntropySource` traits live in `core/`; production uses `OsClock`/`OsRng`; tests use `SeededClock`/`SeededRng`; replace 60 direct time/entropy uses across 31 files; `RuntimeContext` bundles both providers; a CI grep enforces the boundary (`L46–48`).
- **[E] 0.10 determinism proof:** with `ELAI_SEED=42` and `--test-threads=1`, two runs must have a zero diff (`L49`).
- **[E] Phase output:** exactly the named nine crates—`core`, `providers`, `knowledge`, `reasoning`, `orchestrator`, `tools`, `plugins`, `rules`, `cli`—plus the contract-test crate and no raw time/entropy outside core implementations (`L50–51`).

### PHASE 1A — DKG / tree-sitter-to-SQLite graph (`L57–76`)

- **[E] Goal/layer:** parse source into a dependency knowledge graph in SQLite as knowledge-pipeline Layer 1, initially Rust with Python/TypeScript/Go later, in `.elai_cc/crates/knowledge/` (`L57–60`).
- **[E] Dependencies:** `tree-sitter 0.26`, `tree-sitter-rust 0.24`, `rusqlite 0.39` with bundled SQLite, `blake2`, and development-only `proptest` (`L59–60`).
- **[E] 1A.1 interfaces:** object-safe `KnowledgeEngine` (`build_graph`, `query`, `rerank`, `update`, `readiness_check`) and `LanguageGrammar` with `&self` methods (`L62–63`).
- **[E] 1A.1 graph types:** `SymbolKind`; `DkgNodeInfo {node_id, symbol_name, file_path, signature, kind, start_line, end_line}`; `DkgEngine::new(db_path)` (`L63–65`).
- **[E] 1A.1 schema:** `dkg_nodes` including `concept`, `concept_stale`, `file_hash`; `dkg_edges` typed `imports`/`contains`/`implements` with `ON DELETE CASCADE`; `dkg_files` including `parse_status`; `dkg_meta`; indexes on name/file/kind; WAL mode; five-field `index_version.toml` (`L65–67`).
- **[E] 1A.1 read API:** nodes, edges, outbound edges, inbound edges, failed files, and `dkg_readiness` (`L68`).
- **[E] 1A.2 build gate:** workspace compiles with no regressions (`L69`).
- **[E] 1A.3 tests:** at least 20 tests for Rust parsing, populated signatures, directions/types of edges (`impl→method=contains`, `use_decl→target=imports`, `impl Trait=implements`), parse-failure tracking, readiness states, project isolation, and property tests (`L70–72`).
- **[E] 1A.4 implementation:** reconstruct Rust signatures from AST; dispatch grammars through `HashMap<&str, Box<dyn LanguageGrammar>>`; create schema and WAL pragma; use Blake2s content hashes for incremental update with cascade cleanup; log and skip parse failures; readiness moves `Unavailable → Ready|Empty` (`L73–75`).
- **[E] 1A.5 measurements:** record build time, query p50/p99, DB size, node count, and edge count in `phase_1A.toml` (`L76`).

### PHASE 1B — BM25 / Tantivy full-text retrieval (`L78–93`)

- **[E] Goal/SLO:** a Tantivy index per project; query p99 below 10 ms, `update_file` below 5 seconds, zero orphan text files, and binary-file exclusion when a null byte occurs in the first 8 KiB (`L78–80`).
- **[E] Storage:** `.elai_cc/projects/{id}/bm25/` per project (`L80`).
- **[E] Schema invariant:** `path` is `STRING|STORED` and untokenized; `content` is `TEXT|STORED`; `line_start`/`line_end` are `u64`; `chunk_kind` is `STRING`; using `TEXT` for path is forbidden because it breaks `delete_term` (`L81–82`).
- **[E] 1B.1 compatibility pin:** pin exact Tantivy version and record `INDEX_FORMAT_VERSION` (`L84`).
- **[E] 1B.2 API:** distinct `BM25Chunk` (not `DkgNode`) and `Bm25Index::{index_file,index_directory,query,delete_file,update_file}`, with atomic update (`L85–86`).
- **[E] 1B.3 tests:** at least ten tests for index/query/delete/incremental behavior, project isolation, and a path-field precision regression using `src/deep/nested/module.rs` (`L87–88`).
- **[E] 1B.4 fallback chunking:** when DKG has no nodes, use 50-line sliding windows with ten-line overlap; overlap is explicitly ungrounded for BM25 and retained only for vector-layer alignment (`L89–90`).
- **[E] 1B.4 chunk/update semantics:** `ChunkKind::{Function,LineBlock,File}` and atomic delete+insert in a single `writer.commit()` (`L91`).
- **[E] 1B.5 benchmark:** run on 69 files from `.elai_cc/rust/`; record p50/p99, `orphan_files=0`, Tantivy version, and index format version in `phase_1B.toml`; latency gate is <10 ms dev and <50 ms CI (`L92–93`).

### PHASE 1C — dual-model ONNX vector search (`L95–120`)

- **[E] Semantic models:** GraphCodeBERT for code (768 dimensions) and Granite for text (384 dimensions), both ONNX (`L95–97`).
- **[E] Vector cache:** SQLite cache keyed by Blake2s content hash (`L96–97`).
- **[E] Resilience:** per-model circuit breakers; startup must embed `"fn main()"` within five seconds; canonical model path `.elai_cc/models/onnx/`; a missing model degrades to `BM25_ONLY` rather than crashing (`L97–99`).
- **[E] Dependency rule:** use pinned `ort`, `usearch 2.24`, `fastembed 5.12`, and `blake2`/`Blake2s256`, but do not mix `fastembed` with `ort`; SPLADE waits for 1H (`L100–103`).
- **[E] 1C.2 modules:** `vector/{mod,cache,ort_session,circuit_breaker,hnsw_index}.rs` (`L104`).
- **[E] 1C.2 ONNX session:** `Arc<Mutex<ort::Session>>`, sequence length 512, truncate and warn on overflow (`L105`).
- **[E] 1C.2 ANN index:** `UsearchIndex` with cosine metric, save/load, and project-isolated filtered search (`L106`).
- **[E] 1C.2 breaker:** open after three failures in 60 seconds, half-open after 30 seconds, and use injected `ClockProvider` rather than `std::time` (`L107`).
- **[E] 1C.3 tests:** at least 22 covering cache lookup/invalidation, breaker open/half-open, degradation cascade, validation of empty/oversized/control-character queries, canonical path, missing-model degradation, semaphore overflow, cache-hit embed avoidance, truncation warnings, HNSW persistence, three-ranker RRF with `k=60`, and concurrent embedding mutex behavior (`L108–111`).
- **[E] 1C.3.5 tensor gate:** verify GraphCodeBERT/Granite tensor shapes from model cards or `ort` metadata and record the decision in `workflow/reasoning/phase-1C-decisions.md` (`L112–113`).
- **[E] 1C.4 implementation:** embedding; SSB-single-writer vector-cache writes; clocked breaker; HNSW persistence with cache rebuild when load fails; three-ranker RRF `k=60`; SSB `file_write` causes cache deletion; mode changes emit `action_type='knowledge_degradation'`; readiness canary doubles as warmup (`L114–117`).
- **[E] 1C.5 measurement:** embedding throughput, cache-hit rate ≥90%, and dual-vector recall versus BM25 (`L118`).
- **[D] TurboQuant:** grounded as approximately 10.7× compression at `b=3` for 768d/384d and ~0.001 s indexing, but deferred to 1G or later; Phase 1C stores raw `f32` (`L119–120`).

### PHASE 1D — PageRank repository map (`L122–141`)

- **[E] Goal:** rank symbols by importance and fit repo maps to a context token budget (`L122–123`).
- **[E] Aider derivation/divergence:** derived from `aider/repomap.py`; use symbol-level rather than file-level nodes, a 0.3 density guard, and insertion-order fallback (`L124–125`).
- **[E] Adopted algorithms:** redistribute dangling mass through personalization; orphan self-edges weight 0.1; edge weights scale with square root of reference count; binary-search initial guess `max_map_tokens/25`; truncate lines to 100 characters before token estimation (`L126–128`).
- **[E] 1D.1 snapshot/performance design:** `DkgSnapshot::from_engine()` provides an in-memory DKG mirror to avoid SQLite queries per iteration (`L130`).
- **[E] 1D.1 configuration:** `PageRankConfig {damping=0.85,max_iter=100,tol=1e-6,density_threshold=0.3,max_map_tokens=2048}` (`L131–132`).
- **[E] 1D.1 output/API:** `RepoMap {ranked_symbols,token_count,density,used_pagerank,fit_iterations}` and `compute_repo_map(snapshot,task,active_files,config,token_estimator)` (`L132–133`).
- **[E] 1D.2–1D.3 gates:** workspace compiles; tests T1–T18, properties P1–P3, adversarial A1–A3 (at least 21), with `make_snapshot` helper (`L134–135`).
- **[E] 1D.4 scoring:** adjacency matrix with square-root scaling/self-edges; 10× personalization for task-mentioned nodes normalized to sum 1; 50× edge boost for active files (`L136–138`).
- **[E] 1D.4 safety/fallback:** density >0.3 disables PageRank and uses insertion order; power iteration clamps NaN/Inf and redistributes dangling mass (`L138–140`).
- **[E] 1D.4 special files/fit:** include a 13-item Rust special-file list such as `Cargo.toml`, `README.md`, `build.rs`; binary-search fitting is capped at ten iterations (`L139–140`).
- **[E] 1D.5:** persist metrics in `phase_1D.toml` (`L141`).

### PHASE 1E — CatRAG + A-RAG assembled query pipeline (`L143–166`)

- **[E] End-to-end route:** `query → BM25 → vector → PageRank → CatRAG → A-RAG → snippets`; target 55–87% token savings against naive full-chunk retrieval and latency under two seconds (`L143–146`).
- **[E] Adaptation:** CatRAG/A-RAG paper entity nodes map to `DkgNode`, passage nodes to file chunks; tree-sitter replaces OpenIE triples; regex/fuzzy DKG-name matching replaces NER (`L146–148`).
- **[E] 1E.1 CatRAG config:** `damping=0.5`, `epsilon=0.2`, `beta=2.5`, `n_seed=5`, `k_edge=15`, `min_graph_density=0.01`; overlays are sparse deltas and do not copy full matrices (`L150–151`).
- **[E] 1E.1 A-RAG config:** `max_loops=15`, `max_token_budget=128000`, `top_k=5`; `RetrievalTool::{KeywordSearch,SemanticSearch,ChunkRead}`; `ContextTracker` deduplicates (`L152–153`).
- **[E] 1E.1 tracing:** `QueryPipeline` is stage-based and every `PipelineStage` carries `latency_ms` (`L154`).
- **[E] 1E.2–1E.3 gates:** compile verification plus five CatRAG tests (anchoring, weighting, enhancement, density guard, `phi()` tiers), five A-RAG tests, three pipeline tests, and two contract tests (`L155–157`).
- **[E] 1E.4 CatRAG behavior:** extract CamelCase/snake_case symbols, fuzzy-match DKG names, score edges deterministically (`call=8`, `implements=7`, `references=5`, `imports=3`) through `phi()` tiers with no hot-path LLM, and boost verified context edges by `1+beta` (`L158–160`).
- **[E] 1E.5 keyword search:** wrap BM25 and return sentence-level snippets with ±2 lines (`L161`).
- **[E] 1E.5 semantic search:** oversample by 3× and aggregate candidates per file using maximum score (`L162`).
- **[E] 1E.5 chunk reads:** repeated content becomes a `"(previously read)"` placeholder through `ContextTracker` (`L162–163`).
- **[E] 1E.6 composition:** stages are `bm25→vector→pagerank→catrag→arag`, with rerank (1G) and token-fit (3E) stubs; BM25/vector fuse by `1/(60+rank)`; density guard passes through (`L164–165`).
- **[E] 1E.7 acceptance:** five representative queries, mean token savings >50%, p50 pipeline latency <2000 ms (`L166`).

### PHASE 1F — speculative decoding evaluation (`L168–181`)

- **[E] Decision feature:** an `ADOPT`/`DROP` gate for speculative decoding, dependent on 1C `ModelPool` VRAM accounting (`L168–169`).
- **[C] Resolution:** Ollama 0.20.2 lacked speculative decoding; 1F.3–1F.5 were skipped; Phase 1 outcome is `DROP`; revisit under the 3J vLLM provider in Phase 2+; config skeleton and provider probe are done; prompt-lookup decoding was also dropped (`L170–172`).
- **[E] 1F.1 dormant config:** `OllamaConfig.speculative_decoding`, `draft_model: Option<String>`, and `num_draft_tokens: u8` (`L173–175`).
- **[E] 1F.2 capability probe:** resolve `SUPPORTED` versus `NOT_SUPPORTED` for `draft_model` (`L175`).
- **[D] Original evaluation:** ten temperature-zero prompts with soft-text equivalence; prompt-lookup zero-VRAM baseline; throughput sweep `k={1,2,3,4,5,7}` and early kill below 0.5 acceptance; VRAM measurement; adopt only above 1.3× throughput and below 3 GB extra VRAM (`L176–179`).
- **[E] Hardware correction:** RTX 3060 12 GB has 4.8 GB headroom after Gemma 4 E2B (~7.2 GB Q4_K_M), not the prior ~10 GB claim in `providers.txt` (`L180–181`).

### PHASE 1G — cross-encoder reranker (`L183–206`)

- **[E] Model/SLO:** post-A-RAG `ms-marco-MiniLM-L12-v2` (33.4M parameters, ~120 MB ONNX fp32/~30 MB int8), CPU-only, target <50 ms for 50 candidates (`L183–186`).
- **[E] Skip/concurrency:** skip at candidate count ≤10 by default; `tokio::Semaphore(1)`, bounded MPSC queue depth 4, request batching (`L185–186`).
- **[E] Runtime decisions:** raw `ort` + `tokenizers`, not `fastembed-rs`; begin fp32; consider int8 only if baseline p50 >30 ms and NDCG@10 delta <0.5; MPSC drain batching with per-caller oneshot result splitting (`L187–189`).
- **[E] 1G.1 config:** max sequence 512, queue 4, one intra-op thread, `ORT_ENABLE_ALL`, deterministic compute enabled (`L190–193`).
- **[E] 1G.2 load/tokenize/fallback:** deterministic optimized ORT session, BERT `tokenizer.json` truncation at 512, lazy L-6 fallback loaded only after L-12 breaker opens (`L194–196`).
- **[E] 1G.3:** pair scoring, reranking, semaphore-protected MPSC batch draining, offset-tracked split, oneshot return per caller (`L197–198`).
- **[E] 1G.4 resilience:** three failures/60 s, 30 s half-open, injected clock; lazy L-6 fallback; SSB `knowledge_degradation`; `StageTrace 'rerank'`; final fallback skips reranking (`L199–201`).
- **[E] 1G.5 readiness:** startup canary performs embedding, BM25, and rerank inside five seconds, using BM25 candidates (`L202`).
- **[E] 1G.6 optimization harness:** thread sweep 1/2/4/8 with determinism/rank-order stability; conditional int8; skip thresholds 5/10/15/20/30 selected by NDCG@10; committed 50-candidate fixture `tests/fixtures/reranker_benchmark.json`; quantization script `scripts/quantize_cross_encoder.py` (`L203–206`).

### PHASE 1H — conditional SPLADE keyword expansion (`L208–225`)

- **[E] Experiment:** feature-gated `splade` evaluation comparing `prithivida/Splade_PP_en_v1` with `naver/splade-v3` (`L208–210`).
- **[E] Decision gates:** drop if recall@10 improvement over BM25 is <5% or Wilcoxon `p ≥ 0.05`; early-drop before recall if programming-identifier vocabulary coverage <0.3 because BERT WordPiece has 30,522 non-code-specific tokens (`L210–212`).
- **[E] 1H.1 build isolation:** `splade.rs` behind `cfg(feature="splade")`; `ort` remains unconditional for GraphCodeBERT/Granite; only SPLADE session construction is gated (`L214–215`).
- **[E] 1H.2 tests:** finite/nonnegative sparse expansion, empty/long query behavior, RRF changes versus BM25 alone, and recall integration (`L216–217`).
- **[E] 1H.3 variants:** `PrePooled` for PP_en_v1; `RawMLM` for splade-v3 with max-pool + ReLU + `log1p` in Rust; tree-sitter identifier extraction for `vocab_coverage()`; weighted RRF fusion (`L218–220`).
- **[E] 1H.4 measurement:** vocabulary fitness, model selection (Jaccard only for quantization regression and recall@10 for A/B model comparison), 50-query Wilcoxon test, and keep only when mean delta ≥5% and `p<0.05` (`L221–223`).
- **[D] Early-drop completion:** phase can complete without recall measurement; feature remains disabled by default while module is preserved for later production-data reevaluation (`L224–225`).

### PHASE 1I — retrieval resilience/readiness/tracing (`L227–248`)

- **[E] Dependency:** requires 1E and 1G; add the missing `1G→1I` dependency edge before implementation (`L227–228`).
- **[E] 1I.1 breaker classification:** transient errors trip after three failures; permanent errors trip after one; exponential backoff 30/60/120 seconds capped, with deterministic jitter from `model_id`; half-open needs two consecutive successes (`L230–233`).
- **[E] 1I.1 safe recovery:** a half-open probe creates a fresh ONNX session and never reuses a possibly corrupt one; clock injected; breaker state is not checkpointed and restarts closed (`L232–234`).
- **[E] 1I.2 modes:** `SearchMode::{HYBRID,SPARSE_ONLY,BM25_ONLY}` (`L235`).
- **[E] 1I.2 cascade:** GraphCodeBERT failure → text-only; Granite failure → code-only; both → `SPARSE_ONLY`; SPLADE failure → `BM25_ONLY`; reranker failure → no reranking; mode restores after a successful half-open probe; every change journals `knowledge_degradation` (`L235–238`).
- **[E] 1I.3 validation:** under 1 ms; reject empty queries, >10K characters, null bytes, and control characters except newline (`L239`).
- **[E] 1I.4 readiness:** load ONNX, open SQLite, run embed+BM25+rerank canary within five seconds; `READY`, `DEGRADED` (which invariantly means `BM25_ONLY`), or `UNAVAILABLE` (blocks all FSM transitions) (`L240–242`).
- **[E] 1I.4 drift guard:** cosine similarity between current and reference embedding for `"fn main()"` below 0.85 forces `DEGRADED`; persist reference vector in `index_version.toml` (`L242–243`).
- **[E] 1I.5 overload behavior:** one concurrent cross-encoder, queue four; sixth concurrent request is returned unranked; compatible requests batch (`L244`).
- **[E] 1I.6 observability:** debug `QueryTrace`/`StageTrace` for nine stages—`bm25`, `splade`, `code_vector`, `text_vector`, `pagerank`, `catrag`, `arag`, `rerank`, `token_fit`; aggregate p50/p99 to SSB only at session end, not per query (`L245–246`).
- **[E] 1I.7 migration:** index-version mismatch rebuilds and archives old index to `index_backup_{version}/`; missing version file defaults to v1 (`L247–248`).

### PHASE 3K — RAG production hardening (logically in knowledge pipeline) (`L250–271`)

- **[E] Dependency:** requires 1E, 1G, and 3J (`L250–251`).
- **[E] 3K.1 adaptive chunking:** replace fixed 50-line BM25 windows with DKG/tree-sitter AST splitting; Rust boundaries `fn`/`impl`/`struct`/`enum`/`mod`, Python/TS/Go `def`/`class`; target 200–800 tokens with 50-token overlap; split Markdown by headings (`L253–255`).
- **[E] 3K.2 deterministic multi-query:** no LLM; keyword extraction, CamelCase/snake_case variants, synonyms such as error→failure/exception; merge and deduplicate by `chunk_id` (`L256–257`).
- **[E] 3K.3 HyDE:** sidecar creates a 128-token hypothetical answer at temperature 0.7, embeds it, searches vectors, then reranks against the original query; cache five minutes; sidecar outage degrades gracefully (`L258–260`).
- **[E] 3K.4 intent tiering:** `FAST`/`STANDARD`/`DEEP`, auto-detected from query length and keywords; distinct from 1I's fault-degradation `SearchMode`; `AUTO` remains backward compatible (`L261–262`).
- **[E] 3K.5 quality bench:** 30 queries (10 simple/10 code/10 complex), measuring Recall@5, Recall@10, MRR, and per-tier latency in `phase_3K_rag.toml` (`L263–264`).
- **[E] 3K.6:** A-RAG keyword score `Σ count(k,text)·len(k)`, 3× semantic oversampling, progressive disclosure through snippets, goal ≥50% token savings (`L265–267`).
- **[E] 3K.7:** CatRAG symbolic anchoring with `epsilon=0.2` weak DKG-entity seeds and `beta=2.5` verified-edge boost (`L268–269`).
- **[D] 3K.7 Innovation 2:** LLM edge scoring deferred because it is too expensive for code search (`L269–270`).
- **[E] 3K.8 acceptance:** compare against 3K.5 baseline and allow no metric regression above 5% (`L271`).

### PHASE 2A — Grounded Reasoning Chain state machine (`L277–299`)

- **[E] Goal:** core GRC state machine, reasoning budget, and grounding-format validation (`L277–278`).
- **[E] State type:** `GrcState::{Observe,Hypothesize,PlanAction,Act,Verify,Converge,Status}` with `allowed_next()` (`L279–280`).
- **[E] Step/result types:** `ThinkStep {step_type,content,topic,grounding: Option<String>,conclusion,scope: Option<Vec<String>>}`; `ThinkResult {step,budget_remaining,grounding_level,warnings}` (`L280–282`).
- **[E] Budget:** `GrcBudget {max_calls,max_chars,calls_used,chars_used}` and forced convergence when remaining calls ≤1 (`L282`).
- **[E] Grounding/error taxonomy:** ordered `GroundingLevel::{Uncertain,High,Verified}` and `GrcError::{UngroundedStep,BudgetExhausted,InvalidTransition,MalformedGrounding}` (`L283–284`).
- **[E] Stable trait:** `GrcEngine::{think,set_budget,get_budget,reset_topic}` implemented now; `verify_claim` and `ground_plan` declared as `unimplemented!()` stubs for 2B so the trait shape remains stable (`L285–286`).
- **[E] 2A.1 minimal crate:** dependencies only on core, `thiserror`, and `serde`; types in `types.rs`; `BasicGrcEngine` in `engine.rs`; per-topic state in `HashMap<String,TopicState>` (`L288–289`).
- **[E] 2A.2 dependency gate:** no providers, tools, or knowledge dependencies (`L290`).
- **[E] 2A.3 tests:** 12 covering grounding-required observe, forced convergence, per-topic isolation, invalid transitions, convergence threshold, malformed grounding, free `Status`, default budget, first-step observe, and scope present/missing/optional (`L291–294`).
- **[E] 2A.4 transition rules:** default `Observe→Hypothesize→PlanAction→Act→Verify`; `Status` and `Converge` always permitted; first step must be `Observe` (`L295–296`).
- **[E] 2A.4 grounding parser:** parse `type:reference`; `test`/`search` → `Verified`, `file`/`grep`/`tool` → `High`, `training` → `Uncertain` (`L296–298`).
- **[E] 2A.4 budget/state behavior:** check before increment; use `saturating_add`; topics isolated; missing scope enumeration warns (`L298`).
- **[E] 2A.5 SLO:** validation below 5000 µs and memory below 1 KiB per topic (`L299`).

### PHASE 2B — grounding verification (`L301–330`)

- **[E] Verification methods:** six named binary MiniCheck-style methods—`FILE_CHECK`, `SOURCE_CHECK`, `TEST_CHECK`, `WEB_CHECK`, `API_CHECK`, `API_REGISTRY_CHECK` (`L301–304`).
- **[E] Plan gate:** a grounding sweep blocks `PLANNING→EXECUTING` if any claim is `UNGROUNDED` and resolves 2A uncertainty where possible (`L303–304`).
- **[E] Design decisions:** binary verdicts; structural-regex claim decomposition rather than NLI; enum-based method dispatch; `verified_at` from a `TimeSource`; add `tools/` to reasoning; breaking `ground_plan` result becomes `PlanGroundingResult`; `verify_claim` gains context; injectable `TestRunner` with production `CargoTestRunner` and test `MockTestRunner` (`L305–309`).
- **[E] 2B.1 method metadata:** `VerificationMethod` has six variants plus `requires_network`/`requires_tools`; typed method stored in `GroundingResult` and `VerificationResult`; `GroundingResult.verified_at: u64` (`L311–313`).
- **[E] 2B.1 plan result:** `PlanGroundingResult {claims,all_grounded,ungrounded,verify_needed}` (`L313–315`).
- **[E] 2B.1 verifier abstractions:** object-safe `Verifier`, `TestRunner`, `LocalVerifier` and `NetworkVerifier` stubs (`L314–315`).
- **[E] 2B.2 dependency gate:** prevent tools→reasoning circularity and transitive knowledge inclusion (`L316`).
- **[E] 2B.3 tests:** 16 for existing/missing files and source symbols, mock test execution, typed network-method errors without test networking, plan block/pass/empty-pass, uncertain auto-resolution, object safety, comment-vs-real grounding adversary, and stripping the `file:` prefix (`L317–321`).
- **[E] 2B.4 local verifier:** file check delegates to `tools::read_file`; source check reads then matches `(fn|struct|enum|trait|impl|const|static|type|mod)\s+{name}\b` (`L322–324`).
- **[E] 2B.4 test verifier:** `CargoTestRunner` runs through `BashSandbox::execute` with 30-second timeout (`L324–325`).
- **[D] 2B.4 network verifier:** returns typed `VerificationError("not impl until Phase 4A IPC proxy")` (`L325`).
- **[E] 2B.4 claim/plan processing:** strip grounding prefix before verifier dispatch; extract `type:reference` claims by regex; classify; attempt FILE/SOURCE auto-resolution for training-tagged claims; return aggregate plan result (`L326–328`).
- **[E] 2B.5 SLO/status:** file check <2000 µs, source check <10000 µs for 1000 lines, 20-claim sweep <100000 µs; only 3/6 methods implemented and three stubbed at this phase (`L329–330`).

### PHASE 2C — per-agent GRC configuration (`L332–362`)

- **[E] Role specialization:** six roles get independent budgets, step sequences, grounding constraints, and allowed verification methods (`L332–334`).
- **[E] Config type:** `AgentGrcConfig` fields `role`, `budget`, `required_steps`, `grounding_level`, `csq_required`, `allowed_methods`, `transition_overrides`, `require_scope_enumeration` (`L334–336`).
- **[E] Transition invariant:** per-role overrides are additive and can never remove a default transition (`L335–336`).
- **[E] Role defaults:** executor 8 calls/1200 chars and full chain; auditor 10/1200 with double-verify and anti-rubber-stamp; context compiler observe-only and only FILE/SOURCE checks; memory writer 2/400; prompt engineer 5/800; designer 8/1200 and distinct from executor because it emits `PipelineSpec` (`L337–340`).
- **[E] TOML layering:** `[grc.defaults]` plus `[agents.{role}.grc]`; role keys use underscores (for example `context_compiler`), not hyphens (`L341–342`).
- **[E] Auditor anti-rubber-stamp:** on auditor `Converge`, history must contain at least one `Verify` whose content begins `FINDING:`; otherwise `InsufficientFindings` (`L343–344`).
- **[E] Grounding order invariant:** enum order must be `Uncertain, High, Verified` or explicit comparison must enforce `Verified > Uncertain` (`L345–346`).
- **[E] 2C.1 interfaces/errors:** `GrcError::{InsufficientFindings,ConfigValidation}`; `AgentGrcConfigLoader::{load_config,validate_config}`; `GrcDefaults`; `TomlConfigLoader` stub; engine `with_agent_config`/`get_agent_config` (`L348–350`).
- **[E] 2C.2 dependency gate:** no new external dependency; core parses TOML and reasoning accepts typed data (`L351`).
- **[E] 2C.3 tests:** 15 for role defaults, double verification, observe-only mode, config rejection, rubber-stamp rejection, uncertainty rejection, method allowlist, layered merge, additive transitions, distinct designer, and per-role scope requirement (`L352–356`).
- **[E] 2C.4 implementation:** five config-validation rules; effective next states are `(base ∪ overrides) ∩ (required_steps ∪ {Status,Converge})`; enforce minimum grounding; anti-rubber-stamp; merge defaults/overrides; assigning agent config resets topic state (`L357–359`).
- **[E] Authority transition:** 2C budget enforcement is authoritative in-process through Phases 0–3; at 4A agent-side enforcement becomes advisory and orchestrator enforcement authoritative (`L360–361`).
- **[E] 2C.5 SLO:** config load <1000 µs and routing overhead <10% (`L362`).

### PHASE 2D — counterfactual safety questions and inline feasibility (`L364–385`)

- **[E] CSQ trigger:** generate counterfactual safety questions for functions touching files, network, money, or authentication; allow at most two inline feasibility calls during `EXECUTING` (`L364–366`).
- **[E] CSQ types:** `CsqSeverity::{Critical,Warning}`; `CsqCheck {assumption,consequence,severity,guard_added}`; `CsqResult {function_id,checks}` where pass means every critical check gained a guard (`L367–368`).
- **[E] Feasibility types:** `FeasibilityMethod::SourceMatch` now, with `WebSearch`/`Grep`/`ApiRegistryCheck` added in 4F; `InlineFeasibilityResult {claim,method,outcome: Pass|Fail|Blocked}` (`L368–370`).
- **[E] Feasibility state:** `InlineFeasibilityState {checks_used,max_checks=2}` with `is_exhausted`, `record_check`, `reset` (`L370–371`).
- **[E] Trait surface:** `generate_csq(function_id,sensitive_ops)` and `check_inline_feasibility(claim,sources)` (`L372`).
- **[E] Critical patterns:** case-insensitive `fs::`, `file`, `write`, `delete`, `remove`, `net::`, `tcpstream`, `money`, `payment`, `auth`, `token`, `credential`, `password`; everything else warns (`L373–375`).
- **[E] 2D.1–2D.3:** add seven types, two trait methods, feasibility state; no dependency changes; ten tests for all outcomes, two-call budget, reset, and `CONTRADICTS_PLAN:` blocking marker (`L376–380`).
- **[E] 2D.4 implementation:** lowercase-pattern classification; source substring matching; `CONTRADICTS_PLAN:` → `Blocked`, signaling a `MAJOR` `ReplanRequest`; record count with saturating addition (`L381–384`).
- **[D] Orchestrator emission:** actual `ReplanRequest` emission waits for 3A+ (`L383`).
- **[E] 2D.5 SLO:** CSQ <1000 µs, inline check <2000 µs, false-positive rate exactly zero (`L385`).

### PHASE 3A — full interactive FSM (`L391–414`)

- **[E] Nine states:** `IDLE`, `PLANNING`, `FEASIBILITY_PROBE`, `TESTING_FIRST`, `EXECUTING`, `AUDITING`, `MEMORIZING`, `COMPLETE`, `CONFIGURING` (`L391–394`).
- **[E] Recovery transitions:** `AUDITING→EXECUTING` for blocking fixes and `AUDITING→PLANNING` for dead ends (`L394–395`).
- **[E] 3A.1 state/persistence:** transition matrix; `ReplanRequest::{MINOR,MAJOR}` capped at one per agent per phase; `token_budget_remaining()`; MessagePack state at `.elai_cc/sessions/{sid}/.fsm-state.msgpack` (`L397–399`).
- **[E] 3A.2 phase-aware capability filters:** PLANNING blocks edit/bash; EXECUTING blocks planning tools; AUDITING blocks edit/planning; MEMORIZING allows memory-bank tools only; CONFIGURING only config paths; COMPLETE read-only; IDLE unrestricted (`L400–402`).
- **[E] Enforcement point:** `ToolRegistry::execute()` must return `ToolBlockedByState`; filters are not advisory (`L402–403`).
- **[E] 3A.3 seven evidence gates:** IDLE→PLANNING requires knowledge `READY|DEGRADED`; PLANNING→TESTING_FIRST requires zero ungrounded claims; TESTING_FIRST→EXECUTING requires tests fail for correct reasons; EXECUTING→AUDITING requires at least one FAIL→PASS; EXECUTING→PLANNING fast-track requires minor replan plus auditor validation; AUDITING→MEMORIZING requires zero blocks; MEMORIZING→next requires DKG + changelog (`L404–408`).
- **[E] Gate error UX:** `EvidenceError` includes actionable suggestions (`L408`).
- **[D] Feasibility transition gates:** wait for 4F (`L409`).
- **[E] 3A.4 human gate:** mockable `InteractiveGate::present_gate(evidence)` yields `APPROVE`, `REJECT` (no transition), `MODIFY` (recompute evidence), or `REDIRECT` (valid target only); interactive mode has no timeout (`L410–412`).
- **[E] 3A.5 risk UX:** every transition has `LOW|MEDIUM|HIGH`; interactive pauses on medium/high and automatically advances low; overrides in `[fsm.risk_overrides]` (`L413–414`).

### PHASE 3B — Shared State Board (`L416–439`)

- **[E] Journal architecture:** single-writer append-only SQLite/WAL inter-agent state; each `Ssb` instance permanently binds one `(project_id,session_id)` (`L416–419`).
- **[E] Schema:** `ssb`, `ssb_files` junction, `PRAGMA user_version` migrations, and r2d2 read pool whose `CustomizeConnection` enables foreign keys on every connection (`L418–420`).
- **[E] 17 action types:** `file_write`, `file_read`, `tool_result`, `test_outcome`, `finding`, `claim`, `checkpoint`, `model_swap`, `phase_transition`, `knowledge_degradation`, `isolation_metric`, `plan_update`, `replan_request`, `stub_reference`, `reasoning_trace`, `phase_complete`, `memory_monitoring` (`L421–423`).
- **[E] SQLite pragmas/order:** WAL, `synchronous=NORMAL`, auto-checkpoint 1000, busy timeout 5000, 64 MB negative cache size, foreign keys on, temp store memory; throughput expectation ~80K inserts/s (`L424–425`).
- **[E] Conflict query:** join `ssb_files.seq=ssb.seq`, filter project, and use `rarray()` for `IN` to preserve statement caching (`L426–427`).
- **[E] 3B.1 payload/read-pool behavior:** `Arc<[u8]>` for O(1) broadcast clones; disable idle timeout/max lifetime so default pool recycling does not close WAL (`L429–430`).
- **[E] 3B.2 commit protocol:** `BEGIN IMMEDIATE → INSERT ssb → INSERT ssb_files → COMMIT → broadcast`; broadcast only after commit; `created_at` from clock provider; SSB write completes before tool result dispatch; shutdown drains writes then exits (`L431–433`).
- **[E] 3B.3 reads/immutability:** WAL snapshot isolation; no update/delete APIs; debug update hook panics on mutation (`L434–435`).
- **[E] 3B.4 typing/dependency boundary:** typed MessagePack payloads for all 17 types, no opaque blobs; `SsbTraceable` lives in orchestrator to avoid dependency cycles; reasoning/tools emit IPC while orchestrator persists (`L436–438`).
- **[E] Determinism:** seed + injected clock yields byte-identical database dumps (`L439`).

### PHASE 3C — agent lifecycle, recovery, supervision, anti-spiral (`L441–472`)

- **[E] Scope:** spawn/lifecycle/checkpoint/crash recovery/supervision/anti-spiral; Phase 3 agents remain in-process Tier A until 4A Tier C (`L441–443`).
- **[E] Port mapping:** `WorkerRegistry→AgentPool`; UUID v4 from `EntropySource`; `WorkerStatus→AgentStatus` plus `Checkpointing`/`BudgetExceeded` and removal of `TrustRequired`; failure scenario maps to `AgentDeathClassification` (`L444–446`).
- **[E] 3C.1 pool:** `spawn`; sequential stub `spawn_parallel`; direct stub `communicate`; VRAM availability checked through `ModelPool` before spawn (`L447–452`).
- **[E] Agent handle:** `{agent_id,role,status,active_files,grc_config,scratch_dir}` (`L449`).
- **[E] Eight roles:** `Executor`, `Auditor`, `PromptEngineer`, `ContextCompiler`, `MemoryWriter`, `Researcher`, `MediaValidator`, `Designer`; designer differs by prompt/required steps despite shared model (`L450–452`).
- **[E] 3C.2 lifecycle/deaths:** `Spawning→Ready→Running→Finished|Failed`; death classes `CLEAN_EXIT`, `CRASH`, `TIMEOUT` (heartbeat 5 s/15 s), `BUDGET_EXCEEDED` (`L453–455`).
- **[E] Tier-A invisible walls:** hidden paths return exactly the same `NotFound` as absent paths (`L454–455`).
- **[E] Reasoning telemetry:** at most one replan per phase; every GRC step writes `ReasoningTraceEntry` to SSB (`L455–456`).
- **[E] 3C.3 checkpoint contents/location:** `.elai_cc/sessions/{sid}/agents/{agent_id}/{state,grc_history,tool_results,output_draft}.msgpack` plus `checkpoint_ts` (`L457–459`).
- **[E] Checkpoint triggers/durability:** every GRC step, tool call, file edit, and before slow operations; atomic temp-write + fsync + rename (`L458–459`).
- **[E] Channel checkpointability:** single `Topic`/`LastValue` references allowed; `UntrackedValue` references fail with `CheckpointError::UntrackedValueReference` (`L460–461`).
- **[E] 3C.4 recovery:** classify death, journal it, respawn with the same UUID, inject structured checkpoint; first recovered GRC step grounds to `CHECKPOINT:{path}`; corrupt checkpoint warns and starts fresh rather than permanently failing (`L462–464`).
- **[E] 3C.5 supervision:** default max three restarts within a phase; actions `Escalate` or `SubstituteModel(ModelId)`; Phase 3 escalates; model substitution waits for 4G; budget exhaustion does not increment restart count; reset count at phase boundary (`L465–468`).
- **[E] 3C.6 loop detection:** Blake3 content hash of step type + topic + normalized content; three consecutive identical hashes trigger spiral handling—checkpoint, convergence signal, then permanent stop if another identical step; recover from N−3 checkpoint, not latest (`L469–472`).

### PHASE 3D — BSP supersteps and typed channels (`L474–501`)

- **[E] Goal/guarantee:** deterministic BSP macro-orchestration; strict determinism only with mocked agents and fixed seed because live-LLM timeout participation can vary (`L474–477`).
- **[E] Removed primitive:** `BinaryOp` dropped because concurrent writes are nondeterministic (`L478`).
- **[E] `LastValue<T>`:** one value; winner is highest/last SSB sequence (`L479`).
- **[E] `Topic<T>`:** accumulating mode persists `Vec<T>`; nonaccumulating mode snapshots then clears after snapshot and before task writes (`L480–481`).
- **[E] `EphemeralValue<T>`:** saturating per-superstep write count; zero at barrier means `MISSING`; clearing is journaled so SSB-only replay works (`L482–483`).
- **[E] `UntrackedValue<T:DeepSizeOf>`:** memory-only, never serialized; force-clear beyond 2× default 10 MB limit; do not use shallow `size_of_val` (`L484–485`).
- **[E] 3D.1 registry:** read, write, scheduling snapshot, ephemeral clear, untracked force-clear, deep-size measurement (`L486–488`).
- **[E] 3D.2 loop order:** snapshot → prepare tasks → clear nonaccumulating topics → parallel execution → sequence-sorted apply → one-pass deferred phase → clear ephemerals → evidence gate → checkpoint only after gate passes; cap superstep attempts (`L489–492`).
- **[E] 3D.3 deferred nodes:** `NodeSpec.is_deferred`, same-superstep post-barrier, one pass with no cascade, `all_inputs_satisfied` gate (`L493–494`).
- **[E] 3D.4 checkpoint/replay:** serialize `HashMap<ChannelId,ChannelState>` except untracked values; separate `memory_monitoring` event; recovery loads checkpoint then replays SSB in sequence, reconstructing ephemeral counts and topic-boundary clears (`L495–498`).
- **[E] 3D.5 real parallelism:** upgrade pool to `tokio::JoinSet`, per-agent heartbeat timeout, `max_concurrent=floor(available_vram/per_agent_vram)`, and apply results in SSB order rather than completion order (`L499–501`).

### PHASE 3E — fixed-slot context budget allocator (`L503–521`)

- **[E] Eight slots:** `SystemPrompt`, `Rules`, `PhaseSpec`, `RepoMap`, `CodeContext`, `SsbEntries`, `GrcHistory`, `OutputReserve` (`L503–506`).
- **[E] Allocation:** exact-measure slots 1–3; reserve `max(configured,2000)` output tokens; split dynamic remainder repo map 15%, code context 40%, SSB 25%, GRC 20% (`L506–507`).
- **[E] Floors:** SSB ≥200, GRC ≥500, output ≥2000; steal from code context then repo map; dynamic remainder <700 is `BudgetInfeasible` (`L508–509`).
- **[E] Cache stability:** breakpoints after PhaseSpec and CodeContext; use deterministic xxhash/FxHash, never `DefaultHasher` (`L510–511`).
- **[E] SSB relevance:** file overlap `|entry.files ∩ agent.files| / |agent.files|`; sort score descending then sequence descending; omit zero score; empty agent file set safely scores zero (`L512–513`).
- **[E] Three reductions:** `FilteredByRelevance` (stricter overlap/same-topic GRC), `ReducedGranularity` (denser map, signatures/doc comments), `ReducedScope` (current file/direct deps only); journal `context_reduction` (`L514–517`).
- **[E] Terminal handling:** interactive escalates with breakdown; autonomous splits task in 4G (`L518`).
- **[E] Counting/SLO:** sampling estimate for allocation, exact count for validation; under 50 ms (`L519–520`).
- **[Q] Step detail:** only “Steps 3E.1–3E.7 as numbered” appears; individual tasks require `phase-3E.txt` (`L521`).

### PHASE 3F — deterministic, self-improving rules engine (`L523–553`)

- **[E] Rules-as-data:** deterministic enforcement from SQLite rather than prompt rules (`L523–524`).
- **[E] Dependency architecture:** rules does not depend directly on tree-sitter; it consumes knowledge through `AstEvaluator` (`L524–526`).
- **[E] Evaluation order:** cheapest first, short-circuit on block: `METRIC_GATE → GREP_PATTERN → AST_PATTERN → COMPOSITE` (`L526–527`).
- **[E] Predicates:** `AST_PATTERN`, `GREP_PATTERN`, `SCHEMA_REQUIRED` (escalates provider tier), `METRIC_GATE`, and depth≤8 `COMPOSITE` `AND|OR|NOT`; verdict `ALLOW|WARN|BLOCK|ESCALATE(target)` (`L528–530`).
- **[E] 3F.1 schema:** `rules`, `rule_metrics`, `contract_tests`, `violation_patterns`, `schema_version` (`L531–532`).
- **[E] 3F.2 admission compile:** validate regex, AST query, and JSON schema at insertion; `AstEvaluator` defined for knowledge implementation (`L532–534`).
- **[E] 3F.3 object-safe API:** `RuleEngine::{evaluate,get_active_rules,compile_predicate}` (`L535`).
- **[E] 3F.4 lifecycle:** `PROPOSED→CANDIDATE→SHADOW→ACTIVE→RETIRED` (`L536`).
- **[E] Promotion gates:** at least three contract tests; human/independent second-auditor approval, with finding-source auditor forbidden to self-approve; shadow→active requires FPR<5%, TPR>80%, counterfactual causal rate>50%, and no safety regressions (`L536–539`).
- **[E] Immutable exception:** `mutable=0` rules do not auto-retire and need not pass the causal-rate gate (`L539`).
- **[E] 3F.5 interpreter:** compile regex once; depth guard 8; NaN/Inf guard; escalation route is shadow→auditor, immutable→human, tier chain→next-tier rule (`L540–541`).
- **[E] 3F.6 activation:** phase bitmask plus domain detection; engineering/security always active; indexed case-insensitive domain query (`L542–543`).
- **[E] 3F.7 discovery:** orchestrator normalizes and Blake2s-hashes recurring violations; three occurrences auto-propose after compile validation; auditor-origin rules remain proposed; orchestrator, not auditor, creates ALLOW counterexamples (`L544–547`).
- **[E] 3F.8 seeds:** `rules_seed.db` with at least 33 rules, including three post-tool completeness grep rules; web fetch image/table warnings and file-read TODO/FIXME info findings; immutable seeds protected (`L548–549`).
- **[E] 3F.9 test scale:** at least 254 static tests plus 256 property cases/rule for violation/valid generators, mutation invariance, fix mutation, De Morgan, associativity; shrunk counterexamples reinserted with `source=GENERATED` (`L550–552`).
- **[E] 3F.10 SLO:** grep median <1 ms, metric <0.1 ms, mixed ten-rule evaluation <10 ms (`L553`).

### PHASE 3G — hierarchical plan management (`L555–585`)

- **[E] Two-level artifact:** skeleton ≤2–4K tokens always in Slot 3 plus per-phase elaborations, at `.elai_cc/pipelines/{id}/skeleton.toml` and `phases/{phase_id}.toml`; every update journals `plan_update` (`L555–558`).
- **[E] Storage clarification:** no `plan_phases` table; queries use SSB action filter plus JSON payload (`L558–559`).
- **[E] Types:** ordered `PlanSkeleton`; 120-character-goal `PhaseEntry` with typed inputs/outputs, constraints, dependency links, versions, output type/visual check; `ContractRef`; `PhaseDetail` with skeleton version, contracts, implemented contracts, structured declared interfaces, content/token count; `ExtractedContracts` (`L560–564`).
- **[E] 3G.1:** TOML serde and goal-length validation (`L565–566`).
- **[E] 3G.2 size/write gate:** estimated precheck, exact count at 3600 safety margin, hard reject above 4096; every write emits plan update; missing detail emits `stub_reference`; contract changes require versioning (`L567–569`).
- **[E] 3G.3 extraction:** inputs are union of dependency outputs; outputs union of dependents' inputs; shared constraints are global+phase; conflicting contract names across dependencies detected (`L570–571`).
- **[E] 3G.4 context:** skeleton always, active detail, extracted contracts, and one-hop adjacent contracts in Slot 3; reduce granularity on overflow; skeleton-version mismatch marks stale (`L572–574`).
- **[E] 3G.5 coherence:** implemented contracts must superset extracted contracts; declared interfaces must be subset of skeleton; contract change must increment both phase and skeleton version; failures block and journal finding (`L575–579`).
- **[E] 3G.6 propagation:** BFS `depended_by` marks `needs_review`; Kahn cycle detection O(V+E); checked version increment; split preserves contracts and depth≤2 to match 4G (`L580–583`).
- **[E] 3G.7 acceptance:** five-phase E2E verifying budget/extraction/coherence/propagation/version/journal, deterministic 100 times (`L584–585`).

### PHASE 3H — DAG session handoff and lossless trimming (`L587–614`)

- **[E] Goal:** DAG-versioned sessions with structurally lossless three-pass trimming; stubs are recoverable by SSB sequence (`L587–589`).
- **[Q] Required amendments:** per-message `snapshot_context` rather than snapshot blobs; branch-op action type must reuse checkpoint or add `branch_abandon`; merged-branch cleanup must be logical filtering, never physical deletion (`L590–592`).
- **[E] Schema:** snapshot UUID/parent/FSM/phase/step/findings/code changes/tests/commits/tag; context row role constrained to system/user/assistant/tool plus content/stub fields; partial index on stubbed rows (`L593–596`).
- **[E] 3H.1 DB safety:** default journal mode, not WAL, because session has one writer; integrity check on open; foreign keys (`L597–599`).
- **[E] 3H.2 CRUD/traversal:** transactional save, sequence-ordered joined load, latest snapshot, cycle-guarded parent chain; ~50K batch inserts/s (`L600–601`).
- **[E] 3H.3 trimming:** stub tool results over configurable threshold constrained to [100,1000]; stub records original type/token count, Blake3 summary hash, SSB sequence; validate sequence exists or un-stub; never stub user/assistant or cache-prefix slots 1–3; target tokens = model context − allocated slots − output reserve (`L602–606`).
- **[E] 3H.4 branching:** maximum three active branches; merge creates a new trunk snapshot from winner tip; immutable DAG parents; abandoned branches tagged, not deleted; overlapping modified files produce conflict (`L607–609`).
- **[E] 3H.5 restore:** lookup by SSB sequence, verify Blake3, restore content/flags; pruned SSB is a typed nonpermanent error (`L610–611`).
- **[E] 3H.6 lifecycle:** trigger handoff at 85% token usage; when branches exist load branch tip, not trunk handoff; commits persist; 100-run deterministic test with seeded providers (`L612–614`).

### PHASE 3I — deterministic simulation testing (`L616–644`)

- **[E] Goal/oracle constraint:** prove FSM+SSB+BSP using an infallible digital-sibling reference model whose LOC is <30% of real FSM (`L616–618`).
- **[E] Test infrastructure:** VCR at HTTP transport (not provider trait), trajectory `TraceOps` record/replay, and SQLite VFS write/fsync interception subject to spike (`L619–621`).
- **[E] 3I.1 platform spike:** evaluate `madsim 0.2.34` on Windows with feature-gated dependency swap; document manual deterministic scheduler if it fails (`L622–624`).
- **[E] 3I.2 model:** `SimulatorRuntime`; proptest reference-state-machine implementation; SUT returned by value; ten-event `FsmEvent`, including composite `SuperstepBoundary` and post-recovery `AgentCleanup` (`L625–628`).
- **[E] 3I.3 build gate:** zero regressions and no warnings above baseline (`L629`).
- **[E] 3I.4 state testing:** sequences 1..200; 10,000 CI cases with 10-minute limit, 256 local; seed regression corpus through synthetic failure; `check_determinism` takes bare static fn, never closure (`L630–633`).
- **[E] 3I.5 BSP proof:** 100 identical checkpoints; untracked excluded; zero-write ephemeral cleared; topics ordered by SSB sequence; two-phase reset represented; entropy grep over orchestrator must return zero raw time/random calls (`L634–637`).
- **[E] 3I.6 faults:** 30% packet loss, uniform 0..500 ms latency, asymmetric link clogging, SIGKILL, correlated burst loss 100% for five then zero (`L638–639`).
- **[E] 3I.7 adversarial space:** anti-spiral, gate bypass, context overflow, SSB conflict, demonic scheduling (`L640–641`).
- **[E] 3I.8:** implement reference/bridge/invariants and cross-platform LOC ratio check (`L642–643`).
- **[E] 3I.9 acceptance:** `phase_3I.toml` state coverage >80%, determinism 100%, regression count (`L644`).

### PHASE 3J — vLLM, advanced providers, plugins, orchestration hardening (`L646–682`)

- **[E] Backend:** vLLM over OpenAI-compatible HTTP enables layer-level DoLa, TurboQuant KV, and ActAdd that Ollama cannot expose (`L646–649`).
- **[E] Hardware envelope:** dev Ryzen 7800X3D/64 GB/RTX 3060 12 GB, primary Gemma4 E2B ~2B, target Gemma4 31B ~18 GB Q4 on 48 GB GPU; CPU offload 0.5–2 tok/s is enough for artifacts but not EAGLE-3 training (`L650–652`).
- **[E] Installed/model assumptions:** `gemma4:31b` 19 GB Q4/262144 context; `gemma4:e2b` 7.2 GB Q4_K_M; `gemma3:4b` fallback; vLLM 0.19.0 in WSL2 `~/vllm-env/` (`L653–654`).
- **[E] 3J.1:** `VllmProvider` for `/v1/completions`, `/v1/models`, SSE streaming (`L655–656`).
- **[E] 3J.2:** `InferenceBackend::{Ollama,Vllm}`, tensor parallelism, fallback with warning (`L657`).
- **[E] 3J.3 DoLa:** JSD-selected premature layer, plausibility alpha 0.1, model-specific buckets, per-role toggle, guard below 1B parameters, degraded top-N-logprobs path (`L658–660`).
- **[E]/[D] 3J.4 TurboQuant:** evaluate `FEASIBLE|DEFER`; QR rotation + Lloyd-Max codebook, `b=3` ~10.7×; codebooks reusable by models sharing head dimension (`L661–662`).
- **[E] 3J.5 ActAdd:** mid-layer contrast `h_A^l=h+^l−h−^l`, sharing DoLa hook path (`L663–664`).
- **[E] 3J.6:** GPU multiplexing with 1 GB margin and `ModelPool.can_fit_onnx()` (`L665`).
- **[E] 3J.7–3J.8:** FSM-driven predictive preload and per-agent/per-phase workload profiles in SSB (`L666–667`).
- **[E]/[D] 3J.9:** MILP scheduling evaluation (`good_lp`+HiGHS) against greedy, with `ADOPT|DROP`; Windows build risk (`L668–669`).
- **[E] 3J.10–3J.13:** port 1751-LOC MCP bridge; static hooks T1<1 ms/T2<10 ms/T3<5 ms and no hot-path LLM; Windows AppContainer/job limits/self-mod locks; complete typed SSB actions (`L670–673`).
- **[E] 3J.14/15:** validate Gemma4 31B CPU offload for steering vectors, DoLa, and codebook generation; numbering moved from 3J.14 to 3J.15 (`L674, L682`).
- **[C] Added composition:** Python Anthropic-compatible composed inference sidecar auto-started by CLI; `--local` full interactive mode; `InferenceDispatcher` with API/local implementations, FSM tool filtering, backend mode `local|api|hybrid|auto`; wiring substeps foundation/budget/rules/session/plan/stubs/E2E including Ollama fallback, retrieval prompt injection, release build, one `elai.bat` command (`L675–681`).

### PHASE 4A — Tier-C agent process isolation (`L688–719`)

- **[E] Isolation boundary:** every agent is an isolated OS process; orchestrator-agent IPC uses Unix sockets or named pipes; all Phase 4 subphases must cross this boundary (`L688–690`).
- **[E] Framing:** one-byte version + four-byte length + Postcard payload; absolute frame cap 10 MB; per-type caps heartbeat <1 KiB, think step <64 KiB, tool result <10 MB, response <context limit (`L691–692`).
- **[E] 15 messages:** `AgentRequest`, `AgentResponse`, `InferRequest`, `InferResponse`, `ToolRequest`, `ToolResult`, `BashRequest`, `BashResult`, `GroundingRequest`, `GroundingResult`, `ThinkStep`, `CheckpointSync`, `AgentHeartbeat`, `AgentKill`, `ReplanRequest` (`L693–695`).
- **[E] BLOCK-1:** Linux seccomp profiles `IPC_BASELINE` for all roles and `NETWORK_EXTRA` only for researcher; BPF checks socket-domain arguments, not blanket socket permission (`L696–698`).
- **[E] BLOCK-2:** researcher networking uses veth+iptables with zero capabilities, explicitly no `CAP_NET_RAW` (`L699`).
- **[E] BLOCK-3/4:** agent binary has no LLM client, `reqwest`, or providers dependency; all inference is proxied; orchestrator externally counts think steps and owns GRC budget while agent budget is advisory (`L700–702`).
- **[E] BLOCK-5:** IPC sockets under 0700 `/tmp/elai_cc_{session}/`, peer credentials verified on accept, socket removed on agent death (`L703–704`).
- **[E] BLOCK-6:** auditor rules stay proposed until approved by a separate auditor (`L705`).
- **[E] BLOCK-7:** Windows prefers AppContainer; restricted token + ACL deny is documented as weaker fallback (`L706–707`).
- **[E] BLOCK-8:** self-mod locks enforced by canonical-realpath IPC check, read-only filesystem mount, and commit diff filter (`L708–709`).
- **[E] BLOCK-9:** adversarial IPC battery (`L710`).
- **[E] Dispatch authority:** `AgentDispatchConfig {role,model_preference,tool_filter,rule_domains,grc_config,sandbox_config}` is authoritative, including IPC whitelist; role alone is insufficient; feasibility executor override is GRC 3/600, only stub-write/bash, 25/30-second timeouts (`L711–714`).
- **[E] Swap durability:** `phase_complete` SSB row is atomic boundary marker and fsyncs before model unload (`L715`).
- **[E] 4A gates/SLO:** skeleton, compile, functional isolation, adversarial IPC, implementation, measurements; IPC p99 <1 ms, spawn <100 ms, memory <30 MB, overhead <5% (`L716–717`).
- **[D] ShortCoder placement:** wait for 4B; orchestrator must run it after `AgentResponse` and before auditor views filesystem (`L718–719`).

### PHASE 4B — orchestrator-owned tools (`L721–743`)

- **[E] Architecture:** tools execute in orchestrator and agents reach them only through typed IPC requests/results (`L721–722`).
- **[C] Recorded status 2026-04-16/commit `55ece36`:** 4B.1–4B.4 done—ten tool modules, `IpcToolRegistry`, 15 typed argument structs, 19/19 tests, three blocks fixed (`L723–725`).
- **[D] Recorded status:** 4B.5 measurement deferred; 4B.6–4B.7 `IntegrationTestRunner` deferred pending 4E `DecompositionTree` types (`L724–725`).
- **[E] Git tools:** native `git2`, deterministic auto-commit message, `plan-{id}` branch, WIP commits, safe undo (`L727`).
- **[E] Five-tier validation:** determinism → quality/AST → unit → integration → stress; auditor-only (`L728`).
- **[E] DAIRA markers:** `[CALL]`, `[RETURN]`, `[MUTATION]`, `[BRANCH]`, `[ERROR]`, `[PANIC]`, `[TIMEOUT]`, `[FINDING]`, `[PATTERN]`, `[ANOMALY]`, `[STAT:name]`, `[DURATION:name]`, `[MEMORY:name]`, `[CHECKPOINT]`, `[STEP:n]` (`L729–730`).
- **[E] Adaptive tool:** `AdaptiveTool<T>` reduces scope based on budget at most three times (`L730–731`).
- **[E] Edit matching:** exact → whitespace-normalized → ellipsis-elision, returning a `SequenceMatcher` similarity hint on failure (`L732–733`).
- **[E] ShortCoder:** AST-equivalent compression with ten tree-sitter rules R1–R10; R4/R6 configurable (`L734`).
- **[E] Failure taxonomy:** deterministic regex categories `SYNTAX_ERROR`, `TYPE_ERROR`, `IMPORT_ERROR`, `ASSERTION_ERROR`, `TIMEOUT`, `EMPTY_OUTPUT`, `RUNTIME_PANIC`, `PERMISSION_DENIED`, `RESOURCE_EXHAUSTED` (`L735–736`).
- **[E] Large-file read:** files >300 lines default to outline; `force_full` is budget guarded (`L737`).
- **[E] Async incremental compilation:** after write run `cargo check`, `ruff`, `tsc`, or `go vet` for the file while executor proceeds; result returns through IPC; three retries then `NEEDS_AUDIT`; skip during TESTING_FIRST; defer errors referencing modules not yet written (`L738–740`).
- **[E] Step ladder:** 15 typed args, verify, ≥15 tests, implement, measure, then integration runner skeleton/implementation with `BABY_STEP<5s`, `STEP<30s`, `PHASE<60s`, `CONCEPT<120s` (`L741–743`).

### PHASE 4C — invisible hooks and tiered static analysis (`L745–768`)

- **[E] Interception model:** hooks run in orchestrator around IPC and agents cannot see that hooks exist; T3 is deterministic pattern matching with no LLM; `ruff` post-tool subprocess gets a circuit breaker above 10,000 lines (`L745–748`).
- **[E] Hook API:** `Hook` with `PreToolUse`, `PostToolUse`, `Stop`, `PreCompact`; verdict `Allow|Block{reason}|Warn{reason}` (`L749–750`).
- **[E] Hook timing:** pre fires before forwarding tool/bash; post runs ruff + T1/T2 + ShortCoder before result/response; stop requires FSM idle; precompact checkpoints at 85% (`L750–752`).
- **[E] T1:** indexed phase/domain rules DSL, <1 ms; schema predicate escalates provider tier rather than being handled at T1 (`L753–755`).
- **[E] T2:** tree-sitter syntax/dead-code/lazy-code checks, <10 ms under 5000 lines; breaker above 5000 (`L756–757`).
- **[E] T3:** pattern library, runs only when T1+T2 warn without blocking, <5 ms; never runs once already blocked (`L758–759`).
- **[E] Error sanitization:** external block message is generic `"Tool call rejected by policy"`; internal rule ID/pattern/severity removed at enum level, not by string scrubbing (`L760–761`).
- **[E] Circular trust:** orchestrator normalizes and Blake2s-hashes; independent approval for auditor rules (`L762–763`).
- **[E] 4C tests:** 30 contract tests covering hooks, tiers, IPC, trust, session locks, schema escalation, T3 compound behavior, ShortCoder placement; ten adversarial tests (`L764–766`).
- **[E] 4C SLO:** T1<1 ms, T2<10 ms, T3<5 ms, generic hook overhead <2 ms, ruff-inclusive post-hook <500 ms (`L766–767`).
- **[D] Skills/MCP:** prompt-template skills have no code path here; MCP bridge is a separate 1751-LOC subphase (`L768`).

### PHASE 4D — Pipeline Factory L1, research + design + recursive decomposition (`L770–798`)

- **[E] Goal:** isolated Research Agent + distinct Designer turn task text into `PipelineSpec` and `QualitySpec` through grounded tree research (`L770–775`).
- **[E] Research isolation:** Tier-C process, web only via IPC proxy, no `.elai_cc/`, no project source, no direct model access (`L772–775`).
- **[E] Role distinction:** Designer and Executor may use same model but differ in role prompt, required steps, and output (`L774–775`).
- **[E] Research types:** `ResearchTree` config (`breadth_factor`, `depth_limit`, `max_iterations`, `phi_min`, `psi_min`, `speculative_depth`, `time_budget`); `ResearchNode` with parent/children/scores/status/info gain/iterations; `ResearchPlan`; `ResearchReport`; snapshot; convergence status (`L776–780`).
- **[E] Evidence/source types:** `EvidenceLevel::{Benchmark(1.0),OfficialDocs(0.8),BlogPost(0.6),Training(0.4)}`; `Alternative`; `Contradiction`; deterministic five-tier `SourceQuality` (`L780–782`).
- **[E] Product-spec types:** `PipelineSpec`, `QualityMetric`, `PerfSpec`, `SourceCitation`; `MediaSpec` output `Web|Model3D|Audio|UiScreenshot|Animation|Image|Custom` (`L782–784`).
- **[E] Complexity presets:** Simple 2×depth3/φ.8/ψ.7; Moderate 3×depth4/.7/.6; Complex 4×depth5/.6/.5 (`L785–788`).
- **[E] Pruning:** info gain under threshold three consecutive iterations marks `Pruned` and frees resources (`L789`).
- **[E] Base gates:** skeleton, compile, ≥14 contract/integration/property tests, adversarial tests, implement, measure (`L790–791`).
- **[E] 4D.7–4D.10 recursive decomposition:** `Idea→Concepts→Phases→Steps→BabySteps`; every level invokes grounded FlashResearch (`L792–794`).
- **[E] Decomposition types:** `ConceptDecomposition`, `ConceptNode`, `PhaseNode`, `StepNode`, `BabyStepSpec` (`L793–795`).
- **[E] Eight baby-step blocks:** `GROUNDING`, `PRE_GATE`, `TASK`, `TDD`, `DETERMINISTIC_CHECK`, `ACCEPTANCE`, `INTEGRATION`, `EXIT_CRITERIA` (`L794–795`).
- **[E] Technology/package grounding:** `TechStackDecision` source is `RESEARCH|EXISTING_PROJECT|USER_SPECIFIED|FALLBACK`; `PackageRef` must pass `API_REGISTRY_CHECK`; `ResearchCache` is LRU (`L796–797`).
- **[E] Selection UX:** interactive exposes alternatives; autonomous selects highest evidence (`L798`).

### PHASE 4E — deterministic Pipeline Factory L2 (`L800–828`)

- **[E] Pure builder:** no LLM; pure-Rust orchestrator transforms `PipelineSpec` into executable pipeline (`L800–802`).
- **[E] Five-stage flow:** Research → Plan Assembly → Structural Validation → Phase Sizing (stage 3.5) → Feasibility Probe (`L802–803`).
- **[E] Bounded DAG repair:** at most ten rounds with structured delta feedback; deterministic `REORDER` repairs cycles and `SUBSTITUTE` replaces missing tools only when category and I/O match; reject SOCIA LLM repair (`L804–807`).
- **[E] Types:** `ExecutablePipeline` with ID/spec/FSM/dependency graph/novel steps/agent configs/rules/resources/tests/repair log/output; `DependencyGraph`; `DagNode`; compiled `PipelineFsmState`; `RepairAction::{Reorder,Substitute}`; `RepairOutcome::{Repaired,Escalated}`; eight-kind validation error (`L808–811`).
- **[E] Eight structural checks:** registered tool, available model, acyclic graph via Kahn, hardware fit, measurable quality metrics, compiled predicates, novel-step marking for 4F, visual-check adapter mapping (`L812–814`).
- **[E] Sizing formula:** `files_touched·avg_file_tokens + test_count·avg_test_tokens + contract_count·200`; split above 60% of available work budget; depth≤2 (`L815–817`).
- **[E] 4E.1–4E.6:** base builder with at least 30 tests (`L818`).
- **[E] 4E.7–4E.12 primary execution artifact:** typed `DecompositionTree` + `TreeNavigator`, not prompts; depth-first/topological concept→phase→step→baby step (`L819–822`).
- **[E] Baby-step cycle:** context Slot 3 → pre-gate → TESTING_FIRST → EXECUTING → AUDITING → acceptance → integration → exit criteria (`L822–823`).
- **[E] Bubble-up retries/actions:** retry thresholds baby 3, step 2, phase 1, concept 1; actions `ResearchAlternative`, `RedecomposeStep`, `ReplanPhase`, `ReplanConcept`, `Escalate` (`L824–825`).
- **[E] Integration checkpoints:** baby `cargo check` <5 s, step module tests <30 s, phase crate tests <60 s, concept workspace tests <120 s (`L826–827`).
- **[E] Crash recovery:** serialize `TreeNavigatorState` to MessagePack after every baby-step completion (`L828`).

### PHASE 4F — feasibility probing (`L830–845`)

- **[E] Scope:** resource probe plus novel-step stubs in Tier-C sandbox with branch-and-explore (`L830–832`).
- **[E] Skip rule:** skip only if there are zero novel steps and each of VRAM, RAM, disk has ≥20% margin (`L831–832`).
- **[E] Types:** `SystemProbeResult`, stable-NVML-UUID `GpuInfo`, `SystemConstraints`, `StubResult`, `ResourceDelta`, `FeasibilityReport`, `ConstraintResult`, `FeasibilityEscalation`, and decision `Proceed|ProceedWithConstraints|Escalate|Replan` (`L833–835`).
- **[E] Restricted executor:** GRC 3/600, only `stub_write`/`bash`, 25 s graceful and 30 s hard kill, each stub <50 lines, via 4A IPC (`L836–837`).
- **[E] Tests/workflow:** object-safe trait and eight types; compile; ≥20 tests for success/no GPU/low RAM/skip/isolation/dual timeout/max-three alternatives/escalation/oversized report; implement, measure, adversarial (`L838–841`).
- **[E] 4F.7–4F.8 concept sequencing:** baby-step feasibility per concept, sequential because earlier outcomes may change later approach; types `BabyStepFeasibilityReport`, `BabyStepStubResult`, `BabyStepEscalation` (`L842–844`).
- **[E] Novel baby-step predicates:** unverified package, unused framework, or untested capability (`L844–845`).

### PHASE 4G — autonomous-mode wrapper (`L847–873`)

- **[E] Mode architecture:** autonomy adds no FSM state; `AutonomousGate` wraps an `InteractiveGate` with evaluator/mode/escalation policy/dispatch table (`L847–850`).
- **[E] Compatibility:** interactive delegates exactly to inner gate; autonomous checks deterministic evidence first and calls human only on high risk, missing evidence, or escalation; in-state tool review remains auditor-driven (`L849–852`).
- **[E] Evidence map:** knowledge not unavailable; zero ungrounded; test exits nonzero without Rust compile error; fail→pass; zero audit block; DKG+changelog (`L853–856`).
- **[E] Escalation defaults:** spiral threshold 3, high risk pauses in both modes, major replan always human, at most ten auto advances/phase (`L857–858`).
- **[E] Triggers:** `AntiSpiral`, `HighRisk`, `ReplanMajor`, `CircuitBreaker`, `SubstituteModelFailed`, `SplitTaskFailed` (`L859`).
- **[E] Split task:** at most four children, depth two, sequential merge of all results; parent writes directly to SSB; pick-one branch semantics are explicitly incompatible (`L860–862`).
- **[E] Model fallback:** iterate fallback chain through swap protocol; add `ModelPool.can_fit_llm()`; no fit yields `SubstituteModelFailed` (`L863–864`).
- **[E] Base gates:** skeleton, interactive regression check, contract tests, adversarial, implementation, measurement (`L865–866`).
- **[E] Tree autonomy:** operate at baby-step grain; risk baby/step low, phase medium, concept high; concept bubble-up always human; human check forced after 50 auto baby steps per concept (`L867–870`).
- **[E] Runtime-selection UX:** `elai --local` sidecar, `--hybrid` per-role, default cloud API; `/model` live change; `[agents.{role}.backend] = local|api|auto` (`L871–873`).

### PHASE 4H — measured, bounded self-modification (`L875–910`)

- **[E] Core separation:** learn configs/rules/data from outcomes; modifying agent never evaluates its own change; DGM archive accumulates variants for parent selection rather than direct pairwise comparison (`L875–878`).
- **[E] Class A:** additive data rows for deprecated patterns, failure categories, lazy-code regexes, source tiers; validate FPR across N pipelines; no staged benchmark (`L879–881`).
- **[E] Class B:** tune ETC threshold, baby-step LOC, AB-MCTS temperatures, context weights, freshness half-life, retries, steering coefficients; staged 10/50/200 evaluation (`L882–884`).
- **[E] Layer 3:** FSM states, roles, and GRC protocol are human-only (`L885`).
- **[E] Outcome telemetry:** mode `SURGICAL|GREENFIELD|ORACLE`, ETC triggers, stale API catches, context truncations, taxonomy misses (`L886–888`).
- **[E] Catch taxonomy:** `T1_HOOK|T2_HOOK|AUDITOR|ETC`; three unclassified occurrences auto-propose a failure category (`L887–889`).
- **[E] SWE-bench outcome matrix:** `FAIL_TO_PASS`, `PASS_TO_PASS`, `FAIL_TO_FAIL`, `PASS_TO_FAIL`; regression hard-rejects (`L889–891`).
- **[E] Variant:** ID, parent, normalized score [0,1], mode, status `ACTIVE|ARCHIVED|REVERTED` (`L891–892`).
- **[E] Score:** normalized weighted components only, never raw count; default weights first-try .3, coverage .2, efficiency .2, intervention .15, findings .15 (`L893–895`).
- **[E] MSI:** λ=.3 exponentially decayed delta score over N=10 per mode; improving >.3, stabilizing −.1.. .3, diverging ≤−.1 pauses+alerts (`L896–898`).
- **[E] Parent selection:** `sigmoid(10·(alpha_i−.5))`, exploration `1/(1+children_count)`, product normalized to probability; surgical parent never feeds greenfield evaluation (`L899–901`).
- **[E] Closed mutation surface:** new allowed keys require Layer-3 approval; parameter keys `EtcThreshold`, `FreshnessHalfLife`, `BabyStepMaxLoc{vram_tier}`, `ContextSlotWeight{slot}`, `RetryBudget`, `AbMctsMaxBranches`, `AbMctsTemperatures`, `MicroReflectionMaxTokens`, `SteeringVectorCoefficient`, `SelfModFrequency`; data keys `DeprecatedPattern`, `FailureTaxonomyCategory`, `LazyCodePattern`, `SourceTierEntry` (`L902–907`).
- **[E] Four safety layers:** dispatch, IPC, filesystem, git; modifying agent cannot read lock list; auto-revert after >10% degradation across three pipelines; trigger task detects objective hacking (`L908–910`).

### PHASE 4I — conditional media validation (`L912–928`)

- **[E] Condition/scope:** instantiate only for media pipelines; validate image, 3D, audio, UI screenshot (`L912–914, L923`).
- **[E] Tier 1:** deterministic ImageMagick/FFmpeg/Tesseract/Blender CLI, target <100 ms, veto authority (`L914–915`).
- **[E] Tier 2:** VLM constrained JSON, advisory only (`L915`).
- **[E] Mode A:** Qwen3.5-VL-30B-A3B via Ollama Q4_K_M ~17 GB, all-30B storage despite ~3B active, Ollama ≥0.12.7 (`L916–917`).
- **[E] Mode B:** InternVL3.5-38B via SGLang only with RadixAttention; ~38 GB 8-bit needs dual 3090 tensor parallel, or ~21 GB Q4 (`L918–919`).
- **[E] Audio:** CLAP audio-text similarity, ~200M/~0.4 GB FP16, Xenova export (`L920`).
- **[E] Image metrics determinism:** SSIM/LPIPS CPU execution only; epsilon 1e−6 rather than bit-exact (`L921–922`).
- **[E] Render/capture/evaluate:** adapter trait for Playwright web, Blender 3D, FFmpeg animation; produce→render→VLM→structured feedback→iterate, bounded N (`L924–926`).
- **[E] Concurrency/verdicts:** Tier1 max4, Tier2 max1, queue64; T1 fail→Fail; T1 pass+T2 disagree→HumanReview; no VLM→Degraded (`L927–928`).

### PHASE 4J — conditional Empire integration (`L930–950`)

- **[E] Condition/value exchange:** only when Empire source available; ELAI becomes a brain and supplies self-improvement/reasoning/pipelines, receiving a live deployment environment (`L930–933`).
- **[E] Verified topology:** Engine MKI→MKII→MKIII; engine IPC at `/tmp/brain-engine.sock` and factory line `/tmp/brain-factory.sock`, newline JSON/Unix stream; max 65,536-byte engine messages and 256 KiB FSB frames; hook route/event, fnmatch bus, multi-brain isolation (`L934–937`).
- **[E] Repair targets:** `brainctl↔brain_lifecycle` circular dependency, EngineMKII hard import of `slc`, and `unique-factory/types` standard-library shadow (`L938–939`).
- **[E] Supersession:** replace seven-line lambda action stubs and unvalidated `brain.json` with PipelineSpec's six enforcement layers (`L940–941`).
- **[E] 4J.1 abstractions:** cross-platform `FactoryLineTransport` with Unix/TCP; generic client; schema-validated `BrainManifest`; rate-limited/filtered event bridge; `ElaiNeuron`; repair target with `CircularDep|HardImport|StdlibShadow` (`L942–947`).
- **[E] Exposed capabilities:** only `task_submit`, `task_status`, `pipeline_result`, `health_check`; tool filter; leak zero internal state (`L944–946`).
- **[E] Wire types:** `FactoryLineCommand` (`Register|Heartbeat|List|RequestUpdate`) with protocol `v:u8`; nine-kind `IpcMessage`; shared Event/Command/Response (`L948–949`).
- **[E] Degradation/build:** generic transport, graceful operation without Empire, feature gate `empire` (`L950`).

### PHASE 4K — native Tauri dashboard MVP (`L952–974`)

- **[E] UX:** native structured control surface, not a chatbot; every input undergoes FSM classification; interactive user drives and approves while ELAI executes (`L952–955`).
- **[E] Stack:** Tauri 2.10/build 2.5/CLI 2.10, Svelte/Vite webview, Rust commands; no HTTP, REST, WebSocket, or CORS; invoke-poll read model every 500 ms (`L956–958`).
- **[E] Visual identity:** teal `#1B7A6E`, charcoal `#0f1318`, JetBrains Mono, information density over whitespace (`L959–960`).
- **[E] 4K.1 read model:** read-only WAL `SsbViewer`; typed queries `latest_fsm_state`, `active_session`, `phase_progress`, `recent_actions`, `agent_actions`, `pending_interventions`, `file_changes`, `reasoning_chain` (`L961–964`).
- **[E] Intervention events:** add `INTERVENTION_REQUEST` with type `ApproveChange|ReplanApproval|ErrorRecovery` and options `Accept|Decline|Retry(String)|Skip`; add `INTERVENTION_RESPONSE` (`L964–966`).
- **[E] Viewer performance:** lightweight projection, pagination, <10 ms on 100K rows (`L967`).
- **[E] 4K.2 scaffold:** dashboard crate, six Tauri commands, Svelte/Vite/CSS, `elai --dashboard` (`L968–969`).
- **[E] UI surfaces:** project selector, task input, phase ribbon, intervention panel, activity feed, reasoning chain, commit history (`L970–971`).
- **[E]/[D] Deployment risks/path:** WebView2 expected on modern Windows; bundle 3–8 MB accepted; 500 ms poll accepted for MVP; future push through `app.emit` (`L972–974`).

### PHASE 4M — multi-runtime backend abstraction (`L976–1000`)

- **[E] Goal:** replace bespoke local-sidecar/cloud dispatch with one workflow spec and interchangeable runtime adapters; port the Ouroboros adapter/factory/name-resolution pattern (`L976–981`).
- **[E] Non-goals:** do not build runtimes, use magic-keyword routing, or add LiteLLM; adapt existing CLIs/APIs, retain FSM+Designer routing, extend `LlmProvider` (`L982–983`).
- **[E] Backends:** `Sidecar`, `AnthropicApi`, `ClaudeCode`, `CodexCli`, `OpenCode`, `Hermes`, `GeminiCli`, `KiroCli`, `CopilotCli`, `Ollama`, `Vllm` (`L984–985`).
- **[E] Adapter:** `complete`, `stream`, `capabilities`, `health_check`; verify vtable compatibility and alias to `LlmProvider` if compatible (`L986–987`).
- **[E] Factory config:** CLI path, base URL, secret API key, model, permission mode (`AcceptEdits|Plan|DangerouslySkipPermissions|Default`), profile, CWD, timeouts (`L987–989`).
- **[E] Per-role hybrid:** `PerRoleRuntime {role,backend,fallback_chain}` assigns arbitrary runtime per role (`L990–991`).
- **[E] Resolution error:** unknown candidate returns candidate plus full supported-name list (`L992–993`).
- **[E] 4M.0 provenance:** ingest Ouroboros source and write `workflow/papers/ouroboros-runtime-interface.md` (`L994–996`).
- **[E] 4M.1 skeleton/config:** backend enum, adapter or alias, factory config, permission/error/role/capability/health types, stub resolve/create, `[runtime] backend` and `[agents.<role>.runtime]` schema (`L997–1000`).

## Interactions and dependencies

- **[E] Phase-0 substrate:** the nine-crate split creates the dependency boundaries used by every later phase; `ClockProvider`/`EntropySource`, `RuntimeContext`, the provider trait, minimal FSM, visibility filter, session format, and contract crate are foundational (`L24–51`).
- **[E] Retrieval ladder:** 1A DKG supplies symbols/edges to 1B chunking fallback alignment, 1D snapshots/PageRank, 1E graph anchoring, 3F AST evaluation, 3G contract context, and 3K AST chunking (`L57–76, L89–90, L122–165, L524–534, L555–585, L250–271`).
- **[E] Retrieval assembly:** 1E composes 1B BM25 + 1C vectors + 1D PageRank, leaves explicit 1G rerank and 3E token-fit stubs, and 3K later hardens the assembled pipeline (`L143–166, L250–271`).
- **[E] Resilience dependency fix:** 1I cannot start until 1E and 1G; the missing graph edge `1G→1I` must be added first (`L227–228`).
- **[E] RAG hardening prerequisites:** 3K depends on 1E+1G+3J because it uses the assembled retrieval/reranking path and sidecar generation for HyDE (`L250–260`).
- **[E] Speculative decoding revisit:** 1F is dropped on Ollama but explicitly reopened when the 3J vLLM backend makes layer-level control possible (`L168–181, L646–664`).
- **[E] Shared model-runtime degradation:** 1C/1G/1I circuit breakers emit the same SSB `knowledge_degradation` event; readiness gates the full FSM (`L114–117, L199–202, L230–246, L404–405`).
- **[E] Reasoning evolution:** 2A freezes trait methods for 2B; 2B breaks/implements their signatures; 2C layers role config; 2D adds safety/feasibility methods (`L277–385`).
- **[E] Reasoning dependency boundary:** 2A is core-only; 2B deliberately adds tools without allowing a tools→reasoning cycle or knowledge transitively; 4A later replaces direct network verification with IPC (`L288–290, L305–330, L700–702`).
- **[E] Budget authority handoff:** 2C owns in-process budgets through Phase 3; 4A demotes it to advisory and makes the external orchestrator authoritative (`L360–361, L700–702`).
- **[E] Feasibility escalation:** 2D's blocked inline check signals a major replan whose actual SSB/FSM emission arrives in 3A+ (`L381–384, L397–408`).
- **[E] FSM evidence composition:** transitions consume knowledge readiness (1I), plan grounding (2B), expected failing tests and fail→pass results, auditor blocks, and DKG/changelog completion (`L404–409`).
- **[E] SSB backbone:** retrieval mode changes, GRC traces, phase transitions, plans, conflicts, checkpoints, channel clears, context reductions, findings, model swaps, phase completion, interventions, and metrics all converge on 3B's ordered journal (`L114–117, L199–200, L235–246, L416–439 and downstream references`).
- **[E] Deterministic parallelism:** 3D schedules concurrently but applies writes in SSB sequence order; 3C spawn-parallel is a sequential stub until 3D upgrades it (`L447–449, L474–501`).
- **[E] Checkpoint composition:** 3C per-agent checkpoints + 3D channel checkpoint/replay + 3H session snapshots/handoffs + 4E navigator state form layered recovery; all avoid nonserializable untracked state (`L457–464, L495–498, L587–614, L828`).
- **[E] Context composition:** 3E's eight slots consume PageRank/retrieval output, SSB entries, GRC history, and 3G skeleton/detail; 3H preserves cache-prefix slots 1–3 and trims tool results only (`L503–520, L555–574, L602–606`).
- **[E] Rules dependency direction:** rules consumes a knowledge-implemented `AstEvaluator`; orchestrator owns discovery/normalization and SSB writes; the originating auditor cannot approve its own rule (`L523–552`).
- **[E] Plan splitting contract:** 3G and 4E both cap split depth at two to match 4G autonomous splitting; 4G needs all child results and therefore uses sequential merge rather than 3H's pick-one winner branches (`L580–583, L815–817, L860–862`).
- **[E] Plan-to-execution chain:** 4D Research+Designer creates grounded `PipelineSpec` and recursive decomposition; 4E deterministically builds/repairs/sizes it; 4F probes novel work/resources; 4G advances the resulting validated tree (`L770–870`).
- **[E] Feasibility executor:** 4F reuses 4A `AgentDispatchConfig` and IPC restrictions rather than trusting the generic `AgentRole` (`L711–714, L830–841`).
- **[E] Integration-test dependency:** 4B's tree-grained runner waits for 4E `DecompositionTree` types; its time limits match 4E's four integration levels (`L723–725, L741–743, L819–828`).
- **[E] ShortCoder placement:** tool compression is defined in 4B and invoked from 4C post-hooks in orchestrator after an agent response, before auditor visibility (`L718–719, L734, L750–752`).
- **[E] Isolation/control:** 4A makes inference, tools, grounding, budgets, and whitelists orchestrator-owned; 4B and 4C execute tools/hooks on the trusted side of that boundary (`L688–768`).
- **[E] Autonomous compatibility:** 4G wraps 3A's gate rather than altering its FSM; interactive behavior is invariant; high-risk and major-replan human control remains (`L847–870, L1032–1034`).
- **[E] Model substitution:** 3C declares but disables `SubstituteModel`; 3J builds providers/pool; 4G supplies fit checking and fallback iteration; 4M generalizes the backend interface and per-role chains (`L465–468, L646–681, L863–873, L976–1000`).
- **[E] Self-mod inputs and protection:** 4H consumes normalized `PipelineOutcome`/test/security/retrieval telemetry but is fenced by 3F immutable rules and 4A's dispatch+IPC+filesystem+git protections (`L536–547, L696–710, L875–910`).
- **[E] Conditional media path:** 4D `MediaSpec` and 4E visual-adapter mapping decide whether 4I is lazily instantiated; deterministic Tier 1 veto remains authoritative over VLM advice (`L782–784, L812–814, L912–928`).
- **[E] Conditional Empire path:** 4J is behind a feature gate and degrades gracefully; it uses 4D/4E PipelineSpec enforcement to repair verified Empire issues (`L930–950`).
- **[E] Dashboard read/write split:** 4K queries SSB through read-only WAL connections and writes user decisions as typed intervention events; no alternate HTTP/control plane bypasses FSM classification (`L952–974`).
- **[E] 3J→4M refactor:** 3J's bespoke `InferenceDispatcher` and backend mode are the legacy surface that 4M converts into `RuntimeAdapter`/factory/per-role runtime configuration (`L675–681, L976–1000`).
- **[I] Full completion path:** a task is classified by FSM, grounded/researched, converted to a typed/sized/probed tree, executed by isolated role-configured agents through orchestrator-owned tools, journaled in SSB, verified at baby/step/phase/concept and FSM gates, optionally media-validated, memorized/handoff-checkpointed, and only then complete. This is the composition implied by the explicit phase interfaces; no single MASTER passage states the whole workflow.

## Constraints and invariants

### System-wide (`L1002–1034`)

- **[E] Determinism/entropy quarantine:** all phases must use injected time/randomness; CI grep, DST reference model, crash replay, and 100-run identity checks enforce it (`L1006–1008`).
- **[E] TDD ordering:** every implementation phase creates failing tests before implementation and confirms the failures are correct (`L1010–1011`).
- **[E] Anti-stale choices:** prefer Blake3 over Blake2s where the anti-stale rule applies, `tracing` over `log`, `thiserror`/`anyhow` over boxed dynamic errors, `deadpool` over r2d2 for new code, and `globset` over glob; deeper policy is in `CLAUDE.md` (`L1013–1014`).
- **[Q] Hash coexistence:** the MASTER still explicitly requires Blake2s in 1A/1C/3F/4C while its cross-cutting anti-stale line says Blake3 over Blake2s. Treat phase-local named hashes as explicit exceptions until original plans clarify (`L74, L96–103, L544, L762, L1013–1014`).
- **[E] Grounding hygiene:** every phase classifies claims as `GROUNDED|VERIFIED|UNGROUNDED` with cited source; fixtures/bench corpora are committed for reproducibility (`L1016–1018`).
- **[E] Tool/rule filtering:** enforce at `ToolRegistry::execute`, not in prompts; hidden and missing paths are observationally identical (`L1020–1022`).
- **[E] Isolation authority:** after 4A agents have no network, model client, or tools; orchestrator owns GRC budget, inference, tools, and grounding over capped IPC (`L1024–1026`).
- **[E] Self-mod immutability:** security/finance `mutable=0` rules cannot be automatically modified; auditor proposals need independent approval; 4H may change only closed enum keys, not source code (`L1028–1030`).
- **[E] Graduation:** interactive first, autonomous opt-in; no added FSM state; high risk and major replan pause in both; within-execution tool review belongs to auditor (`L1032–1034`).

### Numeric and structural invariants

- **[E] Build baseline/exit:** Windows 684-test baseline; zero compile warnings/errors and zero failures before Phase 0 (`L10–22`).
- **[E] Thin entry point/crate map:** nine named crates and `main.rs<500` (`L24–51`).
- **[E] Repro run:** seed 42, single test thread, diff zero (`L46–49`).
- **[E] DKG:** WAL, cascading typed edges, logged parse skip, indexed name/file/kind (`L62–76`).
- **[E] BM25:** untokenized path, atomic delete+insert, no binary ingestion, no orphans (`L78–93`).
- **[E] Vector:** max sequence 512, cache hit avoids embed, missing model never crashes, cache hit target ≥90% (`L95–120`).
- **[E] PageRank:** density >.3 forces insertion fallback; personalization sums to one; clamp nonfinite values; ≤10 fit iterations (`L122–141`).
- **[E] Retrieval:** no hot-path LLM for CatRAG; A-RAG loops≤15/tokens≤128K; target savings/latency gates (`L143–166`).
- **[E] Statistical feature gates:** SPLADE requires ≥5% mean recall improvement and `p<.05`; speculative decoding originally required >1.3× and <3 GB (`L168–179, L208–225`).
- **[E] Circuit behavior:** clocked breakers and deterministic jitter; fresh session on probe; half-open recovery is evidence-driven; degraded readiness means BM25-only (`L199–202, L227–248`).
- **[E] GRC:** first step Observe; default chain; Status/Converge always available; budget checked before saturating increment; enum order carries meaning (`L277–299`).
- **[E] Verification:** six binary methods, empty plan passes, any ungrounded plan blocks, local test timeout 30 s (`L301–330`).
- **[E] Role config:** overrides only add transitions; auditor cannot converge finding-free; assigning config resets topic (`L332–362`).
- **[E] CSQ:** all critical assumptions require guards; exactly two inline checks; false positive target zero (`L364–385`).
- **[E] FSM:** tool permissions depend on state; redirect target must be valid; reject leaves state unchanged; interactive has no timeout (`L391–414`).
- **[E] SSB:** one writer/binding; append-only typed rows; commit before broadcast and result dispatch; drain on shutdown; deterministic dump (`L416–439`).
- **[E] Agent:** UUID from entropy provider; hidden paths indistinguishable; same UUID on recovery; corrupt checkpoint is recoverable; budget death does not count as restart; spiral recovery uses pre-spiral checkpoint (`L441–472`).
- **[E] BSP:** no concurrent `BinaryOp`; sequence orders writes; untracked never serializes; checkpoint only after gate; deferred nodes cannot cascade (`L474–501`).
- **[E] Context:** sum never exceeds model window; SSB/GRC/output floors; deterministic prefix hashes; terminal overflow escalates/splits (`L503–520`).
- **[E] Rules:** predicates compile at admission; composites depth≤8; no LLM in deterministic tiers; immutable rules are protected; self-approval impossible (`L523–553`).
- **[E] Plans:** skeleton ≤4096 exact tokens; structured—not prose—coherence checks; contract/version changes propagate; cycles forbidden; split depth≤2 (`L555–585`).
- **[E] Handoff:** session DAG immutable; no physical branch deletion; user/assistant/cache-prefix never stubbed; stub must resolve+hash or become explicit error (`L587–614`).
- **[E] DST:** oracle <30% SUT LOC; transport-level replay; CI state-machine bounds; deterministic rate 100% (`L616–644`).
- **[E] Provider/GPU:** 1 GB multiplexing margin; model-specific guards; provider failure warns/falls back (`L646–681`).
- **[E] IPC:** 10 MB frame hard cap plus lower per-type caps; 0700 socket dir and peer credentials; no agent provider dependency; phase marker fsync before unload (`L688–717`).
- **[E] Tools/hooks:** auditor-only validation; bounded reduction/retries; ShortCoder AST-equivalent; generic sanitized block errors; T3 never runs after a block (`L721–767`).
- **[E] Research:** deterministic evidence weights and five-tier source quality; prune only after three low-gain iterations; package registry verification (`L770–798`).
- **[E] Builder:** pure deterministic assembly; repair≤10; substitute only compatible I/O/category; eight checks; phase work≤60%; depth≤2 (`L800–828`).
- **[E] Probe:** skip only on conjunctive zero-novel + all-resource margin; stubs <50 LOC; max three alternatives; strict dual timeout (`L830–845`).
- **[E] Autonomy:** evidence precedes advance; high risk/major replan stays human; ten advances/phase; four subtasks/depth two; 50 baby steps/concept then human (`L847–873`).
- **[E] Self-mod:** generator never evaluator; scores normalized; mode-separated parent selection; hard reject regression; layer-3 human; auto-revert threshold; lock list hidden (`L875–910`).
- **[E] Media:** deterministic tier vetoes advisory model; CPU metrics with epsilon; bounded concurrency; unavailable VLM degrades rather than fabricates verdict (`L912–928`).
- **[E] Empire:** explicit size/versioned newline-JSON transport and narrow four-capability surface, no internal-state leakage (`L930–950`).
- **[E] Dashboard:** SSB is read-only from viewer; no network server; every input classified by FSM; typed/paginated projections (`L952–974`).
- **[E] Runtime adapters:** known enum rather than freeform magic routing; API keys typed secret; resolver returns supported set; role-specific fallbacks (`L976–1000`).

## Open and deferred items

- **[D] Master granularity:** individual phase files still own the exhaustive pre-gates/tasks/checks/adversarial/acceptance details (`L1–4`).
- **[D] 1A language expansion:** Python/TypeScript/Go are later than initial Rust (`L57–60`).
- **[D] 1C TurboQuant:** raw f32 now; compression later (`L119–120`).
- **[C]/[D] 1F:** Phase-1 speculative decoding and prompt lookup dropped; vLLM revisit remains (`L168–181`).
- **[D] 1H:** entirely conditional; early drop is valid completion and stays disabled by default (`L208–225`).
- **[D] 3K:** LLM edge scoring rejected/deferred on cost (`L268–270`).
- **[D] 2A:** `verify_claim`/`ground_plan` initially unimplemented pending 2B (`L285–286`).
- **[D] 2B:** WEB/API/API_REGISTRY remain stubbed until 4A IPC proxy; phase ends at 3/6 methods (`L301–330`).
- **[D] 2D:** web/grep/registry feasibility methods wait for 4F; orchestrator replan emission waits for 3A+ (`L368–384`).
- **[D] 3A:** feasibility-state evidence gates wait for 4F (`L404–409`).
- **[D] 3C:** parallel spawn/direct communication are stubs until 3D; model substitution waits for 4G; Phase 3 remains Tier A (`L441–468`).
- **[Q] 3E:** numbered substeps are omitted from MASTER (`L521`).
- **[Q] 3H amendments:** branch event name, row granularity, and logical cleanup need reconciliation in original plan (`L590–592`).
- **[Q]/[D] 3I:** Windows madsim/VFS interception depend on spikes; manual scheduler may replace madsim (`L619–624`).
- **[D] 3J:** TurboQuant is FEASIBLE/DEFER; MILP is ADOPT/DROP and has Windows build risk; CPU offload is not fit for EAGLE-3 training (`L650–669`).
- **[D] 4A:** Windows fallback is explicitly weaker; ShortCoder moves to 4B (`L706–719`).
- **[C]/[D] 4B:** measurement and integration runner were deferred in recorded status because decomposition types were missing (`L723–725`).
- **[D] 4C:** skills code path and MCP bridge are outside the phase (`L768`).
- **[D] 4H:** structural FSM/role/GRC mutation remains human-only (`L885, L902`).
- **[D] 4I:** exists only when pipeline emits media; VLM absence yields degraded state (`L912–928`).
- **[D] 4J:** only with Empire source and `empire` feature; otherwise graceful degradation (`L930–950`).
- **[D] 4K:** 500 ms polling is MVP; true push is future (`L972–974`).
- **[Q] 4M completeness:** MASTER lists only 4M.0–4M.1 even though the phase goal implies adapter implementations, resolution, health, streaming, fallbacks, migrations, and tests; original `phase-4M.txt` must supply them (`L976–1000`).
- **[Q] Inventory drift:** L1037 says “38 files” but L1039–1056 enumerates 38 phase-detail files plus two split indexes, the MASTER, and `desktop.ini`; the current directory also has four extra phase-detail files. Treat MASTER's completeness claim as dated, not current (`L2, L1037–1056`).
- **[Q] Current unindexed phase details:** `phase-3J-BASELINE.txt`, `phase-3L.txt`, `phase-4D-ORACLE.txt`, `phase-4D-SURGICAL.txt` exist now but are not named in MASTER inventory; deeper agents must inspect them rather than infer content.
- **[Q] Event enumeration drift:** 3B says exactly 17 action types, 3E later uses `context_reduction`, 3H may add `branch_abandon`, and 4K adds two intervention types. The eventual canonical enum/migration needs reconciliation (`L416–423, L514–518, L590–592, L961–967`).
- **[Q] Pool guidance drift:** 3B explicitly chooses r2d2 while cross-cutting anti-stale guidance says deadpool for new code; original/implementation scope must clarify grandfathering (`L418–420, L1013–1014`).
- **[Q] Dashboard command count:** 4K.1 names eight viewer queries while 4K.2 says six Tauri wrappers; original plan must map which queries are combined or not exposed (`L961–969`).
- **[Q] “Phase 2+” wording:** 1F says revisit in Phase 2+ vLLM provider but vLLM is Phase 3J; interpret as any post-Phase-1 implementation, not a Phase-2 dependency (`L168–172, L646–656`).

## Deeper-source manifest

No file in this manifest was read for feature extraction during this assignment. Paths were resolved by filename enumeration only. Each is a handoff target for the deeper agents.

### A. Mandatory original phase-detail sweep

Base directory: `docs/specs/guide/steps/`. The MASTER says every individual phase file contains fuller `PRE_GATE`, `TASK`, `DETERMINISTIC_CHECK`, `ADVERSARIAL`, and `ACCEPTANCE` blocks (`L1–4`), so all of the following are mandatory, not optional:

- `docs/specs/guide/steps/pre-phase-0.txt` — Windows baseline/fix evidence.
- `docs/specs/guide/steps/phase-0.txt` — strip/restructure/determinism detail.
- `docs/specs/guide/steps/phase-1A.txt`
- `docs/specs/guide/steps/phase-1B.txt`
- `docs/specs/guide/steps/phase-1C.txt`
- `docs/specs/guide/steps/phase-1D.txt`
- `docs/specs/guide/steps/phase-1E.txt`
- `docs/specs/guide/steps/phase-1F.txt`
- `docs/specs/guide/steps/phase-1G.txt`
- `docs/specs/guide/steps/phase-1H.txt`
- `docs/specs/guide/steps/phase-1I.txt`
- `docs/specs/guide/steps/phase-2A.txt`
- `docs/specs/guide/steps/phase-2B.txt`
- `docs/specs/guide/steps/phase-2C.txt`
- `docs/specs/guide/steps/phase-2D.txt`
- `docs/specs/guide/steps/phase-3A.txt`
- `docs/specs/guide/steps/phase-3B.txt`
- `docs/specs/guide/steps/phase-3C.txt`
- `docs/specs/guide/steps/phase-3D.txt`
- `docs/specs/guide/steps/phase-3E.txt`
- `docs/specs/guide/steps/phase-3F.txt`
- `docs/specs/guide/steps/phase-3G.txt`
- `docs/specs/guide/steps/phase-3H.txt`
- `docs/specs/guide/steps/phase-3I.txt`
- `docs/specs/guide/steps/phase-3J.txt`
- `docs/specs/guide/steps/phase-3K.txt`
- `docs/specs/guide/steps/phase-4A.txt`
- `docs/specs/guide/steps/phase-4B.txt`
- `docs/specs/guide/steps/phase-4C.txt`
- `docs/specs/guide/steps/phase-4D.txt`
- `docs/specs/guide/steps/phase-4E.txt`
- `docs/specs/guide/steps/phase-4F.txt`
- `docs/specs/guide/steps/phase-4G.txt`
- `docs/specs/guide/steps/phase-4H.txt`
- `docs/specs/guide/steps/phase-4I.txt`
- `docs/specs/guide/steps/phase-4J.txt`
- `docs/specs/guide/steps/phase-4K.txt`
- `docs/specs/guide/steps/phase-4M.txt`

### B. Current phase files absent from the MASTER inventory

These require an explicit sweep because their current existence disproves using the dated MASTER as a complete folder manifest:

- `docs/specs/guide/steps/phase-3J-BASELINE.txt`
- `docs/specs/guide/steps/phase-3L.txt`
- `docs/specs/guide/steps/phase-4D-ORACLE.txt`
- `docs/specs/guide/steps/phase-4D-SURGICAL.txt`

For reconciliation/deduplication, the two older split indexes should also be compared, but not treated as substitutes for originals:

- `docs/specs/guide/steps/phase-index-pre0-2D.txt`
- `docs/specs/guide/steps/phase-index-3A-4J.txt`

`docs/specs/guide/steps/desktop.ini` is not a plan and needs no feature extraction.

### C. Cross-spec and decision documents explicitly referenced by the MASTER

- `docs/specs/crates/knowledge.txt` — reranker final fallback (`L201`) and likely full knowledge architecture.
- `docs/specs/crates/providers.txt` — corrected VRAM claim (`L180–181`) and `can_fit_llm` propagation (`L863–864`).
- `docs/specs/crates/reasoning.txt` — role configurations (`L337–340`).
- `docs/specs/crates/orchestrator.txt` — FSM transitions/gates, reasoning traces, conceptual plan query, and split semantics (`L394–408, L455–456, L558–559, L860–862`).
- `docs/specs/crates/rules.txt` — auditor-proposal trust rule (`L544–547, L762–763`).
- `docs/specs/architecture-overview.txt` — context allocation latency target (`L519–520`).
- `docs/specs/.old/decisions/technical-analysis-v3.txt` — untracked checkpoint concern (`L460–461`); archived location means the current replacement, if any, should also be located.
- `docs/specs/integration/pipeline-factory.txt` — complexity presets and full pipeline-factory design (`L785–788`).
- `workflow/papers/flashresearch-interface.md` — research interface cited alongside the factory spec (`L785`).
- `docs/specs/integration/empire.txt` — six-step Empire plan (`L942–950`).
- `workflow/reasoning/phase-1C-decisions.md` — tensor-format decision record (`L112–113`).
- `workflow/papers/ouroboros-runtime-interface.md` — 4M source-ingestion output (`L994–996`).
- `CLAUDE.md` — anti-stale-training rules (`L1013–1014`).
- **[Q] Unresolved exact file:** `SKILL.md` is referenced generically for FSM workflow patterns (`L394–395`), but no directory is supplied and the repository has multiple skill files; `phase-3A.txt` should identify the intended one.

### D. Referenced implementation/config/fixture artifacts worth a later conformance sweep

These are not substitutes for plan reads, but each carries a feature contract named in the MASTER:

- `.elai_cc/metrics/pre_phase_0.toml`, `.elai_cc/Cargo.toml`, `migration_manifest.toml`, `elai_cc.toml` (`L20, L27–40 and later config references`).
- `.elai_cc/crates/`, `.elai_cc/crates/knowledge/`, `.elai_cc/crates/dashboard/`, `.elai_cc/projects/{id}/bm25/`, `.elai_cc/models/onnx/` (`L26, L59, L80, L98, L968`).
- `index_version.toml`, `phase_1A.toml`, `phase_1B.toml`, `phase_1D.toml`, `phase_3K_rag.toml`, `phase_3I.toml` (`L67, L76, L93, L141, L264, L644`).
- `tests/fixtures/reranker_benchmark.json`, `scripts/quantize_cross_encoder.py`, `rules_seed.db`, and `proptest-regressions/` (`L205–206, L548–552, L630–633`).
- `.elai_cc/sessions/{sid}/.fsm-state.msgpack`, per-agent checkpoint files, `.elai_cc/pipelines/{id}/skeleton.toml`, and per-phase TOML (`L397–399, L457–461, L555–569`).
- `.elai_cc/rust/`, `runtime/mcp_stdio.rs`, `runtime/mcp_tool_bridge.rs`, `rusty-claude-cli/tests/mock_parity_harness.rs`, `aider/repomap.py`, `tokenizer.json`, `splade.rs`, `catrag.rs`, `arag.rs`, `pipeline.rs` (`L10–18, L37, L104–154, L195, L214`).
- `workforce/assembly/runtime/actions.py`, `brain.json`, and the Empire source tree (`L930–947`).
- `~/vllm-env/` and the composed Python sidecar/CLI launch path (`L653–681`).

### E. External named sources whose adaptations should be verified in the originals

- CatRAG `arXiv:2602.01965` and A-RAG `arXiv:2602.03442` (`L143–160`).
- TurboQuant `arXiv:2504.19874` (`L119–120, L661–662`).
- HyDE `arXiv:2212.10496` (`L258–260`).
- CMV `arXiv:2602.22402` (`L587–589`).
- DoLa `arXiv:2309.03883` and ActAdd `arXiv:2308.10248` (`L658–664`).
- ShortCoder `arXiv:2601.09703` (`L734`).
- FlashResearch `arXiv:2510.05145` (`L770–790`).
- AutoB2G `arXiv:2603.26005`, plus the explicitly rejected SOCIA LLM-repair route (`L800–807`).
- DGM/CEL/DarwinNet-derived self-modification and DGM Appendix H objective-hacking trigger (`L875–910`).
- Ouroboros source repository `github.com/Q00/ouroboros` (MIT) (`L976–996`).

## Coverage conclusion

The atomic inventory above accounts for every substantive line block in the MASTER: metadata (`L1–4`), all named phases from PRE-PHASE 0 through 4M (`L10–1000`), every cross-cutting engineering theme (`L1002–1034`), and the file inventory (`L1036–1056`). Explicit requirements are separated from recorded completion, deferral/conditional status, inferred composition, and unresolved questions. The next required work is the original-file sweep in sections A–C; the dated MASTER alone cannot establish “all files/all features” because the current directory contains unindexed phase details.
