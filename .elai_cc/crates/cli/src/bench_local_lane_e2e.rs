//! PS-READINESS-E2ERUNNER Slice B — `elai bench local-lane-e2e` CLI driver.
//!
//! The ONLY live consumer of the 4B sidecar at http://127.0.0.1:8181. Drives
//! a REAL model: health-checks the sidecar, asks the 4B to generate a small
//! Rust function, splices it into a fixed test harness, compiles + runs it,
//! and emits ONE signed off-agent artifact carrying the REAL compile exit
//! code + test exit code + a lane tag (model + host).
//!
//! # Wiring discipline
//!
//! producer  : `run_local_lane_e2e` (orchestrator benchmarks).
//! consumer  : this file — `elai bench local-lane-e2e` is the REAL
//!             entrypoint. Writes a signed `.rra` manifest via G-2
//!             `build_rra`/`write_rra` (blake3-only root, SC-1), and emits
//!             EXACTLY ONE `LocalLaneE2eMeasured` SSB beacon.
//! ssb row   : `SsbActionType::LocalLaneE2eMeasured` (one per run).
//! rra       : `benchmarks/local_lane_e2e/local-lane-e2e-<unix_ms>.rra`.
//!
//! # No fixture fallback (R1)
//!
//! If `run_local_lane_e2e` returns `Err(SidecarUnavailable)` the command
//! exits nonzero with a clear message — it does NOT emit a green beacon.
//! A missing/unreachable sidecar is a hard FAIL, never a synthesized pass.
//!
//! # No `.unwrap()` / `.expect()` in non-test paths.
//! # blake3 only for all content hashing (anti-stale rule).

use std::fs;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

use elai_core::OsClock;
use elai_core::paths::sessions_dir_path;
use elai_orchestrator::benchmarks::local_lane_e2e_runner::{
    LocalLaneE2eError, LocalLaneE2eReport, run_local_lane_e2e,
};
use elai_orchestrator::determinism::DeterminismConfig;
use elai_orchestrator::rra::{build_rra, write_rra};
use elai_orchestrator::ssb::{LocalLaneE2eMeasuredPayload, SsbReader as _, SCHEMA_VERSION_PUBLIC};
use elai_orchestrator::{Ssb, SsbConfig, SsbPayload};
use thiserror::Error;

// ---------------------------------------------------------------------------
// Constants
// ---------------------------------------------------------------------------

/// Bench session id embedded in every SSB row this bench emits.
const BENCH_SESSION_ID: &str = "cli-bench-local-lane-e2e";

/// Agent id label for SSB rows (mirrors bench_safedelta pattern).
const BENCH_AGENT_ID: &str = "cli:bench-local-lane-e2e";

/// Default sidecar endpoint: the 4B lane at http://127.0.0.1:8181.
/// DEFAULT_SIDECAR_URL (http://localhost:8100) is DEAD — we pass :8181
/// explicitly.
const DEFAULT_ENDPOINT: &str = "http://127.0.0.1:8181";

/// Default model id for the 4B sidecar lane.
const DEFAULT_MODEL: &str = "qwen3.5-4b-q4_k_m";

// ---------------------------------------------------------------------------
// Error type
// ---------------------------------------------------------------------------

