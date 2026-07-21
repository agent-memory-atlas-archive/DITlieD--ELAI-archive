//! J-34 leg-3 -- `elai bench positional-probe --suite <PATH> [--seeds <u32>] [--json]`
//! CLI driver.
//!
//! POSITIONAL-VALUE PROBE = constraint-recall rate vs placement-depth fraction
//! curve per model. The curve carries the recall_rate (gate verdicts, never
//! LLM-judged -- TD-3) at each depth level. Tier-1 = fixture TOML only (no live
//! inference, no GPU); the fixture supplies honored/violated boolean outcomes
//! per task per seed per depth level.
//!
//! # What this measures (honestly)
//!
//! For each depth level D in the fixture suite, the probe counts the fraction
//! of constraints that were honored (recall_rate). The curve is expected to be
//! monotonically non-increasing (deeper placement = worse recall) OR to show a
//! detectable knee. A flat curve (no degradation) is flagged as "no signal;
//! investigate." Nothing is smoothed (GD-5).
//!
//! # Fixture tier (Tier-1)
//!
//! The fixture TOML supplies honored/violated booleans per task per seed per
//! depth level. No live inference runs. The live per-model inference leg is
//! PENDING (WIRE-DARK[J-34.leg3-live]); the payload shape is forward-compatible
//! (depth_fraction is an opaque f32).
//!
//! # Determinism (ELAI_SEED)
//!
//! When ELAI_SEED is set in the environment, the probe's output curve is
//! byte-identical across two runs with the same seed + fixture. The curve is
//! derived purely from the fixture TOML outcomes (no RNG in Tier-1); the seed
//! is recorded in the report for reproducibility but does not alter the Tier-1
//! curve because the fixture supplies deterministic outcomes. The seed
//! matters for the live leg (WIRE-DARK) where it controls constraint
//! placement.
//!
//! # Statistical floor (mechanically enforced)
//!
//!   - `seed_count >= 5`  (SEM meaningful at this N, I-38 floor)
//!   - `paired == true`   (same tasks across depth levels)
//!
//! # Wiring
//!
//! producer  : this file -- reads PositionalProbeConfig TOML, validates floor,
//!             sweeps depth levels, computes mean recall_rate per level,
//!             builds a PositionalValueCurve, constructs a CalibrationArtifact
//!             via `CalibrationArtifact::from_curves` (with a default
//!             FillDegradationCurve so the artifact can carry the positional
//!             curve), opens the canonical Ssb writer, calls
//!             `CalibrationArtifactStore::insert_with_ssb` (which emits the
//!             ContextCalibrationEvent(CurveMeasured) SSB row via slice-2
//!             seam), and writes a timestamped TOML report.
//! ssb row   : `SsbActionType::ContextCalibrationEvent(ContextCalibrationKind::CurveMeasured)`
//!             (Pattern-2 single-variant, 4-arm Kind rides in payload).
//! entrypoint: `elai bench positional-probe --suite <PATH> [--seeds <u32>]`.
//!
//! # No `.unwrap()` / `.expect()` in non-test paths.
//! # blake3 only for all content hashing.

use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

use elai_core::paths::sessions_dir_path;
use elai_orchestrator::context_calibration::{
    CalibrationArtifact, CalibrationArtifactStore, FillDegradationCurve, PositionalSample,
    PositionalValueCurve,
};
use elai_orchestrator::ssb::{SsbActionType, SsbReader as _};
use elai_orchestrator::{Ssb, SsbConfig};
use serde::{Deserialize, Serialize};
use thiserror::Error;

// ---------------------------------------------------------------------------
// Constants
// ---------------------------------------------------------------------------

const FLOOR_MIN_SEED_COUNT: u32 = 5;

const BENCH_SESSION_ID: &str = "cli-bench-positional-probe";

#[allow(dead_code)]
const BENCH_AGENT_ID: &str = "cli:bench-positional-probe";

