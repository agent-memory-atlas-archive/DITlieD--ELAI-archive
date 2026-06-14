//! I-9 — `elai bench error-amplification [--fixture PATH] [--seed <u64>]
//! [--json] [--no-repair]` CLI driver.
//!
//! Surfaces
//! [`elai_orchestrator::benchmarks::error_amplification_runner::ErrorPropagationHarness`]
//! as a user-facing subcommand + CI hook. The harness injects a deterministic
//! byte-perturbation at one step of a frozen pipeline fixture OFFLINE (no
//! model, no sandbox — INFERENCE-FREE, fully deterministic), classifies each
//! downstream step RECOVERED vs PROPAGATED, and computes `alpha_t` (the
//! downstream propagated fraction) + the recovered fraction. The
//! `--no-repair` arm is the R2 negative control (recovery disabled =>
//! `alpha_t == 1.0`). Per ID-5 this is ELAI's OWN measured baseline; no
//! literature alpha is carried as a target.
//!
//! # Wiring discipline
//!
//! producer  : `ErrorPropagationHarness::run` (orchestrator-side, deterministic).
//! consumer  : this file — runs the harness, writes a timestamped
//!             `error-amplification-<unix_ms>.json` report under
//!             `<cwd>/benchmarks/error_amplification/`, and emits EXACTLY ONE
//!             `ErrorAmplificationMeasured` SSB beacon (default-ON; NO
//!             `ELAI_*_ENABLE` gate — this is a bench telemetry beacon).
//! ssb row   : `SsbActionType::ErrorAmplificationMeasured` (one per run).
//!
//! The load-bearing SSB emit calls the SHARED orchestrator helper
//! [`error_amplification_runner::emit_error_amplification_measured`] (the same
//! fn the I-9 contract test drives against a temp DB) so the production seam
//! and the contract share one emit path.
//!
//! # Exit codes
//!
//! 0 = run completed (report written + SSB row emitted).
//! 1 = infrastructural error (malformed fixture, SSB writer failure, report I/O).
//!
//! # No `.unwrap()` / `.expect()` in non-test code.

use std::fs;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

use elai_core::OsClock;
use elai_core::paths::sessions_dir_path;
use elai_orchestrator::benchmarks::error_amplification_runner::{
    emit_error_amplification_measured, ErrorAmpError, ErrorAmpReport, ErrorPropagationHarness,
};
use elai_orchestrator::{Ssb, SsbConfig};
use thiserror::Error;

/// SSB agent label of this emit seam.
const SSB_AGENT: &str = "cli:bench-error-amplification";

// ---------------------------------------------------------------------------
// Error type (thiserror per CLAUDE.md anti-stale rule).
// ---------------------------------------------------------------------------

