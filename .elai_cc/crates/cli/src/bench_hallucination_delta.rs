//! PS-3 — `elai bench hallucination-delta [--suites <csv>] [--seed <u64>] [--json]` CLI driver.
//!
//! FAITHFUL MIRROR of G-6 [`bench_safedelta`]: surfaces the gate-ON-vs-gate-OFF
//! hallucination-delta ablation as a user-facing subcommand + CI hook. Tier-1
//! runs the embedded fixture corpus (pure computation, NO live model call).
//!
//! # What this measures (honestly)
//!
//! `hallucination_delta = hallucination_rate_gate_off - hallucination_rate_gate_on`
//! over the embedded deterministic fixture corpus. Both arms are DETERMINISTIC
//! from the embedded corpus + seed (fixture grade). `runpod_pending = true`
//! for Tier-1 (no significance claim; the live CGG ablation arm is separately
//! RunPod-gated).
//!
//! # Wiring discipline
//!
//! producer  : `run_hallucination_delta` (orchestrator, deterministic).
//! consumer  : this file — writes a timestamped
//!             `hallucination_delta-<unix_ms>.json` report under
//!             `<cwd>/benchmarks/hallucination_delta/`, seals a signed `.rra`
//!             manifest via G-2 `build_rra`/`write_rra` (blake3-only root,
//!             SC-1), and emits EXACTLY ONE `HallucinationDeltaMeasured` SSB
//!             beacon.
//! ssb row   : `SsbActionType::HallucinationDeltaMeasured` (one per run).
//! rra       : `benchmarks/hallucination_delta/hallucination_delta-<unix_ms>.rra`.
//!
//! # No `.unwrap()` / `.expect()` in non-test paths.
//! # blake3 only for all content hashing (anti-stale rule, DETERMINISTIC_CHECK).

use std::fs;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

use elai_core::OsClock;
use elai_core::paths::sessions_dir_path;
use elai_orchestrator::benchmarks::hallucination_delta_runner::{
    HallucinationDeltaError, HallucinationDeltaReport, HallucinationDeltaTier2Error,
    run_hallucination_delta,
};
use elai_orchestrator::determinism::DeterminismConfig;
use elai_orchestrator::rra::{build_rra, write_rra};
use elai_orchestrator::ssb::{HallucinationDeltaMeasuredPayload, SsbReader as _};
use elai_orchestrator::{Ssb, SsbConfig, SsbPayload};
use thiserror::Error;

// ---------------------------------------------------------------------------
// Constants
// ---------------------------------------------------------------------------

/// Bench session id embedded in every SSB row this bench emits.
const BENCH_SESSION_ID: &str = "cli-bench-hallucination-delta";

/// Agent id label for SSB rows (mirrors bench_safedelta pattern).
const BENCH_AGENT_ID: &str = "cli:bench-hallucination-delta";

// ---------------------------------------------------------------------------
// Error type
// ---------------------------------------------------------------------------

