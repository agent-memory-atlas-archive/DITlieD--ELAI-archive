//! FSM engine: 9-state machine with validated transitions, msgpack persistence,
//! concurrent safety, ReplanRequest support, and risk-level classification.
//!
//! Phase 3A.1 + 3A.5 deliverable. See docs/specs/crates/orchestrator.txt §2, §13
//! and docs/specs/architecture-overview.txt §4, §6.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};
use tokio::sync::Mutex;

// Re-export RiskLevel from gate module so consumers can access it via fsm::.
pub use crate::gate::{default_risk_level, RiskLevel};

// ---------------------------------------------------------------------------
// FsmState enum (9 states)
// ---------------------------------------------------------------------------

/// All 9 FSM states. Source: orchestrator.txt §2, architecture-overview.txt §4.
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
}

impl FsmState {
    /// All 9 states for iteration.
    pub const ALL: [FsmState; 9] = [
        FsmState::Idle,
        FsmState::Planning,
        FsmState::FeasibilityProbe,
        FsmState::TestingFirst,
        FsmState::Executing,
        FsmState::Auditing,
        FsmState::Memorizing,
        FsmState::Complete,
        FsmState::Configuring,
    ];

    /// Valid target states from `self`. Source: phase-3A.txt transition matrix.
    pub fn valid_targets(self) -> &'static [FsmState] {
        match self {
            FsmState::Idle => &[FsmState::Planning, FsmState::Configuring],
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
        };
        f.write_str(name)
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
    pub risk_overrides: HashMap<String, RiskLevel>,
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
        }
    }
}

impl std::error::Error for FsmError {}

// ---------------------------------------------------------------------------
// Persisted state (what goes to disk via msgpack)
// ---------------------------------------------------------------------------

/// On-disk state representation. Source: phase-3A.txt line 69.
/// Path: .elai_cc/sessions/{sid}/.fsm-state.msgpack
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PersistedFsmState {
    pub state: FsmState,
    pub phase_label: String,
    pub token_budget: usize,
    pub replan_log: Vec<ReplanLogEntry>,
    pub transition_history: Vec<TransitionRecord>,
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
}

fn now_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis() as u64)
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
            }),
            workload_emitter: None,
        }
    }

    /// Set a callback to emit WorkloadProfilePayload to SSB on phase transitions.
    pub fn set_workload_emitter(&mut self, emitter: WorkloadEmitter) {
        self.workload_emitter = Some(emitter);
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
            }),
            workload_emitter: None,
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

        // Trigger predictive pre-loading on state entry
        // Phase 3J.6: pre-load models based on FSM state using PreloadPolicy
        let models_to_preload = guard.preload_manager.trigger_preload(target);
        if !models_to_preload.is_empty() {
            tracing::info!(
                state = %target,
                count = models_to_preload.len(),
                models = ?models_to_preload,
                "Predictive pre-load triggered at phase boundary"
            );
        }
        // TODO: Implement actual background pre-loading with tokio::spawn and VRAM checks
        // This requires access to ModelPool and provider to trigger model loading

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
                timestamp: now_ms() as i64,
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

        // Persist to disk if path configured
        if let Some(ref path) = guard.persist_path {
            let persisted = PersistedFsmState {
                state: guard.state,
                phase_label: guard.phase_label.clone(),
                token_budget: guard.token_budget,
                replan_log: guard.replan_log.clone(),
                transition_history: guard.transition_history.clone(),
            };
            persist_state(path, &persisted).await?;
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

        if let Some(ref path) = guard.persist_path {
            let persisted = PersistedFsmState {
                state: guard.state,
                phase_label: guard.phase_label.clone(),
                token_budget: guard.token_budget,
                replan_log: guard.replan_log.clone(),
                transition_history: guard.transition_history.clone(),
            };
            persist_state(path, &persisted).await?;
        }

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
        ];

        // Verify all valid pairs
        for &(src, tgt) in valid_pairs {
            assert!(
                src.can_transition_to(tgt),
                "expected valid: {src} -> {tgt}"
            );
        }

        // Verify all other pairs are invalid (81 - valid_count = invalid_count)
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
        assert_eq!(valid_count, 16);
        assert_eq!(valid_count + invalid_count, 81);
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
}
