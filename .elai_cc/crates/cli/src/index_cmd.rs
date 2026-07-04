//! dkg-stream-rigorous Task 2: `elai index build` — non-interactive full index
//! build.
//!
//! # Why this exists
//!
//! Before this command the only way to populate the on-disk index was the
//! implicit first-run build inside `init_indexing_service` (driven from a live
//! chat/REPL session). The `bench rag` eval seam needs the index populated at
//! the CANONICAL `.elai_cc/indexes/projects/default/bm25` + `dkg.db` +
//! `vector_index.json` paths WITHOUT entering an interactive session. This
//! command is that scriptable seam: it constructs the same `IndexingService`
//! production uses, points it at the canonical indexes dir, and runs the full
//! build (BM25 + DKG + optional streaming vectors).
//!
//! # Wiring discipline
//!
//! producer : [`elai_orchestrator::indexing_service::IndexingService::run_full_index`]
//!            (the exact constructor + entrypoint `init_indexing_service` uses).
//! reader   : `crate::live_cli::LiveCli::build_local_knowledge_stack` +
//!            `crate::bench_rag` (both open BM25 via `bm25_index_root` +
//!            `BM25_PROJECT_ID`, the same canonical location this writes).
//!
//! # Memory
//!
//! The DKG build is now index-time memory-bounded (the dense-PageRank-matrix
//! OOM was replaced with a sparse transition in
//! `elai_knowledge::pagerank`). The optional vector stage (`--vectors`) embeds
//! ONE file at a time via `VectorIndexer::index_file`, so peak RAM is bounded
//! by the embedding model + a single file's chunks, never the whole corpus.

use std::path::PathBuf;
use std::sync::Arc;

use elai_core::clock::{ClockProvider, OsClock};
use elai_orchestrator::indexing_service::{IndexingService, ELAI_BULK_VECTOR_BUILD_ENV};
use elai_orchestrator::ssb::{LanguageGrammarStatusPayload, Ssb, SsbConfig, SsbPayload};

/// Construct an SSB writer for `run_full_index` observability, or `None` on
/// init failure (the build proceeds without observability rows in that case).
///
/// `Ssb::new` calls `tokio::spawn`, spawning the single-writer `write_loop`
/// task. A `current_thread` runtime only polls spawned tasks while something
/// is actively driving it via `block_on`/`run` on that same thread — merely
/// `.enter()`-ing it (the previous design) sets thread-local context but never
/// polls, so the write_loop task is scheduled and then starved forever. The
/// fix: own the runtime on a dedicated background OS thread that `block_on`s
/// an async future for the caller's entire synchronous work window, so the
/// write_loop task keeps getting polled the whole time.
fn build_index_ssb(project_root: &std::path::Path) -> anyhow::Result<(Option<Arc<Ssb>>, SsbRuntime)> {
    let sessions_dir = elai_core::paths::sessions_dir_path(project_root);
    // B-31: ensure the sessions dir exists before Ssb::new tries to
    // Connection::open the db file. Without this, the parent dir is absent
    // on a fresh project, Ssb::new returns Err, build_index_ssb returns
    // None, and the entire B-31 SSB emit chain (IndexCheckpoint /
    // IndexResume / IndexCancel via with_ssb_writer) is silently dropped
    // — the contract's expect_ssb observable_effect can never fire on the
    // `elai index build` path. Mirrors runtime_setup.rs:288-294.
    if let Err(e) = std::fs::create_dir_all(&sessions_dir) {
        tracing::warn!(
            dir = %sessions_dir.display(),
            error = %e,
            "index_cmd: failed to create sessions dir; SSB emits will be dropped"
        );
    }
    let ssb_db_path = sessions_dir.join("ssb.db");
    let project_id = elai_core::project_id_from_cwd()
        .unwrap_or_else(|| format!("{}unknown", elai_core::PROJECT_ID_PREFIX));

    let (ready_tx, ready_rx) = std::sync::mpsc::channel::<Option<Arc<Ssb>>>();
    // B-31: use a tokio mpsc channel for shutdown so the SSB runtime thread's
    // `recv().await` is async and does NOT block the current-thread runtime
    // from polling the spawned write_loop task. The prior std::sync::mpsc
    // `recv()` is a blocking call that froze the entire runtime, so no SSB
    // rows were ever written on the `elai index build` path.
    let (shutdown_tx, mut shutdown_rx) = tokio::sync::mpsc::channel::<()>(1);

    let thread = std::thread::Builder::new()
        .name("elai-index-ssb-rt".to_string())
        .spawn(move || {
            let rt = match tokio::runtime::Builder::new_current_thread().enable_all().build() {
                Ok(rt) => rt,
                Err(e) => {
                    tracing::warn!(error = %e, "building tokio runtime for SSB");
                    let _ = ready_tx.send(None);
                    return;
                }
            };
            rt.block_on(async move {
                let ssb_config = SsbConfig {
                    db_path: ssb_db_path,
                    pool_size: None,
                    project_id,
                    session_id: "cli-index-build".to_string(),
                };
                let clock: Arc<dyn ClockProvider> = Arc::new(OsClock);
                let ssb = match Ssb::new(ssb_config, clock) {
                    Ok(ssb) => Some(Arc::new(ssb)),
                    Err(e) => {
                        tracing::warn!(error = %e, "SSB init failed; index build proceeds without observability rows");
                        None
                    }
                };
                let _ = ready_tx.send(ssb);
                // Keep this future (and therefore the runtime driving the
                // spawned write_loop task) alive until the caller's
                // synchronous `run_full_index()` finishes and signals shutdown.
                let _ = shutdown_rx.recv().await;
            });
        })
        .map_err(|e| anyhow::anyhow!("spawning SSB runtime thread: {e}"))?;

    let ssb = ready_rx
        .recv()
        .map_err(|e| anyhow::anyhow!("SSB runtime thread died before init: {e}"))?;

    Ok((
        ssb,
        SsbRuntime {
            thread: Some(thread),
            shutdown_tx: Some(shutdown_tx),
        },
    ))
}