#[derive(Debug, Error)]
pub enum BenchHallucinationDeltaError {
    /// The hallucination-delta runner failed (e.g. degenerate corpus).
    #[error("hallucination-delta runner error: {0}")]
    Runner(#[from] HallucinationDeltaError),
    /// The requested suite requires a live RunPod endpoint (Tier-2 deferred).
    #[error("hallucination-delta Tier-2 deferred: {0}")]
    RunpodPending(#[from] HallucinationDeltaTier2Error),
    /// `--endpoint` was passed but this is a fixture-tier-only slice.
    #[error(
        "hallucination-delta live tier deferred (fixture-grade only), rerun without --endpoint"
    )]
    LiveTierDeferred,
    /// Could not determine the current working directory for the report dir.
    #[error("hallucination-delta could not determine cwd: {source_msg}")]
    Cwd { source_msg: String },
    /// Could not create the on-disk report or .rra directory.
    #[error("hallucination-delta could not create dir {path}: {source_msg}")]
    ReportDir { path: String, source_msg: String },
    /// Generic I/O failure writing the JSON report or .rra file.
    #[error("hallucination-delta could not write {path}: {source_msg}")]
    OutputIo { path: String, source_msg: String },
    /// JSON serialisation failure.
    #[error("hallucination-delta could not serialise report: {source_msg}")]
    Serialise { source_msg: String },
    /// Building the async runtime failed.
    #[error("hallucination-delta could not build async runtime: {source_msg}")]
    Runtime { source_msg: String },
    /// Opening the SSB single-writer failed.
    #[error("hallucination-delta could not open SSB writer: {source_msg}")]
    SsbOpen { source_msg: String },
    /// Writing the HallucinationDeltaMeasured beacon failed.
    #[error("hallucination-delta could not write SSB beacon: {source_msg}")]
    SsbWrite { source_msg: String },
    /// Resolving determinism pins (seed / hardware class / fingerprint) failed.
    #[error("hallucination-delta could not resolve determinism pins: {source_msg}")]
    DeterminismPins { source_msg: String },
    /// Querying the SSB session stream (for the .rra) failed.
    #[error("hallucination-delta could not query SSB session stream: {source_msg}")]
    SsbStreamQuery { source_msg: String },
    /// Writing the .rra manifest failed.
    #[error("hallucination-delta could not write .rra manifest to {path}: {source_msg}")]
    RraWrite { path: String, source_msg: String },
}

// ---------------------------------------------------------------------------
// CLI entry
// ---------------------------------------------------------------------------

/// CLI entry — runs the hallucination-delta ablation, writes a timestamped
/// JSON report, seals a signed .rra manifest, emits one
/// `HallucinationDeltaMeasured` SSB beacon, and prints a summary (or the full
/// report JSON with `--json`).
///
/// Returns `Ok(())` on a clean run (rc=0). On a handled error it prints a
/// `HALLUCINATION-DELTA ABORT:` line to stderr and calls
/// `std::process::exit(1)`.
pub(crate) fn run(
    suites: Vec<String>,
    seed: u64,
    json: bool,
    endpoint: Option<String>,
    model: Option<String>,
) -> anyhow::Result<()> {
    match run_inner(suites, seed, json, endpoint, model) {
        Ok(()) => Ok(()),
        Err(e) => {
            eprintln!("HALLUCINATION-DELTA ABORT: {e}");
            std::process::exit(1);
        }
    }
}

