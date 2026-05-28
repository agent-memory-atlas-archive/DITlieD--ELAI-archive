//! E-48 Task 1 — editable plan stage (Gemini-style editable CoT).
//!
//! Produces a [`WebResearchPlan`] artifact BEFORE the SEARCH stage fires.
//! Sub-questions are ranked by expected information gain via a deterministic
//! heuristic combining a token-overlap novelty score (lower overlap with
//! the parent question == higher novelty == higher gain) and a named-entity
//! specificity score (more named entities == more specific == higher gain).
//!
//! Grounding:
//!   * gemini.google/overview/deep-research/ — editable CoT plan stage
//!     surfaced to the user for approve/edit before the orchestrator
//!     dispatches workers.
//!   * Anthropic multi-agent orchestrator+workers pattern (per
//!     simonwillison.net/2025/Jun/14) — lead emits the plan, workers
//!     execute; plan is human-inspectable.
//!
//! Naming: sibling to (NOT replacement of) the existing
//! [`crate::research_tree::ResearchPlan`] tree-input struct. The two
//! cover different concerns; the editable-CoT artifact carries an
//! ordering signal (`expected_information_gain_rank`) and a budget
//! cap (`planned_search_budget`) that the tree-input struct lacks.

use serde::{Deserialize, Serialize};

/// Default planned search budget when the caller does not override.
/// 16 search calls is the empirical default from B-7 dual-threshold runs
/// (per research_tier::ResearchTierConfig::depth_limit defaults).
pub const DEFAULT_PLANNED_SEARCH_BUDGET: usize = 16;

/// Maximum sub-questions surfaced in an editable plan. The dashboard
/// (deferred per the session's NO-dashboard scope) is sized for ~8 rows
/// in the ReasoningDrawer; keeping the cap here so the producer side
/// never emits an overflowing plan.
pub const MAX_SUB_QUESTIONS: usize = 8;

/// Truncation cap on sub-question strings before they ride into the SSB
/// payload `details_json`. Prevents giant LLM-emitted prompts from
/// bloating the row.
pub const SUB_QUESTION_CHAR_CAP: usize = 280;

/// One editable sub-question with its information-gain rank.
///
/// Field semantics:
///   * `text` -- canonical sub-question, truncated to
///     [`SUB_QUESTION_CHAR_CAP`].
///   * `novelty_score` -- 0.0 (full token overlap with parent) ..= 1.0
///     (zero overlap with parent). Higher == more novel.
///   * `specificity_score` -- 0.0 (no named-entity tokens) ..= 1.0
///     (>= [`SPECIFICITY_SATURATION`] named-entity tokens). Higher
///     == more specific.
///   * `expected_information_gain` -- linear combination
///     `0.7 * novelty + 0.3 * specificity` so novelty dominates.
///     Range [0.0, 1.0].
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SubQuestion {
    pub text: String,
    pub novelty_score: f64,
    pub specificity_score: f64,
    pub expected_information_gain: f64,
}

/// Number of named-entity tokens at which `specificity_score` saturates
/// at 1.0. Below this it ramps linearly from 0 to 1.0.
pub const SPECIFICITY_SATURATION: usize = 4;

/// Weight of the novelty arm in the information-gain combination.
pub const NOVELTY_WEIGHT: f64 = 0.7;

/// Weight of the specificity arm in the information-gain combination.
pub const SPECIFICITY_WEIGHT: f64 = 0.3;

/// E-48 Task 1 -- editable plan artifact emitted BEFORE the SEARCH stage.
///
/// Sibling to [`crate::research_tree::ResearchPlan`] (the tree-input
/// struct) -- different concerns; both can coexist on the same research
/// run.
///
/// `expected_information_gain_rank` is a permutation of `0..sub_questions.len()`
/// produced by [`WebResearchPlan::with_ranked_sub_questions`].
/// `expected_information_gain_rank[i]` is the position the sub-question
/// at index `i` occupies after sorting by descending
/// `expected_information_gain` -- rank 0 == highest gain.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct WebResearchPlan {
    /// Parent research question the plan is decomposing.
    pub parent_question: String,
    /// Sub-questions in their original emission order (so the dashboard
    /// can preserve LLM emission order if the user wants to edit).
    pub sub_questions: Vec<SubQuestion>,
    /// Permutation: `rank[i] = j` means the i-th sub_question has rank
    /// j after sorting by descending information gain. Rank 0 is best.
    pub expected_information_gain_rank: Vec<usize>,
    /// Maximum number of SEARCH-stage calls the orchestrator may make
    /// for this plan. Defaults to [`DEFAULT_PLANNED_SEARCH_BUDGET`].
    pub planned_search_budget: usize,
    /// Whether this plan was surfaced to the user for interactive edit
    /// (when the `--interactive-research-plan` CLI flag is set; flag
    /// wiring lives outside this module and is RUNPOD-PENDING).
    pub interactive: bool,
    /// Whether the user explicitly approved the plan (only meaningful
    /// when `interactive == true`; auto-approve when `false`).
    pub approved_by_user: bool,
}

