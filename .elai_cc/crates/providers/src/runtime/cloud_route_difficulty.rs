//! E-20 -- `BackendDifficultyAdapter` scaffold.
//!
//! A `RuntimeAdapter` whose purpose is NOT inference but PRE-INFERENCE
//! signal emission: given a prompt, it produces a 3-arm difficulty
//! verdict (easy / medium / hard) that
//! [`elai_orchestrator::cost::cost_router::route_for_difficulty_and_budget`]
//! consumes to choose a real backend.
//!
//! The adapter sits behind the same [`crate::runtime::RuntimeAdapter`]
//! trait as the live inference adapters so the orchestrator's
//! `RuntimeOrchestrator` can dispatch it identically. Its `complete()` /
//! `stream()` methods return `RuntimeAdapterError::NotImplemented` until
//! the live RouteLLM weights (arXiv:2406.18665, ~110M-param BERT-scale
//! preference-trained router) land via a follow-up E-20.B plan. Until
//! then the [`classify_difficulty`] helper provides a deterministic
//! sentinel difficulty score using the same heuristic as
//! `cost::cost_router::TaskDifficultyClassifier` so the wire shape +
//! routing decisions match what the live classifier will produce.
//!
//! Naming: deliberately `BackendDifficultyAdapter`, NOT
//! `TaskDifficultyClassifier`, to avoid shadowing C-46's
//! `orchestrator::routing::difficulty::TaskDifficultyClassifier`
//! (AgentRole router) AND E-39's
//! `orchestrator::cost::cost_router::TaskDifficultyClassifier`
//! (RouteDecision router). Per E-20-decisions.md DECISION-2.
//!
//! Source grounding (verified May 2026 via WebSearch):
//!   * arXiv:2406.18665 RouteLLM LMSYS 2024 -- BERT-scale router,
//!     ~2x cost reduction at near-GPT-4 quality on Chatbot Arena
//!     preference data (80k battles).
//!   * arXiv:2511.03808 Prompt-Difficulty Prediction Nov 2025 --
//!     intermediate-representation activation probes from s1.1-32B;
//!     deferred to E-20.B (needs labeled hidden-state corpus).

use crate::runtime::types::{
    unavailable, CompletionConfig, CompletionResponse, Message, RuntimeAdapter,
    RuntimeAdapterError, RuntimeBackend, RuntimeCapabilities, RuntimeFuture, RuntimeHealth,
    RuntimeHealthFuture, StreamChunk,
};
use futures_util::stream::{self, BoxStream};

/// 3-arm difficulty bin emitted by the adapter. Mirrors
/// [`crate::runtime::cloud_route_difficulty::DifficultyBin`] / the
/// orchestrator's `cost::cloud_route::semantic_reasoning::DifficultyBin`
/// shape. Defined here so the providers crate has no upward dependency
/// on the orchestrator.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DifficultyBin {
    Easy,
    Medium,
    Hard,
}

impl DifficultyBin {
    /// Snake-case wire label.
    #[must_use]
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Easy => "easy",
            Self::Medium => "medium",
            Self::Hard => "hard",
        }
    }
}

/// Composite difficulty verdict carried over the
/// `BackendDifficultyAdapter` -> `cost_router` boundary.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct DifficultyVerdict {
    /// 3-arm bin.
    pub bin: DifficultyBin,
    /// Raw score in `[0.0, 1.0]`. 0.0 = trivial, 1.0 = frontier-hard.
    pub score: f32,
    /// Whether the adapter recommends a cloud dispatch (composes with
    /// the budget-side gate in `cost_router`).
    pub cloud_recommended: bool,
    /// Snake-case classifier name. `"heuristic"` until the live
    /// RouteLLM weights ship; will become `"routellm"` after E-20.B.
    pub classifier: &'static str,
}

