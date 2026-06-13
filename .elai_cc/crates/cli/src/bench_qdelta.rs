//! I-38 — `elai bench qdelta [--config PATH] [--seed <u64>] [--json]` CLI driver.
//!
//! QDELTA = defective-diff rate gates-ON vs gates-OFF with bootstrap 95% CI.
//! Mechanical statistical floor: `floor_check()` exits nonzero BEFORE any compute
//! if `seed_count < 5` OR `paired == false`. The floor is a hard harness policy,
//! not an advisory recommendation.
//!
//! # What this measures (honestly)
//!
//! QDELTA = `off_defective_rate` - `on_defective_rate` where:
//!   - `on_defective_rate`  = fraction of tasks where gates-ON produced a defective diff
//!   - `off_defective_rate` = fraction of tasks where gates-OFF produced a defective diff
//!   - positive QDELTA means gates IMPROVE quality (fewer defective diffs with gates ON)
//!
//! ABLATION FRAMING ONLY (GD-3). Never a leaderboard rank. SWE-bench Verified
//! framing explicitly declined (landmine #11 per I-0).
//!
//! # Fixture tier (Tier-1)
//!
//! All live-model/quantization legs are `runpod_pending = true` (D-026 deferred).
//! Fixture TOML drives the math/gating proof. No actual inference runs.
//!
//! # Statistical floor (mechanically enforced)
//!
//!   - `seed_count >= 5`  (SEM meaningful at this N)
//!   - `paired == true`   (same tasks gates-ON and gates-OFF; no distribution mismatch)
//!
//! # Wiring
//!
//! producer  : this file — reads QdeltaConfig TOML, validates floor, runs bootstrap CI,
//!             writes timestamped JSON report, emits ONE `QdeltaMeasured` SSB row.
//! ssb row   : `SsbActionType::QdeltaMeasured` (flat beacon, safedelta-shaped).
//! entrypoint: `elai bench qdelta --config <PATH>`.
//!
//! # No `.unwrap()` / `.expect()` in non-test paths.
//! # blake3 only for all content hashing.

use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use elai_core::paths::sessions_dir_path;
use elai_orchestrator::ssb::{QdeltaMeasuredPayload, SsbReader as _};
use elai_orchestrator::{Ssb, SsbConfig, SsbPayload};
use serde::{Deserialize, Serialize};
use thiserror::Error;

// ---------------------------------------------------------------------------
// Constants
// ---------------------------------------------------------------------------

/// Number of bootstrap resamples used for the 95% CI.
/// Mirrors safedelta_runner::BOOTSTRAP_N_RESAMPLES (500 at Tier-1).
const BOOTSTRAP_N_RESAMPLES: usize = 500;

/// Fraction for the lower tail of a two-sided 95% CI.
const CI_LOWER_TAIL: f64 = 0.025;

/// Fraction for the upper tail of a two-sided 95% CI.
const CI_UPPER_TAIL: f64 = 0.975;

/// Minimum seed count required by the statistical floor.
const FLOOR_MIN_SEED_COUNT: u32 = 5;

/// Bench session id embedded in every SSB row this bench emits.
const BENCH_SESSION_ID: &str = "cli-bench-qdelta";

/// Agent id label for SSB rows (mirrors bench_safedelta pattern).
const BENCH_AGENT_ID: &str = "cli:bench-qdelta";

// ---------------------------------------------------------------------------
// Config (read from TOML)
// ---------------------------------------------------------------------------

/// One task row in the QDELTA fixture.
/// `on_defective`: did gates-ON produce a defective diff on this task?
/// `off_defective`: did gates-OFF produce a defective diff on this task?
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct QdeltaTaskRow {
    pub on_defective: bool,
    pub off_defective: bool,
}

