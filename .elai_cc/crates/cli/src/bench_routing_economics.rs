//! K-2 `elai bench routing-economics` production handler.
//!
//! Drives the orchestrator runner, writes the deterministic fixture report,
//! emits one `RoutingEconomicsMeasured` SSB row, and seals the session stream
//! into an RRA. The live lane remains honestly `RunnerPresent` until funded.

use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

use elai_core::paths::sessions_dir_path;
use elai_core::OsClock;
use elai_orchestrator::benchmarks::routing_economics_runner::{
    run_routing_economics, RoutingEconomicsError, RoutingEconomicsReport, RunnerStatus,
};
use elai_orchestrator::determinism::DeterminismConfig;
use elai_orchestrator::rra::{build_rra, write_rra};
use elai_orchestrator::ssb::{RoutingEconomicsMeasuredPayload, SsbReader as _};
use elai_orchestrator::{Ssb, SsbConfig, SsbPayload};
use thiserror::Error;

const BENCH_SESSION_ID: &str = "cli-bench-routing-economics";
const BENCH_AGENT_ID: &str = "cli:bench-routing-economics";

#[derive(Debug, Error)]
enum BenchRoutingEconomicsError {
    #[error("routing-economics runner failed: {0}")]
    Runner(#[from] RoutingEconomicsError),
    #[error("routing-economics could not determine cwd: {source_msg}")]
    Cwd { source_msg: String },
    #[error("routing-economics could not create {path}: {source_msg}")]
    ReportDir { path: String, source_msg: String },
    #[error("routing-economics could not serialize report: {source_msg}")]
    Serialize { source_msg: String },
    #[error("routing-economics could not write {path}: {source_msg}")]
    OutputIo { path: String, source_msg: String },
    #[error("routing-economics could not build async runtime: {source_msg}")]
    Runtime { source_msg: String },
    #[error("routing-economics could not open SSB: {source_msg}")]
    SsbOpen { source_msg: String },
    #[error("routing-economics could not write SSB beacon: {source_msg}")]
    SsbWrite { source_msg: String },
    #[error("routing-economics could not query SSB stream: {source_msg}")]
    SsbStreamQuery { source_msg: String },
    #[error("routing-economics determinism pins failed: {source_msg}")]
    DeterminismPins { source_msg: String },
    #[error("routing-economics could not write RRA {path}: {source_msg}")]
    RraWrite { path: String, source_msg: String },
}

pub(crate) fn run(
    seed: u64,
    json: bool,
    force_cloud_everything: bool,
) -> anyhow::Result<()> {
    run_inner(seed, json, force_cloud_everything).map_err(anyhow::Error::new)
}

fn run_inner(
    seed: u64,
    json: bool,
    force_cloud_everything: bool,
) -> Result<(), BenchRoutingEconomicsError> {
    let report = run_routing_economics(seed, force_cloud_everything)?;
    let project_root = std::env::current_dir().map_err(|e| BenchRoutingEconomicsError::Cwd {
        source_msg: e.to_string(),
    })?;
    let report_path = write_json_report(&project_root, &report)?;
    let rra_path = report_path.with_extension("rra");
    emit_beacon_and_rra(&project_root, &rra_path, &report)?;

    println!(
        "ROUTING-ECONOMICS runner_status={:?} selected_arm={} hybrid_offloaded={:.2}% \
         force_cloud_offloaded={:.2}% hybrid_cost_per_verified_usd={:?} \
         force_cloud_cost_per_verified_usd={:?} live_cost={} report={} rra={}",
        report.runner_status,
        report.selected_arm,
        report.hybrid.percent_offloaded,
        report.force_cloud_everything.percent_offloaded,
        report.hybrid.cost_per_verified_task_usd,
        report.force_cloud_everything.cost_per_verified_task_usd,
        report.live_cost,
        report_path.display(),
        rra_path.display(),
    );

    if json {
        let body = serde_json::to_string_pretty(&report).map_err(|e| {
            BenchRoutingEconomicsError::Serialize {
                source_msg: e.to_string(),
            }
        })?;
        println!("{body}");
    }
    Ok(())
}

fn write_json_report(
    project_root: &Path,
    report: &RoutingEconomicsReport,
) -> Result<PathBuf, BenchRoutingEconomicsError> {
    let report_dir = project_root.join("benchmarks").join("routing_economics");
    fs::create_dir_all(&report_dir).map_err(|e| BenchRoutingEconomicsError::ReportDir {
        path: report_dir.display().to_string(),
        source_msg: e.to_string(),
    })?;
    let unix_ms = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_millis())
        .unwrap_or_default();
    let report_path = report_dir.join(format!("routing-economics-{unix_ms}.json"));
    let body = serde_json::to_string_pretty(report).map_err(|e| {
        BenchRoutingEconomicsError::Serialize {
            source_msg: e.to_string(),
        }
    })?;
    fs::write(&report_path, body).map_err(|e| BenchRoutingEconomicsError::OutputIo {
        path: report_path.display().to_string(),
        source_msg: e.to_string(),
    })?;
    Ok(report_path)
}

