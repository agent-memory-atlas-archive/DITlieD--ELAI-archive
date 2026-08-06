//! J-93 LEG-A: deterministic default-OFF OBSERVE-ONLY in-loop MAST supervisor.
//!
//! Pattern-matches the live tool-call stream for:
//!   * FM-1.3 / StuckLoop: same (tool_name, action_type) repeated >= 3 times
//!   * FM-2.6: tool not in stated_plan_tools when that set is non-empty
//!
//! No LLM. Pure deterministic matching. Flag default OFF -> always Observed;
//! flag ON -> Warned only (never hard-abort).

use std::collections::BTreeSet;

pub const ELAI_MAST_INLOOP_SUPERVISOR_ENABLE: &str = "ELAI_MAST_INLOOP_SUPERVISOR_ENABLE";
pub const REPETITION_THRESHOLD: usize = 3;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum InloopMode {
    Fm13StepRepetition,
    Fm26ReasoningActionMismatch,
    StuckLoop,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum InloopGateOutcome {
    Observed,
    Warned,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InloopFinding {
    pub mode: InloopMode,
    pub tool_name: String,
    pub action_type: String,
    pub repetition_count: usize,
}

/// Deterministic in-loop MAST supervisor. Holds the stated plan tool set
/// (for FM-2.6 mismatch) and the last (tool_name, action_type) fingerprint
/// with its running count (for FM-1.3 repetition).
pub struct InloopSupervisor {
    stated_plan_tools: BTreeSet<String>,
    last_fingerprint: Option<(String, String)>,
    last_count: usize,
}

impl InloopSupervisor {
    pub fn new(stated_plan_tools: impl IntoIterator<Item = String>) -> Self {
        Self {
            stated_plan_tools: stated_plan_tools.into_iter().collect(),
            last_fingerprint: None,
            last_count: 0,
        }
    }

    /// Observe one tool call. Returns `Some(InloopFinding)` when a pattern
    /// fires (FM-1.3 repetition at threshold, or FM-2.6 plan mismatch),
    /// `None` otherwise. FM-2.6 is skipped when the plan set is empty.
    pub fn observe_tool_call(&mut self, tool_name: &str, action_type: &str) -> Option<InloopFinding> {
        // FM-2.6: plan mismatch (only when the stated plan set is non-empty).
        if !self.stated_plan_tools.is_empty()
            && !self.stated_plan_tools.contains(tool_name)
        {
            return Some(InloopFinding {
                mode: InloopMode::Fm26ReasoningActionMismatch,
                tool_name: tool_name.to_owned(),
                action_type: action_type.to_owned(),
                repetition_count: 1,
            });
        }

        // FM-1.3 / StuckLoop: same (tool_name, action_type) repeated >= threshold.
        let fingerprint = (tool_name.to_owned(), action_type.to_owned());
        if let Some(ref last) = self.last_fingerprint {
            if *last == fingerprint {
                self.last_count += 1;
            } else {
                self.last_fingerprint = Some(fingerprint.clone());
                self.last_count = 1;
            }
        } else {
            self.last_fingerprint = Some(fingerprint.clone());
            self.last_count = 1;
        }

        if self.last_count >= REPETITION_THRESHOLD {
            return Some(InloopFinding {
                mode: InloopMode::Fm13StepRepetition,
                tool_name: fingerprint.0,
                action_type: fingerprint.1,
                repetition_count: self.last_count,
            });
        }

        None
    }
}

/// Public accessor for the in-loop MAST supervisor kill-switch. Default OFF.
pub fn mast_inloop_supervisor_enabled() -> bool {
    std::env::var(ELAI_MAST_INLOOP_SUPERVISOR_ENABLE)
        .map(|v| !v.trim().is_empty() && v != "0")
        .unwrap_or(false)
}

/// Gate outcome for a finding: `Observed` when the flag is OFF (default),
/// `Warned` when the flag is ON. Never hard-aborts.
pub fn gate_outcome_for(_finding: &InloopFinding) -> InloopGateOutcome {
    if mast_inloop_supervisor_enabled() {
        InloopGateOutcome::Warned
    } else {
        InloopGateOutcome::Observed
    }
}

#[cfg(test)]
mod j93_mast_inloop_tests {
    use super::*;

    #[test]
    fn flag_off_observes_dont_block() {
        std::env::remove_var(ELAI_MAST_INLOOP_SUPERVISOR_ENABLE);
        let mut supervisor = InloopSupervisor::new(vec!["edit".to_owned()]);
        let finding = supervisor
            .observe_tool_call("bash", "tool_call")
            .expect("plan mismatch must produce FM-2.6");
        assert_eq!(finding.mode, InloopMode::Fm26ReasoningActionMismatch);
        assert_eq!(gate_outcome_for(&finding), InloopGateOutcome::Observed);
    }

    #[test]
    fn repetition_fires_at_threshold() {
        let mut supervisor = InloopSupervisor::new(Vec::new());
        assert!(supervisor.observe_tool_call("edit", "tool_call").is_none());
        assert!(supervisor.observe_tool_call("edit", "tool_call").is_none());
        assert_eq!(
            supervisor.observe_tool_call("edit", "tool_call").unwrap().mode,
            InloopMode::Fm13StepRepetition
        );
    }

    #[test]
    fn repetition_below_threshold_silent() {
        let mut supervisor = InloopSupervisor::new(Vec::new());
        assert!(supervisor.observe_tool_call("edit", "tool_call").is_none());
        assert!(supervisor.observe_tool_call("edit", "tool_call").is_none());
    }

    #[test]
    fn fm26_plan_mismatch() {
        let mut supervisor = InloopSupervisor::new(vec!["edit".to_owned()]);
        assert_eq!(
            supervisor.observe_tool_call("bash", "tool_call").unwrap().mode,
            InloopMode::Fm26ReasoningActionMismatch
        );
    }

    #[test]
    fn fm26_empty_plan_skips() {
        let mut supervisor = InloopSupervisor::new(Vec::new());
        assert!(supervisor.observe_tool_call("bash", "tool_call").is_none());
    }

    #[test]
    fn enabled_accessor_reads_env() {
        std::env::set_var(ELAI_MAST_INLOOP_SUPERVISOR_ENABLE, "1");
        assert!(mast_inloop_supervisor_enabled());
        std::env::remove_var(ELAI_MAST_INLOOP_SUPERVISOR_ENABLE);
    }
}
