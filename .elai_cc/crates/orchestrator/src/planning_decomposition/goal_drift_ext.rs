//! Goal-drift cosine extension to the shipped Jaccard-only
//! [`crate::goal_drift_canary`] module.
//!
//! Plan task 4 of E-46. The shipped
//! [`crate::goal_drift_canary::GoalDriftCanary::fired`] /
//! [`crate::goal_drift_canary::measure_drift`] surface computes Jaccard
//! distance only. The plan requires BOTH a Jaccard arm AND a cosine arm
//! plus a 2-of-2 consecutive-below-threshold trigger.
//!
//! Cosine source: per the cross-check exploration GAP §78, the sidecar
//! exposes no `/embed` endpoint and the B-40 FramedIdea struct carries no
//! embedding field. This module computes a TERM-FREQUENCY cosine over
//! tokenized acceptance-criteria text (using
//! [`super::skill_prior::blake3_bucket_vector`] + [`super::skill_prior::cosine_u32`]
//! for the substitute embedding) — no external dependency, no sidecar
//! coupling. When (and only when) a sidecar `/embed` route ships, this
//! module's cosine call swaps to that endpoint without changing the
//! public `evaluate()` signature.
//!
//! Reference: arXiv:2505.02709 (AIES 2025) — Goal Drift Evaluation; PRM
//! supervision corrects deviations; cosine on goal embedding is
//! canonical.

use serde::{Deserialize, Serialize};

use super::decomposition_entropy::tokenize_simple;
use super::skill_prior::{blake3_bucket_vector, cosine_u32};

/// Default Jaccard distance threshold below which a phase is flagged
/// drifting (plan task 4 §43). Surface in `elai_cc.toml`
/// `[planning.goal_drift] jaccard_threshold` to override.
///
/// Note: stored as 1 - Jaccard SIMILARITY threshold per plan body — the
/// plan says "jaccard < 0.25" meaning Jaccard similarity below 0.25; we
/// follow that interpretation (similarity, not distance).
pub const JACCARD_SIMILARITY_THRESHOLD: f32 = 0.25;

/// Default cosine similarity threshold below which a phase is flagged
/// drifting (plan task 4 §43). Surface in `elai_cc.toml`
/// `[planning.goal_drift] cosine_threshold` to override.
pub const COSINE_SIMILARITY_THRESHOLD: f32 = 0.55;

/// Bucket count for the substitute-cosine vector.
pub const GOAL_DRIFT_BUCKET_COUNT: usize = 96;

/// 200-word static stopword list (plan task 4 §43). Truncated set; the
/// goal-drift use case is concerned with content tokens, not stop-word
/// noise. Curated from the canonical NLTK English stop-word list (verified
/// May 2026 via WebSearch).
const STOPWORDS: &[&str] = &[
    "a", "about", "above", "after", "again", "against", "ain", "all", "am",
    "an", "and", "any", "are", "aren", "as", "at", "be", "because", "been",
    "before", "being", "below", "between", "both", "but", "by", "can", "couldn",
    "d", "did", "didn", "do", "does", "doesn", "doing", "don", "down", "during",
    "each", "few", "for", "from", "further", "had", "hadn", "has", "hasn",
    "have", "haven", "having", "he", "her", "here", "hers", "herself", "him",
    "himself", "his", "how", "i", "if", "in", "into", "is", "isn", "it", "its",
    "itself", "just", "ll", "m", "ma", "me", "might", "mightn", "more", "most",
    "mustn", "my", "myself", "needn", "no", "nor", "not", "now", "o", "of",
    "off", "on", "once", "only", "or", "other", "our", "ours", "ourselves",
    "out", "over", "own", "re", "s", "same", "shan", "she", "should", "shouldn",
    "so", "some", "such", "t", "than", "that", "the", "their", "theirs", "them",
    "themselves", "then", "there", "these", "they", "this", "those", "through",
    "to", "too", "under", "until", "up", "ve", "very", "was", "wasn", "we",
    "were", "weren", "what", "when", "where", "which", "while", "who", "whom",
    "why", "will", "with", "won", "would", "wouldn", "y", "you", "your", "yours",
    "yourself", "yourselves", "could", "shall", "may", "must", "ought", "around",
    "above", "across", "along", "among", "behind", "beneath", "beside", "beyond",
    "during", "except", "inside", "outside", "since", "toward", "towards",
    "upon", "within", "without", "above", "either", "neither", "whether",
    "although", "however", "moreover", "nevertheless", "therefore", "thus",
    "hence", "indeed", "rather", "instead", "perhaps", "really", "actually",
    "literally", "currently", "previously", "subsequently", "consequently",
    "thereby", "whereby", "thereof", "wherein", "whereas", "whereupon",
];

