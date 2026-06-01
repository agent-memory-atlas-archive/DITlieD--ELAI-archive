//! FSM engine: 10-state machine with validated transitions, msgpack persistence,
//! concurrent safety, ReplanRequest support, and risk-level classification.
//!
//! Phase 3A.1 + 3A.5 deliverable. See docs/specs/crates/orchestrator.txt §2, §13
//! and docs/specs/architecture-overview.txt §4, §6.

use serde::{Deserialize, Serialize};
use rustc_hash::FxHashMap;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, OnceLock};
use std::time::UNIX_EPOCH;
use tokio::sync::Mutex;

use elai_core::{ClockProvider as _, OsClock};

// Re-export RiskLevel from gate module so consumers can access it via fsm::.
pub use crate::gate::{default_risk_level, RiskLevel};

// ---------------------------------------------------------------------------
// FsmState enum (9 states)
// ---------------------------------------------------------------------------

/// All 10 FSM states. Source: orchestrator.txt §2, architecture-overview.txt §4,
/// pipeline-modes-design.txt §2 (RESEARCHING).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum FsmState {
    Idle,
    Planning,
    FeasibilityProbe,
    TestingFirst,
    Executing,
    Auditing,
    Memorizing,
    Complete,
    Configuring,
    /// Research-only state for Oracle Mode. Allows WebSearch + KnowledgeEngine +
    /// GRC reasoning. Blocks file edit, bash, git, all mutation tools.
    /// Source: pipeline-modes-design.txt §2 (FSM PATH), INTEGRATION §2.
    Researching,
}

impl FsmState {
    /// All 10 states for iteration.
    pub const ALL: [FsmState; 10] = [
        FsmState::Idle,
        FsmState::Planning,
        FsmState::FeasibilityProbe,
        FsmState::TestingFirst,
        FsmState::Executing,
        FsmState::Auditing,
        FsmState::Memorizing,
        FsmState::Complete,
        FsmState::Configuring,
        FsmState::Researching,
    ];

    /// Valid target states from `self`. Source: phase-3A.txt transition matrix,
    /// pipeline-modes-design.txt §2 + INTEGRATION §2 (RESEARCHING transitions).
    pub fn valid_targets(self) -> &'static [FsmState] {
        match self {
            FsmState::Idle => &[FsmState::Planning, FsmState::Configuring, FsmState::Researching],
            FsmState::Planning => &[FsmState::FeasibilityProbe, FsmState::TestingFirst],
            FsmState::FeasibilityProbe => &[FsmState::Planning, FsmState::TestingFirst],
            FsmState::TestingFirst => &[FsmState::Executing],
            FsmState::Executing => &[FsmState::Auditing, FsmState::Planning],
            FsmState::Auditing => &[
                FsmState::Memorizing,
                FsmState::Executing,
                FsmState::Planning,
            ],
            FsmState::Memorizing => &[FsmState::Planning, FsmState::Complete],
            FsmState::Complete => &[FsmState::Idle],
            FsmState::Configuring => &[FsmState::Idle],
            // RESEARCHING: follow-up queries (self-loop), exit to IDLE, or
            // escalate to PLANNING if research reveals need for code changes.
            // Source: pipeline-modes-design.txt §2 L327-338, open question 5 L1354.
            FsmState::Researching => &[FsmState::Researching, FsmState::Idle, FsmState::Planning],
        }
    }

    /// Whether `target` is a valid next state from `self`.
    pub fn can_transition_to(self, target: FsmState) -> bool {
        self.valid_targets().contains(&target)
    }
}

impl std::fmt::Display for FsmState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let name = match self {
            FsmState::Idle => "IDLE",
            FsmState::Planning => "PLANNING",
            FsmState::FeasibilityProbe => "FEASIBILITY_PROBE",
            FsmState::TestingFirst => "TESTING_FIRST",
            FsmState::Executing => "EXECUTING",
            FsmState::Auditing => "AUDITING",
            FsmState::Memorizing => "MEMORIZING",
            FsmState::Complete => "COMPLETE",
            FsmState::Configuring => "CONFIGURING",
            FsmState::Researching => "RESEARCHING",
        };
        f.write_str(name)
    }
}

// ---------------------------------------------------------------------------
// FsmSubstate (B-40): per-state agent substate
// ---------------------------------------------------------------------------
//
// Agent-scoped substate distinct from top-level FsmState. FRAMING is the only
// variant at MVP (B-40 P2); future substates (Decomposing under PLANNING,
// Validating under EXECUTING, etc.) follow the same pattern. This mirrors the
// AUDITING + MEMORIZING agent_substate pattern used by mcp_fsm_agent_transition
// (CLAUDE.md FSM cleanup rule).
//
// Substates are NOT validated by `valid_targets()` — they're orthogonal to the
// top-level transition matrix. PLANNING + FRAMING means "agent is in the
// Idea-Framer phase under the PLANNING top-level state". The PreToolUse hook
// gates tool access on (state, substate) tuples.

/// Per-state agent substate. Used by B-40 to mark FRAMING within PLANNING.
/// Persisted via [`PersistedFsmState::substate`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FsmSubstate {
    /// B-40: Idea-Framer + critic-loop pass under PLANNING. Restricts tools
    /// to research / read-only (no Edit/Write/Bash beyond research scratch).
    /// Exited when run_critic_loop converges (zero BLOCK findings within
    /// MAX_CRITIC_ROUNDS=3) AND the user has submitted tradeoff responses.
    Framing,
    /// E-1: Memory-evolution / "sleep-time compute" pass under IDLE
    /// (arXiv:2504.13171). The orchestrator transitions IDLE -> SLEEP_COMPUTE
    /// when the user has been idle for >= 2s (default debounce) AND a
    /// chat-reflect window is due. While in this substate the digest-writer
    /// agent runs the [`elai_reasoning::chat_memory_reflect::reflect`] pass
    /// (note insert + CoreBlock append + decay tick). The substate falls
    /// through within 50ms on `user_input_committed`, and the orchestrator
    /// transitions SLEEP_COMPUTE -> IDLE (top-level state unchanged at
    /// IDLE — substate is the only marker).
    SleepCompute,
    /// E-13: TDD verifier-agent pass under EXECUTING / AUDITING.
    /// `Tdd` marks an active Red->Green->Refactor cycle driven by
    /// [`crate::tdd_verifier::TddVerifierAgent`]. While in this
    /// substate the agent emits `TddCycleStarted`, `TddTestFailed`,
    /// and `TddTestPassed` SSB beacons; the auditor consumes the
    /// cycle result via D-3's `ConsensusRule::
    /// VerifierKindHeterogeneous { required_kinds:
    /// vec![VerifierKind::Deterministic] }`. PreToolUse hooks gate
    /// tool access on (state, substate) — under Tdd the agent is
    /// restricted to test-file writes + cargo-mutants invocation.
    Tdd,
    /// E-7: Darwinian self-improvement generation evaluation substate
    /// under top-level `FsmState::Auditing`. Named
    /// `DarwinianGenerationEvaluating` (NOT `Promoting`, NOT plain
    /// `Auditing`) to avoid collision with the top-level
    /// `FsmState::Auditing` — per workflow/reasoning/E-7-decisions.md
    /// DECISION-8. Wire-dark unless `ELAI_DARWINIAN_LOOP_ENABLE=1` AND
    /// `cfg(feature = "darwinian-evolve")`. Paper grounding:
    /// arXiv:2510.04399 §Theorem 2 Two-Gate finite-sample safety.
    DarwinianGenerationEvaluating,
    /// E-6: Adversary substate — cross-family heterogeneous-verifier
    /// probe window under AUDITING. Entered when the primary auditor
    /// emits a verdict that requires cross-family confirmation per
    /// [`crate::phase_kind::PhaseKind::requires_cross_family_adversary`].
    /// Exited when [`crate::adversary::AdversaryVerdict::cross_family_confirmed`]
    /// is true (proceeds to MEMORIZING) or when cross-family is unavailable
    /// AND `ELAI_ALLOW_CLOUD_ADVERSARY=0` (emits
    /// `SsbActionType::CrossFamilyUnavailable` + refuses AUDITING ->
    /// MEMORIZING). PreToolUse hooks restrict the agent to verifier
    /// invocation only — no Edit/Write/Bash. Grounding: arXiv:2507.02778
    /// (64.5% blind-spot rate) + arXiv:2503.13657 MAST FC3.
    Adversary,
}

impl FsmSubstate {
    pub fn as_str(self) -> &'static str {
        match self {
            FsmSubstate::Framing => "framing",
            FsmSubstate::SleepCompute => "sleep_compute",
            FsmSubstate::Tdd => "tdd",
            // E-7: darwinian self-improvement substrate -- DECISION-8.
            FsmSubstate::DarwinianGenerationEvaluating => "darwinian_generation_evaluating",
            // E-6: cross-family adversary probe substate.
            FsmSubstate::Adversary => "adversary",
        }
    }
}

impl std::fmt::Display for FsmSubstate {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

// ---------------------------------------------------------------------------
// E-6 — audit_clear_for_memorize guard
// ---------------------------------------------------------------------------

/// Error returned by [`audit_clear_for_memorize`] when the verdict does
/// NOT clear AUDITING for transition to MEMORIZING.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum AdversaryGateError {
    /// The phase kind requires cross-family adversary confirmation but
    /// no AdversaryVerdict was attached.
    #[error("phase {phase} requires cross-family adversary but verdict was not supplied")]
    AdversaryMissing { phase: String },
    /// AdversaryVerdict supplied but `cross_family_confirmed` is false.
    #[error(
        "phase {phase} requires cross-family confirmation but verdict reports cross_family_confirmed=false"
    )]
    NotCrossFamilyConfirmed { phase: String },
}

