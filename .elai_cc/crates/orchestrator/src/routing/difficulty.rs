//! C-46 — Task difficulty classifier with heuristic fallback.
//!
//! The classifier emits a scalar difficulty score in `[0, 1]` over a
//! [`RoutingTask`] description. The score composes three optional
//! signals via a simple weighted average:
//!
//!   - HEURISTIC (always available): pure features over the task text
//!     and metadata (token count, code-block presence, question-
//!     complexity keywords, file count, multi-turn flag). No external
//!     dependency; deterministic on `(text, file_count, multi_turn)`.
//!   - REPO-CENTRALITY (optional, async): caller supplies a closure
//!     returning a centrality score in `[0, 1]` from the DKG. When the
//!     closure returns `None` the signal is dropped from the average.
//!     ELAI's DKG access lives in `crate::symbol_graph` / the
//!     knowledge crate; callers that have no DKG return `None`.
//!   - EMBEDDING-COSINE (optional, async): caller supplies a closure
//!     returning a cosine similarity in `[0, 1]` against a fixed hard-
//!     task reference set. When the sidecar embedding model is
//!     unavailable the closure returns `None` and the signal is
//!     dropped.
//!
//! Composition: the final score is the unweighted mean of the
//! AVAILABLE signals. If only the heuristic signal is available the
//! score is the heuristic value verbatim. If all three signals fire
//! the score is `(heuristic + centrality + cosine) / 3`. NaN-safety:
//! every signal's contribution is clamped to `[0, 1]` before
//! aggregation so out-of-band closure return values never poison the
//! final score.
//!
//! Paper anchor: arXiv:2605.05007 Section 5 — the orchestrator
//! observes only verifier signal + token cost, not auxiliary heads.
//! The difficulty classifier is an ELAI design hypothesis providing
//! pre-dispatch routing input; it does NOT replace the verifier
//! signal (the verifier remains binary-only per the paper's
//! `verifier_binary_only` invariant).

use std::future::Future;

use serde::{Deserialize, Serialize};

/// Input shape for [`TaskDifficultyClassifier::heuristic_score`].
///
/// Mirrors the relevant fields a `TaskLedger` would expose without
/// taking a direct dependency on the ledger — the classifier must be
/// callable from contract tests and the CLI without a live SSB or
/// DualLedger.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct RoutingTask {
    /// Raw task description / query / prompt text. The classifier
    /// counts tokens by simple whitespace split (heuristic-only;
    /// production callers can preprocess with the same tokenizer
    /// they use elsewhere).
    pub text: String,
    /// Count of files referenced by the task (zero when the task is
    /// not file-scoped). Higher counts push the heuristic score up.
    pub file_count: u32,
    /// True when the task spans multiple turns (chat-style). Multi-
    /// turn tasks generally need more decomposition work, pushing the
    /// heuristic score up.
    pub multi_turn: bool,
    /// Caller-supplied stable id for the task. Hashed via blake3-
    /// prefix u64 by [`super::selective::task_id_hash`] before
    /// emission to SSB — never sent in plaintext.
    pub task_id: String,
}

impl RoutingTask {
    /// Minimal constructor with the text-only path. `file_count` and
    /// `multi_turn` default to 0 / false respectively.
    #[must_use]
    pub fn new_text(task_id: impl Into<String>, text: impl Into<String>) -> Self {
        Self {
            text: text.into(),
            file_count: 0,
            multi_turn: false,
            task_id: task_id.into(),
        }
    }
}

/// Decomposed heuristic features. Exposed so callers (the dashboard,
/// pilot artifacts) can render the same features the classifier
/// observes without re-running the classifier in foreign code.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct HeuristicFeatures {
    /// Raw whitespace-split token count of the task text.
    pub token_count: u32,
    /// True when the task text contains at least one Markdown-style
    /// fenced code block (``` triple-backtick) — a strong proxy for
    /// "this task has embedded code context".
    pub has_code_block: bool,
    /// Count of question-complexity keywords found in the task text
    /// (e.g. "design", "architecture", "refactor", "why", "how").
    pub question_keyword_count: u32,
    /// `file_count` from the task (mirrored for completeness).
    pub file_count: u32,
    /// `multi_turn` from the task (mirrored for completeness).
    pub multi_turn: bool,
}

