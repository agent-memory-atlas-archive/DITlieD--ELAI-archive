//! Admission checks for auditor reasoning distillates.

use crate::chat_reflect::TurnClass;

/// The result of checking whether a proposed distillate may reseed auditing.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DistillateVerdict {
    /// The proposed distillate preserves the required admission properties.
    Admitted,
    /// The proposed distillate contains no non-whitespace characters.
    RejectedEmpty,
    /// The proposed distillate exceeds the configured character ceiling.
    RejectedOversize,
    /// The proposed distillate removed uncertainty present in the prior trace.
    RejectedUncertaintyStripped,
}

/// What armed a distill-reset checkpoint. The deterministic trigger, never a
/// model decision (J-42 plan line 3).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CheckpointTrigger {
    /// Accumulated auditor tokens crossed the configured fraction of the
    /// stage budget cap. The trigger the refine loop uses today.
    ContextFill,
    /// A GRC topic budget (`max_chars` / `max_calls`) neared exhaustion.
    /// Reserved for the `checkpoint_and_reset` operator on `GrcEngine`,
    /// which is a later tranche; no caller selects it yet.
    GrcBudget,
}

impl CheckpointTrigger {
    /// Snake-case wire label.
    #[must_use]
    pub fn as_wire_str(&self) -> &'static str {
        match self {
            Self::ContextFill => "context_fill",
            Self::GrcBudget => "grc_budget",
        }
    }
}

/// What the harness did after the admission check ran.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CheckpointOutcome {
    /// The distillate was admitted; the pass reseeded from the pinned
    /// original question plus that distillate.
    ResetAndContinued,
    /// Admission failed twice; the pass fell back to append-mode refinement.
    DistillateRejected,
    /// The reasoning step escalated to a cloud lane instead of resetting.
    /// Reserved for the I-7 cascade seam, which is a later tranche; no
    /// caller selects it yet.
    EscalatedToCloud,
}

impl CheckpointOutcome {
    /// Snake-case wire label.
    #[must_use]
    pub fn as_wire_str(&self) -> &'static str {
        match self {
            Self::ResetAndContinued => "reset_and_continued",
            Self::DistillateRejected => "distillate_rejected",
            Self::EscalatedToCloud => "escalated_to_cloud",
        }
    }
}

/// Accumulated-token threshold that arms a distill-reset, as a fraction of the
/// stage budget cap. `pct` above 100 collapses to 100; a `pct` of 0 disables the
/// trigger by returning 0.
pub(crate) fn distill_threshold(budget_cap_tokens: u32, pct: u8) -> u32 {
    budget_cap_tokens.saturating_mul(u32::from(pct.min(100))) / 100
}

pub(crate) fn build_distill_prompt(
    user_input: &str,
    class: TurnClass,
    prev_best: u32,
    prev_rationale: &str,
) -> String {
    format!(
        "Compress the audit reasoning for this {class_str} message into at most a few sentences. \
         Keep every hedge, open question, and stated uncertainty; do not resolve or strengthen it.\n\n\
         User message: {user_input}\n\
         Current pick: {prev_best}; reasoning: {prev_rationale}\n\n\
         Distillate:",
        class_str = class.as_wire_str(),
    )
}

pub(crate) fn build_reseed_prompt(
    user_input: &str,
    class: TurnClass,
    candidates: &[String],
    distillate: &str,
) -> String {
    let mut joined = String::new();
    for (i, c) in candidates.iter().enumerate() {
        joined.push_str(&format!("[{i}]\n{c}\n\n"));
    }
    format!(
        "You re-evaluate {n} candidate answers to a {class_str} message using the distilled reasoning.\n\
         Output strictly: <index>|<one-sentence rationale>\n\
         Example: 2|Most directly addresses the requested fix.\n\n\
         User message: {user_input}\n\n\
         Distilled reasoning so far: {distillate}\n\n\
         Candidates:\n{joined}\
         Verdict:",
        n = candidates.len(),
        class_str = class.as_wire_str(),
    )
}

const MARKERS: &[&str] = &[
    "unsure", "unclear", "uncertain", "not sure", "unknown", "maybe",
    "possibly", "might", "appears", "seems", "assume", "unverified",
    "cannot determine", "insufficient", "open question", "needs verification", "?",
];

fn has_uncertainty_marker(s: &str) -> bool {
    let lowercase = s.to_ascii_lowercase();
    MARKERS.iter().any(|marker| lowercase.contains(marker))
}

/// Check whether a proposed distillate is safe to use as the next audit seed.
///
/// Empty and oversized proposals are rejected first; when the prior trace
/// contains uncertainty, the proposal must retain at least one uncertainty
/// marker.
pub fn admit_distillate(
    pre_reset_trace: &str,
    proposed: &str,
    max_chars: usize,
) -> DistillateVerdict {
    if proposed.trim().is_empty() {
        return DistillateVerdict::RejectedEmpty;
    }
    if proposed.chars().count() > max_chars {
        return DistillateVerdict::RejectedOversize;
    }
    if has_uncertainty_marker(pre_reset_trace) && !has_uncertainty_marker(proposed) {
        return DistillateVerdict::RejectedUncertaintyStripped;
    }
    DistillateVerdict::Admitted
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_empty_and_whitespace() {
        assert_eq!(admit_distillate("trace", "", 10), DistillateVerdict::RejectedEmpty);
        assert_eq!(admit_distillate("trace", " \n\t", 10), DistillateVerdict::RejectedEmpty);
    }

    #[test]
    fn rejects_oversize_by_character_count_at_boundary() {
        let exact = "ééé";
        assert_eq!(admit_distillate("trace", exact, 3), DistillateVerdict::Admitted);
        assert_eq!(admit_distillate("trace", "éééé", 3), DistillateVerdict::RejectedOversize);
    }

    #[test]
    fn rejects_uncertainty_stripped() {
        assert_eq!(
            admit_distillate("The answer is unsure.", "The answer is correct.", 100),
            DistillateVerdict::RejectedUncertaintyStripped
        );
    }

    #[test]
    fn admits_different_uncertainty_marker() {
        assert_eq!(
            admit_distillate("The answer is unsure.", "Maybe this is correct.", 100),
            DistillateVerdict::Admitted
        );
    }

    #[test]
    fn zero_percent_disables_distill_threshold() {
        assert_eq!(distill_threshold(8, 0), 0);
    }

    #[test]
    fn percentages_above_100_are_clamped() {
        assert_eq!(distill_threshold(8, 101), distill_threshold(8, 100));
    }

    #[test]
    fn no_trace_uncertainty_does_not_invent_doubt() {
        assert_eq!(
            admit_distillate("The answer is clear.", "The answer is correct.", 100),
            DistillateVerdict::Admitted
        );
    }
    #[test]
    fn wire_labels_are_snake_case() {
        assert_eq!(CheckpointTrigger::ContextFill.as_wire_str(), "context_fill");
        assert_eq!(CheckpointTrigger::GrcBudget.as_wire_str(), "grc_budget");
        assert_eq!(
            CheckpointOutcome::ResetAndContinued.as_wire_str(),
            "reset_and_continued"
        );
        assert_eq!(
            CheckpointOutcome::DistillateRejected.as_wire_str(),
            "distillate_rejected"
        );
        assert_eq!(
            CheckpointOutcome::EscalatedToCloud.as_wire_str(),
            "escalated_to_cloud"
        );
    }
}
