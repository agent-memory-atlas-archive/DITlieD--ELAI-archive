//! Batch 3B — Phase 4G: AutonomousGate.
//!
//! Wraps the [`InteractiveGate`] protocol with an autonomous-mode layer that
//! makes gate decisions from evidence snapshots without waiting for a human.
//! Per spec:
//! * Six evidence checks (KnowledgeReadiness, ZeroUngrounded,
//!   TestsFailCorrectly, TestFlipped, ZeroBlocks, DkgAndChangelogPresent)
//!   gate auto-advance. Any failure flips the decision to
//!   [`AutonomyDecision::RequestHuman`].
//! * Four escalation triggers (AntiSpiral, HighRisk, ReplanMajor,
//!   CircuitBreaker) halt the task and hand control back to a human.
//! * [`AutonomyDecision::SplitTask`] fires when the context budget reaches
//!   Terminal — max 4 sub-tasks, max depth 2.
//! * [`AutonomyDecision::SubstituteModel`] cycles through the configured
//!   fallback chain on max_restarts exhaustion.
//!
//! The `GateDecision` enum from `gate.rs` is INTERACTIVE (Approve/Reject/
//! Modify/Redirect). This module introduces [`AutonomyDecision`] as a
//! disjoint, orthogonal output type so the two gates can co-exist.
//!
//! Source: docs/specs/guide/steps/phase-4G.txt §§4G.1-4G.8;
//! docs/specs/guide/steps/phase-index-3A-4J.txt §Phase 4G.

use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::Arc;

use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;

use crate::gate::{InteractiveGate, RiskLevel};

// ---------------------------------------------------------------------------
// Env kill switch
// ---------------------------------------------------------------------------

/// When this env var is set to any non-empty string, [`AutonomousGate::decide`]
/// short-circuits to [`AutonomyDecision::RequestHuman`] regardless of mode or
/// evidence. Matches the "emergency shutoff" pattern established by
/// `ELAI_DISABLE_TOOL_DISPATCH` et al.
pub const ELAI_DISABLE_AUTONOMOUS_GATE_ENV: &str = "ELAI_DISABLE_AUTONOMOUS_GATE";

/// Returns true iff the kill switch is set. Cheap: single env read.
pub fn autonomous_gate_disabled_via_env() -> bool {
    std::env::var(ELAI_DISABLE_AUTONOMOUS_GATE_ENV)
        .map(|v| !v.is_empty())
        .unwrap_or(false)
}

// ---------------------------------------------------------------------------
// GateMode — runtime switch between InteractiveGate-only vs AutonomousGate
// ---------------------------------------------------------------------------

/// Runtime gate operating mode. Toggled per session via the `/mode` slash
/// command (see `dashboard/src/commands.rs::set_gate_mode`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum GateMode {
    /// Default. Every MEDIUM/HIGH transition prompts a human. Matches the
    /// pre-4G behaviour of `run_interactive_gate` in `gate.rs`.
    Interactive,
    /// Autonomous mode. Evidence checks + escalation triggers drive
    /// decisions without human input unless an escalation fires.
    Autonomous,
}

impl std::fmt::Display for GateMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            GateMode::Interactive => f.write_str("interactive"),
            GateMode::Autonomous => f.write_str("autonomous"),
        }
    }
}

impl GateMode {
    /// Parse a user-supplied mode string. Lower-cased and trimmed first so
    /// `/mode AUTONOMOUS ` and `/mode autonomous` are equivalent.
    pub fn from_str(raw: &str) -> Option<Self> {
        match raw.trim().to_ascii_lowercase().as_str() {
            "interactive" => Some(GateMode::Interactive),
            "autonomous" => Some(GateMode::Autonomous),
            _ => None,
        }
    }
}

// ---------------------------------------------------------------------------
// Pipeline level — tree-wide auto-advance policy input
// ---------------------------------------------------------------------------

/// Node level in the concept → phase → step → baby-step tree. Drives the
/// [`TreeAutoAdvancePolicy`] risk classification: concepts never auto-advance,
/// baby steps nearly always do. Source: phase-4G.txt §4G.7-4G.8.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PipelineLevel {
    Concept,
    Phase,
    Step,
    BabyStep,
}

