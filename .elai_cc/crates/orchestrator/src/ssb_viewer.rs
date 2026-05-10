//! Read-only view over the SSB for the dashboard (4K.1).
//!
//! [`SsbViewer`] opens the SSB SQLite database with `SQLITE_OPEN_READ_ONLY` —
//! the SQLite engine refuses any write from this handle, so the dashboard
//! cannot accidentally corrupt agent state even if a query fires an INSERT.
//!
//! The connection is wrapped in `std::sync::Mutex` so the type is `Send + Sync`
//! and can be registered as Tauri managed state. No query holds the mutex
//! across an `.await`, so `std::sync::Mutex` is correct here (using
//! `tokio::sync::Mutex` would add async overhead for no benefit).

use std::path::Path;
use std::sync::Mutex;

use elai_core::ClockProvider;
use rusqlite::{params, Connection, OpenFlags};
use serde::{Deserialize, Serialize};

use crate::ssb::{
    ssb_action_type_from_str_public, BubbleUpTypedPayload, ConceptDecompositionNodePayload,
    ContextBudgetComputedPayload, DagRepairRoundPayload, FeasibilityReportPayload,
    InterventionOption, InterventionRequestPayload, InterventionType, NovelStepDetectedPayload,
    SsbActionType, SsbPayload, StubRunPayload, TaskSubmittedPayload,
};

/// Maximum length of `phase_id` accepted by [`SsbViewer::step_progress`].
const PHASE_ID_MAX_LEN: usize = 64;

/// Tier 2 Track B: maximum length of `baby_step_id` accepted by
/// [`SsbViewer::baby_step_detail`]. Mirrors [`PHASE_ID_MAX_LEN`] for
/// consistency — both ids share the orchestrator's `<noun>_<n>` naming.
const BABY_STEP_ID_MAX_LEN: usize = 96;

/// Tier 2 Track B: hard cap on rows returned by
/// [`SsbViewer::recent_preview_available`] regardless of the caller's
/// `limit` argument. Matches the shape of the [`MAX_PAGE_SIZE`] cap on
/// `recent_actions` so the dashboard cannot accidentally pull a 50k-row
/// payload across the IPC boundary.
const PREVIEW_AVAILABLE_HARD_CAP: usize = 64;

/// Tier 2 audit WARN 2 fix: maximum number of `phase_complete` rows
/// scanned by [`SsbViewer::phase_detail`] when checking whether the
/// requested `phase_id` exists. Bounds full-table scans on large SSBs so
/// PlanWindow clicks never hold the connection mutex for hundreds of ms.
/// Sized for the realistic upper bound of completed phases per project
/// (1k phases ~= a year of dense execution); a phase older than the cap
/// is reported as NotFound.
const PHASE_DETAIL_SCAN_LIMIT: usize = 1000;

// ---------------------------------------------------------------------------
// Constants
// ---------------------------------------------------------------------------

/// Maximum bytes lifted from the raw msgpack payload for the `summary` field
/// in [`SsbActionSummary`]. We return printable chars only — binary payloads
/// produce a short stub rather than garbled output.
const SUMMARY_TRUNCATE_CHARS: usize = 100;

/// Lowest seq value that fits in a signed 64-bit integer when converted from u64.
// NARROWING-OK: i64::MAX (positive) always fits u64; const init — cannot use TryFrom in const context.
const MAX_SEQ_I64_AS_U64: u64 = i64::MAX as u64;

// ---------------------------------------------------------------------------
// Error type
// ---------------------------------------------------------------------------

/// Errors produced by [`SsbViewer`].
#[derive(Debug, thiserror::Error)]
pub enum SsbViewerError {
    #[error("sqlite error: {0}")]
    Sqlite(#[from] rusqlite::Error),

    #[error("lock poisoned")]
    LockPoisoned,

    #[error("payload decode error: {0}")]
    Decode(#[from] rmp_serde::decode::Error),

    #[error("unknown action type: {0}")]
    UnknownActionType(String),

    #[error("value out of range: {0}")]
    OutOfRange(String),

    #[error("validation: {0}")]
    Validation(String),

    /// Tier 2 Track B (dashboard-wiring §6:417..420): an entity referenced
    /// by id was not present in the SSB. Used by the four "lookup by id"
    /// commands (`get_phase_detail`, `get_baby_step_detail`,
    /// `get_explore_result`, `stop_dev_server`) so each one maps a single
    /// failure mode to a single variant. `kind` is a stable lower-cased
    /// noun (`"phase"`, `"baby_step"`, `"explore"`, `"preview"`) so the
    /// frontend can branch on the noun without parsing the message.
    #[error("not found: {kind} '{id}'")]
    NotFound { kind: String, id: String },
}

// ---------------------------------------------------------------------------
// DTO types returned to the dashboard
// ---------------------------------------------------------------------------

/// Compact per-action row for dashboard lists.
///
/// `summary` is the first [`SUMMARY_TRUNCATE_CHARS`] printable chars of the
/// raw msgpack bytes. We intentionally do NOT deserialize the full
/// [`SsbPayload`] per row — list views must stay cheap at 10k+ entries.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SsbActionSummary {
    pub seq: i64,
    pub timestamp: i64,
    pub action_type: SsbActionType,
    pub agent_id: String,
    pub summary: String,
}

/// Snapshot of FSM state derived from the latest `phase_transition` row.
///
/// Defined locally: the live [`crate::fsm::FsmState`] is tied to an in-process
/// state machine. The dashboard only reconstructs what the orchestrator told
/// the SSB, which is a string snapshot.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FsmStateSnapshot {
    pub state: String,
    pub phase: String,
}

/// Project/session the viewer is currently inspecting.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SessionInfo {
    pub project_id: String,
    pub session_id: String,
    pub last_seq: i64,
}

/// Progress of one phase, derived from `phase_complete` rows.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PhaseStatus {
    pub phase_id: String,
    pub status: String,
    pub step_count: u32,
    pub completed_count: u32,
}

/// Human-in-the-loop gate still waiting for a response.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct InterventionRequest {
    pub request_id: i64,
    pub timestamp: i64,
    pub step_id: String,
    pub intervention_type: InterventionType,
    pub description: String,
    pub options: Vec<InterventionOption>,
}

/// One file touched inside a session.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FileChange {
    pub path: String,
    pub action: String,
    pub summary: String,
}

/// One step in a reasoning chain (from `reasoning_trace` rows).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ThinkStepSummary {
    pub step_type: String,
    pub content: String,
    pub grounding: String,
    pub confidence: f64,
}

/// Per-step status row returned by [`SsbViewer::step_progress`] (4K.4).
///
/// Aggregates `phase_transition`, `phase_complete`, and `reasoning_trace`
/// entries by `step_id` (extracted from the typed payload via msgpack decode,
/// not JSON path).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StepStatus {
    pub step_id: String,
    pub status: String,
    pub started_at: Option<i64>,
    pub completed_at: Option<i64>,
    pub agent_id: Option<String>,
}

/// Full row detail returned by [`SsbViewer::get_action_detail`] (4K.5).
///
/// `payload_json` is the typed [`SsbPayload`] re-serialized as JSON for
/// frontend consumption (msgpack on the wire, JSON at the IPC boundary).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SsbActionDetail {
    pub seq: i64,
    pub ts: i64,
    pub action_type: String,
    pub agent_id: String,
    pub payload_json: serde_json::Value,
    pub files: Vec<String>,
}

/// Tier 1 CHAT window — one message or card row for the chat feed.
///
/// `kind` is `"message"` for user-submitted chat_message rows or
/// `"<card_kind>"` (`"status"` | `"error"` | `"disambiguation"` |
/// `"steering_ack"`) for chat_response rows. Callers render the two uniformly
/// in the same MessageList without needing to branch on SsbActionType.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ChatEntry {
    pub seq: i64,
    pub timestamp: i64,
    pub kind: String,
    pub session_id: String,
    pub text: String,
    pub intent: Option<String>,
    pub project_id: String,
}

/// Tier 1 CHAT window — compact row for recent `task_submitted` entries
/// rendered in the IDLE-state chat body.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TaskSummary {
    pub seq: i64,
    pub submitted_at_ms: i64,
    pub task: String,
    pub project_id: String,
}

// ---------------------------------------------------------------------------
// Tier 2 Track B DTOs (dashboard-wiring §6:417..480)
//
// PlanWindow tree: Concept > Phase > Step > BabyStep. The deepest node is
// `BabyStepRef` rather than the full `BabyStepSpec` (research_tree.rs:1284)
// to keep the tree payload compact for the 1500ms PlanTree poll. Full
// specs are fetched on click via [`SsbViewer::baby_step_detail`].
//
// Source data: at the time of writing the SSB does NOT persist a typed
// snapshot of the orchestrator's [`crate::research_tree::ConceptDecomposition`].
// `decomposition_tree` therefore returns an empty snapshot on a fresh DB
// and an empty snapshot on databases that pre-date the persistence work.
// Once a `concept_decomposition` action_type lands the implementation can
// scan the SSB without changing any of these DTOs.
// ---------------------------------------------------------------------------

/// PlanWindow tree root. `generated_at_ms` is wall-clock UTC ms so the
/// frontend can show a "fetched X ms ago" hint without re-querying.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DecompositionTreeSnapshot {
    pub concepts: Vec<ConceptNodeRef>,
    pub generated_at_ms: i64,
}

/// Compact projection of [`crate::research_tree::ConceptNode`] used by the
/// PlanWindow tree. Children are eagerly included so a single fetch produces
/// the full Concept->Phase->Step->BabyStep view.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConceptNodeRef {
    pub id: String,
    pub title: String,
    pub phases: Vec<PhaseNodeRef>,
}

/// Compact projection of [`crate::research_tree::PhaseNode`].
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PhaseNodeRef {
    pub id: String,
    pub title: String,
    pub steps: Vec<StepNodeRef>,
}

/// Compact projection of [`crate::research_tree::StepNode`].
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StepNodeRef {
    pub id: String,
    pub title: String,
    pub baby_steps: Vec<BabyStepRef>,
}

/// Tree-leaf reference to a BabyStep. The full spec is fetched lazily via
/// [`SsbViewer::baby_step_detail`] on user click.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BabyStepRef {
    pub id: String,
    pub title: String,
    pub status: String,
}

/// PhaseDetail returned by [`SsbViewer::phase_detail`]. All semantic fields
/// are `Option` because the source data ([`crate::research_tree::PipelineSpec`]
/// and PhaseNode) does NOT carry an `expected_outcome` or
/// `estimated_steps_remaining` field today — those values are derived if and
/// when the orchestrator persists them. Returning `None` is the documented
/// "unknown" signal; callers MUST NOT fabricate a placeholder.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PhaseDetail {
    pub phase_id: String,
    pub description: Option<String>,
    pub expected_outcome: Option<String>,
    pub dependencies: Vec<String>,
    pub estimated_steps_remaining: Option<u32>,
}

impl PhaseDetail {
    /// Construct an "unknown" PhaseDetail with all semantic fields cleared.
    /// Used by callers that have a `phase_id` but no persisted detail row.
    pub fn empty(phase_id: String) -> Self {
        Self {
            phase_id,
            description: None,
            expected_outcome: None,
            dependencies: Vec::new(),
            estimated_steps_remaining: None,
        }
    }
}

/// BabyStepDetail returned by [`SsbViewer::baby_step_detail`].
///
/// Mirrors the 6 fields of [`crate::research_tree::BabyStepSpec`] (the
/// briefing's "8 blocks" reference predates the source — actual struct has
/// 6 fields). `tdd` and `acceptance` are Optional because the source's
/// matching fields are also `Option`. `task`, `concept_name`, `phase_name`,
/// `step_name` are populated from the `BabyStepGrounding` substruct when
/// the source spec exists; otherwise empty.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BabyStepDetail {
    pub baby_step_id: String,
    pub task: String,
    pub estimated_tokens: u64,
    pub concept_name: String,
    pub phase_name: String,
    pub step_name: String,
    /// Serialised TddSpec when present; `None` when the source spec did not
    /// supply one. JSON for forward-compatibility with future spec fields.
    pub tdd: Option<serde_json::Value>,
    /// Serialised AcceptanceCriteria when present.
    pub acceptance: Option<serde_json::Value>,
}

impl BabyStepDetail {
    /// "Unknown" baby-step detail: every field zeroed/empty. Used by callers
    /// that have an id but no persisted spec row in the SSB.
    pub fn empty(baby_step_id: String) -> Self {
        Self {
            baby_step_id,
            task: String::new(),
            estimated_tokens: 0,
            concept_name: String::new(),
            phase_name: String::new(),
            step_name: String::new(),
            tdd: None,
            acceptance: None,
        }
    }
}

/// PreviewWindow row mirrored from a `preview_available` SSB action. The
/// runtime-join with `web_validator::DevServerStatus` happens at the
/// command layer (Tauri-managed registry) — this struct stays SSB-only.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PreviewAvailableRow {
    pub seq: i64,
    pub timestamp_ms: i64,
    pub url: String,
    pub step_id: String,
    pub slot: String,
    pub agent_id: String,
    pub build_success: bool,
    pub dev_server_ok: bool,
}

/// Batch 4B: compact Finding row used by the "Rule violations" strip at the
/// top of the dashboard ACTIVITY panel. Derived from Finding SSB rows whose
/// `audit_step` is `"post_execution_rules"`. The strip aggregates by
/// severity (BLOCK / WARN) so the frontend can show a one-line count with a
/// click-to-expand filtered list.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RuleViolationSummary {
    pub seq: i64,
    pub ts: i64,
    /// `"error"` (BLOCK), `"warning"` (WARN), `"escalate"` (Escalate).
    pub severity: String,
    pub category: String,
    pub message: String,
    /// Always [`POST_EXECUTION_RULES_AUDIT_STEP`] for rows returned by
    /// [`SsbViewer::rule_violations`]. The field is retained verbatim so
    /// future audit_step tags can reuse the same command.
    pub audit_step: String,
}

// ---------------------------------------------------------------------------
// Tier 3 partial — drawer DTOs (session s-84064).
//
// All structs are `Serialize` so they cross the Tauri IPC boundary unchanged.
// `Deserialize` is kept for test parity with the rest of the DTO set.
// ---------------------------------------------------------------------------

/// Tier 3 Reasoning drawer — GRC budget snapshot derived from
/// `ReasoningTracePayload` rows. One entry per (agent_id, topic) pair with
/// the most recent `budget_remaining` observed and the running call count.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GrcBudgetEntry {
    pub agent_id: String,
    pub topic: String,
    pub calls_used: u32,
    pub budget_remaining: u32,
    pub last_step_index: u32,
    pub last_ts_ms: i64,
}

/// Tier 3 Reasoning drawer — MistakeBook entry flattened from
/// `MistakeEntryPayload`. Cleared tombstones are filtered by the viewer
/// before the row is returned.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MistakeBookEntry {
    pub seq: i64,
    pub ts_ms: i64,
    pub baby_step_id: String,
    pub diff_hash: String,
    pub category: String,
    pub why: String,
}

/// Tier 3 Reasoning drawer — GG signal derived from `ProviderSignalsPayload`.
/// All four logprob/entropy fields are `Option` because the sidecar reports
/// them only when available.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GgSignalEntry {
    pub seq: i64,
    pub ts_ms: i64,
    pub step_id: String,
    pub gg_mean_logprob: Option<f64>,
    pub gg_min_logprob: Option<f64>,
    pub entropy_mean: Option<f64>,
    pub entropy_max: Option<f64>,
    pub novelty_cosine: Option<f64>,
    pub steering_vector_active: bool,
    pub etc_disabled: bool,
}

/// B-22 Phase 4G — auto-advance counters derived from the GateDecision SSB
/// rows. Returned by [`SsbViewer::auto_advance_stats`] and reshaped into
/// the dashboard `AutoAdvanceStats` JSON shape on the way out.
///
/// Field semantics:
/// * `total_steps` — every `GateDecision` row regardless of outcome.
/// * `auto_advanced` — rows with `decision == "auto_advance"`.
/// * `intervention_required` — rows with `decision == "request_human"`.
///   Other variants (`escalate` / `split_task` / `substitute_model`) are
///   counted toward `total_steps` but not toward either bucket above.
/// * `auto_advance_rate` — `auto_advanced / total_steps`, 0 when total = 0.
/// * `last_24h_rate` — same ratio, but restricted to rows whose `created_at`
///   lies within the trailing 24 hours of the supplied `now_ms`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AutoAdvanceStats {
    pub total_steps: u64,
    pub auto_advanced: u64,
    pub intervention_required: u64,
    pub auto_advance_rate: f64,
    pub last_24h_rate: f64,
}

/// Tier 3 Reasoning drawer — latest `FailureClassifiedPayload` snapshot.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FailureTaxonomyEntry {
    pub seq: i64,
    pub ts_ms: i64,
    pub category: String,
    /// Confidence expressed in the same basis-points units as the payload
    /// (0..=10000). The dashboard scales to a 0..1 float for display.
    pub confidence_bp: u16,
    pub step_id: String,
    pub source: String,
}

