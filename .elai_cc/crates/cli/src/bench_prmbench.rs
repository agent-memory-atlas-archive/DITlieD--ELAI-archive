//! D-19 — `elai bench prmbench` CLI driver.
//!
//! Surfaces
//! [`elai_orchestrator::benchmarks::prmbench_runner::PRMBenchRunner`] as a
//! user-facing subcommand + CI hook. The PRMBench benchmark exercises the
//! PPM verifier against adversarial gaming categories (Deception Resistance
//! + Circular Logic per arxiv:2501.03124, T1 §4.2) and reports the
//! aggregate verdict.
//!
//! # Wiring discipline
//!
//! producer  : `PRMBenchRunner::run_all` (orchestrator-side).
//! consumer  : this file — owns its tokio runtime + emits an SSB row via
//!             `VecPrmBenchSsbEmitter` when run with `--hook mock` (no
//!             persistent SSB writer in CLI scope yet; the orchestrator
//!             producer writes the canonical row when the runner is
//!             invoked from inside a task_runner). CI consumes the exit
//!             code (0 = Pass, 1 = Fail).
//! ssb row   : `SsbActionType::PrmBenchRunCompleted` (one per run).
//!
//! # Exit codes
//!
//! 0 = report PASSED (`gating_verdict == "pass"`)
//! 1 = report FAILED (`gating_verdict == "fail"`) OR infrastructural error
//!
//! # Flags
//!
//!   --dataset PATH           JSONL fixture (default
//!                            `<cwd>/benchmarks/prmbench/prmbench_mini_fixture.jsonl`)
//!   --hook mock              Use the deterministic `MockSidecarPpmHook`
//!                            (forced score 0.85). Default.
//!   --hook http://host:port  TODO: HTTP PPM hook (deferred to D-2 follow-up).
//!   --output PATH            Write the JSON report to PATH (default: stdout).
//!   --gating-threshold F     Override the default per-category baseline
//!                            (0.30) — applied to ALL categories.
//!   --category NAME          Run only one category via
//!                            `PRMBenchRunner::run_category`.
//!
//! # No `.unwrap()` in non-test code.

use std::path::PathBuf;
use std::sync::Arc;

use elai_orchestrator::benchmarks::prmbench_runner::{
    default_mini_fixture_path, PRMBenchReport, PRMBenchRunner, PrmBenchError,
    VecPrmBenchSsbEmitter,
};
use elai_orchestrator::sidecar_ppm_hook::MockSidecarPpmHook;
use thiserror::Error;

// ---------------------------------------------------------------------------
// Error type (thiserror per CLAUDE.md anti-stale rule).
// ---------------------------------------------------------------------------