fn status_str(status: RunnerStatus) -> String {
    match status {
        RunnerStatus::Measured => "measured".to_string(),
        RunnerStatus::RunnerPresent => "runner_present".to_string(),
    }
}

fn build_payload(
    report: &RoutingEconomicsReport,
) -> Result<RoutingEconomicsMeasuredPayload, BenchRoutingEconomicsError> {
    let report_bytes = serde_json::to_vec(report).map_err(|e| {
        BenchRoutingEconomicsError::Serialize {
            source_msg: e.to_string(),
        }
    })?;
    Ok(RoutingEconomicsMeasuredPayload {
        seed: report.seed,
        runner_status: status_str(report.runner_status),
        fixture_status: status_str(report.fixture_status),
        selected_arm: report.selected_arm.clone(),
        suite: report.suite.clone(),
        fixture_blake3: report.fixture_blake3.clone(),
        report_blake3: blake3::hash(&report_bytes).to_hex().to_string(),
        hybrid_n_tasks: report.hybrid.n_tasks,
        hybrid_n_offloaded: report.hybrid.n_offloaded,
        hybrid_percent_offloaded: report.hybrid.percent_offloaded,
        hybrid_n_verified: report.hybrid.n_verified,
        hybrid_total_cost_micros: report.hybrid.total_cost_micros,
        hybrid_cost_per_verified_task_micros: report.hybrid.cost_per_verified_task_micros,
        force_cloud_n_tasks: report.force_cloud_everything.n_tasks,
        force_cloud_n_offloaded: report.force_cloud_everything.n_offloaded,
        force_cloud_percent_offloaded: report.force_cloud_everything.percent_offloaded,
        force_cloud_n_verified: report.force_cloud_everything.n_verified,
        force_cloud_total_cost_micros: report.force_cloud_everything.total_cost_micros,
        force_cloud_cost_per_verified_task_micros: report
            .force_cloud_everything
            .cost_per_verified_task_micros,
        live_cost: report.live_cost.clone(),
        live_lane_funded: report.live_lane_funded,
    })
}

fn emit_beacon_and_rra(
    project_root: &Path,
    rra_path: &Path,
    report: &RoutingEconomicsReport,
) -> Result<(), BenchRoutingEconomicsError> {
    let db_path = sessions_dir_path(project_root).join("ssb.db");
    if let Some(parent) = db_path.parent() {
        fs::create_dir_all(parent).map_err(|e| BenchRoutingEconomicsError::ReportDir {
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
    let pins = DeterminismConfig::from_env().map_err(|e| {
        BenchRoutingEconomicsError::DeterminismPins {
            source_msg: e.to_string(),
        }
    })?;
    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .map_err(|e| BenchRoutingEconomicsError::Runtime {
            source_msg: e.to_string(),
        })?;
    let payload = build_payload(report)?;

    runtime.block_on(async move {
        let ssb = Ssb::new(cfg, clock).map_err(|e| BenchRoutingEconomicsError::SsbOpen {
            source_msg: e.to_string(),
        })?;
        ssb.write_payload(
            BENCH_AGENT_ID,
            0,
            SsbPayload::RoutingEconomicsMeasured(payload),
            Vec::new(),
        )
        .await
        .map_err(|e| BenchRoutingEconomicsError::SsbWrite {
            source_msg: e.to_string(),
        })?;
        let stream = ssb
            .query_session_stream_asc(BENCH_SESSION_ID)
            .map_err(|e| BenchRoutingEconomicsError::SsbStreamQuery {
                source_msg: e.to_string(),
            })?;
        let artifact = build_rra(BENCH_SESSION_ID, &stream, &pins);
        write_rra(&artifact, rra_path).map_err(|e| BenchRoutingEconomicsError::RraWrite {
            path: rra_path.display().to_string(),
            source_msg: e.to_string(),
        })?;
        Ok::<(), BenchRoutingEconomicsError>(())
    })
}