/// Tier 3 Reasoning drawer — most-recent DAIRA trace breadcrumb.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DairTraceEntry {
    pub seq: i64,
    pub ts_ms: i64,
    pub phase: String,
    pub tool: String,
    pub step_id: String,
    pub outcome: String,
    pub elapsed_us: u64,
}

/// Tier 3 Tools/Hooks drawer — hook verdict derived from `HookEventPayload`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct HookVerdictEntry {
    pub seq: i64,
    pub ts_ms: i64,
    pub tier: String,
    pub verdict: String,
    pub rule_id: Option<String>,
    pub hook_point: String,
    pub step_id: String,
    pub message: String,
}

/// Tier 3 Tools/Hooks drawer — ShortCoder reduction row derived from
/// `ShortCoderCompressionPayload`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ShortCoderEffectEntry {
    pub seq: i64,
    pub ts_ms: i64,
    pub step_id: String,
    pub tokens_in: u32,
    pub tokens_out: u32,
    pub reduction_pct: u32,
}

/// Tier 3 Tools/Hooks drawer — aggregated token reclamation totals across
/// all `ShortCoderCompression` + `ContextReduction` rows currently in SSB.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TokenReclamationStats {
    pub shortcoder_tokens_in_total: u64,
    pub shortcoder_tokens_out_total: u64,
    pub shortcoder_saved_total: u64,
    pub shortcoder_row_count: u32,
    pub context_reduction_before_total: u64,
    pub context_reduction_after_total: u64,
    pub context_reduction_saved_total: u64,
    pub context_reduction_row_count: u32,
}

/// Tier 3 Provider/VRAM drawer — latest VRAM tier detection row.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct VramTierSnapshot {
    pub tier: String,
    pub detected_via: String,
    pub free_mb: Option<u32>,
    pub ts_ms: i64,
    pub known: bool,
}

/// Tier 3 Knowledge drawer — overall readiness snapshot assembled from the
/// latest `DkgReady` + `IndexingProgress` rows.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct KnowledgeReadinessSnapshot {
    /// `"READY"` when both DKG and the latest indexing stage are complete;
    /// `"DEGRADED"` when indexing is running; `"UNAVAILABLE"` when no
    /// progress has been emitted yet.
    pub overall: String,
    pub dkg_ready: bool,
    pub indexing_stage: String,
    pub files_processed: u32,
    pub files_total: u32,
    pub last_project_id: String,
    pub last_ts_ms: i64,
}

/// Track A.6 — grounding cache aggregate stats, computed from SSB rows.
/// Returned by `SsbViewer::grounding_cache_stats()`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GroundingCacheStats {
    /// Approximate number of distinct cache keys observed (lower bound).
    pub total_keys: u32,
    /// Total cache hits in the scanned window.
    pub hits: u32,
    /// Total cache misses in the scanned window.
    pub misses: u32,
    /// elapsed_us of the most recent hit (0 if none).
    pub last_hit_us: u64,
    /// elapsed_us of the most recent miss (0 if none).
    pub last_miss_us: u64,
}

/// Tier 3 Knowledge drawer — DKG size snapshot from the latest
/// `DkgReadyPayload` row.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DkgStatsSnapshot {
    pub project_id: String,
    pub node_count: u32,
    pub edge_count: u32,
    pub languages: Vec<String>,
    pub ts_ms: i64,
    pub known: bool,
}

/// Tier 3 Knowledge drawer — BM25 index size snapshot. Derived from the
/// latest `IndexingProgressPayload` row; file counts mirror what the
/// indexing service reports.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Bm25StatsSnapshot {
    pub project_id: String,
    pub files_indexed: u32,
    pub files_total: u32,
    pub last_stage: String,
    pub ts_ms: i64,
    pub known: bool,
}

/// Tier 3 Knowledge drawer — vector-store snapshot. Minimal surface until a
/// dedicated SSB row type lands; mirrors the BM25 size fields as a best
/// effort until the writer side exposes codebert/granite document counts.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct VectorStatsSnapshot {
    pub project_id: String,
    pub docs_estimated: u32,
    pub ts_ms: i64,
    pub known: bool,
}

/// Tier 3 Knowledge drawer — embedding/reranker circuit-breaker snapshot.
/// B-4 SCHEMA: populated from `CircuitBreakerStateChangePayload` rows.
/// One entry per model; latest row per model name wins.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EmbreakerSnapshot {
    /// Model identifier matching dashboard EmbeddingBreaker.model field.
    pub model: String,
    /// "CLOSED" | "OPEN" | "HALF_OPEN"
    pub state: String,
    /// Consecutive failure count at last state change.
    pub failures: u32,
    /// Rolling success rate [0.0, 1.0] at last state change.
    pub success_rate: f64,
    /// Unix epoch ms of last failure; None when no failure recorded.
    pub last_failure_ts: Option<i64>,
}

/// Tier 3 Knowledge drawer — PageRank top-N entry. No SSB row type emits
/// per-symbol PageRank scores yet, so the viewer returns an empty list and
/// the dashboard renders a "not yet measured" line.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PageRankEntry {
    pub symbol: String,
    pub path: String,
    pub score: f64,
}

/// Tier 3 Knowledge drawer — query-tier routing histogram. Stub until a
/// dedicated SSB row type records per-query tier decisions.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct QueryTierStats {
    pub fast_pct: f32,
    pub standard_pct: f32,
    pub deep_pct: f32,
    pub avg_latency_ms_fast: f32,
    pub avg_latency_ms_standard: f32,
    pub avg_latency_ms_deep: f32,
    pub sample_size: u32,
}

/// Tier 3 Knowledge drawer — RAG freshness state. Stub until the freshness
/// meta DB exposes a read endpoint; only the decay configuration is known
/// at this point.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FreshnessState {
    pub decay_rate: f32,
    pub access_boost: f32,
    pub last_recalc_ms: i64,
    pub known: bool,
}

/// Tier 3 Orchestration drawer — 8-slot context budget snapshot derived from
/// the latest `ContextReductionPayload` row. Falls back to zeroed fields
/// when no reduction event has been recorded yet.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ContextBudgetSnapshot {
    pub level: u8,
    pub before_tokens: u64,
    pub after_tokens: u64,
    pub slots_affected: Vec<u8>,
    pub ts_ms: i64,
    pub known: bool,
}

/// Tier 3 Orchestration drawer — per-agent checkpoint row derived from
/// `CheckpointPayload`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AgentCheckpointEntry {
    pub seq: i64,
    pub ts_ms: i64,
    pub agent_id: String,
    pub step_index: u64,
    pub state_hash: String,
    pub file_sizes_total: u64,
}

/// Tier 3 Orchestration drawer — session handoff summary derived from
/// `BranchMergePayload`. Each branch-merge row boundary counts as a
/// single handoff (winner + abandoned names preserved verbatim).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SessionHandoffEntry {
    pub seq: i64,
    pub ts_ms: i64,
    pub winner: String,
    pub abandoned: Vec<String>,
    pub trunk_snap_id: String,
    pub from_snap_count: u32,
}

/// Tier 3 WebQuality drawer — per-tier validation verdict derived from
/// `ValidationTierPayload`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ValidationTierEntry {
    pub seq: i64,
    pub ts_ms: i64,
    pub tier: String,
    pub passed: bool,
    pub elapsed_ms: u32,
    pub rules_fired: u32,
    pub step_id: String,
}

/// Tier 3 complete Research drawer — one row in the research activity feed.
///
/// Derived from `WebResearchStepPayload` and `ResearchStepPayload` SSB
/// rows. The `kind` discriminator is `"web"` for web research and
/// `"local"` for local FlashResearch; all other fields mirror the payload
/// shape (field-by-field on WebResearchStep; zeroed sources_* for local).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ResearchStepEntry {
    pub seq: i64,
    pub ts_ms: i64,
    pub kind: String,
    pub agent_id: String,
    pub node_id: u64,
    pub action: String,
    pub phi: f64,
    pub psi: f64,
    pub iteration: u32,
    pub sources_fetched: u32,
    pub sources_used: u32,
    // B-7: fine-grained stage fields. Empty string / None for coarse rows.
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub stage: String,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub stage_phase: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub stage_payload: Option<serde_json::Value>,
}

/// Tier 3 complete Research drawer — research budget snapshot.
///
/// Graceful-empty stub (D3-3 precedent). The in-memory
/// `ResearchBudgetManager` does not persist to SSB, so this viewer method
/// returns zero values with `known: false`. Candidate writer-side row
/// type is documented at DEF-T3-RESEARCH-1 (tokens_used / tokens_max
/// progression per research run).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ResearchBudgetSnapshot {
    pub admitted: u32,
    pub pruned: u32,
    pub tokens_used: u64,
    pub tokens_max: u64,
    pub known: bool,
}

/// Tier 3 complete Research drawer — T1..T5 source-quality distribution.
///
/// Graceful-empty stub (D3-3 precedent). No SSB row type emits per-source
/// tier counts; the 5-tier domain lookup table is deterministic but not
/// persisted per research-id. Candidate writer-side row type is
/// documented at DEF-T3-RESEARCH-2.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SourceTierDistribution {
    pub t1: u32,
    pub t2: u32,
    pub t3: u32,
    pub t4: u32,
    pub t5: u32,
    pub known: bool,
}

/// Phase 4F (B-21) — flattened feasibility-probe view for the Reasoning drawer.
/// Combines the most recent `feasibility_report` summary with the per-step
/// `novel_step_detected` and `stub_run` rows that share the same probe run.
/// `latest_report` is `None` when the SSB has not yet seen a probe.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct FeasibilitySummary {
    pub latest_report: Option<FeasibilityReportRow>,
    pub novel_steps: Vec<NovelStepRow>,
    pub stub_runs: Vec<StubRunRow>,
}

/// One decoded `feasibility_report` SSB row.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FeasibilityReportRow {
    pub seq: i64,
    pub ts_ms: i64,
    /// `"feasible"` / `"marginal"` / `"infeasible"`.
    pub task_feasibility: String,
    /// `Some` when the probe was skipped — null on the wire (Option<String>
    /// serialises to JSON null, not undefined).
    pub skip_reason: Option<String>,
    pub novel_step_count: u32,
    pub escalations_count: u32,
    /// Margins are basis-points (10000 = 100.00%).
    pub vram_margin_bp: u16,
    pub ram_margin_bp: u16,
    pub disk_margin_bp: u16,
}

/// One decoded `novel_step_detected` SSB row.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NovelStepRow {
    pub seq: i64,
    pub ts_ms: i64,
    pub step_id: String,
    pub reason: String,
    pub alternative_count: u32,
}

/// One decoded `stub_run` SSB row.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StubRunRow {
    pub seq: i64,
    pub ts_ms: i64,
    pub step_id: String,
    pub passed: bool,
    pub elapsed_ms: u32,
}

// ---------------------------------------------------------------------------
// SsbViewer
// ---------------------------------------------------------------------------

/// Read-only SSB accessor used by the dashboard (4K.1).
pub struct SsbViewer {
    conn: Mutex<Connection>,
}

impl SsbViewer {
    /// Open the SSB database in read-only mode.
    ///
    /// Uses `SQLITE_OPEN_READ_ONLY` — any INSERT/UPDATE/DELETE executed on
    /// this connection is rejected by SQLite with `readonly database`.
    pub fn open(db_path: &Path) -> Result<Self, SsbViewerError> {
        let conn = Connection::open_with_flags(
            db_path,
            OpenFlags::SQLITE_OPEN_READ_ONLY | OpenFlags::SQLITE_OPEN_NO_MUTEX,
        )?;
        // `query_only` pragma is belt-and-braces on top of the open flag: it
        // makes the read-only intent explicit in the connection handle.
        conn.pragma_update(None, "query_only", "ON")?;
        Ok(Self {
            conn: Mutex::new(conn),
        })
    }