#[derive(Debug, Error)]
pub enum BenchPrmbenchError {
    /// Underlying runner failure (I/O, malformed JSON, label, SSB emit).
    #[error("prmbench runner error: {0}")]
    Runner(#[from] PrmBenchError),
    /// `--hook` value pointed at a transport we have not wired yet.
    #[error("prmbench --hook value '{value}' is not supported in this build (only 'mock' is wired)")]
    UnsupportedHook { value: String },
    /// Generic I/O failure writing the JSON report to `--output`.
    #[error("prmbench could not write report to {path}: {source_msg}")]
    OutputIo { path: String, source_msg: String },
    /// JSON serialisation failure when rendering the report.
    #[error("prmbench could not serialise report: {source_msg}")]
    Serialise { source_msg: String },
    /// `--gating-threshold` value outside [0.0, 1.0].
    #[error("prmbench --gating-threshold {value} is out of range [0.0, 1.0]")]
    InvalidThreshold { value: f64 },
    /// Runner construction failed (e.g. PPM client threshold range).
    #[error("prmbench runner construction failed: {source_msg}")]
    Construction { source_msg: String },
}

// ---------------------------------------------------------------------------
// Sync CLI entry — owns its own tokio runtime.
// ---------------------------------------------------------------------------

/// CLI entry — parses (already-validated) flags, runs the benchmark,
/// renders + exits.
///
/// Returns `Ok(())` on clean PASS (rc=0). Calls `std::process::exit(1)`
/// on FAIL or any handled error. `Err(anyhow::Error)` is reserved for
/// infrastructural failures the top-level dispatcher renders.
pub(crate) fn run(
    dataset: Option<String>,
    hook: String,
    output: Option<String>,
    gating_threshold: Option<String>,
    category: Option<String>,
) -> anyhow::Result<()> {
    // Parse + validate threshold range up front. Out-of-range -> exit 1.
    let gating_threshold: Option<f64> = match gating_threshold {
        None => None,
        Some(raw) => match raw.parse::<f64>() {
            Ok(t) if (0.0..=1.0).contains(&t) => Some(t),
            Ok(t) => {
                eprintln!("PRMBENCH ABORT: --gating-threshold {t} out of [0.0, 1.0]");
                std::process::exit(1);
            }
            Err(e) => {
                eprintln!("PRMBENCH ABORT: --gating-threshold {raw:?} could not parse: {e}");
                std::process::exit(1);
            }
        },
    };

    // Validate hook surface. Today only `mock` is wired; an HTTP hook will
    // land alongside the D-2 HttpSidecarPpmHook follow-up.
    if hook != "mock" {
        eprintln!(
            "PRMBENCH ABORT: --hook '{hook}' not supported in this build (only 'mock' is wired)"
        );
        std::process::exit(1);
    }

    let project_root = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    let dataset_path = dataset
        .map(PathBuf::from)
        .unwrap_or_else(|| default_mini_fixture_path(&project_root));

    // Single-threaded runtime is enough — the runner is sequential per
    // candidate row (cache concurrency is provided by the PpmClient mutex).
    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .map_err(anyhow::Error::from)?;

    let outcome = rt.block_on(async move {
        run_async(dataset_path, gating_threshold, category).await
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
            eprintln!("PRMBENCH ABORT: {e}");
            std::process::exit(1);
        }
    }
}

/// Async core — constructs the runner, runs the benchmark, returns the
/// report. Public so integration tests can drive it without
/// `std::process::exit`.
pub async fn run_async(
    dataset_path: PathBuf,
    gating_threshold: Option<f64>,
    category: Option<String>,
) -> Result<PRMBenchReport, BenchPrmbenchError> {
    let hook = Arc::new(MockSidecarPpmHook::with_forced_score(0.85));
    let mut runner = PRMBenchRunner::new(hook, dataset_path)
        .map_err(|e| BenchPrmbenchError::Construction {
            source_msg: format!("{e}"),
        })?;
    if let Some(t) = gating_threshold {
        // Apply the user-supplied threshold to BOTH default categories
        // and any future fallback. (`with_baseline` doesn't take a
        // wildcard; we set each explicitly.)
        runner = runner
            .with_baseline("deception_resistance", t)
            .with_baseline("circular_logic", t);
    }
    let emitter = VecPrmBenchSsbEmitter::new();
    let report = match category {
        Some(cat) => runner.run_category(&cat, &emitter).await?,
        None => runner.run_all(&emitter).await?,
    };
    Ok(report)
}

fn render_report(output: Option<&str>, report: &PRMBenchReport) -> Result<(), BenchPrmbenchError> {
    let body = serde_json::to_string_pretty(report).map_err(|e| BenchPrmbenchError::Serialise {
        source_msg: format!("{e}"),
    })?;
    match output {
        Some(path) => {
            std::fs::write(path, &body).map_err(|e| BenchPrmbenchError::OutputIo {
                path: path.to_string(),
                source_msg: format!("{e}"),
            })?;
            println!(
                "PRMBENCH {} (accuracy={:.3}, precision={:.3}, recall={:.3}, total={}); report written to {}",
                report.gating_verdict.as_str().to_uppercase(),
                report.accuracy,
                report.precision,
                report.recall,
                report.total,
                path,
            );
        }
        None => {
            println!("{body}");
            eprintln!(
                "PRMBENCH {} (accuracy={:.3}, precision={:.3}, recall={:.3}, total={})",
                report.gating_verdict.as_str().to_uppercase(),
                report.accuracy,
                report.precision,
                report.recall,
                report.total,
            );
        }
    }
    Ok(())
}

// ---------------------------------------------------------------------------
// Unit tests (in-crate; observable_effect smoke lives in cli/tests/).
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::TempDir;

    fn write_minimal_fixture(dir: &TempDir) -> PathBuf {
        let path = dir.path().join("fixture.jsonl");
        let row = serde_json::json!({
            "task_id": "deception_resistance_0001",
            "category": "deception_resistance",
            "problem_statement": "compute the area of a rectangle",
            "step_history": ["Step 1: width=4 height=5"],
            "candidate_step": "Step 2: area = 4 * 5 = 20",
            "ground_truth_label": "correct",
        });
        let mut f = std::fs::File::create(&path).expect("create fixture");
        writeln!(f, "{row}").expect("write fixture");
        path
    }

    #[tokio::test]
    async fn run_async_against_one_row_fixture_returns_report() {
        let dir = TempDir::new().expect("tempdir");
        let path = write_minimal_fixture(&dir);
        let report = run_async(path, None, None).await.expect("run ok");
        assert_eq!(report.total, 1);
        assert_eq!(report.per_category.len(), 1);
    }

    #[tokio::test]
    async fn run_async_with_category_filter_runs_one_category() {
        let dir = TempDir::new().expect("tempdir");
        let path = write_minimal_fixture(&dir);
        let report = run_async(path, None, Some("deception_resistance".to_string()))
            .await
            .expect("run ok");
        assert_eq!(report.total, 1);
        assert_eq!(report.per_category.len(), 1);
        assert_eq!(report.per_category[0].category, "deception_resistance");
    }

    #[tokio::test]
    async fn run_async_with_impossible_threshold_returns_fail() {
        let dir = TempDir::new().expect("tempdir");
        let path = write_minimal_fixture(&dir);
        // 1.01 is unreachable for accuracy => Fail.
        let report = run_async(path, Some(1.01), None).await.expect("run ok");
        assert!(!report.passed());
    }
}
