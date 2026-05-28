//! LATS-style deterministic process-reward scalar for
//! [`crate::auditor_mode::LevelAuditVerdict`].
//!
//! Plan task 6 of E-46. The shipped per-level audit verdict (B-41
//! SHIPPED, SCHEMA 45) carries categorical PASS/BLOCK + a list of
//! findings. This module computes a deterministic process-reward scalar
//! in `[0.0, 1.0]` from that verdict — additive over the existing struct,
//! NOT a struct-field modification (so 100% backward compatible).
//!
//! Computation (per plan §47):
//!
//! ```text
//!   score = (1.0 - findings.len() / MAX_EXPECTED_FINDINGS) * verdict_modifier
//! ```
//!
//! where `verdict_modifier`:
//!   * `Verdict::Allow`              -> 1.0
//!   * `Verdict::Block`              -> 0.1
//!   * Severity::Warn-dominant Allow -> 0.6   (computed from finding severity
//!     histogram; the canonical Verdict enum does not carry a Warn variant
//!     so we synthesize it from Severity::Warn count > 0 && no Severity::Block)
//!   * AGENTLESS shortcut eligible   -> 0.85  (caller signals via the
//!     dedicated [`compute_score_for_agentless_shortcut_eligible`] helper)
//!
//! Default `MAX_EXPECTED_FINDINGS = 5` (configurable in
//! `elai_cc.toml` `[planning.process_reward] max_expected_findings`).
//!
//! Reference: arXiv:2310.04406 LATS — value-function as auditor at every
//! fork. The deterministic formula is documented in plan §47 as the
//! starting point; an LLM-estimated arm is left as an open question
//! deferred to a follow-up patch (plan §209).

use serde::{Deserialize, Serialize};

use crate::auditor_mode::{LevelAuditVerdict, Severity, Verdict};

/// Default max-expected findings denominator per plan §47. Surface in
/// `elai_cc.toml` `[planning.process_reward] max_expected_findings`.
pub const MAX_EXPECTED_FINDINGS: f32 = 5.0;

/// Default fork-pruning floor per plan §47 — scope-planner-elai prunes
/// forks where the parent audit's score is below this value. Surface in
/// `elai_cc.toml` `[planning.process_reward] fork_prune_floor`.
pub const FORK_PRUNE_FLOOR: f32 = 0.4;

/// Snapshot of the process-reward scalar + the verdict-modifier label
/// used at compute time. Persisted alongside the original
/// [`LevelAuditVerdict`] (the producer emits a `process_reward_score_emitted`
/// SSB row); decomposers consume it as the LATS-style value function.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ProcessRewardScore {
    /// Score in [0.0, 1.0].
    pub score: f32,
    /// Categorical label for the verdict modifier used in the computation.
    pub verdict_modifier_label: VerdictModifierLabel,
    /// Number of findings the score was computed from (helps debugging
    /// the score on the dashboard).
    pub findings_count: u32,
    /// Max-expected findings denominator used in the formula.
    pub max_expected_findings: f32,
}

/// Categorical label for the verdict-modifier arm chosen at compute time.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum VerdictModifierLabel {
    /// `Verdict::Allow` with no Warn findings — score modifier 1.0.
    Pass,
    /// `Verdict::Allow` with Severity::Warn findings — score modifier 0.6.
    Warn,
    /// `Verdict::Block` — score modifier 0.1.
    Block,
    /// AGENTLESS shortcut eligible — score modifier 0.85 (caller signals
    /// this distinct arm via [`compute_score_for_agentless_shortcut_eligible`]).
    AgentlessShortcutEligible,
}

impl VerdictModifierLabel {
    /// Numeric modifier per plan §47.
    #[must_use]
    pub fn modifier(&self) -> f32 {
        match self {
            Self::Pass => 1.0,
            Self::Warn => 0.6,
            Self::Block => 0.1,
            Self::AgentlessShortcutEligible => 0.85,
        }
    }

    /// Snake-case wire label.
    #[must_use]
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Pass => "pass",
            Self::Warn => "warn",
            Self::Block => "block",
            Self::AgentlessShortcutEligible => "agentless_shortcut_eligible",
        }
    }
}

