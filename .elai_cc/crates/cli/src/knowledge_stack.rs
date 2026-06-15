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
use std::path::{Path, PathBuf};
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

        check_index_version(&project_root);

        let knowledge_pipeline = match Bm25Index::new(&project_root, session_id) {
            Ok(bm25) => {
                let models_dir = core_models_dir_path(&project_root);
                let mut pipeline = build_base_pipeline(
                    bm25,
                    &project_root,
                    &models_dir,
                    session_id,
                    clock,
                    &ssb,
                );
                pipeline = attach_vector_module(pipeline, &project_root, &models_dir, &ssb);
                pipeline = attach_reranker(pipeline, &models_dir, &ssb, clock);
                pipeline = attach_splade(pipeline, &models_dir, clock);
                pipeline = attach_dkg(pipeline, &project_root, &ssb);
                pipeline = attach_hyde(pipeline, backend_config, &ssb, clock);

                let cascade = run_startup_and_emit(
                    &pipeline,
                    &ssb,
                    clock,
                    &search_mode_shared,
                );

                pipeline = attach_cosmos(pipeline, backend_config, &ssb);
                spawn_playbook_curator(&project_root, &ssb, clock);

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

// ---------------------------------------------------------------------------
// Private helpers
// ---------------------------------------------------------------------------

/// Build one `Arc<dyn KnowledgeSsbSink>` from an optional Ssb writer.
/// Returns the real sink when `ssb` is `Some`, otherwise the noop sink.
/// Replaces the identical `match ssb.as_ref() { Some => ..., None => noop }`
/// blocks that appeared five times inside the outer function.
fn make_ssb_knowledge_sink(
    ssb: &Option<Arc<elai_orchestrator::Ssb>>,
) -> Arc<dyn elai_knowledge::knowledge_ssb_sink::KnowledgeSsbSink> {
    match ssb.as_ref() {
        Some(ssb_arc) => Arc::new(
            elai_orchestrator::OrchestratorKnowledgeSsbSink::new(ssb_arc.clone()),
        ),
        None => elai_knowledge::knowledge_ssb_sink::noop_sink(),
    }
}

/// Check and upgrade the on-disk index version if the `.elai` dir exists.
/// Phase 1I.7. Non-fatal: errors are logged and ignored.
fn check_index_version(project_root: &Path) {
    let elai_data_dir = project_root.join(".elai");
    if !elai_data_dir.exists() {
        return;
    }
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

/// Construct the base `QueryPipeline` from BM25 + CatRag + A-RAG.
/// Handles A-RAG embedder loading and optional DKG corpus population.
/// Returns a pipeline with SSB sink and session id installed.
fn build_base_pipeline(
    bm25: Bm25Index,
    project_root: &Path,
    models_dir: &Path,
    session_id: &str,
    clock: &Arc<dyn ClockProvider>,
    ssb: &Option<Arc<elai_orchestrator::Ssb>>,
) -> QueryPipeline {
    let catrag = CatRag::new(CatRagConfig::default());

    // C-8 Task #8: try to load a standalone graphcodebert ONNX
    // session for the A-RAG semantic_search dot-product path.
    let code_model_path = models_dir.join("onnx").join("graphcodebert.onnx");
    let code_tok_path = models_dir.join("onnx").join("graphcodebert-tokenizer.json");
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

    // Wire the A-RAG corpus from the on-disk DKG when opt-in env flag is set.
    let arag_populate_enabled = std::env::var("ELAI_ARAG_POPULATE_FROM_DKG")
        .map(|v| {
            let v = v.trim().to_ascii_lowercase();
            matches!(v.as_str(), "1" | "true" | "yes" | "on")
        })
        .unwrap_or(false);
    if arag_populate_enabled {
        let dkg_db_path_for_arag = core_dkg_db_path(project_root);
        if dkg_db_path_for_arag.exists() {
            match DkgEngine::new(&dkg_db_path_for_arag, None, 0) {
                Ok(dkg_engine) => {
                    match elai_knowledge::arag::populate_corpus_from_dkg(&dkg_engine, &mut arag) {
                        Ok(n) => eprintln!("info: arag corpus populated from DKG ({n} chunks)"),
                        Err(e) => eprintln!("warn: arag corpus population failed: {e}"),
                    }
                }
                Err(e) => eprintln!(
                    "warn: arag corpus population: DkgEngine open failed: {e}"
                ),
            }
        }
    }

    // C-1: install the typed knowledge SSB sink + session id.
    let pipeline_sink = make_ssb_knowledge_sink(ssb);
    QueryPipeline::new(bm25, catrag, arag)
        .with_ssb(pipeline_sink)
        .with_session_id(session_id.to_owned())
        .with_clock(clock.clone())
}

/// Attempt to load ONNX vector embedding models and install the vector module,
/// indexer, and dual-vector cache on the pipeline (graceful degradation).
fn attach_vector_module(
    mut pipeline: QueryPipeline,
    project_root: &Path,
    models_dir: &Path,
    ssb: &Option<Arc<elai_orchestrator::Ssb>>,
) -> QueryPipeline {
    let code_model_path = models_dir.join("onnx").join("graphcodebert.onnx");
    let code_tok_path = models_dir.join("onnx").join("graphcodebert-tokenizer.json");
    let text_model_path = models_dir.join("onnx").join("granite-embed.onnx");
    let text_tok_path = models_dir.join("onnx").join("granite-embed-tokenizer.json");

    // C-8: code_sess and arag embedder are independent OrtSession instances.
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

    if pipeline.vector_module.is_some() {
        pipeline = attach_vector_indexer_and_cache(pipeline, project_root, ssb);
    }

    pipeline
}

/// Install the vector indexer (with optional code-vec secondary index) and the
/// dual-vector cache on a pipeline that already has `vector_module` set.
/// 1C-R2 + B-4.
fn attach_vector_indexer_and_cache(
    mut pipeline: QueryPipeline,
    project_root: &Path,
    ssb: &Option<Arc<elai_orchestrator::Ssb>>,
) -> QueryPipeline {
    // Initialize vector indexer.
    let index_path = core_vectors_db_path(project_root);
    let mut indexer = if index_path.exists() {
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
    let code_vec_enabled = std::env::var("ELAI_CODE_VEC")
        .map(|v| v == "1" || v.eq_ignore_ascii_case("true"))
        .unwrap_or(false);
    if code_vec_enabled {
        indexer.enable_code_index();
        eprintln!(
            "info: code_vec secondary index enabled (has_code_index={})",
            indexer.has_code_index()
        );
    }
    pipeline.vector_indexer = Some(indexer);

    // 1C-R2: construct + install the on-disk dual-vector cache.
    let cache_path = core_vector_cache_db_path(project_root);
    if let Some(parent) = cache_path.parent() {
        if let Err(e) = std::fs::create_dir_all(parent) {
            tracing::warn!(
                path = %parent.display(),
                error = %e,
                "1C-R2: failed to create indexes dir for vector cache (cache disabled this run)"
            );
        }
    }
    let cache_sink = make_ssb_knowledge_sink(ssb);
    match elai_knowledge::vector::cache::VectorCache::open_with_ssb(&cache_path, cache_sink) {
        Ok(cache) => {
            pipeline = pipeline.with_vector_cache(Arc::new(cache));
            tracing::info!(
                path = %cache_path.display(),
                "1C-R2: vector embedding cache installed (re-index eviction live)"
            );
        }
        Err(e) => {
            tracing::warn!(
                path = %cache_path.display(),
                error = %e,
                "1C-R2: vector cache open failed; re-indexing without cache this run"
            );
        }
    }

    pipeline
}

/// Attach a cross-encoder reranker to the pipeline (graceful degradation).
/// E-17 SP2: uses `RerankerConfig::with_project_root` for absolute paths.
fn attach_reranker(
    mut pipeline: QueryPipeline,
    models_dir: &Path,
    ssb: &Option<Arc<elai_orchestrator::Ssb>>,
    clock: &Arc<dyn ClockProvider>,
) -> QueryPipeline {
    let ce_model_path = models_dir.join("onnx").join("cross-encoder-l12.onnx");
    let ce_tok_path = models_dir.join("onnx").join("tokenizer.json");
    if !ce_model_path.exists() || !ce_tok_path.exists() {
        return pipeline;
    }
    // E-17 SP2 (SP1 carryover): switched from `..RerankerConfig::default()`
    // (relative PathBufs) to `RerankerConfig::with_project_root`.
    let project_root = env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    let mut ce_config = RerankerConfig::with_project_root(&project_root);
    ce_config.model_path = ce_model_path;
    ce_config.tokenizer_path = ce_tok_path;
    // C-1: install SsbRerankerNotifier when an Ssb writer is available.
    let reranker_notifier: Arc<dyn elai_knowledge::reranker::SsbNotifier> = match ssb.as_ref() {
        Some(ssb_arc) => Arc::new(SsbRerankerNotifier::new(ssb_arc.clone())),
        None => Arc::new(NoopRerankerNotifier),
    };
    let mut reranker = CrossEncoderReranker::with_ssb(ce_config, clock.clone(), reranker_notifier);
    match reranker.load() {
        Ok(()) => {
            pipeline = pipeline.with_cross_encoder(reranker);
        }
        Err(e) => {
            eprintln!("warn: cross-encoder reranker not loaded: {e}");
        }
    }
    pipeline
}

/// Attach the SPLADE expander to the pipeline (graceful degradation).
/// E-17 SP2: uses `SpladeConfig::with_project_root` for absolute paths.
fn attach_splade(
    mut pipeline: QueryPipeline,
    models_dir: &Path,
    clock: &Arc<dyn ClockProvider>,
) -> QueryPipeline {
    let splade_model_path = models_dir.join("splade").join("model.onnx");
    let splade_tok_path = models_dir.join("splade").join("tokenizer.json");
    if !splade_model_path.exists() || !splade_tok_path.exists() {
        return pipeline;
    }
    // E-17 SP2 (SP1 carryover): switched from `..SpladeConfig::default()`
    // (relative PathBufs) to `SpladeConfig::with_project_root`.
    let project_root = env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
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
    pipeline
}

/// Ensure the DKG engine exists (rebuilding if necessary) and attach it to
/// the pipeline. C-38: replaced the silent-skip gate with `ensure_dkg_or_rebuild`.
fn attach_dkg(
    mut pipeline: QueryPipeline,
    project_root: &Path,
    ssb: &Option<Arc<elai_orchestrator::Ssb>>,
) -> QueryPipeline {
    let dkg_lifecycle_sink = make_ssb_knowledge_sink(ssb);
    match crate::dkg_lifecycle::ensure_dkg_or_rebuild(project_root, dkg_lifecycle_sink) {
        Ok(engine) => {
            pipeline.dkg_engine = Some(engine);
        }
        Err(e) => {
            eprintln!(
                "warn: ensure_dkg_or_rebuild failed \
                 (PageRank stage will run with empty graph): {e}"
            );
        }
    }
    pipeline
}

/// Attach the HyDE stage when the sidecar is reachable (graceful degradation).
fn attach_hyde(
    mut pipeline: QueryPipeline,
    backend_config: &BackendConfig,
    ssb: &Option<Arc<elai_orchestrator::Ssb>>,
    clock: &Arc<dyn ClockProvider>,
) -> QueryPipeline {
    if !sidecar::sidecar_probe_ok(&backend_config.sidecar_url) {
        tracing::debug!(
            sidecar = %backend_config.sidecar_url,
            "sidecar health probe failed; HyDE disabled"
        );
        return pipeline;
    }
    match sidecar::build_hyde_sidecar_provider(backend_config) {
        Ok(provider) => {
            match sidecar_hyde::SidecarHydeClient::new(Arc::new(provider)) {
                Ok(hyde_client) => {
                    // C-1: thread the orchestrator's typed knowledge SSB sink
                    // so every `HydeStage::hypothesize` call writes one
                    // `hyde_query` SSB row.
                    let hyde_sink = make_ssb_knowledge_sink(ssb);
                    let hyde_stage = elai_knowledge::hyde::HydeStage::with_ssb(
                        Arc::new(hyde_client),
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
    pipeline
}

/// Run the startup readiness check, emit SSB startup events, and spawn the
/// cascade evaluator loop. Returns the constructed `DegradationCascade`.
fn run_startup_and_emit(
    pipeline: &QueryPipeline,
    ssb: &Option<Arc<elai_orchestrator::Ssb>>,
    clock: &Arc<dyn ClockProvider>,
    search_mode_shared: &Arc<std::sync::Mutex<elai_knowledge::degradation::SearchMode>>,
) -> DegradationCascade {
    let mut cascade = DegradationCascade::new(pipeline.splade.is_some());
    let startup_config = StartupConfig::default();
    let startup_result = startup_readiness_check(
        pipeline,
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
    emit_startup_ssb_events(&cascade, ssb, startup_result, search_mode_shared);
    cascade
}

/// Emit the SearchModeChanged and CanaryDrift SSB rows and spawn the cascade
/// evaluator loop after startup. C-8 Task #7, B-4 task #14, 1I-R1.
fn emit_startup_ssb_events(
    cascade: &DegradationCascade,
    ssb: &Option<Arc<elai_orchestrator::Ssb>>,
    startup_result: elai_knowledge::startup::StartupResult,
    search_mode_shared: &Arc<std::sync::Mutex<elai_knowledge::degradation::SearchMode>>,
) {
    // C-8 Task #7: emit one SearchModeChanged SSB row.
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
                tracing::warn!(error = %e, "C-8: SearchModeChanged SSB write failed (non-fatal)");
            }
        });
    }

    // B-4 task #14 + producer wiring: emit one CanaryDrift row.
    if let Some(ref ssb_arc) = ssb {
        let (similarity, passed, reference_present) = match startup_result.drift_check {
            elai_knowledge::startup::DriftCheckResult::Passed { similarity } => {
                (similarity as f64, true, true)
            }
            elai_knowledge::startup::DriftCheckResult::Failed { similarity } => {
                (similarity as f64, false, true)
            }
            // First-startup path: no reference vector existed; surface 1.0 so
            // the dashboard does not render "drifted" on first run.
            elai_knowledge::startup::DriftCheckResult::ReferenceGenerated => (1.0_f64, true, false),
            // Skipped path: surface 1.0 so first run reads as healthy.
            elai_knowledge::startup::DriftCheckResult::Skipped { .. } => (1.0_f64, true, false),
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
                tracing::warn!(error = %e, "B-4: CanaryDrift SSB write failed (non-fatal)");
            }
        });
    }

    // B-4 producer wiring: spawn the periodic cascade evaluator loop.
    if let Some(ref ssb_arc) = ssb {
        let ssb_for_loop = ssb_arc.clone();
        let initial_mode = cascade.current_mode();
        // 1I-R1: seed the shared mode with the post-startup mode.
        if let Ok(mut m) = search_mode_shared.lock() {
            *m = initial_mode;
        }
        let _mode_handle = search_mode_shared.clone();
        tokio::spawn(async move {
            spawn_cascade_evaluator_loop(ssb_for_loop, initial_mode).await;
        });
    }
}

/// Construct and attach the CosmosWorker + CosmosStage and spawn the tick
/// scheduler. C-8 Task #3 + #10, C-16 DECISION-B + DECISION-E.
fn attach_cosmos(
    mut pipeline: QueryPipeline,
    backend_config: &BackendConfig,
    ssb: &Option<Arc<elai_orchestrator::Ssb>>,
) -> QueryPipeline {
    let cosmos_root = match elai_knowledge::default_cosmos_root() {
        Ok(r) if r.exists() => r,
        _ => return pipeline,
    };
    let manifest_path = cosmos_root.join("manifest.toml");
    let manifest = match elai_knowledge::CosmosManifest::load(&manifest_path) {
        Ok(m) => m,
        Err(e) => {
            tracing::debug!(
                error = %e,
                "C-8: cosmos manifest absent or invalid; cosmos stage + tick scheduler disabled"
            );
            return pipeline;
        }
    };

    let worker_arc = match build_cosmos_worker(cosmos_root, manifest, ssb) {
        Some(w) => w,
        None => return pipeline,
    };
    let cosmos_stage = elai_knowledge::CosmosStage::new(worker_arc.clone());
    pipeline = pipeline.with_cosmos_stage(cosmos_stage);

    // Spawn the tick scheduler.
    if let Some(ref ssb_arc) = ssb {
        let ssb_for_tick = ssb_arc.clone();
        let worker_for_tick = worker_arc.clone();
        tokio::spawn(async move {
            let interval_secs = elai_knowledge::DEFAULT_TICK_INTERVAL_SECS;
            let mut interval =
                tokio::time::interval(std::time::Duration::from_secs(interval_secs));
            // tokio::interval fires immediately on the first tick(); wait the
            // full period before the first refresh (worker just initialised).
            interval.tick().await;
            loop {
                interval.tick().await;
                cosmos_tick_emit_loop(&ssb_for_tick, &worker_for_tick).await;
            }
        });
    }

    pipeline
}

/// Build a `CosmosWorker` with an optional SSB emitter attached, then wrap it
/// in an Arc. Returns `None` on construction failure (logs a warn).
/// C-16 DECISION-B (HttpCosmosResearchDispatcher) + DECISION-E (SsbEmitterAdapter).
fn build_cosmos_worker(
    cosmos_root: PathBuf,
    manifest: elai_knowledge::CosmosManifest,
    ssb: &Option<Arc<elai_orchestrator::Ssb>>,
) -> Option<Arc<elai_knowledge::CosmosWorker>> {
    let cosmos_clock: Arc<dyn elai_knowledge::CosmosClock> =
        Arc::new(elai_knowledge::SystemCosmosClock);

    // C-16 (DECISION-B): prefer the real HttpCosmosResearchDispatcher.
    let sidecar_base = std::env::var("ELAI_SIDECAR_URL")
        .or_else(|_| std::env::var("ANTHROPIC_BASE_URL"))
        .unwrap_or_else(|_| elai_orchestrator::DEFAULT_SIDECAR_SEARCH_URL.to_string());
    let research: Arc<dyn elai_knowledge::ResearchDispatcher> =
        match elai_orchestrator::HttpCosmosResearchDispatcher::new(sidecar_base.clone()) {
            Ok(d) => Arc::new(d),
            Err(e) => {
                tracing::warn!(
                    error = %e,
                    sidecar = %sidecar_base,
                    "C-16: HttpCosmosResearchDispatcher init failed; \
                     falling back to Noop (refresh path disabled)"
                );
                Arc::new(elai_knowledge::NoopResearchDispatcher)
            }
        };

    let mut worker = match elai_knowledge::CosmosWorker::new(
        cosmos_root,
        manifest,
        cosmos_clock,
        research,
    ) {
        Ok(w) => w,
        Err(e) => {
            tracing::warn!(
                error = %e,
                "C-8: CosmosWorker init failed; cosmos stage + tick scheduler disabled"
            );
            return None;
        }
    };

    // C-16 (DECISION-E): attach SSB emitter so tick() emits cosmos_* rows.
    if let Some(ref ssb_arc) = ssb {
        match elai_orchestrator::SsbEmitterAdapter::new(ssb_arc.clone()) {
            Ok(adapter) => {
                let emitter: Arc<dyn elai_knowledge::SsbEmitter> = Arc::new(adapter);
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

    Some(Arc::new(worker))
}

/// I-12: spawn the PlaybookCurator offline tick (1-hour interval) if the
/// rules.db exists. Skips the first fire per cosmos-tick pattern.
fn spawn_playbook_curator(
    project_root: &Path,
    ssb: &Option<Arc<elai_orchestrator::Ssb>>,
    clock: &Arc<dyn ClockProvider>,
) {
    let ssb_arc = match ssb.as_ref() {
        Some(s) => s.clone(),
        None => return,
    };
    let rules_db = elai_core::paths::rules_db_path(project_root);
    if !rules_db.exists() {
        return;
    }
    let ssb_for_playbook = ssb_arc;
    let rules_db_for_tick = rules_db;
    let clock_for_tick = Arc::clone(clock);
    tokio::spawn(async move {
        let curator = elai_orchestrator::rule_curator::PlaybookCurator::new(
            &rules_db_for_tick,
            clock_for_tick,
        );
        let curator = Arc::new(curator);
        let mut interval = tokio::time::interval(std::time::Duration::from_secs(
            elai_orchestrator::rule_curator::PLAYBOOK_TICK_INTERVAL_SECS,
        ));
        interval.tick().await; // skip first fire
        loop {
            interval.tick().await;
            if let Err(e) = curator.run_promotion_tick(&ssb_for_playbook).await {
                tracing::warn!(error = %e, "I-12: PlaybookCurator tick failed");
            }
        }
    });
}