impl PipelineLevel {
    /// Per-level default risk. Concept = HIGH (architectural — always
    /// request human). Phase = MEDIUM (request unless the pipeline marks it
    /// LOW). Step = LOW (proceed). BabyStep = LOW (proceed).
    pub fn default_risk(self) -> RiskLevel {
        match self {
            PipelineLevel::Concept => RiskLevel::High,
            PipelineLevel::Phase => RiskLevel::Medium,
            PipelineLevel::Step => RiskLevel::Low,
            PipelineLevel::BabyStep => RiskLevel::Low,
        }
    }
}

// ---------------------------------------------------------------------------
// TreeAutoAdvancePolicy — per-level auto-advance rules
// ---------------------------------------------------------------------------

/// Per-level auto-advance policy. New in Batch 3B. Applied inside
/// [`AutonomousGate::decide`] when evidence passes and no escalation fires —
/// determines whether the gate auto-advances or still requests a human.
///
/// Source: phase-4G.txt §4G.7 (tree-level auto-advance).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TreeAutoAdvancePolicy {
    /// Max auto-advances per concept before forcing a human checkpoint.
    /// Defaults to 50 per spec §4G.7 circuit breaker.
    pub max_auto_baby_steps_per_concept: u32,
    /// When true, a BabyStep-level gate auto-advances unless a human gate
    /// explicitly fired. Defaults to true (spec §4G.8).
    pub baby_step_auto_advance: bool,
    /// When true, a Step-level gate auto-advances when risk is LOW. Defaults
    /// to true (spec §4G.8).
    pub step_auto_advance: bool,
    /// When true, a Phase-level gate auto-advances only if NOT marked
    /// MEDIUM-risk. Defaults to true (spec §4G.8).
    pub phase_auto_advance_if_not_medium: bool,
    /// When true, a Concept-level gate always requests human (HIGH risk).
    /// Defaults to true (spec §4G.8).
    pub concept_always_human: bool,
}

impl Default for TreeAutoAdvancePolicy {
    fn default() -> Self {
        Self {
            max_auto_baby_steps_per_concept: 50,
            baby_step_auto_advance: true,
            step_auto_advance: true,
            phase_auto_advance_if_not_medium: true,
            concept_always_human: true,
        }
    }
}

impl TreeAutoAdvancePolicy {
    /// Returns `true` when the policy permits an auto-advance for the given
    /// level AND the provided effective risk. Escalation / evidence checks
    /// are applied separately by the caller.
    pub fn should_auto_advance(
        &self,
        level: PipelineLevel,
        effective_risk: RiskLevel,
    ) -> bool {
        match level {
            PipelineLevel::Concept => !self.concept_always_human,
            PipelineLevel::Phase => {
                self.phase_auto_advance_if_not_medium && effective_risk != RiskLevel::Medium
            }
            PipelineLevel::Step => self.step_auto_advance && effective_risk == RiskLevel::Low,
            PipelineLevel::BabyStep => self.baby_step_auto_advance,
        }
    }
}

// ---------------------------------------------------------------------------
// Evidence — snapshot of the six 4G evidence-check inputs
// ---------------------------------------------------------------------------

/// Snapshot of the six evidence checks required before any autonomous
/// auto-advance. Sourced from SSB rows at the call site; passed in as a pure
/// data structure so the gate stays unit-testable without spinning up SQLite.
/// Source: phase-4G.txt §4G.3.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Evidence {
    /// Check 1 — `project_profile.toml` exists, DKG has >0 nodes, BM25
    /// index populated (DkgReady SSB row present).
    pub knowledge_readiness: bool,
    /// Check 2 — zero ungrounded `Finding` rows at the current phase.
    pub zero_ungrounded: bool,
    /// Check 3 — TESTING_FIRST → EXECUTING: at least one test is red
    /// before the executing gate runs.
    pub tests_fail_correctly: bool,
    /// Check 4 — EXECUTING → AUDITING: every test that was red at
    /// TESTING_FIRST has a matching green row at EXECUTING.
    pub test_flipped: bool,
    /// Check 5 — AUDITING → MEMORIZING: zero `severity == "error"` Finding
    /// rows for the current phase.
    pub zero_blocks: bool,
    /// Check 6 — MEMORIZING → next: DkgReady row for current project +
    /// changelog entry present for the current phase.
    pub dkg_and_changelog_present: bool,
}

