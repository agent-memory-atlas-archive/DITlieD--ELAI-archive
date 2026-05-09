//! GRC (Grounded Reasoning Chain) integration layer for the orchestrator pipeline.
//!
//! This module wraps `elai_reasoning::BasicGrcEngine` and exposes GRC operations
//! at the orchestrator level. It provides integration points for:
//!
//! - `think()` — agent reasoning steps with budget enforcement
//! - `verify_claim()` / `ground_plan()` — plan evaluation and claim grounding
//! - `generate_csq()` — counterfactual safety questions before sensitive tool calls
//! - `check_inline_feasibility()` — feasibility checks during EXECUTING phase
//!
//! The `PipelineGrc` struct is the single entry point. It is constructed from an
//! `AgentGrcConfig` (obtained from `default_grc_config` or custom config) and holds
//! one `BasicGrcEngine`. The orchestrator pipeline creates one `PipelineGrc` per
//! session or per phase transition as appropriate.

use elai_reasoning::{
    AgentGrcConfig, BasicGrcEngine, CsqResult, GrcEngine, GrcError, InlineFeasibilityResult,
    PlanGroundingResult, ThinkResult, ThinkStep, VerificationResult,
};

// ---------------------------------------------------------------------------
// Kill-switch: ELAI_DISABLE_PIPELINE_GRC
// ---------------------------------------------------------------------------

/// Env-var kill switch (B-8). When set to "1" or "true" (case-insensitive),
/// the orchestrator construction site skips wiring `PipelineGrc` and every
/// gated call site (ground_plan / generate_csq / check_inline_feasibility /
/// auditor anti-rubber-stamp seed) short-circuits to a no-op + one-shot
/// tracing::warn. Precedent: 11 sibling kill-switches such as
/// `ELAI_DISABLE_PROVIDER_SIGNALS` and `ELAI_DISABLE_IRREVERSIBLE_CLASSIFIER`.
pub const ELAI_DISABLE_PIPELINE_GRC_ENV: &str = "ELAI_DISABLE_PIPELINE_GRC";

/// Returns true when the kill switch env-var is set to "1" or "true"
/// (case-insensitive). All other values (including absent / empty / "0") are
/// treated as the gate being enabled.
pub fn pipeline_grc_disabled_via_env() -> bool {
    match std::env::var(ELAI_DISABLE_PIPELINE_GRC_ENV) {
        Ok(v) => {
            let trimmed = v.trim();
            trimmed == "1" || trimmed.eq_ignore_ascii_case("true")
        }
        Err(_) => false,
    }
}

// ---------------------------------------------------------------------------
// PipelineGrcError
// ---------------------------------------------------------------------------

/// Errors returned by `PipelineGrc` operations.
#[derive(Debug, thiserror::Error)]
pub enum PipelineGrcError {
    #[error("reasoning engine error: {0}")]
    Engine(#[from] GrcError),
}

// ---------------------------------------------------------------------------
// PipelineGrc
// ---------------------------------------------------------------------------

/// Orchestrator-level GRC integration point.
///
/// Wraps `BasicGrcEngine` and exposes the four integration categories required
/// by Phase 2 wiring:
///
/// 1. `think()` — driven by agent reasoning loops
/// 2. `verify_claim()` / `ground_plan()` — driven by plan evaluation code
/// 3. `generate_csq()` — driven by pre-tool-call hooks for sensitive operations
/// 4. `check_inline_feasibility()` — driven by EXECUTING phase transition handlers
pub struct PipelineGrc {
    engine: BasicGrcEngine,
}

impl PipelineGrc {
    /// Create a new `PipelineGrc` with the given agent GRC configuration applied.
    ///
    /// `with_agent_config` is called immediately so the engine's topic budget
    /// and required steps reflect the role from the start.
    pub fn new(config: AgentGrcConfig) -> Self {
        let mut engine = BasicGrcEngine::new();
        engine.with_agent_config(config);
        Self { engine }
    }

