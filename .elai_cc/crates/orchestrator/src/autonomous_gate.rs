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

use std::path::PathBuf;
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::Arc;

use rustc_hash::FxHashMap;
use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;

use crate::agent_pool::ModelPool;
use crate::gate::{InteractiveGate, RiskLevel};
use crate::skill_trust::SkillTrustTier;

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

/// F-6 (C6a) — kill switch for the learned-skill autonomy barrier. When set to
/// any non-empty string, [`AutonomousGate::decide`] skips the sub-S4 skill
/// barrier (step 7.5) entirely, restoring pre-F-6 behaviour. Default-ON
/// (W-2): the barrier is active unless this var is explicitly set. Mirrors the
/// [`autonomous_gate_disabled_via_env`] emergency-shutoff pattern.
pub const ELAI_SKILL_AUTONOMY_BARRIER_DISABLE_ENV: &str =
    "ELAI_SKILL_AUTONOMY_BARRIER_DISABLE";

/// Returns true iff the learned-skill autonomy barrier is disabled via env.
/// Cheap: single env read. When true, a sub-S4 candidate skill does NOT force
/// [`AutonomyDecision::RequestHuman`].
pub fn skill_autonomy_barrier_disabled() -> bool {
    std::env::var(ELAI_SKILL_AUTONOMY_BARRIER_DISABLE_ENV)
        .map(|v| !v.is_empty())
        .unwrap_or(false)
}