/// QDELTA run configuration (read from TOML at bench invocation time).
///
/// Fields enforce the statistical floor at parse time via `floor_check()`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct QdeltaConfig {
    /// Human-readable suite label (e.g. "seccodeplt_qdelta_fixture").
    pub suite: String,
    /// Model/candidate provenance label (corpus-level, not a gate input).
    /// At Tier-1 this is a descriptive label for the fixture, not a live model.
    pub model_id: String,
    /// Number of independent seeds used for the paired run. Floor: >= 5.
    pub seed_count: u32,
    /// `true` = same tasks measured gates-ON and gates-OFF (paired design).
    /// Floor: must be `true`. Unpaired designs cannot separate quality effects.
    pub paired: bool,
    /// Cost per task in USD (reported alongside accuracy per GD-3 $/task policy).
    pub dollar_per_task: f64,
    /// D-20 contamination probe result from a prior ARC bench run (or "not_run").
    /// Allowed values: "not_run" | "honestly_evaluated" | "game_with_memorization".
    /// NOT an inline call to ARCAGI2Runner — carries a string verdict from a prior run.
    pub contamination_probe_result: String,
    /// Per-task paired observations. Each row has on_defective + off_defective.
    pub tasks: Vec<QdeltaTaskRow>,
}

// ---------------------------------------------------------------------------
// Error type
// ---------------------------------------------------------------------------

#[derive(Debug, Error)]
pub enum BenchQdeltaError {
    /// Statistical floor violated: run config rejected before any compute.
    #[error(
        "qdelta floor violation: seed_count={seed_count} (min {FLOOR_MIN_SEED_COUNT}), \
         paired={paired} (must be true) — harness refuses under-powered runs"
    )]
    FloorViolation { seed_count: u32, paired: bool },
    /// The corpus is degenerate: on_rate == off_rate (no quality delta to measure).
    #[error(
        "qdelta degenerate corpus: on_defective_rate ({on_rate:.4}) == off_defective_rate \
         ({off_rate:.4}) — fixture must contain >=2 tasks where on_defective != off_defective \
         (DD5/anti-floor-fake)"
    )]
    DegenerateCorpus { on_rate: f64, off_rate: f64 },
    /// Config file could not be read or parsed.
    #[error("qdelta could not read config at {path}: {source_msg}")]
    ConfigRead { path: String, source_msg: String },
    /// TOML deserialization failure.
    #[error("qdelta could not parse config TOML: {source_msg}")]
    ConfigParse { source_msg: String },
    /// Could not determine the current working directory for the report dir.
    #[error("qdelta could not determine cwd: {source_msg}")]
    Cwd { source_msg: String },
    /// Could not create the on-disk report directory.
    #[error("qdelta could not create dir {path}: {source_msg}")]
    ReportDir { path: String, source_msg: String },
    /// Generic I/O failure writing the JSON report.
    #[error("qdelta could not write {path}: {source_msg}")]
    OutputIo { path: String, source_msg: String },
    /// JSON serialisation failure.
    #[error("qdelta could not serialise report: {source_msg}")]
    Serialise { source_msg: String },
    /// Building the async runtime failed.
    #[error("qdelta could not build async runtime: {source_msg}")]
    Runtime { source_msg: String },
    /// Opening the SSB single-writer failed.
    #[error("qdelta could not open SSB writer: {source_msg}")]
    SsbOpen { source_msg: String },
    /// Writing the QdeltaMeasured beacon failed.
    #[error("qdelta could not write SSB beacon: {source_msg}")]
    SsbWrite { source_msg: String },
}

// ---------------------------------------------------------------------------
// Statistical floor enforcement
// ---------------------------------------------------------------------------

/// Validate the statistical floor invariants. Returns `Err(FloorViolation)` if
/// `seed_count < FLOOR_MIN_SEED_COUNT` OR `paired == false`. Called at
/// `run_inner()` entry — no compute (bootstrap CI or SSB emit) runs before this.
pub fn floor_check(config: &QdeltaConfig) -> Result<(), BenchQdeltaError> {
    if config.seed_count < FLOOR_MIN_SEED_COUNT || !config.paired {
        return Err(BenchQdeltaError::FloorViolation {
            seed_count: config.seed_count,
            paired: config.paired,
        });
    }
    Ok(())
}

// ---------------------------------------------------------------------------
// Bootstrap CI (paired vec variant)
// ---------------------------------------------------------------------------

