//! E-9 — `elai eval <run|list>` CLI driver.
//!
//! Surfaces [`elai_orchestrator::external_benches`] as a user-facing
//! subcommand + CI hook. Drives the host-side scaffolding for the 11
//! external benchmark suites:
//!   - swe-bench-pro       (arXiv:2509.16941, primary, RAG-indexed)
//!   - swe-bench-live      (arXiv:2505.23419)
//!   - livecodebench-v6    (arXiv:2403.07974)
//!   - tau2-bench          (arXiv:2506.07982)
//!   - bfcl-v3             (gorilla.cs.berkeley.edu)
//!   - gaia-l1-l2          (arXiv:2311.12983)
//!   - clawsbench-split    (arXiv:2604.05172)
//!   - refinebench         (arXiv:2511.22173)
//!   - sycophancy-probes   (arXiv:2310.13548)
//!   - math-floors         (gradientscience.org/gsm8k-platinum)
//!   - deep-planning       (arXiv:2601.18137)
//!
//! Live model evals RUNPOD-PENDING — this driver enumerates the registry,
//! emits the `BenchmarkRunObserved` (phase=Started) SSB row at dispatch,
//! and SHELLS OUT to the Python runner at
//! `.elai_cc/benches/external/<suite>.py` when the
//! `ELAI_EVAL_BENCHMARK_ENABLE=1` env var is set. Without the env var, the
//! `--list` flag still works (registry enumeration is dependency-free).
//!
//! Exit codes:
//!   0 = success (--list, or `run` completed with significance verdict
//!       NoChange / Improvement / LowPowerNoConclusion).
//!   1 = `run` produced a `SignificanceVerdict::Regression` for the
//!       configured `--p-target` AND the candidate was below floor.
//!   2 = scaffolding / I/O error (Python runner missing, registry lookup
//!       failure, env var missing on `run`).
//!
//! See workflow/reasoning/E-9-decisions.md DECISION-8.

use std::path::PathBuf;
use std::process::Command;
use std::sync::Arc;

use elai_core::clock::OsClock;
use elai_core::paths::sessions_dir_path;
use elai_orchestrator::external_benches::{
    get_suite, suite_metadata, BenchRegistryError, EXPECTED_SUITE_COUNT,
};
use elai_orchestrator::ssb::{BenchPhase, BenchmarkRunObservedPayload, SsbPayload};
use elai_orchestrator::{Ssb, SsbConfig};
use thiserror::Error;

const PYTHON_RUNNERS_DIR: &str = ".elai_cc/benches/external";
const ENV_GATE: &str = "ELAI_EVAL_BENCHMARK_ENABLE";

/// Runtime lane label for the BenchmarkRunObserved SSB row. Host-side
/// scaffolding lane until the remote sidecar lane comes online
/// (RUNPOD-PENDING per E-9 DECISION-5).
const RUNTIME_LANE_SCAFFOLD: &str = "host_scaffold";
/// Harness scaffold blake3 prefix placeholder until the canonical
/// scaffold-hash pipeline lands. Empty string is the documented default
/// for the Loaded phase per ssb.rs:18094.
const HARNESS_BLAKE3_PLACEHOLDER: &str = "";