impl Evidence {
    /// Every check passed. Required for any [`AutonomyDecision::AutoAdvance`].
    pub fn all_passed(&self) -> bool {
        self.knowledge_readiness
            && self.zero_ungrounded
            && self.tests_fail_correctly
            && self.test_flipped
            && self.zero_blocks
            && self.dkg_and_changelog_present
    }

    /// Human-readable summary for the `GateDecision` SSB payload. Format is
    /// the six single-letter pass/fail flags in check order.
    pub fn summary(&self) -> String {
        format!(
            "kr={} zu={} tfc={} tf={} zb={} dcg={}",
            self.knowledge_readiness as u8,
            self.zero_ungrounded as u8,
            self.tests_fail_correctly as u8,
            self.test_flipped as u8,
            self.zero_blocks as u8,
            self.dkg_and_changelog_present as u8,
        )
    }

    /// List of the evidence check names that failed. Used by the
    /// RequestHuman reason string.
    pub fn failed_checks(&self) -> Vec<&'static str> {
        let mut v = Vec::new();
        if !self.knowledge_readiness {
            v.push("knowledge_readiness");
        }
        if !self.zero_ungrounded {
            v.push("zero_ungrounded");
        }
        if !self.tests_fail_correctly {
            v.push("tests_fail_correctly");
        }
        if !self.test_flipped {
            v.push("test_flipped");
        }
        if !self.zero_blocks {
            v.push("zero_blocks");
        }
        if !self.dkg_and_changelog_present {
            v.push("dkg_and_changelog_present");
        }
        v
    }
}

// ---------------------------------------------------------------------------
// EscalationTrigger — reasons the gate halts for a human
// ---------------------------------------------------------------------------

/// Discriminant for [`AutonomyDecision::Escalate`]. Source: phase-4G.txt §4G.4.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum EscalationTrigger {
    /// [`crate::anti_spiral::AntiSpiralDetector`] reported `spiral_detected`.
    AntiSpiral,
    /// Transition metadata marked the current gate as `RiskLevel::High`.
    HighRisk,
    /// A `ReplanRequest` SSB row with `severity == "MAJOR"` is active.
    ReplanMajor,
    /// The autonomous gate has already auto-advanced
    /// `circuit_breaker_limit` times this session.
    CircuitBreaker,
}

impl EscalationTrigger {
    /// Snake-case label used in `EscalationFired.trigger` SSB rows and the
    /// `GateDecision` evidence summary.
    pub fn as_str(self) -> &'static str {
        match self {
            EscalationTrigger::AntiSpiral => "anti_spiral",
            EscalationTrigger::HighRisk => "high_risk",
            EscalationTrigger::ReplanMajor => "replan_major",
            EscalationTrigger::CircuitBreaker => "circuit_breaker",
        }
    }
}

// ---------------------------------------------------------------------------
// AutonomyDecision — output of AutonomousGate::decide
// ---------------------------------------------------------------------------

/// Autonomous gate output. Orthogonal to the `GateDecision` enum in
/// `gate.rs`; callers dispatch on which gate ran.
///
/// Five variants per spec §4G.1.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum AutonomyDecision {
    /// All six evidence checks passed, no escalation fired, tree policy
    /// permits auto-advance at the current level. Proceed without human.
    AutoAdvance,
    /// Evidence failed, risk was HIGH, or the tree policy requires human
    /// confirmation at this level. Payload is used verbatim in the
    /// `GateDecision` SSB row + renderered inside InterventionCard.svelte.
    RequestHuman {
        reason: String,
        risk_level: RiskLevel,
        evidence: Evidence,
    },
    /// Context budget reached Terminal (see `crate::context_budget`). Split
    /// the active task into `sub_task_count` (≤4) sub-tasks at `depth` (≤2).
    SplitTask {
        sub_task_count: u32,
        depth: u8,
    },
    /// max_restarts exhausted. Swap the active model identifier from `from`
    /// to `to` via the ModelPool / provider config. Emits a `ModelSwap`
    /// SSB row (already part of the enum, no schema bump required for this
    /// variant).
    SubstituteModel {
        from: String,
        to: String,
    },
    /// An escalation trigger fired. The task halts and a human is required
    /// regardless of mode.
    Escalate {
        trigger: EscalationTrigger,
    },
}

