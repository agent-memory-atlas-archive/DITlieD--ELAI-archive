//! K-4 — `elai bench gate-tax [--seed <u64>] [--json] [--enforce-ceiling]` CLI driver.
//!
//! Surfaces the PS-2 composed gate-tax budget (KEYSTONE of K-T3) as a
//! user-facing subcommand. DRIVES the SHIPPED `gate_tax_runner` (orchestrator)
//! over the real session journal (best-effort) or a labeled fixture
//! composition on a dry journal, then writes a timestamped JSON report under
//! `<cwd>/benchmarks/gate_tax/` and emits ONE `GateTaxMeasured` SSB beacon +
//! a signed `.rra` manifest per clean run (SCHEMA_VERSION 271).
//!
//! # What this measures (honestly, GD-5)
//!
//! TWO labeled numbers, NEVER summed (rider a): `coding_task_gate_tax_ms`
//! (the ceiling-bound headline) and `audit_gate_tax_ms` (MOVE 4b, SP2: now
//! measured — the median wall-clock of 5 offline auditor checkers). Plus
//! `over_block_rate` (a benign-corpus sweep, FIX-4) and the RIDER-b
//! gate-manifest coverage table. See
//! `crates/orchestrator/src/benchmarks/gate_tax_runner.rs` for the full
//! composition doctrine and named residue (GD-5).
//!
//! # `--enforce-ceiling` (MOVE 3/6, SP2 — the actual "release gate")
//!
//! Per ATTACK-4 there is no Rust `release_gate` fn anywhere in this repo —
//! the release gate IS this exit-code contract. WITHOUT `--enforce-ceiling`
//! this CLI always exits 0 on a successful composition (Measured or
//! RunnerPresent) and 1 on a hard runner error (the SP1 happy path,
//! preserved). WITH `--enforce-ceiling`, `evaluate_ceiling_breach` (see
//! `gate_tax_runner.rs`) is consulted after the report is built (and always
//! printed either way — a breach is never a silent drop, R1/ABORT-5): a
//! `Some(reason)` verdict becomes a non-zero exit via the existing
//! `GATE-TAX ABORT:` path. Since `CODING_TASK_GATE_TAX_CEILING_MS` is
//! `CEILING-PENDING[K-4]` (no user-confirmed value yet), every
//! `--enforce-ceiling` run currently breaches by construction (fail-closed,
//! never a silent pass) until a human confirms a real ceiling.
//!
//! No `.unwrap()` / `.expect()` in non-test paths.

use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

use elai_core::paths::sessions_dir_path;
use elai_core::OsClock;
use elai_orchestrator::benchmarks::gate_tax_runner::{
    evaluate_ceiling_breach, run_gate_tax, GateCostContributor, GateTaxError, GateTaxReport,
    RunnerStatus,
};
use elai_orchestrator::determinism::DeterminismConfig;
use elai_orchestrator::rra::{build_rra, write_rra};
use elai_orchestrator::ssb::{GateTaxMeasuredPayload, SsbReader as _};
use elai_orchestrator::{Ssb, SsbConfig, SsbPayload};
use thiserror::Error;

// ---------------------------------------------------------------------------
// Constants
// ---------------------------------------------------------------------------

/// Bench session id embedded in every SSB row this bench emits.
const BENCH_SESSION_ID: &str = "cli-bench-gate-tax";

/// Agent id label for SSB rows (mirrors bench_case_reuse pattern).
const BENCH_AGENT_ID: &str = "cli:bench-gate-tax";

// ---------------------------------------------------------------------------
// Error type
// ---------------------------------------------------------------------------

