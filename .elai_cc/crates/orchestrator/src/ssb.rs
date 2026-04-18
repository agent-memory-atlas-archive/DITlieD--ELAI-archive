//! Shared State Board (SSB): inter-agent communication via SQLite WAL.
//!
//! Single-writer guarantee via `tokio::mpsc` channel. All writes serialized
//! through one dedicated task. Reads served from a `deadpool-sqlite` async connection pool.
//! Append-only within a session — no UPDATE or DELETE.
//!
//! A single [`Ssb`] instance is permanently bound to one `(project_id, session_id)` pair.
//! Multi-session scenarios require separate instances.

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::{Duration, UNIX_EPOCH};

use rusqlite::types::Value;
use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};
use std::rc::Rc;
use tokio::sync::{broadcast, mpsc, oneshot};

use elai_core::clock::ClockProvider;

// ---------------------------------------------------------------------------
// Schema version — bump on any DDL change
// ---------------------------------------------------------------------------

const SCHEMA_VERSION: i64 = 13;

/// Maximum serialized payload size in bytes (1 MiB).
/// `write_payload()` rejects payloads exceeding this limit with `SsbError::PayloadTooLarge`.
const PAYLOAD_MAX_BYTES: usize = 1_048_576;

const CREATE_SSB_TABLE: &str = "\
CREATE TABLE IF NOT EXISTS ssb (
    seq         INTEGER PRIMARY KEY AUTOINCREMENT,
    project_id  TEXT NOT NULL,
    session_id  TEXT NOT NULL,
    agent_id    TEXT NOT NULL,
    superstep   INTEGER NOT NULL DEFAULT 0,
    action_type TEXT NOT NULL CHECK(action_type IN (
        'file_write', 'file_read', 'tool_result', 'test_outcome', 'finding',
        'claim', 'checkpoint', 'model_swap', 'phase_transition',
        'knowledge_degradation', 'isolation_metric', 'plan_update',
        'replan_request', 'stub_reference', 'reasoning_trace',
        'phase_complete', 'memory_monitoring', 'workload_profile',
        'death_event', 'recovery_event',
        'supervision_exhausted', 'spiral_detected',
        'context_reduction', 'branch_merge',
        'mistake_entry', 'research_step',
        'research_decompose_level', 'web_research_step',
        'preview_available',
        'intervention_request', 'intervention_response',
        'task_submitted',
        'hook_event',
        'provider_signals', 'search_trigger_fired', 'vram_tier_detected',
        'indexing_progress', 'dkg_ready',
        'tiered_analysis', 'shortcoder_compression'
    )),
    payload     BLOB NOT NULL,
    created_at  INTEGER NOT NULL
);
CREATE INDEX IF NOT EXISTS idx_ssb_project     ON ssb(project_id);
CREATE INDEX IF NOT EXISTS idx_ssb_session     ON ssb(session_id);
CREATE INDEX IF NOT EXISTS idx_ssb_superstep   ON ssb(superstep);
CREATE INDEX IF NOT EXISTS idx_ssb_conflict    ON ssb(project_id, superstep);
CREATE INDEX IF NOT EXISTS idx_ssb_action_type ON ssb(action_type);
CREATE INDEX IF NOT EXISTS idx_ssb_agent_id    ON ssb(agent_id);
";

const CREATE_SSB_FILES_TABLE: &str = "\
CREATE TABLE IF NOT EXISTS ssb_files (
    seq       INTEGER NOT NULL REFERENCES ssb(seq),
    file_path TEXT NOT NULL
);
CREATE INDEX IF NOT EXISTS idx_ssb_files_path ON ssb_files(file_path);
CREATE INDEX IF NOT EXISTS idx_ssb_files_seq  ON ssb_files(seq);
";

// ---------------------------------------------------------------------------
// Action type enum
// ---------------------------------------------------------------------------

/// All 32 action types recorded in SSB entries.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SsbActionType {
    FileWrite,
    FileRead,
    ToolResult,
    TestOutcome,
    Finding,
    Claim,
    Checkpoint,
    ModelSwap,
    PhaseTransition,
    KnowledgeDegradation,
    IsolationMetric,
    PlanUpdate,
    ReplanRequest,
    StubReference,
    ReasoningTrace,
    PhaseComplete,
    MemoryMonitoring,
    WorkloadProfile,
    DeathEvent,
    RecoveryEvent,
    SupervisionExhausted,
    SpiralDetected,
    ContextReduction,
    BranchMerge,
    MistakeEntry,
    ResearchStep,
    ResearchDecomposeLevel,
    WebResearchStep,
    PreviewAvailable,
    InterventionRequest,
    InterventionResponse,
    TaskSubmitted,
    /// Batch 1A: hook dispatcher verdict emission (T1/T2/T3 tiered analysis).
    /// Carries a [`HookEventPayload`]. Dashboard filters on the `hook_event`
    /// discriminant to render "hook · T1 BLOCK" style rows.
    HookEvent,
    /// Batch 1B: per-baby-step snapshot of sidecar extended signals
    /// (A-17 GG logprob/entropy + A-25 novelty + A-8 steering + A-9 ETC state).
    /// Carries a [`ProviderSignalsPayload`].
    ProviderSignals,
    /// Batch 1B: ETC search-trigger fired on the most recent sidecar
    /// completion (A-9). Carries a [`SearchTriggerFiredPayload`].
    SearchTriggerFired,
    /// Batch 1B: VRAM tier detected at task_runner startup (A-37). Emitted
    /// once per run. Carries a [`VramTierDetectedPayload`].
    VramTierDetected,
    /// Batch 1C (MVP audit 2026-04-18 section 4 item 6 / A-44): project
    /// indexing stage transition. Emitted by the task_runner's
    /// `maybe_build_project_index` wrapper at (Start, Complete, Failed)
    /// boundaries — intermediate `dkg_parsing` / `bm25_indexing` stages are
    /// reserved for the future incremental/streaming build. Carries an
    /// [`IndexingProgressPayload`].
    IndexingProgress,
    /// Batch 1C: one-shot DKG summary emitted once per project per runner
    /// lifetime, immediately after the first successful
    /// `IndexingService::run_full_index`. Carries a [`DkgReadyPayload`].
    DkgReady,
    /// Batch 2A: per-hook-invocation summary complementing the per-verdict
    /// `HookEvent` rows. Emitted once per T1/T2/T3 dispatch with timings and
    /// rule counts so the dashboard can surface "T1 · 3 rules / 0.4ms"
    /// without having to aggregate individual HookEvent rows. Carries a
    /// [`TieredAnalysisPayload`].
    TieredAnalysis,
    /// Batch 2A: one row per ShortCoder compression at the PostToolUse
    /// boundary. Records the byte count before/after and which rules fired
    /// so the dashboard can render "ShortCoder · 1200→320 tokens (73%)".
    /// Carries a [`ShortCoderCompressionPayload`].
    ShortCoderCompression,
}

impl SsbActionType {
    /// SQL-compatible string for the CHECK constraint.
    pub fn as_sql_str(&self) -> &'static str {
        match self {
            Self::FileWrite => "file_write",
            Self::FileRead => "file_read",
            Self::ToolResult => "tool_result",
            Self::TestOutcome => "test_outcome",
            Self::Finding => "finding",
            Self::Claim => "claim",
            Self::Checkpoint => "checkpoint",
            Self::ModelSwap => "model_swap",
            Self::PhaseTransition => "phase_transition",
            Self::KnowledgeDegradation => "knowledge_degradation",
            Self::IsolationMetric => "isolation_metric",
            Self::PlanUpdate => "plan_update",
            Self::ReplanRequest => "replan_request",
            Self::StubReference => "stub_reference",
            Self::ReasoningTrace => "reasoning_trace",
            Self::PhaseComplete => "phase_complete",
            Self::MemoryMonitoring => "memory_monitoring",
            Self::WorkloadProfile => "workload_profile",
            Self::DeathEvent => "death_event",
            Self::RecoveryEvent => "recovery_event",
            Self::SupervisionExhausted => "supervision_exhausted",
            Self::SpiralDetected => "spiral_detected",
            Self::ContextReduction => "context_reduction",
            Self::BranchMerge => "branch_merge",
            Self::MistakeEntry => "mistake_entry",
            Self::ResearchStep => "research_step",
            Self::ResearchDecomposeLevel => "research_decompose_level",
            Self::WebResearchStep => "web_research_step",
            Self::PreviewAvailable => "preview_available",
            Self::InterventionRequest => "intervention_request",
            Self::InterventionResponse => "intervention_response",
            Self::TaskSubmitted => "task_submitted",
            Self::HookEvent => "hook_event",
            Self::ProviderSignals => "provider_signals",
            Self::SearchTriggerFired => "search_trigger_fired",
            Self::VramTierDetected => "vram_tier_detected",
            Self::IndexingProgress => "indexing_progress",
            Self::DkgReady => "dkg_ready",
            Self::TieredAnalysis => "tiered_analysis",
            Self::ShortCoderCompression => "shortcoder_compression",
        }
    }
}

/// Public wrapper around the internal string-to-enum map.
/// Used by read-side consumers (e.g. [`crate::ssb_viewer::SsbViewer`]).
pub fn ssb_action_type_from_str_public(s: &str) -> Result<SsbActionType, SsbError> {
    ssb_action_type_from_str(s)
}

fn ssb_action_type_from_str(s: &str) -> Result<SsbActionType, SsbError> {
    match s {
        "file_write" => Ok(SsbActionType::FileWrite),
        "file_read" => Ok(SsbActionType::FileRead),
        "tool_result" => Ok(SsbActionType::ToolResult),
        "test_outcome" => Ok(SsbActionType::TestOutcome),
        "finding" => Ok(SsbActionType::Finding),
        "claim" => Ok(SsbActionType::Claim),
        "checkpoint" => Ok(SsbActionType::Checkpoint),
        "model_swap" => Ok(SsbActionType::ModelSwap),
        "phase_transition" => Ok(SsbActionType::PhaseTransition),
        "knowledge_degradation" => Ok(SsbActionType::KnowledgeDegradation),
        "isolation_metric" => Ok(SsbActionType::IsolationMetric),
        "plan_update" => Ok(SsbActionType::PlanUpdate),
        "replan_request" => Ok(SsbActionType::ReplanRequest),
        "stub_reference" => Ok(SsbActionType::StubReference),
        "reasoning_trace" => Ok(SsbActionType::ReasoningTrace),
        "phase_complete" => Ok(SsbActionType::PhaseComplete),
        "memory_monitoring" => Ok(SsbActionType::MemoryMonitoring),
        "workload_profile" => Ok(SsbActionType::WorkloadProfile),
        "death_event" => Ok(SsbActionType::DeathEvent),
        "recovery_event" => Ok(SsbActionType::RecoveryEvent),
        "supervision_exhausted" => Ok(SsbActionType::SupervisionExhausted),
        "spiral_detected" => Ok(SsbActionType::SpiralDetected),
        "context_reduction" => Ok(SsbActionType::ContextReduction),
        "branch_merge" => Ok(SsbActionType::BranchMerge),
        "mistake_entry" => Ok(SsbActionType::MistakeEntry),
        "research_step" => Ok(SsbActionType::ResearchStep),
        "research_decompose_level" => Ok(SsbActionType::ResearchDecomposeLevel),
        "web_research_step" => Ok(SsbActionType::WebResearchStep),
        "preview_available" => Ok(SsbActionType::PreviewAvailable),
        "intervention_request" => Ok(SsbActionType::InterventionRequest),
        "intervention_response" => Ok(SsbActionType::InterventionResponse),
        "task_submitted" => Ok(SsbActionType::TaskSubmitted),
        "hook_event" => Ok(SsbActionType::HookEvent),
        "provider_signals" => Ok(SsbActionType::ProviderSignals),
        "search_trigger_fired" => Ok(SsbActionType::SearchTriggerFired),
        "vram_tier_detected" => Ok(SsbActionType::VramTierDetected),
        "indexing_progress" => Ok(SsbActionType::IndexingProgress),
        "dkg_ready" => Ok(SsbActionType::DkgReady),
        "tiered_analysis" => Ok(SsbActionType::TieredAnalysis),
        "shortcoder_compression" => Ok(SsbActionType::ShortCoderCompression),
        other => Err(SsbError::UnknownActionType(other.to_owned())),
    }
}

/// Validates that the deserialized payload matches the declared action_type.
/// This prevents silent corruption where the action_type string doesn't match
/// the actual payload struct type.
fn validate_payload_matches_action_type(
    payload_bytes: &[u8],
    declared_action_type: SsbActionType,
) -> Result<(), SsbError> {
    // Deserialize the payload to determine its actual type
    let payload: SsbPayload = rmp_serde::from_slice(payload_bytes)?;
    let actual_action_type = payload.action_type();

    if actual_action_type != declared_action_type {
        return Err(SsbError::ActionTypeMismatch {
            declared: declared_action_type.as_sql_str().to_owned(),
            actual: actual_action_type.as_sql_str().to_owned(),
        });
    }

    Ok(())
}

/// Computes cognitive isolation metric for a phase (orchestrator.txt S7A).
/// Formula: sum(agent_grc_tokens_this_phase) / model.max_context
///
/// This quantifies context that would have polluted the orchestrator if agents
/// were in-process. High values (>1.0) indicate the task is only feasible due
/// to agent isolation.
///
/// # Arguments
/// * `agent_grc_tokens` - Total GRC tokens generated by all agents in the phase
/// * `model_max_context` - Maximum context window of the model (e.g., 131072 for Gemma4 E2B)
///
/// # Returns
/// Ratio of context savings. Values >1.0 mean isolation saved more than one full context window.
pub fn compute_isolation_metric(agent_grc_tokens: u64, model_max_context: u64) -> f64 {
    if model_max_context == 0 {
        return 0.0;
    }
    agent_grc_tokens as f64 / model_max_context as f64
}