/// The canonical depth fractions the probe fills (0.0 / 0.25 / 0.5 / 0.75 / 1.0).
/// A fixture MAY supply any set of depth levels, but these are the expected
/// probe points.
pub const CANONICAL_DEPTH_FRACTIONS: &[f32] = &[0.0, 0.25, 0.5, 0.75, 1.0];

// ---------------------------------------------------------------------------
// Config (read from TOML)
// ---------------------------------------------------------------------------

/// One task row at a single depth level. `honored`: did the model honor this
/// constraint at this placement depth?
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PositionalProbeTaskRow {
    pub honored: bool,
}

/// One depth level: a depth fraction + per-seed, per-task honored outcomes.
///
/// `outcomes` has `seed_count` rows per level (outer vec = seeds, inner vec =
/// tasks). For Tier-1 fixtures, all seeds share the same task count.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PositionalProbeLevel {
    /// Placement depth fraction in [0.0, 1.0].
    pub depth_fraction: f32,
    /// Per-seed, per-task honored outcomes. Outer vec = seeds, inner = tasks.
    pub outcomes: Vec<Vec<bool>>,
}

/// Positional-probe bench configuration (read from TOML at bench invocation time).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PositionalProbeConfig {
    /// Human-readable suite label (e.g. "positional_probe_fixture_v1").
    pub suite: String,
    /// Number of independent seeds (floor >= 5, I-38).
    pub seed_count: u32,
    /// `true` = same tasks appear across all depth levels (paired design).
    pub paired: bool,
    /// Model identity key the produced calibration artifact is keyed by.
    /// When absent, the bench derives a synthetic key from the suite label.
    #[serde(default)]
    pub identity_key: Option<String>,
    /// Ordered depth levels, from shallowest to deepest.
    pub levels: Vec<PositionalProbeLevel>,
}

// ---------------------------------------------------------------------------
// Error type
// ---------------------------------------------------------------------------

#[derive(Debug, Error)]
pub enum BenchPositionalProbeError {
    #[error(
        "positional-probe floor violation: seed_count={seed_count} (min {FLOOR_MIN_SEED_COUNT}), \
         paired={paired} (must be true)"
    )]
    FloorViolation { seed_count: u32, paired: bool },
    #[error(
        "positional-probe fixture has {level_count} depth level(s); need >= 2 for a curve"
    )]
    TooFewLevels { level_count: usize },
    #[error("depth level depth_fraction={depth_fraction:.4} has 0 tasks; each level must have >= 1 task")]
    EmptyLevel { depth_fraction: f32 },
    #[error(
        "depth level depth_fraction={depth_fraction:.4}: expected {expected} seeds, got {actual}"
    )]
    SeedMismatch { depth_fraction: f32, expected: u32, actual: usize },
    #[error(
        "depth level depth_fraction={depth_fraction:.4}: ragged outcomes, expected {expected} tasks per seed, got {actual}"
    )]
    RaggedOutcomes { depth_fraction: f32, expected: usize, actual: usize },
    #[error("depth level depth_fraction={depth_fraction:.4} is out of range [0.0, 1.0]")]
    DepthOutOfRange { depth_fraction: f32 },
    #[error("positional-probe could not read config at {path}: {source_msg}")]
    ConfigRead { path: String, source_msg: String },
    #[error("positional-probe could not parse config TOML: {source_msg}")]
    ConfigParse { source_msg: String },
    #[error("positional-probe could not create dir {path}: {source_msg}")]
    ReportDir { path: String, source_msg: String },
    #[error("positional-probe could not write {path}: {source_msg}")]
    OutputIo { path: String, source_msg: String },
    #[error("positional-probe could not serialise report: {source_msg}")]
    Serialise { source_msg: String },
    #[error("positional-probe could not build async runtime: {source_msg}")]
    Runtime { source_msg: String },
    #[error("positional-probe could not open SSB writer: {source_msg}")]
    SsbOpen { source_msg: String },
    #[error("positional-probe fixture audit failed: {reason}")]
    FixtureAudit { reason: String },
}