/// Compute the process-reward score for a standard per-level audit
/// verdict. Caller passes the verdict + the `max_expected_findings`
/// denominator (use [`MAX_EXPECTED_FINDINGS`] for the default).
///
/// Determinism guarantees:
///   * Score is always in `[0.0, 1.0]` (clamped).
///   * No NaN — denominator `max_expected_findings.max(1.0)` clamps to >0.
///   * Sort order of findings does not affect score (only the histogram
///     does).
#[must_use]
pub fn compute_score(
    verdict: &LevelAuditVerdict,
    max_expected_findings: f32,
) -> ProcessRewardScore {
    let denom = max_expected_findings.max(1.0);
    let findings_count = verdict.findings.len() as f32;
    let saturation = (1.0 - findings_count / denom).clamp(0.0, 1.0);
    let label = classify_verdict(verdict);
    let modifier = label.modifier();
    let score = (saturation * modifier).clamp(0.0, 1.0);
    ProcessRewardScore {
        score,
        verdict_modifier_label: label,
        findings_count: verdict.findings.len() as u32,
        max_expected_findings: denom,
    }
}

/// Compute the process-reward score for an AGENTLESS-shortcut-eligible
/// step. The 0.85 modifier reflects plan §47's hand-picked value
/// (single-file / single-symbol steps that bypass BabyStep decomposition).
#[must_use]
pub fn compute_score_for_agentless_shortcut_eligible(
    verdict: &LevelAuditVerdict,
    max_expected_findings: f32,
) -> ProcessRewardScore {
    let denom = max_expected_findings.max(1.0);
    let findings_count = verdict.findings.len() as f32;
    let saturation = (1.0 - findings_count / denom).clamp(0.0, 1.0);
    let label = VerdictModifierLabel::AgentlessShortcutEligible;
    let modifier = label.modifier();
    let score = (saturation * modifier).clamp(0.0, 1.0);
    ProcessRewardScore {
        score,
        verdict_modifier_label: label,
        findings_count: verdict.findings.len() as u32,
        max_expected_findings: denom,
    }
}

/// True iff the score is below the [`FORK_PRUNE_FLOOR`] — caller (scope-
/// planner-elai) should prune the fork before TreeNavigator commits.
#[must_use]
pub fn should_prune_fork(score: f32, floor: f32) -> bool {
    score < floor.clamp(0.0, 1.0)
}

