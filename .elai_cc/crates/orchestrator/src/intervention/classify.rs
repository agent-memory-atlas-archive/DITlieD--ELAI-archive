// J-12 H6: offline intervention failure taxonomy (Appendix A.1 priority) +
// regression-check harness (Appendix A.2). Pure, LLM-free, never panics.
// I-12 remains the promotion owner; this module is the intake taxonomy J-12 ships.

/// Appendix A.1 priority order (mutually exclusive; checked FIRST->LAST).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum InterventionFailureClass {
    /// Priority 1: non-executable tool_calls, invalid name, JSON fail,
    /// missing required args, type errors, dialect/interface violations.
    ActionRealization,
    /// Priority 2: wrong tool for critical steps / env contract mismatch.
    EnvContract,
    /// Priority 3: trajectory degeneration (loops, stalls, budget thrash).
    TrajectoryDegeneration,
    /// Priority 4: residual reasoning (fallback when protocol was followed).
    ResidualReasoning,
}

/// Deterministic signals the offline classifier can inspect.
/// All fields default false/empty; empty signals => ResidualReasoning.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct FailureSignals {
    pub non_executable_tool_call: bool,
    pub invalid_tool_name: bool,
    pub json_parse_failure: bool,
    pub missing_required_arg: bool,
    pub arg_type_error: bool,
    pub interface_violation: bool,
    pub wrong_tool_for_step: bool,
    pub contract_mismatch: bool,
    pub action_loop: bool,
    pub stall: bool,
    pub budget_thrash: bool,
    /// Free-text note for residual (never used for P1-P3 decisions).
    pub note: String,
}

/// Classification outcome + which priority fired.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Classification {
    pub class: InterventionFailureClass,
    /// 1..=4 matching Appendix A.1 priority.
    pub priority: u8,
}

/// Classify using Appendix A.1 priority (first match wins).
/// - P1 ActionRealization if ANY of: non_executable_tool_call, invalid_tool_name,
///   json_parse_failure, missing_required_arg, arg_type_error, interface_violation
/// - P2 EnvContract if ANY of: wrong_tool_for_step, contract_mismatch
/// - P3 TrajectoryDegeneration if ANY of: action_loop, stall, budget_thrash
/// - P4 ResidualReasoning otherwise (including all-false / empty)
/// Pure, LLM-free, never panics.
pub fn classify_intervention_failure(signals: &FailureSignals) -> Classification {
    if signals.non_executable_tool_call
        || signals.invalid_tool_name
        || signals.json_parse_failure
        || signals.missing_required_arg
        || signals.arg_type_error
        || signals.interface_violation
    {
        return Classification {
            class: InterventionFailureClass::ActionRealization,
            priority: 1,
        };
    }
    if signals.wrong_tool_for_step || signals.contract_mismatch {
        return Classification {
            class: InterventionFailureClass::EnvContract,
            priority: 2,
        };
    }
    if signals.action_loop || signals.stall || signals.budget_thrash {
        return Classification {
            class: InterventionFailureClass::TrajectoryDegeneration,
            priority: 3,
        };
    }
    Classification {
        class: InterventionFailureClass::ResidualReasoning,
        priority: 4,
    }
}

/// One proposed harness update candidate from offline authoring.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProposedUpdate {
    pub id: String,
    /// Which failure class this update targets.
    pub targets: InterventionFailureClass,
    /// True if this update might over-trigger (block a valid action).
    pub may_over_trigger: bool,
    /// True if a known valid fixture was blocked by this proposal (hard fail).
    pub blocked_valid_fixture: bool,
}

/// Regression-check result (Appendix A.2 mandatory).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RegressionCheck {
    /// Proposals that pass (may_over_trigger may be true but must NOT have
    /// blocked_valid_fixture).
    pub accepted: Vec<ProposedUpdate>,
    /// Proposals rejected because blocked_valid_fixture == true.
    pub rejected: Vec<ProposedUpdate>,
}

