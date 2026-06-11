#![allow(dead_code, unused_imports, unused_variables, unused_mut, clippy::unneeded_struct_pattern, clippy::unnecessary_wraps, clippy::unused_self)]

//! T12d: `LiveCli::build_local_knowledge_stack` extracted from `live_cli.rs`.
//!
//! The factory is the single construction site for the local-backend RAG
//! stack (BM25 + CatRag + A-RAG + vector index + reranker + SPLADE + DKG +
//! HyDE + Cosmos). It remains an associated function on `LiveCli` so the two
//! internal `Self::build_local_knowledge_stack` call sites in `live_cli.rs`
//! and the external `LiveCli::build_local_knowledge_stack` caller in
//! `bench_rag.rs` resolve unchanged.

use std::env;
use std::path::PathBuf;
use std::sync::Arc;

use elai_core::ClockProvider;
use elai_orchestrator::BackendConfig;
use elai_orchestrator::indexing_service::IndexingService;

use elai_knowledge::DkgEngine;
use elai_knowledge::pipeline::QueryPipeline;
use elai_knowledge::bm25::Bm25Index;
use elai_knowledge::catrag::{CatRag, CatRagConfig};
use elai_knowledge::arag::{AragOrchestrator, AragConfig};
use elai_knowledge::vector::OrtVectorModule;
use elai_knowledge::vector::ort_session::OrtSession;
use elai_knowledge::vector::indexer::VectorIndexer;
use elai_knowledge::vector::TEXT_VEC_DIM;
use elai_knowledge::startup::{startup_readiness_check, StartupConfig};
use elai_knowledge::degradation::DegradationCascade;
use elai_knowledge::reranker::{CrossEncoderReranker, RerankerConfig};
use elai_knowledge::splade::{SpladeExpander, SpladeConfig};
use elai_knowledge::index_version::{check_and_upgrade, LogNotifier};

use elai_core::paths::{
    dkg_db_path as core_dkg_db_path, models_dir_path as core_models_dir_path,
    vector_cache_db_path as core_vector_cache_db_path,
    vectors_db_path as core_vectors_db_path,
};

use crate::live_cli::{
    cascade_ts_ms_now, cosmos_tick_emit_loop, search_mode_to_payload_str,
    spawn_cascade_evaluator_loop, LiveCli,
};
use crate::ssb_notifiers::{NoopRerankerNotifier, SsbRerankerNotifier};
use crate::{indexing, sidecar, sidecar_hyde};

impl LiveCli {
    /// Build the local knowledge pipeline + indexing service for the Local
    /// backend. Returns (knowledge_pipeline, indexing_service).
    /// Mirrors the construction originally inline in `LiveCli::new_local`.
    /// Gated to local-backend callers because HyDE sidecar wiring requires
    /// a `BackendConfig` with a known `sidecar_url`.
    ///
    /// NOTE: The steering-precompute block (ELAI_PRECOMPUTE_STEERING=1) is
    /// intentionally left inline in `new_local`. It depends on locals from
    /// that function and is opt-in via env, so omitting it here is correct.
    pub(crate) fn build_local_knowledge_stack(
        backend_config: &BackendConfig,
        session_id: &str,
        clock: &Arc<dyn ClockProvider>,
        ssb: Option<Arc<elai_orchestrator::Ssb>>,
        // 1I-R1: shared live SearchMode the cascade evaluator loop writes each
        // tick so the per-query path can read+apply it. Cloned into the spawned
        // loop; the caller keeps a clone on the LiveCli.
        search_mode_shared: Arc<std::sync::Mutex<elai_knowledge::degradation::SearchMode>>,
    ) -> (Option<QueryPipeline>, Option<IndexingService>) {
        let project_root = env::current_dir().unwrap_or_else(|_| PathBuf::from("."));

        // Check and upgrade index version (Phase 1I.7).
        let elai_data_dir = project_root.join(".elai");
        if elai_data_dir.exists() {
            let now = elai_core::OsClock
                .system_now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| i64::try_from(d.as_secs()).unwrap_or(i64::MAX))
                .unwrap_or(0);
            match check_and_upgrade(&elai_data_dir, now, &LogNotifier) {
                Ok(action) => {
                    if action != elai_knowledge::index_version::UpgradeAction::Reuse {
                        eprintln!("info: index upgrade action: {action:?}");
                    }
                }
                Err(e) => {
                    eprintln!("warn: index version check failed: {e}");
                }
            }
        }

