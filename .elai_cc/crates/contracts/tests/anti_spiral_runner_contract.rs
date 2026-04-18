// Session wire-antispiral contract tests.
//
// Exercises the AntiSpiralDetector wiring surfaces the runner depends on:
//   * `TaskRunnerConfig::anti_spiral_enabled` toggles the detector hook.
//   * `AntiSpiralDetector::record_step` surfaces the same three outcomes the
//     runner branches on: `Clean`, `EscalationSignalNeeded { message, .. }`,
//     `HardStop { spiral_depth }`.
//   * `SsbPayload::SpiralDetected` has the payload shape the runner emits on
//     HARD_STOP (stable `agent_id`/`phase_id`/`spiral_depth` fields).
//   * `ESCALATION_MESSAGE` + `HARD_STOP_DEPTH` constants stay stable — the
//     runner injects the exact message into the next agent turn and writes
//     the exact spiral_depth into the SSB row.
//
// Note: the full "drive_task + three identical GRC steps -> SpiralDetected row"
// path requires a live sidecar for `research_decompose` in synthetic mode,
// which is intentionally out of scope for hermetic contract tests. The
// existing `task_runner_consumes_submitted_task_and_emits_progress` test
// covers the end-to-end happy path; this contract proves the detector +
// payload surfaces are the contract the runner wires to.

use elai_core::SeededClock;
use elai_orchestrator::anti_spiral::{
    AntiSpiralDetector, SpiralResult, ESCALATION_DEPTH, ESCALATION_MESSAGE, HARD_STOP_DEPTH,
    SPIRAL_THRESHOLD,
};
use elai_orchestrator::ssb::{SpiralDetectedPayload, SsbPayload};
use elai_orchestrator::{TaskRunnerConfig, SPIRAL_HARDSTOP_CATEGORY};
use uuid::Uuid;

// Contract 1: anti_spiral_enabled defaults to false on new_minimal — the
// pre-wire-antispiral behavior stays the default so legacy tests do not
// observe new SpiralDetected rows.
#[test]
fn contract_anti_spiral_disabled_by_default() {
    let cfg = TaskRunnerConfig::new_minimal(std::path::PathBuf::from("cursor.json"));
    assert!(
        !cfg.anti_spiral_enabled,
        "wire-antispiral must stay off by default to preserve legacy runner semantics"
    );
}

// Contract 2: Three identical steps are Clean (SPIRAL_THRESHOLD == 3 and
// escalation fires strictly _after_ the threshold). Runner branches on the
// fourth step to inject ESCALATION_MESSAGE into the next agent turn.
#[test]
fn contract_three_identical_steps_clean_fourth_escalates() {
    let mut det = AntiSpiralDetector::<SeededClock>::new();
    let id = Uuid::from_u128(1);

    for i in 0..SPIRAL_THRESHOLD as i64 {
        let r = det.record_step(id, "observe", "debug", "looping", i);
        assert_eq!(r, SpiralResult::Clean, "step {i} must be Clean");
    }

    let escalate = det.record_step(
        id,
        "observe",
        "debug",
        "looping",
        SPIRAL_THRESHOLD as i64,
    );
    match escalate {
        SpiralResult::EscalationSignalNeeded {
            message,
            spiral_depth,
        } => {
            assert_eq!(
                message, ESCALATION_MESSAGE,
                "runner injects this exact string into the next agent turn"
            );
            assert_eq!(spiral_depth, ESCALATION_DEPTH);
        }
        other => panic!("expected EscalationSignalNeeded, got {other:?}"),
    }
}

// Contract 3: After escalation, one more identical step triggers HardStop
// with HARD_STOP_DEPTH (this is the spiral_depth value written into the
// SpiralDetected SSB row by the runner).
#[test]
fn contract_hard_stop_spiral_depth_matches_constant() {
    let mut det = AntiSpiralDetector::<SeededClock>::new();
    let id = Uuid::from_u128(2);

    for i in 0..=SPIRAL_THRESHOLD as i64 {
        det.record_step(id, "observe", "debug", "stuck", i);
    }
    let hard_stop = det.record_step(
        id,
        "observe",
        "debug",
        "stuck",
        (SPIRAL_THRESHOLD as i64) + 1,
    );
    match hard_stop {
        SpiralResult::HardStop { spiral_depth } => {
            assert_eq!(spiral_depth, HARD_STOP_DEPTH);
        }
        other => panic!("expected HardStop, got {other:?}"),
    }
}

// Contract 4: SpiralDetected payload shape is stable — the runner builds it
// from `(agent_id.to_string(), phase_id, HARD_STOP_DEPTH)`. If any of those
// fields rename or move the dashboard filter breaks.
#[test]
fn contract_spiral_detected_payload_shape() {
    let payload = SpiralDetectedPayload {
        agent_id: "task_runner".into(),
        phase_id: "task_seq_42".into(),
        spiral_depth: HARD_STOP_DEPTH,
    };
    let wrapped = SsbPayload::SpiralDetected(payload.clone());
    match wrapped {
        SsbPayload::SpiralDetected(p) => {
            assert_eq!(p.agent_id, "task_runner");
            assert_eq!(p.phase_id, "task_seq_42");
            assert_eq!(p.spiral_depth, HARD_STOP_DEPTH);
        }
        _ => unreachable!(),
    }
    // JSON round-trip proves the payload is serde-stable for SSB persistence.
    let json = serde_json::to_string(&payload).unwrap();
    let back: SpiralDetectedPayload = serde_json::from_str(&json).unwrap();
    assert_eq!(back.spiral_depth, HARD_STOP_DEPTH);
}

// Contract 5: SPIRAL_HARDSTOP_CATEGORY is the exact string the dashboard
// filter keys on for hard-stop Findings.
#[test]
fn contract_spiral_hardstop_category_is_stable() {
    assert_eq!(SPIRAL_HARDSTOP_CATEGORY, "spiral_hardstop");
}

// Contract 6: Independent agents are tracked independently — the runner keys
// SpiralTracker per task seq so cross-task state never leaks.
#[test]
fn contract_independent_agent_tracking() {
    let mut det = AntiSpiralDetector::<SeededClock>::new();
    let a = Uuid::from_u128(10);
    let b = Uuid::from_u128(11);

    for i in 0..=SPIRAL_THRESHOLD as i64 {
        det.record_step(a, "observe", "debug", "a-stuck", i);
    }
    let a_hard = det.record_step(
        a,
        "observe",
        "debug",
        "a-stuck",
        (SPIRAL_THRESHOLD as i64) + 1,
    );
    assert!(matches!(a_hard, SpiralResult::HardStop { .. }));

    // Agent B's first step on the same content is Clean — a different agent_id
    // key.
    let b_first = det.record_step(b, "observe", "debug", "a-stuck", 0);
    assert_eq!(b_first, SpiralResult::Clean);
}