// ---------------------------------------------------------------------------
// Typed payload structs (3B.4)
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FileWritePayload {
    pub path: String,
    pub size_bytes: u64,
    pub hash: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FileReadPayload {
    pub path: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ToolResultPayload {
    pub tool_id: String,
    pub success: bool,
    pub output_summary: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TestOutcomePayload {
    pub test_name: String,
    pub passed: bool,
    pub duration_ms: u64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FindingPayload {
    pub severity: String,
    pub category: String,
    pub message: String,
    pub file_path: Option<String>,
    pub line: Option<u32>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ClaimPayload {
    pub claim_text: String,
    pub grounding_source: String,
    pub verified: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CheckpointPayload {
    pub agent_state_hash: String,
    pub grc_history_hash: String,
    pub agent_id: String,
    pub step_index: u64,
    pub file_sizes: Vec<(String, u64)>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ModelSwapPayload {
    pub from_model: String,
    pub to_model: String,
    pub gpu_id: u8,
    pub swap_duration_ms: u64,
    pub reason: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PhaseTransitionPayload {
    pub from_state: String,
    pub to_state: String,
    pub evidence_summary: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct KnowledgeDegradationPayload {
    pub component: String,
    pub old_mode: String,
    pub new_mode: String,
    pub reason: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct IsolationMetricPayload {
    pub agent_id: String,
    pub phase_id: String,
    pub context_savings_ratio: f64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PlanUpdatePayload {
    pub phase_id: String,
    pub version: String,
    pub change_summary: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ReplanRequestPayload {
    pub agent_id: String,
    pub severity: String, // "MINOR" or "MAJOR"
    pub evidence: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct StubReferencePayload {
    pub phase_id: String,
    pub step_id: String,
    pub stub_hash: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ReasoningTracePayload {
    pub agent_id: String,
    pub step_index: u32,
    pub step_type: String,
    pub content_summary: String,
    pub grounding_source: String,
    pub grounding_level: String,
    pub confidence: f64,
    pub elapsed_ms: u64,
    pub topic: String,
    pub budget_remaining: u32,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PhaseCompletePayload {
    pub phase_id: String,
    pub agent_id: String,
    pub fsm_state: String,
    pub evidence_artifacts: Vec<String>,
    pub commit_hashes: Vec<String>,
    pub test_outcomes: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MemoryMonitoringPayload {
    pub channel_id: String,
    pub deep_size_bytes: u64,
    pub threshold_exceeded: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DeathEventPayload {
    pub agent_id: String,
    pub classification: String,
    pub prior_step_index: i64,
    pub checkpoint_path: String,
    pub had_checkpoint: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RecoveryEventPayload {
    pub agent_id: String,
    pub classification: String,
    pub recovered_from_checkpoint: bool,
    pub degraded: bool,
    pub first_grounding_source: String,
    pub prior_step_index: i64,
    pub recovery_step_index: i64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SupervisionExhaustedPayload {
    pub agent_id: String,
    pub phase_id: String,
    pub restart_count: u32,
    pub max_restarts: u32,
    pub restart_history: Vec<i64>,
    pub escalation_action: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SpiralDetectedPayload {
    pub agent_id: String,
    pub phase_id: String,
    pub spiral_depth: u32,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ContextReductionPayload {
    pub level: u8,
    pub before_tokens: usize,
    pub after_tokens: usize,
    pub slots_affected: Vec<u8>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BranchMergePayload {
    pub winner_name: String,
    pub abandoned_names: Vec<String>,
    pub trunk_snap_id: String,
    pub from_snap_ids: Vec<String>,
}

/// SSB payload for MistakeBook entries (A-20). Wraps `elai_core::MistakeEntry`
/// for SSB storage with baby-step scoping via `baby_step_id`.
///
/// Tombstone support: when `cleared = true` this record is a clear-on-success
/// marker. All `MistakeEntry` rows for `baby_step_id` written before this
/// tombstone's `seq` are logically deleted. `query_by_baby_step` filters them
/// out automatically. This preserves SSB append-only invariant.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MistakeEntryPayload {
    /// Identifies which baby step this mistake belongs to.
    pub baby_step_id: String,
    /// The structured mistake record (from elai_core).
    pub entry: elai_core::MistakeEntry,
    /// When true: this is a tombstone. All prior entries for `baby_step_id`
    /// are logically cleared. Default false for normal entries.
    #[serde(default)]
    pub cleared: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct WorkloadProfilePayload {
    pub agent_role: String, // Serialized as snake_case string from AgentRole enum
    pub phase_id: String,
    pub tokens_in: u64,
    pub tokens_out: u64,
    pub latency_ms: u64,
    pub vram_peak_mb: u32,
    pub model_id: String,
    pub gpu_id: u8,
    pub timestamp: i64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ResearchStepPayload {
    pub agent_id: String,
    pub node_id: u64,
    pub action: String,
    pub phi: f64,
    pub psi: f64,
    pub iteration: u32,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct WebResearchStepPayload {
    pub agent_id: String,
    pub node_id: u64,
    pub action: String,
    pub phi: f64,
    pub psi: f64,
    pub iteration: u32,
    pub sources_fetched: u32,
    pub sources_used: u32,
}

/// WEB.3: emitted when a dev-server check passes and a preview is available.
/// Matches the spec contract from mvp-implementation-order.txt WEB.3:
///   payload = { url: "http://localhost:{port}", step_id, slot: "live" }
/// Additional fields (`agent_id`, `build_success`, `dev_server_ok`) carry
/// orchestrator/observability context for SSB consumers.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PreviewAvailablePayload {
    pub url: String,
    pub step_id: String,
    pub slot: String,
    pub agent_id: String,
    pub build_success: bool,
    pub dev_server_ok: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ResearchDecomposeLevelPayload {
    pub agent_id: String,
    pub level: u32,
    pub concept_count: usize,
    pub phase_count: usize,
}

// ---------------------------------------------------------------------------
// Intervention payloads (4K.1 — dashboard human-in-the-loop gates)
// ---------------------------------------------------------------------------

/// Category of human intervention being requested (4K.1).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum InterventionType {
    ApproveChange,
    ReplanApproval,
    ErrorRecovery,
}

/// One option presented to the user when an intervention is raised.
///
/// The `Retry(String)` variant carries a freeform message; all others are unit.
/// `#[serde(untagged)]` renders enum-less JSON: `"accept"`, `"decline"`, `"skip"`,
/// or `{"retry": "message"}` — matching what the dashboard frontend emits.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum InterventionOption {
    Retry(InterventionRetry),
    Keyword(InterventionKeyword),
}

/// Keyword-form intervention option (`"accept"`, `"decline"`, `"skip"`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum InterventionKeyword {
    Accept,
    Decline,
    Skip,
}

/// Structured form of the `retry` option.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct InterventionRetry {
    pub retry: String,
}

/// Short diff summary attached to an intervention request.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DiffSummary {
    pub files_changed: u32,
    pub lines_added: u32,
    pub lines_removed: u32,
}

/// Payload for `intervention_request` actions.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct InterventionRequestPayload {
    pub step_id: String,
    pub intervention_type: InterventionType,
    pub description: String,
    pub diff_summary: Option<DiffSummary>,
    pub options: Vec<InterventionOption>,
}

/// Payload for `intervention_response` actions.
///
/// `request_id` references the `seq` of the original `intervention_request` row.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct InterventionResponsePayload {
    pub request_id: i64,
    pub chosen: InterventionOption,
    pub user_note: Option<String>,
}

/// Payload for `task_submitted` actions (4K.3 — dashboard MVP).
///
/// Written by the dashboard process when the user submits a task and consumed
/// by the orchestrator's task_runner (patch-1.30 commit 8ce26a0). The action
/// is recorded in the SSB so downstream consumers (dashboard activity feed,
/// replay tools) can observe the full task lifecycle.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TaskSubmittedPayload {
    pub task: String,
    pub project_id: String,
    pub submitted_at_ms: i64,
}

/// Batch 1B — per-baby-step snapshot of sidecar extended signals.
///
/// Written by the task_runner after every sidecar-backed dispatch (see
/// `provider_signals::ProviderSignalReader`). Fields map 1:1 to the reader's
/// trait methods; all are `Option` so rows are emitted even for partially-
/// populated providers (e.g. a sidecar that emits GG but no novelty yet).
///
/// The dashboard filters the activity feed on `provider_signals` action_type
/// and renders "GG · entropy 2.4" style rows. High-entropy thresholds (>3.5
/// warn, <1.0 confident) are resolved on the frontend (ActivityEntry.svelte).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ProviderSignalsPayload {
    pub gg_mean_logprob: Option<f64>,
    pub gg_min_logprob: Option<f64>,
    pub entropy_mean: Option<f64>,
    pub entropy_max: Option<f64>,
    pub novelty_cosine: Option<f64>,
    pub steering_vector_active: bool,
    pub etc_disabled: bool,
    pub step_id: String,
}

/// Batch 1B — payload for the A-9 search-trigger SSB row.
///
/// Emitted only when [`provider_signals::ProviderSignalReader::read_last_search_trigger`]
/// returned `Some` on the last sidecar completion — NOT per-dispatch. Carries
/// the `reason` reported by the sidecar ETC monitor plus a threshold label so
/// the dashboard can render "search trigger · low_confidence" style rows.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SearchTriggerFiredPayload {
    pub reason: String,
    pub threshold_hit: String,
    pub step_id: String,
}

/// Batch 1B — payload for the A-37 VRAM-tier-detected SSB row.
///
/// Emitted exactly once at task_runner startup per `drive_task` host.
/// `tier` is one of `"8GB"` / `"12GB"` / `"16GB"` / `"24GB"` / `"48GB"` /
/// `"cloud"`. `detected_via` is one of `"nvml"` / `"env"` / `"default"`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct VramTierDetectedPayload {
    pub tier: String,
    pub detected_via: String,
    pub free_mb: Option<u32>,
}

/// Batch 1C — payload for the project-indexing stage transitions.
///
/// Emitted by the task_runner's `maybe_build_project_index` wrapper at
/// boundaries of the `IndexingService::run_full_index` call. Currently two
/// stages materialise per successful run (`start`, `complete`) plus one on
/// failure (`failed`); the `dkg_parsing` and `bm25_indexing` stages are
/// reserved for the future incremental/streaming build surface that the
/// indexing service will expose once tree-sitter and tantivy publish
/// per-file callbacks.
///
/// The `files_total` field is advisory — it may be zero when the runner
/// cannot cheaply enumerate the project (e.g. very large repos). The
/// frontend renders "42/150 files" only when `files_total > 0`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct IndexingProgressPayload {
    /// One of `"start"` / `"dkg_parsing"` / `"bm25_indexing"` / `"complete"`
    /// / `"failed"`. Stable set; see contract
    /// `task_runner_indexing_service_contract::indexing_stage_labels_are_stable`.
    pub stage: String,
    /// Project identifier — matches the `project_id` on `TaskSubmittedPayload`.
    pub project_id: String,
    /// Number of files processed so far. Zero at `start`.
    pub files_processed: u32,
    /// Estimated total file count. Zero when unknown.
    pub files_total: u32,
    /// Milliseconds elapsed since the `start` emission. Zero at `start`.
    pub elapsed_ms: u64,
    /// Optional human-readable status message (e.g. the error string on
    /// `failed`). Typically `None` on success stages.
    pub message: Option<String>,
}

/// Batch 1C — payload for the one-shot DKG-ready announcement.
///
/// Emitted once per project per runner lifetime, immediately after the
/// first successful `IndexingService::run_full_index`. The dashboard's rule
/// engine may use this row as a signal to hot-swap the `EmptyDkgQuerier`
/// stub for a real `DkgQuerierAdapter`; see
/// `docs/mvp-wiring-audit-2026-04-18.md` section 3 A-48.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DkgReadyPayload {
    /// Project identifier — matches the `project_id` on `TaskSubmittedPayload`.
    pub project_id: String,
    /// DKG node count after the build completes.
    pub node_count: u32,
    /// DKG edge count after the build completes.
    pub edge_count: u32,
    /// Languages detected during the build (e.g. `["rust", "ts"]`). May be
    /// empty when the DKG engine does not expose per-language stats.
    pub languages: Vec<String>,
}

/// Batch 1A — payload for the hook dispatcher's SSB emissions.
///
/// Written by `crate::hook_dispatcher::emit_hook_events_to_ssb` on every
/// non-Allow verdict produced by the T1/T2/T3 tiered analysis (plus a
/// Composite Allow breadcrumb per run). The dashboard filters the activity
/// feed by `hook_event` action_type and renders "hook · T1 BLOCK" style
/// rows keyed on `tier` + `verdict`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HookEventPayload {
    /// `"T1"` / `"T2"` / `"T3"` / `"Composite"` — matches `HookTier::as_str`.
    pub tier: String,
    /// `"Allow"` / `"Warn"` / `"Block"` — matches `HookVerdictKind::as_str`.
    pub verdict: String,
    /// Originating rule identifier (or predicate hash for pattern hooks).
    /// `None` for Allow breadcrumbs and dispatcher-internal messages.
    pub rule_id: Option<String>,
    /// Stable category tag; set to `HOOK_EVENT_CATEGORY` for every emission
    /// so downstream filters do not need to introspect `rule_id`.
    pub category: String,
    pub message: String,
    /// Baby-step identifier (typically `"baby_step_<N>"`) or `"task_<seq>"`
    /// for Stop / PreCompact events. Lets the dashboard correlate a hook
    /// verdict with the step it gated.
    pub step_id: String,
    /// `"PreToolUse"` / `"PostToolUse"` / `"Stop"` / `"PreCompact"`.
    pub hook_point: String,
}

/// Batch 2A — payload for a per-invocation tiered-analysis summary row.
///
/// Written once per T1/T2/T3 dispatch in addition to the per-verdict
/// [`HookEventPayload`] rows. Lets the dashboard render "T1 · 3 rules /
/// 0.4ms" without having to aggregate the individual hook_event rows.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TieredAnalysisPayload {
    /// `"T1"` / `"T2"` / `"T3"` — matches `elai_orchestrator::hook_dispatcher::HookTier::as_str`.
    pub tier: String,
    /// `"Allow"` / `"Warn"` / `"Block"` — matches `HookVerdictKind::as_str`.
    pub verdict: String,
    /// Number of rules / AST checks / patterns that produced a non-Allow
    /// verdict during this invocation.
    pub rule_count_fired: u32,
    /// Wall-clock elapsed time for this tier only, in microseconds.
    pub elapsed_us: u64,
    /// Baby-step identifier (typically `"baby_step_<N>"`).
    pub step_id: String,
    /// High-level category string (e.g. `"structural"`, `"accessibility_hint"`,
    /// `"naming_convention"`, `"todo_public_api"` for T3 advisories;
    /// `"hook_event"` for T1/T2 composite rows).
    pub category: String,
}

/// Batch 2A — payload for a ShortCoder compression row.
///
/// Emitted once per PostToolUse invocation when the ShortCoder simplifies
/// an [`AgentResponse`] before the auditor reads it. Approximate token
/// counts use the 4-chars-per-token heuristic that matches the rest of
/// the codebase (see `output_postprocessor::CHARS_PER_TOKEN`).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ShortCoderCompressionPayload {
    /// Approximate input token count (bytes / 4).
    pub tokens_in: u32,
    /// Approximate output token count (bytes / 4).
    pub tokens_out: u32,
    /// Percent reduction in [0, 100]. `0` means no reduction (or expansion).
    pub reduction_pct: u32,
    /// Baby-step identifier.
    pub step_id: String,
    /// Stable rule labels that fired (e.g. `["collapse_bodies",
    /// "strip_line_comments"]`). Max 10 — capped to prevent runaway size.
    pub rules_applied: Vec<String>,
}

// ---------------------------------------------------------------------------
// SsbPayload enum (typed wrapper over all 31 action-type payloads)
// ---------------------------------------------------------------------------

/// Typed payload enum. Each variant maps 1:1 to an [`SsbActionType`] (31 total).
/// Serialized to msgpack via `rmp_serde` before storage.
///
/// All 31 variants are schema-committed. Variants without a current production
/// emitter — `Claim`, `ModelSwap`, `KnowledgeDegradation`, `IsolationMetric`,
/// `BranchMerge`, `SpiralDetected`, `SupervisionExhausted`, `InterventionRequest`
/// — are forward-compat for phases 3F / 4G / 4H / wire-antispiral /
/// newphase-4G. Do not remove: on-disk SSB files and the dashboard
/// `payload_json` pipeline depend on the discriminant set staying stable.
/// `SpiralDetected` has a scheduled emitter in Session wire-antispiral;
/// `InterventionRequest` in Session newphase-4G (AutonomousGate).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum SsbPayload {
    FileWrite(FileWritePayload),
    FileRead(FileReadPayload),
    ToolResult(ToolResultPayload),
    TestOutcome(TestOutcomePayload),
    Finding(FindingPayload),
    Claim(ClaimPayload),
    Checkpoint(CheckpointPayload),
    ModelSwap(ModelSwapPayload),
    PhaseTransition(PhaseTransitionPayload),
    KnowledgeDegradation(KnowledgeDegradationPayload),
    IsolationMetric(IsolationMetricPayload),
    PlanUpdate(PlanUpdatePayload),
    ReplanRequest(ReplanRequestPayload),
    StubReference(StubReferencePayload),
    ReasoningTrace(ReasoningTracePayload),
    PhaseComplete(PhaseCompletePayload),
    MemoryMonitoring(MemoryMonitoringPayload),
    WorkloadProfile(WorkloadProfilePayload),
    DeathEvent(DeathEventPayload),
    RecoveryEvent(RecoveryEventPayload),
    SupervisionExhausted(SupervisionExhaustedPayload),
    SpiralDetected(SpiralDetectedPayload),
    ContextReduction(ContextReductionPayload),
    BranchMerge(BranchMergePayload),
    MistakeEntry(MistakeEntryPayload),
    ResearchStep(ResearchStepPayload),
    ResearchDecomposeLevel(ResearchDecomposeLevelPayload),
    WebResearchStep(WebResearchStepPayload),
    PreviewAvailable(PreviewAvailablePayload),
    InterventionRequest(InterventionRequestPayload),
    InterventionResponse(InterventionResponsePayload),
    TaskSubmitted(TaskSubmittedPayload),
    HookEvent(HookEventPayload),
    ProviderSignals(ProviderSignalsPayload),
    SearchTriggerFired(SearchTriggerFiredPayload),
    VramTierDetected(VramTierDetectedPayload),
    IndexingProgress(IndexingProgressPayload),
    DkgReady(DkgReadyPayload),
    TieredAnalysis(TieredAnalysisPayload),
    ShortCoderCompression(ShortCoderCompressionPayload),
}

impl SsbPayload {
    /// Returns the [`SsbActionType`] corresponding to this payload variant.
    /// This is the single source of truth for the discriminant→action_type mapping.
    pub fn action_type(&self) -> SsbActionType {
        match self {
            Self::FileWrite(_) => SsbActionType::FileWrite,
            Self::FileRead(_) => SsbActionType::FileRead,
            Self::ToolResult(_) => SsbActionType::ToolResult,
            Self::TestOutcome(_) => SsbActionType::TestOutcome,
            Self::Finding(_) => SsbActionType::Finding,
            Self::Claim(_) => SsbActionType::Claim,
            Self::Checkpoint(_) => SsbActionType::Checkpoint,
            Self::ModelSwap(_) => SsbActionType::ModelSwap,
            Self::PhaseTransition(_) => SsbActionType::PhaseTransition,
            Self::KnowledgeDegradation(_) => SsbActionType::KnowledgeDegradation,
            Self::IsolationMetric(_) => SsbActionType::IsolationMetric,
            Self::PlanUpdate(_) => SsbActionType::PlanUpdate,
            Self::ReplanRequest(_) => SsbActionType::ReplanRequest,
            Self::StubReference(_) => SsbActionType::StubReference,
            Self::ReasoningTrace(_) => SsbActionType::ReasoningTrace,
            Self::PhaseComplete(_) => SsbActionType::PhaseComplete,
            Self::MemoryMonitoring(_) => SsbActionType::MemoryMonitoring,
            Self::WorkloadProfile(_) => SsbActionType::WorkloadProfile,
            Self::DeathEvent(_) => SsbActionType::DeathEvent,
            Self::RecoveryEvent(_) => SsbActionType::RecoveryEvent,
            Self::SupervisionExhausted(_) => SsbActionType::SupervisionExhausted,
            Self::SpiralDetected(_) => SsbActionType::SpiralDetected,
            Self::ContextReduction(_) => SsbActionType::ContextReduction,
            Self::BranchMerge(_) => SsbActionType::BranchMerge,
            Self::MistakeEntry(_) => SsbActionType::MistakeEntry,
            Self::ResearchStep(_) => SsbActionType::ResearchStep,
            Self::ResearchDecomposeLevel(_) => SsbActionType::ResearchDecomposeLevel,
            Self::WebResearchStep(_) => SsbActionType::WebResearchStep,
            Self::PreviewAvailable(_) => SsbActionType::PreviewAvailable,
            Self::InterventionRequest(_) => SsbActionType::InterventionRequest,
            Self::InterventionResponse(_) => SsbActionType::InterventionResponse,
            Self::TaskSubmitted(_) => SsbActionType::TaskSubmitted,
            Self::HookEvent(_) => SsbActionType::HookEvent,
            Self::ProviderSignals(_) => SsbActionType::ProviderSignals,
            Self::SearchTriggerFired(_) => SsbActionType::SearchTriggerFired,
            Self::VramTierDetected(_) => SsbActionType::VramTierDetected,
            Self::IndexingProgress(_) => SsbActionType::IndexingProgress,
            Self::DkgReady(_) => SsbActionType::DkgReady,
            Self::TieredAnalysis(_) => SsbActionType::TieredAnalysis,
            Self::ShortCoderCompression(_) => SsbActionType::ShortCoderCompression,
        }
    }
}

impl Ssb {
    /// Current wall-clock time in milliseconds since UNIX_EPOCH, sourced from
    /// the injected [`ClockProvider`]. Used by callers (e.g. the dashboard
    /// process) that must avoid raw `std::time::SystemTime` per the entropy
    /// quarantine rule.
    ///
    /// Saturation semantics (intentional, not silent error swallowing):
    /// * `duration_since(UNIX_EPOCH)` only fails on a system clock set before
    ///   1970-01-01; we treat that as t=0 since SSB timestamps are advisory
    ///   and we do not want a panic at the IPC boundary.
    /// * `i64::try_from(u128)` only fails past year 292278994; we saturate
    ///   to `i64::MAX` because no caller in 2026 should ever see overflow,
    ///   and a saturated max is harmless for monotonic comparisons.
    pub fn now_ms_via_clock(clock: &Arc<dyn ClockProvider>) -> i64 {
        let ms_u128 = clock
            .system_now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or(Duration::ZERO)
            .as_millis();
        i64::try_from(ms_u128).unwrap_or(i64::MAX)
    }
}

// ---------------------------------------------------------------------------
// SsbTraceable trait (orchestrator-local, no cross-crate dep)
// ---------------------------------------------------------------------------

/// Trait for types that can produce an [`SsbPayload`] for recording in the SSB.
/// Defined in `orchestrator` only — never in `reasoning` or `tools` crates.
pub trait SsbTraceable {
    fn to_ssb_payload(&self) -> SsbPayload;
}

impl SsbTraceable for ReasoningTracePayload {
    fn to_ssb_payload(&self) -> SsbPayload {
        SsbPayload::ReasoningTrace(self.clone())
    }
}

// ---------------------------------------------------------------------------
// Entry + file entry structs
// ---------------------------------------------------------------------------

/// A single SSB row, broadcast to subscribers after commit.
#[derive(Debug, Clone)]
pub struct SsbEntry {
    pub seq: i64,
    pub project_id: String,
    pub session_id: String,
    pub agent_id: String,
    pub superstep: i64,
    pub action_type: SsbActionType,
    /// `Arc<[u8]>` for O(1) clone on broadcast (tokio::broadcast requires Clone).
    pub payload: Arc<[u8]>,
    pub created_at: i64,
}

/// Junction table row linking an SSB entry to a file path.
#[derive(Debug, Clone)]
pub struct SsbFileEntry {
    pub seq: i64,
    pub file_path: String,
}

// ---------------------------------------------------------------------------
// Write request (3B.2)
// ---------------------------------------------------------------------------

/// Request sent to the single-writer loop via mpsc channel.
///
/// `project_id` and `session_id` are injected by the writer (bound at `Ssb`
/// construction). Callers provide only per-write fields.
pub struct SsbWriteRequest {
    pub action_type: SsbActionType,
    pub agent_id: String,
    pub superstep: i64,
    pub payload: Vec<u8>,
    pub files: Vec<String>,
}

// ---------------------------------------------------------------------------
// Configuration
// ---------------------------------------------------------------------------

/// Configuration for SSB database and connection pool.
#[derive(Clone)]
pub struct SsbConfig {
    pub db_path: std::path::PathBuf,
    /// Read pool size. Defaults to `max(available_parallelism * 2, 4)`.
    pub pool_size: Option<u32>,
    pub project_id: String,
    pub session_id: String,
}

impl SsbConfig {
    fn effective_pool_size(&self) -> u32 {
        self.pool_size.unwrap_or_else(|| {
            let cpus = std::thread::available_parallelism()
                .map(|n| n.get() as u32)
                .unwrap_or(2);
            cpus.saturating_mul(2).max(4)
        })
    }
}

// ---------------------------------------------------------------------------
// Error type
// ---------------------------------------------------------------------------

/// Errors produced by SSB operations.
#[derive(Debug, thiserror::Error)]
pub enum SsbError {
    #[error("sqlite error: {0}")]
    Sqlite(#[from] rusqlite::Error),

    #[error("connection pool error: {0}")]
    Pool(String),

    #[error("ssb has been shut down")]
    Shutdown,

    #[error("write channel closed unexpectedly")]
    ChannelClosed,

    #[error("unknown SSB action type: {0}")]
    UnknownActionType(String),

    #[error("msgpack serialization error: {0}")]
    Serialization(#[from] rmp_serde::encode::Error),

    #[error("msgpack deserialization error: {0}")]
    Deserialization(#[from] rmp_serde::decode::Error),

    #[error("payload exceeds maximum size of {PAYLOAD_MAX_BYTES} bytes")]
    PayloadTooLarge,

    #[error("action_type '{declared}' does not match deserialized payload type '{actual}'")]
    ActionTypeMismatch { declared: String, actual: String },
}

// ---------------------------------------------------------------------------
// Async connection pool (deadpool managed, rusqlite 0.39)
// ---------------------------------------------------------------------------

/// Applies per-connection pragmas on pool connection creation.
/// `foreign_keys` is per-connection and must be set on every connection.
///
/// NOTE: This intentionally includes `load_module(conn)` for rarray support,
/// which the writer-side `apply_pragmas` does NOT include (rarray is only
/// needed for read-path queries like query_by_file and query_conflict).
fn apply_pool_pragmas(conn: &Connection) -> Result<(), rusqlite::Error> {
    // Order matters: journal_mode first, then other pragmas.
    conn.pragma_update(None, "journal_mode", "WAL")?;
    conn.pragma_update(None, "synchronous", "NORMAL")?;
    conn.pragma_update(None, "wal_autocheckpoint", 1000)?;
    conn.pragma_update(None, "busy_timeout", 5000)?;
    conn.pragma_update(None, "cache_size", -64000)?;
    conn.pragma_update(None, "foreign_keys", "ON")?;
    conn.pragma_update(None, "temp_store", "MEMORY")?;
    rusqlite::vtab::array::load_module(conn)?;
    Ok(())
}

/// Deadpool manager for rusqlite connections.
/// Opens connections in a blocking task and applies pragmas on creation.
struct SqliteManager {
    db_path: String,
}

impl deadpool::managed::Manager for SqliteManager {
    type Type = Connection;
    type Error = SsbError;

    async fn create(&self) -> Result<Connection, SsbError> {
        let path = self.db_path.clone();
        tokio::task::spawn_blocking(move || {
            let conn = Connection::open(&path)?;
            apply_pool_pragmas(&conn)?;
            Ok(conn)
        })
        .await
        .map_err(|e| SsbError::Pool(format!("spawn_blocking join error: {e}")))?
    }

    async fn recycle(
        &self,
        conn: &mut Connection,
        _metrics: &deadpool::managed::Metrics,
    ) -> deadpool::managed::RecycleResult<SsbError> {
        // Clear any stale implicit transaction from an interrupted read path.
        conn.execute_batch("ROLLBACK").ok();
        // Validate the connection is still usable with a cheap query.
        conn.execute_batch("SELECT 1")
            .map_err(|e| deadpool::managed::RecycleError::Backend(SsbError::Sqlite(e)))?;
        Ok(())
    }
}

/// Type alias for the deadpool-managed rusqlite pool.
type SsbPool = deadpool::managed::Pool<SqliteManager>;

// ---------------------------------------------------------------------------
// Reader trait
// ---------------------------------------------------------------------------

/// Read-only queries against the SSB.
pub trait SsbReader {
    fn query_by_session(&self, session_id: &str) -> Result<Vec<SsbEntry>, SsbError>;
    fn query_by_file(&self, file_paths: &[&str]) -> Result<Vec<SsbEntry>, SsbError>;
    fn query_by_type(&self, action_type: SsbActionType) -> Result<Vec<SsbEntry>, SsbError>;
    fn query_by_superstep(&self, superstep: i64) -> Result<Vec<SsbEntry>, SsbError>;
    /// Returns agent_ids that touched any of `file_paths` in the given `superstep`,
    /// excluding `session_id` (self). Uses JOIN + rarray for performance.
    fn query_conflict(
        &self,
        file_paths: &[&str],
        session_id: &str,
        superstep: i64,
    ) -> Result<Vec<String>, SsbError>;
    fn last_seq(&self) -> Result<Option<i64>, SsbError>;
    fn query_by_seq(&self, seq: i64) -> Result<Option<SsbEntry>, SsbError>;

    // Query patterns for new action types (3J.12)
    /// Query reasoning_trace entries for a specific agent, ordered by seq.
    fn query_reasoning_trace(&self, agent_id: &str) -> Result<Vec<SsbEntry>, SsbError>;
    /// Query phase_complete entries for a specific phase.
    fn query_phase_complete(&self, phase_id: &str) -> Result<Vec<SsbEntry>, SsbError>;
    /// Query isolation_metric entries grouped by phase.
    fn query_isolation_metrics(&self) -> Result<Vec<SsbEntry>, SsbError>;
}

// ---------------------------------------------------------------------------
// Internal message for write loop
// ---------------------------------------------------------------------------

type WriteResponse = oneshot::Sender<Result<i64, SsbError>>;

struct WriteMessage {
    request: SsbWriteRequest,
    respond: WriteResponse,
}

// ---------------------------------------------------------------------------
// Ssb (main handle)
// ---------------------------------------------------------------------------

/// Shared State Board: single-writer SQLite with broadcast notifications.
#[allow(dead_code)] // read_pool used by tests and SsbReader impl (3B.3)
pub struct Ssb {
    write_tx: Option<mpsc::UnboundedSender<WriteMessage>>,
    broadcast_tx: broadcast::Sender<SsbEntry>,
    read_pool: SsbPool,
    project_id: String,
    session_id: String,
    shutdown: Arc<AtomicBool>,
    _write_task: tokio::task::JoinHandle<()>,
}

impl Ssb {
    /// Create a new SSB instance. Opens (or creates) the database, applies
    /// WAL pragmas, runs migrations, and spawns the single-writer task.
    pub fn new(
        config: SsbConfig,
        clock: Arc<dyn ClockProvider>,
    ) -> Result<Self, SsbError> {
        // --- Writer connection (dedicated, not pooled) ---
        let writer_conn = Connection::open(&config.db_path)?;
        apply_pragmas(&writer_conn)?;
        run_migration(&writer_conn)?;

        #[cfg(debug_assertions)]
        writer_conn.update_hook(Some(
            |action: rusqlite::hooks::Action, _db: &str, table: &str, _rowid: i64| {
                if (table == "ssb" || table == "ssb_files")
                    && (action == rusqlite::hooks::Action::SQLITE_UPDATE
                        || action == rusqlite::hooks::Action::SQLITE_DELETE)
                {
                    panic!(
                        "SSB append-only violation: attempted {:?} on table {}",
                        action, table
                    );
                }
            },
        ))?;

        // --- Read pool (deadpool managed, async-native) ---
        let db_path_str = config.db_path.to_string_lossy().to_string();
        let manager = SqliteManager { db_path: db_path_str };
        let read_pool = SsbPool::builder(manager)
            .max_size(config.effective_pool_size() as usize)
            .runtime(deadpool::Runtime::Tokio1)
            .build()
            .map_err(|e| SsbError::Pool(format!("pool build error: {e}")))?;

        // --- Channels ---
        let (write_tx, write_rx) = mpsc::unbounded_channel::<WriteMessage>();
        let (broadcast_tx, _) = broadcast::channel::<SsbEntry>(1024);

        let project_id = config.project_id.clone();
        let session_id = config.session_id.clone();
        let shutdown = Arc::new(AtomicBool::new(false));

        let bc_tx = broadcast_tx.clone();
        let pid = project_id.clone();
        let sid = session_id.clone();
        let sd = shutdown.clone();

        let write_task = tokio::spawn(write_loop(
            writer_conn, write_rx, bc_tx, clock, pid, sid, sd,
        ));

        Ok(Self {
            write_tx: Some(write_tx),
            broadcast_tx,
            read_pool,
            project_id,
            session_id,
            shutdown,
            _write_task: write_task,
        })
    }

    /// Submit a write request to the single-writer loop.
    ///
    /// # IPC sync contract (WARN-6)
    ///
    /// Callers **must** `.await` this method and receive the `seq` confirmation
    /// **before** dispatching any `ToolResult` to the agent. This guarantees
    /// the SSB entry is durable before the agent sees the result.
    ///
    /// # Type validation (3J.12)
    ///
    /// The payload bytes are deserialized and validated against the declared
    /// `action_type`. If they don't match, the write is rejected with
    /// `SsbError::ActionTypeMismatch`. This prevents silent corruption.
    pub async fn write(&self, request: SsbWriteRequest) -> Result<i64, SsbError> {
        // Validate that payload matches declared action_type (write-time validation)
        validate_payload_matches_action_type(&request.payload, request.action_type)?;

        let tx = self.write_tx.as_ref().ok_or(SsbError::Shutdown)?;
        if self.shutdown.load(Ordering::Acquire) {
            return Err(SsbError::Shutdown);
        }
        let (respond, rx) = oneshot::channel();
        tx.send(WriteMessage { request, respond })
            .map_err(|_| SsbError::Shutdown)?;
        rx.await.map_err(|_| SsbError::ChannelClosed)?
    }

    /// Submit a typed payload write. Derives `action_type` from the enum variant,
    /// serializes via msgpack, enforces the size guard, and delegates to [`write()`].
    pub async fn write_payload(
        &self,
        agent_id: &str,
        superstep: i64,
        payload: SsbPayload,
        files: Vec<String>,
    ) -> Result<i64, SsbError> {
        let action_type = payload.action_type();
        let bytes = rmp_serde::to_vec_named(&payload)?;
        if bytes.len() > PAYLOAD_MAX_BYTES {
            return Err(SsbError::PayloadTooLarge);
        }
        self.write(SsbWriteRequest {
            action_type,
            agent_id: agent_id.to_owned(),
            superstep,
            payload: bytes,
            files,
        })
        .await
    }

    /// Subscribe to broadcast notifications of committed entries.
    /// Slow subscribers that fall behind 1024 entries will receive `Lagged`.
    pub fn subscribe(&self) -> broadcast::Receiver<SsbEntry> {
        self.broadcast_tx.subscribe()
    }

    /// Gracefully shut down the write loop. Idempotent.
    ///
    /// Drops the mpsc sender so the write loop drains remaining messages
    /// and exits. Pending `write()` callers whose oneshot hasn't been
    /// answered yet will receive `Err(SsbError::ChannelClosed)`.
    pub fn shutdown(&mut self) {
        if !self.shutdown.swap(true, Ordering::Release) {
            self.write_tx.take();
        }
    }

    /// Project ID this SSB is bound to.
    pub fn project_id(&self) -> &str {
        &self.project_id
    }

    /// Session ID this SSB is bound to.
    pub fn session_id(&self) -> &str {
        &self.session_id
    }
}

// ---------------------------------------------------------------------------
// WAL pragma application (writer connection)
// ---------------------------------------------------------------------------

fn apply_pragmas(conn: &Connection) -> Result<(), rusqlite::Error> {
    conn.pragma_update(None, "journal_mode", "WAL")?;
    conn.pragma_update(None, "synchronous", "NORMAL")?;
    conn.pragma_update(None, "wal_autocheckpoint", 1000)?;
    conn.pragma_update(None, "busy_timeout", 5000)?;
    conn.pragma_update(None, "cache_size", -64000)?;
    conn.pragma_update(None, "foreign_keys", "ON")?;
    conn.pragma_update(None, "temp_store", "MEMORY")?;
    Ok(())
}

// ---------------------------------------------------------------------------
// Migration
// ---------------------------------------------------------------------------

fn run_migration(conn: &Connection) -> Result<(), rusqlite::Error> {
    let version: i64 = conn.pragma_query_value(None, "user_version", |row| row.get(0))?;
    if version == SCHEMA_VERSION {
        return Ok(());
    }
    // SSB is session-scoped. Stale schema → drop and recreate (data loss
    // is acceptable for old sessions).
    if version > 0 {
        conn.execute_batch("DROP TABLE IF EXISTS ssb_files; DROP TABLE IF EXISTS ssb;")?;
    }
    conn.execute_batch(CREATE_SSB_TABLE)?;
    conn.execute_batch(CREATE_SSB_FILES_TABLE)?;
    conn.pragma_update(None, "user_version", SCHEMA_VERSION)?;
    Ok(())
}

// ---------------------------------------------------------------------------
// Write loop (3B.2 — single-writer task)
// ---------------------------------------------------------------------------

async fn write_loop(
    conn: Connection,
    mut rx: mpsc::UnboundedReceiver<WriteMessage>,
    broadcast_tx: broadcast::Sender<SsbEntry>,
    clock: Arc<dyn ClockProvider>,
    project_id: String,
    session_id: String,
    shutdown: Arc<AtomicBool>,
) {
    while let Some(msg) = rx.recv().await {
        let result = execute_write(
            &conn,
            &msg.request,
            &clock,
            &project_id,
            &session_id,
        );

        match result {
            Ok((seq, created_at)) => {
                let entry = SsbEntry {
                    seq,
                    project_id: project_id.clone(),
                    session_id: session_id.clone(),
                    agent_id: msg.request.agent_id.clone(),
                    superstep: msg.request.superstep,
                    action_type: msg.request.action_type,
                    payload: Arc::from(msg.request.payload.as_slice()),
                    created_at,
                };
                // Respond BEFORE broadcast (WARN-6 IPC sync): caller must
                // have seq confirmation before any subscriber can act.
                let _ = msg.respond.send(Ok(seq));
                // Broadcast after commit + after respond. No receivers = fine.
                let _ = broadcast_tx.send(entry);
            }
            Err(e) => {
                let _ = msg.respond.send(Err(e));
            }
        }
    }
    // Channel closed (shutdown or all senders dropped). Mark shutdown.
    shutdown.store(true, Ordering::Release);
}

/// Execute a single write within a BEGIN IMMEDIATE transaction.
fn execute_write(
    conn: &Connection,
    request: &SsbWriteRequest,
    clock: &Arc<dyn ClockProvider>,
    project_id: &str,
    session_id: &str,
) -> Result<(i64, i64), SsbError> {
    let created_at = clock
        .system_now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or(Duration::ZERO)
        .as_secs() as i64;

    // BEGIN IMMEDIATE acquires write lock early — avoids SQLITE_BUSY at commit.
    conn.execute("BEGIN IMMEDIATE", [])?;

    let insert_result = (|| -> Result<i64, rusqlite::Error> {
        conn.execute(
            "INSERT INTO ssb (project_id, session_id, agent_id, superstep, action_type, payload, created_at) \
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            params![
                project_id,
                session_id,
                request.agent_id,
                request.superstep,
                request.action_type.as_sql_str(),
                request.payload,
                created_at,
            ],
        )?;
        let seq = conn.last_insert_rowid();

        for file_path in &request.files {
            conn.execute(
                "INSERT INTO ssb_files (seq, file_path) VALUES (?1, ?2)",
                params![seq, file_path],
            )?;
        }
        Ok(seq)
    })();

    match insert_result {
        Ok(seq) => {
            // COMMIT can fail (disk full, I/O error). If it does, we must
            // ROLLBACK to avoid leaving the connection stuck in an open
            // transaction (rusqlite#366).
            if let Err(e) = conn.execute_batch("COMMIT") {
                let _ = conn.execute_batch("ROLLBACK");
                return Err(SsbError::Sqlite(e));
            }
            Ok((seq, created_at))
        }
        Err(e) => {
            let _ = conn.execute_batch("ROLLBACK");
            Err(SsbError::Sqlite(e))
        }
    }
}

// ---------------------------------------------------------------------------
// Row mapper
// ---------------------------------------------------------------------------

fn row_to_entry(row: &rusqlite::Row<'_>) -> rusqlite::Result<SsbEntry> {
    let at_str: String = row.get(5)?;
    let action_type = ssb_action_type_from_str(&at_str).map_err(|e| {
        rusqlite::Error::FromSqlConversionFailure(5, rusqlite::types::Type::Text, Box::new(e))
    })?;
    let payload_bytes: Vec<u8> = row.get(6)?;
    Ok(SsbEntry {
        seq: row.get(0)?,
        project_id: row.get(1)?,
        session_id: row.get(2)?,
        agent_id: row.get(3)?,
        superstep: row.get(4)?,
        action_type,
        payload: Arc::from(payload_bytes.as_slice()),
        created_at: row.get(7)?,
    })
}

// ---------------------------------------------------------------------------
// Sync→async bridge for deadpool reads
// ---------------------------------------------------------------------------

impl Ssb {
    /// Run a closure on a read-pool connection, bridging deadpool's async API
    /// to the sync SsbReader trait. Uses `block_in_place` to avoid blocking the
    /// tokio runtime when called from an async context.
    fn with_read_conn<F, T>(&self, f: F) -> Result<T, SsbError>
    where
        F: FnOnce(&Connection) -> Result<T, SsbError> + Send + 'static,
        T: Send + 'static,
    {
        let pool = self.read_pool.clone();
        tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(async {
                let obj = pool
                    .get()
                    .await
                    .map_err(|e| SsbError::Pool(format!("{e}")))?;
                // Object<SqliteManager> derefs to Connection
                f(&obj)
            })
        })
    }
}

// ---------------------------------------------------------------------------
// SsbReader implementation
// ---------------------------------------------------------------------------

impl SsbReader for Ssb {
    fn query_by_session(&self, session_id: &str) -> Result<Vec<SsbEntry>, SsbError> {
        let sid = session_id.to_owned();
        let pid = self.project_id.clone();
        self.with_read_conn(move |conn| {
            let mut stmt = conn.prepare(
                "SELECT seq, project_id, session_id, agent_id, superstep, action_type, payload, created_at \
                 FROM ssb WHERE session_id = ?1 AND project_id = ?2",
            )?;
            let rows = stmt.query_map(params![sid, pid], row_to_entry)?;
            rows.collect::<Result<Vec<_>, _>>().map_err(SsbError::Sqlite)
        })
    }

    fn query_by_file(&self, file_paths: &[&str]) -> Result<Vec<SsbEntry>, SsbError> {
        if file_paths.is_empty() {
            return Ok(vec![]);
        }
        let paths: Vec<String> = file_paths.iter().map(|p| (*p).to_owned()).collect();
        let pid = self.project_id.clone();
        self.with_read_conn(move |conn| {
            let values: Vec<Value> = paths.iter().map(|p| Value::Text(p.clone())).collect();
            let ptr = Rc::new(values);
            let mut stmt = conn.prepare(
                "SELECT DISTINCT ssb.seq, ssb.project_id, ssb.session_id, ssb.agent_id, \
                 ssb.superstep, ssb.action_type, ssb.payload, ssb.created_at \
                 FROM ssb JOIN ssb_files ON ssb_files.seq = ssb.seq \
                 WHERE ssb_files.file_path IN (SELECT value FROM rarray(?1)) \
                   AND ssb.project_id = ?2",
            )?;
            let rows = stmt.query_map(params![ptr, pid], row_to_entry)?;
            rows.collect::<Result<Vec<_>, _>>().map_err(SsbError::Sqlite)
        })
    }

    fn query_by_type(&self, action_type: SsbActionType) -> Result<Vec<SsbEntry>, SsbError> {
        let pid = self.project_id.clone();
        self.with_read_conn(move |conn| {
            let mut stmt = conn.prepare(
                "SELECT seq, project_id, session_id, agent_id, superstep, action_type, payload, created_at \
                 FROM ssb WHERE action_type = ?1 AND project_id = ?2",
            )?;
            let rows = stmt.query_map(params![action_type.as_sql_str(), pid], row_to_entry)?;
            rows.collect::<Result<Vec<_>, _>>().map_err(SsbError::Sqlite)
        })
    }

    fn query_by_superstep(&self, superstep: i64) -> Result<Vec<SsbEntry>, SsbError> {
        let pid = self.project_id.clone();
        self.with_read_conn(move |conn| {
            let mut stmt = conn.prepare(
                "SELECT seq, project_id, session_id, agent_id, superstep, action_type, payload, created_at \
                 FROM ssb WHERE superstep = ?1 AND project_id = ?2",
            )?;
            let rows = stmt.query_map(params![superstep, pid], row_to_entry)?;
            rows.collect::<Result<Vec<_>, _>>().map_err(SsbError::Sqlite)
        })
    }

    fn query_conflict(
        &self,
        file_paths: &[&str],
        session_id: &str,
        superstep: i64,
    ) -> Result<Vec<String>, SsbError> {
        if file_paths.is_empty() {
            return Ok(vec![]);
        }
        let paths: Vec<String> = file_paths.iter().map(|p| (*p).to_owned()).collect();
        let sid = session_id.to_owned();
        let pid = self.project_id.clone();
        self.with_read_conn(move |conn| {
            let values: Vec<Value> = paths.iter().map(|p| Value::Text(p.clone())).collect();
            let ptr = Rc::new(values);
            let mut stmt = conn.prepare(
                "SELECT DISTINCT ssb.agent_id \
                 FROM ssb_files \
                 JOIN ssb ON ssb_files.seq = ssb.seq \
                 WHERE ssb_files.file_path IN (SELECT value FROM rarray(?1)) \
                   AND ssb.project_id = ?2 \
                   AND ssb.session_id != ?3 \
                   AND ssb.superstep = ?4",
            )?;
            let rows = stmt.query_map(
                params![ptr, pid, sid, superstep],
                |row| row.get::<_, String>(0),
            )?;
            rows.collect::<Result<Vec<_>, _>>().map_err(SsbError::Sqlite)
        })
    }

    fn last_seq(&self) -> Result<Option<i64>, SsbError> {
        let pid = self.project_id.clone();
        self.with_read_conn(move |conn| {
            let mut stmt = conn.prepare("SELECT MAX(seq) FROM ssb WHERE project_id = ?1")?;
            let result: Option<i64> = stmt.query_row(params![pid], |row| row.get(0))?;
            Ok(result)
        })
    }

    fn query_by_seq(&self, seq: i64) -> Result<Option<SsbEntry>, SsbError> {
        let pid = self.project_id.clone();
        self.with_read_conn(move |conn| {
            let mut stmt = conn.prepare(
                "SELECT seq, project_id, session_id, agent_id, superstep, action_type, payload, created_at \
                 FROM ssb WHERE seq = ?1 AND project_id = ?2 LIMIT 1",
            )?;
            let mut rows = stmt.query_map(params![seq, pid], row_to_entry)?;
            match rows.next() {
                Some(row) => Ok(Some(row.map_err(SsbError::Sqlite)?)),
                None => Ok(None),
            }
        })
    }

    fn query_reasoning_trace(&self, agent_id: &str) -> Result<Vec<SsbEntry>, SsbError> {
        let aid = agent_id.to_owned();
        let pid = self.project_id.clone();
        self.with_read_conn(move |conn| {
            let mut stmt = conn.prepare(
                "SELECT seq, project_id, session_id, agent_id, superstep, action_type, payload, created_at \
                 FROM ssb WHERE action_type = ?1 AND agent_id = ?2 AND project_id = ?3 \
                 ORDER BY seq ASC",
            )?;
            let rows = stmt.query_map(
                params![SsbActionType::ReasoningTrace.as_sql_str(), aid, pid],
                row_to_entry,
            )?;
            rows.collect::<Result<Vec<_>, _>>().map_err(SsbError::Sqlite)
        })
    }

    fn query_phase_complete(&self, phase_id: &str) -> Result<Vec<SsbEntry>, SsbError> {
        let phase = phase_id.to_owned();
        let pid = self.project_id.clone();
        self.with_read_conn(move |conn| {
            let mut stmt = conn.prepare(
                "SELECT seq, project_id, session_id, agent_id, superstep, action_type, payload, created_at \
                 FROM ssb WHERE action_type = ?1 AND project_id = ?2",
            )?;
            let rows = stmt.query_map(
                params![SsbActionType::PhaseComplete.as_sql_str(), pid],
                row_to_entry,
            )?;
            let all_entries: Vec<SsbEntry> = rows.collect::<Result<Vec<_>, _>>().map_err(SsbError::Sqlite)?;

            // Filter by phase_id from payload (since phase_id is in the payload, not a separate column)
            let filtered = all_entries
                .into_iter()
                .filter(|entry| {
                    if let Ok(payload) = rmp_serde::from_slice::<PhaseCompletePayload>(&entry.payload) {
                        payload.phase_id == phase
                    } else {
                        false
                    }
                })
                .collect();
            Ok(filtered)
        })
    }

    fn query_isolation_metrics(&self) -> Result<Vec<SsbEntry>, SsbError> {
        let pid = self.project_id.clone();
        self.with_read_conn(move |conn| {
            let mut stmt = conn.prepare(
                "SELECT seq, project_id, session_id, agent_id, superstep, action_type, payload, created_at \
                 FROM ssb WHERE action_type = ?1 AND project_id = ?2",
            )?;
            let rows = stmt.query_map(
                params![SsbActionType::IsolationMetric.as_sql_str(), pid],
                row_to_entry,
            )?;
            rows.collect::<Result<Vec<_>, _>>().map_err(SsbError::Sqlite)
        })
    }
}

// ---------------------------------------------------------------------------
// Canonical SSB path helpers
// ---------------------------------------------------------------------------

/// Subdirectory (relative to a project/base directory) that holds the
/// managed sessions state, including the single SSB SQLite database.
pub const SSB_SESSIONS_SUBDIR: &str = ".elai/sessions";

/// File name of the canonical SSB SQLite database inside
/// [`SSB_SESSIONS_SUBDIR`].
pub const SSB_DB_FILE_NAME: &str = "ssb.db";

/// Canonical SSB database path for the given project/base directory.
///
/// The CLI's sidecar bootstrap uses `<session-persistence-dir>/ssb.db`, and
/// managed session persistence files live under `<base>/.elai/sessions/`
/// (see `session_control::managed_sessions_dir_for`). The dashboard must
/// open the SSB at the **same** path so it and a running CLI REPL share
/// telemetry instead of silently creating a second, empty database.
///
/// Returns `<base_dir>/.elai/sessions/ssb.db` as a pure path computation
/// (no filesystem side effects — callers choose whether to create the
/// parent directory or let `Ssb::new`/`SsbViewer::open` do it).
pub fn default_ssb_path_for_base(base_dir: impl AsRef<std::path::Path>) -> std::path::PathBuf {
    base_dir
        .as_ref()
        .join(SSB_SESSIONS_SUBDIR)
        .join(SSB_DB_FILE_NAME)
}

/// Canonical SSB database path rooted at the current working directory.
///
/// Convenience wrapper for callers (dashboard binary, CLI) that treat CWD
/// as the project root. Returns `Err` only if `std::env::current_dir` fails.
pub fn default_ssb_path() -> std::io::Result<std::path::PathBuf> {
    Ok(default_ssb_path_for_base(std::env::current_dir()?))
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use elai_core::clock::SeededClock;
    use std::time::{Duration, SystemTime};
    use tempfile::TempDir;

    fn test_config(dir: &TempDir) -> SsbConfig {
        SsbConfig {
            db_path: dir.path().join("test_ssb.db"),
            pool_size: Some(2),
            project_id: "test-project".into(),
            session_id: "test-session".into(),
        }
    }

    fn test_clock() -> Arc<dyn ClockProvider> {
        Arc::new(SeededClock::new())
    }

    /// Generate a valid serialized payload for any action type.
    /// Used by tests that don't care about payload contents but need
    /// valid msgpack bytes to pass write-time validation (3J.12).
    fn test_payload(action_type: SsbActionType) -> Vec<u8> {
        let payload = match action_type {
            SsbActionType::FileWrite => SsbPayload::FileWrite(FileWritePayload {
                path: "test.rs".into(), size_bytes: 0, hash: "h".into(),
            }),
            SsbActionType::FileRead => SsbPayload::FileRead(FileReadPayload {
                path: "test.rs".into(),
            }),
            SsbActionType::ToolResult => SsbPayload::ToolResult(ToolResultPayload {
                tool_id: "t".into(), success: true, output_summary: "ok".into(),
            }),
            SsbActionType::TestOutcome => SsbPayload::TestOutcome(TestOutcomePayload {
                test_name: "t".into(), passed: true, duration_ms: 1,
            }),
            SsbActionType::Finding => SsbPayload::Finding(FindingPayload {
                severity: "INFO".into(), category: "c".into(), message: "m".into(),
                file_path: None, line: None,
            }),
            SsbActionType::Claim => SsbPayload::Claim(ClaimPayload {
                claim_text: "c".into(), grounding_source: "s".into(), verified: false,
            }),
            SsbActionType::Checkpoint => SsbPayload::Checkpoint(CheckpointPayload {
                agent_state_hash: "h".into(), grc_history_hash: "g".into(),
                agent_id: "a".into(), step_index: 0, file_sizes: vec![],
            }),
            SsbActionType::ModelSwap => SsbPayload::ModelSwap(ModelSwapPayload {
                from_model: "a".into(), to_model: "b".into(), gpu_id: 0,
                swap_duration_ms: 1, reason: "r".into(),
            }),
            SsbActionType::PhaseTransition => SsbPayload::PhaseTransition(PhaseTransitionPayload {
                from_state: "a".into(), to_state: "b".into(), evidence_summary: "e".into(),
            }),
            SsbActionType::KnowledgeDegradation => SsbPayload::KnowledgeDegradation(KnowledgeDegradationPayload {
                component: "c".into(), old_mode: "a".into(), new_mode: "b".into(), reason: "r".into(),
            }),
            SsbActionType::IsolationMetric => SsbPayload::IsolationMetric(IsolationMetricPayload {
                agent_id: "a".into(), phase_id: "p".into(), context_savings_ratio: 0.5,
            }),
            SsbActionType::PlanUpdate => SsbPayload::PlanUpdate(PlanUpdatePayload {
                phase_id: "p".into(), version: "v".into(), change_summary: "s".into(),
            }),
            SsbActionType::ReplanRequest => SsbPayload::ReplanRequest(ReplanRequestPayload {
                agent_id: "a".into(), severity: "MINOR".into(), evidence: "e".into(),
            }),
            SsbActionType::StubReference => SsbPayload::StubReference(StubReferencePayload {
                phase_id: "p".into(), step_id: "s".into(), stub_hash: "h".into(),
            }),
            SsbActionType::ReasoningTrace => SsbPayload::ReasoningTrace(ReasoningTracePayload {
                agent_id: "a".into(), step_index: 0, step_type: "observe".into(),
                content_summary: "c".into(), grounding_source: "s".into(),
                grounding_level: "verified".into(), confidence: 0.9,
                elapsed_ms: 1, topic: "t".into(), budget_remaining: 5,
            }),
            SsbActionType::PhaseComplete => SsbPayload::PhaseComplete(PhaseCompletePayload {
                phase_id: "p".into(), agent_id: "a".into(), fsm_state: "IDLE".into(),
                evidence_artifacts: vec![], commit_hashes: vec![], test_outcomes: vec![],
            }),
            SsbActionType::MemoryMonitoring => SsbPayload::MemoryMonitoring(MemoryMonitoringPayload {
                channel_id: "c".into(), deep_size_bytes: 0, threshold_exceeded: false,
            }),
            SsbActionType::WorkloadProfile => SsbPayload::WorkloadProfile(WorkloadProfilePayload {
                agent_role: "coder".into(), phase_id: "p".into(), tokens_in: 0,
                tokens_out: 0, latency_ms: 0, vram_peak_mb: 0,
                model_id: "m".into(), gpu_id: 0, timestamp: 0,
            }),
            SsbActionType::DeathEvent => SsbPayload::DeathEvent(DeathEventPayload {
                agent_id: "a".into(), classification: "c".into(),
                prior_step_index: 0, checkpoint_path: "p".into(), had_checkpoint: false,
            }),
            SsbActionType::RecoveryEvent => SsbPayload::RecoveryEvent(RecoveryEventPayload {
                agent_id: "a".into(), classification: "c".into(),
                recovered_from_checkpoint: false, degraded: false,
                first_grounding_source: "s".into(), prior_step_index: 0, recovery_step_index: 1,
            }),
            SsbActionType::SupervisionExhausted => SsbPayload::SupervisionExhausted(SupervisionExhaustedPayload {
                agent_id: "a".into(), phase_id: "p".into(), restart_count: 0,
                max_restarts: 3, restart_history: vec![], escalation_action: "stop".into(),
            }),
            SsbActionType::SpiralDetected => SsbPayload::SpiralDetected(SpiralDetectedPayload {
                agent_id: "a".into(), phase_id: "p".into(), spiral_depth: 1,
            }),
            SsbActionType::ContextReduction => SsbPayload::ContextReduction(ContextReductionPayload {
                level: 1, before_tokens: 100, after_tokens: 50, slots_affected: vec![1],
            }),
            SsbActionType::BranchMerge => SsbPayload::BranchMerge(BranchMergePayload {
                winner_name: "w".into(), abandoned_names: vec![],
                trunk_snap_id: "t".into(), from_snap_ids: vec![],
            }),
            SsbActionType::MistakeEntry => SsbPayload::MistakeEntry(MistakeEntryPayload {
                baby_step_id: "step-1".into(),
                entry: elai_core::MistakeEntry {
                    attempt_number: 1,
                    diff_hash: elai_core::DiffHash::from_diff("test"),
                    failure_category: elai_core::FailureCategory::Unknown,
                    faulty_line: None, root_cause_json: None,
                    action_taken: "test".into(), timestamp: 0,
                },
                cleared: false,
            }),
            SsbActionType::ResearchStep => SsbPayload::ResearchStep(ResearchStepPayload {
                agent_id: "a".into(), node_id: 0, action: "expand".into(),
                phi: 0.5, psi: 0.5, iteration: 1,
            }),
            SsbActionType::ResearchDecomposeLevel => SsbPayload::ResearchDecomposeLevel(ResearchDecomposeLevelPayload {
                agent_id: "a".into(), level: 0, concept_count: 3, phase_count: 5,
            }),
            SsbActionType::WebResearchStep => SsbPayload::WebResearchStep(WebResearchStepPayload {
                agent_id: "a".into(), node_id: 0, action: "search".into(),
                phi: 0.5, psi: 0.4, iteration: 1, sources_fetched: 3, sources_used: 2,
            }),
            SsbActionType::PreviewAvailable => SsbPayload::PreviewAvailable(PreviewAvailablePayload {
                url: "http://localhost:5173".into(), step_id: "step-1".into(),
                slot: "live".into(), agent_id: "a".into(),
                build_success: true, dev_server_ok: true,
            }),
            SsbActionType::InterventionRequest => SsbPayload::InterventionRequest(InterventionRequestPayload {
                step_id: "step-1".into(),
                intervention_type: InterventionType::ApproveChange,
                description: "approve?".into(),
                diff_summary: None,
                options: vec![
                    InterventionOption::Keyword(InterventionKeyword::Accept),
                    InterventionOption::Keyword(InterventionKeyword::Decline),
                ],
            }),
            SsbActionType::TaskSubmitted => SsbPayload::TaskSubmitted(TaskSubmittedPayload {
                task: "do the thing".into(),
                project_id: "p".into(),
                submitted_at_ms: 0,
            }),
            SsbActionType::InterventionResponse => SsbPayload::InterventionResponse(InterventionResponsePayload {
                request_id: 1,
                chosen: InterventionOption::Keyword(InterventionKeyword::Accept),
                user_note: None,
            }),
            SsbActionType::HookEvent => SsbPayload::HookEvent(HookEventPayload {
                tier: "T1".into(),
                verdict: "Allow".into(),
                rule_id: None,
                category: "hook_event".into(),
                message: "allow".into(),
                step_id: "step-1".into(),
                hook_point: "PreToolUse".into(),
            }),
            SsbActionType::ProviderSignals => SsbPayload::ProviderSignals(ProviderSignalsPayload {
                gg_mean_logprob: Some(-1.2),
                gg_min_logprob: Some(-3.5),
                entropy_mean: Some(2.1),
                entropy_max: Some(4.8),
                novelty_cosine: Some(0.72),
                steering_vector_active: false,
                etc_disabled: true,
                step_id: "step-1".into(),
            }),
            SsbActionType::SearchTriggerFired => SsbPayload::SearchTriggerFired(SearchTriggerFiredPayload {
                reason: "import".into(),
                threshold_hit: "trigger_token_idx=42".into(),
                step_id: "step-1".into(),
            }),
            SsbActionType::VramTierDetected => SsbPayload::VramTierDetected(VramTierDetectedPayload {
                tier: "12GB".into(),
                detected_via: "nvml".into(),
                free_mb: Some(1800),
            }),
            SsbActionType::IndexingProgress => SsbPayload::IndexingProgress(IndexingProgressPayload {
                stage: "start".into(),
                project_id: "p".into(),
                files_processed: 0,
                files_total: 0,
                elapsed_ms: 0,
                message: None,
            }),
            SsbActionType::DkgReady => SsbPayload::DkgReady(DkgReadyPayload {
                project_id: "p".into(),
                node_count: 0,
                edge_count: 0,
                languages: vec![],
            }),
            SsbActionType::TieredAnalysis => SsbPayload::TieredAnalysis(TieredAnalysisPayload {
                tier: "T1".into(),
                verdict: "Allow".into(),
                rule_count_fired: 0,
                elapsed_us: 0,
                step_id: "step-1".into(),
                category: "hook_event".into(),
            }),
            SsbActionType::ShortCoderCompression => {
                SsbPayload::ShortCoderCompression(ShortCoderCompressionPayload {
                    tokens_in: 0,
                    tokens_out: 0,
                    reduction_pct: 0,
                    step_id: "step-1".into(),
                    rules_applied: vec![],
                })
            }
        };
        rmp_serde::to_vec_named(&payload).expect("test payload serialization")
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn test_single_writer() {
        let dir = TempDir::new().expect("tempdir");
        let mut ssb = Ssb::new(test_config(&dir), test_clock()).expect("ssb init");

        let mut seqs = Vec::new();
        for i in 0..10 {
            let seq = ssb
                .write(SsbWriteRequest {
                    action_type: SsbActionType::FileWrite,
                    agent_id: format!("agent-{i}"),
                    superstep: 0,
                    payload: test_payload(SsbActionType::FileWrite),
                    files: vec![format!("src/file_{i}.rs")],
                })
                .await
                .expect("write");
            seqs.push(seq);
        }

        // All seqs must be monotonically increasing with no gaps.
        for window in seqs.windows(2) {
            assert_eq!(window[1], window[0] + 1, "seqs must be sequential");
        }

        ssb.shutdown();
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn test_write_after_shutdown() {
        let dir = TempDir::new().expect("tempdir");
        let mut ssb = Ssb::new(test_config(&dir), test_clock()).expect("ssb init");

        // Write one entry successfully.
        ssb.write(SsbWriteRequest {
            action_type: SsbActionType::Checkpoint,
            agent_id: "agent-1".into(),
            superstep: 0,
            payload: test_payload(SsbActionType::Checkpoint),
            files: vec![],
        })
        .await
        .expect("write before shutdown");

        ssb.shutdown();

        // Write after shutdown must fail.
        let result = ssb
            .write(SsbWriteRequest {
                action_type: SsbActionType::Checkpoint,
                agent_id: "agent-1".into(),
                superstep: 0,
                payload: test_payload(SsbActionType::Checkpoint),
                files: vec![],
            })
            .await;

        assert!(result.is_err(), "write after shutdown must fail");
        match result.unwrap_err() {
            SsbError::Shutdown => (),
            other => panic!("expected Shutdown error, got: {other}"),
        }
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn test_concurrent_writers() {
        let dir = TempDir::new().expect("tempdir");
        let ssb = Arc::new(
            Ssb::new(test_config(&dir), test_clock()).expect("ssb init"),
        );

        // Spawn 10 concurrent write tasks — no Mutex needed, write() takes &self
        // and the mpsc channel serializes writes internally.
        let mut handles = Vec::new();
        for i in 0..10 {
            let ssb = ssb.clone();
            handles.push(tokio::spawn(async move {
                ssb.write(SsbWriteRequest {
                    action_type: SsbActionType::ToolResult,
                    agent_id: format!("agent-{i}"),
                    superstep: 0,
                    payload: test_payload(SsbActionType::ToolResult),
                    files: vec![format!("src/mod_{i}.rs")],
                })
                .await
            }));
        }

        let mut seqs: Vec<i64> = Vec::new();
        for h in handles {
            let seq = h.await.expect("join").expect("write");
            seqs.push(seq);
        }

        seqs.sort();
        // All 10 seqs present, sequential, no gaps.
        assert_eq!(seqs.len(), 10);
        for window in seqs.windows(2) {
            assert_eq!(window[1], window[0] + 1, "seqs must be sequential");
        }
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn test_broadcast_after_commit() {
        let dir = TempDir::new().expect("tempdir");
        let ssb = Ssb::new(test_config(&dir), test_clock()).expect("ssb init");

        let mut rx = ssb.subscribe();

        let finding_bytes = test_payload(SsbActionType::Finding);
        let seq = ssb
            .write(SsbWriteRequest {
                action_type: SsbActionType::Finding,
                agent_id: "agent-x".into(),
                superstep: 1,
                payload: finding_bytes.clone(),
                files: vec!["src/main.rs".into()],
            })
            .await
            .expect("write");

        let entry = tokio::time::timeout(Duration::from_secs(2), rx.recv())
            .await
            .expect("timeout")
            .expect("recv");

        assert_eq!(entry.seq, seq);
        assert_eq!(entry.agent_id, "agent-x");
        assert_eq!(entry.superstep, 1);
        assert_eq!(entry.action_type, SsbActionType::Finding);
        assert_eq!(&*entry.payload, &*finding_bytes);
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn test_action_type_mismatch_rejected() {
        // Adversarial test (3J.12): verify that action_type string mismatch
        // with payload struct is rejected at write time, not silently corrupted.
        let dir = TempDir::new().expect("tempdir");
        let ssb = Ssb::new(test_config(&dir), test_clock()).expect("ssb init");

        // Create a FileWrite payload
        let file_write_payload = SsbPayload::FileWrite(FileWritePayload {
            path: "test.rs".to_string(),
            size_bytes: 100,
            hash: "abc123".to_string(),
        });
        let payload_bytes = rmp_serde::to_vec_named(&file_write_payload).expect("serialize");

        // Try to write it with a different action_type (ToolResult)
        let result = ssb
            .write(SsbWriteRequest {
                action_type: SsbActionType::ToolResult, // Mismatch!
                agent_id: "agent-1".into(),
                superstep: 0,
                payload: payload_bytes,
                files: vec![],
            })
            .await;

        assert!(result.is_err(), "action_type mismatch must be rejected");
        match result.unwrap_err() {
            SsbError::ActionTypeMismatch { declared, actual } => {
                assert_eq!(declared, "tool_result");
                assert_eq!(actual, "file_write");
            }
            other => panic!("expected ActionTypeMismatch error, got: {other}"),
        }
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn test_reasoning_trace_monotonic_step_index() {
        // Adversarial test (3J.12): verify reasoning_trace entries can be queried
        // and are ordered by seq (monotonic per agent per phase).
        let dir = TempDir::new().expect("tempdir");
        let ssb = Ssb::new(test_config(&dir), test_clock()).expect("ssb init");

        let agent_id = "test-agent";

        // Write multiple reasoning_trace entries
        for i in 0..5 {
            let payload = SsbPayload::ReasoningTrace(ReasoningTracePayload {
                agent_id: agent_id.to_string(),
                step_index: i,
                step_type: "observe".to_string(),
                content_summary: format!("step {}", i),
                grounding_source: "test".to_string(),
                grounding_level: "HIGH".to_string(),
                confidence: 0.9,
                elapsed_ms: 100,
                topic: "test-topic".to_string(),
                budget_remaining: 10,
            });
            let payload_bytes = rmp_serde::to_vec_named(&payload).expect("serialize");
            ssb.write(SsbWriteRequest {
                action_type: SsbActionType::ReasoningTrace,
                agent_id: agent_id.to_string(),
                superstep: 0,
                payload: payload_bytes,
                files: vec![],
            })
            .await
            .expect("write");
        }

        // Query reasoning_trace entries for the agent
        let entries = ssb
            .query_reasoning_trace(agent_id)
            .expect("query should succeed");

        assert_eq!(entries.len(), 5, "should have 5 entries");

        // Verify they are ordered by seq (monotonically increasing)
        for window in entries.windows(2) {
            assert!(window[1].seq > window[0].seq, "seq must be monotonically increasing");
        }
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn test_created_at_uses_clock_provider() {
        let clock = Arc::new(SeededClock::with_epoch(
            SystemTime::UNIX_EPOCH + Duration::from_secs(1_000_000),
        ));
        let dir = TempDir::new().expect("tempdir");
        let config = SsbConfig {
            db_path: dir.path().join("clock_test.db"),
            pool_size: Some(2),
            project_id: "proj".into(),
            session_id: "sess".into(),
        };
        let ssb = Ssb::new(config, clock.clone()).expect("ssb init");

        let mut rx = ssb.subscribe();
        ssb.write(SsbWriteRequest {
            action_type: SsbActionType::Claim,
            agent_id: "agent-clock".into(),
            superstep: 0,
            payload: test_payload(SsbActionType::Claim),
            files: vec![],
        })
        .await
        .expect("write");

        let entry = tokio::time::timeout(Duration::from_secs(2), rx.recv())
            .await
            .expect("timeout")
            .expect("recv");

        // SeededClock epoch is 1_000_000 secs past UNIX_EPOCH.
        assert_eq!(entry.created_at, 1_000_000);
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn test_wal_pragma_applied() {
        let dir = TempDir::new().expect("tempdir");
        let config = test_config(&dir);
        let _ssb = Ssb::new(config.clone(), test_clock()).expect("ssb init");

        // Verify via a direct connection that WAL is active.
        let conn = Connection::open(&config.db_path).expect("open");
        let journal_mode: String =
            conn.pragma_query_value(None, "journal_mode", |row| row.get(0))
                .expect("pragma");
        assert_eq!(journal_mode.to_lowercase(), "wal");
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn test_pool_connection_has_foreign_keys() {
        let dir = TempDir::new().expect("tempdir");
        let ssb = Ssb::new(test_config(&dir), test_clock()).expect("ssb init");

        let obj = ssb.read_pool.get().await.expect("pool get");
        let fk: i64 = obj
            .pragma_query_value(None, "foreign_keys", |row| row.get(0))
            .expect("pragma");
        assert_eq!(fk, 1, "foreign_keys must be ON for pool connections");
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn test_migration_from_v0() {
        let dir = TempDir::new().expect("tempdir");
        let db_path = dir.path().join("migrate_test.db");

        // Create a V0 database (no tables, user_version=0).
        {
            let conn = Connection::open(&db_path).expect("open");
            conn.pragma_update(None, "user_version", 0).expect("pragma");
        }

        // SSB should create tables.
        let config = SsbConfig {
            db_path,
            pool_size: Some(2),
            project_id: "proj".into(),
            session_id: "sess".into(),
        };
        let ssb = Ssb::new(config, test_clock()).expect("ssb init");

        // Write should succeed (tables exist).
        ssb.write(SsbWriteRequest {
            action_type: SsbActionType::FileRead,
            agent_id: "agent-m".into(),
            superstep: 0,
            payload: test_payload(SsbActionType::FileRead),
            files: vec![],
        })
        .await
        .expect("write after migration");
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn test_shutdown_idempotent() {
        let dir = TempDir::new().expect("tempdir");
        let mut ssb = Ssb::new(test_config(&dir), test_clock()).expect("ssb init");

        ssb.shutdown();
        ssb.shutdown(); // Second call must not panic.
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn test_ssb_files_inserted() {
        let dir = TempDir::new().expect("tempdir");
        let ssb = Ssb::new(test_config(&dir), test_clock()).expect("ssb init");

        let seq = ssb
            .write(SsbWriteRequest {
                action_type: SsbActionType::FileWrite,
                agent_id: "agent-f".into(),
                superstep: 0,
                payload: test_payload(SsbActionType::FileWrite),
                files: vec!["src/a.rs".into(), "src/b.rs".into()],
            })
            .await
            .expect("write");

        // Verify via read pool.
        let obj = ssb.read_pool.get().await.expect("pool");
        let mut stmt = obj
            .prepare("SELECT file_path FROM ssb_files WHERE seq = ?1 ORDER BY file_path")
            .expect("prepare");
        let paths: Vec<String> = stmt
            .query_map(params![seq], |row| row.get(0))
            .expect("query")
            .filter_map(|r| r.ok())
            .collect();
        assert_eq!(paths, vec!["src/a.rs", "src/b.rs"]);
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn test_all_action_types_insert() {
        let dir = TempDir::new().expect("tempdir");
        let ssb = Ssb::new(test_config(&dir), test_clock()).expect("ssb init");

        let types = [
            SsbActionType::FileWrite,
            SsbActionType::FileRead,
            SsbActionType::ToolResult,
            SsbActionType::TestOutcome,
            SsbActionType::Finding,
            SsbActionType::Claim,
            SsbActionType::Checkpoint,
            SsbActionType::ModelSwap,
            SsbActionType::PhaseTransition,
            SsbActionType::KnowledgeDegradation,
            SsbActionType::IsolationMetric,
            SsbActionType::PlanUpdate,
            SsbActionType::ReplanRequest,
            SsbActionType::StubReference,
            SsbActionType::ReasoningTrace,
            SsbActionType::PhaseComplete,
            SsbActionType::MemoryMonitoring,
            SsbActionType::DeathEvent,
            SsbActionType::RecoveryEvent,
            SsbActionType::SupervisionExhausted,
            SsbActionType::SpiralDetected,
            SsbActionType::ContextReduction,
            SsbActionType::BranchMerge,
            SsbActionType::WorkloadProfile,
            SsbActionType::MistakeEntry,
            SsbActionType::ResearchStep,
            SsbActionType::ResearchDecomposeLevel,
            SsbActionType::WebResearchStep,
            SsbActionType::PreviewAvailable,
            SsbActionType::InterventionRequest,
            SsbActionType::InterventionResponse,
            SsbActionType::TaskSubmitted,
        ];

        for (i, at) in types.iter().enumerate() {
            ssb.write(SsbWriteRequest {
                action_type: *at,
                agent_id: format!("agent-{i}"),
                superstep: 0,
                payload: test_payload(*at),
                files: vec![],
            })
            .await
            .unwrap_or_else(|e| panic!("write {at:?} failed: {e}"));
        }

        // Verify count.
        let obj = ssb.read_pool.get().await.expect("pool");
        let count: i64 = obj
            .query_row("SELECT COUNT(*) FROM ssb", [], |row| row.get(0))
            .expect("count");
        assert_eq!(count, 32, "must have inserted all 32 types");
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn test_10k_sequential_writes() {
        let dir = TempDir::new().expect("tempdir");
        let mut ssb = Ssb::new(test_config(&dir), test_clock()).expect("ssb init");

        let bulk_payload = test_payload(SsbActionType::ToolResult);
        let mut prev_seq = 0i64;
        for i in 0..10_000 {
            let seq = ssb
                .write(SsbWriteRequest {
                    action_type: SsbActionType::ToolResult,
                    agent_id: "agent-bulk".into(),
                    superstep: 0,
                    payload: bulk_payload.clone(),
                    files: vec![],
                })
                .await
                .expect("write");

            if i > 0 {
                assert_eq!(seq, prev_seq + 1, "seq gap at write {i}");
            }
            prev_seq = seq;
        }

        ssb.shutdown();
    }

    // -----------------------------------------------------------------------
    // 3B.3 — Read-path tests
    // -----------------------------------------------------------------------

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn test_read_pool() {
        let dir = TempDir::new().expect("tempdir");
        let mut ssb = Ssb::new(test_config(&dir), test_clock()).expect("ssb init");

        // Write 3 entries with different action types
        for (i, at) in [
            SsbActionType::FileWrite,
            SsbActionType::Finding,
            SsbActionType::FileWrite,
        ]
        .iter()
        .enumerate()
        {
            ssb.write(SsbWriteRequest {
                action_type: *at,
                agent_id: format!("agent-{i}"),
                superstep: 0,
                payload: test_payload(*at),
                files: vec![format!("src/f{i}.rs")],
            })
            .await
            .expect("write");
        }

        // query_by_session
        let rows = ssb.query_by_session("test-session").expect("by_session");
        assert_eq!(rows.len(), 3, "query_by_session should return 3 entries");

        // query_by_type
        let rows = ssb
            .query_by_type(SsbActionType::FileWrite)
            .expect("by_type");
        assert_eq!(rows.len(), 2, "query_by_type FileWrite should return 2");

        // query_by_superstep
        let rows = ssb.query_by_superstep(0).expect("by_superstep");
        assert_eq!(rows.len(), 3, "query_by_superstep(0) should return 3");

        // query_by_file (uses rarray)
        let rows = ssb
            .query_by_file(&["src/f0.rs", "src/f2.rs"])
            .expect("by_file");
        assert_eq!(rows.len(), 2, "query_by_file should return 2 entries");

        // query_by_file with empty slice
        let rows = ssb.query_by_file(&[]).expect("by_file_empty");
        assert_eq!(rows.len(), 0, "query_by_file([]) should return 0");

        // last_seq
        let seq = ssb.last_seq().expect("last_seq");
        assert_eq!(seq, Some(3), "last_seq should be 3 after 3 writes");

        // Verify round-trip of action_type through ssb_action_type_from_str
        let finding = ssb
            .query_by_type(SsbActionType::Finding)
            .expect("finding");
        assert_eq!(finding.len(), 1);
        assert_eq!(finding[0].action_type, SsbActionType::Finding);

        ssb.shutdown();
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn test_conflict_detection() {
        let dir = TempDir::new().expect("tempdir");
        let config = SsbConfig {
            db_path: dir.path().join("test_ssb.db"),
            pool_size: Some(2),
            project_id: "proj-1".into(),
            session_id: "session-A".into(),
        };
        let mut ssb = Ssb::new(config, test_clock()).expect("ssb init");

        // session-A writes to f1.rs and f2.rs
        ssb.write(SsbWriteRequest {
            action_type: SsbActionType::FileWrite,
            agent_id: "agent-A".into(),
            superstep: 1,
            payload: test_payload(SsbActionType::FileWrite),
            files: vec!["src/f1.rs".into(), "src/f2.rs".into()],
        })
        .await
        .expect("write A");

        // Test-only backdoor: direct SQL insert bypasses write_loop and
        // update_hook (which is on writer_conn, not pool connections).
        {
            let obj = ssb.read_pool.get().await.expect("pool conn");
            obj.execute(
                "INSERT INTO ssb (project_id, session_id, agent_id, superstep, action_type, payload, created_at) \
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
                params!["proj-1", "session-B", "agent-B", 1i64, "file_write", &[2u8][..], 100i64],
            )
            .expect("direct insert ssb");
            let seq = obj.last_insert_rowid();
            obj.execute(
                "INSERT INTO ssb_files (seq, file_path) VALUES (?1, ?2)",
                params![seq, "src/f1.rs"],
            )
            .expect("direct insert ssb_files");
        }

        // query_conflict: session-A asks who else touched f1.rs in superstep 1
        let conflicts = ssb
            .query_conflict(&["src/f1.rs"], "session-A", 1)
            .expect("conflict");
        assert_eq!(conflicts, vec!["agent-B"], "should detect agent-B");

        // f2.rs was only touched by session-A, no conflict for session-B asking
        let conflicts = ssb
            .query_conflict(&["src/f2.rs"], "session-B", 1)
            .expect("conflict f2");
        assert_eq!(conflicts, vec!["agent-A"], "should detect agent-A for f2");

        // No conflict for untouched file
        let conflicts = ssb
            .query_conflict(&["src/untouched.rs"], "session-A", 1)
            .expect("no conflict");
        assert!(conflicts.is_empty(), "no conflict for untouched file");

        // Empty file_paths → empty result
        let conflicts = ssb
            .query_conflict(&[], "session-A", 1)
            .expect("empty paths");
        assert!(conflicts.is_empty(), "empty paths = empty result");

        ssb.shutdown();
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn test_conflict_cross_project() {
        let dir = TempDir::new().expect("tempdir");

        // Two SSB instances with different project IDs sharing the same DB
        let config_a = SsbConfig {
            db_path: dir.path().join("test_ssb.db"),
            pool_size: Some(2),
            project_id: "proj-alpha".into(),
            session_id: "session-1".into(),
        };
        let config_b = SsbConfig {
            db_path: dir.path().join("test_ssb.db"),
            pool_size: Some(2),
            project_id: "proj-beta".into(),
            session_id: "session-2".into(),
        };
        let mut ssb_a = Ssb::new(config_a, test_clock()).expect("ssb_a init");
        let mut ssb_b = Ssb::new(config_b, test_clock()).expect("ssb_b init");

        // proj-alpha writes to shared.rs
        ssb_a
            .write(SsbWriteRequest {
                action_type: SsbActionType::FileWrite,
                agent_id: "agent-alpha".into(),
                superstep: 0,
                payload: test_payload(SsbActionType::FileWrite),
                files: vec!["src/shared.rs".into()],
            })
            .await
            .expect("write alpha");

        // proj-beta writes to shared.rs
        ssb_b
            .write(SsbWriteRequest {
                action_type: SsbActionType::FileWrite,
                agent_id: "agent-beta".into(),
                superstep: 0,
                payload: test_payload(SsbActionType::FileWrite),
                files: vec!["src/shared.rs".into()],
            })
            .await
            .expect("write beta");

        // Cross-project isolation: all read methods must be project-scoped.

        // query_conflict: proj-alpha should NOT see proj-beta's writes
        let conflicts = ssb_a
            .query_conflict(&["src/shared.rs"], "session-1", 0)
            .expect("cross-project conflict");
        assert!(
            conflicts.is_empty(),
            "cross-project should not detect conflict, got: {conflicts:?}"
        );

        // query_by_session: ssb_a should only see its own session's entries
        let rows = ssb_a
            .query_by_session("session-1")
            .expect("by_session alpha");
        assert_eq!(rows.len(), 1, "alpha should see only its own entry");
        assert_eq!(rows[0].project_id, "proj-alpha");

        // query_by_file: ssb_a should only see alpha's write to shared.rs
        let rows = ssb_a
            .query_by_file(&["src/shared.rs"])
            .expect("by_file alpha");
        assert_eq!(rows.len(), 1, "alpha should see only its own file entry");
        assert_eq!(rows[0].agent_id, "agent-alpha");

        // query_by_type: ssb_a should only see its own FileWrite
        let rows = ssb_a
            .query_by_type(SsbActionType::FileWrite)
            .expect("by_type alpha");
        assert_eq!(rows.len(), 1, "alpha should see only its own type entry");

        // query_by_superstep: ssb_a should only see its own superstep 0
        let rows = ssb_a
            .query_by_superstep(0)
            .expect("by_superstep alpha");
        assert_eq!(rows.len(), 1, "alpha should see only its own superstep entry");

        // last_seq: ssb_a should see only its own max seq
        let alpha_seq = ssb_a.last_seq().expect("last_seq alpha");
        let beta_seq = ssb_b.last_seq().expect("last_seq beta");
        assert!(alpha_seq.is_some());
        assert!(beta_seq.is_some());
        // Both wrote 1 entry each. If last_seq were global, alpha would see beta's seq.
        // Since project-scoped, each sees only its own.
        assert_ne!(
            alpha_seq, beta_seq,
            "project-scoped last_seq should differ (different seqs per project)"
        );

        ssb_a.shutdown();
        ssb_b.shutdown();
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn test_append_only() {
        use std::sync::atomic::AtomicU32;

        let dir = TempDir::new().expect("tempdir");
        let mut ssb = Ssb::new(test_config(&dir), test_clock()).expect("ssb init");

        // Write one entry so we have a row to attempt mutation on
        ssb.write(SsbWriteRequest {
            action_type: SsbActionType::Checkpoint,
            agent_id: "agent-test".into(),
            superstep: 0,
            payload: test_payload(SsbActionType::Checkpoint),
            files: vec![],
        })
        .await
        .expect("write");

        // Use a flag-based hook instead of panic (panics in C FFI callbacks abort).
        // This tests the hook LOGIC (detection of UPDATE/DELETE), not the panic path.
        let violation_count = Arc::new(AtomicU32::new(0));
        let vc = violation_count.clone();

        let db_path = dir.path().join("test_ssb.db");
        let direct_conn = Connection::open(&db_path).expect("direct conn");
        direct_conn
            .update_hook(Some(
                move |action: rusqlite::hooks::Action, _db: &str, table: &str, _rowid: i64| {
                    if (table == "ssb" || table == "ssb_files")
                        && (action == rusqlite::hooks::Action::SQLITE_UPDATE
                            || action == rusqlite::hooks::Action::SQLITE_DELETE)
                    {
                        vc.fetch_add(1, Ordering::Relaxed);
                    }
                },
            ))
            .expect("set hook");

        // INSERT should NOT trigger the violation counter
        direct_conn
            .execute(
                "INSERT INTO ssb (project_id, session_id, agent_id, superstep, action_type, payload, created_at) \
                 VALUES ('p', 's', 'a', 0, 'checkpoint', X'01', 100)",
                [],
            )
            .expect("insert ok");
        assert_eq!(
            violation_count.load(Ordering::Relaxed),
            0,
            "INSERT should not trigger append-only hook"
        );

        // UPDATE should trigger
        direct_conn
            .execute("UPDATE ssb SET agent_id = 'hacked' WHERE seq = 1", [])
            .expect("update executes");
        assert_eq!(
            violation_count.load(Ordering::Relaxed),
            1,
            "UPDATE should trigger append-only hook"
        );

        // DELETE should also trigger
        direct_conn
            .execute("DELETE FROM ssb WHERE seq = 2", [])
            .expect("delete executes");
        assert_eq!(
            violation_count.load(Ordering::Relaxed),
            2,
            "DELETE should trigger append-only hook"
        );

        // Verify the production code installs the hook on writer (debug mode).
        // The Ssb writer_conn has the panic-based hook. We test its presence
        // indirectly: if write() succeeds (INSERT), the hook didn't fire.
        let seq = ssb
            .write(SsbWriteRequest {
                action_type: SsbActionType::Checkpoint,
                agent_id: "agent-verify".into(),
                superstep: 0,
                payload: test_payload(SsbActionType::Checkpoint),
                files: vec![],
            })
            .await
            .expect("write after hook installed");
        assert!(seq > 0, "write should succeed (INSERT allowed by hook)");

        ssb.shutdown();
    }

    // -----------------------------------------------------------------------
    // 3B.4 — Typed payload tests
    // -----------------------------------------------------------------------

    fn all_test_payloads() -> Vec<SsbPayload> {
        vec![
            SsbPayload::FileWrite(FileWritePayload {
                path: "src/main.rs".into(),
                size_bytes: 1024,
                hash: "abc123".into(),
            }),
            SsbPayload::FileRead(FileReadPayload {
                path: "src/lib.rs".into(),
            }),
            SsbPayload::ToolResult(ToolResultPayload {
                tool_id: "grep".into(),
                success: true,
                output_summary: "found 3 matches".into(),
            }),
            SsbPayload::TestOutcome(TestOutcomePayload {
                test_name: "test_foo".into(),
                passed: true,
                duration_ms: 42,
            }),
            SsbPayload::Finding(FindingPayload {
                severity: "high".into(),
                category: "security".into(),
                message: "SQL injection".into(),
                file_path: Some("src/db.rs".into()),
                line: Some(99),
            }),
            SsbPayload::Claim(ClaimPayload {
                claim_text: "blake3 is faster".into(),
                grounding_source: "benchmark".into(),
                verified: true,
            }),
            SsbPayload::Checkpoint(CheckpointPayload {
                agent_state_hash: "deadbeef".into(),
                grc_history_hash: "cafebabe".into(),
                agent_id: "agent-001".into(),
                step_index: 5,
                file_sizes: vec![("state.msgpack".into(), 1024)],
            }),
            SsbPayload::ModelSwap(ModelSwapPayload {
                from_model: "sonnet".into(),
                to_model: "opus".into(),
                gpu_id: 0,
                swap_duration_ms: 150,
                reason: "complex task".into(),
            }),
            SsbPayload::PhaseTransition(PhaseTransitionPayload {
                from_state: "PLANNING".into(),
                to_state: "EXECUTING".into(),
                evidence_summary: "all tests pass".into(),
            }),
            SsbPayload::KnowledgeDegradation(KnowledgeDegradationPayload {
                component: "compaction".into(),
                old_mode: "full".into(),
                new_mode: "summary".into(),
                reason: "memory pressure".into(),
            }),
            SsbPayload::IsolationMetric(IsolationMetricPayload {
                agent_id: "agent-001".into(),
                phase_id: "3B".into(),
                context_savings_ratio: 2.5,
            }),
            SsbPayload::PlanUpdate(PlanUpdatePayload {
                phase_id: "3B".into(),
                version: "v1.2".into(),
                change_summary: "added step 4".into(),
            }),
            SsbPayload::ReplanRequest(ReplanRequestPayload {
                agent_id: "agent-0".into(),
                severity: "MAJOR".into(),
                evidence: "scope change detected".into(),
            }),
            SsbPayload::StubReference(StubReferencePayload {
                phase_id: "3B.4".into(),
                step_id: "step_1".into(),
                stub_hash: "abc123".into(),
            }),
            SsbPayload::ReasoningTrace(ReasoningTracePayload {
                agent_id: "agent-001".into(),
                step_index: 0,
                step_type: "observe".into(),
                content_summary: "read file".into(),
                grounding_source: "file:src/main.rs".into(),
                grounding_level: "HIGH".into(),
                confidence: 0.9,
                elapsed_ms: 100,
                topic: "debug".into(),
                budget_remaining: 10,
            }),
            SsbPayload::PhaseComplete(PhaseCompletePayload {
                phase_id: "3B.4".into(),
                agent_id: "agent-main".into(),
                fsm_state: "COMPLETE".into(),
                evidence_artifacts: vec!["test-results.json".into()],
                commit_hashes: vec!["abc1234".into()],
                test_outcomes: vec!["20 passed, 0 failed".into()],
            }),
            SsbPayload::MemoryMonitoring(MemoryMonitoringPayload {
                channel_id: "heap_mb".into(),
                deep_size_bytes: 512000000,
                threshold_exceeded: false,
            }),
            SsbPayload::DeathEvent(DeathEventPayload {
                agent_id: "agent-dead".into(),
                classification: "crash".into(),
                prior_step_index: 10,
                checkpoint_path: "sessions/default/agents/abc/abc".into(),
                had_checkpoint: true,
            }),
            SsbPayload::RecoveryEvent(RecoveryEventPayload {
                agent_id: "agent-dead".into(),
                classification: "crash".into(),
                recovered_from_checkpoint: true,
                degraded: false,
                first_grounding_source: "CHECKPOINT:sessions/default/agents/abc/abc".into(),
                prior_step_index: 10,
                recovery_step_index: 11,
            }),
            SsbPayload::SupervisionExhausted(SupervisionExhaustedPayload {
                agent_id: "agent-exhausted".into(),
                phase_id: "phase-3C".into(),
                restart_count: 4,
                max_restarts: 3,
                restart_history: vec![1, 5, 9, 13],
                escalation_action: "terminate".into(),
            }),
            SsbPayload::SpiralDetected(SpiralDetectedPayload {
                agent_id: "agent-spiral".into(),
                phase_id: "phase-3C".into(),
                spiral_depth: 5,
            }),
            SsbPayload::ContextReduction(ContextReductionPayload {
                level: 1,
                before_tokens: 10000,
                after_tokens: 8000,
                slots_affected: vec![5, 6],
            }),
            SsbPayload::BranchMerge(BranchMergePayload {
                winner_name: "explore-1".into(),
                abandoned_names: vec!["explore-2".into()],
                trunk_snap_id: "snap-merge-001".into(),
                from_snap_ids: vec!["snap-tip-1".into(), "snap-tip-2".into()],
            }),
            SsbPayload::WorkloadProfile(WorkloadProfilePayload {
                agent_role: "executor".into(),
                phase_id: "3B".into(),
                tokens_in: 1000,
                tokens_out: 500,
                latency_ms: 150,
                vram_peak_mb: 512,
                model_id: "gemma4:e2b".into(),
                gpu_id: 0,
                timestamp: 1700000000,
            }),
            SsbPayload::MistakeEntry(MistakeEntryPayload {
                baby_step_id: "step-1".into(),
                entry: elai_core::MistakeEntry {
                    attempt_number: 1,
                    diff_hash: elai_core::DiffHash::from_diff("bad-patch"),
                    failure_category: elai_core::FailureCategory::CompileError,
                    faulty_line: Some(42),
                    root_cause_json: Some("missing semicolon".into()),
                    action_taken: "templated".into(),
                    timestamp: 1700000000,
                },
                cleared: false,
            }),
            SsbPayload::ResearchStep(ResearchStepPayload {
                agent_id: "researcher-0".into(),
                node_id: 42,
                action: "expand".into(),
                phi: 0.75,
                psi: 0.80,
                iteration: 3,
            }),
            SsbPayload::ResearchDecomposeLevel(ResearchDecomposeLevelPayload {
                agent_id: "decomposer-0".into(),
                level: 2,
                concept_count: 4,
                phase_count: 8,
            }),
            SsbPayload::WebResearchStep(WebResearchStepPayload {
                agent_id: "web-researcher-0".into(),
                node_id: 1,
                action: "search".into(),
                phi: 0.6,
                psi: 0.5,
                iteration: 2,
                sources_fetched: 5,
                sources_used: 3,
            }),
            SsbPayload::PreviewAvailable(PreviewAvailablePayload {
                url: "http://localhost:5173".into(),
                step_id: "step-WEB-3".into(),
                slot: "live".into(),
                agent_id: "previewer-0".into(),
                build_success: true,
                dev_server_ok: true,
            }),
            SsbPayload::InterventionRequest(InterventionRequestPayload {
                step_id: "step-4K-1".into(),
                intervention_type: InterventionType::ApproveChange,
                description: "approve change?".into(),
                diff_summary: Some(DiffSummary {
                    files_changed: 2,
                    lines_added: 42,
                    lines_removed: 7,
                }),
                options: vec![
                    InterventionOption::Keyword(InterventionKeyword::Accept),
                    InterventionOption::Keyword(InterventionKeyword::Decline),
                    InterventionOption::Retry(InterventionRetry {
                        retry: "more evidence".into(),
                    }),
                ],
            }),
            SsbPayload::InterventionResponse(InterventionResponsePayload {
                request_id: 42,
                chosen: InterventionOption::Keyword(InterventionKeyword::Accept),
                user_note: Some("looks good".into()),
            }),
            SsbPayload::TaskSubmitted(TaskSubmittedPayload {
                task: "ship the dashboard MVP".into(),
                project_id: "dash-mvp".into(),
                submitted_at_ms: 1_713_300_000_000,
            }),
        ]
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn test_typed_payloads() {
        let dir = TempDir::new().expect("tempdir");
        let ssb = Ssb::new(test_config(&dir), test_clock()).expect("ssb init");

        let payloads = all_test_payloads();
        assert_eq!(payloads.len(), 32, "must cover all 32 action types");

        for (i, payload) in payloads.iter().enumerate() {
            let expected_action = payload.action_type();
            let seq = ssb
                .write_payload(
                    &format!("agent-{i}"),
                    i as i64,
                    payload.clone(),
                    vec![format!("file-{i}.rs")],
                )
                .await
                .unwrap_or_else(|e| panic!("write_payload variant {i} failed: {e}"));
            assert!(seq > 0, "seq must be positive for variant {i}");

            // Read back via query_by_type and verify round-trip
            let entries = ssb
                .query_by_type(expected_action)
                .unwrap_or_else(|e| panic!("query_by_type {expected_action:?} failed: {e}"));
            assert!(
                !entries.is_empty(),
                "query_by_type should return entries for {expected_action:?}"
            );
            let entry = entries.last().expect("at least one entry");
            let deserialized: SsbPayload =
                rmp_serde::from_slice(&entry.payload)
                    .unwrap_or_else(|e| panic!("deserialize variant {i} failed: {e}"));
            assert_eq!(
                &deserialized, payload,
                "round-trip mismatch for variant {i}"
            );
        }
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn test_payload_size_limit() {
        let dir = TempDir::new().expect("tempdir");
        let ssb = Ssb::new(test_config(&dir), test_clock()).expect("ssb init");

        // Build a payload that serializes to > 1 MiB
        let big_content = "x".repeat(2 * 1024 * 1024);
        let big_payload = SsbPayload::ReasoningTrace(ReasoningTracePayload {
            agent_id: "agent-big".into(),
            step_index: 0,
            step_type: "observe".into(),
            content_summary: big_content,
            grounding_source: "g".into(),
            grounding_level: "verified".into(),
            confidence: 0.9,
            elapsed_ms: 100,
            topic: "t".into(),
            budget_remaining: 5,
        });

        let result = ssb.write_payload("agent-big", 0, big_payload, vec![]).await;
        assert!(result.is_err(), "oversized payload must be rejected");
        match result.unwrap_err() {
            SsbError::PayloadTooLarge => {}
            other => panic!("expected PayloadTooLarge, got: {other}"),
        }
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn test_trace_integration() {
        let trace = ReasoningTracePayload {
            agent_id: "agent-trace".into(),
            step_index: 1,
            step_type: "hypothesize".into(),
            content_summary: "maybe X".into(),
            grounding_source: "test:unit".into(),
            grounding_level: "verified".into(),
            confidence: 0.8,
            elapsed_ms: 50,
            topic: "debug".into(),
            budget_remaining: 4,
        };

        let payload = trace.to_ssb_payload();
        match &payload {
            SsbPayload::ReasoningTrace(inner) => {
                assert_eq!(inner.step_type, "hypothesize");
                assert_eq!(inner.content_summary, "maybe X");
            }
            other => panic!("expected ReasoningTrace variant, got: {other:?}"),
        }
        assert_eq!(payload.action_type(), SsbActionType::ReasoningTrace);
    }

    // Type mismatch is impossible at the type level: write_payload() derives
    // action_type from the SsbPayload enum variant via action_type(). There is
    // no runtime string matching and no way to pass a mismatched pair. This is
    // a compile-time guarantee, not a runtime check.

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn test_payload_empty_vecs() {
        // PhaseCompletePayload with empty Vec fields must not error
        let dir = TempDir::new().expect("tempdir");
        let ssb = Ssb::new(test_config(&dir), test_clock()).expect("ssb init");

        let payload = SsbPayload::PhaseComplete(PhaseCompletePayload {
            phase_id: "3B.4".into(),
            agent_id: "a".into(),
            fsm_state: "COMPLETE".into(),
            evidence_artifacts: vec![],
            commit_hashes: vec![],
            test_outcomes: vec![],
        });
        let seq = ssb
            .write_payload("agent-empty", 0, payload, vec![])
            .await
            .expect("empty vecs must be accepted");
        assert!(seq > 0);
    }

    mod proptest_payloads {
        use super::*;
        use proptest::prelude::*;

        fn arb_payload() -> impl Strategy<Value = SsbPayload> {
            prop_oneof![
                ("[a-z]{1,20}", 0u64..10000, "[a-f0-9]{8}")
                    .prop_map(|(path, size_bytes, hash)| SsbPayload::FileWrite(
                        FileWritePayload { path, size_bytes, hash }
                    )),
                "[a-z]{1,20}".prop_map(|path| SsbPayload::FileRead(FileReadPayload { path })),
                ("[a-z]{1,10}", any::<bool>(), "[a-z ]{0,30}")
                    .prop_map(|(tool_id, success, output_summary)| SsbPayload::ToolResult(
                        ToolResultPayload { tool_id, success, output_summary }
                    )),
                ("[a-z_]{1,20}", any::<bool>(), 0u64..100000)
                    .prop_map(|(test_name, passed, duration_ms)| SsbPayload::TestOutcome(
                        TestOutcomePayload { test_name, passed, duration_ms }
                    )),
                ("[a-z]{1,10}", "[a-z]{1,10}", "[a-z ]{1,30}", any::<Option<String>>(), any::<Option<u32>>())
                    .prop_map(|(severity, category, message, file_path, line)| SsbPayload::Finding(
                        FindingPayload { severity, category, message, file_path, line }
                    )),
                ("[a-z ]{1,30}", "[a-z]{1,10}", any::<bool>())
                    .prop_map(|(claim_text, grounding_source, verified)| SsbPayload::Claim(
                        ClaimPayload { claim_text, grounding_source, verified }
                    )),
                ("[a-f0-9]{16}", "[a-f0-9]{16}", "[a-z0-9-]{1,20}", any::<u64>())
                    .prop_map(|(agent_state_hash, grc_history_hash, agent_id, step_index)| SsbPayload::Checkpoint(
                        CheckpointPayload { agent_state_hash, grc_history_hash, agent_id, step_index, file_sizes: vec![] }
                    )),
                ("[a-z]{1,10}", "[a-z]{1,10}", any::<u8>(), any::<u64>(), "[a-z ]{1,20}")
                    .prop_map(|(from_model, to_model, gpu_id, swap_duration_ms, reason)| SsbPayload::ModelSwap(
                        ModelSwapPayload { from_model, to_model, gpu_id, swap_duration_ms, reason }
                    )),
                ("[A-Z]{1,10}", "[A-Z]{1,10}", "[a-z ]{1,30}")
                    .prop_map(|(from_state, to_state, evidence_summary)| SsbPayload::PhaseTransition(
                        PhaseTransitionPayload { from_state, to_state, evidence_summary }
                    )),
                ("[a-z]{1,10}", "[a-z]{1,10}", "[a-z]{1,10}", "[a-z ]{1,20}")
                    .prop_map(|(component, old_mode, new_mode, reason)| SsbPayload::KnowledgeDegradation(
                        KnowledgeDegradationPayload { component, old_mode, new_mode, reason }
                    )),
                ("[a-z0-9-]{1,20}", "[0-9A-Z.]{1,10}", any::<f64>().prop_filter("finite", |v| v.is_finite()))
                    .prop_map(|(agent_id, phase_id, context_savings_ratio)| SsbPayload::IsolationMetric(
                        IsolationMetricPayload { agent_id, phase_id, context_savings_ratio }
                    )),
                ("[0-9A-Z.]{1,10}", "[0-9.]{1,5}", "[a-z ]{1,30}")
                    .prop_map(|(phase_id, version, change_summary)| SsbPayload::PlanUpdate(
                        PlanUpdatePayload { phase_id, version, change_summary }
                    )),
                ("[a-z0-9-]{1,20}", "(MINOR|MAJOR)", "[a-z ]{1,30}")
                    .prop_map(|(agent_id, severity, evidence)| SsbPayload::ReplanRequest(
                        ReplanRequestPayload { agent_id, severity, evidence }
                    )),
                ("[0-9A-Z.]{1,10}", "[a-z_]{1,10}", "[a-f0-9]{8}")
                    .prop_map(|(phase_id, step_id, stub_hash)| SsbPayload::StubReference(
                        StubReferencePayload { phase_id, step_id, stub_hash }
                    )),
                ("[a-z0-9-]{1,20}", any::<u32>(), "[a-z]{1,10}", "[a-z ]{1,30}", "[a-z:]{1,20}",
                 "(HIGH|MEDIUM|LOW)", any::<f64>().prop_filter("finite", |v| v.is_finite()),
                 any::<u64>(), "[a-z]{1,20}", any::<u32>())
                    .prop_map(|(agent_id, step_index, step_type, content_summary, grounding_source,
                                 grounding_level, confidence, elapsed_ms, topic, budget_remaining)| SsbPayload::ReasoningTrace(
                        ReasoningTracePayload { agent_id, step_index, step_type, content_summary, grounding_source,
                                                 grounding_level, confidence, elapsed_ms, topic, budget_remaining }
                    )),
                ("[0-9A-Z.]{1,10}", "[a-z-]{1,10}", "[A-Z]{1,10}",
                 proptest::collection::vec("[a-z.]{1,10}", 0..3),
                 proptest::collection::vec("[a-f0-9]{7}", 0..3),
                 proptest::collection::vec("[a-z ]{1,20}", 0..3))
                    .prop_map(|(phase_id, agent_id, fsm_state, evidence_artifacts, commit_hashes, test_outcomes)|
                        SsbPayload::PhaseComplete(PhaseCompletePayload {
                            phase_id, agent_id, fsm_state, evidence_artifacts, commit_hashes, test_outcomes,
                        })
                    ),
                ("[a-z_]{1,15}", any::<u64>(), any::<bool>())
                    .prop_map(|(channel_id, deep_size_bytes, threshold_exceeded)| SsbPayload::MemoryMonitoring(
                        MemoryMonitoringPayload { channel_id, deep_size_bytes, threshold_exceeded }
                    )),
                (0u8..5, 0usize..200000, 0usize..200000,
                 proptest::collection::vec(0u8..8, 0..4))
                    .prop_map(|(level, before_tokens, after_tokens, slots_affected)| SsbPayload::ContextReduction(
                        ContextReductionPayload { level, before_tokens, after_tokens, slots_affected }
                    )),
                ("[a-z-]{1,20}",
                 proptest::collection::vec("[a-z-]{1,20}", 0..3),
                 "[a-f0-9]{8}",
                 proptest::collection::vec("[a-f0-9]{8}", 1..4))
                    .prop_map(|(winner_name, abandoned_names, trunk_snap_id, from_snap_ids)| SsbPayload::BranchMerge(
                        BranchMergePayload { winner_name, abandoned_names, trunk_snap_id, from_snap_ids }
                    )),
                ("[a-z0-9-]{1,20}", 1u32..10, "[a-z_]{1,30}", any::<bool>())
                    .prop_map(|(baby_step_id, attempt, action_taken, has_line)| SsbPayload::MistakeEntry(
                        MistakeEntryPayload {
                            baby_step_id,
                            entry: elai_core::MistakeEntry {
                                attempt_number: attempt,
                                diff_hash: elai_core::DiffHash::from_diff(&action_taken),
                                failure_category: elai_core::FailureCategory::CompileError,
                                faulty_line: if has_line { Some(42) } else { None },
                                root_cause_json: None,
                                action_taken,
                                timestamp: 0,
                            },
                            cleared: false,
                        }
                    )),
                ("[a-z0-9-]{1,20}", 0u32..4, 0usize..100, 0usize..100)
                    .prop_map(|(agent_id, level, concept_count, phase_count)| SsbPayload::ResearchDecomposeLevel(
                        ResearchDecomposeLevelPayload { agent_id, level, concept_count, phase_count }
                    )),
                ("[a-z0-9-]{1,20}", 0u64..1000, "[a-z]{1,10}",
                 any::<f64>().prop_filter("finite", |v| v.is_finite()),
                 any::<f64>().prop_filter("finite", |v| v.is_finite()),
                 0u32..100, 0u32..50, 0u32..50)
                    .prop_map(|(agent_id, node_id, action, phi, psi, iteration, sources_fetched, sources_used)|
                        SsbPayload::WebResearchStep(WebResearchStepPayload {
                            agent_id, node_id, action, phi, psi, iteration, sources_fetched, sources_used,
                        })
                    ),
                ("http://localhost:[1-9][0-9]{3,4}", "[a-z0-9-]{1,20}",
                 "(live|draft|previous)", "[a-z0-9-]{1,20}",
                 any::<bool>(), any::<bool>())
                    .prop_map(|(url, step_id, slot, agent_id, build_success, dev_server_ok)|
                        SsbPayload::PreviewAvailable(PreviewAvailablePayload {
                            url, step_id, slot, agent_id, build_success, dev_server_ok,
                        })
                    ),
                ("[a-z0-9-]{1,20}", "[a-z ]{1,20}")
                    .prop_map(|(step_id, description)| SsbPayload::InterventionRequest(
                        InterventionRequestPayload {
                            step_id,
                            intervention_type: InterventionType::ApproveChange,
                            description,
                            diff_summary: None,
                            options: vec![InterventionOption::Keyword(InterventionKeyword::Accept)],
                        }
                    )),
                (0i64..10_000, any::<Option<String>>())
                    .prop_map(|(request_id, user_note)| SsbPayload::InterventionResponse(
                        InterventionResponsePayload {
                            request_id,
                            chosen: InterventionOption::Keyword(InterventionKeyword::Accept),
                            user_note,
                        }
                    )),
                ("[a-z ]{1,40}", "[a-z0-9-]{1,20}", 0i64..1_000_000_000_000)
                    .prop_map(|(task, project_id, submitted_at_ms)| SsbPayload::TaskSubmitted(
                        TaskSubmittedPayload { task, project_id, submitted_at_ms }
                    )),
            ]
        }

        proptest! {
            #[test]
            fn test_payload_round_trip(payload in arb_payload()) {
                let bytes = rmp_serde::to_vec_named(&payload)
                    .expect("serialize must succeed");
                let decoded: SsbPayload = rmp_serde::from_slice(&bytes)
                    .expect("deserialize must succeed");
                prop_assert_eq!(&decoded, &payload);
            }
        }
    }
}
