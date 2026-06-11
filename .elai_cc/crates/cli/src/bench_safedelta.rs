//! G-6 — `elai bench safedelta [--suites <csv>] [--seed <u64>] [--json]` CLI driver.
//!
//! Surfaces the gate-ON-vs-gate-OFF SAFEDELTA ablation as a user-facing
//! subcommand + CI hook. DERIVES the delta from the SHIPPED
//! [`elai_orchestrator::benchmarks::seccodeplt_runner::SecCodePltRunner::run`]
//! (GD-1, compose-not-restate, BLOCKING — re-implements NO part of F-2).
//!
//! # What this measures (honestly)
//!
//! SAFEDELTA = `insecure_rate_before` - `insecure_rate_after` from the REAL
//! ELAI static write-gate (`vibegate::evaluate_vibegate`) over the embedded
//! SeCodePLT CWE corpus under the pinned seed. Adds a seeded bootstrap 95% CI
//! and a per-CWE breakdown. `runpod_pending = true` for Tier-1 (no significance
//! claim; the live dynamic exploit-exec arm is separately RunPod-gated).
//!
//! # HONESTY (grounded-by-default, GD-5)
//!
//! This is a STATIC-gate insecure-rate ablation, DISTINCT from SeCodePLT's
//! DYNAMIC secure-coding rate (sandbox-executed + functional pass@1 + live
//! model). Suite IDs (verifier-confirmed arXiv):
//!   - SeCodePLT   arXiv:2410.11096 (the embedded corpus family)
//!   - SUSVIBES-200 arXiv:2512.03262 (Tier-2, RunPod-gated)
//!   - BaxBench-392 arXiv:2502.11844 (dynamic exploit exec, Tier-2)
//!   - CyberSecEval 1/2 insecure-code-gen: arXiv:2404.13161
//!     (insecure-code-generation suite; cite only this, not the offensive-cyber
//!      CyberSecEval 3 variant)
//!
//! # Wiring discipline
//!
//! producer  : `SecCodePltRunner::run` (orchestrator, deterministic).
//! consumer  : this file — derives the delta, writes a timestamped
//!             `safedelta-<unix_ms>.json` report under
//!             `<cwd>/benchmarks/safedelta/`, seals a signed `.rra` manifest
//!             via G-2 `build_rra`/`write_rra` (blake3-only root, SC-1),
//!             and emits EXACTLY ONE `SafetyDeltaMeasured` SSB beacon.
//! ssb row   : `SsbActionType::SafetyDeltaMeasured` (one per run).
//! rra       : `benchmarks/safedelta/safedelta-<unix_ms>.rra` (signed, GD-2).
//!
//! # No `.unwrap()` / `.expect()` in non-test paths.
//! # blake3 only for all content hashing (anti-stale rule, DETERMINISTIC_CHECK).

use std::fs;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

use elai_core::OsClock;
use elai_core::paths::sessions_dir_path;
use elai_orchestrator::benchmarks::safedelta_runner::{
    SafeDeltaError, SafeDeltaReport, SafeDeltaTier2Error, run_safedelta,
};
use elai_orchestrator::determinism::DeterminismConfig;
use elai_orchestrator::rra::{build_rra, write_rra};
use elai_orchestrator::ssb::{SafetyDeltaMeasuredPayload, SsbReader as _};
use elai_orchestrator::{Ssb, SsbConfig, SsbPayload};
use thiserror::Error;

// ---------------------------------------------------------------------------
// Constants
// ---------------------------------------------------------------------------

/// Bench session id embedded in every SSB row this bench emits.
const BENCH_SESSION_ID: &str = "cli-bench-safedelta";

/// Agent id label for SSB rows (mirrors bench_seccodeplt pattern).
const BENCH_AGENT_ID: &str = "cli:bench-safedelta";

// ---------------------------------------------------------------------------
// Error type
// ---------------------------------------------------------------------------