    fn lock(&self) -> Result<std::sync::MutexGuard<'_, Connection>, SsbViewerError> {
        self.conn.lock().map_err(|_| SsbViewerError::LockPoisoned)
    }

    /// Latest FSM-state entry converted to a [`FsmStateSnapshot`].
    ///
    /// Queries both `phase_transition` (emitted by Fsm::transition_with_metrics)
    /// and `fsm_transition` (emitted by task_runner for the RESEARCHING sub-state
    /// path) using the same IN-clause pattern as `recent_phase_transitions()`.
    /// This ensures the StatusBar badge reflects RESEARCHING even when no
    /// `phase_transition` row has been written yet for the current task.
    ///
    /// Returns `FsmStateSnapshot { state: "IDLE", phase: "" }` when the SSB
    /// contains no transition rows.
    pub fn latest_fsm_state(&self) -> Result<FsmStateSnapshot, SsbViewerError> {
        let conn = self.lock()?;
        let mut stmt = conn.prepare(
            "SELECT payload, action_type FROM ssb \
             WHERE action_type IN (?1, ?2) ORDER BY seq DESC LIMIT 1",
        )?;
        let row: Option<(Vec<u8>, String)> = stmt
            .query_row(
                params![
                    SsbActionType::PhaseTransition.as_sql_str(),
                    SsbActionType::FsmTransition.as_sql_str(),
                ],
                |row| Ok((row.get::<_, Vec<u8>>(0)?, row.get::<_, String>(1)?)),
            )
            .map(Some)
            .or_else(|e| match e {
                rusqlite::Error::QueryReturnedNoRows => Ok(None),
                other => Err(other),
            })?;
        let Some((bytes, _action_type)) = row else {
            return Ok(FsmStateSnapshot {
                state: "IDLE".to_owned(),
                phase: String::new(),
            });
        };
        let payload: SsbPayload = rmp_serde::from_slice(&bytes)?;
        match payload {
            SsbPayload::PhaseTransition(inner) => Ok(FsmStateSnapshot {
                state: inner.to_state,
                phase: inner.evidence_summary,
            }),
            SsbPayload::FsmTransition(inner) => Ok(FsmStateSnapshot {
                state: inner.to,
                phase: String::new(),
            }),
            _ => Ok(FsmStateSnapshot {
                state: "IDLE".to_owned(),
                phase: String::new(),
            }),
        }
    }

    /// B-2: Return recent FSM-transition rows with decoded from/to state fields.
    ///
    /// Queries both `phase_transition` (emitted by the Fsm engine) and
    /// `fsm_transition` (emitted by task_runner::emit_fsm_transition for the
    /// RESEARCHING sub-state path) so the dashboard history covers both producers
    /// without requiring a producer-side migration (Option B per B-2 context).
    ///
    /// Returns `Vec<(seq, from_state, to_state, created_at_ms)>` ordered newest-first.
    /// Rows whose msgpack payload cannot be decoded are silently skipped so a
    /// corrupt entry does not block the entire history view.
    pub fn recent_phase_transitions(
        &self,
        limit: usize,
    ) -> Result<Vec<(i64, String, String, i64)>, SsbViewerError> {
        let limit_i64 = i64::try_from(limit)
            .map_err(|e| SsbViewerError::OutOfRange(format!("limit: {e}")))?;
        let conn = self.lock()?;
        let mut stmt = conn.prepare(
            "SELECT seq, created_at, payload, action_type FROM ssb \
             WHERE action_type IN (?1, ?2) ORDER BY seq DESC LIMIT ?3",
        )?;
        let rows = stmt.query_map(
            params![
                SsbActionType::PhaseTransition.as_sql_str(),
                SsbActionType::FsmTransition.as_sql_str(),
                limit_i64,
            ],
            |row| {
                Ok((
                    row.get::<_, i64>(0)?,
                    row.get::<_, i64>(1)?,
                    row.get::<_, Vec<u8>>(2)?,
                    row.get::<_, String>(3)?,
                ))
            },
        )?;
        let mut out = Vec::new();
        for row in rows {
            let (seq, created_at, bytes, _action_type) = row?;
            match rmp_serde::from_slice::<SsbPayload>(&bytes) {
                Ok(SsbPayload::PhaseTransition(p)) => {
                    out.push((seq, p.from_state, p.to_state, created_at));
                }
                Ok(SsbPayload::FsmTransition(p)) => {
                    out.push((seq, p.from, p.to, created_at));
                }
                _ => {} // skip unknown or non-transition payloads
            }
        }
        Ok(out)
    }

    /// Most recent session row's project/session identifiers plus last seq.
    pub fn active_session(&self) -> Result<Option<SessionInfo>, SsbViewerError> {
        let conn = self.lock()?;
        let mut stmt = conn.prepare(
            "SELECT project_id, session_id, seq FROM ssb ORDER BY seq DESC LIMIT 1",
        )?;
        let row = stmt
            .query_row([], |row| {
                Ok(SessionInfo {
                    project_id: row.get(0)?,
                    session_id: row.get(1)?,
                    last_seq: row.get(2)?,
                })
            })
            .map(Some)
            .or_else(|e| match e {
                rusqlite::Error::QueryReturnedNoRows => Ok(None),
                other => Err(other),
            })?;
        Ok(row)
    }

    /// Aggregate per-phase progress from `phase_complete` entries.
    pub fn phase_progress(&self) -> Result<Vec<PhaseStatus>, SsbViewerError> {
        let conn = self.lock()?;
        let mut stmt = conn.prepare(
            "SELECT payload FROM ssb WHERE action_type = ?1 ORDER BY seq ASC",
        )?;
        let rows = stmt.query_map(
            params![SsbActionType::PhaseComplete.as_sql_str()],
            |row| row.get::<_, Vec<u8>>(0),
        )?;
        let mut out: Vec<PhaseStatus> = Vec::new();
        for row in rows {
            let bytes = row?;
            let payload: SsbPayload = rmp_serde::from_slice(&bytes)?;
            if let SsbPayload::PhaseComplete(inner) = payload {
                let completed = u32::try_from(inner.test_outcomes.len())
                    .map_err(|e| SsbViewerError::OutOfRange(e.to_string()))?;
                out.push(PhaseStatus {
                    phase_id: inner.phase_id,
                    status: inner.fsm_state,
                    step_count: completed,
                    completed_count: completed,
                });
            }
        }
        Ok(out)
    }

    /// Most recent SSB rows as compact summaries. `summary` is a truncated
    /// slice of the raw msgpack bytes — no full payload deserialization.
    pub fn recent_actions(
        &self,
        limit: usize,
        offset: usize,
    ) -> Result<Vec<SsbActionSummary>, SsbViewerError> {
        let limit_i64 = i64::try_from(limit)
            .map_err(|e| SsbViewerError::OutOfRange(format!("limit: {e}")))?;
        let offset_i64 = i64::try_from(offset)
            .map_err(|e| SsbViewerError::OutOfRange(format!("offset: {e}")))?;
        let conn = self.lock()?;
        let mut stmt = conn.prepare(
            "SELECT seq, created_at, action_type, agent_id, payload \
             FROM ssb ORDER BY seq DESC LIMIT ?1 OFFSET ?2",
        )?;
        let rows = stmt.query_map(params![limit_i64, offset_i64], |row| {
            Ok((
                row.get::<_, i64>(0)?,
                row.get::<_, i64>(1)?,
                row.get::<_, String>(2)?,
                row.get::<_, String>(3)?,
                row.get::<_, Vec<u8>>(4)?,
            ))
        })?;
        let mut out = Vec::new();
        for row in rows {
            let (seq, created_at, at_str, agent_id, payload) = row?;
            let action_type = ssb_action_type_from_str_public(&at_str)
                .map_err(|e| SsbViewerError::UnknownActionType(e.to_string()))?;
            out.push(SsbActionSummary {
                seq,
                timestamp: created_at,
                action_type,
                agent_id,
                summary: summarize_payload(&payload),
            });
        }
        Ok(out)
    }

    /// Batch 4B: Finding rows whose `audit_step` is
    /// [`crate::POST_EXECUTION_RULES_AUDIT_STEP`], optionally narrowed to a
    /// single baby-step via the `step_id` prefix the task_runner writes
    /// into the Finding message (`baby_step={N} ...`). Results are ordered
    /// newest-first and hard-capped at 200 rows so a long-running session
    /// can't balloon the payload.
    pub fn rule_violations(
        &self,
        step_id: Option<&str>,
    ) -> Result<Vec<RuleViolationSummary>, SsbViewerError> {
        const MAX_ROWS: usize = 200;
        let conn = self.lock()?;
        let mut stmt = conn.prepare(
            "SELECT seq, created_at, payload FROM ssb \
             WHERE action_type = ?1 ORDER BY seq DESC LIMIT ?2",
        )?;
        let rows = stmt.query_map(
            params![
                SsbActionType::Finding.as_sql_str(),
                i64::try_from(MAX_ROWS.saturating_mul(4)).unwrap_or(i64::MAX),
            ],
            |row| {
                Ok((
                    row.get::<_, i64>(0)?,
                    row.get::<_, i64>(1)?,
                    row.get::<_, Vec<u8>>(2)?,
                ))
            },
        )?;
        let mut out: Vec<RuleViolationSummary> = Vec::new();
        for row in rows {
            let (seq, created_at, payload) = row?;
            // Decode as typed SsbPayload so we can read the audit_step tag —
            // the summary-truncation path used by `recent_actions` throws
            // away the fields we need.
            let typed: SsbPayload = match rmp_serde::from_slice(&payload) {
                Ok(t) => t,
                Err(_) => continue,
            };
            let SsbPayload::Finding(f) = typed else { continue };
            let audit_step = match f.audit_step.as_deref() {
                Some(s) => s,
                None => continue,
            };
            if audit_step != crate::POST_EXECUTION_RULES_AUDIT_STEP {
                continue;
            }
            if let Some(sid) = step_id {
                // The task_runner embeds `baby_step={N} ...` in the message.
                // `baby_step_7` from the dashboard maps to `baby_step=7`
                // in the payload — strip the prefix and match the number.
                let needle = sid
                    .strip_prefix("baby_step_")
                    .map(|n| format!("baby_step={n} "))
                    .unwrap_or_else(|| format!("{sid} "));
                if !f.message.contains(&needle) {
                    continue;
                }
            }
            out.push(RuleViolationSummary {
                seq,
                ts: created_at,
                severity: f.severity,
                category: f.category,
                message: f.message,
                audit_step: audit_step.to_owned(),
            });
            if out.len() >= MAX_ROWS {
                break;
            }
        }
        Ok(out)
    }

    /// All actions emitted by a specific agent (newest first).
    pub fn agent_actions(
        &self,
        agent_id: &str,
    ) -> Result<Vec<SsbActionSummary>, SsbViewerError> {
        let conn = self.lock()?;
        let mut stmt = conn.prepare(
            "SELECT seq, created_at, action_type, agent_id, payload \
             FROM ssb WHERE agent_id = ?1 ORDER BY seq DESC",
        )?;
        let rows = stmt.query_map(params![agent_id], |row| {
            Ok((
                row.get::<_, i64>(0)?,
                row.get::<_, i64>(1)?,
                row.get::<_, String>(2)?,
                row.get::<_, String>(3)?,
                row.get::<_, Vec<u8>>(4)?,
            ))
        })?;
        let mut out = Vec::new();
        for row in rows {
            let (seq, created_at, at_str, agent_id, payload) = row?;
            let action_type = ssb_action_type_from_str_public(&at_str)
                .map_err(|e| SsbViewerError::UnknownActionType(e.to_string()))?;
            out.push(SsbActionSummary {
                seq,
                timestamp: created_at,
                action_type,
                agent_id,
                summary: summarize_payload(&payload),
            });
        }
        Ok(out)
    }

    /// All `intervention_request` rows whose `seq` has no later matching
    /// `intervention_response`.
    pub fn pending_interventions(&self) -> Result<Vec<InterventionRequest>, SsbViewerError> {
        let conn = self.lock()?;
        let mut answered: std::collections::HashSet<i64> = std::collections::HashSet::new();
        {
            let mut stmt = conn.prepare(
                "SELECT payload FROM ssb WHERE action_type = ?1",
            )?;
            let rows = stmt.query_map(
                params![SsbActionType::InterventionResponse.as_sql_str()],
                |row| row.get::<_, Vec<u8>>(0),
            )?;
            for row in rows {
                let bytes = row?;
                let payload: SsbPayload = rmp_serde::from_slice(&bytes)?;
                if let SsbPayload::InterventionResponse(resp) = payload {
                    answered.insert(resp.request_id);
                }
            }
        }

        let mut stmt = conn.prepare(
            "SELECT seq, created_at, payload FROM ssb WHERE action_type = ?1 ORDER BY seq ASC",
        )?;
        let rows = stmt.query_map(
            params![SsbActionType::InterventionRequest.as_sql_str()],
            |row| {
                Ok((
                    row.get::<_, i64>(0)?,
                    row.get::<_, i64>(1)?,
                    row.get::<_, Vec<u8>>(2)?,
                ))
            },
        )?;
        let mut out = Vec::new();
        for row in rows {
            let (seq, created_at, bytes) = row?;
            if answered.contains(&seq) {
                continue;
            }
            let payload: SsbPayload = rmp_serde::from_slice(&bytes)?;
            if let SsbPayload::InterventionRequest(InterventionRequestPayload {
                step_id,
                intervention_type,
                description,
                options,
                ..
            }) = payload
            {
                out.push(InterventionRequest {
                    request_id: seq,
                    timestamp: created_at,
                    step_id,
                    intervention_type,
                    description,
                    options,
                });
            }
        }
        Ok(out)
    }

    /// Phase 4F (B-21) — combined feasibility-probe view for the dashboard.
    /// Returns the most recent `feasibility_report` row plus all
    /// `novel_step_detected` / `stub_run` rows in the latest probe run
    /// (newest-first). Limit `n_steps` caps the per-collection size so the
    /// dashboard cannot pull thousands of rows on a noisy task.
    pub fn feasibility_summary(
        &self,
        n_steps: usize,
    ) -> Result<FeasibilitySummary, SsbViewerError> {
        let cap = i64::try_from(n_steps.min(500))
            .map_err(|e| SsbViewerError::OutOfRange(format!("n_steps: {e}")))?;
        let conn = self.lock()?;

        // 1. latest feasibility_report row.
        let latest_report = {
            let mut stmt = conn.prepare(
                "SELECT seq, created_at, payload FROM ssb \
                 WHERE action_type = ?1 ORDER BY seq DESC LIMIT 1",
            )?;
            let mut rows = stmt.query(params![
                SsbActionType::FeasibilityReport.as_sql_str()
            ])?;
            match rows.next()? {
                Some(row) => {
                    let seq: i64 = row.get(0)?;
                    let ts_ms: i64 = row.get(1)?;
                    let bytes: Vec<u8> = row.get(2)?;
                    match rmp_serde::from_slice::<SsbPayload>(&bytes) {
                        Ok(SsbPayload::FeasibilityReport(p)) => {
                            Some(decode_feasibility_report_row(seq, ts_ms, p))
                        }
                        Ok(_) => None,
                        Err(e) => {
                            tracing::warn!(
                                error = %e,
                                "ssb_viewer: feasibility_report decode failed; skipping"
                            );
                            None
                        }
                    }
                }
                None => None,
            }
        };

        // 2. recent novel_step_detected rows.
        let novel_steps = {
            let mut stmt = conn.prepare(
                "SELECT seq, created_at, payload FROM ssb \
                 WHERE action_type = ?1 ORDER BY seq DESC LIMIT ?2",
            )?;
            let rows = stmt.query_map(
                params![SsbActionType::NovelStepDetected.as_sql_str(), cap],
                |row| {
                    Ok((
                        row.get::<_, i64>(0)?,
                        row.get::<_, i64>(1)?,
                        row.get::<_, Vec<u8>>(2)?,
                    ))
                },
            )?;
            let mut out: Vec<NovelStepRow> = Vec::new();
            for row in rows {
                let (seq, ts_ms, bytes) = row?;
                match rmp_serde::from_slice::<SsbPayload>(&bytes) {
                    Ok(SsbPayload::NovelStepDetected(p)) => {
                        out.push(decode_novel_step_row(seq, ts_ms, p));
                    }
                    Ok(_) => {}
                    Err(e) => {
                        tracing::warn!(
                            error = %e,
                            "ssb_viewer: novel_step_detected decode failed; skipping"
                        );
                    }
                }
            }
            out
        };

        // 3. recent stub_run rows.
        let stub_runs = {
            let mut stmt = conn.prepare(
                "SELECT seq, created_at, payload FROM ssb \
                 WHERE action_type = ?1 ORDER BY seq DESC LIMIT ?2",
            )?;
            let rows = stmt.query_map(
                params![SsbActionType::StubRun.as_sql_str(), cap],
                |row| {
                    Ok((
                        row.get::<_, i64>(0)?,
                        row.get::<_, i64>(1)?,
                        row.get::<_, Vec<u8>>(2)?,
                    ))
                },
            )?;
            let mut out: Vec<StubRunRow> = Vec::new();
            for row in rows {
                let (seq, ts_ms, bytes) = row?;
                match rmp_serde::from_slice::<SsbPayload>(&bytes) {
                    Ok(SsbPayload::StubRun(p)) => {
                        out.push(decode_stub_run_row(seq, ts_ms, p));
                    }
                    Ok(_) => {}
                    Err(e) => {
                        tracing::warn!(
                            error = %e,
                            "ssb_viewer: stub_run decode failed; skipping"
                        );
                    }
                }
            }
            out
        };

        Ok(FeasibilitySummary {
            latest_report,
            novel_steps,
            stub_runs,
        })
    }

    /// Every file touched in a given `session_id` with a short action tag.
    pub fn file_changes(&self, session_id: &str) -> Result<Vec<FileChange>, SsbViewerError> {
        let conn = self.lock()?;
        let mut stmt = conn.prepare(
            "SELECT f.file_path, s.action_type, s.payload \
             FROM ssb_files f JOIN ssb s ON s.seq = f.seq \
             WHERE s.session_id = ?1 ORDER BY s.seq ASC",
        )?;
        let rows = stmt.query_map(params![session_id], |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, Vec<u8>>(2)?,
            ))
        })?;
        let mut out = Vec::new();
        for row in rows {
            let (path, action, payload) = row?;
            out.push(FileChange {
                path,
                action,
                summary: summarize_payload(&payload),
            });
        }
        Ok(out)
    }

    /// Per-step progress for a given `phase_id` prefix (4K.4).
    ///
    /// Scans `phase_complete`, `phase_transition`, and `reasoning_trace`
    /// rows; groups by the `step_id`/`phase_id` extracted from the typed
    /// payload (msgpack decode, NOT JSON path on the raw bytes); orders the
    /// result by `step_id` ascending. `status` priority is
    /// `complete > transition > trace`.
    pub fn step_progress(&self, phase_id: &str) -> Result<Vec<StepStatus>, SsbViewerError> {
        validate_phase_id(phase_id)?;

        use std::collections::BTreeMap;

        struct Acc {
            status: u8,
            status_str: String,
            started_at: Option<i64>,
            completed_at: Option<i64>,
            agent_id: Option<String>,
        }
        impl Acc {
            fn empty() -> Self {
                Self {
                    status: 0,
                    status_str: String::new(),
                    started_at: None,
                    completed_at: None,
                    agent_id: None,
                }
            }
            fn observe(
                &mut self,
                rank: u8,
                status_str: &str,
                ts: i64,
                agent_id: Option<&str>,
                completed: bool,
            ) {
                if rank > self.status {
                    self.status = rank;
                    self.status_str = status_str.to_owned();
                }
                self.started_at = Some(self.started_at.map_or(ts, |s| s.min(ts)));
                if completed {
                    self.completed_at = Some(self.completed_at.map_or(ts, |c| c.max(ts)));
                }
                if let Some(a) = agent_id {
                    if self.agent_id.as_deref().map(|x| x < a).unwrap_or(true) {
                        self.agent_id = Some(a.to_owned());
                    }
                }
            }
        }

        let conn = self.lock()?;
        let mut stmt = conn.prepare(
            "SELECT created_at, action_type, agent_id, payload \
             FROM ssb \
             WHERE action_type IN (?1, ?2, ?3) \
             ORDER BY seq ASC",
        )?;
        let rows = stmt.query_map(
            params![
                SsbActionType::PhaseComplete.as_sql_str(),
                SsbActionType::PhaseTransition.as_sql_str(),
                SsbActionType::ReasoningTrace.as_sql_str(),
            ],
            |row| {
                Ok((
                    row.get::<_, i64>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, String>(2)?,
                    row.get::<_, Vec<u8>>(3)?,
                ))
            },
        )?;

        let mut acc: BTreeMap<String, Acc> = BTreeMap::new();
        for row in rows {
            let (ts, at_str, agent_id, bytes) = row?;
            let payload: SsbPayload = match rmp_serde::from_slice(&bytes) {
                Ok(p) => p,
                Err(_) => continue,
            };
            match payload {
                SsbPayload::PhaseComplete(inner) => {
                    if !step_matches(&inner.phase_id, phase_id) {
                        continue;
                    }
                    let entry = acc
                        .entry(inner.phase_id.clone())
                        .or_insert_with(Acc::empty);
                    entry.observe(3, &inner.fsm_state, ts, Some(&inner.agent_id), true);
                }
                SsbPayload::PhaseTransition(inner) => {
                    if at_str.as_str() != SsbActionType::PhaseTransition.as_sql_str() {
                        continue;
                    }
                    let key = inner.evidence_summary.clone();
                    if !step_matches(&key, phase_id) {
                        continue;
                    }
                    let entry = acc.entry(key).or_insert_with(Acc::empty);
                    entry.observe(2, &inner.to_state, ts, Some(&agent_id), false);
                }
                SsbPayload::ReasoningTrace(inner) => {
                    let key = inner.topic.clone();
                    if !step_matches(&key, phase_id) {
                        continue;
                    }
                    let entry = acc.entry(key).or_insert_with(Acc::empty);
                    entry.observe(1, &inner.step_type, ts, Some(&inner.agent_id), false);
                }
                _ => {}
            }
        }

        let out: Vec<StepStatus> = acc
            .into_iter()
            .map(|(step_id, a)| StepStatus {
                step_id,
                status: a.status_str,
                started_at: a.started_at,
                completed_at: a.completed_at,
                agent_id: a.agent_id,
            })
            .collect();
        Ok(out)
    }

    /// Full row detail for one SSB seq (4K.5). Decodes the msgpack payload
    /// to typed [`SsbPayload`] then re-encodes as JSON for frontend
    /// consumption. Returns `OutOfRange` for negative seq or unknown row.
    pub fn get_action_detail(&self, seq: i64) -> Result<SsbActionDetail, SsbViewerError> {
        if seq < 0 {
            return Err(SsbViewerError::Validation(format!(
                "seq must be >= 0, got {seq}"
            )));
        }
        let conn = self.lock()?;
        let row = conn
            .query_row(
                "SELECT seq, created_at, action_type, agent_id, payload \
                 FROM ssb WHERE seq = ?1",
                params![seq],
                |row| {
                    Ok((
                        row.get::<_, i64>(0)?,
                        row.get::<_, i64>(1)?,
                        row.get::<_, String>(2)?,
                        row.get::<_, String>(3)?,
                        row.get::<_, Vec<u8>>(4)?,
                    ))
                },
            )
            .map(Some)
            .or_else(|e| match e {
                rusqlite::Error::QueryReturnedNoRows => Ok(None),
                other => Err(other),
            })?;

        let Some((seq, ts, action_type, agent_id, payload_bytes)) = row else {
            return Err(SsbViewerError::Validation(format!(
                "no SSB row for seq {seq}"
            )));
        };

        let typed: SsbPayload = rmp_serde::from_slice(&payload_bytes)?;
        let payload_json = serde_json::to_value(&typed)
            .map_err(|e| SsbViewerError::Validation(format!("payload json encode: {e}")))?;

        let mut files_stmt = conn.prepare(
            "SELECT file_path FROM ssb_files WHERE seq = ?1 ORDER BY file_path",
        )?;
        let file_rows = files_stmt.query_map(params![seq], |row| row.get::<_, String>(0))?;
        let mut files: Vec<String> = Vec::new();
        for f in file_rows {
            files.push(f?);
        }

        Ok(SsbActionDetail {
            seq,
            ts,
            action_type,
            agent_id,
            payload_json,
            files,
        })
    }

    /// All `reasoning_trace` rows that reference a given `step_id`, ordered
    /// by `seq` ascending. `step_id` matches `topic` in the stored payload
    /// (the orchestrator writes the plan step into `topic`).
    ///
    /// Uses `idx_ssb_action_type` to restrict the scan to reasoning_trace
    /// rows only. A per-topic secondary index lands in 4K.3 to keep this
    /// sub-10ms for sessions with >10k reasoning steps.
    pub fn reasoning_chain(
        &self,
        step_id: &str,
    ) -> Result<Vec<ThinkStepSummary>, SsbViewerError> {
        let conn = self.lock()?;
        let mut stmt = conn.prepare(
            "SELECT payload FROM ssb WHERE action_type = ?1 ORDER BY seq ASC",
        )?;
        let rows = stmt.query_map(
            params![SsbActionType::ReasoningTrace.as_sql_str()],
            |row| row.get::<_, Vec<u8>>(0),
        )?;
        let mut out = Vec::new();
        for row in rows {
            let bytes = row?;
            let payload: SsbPayload = rmp_serde::from_slice(&bytes)?;
            if let SsbPayload::ReasoningTrace(inner) = payload {
                if inner.topic == step_id {
                    out.push(ThinkStepSummary {
                        step_type: inner.step_type,
                        content: inner.content_summary,
                        grounding: inner.grounding_source,
                        confidence: inner.confidence,
                    });
                }
            }
        }
        Ok(out)
    }

    /// Tier 1 CHAT window — the most recent `limit` chat_message and
    /// chat_response rows whose payload `project_id` matches the argument,
    /// returned in seq-ascending order (oldest first, newest last).
    ///
    /// Scans both `chat_message` and `chat_response` action types and
    /// decodes the payload to read the embedded `project_id`. The SSB row
    /// column `project_id` is not authoritative here (see Tier 1 decisions
    /// doc: dashboard writer is bound to one fixed project_id at startup,
    /// but payloads encode the user-selected project).
    pub fn chat_history(
        &self,
        project_id: &str,
        limit: u32,
    ) -> Result<Vec<ChatEntry>, SsbViewerError> {
        if project_id.is_empty() {
            return Err(SsbViewerError::Validation(
                "project_id is empty".to_string(),
            ));
        }
        let conn = self.lock()?;
        let mut stmt = conn.prepare(
            "SELECT seq, created_at, action_type, payload \
             FROM ssb \
             WHERE action_type IN (?1, ?2) \
             ORDER BY seq DESC \
             LIMIT ?3",
        )?;
        let rows = stmt.query_map(
            params![
                SsbActionType::ChatMessage.as_sql_str(),
                SsbActionType::ChatResponse.as_sql_str(),
                limit as i64, // NARROWING-OK: u32->i64 is widening; SQLite LIMIT parameter requires i64.
            ],
            |row| {
                Ok((
                    row.get::<_, i64>(0)?,
                    row.get::<_, i64>(1)?,
                    row.get::<_, String>(2)?,
                    row.get::<_, Vec<u8>>(3)?,
                ))
            },
        )?;
        let mut out: Vec<ChatEntry> = Vec::new();
        for row in rows {
            let (seq, created_at, action_type_str, bytes) = row?;
            let action_type = ssb_action_type_from_str_public(&action_type_str)
                .map_err(|e| SsbViewerError::UnknownActionType(e.to_string()))?;
            let payload: SsbPayload = rmp_serde::from_slice(&bytes)?;
            match (action_type, payload) {
                (SsbActionType::ChatMessage, SsbPayload::ChatMessage(msg)) => {
                    if msg.project_id != project_id {
                        continue;
                    }
                    out.push(ChatEntry {
                        seq,
                        timestamp: created_at,
                        kind: "message".to_string(),
                        session_id: msg.session_id,
                        text: msg.text,
                        intent: Some(msg.intent),
                        project_id: msg.project_id,
                    });
                }
                (SsbActionType::ChatResponse, SsbPayload::ChatResponse(resp)) => {
                    if resp.project_id != project_id {
                        continue;
                    }
                    out.push(ChatEntry {
                        seq,
                        timestamp: created_at,
                        kind: resp.card_kind,
                        session_id: resp.session_id,
                        text: resp.body,
                        intent: None,
                        project_id: resp.project_id,
                    });
                }
                _ => {
                    // action_type / payload drift — skip silently; the
                    // write-side validator rejects mismatched rows, so
                    // anything reaching here predates that validator.
                    continue;
                }
            }
        }
        out.reverse();
        Ok(out)
    }

    /// Tier 2 Track B — Concept>Phase>Step>BabyStep snapshot for the
    /// PlanWindow (dashboard-wiring §6:417..420).
    ///
    /// Returns an empty snapshot today. The orchestrator does NOT yet
    /// persist a `concept_decomposition` SSB row carrying the full
    /// [`crate::research_tree::ConceptDecomposition`] tree; until that work
    /// lands the dashboard renders a "no decomposition yet" placeholder.
    /// `task_id` is reserved for when the writer side starts emitting the
    /// row (it will scope the snapshot to one task). Returning `Ok(empty)`
    /// instead of an error preserves the documented "empty list = empty UI"
    /// contract followed by the rest of the SsbViewer methods.
    pub fn decomposition_tree(
        &self,
        _task_id: Option<&str>,
    ) -> Result<DecompositionTreeSnapshot, SsbViewerError> {
        // B-7b: read `concept_decomposition_node` rows emitted by SsbDecompositionEmitter
        // and reconstruct the Concept > Phase > Step > BabyStep hierarchy.
        // Rows are ordered seq-asc so parents always appear before children.
        // Cap at 2000 rows — a task with >2000 decomposition nodes is implausible.
        const DECOMP_SCAN_CAP: i64 = 2000;
        let conn = self.lock()?;
        let mut stmt = conn.prepare(
            "SELECT payload, created_at FROM ssb \
             WHERE action_type = ?1 ORDER BY seq ASC LIMIT ?2",
        )?;
        let rows = stmt.query_map(
            params![SsbActionType::ConceptDecompositionNode.as_sql_str(), DECOMP_SCAN_CAP],
            |row| Ok((row.get::<_, Vec<u8>>(0)?, row.get::<_, i64>(1)?)),
        )?;

        // Collect decoded payloads (silently skip corrupt rows).
        let mut payloads: Vec<ConceptDecompositionNodePayload> = Vec::new();
        for row in rows {
            let (bytes, _created_at) = row?;
            if let Ok(SsbPayload::ConceptDecompositionNode(p)) = rmp_serde::from_slice(&bytes) {
                payloads.push(p);
            }
        }

        if payloads.is_empty() {
            // No rows yet — return empty snapshot; dashboard renders placeholder.
            return Ok(DecompositionTreeSnapshot::default());
        }

        // Two-pass tree reconstruction: collect all concepts first, then
        // attach phases -> steps -> baby_steps bottom-up.
        use rustc_hash::FxHashMap;
        let mut concepts: Vec<ConceptNodeRef> = Vec::new();
        // Map node_id -> index in `concepts`.
        let mut concept_idx: FxHashMap<u64, usize> = FxHashMap::default();
        // Map node_id -> (concept_idx, phase_idx).
        let mut phase_idx: FxHashMap<u64, (usize, usize)> = FxHashMap::default();
        // Map node_id -> (concept_idx, phase_idx, step_idx).
        let mut step_idx: FxHashMap<u64, (usize, usize, usize)> = FxHashMap::default();

        for p in &payloads {
            match p.node_kind.as_str() {
                "concept" => {
                    let idx = concepts.len();
                    concept_idx.insert(p.node_id, idx);
                    concepts.push(ConceptNodeRef {
                        id: p.node_id.to_string(),
                        title: p.name.clone(),
                        phases: Vec::new(),
                    });
                }
                "phase" => {
                    if let Some(&ci) = concept_idx.get(&p.parent_id) {
                        let pi = concepts[ci].phases.len();
                        phase_idx.insert(p.node_id, (ci, pi));
                        concepts[ci].phases.push(PhaseNodeRef {
                            id: p.node_id.to_string(),
                            title: p.name.clone(),
                            steps: Vec::new(),
                        });
                    }
                }
                "step" => {
                    if let Some(&(ci, pi)) = phase_idx.get(&p.parent_id) {
                        let si = concepts[ci].phases[pi].steps.len();
                        step_idx.insert(p.node_id, (ci, pi, si));
                        concepts[ci].phases[pi].steps.push(StepNodeRef {
                            id: p.node_id.to_string(),
                            title: p.name.clone(),
                            baby_steps: Vec::new(),
                        });
                    }
                }
                "baby_step" => {
                    if let Some(&(ci, pi, si)) = step_idx.get(&p.parent_id) {
                        concepts[ci].phases[pi].steps[si].baby_steps.push(BabyStepRef {
                            id: p.node_id.to_string(),
                            title: p.name.clone(),
                            status: "pending".to_owned(),
                        });
                    }
                }
                _ => {} // unknown kind — skip silently.
            }
        }

        let generated_at_ms = elai_core::clock::OsClock
            .system_now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| i64::try_from(d.as_millis()).unwrap_or(i64::MAX))
            .unwrap_or(0);

        Ok(DecompositionTreeSnapshot { concepts, generated_at_ms })
    }

    /// Tier 2 Track B — phase metadata fetched on PlanWindow node click.
    ///
    /// Today the SSB persists no row carrying description / expected_outcome
    /// / dependencies. The implementation confirms the phase is real by
    /// scanning the most recent [`PHASE_DETAIL_SCAN_LIMIT`] `phase_complete`
    /// rows for a matching `phase_id`, and then returns a
    /// [`PhaseDetail::empty`] shell. When no matching row exists the phase
    /// is treated as unknown and a [`SsbViewerError::NotFound`] is surfaced.
    ///
    /// Tier 2 audit BLOCK 2 fix: the previous implementation also matched
    /// against `PhaseTransitionPayload::evidence_summary`, but that field
    /// is free-form text and contains no structured `phase_id`, producing
    /// both false positives (text that accidentally matches) and false
    /// negatives (mid-execution phases that have only emitted transitions).
    /// The fix narrows the lookup to `PhaseComplete.phase_id` and adds a
    /// SQL LIMIT to bound full-table scans on large SSBs (audit WARN 2).
    /// The known limitation: a phase that has not yet emitted a
    /// `phase_complete` row will be reported as NotFound. This is acceptable
    /// because PlanWindow only renders detail for completed phases today.
    pub fn phase_detail(&self, phase_id: &str) -> Result<PhaseDetail, SsbViewerError> {
        validate_phase_id(phase_id)?;
        let conn = self.lock()?;
        let scan_limit_i64 = i64::try_from(PHASE_DETAIL_SCAN_LIMIT)
            .map_err(|e| SsbViewerError::OutOfRange(format!("scan_limit: {e}")))?;
        let mut stmt = conn.prepare(
            "SELECT payload FROM ssb \
             WHERE action_type = ?1 \
             ORDER BY seq DESC LIMIT ?2",
        )?;
        let mut rows = stmt.query(params![
            SsbActionType::PhaseComplete.as_sql_str(),
            scan_limit_i64,
        ])?;
        let mut found = false;
        while let Some(row) = rows.next()? {
            let bytes: Vec<u8> = row.get(0)?;
            let payload: SsbPayload = match rmp_serde::from_slice(&bytes) {
                Ok(p) => p,
                Err(_) => continue,
            };
            if let SsbPayload::PhaseComplete(inner) = &payload {
                if step_matches(&inner.phase_id, phase_id) {
                    found = true;
                    break;
                }
            }
        }
        if !found {
            return Err(SsbViewerError::NotFound {
                kind: "phase".to_owned(),
                id: phase_id.to_owned(),
            });
        }
        Ok(PhaseDetail::empty(phase_id.to_owned()))
    }

    /// Tier 2 Track B — full BabyStep view fetched on PlanWindow leaf click.
    ///
    /// Mirrors [`crate::research_tree::BabyStepSpec`]'s 6 fields. The
    /// orchestrator does not yet persist BabyStepSpec rows in the SSB, so
    /// this implementation returns [`SsbViewerError::NotFound`] for unknown
    /// ids. Once the writer side emits a `baby_step_spec` action the lookup
    /// scans the SSB by id; today it can only confirm the id syntactically.
    pub fn baby_step_detail(
        &self,
        baby_step_id: &str,
    ) -> Result<BabyStepDetail, SsbViewerError> {
        if baby_step_id.is_empty() {
            return Err(SsbViewerError::Validation(
                "baby_step_id empty".to_owned(),
            ));
        }
        if baby_step_id.len() > BABY_STEP_ID_MAX_LEN {
            return Err(SsbViewerError::Validation(format!(
                "baby_step_id length {} exceeds max {BABY_STEP_ID_MAX_LEN}",
                baby_step_id.len()
            )));
        }
        // Touch the lock so connection-poison errors surface here.
        let _conn = self.lock()?;
        // No BabyStepSpec persistence yet — every lookup is a NotFound.
        Err(SsbViewerError::NotFound {
            kind: "baby_step".to_owned(),
            id: baby_step_id.to_owned(),
        })
    }

    /// Tier 2 Track B — most recent `preview_available` rows scoped to one
    /// project, ordered newest-first, capped at `limit` (and at
    /// [`PREVIEW_AVAILABLE_HARD_CAP`] regardless of caller).
    ///
    /// The dashboard's get_preview_state command joins this list with the
    /// in-process DevServer registry so the frontend sees URL + port + pid +
    /// status without an SSB schema bump (per design decision GAP-4).
    pub fn recent_preview_available(
        &self,
        project_id: &str,
        limit: usize,
    ) -> Result<Vec<PreviewAvailableRow>, SsbViewerError> {
        if project_id.is_empty() {
            return Err(SsbViewerError::Validation("project_id empty".into()));
        }
        let bounded = limit.min(PREVIEW_AVAILABLE_HARD_CAP);
        let bounded_i64 = i64::try_from(bounded)
            .map_err(|e| SsbViewerError::OutOfRange(format!("limit: {e}")))?;
        let conn = self.lock()?;
        let mut stmt = conn.prepare(
            "SELECT seq, created_at, payload FROM ssb \
             WHERE action_type = ?1 AND project_id = ?2 \
             ORDER BY seq DESC LIMIT ?3",
        )?;
        let rows = stmt.query_map(
            params![
                SsbActionType::PreviewAvailable.as_sql_str(),
                project_id,
                bounded_i64,
            ],
            |row| {
                Ok((
                    row.get::<_, i64>(0)?,
                    row.get::<_, i64>(1)?,
                    row.get::<_, Vec<u8>>(2)?,
                ))
            },
        )?;
        let mut out: Vec<PreviewAvailableRow> = Vec::new();
        for row in rows {
            let (seq, created_at, bytes) = row?;
            let payload: SsbPayload = match rmp_serde::from_slice(&bytes) {
                Ok(p) => p,
                Err(_) => continue,
            };
            if let SsbPayload::PreviewAvailable(p) = payload {
                out.push(PreviewAvailableRow {
                    seq,
                    timestamp_ms: created_at,
                    url: p.url,
                    step_id: p.step_id,
                    slot: p.slot,
                    agent_id: p.agent_id,
                    build_success: p.build_success,
                    dev_server_ok: p.dev_server_ok,
                });
            }
        }
        Ok(out)
    }

    /// Tier 1 CHAT window — the most recent `limit` `task_submitted` rows
    /// whose payload `project_id` matches the argument, ordered seq-desc
    /// (newest first). Used for the IDLE-state "recent tasks" list in the
    /// chat body.
    pub fn recent_tasks(
        &self,
        project_id: &str,
        limit: u32,
    ) -> Result<Vec<TaskSummary>, SsbViewerError> {
        if project_id.is_empty() {
            return Err(SsbViewerError::Validation(
                "project_id is empty".to_string(),
            ));
        }
        let conn = self.lock()?;
        let mut stmt = conn.prepare(
            "SELECT seq, payload FROM ssb \
             WHERE action_type = ?1 \
             ORDER BY seq DESC \
             LIMIT ?2",
        )?;
        let rows = stmt.query_map(
            params![
                SsbActionType::TaskSubmitted.as_sql_str(),
                limit as i64, // NARROWING-OK: u32->i64 is widening; SQLite LIMIT parameter requires i64.
            ],
            |row| {
                Ok((
                    row.get::<_, i64>(0)?,
                    row.get::<_, Vec<u8>>(1)?,
                ))
            },
        )?;
        let mut out: Vec<TaskSummary> = Vec::new();
        for row in rows {
            let (seq, bytes) = row?;
            let payload: SsbPayload = rmp_serde::from_slice(&bytes)?;
            if let SsbPayload::TaskSubmitted(TaskSubmittedPayload {
                task,
                project_id: pid,
                submitted_at_ms,
            }) = payload
            {
                if pid != project_id {
                    continue;
                }
                out.push(TaskSummary {
                    seq,
                    submitted_at_ms,
                    task,
                    project_id: pid,
                });
            }
        }
        Ok(out)
    }

    // -----------------------------------------------------------------------
    // Tier 3 partial — drawer read methods (session s-84064).
    //
    // Every method opens zero network or sidecar connections; they are all
    // SQLite reads against the read-only SSB handle held by this viewer.
    // Methods returning `Option` use `None` for "no data yet" so the
    // dashboard drawer can render a graceful empty state without mapping
    // an error variant to an empty UI. Vec-returning methods cap at
    // `DRAWER_QUERY_HARD_CAP` regardless of the caller's `limit`.
    // -----------------------------------------------------------------------

    /// GRC budget aggregation from `ReasoningTrace` rows (Tier 3 Reasoning
    /// drawer). Groups by (agent_id, topic) and reports the most-recent
    /// `budget_remaining` plus a `calls_used` count derived from the row
    /// count per group. `limit` caps the number of groups returned; the
    /// underlying SQL scan is hard-capped at 4× `limit` rows.
    pub fn grc_budgets(
        &self,
        limit: usize,
    ) -> Result<Vec<GrcBudgetEntry>, SsbViewerError> {
        let effective = limit.min(DRAWER_QUERY_HARD_CAP);
        let scan_cap = i64::try_from(effective.saturating_mul(4))
            .unwrap_or(i64::MAX);
        let conn = self.lock()?;
        let mut stmt = conn.prepare(
            "SELECT seq, created_at, payload FROM ssb \
             WHERE action_type = ?1 ORDER BY seq DESC LIMIT ?2",
        )?;
        let rows = stmt.query_map(
            params![SsbActionType::ReasoningTrace.as_sql_str(), scan_cap],
            |row| {
                Ok((
                    row.get::<_, i64>(0)?,
                    row.get::<_, i64>(1)?,
                    row.get::<_, Vec<u8>>(2)?,
                ))
            },
        )?;
        let mut groups: BTreeMapStub<(String, String), GrcBudgetEntry> =
            BTreeMapStub::new();
        for row in rows {
            let (_seq, ts_ms, bytes) = row?;
            let payload: SsbPayload = match rmp_serde::from_slice(&bytes) {
                Ok(p) => p,
                Err(_) => continue,
            };
            let SsbPayload::ReasoningTrace(trace) = payload else {
                continue;
            };
            let key = (trace.agent_id.clone(), trace.topic.clone());
            match groups.get_mut(&key) {
                Some(existing) => {
                    existing.calls_used = existing.calls_used.saturating_add(1);
                }
                None => {
                    groups.insert(
                        key,
                        GrcBudgetEntry {
                            agent_id: trace.agent_id,
                            topic: trace.topic,
                            calls_used: 1,
                            budget_remaining: trace.budget_remaining,
                            last_step_index: trace.step_index,
                            last_ts_ms: ts_ms,
                        },
                    );
                }
            }
            if groups.len() >= effective {
                break;
            }
        }
        Ok(groups.into_values().take(effective).collect())
    }

    /// Mistake book entries (Tier 3 Reasoning drawer). Silently skips
    /// tombstone rows (`cleared = true`) and any row whose seq is older
    /// than the latest tombstone for the same `baby_step_id`.
    pub fn mistake_book(
        &self,
        limit: usize,
    ) -> Result<Vec<MistakeBookEntry>, SsbViewerError> {
        let effective = limit.min(DRAWER_QUERY_HARD_CAP);
        let scan_cap = i64::try_from(effective.saturating_mul(4))
            .unwrap_or(i64::MAX);
        let conn = self.lock()?;
        let mut stmt = conn.prepare(
            "SELECT seq, created_at, payload FROM ssb \
             WHERE action_type = ?1 ORDER BY seq DESC LIMIT ?2",
        )?;
        let rows = stmt.query_map(
            params![SsbActionType::MistakeEntry.as_sql_str(), scan_cap],
            |row| {
                Ok((
                    row.get::<_, i64>(0)?,
                    row.get::<_, i64>(1)?,
                    row.get::<_, Vec<u8>>(2)?,
                ))
            },
        )?;
        let mut out: Vec<MistakeBookEntry> = Vec::new();
        let mut tombstoned: std::collections::BTreeSet<String> =
            std::collections::BTreeSet::new();
        for row in rows {
            let (seq, ts_ms, bytes) = row?;
            let payload: SsbPayload = match rmp_serde::from_slice(&bytes) {
                Ok(p) => p,
                Err(_) => continue,
            };
            let SsbPayload::MistakeEntry(entry) = payload else {
                continue;
            };
            if entry.cleared {
                tombstoned.insert(entry.baby_step_id);
                continue;
            }
            if tombstoned.contains(&entry.baby_step_id) {
                continue;
            }
            out.push(MistakeBookEntry {
                seq,
                ts_ms,
                baby_step_id: entry.baby_step_id,
                diff_hash: entry.entry.diff_hash.to_short_hex(),
                category: format!("{:?}", entry.entry.failure_category),
                why: entry.entry.action_taken.clone(),
            });
            if out.len() >= effective {
                break;
            }
        }
        Ok(out)
    }

    /// GG signals (Tier 3 Reasoning drawer) from `ProviderSignals` rows.
    pub fn gg_signals(
        &self,
        limit: usize,
    ) -> Result<Vec<GgSignalEntry>, SsbViewerError> {
        let effective = limit.min(DRAWER_QUERY_HARD_CAP);
        let limit_i64 = i64::try_from(effective).unwrap_or(i64::MAX);
        let conn = self.lock()?;
        let mut stmt = conn.prepare(
            "SELECT seq, created_at, payload FROM ssb \
             WHERE action_type = ?1 ORDER BY seq DESC LIMIT ?2",
        )?;
        let rows = stmt.query_map(
            params![SsbActionType::ProviderSignals.as_sql_str(), limit_i64],
            |row| {
                Ok((
                    row.get::<_, i64>(0)?,
                    row.get::<_, i64>(1)?,
                    row.get::<_, Vec<u8>>(2)?,
                ))
            },
        )?;
        let mut out: Vec<GgSignalEntry> = Vec::new();
        for row in rows {
            let (seq, ts_ms, bytes) = row?;
            let payload: SsbPayload = match rmp_serde::from_slice(&bytes) {
                Ok(p) => p,
                Err(_) => continue,
            };
            if let SsbPayload::ProviderSignals(s) = payload {
                out.push(GgSignalEntry {
                    seq,
                    ts_ms,
                    step_id: s.step_id,
                    gg_mean_logprob: s.gg_mean_logprob,
                    gg_min_logprob: s.gg_min_logprob,
                    entropy_mean: s.entropy_mean,
                    entropy_max: s.entropy_max,
                    novelty_cosine: s.novelty_cosine,
                    steering_vector_active: s.steering_vector_active,
                    etc_disabled: s.etc_disabled,
                });
            }
        }
        Ok(out)
    }

    /// Latest failure-classified row (Tier 3 Reasoning drawer).
    pub fn failure_taxonomy_last(
        &self,
    ) -> Result<Option<FailureTaxonomyEntry>, SsbViewerError> {
        let conn = self.lock()?;
        let mut stmt = conn.prepare(
            "SELECT seq, created_at, payload FROM ssb \
             WHERE action_type = ?1 ORDER BY seq DESC LIMIT 1",
        )?;
        let first = stmt
            .query_row(
                params![SsbActionType::FailureClassified.as_sql_str()],
                |row| {
                    Ok((
                        row.get::<_, i64>(0)?,
                        row.get::<_, i64>(1)?,
                        row.get::<_, Vec<u8>>(2)?,
                    ))
                },
            )
            .map(Some)
            .or_else(|e| match e {
                rusqlite::Error::QueryReturnedNoRows => Ok(None),
                other => Err(other),
            })?;
        let Some((seq, ts_ms, bytes)) = first else {
            return Ok(None);
        };
        let payload: SsbPayload = rmp_serde::from_slice(&bytes)?;
        if let SsbPayload::FailureClassified(f) = payload {
            return Ok(Some(FailureTaxonomyEntry {
                seq,
                ts_ms,
                category: f.category,
                confidence_bp: f.confidence_bp,
                step_id: f.step_id,
                source: f.source,
            }));
        }
        Ok(None)
    }

    /// Recent DAIRA breadcrumbs (Tier 3 Reasoning drawer).
    pub fn dair_trace_last(
        &self,
        limit: usize,
    ) -> Result<Vec<DairTraceEntry>, SsbViewerError> {
        let effective = limit.min(DRAWER_QUERY_HARD_CAP);
        let limit_i64 = i64::try_from(effective).unwrap_or(i64::MAX);
        let conn = self.lock()?;
        let mut stmt = conn.prepare(
            "SELECT seq, created_at, payload FROM ssb \
             WHERE action_type = ?1 ORDER BY seq DESC LIMIT ?2",
        )?;
        let rows = stmt.query_map(
            params![SsbActionType::DairaTrace.as_sql_str(), limit_i64],
            |row| {
                Ok((
                    row.get::<_, i64>(0)?,
                    row.get::<_, i64>(1)?,
                    row.get::<_, Vec<u8>>(2)?,
                ))
            },
        )?;
        let mut out: Vec<DairTraceEntry> = Vec::new();
        for row in rows {
            let (seq, ts_ms, bytes) = row?;
            let payload: SsbPayload = match rmp_serde::from_slice(&bytes) {
                Ok(p) => p,
                Err(_) => continue,
            };
            if let SsbPayload::DairaTrace(d) = payload {
                out.push(DairTraceEntry {
                    seq,
                    ts_ms,
                    phase: d.phase,
                    tool: d.tool,
                    step_id: d.step_id,
                    outcome: d.outcome,
                    elapsed_us: d.elapsed_us,
                });
            }
        }
        Ok(out)
    }

    /// Hook verdict log (Tier 3 Tools/Hooks drawer).
    pub fn hook_verdict_log(
        &self,
        limit: usize,
    ) -> Result<Vec<HookVerdictEntry>, SsbViewerError> {
        let effective = limit.min(DRAWER_QUERY_HARD_CAP);
        let limit_i64 = i64::try_from(effective).unwrap_or(i64::MAX);
        let conn = self.lock()?;
        let mut stmt = conn.prepare(
            "SELECT seq, created_at, payload FROM ssb \
             WHERE action_type = ?1 ORDER BY seq DESC LIMIT ?2",
        )?;
        let rows = stmt.query_map(
            params![SsbActionType::HookEvent.as_sql_str(), limit_i64],
            |row| {
                Ok((
                    row.get::<_, i64>(0)?,
                    row.get::<_, i64>(1)?,
                    row.get::<_, Vec<u8>>(2)?,
                ))
            },
        )?;
        let mut out: Vec<HookVerdictEntry> = Vec::new();
        for row in rows {
            let (seq, ts_ms, bytes) = row?;
            let payload: SsbPayload = match rmp_serde::from_slice(&bytes) {
                Ok(p) => p,
                Err(_) => continue,
            };
            if let SsbPayload::HookEvent(h) = payload {
                out.push(HookVerdictEntry {
                    seq,
                    ts_ms,
                    tier: h.tier,
                    verdict: h.verdict,
                    rule_id: h.rule_id,
                    hook_point: h.hook_point,
                    step_id: h.step_id,
                    message: h.message,
                });
            }
        }
        Ok(out)
    }

    /// ShortCoder compression rows (Tier 3 Tools/Hooks drawer).
    pub fn shortcoder_effects(
        &self,
        limit: usize,
    ) -> Result<Vec<ShortCoderEffectEntry>, SsbViewerError> {
        let effective = limit.min(DRAWER_QUERY_HARD_CAP);
        let limit_i64 = i64::try_from(effective).unwrap_or(i64::MAX);
        let conn = self.lock()?;
        let mut stmt = conn.prepare(
            "SELECT seq, created_at, payload FROM ssb \
             WHERE action_type = ?1 ORDER BY seq DESC LIMIT ?2",
        )?;
        let rows = stmt.query_map(
            params![
                SsbActionType::ShortCoderCompression.as_sql_str(),
                limit_i64,
            ],
            |row| {
                Ok((
                    row.get::<_, i64>(0)?,
                    row.get::<_, i64>(1)?,
                    row.get::<_, Vec<u8>>(2)?,
                ))
            },
        )?;
        let mut out: Vec<ShortCoderEffectEntry> = Vec::new();
        for row in rows {
            let (seq, ts_ms, bytes) = row?;
            let payload: SsbPayload = match rmp_serde::from_slice(&bytes) {
                Ok(p) => p,
                Err(_) => continue,
            };
            if let SsbPayload::ShortCoderCompression(c) = payload {
                out.push(ShortCoderEffectEntry {
                    seq,
                    ts_ms,
                    step_id: c.step_id,
                    tokens_in: c.tokens_in,
                    tokens_out: c.tokens_out,
                    reduction_pct: c.reduction_pct,
                });
            }
        }
        Ok(out)
    }

    /// Aggregated token reclamation totals (Tier 3 Tools/Hooks drawer).
    /// Runs two bounded scans: `ShortCoderCompression` for per-response
    /// reduction and `ContextReduction` for budget-level adjustments.
    /// Hard-capped at `TOKEN_RECLAMATION_SCAN_CAP` rows per scan.
    pub fn token_reclamation_stats(
        &self,
    ) -> Result<TokenReclamationStats, SsbViewerError> {
        let scan_cap = i64::try_from(TOKEN_RECLAMATION_SCAN_CAP)
            .unwrap_or(i64::MAX);
        let conn = self.lock()?;

        let (mut sc_in, mut sc_out, mut sc_rows): (u64, u64, u32) = (0, 0, 0);
        {
            let mut stmt = conn.prepare(
                "SELECT payload FROM ssb WHERE action_type = ?1 \
                 ORDER BY seq DESC LIMIT ?2",
            )?;
            let rows = stmt.query_map(
                params![
                    SsbActionType::ShortCoderCompression.as_sql_str(),
                    scan_cap,
                ],
                |row| row.get::<_, Vec<u8>>(0),
            )?;
            for row in rows {
                let bytes = row?;
                let payload: SsbPayload = match rmp_serde::from_slice(&bytes) {
                    Ok(p) => p,
                    Err(_) => continue,
                };
                if let SsbPayload::ShortCoderCompression(c) = payload {
                    sc_in = sc_in.saturating_add(u64::from(c.tokens_in));
                    sc_out = sc_out.saturating_add(u64::from(c.tokens_out));
                    sc_rows = sc_rows.saturating_add(1);
                }
            }
        }

        let (mut cr_before, mut cr_after, mut cr_rows): (u64, u64, u32) =
            (0, 0, 0);
        {
            let mut stmt = conn.prepare(
                "SELECT payload FROM ssb WHERE action_type = ?1 \
                 ORDER BY seq DESC LIMIT ?2",
            )?;
            let rows = stmt.query_map(
                params![
                    SsbActionType::ContextReduction.as_sql_str(),
                    scan_cap,
                ],
                |row| row.get::<_, Vec<u8>>(0),
            )?;
            for row in rows {
                let bytes = row?;
                let payload: SsbPayload = match rmp_serde::from_slice(&bytes) {
                    Ok(p) => p,
                    Err(_) => continue,
                };
                if let SsbPayload::ContextReduction(cr) = payload {
                    cr_before =
                        cr_before.saturating_add(cr.before_tokens as u64);
                    cr_after =
                        cr_after.saturating_add(cr.after_tokens as u64);
                    cr_rows = cr_rows.saturating_add(1);
                }
            }
        }

        Ok(TokenReclamationStats {
            shortcoder_tokens_in_total: sc_in,
            shortcoder_tokens_out_total: sc_out,
            shortcoder_saved_total: sc_in.saturating_sub(sc_out),
            shortcoder_row_count: sc_rows,
            context_reduction_before_total: cr_before,
            context_reduction_after_total: cr_after,
            context_reduction_saved_total: cr_before.saturating_sub(cr_after),
            context_reduction_row_count: cr_rows,
        })
    }

    /// Latest VRAM tier detection (Tier 3 Provider/VRAM drawer).
    pub fn vram_tier_state(&self) -> Result<VramTierSnapshot, SsbViewerError> {
        let conn = self.lock()?;
        let mut stmt = conn.prepare(
            "SELECT created_at, payload FROM ssb \
             WHERE action_type = ?1 ORDER BY seq DESC LIMIT 1",
        )?;
        let row = stmt
            .query_row(
                params![SsbActionType::VramTierDetected.as_sql_str()],
                |row| {
                    Ok((row.get::<_, i64>(0)?, row.get::<_, Vec<u8>>(1)?))
                },
            )
            .map(Some)
            .or_else(|e| match e {
                rusqlite::Error::QueryReturnedNoRows => Ok(None),
                other => Err(other),
            })?;
        let Some((ts_ms, bytes)) = row else {
            return Ok(VramTierSnapshot {
                tier: "unknown".to_string(),
                detected_via: "unknown".to_string(),
                free_mb: None,
                ts_ms: 0,
                known: false,
            });
        };
        let payload: SsbPayload = rmp_serde::from_slice(&bytes)?;
        if let SsbPayload::VramTierDetected(v) = payload {
            return Ok(VramTierSnapshot {
                tier: v.tier,
                detected_via: v.detected_via,
                free_mb: v.free_mb,
                ts_ms,
                known: true,
            });
        }
        Ok(VramTierSnapshot {
            tier: "unknown".to_string(),
            detected_via: "unknown".to_string(),
            free_mb: None,
            ts_ms: 0,
            known: false,
        })
    }

    /// Knowledge engine readiness snapshot (Tier 3 Knowledge drawer).
    /// Assembles the latest `DkgReady` + latest `IndexingProgress` rows.
    pub fn knowledge_readiness(
        &self,
    ) -> Result<KnowledgeReadinessSnapshot, SsbViewerError> {
        let conn = self.lock()?;
        let dkg = Self::latest_dkg_ready_locked(&conn)?;
        let idx = Self::latest_indexing_progress_locked(&conn)?;
        let dkg_ready = dkg.is_some();
        let (stage, files_processed, files_total, project_id, last_ts_ms) =
            match idx {
                Some((ts, p)) => {
                    (p.stage, p.files_processed, p.files_total, p.project_id, ts)
                }
                None => (
                    "not_started".to_string(),
                    0,
                    0,
                    String::new(),
                    0,
                ),
            };
        let overall = if dkg_ready && stage == "complete" {
            "READY"
        } else if !dkg_ready && files_processed == 0 && files_total == 0 {
            "UNAVAILABLE"
        } else {
            "DEGRADED"
        };
        Ok(KnowledgeReadinessSnapshot {
            overall: overall.to_string(),
            dkg_ready,
            indexing_stage: stage,
            files_processed,
            files_total,
            last_project_id: project_id,
            last_ts_ms,
        })
    }

    /// DKG stats from the latest `DkgReady` row (Tier 3 Knowledge drawer).
    pub fn dkg_stats(&self) -> Result<DkgStatsSnapshot, SsbViewerError> {
        let conn = self.lock()?;
        let dkg = Self::latest_dkg_ready_locked(&conn)?;
        match dkg {
            Some((ts_ms, d)) => Ok(DkgStatsSnapshot {
                project_id: d.project_id,
                node_count: d.node_count,
                edge_count: d.edge_count,
                languages: d.languages,
                ts_ms,
                known: true,
            }),
            None => Ok(DkgStatsSnapshot {
                project_id: String::new(),
                node_count: 0,
                edge_count: 0,
                languages: Vec::new(),
                ts_ms: 0,
                known: false,
            }),
        }
    }

    /// BM25 stats from the latest `IndexingProgress` row (Tier 3 Knowledge).
    pub fn bm25_stats(&self) -> Result<Bm25StatsSnapshot, SsbViewerError> {
        let conn = self.lock()?;
        let idx = Self::latest_indexing_progress_locked(&conn)?;
        match idx {
            Some((ts_ms, p)) => Ok(Bm25StatsSnapshot {
                project_id: p.project_id,
                files_indexed: p.files_processed,
                files_total: p.files_total,
                last_stage: p.stage,
                ts_ms,
                known: true,
            }),
            None => Ok(Bm25StatsSnapshot {
                project_id: String::new(),
                files_indexed: 0,
                files_total: 0,
                last_stage: "not_started".to_string(),
                ts_ms: 0,
                known: false,
            }),
        }
    }

    /// Vector store size estimate (Tier 3 Knowledge drawer). Derived from
    /// the latest `IndexingProgress` row as a coarse best-effort estimate;
    /// no SSB row type emits per-embedding counts at present.
    pub fn vector_stats(&self) -> Result<VectorStatsSnapshot, SsbViewerError> {
        let conn = self.lock()?;
        let idx = Self::latest_indexing_progress_locked(&conn)?;
        match idx {
            Some((ts_ms, p)) => Ok(VectorStatsSnapshot {
                project_id: p.project_id,
                docs_estimated: p.files_processed,
                ts_ms,
                known: true,
            }),
            None => Ok(VectorStatsSnapshot {
                project_id: String::new(),
                docs_estimated: 0,
                ts_ms: 0,
                known: false,
            }),
        }
    }

    /// Embedding/reranker circuit-breaker snapshots (Tier 3 Knowledge drawer).
    ///
    /// B-4 SCHEMA: reads `circuit_breaker_state_change` SSB rows (v25->v26 bump
    /// coordinated with B-6/B-9). Returns the latest row per `model` field.
    /// When no rows exist (schema bump not yet deployed), returns empty Vec so
    /// the dashboard falls back to its hardcoded stubs gracefully.
    pub fn embedding_breakers(&self) -> Result<Vec<EmbreakerSnapshot>, SsbViewerError> {
        let conn = self.lock()?;
        // Pull the 64 most recent circuit_breaker_state_change rows and
        // deduplicate to latest-per-model. 64 rows bounds the scan even on
        // busy SSBs (5 models, each flipping O(10) times per run at most).
        let mut stmt = conn.prepare(
            "SELECT created_at, payload FROM ssb \
             WHERE action_type = ?1 ORDER BY seq DESC LIMIT 64",
        )?;
        let rows_iter = stmt.query_map(
            params![crate::ssb::SsbActionType::CircuitBreakerStateChange.as_sql_str()],
            |row| Ok((row.get::<_, i64>(0)?, row.get::<_, Vec<u8>>(1)?)),
        )?;
        let mut rows: Vec<(i64, Vec<u8>)> = Vec::new();
        for row in rows_iter {
            rows.push(row?);
        }

        // Deserialize and deduplicate: keep first (= latest) row per model.
        use indexmap::IndexMap;
        let mut by_model: IndexMap<String, EmbreakerSnapshot> = IndexMap::new();
        for (_ts, bytes) in rows {
            if let Ok(crate::ssb::SsbPayload::CircuitBreakerStateChange(p)) =
                rmp_serde::from_slice::<crate::ssb::SsbPayload>(&bytes)
            {
                by_model.entry(p.model.clone()).or_insert(EmbreakerSnapshot {
                    model: p.model,
                    state: p.state,
                    failures: p.failures,
                    success_rate: p.success_rate,
                    last_failure_ts: p.last_failure_ts,
                });
            }
        }
        Ok(by_model.into_values().collect())
    }

    /// PageRank top-N stub (Tier 3 Knowledge drawer). No SSB row type
    /// currently persists per-symbol PageRank scores; the viewer returns
    /// an empty list so the drawer renders a graceful "not available".
    pub fn pagerank_top(
        &self,
        _n: usize,
    ) -> Result<Vec<PageRankEntry>, SsbViewerError> {
        Ok(Vec::new())
    }

    /// Query tier stats stub (Tier 3 Knowledge drawer). Reserved until a
    /// dedicated SSB row records per-query tier decisions.
    pub fn query_tier_stats(&self) -> Result<QueryTierStats, SsbViewerError> {
        Ok(QueryTierStats {
            fast_pct: 0.0,
            standard_pct: 0.0,
            deep_pct: 0.0,
            avg_latency_ms_fast: 0.0,
            avg_latency_ms_standard: 0.0,
            avg_latency_ms_deep: 0.0,
            sample_size: 0,
        })
    }

    /// Freshness state stub (Tier 3 Knowledge drawer). The freshness meta
    /// DB does not expose a read endpoint yet; default values reported.
    pub fn freshness_state(&self) -> Result<FreshnessState, SsbViewerError> {
        Ok(FreshnessState {
            decay_rate: 0.0,
            access_boost: 0.0,
            last_recalc_ms: 0,
            known: false,
        })
    }

    /// Per-dispatch ContextBudget snapshot from the latest `ContextBudgetComputed`
    /// row (Tier 3 Orchestration drawer, B-10 producer path).
    ///
    /// Returns `Ok(Some(payload))` when at least one `context_budget_computed` row
    /// exists, `Ok(None)` when the SSB has no such row yet (pre-task state), and
    /// `Err(SsbViewerError)` on a SQLite or deserialisation failure.
    ///
    /// The payload carries per-slot `allocations` (real token counts) and `slots`
    /// (basis-point weights 0..=10000) so the dashboard bar chart can render both
    /// the absolute allocation and the fractional weight without re-deriving either.
    pub fn context_budget_computed_snapshot(
        &self,
    ) -> Result<Option<ContextBudgetComputedPayload>, SsbViewerError> {
        let conn = self.lock()?;
        let mut stmt = conn.prepare(
            "SELECT payload FROM ssb \
             WHERE action_type = ?1 ORDER BY seq DESC LIMIT 1",
        )?;
        let row: Option<Vec<u8>> = stmt
            .query_row(
                params![SsbActionType::ContextBudgetComputed.as_sql_str()],
                |row| row.get::<_, Vec<u8>>(0),
            )
            .map(Some)
            .or_else(|e| match e {
                rusqlite::Error::QueryReturnedNoRows => Ok(None),
                other => Err(other),
            })?;
        let Some(bytes) = row else {
            return Ok(None);
        };
        let payload: SsbPayload = rmp_serde::from_slice(&bytes)?;
        if let SsbPayload::ContextBudgetComputed(p) = payload {
            return Ok(Some(p));
        }
        Ok(None)
    }

    /// Context budget snapshot from the latest `ContextReduction` row
    /// (Tier 3 Orchestration drawer).
    pub fn context_budget_snapshot(
        &self,
    ) -> Result<ContextBudgetSnapshot, SsbViewerError> {
        let conn = self.lock()?;
        let mut stmt = conn.prepare(
            "SELECT created_at, payload FROM ssb \
             WHERE action_type = ?1 ORDER BY seq DESC LIMIT 1",
        )?;
        let row = stmt
            .query_row(
                params![SsbActionType::ContextReduction.as_sql_str()],
                |row| {
                    Ok((row.get::<_, i64>(0)?, row.get::<_, Vec<u8>>(1)?))
                },
            )
            .map(Some)
            .or_else(|e| match e {
                rusqlite::Error::QueryReturnedNoRows => Ok(None),
                other => Err(other),
            })?;
        let Some((ts_ms, bytes)) = row else {
            return Ok(ContextBudgetSnapshot {
                level: 0,
                before_tokens: 0,
                after_tokens: 0,
                slots_affected: Vec::new(),
                ts_ms: 0,
                known: false,
            });
        };
        let payload: SsbPayload = rmp_serde::from_slice(&bytes)?;
        if let SsbPayload::ContextReduction(cr) = payload {
            return Ok(ContextBudgetSnapshot {
                level: cr.level,
                before_tokens: cr.before_tokens as u64,
                after_tokens: cr.after_tokens as u64,
                slots_affected: cr.slots_affected,
                ts_ms,
                known: true,
            });
        }
        Ok(ContextBudgetSnapshot {
            level: 0,
            before_tokens: 0,
            after_tokens: 0,
            slots_affected: Vec::new(),
            ts_ms: 0,
            known: false,
        })
    }

    /// Agent checkpoints (Tier 3 Orchestration drawer). Reads `Checkpoint`
    /// SSB rows; aggregates `file_sizes` into a single total so the drawer
    /// row stays compact.
    pub fn agent_checkpoints(
        &self,
        limit: usize,
    ) -> Result<Vec<AgentCheckpointEntry>, SsbViewerError> {
        let effective = limit.min(DRAWER_QUERY_HARD_CAP);
        let limit_i64 = i64::try_from(effective).unwrap_or(i64::MAX);
        let conn = self.lock()?;
        let mut stmt = conn.prepare(
            "SELECT seq, created_at, payload FROM ssb \
             WHERE action_type = ?1 ORDER BY seq DESC LIMIT ?2",
        )?;
        let rows = stmt.query_map(
            params![SsbActionType::Checkpoint.as_sql_str(), limit_i64],
            |row| {
                Ok((
                    row.get::<_, i64>(0)?,
                    row.get::<_, i64>(1)?,
                    row.get::<_, Vec<u8>>(2)?,
                ))
            },
        )?;
        let mut out: Vec<AgentCheckpointEntry> = Vec::new();
        for row in rows {
            let (seq, ts_ms, bytes) = row?;
            let payload: SsbPayload = match rmp_serde::from_slice(&bytes) {
                Ok(p) => p,
                Err(_) => continue,
            };
            if let SsbPayload::Checkpoint(c) = payload {
                let total: u64 = c
                    .file_sizes
                    .iter()
                    .map(|(_, size)| *size)
                    .fold(0u64, |acc, s| acc.saturating_add(s));
                out.push(AgentCheckpointEntry {
                    seq,
                    ts_ms,
                    agent_id: c.agent_id,
                    step_index: c.step_index,
                    state_hash: c.agent_state_hash,
                    file_sizes_total: total,
                });
            }
        }
        Ok(out)
    }

    /// Session handoffs (Tier 3 Orchestration drawer). Derived from
    /// `BranchMerge` SSB rows — each row describes a handoff boundary
    /// where the winner name survived and the abandoned names did not.
    pub fn session_handoffs(
        &self,
        limit: usize,
    ) -> Result<Vec<SessionHandoffEntry>, SsbViewerError> {
        let effective = limit.min(DRAWER_QUERY_HARD_CAP);
        let limit_i64 = i64::try_from(effective).unwrap_or(i64::MAX);
        let conn = self.lock()?;
        let mut stmt = conn.prepare(
            "SELECT seq, created_at, payload FROM ssb \
             WHERE action_type = ?1 ORDER BY seq DESC LIMIT ?2",
        )?;
        let rows = stmt.query_map(
            params![SsbActionType::BranchMerge.as_sql_str(), limit_i64],
            |row| {
                Ok((
                    row.get::<_, i64>(0)?,
                    row.get::<_, i64>(1)?,
                    row.get::<_, Vec<u8>>(2)?,
                ))
            },
        )?;
        let mut out: Vec<SessionHandoffEntry> = Vec::new();
        for row in rows {
            let (seq, ts_ms, bytes) = row?;
            let payload: SsbPayload = match rmp_serde::from_slice(&bytes) {
                Ok(p) => p,
                Err(_) => continue,
            };
            if let SsbPayload::BranchMerge(b) = payload {
                let from_count =
                    u32::try_from(b.from_snap_ids.len()).unwrap_or(u32::MAX);
                out.push(SessionHandoffEntry {
                    seq,
                    ts_ms,
                    winner: b.winner_name,
                    abandoned: b.abandoned_names,
                    trunk_snap_id: b.trunk_snap_id,
                    from_snap_count: from_count,
                });
            }
        }
        Ok(out)
    }

    /// Per-tier web validation results (Tier 3 WebQuality drawer). The
    /// `ValidationTier` payload has no `project_id` field; the optional
    /// filter is therefore unused here and retained in the signature for
    /// API stability — future writer-side extensions can honour it
    /// without breaking the Tauri contract.
    pub fn web_validation_results(
        &self,
        _project_id: Option<&str>,
        limit: usize,
    ) -> Result<Vec<ValidationTierEntry>, SsbViewerError> {
        let effective = limit.min(DRAWER_QUERY_HARD_CAP);
        let limit_i64 = i64::try_from(effective).unwrap_or(i64::MAX);
        let conn = self.lock()?;
        let mut stmt = conn.prepare(
            "SELECT seq, created_at, payload FROM ssb \
             WHERE action_type = ?1 ORDER BY seq DESC LIMIT ?2",
        )?;
        let rows = stmt.query_map(
            params![SsbActionType::ValidationTier.as_sql_str(), limit_i64],
            |row| {
                Ok((
                    row.get::<_, i64>(0)?,
                    row.get::<_, i64>(1)?,
                    row.get::<_, Vec<u8>>(2)?,
                ))
            },
        )?;
        let mut out: Vec<ValidationTierEntry> = Vec::new();
        for row in rows {
            let (seq, ts_ms, bytes) = row?;
            let payload: SsbPayload = match rmp_serde::from_slice(&bytes) {
                Ok(p) => p,
                Err(_) => continue,
            };
            if let SsbPayload::ValidationTier(v) = payload {
                out.push(ValidationTierEntry {
                    seq,
                    ts_ms,
                    tier: v.tier,
                    passed: v.passed,
                    elapsed_ms: v.elapsed_ms,
                    rules_fired: v.rules_fired,
                    step_id: v.step_id,
                });
            }
        }
        Ok(out)
    }

    /// Recent research steps for the Research drawer (Tier 3 complete).
    ///
    /// Reads the two research-related SSB action types — `web_research_step`
    /// and `research_step` — in a single UNION query, newest-first, capped
    /// at `DRAWER_QUERY_HARD_CAP`. The `kind` discriminator on the returned
    /// entry is `"web"` for WebResearchStep rows and `"local"` for
    /// ResearchStep rows; source-fetch counters are zero for the `local`
    /// variant (the local payload does not carry them).
    pub fn research_steps(
        &self,
        limit: usize,
    ) -> Result<Vec<ResearchStepEntry>, SsbViewerError> {
        let effective = limit.min(DRAWER_QUERY_HARD_CAP);
        let limit_i64 = i64::try_from(effective).unwrap_or(i64::MAX);
        let conn = self.lock()?;
        let mut stmt = conn.prepare(
            "SELECT seq, created_at, action_type, payload FROM ssb \
             WHERE action_type IN (?1, ?2) ORDER BY seq DESC LIMIT ?3",
        )?;
        let rows = stmt.query_map(
            params![
                SsbActionType::WebResearchStep.as_sql_str(),
                SsbActionType::ResearchStep.as_sql_str(),
                limit_i64,
            ],
            |row| {
                Ok((
                    row.get::<_, i64>(0)?,
                    row.get::<_, i64>(1)?,
                    row.get::<_, String>(2)?,
                    row.get::<_, Vec<u8>>(3)?,
                ))
            },
        )?;
        let mut out: Vec<ResearchStepEntry> = Vec::new();
        for row in rows {
            let (seq, ts_ms, _action_type, bytes) = row?;
            let payload: SsbPayload = match rmp_serde::from_slice(&bytes) {
                Ok(p) => p,
                Err(_) => continue,
            };
            match payload {
                SsbPayload::WebResearchStep(w) => out.push(ResearchStepEntry {
                    seq,
                    ts_ms,
                    kind: "web".to_string(),
                    agent_id: w.agent_id,
                    node_id: w.node_id,
                    action: w.action,
                    phi: w.phi,
                    psi: w.psi,
                    iteration: w.iteration,
                    sources_fetched: w.sources_fetched,
                    sources_used: w.sources_used,
                    stage: w.stage,
                    stage_phase: w.stage_phase,
                    stage_payload: w.stage_payload,
                }),
                SsbPayload::ResearchStep(r) => out.push(ResearchStepEntry {
                    seq,
                    ts_ms,
                    kind: "local".to_string(),
                    agent_id: r.agent_id,
                    node_id: r.node_id,
                    action: r.action,
                    phi: r.phi,
                    psi: r.psi,
                    iteration: r.iteration,
                    sources_fetched: 0,
                    sources_used: 0,
                    stage: String::new(),
                    stage_phase: String::new(),
                    stage_payload: None,
                }),
                _ => continue,
            }
        }
        Ok(out)
    }

    /// Research budget snapshot stub (Tier 3 complete Research drawer).
    ///
    /// `ResearchBudgetManager` is in-memory only; no SSB row type carries
    /// its progression yet. The viewer therefore returns zeroes with
    /// `known: false` and the dashboard renders "no data yet" in the
    /// drawer body. Candidate writer-side row types documented at
    /// DEF-T3-RESEARCH-1 of the dashboard-wiring plan.
    pub fn research_budget_snapshot(
        &self,
    ) -> Result<ResearchBudgetSnapshot, SsbViewerError> {
        Ok(ResearchBudgetSnapshot {
            admitted: 0,
            pruned: 0,
            tokens_used: 0,
            tokens_max: 0,
            known: false,
        })
    }

    /// Source-tier distribution stub (Tier 3 complete Research drawer).
    ///
    /// The 5-tier domain lookup (`source_quality.rs`) is deterministic but
    /// its per-research counts are not persisted. Returns zeroes with
    /// `known: false` (D3-3 precedent). DEF-T3-RESEARCH-2 tracks the
    /// candidate writer-side row type.
    pub fn source_tier_distribution(
        &self,
    ) -> Result<SourceTierDistribution, SsbViewerError> {
        Ok(SourceTierDistribution {
            t1: 0,
            t2: 0,
            t3: 0,
            t4: 0,
            t5: 0,
            known: false,
        })
    }

    /// Track A.6 — aggregate grounding cache statistics from SSB rows.
    ///
    /// Scans the most recent 2000 `grounding_cache_event` rows and computes:
    /// - `hits` / `misses` counts,
    /// - `total_keys`: distinct key fingerprints observed (lower-bound, capped),
    /// - `last_hit_us` / `last_miss_us`: elapsed_us of the most recent hit/miss.
    ///
    /// Returns zeroes if no rows exist yet.
    pub fn grounding_cache_stats(
        &self,
    ) -> Result<GroundingCacheStats, SsbViewerError> {
        const SCAN_LIMIT: i64 = 2000;
        let conn = self.lock()?;
        let mut stmt = conn.prepare(
            "SELECT payload FROM ssb \
             WHERE action_type = 'grounding_cache_event' \
             ORDER BY seq DESC LIMIT ?1",
        )?;
        let bytes_iter = stmt.query_map(params![SCAN_LIMIT], |row| {
            row.get::<_, Vec<u8>>(0)
        })?;

        let mut hits: u32 = 0;
        let mut misses: u32 = 0;
        let mut last_hit_us: u64 = 0;
        let mut last_miss_us: u64 = 0;
        let mut first_hit = true;
        let mut first_miss = true;
        let mut fingerprints = std::collections::BTreeSet::<String>::new();

        for bytes in bytes_iter {
            let bytes = bytes?;
            let payload: SsbPayload = match rmp_serde::from_slice(&bytes) {
                Ok(p) => p,
                Err(_) => continue,
            };
            if let SsbPayload::GroundingCacheEvent(ev) = payload {
                fingerprints.insert(ev.key_fingerprint.clone());
                match ev.kind.as_str() {
                    "hit" => {
                        hits += 1;
                        if first_hit {
                            last_hit_us = ev.elapsed_us;
                            first_hit = false;
                        }
                    }
                    "miss" => {
                        misses += 1;
                        if first_miss {
                            last_miss_us = ev.elapsed_us;
                            first_miss = false;
                        }
                    }
                    _ => {}
                }
            }
        }
        Ok(GroundingCacheStats {
            // NARROWING-OK: fingerprint count is bounded by DB capacity (well under u32::MAX ~4B rows).
            total_keys: fingerprints.len() as u32,
            hits,
            misses,
            last_hit_us,
            last_miss_us,
        })
    }

    // ----- Tier 3 internal helpers ------------------------------------------

    fn latest_dkg_ready_locked(
        conn: &Connection,
    ) -> Result<Option<(i64, crate::ssb::DkgReadyPayload)>, SsbViewerError> {
        let mut stmt = conn.prepare(
            "SELECT created_at, payload FROM ssb \
             WHERE action_type = ?1 ORDER BY seq DESC LIMIT 1",
        )?;
        let row = stmt
            .query_row(
                params![SsbActionType::DkgReady.as_sql_str()],
                |row| {
                    Ok((row.get::<_, i64>(0)?, row.get::<_, Vec<u8>>(1)?))
                },
            )
            .map(Some)
            .or_else(|e| match e {
                rusqlite::Error::QueryReturnedNoRows => Ok(None),
                other => Err(other),
            })?;
        let Some((ts, bytes)) = row else {
            return Ok(None);
        };
        let payload: SsbPayload = rmp_serde::from_slice(&bytes)?;
        if let SsbPayload::DkgReady(d) = payload {
            return Ok(Some((ts, d)));
        }
        Ok(None)
    }

    fn latest_indexing_progress_locked(
        conn: &Connection,
    ) -> Result<
        Option<(i64, crate::ssb::IndexingProgressPayload)>,
        SsbViewerError,
    > {
        let mut stmt = conn.prepare(
            "SELECT created_at, payload FROM ssb \
             WHERE action_type = ?1 ORDER BY seq DESC LIMIT 1",
        )?;
        let row = stmt
            .query_row(
                params![SsbActionType::IndexingProgress.as_sql_str()],
                |row| {
                    Ok((row.get::<_, i64>(0)?, row.get::<_, Vec<u8>>(1)?))
                },
            )
            .map(Some)
            .or_else(|e| match e {
                rusqlite::Error::QueryReturnedNoRows => Ok(None),
                other => Err(other),
            })?;
        let Some((ts, bytes)) = row else {
            return Ok(None);
        };
        let payload: SsbPayload = rmp_serde::from_slice(&bytes)?;
        if let SsbPayload::IndexingProgress(p) = payload {
            return Ok(Some((ts, p)));
        }
        Ok(None)
    }

    // -----------------------------------------------------------------------
    // B-6: Orchestration drawer query methods
    // -----------------------------------------------------------------------

    /// Returns up to `limit` recent `evidence_gate_evaluated` rows in
    /// descending seq order (most recent first).
    pub fn recent_evidence_gates(
        &self,
        limit: usize,
    ) -> Result<Vec<crate::ssb::EvidenceGateEvaluatedPayload>, SsbViewerError> {
        let capped = limit.min(DRAWER_QUERY_HARD_CAP);
        let limit_i64 =
            i64::try_from(capped).map_err(|e| SsbViewerError::OutOfRange(e.to_string()))?;
        let conn = self.lock()?;
        let mut stmt = conn.prepare(
            "SELECT payload FROM ssb \
             WHERE action_type = ?1 ORDER BY seq DESC LIMIT ?2",
        )?;
        let rows = stmt.query_map(
            params![
                SsbActionType::EvidenceGateEvaluated.as_sql_str(),
                limit_i64
            ],
            |row| row.get::<_, Vec<u8>>(0),
        )?;
        let mut out = Vec::new();
        for row in rows {
            let bytes = row?;
            let payload: SsbPayload = rmp_serde::from_slice(&bytes)?;
            if let SsbPayload::EvidenceGateEvaluated(p) = payload {
                out.push(p);
            }
        }
        Ok(out)
    }

    /// Returns up to `limit` recent `supervision_event` rows in descending
    /// seq order (most recent first).
    pub fn recent_supervision_events(
        &self,
        limit: usize,
    ) -> Result<Vec<crate::ssb::SupervisionEventPayload>, SsbViewerError> {
        let capped = limit.min(DRAWER_QUERY_HARD_CAP);
        let limit_i64 =
            i64::try_from(capped).map_err(|e| SsbViewerError::OutOfRange(e.to_string()))?;
        let conn = self.lock()?;
        let mut stmt = conn.prepare(
            "SELECT payload FROM ssb \
             WHERE action_type = ?1 ORDER BY seq DESC LIMIT ?2",
        )?;
        let rows = stmt.query_map(
            params![SsbActionType::SupervisionEvent.as_sql_str(), limit_i64],
            |row| row.get::<_, Vec<u8>>(0),
        )?;
        let mut out = Vec::new();
        for row in rows {
            let bytes = row?;
            let payload: SsbPayload = rmp_serde::from_slice(&bytes)?;
            if let SsbPayload::SupervisionEvent(p) = payload {
                out.push(p);
            }
        }
        Ok(out)
    }

    /// Returns up to `limit` recent `bsp_superstep` rows in descending
    /// seq order (most recent first).
    pub fn recent_bsp_supersteps(
        &self,
        limit: usize,
    ) -> Result<Vec<crate::ssb::BspSuperstepPayload>, SsbViewerError> {
        let capped = limit.min(DRAWER_QUERY_HARD_CAP);
        let limit_i64 =
            i64::try_from(capped).map_err(|e| SsbViewerError::OutOfRange(e.to_string()))?;
        let conn = self.lock()?;
        let mut stmt = conn.prepare(
            "SELECT payload FROM ssb \
             WHERE action_type = ?1 ORDER BY seq DESC LIMIT ?2",
        )?;
        let rows = stmt.query_map(
            params![SsbActionType::BspSuperstep.as_sql_str(), limit_i64],
            |row| row.get::<_, Vec<u8>>(0),
        )?;
        let mut out = Vec::new();
        for row in rows {
            let bytes = row?;
            let payload: SsbPayload = rmp_serde::from_slice(&bytes)?;
            if let SsbPayload::BspSuperstep(p) = payload {
                out.push(p);
            }
        }
        Ok(out)
    }

    // -----------------------------------------------------------------------
    // B-7b: Bubble-up + DAG repair drawer query methods
    // -----------------------------------------------------------------------

    /// Returns up to `limit` recent `bubble_up_typed` rows in descending seq
    /// order (most recent first). Used by the dashboard BUBBLE_UP drawer.
    pub fn recent_bubble_up_events(
        &self,
        limit: usize,
    ) -> Result<Vec<BubbleUpTypedPayload>, SsbViewerError> {
        let capped = limit.min(DRAWER_QUERY_HARD_CAP);
        let limit_i64 =
            i64::try_from(capped).map_err(|e| SsbViewerError::OutOfRange(e.to_string()))?;
        let conn = self.lock()?;
        let mut stmt = conn.prepare(
            "SELECT payload FROM ssb \
             WHERE action_type = ?1 ORDER BY seq DESC LIMIT ?2",
        )?;
        let rows = stmt.query_map(
            params![SsbActionType::BubbleUpTyped.as_sql_str(), limit_i64],
            |row| row.get::<_, Vec<u8>>(0),
        )?;
        let mut out = Vec::new();
        for row in rows {
            let bytes = row?;
            let payload: SsbPayload = rmp_serde::from_slice(&bytes)?;
            if let SsbPayload::BubbleUpTyped(p) = payload {
                out.push(p);
            }
        }
        Ok(out)
    }

    /// Returns up to `limit` recent `dag_repair_round` rows in descending seq
    /// order (most recent first). Used by the dashboard DAG_REPAIR drawer.
    pub fn recent_dag_repair_rounds(
        &self,
        limit: usize,
    ) -> Result<Vec<DagRepairRoundPayload>, SsbViewerError> {
        let capped = limit.min(DRAWER_QUERY_HARD_CAP);
        let limit_i64 =
            i64::try_from(capped).map_err(|e| SsbViewerError::OutOfRange(e.to_string()))?;
        let conn = self.lock()?;
        let mut stmt = conn.prepare(
            "SELECT payload FROM ssb \
             WHERE action_type = ?1 ORDER BY seq DESC LIMIT ?2",
        )?;
        let rows = stmt.query_map(
            params![SsbActionType::DagRepairRound.as_sql_str(), limit_i64],
            |row| row.get::<_, Vec<u8>>(0),
        )?;
        let mut out = Vec::new();
        for row in rows {
            let bytes = row?;
            let payload: SsbPayload = rmp_serde::from_slice(&bytes)?;
            if let SsbPayload::DagRepairRound(p) = payload {
                out.push(p);
            }
        }
        Ok(out)
    }

    /// B-22 Phase 4G — aggregate AutoAdvanceStats from `GateDecision` rows.
    ///
    /// Scans every `GateDecision` row newest-first up to a hard cap of 5000
    /// rows. Older history is irrelevant to the "did the gate auto-advance
    /// recently" signal so an unbounded scan would be wasted I/O. The
    /// `now_ms` argument is wall-clock UTC ms used to filter the trailing
    /// 24-hour window — passed in by the caller so the dashboard's
    /// `ClockProvider` controls deterministic-replay clock sources.
    pub fn auto_advance_stats(
        &self,
        now_ms: i64,
    ) -> Result<AutoAdvanceStats, SsbViewerError> {
        const MAX_ROWS: usize = 5_000;
        const DAY_MS: i64 = 24 * 60 * 60 * 1_000;
        let last_24h_threshold = now_ms.saturating_sub(DAY_MS);

        let conn = self.lock()?;
        let mut stmt = conn.prepare(
            "SELECT created_at, payload FROM ssb \
             WHERE action_type = ?1 ORDER BY seq DESC LIMIT ?2",
        )?;
        // NARROWING-OK: const literal MAX_ROWS=5000 fits i64 trivially.
        let rows = stmt.query_map(
            params![SsbActionType::GateDecision.as_sql_str(), MAX_ROWS as i64],
            |row| {
                Ok((
                    row.get::<_, i64>(0)?,
                    row.get::<_, Vec<u8>>(1)?,
                ))
            },
        )?;

        let mut total_all: u64 = 0;
        let mut auto_all: u64 = 0;
        let mut intervention_all: u64 = 0;
        let mut total_24h: u64 = 0;
        let mut auto_24h: u64 = 0;

        for row in rows {
            let (created_at, bytes) = row?;
            let Ok(payload) = rmp_serde::from_slice::<SsbPayload>(&bytes) else {
                continue;
            };
            let SsbPayload::GateDecision(p) = payload else {
                continue;
            };
            total_all = total_all.saturating_add(1);
            let in_24h = created_at >= last_24h_threshold;
            if in_24h {
                total_24h = total_24h.saturating_add(1);
            }
            match p.decision.as_str() {
                "auto_advance" => {
                    auto_all = auto_all.saturating_add(1);
                    if in_24h {
                        auto_24h = auto_24h.saturating_add(1);
                    }
                }
                "request_human" => {
                    intervention_all = intervention_all.saturating_add(1);
                }
                _ => {}
            }
        }

        let auto_advance_rate = if total_all == 0 {
            0.0_f64
        } else {
            // NARROWING-OK: counters are bounded by MAX_ROWS=5000; f64 mantissa is 53 bits.
            auto_all as f64 / total_all as f64
        };
        let last_24h_rate = if total_24h == 0 {
            0.0_f64
        } else {
            auto_24h as f64 / total_24h as f64
        };

        Ok(AutoAdvanceStats {
            total_steps: total_all,
            auto_advanced: auto_all,
            intervention_required: intervention_all,
            auto_advance_rate,
            last_24h_rate,
        })
    }

    /// B-23 Phase 4H Group F (4H.8) — pending Layer 1A proposals (proposals
    /// emitted via `proposal_generated` that have NOT yet been resolved by
    /// a later `proposal_applied` or `proposal_rejected` row with the same
    /// proposal_id). Newest-first; capped at `DRAWER_QUERY_HARD_CAP`.
    pub fn pending_proposals_layer1a(&self) -> Result<Vec<SelfModProposalRow>, SsbViewerError> {
        let conn = self.lock()?;
        // Resolved set = applied ∪ rejected proposal_ids.
        let mut resolved: std::collections::BTreeSet<String> = std::collections::BTreeSet::new();
        for action in [SsbActionType::ProposalApplied, SsbActionType::ProposalRejected] {
            let mut stmt = conn.prepare(
                "SELECT payload FROM ssb WHERE action_type = ?1 ORDER BY seq DESC LIMIT ?2",
            )?;
            let rows = stmt.query_map(
                params![action.as_sql_str(), DRAWER_QUERY_HARD_CAP as i64],
                |row| row.get::<_, Vec<u8>>(0),
            )?;
            for row in rows {
                let bytes = row?;
                let Ok(payload): Result<SsbPayload, _> = rmp_serde::from_slice(&bytes) else {
                    continue;
                };
                match payload {
                    SsbPayload::ProposalApplied(p) => {
                        resolved.insert(p.proposal_id);
                    }
                    SsbPayload::ProposalRejected(p) => {
                        resolved.insert(p.proposal_id);
                    }
                    _ => {}
                }
            }
        }

        let mut stmt = conn.prepare(
            "SELECT seq, created_at, payload FROM ssb \
             WHERE action_type = ?1 ORDER BY seq DESC LIMIT ?2",
        )?;
        let rows = stmt.query_map(
            params![
                SsbActionType::ProposalGenerated.as_sql_str(),
                DRAWER_QUERY_HARD_CAP as i64,
            ],
            |row| {
                Ok((
                    row.get::<_, i64>(0)?,
                    row.get::<_, i64>(1)?,
                    row.get::<_, Vec<u8>>(2)?,
                ))
            },
        )?;
        let mut out: Vec<SelfModProposalRow> = Vec::new();
        let mut seen: std::collections::BTreeSet<String> = std::collections::BTreeSet::new();
        for row in rows {
            let (_seq, ts_ms, bytes) = row?;
            let Ok(payload): Result<SsbPayload, _> = rmp_serde::from_slice(&bytes) else {
                continue;
            };
            if let SsbPayload::ProposalGenerated(p) = payload {
                if resolved.contains(&p.proposal_id) {
                    continue;
                }
                if !seen.insert(p.proposal_id.clone()) {
                    continue;
                }
                out.push(SelfModProposalRow {
                    proposal_id: p.proposal_id,
                    table: p.table,
                    fpr_estimate: Some(p.fpr_estimate),
                    occurrences_seen: None,
                    pipeline_mode: String::new(),
                    applied_by: None,
                    ts_ms,
                });
            }
        }
        Ok(out)
    }

    /// B-23 Phase 4H Group F (4H.8) — applied Layer 1A proposals history.
    pub fn applied_proposals_layer1a(&self) -> Result<Vec<SelfModProposalRow>, SsbViewerError> {
        let conn = self.lock()?;
        let mut stmt = conn.prepare(
            "SELECT seq, created_at, payload FROM ssb \
             WHERE action_type = ?1 ORDER BY seq DESC LIMIT ?2",
        )?;
        let rows = stmt.query_map(
            params![
                SsbActionType::ProposalApplied.as_sql_str(),
                DRAWER_QUERY_HARD_CAP as i64,
            ],
            |row| {
                Ok((
                    row.get::<_, i64>(0)?,
                    row.get::<_, i64>(1)?,
                    row.get::<_, Vec<u8>>(2)?,
                ))
            },
        )?;
        let mut out: Vec<SelfModProposalRow> = Vec::new();
        for row in rows {
            let (_seq, ts_ms, bytes) = row?;
            let Ok(payload): Result<SsbPayload, _> = rmp_serde::from_slice(&bytes) else {
                continue;
            };
            if let SsbPayload::ProposalApplied(p) = payload {
                out.push(SelfModProposalRow {
                    proposal_id: p.proposal_id,
                    table: p.table,
                    fpr_estimate: None,
                    occurrences_seen: None,
                    pipeline_mode: String::new(),
                    applied_by: Some(p.applied_by),
                    ts_ms,
                });
            }
        }
        Ok(out)
    }

    /// B-23 Phase 4H Group F (4H.8) — latest MSI snapshot per pipeline mode
    /// (newest `msi_update` row for each surgical/greenfield/oracle bucket).
    pub fn latest_msi_per_mode(&self) -> Result<Vec<SelfModMsiSnapshotRow>, SsbViewerError> {
        let conn = self.lock()?;
        let mut stmt = conn.prepare(
            "SELECT seq, created_at, payload FROM ssb \
             WHERE action_type = ?1 ORDER BY seq DESC LIMIT ?2",
        )?;
        let rows = stmt.query_map(
            params![
                SsbActionType::MsiUpdate.as_sql_str(),
                DRAWER_QUERY_HARD_CAP as i64,
            ],
            |row| {
                Ok((
                    row.get::<_, i64>(0)?,
                    row.get::<_, i64>(1)?,
                    row.get::<_, Vec<u8>>(2)?,
                ))
            },
        )?;
        let mut latest: std::collections::BTreeMap<String, SelfModMsiSnapshotRow> =
            std::collections::BTreeMap::new();
        for row in rows {
            let (_seq, ts_ms, bytes) = row?;
            let Ok(payload): Result<SsbPayload, _> = rmp_serde::from_slice(&bytes) else {
                continue;
            };
            if let SsbPayload::MsiUpdate(p) = payload {
                latest.entry(p.pipeline_mode.clone()).or_insert(SelfModMsiSnapshotRow {
                    pipeline_mode: p.pipeline_mode,
                    msi_value: p.msi_value,
                    status: p.status,
                    ts_ms,
                });
            }
        }
        Ok(latest.into_values().collect())
    }
}