/// F-6 (C6a) — true iff a candidate skill at `tier` is cleared for autonomous
/// dispatch. Only the apex rung [`SkillTrustTier::S4`] permits autonomy; every
/// sub-S4 candidate routes to a human via the barrier in
/// [`AutonomousGate::decide`] step 7.5. Mirrors
/// [`crate::skill_trust::SkillStore::permits_autonomous`] WITHOUT taking a
/// store handle — the gate stays decoupled from the [`crate::skill_trust::SkillStore`].
#[must_use]
pub fn skill_permits_autonomous(tier: SkillTrustTier) -> bool {
    tier == SkillTrustTier::S4
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

    /// C-9 — resolve the gate mode from a parsed `--mode <value>` CLI flag.
    /// Accepts the same three labels as [`crate::cli_mode::CliGateMode`]:
    /// `interactive`, `autonomous`, `hybrid`. `hybrid` resolves to
    /// [`GateMode::Interactive`] for the gate itself (hybrid mode is enforced
    /// at the dispatcher tier — the gate only knows two modes today).
    /// Returns [`GateMode::Interactive`] when `raw` is `None` to preserve the
    /// pre-C-9 default at the production wiring point.
    pub fn from_cli_args(raw: Option<&str>) -> Self {
        match raw.map(|s| s.trim().to_ascii_lowercase()) {
            Some(ref v) if v == "autonomous" => GateMode::Autonomous,
            Some(ref v) if v == "interactive" => GateMode::Interactive,
            Some(ref v) if v == "hybrid" => GateMode::Interactive,
            _ => GateMode::Interactive,
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
///
/// B-40 added the 7th check `framing_converged` for the FRAMING substate.
/// Gated on `agent_substate == Framing` — for all other states/substates the
/// field defaults to `true` (vacuous-default per task_runner.rs:6996 pattern)
/// so the existing 14 autonomous_gate_contract.rs tests pass UNCHANGED.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
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
    /// B-40 Check 7 — PLANNING + FRAMING substate: an `IdeaFramed` row with
    /// `converged=true` is present for the current session. Vacuous-default
    /// `true` for any non-FRAMING context — only `derive_framing_evidence`
    /// at the FRAMING call site flips this to a real value. The
    /// `#[serde(default = "default_true")]` makes older Evidence snapshots
    /// (pre-B-40 v40 SSB rows) decode with `framing_converged=true`,
    /// preserving the autonomous_gate_contract.rs 14-test invariant.
    #[serde(default = "default_true_for_framing_converged")]
    pub framing_converged: bool,
    /// B-41 Check 8 — Concepts-tier per-level audit produced ZERO BLOCK
    /// verdicts that are still outstanding (i.e. no `cascade_to_parent=true`
    /// row exists for the AuditorConcepts mode). Vacuous-default `true` for
    /// any non-Concepts evaluation context — only
    /// `task_runner::derive_concept_zero_blocks` flips this to a real value
    /// from the SSB row stream. Matches the framing_converged pattern.
    #[serde(default = "default_true_for_per_level_zero_blocks")]
    pub concept_zero_blocks: bool,
    /// B-41 Check 9 — Phases-tier per-level audit zero-blocks. Vacuous-default
    /// true (see [`Evidence::concept_zero_blocks`]).
    #[serde(default = "default_true_for_per_level_zero_blocks")]
    pub phase_zero_blocks: bool,
    /// B-41 Check 10 — Steps-tier per-level audit zero-blocks. Vacuous-default
    /// true (see [`Evidence::concept_zero_blocks`]).
    #[serde(default = "default_true_for_per_level_zero_blocks")]
    pub step_zero_blocks: bool,
    /// B-41 Check 11 — BabySteps-tier per-level audit zero-blocks.
    /// Vacuous-default true (see [`Evidence::concept_zero_blocks`]).
    #[serde(default = "default_true_for_per_level_zero_blocks")]
    pub babystep_zero_blocks: bool,
    /// B-41 Check 12 — Plan-tier per-level audit zero-blocks. Vacuous-default
    /// true (see [`Evidence::concept_zero_blocks`]).
    #[serde(default = "default_true_for_per_level_zero_blocks")]
    pub plan_zero_blocks: bool,
    /// F-10 Check 13 — AUTO-ADVANCE security monotonicity: advancing to the
    /// next step would NOT introduce a CWE security class that was absent at the
    /// prior iteration. Vacuous-default `true` for any context that did not run
    /// the security-monotonicity gate — only
    /// `task_runner::derive_security_monotonicity_evidence` flips this to a real
    /// value (set `false` on a regression, which blocks auto-advance and emits a
    /// `SecurityClassRegressionBlocked` SSB row with `loop_kind=AutoAdvance`).
    /// Same vacuous-default pattern as `framing_converged` / the B-41 fields, so
    /// the existing autonomous_gate cold-start contracts pass UNCHANGED.
    #[serde(default = "default_true_for_security_monotonicity")]
    pub security_monotonicity: bool,
    /// SP5 wiring-fix Check 14 — EXECUTING -> AUDITING Layer-3 entrypoint
    /// coverage: every newly-added function was driven from a REAL entrypoint
    /// (instrumented-binary hit-count > 0), proven by an in-window
    /// `wiring_evidence` SSB row whose `entrypoint == LAYER3_LEG`
    /// ("layer3_coverage") AND zero in-window `wiring_violation` rows whose
    /// `file == LAYER3_LEG`. Unlike the historical vacuous-default-`true`
    /// checks, this defaults to `false` (FAIL-CLOSED): a context that did not
    /// run the Layer-3 coverage runner observes `false` and CANNOT
    /// auto-advance. Set `true` only by `derive_layer3_entrypoint_covered`.
    /// Source: wiring-elai-adr.md ADR-2 + wiring-enforcement.md R1/R4.
    #[serde(default = "default_false_for_layer3_entrypoint_covered")]
    pub layer3_entrypoint_covered: bool,
    /// SP5 wiring-fix Check 15 — EXECUTING -> AUDITING Layer-4 off-agent
    /// trace: a real SSB row landed for the driven entrypoint (an artifact
    /// the producing agent did not author), proven by an in-window
    /// `wiring_evidence` SSB row whose `entrypoint == LAYER4_LEG`
    /// ("layer4_trace"). Defaults to `false` (FAIL-CLOSED) for the same
    /// reason as `layer3_entrypoint_covered`. Set `true` only by
    /// `derive_layer4_trace_present` (WAVE 5 emits the row). Source:
    /// wiring-elai-adr.md ADR-1/ADR-2 + wiring-enforcement.md R1/R4.
    #[serde(default = "default_false_for_layer4_trace_present")]
    pub layer4_trace_present: bool,
}

impl Default for Evidence {
    fn default() -> Self {
        Self {
            knowledge_readiness: false,
            zero_ungrounded: false,
            tests_fail_correctly: false,
            test_flipped: false,
            zero_blocks: false,
            dkg_and_changelog_present: false,
            // Vacuous default true: pre-FRAMING phases (i.e. phases not
            // gated on framing) must not be blocked by an unset field.
            framing_converged: true,
            // B-41: vacuous-default true for all 5 per-level audit
            // zero-blocks fields. Same pattern as `framing_converged` —
            // contexts not in that tier's evaluation window observe `true`
            // and the existing 14 autonomous_gate_contract.rs + 11
            // autonomous_gate_with_framing_contract.rs tests pass UNCHANGED.
            concept_zero_blocks: true,
            phase_zero_blocks: true,
            step_zero_blocks: true,
            babystep_zero_blocks: true,
            plan_zero_blocks: true,
            // F-10: vacuous-default true. A context that did not run the
            // security-monotonicity gate observes `true` and is not blocked;
            // only derive_security_monotonicity_evidence sets a real value.
            security_monotonicity: true,
            // SP5 wiring-fix: FAIL-CLOSED defaults. Unlike every check above,
            // these default FALSE: a context that did not run the Layer-3
            // coverage runner / observe a Layer-4 trace row must NOT
            // auto-advance EXECUTING -> AUDITING on a unit-test proxy alone.
            layer3_entrypoint_covered: false,
            layer4_trace_present: false,
        }
    }
}

fn default_true_for_framing_converged() -> bool {
    true
}

/// SP5 wiring-fix serde default for [`Evidence::layer3_entrypoint_covered`].
/// FAIL-CLOSED: a pre-SP5 serialized Evidence snapshot (no field) decodes to
/// `false`, so an old snapshot cannot auto-advance without a fresh Layer-3
/// coverage derivation.
fn default_false_for_layer3_entrypoint_covered() -> bool {
    false
}

/// SP5 wiring-fix serde default for [`Evidence::layer4_trace_present`].
/// FAIL-CLOSED for the same reason as
/// [`default_false_for_layer3_entrypoint_covered`].
fn default_false_for_layer4_trace_present() -> bool {
    false
}

fn default_true_for_per_level_zero_blocks() -> bool {
    true
}

fn default_true_for_security_monotonicity() -> bool {
    true
}

impl Evidence {
    /// Check 1: knowledge readiness. Mirrors [`KnowledgeReadinessGate`].
    /// Source: orchestrator.txt §13 L626-629.
    pub fn check_knowledge_readiness(&self) -> EvidenceVerdict {
        if self.knowledge_readiness {
            EvidenceVerdict::Pass
        } else {
            EvidenceVerdict::Fail {
                reason: "knowledge_readiness".to_string(),
            }
        }
    }

    /// Check 2: zero UNGROUNDED claims. Mirrors [`GroundingSweepGate`].
    /// Source: orchestrator.txt §13 L631.
    pub fn check_zero_ungrounded(&self) -> EvidenceVerdict {
        if self.zero_ungrounded {
            EvidenceVerdict::Pass
        } else {
            EvidenceVerdict::Fail {
                reason: "zero_ungrounded".to_string(),
            }
        }
    }

    /// Check 3: tests fail correctly (not syntax errors). Mirrors [`TestFailReasonsGate`].
    /// Source: orchestrator.txt §13 L632.
    pub fn check_tests_fail_correctly(&self) -> EvidenceVerdict {
        if self.tests_fail_correctly {
            EvidenceVerdict::Pass
        } else {
            EvidenceVerdict::Fail {
                reason: "tests_fail_correctly".to_string(),
            }
        }
    }

    /// Check 4: test flipped FAIL -> PASS. Mirrors [`TestFlippedGate`].
    /// Source: orchestrator.txt §13 L633.
    pub fn check_test_flipped(&self) -> EvidenceVerdict {
        if self.test_flipped {
            EvidenceVerdict::Pass
        } else {
            EvidenceVerdict::Fail {
                reason: "test_flipped".to_string(),
            }
        }
    }

    /// Check 5: zero BLOCK findings. Mirrors [`ZeroBlocksGate`].
    /// Source: orchestrator.txt §13 L639.
    pub fn check_zero_blocks(&self) -> EvidenceVerdict {
        if self.zero_blocks {
            EvidenceVerdict::Pass
        } else {
            EvidenceVerdict::Fail {
                reason: "zero_blocks".to_string(),
            }
        }
    }

    /// Check 6: DKG reflects new code and changelog has entry. Mirrors [`DkgChangelogGate`].
    /// Source: orchestrator.txt §13 L640.
    pub fn check_dkg_and_changelog_present(&self) -> EvidenceVerdict {
        if self.dkg_and_changelog_present {
            EvidenceVerdict::Pass
        } else {
            EvidenceVerdict::Fail {
                reason: "dkg_and_changelog_present".to_string(),
            }
        }
    }

    /// Every check passed. Required for any [`AutonomyDecision::AutoAdvance`].
    ///
    /// B-40: includes `framing_converged` as the 7th check. Because the field
    /// defaults to `true` for non-FRAMING contexts (vacuous-default), the
    /// existing 14 autonomous_gate_contract.rs tests pass UNCHANGED — only
    /// FRAMING-substate call sites flip the field to a real value via
    /// `derive_framing_evidence`.
    pub fn all_passed(&self) -> bool {
        self.knowledge_readiness
            && self.zero_ungrounded
            && self.tests_fail_correctly
            && self.test_flipped
            && self.zero_blocks
            && self.dkg_and_changelog_present
            && self.framing_converged
            // B-41 per-level audit zero-blocks (vacuous-default true).
            && self.concept_zero_blocks
            && self.phase_zero_blocks
            && self.step_zero_blocks
            && self.babystep_zero_blocks
            && self.plan_zero_blocks
            // F-10 AUTO-ADVANCE security monotonicity (vacuous-default true).
            && self.security_monotonicity
            // SP5 wiring-fix: EXECUTING -> AUDITING off-agent legs
            // (FAIL-CLOSED default false). A unit-test flip (`test_flipped`)
            // is necessary-but-not-sufficient; the auto-advance also requires
            // a Layer-3 entrypoint-coverage hit AND a Layer-4 off-agent SSB
            // trace row. wiring-enforcement.md R1/R4.
            && self.layer3_entrypoint_covered
            && self.layer4_trace_present
    }

    /// Human-readable summary for the `GateDecision` SSB payload. Format is
    /// the seven single-letter pass/fail flags in check order (B-40 adds fc).
    pub fn summary(&self) -> String {
        // NARROWING-OK: bool as u8 is always 0 or 1; no truncation possible.
        // B-41 NOTE: per-level audit zero-blocks are intentionally OMITTED
        // from this summary string. The B-40 contract
        // `autonomous_gate_with_framing_contract::contract_summary_includes_fc_suffix`
        // asserts the string ENDS with `fc=<0|1>` — appending more flags
        // breaks that invariant. The 5 B-41 flags are exposed via
        // `failed_checks()` (which has no such ordering invariant), via the
        // per-level SSB rows themselves, and via the dashboard timeline.
        format!(
            "kr={} zu={} tfc={} tf={} zb={} dcg={} fc={}",
            self.knowledge_readiness as u8,
            self.zero_ungrounded as u8,
            self.tests_fail_correctly as u8,
            self.test_flipped as u8,
            self.zero_blocks as u8,
            self.dkg_and_changelog_present as u8,
            self.framing_converged as u8,
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
        if !self.framing_converged {
            v.push("framing_converged");
        }
        // B-41 per-level audit zero-blocks failed-check labels.
        if !self.concept_zero_blocks {
            v.push("concept_zero_blocks");
        }
        if !self.phase_zero_blocks {
            v.push("phase_zero_blocks");
        }
        if !self.step_zero_blocks {
            v.push("step_zero_blocks");
        }
        if !self.babystep_zero_blocks {
            v.push("babystep_zero_blocks");
        }
        if !self.plan_zero_blocks {
            v.push("plan_zero_blocks");
        }
        // F-10 AUTO-ADVANCE security monotonicity failed-check label.
        if !self.security_monotonicity {
            v.push("security_monotonicity");
        }
        // SP5 wiring-fix: off-agent EXECUTING -> AUDITING legs. Surfaced so a
        // FAIL-CLOSED block reports a named reason rather than an empty list.
        if !self.layer3_entrypoint_covered {
            v.push("layer3_entrypoint_covered");
        }
        if !self.layer4_trace_present {
            v.push("layer4_trace_present");
        }
        v
    }
}

// ---------------------------------------------------------------------------
// EscalationTrigger — reasons the gate halts for a human
// ---------------------------------------------------------------------------

/// Discriminant for [`AutonomyDecision::Escalate`]. Source: phase-4G.txt §4G.4.
///
/// C-9: extended to 6 variants per spec §4G.1. `SubstituteModelFailed` fires
/// when the gate exhausts `model_fallback_chain` (or when
/// [`ModelPool::can_fit_llm`] reports the target won't fit in available VRAM).
/// `SplitTaskFailed` fires when [`crate::research_tree::ResearchTree::split_step`]
/// produces an empty sub-task vector for a non-empty plan, or when the runner
/// has no `ResearchTree` handle wired to dispatch against.
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
    /// C-9: the SUBSTITUTE_MODEL protocol could not satisfy the request —
    /// either the fallback chain is exhausted OR the target model exceeds
    /// available VRAM per [`ModelPool::can_fit_llm`].
    SubstituteModelFailed,
    /// C-9: the SPLIT_TASK dispatch could not produce sub-tasks — either
    /// the ResearchTree handle is unavailable to the gate's caller OR
    /// `ResearchTree::split_step` returned an empty vector for a non-empty
    /// `SplitTaskPlan`.
    SplitTaskFailed,
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
            EscalationTrigger::SubstituteModelFailed => "substitute_model_failed",
            EscalationTrigger::SplitTaskFailed => "split_task_failed",
        }
    }
}