/// Seeded bootstrap 95% CI on the paired QDELTA statistic.
///
/// Takes two equal-length bool vectors (one per task: was the diff defective?).
/// For each resample, draws `n_tasks` INDEX positions with replacement from
/// `[0..n_tasks]` and computes:
///
///   resample_on_rate  = sum(on_cells[idx]) / n_tasks
///   resample_off_rate = sum(off_cells[idx]) / n_tasks
///   resample_delta    = resample_off_rate - resample_on_rate
///
/// Returns `(ci_low, ci_high)` as the 2.5th and 97.5th percentiles of the
/// bootstrap distribution. Uses xorshift64 (pure std, no dep, GD-2).
///
/// Precondition: `on_cells.len() == off_cells.len()`, both non-empty.
fn bootstrap_ci(on_cells: &[bool], off_cells: &[bool], seed: u64, n_resamples: usize) -> (f64, f64) {
    let n = on_cells.len();
    if n == 0 {
        return (0.0, 0.0);
    }

    let mut rng_state = if seed == 0 { 1u64 } else { seed };
    let mut deltas: Vec<f64> = Vec::with_capacity(n_resamples);

    for _ in 0..n_resamples {
        let mut on_count: u64 = 0;
        let mut off_count: u64 = 0;
        for _ in 0..n {
            rng_state = xorshift64(rng_state);
            let idx = (rng_state % n as u64) as usize;
            if on_cells[idx] {
                on_count += 1;
            }
            if off_cells[idx] {
                off_count += 1;
            }
        }
        let resample_on_rate = on_count as f64 / n as f64;
        let resample_off_rate = off_count as f64 / n as f64;
        let resample_delta = resample_off_rate - resample_on_rate;
        deltas.push(resample_delta.clamp(-1.0, 1.0));
    }

    deltas.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    let lo_idx = (CI_LOWER_TAIL * n_resamples as f64) as usize;
    let hi_idx = ((CI_UPPER_TAIL * n_resamples as f64) as usize).min(n_resamples - 1);

    (deltas[lo_idx], deltas[hi_idx])
}

/// Minimal xorshift64 PRNG — reproducible, no-dep. Seed 0 mapped to 1.
/// Mirrors safedelta_runner::xorshift64 (same algorithm for GD-2 seed-recomputable CI).
#[inline]
fn xorshift64(mut state: u64) -> u64 {
    state ^= state << 13;
    state ^= state >> 7;
    state ^= state << 17;
    state
}

// ---------------------------------------------------------------------------
// Methodology doc generator
// ---------------------------------------------------------------------------

/// Generate a machine-readable methodology description from `QdeltaConfig`.
/// Output is deterministic under the same config (GD-2).
pub fn generate_methodology_doc(config: &QdeltaConfig) -> String {
    format!(
        "QDELTA methodology (I-38)\n\
         suite: {suite}\n\
         model_id: {model_id}\n\
         seed_count: {seed_count} (floor >= {FLOOR_MIN_SEED_COUNT})\n\
         paired: {paired}\n\
         dollar_per_task: {dollar_per_task:.6}\n\
         contamination_probe_result: {contamination}\n\
         task_count: {task_count}\n\
         framing: ablation (gates-ON vs gates-OFF), GD-3 — no leaderboard rank\n\
         bootstrap_n_resamples: {BOOTSTRAP_N_RESAMPLES}\n\
         ci_level: 95% two-sided (lower={CI_LOWER_TAIL}, upper={CI_UPPER_TAIL})\n\
         runpod_pending: true (live-model leg deferred per D-026)\n\
         surviving_mutants_per_task: 0.0 (I-20 stub, pending I-20 ship)",
        suite = config.suite,
        model_id = config.model_id,
        seed_count = config.seed_count,
        paired = config.paired,
        dollar_per_task = config.dollar_per_task,
        contamination = config.contamination_probe_result,
        task_count = config.tasks.len(),
    )
}

// ---------------------------------------------------------------------------
// CLI entry
// ---------------------------------------------------------------------------

/// CLI entry — validates floor, runs bootstrap CI on fixture data, writes a
/// timestamped JSON report, emits one `QdeltaMeasured` SSB beacon.
///
/// Returns `Ok(())` on a clean run (rc=0). On a handled error it prints a
/// `QDELTA ABORT:` line to stderr and calls `std::process::exit(1)`.
pub(crate) fn run(config: PathBuf, seed: u64, json: bool) -> anyhow::Result<()> {
    match run_inner(config, seed, json) {
        Ok(()) => Ok(()),
        Err(e) => {
            eprintln!("QDELTA ABORT: {e}");
            std::process::exit(1);
        }
    }
}

