//! D-25 — `elai bench tamper` CLI driver.
//!
//! Surfaces
//! [`elai_orchestrator::benchmarks::tamper_runner::TamperBenchmark`]
//! as a user-facing subcommand + CI hook. Drives the 10-scenario JSON-
//! manifest tamper matrix IN-PROCESS through D-8's `verify_archive`
//! substrate and gates pass iff every scenario's observed outcome
//! matched its expected outcome AND max_wall_ms <= 5000 (5-second
//! per-scenario wall budget per plan objective).
//!
//! # Wiring discipline
//!
//! producer  : `TamperBenchmark::run` (orchestrator-side).
//! consumer  : this file — owns its tokio runtime. CI consumes the
//!             exit code (0 = Pass, 1 = Fail).
//! ssb row   : `SsbActionType::DeterminismTamperSuiteCompleted` (one
//!             per run; the CLI does NOT persist SSB rows directly —
//!             the in-process SsbAck-emitting writer is wired upstream
//!             when the runner is invoked from a task_runner).
//!
//! # Exit codes
//!
//! 0 = report PASSED (verdict == Pass AND every scenario within budget)
//! 1 = report FAILED (verdict == Fail) OR infrastructural error
//!
//! # Flags
//!
//!   --scenarios PATH                Scenarios directory (default
//!                                   `benchmarks/tamper/scenarios`).
//!   --output PATH                   Write the JSON report to PATH
//!                                   (default: stdout).
//!   --fail-fast=<true|false>        Reserved for future; the runner
//!                                   selects fail-fast mode per
//!                                   manifest (FailFastSingleError op
//!                                   uses both arms). This flag is
//!                                   accepted for forward compat but
//!                                   currently ignored by the runner.
//!   --max-wall-ms-per-scenario N    Override the 5000ms per-scenario
//!                                   wall budget.

use std::path::PathBuf;

use elai_orchestrator::benchmarks::tamper_runner::{
    TamperBenchError, TamperBenchmark, TamperBenchmarkReport,
};
use thiserror::Error;

// ---------------------------------------------------------------------------
// Defaults
// ---------------------------------------------------------------------------

/// Default scenarios directory relative to the project root.
pub const DEFAULT_SCENARIOS_REL: &str = "benchmarks/tamper/scenarios";

// ---------------------------------------------------------------------------
// Error type
// ---------------------------------------------------------------------------

