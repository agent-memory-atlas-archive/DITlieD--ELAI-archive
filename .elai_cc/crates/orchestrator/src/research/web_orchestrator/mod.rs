//! E-48 — web-research orchestrator hardening substrate.
//!
//! Wire-dark by default: every producer in [`producers`] checks
//! [`is_enabled`] and silently returns `Ok(None)` when the env gate is
//! not set, so existing B-7 web_research_orchestrator behaviour is
//! unchanged for non-E-48 callers. Read-side consumers (panel queries,
//! contract tests) remain operational.
//!
//! Subtree layout (all additive — no existing public APIs are modified):
//!
//!   * [`editable_plan`] -- Task 1: deterministic FORMULATE-stage
//!     [`editable_plan::WebResearchPlan`] artifact with sub-question
//!     ranking via a token-overlap novelty score + named-entity
//!     specificity score. Sibling to the existing
//!     [`crate::research_tree::ResearchPlan`] tree-input struct (different
//!     concerns — `ResearchPlan` is the tree's root-question + budget;
//!     `WebResearchPlan` is the editable-CoT artifact the user can
//!     approve / edit before SEARCH fires).
//!   * [`tier_pool_telemetry`] -- Task 2: thin adapter on
//!     [`elai_web_search::tiered_search_pool`] that re-exports the
//!     `BackendTier` rename + writes the
//!     [`crate::ssb::WebResearchEventKind::TieredSearchDispatched`] row
//!     when wired into `WebResearchOrchestrator::search_and_rank`.
//!   * [`speculative_pruner`] -- Task 5: FlashResearch
//!     `BranchType` discriminator + low-evidence-delta + low-gain-streak
//!     pruner predicate (arXiv:2510.05145). Sibling to the existing
//!     [`crate::research_tree::ResearchTree::prune_speculative_children`]
//!     — the predicate fn here returns the prune verdict; the actual
//!     drop is delegated to the existing tree method to preserve B-7
//!     pruner semantics.
//!   * [`conformal_stop`] -- Task 6: triple-threshold convergence
//!     evaluator returning
//!     [`crate::research_tree::ConvergenceStatus::PreemptiveUnsolvable`]
//!     when the lower bound fires. Calibration is conservative until
//!     ELAI accumulates a tagged-unsolvable corpus (open question #5).
//!   * [`contradiction_classifier`] -- Task 7: never-silently-resolved
//!     contradiction surfacing. The classifier is sidecar-backed (LLM
//!     pair-judge); host-side scaffolding ships here, live LLM call is
//!     RUNPOD-PENDING gated by sidecar availability. When wired, sets
//!     `prune_protected: true` on the materialized
//!     [`crate::research_tree::Contradiction`].
//!   * [`citation_verifier`] -- Tasks 8 + 9: 4-stage citation
//!     verification (URL-liveness, arXiv-ID, CrossRef-DOI, SemanticCite
//!     faithfulness) returning [`citation_verifier::VerificationOutcome`]
//!     + the [`citation_verifier::VerifiedCitation`] newtype
//!     (compile-time enforcement of "no final-report citation without
//!     verification"). Stage (a) and (b) and (c) are deterministic HTTP
//!     HEAD; stage (d) is sidecar-backed and RUNPOD-PENDING by default.
//!     Sub-agent attribution lock lives here too — the
//!     [`citation_verifier::AttributionSet`] type enforces the
//!     Anthropic lead-never-cites rule.
//!   * [`recency_decay`] -- Task 10: temporal-marker regex + exponential
//!     decay multiplier on the existing
//!     [`elai_web_search::source_quality::SourceQualityTable::score`]
//!     surface. Default half-life 365 days, configurable via
//!     `[web_search.recency]` in `elai_cc.toml`.
//!   * [`producers`] -- producer-side `emit_*` free-fns mirroring the
//!     [`crate::planning_decomposition::producers`] shape. Each emit
//!     fn checks [`is_enabled`] and writes one
//!     [`crate::ssb::WebResearchEventPayload`] row per fire-point.
//!
//! Paper grounding (all arXiv IDs verified May 2026 via WebSearch):
//!
//!   * arXiv:2510.05145 -- FlashResearch / ParallelResearch (Task 5)
//!   * arXiv:2602.06718 -- GhostCite citation hallucination baseline (Task 8)
//!   * arXiv:2602.05930 -- Compound Deception NeurIPS 2025 taxonomy
//!   * arXiv:2604.03173 -- CITEVERIFIER / SemanticCite / CiteGuard (Task 8)
//!   * arXiv:2601.18724 -- HalluCitation Matters (Task 8 stage-b)
//!   * arXiv:2602.03814 -- Conformal Thinking triple-threshold (Task 6)
//!   * arXiv:2504.00180 -- Contradiction Detection in RAG (Task 7)
//!   * arXiv:2511.06668 -- Healthcare RAG never-silently-resolve (Task 7)
//!   * arXiv:2510.03418 -- LegalWiz multi-agent consistency (Task 7)
//!   * arXiv:2509.19376 -- Solving Freshness in RAG (Task 10)
//!   * arXiv:2509.23519 -- ReliabilityRAG tier rationale (Task 2)
//!   * simonwillison.net/2025/Jun/14 -- Anthropic multi-agent (Task 9)
//!   * docs.cohere.com/docs/reranking-best-practices (Task 4 — reranker
//!     lives in `elai_web_search::cross_encoder_rerank`)
//!
//! Backward compatibility:
//!   * No existing struct field renamed or removed. The
//!     [`crate::research_tree::Contradiction`] struct gains 3 new
//!     `#[serde(default)]` fields (`prune_protected`,
//!     `surfaced_at_unix_ms`, `classification_reason`) so legacy state
//!     deserializes without migration.
//!   * [`crate::research_tree::ConvergenceStatus`] gains a
//!     `PreemptiveUnsolvable` variant. All known consumers use
//!     wildcard patterns (`_ => ...`) or `matches!(_, Failed)` so the
//!     addition is non-breaking; the permissive deserializer extends
//!     to parse the new variant from both unit-string and struct form.
//!   * New SSB action_type is added as a single `WebResearchEvent`
//!     variant carrying a Kind-discriminator payload (Pattern-2, 37th
//!     application). SCHEMA_VERSION bumps 184 -> 185.
//!
//! See `workflow/reasoning/E-48-decisions.md` for the full decision log.