// ---------------------------------------------------------------------------
// GateContext — per-call inputs
// ---------------------------------------------------------------------------

/// Inputs the gate needs to make a decision. Composed by the caller
/// (task_runner) from SSB snapshots + current pipeline position.
#[derive(Debug, Clone)]
pub struct GateContext {
    /// Current pipeline level.
    pub level: PipelineLevel,
    /// Risk level inherited from the FSM transition (from
    /// `fsm::resolve_risk_level`).
    pub risk_level: RiskLevel,
    /// Evidence snapshot computed for this gate.
    pub evidence: Evidence,
    /// True iff [`crate::anti_spiral::AntiSpiralDetector::record_step`]
    /// returned HardStop / EscalationSignalNeeded during the current phase.
    pub anti_spiral_fired: bool,
    /// True iff an SSB `ReplanRequest` row with severity `"MAJOR"` is
    /// pending for the current task.
    pub replan_major: bool,
    /// True iff the active baby-step sub-tree has exhausted max_restarts.
    /// Drives [`AutonomyDecision::SubstituteModel`].
    pub max_restarts_exhausted: bool,
    /// True iff the context budget reported Terminal at the last sample.
    /// Drives [`AutonomyDecision::SplitTask`].
    pub context_budget_terminal: bool,
    /// Auto-advance count for the current concept — used by the tree-level
    /// 50/concept circuit breaker.
    pub auto_advance_count_for_concept: u32,
    /// Diagnostic label for the step / transition being evaluated. Used
    /// verbatim in the `GateDecision.step_id` SSB field.
    pub step_id: String,
}

// ---------------------------------------------------------------------------
// AutonomousGateConfig — per-gate configuration
// ---------------------------------------------------------------------------

/// Runtime configuration for [`AutonomousGate`]. Defaults per spec §4G.1:
/// Interactive mode, circuit breaker limit 10 per session, default tree
/// policy, two-model fallback chain (7B → 3B).
#[derive(Debug, Clone)]
pub struct AutonomousGateConfig {
    pub mode: GateMode,
    pub circuit_breaker_limit: u32,
    pub tree_auto_advance_policy: TreeAutoAdvancePolicy,
    pub model_fallback_chain: Vec<String>,
}

impl Default for AutonomousGateConfig {
    fn default() -> Self {
        Self {
            mode: GateMode::Interactive,
            circuit_breaker_limit: 10,
            tree_auto_advance_policy: TreeAutoAdvancePolicy::default(),
            model_fallback_chain: vec![
                "qwen2.5-coder-7b".to_owned(),
                "qwen2.5-coder-3b".to_owned(),
            ],
        }
    }
}

// ---------------------------------------------------------------------------
// AutonomousGate — the main type
// ---------------------------------------------------------------------------

/// Autonomous gate wrapping [`InteractiveGate`]. Emits
/// [`AutonomyDecision`] per call to [`AutonomousGate::decide`].
///
/// Mode is shared via `Arc<RwLock<_>>` so the `/mode` slash command can
/// flip it at runtime without rebuilding the gate.
pub struct AutonomousGate {
    config: Arc<RwLock<AutonomousGateConfig>>,
    #[allow(dead_code)] // Kept for future MODIFY / REDIRECT reuse in mixed-mode.
    interactive_gate: Option<Arc<dyn InteractiveGate>>,
    circuit_breaker_count: Arc<AtomicU32>,
    /// Active model identifier for the SUBSTITUTE_MODEL protocol. Swapped
    /// via [`AutonomousGate::substitute_model`]; exposed via
    /// [`AutonomousGate::active_model`]. Arc<RwLock> so the
    /// SidecarProvider can re-read on each dispatch without locking the
    /// full config.
    active_model: Arc<RwLock<String>>,
}

