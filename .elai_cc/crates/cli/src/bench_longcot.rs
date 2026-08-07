//! J-107 — deterministic two-arm LongCoT fixture benchmark.
//!
//! This module deliberately consumes pre-computed fixture outcomes.  It does not
//! invoke a model or an LLM judge; subjective rows are retained in the fixture
//! but are not included in the verified-outcome curve.

use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use elai_core::paths::sessions_dir_path;
use elai_memory::NoteStore;
use elai_orchestrator::ssb::{ConvergenceCurveMeasuredPayload, ConvergenceCurvePoint};
use elai_orchestrator::{Ssb, SsbConfig, SsbPayload};
use serde::{Deserialize, Serialize};
use thiserror::Error;

const FLOOR_MIN_SEED_COUNT: u32 = 5;
const BOOTSTRAP_N_RESAMPLES: usize = 500;
const KNEE_GAIN_THRESHOLD: f64 = 0.01;
const BENCH_SESSION_ID: &str = "cli-bench-longcot";
const BENCH_AGENT_ID: &str = "cli:bench-longcot";

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LongCotTaskRow {
    pub prompt: String,
    pub expected: String,
    pub checkable: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LongCotBand {
    pub label: String,
    pub budget_fraction: f32,
    pub outcomes: Vec<Vec<bool>>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LongCotConfig {
    pub suite: String,
    pub model: String,
    pub lane: String,
    pub seed_count: u32,
    pub paired: bool,
    pub seeds: u64,
    #[serde(default = "default_arms")]
    pub arms: Vec<String>,
    pub bands: Vec<LongCotBand>,
    #[serde(default)]
    pub tasks: Vec<LongCotTaskRow>,
}

fn default_arms() -> Vec<String> {
    vec!["flat".to_string(), "scaffold".to_string()]
}

#[derive(Debug, Error)]
pub enum LongCotBenchError {
    #[error("longcot statistical floor violation: seed_count={seed_count}, paired={paired}")]
    FloorViolation { seed_count: u32, paired: bool },
    #[error("longcot fixture error: {0}")]
    FixtureError(String),
    #[error("longcot beacon emission failed: {0}")]
    BeaconEmit(String),
    #[error("longcot I/O failed: {0}")]
    Io(String),
}

pub fn floor_check(config: &LongCotConfig) -> Result<(), LongCotBenchError> {
    if config.seed_count < FLOOR_MIN_SEED_COUNT || !config.paired {
        return Err(LongCotBenchError::FloorViolation {
            seed_count: config.seed_count,
            paired: config.paired,
        });
    }
    Ok(())
}

pub fn validate_fixture(config: &LongCotConfig) -> Result<(), LongCotBenchError> {
    if config.bands.len() < 2 {
        return Err(LongCotBenchError::FixtureError(
            "need at least 2 budget bands".to_string(),
        ));
    }
    for band in &config.bands {
        if band.outcomes.is_empty() {
            return Err(LongCotBenchError::FixtureError(format!(
                "band '{}' has no outcomes",
                band.label
            )));
        }
        if band.outcomes.len() != config.seed_count as usize {
            return Err(LongCotBenchError::FixtureError(format!(
                "band '{}': expected {} seeds, got {}",
                band.label,
                config.seed_count,
                band.outcomes.len()
            )));
        }
        if band.outcomes[0].is_empty() {
            return Err(LongCotBenchError::FixtureError(format!(
                "band '{}' has no tasks",
                band.label
            )));
        }
        let task_count = band.outcomes[0].len();
        if band.outcomes.iter().any(|seed| seed.len() != task_count) {
            return Err(LongCotBenchError::FixtureError(format!(
                "band '{}' has ragged outcomes",
                band.label
            )));
        }
    }
    Ok(())
}

/// Deterministic receipt checker. Empty actual output is an abstention channel,
/// not a false receipt; fixture outcomes already encode the verified result.
fn score_task(actual: &str, expected: &str) -> bool {
    let normalize = |value: &str| {
        value
            .split_whitespace()
            .collect::<Vec<_>>()
            .join(" ")
            .to_lowercase()
    };
    !actual.trim().is_empty() && normalize(actual) == normalize(expected)
}

fn mean_rate(outcomes: &[Vec<bool>]) -> f64 {
    let total = outcomes.iter().flatten().count();
    if total == 0 {
        return 0.0;
    }
    outcomes.iter().flatten().filter(|&&value| value).count() as f64 / total as f64
}

fn xorshift64(mut state: u64) -> u64 {
    state ^= state << 13;
    state ^= state >> 7;
    state ^= state << 17;
    state
}

fn bootstrap_ci(outcomes: &[Vec<bool>], seed: u64) -> (f64, f64) {
    if outcomes.is_empty() || outcomes[0].is_empty() {
        return (0.0, 0.0);
    }
    let n = outcomes[0].len();
    let mut state = if seed == 0 { 1 } else { seed };
    let mut rates = Vec::with_capacity(BOOTSTRAP_N_RESAMPLES);
    for _ in 0..BOOTSTRAP_N_RESAMPLES {
        let mut count = 0usize;
        for row in outcomes {
            for _ in 0..n {
                state = xorshift64(state);
                count += usize::from(row[(state % n as u64) as usize]);
            }
        }
        rates.push(count as f64 / (outcomes.len() * n) as f64);
    }
    rates.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    (rates[12], rates[487])
}

fn detect_knee(points: &[ConvergenceCurvePoint]) -> String {
    if points.is_empty() {
        return String::new();
    }
    for i in 1..points.len() {
        if points[i].mean_rate - points[i - 1].mean_rate < KNEE_GAIN_THRESHOLD {
            return points[i - 1].compute_level.clone();
        }
    }
    points
        .last()
        .map(|point| point.compute_level.clone())
        .unwrap_or_default()
}

fn non_monotonic(points: &[ConvergenceCurvePoint]) -> Vec<String> {
    points
        .windows(2)
        .filter_map(|pair| {
            (pair[1].mean_rate < pair[0].mean_rate).then(|| pair[1].compute_level.clone())
        })
        .collect()
}

/// The constrained-scaffold seam. Fixture execution passes `None`; live callers
/// may provide a NoteStore, in which case retrieval uses the J-108 primitive.
fn scaffold_notes(
    note_store: Option<&NoteStore>,
    entities: &[String],
    now_ms: u128,
) -> Result<usize, LongCotBenchError> {
    match note_store {
        Some(store) => elai_memory::ppr_seeded_top_k(store, entities, 3, now_ms)
            .map(|notes| notes.len())
            .map_err(|error| LongCotBenchError::FixtureError(error.to_string())),
        None => Ok(0),
    }
}

pub(crate) fn run(suite: PathBuf, seeds: u32, json: bool) -> anyhow::Result<()> {
    match run_inner(suite, seeds, json) {
        Ok(()) => Ok(()),
        Err(error) => {
            eprintln!("LONGCOT ABORT: {error}");
            std::process::exit(1);
        }
    }
}

pub fn run_inner(suite_path: PathBuf, seeds: u32, json: bool) -> Result<(), LongCotBenchError> {
    let bytes = fs::read(&suite_path).map_err(|error| LongCotBenchError::Io(error.to_string()))?;
    let mut config: LongCotConfig = toml::from_str(&String::from_utf8_lossy(&bytes))
        .map_err(|error| LongCotBenchError::FixtureError(error.to_string()))?;
    if seeds > 0 {
        config.seed_count = seeds;
    }
    floor_check(&config)?;
    validate_fixture(&config)?;
    let fixture_blake3 = blake3::hash(&bytes).to_hex().to_string();
    let now_ms = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|error| LongCotBenchError::Io(error.to_string()))?
        .as_millis() as i64;
    let mut payloads = Vec::new();
    for arm in config
        .arms
        .iter()
        .filter(|arm| *arm == "flat" || *arm == "scaffold")
    {
        let mut points = Vec::with_capacity(config.bands.len());
        for band in &config.bands {
            if arm == "scaffold" {
                let entities = vec![band.label.clone(), config.suite.clone()];
                let _ = scaffold_notes(None, &entities, now_ms as u128)?;
            }
            let rate = mean_rate(&band.outcomes);
            let (ci_low, ci_high) = bootstrap_ci(&band.outcomes, config.seeds);
            points.push(ConvergenceCurvePoint {
                compute_level: format!("{}|{}", band.label, arm),
                mean_rate: rate,
                ci_low,
                ci_high,
                cost_per_verified_task: None,
            });
        }
        let payload = ConvergenceCurveMeasuredPayload {
            fixture_suite: config.suite.clone(),
            seed_count: config.seed_count,
            task_count: config.bands[0].outcomes[0].len() as u32,
            knee_compute_level: detect_knee(&points),
            non_monotonic_at: non_monotonic(&points),
            curve_points: points,
            seed: config.seeds,
            fixture_blake3: fixture_blake3.clone(),
            runpod_pending: true,
            ts_ms: now_ms,
        };
        let project_root =
            std::env::current_dir().map_err(|error| LongCotBenchError::Io(error.to_string()))?;
        emit_beacon(&project_root, &payload)?;
        if json {
            println!(
                "{}",
                serde_json::to_string_pretty(&payload)
                    .map_err(|error| LongCotBenchError::Io(error.to_string()))?
            );
        }
        payloads.push(payload);
    }
    println!(
        "LONGCOT suite={} model={} lane={} arms={} rows={}",
        config.suite,
        config.model,
        config.lane,
        config.arms.join(","),
        payloads.len()
    );
    Ok(())
}

fn emit_beacon(
    project_root: &Path,
    payload: &ConvergenceCurveMeasuredPayload,
) -> Result<(), LongCotBenchError> {
    let db_path = sessions_dir_path(project_root).join("ssb.db");
    if let Some(parent) = db_path.parent() {
        fs::create_dir_all(parent)
            .map_err(|error| LongCotBenchError::BeaconEmit(error.to_string()))?;
    }
    let cfg = SsbConfig {
        db_path,
        pool_size: None,
        project_id: "default".to_string(),
        session_id: BENCH_SESSION_ID.to_string(),
    };
    let clock: std::sync::Arc<dyn elai_core::clock::ClockProvider> =
        std::sync::Arc::new(elai_core::OsClock);
    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .map_err(|error| LongCotBenchError::BeaconEmit(error.to_string()))?;
    let copy = payload.clone();
    runtime.block_on(async move {
        let ssb = Ssb::new(cfg, clock)
            .map_err(|error| LongCotBenchError::BeaconEmit(error.to_string()))?;
        ssb.write_payload(
            BENCH_AGENT_ID,
            0,
            SsbPayload::ConvergenceCurveMeasured(copy),
            Vec::new(),
        )
        .await
        .map_err(|error| LongCotBenchError::BeaconEmit(error.to_string()))?;
        Ok::<(), LongCotBenchError>(())
    })
}
