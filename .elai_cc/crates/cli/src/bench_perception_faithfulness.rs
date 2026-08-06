//! J-77 I11 — `elai bench perception-faithfulness [--fixture PATH] [--json]
//! [--enforce]` CLI driver.
//!
//! Surfaces
//! [`elai_orchestrator::benchmarks::perception_faithfulness_runner::
//! run_perception_faithfulness_on_fixture`] as a user-facing subcommand + CI
//! hook. The runner scores a labelled fixture against the REAL production
//! perception fns (`fuse_detections` + `filter_ghost_regions`) using six
//! deterministic, model-free metrics. This CLI writes a timestamped JSON
//! report under `<cwd>/benchmarks/perception_faithfulness/` and prints a
//! PASS/FAIL summary.
//!
//! # Wiring discipline
//!
//! producer  : `run_perception_faithfulness_on_fixture` (orchestrator-side).
//! consumer  : this file — calls the SAME public runner production uses,
//!             writes the report, prints the summary.
//! No SSB beacon (fixture-only thin bench; no new SsbActionType variant).
//!
//! # Exit codes
//!
//! - 0 = run completed (report written). With `--enforce`, 0 only if
//!   `overall_pass` is true.
//! - 1 = infrastructural error (fixture read/parse, report I/O), OR
//!   `--enforce` is set AND `overall_pass` is false.
//!
//! # Flags
//!
//!   --fixture PATH  Fixture JSON (default:
//!                   `.elai_cc/data/perception/fixtures/j77_6metric_fixture.json`,
//!                   resolved relative to CWD first, then common elai roots).
//!   --json          Print the full report JSON to stdout.
//!   --enforce       Exit non-zero when `overall_pass` is false.
//!
//! No `.unwrap()` / `.expect()` in non-test paths.

use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use elai_orchestrator::benchmarks::perception_faithfulness_runner::{
    PerceptionFaithfulnessError, PerceptionFaithfulnessReport,
    run_perception_faithfulness_on_fixture,
};
use thiserror::Error;

// ---------------------------------------------------------------------------
// Error type
// ---------------------------------------------------------------------------

#[derive(Debug, Error)]
pub enum BenchPerceptionFaithfulnessError {
    /// The orchestrator runner failed (fixture read / parse).
    #[error("perception-faithfulness runner error: {0}")]
    Runner(#[from] PerceptionFaithfulnessError),
    /// Could not determine the current working directory for the report dir.
    #[error("perception-faithfulness could not determine cwd: {source_msg}")]
    Cwd { source_msg: String },
    /// Could not create the on-disk report directory.
    #[error("perception-faithfulness could not create dir {path}: {source_msg}")]
    ReportDir { path: String, source_msg: String },
    /// Generic I/O failure writing the JSON report.
    #[error("perception-faithfulness could not write {path}: {source_msg}")]
    OutputIo { path: String, source_msg: String },
    /// JSON serialisation failure.
    #[error("perception-faithfulness could not serialise report: {source_msg}")]
    Serialise { source_msg: String },
}

// ---------------------------------------------------------------------------
// Default fixture resolution
// ---------------------------------------------------------------------------

/// Default fixture relative path inside the repo.
const DEFAULT_FIXTURE_REL: &str =
    ".elai_cc/data/perception/fixtures/j77_6metric_fixture.json";

/// Resolve the default fixture path: try CWD first, then a few common elai
/// roots. Returns the first path that exists, or the CWD-relative path if
/// none exist (so the runner produces a clear read-error).
fn resolve_default_fixture() -> PathBuf {
    let cwd_rel = PathBuf::from(DEFAULT_FIXTURE_REL);
    if cwd_rel.exists() {
        return cwd_rel;
    }
    // Common elai roots where the worktree may be nested.
    let candidates = [
        PathBuf::from("..").join(DEFAULT_FIXTURE_REL),
        PathBuf::from("../..").join(DEFAULT_FIXTURE_REL),
        PathBuf::from("../../..").join(DEFAULT_FIXTURE_REL),
    ];
    for c in &candidates {
        if c.exists() {
            return c.clone();
        }
    }
    cwd_rel
}

// ---------------------------------------------------------------------------
// CLI entry
// ---------------------------------------------------------------------------

/// CLI entry — runs the 6-metric perception-faithfulness harness on a
/// fixture, writes a timestamped JSON report, prints a summary (or the full
/// report JSON with `--json`).
///
/// Returns `Ok(())` on a clean run. When `enforce` is true and
/// `overall_pass` is false, calls `std::process::exit(1)`. On a handled
/// error it prints a `PERCEPTION-FAITHFULNESS ABORT:` line to stderr and
/// calls `std::process::exit(1)`.
pub(crate) fn run(fixture: PathBuf, json: bool, enforce: bool) -> anyhow::Result<()> {
    match run_inner(fixture, json, enforce) {
        Ok(()) => Ok(()),
        Err(e) => {
            eprintln!("PERCEPTION-FAITHFULNESS ABORT: {e}");
            std::process::exit(1);
        }
    }
}

/// Core — resolves the fixture, runs the REAL orchestrator runner, writes
/// the report, prints the summary. Public so integration tests can drive it
/// without the process exit.
pub fn run_inner(
    fixture: PathBuf,
    json: bool,
    enforce: bool,
) -> Result<(), BenchPerceptionFaithfulnessError> {
    // 1. Resolve fixture path.
    let fixture_path = if fixture.as_os_str().is_empty() {
        resolve_default_fixture()
    } else {
        fixture
    };

    // 2. Run the REAL orchestrator runner (same public fn production uses).
    let report = run_perception_faithfulness_on_fixture(&fixture_path)?;

    // 3. Write the timestamped JSON report.
    let project_root = std::env::current_dir().map_err(|e| {
        BenchPerceptionFaithfulnessError::Cwd {
            source_msg: e.to_string(),
        }
    })?;
    let report_path = write_report(&project_root, &report)?;

    // 4. Summary line + optional full JSON.
    let verdict = if report.overall_pass { "PASS" } else { "FAIL" };
    println!(
        "PERCEPTION-FAITHFULNESS {} fixture={} ocr_exact_match={:.3} \
         region_grounding_recall_iou05={:.3} duplicate_region_rate={:.4} \
         exact_count_accuracy={:.3} uncited_claims={} blind_llm_qa_lift={:.3}; \
         report written to {}",
        verdict,
        report.fixture_id,
        report.ocr_exact_match.value,
        report.region_grounding_recall_iou05.value,
        report.duplicate_region_rate.value,
        report.exact_count_accuracy.value,
        report.uncited_claims.value,
        report.blind_llm_qa_lift.value,
        report_path.display(),
    );
    if json {
        let body = serde_json::to_string_pretty(&report).map_err(|e| {
            BenchPerceptionFaithfulnessError::Serialise {
                source_msg: e.to_string(),
            }
        })?;
        println!("{body}");
    }

    // 5. Enforce exit code on metric fail.
    if enforce && !report.overall_pass {
        std::process::exit(1);
    }
    Ok(())
}

/// Write the report JSON to
/// `<project_root>/benchmarks/perception_faithfulness/perception_faithfulness-<unix_ms>.json`
/// and return the written path.
fn write_report(
    project_root: &Path,
    report: &PerceptionFaithfulnessReport,
) -> Result<PathBuf, BenchPerceptionFaithfulnessError> {
    let report_dir = project_root
        .join("benchmarks")
        .join("perception_faithfulness");
    fs::create_dir_all(&report_dir).map_err(|e| BenchPerceptionFaithfulnessError::ReportDir {
        path: report_dir.display().to_string(),
        source_msg: e.to_string(),
    })?;

    let unix_ms = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis())
        .unwrap_or_default();
    let report_path = report_dir.join(format!("perception_faithfulness-{unix_ms}.json"));