/// B-23 Phase 4H Group F — DTO emitted by [`SsbViewer::pending_proposals_layer1a`]
/// and [`SsbViewer::applied_proposals_layer1a`]. Mirrors the dashboard-v2
/// frontend `SelfModProposalRow` interface 1:1.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SelfModProposalRow {
    pub proposal_id: String,
    pub table: String,
    pub fpr_estimate: Option<f32>,
    pub occurrences_seen: Option<u32>,
    pub pipeline_mode: String,
    pub applied_by: Option<String>,
    pub ts_ms: i64,
}

/// B-23 Phase 4H Group F — DTO for [`SsbViewer::latest_msi_per_mode`].
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SelfModMsiSnapshotRow {
    pub pipeline_mode: String,
    pub msi_value: f32,
    pub status: String,
    pub ts_ms: i64,
}

/// Tier 3 partial — hard cap on rows returned by any drawer query
/// regardless of the `limit` argument. Keeps the IPC payload bounded
/// even if a future drawer is wired to an unbounded user control.
const DRAWER_QUERY_HARD_CAP: usize = 500;

/// Tier 3 partial — scan cap for the two-pass token reclamation
/// aggregation. Sized at 10k rows per action type to keep the drawer
/// refresh under the existing 500 ms polling budget on a realistic
/// SSB (approximately 50ms on a 10k row scan in integration tests).
const TOKEN_RECLAMATION_SCAN_CAP: usize = 10_000;