/// Stopword check (case-insensitive ascii match).
#[must_use]
pub fn is_stopword(tok: &str) -> bool {
    STOPWORDS.binary_search(&tok).is_ok()
        || STOPWORDS.iter().any(|sw| sw.eq_ignore_ascii_case(tok))
}

/// Tokenize + filter stop words for the Jaccard arm.
#[must_use]
pub fn tokenize_no_stopwords(s: &str) -> Vec<String> {
    tokenize_simple(s)
        .into_iter()
        .filter(|t| !is_stopword(t))
        .collect()
}

/// Jaccard similarity over two token-bag sets in `[0, 1]`. Returns 1.0
/// when both inputs are empty (degenerate convention — empty == empty).
#[must_use]
pub fn jaccard_similarity(a: &[String], b: &[String]) -> f32 {
    if a.is_empty() && b.is_empty() {
        return 1.0;
    }
    let set_a: rustc_hash::FxHashSet<&str> = a.iter().map(|s| s.as_str()).collect();
    let set_b: rustc_hash::FxHashSet<&str> = b.iter().map(|s| s.as_str()).collect();
    let intersection = set_a.intersection(&set_b).count() as f32;
    let union = set_a.union(&set_b).count() as f32;
    if union <= 0.0 {
        return 0.0;
    }
    intersection / union
}

/// Goal-drift report. Carries BOTH jaccard and cosine arms (per plan
/// stub_ban §89 "no goal-drift canary emit without BOTH jaccard and
/// cosine populated") plus a `prev_below_threshold` flag the caller
/// threads through across consecutive Phase evaluations.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GoalDriftReport {
    /// Jaccard similarity in [0, 1] of phase-acceptance vs idea-success
    /// token bags.
    pub jaccard: f32,
    /// Cosine similarity in [0, 1] of phase-acceptance vs idea-success
    /// blake3-bucket vectors (substitute embedding).
    pub cosine: f32,
    /// True iff (jaccard < JACCARD_SIMILARITY_THRESHOLD) OR
    /// (cosine < COSINE_SIMILARITY_THRESHOLD).
    pub below_threshold: bool,
    /// Caller-supplied: was the PREVIOUS phase also below threshold ?
    /// Plan task 4 §43 requires the GOAL_DRIFT bubble-up to fire only on
    /// two consecutive below-threshold Phases.
    pub prev_below_threshold: bool,
    /// True iff `below_threshold && prev_below_threshold` — i.e. fire the
    /// `GOAL_DRIFT` cascade.
    pub fire_cascade: bool,
}

/// Goal-drift evaluator. Encapsulates the parent-idea baseline tokens +
/// bucket vector so consecutive phase evaluations don't recompute them.
/// Constructed from the parent-Idea's success-criteria text once at
/// session start (B-40 `FramedIdea` provides the text).
#[derive(Debug, Clone)]
pub struct GoalDriftEvaluator {
    idea_tokens_no_stop: Vec<String>,
    idea_buckets: Vec<u32>,
    jaccard_threshold: f32,
    cosine_threshold: f32,
}

impl GoalDriftEvaluator {
    /// New evaluator from raw idea-success-criteria text. Tokens are
    /// lowercased + stopword-filtered; buckets are blake3-derived.
    #[must_use]
    pub fn from_idea_text(idea_text: &str) -> Self {
        Self::with_thresholds(
            idea_text,
            JACCARD_SIMILARITY_THRESHOLD,
            COSINE_SIMILARITY_THRESHOLD,
        )
    }

    /// Same as [`Self::from_idea_text`] but with caller-supplied
    /// thresholds. Each threshold is clamped to `[0.0, 1.0]`.
    #[must_use]
    pub fn with_thresholds(
        idea_text: &str,
        jaccard_threshold: f32,
        cosine_threshold: f32,
    ) -> Self {
        Self {
            idea_tokens_no_stop: tokenize_no_stopwords(idea_text),
            idea_buckets: blake3_bucket_vector(idea_text, GOAL_DRIFT_BUCKET_COUNT),
            jaccard_threshold: jaccard_threshold.clamp(0.0, 1.0),
            cosine_threshold: cosine_threshold.clamp(0.0, 1.0),
        }
    }