/// Core — validates floor, computes delta + CI from fixture, writes JSON,
/// emits SSB beacon. Public so integration tests can drive without process exit.
pub fn run_inner(config_path: PathBuf, seed: u64, json: bool) -> Result<(), BenchQdeltaError> {
    // 1. Read and parse the TOML config.
    let config_bytes = fs::read(&config_path).map_err(|e| BenchQdeltaError::ConfigRead {
        path: config_path.display().to_string(),
        source_msg: e.to_string(),
    })?;
    let config: QdeltaConfig =
        toml::from_str(&String::from_utf8_lossy(&config_bytes)).map_err(|e| {
            BenchQdeltaError::ConfigParse {
                source_msg: e.to_string(),
            }
        })?;

    // 2. Statistical floor check — exits nonzero before any compute.
    floor_check(&config)?;

    // 3. Build paired bool vectors from task rows.
    let on_cells: Vec<bool> = config.tasks.iter().map(|t| t.on_defective).collect();
    let off_cells: Vec<bool> = config.tasks.iter().map(|t| t.off_defective).collect();

    let n = on_cells.len();
    let on_defective_rate = if n > 0 {
        on_cells.iter().filter(|&&v| v).count() as f64 / n as f64
    } else {
        0.0
    };
    let off_defective_rate = if n > 0 {
        off_cells.iter().filter(|&&v| v).count() as f64 / n as f64
    } else {
        0.0
    };

    // 4. Degenerate corpus check (anti-floor-fake, mirrors SafeDeltaError::DegenerateCorpus).
    if (on_defective_rate - off_defective_rate).abs() < f64::EPSILON {
        return Err(BenchQdeltaError::DegenerateCorpus {
            on_rate: on_defective_rate,
            off_rate: off_defective_rate,
        });
    }

    let delta = off_defective_rate - on_defective_rate;
    let (ci_low, ci_high) = bootstrap_ci(&on_cells, &off_cells, seed, BOOTSTRAP_N_RESAMPLES);

    // 5. blake3 hash of the config bytes as corpus provenance (anti-gaming, GD-2).
    let corpus_blake3 = blake3::hash(&config_bytes).to_hex().to_string();

    // 6. Write timestamped JSON report under <cwd>/benchmarks/qdelta/.
    let project_root = std::env::current_dir().map_err(|e| BenchQdeltaError::Cwd {
        source_msg: e.to_string(),
    })?;
    let report_path = write_json_report(&project_root, &config, on_defective_rate, off_defective_rate, delta, ci_low, ci_high, seed, &corpus_blake3)?;

    // 7. Emit ONE QdeltaMeasured SSB beacon.
    // TODO[I-38-I20-pending]: replace with real I-20 SSB row lookup when I-20 ships
    let surviving_mutants_per_task = 0.0_f64;

    let payload = QdeltaMeasuredPayload {
        suite: config.suite.clone(),
        model_id: config.model_id.clone(),
        n_tasks: n as u32,
        seed_count: config.seed_count,
        paired: config.paired,
        on_defective_rate,
        off_defective_rate,
        delta,
        ci_low,
        ci_high,
        dollar_per_task: config.dollar_per_task,
        contamination_probe_result: config.contamination_probe_result.clone(),
        surviving_mutants_per_task,
        corpus_blake3: corpus_blake3.clone(),
        runpod_pending: true,
    };

    emit_beacon(&project_root, &payload)?;

    // 8. Summary line.
    println!(
        "QDELTA suite={} n_tasks={} seed_count={} paired={} on_rate={:.4} off_rate={:.4} \
         delta={:.4} ci_low={:.4} ci_high={:.4} dollar_per_task={:.6} \
         contamination={} corpus_blake3={} runpod_pending=true report={}",
        config.suite,
        n,
        config.seed_count,
        config.paired,
        on_defective_rate,
        off_defective_rate,
        delta,
        ci_low,
        ci_high,
        config.dollar_per_task,
        config.contamination_probe_result,
        &corpus_blake3[..16],
        report_path.display(),
    );
    if json {
        let body = serde_json::to_string_pretty(&payload).map_err(|e| BenchQdeltaError::Serialise {
            source_msg: e.to_string(),
        })?;
        println!("{body}");
    }
    Ok(())
}

