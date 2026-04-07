// Contract tests for the anti_spiral module.
// These verify interface-level guarantees that downstream code depends on.

use elai_core::SeededClock;
use elai_orchestrator::anti_spiral::{
    AntiSpiralDetector, SpiralResult, ESCALATION_MESSAGE, SPIRAL_THRESHOLD,
};
use uuid::Uuid;

// Contract 1: EscalationSignalNeeded.message matches ESCALATION_MESSAGE constant.
// Downstream relies on this: escalation injection checks the message string.
#[test]
fn contract_escalation_message_matches_constant() {
    let mut det = AntiSpiralDetector::<SeededClock>::new();
    let id = Uuid::nil();

    // Push past threshold: SPIRAL_THRESHOLD + 1 identical steps.
    for i in 0..=(SPIRAL_THRESHOLD as i64) {
        det.record_step(id, "observe", "debug", "same step", i);
    }

    let _result = det.record_step(
        id,
        "observe",
        "debug",
        "same step",
        SPIRAL_THRESHOLD as i64 + 1,
    );

    // The first escalation fires at step SPIRAL_THRESHOLD+1 (same_step_count > SPIRAL_THRESHOLD).
    // But we already iterated SPIRAL_THRESHOLD+1 times (0..=SPIRAL_THRESHOLD), so the last
    // iteration (i = SPIRAL_THRESHOLD) triggered escalation. Verify the *next* step is HardStop
    // or re-check by building from scratch.
    // Actually: steps 0..=SPIRAL_THRESHOLD is SPIRAL_THRESHOLD+1 calls. After step index
    // SPIRAL_THRESHOLD, same_step_count = SPIRAL_THRESHOLD+1 > SPIRAL_THRESHOLD, so escalation
    // fires on that call. The extra record_step above is the 5th call -> HardStop.
    // Fix: only iterate up to threshold, then check.
    let mut det2 = AntiSpiralDetector::<SeededClock>::new();
    let id2 = Uuid::from_u128(42);

    // Feed exactly SPIRAL_THRESHOLD identical steps: same_step_count reaches 3, all Clean.
    for i in 0..SPIRAL_THRESHOLD as i64 {
        det2.record_step(id2, "observe", "debug", "same step", i);
    }

    // The (SPIRAL_THRESHOLD+1)-th call makes same_step_count=4 > 3 -> EscalationSignalNeeded.
    match det2.record_step(
        id2,
        "observe",
        "debug",
        "same step",
        SPIRAL_THRESHOLD as i64,
    ) {
        SpiralResult::EscalationSignalNeeded { message, .. } => {
            assert_eq!(message, ESCALATION_MESSAGE);
        }
        other => panic!(
            "expected EscalationSignalNeeded, got {:?}",
            other
        ),
    }
}

// Contract 2: spiral_depth increments correctly (1 for escalation, 2 for HardStop).
// Downstream relies on this: SSB payload.spiral_depth distinguishes severity.
#[test]
fn contract_spiral_depth_increments() {
    let mut det = AntiSpiralDetector::<SeededClock>::new();
    let id = Uuid::nil();

    // Build to escalation: need SPIRAL_THRESHOLD+1 identical steps.
    // Steps 0..SPIRAL_THRESHOLD are clean (same_step_count 1,2,3).
    // Step SPIRAL_THRESHOLD triggers escalation (same_step_count = 4 > 3).
    for i in 0..SPIRAL_THRESHOLD as i64 {
        let r = det.record_step(id, "observe", "t", "repeated", i);
        assert_eq!(r, SpiralResult::Clean);
    }

    let escalation = det.record_step(id, "observe", "t", "repeated", SPIRAL_THRESHOLD as i64);
    match escalation {
        SpiralResult::EscalationSignalNeeded { spiral_depth, .. } => {
            assert_eq!(spiral_depth, 1);
        }
        other => panic!("expected escalation, got {:?}", other),
    }

    // One more identical step -> HardStop with spiral_depth = 2.
    let hard_stop = det.record_step(
        id,
        "observe",
        "t",
        "repeated",
        SPIRAL_THRESHOLD as i64 + 1,
    );
    match hard_stop {
        SpiralResult::HardStop { spiral_depth } => {
            assert_eq!(spiral_depth, 2);
        }
        other => panic!("expected HardStop, got {:?}", other),
    }
}

// Contract 3: HardStop cannot occur without prior EscalationSignalNeeded.
// Downstream relies on this: escalation gives the agent a chance to recover
// before hard-stopping. A HardStop without warning is a logic bug.
#[test]
fn contract_hard_stop_requires_prior_escalation() {
    let mut det = AntiSpiralDetector::<SeededClock>::new();
    let id = Uuid::nil();

    let mut saw_escalation = false;
    let mut saw_hard_stop = false;

    // Run enough steps to hit HardStop (SPIRAL_THRESHOLD + 2 identical steps).
    for i in 0..(SPIRAL_THRESHOLD as i64 + 2) {
        let result = det.record_step(id, "observe", "t", "same", i);
        match &result {
            SpiralResult::EscalationSignalNeeded { .. } => {
                assert!(
                    !saw_hard_stop,
                    "HardStop must not precede EscalationSignalNeeded"
                );
                saw_escalation = true;
            }
            SpiralResult::HardStop { .. } => {
                assert!(
                    saw_escalation,
                    "HardStop fired without prior EscalationSignalNeeded"
                );
                saw_hard_stop = true;
            }
            SpiralResult::Clean => {}
        }
    }

    assert!(saw_escalation, "escalation should have fired");
    assert!(saw_hard_stop, "hard stop should have fired");
}