impl AutonomousGate {
    /// Construct an autonomous gate. `interactive_gate` is optional — pass
    /// `None` when the task_runner is not expected to fall back to human
    /// prompts (headless batch runs).
    pub fn new(
        config: AutonomousGateConfig,
        interactive_gate: Option<Arc<dyn InteractiveGate>>,
    ) -> Self {
        let initial_model = config
            .model_fallback_chain
            .first()
            .cloned()
            .unwrap_or_default();
        Self {
            config: Arc::new(RwLock::new(config)),
            interactive_gate,
            circuit_breaker_count: Arc::new(AtomicU32::new(0)),
            active_model: Arc::new(RwLock::new(initial_model)),
        }
    }

    /// Core decision routine. Order-of-evaluation per spec §4G.2:
    /// 1. Env kill switch → RequestHuman (kill switch escape hatch).
    /// 2. Escalation triggers (AntiSpiral > HighRisk > ReplanMajor >
    ///    CircuitBreaker) → Escalate.
    /// 3. Context budget Terminal → SplitTask (autonomous only).
    /// 4. max_restarts_exhausted → SubstituteModel (autonomous only).
    /// 5. Interactive mode → RequestHuman unconditionally for LOW+ risk.
    /// 6. Tree-level auto-advance check → AutoAdvance or RequestHuman.
    /// 7. Evidence all-passed → AutoAdvance, else RequestHuman.
    pub async fn decide(&self, ctx: &GateContext) -> AutonomyDecision {
        // 1. Kill switch.
        if autonomous_gate_disabled_via_env() {
            return AutonomyDecision::RequestHuman {
                reason: "autonomous gate disabled via env".to_owned(),
                risk_level: ctx.risk_level,
                evidence: ctx.evidence.clone(),
            };
        }

        // 2. Escalation triggers. Order matters — AntiSpiral is the loudest.
        if ctx.anti_spiral_fired {
            return AutonomyDecision::Escalate {
                trigger: EscalationTrigger::AntiSpiral,
            };
        }
        if ctx.risk_level == RiskLevel::High {
            return AutonomyDecision::Escalate {
                trigger: EscalationTrigger::HighRisk,
            };
        }
        if ctx.replan_major {
            return AutonomyDecision::Escalate {
                trigger: EscalationTrigger::ReplanMajor,
            };
        }
        // Circuit breaker count reflects auto-advances already granted — trip
        // on the N+1th call. Read mode to inspect limit without locking twice.
        let (mode, limit, policy) = {
            let cfg = self.config.read().await;
            (
                cfg.mode,
                cfg.circuit_breaker_limit,
                cfg.tree_auto_advance_policy.clone(),
            )
        };
        if self.circuit_breaker_count.load(Ordering::Acquire) >= limit {
            return AutonomyDecision::Escalate {
                trigger: EscalationTrigger::CircuitBreaker,
            };
        }

        // 3. Context budget Terminal → SplitTask (autonomous mode only).
        if mode == GateMode::Autonomous && ctx.context_budget_terminal {
            return AutonomyDecision::SplitTask {
                sub_task_count: 4,
                depth: 2,
            };
        }

        // 4. max_restarts_exhausted → SubstituteModel.
        if mode == GateMode::Autonomous && ctx.max_restarts_exhausted {
            if let Some(next) = self.next_fallback_model().await {
                let from = {
                    let m = self.active_model.read().await;
                    m.clone()
                };
                return AutonomyDecision::SubstituteModel {
                    from,
                    to: next,
                };
            }
            // No fallback left — escalate instead.
            return AutonomyDecision::Escalate {
                trigger: EscalationTrigger::HighRisk,
            };
        }

        // 5. Interactive mode: every non-LOW transition prompts a human.
        // LOW risk is the only auto-advance path in Interactive mode.
        if mode == GateMode::Interactive && ctx.risk_level != RiskLevel::Low {
            return AutonomyDecision::RequestHuman {
                reason: "interactive mode".to_owned(),
                risk_level: ctx.risk_level,
                evidence: ctx.evidence.clone(),
            };
        }

        // 6. Tree-level circuit breaker: cap auto baby-steps per concept.
        if ctx.level == PipelineLevel::BabyStep
            && ctx.auto_advance_count_for_concept >= policy.max_auto_baby_steps_per_concept
        {
            return AutonomyDecision::RequestHuman {
                reason: format!(
                    "per-concept auto-advance cap reached ({} baby steps)",
                    policy.max_auto_baby_steps_per_concept
                ),
                risk_level: ctx.risk_level,
                evidence: ctx.evidence.clone(),
            };
        }

        // 7. Tree-level policy.
        if !policy.should_auto_advance(ctx.level, ctx.risk_level) {
            return AutonomyDecision::RequestHuman {
                reason: format!("tree policy: {:?} requires human", ctx.level),
                risk_level: ctx.risk_level,
                evidence: ctx.evidence.clone(),
            };
        }

        // 8. Evidence checks.
        if !ctx.evidence.all_passed() {
            return AutonomyDecision::RequestHuman {
                reason: format!(
                    "evidence failed: {}",
                    ctx.evidence.failed_checks().join(",")
                ),
                risk_level: ctx.risk_level,
                evidence: ctx.evidence.clone(),
            };
        }

        // Happy path — bump the circuit breaker and auto-advance.
        self.circuit_breaker_count.fetch_add(1, Ordering::AcqRel);
        AutonomyDecision::AutoAdvance
    }