/// Filter proposals: reject any with blocked_valid_fixture.
/// Stable order: sort accepted and rejected by `id` ascending.
/// Empty input => both vecs empty.
/// Pure, LLM-free, never panics.
pub fn regression_check(proposals: &[ProposedUpdate]) -> RegressionCheck {
    let mut accepted: Vec<ProposedUpdate> = Vec::new();
    let mut rejected: Vec<ProposedUpdate> = Vec::new();
    for p in proposals {
        if p.blocked_valid_fixture {
            rejected.push(p.clone());
        } else {
            accepted.push(p.clone());
        }
    }
    accepted.sort_by(|a, b| a.id.cmp(&b.id));
    rejected.sort_by(|a, b| a.id.cmp(&b.id));
    RegressionCheck { accepted, rejected }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn upd(id: &str, blocked_valid: bool, may_over: bool) -> ProposedUpdate {
        ProposedUpdate {
            id: id.to_string(),
            targets: InterventionFailureClass::ActionRealization,
            may_over_trigger: may_over,
            blocked_valid_fixture: blocked_valid,
        }
    }

    #[test]
    fn empty_signals_is_residual() {
        let c = classify_intervention_failure(&FailureSignals::default());
        assert_eq!(c.class, InterventionFailureClass::ResidualReasoning);
        assert_eq!(c.priority, 4);
    }

    #[test]
    fn p1_missing_arg_beats_p2_and_p3() {
        let s = FailureSignals {
            missing_required_arg: true,
            wrong_tool_for_step: true,
            stall: true,
            ..Default::default()
        };
        let c = classify_intervention_failure(&s);
        assert_eq!(c.class, InterventionFailureClass::ActionRealization);
        assert_eq!(c.priority, 1);
    }

    #[test]
    fn p2_when_only_contract() {
        let s = FailureSignals {
            contract_mismatch: true,
            ..Default::default()
        };
        let c = classify_intervention_failure(&s);
        assert_eq!(c.class, InterventionFailureClass::EnvContract);
        assert_eq!(c.priority, 2);
    }

    #[test]
    fn p3_when_only_stall() {
        let s = FailureSignals {
            stall: true,
            ..Default::default()
        };
        let c = classify_intervention_failure(&s);
        assert_eq!(c.class, InterventionFailureClass::TrajectoryDegeneration);
        assert_eq!(c.priority, 3);
    }

    #[test]
    fn p1_json_parse() {
        let s = FailureSignals {
            json_parse_failure: true,
            ..Default::default()
        };
        let c = classify_intervention_failure(&s);
        assert_eq!(c.class, InterventionFailureClass::ActionRealization);
        assert_eq!(c.priority, 1);
    }

    #[test]
    fn regression_rejects_blocked_valid_fixture() {
        let r = regression_check(&[upd("a1", true, false), upd("b2", false, false)]);
        assert_eq!(r.accepted.len(), 1);
        assert_eq!(r.accepted[0].id, "b2");
        assert_eq!(r.rejected.len(), 1);
        assert_eq!(r.rejected[0].id, "a1");
    }

    #[test]
    fn regression_accepts_may_over_trigger_without_blocked_valid() {
        let r = regression_check(&[upd("a1", false, true)]);
        assert_eq!(r.accepted.len(), 1);
        assert!(r.accepted[0].may_over_trigger);
        assert!(r.rejected.is_empty());
    }

    #[test]
    fn regression_empty_input() {
        let r = regression_check(&[]);
        assert!(r.accepted.is_empty());
        assert!(r.rejected.is_empty());
    }

    #[test]
    fn regression_sorts_by_id() {
        let r = regression_check(&[upd("z1", false, false), upd("a1", false, false), upd("m1", true, false)]);
        assert_eq!(r.accepted.len(), 2);
        assert_eq!(r.accepted[0].id, "a1");
        assert_eq!(r.accepted[1].id, "z1");
        assert_eq!(r.rejected.len(), 1);
        assert_eq!(r.rejected[0].id, "m1");
    }
}
