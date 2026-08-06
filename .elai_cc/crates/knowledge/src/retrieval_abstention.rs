//! J-73 Slice A — Tier-1 retrieval-abstention pure module (OBSERVE-ONLY).
//!
//! Local model-free sufficiency heuristic for deciding whether the retrieval
//! cascade should skip for a given query. Reuses the *idea* of calibrated
//! score vs threshold from the orchestrator's answer-abstention, without
//! importing it (elai-knowledge cannot depend on elai-orchestrator).
//!
//! Tier-1 is always observe-only (GD-4): `enforce_skip` always returns false
//! until FP is bounded on >=3 real runs. Pure std + serde; no pipeline wire.

use serde::{Deserialize, Serialize};

/// Repoformer-shaped task classes for per-type T.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RetrievalTaskType {
    Function,
    LineApi,
    Other,
}

/// Named thresholds (not magic numbers). Repoformer defaults as starting T;
/// ELAI will re-calibrate later; Tier-1 is OBSERVE-ONLY so wrong T only journals.
pub const RETRIEVAL_THRESHOLD_FUNCTION: f64 = 0.15;
pub const RETRIEVAL_THRESHOLD_LINE_API: f64 = 0.20;
pub const RETRIEVAL_THRESHOLD_OTHER: f64 = 0.20;

