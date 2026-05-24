//! C-41 — Skill scoring (locally authored).
//!
//! Reliability caveat: arXiv:2604.21910 specifies "domain classification, not
//! similarity search" (Section 2.3) but does NOT publish the classifier prompt,
//! scoring function, or activation threshold (paper-ingestor note in
//! `workflow/papers/scientific-workflow-interface.md`, "Skill selection /
//! trigger predicate (CRITICAL GAP)"). The scoring algorithm here is
//! LOCALLY AUTHORED. It must be deterministic, side-effect-free, and never
//! panic — the contract test in
//! `crates/contracts/tests/c41_skills_driven_workflow_contract.rs` asserts
//! these properties.
//!
//! Scoring contract:
//!   - Inputs are normalized to ASCII-lowercase before comparison.
//!   - Each substring trigger match contributes its weight (length-normalized).
//!   - Scope match contributes a fixed bonus.
//!   - Score is clamped to [0.0, 1.0]; ties broken deterministically by
//!     version (newer wins) then by skill name (lexicographic) in
//!     `ranker.rs::top_k_skills`.

use super::manifest::SkillManifest;

/// Minimum confidence below which the orchestrator should treat the
/// top-ranked candidate as a fall-through (no skill auto-selected). Tuned
/// conservatively at 0.20 — lower values produce too many false positives
/// against the legacy skills lacking trigger metadata; higher values miss
/// short-trigger matches. The threshold is exposed as a module constant so
/// the test suite can pin it.
pub const SKILL_CONFIDENCE_THRESHOLD: f32 = 0.20;

/// Context envelope supplied to [`score_skill_for_context`].
///
/// `task_text` is the natural-language task / query the orchestrator is
/// classifying. `scope_hint` is an optional invocation tag (e.g.
/// "execution" / "planning" / "research") set by the dispatcher when
/// known; an empty hint disables the scope-bonus arm of the scoring
/// formula.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SkillContext<'a> {
    pub task_text: &'a str,
    pub scope_hint: &'a str,
}

impl<'a> SkillContext<'a> {
    /// Convenience constructor; scope hint defaults to empty.
    #[must_use]
    pub fn from_text(task_text: &'a str) -> Self {
        Self {
            task_text,
            scope_hint: "",
        }
    }
}

/// Compute a deterministic score in [0.0, 1.0] for one skill against the
/// supplied context.
///
/// Returns 0.0 when the manifest has no triggers — empty-triggers skills
/// are deliberately not auto-selectable. Returns 1.0 only when EVERY
/// trigger matches AND the scope matches; for typical multi-trigger
/// manifests this is essentially unreachable, leaving headroom.
///
/// The formula is:
///   raw = sum(weight_i for each matching trigger) / total_trigger_count
///   raw += SCOPE_BONUS if scope match
///   score = clamp(raw, 0.0, 1.0)
///
/// `weight_i` is currently 1.0 per matching trigger; the length-normalized
/// variant is reserved for a future ranker tweak (kept in this comment so
/// the rationale survives across refactors).
#[must_use]
pub fn score_skill_for_context(manifest: &SkillManifest, ctx: &SkillContext<'_>) -> f32 {
    if manifest.triggers.is_empty() {
        return 0.0;
    }
    let normalized_task = ctx.task_text.to_ascii_lowercase();
    if normalized_task.is_empty() {
        return 0.0;
    }
    let mut hits: u32 = 0;
    for trigger in &manifest.triggers {
        let needle = trigger.trim().to_ascii_lowercase();
        if needle.is_empty() {
            continue;
        }
        if normalized_task.contains(&needle) {
            hits = hits.saturating_add(1);
        }
    }

    let total = manifest.triggers.len() as u32;
    if total == 0 || hits == 0 {
        // hits==0 short-circuits scope-only matches: a skill must engage
        // at least one trigger before the scope bonus applies. This
        // prevents a pure scope match from out-ranking a partial trigger
        // match.
        return 0.0;
    }

    // Cast through u32 -> f32 explicitly per the Anti-Stale-Training rule
    // (.try_into() / explicit cast on narrowing). u32 -> f32 is widening
    // for the typical hit count (< 16) so `as f32` is bounded-safe here.
    let raw_match = f32::from(hits as u16) / f32::from(total as u16);

    let scope_bonus = if !ctx.scope_hint.is_empty()
        && ctx.scope_hint.eq_ignore_ascii_case(manifest.scope.trim())
    {
        SCOPE_MATCH_BONUS
    } else {
        0.0
    };

    let raw = raw_match + scope_bonus;
    raw.clamp(0.0, 1.0)
}