    /// Create a `PipelineGrc` using an already-configured `BasicGrcEngine`.
    ///
    /// Intended for cases where the caller holds an `AgentHandle` and wants
    /// to delegate GRC calls through this wrapper:
    ///
    /// ```ignore
    /// let grc = PipelineGrc::from_engine(handle.grc_engine);
    /// ```
    pub fn from_engine(engine: BasicGrcEngine) -> Self {
        Self { engine }
    }

    /// Create a `PipelineGrc` using `AgentGrcConfig::default()`.
    ///
    /// Production construction sites (dashboard task runner, CLI `start_task_runner`
    /// call sites) use this to inject a live GRC instance into `TaskRunnerConfig`
    /// when no per-role config is available yet. The default config enables all
    /// verification methods, requires the standard 5-step chain, and does not
    /// enforce scope enumeration or CSQ.
    ///
    /// Call sites that have a real per-role config should use `PipelineGrc::new`
    /// with the loaded `AgentGrcConfig` instead.
    pub fn new_from_default() -> Self {
        Self::new(AgentGrcConfig::default())
    }

    // -----------------------------------------------------------------------
    // Integration point 1: agent reasoning steps
    // -----------------------------------------------------------------------

    /// Record a GRC reasoning step for the given topic.
    ///
    /// Called by agent reasoning loops. Returns the step result including
    /// remaining budget and any grounding warnings.
    pub fn think(&mut self, step: ThinkStep) -> Result<ThinkResult, PipelineGrcError> {
        self.engine.think(step).map_err(PipelineGrcError::Engine)
    }

    // -----------------------------------------------------------------------
    // Integration point 2: plan evaluation / claim grounding
    // -----------------------------------------------------------------------

    /// Verify a single claim against a source using the engine's verifier.
    ///
    /// Called from plan evaluation code. Returns `VerificationResult` with
    /// `passes: true` when the claim is grounded.
    pub fn verify_claim(
        &self,
        claim: &str,
        source: &str,
        context: &str,
    ) -> Result<VerificationResult, PipelineGrcError> {
        self.engine
            .verify_claim(claim, source, context)
            .map_err(PipelineGrcError::Engine)
    }

    /// Scan a plan string for `Grounding:` citations and verify each one.
    ///
    /// Returns a `PlanGroundingResult` listing which claims passed, which are
    /// ungrounded, and which need manual verification. A plan with
    /// `all_grounded: true` is safe to proceed with.
    pub fn ground_plan(&self, plan: &str) -> PlanGroundingResult {
        self.engine.ground_plan(plan)
    }

    // -----------------------------------------------------------------------
    // Integration point 3: pre-tool-call CSQ
    // -----------------------------------------------------------------------

    /// Generate Counterfactual Safety Questions for a function before it executes.
    ///
    /// Called from pre-tool-call hooks for sensitive operations (file writes,
    /// network calls, auth operations, etc.). `sensitive_ops` is the list of
    /// sensitive API calls found by the caller's static analysis. An empty
    /// `sensitive_ops` list returns an empty `CsqResult` with no blocking.
    pub fn generate_csq(
        &self,
        function_id: &str,
        sensitive_ops: &[String],
    ) -> Result<CsqResult, PipelineGrcError> {
        self.engine
            .generate_csq(function_id, sensitive_ops)
            .map_err(PipelineGrcError::Engine)
    }

    // -----------------------------------------------------------------------
    // Integration point 4: EXECUTING phase inline feasibility
    // -----------------------------------------------------------------------

    /// Check inline feasibility of a claim during the EXECUTING phase.
    ///
    /// Called from phase transition handlers. Limited to 2 checks per engine
    /// instance (per EXECUTING phase). Returns `BudgetExhausted` after the
    /// limit is reached.
    pub fn check_inline_feasibility(
        &mut self,
        claim: &str,
        sources: &[String],
    ) -> Result<InlineFeasibilityResult, PipelineGrcError> {
        self.engine
            .check_inline_feasibility(claim, sources)
            .map_err(PipelineGrcError::Engine)
    }

    // -----------------------------------------------------------------------
    // Accessors
    // -----------------------------------------------------------------------

    /// Access the underlying engine for advanced use (e.g., set_budget, reset_topic).
    pub fn engine(&self) -> &BasicGrcEngine {
        &self.engine
    }

