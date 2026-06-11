//! H-30 — `elai bench staleness [--json] [--broken-retrieval]` CLI driver.
//!
//! Surfaces
//! [`elai_orchestrator::benchmarks::staleness_runner::StalenessEvalRunner`] as
//! a user-facing subcommand + CI hook. The benchmark drives the SHIPPED
//! bi-temporal memory substrate (`FactStore` + `audit_conflicts` +
//! `get_relevant_facts`, REUSE GD-1) over a labeled supersession fixture in an
//! ISOLATED in-memory store OFFLINE (no model, no live store, no network) and
//! computes the two grounded STALE (arXiv:2605.06527) rates:
//!   * `sr_rate`  -- State Resolution (RECOGNITION): the supersession path
//!     closed the stale row.
//!   * `ipa_rate` -- Implicit Policy Adaptation (APPLICATION): retrieval served
//!     the fresh value and NEVER the stale one.
//!
//! INVARIANT `sr_rate >= ipa_rate` (application cannot exceed recognition).
//!
//! The `--broken-retrieval` arm is the R2 negative control: a retriever that
//! reads include-closed rows so a recognized-stale row LEAKS
//! (`stale_fact_reached_prompt > 0`). A broken-arm run that still reports 0 is
//! itself a FAIL (the instrument failed to scream) and the runner flags
//! `instrument_failed`.
//!
//! # Wiring discipline
//!
//! producer  : `StalenessEvalRunner::run` (orchestrator-side, deterministic).
//! consumer  : this file — runs the eval, writes a timestamped
//!             `staleness-<unix_ms>.json` report under
//!             `<cwd>/benchmarks/staleness/`, and emits EXACTLY ONE
//!             `StalenessEvalMeasured` SSB beacon (default-ON on run; NO
//!             `ELAI_*_ENABLE` gate — this is a measure-not-gate instrument).
//! ssb row   : `SsbActionType::StalenessEvalMeasured` (one per run).
//!
//! # Exit codes
//!
//! 0 = run completed (report written + SSB row emitted).
//! 1 = infrastructural error (malformed corpus, store error, SSB writer
//!     failure, report I/O).
//!
//! # No `.unwrap()` / `.expect()` in non-test code.

use std::fs;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

use elai_core::paths::sessions_dir_path;
use elai_core::OsClock;
use elai_orchestrator::benchmarks::staleness_runner::{
    StalenessEvalError, StalenessEvalReport, StalenessEvalRunner,
};
use elai_orchestrator::ssb::StalenessEvalMeasuredPayload;
use elai_orchestrator::{Ssb, SsbConfig, SsbPayload};
use thiserror::Error;

// ---------------------------------------------------------------------------
// Error type (thiserror per CLAUDE.md anti-stale rule).
// ---------------------------------------------------------------------------