/// Question-complexity keywords. Each occurrence in the task text
/// increments [`HeuristicFeatures::question_keyword_count`]. The list
/// is intentionally short + closed (no regex, no NLP — keep the
/// heuristic deterministic + cheap).
const COMPLEXITY_KEYWORDS: &[&str] = &[
    "design",
    "architecture",
    "refactor",
    "implement",
    "integrate",
    "debug",
    "diagnose",
    "investigate",
    "optimize",
    "trade-off",
    "tradeoff",
    "why",
    "how",
    "compare",
    "evaluate",
];

/// Thresholds for the heuristic-score transform. Encoded as `const`
/// so the contract test can cite + verify them without magic-number
/// repetition.
const TOKEN_COUNT_SATURATION: f32 = 500.0;
const FILE_COUNT_SATURATION: f32 = 10.0;
const KEYWORD_SATURATION: f32 = 5.0;

const W_TOKENS: f32 = 0.30;
const W_KEYWORDS: f32 = 0.30;
const W_FILES: f32 = 0.20;
const W_CODE_BLOCK: f32 = 0.10;
const W_MULTI_TURN: f32 = 0.10;

/// Task difficulty classifier. Stateless — `new()` is a marker so the
/// type can grow internal state (e.g. a cached repo-map snapshot) in
/// follow-on plans without breaking the call surface.
#[derive(Debug, Clone, Copy, Default)]
pub struct TaskDifficultyClassifier;

impl TaskDifficultyClassifier {
    /// Construct a fresh classifier instance.
    #[must_use]
    pub fn new() -> Self {
        Self
    }

    /// Extract heuristic features from a task. Pure function; the
    /// dashboard + pilot CLI render these directly.
    #[must_use]
    pub fn features(task: &RoutingTask) -> HeuristicFeatures {
        let text = task.text.as_str();
        let token_count = u32::try_from(
            text.split_whitespace().count(),
        )
        .unwrap_or(u32::MAX);
        let has_code_block = text.contains("```");
        let lower = text.to_ascii_lowercase();
        let mut keyword_count: u32 = 0;
        for kw in COMPLEXITY_KEYWORDS {
            // count non-overlapping matches; bound on token_count so
            // pathological inputs cannot wrap u32.
            let occurrences = u32::try_from(lower.matches(kw).count())
                .unwrap_or(u32::MAX);
            keyword_count = keyword_count.saturating_add(occurrences);
        }
        HeuristicFeatures {
            token_count,
            has_code_block,
            question_keyword_count: keyword_count,
            file_count: task.file_count,
            multi_turn: task.multi_turn,
        }
    }

    /// Heuristic-only difficulty score in `[0, 1]`. Always available
    /// (no external dependency). Each feature contributes via a
    /// saturating linear ramp, then the five contributions are
    /// weighted-averaged using the `W_*` constants above.
    #[must_use]
    pub fn heuristic_score(task: &RoutingTask) -> f32 {
        let f = Self::features(task);

        let token_ramp = (f.token_count as f32 / TOKEN_COUNT_SATURATION).clamp(0.0, 1.0);
        let keyword_ramp =
            (f.question_keyword_count as f32 / KEYWORD_SATURATION).clamp(0.0, 1.0);
        let file_ramp = (f.file_count as f32 / FILE_COUNT_SATURATION).clamp(0.0, 1.0);
        let code_block_contribution = if f.has_code_block { 1.0 } else { 0.0 };
        let multi_turn_contribution = if f.multi_turn { 1.0 } else { 0.0 };

        let score = W_TOKENS * token_ramp
            + W_KEYWORDS * keyword_ramp
            + W_FILES * file_ramp
            + W_CODE_BLOCK * code_block_contribution
            + W_MULTI_TURN * multi_turn_contribution;

        // Clamp defensively — sum of weights is 1.0 by construction so
        // this is a guard against future weight edits drifting the sum.
        score.clamp(0.0, 1.0)
    }