/// Core — constructs the ablation report, writes the JSON + .rra, emits the
/// SSB beacon. Public so an integration test can drive it without the exit.
pub fn run_inner(
    suites: Vec<String>,
    seed: u64,
    json: bool,
    endpoint: Option<String>,
    _model: Option<String>,
) -> Result<(), BenchHallucinationDeltaError> {
    // Tier-2 gate: reject non-cgg suites at Tier-1.
    for suite in &suites {
        if suite != "cgg" {
            return Err(BenchHallucinationDeltaError::RunpodPending(
                HallucinationDeltaTier2Error::RunpodPending,
            ));
        }
    }

    // Fixture-tier-only: --endpoint is deferred. Do NOT call run_live_probe.
    if endpoint.is_some() {
        return Err(BenchHallucinationDeltaError::LiveTierDeferred);
    }

    // 1. Run the gate-ON-vs-gate-OFF hallucination-delta ablation.
    let report = run_hallucination_delta(seed)?;

    // 2. Write the timestamped JSON report under
    //    <cwd>/benchmarks/hallucination_delta/.
    let project_root = std::env::current_dir().map_err(|e| BenchHallucinationDeltaError::Cwd {
        source_msg: e.to_string(),
    })?;
    let report_path = write_json_report(&project_root, &report)?;

    // 3. Emit EXACTLY ONE HallucinationDeltaMeasured SSB beacon + seal the
    //    .rra manifest. Ssb::new must be constructed INSIDE rt.block_on.
    let rra_path = rra_path(
        &project_root,
        report_path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("hallucination_delta-unknown"),
    );
    emit_beacon_and_rra(&project_root, &rra_path, &report)?;

    // 4. Summary line.
    println!(
        "HALLUCINATION-DELTA suite={} n_tasks={} off={:.4} on={:.4} \
         hallucination_delta={:.4} ci_low={:.4} ci_high={:.4} gate_recall={:.4} \
         seed={} corpus_blake3={} runpod_pending={} report={}",
        report.suite,
        report.n_samples,
        report.hallucination_rate_gate_off,
        report.hallucination_rate_gate_on,
        report.hallucination_delta,
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
            BenchHallucinationDeltaError::Serialise {
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

/// Write the report JSON to
/// `<project_root>/benchmarks/hallucination_delta/hallucination_delta-<unix_ms>.json`
/// and return the written path.
fn write_json_report(
    project_root: &std::path::Path,
    report: &HallucinationDeltaReport,
) -> Result<PathBuf, BenchHallucinationDeltaError> {
    let report_dir = project_root.join("benchmarks").join("hallucination_delta");
    fs::create_dir_all(&report_dir).map_err(|e| BenchHallucinationDeltaError::ReportDir {
        path: report_dir.display().to_string(),
        source_msg: e.to_string(),
    })?;

    let unix_ms = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis())
        .unwrap_or_default();
    let report_path = report_dir.join(format!("hallucination_delta-{unix_ms}.json"));

    let body = serde_json::to_string_pretty(report).map_err(|e| {
        BenchHallucinationDeltaError::Serialise {
            source_msg: e.to_string(),
        }
    })?;
    fs::write(&report_path, &body).map_err(|e| BenchHallucinationDeltaError::OutputIo {
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
        .join("hallucination_delta")
        .join(format!("{stem_no_ext}.rra"))
}

// ---------------------------------------------------------------------------
// SSB emit + .rra seal
// ---------------------------------------------------------------------------

/// Open the canonical session `ssb.db`, emit ONE `HallucinationDeltaMeasured`
/// beacon (durable, awaited), then query the session stream and seal a signed
/// `.rra` manifest via G-2 `build_rra`/`write_rra` (blake3-only root, SC-1).
///
/// `Ssb::new` spawns the single-writer task and MUST be constructed inside
/// `rt.block_on` (mirrors bench_safedelta.rs::emit_beacon_and_rra).
fn emit_beacon_and_rra(
    project_root: &std::path::Path,
    rra_path: &std::path::Path,
    report: &HallucinationDeltaReport,
) -> Result<(), BenchHallucinationDeltaError> {
    let db_path = sessions_dir_path(project_root).join("ssb.db");
    if let Some(parent) = db_path.parent() {
        fs::create_dir_all(parent).map_err(|e| BenchHallucinationDeltaError::ReportDir {
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
        .map_err(|e| BenchHallucinationDeltaError::Runtime {
            source_msg: e.to_string(),
        })?;

    let payload = HallucinationDeltaMeasuredPayload {
        suite: report.suite.clone(),
        model_id: report.model_id.clone(),
        hallucination_delta: report.hallucination_delta,
        ci_low: report.ci_low,
        ci_high: report.ci_high,
        n_tasks: report.n_samples,
        seed: report.seed,
        corpus_blake3: report.corpus_blake3.clone(),
        gate_recall: report.gate_recall,
        stdout_hash: None,
        runpod_pending: report.runpod_pending,
    };

    let pins = DeterminismConfig::from_env().map_err(|e| {
        BenchHallucinationDeltaError::DeterminismPins {
            source_msg: e.to_string(),
        }
    })?;

    rt.block_on(async move {
        let ssb = Ssb::new(cfg, clock).map_err(|e| BenchHallucinationDeltaError::SsbOpen {
            source_msg: e.to_string(),
        })?;

        ssb.write_payload(
            BENCH_AGENT_ID,
            0,
            SsbPayload::HallucinationDeltaMeasured(payload),
            Vec::new(),
        )
        .await
        .map_err(|e| BenchHallucinationDeltaError::SsbWrite {
            source_msg: e.to_string(),
        })?;

        let stream = ssb
            .query_session_stream_asc(BENCH_SESSION_ID)
            .map_err(|e| BenchHallucinationDeltaError::SsbStreamQuery {
                source_msg: e.to_string(),
            })?;

        let artifact = build_rra(BENCH_SESSION_ID, &stream, &pins);
        write_rra(&artifact, rra_path).map_err(|e| BenchHallucinationDeltaError::RraWrite {
            path: rra_path.display().to_string(),
            source_msg: e.to_string(),
        })?;

        tracing::info!(
            hallucination_delta = report.hallucination_delta,
            rra_path = %rra_path.display(),
            root_blake3 = %artifact.root_blake3,
            "hallucination-delta: PS-3 HallucinationDeltaMeasured SSB row + .rra sealed",
        );

        Ok::<(), BenchHallucinationDeltaError>(())
    })
}

// ---------------------------------------------------------------------------
// Unit tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn run_inner_fails_on_nonexistent_suite() {
        let err = run_inner(
            vec!["susvibes".to_string()],
            42,
            false,
            None,
            None,
        )
        .unwrap_err();
        assert!(
            matches!(err, BenchHallucinationDeltaError::RunpodPending(_)),
            "non-cgg Tier-1 suite must return RunpodPending, got {err}"
        );
    }

    #[test]
    fn run_inner_fails_on_endpoint() {
        let err = run_inner(
            vec!["cgg".to_string()],
            42,
            false,
            Some("http://127.0.0.1:8181".to_string()),
            None,
        )
        .unwrap_err();
        assert!(
            matches!(err, BenchHallucinationDeltaError::LiveTierDeferred),
            "--endpoint must return LiveTierDeferred in fixture tier, got {err}"
        );
    }

    #[test]
    fn write_json_report_creates_file_under_benchmarks_hallucination_delta() {
        let report = run_hallucination_delta(7).expect("fixture");
        let dir = tempfile::TempDir::new().expect("tempdir");
        let path = write_json_report(dir.path(), &report).expect("write_json_report");

        assert!(path.exists(), "report file must exist on disk");
        let on_disk = path.to_string_lossy();
        assert!(
            on_disk.contains("benchmarks") && on_disk.contains("hallucination_delta"),
            "report must live under benchmarks/hallucination_delta/, got {on_disk}"
        );

        let text = fs::read_to_string(&path).expect("read report");
        let parsed: HallucinationDeltaReport =
            serde_json::from_str(&text).expect("round-trips as HallucinationDeltaReport");
        assert!(
            parsed.hallucination_delta > 0.0,
            "hallucination_delta must be non-zero in written report"
        );
        assert!(parsed.runpod_pending, "runpod_pending must be true");
    }

    #[test]
    fn payload_fields_mirror_report() {
        let report = run_hallucination_delta(99).expect("fixture");
        let payload = HallucinationDeltaMeasuredPayload {
            suite: report.suite.clone(),
            model_id: report.model_id.clone(),
            hallucination_delta: report.hallucination_delta,
            ci_low: report.ci_low,
            ci_high: report.ci_high,
            n_tasks: report.n_samples,
            seed: report.seed,
            corpus_blake3: report.corpus_blake3.clone(),
            gate_recall: report.gate_recall,
            stdout_hash: None,
            runpod_pending: report.runpod_pending,
        };
        assert_eq!(payload.hallucination_delta, report.hallucination_delta);
        assert_eq!(payload.seed, report.seed);
        assert_eq!(payload.corpus_blake3, report.corpus_blake3);
        assert!(payload.runpod_pending, "Tier-1 runpod_pending must be true");
    }
}