// ---------------------------------------------------------------------------
// Statistical floor enforcement
// ---------------------------------------------------------------------------

pub fn floor_check(config: &PositionalProbeConfig) -> Result<(), BenchPositionalProbeError> {
    if config.seed_count < FLOOR_MIN_SEED_COUNT || !config.paired {
        return Err(BenchPositionalProbeError::FloorViolation {
            seed_count: config.seed_count,
            paired: config.paired,
        });
    }
    Ok(())
}

pub fn validate_fixture(config: &PositionalProbeConfig) -> Result<(), BenchPositionalProbeError> {
    if config.levels.len() < 2 {
        return Err(BenchPositionalProbeError::TooFewLevels {
            level_count: config.levels.len(),
        });
    }
    for level in &config.levels {
        if level.depth_fraction < 0.0 || level.depth_fraction > 1.0 {
            return Err(BenchPositionalProbeError::DepthOutOfRange {
                depth_fraction: level.depth_fraction,
            });
        }
        if level.outcomes.is_empty() {
            return Err(BenchPositionalProbeError::EmptyLevel {
                depth_fraction: level.depth_fraction,
            });
        }
        if level.outcomes.len() != config.seed_count as usize {
            return Err(BenchPositionalProbeError::SeedMismatch {
                depth_fraction: level.depth_fraction,
                expected: config.seed_count,
                actual: level.outcomes.len(),
            });
        }
        if level.outcomes[0].is_empty() {
            return Err(BenchPositionalProbeError::EmptyLevel {
                depth_fraction: level.depth_fraction,
            });
        }
        let n_tasks = level.outcomes[0].len();
        for seed in &level.outcomes {
            if seed.len() != n_tasks {
                return Err(BenchPositionalProbeError::RaggedOutcomes {
                    depth_fraction: level.depth_fraction,
                    expected: n_tasks,
                    actual: seed.len(),
                });
            }
        }
    }
    Ok(())
}

/// Audit the fixture content for NEGATIVE cases: lorem/random padding must
/// fail the fixture audit. A valid fixture has structured constraint outcomes
/// (honored booleans), not filler. This checks that the suite label is not
/// "lorem" or "random" and that the outcomes are not all-true (which would
/// indicate a flat curve with no signal, not a real probe).
pub fn audit_fixture(config: &PositionalProbeConfig) -> Result<(), BenchPositionalProbeError> {
    let suite_lower = config.suite.to_lowercase();
    if suite_lower.contains("lorem") || suite_lower.contains("random") {
        return Err(BenchPositionalProbeError::FixtureAudit {
            reason: format!(
                "suite label '{suite}' contains lorem/random filler; fixture audit rejects unstructured padding",
                suite = config.suite
            ),
        });
    }
    let all_true = config
        .levels
        .iter()
        .flat_map(|lvl| lvl.outcomes.iter())
        .flat_map(|seed| seed.iter())
        .all(|&v| v);
    if all_true {
        return Err(BenchPositionalProbeError::FixtureAudit {
            reason: "all outcomes are 'honored=true'; flat curve has no probe signal (NEGATIVE: lorem/random padding)".to_string(),
        });
    }
    Ok(())
}

// ---------------------------------------------------------------------------
// Curve computation
// ---------------------------------------------------------------------------

/// Compute the mean recall_rate for a single depth level.
fn mean_rate_for_level(level: &PositionalProbeLevel) -> f64 {
    let n_seeds = level.outcomes.len();
    if n_seeds == 0 {
        return 0.0;
    }
    let n_tasks = level.outcomes[0].len();
    if n_tasks == 0 {
        return 0.0;
    }
    let total_honored: usize = level
        .outcomes
        .iter()
        .flat_map(|seed_outcomes| seed_outcomes.iter())
        .filter(|&&v| v)
        .count();
    total_honored as f64 / (n_seeds * n_tasks) as f64
}