#[derive(Debug, Error)]
pub enum BenchStalenessError {
    /// The embedded corpus / fixture store failed to load / drive.
    #[error("staleness runner error: {0}")]
    Runner(#[from] StalenessEvalError),
    /// Could not determine the current working directory for the report dir.
    #[error("staleness could not determine cwd: {source_msg}")]
    Cwd { source_msg: String },
    /// Could not create the on-disk report directory.
    #[error("staleness could not create report dir {path}: {source_msg}")]
    ReportDir { path: String, source_msg: String },
    /// Generic I/O failure writing the JSON report.
    #[error("staleness could not write report to {path}: {source_msg}")]
    OutputIo { path: String, source_msg: String },
    /// JSON serialisation failure when rendering the report.
    #[error("staleness could not serialise report: {source_msg}")]
    Serialise { source_msg: String },
    /// Building the SSB async runtime failed.
    #[error("staleness could not build async runtime: {source_msg}")]
    Runtime { source_msg: String },
    /// Opening the SSB single-writer failed.
    #[error("staleness could not open SSB writer: {source_msg}")]
    SsbOpen { source_msg: String },
    /// Writing the StalenessEvalMeasured beacon failed.
    #[error("staleness could not write SSB beacon: {source_msg}")]
    SsbWrite { source_msg: String },
}

// ---------------------------------------------------------------------------
// Sync CLI entry.
// ---------------------------------------------------------------------------

/// CLI entry — runs the staleness instrument, writes a timestamped JSON report,
/// emits one `StalenessEvalMeasured` SSB beacon, and prints a summary (or the
/// full report JSON with `--json`).
///
/// Returns `Ok(())` on a clean run (rc=0). On a handled error it prints a
/// `STALENESS ABORT:` line to stderr and calls `std::process::exit(1)`.
pub(crate) fn run(json: bool, broken_retrieval: bool) -> anyhow::Result<()> {
    match run_inner(json, broken_retrieval) {
        Ok(()) => Ok(()),
        Err(e) => {
            eprintln!("STALENESS ABORT: {e}");
            std::process::exit(1);
        }
    }
}

/// Core — constructs the REAL runner, runs the eval, writes the report, emits
/// the beacon. Public so an integration test can drive it without the process
/// exit.
pub fn run_inner(json: bool, broken_retrieval: bool) -> Result<(), BenchStalenessError> {
    // 1. Construct the REAL runner from the embedded fixture corpus + run the
    //    eval against the SHIPPED memory substrate (isolated in-memory store).
    let runner = StalenessEvalRunner::from_embedded(broken_retrieval)?;
    let report = runner.run()?;

    // 2. Write the timestamped JSON report under <cwd>/benchmarks/staleness/.
    let project_root = std::env::current_dir().map_err(|e| BenchStalenessError::Cwd {
        source_msg: e.to_string(),
    })?;
    let report_path = write_report(&project_root, &report)?;

    // 3. Emit EXACTLY ONE StalenessEvalMeasured SSB beacon (default-ON; no
    //    ELAI_*_ENABLE gate). Ssb::new spawns the single-writer task and MUST be
    //    constructed inside rt.block_on (else it panics "no reactor running" —
    //    only observable on a live run; cargo check passes regardless).
    emit_staleness_eval_measured(&project_root, &report)?;

    // 4. Summary line, plus the full report JSON when --json is set.
    println!(
        "STALENESS suite={} n_cases={} sr_rate={:.4} ipa_rate={:.4} gap={:.4} stale_reached={} broken_arm={} instrument_failed={} corpus_blake3={}; report written to {}",
        report.suite,
        report.n_cases,
        report.sr_rate,
        report.ipa_rate,
        report.recognition_application_gap,
        report.stale_fact_reached_prompt,
        report.broken_arm,
        report.instrument_failed,
        report.corpus_blake3,
        report_path.display(),
    );
    if json {
        let body =
            serde_json::to_string_pretty(&report).map_err(|e| BenchStalenessError::Serialise {
                source_msg: e.to_string(),
            })?;
        println!("{body}");
    }
    Ok(())
}

/// Write the report JSON to
/// `<project_root>/benchmarks/staleness/staleness-<unix_ms>.json` and return the
/// written path. Mirrors the `bench_seccodeplt` project-relative
/// `benchmarks/<bench>/` convention.
fn write_report(
    project_root: &std::path::Path,
    report: &StalenessEvalReport,
) -> Result<PathBuf, BenchStalenessError> {
    let report_dir = project_root.join("benchmarks").join("staleness");
    fs::create_dir_all(&report_dir).map_err(|e| BenchStalenessError::ReportDir {
        path: report_dir.display().to_string(),
        source_msg: e.to_string(),
    })?;

    // Unix-millis timestamp (mirrors bench_seccodeplt UNIX_EPOCH idiom). On a
    // clock earlier than the epoch we fall back to 0 rather than panicking. The
    // timestamp is ONLY the on-disk file name — it never enters the report
    // payload, so the report stays byte-reproducible under ELAI_SEED.
    let unix_ms = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis())
        .unwrap_or_default();
    let report_path = report_dir.join(format!("staleness-{unix_ms}.json"));

    let body =
        serde_json::to_string_pretty(report).map_err(|e| BenchStalenessError::Serialise {
            source_msg: e.to_string(),
        })?;
    fs::write(&report_path, &body).map_err(|e| BenchStalenessError::OutputIo {
        path: report_path.display().to_string(),
        source_msg: e.to_string(),
    })?;
    Ok(report_path)
}