pub mod citation_verifier;
pub mod conformal_stop;
pub mod contradiction_classifier;
pub mod editable_plan;
pub mod producers;
pub mod recency_decay;
pub mod speculative_pruner;
pub mod tier_pool_telemetry;

/// Env-var name gating SSB emission for the web-research orchestrator
/// hardening substrate.
///
/// Wire-dark default: producers in [`producers`] check this gate and
/// silently return without writing an SSB row unless the env var is set
/// to a truthy value (`"1"` / `"true"` / `"yes"` / `"on"`). Mirrors the
/// precedent set by
/// [`crate::ssb::ELAI_PLANNING_DECOMPOSITION_ENABLE_ENV`] +
/// [`crate::ssb::ELAI_TREE_SITTER_GRAMMARS_ENABLE_ENV`].
///
/// CONTRACT: The literal string MUST match
/// [`crate::ssb::ELAI_WEB_RESEARCH_ORCHESTRATOR_ENABLE_ENV`] verbatim.
/// A drift between the two would silently disable the producers without
/// any SSB row to point at — strict-equality assertion lives in the
/// `web_research_env_gate_matches_ssb` contract test.
pub const WEB_RESEARCH_ORCHESTRATOR_ENABLE_ENV: &str =
    "ELAI_WEB_RESEARCH_ORCHESTRATOR_ENABLE";

/// Returns `true` when [`WEB_RESEARCH_ORCHESTRATOR_ENABLE_ENV`] is set to
/// a recognised truthy literal.
///
/// Recognised truthy values: `"1"`, `"true"`, `"yes"`, `"on"` (case
/// insensitive). Anything else (including unset, empty, `"0"`,
/// `"false"`, garbled) returns `false`.
#[must_use]
pub fn is_enabled() -> bool {
    match std::env::var(WEB_RESEARCH_ORCHESTRATOR_ENABLE_ENV) {
        Ok(v) => matches!(
            v.trim().to_ascii_lowercase().as_str(),
            "1" | "true" | "yes" | "on"
        ),
        Err(_) => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Mutex;

    /// Tests in this module manipulate the process-global env var so they
    /// must run serially with one another to avoid cross-test interference.
    /// A poisoned mutex still lets subsequent tests proceed (we recover the
    /// guard) — we just need mutual exclusion.
    static ENV_LOCK: Mutex<()> = Mutex::new(());

    #[test]
    fn web_research_env_gate_matches_ssb() {
        // Strict-equality with the ssb-side const. Drift would silently
        // disable producers without any SSB row to point at.
        assert_eq!(
            WEB_RESEARCH_ORCHESTRATOR_ENABLE_ENV,
            crate::ssb::ELAI_WEB_RESEARCH_ORCHESTRATOR_ENABLE_ENV,
            "web_orchestrator env gate const must match the ssb-side const"
        );
    }

    #[test]
    fn is_enabled_default_off() {
        let _g = ENV_LOCK.lock().unwrap_or_else(|p| p.into_inner());
        std::env::remove_var(WEB_RESEARCH_ORCHESTRATOR_ENABLE_ENV);
        assert!(!is_enabled());
    }

    #[test]
    fn is_enabled_truthy_set() {
        let _g = ENV_LOCK.lock().unwrap_or_else(|p| p.into_inner());
        for v in ["1", "TRUE", "Yes", "on"] {
            std::env::set_var(WEB_RESEARCH_ORCHESTRATOR_ENABLE_ENV, v);
            assert!(is_enabled(), "{v} should enable");
        }
        std::env::remove_var(WEB_RESEARCH_ORCHESTRATOR_ENABLE_ENV);
    }

    #[test]
    fn is_enabled_falsy_set() {
        let _g = ENV_LOCK.lock().unwrap_or_else(|p| p.into_inner());
        for v in ["0", "false", "", "garbage"] {
            std::env::set_var(WEB_RESEARCH_ORCHESTRATOR_ENABLE_ENV, v);
            assert!(!is_enabled(), "{v} should NOT enable");
        }
        std::env::remove_var(WEB_RESEARCH_ORCHESTRATOR_ENABLE_ENV);
    }
}