// ---------------------------------------------------------------------------
// C-9 — FsmTransition discriminant for AutoAdvanceEvaluator
// ---------------------------------------------------------------------------

/// Named FSM-style transitions the [`AutoAdvanceEvaluator`] gates. Each variant
/// keys a registered [`EvidenceCheck`] in the evaluator's `gate_checks` map.
/// Source: phase-4G.txt §4G.1; consumed by the gate's evaluator dispatch.
///
/// The variant names mirror `EscalationTrigger` labels for the first four
/// entries so the per-transition check semantics map 1:1 onto the existing
/// trigger taxonomy; `AutoAdvance` is the catch-all key the gate consults
/// when no escalation trigger is active.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum FsmTransition {
    /// Anti-spiral trigger boundary. Maps to the `anti_spiral_fired` field
    /// in [`GateContext`].
    AntiSpiral,
    /// Risk-level boundary (HIGH). Maps to `risk_level` in [`GateContext`].
    HighRisk,
    /// Replan-major boundary. Maps to `replan_major` in [`GateContext`].
    ReplanMajor,
    /// Circuit-breaker boundary (per-session auto-advance cap reached).
    CircuitBreaker,
    /// Default boundary — all evidence checks pass. Maps to
    /// [`Evidence::all_passed`].
    AutoAdvance,
}

impl FsmTransition {
    /// Snake-case label for logging and SSB summaries.
    pub fn as_str(self) -> &'static str {
        match self {
            FsmTransition::AntiSpiral => "anti_spiral",
            FsmTransition::HighRisk => "high_risk",
            FsmTransition::ReplanMajor => "replan_major",
            FsmTransition::CircuitBreaker => "circuit_breaker",
            FsmTransition::AutoAdvance => "auto_advance",
        }
    }
}

// ---------------------------------------------------------------------------
// C-9 — EvidenceVerdict + EvidenceCheck + AutoAdvanceEvaluator
// ---------------------------------------------------------------------------

/// Outcome of a single [`EvidenceCheck::check_fn`] invocation.
/// Source: phase-4G.txt §4G.1 (the three-state evidence verdict).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum EvidenceVerdict {
    /// The check passed — auto-advance is permitted on this axis.
    Pass,
    /// The check failed. `reason` is the snake-case label of the failing
    /// underlying evidence field for logging + SSB row labelling.
    Fail { reason: String },
    /// No registered check fires for this transition. The evaluator treats
    /// `Missing` as "no opinion" — caller falls back to
    /// [`Evidence::all_passed`] via [`AutoAdvanceEvaluator::evaluate_all`].
    Missing,
}

impl EvidenceVerdict {
    /// Snake-case label used in SSB row reasons and `EscalationFired.trigger`
    /// strings. Returns `pass`, `fail`, or `missing`; the inner `reason` of
    /// `Fail { reason }` is exposed separately by callers when they need it.
    pub fn as_str(&self) -> &'static str {
        match self {
            EvidenceVerdict::Pass => "pass",
            EvidenceVerdict::Fail { .. } => "fail",
            EvidenceVerdict::Missing => "missing",
        }
    }

    /// True iff the verdict permits auto-advance.
    pub fn is_pass(&self) -> bool {
        matches!(self, EvidenceVerdict::Pass)
    }
}

