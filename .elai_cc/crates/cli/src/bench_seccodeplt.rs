//! F-2b — `elai bench seccodeplt [--json]` CLI driver (Slice A3, W-5).
//!
//! Surfaces
//! [`elai_orchestrator::benchmarks::seccodeplt_runner::SecCodePltRunner`] as a
//! user-facing subcommand + CI hook. The benchmark drives the REAL ELAI static
//! write-gate (`vibegate::evaluate_vibegate`) over a SeCodePLT-structured CWE
//! corpus OFFLINE (no model, no sandbox) and computes a before/after
//! insecure-rate plus the gate's recall. `before` = corpus ground-truth
//! insecure-rate across ALL candidate completions; `after` = the static
//! false-negative residual (vulnerable completions the gate did NOT Block);
//! `gate_recall` = (before - after) / before. Both rates are COMPUTED from the
//! scorer, never literals.
//!
//! # HONESTY (grounded-by-default)
//!
//! This is a STATIC-gate insecure-rate, DISTINCT from SeCodePLT's DYNAMIC
//! secure-coding rate (sandbox-executed 3-way label). The dynamic metric +
//! functional pass@1 + the live-model number are a separate, RunPod-gated
//! measurement and are NOT computed here. See
//! `workflow/papers/seccodeplt-interface.md` +
//! `workflow/reasoning/F-2-decisions.md`.
//!
//! # Wiring discipline
//!
//! producer  : `SecCodePltRunner::run` (orchestrator-side, deterministic).
//! consumer  : this file — runs the gate, writes a timestamped
//!             `seccodeplt-<unix_ms>.json` report under
//!             `<cwd>/benchmarks/seccodeplt/`, and emits EXACTLY ONE
//!             `InsecureRateMeasured` SSB beacon (W-2: always-emitted on run;
//!             NO `ELAI_*_ENABLE` gate — this is a bench telemetry beacon, not
//!             a wire-dark security signal).
//! ssb row   : `SsbActionType::InsecureRateMeasured` (one per run).
//!
//! # Exit codes
//!
//! 0 = run completed (report written + SSB row emitted).
//! 1 = infrastructural error (malformed corpus, SSB writer failure, report I/O).
//!
//! # Flags
//!
//!   --json   Print the full report JSON to stdout (in addition to the
//!            on-disk report). Without it, only a one-line summary is printed.
//!
//! # No `.unwrap()` / `.expect()` in non-test code.

use std::fs;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

use elai_core::OsClock;
use elai_orchestrator::benchmarks::seccodeplt_runner::{
    SecCodePltError, SecCodePltReport, SecCodePltRunner,
};
use elai_orchestrator::{Ssb, SsbConfig, SsbPayload};
use elai_orchestrator::ssb::InsecureRateMeasuredPayload;
use elai_core::paths::sessions_dir_path;
use thiserror::Error;

// ---------------------------------------------------------------------------
// Error type (thiserror per CLAUDE.md anti-stale rule).
// ---------------------------------------------------------------------------