#[derive(Debug, Error)]
pub enum BenchLocalLaneE2eError {
    /// The local-lane e2e runner failed.
    #[error("local-lane-e2e runner error: {0}")]
    Runner(#[from] LocalLaneE2eError),
    /// Could not determine the current working directory for the report dir.
    #[error("local-lane-e2e could not determine cwd: {source_msg}")]
    Cwd { source_msg: String },
    /// Could not create the on-disk report or .rra directory.
    #[error("local-lane-e2e could not create dir {path}: {source_msg}")]
    ReportDir { path: String, source_msg: String },
    /// Generic I/O failure writing the .rra file.
    #[error("local-lane-e2e could not write {path}: {source_msg}")]
    OutputIo { path: String, source_msg: String },
    /// JSON serialisation failure.
    #[error("local-lane-e2e could not serialise report: {source_msg}")]
    Serialise { source_msg: String },
    /// Building the async runtime failed.
    #[error("local-lane-e2e could not build async runtime: {source_msg}")]
    Runtime { source_msg: String },
    /// Opening the SSB single-writer failed.
    #[error("local-lane-e2e could not open SSB writer: {source_msg}")]
    SsbOpen { source_msg: String },
    /// Writing the LocalLaneE2eMeasured beacon failed.
    #[error("local-lane-e2e could not write SSB beacon: {source_msg}")]
    SsbWrite { source_msg: String },
    /// Resolving determinism pins (seed / hardware class / fingerprint) failed.
    #[error("local-lane-e2e could not resolve determinism pins: {source_msg}")]
    DeterminismPins { source_msg: String },
    /// Querying the SSB session stream (for the .rra) failed.
    #[error("local-lane-e2e could not query SSB session stream: {source_msg}")]
    SsbStreamQuery { source_msg: String },
    /// Writing the .rra manifest failed.
    #[error("local-lane-e2e could not write .rra manifest to {path}: {source_msg}")]
    RraWrite { path: String, source_msg: String },
}

// ---------------------------------------------------------------------------
// CLI entry
// ---------------------------------------------------------------------------

/// CLI entry — runs the local-lane e2e benchmark against the 4B sidecar,
/// emits one `LocalLaneE2eMeasured` SSB beacon, seals a signed `.rra`
/// manifest, and prints a summary (or the full payload JSON with `--json`).
///
/// Returns `Ok(())` on a clean run (rc=0). On a handled error it prints a
/// `LOCAL-LANE-E2E ABORT:` line to stderr and calls `std::process::exit(1)`.
/// `SidecarUnavailable` is a nonzero exit with NO beacon (R1: no live
/// sidecar = FAIL, not a synthesized pass).
pub(crate) fn run(
    json: bool,
    endpoint: Option<String>,
    model: Option<String>,
) -> anyhow::Result<()> {
    match run_inner(json, endpoint, model) {
        Ok(()) => Ok(()),
        Err(e) => {
            eprintln!("LOCAL-LANE-E2E ABORT: {e}");
            std::process::exit(1);
        }
    }
}

/// Core — runs the live e2e benchmark, emits the SSB beacon + seals the
/// `.rra` manifest. Public so an integration test can drive it without the
/// process exit.
pub fn run_inner(
    json: bool,
    endpoint: Option<String>,
    model: Option<String>,
) -> Result<(), BenchLocalLaneE2eError> {
    let endpoint = endpoint.unwrap_or_else(|| DEFAULT_ENDPOINT.to_string());
    let model = model.unwrap_or_else(|| DEFAULT_MODEL.to_string());

    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .map_err(|e| BenchLocalLaneE2eError::Runtime {
            source_msg: e.to_string(),
        })?;

    let report = rt.block_on(async move {
        run_local_lane_e2e(&endpoint, &model).await
    })?;

    // Build the payload from the report.
    let payload = LocalLaneE2eMeasuredPayload {
        model: report.model.clone(),
        host: report.host.clone(),
        health_check_passed: report.health_check_passed,
        complete_latency_ms: report.complete_latency_ms,
        generated_chars: report.generated_chars as u64,
        compile_exit_code: report.compile_exit_code,
        test_exit_code: report.test_exit_code,
        functional_pass: report.functional_pass,
        schema_version: SCHEMA_VERSION_PUBLIC as u32,
    };

    // Emit the SSB beacon + seal the .rra manifest.
    let project_root = std::env::current_dir().map_err(|e| {
        BenchLocalLaneE2eError::Cwd {
            source_msg: e.to_string(),
        }
    })?;
    let rra_path = rra_path(&project_root);
    emit_beacon_and_rra(&project_root, &rra_path, &payload)?;

    // Summary line.
    println!(
        "LOCAL-LANE-E2E model={} host={} health={} latency_ms={} chars={} \
         compile_rc={} test_rc={} functional_pass={}",
        report.model,
        report.host,
        report.health_check_passed,
        report.complete_latency_ms,
        report.generated_chars,
        report.compile_exit_code,
        report.test_exit_code,
        report.functional_pass,
    );
    if json {
        let body = serde_json::to_string_pretty(&payload).map_err(|e| {
            BenchLocalLaneE2eError::Serialise {
                source_msg: e.to_string(),
            }
        })?;
        println!("{body}");
    }
    Ok(())
}

// ---------------------------------------------------------------------------
// .rra path helper
// ---------------------------------------------------------------------------

/// Build the .rra manifest path under
/// `<project_root>/benchmarks/local_lane_e2e/local-lane-e2e-<unix_ms>.rra`.
fn rra_path(project_root: &std::path::Path) -> PathBuf {
    let dir = project_root.join("benchmarks").join("local_lane_e2e");
    let unix_ms = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis())
        .unwrap_or_default();
    dir.join(format!("local-lane-e2e-{unix_ms}.rra"))
}