/// E-6 — verify AUDITING -> MEMORIZING is cleared per cross-family
/// adversary requirements. See workflow/reasoning/E-6-decisions.md
/// DECISION-7 for the placement rationale (plan task 9 originally
/// targeted AUDITING -> COMPLETE which is impossible under the
/// transition matrix — the actual choke point is AUDITING -> MEMORIZING).
///
/// Returns `Ok(())` when phase does not require adversary OR a verdict
/// with `cross_family_confirmed=true` is supplied. `Err` otherwise;
/// caller should emit `CrossFamilyUnavailable` SSB row before refusing.
pub fn audit_clear_for_memorize(
    phase: &crate::phase_kind::PhaseKind,
    adversary_verdict: Option<&crate::adversary::AdversaryVerdict>,
) -> Result<(), AdversaryGateError> {
    if !phase.requires_cross_family_adversary() {
        return Ok(());
    }
    let verdict = match adversary_verdict {
        Some(v) => v,
        None => {
            return Err(AdversaryGateError::AdversaryMissing {
                phase: phase.as_str().to_owned(),
            });
        }
    };
    if !verdict.cross_family_confirmed {
        return Err(AdversaryGateError::NotCrossFamilyConfirmed {
            phase: phase.as_str().to_owned(),
        });
    }
    Ok(())
}

#[cfg(test)]
mod e6_audit_clear_tests {
    use super::*;
    use crate::adversary::AdversaryVerdict;
    use crate::phase_kind::PhaseKind;

    #[test]
    fn docs_phase_never_requires_adversary() {
        audit_clear_for_memorize(&PhaseKind::Docs, None).expect("docs must clear");
        audit_clear_for_memorize(&PhaseKind::Test, None).expect("test must clear");
        audit_clear_for_memorize(&PhaseKind::Other, None).expect("other must clear");
    }

    #[test]
    fn code_phase_without_verdict_blocks() {
        let err = audit_clear_for_memorize(&PhaseKind::Code, None)
            .expect_err("code phase MUST require verdict");
        assert!(matches!(err, AdversaryGateError::AdversaryMissing { .. }));
    }

    #[test]
    fn architecture_phase_with_unconfirmed_verdict_blocks() {
        let v = AdversaryVerdict::robust("c-1", false);
        let err = audit_clear_for_memorize(&PhaseKind::Architecture, Some(&v))
            .expect_err("architecture + unconfirmed must block");
        assert!(matches!(
            err,
            AdversaryGateError::NotCrossFamilyConfirmed { .. }
        ));
    }

    #[test]
    fn security_phase_with_confirmed_verdict_clears() {
        let v = AdversaryVerdict::robust("c-1", true);
        audit_clear_for_memorize(&PhaseKind::Security, Some(&v))
            .expect("security + confirmed must clear");
    }

    #[test]
    fn fsm_substate_adversary_as_str_stable() {
        assert_eq!(FsmSubstate::Adversary.as_str(), "adversary");
    }
}

// ---------------------------------------------------------------------------
// FsmConfig — per-project FSM configuration (risk overrides)
// ---------------------------------------------------------------------------

/// FSM configuration, including per-transition risk overrides.
/// Loaded from elai_cc.toml [fsm] section. All fields optional with sensible defaults.
///
/// Key format for risk_overrides: "SOURCE->TARGET" (e.g., "PLANNING->TESTING_FIRST").
/// UNGROUNDED: inferred from architecture-overview.txt §9 L295 "All routing
/// configurable in elai_cc.toml".
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct FsmConfig {
    /// Per-transition risk overrides. Key: "SOURCE->TARGET", Value: RiskLevel.
    #[serde(default)]
    pub risk_overrides: FxHashMap<String, RiskLevel>,
}

/// Build the canonical transition key for risk override lookup.
fn transition_key(source: FsmState, target: FsmState) -> String {
    format!("{}->{}", source, target)
}

/// Resolve the effective risk level for a transition, consulting config overrides first.
/// Falls back to `default_risk_level` if no override is configured.
pub fn resolve_risk_level(config: &FsmConfig, source: FsmState, target: FsmState) -> RiskLevel {
    let key = transition_key(source, target);
    config
        .risk_overrides
        .get(&key)
        .copied()
        .unwrap_or_else(|| RiskLevel::from(default_risk_level(source, target)))
}

/// Convenience function: get the default risk level for a transition without config.
/// Re-exported from gate module. Uses `RiskLevel::Low`, `RiskLevel::Medium`,
/// `RiskLevel::High` classification from architecture-overview.txt §6 L227-230.
pub fn risk_level_for_transition(source: FsmState, target: FsmState) -> RiskLevel {
    default_risk_level(source, target)
}

// ---------------------------------------------------------------------------
// Evidence artifacts
// ---------------------------------------------------------------------------

/// Artifact attached to a transition as evidence. Source: phase-3A.txt line 65.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvidenceArtifact {
    pub kind: String,
    pub summary: String,
    pub data: Vec<u8>,
}

// ---------------------------------------------------------------------------
// Transition record
// ---------------------------------------------------------------------------

/// Recorded transition with full provenance. Source: phase-3A.txt line 63-65.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransitionRecord {
    pub source: FsmState,
    pub target: FsmState,
    pub phase_label: String,
    pub timestamp_ms: u64,
    pub evidence: Vec<EvidenceArtifact>,
}

// ---------------------------------------------------------------------------
// ReplanRequest
// ---------------------------------------------------------------------------

/// Severity of a replan request. Source: orchestrator.txt §13 L506-513.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ReplanSeverity {
    /// Auditor-validated fast-track, no human gate required.
    Minor,
    /// Human approval required before replanning.
    Major,
}

/// A replan request from an executor agent. Max 1 per agent per phase.
/// Source: orchestrator.txt §13 L506-513, phase-3A.txt line 71.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReplanRequest {
    pub severity: ReplanSeverity,
    pub evidence: String,
    pub agent_id: String,
    pub phase_label: String,
}

// ---------------------------------------------------------------------------
// Errors
// ---------------------------------------------------------------------------

/// FSM error types. Source: phase-3A.txt adversarial section.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FsmError {
    /// Attempted transition between states that are not connected.
    InvalidTransition {
        source: FsmState,
        target: FsmState,
    },
    /// State file on disk is corrupted or unreadable.
    CorruptState {
        path: PathBuf,
        detail: String,
    },
    /// Second ReplanRequest in the same phase by the same agent.
    ReplanLimitExceeded {
        agent_id: String,
        phase_label: String,
    },
    /// Another transition is already in progress (concurrent access).
    TransitionConflict,
    /// I/O error during state persistence.
    PersistenceError(String),
    /// Tool is not allowed in the current FSM state.
    /// Phase 3A.2: enforced at ToolRegistry::execute() level.
    ToolBlockedByState {
        tool_name: String,
        state: FsmState,
        category: crate::tool_filter::ToolCategory,
    },
    /// Session wire-a12: the `AUDITING -> MEMORIZING` transition is rejected
    /// because the cached [`crate::deterministic_audit_gate::DeterministicAuditGate`]
    /// result from the most recent `_ -> Auditing` evaluation carries at least
    /// one BLOCK-severity finding. The agent must resolve the BLOCK findings
    /// and re-enter AUDITING before MEMORIZING can be reached.
    AuditingBlocked {
        block_count: usize,
    },
    /// E-38 (SCHEMA_VERSION 175): the
    /// `PLANNING|FEASIBILITY_PROBE -> TESTING_FIRST` transition was
    /// refused by the EU AI Act compliance pre-gate. The agent must
    /// fix the cited assertion before re-attempting the transition.
    ComplianceBlocked {
        /// Snake_case assertion label (e.g. `clock_provider_intact`).
        assertion_label: String,
        /// Static remediation hint.
        hint: String,
    },
}

impl std::fmt::Display for FsmError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            FsmError::InvalidTransition { source, target } => {
                write!(f, "invalid transition: {source} -> {target}")
            }
            FsmError::CorruptState { path, detail } => {
                write!(f, "corrupt state at {}: {detail}", path.display())
            }
            FsmError::ReplanLimitExceeded {
                agent_id,
                phase_label,
            } => {
                write!(
                    f,
                    "replan limit exceeded: agent {agent_id} in phase {phase_label}"
                )
            }
            FsmError::TransitionConflict => {
                f.write_str("concurrent transition conflict")
            }
            FsmError::PersistenceError(detail) => {
                write!(f, "persistence error: {detail}")
            }
            FsmError::ToolBlockedByState {
                tool_name,
                state,
                category,
            } => {
                write!(
                    f,
                    "tool {tool_name} ({category:?}) blocked in state {state}"
                )
            }
            FsmError::AuditingBlocked { block_count } => {
                write!(
                    f,
                    "AUDITING -> MEMORIZING rejected: deterministic auditor reported {block_count} BLOCK finding(s); resolve and re-enter AUDITING"
                )
            }
            FsmError::ComplianceBlocked { assertion_label, hint } => {
                write!(
                    f,
                    "PLANNING -> TESTING_FIRST refused: EU AI Act assertion {assertion_label} failed; {hint}; see docs/strategy/ELAI-2026-POSTURE.md"
                )
            }
        }
    }
}

impl std::error::Error for FsmError {}

// ---------------------------------------------------------------------------
// Persisted state (what goes to disk via msgpack)
// ---------------------------------------------------------------------------