#[derive(Debug, Error)]
pub enum BenchSeccodepltError {
    /// The embedded corpus failed to load / validate at construction.
    #[error("seccodeplt runner error: {0}")]
    Runner(#[from] SecCodePltError),
    /// Could not determine the current working directory for the report dir.
    #[error("seccodeplt could not determine cwd: {source_msg}")]
    Cwd { source_msg: String },
    /// Could not create the on-disk report directory.
    #[error("seccodeplt could not create report dir {path}: {source_msg}")]
    ReportDir { path: String, source_msg: String },
    /// Generic I/O failure writing the JSON report.
    #[error("seccodeplt could not write report to {path}: {source_msg}")]
    OutputIo { path: String, source_msg: String },
    /// JSON serialisation failure when rendering the report.
    #[error("seccodeplt could not serialise report: {source_msg}")]
    Serialise { source_msg: String },
    /// Building the SSB async runtime failed.
    #[error("seccodeplt could not build async runtime: {source_msg}")]
    Runtime { source_msg: String },
    /// Opening the SSB single-writer failed.
    #[error("seccodeplt could not open SSB writer: {source_msg}")]
    SsbOpen { source_msg: String },
    /// Writing the InsecureRateMeasured beacon failed.
    #[error("seccodeplt could not write SSB beacon: {source_msg}")]
    SsbWrite { source_msg: String },
}

// ---------------------------------------------------------------------------
// Sync CLI entry.
// ---------------------------------------------------------------------------

/// CLI entry — runs the static-gate insecure-rate benchmark, writes a
/// timestamped JSON report, emits one `InsecureRateMeasured` SSB beacon, and
/// prints a summary (or the full report JSON with `--json`).
///
/// Returns `Ok(())` on a clean run (rc=0). On a handled error it prints a
/// `SECCODEPLT ABORT:` line to stderr and calls `std::process::exit(1)`.
/// `Err(anyhow::Error)` is reserved for failures the top-level dispatcher
/// renders.
pub(crate) fn run(json: bool) -> anyhow::Result<()> {
    match run_inner(json) {
        Ok(()) => Ok(()),
        Err(e) => {
            eprintln!("SECCODEPLT ABORT: {e}");
            std::process::exit(1);
        }
    }
}

/// Core — constructs the REAL runner, runs the gate, writes the report, emits
/// the beacon. Returns the structured error so the entrypoint can render +
/// exit. Public so an integration test can drive it without the process exit.
pub fn run_inner(json: bool) -> Result<(), BenchSeccodepltError> {
    // 1. Construct the REAL runner from the embedded corpus + run the gate.
    let runner = SecCodePltRunner::from_embedded()?;
    let report = runner.run();

    // 2. Write the timestamped JSON report under <cwd>/benchmarks/seccodeplt/.
    let project_root = std::env::current_dir().map_err(|e| BenchSeccodepltError::Cwd {
        source_msg: e.to_string(),
    })?;
    let report_path = write_report(&project_root, &report)?;

    // 3. Emit EXACTLY ONE InsecureRateMeasured SSB beacon (W-2: always-on; no
    //    ELAI_*_ENABLE gate). The writer panics "no reactor running" unless
    //    Ssb::new is constructed INSIDE rt.block_on — observable only on a live
    //    run (cargo check cannot catch it).
    emit_insecure_rate_measured(&project_root, &report)?;

    // 4. Summary line, plus the full report JSON when --json is set.
    println!(
        "SECCODEPLT suite={} n_samples={} before={:.4} after={:.4} gate_recall={:.4} corpus_blake3={}; report written to {}",
        report.suite,
        report.n_samples,
        report.insecure_rate_before,
        report.insecure_rate_after,
        report.gate_recall,
        report.corpus_blake3,
        report_path.display(),
    );
    if json {
        let body =
            serde_json::to_string_pretty(&report).map_err(|e| BenchSeccodepltError::Serialise {
                source_msg: e.to_string(),
            })?;
        println!("{body}");
    }
    Ok(())
}

/// Write the report JSON to `<project_root>/benchmarks/seccodeplt/seccodeplt-<unix_ms>.json`
/// and return the written path. Mirrors the `farm_adv_rm` project-relative
/// `benchmarks/<bench>/` convention + the `drift_cmd` timestamped-report idiom.
fn write_report(
    project_root: &std::path::Path,
    report: &SecCodePltReport,
) -> Result<PathBuf, BenchSeccodepltError> {
    let report_dir = project_root.join("benchmarks").join("seccodeplt");
    fs::create_dir_all(&report_dir).map_err(|e| BenchSeccodepltError::ReportDir {
        path: report_dir.display().to_string(),
        source_msg: e.to_string(),
    })?;

    // Unix-millis timestamp (mirrors handoff.rs UNIX_EPOCH idiom). On a clock
    // earlier than the epoch we fall back to 0 rather than panicking.
    let unix_ms = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis())
        .unwrap_or_default();
    let report_path = report_dir.join(format!("seccodeplt-{unix_ms}.json"));

    let body =
        serde_json::to_string_pretty(report).map_err(|e| BenchSeccodepltError::Serialise {
            source_msg: e.to_string(),
        })?;
    fs::write(&report_path, &body).map_err(|e| BenchSeccodepltError::OutputIo {
        path: report_path.display().to_string(),
        source_msg: e.to_string(),
    })?;
    Ok(report_path)
}