#[derive(Debug, Error)]
pub enum BenchGateTaxError {
    /// The gate-tax runner failed (registry mismatch / untagged gate).
    #[error("gate-tax runner error: {0}")]
    Runner(#[from] GateTaxError),
    /// Could not determine the current working directory for the report dir.
    #[error("gate-tax could not determine cwd: {source_msg}")]
    Cwd { source_msg: String },
    /// Could not create the on-disk report directory.
    #[error("gate-tax could not create dir {path}: {source_msg}")]
    ReportDir { path: String, source_msg: String },
    /// Generic I/O failure writing the JSON report.
    #[error("gate-tax could not write {path}: {source_msg}")]
    OutputIo { path: String, source_msg: String },
    /// JSON serialisation failure.
    #[error("gate-tax could not serialise report: {source_msg}")]
    Serialise { source_msg: String },
    /// Opening the SSB single-writer failed.
    #[error("gate-tax could not open SSB writer: {source_msg}")]
    SsbOpen { source_msg: String },
    /// Writing the GateTaxMeasured beacon failed.
    #[error("gate-tax could not write SSB beacon: {source_msg}")]
    SsbWrite { source_msg: String },
    /// Resolving determinism pins (seed / hardware class / fingerprint) failed.
    #[error("gate-tax could not resolve determinism pins: {source_msg}")]
    DeterminismPins { source_msg: String },
    /// Querying the SSB session stream (for the .rra) failed.
    #[error("gate-tax could not query SSB session stream: {source_msg}")]
    SsbStreamQuery { source_msg: String },
    /// Writing the .rra manifest failed.
    #[error("gate-tax could not write .rra manifest to {path}: {source_msg}")]
    RraWrite { path: String, source_msg: String },
    /// Building the async runtime failed.
    #[error("gate-tax could not build async runtime: {source_msg}")]
    Runtime { source_msg: String },
    /// MOVE 3/6 (SP2): `--enforce-ceiling` breach (ABORT-5 fail-closed exit
    /// contract) — reused `run()`'s existing `GATE-TAX ABORT:` + exit(1) path.
    #[error("gate-tax ceiling breach: {reason}")]
    CeilingBreach { reason: String },
}

// ---------------------------------------------------------------------------
// CLI entry
// ---------------------------------------------------------------------------

/// CLI entry — runs the composed gate-tax measurement, writes a timestamped
/// JSON report, prints a summary (or the full report JSON with `--json`).
/// On a handled error (including a `--enforce-ceiling` breach) prints
/// `GATE-TAX ABORT:` to stderr and exits 1.
pub(crate) fn run(seed: u64, json: bool, enforce_ceiling: bool) -> anyhow::Result<()> {
    match run_inner(seed, json, enforce_ceiling) {
        Ok(()) => Ok(()),
        Err(e) => {
            eprintln!("GATE-TAX ABORT: {e}");
            std::process::exit(1);
        }
    }
}

/// MOVE 3/6: whether `run_inner` should return a `CeilingBreach` error for
/// this report. Pure and side-effect-free so it is directly unit-testable
/// without the report-writing / SSB-emitting I/O `run_inner` performs.
/// `enforce_ceiling == false` (SP1's default) is ALWAYS a no-op — the
/// happy path is preserved by construction, never by a special-cased branch
/// deeper in the breach logic itself.
fn ceiling_gate(report: &GateTaxReport, enforce_ceiling: bool) -> Option<String> {
    if !enforce_ceiling {
        return None;
    }
    evaluate_ceiling_breach(report)
}

/// Core — drives the runner, writes the JSON report, emits the beacon, then
/// (MOVE 3/6, SP2) applies the `--enforce-ceiling` breach gate. Public so an
/// integration test can drive it without the process exit.
pub fn run_inner(seed: u64, json: bool, enforce_ceiling: bool) -> Result<(), BenchGateTaxError> {
    let report = run_gate_tax(seed)?;

    let project_root = std::env::current_dir().map_err(|e| BenchGateTaxError::Cwd {
        source_msg: e.to_string(),
    })?;
    let report_path = write_json_report(&project_root, &report)?;

    let rra_path_buf = rra_path(
        &project_root,
        report_path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("gate-tax-unknown"),
    );
    emit_beacon_and_rra(&project_root, &rra_path_buf, &report)?;

    println!(
        "GATE-TAX runner_status={:?} coding_task_gate_tax_ms={:.4} task_class={} \
         audit_gate_tax_ms={:.4} audit_unbounded={} over_block_rate={:.4} ({}/{}) \
         coverage(coding={} trading={} audit={} dormant={} untagged={}) n_gates={} n_fires={} \
         seed={} report={} rra={}",
        report.runner_status,
        report.coding_task_gate_tax_ms,
        report.task_class_label,
        report.audit_gate_tax_ms,
        report.audit_unbounded,
        report.over_block_rate,
        report.over_block_blocked,
        report.over_block_total,
        report.n_in_scope_coding,
        report.n_trading_surface,
        report.n_offline_audit,
        report.n_dormant,
        report.n_untagged,
        report.n_gates,
        report.n_fires,
        report.seed,
        report_path.display(),
        rra_path_buf.display(),
    );

    if json {
        let body = serde_json::to_string_pretty(&report).map_err(|e| BenchGateTaxError::Serialise {
            source_msg: e.to_string(),
        })?;
        println!("{body}");
    }

    // MOVE 3/6 (SP2): the report above is ALWAYS printed regardless of the
    // verdict (R1/ABORT-5 — a breach is never a silent drop); only the exit
    // code differs.
    if let Some(reason) = ceiling_gate(&report, enforce_ceiling) {
        return Err(BenchGateTaxError::CeilingBreach { reason });
    }
    Ok(())
}

// ---------------------------------------------------------------------------
// JSON report writer
// ---------------------------------------------------------------------------

/// Write the report JSON to
/// `<project_root>/benchmarks/gate_tax/gate-tax-<unix_ms>.json` and return
/// the written path. Mirrors bench_case_reuse::write_json_report.
fn write_json_report(
    project_root: &Path,
    report: &GateTaxReport,
) -> Result<PathBuf, BenchGateTaxError> {
    let report_dir = project_root.join("benchmarks").join("gate_tax");
    fs::create_dir_all(&report_dir).map_err(|e| BenchGateTaxError::ReportDir {
        path: report_dir.display().to_string(),
        source_msg: e.to_string(),
    })?;

    let unix_ms = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis())
        .unwrap_or_default();
    let report_path = report_dir.join(format!("gate-tax-{unix_ms}.json"));