/// Tier 3 partial — thin wrapper around `std::collections::BTreeMap` so
/// `grc_budgets` compiles without pulling in an extra `use` in this
/// module. `BTreeMap` was chosen over `HashMap` for deterministic
/// iteration order (HashMap ban per CLAUDE.md).
type BTreeMapStub<K, V> = std::collections::BTreeMap<K, V>;

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Phase 4F (B-21): map a typed [`FeasibilityReportPayload`] onto the
/// dashboard-facing [`FeasibilityReportRow`]. Pure projection — preserves
/// `skip_reason: Option<String>` so the wire format is null (not undefined)
/// when no skip path was taken.
fn decode_feasibility_report_row(
    seq: i64,
    ts_ms: i64,
    p: FeasibilityReportPayload,
) -> FeasibilityReportRow {
    FeasibilityReportRow {
        seq,
        ts_ms,
        task_feasibility: p.task_feasibility,
        skip_reason: p.skip_reason,
        novel_step_count: p.novel_step_count,
        escalations_count: p.escalations_count,
        vram_margin_bp: p.vram_margin_bp,
        ram_margin_bp: p.ram_margin_bp,
        disk_margin_bp: p.disk_margin_bp,
    }
}

/// Phase 4F (B-21): map a typed [`NovelStepDetectedPayload`] onto the
/// dashboard-facing [`NovelStepRow`].
fn decode_novel_step_row(
    seq: i64,
    ts_ms: i64,
    p: NovelStepDetectedPayload,
) -> NovelStepRow {
    NovelStepRow {
        seq,
        ts_ms,
        step_id: p.step_id,
        reason: p.reason,
        alternative_count: p.alternative_count,
    }
}