#[derive(Debug, Error)]
pub enum BenchErrorAmplificationError {
    /// The embedded / supplied fixture failed to load or validate.
    #[error("error-amplification runner error: {0}")]
    Runner(#[from] ErrorAmpError),
    /// Could not read the `--fixture` file.
    #[error("error-amplification could not read fixture {path}: {source_msg}")]
    FixtureIo { path: String, source_msg: String },
    /// Could not determine the current working directory for the report dir.
    #[error("error-amplification could not determine cwd: {source_msg}")]
    Cwd { source_msg: String },
    /// Could not create the on-disk report directory.
    #[error("error-amplification could not create report dir {path}: {source_msg}")]
    ReportDir { path: String, source_msg: String },
    /// Generic I/O failure writing the JSON report.
    #[error("error-amplification could not write report to {path}: {source_msg}")]
    OutputIo { path: String, source_msg: String },
    /// JSON serialisation failure when rendering the report.
    #[error("error-amplification could not serialise report: {source_msg}")]
    Serialise { source_msg: String },
    /// Building the SSB async runtime failed.
    #[error("error-amplification could not build async runtime: {source_msg}")]
    Runtime { source_msg: String },
    /// Opening the SSB single-writer failed.
    #[error("error-amplification could not open SSB writer: {source_msg}")]
    SsbOpen { source_msg: String },
    /// Writing the ErrorAmplificationMeasured beacon failed.
    #[error("error-amplification could not write SSB beacon: {source_msg}")]
    SsbWrite { source_msg: String },
}

// ---------------------------------------------------------------------------
// Sync CLI entry.
// ---------------------------------------------------------------------------

/// CLI entry — runs the error-amplification harness, writes a timestamped JSON
/// report, emits one `ErrorAmplificationMeasured` SSB beacon, and prints a
/// summary (or the full report JSON with `--json`).
///
/// Returns `Ok(())` on a clean run (rc=0). On a handled error it prints an
/// `ERROR-AMPLIFICATION ABORT:` line to stderr and calls `std::process::exit(1)`.
pub(crate) fn run(
    fixture: Option<String>,
    seed: Option<u64>,
    json: bool,
    no_repair: bool,
) -> anyhow::Result<()> {
    match run_inner(fixture, seed, json, no_repair) {
        Ok(()) => Ok(()),
        Err(e) => {
            eprintln!("ERROR-AMPLIFICATION ABORT: {e}");
            std::process::exit(1);
        }
    }
}

/// Core — constructs the harness, runs the measurement, writes the report,
/// emits the beacon. Crate-visible so an integration test can drive it without the
/// process exit.
pub(crate) fn run_inner(
    fixture: Option<String>,
    seed: Option<u64>,
    json: bool,
    no_repair: bool,
) -> Result<(), BenchErrorAmplificationError> {
    // 1. Construct the harness (embedded fixture or an explicit --fixture path).
    let harness = build_harness(fixture.as_deref(), seed)?;

    // 2. Run the measurement. The repaired arm is the canonical default;
    //    --no-repair runs the R2 negative control.
    let report = harness.run(!no_repair);

    // 3. Write the timestamped JSON report under
    //    <cwd>/benchmarks/error_amplification/.
    let project_root = std::env::current_dir().map_err(|e| BenchErrorAmplificationError::Cwd {
        source_msg: e.to_string(),
    })?;
    let report_path = write_report(&project_root, &report)?;

    // 4. Emit EXACTLY ONE ErrorAmplificationMeasured SSB beacon (default-on; no
    //    ELAI_*_ENABLE gate). Ssb::new MUST be constructed inside rt.block_on
    //    (the writer spawns a task; observable only on a live run).
    emit_beacon(&project_root, &report)?;

    // 5. Summary line, plus the full report JSON when --json is set.
    println!(
        "ERROR-AMPLIFICATION suite={} n_steps={} injected_step={} alpha_t={:.4} \
         recovered_fraction={:.4} repaired={} annealed={} seed={} fixture_blake3={}; \
         report written to {}",
        report.suite,
        report.n_steps,
        report.injected_step,
        report.alpha_t,
        report.recovered_fraction,
        report.repaired,
        report.annealed_temperature.is_some(),
        report.seed,
        report.fixture_blake3,
        report_path.display(),
    );
    if json {
        let body = serde_json::to_string_pretty(&report).map_err(|e| {
            BenchErrorAmplificationError::Serialise {
                source_msg: e.to_string(),
            }
        })?;
        println!("{body}");
    }
    Ok(())
}

/// Build the harness from either the embedded fixture (default) or an explicit
/// `--fixture` path read from disk. An explicit `--seed` overrides the
/// `ELAI_SEED` path; absence falls back to `from_embedded` (which reads
/// `ELAI_SEED`, default 0).
fn build_harness(
    fixture: Option<&str>,
    seed: Option<u64>,
) -> Result<ErrorPropagationHarness, BenchErrorAmplificationError> {
    match (fixture, seed) {
        (Some(path), seed) => {
            let json = fs::read_to_string(path).map_err(|e| {
                BenchErrorAmplificationError::FixtureIo {
                    path: path.to_string(),
                    source_msg: e.to_string(),
                }
            })?;
            // No explicit seed => default 0 (deterministic).
            let s = seed.unwrap_or(0);
            Ok(ErrorPropagationHarness::from_json_with_seed(&json, s)?)
        }
        (None, Some(s)) => {
            // Embedded fixture, explicit seed: re-serialise via the
            // embedded-with-seed path. The embedded fixture is the same blob
            // `from_embedded` reads; we read ELAI_SEED only when no --seed.
            Ok(ErrorPropagationHarness::from_embedded_with_seed(s)?)
        }
        (None, None) => Ok(ErrorPropagationHarness::from_embedded()?),
    }
}

/// Write the report JSON to
/// `<project_root>/benchmarks/error_amplification/error-amplification-<unix_ms>.json`
/// and return the written path. Mirrors the seccodeplt timestamped-report idiom.
fn write_report(
    project_root: &std::path::Path,
    report: &ErrorAmpReport,
) -> Result<PathBuf, BenchErrorAmplificationError> {
    let report_dir = project_root
        .join("benchmarks")
        .join("error_amplification");
    fs::create_dir_all(&report_dir).map_err(|e| BenchErrorAmplificationError::ReportDir {
        path: report_dir.display().to_string(),
        source_msg: e.to_string(),
    })?;

    let unix_ms = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis())
        .unwrap_or_default();
    let report_path = report_dir.join(format!("error-amplification-{unix_ms}.json"));

