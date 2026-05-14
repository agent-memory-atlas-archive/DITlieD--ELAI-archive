// Phase A1b — production task_runner wiring helpers.
//
// Moved from `cli::runtime_builder` so the dashboard-v2 Tauri `setup()`
// callback, the `elai run` CLI subcommand, and any future surface can share
// one chokepoint that constructs the SSB writer + feasibility prober +
// Layer 1A processor as a coherent bundle.
//
// SSB path: this module ALWAYS opens the canonical flat path
// `<project_root>/.elai_cc/sessions/ssb.db` via
// `elai_core::paths::sessions_dir_path` — the same bucket the dashboard's
// `state::resolve_ssb_path` reads from. The previous CLI variant nested the
// DB under `sessions/<session_id>/ssb.db` which caused WARN-1 in
// workflow/handoff/notes-A1a.md (the dashboard never saw rows written by
// `elai run`).
//
// Anti-stub: every fallible step bubbles its error as `anyhow::Error`. No
// `.unwrap()` / `.expect()` / `panic!` / `todo!()` lives on this code path.
//
// Contract: `crates/contracts/tests/task_runner_chokepoint_contract.rs`
// (`contract_apply_task_runner_wiring_attaches_both_processors`).

use std::sync::Arc;

use elai_core::clock::{ClockProvider, OsClock};

use crate::{
    agent_pool::{AgentPoolConfig, InProcessAgentPool},
    task_runner::{AgentPoolBabyStepExecutor, BabyStepExecutor},
    DefaultFeasibilityProber, DefaultLayer1aProcessor, FeasibilityProber, Layer1aProcessor, Ssb,
    SsbConfig, TaskRunnerConfig, SSB_DB_FILE_NAME,
};

/// Name of the agent binary that `AgentPoolBabyStepExecutor` spawns.
/// Includes the `.exe` suffix on Windows; bare on other platforms.
#[cfg(windows)]
const AGENT_BINARY_NAME: &str = "elai_cc_agent.exe";
#[cfg(not(windows))]
const AGENT_BINARY_NAME: &str = "elai_cc_agent";

/// Environment variable that overrides the agent binary path.
const ELAI_AGENT_BIN_ENV: &str = "ELAI_AGENT_BIN";

/// Production wiring bundle for `start_task_runner` callers. Holds the four
/// long-lived handles the chokepoint must construct before spawning the
/// background runner task: the SSB writer, the feasibility prober, the
/// Layer 1A processor, and the optional baby-step executor. All are `Arc`'d
/// so the runner loop and the caller's shutdown path can share them.
///
/// `baby_step_executor` defaults to `None` (synthetic / legacy fallback).
/// Callers that want the production agent-pool path call
/// [`build_default_baby_step_executor`] and assign the result before calling
/// [`apply_task_runner_wiring`].
pub struct RuntimeWiring {
    pub ssb: Arc<Ssb>,
    pub feasibility_prober: Arc<DefaultFeasibilityProber>,
    pub layer1a_processor: Arc<DefaultLayer1aProcessor>,
    /// Optional production baby-step executor. `None` keeps the legacy
    /// synthetic path. Set to `Some(build_default_baby_step_executor(...))`
    /// for the real agent-pool path.
    pub baby_step_executor: Option<Arc<dyn BabyStepExecutor>>,
}

