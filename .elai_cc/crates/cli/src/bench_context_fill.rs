//! J-34 leg-1 -- `elai bench context-fill --suite <PATH> [--seeds <u32>] [--json]`
//! CLI driver.
//!
//! CONTEXT-FILL = verified-outcome-rate vs context-window fill fraction curve
//! per model. The curve carries the W-1-grade verified-outcome rate (gate
//! verdicts, never LLM-judged -- TD-3) at each fill level. Tier-1 = fixture
//! TOML only (no live inference, no GPU); the fixture supplies verified
//! boolean outcomes per task per seed per fill level.
//!
//! # What this measures (honestly)
//!
//! For each fill level L in the fixture suite, the bench counts the fraction
//! of tasks that produced a verified outcome. It does NOT use pass@k
//! (stub_ban). Monotonicity is MEASURED and REPORTED -- if the rate rises
//! between adjacent fill levels (degradation curves are expected to be
//! non-increasing), the report flags it as "non-monotonic at [level];
//! investigate." Nothing is smoothed (GD-5).
//!
//! # Fixture tier (Tier-1)
//!
//! The fixture TOML supplies verified boolean outcomes per task per seed per
//! fill level. No live inference runs. The live per-model inference leg is
//! PENDING (WIRE-DARK[J-34.leg1-live]); the payload shape is forward-
//! compatible (fill_fraction is an opaque f32).
//!
//! # Statistical floor (mechanically enforced)
//!
//!   - `seed_count >= 5`  (SEM meaningful at this N, I-38 floor)
//!   - `paired == true`   (same tasks across fill levels)
//!
//! # Wiring
//!
//! producer  : this file -- reads ContextFillConfig TOML, validates floor,
//!             sweeps fill levels, computes mean outcome rate per level,
//!             builds a FillDegradationCurve, constructs a CalibrationArtifact
//!             via `CalibrationArtifact::from_curves`, opens the canonical
//!             Ssb writer, calls `CalibrationArtifactStore::insert_with_ssb`
//!             (which emits the ContextCalibrationEvent(CurveMeasured) SSB
//!             row via slice-2 seam), and writes a timestamped TOML report.
//! ssb row   : `SsbActionType::ContextCalibrationEvent(ContextCalibrationKind::CurveMeasured)`
//!             (Pattern-2 single-variant, 4-arm Kind rides in payload).
//! entrypoint: `elai bench context-fill --suite <PATH> [--seeds <u32>]`.
//!
//! # No `.unwrap()` / `.expect()` in non-test paths.
//! # blake3 only for all content hashing.

use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

use elai_core::paths::sessions_dir_path;
use elai_orchestrator::context_calibration::{
    CalibrationArtifact, CalibrationArtifactStore, FillDegradationCurve, FillSample,
    PositionalValueCurve,
};
use elai_orchestrator::ssb::{SsbActionType, SsbReader as _};
use elai_orchestrator::{Ssb, SsbConfig};
use serde::{Deserialize, Serialize};
use thiserror::Error;

// ---------------------------------------------------------------------------
// Constants
// ---------------------------------------------------------------------------

/// Minimum seed count required by the I-38 statistical floor.
const FLOOR_MIN_SEED_COUNT: u32 = 5;

/// Marginal gain threshold below which the curve is considered past the "knee".
/// Mirrors bench_convergence::KNEE_GAIN_THRESHOLD for reporting parity; the
/// artifact's own knee detection (`context_calibration::detect_knee`) uses the
/// largest step-down and is the authoritative knee for trim_fraction.
const KNEE_GAIN_THRESHOLD: f64 = 0.01;

/// Bench session id embedded in every SSB row this bench emits.
const BENCH_SESSION_ID: &str = "cli-bench-context-fill";

/// Agent id label for SSB rows.
const BENCH_AGENT_ID: &str = "cli:bench-context-fill";

// ---------------------------------------------------------------------------
// Config (read from TOML)
// ---------------------------------------------------------------------------

/// One task row at a single fill level. `verified`: did this task produce a
/// W-1-grade verified outcome at this fill level?
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ContextFillTaskRow {
    /// True if the task produced a verified outcome at this fill level.
    pub verified: bool,
}

/// One fill level: a fill fraction + per-seed, per-task verified outcomes.
///
/// `outcomes` has `seed_count` rows per level (outer vec = seeds, inner vec =
/// tasks). For Tier-1 fixtures, all seeds share the same task count.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ContextFillLevel {
    /// Context-window fill fraction in [0.0, 1.0].
    pub fill_fraction: f32,
    /// Per-seed, per-task verified outcomes. Outer vec = seeds, inner = tasks.
    /// All seeds must have the same task count.
    pub outcomes: Vec<Vec<bool>>,
}