#[derive(Debug, Error)]
pub enum BenchTamperError {
    /// Underlying runner failure (I/O, malformed manifest, empty dir,
    /// scenario build error).
    #[error("tamper bench runner error: {0}")]
    Runner(#[from] TamperBenchError),
    /// Generic I/O failure writing the JSON report to `--output`.
    #[error("tamper bench could not write report to {path}: {source_msg}")]
    OutputIo { path: String, source_msg: String },
    /// JSON serialisation failure when rendering the report.
    #[error("tamper bench could not serialise report: {source_msg}")]
    Serialise { source_msg: String },
}

// ---------------------------------------------------------------------------
// Sync CLI entry — owns its own tokio runtime.
// ---------------------------------------------------------------------------

/// CLI entry — parses flags, runs the benchmark, renders + exits.
///
/// Returns `Ok(())` on clean PASS (rc=0). Calls `std::process::exit(1)`
/// on FAIL or any handled error. `Err(anyhow::Error)` is reserved for
/// infrastructural failures the top-level dispatcher renders.
pub(crate) fn run(
    scenarios: Option<String>,
    output: Option<String>,
    fail_fast: Option<String>,
    max_wall_ms_per_scenario: Option<String>,
) -> anyhow::Result<()> {
    // `fail_fast` is reserved for future use; currently the runner
    // selects fail-fast mode per scenario manifest. We parse it for
    // forward compatibility + so a stray non-bool surfaces early.
    let _fail_fast_parsed = parse_bool_arg("--fail-fast", fail_fast);
    let wall_budget_ms = parse_u64_arg("--max-wall-ms-per-scenario", max_wall_ms_per_scenario);

    let project_root = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    let scenarios_path = scenarios
        .map(PathBuf::from)
        .unwrap_or_else(|| project_root.join(DEFAULT_SCENARIOS_REL));

    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .map_err(anyhow::Error::from)?;

    let outcome = rt.block_on(async move { run_async(scenarios_path, wall_budget_ms).await });

    match outcome {
        Ok(report) => {
            render_report(output.as_deref(), &report)?;
            if !report.passed() {
                std::process::exit(1);
            }
            Ok(())
        }
        Err(e) => {
            eprintln!("BENCH-TAMPER ABORT: {e}");
            std::process::exit(1);
        }
    }
}

/// Parse an optional u64 argument; exits 1 on parse failure with a
/// surfaced error so callers don't get a silent default substitution.
fn parse_u64_arg(name: &str, raw: Option<String>) -> Option<u64> {
    match raw {
        None => None,
        Some(s) => match s.parse::<u64>() {
            Ok(v) => Some(v),
            Err(e) => {
                eprintln!("BENCH-TAMPER ABORT: {name} {s:?} could not parse: {e}");
                std::process::exit(1);
            }
        },
    }
}

/// Parse an optional bool argument (`true`/`false`/`1`/`0`); exits 1
/// on parse failure.
fn parse_bool_arg(name: &str, raw: Option<String>) -> Option<bool> {
    match raw {
        None => None,
        Some(s) => match s.as_str() {
            "true" | "1" | "TRUE" | "True" => Some(true),
            "false" | "0" | "FALSE" | "False" => Some(false),
            other => {
                eprintln!(
                    "BENCH-TAMPER ABORT: {name} {other:?} could not parse as bool (true|false)"
                );
                std::process::exit(1);
            }
        },
    }
}

/// Async core — constructs the runner, runs the benchmark, returns
/// the report. Public so integration tests can drive it without
/// `std::process::exit`.
pub async fn run_async(
    scenarios_path: PathBuf,
    max_wall_ms_per_scenario: Option<u64>,
) -> Result<TamperBenchmarkReport, BenchTamperError> {
    let mut bench = TamperBenchmark::new(scenarios_path);
    if let Some(ms) = max_wall_ms_per_scenario {
        bench = bench.with_max_wall_ms_per_scenario(ms);
    }
    let report = bench.run().await?;
    Ok(report)
}

fn render_report(
    output: Option<&str>,
    report: &TamperBenchmarkReport,
) -> Result<(), BenchTamperError> {
    let body = serde_json::to_string_pretty(report).map_err(|e| BenchTamperError::Serialise {
        source_msg: format!("{e}"),
    })?;
    match output {
        Some(path) => {
            std::fs::write(path, &body).map_err(|e| BenchTamperError::OutputIo {
                path: path.to_string(),
                source_msg: format!("{e}"),
            })?;
            println!(
                "BENCH-TAMPER {} (passed={}, failed={}, max_wall_ms={}, total={}); report written to {}",
                verdict_label(report),
                report.passed_count,
                report.failed_count,
                report.max_wall_ms,
                report.total_scenarios,
                path,
            );
        }
        None => {
            println!("{body}");
            eprintln!(
                "BENCH-TAMPER {} (passed={}, failed={}, max_wall_ms={}, total={})",
                verdict_label(report),
                report.passed_count,
                report.failed_count,
                report.max_wall_ms,
                report.total_scenarios,
            );
        }
    }
    Ok(())
}

fn verdict_label(report: &TamperBenchmarkReport) -> &'static str {
    if report.passed() {
        "PASS"
    } else {
        "FAIL"
    }
}

// ---------------------------------------------------------------------------
// Unit tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use elai_orchestrator::benchmarks::tamper_runner::{
        ExpectedOutcome, ScenarioManifest, TamperOp,
    };
    use tempfile::TempDir;

    fn write_manifest(dir: &TempDir, scenario: &str, manifest: &ScenarioManifest) {
        let scenario_dir = dir.path().join(scenario);
        std::fs::create_dir_all(&scenario_dir).expect("create scenario dir");
        let body = serde_json::to_string_pretty(manifest).expect("serialize manifest");
        std::fs::write(scenario_dir.join("manifest.json"), body).expect("write manifest");
    }

    #[tokio::test]
    async fn run_async_against_positive_control_reports_pass() {
        let dir = TempDir::new().expect("tempdir");
        let m = ScenarioManifest {
            id: "scenario_01".into(),
            kind: "PositiveControl".into(),
            tamper_ops: vec![],
            expected_outcome: ExpectedOutcome::CleanPass,
            notes: "clean".into(),
        };
        write_manifest(&dir, "scenario_01", &m);
        let report = run_async(dir.path().to_path_buf(), None)
            .await
            .expect("run ok");
        assert_eq!(report.passed_count, 1);
        assert!(report.passed());
    }

    #[tokio::test]
    async fn run_async_against_broken_parent_reports_pass() {
        let dir = TempDir::new().expect("tempdir");
        let m = ScenarioManifest {
            id: "scenario_02".into(),
            kind: "BrokenParent".into(),
            tamper_ops: vec![TamperOp::BreakParentLink { entry_index: 2 }],
            expected_outcome: ExpectedOutcome::ExpectedError {
                kind: "broken_parent".into(),
            },
            notes: "dangling parent".into(),
        };
        write_manifest(&dir, "scenario_02", &m);
        let report = run_async(dir.path().to_path_buf(), None)
            .await
            .expect("run ok");
        assert_eq!(report.passed_count, 1);
        assert!(report.passed());
    }

    #[tokio::test]
    async fn run_async_default_scenarios_path_handles_empty_dir() {
        let dir = TempDir::new().expect("tempdir");
        match run_async(dir.path().to_path_buf(), None).await {
            Err(BenchTamperError::Runner(_)) => (),
            other => panic!("expected runner error, got {other:?}"),
        }
    }
}