/// Map a [`LevelAuditVerdict`] to the [`VerdictModifierLabel`] used by
/// the compute formula. The plan §47 modifier table includes `WARN` and
/// `AGENTLESS_SHORTCUT_ELIGIBLE` arms not present in the shipped
/// `Verdict` enum (which is 2-arm Allow/Block). We synthesize:
///
///   * `Verdict::Block` -> `Block`
///   * `Verdict::Allow` AND any `Severity::Warn` finding -> `Warn`
///   * `Verdict::Allow` otherwise -> `Pass`
///
/// The `AgentlessShortcutEligible` arm is reached only via the dedicated
/// [`compute_score_for_agentless_shortcut_eligible`] helper.
fn classify_verdict(verdict: &LevelAuditVerdict) -> VerdictModifierLabel {
    match verdict.verdict {
        Verdict::Block => VerdictModifierLabel::Block,
        Verdict::Allow => {
            if verdict.findings.iter().any(|f| f.severity == Severity::Warn) {
                VerdictModifierLabel::Warn
            } else {
                VerdictModifierLabel::Pass
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::auditor_mode::{AuditorMode, Finding, TierNodeId};
    use crate::research_tree::BabyStepId;

    fn mk_verdict(verdict: Verdict, findings: Vec<Finding>) -> LevelAuditVerdict {
        LevelAuditVerdict {
            mode: AuditorMode::BabyStepDetail,
            tier_node_id: TierNodeId::BabyStep(BabyStepId(1)),
            findings,
            verdict,
            respawned_once: false,
            cascade_to_parent: false,
        }
    }

    fn mk_finding(sev: Severity) -> Finding {
        Finding {
            category: "C-TEST".into(),
            severity: sev,
            message: "test".into(),
            source: None,
        }
    }

    #[test]
    fn allow_no_findings_score_one() {
        let v = mk_verdict(Verdict::Allow, vec![]);
        let r = compute_score(&v, MAX_EXPECTED_FINDINGS);
        assert_eq!(r.score, 1.0);
        assert_eq!(r.verdict_modifier_label, VerdictModifierLabel::Pass);
    }

    #[test]
    fn block_no_findings_score_block_modifier() {
        let v = mk_verdict(Verdict::Block, vec![]);
        let r = compute_score(&v, MAX_EXPECTED_FINDINGS);
        assert!((r.score - 0.1).abs() < 1e-5, "got {}", r.score);
        assert_eq!(r.verdict_modifier_label, VerdictModifierLabel::Block);
    }

    #[test]
    fn allow_with_warn_finding_is_warn_modifier() {
        let v = mk_verdict(Verdict::Allow, vec![mk_finding(Severity::Warn)]);
        let r = compute_score(&v, MAX_EXPECTED_FINDINGS);
        assert_eq!(r.verdict_modifier_label, VerdictModifierLabel::Warn);
        // (1 - 1/5) * 0.6 = 0.48
        assert!((r.score - 0.48).abs() < 1e-5, "got {}", r.score);
    }

    #[test]
    fn allow_with_info_only_is_pass_modifier() {
        let v = mk_verdict(Verdict::Allow, vec![mk_finding(Severity::Info)]);
        let r = compute_score(&v, MAX_EXPECTED_FINDINGS);
        assert_eq!(r.verdict_modifier_label, VerdictModifierLabel::Pass);
        // (1 - 1/5) * 1.0 = 0.8
        assert!((r.score - 0.8).abs() < 1e-5, "got {}", r.score);
    }

    #[test]
    fn many_findings_saturates_to_zero() {
        let v = mk_verdict(
            Verdict::Allow,
            (0..10).map(|_| mk_finding(Severity::Info)).collect(),
        );
        let r = compute_score(&v, MAX_EXPECTED_FINDINGS);
        // saturation is (1 - 10/5) = -1 -> clamped to 0 -> score 0
        assert_eq!(r.score, 0.0);
    }

    #[test]
    fn agentless_shortcut_eligible_modifier_applied() {
        let v = mk_verdict(Verdict::Allow, vec![]);
        let r = compute_score_for_agentless_shortcut_eligible(&v, MAX_EXPECTED_FINDINGS);
        assert!((r.score - 0.85).abs() < 1e-5, "got {}", r.score);
        assert_eq!(
            r.verdict_modifier_label,
            VerdictModifierLabel::AgentlessShortcutEligible
        );
    }

    #[test]
    fn score_always_in_unit_interval_for_50_synthetic_verdicts() {
        // Plan test §169: 50 synthetic verdicts; assert score in [0, 1].
        for i in 0..50 {
            let n_findings = i % 8;
            let verdict_choice = if i % 3 == 0 { Verdict::Block } else { Verdict::Allow };
            let severities = [Severity::Block, Severity::Warn, Severity::Info];
            let findings = (0..n_findings)
                .map(|j| mk_finding(severities[j % 3]))
                .collect();
            let v = mk_verdict(verdict_choice, findings);
            let r = compute_score(&v, MAX_EXPECTED_FINDINGS);
            assert!(
                (0.0..=1.0).contains(&r.score),
                "iteration {} produced score {} outside [0, 1]",
                i,
                r.score
            );
        }
    }

    #[test]
    fn denominator_clamp_zero_yields_clamped_score() {
        // max_expected_findings <= 0 falls through to denom=1.0
        let v = mk_verdict(Verdict::Allow, vec![mk_finding(Severity::Info)]);
        let r = compute_score(&v, 0.0);
        // denom=1.0, sat = 1 - 1/1 = 0 -> score 0
        assert_eq!(r.score, 0.0);
        assert_eq!(r.max_expected_findings, 1.0);
    }

    #[test]
    fn should_prune_fork_floor_check() {
        assert!(should_prune_fork(0.3, FORK_PRUNE_FLOOR));
        assert!(!should_prune_fork(0.5, FORK_PRUNE_FLOOR));
        assert!(!should_prune_fork(0.4, FORK_PRUNE_FLOOR));
    }

    #[test]
    fn verdict_modifier_label_str_roundtrip() {
        assert_eq!(VerdictModifierLabel::Pass.as_str(), "pass");
        assert_eq!(VerdictModifierLabel::Warn.as_str(), "warn");
        assert_eq!(VerdictModifierLabel::Block.as_str(), "block");
        assert_eq!(
            VerdictModifierLabel::AgentlessShortcutEligible.as_str(),
            "agentless_shortcut_eligible"
        );
    }
}