/// Build the production wiring bundle for `start_task_runner`.
///
/// Side effects:
/// - Creates `<project_root>/.elai_cc/` and
///   `<project_root>/.elai_cc/sessions/` if missing (matches the
///   dashboard's persistence layout).
/// - Opens (or creates) the canonical project SSB at
///   `<project_root>/.elai_cc/sessions/{SSB_DB_FILE_NAME}` — flat, no
///   per-session sub-directory, so the dashboard reader and any runner
///   writer share the same bucket.
/// - Opens (or creates) the project rules.db at
///   `<project_root>/.elai_cc/rules.db` and runs the standard schema
///   migrations via `elai_rules::schema::open_rules_db`.
/// - Calls `DefaultLayer1aProcessor::ensure_tables` exactly once.
///
/// Anti-stub: every fallible step bubbles the error as `anyhow::Error`; no
/// `.unwrap()` / `panic!` / `todo!()` on the hot path.
pub async fn build_task_runner_wiring(
    project_root: &std::path::Path,
    session_id: &str,
) -> Result<RuntimeWiring, anyhow::Error> {
    // 1. Ensure the `.elai_cc/` data directory and its `sessions/` subtree
    //    exist before we try to open the SQLite files below.
    let data_dir = project_root.join(elai_core::paths::ELAI_DATA_DIR_NAME);
    std::fs::create_dir_all(&data_dir).map_err(|e| {
        anyhow::anyhow!(
            "build_task_runner_wiring: create {} failed: {e}",
            data_dir.display()
        )
    })?;
    let sessions_dir = elai_core::paths::sessions_dir_path(project_root);
    std::fs::create_dir_all(&sessions_dir).map_err(|e| {
        anyhow::anyhow!(
            "build_task_runner_wiring: create {} failed: {e}",
            sessions_dir.display()
        )
    })?;

    // 2. Open the canonical SSB. Flat path `<project_root>/.elai_cc/sessions/ssb.db`
    //    — same bucket the dashboard reads via
    //    `dashboard-v2/src-tauri/src/state.rs::resolve_ssb_path` (the
    //    `sessions_dir_path(&cwd).join("ssb.db")` candidate).
    let ssb_db_path = sessions_dir.join(SSB_DB_FILE_NAME);
    // Phase A1c: derive the project_id from the caller-supplied
    // `project_root` (NOT the process CWD). The dashboard Tauri `setup()`
    // and the `elai run` CLI both pass an explicit `project_root` — using
    // `project_id_from_cwd()` here meant the CLI launched from one directory
    // and the dashboard launched from another bucketed their SSB rows under
    // different `project_id` values for the same on-disk project. The
    // canonical helper canonicalises the path internally so `./` and the
    // real path collapse to the same id (see project_id.rs:42-49).
    let project_id = elai_core::project_id_from_path(project_root);
    let ssb_config = SsbConfig {
        db_path: ssb_db_path,
        pool_size: Some(2),
        project_id,
        session_id: session_id.to_string(),
    };
    let clock_for_ssb: Arc<dyn ClockProvider> = Arc::new(OsClock);
    let ssb = Ssb::new(ssb_config, clock_for_ssb)
        .map_err(|e| anyhow::anyhow!("build_task_runner_wiring: SSB init failed: {e}"))?;
    let ssb = Arc::new(ssb);

    // 3. Construct the production feasibility prober. Mirrors
    //    `dashboard-v2/src-tauri/src/state.rs:381-383`.
    let feasibility_root = data_dir.join("feasibility");
    let feasibility_prober = Arc::new(DefaultFeasibilityProber::new(feasibility_root));

    // 4. Open the project rules.db via the canonical helper. This runs the
    //    standard schema migration; the resulting Connection is then wrapped
    //    in `Arc<tokio::sync::Mutex<_>>` so the async Layer 1A methods can
    //    take the lock across await boundaries.
    let rules_db_path = elai_core::paths::rules_db_path(project_root);
    let clock_for_rules = OsClock;
    let rules_conn = elai_rules::schema::open_rules_db(&rules_db_path, &clock_for_rules)
        .map_err(|e| {
            anyhow::anyhow!(
                "build_task_runner_wiring: open rules.db at {} failed: {e}",
                rules_db_path.display()
            )
        })?;
    let rules_db = Arc::new(tokio::sync::Mutex::new(rules_conn));

    // 5. Build the Layer 1A processor and run `ensure_tables` once. The
    //    constructor at `self_mod_layer1a.rs:486` is the production entry
    //    point — no test-only or stub helper is used here.
    let layer1a = DefaultLayer1aProcessor::new(rules_db, rules_db_path.clone(), ssb.clone());
    layer1a
        .ensure_tables()
        .await
        .map_err(|e| anyhow::anyhow!("build_task_runner_wiring: Layer1A ensure_tables failed: {e}"))?;
    let layer1a_processor = Arc::new(layer1a);

    Ok(RuntimeWiring {
        ssb,
        feasibility_prober,
        layer1a_processor,
        baby_step_executor: None,
    })
}

