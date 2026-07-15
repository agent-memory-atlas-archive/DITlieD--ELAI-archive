//! K-9 model-refresh cadence benchmark runner.
//!
//! Pure + deterministic composition over J-9 [`HardwareModelCatalog`]. Builds
//! per-lane keep / re_evaluate decision rows against a fixed probe fixture.
//! Does NOT write SSB (the CLI bench seam owns write_payload + the RRA seal).
//! Live W-10 pass-rate remains deferred via [`live_eval_pass_rate`].

use serde::{Deserialize, Serialize};

use crate::hardware_catalog::{CatalogLaneKind, HardwareModelCatalog, ModelCatalogEntry};
use crate::system_probe::{classify_vram_tier, DetectionMethod, GpuInfo, SystemProbeResult};

/// Incumbent model_id + recorded fit_score baseline per lane (last cadence snapshot).
const INCUMBENTS: &[(CatalogLaneKind, &str, u16)] = &[
    (CatalogLaneKind::Coding, "Qwen2.5-Coder-7B-Instruct", 850),
    (CatalogLaneKind::Vision, "Qwen2.5-VL-7B", 800),
    (CatalogLaneKind::Ocr, "PaddleOCR-VL", 750),
    (
        CatalogLaneKind::EmbeddingReranker,
        "BGE-Reranker",
        780,
    ),
];

/// Fixed 24 GB probe so catalog + verdicts are byte-reproducible (no live prober).
const FIXTURE_VRAM_MB: u32 = 24_576;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ModelRefreshRunnerStatus {
    Measured,
    RunnerPresent,
}

/// One decision row per lane. Fields map 1:1 onto ModelRefreshCadencePayload.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ModelRefreshDecision {
    pub lane: String,
    pub candidate_model: String,
    pub incumbent_model: String,
    /// "adopt" | "keep" | "re_evaluate" (this leg only emits keep / re_evaluate).
    pub verdict: String,
    pub pass_rate: Option<f64>,
    pub vram_mb: Option<u32>,
    pub tok_per_s: Option<f64>,
    pub dollars_per_verified_task: Option<f64>,
    pub fixture_run: bool,
    pub notes: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ModelRefreshReport {
    pub runner_status: ModelRefreshRunnerStatus,
    pub seed: u64,
    pub decisions: Vec<ModelRefreshDecision>,
}

#[derive(Debug)]
pub enum ModelRefreshError {
    EmptyCatalog,
}

impl std::fmt::Display for ModelRefreshError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::EmptyCatalog => write!(f, "model-refresh catalog produced zero lanes"),
        }
    }
}

impl std::error::Error for ModelRefreshError {}

/// Production runner called by the `elai bench model-refresh` CLI handler (slice C).
pub fn run_model_refresh(
    lane: Option<CatalogLaneKind>,
    seed: u64,
) -> Result<ModelRefreshReport, ModelRefreshError> {
    let probe = fixture_probe();
    let catalog = HardwareModelCatalog::build(&probe);
    if catalog.lanes.is_empty() {
        return Err(ModelRefreshError::EmptyCatalog);
    }

    let selected: Vec<_> = catalog
        .lanes
        .iter()
        .filter(|l| lane.map(|want| l.lane == want).unwrap_or(true))
        .collect();

    let mut decisions = Vec::with_capacity(selected.len());
    for catalog_lane in selected {
        let candidate = catalog_lane
            .entries
            .iter()
            .find(|entry| entry.eligible)
            .or_else(|| catalog_lane.entries.first());

        let Some(candidate) = candidate else {
            decisions.push(ModelRefreshDecision {
                lane: catalog_lane.lane.as_str().to_owned(),
                candidate_model: String::new(),
                incumbent_model: incumbent_for(catalog_lane.lane)
                    .map(|(_, id, _)| (*id).to_owned())
                    .unwrap_or_default(),
                verdict: "keep".to_owned(),
                pass_rate: None,
                vram_mb: None,
                tok_per_s: None,
                dollars_per_verified_task: None,
                fixture_run: false,
                notes: "no catalog entries for lane".to_owned(),
            });
            continue;
        };

        let (incumbent_model, baseline) = incumbent_for(catalog_lane.lane)
            .map(|(_, id, score)| ((*id).to_owned(), *score))
            .unwrap_or_else(|| (String::new(), 0));

        // Safe-arm: better candidate only flags re_evaluate; adoption needs W-10 + user ratification.
        let verdict = if candidate.fit_score > baseline {
            "re_evaluate"
        } else {
            "keep"
        };

        let vram_mb = top_quant_vram_mb(candidate);
        let pass_rate = live_eval_pass_rate(candidate);

        decisions.push(ModelRefreshDecision {
            lane: catalog_lane.lane.as_str().to_owned(),
            candidate_model: candidate.model_id.clone(),
            incumbent_model,
            verdict: verdict.to_owned(),
            pass_rate,
            vram_mb,
            tok_per_s: None,
            dollars_per_verified_task: None,
            fixture_run: false,
            notes: format!(
                "deterministic catalog score; candidate_fit={} baseline={}",
                candidate.fit_score, baseline
            ),
        });
    }

    Ok(ModelRefreshReport {
        runner_status: ModelRefreshRunnerStatus::RunnerPresent,
        seed,
        decisions,
    })
}

fn fixture_probe() -> SystemProbeResult {
    SystemProbeResult {
        gpu: Some(GpuInfo {
            name: "fixture-gpu-24gb".to_owned(),
            vram_total_mb: FIXTURE_VRAM_MB,
            vram_free_mb: FIXTURE_VRAM_MB,
            tier: classify_vram_tier(FIXTURE_VRAM_MB),
            driver_version: Some("fixture".to_owned()),
            uuid: None,
        }),
        ram_total_mb: 65_536,
        ram_free_mb: 49_152,
        disk_total_mb: 1_000_000,
        disk_free_mb: 750_000,
        detection_method: DetectionMethod::EnvFallback,
        probed_at_ms: 0,
    }
}

fn incumbent_for(lane: CatalogLaneKind) -> Option<&'static (CatalogLaneKind, &'static str, u16)> {
    INCUMBENTS.iter().find(|(kind, _, _)| *kind == lane)
}

fn top_quant_vram_mb(entry: &ModelCatalogEntry) -> Option<u32> {
    entry.fits.first().map(|fit| fit.min_vram_mb)
}

// WIRE-DARK[K-9-W10]
/// RunPod / W-10 live eval; returns None until that leg is funded.
fn live_eval_pass_rate(_entry: &ModelCatalogEntry) -> Option<f64> {
    None
}
