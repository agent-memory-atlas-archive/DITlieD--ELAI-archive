// B-15D AUDIT: .elai vs .elai_cc dual-path bug preserved verbatim from
// main.rs:3991-4084. Fix is out-of-scope for B-15D (see HANDOFF section A.4).

use std::env;
use std::path::PathBuf;
use std::sync::Arc;

use elai_core::clock::{ClockProvider, OsClock};
use elai_core::paths::{
    dkg_db_path as core_dkg_db_path, ELAI_DATA_DIR_NAME, INDEXES_DIR_NAME,
};
use elai_knowledge::index_version::LogNotifier;
use elai_orchestrator::indexing_service::IndexingService;
use elai_knowledge::LanguageRegistryStatus;
use elai_orchestrator::ssb::{
    DkgReadyPayload, IndexingProgressPayload, LanguageGrammarStatusPayload, Ssb, SsbPayload,
};

/// Agent-id stamped on CLI-path SSB rows (distinguishes from task_runner rows).
const CLI_INDEXING_AGENT_ID: &str = "cli_indexing";
/// Superstep for CLI-emitted rows. Dashboard keys on seq, not superstep.
const CLI_INDEXING_SUPERSTEP: i64 = 0;

/// Stage labels — must match the stable set in task_runner.rs.
const INDEXING_STAGE_START: &str = "start";
const INDEXING_STAGE_COMPLETE: &str = "complete";
const INDEXING_STAGE_FAILED: &str = "failed";

/// Return current time as milliseconds since Unix epoch. Best-effort; returns
/// 0 on overflow or clock failure.
///
/// C-15: sourced from `OsClock` (the `ClockProvider` default) so the entropy
/// quarantine scanner does not flag this CLI helper. Indexing-progress
/// timestamps are observation-only.
fn now_unix_ms() -> u64 {
    u64::try_from(OsClock.now_ms()).unwrap_or(u64::MAX)
}

/// Emit `IndexingProgress` and/or `DkgReady` SSB rows from sync context by
/// spawning a short-lived current_thread tokio runtime.
///
/// No-op when `ssb` is `None` (zero-SSB callers: tests, dashboard cold-start).
/// Mirrors the established pattern in `sidecar::try_precompute_role_steering`.
fn emit_indexing_ssb(
    ssb: &Arc<Ssb>,
    project_id: &str,
    stage: &str,
    files_processed: u32,
    elapsed_ms: u64,
    // Track A.1: human-readable error string for the "failed" stage.
    // None on non-failure stages.
    message: Option<String>,
    // DkgReady fields — None means: do not emit DkgReady row.
    dkg: Option<(u32, u32)>, // (node_count, edge_count)
    // B-30 Pass C: LanguageGrammarStatus payload — emitted BEFORE DkgReady.
    // None when grammar status is unavailable (e.g. indexing failed).
    grammar_status: Option<LanguageRegistryStatus>,
    // B-4: failed-file count for DkgReadyPayload; 0 when dkg is None.
    dkg_failed_files: u64,
    // B-30: per-language node counts — populates `DkgReadyPayload.languages`
    // with the set of languages that have >=1 node. Empty when dkg is None.
    dkg_languages: Vec<String>,
) {
    let rt = match tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
    {
        Ok(r) => r,
        Err(e) => {
            tracing::warn!(
                error = %e,
                stage = %stage,
                "cli_indexing: failed to build tokio runtime for SSB emit; skipping"
            );
            eprintln!("WARN cli_indexing: failed to build tokio runtime for SSB emit (stage={stage}): {e}");
            return;
        }
    };

    let writer = Arc::clone(ssb);
    let project_id = project_id.to_owned();
    let stage = stage.to_owned();

    rt.block_on(async move {
        // Track A.1: on the "failed" stage, error_kind and error_message carry
        // structured error context. On success stages, both are None.
        let (error_kind, error_message) = if stage == INDEXING_STAGE_FAILED {
            let em = message.clone();
            (Some("indexing_error".to_owned()), em)
        } else {
            (None, None)
        };
        let progress_payload = SsbPayload::IndexingProgress(IndexingProgressPayload {
            stage: stage.clone(),
            project_id: project_id.clone(),
            files_processed,
            files_total: 0,
            elapsed_ms,
            message,
            error_kind,
            error_message,
        });
        if let Err(e) = writer
            .write_payload(CLI_INDEXING_AGENT_ID, CLI_INDEXING_SUPERSTEP, progress_payload, Vec::new())
            .await
        {
            tracing::warn!(
                stage = %stage,
                error = %e,
                "cli_indexing: write IndexingProgress failed"
            );
            eprintln!("WARN cli_indexing: write IndexingProgress failed (stage={stage}): {e}");
        }

        // B-30 Pass C: emit LanguageGrammarStatus BEFORE DkgReady.
        if let Some(status) = grammar_status {
            let grammar_payload = SsbPayload::LanguageGrammarStatus(
                LanguageGrammarStatusPayload::from(status),
            );
            if let Err(e) = writer
                .write_payload("dkg_indexing", 0, grammar_payload, Vec::new())
                .await
            {
                tracing::warn!(
                    error = %e,
                    "cli_indexing: write LanguageGrammarStatus failed"
                );
                eprintln!("WARN cli_indexing: write LanguageGrammarStatus failed: {e}");
            }
        }

        if let Some((node_count, edge_count)) = dkg {
            let dkg_payload = SsbPayload::DkgReady(DkgReadyPayload {
                project_id: project_id.clone(),
                node_count,
                edge_count,
                languages: dkg_languages,
                failed_files: dkg_failed_files,
            });
            if let Err(e) = writer
                .write_payload(CLI_INDEXING_AGENT_ID, CLI_INDEXING_SUPERSTEP, dkg_payload, Vec::new())
                .await
            {
                tracing::warn!(
                    error = %e,
                    "cli_indexing: write DkgReady failed"
                );
                eprintln!("WARN cli_indexing: write DkgReady failed: {e}");
            }
        }
    });
    // rt drops here; all async work is complete.
}

