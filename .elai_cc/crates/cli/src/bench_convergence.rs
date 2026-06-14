//! I-27 — `elai bench convergence --suite <PATH> [--seeds <u32>] [--seed <u64>] [--json]`
//! CLI driver.
//!
//! CONVERGENCE = verified-outcome-rate vs total-test-time compute curve per config,
//! plus a Pareto cost/accuracy view. The curve carries bootstrap 95% CI (paired seeds
//! >= 5, per I-38 statistical floor) against cumulative compute spend.
//!
//! # What this measures (honestly)
//!
//! For each compute level L in the fixture suite, the bench counts the fraction of
//! tasks that produced a verified outcome (W-1-grade: the fixture supplies a `verified`
//! bool per task per seed). It does NOT use pass@k (stub_ban). The Pareto view overlays
//! cost_per_verified_task = dollar_per_task / mean_rate. Monotonicity is MEASURED and
//! REPORTED — if the rate dips between adjacent levels, the report flags it as
//! "non-monotonic at [level]; investigate." Nothing is smoothed (GD-5).
//!
//! # Fixture tier (Tier-1)
//!
//! The fixture TOML supplies verified boolean outcomes per task per compute level.
//! No live inference runs. The live per-node compute-spend leg (I-4 SearchNodeEvent
//! rows) is PENDING-on-I-4 (search/ladder.rs UNBUILT at HEAD).
//!
//! # WIRE-DARK[I-27]: live SearchNodeEvent compute-spend axis
//!
//! The live per-node compute-spend resolution from I-4 SearchNodeEvent rows is
//! PENDING-on-I-4. When I-4 ships, the `compute_level` field will carry per-node
//! spend from SearchNodeEvent. The payload shape is forward-compatible
//! (compute_level is an opaque label). See wire-dark-allowlist.toml I-27 row.
//!
//! # Statistical floor (mechanically enforced)
//!
//!   - `seed_count >= 5`  (SEM meaningful at this N, I-38 floor)
//!   - `paired == true`   (same tasks across compute levels)
//!
//! # Wiring
//!
//! producer  : this file — reads ConvergenceConfig TOML, validates floor, sweeps
//!             compute levels, computes bootstrap CI, writes timestamped TOML report,
//!             emits ONE `ConvergenceCurveMeasured` SSB row.
//! ssb row   : `SsbActionType::ConvergenceCurveMeasured` (flat beacon).
//! entrypoint: `elai bench convergence --suite <PATH> [--seeds <u32>]`.
//!
//! # No `.unwrap()` / `.expect()` in non-test paths.
//! # blake3 only for all content hashing.

use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use elai_core::paths::sessions_dir_path;
use elai_orchestrator::ssb::{ConvergenceCurveMeasuredPayload, ConvergenceCurvePoint, SsbReader as _};
use elai_orchestrator::{Ssb, SsbConfig, SsbPayload};
use serde::{Deserialize, Serialize};
use thiserror::Error;

// ---------------------------------------------------------------------------
// Constants
// ---------------------------------------------------------------------------

/// Number of bootstrap resamples used for the 95% CI.
/// Mirrors bench_qdelta::BOOTSTRAP_N_RESAMPLES (500 at Tier-1).
const BOOTSTRAP_N_RESAMPLES: usize = 500;

/// Fraction for the lower tail of a two-sided 95% CI.
const CI_LOWER_TAIL: f64 = 0.025;

/// Fraction for the upper tail of a two-sided 95% CI.
const CI_UPPER_TAIL: f64 = 0.975;

/// Minimum seed count required by the I-38 statistical floor.
const FLOOR_MIN_SEED_COUNT: u32 = 5;

/// Marginal gain threshold below which the curve is considered past the "knee".
/// If rate[i] - rate[i-1] < KNEE_GAIN_THRESHOLD, the knee is at level i-1.
const KNEE_GAIN_THRESHOLD: f64 = 0.01;

/// Bench session id embedded in every SSB row this bench emits.
const BENCH_SESSION_ID: &str = "cli-bench-convergence";