    /// Flip the runtime gate mode. Invoked by the `/mode` slash command.
    pub async fn set_mode(&self, mode: GateMode) -> GateMode {
        let mut cfg = self.config.write().await;
        let prev = cfg.mode;
        cfg.mode = mode;
        prev
    }

    /// Read the current mode. Used by the StatusBar polling path.
    pub async fn mode(&self) -> GateMode {
        let cfg = self.config.read().await;
        cfg.mode
    }

    /// Current circuit-breaker count. Exposed so `get_gate_status` can
    /// render "circuit N/10".
    pub fn circuit_breaker_count(&self) -> u32 {
        self.circuit_breaker_count.load(Ordering::Acquire)
    }

    /// Current circuit-breaker limit.
    pub async fn circuit_breaker_limit(&self) -> u32 {
        let cfg = self.config.read().await;
        cfg.circuit_breaker_limit
    }

    /// Reset the per-session circuit breaker. Called by task_runner at
    /// MEMORIZING → next-phase boundary.
    pub fn reset_circuit_breaker(&self) {
        self.circuit_breaker_count.store(0, Ordering::Release);
    }

    /// Active model identifier (the head of the fallback chain after any
    /// SUBSTITUTE_MODEL swaps).
    pub async fn active_model(&self) -> String {
        self.active_model.read().await.clone()
    }

    /// Execute the SUBSTITUTE_MODEL protocol: swap `active_model` to the
    /// next entry in the fallback chain. Returns `(from, to)` when a swap
    /// happened, `None` when the chain is exhausted.
    pub async fn substitute_model(&self) -> Option<(String, String)> {
        let next = self.next_fallback_model().await?;
        let from = {
            let mut m = self.active_model.write().await;
            let old = m.clone();
            *m = next.clone();
            old
        };
        Some((from, next))
    }

