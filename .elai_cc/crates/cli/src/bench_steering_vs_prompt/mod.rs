//! J-24 MS4 -- `elai bench steering-vs-prompt --task <fixture> [--seed <u64>] [--json]`
//! CLI driver.
//!
//! STEERING-VS-PROMPT = deterministic fixture scoring of a steered arm vs a
//! role-prompt baseline arm. Two fixture evaluations run; the delta between
//! them is the behavior_delta. No live LLM judge (TD-3): all metrics come
//! from the fixture TOML (sycophancy-marker count, output length, tool-call
//! pattern match against expected).
//!
//! # What this measures (honestly)
//!
//! For each arm (steered / baseline), the fixture supplies deterministic
//! outputs and expected metrics. The bench computes:
//! - behavior_delta: difference in sycophancy-marker count between arms
//! - token_cost_delta: difference in output length between arms
//! - tool_call_pattern_match: whether the steered arm's tool-call pattern
//!   matches the expected pattern in the fixture
//!
//! A non-zero behavior_delta on a fixture that expects a delta is the
//! positive signal. A near-zero delta on an identical-arms fixture is the
//! negative control.
//!
//! # Fixture tier (Tier-1)
//!
//! The fixture TOML supplies both arms' outputs and expected metrics. No
//! live inference runs. The live per-model leg is deferred; the payload
//! shape is forward-compatible.
//!
//! # Wiring
//!
//! producer  : this file -- reads SteeringVsPromptConfig TOML, validates,
//!             scores both arms, computes deltas, emits a
//!             SteeringEffectMeasured SSB row via
//!             emit_behavior_steering_event, writes a TOML report.
//! ssb row   : `SsbActionType::BehaviorSteeringEvent(
//!               BehaviorSteeringKind::SteeringEffectMeasured)`
//! entrypoint: `elai bench steering-vs-prompt --task <fixture> [--seed <u64>]`.
//!
//! # No `.unwrap()` / `.expect()` in non-test paths.
//! # blake3 only for all content hashing.

use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

use elai_core::paths::sessions_dir_path;
use elai_orchestrator::behavior_steering_ssb::emit_behavior_steering_event;
use elai_orchestrator::ssb::{BehaviorSteeringKind, SsbActionType};
use elai_orchestrator::{Ssb, SsbConfig};
use serde::{Deserialize, Serialize};
use thiserror::Error;

const BENCH_SESSION_ID: &str = "cli-bench-steering-vs-prompt";

#[allow(dead_code)]
const BENCH_AGENT_ID: &str = "cli:bench-steering-vs-prompt";

// ---------------------------------------------------------------------------
// Config (read from TOML)
// ---------------------------------------------------------------------------

/// One arm of the steering-vs-prompt comparison.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SteeringArm {
    /// Output text the arm produced (fixture-supplied, not live).
    pub output: String,
    /// Sycophancy markers found in the output (fixture-supplied count).
    pub sycophancy_markers: u32,
    /// Token cost of the output (fixture-supplied).
    pub token_cost: u32,
    /// Tool-call pattern the arm produced (fixture-supplied).
    pub tool_call_pattern: String,
}

/// Expected metrics for the comparison.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SteeringExpected {
    /// Expected behavior_delta (sycophancy_markers difference). Non-zero
    /// for a fixture that expects a steering effect.
    pub behavior_delta: f64,
    /// Expected token_cost_delta.
    pub token_cost_delta: f64,
    /// Whether the steered arm's tool_call_pattern should match this string.
    pub tool_call_pattern_match: bool,
    /// Expected tool-call pattern for the steered arm.
    pub expected_tool_call_pattern: String,
}

/// Steering-vs-prompt bench configuration (read from TOML).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SteeringVsPromptConfig {
    /// Human-readable task label.
    pub task: String,
    /// The steered arm.
    pub steered: SteeringArm,
    /// The baseline (role-prompt) arm.
    pub baseline: SteeringArm,
    /// Expected metrics for the comparison.
    pub expected: SteeringExpected,
}

// ---------------------------------------------------------------------------
// Error type
// ---------------------------------------------------------------------------