/// Context-fill bench configuration (read from TOML at bench invocation time).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ContextFillConfig {
    /// Human-readable suite label (e.g. "context_fill_fixture_v1").
    pub suite: String,
    /// Number of independent seeds (floor >= 5, I-38).
    pub seed_count: u32,
    /// `true` = same tasks appear across all fill levels (paired design).
    pub paired: bool,
    /// Model identity key (`ModelIdentity::identity_key`, model_drift.rs:160)
    /// the produced calibration artifact is keyed by. When absent, the bench
    /// derives a synthetic key from the suite label (Tier-1 fixture mode).
    #[serde(default)]
    pub identity_key: Option<String>,
    /// Ordered fill levels, from lowest to highest fill fraction.
    pub levels: Vec<ContextFillLevel>,
}

// ---------------------------------------------------------------------------
// Error type
// ---------------------------------------------------------------------------

#[derive(Debug, Error)]
pub enum BenchContextFillError {
    /// Statistical floor violated: run config rejected before any compute.
    #[error(
        "context-fill floor violation: seed_count={seed_count} (min {FLOOR_MIN_SEED_COUNT}), \
         paired={paired} (must be true)"
    )]
    FloorViolation { seed_count: u32, paired: bool },
    /// The fixture has fewer than 2 fill levels.
    #[error(
        "context-fill fixture has {level_count} fill level(s); need >= 2 for a curve"
    )]
    TooFewLevels { level_count: usize },
    /// A fill level has no tasks.
    #[error("fill level fill_fraction={fill_fraction:.4} has 0 tasks; each level must have >= 1 task")]
    EmptyLevel { fill_fraction: f32 },
    /// Seed count in a level does not match config.seed_count.
    #[error(
        "fill level fill_fraction={fill_fraction:.4}: expected {expected} seeds, got {actual}"
    )]
    SeedMismatch { fill_fraction: f32, expected: u32, actual: usize },
    /// Per-seed task counts disagree within a level.
    #[error(
        "fill level fill_fraction={fill_fraction:.4}: ragged outcomes, expected {expected} tasks per seed, got {actual}"
    )]
    RaggedOutcomes { fill_fraction: f32, expected: usize, actual: usize },
    /// Config file could not be read.
    #[error("context-fill could not read config at {path}: {source_msg}")]
    ConfigRead { path: String, source_msg: String },
    /// TOML deserialization failure.
    #[error("context-fill could not parse config TOML: {source_msg}")]
    ConfigParse { source_msg: String },
    /// Could not create the on-disk report directory.
    #[error("context-fill could not create dir {path}: {source_msg}")]
    ReportDir { path: String, source_msg: String },
    /// Generic I/O failure writing the TOML report.
    #[error("context-fill could not write {path}: {source_msg}")]
    OutputIo { path: String, source_msg: String },
    /// TOML serialisation failure.
    #[error("context-fill could not serialise report: {source_msg}")]
    Serialise { source_msg: String },
    /// Building the async runtime failed.
    #[error("context-fill could not build async runtime: {source_msg}")]
    Runtime { source_msg: String },
    /// Opening the SSB single-writer failed.
    #[error("context-fill could not open SSB writer: {source_msg}")]
    SsbOpen { source_msg: String },
}

// ---------------------------------------------------------------------------
// Statistical floor enforcement
// ---------------------------------------------------------------------------

/// Validate the I-38 statistical floor invariants.
/// Returns `Err(FloorViolation)` if `seed_count < 5` OR `paired == false`.
pub fn floor_check(config: &ContextFillConfig) -> Result<(), BenchContextFillError> {
    if config.seed_count < FLOOR_MIN_SEED_COUNT || !config.paired {
        return Err(BenchContextFillError::FloorViolation {
            seed_count: config.seed_count,
            paired: config.paired,
        });
    }
    Ok(())
}