/// Agent id label for SSB rows.
const BENCH_AGENT_ID: &str = "cli:bench-convergence";

// ---------------------------------------------------------------------------
// Config (read from TOML)
// ---------------------------------------------------------------------------

/// One task row at a single compute level.
/// `verified`: did this task produce a W-1-grade verified outcome at this level?
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ConvergenceTaskRow {
    /// True if the task produced a verified outcome at this compute level.
    pub verified: bool,
}

/// One compute level: a label + per-task verified outcomes across seeds.
///
/// `tasks` has `seed_count` rows per task (tasks[seed_idx][task_idx] = verified).
/// For Tier-1 fixtures, tasks is a flat Vec<bool> of length seed_count * task_count;
/// the layout is seed-major (tasks 0..N for seed 0, then 0..N for seed 1, etc.).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ConvergenceLevel {
    /// Compute level label (e.g. "1x", "2x", "4x", "8x").
    pub label: String,
    /// Cost per task in USD at this compute level (E-49 $/CTS denominator).
    /// None if cost data absent (Pareto view shows None).
    pub dollar_per_task: Option<f64>,
    /// Per-seed, per-task verified outcomes. Outer vec = seeds, inner vec = tasks.
    /// All seeds must have the same task count.
    pub outcomes: Vec<Vec<bool>>,
}

/// Convergence bench configuration (read from TOML at bench invocation time).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ConvergenceConfig {
    /// Human-readable suite label (e.g. "convergence_fixture_v1").
    pub suite: String,
    /// Number of independent seeds (floor >= 5, I-38).
    pub seed_count: u32,
    /// `true` = same tasks appear across all compute levels (paired design).
    pub paired: bool,
    /// Ordered compute levels, from lowest to highest compute spend.
    pub levels: Vec<ConvergenceLevel>,
}

// ---------------------------------------------------------------------------
// Error type
// ---------------------------------------------------------------------------

#[derive(Debug, Error)]
pub enum BenchConvergenceError {
    /// Statistical floor violated: run config rejected before any compute.
    #[error(
        "convergence floor violation: seed_count={seed_count} (min {FLOOR_MIN_SEED_COUNT}), \
         paired={paired} (must be true)"
    )]
    FloorViolation { seed_count: u32, paired: bool },
    /// The fixture has fewer than 2 compute levels.
    #[error(
        "convergence fixture has {level_count} compute level(s); need >= 2 for a curve"
    )]
    TooFewLevels { level_count: usize },
    /// A compute level has no tasks.
    #[error("compute level '{label}' has 0 tasks; each level must have >= 1 task")]
    EmptyLevel { label: String },
    /// Seed count in a level does not match config.seed_count.
    #[error(
        "compute level '{label}': expected {expected} seeds, got {actual}"
    )]
    SeedMismatch { label: String, expected: u32, actual: usize },
    /// Config file could not be read.
    #[error("convergence could not read config at {path}: {source_msg}")]
    ConfigRead { path: String, source_msg: String },
    /// TOML deserialization failure.
    #[error("convergence could not parse config TOML: {source_msg}")]
    ConfigParse { source_msg: String },
    /// Could not determine the current working directory.
    #[error("convergence could not determine cwd: {source_msg}")]
    Cwd { source_msg: String },
    /// Could not create the on-disk report directory.
    #[error("convergence could not create dir {path}: {source_msg}")]
    ReportDir { path: String, source_msg: String },
    /// Generic I/O failure writing the TOML report.
    #[error("convergence could not write {path}: {source_msg}")]
    OutputIo { path: String, source_msg: String },
    /// TOML serialisation failure.
    #[error("convergence could not serialise report: {source_msg}")]
    Serialise { source_msg: String },
    /// Building the async runtime failed.
    #[error("convergence could not build async runtime: {source_msg}")]
    Runtime { source_msg: String },
    /// Opening the SSB single-writer failed.
    #[error("convergence could not open SSB writer: {source_msg}")]
    SsbOpen { source_msg: String },
    /// Writing the ConvergenceCurveMeasured beacon failed.
    #[error("convergence could not write SSB beacon: {source_msg}")]
    SsbWrite { source_msg: String },
}