/// Phase 4F (B-21): map a typed [`StubRunPayload`] onto the dashboard-facing
/// [`StubRunRow`].
fn decode_stub_run_row(seq: i64, ts_ms: i64, p: StubRunPayload) -> StubRunRow {
    StubRunRow {
        seq,
        ts_ms,
        step_id: p.step_id,
        passed: p.passed,
        elapsed_ms: p.elapsed_ms,
    }
}

/// Produce a short UTF-8 summary of a raw msgpack payload without
/// round-tripping through `SsbPayload`. Non-UTF-8 bytes become `U+FFFD`.
fn summarize_payload(bytes: &[u8]) -> String {
    let text = String::from_utf8_lossy(bytes);
    text.chars()
        .filter(|c| !c.is_control())
        .take(SUMMARY_TRUNCATE_CHARS)
        .collect()
}

/// Validate `phase_id` for [`SsbViewer::step_progress`].
///
/// Accepts `[A-Za-z0-9._-]{1,64}`; rejects empty, dot-traversal, and
/// over-length input. Returns [`SsbViewerError::OutOfRange`] on failure so
/// callers can surface a single error variant.
fn validate_phase_id(phase_id: &str) -> Result<(), SsbViewerError> {
    if phase_id.is_empty() {
        return Err(SsbViewerError::Validation("phase_id empty".into()));
    }
    if phase_id.len() > PHASE_ID_MAX_LEN {
        return Err(SsbViewerError::Validation(format!(
            "phase_id length {} exceeds max {}",
            phase_id.len(),
            PHASE_ID_MAX_LEN
        )));
    }
    if phase_id == ".." || phase_id.contains("../") || phase_id.contains("..\\") {
        return Err(SsbViewerError::Validation(
            "phase_id path-traversal".into(),
        ));
    }
    let ok = phase_id
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || matches!(c, '.' | '_' | '-'));
    if !ok {
        return Err(SsbViewerError::Validation(format!(
            "phase_id contains invalid chars: {phase_id}"
        )));
    }
    Ok(())
}