/// Owns the background thread + shutdown signal that keep the SSB write_loop
/// task alive for the duration of the index build. Dropping/shutting this
/// down signals the thread to exit and joins it with a bounded timeout so the
/// CLI process never hangs waiting on it.
struct SsbRuntime {
    thread: Option<std::thread::JoinHandle<()>>,
    shutdown_tx: Option<tokio::sync::mpsc::Sender<()>>,
}

impl SsbRuntime {
    /// Signal the background runtime thread to stop driving the write_loop
    /// task and join it, bounded so a stuck thread can't hang the CLI.
    fn shutdown(mut self) {
        const JOIN_TIMEOUT: std::time::Duration = std::time::Duration::from_secs(10);
        drop(self.shutdown_tx.take());
        if let Some(handle) = self.thread.take() {
            let (done_tx, done_rx) = std::sync::mpsc::channel::<()>();
            let waiter = std::thread::spawn(move || {
                let _ = handle.join();
                let _ = done_tx.send(());
            });
            if done_rx.recv_timeout(JOIN_TIMEOUT).is_err() {
                tracing::warn!("SSB runtime thread did not exit within timeout; proceeding without join");
            }
            let _ = waiter.join();
        }
    }
}

/// Run a full, non-interactive index build at `project_root`.
///
/// When `vectors` is set, `ELAI_BULK_VECTOR_BUILD=1` is exported for this
/// process so `run_full_index` populates the (streaming, memory-bounded)
/// vector stage. Writes to the canonical `.elai_cc/indexes` layout.
pub(crate) fn run_index_build(
    project_root: PathBuf,
    vectors: bool,
    json: bool,
) -> anyhow::Result<()> {
    if vectors {
        // Opt the streaming vector stage into this build. `bulk_vector_build_enabled`
        // reads this var when `build_index_resumable` reaches the vector step.
        // SAFETY: single-threaded CLI entrypoint, set before any worker spawns.
        std::env::set_var(ELAI_BULK_VECTOR_BUILD_ENV, "1");
    }

    let indexes_dir = elai_core::paths::indexes_dir_path(&project_root);
    let (ssb, ssb_rt) = build_index_ssb(&project_root)?;

    let mut service = IndexingService::new(project_root.clone(), indexes_dir.clone());
    if let Some(ref writer) = ssb {
        service = service.with_ssb_writer(Arc::clone(writer));
    }

    let summary = service.run_full_index();
    // Signal the background SSB runtime thread to stop driving the
    // write_loop task now that the synchronous build work is done, and join
    // it with a bounded timeout (never hang the CLI on this).
    ssb_rt.shutdown();
    let summary = summary.map_err(|e| anyhow::anyhow!("index build failed: {e}"))?;

    // B-30: emit LanguageGrammarStatus SSB row from the CLI `index build`
    // path, mirroring the `init_indexing_service` emission in indexing.rs.
    // The `run_full_index` summary carries the registry status captured at
    // DKG engine startup; without this emit, the `elai index build` trigger
    // produces no `language_grammar_status` SSB row (WIRING_CONTRACT
    // expect_ssb leg).
    if let (Some(ref writer), Some(ref status)) = (&ssb, &summary.registry_status) {
        let writer = Arc::clone(writer);
        let payload = SsbPayload::LanguageGrammarStatus(
            LanguageGrammarStatusPayload::from(status.clone()),
        );
        let rt = match tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
        {
            Ok(r) => Some(r),
            Err(e) => {
                tracing::warn!(
                    error = %e,
                    "index_cmd: failed to build tokio runtime for LanguageGrammarStatus emit; skipping"
                );
                eprintln!("WARN index_cmd: LanguageGrammarStatus SSB emit skipped: {e}");
                None
            }
        };
        if let Some(rt) = rt {
            rt.block_on(async move {
            if let Err(e) = writer
                .write_payload("dkg_indexing", 0, payload, Vec::new())
                .await
            {
                tracing::warn!(
                    error = %e,
                    "index_cmd: write LanguageGrammarStatus failed"
                );
                eprintln!("WARN index_cmd: write LanguageGrammarStatus failed: {e}");
            }
            });
        }
    }

    if json {
        let body = serde_json::json!({
            "project_root": project_root.display().to_string(),
            "indexes_dir": indexes_dir.display().to_string(),
            "vectors_requested": vectors,
            "dkg_files_parsed": summary.dkg_files_parsed,
            "dkg_nodes": summary.dkg_nodes,
            "dkg_edges": summary.dkg_edges,
            "dkg_files_failed": summary.dkg_files_failed,
            "bm25_files": summary.bm25_files,
            "bm25_chunks": summary.bm25_chunks,
            "bm25_skipped_binary": summary.bm25_skipped_binary,
        });
        println!("{}", serde_json::to_string_pretty(&body)?);
    } else {
        println!("elai index build");
        println!("  project_root = {}", project_root.display());
        println!("  indexes_dir  = {}", indexes_dir.display());
        println!("  vectors      = {vectors}");
        println!(
            "  dkg          = {} files / {} nodes / {} edges ({} failed)",
            summary.dkg_files_parsed, summary.dkg_nodes, summary.dkg_edges, summary.dkg_files_failed
        );
        println!(
            "  bm25         = {} files / {} chunks ({} binary skipped)",
            summary.bm25_files, summary.bm25_chunks, summary.bm25_skipped_binary
        );
    }

    Ok(())
}