// ---------------------------------------------------------------------------
// Statistical floor enforcement
// ---------------------------------------------------------------------------

/// Validate the I-38 statistical floor invariants.
/// Returns `Err(FloorViolation)` if `seed_count < 5` OR `paired == false`.
pub fn floor_check(config: &ConvergenceConfig) -> Result<(), BenchConvergenceError> {
    if config.seed_count < FLOOR_MIN_SEED_COUNT || !config.paired {
        return Err(BenchConvergenceError::FloorViolation {
            seed_count: config.seed_count,
            paired: config.paired,
        });
    }
    Ok(())
}

/// Validate the fixture structure: >= 2 levels, each level has >= 1 task and
/// the correct number of seeds.
pub fn validate_fixture(config: &ConvergenceConfig) -> Result<(), BenchConvergenceError> {
    if config.levels.len() < 2 {
        return Err(BenchConvergenceError::TooFewLevels {
            level_count: config.levels.len(),
        });
    }
    for level in &config.levels {
        if level.outcomes.is_empty() {
            return Err(BenchConvergenceError::EmptyLevel {
                label: level.label.clone(),
            });
        }
        if level.outcomes.len() != config.seed_count as usize {
            return Err(BenchConvergenceError::SeedMismatch {
                label: level.label.clone(),
                expected: config.seed_count,
                actual: level.outcomes.len(),
            });
        }
        if level.outcomes[0].is_empty() {
            return Err(BenchConvergenceError::EmptyLevel {
                label: level.label.clone(),
            });
        }
    }
    Ok(())
}

// ---------------------------------------------------------------------------
// Bootstrap CI (verified-outcome-rate variant)
// ---------------------------------------------------------------------------

/// Seeded bootstrap 95% CI on the mean verified-outcome rate.
///
/// `outcomes_per_seed` is a slice of per-seed task-outcome vecs. Each inner
/// vec has `n_tasks` bool values (true = verified outcome). For each resample,
/// draws `n_tasks` index positions with replacement and computes:
///
///   resample_rate = sum(task_verified[idx]) / n_tasks
///
/// across all seeds, then averages. Returns `(ci_low, ci_high)` as the 2.5th
/// and 97.5th percentiles of the bootstrap distribution.
///
/// Mirrors `bench_qdelta::bootstrap_ci` (xorshift64, no dep, GD-2).
fn bootstrap_ci(
    outcomes_per_seed: &[Vec<bool>],
    seed: u64,
    n_resamples: usize,
) -> (f64, f64) {
    let n_seeds = outcomes_per_seed.len();
    if n_seeds == 0 {
        return (0.0, 0.0);
    }
    let n_tasks = outcomes_per_seed[0].len();
    if n_tasks == 0 {
        return (0.0, 0.0);
    }

    let mut rng_state = if seed == 0 { 1u64 } else { seed };
    let mut mean_rates: Vec<f64> = Vec::with_capacity(n_resamples);

    for _ in 0..n_resamples {
        let mut total_verified: u64 = 0;
        for seed_outcomes in outcomes_per_seed {
            let mut verified_count: u64 = 0;
            for _ in 0..n_tasks {
                rng_state = xorshift64(rng_state);
                let idx = (rng_state % n_tasks as u64) as usize;
                if seed_outcomes[idx] {
                    verified_count += 1;
                }
            }
            total_verified += verified_count;
        }
        let mean_rate = total_verified as f64 / (n_seeds * n_tasks) as f64;
        mean_rates.push(mean_rate.clamp(0.0, 1.0));
    }

    mean_rates.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    let lo_idx = (CI_LOWER_TAIL * n_resamples as f64) as usize;
    let hi_idx = ((CI_UPPER_TAIL * n_resamples as f64) as usize).min(n_resamples - 1);

    (mean_rates[lo_idx], mean_rates[hi_idx])
}