/// True if `step_id` belongs to `phase_id` (exact match OR `phase_id.<rest>`).
fn step_matches(step_id: &str, phase_id: &str) -> bool {
    if step_id == phase_id {
        return true;
    }
    if step_id.len() > phase_id.len() {
        let bytes = step_id.as_bytes();
        let plen = phase_id.len();
        if &step_id[..plen] == phase_id && bytes[plen] == b'.' {
            return true;
        }
    }
    false
}

/// Validate that a `u64` request id fits in `i64` (the native seq width).
pub fn validate_request_id(request_id: u64) -> Result<i64, SsbViewerError> {
    if request_id > MAX_SEQ_I64_AS_U64 {
        return Err(SsbViewerError::OutOfRange(format!(
            "request_id {request_id} exceeds i64::MAX"
        )));
    }
    Ok(request_id as i64) // NARROWING-OK: guard above rejects request_id > i64::MAX; cast is safe post-check
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use elai_core::clock::SeededClock;
    use tempfile::TempDir;

    use super::*;
    use crate::ssb::{InterventionKeyword, Ssb, SsbConfig};

    fn test_config(dir: &TempDir) -> SsbConfig {
        SsbConfig {
            db_path: dir.path().join("test_ssb.db"),
            pool_size: Some(2),
            project_id: "proj-test".into(),
            session_id: "session-test".into(),
        }
    }

    fn test_clock() -> Arc<dyn elai_core::clock::ClockProvider> {
        Arc::new(SeededClock::new())
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn ssb_viewer_opens_read_only() {
        let dir = TempDir::new().expect("tempdir");
        let config = test_config(&dir);
        let db_path = config.db_path.clone();
        // Create the DB via the writer path so schema + pragmas are applied.
        let ssb = Ssb::new(config, test_clock()).expect("ssb init");
        drop(ssb);

        let viewer = SsbViewer::open(&db_path).expect("viewer open");
        let conn = viewer.conn.lock().expect("lock");
        // SQLite will reject the INSERT on a read-only connection.
        let err = conn
            .execute(
                "INSERT INTO ssb (project_id, session_id, agent_id, superstep, \
                 action_type, payload, created_at) VALUES \
                 (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
                params!["p", "s", "a", 0i64, "file_write", &[0u8][..], 0i64],
            )
            .unwrap_err();
        let msg = err.to_string();
        assert!(
            msg.contains("readonly") || msg.contains("read-only") || msg.contains("read only"),
            "expected readonly error, got: {msg}"
        );
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn intervention_round_trip() {
        let dir = TempDir::new().expect("tempdir");
        let config = test_config(&dir);
        let db_path = config.db_path.clone();
        let ssb = Ssb::new(config, test_clock()).expect("ssb init");

        let payload = SsbPayload::InterventionRequest(InterventionRequestPayload {
            step_id: "step-42".into(),
            intervention_type: InterventionType::ApproveChange,
            description: "approve?".into(),
            diff_summary: None,
            options: vec![InterventionOption::Keyword(InterventionKeyword::Accept)],
        });
        let seq = ssb
            .write_payload("agent-0", 0, payload, vec![])
            .await
            .expect("write intervention_request");
        assert!(seq > 0);
        drop(ssb);

        let viewer = SsbViewer::open(&db_path).expect("viewer open");
        let pending = viewer.pending_interventions().expect("pending");
        assert_eq!(pending.len(), 1, "one unanswered intervention");
        assert_eq!(pending[0].step_id, "step-42");
        assert_eq!(
            pending[0].intervention_type,
            InterventionType::ApproveChange
        );
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn recent_actions_summary_is_truncated() {
        let dir = TempDir::new().expect("tempdir");
        let config = test_config(&dir);
        let db_path = config.db_path.clone();
        let ssb = Ssb::new(config, test_clock()).expect("ssb init");

        let big_description = "x".repeat(5_000);
        let payload = SsbPayload::InterventionRequest(InterventionRequestPayload {
            step_id: "step-big".into(),
            intervention_type: InterventionType::ApproveChange,
            description: big_description,
            diff_summary: None,
            options: vec![InterventionOption::Keyword(InterventionKeyword::Accept)],
        });
        ssb.write_payload("agent-big", 0, payload, vec![])
            .await
            .expect("write big");
        drop(ssb);

        let viewer = SsbViewer::open(&db_path).expect("viewer open");
        let rows = viewer.recent_actions(10, 0).expect("recent");
        assert!(!rows.is_empty(), "at least one row");
        for row in &rows {
            assert!(
                row.summary.chars().count() <= SUMMARY_TRUNCATE_CHARS,
                "summary must be truncated, got {} chars",
                row.summary.chars().count()
            );
        }
    }

    #[test]
    fn validate_request_id_rejects_overflow() {
        assert!(validate_request_id(0).is_ok());
        assert!(validate_request_id(i64::MAX as u64).is_ok());
        assert!(validate_request_id((i64::MAX as u64) + 1).is_err());
        assert!(validate_request_id(u64::MAX).is_err());
    }
}