#[derive(Debug, Error)]
pub enum BenchSteeringVsPromptError {
    #[error("steering-vs-prompt could not read config at {path}: {source_msg}")]
    ConfigRead { path: String, source_msg: String },
    #[error("steering-vs-prompt could not parse config TOML: {source_msg}")]
    ConfigParse { source_msg: String },
    #[error("steering-vs-prompt could not create dir {path}: {source_msg}")]
    ReportDir { path: String, source_msg: String },
    #[error("steering-vs-prompt could not write {path}: {source_msg}")]
    OutputIo { path: String, source_msg: String },
    #[error("steering-vs-prompt could not serialise report: {source_msg}")]
    Serialise { source_msg: String },
    #[error("steering-vs-prompt could not build async runtime: {source_msg}")]
    Runtime { source_msg: String },
    #[error("steering-vs-prompt could not open SSB writer: {source_msg}")]
    SsbOpen { source_msg: String },
    #[error("steering-vs-prompt fixture audit failed: {reason}")]
    FixtureAudit { reason: String },
}

// ---------------------------------------------------------------------------
// Scoring
// ---------------------------------------------------------------------------

/// Computed deltas between steered and baseline arms.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SteeringDeltas {
    pub behavior_delta: f64,
    pub token_cost_delta: f64,
    pub tool_call_pattern_match: bool,
}

/// Score the two arms deterministically.
pub fn score_arms(config: &SteeringVsPromptConfig) -> SteeringDeltas {
    let behavior_delta =
        f64::from(config.steered.sycophancy_markers) - f64::from(config.baseline.sycophancy_markers);
    let token_cost_delta =
        f64::from(config.steered.token_cost) - f64::from(config.baseline.token_cost);
    let tool_call_pattern_match =
        config.steered.tool_call_pattern == config.expected.expected_tool_call_pattern;
    SteeringDeltas {
        behavior_delta,
        token_cost_delta,
        tool_call_pattern_match,
    }
}