/// Validate the fixture structure: >= 2 levels, each level has >= 1 task and
/// the correct number of seeds with consistent per-seed task counts.
pub fn validate_fixture(config: &ContextFillConfig) -> Result<(), BenchContextFillError> {
    if config.levels.len() < 2 {
        return Err(BenchContextFillError::TooFewLevels {
            level_count: config.levels.len(),
        });
    }
    for level in &config.levels {
        if level.outcomes.is_empty() {
            return Err(BenchContextFillError::EmptyLevel {
                fill_fraction: level.fill_fraction,
            });
        }
        if level.outcomes.len() != config.seed_count as usize {
            return Err(BenchContextFillError::SeedMismatch {
                fill_fraction: level.fill_fraction,
                expected: config.seed_count,
                actual: level.outcomes.len(),
            });
        }
        if level.outcomes[0].is_empty() {
            return Err(BenchContextFillError::EmptyLevel {
                fill_fraction: level.fill_fraction,
            });
        }
        let n_tasks = level.outcomes[0].len();
        for seed in &level.outcomes {
            if seed.len() != n_tasks {
                return Err(BenchContextFillError::RaggedOutcomes {
                    fill_fraction: level.fill_fraction,
                    expected: n_tasks,
                    actual: seed.len(),
                });
            }
        }
    }
    Ok(())
}

// ---------------------------------------------------------------------------
// Curve computation
// ---------------------------------------------------------------------------

/// Compute the mean verified-outcome rate for a single fill level.
fn mean_rate_for_level(level: &ContextFillLevel) -> f64 {
    let n_seeds = level.outcomes.len();
    if n_seeds == 0 {
        return 0.0;
    }
    let n_tasks = level.outcomes[0].len();
    if n_tasks == 0 {
        return 0.0;
    }
    let total_verified: usize = level
        .outcomes
        .iter()
        .flat_map(|seed_outcomes| seed_outcomes.iter())
        .filter(|&&v| v)
        .count();
    total_verified as f64 / (n_seeds * n_tasks) as f64
}