    /// Return the next entry in the fallback chain after `active_model`, or
    /// `None` if the active model is the last entry (or not in the chain).
    async fn next_fallback_model(&self) -> Option<String> {
        let cfg = self.config.read().await;
        let current = self.active_model.read().await.clone();
        let chain = &cfg.model_fallback_chain;
        let idx = chain.iter().position(|m| m == &current)?;
        chain.get(idx + 1).cloned()
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    fn all_pass_evidence() -> Evidence {
        Evidence {
            knowledge_readiness: true,
            zero_ungrounded: true,
            tests_fail_correctly: true,
            test_flipped: true,
            zero_blocks: true,
            dkg_and_changelog_present: true,
        }
    }

    fn ctx_with(level: PipelineLevel, risk: RiskLevel) -> GateContext {
        GateContext {
            level,
            risk_level: risk,
            evidence: all_pass_evidence(),
            anti_spiral_fired: false,
            replan_major: false,
            max_restarts_exhausted: false,
            context_budget_terminal: false,
            auto_advance_count_for_concept: 0,
            step_id: "step-1".into(),
        }
    }

    #[tokio::test]
    async fn auto_advance_when_all_evidence_passes_and_low_risk() {
        let gate = AutonomousGate::new(
            AutonomousGateConfig {
                mode: GateMode::Autonomous,
                ..AutonomousGateConfig::default()
            },
            None,
        );
        let ctx = ctx_with(PipelineLevel::BabyStep, RiskLevel::Low);
        let decision = gate.decide(&ctx).await;
        assert_eq!(decision, AutonomyDecision::AutoAdvance);
    }

    #[tokio::test]
    async fn interactive_mode_prompts_for_medium_risk() {
        let gate = AutonomousGate::new(AutonomousGateConfig::default(), None);
        let ctx = ctx_with(PipelineLevel::Phase, RiskLevel::Medium);
        let decision = gate.decide(&ctx).await;
        assert!(matches!(decision, AutonomyDecision::RequestHuman { .. }));
    }

    #[tokio::test]
    async fn high_risk_always_escalates() {
        let gate = AutonomousGate::new(
            AutonomousGateConfig {
                mode: GateMode::Autonomous,
                ..AutonomousGateConfig::default()
            },
            None,
        );
        let ctx = ctx_with(PipelineLevel::Phase, RiskLevel::High);
        let decision = gate.decide(&ctx).await;
        assert!(matches!(
            decision,
            AutonomyDecision::Escalate {
                trigger: EscalationTrigger::HighRisk,
            }
        ));
    }

    #[tokio::test]
    async fn anti_spiral_escalates_over_everything() {
        let gate = AutonomousGate::new(
            AutonomousGateConfig {
                mode: GateMode::Autonomous,
                ..AutonomousGateConfig::default()
            },
            None,
        );
        let mut ctx = ctx_with(PipelineLevel::BabyStep, RiskLevel::Low);
        ctx.anti_spiral_fired = true;
        let decision = gate.decide(&ctx).await;
        assert!(matches!(
            decision,
            AutonomyDecision::Escalate {
                trigger: EscalationTrigger::AntiSpiral,
            }
        ));
    }

    #[tokio::test]
    async fn circuit_breaker_trips_after_limit() {
        let gate = AutonomousGate::new(
            AutonomousGateConfig {
                mode: GateMode::Autonomous,
                circuit_breaker_limit: 2,
                ..AutonomousGateConfig::default()
            },
            None,
        );
        let ctx = ctx_with(PipelineLevel::BabyStep, RiskLevel::Low);
        // First two auto-advances OK.
        for _ in 0..2 {
            let d = gate.decide(&ctx).await;
            assert_eq!(d, AutonomyDecision::AutoAdvance);
        }
        // Third decision trips the breaker.
        let d = gate.decide(&ctx).await;
        assert!(matches!(
            d,
            AutonomyDecision::Escalate {
                trigger: EscalationTrigger::CircuitBreaker,
            }
        ));
    }

    #[tokio::test]
    async fn context_budget_terminal_triggers_split() {
        let gate = AutonomousGate::new(
            AutonomousGateConfig {
                mode: GateMode::Autonomous,
                ..AutonomousGateConfig::default()
            },
            None,
        );
        let mut ctx = ctx_with(PipelineLevel::Step, RiskLevel::Low);
        ctx.context_budget_terminal = true;
        let decision = gate.decide(&ctx).await;
        assert_eq!(
            decision,
            AutonomyDecision::SplitTask {
                sub_task_count: 4,
                depth: 2,
            }
        );
    }

    #[tokio::test]
    async fn substitute_model_on_max_restarts() {
        let gate = AutonomousGate::new(
            AutonomousGateConfig {
                mode: GateMode::Autonomous,
                model_fallback_chain: vec!["m1".into(), "m2".into(), "m3".into()],
                ..AutonomousGateConfig::default()
            },
            None,
        );
        let mut ctx = ctx_with(PipelineLevel::Step, RiskLevel::Low);
        ctx.max_restarts_exhausted = true;
        let decision = gate.decide(&ctx).await;
        assert_eq!(
            decision,
            AutonomyDecision::SubstituteModel {
                from: "m1".into(),
                to: "m2".into(),
            }
        );
    }

    #[tokio::test]
    async fn substitute_model_chain_swap() {
        let gate = AutonomousGate::new(
            AutonomousGateConfig {
                mode: GateMode::Autonomous,
                model_fallback_chain: vec!["a".into(), "b".into()],
                ..AutonomousGateConfig::default()
            },
            None,
        );
        assert_eq!(gate.active_model().await, "a");
        let swap = gate.substitute_model().await;
        assert_eq!(swap, Some(("a".into(), "b".into())));
        assert_eq!(gate.active_model().await, "b");
        // Chain exhausted.
        assert_eq!(gate.substitute_model().await, None);
    }

    #[tokio::test]
    async fn set_mode_roundtrip() {
        let gate = AutonomousGate::new(AutonomousGateConfig::default(), None);
        assert_eq!(gate.mode().await, GateMode::Interactive);
        let prev = gate.set_mode(GateMode::Autonomous).await;
        assert_eq!(prev, GateMode::Interactive);
        assert_eq!(gate.mode().await, GateMode::Autonomous);
    }

    #[tokio::test]
    async fn evidence_failure_requests_human() {
        let gate = AutonomousGate::new(
            AutonomousGateConfig {
                mode: GateMode::Autonomous,
                ..AutonomousGateConfig::default()
            },
            None,
        );
        let mut ctx = ctx_with(PipelineLevel::BabyStep, RiskLevel::Low);
        ctx.evidence.zero_ungrounded = false;
        let decision = gate.decide(&ctx).await;
        match decision {
            AutonomyDecision::RequestHuman { reason, .. } => {
                assert!(reason.contains("zero_ungrounded"));
            }
            other => panic!("expected RequestHuman, got {other:?}"),
        }
    }

    #[tokio::test]
    async fn per_concept_cap_requests_human() {
        let gate = AutonomousGate::new(
            AutonomousGateConfig {
                mode: GateMode::Autonomous,
                tree_auto_advance_policy: TreeAutoAdvancePolicy {
                    max_auto_baby_steps_per_concept: 3,
                    ..TreeAutoAdvancePolicy::default()
                },
                // Make sure the session-wide breaker doesn't fire first.
                circuit_breaker_limit: 1_000,
                ..AutonomousGateConfig::default()
            },
            None,
        );
        let mut ctx = ctx_with(PipelineLevel::BabyStep, RiskLevel::Low);
        ctx.auto_advance_count_for_concept = 3;
        let decision = gate.decide(&ctx).await;
        match decision {
            AutonomyDecision::RequestHuman { reason, .. } => {
                assert!(reason.contains("cap reached"));
            }
            other => panic!("expected RequestHuman, got {other:?}"),
        }
    }

    #[test]
    fn tree_policy_concept_is_always_human() {
        let p = TreeAutoAdvancePolicy::default();
        assert!(!p.should_auto_advance(PipelineLevel::Concept, RiskLevel::Low));
    }

    #[test]
    fn tree_policy_phase_human_when_medium() {
        let p = TreeAutoAdvancePolicy::default();
        assert!(p.should_auto_advance(PipelineLevel::Phase, RiskLevel::Low));
        assert!(!p.should_auto_advance(PipelineLevel::Phase, RiskLevel::Medium));
    }

    #[test]
    fn gate_mode_parser() {
        assert_eq!(GateMode::from_str("interactive"), Some(GateMode::Interactive));
        assert_eq!(
            GateMode::from_str("  AUTONOMOUS  "),
            Some(GateMode::Autonomous)
        );
        assert_eq!(GateMode::from_str("rogue"), None);
    }

    #[test]
    fn evidence_summary_format() {
        let mut e = Evidence::default();
        e.knowledge_readiness = true;
        assert_eq!(e.summary(), "kr=1 zu=0 tfc=0 tf=0 zb=0 dcg=0");
    }

    #[test]
    fn pipeline_level_risk_defaults() {
        assert_eq!(PipelineLevel::Concept.default_risk(), RiskLevel::High);
        assert_eq!(PipelineLevel::Phase.default_risk(), RiskLevel::Medium);
        assert_eq!(PipelineLevel::Step.default_risk(), RiskLevel::Low);
        assert_eq!(PipelineLevel::BabyStep.default_risk(), RiskLevel::Low);
    }
}
