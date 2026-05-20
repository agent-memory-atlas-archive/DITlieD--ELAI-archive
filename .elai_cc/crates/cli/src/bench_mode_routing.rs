//! D-21 — `elai bench mode-routing` CLI driver.
//!
//! Surfaces
//! [`elai_orchestrator::benchmarks::mode_routing_runner::ModeRoutingBenchmark`]
//! as a user-facing subcommand + CI hook. The runner replays a hand-labeled
//! JSONL fixture through D-10's pure predicate
//! [`elai_orchestrator::grounding_mode::select_grounding_mode`] and reports
//! macro F1 + per-mode F1 + 5x5 confusion matrix. The gate passes iff
//! macro F1 >= 0.85 AND every per-mode F1 >= 0.70.
//!
//! # Wiring discipline
//!
//! producer  : `ModeRoutingBenchmark::run_all` (orchestrator-side).
//! consumer  : this file — owns its tokio runtime + emits one SSB row via
//!             `VecModeRoutingBenchEmitter`. CI consumes the exit code
//!             (0 = Pass, 1 = Fail).
//! ssb row   : `SsbActionType::ModeRoutingBenchRunCompleted` (one per run).
//!
//! # Exit codes
//!
//! 0 = report PASSED (`gating_verdict == "pass"`)
//! 1 = report FAILED (`gating_verdict == "fail"`) OR infrastructural error
//!
//! # Flags
//!
//!   --dataset PATH    JSONL fixture (default
//!                     `<cwd>/benchmarks/mode_routing/fixtures/mode_routing_mini.jsonl`)
//!   --output PATH     Write the JSON report to PATH (default: stdout).
//!   --macro-floor F   Override the macro F1 floor (default 0.85).
//!   --per-mode-floor F Override the per-mode F1 floor (default 0.70).
//!
//! No `.unwrap()` in non-test code; thiserror for the error enum;
//! hand-rolled flag parsing (no new clap dep).

use std::path::PathBuf;

