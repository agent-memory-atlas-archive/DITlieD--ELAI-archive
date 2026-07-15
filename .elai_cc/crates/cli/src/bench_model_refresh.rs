//! K-9 `elai bench model-refresh` production handler.
//!
//! Drives the orchestrator runner, writes the deterministic fixture report,
//! emits one `ModelRefreshCadence` SSB row per decision, and seals the session
//! stream into an RRA.

use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

use elai_core::paths::sessions_dir_path;
use elai_core::OsClock;
use elai_orchestrator::benchmarks::model_refresh_runner::{
    run_model_refresh, ModelRefreshDecision, ModelRefreshError, ModelRefreshReport,
};
use elai_orchestrator::determinism::DeterminismConfig;
use elai_orchestrator::hardware_catalog::CatalogLaneKind;
use elai_orchestrator::rra::{build_rra, write_rra};
use elai_orchestrator::ssb::{
    ModelRefreshCadenceKind, ModelRefreshCadencePayload, SsbReader as _,
};
use elai_orchestrator::{Ssb, SsbConfig, SsbPayload};
use thiserror::Error;

const BENCH_SESSION_ID: &str = "cli-bench-model-refresh";
const BENCH_AGENT_ID: &str = "cli:bench-model-refresh";

#[derive(Debug, Error)]
enum BenchModelRefreshError {
    #[error("model-refresh runner failed: {0}")]
    RunnerFailed(String),
    #[error("model-refresh unknown --lane '{lane}' (expected coding|vision|ocr|embedding_reranker)")]
    InvalidLane { lane: String },
    #[error("model-refresh could not determine cwd: {source_msg}")]
    Cwd { source_msg: String },
    #[error("model-refresh could not create {path}: {source_msg}")]
    ReportDir { path: String, source_msg: String },
    #[error("model-refresh could not serialize report: {source_msg}")]
    Serialize { source_msg: String },
    #[error("model-refresh could not write {path}: {source_msg}")]
    OutputIo { path: String, source_msg: String },
    #[error("model-refresh could not build async runtime: {source_msg}")]
    Runtime { source_msg: String },
    #[error("model-refresh could not open SSB: {source_msg}")]
    SsbOpen { source_msg: String },
    #[error("model-refresh could not write SSB beacon: {source_msg}")]
    SsbWrite { source_msg: String },
    #[error("model-refresh could not query SSB stream: {source_msg}")]
    SsbStreamQuery { source_msg: String },
    #[error("model-refresh determinism pins failed: {source_msg}")]
    DeterminismPins { source_msg: String },
    #[error("model-refresh could not write RRA {path}: {source_msg}")]
    RraWrite { path: String, source_msg: String },
}

impl From<ModelRefreshError> for BenchModelRefreshError {
    fn from(err: ModelRefreshError) -> Self {
        Self::RunnerFailed(err.to_string())
    }
}

pub(crate) fn run(lane: Option<String>, seed: u64, json: bool) -> anyhow::Result<()> {
    run_inner(lane, seed, json).map_err(anyhow::Error::new)
}

fn run_inner(
    lane: Option<String>,
    seed: u64,
    json: bool,
) -> Result<(), BenchModelRefreshError> {
    let lane_kind = map_lane(lane)?;
    let report = run_model_refresh(lane_kind, seed)?;
    let project_root = std::env::current_dir().map_err(|e| BenchModelRefreshError::Cwd {
        source_msg: e.to_string(),
    })?;
    let report_path = write_json_report(&project_root, &report)?;
    let rra_path = report_path.with_extension("rra");
    emit_beacon_and_rra(&project_root, &rra_path, &report)?;

    if json {
        let body = serde_json::to_string_pretty(&report).map_err(|e| {
            BenchModelRefreshError::Serialize {
                source_msg: e.to_string(),
            }
        })?;
        println!("{body}");
    } else {
        print_human_summary(&report, &report_path, &rra_path);
    }
    Ok(())
}

fn map_lane(lane: Option<String>) -> Result<Option<CatalogLaneKind>, BenchModelRefreshError> {
    match lane {
        None => Ok(None),
        Some(raw) => match raw.as_str() {
            "coding" => Ok(Some(CatalogLaneKind::Coding)),
            "vision" => Ok(Some(CatalogLaneKind::Vision)),
            "ocr" => Ok(Some(CatalogLaneKind::Ocr)),
            "embedding_reranker" => Ok(Some(CatalogLaneKind::EmbeddingReranker)),
            other => Err(BenchModelRefreshError::InvalidLane {
                lane: other.to_string(),
            }),
        },
    }
}

