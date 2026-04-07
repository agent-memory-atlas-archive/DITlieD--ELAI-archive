// Contract tests for the supervision module.
// These verify interface-level guarantees that downstream code depends on.

use std::time::Duration;

use elai_core::SeededClock;
use elai_orchestrator::recovery::AgentDeathClassification;
use elai_orchestrator::supervision::{
    EscalationAction, SupervisionError, SupervisionManager, SupervisionOutcome, SupervisionPolicy,
};
use uuid::Uuid;

// Contract 1: BudgetExceeded does NOT increment restart_count.
// Downstream relies on this: convergence injection must not consume a restart slot.
#[test]
fn contract_budget_exceeded_no_restart_increment() {
    let policy = SupervisionPolicy {
        max_restarts: 3,
        restart_window: Duration::from_secs(300),
        on_max_restarts: EscalationAction::Terminate,
    };
    let mut mgr = SupervisionManager::new(SeededClock::new(), policy);
    let id = Uuid::nil();

    // One crash to establish a non-zero restart count.
    mgr.handle_death(id, AgentDeathClassification::Crash, "p", 1, false)
        .unwrap();
    let count_before = mgr.restart_count(&id);

    // BudgetExceeded must return ConvergenceInjected and leave count unchanged.
    let outcome = mgr
        .handle_death(id, AgentDeathClassification::BudgetExceeded, "p", 2, false)
        .unwrap();
    assert_eq!(outcome, SupervisionOutcome::ConvergenceInjected);
    assert_eq!(mgr.restart_count(&id), count_before);
}

// Contract 2: restart_count resets to 0 at phase boundary (reset_phase).
// Downstream relies on this: agents must get a fresh restart budget per phase.
#[test]
fn contract_restart_count_resets_at_phase_boundary() {
    let policy = SupervisionPolicy {
        max_restarts: 5,
        restart_window: Duration::from_secs(300),
        on_max_restarts: EscalationAction::Terminate,
    };
    let mut mgr = SupervisionManager::new(SeededClock::new(), policy);
    let id = Uuid::nil();

    // Accumulate restarts.
    for i in 1..=3 {
        mgr.handle_death(id, AgentDeathClassification::Crash, "phase-a", i, false)
            .unwrap();
    }
    assert!(mgr.restart_count(&id) > 0);

    // Phase boundary — must reset to 0.
    mgr.reset_phase();
    assert_eq!(mgr.restart_count(&id), 0);
}

// Contract 3: SubstituteModel in INTERACTIVE mode returns Err.
// Downstream relies on this: interactive sessions must never silently swap models.
#[test]
fn contract_substitute_model_interactive_returns_err() {
    let policy = SupervisionPolicy {
        max_restarts: 1,
        restart_window: Duration::from_secs(300),
        on_max_restarts: EscalationAction::SubstituteModel("fallback-7b".into()),
    };
    let mut mgr = SupervisionManager::new(SeededClock::new(), policy);
    let id = Uuid::nil();

    // First crash: restart (within budget).
    mgr.handle_death(id, AgentDeathClassification::Crash, "p", 1, true)
        .unwrap();

    // Second crash: escalation triggered, but SubstituteModel in interactive = error.
    let result = mgr.handle_death(id, AgentDeathClassification::Crash, "p", 2, true);
    assert!(result.is_err());
    assert!(matches!(
        result.unwrap_err(),
        SupervisionError::SubstituteModelInteractiveGate
    ));
}