/// Boundary thresholds matching the
/// [`elai_orchestrator::cost::cost_router`] difficulty bins
/// (`DIFFICULTY_EASY_CEIL = 0.33`, `DIFFICULTY_HARD_CEIL = 0.66`). The
/// constants are repeated here so the providers crate stays standalone.
pub const DIFFICULTY_EASY_CEIL: f32 = 0.33;
pub const DIFFICULTY_HARD_CEIL: f32 = 0.66;

const FRONTIER_KEYWORDS: &[&str] = &[
    "algorithm",
    "prove",
    "optimize",
    "theorem",
    "derive",
    "parallel",
    "lock-free",
    "monad",
    "formal-verify",
    "concurrency",
    "refactor",
    "architecture",
];

/// Pure helper -- deterministic 3-feature average matching
/// `cost_router::TaskDifficultyClassifier`. The features are:
///   * `trigram_unique_ratio` -- lexical variety (cap 1.0)
///   * `length_factor` -- log10(1 + chars) / 4 (cap 1.0)
///   * `keyword_signal` -- frontier-keyword count / 8 (cap 1.0)
#[must_use]
pub fn classify_difficulty(prompt: &str) -> DifficultyVerdict {
    let trigrams = unique_trigram_ratio(prompt);
    let length = length_factor(prompt);
    let keywords = keyword_signal(prompt);
    let score = ((trigrams + length + keywords) / 3.0).clamp(0.0, 1.0);
    let bin = if score <= DIFFICULTY_EASY_CEIL {
        DifficultyBin::Easy
    } else if score <= DIFFICULTY_HARD_CEIL {
        DifficultyBin::Medium
    } else {
        DifficultyBin::Hard
    };
    let cloud_recommended = matches!(bin, DifficultyBin::Medium | DifficultyBin::Hard);
    DifficultyVerdict {
        bin,
        score,
        cloud_recommended,
        classifier: "heuristic",
    }
}

fn unique_trigram_ratio(text: &str) -> f32 {
    if text.len() < 3 {
        return 0.0;
    }
    let chars: Vec<char> = text.chars().collect();
    if chars.len() < 3 {
        return 0.0;
    }
    let total = (chars.len() - 2) as f32;
    let mut seen = std::collections::HashSet::new();
    for w in chars.windows(3) {
        seen.insert((w[0], w[1], w[2]));
    }
    let ratio = (seen.len() as f32) / total;
    ratio.clamp(0.0, 1.0)
}

fn length_factor(text: &str) -> f32 {
    let len = text.chars().count() as f32;
    let log10 = (1.0 + len).log10();
    (log10 / 4.0).clamp(0.0, 1.0)
}

fn keyword_signal(text: &str) -> f32 {
    let lower = text.to_lowercase();
    let count = FRONTIER_KEYWORDS
        .iter()
        .filter(|kw| lower.contains(*kw))
        .count() as f32;
    (count / 8.0).clamp(0.0, 1.0)
}

/// `RuntimeAdapter` impl that reports difficulty rather than running
/// inference. The orchestrator dispatches this BEFORE the cost-router
/// picks a backend; the live inference adapter is dispatched
/// afterwards.
#[derive(Debug, Default)]
pub struct BackendDifficultyAdapter;

impl BackendDifficultyAdapter {
    /// Construct with the default heuristic classifier.
    #[must_use]
    pub fn new() -> Self {
        Self
    }

    /// Direct classifier call -- callers that do not need the
    /// `RuntimeAdapter` trait surface use this. Equivalent to
    /// [`classify_difficulty`].
    #[must_use]
    pub fn classify(&self, prompt: &str) -> DifficultyVerdict {
        classify_difficulty(prompt)
    }
}

impl RuntimeAdapter for BackendDifficultyAdapter {
    fn backend(&self) -> RuntimeBackend {
        // The adapter does NOT have its own RuntimeBackend variant
        // (would require a SCHEMA bump on RuntimeBackend, rejected per
        // E-20-decisions.md DECISION-3). We reuse Sidecar as the
        // canonical "local" home for the classifier signal; the
        // RuntimeOrchestrator dispatches this adapter via a separate
        // PerRoleRuntime injection point, not the regular backend
        // selector.
        RuntimeBackend::Sidecar
    }