/// Bonus added to the raw trigger-match score when the invocation context's
/// scope hint matches the manifest's declared scope. Calibrated against the
/// 0.20 threshold so a single-trigger skill (raw = 1/N) still clears the
/// fall-through floor when scope matches.
const SCOPE_MATCH_BONUS: f32 = 0.10;

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    use super::super::manifest::SkillOrigin;

    fn skill(name: &str, triggers: Vec<&str>, scope: &str) -> SkillManifest {
        SkillManifest {
            name: name.into(),
            version: "1.0.0".into(),
            triggers: triggers.into_iter().map(String::from).collect(),
            inputs: vec![],
            outputs: vec![],
            scope: scope.into(),
            origin: SkillOrigin::SkillsDir,
            file_path: PathBuf::from(name),
            learned_credit_score: None,
            distilled_fragments: Vec::new(),
        }
    }

    #[test]
    fn empty_triggers_returns_zero() {
        let m = skill("legacy", vec![], "");
        let ctx = SkillContext::from_text("implement feature");
        assert_eq!(score_skill_for_context(&m, &ctx), 0.0);
    }

    #[test]
    fn partial_trigger_match_scales_with_hits() {
        let m = skill("planner", vec!["plan", "design", "decompose"], "planning");
        let ctx = SkillContext::from_text("Please plan this change");
        let score = score_skill_for_context(&m, &ctx);
        // 1 of 3 triggers, no scope hint.
        let expected = 1.0 / 3.0;
        assert!((score - expected).abs() < 1e-5, "score={score}");
    }

    #[test]
    fn scope_match_bonus_applies_when_scope_hint_matches() {
        let m = skill("planner", vec!["plan"], "planning");
        let ctx = SkillContext {
            task_text: "plan this",
            scope_hint: "planning",
        };
        let score = score_skill_for_context(&m, &ctx);
        // Full trigger match (1.0) + bonus, clamped to 1.0.
        assert!((score - 1.0).abs() < 1e-5, "score={score}");
    }

    #[test]
    fn scope_alone_without_trigger_match_yields_zero() {
        let m = skill("planner", vec!["plan"], "planning");
        let ctx = SkillContext {
            task_text: "unrelated task body",
            scope_hint: "planning",
        };
        let score = score_skill_for_context(&m, &ctx);
        assert_eq!(score, 0.0, "scope-only must not select a skill");
    }

    #[test]
    fn case_insensitive_trigger_match() {
        let m = skill("upper", vec!["IMPLEMENT"], "");
        let ctx = SkillContext::from_text("please implement the feature");
        let score = score_skill_for_context(&m, &ctx);
        assert!(score > 0.0);
    }

    #[test]
    fn score_is_clamped_within_unit_interval() {
        let m = skill("multi", vec!["a", "b", "c"], "x");
        let ctx = SkillContext {
            task_text: "a b c",
            scope_hint: "x",
        };
        let score = score_skill_for_context(&m, &ctx);
        assert!(score <= 1.0 && score >= 0.0, "score={score}");
    }

    #[test]
    fn empty_task_text_returns_zero() {
        let m = skill("any", vec!["plan"], "");
        let ctx = SkillContext::from_text("");
        assert_eq!(score_skill_for_context(&m, &ctx), 0.0);
    }
}