    let body = serde_json::to_string_pretty(report).map_err(|e| {
        BenchPerceptionFaithfulnessError::Serialise {
            source_msg: e.to_string(),
        }
    })?;
    fs::write(&report_path, &body).map_err(|e| BenchPerceptionFaithfulnessError::OutputIo {
        path: report_path.display().to_string(),
        source_msg: e.to_string(),
    })?;
    Ok(report_path)
}

// ---------------------------------------------------------------------------
// Unit tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn write_report_emits_json_under_benchmarks_dir() {
        let report = PerceptionFaithfulnessReport {
            fixture_id: "test".to_string(),
            ocr_exact_match: metric("ocr_exact_match", 1.0, true),
            region_grounding_recall_iou05: metric(
                "region_grounding_recall_iou05",
                1.0,
                true,
            ),
            duplicate_region_rate: metric("duplicate_region_rate", 0.0, true),
            exact_count_accuracy: metric("exact_count_accuracy", 1.0, true),
            uncited_claims: metric("uncited_claims", 0.0, true),
            blind_llm_qa_lift: metric("blind_llm_qa_lift", 0.3, true),
            overall_pass: true,
        };
        let dir = tempfile::TempDir::new().expect("tempdir");
        let path = write_report(dir.path(), &report).expect("write report");
        assert!(path.exists());
        let on_disk = path.to_string_lossy().to_string();
        assert!(
            on_disk.contains("benchmarks") && on_disk.contains("perception_faithfulness"),
            "report must live under benchmarks/perception_faithfulness/, got {on_disk}"
        );
        let text = fs::read_to_string(&path).expect("read report");
        let parsed: PerceptionFaithfulnessReport =
            serde_json::from_str(&text).expect("round-trips");
        assert!(parsed.overall_pass);
    }

    /// Helper: build a MetricResult for the test report.
    fn metric(
        name: &str,
        value: f64,
        pass: bool,
    ) -> elai_orchestrator::benchmarks::perception_faithfulness_runner::MetricResult {
        elai_orchestrator::benchmarks::perception_faithfulness_runner::MetricResult {
            name: name.to_string(),
            value,
            pass,
        }
    }
}