/// A single check registered against an [`FsmTransition`]. `check_fn` is a
/// pure function over a snapshot of [`Evidence`] — no async, no SSB access,
/// no clock — so the evaluator stays unit-testable without spinning up the
/// runtime. The caller (gate's `derive_autonomous_evidence`) is responsible
/// for populating `Evidence` from the SSB stream BEFORE the evaluator runs.
///
/// `description` is a human-readable label rendered in dashboard
/// AutoAdvanceStats reason strings; `artifact_path` records the optional
/// source-of-truth artefact (e.g. `metrics/phase_4G.toml`) the check is
/// gated on so the evaluator can attribute a failure to a missing file.
#[derive(Clone)]
pub struct EvidenceCheck {
    /// Optional artefact path the check is gated on (for diagnostic display).
    pub artifact_path: Option<PathBuf>,
    /// Human-readable description of the check.
    pub description: &'static str,
    /// Pure check function. Implementations consult fields on `Evidence`
    /// and return one of [`EvidenceVerdict::Pass`] / `Fail { reason }` /
    /// `Missing`. Use `Missing` when the check is not applicable in the
    /// current context (e.g. per-level audit zero-blocks for a non-Concepts
    /// transition).
    pub check_fn: fn(&Evidence) -> EvidenceVerdict,
}

impl std::fmt::Debug for EvidenceCheck {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("EvidenceCheck")
            .field("artifact_path", &self.artifact_path)
            .field("description", &self.description)
            .field("check_fn", &"<fn>")
            .finish()
    }
}

impl PartialEq for EvidenceCheck {
    /// Equality compares the artefact path and description; function
    /// pointers do not have stable equality on every platform.
    fn eq(&self, other: &Self) -> bool {
        self.artifact_path == other.artifact_path
            && self.description == other.description
    }
}

/// Consolidates the four evidence-check axes registered against
/// [`FsmTransition`] variants. Wraps [`Evidence::all_passed`] via
/// [`AutoAdvanceEvaluator::evaluate_all`] so the gate's evaluator dispatch
/// stays a single named type.
///
/// Source: phase-4G.txt §4G.1; consolidates the existing
/// [`TreeAutoAdvancePolicy`] + [`Evidence::all_passed`] semantics behind one
/// named struct so callers can program against `AutoAdvanceEvaluator::evaluate`
/// instead of poking individual fields.
#[derive(Debug, Clone)]
pub struct AutoAdvanceEvaluator {
    /// Map of FSM transitions to their evidence-check predicates.
    pub gate_checks: FxHashMap<FsmTransition, EvidenceCheck>,
}

impl Default for AutoAdvanceEvaluator {
    /// Pre-populate one check per [`EscalationTrigger`] axis plus an
    /// `AutoAdvance` catch-all that runs all 6 evidence checks mirroring
    /// gates S13 before auto-advancing.
    fn default() -> Self {
        let mut gate_checks: FxHashMap<FsmTransition, EvidenceCheck> =
            FxHashMap::with_capacity_and_hasher(5, Default::default());
        gate_checks.insert(
            FsmTransition::AntiSpiral,
            EvidenceCheck {
                artifact_path: None,
                description: "anti-spiral detector not firing",
                // AntiSpiral is sourced from the GateContext, not Evidence;
                // the check defers to Pass (the gate's decide() inspects
                // ctx.anti_spiral_fired separately and emits Escalate).
                check_fn: |_ev: &Evidence| EvidenceVerdict::Pass,
            },
        );
        gate_checks.insert(
            FsmTransition::HighRisk,
            EvidenceCheck {
                artifact_path: None,
                description: "risk-level below HIGH",
                check_fn: |_ev: &Evidence| EvidenceVerdict::Pass,
            },
        );
        gate_checks.insert(
            FsmTransition::ReplanMajor,
            EvidenceCheck {
                artifact_path: None,
                description: "no replan-major pending",
                check_fn: |_ev: &Evidence| EvidenceVerdict::Pass,
            },
        );
        gate_checks.insert(
            FsmTransition::CircuitBreaker,
            EvidenceCheck {
                artifact_path: None,
                description: "session circuit breaker not tripped",
                check_fn: |_ev: &Evidence| EvidenceVerdict::Pass,
            },
        );
        gate_checks.insert(
            FsmTransition::AutoAdvance,
            EvidenceCheck {
                artifact_path: None,
                description: "all 6 evidence checks passing",
                check_fn: |ev: &Evidence| {
                    // Run all 6 evidence checks mirroring gates S13
                    let checks = [
                        ev.check_knowledge_readiness(),
                        ev.check_zero_ungrounded(),
                        ev.check_tests_fail_correctly(),
                        ev.check_test_flipped(),
                        ev.check_zero_blocks(),
                        ev.check_dkg_and_changelog_present(),
                    ];
                    // All must pass
                    for check in checks {
                        if !check.is_pass() {
                            return check;
                        }
                    }
                    EvidenceVerdict::Pass
                },
            },
        );
        Self { gate_checks }
    }
}

impl AutoAdvanceEvaluator {
    /// Look up and run the check registered for `transition`. Returns
    /// [`EvidenceVerdict::Missing`] when no entry exists for that transition.
    pub fn evaluate(
        &self,
        transition: &FsmTransition,
        evidence: &Evidence,
    ) -> EvidenceVerdict {
        match self.gate_checks.get(transition) {
            Some(check) => (check.check_fn)(evidence),
            None => EvidenceVerdict::Missing,
        }
    }

    /// Run every registered check against `evidence`. Returns
    /// [`EvidenceVerdict::Pass`] iff EVERY check returns `Pass`. The first
    /// non-`Pass` verdict short-circuits the result. `Missing` verdicts are
    /// ignored (they represent "no opinion"). This mirrors the semantic of
    /// [`Evidence::all_passed`] but routes through the named evaluator type.
    pub fn evaluate_all(&self, evidence: &Evidence) -> EvidenceVerdict {
        for (_t, check) in &self.gate_checks {
            let v = (check.check_fn)(evidence);
            match v {
                EvidenceVerdict::Pass | EvidenceVerdict::Missing => continue,
                EvidenceVerdict::Fail { .. } => return v,
            }
        }
        EvidenceVerdict::Pass
    }
}

// ---------------------------------------------------------------------------
// C-9 — EscalationPolicy + SplitTaskPlan + SubTask + MergeStrategy
// ---------------------------------------------------------------------------

/// Policy knobs governing when the autonomous gate fires
/// [`AutonomyDecision::Escalate`]. Source: phase-4G.txt §4G.1.
///
/// Coexists with the legacy `circuit_breaker_limit` on
/// [`AutonomousGateConfig`] — `max_auto_advances_per_phase` mirrors that field
/// via [`AutonomousGateConfig::default`] so existing callers keep working.
/// C-9 decision (T15.d): EscalationPolicy is the source of truth; the legacy
/// field stays for backward compatibility but new callers should consult
/// `escalation_policy` directly.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EscalationPolicy {
    /// Number of identical failed-hypothesis cycles before the gate emits
    /// [`EscalationTrigger::AntiSpiral`]. The anti-spiral detector itself
    /// reads this when deciding whether to surface a hard-stop signal.
    pub anti_spiral_threshold: u32,
    /// When true, every HIGH-risk transition pauses the runner and emits
    /// [`AutonomyDecision::RequestHuman`]. Defaults to true (spec §4G.4).
    pub high_risk_pauses: bool,
    /// Per-phase auto-advance cap. Mirrors the legacy `circuit_breaker_limit`
    /// on [`AutonomousGateConfig`] and is the source of truth going forward.
    pub max_auto_advances_per_phase: u32,
    /// C-30 (blast-radius gate): when the SurgicalDecomposer estimates more
    /// than this many LOC across the AffectedSet, the autonomous gate fires
    /// [`AutonomyDecision::RequestHuman`] (Interactive) or
    /// [`AutonomyDecision::Escalate`] (Autonomous), depending on
    /// `autonomous_refuse_above_loc`. Mirrors
    /// [`crate::surgical_decomposer::SURGICAL_LOC_WARN_THRESHOLD`].
    pub autonomous_confirm_above_loc: u32,
    /// C-30 (blast-radius gate): when the SurgicalDecomposer estimates more
    /// than this many LOC in Autonomous mode, the gate refuses the plan and
    /// escalates immediately. Mirrors
    /// [`crate::surgical_decomposer::SURGICAL_LOC_ESCALATE_THRESHOLD`].
    pub autonomous_refuse_above_loc: u32,
}

