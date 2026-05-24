//! C-49 — Query-time lookup surface used by prompt assembly.
//!
//! `lookup_distilled_fragments` is the read side of the experience
//! corpus. It composes [`distill_fragments`] with the module's default
//! thresholds so callers in the prompt-assembly pipeline get a single
//! convenience entry point.
//!
//! Empty corpus / unknown skill returns an empty `Vec` (transparent
//! skip — stub_ban from the plan: "greenfield (empty corpus) MUST NOT
//! panic; transparent skip").

use super::corpus::ExperienceCorpus;
use super::distiller::{distill_fragments, DEFAULT_MIN_FREQ, DEFAULT_MIN_SUCCESS_RATE};
use super::fragment::PromptFragment;

/// Query-time interface used by prompt assembly. Returns the distilled
/// prompt fragments for `skill_name` ranked by `rank_score` descending.
/// Returns an empty vec for greenfield / unknown skill / threshold
/// miss.
///
/// Caller-side composition: call this AFTER the `skill_registry`
/// top-k lookup so the per-skill fragments are scoped to the skill the
/// router actually selected. The C-49 plan's "consult corpus AFTER
/// SkillRegistry lookup" wiring contract is encoded here as a comment
/// + as the contract test's call-order assertion.
#[must_use]
pub fn lookup_distilled_fragments(
    skill_name: &str,
    registry: &ExperienceCorpus,
) -> Vec<PromptFragment> {
    distill_fragments(
        registry,
        skill_name,
        DEFAULT_MIN_FREQ,
        DEFAULT_MIN_SUCCESS_RATE,
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::experience::fragment::PromptFragment;
    use crate::experience::trace::SuccessTrace;

    #[test]
    fn unknown_skill_returns_empty() {
        let c = ExperienceCorpus::new();
        let out = lookup_distilled_fragments("nonexistent", &c);
        assert!(out.is_empty());
    }

    #[test]
    fn rebuilds_ranked_fragments() {
        let mut c = ExperienceCorpus::new();
        let f = PromptFragment::new("decompose first".into(), 1, 1.0, 10);
        for ts in 1..=3 {
            c.record_trace(
                "planning_workflow",
                SuccessTrace::new(
                    "planning_workflow".into(),
                    0,
                    0,
                    true,
                    ts,
                    vec![f.clone()],
                ),
            )
            .expect("rec");
        }
        let out = lookup_distilled_fragments("planning_workflow", &c);
        assert_eq!(out.len(), 1);
        assert_eq!(out[0].text, "decompose first");
        assert!(out[0].freq_used >= DEFAULT_MIN_FREQ);
    }
}