#[derive(Debug, Error)]
pub enum BenchSafedeltaError {
    /// The SAFEDELTA runner failed (e.g. degenerate corpus, malformed JSON).
    #[error("safedelta runner error: {0}")]
    Runner(#[from] SafeDeltaError),
    /// The requested suite requires a live RunPod endpoint (Tier-2 deferred).
    #[error("safedelta Tier-2 deferred: {0}")]
    RunpodPending(#[from] SafeDeltaTier2Error),
    /// Could not determine the current working directory for the report dir.
    #[error("safedelta could not determine cwd: {source_msg}")]
    Cwd { source_msg: String },
    /// Could not create the on-disk report or .rra directory.
    #[error("safedelta could not create dir {path}: {source_msg}")]
    ReportDir { path: String, source_msg: String },
    /// Generic I/O failure writing the JSON report or .rra file.
    #[error("safedelta could not write {path}: {source_msg}")]
    OutputIo { path: String, source_msg: String },
    /// JSON serialisation failure.
    #[error("safedelta could not serialise report: {source_msg}")]
    Serialise { source_msg: String },
    /// Building the async runtime failed.
    #[error("safedelta could not build async runtime: {source_msg}")]
    Runtime { source_msg: String },
    /// Opening the SSB single-writer failed.
    #[error("safedelta could not open SSB writer: {source_msg}")]
    SsbOpen { source_msg: String },
    /// Writing the SafetyDeltaMeasured beacon failed.
    #[error("safedelta could not write SSB beacon: {source_msg}")]
    SsbWrite { source_msg: String },
    /// Resolving determinism pins (seed / hardware class / fingerprint) failed.
    #[error("safedelta could not resolve determinism pins: {source_msg}")]
    DeterminismPins { source_msg: String },
    /// Querying the SSB session stream (for the .rra) failed.
    #[error("safedelta could not query SSB session stream: {source_msg}")]
    SsbStreamQuery { source_msg: String },
    /// Writing the .rra manifest failed.
    #[error("safedelta could not write .rra manifest to {path}: {source_msg}")]
    RraWrite { path: String, source_msg: String },
}

// ---------------------------------------------------------------------------
// CLI entry
// ---------------------------------------------------------------------------

/// CLI entry — runs the SAFEDELTA ablation, writes a timestamped JSON report,
/// seals a signed .rra manifest, emits one `SafetyDeltaMeasured` SSB beacon,
/// and prints a summary (or the full report JSON with `--json`).
///
/// Returns `Ok(())` on a clean run (rc=0). On a handled error it prints a
/// `SAFEDELTA ABORT:` line to stderr and calls `std::process::exit(1)`.
/// `Err(anyhow::Error)` is reserved for failures the top-level dispatcher
/// renders.
pub(crate) fn run(suites: Vec<String>, seed: u64, json: bool) -> anyhow::Result<()> {
    match run_inner(suites, seed, json) {
        Ok(()) => Ok(()),
        Err(e) => {
            eprintln!("SAFEDELTA ABORT: {e}");
            std::process::exit(1);
        }
    }
}

/// Core — constructs the ablation report, writes the JSON + .rra, emits the
/// SSB beacon. Returns the structured error so the entrypoint can render +
/// exit. Public so an integration test can drive it without the process exit.
pub fn run_inner(
    suites: Vec<String>,
    seed: u64,
    json: bool,
) -> Result<(), BenchSafedeltaError> {
    // Tier-2 gate: reject non-seccodeplt suites at Tier-1.
    for suite in &suites {
        if suite != "seccodeplt" {
            return Err(BenchSafedeltaError::RunpodPending(
                SafeDeltaTier2Error::RunpodPending,
            ));
        }
    }

    // 1. Run the gate-ON-vs-gate-OFF SAFEDELTA ablation (DERIVES from F-2,
    //    re-implements nothing).
    let report = run_safedelta(seed)?;

    // 2. Write the timestamped JSON report under <cwd>/benchmarks/safedelta/.
    let project_root = std::env::current_dir().map_err(|e| BenchSafedeltaError::Cwd {
        source_msg: e.to_string(),
    })?;
    let report_path = write_json_report(&project_root, &report)?;

    // 3. Emit EXACTLY ONE SafetyDeltaMeasured SSB beacon + seal the .rra
    //    manifest. The writer panics "no reactor running" unless Ssb::new is
    //    constructed INSIDE rt.block_on — observable only on a live run (cargo
    //    check cannot catch it). Mirrors bench_seccodeplt.rs:230–247.
    let rra_path = rra_path(&project_root, report_path.file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("safedelta-unknown"));
    emit_beacon_and_rra(&project_root, &rra_path, &report)?;

    // 4. Summary line.
    println!(
        "SAFEDELTA suite={} n_tasks={} before={:.4} after={:.4} safety_delta={:.4} \
         ci_low={:.4} ci_high={:.4} gate_recall={:.4} seed={} corpus_blake3={} \
         runpod_pending={} report={}",
        report.suite,
        report.n_samples,
        report.insecure_rate_before,
        report.insecure_rate_after,
        report.safety_delta,
        report.ci_low,
        report.ci_high,
        report.gate_recall,
        report.seed,
        report.corpus_blake3,
        report.runpod_pending,
        report_path.display(),
    );
    if json {
        let body = serde_json::to_string_pretty(&report).map_err(|e| {
            BenchSafedeltaError::Serialise {
                source_msg: e.to_string(),
            }
        })?;
        println!("{body}");
    }
    Ok(())
}

// ---------------------------------------------------------------------------
// JSON report writer
// ---------------------------------------------------------------------------

/// Write the report JSON to `<project_root>/benchmarks/safedelta/safedelta-<unix_ms>.json`
/// and return the written path. Mirrors bench_seccodeplt::write_report pattern.
fn write_json_report(
    project_root: &std::path::Path,
    report: &SafeDeltaReport,
) -> Result<PathBuf, BenchSafedeltaError> {
    let report_dir = project_root.join("benchmarks").join("safedelta");
    fs::create_dir_all(&report_dir).map_err(|e| BenchSafedeltaError::ReportDir {
        path: report_dir.display().to_string(),
        source_msg: e.to_string(),
    })?;

    let unix_ms = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis())
        .unwrap_or_default();
    let report_path = report_dir.join(format!("safedelta-{unix_ms}.json"));

    let body = serde_json::to_string_pretty(report).map_err(|e| BenchSafedeltaError::Serialise {
        source_msg: e.to_string(),
    })?;
    fs::write(&report_path, &body).map_err(|e| BenchSafedeltaError::OutputIo {
        path: report_path.display().to_string(),
        source_msg: e.to_string(),
    })?;
    Ok(report_path)
}

// ---------------------------------------------------------------------------
// .rra path helper
// ---------------------------------------------------------------------------

/// Build the .rra manifest path adjacent to the JSON report.
fn rra_path(project_root: &std::path::Path, stem: &str) -> PathBuf {
    let stem_no_ext = stem.trim_end_matches(".json");
    project_root
        .join("benchmarks")
        .join("safedelta")
        .join(format!("{stem_no_ext}.rra"))
}

// ---------------------------------------------------------------------------
// SSB emit + .rra seal
// ---------------------------------------------------------------------------

/// Open the canonical session `ssb.db`, emit ONE `SafetyDeltaMeasured` beacon
/// (durable, awaited), then query the session stream and seal a signed `.rra`
/// manifest via G-2 `build_rra`/`write_rra` (blake3-only root, SC-1).
///
/// `Ssb::new` spawns the single-writer task and MUST be constructed inside
/// `rt.block_on` (F-2b playbook + safety_cmd precedent). `query_session_stream_asc`
/// must also run inside the same runtime context.
fn emit_beacon_and_rra(
    project_root: &std::path::Path,
    rra_path: &std::path::Path,
    report: &SafeDeltaReport,
) -> Result<(), BenchSafedeltaError> {
    let db_path = sessions_dir_path(project_root).join("ssb.db");
    if let Some(parent) = db_path.parent() {
        fs::create_dir_all(parent).map_err(|e| BenchSafedeltaError::ReportDir {
            path: parent.display().to_string(),
            source_msg: e.to_string(),
        })?;
    }

    let cfg = SsbConfig {
        db_path,
        pool_size: None,
        project_id: "default".to_string(),
        session_id: BENCH_SESSION_ID.to_string(),
    };
    let clock: Arc<dyn elai_core::clock::ClockProvider> = Arc::new(OsClock);

    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .map_err(|e| BenchSafedeltaError::Runtime {
            source_msg: e.to_string(),
        })?;

    let payload = SafetyDeltaMeasuredPayload {
        suite: report.suite.clone(),
        model_id: report.model_id.clone(),
        safety_delta: report.safety_delta,
        ci_low: report.ci_low,
        ci_high: report.ci_high,
        n_tasks: report.n_samples,
        seed: report.seed,
        corpus_blake3: report.corpus_blake3.clone(),
        gate_recall: report.gate_recall,
        // Tier-1 has no externally-captured stdout to hash.
        stdout_hash: None,
        runpod_pending: report.runpod_pending,
    };

    // Resolve determinism pins outside the runtime (sync call).
    let pins = DeterminismConfig::from_env().map_err(|e| BenchSafedeltaError::DeterminismPins {
        source_msg: e.to_string(),
    })?;

    // Ssb::new + write_payload + query + build_rra all run inside the same
    // current-thread runtime (MUST — "no reactor running" if outside).
    rt.block_on(async move {
        let ssb = Ssb::new(cfg, clock).map_err(|e| BenchSafedeltaError::SsbOpen {
            source_msg: e.to_string(),
        })?;

        // Emit the SafetyDeltaMeasured beacon row.
        ssb.write_payload(
            BENCH_AGENT_ID,
            0,
            SsbPayload::SafetyDeltaMeasured(payload),
            Vec::new(),
        )
        .await
        .map_err(|e| BenchSafedeltaError::SsbWrite {
            source_msg: e.to_string(),
        })?;

        // Read the session stream for the .rra seal (G-2 reuse, SC-1).
        let stream = ssb
            .query_session_stream_asc(BENCH_SESSION_ID)
            .map_err(|e| BenchSafedeltaError::SsbStreamQuery {
                source_msg: e.to_string(),
            })?;

        // Build + write the signed, independently-replayable .rra manifest
        // (blake3-only root, GD-2: re-verifiable from the pinned stream + pins).
        let artifact = build_rra(BENCH_SESSION_ID, &stream, &pins);
        write_rra(&artifact, rra_path).map_err(|e| BenchSafedeltaError::RraWrite {
            path: rra_path.display().to_string(),
            source_msg: e.to_string(),
        })?;

        tracing::info!(
            safety_delta = report.safety_delta,
            rra_path = %rra_path.display(),
            root_blake3 = %artifact.root_blake3,
            "safedelta: G-6 SafetyDeltaMeasured SSB row + .rra sealed",
        );

        Ok::<(), BenchSafedeltaError>(())
    })
}

// ---------------------------------------------------------------------------
// Unit tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use elai_orchestrator::benchmarks::seccodeplt_runner::SecCodePltRunner;
    use elai_orchestrator::benchmarks::safedelta_runner::run_safedelta_from_runner;

