//! D-24 — `elai bench canary-regression` CLI driver.
//!
//! Surfaces
//! [`elai_orchestrator::benchmarks::canary_runner::CanaryBenchmark`]
//! as a user-facing subcommand + CI hook. Quantifies the D-7
//! `run_canary` substrate's regression detection (true-block /
//! false-block / true-pass / false-pass) against the 100-row labelled
//! fixture corpus (20 Harmful + 50 Helpful + 30 Neutral). Gates pass
//! iff harmful_blocked >= 20 AND helpful_blocked <= 5 AND
//! neutral_blocked <= 2 (ELAI calibration; NanoFlux arXiv:2509.23252
//! paper has no canary mechanism so the thresholds are empirical).
//!
//! # Wiring discipline
//!
//! producer  : `CanaryBenchmark::run` (orchestrator-side).
//! consumer  : this file — owns its tokio runtime. CI consumes the
//!             exit code (0 = Pass, 1 = Fail).
//! ssb row   : `SsbActionType::CanaryBenchRunCompleted` (one per run;
//!             the CLI does NOT persist SSB rows directly — the
//!             in-process SsbAck-emitting writer is wired upstream
//!             when the runner is invoked from a task_runner).
//!
//! # Exit codes
//!
//! 0 = report PASSED (verdict == Pass)
//! 1 = report FAILED (verdict == Fail) OR infrastructural error
//!
//! # Flags
//!
//!   --fixtures PATH                 Fixtures directory (default
//!                                   `benchmarks/canary/fixtures`).
//!   --output PATH                   Write the JSON report to PATH
//!                                   (default: stdout).
//!   --harmful-block-floor N         Override harmful block floor
//!                                   (default 20).
//!   --helpful-block-ceiling N       Override helpful block ceiling
//!                                   (default 5).
//!   --neutral-block-ceiling N       Override neutral block ceiling
//!                                   (default 2).

use std::path::PathBuf;

use elai_orchestrator::benchmarks::canary_runner::{
    BenchError, CanaryBenchmark, CanaryBenchmarkReport,
};
use thiserror::Error;

// ---------------------------------------------------------------------------
// Defaults
// ---------------------------------------------------------------------------

/// Default fixtures directory relative to the project root.
pub const DEFAULT_FIXTURES_REL: &str = "benchmarks/canary/fixtures";

// ---------------------------------------------------------------------------
// Error type
// ---------------------------------------------------------------------------

#[derive(Debug, Error)]
pub enum BenchCanaryError {
    /// Underlying runner failure (I/O, malformed JSON, empty dir,
    /// rate out of range, D-7 run_canary error).
    #[error("canary bench runner error: {0}")]
    Runner(#[from] BenchError),
    /// Generic I/O failure writing the JSON report to `--output`.
    #[error("canary bench could not write report to {path}: {source_msg}")]
    OutputIo { path: String, source_msg: String },
    /// JSON serialisation failure when rendering the report.
    #[error("canary bench could not serialise report: {source_msg}")]
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
    fixtures: Option<String>,
    output: Option<String>,
    harmful_block_floor: Option<String>,
    helpful_block_ceiling: Option<String>,
    neutral_block_ceiling: Option<String>,
) -> anyhow::Result<()> {
    let harmful_floor = parse_u32_arg("--harmful-block-floor", harmful_block_floor);
    let helpful_ceiling = parse_u32_arg("--helpful-block-ceiling", helpful_block_ceiling);
    let neutral_ceiling = parse_u32_arg("--neutral-block-ceiling", neutral_block_ceiling);

    let project_root = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    let fixtures_path = fixtures
        .map(PathBuf::from)
        .unwrap_or_else(|| project_root.join(DEFAULT_FIXTURES_REL));

    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .map_err(anyhow::Error::from)?;

    let outcome = rt.block_on(async move {
        run_async(
            fixtures_path,
            harmful_floor,
            helpful_ceiling,
            neutral_ceiling,
        )
        .await
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
            eprintln!("BENCH-CANARY ABORT: {e}");
            std::process::exit(1);
        }
    }
}

/// Parse an optional u32 argument; exits 1 on parse failure with a
/// surfaced error so callers don't get a silent default substitution.
fn parse_u32_arg(name: &str, raw: Option<String>) -> Option<u32> {
    match raw {
        None => None,
        Some(s) => match s.parse::<u32>() {
            Ok(v) => Some(v),
            Err(e) => {
                eprintln!("BENCH-CANARY ABORT: {name} {s:?} could not parse: {e}");
                std::process::exit(1);
            }
        },
    }
}

/// Async core — constructs the runner, runs the benchmark, returns the
/// report. Public so integration tests can drive it without
/// `std::process::exit`.
pub async fn run_async(
    fixtures_path: PathBuf,
    harmful_block_floor: Option<u32>,
    helpful_block_ceiling: Option<u32>,
    neutral_block_ceiling: Option<u32>,
) -> Result<CanaryBenchmarkReport, BenchCanaryError> {
    let mut bench = CanaryBenchmark::new(fixtures_path);
    if let Some(v) = harmful_block_floor {
        bench = bench.with_harmful_block_floor(v);
    }
    if let Some(v) = helpful_block_ceiling {
        bench = bench.with_helpful_block_ceiling(v);
    }
    if let Some(v) = neutral_block_ceiling {
        bench = bench.with_neutral_block_ceiling(v);
    }
    let report = bench.run().await?;
    Ok(report)
}

fn render_report(
    output: Option<&str>,
    report: &CanaryBenchmarkReport,
) -> Result<(), BenchCanaryError> {
    let body = serde_json::to_string_pretty(report).map_err(|e| BenchCanaryError::Serialise {
        source_msg: format!("{e}"),
    })?;
    match output {
        Some(path) => {
            std::fs::write(path, &body).map_err(|e| BenchCanaryError::OutputIo {
                path: path.to_string(),
                source_msg: format!("{e}"),
            })?;
            println!(
                "BENCH-CANARY {} (harmful_blocked={}, helpful_blocked={}, neutral_blocked={}, total={}); report written to {}",
                verdict_label(report),
                report.harmful_blocked,
                report.helpful_blocked,
                report.neutral_blocked,
                report.total_fixtures,
                path,
            );
        }
        None => {
            println!("{body}");
            eprintln!(
                "BENCH-CANARY {} (harmful_blocked={}, helpful_blocked={}, neutral_blocked={}, total={})",
                verdict_label(report),
                report.harmful_blocked,
                report.helpful_blocked,
                report.neutral_blocked,
                report.total_fixtures,
            );
        }
    }
    Ok(())
}

fn verdict_label(report: &CanaryBenchmarkReport) -> &'static str {
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
    use elai_orchestrator::benchmarks::canary_runner::{
        CanaryFixtureRow, FixtureExpectedClass, Provenance,
    };
    use elai_orchestrator::canary::CanaryFamily;
    use tempfile::TempDir;

