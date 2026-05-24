//! C-41 — Skill routing evaluation harness.
//!
//! Mirrors the test-set structure from arXiv:2604.21910 Section 5.1 Table 2:
//! 5 tiers x 30 queries = 150 stratified examples, full-match accuracy
//! (entire intent correct, not per-field). The 44.0% -> 83.3% effect-size
//! is on Claude Opus 4.6 (S0 vs S3); reproducing the absolute number
//! requires the paper's Conductor + Composer + LLM stack, NOT just the
//! local registry.
//!
//! For local CI the included fixture is a small 5-query stub at
//! `crates/contracts/tests/fixtures/skills_eval/`. The full 150-query
//! evaluation is marked RUNPOD-PENDING because it requires Claude Opus
//! 4.6 inference for an apples-to-apples baseline against the paper.

use super::manifest::SkillManifest;
use super::ranker::top_k_skills;
use super::scoring::SkillContext;

/// One evaluation query: a natural-language task + the skill the registry
/// is expected to rank first.
///
/// `tier` is the SciQuest-style difficulty label (1 = explicit code,
/// 5 = adversarial). Per the paper, T1-T2 reach near-100% with vocabulary
/// skills (S1), while T3-T5 drop sharply without strategy skills (S3).
/// Local evaluation tracks per-tier accuracy alongside the aggregate so
/// regression in any single tier is visible.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SkillEvalQuery {
    /// Natural-language task description that the registry will score.
    pub question: String,
    /// Skill name the registry MUST rank first for this query to count as
    /// a full match.
    pub expected_skill_name: String,
    /// Difficulty tier (1..=5). Mirrors the paper's 5-tier stratification.
    pub tier: u8,
}

/// Aggregate result of one evaluation pass.
///
/// `full_match_rate` is the fraction of queries where the registry's
/// top-1 candidate name matched `expected_skill_name`. `per_tier_match`
/// is an array of 5 entries (one per tier) where each entry is the
/// fraction of queries within that tier that full-matched. Tier indices
/// are 1-based to match the paper; index 0 of the array is always 0.0.
#[derive(Debug, Clone, PartialEq)]
pub struct EvalResult {
    pub total_queries: usize,
    pub full_match_count: usize,
    pub full_match_rate: f32,
    /// Index 0 is unused; valid tiers are 1..=5.
    pub per_tier_match: [f32; 6],
}

/// Evaluate the skill registry against the supplied query set.
///
/// For each query, the registry is asked for `top_k_skills(candidates, &ctx, 1)`;
/// the query counts as a full match iff the returned top-1 skill name
/// equals `expected_skill_name`. Empty result sets count as a miss.
///
/// RUNPOD-PENDING: a 150-query stratified eval on Claude Opus 4.6 to
/// reproduce the paper's 44.0% -> 83.3% baseline-vs-Skills effect-size.
/// The harness here is the deterministic check that the registry surface
/// is wired correctly; the LLM-grounded baseline lives on remote GPU.
/// See `gates_eval_skills_150queries_qwen3_8b` placeholder above.
//
// RUNPOD-PENDING: gates_eval_skills_150queries_qwen3_8b
#[must_use]
pub fn eval_skill_routing(
    queries: &[SkillEvalQuery],
    registry: &[SkillManifest],
) -> EvalResult {
    let mut full_match_count = 0usize;
    let mut per_tier_total = [0u32; 6];
    let mut per_tier_hits = [0u32; 6];

    for query in queries {
        let tier_index = usize::from(query.tier.clamp(1, 5));
        per_tier_total[tier_index] = per_tier_total[tier_index].saturating_add(1);
        let ctx = SkillContext::from_text(&query.question);
        let ranked = top_k_skills(registry, &ctx, 1);
        if let Some(top) = ranked.first() {
            if top.manifest.name == query.expected_skill_name {
                full_match_count += 1;
                per_tier_hits[tier_index] = per_tier_hits[tier_index].saturating_add(1);
            }
        }
    }

    let total = queries.len();
    let full_match_rate = if total == 0 {
        0.0
    } else {
        // u32 -> f32 widening; counts cannot exceed query slice length.
        f32::from(full_match_count as u16) / f32::from(total as u16)
    };

    let mut per_tier_match = [0.0_f32; 6];
    for tier in 1..=5_usize {
        let total = per_tier_total[tier];
        if total > 0 {
            per_tier_match[tier] =
                f32::from(per_tier_hits[tier] as u16) / f32::from(total as u16);
        }
    }

    EvalResult {
        total_queries: total,
        full_match_count,
        full_match_rate,
        per_tier_match,
    }
}

#[cfg(test)]
mod tests {
    use super::super::manifest::SkillOrigin;
    use super::*;
    use std::path::PathBuf;

    fn skill(name: &str, triggers: Vec<&str>) -> SkillManifest {
        SkillManifest {
            name: name.into(),
            version: "1.0.0".into(),
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
    fn empty_queries_returns_zero_rate() {
        let registry = vec![skill("a", vec!["x"])];
        let result = eval_skill_routing(&[], &registry);
        assert_eq!(result.total_queries, 0);
        assert_eq!(result.full_match_count, 0);
        assert_eq!(result.full_match_rate, 0.0);
    }

    #[test]
    fn full_match_when_top_one_matches_expected() {
        let registry = vec![
            skill("planner", vec!["plan", "design"]),
            skill("writer", vec!["write", "compose"]),
        ];
        let queries = vec![
            SkillEvalQuery {
                question: "plan the project".into(),
                expected_skill_name: "planner".into(),
                tier: 1,
            },
            SkillEvalQuery {
                question: "compose a paragraph".into(),
                expected_skill_name: "writer".into(),
                tier: 1,
            },
        ];
        let result = eval_skill_routing(&queries, &registry);
        assert_eq!(result.full_match_count, 2);
        assert!((result.full_match_rate - 1.0).abs() < 1e-5);
        assert!((result.per_tier_match[1] - 1.0).abs() < 1e-5);
    }

    #[test]
    fn miss_when_top_one_is_different_skill() {
        let registry = vec![skill("planner", vec!["plan"])];
        let queries = vec![SkillEvalQuery {
            question: "plan something".into(),
            expected_skill_name: "writer".into(),
            tier: 2,
        }];
        let result = eval_skill_routing(&queries, &registry);
        assert_eq!(result.full_match_count, 0);
    }
}