/// On-disk state representation. Source: phase-3A.txt line 69.
/// Path: .elai_cc/sessions/{sid}/.fsm-state.msgpack
///
/// B-40: added `substate: Option<FsmSubstate>` (additive, defaulted to None on
/// deserialization of older snapshots — `#[serde(default)]` makes the field
/// backward-compatible with the v40-and-earlier msgpack format).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PersistedFsmState {
    pub state: FsmState,
    pub phase_label: String,
    pub token_budget: usize,
    pub replan_log: Vec<ReplanLogEntry>,
    pub transition_history: Vec<TransitionRecord>,
    /// B-40: optional per-state agent substate (e.g. FRAMING under PLANNING).
    /// `None` for v40-and-earlier snapshots; loaders should treat missing as
    /// "no substate active".
    #[serde(default)]
    pub substate: Option<FsmSubstate>,
}

/// Tracks which agents have submitted replan requests per phase.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReplanLogEntry {
    pub agent_id: String,
    pub phase_label: String,
    pub severity: ReplanSeverity,
    pub timestamp_ms: u64,
}

// ---------------------------------------------------------------------------
// PhaseMetrics — collected by orchestrator, emitted at phase boundary
// ---------------------------------------------------------------------------

/// Metrics collected during phase execution, emitted as WorkloadProfilePayload
/// at each FSM transition. Caller (orchestrator) populates from provider usage tracking.
#[derive(Debug, Clone, Default)]
pub struct PhaseMetrics {
    pub agent_role: String,
    pub tokens_in: u64,
    pub tokens_out: u64,
    pub latency_ms: u64,
    pub vram_peak_mb: u32,
    pub model_id: String,
    pub gpu_id: u8,
}

/// Callback type for emitting workload profiles to SSB at phase boundaries.
pub type WorkloadEmitter =
    Box<dyn Fn(crate::ssb::WorkloadProfilePayload) + Send + Sync>;

// ---------------------------------------------------------------------------
// Fsm (the live engine)
// ---------------------------------------------------------------------------

/// The FSM engine. Thread-safe via tokio::Mutex.
/// Implements the Fsm trait from architecture-overview.txt §4.
pub struct Fsm {
    inner: Mutex<FsmInner>,
    workload_emitter: Option<WorkloadEmitter>,
    model_pool: Option<Arc<dyn crate::agent_pool::ModelPool>>,
    /// Session wire-a12: when `Some`, the gate is consulted on every
    /// `_ -> Auditing` transition (result cached in `FsmInner.last_audit_result`)
    /// and the `Auditing -> Memorizing` transition rejects with
    /// `FsmError::AuditingBlocked` if the cached result has any BLOCK findings.
    deterministic_audit_gate:
        Option<Arc<dyn crate::deterministic_audit_gate::DeterministicAuditGate>>,
    /// B-2: SSB writer for emitting `PhaseTransition` rows on every successful
    /// transition. Uses `OnceLock` so the field can be set once after the Fsm is
    /// wrapped in `Arc` (needed because build_runtime returns ssb after Fsm is
    /// already arc-shared). Write failures are non-fatal — a tracing::warn is
    /// emitted but the FSM transition always completes.
    ssb_writer: OnceLock<Arc<crate::ssb::Ssb>>,
    /// E-38 (SCHEMA_VERSION 175): optional EU AI Act compliance pre-gate
    /// provider. When `Some`, the closure is consulted at every
    /// `PLANNING|FEASIBILITY_PROBE -> TESTING_FIRST` transition to produce a
    /// [`crate::compliance::ComplianceVerdict`]. A `Block` verdict refuses
    /// the transition with [`FsmError::ComplianceBlocked`] and emits an
    /// `AiRoadmapSignalEvent::ComplianceBlocked` SSB row (when the
    /// `ELAI_AI_ROADMAP_SIGNALS_ENABLE` env-gate is on); a `Pass` verdict
    /// emits a `ComplianceVerified` row and proceeds. `None` means the
    /// pre-gate is disabled (legacy callers + tests that don't ship a
    /// PlanContext yet).
    compliance_provider: Option<Arc<dyn ComplianceProvider>>,
    /// E-25 Tranche B-4 (D3): opusplan FsmState overlay marker. Wire-dark
    /// metadata flag that signals an active "opus-plan" tier engagement
    /// (planner-tier vs executor-tier runtime routing). Stored at Fsm level
    /// (NOT FsmInner) so the setter/getter stay sync; the FsmInner Mutex is
    /// tokio::sync (async-only). Process-local, not persisted to disk --
    /// restarts default OFF. Pattern-2 reuse: emission goes through
    /// `ClaudeCodeArchitectureEventKind::OpusplanOverlayActivated` via
    /// details_json; NO new FsmSubstate variant minted (E-25 exploration
    /// GAP-4 + decisions D4 -- key overlay on FsmState, not FsmSubstate).
    opus_plan_active: AtomicBool,
}

/// E-38 -- pluggable producer of [`crate::compliance::ComplianceVerdict`]
/// for the FSM `PLANNING|FEASIBILITY_PROBE -> TESTING_FIRST` pre-gate.
/// Implementations build a [`crate::compliance::PlanContext`] from the
/// orchestrator's planning state + clock + D-1 archive presence flag, then
/// call [`crate::compliance::pre_gate_planning_to_testing_first`].
pub trait ComplianceProvider: Send + Sync {
    /// Build a verdict for the upcoming transition. Called with the
    /// current FSM source state so the provider can distinguish the
    /// direct PLANNING -> TESTING_FIRST edge from the
    /// FEASIBILITY_PROBE -> TESTING_FIRST edge.
    fn verdict(
        &self,
        source: FsmState,
        target: FsmState,
    ) -> crate::compliance::ComplianceVerdict;
}

impl std::fmt::Debug for Fsm {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Fsm").field("inner", &"<locked>").finish()
    }
}

struct FsmInner {
    state: FsmState,
    phase_label: String,
    token_budget: usize,
    replan_log: Vec<ReplanLogEntry>,
    transition_history: Vec<TransitionRecord>,
    persist_path: Option<PathBuf>,
    preload_manager: crate::preload::PreloadManager,
    /// Session wire-a12: cached result of the most recent `_ -> Auditing`
    /// gate evaluation. Consumed by the AUDITING -> MEMORIZING transition
    /// guard. Cleared when AUDITING is re-entered (a new run overwrites).
    last_audit_result: Option<crate::deterministic_auditor::DeterministicAuditResult>,
    /// B-40 (TASK-B40-003): active per-state agent substate. Orthogonal to
    /// the top-level FsmState transition matrix. Set by [`Fsm::enter_framing`]
    /// when the Idea-Framer pre-PLANNING pass starts; cleared by
    /// [`Fsm::exit_framing_to_planning`] when the critic loop converges and
    /// the tradeoff card is emitted. Persisted via [`PersistedFsmState::substate`].
    substate: Option<FsmSubstate>,
}

/// E-25 D3: env-gate name controlling whether
/// `OpusplanOverlayActivated` SSB rows are emitted. Default OFF.
pub const ELAI_E25_OPUSPLAN_OVERLAY_ENABLE_VAR: &str = "ELAI_E25_OPUSPLAN_OVERLAY_ENABLE";

/// E-25 D3: returns true iff the opusplan overlay emit path is wire-enabled.
#[must_use]
pub fn opus_plan_overlay_enabled() -> bool {
    std::env::var(ELAI_E25_OPUSPLAN_OVERLAY_ENABLE_VAR)
        .map(|v| v == "1")
        .unwrap_or(false)
}

fn now_ms() -> u64 {
    OsClock
        .system_now()
        .duration_since(UNIX_EPOCH)
        .map(|d| u64::try_from(d.as_millis()).unwrap_or(u64::MAX))
        .unwrap_or(0)
}

impl Fsm {
    /// Create a new FSM in IDLE state.
    pub fn new(persist_path: Option<PathBuf>, token_budget: usize) -> Self {
        Self {
            inner: Mutex::new(FsmInner {
                state: FsmState::Idle,
                phase_label: String::new(),
                token_budget,
                replan_log: Vec::new(),
                transition_history: Vec::new(),
                persist_path,
                preload_manager: crate::preload::PreloadManager::new(),
                last_audit_result: None,
                substate: None,
            }),
            workload_emitter: None,
            model_pool: None,
            deterministic_audit_gate: None,
            ssb_writer: OnceLock::new(),
            compliance_provider: None,
            opus_plan_active: AtomicBool::new(false),
        }
    }

    /// Set the model pool for background pre-loading at phase boundaries.
    pub fn with_model_pool(mut self, pool: Arc<dyn crate::agent_pool::ModelPool>) -> Self {
        self.model_pool = Some(pool);
        self
    }

    /// Session wire-a12: install a [`crate::deterministic_audit_gate::DeterministicAuditGate`].
    ///
    /// After this is set, the FSM:
    ///   * calls `gate.evaluate(source, target)` on every `_ -> Auditing`
    ///     transition and caches the [`crate::deterministic_auditor::DeterministicAuditResult`];
    ///   * rejects `Auditing -> Memorizing` with [`FsmError::AuditingBlocked`]
    ///     whenever the cached result has any BLOCK-severity findings.
    ///
    /// Builder-style consumer; returns `self` so the setter can be chained with
    /// `Fsm::new(...).with_model_pool(...).with_deterministic_audit_gate(...)`.
    pub fn with_deterministic_audit_gate(
        mut self,
        gate: Arc<dyn crate::deterministic_audit_gate::DeterministicAuditGate>,
    ) -> Self {
        self.deterministic_audit_gate = Some(gate);
        self
    }

    /// E-38 (SCHEMA_VERSION 175): install an EU AI Act compliance
    /// [`ComplianceProvider`]. When set, every
    /// `PLANNING|FEASIBILITY_PROBE -> TESTING_FIRST` transition consults
    /// the provider; a `Block` verdict refuses the transition with
    /// [`FsmError::ComplianceBlocked`] and emits an
    /// `AiRoadmapSignalEvent::ComplianceBlocked` SSB row (wire-dark
    /// unless `ELAI_AI_ROADMAP_SIGNALS_ENABLE=1`); a `Pass` verdict
    /// emits a `ComplianceVerified` row and proceeds.
    pub fn with_compliance_provider(
        mut self,
        provider: Arc<dyn ComplianceProvider>,
    ) -> Self {
        self.compliance_provider = Some(provider);
        self
    }