    fn capabilities(&self) -> RuntimeCapabilities {
        // Empty caps: this adapter does not stream / complete; it
        // only emits difficulty verdicts. The orchestrator MUST NOT
        // call `complete()` / `stream()` on it.
        RuntimeCapabilities::none()
    }

    fn health_check(&self) -> RuntimeHealthFuture<'_> {
        // Pure-CPU adapter -- always healthy.
        Box::pin(async move { RuntimeHealth::Ready })
    }

    fn complete(
        &self,
        _messages: Vec<Message>,
        _config: CompletionConfig,
    ) -> RuntimeFuture<'_, CompletionResponse> {
        Box::pin(async move {
            Err(RuntimeAdapterError::NotImplemented {
                backend: RuntimeBackend::Sidecar,
                hint: "BackendDifficultyAdapter.complete() -- use classify() instead",
            })
        })
    }

    fn stream(
        &self,
        _messages: Vec<Message>,
        _config: CompletionConfig,
    ) -> BoxStream<'static, Result<StreamChunk, RuntimeAdapterError>> {
        let chunks: Vec<Result<StreamChunk, RuntimeAdapterError>> = vec![
            Ok(StreamChunk::Warning {
                message:
                    "BackendDifficultyAdapter does not stream; use classify() for difficulty."
                        .to_string(),
            }),
            Ok(StreamChunk::Stop {
                reason: Some("adapter does not stream".to_string()),
            }),
        ];
        Box::pin(stream::iter(chunks))
    }
}

/// Convenience: report `Unavailable` for any future caller that asks
/// about a "difficulty classifier backend". Mirrors the
/// [`crate::runtime::types::unavailable`] helper shape.
#[must_use]
pub fn classifier_unavailable_reason() -> RuntimeHealth {
    unavailable("BackendDifficultyAdapter is signal-only (no inference)")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn difficulty_bin_labels_stable() {
        assert_eq!(DifficultyBin::Easy.as_str(), "easy");
        assert_eq!(DifficultyBin::Medium.as_str(), "medium");
        assert_eq!(DifficultyBin::Hard.as_str(), "hard");
    }

    #[test]
    fn empty_prompt_is_easy() {
        let verdict = classify_difficulty("");
        assert_eq!(verdict.bin, DifficultyBin::Easy);
        assert!(!verdict.cloud_recommended);
    }

    #[test]
    fn very_short_prompt_is_easy() {
        // Very short + low trigram variety + no keywords -> Easy.
        let verdict = classify_difficulty("hi");
        assert_eq!(verdict.bin, DifficultyBin::Easy);
    }

    #[test]
    fn long_keyword_heavy_prompt_is_hard() {
        let prompt =
            "Please prove correctness, optimize the algorithm, and derive the theorem for the \
             lock-free concurrency primitive. Formal-verify the parallel monad. Refactor the \
             architecture to be lock-free and monad-based and parallel.";
        let verdict = classify_difficulty(prompt);
        assert_eq!(verdict.bin, DifficultyBin::Hard);
        assert!(verdict.cloud_recommended);
    }

    #[test]
    fn classifier_label_is_heuristic_until_routellm_ships() {
        let verdict = classify_difficulty("anything");
        assert_eq!(verdict.classifier, "heuristic");
    }

    #[tokio::test]
    async fn adapter_complete_not_implemented() {
        let adapter = BackendDifficultyAdapter::new();
        let err = adapter
            .complete(Vec::new(), CompletionConfig::default())
            .await
            .expect_err("must error");
        assert!(matches!(err, RuntimeAdapterError::NotImplemented { .. }));
    }

    #[tokio::test]
    async fn adapter_health_is_ready() {
        let adapter = BackendDifficultyAdapter::new();
        let health = adapter.health_check().await;
        assert!(matches!(health, RuntimeHealth::Ready));
    }
}