/// Build a `PositionalValueCurve` from the config's depth levels (mean rate
/// per level). Samples are ordered by depth_fraction ascending.
fn build_positional_curve(config: &PositionalProbeConfig) -> PositionalValueCurve {
    let mut samples: Vec<PositionalSample> = config
        .levels
        .iter()
        .map(|lvl| PositionalSample {
            depth_fraction: lvl.depth_fraction,
            recall_rate: mean_rate_for_level(lvl) as f32,
        })
        .collect();
    samples.sort_by(|a, b| {
        a.depth_fraction
            .partial_cmp(&b.depth_fraction)
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    PositionalValueCurve { samples }
}

/// Detect non-monotonic levels (where recall_rate rises above prior level; a
/// positional-value curve is expected to be non-increasing, so an increase is
/// the anomaly to flag).
fn detect_non_monotonic(samples: &[PositionalSample]) -> Vec<String> {
    let mut non_monotonic = Vec::new();
    for i in 1..samples.len() {
        if f64::from(samples[i].recall_rate) > f64::from(samples[i - 1].recall_rate) {
            non_monotonic.push(format!("{:.4}", samples[i].depth_fraction));
        }
    }
    non_monotonic
}

/// Check if the curve is flat (all recall_rate values equal).
fn is_flat_curve(samples: &[PositionalSample]) -> bool {
    if samples.len() < 2 {
        return true;
    }
    let first = f64::from(samples[0].recall_rate);
    samples.iter().skip(1).all(|s| (f64::from(s.recall_rate) - first).abs() < 1e-9)
}

/// Detect the knee: the depth level where recall_rate drops most steeply.
/// Mirrors the fill-curve knee detection (largest step-down).
fn detect_knee(samples: &[PositionalSample]) -> Option<(f32, f32)> {
    if samples.len() < 2 {
        return None;
    }
    let mut best_idx: Option<usize> = None;
    let mut best_drop: f64 = 0.0;
    for i in 0..samples.len() - 1 {
        let delta = f64::from(samples[i + 1].recall_rate) - f64::from(samples[i].recall_rate);
        if delta < 0.0 && (-delta) > best_drop {
            best_drop = -delta;
            best_idx = Some(i);
        }
    }
    best_idx.map(|i| (samples[i].depth_fraction, best_drop as f32))
}

/// Resolve the model identity key: use the config's `identity_key` if present,
/// else derive a synthetic Tier-1 key from the suite label.
fn resolve_identity_key(config: &PositionalProbeConfig, config_bytes: &[u8]) -> String {
    if let Some(key) = &config.identity_key {
        if !key.is_empty() {
            return key.clone();
        }
    }
    let mut hasher = blake3::Hasher::new();
    hasher.update(b"fixture:positional-probe-v1:");
    hasher.update(config.suite.as_bytes());
    hasher.update(b":");
    hasher.update(config_bytes);
    format!("fixture:positional-probe:{}", &hasher.finalize().to_hex()[..16])
}

/// Read ELAI_SEED from the environment, defaulting to 0 if unset.
fn read_elai_seed() -> u64 {
    std::env::var("ELAI_SEED")
        .ok()
        .and_then(|s| s.parse::<u64>().ok())
        .unwrap_or(0)
}

// ---------------------------------------------------------------------------
// CLI entry
// ---------------------------------------------------------------------------

pub(crate) fn run(suite: PathBuf, seeds: u32, json: bool) -> anyhow::Result<()> {
    let project_root = std::env::current_dir().map_err(|e| anyhow::anyhow!("cwd: {e}"))?;
    match run_inner(suite, seeds, json, &project_root) {
        Ok(()) => Ok(()),
        Err(e) => {
            eprintln!("POSITIONAL-PROBE ABORT: {e}");
            std::process::exit(1);
        }
    }
}

pub fn run_inner(
    suite_path: PathBuf,
    seeds: u32,
    json: bool,
    project_root: &Path,
) -> Result<(), BenchPositionalProbeError> {
    let config_bytes = fs::read(&suite_path).map_err(|e| BenchPositionalProbeError::ConfigRead {
        path: suite_path.display().to_string(),
        source_msg: e.to_string(),
    })?;
    let mut config: PositionalProbeConfig =
        toml::from_str(&String::from_utf8_lossy(&config_bytes)).map_err(|e| {
            BenchPositionalProbeError::ConfigParse {
                source_msg: e.to_string(),
            }
        })?;

    if seeds > 0 {
        config.seed_count = seeds;
    }

    floor_check(&config)?;
    validate_fixture(&config)?;
    audit_fixture(&config)?;

    let fixture_blake3 = blake3::hash(&config_bytes).to_hex().to_string();
    let elai_seed = read_elai_seed();

    let positional_curve = build_positional_curve(&config);
    let non_monotonic_at = detect_non_monotonic(&positional_curve.samples);
    let flat = is_flat_curve(&positional_curve.samples);
    let knee = detect_knee(&positional_curve.samples);

    let identity_key = resolve_identity_key(&config, &config_bytes);

    let now_ms = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis() as i64)
        .unwrap_or_default();
    let now_secs = u64::try_from(now_ms / 1000).unwrap_or(0);

    let artifact = CalibrationArtifact::from_curves(
        identity_key.clone(),
        FillDegradationCurve::default(),
        positional_curve.clone(),
        now_secs,
    );

    let report_path = write_toml_report(
        project_root,
        &config.suite,
        &positional_curve,
        &artifact,
        &fixture_blake3,
        &non_monotonic_at,
        flat,
        knee,
        elai_seed,
        now_ms,
    )?;

    emit_calibration_artifact(project_root, artifact.clone())?;

    let monotonicity_note = if non_monotonic_at.is_empty() {
        "monotonic".to_string()
    } else {
        format!("non-monotonic at: {}", non_monotonic_at.join(", "))
    };
    let knee_label = match knee {
        Some((d, drop_)) => format!("depth={:.4} drop={:.4}", d, drop_),
        None => "none".to_string(),
    };
    let flat_note = if flat { "FLAT (no signal)" } else { "has signal" };
    println!(
        "POSITIONAL-PROBE suite={} seed_count={} n_levels={} knee={} {} \
         identity_key={} {} elai_seed={} fixture_blake3={} report={}",
        config.suite,
        config.seed_count,
        positional_curve.samples.len(),
        knee_label,
        flat_note,
        &identity_key[..identity_key.len().min(48)],
        monotonicity_note,
        elai_seed,
        &fixture_blake3[..16],
        report_path.display(),
    );
    if json {
        let body = serde_json::to_string_pretty(&positional_curve).map_err(|e| {
            BenchPositionalProbeError::Serialise {
                source_msg: e.to_string(),
            }
        })?;
        println!("{body}");
    }
    Ok(())
}

// ---------------------------------------------------------------------------
// TOML report writer
// ---------------------------------------------------------------------------

fn write_toml_report(
    project_root: &Path,
    suite: &str,
    positional_curve: &PositionalValueCurve,
    artifact: &CalibrationArtifact,
    fixture_blake3: &str,
    non_monotonic_at: &[String],
    flat: bool,
    knee: Option<(f32, f32)>,
    elai_seed: u64,
    ts_ms: i64,
) -> Result<PathBuf, BenchPositionalProbeError> {
    let report_dir = project_root.join("benchmarks").join("positional_probe");
    fs::create_dir_all(&report_dir).map_err(|e| BenchPositionalProbeError::ReportDir {
        path: report_dir.display().to_string(),
        source_msg: e.to_string(),
    })?;

    let safe_suite = suite.replace(|c: char| !c.is_alphanumeric() && c != '_' && c != '-', "_");
    let report_path = report_dir.join(format!("positional-probe-{safe_suite}-{ts_ms}.toml"));

    let knee_line = match knee {
        Some((d, drop_)) => format!(
            "knee_depth_fraction = {:.6}\nknee_drop_magnitude = {:.6}\n",
            d, drop_
        ),
        None => "knee_depth_fraction = \"none\"\n".to_string(),
    };

    let mut body = format!(
        "# J-34 positional-probe curve report\n\
         suite = {suite:?}\n\
         identity_key = {identity_key:?}\n\
         seed_count = {seed_count}\n\
         sample_count = {sample_count}\n\
         {knee_line}\
         content_hash = {content_hash:?}\n\
         fixture_blake3 = {blake3:?}\n\
         elai_seed = {elai_seed}\n\
         produced_at_secs = {produced_at_secs}\n\
         ts_ms = {ts_ms}\n\
         flat_curve = {flat}\n\n\
         non_monotonic_at = {non_mon:?}\n\n",
        suite = suite,
        identity_key = artifact.identity_key,
        seed_count = positional_curve.samples.len(),
        sample_count = positional_curve.samples.len(),
        content_hash = artifact.content_hash,
        blake3 = fixture_blake3,
        elai_seed = elai_seed,
        produced_at_secs = artifact.produced_at_secs,
        ts_ms = ts_ms,
        flat = flat,
        non_mon = non_monotonic_at,
    );

    for (i, s) in positional_curve.samples.iter().enumerate() {
        body.push_str(&format!(
            "[[samples]]\ndepth_fraction = {:.6}\nrecall_rate = {:.6}\n",
            s.depth_fraction, s.recall_rate,
        ));
        if i + 1 < positional_curve.samples.len() {
            body.push('\n');
        }
    }

    fs::write(&report_path, &body).map_err(|e| BenchPositionalProbeError::OutputIo {
        path: report_path.display().to_string(),
        source_msg: e.to_string(),
    })?;
    Ok(report_path)
}

// ---------------------------------------------------------------------------
// SSB emit (via CalibrationArtifactStore::insert_with_ssb, slice-2 seam)
// ---------------------------------------------------------------------------

fn emit_calibration_artifact(
    project_root: &Path,
    artifact: CalibrationArtifact,
) -> Result<(), BenchPositionalProbeError> {
    let db_path = sessions_dir_path(project_root).join("ssb.db");
    if let Some(parent) = db_path.parent() {
        fs::create_dir_all(parent).map_err(|e| BenchPositionalProbeError::ReportDir {
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
    let clock: Arc<dyn elai_core::clock::ClockProvider> = Arc::new(elai_core::OsClock);

    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .map_err(|e| BenchPositionalProbeError::Runtime {
            source_msg: e.to_string(),
        })?;

    let identity_key = artifact.identity_key.clone();
    rt.block_on(async move {
        let ssb: Arc<Ssb> = Arc::new(
            Ssb::new(cfg, clock).map_err(|e| BenchPositionalProbeError::SsbOpen {
                source_msg: e.to_string(),
            })?,
        );

        let mut store = CalibrationArtifactStore::new();
        store.insert_with_ssb(artifact, Some(&ssb));

        tokio::time::sleep(std::time::Duration::from_millis(80)).await;
        let rows = ssb
            .query_by_type(SsbActionType::ContextCalibrationEvent(
                elai_orchestrator::ssb::ContextCalibrationKind::CurveMeasured,
            ))
            .map_err(|e| BenchPositionalProbeError::SsbOpen {
                source_msg: format!("query_by_type failed: {e}"),
            })?;
        if rows.is_empty() {
            return Err(BenchPositionalProbeError::SsbOpen {
                source_msg: format!(
                    "no ContextCalibrationEvent(CurveMeasured) row landed for identity_key={identity_key}"
                ),
            });
        }

        tracing::info!(
            positional_probe.ssb_rows = rows.len(),
            "positional-probe: J-34 CurveMeasured SSB row emitted via CalibrationArtifactStore"
        );

        Ok::<(), BenchPositionalProbeError>(())
    })
}

#[cfg(test)]
mod tests_inline;