    /// Mutable setter companion to [`Self::with_compliance_provider`].
    pub fn set_compliance_provider(&mut self, provider: Arc<dyn ComplianceProvider>) {
        self.compliance_provider = Some(provider);
    }

    /// Mutable setter companion to [`Self::with_deterministic_audit_gate`].
    /// Useful when the FSM is already owned by a struct field.
    pub fn set_deterministic_audit_gate(
        &mut self,
        gate: Arc<dyn crate::deterministic_audit_gate::DeterministicAuditGate>,
    ) {
        self.deterministic_audit_gate = Some(gate);
    }

    /// Set a callback to emit WorkloadProfilePayload to SSB on phase transitions.
    pub fn set_workload_emitter(&mut self, emitter: WorkloadEmitter) {
        self.workload_emitter = Some(emitter);
    }

    /// E-25 D3: flip the opusplan overlay marker. Pattern-2 reuse -- when
    /// `ELAI_E25_OPUSPLAN_OVERLAY_ENABLE` is `1` and the flag transitions
    /// (off->on / on->off) the caller is expected to emit a
    /// `ClaudeCodeArchitectureEvent` (`OpusplanOverlayActivated`) row whose
    /// `details_json` carries `fsm_overlay_event` =
    /// `opus_plan_started` | `opus_plan_ended`. Emission is the caller's
    /// responsibility because the Ssb writer is async and this setter is
    /// sync; the helper [`opus_plan_overlay_enabled`] surfaces the env-gate
    /// state for the caller to consult.
    pub fn set_opus_plan_active(&self, on: bool) {
        self.opus_plan_active.store(on, Ordering::Release);
    }

    /// E-25 D3: read the current opus_plan overlay flag.
    pub fn opus_plan_active(&self) -> bool {
        self.opus_plan_active.load(Ordering::Acquire)
    }

    /// B-2: builder-style setter for the SSB writer.  Call before wrapping in Arc.
    pub fn with_ssb_writer(self, writer: Arc<crate::ssb::Ssb>) -> Self {
        // Ignore the result: OnceLock guarantees the first write wins.
        let _ = self.ssb_writer.set(writer);
        self
    }

    /// B-2: wire an SSB writer into an already-Arc-shared Fsm.
    ///
    /// Uses `OnceLock::set` which takes `&self`, so this can be called through
    /// `Arc<Fsm>` after `build_runtime` returns.  The first caller wins;
    /// subsequent calls are silently ignored (the lock is already initialised).
    pub fn set_ssb_writer(&self, writer: Arc<crate::ssb::Ssb>) {
        let _ = self.ssb_writer.set(writer);
    }

    /// Load FSM state from a msgpack file on disk.
    /// Returns CorruptState if the file exists but cannot be deserialized.
    pub async fn load(path: &Path) -> Result<Self, FsmError> {
        let bytes = tokio::fs::read(path).await.map_err(|e| {
            if e.kind() == std::io::ErrorKind::NotFound {
                // No state file -> start fresh from IDLE
                return FsmError::PersistenceError(format!("file not found: {}", path.display()));
            }
            FsmError::CorruptState {
                path: path.to_path_buf(),
                detail: format!("io error: {e}"),
            }
        })?;

        let persisted: PersistedFsmState =
            rmp_serde::from_slice(&bytes).map_err(|e| FsmError::CorruptState {
                path: path.to_path_buf(),
                detail: format!("msgpack decode: {e}"),
            })?;

        Ok(Self {
            inner: Mutex::new(FsmInner {
                state: persisted.state,
                phase_label: persisted.phase_label,
                token_budget: persisted.token_budget,
                replan_log: persisted.replan_log,
                transition_history: persisted.transition_history,
                persist_path: Some(path.to_path_buf()),
                preload_manager: crate::preload::PreloadManager::new(),
                last_audit_result: None,
                // B-40 (TASK-B40-003): restore substate from persisted snapshot.
                // Older v40-and-earlier snapshots default to None via #[serde(default)].
                substate: persisted.substate,
            }),
            workload_emitter: None,
            model_pool: None,
            deterministic_audit_gate: None,
            ssb_writer: OnceLock::new(),
            compliance_provider: None,
            opus_plan_active: AtomicBool::new(false),
        })
    }

    /// Read persisted FSM state from disk synchronously. Returns Idle if missing or corrupt.
    pub fn read_persisted_state(path: &Path) -> FsmState {
        match std::fs::read(path) {
            Ok(bytes) => rmp_serde::from_slice::<PersistedFsmState>(&bytes)
                .map(|p| p.state)
                .unwrap_or(FsmState::Idle),
            Err(_) => FsmState::Idle,
        }
    }

    /// Current FSM state.
    pub async fn get_state(&self) -> FsmState {
        self.inner.lock().await.state
    }

    /// Remaining token budget for context allocation.
    /// Source: architecture-overview.txt §4 L136.
    pub async fn token_budget_remaining(&self) -> usize {
        self.inner.lock().await.token_budget
    }

    /// Set the token budget (called by context budget allocator).
    pub async fn set_token_budget(&self, budget: usize) {
        self.inner.lock().await.token_budget = budget;
    }

    /// Current phase label.
    pub async fn phase_label(&self) -> String {
        self.inner.lock().await.phase_label.clone()
    }

    /// Transition history (clone for inspection).
    pub async fn transition_history(&self) -> Vec<TransitionRecord> {
        self.inner.lock().await.transition_history.clone()
    }

    /// Session wire-a12: inspect the cached deterministic audit result from
    /// the most recent `_ -> Auditing` transition. Returns `None` if no gate
    /// was installed or AUDITING has not been entered yet.
    pub async fn last_audit_result(
        &self,
    ) -> Option<crate::deterministic_auditor::DeterministicAuditResult> {
        self.inner.lock().await.last_audit_result.clone()
    }

    /// Attempt a state transition. Validates against the transition matrix.
    /// On success, persists to disk (if persist_path set) and records the transition.
    /// If `metrics` is provided and a workload emitter is set, emits a
    /// `WorkloadProfilePayload` to SSB at the phase boundary.
    ///
    /// Uses try_lock to detect concurrent transitions (TransitionConflict).
    pub async fn transition(
        &self,
        target: FsmState,
        phase_label: &str,
        evidence: Vec<EvidenceArtifact>,
    ) -> Result<TransitionRecord, FsmError> {
        self.transition_with_metrics(target, phase_label, evidence, None)
            .await
    }

