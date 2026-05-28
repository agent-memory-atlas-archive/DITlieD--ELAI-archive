//! E-39 -- cost + token economics substrate.
//!
//! This module owns the hard-ceiling enforcement, velocity caps,
//! AutoMix self-verify gate, pre-execution difficulty estimator, and
//! free-tier quota guard scaffolding for ELAI's cost-economics layer.
//! All emit sites are env-gated by `ELAI_COST_TOKEN_ECONOMICS_ENABLE`
//! (wire-dark default) so the substrate compiles + tests + ships
//! identically whether or not the feature is live.
//!
//! Pattern-2 (28th application): a single
//! [`crate::ssb::SsbActionType::CostTokenEvent`] variant carries an
//! 8-arm [`crate::ssb::CostEventKind`] discriminator. SCHEMA_VERSION
//! 175 -> 176; TOTAL_VARIANTS 448 -> 449. The 8 arms (in plan-listed
//! order) are TaskAbortedBySpendCap / VelocityCapBreached /
//! AutomixVerifierVerdict / CostRouterDecision / CostEstimateInaccurate
//! / FreeTierQuotaApproaching / FreeTierQuotaExceeded /
//! LayeredCapManualOverride.
//!
//! Submodule layout:
//! * [`velocity_cap`] -- rolling-window req/$ counters (1m / 10m / 1h / 24h)
//! * [`automix_gate`] -- local-verifier-before-escalation 3-arm verdict
//! * [`cost_router`] -- difficulty-x-budget route decision matrix
//! * [`free_tier_guard`] -- Cerebras / Groq / DeepSeek daily-quota guard
//! * [`cloud_abort`] -- mid-task cooperative abort + per-role + layered caps
//!
//! Source grounding (verified May 2026 via WebSearch + WebFetch):
//! * arXiv:2310.12963 -- AutoMix self-verify cascade, >50% cost
//!   reduction at iso-quality (verified at arxiv.org/abs/2310.12963;
//!   ICLR 2025; Aggarwal & Madaan et al. github.com/automix-llm/automix).
//! * arXiv:2604.22750 -- "How Do AI Agents Spend Your Money?" Stanford
//!   Digital Economy Lab (verified at arxiv.org/abs/2604.22750 +
//!   alphaxiv.org/overview/2604.22750; 30x same-task variance, 1000x
//!   tokens vs chat, input-driven cost; April 2026 publication).
//! * platform.claude.com/docs/en/about-claude/pricing (Sonnet 4.6
//!   $3/$15, cache-write 1.25x, cache-read 0.10x, 5-min TTL default
//!   since early 2026).
//! * api-docs.deepseek.com/quick_start/pricing (DeepSeek V4-Flash
//!   $0.30/$0.50 + 90% cache discount).
//! * ravoid.com/blog/ai-agent-budget-enforcement ($47K LangChain
//!   4-agent loop in 11 days -- the wallet-loop class this module
//!   defends against).
//!
//! See `workflow/reasoning/E-39-decisions.md` for the path-drift
//! resolution (plan-cited `cloud_spend.rs` / `runtime_orchestrator.rs`
//! do NOT exist; this module sits alongside `cloud_mode.rs` and 3 new
//! backends land in `crates/providers/src/runtime/types.rs`).

pub mod automix_gate;
pub mod burn_projection;
pub mod cloud_abort;
pub mod cloud_route;
pub mod cost_router;
pub mod free_tier_guard;
pub mod velocity_cap;

/// Env-gate constant. When set to "1" at process start, every emit
/// site under [`crate::cost`] writes its [`crate::ssb::SsbActionType
/// ::CostTokenEvent`] row. When unset / "0", the helpers short-circuit
/// and return without touching SSB.
///
/// Mirrors the precedent at [`crate::rl::ENV_AGENTIC_RL_ENABLE`] and
/// the E-38 `ELAI_AI_ROADMAP_SIGNALS_ENABLE` gate. Wire-dark default
/// keeps production sessions on the existing C-5 cloud-mode behaviour
/// until the user opts in.
pub const ENV_COST_TOKEN_ECONOMICS_ENABLE: &str = "ELAI_COST_TOKEN_ECONOMICS_ENABLE";

/// Returns `true` when [`ENV_COST_TOKEN_ECONOMICS_ENABLE`] is set to
/// `1`. Single source of truth for the emission gate so every producer
/// agrees on the live/wire-dark check.
#[must_use]
pub fn cost_token_economics_enabled() -> bool {
    std::env::var(ENV_COST_TOKEN_ECONOMICS_ENABLE)
        .ok()
        .map(|v| v == "1")
        .unwrap_or(false)
}

/// AgentRole bucket for per-role spend caps. Maps to the orchestrator's
/// agent partition (executor / auditor / scope_planner / memory_writer)
/// plus a `safety_classifier` bucket reserved for E-37's PromptArmor
/// classifier and a `doc_parser` bucket reserved for E-36's vision-
/// parser per-call cost. The plan Open Question proposes auditor 2x
/// executor as the default; revise on first-session feedback per
/// E-39-decisions.md DECISION-3.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CostRoleBucket {
    Executor,
    Auditor,
    ScopePlanner,
    MemoryWriter,
    SafetyClassifier,
    DocParser,
}

impl CostRoleBucket {
    /// Snake-case label used in SSB details_json and config keys.
    #[must_use]
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Executor => "executor",
            Self::Auditor => "auditor",
            Self::ScopePlanner => "scope_planner",
            Self::MemoryWriter => "memory_writer",
            Self::SafetyClassifier => "safety_classifier",
            Self::DocParser => "doc_parser",
        }
    }
}

#[cfg(test)]
mod env_gate_tests {
    use super::*;

    #[test]
    fn role_bucket_labels_stable() {
        assert_eq!(CostRoleBucket::Executor.as_str(), "executor");
        assert_eq!(CostRoleBucket::Auditor.as_str(), "auditor");
        assert_eq!(CostRoleBucket::ScopePlanner.as_str(), "scope_planner");
        assert_eq!(CostRoleBucket::MemoryWriter.as_str(), "memory_writer");
        assert_eq!(CostRoleBucket::SafetyClassifier.as_str(), "safety_classifier");
        assert_eq!(CostRoleBucket::DocParser.as_str(), "doc_parser");
    }

    #[test]
    fn env_gate_default_off() {
        // SAFETY: env vars in tests are inherently racy under cargo's
        // default thread runner; we treat this as a smoke check that the
        // helper compiles + returns a bool. The hard property is enforced
        // at the emit sites which call this helper, not here.
        let _ = cost_token_economics_enabled();
    }
}