impl WebResearchPlan {
    /// Build a plan with sub-questions ranked by the deterministic
    /// information-gain heuristic. The parent_question is stripped of
    /// surrounding whitespace; sub_questions are truncated to
    /// [`SUB_QUESTION_CHAR_CAP`] characters and capped at
    /// [`MAX_SUB_QUESTIONS`] entries.
    ///
    /// Returns the plan with `interactive: false` and
    /// `approved_by_user: false`; callers needing the interactive arm
    /// must call [`Self::mark_interactive`] / [`Self::mark_approved`]
    /// before passing the plan to the producer-side emit fn.
    #[must_use]
    pub fn with_ranked_sub_questions(
        parent_question: impl Into<String>,
        raw_sub_questions: Vec<String>,
        planned_search_budget: usize,
    ) -> Self {
        let parent = parent_question.into().trim().to_owned();
        let parent_tokens = tokenize(&parent);
        let sub_qs: Vec<SubQuestion> = raw_sub_questions
            .into_iter()
            .take(MAX_SUB_QUESTIONS)
            .map(|raw| {
                let trimmed = truncate(&raw, SUB_QUESTION_CHAR_CAP);
                let novelty = novelty_score(&parent_tokens, &trimmed);
                let specificity = specificity_score(&trimmed);
                let gain = NOVELTY_WEIGHT * novelty + SPECIFICITY_WEIGHT * specificity;
                SubQuestion {
                    text: trimmed,
                    novelty_score: novelty,
                    specificity_score: specificity,
                    expected_information_gain: gain,
                }
            })
            .collect();
        let rank = compute_rank(&sub_qs);
        Self {
            parent_question: parent,
            sub_questions: sub_qs,
            expected_information_gain_rank: rank,
            planned_search_budget: planned_search_budget.max(1),
            interactive: false,
            approved_by_user: false,
        }
    }

    /// Mark the plan as surfaced to an interactive user. Idempotent.
    pub fn mark_interactive(&mut self) {
        self.interactive = true;
    }

    /// Mark the plan as user-approved (only valid when interactive is
    /// already set; otherwise the flag is informational).
    pub fn mark_approved(&mut self) {
        self.approved_by_user = true;
    }

    /// Number of sub-questions in this plan (handy for the producer
    /// telemetry row).
    #[must_use]
    pub fn sub_question_count(&self) -> usize {
        self.sub_questions.len()
    }
}

// ---------------------------------------------------------------------------
// Heuristic helpers (deterministic, no I/O)
// ---------------------------------------------------------------------------

/// Tokenize a string into lowercased ASCII-alphanumeric word units.
/// Punctuation, whitespace, and non-ASCII are treated as separators.
fn tokenize(s: &str) -> Vec<String> {
    s.split(|c: char| !c.is_ascii_alphanumeric())
        .filter(|t| !t.is_empty())
        .map(|t| t.to_ascii_lowercase())
        .collect()
}

/// Novelty score in `[0.0, 1.0]` -- the fraction of sub_question tokens
/// that do NOT appear in the parent question's token set. A sub-question
/// using entirely fresh terminology scores 1.0; one that copies the
/// parent verbatim scores 0.0.
fn novelty_score(parent_tokens: &[String], sub: &str) -> f64 {
    let sub_tokens = tokenize(sub);
    if sub_tokens.is_empty() {
        return 0.0;
    }
    let parent_set: std::collections::HashSet<&String> = parent_tokens.iter().collect();
    let novel = sub_tokens.iter().filter(|t| !parent_set.contains(t)).count();
    (novel as f64) / (sub_tokens.len() as f64)
}

/// Specificity score in `[0.0, 1.0]` -- approximates the named-entity
/// count by tallying capitalized tokens (excluding the first word so a
/// natural-language sentence opening does not inflate the score). Ramps
/// linearly from 0 at zero named-entity tokens to 1.0 at
/// [`SPECIFICITY_SATURATION`] tokens.
fn specificity_score(s: &str) -> f64 {
    // Skip the first whitespace-separated token (sentence opener).
    let mut iter = s.split_whitespace();
    let _ = iter.next();
    let cap_count = iter.filter(|tok| starts_with_uppercase(tok)).count();
    if cap_count >= SPECIFICITY_SATURATION {
        return 1.0;
    }
    (cap_count as f64) / (SPECIFICITY_SATURATION as f64)
}

/// Returns true when the first char of `tok` is an ASCII uppercase letter.
/// Punctuation is allowed before the letter (e.g. `"(FastAPI)"` counts).
fn starts_with_uppercase(tok: &str) -> bool {
    tok.chars()
        .find(|c| c.is_ascii_alphanumeric())
        .map(|c| c.is_ascii_uppercase())
        .unwrap_or(false)
}