    /// Transition with optional phase metrics for workload profiling.
    pub async fn transition_with_metrics(
        &self,
        target: FsmState,
        phase_label: &str,
        evidence: Vec<EvidenceArtifact>,
        metrics: Option<PhaseMetrics>,
    ) -> Result<TransitionRecord, FsmError> {
        let mut guard = self
            .inner
            .try_lock()
            .map_err(|_| FsmError::TransitionConflict)?;

        let source = guard.state;
        if !source.can_transition_to(target) {
            return Err(FsmError::InvalidTransition { source, target });
        }

        // Session wire-a12: gate AUDITING -> MEMORIZING on any BLOCK finding
        // from the most recent `_ -> Auditing` evaluation. This is consulted
        // BEFORE state mutation so the FSM stays in AUDITING on rejection.
        if source == FsmState::Auditing && target == FsmState::Memorizing {
            if let Some(ref result) = guard.last_audit_result {
                let block_count = result.block_count();
                if block_count > 0 {
                    return Err(FsmError::AuditingBlocked { block_count });
                }
            }
        }

        // E-38 (SCHEMA_VERSION 175): EU AI Act compliance pre-gate at
        // PLANNING|FEASIBILITY_PROBE -> TESTING_FIRST. Block on any of
        // the 4 assertions failing; emit AiRoadmapSignalEvent SSB row
        // (wire-dark unless ELAI_AI_ROADMAP_SIGNALS_ENABLE=1). The gate
        // ITSELF runs whenever a provider is installed; the env-gate
        // controls observability only. See compliance.rs +
        // workflow/reasoning/E-38-decisions.md.
        if target == FsmState::TestingFirst
            && (source == FsmState::Planning || source == FsmState::FeasibilityProbe)
        {
            if let Some(ref provider) = self.compliance_provider {
                let verdict = provider.verdict(source, target);
                match &verdict {
                    crate::compliance::ComplianceVerdict::Block { assertion, hint } => {
                        // Emit the SSB row before returning the error so
                        // the dashboard tile reflects the block.
                        if crate::roadmap_signals::roadmap_signals_enabled() {
                            if let Some(writer) = self.ssb_writer.get() {
                                let ts_ms = i64::try_from(now_ms()).unwrap_or(i64::MAX);
                                let ctx = crate::compliance::PlanContext {
                                    plan_session_id: guard.phase_label.clone(),
                                    has_clock_provider: false,
                                    has_deterministic_seed: false,
                                    template_hash_chain_verified: false,
                                    template_archive_present: false,
                                    ssb_row_count_in_planning_phase: 0,
                                };
                                let payload = crate::ssb::SsbPayload::AiRoadmapSignalEvent(
                                    crate::compliance::build_block_payload(
                                        &ctx,
                                        *assertion,
                                        hint,
                                        ts_ms,
                                    ),
                                );
                                if let Err(e) = writer
                                    .write_payload("fsm", 0, payload, Vec::new())
                                    .await
                                {
                                    tracing::warn!(
                                        assertion = assertion.as_str(),
                                        error = %e,
                                        "fsm: ai_roadmap_signal_event compliance_blocked SSB write failed (non-fatal)"
                                    );
                                }
                            }
                        }
                        return Err(FsmError::ComplianceBlocked {
                            assertion_label: assertion.as_str().to_string(),
                            hint: hint.clone(),
                        });
                    }
                    crate::compliance::ComplianceVerdict::Pass { assertions_checked } => {
                        if crate::roadmap_signals::roadmap_signals_enabled() {
                            if let Some(writer) = self.ssb_writer.get() {
                                let ts_ms = i64::try_from(now_ms()).unwrap_or(i64::MAX);
                                let ctx = crate::compliance::PlanContext {
                                    plan_session_id: guard.phase_label.clone(),
                                    has_clock_provider: true,
                                    has_deterministic_seed: true,
                                    template_hash_chain_verified: true,
                                    template_archive_present: false,
                                    ssb_row_count_in_planning_phase:
                                        crate::compliance::AUDIT_TRAIL_MIN_SSB_ROWS,
                                };
                                let payload = crate::ssb::SsbPayload::AiRoadmapSignalEvent(
                                    crate::compliance::build_verified_payload(
                                        &ctx,
                                        *assertions_checked,
                                        ts_ms,
                                    ),
                                );
                                if let Err(e) = writer
                                    .write_payload("fsm", 0, payload, Vec::new())
                                    .await
                                {
                                    tracing::warn!(
                                        assertions_checked = assertions_checked,
                                        error = %e,
                                        "fsm: ai_roadmap_signal_event compliance_verified SSB write failed (non-fatal)"
                                    );
                                }
                            }
                        }
                        // Fall through to standard transition path.
                    }
                }
            }
        }

        // Session wire-a12: on entering AUDITING, evaluate the deterministic
        // gate and cache the result. Findings are emitted by the gate impl
        // (fire-and-forget — see `DefaultDeterministicAuditGate`).
        if target == FsmState::Auditing {
            if let Some(ref gate) = self.deterministic_audit_gate {
                let result = gate.evaluate(source, target);
                guard.last_audit_result = Some(result);
            } else {
                guard.last_audit_result = None;
            }
        }

        let record = TransitionRecord {
            source,
            target,
            phase_label: phase_label.to_string(),
            timestamp_ms: now_ms(),
            evidence,
        };

        guard.state = target;
        guard.phase_label = phase_label.to_string();
        guard.transition_history.push(record.clone());

        // Trigger predictive pre-loading on state entry (Phase 3J.6).
        // If a ModelPool is configured, spawns background VRAM checks + load readiness.
        // Otherwise falls back to placement-only computation (no actual loading).
        if let Some(ref pool) = self.model_pool {
            let _handle = guard.preload_manager.trigger_preload_background(target, Arc::clone(pool));
        } else {
            let models_to_preload = guard.preload_manager.trigger_preload(target);
            if !models_to_preload.is_empty() {
                tracing::info!(
                    state = %target,
                    count = models_to_preload.len(),
                    models = ?models_to_preload,
                    "Predictive pre-load triggered (no ModelPool configured, placement only)"
                );
            }
        }

        // Emit workload profile at phase boundary via callback
        if let (Some(ref emitter), Some(ref m)) = (&self.workload_emitter, &metrics) {
            let payload = crate::ssb::WorkloadProfilePayload {
                agent_role: m.agent_role.clone(),
                phase_id: phase_label.to_string(),
                tokens_in: m.tokens_in,
                tokens_out: m.tokens_out,
                latency_ms: m.latency_ms,
                vram_peak_mb: m.vram_peak_mb,
                model_id: m.model_id.clone(),
                gpu_id: m.gpu_id,
                timestamp: i64::try_from(now_ms()).unwrap_or(i64::MAX),
            };
            tracing::info!(
                phase = phase_label,
                agent = %m.agent_role,
                tokens_in = m.tokens_in,
                tokens_out = m.tokens_out,
                latency_ms = m.latency_ms,
                "Emitting workload profile at phase boundary"
            );
            emitter(payload);
        }

        // Clear replan log on phase boundary (new phase = fresh limit)
        if source == FsmState::Memorizing || source == FsmState::Complete {
            guard.replan_log.clear();
        }

        // B-40 (TASK-B40-003): top-level transition implicitly exits any
        // active substate. Clear in-memory before persisting so the on-disk
        // snapshot reflects the current FsmInner.
        guard.substate = None;

        // Persist to disk if path configured
        if let Some(ref path) = guard.persist_path {
            let persisted = PersistedFsmState {
                state: guard.state,
                phase_label: guard.phase_label.clone(),
                token_budget: guard.token_budget,
                replan_log: guard.replan_log.clone(),
                transition_history: guard.transition_history.clone(),
                // B-40: substate is tracked separately by the framing
                // workflow; FSM transitions clear any prior substate
                // implicitly (top-level transition = exit any substate).
                substate: guard.substate,
            };
            persist_state(path, &persisted).await?;
        }

        // B-2: emit PhaseTransition SSB row on every successful transition.
        // Non-blocking: SSB write failure must not block the FSM transition.
        if let Some(writer) = self.ssb_writer.get() {
            let payload = crate::ssb::SsbPayload::PhaseTransition(
                crate::ssb::PhaseTransitionPayload {
                    from_state: source.to_string(),
                    to_state:   target.to_string(),
                    evidence_summary: phase_label.to_string(),
                },
            );
            if let Err(e) = writer
                .write_payload("fsm", 0, payload, Vec::new())
                .await
            {
                tracing::warn!(
                    from = %source,
                    to   = %target,
                    error = %e,
                    "fsm: phase_transition SSB write failed (non-fatal)"
                );
            }
        }

        Ok(record)
    }

    /// Submit a ReplanRequest. Enforces max-1 per agent per phase.
    /// Source: orchestrator.txt §13 L511, phase-3A.txt line 71.
    pub async fn submit_replan(
        &self,
        request: ReplanRequest,
    ) -> Result<ReplanLogEntry, FsmError> {
        let mut guard = self.inner.lock().await;

        // Check: must be in EXECUTING state
        if guard.state != FsmState::Executing {
            return Err(FsmError::InvalidTransition {
                source: guard.state,
                target: FsmState::Planning,
            });
        }

        // Check: max 1 per agent per phase
        let already_requested = guard.replan_log.iter().any(|entry| {
            entry.agent_id == request.agent_id && entry.phase_label == request.phase_label
        });
        if already_requested {
            return Err(FsmError::ReplanLimitExceeded {
                agent_id: request.agent_id,
                phase_label: request.phase_label,
            });
        }

        let entry = ReplanLogEntry {
            agent_id: request.agent_id,
            phase_label: request.phase_label,
            severity: request.severity,
            timestamp_ms: now_ms(),
        };
        guard.replan_log.push(entry.clone());

        // Persist
        if let Some(ref path) = guard.persist_path {
            let persisted = PersistedFsmState {
                state: guard.state,
                phase_label: guard.phase_label.clone(),
                token_budget: guard.token_budget,
                replan_log: guard.replan_log.clone(),
                transition_history: guard.transition_history.clone(),
                substate: guard.substate,
            };
            persist_state(path, &persisted).await?;
        }

        Ok(entry)
    }

    /// Check whether a tool is allowed in the current FSM state.
    /// Returns `Ok(())` if allowed, `Err(ToolBlockedByState)` if blocked.
    /// Phase 3A.2: enforcement point. Source: phase-3A.txt L125-126.
    pub async fn check_tool_allowed(&self, tool_name: &str) -> Result<(), FsmError> {
        let state = self.inner.lock().await.state;
        state.check_tool_allowed(tool_name).map_err(|category| {
            FsmError::ToolBlockedByState {
                tool_name: tool_name.to_string(),
                state,
                category,
            }
        })
    }

    /// Reset FSM to IDLE. Escape hatch from any state.
    pub async fn reset(&self) -> Result<(), FsmError> {
        let mut guard = self.inner.lock().await;
        guard.state = FsmState::Idle;
        guard.phase_label.clear();
        guard.replan_log.clear();
        // B-40 (TASK-B40-003): reset clears any active substate too.
        guard.substate = None;

        if let Some(ref path) = guard.persist_path {
            let persisted = PersistedFsmState {
                state: guard.state,
                phase_label: guard.phase_label.clone(),
                token_budget: guard.token_budget,
                replan_log: guard.replan_log.clone(),
                transition_history: guard.transition_history.clone(),
                substate: None,
            };
            persist_state(path, &persisted).await?;
        }

        Ok(())
    }

    // -----------------------------------------------------------------------
    // B-40 (TASK-B40-003): FsmSubstate::Framing transition helpers
    // -----------------------------------------------------------------------

    /// Inspect the current per-state substate. Returns `None` when no substate
    /// is active. See [`FsmSubstate`] doc for the lifecycle.
    pub async fn current_substate(&self) -> Option<FsmSubstate> {
        self.inner.lock().await.substate
    }