// ---------------------------------------------------------------------------
// SSB emit + .rra seal
// ---------------------------------------------------------------------------

/// Open the canonical session `ssb.db`, emit ONE `LocalLaneE2eMeasured`
/// beacon (durable, awaited), then query the session stream and seal a
/// signed `.rra` manifest via G-2 `build_rra`/`write_rra` (blake3-only root,
/// SC-1). Mirrors bench_hallucination_delta.rs::emit_beacon_and_rra.
fn emit_beacon_and_rra(
    project_root: &std::path::Path,
    rra_path: &std::path::Path,
    payload: &LocalLaneE2eMeasuredPayload,
) -> Result<(), BenchLocalLaneE2eError> {
    let dir = rra_path.parent().ok_or_else(|| {
        BenchLocalLaneE2eError::ReportDir {
            path: rra_path.display().to_string(),
            source_msg: "rra_path has no parent".to_string(),
        }
    })?;
    fs::create_dir_all(dir).map_err(|e| BenchLocalLaneE2eError::ReportDir {
        path: dir.display().to_string(),
        source_msg: e.to_string(),
    })?;

    let db_path = sessions_dir_path(project_root).join("ssb.db");
    if let Some(parent) = db_path.parent() {
        fs::create_dir_all(parent).map_err(|e| BenchLocalLaneE2eError::ReportDir {
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
        .map_err(|e| BenchLocalLaneE2eError::Runtime {
            source_msg: e.to_string(),
        })?;

    let pins = DeterminismConfig::from_env().map_err(|e| {
        BenchLocalLaneE2eError::DeterminismPins {
            source_msg: e.to_string(),
        }
    })?;

    let payload = payload.clone();
    rt.block_on(async move {
        let ssb = Ssb::new(cfg, clock).map_err(|e| BenchLocalLaneE2eError::SsbOpen {
            source_msg: e.to_string(),
        })?;

        ssb.write_payload(
            BENCH_AGENT_ID,
            0,
            SsbPayload::LocalLaneE2eMeasured(payload),
            Vec::new(),
        )
        .await
        .map_err(|e| BenchLocalLaneE2eError::SsbWrite {
            source_msg: e.to_string(),
        })?;

        let stream = ssb
            .query_session_stream_asc(BENCH_SESSION_ID)
            .map_err(|e| BenchLocalLaneE2eError::SsbStreamQuery {
                source_msg: e.to_string(),
            })?;

        let artifact = build_rra(BENCH_SESSION_ID, &stream, &pins);
        write_rra(&artifact, rra_path).map_err(|e| BenchLocalLaneE2eError::RraWrite {
            path: rra_path.display().to_string(),
            source_msg: e.to_string(),
        })?;

        tracing::info!(
            rra_path = %rra_path.display(),
            root_blake3 = %artifact.root_blake3,
            "local-lane-e2e: LocalLaneE2eMeasured SSB row + .rra sealed",
        );

        Ok::<(), BenchLocalLaneE2eError>(())
    })
}