/// A-44: IndexingService — coordinated DKG+BM25 build on first run,
/// or schema-version check + rebuild on subsequent runs.
///
/// Extracted from `LiveCli::new_local()` lines 3991-4084.
///
/// # Dual-path note
/// The `.elai` vs `.elai_cc` directory check inside this function is preserved
/// verbatim — do NOT fix the discrepancy here. See HANDOFF section A.4 for
/// the dedicated fix scope (B-15D-BUG-FIX).
///
/// # SSB writer
/// When `ssb` is `Some`, emits `IndexingProgress(start)` before and
/// `IndexingProgress(complete)` + `DkgReady` after each `run_full_index` call.
/// When `ssb` is `None` (tests, dashboard cold-start), all SSB writes are
/// skipped — existing behaviour is preserved.
pub(crate) fn init_indexing_service(ssb: Option<&Arc<Ssb>>) -> Option<IndexingService> {
    let project_root = env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    let project_id = project_root
        .to_str()
        .unwrap_or("unknown")
        .to_owned();

    // B-1 Task #10: emit a one-time deprecation warning when both
    // legacy `.elai/` and canonical `.elai_cc/` directories exist
    // on disk. We never auto-delete `.elai/` (other tools may use
    // it); we just bias toward `.elai_cc/`.
    let legacy_data_dir = project_root.join(".elai");
    let canonical_data_dir = project_root.join(ELAI_DATA_DIR_NAME);
    if legacy_data_dir.exists() && canonical_data_dir.exists() {
        tracing::warn!(
            legacy = %legacy_data_dir.display(),
            canonical = %canonical_data_dir.display(),
            "B-1: legacy `.elai/` directory present alongside canonical `.elai_cc/`; \
             ELAI uses .elai_cc/ — .elai/ is no longer read and may be removed manually",
        );
    }

    let indexes_dir = project_root.join(ELAI_DATA_DIR_NAME).join(INDEXES_DIR_NAME);
    let mut svc = IndexingService::new(project_root.clone(), indexes_dir);
    if let Some(writer) = ssb {
        svc = svc.with_ssb_writer(Arc::clone(writer));
    }
    if !svc.indexes_exist() {
        // Fresh index path: emit start, run, emit complete + DkgReady.
        let start_ms = now_unix_ms();
        if let Some(writer) = ssb {
            emit_indexing_ssb(writer, &project_id, INDEXING_STAGE_START, 0, 0, None, None, None, 0, Vec::new());
        }
        match svc.run_full_index() {
            Ok(summary) => {
                let elapsed_ms = now_unix_ms().saturating_sub(start_ms);
                let files_processed: u32 = summary.bm25_files.try_into().unwrap_or(u32::MAX);
                let node_count: u32 = summary.dkg_nodes.try_into().unwrap_or(u32::MAX);
                let edge_count: u32 = summary.dkg_edges.try_into().unwrap_or(u32::MAX);
                tracing::info!(
                    dkg_files = summary.dkg_files_parsed,
                    dkg_nodes = summary.dkg_nodes,
                    bm25_files = summary.bm25_files,
                    bm25_chunks = summary.bm25_chunks,
                    "IndexingService: full index complete"
                );
                if let Some(writer) = ssb {
                    // B-30: populate DkgReadyPayload.languages from the
                    // per-language node counts (languages with >=1 node).
                    let langs: Vec<String> = summary
                        .node_counts_by_language
                        .iter()
                        .filter(|(_, c)| *c > 0)
                        .map(|(lang, _)| lang.clone())
                        .collect();
                    emit_indexing_ssb(
                        writer,
                        &project_id,
                        INDEXING_STAGE_COMPLETE,
                        files_processed,
                        elapsed_ms,
                        None,
                        Some((node_count, edge_count)),
                        summary.registry_status,
                        summary.dkg_files_failed,
                        langs,
                    );
                }
            }
            Err(e) => {
                // Degraded-mode: log warning, continue without IndexingService indexes.
                tracing::warn!("IndexingService: full index failed, continuing in degraded mode: {e}");
                let elapsed_ms = now_unix_ms().saturating_sub(start_ms);
                if let Some(writer) = ssb {
                    emit_indexing_ssb(
                        writer,
                        &project_id,
                        INDEXING_STAGE_FAILED,
                        0,
                        elapsed_ms,
                        Some(e.to_string()),
                        None,
                        None,
                        0,
                        Vec::new(),
                    );
                }
            }
        }
    } else {
        // Indexes exist — check schema version and rebuild if needed.
        match svc.needs_rebuild(&LogNotifier) {
            Ok(elai_knowledge::index_version::UpgradeAction::Reuse) => {
                // Schema is current, nothing to do.
            }
            Ok(action) => {
                tracing::info!(?action, "IndexingService: schema version changed, rebuilding");
                // Schema-rebuild path: emit start, run, emit complete + DkgReady.
                let start_ms = now_unix_ms();
                if let Some(writer) = ssb {
                    emit_indexing_ssb(writer, &project_id, INDEXING_STAGE_START, 0, 0, None, None, None, 0, Vec::new());
                }
                match svc.run_full_index() {
                    Ok(summary) => {
                        let elapsed_ms = now_unix_ms().saturating_sub(start_ms);
                        let files_processed: u32 = summary.bm25_files.try_into().unwrap_or(u32::MAX);
                        let node_count: u32 = summary.dkg_nodes.try_into().unwrap_or(u32::MAX);
                        let edge_count: u32 = summary.dkg_edges.try_into().unwrap_or(u32::MAX);
                        tracing::info!(
                            dkg_files = summary.dkg_files_parsed,
                            bm25_files = summary.bm25_files,
                            "IndexingService: rebuild complete"
                        );
                        if let Some(writer) = ssb {
                            let langs: Vec<String> = summary
                                .node_counts_by_language
                                .iter()
                                .filter(|(_, c)| *c > 0)
                                .map(|(lang, _)| lang.clone())
                                .collect();
                            emit_indexing_ssb(
                                writer,
                                &project_id,
                                INDEXING_STAGE_COMPLETE,
                                files_processed,
                                elapsed_ms,
                                None,
                                Some((node_count, edge_count)),
                                summary.registry_status,
                                summary.dkg_files_failed,
                                langs,
                            );
                        }
                    }
                    Err(e) => {
                        tracing::warn!("IndexingService: rebuild failed, continuing with stale indexes: {e}");
                        let elapsed_ms = now_unix_ms().saturating_sub(start_ms);
                        if let Some(writer) = ssb {
                            emit_indexing_ssb(
                                writer,
                                &project_id,
                                INDEXING_STAGE_FAILED,
                                0,
                                elapsed_ms,
                                Some(e.to_string()),
                                None,
                                None,
                                0,
                                Vec::new(),
                            );
                        }
                    }
                }
            }
            Err(e) => {
                tracing::warn!("IndexingService: schema version check failed: {e}");
            }
        }
    }

    // B-1 Task #9: post-run startup assertion. Fires AFTER
    // `IndexingService` had a chance to create `.elai_cc/` so
    // the dashboard's first-run graceful-degradation path is
    // preserved. This is a non-fatal warning — the CLI proceeds
    // with degraded queriers — but it makes the silent-empty-DKG
    // failure mode visible in logs.
    let project_root_check =
        env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    let canonical_dkg = core_dkg_db_path(&project_root_check);
    if !project_root_check.join(ELAI_DATA_DIR_NAME).exists() {
        tracing::warn!(
            project_root = %project_root_check.display(),
            expected = %canonical_dkg.display(),
            "B-1: `.elai_cc/` directory missing after IndexingService run; \
             project may be uninitialized — DKG-backed features will degrade",
        );
    } else if !canonical_dkg.exists() {
        tracing::warn!(
            expected = %canonical_dkg.display(),
            "B-1: canonical DKG database not found at `.elai_cc/indexes/dkg.db`; \
             PageRank/CatRAG/A-RAG stages will degrade to empty results",
        );
    } else {
        tracing::info!(
            dkg_db = %canonical_dkg.display(),
            "B-1: canonical DKG database present and ready",
        );
    }

    Some(svc)
}