    /// B-40 (TASK-B40-003) — enter the [`FsmSubstate::Framing`] substate.
    ///
    /// Entry is permitted from `FsmState::Idle` (pre-PLANNING idea-stage
    /// discussion) or `FsmState::Planning` (mid-PLANNING revisit of the
    /// framing pass). The top-level state is NOT mutated — substates are
    /// orthogonal to the `FsmState` transition matrix per the comment on
    /// [`FsmSubstate`].
    ///
    /// Errors:
    ///   * [`FsmError::InvalidTransition`] when called from a state that
    ///     does not permit Framing entry (anything outside Idle / Planning).
    ///   * [`FsmError::TransitionConflict`] when another task is mid-transition.
    pub async fn enter_framing(&self) -> Result<(), FsmError> {
        let mut guard = self
            .inner
            .try_lock()
            .map_err(|_| FsmError::TransitionConflict)?;
        let source = guard.state;
        if source != FsmState::Idle && source != FsmState::Planning {
            return Err(FsmError::InvalidTransition {
                source,
                target: FsmState::Planning,
            });
        }
        guard.substate = Some(FsmSubstate::Framing);

        if let Some(ref path) = guard.persist_path {
            let persisted = PersistedFsmState {
                state: guard.state,
                phase_label: guard.phase_label.clone(),
                token_budget: guard.token_budget,
                replan_log: guard.replan_log.clone(),
                transition_history: guard.transition_history.clone(),
                substate: guard.substate,
            };
            persist_state(path, &persisted).await?;
        }
        tracing::info!(
            source = %source,
            substate = %FsmSubstate::Framing,
            "fsm: entered FRAMING substate (B-40 idea-framer pre-PLANNING pass)"
        );
        Ok(())
    }

    /// B-40 (TASK-B40-003) — exit [`FsmSubstate::Framing`] and advance the
    /// top-level state to [`FsmState::Planning`].
    ///
    /// Called when the idea-framer critic loop converges and the
    /// `tradeoff_card_emitted` SSB row has been written. Clears the substate
    /// and, if the current top-level state is `FsmState::Idle`, transitions
    /// to `FsmState::Planning`. If already in PLANNING, just clears the
    /// substate (idempotent revisit from within PLANNING).
    ///
    /// Errors:
    ///   * [`FsmError::InvalidTransition`] when the current substate is not
    ///     `Some(FsmSubstate::Framing)` — calling this without enter_framing
    ///     first is a contract violation.
    ///   * Top-level transition errors propagate from
    ///     [`Self::transition_with_metrics`] (only when source==Idle).
    pub async fn exit_framing_to_planning(
        &self,
        phase_label: &str,
    ) -> Result<(), FsmError> {
        // Lock + verify substate is Framing.
        {
            let mut guard = self
                .inner
                .try_lock()
                .map_err(|_| FsmError::TransitionConflict)?;
            if guard.substate != Some(FsmSubstate::Framing) {
                return Err(FsmError::InvalidTransition {
                    source: guard.state,
                    target: FsmState::Planning,
                });
            }
            // Clear substate up-front so the top-level transition below
            // does not redundantly re-clear it (the transition path already
            // sets guard.substate = None unconditionally).
            guard.substate = None;
            if guard.state == FsmState::Planning {
                // Already in PLANNING — persist the cleared substate and return.
                if let Some(ref path) = guard.persist_path {
                    let persisted = PersistedFsmState {
                        state: guard.state,
                        phase_label: guard.phase_label.clone(),
                        token_budget: guard.token_budget,
                        replan_log: guard.replan_log.clone(),
                        transition_history: guard.transition_history.clone(),
                        substate: None,
                    };
                    persist_state(path, &persisted).await?;
                }
                tracing::info!(
                    "fsm: exited FRAMING substate (already in PLANNING; idempotent)"
                );
                return Ok(());
            }
            // Source is Idle (the only other entry-permitted state). Fall
            // through to the standard transition path below to handle SSB
            // emission + transition_history + workload_emitter.
        }
        // Standard Idle -> Planning transition with no metrics/evidence.
        self
            .transition_with_metrics(FsmState::Planning, phase_label, Vec::new(), None)
            .await?;
        tracing::info!(
            substate = %FsmSubstate::Framing,
            "fsm: exited FRAMING substate -> PLANNING (B-40 idea-framer convergence)"
        );
        Ok(())
    }
}