/// Open the canonical session `ssb.db` and emit ONE `InsecureRateMeasured`
/// beacon, awaited (durable). Mirrors `safety_cmd::write_monotonicity_ssb_rows`:
/// `Ssb::new` spawns the single-writer task and MUST be constructed inside
/// `rt.block_on`, else it panics "no reactor running" (only observable on a
/// live run; cargo check passes regardless).
fn emit_insecure_rate_measured(
    project_root: &std::path::Path,
    report: &SecCodePltReport,
) -> Result<(), BenchSeccodepltError> {
    let db_path = sessions_dir_path(project_root).join("ssb.db");
    if let Some(parent) = db_path.parent() {
        fs::create_dir_all(parent).map_err(|e| BenchSeccodepltError::ReportDir {
            path: parent.display().to_string(),
            source_msg: e.to_string(),
        })?;
    }

    let cfg = SsbConfig {
        db_path,
        pool_size: None,
        project_id: "default".to_string(),
        session_id: "cli-bench-seccodeplt".to_string(),
    };
    let clock: Arc<dyn elai_core::clock::ClockProvider> = Arc::new(OsClock);

    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .map_err(|e| BenchSeccodepltError::Runtime {
            source_msg: e.to_string(),
        })?;

    let payload = InsecureRateMeasuredPayload {
        suite: report.suite.clone(),
        model_id: report.model_id.clone(),
        n_samples: report.n_samples,
        insecure_rate_before: report.insecure_rate_before,
        insecure_rate_after: report.insecure_rate_after,
        gate_recall: report.gate_recall,
        seed: report.seed,
        corpus_blake3: report.corpus_blake3.clone(),
    };

    // Ssb::new spawns the single-writer task, so it MUST be constructed inside
    // the runtime context (CLAUDE.md F-2b playbook + safety_cmd precedent).
    rt.block_on(async move {
        let ssb = Ssb::new(cfg, clock).map_err(|e| BenchSeccodepltError::SsbOpen {
            source_msg: e.to_string(),
        })?;
        ssb.write_payload(
            "cli:bench-seccodeplt",
            0,
            SsbPayload::InsecureRateMeasured(payload),
            Vec::new(),
        )
        .await
        .map_err(|e| BenchSeccodepltError::SsbWrite {
            source_msg: e.to_string(),
        })?;
        Ok::<(), BenchSeccodepltError>(())
    })
}

// ---------------------------------------------------------------------------
// Unit tests (in-crate; the live `elai bench seccodeplt --run-trigger` smoke is
// driven by wiring-contract-validator, NOT here — we do not run the binary).
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn write_report_emits_a_nondegenerate_json_under_benchmarks_dir() {
        let runner = SecCodePltRunner::from_embedded().expect("embedded corpus parses");
        let report = runner.run();
        let dir = tempfile::TempDir::new().expect("tempdir");
        let path = write_report(dir.path(), &report).expect("write report");

        assert!(path.exists(), "report file must exist on disk");
        let on_disk = path.to_string_lossy();
        assert!(
            on_disk.contains("benchmarks") && on_disk.contains("seccodeplt"),
            "report must live under benchmarks/seccodeplt/, got {on_disk}"
        );

        let text = fs::read_to_string(&path).expect("read report");
        let parsed: SecCodePltReport =
            serde_json::from_str(&text).expect("report round-trips as SecCodePltReport");
        // Non-degenerate: before > after, both COMPUTED (not literals).
        assert!(
            parsed.insecure_rate_before > parsed.insecure_rate_after,
            "before ({}) must exceed after ({})",
            parsed.insecure_rate_before,
            parsed.insecure_rate_after
        );
        assert!(
            parsed.gate_recall > 0.0 && parsed.gate_recall < 1.0,
            "gate_recall ({}) must be a non-trivial fraction",
            parsed.gate_recall
        );
        assert_eq!(parsed.corpus_blake3.len(), 64, "blake3 hex is 64 chars");
    }

    #[test]
    fn payload_mirrors_report_fields() {
        // Guards the field-by-field copy from SecCodePltReport into the SSB
        // beacon: a renamed/dropped field on either side breaks this.
        let runner = SecCodePltRunner::from_embedded().expect("parse");
        let report = runner.run();
        let payload = InsecureRateMeasuredPayload {
            suite: report.suite.clone(),
            model_id: report.model_id.clone(),
            n_samples: report.n_samples,
            insecure_rate_before: report.insecure_rate_before,
            insecure_rate_after: report.insecure_rate_after,
            gate_recall: report.gate_recall,
            seed: report.seed,
            corpus_blake3: report.corpus_blake3.clone(),
        };
        assert_eq!(payload.suite, report.suite);
        assert_eq!(payload.n_samples, report.n_samples);
        assert_eq!(payload.insecure_rate_before, report.insecure_rate_before);
        assert_eq!(payload.insecure_rate_after, report.insecure_rate_after);
        assert_eq!(payload.gate_recall, report.gate_recall);
        assert_eq!(payload.corpus_blake3, report.corpus_blake3);
    }
}
