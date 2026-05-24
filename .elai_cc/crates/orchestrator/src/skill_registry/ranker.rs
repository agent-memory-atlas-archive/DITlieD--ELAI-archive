//! C-41 — Skill ranker (top-k selection with deterministic tie-breaking).
//!
//! Calls [`super::scoring::score_skill_for_context`] for every candidate,
//! sorts the results, and returns the top `k`. Tie-breaking is deterministic
//! so identical inputs always produce identical orderings — the contract test
//! pins this property.
//!
//! Tie-break order (when two candidates have equal scores):
//!   1. Higher parsed version wins (via [`super::manifest::SkillManifest::parsed_version`]).
//!   2. Lexicographically smaller name wins (ASCII case-insensitive).

use super::manifest::SkillManifest;
use super::scoring::{score_skill_for_context, SkillContext};

/// One scored candidate ready for ranking / display.
#[derive(Debug, Clone, PartialEq)]
pub struct ScoredSkill<'a> {
    pub manifest: &'a SkillManifest,
    pub score: f32,
}

/// Return the top-k candidates by score (descending). When `k > candidates.len()`,
/// returns all candidates. When `candidates` is empty, returns an empty Vec.
///
/// Determinism:
///   - The comparator is total over `(score, version, name)` so a stable sort
///     is unnecessary; the output is identical for any sort algorithm.
///   - `f32::partial_cmp` on score is `Less`/`Equal`/`Greater` for finite
///     inputs; if any score is NaN the comparator falls back to "candidate
///     less than" — `score_skill_for_context` is documented to never return
///     NaN (it returns clamped finite values), so this branch is defensive.
#[must_use]
pub fn top_k_skills<'a>(
    candidates: &'a [SkillManifest],
    ctx: &SkillContext<'_>,
    k: usize,
) -> Vec<ScoredSkill<'a>> {
    if k == 0 || candidates.is_empty() {
        return Vec::new();
    }
    let mut scored: Vec<ScoredSkill<'a>> = candidates
        .iter()
        .map(|manifest| ScoredSkill {
            manifest,
            score: score_skill_for_context(manifest, ctx),
        })
        .collect();

    scored.sort_by(|left, right| {
        // 1. Higher score first.
        match right.score.partial_cmp(&left.score) {
            Some(std::cmp::Ordering::Equal) | None => {}
            Some(other) => return other,
        }
        // 2. Higher parsed version first.
        let lv = left.manifest.parsed_version();
        let rv = right.manifest.parsed_version();
        match rv.cmp(&lv) {
            std::cmp::Ordering::Equal => {}
            other => return other,
        }
        // 3. ASCII case-insensitive name ascending.
        left.manifest
            .name
            .to_ascii_lowercase()
            .cmp(&right.manifest.name.to_ascii_lowercase())
    });

    scored.truncate(k);
    scored
}

#[cfg(test)]
mod tests {
    use super::super::manifest::SkillOrigin;
    use super::*;
    use std::path::PathBuf;

    fn skill(name: &str, version: &str, triggers: Vec<&str>) -> SkillManifest {
        SkillManifest {
            name: name.into(),
            version: version.into(),
            triggers: triggers.into_iter().map(String::from).collect(),
            inputs: vec![],
            outputs: vec![],
            scope: String::new(),
            origin: SkillOrigin::SkillsDir,
            file_path: PathBuf::from(name),
            learned_credit_score: None,
            distilled_fragments: Vec::new(),
        }
    }

    #[test]
    fn empty_candidates_yields_empty_top_k() {
        let ctx = SkillContext::from_text("anything");
        assert!(top_k_skills(&[], &ctx, 5).is_empty());
    }

    #[test]
    fn k_zero_yields_empty() {
        let pool = vec![skill("a", "1.0.0", vec!["x"])];
        let ctx = SkillContext::from_text("x");
        assert!(top_k_skills(&pool, &ctx, 0).is_empty());
    }

    #[test]
    fn higher_score_wins() {
        let pool = vec![
            skill("a", "1.0.0", vec!["foo", "bar"]),
            skill("b", "1.0.0", vec!["foo"]),
        ];
        let ctx = SkillContext::from_text("foo bar");
        let ranked = top_k_skills(&pool, &ctx, 2);
        assert_eq!(ranked.len(), 2);
        assert_eq!(ranked[0].manifest.name, "a");
        assert_eq!(ranked[1].manifest.name, "b");
    }

    #[test]
    fn tie_break_uses_version_then_name() {
        let pool = vec![
            skill("zeta", "1.0.0", vec!["foo"]),
            skill("alpha", "1.0.0", vec!["foo"]),
            skill("beta", "2.0.0", vec!["foo"]),
        ];
        let ctx = SkillContext::from_text("foo");
        let ranked = top_k_skills(&pool, &ctx, 3);
        assert_eq!(ranked[0].manifest.name, "beta", "newer version wins");
        // Then alphabetical among the 1.0.0 cohort: alpha < zeta.
        assert_eq!(ranked[1].manifest.name, "alpha");
        assert_eq!(ranked[2].manifest.name, "zeta");
    }

    #[test]
    fn deterministic_repeat_invocations_identical() {
        let pool = vec![
            skill("a", "1.0.0", vec!["foo"]),
            skill("b", "1.0.0", vec!["foo"]),
            skill("c", "1.0.0", vec!["foo"]),
        ];
        let ctx = SkillContext::from_text("foo");
        let first = top_k_skills(&pool, &ctx, 3);
        let second = top_k_skills(&pool, &ctx, 3);
        let first_names: Vec<&str> = first.iter().map(|s| s.manifest.name.as_str()).collect();
        let second_names: Vec<&str> = second.iter().map(|s| s.manifest.name.as_str()).collect();
        assert_eq!(first_names, second_names);
    }

    #[test]
    fn k_greater_than_pool_returns_all() {
        let pool = vec![
            skill("a", "1.0.0", vec!["x"]),
            skill("b", "1.0.0", vec!["x"]),
        ];
        let ctx = SkillContext::from_text("x");
        let ranked = top_k_skills(&pool, &ctx, 10);
        assert_eq!(ranked.len(), 2);
    }
}