/// Open the canonical session `ssb.db` and emit ONE `StalenessEvalMeasured`
/// beacon, awaited (durable). Mirrors `bench_seccodeplt::emit_insecure_rate_measured`:
/// `Ssb::new` spawns the single-writer task and MUST be constructed inside
/// `rt.block_on`, else it panics "no reactor running" (only observable on a live
/// run; cargo check passes regardless).
fn emit_staleness_eval_measured(
    project_root: &std::path::Path,
    report: &StalenessEvalReport,
) -> Result<(), BenchStalenessError> {
    let db_path = sessions_dir_path(project_root).join("ssb.db");
    if let Some(parent) = db_path.parent() {
        fs::create_dir_all(parent).map_err(|e| BenchStalenessError::ReportDir {
            path: parent.display().to_string(),
            source_msg: e.to_string(),
        })?;
    }

    let cfg = SsbConfig {
        db_path,
        pool_size: None,
        project_id: "default".to_string(),
        session_id: "cli-bench-staleness".to_string(),
    };
    let clock: Arc<dyn elai_core::clock::ClockProvider> = Arc::new(OsClock);

    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .map_err(|e| BenchStalenessError::Runtime {
            source_msg: e.to_string(),
        })?;

    let payload = StalenessEvalMeasuredPayload {
        suite: report.suite.clone(),
        n_cases: report.n_cases,
        sr_rate: report.sr_rate,
        ipa_rate: report.ipa_rate,
        recognition_application_gap: report.recognition_application_gap,
        stale_fact_reached_prompt: report.stale_fact_reached_prompt,
        total_retrievals: report.total_retrievals,
        broken_arm: report.broken_arm,
        instrument_failed: report.instrument_failed,
        seed: report.seed,
        corpus_blake3: report.corpus_blake3.clone(),
    };

    rt.block_on(async move {
        let ssb = Ssb::new(cfg, clock).map_err(|e| BenchStalenessError::SsbOpen {
            source_msg: e.to_string(),
        })?;
        ssb.write_payload(
            "cli:bench-staleness",
            0,
            SsbPayload::StalenessEvalMeasured(payload),
            Vec::new(),
        )
        .await
        .map_err(|e| BenchStalenessError::SsbWrite {
            source_msg: e.to_string(),
        })?;
        Ok::<(), BenchStalenessError>(())
    })
}

// ---------------------------------------------------------------------------
// Unit tests (in-crate; the live `elai bench staleness` smoke is driven by the
// h30 contract + the manual real-binary run — we do not run the binary here).
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn write_report_emits_json_under_benchmarks_dir() {
        let runner = StalenessEvalRunner::from_embedded(false).expect("embedded corpus parses");
        let report = runner.run().expect("clean run");
        let dir = tempfile::TempDir::new().expect("tempdir");
        let path = write_report(dir.path(), &report).expect("write report");

        assert!(path.exists(), "report file must exist on disk");
        let on_disk = path.to_string_lossy();
        assert!(
            on_disk.contains("benchmarks") && on_disk.contains("staleness"),
            "report must live under benchmarks/staleness/, got {on_disk}"
        );

        let text = fs::read_to_string(&path).expect("read report");
        let parsed: StalenessEvalReport =
            serde_json::from_str(&text).expect("report round-trips as StalenessEvalReport");
        // Clean path: no stale row leaks; the grounded invariant holds.
        assert_eq!(parsed.stale_fact_reached_prompt, 0, "clean path leaks nothing");
        assert!(parsed.sr_rate >= parsed.ipa_rate, "SR must be >= IPA");
        assert_eq!(parsed.corpus_blake3.len(), 64, "blake3 hex is 64 chars");
    }

    #[test]
    fn payload_mirrors_report_fields() {
        // Guards the field-by-field copy from StalenessEvalReport into the SSB
        // beacon: a renamed/dropped field on either side breaks this.
        let runner = StalenessEvalRunner::from_embedded(false).expect("parse");
        let report = runner.run().expect("run");
        let payload = StalenessEvalMeasuredPayload {
            suite: report.suite.clone(),
            n_cases: report.n_cases,
            sr_rate: report.sr_rate,
            ipa_rate: report.ipa_rate,
            recognition_application_gap: report.recognition_application_gap,
            stale_fact_reached_prompt: report.stale_fact_reached_prompt,
            total_retrievals: report.total_retrievals,
            broken_arm: report.broken_arm,
            instrument_failed: report.instrument_failed,
            seed: report.seed,
            corpus_blake3: report.corpus_blake3.clone(),
        };
        assert_eq!(payload.suite, report.suite);
        assert_eq!(payload.n_cases, report.n_cases);
        assert_eq!(payload.sr_rate, report.sr_rate);
        assert_eq!(payload.ipa_rate, report.ipa_rate);
        assert_eq!(payload.stale_fact_reached_prompt, report.stale_fact_reached_prompt);
        assert_eq!(payload.broken_arm, report.broken_arm);
        assert_eq!(payload.corpus_blake3, report.corpus_blake3);
    }

    #[test]
    fn broken_arm_report_has_stale_reached_gt_zero() {
        let runner = StalenessEvalRunner::from_embedded(true).expect("parse broken");
        let report = runner.run().expect("broken run");
        assert!(report.broken_arm, "broken arm flag set");
        assert!(
            report.stale_fact_reached_prompt > 0,
            "broken arm must leak >=1 superseded row (R2 scream)"
        );
        assert!(!report.instrument_failed, "broken arm leaked, not an instrument failure");
    }
}