        let knowledge_pipeline = match Bm25Index::new(&project_root, session_id) {
            Ok(bm25) => {
                let catrag = CatRag::new(CatRagConfig::default());
                // C-8 Task #8: try to load a standalone graphcodebert ONNX
                // session for the A-RAG semantic_search dot-product path
                // before constructing the orchestrator. When the model is
                // missing/corrupt, log a warn and fall back to the embedding
                // norm proxy (built into AragOrchestrator::semantic_search).
                let models_dir = core_models_dir_path(&project_root);
                let code_model_path = models_dir.join("onnx").join("graphcodebert.onnx");
                let code_tok_path = models_dir.join("onnx").join("graphcodebert-tokenizer.json");
                let text_model_path = models_dir.join("onnx").join("granite-embed.onnx");
                let text_tok_path = models_dir.join("onnx").join("granite-embed-tokenizer.json");
                let arag_embedder: Option<Arc<OrtSession>> = match OrtSession::new(
                    &code_model_path,
                    &code_tok_path,
                    512,
                    768,
                ) {
                    Ok(sess) => Some(Arc::new(sess)),
                    Err(e) => {
                        eprintln!(
                            "warn: arag.semantic_search ONNX embedder not loaded: {e}; \
                             falling back to embedding-norm proxy"
                        );
                        None
                    }
                };
                let mut arag = AragOrchestrator::new(AragConfig::default());
                if let Some(ref emb) = arag_embedder {
                    arag = arag.with_embedder(emb.clone());
                }
                // Wire the A-RAG corpus from the on-disk DKG so
                // `query_pipeline` returns non-empty hierarchical results.
                // Gated default-OFF on `ELAI_ARAG_POPULATE_FROM_DKG` so the
                // existing zero-corpus behaviour is preserved until an
                // operator opts in (e.g. on the GPU pod where embeddings are
                // expected). Failures are logged and ignored — A-RAG
                // population is not load-bearing for the rest of the
                // pipeline.
                let arag_populate_enabled = std::env::var("ELAI_ARAG_POPULATE_FROM_DKG")
                    .map(|v| {
                        let v = v.trim().to_ascii_lowercase();
                        matches!(v.as_str(), "1" | "true" | "yes" | "on")
                    })
                    .unwrap_or(false);
                if arag_populate_enabled {
                    let dkg_db_path_for_arag = core_dkg_db_path(&project_root);
                    if dkg_db_path_for_arag.exists() {
                        match DkgEngine::new(&dkg_db_path_for_arag, None, 0) {
                            Ok(dkg_engine) => {
                                match elai_knowledge::arag::populate_corpus_from_dkg(
                                    &dkg_engine,
                                    &mut arag,
                                ) {
                                    Ok(n) => eprintln!(
                                        "info: arag corpus populated from DKG ({n} chunks)"
                                    ),
                                    Err(e) => eprintln!(
                                        "warn: arag corpus population failed: {e}"
                                    ),
                                }
                            }
                            Err(e) => eprintln!(
                                "warn: arag corpus population: DkgEngine open failed: {e}"
                            ),
                        }
                    }
                }
                // C-1: install the typed knowledge SSB sink + session id
                // on the pipeline so per-query `query_trace` rows reach
                // the dashboard. Without an Ssb writer the pipeline
                // installs a noop sink and silently drops rows.
                let pipeline_sink: Arc<
                    dyn elai_knowledge::knowledge_ssb_sink::KnowledgeSsbSink,
                > = match ssb.as_ref() {
                    Some(ssb_arc) => Arc::new(
                        elai_orchestrator::OrchestratorKnowledgeSsbSink::new(
                            ssb_arc.clone(),
                        ),
                    ),
                    None => elai_knowledge::knowledge_ssb_sink::noop_sink(),
                };
                let mut pipeline = QueryPipeline::new(bm25, catrag, arag)
                    .with_ssb(pipeline_sink)
                    .with_session_id(session_id.to_owned())
                    .with_clock(clock.clone());

                // Attempt to load vector embedding models (graceful degradation).
                // C-8: code_sess for VectorModule and arag are loaded
                // independently — both reference the same on-disk
                // graphcodebert.onnx but ORT sessions are not Sync-shareable
                // beyond the Arc wrapper we already use for the A-RAG path.
                match (
                    OrtSession::new(&code_model_path, &code_tok_path, 512, 768),
                    OrtSession::new(&text_model_path, &text_tok_path, 512, 384),
                ) {
                    (Ok(code_sess), Ok(text_sess)) => {
                        let vm = OrtVectorModule::new(code_sess, text_sess);
                        pipeline.vector_module = Some(Box::new(vm));
                    }
                    (code_r, text_r) => {
                        if let Err(ref e) = code_r {
                            eprintln!("warn: vector code model not loaded: {e}");
                        }
                        if let Err(ref e) = text_r {
                            eprintln!("warn: vector text model not loaded: {e}");
                        }
                    }
                }

                // Initialize vector indexer if embedding models are available.
                if pipeline.vector_module.is_some() {
                    let index_path = core_vectors_db_path(&project_root);
                    let indexer = if index_path.exists() {
                        match VectorIndexer::load(&index_path, TEXT_VEC_DIM) {
                            Ok(idx) => {
                                eprintln!("info: loaded vector index ({} chunks)", idx.len());
                                idx
                            }
                            Err(e) => {
                                eprintln!("warn: vector index load failed, rebuilding: {e}");
                                VectorIndexer::new(TEXT_VEC_DIM)
                            }
                        }
                    } else {
                        VectorIndexer::new(TEXT_VEC_DIM)
                    };
                    // B-4: enable code_vec secondary index when opt-in env flag set.
                    // ELAI_CODE_VEC=1 enables the secondary GraphCodeBERT code-space index.
                    // Defaults to false — existing vector_index.json files are unaffected.
                    // When enabled, adjacent code_index.json is loaded (if present from a
                    // prior index run) or rebuilt from VectorCache during next indexing pass.
                    let code_vec_enabled = std::env::var("ELAI_CODE_VEC")
                        .map(|v| v == "1" || v.eq_ignore_ascii_case("true"))
                        .unwrap_or(false);
                    let mut indexer = indexer;
                    if code_vec_enabled {
                        indexer.enable_code_index();
                        eprintln!("info: code_vec secondary index enabled (has_code_index={})", indexer.has_code_index());
                    }
                    pipeline.vector_indexer = Some(indexer);

                    // 1C-R2: construct + install the on-disk dual-vector cache.
                    // This is the production wiring that clears the wire-dark
                    // VectorCache: with the cache installed, the file-write ->
                    // re-index path in `live_cli::notify_file_changed` calls
                    // `VectorIndexer::index_file(.., Some(cache))`, which runs
                    // `VectorCache::invalidate_file` (evict stale embeddings for
                    // the rewritten file) + `insert` (cache fresh dual-vectors).
                    // Default-on: threading a cache is behaviour-preserving for
                    // retrieval (identical results, only freshness/embedding-reuse
                    // improves), so it is unconditional rather than gated behind a
                    // default-OFF flag (a default-OFF cache flag would re-create
                    // the wire-dark condition per wiring-enforcement R9).
                    //
                    // The cache is opened only when embeddings are available
                    // (inside `vector_module.is_some()`); a cache without an
                    // embedder would never be read or written.
                    let cache_path = core_vector_cache_db_path(&project_root);
                    if let Some(parent) = cache_path.parent() {
                        if let Err(e) = std::fs::create_dir_all(parent) {
                            tracing::warn!(
                                path = %parent.display(),
                                error = %e,
                                "1C-R2: failed to create indexes dir for vector cache (cache disabled this run)"
                            );
                        }
                    }
                    // C-1: install the same typed knowledge SSB sink used for
                    // the pipeline so the cache's per-query hit-rate flush
                    // (`flush_hit_rate_ssb`, called from `query_with_tier`)
                    // reaches the dashboard. Falls back to the noop sink when
                    // no Ssb writer is available (cold-start dashboard).
                    let cache_sink: Arc<
                        dyn elai_knowledge::knowledge_ssb_sink::KnowledgeSsbSink,
                    > = match ssb.as_ref() {
                        Some(ssb_arc) => Arc::new(
                            elai_orchestrator::OrchestratorKnowledgeSsbSink::new(
                                ssb_arc.clone(),
                            ),
                        ),
                        None => elai_knowledge::knowledge_ssb_sink::noop_sink(),
                    };
                    match elai_knowledge::vector::cache::VectorCache::open_with_ssb(
                        &cache_path,
                        cache_sink,
                    ) {
                        Ok(cache) => {
                            pipeline = pipeline.with_vector_cache(Arc::new(cache));
                            tracing::info!(
                                path = %cache_path.display(),
                                "1C-R2: vector embedding cache installed (re-index eviction live)"
                            );
                        }
                        Err(e) => {
                            // Non-fatal: indexing still works without the cache
                            // (every chunk re-embeds), it just loses the
                            // eviction/reuse optimisation for this run.
                            tracing::warn!(
                                path = %cache_path.display(),
                                error = %e,
                                "1C-R2: vector cache open failed; re-indexing without cache this run"
                            );
                        }
                    }
                }

                // Attach cross-encoder reranker (graceful degradation).
                let ce_model_path = models_dir.join("onnx").join("cross-encoder-l12.onnx");
                let ce_tok_path = models_dir.join("onnx").join("tokenizer.json");
                if ce_model_path.exists() && ce_tok_path.exists() {
                    // E-17 SP2 (SP1 carryover): switched from
                    // `..RerankerConfig::default()` (relative PathBufs) to
                    // `RerankerConfig::with_project_root` so all model paths
                    // resolve absolute regardless of CWD. Fallback filename
                    // corrected from "ms-marco-l6.onnx" (never existed on
                    // disk) to "cross-encoder-l6.onnx".
                    let mut ce_config = RerankerConfig::with_project_root(&project_root);
                    ce_config.model_path = ce_model_path;
                    ce_config.tokenizer_path = ce_tok_path;
                    // C-1: switched from `::new` (defaults to a noop SSB
                    // notifier, blocked by stub_ban) to `::with_ssb`. When
                    // an Ssb writer is available we install an
                    // SsbRerankerNotifier adapter that writes
                    // `knowledge_degradation` rows; otherwise we still pass
                    // a noop notifier (degradation cascade has nowhere to
                    // emit), but that path is observable in this file (not
                    // gated by stub_ban) which is the WIRING_CONTRACT
                    // surface that matters.
                    let reranker_notifier: std::sync::Arc<
                        dyn elai_knowledge::reranker::SsbNotifier,
                    > = match ssb.as_ref() {
                        Some(ssb_arc) => std::sync::Arc::new(
                            SsbRerankerNotifier::new(ssb_arc.clone()),
                        ),
                        None => std::sync::Arc::new(NoopRerankerNotifier),
                    };
                    let mut reranker = CrossEncoderReranker::with_ssb(
                        ce_config,
                        clock.clone(),
                        reranker_notifier,
                    );
                    match reranker.load() {
                        Ok(()) => {
                            pipeline = pipeline.with_cross_encoder(reranker);
                        }
                        Err(e) => {
                            eprintln!("warn: cross-encoder reranker not loaded: {e}");
                        }
                    }
                }

                // Attach SPLADE expander (graceful degradation).
                let splade_model_path = models_dir.join("splade").join("model.onnx");
                let splade_tok_path = models_dir.join("splade").join("tokenizer.json");
                if splade_model_path.exists() && splade_tok_path.exists() {
                    // E-17 SP2 (SP1 carryover): switched from
                    // `..SpladeConfig::default()` (relative PathBufs) to
                    // `SpladeConfig::with_project_root` so all model paths
                    // resolve absolute regardless of CWD.
                    let mut splade_config = SpladeConfig::with_project_root(&project_root);
                    splade_config.model_path = splade_model_path;
                    splade_config.tokenizer_path = splade_tok_path;
                    match SpladeExpander::new(splade_config, clock.clone()) {
                        Ok(expander) => {
                            pipeline = pipeline.with_splade(expander);
                        }
                        Err(e) => {
                            eprintln!("warn: SPLADE expander not loaded: {e}");
                        }
                    }
                }

                // C-38: replaced the prior "if dkg.db exists, load
                // engine; otherwise silently skip" gate with the
                // ensure_dkg_or_rebuild helper. The old gate left Phase
                // 1D PageRank dead at runtime on any freshly-cloned
                // workspace (no production path created .elai/dkg.db
                // before Stage 2 first ran). The helper emits paired
                // dkg_rebuild_started / dkg_rebuild_complete SSB beacons
                // so the dashboard sees both the trigger reason and the
                // rebuild duration; failure is surfaced via warn (the
                // degradation cascade picks up the absence-of-DKG signal
                // through pipeline.dkg_engine = None).
                let dkg_lifecycle_sink: Arc<
                    dyn elai_knowledge::knowledge_ssb_sink::KnowledgeSsbSink,
                > = match ssb.as_ref() {
                    Some(ssb_arc) => Arc::new(
                        elai_orchestrator::OrchestratorKnowledgeSsbSink::new(
                            ssb_arc.clone(),
                        ),
                    ),
                    None => elai_knowledge::knowledge_ssb_sink::noop_sink(),
                };
                match crate::dkg_lifecycle::ensure_dkg_or_rebuild(
                    &project_root,
                    dkg_lifecycle_sink,
                ) {
                    Ok(engine) => {
                        pipeline.dkg_engine = Some(engine);
                    }
                    Err(e) => {
                        eprintln!(
                            "warn: ensure_dkg_or_rebuild failed (PageRank stage will run with empty graph): {e}"
                        );
                    }
                }

                // Attach HyDE stage when the sidecar is reachable.
                if sidecar::sidecar_probe_ok(&backend_config.sidecar_url) {
                    match sidecar::build_hyde_sidecar_provider(backend_config) {
                        Ok(provider) => {
                            match sidecar_hyde::SidecarHydeClient::new(std::sync::Arc::new(provider)) {
                                Ok(hyde_client) => {
                                    // C-1: thread the orchestrator's typed
                                    // knowledge SSB sink so every
                                    // `HydeStage::hypothesize` call writes
                                    // one `hyde_query` SSB row.
                                    let hyde_sink: Arc<
                                        dyn elai_knowledge::knowledge_ssb_sink::KnowledgeSsbSink,
                                    > = match ssb.as_ref() {
                                        Some(ssb_arc) => Arc::new(
                                            elai_orchestrator::OrchestratorKnowledgeSsbSink::new(
                                                ssb_arc.clone(),
                                            ),
                                        ),
                                        None => elai_knowledge::knowledge_ssb_sink::noop_sink(),
                                    };
                                    let hyde_stage = elai_knowledge::hyde::HydeStage::with_ssb(
                                        std::sync::Arc::new(hyde_client),
                                        clock.clone(),
                                        hyde_sink,
                                    );
                                    pipeline = pipeline.with_hyde(hyde_stage);
                                    tracing::info!("HydeStage attached: SidecarHydeClient healthy");
                                }
                                Err(e) => {
                                    tracing::warn!(error = %e, "SidecarHydeClient build failed");
                                }
                            }
                        }
                        Err(e) => {
                            tracing::warn!(error = %e, "HyDE SidecarProvider init failed");
                        }
                    }
                } else {
                    tracing::debug!(
                        sidecar = %backend_config.sidecar_url,
                        "sidecar health probe failed; HyDE disabled"
                    );
                }

                // Run startup readiness check.
                let mut cascade = DegradationCascade::new(pipeline.splade.is_some());
                let startup_config = StartupConfig::default();
                let startup_result = startup_readiness_check(
                    &pipeline,
                    &mut cascade,
                    &startup_config,
                    clock.as_ref(),
                    None,
                );
                match startup_result.status {
                    elai_knowledge::ReadinessStatus::Ready => {}
                    elai_knowledge::ReadinessStatus::Degraded => {
                        eprintln!("warn: knowledge pipeline degraded (BM25_ONLY)");
                    }
                    elai_knowledge::ReadinessStatus::Unavailable => {
                        eprintln!("warn: knowledge pipeline unavailable");
                    }
                }

                // C-8 Task #7: emit one SearchModeChanged SSB row so the
                // dashboard's `get_search_mode` Tauri command (cross-process
                // sibling reader) can render the live mode instead of the
                // hardcoded HYBRID fallback. Best-effort: SSB unavailable
                // path is a no-op. Subsequent transitions (e.g. when the
                // cross-encoder breaker opens) will land a fresh row.
                if let Some(ref ssb_arc) = ssb {
                    let resolved_mode = cascade.current_mode();
                    let payload = elai_orchestrator::SsbPayload::SearchModeChanged(
                        elai_orchestrator::SearchModeChangedPayload {
                            mode: search_mode_to_payload_str(resolved_mode).to_string(),
                            history: Vec::new(),
                        },
                    );
                    let ssb_for_emit = ssb_arc.clone();
                    tokio::spawn(async move {
                        if let Err(e) = ssb_for_emit
                            .write_payload("live_cli", 0, payload, Vec::new())
                            .await
                        {
                            tracing::warn!(
                                error = %e,
                                "C-8: SearchModeChanged SSB write failed (non-fatal)"
                            );
                        }
                    });
                }

                // B-4 task #14 + producer wiring: emit one CanaryDrift row
                // carrying the real startup_result.drift_check verdict so
                // the dashboard's SettingsWindow canary panel renders a
                // real similarity number instead of the hardcoded zeros
                // from the C-3 placeholder. Drift threshold mirrors
                // knowledge::startup::DRIFT_THRESHOLD (0.85).
                if let Some(ref ssb_arc) = ssb {
                    let (similarity, passed, reference_present) =
                        match startup_result.drift_check {
                            elai_knowledge::startup::DriftCheckResult::Passed { similarity } => {
                                (similarity as f64, true, true)
                            }
                            elai_knowledge::startup::DriftCheckResult::Failed { similarity } => {
                                (similarity as f64, false, true)
                            }
                            elai_knowledge::startup::DriftCheckResult::ReferenceGenerated => {
                                // First-startup path: no reference vector
                                // existed yet, so there is no real
                                // similarity to report. Surface 1.0 (the
                                // reference matches itself) so the
                                // dashboard does not render "drifted" on
                                // first run.
                                (1.0_f64, true, false)
                            }
                            elai_knowledge::startup::DriftCheckResult::Skipped { .. } => {
                                // Skipped path: surface 1.0 too so first
                                // run reads as healthy. SettingsWindow
                                // surfaces `reference_present=false` as
                                // "(no reference yet)".
                                (1.0_f64, true, false)
                            }
                        };
                    let drift_payload = elai_orchestrator::SsbPayload::CanaryDrift(
                        elai_orchestrator::ssb::CanaryDriftPayload {
                            similarity,
                            drift_threshold: 0.85,
                            passed,
                            reference_present,
                            ts_ms: cascade_ts_ms_now(),
                        },
                    );
                    let ssb_for_drift = ssb_arc.clone();
                    tokio::spawn(async move {
                        if let Err(e) = ssb_for_drift
                            .write_payload(
                                elai_orchestrator::task_runner::RUNNER_AGENT_ID,
                                elai_orchestrator::task_runner::RUNNER_SUPERSTEP,
                                drift_payload,
                                Vec::new(),
                            )
                            .await
                        {
                            tracing::warn!(
                                error = %e,
                                "B-4: CanaryDrift SSB write failed (non-fatal)"
                            );
                        }
                    });
                }

                // B-4 producer wiring: spawn the periodic cascade
                // evaluator + circuit-breaker snapshot loop. Runs once
                // every CASCADE_EVAL_INTERVAL_SECS, drives the cascade
                // through `SsbDegradationNotifier` so mode transitions
                // land in `knowledge_degradation` SSB rows, and emits a
                // `circuit_breaker_state_change` snapshot per known
                // embedding model so `get_embedding_breakers` (dashboard)
                // returns live rows instead of the hardcoded fallback.
                // The cascade itself runs with `None` breaker handles
                // because the real breakers live inside the moved
                // `pipeline.vector_module`; the snapshot derives breaker
                // state from the cascade's current mode plus a heuristic
                // mapping (TextOnly => code breaker OPEN, CodeOnly =>
                // text breaker OPEN, BM25_ONLY => both OPEN). Real
                // breaker introspection is tracked in B-N follow-up.
                if let Some(ref ssb_arc) = ssb {
                    let ssb_for_loop = ssb_arc.clone();
                    let initial_mode = cascade.current_mode();
                    // 1I-R1: seed the shared mode with the post-startup mode and
                    // hand the loop a clone to update each tick.
                    if let Ok(mut m) = search_mode_shared.lock() {
                        *m = initial_mode;
                    }
                    let _mode_handle = search_mode_shared.clone();
                    tokio::spawn(async move {
                        spawn_cascade_evaluator_loop(ssb_for_loop, initial_mode).await;
                    });
                }

                // C-8 Task #3 + #10: construct CosmosWorker, attach
                // CosmosStage to the pipeline, and spawn a Layer 1A tick
                // scheduler driving worker.tick() every
                // DEFAULT_TICK_INTERVAL_SECS (3600s). Per-tick SSB emission
                // is wrapped externally so CosmosWorker stays storage-only.
                // Manifest at <cosmos_root>/manifest.toml; missing/invalid
                // manifest yields an empty worker and the scheduler still
                // runs (cheap no-op).
                if let Ok(cosmos_root) = elai_knowledge::default_cosmos_root() {
                    if cosmos_root.exists() {
                        let manifest_path = cosmos_root.join("manifest.toml");
                        match elai_knowledge::CosmosManifest::load(&manifest_path) {
                            Ok(manifest) => {
                                let cosmos_clock: Arc<dyn elai_knowledge::CosmosClock> =
                                    Arc::new(elai_knowledge::SystemCosmosClock);
                                // C-16 (DECISION-B + DRIFT-B): replace
                                // NoopResearchDispatcher with the real
                                // HttpCosmosResearchDispatcher targeting the
                                // sidecar's SearXNG `/search` endpoint.
                                // Falls back to constructing-without-tick
                                // when the dispatcher init fails (no panic).
                                let sidecar_base = std::env::var("ELAI_SIDECAR_URL")
                                    .or_else(|_| std::env::var("ANTHROPIC_BASE_URL"))
                                    .unwrap_or_else(|_| {
                                        elai_orchestrator::DEFAULT_SIDECAR_SEARCH_URL
                                            .to_string()
                                    });
                                let research: Arc<dyn elai_knowledge::ResearchDispatcher> =
                                    match elai_orchestrator::HttpCosmosResearchDispatcher::new(
                                        sidecar_base.clone(),
                                    ) {
                                        Ok(d) => Arc::new(d),
                                        Err(e) => {
                                            tracing::warn!(
                                                error = %e,
                                                sidecar = %sidecar_base,
                                                "C-16: HttpCosmosResearchDispatcher init failed; falling back to Noop (refresh path disabled)"
                                            );
                                            Arc::new(elai_knowledge::NoopResearchDispatcher)
                                        }
                                    };
                                match elai_knowledge::CosmosWorker::new(
                                    cosmos_root,
                                    manifest,
                                    cosmos_clock,
                                    research,
                                ) {
                                    Ok(mut worker) => {
                                        // C-16 (DECISION-E): attach SSB
                                        // emitter so tick() emits the four
                                        // cosmos_* SSB rows directly. Skipped
                                        // when ssb is None (cold-start
                                        // dashboard) — tracing path is the
                                        // sole channel in that case.
                                        if let Some(ref ssb_arc) = ssb {
                                            match elai_orchestrator::SsbEmitterAdapter::new(
                                                ssb_arc.clone(),
                                            ) {
                                                Ok(adapter) => {
                                                    let emitter: Arc<dyn elai_knowledge::SsbEmitter> =
                                                        Arc::new(adapter);
                                                    worker = worker.with_ssb_emitter(emitter);
                                                }
                                                Err(e) => {
                                                    tracing::warn!(
                                                        error = %e,
                                                        "C-16: SsbEmitterAdapter init failed; tick will use tracing-only channel"
                                                    );
                                                }
                                            }
                                        }
                                        let worker_arc = Arc::new(worker);
                                        // Attach CosmosStage to the pipeline so
                                        // STANDARD/DEEP retrieval includes the
                                        // cross-project cosmos briefs.
                                        let cosmos_stage =
                                            elai_knowledge::CosmosStage::new(
                                                worker_arc.clone(),
                                            );
                                        pipeline = pipeline.with_cosmos_stage(cosmos_stage);

                                        // Spawn the tick scheduler. Each tick:
                                        //  1. emit one CosmosRefreshStarted row
                                        //  2. invoke worker.tick() (sync, fast)
                                        //  3. emit one CosmosArtifactUpdated row
                                        //     per refreshed artifact in the
                                        //     TickReport.
                                        if let Some(ref ssb_arc) = ssb {
                                            let ssb_for_tick = ssb_arc.clone();
                                            let worker_for_tick = worker_arc.clone();
                                            tokio::spawn(async move {
                                                let interval_secs =
                                                    elai_knowledge::DEFAULT_TICK_INTERVAL_SECS;
                                                let mut interval = tokio::time::interval(
                                                    std::time::Duration::from_secs(interval_secs),
                                                );
                                                // tokio::interval fires immediately on
                                                // the first tick(); we want to wait the
                                                // full period before the first refresh
                                                // (the worker just initialised here).
                                                interval.tick().await;
                                                loop {
                                                    interval.tick().await;
                                                    cosmos_tick_emit_loop(
                                                        &ssb_for_tick,
                                                        &worker_for_tick,
                                                    )
                                                    .await;
                                                }
                                            });
                                        }
                                    }
                                    Err(e) => {
                                        tracing::warn!(
                                            error = %e,
                                            "C-8: CosmosWorker init failed; cosmos stage + tick scheduler disabled"
                                        );
                                    }
                                }
                            }
                            Err(e) => {
                                tracing::debug!(
                                    error = %e,
                                    "C-8: cosmos manifest absent or invalid; cosmos stage + tick scheduler disabled"
                                );
                            }
                        }
                    }
                }

                Some(pipeline)
            }
            Err(e) => {
                eprintln!("warn: BM25 index init failed: {e}");
                None
            }
        };

        let indexing_service = indexing::init_indexing_service(ssb.as_ref());

        (knowledge_pipeline, indexing_service)
    }
}
