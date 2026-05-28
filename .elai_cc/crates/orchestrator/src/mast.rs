//! E-6 — MastMode: 14-variant taxonomy of multi-agent failure modes.
//!
//! ## Grounding (PRIMARY anchor)
//!
//! Cemri et al. 2025, "Why Do Multi-Agent LLM Systems Fail?" (MAST),
//! arXiv:2503.13657. Cohen's kappa final generalization = 0.79; within-rounds
//! 0.84-0.92. LLM annotator accuracy 94%, precision 0.833, recall 0.77, kappa 0.77.
//! 1600+ traces annotated across 7 multi-agent frameworks (MetaGPT, ChatDev,
//! HyperAgent, AppWorld, AG2, Magentic-One, OpenManus).
//!
//! ## Citation correction
//!
//! The plan originally cited arXiv:2406.12624. Per paper-ingestor triage
//! (workflow/sessions/s-25120/completion-drive/grounding/E-6_multi_agent_failure_modes-grounding.md)
//! 2406.12624 is "Judging the Judges" (LLM-as-judge eval, wrong regime). The
//! ID was an ID_COLLISION. Substituted with arXiv:2503.13657 (MAST) — verified
//! on arxiv.org, matches the 14-mode / 3-category empirical taxonomy described
//! in the plan body. See gotchas.md entry E6_CITATION_2406_12624_ID_COLLISION.
//!
//! ## 14 modes in 3 categories (per MAST Table 1, frequencies)
//!
//! FC1 Specification Issues (41.77%):
//!   FM-1.1 DisobeyTaskSpec (10.98%), FM-1.2 DisobeyRoleSpec (0.50%),
//!   FM-1.3 StepRepetition (17.14%), FM-1.4 LossOfConversationHistory (3.33%),
//!   FM-1.5 UnawareOfTerminationConditions (9.82%).
//! FC2 Inter-Agent Misalignment (36.94%):
//!   FM-2.1 ConversationReset (2.33%), FM-2.2 FailToAskForClarification (11.65%),
//!   FM-2.3 TaskDerailment (7.15%), FM-2.4 InformationWithholding (1.66%),
//!   FM-2.5 IgnoredOtherAgentInput (0.17%), FM-2.6 ReasoningActionMismatch (13.98%).
//! FC3 Task Verification (21.30%):
//!   FM-3.1 PrematureTermination (7.82%), FM-3.2 NoOrIncompleteVerification (6.82%),
//!   FM-3.3 IncorrectVerification (6.66%).

use serde::{Deserialize, Serialize};

// ---------------------------------------------------------------------------
// MastCategory — 3 top-level groupings
// ---------------------------------------------------------------------------

/// Top-level grouping of MastMode variants. Maps 1:1 to the 3 MAST FC
/// categories. `as_str` returns the wire-stable snake_case tag.
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize,
)]
#[serde(rename_all = "snake_case")]
pub enum MastCategory {
    /// FC1 — specification + role + history failures (41.77% of MAST corpus).
    Specification,
    /// FC2 — inter-agent communication + coordination failures (36.94%).
    InterAgent,
    /// FC3 — verification + termination failures (21.30%).
    Verification,
}

impl MastCategory {
    /// Wire-stable label used in SSB rows + dashboard rendering.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Specification => "specification",
            Self::InterAgent => "inter_agent",
            Self::Verification => "verification",
        }
    }

    /// All three categories in canonical (FC1 -> FC2 -> FC3) order.
    pub const ALL: [MastCategory; 3] = [
        Self::Specification,
        Self::InterAgent,
        Self::Verification,
    ];
}