    let body = serde_json::to_string_pretty(report).map_err(|e| {
        BenchErrorAmplificationError::Serialise {
            source_msg: e.to_string(),
        }
    })?;
    fs::write(&report_path, &body).map_err(|e| BenchErrorAmplificationError::OutputIo {
        path: report_path.display().to_string(),
        source_msg: e.to_string(),
    })?;
    Ok(report_path)
}

/// Open the canonical session `ssb.db` and emit ONE `ErrorAmplificationMeasured`
/// beacon via the SHARED orchestrator emit helper, awaited (durable).
/// `Ssb::new` MUST be constructed inside `rt.block_on` (the single-writer task
/// requires a reactor; observable only on a live run).
fn emit_beacon(
    project_root: &std::path::Path,
    report: &ErrorAmpReport,
) -> Result<(), BenchErrorAmplificationError> {
    let db_path = sessions_dir_path(project_root).join("ssb.db");
    if let Some(parent) = db_path.parent() {
        fs::create_dir_all(parent).map_err(|e| BenchErrorAmplificationError::ReportDir {
            path: parent.display().to_string(),
            source_msg: e.to_string(),
        })?;
    }

    let cfg = SsbConfig {
        db_path,
        pool_size: None,
        project_id: "default".to_string(),
        session_id: "cli-bench-error-amplification".to_string(),
    };
    let clock: Arc<dyn elai_core::clock::ClockProvider> = Arc::new(OsClock);

    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .map_err(|e| BenchErrorAmplificationError::Runtime {
            source_msg: e.to_string(),
        })?;

    rt.block_on(async move {
        let ssb = Ssb::new(cfg, clock).map_err(|e| BenchErrorAmplificationError::SsbOpen {
            source_msg: e.to_string(),
        })?;
        emit_error_amplification_measured(&ssb, SSB_AGENT, report)
            .await
            .map_err(|e| BenchErrorAmplificationError::SsbWrite {
                source_msg: e.to_string(),
            })?;
        Ok::<(), BenchErrorAmplificationError>(())
    })
}

// ---------------------------------------------------------------------------
// Unit tests (in-crate; the live `elai bench error-amplification` smoke is
// driven by wiring-contract-validator + the i9 contract, NOT here).
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn write_report_emits_a_nondegenerate_json_under_benchmarks_dir() {
        let harness =
            ErrorPropagationHarness::from_embedded().expect("embedded fixture parses");
        let report = harness.run(true);
        let dir = tempfile::TempDir::new().expect("tempdir");
        let path = write_report(dir.path(), &report).expect("write report");

        assert!(path.exists(), "report file must exist on disk");
        let on_disk = path.to_string_lossy();
        assert!(
            on_disk.contains("benchmarks") && on_disk.contains("error_amplification"),
            "report must live under benchmarks/error_amplification/, got {on_disk}"
        );

        let text = fs::read_to_string(&path).expect("read report");
        let parsed: ErrorAmpReport =
            serde_json::from_str(&text).expect("report round-trips as ErrorAmpReport");
        // Non-degenerate repaired arm: alpha < 1.0, recovered some downstream.
        assert!(
            parsed.alpha_t < 1.0,
            "repaired arm alpha ({}) must contract below 1.0",
            parsed.alpha_t
        );
        assert_eq!(parsed.fixture_blake3.len(), 64, "blake3 hex is 64 chars");
    }

    #[test]
    fn no_repair_arm_amplifies_fully() {
        let harness =
            ErrorPropagationHarness::from_embedded().expect("embedded fixture parses");
        let report = harness.run(false);
        assert_eq!(
            report.alpha_t, 1.0,
            "no-repair arm must propagate fully (alpha == 1.0)"
        );
    }
}
