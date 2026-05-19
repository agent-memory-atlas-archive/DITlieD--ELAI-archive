//! C-17 lifecycle helper for the `elai` CLI entrypoints.
//!
//! Wires the orchestrator's `auto_resume_indexed_projects` + a global
//! SIGINT/SIGTERM observer into the long-lived CLI scopes (`CliAction::Run`,
//! `CliAction::Repl`, `CliAction::Prompt`). Short-lived actions (Version,
//! Help, DumpManifests) deliberately bypass this helper — there is no
//! indexing state to resume and no scope long enough to graceful-shutdown.
//!
//! Design (workflow/reasoning/C-17-decisions.md DRIFT-D, DRIFT-E):
//!   * `auto_resume_indexed_projects` is async; CLI entrypoints are sync.
//!     We build a transient current-thread tokio runtime, run the resume
//!     pass to completion, then drop the runtime.
//!   * The signal observer is installed exactly once per process via a
//!     `OnceLock`. It spawns a dedicated background tokio thread whose
//!     only job is to await SIGINT/SIGTERM and then call
//!     `startup::graceful_shutdown` so the process-global
//!     `SHUTTING_DOWN` flag flips. Per-scope ctrl_c handlers continue
//!     to observe the same OS signal independently — tokio installs a
//!     multiplexed listener so all consumers wake.

use std::path::PathBuf;
use std::sync::{Arc, OnceLock};

use elai_core::{ClockProvider, OsClock};

/// Compute the per-CLI `<project_root>/.elai_cc/indexes` path.
fn indexes_root_for(project_root: &std::path::Path) -> PathBuf {
    project_root
        .join(elai_core::paths::ELAI_DATA_DIR_NAME)
        .join(elai_core::paths::INDEXES_DIR_NAME)
}

/// Run the auto-resume pass synchronously. Blocks the caller until every
/// per-project resume task finishes. Failures are logged via `tracing::warn!`
/// and never surface as an error — the CLI continues regardless.
pub(crate) fn auto_resume_blocking(project_root: &std::path::Path) {
    let indexes_root = indexes_root_for(project_root);
    if !indexes_root.exists() {
        tracing::debug!(
            indexes_root = %indexes_root.display(),
            "C-17: indexes root absent; skipping auto_resume"
        );
        return;
    }
    let clock: Arc<dyn ClockProvider> = Arc::new(OsClock);
    let runtime = match tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
    {
        Ok(rt) => rt,
        Err(e) => {
            tracing::warn!(
                error = %e,
                "C-17: failed to build transient runtime for auto_resume; skipping"
            );
            return;
        }
    };
    let report = runtime.block_on(async move {
        elai_orchestrator::startup::auto_resume_indexed_projects(
            indexes_root,
            clock,
        )
        .await
    });
    tracing::info!(?report, "C-17: auto_resume_indexed_projects completed");
}

/// Install the process-wide SIGINT/SIGTERM observer exactly once. Subsequent
/// calls are no-ops. Each call also (re)runs `auto_resume_blocking` for the
/// supplied `project_root` because the project may differ between
/// invocations (e.g. REPL launched after a Run subcommand).
pub(crate) fn install_lifecycle(project_root: &std::path::Path) {
    // Always resume first — it's idempotent at the orchestrator layer
    // (per-project metadata DB load + status check).
    auto_resume_blocking(project_root);

    static SIGNAL_GUARD: OnceLock<()> = OnceLock::new();
    if SIGNAL_GUARD.get().is_some() {
        return;
    }
    let indexes_root = indexes_root_for(project_root);
    SIGNAL_GUARD.get_or_init(move || {
        // Spawn a dedicated thread with its own current-thread runtime so
        // the signal observer never competes with the foreground runtime
        // for the reactor.
        std::thread::Builder::new()
            .name("elai-c17-signal-watcher".to_string())
            .spawn(move || {
                let rt = match tokio::runtime::Builder::new_current_thread()
                    .enable_all()
                    .build()
                {
                    Ok(rt) => rt,
                    Err(e) => {
                        tracing::warn!(
                            error = %e,
                            "C-17: signal-watcher runtime build failed; \
                             process-wide graceful shutdown disabled"
                        );
                        return;
                    }
                };
                rt.block_on(async move {
                    wait_for_terminating_signal().await;
                    // Already shutting down? then nothing to do.
                    if elai_orchestrator::startup::shutting_down() {
                        tracing::info!(
                            "C-17: signal observer fired but SHUTTING_DOWN \
                             already set; no-op"
                        );
                        return;
                    }
                    tracing::info!(
                        "C-17: signal observer fired; calling graceful_shutdown"
                    );
                    let clock: Arc<dyn ClockProvider> = Arc::new(OsClock);
                    let report = elai_orchestrator::startup::graceful_shutdown(
                        indexes_root,
                        elai_orchestrator::startup::SHUTDOWN_DEADLINE,
                        clock,
                    )
                    .await;
                    tracing::info!(
                        ?report,
                        "C-17: graceful_shutdown complete (signal path)"
                    );
                });
            })
            .map(|_| ())
            .unwrap_or_else(|e| {
                tracing::warn!(
                    error = %e,
                    "C-17: failed to spawn signal-watcher thread; \
                     process-wide graceful shutdown disabled"
                );
            });
    });
}

/// Await either SIGINT (cross-platform) or SIGTERM (Unix only). Resolves on
/// the first signal received. On Windows the SIGTERM listener is omitted —
/// `ctrl_c()` already covers `CTRL_C_EVENT` and `CTRL_BREAK_EVENT`.
#[cfg(unix)]
async fn wait_for_terminating_signal() {
    use tokio::signal::unix::{signal, SignalKind};
    let mut sigterm = match signal(SignalKind::terminate()) {
        Ok(s) => s,
        Err(e) => {
            tracing::warn!(
                error = %e,
                "C-17: SIGTERM listener install failed; waiting on SIGINT only"
            );
            // Fall back to ctrl_c-only.
            let _ = tokio::signal::ctrl_c().await;
            return;
        }
    };
    tokio::select! {
        _ = tokio::signal::ctrl_c() => {
            tracing::info!("C-17: SIGINT received");
        }
        _ = sigterm.recv() => {
            tracing::info!("C-17: SIGTERM received");
        }
    }
}

#[cfg(not(unix))]
async fn wait_for_terminating_signal() {
    if let Err(e) = tokio::signal::ctrl_c().await {
        tracing::warn!(error = %e, "C-17: ctrl_c handler error");
    } else {
        tracing::info!("C-17: SIGINT (CTRL_C_EVENT) received");
    }
}
