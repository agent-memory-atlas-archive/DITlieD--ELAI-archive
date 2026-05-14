// Phase A1a — `elai run` subcommand.
//
// First production caller of `start_task_runner` outside the orchestrator
// crate. The dashboard never spawned the task_runner (the dashboard reads
// SSB rows but does not own the runner loop), so this module is the
// canonical entry that turns a SSB `TaskSubmitted` row into a live
// `drive_task` invocation.
//
// Contract test: `crates/contracts/tests/task_runner_chokepoint_contract.rs`.
// Spec gap closed: `metrics/phase_4F.toml:103`
// (`production_construction_paths_wired = false`).

use std::path::PathBuf;
use std::sync::Arc;

use elai_core::OsEntropy;
use elai_orchestrator::start_task_runner;

use crate::runtime_builder::{apply_task_runner_wiring, build_task_runner_wiring};

/// Parsed `elai run` arguments.
#[derive(Debug, Clone)]
pub(crate) struct RunArgs {
    /// Project root the runner should bind to. Defaults to the CWD when
    /// the user does not pass `--project-root`.
    pub project_root: PathBuf,
}

/// Synchronous entry consumed by the `CliAction::Run` match arm. Owns its
/// own tokio runtime — the parent `fn run()` in `main.rs` is sync, so we
/// cannot share the runtime that `runtime_client.rs` builds for the REPL
/// path.
pub(crate) fn run_task_runner_subcommand(args: RunArgs) -> Result<(), anyhow::Error> {
    let runtime = tokio::runtime::Builder::new_multi_thread()
        .worker_threads(2)
        .thread_name("elai-runner")
        .enable_all()
        .build()
        .map_err(|e| anyhow::anyhow!("elai run: tokio runtime init failed: {e}"))?;
    runtime.block_on(async move { run_async(args).await })
}

/// Async body of `elai run`. Builds the production wiring, spawns the task
/// runner, and waits for either runner completion or a Ctrl-C.
async fn run_async(args: RunArgs) -> Result<(), anyhow::Error> {
    // session_id matches the dashboard's convention (`dash-v2-<pid>` at
    // state.rs:452). The CLI variant uses a different prefix so the two
    // surfaces are distinguishable in SSB row diagnostics.
    let session_id = format!("cli-run-{}", std::process::id());
    let project_root = args.project_root.clone();
    tracing::info!(
        project_root = %project_root.display(),
        session_id = %session_id,
        "elai run: starting task_runner"
    );

    let mut wiring = build_task_runner_wiring(&project_root, &session_id).await?;

    // Cursor lives under <project_root>/.elai_cc/sessions/<sid>/cursor.json
    // so the runner can resume across CLI invocations from the same
    // project. The build_task_runner_wiring helper already created the
    // session dir, so this path is guaranteed to be writable.
    let cursor_path = project_root
        .join(elai_core::paths::ELAI_DATA_DIR_NAME)
        .join("sessions")
        .join(&session_id)
        .join("cursor.json");

    // Attempt to install the production agent-pool executor. The IPC socket
    // directory lives alongside the cursor so all runner artefacts are
    // co-located under the session subtree.
    let ipc_dir = project_root
        .join(elai_core::paths::ELAI_DATA_DIR_NAME)
        .join("sessions")
        .join(&session_id)
        .join("ipc");
    if let Err(e) = std::fs::create_dir_all(&ipc_dir) {
        tracing::warn!(
            path = %ipc_dir.display(),
            error = %e,
            "elai run: failed to create IPC socket dir; executor will fall back to synthetic"
        );
    } else {
        match elai_orchestrator::locate_default_agent_binary() {
            Ok(agent_bin) => {
                let executor = elai_orchestrator::build_default_baby_step_executor(
                    OsEntropy,
                    agent_bin.clone(),
                    ipc_dir.clone(),
                );
                wiring.baby_step_executor = Some(executor);
                tracing::info!(
                    agent_bin = %agent_bin.display(),
                    "elai run: production AgentPoolBabyStepExecutor installed"
                );
            }
            Err(e) => {
                tracing::warn!(
                    error = %e,
                    "elai run: agent binary not found; using synthetic fallback executor"
                );
            }
        }
    }

    let ssb_writer: Arc<elai_orchestrator::Ssb> = wiring.ssb.clone();
    let cfg = apply_task_runner_wiring(cursor_path, &wiring);

    let (mut handle, cancel) = start_task_runner(ssb_writer, cfg);

    tracing::info!("elai run: task_runner: subscribed to SSB, awaiting tasks");

    // Race the runner future against Ctrl-C. The `tokio::signal::ctrl_c`
    // future resolves when SIGINT (Unix) or CTRL_C_EVENT (Windows) is
    // delivered to the process. Borrow `handle` mutably in the first arm
    // so the second arm can still await it during graceful shutdown.
    tokio::select! {
        runner_result = &mut handle => {
            match runner_result {
                Ok(Ok(())) => {
                    tracing::info!("elai run: task_runner: exited cleanly");
                    Ok(())
                }
                Ok(Err(runner_err)) => {
                    Err(anyhow::anyhow!("elai run: task_runner: {runner_err}"))
                }
                Err(join_err) => {
                    Err(anyhow::anyhow!(
                        "elai run: task_runner: join failed: {join_err}"
                    ))
                }
            }
        }
        ctrl_c = tokio::signal::ctrl_c() => {
            if let Err(e) = ctrl_c {
                tracing::warn!(error = %e, "elai run: ctrl_c handler error; cancelling anyway");
            }
            tracing::info!("elai run: SIGINT received, cancelling task_runner");
            cancel.cancel();
            match tokio::time::timeout(std::time::Duration::from_secs(5), handle).await {
                Ok(Ok(Ok(()))) => {
                    tracing::info!("elai run: task_runner: graceful shutdown complete");
                }
                Ok(Ok(Err(runner_err))) => {
                    tracing::warn!(error = %runner_err, "elai run: task_runner: error during shutdown");
                }
                Ok(Err(join_err)) => {
                    tracing::warn!(error = %join_err, "elai run: task_runner: join error during shutdown");
                }
                Err(_) => {
                    tracing::warn!(
                        "elai run: task_runner: shutdown timed out after 5s; dropping handle"
                    );
                }
            }
            Ok(())
        }
    }
}