fn print_human_summary(report: &ModelRefreshReport, report_path: &Path, rra_path: &Path) {
    println!(
        "MODEL-REFRESH runner_status={:?} seed={} decisions={} report={} rra={}",
        report.runner_status,
        report.seed,
        report.decisions.len(),
        report_path.display(),
        rra_path.display(),
    );
    for d in &report.decisions {
        println!(
            "  lane={} verdict={} candidate={} incumbent={} pass_rate={:?} vram_mb={:?} notes={}",
            d.lane,
            d.verdict,
            d.candidate_model,
            d.incumbent_model,
            d.pass_rate,
            d.vram_mb,
            d.notes,
        );
    }
}

fn write_json_report(
    project_root: &Path,
    report: &ModelRefreshReport,
) -> Result<PathBuf, BenchModelRefreshError> {
    let report_dir = project_root.join("benchmarks").join("model_refresh");
    fs::create_dir_all(&report_dir).map_err(|e| BenchModelRefreshError::ReportDir {
        path: report_dir.display().to_string(),
        source_msg: e.to_string(),
    })?;
    let unix_ms = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_millis())
        .unwrap_or_default();
    let report_path = report_dir.join(format!("model-refresh-{unix_ms}.json"));
    let body = serde_json::to_string_pretty(report).map_err(|e| {
        BenchModelRefreshError::Serialize {
            source_msg: e.to_string(),
        }
    })?;
    fs::write(&report_path, body).map_err(|e| BenchModelRefreshError::OutputIo {
        path: report_path.display().to_string(),
        source_msg: e.to_string(),
    })?;
    Ok(report_path)
}

fn emit_beacon_and_rra(
    project_root: &Path,
    rra_path: &Path,
    report: &ModelRefreshReport,
) -> Result<(), BenchModelRefreshError> {
    let db_path = sessions_dir_path(project_root).join("ssb.db");
    if let Some(parent) = db_path.parent() {
        fs::create_dir_all(parent).map_err(|e| BenchModelRefreshError::ReportDir {
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
        BenchModelRefreshError::DeterminismPins {
            source_msg: e.to_string(),
        }
    })?;
    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .map_err(|e| BenchModelRefreshError::Runtime {
            source_msg: e.to_string(),
        })?;

    runtime.block_on(async move {
        let ssb = Ssb::new(cfg, clock).map_err(|e| BenchModelRefreshError::SsbOpen {
            source_msg: e.to_string(),
        })?;
        for d in &report.decisions {
            let payload = build_payload(d);
            ssb.write_payload(
                BENCH_AGENT_ID,
                0,
                SsbPayload::ModelRefreshCadence(payload),
                Vec::new(),
            )
            .await
            .map_err(|e| BenchModelRefreshError::SsbWrite {
                source_msg: e.to_string(),
            })?;
        }
        let stream = ssb
            .query_session_stream_asc(BENCH_SESSION_ID)
            .map_err(|e| BenchModelRefreshError::SsbStreamQuery {
                source_msg: e.to_string(),
            })?;
        let artifact = build_rra(BENCH_SESSION_ID, &stream, &pins);
        write_rra(&artifact, rra_path).map_err(|e| BenchModelRefreshError::RraWrite {
            path: rra_path.display().to_string(),
            source_msg: e.to_string(),
        })?;
        Ok::<(), BenchModelRefreshError>(())
    })
}

fn build_payload(d: &ModelRefreshDecision) -> ModelRefreshCadencePayload {
    ModelRefreshCadencePayload {
        kind: if d.verdict == "keep" {
            ModelRefreshCadenceKind::IncumbentKept
        } else {
            // CandidateEvaluated is the W-10 live leg; this runner seals DecisionRecorded
            ModelRefreshCadenceKind::DecisionRecorded
        },
        lane: d.lane.clone(),
        candidate_model: d.candidate_model.clone(),
        incumbent_model: d.incumbent_model.clone(),
        verdict: d.verdict.clone(),
        pass_rate: d.pass_rate,
        vram_mb: d.vram_mb,
        tok_per_s: d.tok_per_s,
        dollars_per_verified_task: d.dollars_per_verified_task,
        fixture_run: d.fixture_run,
        notes: d.notes.clone(),
    }
}