impl std::fmt::Display for MastCategory {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

// ---------------------------------------------------------------------------
// MastMode — 14-variant enum
// ---------------------------------------------------------------------------

/// One of the 14 MAST failure modes per Cemri et al. 2025 Table 1.
///
/// Two-letter prefix encodes the FC category (`Fm1*` = FC1, etc).
/// `as_sql_str` returns the wire-stable snake_case tag emitted into
/// SSB rows; `from_sql_str` is the inverse. `mode_category` projects
/// to the FC grouping.
///
/// Derives `Copy` so `Option<MastMode>` round-trips without `Clone`
/// boilerplate. `PartialOrd` + `Ord` are declaration-order (FM-1.1
/// before FM-1.2, etc) so dashboard counts can render deterministic
/// tables without an explicit sort key.
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize,
)]
#[serde(rename_all = "snake_case")]
pub enum MastMode {
    /// FM-1.1 — agent disobeys the task specification (e.g. wrong output
    /// schema, ignores explicit constraint). MAST freq: 10.98%.
    Fm11DisobeyTaskSpec,
    /// FM-1.2 — agent disobeys its assigned role specification (e.g. an
    /// auditor produces code). MAST freq: 0.50%.
    Fm12DisobeyRoleSpec,
    /// FM-1.3 — agent repeats prior step(s) without progress. Detected by
    /// the existing AntiSpiralDetector + step-content blake3 dedupe. MAST freq: 17.14%.
    Fm13StepRepetition,
    /// FM-1.4 — agent loses earlier conversation history (e.g. exceeds
    /// context budget without checkpoint). MAST freq: 3.33%.
    Fm14LossOfConversationHistory,
    /// FM-1.5 — agent unaware of termination conditions (e.g. continues
    /// after task complete). MAST freq: 9.82%.
    Fm15UnawareOfTerminationConditions,
    /// FM-2.1 — conversation reset / context loss between agent handoffs.
    /// MAST freq: 2.33%.
    Fm21ConversationReset,
    /// FM-2.2 — agent fails to ask for clarification when ambiguous.
    /// MAST freq: 11.65%.
    Fm22FailToAskForClarification,
    /// FM-2.3 — task derailment: agents drift from the original goal.
    /// MAST freq: 7.15%.
    Fm23TaskDerailment,
    /// FM-2.4 — information withholding: agent has relevant info but
    /// does not share with peers. MAST freq: 1.66%.
    Fm24InformationWithholding,
    /// FM-2.5 — agent ignores another agent's input even when relevant.
    /// MAST freq: 0.17% (rarest mode).
    Fm25IgnoredOtherAgentInput,
    /// FM-2.6 — reasoning-action mismatch: agent's stated reasoning
    /// does not match the action taken. MAST freq: 13.98%.
    Fm26ReasoningActionMismatch,
    /// FM-3.1 — premature termination: verifier accepts incomplete result.
    /// MAST freq: 7.82%.
    Fm31PrematureTermination,
    /// FM-3.2 — no or incomplete verification: verifier skips required
    /// checks. MAST freq: 6.82%.
    Fm32NoOrIncompleteVerification,
    /// FM-3.3 — incorrect verification: verifier returns wrong verdict.
    /// MAST freq: 6.66%.
    Fm33IncorrectVerification,
}