/// Minimal xorshift64 PRNG — reproducible, no-dep. Seed 0 mapped to 1.
/// Mirrors bench_qdelta::xorshift64 (same algorithm for GD-2 seed-recomputable CI).
#[inline]
fn xorshift64(mut state: u64) -> u64 {
    state ^= state << 13;
    state ^= state >> 7;
    state ^= state << 17;
    state
}

// ---------------------------------------------------------------------------
// Curve computation
// ---------------------------------------------------------------------------

/// Compute the mean verified-outcome rate for a single compute level.
fn mean_rate_for_level(level: &ConvergenceLevel) -> f64 {
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

/// Detect the knee: the first level where the marginal gain collapses.
/// If no knee found, returns the last level's label.
fn detect_knee(curve_points: &[ConvergenceCurvePoint]) -> String {
    if curve_points.is_empty() {
        return String::new();
    }
    for i in 1..curve_points.len() {
        let gain = curve_points[i].mean_rate - curve_points[i - 1].mean_rate;
        if gain < KNEE_GAIN_THRESHOLD {
            return curve_points[i - 1].compute_level.clone();
        }
    }
    curve_points.last().map(|p| p.compute_level.clone()).unwrap_or_default()
}

/// Detect non-monotonic levels (where rate dips below prior level).
fn detect_non_monotonic(curve_points: &[ConvergenceCurvePoint]) -> Vec<String> {
    let mut non_monotonic = Vec::new();
    for i in 1..curve_points.len() {
        if curve_points[i].mean_rate < curve_points[i - 1].mean_rate {
            non_monotonic.push(curve_points[i].compute_level.clone());
        }
    }
    non_monotonic
}

// ---------------------------------------------------------------------------
// CLI entry
// ---------------------------------------------------------------------------

/// CLI entry — validates floor, sweeps compute levels, computes CI, writes a
/// timestamped TOML report, emits one `ConvergenceCurveMeasured` SSB beacon.
///
/// Returns `Ok(())` on a clean run (rc=0). On a handled error it prints a
/// `CONVERGENCE ABORT:` line to stderr and calls `std::process::exit(1)`.
pub(crate) fn run(suite: PathBuf, seeds: u32, json: bool) -> anyhow::Result<()> {
    match run_inner(suite, seeds, json) {
        Ok(()) => Ok(()),
        Err(e) => {
            eprintln!("CONVERGENCE ABORT: {e}");
            std::process::exit(1);
        }
    }
}

/// Core — validates floor, sweeps compute levels, computes curve + CI, writes TOML,
/// emits SSB beacon. Public so integration/contract tests can drive without process exit.
pub fn run_inner(suite_path: PathBuf, seeds: u32, json: bool) -> Result<(), BenchConvergenceError> {
    // 1. Read and parse the TOML config.
    let config_bytes = fs::read(&suite_path).map_err(|e| BenchConvergenceError::ConfigRead {
        path: suite_path.display().to_string(),
        source_msg: e.to_string(),
    })?;
    let mut config: ConvergenceConfig =
        toml::from_str(&String::from_utf8_lossy(&config_bytes)).map_err(|e| {
            BenchConvergenceError::ConfigParse {
                source_msg: e.to_string(),
            }
        })?;

    // 2. Override seed_count if --seeds was provided (> 0 means user specified).
    if seeds > 0 {
        config.seed_count = seeds;
    }

    // 3. Statistical floor check — exits nonzero before any compute.
    floor_check(&config)?;

    // 4. Fixture structure check.
    validate_fixture(&config)?;

    // 5. blake3 hash of config bytes as corpus provenance (anti-gaming, GD-2).
    let fixture_blake3 = blake3::hash(&config_bytes).to_hex().to_string();

    // 6. Default seed for bootstrap CI (mirrors bench_qdelta: 42).
    let ci_seed: u64 = 42;

    // 7. Sweep compute levels and build curve_points.
    let mut curve_points: Vec<ConvergenceCurvePoint> = Vec::with_capacity(config.levels.len());
    for level in &config.levels {
        let mean_rate = mean_rate_for_level(level);
        let (ci_low, ci_high) = bootstrap_ci(&level.outcomes, ci_seed, BOOTSTRAP_N_RESAMPLES);

        let cost_per_verified_task = level.dollar_per_task.and_then(|dpt| {
            if mean_rate > 0.0 { Some(dpt / mean_rate) } else { None }
        });

        curve_points.push(ConvergenceCurvePoint {
            compute_level: level.label.clone(),
            mean_rate,
            ci_low,
            ci_high,
            cost_per_verified_task,
        });
    }

    // 8. Knee detection and monotonicity check.
    let knee_compute_level = detect_knee(&curve_points);
    let non_monotonic_at = detect_non_monotonic(&curve_points);

    // 9. Wall-clock timestamp.
    let ts_ms = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis() as i64)
        .unwrap_or_default();

    // 10. Build the SSB payload.
    let payload = ConvergenceCurveMeasuredPayload {
        fixture_suite: config.suite.clone(),
        seed_count: config.seed_count,
        task_count: config.levels.first().map(|l| l.outcomes.first().map(|s| s.len()).unwrap_or(0)).unwrap_or(0) as u32,
        curve_points: curve_points.clone(),
        knee_compute_level: knee_compute_level.clone(),
        non_monotonic_at: non_monotonic_at.clone(),
        seed: ci_seed,
        fixture_blake3: fixture_blake3.clone(),
        runpod_pending: true,
        ts_ms,
    };

    // 11. Write timestamped TOML report under <cwd>/benchmarks/convergence/.
    let project_root = std::env::current_dir().map_err(|e| BenchConvergenceError::Cwd {
        source_msg: e.to_string(),
    })?;
    let report_path = write_toml_report(&project_root, &config.suite, &payload, ts_ms)?;

    // 12. Emit ONE ConvergenceCurveMeasured SSB beacon (durable, awaited).
    emit_beacon(&project_root, &payload)?;

    // 13. Summary line.
    let monotonicity_note = if non_monotonic_at.is_empty() {
        "monotonic".to_string()
    } else {
        format!("non-monotonic at: {}", non_monotonic_at.join(", "))
    };
    println!(
        "CONVERGENCE suite={} seed_count={} n_levels={} knee={} {} \
         fixture_blake3={} runpod_pending=true report={}",
        config.suite,
        config.seed_count,
        curve_points.len(),
        knee_compute_level,
        monotonicity_note,
        &fixture_blake3[..16],
        report_path.display(),
    );
    if json {
        let body = serde_json::to_string_pretty(&payload).map_err(|e| BenchConvergenceError::Serialise {
            source_msg: e.to_string(),
        })?;
        println!("{body}");
    }
    Ok(())
}

