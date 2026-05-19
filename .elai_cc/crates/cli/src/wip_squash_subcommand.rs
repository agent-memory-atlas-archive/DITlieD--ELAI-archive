// C-13 — `elai wip-squash [--project-root <path>]` CLI subcommand.
//
// Side-channel entry point for the Python FSM hook chain to trigger the
// leading-WIP-commit squash at the AUDITING -> MEMORIZING transition. The
// Rust task_runner also calls
// `elai_orchestrator::wip_squash::squash_wip_commits_if_any` directly
// from its audit loop (DECISION D4 option A); this CLI subcommand is the
// Python-side accessor.
//
// Lifecycle:
//   1. Boot a 2-worker tokio runtime (the squash itself is sync but the
//      SSB write is async).
//   2. Build a fresh RuntimeWiring so the SSB writer routes to the same
//      `.elai_cc/sessions/ssb.db` the dashboard reads from.
//   3. Forward to `wip_squash::squash_wip_commits_if_any` with an empty
//      step_id (no baby-step is in flight when invoked from the hook).
//   4. Print a one-line summary to stdout so the hook can grep it.
//
// No `.unwrap()` / `.expect()` / `panic!`. SSB writes are best-effort
// inside the helper; errors from the squash itself bubble back to the
// caller as `anyhow::Error`.

use std::path::PathBuf;

use elai_orchestrator::wip_squash;
use elai_orchestrator::runtime_setup::build_task_runner_wiring;

/// Entry point invoked by the `CliAction::WipSquash` match arm in main.rs.
pub(crate) fn run_wip_squash_subcommand(project_root: PathBuf) -> Result<(), anyhow::Error> {
    let runtime = tokio::runtime::Builder::new_multi_thread()
        .worker_threads(2)
        .thread_name("elai-wip-squash")
        .enable_all()
        .build()
        .map_err(|e| anyhow::anyhow!("elai wip-squash: tokio runtime init failed: {e}"))?;

    runtime.block_on(async move { run_async(project_root).await })
}

async fn run_async(project_root: PathBuf) -> Result<(), anyhow::Error> {
    // Use a synthetic session id distinct from the runner's `cli-run-*`
    // prefix so the SSB row carries an unambiguous origin tag.
    let session_id = format!("cli-wip-squash-{}", std::process::id());
    let wiring = build_task_runner_wiring(&project_root, &session_id).await?;
    let ssb = wiring.ssb.clone();

    let outcome =
        wip_squash::squash_wip_commits_if_any(&project_root, ssb, "").await?;

    println!(
        "elai wip-squash: wip_count={} squashed={}",
        outcome.wip_count, outcome.squashed
    );
    Ok(())
}