/// Build a `FillDegradationCurve` from the config's fill levels (mean rate
/// per level). Samples are ordered by fill_fraction ascending (defensive;
/// `detect_knee` re-sorts anyway).
fn build_fill_curve(config: &ContextFillConfig) -> FillDegradationCurve {
    let mut samples: Vec<FillSample> = config
        .levels
        .iter()
        .map(|lvl| FillSample {
            fill_fraction: lvl.fill_fraction,
            outcome_rate: mean_rate_for_level(lvl) as f32,
        })
        .collect();
    samples.sort_by(|a, b| {
        a.fill_fraction
            .partial_cmp(&b.fill_fraction)
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    FillDegradationCurve { samples }
}

/// Detect the knee: the first level where the marginal gain collapses.
/// Mirrors bench_convergence::detect_knee for the human-readable report; the
/// artifact's own `detect_knee` (largest step-down) is the authoritative knee
/// for `calibrated_trim_fraction`.
fn detect_knee_label(samples: &[FillSample]) -> String {
    if samples.is_empty() {
        return String::new();
    }
    for i in 1..samples.len() {
        let gain = f64::from(samples[i].outcome_rate) - f64::from(samples[i - 1].outcome_rate);
        if gain < KNEE_GAIN_THRESHOLD {
            return format!("{:.4}", samples[i - 1].fill_fraction);
        }
    }
    samples
        .last()
        .map(|p| format!("{:.4}", p.fill_fraction))
        .unwrap_or_default()
}

/// Detect non-monotonic levels (where rate rises above prior level; a
/// degradation curve is expected to be non-increasing, so an increase is the
/// anomaly to flag -- the mirror of bench_convergence's non-monotonic check).
fn detect_non_monotonic(samples: &[FillSample]) -> Vec<String> {
    let mut non_monotonic = Vec::new();
    for i in 1..samples.len() {
        if f64::from(samples[i].outcome_rate) > f64::from(samples[i - 1].outcome_rate) {
            non_monotonic.push(format!("{:.4}", samples[i].fill_fraction));
        }
    }
    non_monotonic
}

/// Resolve the model identity key: use the config's `identity_key` if present,
/// else derive a synthetic Tier-1 key from the suite label. The synthetic key
/// is prefixed so it never collides with a real `ModelIdentity::identity_key`
/// (blake3 of model id string).
fn resolve_identity_key(config: &ContextFillConfig, config_bytes: &[u8]) -> String {
    if let Some(key) = &config.identity_key {
        if !key.is_empty() {
            return key.clone();
        }
    }
    // Synthetic Tier-1 key: blake3 of (suite label + fixture bytes) so the
    // artifact is keyed by the exact fixture content (GD-2 byte-recomputability).
    let mut hasher = blake3::Hasher::new();
    hasher.update(b"fixture:context-fill-v1:");
    hasher.update(config.suite.as_bytes());
    hasher.update(b":");
    hasher.update(config_bytes);
    format!("fixture:context-fill:{}", &hasher.finalize().to_hex()[..16])
}

// ---------------------------------------------------------------------------
// CLI entry
// ---------------------------------------------------------------------------

/// CLI entry -- validates floor, sweeps fill levels, builds the
/// FillDegradationCurve, constructs a CalibrationArtifact, opens the Ssb
/// writer, inserts the artifact via `CalibrationArtifactStore::insert_with_ssb`
/// (emits the ContextCalibrationEvent(CurveMeasured) SSB row), and writes a
/// timestamped TOML report.
///
/// Returns `Ok(())` on a clean run (rc=0). On a handled error it prints a
/// `CONTEXT-FILL ABORT:` line to stderr and calls `std::process::exit(1)`.
pub(crate) fn run(suite: PathBuf, seeds: u32, json: bool) -> anyhow::Result<()> {
    let project_root = std::env::current_dir().map_err(|e| anyhow::anyhow!("cwd: {e}"))?;
    match run_inner(suite, seeds, json, &project_root) {
        Ok(()) => Ok(()),
        Err(e) => {
            eprintln!("CONTEXT-FILL ABORT: {e}");
            std::process::exit(1);
        }
    }
}

/// Core -- validates floor, sweeps fill levels, builds curve + artifact,
/// emits CurveMeasured SSB row via the artifact store, writes TOML report.
/// Public so integration/contract tests can drive without process exit.
pub fn run_inner(
    suite_path: PathBuf,
    seeds: u32,
    json: bool,
    project_root: &Path,
) -> Result<(), BenchContextFillError> {
    // 1. Read and parse the TOML config.
    let config_bytes = fs::read(&suite_path).map_err(|e| BenchContextFillError::ConfigRead {
        path: suite_path.display().to_string(),
        source_msg: e.to_string(),
    })?;
    let mut config: ContextFillConfig =
        toml::from_str(&String::from_utf8_lossy(&config_bytes)).map_err(|e| {
            BenchContextFillError::ConfigParse {
                source_msg: e.to_string(),
            }
        })?;

    // 2. Override seed_count if --seeds was provided (> 0 means user specified).
    if seeds > 0 {
        config.seed_count = seeds;
    }

    // 3. Statistical floor check -- exits nonzero before any compute.
    floor_check(&config)?;

    // 4. Fixture structure check.
    validate_fixture(&config)?;

    // 5. blake3 hash of config bytes as corpus provenance (anti-gaming, GD-2).
    let fixture_blake3 = blake3::hash(&config_bytes).to_hex().to_string();

    // 6. Build the FillDegradationCurve from mean rates per fill level.
    let fill_curve = build_fill_curve(&config);

    // 7. Knee detection and monotonicity check (report-facing).
    let knee_fill_label = detect_knee_label(&fill_curve.samples);
    let non_monotonic_at = detect_non_monotonic(&fill_curve.samples);

    // 8. Resolve the model identity key.
    let identity_key = resolve_identity_key(&config, &config_bytes);

    // 9. Wall-clock timestamp (seconds for artifact, millis for report).
    let now_ms = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis() as i64)
        .unwrap_or_default();
    let now_secs = u64::try_from(now_ms / 1000).unwrap_or(0);

    // 10. Construct the CalibrationArtifact (computes knee + content hash).
    let artifact = CalibrationArtifact::from_curves(
        identity_key.clone(),
        fill_curve.clone(),
        PositionalValueCurve::default(),
        now_secs,
    );

    // 11. Write timestamped TOML report under <project_root>/benchmarks/context_fill/.
    let report_path = write_toml_report(
        project_root,
        &config.suite,
        &fill_curve,
        &artifact,
        &fixture_blake3,
        &non_monotonic_at,
        now_ms,
    )?;

    // 12. Open the canonical Ssb writer and insert the artifact via the store
    //     (slice-2 seam: insert_with_ssb emits the CurveMeasured SSB row).
    emit_calibration_artifact(project_root, artifact.clone())?;

    // 13. Summary line.
    let monotonicity_note = if non_monotonic_at.is_empty() {
        "monotonic".to_string()
    } else {
        format!("non-monotonic at: {}", non_monotonic_at.join(", "))
    };
    let artifact_knee = artifact
        .knee
        .as_ref()
        .map(|k| format!("{:.4}", k.knee_fill_fraction))
        .unwrap_or_else(|| "none".to_string());
    println!(
        "CONTEXT-FILL suite={} seed_count={} n_levels={} knee_report={} knee_artifact={} \
         identity_key={} {} fixture_blake3={} report={}",
        config.suite,
        config.seed_count,
        fill_curve.samples.len(),
        knee_fill_label,
        artifact_knee,
        &identity_key[..identity_key.len().min(48)],
        monotonicity_note,
        &fixture_blake3[..16],
        report_path.display(),
    );
    if json {
        let body = serde_json::to_string_pretty(&fill_curve).map_err(|e| {
            BenchContextFillError::Serialise {
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

/// Write the context-fill report to
/// `<project_root>/benchmarks/context_fill/context-fill-<suite>-<unix_ms>.toml`.
fn write_toml_report(
    project_root: &Path,
    suite: &str,
    fill_curve: &FillDegradationCurve,
    artifact: &CalibrationArtifact,
    fixture_blake3: &str,
    non_monotonic_at: &[String],
    ts_ms: i64,
) -> Result<PathBuf, BenchContextFillError> {
    let report_dir = project_root.join("benchmarks").join("context_fill");
    fs::create_dir_all(&report_dir).map_err(|e| BenchContextFillError::ReportDir {
        path: report_dir.display().to_string(),
        source_msg: e.to_string(),
    })?;

    let safe_suite = suite.replace(|c: char| !c.is_alphanumeric() && c != '_' && c != '-', "_");
    let report_path = report_dir.join(format!("context-fill-{safe_suite}-{ts_ms}.toml"));

    let knee_line = match &artifact.knee {
        Some(k) => format!(
            "knee_fill_fraction = {:.6}\ndrop_magnitude = {:.6}\n",
            k.knee_fill_fraction, k.drop_magnitude
        ),
        None => "knee_fill_fraction = \"none\"\n".to_string(),
    };

    let mut body = format!(
        "# J-34 context-fill curve report\n\
         suite = {suite:?}\n\
         identity_key = {identity_key:?}\n\
         seed_count = {seed_count}\n\
         sample_count = {sample_count}\n\
         {knee_line}\
         content_hash = {content_hash:?}\n\
         fixture_blake3 = {blake3:?}\n\
         produced_at_secs = {produced_at_secs}\n\
         ts_ms = {ts_ms}\n\n\
         non_monotonic_at = {non_mon:?}\n\n",
        suite = suite,
        identity_key = artifact.identity_key,
        seed_count = artifact.fill_curve.samples.len(),
        sample_count = artifact.fill_curve.samples.len(),
        content_hash = artifact.content_hash,
        blake3 = fixture_blake3,
        produced_at_secs = artifact.produced_at_secs,
        ts_ms = ts_ms,
        non_mon = non_monotonic_at,
    );

    for (i, s) in fill_curve.samples.iter().enumerate() {
        body.push_str(&format!(
            "[[samples]]\nfill_fraction = {:.6}\noutcome_rate = {:.6}\n",
            s.fill_fraction, s.outcome_rate,
        ));
        if i + 1 < fill_curve.samples.len() {
            body.push('\n');
        }
    }

    fs::write(&report_path, &body).map_err(|e| BenchContextFillError::OutputIo {
        path: report_path.display().to_string(),
        source_msg: e.to_string(),
    })?;
    Ok(report_path)
}

// ---------------------------------------------------------------------------
// SSB emit (via CalibrationArtifactStore::insert_with_ssb, slice-2 seam)
// ---------------------------------------------------------------------------

/// Open the canonical session `ssb.db`, construct a `CalibrationArtifactStore`,
/// and call `insert_with_ssb` so the ContextCalibrationEvent(CurveMeasured)
/// SSB row is emitted by the slice-2 seam. Mirrors bench_convergence::emit_beacon
/// shape but threads the artifact through the store so the emit path is the
/// SAME one production recalibrate uses (R8: frozen gate, no bench-specific
/// SSB write path that could diverge from the canonical seam).
fn emit_calibration_artifact(
    project_root: &Path,
    artifact: CalibrationArtifact,
) -> Result<(), BenchContextFillError> {
    let db_path = sessions_dir_path(project_root).join("ssb.db");
    if let Some(parent) = db_path.parent() {
        fs::create_dir_all(parent).map_err(|e| BenchContextFillError::ReportDir {
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
        .map_err(|e| BenchContextFillError::Runtime {
            source_msg: e.to_string(),
        })?;

    let identity_key = artifact.identity_key.clone();
    rt.block_on(async move {
        let ssb: Arc<Ssb> = Arc::new(
            Ssb::new(cfg, clock).map_err(|e| BenchContextFillError::SsbOpen {
                source_msg: e.to_string(),
            })?,
        );

        let mut store = CalibrationArtifactStore::new();
        store.insert_with_ssb(artifact, Some(&ssb));

        // Verify the CurveMeasured row landed (R1: missing evidence = FAIL).
        // The insert_with_ssb emit is best-effort in the store, but the bench
        // treats a missing row as a hard failure because the bench's whole
        // purpose is to produce the row.
        tokio::time::sleep(std::time::Duration::from_millis(80)).await;
        let rows = ssb
            .query_by_type(SsbActionType::ContextCalibrationEvent(
                elai_orchestrator::ssb::ContextCalibrationKind::CurveMeasured,
            ))
            .map_err(|e| BenchContextFillError::SsbOpen {
                source_msg: format!("query_by_type failed: {e}"),
            })?;
        if rows.is_empty() {
            return Err(BenchContextFillError::SsbOpen {
                source_msg: format!(
                    "no ContextCalibrationEvent(CurveMeasured) row landed for identity_key={identity_key}"
                ),
            });
        }

        tracing::info!(
            context_fill.ssb_rows = rows.len(),
            "context-fill: J-34 CurveMeasured SSB row emitted via CalibrationArtifactStore"
        );

        Ok::<(), BenchContextFillError>(())
    })
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    /// Valid fixture: 3 fill levels, 5 seeds each, 4 tasks per seed.
    /// Level 0.10: 4/4 verified -> rate 1.00
    /// Level 0.50: 3/4 verified -> rate 0.75
    /// Level 0.90: 1/4 verified -> rate 0.25
    /// Monotonically decreasing (degradation curve); knee at 0.50 (largest
    /// step-down 0.75 -> 0.25 = -0.50).
    const CONTEXT_FILL_FIXTURE_VALID: &str = r#"
suite = "test_context_fill_fixture"
seed_count = 5
paired = true

[[levels]]
fill_fraction = 0.10
outcomes = [
  [true, true, true, true],
  [true, true, true, true],
  [true, true, true, true],
  [true, true, true, true],
  [true, true, true, true]
]

[[levels]]
fill_fraction = 0.50
outcomes = [
  [true, true, true, false],
  [true, true, true, false],
  [true, true, true, false],
  [true, true, true, false],
  [true, true, true, false]
]

[[levels]]
fill_fraction = 0.90
outcomes = [
  [true, false, false, false],
  [true, false, false, false],
  [true, false, false, false],
  [true, false, false, false],
  [true, false, false, false]
]
"#;

    /// Under-powered fixture (seed_count=3, paired=false).
    const CONTEXT_FILL_FIXTURE_UNDERPOWER: &str = r#"
suite = "test_context_fill_underpower"
seed_count = 3
paired = false

[[levels]]
fill_fraction = 0.10
outcomes = [[true, false], [true, false], [false, false]]

[[levels]]
fill_fraction = 0.50
outcomes = [[true, true], [true, true], [true, false]]
"#;

    /// Non-monotonic fixture: level 0.50 has a HIGHER rate than 0.10 (a
    /// degradation curve should be non-increasing; this is the anomaly).
    const CONTEXT_FILL_FIXTURE_NON_MONOTONIC: &str = r#"
suite = "test_context_fill_non_monotonic"
seed_count = 5
paired = true

[[levels]]
fill_fraction = 0.10
outcomes = [
  [true, false, false, false],
  [true, false, false, false],
  [true, false, false, false],
  [true, false, false, false],
  [true, false, false, false]
]

[[levels]]
fill_fraction = 0.50
outcomes = [
  [true, true, true, true],
  [true, true, true, true],
  [true, true, true, true],
  [true, true, true, true],
  [true, true, true, true]
]

[[levels]]
fill_fraction = 0.90
outcomes = [
  [true, false, false, false],
  [true, false, false, false],
  [true, false, false, false],
  [true, false, false, false],
  [true, false, false, false]
]
"#;

    fn parse_fixture(s: &str) -> ContextFillConfig {
        toml::from_str(s).expect("fixture parse")
    }

    // -----------------------------------------------------------------------
    // Floor check tests
    // -----------------------------------------------------------------------

    #[test]
    fn floor_check_passes_valid_config() {
        let config = parse_fixture(CONTEXT_FILL_FIXTURE_VALID);
        assert!(floor_check(&config).is_ok(), "valid config must pass floor_check");
    }

    #[test]
    fn floor_check_rejects_underpower() {
        let config = parse_fixture(CONTEXT_FILL_FIXTURE_UNDERPOWER);
        let result = floor_check(&config);
        assert!(result.is_err(), "under-powered config must fail floor_check");
        match result.unwrap_err() {
            BenchContextFillError::FloorViolation { seed_count, paired } => {
                assert_eq!(seed_count, 3);
                assert!(!paired);
            }
            other => panic!("expected FloorViolation, got: {other}"),
        }
    }

    #[test]
    fn floor_check_rejects_seed_count_below_5() {
        let mut config = parse_fixture(CONTEXT_FILL_FIXTURE_VALID);
        config.seed_count = 4;
        assert!(floor_check(&config).is_err(), "seed_count=4 must fail floor_check");
    }

    #[test]
    fn floor_check_rejects_unpaired() {
        let mut config = parse_fixture(CONTEXT_FILL_FIXTURE_VALID);
        config.paired = false;
        assert!(floor_check(&config).is_err(), "paired=false must fail floor_check");
    }

    // -----------------------------------------------------------------------
    // Mean rate computation
    // -----------------------------------------------------------------------

    #[test]
    fn mean_rate_correct_for_all_verified() {
        let config = parse_fixture(CONTEXT_FILL_FIXTURE_VALID);
        let level_010 = &config.levels[0];
        let rate = mean_rate_for_level(level_010);
        assert!(
            (rate - 1.0_f64).abs() < 1e-9,
            "0.10 level mean_rate must be 1.00; got {rate}"
        );
    }

    #[test]
    fn mean_rate_correct_for_quarter_verified() {
        let config = parse_fixture(CONTEXT_FILL_FIXTURE_VALID);
        let level_090 = &config.levels[2];
        let rate = mean_rate_for_level(level_090);
        assert!(
            (rate - 0.25_f64).abs() < 1e-9,
            "0.90 level mean_rate must be 0.25; got {rate}"
        );
    }

    // -----------------------------------------------------------------------
    // Curve construction
    // -----------------------------------------------------------------------

    #[test]
    fn build_fill_curve_produces_ordered_samples() {
        let config = parse_fixture(CONTEXT_FILL_FIXTURE_VALID);
        let curve = build_fill_curve(&config);
        assert_eq!(curve.samples.len(), 3);
        // Ordered by fill_fraction ascending.
        assert!((curve.samples[0].fill_fraction - 0.10).abs() < 1e-6);
        assert!((curve.samples[1].fill_fraction - 0.50).abs() < 1e-6);
        assert!((curve.samples[2].fill_fraction - 0.90).abs() < 1e-6);
        // Rates match mean_rate_for_level.
        assert!((f64::from(curve.samples[0].outcome_rate) - 1.00).abs() < 1e-6);
        assert!((f64::from(curve.samples[1].outcome_rate) - 0.75).abs() < 1e-6);
        assert!((f64::from(curve.samples[2].outcome_rate) - 0.25).abs() < 1e-6);
    }

    // -----------------------------------------------------------------------
    // Knee detection (report-facing label)
    // -----------------------------------------------------------------------

    #[test]
    fn knee_label_for_monotone_decreasing_is_last_collapse() {
        let config = parse_fixture(CONTEXT_FILL_FIXTURE_VALID);
        let curve = build_fill_curve(&config);
        let label = detect_knee_label(&curve.samples);
        // Gains: 0.50-0.10 = -0.25 (< KNEE_GAIN_THRESHOLD=0.01), so knee
        // is at the level BEFORE the first collapse, i.e. 0.10.
        assert!(
            label.starts_with("0.10"),
            "knee label must be 0.10 (first collapse starts at 0.10); got {label}"
        );
    }

    // -----------------------------------------------------------------------
    // Monotonicity detection (GD-5 honesty)
    // -----------------------------------------------------------------------

    #[test]
    fn non_monotonic_detected_correctly() {
        let config = parse_fixture(CONTEXT_FILL_FIXTURE_NON_MONOTONIC);
        let curve = build_fill_curve(&config);
        let non_mon = detect_non_monotonic(&curve.samples);
        // 0.50 rises from 0.25 (0.10) to 1.00 (0.50) -- a rise on a
        // degradation curve is the anomaly.
        assert!(
            non_mon.iter().any(|s| s.starts_with("0.50")),
            "non_monotonic_at must contain '0.50'; got: {non_mon:?}"
        );
    }

    #[test]
    fn monotonic_fixture_has_empty_non_monotonic_list() {
        let config = parse_fixture(CONTEXT_FILL_FIXTURE_VALID);
        let curve = build_fill_curve(&config);
        let non_mon = detect_non_monotonic(&curve.samples);
        assert!(non_mon.is_empty(), "monotonic fixture must produce empty non_monotonic_at");
    }

    // -----------------------------------------------------------------------
    // Identity key resolution
    // -----------------------------------------------------------------------

    #[test]
    fn resolve_identity_key_uses_config_when_present() {
        let mut config = parse_fixture(CONTEXT_FILL_FIXTURE_VALID);
        config.identity_key = Some("model:test-identity".to_string());
        let key = resolve_identity_key(&config, b"some-bytes");
        assert_eq!(key, "model:test-identity");
    }

    #[test]
    fn resolve_identity_key_synthetic_when_absent() {
        let config = parse_fixture(CONTEXT_FILL_FIXTURE_VALID);
        let key = resolve_identity_key(&config, b"some-bytes");
        assert!(
            key.starts_with("fixture:context-fill:"),
            "synthetic key must be prefixed; got {key}"
        );
        // Same input -> same key (deterministic, GD-2).
        let key2 = resolve_identity_key(&config, b"some-bytes");
        assert_eq!(key, key2);
        // Different bytes -> different key.
        let key3 = resolve_identity_key(&config, b"different-bytes");
        assert_ne!(key, key3);
    }

    // -----------------------------------------------------------------------
    // Artifact construction (end-to-end curve -> artifact)
    // -----------------------------------------------------------------------

    #[test]
    fn artifact_from_built_curve_has_knee_and_hash() {
        let config = parse_fixture(CONTEXT_FILL_FIXTURE_VALID);
        let curve = build_fill_curve(&config);
        let art = CalibrationArtifact::from_curves(
            "test-identity".to_string(),
            curve,
            PositionalValueCurve::default(),
            1_700_000_000,
        );
        // Largest step-down is 0.75 -> 0.25 (-0.50) at fill=0.50.
        let knee = art.knee.as_ref().expect("monotone-decreasing curve has a knee");
        assert!((knee.knee_fill_fraction - 0.50).abs() < 1e-6);
        assert!((knee.drop_magnitude - 0.50).abs() < 1e-6);
        assert!(!art.content_hash.is_empty());
        assert_eq!(art.content_hash, art.compute_hash());
    }

    // -----------------------------------------------------------------------
    // run_inner end-to-end on a tempdir (no canonical ssb.db pollution)
    // -----------------------------------------------------------------------

    #[test]
    fn run_inner_on_valid_fixture_emits_curve_measured_and_writes_report() {
        let dir = tempfile::TempDir::new().expect("tempdir");
        let suite_path = dir.path().join("fixture.toml");
        fs::write(&suite_path, CONTEXT_FILL_FIXTURE_VALID).expect("write fixture");

        let result = run_inner(suite_path, 0, false, dir.path());
        result.expect("run_inner must succeed on valid fixture");

        // Report file landed under benchmarks/context_fill/.
        let report_dir = dir.path().join("benchmarks").join("context_fill");
        let mut found = false;
        if let Ok(entries) = fs::read_dir(&report_dir) {
            for entry in entries.flatten() {
                if entry
                    .file_name()
                    .to_string_lossy()
                    .starts_with("context-fill-test_context_fill_fixture-")
                {
                    found = true;
                    break;
                }
            }
        }
        assert!(found, "report file must land under benchmarks/context_fill/");
    }

    #[test]
    fn run_inner_rejects_underpowered_fixture() {
        let dir = tempfile::TempDir::new().expect("tempdir");
        let suite_path = dir.path().join("fixture.toml");
        fs::write(&suite_path, CONTEXT_FILL_FIXTURE_UNDERPOWER).expect("write fixture");

        let result = run_inner(suite_path, 0, false, dir.path());

        assert!(result.is_err(), "underpowered fixture must fail");
        match result.unwrap_err() {
            BenchContextFillError::FloorViolation { .. } => {}
            other => panic!("expected FloorViolation, got: {other}"),
        }
    }

    #[test]
    fn run_inner_on_shipped_fixture_succeeds() {
        let shipped = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("..")
            .join("..")
            .join("benchmarks")
            .join("context_fill")
            .join("context_fill_fixture_v1.toml");
        if !shipped.exists() {
            eprintln!("skipped: shipped fixture not found at {}", shipped.display());
            return;
        }
        let dir = tempfile::TempDir::new().expect("tempdir");
        let result = run_inner(shipped, 0, false, dir.path());
        result.expect("run_inner must succeed on shipped fixture");
    }
}