    /// Mutably access the underlying engine (e.g., set_budget, reset_topic).
    pub fn engine_mut(&mut self) -> &mut BasicGrcEngine {
        &mut self.engine
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use elai_reasoning::{AgentGrcConfig, GrcBudget, GrcState, GroundingLevel, ThinkStep};

    fn executor_config() -> AgentGrcConfig {
        AgentGrcConfig {
            role: "executor".into(),
            budget: GrcBudget {
                max_calls: 8,
                max_chars: 1200,
                calls_used: 0,
                chars_used: 0,
            },
            required_steps: vec![
                GrcState::Observe,
                GrcState::Hypothesize,
                GrcState::PlanAction,
                GrcState::Act,
                GrcState::Verify,
            ],
            grounding_level: GroundingLevel::High,
            csq_required: true,
            require_scope_enumeration: false,
            ..AgentGrcConfig::default()
        }
    }

    #[test]
    fn pipeline_grc_new_applies_config() {
        let grc = PipelineGrc::new(executor_config());
        let config = grc.engine().get_agent_config();
        assert!(config.is_some());
        assert_eq!(config.unwrap().role, "executor");
    }

    #[test]
    fn think_observe_succeeds() {
        let mut grc = PipelineGrc::new(executor_config());
        let step = ThinkStep {
            step_type: GrcState::Observe,
            content: "reading engine.rs to understand BasicGrcEngine".into(),
            topic: "test-topic".into(),
            grounding: Some("file:crates/reasoning/src/engine.rs".into()),
            conclusion: None,
            scope: None,
        };
        let result = grc.think(step);
        assert!(result.is_ok(), "think(Observe) should succeed: {result:?}");
    }

    #[test]
    fn generate_csq_empty_ops_returns_no_checks() {
        let grc = PipelineGrc::new(executor_config());
        let result = grc.generate_csq("my_function", &[]).unwrap();
        assert!(result.checks.is_empty());
        assert_eq!(result.function_id, "my_function");
    }

    #[test]
    fn generate_csq_file_write_critical() {
        let grc = PipelineGrc::new(executor_config());
        let ops = vec!["std::fs::write".to_string()];
        let result = grc.generate_csq("write_config", &ops).unwrap();
        assert_eq!(result.checks.len(), 1);
        use elai_reasoning::CsqSeverity;
        assert_eq!(result.checks[0].severity, CsqSeverity::Critical);
    }

    #[test]
    fn check_inline_feasibility_pass_when_claim_in_sources() {
        let mut grc = PipelineGrc::new(executor_config());
        let sources = vec!["The engine is ready for use".to_string()];
        let result = grc
            .check_inline_feasibility("The engine is ready for use", &sources)
            .unwrap();
        use elai_reasoning::FeasibilityOutcome;
        assert_eq!(result.outcome, FeasibilityOutcome::Pass);
    }

    #[test]
    fn check_inline_feasibility_exhausted_after_two_calls() {
        let mut grc = PipelineGrc::new(executor_config());
        let sources = vec!["source text".to_string()];
        grc.check_inline_feasibility("source text", &sources).unwrap();
        grc.check_inline_feasibility("source text", &sources).unwrap();
        let third = grc.check_inline_feasibility("source text", &sources);
        assert!(
            third.is_err(),
            "third inline feasibility check should be budget-exhausted"
        );
    }

    #[test]
    fn ground_plan_empty_is_all_grounded() {
        let grc = PipelineGrc::new(executor_config());
        let result = grc.ground_plan("");
        assert!(result.all_grounded);
        assert!(result.claims.is_empty());
    }

    #[test]
    fn verify_claim_file_prefix_dispatches() {
        let grc = PipelineGrc::new(executor_config());
        // File check on a non-existent path: should not panic, may fail gracefully
        let result = grc.verify_claim("file:/nonexistent/path.rs", "", "");
        // Either Ok (file check ran, passes=false) or Err is acceptable — not a panic.
        match result {
            Ok(vr) => assert!(!vr.passes, "nonexistent file should not pass"),
            Err(_) => {}
        }
    }
}