/// Compute the rank-by-information-gain permutation. `rank[i]` is the
/// 0-based position of `sub_qs[i]` after sorting by descending
/// `expected_information_gain`. Stable on ties (preserves emission
/// order — earlier sub-question wins).
fn compute_rank(sub_qs: &[SubQuestion]) -> Vec<usize> {
    let mut indices: Vec<usize> = (0..sub_qs.len()).collect();
    indices.sort_by(|&a, &b| {
        // Descending by gain; NaN-safe (NaN sorts last).
        let ga = sub_qs[a].expected_information_gain;
        let gb = sub_qs[b].expected_information_gain;
        match gb.partial_cmp(&ga) {
            Some(o) => o.then_with(|| a.cmp(&b)),
            None => a.cmp(&b),
        }
    });
    let mut rank = vec![0_usize; sub_qs.len()];
    for (position, &idx) in indices.iter().enumerate() {
        rank[idx] = position;
    }
    rank
}

/// Truncate a string to `max_chars` characters (char count, not byte
/// count) preserving valid UTF-8 boundaries.
fn truncate(s: &str, max_chars: usize) -> String {
    if s.chars().count() <= max_chars {
        return s.to_owned();
    }
    s.chars().take(max_chars).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tokenize_strips_punctuation() {
        assert_eq!(
            tokenize("Which Python web framework is fastest?"),
            vec!["which", "python", "web", "framework", "is", "fastest"]
        );
    }

    #[test]
    fn novelty_zero_when_sub_copies_parent() {
        let parent = tokenize("which python web framework is fastest");
        let s = novelty_score(&parent, "which python web framework is fastest");
        assert!((s - 0.0).abs() < f64::EPSILON);
    }

    #[test]
    fn novelty_one_when_sub_uses_fresh_tokens() {
        let parent = tokenize("which python web framework is fastest");
        let s = novelty_score(&parent, "litestar throughput numbers");
        assert!((s - 1.0).abs() < f64::EPSILON);
    }

    #[test]
    fn specificity_saturates_at_four_named_entities() {
        // First token skipped; remaining 4 tokens uppercase -> saturation.
        let s = specificity_score("compare FastAPI Litestar Starlette Sanic");
        assert!((s - 1.0).abs() < f64::EPSILON);
    }

    #[test]
    fn specificity_zero_when_no_named_entities() {
        let s = specificity_score("which framework is fastest");
        assert!((s - 0.0).abs() < f64::EPSILON);
    }

    #[test]
    fn plan_ranks_high_gain_first() {
        let plan = WebResearchPlan::with_ranked_sub_questions(
            "which python web framework is fastest",
            vec![
                "which python web framework is fastest".to_owned(),
                "FastAPI Litestar Starlette Sanic throughput benchmarks".to_owned(),
            ],
            DEFAULT_PLANNED_SEARCH_BUDGET,
        );
        // Second sub-question has high novelty AND high specificity -> rank 0.
        assert_eq!(plan.expected_information_gain_rank[1], 0);
        assert_eq!(plan.expected_information_gain_rank[0], 1);
    }

    #[test]
    fn plan_caps_sub_question_count() {
        let many: Vec<String> = (0..(MAX_SUB_QUESTIONS + 5))
            .map(|i| format!("question {i}"))
            .collect();
        let plan = WebResearchPlan::with_ranked_sub_questions(
            "root",
            many,
            DEFAULT_PLANNED_SEARCH_BUDGET,
        );
        assert_eq!(plan.sub_questions.len(), MAX_SUB_QUESTIONS);
        assert_eq!(plan.expected_information_gain_rank.len(), MAX_SUB_QUESTIONS);
    }

    #[test]
    fn plan_truncates_long_sub_questions() {
        let long = "x".repeat(SUB_QUESTION_CHAR_CAP * 2);
        let plan = WebResearchPlan::with_ranked_sub_questions(
            "root",
            vec![long],
            DEFAULT_PLANNED_SEARCH_BUDGET,
        );
        assert_eq!(plan.sub_questions[0].text.chars().count(), SUB_QUESTION_CHAR_CAP);
    }

    #[test]
    fn plan_minimum_budget_is_one() {
        let plan = WebResearchPlan::with_ranked_sub_questions("root", vec![], 0);
        assert_eq!(plan.planned_search_budget, 1);
    }

    #[test]
    fn mark_interactive_then_approved() {
        let mut plan =
            WebResearchPlan::with_ranked_sub_questions("root", vec!["q".into()], 4);
        assert!(!plan.interactive);
        assert!(!plan.approved_by_user);
        plan.mark_interactive();
        plan.mark_approved();
        assert!(plan.interactive);
        assert!(plan.approved_by_user);
    }
}