// ---------------------------------------------------------------------------
// TOML report writer
// ---------------------------------------------------------------------------

/// Write the convergence report to `<project_root>/benchmarks/convergence/convergence-<unix_ms>.toml`.
fn write_toml_report(
    project_root: &Path,
    suite: &str,
    payload: &ConvergenceCurveMeasuredPayload,
    ts_ms: i64,
) -> Result<PathBuf, BenchConvergenceError> {
    let report_dir = project_root.join("benchmarks").join("convergence");
    fs::create_dir_all(&report_dir).map_err(|e| BenchConvergenceError::ReportDir {
        path: report_dir.display().to_string(),
        source_msg: e.to_string(),
    })?;

    let safe_suite = suite.replace(|c: char| !c.is_alphanumeric() && c != '_' && c != '-', "_");
    let report_path = report_dir.join(format!("convergence-{safe_suite}-{ts_ms}.toml"));

    // Build a human-readable TOML body.
    let mut body = format!(
        "# I-27 convergence curve report\n\
         suite = {suite:?}\n\
         seed_count = {seed_count}\n\
         task_count = {task_count}\n\
         knee_compute_level = {knee:?}\n\
         fixture_blake3 = {blake3:?}\n\
         runpod_pending = true\n\
         ts_ms = {ts}\n\n\
         non_monotonic_at = {non_mon:?}\n\n",
        suite = payload.fixture_suite,
        seed_count = payload.seed_count,
        task_count = payload.task_count,
        knee = payload.knee_compute_level,
        blake3 = payload.fixture_blake3,
        ts = payload.ts_ms,
        non_mon = payload.non_monotonic_at,
    );

    for (i, pt) in payload.curve_points.iter().enumerate() {
        body.push_str(&format!(
            "[[curve_points]]\ncompute_level = {:?}\nmean_rate = {:.6}\n\
             ci_low = {:.6}\nci_high = {:.6}\n",
            pt.compute_level, pt.mean_rate, pt.ci_low, pt.ci_high,
        ));
        if let Some(cost) = pt.cost_per_verified_task {
            body.push_str(&format!("cost_per_verified_task = {cost:.6}\n"));
        }
        if i + 1 < payload.curve_points.len() {
            body.push('\n');
        }
    }

    fs::write(&report_path, &body).map_err(|e| BenchConvergenceError::OutputIo {
        path: report_path.display().to_string(),
        source_msg: e.to_string(),
    })?;
    Ok(report_path)
}