    let body = serde_json::to_string_pretty(report).map_err(|e| BenchGateTaxError::Serialise {
        source_msg: e.to_string(),
    })?;
    fs::write(&report_path, &body).map_err(|e| BenchGateTaxError::OutputIo {
        path: report_path.display().to_string(),
        source_msg: e.to_string(),
    })?;
    Ok(report_path)
}

// ---------------------------------------------------------------------------
// .rra path helper
// ---------------------------------------------------------------------------

fn rra_path(project_root: &Path, stem: &str) -> PathBuf {
    let stem_no_ext = stem.trim_end_matches(".json");
    project_root
        .join("benchmarks")
        .join("gate_tax")
        .join(format!("{stem_no_ext}.rra"))
}

// ---------------------------------------------------------------------------
// SSB emit + .rra seal
// ---------------------------------------------------------------------------

fn top_id(contributors: &[GateCostContributor], idx: usize) -> String {
    contributors.get(idx).map(|c| c.gate_id.clone()).unwrap_or_default()
}

fn top_cost(contributors: &[GateCostContributor], idx: usize) -> f64 {
    contributors.get(idx).map(|c| c.cost_ms).unwrap_or(0.0)
}

fn runner_status_str(status: RunnerStatus) -> String {
    match status {
        RunnerStatus::Measured => "measured".to_string(),
        RunnerStatus::RunnerPresent => "runner_present".to_string(),
    }
}