/// Write persisted state to disk atomically (write-then-rename).
async fn persist_state(path: &Path, state: &PersistedFsmState) -> Result<(), FsmError> {
    let bytes =
        rmp_serde::to_vec(state).map_err(|e| FsmError::PersistenceError(format!("encode: {e}")))?;

    // Ensure parent dir exists
    if let Some(parent) = path.parent() {
        tokio::fs::create_dir_all(parent)
            .await
            .map_err(|e| FsmError::PersistenceError(format!("mkdir: {e}")))?;
    }

    // Atomic write: write to .tmp then rename
    let tmp_path = path.with_extension("msgpack.tmp");
    tokio::fs::write(&tmp_path, &bytes)
        .await
        .map_err(|e| FsmError::PersistenceError(format!("write: {e}")))?;
    tokio::fs::rename(&tmp_path, path)
        .await
        .map_err(|e| FsmError::PersistenceError(format!("rename: {e}")))?;

    Ok(())
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;
    use std::sync::Arc;

    // -- Helpers --

    fn arbitrary_state() -> impl Strategy<Value = FsmState> {
        prop_oneof![
            Just(FsmState::Idle),
            Just(FsmState::Planning),
            Just(FsmState::FeasibilityProbe),
            Just(FsmState::TestingFirst),
            Just(FsmState::Executing),
            Just(FsmState::Auditing),
            Just(FsmState::Memorizing),
            Just(FsmState::Complete),
            Just(FsmState::Configuring),
            Just(FsmState::Researching),
        ]
    }

    // -- Unit: all 9 states serialize/deserialize via msgpack round-trip --

    #[test]
    fn test_all_states_msgpack_roundtrip() {
        for state in FsmState::ALL {
            let bytes = rmp_serde::to_vec(&state).expect("encode");
            let decoded: FsmState = rmp_serde::from_slice(&bytes).expect("decode");
            assert_eq!(state, decoded, "round-trip failed for {state}");
        }
    }

    #[test]
    fn test_persisted_state_msgpack_roundtrip() {
        let persisted = PersistedFsmState {
            state: FsmState::Executing,
            phase_label: "3A.1".to_string(),
            token_budget: 8000,
            replan_log: vec![ReplanLogEntry {
                agent_id: "executor-1".to_string(),
                phase_label: "3A.1".to_string(),
                severity: ReplanSeverity::Minor,
                timestamp_ms: 1700000000000,
            }],
            transition_history: vec![TransitionRecord {
                source: FsmState::Idle,
                target: FsmState::Planning,
                phase_label: "3A.1".to_string(),
                timestamp_ms: 1700000000000,
                evidence: vec![EvidenceArtifact {
                    kind: "readiness".to_string(),
                    summary: "READY".to_string(),
                    data: vec![1, 2, 3],
                }],
            }],
            substate: None,
        };
        let bytes = rmp_serde::to_vec(&persisted).expect("encode");
        let decoded: PersistedFsmState = rmp_serde::from_slice(&bytes).expect("decode");
        assert_eq!(decoded.state, FsmState::Executing);
        assert_eq!(decoded.phase_label, "3A.1");
        assert_eq!(decoded.token_budget, 8000);
        assert_eq!(decoded.replan_log.len(), 1);
        assert_eq!(decoded.transition_history.len(), 1);
    }

    // -- Unit: exhaustive valid/invalid transition testing (9x9 = 81) --

    #[test]
    fn test_transition_matrix_exhaustive() {
        let valid_pairs: &[(FsmState, FsmState)] = &[
            // IDLE
            (FsmState::Idle, FsmState::Planning),
            (FsmState::Idle, FsmState::Configuring),
            (FsmState::Idle, FsmState::Researching),
            // PLANNING
            (FsmState::Planning, FsmState::FeasibilityProbe),
            (FsmState::Planning, FsmState::TestingFirst),
            // FEASIBILITY_PROBE
            (FsmState::FeasibilityProbe, FsmState::Planning),
            (FsmState::FeasibilityProbe, FsmState::TestingFirst),
            // TESTING_FIRST
            (FsmState::TestingFirst, FsmState::Executing),
            // EXECUTING
            (FsmState::Executing, FsmState::Auditing),
            (FsmState::Executing, FsmState::Planning),
            // AUDITING
            (FsmState::Auditing, FsmState::Memorizing),
            (FsmState::Auditing, FsmState::Executing),
            (FsmState::Auditing, FsmState::Planning),
            // MEMORIZING
            (FsmState::Memorizing, FsmState::Planning),
            (FsmState::Memorizing, FsmState::Complete),
            // COMPLETE
            (FsmState::Complete, FsmState::Idle),
            // CONFIGURING
            (FsmState::Configuring, FsmState::Idle),
            // RESEARCHING (follow-up self-loop, exit to IDLE, escalate to PLANNING)
            (FsmState::Researching, FsmState::Researching),
            (FsmState::Researching, FsmState::Idle),
            (FsmState::Researching, FsmState::Planning),
        ];

        // Verify all valid pairs
        for &(src, tgt) in valid_pairs {
            assert!(
                src.can_transition_to(tgt),
                "expected valid: {src} -> {tgt}"
            );
        }

        // 10 states = 100 pairs. valid_count = 20.
        let mut valid_count = 0;
        let mut invalid_count = 0;
        for src in FsmState::ALL {
            for tgt in FsmState::ALL {
                if valid_pairs.contains(&(src, tgt)) {
                    valid_count += 1;
                } else {
                    assert!(
                        !src.can_transition_to(tgt),
                        "expected invalid: {src} -> {tgt}"
                    );
                    invalid_count += 1;
                }
            }
        }
        assert_eq!(valid_count, 20);
        assert_eq!(valid_count + invalid_count, 100);
    }

    // -- Unit: specific invalid transitions from adversarial spec --

    #[test]
    fn test_invalid_transitions_adversarial() {
        // phase-3A.txt adversarial: IDLE->AUDITING, TESTING_FIRST->MEMORIZING,
        // COMPLETE->EXECUTING must all fail.
        assert!(!FsmState::Idle.can_transition_to(FsmState::Auditing));
        assert!(!FsmState::TestingFirst.can_transition_to(FsmState::Memorizing));
        assert!(!FsmState::Complete.can_transition_to(FsmState::Executing));
    }

    // -- Unit: FSM transition via engine --

    #[tokio::test]
    async fn test_fsm_transition_valid() {
        let fsm = Fsm::new(None, 8000);
        assert_eq!(fsm.get_state().await, FsmState::Idle);

        let record = fsm
            .transition(FsmState::Planning, "3A.1", vec![])
            .await
            .expect("valid transition");
        assert_eq!(record.source, FsmState::Idle);
        assert_eq!(record.target, FsmState::Planning);
        assert_eq!(fsm.get_state().await, FsmState::Planning);
    }

    #[tokio::test]
    async fn test_fsm_transition_invalid() {
        let fsm = Fsm::new(None, 8000);
        let err = fsm
            .transition(FsmState::Auditing, "3A.1", vec![])
            .await
            .expect_err("should be invalid");
        assert_eq!(
            err,
            FsmError::InvalidTransition {
                source: FsmState::Idle,
                target: FsmState::Auditing,
            }
        );
        // State should not have changed
        assert_eq!(fsm.get_state().await, FsmState::Idle);
    }

    // -- Unit: full 9-state traversal --

    #[tokio::test]
    async fn test_full_traversal_all_states() {
        let fsm = Fsm::new(None, 8000);

        // Main path: IDLE -> PLANNING -> FEASIBILITY_PROBE -> TESTING_FIRST
        //   -> EXECUTING -> AUDITING -> MEMORIZING -> COMPLETE -> IDLE
        let path = [
            FsmState::Planning,
            FsmState::FeasibilityProbe,
            FsmState::TestingFirst,
            FsmState::Executing,
            FsmState::Auditing,
            FsmState::Memorizing,
            FsmState::Complete,
            FsmState::Idle,
        ];

        for target in path {
            fsm.transition(target, "traversal", vec![])
                .await
                .unwrap_or_else(|e| panic!("failed at -> {target}: {e}"));
        }
        assert_eq!(fsm.get_state().await, FsmState::Idle);

        // Also test CONFIGURING path
        fsm.transition(FsmState::Configuring, "config", vec![])
            .await
            .expect("IDLE -> CONFIGURING");
        fsm.transition(FsmState::Idle, "config-done", vec![])
            .await
            .expect("CONFIGURING -> IDLE");
        assert_eq!(fsm.get_state().await, FsmState::Idle);
    }

    // -- Unit: token_budget_remaining --

    #[tokio::test]
    async fn test_token_budget_remaining() {
        let fsm = Fsm::new(None, 12000);
        assert_eq!(fsm.token_budget_remaining().await, 12000);
        fsm.set_token_budget(5000).await;
        assert_eq!(fsm.token_budget_remaining().await, 5000);
    }

    // -- Unit: ReplanRequest limit enforcement --

    #[tokio::test]
    async fn test_replan_limit_enforcement() {
        let fsm = Fsm::new(None, 8000);
        // Move to EXECUTING
        fsm.transition(FsmState::Planning, "p1", vec![])
            .await
            .unwrap();
        fsm.transition(FsmState::TestingFirst, "p1", vec![])
            .await
            .unwrap();
        fsm.transition(FsmState::Executing, "p1", vec![])
            .await
            .unwrap();

        // First replan: accepted
        let entry = fsm
            .submit_replan(ReplanRequest {
                severity: ReplanSeverity::Minor,
                evidence: "tests indicate wrong approach".to_string(),
                agent_id: "executor-1".to_string(),
                phase_label: "p1".to_string(),
            })
            .await
            .expect("first replan accepted");
        assert_eq!(entry.severity, ReplanSeverity::Minor);

        // Second replan from same agent in same phase: rejected
        let err = fsm
            .submit_replan(ReplanRequest {
                severity: ReplanSeverity::Major,
                evidence: "still wrong".to_string(),
                agent_id: "executor-1".to_string(),
                phase_label: "p1".to_string(),
            })
            .await
            .expect_err("second replan rejected");
        assert_eq!(
            err,
            FsmError::ReplanLimitExceeded {
                agent_id: "executor-1".to_string(),
                phase_label: "p1".to_string(),
            }
        );
    }

    // -- Unit: ReplanRequest from wrong state --

    #[tokio::test]
    async fn test_replan_wrong_state() {
        let fsm = Fsm::new(None, 8000);
        let err = fsm
            .submit_replan(ReplanRequest {
                severity: ReplanSeverity::Minor,
                evidence: "n/a".to_string(),
                agent_id: "executor-1".to_string(),
                phase_label: "p1".to_string(),
            })
            .await
            .expect_err("replan from IDLE should fail");
        matches!(err, FsmError::InvalidTransition { .. });
    }

    // -- Unit: state persistence and recovery --

    #[tokio::test]
    async fn test_persistence_roundtrip() {
        let dir = tempfile::tempdir().expect("tmpdir");
        let path = dir.path().join("sessions").join("test").join(".fsm-state.msgpack");

        let fsm = Fsm::new(Some(path.clone()), 8000);
        fsm.transition(FsmState::Planning, "3A.1", vec![])
            .await
            .expect("transition");
        fsm.transition(
            FsmState::TestingFirst,
            "3A.1",
            vec![EvidenceArtifact {
                kind: "grounding".to_string(),
                summary: "all grounded".to_string(),
                data: vec![],
            }],
        )
        .await
        .expect("transition");

        // Load from disk
        let fsm2 = Fsm::load(&path).await.expect("load");
        assert_eq!(fsm2.get_state().await, FsmState::TestingFirst);
        assert_eq!(fsm2.phase_label().await, "3A.1");
        assert_eq!(fsm2.token_budget_remaining().await, 8000);
        let history = fsm2.transition_history().await;
        assert_eq!(history.len(), 2);
    }

    // -- Adversarial: corrupt msgpack --

    #[tokio::test]
    async fn test_corrupt_state_file() {
        let dir = tempfile::tempdir().expect("tmpdir");
        let path = dir.path().join(".fsm-state.msgpack");
        tokio::fs::write(&path, b"this is not msgpack")
            .await
            .expect("write corrupt");

        let err = Fsm::load(&path).await.expect_err("should fail on corrupt");
        match err {
            FsmError::CorruptState { detail, .. } => {
                assert!(detail.contains("msgpack decode"), "detail: {detail}");
            }
            other => panic!("expected CorruptState, got: {other}"),
        }
    }

    // -- Adversarial: concurrent transition attempts --

    #[tokio::test]
    async fn test_concurrent_transitions() {
        let fsm = Arc::new(Fsm::new(None, 8000));

        // Move to PLANNING first (single-thread)
        fsm.transition(FsmState::Planning, "p1", vec![])
            .await
            .unwrap();

        // PLANNING has two valid targets: FeasibilityProbe and TestingFirst.
        // Spawn 2 tasks attempting different transitions simultaneously.
        let fsm1 = Arc::clone(&fsm);
        let fsm2 = Arc::clone(&fsm);

        // Use a barrier to synchronize the start
        let barrier = Arc::new(tokio::sync::Barrier::new(2));
        let b1 = Arc::clone(&barrier);
        let b2 = Arc::clone(&barrier);

        let h1 = tokio::spawn(async move {
            b1.wait().await;
            fsm1.transition(FsmState::TestingFirst, "p1", vec![]).await
        });
        let h2 = tokio::spawn(async move {
            b2.wait().await;
            fsm2.transition(FsmState::FeasibilityProbe, "p1", vec![])
                .await
        });

        let (r1, r2) = tokio::join!(h1, h2);
        let r1 = r1.unwrap();
        let r2 = r2.unwrap();

        // Exactly one should succeed, the other should get TransitionConflict
        // OR InvalidTransition (if the first changed state before the second tried).
        let successes = [&r1, &r2].iter().filter(|r| r.is_ok()).count();
        let conflicts = [&r1, &r2]
            .iter()
            .filter(|r| {
                matches!(
                    r,
                    Err(FsmError::TransitionConflict) | Err(FsmError::InvalidTransition { .. })
                )
            })
            .count();

        assert!(
            successes >= 1 && successes + conflicts == 2,
            "expected 1 success + 1 conflict/invalid, got: r1={r1:?}, r2={r2:?}"
        );
    }

    // -- Unit: reset from any state --

    #[tokio::test]
    async fn test_reset_to_idle() {
        let fsm = Fsm::new(None, 8000);
        fsm.transition(FsmState::Planning, "p1", vec![])
            .await
            .unwrap();
        fsm.transition(FsmState::TestingFirst, "p1", vec![])
            .await
            .unwrap();
        fsm.reset().await.expect("reset");
        assert_eq!(fsm.get_state().await, FsmState::Idle);
    }

    // -- Unit: display for all states --

    #[test]
    fn test_display_all_states() {
        let expected = [
            "IDLE",
            "PLANNING",
            "FEASIBILITY_PROBE",
            "TESTING_FIRST",
            "EXECUTING",
            "AUDITING",
            "MEMORIZING",
            "COMPLETE",
            "CONFIGURING",
            "RESEARCHING",
        ];
        for (state, name) in FsmState::ALL.iter().zip(expected.iter()) {
            assert_eq!(state.to_string(), *name);
        }
    }

    // -- Property-based: random valid transitions always reach IDLE or COMPLETE --

    proptest! {
        #[test]
        fn prop_valid_transitions_reach_terminal(
            steps in proptest::collection::vec(0usize..3, 1..20)
        ) {
            let rt = tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
                .unwrap();
            rt.block_on(async {
                let fsm = Fsm::new(None, 8000);
                let mut current = FsmState::Idle;

                for step_idx in steps {
                    let targets = current.valid_targets();
                    if targets.is_empty() {
                        break;
                    }
                    let target = targets[step_idx % targets.len()];
                    match fsm.transition(target, "prop", vec![]).await {
                        Ok(record) => {
                            current = record.target;
                        }
                        Err(_) => break,
                    }
                }
                // FSM should be in some valid state (not corrupted)
                let final_state = fsm.get_state().await;
                prop_assert!(FsmState::ALL.contains(&final_state));
                Ok(())
            })?;
        }

        #[test]
        fn prop_invalid_transitions_never_change_state(
            src in arbitrary_state(),
            tgt in arbitrary_state()
        ) {
            if !src.can_transition_to(tgt) {
                let rt = tokio::runtime::Builder::new_current_thread()
                    .enable_all()
                    .build()
                    .unwrap();
                rt.block_on(async {
                    // Build FSM at `src` state by resetting and walking
                    let fsm = Fsm::new(None, 8000);
                    // Walk to src state if possible
                    let walked = walk_to_state(&fsm, src).await;
                    if walked {
                        let before = fsm.get_state().await;
                        let result = fsm.transition(tgt, "prop", vec![]).await;
                        prop_assert!(result.is_err());
                        prop_assert_eq!(fsm.get_state().await, before);
                    }
                    Ok(())
                })?;
            }
        }
    }

    /// Helper: walk FSM to a target state via a known path. Returns false if
    /// no path is known (CONFIGURING requires special handling).
    async fn walk_to_state(fsm: &Fsm, target: FsmState) -> bool {
        let path: &[FsmState] = match target {
            FsmState::Idle => &[],
            FsmState::Planning => &[FsmState::Planning],
            FsmState::FeasibilityProbe => &[FsmState::Planning, FsmState::FeasibilityProbe],
            FsmState::TestingFirst => &[FsmState::Planning, FsmState::TestingFirst],
            FsmState::Executing => &[
                FsmState::Planning,
                FsmState::TestingFirst,
                FsmState::Executing,
            ],
            FsmState::Auditing => &[
                FsmState::Planning,
                FsmState::TestingFirst,
                FsmState::Executing,
                FsmState::Auditing,
            ],
            FsmState::Memorizing => &[
                FsmState::Planning,
                FsmState::TestingFirst,
                FsmState::Executing,
                FsmState::Auditing,
                FsmState::Memorizing,
            ],
            FsmState::Complete => &[
                FsmState::Planning,
                FsmState::TestingFirst,
                FsmState::Executing,
                FsmState::Auditing,
                FsmState::Memorizing,
                FsmState::Complete,
            ],
            FsmState::Configuring => &[FsmState::Configuring],
            FsmState::Researching => &[FsmState::Researching],
        };

        for &step in path {
            if fsm.transition(step, "walk", vec![]).await.is_err() {
                return false;
            }
        }
        true
    }

    // -- Adversarial: evidence gates reject without artifacts --
    // (Evidence gate struct is 3A.3, but we test that transition records carry evidence)

    #[tokio::test]
    async fn test_transition_records_evidence() {
        let fsm = Fsm::new(None, 8000);
        let artifact = EvidenceArtifact {
            kind: "readiness".to_string(),
            summary: "knowledge engine READY".to_string(),
            data: vec![42],
        };

        let record = fsm
            .transition(FsmState::Planning, "p1", vec![artifact])
            .await
            .expect("valid");
        assert_eq!(record.evidence.len(), 1);
        assert_eq!(record.evidence[0].kind, "readiness");
        assert_eq!(record.evidence[0].data, vec![42]);

        let history = fsm.transition_history().await;
        assert_eq!(history.len(), 1);
        assert_eq!(history[0].evidence.len(), 1);
    }

    // -- Unit: RESEARCHING state traversal (A-22) --

    #[tokio::test]
    async fn test_researching_traversal() {
        let fsm = Fsm::new(None, 8000);

        // IDLE -> RESEARCHING
        fsm.transition(FsmState::Researching, "oracle", vec![])
            .await
            .expect("IDLE -> RESEARCHING");
        assert_eq!(fsm.get_state().await, FsmState::Researching);

        // RESEARCHING -> RESEARCHING (follow-up query)
        fsm.transition(FsmState::Researching, "oracle-followup", vec![])
            .await
            .expect("RESEARCHING -> RESEARCHING");
        assert_eq!(fsm.get_state().await, FsmState::Researching);

        // RESEARCHING -> IDLE (research complete, no code changes needed)
        fsm.transition(FsmState::Idle, "oracle-done", vec![])
            .await
            .expect("RESEARCHING -> IDLE");
        assert_eq!(fsm.get_state().await, FsmState::Idle);
    }

    #[tokio::test]
    async fn test_researching_escalate_to_planning() {
        let fsm = Fsm::new(None, 8000);

        // IDLE -> RESEARCHING -> PLANNING (research reveals code change needed)
        fsm.transition(FsmState::Researching, "oracle", vec![])
            .await
            .expect("IDLE -> RESEARCHING");
        fsm.transition(FsmState::Planning, "escalate", vec![])
            .await
            .expect("RESEARCHING -> PLANNING");
        assert_eq!(fsm.get_state().await, FsmState::Planning);
    }

    // -- Adversarial: RESEARCHING cannot reach EXECUTING/AUDITING directly --

    #[tokio::test]
    async fn test_researching_blocks_invalid_transitions() {
        let fsm = Fsm::new(None, 8000);
        fsm.transition(FsmState::Researching, "oracle", vec![])
            .await
            .unwrap();

        // RESEARCHING -> EXECUTING: must be invalid
        assert!(fsm
            .transition(FsmState::Executing, "bad", vec![])
            .await
            .is_err());
        assert_eq!(fsm.get_state().await, FsmState::Researching);

        // RESEARCHING -> AUDITING: must be invalid
        assert!(fsm
            .transition(FsmState::Auditing, "bad", vec![])
            .await
            .is_err());
        assert_eq!(fsm.get_state().await, FsmState::Researching);

        // RESEARCHING -> CONFIGURING: must be invalid
        assert!(fsm
            .transition(FsmState::Configuring, "bad", vec![])
            .await
            .is_err());
        assert_eq!(fsm.get_state().await, FsmState::Researching);
    }

    // -- Adversarial: replan log clears on phase boundary --

    #[tokio::test]
    async fn test_replan_log_clears_on_phase_boundary() {
        let fsm = Fsm::new(None, 8000);
        // Walk to EXECUTING
        fsm.transition(FsmState::Planning, "p1", vec![])
            .await
            .unwrap();
        fsm.transition(FsmState::TestingFirst, "p1", vec![])
            .await
            .unwrap();
        fsm.transition(FsmState::Executing, "p1", vec![])
            .await
            .unwrap();

        // Submit replan
        fsm.submit_replan(ReplanRequest {
            severity: ReplanSeverity::Minor,
            evidence: "test".to_string(),
            agent_id: "e1".to_string(),
            phase_label: "p1".to_string(),
        })
        .await
        .unwrap();

        // Walk through AUDITING -> MEMORIZING (this clears replan log)
        fsm.transition(FsmState::Auditing, "p1", vec![])
            .await
            .unwrap();
        fsm.transition(FsmState::Memorizing, "p1", vec![])
            .await
            .unwrap();

        // Go to next phase
        fsm.transition(FsmState::Planning, "p2", vec![])
            .await
            .unwrap();
        fsm.transition(FsmState::TestingFirst, "p2", vec![])
            .await
            .unwrap();
        fsm.transition(FsmState::Executing, "p2", vec![])
            .await
            .unwrap();

        // Should be able to replan again in new phase
        fsm.submit_replan(ReplanRequest {
            severity: ReplanSeverity::Major,
            evidence: "new phase".to_string(),
            agent_id: "e1".to_string(),
            phase_label: "p2".to_string(),
        })
        .await
        .expect("replan in new phase should succeed");
    }

    // -- B-2: SSB PhaseTransition emission tests --

    /// Verify that a wired SSB writer receives exactly one `phase_transition`
    /// row after a successful IDLE -> PLANNING transition.
    #[tokio::test]
    async fn test_transition_emits_phase_transition_ssb() {
        use crate::ssb::SsbConfig;
        use elai_core::OsClock;
        use tempfile::TempDir;

        let dir = TempDir::new().expect("tempdir");
        let config = SsbConfig {
            db_path: dir.path().join("test_ssb.db"),
            pool_size: Some(2),
            project_id: "test".into(),
            session_id: "s-test".into(),
        };
        let clock: Arc<dyn elai_core::ClockProvider> = Arc::new(OsClock);
        let ssb = Arc::new(crate::ssb::Ssb::new(config, clock).expect("ssb init"));

        let fsm = Fsm::new(None, 8000).with_ssb_writer(Arc::clone(&ssb));
        fsm.transition(FsmState::Planning, "plan-phase", vec![])
            .await
            .expect("transition should succeed");

        // Give the SSB async write task time to flush.
        tokio::time::sleep(std::time::Duration::from_millis(50)).await;

        // Read back via the viewer.
        let db_path = dir.path().join("test_ssb.db");
        let viewer = crate::ssb_viewer::SsbViewer::open(&db_path).expect("viewer open");
        let rows = viewer
            .recent_phase_transitions(5)
            .expect("recent_phase_transitions");
        assert_eq!(rows.len(), 1, "expected exactly 1 phase_transition row");
        let (_, from, to, _ts) = &rows[0];
        assert_eq!(from, "IDLE");
        assert_eq!(to, "PLANNING");
    }

    /// FSM transition must succeed even when no SSB writer is configured.
    #[tokio::test]
    async fn test_transition_succeeds_when_ssb_writer_none() {
        let fsm = Fsm::new(None, 8000);
        let result = fsm
            .transition(FsmState::Planning, "plan-phase", vec![])
            .await;
        assert!(result.is_ok(), "transition must succeed without ssb_writer: {result:?}");
    }
}
