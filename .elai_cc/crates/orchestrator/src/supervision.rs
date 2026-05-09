// Supervision tree: policy layer above RecoveryManager.
// Tracks per-agent restart counts within phase windows and escalates when
// the configured limit is hit. Prevents infinite restart loops.
//
// Production wiring follow-up: RecoveryManager is wired in 3C.REDO (commit
// 893e69f), but the supervision layer on top of it is not yet plugged into
// task_runner. Test callers (contracts/tests/supervision_contract.rs and
// in-file `#[cfg(test)]` modules) validate the escalation-policy contract.
// Follow-up session will wire `SupervisionManager::record_restart` into the
// RecoveryManager failure path and emit `SsbPayload::SupervisionExhausted`
// on Terminate escalation.

use rustc_hash::FxHashMap;
use std::time::Duration;

use uuid::Uuid;

use elai_core::clock::{ClockProvider, LogicalInstant};

use crate::recovery::AgentDeathClassification;

// ---------------------------------------------------------------------------
// Constants
// ---------------------------------------------------------------------------

// Minimum allowed value for max_restarts. Values below this are clamped.
const MAX_RESTART_CLAMP_MIN: u32 = 1;

// ---------------------------------------------------------------------------
// EscalationAction
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EscalationAction {
    Terminate,
    SubstituteModel(String),
    ReduceScope,
}

impl EscalationAction {
    // Used by callers constructing SupervisionExhaustedPayload (3C.6).
    #[allow(dead_code)]
    fn as_str(&self) -> &'static str {
        match self {
            Self::Terminate => "terminate",
            Self::SubstituteModel(_) => "substitute_model",
            Self::ReduceScope => "reduce_scope",
        }
    }
}

// ---------------------------------------------------------------------------
// SupervisionPolicy
// ---------------------------------------------------------------------------

#[derive(Debug, Clone)]
pub struct SupervisionPolicy {
    pub max_restarts: u32,
    pub restart_window: Duration,
    pub on_max_restarts: EscalationAction,
}

// ---------------------------------------------------------------------------
// SupervisionState (per-agent tracking)
// ---------------------------------------------------------------------------

#[derive(Debug, Clone)]
struct SupervisionState {
    restart_count: u32,
    window_start: LogicalInstant,
    phase_id: String,
    restart_history: Vec<i64>,
}

// ---------------------------------------------------------------------------
// SupervisionOutcome
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SupervisionOutcome {
    // Agent was restarted; current restart count returned.
    Restarted { restart_count: u32 },
    // Restart limit exhausted; escalation action and metadata returned.
    Escalated {
        restart_count: u32,
        max_restarts: u32,
        escalation_action: EscalationAction,
        restart_history: Vec<i64>,
    },
    // Clean exit — no restart needed.
    NoRestartNeeded,
    // Budget exceeded — convergence injected, restart count unchanged.
    ConvergenceInjected,
}

// ---------------------------------------------------------------------------
// SupervisionError
// ---------------------------------------------------------------------------

#[derive(Debug, thiserror::Error)]
pub enum SupervisionError {
    #[error("SubstituteModel escalation is not permitted in interactive mode")]
    SubstituteModelInteractiveGate,

    #[error("supervision invariant violated: {0}")]
    InvariantViolation(String),
}

// ---------------------------------------------------------------------------
// SupervisionManager<C: ClockProvider>
// ---------------------------------------------------------------------------

pub struct SupervisionManager<C: ClockProvider> {
    clock: C,
    policy: SupervisionPolicy,
    states: FxHashMap<Uuid, SupervisionState>,
    effective_max_restarts: u32,
}

impl<C: ClockProvider> SupervisionManager<C> {
    pub fn new(clock: C, policy: SupervisionPolicy) -> Self {
        let effective_max_restarts = if policy.max_restarts < MAX_RESTART_CLAMP_MIN {
            tracing::warn!(
                configured = policy.max_restarts,
                clamped_to = MAX_RESTART_CLAMP_MIN,
                "max_restarts below minimum, clamping"
            );
            MAX_RESTART_CLAMP_MIN
        } else {
            policy.max_restarts
        };
        Self {
            clock,
            policy,
            states: FxHashMap::default(),
            effective_max_restarts,
        }
    }

