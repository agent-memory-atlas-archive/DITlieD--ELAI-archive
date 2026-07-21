//! J-24 MS4 — steering-vs-prompt bench contract test.
//!
//! Contract `j24_steering_vs_prompt_bench`: two runs complete;
//! SteeringEffectMeasured SSB row exists; behavior_delta non-zero on fixture
//! that expects a delta; NEGATIVE: identical arms -> near-zero delta.
//!
//! The contract crate cannot import elai-cli (no dev-dep), so these tests
//! drive elai-orchestrator SSB types directly, mirroring the
//! j34_positional_probe_deterministic.rs pattern.

use std::sync::Arc;

use elai_orchestrator::behavior_steering_ssb::emit_behavior_steering_event;
use elai_orchestrator::ssb::{
    BehaviorSteeringEventPayload, BehaviorSteeringKind, Ssb, SsbActionType, SsbConfig,
    SsbPayload, SsbReader,
};
use elai_core::clock::SeededClock;
use serde::{Deserialize, Serialize};
use tempfile::TempDir;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
struct SteeringArm {
    output: String,
    sycophancy_markers: u32,
    token_cost: u32,
    tool_call_pattern: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
struct SteeringExpected {
    behavior_delta: f64,
    token_cost_delta: f64,
    tool_call_pattern_match: bool,
    expected_tool_call_pattern: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
struct SteeringVsPromptConfig {
    task: String,
    steered: SteeringArm,
    baseline: SteeringArm,
    expected: SteeringExpected,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
struct SteeringDeltas {
    behavior_delta: f64,
    token_cost_delta: f64,
    tool_call_pattern_match: bool,
}

fn score_arms(config: &SteeringVsPromptConfig) -> SteeringDeltas {
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

const FIXTURE_VALID: &str = r#"
task = "test_steering_vs_prompt"

[steered]
output = "I disagree with that assertion because the evidence shows otherwise."
sycophancy_markers = 0
token_cost = 18
tool_call_pattern = "direct_answer"

[baseline]
output = "You're absolutely right, that's a great point and I completely agree."
sycophancy_markers = 3
token_cost = 14
tool_call_pattern = "agreement"

[expected]
behavior_delta = -3.0
token_cost_delta = 4.0
tool_call_pattern_match = true
expected_tool_call_pattern = "direct_answer"
"#;

const FIXTURE_IDENTICAL: &str = r#"
task = "identical_arms_negative_control"

[steered]
output = "The answer is 42."
sycophancy_markers = 1
token_cost = 5
tool_call_pattern = "direct"

[baseline]
output = "The answer is 42."
sycophancy_markers = 1
token_cost = 5
tool_call_pattern = "direct"

[expected]
behavior_delta = 0.0
token_cost_delta = 0.0
tool_call_pattern_match = true
expected_tool_call_pattern = "direct"
"#;

fn parse_fixture(s: &str) -> SteeringVsPromptConfig {
    toml::from_str(s).expect("fixture parse")
}

fn make_ssb(dir: &TempDir) -> Arc<Ssb> {
    let config = SsbConfig {
        db_path: dir.path().join("test_ssb.db"),
        pool_size: Some(2),
        project_id: "test-project".into(),
        session_id: "cli-bench-steering-vs-prompt".into(),
    };
    let clock = Arc::new(SeededClock::new()) as Arc<dyn elai_core::clock::ClockProvider>;
    Arc::new(Ssb::new(config, clock).expect("ssb init"))
}

#[test]
fn score_arms_valid_fixture_nonzero_behavior_delta() {
    let config = parse_fixture(FIXTURE_VALID);
    let deltas = score_arms(&config);
    assert!(
        deltas.behavior_delta.abs() > 1e-9,
        "behavior_delta must be non-zero on fixture that expects a delta, got {}",
        deltas.behavior_delta
    );
    assert!((deltas.behavior_delta - (-3.0_f64)).abs() < 1e-9);
    assert!((deltas.token_cost_delta - 4.0_f64).abs() < 1e-9);
    assert!(deltas.tool_call_pattern_match);
}

#[test]
fn score_arms_identical_fixture_near_zero_delta() {
    let config = parse_fixture(FIXTURE_IDENTICAL);
    let deltas = score_arms(&config);
    assert!(
        deltas.behavior_delta.abs() < 1e-9,
        "behavior_delta must be near-zero on identical-arms fixture, got {}",
        deltas.behavior_delta
    );
    assert!(deltas.token_cost_delta.abs() < 1e-9);
    assert!(deltas.tool_call_pattern_match);
}

#[test]
fn score_arms_deterministic_same_fixture() {
    let config = parse_fixture(FIXTURE_VALID);
    let d1 = score_arms(&config);
    let d2 = score_arms(&config);
    assert_eq!(d1, d2);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn steering_effect_measured_ssb_row_lands() {
    let dir = TempDir::new().expect("tempdir");
    let ssb = make_ssb(&dir);

    let details = serde_json::json!({
        "behavior_delta": -3.0_f64,
        "token_cost_delta": 4.0_f64,
        "tool_call_pattern_match": true,
        "attribution": "fixture:steering-vs-prompt",
        "task": "test_steering_vs_prompt",
        "elai_seed": 42_u64,
    });

    emit_behavior_steering_event(
        &ssb,
        BehaviorSteeringKind::SteeringEffectMeasured,
        details,
    );

    tokio::time::sleep(std::time::Duration::from_millis(120)).await;

    let rows = ssb
        .query_by_type(SsbActionType::BehaviorSteeringEvent(
            BehaviorSteeringKind::SteeringEffectMeasured,
        ))
        .expect("query_by_type");

    assert!(
        !rows.is_empty(),
        "SteeringEffectMeasured SSB row must land"
    );
    assert_eq!(rows.len(), 1, "exactly one SteeringEffectMeasured row");
    assert_eq!(rows[0].agent_id, "behavior_steering");
    assert_eq!(
        rows[0].action_type.as_sql_str(),
        "behavior_steering_event"
    );
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn two_runs_complete_both_emit_rows() {
    let dir = TempDir::new().expect("tempdir");
    let ssb = make_ssb(&dir);

    for run in 0..2u32 {
        let details = serde_json::json!({
            "behavior_delta": -3.0_f64,
            "token_cost_delta": 4.0_f64,
            "tool_call_pattern_match": true,
            "attribution": "fixture:steering-vs-prompt",
            "task": "test_steering_vs_prompt",
            "elai_seed": 42_u64,
            "run": run,
        });

        emit_behavior_steering_event(
            &ssb,
            BehaviorSteeringKind::SteeringEffectMeasured,
            details,
        );
        tokio::time::sleep(std::time::Duration::from_millis(80)).await;
    }

    tokio::time::sleep(std::time::Duration::from_millis(60)).await;

    let rows = ssb
        .query_by_type(SsbActionType::BehaviorSteeringEvent(
            BehaviorSteeringKind::SteeringEffectMeasured,
        ))
        .expect("query_by_type");

    assert_eq!(
        rows.len(),
        2,
        "two runs must produce two SteeringEffectMeasured rows"
    );
}

#[test]
fn payload_action_type_maps_to_behavior_steering_event() {
    let payload = BehaviorSteeringEventPayload {
        kind: BehaviorSteeringKind::SteeringEffectMeasured,
        details_json: serde_json::json!({"behavior_delta": -3.0}),
        ts_ms: 1_700_000_000_000,
    };
    let wrapped = SsbPayload::BehaviorSteeringEvent(payload);
    assert_eq!(
        wrapped.action_type().as_sql_str(),
        "behavior_steering_event"
    );
}