impl Default for EscalationPolicy {
    fn default() -> Self {
        Self {
            anti_spiral_threshold: 3,
            high_risk_pauses: true,
            max_auto_advances_per_phase: 10,
            // C-30: blast-radius defaults mirror surgical_decomposer constants.
            autonomous_confirm_above_loc: 500,
            autonomous_refuse_above_loc: 1000,
        }
    }
}

/// Sub-task carrier for [`AutonomyDecision::SplitTask`]. Source: phase-4G.txt
/// §4G.1.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SubTask {
    /// Stable identifier (parent step id + suffix). Conventional format
    /// `<parent_step_id>.subN` matches the deterministic-fallback path in
    /// `task_runner::emit_autonomous_decision` so SSB consumers see
    /// consistent ids regardless of which code path produced them.
    pub task_id: String,
    /// Free-form description of the sub-task scope.
    pub scope: String,
    /// Per-sub-task context-token budget for the dispatcher. 0 = inherit
    /// from parent.
    pub context_budget: u32,
    /// Sub-task ids this entry depends on (DAG edges). Empty = no edges.
    pub depends_on: Vec<String>,
}

/// Merge strategy for re-assembling the outputs of a [`SplitTaskPlan`].
/// Source: phase-4G.txt §4G.1.
///
/// C-9 ships only `Sequential`. Parallel fan-out is deferred until C-10
/// anti-spiral primitives are stable so the gate has stagnation signals to
/// arbitrate against (see workflow/reasoning/C-9-decisions.md DECISION-B).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MergeStrategy {
    /// Execute sub-tasks one at a time in `sub_tasks` order, halting on the
    /// first failure.
    Sequential,
}

/// Plan carried by [`AutonomyDecision::SplitTask`]. Source: phase-4G.txt
/// §4G.1.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SplitTaskPlan {
    /// Identifier of the parent phase/step being split (matches
    /// [`GateContext::step_id`]).
    pub parent_phase: String,
    /// Ordered sub-tasks.
    pub sub_tasks: Vec<SubTask>,
    /// Merge strategy for re-assembling sub-task outputs.
    pub merge_strategy: MergeStrategy,
    /// Maximum recursion depth (caps unbounded splits).
    pub max_depth: u8,
    /// Current recursion depth (incremented on each successive split).
    pub current_depth: u8,
}

impl SplitTaskPlan {
    /// Build a default plan with `n` deterministic sub-tasks under the
    /// `<parent>.subN` naming convention. Preserves the Batch-3B fallback
    /// id format that the existing SSB `task_split` consumer expects.
    pub fn deterministic(parent_phase: &str, n: u32, depth: u8) -> Self {
        let sub_tasks: Vec<SubTask> = (0..n)
            .map(|i| SubTask {
                task_id: format!("{parent_phase}.sub{i}"),
                scope: format!("auto-split sub-task {} of {}", i + 1, n),
                context_budget: 0,
                depends_on: Vec::new(),
            })
            .collect();
        Self {
            parent_phase: parent_phase.to_owned(),
            sub_tasks,
            merge_strategy: MergeStrategy::Sequential,
            max_depth: 2,
            current_depth: depth,
        }
    }

    /// Convenience accessor — the legacy gate emitted `sub_task_count` and
    /// `depth` as bare fields. C-9's SplitTaskPlan carries richer data but
    /// downstream SSB consumers still want the integer counts.
    pub fn sub_task_count(&self) -> u32 {
        // NARROWING-OK: u32::try_from on Vec::len. Vec capacity practically
        // cannot exceed u32::MAX for plan sub-tasks (max_depth=2, 4 splits
        // per node). Saturate on overflow to preserve forward progress
        // rather than panic.
        u32::try_from(self.sub_tasks.len()).unwrap_or(u32::MAX)
    }