// ---------------------------------------------------------------------------
// SSB emit
// ---------------------------------------------------------------------------

/// Open the canonical session `ssb.db`, emit ONE `ConvergenceCurveMeasured` beacon
/// (durable, awaited). Mirrors bench_qdelta::emit_beacon.
fn emit_beacon(
    project_root: &Path,
    payload: &ConvergenceCurveMeasuredPayload,
) -> Result<(), BenchConvergenceError> {
    let db_path = sessions_dir_path(project_root).join("ssb.db");
    if let Some(parent) = db_path.parent() {
        fs::create_dir_all(parent).map_err(|e| BenchConvergenceError::ReportDir {
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
    let clock: std::sync::Arc<dyn elai_core::clock::ClockProvider> =
        std::sync::Arc::new(elai_core::OsClock);

    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .map_err(|e| BenchConvergenceError::Runtime {
            source_msg: e.to_string(),
        })?;

    let payload_clone = payload.clone();
    rt.block_on(async move {
        let ssb = Ssb::new(cfg, clock).map_err(|e| BenchConvergenceError::SsbOpen {
            source_msg: e.to_string(),
        })?;

        ssb.write_payload(
            BENCH_AGENT_ID,
            0,
            SsbPayload::ConvergenceCurveMeasured(payload_clone),
            Vec::new(),
        )
        .await
        .map_err(|e| BenchConvergenceError::SsbWrite {
            source_msg: e.to_string(),
        })?;

        tracing::info!("convergence: I-27 ConvergenceCurveMeasured SSB row emitted");

        Ok::<(), BenchConvergenceError>(())
    })
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    // -----------------------------------------------------------------------
    // Inline fixture constants (Tier-1 — no separate fixture files required).
    // -----------------------------------------------------------------------

    /// Valid fixture: 3 compute levels, 5 seeds each, 4 tasks per seed.
    /// Level "1x": 2/4 tasks verified per seed -> mean_rate = 0.50
    /// Level "2x": 3/4 tasks verified per seed -> mean_rate = 0.75
    /// Level "4x": 4/4 tasks verified per seed -> mean_rate = 1.00
    /// Monotonically increasing; knee at "4x" (last level).
    const CONVERGENCE_FIXTURE_VALID: &str = r#"
suite = "test_convergence_fixture"
seed_count = 5
paired = true

[[levels]]
label = "1x"
dollar_per_task = 0.01
outcomes = [
  [true, false, true, false],
  [true, false, true, false],
  [true, false, true, false],
  [true, false, true, false],
  [true, false, true, false]
]

[[levels]]
label = "2x"
dollar_per_task = 0.02
outcomes = [
  [true, true, true, false],
  [true, true, true, false],
  [true, true, true, false],
  [true, true, true, false],
  [true, true, true, false]
]

[[levels]]
label = "4x"
dollar_per_task = 0.04
outcomes = [
  [true, true, true, true],
  [true, true, true, true],
  [true, true, true, true],
  [true, true, true, true],
  [true, true, true, true]
]
"#;

    /// Under-powered fixture (seed_count=3, paired=false).
    /// floor_check() MUST return Err(FloorViolation).
    const CONVERGENCE_FIXTURE_UNDERPOWER: &str = r#"
suite = "test_convergence_underpower"
seed_count = 3
paired = false

[[levels]]
label = "1x"
outcomes = [[true, false], [true, false], [false, false]]

[[levels]]
label = "2x"
outcomes = [[true, true], [true, true], [true, false]]
"#;

    /// Non-monotonic fixture: level "2x" has a lower rate than "1x".
    /// Exposes GD-5 honesty: the bench MUST flag this, not smooth it.
    const CONVERGENCE_FIXTURE_NON_MONOTONIC: &str = r#"
suite = "test_convergence_non_monotonic"
seed_count = 5
paired = true

[[levels]]
label = "1x"
outcomes = [
  [true, true, true, false],
  [true, true, true, false],
  [true, true, true, false],
  [true, true, true, false],
  [true, true, true, false]
]

[[levels]]
label = "2x"
outcomes = [
  [true, false, false, false],
  [true, false, false, false],
  [true, false, false, false],
  [true, false, false, false],
  [true, false, false, false]
]

[[levels]]
label = "4x"
outcomes = [
  [true, true, true, true],
  [true, true, true, true],
  [true, true, true, true],
  [true, true, true, true],
  [true, true, true, true]
]
"#;

    fn parse_fixture(s: &str) -> ConvergenceConfig {
        toml::from_str(s).expect("fixture parse")
    }

    // -----------------------------------------------------------------------
    // Floor check tests
    // -----------------------------------------------------------------------

    #[test]
    fn floor_check_passes_valid_config() {
        let config = parse_fixture(CONVERGENCE_FIXTURE_VALID);
        assert!(floor_check(&config).is_ok(), "valid config must pass floor_check");
    }

    #[test]
    fn floor_check_rejects_underpower() {
        let config = parse_fixture(CONVERGENCE_FIXTURE_UNDERPOWER);
        let result = floor_check(&config);
        assert!(result.is_err(), "under-powered config must fail floor_check");
        match result.unwrap_err() {
            BenchConvergenceError::FloorViolation { seed_count, paired } => {
                assert_eq!(seed_count, 3);
                assert!(!paired);
            }
            other => panic!("expected FloorViolation, got: {other}"),
        }
    }

    #[test]
    fn floor_check_rejects_seed_count_below_5() {
        let mut config = parse_fixture(CONVERGENCE_FIXTURE_VALID);
        config.seed_count = 4;
        assert!(floor_check(&config).is_err(), "seed_count=4 must fail floor_check");
    }

    #[test]
    fn floor_check_rejects_unpaired() {
        let mut config = parse_fixture(CONVERGENCE_FIXTURE_VALID);
        config.paired = false;
        assert!(floor_check(&config).is_err(), "paired=false must fail floor_check");
    }

    // -----------------------------------------------------------------------
    // Mean rate computation
    // -----------------------------------------------------------------------

    #[test]
    fn mean_rate_correct_for_half_verified() {
        let config = parse_fixture(CONVERGENCE_FIXTURE_VALID);
        let level_1x = &config.levels[0];
        let rate = mean_rate_for_level(level_1x);
        // 2/4 tasks verified per seed, 5 seeds: total 10/20 = 0.50
        assert!(
            (rate - 0.5_f64).abs() < 1e-9,
            "1x level mean_rate must be 0.50; got {rate}"
        );
    }

    #[test]
    fn mean_rate_correct_for_all_verified() {
        let config = parse_fixture(CONVERGENCE_FIXTURE_VALID);
        let level_4x = &config.levels[2];
        let rate = mean_rate_for_level(level_4x);
        // 4/4 tasks verified per seed, 5 seeds: total 20/20 = 1.00
        assert!(
            (rate - 1.0_f64).abs() < 1e-9,
            "4x level mean_rate must be 1.00; got {rate}"
        );
    }

    // -----------------------------------------------------------------------
    // Bootstrap CI tests
    // -----------------------------------------------------------------------

    #[test]
    fn bootstrap_ci_deterministic_under_same_seed() {
        let config = parse_fixture(CONVERGENCE_FIXTURE_VALID);
        let level = &config.levels[0];
        let (lo1, hi1) = bootstrap_ci(&level.outcomes, 42, BOOTSTRAP_N_RESAMPLES);
        let (lo2, hi2) = bootstrap_ci(&level.outcomes, 42, BOOTSTRAP_N_RESAMPLES);
        assert_eq!(lo1, lo2, "bootstrap CI must be deterministic under same seed");
        assert_eq!(hi1, hi2, "bootstrap CI must be deterministic under same seed");
        assert!(hi1 > lo1, "CI must be non-trivial (hi > lo)");
    }

    #[test]
    fn bootstrap_ci_different_seeds_differ() {
        let config = parse_fixture(CONVERGENCE_FIXTURE_VALID);
        let level = &config.levels[0];
        let (lo1, hi1) = bootstrap_ci(&level.outcomes, 1, BOOTSTRAP_N_RESAMPLES);
        let (lo2, hi2) = bootstrap_ci(&level.outcomes, 9999, BOOTSTRAP_N_RESAMPLES);
        let are_different = (lo1 - lo2).abs() > 1e-9 || (hi1 - hi2).abs() > 1e-9;
        assert!(are_different, "different seeds must produce different CI values");
    }

    // -----------------------------------------------------------------------
    // Knee detection
    // -----------------------------------------------------------------------

    #[test]
    fn knee_detected_at_last_level_for_monotonic_fixture() {
        let config = parse_fixture(CONVERGENCE_FIXTURE_VALID);
        let points: Vec<ConvergenceCurvePoint> = config.levels.iter().map(|l| {
            let rate = mean_rate_for_level(l);
            let (ci_low, ci_high) = bootstrap_ci(&l.outcomes, 42, BOOTSTRAP_N_RESAMPLES);
            ConvergenceCurvePoint {
                compute_level: l.label.clone(),
                mean_rate: rate,
                ci_low,
                ci_high,
                cost_per_verified_task: None,
            }
        }).collect();
        let knee = detect_knee(&points);
        // Gains: 2x-1x = 0.25, 4x-2x = 0.25; both >= KNEE_GAIN_THRESHOLD (0.01)
        // so no collapse found; knee = last level = "4x".
        assert_eq!(knee, "4x", "knee must be last level for monotonic fixture");
    }

    // -----------------------------------------------------------------------
    // Monotonicity detection (GD-5 honesty)
    // -----------------------------------------------------------------------

    #[test]
    fn non_monotonic_detected_correctly() {
        let config = parse_fixture(CONVERGENCE_FIXTURE_NON_MONOTONIC);
        let points: Vec<ConvergenceCurvePoint> = config.levels.iter().map(|l| {
            ConvergenceCurvePoint {
                compute_level: l.label.clone(),
                mean_rate: mean_rate_for_level(l),
                ci_low: 0.0,
                ci_high: 0.0,
                cost_per_verified_task: None,
            }
        }).collect();
        let non_mon = detect_non_monotonic(&points);
        // "2x" dips from 0.75 (1x) to 0.25 (2x)
        assert!(
            non_mon.contains(&"2x".to_string()),
            "non_monotonic_at must contain '2x'; got: {non_mon:?}"
        );
    }

    #[test]
    fn monotonic_fixture_has_empty_non_monotonic_list() {
        let config = parse_fixture(CONVERGENCE_FIXTURE_VALID);
        let points: Vec<ConvergenceCurvePoint> = config.levels.iter().map(|l| {
            ConvergenceCurvePoint {
                compute_level: l.label.clone(),
                mean_rate: mean_rate_for_level(l),
                ci_low: 0.0,
                ci_high: 0.0,
                cost_per_verified_task: None,
            }
        }).collect();
        let non_mon = detect_non_monotonic(&points);
        assert!(non_mon.is_empty(), "monotonic fixture must produce empty non_monotonic_at");
    }
}