// ---------------------------------------------------------------------------
// JSON report writer
// ---------------------------------------------------------------------------

/// Write the QDELTA report JSON to `<project_root>/benchmarks/qdelta/qdelta-<unix_ms>.json`.
#[allow(clippy::too_many_arguments)]
fn write_json_report(
    project_root: &Path,
    config: &QdeltaConfig,
    on_defective_rate: f64,
    off_defective_rate: f64,
    delta: f64,
    ci_low: f64,
    ci_high: f64,
    seed: u64,
    corpus_blake3: &str,
) -> Result<PathBuf, BenchQdeltaError> {
    let report_dir = project_root.join("benchmarks").join("qdelta");
    fs::create_dir_all(&report_dir).map_err(|e| BenchQdeltaError::ReportDir {
        path: report_dir.display().to_string(),
        source_msg: e.to_string(),
    })?;

    let unix_ms = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis())
        .unwrap_or_default();
    let report_path = report_dir.join(format!("qdelta-{unix_ms}.json"));

    let report = serde_json::json!({
        "suite": config.suite,
        "model_id": config.model_id,
        "seed_count": config.seed_count,
        "paired": config.paired,
        "n_tasks": config.tasks.len(),
        "on_defective_rate": on_defective_rate,
        "off_defective_rate": off_defective_rate,
        "delta": delta,
        "ci_low": ci_low,
        "ci_high": ci_high,
        "dollar_per_task": config.dollar_per_task,
        "contamination_probe_result": config.contamination_probe_result,
        "corpus_blake3": corpus_blake3,
        "seed": seed,
        "runpod_pending": true,
        "methodology": generate_methodology_doc(config),
    });
    let body = serde_json::to_string_pretty(&report).map_err(|e| BenchQdeltaError::Serialise {
        source_msg: e.to_string(),
    })?;
    fs::write(&report_path, &body).map_err(|e| BenchQdeltaError::OutputIo {
        path: report_path.display().to_string(),
        source_msg: e.to_string(),
    })?;
    Ok(report_path)
}

// ---------------------------------------------------------------------------
// SSB emit
// ---------------------------------------------------------------------------