    /// Full async classify path with optional DKG centrality + sidecar
    /// cosine signals. The two closures are independent; either may
    /// return `None` and the classifier drops that signal from the
    /// average. Returns a score in `[0, 1]`.
    ///
    /// If BOTH optional signals return `None`, the score equals
    /// [`Self::heuristic_score`] verbatim — the cold-start /
    /// offline-build path.
    pub async fn classify<FC, FCFut, FE, FEFut>(
        task: &RoutingTask,
        repo_centrality: Option<FC>,
        embedding_cosine: Option<FE>,
    ) -> f32
    where
        FC: FnOnce(&RoutingTask) -> FCFut,
        FCFut: Future<Output = Option<f32>>,
        FE: FnOnce(&RoutingTask) -> FEFut,
        FEFut: Future<Output = Option<f32>>,
    {
        let heuristic = Self::heuristic_score(task);

        let centrality_signal = match repo_centrality {
            Some(f) => f(task).await.map(|v| v.clamp(0.0, 1.0)),
            None => None,
        };
        let cosine_signal = match embedding_cosine {
            Some(f) => f(task).await.map(|v| v.clamp(0.0, 1.0)),
            None => None,
        };

        let mut total = heuristic;
        let mut n: f32 = 1.0;
        if let Some(c) = centrality_signal {
            total += c;
            n += 1.0;
        }
        if let Some(c) = cosine_signal {
            total += c;
            n += 1.0;
        }
        (total / n).clamp(0.0, 1.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn heuristic_score_in_unit_interval_for_empty_task() {
        let t = RoutingTask::new_text("t1", "");
        let s = TaskDifficultyClassifier::heuristic_score(&t);
        assert!((0.0..=1.0).contains(&s), "score {s} must be in [0, 1]");
        assert!(s.abs() < f32::EPSILON, "empty task scores zero");
    }

    #[test]
    fn heuristic_score_higher_for_complex_task() {
        let trivial = RoutingTask::new_text("t1", "hi");
        let complex = RoutingTask {
            text: "refactor the orchestrator architecture: investigate why the dispatcher \
                   integrate path fails. ```rust let x = 1; ``` how does it interact with \
                   the auditor design? compare two approaches and evaluate trade-off."
                .into(),
            file_count: 6,
            multi_turn: true,
            task_id: "t2".into(),
        };
        let s_trivial = TaskDifficultyClassifier::heuristic_score(&trivial);
        let s_complex = TaskDifficultyClassifier::heuristic_score(&complex);
        assert!(
            s_complex > s_trivial,
            "complex task ({s_complex}) must score higher than trivial ({s_trivial})"
        );
        assert!(
            (0.0..=1.0).contains(&s_complex),
            "complex score {s_complex} must be in [0, 1]"
        );
    }

    #[test]
    fn features_count_code_block_and_keywords() {
        let t = RoutingTask {
            text: "design and integrate ```rust fn x() {}``` how does refactor work".into(),
            file_count: 0,
            multi_turn: false,
            task_id: "t3".into(),
        };
        let f = TaskDifficultyClassifier::features(&t);
        assert!(f.has_code_block);
        // design + integrate + how + refactor = 4 (case-insensitive)
        assert!(f.question_keyword_count >= 4);
    }

    #[tokio::test]
    async fn classify_falls_back_to_heuristic_when_no_optional_signals() {
        let t = RoutingTask::new_text("t1", "implement a small refactor");
        let h = TaskDifficultyClassifier::heuristic_score(&t);
        let no_centrality: Option<fn(&RoutingTask) -> std::future::Ready<Option<f32>>> = None;
        let no_embedding: Option<fn(&RoutingTask) -> std::future::Ready<Option<f32>>> = None;
        let s = TaskDifficultyClassifier::classify(&t, no_centrality, no_embedding).await;
        assert!((s - h).abs() < f32::EPSILON);
    }

    #[tokio::test]
    async fn classify_combines_optional_signals() {
        let t = RoutingTask::new_text("t1", "x");
        let h = TaskDifficultyClassifier::heuristic_score(&t);
        let centrality = Some(|_t: &RoutingTask| std::future::ready(Some(1.0_f32)));
        let cosine = Some(|_t: &RoutingTask| std::future::ready(Some(1.0_f32)));
        let s = TaskDifficultyClassifier::classify(&t, centrality, cosine).await;
        // (h + 1 + 1) / 3 — must be greater than the heuristic alone.
        assert!(s > h);
        assert!(s <= 1.0);
    }

    #[tokio::test]
    async fn classify_clamps_out_of_band_optional_signals() {
        let t = RoutingTask::new_text("t1", "x");
        let centrality = Some(|_t: &RoutingTask| std::future::ready(Some(5.0_f32)));
        let cosine = Some(|_t: &RoutingTask| std::future::ready(Some(-2.0_f32)));
        let s = TaskDifficultyClassifier::classify(&t, centrality, cosine).await;
        assert!((0.0..=1.0).contains(&s));
    }
}