    // Handle an agent death. Returns the supervision outcome or an error
    // if the escalation action is gated (e.g., SubstituteModel in interactive mode).
    //
    // This method is synchronous — it does not write to SSB. The caller is
    // responsible for writing supervision_exhausted payloads on Escalated outcome.
    pub fn handle_death(
        &mut self,
        agent_id: Uuid,
        classification: AgentDeathClassification,
        phase_id: &str,
        step_index: i64,
        interactive_mode: bool,
    ) -> Result<SupervisionOutcome, SupervisionError> {
        // Clean exit: no restart needed, do not touch restart state.
        if classification == AgentDeathClassification::CleanExit {
            return Ok(SupervisionOutcome::NoRestartNeeded);
        }

        // Budget exceeded: inject convergence, do NOT increment restart count.
        if classification == AgentDeathClassification::BudgetExceeded {
            return Ok(SupervisionOutcome::ConvergenceInjected);
        }

        // Crash or Timeout: increment restart count and check threshold.
        let now = self.clock.now();
        let state = self.states.entry(agent_id).or_insert_with(|| SupervisionState {
            restart_count: 0,
            window_start: now,
            phase_id: phase_id.to_owned(),
            restart_history: Vec::new(),
        });

        // If the phase changed, reset (handles cross-phase transitions that
        // didn't go through reset_phase).
        if state.phase_id != phase_id {
            state.restart_count = 0;
            state.window_start = now;
            state.phase_id = phase_id.to_owned();
            state.restart_history.clear();
        }

        // Window expiry: if elapsed > restart_window, reset count and window.
        let elapsed = self.clock.elapsed_since(state.window_start);
        if elapsed > self.policy.restart_window {
            state.restart_count = 0;
            state.window_start = now;
            state.restart_history.clear();
        }

        // Increment restart count (overflow-safe).
        state.restart_count = state.restart_count.checked_add(1).ok_or_else(|| {
            SupervisionError::InvariantViolation(
                "restart_count overflow at u32::MAX".into(),
            )
        })?;
        state.restart_history.push(step_index);

        // Check if restart limit is exhausted.
        if state.restart_count > self.effective_max_restarts {
            // Gate: SubstituteModel not allowed in interactive mode.
            if interactive_mode {
                if let EscalationAction::SubstituteModel(_) = &self.policy.on_max_restarts {
                    return Err(SupervisionError::SubstituteModelInteractiveGate);
                }
            }

            let outcome = SupervisionOutcome::Escalated {
                restart_count: state.restart_count,
                max_restarts: self.effective_max_restarts,
                escalation_action: self.policy.on_max_restarts.clone(),
                restart_history: state.restart_history.clone(),
            };
            return Ok(outcome);
        }

        Ok(SupervisionOutcome::Restarted {
            restart_count: state.restart_count,
        })
    }

    // Reset all tracked agent states for a phase boundary.
    // After this call, all agents start with restart_count=0 in the next phase.
    pub fn reset_phase(&mut self) {
        self.states.clear();
    }

    // Returns the current restart count for an agent (0 if untracked).
    pub fn restart_count(&self, agent_id: &Uuid) -> u32 {
        self.states.get(agent_id).map_or(0, |s| s.restart_count)
    }

    // Returns the effective max_restarts (after clamping).
    pub fn effective_max_restarts(&self) -> u32 {
        self.effective_max_restarts
    }

    // Returns the escalation action string for SSB payload use.
    // Used by orchestrator loop constructing SSB payloads (3C.6).
    #[allow(dead_code)]
    pub(crate) fn escalation_action_str(&self) -> &'static str {
        self.policy.on_max_restarts.as_str()
    }
}