impl MastMode {
    /// Wire-stable SQL / SSB tag for this mode. Format: `fm_<major>_<minor>`
    /// where `<major>.<minor>` is the MAST mode number (1.1, 1.2, ..., 3.3).
    /// Stable across releases — adding a new variant requires appending only.
    #[must_use]
    pub const fn as_sql_str(self) -> &'static str {
        match self {
            Self::Fm11DisobeyTaskSpec => "fm_1_1_disobey_task_spec",
            Self::Fm12DisobeyRoleSpec => "fm_1_2_disobey_role_spec",
            Self::Fm13StepRepetition => "fm_1_3_step_repetition",
            Self::Fm14LossOfConversationHistory => "fm_1_4_loss_of_conversation_history",
            Self::Fm15UnawareOfTerminationConditions => {
                "fm_1_5_unaware_of_termination_conditions"
            }
            Self::Fm21ConversationReset => "fm_2_1_conversation_reset",
            Self::Fm22FailToAskForClarification => "fm_2_2_fail_to_ask_for_clarification",
            Self::Fm23TaskDerailment => "fm_2_3_task_derailment",
            Self::Fm24InformationWithholding => "fm_2_4_information_withholding",
            Self::Fm25IgnoredOtherAgentInput => "fm_2_5_ignored_other_agent_input",
            Self::Fm26ReasoningActionMismatch => "fm_2_6_reasoning_action_mismatch",
            Self::Fm31PrematureTermination => "fm_3_1_premature_termination",
            Self::Fm32NoOrIncompleteVerification => "fm_3_2_no_or_incomplete_verification",
            Self::Fm33IncorrectVerification => "fm_3_3_incorrect_verification",
        }
    }

    /// Project this mode to its top-level MAST category.
    #[must_use]
    pub const fn mode_category(self) -> MastCategory {
        match self {
            Self::Fm11DisobeyTaskSpec
            | Self::Fm12DisobeyRoleSpec
            | Self::Fm13StepRepetition
            | Self::Fm14LossOfConversationHistory
            | Self::Fm15UnawareOfTerminationConditions => MastCategory::Specification,
            Self::Fm21ConversationReset
            | Self::Fm22FailToAskForClarification
            | Self::Fm23TaskDerailment
            | Self::Fm24InformationWithholding
            | Self::Fm25IgnoredOtherAgentInput
            | Self::Fm26ReasoningActionMismatch => MastCategory::InterAgent,
            Self::Fm31PrematureTermination
            | Self::Fm32NoOrIncompleteVerification
            | Self::Fm33IncorrectVerification => MastCategory::Verification,
        }
    }

    /// Parse a wire-stable SQL tag back to the enum. Returns `None` for
    /// unknown tags (forward-compat: dashboard receiving a row with an
    /// unknown mode should render a generic "unknown" cell, not panic).
    #[must_use]
    pub fn from_sql_str(s: &str) -> Option<MastMode> {
        Some(match s {
            "fm_1_1_disobey_task_spec" => Self::Fm11DisobeyTaskSpec,
            "fm_1_2_disobey_role_spec" => Self::Fm12DisobeyRoleSpec,
            "fm_1_3_step_repetition" => Self::Fm13StepRepetition,
            "fm_1_4_loss_of_conversation_history" => Self::Fm14LossOfConversationHistory,
            "fm_1_5_unaware_of_termination_conditions" => {
                Self::Fm15UnawareOfTerminationConditions
            }
            "fm_2_1_conversation_reset" => Self::Fm21ConversationReset,
            "fm_2_2_fail_to_ask_for_clarification" => Self::Fm22FailToAskForClarification,
            "fm_2_3_task_derailment" => Self::Fm23TaskDerailment,
            "fm_2_4_information_withholding" => Self::Fm24InformationWithholding,
            "fm_2_5_ignored_other_agent_input" => Self::Fm25IgnoredOtherAgentInput,
            "fm_2_6_reasoning_action_mismatch" => Self::Fm26ReasoningActionMismatch,
            "fm_3_1_premature_termination" => Self::Fm31PrematureTermination,
            "fm_3_2_no_or_incomplete_verification" => Self::Fm32NoOrIncompleteVerification,
            "fm_3_3_incorrect_verification" => Self::Fm33IncorrectVerification,
            _ => return None,
        })
    }

    /// All 14 modes in canonical declaration order (FM-1.1 first, FM-3.3 last).
    /// Used by the dashboard panel + CLI report to enumerate the full grid.
    pub const ALL: [MastMode; 14] = [
        Self::Fm11DisobeyTaskSpec,
        Self::Fm12DisobeyRoleSpec,
        Self::Fm13StepRepetition,
        Self::Fm14LossOfConversationHistory,
        Self::Fm15UnawareOfTerminationConditions,
        Self::Fm21ConversationReset,
        Self::Fm22FailToAskForClarification,
        Self::Fm23TaskDerailment,
        Self::Fm24InformationWithholding,
        Self::Fm25IgnoredOtherAgentInput,
        Self::Fm26ReasoningActionMismatch,
        Self::Fm31PrematureTermination,
        Self::Fm32NoOrIncompleteVerification,
        Self::Fm33IncorrectVerification,
    ];
}

impl std::fmt::Display for MastMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_sql_str())
    }
}

// ---------------------------------------------------------------------------
// finding_to_mast_mode — deterministic attribution from auditor findings
// ---------------------------------------------------------------------------