    /// Evaluate the drift of `phase_text` against the parent idea.
    /// `prev_below_threshold` is supplied by the caller (tracks the prior
    /// Phase's verdict so the 2-of-2 trigger fires correctly).
    #[must_use]
    pub fn evaluate(&self, phase_text: &str, prev_below_threshold: bool) -> GoalDriftReport {
        let phase_tokens = tokenize_no_stopwords(phase_text);
        let phase_buckets = blake3_bucket_vector(phase_text, GOAL_DRIFT_BUCKET_COUNT);
        let jaccard = jaccard_similarity(&self.idea_tokens_no_stop, &phase_tokens);
        let cosine = cosine_u32(&self.idea_buckets, &phase_buckets);
        let below_threshold =
            jaccard < self.jaccard_threshold || cosine < self.cosine_threshold;
        let fire_cascade = below_threshold && prev_below_threshold;
        GoalDriftReport {
            jaccard,
            cosine,
            below_threshold,
            prev_below_threshold,
            fire_cascade,
        }
    }

    /// Inspected by the contract test — the canonical thresholds in use.
    #[must_use]
    pub fn thresholds(&self) -> (f32, f32) {
        (self.jaccard_threshold, self.cosine_threshold)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn jaccard_identical_token_bags_one() {
        let a: Vec<String> = vec!["user".into(), "form".into(), "schema".into()];
        let b = a.clone();
        let sim = jaccard_similarity(&a, &b);
        assert!((sim - 1.0).abs() < 1e-5);
    }

    #[test]
    fn jaccard_disjoint_token_bags_zero() {
        let a: Vec<String> = vec!["user".into(), "form".into()];
        let b: Vec<String> = vec!["graphics".into(), "rendering".into()];
        let sim = jaccard_similarity(&a, &b);
        assert_eq!(sim, 0.0);
    }

    #[test]
    fn jaccard_empty_inputs_one() {
        let sim = jaccard_similarity(&[], &[]);
        assert_eq!(sim, 1.0);
    }

    #[test]
    fn stopword_filtering_removes_known_words() {
        let tokens = tokenize_no_stopwords("The user submits the form and validates the schema");
        // "the" / "and" should be filtered
        for t in &tokens {
            assert_ne!(t, "the");
            assert_ne!(t, "and");
        }
        assert!(tokens.iter().any(|t| t == "user"));
        assert!(tokens.iter().any(|t| t == "submits"));
        assert!(tokens.iter().any(|t| t == "schema"));
    }

    #[test]
    fn evaluator_aligned_phase_below_neither_threshold() {
        let idea = "user submits form validates schema stores database indexed tested endpoint secured rate limited audit logged monitored deployable observable reliable verified";
        let phase = idea; // identical content
        let ev = GoalDriftEvaluator::from_idea_text(idea);
        let report = ev.evaluate(phase, false);
        assert_eq!(report.jaccard, 1.0);
        assert!(report.cosine > 0.99, "got cosine={}", report.cosine);
        assert!(!report.below_threshold);
        assert!(!report.fire_cascade);
    }

    #[test]
    fn evaluator_disjoint_phase_below_both_thresholds() {
        let idea = "user submits form validates schema stores database";
        let phase = "graphics shader pipeline raster vertex fragment depth buffer";
        let ev = GoalDriftEvaluator::from_idea_text(idea);
        let report = ev.evaluate(phase, false);
        assert!(report.jaccard < JACCARD_SIMILARITY_THRESHOLD);
        assert!(report.cosine < COSINE_SIMILARITY_THRESHOLD);
        assert!(report.below_threshold);
    }

    #[test]
    fn evaluator_fire_cascade_requires_two_in_row() {
        let idea = "user submits form validates schema";
        let drifty_phase = "graphics shader pipeline raster";
        let ev = GoalDriftEvaluator::from_idea_text(idea);
        // First phase below threshold but no prior -> no cascade
        let r1 = ev.evaluate(drifty_phase, false);
        assert!(r1.below_threshold);
        assert!(!r1.fire_cascade);
        // Second phase below threshold with prior_below=true -> cascade
        let r2 = ev.evaluate(drifty_phase, true);
        assert!(r2.below_threshold);
        assert!(r2.fire_cascade);
    }

    #[test]
    fn evaluator_thresholds_clamped() {
        let ev = GoalDriftEvaluator::with_thresholds("idea text", -0.5, 2.0);
        let (j, c) = ev.thresholds();
        assert_eq!(j, 0.0);
        assert_eq!(c, 1.0);
    }

    #[test]
    fn report_carries_both_arms_populated() {
        // Plan stub_ban L89: no goal-drift canary emit without BOTH jaccard
        // and cosine populated. The struct shape itself enforces this; this
        // test verifies the evaluator always returns both arms.
        let ev = GoalDriftEvaluator::from_idea_text("idea text");
        let report = ev.evaluate("phase text", false);
        assert!(report.jaccard.is_finite());
        assert!(report.cosine.is_finite());
        assert!((0.0..=1.0).contains(&report.jaccard));
        assert!((0.0..=1.0).contains(&report.cosine));
    }
}