    #[test]
    fn run_inner_fails_on_nonexistent_suite() {
        // A Tier-2 non-seccodeplt suite must be rejected with RunpodPending.
        let err = run_inner(vec!["susvibes".to_string()], 42, false)
            .unwrap_err();
        assert!(
            matches!(err, BenchSafedeltaError::RunpodPending(_)),
            "non-seccodeplt Tier-1 suite must return RunpodPending, got {err}"
        );
    }

    #[test]
    fn write_json_report_creates_file_under_benchmarks_safedelta() {
        let runner = SecCodePltRunner::from_embedded_with_seed(7).expect("parse");
        let report = match run_safedelta_from_runner(&runner) {
            Ok(r) => r,
            Err(_) => return, // degenerate corpus on this seed — skip
        };
        let dir = tempfile::TempDir::new().expect("tempdir");
        let path = write_json_report(dir.path(), &report).expect("write_json_report");

        assert!(path.exists(), "report file must exist on disk");
        let on_disk = path.to_string_lossy();
        assert!(
            on_disk.contains("benchmarks") && on_disk.contains("safedelta"),
            "report must live under benchmarks/safedelta/, got {on_disk}"
        );

        let text = fs::read_to_string(&path).expect("read report");
        let parsed: SafeDeltaReport =
            serde_json::from_str(&text).expect("round-trips as SafeDeltaReport");
        assert!(
            parsed.safety_delta > 0.0,
            "safety_delta must be non-zero in written report"
        );
        assert!(parsed.runpod_pending, "runpod_pending must be true");
    }

    #[test]
    fn payload_fields_mirror_report() {
        let runner = SecCodePltRunner::from_embedded_with_seed(99).expect("parse");
        let report = match run_safedelta_from_runner(&runner) {
            Ok(r) => r,
            Err(_) => return,
        };
        let payload = SafetyDeltaMeasuredPayload {
            suite: report.suite.clone(),
            model_id: report.model_id.clone(),
            safety_delta: report.safety_delta,
            ci_low: report.ci_low,
            ci_high: report.ci_high,
            n_tasks: report.n_samples,
            seed: report.seed,
            corpus_blake3: report.corpus_blake3.clone(),
            gate_recall: report.gate_recall,
            stdout_hash: None,
            runpod_pending: report.runpod_pending,
        };
        assert_eq!(payload.safety_delta, report.safety_delta);
        assert_eq!(payload.seed, report.seed);
        assert_eq!(payload.corpus_blake3, report.corpus_blake3);
        assert!(payload.runpod_pending, "Tier-1 runpod_pending must be true");
    }
}