/// Best-effort attribution of an auditor finding category to a MAST mode.
/// This is a DETERMINISTIC predicate, not a learned classifier — mis-tags
/// are possible. Mitigation: dashboard panel surfaces low-confidence cells
/// and the auditor can override on the next pass. See plan §Open questions / risks.
///
/// `category` is the `Finding::category` string from auditor_mode.rs.
/// `None` means the finding does not map cleanly to any MAST mode; the
/// caller should fall back to recording the verdict without a tag rather
/// than guess.
#[must_use]
pub fn finding_to_mast_mode(category: &str) -> Option<MastMode> {
    // Case-insensitive prefix match on canonical category names from the
    // auditor finding taxonomy. The MAST mode is the "best guess" mapping;
    // a finding category can legitimately match no MAST mode (returns None).
    let c = category.to_ascii_lowercase();
    let c = c.as_str();
    // FC1 — specification
    if c.contains("disobey_task")
        || c.contains("schema_violation")
        || c.contains("missing_required_field")
    {
        return Some(MastMode::Fm11DisobeyTaskSpec);
    }
    if c.contains("disobey_role") || c.contains("role_violation") {
        return Some(MastMode::Fm12DisobeyRoleSpec);
    }
    if c.contains("repetition") || c.contains("spiral") || c.contains("step_repeat") {
        return Some(MastMode::Fm13StepRepetition);
    }
    if c.contains("conversation_history") || c.contains("context_lost") {
        return Some(MastMode::Fm14LossOfConversationHistory);
    }
    if c.contains("termination") || c.contains("unaware_of_end") {
        return Some(MastMode::Fm15UnawareOfTerminationConditions);
    }
    // FC2 — inter-agent
    if c.contains("conversation_reset") {
        return Some(MastMode::Fm21ConversationReset);
    }
    if c.contains("clarification") || c.contains("ambiguous") {
        return Some(MastMode::Fm22FailToAskForClarification);
    }
    if c.contains("derailment") || c.contains("goal_drift") {
        return Some(MastMode::Fm23TaskDerailment);
    }
    if c.contains("information_withholding") || c.contains("withheld") {
        return Some(MastMode::Fm24InformationWithholding);
    }
    if c.contains("ignored_input") || c.contains("ignored_other_agent") {
        return Some(MastMode::Fm25IgnoredOtherAgentInput);
    }
    if c.contains("reasoning_action_mismatch") || c.contains("reasoning_mismatch") {
        return Some(MastMode::Fm26ReasoningActionMismatch);
    }
    // FC3 — verification
    if c.contains("premature_termination") || c.contains("early_termination") {
        return Some(MastMode::Fm31PrematureTermination);
    }
    if c.contains("incomplete_verification") || c.contains("no_verification") {
        return Some(MastMode::Fm32NoOrIncompleteVerification);
    }
    if c.contains("incorrect_verification") || c.contains("wrong_verdict") {
        return Some(MastMode::Fm33IncorrectVerification);
    }
    None
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn all_14_modes_roundtrip_through_sql_str() {
        for mode in MastMode::ALL {
            let s = mode.as_sql_str();
            let parsed = MastMode::from_sql_str(s)
                .unwrap_or_else(|| panic!("mode {mode:?} did not round-trip through {s}"));
            assert_eq!(parsed, mode);
        }
    }

    #[test]
    fn all_14_modes_have_distinct_sql_tags() {
        use std::collections::BTreeSet;
        let mut set = BTreeSet::new();
        for mode in MastMode::ALL {
            assert!(
                set.insert(mode.as_sql_str()),
                "duplicate sql tag for {mode:?}"
            );
        }
        assert_eq!(set.len(), 14);
    }

    #[test]
    fn category_partitions_match_mast_table() {
        // FC1: 5 modes
        let fc1_count = MastMode::ALL
            .iter()
            .filter(|m| m.mode_category() == MastCategory::Specification)
            .count();
        assert_eq!(fc1_count, 5);
        // FC2: 6 modes
        let fc2_count = MastMode::ALL
            .iter()
            .filter(|m| m.mode_category() == MastCategory::InterAgent)
            .count();
        assert_eq!(fc2_count, 6);
        // FC3: 3 modes
        let fc3_count = MastMode::ALL
            .iter()
            .filter(|m| m.mode_category() == MastCategory::Verification)
            .count();
        assert_eq!(fc3_count, 3);
        // sum
        assert_eq!(fc1_count + fc2_count + fc3_count, 14);
    }

    #[test]
    fn from_sql_str_unknown_returns_none() {
        assert!(MastMode::from_sql_str("fm_9_9_not_a_real_mode").is_none());
        assert!(MastMode::from_sql_str("").is_none());
    }

    #[test]
    fn finding_to_mast_mode_known_categories() {
        assert_eq!(
            finding_to_mast_mode("disobey_task_spec"),
            Some(MastMode::Fm11DisobeyTaskSpec)
        );
        assert_eq!(
            finding_to_mast_mode("step_repetition"),
            Some(MastMode::Fm13StepRepetition)
        );
        assert_eq!(
            finding_to_mast_mode("incorrect_verification"),
            Some(MastMode::Fm33IncorrectVerification)
        );
        assert_eq!(finding_to_mast_mode("totally_unrelated"), None);
    }

    #[test]
    fn category_as_str_stable() {
        assert_eq!(MastCategory::Specification.as_str(), "specification");
        assert_eq!(MastCategory::InterAgent.as_str(), "inter_agent");
        assert_eq!(MastCategory::Verification.as_str(), "verification");
    }
}