fn build_payload(report: &GateTaxReport, ts_ms: i64) -> GateTaxMeasuredPayload {
    GateTaxMeasuredPayload {
        seed: report.seed,
        runner_status: runner_status_str(report.runner_status),
        coding_task_gate_tax_ms: report.coding_task_gate_tax_ms,
        task_class_label: report.task_class_label.clone(),
        audit_gate_tax_ms: report.audit_gate_tax_ms,
        audit_unbounded: report.audit_unbounded,
        over_block_rate: report.over_block_rate,
        over_block_blocked: report.over_block_blocked,
        over_block_total: report.over_block_total,
        n_in_scope_coding: report.n_in_scope_coding,
        n_trading_surface: report.n_trading_surface,
        n_offline_audit: report.n_offline_audit,
        n_dormant: report.n_dormant,
        n_untagged: report.n_untagged,
        top1_gate_id: top_id(&report.top_contributors, 0),
        top1_cost_ms: top_cost(&report.top_contributors, 0),
        top2_gate_id: top_id(&report.top_contributors, 1),
        top2_cost_ms: top_cost(&report.top_contributors, 1),
        top3_gate_id: top_id(&report.top_contributors, 2),
        top3_cost_ms: top_cost(&report.top_contributors, 2),
        n_gates: report.n_gates,
        n_fires: report.n_fires,
        ts_ms,
    }
}