/// Apply a `RuntimeWiring` bundle to a fresh `TaskRunnerConfig::new_minimal`
/// without mutating the rest of the orchestrator. Both fields are exposed
/// `pub` on `TaskRunnerConfig`, so we set them through struct field
/// assignment; every other field stays at the `new_minimal` default. This is
/// the single place that knows the chokepoint contract — the contract test
/// at `crates/contracts/tests/task_runner_chokepoint_contract.rs` pins it.
pub fn apply_task_runner_wiring(
    cursor_path: std::path::PathBuf,
    wiring: &RuntimeWiring,
) -> TaskRunnerConfig {
    let mut cfg = TaskRunnerConfig::new_minimal(cursor_path);
    let prober: Arc<dyn FeasibilityProber> = wiring.feasibility_prober.clone();
    let processor: Arc<dyn Layer1aProcessor> = wiring.layer1a_processor.clone();
    cfg.feasibility_prober = Some(prober);
    cfg.layer1a_processor = Some(processor);
    cfg.baby_step_executor = wiring.baby_step_executor.clone();
    cfg
}

/// Construct the production [`AgentPoolBabyStepExecutor`] for callers that
/// want to install the real agent-pool path rather than the synthetic
/// fallback.
///
/// The returned `Arc<dyn BabyStepExecutor>` wraps an
/// [`AgentPoolBabyStepExecutor`] backed by:
/// - an [`InProcessAgentPool`] seeded with `entropy` and configured with
///   [`AgentPoolConfig::default()`],
/// - the production [`runtime_model_pool()`] (reports `u64::MAX` VRAM),
/// - the caller-supplied `agent_binary_path` and `socket_dir`.
///
/// Pass the result as `wiring.baby_step_executor = Some(executor)` before
/// calling [`apply_task_runner_wiring`].
///
/// Anti-stub: no `.unwrap()` / `.expect()` / `panic!` — the pool constructor
/// is infallible; the executor is created but never run here.
pub fn build_default_baby_step_executor<E>(
    entropy: E,
    agent_binary_path: std::path::PathBuf,
    socket_dir: std::path::PathBuf,
) -> Arc<dyn BabyStepExecutor>
where
    E: elai_core::EntropySource + Send + 'static,
{
    let model_pool = crate::task_runner::runtime_model_pool();
    let pool = Arc::new(tokio::sync::Mutex::new(InProcessAgentPool::new(
        AgentPoolConfig::default(),
        entropy,
        model_pool,
    )));
    Arc::new(AgentPoolBabyStepExecutor::new(pool, agent_binary_path, socket_dir))
}

/// Locate the `elai_cc_agent[.exe]` binary.
///
/// Resolution order (first found wins):
/// 1. `ELAI_AGENT_BIN` environment variable — returned verbatim as a
///    [`PathBuf`] (file existence is NOT checked, so callers can use this for
///    out-of-process testing).
/// 2. Same directory as the current executable
///    (`std::env::current_exe()?.parent()/elai_cc_agent[.exe]`).
/// 3. `target/debug/elai_cc_agent[.exe]` and
///    `target/release/elai_cc_agent[.exe]` reachable by walking up from the
///    current executable's directory until a `target/{debug,release}` subtree
///    is found or the filesystem root is reached.
///
/// Returns `Err` with a descriptive message listing all probed paths if the
/// binary cannot be located.
pub fn locate_default_agent_binary() -> Result<std::path::PathBuf, anyhow::Error> {
    // 1. Env-var override.
    if let Ok(env_val) = std::env::var(ELAI_AGENT_BIN_ENV) {
        return Ok(std::path::PathBuf::from(env_val));
    }

    let mut probed: Vec<std::path::PathBuf> = Vec::new();

    // 2. Sibling of current_exe.
    let current_exe = std::env::current_exe().map_err(|e| {
        anyhow::anyhow!("locate_default_agent_binary: current_exe() failed: {e}")
    })?;
    if let Some(exe_dir) = current_exe.parent() {
        let candidate = exe_dir.join(AGENT_BINARY_NAME);
        if candidate.exists() {
            return Ok(candidate);
        }
        probed.push(candidate);

        // 3. Walk up looking for target/{debug,release}.
        let mut dir = exe_dir.to_path_buf();
        loop {
            for profile in &["debug", "release"] {
                let candidate = dir.join("target").join(profile).join(AGENT_BINARY_NAME);
                if candidate.exists() {
                    return Ok(candidate);
                }
                probed.push(candidate);
            }
            match dir.parent() {
                Some(parent) => dir = parent.to_path_buf(),
                None => break,
            }
        }
    }

    Err(anyhow::anyhow!(
        "locate_default_agent_binary: binary '{}' not found. Checked:\n  {}",
        AGENT_BINARY_NAME,
        probed
            .iter()
            .map(|p| p.display().to_string())
            .collect::<Vec<_>>()
            .join("\n  "),
    ))
}
