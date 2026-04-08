// Phase 3I.9 measurement harness: times FsmReferenceModel::apply() throughput.
// Run: cargo run --example measure_3i -p elai-orchestrator
// Prints ops_per_sec to stdout for recording in metrics/phase_3I.toml.

use std::time::Instant;

use elai_orchestrator::dst::{FsmEvent, FsmReferenceModel, ReferenceState};
use elai_orchestrator::fsm::FsmState;
use elai_orchestrator::ssb::SsbActionType;

const ITERATIONS: u64 = 10_000;

fn main() {
    // Build a mixed event sequence that exercises multiple apply() code paths.
    // This avoids measuring only the cheapest branch.
    let events: Vec<FsmEvent> = vec![
        FsmEvent::PhaseTransition {
            from: FsmState::Idle,
            to: FsmState::Planning,
        },
        FsmEvent::SsbWrite {
            action_type: SsbActionType::PlanUpdate,
        },
        FsmEvent::PhaseTransition {
            from: FsmState::Planning,
            to: FsmState::TestingFirst,
        },
        FsmEvent::PhaseTransition {
            from: FsmState::TestingFirst,
            to: FsmState::Executing,
        },
        FsmEvent::SuperstepBoundary,
        FsmEvent::SsbWrite {
            action_type: SsbActionType::FileWrite,
        },
        FsmEvent::PhaseTransition {
            from: FsmState::Executing,
            to: FsmState::Auditing,
        },
        FsmEvent::PhaseTransition {
            from: FsmState::Auditing,
            to: FsmState::Memorizing,
        },
        FsmEvent::PhaseTransition {
            from: FsmState::Memorizing,
            to: FsmState::Complete,
        },
        FsmEvent::PhaseTransition {
            from: FsmState::Complete,
            to: FsmState::Idle,
        },
    ];

    let cycle_len = events.len() as u64;

    // Warm-up: one full cycle to avoid cold-start effects.
    let mut state = ReferenceState::initial();
    for event in &events {
        if FsmReferenceModel::preconditions(&state, event) {
            state = FsmReferenceModel::apply(state, event);
        }
    }

    // Timed run: ITERATIONS apply() calls cycling through the event sequence.
    let mut timed_state = ReferenceState::initial();
    let start = Instant::now();
    for i in 0..ITERATIONS {
        let event = &events[(i % cycle_len) as usize];
        if FsmReferenceModel::preconditions(&timed_state, event) {
            timed_state = FsmReferenceModel::apply(timed_state, event);
        }
    }
    let elapsed = start.elapsed();

    let ops_per_sec = ITERATIONS as f64 / elapsed.as_secs_f64();
    println!("iterations: {ITERATIONS}");
    println!("elapsed_ms: {:.2}", elapsed.as_secs_f64() * 1000.0);
    println!("ops_per_sec: {ops_per_sec:.0}");
}