/// Open the canonical session `ssb.db`, emit ONE `GateTaxMeasured` beacon
/// (durable, awaited), then query the session stream and seal a signed
/// `.rra` manifest via G-2 `build_rra`/`write_rra` (blake3-only root, SC-1).
///
/// `Ssb::new` spawns the single-writer task and MUST be constructed inside
/// `rt.block_on` (bench_case_reuse.rs::emit_beacon_and_rra precedent — the
/// writer panics "no reactor running" outside the runtime).
fn emit_beacon_and_rra(
    project_root: &Path,
    rra_path: &Path,
    report: &GateTaxReport,
) -> Result<(), BenchGateTaxError> {
    let db_path = sessions_dir_path(project_root).join("ssb.db");
    if let Some(parent) = db_path.parent() {
        fs::create_dir_all(parent).map_err(|e| BenchGateTaxError::ReportDir {
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
        .map_err(|e| BenchGateTaxError::Runtime {
            source_msg: e.to_string(),
        })?;

    let ts_ms = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis() as i64)
        .unwrap_or_default();

    let payload = build_payload(report, ts_ms);

    // Resolve determinism pins outside the runtime (sync call).
    let pins = DeterminismConfig::from_env().map_err(|e| BenchGateTaxError::DeterminismPins {
        source_msg: e.to_string(),
    })?;

    rt.block_on(async move {
        let ssb = Ssb::new(cfg, clock).map_err(|e| BenchGateTaxError::SsbOpen {
            source_msg: e.to_string(),
        })?;

        ssb.write_payload(
            BENCH_AGENT_ID,
            0,
            SsbPayload::GateTaxMeasured(payload),
            Vec::new(),
        )
        .await
        .map_err(|e| BenchGateTaxError::SsbWrite {
            source_msg: e.to_string(),
        })?;

        let stream = ssb
            .query_session_stream_asc(BENCH_SESSION_ID)
            .map_err(|e| BenchGateTaxError::SsbStreamQuery {
                source_msg: e.to_string(),
            })?;

        let artifact = build_rra(BENCH_SESSION_ID, &stream, &pins);
        write_rra(&artifact, rra_path).map_err(|e| BenchGateTaxError::RraWrite {
            path: rra_path.display().to_string(),
            source_msg: e.to_string(),
        })?;

        tracing::info!(
            coding_task_gate_tax_ms = report.coding_task_gate_tax_ms,
            rra_path = %rra_path.display(),
            root_blake3 = %artifact.root_blake3,
            "gate-tax: K-4 SP1 GateTaxMeasured SSB row + .rra sealed",
        );

        Ok::<(), BenchGateTaxError>(())
    })
}

// ---------------------------------------------------------------------------
// Unit tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use elai_orchestrator::ssb::{SsbActionType, SsbReader as _};

    /// K-4 SP1 run-trigger contract: `emit_beacon_and_rra` writes EXACTLY ONE
    /// `GateTaxMeasured` row to a fresh session `ssb.db` AND a signed `.rra`
    /// manifest, and the row's decoded payload mirrors the report it was
    /// built from (R-8/D-030: the off-agent evidence a real entrypoint
    /// drove, not this test's own stdout).
    #[test]
    fn emit_beacon_and_rra_writes_row_and_manifest() {
        let report = run_gate_tax(1234).expect("run_gate_tax");
        let dir = tempfile::TempDir::new().expect("tempdir");
        let rra = rra_path(dir.path(), "gate-tax-test.json");

        emit_beacon_and_rra(dir.path(), &rra, &report).expect("emit_beacon_and_rra");

        assert!(rra.exists(), ".rra manifest must exist on disk");

        let db_path = elai_core::paths::sessions_dir_path(dir.path()).join("ssb.db");
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
            .expect("build runtime");

        let rows = rt.block_on(async move {
            let ssb = Ssb::new(cfg, clock).expect("open ssb.db");
            ssb.query_by_type(SsbActionType::GateTaxMeasured)
                .expect("query_by_type")
        });
        assert_eq!(rows.len(), 1, "exactly one GateTaxMeasured row");

        let decoded: SsbPayload =
            elai_orchestrator::ssb::decode_ssb_payload(&rows[0].payload).expect("decode payload");
        match decoded {
            SsbPayload::GateTaxMeasured(p) => {
                assert_eq!(p.seed, report.seed);
                assert_eq!(p.runner_status, runner_status_str(report.runner_status));
                assert_eq!(p.coding_task_gate_tax_ms, report.coding_task_gate_tax_ms);
                assert_eq!(p.task_class_label, report.task_class_label);
                assert_eq!(p.audit_gate_tax_ms, report.audit_gate_tax_ms);
                assert_eq!(p.audit_unbounded, report.audit_unbounded);
                assert_eq!(p.n_gates, report.n_gates);
                assert_eq!(p.n_fires, report.n_fires);
                assert_eq!(p.n_untagged, 0);
            }
            other => panic!("wrong payload variant decoded: {other:?}"),
        }
    }

    #[test]
    fn write_json_report_creates_file_under_benchmarks_gate_tax() {
        let report = run_gate_tax(1234).expect("run");
        let dir = tempfile::TempDir::new().expect("tempdir");
        let path = write_json_report(dir.path(), &report).expect("write");
        assert!(path.exists(), "report file must exist");
        let on_disk = path.to_string_lossy();
        assert!(
            on_disk.contains("benchmarks") && on_disk.contains("gate_tax"),
            "report must live under benchmarks/gate_tax/, got {on_disk}"
        );
        let text = fs::read_to_string(&path).expect("read");
        let parsed: GateTaxReport =
            serde_json::from_str(&text).expect("round-trips as GateTaxReport");
        assert_eq!(parsed.n_untagged, 0);
    }

    /// (i) MOVE 3/6, SP2: `enforce_ceiling == false` is a hard no-op — the
    /// SP1 happy path (always exit 0 on a clean composition) is preserved by
    /// construction, even though `evaluate_ceiling_breach` itself ALWAYS
    /// trips while `CODING_TASK_GATE_TAX_CEILING_MS` is CEILING-PENDING.
    /// Exercises `ceiling_gate` directly (not `run_inner`, which does real
    /// filesystem/SSB I/O against `std::env::current_dir()` — not
    /// injectable, so driving it here would pollute the real repo tree).
    #[test]
    fn ceiling_gate_is_a_noop_when_enforce_ceiling_is_false_sp1_happy_path_preserved() {
        let report = run_gate_tax(1234).expect("run_gate_tax");
        assert!(ceiling_gate(&report, false).is_none());
        assert!(
            ceiling_gate(&report, true).is_some(),
            "sanity: the pending ceiling DOES trip when enforced"
        );
    }
}