    /// Convenience accessor — current depth.
    pub fn depth(&self) -> u8 {
        self.current_depth
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
    /// the active task per the carried [`SplitTaskPlan`].
    ///
    /// C-9: variant payload replaced from bare `sub_task_count: u32, depth: u8`
    /// to `split_plan: SplitTaskPlan`. The struct exposes `.sub_task_count()`
    /// and `.depth()` accessors so existing SSB row producers stay compatible
    /// — see `task_runner::emit_autonomous_decision`.
    SplitTask {
        split_plan: SplitTaskPlan,
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
#[derive(Clone)]
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
    /// C-9 — optional [`ModelPool`] handle. When `Some`, the gate consults
    /// [`ModelPool::can_fit_llm`] BEFORE emitting
    /// [`AutonomyDecision::SubstituteModel`] so a fallback that won't fit in
    /// available VRAM fires
    /// [`EscalationTrigger::SubstituteModelFailed`] instead. When `None`,
    /// the legacy unconstrained substitution path is preserved (no VRAM
    /// gating). Keeps the gate stateless w.r.t. provider concerns — every
    /// caller supplies their own pool snapshot.
    pub model_pool: Option<Arc<dyn ModelPool>>,
    /// C-10 — per-task attempt history fed into
    /// [`crate::anti_spiral_extended::StagnationDetection::detect_pattern`].
    /// Defaulted to an empty buffer so pre-C-10 callers compile unchanged
    /// (zero-allocation default). The task_runner caller populates this
    /// before each `decide()` invocation.
    pub execution_history: crate::anti_spiral_extended::ExecutionHistory,
    /// C-10 — current attempt index in the per-task loop. Used by SSB
    /// emissions to keyed stagnation rows. Defaulted to 0.
    pub attempt_index: u32,
    /// C-10 — set by the task_runner caller (DRIFT-7 / DECISION-PLACEMENT
    /// in workflow/reasoning/C-10-decisions.md) when
    /// `StagnationDetection::detect_pattern` returns `Some(pattern)`. The
    /// gate reads this on `decide()` to know which persona prefix to thread
    /// into the next baby_step prompt. Defaulted to `None` so existing
    /// contract tests compile unchanged.
    pub suggested_prompt_prefix: Option<&'static str>,
    /// F-6 (C6a) — pre-computed trust tier of the learned skill the runtime is
    /// about to dispatch for this step, or `None` when no learned skill is in
    /// play (the common case — no candidate means no barrier). The task_runner
    /// caller derives this from the candidate [`crate::skill_trust::LearnedSkill`]
    /// BEFORE the [`AutonomousGate::decide`] call so the gate stays decoupled
    /// from the [`crate::skill_trust::SkillStore`]. `decide()` reads it at step
    /// 7.5: a sub-[`SkillTrustTier::S4`] candidate forces
    /// [`AutonomyDecision::RequestHuman`] unless the barrier is disabled via
    /// [`ELAI_SKILL_AUTONOMY_BARRIER_DISABLE_ENV`].
    pub candidate_skill_tier: Option<SkillTrustTier>,
}

impl std::fmt::Debug for GateContext {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("GateContext")
            .field("level", &self.level)
            .field("risk_level", &self.risk_level)
            .field("evidence", &self.evidence)
            .field("anti_spiral_fired", &self.anti_spiral_fired)
            .field("replan_major", &self.replan_major)
            .field("max_restarts_exhausted", &self.max_restarts_exhausted)
            .field("context_budget_terminal", &self.context_budget_terminal)
            .field(
                "auto_advance_count_for_concept",
                &self.auto_advance_count_for_concept,
            )
            .field("step_id", &self.step_id)
            .field("model_pool", &self.model_pool.as_ref().map(|_| "<ModelPool>"))
            .field("execution_history_len", &self.execution_history.len())
            .field("attempt_index", &self.attempt_index)
            .field("suggested_prompt_prefix", &self.suggested_prompt_prefix)
            .field("candidate_skill_tier", &self.candidate_skill_tier)
            .finish()
    }
}

// ---------------------------------------------------------------------------
// AutonomousGateConfig — per-gate configuration
// ---------------------------------------------------------------------------

/// Runtime configuration for [`AutonomousGate`]. Defaults per spec §4G.1:
/// Interactive mode, circuit breaker limit 10 per session, default tree
/// policy, two-model fallback chain (7B → 3B).
///
/// C-9: gains `escalation_policy: EscalationPolicy` (source of truth going
/// forward) and `estimated_model_sizes: FxHashMap<String, u64>` (consulted by
/// the SUBSTITUTE_MODEL guard via [`ModelPool::can_fit_llm`]).
#[derive(Debug, Clone)]
pub struct AutonomousGateConfig {
    pub mode: GateMode,
    /// Legacy circuit breaker count (kept for backward compatibility).
    /// `escalation_policy.max_auto_advances_per_phase` is the canonical
    /// source of truth — `Default::default()` keeps both in sync at 10.
    pub circuit_breaker_limit: u32,
    pub tree_auto_advance_policy: TreeAutoAdvancePolicy,
    pub model_fallback_chain: Vec<String>,
    /// C-9 — escalation policy knobs. New callers consult this field
    /// directly; the legacy `circuit_breaker_limit` mirrors
    /// `escalation_policy.max_auto_advances_per_phase` for backward
    /// compatibility (see C-9-decisions.md DECISION-D).
    pub escalation_policy: EscalationPolicy,
    /// C-9 — estimated model size in bytes for each entry in
    /// `model_fallback_chain`. Consulted by the SUBSTITUTE_MODEL guard via
    /// [`ModelPool::can_fit_llm`] BEFORE emitting `AutonomyDecision::
    /// SubstituteModel`. When a target model has no entry here, the guard
    /// defaults to "fits" so the legacy unconstrained behaviour is preserved
    /// for callers that have not opted in (Default::default() keeps the map
    /// empty).
    pub estimated_model_sizes: FxHashMap<String, u64>,
}

impl Default for AutonomousGateConfig {
    fn default() -> Self {
        let policy = EscalationPolicy::default();
        Self {
            mode: GateMode::Interactive,
            circuit_breaker_limit: policy.max_auto_advances_per_phase,
            tree_auto_advance_policy: TreeAutoAdvancePolicy::default(),
            model_fallback_chain: vec![
                "qwen2.5-coder-7b".to_owned(),
                "qwen2.5-coder-3b".to_owned(),
            ],
            escalation_policy: policy,
            estimated_model_sizes: FxHashMap::default(),
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
            // C-9: variant now carries a SplitTaskPlan. Produce a
            // deterministic 4-way plan at depth=2; richer plans are built by
            // ResearchTree::split_step at the task_runner dispatch site.
            return AutonomyDecision::SplitTask {
                split_plan: SplitTaskPlan::deterministic(&ctx.step_id, 4, 2),
            };
        }

        // 4. max_restarts_exhausted → SubstituteModel.
        if mode == GateMode::Autonomous && ctx.max_restarts_exhausted {
            if let Some(next) = self.next_fallback_model().await {
                let from = {
                    let m = self.active_model.read().await;
                    m.clone()
                };
                // C-9: VRAM-guard via ModelPool::can_fit_llm. When the
                // caller supplied a model_pool and the target size is known
                // AND the pool reports the target won't fit, fire
                // SubstituteModelFailed instead of attempting the swap.
                let estimated_size = {
                    let cfg = self.config.read().await;
                    cfg.estimated_model_sizes.get(&next).copied()
                };
                if let (Some(pool), Some(size)) = (ctx.model_pool.as_ref(), estimated_size) {
                    if !pool.can_fit_llm(size) {
                        return AutonomyDecision::Escalate {
                            trigger: EscalationTrigger::SubstituteModelFailed,
                        };
                    }
                }
                return AutonomyDecision::SubstituteModel { from, to: next };
            }
            // No fallback left — fire SubstituteModelFailed (C-9 expanded
            // taxonomy; previously fell back to HighRisk).
            return AutonomyDecision::Escalate {
                trigger: EscalationTrigger::SubstituteModelFailed,
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

        // 7.5. F-6 (C6a) learned-skill autonomy barrier. When the runtime is
        // about to dispatch a learned skill, only an apex S4 skill is cleared
        // for autonomous dispatch — a sub-S4 candidate routes to a human.
        // `candidate_skill_tier == None` means no learned skill is in play, so
        // the barrier never fires (the common path). The barrier honours the
        // default-ON kill switch ELAI_SKILL_AUTONOMY_BARRIER_DISABLE.
        if let Some(tier) = ctx.candidate_skill_tier {
            if !skill_permits_autonomous(tier) && !skill_autonomy_barrier_disabled() {
                return AutonomyDecision::RequestHuman {
                    reason: format!(
                        "learned-skill autonomy barrier: candidate skill at tier {} \
                         is sub-S4, human grant required for autonomous dispatch",
                        tier.as_str()
                    ),
                    risk_level: ctx.risk_level,
                    evidence: ctx.evidence.clone(),
                };
            }
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
            // B-40: 7th check vacuous-default true for non-FRAMING tests.
            framing_converged: true,
            // B-41: 5 per-level audit zero-blocks vacuous-default true.
            concept_zero_blocks: true,
            phase_zero_blocks: true,
            step_zero_blocks: true,
            babystep_zero_blocks: true,
            plan_zero_blocks: true,
            // F-10: vacuous-default true, matching Evidence::default. The gate
            // helper is not in a security-monotonicity context, so true = no block.
            security_monotonicity: true,
            // SP5 wiring-fix: the off-agent EXECUTING -> AUDITING legs must be
            // explicitly true for an all-pass fixture (they default FALSE).
            layer3_entrypoint_covered: true,
            layer4_trace_present: true,
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
            // C-9: ModelPool is opt-in. Tests that need VRAM gating supply
            // their own; the default keeps the legacy unconstrained path.
            model_pool: None,
            // C-10: anti-spiral extended fields default to empty/None for
            // pre-C-10 in-module tests.
            execution_history: crate::anti_spiral_extended::ExecutionHistory::default(),
            attempt_index: 0,
            suggested_prompt_prefix: None,
            // F-6 (C6a) — no candidate skill in the base test fixture. The
            // step-7.5 barrier is exercised by the dedicated tests below that
            // override this field: skill_barrier_sub_s4_candidate_requests_human,
            // skill_barrier_apex_s4_candidate_auto_advances,
            // skill_barrier_none_candidate_passes_step_7_5, and
            // skill_barrier_kill_switch_overrides_sub_s4.
            candidate_skill_tier: None,
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
        // C-9: AutonomyDecision::SplitTask now carries SplitTaskPlan; verify
        // both the carrier shape and the legacy count/depth accessors.
        match decision {
            AutonomyDecision::SplitTask { split_plan } => {
                assert_eq!(split_plan.sub_task_count(), 4);
                assert_eq!(split_plan.depth(), 2);
                assert_eq!(split_plan.merge_strategy, MergeStrategy::Sequential);
                assert_eq!(split_plan.parent_phase, "step-1");
                assert_eq!(split_plan.sub_tasks.len(), 4);
                assert!(split_plan.sub_tasks[0].task_id.starts_with("step-1.sub"));
            }
            other => panic!("expected SplitTask, got {other:?}"),
        }
    }

    // -----------------------------------------------------------------------
    // F-6 (C6a) — learned-skill autonomy barrier (step 7.5). These are the
    // dedicated tests the base-fixture comment in `ctx_with` points to: they
    // are the ONLY tests that set `ctx.candidate_skill_tier = Some(_)` and so
    // are the only coverage of the step-7.5 RequestHuman branch.
    // -----------------------------------------------------------------------

    /// RAII env-gate guard for the kill-switch test. Restores the prior value
    /// on drop. Mirrors the `EnvGuard` precedent in `compaction_ladder.rs`.
    struct BarrierEnvGuard {
        prior: Option<String>,
    }

    impl BarrierEnvGuard {
        fn set(value: &str) -> Self {
            let prior = std::env::var(ELAI_SKILL_AUTONOMY_BARRIER_DISABLE_ENV).ok();
            // SAFETY: env-gate flips in this test module are single-threaded
            // per-process; matches the codebase EnvGuard precedent.
            unsafe {
                std::env::set_var(ELAI_SKILL_AUTONOMY_BARRIER_DISABLE_ENV, value);
            }
            Self { prior }
        }
    }

    impl Drop for BarrierEnvGuard {
        fn drop(&mut self) {
            match &self.prior {
                Some(v) => unsafe {
                    std::env::set_var(ELAI_SKILL_AUTONOMY_BARRIER_DISABLE_ENV, v);
                },
                None => unsafe {
                    std::env::remove_var(ELAI_SKILL_AUTONOMY_BARRIER_DISABLE_ENV);
                },
            }
        }
    }

    fn autonomous_gate_for_barrier() -> AutonomousGate {
        AutonomousGate::new(
            AutonomousGateConfig {
                mode: GateMode::Autonomous,
                ..AutonomousGateConfig::default()
            },
            None,
        )
    }

    #[tokio::test]
    async fn skill_barrier_sub_s4_candidate_requests_human() {
        // A sub-S4 learned skill in play at an otherwise-auto-advancing step
        // must route to a human (the F-6 observable_effect).
        let gate = autonomous_gate_for_barrier();
        let mut ctx = ctx_with(PipelineLevel::BabyStep, RiskLevel::Low);
        ctx.candidate_skill_tier = Some(SkillTrustTier::S3);
        let decision = gate.decide(&ctx).await;
        match decision {
            AutonomyDecision::RequestHuman { reason, .. } => {
                assert!(
                    reason.contains("learned-skill autonomy barrier"),
                    "expected the barrier reason, got: {reason}"
                );
            }
            other => panic!("expected RequestHuman from the sub-S4 barrier, got {other:?}"),
        }
    }

    #[tokio::test]
    async fn skill_barrier_apex_s4_candidate_auto_advances() {
        // An apex S4 (human-granted) skill is cleared for autonomous dispatch:
        // step 7.5 does not fire, the happy path auto-advances.
        let gate = autonomous_gate_for_barrier();
        let mut ctx = ctx_with(PipelineLevel::BabyStep, RiskLevel::Low);
        ctx.candidate_skill_tier = Some(SkillTrustTier::S4);
        let decision = gate.decide(&ctx).await;
        assert_eq!(decision, AutonomyDecision::AutoAdvance);
    }

    #[tokio::test]
    async fn skill_barrier_none_candidate_passes_step_7_5() {
        // `None` = no learned skill in play; the barrier is dormant and the
        // step auto-advances exactly as it did pre-F-6.
        let gate = autonomous_gate_for_barrier();
        let mut ctx = ctx_with(PipelineLevel::BabyStep, RiskLevel::Low);
        ctx.candidate_skill_tier = None;
        let decision = gate.decide(&ctx).await;
        assert_eq!(decision, AutonomyDecision::AutoAdvance);
    }

    #[tokio::test]
    async fn skill_barrier_kill_switch_overrides_sub_s4() {
        // The default-ON barrier is disabled by
        // ELAI_SKILL_AUTONOMY_BARRIER_DISABLE: a sub-S4 candidate then
        // auto-advances (pre-F-6 behaviour) instead of routing to a human.
        let _g = BarrierEnvGuard::set("1");
        let gate = autonomous_gate_for_barrier();
        let mut ctx = ctx_with(PipelineLevel::BabyStep, RiskLevel::Low);
        ctx.candidate_skill_tier = Some(SkillTrustTier::S3);
        let decision = gate.decide(&ctx).await;
        assert_eq!(
            decision,
            AutonomyDecision::AutoAdvance,
            "kill switch must disable the sub-S4 barrier"
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
        // B-40: Evidence::default() yields framing_converged=true (vacuous
        // default for non-FRAMING contexts); the summary tail is `fc=1`.
        let mut e = Evidence::default();
        e.knowledge_readiness = true;
        assert_eq!(e.summary(), "kr=1 zu=0 tfc=0 tf=0 zb=0 dcg=0 fc=1");
    }

    #[test]
    fn pipeline_level_risk_defaults() {
        assert_eq!(PipelineLevel::Concept.default_risk(), RiskLevel::High);
        assert_eq!(PipelineLevel::Phase.default_risk(), RiskLevel::Medium);
        assert_eq!(PipelineLevel::Step.default_risk(), RiskLevel::Low);
        assert_eq!(PipelineLevel::BabyStep.default_risk(), RiskLevel::Low);
    }

    // -----------------------------------------------------------------
    // C-9 — AutoAdvanceEvaluator + EscalationPolicy + SubstituteModelFailed
    // -----------------------------------------------------------------

    #[test]
    fn c9_escalation_policy_defaults() {
        let p = EscalationPolicy::default();
        assert_eq!(p.anti_spiral_threshold, 3);
        assert!(p.high_risk_pauses);
        assert_eq!(p.max_auto_advances_per_phase, 10);
    }

    #[test]
    fn c9_auto_advance_evaluator_defaults_populated() {
        let ev = AutoAdvanceEvaluator::default();
        assert_eq!(ev.gate_checks.len(), 5);
        assert!(ev.gate_checks.contains_key(&FsmTransition::AutoAdvance));
        assert!(ev.gate_checks.contains_key(&FsmTransition::AntiSpiral));
        assert!(ev.gate_checks.contains_key(&FsmTransition::HighRisk));
        assert!(ev.gate_checks.contains_key(&FsmTransition::ReplanMajor));
        assert!(ev.gate_checks.contains_key(&FsmTransition::CircuitBreaker));
    }

    #[test]
    fn c9_evaluator_evaluate_pass_when_evidence_passes() {
        let ev = AutoAdvanceEvaluator::default();
        let e = all_pass_evidence();
        let v = ev.evaluate(&FsmTransition::AutoAdvance, &e);
        assert_eq!(v, EvidenceVerdict::Pass);
        assert!(v.is_pass());
    }

    #[test]
    fn c9_evaluator_evaluate_fail_when_evidence_fails() {
        let ev = AutoAdvanceEvaluator::default();
        let mut e = all_pass_evidence();
        e.zero_ungrounded = false;
        let v = ev.evaluate(&FsmTransition::AutoAdvance, &e);
        match v {
            EvidenceVerdict::Fail { reason } => {
                assert!(reason.contains("zero_ungrounded"));
            }
            other => panic!("expected Fail, got {other:?}"),
        }
    }

    #[test]
    fn c9_evaluator_evaluate_all_pass_when_evidence_passes() {
        let ev = AutoAdvanceEvaluator::default();
        let e = all_pass_evidence();
        assert!(ev.evaluate_all(&e).is_pass());
    }

    #[test]
    fn c9_evaluator_evaluate_all_fail_when_evidence_fails() {
        let ev = AutoAdvanceEvaluator::default();
        let mut e = all_pass_evidence();
        e.knowledge_readiness = false;
        let v = ev.evaluate_all(&e);
        assert!(!v.is_pass());
    }

    #[test]
    fn c9_evidence_verdict_as_str_labels() {
        assert_eq!(EvidenceVerdict::Pass.as_str(), "pass");
        assert_eq!(
            EvidenceVerdict::Fail {
                reason: "x".to_owned()
            }
            .as_str(),
            "fail"
        );
        assert_eq!(EvidenceVerdict::Missing.as_str(), "missing");
    }

    #[test]
    fn c9_escalation_trigger_new_variants_as_str() {
        assert_eq!(
            EscalationTrigger::SubstituteModelFailed.as_str(),
            "substitute_model_failed"
        );
        assert_eq!(
            EscalationTrigger::SplitTaskFailed.as_str(),
            "split_task_failed"
        );
    }

    #[test]
    fn c9_split_task_plan_deterministic_shape() {
        let plan = SplitTaskPlan::deterministic("step-1", 4, 2);
        assert_eq!(plan.parent_phase, "step-1");
        assert_eq!(plan.sub_task_count(), 4);
        assert_eq!(plan.depth(), 2);
        assert_eq!(plan.merge_strategy, MergeStrategy::Sequential);
        assert_eq!(plan.sub_tasks.len(), 4);
        for (i, sub) in plan.sub_tasks.iter().enumerate() {
            assert_eq!(sub.task_id, format!("step-1.sub{i}"));
            assert!(sub.depends_on.is_empty());
        }
    }

    #[test]
    fn c9_gate_mode_from_cli_args() {
        assert_eq!(GateMode::from_cli_args(None), GateMode::Interactive);
        assert_eq!(
            GateMode::from_cli_args(Some("autonomous")),
            GateMode::Autonomous
        );
        assert_eq!(
            GateMode::from_cli_args(Some("interactive")),
            GateMode::Interactive
        );
        // hybrid resolves to Interactive at the gate tier (hybrid is enforced
        // at the dispatcher).
        assert_eq!(
            GateMode::from_cli_args(Some("hybrid")),
            GateMode::Interactive
        );
        // Unknown labels fall back to default.
        assert_eq!(
            GateMode::from_cli_args(Some("rogue")),
            GateMode::Interactive
        );
    }

    struct FixedVramPool(u64);
    impl crate::agent_pool::ModelPool for FixedVramPool {
        fn available_vram(&self) -> u64 {
            self.0
        }
    }

    #[tokio::test]
    async fn c9_substitute_model_failed_when_no_fallback() {
        let gate = AutonomousGate::new(
            AutonomousGateConfig {
                mode: GateMode::Autonomous,
                model_fallback_chain: vec!["only-one".into()],
                ..AutonomousGateConfig::default()
            },
            None,
        );
        let mut ctx = ctx_with(PipelineLevel::Step, RiskLevel::Low);
        ctx.max_restarts_exhausted = true;
        let d = gate.decide(&ctx).await;
        assert_eq!(
            d,
            AutonomyDecision::Escalate {
                trigger: EscalationTrigger::SubstituteModelFailed,
            }
        );
    }

    #[tokio::test]
    async fn c9_substitute_model_failed_when_vram_insufficient() {
        let mut sizes = FxHashMap::default();
        sizes.insert("big".to_owned(), 8 * 1024 * 1024 * 1024); // 8 GiB
        let gate = AutonomousGate::new(
            AutonomousGateConfig {
                mode: GateMode::Autonomous,
                model_fallback_chain: vec!["small".into(), "big".into()],
                estimated_model_sizes: sizes,
                ..AutonomousGateConfig::default()
            },
            None,
        );
        let mut ctx = ctx_with(PipelineLevel::Step, RiskLevel::Low);
        ctx.max_restarts_exhausted = true;
        // Pool reports 2 GiB — `big` (8 GiB) does not fit.
        ctx.model_pool = Some(Arc::new(FixedVramPool(2 * 1024 * 1024 * 1024)));
        let d = gate.decide(&ctx).await;
        assert_eq!(
            d,
            AutonomyDecision::Escalate {
                trigger: EscalationTrigger::SubstituteModelFailed,
            }
        );
    }

    #[tokio::test]
    async fn c9_substitute_model_proceeds_when_vram_sufficient() {
        let mut sizes = FxHashMap::default();
        sizes.insert("small".to_owned(), 1 * 1024 * 1024 * 1024); // 1 GiB
        sizes.insert("smaller".to_owned(), 512 * 1024 * 1024);
        let gate = AutonomousGate::new(
            AutonomousGateConfig {
                mode: GateMode::Autonomous,
                model_fallback_chain: vec!["small".into(), "smaller".into()],
                estimated_model_sizes: sizes,
                ..AutonomousGateConfig::default()
            },
            None,
        );
        let mut ctx = ctx_with(PipelineLevel::Step, RiskLevel::Low);
        ctx.max_restarts_exhausted = true;
        ctx.model_pool = Some(Arc::new(FixedVramPool(4 * 1024 * 1024 * 1024)));
        let d = gate.decide(&ctx).await;
        match d {
            AutonomyDecision::SubstituteModel { from, to } => {
                assert_eq!(from, "small");
                assert_eq!(to, "smaller");
            }
            other => panic!("expected SubstituteModel, got {other:?}"),
        }
    }
}