#[derive(Debug, Error)]
pub enum EvalCmdError {
    #[error("eval registry error: {0}")]
    Registry(#[from] BenchRegistryError),

    #[error("eval needs --benchmark=<NAME> for `run` (use --list to enumerate)")]
    MissingBenchmarkName,

    #[error(
        "live eval requires {env}=1 (got '{got}'); host-side scaffolding only — \
         set the env var to enable Python runner dispatch"
    )]
    EnvGateNotSet { env: &'static str, got: String },

    #[error("python runner missing at {path}")]
    PythonRunnerMissing { path: String },

    #[error("python runner exited with non-zero status {status} for suite '{suite}'")]
    PythonRunnerFailed { status: i32, suite: String },

    #[error("could not serialise json output: {0}")]
    SerdeJson(#[from] serde_json::Error),

    #[error("could not invoke python runner: {0}")]
    Io(#[from] std::io::Error),
}

/// CLI entrypoint. Owns its own work; no tokio runtime required (the
/// significance + power-calibration paths are pure-stdlib + sync).
pub(crate) fn run(
    benchmark: Option<String>,
    harness_vary: bool,
    p_target: Option<String>,
    list: bool,
    json: bool,
) -> anyhow::Result<()> {
    run_inner(benchmark, harness_vary, p_target, list, json)?;
    Ok(())
}

fn run_inner(
    benchmark: Option<String>,
    harness_vary: bool,
    p_target: Option<String>,
    list: bool,
    json: bool,
) -> Result<(), EvalCmdError> {
    if list {
        return print_registry(json);
    }
    let suite_name = benchmark.ok_or(EvalCmdError::MissingBenchmarkName)?;
    let suite = get_suite(&suite_name).ok_or_else(|| {
        let valid: Vec<String> = suite_metadata().keys().cloned().collect();
        BenchRegistryError::UnknownSuite {
            name: suite_name.clone(),
            valid: valid.join(", "),
        }
    })?;
    let suite_version = suite.version().to_string();
    let sota_reference = suite.sota_reference();

    // Gate live runner dispatch behind env var.
    let env_val = std::env::var(ENV_GATE).unwrap_or_default();
    if env_val != "1" {
        return Err(EvalCmdError::EnvGateNotSet {
            env: ENV_GATE,
            got: env_val,
        });
    }

    // Resolve Python runner path. Snake-case the suite name for the filename.
    let py_file = suite_to_runner_filename(&suite_name);
    let runner_path = std::path::PathBuf::from(PYTHON_RUNNERS_DIR).join(&py_file);
    if !runner_path.exists() {
        // Emit Aborted SSB row before propagating the I/O failure so the
        // dashboard surface does not silently lose the dispatch attempt.
        emit_bench_run_best_effort(BenchmarkRunObservedPayload {
            phase: BenchPhase::Aborted,
            suite_name: suite_name.clone(),
            suite_version: suite_version.clone(),
            score: None,
            n: None,
            sota_reference,
            p_value: None,
            low_power: false,
            required_n: None,
            model_effect_pp: None,
            harness_effect_pp: None,
            coverage_gap_pp: None,
            runtime_lane: RUNTIME_LANE_SCAFFOLD.to_string(),
            harness_blake3_prefix: HARNESS_BLAKE3_PLACEHOLDER.to_string(),
            ts_ms: current_ms(),
        });
        return Err(EvalCmdError::PythonRunnerMissing {
            path: runner_path.display().to_string(),
        });
    }

    // Emit Started SSB row at dispatch (gated by ELAI_EVAL_BENCHMARK_ENABLE=1
    // which is already validated above). Per E-9 DECISION-1 the suite_name
    // is the discriminator, not a separate action_type.
    emit_bench_run_best_effort(BenchmarkRunObservedPayload {
        phase: BenchPhase::Started,
        suite_name: suite_name.clone(),
        suite_version: suite_version.clone(),
        score: None,
        n: None,
        sota_reference,
        p_value: None,
        low_power: false,
        required_n: None,
        model_effect_pp: None,
        harness_effect_pp: None,
        coverage_gap_pp: None,
        runtime_lane: RUNTIME_LANE_SCAFFOLD.to_string(),
        harness_blake3_prefix: HARNESS_BLAKE3_PLACEHOLDER.to_string(),
        ts_ms: current_ms(),
    });

    // RUNPOD-PENDING: ssh maintainer@192.0.2.10 -p 6465 -- run
    // `python {runner_path} --suite={suite_name} [--harness-vary]
    // [--p-target={p}]` against the remote sidecar host. Local dispatch
    // here only confirms the runner file exists + the env gate is set.
    // ItemEvaluated phase rows are emitted by the Python runner once the
    // live sidecar path is wired (RUNPOD-PENDING).
    let p_target_arg = p_target.as_deref().unwrap_or("0.05");
    let mut cmd = Command::new("python");
    cmd.arg(&runner_path)
        .arg(format!("--suite={suite_name}"))
        .arg(format!("--p-target={p_target_arg}"));
    if harness_vary {
        cmd.arg("--harness-vary");
    }
    if json {
        cmd.arg("--json");
    }

    let status = cmd.status()?;
    if !status.success() {
        // Emit Aborted row carrying the non-zero exit so dashboards observe
        // tripwire / power-calibration refusals coming back from the runner.
        emit_bench_run_best_effort(BenchmarkRunObservedPayload {
            phase: BenchPhase::Aborted,
            suite_name: suite_name.clone(),
            suite_version: suite_version.clone(),
            score: None,
            n: None,
            sota_reference,
            p_value: None,
            low_power: false,
            required_n: None,
            model_effect_pp: None,
            harness_effect_pp: None,
            coverage_gap_pp: None,
            runtime_lane: RUNTIME_LANE_SCAFFOLD.to_string(),
            harness_blake3_prefix: HARNESS_BLAKE3_PLACEHOLDER.to_string(),
            ts_ms: current_ms(),
        });
        return Err(EvalCmdError::PythonRunnerFailed {
            status: status.code().unwrap_or(-1),
            suite: suite_name,
        });
    }

    // Emit Completed row on clean exit. score / p_value / required_n /
    // model_effect_pp / harness_effect_pp / coverage_gap_pp remain None at
    // the Rust dispatch layer; the Python runner emits a richer Completed
    // row with the bootstrap-significance scalars once RUNPOD-PENDING is
    // resolved (DECISION-5).
    emit_bench_run_best_effort(BenchmarkRunObservedPayload {
        phase: BenchPhase::Completed,
        suite_name: suite_name.clone(),
        suite_version,
        score: None,
        n: None,
        sota_reference,
        p_value: None,
        low_power: false,
        required_n: None,
        model_effect_pp: None,
        harness_effect_pp: None,
        coverage_gap_pp: None,
        runtime_lane: RUNTIME_LANE_SCAFFOLD.to_string(),
        harness_blake3_prefix: HARNESS_BLAKE3_PLACEHOLDER.to_string(),
        ts_ms: current_ms(),
    });
    Ok(())
}

fn print_registry(json: bool) -> Result<(), EvalCmdError> {
    let metadata = suite_metadata();
    // Sanity-check the registry count matches the canonical expected value.
    if metadata.len() != EXPECTED_SUITE_COUNT {
        return Err(EvalCmdError::Registry(BenchRegistryError::InvariantViolation {
            actual: metadata.len(),
            expected: EXPECTED_SUITE_COUNT,
        }));
    }

    // Emit one Loaded SSB row per suite at registry enumeration. Best-effort
    // emission: a writer failure (no sessions dir, locked DB) MUST NOT mask
    // the registry print, matching archive_verify emission discipline.
    let ts_ms = current_ms();
    for (name, m) in &metadata {
        emit_bench_run_best_effort(BenchmarkRunObservedPayload {
            phase: BenchPhase::Loaded,
            suite_name: name.clone(),
            suite_version: m.version.clone(),
            score: None,
            n: None,
            sota_reference: m.sota_reference,
            p_value: None,
            low_power: false,
            required_n: None,
            model_effect_pp: None,
            harness_effect_pp: None,
            coverage_gap_pp: None,
            runtime_lane: String::new(),
            harness_blake3_prefix: HARNESS_BLAKE3_PLACEHOLDER.to_string(),
            ts_ms,
        });
    }

    if json {
        let mut suites: Vec<&str> = metadata.keys().map(|s| s.as_str()).collect();
        suites.sort();
        let body = serde_json::json!({
            "suites": metadata.values().collect::<Vec<_>>(),
            "count": metadata.len(),
            "expected_count": EXPECTED_SUITE_COUNT,
        });
        println!("{}", serde_json::to_string_pretty(&body)?);
    } else {
        println!("{} external benchmark suites:", metadata.len());
        for (name, m) in &metadata {
            println!(
                "  {name:<20} v{ver:<24} sota={sota:>5} local_floor={lf:.2} cloud_floor={cf:.2}",
                ver = m.version,
                sota = match m.sota_reference {
                    Some(s) => format!("{s:.3}"),
                    None => "n/a ".to_string(),
                },
                lf = m.local_floor,
                cf = m.cloud_floor,
            );
        }
    }
    Ok(())
}

/// Map suite wire name (kebab-case) -> Python runner filename (snake_case).
fn suite_to_runner_filename(suite_name: &str) -> String {
    let snake = suite_name.replace('-', "_");
    format!("{snake}.py")
}

/// Best-effort BenchmarkRunObserved SSB emission. Failures are surfaced to
/// stderr but never propagated — the registry / dispatch result must not be
/// masked by writer construction or DB-lock failures (mirrors
/// archive_verify::emit_archive_verify_ssb semantics).
fn emit_bench_run_best_effort(payload: BenchmarkRunObservedPayload) {
    if let Err(e) = emit_bench_run(payload) {
        eprintln!("warning: failed to emit BenchmarkRunObserved SSB row: {e}");
    }
}

/// Synchronous wrapper around the async [`Ssb::write_payload`] writer using
/// a single-threaded tokio runtime (matches archive_verify discipline).
fn emit_bench_run(payload: BenchmarkRunObservedPayload) -> anyhow::Result<()> {
    let project_root = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    let db_path = sessions_dir_path(&project_root).join("ssb.db");
    let parent = db_path
        .parent()
        .ok_or_else(|| anyhow::anyhow!("ssb.db has no parent dir; cannot prepare path"))?;
    std::fs::create_dir_all(parent)
        .map_err(|e| anyhow::anyhow!("creating ssb sessions dir: {e}"))?;
    let cfg = SsbConfig {
        db_path,
        pool_size: None,
        project_id: "default".to_string(),
        session_id: "cli-eval".to_string(),
    };
    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .map_err(|e| anyhow::anyhow!("building tokio runtime for eval emit: {e}"))?;
    rt.block_on(async move {
        let clock: Arc<dyn elai_core::clock::ClockProvider> = Arc::new(OsClock);
        let ssb = Ssb::new(cfg, clock)
            .map_err(|e| anyhow::anyhow!("opening SSB writer: {e}"))?;
        ssb.write_payload(
            "cli:eval",
            0,
            SsbPayload::BenchmarkRunObserved(payload),
            Vec::new(),
        )
        .await
        .map_err(|e| anyhow::anyhow!("writing BenchmarkRunObserved row: {e}"))?;
        Ok::<(), anyhow::Error>(())
    })
}

fn current_ms() -> i64 {
    use std::time::{SystemTime, UNIX_EPOCH};
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| i64::try_from(d.as_millis()).unwrap_or(i64::MAX))
        .unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn list_runs_without_env() {
        // Removing the env var so we exercise the "list bypasses env gate" path.
        let prev = std::env::var(ENV_GATE).ok();
        std::env::remove_var(ENV_GATE);
        let result = print_registry(false);
        if let Some(v) = prev {
            std::env::set_var(ENV_GATE, v);
        }
        assert!(result.is_ok(), "registry print should succeed without env");
    }

    #[test]
    fn run_missing_benchmark_errors() {
        let result = run_inner(None, false, None, false, false);
        assert!(matches!(result, Err(EvalCmdError::MissingBenchmarkName)));
    }

    #[test]
    fn run_unknown_benchmark_errors() {
        // Env gate isn't reached because suite lookup fails first.
        let result = run_inner(
            Some("not-a-suite".to_string()),
            false,
            None,
            false,
            false,
        );
        assert!(matches!(
            result,
            Err(EvalCmdError::Registry(BenchRegistryError::UnknownSuite { .. }))
        ));
    }

    #[test]
    fn suite_to_runner_filename_snake_case() {
        assert_eq!(
            suite_to_runner_filename("swe-bench-pro"),
            "swe_bench_pro.py"
        );
        assert_eq!(
            suite_to_runner_filename("clawsbench-split"),
            "clawsbench_split.py"
        );
    }
}