/// Audit the fixture: reject lorem/random padding in the task label, and
/// reject empty outputs.
pub fn audit_fixture(config: &SteeringVsPromptConfig) -> Result<(), BenchSteeringVsPromptError> {
    let task_lower = config.task.to_lowercase();
    if task_lower.contains("lorem") || task_lower.contains("random") {
        return Err(BenchSteeringVsPromptError::FixtureAudit {
            reason: format!(
                "task label '{task}' contains lorem/random filler; fixture audit rejects unstructured padding",
                task = config.task
            ),
        });
    }
    if config.steered.output.is_empty() || config.baseline.output.is_empty() {
        return Err(BenchSteeringVsPromptError::FixtureAudit {
            reason: "arm output is empty; fixture must supply non-empty outputs".to_string(),
        });
    }
    Ok(())
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

pub(crate) fn run(task: PathBuf, seed: u64, json: bool) -> anyhow::Result<()> {
    let project_root = std::env::current_dir().map_err(|e| anyhow::anyhow!("cwd: {e}"))?;
    match run_inner(task, seed, json, &project_root) {
        Ok(()) => Ok(()),
        Err(e) => {
            eprintln!("STEERING-VS-PROMPT ABORT: {e}");
            std::process::exit(1);
        }
    }
}

pub fn run_inner(
    task_path: PathBuf,
    seed: u64,
    json: bool,
    project_root: &Path,
) -> Result<(), BenchSteeringVsPromptError> {
    let config_bytes = fs::read(&task_path).map_err(|e| BenchSteeringVsPromptError::ConfigRead {
        path: task_path.display().to_string(),
        source_msg: e.to_string(),
    })?;
    let config: SteeringVsPromptConfig =
        toml::from_str(&String::from_utf8_lossy(&config_bytes)).map_err(|e| {
            BenchSteeringVsPromptError::ConfigParse {
                source_msg: e.to_string(),
            }
        })?;

    audit_fixture(&config)?;

    let fixture_blake3 = blake3::hash(&config_bytes).to_hex().to_string();
    let elai_seed = if seed > 0 { seed } else { read_elai_seed() };

    let deltas = score_arms(&config);

    let now_ms = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis() as i64)
        .unwrap_or_default();

    let report_path = write_toml_report(
        project_root,
        &config.task,
        &deltas,
        &fixture_blake3,
        elai_seed,
        now_ms,
    )?;

    emit_steering_event(project_root, &deltas, &config, elai_seed)?;

    let attribution = "fixture:steering-vs-prompt";

    println!(
        "STEERING-VS-PROMPT task={} behavior_delta={:.4} token_cost_delta={:.4} \
         tool_call_pattern_match={} attribution={} elai_seed={} fixture_blake3={} report={}",
        config.task,
        deltas.behavior_delta,
        deltas.token_cost_delta,
        deltas.tool_call_pattern_match,
        attribution,
        elai_seed,
        &fixture_blake3[..16],
        report_path.display(),
    );

    if json {
        let body = serde_json::to_string_pretty(&deltas).map_err(|e| {
            BenchSteeringVsPromptError::Serialise {
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
    task: &str,
    deltas: &SteeringDeltas,
    fixture_blake3: &str,
    elai_seed: u64,
    ts_ms: i64,
) -> Result<PathBuf, BenchSteeringVsPromptError> {
    let report_dir = project_root
        .join("benchmarks")
        .join("steering_vs_prompt");
    fs::create_dir_all(&report_dir).map_err(|e| BenchSteeringVsPromptError::ReportDir {
        path: report_dir.display().to_string(),
        source_msg: e.to_string(),
    })?;

    let safe_task = task.replace(|c: char| !c.is_alphanumeric() && c != '_' && c != '-', "_");
    let report_path = report_dir.join(format!("steering-vs-prompt-{safe_task}-{ts_ms}.toml"));

    let body = format!(
        "# J-24 steering-vs-prompt report\n\
         task = {task:?}\n\
         behavior_delta = {behavior_delta:.6}\n\
         token_cost_delta = {token_cost_delta:.6}\n\
         tool_call_pattern_match = {tool_call_pattern_match}\n\
         fixture_blake3 = {blake3:?}\n\
         elai_seed = {elai_seed}\n\
         ts_ms = {ts_ms}\n",
        task = task,
        behavior_delta = deltas.behavior_delta,
        token_cost_delta = deltas.token_cost_delta,
        tool_call_pattern_match = deltas.tool_call_pattern_match,
        blake3 = fixture_blake3,
        elai_seed = elai_seed,
        ts_ms = ts_ms,
    );

    fs::write(&report_path, &body).map_err(|e| BenchSteeringVsPromptError::OutputIo {
        path: report_path.display().to_string(),
        source_msg: e.to_string(),
    })?;
    Ok(report_path)
}

// ---------------------------------------------------------------------------
// SSB emit (via emit_behavior_steering_event)
// ---------------------------------------------------------------------------

fn emit_steering_event(
    project_root: &Path,
    deltas: &SteeringDeltas,
    config: &SteeringVsPromptConfig,
    elai_seed: u64,
) -> Result<(), BenchSteeringVsPromptError> {
    let db_path = sessions_dir_path(project_root).join("ssb.db");
    if let Some(parent) = db_path.parent() {
        fs::create_dir_all(parent).map_err(|e| BenchSteeringVsPromptError::ReportDir {
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
        .map_err(|e| BenchSteeringVsPromptError::Runtime {
            source_msg: e.to_string(),
        })?;

    let details_json = serde_json::json!({
        "behavior_delta": deltas.behavior_delta,
        "token_cost_delta": deltas.token_cost_delta,
        "tool_call_pattern_match": deltas.tool_call_pattern_match,
        "attribution": "fixture:steering-vs-prompt",
        "task": config.task,
        "elai_seed": elai_seed,
    });

    rt.block_on(async move {
        let ssb: Arc<Ssb> = Arc::new(
            Ssb::new(cfg, clock).map_err(|e| BenchSteeringVsPromptError::SsbOpen {
                source_msg: e.to_string(),
            })?,
        );

        emit_behavior_steering_event(
            &ssb,
            BehaviorSteeringKind::SteeringEffectMeasured,
            details_json,
        );

        let expected_seed = elai_seed;
        let entry = crate::bench_ssb_poll::poll_until_this_run_row(
            &ssb,
            SsbActionType::BehaviorSteeringEvent(
                BehaviorSteeringKind::SteeringEffectMeasured,
            ),
            move |row| {
                if let Ok(elai_orchestrator::ssb::SsbPayload::BehaviorSteeringEvent(
                    elai_orchestrator::ssb::BehaviorSteeringEventPayload {
                        details_json, ..
                    },
                )) = elai_orchestrator::ssb::decode_ssb_payload(&row.payload)
                {
                    details_json
                        .get("elai_seed")
                        .and_then(|v| v.as_u64())
                        .is_some_and(|s| s == expected_seed)
                } else {
                    false
                }
            },
        )
        .await
        .map_err(|e| BenchSteeringVsPromptError::SsbOpen {
            source_msg: format!(
                "no BehaviorSteeringEvent(SteeringEffectMeasured) row with elai_seed={elai_seed} landed: {e}"
            ),
        })?;

        tracing::info!(
            steering_vs_prompt.ssb_seq = entry.seq,
            "steering-vs-prompt: SteeringEffectMeasured SSB row emitted (elai_seed={elai_seed})"
        );

        Ok::<(), BenchSteeringVsPromptError>(())
    })
}

#[cfg(test)]
mod tests_inline;