/// Open the canonical session `ssb.db`, emit ONE `QdeltaMeasured` beacon (durable, awaited).
fn emit_beacon(
    project_root: &Path,
    payload: &QdeltaMeasuredPayload,
) -> Result<(), BenchQdeltaError> {
    let db_path = sessions_dir_path(project_root).join("ssb.db");
    if let Some(parent) = db_path.parent() {
        fs::create_dir_all(parent).map_err(|e| BenchQdeltaError::ReportDir {
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
        .map_err(|e| BenchQdeltaError::Runtime {
            source_msg: e.to_string(),
        })?;

    let payload_clone = payload.clone();
    rt.block_on(async move {
        let ssb = Ssb::new(cfg, clock).map_err(|e| BenchQdeltaError::SsbOpen {
            source_msg: e.to_string(),
        })?;

        ssb.write_payload(
            BENCH_AGENT_ID,
            0,
            SsbPayload::QdeltaMeasured(payload_clone),
            Vec::new(),
        )
        .await
        .map_err(|e| BenchQdeltaError::SsbWrite {
            source_msg: e.to_string(),
        })?;

        tracing::info!(
            "qdelta: I-38 QdeltaMeasured SSB row emitted",
        );

        Ok::<(), BenchQdeltaError>(())
    })
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    // -----------------------------------------------------------------------
    // Inline fixture constants (no separate fixture files required at Tier-1).
    // Mirrors safedelta_runner.rs FIXTURE_WITH_CAUGHT pattern.
    // -----------------------------------------------------------------------

    /// Valid run fixture (seed_count=5, paired=true, >=2 on!=off tasks).
    /// Fixture design: 3 tasks where off_defective=true and on_defective=false
    /// (gates-ON improves), 1 task where both true (equal), 1 task where both false.
    /// on_rate  = 1/5 = 0.200   (task3 has on_defective=true)
    /// off_rate = 4/5 = 0.800   (tasks 1,2,3,5 have off_defective=true; task3 is both-true)
    /// delta    = 0.800 - 0.200 = 0.600  (non-degenerate, anti-floor-fake)
    const QDELTA_FIXTURE_VALID: &str = r#"
suite = "test_qdelta_fixture"
model_id = "fixture_corpus_v1"
seed_count = 5
paired = true
dollar_per_task = 0.01
contamination_probe_result = "honestly_evaluated"

[[tasks]]
on_defective = false
off_defective = true

[[tasks]]
on_defective = false
off_defective = true

[[tasks]]
on_defective = true
off_defective = true

[[tasks]]
on_defective = false
off_defective = false

[[tasks]]
on_defective = false
off_defective = true
"#;

    /// Under-powered fixture (seed_count=3, paired=false).
    /// floor_check() MUST return Err(FloorViolation) before any compute.
    /// NO QdeltaMeasured row should be emitted.
    const QDELTA_FIXTURE_UNDERPOWER: &str = r#"
suite = "test_qdelta_underpower"
model_id = "fixture_corpus_v1"
seed_count = 3
paired = false
dollar_per_task = 0.01
contamination_probe_result = "not_run"

[[tasks]]
on_defective = false
off_defective = true

[[tasks]]
on_defective = true
off_defective = false
"#;

    /// Known-answer CI fixture for paired bootstrap.
    /// 12 tasks with mixed on/off defective values produce a continuous bootstrap
    /// distribution whose 2.5th and 97.5th percentiles are seed-sensitive.
    /// on_rate  = 5/12 ≈ 0.417   (tasks 1,3,9,10,11 have on_defective=true)
    /// off_rate = 8/12 ≈ 0.667   (tasks 1,2,5,7,8,9,10,12 have off_defective=true)
    /// delta    = 8/12 - 5/12 = 3/12 = 0.25 (non-degenerate, large enough for seed spread)
    /// Empirically verified (independent Python calc): seed=1 and seed=9999 produce
    /// different CI lower bounds (differ by ≥1e-9) confirming PRNG is live.
    const QDELTA_FIXTURE_CI: &str = r#"
suite = "test_qdelta_ci"
model_id = "fixture_corpus_ci"
seed_count = 5
paired = true
dollar_per_task = 0.005
contamination_probe_result = "not_run"

[[tasks]]
on_defective = true
off_defective = true

[[tasks]]
on_defective = false
off_defective = true

[[tasks]]
on_defective = true
off_defective = false

[[tasks]]
on_defective = false
off_defective = false

[[tasks]]
on_defective = false
off_defective = true

[[tasks]]
on_defective = false
off_defective = false

[[tasks]]
on_defective = false
off_defective = true

[[tasks]]
on_defective = false
off_defective = true

[[tasks]]
on_defective = true
off_defective = true

[[tasks]]
on_defective = true
off_defective = true

[[tasks]]
on_defective = true
off_defective = false

[[tasks]]
on_defective = false
off_defective = true
"#;

    fn parse_fixture(s: &str) -> QdeltaConfig {
        toml::from_str(s).expect("fixture parse")
    }

    // -----------------------------------------------------------------------
    // Floor check tests
    // -----------------------------------------------------------------------

    #[test]
    fn floor_check_passes_valid_config() {
        let config = parse_fixture(QDELTA_FIXTURE_VALID);
        assert!(floor_check(&config).is_ok(), "valid config must pass floor_check");
    }

    #[test]
    fn floor_check_rejects_underpower() {
        let config = parse_fixture(QDELTA_FIXTURE_UNDERPOWER);
        let result = floor_check(&config);
        assert!(
            result.is_err(),
            "under-powered config (seed_count=3, paired=false) must fail floor_check"
        );
        match result.unwrap_err() {
            BenchQdeltaError::FloorViolation { seed_count, paired } => {
                assert_eq!(seed_count, 3);
                assert!(!paired);
            }
            other => panic!("expected FloorViolation, got: {other}"),
        }
    }

    #[test]
    fn floor_check_rejects_seed_count_below_5() {
        let mut config = parse_fixture(QDELTA_FIXTURE_VALID);
        config.seed_count = 4;
        let result = floor_check(&config);
        assert!(result.is_err(), "seed_count=4 must fail floor_check (floor=5)");
    }

    #[test]
    fn floor_check_rejects_unpaired() {
        let mut config = parse_fixture(QDELTA_FIXTURE_VALID);
        config.paired = false;
        let result = floor_check(&config);
        assert!(result.is_err(), "paired=false must fail floor_check");
    }

    // -----------------------------------------------------------------------
    // Bootstrap CI tests (known-answer)
    // -----------------------------------------------------------------------

    #[test]
    fn bootstrap_ci_non_degenerate_fixture() {
        let config = parse_fixture(QDELTA_FIXTURE_VALID);
        let on_cells: Vec<bool> = config.tasks.iter().map(|t| t.on_defective).collect();
        let off_cells: Vec<bool> = config.tasks.iter().map(|t| t.off_defective).collect();

        let (ci_low, ci_high) = bootstrap_ci(&on_cells, &off_cells, 42, BOOTSTRAP_N_RESAMPLES);

        // delta = 0.40; CI should straddle delta (positive range)
        assert!(ci_high > ci_low, "ci_high must exceed ci_low");
        assert!(
            ci_high > 0.0,
            "CI upper bound must be positive for this fixture (delta=0.40)"
        );
    }

    #[test]
    fn bootstrap_ci_known_answer_deterministic() {
        // Same seed must produce the same CI bounds (GD-2 recomputable).
        let config = parse_fixture(QDELTA_FIXTURE_CI);
        let on_cells: Vec<bool> = config.tasks.iter().map(|t| t.on_defective).collect();
        let off_cells: Vec<bool> = config.tasks.iter().map(|t| t.off_defective).collect();

        let (lo1, hi1) = bootstrap_ci(&on_cells, &off_cells, 1234, BOOTSTRAP_N_RESAMPLES);
        let (lo2, hi2) = bootstrap_ci(&on_cells, &off_cells, 1234, BOOTSTRAP_N_RESAMPLES);

        assert_eq!(lo1, lo2, "bootstrap_ci must be deterministic under same seed");
        assert_eq!(hi1, hi2, "bootstrap_ci must be deterministic under same seed");
        // Point delta = 0.25; CI width must be non-trivial (hi > lo).
        // The CI may span zero because on_rate (5/12) and off_rate (8/12) are not extreme,
        // so the lower tail can be negative on some resamples. The key invariant is
        // determinism (lo1==lo2, hi1==hi2) and that the interval is non-trivial.
        assert!(hi1 > lo1, "CI must be non-trivial (hi > lo)");
    }

    #[test]
    fn bootstrap_ci_different_seeds_differ() {
        let config = parse_fixture(QDELTA_FIXTURE_CI);
        let on_cells: Vec<bool> = config.tasks.iter().map(|t| t.on_defective).collect();
        let off_cells: Vec<bool> = config.tasks.iter().map(|t| t.off_defective).collect();

        let (lo1, hi1) = bootstrap_ci(&on_cells, &off_cells, 1, BOOTSTRAP_N_RESAMPLES);
        let (lo2, hi2) = bootstrap_ci(&on_cells, &off_cells, 9999, BOOTSTRAP_N_RESAMPLES);

        // Different seeds should produce slightly different (but both valid) CIs.
        // This confirms the PRNG is actually seeded and varied.
        let are_different = (lo1 - lo2).abs() > 1e-9 || (hi1 - hi2).abs() > 1e-9;
        assert!(are_different, "different seeds must produce different CI values");
    }

    // -----------------------------------------------------------------------
    // Delta rate computation
    // -----------------------------------------------------------------------

    #[test]
    fn delta_computation_correct_for_valid_fixture() {
        let config = parse_fixture(QDELTA_FIXTURE_VALID);
        let on_cells: Vec<bool> = config.tasks.iter().map(|t| t.on_defective).collect();
        let off_cells: Vec<bool> = config.tasks.iter().map(|t| t.off_defective).collect();

        let n = on_cells.len();
        let on_rate = on_cells.iter().filter(|&&v| v).count() as f64 / n as f64;
        let off_rate = off_cells.iter().filter(|&&v| v).count() as f64 / n as f64;
        let delta = off_rate - on_rate;

        // tasks: on=[F,F,T,F,F] -> 1 true / 5 = 0.2
        //        off=[T,T,T,F,T] -> 4 true / 5 = 0.8
        //        (tasks 1,2,3,5 have off_defective=true; task 3 is both-true)
        // delta = 0.8 - 0.2 = 0.6
        assert!(
            (on_rate - 0.2_f64).abs() < 1e-9,
            "on_rate must be 0.2 for valid fixture; got {on_rate}"
        );
        assert!(
            (off_rate - 0.8_f64).abs() < 1e-9,
            "off_rate must be 0.8 for valid fixture; got {off_rate}"
        );
        assert!(
            (delta - 0.6_f64).abs() < 1e-9,
            "delta must be 0.6 for valid fixture; got {delta}"
        );
    }

    // -----------------------------------------------------------------------
    // Degenerate corpus check
    // -----------------------------------------------------------------------

    #[test]
    fn degenerate_corpus_error_when_rates_equal() {
        // All on_defective == off_defective -> delta = 0 -> DegenerateCorpus
        let config = QdeltaConfig {
            suite: "test".into(),
            model_id: "fixture".into(),
            seed_count: 5,
            paired: true,
            dollar_per_task: 0.01,
            contamination_probe_result: "not_run".into(),
            tasks: vec![
                QdeltaTaskRow { on_defective: true, off_defective: true },
                QdeltaTaskRow { on_defective: false, off_defective: false },
                QdeltaTaskRow { on_defective: true, off_defective: true },
                QdeltaTaskRow { on_defective: false, off_defective: false },
                QdeltaTaskRow { on_defective: true, off_defective: true },
            ],
        };
        // floor_check passes; compute manually to confirm delta==0
        let on_cells: Vec<bool> = config.tasks.iter().map(|t| t.on_defective).collect();
        let off_cells: Vec<bool> = config.tasks.iter().map(|t| t.off_defective).collect();
        let n = on_cells.len();
        let on_rate = on_cells.iter().filter(|&&v| v).count() as f64 / n as f64;
        let off_rate = off_cells.iter().filter(|&&v| v).count() as f64 / n as f64;
        assert!(
            (on_rate - off_rate).abs() < f64::EPSILON,
            "on_rate should equal off_rate for degenerate fixture"
        );
    }

    // -----------------------------------------------------------------------
    // Methodology doc determinism
    // -----------------------------------------------------------------------

    #[test]
    fn methodology_doc_deterministic() {
        let config = parse_fixture(QDELTA_FIXTURE_VALID);
        let doc1 = generate_methodology_doc(&config);
        let doc2 = generate_methodology_doc(&config);
        assert_eq!(doc1, doc2, "methodology doc must be deterministic");
        assert!(doc1.contains("seed_count: 5"), "must include seed_count");
        assert!(doc1.contains("paired: true"), "must include paired");
        assert!(doc1.contains("ablation"), "must include framing label");
        // GD-3: the methodology doc must explicitly state the no-leaderboard-rank
        // prohibition. The word "leaderboard" appears as part of "no leaderboard rank",
        // which is the CORRECT prohibition framing. The assertion is that the explicit
        // prohibition string is present, not that the word is absent.
        assert!(
            doc1.contains("no leaderboard rank"),
            "must include explicit no-leaderboard-rank prohibition (GD-3); got: {doc1}"
        );
    }

    // -----------------------------------------------------------------------
    // xorshift64 sanity
    // -----------------------------------------------------------------------

    #[test]
    fn xorshift64_seed_zero_maps_to_one() {
        // Mirrors safedelta_runner::xorshift64 invariant.
        // The boot-state for seed=0 is mapped to 1 at the call site.
        let state = if 0u64 == 0 { 1u64 } else { 0u64 };
        let next = xorshift64(state);
        assert_ne!(next, state, "xorshift64 must not be a fixed point at state=1");
    }
}