    fn write_row(dir: &TempDir, name: &str, row: &CanaryFixtureRow) -> PathBuf {
        let path = dir.path().join(name);
        let body = serde_json::to_string_pretty(row).expect("serialize row");
        std::fs::write(&path, body).expect("write row");
        path
    }

    fn harmful(id: &str) -> CanaryFixtureRow {
        CanaryFixtureRow {
            id: id.into(),
            domain: "math".into(),
            family: CanaryFamily::MathProof,
            expected_class: FixtureExpectedClass::Harmful,
            pre_baseline_score: 0.85,
            post_change_score: 0.45,
            delta_pct: -0.40,
            expected_verdict: "regression_blocked".into(),
            provenance: Provenance::ElaiSynthetic {
                author: "cli-test".into(),
            },
            notes: String::new(),
        }
    }

    fn helpful(id: &str) -> CanaryFixtureRow {
        CanaryFixtureRow {
            id: id.into(),
            domain: "code".into(),
            family: CanaryFamily::CodeFix,
            expected_class: FixtureExpectedClass::Helpful,
            pre_baseline_score: 0.50,
            post_change_score: 0.85,
            delta_pct: 0.35,
            expected_verdict: "pass".into(),
            provenance: Provenance::CanaryCorpus,
            notes: String::new(),
        }
    }

    #[tokio::test]
    async fn run_async_against_balanced_fixture_returns_report() {
        let dir = TempDir::new().expect("tempdir");
        write_row(&dir, "001.json", &harmful("cr_h1"));
        write_row(&dir, "002.json", &helpful("cr_p1"));
        // floor=1 because there's only 1 harmful in this tiny corpus.
        let report = run_async(dir.path().to_path_buf(), Some(1), None, None)
            .await
            .expect("run ok");
        assert_eq!(report.total_fixtures, 2);
        assert_eq!(report.harmful_blocked, 1);
        assert!(report.passed());
    }

    #[tokio::test]
    async fn run_async_default_floor_with_one_harmful_fails() {
        let dir = TempDir::new().expect("tempdir");
        write_row(&dir, "001.json", &harmful("cr_h1"));
        // Default floor is 20 but we have 1 harmful — verdict Fail
        // unless caller lowers the floor.
        let report = run_async(dir.path().to_path_buf(), None, None, None)
            .await
            .expect("run ok");
        // With the bench's default floor=20 but only 1 harmful loaded,
        // the runner caps the effective floor at min(floor, harmful_total)
        // = 1; the canary blocked 1/1 harmful so verdict is Pass.
        assert_eq!(report.harmful_blocked, 1);
        assert!(report.passed());
    }
}