// ===========================================================================
// Tests
// ===========================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use elai_core::clock::SeededClock;

    fn default_policy(max_restarts: u32) -> SupervisionPolicy {
        SupervisionPolicy {
            max_restarts,
            restart_window: Duration::from_secs(300),
            on_max_restarts: EscalationAction::Terminate,
        }
    }

    fn make_manager(max_restarts: u32) -> SupervisionManager<SeededClock> {
        SupervisionManager::new(SeededClock::new(), default_policy(max_restarts))
    }

    // -----------------------------------------------------------------------
    // DETERMINISTIC_CHECK: 3 crashes -> 3 restarts -> 4th crash -> Escalate
    // -----------------------------------------------------------------------

    #[test]
    fn three_crashes_then_escalate() {
        let mut mgr = make_manager(3);
        let id = Uuid::nil();

        for i in 1..=3 {
            let outcome = mgr
                .handle_death(id, AgentDeathClassification::Crash, "phase-1", i, false)
                .unwrap();
            assert_eq!(outcome, SupervisionOutcome::Restarted { restart_count: i as u32 });
        }

        let outcome = mgr
            .handle_death(id, AgentDeathClassification::Crash, "phase-1", 4, false)
            .unwrap();
        match outcome {
            SupervisionOutcome::Escalated {
                restart_count,
                max_restarts,
                escalation_action,
                ..
            } => {
                assert_eq!(restart_count, 4);
                assert_eq!(max_restarts, 3);
                assert_eq!(escalation_action, EscalationAction::Terminate);
            }
            other => panic!("expected Escalated, got {other:?}"),
        }
    }

    // -----------------------------------------------------------------------
    // DETERMINISTIC_CHECK: phase boundary resets restart count
    // -----------------------------------------------------------------------

    #[test]
    fn phase_boundary_resets_count() {
        let mut mgr = make_manager(3);
        let id = Uuid::nil();

        // 2 crashes in phase-1
        mgr.handle_death(id, AgentDeathClassification::Crash, "phase-1", 1, false)
            .unwrap();
        mgr.handle_death(id, AgentDeathClassification::Crash, "phase-1", 2, false)
            .unwrap();
        assert_eq!(mgr.restart_count(&id), 2);

        // Phase boundary
        mgr.reset_phase();
        assert_eq!(mgr.restart_count(&id), 0);

        // 1 crash in phase-2 — should restart, not escalate
        let outcome = mgr
            .handle_death(id, AgentDeathClassification::Crash, "phase-2", 3, false)
            .unwrap();
        assert_eq!(outcome, SupervisionOutcome::Restarted { restart_count: 1 });
    }

    // -----------------------------------------------------------------------
    // DETERMINISTIC_CHECK: max_restarts=1 -> 2nd crash -> Escalate
    // -----------------------------------------------------------------------

    #[test]
    fn max_restarts_one_escalates_on_second() {
        let mut mgr = make_manager(1);
        let id = Uuid::nil();

        let r1 = mgr
            .handle_death(id, AgentDeathClassification::Crash, "p", 1, false)
            .unwrap();
        assert_eq!(r1, SupervisionOutcome::Restarted { restart_count: 1 });

        let r2 = mgr
            .handle_death(id, AgentDeathClassification::Crash, "p", 2, false)
            .unwrap();
        match r2 {
            SupervisionOutcome::Escalated { restart_count, .. } => {
                assert_eq!(restart_count, 2);
            }
            other => panic!("expected Escalated, got {other:?}"),
        }
    }

    // -----------------------------------------------------------------------
    // DETERMINISTIC_CHECK: BUDGET_EXCEEDED does NOT increment restart_count
    // -----------------------------------------------------------------------

    #[test]
    fn budget_exceeded_no_restart_increment() {
        let mut mgr = make_manager(3);
        let id = Uuid::nil();

        // One crash first
        mgr.handle_death(id, AgentDeathClassification::Crash, "p", 1, false)
            .unwrap();
        assert_eq!(mgr.restart_count(&id), 1);

        // Budget exceeded — count must NOT change
        let outcome = mgr
            .handle_death(id, AgentDeathClassification::BudgetExceeded, "p", 2, false)
            .unwrap();
        assert_eq!(outcome, SupervisionOutcome::ConvergenceInjected);
        assert_eq!(mgr.restart_count(&id), 1);
    }

    // -----------------------------------------------------------------------
    // DETERMINISTIC_CHECK: SubstituteModel in INTERACTIVE -> error
    // -----------------------------------------------------------------------

    #[test]
    fn substitute_model_interactive_gate() {
        let policy = SupervisionPolicy {
            max_restarts: 1,
            restart_window: Duration::from_secs(300),
            on_max_restarts: EscalationAction::SubstituteModel("fallback-7b".into()),
        };
        let mut mgr = SupervisionManager::new(SeededClock::new(), policy);
        let id = Uuid::nil();

        // First crash: restart
        mgr.handle_death(id, AgentDeathClassification::Crash, "p", 1, true)
            .unwrap();

        // Second crash: should fail with interactive gate error
        let err = mgr
            .handle_death(id, AgentDeathClassification::Crash, "p", 2, true)
            .unwrap_err();
        assert!(
            matches!(err, SupervisionError::SubstituteModelInteractiveGate),
            "expected SubstituteModelInteractiveGate, got {err:?}"
        );
    }

    // -----------------------------------------------------------------------
    // ADVERSARIAL: restart_count == 0 after phase transition
    // -----------------------------------------------------------------------

    #[test]
    fn restart_count_zero_after_phase_transition() {
        let mut mgr = make_manager(5);
        let id = Uuid::nil();

        for i in 1..=3 {
            mgr.handle_death(id, AgentDeathClassification::Crash, "old", i, false)
                .unwrap();
        }
        assert_eq!(mgr.restart_count(&id), 3);

        mgr.reset_phase();
        assert_eq!(mgr.restart_count(&id), 0);
    }

    // -----------------------------------------------------------------------
    // ADVERSARIAL: max_restarts < 1 -> clamp to 1 with WARNING
    // -----------------------------------------------------------------------

    #[test]
    fn max_restarts_zero_clamped_to_one() {
        let mgr = make_manager(0);
        assert_eq!(mgr.effective_max_restarts(), 1);
    }

    // -----------------------------------------------------------------------
    // Clean exit -> NoRestartNeeded
    // -----------------------------------------------------------------------

    #[test]
    fn clean_exit_no_restart() {
        let mut mgr = make_manager(3);
        let id = Uuid::nil();

        let outcome = mgr
            .handle_death(id, AgentDeathClassification::CleanExit, "p", 1, false)
            .unwrap();
        assert_eq!(outcome, SupervisionOutcome::NoRestartNeeded);
        assert_eq!(mgr.restart_count(&id), 0);
    }

    // -----------------------------------------------------------------------
    // Timeout treated same as Crash
    // -----------------------------------------------------------------------

    #[test]
    fn timeout_increments_like_crash() {
        let mut mgr = make_manager(3);
        let id = Uuid::nil();

        let outcome = mgr
            .handle_death(id, AgentDeathClassification::Timeout, "p", 1, false)
            .unwrap();
        assert_eq!(outcome, SupervisionOutcome::Restarted { restart_count: 1 });
    }

    // -----------------------------------------------------------------------
    // Window expiry resets restart count
    // -----------------------------------------------------------------------

    #[test]
    fn window_expiry_resets_count() {
        let clock = SeededClock::new();
        let policy = SupervisionPolicy {
            max_restarts: 2,
            // 5ms window — SeededClock ticks at 1ms per now() call
            restart_window: Duration::from_millis(5),
            on_max_restarts: EscalationAction::Terminate,
        };
        let mut mgr = SupervisionManager::new(clock, policy);
        let id = Uuid::nil();

        // Two crashes (ticks: 0, 1 for now() calls inside handle_death
        // plus the initial or_insert now() call)
        mgr.handle_death(id, AgentDeathClassification::Crash, "p", 1, false)
            .unwrap();
        mgr.handle_death(id, AgentDeathClassification::Crash, "p", 2, false)
            .unwrap();
        assert_eq!(mgr.restart_count(&id), 2);

        // Burn clock ticks to exceed the 5ms window.
        // Each now() call advances tick by 1. We need elapsed > 5ms.
        // Current tick is around 2-4 depending on internal calls.
        // Call now() 10 times to guarantee we pass the window.
        for _ in 0..10 {
            let _ = mgr.clock.now();
        }

        // Next crash should see expired window -> reset count -> restart_count = 1
        let outcome = mgr
            .handle_death(id, AgentDeathClassification::Crash, "p", 3, false)
            .unwrap();
        assert_eq!(outcome, SupervisionOutcome::Restarted { restart_count: 1 });
    }

    // -----------------------------------------------------------------------
    // Multiple agents tracked independently
    // -----------------------------------------------------------------------

    #[test]
    fn multiple_agents_independent() {
        let mut mgr = make_manager(2);
        let id_a = Uuid::from_u128(1);
        let id_b = Uuid::from_u128(2);

        mgr.handle_death(id_a, AgentDeathClassification::Crash, "p", 1, false)
            .unwrap();
        mgr.handle_death(id_a, AgentDeathClassification::Crash, "p", 2, false)
            .unwrap();
        assert_eq!(mgr.restart_count(&id_a), 2);
        assert_eq!(mgr.restart_count(&id_b), 0);

        mgr.handle_death(id_b, AgentDeathClassification::Crash, "p", 3, false)
            .unwrap();
        assert_eq!(mgr.restart_count(&id_b), 1);
        // id_a unchanged
        assert_eq!(mgr.restart_count(&id_a), 2);
    }

    // -----------------------------------------------------------------------
    // SubstituteModel works in non-interactive mode
    // -----------------------------------------------------------------------

    #[test]
    fn substitute_model_non_interactive_ok() {
        let policy = SupervisionPolicy {
            max_restarts: 1,
            restart_window: Duration::from_secs(300),
            on_max_restarts: EscalationAction::SubstituteModel("fallback-7b".into()),
        };
        let mut mgr = SupervisionManager::new(SeededClock::new(), policy);
        let id = Uuid::nil();

        mgr.handle_death(id, AgentDeathClassification::Crash, "p", 1, false)
            .unwrap();
        let outcome = mgr
            .handle_death(id, AgentDeathClassification::Crash, "p", 2, false)
            .unwrap();
        match outcome {
            SupervisionOutcome::Escalated {
                escalation_action, ..
            } => {
                assert_eq!(
                    escalation_action,
                    EscalationAction::SubstituteModel("fallback-7b".into())
                );
            }
            other => panic!("expected Escalated, got {other:?}"),
        }
    }

    // -----------------------------------------------------------------------
    // Escalated outcome contains restart_history
    // -----------------------------------------------------------------------

    #[test]
    fn escalated_includes_restart_history() {
        let mut mgr = make_manager(2);
        let id = Uuid::nil();

        mgr.handle_death(id, AgentDeathClassification::Crash, "p", 10, false)
            .unwrap();
        mgr.handle_death(id, AgentDeathClassification::Crash, "p", 20, false)
            .unwrap();
        let outcome = mgr
            .handle_death(id, AgentDeathClassification::Crash, "p", 30, false)
            .unwrap();
        match outcome {
            SupervisionOutcome::Escalated {
                restart_history, ..
            } => {
                assert_eq!(restart_history, vec![10, 20, 30]);
            }
            other => panic!("expected Escalated, got {other:?}"),
        }
    }

    // -----------------------------------------------------------------------
    // Phase ID change without explicit reset_phase still resets
    // -----------------------------------------------------------------------

    #[test]
    fn phase_id_change_implicit_reset() {
        let mut mgr = make_manager(3);
        let id = Uuid::nil();

        mgr.handle_death(id, AgentDeathClassification::Crash, "phase-1", 1, false)
            .unwrap();
        mgr.handle_death(id, AgentDeathClassification::Crash, "phase-1", 2, false)
            .unwrap();
        assert_eq!(mgr.restart_count(&id), 2);

        // Different phase_id — should implicitly reset
        let outcome = mgr
            .handle_death(id, AgentDeathClassification::Crash, "phase-2", 3, false)
            .unwrap();
        assert_eq!(outcome, SupervisionOutcome::Restarted { restart_count: 1 });
    }
}