use elai_orchestrator::benchmarks::mode_routing_runner::{
    default_mini_fixture_path, ModeRoutingBenchError, ModeRoutingBenchmark, ModeRoutingReport,
    VecModeRoutingBenchEmitter,
};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum BenchModeRoutingError {
    /// Underlying runner failure (I/O, malformed JSON, label, SSB emit).
    #[error("mode-routing runner error: {0}")]
    Runner(#[from] ModeRoutingBenchError),
    /// Generic I/O failure writing the JSON report to `--output`.
    #[error("mode-routing could not write report to {path}: {source_msg}")]
    OutputIo { path: String, source_msg: String },
    /// JSON serialisation failure when rendering the report.
    #[error("mode-routing could not serialise report: {source_msg}")]
    Serialise { source_msg: String },
    /// `--macro-floor` or `--per-mode-floor` outside `[0.0, 1.0]`.
    #[error("mode-routing --{name} {value} is out of range [0.0, 1.0]")]
    InvalidThreshold { name: &'static str, value: f64 },
}

/// CLI entry — parses validated flags, runs the benchmark, renders, exits.
///
/// Returns `Ok(())` on clean PASS (rc=0). Calls `std::process::exit(1)`
/// on FAIL or any handled error. `Err(anyhow::Error)` is reserved for
/// infrastructural failures the top-level dispatcher renders.
pub(crate) fn run(
    dataset: Option<String>,
    output: Option<String>,
    macro_floor: Option<String>,
    per_mode_floor: Option<String>,
) -> anyhow::Result<()> {
    let macro_floor = parse_floor("macro-floor", macro_floor)?;
    let per_mode_floor = parse_floor("per-mode-floor", per_mode_floor)?;

    let project_root = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    let dataset_path = dataset
        .map(PathBuf::from)
        .unwrap_or_else(|| default_mini_fixture_path(&project_root));

    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .map_err(anyhow::Error::from)?;

    let outcome = rt.block_on(async move {
        run_async(dataset_path, macro_floor, per_mode_floor).await
    });

    match outcome {
        Ok(report) => {
            render_report(output.as_deref(), &report)?;
            if !report.passed() {
                std::process::exit(1);
            }
            Ok(())
        }
        Err(e) => {
            eprintln!("MODE-ROUTING ABORT: {e}");
            std::process::exit(1);
        }
    }
}

fn parse_floor(name: &'static str, raw: Option<String>) -> anyhow::Result<Option<f64>> {
    let Some(raw) = raw else {
        return Ok(None);
    };
    let parsed: f64 = raw
        .parse()
        .map_err(|e| anyhow::anyhow!("invalid --{name} {raw:?}: {e}"))?;
    if !(0.0..=1.0).contains(&parsed) {
        eprintln!("MODE-ROUTING ABORT: --{name} {parsed} out of [0.0, 1.0]");
        std::process::exit(1);
    }
    Ok(Some(parsed))
}

/// Async core — constructs the runner, runs the benchmark, returns the
/// report. Public so integration tests can drive it without
/// `std::process::exit`.
pub async fn run_async(
    dataset_path: PathBuf,
    macro_floor: Option<f64>,
    per_mode_floor: Option<f64>,
) -> Result<ModeRoutingReport, BenchModeRoutingError> {
    let mut runner = ModeRoutingBenchmark::new(dataset_path);
    if let Some(m) = macro_floor {
        runner = runner.with_macro_floor(m);
    }
    if let Some(p) = per_mode_floor {
        runner = runner.with_per_mode_floor(p);
    }
    let emitter = VecModeRoutingBenchEmitter::new();
    let report = runner.run_all(&emitter).await?;
    Ok(report)
}

fn render_report(
    output: Option<&str>,
    report: &ModeRoutingReport,
) -> Result<(), BenchModeRoutingError> {
    let body = serde_json::to_string_pretty(report).map_err(|e| BenchModeRoutingError::Serialise {
        source_msg: format!("{e}"),
    })?;
    match output {
        Some(path) => {
            std::fs::write(path, &body).map_err(|e| BenchModeRoutingError::OutputIo {
                path: path.to_string(),
                source_msg: format!("{e}"),
            })?;
            println!(
                "MODE-ROUTING {} (accuracy={:.3}, f1_macro={:.3}, total={}); report written to {}",
                verdict_label(report),
                report.accuracy,
                report.f1_macro,
                report.total,
                path,
            );
        }
        None => {
            println!("{body}");
            eprintln!(
                "MODE-ROUTING {} (accuracy={:.3}, f1_macro={:.3}, total={})",
                verdict_label(report),
                report.accuracy,
                report.f1_macro,
                report.total,
            );
        }
    }
    Ok(())
}

fn verdict_label(report: &ModeRoutingReport) -> String {
    if report.passed() {
        "PASS".to_string()
    } else {
        "FAIL".to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::TempDir;

    fn write_minimal_fixture(dir: &TempDir) -> PathBuf {
        let path = dir.path().join("fixture.jsonl");
        let mut f = std::fs::File::create(&path).expect("create fixture");
        // Five rows, one per mode, each engineered to land on its
        // expected mode -> macro F1 = 1.0 -> PASS.
        let lines = [
            r#"{"claim_id":"c1","claim_text":"x","claim_type":"factual","expected_mode":"constructive","category":"a","with_evidence":true,"available_verifiers":["deterministic","intrinsic"]}"#,
            r#"{"claim_id":"c2","claim_text":"x","claim_type":"speculative","expected_mode":"falsifiability_cross_family","category":"b","with_evidence":false,"available_verifiers":["intrinsic","ppm"]}"#,
            r#"{"claim_id":"c3","claim_text":"x","claim_type":"speculative","expected_mode":"conformal_refusal","category":"e","with_evidence":false,"available_verifiers":["intrinsic"],"conformal_calibration_ready":true}"#,
            r#"{"claim_id":"c4","claim_text":"x","claim_type":"procedural","expected_mode":"operational_deferred","category":"d","with_evidence":false,"available_verifiers":["intrinsic"]}"#,
            r#"{"claim_id":"c5","claim_text":"x","claim_type":"normative","expected_mode":"conservative_fallback","category":"f","with_evidence":true,"available_verifiers":["deterministic"]}"#,
        ];
        for line in lines {
            writeln!(f, "{line}").expect("write fixture line");
        }
        path
    }

    #[tokio::test]
    async fn run_async_against_perfect_fixture_returns_pass() {
        let dir = TempDir::new().expect("tempdir");
        let path = write_minimal_fixture(&dir);
        let report = run_async(path, None, None).await.expect("run ok");
        assert!(report.passed());
        assert_eq!(report.total, 5);
    }

    #[tokio::test]
    async fn run_async_with_impossible_macro_floor_returns_fail() {
        let dir = TempDir::new().expect("tempdir");
        let path = write_minimal_fixture(&dir);
        let report = run_async(path, Some(1.01), None).await.expect("run ok");
        assert!(!report.passed());
    }
}
