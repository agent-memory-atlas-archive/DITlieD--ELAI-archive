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
    ssb_action_type_from_str_public, BabyStepReadyPayload, BubbleUpTypedPayload,
    ConceptDecompositionNodePayload, ContextBudgetComputedPayload, DagRepairEscalatedPayload,
    DagRepairRoundPayload, FeasibilityReportPayload, InterventionOption,
    InterventionRequestPayload, InterventionType, NovelStepDetectedPayload,
    ReflectAuditVerdictPayload, ReflectRefinedPayload, ReflectTriagedPayload, ReproPackKind,
    ReproPackPayload, RerunHashEqualityPayload, SsbActionType, SsbPayload, StubRunPayload,
    TaskSplitPayload, TaskSubmittedPayload, UsageTurnEventPayload,
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

/// B-17 Task 2: per-gate aggregated verdict counts derived from
/// `evidence_gate_evaluated` SSB rows. One row per distinct `gate_id`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GateVerdictCount {
    pub gate_id: String,
    pub pass_count: u64,
    pub block_count: u64,
    pub total: u64,
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

/// C-4 — aggregated cloud-spend snapshot returned by
/// [`SsbViewer::cloud_spend_summary`] and surfaced to the dashboard
/// `MonitorWindow` cloud gauges via the `get_cloud_spend` Tauri command.
///
/// All fields are zero when the SSB has no `cloud_spend` rows (fresh
/// install / no cloud invocations yet) so the frontend never sees
/// `undefined`. `redaction_count` is currently always 0 because no SSB
/// action type emits a redaction counter — wired in for a future
/// `redaction_event` row type (documented in C-4-decisions.md).
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct CloudSpendSummary {
    /// Most recent CloudSpend row's `cost_usd`. Approximates the cost of
    /// the active task.
    pub task_usd: f64,
    /// Sum of `cost_usd` across every CloudSpend row whose `created_at`
    /// falls inside the current local-day window.
    pub day_usd: f64,
    /// C-5: sum of `cost_usd` across every CloudSpend row whose
    /// `created_at` falls inside the current 30-day rolling window
    /// (approximation of monthly spend; calendar-month boundaries cost
    /// an extra SQL pass that buys little on the cap-enforcement path).
    /// Used by [`crate::cloud_mode::CloudModeRouter::route_with_spend`]
    /// to enforce `spend_cap_monthly_usd`.
    pub month_usd: f64,
    /// Sum of `cache_creation_tokens` over today's rows.
    pub cache_creation_tokens: u64,
    /// Sum of `cache_read_tokens` over today's rows.
    pub cache_read_tokens: u64,
    /// Number of redaction events. Always 0 today (no source SSB row);
    /// kept on the wire so a future writer side can populate without
    /// breaking the frontend.
    pub redaction_count: u64,
    /// I-10: most-recent TTL tier label selected by the cache-economics planner
    /// ("1h" | "5m" | "default"). `None` when no CacheEconomicsEvent row exists.
    #[serde(default)]
    pub ttl_tier: Option<String>,
    /// I-10: breakeven read count for the most-recent TTL tier (3 for 1h, 1 for 5m, 0 otherwise).
    #[serde(default)]
    pub breakeven_reads: u32,
    /// I-10: number of HitRateGuardTripped events emitted today.
    #[serde(default)]
    pub guard_trips_today: u32,
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
    pub failed_files: u64,
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

/// B-39: snapshot of the latest REFLECT pipeline state for a session.
///
/// Returned by [`SsbViewer::get_reflect_state`]. Each `last_*` field holds the
/// payload of the most recent SSB row of that type (by max seq); `None` when
/// no such row exists yet.
///
/// `enabled` is `true` when any `reflect_*` row exists in the SSB, `false`
/// otherwise. Future work: read from a project config table.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReflectState {
    pub enabled: bool,
    pub last_triage: Option<ReflectTriagedPayload>,
    pub last_verdict: Option<ReflectAuditVerdictPayload>,
    pub last_summary: Option<ReflectRefinedPayload>,
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

    /// Return the most recent SSB row with the given `action_type`, or `None`
    /// if no such row exists. Decodes the msgpack payload and re-encodes as
    /// JSON for frontend consumption, matching [`get_action_detail`] semantics.
    ///
    /// Used by the dashboard `get_language_grammar_status` command (Pass D).
    pub fn latest_action_of_type(
        &self,
        action_type: &str,
    ) -> Result<Option<SsbActionDetail>, SsbViewerError> {
        let conn = self.lock()?;
        let row = conn
            .query_row(
                "SELECT seq, created_at, action_type, agent_id, payload \
                 FROM ssb WHERE action_type = ?1 ORDER BY seq DESC LIMIT 1",
                params![action_type],
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

        let Some((seq, ts, at, agent_id, payload_bytes)) = row else {
            return Ok(None);
        };

        let typed: SsbPayload = rmp_serde::from_slice(&payload_bytes)?;
        let payload_json = serde_json::to_value(&typed)
            .map_err(|e| SsbViewerError::Validation(format!("payload json encode: {e}")))?;

        Ok(Some(SsbActionDetail {
            seq,
            ts,
            action_type: at,
            agent_id,
            payload_json,
            files: Vec::new(),
        }))
    }

    /// C-1: read the N most recent rows for a single `action_type`.
    /// Returns full `SsbActionDetail` records with deserialised payload
    /// JSON so dashboard commands (`get_query_trace_recent`,
    /// `get_grounding_sweep`, `get_csq_pending`) can reshape the rows
    /// without writing per-type viewer methods. `limit` is hard-capped
    /// at [`DRAWER_QUERY_HARD_CAP`] to bound the result size.
    ///
    /// Rows whose msgpack payload fails to deserialise are SKIPPED with
    /// a `tracing::warn` — matching the existing `gg_signals` /
    /// `failure_taxonomy_last` reader semantics. The dashboard always
    /// sees a well-formed array.
    pub fn recent_rows_of_type(
        &self,
        action_type: &str,
        limit: usize,
    ) -> Result<Vec<SsbActionDetail>, SsbViewerError> {
        let effective = limit.min(DRAWER_QUERY_HARD_CAP);
        let limit_i64 = i64::try_from(effective).unwrap_or(i64::MAX);
        let conn = self.lock()?;
        let mut stmt = conn.prepare(
            "SELECT seq, created_at, action_type, agent_id, payload \
             FROM ssb WHERE action_type = ?1 ORDER BY seq DESC LIMIT ?2",
        )?;
        let rows = stmt.query_map(params![action_type, limit_i64], |row| {
            Ok((
                row.get::<_, i64>(0)?,
                row.get::<_, i64>(1)?,
                row.get::<_, String>(2)?,
                row.get::<_, String>(3)?,
                row.get::<_, Vec<u8>>(4)?,
            ))
        })?;
        let mut out: Vec<SsbActionDetail> = Vec::new();
        for row in rows {
            let (seq, ts, at, agent_id, payload_bytes) = row?;
            let typed: SsbPayload = match rmp_serde::from_slice(&payload_bytes) {
                Ok(p) => p,
                Err(e) => {
                    tracing::warn!(
                        seq,
                        action_type = %at,
                        error = %e,
                        "ssb_viewer::recent_rows_of_type: payload decode failed; skipping"
                    );
                    continue;
                }
            };
            let payload_json = match serde_json::to_value(&typed) {
                Ok(v) => v,
                Err(e) => {
                    tracing::warn!(
                        seq,
                        action_type = %at,
                        error = %e,
                        "ssb_viewer::recent_rows_of_type: json encode failed; skipping"
                    );
                    continue;
                }
            };
            out.push(SsbActionDetail {
                seq,
                ts,
                action_type: at,
                agent_id,
                payload_json,
                files: Vec::new(),
            });
        }
        Ok(out)
    }

    /// J-28 T5/T8 -- read back `UsageTurnEvent` rows for `elai usage`'s
    /// grouped rollup (`crate::usage_ledger::rollup_usage_turns`). Reuses
    /// [`Self::recent_rows_of_type`] (GD-1: no duplicate SQL query) and
    /// re-types each row's JSON payload back into
    /// [`UsageTurnEventPayload`]; a row that fails to re-type is skipped
    /// and warn-logged (the underlying decode failure is already
    /// warn-logged by `recent_rows_of_type`).
    pub fn usage_turn_events(
        &self,
        limit: usize,
    ) -> Result<Vec<UsageTurnEventPayload>, SsbViewerError> {
        let rows = self.recent_rows_of_type(SsbActionType::UsageTurnEvent.as_sql_str(), limit)?;
        let mut out = Vec::with_capacity(rows.len());
        for row in rows {
            match serde_json::from_value::<UsageTurnEventPayload>(row.payload_json) {
                Ok(payload) => out.push(payload),
                Err(e) => {
                    tracing::warn!(
                        seq = row.seq,
                        error = %e,
                        "ssb_viewer::usage_turn_events: payload re-type failed; skipping"
                    );
                }
            }
        }
        Ok(out)
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
    /// C-4 update: C-2 ships `BabyStepReady` SSB rows + the
    /// `ConceptDecompositionNode` tree, so this method now scans both to
    /// produce a partially-populated [`BabyStepDetail`].
    ///
    /// Lookup strategy (in order):
    ///   1. Scan recent `BabyStepReady` rows and match `baby_step_id` against
    ///      `payload.baby_step_id.to_string()`.
    ///   2. When a match is found, walk the decomposition tree to recover
    ///      `concept_name` / `phase_name` / `step_name` from the ancestry
    ///      ids (concept_id / phase_id / step_id).
    ///   3. `task` is populated from `payload.label`.
    ///   4. `tdd` / `acceptance` / `estimated_tokens` remain `None` / 0 —
    ///      no SSB row carries those fields today (BabyStepSpec persistence
    ///      is a documented C-5+ deferral).
    ///
    /// Returns [`SsbViewerError::NotFound`] when no matching BabyStepReady
    /// row exists. The dashboard renders that as "BabyStepNotPersisted".
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

        // Scan recent BabyStepReady rows (descending seq) so the latest
        // emission for an id wins. Cap at BABY_STEP_DETAIL_SCAN_LIMIT —
        // a project with >2000 unique baby-steps is implausible.
        const BABY_STEP_DETAIL_SCAN_LIMIT: i64 = 2000;
        let mut matched: Option<BabyStepReadyPayload> = None;
        {
            let conn = self.lock()?;
            let mut stmt = conn.prepare(
                "SELECT payload FROM ssb \
                 WHERE action_type = ?1 ORDER BY seq DESC LIMIT ?2",
            )?;
            let rows = stmt.query_map(
                params![
                    SsbActionType::BabyStepReady.as_sql_str(),
                    BABY_STEP_DETAIL_SCAN_LIMIT,
                ],
                |row| row.get::<_, Vec<u8>>(0),
            )?;
            for row in rows {
                let bytes = row?;
                let payload: SsbPayload = match rmp_serde::from_slice(&bytes) {
                    Ok(p) => p,
                    Err(_) => continue,
                };
                if let SsbPayload::BabyStepReady(p) = payload {
                    if p.baby_step_id.to_string() == baby_step_id {
                        matched = Some(p);
                        break;
                    }
                }
            }
        }

        let Some(payload) = matched else {
            return Err(SsbViewerError::NotFound {
                kind: "baby_step".to_owned(),
                id: baby_step_id.to_owned(),
            });
        };

        // Walk the decomposition tree to derive concept / phase / step
        // names. Each lookup is O(N) over a small tree (typical project
        // <100 nodes); we trade clarity for an extra pass.
        let tree = self.decomposition_tree(None).unwrap_or_default();
        let mut concept_name = String::new();
        let mut phase_name = String::new();
        let mut step_name = String::new();
        let concept_id_s = payload.concept_id.to_string();
        let phase_id_s = payload.phase_id.to_string();
        let step_id_s = payload.step_id.to_string();
        for c in &tree.concepts {
            if c.id == concept_id_s {
                concept_name = c.title.clone();
            }
            for ph in &c.phases {
                if ph.id == phase_id_s {
                    phase_name = ph.title.clone();
                    if concept_name.is_empty() {
                        concept_name = c.title.clone();
                    }
                }
                for st in &ph.steps {
                    if st.id == step_id_s {
                        step_name = st.title.clone();
                        if phase_name.is_empty() {
                            phase_name = ph.title.clone();
                        }
                        if concept_name.is_empty() {
                            concept_name = c.title.clone();
                        }
                    }
                }
            }
        }

        Ok(BabyStepDetail {
            baby_step_id: baby_step_id.to_owned(),
            task: payload.label,
            estimated_tokens: 0,
            concept_name,
            phase_name,
            step_name,
            tdd: None,
            acceptance: None,
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
                failed_files: d.failed_files,
                languages: d.languages,
                ts_ms,
                known: true,
            }),
            None => Ok(DkgStatsSnapshot {
                project_id: String::new(),
                node_count: 0,
                edge_count: 0,
                failed_files: 0,
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

    /// PageRank top-N (Tier 3 Knowledge drawer).
    ///
    /// C-3 (DRIFT-1): no SSB row type persists per-symbol PageRank scores
    /// and the codebase does not currently materialise a `dkg_nodes_pagerank`
    /// table — PageRank in elai_cc is recomputed in-memory per query by
    /// `RetrievalPipeline::compute_repo_map` (knowledge/src/pipeline.rs).
    /// To future-proof the viewer for the day a persistent table lands,
    /// we probe `sqlite_master` for the table name and only run the
    /// canonical SELECT when it exists. When the table is absent we
    /// return `Ok(Vec::new())` so the dashboard renders "no data" without
    /// surfacing an error — this matches the plan acceptance rule
    /// "returns empty array (not error) when table is empty".
    pub fn pagerank_top(
        &self,
        n: usize,
    ) -> Result<Vec<PageRankEntry>, SsbViewerError> {
        let limit_i64 = i64::try_from(n).unwrap_or(i64::MAX);
        let conn = self.lock()?;
        let table_exists: bool = conn
            .query_row(
                "SELECT 1 FROM sqlite_master WHERE type = 'table' AND name = 'dkg_nodes_pagerank'",
                [],
                |_| Ok(true),
            )
            .or_else(|e| match e {
                rusqlite::Error::QueryReturnedNoRows => Ok(false),
                other => Err(other),
            })?;
        if !table_exists {
            return Ok(Vec::new());
        }
        let mut stmt = conn.prepare(
            "SELECT symbol, path, score FROM dkg_nodes_pagerank \
             ORDER BY score DESC LIMIT ?1",
        )?;
        let rows = stmt.query_map(params![limit_i64], |row| {
            Ok(PageRankEntry {
                symbol: row.get::<_, String>(0)?,
                path: row.get::<_, String>(1)?,
                score: row.get::<_, f64>(2)?,
            })
        })?;
        let mut out: Vec<PageRankEntry> = Vec::new();
        for row in rows {
            match row {
                Ok(entry) => out.push(entry),
                Err(e) => {
                    tracing::warn!(error = %e, "pagerank_top: row decode failed; skipping");
                }
            }
        }
        Ok(out)
    }

    /// C-3 (DRIFT-6): thin alias over [`recent_rows_of_type`] for
    /// `query_trace` rows so the deterministic check
    /// `rg "fn recent_query_traces"` resolves. The implementation just
    /// forwards to the canonical reader; no extra DB work.
    pub fn recent_query_traces(
        &self,
        limit: usize,
    ) -> Result<Vec<SsbActionDetail>, SsbViewerError> {
        self.recent_rows_of_type(SsbActionType::QueryTrace.as_sql_str(), limit)
    }

    /// Query-tier routing histogram (Tier 3 Knowledge drawer).
    ///
    /// C-3: real aggregate over the most recent `query_trace` SSB rows
    /// (capped at [`DRAWER_QUERY_HARD_CAP`]). Each row carries a
    /// [`crate::ssb::QueryTracePayload`] with a `tier`
    /// (`"FAST" / "STANDARD" / "DEEP" / "AUTO"`) and `total_latency_ms`.
    /// Rows are bucketed by tier; `*_pct` is the share of the sample in
    /// each bucket and `avg_latency_ms_*` is the mean latency within the
    /// bucket. `AUTO` rows count toward `sample_size` and the percentage
    /// denominator but are not surfaced as their own bucket (the histogram
    /// has three named lanes). When no `query_trace` rows exist yet the
    /// method returns an all-zero `QueryTierStats` (sample_size = 0), so the
    /// dashboard renders "no data" without erroring.
    pub fn query_tier_stats(&self) -> Result<QueryTierStats, SsbViewerError> {
        let conn = self.lock()?;
        let mut stmt = conn.prepare(
            "SELECT payload FROM ssb \
             WHERE action_type = ?1 ORDER BY seq DESC LIMIT ?2",
        )?;
        let rows = stmt.query_map(
            params![
                SsbActionType::QueryTrace.as_sql_str(),
                DRAWER_QUERY_HARD_CAP as i64,
            ],
            |row| row.get::<_, Vec<u8>>(0),
        )?;

        let mut fast_n: u32 = 0;
        let mut standard_n: u32 = 0;
        let mut deep_n: u32 = 0;
        let mut fast_lat_sum: u64 = 0;
        let mut standard_lat_sum: u64 = 0;
        let mut deep_lat_sum: u64 = 0;
        let mut sample_size: u32 = 0;

        for row in rows {
            let bytes = row?;
            let Ok(payload): Result<SsbPayload, _> = rmp_serde::from_slice(&bytes) else {
                continue;
            };
            let SsbPayload::QueryTrace(p) = payload else {
                continue;
            };
            sample_size = sample_size.saturating_add(1);
            match p.tier.as_str() {
                "FAST" => {
                    fast_n = fast_n.saturating_add(1);
                    fast_lat_sum = fast_lat_sum.saturating_add(p.total_latency_ms);
                }
                "STANDARD" => {
                    standard_n = standard_n.saturating_add(1);
                    standard_lat_sum =
                        standard_lat_sum.saturating_add(p.total_latency_ms);
                }
                "DEEP" => {
                    deep_n = deep_n.saturating_add(1);
                    deep_lat_sum = deep_lat_sum.saturating_add(p.total_latency_ms);
                }
                // "AUTO" (or any future tier label) counts toward the
                // sample denominator but has no dedicated histogram lane.
                _ => {}
            }
        }

        let pct = |n: u32| -> f32 {
            if sample_size == 0 {
                0.0
            } else {
                (n as f32 / sample_size as f32) * 100.0
            }
        };
        let avg = |sum: u64, n: u32| -> f32 {
            if n == 0 {
                0.0
            } else {
                sum as f32 / n as f32
            }
        };

        Ok(QueryTierStats {
            fast_pct: pct(fast_n),
            standard_pct: pct(standard_n),
            deep_pct: pct(deep_n),
            avg_latency_ms_fast: avg(fast_lat_sum, fast_n),
            avg_latency_ms_standard: avg(standard_lat_sum, standard_n),
            avg_latency_ms_deep: avg(deep_lat_sum, deep_n),
            sample_size,
        })
    }

    /// Freshness state (Tier 3 Knowledge drawer).
    ///
    /// WIRE-DARK[C-3]: there is no `freshness_computed` SSB action_type and
    /// the freshness meta DB (`crates/knowledge/src/freshness.rs`) does not
    /// expose a read endpoint reachable from the orchestrator SSB. Until a
    /// freshness producer row lands, this honestly reports `known: false`
    /// with zeroed config rather than fabricating decay/boost numbers (the
    /// dashboard treats `known: false` as "no data yet"). Real SQL read is
    /// blocked on the producer side, owned by a future knowledge-crate plan.
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
    /// branch-merge SSB rows — each row describes a handoff boundary
    /// where the winner name survived and the abandoned names did not.
    ///
    /// C-23: reads BOTH the legacy `BranchMerge` rows (no producer in
    /// the current codebase but kept for forward-compat with externally
    /// authored rows) AND the new `BranchMerged` rows emitted by
    /// `merge_branch_with_ssb` after the SCHEMA_VERSION 55 bump.
    pub fn session_handoffs(
        &self,
        limit: usize,
    ) -> Result<Vec<SessionHandoffEntry>, SsbViewerError> {
        let effective = limit.min(DRAWER_QUERY_HARD_CAP);
        let limit_i64 = i64::try_from(effective).unwrap_or(i64::MAX);
        let conn = self.lock()?;
        let mut stmt = conn.prepare(
            "SELECT seq, created_at, payload FROM ssb \
             WHERE action_type IN (?1, ?2) ORDER BY seq DESC LIMIT ?3",
        )?;
        let rows = stmt.query_map(
            params![
                SsbActionType::BranchMerge.as_sql_str(),
                SsbActionType::BranchMerged.as_sql_str(),
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
        let mut out: Vec<SessionHandoffEntry> = Vec::new();
        for row in rows {
            let (seq, ts_ms, bytes) = row?;
            let payload: SsbPayload = match rmp_serde::from_slice(&bytes) {
                Ok(p) => p,
                Err(_) => continue,
            };
            match payload {
                SsbPayload::BranchMerge(b) => {
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
                SsbPayload::BranchMerged(b) => {
                    // C-23 BranchMerged payload has no from_snap_ids field;
                    // expose 0 (callers use this only for activity-feed
                    // labelling, not for chain reconstruction).
                    out.push(SessionHandoffEntry {
                        seq,
                        ts_ms,
                        winner: b.winner_name,
                        abandoned: b.abandoned_names,
                        trunk_snap_id: b.trunk_snap_id,
                        from_snap_count: 0,
                    });
                }
                _ => {}
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

    /// B-17 Task 2: aggregates `evidence_gate_evaluated` rows (capped at
    /// [`DRAWER_QUERY_HARD_CAP`]) into per-gate verdict counts. Returns one
    /// [`GateVerdictCount`] per distinct `gate_id` encountered in the SSB,
    /// grouped by `gate_id` with `pass_count`, `block_count`, and `total`.
    /// Gates with no rows are NOT included — callers render only the gates
    /// that have actually fired. Empty SSB returns an empty Vec.
    pub fn aggregate_gate_verdict_counts(
        &self,
    ) -> Result<Vec<GateVerdictCount>, SsbViewerError> {
        let rows = self.recent_evidence_gates(DRAWER_QUERY_HARD_CAP)?;
        let mut counts: std::collections::HashMap<String, GateVerdictCount> =
            std::collections::HashMap::new();
        for row in rows {
            let entry = counts.entry(row.gate_id.clone()).or_insert_with(|| {
                GateVerdictCount {
                    gate_id: row.gate_id.clone(),
                    pass_count: 0,
                    block_count: 0,
                    total: 0,
                }
            });
            entry.total += 1;
            if row.verdict == "pass" {
                entry.pass_count += 1;
            } else if row.verdict == "block" {
                entry.block_count += 1;
            }
        }
        let mut out: Vec<GateVerdictCount> = counts.into_values().collect();
        out.sort_by(|a, b| b.total.cmp(&a.total).then_with(|| a.gate_id.cmp(&b.gate_id)));
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

    // -----------------------------------------------------------------------
    // B-7 (SCHEMA_VERSION 100): producer-wire viewer readers
    // -----------------------------------------------------------------------

    /// B-7 (SCHEMA_VERSION 100): returns up to `limit` recent
    /// `failure_recorded` rows in descending seq order. Used by the dashboard
    /// PlanWindow detail pane to render per-baby-step retry history.
    pub fn recent_failure_records(
        &self,
        limit: usize,
    ) -> Result<Vec<crate::ssb::FailureRecordedPayload>, SsbViewerError> {
        let capped = limit.min(DRAWER_QUERY_HARD_CAP);
        let limit_i64 =
            i64::try_from(capped).map_err(|e| SsbViewerError::OutOfRange(e.to_string()))?;
        let conn = self.lock()?;
        let mut stmt = conn.prepare(
            "SELECT payload FROM ssb \
             WHERE action_type = ?1 ORDER BY seq DESC LIMIT ?2",
        )?;
        let rows = stmt.query_map(
            params![SsbActionType::FailureRecorded.as_sql_str(), limit_i64],
            |row| row.get::<_, Vec<u8>>(0),
        )?;
        let mut out = Vec::new();
        for row in rows {
            let bytes = row?;
            let payload: SsbPayload = rmp_serde::from_slice(&bytes)?;
            if let SsbPayload::FailureRecorded(p) = payload {
                out.push(p);
            }
        }
        Ok(out)
    }

    /// B-7 (SCHEMA_VERSION 100): returns up to `limit` recent
    /// `tree_position_changed` rows in descending seq order. Used by the
    /// dashboard PlanWindow to highlight cursor drift.
    pub fn recent_tree_positions(
        &self,
        limit: usize,
    ) -> Result<Vec<crate::ssb::TreePositionChangedPayload>, SsbViewerError> {
        let capped = limit.min(DRAWER_QUERY_HARD_CAP);
        let limit_i64 =
            i64::try_from(capped).map_err(|e| SsbViewerError::OutOfRange(e.to_string()))?;
        let conn = self.lock()?;
        let mut stmt = conn.prepare(
            "SELECT payload FROM ssb \
             WHERE action_type = ?1 ORDER BY seq DESC LIMIT ?2",
        )?;
        let rows = stmt.query_map(
            params![SsbActionType::TreePositionChanged.as_sql_str(), limit_i64],
            |row| row.get::<_, Vec<u8>>(0),
        )?;
        let mut out = Vec::new();
        for row in rows {
            let bytes = row?;
            let payload: SsbPayload = rmp_serde::from_slice(&bytes)?;
            if let SsbPayload::TreePositionChanged(p) = payload {
                out.push(p);
            }
        }
        Ok(out)
    }

    /// B-7 (SCHEMA_VERSION 100): returns up to `limit` recent
    /// `package_ref_verified` rows in descending seq order. Used by the
    /// dashboard PlanWindow + new `get_package_verifications` Tauri command.
    pub fn recent_package_verifications(
        &self,
        limit: usize,
    ) -> Result<Vec<crate::ssb::PackageRefVerifiedPayload>, SsbViewerError> {
        let capped = limit.min(DRAWER_QUERY_HARD_CAP);
        let limit_i64 =
            i64::try_from(capped).map_err(|e| SsbViewerError::OutOfRange(e.to_string()))?;
        let conn = self.lock()?;
        let mut stmt = conn.prepare(
            "SELECT payload FROM ssb \
             WHERE action_type = ?1 ORDER BY seq DESC LIMIT ?2",
        )?;
        let rows = stmt.query_map(
            params![SsbActionType::PackageRefVerified.as_sql_str(), limit_i64],
            |row| row.get::<_, Vec<u8>>(0),
        )?;
        let mut out = Vec::new();
        for row in rows {
            let bytes = row?;
            let payload: SsbPayload = rmp_serde::from_slice(&bytes)?;
            if let SsbPayload::PackageRefVerified(p) = payload {
                out.push(p);
            }
        }
        Ok(out)
    }

    /// B-7 (SCHEMA_VERSION 100): returns up to `limit` recent
    /// `tech_stack_decided` rows in descending seq order. Used by the
    /// dashboard PlanWindow tech-stack detail pane.
    pub fn recent_tech_stack_decisions(
        &self,
        limit: usize,
    ) -> Result<Vec<crate::ssb::TechStackDecidedPayload>, SsbViewerError> {
        let capped = limit.min(DRAWER_QUERY_HARD_CAP);
        let limit_i64 =
            i64::try_from(capped).map_err(|e| SsbViewerError::OutOfRange(e.to_string()))?;
        let conn = self.lock()?;
        let mut stmt = conn.prepare(
            "SELECT payload FROM ssb \
             WHERE action_type = ?1 ORDER BY seq DESC LIMIT ?2",
        )?;
        let rows = stmt.query_map(
            params![SsbActionType::TechStackDecided.as_sql_str(), limit_i64],
            |row| row.get::<_, Vec<u8>>(0),
        )?;
        let mut out = Vec::new();
        for row in rows {
            let bytes = row?;
            let payload: SsbPayload = rmp_serde::from_slice(&bytes)?;
            if let SsbPayload::TechStackDecided(p) = payload {
                out.push(p);
            }
        }
        Ok(out)
    }

    /// B-7 (SCHEMA_VERSION 100): returns up to `limit` recent
    /// `web_research_step` rows for the given `node_id`, filtered to ONLY
    /// stage-populated rows (fine-grained per-stage events; coarse rows with
    /// `stage = ""` are dropped). Routed via the existing
    /// `SsbActionType::WebResearchStep` channel — no new SsbActionType
    /// variant is fabricated for per-stage history.
    ///
    /// Used by the dashboard ResearchDrawer stage timeline (rewires the
    /// previous C-4 DRIFT-6 deferral that returned an empty Vec).
    pub fn research_node_stages(
        &self,
        node_id: u64,
        limit: usize,
    ) -> Result<Vec<crate::ssb::WebResearchStepPayload>, SsbViewerError> {
        let capped = limit.min(DRAWER_QUERY_HARD_CAP);
        // Over-fetch to compensate for the post-decode node_id + stage filter.
        // Hard cap on the SQL side stays bounded by DRAWER_QUERY_HARD_CAP so
        // a runaway request never reads the entire ssb table.
        let scan_cap = capped.saturating_mul(4).min(DRAWER_QUERY_HARD_CAP);
        let scan_i64 =
            i64::try_from(scan_cap).map_err(|e| SsbViewerError::OutOfRange(e.to_string()))?;
        let conn = self.lock()?;
        let mut stmt = conn.prepare(
            "SELECT payload FROM ssb \
             WHERE action_type = ?1 ORDER BY seq DESC LIMIT ?2",
        )?;
        let rows = stmt.query_map(
            params![SsbActionType::WebResearchStep.as_sql_str(), scan_i64],
            |row| row.get::<_, Vec<u8>>(0),
        )?;
        let mut out = Vec::new();
        for row in rows {
            if out.len() >= capped {
                break;
            }
            let bytes = row?;
            let payload: SsbPayload = rmp_serde::from_slice(&bytes)?;
            if let SsbPayload::WebResearchStep(p) = payload {
                if p.node_id == node_id && !p.stage.is_empty() {
                    out.push(p);
                }
            }
        }
        Ok(out)
    }

    /// B-4 / cross-check 2026-05-30: returns up to `limit` most-recent
    /// [`crate::ssb::AgentLifecyclePayload`] rows (newest-first, seq DESC).
    /// Reads the canonical `agent_lifecycle` SSB action_type written by the
    /// BSP engine at spawn / ready / exit transitions (bsp/engine.rs:243,
    /// 278, 344, 393, 563, 648, 681). Sibling of
    /// [`Self::recent_agent_status_changes`] which reads the C-2 mirror
    /// variant; this helper exposes the unfiltered lifecycle stream for
    /// callers that need the full spawn/exit chain (e.g. supervision
    /// timeline, crash forensics) rather than the latest-per-agent dedup.
    /// Honors the plan B-4 producer-wiring name; the dashboard `get_agents`
    /// command can compose either reader depending on whether dedup is
    /// desired.
    pub fn recent_agent_lifecycle_rows(
        &self,
        limit: usize,
    ) -> Result<Vec<crate::ssb::AgentLifecyclePayload>, SsbViewerError> {
        let capped = limit.min(DRAWER_QUERY_HARD_CAP);
        let limit_i64 =
            i64::try_from(capped).map_err(|e| SsbViewerError::OutOfRange(e.to_string()))?;
        let conn = self.lock()?;
        let mut stmt = conn.prepare(
            "SELECT payload FROM ssb \
             WHERE action_type = ?1 ORDER BY seq DESC LIMIT ?2",
        )?;
        let rows = stmt.query_map(
            params![SsbActionType::AgentLifecycle.as_sql_str(), limit_i64],
            |row| row.get::<_, Vec<u8>>(0),
        )?;
        let mut out = Vec::new();
        for row in rows {
            let bytes = row?;
            let payload: SsbPayload = rmp_serde::from_slice(&bytes)?;
            if let SsbPayload::AgentLifecycle(p) = payload {
                out.push(p);
            }
        }
        Ok(out)
    }

    /// C-2: returns the latest [`crate::ssb::AgentStatusChangedPayload`] per
    /// `agent_id` in descending seq order. Used by the dashboard `get_agents`
    /// Tauri command to replace the hardcoded 8-role IDLE list. When the SSB
    /// has zero rows yet, returns an empty Vec; the caller falls back to the
    /// canonical IDLE list so the AGENTS section never empty-renders.
    pub fn recent_agent_status_changes(
        &self,
        limit: usize,
    ) -> Result<Vec<crate::ssb::AgentStatusChangedPayload>, SsbViewerError> {
        let capped = limit.min(DRAWER_QUERY_HARD_CAP);
        let limit_i64 =
            i64::try_from(capped).map_err(|e| SsbViewerError::OutOfRange(e.to_string()))?;
        let conn = self.lock()?;
        let mut stmt = conn.prepare(
            "SELECT payload FROM ssb \
             WHERE action_type = ?1 ORDER BY seq DESC LIMIT ?2",
        )?;
        let rows = stmt.query_map(
            params![SsbActionType::AgentStatusChanged.as_sql_str(), limit_i64],
            |row| row.get::<_, Vec<u8>>(0),
        )?;
        // Walk the descending stream and keep the FIRST payload per agent_id
        // (which is the latest because of DESC ordering). Preserves order
        // across roles for stable dashboard rendering.
        use std::collections::HashSet;
        let mut seen: HashSet<String> = HashSet::new();
        let mut out = Vec::new();
        for row in rows {
            let bytes = row?;
            let payload: SsbPayload = rmp_serde::from_slice(&bytes)?;
            if let SsbPayload::AgentStatusChanged(p) = payload {
                if seen.insert(p.agent_id.clone()) {
                    out.push(p);
                }
            }
        }
        Ok(out)
    }

    /// B-6: returns the latest [`crate::ssb::AntiSpiralHitPayload`] or `None`
    /// when no anti-spiral hit has been recorded yet. Used by the dashboard
    /// `get_anti_spiral_counter` Tauri command to surface the live counter
    /// instead of the hardcoded zero stub.
    pub fn latest_anti_spiral_hit(
        &self,
    ) -> Result<Option<crate::ssb::AntiSpiralHitPayload>, SsbViewerError> {
        let conn = self.lock()?;
        let mut stmt = conn.prepare(
            "SELECT payload FROM ssb \
             WHERE action_type = ?1 ORDER BY seq DESC LIMIT 1",
        )?;
        let mut rows = stmt.query_map(
            params![SsbActionType::AntiSpiralHit.as_sql_str()],
            |row| row.get::<_, Vec<u8>>(0),
        )?;
        if let Some(row) = rows.next() {
            let bytes = row?;
            let payload: SsbPayload = rmp_serde::from_slice(&bytes)?;
            if let SsbPayload::AntiSpiralHit(p) = payload {
                return Ok(Some(p));
            }
            tracing::warn!(
                "latest_anti_spiral_hit: payload variant mismatch (schema drift / corrupted row); returning None"
            );
        }
        Ok(None)
    }

    /// C-2: returns up to `limit` recent ambiguity rows. Used by the
    /// dashboard `get_ambiguity_state` Tauri command. Returns the latest
    /// [`crate::ssb::AmbiguityScoredPayload`] plus optional last
    /// [`crate::ssb::AmbiguityFloorFailedPayload`] / streak rows so the
    /// drawer can render score + breakdown + floor warnings + streak.
    pub fn latest_ambiguity_scored(
        &self,
    ) -> Result<Option<crate::ssb::AmbiguityScoredPayload>, SsbViewerError> {
        let conn = self.lock()?;
        let mut stmt = conn.prepare(
            "SELECT payload FROM ssb \
             WHERE action_type = ?1 ORDER BY seq DESC LIMIT 1",
        )?;
        let mut rows = stmt.query_map(
            params![SsbActionType::AmbiguityScored.as_sql_str()],
            |row| row.get::<_, Vec<u8>>(0),
        )?;
        if let Some(row) = rows.next() {
            let bytes = row?;
            let payload: SsbPayload = rmp_serde::from_slice(&bytes)?;
            if let SsbPayload::AmbiguityScored(p) = payload {
                return Ok(Some(p));
            }
        }
        Ok(None)
    }

    /// C-2: returns the latest [`crate::ssb::AmbiguityFloorFailedPayload`] or
    /// `None` when no floor-fail row has been written yet.
    pub fn latest_ambiguity_floor_failed(
        &self,
    ) -> Result<Option<crate::ssb::AmbiguityFloorFailedPayload>, SsbViewerError> {
        let conn = self.lock()?;
        let mut stmt = conn.prepare(
            "SELECT payload FROM ssb \
             WHERE action_type = ?1 ORDER BY seq DESC LIMIT 1",
        )?;
        let mut rows = stmt.query_map(
            params![SsbActionType::AmbiguityFloorFailed.as_sql_str()],
            |row| row.get::<_, Vec<u8>>(0),
        )?;
        if let Some(row) = rows.next() {
            let bytes = row?;
            let payload: SsbPayload = rmp_serde::from_slice(&bytes)?;
            if let SsbPayload::AmbiguityFloorFailed(p) = payload {
                return Ok(Some(p));
            }
        }
        Ok(None)
    }

    /// C-2: returns the latest [`crate::ssb::AmbiguityStreakAdvancedPayload`]
    /// or `None` when no streak row has been written yet.
    pub fn latest_ambiguity_streak(
        &self,
    ) -> Result<Option<crate::ssb::AmbiguityStreakAdvancedPayload>, SsbViewerError> {
        let conn = self.lock()?;
        let mut stmt = conn.prepare(
            "SELECT payload FROM ssb \
             WHERE action_type = ?1 ORDER BY seq DESC LIMIT 1",
        )?;
        let mut rows = stmt.query_map(
            params![SsbActionType::AmbiguityStreakAdvanced.as_sql_str()],
            |row| row.get::<_, Vec<u8>>(0),
        )?;
        if let Some(row) = rows.next() {
            let bytes = row?;
            let payload: SsbPayload = rmp_serde::from_slice(&bytes)?;
            if let SsbPayload::AmbiguityStreakAdvanced(p) = payload {
                return Ok(Some(p));
            }
        }
        Ok(None)
    }

    /// B-38 / cross-check 2026-05-30 alias: returns the latest
    /// [`crate::ssb::AmbiguityStreakAdvancedPayload`] under the
    /// fully-qualified `latest_ambiguity_streak_advanced` name to mirror the
    /// `ambiguity_streak_advanced` SSB action_type label (and the sibling
    /// `latest_ambiguity_scored` / `latest_ambiguity_floor_failed` pair).
    /// Thin wrapper over [`Self::latest_ambiguity_streak`] — no extra
    /// query cost. Added to close B-38 viewer-helper coverage gap noted in
    /// workflow/sessions/s-night-2026-05-30/cross-check/B/B-13_b36_b37_b38.md.
    pub fn latest_ambiguity_streak_advanced(
        &self,
    ) -> Result<Option<crate::ssb::AmbiguityStreakAdvancedPayload>, SsbViewerError> {
        self.latest_ambiguity_streak()
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

    /// Returns up to `limit` recent `dag_repair_escalated` rows in descending
    /// seq order (most recent first). Used by the dashboard DAG_REPAIR drawer
    /// to surface escalation events when T_DAG=10 is exhausted.
    pub fn recent_dag_repair_escalated(
        &self,
        limit: usize,
    ) -> Result<Vec<DagRepairEscalatedPayload>, SsbViewerError> {
        let capped = limit.min(DRAWER_QUERY_HARD_CAP);
        let limit_i64 =
            i64::try_from(capped).map_err(|e| SsbViewerError::OutOfRange(e.to_string()))?;
        let conn = self.lock()?;
        let mut stmt = conn.prepare(
            "SELECT payload FROM ssb \
             WHERE action_type = ?1 ORDER BY seq DESC LIMIT ?2",
        )?;
        let rows = stmt.query_map(
            params![SsbActionType::DagRepairEscalated.as_sql_str(), limit_i64],
            |row| row.get::<_, Vec<u8>>(0),
        )?;
        let mut out = Vec::new();
        for row in rows {
            let bytes = row?;
            let payload: SsbPayload = rmp_serde::from_slice(&bytes)?;
            if let SsbPayload::DagRepairEscalated(p) = payload {
                out.push(p);
            }
        }
        Ok(out)
    }

    /// B-7 Task #8: Returns up to `limit` recent `task_split` rows that
    /// carry PhaseSizingSplitter context (`splitter = "phase_sizing"` and
    /// `phase_id = Some(_)`), in descending seq order (most recent first).
    /// Used by the dashboard to render split badges on phase nodes in the
    /// PlanWindow tree. Rows without phase context (supervision-layer
    /// splits, deterministic-fallback, research_tree) are filtered out.
    pub fn recent_phase_sized_splits(
        &self,
        limit: usize,
    ) -> Result<Vec<TaskSplitPayload>, SsbViewerError> {
        let capped = limit.min(DRAWER_QUERY_HARD_CAP);
        let limit_i64 =
            i64::try_from(capped).map_err(|e| SsbViewerError::OutOfRange(e.to_string()))?;
        let conn = self.lock()?;
        let mut stmt = conn.prepare(
            "SELECT payload FROM ssb \
             WHERE action_type = ?1 ORDER BY seq DESC LIMIT ?2",
        )?;
        let rows = stmt.query_map(
            params![SsbActionType::TaskSplit.as_sql_str(), limit_i64],
            |row| row.get::<_, Vec<u8>>(0),
        )?;
        let mut out = Vec::new();
        for row in rows {
            let bytes = row?;
            let payload: SsbPayload = rmp_serde::from_slice(&bytes)?;
            if let SsbPayload::TaskSplit(p) = payload {
                // Only retain rows that carry PhaseSizingSplitter context:
                // splitter must be "phase_sizing" and phase_id must be set.
                if p.splitter.as_deref() == Some("phase_sizing") && p.phase_id.is_some() {
                    out.push(p);
                }
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

    /// F-8 / Slice A1 (SCHEMA_VERSION 204) — SCF symbolic semantic conflicts.
    ///
    /// Returns the most recent [`SsbActionType::SymbolicConflictDetected`] rows
    /// (newest first), each describing a self-mod config variant that the
    /// LLM-free symbolic detector flagged as contradicting an active locked-key
    /// invariant (`self_mod_safety::LOCKED_KEYS`) and held PENDING at the
    /// `elai self-mod apply` admission boundary. POLICY is WARN-then-escalate,
    /// never hard fail-closed BLOCK — these rows surface held-pending variants
    /// for human escalation, not rejections. `cap` is clamped to
    /// [`DRAWER_QUERY_HARD_CAP`] to bound the IPC payload.
    pub fn symbolic_conflicts(
        &self,
        cap: usize,
    ) -> Result<Vec<SymbolicConflictRow>, SsbViewerError> {
        let effective = cap.min(DRAWER_QUERY_HARD_CAP);
        let limit_i64 = i64::try_from(effective).unwrap_or(i64::MAX);
        let conn = self.lock()?;
        let mut stmt = conn.prepare(
            "SELECT seq, created_at, payload FROM ssb \
             WHERE action_type = ?1 ORDER BY seq DESC LIMIT ?2",
        )?;
        let rows = stmt.query_map(
            params![
                SsbActionType::SymbolicConflictDetected.as_sql_str(),
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
        let mut out: Vec<SymbolicConflictRow> = Vec::new();
        for row in rows {
            let (seq, ts_ms, bytes) = row?;
            let payload: SsbPayload = match rmp_serde::from_slice(&bytes) {
                Ok(p) => p,
                Err(_) => continue,
            };
            if let SsbPayload::SymbolicConflictDetected(p) = payload {
                out.push(SymbolicConflictRow {
                    seq,
                    ts_ms,
                    fingerprint: p.fingerprint,
                    kind: p.kind,
                    key: p.key,
                    detail: p.detail,
                    candidate_id: p.candidate_id,
                    fsm_substate: p.fsm_substate,
                });
            }
        }
        Ok(out)
    }

    /// F-13 — list `session_archived` rows newest-first. Each row records a
    /// stale-version SSB (or precautionary session-DAG) store that was
    /// cold-archived to `workflow/sessions/{id}/archive/ssb-v{N}.sqlite` BEFORE
    /// the destructive `run_migration` DROP (gated wire-dark behind
    /// `ELAI_SSB_ARCHIVE_ON_BUMP_ENABLE`). `cap` is clamped to
    /// [`DRAWER_QUERY_HARD_CAP`] to bound the IPC payload. Returns an empty Vec
    /// when no archives have been recorded (the default, gate-OFF, state).
    pub fn session_archives(
        &self,
        cap: usize,
    ) -> Result<Vec<SessionArchiveRow>, SsbViewerError> {
        let effective = cap.min(DRAWER_QUERY_HARD_CAP);
        let limit_i64 = i64::try_from(effective).unwrap_or(i64::MAX);
        let conn = self.lock()?;
        let mut stmt = conn.prepare(
            "SELECT seq, created_at, payload FROM ssb \
             WHERE action_type = ?1 ORDER BY seq DESC LIMIT ?2",
        )?;
        let rows = stmt.query_map(
            params![SsbActionType::SessionArchived.as_sql_str(), limit_i64],
            |row| {
                Ok((
                    row.get::<_, i64>(0)?,
                    row.get::<_, i64>(1)?,
                    row.get::<_, Vec<u8>>(2)?,
                ))
            },
        )?;
        let mut out: Vec<SessionArchiveRow> = Vec::new();
        for row in rows {
            let (seq, ts_ms, bytes) = row?;
            let payload: SsbPayload = match rmp_serde::from_slice(&bytes) {
                Ok(p) => p,
                Err(_) => continue,
            };
            if let SsbPayload::SessionArchived(p) = payload {
                out.push(SessionArchiveRow {
                    seq,
                    ts_ms,
                    session_id: p.session_id,
                    stale_version: p.stale_version,
                    archive_path: p.archive_path,
                    manifest_blake3: p.manifest_blake3,
                    store_kind: p.store_kind,
                });
            }
        }
        Ok(out)
    }

    /// G-4 — list `mcp_rug_pull_blocked` rows newest-first. Each row records a
    /// live MCP rug-pull / line-jump: a reconnect-time `tools/list` re-fetch
    /// from a previously-approved server whose blake3 schema pin no longer
    /// matches. The changed tool(s) were blocked fail-closed until operator
    /// re-approval (`update_pin`). DISTINCT from `mcp_schema_pin_violation`
    /// (E-24 first-connect). `cap` is clamped to [`DRAWER_QUERY_HARD_CAP`] to
    /// bound the IPC payload. Returns an empty Vec when no rug-pulls have been
    /// recorded (the default state).
    pub fn mcp_rug_pulls(&self, cap: usize) -> Result<Vec<McpRugPullRow>, SsbViewerError> {
        let effective = cap.min(DRAWER_QUERY_HARD_CAP);
        let limit_i64 = i64::try_from(effective).unwrap_or(i64::MAX);
        let conn = self.lock()?;
        let mut stmt = conn.prepare(
            "SELECT seq, created_at, payload FROM ssb \
             WHERE action_type = ?1 ORDER BY seq DESC LIMIT ?2",
        )?;
        let rows = stmt.query_map(
            params![SsbActionType::McpRugPullBlocked.as_sql_str(), limit_i64],
            |row| {
                Ok((
                    row.get::<_, i64>(0)?,
                    row.get::<_, i64>(1)?,
                    row.get::<_, Vec<u8>>(2)?,
                ))
            },
        )?;
        let mut out: Vec<McpRugPullRow> = Vec::new();
        for row in rows {
            let (seq, ts_ms, bytes) = row?;
            let payload: SsbPayload = match rmp_serde::from_slice(&bytes) {
                Ok(p) => p,
                Err(_) => continue,
            };
            if let SsbPayload::McpRugPullBlocked(p) = payload {
                out.push(McpRugPullRow {
                    seq,
                    ts_ms,
                    server_url: p.server_url,
                    schema_blake3_before: p.schema_blake3_before,
                    schema_blake3_after: p.schema_blake3_after,
                    changed_tools: p.changed_tools,
                    description_diff_count: p.description_diff_count,
                    ingest_origin: p.ingest_origin,
                });
            }
        }
        Ok(out)
    }

    /// G-1 — list `taint_flow` rows newest-first. Each row records ONE
    /// data-plane taint propagation that was BLOCKED at a destructive sink: an
    /// untrusted ingest (web / tool-output / external file / sandbox stdout)
    /// flowed into a destructive sink (git commit / file write) and the
    /// `allow_tool_with_origin` capability gate rejected the call. `cap` is
    /// clamped to [`DRAWER_QUERY_HARD_CAP`] to bound the IPC payload. Returns an
    /// empty Vec when no tainted flow has been blocked (the default state).
    pub fn taint_flows(&self, cap: usize) -> Result<Vec<TaintFlowRow>, SsbViewerError> {
        let effective = cap.min(DRAWER_QUERY_HARD_CAP);
        let limit_i64 = i64::try_from(effective).unwrap_or(i64::MAX);
        let conn = self.lock()?;
        let mut stmt = conn.prepare(
            "SELECT seq, created_at, payload FROM ssb \
             WHERE action_type = ?1 ORDER BY seq DESC LIMIT ?2",
        )?;
        let rows = stmt.query_map(
            params![SsbActionType::TaintFlow.as_sql_str(), limit_i64],
            |row| {
                Ok((
                    row.get::<_, i64>(0)?,
                    row.get::<_, i64>(1)?,
                    row.get::<_, Vec<u8>>(2)?,
                ))
            },
        )?;
        let mut out: Vec<TaintFlowRow> = Vec::new();
        for row in rows {
            let (seq, ts_ms, bytes) = row?;
            let payload: SsbPayload = match rmp_serde::from_slice(&bytes) {
                Ok(p) => p,
                Err(_) => continue,
            };
            if let SsbPayload::TaintFlow(p) = payload {
                out.push(TaintFlowRow {
                    seq,
                    ts_ms,
                    kind: p.kind.label().to_owned(),
                    origin_blake3: p.origin_blake3,
                    trust_level: p.trust_level,
                    sink_tool: p.sink_tool.unwrap_or_default(),
                    blocked: p.blocked,
                    task_id: p.task_id,
                });
            }
        }
        Ok(out)
    }

    /// G-2 — list `run_artifact_emitted` rows newest-first. Each row records ONE
    /// Reproducible-Run Artifact (`.rra`) that was sealed at run end: the blake3
    /// root over the canonical bundle bytes, the honest per-lane guarantee level
    /// (`reproducible_action_stream` / `verify_divergence` — never
    /// `reproducible_tokens` for a cloud lane, GD-5), the F-gate evidence-
    /// manifest entry count, and the pinned `ELAI_SEED`. The drawer renders
    /// these so an operator can read the artifact's trust posture without
    /// opening the file. `cap` is clamped to [`DRAWER_QUERY_HARD_CAP`] to bound
    /// the IPC payload. Returns an empty Vec when no artifact has been sealed
    /// (the default state on a fresh install).
    pub fn run_artifacts(&self, cap: usize) -> Result<Vec<RunArtifactRow>, SsbViewerError> {
        let effective = cap.min(DRAWER_QUERY_HARD_CAP);
        let limit_i64 = i64::try_from(effective).unwrap_or(i64::MAX);
        let conn = self.lock()?;
        let mut stmt = conn.prepare(
            "SELECT seq, created_at, payload FROM ssb \
             WHERE action_type = ?1 ORDER BY seq DESC LIMIT ?2",
        )?;
        let rows = stmt.query_map(
            params![SsbActionType::RunArtifactEmitted.as_sql_str(), limit_i64],
            |row| {
                Ok((
                    row.get::<_, i64>(0)?,
                    row.get::<_, i64>(1)?,
                    row.get::<_, Vec<u8>>(2)?,
                ))
            },
        )?;
        let mut out: Vec<RunArtifactRow> = Vec::new();
        for row in rows {
            let (seq, ts_ms, bytes) = row?;
            let payload: SsbPayload = match rmp_serde::from_slice(&bytes) {
                Ok(p) => p,
                Err(_) => continue,
            };
            if let SsbPayload::RunArtifactEmitted(p) = payload {
                out.push(RunArtifactRow {
                    seq,
                    ts_ms,
                    root_blake3: p.root_blake3,
                    guarantee_level: p.guarantee_level,
                    gate_count: p.gate_count,
                    seed: p.seed,
                    session_id: p.session_id,
                    // I-45: carry objective hash to the dashboard feed; None on
                    // LEGACY receipts that predate this field.
                    objective_hash: p.objective_hash,
                    // J-14: filled from the ReproPackEvent join below; default
                    // None / empty until the join runs.
                    tier_effective: None,
                    tier_requested: None,
                    downgrades: Vec::new(),
                    divergence_position: None,
                    divergence_expected: None,
                    divergence_actual: None,
                    rerun_kind: None,
                    rerun_digest_a: None,
                    rerun_digest_b: None,
                    rerun_first_divergence_index: None,
                    rerun_divergent_metric: None,
                });
            }
        }
        // J-14: join latest ReproPackEvent rows by session_id. The SSB emits
        // TierEvaluated + FirstDivergenceLocated as separate rows keyed by
        // session_id; we scan the same row budget and keep the newest row per
        // session per kind. The verify-path emit is currently DARK (see S1 §2
        // caveat), so for most installs this leaves the Options None — the
        // honest empty state, not a failure.
        Self::join_repro_pack_events(&conn, &mut out)?;
        Self::join_rerun_hash_equality(&conn, &mut out)?;
        Ok(out)
    }

    // J-14 helper: no new pub symbol — private free fn called only from
    // `run_artifacts` above. Scans recent repro_pack_event rows (same hard cap
    // as the caller) and merges the latest TierEvaluated + FirstDivergenceLocated
    // per session_id into the matching RunArtifactRow entries. Rows with no
    // matching ReproPackEvent keep the default None / empty state.
    fn join_repro_pack_events(
        conn: &Connection,
        rows: &mut [RunArtifactRow],
    ) -> Result<(), SsbViewerError> {
        if rows.is_empty() {
            return Ok(());
        }
        // Pull the newest TierEvaluated + FirstDivergenceLocated per session.
        // We scan the same hard cap of recent repro_pack_event rows the caller
        // used; the SSB is append-only and ordered by seq DESC, so the first
        // match per (session_id, kind) is the latest.
        let effective = DRAWER_QUERY_HARD_CAP;
        let limit_i64 = i64::try_from(effective).unwrap_or(i64::MAX);
        let mut stmt = conn.prepare(
            "SELECT payload FROM ssb \
             WHERE action_type = ?1 ORDER BY seq DESC LIMIT ?2",
        )?;
        let payload_bytes: Vec<Vec<u8>> = stmt
            .query_map(
                params![SsbActionType::ReproPackEvent(ReproPackKind::TierEvaluated).as_sql_str(), limit_i64],
                |row| row.get::<_, Vec<u8>>(0),
            )?
            .filter_map(Result::ok)
            .collect();
        // First pass: latest TierEvaluated per session_id.
        let mut latest_tier: std::collections::HashMap<String, ReproPackPayload> =
            std::collections::HashMap::new();
        for bytes in &payload_bytes {
            let payload: SsbPayload = match rmp_serde::from_slice(bytes) {
                Ok(p) => p,
                Err(_) => continue,
            };
            if let SsbPayload::ReproPackEvent(p) = payload {
                if p.kind == ReproPackKind::TierEvaluated
                    && !latest_tier.contains_key(&p.session_id)
                {
                    latest_tier.insert(p.session_id.clone(), p);
                }
            }
        }
        // Second pass: latest FirstDivergenceLocated per session_id (separate
        // query because the action_type discriminator is the same string for
        // all ReproPackEvent arms — we filter by payload kind after decode).
        let mut latest_div: std::collections::HashMap<String, ReproPackPayload> =
            std::collections::HashMap::new();
        // Re-scan the same window; the action_type filter returns all arms
        // interleaved, so we walk the same bytes and pick the first
        // FirstDivergenceLocated per session.
        for bytes in &payload_bytes {
            let payload: SsbPayload = match rmp_serde::from_slice(bytes) {
                Ok(p) => p,
                Err(_) => continue,
            };
            if let SsbPayload::ReproPackEvent(p) = payload {
                if p.kind == ReproPackKind::FirstDivergenceLocated
                    && !latest_div.contains_key(&p.session_id)
                {
                    latest_div.insert(p.session_id.clone(), p);
                }
            }
        }
        // Merge into the caller's rows.
        for row in rows.iter_mut() {
            if let Some(t) = latest_tier.get(&row.session_id) {
                row.tier_effective = t.tier_effective.clone();
                row.tier_requested = t.tier_requested.clone();
                row.downgrades = t.downgrades.clone();
            }
            if let Some(d) = latest_div.get(&row.session_id) {
                row.divergence_position = d.divergence_position;
                row.divergence_expected = d.expected_value.clone();
                row.divergence_actual = d.actual_value.clone();
            }
        }
        Ok(())
    }

    // H-23 helper: no new pub symbol — private fn called only from
    // `run_artifacts` above. Scans recent rerun_hash_equality rows (same hard
    // cap as the caller) and merges the latest per session_id into the
    // matching RunArtifactRow entries. Rows with no matching row keep None.
    fn join_rerun_hash_equality(
        conn: &Connection,
        rows: &mut [RunArtifactRow],
    ) -> Result<(), SsbViewerError> {
        if rows.is_empty() {
            return Ok(());
        }
        let effective = DRAWER_QUERY_HARD_CAP;
        let limit_i64 = i64::try_from(effective).unwrap_or(i64::MAX);
        let mut stmt = conn.prepare(
            "SELECT session_id, payload FROM ssb \
             WHERE action_type = ?1 ORDER BY seq DESC LIMIT ?2",
        )?;
        let entries: Vec<(String, Vec<u8>)> = stmt
            .query_map(
                params![SsbActionType::RerunHashEquality.as_sql_str(), limit_i64],
                |row| Ok((row.get::<_, String>(0)?, row.get::<_, Vec<u8>>(1)?)),
            )?
            .filter_map(Result::ok)
            .collect();
        let mut latest: std::collections::HashMap<String, RerunHashEqualityPayload> =
            std::collections::HashMap::new();
        for (session_id, bytes) in &entries {
            let payload: SsbPayload = match rmp_serde::from_slice(bytes) {
                Ok(p) => p,
                Err(_) => continue,
            };
            if let SsbPayload::RerunHashEquality(p) = payload {
                if !latest.contains_key(session_id) {
                    latest.insert(session_id.clone(), p);
                }
            }
        }
        for row in rows.iter_mut() {
            if let Some(p) = latest.get(&row.session_id) {
                row.rerun_kind = Some(p.kind.label().to_string());
                row.rerun_digest_a = Some(p.digest_a.clone());
                row.rerun_digest_b = Some(p.digest_b.clone());
                row.rerun_first_divergence_index = Some(p.first_divergence_index);
                row.rerun_divergent_metric = Some(p.divergent_metric.clone());
            }
        }
        Ok(())
    }

    /// G-8 — list `provenance_attestation_emitted` rows newest-first. Each row
    /// records ONE signed in-toto v1 provenance attestation built from a sealed
    /// G-2 Reproducible-Run Artifact and emitted at the `elai run` exit: the
    /// blake3-Merkle subject digest (the value the git trailer carries + the
    /// offline verify recomputes), the G-2 run-artifact root the attestation
    /// chains to, the F-gate `verified_levels` rollup count (CONSUMED from the
    /// G-2 manifest, GD-1), an all-passed flag, and the honest signing model
    /// (`local_blake3_merkle`). The drawer renders these so an operator can read
    /// the run's off-machine trust posture without opening the sidecar. `cap` is
    /// clamped to [`DRAWER_QUERY_HARD_CAP`] to bound the IPC payload. Returns an
    /// empty Vec when no attestation has been emitted (the default state on a
    /// fresh install / before any `elai run` sealed one).
    pub fn attestations(&self, cap: usize) -> Result<Vec<AttestationRow>, SsbViewerError> {
        let effective = cap.min(DRAWER_QUERY_HARD_CAP);
        let limit_i64 = i64::try_from(effective).unwrap_or(i64::MAX);
        let conn = self.lock()?;
        let mut stmt = conn.prepare(
            "SELECT seq, created_at, payload FROM ssb \
             WHERE action_type = ?1 ORDER BY seq DESC LIMIT ?2",
        )?;
        let rows = stmt.query_map(
            params![
                SsbActionType::ProvenanceAttestationEmitted.as_sql_str(),
                limit_i64
            ],
            |row| {
                Ok((
                    row.get::<_, i64>(0)?,
                    row.get::<_, i64>(1)?,
                    row.get::<_, Vec<u8>>(2)?,
                ))
            },
        )?;
        let mut out: Vec<AttestationRow> = Vec::new();
        for row in rows {
            let (seq, ts_ms, bytes) = row?;
            let payload: SsbPayload = match rmp_serde::from_slice(&bytes) {
                Ok(p) => p,
                Err(_) => continue,
            };
            if let SsbPayload::ProvenanceAttestationEmitted(p) = payload {
                out.push(AttestationRow {
                    seq,
                    ts_ms,
                    session_id: p.session_id,
                    subject_blake3: p.subject_blake3,
                    run_artifact_root: p.run_artifact_root,
                    verified_levels: p.verified_levels,
                    all_passed: p.all_passed,
                    signing_model: p.signing_model,
                });
            }
        }
        Ok(out)
    }

    /// G-9 — list `secret_redacted` advisory rows newest-first. Each row records
    /// ONE write-time secret+PII redaction that fired on a typed SsbPayload
    /// free-text field at the top of `Ssb::write_payload`: the redacted field
    /// identity, the stable blake3-keyed token(s) that replaced the match(es),
    /// the match count, and the per-kind `(label, count)` summary — NEVER the
    /// raw secret. The panel renders the counts so an operator can see redaction
    /// is firing on the write path without ever seeing a secret. `cap` is
    /// clamped to [`DRAWER_QUERY_HARD_CAP`]. Returns an empty Vec when no
    /// redaction has fired (the default state when no secret has been written).
    pub fn redactions(&self, cap: usize) -> Result<Vec<RedactionStatRow>, SsbViewerError> {
        let effective = cap.min(DRAWER_QUERY_HARD_CAP);
        let limit_i64 = i64::try_from(effective).unwrap_or(i64::MAX);
        let conn = self.lock()?;
        let mut stmt = conn.prepare(
            "SELECT seq, created_at, payload FROM ssb \
             WHERE action_type = ?1 ORDER BY seq DESC LIMIT ?2",
        )?;
        let rows = stmt.query_map(
            params![SsbActionType::SecretRedacted.as_sql_str(), limit_i64],
            |row| {
                Ok((
                    row.get::<_, i64>(0)?,
                    row.get::<_, i64>(1)?,
                    row.get::<_, Vec<u8>>(2)?,
                ))
            },
        )?;
        let mut out: Vec<RedactionStatRow> = Vec::new();
        for row in rows {
            let (seq, ts_ms, bytes) = row?;
            let payload: SsbPayload = match rmp_serde::from_slice(&bytes) {
                Ok(p) => p,
                Err(_) => continue,
            };
            if let SsbPayload::SecretRedacted(p) = payload {
                out.push(RedactionStatRow {
                    seq,
                    ts_ms,
                    session_id: p.session_id,
                    field: p.field,
                    tokens: p.tokens,
                    match_count: p.match_count,
                    kind_counts: p.kind_counts,
                });
            }
        }
        Ok(out)
    }

    /// G-5 — list `review_surface_scored` rows newest-first. Each row records ONE
    /// reviewer-cockpit Diff-Surface-to-Review (DSR) score: the DSR ratio
    /// (un-vouched changed lines / total changed lines), the vouched / un-vouched
    /// / total changed-line counts, and the pinned diff-base commit the score was
    /// computed against. The cockpit window renders these so a reviewer sees the
    /// headline DSR + the vouched/un-vouched split without re-running the score.
    /// `cap` is clamped to [`DRAWER_QUERY_HARD_CAP`] to bound the IPC payload.
    /// Returns an empty Vec when no score has been recorded (the default state
    /// before any `elai review --cockpit` run).
    pub fn review_surfaces(&self, cap: usize) -> Result<Vec<ReviewSurfaceRow>, SsbViewerError> {
        let effective = cap.min(DRAWER_QUERY_HARD_CAP);
        let limit_i64 = i64::try_from(effective).unwrap_or(i64::MAX);
        let conn = self.lock()?;
        let mut stmt = conn.prepare(
            "SELECT seq, created_at, payload FROM ssb \
             WHERE action_type = ?1 ORDER BY seq DESC LIMIT ?2",
        )?;
        let rows = stmt.query_map(
            params![SsbActionType::ReviewSurfaceScored.as_sql_str(), limit_i64],
            |row| {
                Ok((
                    row.get::<_, i64>(0)?,
                    row.get::<_, i64>(1)?,
                    row.get::<_, Vec<u8>>(2)?,
                ))
            },
        )?;
        let mut out: Vec<ReviewSurfaceRow> = Vec::new();
        for row in rows {
            let (seq, ts_ms, bytes) = row?;
            let payload: SsbPayload = match rmp_serde::from_slice(&bytes) {
                Ok(p) => p,
                Err(_) => continue,
            };
            if let SsbPayload::ReviewSurfaceScored(p) = payload {
                out.push(ReviewSurfaceRow {
                    seq,
                    ts_ms,
                    session_id: p.session_id,
                    dsr: p.dsr,
                    total_changed_lines: p.total_changed_lines,
                    attention_lines: p.attention_lines,
                    vouched_hunks: p.vouched_hunks,
                    unvouched_hunks: p.unvouched_hunks,
                    base_commit: p.base_commit,
                    diff_available: p.diff_available,
                    diff_unavailable_reason: p.diff_unavailable_reason.unwrap_or_default(),
                });
            }
        }
        Ok(out)
    }

    /// G-7 — list `claim_confidence_scored` rows newest-first. Each row records
    /// ONE live per-change conformal decision carrying the REAL fused
    /// `claim_score` (log-prob LOCAL-only + self-consistency + exec-agreement),
    /// the three per-signal components, the lane, tau, the `decide()` verdict and
    /// the abstained flag. The cockpit confidence-triage band reads these to
    /// surface low-confidence hunks first. `cap` is clamped to
    /// [`DRAWER_QUERY_HARD_CAP`]. Returns an empty Vec when no per-change
    /// decision has fired (the default state behind the conformal triple gate).
    pub fn claim_confidences(
        &self,
        cap: usize,
    ) -> Result<Vec<ClaimConfidenceRow>, SsbViewerError> {
        let effective = cap.min(DRAWER_QUERY_HARD_CAP);
        let limit_i64 = i64::try_from(effective).unwrap_or(i64::MAX);
        let conn = self.lock()?;
        let mut stmt = conn.prepare(
            "SELECT seq, created_at, payload FROM ssb \
             WHERE action_type = ?1 ORDER BY seq DESC LIMIT ?2",
        )?;
        let rows = stmt.query_map(
            params![SsbActionType::ClaimConfidenceScored.as_sql_str(), limit_i64],
            |row| {
                Ok((
                    row.get::<_, i64>(0)?,
                    row.get::<_, i64>(1)?,
                    row.get::<_, Vec<u8>>(2)?,
                ))
            },
        )?;
        let mut out: Vec<ClaimConfidenceRow> = Vec::new();
        for row in rows {
            let (seq, ts_ms, bytes) = row?;
            let payload: SsbPayload = match rmp_serde::from_slice(&bytes) {
                Ok(p) => p,
                Err(_) => continue,
            };
            if let SsbPayload::ClaimConfidenceScored(p) = payload {
                out.push(ClaimConfidenceRow {
                    seq,
                    ts_ms,
                    claim_id: p.claim_id,
                    claim_score: p.claim_score,
                    logprob_component: p.logprob_component,
                    self_consistency_component: p.self_consistency_component,
                    exec_agreement_component: p.exec_agreement_component,
                    lane: p.lane,
                    tau: p.tau,
                    conformal_verdict: p.conformal_verdict,
                    abstained: p.abstained,
                });
            }
        }
        Ok(out)
    }

    /// G-7 — list `trifecta_quarantined` rows newest-first. Each row records ONE
    /// tool-call chokepoint where the secret-present + untrusted-span + egress
    /// legs co-occurred and the untrusted span was held back from the tool-caller.
    /// Carries the blake3 of the untrusted origin (NEVER the raw span), the sink
    /// tool, the three component flags, the quarantined flag and the no-tool
    /// sub-LLM verdict. `cap` is clamped to [`DRAWER_QUERY_HARD_CAP`]. Returns an
    /// empty Vec when no quarantine has fired (the default state — a benign run
    /// never trips the trifecta).
    pub fn trifecta_quarantines(
        &self,
        cap: usize,
    ) -> Result<Vec<TrifectaQuarantineRow>, SsbViewerError> {
        let effective = cap.min(DRAWER_QUERY_HARD_CAP);
        let limit_i64 = i64::try_from(effective).unwrap_or(i64::MAX);
        let conn = self.lock()?;
        let mut stmt = conn.prepare(
            "SELECT seq, created_at, payload FROM ssb \
             WHERE action_type = ?1 ORDER BY seq DESC LIMIT ?2",
        )?;
        let rows = stmt.query_map(
            params![SsbActionType::TrifectaQuarantined.as_sql_str(), limit_i64],
            |row| {
                Ok((
                    row.get::<_, i64>(0)?,
                    row.get::<_, i64>(1)?,
                    row.get::<_, Vec<u8>>(2)?,
                ))
            },
        )?;
        let mut out: Vec<TrifectaQuarantineRow> = Vec::new();
        for row in rows {
            let (seq, ts_ms, bytes) = row?;
            let payload: SsbPayload = match rmp_serde::from_slice(&bytes) {
                Ok(p) => p,
                Err(_) => continue,
            };
            if let SsbPayload::TrifectaQuarantined(p) = payload {
                out.push(TrifectaQuarantineRow {
                    seq,
                    ts_ms,
                    task_id: p.task_id,
                    untrusted_origin_blake3: p.untrusted_origin_blake3,
                    sink_tool: p.sink_tool,
                    secret_present: p.secret_present,
                    untrusted_span: p.untrusted_span,
                    egress_available: p.egress_available,
                    quarantined: p.quarantined,
                    sub_llm_verdict: p.sub_llm_verdict,
                });
            }
        }
        Ok(out)
    }

    /// G-3 — list `blast_radius_blocked` rows newest-first. Each row records ONE
    /// deterministic runtime gate block: either a shell-EXPANDED FS-mutating
    /// target that left the worktree root (`kind = fs_escape`) or an egress
    /// host that was NUL/control-rejected or resolved outside the pinned
    /// allow-set (`kind = egress_denied`). The drawer renders the pre- vs
    /// post-expansion command diff (the `#10077` forensics fix) so an operator
    /// can see exactly what the shell would have run. `cap` is clamped to
    /// [`DRAWER_QUERY_HARD_CAP`]. Returns an empty Vec when no block has been
    /// recorded (the default state — a block is the exception, not the rule).
    pub fn blast_radius_blocks(
        &self,
        cap: usize,
    ) -> Result<Vec<BlastRadiusBlockRow>, SsbViewerError> {
        let effective = cap.min(DRAWER_QUERY_HARD_CAP);
        let limit_i64 = i64::try_from(effective).unwrap_or(i64::MAX);
        let conn = self.lock()?;
        let mut stmt = conn.prepare(
            "SELECT seq, created_at, payload FROM ssb \
             WHERE action_type = ?1 ORDER BY seq DESC LIMIT ?2",
        )?;
        let rows = stmt.query_map(
            params![SsbActionType::BlastRadiusBlocked.as_sql_str(), limit_i64],
            |row| {
                Ok((
                    row.get::<_, i64>(0)?,
                    row.get::<_, i64>(1)?,
                    row.get::<_, Vec<u8>>(2)?,
                ))
            },
        )?;
        let mut out: Vec<BlastRadiusBlockRow> = Vec::new();
        for row in rows {
            let (seq, ts_ms, bytes) = row?;
            let payload: SsbPayload = match rmp_serde::from_slice(&bytes) {
                Ok(p) => p,
                Err(_) => continue,
            };
            if let SsbPayload::BlastRadiusBlocked(p) = payload {
                out.push(BlastRadiusBlockRow {
                    seq,
                    ts_ms,
                    kind: p.kind.label().to_owned(),
                    pre_expansion_command: p.pre_expansion_command,
                    post_expansion_command: p.post_expansion_command,
                    target_or_host: p.target_or_host,
                    worktree_root_blake3: p.worktree_root_blake3,
                    trust_label: p.trust_label,
                });
            }
        }
        Ok(out)
    }

    /// F-11 — list `darwinian_admit` rows newest-first. Each row records ONE
    /// candidate routed through the config-DGM four-guard chain: `Admitted`
    /// (passed all guards + inserted), `RejectedByGuard` (a guard fired —
    /// `guard_rejected_by` names it), or `LoopClosed` (`select_parent` ran after
    /// `insert_variant`). Wire-dark producer: the rows only appear after a
    /// `darwin-loop` run with `ELAI_DARWINIAN_LOOP_ENABLE=1` + feature
    /// `darwinian-evolve`. `cap` is clamped to [`DRAWER_QUERY_HARD_CAP`] to bound
    /// the IPC payload. Returns an empty Vec when no admit rows exist (the
    /// default, gate-OFF state).
    pub fn darwinian_admits(
        &self,
        cap: usize,
    ) -> Result<Vec<DarwinianAdmitRow>, SsbViewerError> {
        let effective = cap.min(DRAWER_QUERY_HARD_CAP);
        let limit_i64 = i64::try_from(effective).unwrap_or(i64::MAX);
        let conn = self.lock()?;
        let mut stmt = conn.prepare(
            "SELECT seq, created_at, payload FROM ssb \
             WHERE action_type = ?1 ORDER BY seq DESC LIMIT ?2",
        )?;
        let rows = stmt.query_map(
            params![SsbActionType::DarwinianAdmit.as_sql_str(), limit_i64],
            |row| {
                Ok((
                    row.get::<_, i64>(0)?,
                    row.get::<_, i64>(1)?,
                    row.get::<_, Vec<u8>>(2)?,
                ))
            },
        )?;
        let mut out: Vec<DarwinianAdmitRow> = Vec::new();
        for row in rows {
            let (seq, ts_ms, bytes) = row?;
            let payload: SsbPayload = match rmp_serde::from_slice(&bytes) {
                Ok(p) => p,
                Err(_) => continue,
            };
            if let SsbPayload::DarwinianAdmit(p) = payload {
                out.push(DarwinianAdmitRow {
                    seq,
                    ts_ms,
                    kind: p.kind.as_str().to_string(),
                    variant_id: p.variant_id,
                    parent_id: p.parent_id,
                    alpha: p.alpha,
                    guard_rejected_by: p.guard_rejected_by,
                    candidate_blake3: p.candidate_blake3,
                });
            }
        }
        Ok(out)
    }

    /// H-46 — list CMP parent-selection events newest-first. Each row is one
    /// `CladeSelected` SSB row emitted by `config_archive::select_parent_with_cmp`
    /// during a DGM generation. Wire-dark producer (gated by `darwinian-evolve`
    /// feature + `ELAI_DARWINIAN_LOOP_ENABLE=1`); empty in steady state.
    /// `cap` is clamped to [`DRAWER_QUERY_HARD_CAP`] to bound the IPC payload.
    pub fn clade_selections(
        &self,
        cap: usize,
    ) -> Result<Vec<CladeSelectionRow>, SsbViewerError> {
        let effective = cap.min(DRAWER_QUERY_HARD_CAP);
        let limit_i64 = i64::try_from(effective).unwrap_or(i64::MAX);
        let conn = self.lock()?;
        let mut stmt = conn.prepare(
            "SELECT seq, created_at, payload FROM ssb \
             WHERE action_type = ?1 ORDER BY seq DESC LIMIT ?2",
        )?;
        let rows = stmt.query_map(
            params![SsbActionType::CladeSelected.as_sql_str(), limit_i64],
            |row| {
                Ok((
                    row.get::<_, i64>(0)?,
                    row.get::<_, i64>(1)?,
                    row.get::<_, Vec<u8>>(2)?,
                ))
            },
        )?;
        let mut out: Vec<CladeSelectionRow> = Vec::new();
        for row in rows {
            let (seq, ts_ms, bytes) = row?;
            let payload: SsbPayload = match rmp_serde::from_slice(&bytes) {
                Ok(p) => p,
                Err(_) => continue,
            };
            if let SsbPayload::CladeSelected(p) = payload {
                out.push(CladeSelectionRow {
                    seq,
                    ts_ms,
                    kind: p.kind.as_str().to_string(),
                    node_id: p.node_id,
                    cmp: p.cmp,
                    clade_size: p.clade_size,
                    immediate_score: p.immediate_score,
                    key_used: p.key_used,
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

    /// B-39: latest REFLECT pipeline state for the current session.
    ///
    /// Returns the most recent row of each type (`reflect_triaged`,
    /// `reflect_audit_verdict`, `reflect_refined`) by max seq, plus an
    /// `enabled` boolean that is `true` when any `reflect_*` row exists.
    ///
    /// Rows are fetched in a single SELECT WHERE action_type IN (...) ORDER BY id DESC;
    /// the result is partitioned in Rust so only one DB round-trip is needed.
    pub fn get_reflect_state(&self) -> Result<ReflectState, SsbViewerError> {
        let conn = self.lock()?;
        let mut stmt = conn.prepare(
            "SELECT action_type, payload FROM ssb \
             WHERE action_type IN (?1, ?2, ?3, ?4) \
             ORDER BY seq DESC",
        )?;
        let rows = stmt.query_map(
            params![
                SsbActionType::ReflectTriaged.as_sql_str(),
                SsbActionType::ReflectCandidate.as_sql_str(),
                SsbActionType::ReflectAuditVerdict.as_sql_str(),
                SsbActionType::ReflectRefined.as_sql_str(),
            ],
            |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, Vec<u8>>(1)?,
                ))
            },
        )?;

        let mut last_triage: Option<ReflectTriagedPayload> = None;
        let mut last_verdict: Option<ReflectAuditVerdictPayload> = None;
        let mut last_summary: Option<ReflectRefinedPayload> = None;
        let mut any_reflect_row = false;

        for row in rows {
            let (at_str, bytes) = row?;
            any_reflect_row = true;
            // Only capture the first (most-recent) row of each type; subsequent
            // rows are older and can be skipped once we have all three.
            let need_more = last_triage.is_none() || last_verdict.is_none() || last_summary.is_none();
            if !need_more {
                break;
            }
            match at_str.as_str() {
                "reflect_triaged" if last_triage.is_none() => {
                    if let Ok(SsbPayload::ReflectTriaged(p)) = rmp_serde::from_slice(&bytes) {
                        last_triage = Some(p);
                    }
                }
                "reflect_audit_verdict" if last_verdict.is_none() => {
                    if let Ok(SsbPayload::ReflectAuditVerdict(p)) = rmp_serde::from_slice(&bytes) {
                        last_verdict = Some(p);
                    }
                }
                "reflect_refined" if last_summary.is_none() => {
                    if let Ok(SsbPayload::ReflectRefined(p)) = rmp_serde::from_slice(&bytes) {
                        last_summary = Some(p);
                    }
                }
                _ => {}
            }
        }

        Ok(ReflectState {
            enabled: any_reflect_row,
            last_triage,
            last_verdict,
            last_summary,
        })
    }

    /// B-28 DASHBOARD: read decoded [`SsbPayload`] rows for a set of
    /// focus-tunnel action types, optionally narrowed to a single
    /// `session_id`. Returns newest-first, hard-capped at
    /// [`DRAWER_QUERY_HARD_CAP`] rows total across all requested types.
    ///
    /// The caller passes a slice of [`SsbActionType`] discriminants; rows
    /// whose payload fails to decode are silently skipped (same pattern as
    /// [`SsbViewer::latest_msi_per_mode`] L3553). When `session_id` is
    /// `None`, rows from every session are returned (useful when the
    /// dashboard does not yet know the active task's session_id).
    pub fn focus_tunnel_payloads(
        &self,
        action_types: &[SsbActionType],
        session_id: Option<&str>,
        limit: usize,
    ) -> Result<Vec<SsbPayload>, SsbViewerError> {
        if action_types.is_empty() {
            return Ok(Vec::new());
        }
        let effective = limit.min(DRAWER_QUERY_HARD_CAP);
        let effective_i64 = i64::try_from(effective)
            .map_err(|e| SsbViewerError::OutOfRange(format!("limit: {e}")))?;
        let placeholders = std::iter::repeat("?")
            .take(action_types.len())
            .collect::<Vec<_>>()
            .join(", ");
        let conn = self.lock()?;
        let mut out: Vec<SsbPayload> = Vec::with_capacity(effective);
        if let Some(sid) = session_id {
            let sql = format!(
                "SELECT payload FROM ssb \
                 WHERE action_type IN ({placeholders}) AND session_id = ? \
                 ORDER BY seq DESC LIMIT ?"
            );
            let mut stmt = conn.prepare(&sql)?;
            let mut params_vec: Vec<Box<dyn rusqlite::ToSql>> = action_types
                .iter()
                .map(|at| {
                    Box::new(at.as_sql_str().to_owned()) as Box<dyn rusqlite::ToSql>
                })
                .collect();
            params_vec.push(Box::new(sid.to_owned()));
            params_vec.push(Box::new(effective_i64));
            let params_refs: Vec<&dyn rusqlite::ToSql> =
                params_vec.iter().map(|b| b.as_ref()).collect();
            let rows = stmt.query_map(params_refs.as_slice(), |row| {
                row.get::<_, Vec<u8>>(0)
            })?;
            for row in rows {
                let bytes = row?;
                if let Ok(p) = rmp_serde::from_slice::<SsbPayload>(&bytes) {
                    out.push(p);
                }
            }
        } else {
            let sql = format!(
                "SELECT payload FROM ssb \
                 WHERE action_type IN ({placeholders}) \
                 ORDER BY seq DESC LIMIT ?"
            );
            let mut stmt = conn.prepare(&sql)?;
            let mut params_vec: Vec<Box<dyn rusqlite::ToSql>> = action_types
                .iter()
                .map(|at| {
                    Box::new(at.as_sql_str().to_owned()) as Box<dyn rusqlite::ToSql>
                })
                .collect();
            params_vec.push(Box::new(effective_i64));
            let params_refs: Vec<&dyn rusqlite::ToSql> =
                params_vec.iter().map(|b| b.as_ref()).collect();
            let rows = stmt.query_map(params_refs.as_slice(), |row| {
                row.get::<_, Vec<u8>>(0)
            })?;
            for row in rows {
                let bytes = row?;
                if let Ok(p) = rmp_serde::from_slice::<SsbPayload>(&bytes) {
                    out.push(p);
                }
            }
        }
        Ok(out)
    }

    /// B-40 — Return (session_id, payload_bytes) tuples for an action type,
    /// ordered seq ASC. Caller decodes payload_bytes via `rmp_serde` against
    /// the appropriate `SsbPayload` variant. Cap: 500 rows.
    ///
    /// Used by dashboard-v2 `get_idea_framing_state` to assemble the
    /// IdeaFramingState DTO from IdeaFramingStarted + CriticRoundComplete +
    /// IdeaFramed SSB rows (the dashboard scopes by session_id at decode time).
    pub fn rows_with_session_by_action_type(
        &self,
        action_type: SsbActionType,
    ) -> Result<Vec<(String, Vec<u8>)>, SsbViewerError> {
        const MAX_ROWS: i64 = 500;
        let conn = self.lock()?;
        let mut stmt = conn.prepare(
            "SELECT session_id, payload FROM ssb \
             WHERE action_type = ?1 ORDER BY seq ASC LIMIT ?2",
        )?;
        let rows = stmt.query_map(
            params![action_type.as_sql_str(), MAX_ROWS],
            |row| Ok((row.get::<_, String>(0)?, row.get::<_, Vec<u8>>(1)?)),
        )?;
        let mut out = Vec::new();
        for row in rows {
            out.push(row?);
        }
        Ok(out)
    }

    /// C-4 — aggregate `cloud_spend` rows into a single
    /// [`CloudSpendSummary`] for the dashboard `get_cloud_spend` Tauri
    /// command.
    ///
    /// One pass scans up to [`CLOUD_SPEND_SCAN_LIMIT`] of the most recent
    /// rows. Latest row's `cost_usd` populates `task_usd`. Rows whose
    /// `created_at` falls within the current UTC-day window are summed
    /// into `day_usd`, and their token counters accumulate into
    /// `cache_creation_tokens` / `cache_read_tokens`.
    ///
    /// Empty SSB or no `cloud_spend` rows -> zeroed
    /// [`CloudSpendSummary::default`]. The wire shape is stable regardless
    /// of how many rows exist; consumers never have to branch on
    /// `Option<None>`.
    ///
    /// `now_ms` is the wall-clock millisecond timestamp used to derive the
    /// "today" boundary. Threading a clock through here (rather than
    /// reading `OsClock` directly) keeps deterministic-replay tests stable.
    /// NOTE: the SSB stores `created_at` in seconds since epoch
    /// (`ssb::execute_write` writes `as_secs()`), so we convert the
    /// caller's millisecond `now` to a seconds-based day boundary before
    /// comparing.
    pub fn cloud_spend_summary(
        &self,
        now_ms: i64,
    ) -> Result<CloudSpendSummary, SsbViewerError> {
        const CLOUD_SPEND_SCAN_LIMIT: i64 = 10_000;

        // SSB stores created_at in *seconds*. Convert the caller's `now_ms`
        // into a seconds-based day boundary so the inequality compares
        // like to like.
        const SECS_PER_DAY: i64 = 86_400;
        // C-5: 30-day rolling window approximation of "monthly" — calendar-month
        // boundaries cost an extra SQL pass and DST edge-cases for little
        // gain at the cap-enforcement granularity (monthly_cap=50.0 typical).
        const SECS_PER_MONTH_WINDOW: i64 = 30 * SECS_PER_DAY;
        let now_secs = if now_ms > 0 { now_ms / 1000 } else { 0 };
        let day_start_secs = (now_secs / SECS_PER_DAY) * SECS_PER_DAY;
        let month_window_start_secs = now_secs.saturating_sub(SECS_PER_MONTH_WINDOW);

        let conn = self.lock()?;
        let mut stmt = conn.prepare(
            "SELECT created_at, payload FROM ssb \
             WHERE action_type = ?1 ORDER BY seq DESC LIMIT ?2",
        )?;
        let rows = stmt.query_map(
            params![SsbActionType::CloudSpend.as_sql_str(), CLOUD_SPEND_SCAN_LIMIT],
            |row| Ok((row.get::<_, i64>(0)?, row.get::<_, Vec<u8>>(1)?)),
        )?;

        let mut summary = CloudSpendSummary::default();
        let mut task_assigned = false;
        for row in rows {
            let (created_at_secs, bytes) = row?;
            let payload: SsbPayload = match rmp_serde::from_slice(&bytes) {
                Ok(p) => p,
                Err(_) => continue,
            };
            let SsbPayload::CloudSpend(p) = payload else {
                continue;
            };
            if !task_assigned {
                summary.task_usd = p.cost_usd;
                task_assigned = true;
            }
            if created_at_secs >= day_start_secs {
                summary.day_usd += p.cost_usd;
                summary.cache_creation_tokens = summary
                    .cache_creation_tokens
                    .saturating_add(u64::from(p.cache_creation_tokens));
                summary.cache_read_tokens = summary
                    .cache_read_tokens
                    .saturating_add(u64::from(p.cache_read_tokens));
            }
            // C-5: monthly window is a superset of the daily window, so
            // this branch always fires when the daily branch did. Cap
            // enforcement for `spend_cap_monthly_usd` reads `month_usd`.
            if created_at_secs >= month_window_start_secs {
                summary.month_usd += p.cost_usd;
            }
            // redaction_count: no source field in CloudSpendPayload today;
            // wired in for a future `redaction_event` SSB row (see
            // C-4-decisions.md Q5).
            let _ = &p;
        }

        // I-10: query CacheEconomicsEvent rows for TTL tier + guard trips.
        // ttl_tier = most-recent TtlPlanned row's tier label.
        // guard_trips_today = count of HitRateGuardTripped rows within the day window.
        let mut eco_stmt = conn.prepare(
            "SELECT created_at, payload FROM ssb \
             WHERE action_type = ?1 ORDER BY seq DESC LIMIT ?2",
        )?;
        let eco_rows = eco_stmt.query_map(
            params![SsbActionType::CacheEconomicsEvent(
                crate::ssb::CacheEconomicsEventKind::TtlPlanned
            ).as_sql_str(), CLOUD_SPEND_SCAN_LIMIT],
            |row| Ok((row.get::<_, i64>(0)?, row.get::<_, Vec<u8>>(1)?)),
        )?;
        let mut ttl_tier_assigned = false;
        for row in eco_rows {
            let (created_at_secs, bytes) = match row {
                Ok(r) => r,
                Err(_) => continue,
            };
            let payload: SsbPayload = match rmp_serde::from_slice(&bytes) {
                Ok(p) => p,
                Err(_) => continue,
            };
            let SsbPayload::CacheEconomicsEvent(p) = payload else {
                continue;
            };
            // Most-recent TtlPlanned row sets ttl_tier + breakeven_reads.
            if !ttl_tier_assigned && p.kind == crate::ssb::CacheEconomicsEventKind::TtlPlanned {
                use crate::cache_economics::TtlTier;
                let tier_label = match p.ttl_secs {
                    Some(3600) => TtlTier::Long.label(),
                    Some(300) => TtlTier::Short.label(),
                    _ => TtlTier::NoCache.label(),
                };
                summary.ttl_tier = Some(tier_label.to_string());
                summary.breakeven_reads = p.breakeven_reads;
                ttl_tier_assigned = true;
            }
            // Count HitRateGuardTripped events within today's window.
            if created_at_secs >= day_start_secs
                && p.kind == crate::ssb::CacheEconomicsEventKind::HitRateGuardTripped
            {
                summary.guard_trips_today = summary.guard_trips_today.saturating_add(1);
            }
        }

        Ok(summary)
    }

    /// I-18: Query all `row_chain_sealed` rows for a given `session_id`, in
    /// ascending `seq` order. Returns a `Vec<(seq, payload_bytes)>` so the
    /// caller can decode and walk the chain without loading the full session
    /// stream.
    ///
    /// Uses `SsbViewer` (read-only SQLite) — no write handle needed.
    /// Filters by `session_id` via a SQL `WHERE session_id = ?` clause so only
    /// rows belonging to this session are fetched. A defence-in-depth payload
    /// check re-confirms the session match after decoding to guard against any
    /// divergence between the column value and the embedded payload field.
    pub fn query_row_chain_sealed_for_session(
        &self,
        session_id: &str,
    ) -> Result<Vec<(i64, Vec<u8>)>, SsbViewerError> {
        use crate::ssb::{decode_ssb_payload, RowChainSealedPayload, SsbPayload};

        let conn = self.lock()?;
        // Filter by session_id in SQL (session_id is a first-class column in the
        // ssb table) so we never scan rows from other sessions. The in-process
        // session_id check on the decoded payload is kept as a defence-in-depth
        // guard for any path where the column value and payload diverge.
        let mut stmt = conn.prepare(
            "SELECT seq, payload FROM ssb \
             WHERE action_type = 'row_chain_sealed' \
             AND session_id = ?1 \
             ORDER BY seq ASC",
        )?;
        let rows = stmt.query_map(rusqlite::params![session_id], |row| {
            Ok((row.get::<_, i64>(0)?, row.get::<_, Vec<u8>>(1)?))
        })?;

        let mut result = Vec::new();
        for row in rows {
            let (seq, bytes) = row?;
            // Defence-in-depth: confirm the decoded payload session_id matches.
            if let Ok(SsbPayload::RowChainSealed(p)) = decode_ssb_payload(&bytes) {
                if p.session_id == session_id {
                    result.push((seq, bytes));
                }
            }
        }
        Ok(result)
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

/// F-8 / Slice A1 (SCHEMA_VERSION 204) — DTO emitted by
/// [`SsbViewer::symbolic_conflicts`]. One row per held-pending self-mod config
/// variant the SCF symbolic detector flagged against a locked-key invariant.
/// Flat record (mirrors the flat `SymbolicConflictPayload`); WARN-then-escalate
/// — these are held-pending variants for human escalation, not rejections.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SymbolicConflictRow {
    /// SSB sequence number (newest-first ordering key).
    pub seq: i64,
    /// Row creation wall-clock ms.
    pub ts_ms: i64,
    /// blake3 hex-prefix of the conflict tuple (kind + key + candidate_id).
    pub fingerprint: String,
    /// Conflict class / symbolic tag.
    pub kind: String,
    /// The locked invariant key the variant contradicts.
    pub key: String,
    /// Human-readable contradiction detail (proposed vs locked invariant).
    pub detail: String,
    /// Stable id of the flagged-and-held-pending config variant.
    pub candidate_id: String,
    /// FSM agent sub-state at flag time.
    pub fsm_substate: String,
}

/// F-13 — DTO for [`SsbViewer::session_archives`]. One row per stale-version
/// store cold-archived BEFORE a destructive `run_migration` DROP (gated wire-
/// dark behind `ELAI_SSB_ARCHIVE_ON_BUMP_ENABLE`). Flat record mirroring the
/// flat `SessionArchivedPayload`; field names serialize verbatim, so the
/// frontend `SessionArchive` DTO must mirror them exactly.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionArchiveRow {
    /// SSB sequence number (newest-first ordering key).
    pub seq: i64,
    /// Row creation wall-clock ms.
    pub ts_ms: i64,
    /// The session whose stale-version store was archived.
    pub session_id: String,
    /// The on-disk `user_version` the store carried BEFORE the bump.
    pub stale_version: i64,
    /// Relative path of the archive file under the per-session workflow dir.
    pub archive_path: String,
    /// blake3 hex-prefix of the archived bytes (integrity check).
    pub manifest_blake3: String,
    /// Which store: `"ssb"` (destructive journal) or `"session_dag"`
    /// (precautionary, non-destructive copy).
    pub store_kind: String,
}

/// G-4 — DTO for [`SsbViewer::mcp_rug_pulls`]. One row per live MCP rug-pull /
/// line-jump (a reconnect re-fetch whose schema pin no longer matched). Mirrors
/// the [`crate::ssb::McpRugPullBlockedPayload`] fields, so the frontend
/// `McpRugPull` DTO must mirror them exactly (serde serializes the Rust field
/// names verbatim over the IPC boundary).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpRugPullRow {
    /// SSB sequence number (newest-first ordering key).
    pub seq: i64,
    /// Row creation wall-clock ms.
    pub ts_ms: i64,
    /// The MCP server whose re-fetched tool list no longer matched the pin.
    pub server_url: String,
    /// blake3 schema hash at pin time (before the rug-pull).
    pub schema_blake3_before: String,
    /// blake3 schema hash of the rug-pulled re-fetch (after).
    pub schema_blake3_after: String,
    /// Names of the tools whose description/presence changed (line-jump set).
    pub changed_tools: Vec<String>,
    /// Count of per-tool description swaps (the line-jump signal).
    pub description_diff_count: u32,
    /// The production entrypoint that re-fetched and tripped the gate.
    pub ingest_origin: String,
}

/// G-1 — DTO for [`SsbViewer::taint_flows`]. One row per blocked data-plane
/// taint propagation (an untrusted ingest reaching a destructive sink that the
/// capability gate rejected). Mirrors the [`crate::ssb::TaintFlowPayload`]
/// fields (with `kind` flattened to its `&str` label and `sink_tool`
/// defaulted), so the frontend `TaintFlow` DTO must mirror them exactly.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaintFlowRow {
    /// SSB sequence number (newest-first ordering key).
    pub seq: i64,
    /// Row creation wall-clock ms.
    pub ts_ms: i64,
    /// Ingest Kind label: `web` / `tool_output` / `file` / `sandbox_output`.
    pub kind: String,
    /// blake3 hex-prefix of the ingest origin (URL / server+tool / path / id).
    pub origin_blake3: String,
    /// The joined trust label that reached the sink (`untrusted`).
    pub trust_level: String,
    /// The destructive sink tool name that was blocked (`git_commit` / `edit`).
    pub sink_tool: String,
    /// Whether the sink was blocked (true for an emitted taint_flow row).
    pub blocked: bool,
    /// The task / turn id the flow belongs to (dashboard correlation).
    pub task_id: String,
}

/// G-2 — DTO for [`SsbViewer::run_artifacts`]. One row per sealed Reproducible-
/// Run Artifact (`.rra`). Mirrors the [`crate::ssb::RunArtifactEmittedPayload`]
/// fields verbatim, so the frontend `RunArtifact` DTO must mirror them exactly
/// (serde serializes the Rust field names verbatim over the IPC boundary).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RunArtifactRow {
    /// SSB sequence number (newest-first ordering key).
    pub seq: i64,
    /// Row creation wall-clock ms.
    pub ts_ms: i64,
    /// blake3 hex-prefix of the canonical `.rra` bundle bytes (the root).
    pub root_blake3: String,
    /// Honest per-lane guarantee label (`reproducible_action_stream` /
    /// `verify_divergence`; never `reproducible_tokens` for a cloud lane).
    pub guarantee_level: String,
    /// Number of F-gate verdict-hash entries in the evidence manifest.
    pub gate_count: u32,
    /// The pinned `ELAI_SEED` the action stream re-derives under.
    pub seed: u64,
    /// The session whose stream was bundled.
    pub session_id: String,
    /// I-45: 16-hex blake3 objective hash sealed at run time. `None` on LEGACY
    /// receipts (pre-I-45).
    #[serde(default)]
    pub objective_hash: Option<String>,
    /// J-14: effective tier verdict (snake_case TierVerdictLabel) joined from
    /// the latest `TierEvaluated` ReproPackEvent row for this session_id.
    /// `None` when no tier verdict row exists (LEGACY receipts or verify path
    /// SSB emit still DARK — see S1 §2 caveat).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tier_effective: Option<String>,
    /// J-14: requested tier (snake_case); `None` when no tier row exists.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tier_requested: Option<String>,
    /// J-14: recorded downgrades as already-formatted "from->to:reason" strings
    /// from the latest `TierEvaluated` row. Empty when no tier row or no
    /// downgrades were recorded.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub downgrades: Vec<String>,
    /// J-14: first divergent position (0-based) from the latest
    /// `FirstDivergenceLocated` row for this session_id. `None` when no
    /// divergence row exists (T0 action-faithful or LEGACY).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub divergence_position: Option<u64>,
    /// J-14: expected value (hex) at the divergence position. `None` when no
    /// divergence row exists.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub divergence_expected: Option<String>,
    /// J-14: actual value (hex) at the divergence position. `None` when no
    /// divergence row exists.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub divergence_actual: Option<String>,
    /// H-23: rerun verdict kind ("equal" / "divergent") joined from the latest
    /// `RerunHashEquality` SSB row for this session_id. `None` when no rerun row
    /// exists.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub rerun_kind: Option<String>,
    /// H-23: first digest from the RerunHashEquality row.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub rerun_digest_a: Option<String>,
    /// H-23: second digest from the RerunHashEquality row.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub rerun_digest_b: Option<String>,
    /// H-23: first divergence index (0-based) from the RerunHashEquality row.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub rerun_first_divergence_index: Option<u32>,
    /// H-23: divergent metric name from the RerunHashEquality row.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub rerun_divergent_metric: Option<String>,
}

/// G-8 — DTO for [`SsbViewer::attestations`]. One row per signed in-toto v1
/// provenance attestation emitted at the `elai run` exit. Mirrors the
/// [`crate::ssb::ProvenanceAttestationEmittedPayload`] fields verbatim (plus the
/// row's `seq` + `ts_ms`), so the frontend `Attestation` DTO must mirror them
/// exactly (serde serializes the Rust field names verbatim over the IPC boundary).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AttestationRow {
    /// SSB sequence number (newest-first ordering key).
    pub seq: i64,
    /// Row creation wall-clock ms.
    pub ts_ms: i64,
    /// The run whose Reproducible-Run Artifact was attested.
    pub session_id: String,
    /// The in-toto subject digest: the blake3-Merkle root over the predicate
    /// (the value the git trailer carries; the offline verify recomputes it).
    pub subject_blake3: String,
    /// The G-2 bundle root the attestation chains to (the run-artifact anchor).
    pub run_artifact_root: String,
    /// Count of F-gate verdicts in the SLSA-VSA `verified_levels` rollup.
    pub verified_levels: u32,
    /// `true` iff every present verified-level passed (every present manifest
    /// entry IS a passed gate; gate-off leaves no entry).
    pub all_passed: bool,
    /// The honest signing-model label (default `local_blake3_merkle`).
    pub signing_model: String,
}

/// G-9 — DTO for [`SsbViewer::redactions`]. One row per `secret_redacted`
/// advisory: a write-time secret+PII redaction fired on a typed SsbPayload
/// free-text field. Carries the redacted field identity + the stable
/// blake3-keyed token(s) + the match count + the per-kind `(label, count)`
/// summary — NEVER the raw secret. Serde field names are mirrored verbatim by
/// the frontend `RedactionStat` DTO.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct RedactionStatRow {
    /// SSB sequence number (newest-first ordering key).
    pub seq: i64,
    /// Row creation wall-clock ms.
    pub ts_ms: i64,
    /// The run whose write the redaction fired on.
    pub session_id: String,
    /// The redacted SsbPayload field identity (e.g. `idea_framing_started.prompt`).
    pub field: String,
    /// The stable blake3-keyed redaction token(s) that replaced the match(es).
    pub tokens: Vec<String>,
    /// Total number of redactions that fired on the write.
    pub match_count: u32,
    /// Sorted `(kind_label, count)` summary across the hits (no secret value).
    pub kind_counts: Vec<(String, u32)>,
}

/// G-7 — DTO for [`SsbViewer::claim_confidences`]. One row per live per-change
/// conformal decision. Mirrors the
/// [`crate::ssb::ClaimConfidenceScoredPayload`] fields verbatim (plus the row's
/// `seq` + `ts_ms`), so the frontend `ClaimConfidence` DTO must mirror them
/// exactly (serde serializes the Rust field names verbatim over the IPC
/// boundary).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClaimConfidenceRow {
    /// SSB sequence number (newest-first ordering key).
    pub seq: i64,
    /// Row creation wall-clock ms.
    pub ts_ms: i64,
    /// Synthetic per-claim id the decision was keyed on.
    pub claim_id: String,
    /// The calibrated fused confidence in `[0.0, 1.0]` fed to `decide()`.
    pub claim_score: f64,
    /// (a) Calibrated log-prob component. `None` on the cloud lane (GD-5).
    pub logprob_component: Option<f64>,
    /// (b) Self-consistency agreement component in `[0.0, 1.0]`.
    pub self_consistency_component: f64,
    /// (c) Oracle-free test execution-agreement component in `[0.0, 1.0]`.
    pub exec_agreement_component: f64,
    /// The lane the score was computed on (`local` | `cloud`).
    pub lane: String,
    /// The conformal `tau` threshold the decision used.
    pub tau: f64,
    /// The `decide()` verdict wire label (`abstain` | `answer`).
    pub conformal_verdict: String,
    /// Whether the decision abstained.
    pub abstained: bool,
}

/// G-7 — DTO for [`SsbViewer::trifecta_quarantines`]. One row per lethal-trifecta
/// quarantine. Mirrors the [`crate::ssb::TrifectaQuarantinedPayload`] fields
/// verbatim (plus the row's `seq` + `ts_ms`); NEVER the raw secret or untrusted
/// span. The frontend `TrifectaQuarantine` DTO must mirror them exactly.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrifectaQuarantineRow {
    /// SSB sequence number (newest-first ordering key).
    pub seq: i64,
    /// Row creation wall-clock ms.
    pub ts_ms: i64,
    /// Synthetic task id the quarantine fired on.
    pub task_id: String,
    /// blake3 prefix of the untrusted origin (NEVER the raw span).
    pub untrusted_origin_blake3: String,
    /// The egress / destructive sink tool that was blocked.
    pub sink_tool: String,
    /// (a) A secret pattern matched in the live context (G-9 detector).
    pub secret_present: bool,
    /// (b) The pending span's provenance was `Untrusted` (G-1 taint).
    pub untrusted_span: bool,
    /// (c) The pending tool was an egress channel (capability gate subset).
    pub egress_available: bool,
    /// Whether the span was quarantined.
    pub quarantined: bool,
    /// The no-tool sub-LLM verdict label.
    pub sub_llm_verdict: String,
}

/// G-5 — DTO for [`SsbViewer::review_surfaces`]. One row per reviewer-cockpit
/// Diff-Surface-to-Review (DSR) score. Mirrors the
/// [`crate::ssb::ReviewSurfaceScoredPayload`] fields verbatim (plus the row's
/// `seq` + `ts_ms`), so the frontend `ReviewSurface` DTO must mirror them
/// exactly (serde serializes the Rust field names verbatim over the IPC
/// boundary).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReviewSurfaceRow {
    /// SSB sequence number (newest-first ordering key).
    pub seq: i64,
    /// Row creation wall-clock ms.
    pub ts_ms: i64,
    /// The session whose working-tree changes were scored.
    pub session_id: String,
    /// Diff-Surface-to-Review ratio in `[0.0, 1.0]`.
    pub dsr: f64,
    /// DSR denominator: total added + removed changed lines across all hunks.
    pub total_changed_lines: u32,
    /// DSR numerator: changed lines in un-vouched hunks (need full attention).
    pub attention_lines: u32,
    /// Count of vouched hunks (clean/passing deterministic signal).
    pub vouched_hunks: u32,
    /// Count of un-vouched hunks (need full attention).
    pub unvouched_hunks: u32,
    /// The deterministically resolved diff-base commit SHA the score used.
    pub base_commit: String,
    /// G-5 honesty fix: whether the working-tree diff was actually obtained.
    /// `false` ONLY on a git error (absent / not a repo / non-zero exit /
    /// unresolvable base); in that case the DSR/counts are NOT a review verdict
    /// and the cockpit renders an "unavailable" banner, NOT a clean DSR=0. `true`
    /// for both a real diff AND a genuinely empty diff (honest "no changes").
    pub diff_available: bool,
    /// G-5 honesty fix: the git error reason when `diff_available == false`;
    /// empty otherwise.
    pub diff_unavailable_reason: String,
}

/// G-3 — DTO for [`SsbViewer::blast_radius_blocks`]. One row per deterministic
/// runtime blast-radius gate block. Mirrors the
/// [`crate::ssb::BlastRadiusBlockedPayload`] fields (with `kind` flattened to
/// its snake-case `&str` label and `ts_ms` dropped in favour of the row's
/// `created_at`), so the frontend `BlastRadiusBlock` DTO must mirror them
/// exactly (serde serializes the Rust field names verbatim over the IPC
/// boundary).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlastRadiusBlockRow {
    /// SSB sequence number (newest-first ordering key).
    pub seq: i64,
    /// Row creation wall-clock ms.
    pub ts_ms: i64,
    /// Boundary-crossed label: `fs_escape` / `egress_denied`.
    pub kind: String,
    /// The original, un-expanded command/argv the agent issued.
    pub pre_expansion_command: String,
    /// The resolved/canonicalised target (FS) or rejected host+ip (egress) —
    /// what would actually have executed/connected (the `#10077` forensics fix).
    pub post_expansion_command: String,
    /// The specific resolved FS target path, or the rejected egress host.
    pub target_or_host: String,
    /// blake3 hex-prefix of the canonical worktree root (identifier, not a
    /// path). Empty for the egress arm.
    pub worktree_root_blake3: String,
    /// G-1 provenance trust label join when present; `""` for a pure-containment
    /// block.
    pub trust_label: String,
}

/// F-11 — DTO for [`SsbViewer::darwinian_admits`]. One row per candidate routed
/// through the config-DGM four-guard chain. Mirrors the
/// [`crate::ssb::DarwinianAdmitPayload`] fields (the 3-arm
/// [`crate::ssb::DarwinianAdmitKind`] is flattened to a `kind` string), so the
/// frontend `DarwinianAdmit` DTO must mirror them exactly (serde serializes the
/// Rust field names verbatim over the IPC boundary).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DarwinianAdmitRow {
    /// SSB sequence number (newest-first ordering key).
    pub seq: i64,
    /// Row creation wall-clock ms.
    pub ts_ms: i64,
    /// Admit/reject/loop-closed discriminator as a snake-case string
    /// (`admitted` / `rejected_by_guard` / `loop_closed`).
    pub kind: String,
    /// Stable id of the candidate config variant the loop evaluated.
    pub variant_id: String,
    /// Parent variant id the candidate mutated from (`None` for a root seed).
    pub parent_id: Option<String>,
    /// Real evaluation score in `[0.0, 1.0]` (the Two-Gate margin basis).
    pub alpha: f64,
    /// Firing guard on `rejected_by_guard` (`tripwire`/`trap`/`security`/
    /// `two_gate`); `None` otherwise.
    pub guard_rejected_by: Option<String>,
    /// blake3 hex-prefix (16 chars) of the candidate `config_blob`.
    pub candidate_blake3: String,
}

/// H-46 — DTO for [`SsbViewer::clade_selections`]. One row per parent selection
/// emitted from `config_archive::select_parent_with_cmp`. Mirrors the
/// [`crate::ssb::CladeSelectedPayload`] fields (the 2-arm
/// [`crate::ssb::CladeSelectedKind`] is flattened to a `kind` string), so the
/// frontend `CladeSelection` DTO must mirror them exactly (serde serializes the
/// Rust field names verbatim over the IPC boundary).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CladeSelectionRow {
    /// SSB sequence number (newest-first ordering key).
    pub seq: i64,
    /// Row creation wall-clock ms.
    pub ts_ms: i64,
    /// Selection key discriminator as a snake-case string
    /// (`"clade_metaproductivity"` or `"immediate_score_fallback"`).
    pub kind: String,
    /// Stable id of the selected parent variant.
    pub node_id: String,
    /// Clade-Metaproductivity aggregate (mean execution-grounded delta of
    /// direct descendants). Equals `immediate_score` on fallback.
    pub cmp: f64,
    /// Count of descendants that contributed a measured delta to `cmp`. Zero
    /// on `ImmediateScoreFallback`.
    pub clade_size: usize,
    /// Immediate sigmoid score of the selected node's own `alpha`.
    pub immediate_score: f64,
    /// Snake_case wire string of the selection key used.
    pub key_used: String,
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
    use crate::ssb::{
        AntiSpiralHitPayload, DkgReadyPayload, InterventionKeyword, Ssb, SsbConfig,
    };

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

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn latest_anti_spiral_hit_returns_most_recent_row() {
        let dir = TempDir::new().expect("tempdir");
        let config = test_config(&dir);
        let db_path = config.db_path.clone();
        let ssb = Ssb::new(config, test_clock()).expect("ssb init");

        let first = SsbPayload::AntiSpiralHit(AntiSpiralHitPayload {
            agent_id: "executor".into(),
            step_hash: "abc123".into(),
            count: 1,
        });
        let second = SsbPayload::AntiSpiralHit(AntiSpiralHitPayload {
            agent_id: "executor".into(),
            step_hash: "def456".into(),
            count: 3,
        });
        ssb.write_payload("agent-0", 0, first, vec![])
            .await
            .expect("write first anti_spiral_hit");
        ssb.write_payload("agent-0", 0, second, vec![])
            .await
            .expect("write second anti_spiral_hit");
        drop(ssb);

        let viewer = SsbViewer::open(&db_path).expect("viewer open");
        let hit = viewer
            .latest_anti_spiral_hit()
            .expect("viewer query")
            .expect("at least one row");
        assert_eq!(hit.agent_id, "executor");
        assert_eq!(hit.step_hash, "def456");
        assert_eq!(hit.count, 3);
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn latest_anti_spiral_hit_returns_none_when_empty() {
        let dir = TempDir::new().expect("tempdir");
        let config = test_config(&dir);
        let db_path = config.db_path.clone();
        let ssb = Ssb::new(config, test_clock()).expect("ssb init");
        drop(ssb);

        let viewer = SsbViewer::open(&db_path).expect("viewer open");
        let hit = viewer.latest_anti_spiral_hit().expect("viewer query");
        assert!(hit.is_none(), "no anti_spiral_hit rows yet -> None");
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn dkg_stats_surfaces_failed_files() {
        let dir = TempDir::new().expect("tempdir");
        let config = test_config(&dir);
        let db_path = config.db_path.clone();
        let ssb = Ssb::new(config, test_clock()).expect("ssb init");

        let payload = SsbPayload::DkgReady(DkgReadyPayload {
            project_id: "proj-test".into(),
            node_count: 12,
            edge_count: 19,
            languages: vec!["rust".into()],
            failed_files: 3,
        });
        ssb.write_payload("agent-0", 0, payload, vec![])
            .await
            .expect("write dkg_ready");
        drop(ssb);

        let viewer = SsbViewer::open(&db_path).expect("viewer open");
        let stats = viewer.dkg_stats().expect("dkg stats");
        assert_eq!(stats.failed_files, 3);
        assert!(stats.known);
    }

    #[test]
    fn validate_request_id_rejects_overflow() {
        assert!(validate_request_id(0).is_ok());
        assert!(validate_request_id(i64::MAX as u64).is_ok());
        assert!(validate_request_id((i64::MAX as u64) + 1).is_err());
        assert!(validate_request_id(u64::MAX).is_err());
    }

    // J-14: run_artifacts joins latest ReproPackEvent rows by session_id. This
    // test proves (a) a row with a matching TierEvaluated gets tier_effective
    // populated, and (b) a row with no matching event leaves tier_effective
    // None (the honest empty state, not a fabricated tier).
    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn run_artifacts_joins_repro_pack_tier_evaluated() {
        use crate::ssb::{ReproPackKind, ReproPackPayload, RunArtifactEmittedPayload};

        let dir = TempDir::new().expect("tempdir");
        let config = test_config(&dir);
        let db_path = config.db_path.clone();
        let ssb = Ssb::new(config, test_clock()).expect("ssb init");

        // Session with a tier verdict row.
        let run_with_tier = SsbPayload::RunArtifactEmitted(RunArtifactEmittedPayload {
            root_blake3: "abc".into(),
            guarantee_level: "verify_divergence".into(),
            gate_count: 1,
            seed: 7,
            session_id: "sess-tier".into(),
            objective_hash: None,
        });
        ssb.write_payload("agent-0", 0, run_with_tier, vec![])
            .await
            .expect("write run_artifact");
        let tier_evt = SsbPayload::ReproPackEvent(ReproPackPayload {
            kind: ReproPackKind::TierEvaluated,
            tier_effective: Some("action_faithful".into()),
            tier_requested: Some("token_faithful".into()),
            downgrades: vec!["token_faithful->action_faithful:no_sampling_seed".into()],
            divergence_position: None,
            expected_value: None,
            actual_value: None,
            template_hash: None,
            expected_sequence_hash: None,
            actual_sequence_hash: None,
            fixture_path: None,
            fixture_blake3: None,
            session_id: "sess-tier".into(),
            ts_ms: 1,
        });
        ssb.write_payload("agent-0", 0, tier_evt, vec![])
            .await
            .expect("write tier_evaluated");

        // Session with NO ReproPackEvent row.
        let run_no_tier = SsbPayload::RunArtifactEmitted(RunArtifactEmittedPayload {
            root_blake3: "def".into(),
            guarantee_level: "reproducible_action_stream".into(),
            gate_count: 2,
            seed: 9,
            session_id: "sess-legacy".into(),
            objective_hash: None,
        });
        ssb.write_payload("agent-0", 0, run_no_tier, vec![])
            .await
            .expect("write run_artifact no-tier");
        drop(ssb);

        let viewer = SsbViewer::open(&db_path).expect("viewer open");
        let rows = viewer.run_artifacts(100).expect("run_artifacts");
        // Ordering is seq DESC, so the no-tier row is newest (written last).
        let no_tier = rows
            .iter()
            .find(|r| r.session_id == "sess-legacy")
            .expect("sess-legacy row present");
        assert!(
            no_tier.tier_effective.is_none(),
            "no ReproPackEvent row -> tier_effective stays None (honest empty)"
        );
        assert!(
            no_tier.downgrades.is_empty(),
            "no ReproPackEvent row -> downgrades stays empty"
        );

        let with_tier = rows
            .iter()
            .find(|r| r.session_id == "sess-tier")
            .expect("sess-tier row present");
        assert_eq!(
            with_tier.tier_effective.as_deref(),
            Some("action_faithful"),
            "matching TierEvaluated row populates tier_effective"
        );
        assert_eq!(
            with_tier.tier_requested.as_deref(),
            Some("token_faithful"),
            "matching TierEvaluated row populates tier_requested"
        );
        assert_eq!(
            with_tier.downgrades.len(),
            1,
            "matching TierEvaluated row populates downgrades"
        );
        assert!(
            with_tier.divergence_position.is_none(),
            "no FirstDivergenceLocated row -> divergence_position stays None"
        );
    }
}