/// Tier-1 is always observe-only (GD-4). Enforce skip only after FP bounded on >=3 real runs.
pub const RETRIEVAL_ABSTENTION_OBSERVE_ONLY: bool = true;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RetrievalGateKind {
    /// Score cleared T — cascade WOULD skip if enforce were on.
    WouldSkip,
    /// Score below T or missing/uncertain — cascade proceeds (conservative).
    WouldProceed,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RetrievalGateVerdict {
    pub kind: RetrievalGateKind,
    pub score: f64,
    pub threshold: f64,
    pub task_type: RetrievalTaskType,
    pub observe_only: bool,
}

/// Threshold for a task type.
// WIRE-DARK[teikoku-opus-walk]: pure module, no pipeline wire yet (J-73 Slice A).
pub fn threshold_for(task: RetrievalTaskType) -> f64 {
    match task {
        RetrievalTaskType::Function => RETRIEVAL_THRESHOLD_FUNCTION,
        RetrievalTaskType::LineApi => RETRIEVAL_THRESHOLD_LINE_API,
        RetrievalTaskType::Other => RETRIEVAL_THRESHOLD_OTHER,
    }
}

/// Cheap model-free local-sufficiency score in [0.0, 1.0].
/// Higher = local context more likely enough (prefer skip retrieval).
/// Heuristic (deterministic):
/// - short symbol/path lookups (1-3 tokens, no question words) -> high (~0.7-0.9)
/// - question / how/why/debug/architecture keywords -> low (~0.1-0.3)
/// - default mid (~0.4)
// WIRE-DARK[teikoku-opus-walk]: pure module, no pipeline wire yet (J-73 Slice A).
pub fn local_sufficiency_score(query: &str) -> f64 {
    let lower = query.to_lowercase();
    let tokens: Vec<&str> = query.split_whitespace().filter(|t| !t.is_empty()).collect();
    let n = tokens.len();

    // Question / reasoning keywords => low sufficiency (need retrieval).
    let question_keywords = [
        "how", "why", "what", "when", "where", "who",
        "debug", "architecture", "design", "explain", "investigate",
        "trace", "root cause", "step by step", "does", "should",
        "can", "could", "would", "is", "are",
    ];
    for kw in question_keywords {
        if lower.contains(kw) {
            return 0.2;
        }
    }

    // Short symbol/path lookups (1-3 tokens, no question words) => high.
    if n <= 3 {
        return 0.8;
    }

    // Default mid.
    0.4
}

/// Classify Function vs LineApi vs Other from query text (deterministic keywords).
// WIRE-DARK[teikoku-opus-walk]: pure module, no pipeline wire yet (J-73 Slice A).
pub fn classify_task_type(query: &str) -> RetrievalTaskType {
    let lower = query.to_lowercase();
    if lower.contains("function") || lower.contains("fn ") || lower.contains("method") {
        return RetrievalTaskType::Function;
    }
    if lower.contains("line") || lower.contains("api") || lower.contains("endpoint") {
        return RetrievalTaskType::LineApi;
    }
    RetrievalTaskType::Other
}

/// Core gate: score >= T => WouldSkip else WouldProceed.
/// Missing/NaN/negative score => WouldProceed (fail-closed retrieve).
/// Always sets observe_only = RETRIEVAL_ABSTENTION_OBSERVE_ONLY.
// WIRE-DARK[teikoku-opus-walk]: pure module, no pipeline wire yet (J-73 Slice A).
pub fn should_retrieve(score: f64, task_type: RetrievalTaskType) -> RetrievalGateVerdict {
    let threshold = threshold_for(task_type);
    let kind = if score.is_nan() || score < 0.0 || score < threshold {
        RetrievalGateKind::WouldProceed
    } else {
        RetrievalGateKind::WouldSkip
    };
    RetrievalGateVerdict {
        kind,
        score,
        threshold,
        task_type,
        observe_only: RETRIEVAL_ABSTENTION_OBSERVE_ONLY,
    }
}

/// Convenience: score + classify from query.
// WIRE-DARK[teikoku-opus-walk]: pure module, no pipeline wire yet (J-73 Slice A).
pub fn should_retrieve_for_query(query: &str) -> RetrievalGateVerdict {
    should_retrieve(local_sufficiency_score(query), classify_task_type(query))
}

/// Returns true only if enforce would skip. With OBSERVE_ONLY=true always false.
// WIRE-DARK[teikoku-opus-walk]: pure module, no pipeline wire yet (J-73 Slice A).
pub fn enforce_skip(verdict: &RetrievalGateVerdict) -> bool {
    !verdict.observe_only && verdict.kind == RetrievalGateKind::WouldSkip
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn threshold_function_below_line_api() {
        assert!(RETRIEVAL_THRESHOLD_FUNCTION < RETRIEVAL_THRESHOLD_LINE_API);
        assert_eq!(RETRIEVAL_THRESHOLD_FUNCTION, 0.15);
        assert_eq!(RETRIEVAL_THRESHOLD_LINE_API, 0.20);
    }

    #[test]
    fn high_score_would_skip() {
        let v = should_retrieve(0.9, RetrievalTaskType::Function);
        assert_eq!(v.kind, RetrievalGateKind::WouldSkip);
        assert!(v.observe_only);
        assert_eq!(v.threshold, 0.15);
    }

    #[test]
    fn low_score_would_proceed() {
        let v = should_retrieve(0.05, RetrievalTaskType::Function);
        assert_eq!(v.kind, RetrievalGateKind::WouldProceed);
    }

    #[test]
    fn nan_score_proceeds() {
        let v = should_retrieve(f64::NAN, RetrievalTaskType::Other);
        assert_eq!(v.kind, RetrievalGateKind::WouldProceed);
    }

    #[test]
    fn enforce_skip_false_while_observe_only() {
        let v = should_retrieve(0.9, RetrievalTaskType::Function);
        assert_eq!(v.kind, RetrievalGateKind::WouldSkip);
        assert!(!enforce_skip(&v));
    }

    #[test]
    fn short_symbol_query_scores_high() {
        let short = local_sufficiency_score("Bm25Index");
        let long = local_sufficiency_score("how does the FSM handle transitions");
        assert!(short > long, "short={} long={}", short, long);
        assert!(short >= 0.7);
        assert!(long <= 0.3);
    }

    #[test]
    fn should_retrieve_for_query_deterministic() {
        let v1 = should_retrieve_for_query("fn main");
        let v2 = should_retrieve_for_query("fn main");
        assert_eq!(v1, v2);
        assert_eq!(v1.task_type, RetrievalTaskType::Function);
    }
}
