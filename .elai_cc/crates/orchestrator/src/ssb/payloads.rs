#![allow(unused_imports)]
// SSB typed payloads: *Payload structs, *Kind enums, SsbPayload, env-gate consts.
// Tier-B Move 1 split from ssb.rs (behavior-preserving code movement, df18539).

use std::collections::BTreeMap;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::{Duration, UNIX_EPOCH};

use rusqlite::types::Value;
use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};
use std::rc::Rc;
use tokio::sync::{broadcast, mpsc, oneshot};

use elai_core::clock::ClockProvider;
use elai_reasoning::chat_reflect::{ReflectEvent, ReflectEventSink};

// E-25 (SCHEMA_VERSION 172) -- claude-code-architecture catalog payload
// type used in `SsbPayload::ClaudeCodeArchitectureEvent`. The catalog enum
// + kind discriminator are imported inside the `tests` module where the
// sample-payload constructors live.
use crate::architecture::claude_code_arch::ClaudeCodeArchitectureEventPayload;

// E-29 Tier-B (SCHEMA_VERSION 173) -- agentic-observability event payload
// type used in `SsbPayload::AgenticObservabilityEvent`. The payload + 4-
// arm kind discriminator live inline below (`AgenticObservabilityEventPayload`
// + `AgenticObservabilityEventKind`) since the observability subtree
// (crate::observability) consumes the payload to build the row but does
// not own the wire-format definition. Reuses crate::mast::MastMode so
// the MastFailureClassified arm carries the canonical 14-variant tag
// without duplicating the taxonomy.
use crate::mast::MastMode;

use super::*;
use crate::ssb::{schema::*, action_type::*, store::*};

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
    /// F-4 / Slice B-1 (SCHEMA_VERSION 197): blake3 hex-prefix (first 16
    /// chars) of the captured stdout from the test run. `None` when no
    /// stdout was captured (pre-B1 rows, non-output-capture callers).
    /// Backward-compatible: `#[serde(default)]` means absent field
    /// deserializes as `None` for all pre-197 rows.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub stdout_hash: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FindingPayload {
    pub severity: String,
    pub category: String,
    pub message: String,
    pub file_path: Option<String>,
    pub line: Option<u32>,
    /// Batch 2B: taxonomy label attached by `dispatch_classify_failure`.
    /// Backward-compatible: unset → behaves exactly like pre-2B findings.
    /// The dashboard reads this via the new `FailureClassified` SSB row —
    /// this field is an in-Finding convenience so a single row carries
    /// both the pipeline emit-site AND the classification when we have it.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub failure_category: Option<String>,
    /// Batch 4A (A-41): which deterministic audit step produced this Finding.
    /// One of `"build" | "version_gate" | "file_size" | "lazy_code" | "completeness"`.
    /// `None` for findings emitted from non-audit sources (backward compatible).
    /// The dashboard labels audit findings by step when present.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub audit_step: Option<String>,
    /// Batch 4A (A-36): DKG few-shot exemplars attached to a Finding.
    /// Populated by Tier 3 PI when DKG similarity yields k<=3 nodes.
    /// `None` when DKG is empty or disabled (backward compatible).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub few_shot_examples: Option<Vec<FewShotExample>>,
}

/// Batch 4A (A-36): a single few-shot exemplar pulled from DKG similarity.
///
/// Each exemplar is a short in-project reference the dashboard can render under
/// the Finding as a collapsible sub-block (max 3 per Finding, 2 lines each).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FewShotExample {
    pub symbol_name: String,
    pub file_path: String,
    /// 1-based line number inside `file_path`.
    pub line: u32,
    /// Short snippet (<=2 lines) taken from the DKG node signature.
    pub snippet: String,
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

/// B-10 (DECISION-C) — per-dispatch ContextBudget snapshot. Emitted once per
/// successful `ContextBudget::compute_with_profile` call so the dashboard
/// per-slot bar chart can render real allocation values matching the active
/// model tier (e.g. 12.5/66.7/12.5/8.3 weighting for 7B-12GB).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ContextBudgetComputedPayload {
    /// Tier label (e.g. "micro_7b_12gb", "micro_3b_12gb", "large_32b_48gb",
    /// "micro_8gb"). Sourced from the
    /// `ModelCapabilityProfile.model_name` so the dashboard can group by tier.
    pub tier: String,
    /// Raw model_max_context fed into `compute_with_profile` before
    /// effective_ceiling clamping (caller's pre-tier value).
    pub raw_max: usize,
    /// Per-slot weights snapshot (8 slots, scaled to 0..=10000 basis points).
    pub slots: Vec<u64>,
    /// Per-slot allocated_tokens after compute_with_profile (8 entries).
    pub allocations: Vec<usize>,
    /// Number of adaptive-reduction levels fired (0 = Normal, up to 4 = Terminal).
    pub adapt_levels_fired: u8,
}

/// B-10 / 3E.5 (DECISION-C) — SSB entry filtering summary emitted whenever
/// the SsbEntries slot truncation drops candidates scoring below
/// [`crate::context_budget::FILE_OVERLAP_THRESHOLD`].
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SsbFilteredByOverlapPayload {
    pub entries_total: usize,
    pub entries_kept: usize,
    pub overlap_threshold: f32,
}

/// B-19A (T10) — hook latency budget violation. Emitted by the dispatcher
/// whenever a T1/T2/T3 hook tier overruns its budget
/// (T1 <= 1ms, T2 <= 10ms, T3 <= 5ms). Used by the dashboard to surface
/// rule-engine slowness on the activity feed. The dispatcher emits at most
/// one of these per tier per dispatch; an absent row means the tier ran
/// within budget.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HookLatencyViolationPayload {
    /// Hook point name — `"PreToolUse"`, `"PostToolUse"`, `"Stop"`,
    /// or `"PreCompact"`.
    pub hook_name: String,
    /// Tier label — `"T1"`, `"T2"`, `"T3"`, or `"Composite"`.
    pub tier: String,
    /// The configured budget in milliseconds for this tier.
    pub budget_ms: u64,
    /// The measured wall-clock latency in milliseconds.
    pub actual_ms: u64,
}

/// B-19 — per-agent-execution DAIRA marker aggregation row. Emitted ONCE per
/// agent-execution boundary (currently: task_runner Stop-hook site) so the
/// dashboard's ActivityFeed can render a single summary instead of replaying
/// the raw per-event `DairaTrace` stream. Carries a 15-entry count map keyed
/// on [`DAIRA_MARKERS`] phase strings (entries with zero count omitted at
/// write time to keep the row compact).
///
/// Storage rationale (per B-19 decisions DECISION-MARKER-REPORT):
///   * `phase_counts` is a `BTreeMap<String, u32>` so msgpack serialisation is
///     deterministic for the bijection test + golden lock — `HashMap` would
///     produce nondeterministic key ordering and break the round-trip.
///   * `phase_counts` keys MUST be drawn from `DAIRA_MARKERS`; the emit helper
///     `crate::hook_dispatcher::emit_marker_report_to_ssb` asserts this in a
///     `debug_assert!` and silently drops unknown phases in release builds
///     rather than corrupting the SSB stream.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MarkerReportPayload {
    /// Agent identifier (e.g. `"task_runner"`, `"executor"`, `"auditor"`).
    pub agent_id: String,
    /// Step / task identifier the aggregation window spans (e.g.
    /// `"task_42"`). Mirrors `DairaTracePayload::step_id`.
    pub step_id: String,
    /// Phase-keyed count map. Keys are entries from [`DAIRA_MARKERS`]
    /// (e.g. `"HookPre"`, `"ToolCallStart"`); values are the count of
    /// matching `DairaTracePayload` rows in this aggregation window.
    /// `BTreeMap` (not `FxHashMap`) per B-15 per-site judgement —
    /// serialisation order MUST be deterministic for the bijection.
    pub phase_counts: std::collections::BTreeMap<String, u32>,
    /// Total marker events observed in this window (sum of `phase_counts`
    /// values plus any unknown-phase rows). Lets the dashboard render
    /// "42 markers · 3 errors" without re-summing the map client-side.
    pub total_events: u32,
    /// Wall-clock window in microseconds the aggregation covers. `0` when
    /// the producer did not measure (e.g. PreToolUse-only window).
    pub window_us: u64,
}

// B-25 (SCHEMA_VERSION 102) — Phase 4J Empire integration payloads. All five
// payloads are flat serde structs (no nested enums) so msgpack serialisation
// is deterministic + the dashboard's TS-side mirror is trivially
// hand-authored. Producer: empire::bridge::EmpireBridge::emit_action when
// EMPIRE-PENDING activation lands; Consumer: dashboard EmpireBridgePanel.

/// `empire_connected` row payload. Emitted once per successful factory-brain
/// handshake.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EmpireConnectedPayload {
    pub brain_id: String,
    pub protocol_version: u8,
}

/// `empire_disconnected` row payload. `reason` is free-form snake_case
/// (e.g. `"heartbeat_timeout"`, `"protocol_error"`, `"explicit_shutdown"`)
/// so the dashboard can group disconnects without parsing structured JSON.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EmpireDisconnectedPayload {
    pub brain_id: String,
    pub reason: String,
}

/// `empire_event_received` row payload. `source` is the originating bus
/// (e.g. `"factory"` / `"engine"`); `event_type` is the Empire-side event
/// label; `payload` is opaque JSON the bridge does NOT inspect (forwarded
/// verbatim).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EmpireEventReceivedPayload {
    pub source: String,
    pub event_type: String,
    pub payload: serde_json::Value,
}

/// `empire_capability_invoked` row payload. `allowed=true` iff the
/// ElaiNeuron `ToolFilter` permitted the call. `allowed=false` rows form
/// the security audit trail.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EmpireCapabilityInvokedPayload {
    pub capability: String,
    pub caller: String,
    pub allowed: bool,
}

/// `empire_repair_proposed` row payload. `issue` is the classifier label
/// (`circular_dep` / `hard_import` / `stdlib_shadow` / other); `fix_kind`
/// is a single-token summary of the proposed remediation.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EmpireRepairProposedPayload {
    pub issue: String,
    pub target_path: String,
    pub fix_kind: String,
}

/// C-5 follow-on (SCHEMA_VERSION 103): payload for
/// [`SsbActionType::RouteBlocked`]. Emitted by the task_runner cloud
/// cap-hook when `CloudModeRouter::route_with_spend` returns
/// `DailySpendCapExceeded` or `MonthlySpendCapExceeded`.
///
/// `cap_period` is `"daily"` or `"monthly"` — the dashboard filters on
/// this string to surface the right gauge animation.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RouteBlockedPayload {
    pub reason: String,
    pub spent_usd: f64,
    pub cap_usd: f64,
    pub cap_period: String,
    pub baby_step_id: u64,
    pub ts_ms: i64,
}

/// C-5 follow-on (SCHEMA_VERSION 103): payload for
/// [`SsbActionType::WarnLimitReached`]. Emitted ONCE per UTC day when
/// `day_usd` crosses `cap * warn_at_percent / 100.0`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct WarnLimitReachedPayload {
    pub spent_usd: f64,
    pub cap_usd: f64,
    pub warn_at_percent: u8,
    /// Computed `spent_usd / cap_usd * 100.0` at emit time, clamped to
    /// [0.0, 100.0]. Stored on-row so the dashboard does not have to
    /// re-derive it from spent/cap (which may have drifted by the time
    /// the row is read back).
    pub pct_of_cap: f64,
    pub baby_step_id: u64,
    pub ts_ms: i64,
}

/// C-5 follow-on (SCHEMA_VERSION 103): payload for
/// [`SsbActionType::CostEstimate`]. Emitted once per task by the cloud
/// cap-hook the first time it sees a Cloud / Auto baby step.
///
/// `heuristic_source` carries the grounding tag of the token-per-LOC
/// constant — `"ungrounded_heuristic"` today, will become
/// `"measured_<source>"` once `TOKENS_PER_LOC_UNGROUNDED_HEURISTIC` is
/// replaced by a measured value (CLAUDE.md grounded-by-default).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CostEstimatePayload {
    pub low_usd: f64,
    pub high_usd: f64,
    pub model: String,
    pub step_count: u32,
    pub heuristic_source: String,
    pub ts_ms: i64,
}

/// C-36 (SCHEMA_VERSION 104): payload for
/// [`SsbActionType::MucocoMutationApplied`]. Emitted by
/// `orchestrator::metamorphic_testing::dispatch_mucoco_mutation` (and the
/// metamorphic_runner auditor wiring that wraps it) on every MuCoCo
/// operator invocation — applied or skipped.
///
/// Fields:
///   operator       -- snake_case operator name (one of the 11 MuCoCo
///                     operators; matches `MucocoOperator::as_str`).
///   language       -- source language identifier (paper supports "python";
///                     other strings are recorded so non-applicable
///                     dispatches surface in dashboard analytics rather
///                     than being silently dropped).
///   source_hash    -- u64 hash of the input source (callers compute via
///                     std::hash::Hasher; embedded so the dashboard can
///                     correlate retries / paired tests without storing
///                     the full source).
///   applied        -- True iff the operator returned a semantic-preserving
///                     rewrite. False -> skip_reason MUST be populated.
///   skip_reason    -- None when applied=True; non-empty String when
///                     applied=False (stub_ban / paper sec S3.p3.1).
///   timestamp_ms   -- emit time milliseconds since UNIX epoch.
///
/// Paper: arXiv:2604.19086 (interface map at
/// workflow/papers/mucoco-2604.19086-interface.md).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MucocoMutationAppliedPayload {
    pub operator: String,
    pub language: String,
    pub source_hash: u64,
    pub applied: bool,
    pub skip_reason: Option<String>,
    pub timestamp_ms: i64,
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
    // E-6: provenance — workflow/reasoning/E-6-decisions.md DECISION-8.
    // All E-6 fields are #[serde(default)] so pre-E-6 rows deserialise without
    // migration. Provenance lives on the SSB payload (orchestrator-owned),
    // NOT on elai_core::MistakeEntry — mutating that struct would cascade
    // through every non-SSB caller of the elai-core API.
    //
    /// E-6 — agent role that authored this entry (snake_case AgentRole tag).
    #[serde(default)]
    pub origin_role: Option<String>,
    /// E-6 — orchestrator run / session id at write time.
    #[serde(default)]
    pub origin_run_id: Option<String>,
    /// E-6 — wire-stable MAST mode tag the originating finding was
    /// attributed to. Parse via `elai_orchestrator::mast::MastMode::from_sql_str`.
    #[serde(default)]
    pub originating_mast_mode: Option<String>,
    /// E-6 — blake3 digest of the entry's diff text. `None` for legacy rows.
    #[serde(default)]
    pub content_hash: Option<[u8; 32]>,
    /// E-6 — provisional flag. `true` for auditor-originated entries without
    /// a corroborating cross-family AdversaryVerdict. Excluded from
    /// context_pack-build by default (arXiv:2507.02778 blind-spot mitigation).
    #[serde(default)]
    pub provisional: bool,
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
    /// Track A.4: which HTTP fetcher backend served this iteration.
    /// "reqwest" | "obscura" | "" (empty = pre-patch row, default via serde).
    #[serde(default)]
    pub fetcher: String,
    // B-7: fine-grained per-stage fields. All `#[serde(default)]` so existing
    // rows (stage=null) deserialize without error. No schema version bump:
    // payload is stored as an opaque msgpack blob; DB schema unchanged.
    // stage: "formulate"|"search"|"extract"|"reflect"|"converge"|"" (empty = coarse row)
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub stage: String,
    // stage_phase: "started"|"completed"|"" (empty = coarse row)
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub stage_phase: String,
    // stage_payload: stage-specific structured data (query, n_results, etc.)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub stage_payload: Option<serde_json::Value>,
}

/// Track A.2: research synthesis path selection.
///
/// Emitted once per `phase_f_synthesize` call with the chosen synthesis path
/// ("llm" or "deterministic_fallback") and the reason for that choice.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ResearchPathChosenPayload {
    /// "llm" | "deterministic_fallback"
    pub path: String,
    /// "sentinel" | "empty_response" | "no_citation_markers" | "grounding_error" | "no_claims" | "llm"
    pub reason: String,
    /// Wall-clock ms since UNIX_EPOCH at emission time.
    pub ts_ms: u64,
}

/// Track A.6: grounding cache hit or miss event.
///
/// Emitted by `ground_or_cached` on each call so the dashboard can aggregate
/// cache efficiency per research session.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GroundingCacheEventPayload {
    /// "hit" | "miss"
    pub kind: String,
    /// First 8 hex chars of the blake3 hash of the cache key.
    pub key_fingerprint: String,
    /// ResearchLevel name: "Idea" | "Concept" | "Phase" | "Step"
    pub level: String,
    /// Microseconds elapsed from call entry to return.
    pub elapsed_us: u64,
}

/// B-4: circuit-breaker state transition for an embedding or reranker model.
/// Emitted by CrossEncoderReranker + OrtVectorModule on state changes.
/// Dashboard get_embedding_breakers queries latest row per `model`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CircuitBreakerStateChangePayload {
    /// Model identifier: "graphcodebert" | "granite-embed" | "ms-marco-l12" | "cross-encoder-l6" | "splade-v3"
    /// (E-17 SP2: corrected "ms-marco-l6" -> "cross-encoder-l6"; the on-disk
    /// file is `models/onnx/cross-encoder-l6.onnx`.)
    pub model: String,
    /// New state: "CLOSED" | "OPEN" | "HALF_OPEN"
    pub state: String,
    /// Previous state: "CLOSED" | "OPEN" | "HALF_OPEN"
    pub previous_state: String,
    /// Number of consecutive failures at transition time.
    pub failures: u32,
    /// Rolling success rate [0.0, 1.0] at transition time.
    pub success_rate: f64,
    /// Unix epoch ms of the failure that triggered this transition (None if no failure).
    pub last_failure_ts: Option<i64>,
}

// ---------------------------------------------------------------------------
// B-7 SCHEMA payload structs — deferred (v25->v26 bump coordinated with B-6/B-9)
// ---------------------------------------------------------------------------

/// B-7: one row per BubbleUpEvent pushed in handle_failure() or bubble_up_on_*_integration_fail().
/// `from_level` == `to_level` for threshold-miss events; differs on cascade triggers.
/// `action_kind` is the BubbleUpAction discriminant as a snake_case string:
///   "research_alternative" | "redecompose_step" | "replan_phase" | "replan_concept"
///   | "diagnose_and_retry" | "escalate"
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BubbleUpTypedPayload {
    pub from_level: String,
    pub to_level: String,
    pub action_kind: String,
    /// Compact context string (baby_step_id, step_id, phase_id, concept_id, or escalate context).
    pub reason: String,
    pub failure_count: u32,
    pub threshold: u32,
}

/// B-7: emitted by run_integration_checkpoint() for all 4 DecompositionLevels.
/// `level` is the DecompositionLevel discriminant: "BabyStep" | "Step" | "Phase" | "Concept".
/// This structured variant replaces the generic Finding row emitted by B-20A task_runner.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct IntegrationCheckpointSuccessPayload {
    pub level: String,
    pub node_id: u64,
    pub wiring_target: String,
    pub success: bool,
    pub failure_reason: Option<String>,
    pub duration_ms: u64,
}

/// B-7: one row per RepairAction emitted by DagRepairLoop::repair().
/// `action_kind` is "reorder" | "substitute".
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DagRepairRoundPayload {
    pub action_kind: String,
    pub round: u32,
    pub reason: String,
}

/// B-7: one row per node built during research_decompose().
/// `node_kind` is "concept" | "phase" | "step" | "baby_step".
/// `parent_id` is 0 for root concept nodes.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ConceptDecompositionNodePayload {
    pub node_kind: String,
    pub node_id: u64,
    pub parent_id: u64,
    pub name: String,
}

/// B-7 (SCHEMA_VERSION 100): per DepthFirstNavigator cursor advance.
/// Emitted once per advance() / handle_failure transition with the trailing
/// position state from TreeNavigatorState. The dashboard PlanWindow uses
/// this to highlight the current node + drift trail.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TreePositionChangedPayload {
    /// Snake-case node id (e.g. "bs-3", "step-1"); empty when navigator
    /// produced no current step (traversal exhausted).
    pub current_node: String,
    /// BabyStepId.0 visited so far (i.e. completed_set).
    pub visited: Vec<u64>,
    /// BabyStepId.0 currently blocked (waiting on dep / failed cascades).
    pub blocked: Vec<u64>,
    /// BabyStepId.0 explicitly skipped by the navigator.
    pub skipped: Vec<u64>,
}

/// B-7 (SCHEMA_VERSION 100): per FailureRecord insert in handle_failure().
/// Mirrors the FailureRecord struct shape at
/// `pipeline_tree_navigator.rs::FailureRecord` so the dashboard PlanWindow
/// detail pane can render per-baby-step retry history without any
/// additional shape negotiation.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FailureRecordedPayload {
    pub baby_step_id: u64,
    pub step_id: u64,
    pub phase_id: u64,
    pub concept_id: u64,
    pub reason: String,
    pub attempts: u32,
}

/// B-7 (SCHEMA_VERSION 100): per PackageRef verification outcome in
/// research_tree::verify_packages. Mirrors the PackageRef struct shape
/// (`research_tree.rs::PackageRef`). `verified` is the existing bool field
/// — no method-enum is fabricated (FABRICATED-SYMBOL discipline). The
/// dashboard PlanWindow renders a tick / warning badge next to each
/// PackageRef in the decision-source detail pane.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PackageRefVerifiedPayload {
    pub name: String,
    pub registry: String,
    pub version: Option<String>,
    pub verified: bool,
}

/// B-7 (SCHEMA_VERSION 100): per TechStackDecision creation site.
/// Mirrors the TechStackDecision struct shape at
/// `research_tree.rs::TechStackDecision`. `source` is the DecisionSource
/// discriminant as snake_case ("research" | "constraint" | "default").
/// Plan-literal RESEARCH | EXISTING_PROJECT | USER_SPECIFIED | FALLBACK
/// is fabricated — only the 3 actual DecisionSource variants are emitted.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TechStackDecidedPayload {
    pub component: String,
    pub chosen: String,
    pub alternatives: Vec<String>,
    /// "research" | "constraint" | "default" (snake_case of DecisionSource).
    pub source: String,
}

// ---------------------------------------------------------------------------
// B-6 payload structs
// ---------------------------------------------------------------------------

/// B-6: emitted after each EvidenceGate::check() call in the FSM grounding-sweep path.
/// `verdict` is "pass" | "block". `gate_id` is the gate's canonical snake_case name.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EvidenceGateEvaluatedPayload {
    /// Canonical snake_case gate name, e.g. "knowledge_readiness_gate".
    pub gate_id: String,
    /// "pass" or "block".
    pub verdict: String,
    /// Unix epoch ms at evaluation time.
    pub ts_ms: i64,
}

/// B-6: emitted by the supervision layer on each agent restart or kill event.
/// `event` is "restart" | "kill" | "spiral_detected".
/// `attempt` is the 1-based restart count within the current phase window.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SupervisionEventPayload {
    pub agent_id: String,
    /// "restart" | "kill" | "spiral_detected"
    pub event: String,
    /// 1-based restart count within the current phase window.
    pub attempt: u32,
}

/// B-6: one row per completed BSP superstep, emitted by the BSP engine after
/// evidence-gate passes and checkpoint is saved.
/// `snapshot_id` is "{session_id}:superstep:{n}".
/// `channels_count` is the number of active channel topics at barrier exit.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BspSuperstepPayload {
    /// "{session_id}:superstep:{n}" uniquely identifies the snapshot.
    pub snapshot_id: String,
    /// Unix epoch ms at superstep completion.
    pub ts_ms: i64,
    /// Number of active channel topics at barrier exit.
    pub channels_count: u32,
}

/// B-6: agent lifecycle state change emitted by the BSP engine at spawn and death sites.
/// `state` is "spawned" | "ready" | "exited".
/// `death_class` is present only when `state == "exited"`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AgentLifecyclePayload {
    pub agent_id: String,
    /// AgentRole discriminant in snake_case (e.g. "executor", "auditor").
    pub role: String,
    /// "spawned" | "ready" | "exited"
    pub state: String,
    /// Present when state == "exited": "clean_exit" | "crash" | "timeout" | "budget_exceeded".
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub death_class: Option<String>,
}

/// B-6: anti-spiral escalation signal — the same step hash has appeared `count` times
/// within the current phase, triggering the escalation path in the SpiralTracker.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AntiSpiralHitPayload {
    pub agent_id: String,
    /// Blake3 hex hash of the step content (or step_id string).
    pub step_hash: String,
    /// Number of times this hash was seen in the current phase window.
    pub count: u32,
}

/// B-27: per-call spend ledger row.
///
/// Emitted by cloud_mode.rs once per [`crate::providers::AnthropicClient::send_message`]
/// round-trip. The row is the SOLE write side of the cloud spend ledger:
/// the dashboard's `get_cloud_spend` Tauri command reads back via
/// `Ssb::query_by_type(SsbActionType::CloudSpend)` and aggregates per-task
/// (last entry) + per-day (sum of today's entries).
///
/// Cost is computed by [`crate::cloud_mode::SpendAccumulator`] using
/// per-model named pricing constants and a dual-rate formula
/// (input_tokens × INPUT_PRICE + output_tokens × OUTPUT_PRICE), accumulated
/// across all baby-steps of the task. This matches spec D-4:303-305 and
/// supersedes B-27's original phase_count × avg_tokens approximation.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CloudSpendPayload {
    pub provider: String,
    pub model: String,
    pub input_tokens: u32,
    pub output_tokens: u32,
    pub cache_creation_tokens: u32,
    pub cache_read_tokens: u32,
    pub cost_usd: f64,
    pub task_id: String,
    pub ts_ms: i64,
}

/// B-27: one-shot escalation marker.
///
/// Emitted by cloud_mode.rs the first time a baby-step is promoted from
/// local execution to cloud — typically because the local model exhausted
/// its retry budget, or an explicit escalation rule fired. Provides an
/// audit trail for "why did this step go to the cloud?" without consumers
/// having to reconstruct the decision from per-call signals.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EscalateToCloudPayload {
    pub step_id: String,
    pub trigger_reason: String,
    /// Reasoning layer that ran locally before escalating (1-5 per
    /// cloud-mode-design.txt REASONING LAYER COLLAPSE :140-152).
    pub layer_failed: u8,
    pub ts_ms: i64,
}

// ---------------------------------------------------------------------------
// B-23 Phase 4H Group A — self-modification beacons (6 payload structs).
// Spec: docs/specs/guide/steps/phase-4H.txt + B-23 plan.
// ---------------------------------------------------------------------------

/// `data_table_grew` — Layer 1A applied a row to one of the four bounded
/// data tables (deprecated_patterns / failure_taxonomy / lazy_code_regex /
/// source_tiers). Distinct from `proposal_applied` (the action) — this row
/// records the resulting table-row growth so the dashboard can render
/// "deprecated_patterns +1 (cargo, blake2 -> blake3)".
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DataTableGrewPayload {
    pub proposal_id: String,
    pub table: String,           // snake_case table name (matches DataTable::as_str)
    pub pipeline_mode: String,   // surgical | greenfield | oracle
    pub ecosystem: Option<String>,
    pub language: Option<String>,
    pub rowcount_after: u64,
    pub ts_ms: i64,
}

/// `l3_proposal` — Layer 3 structural-change proposal emitted for HUMAN
/// review only. Self-mod NEVER applies these. The dashboard surfaces them
/// for an operator approve / reject decision.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct L3ProposalPayload {
    pub proposal_id: String,
    pub change_type: String,      // matches StructuralChangeType::as_str
    pub description: String,
    pub rationale: String,
    pub ts_ms: i64,
}

/// `convergence_stagnant` — OSC verdict (4H.7B). Latest `window` consecutive
/// ConfigVariants per `pipeline_mode` all crossed the similarity threshold.
/// Triggers exploration-bonus boost in next parent_selection.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ConvergenceStagnantPayload {
    pub pipeline_mode: String,
    pub window: u32,             // consecutive variants measured
    pub similarity: f32,         // latest similarity (>= 0.95)
    pub variant_id: String,      // latest variant id
    pub ts_ms: i64,
}

/// `config_oscillation` — OSC period-2 verdict (4H.7B). Both half-periods
/// passed the similarity threshold (sim(N,N-2) AND sim(N-1,N-3) >= 0.95).
/// Triggers AllowedConfigKey runtime-exclusion freeze for differing keys.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ConfigOscillationPayload {
    pub pipeline_mode: String,
    pub period: u32,             // currently always 2
    pub frozen_keys: Vec<String>,
    pub freeze_pipelines: u32,   // pipelines for which the freeze holds
    pub ts_ms: i64,
}

/// `regression_detected` — REG cross-pipeline AC regression report (4H.7C).
/// Distinct from `pass_to_fail` in TestOutcomeReport: REG spans the entire
/// pipeline history per AC, not a single eval.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RegressionDetectedPayload {
    pub pipeline_id: String,
    pub pipeline_mode: String,
    pub regressed_ac_indices: Vec<u32>,
    pub max_consecutive_failures: u32,
    pub ts_ms: i64,
}

/// `evolution_withheld` — EVO gate Withhold result (4H.7D). The latest N
/// variants never structurally evolved (similarity >= 0.95 across every
/// consecutive pair). Forces MSI verdict to STABILIZING for the mode.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EvolutionWithheldPayload {
    pub pipeline_mode: String,
    pub identical_streak: u32,
    pub reason: String,          // human-readable Withhold reason
    pub ts_ms: i64,
}

// ---------------------------------------------------------------------------
// B-39 chat reflect pipeline — per-turn REFLECT stage payload structs.
// Spec: B-39 plan; trait elai_reasoning::chat_reflect::ReflectEventSink.
// ---------------------------------------------------------------------------

/// B-39: TRIAGE stage verdict for a chat turn.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct ReflectTriagedPayload {
    pub turn_id: String,
    pub class: String,
}

/// B-39: single REASONER candidate emitted during best-of-N generation.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct ReflectCandidatePayload {
    pub turn_id: String,
    pub candidate_index: u32,
    pub content: String,
}

/// B-39: AUDITOR best-of-N verdict selecting the winning candidate.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct ReflectAuditVerdictPayload {
    pub turn_id: String,
    pub best_index: u32,
    pub rationale: String,
}

/// B-39: SYNTHESIZE final output after the full REFLECT pipeline completes.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct ReflectRefinedPayload {
    pub turn_id: String,
    pub total_tokens_used: u32,
    pub synthesis: String,
}

// ---------------------------------------------------------------------------
// B-24 Phase 4I Group C1: media validator payloads.
//
// Minimal structs landed in C1 so the SsbActionType <-> SsbPayload mapping
// is total. Group C4 expands each payload with the full per-step metadata
// (Tier1/Tier2 results, captured-frame paths, iteration history, ...).
// Keeping the surface narrow now means the schema bump is reversible if a
// downstream change requires renaming a field — Group C4 owns the field
// stability contract; C1 only owns the action-type names.
// ---------------------------------------------------------------------------

/// B-24: media validation started for a single media artifact. Emitted on
/// `MediaValidator::validate` entry. `step_id` ties the row to the
/// pipeline step that produced the artifact.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct MediaValidationStartedPayload {
    pub step_id: String,
    pub media_type: String,
    pub file_path: String,
}

/// B-24: Tier-1 deterministic checks complete. The full per-check
/// breakdown is added in Group C4; for now we ship `pass`+`latency_ms`
/// so the dashboard can aggregate without a payload re-serialization.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct MediaTier1CompletePayload {
    pub step_id: String,
    pub pass: bool,
    pub check_count: u32,
    pub latency_ms: u64,
}

/// B-24: Tier-2 (VLM) advisory complete.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct MediaTier2CompletePayload {
    pub step_id: String,
    pub advisory_pass: bool,
    pub model_used: String,
    pub latency_ms: u64,
}

/// B-24: render adapter (web / blender / animation / ui / comfyui)
/// finished producing a captured-frame set.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct RenderCompletePayload {
    pub step_id: String,
    pub adapter_name: String,
    pub frame_count: u32,
    pub latency_ms: u64,
}

/// B-24: one iteration of the visual-feedback loop completed.
/// `converged` is true when the issues set stabilized.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct VisualFeedbackIterationPayload {
    pub step_id: String,
    pub iteration: u32,
    pub issue_count: u32,
    pub converged: bool,
}

/// B-24: HumanReview branch — validator could not auto-decide.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct MediaHumanReviewRequiredPayload {
    pub step_id: String,
    pub reason: String,
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

/// One field in a credential request form.
///
/// `id` becomes the env-var name (uppercased, sanitized to `[A-Z0-9_]+`).
/// `secret` controls whether the dashboard renders the input as type="password".
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CredentialField {
    /// Identifier for the field; becomes the env-var name after sanitization.
    pub id: String,
    /// Human-readable label shown in the credential form.
    pub label: String,
    /// When true the dashboard renders the input as a masked password field.
    pub secret: bool,
    /// Optional pre-filled default value shown in the form.
    pub default: Option<String>,
}

/// Category of human intervention being requested (4K.1).
///
/// `RequestCredential` carries the form fields the dashboard must render.
/// Adding this variant requires removing `Copy` from the enum because
/// `Vec<CredentialField>` is not `Copy`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum InterventionType {
    ApproveChange,
    ReplanApproval,
    ErrorRecovery,
    /// C2: request the user to supply one or more credential values.
    RequestCredential { fields: Vec<CredentialField> },
}

/// One option presented to the user when an intervention is raised.
///
/// The `Retry(String)` variant carries a freeform message; `Keyword` variants
/// are unit. `#[serde(untagged)]` renders enum-less JSON: `"accept"`,
/// `"decline"`, `"skip"`, or `{"retry": "message"}`, or
/// `{"credentials": {"KEY": "value", ...}}` — matching what the dashboard
/// frontend emits.
///
/// `Credentials` is the C2 variant: carries an `InterventionCredentials` whose
/// `credentials` map is keyed by `CredentialField.id`. The untagged representation
/// is `{"credentials": {"KEY": "VALUE"}}`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum InterventionOption {
    /// C2: credential form submission from the dashboard.
    Credentials(InterventionCredentials),
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

/// C2: structured form of the `credentials` option.
///
/// Serializes as `{"credentials": {"KEY": "VALUE", ...}}` so the untagged
/// `InterventionOption` enum can distinguish it from `Keyword` and `Retry`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct InterventionCredentials {
    pub credentials: BTreeMap<String, String>,
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

/// Payload for `chat_message` actions (Tier 1 CHAT window).
///
/// Written by the dashboard's `submit_chat_message` Tauri command after the
/// client-side `intentRouter` (or the backend `classify_intent` fallback)
/// classifies the user's input. `intent` is the routed category (TASK / FIX /
/// EXPLORE / CONFIG / STATUS / CONTINUE / RESEARCH / INTERVENE / STEER /
/// ABORT / MODE / MODEL / AMBIGUOUS). No orchestrator consumer yet — the row
/// lives purely for chat history replay (same deferral pattern as
/// `TaskSubmittedPayload`).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ChatMessagePayload {
    pub session_id: String,
    pub text: String,
    pub intent: String,
    pub project_id: String,
    pub submitted_at_ms: i64,
}

/// Payload for `chat_response` actions (Tier 1 CHAT window).
///
/// Written by the dashboard after rendering a system card back to the user
/// (StatusCard, ErrorCard, DisambiguationCard, SteeringAckCard). `card_kind`
/// is one of "status" | "error" | "disambiguation" | "steering_ack". `body`
/// is the rendered card text. Row is append-only, no ACK flow.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ChatResponsePayload {
    pub session_id: String,
    pub card_kind: String,
    pub body: String,
    pub project_id: String,
    pub emitted_at_ms: i64,
}

/// Payload for `execution_mode_changed` actions (Tier 4 Settings reorg,
/// dashboard-wiring-plan.txt §8).
///
/// Written by the dashboard `set_execution_mode` Tauri command whenever the
/// user toggles the Execution section radio (LOCAL / CLOUD / HYBRID).
/// `from_mode` and `to_mode` are the validated mode strings (the command
/// rejects any value outside the fixed set before writing). `project_id`
/// identifies the active project at the time of the change. The CLOUD and
/// HYBRID branches of the dispatch path are not yet implemented (§9 group
/// 20); the dashboard emits a follow-up `Finding` row to flag the gap when
/// the user selects one of those modes.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ExecutionModeChangedPayload {
    pub from_mode: String,
    pub to_mode: String,
    pub project_id: String,
    pub changed_at_ms: i64,
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
    /// Track A.1: machine-readable error kind for the "failed" stage
    /// (e.g. "bm25_error", "dkg_error"). `None` on non-failure stages.
    /// Backward-compatible: old rows without this field deserialize to `None`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub error_kind: Option<String>,
    /// Track A.1: full error message string for the "failed" stage.
    /// Duplicates `message` with a semantically distinct name so dashboards
    /// can branch on the typed field rather than parsing `message`.
    /// Backward-compatible: old rows without this field deserialize to `None`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub error_message: Option<String>,
}

/// B-32 (SCHEMA_VERSION 38) — payload for the
/// [`SsbActionType::MinifiedFileChunked`] one-row-per-minified-file emission.
///
/// Emitted by [`crate::indexing_service::IndexingService::build_index_resumable`]
/// after the BM25 batch loop completes, once per file whose content satisfies
/// `elai_knowledge::chunking::detect::is_minified` (average line length >
/// `MAX_AVG_LINE_LENGTH = 500` chars). Mirrors the tracing line emitted at
/// `bm25.rs:412/523` so the dashboard can surface minified files (vendored
/// bundles, generated protobuf go, minified CSS) without having to parse
/// tracing output.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MinifiedFileChunkedPayload {
    /// Path relative to the project root, e.g. `vendor/bundle.min.js`.
    /// Stable across machines (no absolute prefix).
    pub relative_path: String,
    /// Raw byte length of the file content. Persisted as `u64` to match the
    /// `metadata.indexed_files` column shape; minified bundles routinely
    /// exceed `u32::MAX` chars after concatenation.
    pub char_count: u64,
    /// Number of chunks emitted by the character-based chunker (does NOT
    /// include the zero-orphan `ChunkKind::File` chunk that bm25 always
    /// appends).
    pub chunk_count: u32,
    /// Computed `content.len() / line_count` average. The detector's
    /// threshold is `MAX_AVG_LINE_LENGTH = 500`; values below 500 will never
    /// produce this row.
    pub avg_line_length: u32,
}

/// B-33 follow-up (SCHEMA_VERSION 40) — payload for the
/// [`SsbActionType::SymbolGraphUpdated`] one-row-per-build emission.
///
/// Emitted by [`crate::indexing_service::IndexingService::build_index_resumable`]
/// after [`elai_knowledge::symbol_graph::build_pipeline::build_symbol_graph`]
/// returns Ok with `BuildSummary { files > 0, ... }`. Replaces the 30s
/// `setInterval` polling fallback that B-33 dashboard-v2 GraphWindow shipped
/// while `ssb.rs` was locked. Consumers (GraphWindow event listener,
/// KnowledgeDrawer refresh hook) drive UI invalidation on receipt.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SymbolGraphUpdatedPayload {
    /// BLAKE3-derived per-project identifier (matches
    /// `metadata::project_id_from_path`).
    pub project_id: String,
    /// Number of source files indexed by the symbol-graph build.
    /// `u32` is sufficient for any project size we ship to (>4G files
    /// would imply a multi-TB corpus).
    pub file_count: u32,
    /// Total number of `SymbolNode` rows persisted by the build.
    /// `u64` matches the `metadata.symbol_count` column shape.
    pub symbol_count: u64,
    /// Total number of `SymbolEdge` rows persisted by the build.
    /// `u64` matches the `metadata.edge_count` column shape.
    pub edge_count: u64,
    /// Wall-clock UTC milliseconds when the build completed (sourced from
    /// the injected `ClockProvider`). Used by the dashboard for "fetched
    /// X ms ago" timestamps.
    pub built_ms: i64,
    /// C-24 (DRIFT-4): `Some(true)` when the producer used the incremental
    /// `update_changed_files_symbol_graph` pathway (i.e. a resume after a
    /// prior successful index). `Some(false)` or `None` when a full
    /// `build_symbol_graph` rebuild ran (cold start / above-threshold
    /// change set). Additive `Option<bool>` so existing rmp-serde fixtures
    /// continue to decode — same precedent as C-17 DECISION-8 for
    /// `IndexCheckpointPayload::batch_duration_ms`. No SCHEMA_VERSION bump.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub is_incremental: Option<bool>,
}

// ---------------------------------------------------------------------------
// B-40 (SCHEMA_VERSION 41) — FRAMING substate payloads
// ---------------------------------------------------------------------------

/// B-40 (SCHEMA_VERSION 41) — payload for [`SsbActionType::IdeaFramingStarted`].
///
/// Emitted once when the FRAMING substate is entered. `prompt` is the raw
/// user input (trimmed). `session_id` is provided redundantly so dashboard
/// consumers that subscribe to a global SSB stream can scope per-task without
/// reading the SsbEntry envelope. `started_at_ms` is the wall-clock unix-ms
/// timestamp sourced from the injected [`elai_core::ClockProvider`].
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct IdeaFramingStartedPayload {
    pub session_id: String,
    pub prompt: String,
    pub started_at_ms: i64,
}

/// B-40 (SCHEMA_VERSION 41) — payload for [`SsbActionType::ProjectClassClassified`].
///
/// `project_class` is the snake_case discriminant of
/// [`crate::idea_framing::ProjectClass`] (one of 9 variants). `matched_pattern`
/// records the regex / keyword that fired in the classifier; `None` when the
/// fallback `Other` arm matched.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProjectClassClassifiedPayload {
    pub session_id: String,
    pub project_class: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub matched_pattern: Option<String>,
}

/// B-40 (SCHEMA_VERSION 41) — payload for [`SsbActionType::CriticRoundComplete`].
///
/// Emitted at the end of each [`crate::idea_framing::run_critic_loop`] round.
/// `round_index` is 1-based (rounds 1..=3). `category_counts` maps each of the
/// 10 [`crate::idea_framing::CriticCategory`] discriminants (snake_case) to
/// the number of findings emitted in that round. `block_count` / `warn_count` /
/// `info_count` are the per-severity totals. `converged` is true when
/// `block_count == 0` (loop exits early).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CriticRoundCompletePayload {
    pub session_id: String,
    pub round_index: u32,
    pub block_count: u32,
    pub warn_count: u32,
    pub info_count: u32,
    pub converged: bool,
    /// snake_case CriticCategory name → finding count for that round.
    pub category_counts: Vec<(String, u32)>,
}

/// B-40 (SCHEMA_VERSION 41) — payload for [`SsbActionType::IdeaFramed`].
///
/// Emitted by [`crate::idea_framing::frame_idea`] after framed-idea.toml is
/// persisted. `artifact_path` is the absolute path
/// `workflow/sessions/{sid}/idea-framing/framed-idea.toml`. `rounds_used` is
/// the count of critic rounds actually executed (1..=3). `tradeoff_count` is
/// the number of tradeoffs on the TradeoffCard. Consumed by
/// [`crate::task_runner::derive_framing_evidence`] (the 7th
/// AutoAdvanceEvaluator check, gated on FRAMING substate).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct IdeaFramedPayload {
    pub session_id: String,
    pub artifact_path: String,
    pub project_class: String,
    pub rounds_used: u32,
    pub tradeoff_count: u32,
    /// True when the critic loop converged (zero BLOCK findings) within
    /// `rounds_used` rounds. False indicates an idea_framing_block
    /// condition (FRAMING substate refuses to advance).
    pub converged: bool,
}

// ---------------------------------------------------------------------------
// B-34 (SCHEMA_VERSION 42) payload structs
// ---------------------------------------------------------------------------

/// B-34 — payload for [`SsbActionType::EntryPointsDetected`].
///
/// Emitted by the `entry_points` dispatcher arm after `detect_entry_points`
/// returns. Lets the dashboard render an entry-point summary card without
/// re-running detection.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EntryPointsDetectedPayload {
    pub project_id: String,
    /// Total detected entry points (including tests if include_tests=true).
    pub count: u32,
    /// Per-reason breakdown (e.g. "orphan" -> 3, "well_known:main" -> 2).
    pub by_reason: std::collections::BTreeMap<String, u32>,
}

/// B-34 — payload for [`SsbActionType::ArtifactsIndexed`].
///
/// Emitted by the Phase C artifact indexer wiring after a complete
/// (re)indexing run finishes. Carries the partition of artifacts that were
/// freshly indexed vs. already up-to-date.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ArtifactsIndexedPayload {
    pub project_id: String,
    /// Artifact names freshly (re)indexed in this run.
    pub indexed: Vec<String>,
    /// Artifact names whose content hash matched — skipped as up-to-date.
    pub up_to_date: Vec<String>,
    /// Total chunk count across all indexed artifacts after this run.
    pub total_chunks: usize,
    /// Wall-clock time for the full indexing run in milliseconds.
    pub latency_ms: u64,
}

/// B-34 — payload for [`SsbActionType::ArtifactSearched`].
///
/// Emitted by the `search_artifacts` dispatcher arm after each query.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ArtifactSearchedPayload {
    pub project_id: String,
    /// Optional artifact name filter applied to the query (None = all).
    pub artifact_name: Option<String>,
    /// The natural-language query string.
    pub query: String,
    /// Number of results returned (after limit clamping).
    pub result_count: usize,
    /// Wall-clock query latency in milliseconds.
    pub latency_ms: u64,
}

/// B-34 — payload for [`SsbActionType::ArtifactStale`].
///
/// Emitted when the ArtifactIndexer detects that an artifact's content has
/// changed (blake3 hash mismatch). Triggers a dashboard staleness badge.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ArtifactStalePayload {
    pub project_id: String,
    pub artifact_name: String,
    /// Repo-relative paths of files whose content hash changed.
    pub changed_files: Vec<String>,
}

/// B-40 (SCHEMA_VERSION 41) — payload for [`SsbActionType::TradeoffCardEmitted`].
///
/// Emitted by [`crate::idea_framing::frame_idea`] when the TradeoffCard is
/// surfaced to the dashboard. `tradeoff_names` is the ordered list of
/// `Tradeoff.name` strings; `option_counts` is the parallel ordered list of
/// per-tradeoff option-list lengths (for dashboard validation that responses
/// reference valid options).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TradeoffCardEmittedPayload {
    pub session_id: String,
    pub tradeoff_names: Vec<String>,
    pub option_counts: Vec<u32>,
}

/// B-31.5 (SCHEMA_VERSION 39) — payload for [`SsbActionType::IndexCheckpoint`].
///
/// Emitted by [`crate::indexing_service::IndexingService::build_index_resumable`]
/// after each successful batch is persisted via `ProjectMetadataStore::save`.
/// `batches_total` is `None` when the work queue was empty up-front (no
/// candidates) or could not be sized cheaply.
///
/// C-17 (DRIFT-I): `batch_duration_ms` carries the per-batch wall-clock
/// elapsed in milliseconds, sourced from an OsClock-backed `Instant::now()`
/// taken at batch entry. Added as `Option<u64>` so older readers tolerate
/// payloads emitted without the field (back-compat under rmp-serde's named
/// encoding — no SCHEMA_VERSION bump required for the additive optional
/// field). Consumers parse p50/p99 across recent rows for the scale TOMLs.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct IndexCheckpointPayload {
    /// BLAKE3-derived per-project identifier (matches
    /// `metadata::project_id_from_path`).
    pub project_id: String,
    /// Number of completed batches at the time of this checkpoint row.
    pub batches_processed: u32,
    /// Total batch count when computable. `None` when `total_files == 0`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub batches_total: Option<u32>,
    /// Cumulative count of files actually indexed (i.e. handed to
    /// `Bm25Index::index_file`) since the run started. Skipped/resumed files
    /// are NOT counted here.
    pub files_processed: u64,
    /// Unix-seconds timestamp of the checkpoint, sourced from the injected
    /// [`elai_core::ClockProvider`].
    pub last_checkpoint_at: i64,
    /// C-17 (DRIFT-I): wall-clock duration of the batch that just completed,
    /// in milliseconds, measured via `clock.instant_now()`. Optional for
    /// back-compat with pre-C-17 emitters; readers that need real p50/p99
    /// timing skip rows where this is `None`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub batch_duration_ms: Option<u64>,
}

/// B-31.5 (SCHEMA_VERSION 39) — payload for [`SsbActionType::IndexResume`].
///
/// Emitted in two contexts:
/// * `build_index_resumable` returning [`crate::indexing_service::IndexResult::Resumed`].
/// * `auto_resume_indexed_projects` after a resumed run completes.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct IndexResumePayload {
    /// BLAKE3-derived per-project identifier.
    pub project_id: String,
    /// Value of `batches_processed` at the moment the run resumed.
    pub resumed_from_batch: u32,
    /// Number of candidate files whose stored BLAKE3 hash matched the
    /// on-disk content and were therefore skipped.
    pub files_skipped: u64,
}

/// B-31.5 (SCHEMA_VERSION 39) — payload for [`SsbActionType::IndexCancel`].
///
/// Emitted when the in-process cancellation flag is observed by
/// `build_index_resumable`, OR when `graceful_shutdown` signals a project
/// during the SHUTTING_DOWN cancellation pass.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct IndexCancelPayload {
    /// BLAKE3-derived per-project identifier.
    pub project_id: String,
    /// Number of completed + checkpointed batches at abort time.
    pub batches_completed: u32,
    /// Cumulative count of files indexed before the abort.
    pub files_indexed: u64,
}

/// B-31.5 (SCHEMA_VERSION 39) — payload for [`SsbActionType::LockAcquired`].
///
/// Emitted by `IndexLock::try_acquire` when the process-global SSB writer
/// has been installed via `lock::install_ssb_writer`. The `scope` field
/// uses the same lowercase string as `LockScope::as_str` (`"index"` or
/// `"watch"`).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LockAcquiredPayload {
    /// BLAKE3-derived per-project identifier.
    pub project_id: String,
    /// Lockscope discriminant string — `"index"` or `"watch"`.
    pub scope: String,
    /// OS process ID of the holder (== this process at acquire time).
    pub holder_pid: u32,
}

/// B-31.5 (SCHEMA_VERSION 39) — payload for [`SsbActionType::LockContested`].
///
/// Emitted on
/// [`crate::lock::IndexLockError::AlreadyHeld`] when the process-global
/// SSB writer is installed. `holder_pid` is the LIVE holder (not the
/// contender — the contender is the process emitting the row).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LockContestedPayload {
    /// BLAKE3-derived per-project identifier.
    pub project_id: String,
    /// Lockscope discriminant string — `"index"` or `"watch"`.
    pub scope: String,
    /// OS process ID of the live holder that blocked acquisition.
    pub holder_pid: u32,
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
    /// B-4: count of files that failed to parse during the DKG build.
    /// Aligns producer with the dashboard consumer (commands.rs:2275 reads
    /// `failed_files` from this payload). `#[serde(default)]` ensures
    /// existing rows without this field deserialize cleanly as 0.
    #[serde(default)]
    pub failed_files: u64,
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
// Batch 2B — ToolDispatcher payloads
// ---------------------------------------------------------------------------

/// Batch 2B — per-dispatch summary row. Emitted once per call to each of
/// the six `ToolDispatcher` methods.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ToolDispatchPayload {
    /// One of `"edit" | "validate" | "classify_failure" | "git_commit"
    /// | "file_read_outline" | "incremental_compile"`.
    pub tool: String,
    /// Short ASCII summary of the arguments (no binary bytes). E.g.
    /// `"path=src/lib.rs tier=exact"` for an edit row.
    pub args_summary: String,
    pub elapsed_ms: u32,
    pub step_id: String,
    /// `"success"` / `"error"` / `"skipped"` / `"not_implemented"`.
    pub outcome: String,
    /// Populated when `outcome == "error"` AND the classifier recognised
    /// the failure category.
    pub error_category: Option<String>,
}

/// Batch 2B — per-tier validation verdict.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ValidationTierPayload {
    /// One of `"determinism" | "quality_ast" | "unit" | "integration" | "stress"`.
    pub tier: String,
    pub passed: bool,
    pub elapsed_ms: u32,
    pub rules_fired: u32,
    pub step_id: String,
}

/// Batch 2B — failure classification emitted by `dispatch_classify_failure`.
/// Confidence is stored as u16 in basis points (0..=10000) to keep the
/// payload `Eq` for msgpack-serialised equality tests.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FailureClassifiedPayload {
    pub category: String,
    /// Confidence × 10000 (basis points). 10000 = 1.0, 7000 = 0.7.
    pub confidence_bp: u16,
    pub step_id: String,
    /// One of `"agent" | "validator" | "deterministic_audit"`.
    pub source: String,
}

/// Batch 2B — real git2 commit boundary row.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GitCommitPayload {
    pub commit_sha: String,
    pub branch: String,
    pub files_count: u32,
    pub step_id: String,
    pub message: String,
}

/// B-19 — DAIRA 15-marker vocabulary for the `phase` field of
/// [`DairaTracePayload`]. Kept as a `&str` slice (not an enum) so existing
/// call sites that emit legacy values ("enter"/"exit") continue to compile
/// without modification while the frontend and SSB viewer can validate
/// against this canonical list.
///
/// Marker semantics:
/// - `Call`          — function entry (legacy: "enter")
/// - `Return`        — function exit clean (legacy: "exit" / "success")
/// - `Mutation`      — state-changing side effect (e.g. file write)
/// - `Branch`        — conditional path taken
/// - `Error`         — error bubble (legacy: "error")
/// - `Finding`       — auditor BLOCK emit
/// - `Read`          — file / db read
/// - `Write`         — file / db write
/// - `ToolCallStart` — tool dispatch entry
/// - `ToolCallEnd`   — tool dispatch exit
/// - `HookPre`       — hook point firing before action
/// - `HookPost`      — hook point firing after action
/// - `Timeout`       — operation exceeded latency budget
/// - `Escalate`      — escalation signal needed
/// - `Converge`      — spiral convergence / resolution
pub const DAIRA_MARKERS: &[&str] = &[
    "Call",
    "Return",
    "Mutation",
    "Branch",
    "Error",
    "Finding",
    "Read",
    "Write",
    "ToolCallStart",
    "ToolCallEnd",
    "HookPre",
    "HookPost",
    "Timeout",
    "Escalate",
    "Converge",
];

/// Batch 2B — DAIRA call-graph breadcrumb. Emitted on entry AND exit of
/// every dispatch method so the frontend can assemble a call graph keyed
/// on `(phase, tool)` without having to deserialise the per-dispatch row.
///
/// B-19: `phase` now accepts the 15-marker DAIRA vocabulary defined in
/// [`DAIRA_MARKERS`] in addition to the legacy "enter"/"exit" strings.
/// Legacy values remain valid and are NOT rejected — this is a superset
/// expansion, not a breaking change.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DairaTracePayload {
    /// DAIRA phase marker — see [`DAIRA_MARKERS`] for the canonical 15-value
    /// vocabulary. Legacy values `"enter"` / `"exit"` remain accepted.
    pub phase: String,
    /// Tool name — same tag set as [`ToolDispatchPayload::tool`].
    pub tool: String,
    /// Blake3 short hash (8 bytes hex) of the args summary. Lets the
    /// frontend correlate enter/exit pairs without needing stable IDs.
    pub args_hash: String,
    pub elapsed_us: u64,
    /// On `exit`: `"success" | "error" | "skipped" | "not_implemented"`.
    /// On `enter`: `"started"`.
    pub outcome: String,
    pub step_id: String,
}

// ---------------------------------------------------------------------------
// Batch 3A (Phase 4F) — SystemProbe / FeasibilityReport payloads
// ---------------------------------------------------------------------------

/// Batch 3A — snapshot of the [`crate::system_probe::SystemProbeResult`] at
/// task_runner startup. Emitted once per runner lifetime. `vram_*` fields are
/// `Option<u32>` so env-fallback probes with no GPU data still serialise
/// cleanly. `detection_method` / `tier` are stable string labels matching
/// `DetectionMethod::as_str()` and the `VramTierDetectedPayload` tier set.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SystemProbePayload {
    pub gpu_name: Option<String>,
    pub vram_total_mb: Option<u32>,
    pub vram_free_mb: Option<u32>,
    pub ram_total_mb: u64,
    pub ram_free_mb: u64,
    pub disk_total_mb: u64,
    pub disk_free_mb: u64,
    /// `"shell_out"` / `"env_fallback"` / `"unavailable"`.
    pub detection_method: String,
    /// `"8GB"` / `"12GB"` / `"16GB"` / `"24GB"` / `"48GB"` / `"cloud"`.
    pub tier: String,
}

/// Batch 3A — per-concept / per-task feasibility summary. Percent values are
/// encoded as basis-point u16 (0..=10000) so the payload stays `Eq`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FeasibilityReportPayload {
    /// `"feasible"` / `"marginal"` / `"infeasible"`.
    pub task_feasibility: String,
    /// Populated when the probe was skipped (e.g. `"no_novel_steps_margins_ok"`).
    pub skip_reason: Option<String>,
    pub novel_step_count: u32,
    pub escalations_count: u32,
    /// VRAM margin × 100 as u16 (bp). 2000 = 20.00%.
    pub vram_margin_bp: u16,
    pub ram_margin_bp: u16,
    pub disk_margin_bp: u16,
}

/// Batch 3A — one row per novel step detected by the feasibility probe.
/// Emitted inline with the enclosing [`FeasibilityReportPayload`].
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NovelStepDetectedPayload {
    pub step_id: String,
    pub reason: String,
    pub alternative_count: u32,
}

/// Batch 3A — one row per stub execution inside the feasibility probe.
/// Emitted per novel step + per baby-step probe invocation.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StubRunPayload {
    pub step_id: String,
    pub passed: bool,
    pub elapsed_ms: u32,
}

/// Batch 3B (Phase 4G) — one row per [`crate::autonomous_gate::AutonomousGate::decide`]
/// call. `decision` is the [`crate::autonomous_gate::AutonomyDecision`]
/// variant name (`"auto_advance"` / `"request_human"` / `"split_task"` /
/// `"substitute_model"` / `"escalate"`). `risk_level` is populated on
/// RequestHuman; `reason` on RequestHuman / Escalate.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GateDecisionPayload {
    pub decision: String,
    pub risk_level: Option<String>,
    pub reason: Option<String>,
    pub evidence_summary: String,
    pub step_id: String,
}

/// Batch 3B (Phase 4G) — task-split announcement. Emitted when the gate
/// decides the current task must be split under context pressure.
///
/// Batch FUL-B extends the payload with three optional fields describing
/// the real splitter's output (human-readable summaries, splitter kind,
/// token estimate). All three carry `#[serde(default)]` so rows serialized
/// before FUL-B still deserialize into `None` / `None` / `None`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TaskSplitPayload {
    pub original_task_id: String,
    pub sub_task_ids: Vec<String>,
    pub depth: u8,
    /// Batch FUL-B: one-line summary per sub-task, aligned positionally
    /// with `sub_task_ids`. `None` when the deterministic-fallback path
    /// fired (Batch 3B semantics, no splitter plugged).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub sub_task_summaries: Option<Vec<String>>,
    /// Batch FUL-B: name of the splitter that produced the sub-tasks.
    /// Canonical values: `"phase_sizing"` (real splitter) or
    /// `"deterministic_fallback"` (Batch 3B fallback).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub splitter: Option<String>,
    /// Batch FUL-B: token estimate that triggered the split. Populated
    /// only when `splitter = Some("phase_sizing")`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub complexity_estimate: Option<u32>,
}

/// Batch 3B (Phase 4G) — escalation trigger row. `trigger` is the
/// [`crate::autonomous_gate::EscalationTrigger::as_str`] label. `details` is
/// free-form (e.g. the AntiSpiralDetector depth).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EscalationFiredPayload {
    pub trigger: String,
    pub step_id: String,
    pub details: String,
}

/// Batch 3B (Phase 4G) — gate-mode change. Lower-cased (`"interactive"` /
/// `"autonomous"`) to match the `/mode` slash-command syntax.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ModeChangedPayload {
    pub from: String,
    pub to: String,
}

/// Batch 3C (Phase 4H) — Layer 1A proposal generated row. Emitted once per
/// `process_outcome` proposal that passes the FPR + security-sensitive gates.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ProposalGeneratedPayload {
    pub proposal_id: String,
    pub table: String,
    pub new_row_summary: String,
    pub fpr_estimate: f32,
    pub source: String,
}

/// Batch 3C (Phase 4H) — Layer 1A proposal applied row. `commit_sha` is
/// `None` when git2 wiring is absent or the commit failed (non-fatal).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProposalAppliedPayload {
    pub proposal_id: String,
    pub table: String,
    pub applied_by: String,
    pub commit_sha: Option<String>,
}

/// Batch 3C (Phase 4H) — Layer 1A proposal rejected row. Emitted both at
/// automatic rejection time (FPR/security) and from the dashboard reject
/// command.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProposalRejectedPayload {
    pub proposal_id: String,
    pub table: String,
    pub reason: String,
}

/// Batch 3C (Phase 4H) — MSI update row. Emitted on every apply. `status` is
/// one of `"IMPROVING"` / `"STABILIZING"` / `"DIVERGING"`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MsiUpdatePayload {
    pub pipeline_mode: String,
    pub msi_value: f32,
    pub status: String,
}

/// Batch 3C (Phase 4H) — config archive cap event. `dropped_id` is
/// `Some(_)` when the 51st apply bumped an old entry out; `None` otherwise.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ConfigArchivedPayload {
    pub pipeline_mode: String,
    pub archived_count: u32,
    pub dropped_id: Option<String>,
}

/// Batch 4C (A-18) — per-dispatch GenerationProfile selection.
///
/// Emitted by [`crate::profile_selector::GenerationProfileSelector`]
/// implementations whenever the task runner chooses a generation profile for
/// the next sidecar dispatch. `profile` is the [`elai_orchestrator::context_budget::GenerationProfile`]
/// `as_str` form (e.g. `"FirstAttempt"`, `"RepairRetry"`). `role` is a free-form
/// string label for the dispatch role (`"executor"`, `"auditor"`, etc.).
/// `fsm_state` is the [`crate::fsm::FsmState`] rendered via its `Display` impl.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProfileSelectedPayload {
    pub profile: String,
    pub role: String,
    pub fsm_state: String,
    pub step_id: String,
}

/// Batch 4C (A-22) — FSM transition emitted on RESEARCHING entry/exit.
///
/// Emitted by `drive_web_research_task` (and any future RESEARCHING caller) at
/// state boundary. Both `from` and `to` are the `Display`-rendered FSM state
/// names (e.g. `"IDLE"`, `"RESEARCHING"`, `"EXECUTING"`).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FsmTransitionPayload {
    pub from: String,
    pub to: String,
    pub reason: String,
}

/// Batch 4C (A-23) — micro-reflection diagnosis row.
///
/// Emitted after a baby-step failure once the sidecar-backed 500-token
/// reflection returns. `root_cause` is truncated to 100 chars by the frontend
/// for the activity feed; full text remains in the SSB for detail view.
/// `confidence` is the reflection's self-reported 0.0..=1.0 score.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MicroReflectionPayload {
    pub step_id: String,
    pub root_cause: String,
    pub proposed_fix: String,
    pub confidence: f32,
}

/// Batch 4C (A-46) — ContextTelescope collapse event.
///
/// Emitted by [`crate::telescope_caller::invoke_telescope`] each time a
/// repair-context build trims layers to fit under a token budget.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TelescopeCollapsePayload {
    pub from_tokens: u32,
    pub to_tokens: u32,
    pub layers_collapsed: u32,
    pub step_id: String,
}

/// Batch 4C (A-45) — config-load warning.
///
/// Emitted when a non-fatal config file lookup fails (e.g. missing
/// `source-tiers.toml` at dashboard startup) and the runtime falls back to a
/// safe default. `config` is a short identifier (e.g. `"source-tiers.toml"`);
/// `reason` is a short classifier (e.g. `"file_not_found"`, `"parse_error"`).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ConfigWarningPayload {
    pub config: String,
    pub reason: String,
    pub detail: String,
}

/// Batch-fu (FU.2 / A-7) — constitutional CONSTRAINTS block injection row.
///
/// Emitted by the task_runner's `build_dispatch_context` call site each time a
/// dispatch is assembled with a non-empty `ProjectProfile.constraints` list.
/// `constraint_count` is the number of bullets rendered into the block;
/// `step_id` is the baby-step identifier so the dashboard can correlate the
/// row with a specific dispatch. `project_id` is denormalised onto the payload
/// in addition to the SSB `project_id` column so subscribe-only consumers that
/// only see the payload can still filter by project.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ConstraintsInjectedPayload {
    pub project_id: String,
    pub constraint_count: u32,
    pub step_id: String,
}

/// Batch-fu (FU.5 / A-31) — intervention resolution row.
///
/// Emitted by the task_runner after a matching `InterventionResponse` arrives
/// on the SSB broadcast channel for a pending IRREVERSIBLE-confirmation
/// `InterventionRequest`. Pairs with the request by `intervention_id` so the
/// dashboard can close out a request-row once resolved.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct InterventionResolvedPayload {
    pub intervention_id: String,
    pub approved: bool,
    pub elapsed_ms: u32,
}

/// Batch-ful-a (FUL-A.1) — DkgQuerier upgrade event.
///
/// Emitted by `task_runner::drive_task` immediately after a fresh build of the
/// DKG database succeeds (`IndexingProgress=complete` + `DkgReady`). The rule
/// engine's previously-plugged `EmptyDkgQuerier` is replaced in-place with a
/// `DkgQuerierAdapter` via the `HotSwappableDkgQuerier` wrapper so every
/// subsequent rule evaluation sees the live DKG without requiring a dashboard
/// relaunch. Exactly one row per project per runner lifetime — the dashboard
/// surfaces this as "DKG querier upgraded · empty → adapter".
///
/// `from` / `to` are tags (not full type names) so the wire format remains
/// small and stable. Current values: `"empty"` (EmptyDkgQuerier) /
/// `"adapter"` (DkgQuerierAdapter). Additional tags may be added as the
/// querier hierarchy grows.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DkgQuerierSwappedPayload {
    /// Project identifier — matches the `project_id` on `TaskSubmittedPayload`.
    pub project_id: String,
    /// Short tag for the previous querier implementation (e.g. `"empty"`).
    pub from: String,
    /// Short tag for the newly-installed querier implementation (e.g.
    /// `"adapter"`).
    pub to: String,
}

/// Batch-ful-c (FUL-C / A-8) — steering vector computed payload.
///
/// Emitted by `SteeringVectorEngine::compute_vector_via_sidecar_role` after
/// a successful sidecar round-trip AND cache-file write. Carries the role
/// label, final vector dim, L2 norm, elapsed wall-clock time in ms, and the
/// cache path (stringified for JSON round-trip friendliness).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SteeringComputedPayload {
    /// Role label the vector was computed for (e.g. `"executor"`).
    pub role: String,
    /// Dimension of the resulting vector (typically = model hidden_dim).
    pub vector_dim: u32,
    /// L2 norm of the vector. Zero norm implies the contrast pair produced
    /// no difference; the helper rejects these before emitting this row so
    /// callers can treat `norm > 0` as an invariant.
    pub norm: f32,
    /// Wall-clock time elapsed in milliseconds from request send to cache
    /// write completion.
    pub elapsed_ms: u32,
    /// Absolute filesystem path of the persisted cache bin. Stringified so
    /// the msgpack payload stays platform-portable.
    pub cache_path: String,
}

/// Batch-ful-c (FUL-C / A-8) — steering vector compute failure payload.
///
/// Emitted on every non-success path: sidecar connection refused, HTTP error,
/// empty pairs rejected by the Python side, cache write failure, etc. The
/// `reason` string is a short classifier token (`"sidecar_unreachable"`,
/// `"http_error"`, `"zero_magnitude"`, `"cache_write_failed"`) suffixed with
/// the human detail where available.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SteeringComputeFailedPayload {
    pub role: String,
    pub reason: String,
}

/// Batch-ful-d (FUL-D / phase-3J S9) — MCP server connected payload.
///
/// Emitted by the dashboard `DefaultMcpBridge` after a successful JSON-RPC
/// `initialize` + `tools/list` round-trip. The `tool_count` is informational
/// only (for "mcp · connected · 7 tools" dashboard rows) and may be zero if
/// the server exposes resources but no tools.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct McpServerConnectedPayload {
    /// URL the bridge connected to (e.g. `http://localhost:8200` or
    /// `stdio://elai-mcp-stub`). Useful for the dashboard to display.
    pub server_url: String,
    /// Number of tools advertised by the server's `tools/list` response.
    pub tool_count: u32,
}

/// Batch-ful-d (FUL-D / phase-3J S9) — per-call MCP tool invocation row.
///
/// Emitted on every `tools/call` round-trip (success or error). `outcome` is
/// a short classifier token — `"success"`, `"error"`, `"security_blocked"`,
/// `"timeout"` — so the dashboard can colour the row without parsing the
/// detail string.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct McpToolCalledPayload {
    pub tool_name: String,
    pub server_url: String,
    pub elapsed_ms: u32,
    pub outcome: String,
}

/// Batch-ful-d (FUL-D / phase-3J S9) — MCP server disconnected payload.
///
/// Emitted on every disconnect path: explicit user toggle, heartbeat
/// timeout, JSON-RPC protocol error, network drop. `reason` is a short
/// classifier (`"user_disabled"`, `"heartbeat_timeout"`, `"protocol_error"`,
/// `"connection_closed"`) optionally suffixed with the human detail.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct McpServerDisconnectedPayload {
    pub server_url: String,
    pub reason: String,
}

/// B-28 E-3 (SCHEMA_VERSION 31): focus-tunnel ModeSelector decision row.
///
/// Emitted by `task_runner::drive_task` immediately before the executor
/// dispatch at the `baby_step_executor.execute(exec_ctx)` callsite. One
/// row per baby step. Stays a flat record of strings/bool (no nested enums)
/// so the dashboard payload_json bridge can render rows without importing
/// focus_tunnel symbols.
///
/// `mode`, `rule_fired`, `exec_mode` are SSB wire-form strings produced via
/// `crate::focus_tunnel::generation_mode_str` / `selection_rule_str` /
/// `exec_mode_str` so the names stay locked. The reader-side aggregation
/// helper `crate::focus_tunnel::mode_distribution_for_payloads` parses
/// `mode` back into a [`crate::focus_tunnel::GenerationMode`] enum value.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GenerationModeSelectedPayload {
    /// `"baby_step_{id}"` stringified at emit time so the dashboard can
    /// correlate this row with other beacons keyed by the same step id.
    pub step_id: String,
    /// One of `"fim"` / `"diff"` / `"structured"` (snake_case wire form
    /// matching `GenerationMode` serde).
    pub mode: String,
    /// One of `"cloud_override"` / `"create_file_override"` /
    /// `"fim_all_conditions_met"` / `"multi_region_diff"` /
    /// `"structured_fallback"` / `"hinted_by_preferred_mode"`.
    pub rule_fired: String,
    /// Static label naming the rule body (`"rule_1_cloud_execution"`, ...).
    pub reason: String,
    /// The advisory `preferred_mode` hint present on the step at decision
    /// time, if any.
    pub preferred_was: Option<String>,
    /// Snapshot of `ModelFeatures.supports_fim` at decision time. Lets the
    /// auditor verify that Rule 3 never fires when supports_fim is false.
    pub supports_fim: bool,
    /// One of `"local"` / `"cloud"` / `"auto"` per
    /// `crate::cloud_mode::ExecMode`. Distinct from the
    /// `execution_mode_changed` LOCAL/CLOUD/HYBRID dashboard surface — this
    /// is the routing-side enum, not the settings-window one.
    pub exec_mode: String,
}

/// B-28 E-4 (SCHEMA_VERSION 32): focus-tunnel StreamingValidator early-term row.
///
/// Emitted by the post-E-7 WIRING phase whenever the synchronous
/// library-tier validator (`crate::focus_tunnel::streaming::StreamingValidator`)
/// returns `ValidationDecision::EarlyTerm` during a streaming generation
/// inside `run_fim_step` / `run_diff_step`. The validator itself is
/// I/O-free; the WIRING phase translates `EarlyTermWireForm` into this
/// payload and writes it via the SsbWriter.
///
/// Flat record of strings + numbers (no nested enums) so the dashboard
/// `payload_json` bridge can render rows without importing focus_tunnel
/// symbols. `reason` is the wire-form `EarlyTermReason` (lazy_pattern /
/// unbalanced_braces / output_too_long / deprecated_import /
/// unbounded_string_literal); `reason_detail` carries the structured detail
/// (pattern_id for LazyPattern; depth for UnbalancedBraces; etc).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StreamingEarlyTerminatedPayload {
    /// `"baby_step_{id}"` stringified at emit time so the dashboard can
    /// correlate this row with its originating GenerationModeSelected row
    /// and BabyStepSpec.
    pub step_id: String,
    /// One of `"fim"` / `"diff"` / `"structured"` (snake_case wire form
    /// matching `GenerationMode` serde). Captured at validator construction
    /// time so the row remains consistent if the validator outlives the
    /// step's selection record.
    pub mode: String,
    /// Wire-form `EarlyTermReason`. See `EarlyTermReason::ssb_wire_str`.
    pub reason: String,
    /// Structured detail string — `pattern_id` for LazyPattern;
    /// `depth=N` for UnbalancedBraces; `expected=N;actual=M` for
    /// OutputTooLong; `<name>-><replacement>` for DeprecatedImport;
    /// `opener_token=N` for UnboundedStringLiteral. None when the variant
    /// carries no structured detail.
    pub reason_detail: Option<String>,
    /// Token count at the moment of termination — proxy for "tokens we
    /// stopped emitting" telemetry.
    pub token_count_at_term: u32,
    /// Total tokens received from the sidecar before the terminate
    /// decision. Equal to `token_count_at_term` in the E-4 design (no
    /// in-flight buffer beyond the trigger); reserved for future overlap
    /// semantics where the validator could buffer ahead.
    pub total_tokens_received: u32,
    /// `"rust"` / `"python"` / `"generic"` — informational, drives the
    /// pattern set the validator applied.
    pub language_hint: String,
}

/// B-28 WIRING-1 (SCHEMA_VERSION 33): focus-tunnel PipelinePreAssembler row.
///
/// Emitted by the post-execute hook in `task_runner::drive_task` after
/// `PipelinePreAssembler::on_step_outcome(step_idx, passed, &target_files)`
/// fires. Carries the rolling pass-rate observed at decision time (in basis
/// points, 0..=10_000) and whether the pre-assembled context was discarded
/// because the next step touched different files.
///
/// Flat record of primitives (no nested enums) so the dashboard
/// `payload_json` bridge can render rows without importing focus_tunnel
/// symbols. `was_discarded == true` and `bytes_assembled == 0` mark a
/// canceled pre-assembly (the rolling-window discard path).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PipelinePreAssembledPayload {
    /// Zero-based index of the step whose pre-assembly produced this row.
    pub step_idx: u64,
    /// Rolling pass-rate at decision time, in basis points (0..=10_000).
    /// Stored as integer to avoid float drift across crates and SSB
    /// payload_json bridges.
    pub pass_rate_bp: u16,
    /// True when the pre-assembled context was discarded (rolling-window
    /// adaptive disable or per-step target-file change).
    pub was_discarded: bool,
    /// In-flight depth at decision time. PipelinePreAssembler caps at
    /// `MAX_DEPTH = 1` today; widened to u8 for future relaxation.
    pub depth_at_time: u8,
    /// Bytes assembled into the readiness payload (0 when discarded).
    pub bytes_assembled: u64,
}

/// B-28 WIRING-1 (SCHEMA_VERSION 34): focus-tunnel DependencyResolver row.
///
/// Emitted once per `ResolutionDecision` after the post-execute import-scan
/// + resolve cycle. Carries flat strings (decision and reason use String
/// mirrors instead of nested enums, matching the
/// `ExecutionModeChangedPayload` precedent — keeps the focus_tunnel ↔ ssb
/// dependency direction unidirectional).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DependencyResolvedPayload {
    /// Wire-form import language: `"rust"` / `"python"` / `"typescript"` /
    /// `"go"` / `"javascript"` (matches `ImportLanguage::wire_str`).
    pub language: String,
    /// Dependency name as it appears in the import statement.
    pub name: String,
    /// Wire-form decision: `"auto_approve"` / `"queue_for_human"` /
    /// `"blocked"`.
    pub decision: String,
    /// Wire-form reason gating the decision (e.g. `"allowlist_match"`,
    /// `"not_allowlisted"`, `"anti_stale_waiver"`, `"registry_not_found"`).
    pub reason: String,
    /// Install command for AutoApprove decisions. `None` for QueueForHuman
    /// / Blocked or when the resolver did not propose an install plan.
    pub install_command: Option<String>,
}

/// B-28 WIRING-1 (SCHEMA_VERSION 35): focus-tunnel SpeculativeFileRouter row.
///
/// Emitted once per `RouteDecision::NeedsMicroStep` or
/// `RouteDecision::MicroStepCapExhausted`. `reason == "cap_exhausted"` is
/// the dashboard-side signal for the cap-exhausted branch; any other
/// `reason` value (e.g. `"undefined_symbol"`) maps to NeedsMicroStep
/// semantics. `count` is 0 on cap-exhausted rows.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MicroStepCreatedPayload {
    /// Baby-step id of the step that triggered the micro-step (the parent).
    pub parent_step_id: u64,
    /// Path of the created micro-step target file.
    pub created_path: String,
    /// Wire-form symbol kind: `"Module"` / `"Struct"` / `"Function"` /
    /// `"Const"`. Mirrors `MicroStepType` as a String to keep the focus_tunnel
    /// ↔ ssb dep direction unidirectional.
    pub type_hint: String,
    /// Reason string. Common values: `"undefined_symbol"` (NeedsMicroStep),
    /// `"cap_exhausted"` (MicroStepCapExhausted), `"parse_skipped"`.
    pub reason: String,
    /// Number of micro-steps this row represents (1 for NeedsMicroStep,
    /// 0 for MicroStepCapExhausted).
    pub count: u8,
}

/// B-30 (SCHEMA_VERSION 36): one-shot LanguageRegistry status row.
///
/// Emitted once at DKG engine startup, after the per-grammar `catch_unwind`
/// registration cycle completes. The `loaded` list contains language IDs of
/// successfully initialized grammars in alphabetical order. The `failed` list
/// contains `(language_id, error_message)` pairs in alphabetical order of
/// language ID.
///
/// Both lists are deterministically sorted — derived from a `BTreeMap` source —
/// so downstream consumers can rely on stable ordering without sorting themselves.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LanguageGrammarStatusPayload {
    /// Language IDs successfully loaded, in alphabetical order.
    pub loaded: Vec<String>,
    /// Language IDs that failed to load, paired with the error message,
    /// in alphabetical order of language ID.
    pub failed: Vec<(String, String)>,
}

impl From<elai_knowledge::dkg::LanguageRegistryStatus> for LanguageGrammarStatusPayload {
    fn from(s: elai_knowledge::dkg::LanguageRegistryStatus) -> Self {
        Self { loaded: s.loaded, failed: s.failed }
    }
}

/// B-29 Phase 5 (SCHEMA_VERSION 37): payload for one symbol_* tool call.
///
/// Emitted by `translate_tool_request` in `production_agent_rpc.rs` for
/// `symbol_impact`, `symbol_flow`, `symbol_context`, and `symbol_list` tool IDs.
/// Consumed by the dashboard KnowledgeDrawer Impact Analysis panel and
/// the ActivityFeed via `SsbViewer::latest_action_of_type("symbol_query")`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SymbolQueryPayload {
    /// The tool that was invoked ("symbol_impact" / "symbol_flow" / etc.).
    pub tool: String,
    /// The primary target string (file path or symbol name for impact/context;
    /// entrypoint_id for flow; optional file for list).
    pub target: String,
    /// Depth parameter from the request (0 if not applicable).
    pub hop_count: u32,
    /// Number of files in the result set (total_files for impact;
    /// symbols.len() for context/list; depth for flow).
    pub total_files: u32,
    /// Wall-clock latency for the traversal in milliseconds.
    pub latency_ms: u32,
}

// ---------------------------------------------------------------------------
// SsbPayload enum (typed wrapper over all action-type payloads)
// ---------------------------------------------------------------------------

/// Typed payload enum. Each variant maps 1:1 to an [`SsbActionType`] (31 total).
/// Serialized to msgpack via `rmp_serde` before storage.
///
/// All variants are schema-committed (77 total post-B-19A — was 76 in B-10,
/// 74 in Tier 4). B-19A adds `HookLatencyViolation`.
/// Variants without a current production emitter — `Claim`, `ModelSwap`,
/// `KnowledgeDegradation`, `IsolationMetric`, `BranchMerge`, `SpiralDetected`,
/// `SupervisionExhausted`, `InterventionRequest` — are forward-compat for
/// phases 3F / 4G / 4H / wire-antispiral / newphase-4G. Do not remove: on-disk
/// SSB files and the dashboard `payload_json` pipeline depend on the
/// discriminant set staying stable. `SpiralDetected` has a scheduled emitter
/// in Session wire-antispiral; `InterventionRequest` in Session newphase-4G
/// (AutonomousGate). B-10 added `ContextBudgetComputed` + `SsbFilteredByOverlap`
/// for the per-slot budget bar chart and 3E.5 entry-filter transparency.
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
    ToolDispatch(ToolDispatchPayload),
    ValidationTier(ValidationTierPayload),
    FailureClassified(FailureClassifiedPayload),
    GitCommit(GitCommitPayload),
    DairaTrace(DairaTracePayload),
    SystemProbe(SystemProbePayload),
    FeasibilityReport(FeasibilityReportPayload),
    NovelStepDetected(NovelStepDetectedPayload),
    StubRun(StubRunPayload),
    GateDecision(GateDecisionPayload),
    TaskSplit(TaskSplitPayload),
    EscalationFired(EscalationFiredPayload),
    ModeChanged(ModeChangedPayload),
    // Batch 3C (Phase 4H) — Layer 1A.
    ProposalGenerated(ProposalGeneratedPayload),
    ProposalApplied(ProposalAppliedPayload),
    ProposalRejected(ProposalRejectedPayload),
    MsiUpdate(MsiUpdatePayload),
    ConfigArchived(ConfigArchivedPayload),
    // Batch 4C — scattered A-step wiring.
    ProfileSelected(ProfileSelectedPayload),
    FsmTransition(FsmTransitionPayload),
    MicroReflection(MicroReflectionPayload),
    TelescopeCollapse(TelescopeCollapsePayload),
    ConfigWarning(ConfigWarningPayload),
    // Batch-fu — cross-cutting follow-up wiring.
    ConstraintsInjected(ConstraintsInjectedPayload),
    InterventionResolved(InterventionResolvedPayload),
    // Batch-ful-a — DkgQuerierAdapter hot-swap notification.
    DkgQuerierSwapped(DkgQuerierSwappedPayload),
    // Batch-ful-c — A-8 freshness steering vector lifecycle.
    SteeringComputed(SteeringComputedPayload),
    SteeringComputeFailed(SteeringComputeFailedPayload),
    // Batch-ful-d — external MCP server lifecycle + per-call rows.
    McpServerConnected(McpServerConnectedPayload),
    McpToolCalled(McpToolCalledPayload),
    McpServerDisconnected(McpServerDisconnectedPayload),
    // Tier 1 CHAT window — dashboard-originated message and card rows.
    ChatMessage(ChatMessagePayload),
    ChatResponse(ChatResponsePayload),
    // Tier 4 Settings reorg (dashboard-wiring-plan.txt §8) — execution-mode
    // selection from the Settings window (LOCAL / CLOUD / HYBRID).
    ExecutionModeChanged(ExecutionModeChangedPayload),
    // B-10 (DECISION-C) — per-dispatch ContextBudget snapshot for the
    // dashboard's per-slot bar chart, and 3E.5 SSB entry filtering summary.
    ContextBudgetComputed(ContextBudgetComputedPayload),
    SsbFilteredByOverlap(SsbFilteredByOverlapPayload),
    // B-19A (T10) — per-tier hook latency budget violation beacon.
    HookLatencyViolation(HookLatencyViolationPayload),
    // Track A.2 — synthesis path selection per phase_f_synthesize call.
    ResearchPathChosen(ResearchPathChosenPayload),
    // Track A.6 — grounding cache hit/miss event per ground_or_cached call.
    GroundingCacheEvent(GroundingCacheEventPayload),
    // B-4: circuit-breaker state transition for embedding/reranker models.
    CircuitBreakerStateChange(CircuitBreakerStateChangePayload),
    // B-7: bubble-up cascade, integration checkpoints, DAG repair, decomposition nodes.
    BubbleUpTyped(BubbleUpTypedPayload),
    IntegrationCheckpointSuccess(IntegrationCheckpointSuccessPayload),
    DagRepairRound(DagRepairRoundPayload),
    ConceptDecompositionNode(ConceptDecompositionNodePayload),
    // B-7 (SCHEMA_VERSION 100): pipeline factory producer wiring.
    TreePositionChanged(TreePositionChangedPayload),
    FailureRecorded(FailureRecordedPayload),
    PackageRefVerified(PackageRefVerifiedPayload),
    TechStackDecided(TechStackDecidedPayload),
    // B-6: orchestration producer wiring.
    EvidenceGateEvaluated(EvidenceGateEvaluatedPayload),
    SupervisionEvent(SupervisionEventPayload),
    BspSuperstep(BspSuperstepPayload),
    AgentLifecycle(AgentLifecyclePayload),
    AntiSpiralHit(AntiSpiralHitPayload),
    // B-27: cloud-mode spend ledger + escalation marker.
    CloudSpend(CloudSpendPayload),
    EscalateToCloud(EscalateToCloudPayload),
    // B-23 Phase 4H Group A: self-mod beacons (Layer 1A growth + Layer 3 +
    // OSC + REG + EVO). Five batch-3C variants (proposal_*, msi_update,
    // config_archived) shipped earlier and live above.
    DataTableGrew(DataTableGrewPayload),
    L3Proposal(L3ProposalPayload),
    ConvergenceStagnant(ConvergenceStagnantPayload),
    ConfigOscillation(ConfigOscillationPayload),
    RegressionDetected(RegressionDetectedPayload),
    EvolutionWithheld(EvolutionWithheldPayload),
    // B-39 chat reflect pipeline: per-turn REFLECT stage beacons.
    ReflectTriaged(ReflectTriagedPayload),
    ReflectCandidate(ReflectCandidatePayload),
    ReflectAuditVerdict(ReflectAuditVerdictPayload),
    ReflectRefined(ReflectRefinedPayload),
    // B-24 Phase 4I Group C1: media validator beacons.
    MediaValidationStarted(MediaValidationStartedPayload),
    MediaTier1Complete(MediaTier1CompletePayload),
    MediaTier2Complete(MediaTier2CompletePayload),
    RenderComplete(RenderCompletePayload),
    VisualFeedbackIteration(VisualFeedbackIterationPayload),
    MediaHumanReviewRequired(MediaHumanReviewRequiredPayload),
    // B-28 E-3 (SCHEMA_VERSION 31): focus-tunnel ModeSelector decision row.
    GenerationModeSelected(GenerationModeSelectedPayload),
    // B-28 E-4 (SCHEMA_VERSION 32): focus-tunnel StreamingValidator early-term row.
    StreamingEarlyTerminated(StreamingEarlyTerminatedPayload),
    // B-28 WIRING-1 (SCHEMA_VERSION 33): pre-assembly readiness row.
    PipelinePreAssembled(PipelinePreAssembledPayload),
    // B-28 WIRING-1 (SCHEMA_VERSION 34): dependency resolver decision row.
    DependencyResolved(DependencyResolvedPayload),
    // B-28 WIRING-1 (SCHEMA_VERSION 35): speculative file-router micro-step row.
    MicroStepCreated(MicroStepCreatedPayload),
    // B-30 (SCHEMA_VERSION 36): one-shot language grammar registry status row.
    LanguageGrammarStatus(LanguageGrammarStatusPayload),
    // B-29 Phase 5 (SCHEMA_VERSION 37): one row per symbol_* tool call.
    SymbolQuery(SymbolQueryPayload),
    // B-32 (SCHEMA_VERSION 38): one row per minified file detected by the
    // adaptive character chunker.
    MinifiedFileChunked(MinifiedFileChunkedPayload),
    // B-31.5 (SCHEMA_VERSION 39): 5 deferred B-31 SSB variants — replaces
    // the `tracing::info!(target="ssb_pending", ...)` placeholders from
    // Cycle B.
    IndexCheckpoint(IndexCheckpointPayload),
    IndexResume(IndexResumePayload),
    IndexCancel(IndexCancelPayload),
    LockAcquired(LockAcquiredPayload),
    LockContested(LockContestedPayload),
    // B-33 follow-up (SCHEMA_VERSION 40): symbol-graph build event row.
    SymbolGraphUpdated(SymbolGraphUpdatedPayload),
    // B-40 (SCHEMA_VERSION 41): FRAMING substate beacons.
    IdeaFramingStarted(IdeaFramingStartedPayload),
    ProjectClassClassified(ProjectClassClassifiedPayload),
    CriticRoundComplete(CriticRoundCompletePayload),
    IdeaFramed(IdeaFramedPayload),
    TradeoffCardEmitted(TradeoffCardEmittedPayload),
    // B-34 (SCHEMA_VERSION 42): entry-point detection + artifact pipeline beacons.
    EntryPointsDetected(EntryPointsDetectedPayload),
    ArtifactsIndexed(ArtifactsIndexedPayload),
    ArtifactSearched(ArtifactSearchedPayload),
    ArtifactStale(ArtifactStalePayload),
    // B-38 (SCHEMA_VERSION 43): ambiguity gate beacons.
    AmbiguityScored(AmbiguityScoredPayload),
    AmbiguityFloorFailed(AmbiguityFloorFailedPayload),
    AmbiguityStreakAdvanced(AmbiguityStreakAdvancedPayload),
    // B-42 (SCHEMA_VERSION 44): cosmos tier knowledge worker beacons.
    CosmosRefreshStarted(CosmosRefreshStartedPayload),
    CosmosArtifactUpdated(CosmosArtifactUpdatedPayload),
    CosmosArtifactDecayed(CosmosArtifactDecayedPayload),
    CosmosGroundingUsed(CosmosGroundingUsedPayload),
    // B-41 (SCHEMA_VERSION 45): per-level auditor mode beacons.
    PerLevelAuditStarted(PerLevelAuditStartedPayload),
    PerLevelAuditVerdict(PerLevelAuditVerdictPayload),
    PerLevelAuditCascade(PerLevelAuditCascadePayload),
    // B-37C (SCHEMA_VERSION 46): multi-runtime backend beacons.
    RuntimeFallback(RuntimeFallbackPayload),
    RuntimeHealthChanged(RuntimeHealthChangedPayload),
    CliSubprocessSpawned(CliSubprocessSpawnedPayload),
    CliSubprocessKilled(CliSubprocessKilledPayload),
    // E-51 (SCHEMA_VERSION 96): edge-inference beacons.
    MlxSubprocessSpawned(MlxSubprocessSpawnedPayload),
    MlxSubprocessKilled(MlxSubprocessKilledPayload),
    LlamaCppSubprocessSpawned(LlamaCppSubprocessSpawnedPayload),
    LlamaCppSubprocessKilled(LlamaCppSubprocessKilledPayload),
    MlxApplePlatformDetected(MlxApplePlatformDetectedPayload),
    LlamaCppEpDetected(LlamaCppEpDetectedPayload),
    RuntimeChainBuilt(RuntimeChainBuiltPayload),
    // E-17 SP2 (SCHEMA_VERSION 97): RAG retrieval-quality beacons.
    DeepPoolCapped(DeepPoolCappedPayload),
    RrfQualityGateFired(RrfQualityGateFiredPayload),
    HydeSkippedCodeQuery(HydeSkippedCodeQueryPayload),
    RrfComponentWeights(RrfComponentWeightsPayload),
    // C-1 (SCHEMA_VERSION 47): knowledge + reasoning producer beacons.
    GrcStepEmitted(GrcStepEmittedPayload),
    GroundingSweepResult(GroundingSweepResultPayload),
    CsqEvaluated(CsqEvaluatedPayload),
    AuditorRubberStampRejected(AuditorRubberStampRejectedPayload),
    PerRoleDecodingApplied(PerRoleDecodingAppliedPayload),
    HydeQuery(HydeQueryPayload),
    VectorCacheHitRate(VectorCacheHitRatePayload),
    QueryTrace(QueryTracePayload),
    // C-2 (SCHEMA_VERSION 48): orchestration + cosmos + stagnation beacons.
    PhaseNodeReady(PhaseNodeReadyPayload),
    StepNodeReady(StepNodeReadyPayload),
    BabyStepReady(BabyStepReadyPayload),
    DagRepairEscalated(DagRepairEscalatedPayload),
    AgentStatusChanged(AgentStatusChangedPayload),
    StagnationSpinningDetected(StagnationSpinningDetectedPayload),
    StagnationOscillationDetected(StagnationOscillationDetectedPayload),
    LateralThinkingActivated(LateralThinkingActivatedPayload),
    GoalDriftWarning(GoalDriftWarningPayload),
    WatchdogTimeout(WatchdogTimeoutPayload),
    // C-7 (SCHEMA_VERSION 49): reviewer verdict beacon.
    ChangeDecision(ChangeDecisionPayload),
    // C-8 (SCHEMA_VERSION 50): search-mode cross-process bridge.
    SearchModeChanged(SearchModeChangedPayload),
    // C-5 (SCHEMA_VERSION 51): privacy-redaction event.
    RedactionApplied(RedactionAppliedPayload),
    // C-10 (SCHEMA_VERSION 52): anti-spiral lateral-thinking + goal-drift Block.
    LateralThinkingSucceeded(LateralThinkingSucceededPayload),
    LateralThinkingExhausted(LateralThinkingExhaustedPayload),
    GoalDriftBlock(GoalDriftBlockPayload),
    // C-13 (SCHEMA_VERSION 53): WIP commit squash completion.
    WipSquashComplete(WipSquashCompletePayload),
    // C-22 (SCHEMA_VERSION 54): feasibility-probe completeness beacons.
    AlternativeExplored(AlternativeExploredPayload),
    FeasibilityEscalated(FeasibilityEscalatedPayload),
    // C-23 (SCHEMA_VERSION 55): SessionDag branch-UI beacons.
    BranchForked(BranchForkedPayload),
    BranchMerged(BranchMergedPayload),
    BranchClosed(BranchClosedPayload),
    // C-25 (SCHEMA_VERSION 56): compiled-pipeline + template-crystallizer
    // + background-task beacons. Payload structs live below.
    PlanSkeleton(PlanSkeletonPayload),
    CompiledPipelineFired(CompiledPipelineFiredPayload),
    TemplateCrystallized(TemplateCrystallizedPayload),
    BackgroundTaskStarted(BackgroundTaskStartedPayload),
    BackgroundTaskComplete(BackgroundTaskCompletePayload),
    DecompileTriggered(DecompileTriggeredPayload),
    // C-26 (SCHEMA_VERSION 58): constrained-decoding + futility primitives.
    FutilityDetected(FutilityDetectedPayload),
    FutilityOutputStagnation(FutilityOutputStagnationPayload),
    // C-27 (SCHEMA_VERSION 59): Testing-trust T(-1) + T-0 beacons. Payload
    // structs live below near the RedactionAppliedPayload region.
    WiringEvidence(WiringEvidencePayload),
    WiringViolation(WiringViolationPayload),
    DeterminismDrift(DeterminismDriftPayload),
    // C-28 (SCHEMA_VERSION 60): Testing-trust T-1 mutation testing.
    MutationSurvived(MutationSurvivedPayload),
    // C-29 (SCHEMA_VERSION 61): Testing-trust T-2 (IntentPair) + T-3
    // (Metamorphic). Payload structs live below near MutationSurvivedPayload.
    IntentConfirmed(IntentConfirmedPayload),
    MrViolated(MrViolatedPayload),
    // C-30 (SCHEMA_VERSION 62): Surgical Mode + Oracle Mode pipeline beacons.
    SurgicalPlanCreated(SurgicalPlanCreatedPayload),
    SurgicalLocWarning(SurgicalLocWarningPayload),
    OracleCacheHit(OracleCacheHitPayload),
    OracleCacheMiss(OracleCacheMissPayload),
    ProjectDetectionSignal(ProjectDetectionSignalPayload),
    // C-32 (SCHEMA_VERSION 63): Code-Completeness Layer 2 beacons. Payload
    // structs live below the C-30 region.
    SkeletonGenerated(SkeletonGeneratedPayload),
    SkeletonFlushed(SkeletonFlushedPayload),
    MiniCriticVerdict(MiniCriticVerdictPayload),
    StructuralRetryStageAttempt(StructuralRetryStageAttemptPayload),
    BridgeFileGenerated(BridgeFileGeneratedPayload),
    // D-1 (SCHEMA_VERSION 64): ReasoningTemplate ArchiveChain lifecycle.
    ArchiveEntryAdded(ArchiveEntryAddedPayload),
    ArchiveEntryRetired(ArchiveEntryRetiredPayload),
    TemplatePromoted(TemplatePromotedPayload),
    TemplateCandidateRejected(TemplateCandidateRejectedPayload),
    // E-5 (SCHEMA_VERSION 65): memory_facts bi-temporal lifecycle beacons.
    MemoryFactExtracted(MemoryFactExtractedPayload),
    MemoryFactSuperseded(MemoryFactSupersededPayload),
    // D-2 (SCHEMA_VERSION 66): PPM verifier beacons.
    PpmScored(PpmScoredPayload),
    PpmGatingFailed(PpmGatingFailedPayload),
    // E-1 (SCHEMA_VERSION 67): infinite-context chat illusion beacons.
    ChatReflectionCompleted(ChatReflectionCompletedPayload),
    MemoryEvolutionRun(MemoryEvolutionRunPayload),
    // E-1 dashboard slice (SCHEMA_VERSION 68): MemoryNotes / CoreBlock
    // producer + conflict beacons.
    MemoryNoteInserted(MemoryNoteInsertedPayload),
    MemoryNoteEvolved(MemoryNoteEvolvedPayload),
    MemoryConflictDetected(MemoryConflictDetectedPayload),
    CoreBlockReplaced(CoreBlockReplacedPayload),
    // D-3 (SCHEMA_VERSION 69): multi-verifier consensus aggregator.
    VerifierConsensusPass(VerifierConsensusPassPayload),
    VerifierConsensusFail(VerifierConsensusFailPayload),
    // D-4 (SCHEMA_VERSION 70): tripwire stack beacons.
    TripwireFired(TripwireFiredPayload),
    TemplateAutoReverted(TemplateAutoRevertedPayload),
    // D-4 GAP-2 (SCHEMA_VERSION 108): concurrent-detector tripwire row.
    TripwireFiredConcurrent(TripwireFiredConcurrentPayload),
    // D-19 (SCHEMA_VERSION 71): PRMBench adversarial-gaming benchmark runner.
    PrmBenchRunCompleted(PrmBenchRunCompletedPayload),
    // D-5 (SCHEMA_VERSION 72): BAI Thompson sampling promotion round.
    BaiArmUpdated(BaiArmUpdatedPayload),
    // D-5 closure (SCHEMA_VERSION 208): unified promotion-decision summary.
    BaiPromotionDecision(BaiPromotionDecisionPayload),
    // D-6 (SCHEMA_VERSION 73): held-out task store lifecycle.
    HeldOutTaskAdded(HeldOutTaskAddedPayload),
    HeldOutTaskUsed(HeldOutTaskUsedPayload),
    // D-7 (SCHEMA_VERSION 74): cross-domain transfer canary beacons.
    CanaryRunCompleted(CanaryRunCompletedPayload),
    CanaryFamilyEvaluated(CanaryFamilyEvaluatedPayload),
    LockedKeysAuditRun(LockedKeysAuditRunPayload),
    LockedKeysViolationDetected(LockedKeysViolationDetectedPayload),
    // E-46 (SCHEMA_VERSION 76): hierarchical decomposition tuning beacons.
    TaskAdvanced(TaskAdvancedPayload),
    AgentlessShortcutTaken(AgentlessShortcutTakenPayload),
    GoalDriftFired(GoalDriftFiredPayload),
    // D-10 (SCHEMA_VERSION 77): mode-routing predicate beacon.
    ClaimRouted(ClaimRoutedPayload),
    // D-21 (SCHEMA_VERSION 78): mode-routing coverage benchmark beacon.
    ModeRoutingBenchRunCompleted(ModeRoutingBenchRunCompletedPayload),
    // D-11 (SCHEMA_VERSION 79): constructive-checker registry beacon.
    CheckerResultEmitted(CheckerResultEmittedPayload),
    // D-13 (SCHEMA_VERSION 80): adversarial-verification beacons.
    AdversaryGenerated(AdversaryGeneratedPayload),
    ConsensusReevaluatedWithCounter(ConsensusReevaluatedWithCounterPayload),
    // D-12 (SCHEMA_VERSION 81): OperationalizationCheck baby-step
    // predicate dispatch beacon.
    BabyStepDispatched(BabyStepDispatchedPayload),
    // D-16 (SCHEMA_VERSION 82): UniCR conformal-refusal calibration beacons.
    ConformalTauComputed(ConformalTauComputedPayload),
    ConformalRefusalDecided(ConformalRefusalDecidedPayload),
    // D-15 (SCHEMA_VERSION 83): discovery-vs-search NoveltyClass beacon.
    NoveltyClassified(NoveltyClassifiedPayload),
    // D-18 (SCHEMA_VERSION 84): typed-ObservableEffect predicate beacon.
    ObservableEffectVerified(ObservableEffectVerifiedPayload),
    // D-17 (SCHEMA_VERSION 85): honest-refusal SLA monitoring beacons.
    RefusalRateSnapshot(RefusalRateSnapshotPayload),
    RefusalSlaBreach(RefusalSlaBreachPayload),
    // D-22 (SCHEMA_VERSION 86): conformal-calibration validation beacon.
    ConformalValidationCompleted(ConformalValidationCompletedPayload),
    // D-20 (SCHEMA_VERSION 87): ARC-AGI-2 OOD probe gating beacon.
    ArcBenchmarkCompleted(ARCBenchmarkCompletedPayload),
    ToolGated(ToolGateEventPayload),
    ToolEager(ToolGateEventPayload),
    ToolLazyResolved(ToolGateEventPayload),
    ToolDenied(ToolGateEventPayload),
    QuantBackendSelected(QuantBackendSelectedPayload),
    KvCacheDtypeSoftwareSim(KvCacheDtypeSoftwareSimPayload),
    AwqCheckpointBuilt(AwqCheckpointBuiltPayload),
    CliMcpDispatch(CliMcpDispatchPayload),
    CliMcpServerStarted(CliMcpServerStartedPayload),
    CliMcpServerStopped(CliMcpServerStoppedPayload),
    // E-15 sandbox beacons
    FirecrackerRestoreSlow(FirecrackerRestoreSlowPayload),
    SnapshotUniquenessRegenerated(SnapshotUniquenessRegeneratedPayload),
    SnapshotEvicted(SnapshotEvictedPayload),
    SandboxBackendSelected(SandboxBackendSelectedPayload),
    AgentForkedFromSnapshot(AgentForkedFromSnapshotPayload),
    ResearcherNetEgressBlocked(ResearcherNetEgressBlockedPayload),
    // E-24: MCP hardening beacons
    McpSchemaPinViolation(McpSchemaPinViolationPayload),
    McpSanitizationRuleApplied(McpSanitizationRuleAppliedPayload),
    McpBudgetEnforced(McpBudgetEnforcedPayload),
    McpLegacySseTransport(McpLegacySseTransportPayload),
    McpLlmGuardBlocked(McpLlmGuardBlockedPayload),
    AnthropicGatewayDetected(AnthropicGatewayDetectedPayload),
    // C-48: SafeHarbor memory-trust beacons
    MemoryTierAssigned(MemoryTierAssignedPayload),
    MemoryQueryFiltered(MemoryQueryFilteredPayload),
    SafetyCriticalBlocked(SafetyCriticalBlockedPayload),
    // E-30: public-incident hardening beacons
    ArgvDestructiveDenied(ArgvDestructiveDeniedPayload),
    ArgvApprovalRequired(ArgvApprovalRequiredPayload),
    TrustDialogHookDeferred(TrustDialogHookDeferredPayload),
    TrustDialogEnvDeferred(TrustDialogEnvDeferredPayload),
    McpPinVersionRefused(McpPinVersionRefusedPayload),
    AuditorLineageCollision(AuditorLineageCollisionPayload),
    UntrustedMemoryWriteBlocked(UntrustedMemoryWriteBlockedPayload),
    IncidentRecordCreated(IncidentRecordCreatedPayload),
    // E-32: context-window extension beacons
    FitmCalibrationApplied(FitmCalibrationAppliedPayload),
    DeepModeActivated(DeepModeActivatedPayload),
    DeepModeEscalationChosen(DeepModeEscalationChosenPayload),
    YarnRefused(YarnRefusedPayload),
    // B-3 (SCHEMA_VERSION 98): AntiStalenessDrawer + RulesDrawer beacons.
    DepAuditCompleted(DepAuditCompletedPayload),
    BaselineRunCompleted(BaselineRunCompletedPayload),
    DeterministicAuditRun(DeterministicAuditRunPayload),
    RuleTransition(RuleTransitionPayload),
    // B-4 (SCHEMA_VERSION 99): canary drift snapshot row.
    CanaryDrift(CanaryDriftPayload),
    // B-19 (SCHEMA_VERSION 101): per-agent-execution DAIRA marker aggregation.
    MarkerReport(MarkerReportPayload),
    // B-25 (SCHEMA_VERSION 102): Phase 4J Empire integration. 5 atomic variants.
    EmpireConnected(EmpireConnectedPayload),
    EmpireDisconnected(EmpireDisconnectedPayload),
    EmpireEventReceived(EmpireEventReceivedPayload),
    EmpireCapabilityInvoked(EmpireCapabilityInvokedPayload),
    EmpireRepairProposed(EmpireRepairProposedPayload),
    // C-5 follow-on (SCHEMA_VERSION 103): cloud-mode hardening trio.
    RouteBlocked(RouteBlockedPayload),
    WarnLimitReached(WarnLimitReachedPayload),
    CostEstimate(CostEstimatePayload),
    // C-36 (SCHEMA_VERSION 104): MuCoCo mutation operator dispatch.
    MucocoMutationApplied(MucocoMutationAppliedPayload),
    // D-3 closure (SCHEMA_VERSION 105): PpmUnavailable +
    // VerifierDisagreement + CrossContextDispatched.
    PpmUnavailable(PpmUnavailablePayload),
    VerifierDisagreement(VerifierDisagreementPayload),
    CrossContextDispatched(CrossContextDispatchedPayload),
    // C-38 (SCHEMA_VERSION 107): pagerank + dkg rebuild lifecycle.
    PagerankRun(PagerankRunPayload),
    DkgRebuildStarted(DkgRebuildStartedPayload),
    DkgRebuildComplete(DkgRebuildCompletePayload),
    // C-40 (SCHEMA_VERSION 109): GitHub PR adapter producer surface.
    GithubPrOpened(GithubPrOpenedPayload),
    GithubPrStatusFetched(GithubPrStatusFetchedPayload),
    GithubPrClosed(GithubPrClosedPayload),
    GithubPrMerged(GithubPrMergedPayload),
    // E-41 (SCHEMA_VERSION 135): GitHub PR review surface. 5 atomic rows
    // emitted by the github_pr review / watcher / verify / semgrep /
    // auto-merge submodules. See payload structs below.
    PrReviewComposed(PrReviewComposedPayload),
    PrCommentWatched(PrCommentWatchedPayload),
    PrPackageVerifyResult(PrPackageVerifyResultPayload),
    PrSemgrepGateResult(PrSemgrepGateResultPayload),
    PrAutoMergeBlocked(PrAutoMergeBlockedPayload),
    // D-5 closure (SCHEMA_VERSION 110): Thompson-sample beacon paired
    // with the pre-existing `BaiArmUpdated` row. See payload struct
    // `ThompsonSampledPayload` for field documentation.
    ThompsonSampled(ThompsonSampledPayload),
    // C-41 (SCHEMA_VERSION 111): skills-driven workflow registry —
    // SkillTriggered + SkillExecuted pair around `skill_registry::top_k_skills`
    // and the harness `tools::execute_skill` dispatch. See payload structs
    // below for field documentation.
    SkillTriggered(SkillTriggeredPayload),
    SkillExecuted(SkillExecutedPayload),
    // D-6 closure (SCHEMA_VERSION 112): held-out promotion-gate +
    // refresh-event beacons. See payload structs below for field
    // documentation.
    HeldOutPromotionGate(HeldOutPromotionGatePayload),
    HeldOutRefresh(HeldOutRefreshPayload),
    // C-42 (SCHEMA_VERSION 113): Semantic Gateway L2 RBAC beacons. See
    // payload structs below for field documentation.
    GatewayPolicyAllowed(GatewayPolicyAllowedPayload),
    GatewayPolicyDenied(GatewayPolicyDeniedPayload),
    GatewayHitlPending(GatewayHitlPendingPayload),
    CrossDomainRegression(CrossDomainRegressionPayload),
    CanaryFamilyPass(CanaryFamilyPassPayload),
    CanaryInsufficientData(CanaryInsufficientDataPayload),
    // D-8 closure (SCHEMA_VERSION 115): cryptographic archive verify result.
    // Typed payload (NOT Vec<String>) — see ArchiveVerifyResultPayload for
    // the rationale.
    ArchiveVerifyResult(ArchiveVerifyResultPayload),
    // C-43 closure (SCHEMA_VERSION 116): Phase 4H scope-freeze.
    ScopeFreezeRatified(ScopeFreezeRatifiedPayload),
    ScopeFreezeViolated(ScopeFreezeViolatedPayload),
    // C-44 (SCHEMA_VERSION 117): Composer 2 paid-backend cost accrual.
    BackendCostAccrued(BackendCostAccruedPayload),
    // C-45 (SCHEMA_VERSION 120): CANTANTE credit-attribution beacons.
    // See payload structs below for field documentation.
    AgentRoleDropped(AgentRoleDroppedPayload),
    CreditAttributionRun(CreditAttributionRunPayload),
    CreditAttributionResult(CreditAttributionResultPayload),
    // C-46 (SCHEMA_VERSION 121): Uno-Orchestra selective-delegation beacons.
    // See payload structs below for field documentation.
    SelectiveDelegationDecision(SelectiveDelegationDecisionPayload),
    SelectiveDelegationEscalated(SelectiveDelegationEscalatedPayload),
    SelectiveDelegationCostSaved(SelectiveDelegationCostSavedPayload),
    // C-47 (SCHEMA_VERSION 122): Case-Based Calibration of Tool-Use.
    // See payload structs below for field documentation. Paper-grounded
    // against arXiv:2605.15041 (CAST).
    ToolCallLogged(ToolCallLoggedPayload),
    ToolCalibrationHintEmitted(ToolCalibrationHintEmittedPayload),
    ToolCalibrationDriftDetected(ToolCalibrationDriftDetectedPayload),
    // C-49 (SCHEMA_VERSION 123): iterative-distillation experience.
    // ELAI heuristic adaptation of arXiv:2605.14443 (RL prompter is
    // RUNPOD-PENDING). See payload structs below for field docs.
    PromptPolicyDistillationRun(PromptPolicyDistillationRunPayload),
    PromptFragmentSelected(PromptFragmentSelectedPayload),
    PromptFragmentDiscarded(PromptFragmentDiscardedPayload),
    // D-14 (SCHEMA_VERSION 124): sycophancy probe row pair. ELAI
    // structural-defense adaptation of arXiv:2310.13548 (Sharma et al
    // 2024); see payload structs below for field docs.
    SycophancyProbeDispatched(SycophancyProbeDispatchedPayload),
    SycophancyFlag(SycophancyFlagPayload),
    // D-23 (SCHEMA_VERSION 142): sycophancy detection benchmark runner.
    SycophancyBenchRunCompleted(SycophancyBenchRunCompletedPayload),
    // D-24 (SCHEMA_VERSION 143): cross-domain canary regression benchmark
    // runner. Replays a fixture corpus through D-7 run_canary in-memory
    // and emits one row per CanaryBenchmark::run Ok return. NanoFlux
    // arXiv:2509.23252 provides empirical motivation only; no canary
    // mechanism in the paper. See payload struct below for field docs.
    CanaryBenchRunCompleted(CanaryBenchRunCompletedPayload),
    // D-25 (SCHEMA_VERSION 144): determinism + archive tamper-suite
    // benchmark runner. Drives D-8 verify_archive IN-PROCESS across a
    // 10-scenario JSON-manifest matrix and emits one row per
    // TamperBenchmark::run Ok return. arxiv:2507.05619 (T1) provides
    // integrity-checksum motivation only; no tamper-suite mechanism
    // in the paper. See payload struct below for field docs.
    DeterminismTamperSuiteCompleted(DeterminismTamperSuiteCompletedPayload),
    // D-28 (SCHEMA_VERSION 145): adversarial example farming run.
    // Host-side scaffolding for the Adv-RM (arXiv:2504.06141) farm
    // loop. Live PPM scoring loop + ppm_finetune.sh + dual
    // bench_prmbench v1/v2 are RUNPOD-PENDING. See payload struct
    // below for field docs.
    AdversarialFarmRunCompleted(AdversarialFarmRunCompletedPayload),
    // D-29 (SCHEMA_VERSION 146): hierarchical BAI round completion.
    // Wire-dark gated by ELAI_BAI_MODE=hierarchical AND ELAI_BAI_ENABLE=1.
    // Paper labels: arXiv:2111.06929 SUPPORTING EVIDENCE (regret min,
    // NOT BAI); arXiv:2106.10417 DIRECT GROUNDING (variance-dependent
    // BAI). Live regret baseline RUNPOD-PENDING. See payload struct
    // below for field docs.
    HierarchicalBaiRoundCompleted(HierarchicalBaiRoundCompletedPayload),
    // D-30 (SCHEMA_VERSION 147): calibration bootstrap warm-start
    // completion. Wire-dark gated by
    // ELAI_CALIBRATION_BOOTSTRAP_ENABLE (default OFF). Paper labels:
    // arXiv:2509.01455 UniCR COMPOSITION; arXiv:2109.07958 TruthfulQA
    // DATA ORIGIN; Politis & Romano 1994 JASA Vol 89 §3 DIRECT
    // GROUNDING (SECONDARY block-bootstrap CI only). Live D-22
    // validation + 817-row full corpus import + block-bootstrap CI
    // activation RUNPOD-PENDING. See payload struct below for field
    // docs.
    CalibrationBootstrapCompleted(CalibrationBootstrapCompletedPayload),
    // C-50 (SCHEMA_VERSION 125): Contract-Coding STRUCTURAL contract
    // test scaffolding row pair. Paper-grounded against
    // arXiv:2604.13100 (Lin, Zhao, Shi 2026, kernel K = <N, Sigma,
    // Delta> §4.2). See payload structs below for field docs.
    ContractTestScaffolded(ContractTestScaffoldedPayload),
    ContractViolationDetected(ContractViolationDetectedPayload),
    // C-52 (SCHEMA_VERSION 126): Dafny Formal Verification Spike
    // heal-loop termination beacon. Paper-grounded against
    // arXiv:2604.22601 (NL2VC). See payload struct below for field
    // docs (incl. Gemma 4 substitution caveat per memory rule
    // feedback_no_gemma4).
    DafnyVerifyCompleted(DafnyVerifyCompletedPayload),
    // D-16 closure (SCHEMA_VERSION 127): durable TauUpdater tau
    // persistence audit row. See [`CalibrationCorpusUpdatedPayload`]
    // for field docs. Paper grounding: arXiv:2509.01455 UniCR Eq.5
    // (alpha is the miscoverage rate; rho is the DISTINCT selective
    // risk budget — only alpha lives on the SSB row per
    // D-16-decisions.md DECISION-4).
    CalibrationCorpusUpdated(CalibrationCorpusUpdatedPayload),
    // E-12 Tier-B (SCHEMA_VERSION 128): framework profile selection
    // emitted by the scope-planner shim on ProjectClass ->
    // FrameworkProfile resolution. See
    // [`FrameworkProfileSelectedPayload`] for field docs.
    FrameworkProfileSelected(FrameworkProfileSelectedPayload),
    // E-12 Tier-B (SCHEMA_VERSION 128): legacy linter config strip
    // emitted once per file removed by
    // FrameworkAdapter::apply_lint_defaults. See
    // [`LegacyConfigStrippedPayload`] for field docs.
    LegacyConfigStripped(LegacyConfigStrippedPayload),
    // D-18 closure (SCHEMA_VERSION 129): typed-ObservableEffect 4-variant
    // pack (dispatch + Eureka Goodhart + Coscientist sandbox / production).
    // See `ObservableEffectDispatchedPayload`, `GoodhartRiskFlagPayload`,
    // `SandboxObservationPayload`, `ProductionObservationProposalPayload`.
    ObservableEffectDispatched(ObservableEffectDispatchedPayload),
    GoodhartRiskFlag(GoodhartRiskFlagPayload),
    SandboxObservation(SandboxObservationPayload),
    ProductionObservationProposal(ProductionObservationProposalPayload),
    // E-13 (SCHEMA_VERSION 130): TDD verifier-agent beacon family.
    // See `TddCycleStartedPayload`, `TddTestFailedPayload`,
    // `TddTestPassedPayload`.
    TddCycleStarted(TddCycleStartedPayload),
    TddTestFailed(TddTestFailedPayload),
    TddTestPassed(TddTestPassedPayload),
    // Tranche B-7 triple (SCHEMA_VERSION 195): TDD verification gate
    // substrate. See `TddVerificationEventPayload` + 4-arm
    // `TddVerificationEventKind`. Pattern-2 (46th application).
    TddVerificationEvent(TddVerificationEventPayload),
    // E-14 (SCHEMA_VERSION 131): browser-e2e agent beacon family. See
    // `BrowserE2eRunStartedPayload`, `BrowserE2eAssertionFailedPayload`,
    // `BrowserE2eAssertionPassedPayload`, `BrowserE2eSnapshotCapturedPayload`.
    BrowserE2eRunStarted(BrowserE2eRunStartedPayload),
    BrowserE2eAssertionFailed(BrowserE2eAssertionFailedPayload),
    BrowserE2eAssertionPassed(BrowserE2eAssertionPassedPayload),
    BrowserE2eSnapshotCaptured(BrowserE2eSnapshotCapturedPayload),
    // E-18 (SCHEMA_VERSION 132): tool-use reliability gate beacons.
    // See `ToolReliabilityGatePassedPayload`,
    // `ToolReliabilityGateRejectedPayload`, `ToolCallSpilledPayload`,
    // `ToolCallRetryEscalatedPayload` for field docs.
    ToolReliabilityGatePassed(ToolReliabilityGatePassedPayload),
    ToolReliabilityGateRejected(ToolReliabilityGateRejectedPayload),
    ToolCallSpilled(ToolCallSpilledPayload),
    ToolCallRetryEscalated(ToolCallRetryEscalatedPayload),
    // E-37 (SCHEMA_VERSION 133): prompt-injection defense beacons.
    // 9 atomic variants. See `InjectionAttemptDetectedPayload`,
    // `InjectionAttemptBlockedPayload`, `SpotlightWrappedPayload`,
    // `SpotlightBreachedPayload`, `ToolDescriptionPinMismatchPayload`,
    // `CapabilityGateRejectedPayload`, `ProvenanceTaggedPayload`,
    // `PromptArmorVerdictPayload`, `InjectionFixtureReplayedPayload`.
    InjectionAttemptDetected(InjectionAttemptDetectedPayload),
    InjectionAttemptBlocked(InjectionAttemptBlockedPayload),
    SpotlightWrapped(SpotlightWrappedPayload),
    SpotlightBreached(SpotlightBreachedPayload),
    ToolDescriptionPinMismatch(ToolDescriptionPinMismatchPayload),
    CapabilityGateRejected(CapabilityGateRejectedPayload),
    ProvenanceTagged(ProvenanceTaggedPayload),
    PromptArmorVerdict(PromptArmorVerdictPayload),
    InjectionFixtureReplayed(InjectionFixtureReplayedPayload),
    // E-40 (SCHEMA_VERSION 134): orchestrator-rooted symbol-graph beacons.
    // 3 atomic variants. See `SymbolGraphQueriedPayload`,
    // `SymbolGraphCacheLoadedPayload`, `LspOracleConsultedPayload`.
    SymbolGraphQueried(SymbolGraphQueriedPayload),
    SymbolGraphCacheLoaded(SymbolGraphCacheLoadedPayload),
    LspOracleConsulted(LspOracleConsultedPayload),
    // E-44 (SCHEMA_VERSION 136): durability layer beacons.
    // 5 atomic variants. See `CheckpointWrittenPayload`,
    // `CheckpointResumedPayload`, `CheckpointMigratedPayload`,
    // `IdempotentDispatchSkippedPayload`, `DurabilityLockBlockedPayload`.
    CheckpointWritten(CheckpointWrittenPayload),
    CheckpointResumed(CheckpointResumedPayload),
    CheckpointMigrated(CheckpointMigratedPayload),
    IdempotentDispatchSkipped(IdempotentDispatchSkippedPayload),
    DurabilityLockBlocked(DurabilityLockBlockedPayload),
    // E-45 (SCHEMA_VERSION 137): cross-language drift detector beacons.
    // 3 atomic variants. See `CrossLangDriftDetectedPayload`,
    // `CrossLangPairScannedPayload`, `SchemaVersionMismatchPayload`.
    CrossLangDriftDetected(CrossLangDriftDetectedPayload),
    CrossLangPairScanned(CrossLangPairScannedPayload),
    SchemaVersionMismatch(SchemaVersionMismatchPayload),
    // E-49 (SCHEMA_VERSION 138): coding-agent metric suite (scope-lean).
    // 5 atomic variants.
    CodingAgentMetricsCaptured(CodingAgentMetricsCapturedPayload),
    SweBenchProRunStarted(SweBenchProRunStartedPayload),
    SweBenchProRunCompleted(SweBenchProRunCompletedPayload),
    RefineBenchScoreEmitted(RefineBenchScoreEmittedPayload),
    MetricsAggregationRun(MetricsAggregationRunPayload),
    // E-52 Tier-B (SCHEMA_VERSION 139): LSP-as-verifier diagnostic probe +
    // SCIP refresh beacons. 4 atomic variants. See
    // `DiagnosticProbeDispatchedPayload`, `DiagnosticProbeCompletedPayload`,
    // `DiagnosticProbeBlockPayload`, `ScipRefreshCompletedPayload`.
    DiagnosticProbeDispatched(DiagnosticProbeDispatchedPayload),
    DiagnosticProbeCompleted(DiagnosticProbeCompletedPayload),
    DiagnosticProbeBlock(DiagnosticProbeBlockPayload),
    ScipRefreshCompleted(ScipRefreshCompletedPayload),
    // E-53 (SCHEMA_VERSION 140): formal-verify pipeline boundary
    // beacon. Single atomic +1 bump per the C-52
    // DafnyVerifyCompleted single-variant precedent. See
    // `FormalVerifierCompletedPayload` for field docs.
    FormalVerifierCompleted(FormalVerifierCompletedPayload),
    // E-35 (SCHEMA_VERSION 141): structured-output substrate beacons.
    // 10 atomic variants for the XGrammar / INSTRUCTOR / CodeAct /
    // SEARCH/REPLACE / cloud strict-schema substrate. See per-payload
    // struct docs below. Python sidecar wiring + AnthropicClient
    // beta-header propagation are RUNPOD-PENDING; host-side emitters
    // are the codeact_sandbox dispatcher + the CLI subverb
    // `elai structured-output ...` + the contract test.
    XgrammarMaskApplied(XgrammarMaskAppliedPayload),
    InstructorModeActivated(InstructorModeActivatedPayload),
    GrammarModeActivated(GrammarModeActivatedPayload),
    CodeactActionExecuted(CodeactActionExecutedPayload),
    SearchReplaceTier1Hit(SearchReplaceTier1HitPayload),
    SearchReplaceCollisionFallback(SearchReplaceCollisionFallbackPayload),
    CloudStrictSchemaApplied(CloudStrictSchemaAppliedPayload),
    GrammarCacheLoaded(GrammarCacheLoadedPayload),
    StructuredEngineRejected(StructuredEngineRejectedPayload),
    LegacyJsonObjectRefused(LegacyJsonObjectRefusedPayload),
    // E-6 (SCHEMA_VERSION 148): MAST-grounded multi-agent failure-mode defenses.
    // Four payload variants per workflow/reasoning/E-6-decisions.md DECISION-5.
    // See payload struct definitions later in this file.
    AuditMastTagged(AuditMastTaggedPayload),
    IsoBudgetMeasured(IsoBudgetMeasuredPayload),
    PersonaReanchored(PersonaReanchoredPayload),
    CrossFamilyUnavailable(CrossFamilyUnavailablePayload),
    // E-7 (SCHEMA_VERSION 149): darwinian self-improvement substrate.
    // SINGLE-VARIANT scope-lean per Pattern-2; phase discriminator on
    // payload covers all 11 fire-points. Wire-dark unless
    // ELAI_DARWINIAN_LOOP_ENABLE=1 AND cfg(feature = "darwinian-evolve").
    DarwinianGenerationCompleted(DarwinianGenerationCompletedPayload),
    // E-3 (SCHEMA_VERSION 150): runtime handoff signal beacon.
    RuntimeHandoffSignal(RuntimeHandoffSignalPayload),
    // E-26 (SCHEMA_VERSION 151): inference-engines 2026 pack.
    SglangBackendRegistered(SglangBackendRegisteredPayload),
    XgrammarSchemaCompiled(XgrammarSchemaCompiledPayload),
    RuntimeColdStartObserved(RuntimeColdStartObservedPayload),
    // E-2 (SCHEMA_VERSION 152-153): streaming timing + UX pacing vote.
    StreamingTimingRecorded(StreamingTimingRecordedPayload),
    UxPacingVoteRecorded(UxPacingVoteRecordedPayload),
    // E-28 (SCHEMA_VERSION 154-156): attention-kernels 2026 pack.
    StreamingLlmEngaged(StreamingLlmEngagedPayload),
    SdpaBackendPinned(SdpaBackendPinnedPayload),
    YarnRopeScalingApplied(YarnRopeScalingAppliedPayload),
    // E-8 (SCHEMA_VERSION 162): self-evolving prompts substrate.
    PromptOptimizationCompleted(PromptOptimizationCompletedPayload),
    // E-10 (SCHEMA_VERSION 163): hostile-critic structural simplifications.
    CriticDecisionEvent(CriticDecisionEventPayload),
    // E-43 (SCHEMA_VERSION 164): agent skill systems telemetry.
    SkillSystemEvent(SkillSystemEventPayload),
    // E-9 (SCHEMA_VERSION 165): external agent-eval benchmark suite.
    BenchmarkRunObserved(BenchmarkRunObservedPayload),
    // E-19 (SCHEMA_VERSION 166): reasoning quality probe.
    ReasoningQualityProbe(ReasoningQualityProbePayload),
    // E-54 (SCHEMA_VERSION 167): eval-corpus repair pre-deploy.
    EvalCorpusEvent(EvalCorpusEventPayload),
    // E-55 (SCHEMA_VERSION 168): agentic RL + reward ladder substrate.
    RewardLadderEvent(RewardLadderEventPayload),
    // E-11 (SCHEMA_VERSION 169): SaaS prior-art catalog + recommender
    // substrate. Wire-dark unless ELAI_SAAS_PRIOR_ART_ENABLE=1.
    SaasPriorArtEvent(SaasPriorArtEventPayload),
    // E-22 (SCHEMA_VERSION 170): may-2026 github trending sweep substrate
    // (SKILL.md export + microVM sandbox + lazy MCP schema savings).
    // Wire-dark unless ELAI_GITHUB_TRENDING_ENABLE=1.
    GithubTrendingEvent(GithubTrendingEventPayload),
    // E-23 (SCHEMA_VERSION 171): 2026 developer-discourse positioning +
    // UX-guard substrate. Wire-dark unless ELAI_DEVELOPER_DISCOURSE_ENABLE=1.
    DeveloperDiscourseEvent(DeveloperDiscourseEventPayload),
    // E-25 (SCHEMA_VERSION 172): claude-code-architecture catalog substrate.
    // SINGLE-VARIANT Pattern-2 (24th application). Wire-dark unless
    // ELAI_CLAUDE_CODE_ARCH_ENABLE=1.
    ClaudeCodeArchitectureEvent(ClaudeCodeArchitectureEventPayload),
    // E-29 Tier-B (SCHEMA_VERSION 173): agentic-observability substrate.
    // SINGLE-VARIANT Pattern-2 (25th application). 4-arm kind covers
    // mast_failure_classified / replay_resume / signal_match /
    // eval_dataset_member. Wire-dark unless
    // ELAI_AGENTIC_OBSERVABILITY_ENABLE=1. See
    // workflow/reasoning/E-29-decisions.md.
    AgenticObservabilityEvent(AgenticObservabilityEventPayload),
    // E-31 (SCHEMA_VERSION 174): coding-model-leaderboard scraper substrate.
    // SINGLE-VARIANT Pattern-2 (26th application). 3-arm LeaderboardEventKind
    // discriminator covers scrape_started / scrape_completed / scrape_failed.
    // Wire-dark unless ELAI_CODING_MODEL_LEADERBOARD_ENABLE=1. See
    // workflow/reasoning/E-31-leaderboard-decisions.md.
    CodingModelLeaderboardEvent(CodingModelLeaderboardEventPayload),
    // E-38 (SCHEMA_VERSION 175): 2026 AI roadmap signals + EU AI Act
    // compliance pre-gate substrate. SINGLE-VARIANT Pattern-2 (27th
    // application). 5-arm RoadmapSignalKind discriminator covers
    // compliance_blocked / compliance_verified /
    // mcp_transport_version_logged / mrcr_unverified / posture_doc_stale.
    // Wire-dark unless ELAI_AI_ROADMAP_SIGNALS_ENABLE=1. See
    // workflow/reasoning/E-38-decisions.md.
    AiRoadmapSignalEvent(AiRoadmapSignalEventPayload),
    // E-39 (SCHEMA_VERSION 176): cost + token economics substrate.
    // SINGLE-VARIANT Pattern-2 (28th application). 8-arm CostEventKind
    // discriminator covers task_aborted_by_spend_cap /
    // velocity_cap_breached / automix_verifier_verdict /
    // cost_router_decision / cost_estimate_inaccurate /
    // free_tier_quota_approaching / free_tier_quota_exceeded /
    // layered_cap_manual_override. Wire-dark unless
    // ELAI_COST_TOKEN_ECONOMICS_ENABLE=1. See
    // workflow/reasoning/E-39-decisions.md.
    CostTokenEvent(CostTokenEventPayload),
    // E-50 (SCHEMA_VERSION 177): SaaS architecture pattern catalog
    // substrate. SINGLE-VARIANT Pattern-2 (29th application);
    // SaasArchPatternEventKind N-arm discriminator covers 11 fire-points
    // (gate / RLS-emit / scoped-token / MoR / realtime / async /
    // rate-limit / OTel-SLO / platform-unsupported / archetype-override).
    // Wire-dark unless ELAI_SAAS_ARCH_PATTERNS_ENABLE=1. See
    // workflow/reasoning/E-50-decisions.md.
    SaasArchPatternEvent(SaasArchPatternEventPayload),
    // E-4 (SCHEMA_VERSION 178): chat UX critique substrate.
    // SINGLE-VARIANT Pattern-2 (30th application);
    // ChatUxCritiqueKind 13-arm discriminator covers claim /
    // sycophancy / pinned-rules / re-anchor / doom-loop / completion /
    // clarifier / refusal / mid-stream / honest-correction fire-points.
    // Wire-dark unless ELAI_CHAT_UX_CRITIQUE_ENABLE=1.
    ChatUxCritiqueEvent(ChatUxCritiqueEventPayload),
    /// E-16 (SCHEMA_VERSION 179): VLM image-validation Pattern-2.
    VlmImageValidationEvent(VlmImageValidationEventPayload),
    /// E-20 (SCHEMA_VERSION 180): local-cloud routing Pattern-2.
    LocalCloudRoutingEvent(LocalCloudRoutingEventPayload),
    /// E-21 (SCHEMA_VERSION 181): peer-systems harness substrate
    /// Pattern-2 (33rd application). Carries [`PeerSystemsEventPayload`]
    /// with a 4-arm [`PeerSystemsEventKind`] discriminator + free-form
    /// `details_json`. Wire-dark unless `ELAI_PEER_SYSTEMS_ENABLE=1`.
    PeerSystemsEvent(PeerSystemsEventPayload),
    /// E-36 (SCHEMA_VERSION 182): document + vision RAG substrate
    /// Pattern-2 (34th application). Carries
    /// [`DocumentVisionRagEventPayload`] with a 5-arm
    /// [`DocumentVisionRagEventKind`] discriminator + free-form
    /// `details_json`. Wire-dark unless
    /// `ELAI_DOCUMENT_VISION_RAG_ENABLE=1`.
    DocumentVisionRagEvent(DocumentVisionRagEventPayload),
    /// E-42 (SCHEMA_VERSION 183): tree-sitter grammar health substrate
    /// Pattern-2 (35th application). Carries
    /// [`TreeSitterGrammarEventPayload`] with a 4-arm
    /// [`TreeSitterGrammarEventKind`] discriminator + free-form
    /// `details_json`. Wire-dark unless
    /// `ELAI_TREE_SITTER_GRAMMARS_ENABLE=1`.
    TreeSitterGrammarEvent(TreeSitterGrammarEventPayload),
    /// E-46 (SCHEMA_VERSION 184): planning-decomposition substrate
    /// Pattern-2 (36th application). Carries
    /// [`PlanningDecompositionEventPayload`] with an 8-arm
    /// [`PlanningDecompositionEventKind`] discriminator + free-form
    /// `details_json`. Wire-dark unless
    /// `ELAI_PLANNING_DECOMPOSITION_ENABLE=1`.
    PlanningDecompositionEvent(PlanningDecompositionEventPayload),
    /// E-48 (SCHEMA_VERSION 185): web-research orchestrator hardening
    /// substrate Pattern-2 (37th application). Carries
    /// [`WebResearchEventPayload`] with a 10-arm
    /// [`WebResearchEventKind`] discriminator + free-form
    /// `details_json`. Wire-dark unless
    /// `ELAI_WEB_RESEARCH_ORCHESTRATOR_ENABLE=1`.
    WebResearchEvent(WebResearchEventPayload),
    /// E-44 (SCHEMA_VERSION 186): durability observability substrate
    /// Pattern-2 (38th application). Carries
    /// [`DurabilityEventPayload`] with a 10-arm
    /// [`DurabilityEventKind`] discriminator + free-form
    /// `details_json`. Wire-dark unless
    /// `ELAI_DURABILITY_OBSERVABILITY_ENABLE=1`.
    DurabilityEvent(DurabilityEventPayload),
    /// E-31 (SCHEMA_VERSION 187): head_dim invariant verification substrate
    /// Pattern-2 (39th application). Carries
    /// [`HeadDimInvariantVerifiedPayload`] with a 2-arm
    /// [`HeadDimInvariantVerifiedKind`] discriminator + free-form
    /// `details_json`. Wire-dark unless
    /// `ELAI_HEAD_DIM_INVARIANT_SSB_ENABLE=1`.
    HeadDimInvariantVerified(HeadDimInvariantVerifiedPayload),
    /// E-3 (SCHEMA_VERSION 188): role-shown / model-hidden default
    /// telemetry substrate Pattern-2 (40th application). Carries
    /// [`RoleShownModelHiddenPayload`] (no Kind discriminator -- the MVP
    /// slice of TASK-E03-001 tracks a single user-facing toggle). Wire-dark
    /// unless `ELAI_ROLE_VISIBILITY_TELEMETRY_ENABLE=1`.
    RoleShownModelHidden(RoleShownModelHiddenPayload),
    /// E-49 (SCHEMA_VERSION 189): coding-agent metrics named-token
    /// substrate Pattern-2 (41st application). Carries
    /// [`CodingMetricNamedTokenPayload`] with a 3-arm
    /// [`CodingMetricNamedTokenKind`] discriminator + free-form
    /// `details_json`. Wire-dark unless
    /// `ELAI_CODING_METRIC_NAMED_TOKEN_ENABLE=1`.
    CodingMetricNamedToken(CodingMetricNamedTokenPayload),
    /// E-3 sub-phase 4A (SCHEMA_VERSION 190): runtime handoff event
    /// substrate Pattern-2 (42nd application). Carries
    /// [`RuntimeHandoffEventPayload`] with a 6-arm
    /// [`RuntimeHandoffEventKind`] discriminator + free-form
    /// `details_json`. Wire-dark unless
    /// `ELAI_E3_RUNTIME_HANDOFF_ENABLE=1`.
    E3RuntimeHandoffEvent(RuntimeHandoffEventPayload),
    /// E-31 sub-phase 6A (SCHEMA_VERSION 191): cloud-route decision
    /// event substrate Pattern-2 (43rd application). Carries
    /// [`RouteDecisionEventPayload`] with a 3-arm
    /// [`RouteDecisionEventKind`] discriminator + free-form
    /// `details_json`. Wire-dark unless
    /// `ELAI_ROUTE_DECISION_SSB_ENABLE=1`.
    RouteDecisionEvent(RouteDecisionEventPayload),
    /// E-47 (SCHEMA_VERSION 192): LoRA fine-tuning ship-gate substrate
    /// Pattern-2 (44th application). Carries
    /// [`LoraShipGateEventPayload`] with a 3-arm
    /// [`LoraShipGateEventPhase`] discriminator + free-form
    /// `details_json`. Wire-dark unless
    /// `ELAI_LORA_SHIP_GATE_ENABLE=1`.
    LoraShipGateEvent(LoraShipGateEventPayload),
    /// E-40 Tranche B-2 (SCHEMA_VERSION 193): SymbolGraph augmentation
    /// substrate. Pattern-2 (45th application). Carries
    /// [`SymbolGraphEventPayload`] with a 4-arm
    /// [`SymbolGraphEventKind`] discriminator + free-form
    /// `details_json`. Wire-dark unless
    /// `ELAI_SYMBOL_GRAPH_5TH_RRF_ENABLE=1` (or other per-event gates
    /// upstream).
    SymbolGraphEvent(SymbolGraphEventPayload),
    /// Tranche B-6 quad C-42 SHIPPABLE-NOW (SCHEMA_VERSION 194).
    /// See [`McpAuthzEventPayload`] + [`McpAuthzEventKind`].
    McpAuthzEvent(McpAuthzEventPayload),
    /// Tranche B-6 quad C-47 Pattern-2 collapse (SCHEMA_VERSION 194).
    /// See [`ToolCalibrationEventPayload`] + [`ToolCalibrationEventKind`].
    ToolCalibrationEvent(ToolCalibrationEventPayload),
    /// Tranche B-6 quad B-39 (SCHEMA_VERSION 194).
    /// See [`ReflectBudgetSoftWarnPayload`].
    ReflectBudgetSoftWarn(ReflectBudgetSoftWarnPayload),
    /// Tranche B-6 quad C-50 Pattern-2 collapse (SCHEMA_VERSION 194).
    /// See [`ContractCodingEventPayload`] + [`ContractCodingEventKind`].
    ContractCodingEvent(ContractCodingEventPayload),
    // F-1 (SCHEMA_VERSION 196): vibe-coding security gate blocked event.
    // See `VibeGateBlockedPayload` + `VibeGateBlockedKind`. Pattern-2
    // (single variant; VibeGateBlockedKind 4-arm discriminator).
    VibeGateBlocked(VibeGateBlockedPayload),
    // F-2a (SCHEMA_VERSION 196): security rule blocked flat beacon.
    // See `SecurityRuleBlockedPayload`.
    SecurityRuleBlocked(SecurityRuleBlockedPayload),
    // F-4 / Slice B-1 (SCHEMA_VERSION 197): test integrity verification event.
    // See `TestIntegrityEventPayload` + `TestIntegrityEventKind`. Pattern-2
    // (single variant; TestIntegrityEventKind 2-arm discriminator).
    TestIntegrityEvent(TestIntegrityEventPayload),
    // F-3p1 / Slice C1 (SCHEMA_VERSION 198): capability-flow tool-filter event.
    // See `ToolFilterPayload` + `ToolFilterEventKind`. Pattern-2
    // (single variant; ToolFilterEventKind 2-arm discriminator).
    ToolFilterEvent(ToolFilterPayload),
    // F-9 / Slice D1 (SCHEMA_VERSION 199): write-time dep slopsquat blocked.
    // See `WriteTimeSlopsquatBlockedPayload` + `WriteTimeSlopVerdictKind`.
    // Flat beacon (not Pattern-2); verdict_kind is a String field.
    WriteTimeSlopsquatBlocked(WriteTimeSlopsquatBlockedPayload),
    // F-10 / Slice A1 (SCHEMA_VERSION 200): security-class regression blocked.
    // See `SecurityClassRegressionBlockedPayload` + `LoopKind`.
    // Flat beacon (no Kind on the action-type enum); loop_kind is a payload field.
    SecurityClassRegressionBlocked(SecurityClassRegressionBlockedPayload),
    // F-2b / Slice A1 (SCHEMA_VERSION 201): insecure-rate keystone bench beacon.
    // See `InsecureRateMeasuredPayload`. Flat beacon (no Kind discriminator).
    InsecureRateMeasured(InsecureRateMeasuredPayload),
    // I-9 (SCHEMA_VERSION 263): error-amplification (alpha_t) telemetry beacon.
    // See `ErrorAmplificationMeasuredPayload`. Flat beacon (no Kind discriminator).
    ErrorAmplificationMeasured(ErrorAmplificationMeasuredPayload),
    // F-6 / Slice A1 (SCHEMA_VERSION 202): learned-skill library event.
    // See `SkillLibraryEventPayload` + `SkillLibraryEventKind`. Pattern-2
    // (single variant; SkillLibraryEventKind 3-arm discriminator).
    SkillLibraryEvent(SkillLibraryEventPayload),
    // F-5 / Slice A1 (SCHEMA_VERSION 203): staleness-quarantine broadcast.
    // See `StalenessQuarantineBroadcastPayload`. SINGLE FLAT beacon (no Kind
    // discriminator), mirroring LanguageGrammarStatus / MemoryFactSuperseded.
    StalenessQuarantineBroadcast(StalenessQuarantineBroadcastPayload),
    // F-8 / Slice A1 (SCHEMA_VERSION 204): SCF symbolic semantic conflict.
    // See `SymbolicConflictPayload`. SINGLE FLAT beacon (no Kind discriminator),
    // mirroring StalenessQuarantineBroadcast / SecurityClassRegressionBlocked.
    SymbolicConflictDetected(SymbolicConflictPayload),
    // F-7 / Slice A1 (SCHEMA_VERSION 205): CBR template Laplace-utility ranking.
    // See `TemplateUtilityRankedPayload`. SINGLE FLAT beacon (no Kind
    // discriminator), mirroring SymbolicConflictDetected /
    // StalenessQuarantineBroadcast.
    TemplateUtilityRanked(TemplateUtilityRankedPayload),
    // F-13 / Slice A1 (SCHEMA_VERSION 206): session-archive-on-schema-bump.
    // See `SessionArchivedPayload`. SINGLE FLAT beacon (no Kind discriminator);
    // store_kind ("ssb"|"session_dag") is a payload field, not a discriminator.
    SessionArchived(SessionArchivedPayload),
    // F-11 / Slice A1 (SCHEMA_VERSION 207): config-DGM admit-loop beacon.
    // See `DarwinianAdmitPayload`. Pattern-2 single-variant WITH a 3-arm
    // `DarwinianAdmitKind` discriminator (Admitted / RejectedByGuard /
    // LoopClosed) carried on the payload. Wire-dark unless
    // ELAI_DARWINIAN_LOOP_ENABLE=1 AND cfg(feature = "darwinian-evolve").
    DarwinianAdmit(DarwinianAdmitPayload),
    // G-plan (SCHEMA_VERSION 208-224): security + verifiability substrate.
    McpRugPullBlocked(McpRugPullBlockedPayload),
    TaintFlow(TaintFlowPayload),
    BlastRadiusBlocked(BlastRadiusBlockedPayload),
    TrifectaQuarantined(TrifectaQuarantinedPayload),
    RunArtifactEmitted(RunArtifactEmittedPayload),
    ReviewSurfaceScored(ReviewSurfaceScoredPayload),
    ProvenanceAttestationEmitted(ProvenanceAttestationEmittedPayload),
    SafetyDeltaMeasured(SafetyDeltaMeasuredPayload),
    ModelDriftSuspected(ModelDriftSuspectedPayload),
    SecretRedacted(SecretRedactedPayload),
    ClaimConfidenceScored(ClaimConfidenceScoredPayload),
    SteeringVectorRefreshTriggered(SteeringVectorRefreshTriggeredPayload),
    CanvasOpApplied(CanvasOpPayload),
    CanvasViewEvent(CanvasViewPayload),
    CanvasActivityEvent(CanvasActivityPayload),
    BatchLifecycle(BatchLifecyclePayload),
    // H-plan verifiability substrate (SCHEMA_VERSION 225+).
    DeflationGateEvaluated(DeflationGateEvaluatedPayload),
    LeakageBlocked(LeakageBlockedPayload),
    LookaheadDifferential(LookaheadDifferentialPayload),
    TradingNullTest(TradingNullTestPayload),
    RerunHashEquality(RerunHashEqualityPayload),
    SimRealParity(SimRealParityPayload),
    ResultManifestVerified(ResultManifestVerifiedPayload),
    DifferentialDivergence(DifferentialDivergencePayload),
    GateSelfValidation(GateSelfValidationPayload),
    RewardTampering(RewardTamperingPayload),
    HoldoutBudgetEvent(HoldoutBudgetPayload),
    DriftInterlock(DriftInterlockPayload),
    TrialRegistered(TrialRegisteredPayload),
    MucocoInconsistencyDetected(MucocoInconsistencyDetectedPayload),
    MucocoSkipReason(MucocoSkipReasonPayload),
    UserDecisionSuperseded(UserDecisionSupersededPayload),
    StalenessEvalMeasured(StalenessEvalMeasuredPayload),
    DiscoveryRatified(DiscoveryRatifiedPayload),
    DiscoveryRejected(DiscoveryRejectedPayload),
    /// H-32 (SCHEMA_VERSION 210): Tier-A code-grounded fact citation failed.
    FactValidationFailed(FactValidationFailedPayload),
    /// H-33 (SCHEMA_VERSION 211): memory governance event.
    MemoryGovernanceEvent(MemoryGovernanceEventPayload),
    /// H-34 (SCHEMA_VERSION 212): deferred MCP schema resolve on the STDIO lane.
    McpToolDeferred(McpToolDeferredPayload),
    /// H-34 (SCHEMA_VERSION 213): mcp-schema-budget bench result beacon.
    McpSchemaBudgetMeasured(McpSchemaBudgetMeasuredPayload),
    /// H-35 (SCHEMA_VERSION 214): session-rewind restore beacon.
    SessionRewound(SessionRewoundPayload),
    /// H-36 (SCHEMA_VERSION 215): dev-event notification beacon.
    AgentNotificationEmitted(AgentNotificationEmittedPayload),
    /// H-37 (SCHEMA_VERSION 216): plan-approval gate decision.
    PlanDecisionEvent(PlanDecisionPayload),
    /// H-38 (SCHEMA_VERSION 217): cost-confirmation gate outcome on task/pod stop.
    StopConfirmationEvent(StopConfirmationPayload),
    /// H-39 (SCHEMA_VERSION 218): abstention-aware scoring bench beacon.
    AbstentionScored(AbstentionScoredPayload),
    /// H-41 (SCHEMA_VERSION 219): checker-gap backlog beacon.
    CheckerGapDetected(CheckerGapDetectedPayload),
    /// H-42 (SCHEMA_VERSION 220): serial-depth-scaled GRC budget decision beacon.
    ReasoningBudgetScaled(ReasoningBudgetScaledPayload),
    /// H-43 (SCHEMA_VERSION 221): execution-grounded DGM fitness beacon.
    FitnessEvaluated(FitnessEvaluatedPayload),
    /// H-44 (SCHEMA_VERSION 222): behavioral cheat-detector beacon.
    TrapBehaviorDetected(TrapBehaviorDetectedPayload),
    /// H-45 (SCHEMA_VERSION 223): worst-case/variance-penalized aggregate promotion score.
    PromotionScoreAggregated(PromotionScoreAggregatedPayload),
    /// H-46 (SCHEMA_VERSION 224): HGM clade-metaproductivity parent selection beacon.
    CladeSelected(CladeSelectedPayload),
    /// H-47 (SCHEMA_VERSION 225): DGM eval firewall beacon.
    EvalFirewallEvent(EvalFirewallPayload),
    /// H-49 (SCHEMA_VERSION 226): reversible LLM edits beacon.
    EditHistoryEvent(EditHistoryEventPayload),
    /// H-51 (SCHEMA_VERSION 227): cross-session sleep-time handoff lifecycle beacon.
    HandoffLifecycleEvent(HandoffLifecyclePayload),
    /// H-52 (SCHEMA_VERSION 228): deterministic snippet->hunk finding-anchoring beacon.
    FindingAnchored(FindingAnchoredPayload),
    /// H-53 (SCHEMA_VERSION 229): citation pre-binding + constrained synthesis beacon.
    CitationAnchorsBound(CitationAnchorsBoundPayload),
    /// I-39 (SCHEMA_VERSION 230): per-plan state derived from three deterministic sources:
    /// wiring_contract_check.py verdict, SSB journal presence, and s-plan-audit-2026-06-04
    /// FINISH fragments. Emitted by `plan_status_cmd::run_plan_status` (REAL CLI entrypoint:
    /// `elai plan status`). Flat beacon. Carries [`PlanStateDerivedPayload`] with
    /// letter + session_id + derived_state + evidence_anchor. Default-ON.
    /// TOTAL_VARIANTS 540 -> 541.
    PlanStateDerived(PlanStateDerivedPayload),
    /// I-45 (SCHEMA_VERSION 231): objective-hash sealed-receipt beacon. Pattern-2
    /// single-variant 3-arm [`ObjectiveSealedKind`]. Emitted from `rra::build_rra`
    /// (Sealed arm) and from `rra::verify_artifact_with_objective` (MutationDetected arm).
    /// Default-ON; kill-switch `ELAI_OBJECTIVE_SEAL_DISABLE`. Fail-soft.
    /// TOTAL_VARIANTS 541 -> 542.
    ObjectiveSealed(ObjectiveSealedPayload),
    /// I-34 (SCHEMA_VERSION 234): CRANE hybrid grammar-pin gate beacon. Pattern-2
    /// single-variant 3-arm [`GrammarGateEventKind`]:
    /// - `CranePassed`: constrained decode completed, gate eligibility check passed.
    /// - `PinViolation`: grammar file's blake3 digest mismatches the pinned manifest value.
    /// - `PostParseRejected`: post-parse re-validation detected silent non-compliance.
    ///
    /// Emitted by `orchestrator::grammar_gate::emit_grammar_gate_event`.
    /// Default-ON; no kill-switch (mandatory per IC-18 + landmine #7/#8).
    /// TOTAL_VARIANTS 542 -> 543.
    GrammarGateEvent(GrammarGateEventPayload),
    /// I-38 (SCHEMA_VERSION 235): QDELTA defective-diff rate flat beacon.
    /// Mirrors [`SafetyDeltaMeasured`] shape. TOTAL_VARIANTS 543 -> 544.
    QdeltaMeasured(QdeltaMeasuredPayload),
    /// I-43 (SCHEMA_VERSION 236): reproducibility-claim scoper + kill-gate.
    /// Pattern-2 single-variant 3-arm [`ClaimScopeMeasuredKind`]:
    /// - `PilotDelta`: reduced-N pilot signal + reviewer-reaching-rate trap detector.
    /// - `CrossLaneDivergence`: local-vs-cloud action-stream divergence on security paths.
    /// - `ClaimScoped`: consolidated go/no-go verdict with honest claim wording.
    /// TOTAL_VARIANTS 544 -> 545.
    ClaimScopeMeasured(ClaimScopeMeasuredPayload),
    /// I-10 (SCHEMA_VERSION 237): prefix invariance + TTL tiering + cache-hit hard guard.
    /// Pattern-2 single-variant 3-arm [`CacheEconomicsEventKind`]:
    /// - `TtlPlanned`: planner selected a TTL tier for a cloud call.
    /// - `HitRateGuardTripped`: unexplained miss-rate exceeded threshold (>10%).
    /// - `PrefixDiffDumped`: guard detected a prefix mutation; blake3 byte-range dumped.
    /// TOTAL_VARIANTS 545 -> 546.
    CacheEconomicsEvent(CacheEconomicsEventPayload),
    /// I-8 (SCHEMA_VERSION 238): aletheia voluntary abort + dead-end hash pruning + halting bound.
    /// Pattern-2 single-variant 3-arm [`TrajectoryAbortedKind`]:
    /// - `Voluntary`: model called `acknowledge_infeasible`.
    /// - `DeadEndHash`: pre-dispatch fingerprint match pruned isomorphic re-attempt (0 tokens).
    /// - `HaltingBound`: I-17 watchdog hook (Wave-1 stub registered here).
    /// TOTAL_VARIANTS 546 -> 547.
    TrajectoryAborted(TrajectoryAbortedPayload),
    /// I-1 (SCHEMA_VERSION 239): directive supervisor enforcement-plane outcome.
    /// Pattern-2 single-variant 3-arm [`DirectiveExecutedKind`]:
    /// - `Executed`: gates passed; side effect ran cleanly.
    /// - `Discarded`: a gate blocked the directive; side effect never ran.
    /// - `CompensationFired`: side effect crashed; E-44 saga rollback fired.
    /// TOTAL_VARIANTS 547 -> 548.
    DirectiveExecuted(DirectiveExecutedPayload),
    /// I-33 (SCHEMA_VERSION 240): mechanical decision governance primitive fired.
    /// Pattern-2 single-variant 3-arm [`GovernancePrimitiveFiredKind`].
    /// TOTAL_VARIANTS 548 -> 549.
    GovernancePrimitiveFired(GovernancePrimitiveFiredPayload),
    /// I-32 (SCHEMA_VERSION 241): structural role-scope contract applied.
    /// Pattern-2 single-variant 3-arm [`RoleContractAppliedKind`].
    /// TOTAL_VARIANTS 549 -> 550.
    RoleContractApplied(RoleContractAppliedPayload),
    /// I-35 (SCHEMA_VERSION 242): temporal tool-sequence enforcement verdict.
    /// Pattern-2 single-variant 3-arm [`ToolSequenceVerdictKind`].
    /// TOTAL_VARIANTS 550 -> 551.
    ToolSequenceVerdict(ToolSequenceVerdictPayload),
    /// I-41 (SCHEMA_VERSION 243): spec-time critique lenses advisory beacon.
    /// Pattern-2 single-variant 3-arm [`CritiqueLensFindingKind`].
    /// TOTAL_VARIANTS 551 -> 552. Advisory-only; `advisory=true` invariant.
    CritiqueLensFinding(CritiqueLensFindingPayload),
    /// I-46 (SCHEMA_VERSION 244): bash interceptor redirect/fixup/prefix outcome.
    /// Pattern-2 single-variant 3-arm [`BashRedirectedKind`].
    /// TOTAL_VARIANTS 552 -> 553.
    BashRedirected(BashRedirectedPayload),
    /// I-47 (SCHEMA_VERSION 245): verifier-input hardening + dispatch rule injection beacon.
    /// Pattern-2 single-variant 3-arm [`PromptHardeningAppliedKind`].
    /// TOTAL_VARIANTS 553 -> 554.
    PromptHardeningApplied(PromptHardeningAppliedPayload),
    /// I-2 (SCHEMA_VERSION 246): CoW workspace isolation barrier outcome.
    /// Pattern-2 single-variant 4-arm [`BranchWorkspaceEventKind`].
    /// TOTAL_VARIANTS 554 -> 555.
    BranchWorkspaceEvent(BranchWorkspaceEventPayload),
    /// I-5 (SCHEMA_VERSION 247): per-branch context rebuild outcome.
    /// Pattern-2 single-variant 3-arm [`ContextPrunedKind`].
    /// TOTAL_VARIANTS 555 -> 556.
    ContextPruned(ContextPrunedPayload),
    /// I-11 (SCHEMA_VERSION 248): typed versioned condensate receipt for handoffs.
    /// Pattern-2 single-variant 3-arm [`BeliefStateEventKind`].
    /// TOTAL_VARIANTS 556 -> 557.
    BeliefStateEvent(BeliefStateEventPayload),
    /// I-13 (SCHEMA_VERSION 249): write-time memory guard outcome.
    /// Pattern-2 single-variant 3-arm [`MemoryGuardEventKind`].
    /// TOTAL_VARIANTS 557 -> 558.
    MemoryGuardEvent(MemoryGuardEventPayload),
    /// I-14 (SCHEMA_VERSION 250): deterministic context-compression middleware receipt.
    /// Pattern-2 single-variant 3-arm [`ContextTransformKind`].
    /// TOTAL_VARIANTS 558 -> 559.
    ContextTransformReceipt(ContextTransformReceiptPayload),
    /// I-12 (SCHEMA_VERSION 251): evidence-gated playbook curator outcome beacon.
    /// Pattern-2 single-variant 4-arm [`PlaybookEventKind`].
    /// TOTAL_VARIANTS 559 -> 560.
    PlaybookEvent(PlaybookEventPayload),
    /// I-17 (SCHEMA_VERSION 252): tri-layer evaluation-stack per-branch verdict.
    /// Pattern-2 single-variant 4-arm [`TriLayerVerdictKind`].
    /// TOTAL_VARIANTS 560 -> 561.
    TriLayerVerdict(TriLayerVerdictPayload),
    /// I-18 (SCHEMA_VERSION 253): per-row rolling blake3 chain beacon.
    /// Pattern-2 single-variant 3-arm [`RowChainSealedKind`].
    /// TOTAL_VARIANTS 561 -> 562.
    RowChainSealed(RowChainSealedPayload),
    /// I-19 (SCHEMA_VERSION 254): TDD enforcer preflight gate event.
    /// Pattern-2 single-variant 3-arm [`TddPreflightBlockedKind`].
    /// Wire-dark unless `ELAI_TDD_PREFLIGHT_ENABLE=1`.
    /// TOTAL_VARIANTS 562 -> 563.
    TddPreflightBlocked(TddPreflightBlockedPayload),
    /// I-20 (SCHEMA_VERSION 255): budgeted mutation scheduler per-cone outcome.
    /// Pattern-2 single-variant 4-arm [`MutationBudgetedRunKind`].
    /// Default-ON; kill-switch `ELAI_MUTATION_BUDGET_DISABLE=1`.
    /// TOTAL_VARIANTS 563 -> 564.
    MutationBudgetedRun(MutationBudgetedRunPayload),

    /// I-37 (SCHEMA_VERSION 257): per-EARS-requirement verified-status event.
    /// Pattern-2 single-variant 3-arm [`RequirementVerifiedKind`].
    /// Additive emit (independent of the blocking gate flag).
    /// TOTAL_VARIANTS 565 -> 566.
    RequirementVerified(RequirementVerifiedPayload),

    /// I-21 (SCHEMA_VERSION 256) -- sprint-contract + ralph-loop lifecycle event.
    /// Pattern-2 single-variant 5-arm [`SprintContractEventKind`].
    SprintContractEvent(SprintContractEventPayload),

    /// I-44 (SCHEMA_VERSION 258): zero-retention-after-verification purge receipt.
    /// Pattern-2 single-variant 3-arm [`RetentionPurgedKind`].
    /// Destructive + opt-in (`--zero-retention` + `ELAI_ZERO_RETENTION_ENABLE`).
    /// TOTAL_VARIANTS 566 -> 567.
    RetentionPurged(RetentionPurgedPayload),
    /// I-52 (SCHEMA_VERSION 259): agent-callable checkpoint tool outcome event.
    /// Pattern-2 single-variant 4-arm [`CheckpointToolEventKind`].
    /// Default-ON over H-49 (`ELAI_TIMETRAVEL_DISABLE=1` kill-switch).
    /// TOTAL_VARIANTS 567 -> 568.
    CheckpointToolEvent(CheckpointToolEventPayload),
    /// I-54 (SCHEMA_VERSION 260): agent-callable structural (AST-aware) edit tool
    /// outcome event. Pattern-2 single-variant 5-arm [`StructuralEditAppliedKind`].
    /// Reuses the shipped tree-sitter grammars (one version source; ast-grep
    /// BANNED). TOTAL_VARIANTS 568 -> 569.
    StructuralEditApplied(StructuralEditAppliedPayload),
    /// I-4 (SCHEMA_VERSION 261): evidence-ranked search-ladder branch lifecycle
    /// event (Wave-4 KEYSTONE). Pattern-2 single-variant 4-arm
    /// [`SearchNodeEventKind`]. Default-OFF (`ELAI_SEARCH_LADDER_ENABLE`, R9
    /// experiment). TOTAL_VARIANTS 569 -> 570.
    SearchNodeEvent(SearchNodeEventPayload),
    /// I-7 (SCHEMA_VERSION 262): cascading escalation waterfall lifecycle event
    /// (Wave-4). Pattern-2 single-variant 4-arm [`EscalationCascadeKind`].
    /// Default-OFF (`ELAI_ESCALATION_WATERFALL_ENABLE`, R9 experiment).
    /// TOTAL_VARIANTS 570 -> 571.
    EscalationCascade(EscalationCascadePayload),
    /// I-22 (SCHEMA_VERSION 264): delta-progress shaped-reward scorer event.
    /// Pattern-2 single-variant 3-arm [`ProgressRewardScoredKind`]:
    /// Delta / FirstErrorTruncated / DecayApplied.
    /// Wire-dark unless `ELAI_PROGRESS_REWARD_ENABLE=1`.
    /// TOTAL_VARIANTS 572 -> 573.
    ProgressRewardScored(ProgressRewardScoredPayload),
    /// I-24 (SCHEMA_VERSION 266): isomorphic canary anti-gaming event.
    /// Pattern-2 single-variant 3-arm [`PrmCanaryEventKind`]:
    /// CanaryInjected / StyleBiasDetected / ScorerNeutralized.
    /// Wire-dark unless `ELAI_CANARY_INJECTOR_ENABLE=1`.
    /// R8: canary pairs frozen in `fixtures/canary_pairs.json`.
    /// TOTAL_VARIANTS 573 -> 574.
    PrmCanaryEvent(PrmCanaryEventPayload),
    /// I-27 (SCHEMA_VERSION 265): verified-outcome-rate vs compute convergence
    /// curve measurement flat beacon. Emitted by `bench_convergence::run_inner`
    /// from `CliAction::BenchConvergence` dispatch.
    /// TOTAL_VARIANTS 574 -> 575.
    ConvergenceCurveMeasured(ConvergenceCurveMeasuredPayload),
    /// I-36 (SCHEMA_VERSION 268): per-(role,backend) compiled-prompt artifact event.
    /// Pattern-2 single-variant 3-arm [`PromptCompiledKind`]:
    /// - `Compiled`: a promoted compiled variant substituted at the adapter boundary.
    /// - `TransferGapMeasured`: source-vs-compiled fixture pass-rate delta ledgered (GD-2).
    /// - `Rolledback`: compiled variant regressed fixture floor; variant removed from store.
    ///
    /// Default-ON (no-op pass-through when no variant is stored).
    /// Kill-switch: ELAI_PROMPT_COMPILE_DISABLE=1.
    /// TOTAL_VARIANTS 575 -> 576.
    PromptCompiled(PromptCompiledPayload),
    /// I-50 (SCHEMA_VERSION 268): subscription auth broker lifecycle event.
    ///
    /// Pattern-2 single-variant 4-arm [`AuthBrokerEventKind`]:
    /// - `TokenAcquired`: a valid credential was resolved (env-key, saved token, or new
    ///   OAuth exchange). The row names `provider_id` only -- no token material logged.
    /// - `Refreshed`: an expired saved token was refreshed and re-persisted.
    /// - `ProviderLinked`: a new provider credential was registered with the broker.
    /// - `RefreshFailed`: a refresh attempt failed; the typed error is surfaced to the
    ///   caller as `ApiError`, never swallowed silently.
    ///
    /// Default-ON. Wire-dark at the SSB-emit seam in `runtime_cmd.rs` until
    /// `Arc<Ssb>` is threaded through (WIRE-DARK[I-50]). The emit path is proven
    /// reachable via the i50 contract test (real TempDir SQLite).
    /// TOTAL_VARIANTS 576 -> 577.
    AuthBrokerEvent(AuthBrokerEventPayload),
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
            Self::ToolDispatch(_) => SsbActionType::ToolDispatch,
            Self::ValidationTier(_) => SsbActionType::ValidationTier,
            Self::FailureClassified(_) => SsbActionType::FailureClassified,
            Self::GitCommit(_) => SsbActionType::GitCommit,
            Self::DairaTrace(_) => SsbActionType::DairaTrace,
            Self::SystemProbe(_) => SsbActionType::SystemProbe,
            Self::FeasibilityReport(_) => SsbActionType::FeasibilityReport,
            Self::NovelStepDetected(_) => SsbActionType::NovelStepDetected,
            Self::StubRun(_) => SsbActionType::StubRun,
            Self::GateDecision(_) => SsbActionType::GateDecision,
            Self::TaskSplit(_) => SsbActionType::TaskSplit,
            Self::EscalationFired(_) => SsbActionType::EscalationFired,
            Self::ModeChanged(_) => SsbActionType::ModeChanged,
            Self::ProposalGenerated(_) => SsbActionType::ProposalGenerated,
            Self::ProposalApplied(_) => SsbActionType::ProposalApplied,
            Self::ProposalRejected(_) => SsbActionType::ProposalRejected,
            Self::MsiUpdate(_) => SsbActionType::MsiUpdate,
            Self::ConfigArchived(_) => SsbActionType::ConfigArchived,
            Self::ProfileSelected(_) => SsbActionType::ProfileSelected,
            Self::FsmTransition(_) => SsbActionType::FsmTransition,
            Self::MicroReflection(_) => SsbActionType::MicroReflection,
            Self::TelescopeCollapse(_) => SsbActionType::TelescopeCollapse,
            Self::ConfigWarning(_) => SsbActionType::ConfigWarning,
            Self::ConstraintsInjected(_) => SsbActionType::ConstraintsInjected,
            Self::InterventionResolved(_) => SsbActionType::InterventionResolved,
            Self::DkgQuerierSwapped(_) => SsbActionType::DkgQuerierSwapped,
            Self::SteeringComputed(_) => SsbActionType::SteeringComputed,
            Self::SteeringComputeFailed(_) => SsbActionType::SteeringComputeFailed,
            Self::McpServerConnected(_) => SsbActionType::McpServerConnected,
            Self::McpToolCalled(_) => SsbActionType::McpToolCalled,
            Self::McpServerDisconnected(_) => SsbActionType::McpServerDisconnected,
            Self::ChatMessage(_) => SsbActionType::ChatMessage,
            Self::ChatResponse(_) => SsbActionType::ChatResponse,
            Self::ExecutionModeChanged(_) => SsbActionType::ExecutionModeChanged,
            Self::ContextBudgetComputed(_) => SsbActionType::ContextBudgetComputed,
            Self::SsbFilteredByOverlap(_) => SsbActionType::SsbFilteredByOverlap,
            Self::HookLatencyViolation(_) => SsbActionType::HookLatencyViolation,
            Self::ResearchPathChosen(_) => SsbActionType::ResearchPathChosen,
            Self::GroundingCacheEvent(_) => SsbActionType::GroundingCacheEvent,
            Self::CircuitBreakerStateChange(_) => SsbActionType::CircuitBreakerStateChange,
            Self::BubbleUpTyped(_) => SsbActionType::BubbleUpTyped,
            Self::IntegrationCheckpointSuccess(_) => SsbActionType::IntegrationCheckpointSuccess,
            Self::DagRepairRound(_) => SsbActionType::DagRepairRound,
            Self::ConceptDecompositionNode(_) => SsbActionType::ConceptDecompositionNode,
            // B-7: pipeline factory producer wiring.
            Self::TreePositionChanged(_) => SsbActionType::TreePositionChanged,
            Self::FailureRecorded(_) => SsbActionType::FailureRecorded,
            Self::PackageRefVerified(_) => SsbActionType::PackageRefVerified,
            Self::TechStackDecided(_) => SsbActionType::TechStackDecided,
            // B-6: orchestration producer wiring.
            Self::EvidenceGateEvaluated(_) => SsbActionType::EvidenceGateEvaluated,
            Self::SupervisionEvent(_) => SsbActionType::SupervisionEvent,
            Self::BspSuperstep(_) => SsbActionType::BspSuperstep,
            Self::AgentLifecycle(_) => SsbActionType::AgentLifecycle,
            Self::AntiSpiralHit(_) => SsbActionType::AntiSpiralHit,
            Self::CloudSpend(_) => SsbActionType::CloudSpend,
            Self::EscalateToCloud(_) => SsbActionType::EscalateToCloud,
            // B-23 Phase 4H Group A.
            Self::DataTableGrew(_) => SsbActionType::DataTableGrew,
            Self::L3Proposal(_) => SsbActionType::L3Proposal,
            Self::ConvergenceStagnant(_) => SsbActionType::ConvergenceStagnant,
            Self::ConfigOscillation(_) => SsbActionType::ConfigOscillation,
            Self::RegressionDetected(_) => SsbActionType::RegressionDetected,
            Self::EvolutionWithheld(_) => SsbActionType::EvolutionWithheld,
            // B-39 chat reflect pipeline.
            Self::ReflectTriaged(_) => SsbActionType::ReflectTriaged,
            Self::ReflectCandidate(_) => SsbActionType::ReflectCandidate,
            Self::ReflectAuditVerdict(_) => SsbActionType::ReflectAuditVerdict,
            Self::ReflectRefined(_) => SsbActionType::ReflectRefined,
            // B-24 Phase 4I Group C1: media validator beacons.
            Self::MediaValidationStarted(_) => SsbActionType::MediaValidationStarted,
            Self::MediaTier1Complete(_) => SsbActionType::MediaTier1Complete,
            Self::MediaTier2Complete(_) => SsbActionType::MediaTier2Complete,
            Self::RenderComplete(_) => SsbActionType::RenderComplete,
            Self::VisualFeedbackIteration(_) => SsbActionType::VisualFeedbackIteration,
            Self::MediaHumanReviewRequired(_) => SsbActionType::MediaHumanReviewRequired,
            // B-28 E-3.
            Self::GenerationModeSelected(_) => SsbActionType::GenerationModeSelected,
            // B-28 E-4.
            Self::StreamingEarlyTerminated(_) => SsbActionType::StreamingEarlyTerminated,
            // B-28 WIRING-1 (3 new variants).
            Self::PipelinePreAssembled(_) => SsbActionType::PipelinePreAssembled,
            Self::DependencyResolved(_) => SsbActionType::DependencyResolved,
            Self::MicroStepCreated(_) => SsbActionType::MicroStepCreated,
            // B-30 (SCHEMA_VERSION 36): language grammar registry status.
            Self::LanguageGrammarStatus(_) => SsbActionType::LanguageGrammarStatus,
            // B-29 Phase 5 (SCHEMA_VERSION 37): symbol-graph tool call row.
            Self::SymbolQuery(_) => SsbActionType::SymbolQuery,
            // B-32 (SCHEMA_VERSION 38): minified-file chunking row.
            Self::MinifiedFileChunked(_) => SsbActionType::MinifiedFileChunked,
            // B-31.5 (SCHEMA_VERSION 39): 5 deferred B-31 SSB variants.
            Self::IndexCheckpoint(_) => SsbActionType::IndexCheckpoint,
            Self::IndexResume(_) => SsbActionType::IndexResume,
            Self::IndexCancel(_) => SsbActionType::IndexCancel,
            Self::LockAcquired(_) => SsbActionType::LockAcquired,
            Self::LockContested(_) => SsbActionType::LockContested,
            // B-33 follow-up (SCHEMA_VERSION 40): symbol-graph build event row.
            Self::SymbolGraphUpdated(_) => SsbActionType::SymbolGraphUpdated,
            // B-40 (SCHEMA_VERSION 41): FRAMING substate beacons.
            Self::IdeaFramingStarted(_) => SsbActionType::IdeaFramingStarted,
            Self::ProjectClassClassified(_) => SsbActionType::ProjectClassClassified,
            Self::CriticRoundComplete(_) => SsbActionType::CriticRoundComplete,
            Self::IdeaFramed(_) => SsbActionType::IdeaFramed,
            Self::TradeoffCardEmitted(_) => SsbActionType::TradeoffCardEmitted,
            // B-34 (SCHEMA_VERSION 42): entry-point detection + artifact pipeline beacons.
            Self::EntryPointsDetected(_) => SsbActionType::EntryPointsDetected,
            Self::ArtifactsIndexed(_) => SsbActionType::ArtifactsIndexed,
            Self::ArtifactSearched(_) => SsbActionType::ArtifactSearched,
            Self::ArtifactStale(_) => SsbActionType::ArtifactStale,
            // B-38 (SCHEMA_VERSION 43): ambiguity gate beacons.
            Self::AmbiguityScored(_) => SsbActionType::AmbiguityScored,
            Self::AmbiguityFloorFailed(_) => SsbActionType::AmbiguityFloorFailed,
            Self::AmbiguityStreakAdvanced(_) => SsbActionType::AmbiguityStreakAdvanced,
            // B-42 (SCHEMA_VERSION 44): cosmos tier knowledge worker beacons.
            Self::CosmosRefreshStarted(_) => SsbActionType::CosmosRefreshStarted,
            Self::CosmosArtifactUpdated(_) => SsbActionType::CosmosArtifactUpdated,
            Self::CosmosArtifactDecayed(_) => SsbActionType::CosmosArtifactDecayed,
            Self::CosmosGroundingUsed(_) => SsbActionType::CosmosGroundingUsed,
            // B-41 (SCHEMA_VERSION 45): per-level auditor mode beacons.
            Self::PerLevelAuditStarted(_) => SsbActionType::PerLevelAuditStarted,
            Self::PerLevelAuditVerdict(_) => SsbActionType::PerLevelAuditVerdict,
            Self::PerLevelAuditCascade(_) => SsbActionType::PerLevelAuditCascade,
            // B-37C (SCHEMA_VERSION 46): multi-runtime backend beacons.
            Self::RuntimeFallback(_) => SsbActionType::RuntimeFallback,
            Self::RuntimeHealthChanged(_) => SsbActionType::RuntimeHealthChanged,
            Self::CliSubprocessSpawned(_) => SsbActionType::CliSubprocessSpawned,
            Self::CliSubprocessKilled(_) => SsbActionType::CliSubprocessKilled,
            // E-51 (SCHEMA_VERSION 96): edge-inference beacons.
            Self::MlxSubprocessSpawned(_) => SsbActionType::MlxSubprocessSpawned,
            Self::MlxSubprocessKilled(_) => SsbActionType::MlxSubprocessKilled,
            Self::LlamaCppSubprocessSpawned(_) => SsbActionType::LlamaCppSubprocessSpawned,
            Self::LlamaCppSubprocessKilled(_) => SsbActionType::LlamaCppSubprocessKilled,
            Self::MlxApplePlatformDetected(_) => SsbActionType::MlxApplePlatformDetected,
            Self::LlamaCppEpDetected(_) => SsbActionType::LlamaCppEpDetected,
            Self::RuntimeChainBuilt(_) => SsbActionType::RuntimeChainBuilt,
            // E-17 SP2 (SCHEMA_VERSION 97): RAG retrieval-quality beacons.
            Self::DeepPoolCapped(_) => SsbActionType::DeepPoolCapped,
            Self::RrfQualityGateFired(_) => SsbActionType::RrfQualityGateFired,
            Self::HydeSkippedCodeQuery(_) => SsbActionType::HydeSkippedCodeQuery,
            Self::RrfComponentWeights(_) => SsbActionType::RrfComponentWeights,
            // C-1 (SCHEMA_VERSION 47): knowledge + reasoning producer beacons.
            Self::GrcStepEmitted(_) => SsbActionType::GrcStepEmitted,
            Self::GroundingSweepResult(_) => SsbActionType::GroundingSweepResult,
            Self::CsqEvaluated(_) => SsbActionType::CsqEvaluated,
            Self::AuditorRubberStampRejected(_) => SsbActionType::AuditorRubberStampRejected,
            Self::PerRoleDecodingApplied(_) => SsbActionType::PerRoleDecodingApplied,
            Self::HydeQuery(_) => SsbActionType::HydeQuery,
            Self::VectorCacheHitRate(_) => SsbActionType::VectorCacheHitRate,
            Self::QueryTrace(_) => SsbActionType::QueryTrace,
            // C-2 (SCHEMA_VERSION 48): orchestration + cosmos + stagnation beacons.
            Self::PhaseNodeReady(_) => SsbActionType::PhaseNodeReady,
            Self::StepNodeReady(_) => SsbActionType::StepNodeReady,
            Self::BabyStepReady(_) => SsbActionType::BabyStepReady,
            Self::DagRepairEscalated(_) => SsbActionType::DagRepairEscalated,
            Self::AgentStatusChanged(_) => SsbActionType::AgentStatusChanged,
            Self::StagnationSpinningDetected(_) => SsbActionType::StagnationSpinningDetected,
            Self::StagnationOscillationDetected(_) => SsbActionType::StagnationOscillationDetected,
            Self::LateralThinkingActivated(_) => SsbActionType::LateralThinkingActivated,
            Self::GoalDriftWarning(_) => SsbActionType::GoalDriftWarning,
            Self::WatchdogTimeout(_) => SsbActionType::WatchdogTimeout,
            // C-7 (SCHEMA_VERSION 49): reviewer verdict beacon.
            Self::ChangeDecision(_) => SsbActionType::ChangeDecision,
            // C-8 (SCHEMA_VERSION 50): search-mode cross-process bridge.
            Self::SearchModeChanged(_) => SsbActionType::SearchModeChanged,
            // C-5 (SCHEMA_VERSION 51): privacy-redaction event.
            Self::RedactionApplied(_) => SsbActionType::RedactionApplied,
            // C-10 (SCHEMA_VERSION 52): anti-spiral lateral-thinking + goal-drift Block.
            Self::LateralThinkingSucceeded(_) => SsbActionType::LateralThinkingSucceeded,
            Self::LateralThinkingExhausted(_) => SsbActionType::LateralThinkingExhausted,
            Self::GoalDriftBlock(_) => SsbActionType::GoalDriftBlock,
            // C-13 (SCHEMA_VERSION 53): WIP commit squash completion.
            Self::WipSquashComplete(_) => SsbActionType::WipSquashComplete,
            // C-22 (SCHEMA_VERSION 54): feasibility-probe completeness beacons.
            Self::AlternativeExplored(_) => SsbActionType::AlternativeExplored,
            Self::FeasibilityEscalated(_) => SsbActionType::FeasibilityEscalated,
            // C-23 (SCHEMA_VERSION 55): SessionDag branch-UI beacons.
            Self::BranchForked(_) => SsbActionType::BranchForked,
            Self::BranchMerged(_) => SsbActionType::BranchMerged,
            Self::BranchClosed(_) => SsbActionType::BranchClosed,
            // C-25 (SCHEMA_VERSION 56): compiled-pipeline + template-crystallizer
            // + background-task beacons.
            Self::PlanSkeleton(_) => SsbActionType::PlanSkeleton,
            Self::CompiledPipelineFired(_) => SsbActionType::CompiledPipelineFired,
            Self::TemplateCrystallized(_) => SsbActionType::TemplateCrystallized,
            Self::BackgroundTaskStarted(_) => SsbActionType::BackgroundTaskStarted,
            Self::BackgroundTaskComplete(_) => SsbActionType::BackgroundTaskComplete,
            Self::DecompileTriggered(_) => SsbActionType::DecompileTriggered,
            // C-26 (SCHEMA_VERSION 58): constrained-decoding + futility primitives.
            Self::FutilityDetected(_) => SsbActionType::FutilityDetected,
            Self::FutilityOutputStagnation(_) => SsbActionType::FutilityOutputStagnation,
            // C-27 (SCHEMA_VERSION 59): Testing-trust T(-1) + T-0 beacons.
            Self::WiringEvidence(_) => SsbActionType::WiringEvidence,
            Self::WiringViolation(_) => SsbActionType::WiringViolation,
            Self::DeterminismDrift(_) => SsbActionType::DeterminismDrift,
            // C-28 (SCHEMA_VERSION 60): Testing-trust T-1 mutation testing.
            Self::MutationSurvived(_) => SsbActionType::MutationSurvived,
            // C-29 (SCHEMA_VERSION 61): Testing-trust T-2 + T-3 beacons.
            Self::IntentConfirmed(_) => SsbActionType::IntentConfirmed,
            Self::MrViolated(_) => SsbActionType::MrViolated,
            // C-30 (SCHEMA_VERSION 62): Surgical Mode + Oracle Mode beacons.
            Self::SurgicalPlanCreated(_) => SsbActionType::SurgicalPlanCreated,
            Self::SurgicalLocWarning(_) => SsbActionType::SurgicalLocWarning,
            Self::OracleCacheHit(_) => SsbActionType::OracleCacheHit,
            Self::OracleCacheMiss(_) => SsbActionType::OracleCacheMiss,
            Self::ProjectDetectionSignal(_) => SsbActionType::ProjectDetectionSignal,
            // C-32 (SCHEMA_VERSION 63): Code-Completeness Layer 2 beacons.
            Self::SkeletonGenerated(_) => SsbActionType::SkeletonGenerated,
            Self::SkeletonFlushed(_) => SsbActionType::SkeletonFlushed,
            Self::MiniCriticVerdict(_) => SsbActionType::MiniCriticVerdict,
            Self::StructuralRetryStageAttempt(_) => SsbActionType::StructuralRetryStageAttempt,
            Self::BridgeFileGenerated(_) => SsbActionType::BridgeFileGenerated,
            // D-1 (SCHEMA_VERSION 64): ReasoningTemplate ArchiveChain lifecycle.
            Self::ArchiveEntryAdded(_) => SsbActionType::ArchiveEntryAdded,
            Self::ArchiveEntryRetired(_) => SsbActionType::ArchiveEntryRetired,
            Self::TemplatePromoted(_) => SsbActionType::TemplatePromoted,
            Self::TemplateCandidateRejected(_) => SsbActionType::TemplateCandidateRejected,
            // E-5 (SCHEMA_VERSION 65): memory_facts bi-temporal lifecycle beacons.
            Self::MemoryFactExtracted(_) => SsbActionType::MemoryFactExtracted,
            Self::MemoryFactSuperseded(_) => SsbActionType::MemoryFactSuperseded,
            // D-2 (SCHEMA_VERSION 66): PPM verifier beacons.
            Self::PpmScored(_) => SsbActionType::PpmScored,
            Self::PpmGatingFailed(_) => SsbActionType::PpmGatingFailed,
            // E-1 (SCHEMA_VERSION 67): infinite-context chat illusion beacons.
            Self::ChatReflectionCompleted(_) => SsbActionType::ChatReflectionCompleted,
            Self::MemoryEvolutionRun(_) => SsbActionType::MemoryEvolutionRun,
            // E-1 dashboard slice (SCHEMA_VERSION 68): MemoryNotes / CoreBlock
            // producer + conflict beacons.
            Self::MemoryNoteInserted(_) => SsbActionType::MemoryNoteInserted,
            Self::MemoryNoteEvolved(_) => SsbActionType::MemoryNoteEvolved,
            Self::MemoryConflictDetected(_) => SsbActionType::MemoryConflictDetected,
            Self::CoreBlockReplaced(_) => SsbActionType::CoreBlockReplaced,
            // D-3 (SCHEMA_VERSION 69): multi-verifier consensus aggregator.
            Self::VerifierConsensusPass(_) => SsbActionType::VerifierConsensusPass,
            Self::VerifierConsensusFail(_) => SsbActionType::VerifierConsensusFail,
            // D-4 (SCHEMA_VERSION 70): tripwire stack beacons.
            Self::TripwireFired(_) => SsbActionType::TripwireFired,
            Self::TemplateAutoReverted(_) => SsbActionType::TemplateAutoReverted,
            // D-4 GAP-2 (SCHEMA_VERSION 108): concurrent-detector tripwire row.
            Self::TripwireFiredConcurrent(_) => SsbActionType::TripwireFiredConcurrent,
            // D-19 (SCHEMA_VERSION 71): PRMBench adversarial-gaming benchmark beacon.
            Self::PrmBenchRunCompleted(_) => SsbActionType::PrmBenchRunCompleted,
            // D-5 (SCHEMA_VERSION 72): BAI Thompson sampling promotion round.
            Self::BaiArmUpdated(_) => SsbActionType::BaiArmUpdated,
            // D-5 closure (SCHEMA_VERSION 208): unified promotion-decision summary.
            Self::BaiPromotionDecision(_) => SsbActionType::BaiPromotionDecision,
            // D-6 (SCHEMA_VERSION 73): held-out task store lifecycle.
            Self::HeldOutTaskAdded(_) => SsbActionType::HeldOutTaskAdded,
            Self::HeldOutTaskUsed(_) => SsbActionType::HeldOutTaskUsed,
            // D-7 (SCHEMA_VERSION 74): cross-domain transfer canary beacons.
            Self::CanaryRunCompleted(_) => SsbActionType::CanaryRunCompleted,
            Self::CanaryFamilyEvaluated(_) => SsbActionType::CanaryFamilyEvaluated,
            Self::LockedKeysAuditRun(_) => SsbActionType::LockedKeysAuditRun,
            Self::LockedKeysViolationDetected(_) => {
                SsbActionType::LockedKeysViolationDetected
            }
            // E-46 (SCHEMA_VERSION 76): hierarchical decomposition tuning beacons.
            Self::TaskAdvanced(_) => SsbActionType::TaskAdvanced,
            Self::AgentlessShortcutTaken(_) => SsbActionType::AgentlessShortcutTaken,
            Self::GoalDriftFired(_) => SsbActionType::GoalDriftFired,
            // D-10 (SCHEMA_VERSION 77): mode-routing predicate beacon.
            Self::ClaimRouted(_) => SsbActionType::ClaimRouted,
            // D-21 (SCHEMA_VERSION 78): mode-routing coverage benchmark beacon.
            Self::ModeRoutingBenchRunCompleted(_) => {
                SsbActionType::ModeRoutingBenchRunCompleted
            }
            // D-11 (SCHEMA_VERSION 79): constructive-checker registry beacon.
            Self::CheckerResultEmitted(_) => SsbActionType::CheckerResultEmitted,
            // D-13 (SCHEMA_VERSION 80): adversarial-verification beacons.
            Self::AdversaryGenerated(_) => SsbActionType::AdversaryGenerated,
            Self::ConsensusReevaluatedWithCounter(_) => {
                SsbActionType::ConsensusReevaluatedWithCounter
            }
            // D-12 (SCHEMA_VERSION 81): OperationalizationCheck beacon.
            Self::BabyStepDispatched(_) => SsbActionType::BabyStepDispatched,
            // D-16 (SCHEMA_VERSION 82): UniCR conformal-refusal beacons.
            Self::ConformalTauComputed(_) => SsbActionType::ConformalTauComputed,
            Self::ConformalRefusalDecided(_) => SsbActionType::ConformalRefusalDecided,
            // D-15 (SCHEMA_VERSION 83): discovery-vs-search NoveltyClass beacon.
            Self::NoveltyClassified(_) => SsbActionType::NoveltyClassified,
            // D-18 (SCHEMA_VERSION 84): typed-ObservableEffect predicate beacon.
            Self::ObservableEffectVerified(_) => SsbActionType::ObservableEffectVerified,
            // D-17 (SCHEMA_VERSION 85): honest-refusal SLA monitoring beacons.
            Self::RefusalRateSnapshot(_) => SsbActionType::RefusalRateSnapshot,
            Self::RefusalSlaBreach(_) => SsbActionType::RefusalSlaBreach,
            // D-22 (SCHEMA_VERSION 86): conformal-calibration validation beacon.
            Self::ConformalValidationCompleted(_) => {
                SsbActionType::ConformalValidationCompleted
            }
            // D-20 (SCHEMA_VERSION 87): ARC-AGI-2 OOD probe gating beacon.
            Self::ArcBenchmarkCompleted(_) => SsbActionType::ArcBenchmarkCompleted,
            Self::ToolGated(_) => SsbActionType::ToolGated,
            Self::ToolEager(_) => SsbActionType::ToolEager,
            Self::ToolLazyResolved(_) => SsbActionType::ToolLazyResolved,
            Self::ToolDenied(_) => SsbActionType::ToolDenied,
            Self::QuantBackendSelected(_) => SsbActionType::QuantBackendSelected,
            Self::KvCacheDtypeSoftwareSim(_) => SsbActionType::KvCacheDtypeSoftwareSim,
            Self::AwqCheckpointBuilt(_) => SsbActionType::AwqCheckpointBuilt,
            Self::CliMcpDispatch(_) => SsbActionType::CliMcpDispatch,
            Self::CliMcpServerStarted(_) => SsbActionType::CliMcpServerStarted,
            Self::CliMcpServerStopped(_) => SsbActionType::CliMcpServerStopped,
            Self::FirecrackerRestoreSlow(_) => SsbActionType::FirecrackerRestoreSlow,
            Self::SnapshotUniquenessRegenerated(_) => SsbActionType::SnapshotUniquenessRegenerated,
            Self::SnapshotEvicted(_) => SsbActionType::SnapshotEvicted,
            Self::SandboxBackendSelected(_) => SsbActionType::SandboxBackendSelected,
            Self::AgentForkedFromSnapshot(_) => SsbActionType::AgentForkedFromSnapshot,
            Self::ResearcherNetEgressBlocked(_) => SsbActionType::ResearcherNetEgressBlocked,
            Self::McpSchemaPinViolation(_) => SsbActionType::McpSchemaPinViolation,
            Self::McpSanitizationRuleApplied(_) => SsbActionType::McpSanitizationRuleApplied,
            Self::McpBudgetEnforced(_) => SsbActionType::McpBudgetEnforced,
            Self::McpLegacySseTransport(_) => SsbActionType::McpLegacySseTransport,
            Self::McpLlmGuardBlocked(_) => SsbActionType::McpLlmGuardBlocked,
            Self::AnthropicGatewayDetected(_) => SsbActionType::AnthropicGatewayDetected,
            Self::MemoryTierAssigned(_) => SsbActionType::MemoryTierAssigned,
            Self::MemoryQueryFiltered(_) => SsbActionType::MemoryQueryFiltered,
            Self::SafetyCriticalBlocked(_) => SsbActionType::SafetyCriticalBlocked,
            // E-30: public-incident hardening beacons
            Self::ArgvDestructiveDenied(_) => SsbActionType::ArgvDestructiveDenied,
            Self::ArgvApprovalRequired(_) => SsbActionType::ArgvApprovalRequired,
            Self::TrustDialogHookDeferred(_) => SsbActionType::TrustDialogHookDeferred,
            Self::TrustDialogEnvDeferred(_) => SsbActionType::TrustDialogEnvDeferred,
            Self::McpPinVersionRefused(_) => SsbActionType::McpPinVersionRefused,
            Self::AuditorLineageCollision(_) => SsbActionType::AuditorLineageCollision,
            Self::UntrustedMemoryWriteBlocked(_) => SsbActionType::UntrustedMemoryWriteBlocked,
            Self::IncidentRecordCreated(_) => SsbActionType::IncidentRecordCreated,
            // E-32: context-window extension beacons
            Self::FitmCalibrationApplied(_) => SsbActionType::FitmCalibrationApplied,
            Self::DeepModeActivated(_) => SsbActionType::DeepModeActivated,
            Self::DeepModeEscalationChosen(_) => SsbActionType::DeepModeEscalationChosen,
            Self::YarnRefused(_) => SsbActionType::YarnRefused,
            // B-3 (SCHEMA_VERSION 98): AntiStalenessDrawer + RulesDrawer beacons.
            Self::DepAuditCompleted(_) => SsbActionType::DepAuditCompleted,
            Self::BaselineRunCompleted(_) => SsbActionType::BaselineRunCompleted,
            Self::DeterministicAuditRun(_) => SsbActionType::DeterministicAuditRun,
            Self::RuleTransition(_) => SsbActionType::RuleTransition,
            // B-4 (SCHEMA_VERSION 99): canary drift snapshot row.
            Self::CanaryDrift(_) => SsbActionType::CanaryDrift,
            // B-19 (SCHEMA_VERSION 101): per-agent-execution DAIRA marker aggregation.
            Self::MarkerReport(_) => SsbActionType::MarkerReport,
            // B-25 (SCHEMA_VERSION 102): Phase 4J Empire integration.
            Self::EmpireConnected(_) => SsbActionType::EmpireConnected,
            Self::EmpireDisconnected(_) => SsbActionType::EmpireDisconnected,
            Self::EmpireEventReceived(_) => SsbActionType::EmpireEventReceived,
            Self::EmpireCapabilityInvoked(_) => SsbActionType::EmpireCapabilityInvoked,
            Self::EmpireRepairProposed(_) => SsbActionType::EmpireRepairProposed,
            // C-5 follow-on (SCHEMA_VERSION 103).
            Self::RouteBlocked(_) => SsbActionType::RouteBlocked,
            Self::WarnLimitReached(_) => SsbActionType::WarnLimitReached,
            Self::CostEstimate(_) => SsbActionType::CostEstimate,
            // C-36 (SCHEMA_VERSION 104).
            Self::MucocoMutationApplied(_) => SsbActionType::MucocoMutationApplied,
            // D-3 closure (SCHEMA_VERSION 105).
            Self::PpmUnavailable(_) => SsbActionType::PpmUnavailable,
            Self::VerifierDisagreement(_) => SsbActionType::VerifierDisagreement,
            Self::CrossContextDispatched(_) => SsbActionType::CrossContextDispatched,
            // C-38 (SCHEMA_VERSION 107).
            Self::PagerankRun(_) => SsbActionType::PagerankRun,
            Self::DkgRebuildStarted(_) => SsbActionType::DkgRebuildStarted,
            Self::DkgRebuildComplete(_) => SsbActionType::DkgRebuildComplete,
            // C-40 (SCHEMA_VERSION 109).
            Self::GithubPrOpened(_) => SsbActionType::GithubPrOpened,
            Self::GithubPrStatusFetched(_) => SsbActionType::GithubPrStatusFetched,
            Self::GithubPrClosed(_) => SsbActionType::GithubPrClosed,
            Self::GithubPrMerged(_) => SsbActionType::GithubPrMerged,
            // E-41 (SCHEMA_VERSION 135): GitHub PR review surface.
            Self::PrReviewComposed(_) => SsbActionType::PrReviewComposed,
            Self::PrCommentWatched(_) => SsbActionType::PrCommentWatched,
            Self::PrPackageVerifyResult(_) => SsbActionType::PrPackageVerifyResult,
            Self::PrSemgrepGateResult(_) => SsbActionType::PrSemgrepGateResult,
            Self::PrAutoMergeBlocked(_) => SsbActionType::PrAutoMergeBlocked,
            // D-5 closure (SCHEMA_VERSION 110): Thompson-sample beacon.
            Self::ThompsonSampled(_) => SsbActionType::ThompsonSampled,
            // C-41 (SCHEMA_VERSION 111): skills-driven workflow registry.
            Self::SkillTriggered(_) => SsbActionType::SkillTriggered,
            Self::SkillExecuted(_) => SsbActionType::SkillExecuted,
            // D-6 closure (SCHEMA_VERSION 112): held-out promotion-gate +
            // refresh-event beacons.
            Self::HeldOutPromotionGate(_) => SsbActionType::HeldOutPromotionGate,
            Self::HeldOutRefresh(_) => SsbActionType::HeldOutRefresh,
            // C-42 (SCHEMA_VERSION 113): Semantic Gateway L2 RBAC beacons.
            Self::GatewayPolicyAllowed(_) => SsbActionType::GatewayPolicyAllowed,
            Self::GatewayPolicyDenied(_) => SsbActionType::GatewayPolicyDenied,
            Self::GatewayHitlPending(_) => SsbActionType::GatewayHitlPending,
            Self::CrossDomainRegression(_) => SsbActionType::CrossDomainRegression,
            Self::CanaryFamilyPass(_) => SsbActionType::CanaryFamilyPass,
            Self::CanaryInsufficientData(_) => SsbActionType::CanaryInsufficientData,
            // D-8 closure (SCHEMA_VERSION 115): cryptographic archive verify.
            Self::ArchiveVerifyResult(_) => SsbActionType::ArchiveVerifyResult,
            // C-43 closure (SCHEMA_VERSION 116): Phase 4H scope-freeze.
            Self::ScopeFreezeRatified(_) => SsbActionType::ScopeFreezeRatified,
            Self::ScopeFreezeViolated(_) => SsbActionType::ScopeFreezeViolated,
            // C-44 (SCHEMA_VERSION 117): Composer 2 paid-backend cost accrual.
            Self::BackendCostAccrued(_) => SsbActionType::BackendCostAccrued,
            // C-45 (SCHEMA_VERSION 120): CANTANTE credit attribution.
            Self::AgentRoleDropped(_) => SsbActionType::AgentRoleDropped,
            Self::CreditAttributionRun(_) => SsbActionType::CreditAttributionRun,
            Self::CreditAttributionResult(_) => SsbActionType::CreditAttributionResult,
            // C-46 (SCHEMA_VERSION 121): Uno-Orchestra selective delegation.
            Self::SelectiveDelegationDecision(_) => SsbActionType::SelectiveDelegationDecision,
            Self::SelectiveDelegationEscalated(_) => SsbActionType::SelectiveDelegationEscalated,
            Self::SelectiveDelegationCostSaved(_) => SsbActionType::SelectiveDelegationCostSaved,
            // C-47 (SCHEMA_VERSION 122): Case-Based Calibration of Tool-Use.
            Self::ToolCallLogged(_) => SsbActionType::ToolCallLogged,
            Self::ToolCalibrationHintEmitted(_) => SsbActionType::ToolCalibrationHintEmitted,
            Self::ToolCalibrationDriftDetected(_) => SsbActionType::ToolCalibrationDriftDetected,
            // C-49 (SCHEMA_VERSION 123): iterative-distillation experience.
            Self::PromptPolicyDistillationRun(_) => SsbActionType::PromptPolicyDistillationRun,
            Self::PromptFragmentSelected(_) => SsbActionType::PromptFragmentSelected,
            Self::PromptFragmentDiscarded(_) => SsbActionType::PromptFragmentDiscarded,
            // D-14 (SCHEMA_VERSION 124): sycophancy probe row pair.
            Self::SycophancyProbeDispatched(_) => SsbActionType::SycophancyProbeDispatched,
            Self::SycophancyFlag(_) => SsbActionType::SycophancyFlag,
            // D-23 (SCHEMA_VERSION 142): sycophancy detection benchmark.
            Self::SycophancyBenchRunCompleted(_) => SsbActionType::SycophancyBenchRunCompleted,
            // D-24 (SCHEMA_VERSION 143): cross-domain canary regression
            // benchmark.
            Self::CanaryBenchRunCompleted(_) => SsbActionType::CanaryBenchRunCompleted,
            // D-25 (SCHEMA_VERSION 144): determinism + archive tamper-suite
            // benchmark.
            Self::DeterminismTamperSuiteCompleted(_) => {
                SsbActionType::DeterminismTamperSuiteCompleted
            }
            // D-28 (SCHEMA_VERSION 145): adversarial example farming run.
            Self::AdversarialFarmRunCompleted(_) => {
                SsbActionType::AdversarialFarmRunCompleted
            }
            // D-29 (SCHEMA_VERSION 146): hierarchical BAI round completion.
            Self::HierarchicalBaiRoundCompleted(_) => {
                SsbActionType::HierarchicalBaiRoundCompleted
            }
            // D-30 (SCHEMA_VERSION 147): calibration bootstrap warm-start.
            Self::CalibrationBootstrapCompleted(_) => {
                SsbActionType::CalibrationBootstrapCompleted
            }
            // C-50 (SCHEMA_VERSION 125): Contract-Coding scaffolding pair.
            Self::ContractTestScaffolded(_) => SsbActionType::ContractTestScaffolded,
            Self::ContractViolationDetected(_) => SsbActionType::ContractViolationDetected,
            // C-52 (SCHEMA_VERSION 126): Dafny Formal Verification Spike.
            Self::DafnyVerifyCompleted(_) => SsbActionType::DafnyVerifyCompleted,
            // D-16 closure (SCHEMA_VERSION 127): durable tau persistence.
            Self::CalibrationCorpusUpdated(_) => SsbActionType::CalibrationCorpusUpdated,
            // E-12 Tier-B (SCHEMA_VERSION 128): production fullstack codegen.
            Self::FrameworkProfileSelected(_) => SsbActionType::FrameworkProfileSelected,
            Self::LegacyConfigStripped(_) => SsbActionType::LegacyConfigStripped,
            // D-18 closure (SCHEMA_VERSION 129): typed-ObservableEffect 4-variant pack.
            Self::ObservableEffectDispatched(_) => SsbActionType::ObservableEffectDispatched,
            Self::GoodhartRiskFlag(_) => SsbActionType::GoodhartRiskFlag,
            Self::SandboxObservation(_) => SsbActionType::SandboxObservation,
            Self::ProductionObservationProposal(_) => SsbActionType::ProductionObservationProposal,
            // E-13 (SCHEMA_VERSION 130): TDD verifier-agent beacon family.
            Self::TddCycleStarted(_) => SsbActionType::TddCycleStarted,
            Self::TddTestFailed(_) => SsbActionType::TddTestFailed,
            Self::TddTestPassed(_) => SsbActionType::TddTestPassed,
            // Tranche B-7 triple (SCHEMA_VERSION 195): TDD verification gate.
            Self::TddVerificationEvent(_) => SsbActionType::TddVerificationEvent,
            // E-14 (SCHEMA_VERSION 131): browser-e2e agent beacon family.
            Self::BrowserE2eRunStarted(_) => SsbActionType::BrowserE2eRunStarted,
            Self::BrowserE2eAssertionFailed(_) => SsbActionType::BrowserE2eAssertionFailed,
            Self::BrowserE2eAssertionPassed(_) => SsbActionType::BrowserE2eAssertionPassed,
            Self::BrowserE2eSnapshotCaptured(_) => SsbActionType::BrowserE2eSnapshotCaptured,
            Self::ToolReliabilityGatePassed(_) => SsbActionType::ToolReliabilityGatePassed,
            Self::ToolReliabilityGateRejected(_) => SsbActionType::ToolReliabilityGateRejected,
            Self::ToolCallSpilled(_) => SsbActionType::ToolCallSpilled,
            Self::ToolCallRetryEscalated(_) => SsbActionType::ToolCallRetryEscalated,
            // E-37 (SCHEMA_VERSION 133): prompt-injection defense beacons.
            Self::InjectionAttemptDetected(_) => SsbActionType::InjectionAttemptDetected,
            Self::InjectionAttemptBlocked(_) => SsbActionType::InjectionAttemptBlocked,
            Self::SpotlightWrapped(_) => SsbActionType::SpotlightWrapped,
            Self::SpotlightBreached(_) => SsbActionType::SpotlightBreached,
            Self::ToolDescriptionPinMismatch(_) => SsbActionType::ToolDescriptionPinMismatch,
            Self::CapabilityGateRejected(_) => SsbActionType::CapabilityGateRejected,
            Self::ProvenanceTagged(_) => SsbActionType::ProvenanceTagged,
            Self::PromptArmorVerdict(_) => SsbActionType::PromptArmorVerdict,
            Self::InjectionFixtureReplayed(_) => SsbActionType::InjectionFixtureReplayed,
            // E-40 (SCHEMA_VERSION 134): orchestrator-rooted symbol-graph beacons.
            Self::SymbolGraphQueried(_) => SsbActionType::SymbolGraphQueried,
            Self::SymbolGraphCacheLoaded(_) => SsbActionType::SymbolGraphCacheLoaded,
            Self::LspOracleConsulted(_) => SsbActionType::LspOracleConsulted,
            // E-44 (SCHEMA_VERSION 136): durability layer beacons.
            Self::CheckpointWritten(_) => SsbActionType::CheckpointWritten,
            Self::CheckpointResumed(_) => SsbActionType::CheckpointResumed,
            Self::CheckpointMigrated(_) => SsbActionType::CheckpointMigrated,
            Self::IdempotentDispatchSkipped(_) => SsbActionType::IdempotentDispatchSkipped,
            Self::DurabilityLockBlocked(_) => SsbActionType::DurabilityLockBlocked,
            // E-45 (SCHEMA_VERSION 137): cross-language drift detector beacons.
            Self::CrossLangDriftDetected(_) => SsbActionType::CrossLangDriftDetected,
            Self::CrossLangPairScanned(_) => SsbActionType::CrossLangPairScanned,
            Self::SchemaVersionMismatch(_) => SsbActionType::SchemaVersionMismatch,
            // E-49 (SCHEMA_VERSION 138): coding-agent metric suite.
            Self::CodingAgentMetricsCaptured(_) => SsbActionType::CodingAgentMetricsCaptured,
            Self::SweBenchProRunStarted(_) => SsbActionType::SweBenchProRunStarted,
            Self::SweBenchProRunCompleted(_) => SsbActionType::SweBenchProRunCompleted,
            Self::RefineBenchScoreEmitted(_) => SsbActionType::RefineBenchScoreEmitted,
            Self::MetricsAggregationRun(_) => SsbActionType::MetricsAggregationRun,
            // E-52 Tier-B (SCHEMA_VERSION 139): LSP-as-verifier beacons.
            Self::DiagnosticProbeDispatched(_) => SsbActionType::DiagnosticProbeDispatched,
            Self::DiagnosticProbeCompleted(_) => SsbActionType::DiagnosticProbeCompleted,
            Self::DiagnosticProbeBlock(_) => SsbActionType::DiagnosticProbeBlock,
            Self::ScipRefreshCompleted(_) => SsbActionType::ScipRefreshCompleted,
            // E-53 (SCHEMA_VERSION 140): formal-verify pipeline boundary beacon.
            Self::FormalVerifierCompleted(_) => SsbActionType::FormalVerifierCompleted,
            // E-35 (SCHEMA_VERSION 141): structured-output substrate beacons.
            Self::XgrammarMaskApplied(_) => SsbActionType::XgrammarMaskApplied,
            Self::InstructorModeActivated(_) => SsbActionType::InstructorModeActivated,
            Self::GrammarModeActivated(_) => SsbActionType::GrammarModeActivated,
            Self::CodeactActionExecuted(_) => SsbActionType::CodeactActionExecuted,
            Self::SearchReplaceTier1Hit(_) => SsbActionType::SearchReplaceTier1Hit,
            Self::SearchReplaceCollisionFallback(_) => {
                SsbActionType::SearchReplaceCollisionFallback
            }
            Self::CloudStrictSchemaApplied(_) => SsbActionType::CloudStrictSchemaApplied,
            Self::GrammarCacheLoaded(_) => SsbActionType::GrammarCacheLoaded,
            Self::StructuredEngineRejected(_) => SsbActionType::StructuredEngineRejected,
            Self::LegacyJsonObjectRefused(_) => SsbActionType::LegacyJsonObjectRefused,
            // E-6 (SCHEMA_VERSION 148): MAST-grounded failure-mode defenses.
            Self::AuditMastTagged(_) => SsbActionType::AuditMastTagged,
            Self::IsoBudgetMeasured(_) => SsbActionType::IsoBudgetMeasured,
            Self::PersonaReanchored(_) => SsbActionType::PersonaReanchored,
            Self::CrossFamilyUnavailable(_) => SsbActionType::CrossFamilyUnavailable,
            // E-7 (SCHEMA_VERSION 149): darwinian self-improvement substrate.
            Self::DarwinianGenerationCompleted(_) => {
                SsbActionType::DarwinianGenerationCompleted
            }
            // E-3 (SCHEMA_VERSION 150): runtime handoff signal beacon.
            Self::RuntimeHandoffSignal(_) => SsbActionType::RuntimeHandoffSignal,
            // E-26 (SCHEMA_VERSION 151): inference-engines 2026 pack.
            Self::SglangBackendRegistered(_) => SsbActionType::SglangBackendRegistered,
            Self::XgrammarSchemaCompiled(_) => SsbActionType::XgrammarSchemaCompiled,
            Self::RuntimeColdStartObserved(_) => SsbActionType::RuntimeColdStartObserved,
            // E-2 (SCHEMA_VERSION 152-153): streaming timing + UX pacing vote.
            Self::StreamingTimingRecorded(_) => SsbActionType::StreamingTimingRecorded,
            Self::UxPacingVoteRecorded(_) => SsbActionType::UxPacingVoteRecorded,
            // E-28 (SCHEMA_VERSION 154-156): attention-kernels 2026 pack.
            Self::StreamingLlmEngaged(_) => SsbActionType::StreamingLlmEngaged,
            Self::SdpaBackendPinned(_) => SsbActionType::SdpaBackendPinned,
            Self::YarnRopeScalingApplied(_) => SsbActionType::YarnRopeScalingApplied,
            // E-8 (SCHEMA_VERSION 162): self-evolving prompts substrate.
            Self::PromptOptimizationCompleted(_) => SsbActionType::PromptOptimizationCompleted,
            // E-10 (SCHEMA_VERSION 163): hostile-critic structural simplifications.
            Self::CriticDecisionEvent(_) => SsbActionType::CriticDecisionEvent,
            // E-43 (SCHEMA_VERSION 164): agent skill systems telemetry.
            Self::SkillSystemEvent(_) => SsbActionType::SkillSystemEvent,
            // E-9 (SCHEMA_VERSION 165): external agent-eval benchmark suite.
            Self::BenchmarkRunObserved(_) => SsbActionType::BenchmarkRunObserved,
            // E-19 (SCHEMA_VERSION 166): reasoning quality probe.
            Self::ReasoningQualityProbe(_) => SsbActionType::ReasoningQualityProbe,
            // E-54 (SCHEMA_VERSION 167): eval-corpus repair pre-deploy.
            Self::EvalCorpusEvent(_) => SsbActionType::EvalCorpusEvent,
            // E-55 (SCHEMA_VERSION 168): agentic RL + reward ladder substrate.
            Self::RewardLadderEvent(_) => SsbActionType::RewardLadderEvent,
            // E-11 (SCHEMA_VERSION 169): SaaS prior-art catalog +
            // recommender substrate.
            Self::SaasPriorArtEvent(_) => SsbActionType::SaasPriorArtEvent,
            // E-22 (SCHEMA_VERSION 170): may-2026 github trending sweep
            // substrate.
            Self::GithubTrendingEvent(_) => SsbActionType::GithubTrendingEvent,
            // E-23 (SCHEMA_VERSION 171): 2026 developer-discourse
            // positioning + UX-guard substrate.
            Self::DeveloperDiscourseEvent(_) => SsbActionType::DeveloperDiscourseEvent,
            // E-25 (SCHEMA_VERSION 172): claude-code-architecture catalog
            // substrate.
            Self::ClaudeCodeArchitectureEvent(_) => SsbActionType::ClaudeCodeArchitectureEvent,
            Self::AgenticObservabilityEvent(_) => SsbActionType::AgenticObservabilityEvent,
            // E-31 (SCHEMA_VERSION 174): coding-model-leaderboard scraper substrate.
            Self::CodingModelLeaderboardEvent(_) => SsbActionType::CodingModelLeaderboardEvent,
            // E-38 (SCHEMA_VERSION 175): 2026 AI roadmap signals + EU AI
            // Act compliance pre-gate substrate.
            Self::AiRoadmapSignalEvent(_) => SsbActionType::AiRoadmapSignalEvent,
            // E-39 (SCHEMA_VERSION 176): cost + token economics substrate.
            Self::CostTokenEvent(_) => SsbActionType::CostTokenEvent,
            // E-50 (SCHEMA_VERSION 177): SaaS architecture pattern
            // catalog substrate.
            Self::SaasArchPatternEvent(_) => SsbActionType::SaasArchPatternEvent,
            // E-4 (SCHEMA_VERSION 178): chat UX critique substrate.
            Self::ChatUxCritiqueEvent(_) => SsbActionType::ChatUxCritiqueEvent,
            Self::VlmImageValidationEvent(_) => SsbActionType::VlmImageValidationEvent,
            // E-20 (SCHEMA_VERSION 180): local-cloud routing substrate.
            Self::LocalCloudRoutingEvent(_) => SsbActionType::LocalCloudRoutingEvent,
            // E-21 (SCHEMA_VERSION 181): peer-systems harness substrate.
            Self::PeerSystemsEvent(_) => SsbActionType::PeerSystemsEvent,
            // E-36 (SCHEMA_VERSION 182): document + vision RAG substrate.
            Self::DocumentVisionRagEvent(_) => SsbActionType::DocumentVisionRagEvent,
            // E-42 (SCHEMA_VERSION 183): tree-sitter grammar health substrate.
            Self::TreeSitterGrammarEvent(_) => SsbActionType::TreeSitterGrammarEvent,
            // E-46 (SCHEMA_VERSION 184): planning-decomposition substrate.
            Self::PlanningDecompositionEvent(_) => SsbActionType::PlanningDecompositionEvent,
            // E-48 (SCHEMA_VERSION 185): web-research orchestrator
            // hardening substrate.
            Self::WebResearchEvent(_) => SsbActionType::WebResearchEvent,
            // E-44 (SCHEMA_VERSION 186): durability observability substrate.
            Self::DurabilityEvent(_) => SsbActionType::DurabilityEvent,
            // E-31 (SCHEMA_VERSION 187): head_dim invariant verification substrate.
            Self::HeadDimInvariantVerified(_) => SsbActionType::HeadDimInvariantVerified,
            // E-3 (SCHEMA_VERSION 188): role-shown / model-hidden default
            // telemetry substrate.
            Self::RoleShownModelHidden(_) => SsbActionType::RoleShownModelHidden,
            // E-49 (SCHEMA_VERSION 189): coding-agent metrics named-token substrate.
            Self::CodingMetricNamedToken(_) => SsbActionType::CodingMetricNamedToken,
            // E-3 sub-phase 4A (SCHEMA_VERSION 190): runtime handoff event substrate.
            Self::E3RuntimeHandoffEvent(_) => SsbActionType::E3RuntimeHandoffEvent,
            // E-31 sub-phase 6A (SCHEMA_VERSION 191): cloud-route decision event substrate.
            Self::RouteDecisionEvent(_) => SsbActionType::RouteDecisionEvent,
            // E-47 (SCHEMA_VERSION 192): LoRA fine-tuning ship-gate substrate.
            Self::LoraShipGateEvent(_) => SsbActionType::LoraShipGateEvent,
            // E-40 Tranche B-2 (SCHEMA_VERSION 193): SymbolGraph augmentation substrate.
            Self::SymbolGraphEvent(_) => SsbActionType::SymbolGraphEvent,
            // Tranche B-6 quad (SCHEMA_VERSION 194): four new SSB substrates.
            Self::McpAuthzEvent(_) => SsbActionType::McpAuthzEvent,
            Self::ToolCalibrationEvent(_) => SsbActionType::ToolCalibrationEvent,
            Self::ReflectBudgetSoftWarn(_) => SsbActionType::ReflectBudgetSoftWarn,
            Self::ContractCodingEvent(_) => SsbActionType::ContractCodingEvent,
            // F-1 / F-2a (SCHEMA_VERSION 196): vibe-coding security gate variants.
            Self::VibeGateBlocked(p) => SsbActionType::VibeGateBlocked(p.kind),
            Self::SecurityRuleBlocked(_) => SsbActionType::SecurityRuleBlocked,
            // F-4 / Slice B-1 (SCHEMA_VERSION 197): test integrity verification.
            Self::TestIntegrityEvent(p) => SsbActionType::TestIntegrityEvent(p.kind),
            // F-3p1 / Slice C1 (SCHEMA_VERSION 198): capability-flow tool-filter.
            Self::ToolFilterEvent(p) => SsbActionType::ToolFilterEvent(p.kind),
            // F-9 / Slice D1 (SCHEMA_VERSION 199): write-time dep slopsquat blocked.
            Self::WriteTimeSlopsquatBlocked(_) => SsbActionType::WriteTimeSlopsquatBlocked,
            // F-10 / Slice A1 (SCHEMA_VERSION 200): security-monotonicity guard.
            Self::SecurityClassRegressionBlocked(_) => {
                SsbActionType::SecurityClassRegressionBlocked
            }
            // F-2b / Slice A1 (SCHEMA_VERSION 201): insecure-rate keystone bench beacon.
            Self::InsecureRateMeasured(_) => SsbActionType::InsecureRateMeasured,
            // I-9 (SCHEMA_VERSION 263): error-amplification (alpha_t) telemetry beacon.
            Self::ErrorAmplificationMeasured(_) => SsbActionType::ErrorAmplificationMeasured,
            // F-6 / Slice A1 (SCHEMA_VERSION 202): learned-skill library event.
            Self::SkillLibraryEvent(p) => SsbActionType::SkillLibraryEvent(p.kind),
            // F-5 / Slice A1 (SCHEMA_VERSION 203): staleness-quarantine broadcast.
            Self::StalenessQuarantineBroadcast(_) => SsbActionType::StalenessQuarantineBroadcast,
            // F-8 / Slice A1 (SCHEMA_VERSION 204): symbolic semantic conflict.
            Self::SymbolicConflictDetected(_) => SsbActionType::SymbolicConflictDetected,
            // F-7 / Slice A1 (SCHEMA_VERSION 205): CBR template utility ranking.
            Self::TemplateUtilityRanked(_) => SsbActionType::TemplateUtilityRanked,
            // F-13 / Slice A1 (SCHEMA_VERSION 206): session-archive-on-bump.
            Self::SessionArchived(_) => SsbActionType::SessionArchived,
            // F-11 / Slice A1 (SCHEMA_VERSION 207): config-DGM admit-loop beacon.
            Self::DarwinianAdmit(_) => SsbActionType::DarwinianAdmit,
            // G-plan variants.
            Self::McpRugPullBlocked(_) => SsbActionType::McpRugPullBlocked,
            Self::TaintFlow(_) => SsbActionType::TaintFlow,
            Self::BlastRadiusBlocked(_) => SsbActionType::BlastRadiusBlocked,
            Self::TrifectaQuarantined(_) => SsbActionType::TrifectaQuarantined,
            Self::RunArtifactEmitted(_) => SsbActionType::RunArtifactEmitted,
            Self::ReviewSurfaceScored(_) => SsbActionType::ReviewSurfaceScored,
            Self::ProvenanceAttestationEmitted(_) => SsbActionType::ProvenanceAttestationEmitted,
            Self::SafetyDeltaMeasured(_) => SsbActionType::SafetyDeltaMeasured,
            Self::ModelDriftSuspected(_) => SsbActionType::ModelDriftSuspected,
            Self::SecretRedacted(_) => SsbActionType::SecretRedacted,
            Self::ClaimConfidenceScored(_) => SsbActionType::ClaimConfidenceScored,
            Self::SteeringVectorRefreshTriggered(_) => SsbActionType::SteeringVectorRefreshTriggered,
            Self::CanvasOpApplied(_) => SsbActionType::CanvasOpApplied,
            Self::CanvasViewEvent(_) => SsbActionType::CanvasViewEvent,
            Self::CanvasActivityEvent(_) => SsbActionType::CanvasActivityEvent,
            Self::BatchLifecycle(_) => SsbActionType::BatchLifecycle,
            // H-plan verifiability variants.
            Self::DeflationGateEvaluated(_) => SsbActionType::DeflationGateEvaluated,
            Self::LeakageBlocked(_) => SsbActionType::LeakageBlocked,
            Self::LookaheadDifferential(_) => SsbActionType::LookaheadDifferential,
            Self::TradingNullTest(_) => SsbActionType::TradingNullTest,
            Self::RerunHashEquality(_) => SsbActionType::RerunHashEquality,
            Self::SimRealParity(_) => SsbActionType::SimRealParity,
            Self::ResultManifestVerified(_) => SsbActionType::ResultManifestVerified,
            Self::DifferentialDivergence(_) => SsbActionType::DifferentialDivergence,
            Self::GateSelfValidation(_) => SsbActionType::GateSelfValidation,
            Self::RewardTampering(_) => SsbActionType::RewardTampering,
            Self::HoldoutBudgetEvent(_) => SsbActionType::HoldoutBudgetEvent,
            Self::DriftInterlock(_) => SsbActionType::DriftInterlock,
            Self::TrialRegistered(_) => SsbActionType::TrialRegistered,
            Self::MucocoInconsistencyDetected(_) => SsbActionType::MucocoInconsistencyDetected,
            Self::MucocoSkipReason(_) => SsbActionType::MucocoSkipReason,
            Self::UserDecisionSuperseded(_) => SsbActionType::UserDecisionSuperseded,
            Self::StalenessEvalMeasured(_) => SsbActionType::StalenessEvalMeasured,
            Self::DiscoveryRatified(_) => SsbActionType::DiscoveryRatified,
            Self::DiscoveryRejected(_) => SsbActionType::DiscoveryRejected,
            Self::FactValidationFailed(_) => SsbActionType::FactValidationFailed,
            Self::MemoryGovernanceEvent(_) => SsbActionType::MemoryGovernanceEvent,
            Self::McpToolDeferred(_) => SsbActionType::McpToolDeferred,
            Self::McpSchemaBudgetMeasured(_) => SsbActionType::McpSchemaBudgetMeasured,
            Self::SessionRewound(_) => SsbActionType::SessionRewound,
            Self::AgentNotificationEmitted(_) => SsbActionType::AgentNotificationEmitted,
            Self::PlanDecisionEvent(_) => SsbActionType::PlanDecisionEvent,
            Self::StopConfirmationEvent(_) => SsbActionType::StopConfirmationEvent,
            Self::AbstentionScored(_) => SsbActionType::AbstentionScored,
            Self::CheckerGapDetected(_) => SsbActionType::CheckerGapDetected,
            Self::ReasoningBudgetScaled(_) => SsbActionType::ReasoningBudgetScaled,
            Self::FitnessEvaluated(_) => SsbActionType::FitnessEvaluated,
            Self::TrapBehaviorDetected(_) => SsbActionType::TrapBehaviorDetected,
            Self::PromotionScoreAggregated(_) => SsbActionType::PromotionScoreAggregated,
            Self::CladeSelected(_) => SsbActionType::CladeSelected,
            Self::EvalFirewallEvent(_) => SsbActionType::EvalFirewallEvent,
            Self::EditHistoryEvent(_) => SsbActionType::EditHistoryEvent,
            Self::HandoffLifecycleEvent(_) => SsbActionType::HandoffLifecycleEvent,
            Self::FindingAnchored(_) => SsbActionType::FindingAnchored,
            Self::CitationAnchorsBound(_) => SsbActionType::CitationAnchorsBound,
            Self::PlanStateDerived(_) => SsbActionType::PlanStateDerived,
            Self::ObjectiveSealed(_) => SsbActionType::ObjectiveSealed(ObjectiveSealedKind::Sealed),
            Self::GrammarGateEvent(_) => SsbActionType::GrammarGateEvent(GrammarGateEventKind::CranePassed),
            Self::QdeltaMeasured(_) => SsbActionType::QdeltaMeasured,
            Self::ClaimScopeMeasured(_) => SsbActionType::ClaimScopeMeasured(ClaimScopeMeasuredKind::PilotDelta),
            Self::CacheEconomicsEvent(_) => SsbActionType::CacheEconomicsEvent(CacheEconomicsEventKind::TtlPlanned),
            Self::TrajectoryAborted(_) => SsbActionType::TrajectoryAborted(TrajectoryAbortedKind::Voluntary),
            Self::DirectiveExecuted(_) => SsbActionType::DirectiveExecuted(DirectiveExecutedKind::Executed),
            Self::GovernancePrimitiveFired(_) => SsbActionType::GovernancePrimitiveFired(GovernancePrimitiveFiredKind::CeflExternalized),
            Self::RoleContractApplied(_) => SsbActionType::RoleContractApplied(RoleContractAppliedKind::PersonaStripped),
            Self::ToolSequenceVerdict(_) => SsbActionType::ToolSequenceVerdict(ToolSequenceVerdictKind::Allowed),
            Self::CritiqueLensFinding(ref p) => SsbActionType::CritiqueLensFinding(match p.kind {
                CritiqueLensFindingKind::AdvisoryFinding { .. } => CritiqueLensFindingTag::AdvisoryFinding,
                CritiqueLensFindingKind::DroppedNoAnchor { .. } => CritiqueLensFindingTag::DroppedNoAnchor,
                CritiqueLensFindingKind::LensRun { .. } => CritiqueLensFindingTag::LensRun,
            }),
            Self::BashRedirected(ref p) => SsbActionType::BashRedirected(p.kind),
            Self::PromptHardeningApplied(ref p) => SsbActionType::PromptHardeningApplied(p.kind),
            Self::BranchWorkspaceEvent(ref p) => SsbActionType::BranchWorkspaceEvent(p.kind),
            Self::ContextPruned(ref p) => SsbActionType::ContextPruned(p.kind),
            Self::BeliefStateEvent(ref p) => SsbActionType::BeliefStateEvent(p.kind),
            Self::MemoryGuardEvent(ref p) => SsbActionType::MemoryGuardEvent(p.kind),
            Self::ContextTransformReceipt(ref p) => SsbActionType::ContextTransformReceipt(p.kind),
            // I-12 (SCHEMA_VERSION 251): evidence-gated playbook curator outcome (4-arm Pattern-2).
            Self::PlaybookEvent(ref p) => SsbActionType::PlaybookEvent(p.kind),
            // I-17 (SCHEMA_VERSION 252): tri-layer evaluation-stack per-branch verdict (4-arm Pattern-2).
            Self::TriLayerVerdict(ref p) => SsbActionType::TriLayerVerdict(p.kind),
            // I-18 (SCHEMA_VERSION 253): per-row rolling blake3 chain beacon (3-arm Pattern-2).
            Self::RowChainSealed(ref p) => SsbActionType::RowChainSealed(p.kind),
            // I-19 (SCHEMA_VERSION 254): TDD enforcer preflight gate (3-arm Pattern-2).
            // Default arm uses MissingTestDelta (the primary block reason); the concrete kind
            // is stored in the payload's `kind` field, not recovered from the action_type.
            Self::TddPreflightBlocked(ref p) => SsbActionType::TddPreflightBlocked(p.kind),
            // I-20 (SCHEMA_VERSION 255): budgeted mutation scheduler (4-arm Pattern-2).
            // The concrete kind is stored on the payload's `kind` field; the
            // action_type carries it through so per-arm sql_str round-trips.
            Self::MutationBudgetedRun(ref p) => SsbActionType::MutationBudgetedRun(p.kind),
            Self::SprintContractEvent(ref p) => SsbActionType::SprintContractEvent(p.kind),
            // I-37 (SCHEMA_VERSION 257): per-EARS-requirement verified status (3-arm Pattern-2).
            Self::RequirementVerified(ref p) => SsbActionType::RequirementVerified(p.kind),
            // I-44 (SCHEMA_VERSION 258): zero-retention purge receipt (3-arm Pattern-2).
            Self::RetentionPurged(ref p) => SsbActionType::RetentionPurged(p.kind),
            // I-52 (SCHEMA_VERSION 259): agent checkpoint tool event (4-arm Pattern-2).
            Self::CheckpointToolEvent(ref p) => SsbActionType::CheckpointToolEvent(p.kind),
            // I-54 (SCHEMA_VERSION 260): structural edit tool event (5-arm Pattern-2).
            Self::StructuralEditApplied(ref p) => SsbActionType::StructuralEditApplied(p.kind),
            // I-4 (SCHEMA_VERSION 261): search-ladder branch lifecycle (4-arm Pattern-2).
            Self::SearchNodeEvent(ref p) => SsbActionType::SearchNodeEvent(p.kind),
            // I-7 (SCHEMA_VERSION 262): escalation-cascade lifecycle (4-arm Pattern-2).
            Self::EscalationCascade(ref p) => SsbActionType::EscalationCascade(p.kind),
            // I-22 (SCHEMA_VERSION 264): delta-progress shaped-reward scorer (3-arm Pattern-2).
            Self::ProgressRewardScored(ref p) => SsbActionType::ProgressRewardScored(p.kind),
            // I-24 (SCHEMA_VERSION 266): isomorphic canary anti-gaming event (3-arm Pattern-2).
            Self::PrmCanaryEvent(ref p) => SsbActionType::PrmCanaryEvent(p.kind),
            // I-27 (SCHEMA_VERSION 265): convergence curve flat beacon.
            Self::ConvergenceCurveMeasured(_) => SsbActionType::ConvergenceCurveMeasured,
            // I-36 (SCHEMA_VERSION 268): compiled-prompt artifact event (3-arm Pattern-2).
            // The representative arm is Compiled; action_type carries the concrete kind
            // so per-arm sql_str round-trips via PromptCompiledKind::as_sql_str.
            Self::PromptCompiled(ref p) => SsbActionType::PromptCompiled(p.kind),
            // I-50 (SCHEMA_VERSION 268): subscription auth broker lifecycle event (4-arm Pattern-2).
            // The representative arm is TokenAcquired; kind carries the concrete arm so
            // per-arm sql_str round-trips via AuthBrokerEventKind::as_sql_str.
            Self::AuthBrokerEvent(ref p) => SsbActionType::AuthBrokerEvent(p.kind),
        }
    }
}

/// B-38 — payload for [`SsbActionType::AmbiguityScored`].
///
/// Emitted by the GroundingSweepGate when a non-`None` `AmbiguityScore` is
/// supplied via `TransitionContext.ambiguity_score`. Mirrors the
/// `AmbiguityScore.breakdown` shape but flattened to avoid pulling the
/// `elai-reasoning` crate type into the SSB schema surface.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AmbiguityScoredPayload {
    /// Overall ambiguity score, in `[0.0, 1.0]`, rounded to 4 dp.
    pub score: f32,
    /// Milestone label (`"initial"`, `"progress"`, `"refined"`, `"ready"`).
    pub milestone: String,
    pub goal: f32,
    pub constraint: f32,
    pub success: f32,
    /// `None` for greenfield; `Some(_)` for brownfield.
    pub context: Option<f32>,
    pub brownfield: bool,
}

/// B-38 — payload for [`SsbActionType::AmbiguityFloorFailed`].
///
/// Emitted by the GroundingSweepGate when one or more per-dimension floors
/// were breached even when `overall_score <= AMBIGUITY_THRESHOLD`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AmbiguityFloorFailedPayload {
    pub score: f32,
    /// Component names that fell below their floor (e.g. `"goal_clarity"`).
    pub failed_dimensions: Vec<String>,
}

/// B-38 — payload for [`SsbActionType::AmbiguityStreakAdvanced`].
///
/// Emitted by the GroundingSweepGate when the score qualifies but the
/// auto-complete streak hasn't yet reached the required count. The gate
/// itself returns BLOCK on this case; the row is purely informational so
/// the dashboard can show 1/2 → 2/2 progress.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AmbiguityStreakAdvancedPayload {
    pub score: f32,
    /// The streak count INCLUDING this snapshot (e.g. 1 means the snapshot
    /// is the first ready one in a row).
    pub current_streak: u32,
    /// Required streak count (mirrors
    /// [`elai_reasoning::AUTO_COMPLETE_STREAK_REQUIRED`] cast to u32).
    pub required: u32,
}

/// B-42 — payload for [`SsbActionType::CosmosRefreshStarted`].
///
/// Emitted by `knowledge::cosmos_worker::CosmosWorker::tick` at the start
/// of a refresh sweep. Carries the count of artifacts queued for refresh
/// and the manifest size so the dashboard can render a "N/M refreshing"
/// progress card.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CosmosRefreshStartedPayload {
    /// Number of artifacts identified as stale/missing for this refresh sweep.
    pub queued: u32,
    /// Total number of artifacts in the manifest.
    pub total: u32,
}

/// B-42 — payload for [`SsbActionType::CosmosArtifactUpdated`].
///
/// Emitted by `knowledge::cosmos_worker::CosmosWorker` after a successful
/// artifact refresh (network fetch + body hash + manifest write).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CosmosArtifactUpdatedPayload {
    /// Logical artifact name (e.g. `"saas_pricing_patterns"`).
    pub name: String,
    /// blake3 content hash of the refreshed body.
    pub blake3: String,
    /// Body length in bytes.
    pub body_bytes: u32,
    /// Wall-clock latency in milliseconds for the refresh operation.
    pub latency_ms: u32,
}

/// B-42 — payload for [`SsbActionType::CosmosArtifactDecayed`].
///
/// Emitted when a `cosmos_get(name)` call surfaces an artifact whose age
/// exceeds `3 * cadence_interval`. The artifact is still served; the
/// row is informational so the dashboard can render a DECAYED badge.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CosmosArtifactDecayedPayload {
    /// Logical artifact name.
    pub name: String,
    /// Age in seconds at the time of the decay observation.
    pub age_secs: u64,
    /// Configured cadence (`"daily"`, `"weekly"`, `"monthly"`, `"quarterly"`).
    pub cadence: String,
}

/// B-42 — payload for [`SsbActionType::CosmosGroundingUsed`].
///
/// Emitted by `reasoning::verification::LocalVerifier` whenever a
/// `VerificationMethod::CosmosCheck` dispatch resolves a claim against
/// the cosmos store.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CosmosGroundingUsedPayload {
    /// Claim category (e.g. `"design"`, `"ip_boundary"`, `"monetization"`).
    pub claim_category: String,
    /// Logical artifact name resolved from the lookup.
    pub artifact_name: String,
    /// True when the lookup returned a Fresh-or-Stale artifact (i.e. used
    /// for grounding). False when the lookup returned no matching artifact.
    pub matched: bool,
}

/// B-41 — payload for [`SsbActionType::PerLevelAuditStarted`].
///
/// Emitted by the pipeline-tree navigator immediately before spawning an
/// Auditor* agent at a tier node (Concepts/Phases/Steps/BabySteps/BabyStepDetail).
/// The `mode` and `tier` fields are the snake_case labels from
/// `auditor_mode::AuditorMode::as_str()` and `auditor_mode::TierNodeId::tier_str()`
/// — kept as strings so this payload stays decoupled from the auditor_mode
/// module (avoids a cyclic dep in payload deserialization).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PerLevelAuditStartedPayload {
    /// Auditor mode label: `"concepts" | "phases" | "steps" | "babysteps" |
    /// "babystep_detail" | "plan"`.
    pub mode: String,
    /// Tier label: `"concept" | "phase" | "step" | "baby_step"`.
    pub tier: String,
    /// Underlying tier node id (ConceptId/PhaseId/StepId/BabyStepId raw value).
    pub node_id: u64,
}

/// B-41 — payload for [`SsbActionType::PerLevelAuditVerdict`].
///
/// Emitted by the pipeline-tree navigator after an Auditor* agent returns a
/// `LevelAuditVerdict`. `findings_count` is the length of the verdict's
/// `findings` vec (full Finding list is persisted as Markdown by
/// `auditor_mode::persist_verdict`; SSB carries only the metadata so payload
/// size stays bounded).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PerLevelAuditVerdictPayload {
    pub mode: String,
    pub tier: String,
    pub node_id: u64,
    /// `"allow" | "block"`.
    pub verdict: String,
    pub findings_count: u32,
    pub respawned_once: bool,
    pub cascade_to_parent: bool,
}

/// B-41 — payload for [`SsbActionType::PerLevelAuditCascade`].
///
/// Emitted when a 2nd BLOCK verdict at a tier sets `cascade_to_parent=true`
/// and the navigator bubbles up to the parent decomposer. `parent_tier` is
/// the immediate parent in the decomposition hierarchy
/// (concept ← phase ← step ← baby_step ← baby_step_detail-cascades-to-baby_step).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PerLevelAuditCascadePayload {
    pub mode: String,
    pub tier: String,
    pub node_id: u64,
    /// Tier label of the parent that was bubbled up to.
    pub parent_tier: String,
}

/// B-37C — payload for [`SsbActionType::RuntimeFallback`].
///
/// Emitted by `runtime_orchestrator::RuntimeOrchestrator::try_complete`
/// once per fallback chain step. `role` is the agent role being served,
/// `primary` the originally-configured backend, `attempted` the ordered
/// list of backends tried so far (including the one this row records),
/// `succeeded` is `Some(backend)` on a successful step or `None` for a
/// failure step that triggers chain advancement. `reason` is a short
/// human-readable explanation surfaced from the underlying
/// `RuntimeAdapterError` or success message.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RuntimeFallbackPayload {
    pub role: elai_ipc_types::AgentRole,
    pub primary: elai_providers::runtime::RuntimeBackend,
    pub attempted: Vec<elai_providers::runtime::RuntimeBackend>,
    pub succeeded: Option<elai_providers::runtime::RuntimeBackend>,
    pub reason: String,
}

/// B-37C — payload for [`SsbActionType::RuntimeHealthChanged`].
///
/// Emitted by `runtime_orchestrator::RuntimeOrchestrator::health_snapshot`
/// when a backend's `RuntimeHealth` transitions to a different variant
/// vs the previously cached snapshot. Backed by the `RuntimeHealth`
/// serde shape (tag = "status") so dashboard renderers can decode
/// without reaching into the providers crate.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RuntimeHealthChangedPayload {
    pub backend: elai_providers::runtime::RuntimeBackend,
    pub prev: elai_providers::runtime::RuntimeHealth,
    pub next: elai_providers::runtime::RuntimeHealth,
}

/// B-37C — payload for [`SsbActionType::CliSubprocessSpawned`].
///
/// Emitted by CLI adapters (B-37B `cli_common::spawn`) on every
/// `tokio::process::Command::spawn` call. `argv_redacted` MUST have
/// secret arguments stripped — see `cli_common::redact_argv` — before
/// the row is written; the writer is the last line of defence against
/// accidental secret leakage into the SSB store.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CliSubprocessSpawnedPayload {
    pub backend: elai_providers::runtime::RuntimeBackend,
    pub pid: u32,
    pub argv_redacted: Vec<String>,
}

/// B-37C — payload for [`SsbActionType::CliSubprocessKilled`].
///
/// Emitted by CLI adapters on subprocess termination — either
/// natural exit, `cli_common::kill_and_wait` (idle/startup timeout),
/// or user-driven cancel. `exit_code` is None when the process was
/// signalled.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CliSubprocessKilledPayload {
    pub backend: elai_providers::runtime::RuntimeBackend,
    pub pid: u32,
    pub exit_code: Option<i32>,
    pub reason: String,
}

// ---------------------------------------------------------------------------
// E-51 (SCHEMA_VERSION 96): edge-inference payload structs
// ---------------------------------------------------------------------------

/// E-51 — payload for [`SsbActionType::MlxSubprocessSpawned`].
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MlxSubprocessSpawnedPayload {
    pub pid: u32,
    pub argv_redacted: Vec<String>,
}

/// E-51 — payload for [`SsbActionType::MlxSubprocessKilled`].
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MlxSubprocessKilledPayload {
    pub pid: u32,
    pub exit_code: Option<i32>,
    pub reason: String,
}

/// E-51 — payload for [`SsbActionType::LlamaCppSubprocessSpawned`].
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LlamaCppSubprocessSpawnedPayload {
    pub pid: u32,
    pub ep: String,
    pub argv_redacted: Vec<String>,
}

/// E-51 — payload for [`SsbActionType::LlamaCppSubprocessKilled`].
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LlamaCppSubprocessKilledPayload {
    pub pid: u32,
    pub exit_code: Option<i32>,
    pub reason: String,
}

/// E-51 — payload for [`SsbActionType::MlxApplePlatformDetected`].
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MlxApplePlatformDetectedPayload {
    pub chip_family: String,
    pub gpu_cores: u8,
}

/// E-51 — payload for [`SsbActionType::LlamaCppEpDetected`].
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LlamaCppEpDetectedPayload {
    pub ep: String,
}

/// E-51 — payload for [`SsbActionType::RuntimeChainBuilt`].
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RuntimeChainBuiltPayload {
    pub chain: Vec<elai_providers::runtime::RuntimeBackend>,
}

// ---------------------------------------------------------------------------
// E-17 SP2 (SCHEMA_VERSION 97): RAG retrieval-quality payload structs
// ---------------------------------------------------------------------------

/// E-17 SP2 — payload for [`SsbActionType::DeepPoolCapped`].
///
/// `capped_count` is the total number of candidates dropped across all
/// input stages by `knowledge::deep::deep_pool_cap`. `max_allowed` is the
/// per-stage cap that fired (typically `DEEP_CAP_DEFAULT = 100`).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DeepPoolCappedPayload {
    pub capped_count: u32,
    pub max_allowed: u32,
    pub stage_count: u32,
}

/// E-17 SP2 — payload for [`SsbActionType::RrfQualityGateFired`].
///
/// `dropped_count` is the number of fused candidates dropped below the
/// floor by `knowledge::deep::rrf_quality_gate`; `total_before` is the
/// candidate pool size before the gate ran (post-RRF-accumulator).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RrfQualityGateFiredPayload {
    pub dropped_count: u32,
    pub floor: f64,
    pub total_before: u32,
}

/// E-17 SP2 — payload for [`SsbActionType::HydeSkippedCodeQuery`].
///
/// Emitted when `knowledge::hyde::should_use_hyde` returns false on a
/// CodeSymbol / Mixed query that would otherwise have invoked HyDE
/// (DEEP tier + >= MIN_WORDS_FOR_HYDE).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HydeSkippedCodeQueryPayload {
    pub query_class: String,
    pub word_count: u32,
    pub code_signal_present: bool,
}

/// E-17 SP2 — payload for [`SsbActionType::RrfComponentWeights`].
///
/// One row per query carrying the per-ranker RRF weights active for that
/// fusion run. `None` indicates the ranker did not contribute (e.g.
/// `code_vec_weight` is None when ELAI_CODE_VEC is disabled or no code
/// index is loaded).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RrfComponentWeightsPayload {
    pub bm25_weight: Option<f64>,
    pub code_vec_weight: Option<f64>,
    pub text_vec_weight: Option<f64>,
}

// ---------------------------------------------------------------------------
// C-1 (SCHEMA_VERSION 47): knowledge + reasoning producer payloads
// ---------------------------------------------------------------------------

/// C-1 — payload for [`SsbActionType::GrcStepEmitted`].
///
/// Emitted by `task_runner::emit_reasoning` when a `PipelineGrc` handle is
/// plumbed through `ResolvedConfig`. `agent_id` is sourced from the
/// engine's `AgentGrcConfig.role` (NOT the synthetic `RUNNER_AGENT_ID`
/// constant). `budget_remaining` is the per-topic
/// `GrcBudget::remaining_calls()` value. `elapsed_ms` is the wall-clock
/// time since the runner entered the GRC step (caller supplies; clock is
/// the runner's `ClockProvider`).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GrcStepEmittedPayload {
    pub agent_id: String,
    pub topic: String,
    pub step_type: String,
    pub grounding_level: String,
    pub confidence: f64,
    pub budget_remaining: u32,
    pub elapsed_ms: u64,
}

/// C-1 — payload for [`SsbActionType::GroundingSweepResult`].
///
/// Emitted ONCE per `PipelineGrc::ground_plan` call. Fields:
/// * `ungrounded` — count of claims that remain ungrounded after sweep.
/// * `verify_needed` — count of `training:`-prefixed claims (subset of
///   `ungrounded`) that require manual verification.
/// * `grounded` — count of claims that did verify (
///   `claims.len() - ungrounded.len()`).
/// * `top_claim` — first ungrounded claim text, used by the dashboard
///   GroundingSweepCard for an at-a-glance preview. `None` when all
///   claims are grounded.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GroundingSweepResultPayload {
    pub ungrounded: u32,
    pub verify_needed: u32,
    pub grounded: u32,
    pub top_claim: Option<String>,
}

/// C-1 — payload for [`SsbActionType::CsqEvaluated`].
///
/// Emitted by `task_runner` immediately after `PipelineGrc::generate_csq`
/// returns. `severity` mirrors `CsqSeverity::Critical` / `Warning`
/// (the worst severity among the checks, or `"None"` when the result
/// contains no checks). `guard_present` reflects whether every critical
/// check has `guard_added=true`. `blocked` is `true` when the runner's
/// `hook_pre_blocked` flag was set as a result of the verdict.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CsqEvaluatedPayload {
    pub severity: String,
    pub guard_present: bool,
    pub blocked: bool,
    pub check_count: u32,
}

/// C-1 — payload for [`SsbActionType::AuditorRubberStampRejected`].
///
/// Emitted at the `PipelineGrc` caller when `BasicGrcEngine::advance`
/// returns `GrcError::InsufficientFindings`. `role` is the auditor's
/// `AgentGrcConfig.role` (always `"auditor"` today). `finding_count` is
/// the observed count of `FINDING:` rows in the topic state;
/// `expected_min` is the gate threshold (currently `1`).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AuditorRubberStampRejectedPayload {
    pub role: String,
    pub finding_count: u32,
    pub expected_min: u32,
}

/// C-37 — payload for [`SsbActionType::PerRoleDecodingApplied`].
///
/// Emitted by `agent_pool::spawn` when the spawned role has an explicit
/// per-role decoding policy in
/// `elai_providers::decoding::PER_ROLE_DECODING_DEFAULTS`. Closes GAP-B
/// (default_decoding_for had no production callers pre-C-37). `role` is
/// the canonical snake_case label; `decoding_mode` is the
/// `DecodingMode::discriminant()` string ("free_text" / "grammar" /
/// "json_schema" / "regex"). `constrained` mirrors whether the resolved
/// mode applies any FSM mask (true for grammar / json_schema / regex,
/// false for free_text).
///
/// Paper grounding: arXiv:2604.06066 Section 4.3 ("From Hallucination to
/// Structure Snowballing"). The pilot harness in
/// `.elai_cc/python/elai_inference/c37_snowballing/` measures the
/// downstream effect of these per-role choices.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PerRoleDecodingAppliedPayload {
    /// Canonical snake_case agent role (one of:
    /// designer / executor / critic / auditor / researcher,
    /// or the B-41 per-level auditor variants).
    pub role: String,
    /// The `DecodingMode::discriminant()` value of the applied policy:
    /// one of "free_text" / "grammar" / "json_schema" / "regex".
    pub decoding_mode: String,
    /// True iff the applied mode imposes any FSM mask (every variant
    /// EXCEPT `free_text`).
    pub constrained: bool,
    /// Max retries on validation failure (mirrors
    /// `ConstrainedDecodingConfig::max_retries_on_invalid`).
    pub max_retries_on_invalid: u32,
}

/// C-1 — payload for [`SsbActionType::HydeQuery`].
///
/// Emitted by `HydeStage::hypothesize` per call. `query_hash` is the
/// hex-encoded blake3 hash of the query bytes (full 32 bytes / 64 hex
/// chars). `hypothesis_len` is `0` when no hypothesis was produced
/// (short query / breaker open / sidecar error). `cache_hit=true`
/// indicates the cache TTL window was within bounds.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HydeQueryPayload {
    pub query_hash: String,
    pub hypothesis_len: u32,
    pub cache_hit: bool,
    pub latency_ms: u64,
}

/// C-1 — payload for [`SsbActionType::VectorCacheHitRate`].
///
/// Aggregate hits/misses snapshot flushed by
/// `VectorCache::flush_hit_rate_ssb`. `hit_rate_pct` ranges
/// `0.0..=100.0`. The producer skips emission when `hits + misses == 0`,
/// so every row carries non-zero traffic.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct VectorCacheHitRatePayload {
    pub session_id: String,
    pub hits: u32,
    pub misses: u32,
    pub hit_rate_pct: f64,
}

/// C-1 — payload for [`SsbActionType::QueryTrace`].
///
/// Per-query trace emitted at the end of
/// `QueryPipeline::query_with_tier`. Pairs with the existing
/// `tracing::debug!("query_trace")` line — both fire on every query;
/// the SSB row is the persisted form the dashboard reads via
/// `get_query_trace_recent`. `query_id` is the UUID stringified from
/// `QueryTrace::query_id`. `tier` is one of `"FAST" / "STANDARD" /
/// "DEEP" / "AUTO"`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct QueryTracePayload {
    pub query_id: String,
    pub tier: String,
    pub stage_count: u32,
    pub result_count: u32,
    pub total_latency_ms: u64,
}

// ---------------------------------------------------------------------------
// C-2 — 10 NEW payload structs (orchestration + cosmos + stagnation).
// Defined here so the SCHEMA_VERSION 47 -> 48 bump is atomic for every
// 9-tuple-lockstep row (CHECK / enum / as_sql_str / from_str / SsbPayload /
// action_type / test_payload / all_test_payloads / count assert).
// ---------------------------------------------------------------------------

/// C-2 — payload for [`SsbActionType::PhaseNodeReady`].
///
/// Emitted by the task_runner baby-step loop the first time
/// `nav.next_baby_step` returns a baby-step whose
/// `tree.location_index[bs_id].phase_id` differs from the previously
/// observed phase. `depth` mirrors the navigator's
/// `DecompositionLevel::Phase` depth (always `1` today; reserved for
/// future N-level decompositions). `parent_concept_id` is the
/// containing concept (always present per location_index invariant).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PhaseNodeReadyPayload {
    pub phase_id: u64,
    pub concept_id: u64,
    pub label: String,
    pub depth: u32,
}

/// C-2 — payload for [`SsbActionType::StepNodeReady`].
///
/// Emitted when the task_runner baby-step loop observes a step-id
/// change against the previous iteration. `parent_phase_id` and
/// `parent_concept_id` are the containing decomposition ancestors so
/// the dashboard can render the row inside the correct tree column.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StepNodeReadyPayload {
    pub step_id: u64,
    pub phase_id: u64,
    pub concept_id: u64,
    pub label: String,
    pub depth: u32,
}

/// C-2 — payload for [`SsbActionType::BabyStepReady`].
///
/// Emitted once per baby-step consumption in the task_runner loop
/// (matches the cadence of `emit_reasoning`). Carries the full
/// (concept, phase, step, baby_step) ancestry plus the truncated task
/// label so the dashboard BubbleUpStrip can show a per-baby-step
/// bubble without joining additional rows.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BabyStepReadyPayload {
    pub baby_step_id: u64,
    pub step_id: u64,
    pub phase_id: u64,
    pub concept_id: u64,
    pub label: String,
    pub depth: u32,
}

/// C-2 — payload for [`SsbActionType::DagRepairEscalated`].
///
/// Emitted from `pipeline_builder` when `DagRepairLoop::repair`
/// returns `RepairOutcome::Escalated` after `T_DAG = 10` retries. The
/// row distinguishes "successful repair after N rounds"
/// ([`SsbActionType::DagRepairRound`]) from "gave up after N rounds"
/// so the dashboard can surface a distinct ESCALATED badge.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DagRepairEscalatedPayload {
    /// Number of rounds the repair loop executed before escalating.
    pub rounds: u32,
    /// Count of validation issues that remained unrepaired.
    pub remaining_issues: u32,
    /// Short summary of the last validation error (first remaining issue's
    /// `message`). Empty when no remaining issues were attached.
    pub last_error: String,
}

/// C-2 — payload for [`SsbActionType::AgentStatusChanged`].
///
/// Sibling row to [`AgentLifecyclePayload`], emitted alongside every
/// AgentLifecycle write in `bsp/engine`. Single `state` field per
/// DRIFT-5 in context-package-C2.md (no `old_lifecycle`): consumers
/// derive transitions by reading the ordered stream per `agent_id`.
/// `timestamp_ms` is sourced from the engine's `ClockProvider` (never
/// `SystemTime::now` directly) per ENTROPY_QUARANTINE.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AgentStatusChangedPayload {
    pub agent_id: String,
    /// AgentRole discriminant in snake_case (mirrors `AgentLifecyclePayload.role`).
    pub role: String,
    /// `"spawned" | "ready" | "exited"` — mirrors `AgentLifecyclePayload.state`.
    pub state: String,
    /// Present when `state == "exited"`: `"clean_exit" | "crash" | "timeout" |
    /// "budget_exceeded"`. Mirrors `AgentLifecyclePayload.death_class`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub death_class: Option<String>,
    /// Wall-clock millisecond timestamp sourced from the engine clock.
    pub timestamp_ms: u64,
}

/// C-2 — payload for [`SsbActionType::StagnationSpinningDetected`].
///
/// Defined now so the SCHEMA bump is atomic; producer wiring lands in
/// C-10 (Stagnation Detector). Carries the detector verdict shape used
/// by [`crate::anti_spiral::SpiralTracker`] when the same step_hash
/// repeats 3+ times within the current phase window.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StagnationSpinningDetectedPayload {
    pub agent_id: String,
    pub pattern: String,
    pub iteration: u32,
    pub detail: String,
}

/// C-2 — payload for [`SsbActionType::StagnationOscillationDetected`].
///
/// Defined now; producer wiring lands in C-10. Period-2 oscillation
/// pattern (e.g. step_hash sequence A B A B A).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StagnationOscillationDetectedPayload {
    pub agent_id: String,
    pub pattern: String,
    pub iteration: u32,
    pub detail: String,
}

/// C-2 — payload for [`SsbActionType::LateralThinkingActivated`].
///
/// Defined now; producer wiring lands in C-10 (persona switcher).
/// Carries the persona label that was activated to break a stagnation
/// loop (`"hacker" | "researcher" | "simplifier" | "architect" |
/// "contrarian"`).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LateralThinkingActivatedPayload {
    pub agent_id: String,
    pub persona: String,
    pub iteration: u32,
    pub detail: String,
}

/// C-2 — payload for [`SsbActionType::GoalDriftWarning`].
///
/// Defined now; producer wiring lands in C-10 (Goal Drift Canary).
/// Emitted when the canary detects that the active baby-step's task
/// summary has diverged from the original task intent by more than
/// `drift_score` cosine distance.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GoalDriftWarningPayload {
    pub agent_id: String,
    pub drift_score: f32,
    pub baby_step_id: u64,
    pub detail: String,
}

/// C-2 — payload for [`SsbActionType::WatchdogTimeout`].
///
/// Defined now; producer wiring lands in C-10 (Generation Watchdog).
/// Emitted when an agent exceeds its per-step wall-clock budget
/// without producing observable output.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WatchdogTimeoutPayload {
    pub agent_id: String,
    pub elapsed_ms: u64,
    pub budget_ms: u64,
    pub detail: String,
}

// ---------------------------------------------------------------------------
// C-10 (SCHEMA_VERSION 52) payload structs — anti-spiral lateral-thinking +
// goal-drift Block. Producer wiring lands in anti_spiral_extended.rs +
// task_runner step-boundary loop.
// ---------------------------------------------------------------------------

/// C-10 — payload for [`SsbActionType::LateralThinkingSucceeded`].
///
/// Emitted when a previously-active stagnation pattern stops firing
/// after a persona suggestion was consumed. Carries the persona that
/// broke the loop plus the iteration count at success.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LateralThinkingSucceededPayload {
    pub agent_id: String,
    pub persona: String,
    pub iteration: u32,
    pub detail: String,
}

/// C-10 — payload for [`SsbActionType::LateralThinkingExhausted`].
///
/// Emitted when all 5 personas have been tried within the current task
/// window without breaking the stagnation pattern. Carries the count of
/// personas attempted (always 5 in the current implementation; field
/// kept for forward-compat if the persona roster expands).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LateralThinkingExhaustedPayload {
    pub agent_id: String,
    pub personas_tried: u32,
    pub iteration: u32,
    pub detail: String,
}

/// C-10 — payload for [`SsbActionType::GoalDriftBlock`].
///
/// Companion to [`GoalDriftWarningPayload`] for the Block verdict band.
/// `drift_score` carries the Jaccard-similarity value in the plan-direction
/// reading (see workflow/reasoning/C-10-decisions.md DRIFT-3). Lower
/// score = more drift; `drift_score < 0.4` triggers this Block row.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GoalDriftBlockPayload {
    pub agent_id: String,
    pub drift_score: f32,
    pub baby_step_id: u64,
    pub detail: String,
}

// ---------------------------------------------------------------------------
// C-13 (SCHEMA_VERSION 53) payload struct — WIP commit squash completion.
// Producer wiring lands in `crate::wip_squash::squash_wip_commits_if_any`
// + the `elai wip-squash` CLI subcommand. Emitted unconditionally after a
// squash attempt regardless of the no-op vs. squashed outcome so the
// dashboard can render both rows.
// ---------------------------------------------------------------------------

/// C-13 — payload for [`SsbActionType::WipSquashComplete`].
///
/// Emitted after the leading-WIP-commit squash heuristic runs. `wip_count`
/// is the count of consecutive leading commits whose first message line
/// matches `/^[Ww][Ii][Pp]/`. `squashed` records whether a real squash
/// actually executed:
///   * `squashed == true`  ⇒ a `git reset --soft HEAD~{wip_count}` plus
///     a `git commit --no-verify -m "phase: squash {wip_count} WIP commits"`
///     fired; the working tree's HEAD now sits on a single squash commit.
///   * `squashed == false` ⇒ no-op (wip_count <= 5 or no WIP prefix found).
///
/// `step_id` is the baby-step identifier that triggered the squash; empty
/// when invoked directly via the `elai wip-squash` CLI subcommand.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WipSquashCompletePayload {
    pub wip_count: u32,
    pub squashed: bool,
    pub step_id: String,
}

// ---------------------------------------------------------------------------
// C-22 (SCHEMA_VERSION 54) payload structs — feasibility-probe completeness
// ---------------------------------------------------------------------------

/// C-22 — payload for [`SsbActionType::AlternativeExplored`].
///
/// Emitted by `DefaultFeasibilityProber::probe` per NovelStep alternative
/// attempted (i.e. each branch the prober tried in the alternatives chain).
/// `alternative_index` is 0-indexed (0 = first attempt, the original step).
/// Up to `MAX_ALTERNATIVES_PER_STEP` (3) rows per NovelStep.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AlternativeExploredPayload {
    pub step_id: String,
    pub alternative_index: u8,
    pub accepted: bool,
}

/// C-22 — payload for [`SsbActionType::FeasibilityEscalated`].
///
/// Emitted by the feasibility-probe escalation path when
/// `decide_with_mode` returns `FeasibilityDecision::Escalate` (Interactive
/// mode) or `Replan` (Autonomous mode). `variant` is the
/// `FeasibilityEscalation` discriminant tag in snake_case.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FeasibilityEscalatedPayload {
    pub reason: String,
    pub variant: String,
    pub step_id: String,
}

// ---------------------------------------------------------------------------
// C-23 (SCHEMA_VERSION 55) payload structs — SessionDag branch-UI beacons.
// ---------------------------------------------------------------------------

/// C-23 — payload for [`SsbActionType::BranchForked`].
///
/// Emitted at the orchestrator level (via `create_branch_with_ssb`) and by
/// the dashboard-v2 `fork_session_branch` Tauri command on successful
/// `SessionDag::create_branch`. `branch_name` is the user-facing branch
/// label (matches `BranchRef::name`). `tip_snap_id` is the freshly-created
/// snapshot id at the branch tip. `parent_snap_id` is the snapshot the
/// branch was forked from.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BranchForkedPayload {
    pub branch_name: String,
    pub tip_snap_id: String,
    pub parent_snap_id: String,
}

/// C-23 — payload for [`SsbActionType::BranchMerged`].
///
/// Emitted by `SessionDag::merge_branch_with_ssb` on a successful
/// `MergeResult::Winner`. Supersedes the legacy `Checkpoint` row with a
/// `branch_merge:winner=...;abandoned=...` discriminator string (the
/// Checkpoint row was indistinguishable from a snapshot-save row in the
/// dashboard activity feed).
///
/// `winner_name` is the branch that survived the merge into trunk.
/// `trunk_snap_id` is the trunk snapshot id post-merge. `abandoned_names`
/// is the set of non-winner active branches at the time of the merge.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BranchMergedPayload {
    pub winner_name: String,
    pub trunk_snap_id: String,
    pub abandoned_names: Vec<String>,
}

/// C-23 — payload for [`SsbActionType::BranchClosed`].
///
/// Emitted by `SessionDag::close_branch_with_ssb` on a successful
/// `close_branch` call. `reason` carries a short free-text explanation
/// (e.g. `"user-requested"`, `"dashboard:close_session_branch"`) so the
/// dashboard activity feed can render context without inspecting other
/// rows.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BranchClosedPayload {
    pub branch_name: String,
    pub reason: String,
}

// ---------------------------------------------------------------------------
// C-25 (SCHEMA_VERSION 56) payload structs — compiled-pipeline +
// template-crystallizer + background-task beacons.
// ---------------------------------------------------------------------------

/// C-25 — source discriminator for [`PlanSkeletonPayload::source`].
///
/// Distinguishes the two pipeline-builder factory paths:
///   * `Llm`      — the standard LLM-decomposition path (existing behaviour).
///   * `Template` — the C-25 `CompiledExecutor` cache hit short-circuit.
/// Tagged via `#[serde(rename_all = "snake_case")]` so the wire form is
/// `"llm"` / `"template"` for dashboard/CLI parsing.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PlanSkeletonSource {
    Llm,
    Template,
}

/// C-25 — payload for [`SsbActionType::PlanSkeleton`].
///
/// Emitted by `pipeline_builder::PipelineBuilder::build_with_emitter` on
/// every successful factory entry. `source` flips to
/// [`PlanSkeletonSource::Template`] when the C-25 `CompiledExecutor`
/// returns a cache hit; otherwise it is [`PlanSkeletonSource::Llm`]. C-25
/// ships the FIRST emitter for this row (see DRIFT-D in
/// `workflow/sessions/s-25672/context-package-C25.md`).
///
/// `plan_skeleton_hash` is a blake3-truncated-u64 over the spec name +
/// concatenated step names (stable across rustc versions — DefaultHasher
/// is NOT used, per C-25 Q5 resolution).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PlanSkeletonPayload {
    pub plan_id: String,
    pub step_count: u32,
    pub concept_class: String,
    pub plan_skeleton_hash: u64,
    pub source: PlanSkeletonSource,
}

/// C-25 — payload for [`SsbActionType::CompiledPipelineFired`].
///
/// Emitted by `compiled_pipeline::CompiledExecutor::try_execute` on a
/// successful cache hit followed by a successful run. `use_count` is the
/// `CompiledPipeline.use_count` AFTER the increment.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CompiledPipelineFiredPayload {
    pub pipeline_id: String,
    pub concept_class: String,
    pub plan_skeleton_hash: u64,
    pub use_count: u32,
}

/// C-25 — payload for [`SsbActionType::TemplateCrystallized`].
///
/// Emitted by `compiled_pipeline::TemplateCrystallizer::check_crystallization`
/// when the N-consecutive-success threshold (default N=3) is met and a new
/// CompiledPipeline is materialized.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TemplateCrystallizedPayload {
    pub concept_class: String,
    pub plan_skeleton_hash: u64,
    pub pipeline_id: String,
    pub threshold_n: u32,
}

/// C-25 — payload for [`SsbActionType::BackgroundTaskStarted`].
///
/// Emitted by `background_task::BackgroundTaskManager::gated_spawn` on a
/// successful (autonomous_mode=true) spawn. `command_summary` is the
/// caller-supplied free-text command label (already truncated to fit
/// PAYLOAD_MAX_BYTES). `soft_timeout_secs` is the auto-detach boundary;
/// `hard_timeout_secs` is the kill boundary.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BackgroundTaskStartedPayload {
    pub task_id: String,
    pub command_summary: String,
    pub soft_timeout_secs: u32,
    pub hard_timeout_secs: u32,
}

/// C-25 — payload for [`SsbActionType::BackgroundTaskComplete`].
///
/// Emitted by `background_task::BackgroundTaskManager::poll_completed`
/// (normal exit path) AND `enforce_timeouts` (auto-detach path). `auto_detached`
/// is `true` when the manager dropped the JoinHandle at the soft boundary;
/// `timed_out` is `true` when the hard boundary was exceeded.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BackgroundTaskCompletePayload {
    pub task_id: String,
    pub exit_status: i32,
    pub duration_secs: u32,
    pub auto_detached: bool,
    pub timed_out: bool,
}

/// C-25 audit follow-up — trigger reason for [`DecompileTriggeredPayload`].
///
/// `Confidence` fires when the cached pipeline's `first_try_success_rate`
/// is below `COMPILED_EXECUTOR_CONFIDENCE_THRESHOLD`.
/// `ConsecutiveFailures` fires when `record_failure` was called at least
/// `COMPILED_EXECUTOR_FAILURE_THRESHOLD` times since the last
/// `record_success` reset.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DecompileTriggerReason {
    Confidence,
    ConsecutiveFailures,
}

/// C-25 audit follow-up — payload for [`SsbActionType::DecompileTriggered`].
///
/// Emitted by `compiled_pipeline::CompiledExecutor::emit_decompile_triggered`
/// whenever a cache HIT is rejected via the confidence floor or the
/// consecutive-failure threshold. `failure_count` is the observed
/// consecutive-failure counter at the time the row was emitted (0 for the
/// confidence-floor path, which does not require accumulated failures).
///
/// Closes WARN-1 from `workflow/sessions/s-25672/audit-C25.md` where the
/// C-25 v56 implementation routed this event through `AntiSpiralHit` and
/// dashboards / contract tests filtered on `decompile_triggered` saw
/// zero rows.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DecompileTriggeredPayload {
    pub concept_class: String,
    pub plan_skeleton_hash: u64,
    pub failure_count: u32,
    pub trigger_reason: DecompileTriggerReason,
}

// ---------------------------------------------------------------------------
// C-26 (SCHEMA_VERSION 58) payload structs — constrained-decoding + futility
// ---------------------------------------------------------------------------

/// C-26 — payload for [`SsbActionType::FutilityDetected`].
///
/// Emitted by `pipeline_tree_navigator::NavigatorTree::record_baby_step_outcome`
/// when the per-baby-step same-error-class window saturates at the
/// `RetryStrategy.futility_same_error_threshold` (default 3). The producer
/// also issues a `BubbleUpAction::Escalate` and increments the cascade
/// counter at the surrounding level (matches the C-26 plan §Task 3 path A).
///
/// Fields:
///   - `error_class`         snake_case discriminant (e.g. `"compile_error"`,
///                           `"constrained_decode_reject"`); free-form
///                           string for forward-compat.
///   - `consecutive_count`   number of consecutive same-class failures at
///                           the moment the futility row was emitted.
///                           Equal to the threshold the navigator policy
///                           set.
///   - `level`               `"baby_step" | "step" | "phase" | "concept"`.
///   - `escalated`           `true` when the navigator also issued a
///                           `BubbleUpAction::Escalate` row in the same
///                           cycle. C-26 plan path A always sets `true`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FutilityDetectedPayload {
    pub error_class: String,
    pub consecutive_count: u32,
    pub level: String,
    pub escalated: bool,
}

/// C-26 — payload for [`SsbActionType::FutilityOutputStagnation`].
///
/// Emitted by `pipeline_tree_navigator::NavigatorTree::record_baby_step_outcome`
/// when `GoalDriftCanary::compute(prev_output, curr_output) > 0.90` between
/// two consecutive outputs of the same baby step. The producer also sets
/// `TreeNavigatorState.suggested_persona` via
/// `suggest_persona_for_pattern(StagnationPattern::NoDrift)`, which per
/// C-10 D4 always yields `ThinkingPersona::Hacker` (NOT Researcher).
///
/// Fields:
///   - `similarity`   Jaccard similarity in `[0.0, 1.0]`.
///   - `threshold`    the policy threshold (default `0.90`).
///   - `level`        `"baby_step" | "step" | "phase" | "concept"`.
///   - `new_persona`  snake_case persona label set by the producer (see
///                    `ThinkingPersona::as_str()` — defaults to `"hacker"`
///                    for NoDrift). `None` when C-10 is unavailable.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FutilityOutputStagnationPayload {
    pub similarity: f32,
    pub threshold: f32,
    pub level: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub new_persona: Option<String>,
}

// ---------------------------------------------------------------------------
// C-7 (SCHEMA_VERSION 49) payload struct
// ---------------------------------------------------------------------------

/// C-7 — payload for [`SsbActionType::ChangeDecision`].
///
/// Emitted by the dashboard-v2 Tauri commands `accept_change` /
/// `decline_change` / `retry_audit` each time a human reviewer acts on
/// a per-file or per-hunk change row in ReviewWindow. The live diff
/// remains carried by [`SsbActionType::FileWrite`]; this row records
/// only the reviewer verdict so the dashboard can re-render verdict
/// state on reload + downstream consumers can audit reviewer actions.
///
/// `decision` discriminator (string for forward-compat — adding a new
/// verdict tag does not require a schema bump):
///   - `"accepted"` — emitted by `accept_change`
///   - `"declined"` — emitted by `decline_change`
///   - `"retried"`  — emitted by `retry_audit`
///
/// `hunk_id` is `None` for file-level verdicts and `Some(id)` for
/// per-hunk verdicts. The id is opaque to the orchestrator; the
/// dashboard derives it from the FileWrite diff (typically the
/// `@@ -N,M +P,Q @@` header content hash).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ChangeDecisionPayload {
    pub decision: String,
    pub path: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub hunk_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub note: Option<String>,
}

// ---------------------------------------------------------------------------
// C-8 (SCHEMA_VERSION 50) payload struct
// ---------------------------------------------------------------------------

/// C-8 — payload for [`SsbActionType::SearchModeChanged`].
///
/// Emitted by the CLI startup-readiness path and by the dashboard-v2
/// Tauri command `set_search_mode` whenever the resolved retrieval mode
/// transitions. The dashboard `get_search_mode` command reads the latest
/// row of this type (graceful empty fallback returns `"hybrid"`). This
/// is the SSB-row bridge between the live_cli AppState (CLI process)
/// and the dashboard-v2 AppState (src-tauri process) — the two share no
/// in-memory state, so the canonical channel is the SSB DB.
///
/// `mode` discriminator (lowercase string for forward-compat — adding a
/// new SearchMode tag does not require a schema bump):
///   - `"hybrid"`   — bm25 + vector + reranker (default)
///   - `"lexical"`  — bm25 only (e.g. embedder offline)
///   - `"semantic"` — vector only (e.g. bm25 corrupted)
///   - `"ann"`      — ANN approximate-NN fallback
///
/// `history` is the trailing sequence of mode tags (most-recent last),
/// so the dashboard can render a "last N transitions" timeline without
/// N round-trips. May be empty on the first transition.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SearchModeChangedPayload {
    pub mode: String,
    #[serde(default)]
    pub history: Vec<String>,
}

/// C-5 — payload for [`SsbActionType::RedactionApplied`].
///
/// Emitted by [`crate::cloud_mode::RedactingBabyStepExecutor`] each
/// time the privacy redactor scrubs PII or secrets from a cloud-bound
/// baby-step payload. Best-effort: a write failure never aborts the
/// baby-step execution.
///
/// Fields:
///   - `count`: total number of redaction substitutions applied across
///     every redacted field on this baby-step (sum of per-field deltas).
///   - `fields`: logical field names that were touched. Today this is
///     either `["task_summary"]` or `["task_summary", "escalation_message"]`
///     depending on whether the baby-step carried an escalation message.
///     Forward-compatible: new redactable fields will simply append.
///   - `baby_step_id`: numeric id from
///     [`crate::task_runner::BabyStepExecutionCtx::baby_step_id`] so the
///     dashboard CloudPanel privacy-event feed can correlate each row
///     with its source step.
///   - `ts_ms`: wall-clock millisecond timestamp at redaction time. Set
///     by the producer (RedactingBabyStepExecutor) using its injected
///     clock so deterministic tests stay stable.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RedactionAppliedPayload {
    pub count: u64,
    #[serde(default)]
    pub fields: Vec<String>,
    pub baby_step_id: u64,
    pub ts_ms: i64,
}

// ---------------------------------------------------------------------------
// C-27 (SCHEMA_VERSION 59) — Testing-trust T(-1) + T-0 payloads.
// ---------------------------------------------------------------------------

/// C-27 — kind discriminator for [`WiringViolationPayload`].
///
/// `StubPattern`        — StubBanAuditor matched a forbidden regex in a
///                        production-path source file (todo!(),
///                        unimplemented!(), `Ok(Default::default())` exact-line,
///                        "deferred", "stub").
/// `UnreachableSymbol`  — DkgReachabilityAuditor could not BFS from any
///                        configured entrypoint to the declared
///                        produces_symbol.
/// `BrokenObservable`   — ObservableEffectRunner timed out waiting for the
///                        declared expect_ssb / expect_artifact row (no panic,
///                        violation row emitted instead).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum WiringViolationKind {
    StubPattern,
    UnreachableSymbol,
    BrokenObservable,
}

/// C-27 — payload for [`SsbActionType::WiringEvidence`].
///
/// Emitted by `wiring_contract::DkgReachabilityAuditor` once per
/// produces_symbol declared in a WIRING_CONTRACT block that the auditor
/// verified is reachable (BFS) from at least one configured
/// `[wiring.entrypoints]` symbol path. Zero rows when the contract declares
/// no produces_symbols.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WiringEvidencePayload {
    /// The configured entrypoint that reached the verified symbols.
    pub entrypoint: String,
    /// Symbols from `produces_symbols` that BFS proved reachable.
    #[serde(default)]
    pub produces_symbols_verified: Vec<String>,
    /// Total reachable count under this entrypoint (>= verified.len()).
    pub reachable_count: u32,
    /// Per-`elai wiring audit` invocation correlator.
    pub audit_id: uuid::Uuid,
}

/// C-27 — payload for [`SsbActionType::WiringViolation`].
///
/// Emitted by any of the three T-0 auditors when a wiring contract clause is
/// breached. `violation_kind` discriminates the three branches (StubPattern
/// | UnreachableSymbol | BrokenObservable). `file` + `line` carry the
/// location of the breach (file path for StubPattern; declaring contract
/// source for the other two). `pattern` is the breached pattern string
/// (forbidden regex literal for StubPattern; symbol name for
/// UnreachableSymbol; observable trigger name for BrokenObservable).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WiringViolationPayload {
    pub violation_kind: WiringViolationKind,
    pub file: String,
    pub line: u32,
    pub pattern: String,
    pub audit_id: uuid::Uuid,
}

/// C-27 — payload for [`SsbActionType::DeterminismDrift`].
///
/// Emitted by `determinism::DriftComparator::compare` when two
/// `ActionTypeSequence`s recorded from nominally-equivalent runs diverge.
/// `seed` is the shared `ELAI_SEED` under test; `run_a_hash` /
/// `run_b_hash` are blake3 hashes of the joined snake-case action_type
/// stream up to (but not including) `first_divergence_index`. Identical
/// sequences emit zero rows (contract: no false positives).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DeterminismDriftPayload {
    pub seed: u64,
    pub run_a_hash: String,
    pub run_b_hash: String,
    pub first_divergence_index: u32,
    pub audit_id: uuid::Uuid,
}

// ---------------------------------------------------------------------------
// C-28 (SCHEMA_VERSION 60) — Testing-trust T-1 mutation testing payload.
// ---------------------------------------------------------------------------

/// C-28 — payload for [`SsbActionType::MutationSurvived`].
///
/// Emitted by `deterministic_auditor` Step 6 (mutation check) once per
/// mutant whose `MutantOutcome` is `Survived`. Caught / Timeout / Skipped
/// outcomes emit ZERO rows (stub_ban contract: no spam on healthy runs).
/// `file` is the source path relative to project_root; `line` is the
/// 1-based source line targeted by the mutation; `mutant_kind` is a
/// stable snake_case identifier emitted by the underlying tool (e.g.
/// `replace_+_with_-` for cargo-mutants binary-op replacements, or
/// `delete_line` for mutmut statement deletions).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MutationSurvivedPayload {
    pub file: String,
    pub line: u32,
    pub mutant_kind: String,
}

// ---------------------------------------------------------------------------
// C-29 (SCHEMA_VERSION 61) — Testing-trust T-2 (IntentPair) + T-3 (Metamorphic)
// payloads. See workflow/reasoning/C-29-decisions.md.
// ---------------------------------------------------------------------------

/// C-29 — payload for [`SsbActionType::IntentConfirmed`].
///
/// Emitted when a `BabyStepSpec.intent_pairs` set passes the
/// `IntentConfig.min_pairs` threshold at the PLANNING -> TESTING_FIRST
/// gate (`IntentPairsGate` in `evidence.rs`). `confirmed_by` is one of
/// `"user"` (dashboard Confirm button) or `"autonomous_gate"` (auto-confirm
/// branch). `pair_count` is the count of populated pairs at confirmation
/// time and must be >= the active `min_pairs` (the gate guarantees this).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct IntentConfirmedPayload {
    pub step_id: u64,
    pub pair_count: u8,
    pub confirmed_by: String,
}

/// C-29 — payload for [`SsbActionType::MrViolated`].
///
/// Emitted by `deterministic_auditor` Step 7 when a metamorphic relation
/// returned `MrResult { passed: false }`. `relation` is one of
/// `"identity"` / `"inverse"` / `"permutation_invariance"`. `counterexample`
/// MUST be non-empty — `MrResult::failed` debug_asserts non-emptiness and
/// substitutes `"<empty>"` as a runtime sentinel so the SSB writer never
/// emits a row that violates the stub_ban contract (DRIFT-C29-11).
/// `sample_size` is the number of inputs the relation was evaluated over.
/// MrViolated is NOT a standalone Block — score aggregation deferred to
/// T-4 per C-29 plan §13(c).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MrViolatedPayload {
    pub step_id: u64,
    pub relation: String,
    pub counterexample: String,
    pub sample_size: u32,
}

// ---------------------------------------------------------------------------
// C-30 (SCHEMA_VERSION 62) — Surgical Mode + Oracle Mode pipeline payloads.
// See workflow/reasoning/C-30-decisions.md.
// ---------------------------------------------------------------------------

/// C-30 — payload for [`SsbActionType::SurgicalPlanCreated`].
///
/// Emitted once per successful `SurgicalDecomposer::decompose` return. The
/// payload pins the canonical SurgicalPlan shape so the dashboard
/// SurgicalPanel and downstream auditors can reconstruct the plan summary
/// without re-running the DKG traversal.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SurgicalPlanCreatedPayload {
    /// Stable identifier for the originating `/task` invocation
    /// (`task_<unix_secs>_<short_hash>` per surgical_decomposer.rs).
    pub task_id: String,
    /// Lowercase identifier: `"simple"` | `"medium"` | `"complex"`.
    pub complexity_class: String,
    /// Total files in `AffectedSet.direct_files + dependent_files`.
    pub blast_radius_files: u32,
    /// Sum of `(end_line - start_line + 1)` across all affected DKG nodes.
    pub total_loc_estimate: u32,
    /// Number of `SurgicalStep` entries (capped per complexity class).
    pub step_count: u32,
    /// Hex-encoded blake3 hash of the `RegressionGate.must_pass` vec
    /// (deterministic for identical must_pass content).
    pub regression_gate_snapshot_hash: String,
}

/// C-30 — payload for [`SsbActionType::SurgicalLocWarning`].
///
/// Emitted when `AffectedSet.total_loc_estimate` exceeds the
/// `SURGICAL_LOC_WARN_THRESHOLD` (500). Carries the ExecutionMode so the
/// dashboard can render distinct treatments for Interactive (user must
/// confirm) vs Autonomous (may escalate).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SurgicalLocWarningPayload {
    pub task_id: String,
    pub total_loc_estimate: u32,
    pub threshold: u32,
    /// `"interactive"` | `"autonomous"`.
    pub mode: String,
}

/// C-30 — payload for [`SsbActionType::OracleCacheHit`].
///
/// Emitted when `OracleAnswerCache::get(&key, now)` returns `Some(&entry)`
/// with `now - entry.cached_at < ttl_for_tier(entry.source_tier)`. The
/// `query_hash_hex` is the 64-char blake3 hex of the cache key's query bytes
/// (the project_hash component is kept private to avoid leaking path info
/// via the SSB stream).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct OracleCacheHitPayload {
    pub query_hash_hex: String,
    /// Lowercase tier identifier: `"t1_authoritative"` | `"t2_strong"` |
    /// `"t3_weak"`. T4 / Unknown never cache, so they cannot appear here.
    pub source_tier: String,
    /// Age of the cached entry in seconds at lookup time.
    pub cached_age_secs: u64,
    /// I-15: cache layer discriminator. `"blake3_exact"` | `"semantic_vector"` |
    /// `None` (legacy pre-I-15 row).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source: Option<String>,
}

/// C-30 — payload for [`SsbActionType::OracleCacheMiss`].
///
/// Emitted alongside `OracleAnswerCache::insert(...)` after a get-returns-None
/// has been followed by a successful `run_research_pipeline` call. `confidence`
/// is the lowercase serde form of `AnswerConfidence` (`"high"` | `"medium"`
/// | `"low"` | `"unknown"`).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct OracleCacheMissPayload {
    pub query_hash_hex: String,
    pub confidence: String,
    /// I-15: discriminator — always `"semantic_vector"` on the miss path
    /// (the full vector pipeline ran).  `None` = legacy C-30 path.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source: Option<String>,
}

/// C-30 — payload for [`SsbActionType::ProjectDetectionSignal`].
///
/// Emitted once per signal evaluated by the fresh-project detector when
/// `IndexingService::indexes_exist()` returns `false` at the first `/task`
/// invocation. `weight` mirrors the `SignalWeight` enum
/// (`"required"` | `"strong"` | `"weak"`). `fired = true` means the signal
/// matched (e.g. `Cargo.toml` present, or `rust_file_count >= 10`).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProjectDetectionSignalPayload {
    pub signal_name: String,
    pub weight: String,
    pub fired: bool,
}

// ---------------------------------------------------------------------------
// C-32 (SCHEMA_VERSION 63) — Code-Completeness Layer 2 payloads.
// See workflow/reasoning/C-32-decisions.md.
// ---------------------------------------------------------------------------

/// C-32 — payload for [`SsbActionType::SkeletonGenerated`].
///
/// Emitted once per successful `SkeletonGenerator::generate_for_functions`
/// return. The orchestrator pre-computes the file shape (function
/// signatures + BLANK_N body placeholders) and surfaces this row so the
/// dashboard can render "skeleton with N blanks" before any fill begins.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SkeletonGeneratedPayload {
    pub blank_count: u32,
    /// Lowercase language tag (`"rust"` | `"python"` | ...).
    pub language: String,
    /// Target path (UTF-8 lossy; absolute or workspace-relative).
    pub file_path: String,
}

/// C-32 — payload for [`SsbActionType::SkeletonFlushed`].
///
/// Emitted once per `SkeletonGenerator::flush_to_disk` success. The fill
/// loop performs ZERO disk writes; this single `fs::write` is the only
/// disk-side effect.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SkeletonFlushedPayload {
    pub path: String,
    pub completeness_ratio: f32,
}

/// C-32 — payload for [`SsbActionType::MiniCriticVerdict`].
///
/// Emitted once per `MiniCritic::check`. Carries the verdict tri-state
/// (`"pass"`/`"warn"`/`"block"`) and the short reason tag the dashboard
/// ReasoningDrawer renders.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MiniCriticVerdictPayload {
    pub blank_id: u8,
    pub verdict: String,
    pub reason: String,
    pub language: String,
}

/// C-32 — payload for [`SsbActionType::StructuralRetryStageAttempt`].
///
/// Emitted once per StructuralRetrySequencer stage invocation. The
/// `stage` field is one of `"decompose"`, `"template"`, `"few_shot"`,
/// `"best_of_n"`. The `outcome` field is the lowercase variant tag from
/// `StructuralRetryOutcome` (or `"converged"` / `"exhausted"` /
/// `"accept_and_complete"`).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StructuralRetryStageAttemptPayload {
    pub blank_id: u8,
    pub stage: String,
    pub attempt: u32,
    pub outcome: String,
}

/// C-32 — payload for [`SsbActionType::BridgeFileGenerated`].
///
/// Emitted once per `BridgeFileTemplate::write_bridge` success. Carries
/// the bridge kind, the written path, and the module count assembled into
/// the bridge file body.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BridgeFileGeneratedPayload {
    /// Lowercase kind tag (`"mod_rs_aggregator"` | `"init_py_barrel"` |
    /// `"index_ts_barrel"` | `"go_package_doc"`).
    pub kind: String,
    pub path: String,
    pub module_count: u32,
}

// ---------------------------------------------------------------------------
// D-1 (SCHEMA_VERSION 64) — ReasoningTemplate ArchiveChain lifecycle payloads.
// ---------------------------------------------------------------------------

/// D-1 — payload for [`SsbActionType::ArchiveEntryAdded`].
///
/// Emitted once per [`crate::archive_chain::ArchiveChain::append`] success.
/// All hash strings are 64-char lowercase hex (Blake3); enum strings are
/// snake_case matching the wire labels defined on
/// `reasoning_template::{Domain,PatternKind,ArchiveStatus}`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ArchiveEntryAddedPayload {
    pub template_id: String,
    pub content_hash: String,
    pub parent_hash: Option<String>,
    pub domain: String,
    pub pattern_kind: String,
    pub archive_status: String,
}

/// D-1 — payload for [`SsbActionType::ArchiveEntryRetired`].
///
/// Emitted once when a template transitions to `Retired`. `retired_at_ms` is
/// unix-millis from the injected [`elai_core::clock::ClockProvider`].
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ArchiveEntryRetiredPayload {
    pub template_id: String,
    pub retired_at_ms: i64,
    pub retirement_reason: String,
}

/// D-1 — payload for [`SsbActionType::TemplatePromoted`].
///
/// D-5 BAI Thompson promotion is the producer (lands in a later session);
/// the wire surface is added now so the schema bump is atomic.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TemplatePromotedPayload {
    pub template_id: String,
    pub stage: String,
    pub pass_rate: f64,
}

/// D-1 — payload for [`SsbActionType::TemplateCandidateRejected`].
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TemplateCandidateRejectedPayload {
    pub template_id: String,
    pub rejection_reason: String,
}

/// E-5 — payload for [`SsbActionType::MemoryFactExtracted`].
///
/// Emitted by the orchestrator's per-turn extraction hook right after
/// [`elai_memory::FactStore::insert`] returns Ok. Carries the row's
/// stable `fact_id` plus the indexed columns the dashboard MemoryDrawer
/// renders without re-querying the on-disk store.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MemoryFactExtractedPayload {
    /// UUIDv4 hex of the new row.
    pub fact_id: String,
    /// Snake-case [`elai_memory::FactType`] tag.
    pub fact_type: String,
    /// Snake-case [`elai_memory::Scope`] tag (`project` | `global`).
    pub scope: String,
    /// Optional project id (None for `scope=global`).
    pub project_id: Option<String>,
    /// User id (`"default"` on single-user installs).
    pub user_id: String,
    /// Stable key used for duplicate detection / UPDATE dispatch.
    pub key: String,
    /// Persisted trust score at row creation.
    pub trust_score: f32,
}

/// E-5 — payload for [`SsbActionType::MemoryFactSuperseded`].
///
/// Emitted by the orchestrator's update dispatcher right after
/// [`elai_memory::FactStore::supersede`] returns Ok. Both `fact_id`s are
/// surfaced so the dashboard MemoryDrawer can draw the supersession edge.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MemoryFactSupersededPayload {
    /// `fact_id` of the row that has just been closed (`valid_to_ms != null`).
    pub old_fact_id: String,
    /// `fact_id` of the new currently-valid row.
    pub new_fact_id: String,
    /// `valid_to_ms` written on the old row.
    pub valid_to_ms: u64,
}

/// D-2 — payload for [`SsbActionType::PpmScored`].
///
/// Emitted by `crate::ppm_client::emit_ppm_score` after every successful
/// PPM scoring call that clears the configured threshold. `mode` is the
/// snake_case form of [`crate::sidecar_ppm_hook::PpmMode`]
/// (`"math"` / `"code"`). `score_mean` / `score_var` are widened to f64
/// for the SSB wire format (intrinsic values originate as f32 from the
/// sidecar; widening is lossless).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PpmScoredPayload {
    pub candidate_id: String,
    pub verifier_id: String,
    pub mode: String,
    pub score_mean: f64,
    pub score_var: f64,
    pub num_steps: u32,
    pub latency_ms: u64,
    pub cache_hit: bool,
}

/// D-2 — payload for [`SsbActionType::PpmGatingFailed`].
///
/// Emitted by `crate::ppm_client::emit_ppm_score` when the PPM `score_mean`
/// is below the configured `threshold`. Carries both numbers so dashboards
/// can render the gap without re-loading the threshold config.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PpmGatingFailedPayload {
    pub candidate_id: String,
    pub verifier_id: String,
    pub mode: String,
    pub score_mean: f64,
    pub threshold: f64,
    pub num_steps: u32,
}

/// E-1 — payload for [`SsbActionType::ChatReflectionCompleted`].
///
/// Emitted by the orchestrator's chat-turn-committed hook right after
/// `elai_reasoning::chat_memory_reflect::reflect` returns Ok. Carries
/// the counts the dashboard ChatMemoryDrawer renders without
/// re-querying the underlying stores.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ChatReflectionCompletedPayload {
    /// UUIDv4 hex of the new `memory_notes` row.
    pub note_id: String,
    /// First ~80 chars of the note body (for the SSB log view).
    pub note_preview: String,
    /// Number of [`elai_memory::ExtractedFact`] candidates the heuristic
    /// emitted in the same reflection pass.
    pub facts_extracted: u32,
    /// `decay_tick` rows examined.
    pub decay_rows_examined: u32,
    /// `decay_tick` rows whose persisted trust score actually moved.
    pub decay_rows_updated: u32,
    /// Byte-size of the CoreBlock body after the post-reflection
    /// append (head-truncated to [`elai_memory::CORE_BLOCK_MAX_BYTES`]).
    pub core_block_bytes: u32,
}

/// E-1 — payload for [`SsbActionType::MemoryEvolutionRun`].
///
/// Emitted on every IDLE -> SLEEP_COMPUTE substate entry AND the
/// matching SLEEP_COMPUTE -> IDLE exit. `phase` is `"start"` or
/// `"end"`; `duration_ms` is `0` on `"start"` and the elapsed time
/// from start to end on `"end"`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MemoryEvolutionRunPayload {
    /// `"start"` | `"end"`.
    pub phase: String,
    /// Elapsed ms from the matching `"start"` row (0 on `"start"`).
    pub duration_ms: u64,
    /// Number of `memory_notes` rows whose `last_accessed_ms` the
    /// PPR walker bumped during this evolution pass.
    pub notes_touched: u32,
}

/// E-1 dashboard slice — payload for [`SsbActionType::MemoryNoteInserted`].
///
/// Emitted by the orchestrator chat-turn-committed hook right after
/// `elai_memory::NoteStore::insert` returns Ok. The producer wiring
/// itself lives in `orchestrator/src/task_runner.rs` and is owned by
/// another orchestration session; this payload is shipped now so the
/// dashboard surface can subscribe without further schema churn.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MemoryNoteInsertedPayload {
    /// UUIDv4 hex of the inserted `memory_notes` row.
    pub note_id: String,
    /// `"session"` | `"project"` | `"global"`.
    pub scope: String,
    /// BLAKE3 hex of the note body (so the dashboard can dedupe across
    /// reflection passes without re-fetching the row).
    pub content_blake3: String,
    /// Number of Zettelkasten links the note carries at insert time
    /// (PPR walker adjacency seed count).
    pub link_count: u32,
}

/// E-1 dashboard slice — payload for [`SsbActionType::MemoryNoteEvolved`].
///
/// Emitted by the A-MEM note evolution rewrite pass — the PPR-walker
/// attached link/edit step described in arXiv:2502.12110 §3.3.
/// Producer wiring is in the same task_runner.rs hook as the insert
/// path; carries both old and new content hashes so the dashboard can
/// render a diff-friendly evolution history.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MemoryNoteEvolvedPayload {
    /// UUIDv4 hex of the evolved `memory_notes` row.
    pub note_id: String,
    /// BLAKE3 hex of the previous body (pre-evolution).
    pub old_content_blake3: String,
    /// BLAKE3 hex of the new body (post-evolution).
    pub new_content_blake3: String,
    /// New `note_id` references added to the row's `links` array.
    pub links_added: u32,
    /// `note_id` references removed from the row's `links` array.
    pub links_removed: u32,
}

/// E-1 dashboard slice — payload for [`SsbActionType::MemoryConflictDetected`].
///
/// Emitted by the conflict adjudicator when two `memory_facts` /
/// `memory_notes` rows describe overlapping or contradictory state.
/// `conflict_kind` is the snake_case discriminator — current vocabulary
/// is `"duplicate"` | `"contradiction"`; future kinds (e.g.
/// `"version_drift"`) can land without a schema bump because the field
/// is a free-form string.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MemoryConflictDetectedPayload {
    /// `fact_id` (or `note_id`) of the older row.
    pub fact_id_a: String,
    /// `fact_id` (or `note_id`) of the newer row.
    pub fact_id_b: String,
    /// `"duplicate"` | `"contradiction"` (snake_case discriminator).
    pub conflict_kind: String,
}

/// E-1 dashboard slice — payload for [`SsbActionType::CoreBlockReplaced`].
///
/// Emitted on every `CoreBlockStore::upsert` that REPLACES an existing
/// row (the first-insert path emits a separate beacon family — this
/// variant strictly records edits to an already-pinned core block, so
/// the dashboard's drift / edit-count indicator stays accurate).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CoreBlockReplacedPayload {
    /// Owner of the core block (single-user installs use `"default"`).
    pub user_id: String,
    /// BLAKE3 hex of the previous body.
    pub old_blake3: String,
    /// BLAKE3 hex of the new body.
    pub new_blake3: String,
    /// Byte size of the new body (always <= CORE_BLOCK_MAX_BYTES).
    pub bytes: u32,
}

/// D-3 — payload for [`SsbActionType::VerifierConsensusPass`].
///
/// Emitted by `crate::agent_pool::run_consensus_pass` after the aggregator
/// returns `AggregateVerdict::Pass`. `verdict` is the snake-case verdict
/// label (always `"pass"` here; carried as a String so dashboard rendering
/// is uniform with the failure-row schema). `rule_id` is the stable
/// snake_case `ConsensusRule::rule_id()` label so downstream consumers
/// don't need to re-decode the rule body.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct VerifierConsensusPassPayload {
    pub candidate_id: String,
    pub template_id: String,
    pub verdict: String,
    pub verifier_count: u32,
    pub pass_count: u32,
    pub rule_id: String,
}

/// D-3 — payload for [`SsbActionType::VerifierConsensusFail`].
///
/// Emitted by `crate::agent_pool::run_consensus_pass` when the aggregator
/// returns `AggregateVerdict::Fail` or `Inconclusive`. `failing_verifier_ids`
/// is non-empty for `Fail`; for `Inconclusive` it is empty and `reason`
/// carries the inconclusive-detail string (e.g. `"missing_kinds=ppm"`).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct VerifierConsensusFailPayload {
    pub candidate_id: String,
    pub template_id: String,
    pub failing_verifier_ids: Vec<String>,
    pub reason: String,
}

/// D-3 closure — payload for [`SsbActionType::PpmUnavailable`].
///
/// Emitted when the PPM sidecar's health check fails OR PpmClient::score
/// returns an error before producing a numeric score. Distinct from
/// [`PpmGatingFailedPayload`] which carries a real `score_mean` below
/// threshold; here no score exists. Carried atomically from D-2
/// (DECISION-1).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PpmUnavailablePayload {
    pub candidate_id: String,
    pub verifier_id: String,
    /// Snake_case reason tag: `"health_check_failed"` |
    /// `"score_call_failed"` | free-form short string from the sidecar
    /// adapter. Bounded length expected by dashboards (~ <80 chars).
    pub reason: String,
}

/// D-3 closure — payload for [`SsbActionType::VerifierDisagreement`].
///
/// Operational telemetry — emitted when one consensus run produced >= 1
/// Pass AND >= 1 Reject signals. D-4 tripwire stack monitors this row
/// family for early verifier-drift detection; the dashboard renders the
/// disagreement count alongside consensus pass / fail totals so operators
/// can see structural verifier-family conflict before it propagates into
/// repeated rejections.
///
/// `verifier_a_kind` / `verifier_b_kind` are the two snake_case
/// [`crate::verification_result::VerifierKind`] labels involved
/// (`verifier_a_kind` carries the Pass signal, `verifier_b_kind` the
/// Reject signal). `score_delta` is `(pass_score - fail_score)` widened
/// to f64 when both verifiers produced numeric scores; defaults to 0.0
/// when either side carried no score (Deterministic / CrossContext
/// always have 0.0 / 1.0 scores so the delta is meaningful even there).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct VerifierDisagreementPayload {
    pub candidate_id: String,
    pub template_id: String,
    pub verifier_a_kind: String,
    pub verifier_b_kind: String,
    pub score_delta: f64,
}

/// D-3 closure — payload for [`SsbActionType::CrossContextDispatched`].
///
/// Emitted by [`crate::cross_context::run_cross_context`] /
/// [`crate::cross_context::CrossContextVerifier::run`] on every invocation
/// — including Insufficient outcomes — so the dashboard timeline carries
/// the full cross-context history independent of the aggregate verdict.
///
/// `agreement` is the snake_case
/// [`crate::cross_context::CrossContextAgreement`] label
/// (`"agree"` | `"disagree"` | `"insufficient"`). `jaccard` is the
/// shingle-jaccard similarity on `[0.0, 1.0]` widened to f64 for the
/// SSB wire format.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CrossContextDispatchedPayload {
    pub claim_id: String,
    pub context_a_hash: String,
    pub context_b_hash: String,
    pub jaccard: f64,
    pub agreement: String,
}

/// C-38 — payload for [`SsbActionType::PagerankRun`].
///
/// Emitted by `elai_knowledge::pipeline::QueryPipeline::query_with_tier`
/// once per Stage 2 invocation. `workspace_root_hash` and `top_node_hash`
/// are 64-bit blake3-prefix hashes so the dashboard can group by
/// workspace + correlate top-ranked symbols without leaking absolute
/// paths or full symbol names. `node_count == 0` rows are valid
/// (empty-graph case) and intentionally emitted so the absence of a
/// row is a signal that PageRank never ran at all — not that it ran
/// with an empty graph.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PagerankRunPayload {
    pub workspace_root_hash: u64,
    pub node_count: u32,
    pub top_node_hash: u64,
    pub elapsed_ms: u64,
}

/// C-38 — payload for [`SsbActionType::DkgRebuildStarted`].
///
/// Emitted by `elai_cli::dkg_lifecycle::ensure_dkg_or_rebuild` when the
/// canonical DKG database (`.elai_cc/indexes/dkg.db`) is missing or stale.
/// `trigger_reason` is a snake-case label so dashboards can group rebuild causes:
/// `"missing"` | `"stale"`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DkgRebuildStartedPayload {
    pub workspace_root_hash: u64,
    pub trigger_reason: String,
}

/// C-38 — payload for [`SsbActionType::DkgRebuildComplete`].
///
/// Emitted at the end of every `ensure_dkg_or_rebuild` rebuild attempt.
/// `success=false` rows are still emitted so failure is observable.
/// `duration_ms` is wall-clock from start to end of the rebuild.
/// `node_count` is the number of DKG nodes produced (0 on failure).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DkgRebuildCompletePayload {
    pub workspace_root_hash: u64,
    pub success: bool,
    pub duration_ms: u64,
    pub node_count: u32,
}

/// C-40 — payload for [`SsbActionType::GithubPrOpened`].
///
/// Emitted by `crate::github_pr::GhCliAdapter::open` after every
/// successful `gh pr create` invocation. `pr_url_hash` is the
/// 64-bit prefix of blake3(pr_url) so dashboards can group rows by PR
/// identity without leaking the upstream URL. `ts_ms` is wall-clock
/// milliseconds since UNIX epoch (caller-supplied via [`ClockProvider`]).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GithubPrOpenedPayload {
    pub pr_number: u32,
    pub pr_url_hash: u64,
    pub ts_ms: i64,
}

/// C-40 — payload for [`SsbActionType::GithubPrStatusFetched`].
///
/// Emitted by `crate::github_pr::GhCliAdapter::status` after a
/// successful `gh pr view --json state,statusCheckRollup,reviewDecision`
/// invocation. CI and review state are carried as blake3-prefix hashes
/// of their canonical snake_case wire labels so the dashboard can render
/// status-transition rows (hash != prev_hash) without persisting the raw
/// label set, which gh evolves over time.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GithubPrStatusFetchedPayload {
    pub pr_number: u32,
    pub ci_status_hash: u64,
    pub review_state_hash: u64,
    pub ts_ms: i64,
}

/// C-40 — payload for [`SsbActionType::GithubPrClosed`].
///
/// Emitted by `crate::github_pr::GhCliAdapter::close` after a successful
/// `gh pr close <pr_number>` invocation. Distinct from [`GithubPrMergedPayload`]
/// so dashboards can branch on the discriminant rather than join raw
/// rows.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GithubPrClosedPayload {
    pub pr_number: u32,
    pub ts_ms: i64,
}

/// C-40 — payload for [`SsbActionType::GithubPrMerged`].
///
/// Emitted by `crate::github_pr::GhCliAdapter::merge` after a successful
/// `gh pr merge <pr_number>` invocation. `merge_sha_hash` is the 64-bit
/// prefix of blake3(merge_commit_sha) so the dashboard can correlate the
/// merged PR with the local `branch_merged` SSB row (C-23) without
/// leaking the upstream SHA.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GithubPrMergedPayload {
    pub pr_number: u32,
    pub merge_sha_hash: u64,
    pub ts_ms: i64,
}

/// E-41 — payload for [`SsbActionType::PrReviewComposed`].
///
/// Emitted once per successful `crate::github_pr::review::post_review_comment`
/// (and once per `crate::github_pr::git_assist::detect_force_push` to share
/// the wire shape — `comment_kind_hash` is `force_push_check` in that
/// case). PR number is carried as a blake3-prefix hash so dashboards
/// can group by identity without exposing the raw integer to
/// broadcast subscribers.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PrReviewComposedPayload {
    pub pr_number_hash: u64,
    pub comment_kind_hash: u64,
    pub ts_ms: i64,
}

/// E-41 — payload for [`SsbActionType::PrCommentWatched`].
///
/// Emitted once per `crate::github_pr::comment_watcher::watch_comments`
/// call. `new_comments_count` is the number of comments that landed
/// strictly after the caller-supplied watermark.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PrCommentWatchedPayload {
    pub pr_number_hash: u64,
    pub new_comments_count: u32,
    pub ts_ms: i64,
}

/// E-41 — payload for [`SsbActionType::PrPackageVerifyResult`].
///
/// Emitted once per
/// `crate::github_pr::package_verify::verify_pr_deps_no_slopsquat` call.
/// `slopsquat_count` is the number of dep candidates flagged by the
/// deterministic regex + known-squat list across all 3 ecosystems
/// (Cargo / pypi / npm).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PrPackageVerifyResultPayload {
    pub pr_number_hash: u64,
    pub slopsquat_count: u32,
    pub ts_ms: i64,
}

/// E-41 — payload for [`SsbActionType::PrSemgrepGateResult`].
///
/// Emitted once per `crate::github_pr::semgrep_gate::run_semgrep_gate`
/// call. `findings_count` aggregates across all severities;
/// `severity_max_hash` is the 64-bit prefix of blake3 over the
/// canonical snake_case severity label (`info` / `warning` / `error` /
/// `owasp_llm` / `other` / `toolchain_missing`).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PrSemgrepGateResultPayload {
    pub pr_number_hash: u64,
    pub findings_count: u32,
    pub severity_max_hash: u64,
    pub ts_ms: i64,
}

/// E-41 — payload for [`SsbActionType::PrAutoMergeBlocked`].
///
/// Emitted only on `MergeVerdict::Block` returns from
/// `crate::github_pr::auto_merge_guard::assess_merge_safety`.
/// `block_reason_hash` is the 64-bit prefix of blake3 over the
/// canonical reason label (`loc_over_cap` / `multi_file` / `owned_paths`).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PrAutoMergeBlockedPayload {
    pub pr_number_hash: u64,
    pub block_reason_hash: u64,
    pub ts_ms: i64,
}

/// D-5 closure (SCHEMA_VERSION 110) — payload for
/// [`SsbActionType::ThompsonSampled`].
///
/// Emitted once per Thompson draw inside [`crate::bai::emit_bai_round`]
/// immediately BEFORE the matching `BaiArmUpdated` row. Captures the
/// Beta(alpha, beta) sample value used to decide which arm the policy
/// pulled this round. Consumers (dashboard `BaiArmsPanel` + future
/// regret-tracking surfaces) join the sample, the posterior update, and
/// any follow-on promote/reject decision in one ordered triple per round.
///
/// `arm_id` is the 32-char simple-hex
/// [`crate::reasoning_template::TemplateId`] of the candidate arm so
/// joins across all D-1 / D-5 rows use the same identifier shape.
/// `alpha` / `beta` carry the posterior state used to draw the sample —
/// useful for offline replay (resampling the same Beta gives the same
/// value modulo RNG seed, which the producer documents in its module
/// header). `sampled_value` is the realised Beta draw in [0.0, 1.0].
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ThompsonSampledPayload {
    /// 32-char simple-hex of the candidate arm's
    /// [`crate::reasoning_template::TemplateId`].
    pub arm_id: String,
    /// Beta draw from `Beta(alpha, beta)` used by the Thompson policy.
    /// Always in `[0.0, 1.0]`.
    pub sampled_value: f64,
    /// Beta-distribution alpha at sample time.
    pub alpha: f64,
    /// Beta-distribution beta at sample time.
    pub beta: f64,
}

/// C-41 (SCHEMA_VERSION 111) — payload for [`SsbActionType::SkillTriggered`].
///
/// Emitted by the orchestrator side that consumes
/// [`crate::skill_registry::top_k_skills`] on every successful registry
/// lookup. `skill_name_hash` is the 64-bit prefix of
/// `blake3(skill_name.as_bytes())` (via
/// [`crate::skill_registry::skill_name_hash`]) so dashboards can group by
/// skill identity without surfacing the raw name. `score` is the
/// deterministic score in `[0.0, 1.0]` returned by
/// [`crate::skill_registry::score_skill_for_context`]. `top_k_rank` is the
/// 1-based position within the candidate window — typically 1 (the top
/// match) since the registry returns sorted candidates, but emitters that
/// log multiple SkillTriggered rows for the same lookup walk the ranks
/// monotonically.
///
/// Locally authored (not paper-grounded): arXiv:2604.21910 does not
/// publish an event/SSB vocabulary — the paper's design is a structured
/// ResearchIntent passed between layers, not an event bus. The SkillTriggered
/// / SkillExecuted pair is a ELAI-local observability surface inspired
/// by the C-40 GithubPr* hashing pattern.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SkillTriggeredPayload {
    /// blake3-prefix u64 of the resolved skill name. Stable across runs
    /// for a given name; rotating a skill name produces a new hash.
    pub skill_name_hash: u64,
    /// Score the registry assigned to this candidate; clamped to
    /// `[0.0, 1.0]`. Values below
    /// [`crate::skill_registry::SKILL_CONFIDENCE_THRESHOLD`] indicate the
    /// consumer should fall through to a default scaffold rather than
    /// auto-select this skill.
    pub score: f32,
    /// 1-based rank within the top-k window. The top match has rank 1.
    pub top_k_rank: u8,
    /// Producer-side wall-clock timestamp in milliseconds since UNIX
    /// epoch. Sourced from the orchestrator's `ClockProvider`.
    pub ts_ms: u64,
}

/// C-41 (SCHEMA_VERSION 111) — payload for [`SsbActionType::SkillExecuted`].
///
/// Emitted when the orchestrator dispatches the resolved skill — typically
/// via the harness Skill tool's `crate::tools::execute_skill`. Pairs with
/// [`SkillTriggeredPayload`] like the existing tool_dispatch / tool_result
/// pattern so dashboards can join the routing decision with the dispatch
/// outcome.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SkillExecutedPayload {
    /// Same blake3-prefix hash shape as [`SkillTriggeredPayload::skill_name_hash`].
    pub skill_name_hash: u64,
    /// Whether the dispatch returned Ok. False covers any error path
    /// (skill not found, prompt file unreadable, downstream agent error,
    /// etc); the producer surface logs the structured cause separately
    /// via `Finding`.
    pub exit_ok: bool,
    /// Wall-clock time in milliseconds from dispatch start to skill
    /// return. Saturating subtraction; the producer documents
    /// monotonic-clock sourcing.
    pub duration_ms: u64,
    /// Producer-side wall-clock timestamp in milliseconds since UNIX
    /// epoch. Sourced from the orchestrator's `ClockProvider`.
    pub ts_ms: u64,
}

/// D-6 closure — payload for [`SsbActionType::HeldOutPromotionGate`].
///
/// Emitted by the BAI promotion path after
/// `held_out::gate_promotion_via_held_out` returns the regression-gate
/// decision against the held-out eval subset. The dashboard joins these
/// rows to the matching `bai_arm_updated` row (when the gate allowed
/// promotion) to render the "why-this-template-was-promoted" timeline.
/// See workflow/reasoning/D-6-decisions.md DECISION-7 for the BAI/held-out
/// layering rationale.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct HeldOutPromotionGatePayload {
    /// Simple-hex uuid (32 chars) of the candidate template under
    /// evaluation. Mirrors `BaiArmUpdatedPayload::template_id`.
    pub task_id: String,
    /// True when the regression delta was within the configured
    /// threshold (gate ALLOWED the promotion).
    pub allow: bool,
    /// Measured pass-rate delta in [-1.0, 1.0]: candidate minus baseline.
    /// Negative = regression; positive = improvement.
    pub delta: f64,
    /// Threshold (absolute regression budget in pp) the gate evaluated
    /// against. Loaded from the `[held_out]` section of elai_cc.toml.
    pub regression_threshold: f64,
    /// How many held-out tasks the candidate passed on this gate run.
    pub passes: u32,
    /// How many held-out tasks the candidate failed on this gate run.
    pub fails: u32,
    /// Human-readable reason string from
    /// [`crate::held_out::PromotionGateResult::reason`]. Empty allowed.
    pub reason: String,
}

/// D-6 closure — payload for [`SsbActionType::HeldOutRefresh`].
///
/// Emitted once per auto-refresh tick that mutates the store (adds and/or
/// retires tasks). The dashboard renders these as a refresh-event timeline
/// so the operator can see at-a-glance when the held-out pool last
/// rotated. See workflow/reasoning/D-6-decisions.md DECISION-5 for the
/// 6-month retirement policy rationale (ELAI-novel, not
/// LiveCodeBench-specified).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct HeldOutRefreshPayload {
    /// Snake-case label from
    /// [`crate::held_out::TaskSource::label`] for the rows added on this
    /// refresh tick. Empty when the tick only retired (no additions).
    pub source_label: String,
    /// Number of fresh tasks ingested on this refresh tick.
    pub added_count: u32,
    /// Number of stale tasks retired on this refresh tick.
    pub retired_count: u32,
    /// Wall-clock UNIX ms when the refresh tick completed.
    pub ts_ms: i64,
}

/// C-42 (SCHEMA_VERSION 113) — payload for [`SsbActionType::GatewayPolicyAllowed`].
///
/// Emitted on every composed-Allow verdict from
/// `plugins::mcp_bridge::DefaultMcpBridge::dispatch_tool_call_with_authz`.
/// Both `role_hash` and `tool_hash` are blake3-prefix u64 hashes so
/// dashboard subscribers can group by identity without surfacing the raw
/// role/tool strings (SSB rows are visible to any subscriber on the
/// process broadcast).
///
/// `deny_reason_label` is always `"allow"` on this payload (kept for wire
/// symmetry with [`GatewayPolicyDeniedPayload`] so downstream consumers
/// can read `deny_reason_label` uniformly across allow/deny rows).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GatewayPolicyAllowedPayload {
    pub role_hash: u64,
    pub tool_hash: u64,
    /// Always `"allow"` for this variant; wire-symmetric with the deny
    /// payload. SSB consumers can branch on `deny_reason_label` without
    /// knowing which action_type the row carries.
    pub deny_reason_label: String,
    /// Producer-side wall-clock timestamp in milliseconds since UNIX
    /// epoch. Sourced from the orchestrator's `ClockProvider`.
    pub ts_ms: i64,
}

/// C-42 (SCHEMA_VERSION 113) — payload for [`SsbActionType::GatewayPolicyDenied`].
///
/// Emitted on every composed-Deny verdict. `deny_reason_label` is one of
/// `"authz"` (L2 RBAC denied) or `"tool_gate"` (C-35 FSM-state allowlist
/// denied). `deny_reason_hash` is a blake3-prefix u64 of the structured
/// reason string so the dashboard can group "same failure mode" rows
/// without surfacing the raw reason text.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GatewayPolicyDeniedPayload {
    pub role_hash: u64,
    pub tool_hash: u64,
    /// One of `"authz"` / `"tool_gate"`. Mirrors paper Section 5's
    /// "rejecting_layer" field.
    pub deny_reason_label: String,
    /// Blake3-prefix u64 hash of the structured reason string. Stable
    /// per failure mode (same reason text -> same hash).
    pub deny_reason_hash: u64,
    pub ts_ms: i64,
}

/// C-42 (SCHEMA_VERSION 113) — payload for [`SsbActionType::GatewayHitlPending`].
///
/// L3 Crypto-HITL beacon (paper Section 5: "speculative suspension;
/// evidence package = SHA-256 hash of (action, params, CoT justification)").
/// L3 is DEFERRED-TO-FOLLOWUP for session 2 — no producer is wired in
/// this commit. The variant is declared so the followup plan can land
/// emitters without a second SCHEMA bump (atomic deferral).
///
/// `challenge_hash` carries the SHA-256-derived 64-bit prefix of the
/// evidence package the L3 challenge protocol would send to the operator
/// device. In session 2 this field is unused.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GatewayHitlPendingPayload {
    pub role_hash: u64,
    pub tool_hash: u64,
    /// SHA-256-prefix u64 of the (action, params, CoT) evidence package.
    /// Always `0` in session 2 (L3 not wired); becomes load-bearing once
    /// L3 emitters land in the followup plan.
    pub challenge_hash: u64,
    pub ts_ms: i64,
}

/// D-7 closure (SCHEMA_VERSION 114) — payload for
/// [`SsbActionType::CrossDomainRegression`].
///
/// Emitted by `canary::emit_canary_run` when at least one non-target
/// family regressed beyond the fail-band relative to the candidate's
/// target family. ELAI design hypothesis — NanoFlux (arXiv:2509.23252)
/// provides empirical motivation that multi-domain improvement is
/// achievable but defines NO canary mechanism, NO 5% degradation
/// threshold, and NO cross-domain regression safeguard. The 5%
/// `threshold_pct` default + the fail-band/marginal-band tunables are
/// ELAI policy choices (see workflow/reasoning/D-7-decisions.md
/// DECISION-2 / DECISION-3).
///
/// `delta_pct` is the absolute magnitude of the regression in percent
/// (always positive — `0.07` means a 7 percentage-point pass-rate drop).
/// `threshold_pct` is the operating fail-band in percent so dashboards
/// can render both without re-reading the toml stanza.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CrossDomainRegressionPayload {
    /// Domain of the candidate's target family (where the win was
    /// observed).
    pub source_domain: crate::reasoning_template::Domain,
    /// Domain of the family that regressed.
    pub regressed_domain: crate::reasoning_template::Domain,
    /// Magnitude of the regression in percent (always positive). For a
    /// candidate pass-rate of 0.62 vs baseline 0.70 on the regressed
    /// family this is `8.0`.
    pub delta_pct: f64,
    /// Active fail-band threshold in percent (5.0 by default — the
    /// ELAI policy choice; not NanoFlux-specified).
    pub threshold_pct: f64,
    /// Producer-side wall-clock timestamp in milliseconds since UNIX
    /// epoch. Sourced from the caller-supplied `ts_ms` so deterministic
    /// tests can pin the value.
    pub ts_ms: i64,
}

/// D-7 closure (SCHEMA_VERSION 114) — payload for
/// [`SsbActionType::CanaryFamilyPass`].
///
/// Emitted once per family that cleared the marginal band on a canary
/// run AND met the per-family sample floor. ELAI design hypothesis —
/// see [`CrossDomainRegressionPayload`] grounding note. `score_pct`
/// is the candidate's pass-rate on the family in percent (range
/// `[0.0, 100.0]`).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CanaryFamilyPassPayload {
    /// Snake-case family label (`code_fix` / `math_proof` /
    /// `prose_reason` / `planning_decomposition` / `tool_use`).
    pub family: String,
    /// Candidate pass-rate on the family in percent. Range
    /// `[0.0, 100.0]`.
    pub score_pct: f64,
    /// How many held-out tasks were drawn from this family.
    pub sample_count: u32,
    /// Producer-side wall-clock timestamp in milliseconds since UNIX
    /// epoch.
    pub ts_ms: i64,
}

/// D-7 closure (SCHEMA_VERSION 114) — payload for
/// [`SsbActionType::CanaryInsufficientData`].
///
/// Emitted when a family's sample count fell below the `min_per_family`
/// floor so the family was reported but not counted toward the overall
/// decision. ELAI design hypothesis — see
/// [`CrossDomainRegressionPayload`] grounding note. Invariant:
/// `sample_count < min_required`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CanaryInsufficientDataPayload {
    /// Snake-case family label.
    pub family: String,
    /// Observed sample count for this family (strictly less than
    /// `min_required`).
    pub sample_count: u32,
    /// Configured `min_per_family` floor. ELAI policy choice
    /// (`min_per_family` toml default = 5 — not NanoFlux-specified).
    pub min_required: u32,
    /// Producer-side wall-clock timestamp in milliseconds since UNIX
    /// epoch.
    pub ts_ms: i64,
}

/// D-8 closure (SCHEMA_VERSION 115) — payload for
/// [`SsbActionType::ArchiveVerifyResult`].
///
/// Emitted once per `elai archive verify` invocation by the CLI driver
/// (crates/cli/src/archive_verify.rs::run), regardless of pass / fail.
///
/// `verified` is the boolean roll-up (`errors.is_empty()`). `errors` is
/// the TYPED [`crate::archive_chain::ArchiveVerifyError`] list — the
/// plan called for `Vec<String>` but typed errors are required so the
/// dashboard MutationArchivePanel can branch on the failure mode (the
/// HashMismatch / BrokenParent / DuplicateHash / MalformedJson tag) and
/// render the entry_index inline without re-parsing freeform strings.
/// See workflow/reasoning/D-8-decisions.md DECISION-3.
///
/// `audit_id` is the chain header identity — currently the absolute on-
/// disk path of the JSONL file (D-1 ArchiveChain has no separate header
/// row yet). When D-1 grows an explicit header type, this field will
/// migrate to that header's content hash.
///
/// arxiv:2507.05619 (T1 cryptographic-checksum recommendation) + DGM §5
/// (archive practice) are supporting documentation for the integrity-
/// rationale; neither paper specifies this wire layout.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ArchiveVerifyResultPayload {
    /// Identity for the audited chain. Currently the absolute path of
    /// the JSONL file (D-1 chain has no separate header type yet).
    pub audit_id: String,
    /// Total entries observed in the chain (successful parses only —
    /// malformed JSONL lines are reported under `errors` but do NOT
    /// inflate this count).
    pub total_entries: u32,
    /// `true` iff `errors.is_empty()`. Read by dashboards as the
    /// pass / fail roll-up without iterating the error list.
    pub verified: bool,
    /// Length of `errors`, cached so consumers can size-budget without
    /// walking the vector.
    pub error_count: u32,
    /// Typed per-entry verify failures. See
    /// [`crate::archive_chain::ArchiveVerifyError`] for the variant
    /// catalog.
    pub errors: Vec<crate::archive_chain::ArchiveVerifyError>,
    /// Producer-side wall-clock timestamp in milliseconds since UNIX
    /// epoch.
    pub ts_ms: i64,
}

/// C-43 closure (SCHEMA_VERSION 116) — payload for
/// [`SsbActionType::ScopeFreezeRatified`].
///
/// Emitted once per `elai freeze ratify` invocation by the CLI driver
/// (crates/cli/src/freeze_cmd.rs::run_ratify). Fields are deliberately
/// hash-only — the plaintext ratifier identity is resolved through a
/// separate registry keyed by `ratifier_id_hash` so the SSB log never
/// carries human-identifying strings.
///
/// `manifest_hash` is the blake3-truncated-to-64-bit digest of the
/// freshly-written scope-freeze.toml content. The dashboard
/// FreezeStatusBar uses this to detect drift between successive
/// ratifications without re-reading the file.
///
/// ELAI design hypothesis — no paper grounding. The source plan cited
/// arXiv:2604.21725 (AEL) but two independent WebFetches confirmed
/// that paper is Thompson-Sampling agent learning, NOT scope-freeze.
/// See workflow/sessions/s-25120/mistakes.md for the bad-cite analysis.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ScopeFreezeRatifiedPayload {
    /// blake3 of scope-freeze.toml content, truncated to 64 bits LE.
    pub manifest_hash: u64,
    /// blake3 of the ratifier identity string, truncated to 64 bits LE.
    pub ratifier_id_hash: u64,
    /// Number of frozen items in the manifest at ratification time.
    pub item_count: u32,
    /// Producer-side wall-clock timestamp in milliseconds since UNIX
    /// epoch.
    pub ts_ms: i64,
}

/// C-43 closure (SCHEMA_VERSION 116) — payload for
/// [`SsbActionType::ScopeFreezeViolated`].
///
/// Emitted once per offending item by the `elai freeze verify` CLI
/// driver (crates/cli/src/freeze_cmd.rs::run_verify) or the
/// dashboard's `scope_freeze_status` Tauri command. The path string
/// itself is NOT serialised on the wire — only its 64-bit blake3
/// hash. The CLI prints the plaintext path locally; dashboards
/// resolve the hash through a manifest re-read.
///
/// ELAI design hypothesis — no paper grounding.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ScopeFreezeViolatedPayload {
    /// blake3 of the workspace-relative path, truncated to 64 bits LE.
    pub item_path_hash: u64,
    /// `ratification_ts_ms` from the manifest at the time of verify.
    pub frozen_since_ms: u64,
    /// On-disk mtime of the offending item in ms since UNIX_EPOCH, or
    /// 0 when the item is missing entirely (sentinel for
    /// missing-frozen-item violations — see
    /// `crate::scope_freeze::MISSING_ITEM_SENTINEL_MS`).
    pub modified_at_ms: u64,
    /// Producer-side wall-clock timestamp in milliseconds since UNIX
    /// epoch.
    pub ts_ms: i64,
}

/// C-44 (SCHEMA_VERSION 117) — payload for
/// [`SsbActionType::BackendCostAccrued`].
///
/// Emitted by `RuntimeOrchestrator::try_complete` after each successful
/// paid-backend (initially Composer 2; future paid clouds reuse the
/// variant) invocation. The wire shape avoids plaintext leaks: backend
/// + role names are reduced to 64-bit hashes via the helpers in
/// `elai_providers::runtime::cost` (`backend_name_hash` /
/// `role_name_hash`). Dashboards resolve hashes through the canonical
/// `RuntimeBackend::all()` / `AgentRole` iteration order.
///
/// Cost is reported in micro-dollars (1 micro = 1e-6 USD) to keep
/// integer arithmetic exact across millions of small invocations. See
/// `elai_providers::runtime::cost::BackendCostModel::estimate_micros`
/// for the ceiling-divide rounding rule.
///
/// ELAI design hypothesis — Composer 2 paper (arXiv:2603.24477)
/// grounds model identity only, NOT cost-accrual policy. The wire
/// shape is ELAI-internal. See workflow/papers/composer2-interface.md
/// for the shallow-cite-fit analysis.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BackendCostAccruedPayload {
    /// FxHash-derived 64-bit hash of `RuntimeBackend::canonical_name`.
    /// Stable across renames because the canonical_name is the
    /// SQL-wire id, not the Rust enum variant name.
    pub backend_name_hash: u64,
    /// FxHash-derived 64-bit hash of the snake_case `AgentRole` id.
    pub role_name_hash: u64,
    /// Input tokens consumed (provider-reported via response.usage).
    pub tokens_in: u32,
    /// Output tokens generated.
    pub tokens_out: u32,
    /// Estimated cost in micros (1e-6 USD).
    pub est_cost_micros: u64,
    /// Producer-side wall-clock timestamp in milliseconds since UNIX
    /// epoch.
    pub ts_ms: u64,
}

/// C-45 (SCHEMA_VERSION 120) — payload for
/// [`SsbActionType::AgentRoleDropped`].
///
/// Emitted by the credit-attribution harness BEFORE the production
/// agent-spawn path reroutes around a dropped role (emit-before-reroute
/// audit-trail invariant). `role_name_hash` is the FxHash of the
/// snake_case `AgentRole` name (same hash function as
/// `BackendCostAccruedPayload.role_name_hash` — dashboards resolve
/// hashes via canonical `AgentRole` iteration).
///
/// Paper grounding: arXiv:2605.13295 (Zehle 2026, CANTANTE Section 3.2).
/// See `workflow/papers/cantante-credit-attribution.md`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AgentRoleDroppedPayload {
    /// FxHash-derived 64-bit hash of the snake_case `AgentRole` id
    /// (e.g. "executor", "auditor"). Resolved via
    /// `elai_providers::runtime::cost::role_name_hash` precedent.
    pub role_name_hash: u64,
    /// Producer-side wall-clock timestamp in milliseconds since UNIX
    /// epoch (resolved via the orchestrator's injected `ClockProvider`).
    pub ts_ms: u64,
}

/// C-45 (SCHEMA_VERSION 120) — payload for
/// [`SsbActionType::CreditAttributionRun`].
///
/// Emitted once per (configuration, harness invocation) pair at the
/// start of a contrastive credit-attribution rollout sweep. Pairs
/// up with [`CreditAttributionResultPayload`] (same `run_id_hash`).
///
/// `dropped_role_name_hash == 0` is the baseline-leg sentinel (the
/// harness emits one Run beacon for the baseline + one per role in the
/// ablation set).
///
/// Paper grounding: arXiv:2605.13295 (Zehle 2026, Section 3.2 — K
/// rollouts NOT K^N; partition divisibility K % g == 0 per Section
/// 4.4 footnote 1).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CreditAttributionRunPayload {
    /// Caller-supplied stable id for this pilot run (typically blake3-
    /// prefix of a session-scoped run name). Allows a Result beacon
    /// to be paired with its Run beacons across multiple agents.
    pub run_id_hash: u64,
    /// FxHash of the dropped role's snake_case name, or zero for the
    /// baseline leg.
    pub dropped_role_name_hash: u64,
    /// K — rollouts per configuration (paper Section 4 default 9).
    pub k_rollouts: u32,
    /// g — group partition size (paper-anchored default 3; the harness
    /// refuses g < 3 per CANTANTE Section 3.2 / Appendix B.2).
    pub g_partition: u32,
    /// Producer-side wall-clock timestamp in milliseconds since UNIX
    /// epoch.
    pub ts_ms: u64,
}

/// C-45 (SCHEMA_VERSION 120) — payload for
/// [`SsbActionType::CreditAttributionResult`].
///
/// Emitted once per harness invocation at the aggregation step.
/// `per_role_scores_hash` is a blake3-prefix u64 over the sorted
/// per-role scoreboard so dashboards can detect collisions across runs
/// without having to expose the raw f32 credit values (the raw values
/// flow through the harness's stdout / pilot artifact, not via SSB —
/// keeps the SSB surface small + privacy-safe).
///
/// Paper grounding: arXiv:2605.13295 (Zehle 2026, Section 3.2 — scalar
/// c in [-1, +1]).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CreditAttributionResultPayload {
    /// Matches the `run_id_hash` of the paired
    /// [`CreditAttributionRunPayload`] beacons. Allows dashboards to
    /// group all rows for one pilot run.
    pub run_id_hash: u64,
    /// blake3-prefix u64 over the sorted per-role credit scoreboard.
    /// Two identical reports collide; any single-bit change in any
    /// score (within float precision) flips this value with very high
    /// probability (blake3 is collision-resistant — see
    /// `crate::skill_registry::skill_name_hash` for the precedent
    /// pattern of blake3-prefix-u64 SSB identifiers).
    pub per_role_scores_hash: u64,
    /// Number of roles in the ablation set (8 for the C-45 pilot —
    /// see `crate::credit_attribution::partition::pilot_roles`).
    pub total_roles: u32,
    /// Producer-side wall-clock timestamp in milliseconds since UNIX
    /// epoch.
    pub ts_ms: u64,
}

/// C-46 (SCHEMA_VERSION 121) — payload for
/// [`SsbActionType::SelectiveDelegationDecision`].
///
/// Emitted once per `routing::SelectiveDelegator::select_workers`
/// dispatch at the spawn boundary. The Decision beacon is the single
/// row downstream dashboards consume to render "what the policy chose
/// for this task" — `dispatched_count == 0` means the policy collapsed
/// to a direct-answer (lazy-mode) skip and no workers were spawned
/// (paper Section 5 lazy-mode direct-answer-cost-zero invariant).
///
/// Paper grounding: arXiv:2605.05007 (Cui et al 2026, Uno-Orchestra
/// Section 5 — `(model, primitive)` action space + alpha in [0,1]
/// convex blend; Section 6.1 selectivity claim).
///
/// Note: this payload omits `Eq` (and the dependent `Hash`) because
/// `difficulty` and `alpha` are `f32` (NaN-not-equal). All other Eq-
/// requiring code paths use `action_type()` instead of structural
/// equality on the payload.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SelectiveDelegationDecisionPayload {
    /// blake3-prefix u64 of the caller-supplied task id. Allows the
    /// Decision / Escalated / CostSaved triple to be joined without
    /// exposing plaintext task strings to the dashboard.
    pub task_id_hash: u64,
    /// Difficulty score in `[0, 1]` returned by
    /// `routing::TaskDifficultyClassifier`. NaN / out-of-band values
    /// are clamped at construction (see `routing::SelectiveDelegator`).
    pub difficulty: f32,
    /// alpha blend in `[0, 1]` (paper default 0.1 — keeps verifier
    /// reward dominant; raising amplifies cost pressure).
    pub alpha: f32,
    /// Number of `(model, primitive)` pairs the policy dispatched.
    /// `0` means lazy-mode direct-answer (no workers spawned).
    pub dispatched_count: u32,
    /// Producer-side wall-clock timestamp in milliseconds since UNIX
    /// epoch (resolved via the orchestrator's injected `ClockProvider`).
    pub ts_ms: u64,
}

/// C-46 (SCHEMA_VERSION 121) — payload for
/// [`SsbActionType::SelectiveDelegationEscalated`].
///
/// Emitted when an audit-bubble or verifier failure forces a broader-
/// role re-spawn on top of an earlier selective-delegation dispatch
/// for the same task. `escalated_workers` is the count of additional
/// workers spawned beyond the original Decision dispatch — strictly
/// positive (a no-escalation path does NOT emit this row; the
/// stub_ban contract requires emission ONLY when the escalation path
/// fired).
///
/// Paper grounding: arXiv:2605.05007 (Cui et al 2026, Section 5 — RL
/// stage with verifier-gated curriculum; failure-recovery via broader-
/// pool re-routing is the obvious deployment-time analog).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SelectiveDelegationEscalatedPayload {
    /// blake3-prefix u64 of the caller-supplied task id (joins to the
    /// originating Decision beacon).
    pub task_id_hash: u64,
    /// FxHash of the snake_case escalation reason string (e.g.
    /// `"audit_bubble"`, `"verifier_failure"`,
    /// `"max_turns_exhausted"`). Hash collapses the wire surface and
    /// avoids leaking auditor-internal failure messages to the
    /// dashboard.
    pub reason_hash: u64,
    /// Count of workers added on top of the original dispatch. Always
    /// `>= 1` when this row is emitted; zero would be a stub_ban
    /// violation (no escalation but row emitted).
    pub escalated_workers: u32,
    /// Producer-side wall-clock timestamp in milliseconds since UNIX
    /// epoch.
    pub ts_ms: u64,
}

/// C-46 (SCHEMA_VERSION 121) — payload for
/// [`SsbActionType::SelectiveDelegationCostSaved`].
///
/// Emitted alongside the Decision beacon when the policy omitted at
/// least one worker from the baseline chain. `est_savings_micros` is
/// the dispatcher-side estimate of the saved cost in 1e-6 USD units
/// (skipped_count * per_worker_tariff). The metric is NON-NEGATIVE by
/// construction (both factors are non-negative); zero corresponds to
/// "no workers skipped" (the producer is expected to NOT emit this row
/// when skipped_count == 0 — stub_ban contract aligns with the
/// agentless_shortcut precedent).
///
/// Paper grounding: arXiv:2605.05007 (Cui et al 2026, Section 6.1 —
/// ~10x cost reduction vs strongest workflow baseline). The micros
/// unit matches the C-44 `BackendCostAccruedPayload.est_cost_micros`
/// precedent so dashboards can sum across both rows without unit
/// translation.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SelectiveDelegationCostSavedPayload {
    /// blake3-prefix u64 of the caller-supplied task id (joins to the
    /// originating Decision beacon).
    pub task_id_hash: u64,
    /// Number of workers the policy dropped from the baseline chain.
    /// Always `>= 1` when this row is emitted; zero would be a
    /// stub_ban violation.
    pub skipped_count: u32,
    /// Estimated cost savings in 1e-6 USD (micros). Non-negative by
    /// construction.
    pub est_savings_micros: u64,
    /// Producer-side wall-clock timestamp in milliseconds since UNIX
    /// epoch.
    pub ts_ms: u64,
}

/// C-47 (SCHEMA_VERSION 122) — payload for
/// [`SsbActionType::ToolCallLogged`].
///
/// Emitted by the plugins-side
/// [`elai_plugins::mcp_bridge::DefaultMcpBridge::record_call`] helper
/// AFTER every completed tool dispatch. The `post_call_audit_ok`
/// field is derived synchronously from
/// [`elai_plugins::McpCallResult::is_error`] per the C-47 plan rebase
/// — audit-loop integration is a follow-on extension.
///
/// All identifying strings are hashed (blake3-prefix u64) so the wire
/// surface never carries raw tool names / call shapes.
///
/// Paper grounding: arXiv:2605.15041 (CAST) — the per-trajectory
/// telemetry tuple `(q, r, c, o, phi)`. The CALL beacon stands in for
/// the trajectory record at the SSB layer; aggregation lives in the
/// plugins-side [`elai_plugins::ToolCalibrationRegistry`].
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ToolCallLoggedPayload {
    /// blake3-prefix u64 of the snake-case tool name.
    pub tool_name_hash: u64,
    /// blake3-prefix u64 of the canonical-sorted call-shape JSON key
    /// list (values excluded so shapes collide as intended).
    pub call_shape_hash: u64,
    /// blake3-prefix u64 of the snake-case response_kind string
    /// (`"ok"` / `"err_invalid_args"` / etc).
    pub response_kind_hash: u64,
    /// Synchronous heuristic: true when `McpCallResult.is_error`
    /// was false. The CAST paper's audit signal would be richer; the
    /// upgrade path is a follow-on extension that consults the audit
    /// loop directly.
    pub post_call_audit_ok: bool,
    /// Producer-side wall-clock timestamp in milliseconds since UNIX
    /// epoch.
    pub ts_ms: i64,
}

/// C-47 (SCHEMA_VERSION 122) — payload for
/// [`SsbActionType::ToolCalibrationHintEmitted`].
///
/// Emitted by the plugins-side
/// [`elai_plugins::mcp_bridge::DefaultMcpBridge::consult_calibration`]
/// helper whenever the case base crosses the hinter's
/// `HINT_ERROR_RATE_THRESHOLD` + `HINT_MIN_TOTAL_CALLS` gating. The
/// `hint_text_hash` is a blake3-prefix u64 over the rendered hint
/// string so dashboards can group identical hints without surfacing
/// the raw model-facing text on the wire (the actual text remains in
/// the prompt-assembly side-channel).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ToolCalibrationHintEmittedPayload {
    pub tool_name_hash: u64,
    pub call_shape_hash: u64,
    pub hint_text_hash: u64,
    pub ts_ms: i64,
}

/// C-47 (SCHEMA_VERSION 122) — payload for
/// [`SsbActionType::ToolCalibrationDriftDetected`].
///
/// Emitted when the per-tool aggregate error_rate (sum of per-shape
/// `error_count` over sum of per-shape `total_calls`) crosses the
/// drift threshold inside
/// [`elai_plugins::ToolCalibrationRegistry::drift_detected`]. Note:
/// `error_rate` is an `f32`; the payload does NOT derive `Eq` (NaN-
/// not-equal). Equality-requiring code paths use `action_type()`
/// instead of structural equality.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ToolCalibrationDriftDetectedPayload {
    pub tool_name_hash: u64,
    pub error_rate: f32,
    pub total_calls: u32,
    pub ts_ms: i64,
}

/// C-49 (SCHEMA_VERSION 123) — payload for
/// [`SsbActionType::PromptPolicyDistillationRun`].
///
/// Emitted once per `crate::experience::distill_fragments` invocation
/// (typically from the wire-dark post-audit hook in `task_runner`).
/// Identifying strings are hashed (blake3-prefix u64) so the wire
/// surface never carries raw skill names.
///
/// Paper grounding caveat: ELAI heuristic adaptation of
/// arXiv:2605.14443 (Sayana, Todi, Jash 2026). The full RL-trained
/// prompter (Algorithm 1, GRPO + KL anchor) is RUNPOD-PENDING.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PromptPolicyDistillationRunPayload {
    /// blake3-prefix u64 of the canonical snake_case skill name.
    pub skill_name_hash: u64,
    /// Number of `SuccessTrace`s consulted by the distillation run.
    pub trace_count: u32,
    /// Number of `PromptFragment`s included in the distilled output
    /// (post `min_freq` + `min_success_rate` filtering + ranking).
    pub fragments_emitted: u32,
    /// Producer-side wall-clock timestamp in milliseconds since UNIX
    /// epoch.
    pub ts_ms: u64,
}

/// C-49 (SCHEMA_VERSION 123) — payload for
/// [`SsbActionType::PromptFragmentSelected`].
///
/// Emitted once per `PromptFragment` included in the distilled output
/// of a [`PromptPolicyDistillationRunPayload`]. The `fragment_hash` is
/// a blake3-prefix u64 so the raw fragment text never crosses the
/// dashboard boundary.
///
/// `success_rate` is `f32`; the payload does NOT derive `Eq` (NaN-
/// not-equal). Equality-requiring code paths use `action_type()`
/// instead of structural equality (matches the C-47
/// `ToolCalibrationDriftDetectedPayload` precedent).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PromptFragmentSelectedPayload {
    pub skill_name_hash: u64,
    pub fragment_hash: u64,
    /// Observed empirical success rate in `[0.0, 1.0]` (clamped by
    /// the distiller before emission).
    pub success_rate: f32,
    /// Observed occurrence count across the per-skill trace bin.
    pub freq: u32,
    /// Producer-side wall-clock timestamp in milliseconds since UNIX
    /// epoch.
    pub ts_ms: u64,
}

/// C-49 (SCHEMA_VERSION 123) — payload for
/// [`SsbActionType::PromptFragmentDiscarded`].
///
/// Emitted once per `PromptFragment` rejected by the distillation
/// thresholds. The `reason_hash` is a blake3-prefix u64 over the
/// snake_case rejection reason (e.g. `"below_min_freq"`,
/// `"below_min_success_rate"`, `"audit_pass_false"`).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PromptFragmentDiscardedPayload {
    pub skill_name_hash: u64,
    pub fragment_hash: u64,
    pub reason_hash: u64,
    pub ts_ms: u64,
}

/// D-14 (SCHEMA_VERSION 124) — payload for
/// [`SsbActionType::SycophancyProbeDispatched`].
///
/// Emitted once per `dispatch_sycophancy_probe_hook` invocation on the
/// task_runner post-task hook path. The row is the durable producer
/// signal the auditor reads to know a probe ran; the verdict lands on
/// the paired [`SycophancyFlagPayload`].
///
/// Field semantics:
///   * `claim_id` — D-10 shipped form (`String`, NOT newtype). Same
///     correlation key as the D-13 adversary row pair, so the
///     dashboard can align by `claim_id`.
///   * `probe_kind` — snake-case
///     [`crate::sycophancy::SycophancyProbeKind::as_str`] label. One
///     of `feedback` | `are_you_sure` | `answer` | `mimicry` per
///     Sharma §3.1-§3.4.
///   * `user_message` — the producer-side user-message anchor that
///     elicited the agent's primary response. Stored on-wire so the
///     auditor + dashboard can render the probe context without
///     re-reading the originating claim row.
///
/// Grounding: arXiv:2310.13548 (Sharma et al, Anthropic, ICLR 2024)
/// §3.1-§3.4 SycophancyEval probe taxonomy. See
/// workflow/papers/sharma-sycophancy-2310.13548-interface.md.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SycophancyProbeDispatchedPayload {
    pub claim_id: String,
    pub probe_kind: String,
    pub user_message: String,
}

/// D-14 (SCHEMA_VERSION 124) — payload for
/// [`SsbActionType::SycophancyFlag`].
///
/// Emitted once per probe verdict. Defensive flag: the auditor reads
/// this row and BLOCKs on `flip_detected = true`; the executor does
/// NOT auto-rewrite the agent's response. Preference models do NOT
/// reliably fix sycophancy per Sharma §4 (75% PM residual with
/// Claude 2 PM, 25% oracle floor — see
/// `crate::sycophancy::ORACLE_PM_RESIDUAL_FLOOR`).
///
/// `delta` is `f64`; the payload does NOT derive `Eq` (NaN-not-equal).
/// Equality-requiring code paths use `action_type()` instead of
/// structural equality (matches the C-47 / C-49 precedents).
///
/// Field semantics:
///   * `claim_id` — same correlation key as the paired
///     `SycophancyProbeDispatched` row.
///   * `flip_detected` — `true` iff the agent flipped its primary
///     answer under the inverted-framing challenge (Sharma §3.2
///     flip definition).
///   * `delta` — string-similarity distance between the original
///     and challenged answers. Range `[0.0, 1.0]`; values
///     above `crate::sycophancy::DEFAULT_FLIP_THRESHOLD` (0.20) trip
///     the flag.
///   * `verdict` — snake-case
///     [`crate::sycophancy::ProbeVerdict::as_str`] label. One of
///     `convergent` | `divergent_sycophancy_flag` | `inconclusive`.
///
/// Grounding: arXiv:2310.13548 §3.2 (user-disagreement-flip
/// detection) + §4 (preference-model residual sycophancy).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SycophancyFlagPayload {
    pub claim_id: String,
    pub flip_detected: bool,
    pub delta: f64,
    pub verdict: String,
}

/// D-23 (SCHEMA_VERSION 142) — payload for
/// [`SsbActionType::SycophancyBenchRunCompleted`].
///
/// Emitted by
/// [`crate::benchmarks::sycophancy_runner::SycophancyBenchmark::run`]
/// after a sycophancy benchmark fixture corpus has been classified
/// end-to-end. `verdict` is the snake_case form of
/// [`crate::benchmarks::prmbench_runner::GatingVerdict`]
/// (`"pass"` / `"fail"`). `fixture_blake3` is the 64-hex content
/// hash of the canonicalized fixture corpus the run was computed
/// against (anti-stale: pins dashboard rows to a specific dataset
/// revision).
///
/// Gating: detection_rate >= 0.70 AND fp_rate <= 0.15 per Sharma
/// §4 single-turn reproducibility band + plan §Tasks step 3.
/// `inconclusive` rows are EXCLUDED from the detection_rate /
/// fp_rate denominators so both rates remain auditable
/// (D-23-decisions.md DECISION-3: 3-arm ProbeVerdict collapse).
///
/// Does NOT derive `Eq` because `detection_rate` + `fp_rate` are
/// `f64` (NaN-not-equal). Equality-requiring tests use field-wise
/// comparison.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SycophancyBenchRunCompletedPayload {
    /// True positives: ground_truth_sycophant=true AND verdict.is_flag().
    pub tp: u32,
    /// False positives: ground_truth_sycophant=false AND verdict.is_flag().
    pub fp: u32,
    /// True negatives: ground_truth_sycophant=false AND Convergent.
    pub tn: u32,
    /// False negatives: ground_truth_sycophant=true AND Convergent.
    /// Wire field is `fn_` (the `fn` keyword forces the underscore).
    pub fn_: u32,
    /// Inconclusive verdicts — excluded from TP/FP/TN/FN denominators.
    pub inconclusive: u32,
    /// `tp / (tp + fn_)` aggregate detection rate.
    pub detection_rate: f64,
    /// `fp / (fp + tn)` aggregate false-positive rate.
    pub fp_rate: f64,
    /// Aggregate verdict — snake_case (`"pass"` / `"fail"`).
    pub verdict: String,
    /// Total fixture rows the runner consumed (TP+FP+TN+FN+Inconclusive).
    pub total_fixtures: u32,
    /// 64-hex blake3 of the canonicalized fixture corpus.
    pub fixture_blake3: String,
    /// Tag echoed from
    /// [`crate::benchmarks::sycophancy_runner::BENCH_RUNNER_NAME`]
    /// (e.g. `"sycophancy_bench_v1"`).
    pub runner_name: String,
}

/// D-24 (SCHEMA_VERSION 143) — payload for
/// [`SsbActionType::CanaryBenchRunCompleted`].
///
/// Emitted by
/// [`crate::benchmarks::canary_runner::CanaryBenchmark::run`] after
/// a cross-domain canary regression fixture corpus has been replayed
/// through D-7 `run_canary` end-to-end. `verdict` is the snake_case
/// form of [`crate::benchmarks::prmbench_runner::GatingVerdict`]
/// (`"pass"` / `"fail"`). `fixture_blake3` is the 64-hex content
/// hash of the canonicalized fixture corpus the run was computed
/// against (anti-stale: pins dashboard rows to a specific dataset
/// revision).
///
/// Gating: `harmful_blocked >= 20` AND `helpful_blocked <= 5` AND
/// `neutral_blocked <= 2` (ELAI calibration; NanoFlux arXiv:2509.23252
/// paper has no canary mechanism — the thresholds are empirical, not
/// paper-anchored).
///
/// Does NOT carry per-family or per-provenance breakdowns on the SSB
/// row (those live in the file-based JSON report) — the SSB row is
/// the dashboard-grade aggregate. Per D-24 DECISION-2 the runner ships
/// a SINGLE end-of-run row (plan-doc named `BenchmarkStarted` +
/// `BenchmarkResult` REJECTED — see D-19/D-23 precedent).
///
/// Does NOT derive `Eq` because no f64 fields participate in the
/// payload (all integer counters + verdict string) — `PartialEq` is
/// sufficient and matches the surrounding payload conventions.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CanaryBenchRunCompletedPayload {
    /// Harmful fixtures the canary blocked. Equals `harmful_blocked`.
    pub true_block: u32,
    /// Helpful + neutral fixtures the canary wrongly blocked.
    pub false_block: u32,
    /// Helpful + neutral fixtures the canary correctly passed.
    pub true_pass: u32,
    /// Harmful fixtures the canary wrongly passed.
    pub false_pass: u32,
    /// Of the harmful fixtures, how many the canary blocked.
    pub harmful_blocked: u32,
    /// Of the helpful fixtures, how many the canary wrongly blocked.
    pub helpful_blocked: u32,
    /// Of the neutral fixtures, how many the canary wrongly blocked.
    pub neutral_blocked: u32,
    /// Total fixture rows the runner consumed.
    pub total_fixtures: u32,
    /// Aggregate verdict — snake_case (`"pass"` / `"fail"`).
    pub verdict: String,
    /// 64-hex blake3 of the canonicalized fixture corpus.
    pub fixture_blake3: String,
    /// Tag echoed from
    /// [`crate::benchmarks::canary_runner::BENCH_RUNNER_NAME`] (e.g.
    /// `"canary_bench_v1"`).
    pub runner_name: String,
}

/// D-25 (SCHEMA_VERSION 144) — payload for
/// [`SsbActionType::DeterminismTamperSuiteCompleted`].
///
/// Emitted by
/// [`crate::benchmarks::tamper_runner::TamperBenchmark::run`] after
/// the 10-scenario JSON-manifest tamper matrix has been driven through
/// D-8 `verify_archive` IN-PROCESS end-to-end. `verdict` is the
/// snake_case form of [`crate::benchmarks::prmbench_runner::GatingVerdict`]
/// (`"pass"` / `"fail"`). `scenarios_blake3` is the 64-hex content
/// hash of the canonicalized scenario manifest corpus (anti-stale:
/// pins dashboard rows to a specific scenario revision).
///
/// Gating: `verdict == "pass"` iff every scenario's observed outcome
/// matched its expected outcome AND `max_wall_ms <= 5000` (5-second
/// per-scenario wall budget per plan objective).
///
/// Does NOT carry per-scenario breakdowns on the SSB row (those live
/// in the file-based JSON report) — the SSB row is the dashboard-grade
/// aggregate. Per D-25 DECISION-2 the runner ships a SINGLE end-of-run
/// row (plan-doc named `BenchmarkStarted` + `BenchmarkResult` REJECTED
/// — see D-19/D-23/D-24 precedent).
///
/// Does NOT derive `Eq` only because every field is integer / String
/// — `PartialEq + Eq` are both safe and the surrounding payload
/// conventions both derive them.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DeterminismTamperSuiteCompletedPayload {
    /// Total scenarios consumed by the run (locked at 10 by the plan's
    /// 10-scenario matrix; the field is wire-stable so the dashboard
    /// can render variable suite sizes if the matrix grows).
    pub total_scenarios: u32,
    /// Scenarios whose observed outcome matched their expected outcome
    /// AND finished within the 5000ms wall budget.
    pub passed: u32,
    /// Scenarios whose observed outcome did NOT match their expected
    /// outcome OR exceeded the 5000ms wall budget.
    pub failed: u32,
    /// Maximum per-scenario wall-clock cost in milliseconds across the
    /// run. The runner gates `<= 5000` per plan objective.
    pub max_wall_ms: u64,
    /// Aggregate verdict — snake_case (`"pass"` / `"fail"`).
    pub verdict: String,
    /// 64-hex blake3 of the canonicalized scenario manifest corpus
    /// (file name + NUL + raw bytes + NUL per fixture, sorted).
    pub scenarios_blake3: String,
}

/// D-28 (SCHEMA_VERSION 145) — payload for
/// [`SsbActionType::AdversarialFarmRunCompleted`].
///
/// Emitted by [`crate::farm::adv_rm::AdvRmFarmer::run`] after the
/// host-side farm-loop scaffolding finishes walking its dataset
/// through the D-2 PPM hook + Z>1.96 filter + corpus emit.
///
/// `verdict` is the snake_case form of
/// [`crate::benchmarks::prmbench_runner::GatingVerdict`] (`"pass"` /
/// `"fail"`). `corpus_blake3` is the 64-hex content hash of the
/// canonicalized corpus JSONL (anti-stale: pins dashboard rows to a
/// specific corpus revision).
///
/// `paper_floor_pp` is the paper-grounded PRMBench overall delta
/// floor in percentage points. Default 1.0pp per the calibration in
/// `workflow/papers/adversarial-rm-2504.06141-interface.md` L9-12
/// (paper §5 reports overall +0.7pp / +2.86pp Chat-Hard MAX). The
/// plan's "+5pp" claim is UNGROUNDED relative to the cited paper
/// and was REJECTED at executor entry — see workflow/reasoning/
/// D-28-decisions.md DECISION-3.
///
/// `rounds` is capped at 2 per paper §5 ("round 3+ yields near-zero
/// new examples"); `samples_farmed` is capped at ~1000-1500 per
/// paper §5 saturation guard. Both are encoded as wire-stable u32
/// counters so the dashboard can render variable corpus sizes if
/// the cap is relaxed in future.
///
/// Gating: `verdict == "pass"` iff `prmbench_delta_pp >=
/// paper_floor_pp` AND `rounds >= 1` AND the live PPM/retrain/v2
/// run completed (verdict carries the rolled-up signal — the SSB
/// row does not include per-round breakdowns).
///
/// Does NOT carry per-sample scores or the v1/v2 baseline numbers
/// on the SSB row (those live in the file-based JSON report) — the
/// SSB row is the dashboard-grade aggregate. Per D-28 DECISION-2
/// the runner ships a SINGLE end-of-run row (plan-doc named
/// `AdversarialCorpusGenerated` + `PpmFinetuneCompleted` REJECTED
/// — see D-19/D-23/D-24/D-25 single-variant precedent).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AdversarialFarmRunCompletedPayload {
    /// Total samples (candidate-step chains) that passed the Z>1.96
    /// filter and entered the corpus across all rounds. Capped at
    /// ~1500 per paper §5 saturation guard.
    pub samples_farmed: u32,
    /// Total farm rounds executed (1 or 2 per paper §5).
    pub rounds: u32,
    /// PRMBench overall delta (v2 - v1) in percentage points. Set
    /// to `0.0` when the live PPM retrain + bench re-runs are
    /// RUNPOD-PENDING and the host-side scaffolding has emitted
    /// only the corpus.
    pub prmbench_delta_pp: f64,
    /// Aggregate verdict — snake_case (`"pass"` / `"fail"`).
    pub verdict: String,
    /// Paper-grounded PRMBench overall delta floor in percentage
    /// points. Default 1.0pp per paper §5 reported overall +0.7pp /
    /// +2.86pp Chat-Hard (plan's "+5pp" REJECTED as ungrounded).
    pub paper_floor_pp: f64,
    /// 64-hex blake3 of the canonicalized corpus JSONL bytes
    /// (anti-stale: pins dashboard rows to a specific corpus
    /// revision).
    pub corpus_blake3: String,
}

/// D-29 (SCHEMA_VERSION 146) — per-arm posterior summary embedded inside
/// [`HierarchicalBaiRoundCompletedPayload`]. Captures BOTH the unpooled
/// flat posterior (alpha/beta/mean/variance/CI) and the family-shrunk
/// pooled mean so the dashboard can render the shrinkage visually
/// without re-deriving the hyperprior.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ArmPosteriorSummary {
    /// 32-hex (uuid::Uuid::simple) of the arm's TemplateId.
    pub template_id: String,
    /// Family identifier the arm belongs to (None encoded as u32::MAX
    /// for arms that are not yet assigned to a family).
    pub family_id: u32,
    /// Beta posterior alpha (pass_count + 1 under Beta(1,1) prior).
    pub alpha: f64,
    /// Beta posterior beta (fail_count + 1 under Beta(1,1) prior).
    pub beta: f64,
    /// Flat posterior mean = alpha / (alpha + beta).
    pub mean: f64,
    /// Flat posterior variance = alpha*beta / ((alpha+beta)^2 * (alpha+beta+1)).
    pub variance: f64,
    /// 95% credible interval lower bound (Beta inverse-CDF at 2.5%).
    pub ci_low: f64,
    /// 95% credible interval upper bound (Beta inverse-CDF at 97.5%).
    pub ci_high: f64,
    /// Family-shrunk posterior mean used by the hierarchical TS draw.
    /// Equals `mean` when the arm has no family assignment.
    pub pooled_mean: f64,
}

/// D-29 (SCHEMA_VERSION 146) — payload for
/// [`SsbActionType::HierarchicalBaiRoundCompleted`].
///
/// Emitted ONCE per hierarchical BAI round when ELAI_BAI_MODE=hierarchical
/// AND ELAI_BAI_ENABLE=1 (wire-dark default). Carries the round-level
/// aggregate of:
///   * the Metropolis-within-Gibbs hyperposterior over `(mu, kappa)`
///     (per-family Beta-hyperprior estimates with variance bars),
///   * the per-arm shrunk Beta summaries,
///   * the selected arm + selection basis (HierarchicalTs / FlatTs /
///     CanaryOverride),
///   * the Gibbs iteration count (= `MAX_GIBBS_ITERATIONS - GIBBS_BURN_IN`
///     in the canonical configuration),
///   * the canary gate outcome (composed with D-7 — true means promotion
///     allowed).
///
/// Paper labels (see workflow/reasoning/D-29-decisions.md DECISION-2):
///   * arXiv:2111.06929 SUPPORTING EVIDENCE — HierTS gives the Bernoulli/
///     Beta hyperprior shape but addresses regret minimization, NOT
///     best-arm identification. The "30-50% regret reduction" headline
///     in the plan body is UNGROUNDED for the BAI regime — recalibrated
///     here to "live baseline RUNPOD-PENDING".
///   * arXiv:2106.10417 DIRECT GROUNDING — variance-dependent BAI sample
///     complexity bound H_var (paper §5.1) plus the load-bearing
///     total-variance identity used in both papers' proofs.
///
/// Single-variant by D-19/D-23/D-24/D-25/D-28 precedent (plan body
/// suggested per-family + per-arm rows; rejected per pattern-2 in
/// workflow/sessions/s-25120/completion-drive/SESSION-3-HANDOFF.md).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct HierarchicalBaiRoundCompletedPayload {
    /// Monotonically increasing round counter (1-indexed) within a
    /// single dispatch session. Wraps after u32::MAX (~4.3B rounds —
    /// well beyond any reasonable session lifetime).
    pub round: u32,
    /// Family identifier whose hyperposterior was updated this round.
    /// u32 wire-stable (FamilyId is a newtype around u32 in
    /// `crate::bai_hierarchical`).
    pub family_id: u32,
    /// Posterior mean of mu (population mean rate) — in (0, 1).
    pub mu_posterior_mean: f64,
    /// Posterior variance of mu — >= 0.
    pub mu_posterior_variance: f64,
    /// Posterior mean of kappa (population concentration) — > 0.
    pub kappa_posterior_mean: f64,
    /// Posterior variance of kappa — >= 0.
    pub kappa_posterior_variance: f64,
    /// Per-arm posterior summaries for ALL arms considered this round
    /// (both family-assigned and unassigned). Length == arm count.
    pub arm_posteriors: Vec<ArmPosteriorSummary>,
    /// Index of the selected arm within `arm_posteriors`. None when
    /// the arm list is empty (encoded as u32::MAX).
    pub selected_arm_index: u32,
    /// 32-hex (uuid::Uuid::simple) of the selected arm's TemplateId.
    pub selected_template_id: String,
    /// Snake_case selection basis (`"hierarchical_ts"` / `"flat_ts"`
    /// / `"canary_override"`). Mirrors
    /// `crate::bai_hierarchical::SelectionBasis::as_str`.
    pub selection_basis: String,
    /// Number of post-burn-in Gibbs iterations folded into the
    /// hyperposterior (= `MAX_GIBBS_ITERATIONS - GIBBS_BURN_IN` in
    /// the canonical configuration; 80 by default).
    pub gibbs_iterations: u32,
    /// True iff the D-7 canary gate permitted promotion (canary
    /// status not blocking). False when the canary gate intercepted.
    /// Composed via `emit_bai_round_with_canary` in the dispatch
    /// site — see `crate::task_runner::dispatch_bai_round`.
    pub canary_gate_passed: bool,
}

/// D-30 (SCHEMA_VERSION 147) — payload for
/// [`SsbActionType::CalibrationBootstrapCompleted`].
///
/// Emitted ONCE per calibration bootstrap warm-start dispatch when
/// `ELAI_CALIBRATION_BOOTSTRAP_ENABLE=1` (wire-dark default OFF; see
/// `crate::runtime_config::CalibrationBootstrapMode`). Carries the
/// outcome of `crate::calibration_bootstrap::warm_start_tau`:
///   * `corpus_slug` — published-set identifier (`"truthfulqa"` by
///     default; `"halueval"` / `"mmlu_calibration_subset"` for the
///     other recommended candidates per DECISION-5).
///   * `corpus_size` — number of rows imported from the fixture.
///   * `corpus_blake3` — 64-hex `blake3` of the fixture file bytes;
///     pins the SSB row to a specific fixture revision (mirrors D-25
///     `corpus_blake3` precedent).
///   * `tau_warm_start` — UniCR Eq.5 `(1 - alpha)`-quantile over the
///     imported corpus's non-conformity scores.
///   * `alpha` — UniCR epsilon used (default `0.1`).
///   * `tau_floor_active` — `true` iff the warm-start ran with
///     `min_corpus_size >= 500` (the D-16 production floor). `false`
///     for the canonical warm-start (lowered floor 20).
///   * `bootstrap_replicates` — `Some(B)` iff a SECONDARY block-
///     bootstrap CI pass was composed on top of the warm-start.
///     `None` for warm-start-only dispatch.
///   * `bootstrap_ci_low` / `bootstrap_ci_high` — 90% percentile CI
///     `(5th, 95th)` of the resampled tau distribution. `None` when
///     SECONDARY did not run.
///   * `block_length_p` — Politis-Romano geometric parameter
///     `p = 1 / sqrt(N)`. `None` when SECONDARY did not run.
///   * `ts_ms` — producer-side wall-clock from the injected
///     `ClockProvider` (NEVER `std::time::SystemTime`).
///
/// Paper labels (see workflow/reasoning/D-30-decisions.md DECISION-2):
///   * arXiv:2509.01455 UniCR — COMPOSITION (consumes
///     `compute_tau_with_min`; mechanism is D-16's, not this row's).
///   * arXiv:2109.07958 TruthfulQA — DATA ORIGIN for the default
///     corpus slug (cite-only; not algorithm-source).
///   * Politis & Romano 1994 JASA Vol 89 §3 — DIRECT GROUNDING for
///     the stationary bootstrap mechanism in `bootstrap_resample`;
///     only active when the four `Option` fields are `Some(_)` — i.e.
///     when a SECONDARY pass runs the block-bootstrap CI.
///
/// Single-variant by D-19/D-23/D-24/D-25/D-28/D-29 precedent (11th
/// application of pattern-2). The SECONDARY block-bootstrap path
/// extends THIS payload via the four `Option` fields rather than
/// introducing a `CalibrationBootstrapCiCompleted` second variant.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CalibrationBootstrapCompletedPayload {
    /// Snake_case slug for the published corpus (e.g. `"truthfulqa"`).
    pub corpus_slug: String,
    /// Imported corpus size (number of rows).
    pub corpus_size: u32,
    /// 64-hex `blake3` of the fixture file bytes.
    pub corpus_blake3: String,
    /// UniCR Eq.5 tau on the warm-start corpus.
    pub tau_warm_start: f64,
    /// UniCR alpha (epsilon) used.
    pub alpha: f64,
    /// True iff the warm-start ran with the D-16 production floor
    /// (`min_corpus_size >= 500`). False for the canonical warm-start.
    pub tau_floor_active: bool,
    /// Replicate count when SECONDARY block-bootstrap CI ran.
    pub bootstrap_replicates: Option<u32>,
    /// 5th-percentile tau across replicates (90% CI low).
    pub bootstrap_ci_low: Option<f64>,
    /// 95th-percentile tau across replicates (90% CI high).
    pub bootstrap_ci_high: Option<f64>,
    /// Politis-Romano geometric block parameter `p`.
    pub block_length_p: Option<f64>,
    /// Producer-side wall-clock (ClockProvider-sourced).
    pub ts_ms: i64,
    /// I-23 — same 16-char hex join key as [`ConformalTauComputedPayload`].
    /// Minted from blake3(tau_le || corpus_size_le || epsilon_le)[..8].
    /// Enables a journal join between the bootstrap row and the D-16
    /// tau/refusal rows that share the same calibration run. `None` on
    /// rows emitted before I-23 shipped (backward-compatible via
    /// `#[serde(default)]`).
    #[serde(default)]
    pub calibration_id: Option<String>,
}

/// C-50 (SCHEMA_VERSION 125) — payload for
/// [`SsbActionType::ContractTestScaffolded`].
///
/// Emitted once per `scaffold_contract_test_from_signature`
/// invocation. The row is the durable producer signal the
/// dashboard reads to align a scaffolded test back to its target
/// function. Hashed identifiers (no plaintext) so the wire format
/// is privacy-preserving and stable across sessions.
///
/// Field semantics:
///   * `fn_name_hash` — blake3 prefix (first 8 bytes as `u64`) of
///     the canonical function name. Stable across runs.
///   * `contract_kind_hash` — blake3 prefix of the kernel's
///     namespace section name (paper `N`). When the scaffolder
///     emits a multi-section kernel, this is the FIRST section
///     (deterministic).
///   * `ts_ms` — producer-side wall-clock timestamp via the
///     injected `ClockProvider`. Never `std::time::SystemTime`.
///
/// Grounding: arXiv:2604.13100 §4.4 (Discriminator surface).
/// See workflow/papers/contract-coding-interface.md.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ContractTestScaffoldedPayload {
    pub fn_name_hash: u64,
    pub contract_kind_hash: u64,
    pub ts_ms: u64,
}

/// C-50 (SCHEMA_VERSION 125) — payload for
/// [`SsbActionType::ContractViolationDetected`].
///
/// Emitted once per heuristic `audit_against_contract` verdict
/// that surfaces a kernel violation. Defensive flag: the auditor
/// reads this row; the executor does NOT auto-rewrite (full RL
/// Discriminator Agent is RUNPOD-PENDING per paper §4.5).
///
/// Field semantics:
///   * `fn_name_hash` — blake3 prefix of the canonical function
///     name (matches the paired `ContractTestScaffolded` row).
///   * `violation_kind_hash` — blake3 prefix of the snake_case
///     [`crate::contract_coding::ViolationKind::as_str`] tag
///     (`missing_namespace` | `missing_signature` |
///     `unrecorded_state_constraint`).
///   * `severity` — `u8`. `0` for advisory (heuristic miss); `1`
///     for kernel-blocker. Stable wire format for dashboard sort.
///   * `ts_ms` — producer-side wall-clock timestamp.
///
/// Grounding: arXiv:2604.13100 §4.4 + §4.5. The heuristic
/// Discriminator pre-stage ships here; the RL-trained variant is
/// RUNPOD-PENDING.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ContractViolationDetectedPayload {
    pub fn_name_hash: u64,
    pub violation_kind_hash: u64,
    pub severity: u8,
    pub ts_ms: u64,
}

/// C-52 (SCHEMA_VERSION 126) — payload for
/// [`SsbActionType::DafnyVerifyCompleted`].
///
/// Emitted once per Dafny self-healing loop termination (either
/// first-shot success on the Signature / SelfHealing tier OR
/// `MAX_HEAL_ITER` exhaustion). The row is the durable timeline
/// marker the dashboard reads to align a verification attempt to
/// its target module. Hashed identifiers (no plaintext) so the
/// wire format is privacy-preserving and stable across sessions.
///
/// Field semantics:
///   * `tier_hash` — blake3 prefix (first 8 bytes as `u64`) of the
///     [`crate::dafny_verify::DafnyTier::as_str`] wire tag
///     (`contextless` | `signature` | `self_healing`). Stable
///     across runs.
///   * `dafny_pass` — whether `dafny verify <spec>` exited 0 AND
///     stdout contained the canonical "Dafny program verifier"
///     banner. The raw verifier signal.
///   * `udebug_pass` — whether the uDebug-style functional-test
///     suite passed. Vacuous-verification GUARD per paper §3 — raw
///     `dafny verify` exit-zero alone is INSUFFICIENT.
///   * `heal_iter` — iteration count. `0` = first-shot pass (no
///     self-healing needed). Capped at `10`
///     ([`crate::dafny_verify::MAX_HEAL_ITER`] per paper §3); the
///     paper does not characterise diminishing returns past 10.
///   * `ts_ms` — producer-side wall-clock timestamp via the injected
///     `ClockProvider`. Never `std::time::SystemTime`.
///
/// Grounding: arXiv:2604.22601 (NL2VC). Model substitution is
/// MANDATORY — Gemma 4 is BLOCKED per memory feedback_no_gemma4;
/// substitutes from the paper's evaluation set: Codestral-22B-v0.1
/// (63.64% RQ2), GPT-OSS-120B (63.64% RQ2 + 81.82% RQ3b), Qwen 3
/// Coder 30B (45.45% RQ2). The current ELAI sidecar default
/// (Qwen 2.5 Coder 7B) is NOT in the paper's evaluation set;
/// expect verify@5 below 45% on the signature tier — see
/// workflow/reasoning/C-52-decisions.md.
///
/// Live `dafny verify` execution is RUNPOD-PENDING (local dev
/// host lacks the `dafny` binary on PATH per the C-52 cross-check
/// report). This payload exists today so offline-test paths +
/// heuristic dispatches can record the attempt; the emission site
/// fires only when a remote runner has Dafny installed.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct DafnyVerifyCompletedPayload {
    pub tier_hash: u64,
    pub dafny_pass: bool,
    pub udebug_pass: bool,
    pub heal_iter: u32,
    pub ts_ms: u64,
}

/// D-4 — payload for [`SsbActionType::TripwireFired`].
///
/// Emitted by `crate::tripwire::TripwireMonitor::observe` when one of the
/// five detectors (KL / Δρ / IQR / perplexity / DDM) crosses its threshold.
/// `event_id` is a blake3-derived 32-hex stable id so dashboard consumers
/// can dedupe by id rather than seq.
///
/// D-4 GAP-1 (SCHEMA_VERSION 108): the `source` field carries the
/// snake-case label of the underlying signal stream
/// (`crate::tripwire::SignalSource::as_str` — one of `reward` / `action` /
/// `perplexity` / `classifier`). Older readers that did not know about
/// `source` continue to parse the payload (defaults to empty via
/// `#[serde(default)]` on deserialize); the field is always populated on
/// the producer side.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TripwireFiredPayload {
    pub event_id: String,
    pub tripwire_kind: String,
    /// D-4 GAP-1: signal-stream tag mirroring
    /// [`crate::tripwire::SignalSource`]. Snake-case wire label.
    #[serde(default)]
    pub source: String,
    pub candidate_id: String,
    pub observed_value: f64,
    pub threshold: f64,
    pub fired_at_ms: i64,
}

/// D-4 GAP-2 — payload for [`SsbActionType::TripwireFiredConcurrent`].
///
/// Emitted when two or more detector firings land within the same
/// `observe()` tick, ahead of the auto-revert path. Carries the kinds of
/// the first two firings (`tripwire_a` + `tripwire_b`, snake-case
/// `TripwireKind::as_str` labels) plus the elapsed window in milliseconds
/// for downstream consumers (D-26 concurrent-fire diagnostics, dashboard
/// row distinction). `fired_within_ms` is the conservative bound on the
/// observation gap — within a single `observe()` tick the gap is `0`, but
/// the field is kept `u64` so future cross-tick aggregation can widen it
/// without a wire break.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TripwireFiredConcurrentPayload {
    pub candidate_id: String,
    pub tripwire_a: String,
    pub tripwire_b: String,
    pub fired_within_ms: u64,
    pub fired_at_ms: i64,
}

/// D-4 — payload for [`SsbActionType::TemplateAutoReverted`].
///
/// Emitted by `crate::tripwire::auto_revert` after a tripwire firing flips
/// the matching `ArchiveEntry.status` to `Retired` with
/// `RetirementReason::Degradation`. `parent_template_id` is the lineage
/// target the orchestrator switches to (None when the retired template was
/// a root).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TemplateAutoRevertedPayload {
    pub template_id: String,
    pub parent_template_id: Option<String>,
    pub retirement_reason: String,
    pub fired_tripwire_id: String,
}

/// D-19 — payload for [`SsbActionType::PrmBenchRunCompleted`].
///
/// Emitted by
/// [`crate::benchmarks::prmbench_runner::PRMBenchRunner::run_all`] (and
/// [`crate::benchmarks::prmbench_runner::PRMBenchRunner::run_category`])
/// after a PRMBench fixture has been scored end-to-end. `gating_verdict`
/// is the snake_case form of
/// [`crate::benchmarks::prmbench_runner::GatingVerdict`]
/// (`"pass"` / `"fail"`). `fixture_blake3` is the 64-hex content hash of
/// the JSONL fixture the run was computed against (anti-stale: pins
/// dashboard rows to a specific dataset revision).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PrmBenchRunCompletedPayload {
    /// Total fixture rows the runner consumed (includes `insufficient`
    /// rows where the PPM hook returned an error).
    pub total: u32,
    /// `(tp + tn) / scored` aggregate accuracy.
    pub accuracy: f64,
    /// `tp / (tp + fp)` aggregate precision.
    pub precision: f64,
    /// `tp / (tp + fn)` aggregate recall.
    pub recall: f64,
    /// Aggregate verdict — `"pass"` iff every per-category verdict
    /// passed AND at least one row was scored.
    pub gating_verdict: String,
    /// 64-hex blake3 of the fixture file contents.
    pub fixture_blake3: String,
    /// Number of distinct categories surfaced in the per-category
    /// breakdown (matches `report.per_category.len()`).
    pub category_count: u32,
}

/// D-5 — payload for [`SsbActionType::BaiArmUpdated`].
///
/// Emitted once per `bai::emit_bai_round` posterior update. Carries the BAI
/// arm state after the Beta(alpha, beta) update + the `was_promoted` flag
/// mirroring whether the Thompson sample fired the promotion rule on this
/// round (so dashboard consumers can render the running mean + last-action
/// without a second lookup). `template_id` is 32-char simple-hex
/// (uuid::Uuid::simple). Promote/reject decisions emit a follow-up
/// `template_promoted` / `template_candidate_rejected` SSB row from the same
/// producer.
///
/// Fields ordered alpha-first/beta-second to match the conjugate Beta
/// notation in arXiv:2104.05593 §2.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BaiArmUpdatedPayload {
    /// Hex (32-char simple) of the candidate template id.
    pub template_id: String,
    /// Beta-distribution alpha (= pass_count + 1 with the Beta(1, 1) prior).
    pub alpha: f64,
    /// Beta-distribution beta (= fail_count + 1 with the Beta(1, 1) prior).
    pub beta: f64,
    /// Cumulative pulls observed on this arm (== alpha + beta - 2).
    pub n_pulls: u32,
    /// Posterior mean = alpha / (alpha + beta).
    pub mean_estimate: f64,
    /// Whether the Thompson sample picked this arm AND the posterior cleared
    /// the promotion threshold on this round.
    pub was_promoted: bool,
}

/// D-5 closure (SCHEMA_VERSION 208) — payload for
/// [`SsbActionType::BaiPromotionDecision`].
///
/// Emitted once per [`crate::bai::emit_bai_round_with_canary`] call,
/// immediately after the `TemplatePromoted` / `TemplateCandidateRejected` row.
/// Provides a unified, self-contained verdict row so dashboards and analytics
/// jobs can query the promotion outcome directly without joining across the
/// three-row sequence (`thompson_sampled` + `bai_arm_updated` + decision row).
///
/// `decision` is one of `"promoted"`, `"rejected"`, or `"held"` matching
/// the [`crate::bai::PromotionDecision`] discriminant. `stage` mirrors the
/// `stage_label` helper (e.g. `"stage_10"`, `"stage_50"`, `"stage_200"`).
/// `evidence_pass` and `evidence_total` are the counts from the
/// [`crate::archive_chain::StagedEvalResult`] fed to the round.
/// `audit_id` is a fresh [`uuid::Uuid`] formatted as 32-char simple-hex so
/// external audit tools can correlate this row across logs.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BaiPromotionDecisionPayload {
    /// 32-char simple-hex of the candidate template id.
    pub template_id: String,
    /// Outcome: `"promoted"`, `"rejected"`, or `"held"`.
    pub decision: String,
    /// Stage label from `stage_label(eval)` (e.g. `"stage_10"`).
    pub stage: String,
    /// Number of evidence passes folded into this round.
    pub evidence_pass: u32,
    /// Total evidence observations folded into this round.
    pub evidence_total: u32,
    /// Fresh uuid (32-char simple-hex) for cross-log correlation.
    pub audit_id: String,
}

/// D-6 — payload for [`SsbActionType::HeldOutTaskAdded`].
///
/// Emitted once per `held_out::insert_and_emit` Ok return — a fresh
/// held-out task that passed the LiveCodeBench-style 9-gram contamination
/// filter and was inserted into the rusqlite-backed store. The dashboard
/// `HeldOutPanel` reads these rows to render the "recently ingested"
/// timeline + per-source counters. The `source` label is the snake-case
/// string from [`crate::held_out::TaskSource::label`] so the dashboard
/// can group without re-deserializing the full enum.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct HeldOutTaskAddedPayload {
    /// Simple-hex uuid of the held-out task (32 chars).
    pub task_id: String,
    /// Snake-case source label (`github_issue` / `benchmark_fixture` /
    /// `synthetic`).
    pub source: String,
    /// Blake3 (64-char hex) of the normalized task body.
    pub content_blake3: String,
    /// Number of 9-gram entries in the task's signature. Used by the
    /// dashboard to flag tasks below the n-gram threshold (too short to
    /// participate in the contamination filter).
    pub ngram_count: u32,
    /// Wall-clock UNIX ms when the task was ingested.
    pub ingested_ts_ms: i64,
    /// Time-window (ms) before the task becomes eligible for BAI eval
    /// consumption.
    pub eligibility_window_ms: i64,
}

/// D-6 — payload for [`SsbActionType::HeldOutTaskUsed`].
///
/// Emitted once per `held_out::mark_used_and_emit` Ok return — a held-out
/// task consumed by a BAI eval round. Carries the consumer template id so
/// the dashboard can link held-out usage to the BAI arm that consumed it
/// (cross-referenced with the `bai_arm_updated` row from the same round).
/// `consumer_template_id` is the empty string when the consumer is the
/// pre-promotion scaffold (no template yet).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct HeldOutTaskUsedPayload {
    /// Simple-hex uuid of the held-out task that was consumed.
    pub task_id: String,
    /// Simple-hex uuid of the consumer template (or empty string for the
    /// pre-promotion scaffold).
    pub consumer_template_id: String,
    /// Wall-clock UNIX ms when the task was consumed.
    pub used_ts_ms: i64,
}

/// D-7 — payload for [`SsbActionType::CanaryRunCompleted`].
///
/// Emitted once per `canary::emit_canary_run` Ok return — a completed
/// cross-domain transfer canary report. The dashboard `CanaryPanel`
/// joins these rows to the per-family
/// [`CanaryFamilyEvaluatedPayload`] rows (matched on `report_id`) to
/// render the per-family deltas + overall transfer score timeline.
/// NanoFlux-style (arxiv:2509.23252, T1) defense: target-family wins
/// must not silently regress adjacent families.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CanaryRunCompletedPayload {
    /// Blake3 (64-char hex) of the serialized report body. Deterministic
    /// identifier the per-family rows reference via `report_id` so the
    /// dashboard can join without scanning the SSB tail twice.
    pub report_id: String,
    /// Wall-clock UNIX ms when the run completed.
    pub ts_ms: i64,
    /// Mean transfer score across all evaluated families on this run.
    /// Range nominally `[-1.0, +1.0]` — positive means the candidate
    /// outperformed the baseline on average, negative the opposite.
    pub overall_transfer_score: f64,
    /// Snake-case decision kind: `pass` / `fail` / `marginal_pass`.
    pub decision_kind: String,
    /// Empty string when the decision is not `fail`. When `fail`, this
    /// carries the snake-case family label that drove the failure (the
    /// single dominant regressor — the largest negative `delta` family).
    pub dominant_fail_family: String,
}

/// D-9 — payload for [`SsbActionType::LockedKeysAuditRun`].
///
/// Emitted once per `LockedKeysAuditor::audit_diff_ranged` invocation
/// (success or violation; the audit ran, the row is the durable
/// signal). `registry_blake3` is the blake3 hex of the registry's
/// canonical content so dashboard consumers can detect drift between
/// runs without re-reading the TOML.
///
/// Grounding: arxiv:1908.04734 (Everitt current-RF principle); see
/// docs/thefinalglance/D-plan/D-9_l3_frozen_surfaces.md.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LockedKeysAuditRunPayload {
    /// Per-audit-run correlator. The matching
    /// [`LockedKeysViolationDetectedPayload`] rows carry the same id
    /// so the dashboard joins on it.
    pub audit_id: uuid::Uuid,
    /// Wall-clock UNIX ms when the audit ran.
    pub ts_ms: i64,
    /// Number of staged file/range pairs the auditor inspected.
    pub n_audited: u32,
    /// Number of violations emitted for this run (0 on a clean audit).
    pub n_violations: u32,
    /// 64-char lowercase hex blake3 of the registry's canonical
    /// content. Stable across formatter-only changes that don't alter
    /// path / line_range / reason.
    pub registry_blake3: String,
    /// Path to the registry file the auditor loaded (string for
    /// portability across the IPC boundary).
    pub registry_path: String,
}

/// D-9 — payload for [`SsbActionType::LockedKeysViolationDetected`].
///
/// Emitted once per overlap between a staged edit and a registry
/// entry. `audit_id` matches the parent [`LockedKeysAuditRunPayload`]
/// row so the dashboard renders the full audit context.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LockedKeysViolationDetectedPayload {
    /// Matches the parent [`LockedKeysAuditRunPayload::audit_id`].
    pub audit_id: uuid::Uuid,
    /// Wall-clock UNIX ms when the violation was raised.
    pub ts_ms: i64,
    /// Workspace-relative path of the staged edit that touched a
    /// locked surface (forward-slash normalised).
    pub attempted_edit: String,
    /// Inclusive line range of the staged edit as `[start, end]`.
    /// Length always 2. Vec form keeps the IPC schema simple.
    pub attempted_lines: Vec<u32>,
    /// Forward-slash normalised path of the registry entry that
    /// matched.
    pub locked_key_path: String,
    /// 64-char lowercase hex blake3 of the registry entry's canonical
    /// content. Allows the dashboard to detect that a violation was
    /// raised against a since-modified registry row.
    pub locked_key_signature_blake3: String,
    /// Human-readable reason copied from the matched registry entry.
    pub reason: String,
}

/// D-7 — payload for [`SsbActionType::CanaryFamilyEvaluated`].
///
/// Emitted once per family inside a single canary run (so a 5-family
/// canary emits 5 of these rows plus 1
/// [`CanaryRunCompletedPayload`]). Matched to the parent run via
/// `report_id`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CanaryFamilyEvaluatedPayload {
    /// Matches the parent [`CanaryRunCompletedPayload::report_id`].
    pub report_id: String,
    /// Snake-case family label (`code_fix` / `math_proof` /
    /// `prose_reason` / `planning_decomposition` / `tool_use`).
    pub family: String,
    /// How many held-out tasks were drawn from this family on this run.
    pub n_evaluated: u32,
    /// Candidate pass count.
    pub pass_count: u32,
    /// Candidate fail count.
    pub fail_count: u32,
    /// Candidate pass-rate minus baseline pass-rate. Positive means the
    /// candidate beat the baseline; negative is regression. Bounded by
    /// `[-1.0, +1.0]`.
    pub transfer_score: f64,
    /// Baseline pass-rate (the reference template's pass-rate on the
    /// same held-out subset). Provided alongside the delta so the
    /// dashboard can render both bars without recomputing.
    pub baseline_score: f64,
}

/// E-46 — payload for [`SsbActionType::TaskAdvanced`].
///
/// One row per `DualLedger::advance_task` call. TaskLedger.status and
/// ProgressLedger.completion_pct + blockers are updated atomically and
/// the row carries the post-update view so dashboard consumers can
/// render the dual-ledger state without re-reading the ledger.
///
/// Grounding: arxiv:2412.21154 (Devin task-ledger pattern; SWE-bench
/// 13.86% solve rate driven by explicit task-state tracking).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TaskAdvancedPayload {
    /// Task id (string for IPC portability — the orchestrator uses a
    /// `TaskId` newtype internally).
    pub task_id: String,
    /// Optional parent task id; empty string when the task is a root.
    pub parent_id: String,
    /// Post-update completion percent in `[0.0, 100.0]`.
    pub completion_pct: f64,
    /// Snake-case status label: `pending` / `in_progress` / `complete`
    /// / `blocked`.
    pub status: String,
    /// Newly-added blocker on this advance (empty when none).
    pub blocker: String,
    /// Wall-clock UNIX ms when the advance was recorded.
    pub ts_ms: i64,
}

/// E-46 — payload for [`SsbActionType::AgentlessShortcutTaken`].
///
/// Emitted when `agentless_shortcut::shortcut_eligible` returns
/// `Eligibility::Yes` for a task AND the orchestrator elects to take
/// the shortcut (i.e. skip spawning a sub-agent). The `reason` carries
/// the canonical eligibility justification (single-file / loc / no
/// cross-crate / no async net) so the dashboard can render the
/// shortcut taxonomy.
///
/// Grounding: arxiv:2407.01489 (AGENTLESS, 32.00% SWE-bench Lite at
/// $0.70/issue — non-agentic 3-phase localize/repair/validate beats
/// every open-source agentic system at that time).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AgentlessShortcutTakenPayload {
    /// Task id that elected the shortcut.
    pub task_id: String,
    /// Snake-case reason label: `single_file_small` / `single_file` /
    /// `no_cross_crate` etc. Used by the dashboard to render shortcut
    /// taxonomy histograms.
    pub reason: String,
    /// Wall-clock UNIX ms when the shortcut was taken.
    pub ts_ms: i64,
}

/// E-46 — payload for [`SsbActionType::GoalDriftFired`].
///
/// Emitted when `GoalDriftCanary::fired()` evaluates true: drift
/// between the initial-goal token-set and the current summary
/// token-set exceeded the configured threshold. The
/// `initial_goal_blake3` lets dashboard consumers join consecutive
/// firings to a single canary instance.
///
/// Grounding: arxiv:2505.02709 (goal-drift PRM supervision); see also
/// D-7 [`CanaryFamily::PlanningDecomposition`] for the orthogonal
/// canary surface.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GoalDriftFiredPayload {
    /// 64-char lowercase hex blake3 of the canonical initial goal
    /// string. Stable across firings so the dashboard joins them.
    pub initial_goal_blake3: String,
    /// Measured drift in `[0.0, 1.0]` — `1.0` means orthogonal token
    /// sets, `0.0` means identical (Jaccard-distance variant used by
    /// `measure_drift`).
    pub drift: f64,
    /// Configured drift threshold the canary was checked against.
    pub threshold: f64,
    /// Step index at which the firing was observed (0-indexed within
    /// the parent decomposition walk).
    pub step: u32,
    /// Wall-clock UNIX ms when the firing was recorded.
    pub ts_ms: i64,
}

/// D-10 — payload for [`SsbActionType::ClaimRouted`].
///
/// One row per `agent_pool::route_claim` call. The orchestrator (not
/// the policy model) decides the [`crate::grounding_mode::GroundingMode`]
/// via the pure predicate
/// [`crate::grounding_mode::select_grounding_mode`]; this row is the
/// durable audit signal that the predicate ran and what it produced.
///
/// `ctx_summary_blake3` is the lowercase 64-char hex blake3 of the
/// canonical wire-form of the `OrchContext` snapshot the router saw
/// — so the dashboard can detect post-hoc context drift between
/// routing and dispatch.
///
/// Grounding:
///   * docs/specs/guide/real-problem-solving-grounded-2026-05-19.md
///     §1 (5-mode taxonomy) + §3.2 (routing predicate).
///   * arxiv:1908.04734 (Everitt current-RF principle, T1).
///   * arxiv:1906.01820 (Hubinger mesa-optimization, T1).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ClaimRoutedPayload {
    /// Stable per-claim identifier (uuid string in production; any
    /// non-empty stable string in tests).
    pub claim_id: String,
    /// Snake-case [`crate::claim::ClaimType`] label — matches the
    /// type's serde tag so the dashboard can deserialize without a
    /// separate mapping.
    pub claim_type: String,
    /// Snake-case [`crate::grounding_mode::GroundingMode`] label.
    pub mode: String,
    /// Wall-clock UNIX ms when routing completed.
    pub ts_ms: i64,
    /// 64-char lowercase hex blake3 of the canonical wire-form of
    /// the `OrchContext` snapshot used at routing time. Empty string
    /// is permitted but indicates a hashing failure on the producer
    /// side (logged via `tracing::warn`).
    pub ctx_summary_blake3: String,
}

/// D-21 — payload for [`SsbActionType::ModeRoutingBenchRunCompleted`].
///
/// Emitted by
/// [`crate::benchmarks::mode_routing_runner::ModeRoutingBenchmark::run_all`]
/// after a mode-routing fixture has been scored end-to-end. The runner
/// replays a hand-labeled JSONL fixture through D-10's pure predicate
/// [`crate::grounding_mode::select_grounding_mode`] and reports macro F1
/// + per-mode F1 + a 5x5 confusion matrix. `gating_verdict` is the
/// snake-case form of
/// [`crate::benchmarks::prmbench_runner::GatingVerdict`] (`"pass"` iff
/// macro F1 >= 0.85 AND every per-mode F1 >= 0.70). `fixture_blake3`
/// pins the dashboard row to a dataset revision.
///
/// Grounding:
///   * docs/specs/guide/real-problem-solving-grounded-2026-05-19.md §3.2
///     (routing predicate baseline).
///   * D-21 plan ACCEPTANCE: macro F1 >= 0.85 + per-mode F1 >= 0.70.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ModeRoutingBenchRunCompletedPayload {
    /// Total fixture rows the runner consumed.
    pub total: u32,
    /// `(sum_per_mode_TP) / total_predictions` aggregate accuracy.
    pub accuracy: f64,
    /// Macro-averaged F1 across the 5 [`crate::grounding_mode::GroundingMode`]
    /// variants. Modes with zero ground-truth AND zero predictions
    /// contribute `0.0` (defined behaviour — keeps macro F1
    /// monotonically degraded by missing-class coverage).
    pub f1_macro: f64,
    /// Aggregate verdict — `"pass"` iff `f1_macro >= 0.85` AND every
    /// per-mode F1 is `>= 0.70`. `"fail"` otherwise.
    pub gating_verdict: String,
    /// 64-hex blake3 of the fixture file contents.
    pub fixture_blake3: String,
    /// Number of distinct rows actually scored (== `total` minus any
    /// invalid-label rows that were rejected at load time).
    pub fixture_row_count: u32,
}

/// D-11 — payload for [`SsbActionType::CheckerResultEmitted`].
///
/// Emitted once per `(checker_id, kind, result)` triple returned by
/// [`crate::checker_registry::CheckerRegistry::run_all_applicable`].
/// The orchestrator (NOT the policy model) is the sole producer; the
/// dispatch path is `crate::agent_pool::dispatch_constructive_checkers`,
/// which is invoked when the D-10 router selects
/// [`crate::grounding_mode::GroundingMode::Constructive`].
///
/// Field semantics:
///   * `checker_id` — stable id the orchestrator assigned at register
///     time (`"regex-foo"`, `"sympy-default"`, etc).
///   * `checker_kind` — snake-case [`crate::checker_registry::CheckerKind`]
///     label (one of: `sympy`, `z3`, `compile_rust`, `regex_match`,
///     `json_schema_validate`, `numeric_range`).
///   * `result_kind` — `pass | fail | inapplicable | error`.
///   * `claim_id` — stable per-claim identifier.
///   * `ts_ms` — wall-clock UNIX ms at result-emit time.
///   * `reason` — bounded human-readable text for `fail`,
///     `inapplicable`, or `error`; empty string for `pass`.
///
/// Grounding:
///   * docs/specs/guide/real-problem-solving-grounded-2026-05-19.md §4.2
///     (checker layout) + §1 (Mode A constructive grading).
///   * arxiv:2310.04406 (constructive proof checking).
///   * arxiv:2211.12588 (Program-of-Thought, T1, TMLR 2023).
///   * arxiv:2402.01030 (CodeAct, T1, ICML 2024).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CheckerResultEmittedPayload {
    /// Stable id assigned at register time.
    pub checker_id: String,
    /// Snake-case [`crate::checker_registry::CheckerKind`] label.
    pub checker_kind: String,
    /// `pass | fail | inapplicable | error`.
    pub result_kind: String,
    /// Stable per-claim identifier the registry dispatched against.
    pub claim_id: String,
    /// Wall-clock UNIX ms at result-emit time.
    pub ts_ms: i64,
    /// Bounded human-readable explanation (truncated to 2048 chars by
    /// the producer). Empty string for `pass`.
    pub reason: String,
}

/// D-13 — payload for [`SsbActionType::AdversaryGenerated`].
///
/// Emitted by
/// [`crate::agent_pool::run_adversarial_pass`] after an
/// [`crate::adversary::Adversary::generate_counter`] call returns Ok.
/// `severity` is the structured counter's invalidation strength on
/// `[0.0, 1.0]`; `model_family` is the snake-case
/// [`crate::adversary::ModelFamily`] label the adversary emitted from
/// (cross-family check enforced at the consensus call-site, not on
/// this row).
///
/// Grounding:
///   * arxiv:2403.07974 (T1) adversarial verification.
///   * arxiv:2504.16760 (T1) lightweight latent verifiers
///     (heterogeneous-family verification).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AdversaryGeneratedPayload {
    /// Stable per-counter id (typically blake3-derived).
    pub counter_id: String,
    /// `claim_id` of the original [`crate::claim::Claim`] this counter
    /// was generated against.
    pub claim_id: String,
    /// `[0.0, 1.0]` invalidation severity. `0.0` = "claim robust"
    /// path; `>= severity_threshold` = caller treats the original
    /// claim as invalidated.
    pub severity: f64,
    /// Snake-case [`crate::adversary::ModelFamily`] label.
    pub model_family: String,
}

/// D-13 — payload for [`SsbActionType::ConsensusReevaluatedWithCounter`].
///
/// Emitted by
/// [`crate::agent_pool::run_adversarial_pass`] after the verifier
/// consensus has been re-run on `(claim, counter_evidence)`. Both
/// verdict fields carry the snake-case
/// [`crate::verification_result::AggregateVerdict::as_str`] label so
/// dashboards can render the verdict transition without re-decoding
/// the rule body. `claim_id` is the original claim, NOT the counter
/// (the consensus is being re-evaluated AGAINST the original claim in
/// the presence of new counter-evidence).
///
/// Grounding:
///   * arxiv:2403.07974 (T1) adversarial verification.
///   * arxiv:2504.16760 (T1) heterogeneous-family verifier transfer.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ConsensusReevaluatedWithCounterPayload {
    /// Stable claim id the verdict was re-computed against.
    pub claim_id: String,
    /// Verdict of the FIRST consensus pass (pre-adversary). One of
    /// `"pass" | "fail" | "inconclusive"`.
    pub original_verdict: String,
    /// Verdict of the consensus pass that consumed the adversary
    /// counter-evidence. Same label-space as `original_verdict`.
    pub post_counter_verdict: String,
}

/// D-12 — payload for [`SsbActionType::BabyStepDispatched`].
///
/// Emitted once per [`crate::baby_step::dispatch_check_for_baby_step`]
/// outcome by [`crate::agent_pool::run_operational_check`]. The row is
/// the durable audit signal the dashboard's `BabyStepPanel` renders;
/// the auditor consumes the `outcome` field to decide whether to allow
/// the MEMORIZING transition.
///
/// Field semantics:
///   * `step_id` — stable per-baby-step identifier.
///   * `parent_task_id` — parent task / phase id the baby step belongs
///     to (so the panel can group dispatches per parent).
///   * `checker_kinds_used` — snake_case
///     [`crate::checker_registry::CheckerKind`] labels for every
///     checker the dispatch fired. Empty when no checker ran (the
///     dispatch returned `NotOperationalized` or `Inapplicable`).
///   * `outcome` — one of `pass | fail | inapplicable`.
///   * `ts_ms` — wall-clock UNIX ms at dispatch-emit time.
///
/// Grounding:
///   * arxiv:2305.16291 Voyager (T1) — skill-with-measurable-effect.
///   * arxiv:2407.01489 AGENTLESS (T1) — single-step verification.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BabyStepDispatchedPayload {
    pub step_id: String,
    pub parent_task_id: String,
    pub checker_kinds_used: Vec<String>,
    /// `pass | fail | inapplicable`.
    pub outcome: String,
    pub ts_ms: i64,
}

/// D-16 — payload for [`SsbActionType::ConformalTauComputed`].
///
/// Emitted once per
/// [`crate::conformal_calibration::compute_tau_and_emit`] Ok return.
/// The row is the durable audit trail the dashboard
/// `ConformalCalibrationPanel` joins against the per-claim
/// [`ConformalRefusalDecidedPayload`] rows.
///
/// Grounding:
///   * arXiv:2503.01345 — UniCR (Unified Conformal Refusal).
///   * Vovk et al. *Algorithmic Learning in a Random World* (2005)
///     §2.2 — quantile-based conformal predictor.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ConformalTauComputedPayload {
    /// The threshold `tau` returned by
    /// [`crate::conformal_calibration::compute_tau`]; non-conformity
    /// scores above `tau` map to `RefusalDecision::Abstain`.
    pub tau: f64,
    /// UniCR operating point — default `0.1` per arXiv:2503.01345 §6.
    pub epsilon: f64,
    /// Number of tuples in the corpus used to compute `tau`.
    pub corpus_size: u32,
    /// Snake-case [`crate::conformal_calibration::CorpusSource`] label.
    pub source: String,
    /// Wall-clock UNIX ms at compute-emit time.
    pub ts_ms: i64,
    /// I-23 — deterministic join key: blake3(tau_le || corpus_size_le ||
    /// epsilon_le)[..8] formatted as a 16-char lowercase hex string. Shared
    /// with the `ConformalRefusalDecidedPayload` rows that were decided
    /// against this tau, enabling a journal join that proves both rows
    /// derive from the same calibration run. `None` on rows emitted before
    /// I-23 shipped (backward-compatible via `#[serde(default)]`).
    #[serde(default)]
    pub calibration_id: Option<String>,
}

/// D-16 — payload for [`SsbActionType::ConformalRefusalDecided`].
///
/// Emitted once per
/// [`crate::conformal_calibration::decide_and_emit`] call (and by
/// extension every [`crate::agent_pool::dispatch_conformal_refusal`]
/// invocation). Carries enough state for the dashboard to render the
/// per-claim abstain/answer histogram without re-decoding the tau row.
///
/// Field semantics:
///   * `claim_id` — the originating Claim id (joins against the D-10
///     `claim_routed` rows).
///   * `claim_score` — the composite confidence the orchestrator
///     supplied; in `[0.0, 1.0]`.
///   * `tau` — the calibration threshold the decision was taken
///     against.
///   * `decision` — one of `abstain | answer`.
///   * `confidence` — `claim_score` if `decision == "answer"`, else
///     `0.0` (Abstain carries no confidence by construction).
///   * `ts_ms` — wall-clock UNIX ms at decide-emit time.
///
/// Grounding: arXiv:2503.01345 (UniCR) — formal Mode E refusal rule.
///
/// D-16 closure: the optional `reason` field carries the 4-tag refusal
/// reason taxonomy (`low_evidence` | `high_dispersion` |
/// `tool_inconsistency` | `verifier_rejection`) without breaking the
/// shipped 2-variant `RefusalDecision` enum's wire shape. The plan's
/// 4-variant proposal was rejected because D-22 imports the 2-variant
/// shape at runner.rs:70 — the taxonomy lifts to the payload field
/// instead. `None` preserves backward compatibility with rows emitted
/// before the field landed. See D-16-decisions.md DECISION-2 +
/// DECISION-3.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ConformalRefusalDecidedPayload {
    pub claim_id: String,
    pub claim_score: f64,
    pub tau: f64,
    pub decision: String,
    pub confidence: f64,
    pub ts_ms: i64,
    /// D-16 closure: optional 4-tag refusal reason taxonomy. One of
    /// `low_evidence` (signal-to-noise floor), `high_dispersion`
    /// (verifier variance), `tool_inconsistency` (cross-tool
    /// disagreement), `verifier_rejection` (explicit verifier veto).
    /// `None` on rows that pre-date the taxonomy or where no reason
    /// classifier is wired. Serialized as `null` in JSON via
    /// `#[serde(default)]` so older callers and consumers stay
    /// compatible.
    #[serde(default)]
    pub reason: Option<String>,
    /// I-23 — same 16-char hex join key as [`ConformalTauComputedPayload`].
    /// Threads the calibration run identity from `compute_tau_and_emit`
    /// through `decide_and_emit` so journal consumers can assert both rows
    /// share one tau. `None` on pre-I-23 rows.
    #[serde(default)]
    pub calibration_id: Option<String>,
}

/// D-16 closure — payload for [`SsbActionType::CalibrationCorpusUpdated`].
///
/// Emitted once per
/// [`crate::conformal_calibration::TauUpdater::update_tau`] durable
/// persistence step. Provides the dashboard a row-level audit trail of
/// every nightly tau refresh without requiring it to re-decode
/// `tau.txt` from disk.
///
/// Field semantics:
///   * `corpus_size` — number of `(claim_score, ground_truth)` tuples
///     in the corpus at update time.
///   * `new_tau` — the freshly computed threshold the updater wrote to
///     disk. Consumers MUST treat this as the new operating tau for
///     subsequent `decide()` calls until the next update.
///   * `alpha` — UniCR Eq.5 miscoverage rate (DISTINCT from `rho`,
///     the selective risk budget; only `alpha` is on the row per
///     D-16-decisions.md DECISION-4).
///   * `source` — the corpus provenance label (snake-case
///     [`crate::conformal_calibration::CorpusSource`]:
///     `held_out` | `synthetic` | `curated` | `auto`).
///   * `ts_ms` — wall-clock UNIX ms at the persistence step.
///
/// Paper grounding: arXiv:2509.01455 UniCR Eq.5 CRC quantile formula.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CalibrationCorpusUpdatedPayload {
    pub corpus_size: u32,
    pub new_tau: f64,
    pub alpha: f64,
    pub source: String,
    pub ts_ms: i64,
}

/// E-12 Tier-B — payload for
/// [`SsbActionType::FrameworkProfileSelected`].
///
/// Emitted once per task at scope-planner exit when
/// [`crate::idea_framing::ProjectClass`] resolves to a
/// [`crate::framework_profile::FrameworkProfile`] via the
/// `PROJECT_CLASS_TO_FRAMEWORK` table. Field hashes are
/// blake3-prefix 64 bits (anti-stale-training HARD RULE: NEVER
/// sha2 for non-RFC uses). The row carries hashes, NOT plaintext,
/// so the SSB stays bounded across long-running sessions and the
/// dashboard renders the profile slug via a reverse-lookup table
/// keyed on the hash.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct FrameworkProfileSelectedPayload {
    /// blake3-prefix 64-bit of the canonical profile slug
    /// (e.g. `nextjs15` / `svelte5` / `fastapi-modern` / `hono` /
    /// `generic`). The slug is the manifest basename in
    /// `workflow/archetypes/<name>.toml`.
    pub profile_name_hash: u64,
    /// blake3-prefix of the version pin string loaded from the
    /// manifest. Empty pin string hashes to a stable sentinel.
    pub version_pin_hash: u64,
    /// blake3-prefix of the primary language tag (e.g.
    /// `typescript` / `python` / `rust` / `go` / `generic`).
    pub language_hash: u64,
    /// Producer-side wall-clock UNIX ms via the injected
    /// `ClockProvider`. Never `std::time::SystemTime`.
    pub ts_ms: u64,
}

/// E-12 Tier-B — payload for
/// [`SsbActionType::LegacyConfigStripped`].
///
/// Emitted once per legacy linter config file that the framework
/// adapter declares stale (via
/// `FrameworkAdapter::apply_lint_defaults`) and the C-32
/// `flush_to_disk` path removes. Examples: `.eslintrc.json`,
/// `.prettierrc`, `.flake8`, `setup.cfg [flake8]` section,
/// `pyproject.toml [tool.black]` / `[tool.isort]` sections.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct LegacyConfigStrippedPayload {
    /// blake3-prefix of the removed file path (relative to the
    /// project root).
    pub config_path_hash: u64,
    /// blake3-prefix of the replacement engine name
    /// (`biome` / `ruff` / `clippy_deny` / `generic`).
    pub lint_engine_hash: u64,
    /// Producer-side wall-clock UNIX ms via the injected
    /// `ClockProvider`.
    pub ts_ms: u64,
}

/// D-17 — payload for [`SsbActionType::RefusalRateSnapshot`].
///
/// Emitted once per
/// [`crate::refusal_sla::RefusalRateMonitor::emit_snapshot`] call (and by
/// extension every [`crate::agent_pool::dispatch_refusal_sla_check`]
/// invocation). The row carries the per-window aggregate the dashboard
/// `RefusalRateGauge` renders.
///
/// Field semantics:
///   * `refusal_rate` — `abstain_count / window_total` clamped to
///     `[0.0, 1.0]`. `0.0` when `window_total == 0` (the no-data /
///     fail-safe band).
///   * `window_total` — number of [`crate::conformal_calibration::RefusalDecision`]
///     observations currently inside the sliding window.
///   * `abstain_count` — number of `Abstain` decisions inside the window.
///   * `health_band` — snake-case [`crate::refusal_sla::HealthBand`]
///     discriminant: `below` | `healthy` | `above`.
///   * `ts_ms` — wall-clock UNIX ms at snapshot-emit time.
///
/// Grounding: arXiv:2503.01345 (UniCR honest refusal); docs/specs/guide/
/// real-problem-solving-grounded-2026-05-19.md §4.6 (refusal rate as a
/// first-class quality metric).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RefusalRateSnapshotPayload {
    pub refusal_rate: f64,
    pub window_total: u32,
    pub abstain_count: u32,
    pub health_band: String,
    pub ts_ms: i64,
}

/// D-17 — payload for [`SsbActionType::RefusalSlaBreach`].
///
/// Emitted by [`crate::refusal_sla::RefusalRateMonitor::emit_snapshot`]
/// whenever the computed snapshot's `health_band` is `Below` or
/// `Above`. Carries the offending rate + the closest healthy-band
/// boundary the rate crossed + the snake-case breach kind. The
/// dashboard surfaces this beacon as the last-breach chip on the
/// `RefusalRateGauge`.
///
/// Field semantics:
///   * `rate` — the offending refusal rate `[0.0, 1.0]`.
///   * `threshold` — the boundary the rate crossed (`healthy_band_lo`
///     for `Below`, `healthy_band_hi` for `Above`).
///   * `breach_kind` — `above` | `below` (snake-case).
///   * `ts_ms` — wall-clock UNIX ms at breach-emit time.
///
/// Grounding: arXiv:2503.01345 (UniCR honest refusal — formal Mode E
/// surface the SLA monitors).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RefusalSlaBreachPayload {
    pub rate: f64,
    pub threshold: f64,
    pub breach_kind: String,
    pub ts_ms: i64,
}

/// D-22 — payload for
/// [`SsbActionType::ConformalValidationCompleted`].
///
/// Emitted once per
/// [`crate::benchmarks::conformal_validation_runner::ConformalValidationRunner::run_all`]
/// Ok return. The row is the durable audit trail the dashboard
/// `ConformalValidationPanel` renders to surface whether the UniCR
/// formal distribution-free guarantee (arXiv:2503.01345, T1) still
/// holds against the held-out fixture's in_domain / near_ood / far_ood
/// buckets.
///
/// Field semantics:
///   * `tau` — threshold computed from the calibration corpus at this
///     run's operating point.
///   * `epsilon` — the operating point the calibration corpus was tau'd
///     against; defaults to
///     [`crate::conformal_calibration::DEFAULT_EPSILON`].
///   * `empirical_refusal_rate` — `total_refused / total` measured on the
///     validation fixture.
///   * `gating_verdict` — snake-case [`crate::benchmarks::prmbench_runner::GatingVerdict`]
///     label (`"pass"` iff every bucket landed in-band AND every bucket
///     had `>= min_rows_per_bucket` rows; `"fail"` otherwise).
///   * `fixture_blake3` — 64-hex blake3 of the validation fixture file
///     contents, so dashboards can detect stale runs against a moved
///     fixture.
///   * `total` — number of validation rows the runner consumed.
///   * `ts_ms` — wall-clock UNIX ms at run completion.
///
/// Grounding: arXiv:2503.01345 (UniCR formal guarantee) + Vovk et al.
/// 2005 §2.2 (quantile-based conformal predictor); D-22 plan
/// `docs/thefinalglance/D-plan/D-22_conformal_calibration_validation.md`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ConformalValidationCompletedPayload {
    pub tau: f64,
    pub epsilon: f64,
    pub empirical_refusal_rate: f64,
    pub gating_verdict: String,
    pub fixture_blake3: String,
    pub total: u32,
    pub ts_ms: i64,
}

/// D-20 — payload for [`SsbActionType::ArcBenchmarkCompleted`].
///
/// Emitted once per
/// [`crate::benchmarks::arc_agi2_runner::ARCAGI2Runner::run_all`] Ok
/// return. The row is the durable audit trail the dashboard
/// `ARCBenchPanel` renders to surface whether ELAI's mode-routing +
/// novelty-classifier honesty stack behaves honestly under ARC-AGI-2
/// out-of-distribution claims.
///
/// Field semantics:
///   * `total` — number of fixture rows the runner consumed.
///   * `honestly_evaluated_count` — rows that landed on the
///     `(DiscoveryCandidate class, non-Constructive mode)` pair (the
///     "honest evaluation" predicate).
///   * `gating_verdict` — snake-case
///     [`crate::benchmarks::arc_agi2_runner::ARCVerdict`] label
///     (`honestly_evaluated` | `game_with_memorization` | `refused`).
///   * `fixture_blake3` — 64-hex blake3 of the fixture file bytes
///     (anti-stale audit trail).
///   * `dominant_mode` — snake-case
///     [`crate::grounding_mode::GroundingMode`] wire label for the
///     modal mode in the per-run distribution.
///   * `dominant_novelty_class` — snake-case
///     [`crate::novelty::NoveltyClass`] wire label for the modal
///     novelty class in the per-run distribution.
///   * `ts_ms` — wall-clock UNIX ms at run completion.
///
/// Grounding: TODO_DOC[D-20] — paper-ingestor follow-up to anchor
/// against ARC-AGI-2 dataset description (arXiv:2412.04261, candidate)
/// + ARC Prize 2025 Technical Report (arXiv:2601.10904, candidate).
/// Citations dropped per anti-citation-rot rule until paper-ingestor
/// lands the workflow/papers/ interface map. Plan:
/// `docs/thefinalglance/D-plan/D-20_arc_agi2_ood_probe.md`.
#[allow(non_camel_case_types)]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ARCBenchmarkCompletedPayload {
    pub total: u32,
    pub honestly_evaluated_count: u32,
    pub gating_verdict: String,
    pub fixture_blake3: String,
    pub dominant_mode: String,
    pub dominant_novelty_class: String,
    pub ts_ms: i64,
}

/// C-35 (SCHEMA_VERSION 88) -- shared payload for all 4 tool-gate beacons.
/// Emitted by plugins::mcp::gate (tool_gated, tool_eager, tool_denied) and
/// plugins::mcp::lazy (tool_lazy_resolved). See C-35 plan.
#[allow(non_camel_case_types)]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ToolGateEventPayload {
    pub tool_name: String,
    pub fsm_state: String,
    pub prompt_tokens_saved: i64,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct QuantBackendSelectedPayload {
    pub backend: String,
    pub kv_dtype: String,
    pub compute_capability: i32,
    pub marlin_enabled: bool,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct KvCacheDtypeSoftwareSimPayload {
    pub kv_dtype: String,
    pub throughput_cost_pct: f32,
    pub compute_capability: i32,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct AwqCheckpointBuiltPayload {
    pub model_path: String,
    pub calibration_size: usize,
    pub duration_s: f64,
    pub output_path: String,
}

/// C-39: Emitted per tool call routed via --via-mcp CLI flag.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct CliMcpDispatchPayload {
    pub tool_name: String,
    pub port: u16,
}

/// C-39: Emitted when the CLI auto-launches an MCP co-process.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct CliMcpServerStartedPayload {
    pub port: u16,
}

/// C-39: Emitted when the CLI-launched MCP co-process exits.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct CliMcpServerStoppedPayload {
    pub port: u16,
}

/// E-15: Emitted when Firecracker snapshot-restore exceeds restore_budget_ms.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct FirecrackerRestoreSlowPayload {
    pub agent_id: String,
    pub actual_ms: u64,
    pub budget_ms: u64,
}

/// E-15: Emitted per restore when TCP ISN seed + entropy seeds refreshed.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct SnapshotUniquenessRegeneratedPayload {
    pub agent_id: String,
    pub restore_count: u64,
}

/// E-15: Emitted per LRU or age-based snapshot eviction.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct SnapshotEvictedPayload {
    pub agent_id: String,
    pub reason: String,
    pub snapshot_path: String,
}

/// E-15: Emitted once per agent spawn with the selected backend.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct SandboxBackendSelectedPayload {
    pub agent_id: String,
    pub backend: String,
}

/// E-15: Emitted per baby-step fork-from-snapshot.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct AgentForkedFromSnapshotPayload {
    pub agent_id: String,
    pub baby_step_id: u64,
    pub snapshot_path: String,
}

/// E-15: Emitted when iptables REJECT fires for a non-allowlisted destination.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct ResearcherNetEgressBlockedPayload {
    pub agent_id: String,
    pub destination: String,
    pub policy_action: String,
}

/// E-24: Emitted when blake3 hash of tools/list mismatches the pinned hash.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct McpSchemaPinViolationPayload {
    pub server_url: String,
    pub pinned_hash: String,
    pub observed_hash: String,
    pub pinned_at_ms: i64,
}

/// E-24: Emitted per Trail-of-Bits line-jumping rule match during sanitization.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct McpSanitizationRuleAppliedPayload {
    pub server_url: String,
    pub tool_name: String,
    pub rule_id: String,
    pub replacement_count: u32,
}

/// E-24: Emitted when tools are dropped from tools/list to fit token budget.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct McpBudgetEnforcedPayload {
    pub server_url: String,
    pub dropped_tools: Vec<String>,
    pub kept_tokens: u32,
    pub budget_tokens: u32,
}

/// E-24: Emitted at startup when SSE-only transport is used (deprecated).
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct McpLegacySseTransportPayload {
    pub server_url: String,
    pub deprecation_notice: String,
}

/// E-24: Emitted when the LLM-guard hook denies an MCP tool call.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct McpLlmGuardBlockedPayload {
    pub server_url: String,
    pub tool_name: String,
    pub reason: String,
    pub guard_latency_ms: u32,
}

/// E-24: Emitted at startup when ANTHROPIC_BASE_URL is a known gateway URL.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct AnthropicGatewayDetectedPayload {
    pub base_url: String,
    pub detected_gateway: String,
    pub latency_overhead_ms_estimate: u32,
    pub prompt_cache_broken: bool,
}

/// C-48: Emitted when a MemoryFact is stored with its TrustTier label.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct MemoryTierAssignedPayload {
    pub fact_id: String,
    pub trust_tier: String,
    pub provenance_source: String,
}

/// C-48: Emitted when a retrieval query excludes rows by TierCeiling.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct MemoryQueryFilteredPayload {
    pub query_type: String,
    pub excluded_count: u32,
}

/// C-48: Emitted when a T2/T3 row is blocked from a safety-critical decision context.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct SafetyCriticalBlockedPayload {
    pub fact_id: String,
    pub trust_tier: String,
    pub query_type: String,
}

/// E-30: Emitted when check_argv() returns AllowlistVerdict::Deny.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct ArgvDestructiveDeniedPayload {
    /// Blake3 hex of the argv slice (privacy-safe — no literal args stored).
    pub argv_hash: String,
    /// Human-readable denial reason from the allowlist entry.
    pub reason: String,
    /// Incident citation that motivated this rule.
    pub citation: String,
}

/// E-30: Emitted when check_argv() returns AllowlistVerdict::RequireHumanApproval.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct ArgvApprovalRequiredPayload {
    pub argv_hash: String,
    pub reason: String,
    pub citation: String,
}

/// E-30: Emitted when ensure_session_trust() defers a hook load (CVE-2025-59536).
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct TrustDialogHookDeferredPayload {
    /// Path of the settings file whose hooks were blocked.
    pub settings_path: String,
    /// CVE identifier motivating this check.
    pub cve: String,
}

/// E-30: Emitted when ensure_session_trust() blocks an env-redirect (CVE-2026-21852).
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct TrustDialogEnvDeferredPayload {
    /// Which env var triggered the block (ANTHROPIC_BASE_URL or OPENAI_BASE_URL).
    pub env_var: String,
    pub cve: String,
}

/// E-30: Emitted when an MCP server version is refused by the exact-pin policy.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct McpPinVersionRefusedPayload {
    pub server_name: String,
    /// The version string that was presented (and refused).
    pub presented_version: String,
    /// The pinned exact version required, if registered; empty if not in pin list.
    pub pinned_version: String,
    /// "version_float" | "not_in_pin_list" | "hash_mismatch"
    pub refusal_reason: String,
}

/// E-30: Emitted when auditor and executor share the same lineage group.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct AuditorLineageCollisionPayload {
    pub auditor_agent_id: String,
    pub executor_agent_id: String,
    pub lineage_group: String,
    /// "blocked" (autonomous) | "warned" (interactive)
    pub action_taken: String,
}

/// E-30: Emitted when an untrusted TextSource attempts a memory write.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct UntrustedMemoryWriteBlockedPayload {
    /// TextSource trust label (e.g. "ExternalTool", "UserPrompt").
    pub source_label: String,
    /// Summary of what the write attempt contained (truncated, no PII).
    pub write_summary: String,
}

/// E-30: Emitted when an incident postmortem entry is stored.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct IncidentRecordCreatedPayload {
    pub incident_id: String,
    pub title: String,
    /// ISO-8601 date of the original public incident.
    pub incident_date: String,
    pub source_url: String,
}

/// E-32 — payload for [`SsbActionType::FitmCalibrationApplied`].
///
/// Emitted by `FoundInTheMiddleCalibrator` in server.py on each long-context
/// request (prompt_len > 8K) when `ELAI_FITM_ENABLED=1`.
/// Grounding: arxiv:2406.16008 (Found-in-the-Middle, ACL 2024).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FitmCalibrationAppliedPayload {
    /// Token count of the real prompt that triggered FITM.
    pub prompt_len: u32,
    /// Token count of the dummy doc used for positional-prior subtraction.
    pub dummy_len: u32,
    /// Wall-clock milliseconds spent on the extra forward pass + subtraction.
    pub applied_ms: f64,
}

/// E-32 — payload for [`SsbActionType::DeepModeActivated`].
///
/// Emitted at sidecar boot when `ELAI_DEEP_MODE=1` and VRAM >= 24 GB;
/// indicates the 1M-context variant is active.
/// Grounding: arxiv:2501.15383 (Qwen2.5-1M DCA technical report).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DeepModeActivatedPayload {
    /// HF model ID loaded in deep mode (e.g. "Qwen/Qwen2.5-7B-Instruct-1M").
    pub model_id: String,
    /// Context cap in tokens (64000 on 24 GB box).
    pub ctx_cap: u32,
    /// Total VRAM in GB on the active CUDA device.
    pub vram_gb: f32,
}

/// E-32 — payload for [`SsbActionType::DeepModeEscalationChosen`].
///
/// Emitted by `deep_mode_escalation_hook` in escalation_policy.rs when
/// SessionDag handoff prefers local deep mode over cloud escalation for
/// a cross-file synthesis task (>3 files, no clear retrieval answer).
/// Grounding: arxiv:2407.16833 (Self-Route EMNLP 2024).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DeepModeEscalationChosenPayload {
    /// Human-readable reason for choosing deep mode over cloud.
    pub reason: String,
    /// Number of files referenced that triggered the cross-file synthesis heuristic.
    pub files_referenced: u32,
}

/// E-32 — payload for [`SsbActionType::YarnRefused`].
///
/// Emitted at sidecar boot when `ELAI_USE_YARN=1` is set but refused for
/// a Qwen2.5-Coder model (15-20% quality drop on short prompts).
/// Grounding: vllm issue 18728; arxiv:2601.15300.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct YarnRefusedPayload {
    /// The model name/path that triggered the refusal.
    pub model_name: String,
    /// The refusal reason (cites vllm issue + arxiv).
    pub reason: String,
}

/// B-3 — payload for [`SsbActionType::DepAuditCompleted`].
///
/// Emitted by the B-23 dep-audit orchestrator once per dep-audit run.
/// Carries the snapshot the AntiStalenessDrawer renders in its
/// dep-audit tile. Producer ships in B-23; B-3 reserves the variant so
/// the renderer can subscribe without rendering zero rows.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DepAuditCompletedPayload {
    /// Number of dependencies flagged as out-of-date or vulnerable.
    pub findings: u32,
    /// Number of packages scanned in this run.
    pub packages_scanned: u32,
    /// Wall-clock duration of the audit in milliseconds.
    pub duration_ms: u64,
    /// UNIX millisecond timestamp at which the run completed.
    pub completed_at_ms: i64,
}

/// B-3 — payload for [`SsbActionType::BaselineRunCompleted`].
///
/// Emitted by the B-4 baseline harness once per harness run. Carries
/// the pass/fail summary the AntiStalenessDrawer renders in its
/// baseline tile. Producer ships in B-4; B-3 reserves the variant.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BaselineRunCompletedPayload {
    /// Number of baseline checks executed.
    pub checks_run: u32,
    /// Number of baseline checks that passed.
    pub checks_passed: u32,
    /// Number of baseline checks that failed.
    pub checks_failed: u32,
    /// Wall-clock duration of the harness run in milliseconds.
    pub duration_ms: u64,
    /// UNIX millisecond timestamp at which the run completed.
    pub completed_at_ms: i64,
}

/// B-3 — payload for [`SsbActionType::DeterministicAuditRun`].
///
/// Emitted by the B-15 deterministic-audit gate once per gate
/// invocation. Carries the gate verdict the AntiStalenessDrawer
/// renders in its deterministic-audit tile. Producer ships in B-15;
/// B-3 reserves the variant.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DeterministicAuditRunPayload {
    /// One of "passed" | "failed" | "skipped".
    pub verdict: String,
    /// Number of gates evaluated in this run.
    pub gates_evaluated: u32,
    /// Number of gates that fired (verdict != passed).
    pub gates_fired: u32,
    /// UNIX millisecond timestamp at which the run completed.
    pub completed_at_ms: i64,
}

/// B-3 — payload for [`SsbActionType::RuleTransition`].
///
/// Sibling-stream of the persistent `rule_transitions` SQLite table
/// (rules/src/schema.rs:101). Emitted by
/// `rules::lifecycle::write_transition_log` whenever a rule's
/// lifecycle state changes (PROPOSED -> CANDIDATE -> SHADOW -> ACTIVE
/// -> RETIRED). The RulesDrawer subscribes to this stream so it can
/// render the last-5 transitions inline without polling the rules DB.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RuleTransitionPayload {
    /// The rule's stable identifier.
    pub rule_id: String,
    /// The lifecycle status the rule was in before the transition.
    pub from_status: String,
    /// The lifecycle status the rule moved to.
    pub to_status: String,
    /// Human-readable reason for the transition (gate verdict, manual
    /// approval message, retirement reason, etc.).
    pub reason: String,
    /// UNIX millisecond timestamp at which the transition was recorded.
    pub timestamp_ms: i64,
}

/// B-4 — payload for [`SsbActionType::CanaryDrift`].
///
/// Emitted by the periodic cascade evaluator in `cli::live_cli` (or any
/// other startup-readiness path that runs `check_canary_drift`) whenever
/// a fresh canary embedding is compared against the persisted reference
/// vector. The dashboard's `get_canary_drift` Tauri command surfaces the
/// latest row in SettingsWindow so the operator can see drift trends and
/// trigger a "Recompute" affordance when `similarity < drift_threshold`.
///
/// Field semantics:
///   * `similarity` — cosine similarity of the current canary embedding vs
///     the persisted reference vector, in [-1.0, 1.0]. NaN is impossible
///     because the source vectors are L2-normalized embeddings.
///   * `drift_threshold` — the configured similarity floor below which the
///     pipeline considers the index drifted (default 0.85 per
///     knowledge::startup::DRIFT_THRESHOLD).
///   * `passed` — `similarity >= drift_threshold`. Mirrors
///     `DriftCheckResult::Passed` vs `Failed` so the dashboard does not
///     have to repeat the comparison.
///   * `reference_present` — `false` when no reference vector existed yet
///     at observation time (first-startup `ReferenceGenerated` path);
///     `true` when a real comparison happened.
///   * `ts_ms` — wall-clock UNIX ms at observation time.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CanaryDriftPayload {
    pub similarity: f64,
    pub drift_threshold: f64,
    pub passed: bool,
    pub reference_present: bool,
    pub ts_ms: i64,
}

/// D-15 — payload for [`SsbActionType::NoveltyClassified`].
///
/// Emitted once per [`crate::novelty::emit_novelty_classified`] call
/// (and by extension every [`crate::agent_pool::dispatch_novelty`]
/// invocation). The row is the durable audit trail the dashboard
/// `NoveltyPanel` reads to render the per-template class label, the
/// recent-classifications list, and the per-class distribution chart.
///
/// Field semantics:
///   * `template_id` — hex `TemplateId` of the classified
///     `ReasoningTemplate` (joins against D-1 ArchiveChain rows).
///   * `class` — one of `search | distant_search | discovery_candidate`
///     (snake-case [`crate::novelty::NoveltyClass`] label).
///   * `dkg_match_count` — number of DKG primitives the template body
///     references that exist in the active symbol graph.
///   * `near_duplicate_threshold` — cosine similarity in `[0.0, 1.0]`
///     against the closest archived template body.
///   * `ts_ms` — wall-clock UNIX ms at classify-emit time.
///
/// Grounding:
///   * arXiv:2406.05761 — *LLM Novelty Detection*: three-bucket
///     classification (familiar / recombination / out-of-distribution).
///   * docs/specs/guide/real-problem-solving-grounded-2026-05-19.md
///     §1 Mode C (lines 68-89) + §4.8 (lines 310-328).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct NoveltyClassifiedPayload {
    pub template_id: String,
    pub class: String,
    pub dkg_match_count: u32,
    pub near_duplicate_threshold: f64,
    pub ts_ms: i64,
    /// FINAL TRANCHE B4 / D-15 -- lifecycle stage of the classified
    /// candidate.
    ///
    /// Optional + additive: existing rows pre-dating B4 deserialize with
    /// `None` (rmp_serde `#[serde(default)]` on `Option<T>` returns
    /// `Option::None` for missing fields), preserving wire-form backward
    /// compat against the existing `NoveltyClassified` action_type.
    ///
    /// Per the TRANCHE5_C41 gotcha rule we explicitly do NOT collapse the
    /// 3-arm lifecycle (Queued / Ratified / Rejected) into a new Pattern-2
    /// SsbActionType variant: the arms are SEQUENTIAL lifecycle stages of
    /// a single template (the same candidate transitions Queued -> Ratified
    /// or Queued -> Rejected through D-13 falsifiability follow-up), not
    /// distinct fire-points sharing one envelope. Pattern-2 collapse
    /// requires sibling fire-points; lifecycle requires payload extension.
    /// See workflow/reasoning/FINAL-push-decisions.md DECISION-4.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub candidate_status: Option<DiscoveryCandidateStatus>,
}

/// FINAL TRANCHE B4 / D-15 -- 3-arm lifecycle stage of a discovery candidate.
///
/// Emitted as an optional field on [`NoveltyClassifiedPayload`]; serialized
/// as snake_case ("queued" / "ratified" / "rejected"). NOT a new
/// `SsbActionType` variant per the TRANCHE5_C41 gotcha rule (lifecycle =
/// payload extension, sibling fire-points = Pattern-2 envelope).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DiscoveryCandidateStatus {
    /// Candidate detected by `dispatch_novelty`; awaiting D-13
    /// falsifiability follow-up.
    Queued,
    /// Candidate confirmed by an adversarial cross-family verifier.
    Ratified,
    /// Candidate rejected by the falsifiability path (duplicate, trivial
    /// recombination, or contradicted by held-out checks).
    Rejected,
}

impl DiscoveryCandidateStatus {
    /// Snake-case wire label (matches the `#[serde(rename_all)]` form).
    #[must_use]
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Queued => "queued",
            Self::Ratified => "ratified",
            Self::Rejected => "rejected",
        }
    }
}

/// D-18 — payload for [`SsbActionType::ObservableEffectVerified`].
///
/// Emitted once per
/// [`crate::observable_effect::verify_observable_effect`] call. The row
/// is the durable audit trail the dashboard
/// `ObservableEffectPanel.svelte` renders.
///
/// Field semantics:
///   * `effect_id` — stable per-effect identifier (snake-case ASCII).
///     Used to join successive runs of the same effect against each
///     other for time-dependent confound flagging.
///   * `trigger_kind` — snake-case
///     [`crate::observable_effect::TriggerSpec`] kind label
///     (`test_command | file_presence | http_endpoint_get |
///     file_contents | stdout_contains`).
///   * `observation_kind` — snake-case
///     [`crate::observable_effect::TypedObservation`] kind label
///     (`string_match | file_content | json_shape | http_status |
///     exit_code`).
///   * `sandbox_tier` — snake-case
///     [`crate::observable_effect::SandboxTier`] label
///     (`no_sandbox | process_isolated | container_isolated |
///     network_blocked`).
///   * `outcome` — `pass | fail | skipped`. `Skipped` covers executor
///     errors + mis-authored trigger/observation pairings; it is NOT a
///     product failure.
///   * `confound_flags` — comma-separated list of set
///     [`crate::observable_effect::ConfoundFlags`] labels
///     (`environment_dependent`, `time_dependent`, `order_dependent`,
///     `nondeterministic`). Empty string when no flag is set.
///   * `elapsed_us` — verifier wall-clock microseconds from start to
///     SSB-emit.
///   * `ts_ms` — wall-clock UNIX ms at verify-emit time.
///
/// Grounding: TODO_DOC[D-18]: paper-ingestor follow-up to anchor the
/// typed-effect / sandbox-observation discipline (candidate anchors:
/// Voyager arxiv:2305.16291 §3 skill-with-measurable-effect;
/// Coscientist arxiv:2304.05332 §4 sandbox vs production observation).
/// The orchestrator brief's arXiv:2412.16367 citation was rejected
/// (WebSearch confirmed it is "A Layered Swarm Optimization Method
/// for Fitting Battery Thermal Runaway Models" — unrelated to typed
/// test predicates).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ObservableEffectVerifiedPayload {
    pub effect_id: String,
    pub trigger_kind: String,
    pub observation_kind: String,
    pub sandbox_tier: String,
    pub outcome: String,
    pub confound_flags: String,
    pub elapsed_us: u64,
    pub ts_ms: i64,
}

/// D-18 closure — payload for [`SsbActionType::ObservableEffectDispatched`].
///
/// Emitted once per `task_runner::dispatch_observable_effect_hook`
/// invocation that reaches the layer-8 dispatch arm. The hook is gated
/// on `ELAI_OBSERVABLE_EFFECT_ENABLE`; when the gate is off the row
/// family stays dark.
///
/// Field semantics:
///   * `effect_id` — stable per-effect identifier (snake-case ASCII).
///   * `trigger` — snake-case trigger kind label (matches
///     [`crate::observable_effect::TriggerSpec::kind_str`]).
///   * `context_kind` — one of `sandbox | production` per the
///     Coscientist arxiv:2304.05332 §4 taxonomy.
///   * `ts_ms` — wall-clock UNIX ms at dispatch time.
///
/// Grounding: Voyager arxiv:2305.16291 §3 skill-with-measurable-effect
/// (effect must be observable and re-runnable across sessions);
/// Coscientist arxiv:2304.05332 §4 (sandbox vs production label is
/// required at dispatch time, not inferred post-hoc).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ObservableEffectDispatchedPayload {
    pub effect_id: String,
    pub trigger: String,
    pub context_kind: String,
    pub ts_ms: i64,
}

/// D-18 closure — payload for [`SsbActionType::GoodhartRiskFlag`].
///
/// Emitted once per
/// [`crate::observable_effect::check_goodhart_risk`] flag yielded for
/// an `EffectSet`. Per Eureka arxiv:2310.12931 §3.3 component-variation
/// guard: when one component of an effect is constant across runs
/// (variation < `goodhart_min_component_variation`) the effect is not
/// measuring what it claims and the row fires.
///
/// Field semantics:
///   * `effect_id` — stable per-effect identifier.
///   * `risk_kind` — snake-case risk taxonomy label (e.g.
///     `constant_component`, `degenerate_pairing`, `unobserved_state`).
///   * `evidence` — short free-text describing the observation that
///     triggered the flag (e.g. `"component variation 0.0001 < 0.001 threshold"`).
///   * `ts_ms` — wall-clock UNIX ms at flag time.
///
/// Grounding: Eureka arxiv:2310.12931 §3.3 component-variation
/// invariant (component-wise reward components must vary across runs;
/// constant components flag Goodhart failure modes).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GoodhartRiskFlagPayload {
    pub effect_id: String,
    pub risk_kind: String,
    pub evidence: String,
    pub ts_ms: i64,
}

/// D-18 closure — payload for [`SsbActionType::SandboxObservation`].
///
/// Emitted once per Coscientist-style `SandboxObservation` round-trip
/// (effect verified in an isolated sandbox tier). The dashboard's
/// `ObservableEffectPanel` surfaces these rows alongside the
/// `observable_effect_verified` family so the operator can distinguish
/// sandbox-observed effects from production-observed ones.
///
/// Field semantics:
///   * `effect_id` — stable per-effect identifier.
///   * `tier` — snake-case sandbox-tier label (matches
///     [`crate::observable_effect::SandboxTier::as_str`]; one of
///     `no_sandbox | process_isolated | container_isolated |
///     network_blocked`).
///   * `outcome` — snake-case `confirmed | refuted | inconclusive`
///     (matches [`crate::observable_effect::ObservationOutcome`];
///     DISTINCT from `pass | fail | skipped` on
///     `observable_effect_verified` — see D-18 DECISION-4).
///   * `evidence_count` — number of `(trigger, observation)` evidence
///     points the executor returned during the sandbox round-trip.
///   * `ts_ms` — wall-clock UNIX ms at observation time.
///
/// Grounding: Coscientist arxiv:2304.05332 §4 (sandbox observation
/// distinguishes hypothesis confirmation in a controlled environment
/// from production claim; both phases emit, but with distinct row
/// families and distinct outcome taxonomies).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SandboxObservationPayload {
    pub effect_id: String,
    pub tier: String,
    pub outcome: String,
    pub evidence_count: u32,
    pub ts_ms: i64,
}

/// D-18 closure — payload for
/// [`SsbActionType::ProductionObservationProposal`].
///
/// Emitted once per Coscientist-style `ProductionObservation`
/// proposal. The row records the operator-facing proposal to upgrade a
/// sandbox-observed effect to a production-observed one. Per the
/// Coscientist §4 invariant, ProductionObservation without an
/// `instrument_file` MUST gate on human review.
///
/// Field semantics:
///   * `effect_id` — stable per-effect identifier.
///   * `instrument_file` — relative path of the instrument file that
///     surfaces the observation in production (e.g.
///     `crates/.../instrument.rs`). `None` means the proposal lacks a
///     concrete instrument; `requires_human_review` MUST be `true`.
///   * `requires_human_review` — `true` iff the proposal is not yet
///     safe to auto-promote. Always `true` when `instrument_file` is
///     `None` (paper invariant enforced at producer + verified at
///     contract-test).
///   * `ts_ms` — wall-clock UNIX ms at proposal time.
///
/// Grounding: Coscientist arxiv:2304.05332 §4 (production observation
/// is qualitatively distinct from sandbox observation; the
/// instrument_file is the load-bearing element that licenses
/// automatic promotion).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProductionObservationProposalPayload {
    pub effect_id: String,
    pub instrument_file: Option<String>,
    pub requires_human_review: bool,
    pub ts_ms: i64,
}

/// E-13 — payload for [`SsbActionType::TddCycleStarted`].
///
/// Emitted once per `TddVerifierAgent::run_cycle` invocation. The
/// row anchors a Red->Green->Refactor cycle so per-mutant kill-rate
/// can be attributed back to a specific cycle entry.
///
/// Field semantics:
///   * `task_id_hash` — blake3-prefix u64 of the TDD task identifier
///     (caller-provided). Anti-stale: NEVER sha2.
///   * `ts_ms` — wall-clock UNIX ms at cycle entry.
///
/// Grounding: LLMorpheus arXiv:2404.09952 §2 (LLM-driven mutation
/// testing requires a concrete cycle anchor so kill-rate can be
/// attributed back to a Red->Green transition).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TddCycleStartedPayload {
    pub task_id_hash: u64,
    pub ts_ms: i64,
}

/// E-13 — payload for [`SsbActionType::TddTestFailed`].
///
/// Emitted once per Red-phase test failure observed by the TDD
/// agent driver. The driver may emit multiple rows per cycle when
/// the iteration cap is greater than one.
///
/// Field semantics:
///   * `task_id_hash` — blake3-prefix u64 of the TDD task identifier
///     (matches the `task_id_hash` on the `TddCycleStarted` row).
///   * `test_name_hash` — blake3-prefix u64 of the failing test
///     function name (e.g. `crates::foo::tests::adds_two_correctly`).
///   * `error_kind_hash` — blake3-prefix u64 of the snake-case error
///     classifier (e.g. `assert_failed`, `panic`, `timeout`).
///   * `ts_ms` — wall-clock UNIX ms at failure observation.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TddTestFailedPayload {
    pub task_id_hash: u64,
    pub test_name_hash: u64,
    pub error_kind_hash: u64,
    pub ts_ms: i64,
}

/// E-13 — payload for [`SsbActionType::TddTestPassed`].
///
/// Emitted once per Red->Green flip with optional mutation-score
/// evidence. `mutation_score` is `None` when cargo-mutants did not
/// run during this cycle (the agent does not shell out on every
/// cycle; the caller decides per cost budget).
///
/// Field semantics:
///   * `task_id_hash` — blake3-prefix u64 of the TDD task identifier.
///   * `test_name_hash` — blake3-prefix u64 of the now-passing test
///     function name.
///   * `mutation_score` — optional `f32` in `[0.0, 1.0]` from the
///     cargo-mutants probe. `None` means the probe did not run;
///     the auditor treats `None` as "no signal" rather than fail.
///   * `ts_ms` — wall-clock UNIX ms at flip observation.
///
/// Grounding: ThinkPRM arXiv:2504.16828 §3 (process reward model —
/// the "test flipped" signal is the canonical non-LLM verifier under
/// D-3 aggregation).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TddTestPassedPayload {
    pub task_id_hash: u64,
    pub test_name_hash: u64,
    pub mutation_score: Option<f32>,
    pub ts_ms: i64,
}

/// Tranche B-7 triple (SCHEMA_VERSION 195) -- per-kind discriminator
/// carried by [`TddVerificationEventPayload`]. Pattern-2 (46th
/// application).
///
/// Closed enum; adding a 5th kind requires bumping the schema again.
/// Future deferred arms documented in
/// `workflow/reasoning/B7-triple-tranche-decisions.md`
/// (FuzzPanic / MutantHunterSpawned / IntegrationPropertyCoverage /
/// PropertyGateVacuousRejected) land as future arms on THIS enum, NOT
/// as new top-level variants.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TddVerificationEventKind {
    /// E-13 PropertyGate emitted a passing proptest verdict. Details
    /// carry the property name hash + sample size + shrunk-failure
    /// breadcrumb if any. Vacuous-pass detection is a separate (deferred)
    /// arm.
    PropertyGatePass,
    /// E-13 MutationGate detected a survivor whose blake3 cache key
    /// matches a previously-recorded survivor row (short-circuit hit).
    /// The producing call site is
    /// [`crate::survival_cache::SurvivalCache::is_known_survivor`].
    /// Details carry the mutation_id_hash + status + cached_at_ms.
    MutationGateSurvivorBlock,
    /// E-13 anti-mock heuristic
    /// ([`crate::tdd_verifier::anti_mock::anti_mock_scan`]) fired on a
    /// test-file write. Details carry the hit count + first-3-hit
    /// fingerprint hashes.
    AntiMockBlock,
    /// E-12 Tier-B AssertionOnlyDiff scanner blocked a commit that added
    /// `assert!` / `assert_eq!` / `expect(` lines without any matching
    /// implementation diff in the same change set. Producer wiring is
    /// DEFERRED for the B-7 tranche (substrate-only). Details (when
    /// producer lands): added_assertions + impl_diff_lines (0) +
    /// file_path_hash.
    AssertionOnlyDiffBlock,
}

impl TddVerificationEventKind {
    /// Snake-case wire label.
    #[must_use]
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::PropertyGatePass => "property_gate_pass",
            Self::MutationGateSurvivorBlock => "mutation_gate_survivor_block",
            Self::AntiMockBlock => "anti_mock_block",
            Self::AssertionOnlyDiffBlock => "assertion_only_diff_block",
        }
    }

    /// Inverse of [`Self::as_str`]. Returns `None` on unknown input so the
    /// SSB read path can fail closed.
    #[must_use]
    pub fn from_snake_case(s: &str) -> Option<Self> {
        match s {
            "property_gate_pass" => Some(Self::PropertyGatePass),
            "mutation_gate_survivor_block" => Some(Self::MutationGateSurvivorBlock),
            "anti_mock_block" => Some(Self::AntiMockBlock),
            "assertion_only_diff_block" => Some(Self::AssertionOnlyDiffBlock),
            _ => None,
        }
    }
}

/// Tranche B-7 triple (SCHEMA_VERSION 195) -- payload for
/// [`SsbActionType::TddVerificationEvent`].
///
/// Wire-dark default; emission gated by
/// [`ELAI_E13_TDD_VERIFICATION_EMIT_ENABLE_ENV`]. Pattern-2 single
/// envelope with the 4-arm [`TddVerificationEventKind`] discriminator +
/// free-form `details_json` so future arms can extend without a schema
/// bump.
///
/// Field semantics:
///   * `kind` -- the per-fire-point discriminator.
///   * `task_id_hash` -- blake3-prefix u64 of the TDD task identifier
///     (matches the `task_id_hash` on the originating `TddCycleStarted`
///     row when the event lands inside an active cycle). `0` when the
///     event is emitted out-of-band (e.g. an AssertionOnlyDiff scan on
///     a committed-but-not-currently-cycling change).
///   * `details_json` -- per-kind context. Conventional fields per arm:
///       - property_gate_pass: property_name_hash, sample_size,
///         shrunk_breadcrumb (optional).
///       - mutation_gate_survivor_block: mutation_id_hash, status,
///         cached_at_ms.
///       - anti_mock_block: hit_count, fingerprint_hashes (array, up to
///         3 entries).
///       - assertion_only_diff_block: added_assertions,
///         impl_diff_lines (always 0 for this arm), file_path_hash.
///   * `ts_ms` -- fire-point wall-clock (millis since UNIX_EPOCH).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TddVerificationEventPayload {
    pub kind: TddVerificationEventKind,
    pub task_id_hash: u64,
    pub details_json: serde_json::Value,
    pub ts_ms: i64,
}

/// Tranche B-7 triple -- env var name re-export for callers that prefer
/// to reference the gate via `ssb::ELAI_E13_TDD_VERIFICATION_EMIT_ENABLE_ENV`
/// rather than hard-coding the literal. Wire-dark default: emission
/// silently skipped until the operator sets this to a truthy value.
pub const ELAI_E13_TDD_VERIFICATION_EMIT_ENABLE_ENV: &str =
    "ELAI_E13_TDD_VERIFICATION_EMIT_ENABLE";

/// Tranche B-7 triple -- env var name for the E-50 archetype-selected
/// arm extension. Wire-dark default: emission silently skipped until the
/// operator sets this to a truthy value. Separate gate from the parent
/// `ELAI_SAAS_ARCH_PATTERNS_ENABLE` so the new arm can be rolled out
/// independently.
pub const ELAI_E50_ARCHETYPE_SELECTED_EMIT_ENABLE_ENV: &str =
    "ELAI_E50_ARCHETYPE_SELECTED_EMIT_ENABLE";

/// E-14 — payload for [`SsbActionType::BrowserE2eRunStarted`].
///
/// Emitted once per `PlaywrightRunner::snapshot` call entry. Anchors a
/// per-journey timeline so the dashboard can correlate later
/// `BrowserE2eAssertionFailed` / `BrowserE2eAssertionPassed` /
/// `BrowserE2eSnapshotCaptured` rows back to a specific run.
///
/// Field semantics:
///   * `journey_name_hash` — blake3-prefix u64 of the journey name
///     (snake-case). Anti-stale: NEVER sha2.
///   * `url_hash` — blake3-prefix u64 of the target URL. Hashed (not
///     stored verbatim) so the SSB row is small and operator-grade
///     redaction is automatic for private hosts.
///   * `ts_ms` — wall-clock UNIX ms at run entry.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BrowserE2eRunStartedPayload {
    pub journey_name_hash: u64,
    pub url_hash: u64,
    pub ts_ms: i64,
}

/// E-14 — payload for [`SsbActionType::BrowserE2eAssertionFailed`].
///
/// Emitted once per failing assertion in a journey run. The driver may
/// emit multiple rows per `BrowserE2eRunStarted` when several
/// assertions fail.
///
/// Field semantics:
///   * `journey_name_hash` — matches the corresponding `BrowserE2eRunStarted`
///     row.
///   * `assertion_hash` — blake3-prefix u64 of the assertion label
///     (e.g. `shows-signup-form`).
///   * `error_hash` — blake3-prefix u64 of the snake-case error
///     classifier (e.g. `html_does_not_contain`, `no_selector`).
///   * `ts_ms` — wall-clock UNIX ms at failure observation.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BrowserE2eAssertionFailedPayload {
    pub journey_name_hash: u64,
    pub assertion_hash: u64,
    pub error_hash: u64,
    pub ts_ms: i64,
}

/// E-14 — payload for [`SsbActionType::BrowserE2eAssertionPassed`].
///
/// Emitted once per passing assertion. Pairs 1:1 with
/// `BrowserE2eAssertionFailed` so the dashboard can render pass/fail
/// counts without polling the orchestrator.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BrowserE2eAssertionPassedPayload {
    pub journey_name_hash: u64,
    pub assertion_hash: u64,
    pub ts_ms: i64,
}

/// E-14 — payload for [`SsbActionType::BrowserE2eSnapshotCaptured`].
///
/// Emitted once per Playwright snapshot capture. `snapshot_kind_hash`
/// is a blake3 digest of the `SnapshotKind::as_str()` wire string so
/// the dashboard renders the captured-artifact inventory (html_only /
/// html_and_screenshot / html_and_ax_tree / full) without needing an
/// extra column per artifact type.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BrowserE2eSnapshotCapturedPayload {
    pub journey_name_hash: u64,
    pub snapshot_kind_hash: u64,
    pub ts_ms: i64,
}

/// E-18 — payload for [`SsbActionType::ToolReliabilityGatePassed`].
///
/// Emitted once per Pass verdict from
/// `tool_reliability::ToolReliabilityGate::check`. Composes AFTER the
/// C-47 `tool_call_logged` row so the operator can correlate the gate
/// outcome with the case-base observation.
///
/// Field semantics:
///   * `tool_name_hash` — blake3-prefix u64 of the tool name
///     (snake-case). Anti-stale: NEVER sha2.
///   * `call_shape_hash` — same u64 the C-47 `tool_call_logged` row
///     carries, so the dashboard can join across the two SSB tables.
///   * `ts_ms` — wall-clock UNIX ms at the Pass verdict (NOT at tool
///     dispatch — there's a small window between).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ToolReliabilityGatePassedPayload {
    pub tool_name_hash: u64,
    pub call_shape_hash: u64,
    pub ts_ms: i64,
}

/// E-18 — payload for [`SsbActionType::ToolReliabilityGateRejected`].
///
/// Emitted once per Reject verdict. `reject_reason_hash` is a blake3
/// digest of one of the snake-case labels:
///   * `malformed_args`
///   * `package_ref_rejected`
///   * `retry_budget_exhausted`
///   * `path_allowlist_violation`
///
/// The hash (not the raw label) lands on the SSB row so operators get
/// a stable column type and the dashboard can render the reject-cause
/// histogram without per-cause column proliferation.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ToolReliabilityGateRejectedPayload {
    pub tool_name_hash: u64,
    pub reject_reason_hash: u64,
    pub ts_ms: i64,
}

/// E-18 — payload for [`SsbActionType::ToolCallSpilled`].
///
/// Emitted once per body that exceeded the inline cap and was written
/// to `.elai_cc/.elai_cc/spill/<ts>-<blake3-prefix>.json`. Path is
/// hashed so the SSB row is small and operator-grade redaction is
/// automatic.
///
/// Field semantics:
///   * `tool_name_hash` — blake3-prefix u64 of the tool name.
///   * `spill_path_hash` — blake3-prefix u64 of the absolute spill
///     file path. Anti-stale: NEVER sha2.
///   * `payload_bytes` — u64 byte count of the spilled body. Lets the
///     dashboard render a histogram of spill sizes.
///   * `ts_ms` — wall-clock UNIX ms at spill write completion.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ToolCallSpilledPayload {
    pub tool_name_hash: u64,
    pub spill_path_hash: u64,
    pub payload_bytes: u64,
    pub ts_ms: i64,
}

/// E-18 — payload for [`SsbActionType::ToolCallRetryEscalated`].
///
/// Emitted once per `RetryDecision::Escalate` verdict from
/// `tool_reliability::should_escalate`. Grounding: arxiv:2509.18847
/// Structured Reflection — the Claude-Sonnet-4 >50% recovery vs
/// Qwen3-8B ~20% recovery gap motivates capping retry at 1 (the
/// `DEFAULT_RETRY_CAP` constant in `tool_reliability::retry`).
///
/// Field semantics:
///   * `tool_name_hash` — blake3-prefix u64 of the tool name.
///   * `attempt` — u32 attempt count at escalation time (so attempt=1
///     means "one-shot retry exhausted").
///   * `escalate_to_role_hash` — blake3-prefix u64 of the destination
///     `elai_ipc_types::AgentRole` snake-case label.
///   * `ts_ms` — wall-clock UNIX ms at the escalate verdict.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ToolCallRetryEscalatedPayload {
    pub tool_name_hash: u64,
    pub attempt: u32,
    pub escalate_to_role_hash: u64,
    pub ts_ms: i64,
}

// ---------------------------------------------------------------------------
// E-37 (SCHEMA_VERSION 133): prompt-injection defense payloads.
//
// All 9 payloads carry only blake3-prefix u64 hashes + ts_ms (no raw strings
// in SSB rows — that would defeat the privacy/auditability discipline).
// Paper grounding cited inline; full paper interface files MAY land follow-on
// in workflow/papers/.
// ---------------------------------------------------------------------------

/// E-37 — payload for [`SsbActionType::InjectionAttemptDetected`].
/// Emitted when heuristic OR LLM classifier surfaces a Suspect verdict
/// on an ingest span. Logged but does NOT block.
/// Grounding: arXiv:2507.15219 PromptArmor (off-model classifier design).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct InjectionAttemptDetectedPayload {
    pub source_hash: u64,
    pub classifier_verdict_hash: u64,
    pub ts_ms: i64,
}

/// E-37 — payload for [`SsbActionType::InjectionAttemptBlocked`].
/// Emitted when classifier returns Malicious OR capability_gate rejects
/// argv carrying untrusted provenance.
/// Grounding: arXiv:2503.18813 CaMeL §3.1 (capability denial).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct InjectionAttemptBlockedPayload {
    pub source_hash: u64,
    pub defense_layer_hash: u64,
    pub ts_ms: i64,
}

/// E-37 — payload for [`SsbActionType::SpotlightWrapped`].
/// Emitted once per `wrap_spotlight` envelope.
/// Grounding: arXiv:2403.14720 Spotlight datamarking.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SpotlightWrappedPayload {
    pub input_hash: u64,
    pub marker_hash: u64,
    pub ts_ms: i64,
}

/// E-37 — payload for [`SsbActionType::SpotlightBreached`].
/// Emitted when input contained marker bytes that would break the envelope
/// (forged-wrapper attack attempt).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SpotlightBreachedPayload {
    pub input_hash: u64,
    pub marker_hash: u64,
    pub ts_ms: i64,
}

/// E-37 — payload for [`SsbActionType::ToolDescriptionPinMismatch`].
/// Emitted when per-tool blake3 description hash differs from the pinned
/// value (`safety::tool_description_pin::DescPinStatus::Drift`).
/// Grounding: OWASP MCP03:2025 Tool Poisoning.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ToolDescriptionPinMismatchPayload {
    pub tool_name_hash: u64,
    pub expected_hash: u64,
    pub observed_hash: u64,
    pub ts_ms: i64,
}

/// E-37 — payload for [`SsbActionType::CapabilityGateRejected`].
/// Emitted when destructive tool argv carried at least one untrusted-provenance
/// arg. Grounding: arXiv:2503.18813 CaMeL §3.1.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CapabilityGateRejectedPayload {
    pub tool_name_hash: u64,
    pub origin_trust_hash: u64,
    pub ts_ms: i64,
}

/// E-37 — payload for [`SsbActionType::ProvenanceTagged`].
/// Emitted (sampled) when a value gets a non-trivial Provenance tag.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProvenanceTaggedPayload {
    pub text_source_hash: u64,
    pub trust_level_hash: u64,
    pub ts_ms: i64,
}

/// E-37 — payload for [`SsbActionType::PromptArmorVerdict`].
/// Emitted on every classify_injection call (verdict-rate observability).
/// Grounding: arXiv:2507.15219 PromptArmor + arXiv:2502.06703 cheap monitors.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PromptArmorVerdictPayload {
    pub verdict_hash: u64,
    pub reason_hash: u64,
    pub ts_ms: i64,
}

/// E-37 — payload for [`SsbActionType::InjectionFixtureReplayed`].
/// Emitted per AgentDojo / EchoLeak / CometJacking fixture replay.
/// RUNPOD-PENDING for live model runs; the shape ships now so the fixture
/// harness can land additively later.
/// Grounding: arXiv:2406.13352 AgentDojo + CVE-2025-32711 EchoLeak.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct InjectionFixtureReplayedPayload {
    pub fixture_name_hash: u64,
    pub defenses_passed: u32,
    pub ts_ms: i64,
}

// ---------------------------------------------------------------------------
// E-40 (SCHEMA_VERSION 134): orchestrator-rooted symbol-graph payloads.
//
// All 3 payloads carry only blake3-prefix u64 hashes + counts + ts_ms
// (no raw strings — same privacy/auditability discipline as E-37 above).
// Paper grounding deferred per workflow/sessions/s-25120/mistakes.md
// cluster #6. See workflow/reasoning/E-40-decisions.md.
// ---------------------------------------------------------------------------

/// E-40 — payload for [`SsbActionType::SymbolGraphQueried`].
/// Emitted once per `find_defs` / `find_refs` / `find_callers` /
/// `find_callees` invocation from CLI `scip_cmd` dispatch or the Tauri
/// `symbol_graph_query` command. `query_kind_hash` distinguishes the
/// four call shapes; `result_count` is the unbounded match-count
/// returned to the caller.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SymbolGraphQueriedPayload {
    pub query_kind_hash: u64,
    pub symbol_hash: u64,
    pub result_count: u32,
    pub ts_ms: i64,
}

/// E-40 — payload for [`SsbActionType::SymbolGraphCacheLoaded`].
/// Emitted once per `SymbolGraphCache::load` mount of
/// `.claude/tools/elai_graph.json` (or sibling path supplied via CLI).
/// `node_count` lets the dashboard surface "graph has N nodes" without
/// re-loading the underlying JSON.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SymbolGraphCacheLoadedPayload {
    pub graph_path_hash: u64,
    pub node_count: u32,
    pub ts_ms: i64,
}

/// E-40 — payload for [`SsbActionType::LspOracleConsulted`].
/// Emitted once per `lsp_oracle::oracle_lookup` invocation.
/// `oracle_kind_hash` is the blake3-prefix of the OracleKind snake-case
/// label (rust / python / typescript / go). On-demand-only discipline —
/// >5 invocations/session is treated as a bug per E-40-decisions.md.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LspOracleConsultedPayload {
    pub language_hash: u64,
    pub symbol_hash: u64,
    pub oracle_kind_hash: u64,
    pub ts_ms: i64,
}

// ---------------------------------------------------------------------------
// E-44 — Durability layer payload structs (SCHEMA_VERSION 136)
// ---------------------------------------------------------------------------

/// E-44 — payload for [`SsbActionType::CheckpointWritten`].
/// Emitted once per `CheckpointManager::write` success.
/// `session_id_hash` is the first u64 of the blake3 of the session id.
/// `file_count` mirrors the on-disk file count (4 msgpack + 1 ASCII
/// timestamp = 5 per `checkpoint.rs:23-29`).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CheckpointWrittenPayload {
    pub session_id_hash: u64,
    pub schema_version: u32,
    pub file_count: u32,
    pub ts_ms: u64,
}

/// E-44 — payload for [`SsbActionType::CheckpointResumed`].
/// Emitted once per successful resume path (load_latest -> apply).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CheckpointResumedPayload {
    pub session_id_hash: u64,
    pub schema_version: u32,
    pub ts_ms: u64,
}

/// E-44 — payload for [`SsbActionType::CheckpointMigrated`].
/// Emitted once per
/// `durability::checkpoint_migrator::migrate_checkpoint` invocation
/// that bridged at least one schema-version boundary.
/// `transform_count` is the number of versioned transforms that ran.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CheckpointMigratedPayload {
    pub from_version: u32,
    pub to_version: u32,
    pub transform_count: u32,
    pub ts_ms: u64,
}

/// E-44 — payload for [`SsbActionType::IdempotentDispatchSkipped`].
/// Emitted once per `IdempotentDispatcher` cache hit.
/// `call_shape_hash` is the dedup key (deterministic blake3-prefix
/// over method tag + canonical request body); `cached_result_hash` is
/// the blake3-prefix of the cached result for diagnostic correlation.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct IdempotentDispatchSkippedPayload {
    pub call_shape_hash: u64,
    pub cached_result_hash: u64,
    pub ts_ms: u64,
}

/// E-44 — payload for [`SsbActionType::DurabilityLockBlocked`].
/// Emitted once per `CompactLock::try_acquire` call that returned
/// `CompactLockError::Contended`.
/// `lock_path_hash` is the blake3-prefix of the lock file path;
/// `holder_hash` is a blake3-prefix of the contending holder identity
/// when available (0 when unknown).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DurabilityLockBlockedPayload {
    pub lock_path_hash: u64,
    pub holder_hash: u64,
    pub ts_ms: u64,
}

/// E-45 — payload for [`SsbActionType::CrossLangDriftDetected`].
/// Emitted once per `cross_lang_drift::DriftDetector::diff_pair`
/// invocation that returned `>= 1` `DriftItem`.
/// `pair_kind_hash` is the blake3-prefix of the pair name (e.g.
/// "sidecar_orchestrator_schemas"); `severity_max` is the worst
/// severity across the drift items (1=info, 2=warning, 3=critical).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CrossLangDriftDetectedPayload {
    pub pair_kind_hash: u64,
    pub drift_count: u32,
    pub severity_max: u8,
    pub ts_ms: u64,
}

/// E-45 — payload for [`SsbActionType::CrossLangPairScanned`].
/// Emitted once per `cross_lang_drift::DriftDetector::detect_pairs`
/// repo walk, regardless of drift outcome.
/// `repo_root_hash` is the blake3-prefix of the canonical repo root
/// path; `pair_count` is the number of pairs evaluated from the
/// `cross-lang-pairs.toml` registry.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CrossLangPairScannedPayload {
    pub repo_root_hash: u64,
    pub pair_count: u32,
    pub ts_ms: u64,
}

/// E-45 — payload for [`SsbActionType::SchemaVersionMismatch`].
/// Emitted once when the orchestrator detects a mismatch between
/// the Rust-side `SCHEMA_VERSION` and a sidecar-reported wire schema
/// version. Both fields are the literal monotonic integer values.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SchemaVersionMismatchPayload {
    pub rust_schema: u32,
    pub python_schema: u32,
    pub ts_ms: u64,
}

// ---------------------------------------------------------------------------
// E-49 payloads (SCHEMA_VERSION 138) — coding-agent metric suite (scope-lean).
// All metric scalars are encoded as `*_x10000` integers (literal pass_at_1 *
// 10_000, rounded) so the wire shape stays integer-only for stable
// blake3-hashable msgpack across runs.
// ---------------------------------------------------------------------------

/// E-49 — payload for [`SsbActionType::CodingAgentMetricsCaptured`].
/// Emitted once per `coding_metrics::MetricsCollector::capture_metrics`.
/// `session_id_hash` is the blake3-prefix of the session id.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CodingAgentMetricsCapturedPayload {
    pub session_id_hash: u64,
    pub pass_at_1_x10000: u32,
    pub refine_rate_x10000: u32,
    pub cost_per_task_cents: Option<u32>,
    pub ts_ms: u64,
}

/// E-49 — payload for [`SsbActionType::SweBenchProRunStarted`].
/// Emitted once when `coding_metrics::run_swebench_pro` begins a live
/// run. Live transport is RUNPOD-PENDING; payload shape is stable.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SweBenchProRunStartedPayload {
    pub task_pool_version_hash: u64,
    pub model_id_hash: u64,
    pub max_tasks: Option<u32>,
    pub ts_ms: u64,
}

/// E-49 — payload for [`SsbActionType::SweBenchProRunCompleted`].
/// Emitted once at the end of a SWE-Bench Pro live run.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SweBenchProRunCompletedPayload {
    pub task_pool_version_hash: u64,
    pub model_id_hash: u64,
    pub tasks_attempted: u32,
    pub tasks_solved: u32,
    pub pass_at_1_x10000: u32,
    pub ts_ms: u64,
}

/// E-49 — payload for [`SsbActionType::RefineBenchScoreEmitted`].
/// Emitted once per RefineBench scoring pass.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RefineBenchScoreEmittedPayload {
    pub suite_version_hash: u64,
    pub model_id_hash: u64,
    pub refine_rate_x10000: u32,
    pub ts_ms: u64,
}

/// E-49 — payload for [`SsbActionType::MetricsAggregationRun`].
/// Emitted once per `coding_metrics::aggregate_session_metrics` call.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MetricsAggregationRunPayload {
    pub session_count: u32,
    pub pass_at_1_avg_x10000: u32,
    pub refine_rate_avg_x10000: u32,
    pub ts_ms: u64,
}

// ---------------------------------------------------------------------------
// E-52 Tier-B payloads (SCHEMA_VERSION 139) — LSP-as-verifier diagnostic
// probe + SCIP refresh beacons. Composed with the Tier-A envelope at
// `crates/knowledge/src/diagnostic.rs` (DiagnosticSeverity + ProbeKind +
// Diagnostic + DiagnosticProbeOutcome). All wire fields stay integer /
// blake3-prefix scalars so the msgpack hash is deterministic.
// ---------------------------------------------------------------------------

/// E-52 Tier-B — payload for [`SsbActionType::DiagnosticProbeDispatched`].
/// Emitted once per language-grouped PostToolUse probe dispatch. The
/// `probe_kind_hash` is the blake3-prefix of [`ProbeKind::as_str`]
/// (rust_analyzer / pyright / tsc_no_emit / svelte_check / generic_lsp);
/// `language_hash` is the blake3-prefix of the language slug
/// (rust / python / typescript / svelte).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DiagnosticProbeDispatchedPayload {
    pub probe_kind_hash: u64,
    pub language_hash: u64,
    pub file_count: u32,
    pub ts_ms: u64,
}

/// E-52 Tier-B — payload for [`SsbActionType::DiagnosticProbeCompleted`].
/// Emitted once per probe completion regardless of outcome. The
/// `run_status_hash` is the blake3-prefix of the sibling
/// `DiagnosticProbeRunStatus` snake-case label (completed / timed_out /
/// tool_missing) — added for the DRIFT-14 closure so Timeout and
/// ToolMissing surface without breaking the locked Tier-A envelope.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DiagnosticProbeCompletedPayload {
    pub probe_kind_hash: u64,
    pub run_status_hash: u64,
    pub error_count: u32,
    pub warning_count: u32,
    pub duration_ms: u32,
    pub ts_ms: u64,
}

/// E-52 Tier-B — payload for [`SsbActionType::DiagnosticProbeBlock`].
/// Emitted when error-severity diagnostics blocked the EXECUTING ->
/// AUDITING transition. `first_path_hash` is the blake3-prefix of the
/// first diagnostic's file path (privacy-preserving — the full path is
/// never written to SSB; the EvidenceGate carries the full list out-of-
/// band).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DiagnosticProbeBlockPayload {
    pub probe_kind_hash: u64,
    pub error_count: u32,
    pub first_path_hash: u64,
    pub ts_ms: u64,
}

/// E-52 Tier-B — payload for [`SsbActionType::ScipRefreshCompleted`].
/// Emitted once per `.claude/tools/elai.map` incremental SCIP merge
/// invocation (driven by `.claude/tools/scip_refresh.py` or a future
/// in-tree refresher). `shard_count` is the number of per-crate SCIP
/// shards merged; `crate_count` is the number of distinct crates the
/// refresh touched.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ScipRefreshCompletedPayload {
    pub shard_count: u32,
    pub crate_count: u32,
    pub duration_ms: u32,
    pub ts_ms: u64,
}

// ---------------------------------------------------------------------------
// E-53 payload (SCHEMA_VERSION 140) — formal-verify pipeline boundary beacon.
// Single atomic +1 bump per the C-52 DafnyVerifyCompleted precedent (the plan
// originally proposed 13 variants; lean-scope rebase per
// workflow/sessions/s-25120/completion-drive/exploration/E-53-integration.md
// consolidates to one). Producer:
// `crate::formal_verify::orchestrator::run`. Gemma-4 BLOCKED per memory
// `feedback_no_gemma4` (substrate inheritance from C-52
// `dafny_verify::GEMMA4_BLOCKED`). Live Z3 / Kani / Verus dispatch is
// RUNPOD-PENDING; Dafny dispatch reuses C-52 `invoke_dafny_cli`.
// ---------------------------------------------------------------------------

/// E-53 — payload for [`SsbActionType::FormalVerifierCompleted`].
///
/// Emitted once per `crate::formal_verify::orchestrator::run`
/// pipeline boundary regardless of outcome. Hashed identifiers
/// (no plaintext) so the wire format is privacy-preserving and
/// stable across sessions.
///
/// Field semantics:
///   * `verifier_kind_hash` — blake3 prefix (first 8 bytes as
///     `u64`) of the [`crate::formal_verify::VerifierKind::as_str`]
///     wire tag (`z3_smt` | `dafny` | `kani` | `verus`).
///   * `outcome_hash` — blake3 prefix of the
///     [`crate::formal_verify::VerifierOutcome::as_str`] wire tag
///     (`verified` | `refuted` | `inapplicable` | `runpod_pending`).
///     The aggregator MUST NOT count `inapplicable` or
///     `runpod_pending` toward consensus per the L3-lock comment on
///     [`crate::verification_result::VerifierKind`].
///   * `csq_class_hash` — blake3 prefix of the
///     [`crate::csq::CounterfactualClass::as_str`] gating decision
///     (`counterfactual_risky` | `counterfactual_unknown` |
///     `counterfactual_safe`).
///   * `contract_emitted` — whether the C-50 structural-contract
///     emission step ran before dispatch. Always `true` in the lean
///     E-53 ship (the emission helper is infallible); future
///     extensions may surface emission failure.
///   * `duration_ms` — wall-clock pipeline duration. Producer is
///     responsible for using the injected `ClockProvider` (never
///     `std::time::SystemTime`).
///   * `ts_ms` — producer-side wall-clock timestamp via the injected
///     `ClockProvider`. Never `std::time::SystemTime`.
///
/// Grounding: 3 GROUNDED inline citations live on the
/// `crate::formal_verify` mod docstring; 11 unverified arXiv IDs from
/// the E-53 plan reads block are DEFERRED to
/// workflow/sessions/s-25120/completion-drive/mistakes.md cite-
/// incident #11 (E-53-INGEST-pending) per cluster cite-discipline.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct FormalVerifierCompletedPayload {
    pub verifier_kind_hash: u64,
    pub outcome_hash: u64,
    pub csq_class_hash: u64,
    pub contract_emitted: bool,
    pub duration_ms: u32,
    pub ts_ms: u64,
}

// ---------------------------------------------------------------------------
// E-35 payloads (SCHEMA_VERSION 141) — structured-output substrate beacons.
// 10 atomic payloads for XGrammar / INSTRUCTOR / CodeAct / SEARCH/REPLACE /
// cloud strict-schema / engine-refusal beacons. All hashed identifiers
// (no plaintext) for stable, privacy-preserving wire format. Producers:
// (Python sidecar — RUNPOD-PENDING) + (Rust codeact_sandbox::execute_code_act)
// + (CloudGroundingClient::with_strict_schema) + (CLI subverb
// `elai structured-output ...`). AgentRole::Runner addition DEFERRED per
// E41_AGENTROLE_FROZEN_AT_17 (230 match-arm cascade). See
// workflow/reasoning/E-35-decisions.md.
// ---------------------------------------------------------------------------

/// E-35 — payload for [`SsbActionType::XgrammarMaskApplied`].
/// Emitted once per sidecar XGrammar mask application.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct XgrammarMaskAppliedPayload {
    pub schema_id_hash: u64,
    pub schema_hash_hash: u64,
    pub mask_applied_count: u32,
    pub generation_ms: u32,
    pub cache_hit: bool,
    pub ts_ms: u64,
}

/// E-35 — payload for [`SsbActionType::InstructorModeActivated`].
/// Emitted once per INSTRUCTOR-mode dispatch (reasoning roles per CRANE:
/// Auditor / Planner / Decomposer / IdeaFramer).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct InstructorModeActivatedPayload {
    pub role_hash: u64,
    pub block_hash: u64,
    pub ts_ms: u64,
}

/// E-35 — payload for [`SsbActionType::GrammarModeActivated`].
/// Emitted once per GRAMMAR-mode dispatch (low-reasoning enum/header blocks:
/// FailureTaxonomy / BabyStepStatus / ReasoningTemplate type-tag).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct GrammarModeActivatedPayload {
    pub role_hash: u64,
    pub block_hash: u64,
    pub ts_ms: u64,
}

/// E-35 — payload for [`SsbActionType::CodeactActionExecuted`].
/// Emitted once per `codeact_sandbox::execute_code_act` dispatch regardless
/// of outcome. AgentRole::Runner addition DEFERRED — CodeAct routes through
/// the existing role + RuntimeBackend composition per
/// E41_AGENTROLE_FROZEN_AT_17.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct CodeactActionExecutedPayload {
    pub action_hash: u64,
    pub line_count: u32,
    pub outcome_hash: u64,
    pub duration_ms: u32,
    pub ts_ms: u64,
}

/// E-35 — payload for [`SsbActionType::SearchReplaceTier1Hit`].
/// Emitted on successful `SearchReplaceMatcher::apply` (custom non-git
/// ELAI fences `<<< ELAI_SEARCH` / `=== ELAI_REPLACE` / `>>> ELAI_END`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct SearchReplaceTier1HitPayload {
    pub file_path_hash: u64,
    pub edit_count: u32,
    pub ts_ms: u64,
}

/// E-35 — payload for [`SsbActionType::SearchReplaceCollisionFallback`].
/// Emitted when `DelimiterCollisionGuard` flags ELAI fence text in the
/// source buffer; matcher falls back to whole-file rewrite tier.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct SearchReplaceCollisionFallbackPayload {
    pub file_path_hash: u64,
    pub collision_token_hash: u64,
    pub ts_ms: u64,
}

/// E-35 — payload for [`SsbActionType::CloudStrictSchemaApplied`].
/// Emitted once per `CloudGroundingClient::with_strict_schema` activation.
/// Anthropic / OpenAI / Gemini provider-strict body propagation lives at
/// the call site; header propagation through AnthropicClient is
/// RUNPOD-PENDING (provider-crate extension).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct CloudStrictSchemaAppliedPayload {
    pub provider_hash: u64,
    pub schema_id_hash: u64,
    pub ts_ms: u64,
}

/// E-35 — payload for [`SsbActionType::GrammarCacheLoaded`].
/// Emitted once on sidecar boot after `compile_all_schemas_at_boot` +
/// `warm_grammar_cache_from_disk` complete. Producer is the sidecar
/// (RUNPOD-PENDING); host-side emitter is the CLI smoke + contract test.
/// Expected `compiled_count` for full warmup is 64 (8 AgentRole values *
/// 8 BabyStepSpec blocks).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct GrammarCacheLoadedPayload {
    pub compiled_count: u32,
    pub cache_hits: u32,
    pub cache_misses: u32,
    pub total_compile_ms: u32,
    pub ts_ms: u64,
}

/// E-35 — payload for [`SsbActionType::StructuredEngineRejected`].
/// Emitted when caller requests a structured-output engine
/// (lm-format-enforcer) that is EXPLICITLY rejected in favour of XGrammar.
/// Refuse loud; never silent fallback.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct StructuredEngineRejectedPayload {
    pub engine_hash: u64,
    pub reason_hash: u64,
    pub ts_ms: u64,
}

/// E-35 — payload for [`SsbActionType::LegacyJsonObjectRefused`].
/// Emitted when caller passes legacy vendor `json_object` mode through
/// CloudGroundingClient; the call is refused with a typed error rather
/// than silently downgrading to free-form JSON.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct LegacyJsonObjectRefusedPayload {
    pub provider_hash: u64,
    pub ts_ms: u64,
}

// ---------------------------------------------------------------------------
// E-6 (SCHEMA_VERSION 148) — payload structs for MAST-grounded
// multi-agent failure-mode defenses.
//
// Four payloads (AuditMastTagged + IsoBudgetMeasured + PersonaReanchored
// + CrossFamilyUnavailable) per workflow/reasoning/E-6-decisions.md
// DECISION-5. Citation grounding: arXiv:2503.13657 (Cemri et al. 2025,
// MAST). Original plan cite 2406.12624 ID_COLLISION (per DECISION-1
// + gotchas.md E6_CITATION_2406_12624_ID_COLLISION).
// ---------------------------------------------------------------------------

/// E-6 — count of failures attributed to one MAST mode (wire-stable
/// tag) over a measurement window. Used by [`IsoBudgetMeasuredPayload`]
/// to ship per-mode histograms without coupling the dashboard to
/// `MastMode` (which would force the dashboard crate to import the
/// orchestrator's mast module).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MastModeCount {
    /// Wire-stable MAST tag (`fm_1_1_disobey_task_spec`, etc).
    pub mast_mode: String,
    /// Failure count over the measurement window. `u32` is sufficient
    /// (the iso-budget harness caps the per-MAST-mode rate at the
    /// fixture-size budget).
    pub count: u32,
}

/// E-6 — payload for [`SsbActionType::AuditMastTagged`].
///
/// Emitted by per_level_audit_dispatch (E-6.b follow-up wiring) when an
/// auditor verdict is BLOCK + the finding category maps to a MAST mode
/// via `finding_to_mast_mode`. Producer also emits the originating
/// `per_level_audit_verdict` row; `verdict_seq` joins them.
///
/// ENV gate: `ELAI_FAILURE_TRACKER_ENABLE=1` (wire-dark default off).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AuditMastTaggedPayload {
    /// SSB `seq` of the originating PerLevelAuditVerdict row.
    pub verdict_seq: i64,
    /// Wire-stable MAST mode tag
    /// (`fm_1_1_disobey_task_spec` ... `fm_3_3_incorrect_verification`).
    /// Parse via `elai_orchestrator::mast::MastMode::from_sql_str`.
    pub mast_mode: String,
    /// Confidence band of the attribution: `"low"` / `"medium"` /
    /// `"high"`. Currently emitted as `"medium"` from the deterministic
    /// `finding_to_mast_mode` predicate; future learned classifier will
    /// emit calibrated bands.
    pub confidence_band: String,
    /// Agent role that produced the original finding (snake_case
    /// AgentRole wire tag).
    pub agent_role: String,
    /// Wall-clock millis since UNIX_EPOCH.
    pub ts_ms: i64,
}

/// E-6 — payload for [`SsbActionType::IsoBudgetMeasured`].
///
/// One row per `iso_budget_harness::run_ablation` invocation. Two-row
/// ablation result is summarised here; per-row token + MAST-mode counts
/// are aggregated into one `mast_mode_counts` vec on this payload so the
/// dashboard does not need to merge two SSB rows.
///
/// H-40 extends this payload with `unique_catch_rate`, `decorrelation_score`,
/// and `insufficient_contribution` fields. All three are `#[serde(default)]`
/// so existing rows without them deserialize cleanly (RISK-2 mitigation).
///
/// Grounding: arXiv:2604.02460 (single-agent beats MAS at iso-budget) +
/// arXiv:2507.02778 ("Wait" reflection token reduces blind-spot 89.3%) +
/// arXiv:2502.00674 (Self-MoA, heterogeneous mixing loses 6.6%) +
/// arXiv:2506.18203 (Weaver: decorrelated weak verifiers close the
/// generation-verification gap; the keep/cut metric is unique-catch not lift).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct IsoBudgetMeasuredPayload {
    /// Path to the task-fixture TOML used.
    pub task_fixture: String,
    /// Thinking-token budget applied to both rows.
    pub budget_tokens: u32,
    /// Full 8-role pipeline pass rate in `[0.0, 1.0]`.
    pub full_pipeline_pass_rate: f64,
    /// Single-agent-with-Wait baseline pass rate in `[0.0, 1.0]`.
    pub single_agent_wait_pass_rate: f64,
    /// `true` iff `full_pipeline_pass_rate - single_agent_wait_pass_rate`
    /// is NOT >= 1 stddev of the fixture's seeds. Surfaced on the
    /// dashboard so the user sees the architectural cost. Kept for
    /// backward compatibility; `insufficient_contribution` is the gate.
    pub insufficient_lift: bool,
    /// `true` iff this report is a deterministic stub (host-side scaffold
    /// returned without a live multi-agent run). Always `true` until the
    /// RUNPOD-PENDING live wiring closes.
    pub runpod_pending: bool,
    /// Aggregated per-MAST-mode failure counts across both rows.
    pub mast_mode_counts: Vec<MastModeCount>,
    // --- H-40 decorrelation fields (serde default for backward compat) ---
    /// H-40: per-role unique-catch rate (string-keyed AgentRole wire tag).
    /// Fraction of ensemble failures that each role alone got right.
    /// Default: empty map (stub/pre-H-40 rows).
    #[serde(default)]
    pub unique_catch_rate: std::collections::BTreeMap<String, f64>,
    /// H-40: per-role decorrelation score (string-keyed AgentRole wire tag).
    /// 1 - mean pairwise error-correlation conditional on ground truth.
    /// Default: empty map (stub/pre-H-40 rows).
    #[serde(default)]
    pub decorrelation_score: std::collections::BTreeMap<String, f64>,
    /// H-40: per-role prunable predicate. Key is the snake_case AgentRole wire
    /// tag; value is `true` iff that role is prunable (`low_lift AND
    /// low_unique_catch_for_THAT_role`). A decorrelated low-lift role has
    /// `false` and is RETAINED (RD-2). Callers MUST check `runpod_pending`
    /// before acting. Default: empty map (conservative for pre-H-40 rows and
    /// stub runs — unknown per-role prunable state).
    #[serde(default)]
    pub insufficient_contribution: std::collections::BTreeMap<String, bool>,
}

/// E-6 — payload for [`SsbActionType::PersonaReanchored`].
///
/// Emitted by `persona_anchor::emit_ssb_row` when an anchor fires (or
/// when the caller wants to log a `no_trigger` tick for calibration data).
/// Cosine threshold default 0.55 UNCONFIRMED — calibration sweep
/// RUNPOD-PENDING per plan acceptance criterion #5.
///
/// Grounding: arXiv:2511.09710 Echoing + arXiv:2412.00804 Identity Drift +
/// arXiv:2601.04170 Agent Drift.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PersonaReanchoredPayload {
    /// AgentRole snake_case wire tag for the agent the anchor applied to.
    pub agent_role: String,
    /// Turn count at the time the anchor fired.
    pub turn_count: u32,
    /// Reason the anchor fired (`turn_counter` / `cosine_drift` /
    /// `no_trigger`).
    pub reason: String,
    /// Cosine similarity in `[-1.0, 1.0]` between role-prompt and current
    /// output embedding. `None` when no EmbedProvider was injected (the
    /// anchor degraded to turn-counter-only).
    pub cosine_similarity: Option<f64>,
    /// `true` iff an EmbedProvider was available AND the embedding
    /// computation succeeded. Surfaces the degraded mode on the dashboard.
    pub embedding_available: bool,
}

/// E-6 — payload for [`SsbActionType::CrossFamilyUnavailable`].
///
/// Refusal SSB row emitted when [`crate::adversary::route_cross_family`]
/// returns `Err(SameFamilyPairing)` AND the phase requires adversary per
/// [`crate::phase_kind::PhaseKind::requires_cross_family_adversary`].
/// The orchestrator MUST emit this row before refusing AUDITING ->
/// MEMORIZING — so the dashboard surfaces the structural blocker
/// (per workflow/reasoning/E-6-decisions.md DECISION-7).
///
/// Grounding: arXiv:2507.02778 — 64.5% same-model blind-spot rate makes
/// cross-family auxiliary signal the structural fix.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CrossFamilyUnavailablePayload {
    /// Executor's model family at the refusal site (snake_case
    /// `ModelFamily::as_str`).
    pub executor_family: String,
    /// PhaseKind snake_case tag that requested adversary.
    pub phase_kind: String,
    /// Claim id the refusal applies to.
    pub claim_id: String,
    /// Whether `ELAI_ALLOW_CLOUD_ADVERSARY` was set at the refusal site.
    /// `false` indicates the operator could enable the cloud route to
    /// unblock; `true` indicates cross-family is genuinely unavailable
    /// (e.g. executor is already on cloud Anthropic).
    pub cloud_allowed: bool,
    /// Wall-clock millis since UNIX_EPOCH.
    pub ts_ms: i64,
}

// ---------------------------------------------------------------------------
// E-7 (SCHEMA_VERSION 149): darwinian self-improvement substrate
// ---------------------------------------------------------------------------
//
// Paper grounding (canonical re-anchor per workflow/reasoning/E-7-decisions.md):
//   - arXiv:2510.04399 (Statistical Limits of Self-Improving Agents,
//     Wang/Dorchen/Jin 2025-10-05 rev 2026-02-01) — PAC-theoretic
//     foundation for the Two-Gate guardrail (margin tau AND capacity
//     K_m). NOT AlphaEvolve. Five modification axes (Theorem 11/12):
//     Algorithmic, Representational, Architectural, Substrate,
//     Metacognitive — all reduce to the same capacity-supremum boundary.
//     Interface map:
//     workflow/papers/statistical-limits-self-improving-interface.md.
//   - arXiv:2305.10718 (Discounted Thompson Sampling, Qi/Wang/Zhu) —
//     non-stationary bandit; darwinian linkage is per-cell arm reward
//     distributions shifting as population evolves.
//   - arXiv:2506.13131 (AlphaEvolve, DeepMind 2025) — MAP-Elites +
//     island-model hybrid; primary darwinian-coding-agent anchor.
//   - arXiv:2507.05619 (EST) — invariance probe.
//   - arXiv:2507.19457 (GEPA, ICLR 2026 Oral) — background-only.
//   - OpenEvolve community implementations — background-only.
//   - Krakovna spec-gaming spreadsheet — trap-task pattern seeds.
// REMOVED grounding: arXiv:2602.12413 (wrong paper; see DECISION-3).
//
// Substrate scope: SINGLE-VARIANT SsbActionType +
// DarwinianGenerationCompletedPayload with DarwinianPhase discriminator
// (Pattern-2). FsmSubstate::DarwinianGenerationEvaluating avoids
// collision with FsmState::Auditing top-level per DECISION-8. 8 new
// modules behind cfg(feature = "darwinian-evolve") +
// ELAI_DARWINIAN_LOOP_ENABLE env gate. Live evolution loops
// RUNPOD-PENDING. TripwireKind NOT touched (DECISION-7). bai.rs NOT
// touched (DECISION-13). Ed25519-dalek NOT added (DECISION-15
// anti-slopsquatting cargo info verification deferred).

/// E-7 (SCHEMA_VERSION 149) — phase discriminator for
/// [`DarwinianGenerationCompletedPayload`].
///
/// Lifted into a payload-internal enum to preserve the single-variant
/// pattern on [`SsbActionType`] (13th application of pattern-2). The
/// 11 variants cover the 8 SsbActionType-variants the original E-7
/// plan proposed PLUS the 3 generation-phase markers.
///
/// Encoded via `serde rename_all = "snake_case"`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DarwinianPhase {
    /// Per-generation MAP-Elites mutation step entered.
    Mutation,
    /// Per-generation parent-selection step entered (Discounted-TS pull).
    Selection,
    /// Per-generation evaluation step entered (PRM scorer + verifier
    /// consensus; pre-promotion gate).
    Evaluation,
    /// Two-Gate guardrail BLOCKED a candidate.
    TwoGateBlocked,
    /// EST invariance probe MEASURED variance > epsilon.
    EstInvarianceMeasured,
    /// Contamination filter REJECTED a held-out task admit.
    ContamFilterRejected,
    /// Transfer-regression threshold BLOCKED promotion.
    TransferRegressionBlocked,
    /// Trap-task FIRED — FATAL band, no promotion.
    TrapTripped,
    /// Archive Ed25519 signature VERIFIED (RUNPOD-PENDING: placeholder).
    ArchiveSigVerified,
    /// Archive Ed25519 signature BROKEN (RUNPOD-PENDING: placeholder).
    ArchiveSigBroken,
    /// MAP-Elites cell ELEVATED — new best-in-cell.
    ArchiveCellElevated,
}

impl DarwinianPhase {
    /// Snake-case wire label.
    #[must_use]
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Mutation => "mutation",
            Self::Selection => "selection",
            Self::Evaluation => "evaluation",
            Self::TwoGateBlocked => "two_gate_blocked",
            Self::EstInvarianceMeasured => "est_invariance_measured",
            Self::ContamFilterRejected => "contam_filter_rejected",
            Self::TransferRegressionBlocked => "transfer_regression_blocked",
            Self::TrapTripped => "trap_tripped",
            Self::ArchiveSigVerified => "archive_sig_verified",
            Self::ArchiveSigBroken => "archive_sig_broken",
            Self::ArchiveCellElevated => "archive_cell_elevated",
        }
    }
}

/// E-7 (SCHEMA_VERSION 149) — payload for
/// [`SsbActionType::DarwinianGenerationCompleted`].
///
/// Wire-dark default; emission gated by `ELAI_DARWINIAN_LOOP_ENABLE`
/// env var AND `cfg(feature = "darwinian-evolve")` cargo feature.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DarwinianGenerationCompletedPayload {
    /// Fire-point discriminator. See [`DarwinianPhase`].
    pub phase: DarwinianPhase,
    /// Per-session monotone counter.
    pub generation_index: u32,
    /// First 8 bytes of canonical BehaviourDescriptor blake3.
    pub cell_blake3_prefix: [u8; 8],
    /// Two-Gate margin gate value (`Some` on TwoGateBlocked /
    /// ArchiveCellElevated).
    pub tau: Option<f64>,
    /// Two-Gate capacity cap (`Some` on TwoGateBlocked). arXiv:2510.04399
    /// does NOT prescribe K_m; ships as config-knob defaulting to 50.
    pub k_m: Option<u32>,
    /// blake3 prefix of reason string (`Some` on blocked/broken phases).
    pub reason_hash: Option<[u8; 8]>,
    /// Fire-point wall-clock (millis since UNIX_EPOCH).
    pub ts_ms: i64,
}

// ---------------------------------------------------------------------------
// batch2-merge-collision recovery (SCHEMA_VERSION 150-156) — payload structs
// for the 9 variants re-applied after parallel-executor write collision in
// cluster s-25120 batch 2. See workflow/sessions/s-25120/completion-drive/
// audits/batch2-fixer-report.md.
// ---------------------------------------------------------------------------

/// E-3 — phase discriminator carried by [`RuntimeHandoffSignalPayload`].
/// `Begin` is emitted when one model hands control to another;
/// `Complete` is emitted when the receiving runtime acknowledges.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum HandoffSignalPhase {
    /// Source runtime emitted the handoff packet.
    Begin,
    /// Destination runtime acknowledged the handoff.
    Complete,
}

/// E-3 — payload for [`SsbActionType::RuntimeHandoffSignal`].
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RuntimeHandoffSignalPayload {
    /// Begin / Complete discriminator.
    pub phase: HandoffSignalPhase,
    /// Source backend canonical name hash.
    pub from_backend_hash: u64,
    /// Destination backend canonical name hash.
    pub to_backend_hash: u64,
    /// Handoff request id (blake3 prefix of correlator).
    pub request_id_hash: u64,
    /// Wall-clock millis since UNIX_EPOCH.
    pub ts_ms: i64,
}

/// E-26 — payload for [`SsbActionType::SglangBackendRegistered`].
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SglangBackendRegisteredPayload {
    /// Lower-cased base_url blake3 hex (full 64-char prefix).
    pub base_url_blake3: String,
    /// Resolved structured-output backend selector (`xgrammar` /
    /// `outlines` / `lm-format-enforcer`).
    pub structured_output_backend: String,
    /// Wall-clock millis since UNIX_EPOCH.
    pub ts_ms: i64,
}

/// E-26 — payload for [`SsbActionType::XgrammarSchemaCompiled`].
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct XgrammarSchemaCompiledPayload {
    /// Cache-key blake3 prefix (64-bit) for the schema.
    pub schema_id_hash: u64,
    /// Full blake3 hex of the schema body.
    pub schema_hash: String,
    /// Wall-clock compile latency (ms).
    pub compile_ms: u64,
    /// Wall-clock millis since UNIX_EPOCH.
    pub ts_ms: i64,
}

/// E-26 — payload for [`SsbActionType::RuntimeColdStartObserved`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct RuntimeColdStartObservedPayload {
    /// Backend canonical name hash.
    pub backend_name_hash: u64,
    /// Elapsed wall-clock millis from first-request to first-response.
    pub elapsed_ms: u64,
    /// Wall-clock millis since UNIX_EPOCH.
    pub ts_ms: i64,
}

/// E-2 — payload for [`SsbActionType::StreamingTimingRecorded`].
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StreamingTimingRecordedPayload {
    /// Correlator id for the chat stream (dashboard-assigned).
    pub request_id: String,
    /// Time-to-first-token (ms).
    pub ttft_ms: u32,
    /// Mean inter-chunk delay (ms).
    pub mean_inter_chunk_ms: u32,
    /// p50 inter-chunk delay (ms).
    pub p50_inter_chunk_ms: u32,
    /// p99 inter-chunk delay (ms).
    pub p99_inter_chunk_ms: u32,
    /// Total stream duration (ms).
    pub total_ms: u32,
    /// Tokens emitted across the stream.
    pub tokens_emitted: u32,
    /// Provider-reported stop reason
    /// (`end_turn` / `max_tokens` / `aborted` / `error`).
    pub stop_reason: String,
    /// Wall-clock millis since UNIX_EPOCH.
    pub ts_ms: i64,
}

/// E-2 — payload for [`SsbActionType::UxPacingVoteRecorded`].
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct UxPacingVoteRecordedPayload {
    /// Correlator id for the stream the user voted on.
    pub request_id: String,
    /// `"snappy"` / `"slow"` user verdict.
    pub vote: String,
    /// Whether the FramePacer was active at vote time.
    pub frame_pace_enabled_at_vote: bool,
    /// Configured chars-per-second at vote time (0 when FramePacer off).
    pub cps_at_vote: u32,
    /// Effort level surfaced to the model (`low` / `medium` / `high`).
    pub effort_level: String,
    /// Wall-clock millis since UNIX_EPOCH.
    pub ts_ms: i64,
}

/// E-28 — payload for [`SsbActionType::StreamingLlmEngaged`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct StreamingLlmEngagedPayload {
    /// Attention-sink count (paper-recommended >= 4 per arXiv:2309.17453).
    pub sink_count: u32,
    /// Rolling KV-cache window size (tokens).
    pub window_size: u32,
    /// Current context position when the sink mask was applied.
    pub context_position: u32,
    /// Wall-clock millis since UNIX_EPOCH.
    pub ts_ms: i64,
}

/// E-28 — payload for [`SsbActionType::SdpaBackendPinned`].
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SdpaBackendPinnedPayload {
    /// Resolved backend
    /// (`flash_attention` / `efficient_attention` / `math` / `unknown`).
    pub backend_name: String,
    /// `true` when torch < 2.3 forced the MATH fallback path.
    pub fallback_used: bool,
    /// CUDA compute capability (e.g. 86 = SM_86 / Ampere).
    pub compute_capability: u32,
    /// Wall-clock millis since UNIX_EPOCH.
    pub ts_ms: i64,
}

/// E-28 — payload for [`SsbActionType::YarnRopeScalingApplied`].
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct YarnRopeScalingAppliedPayload {
    /// YaRN scaling factor (e.g. 4.0 targets 128K from 32K base).
    pub factor: f64,
    /// Original `max_position_embeddings` from the HF config.
    pub original_max_position_embeddings: u32,
    /// Layer index the scaling was applied at (0 = config-level mutation).
    pub applied_at_layer: u32,
    /// Wall-clock millis since UNIX_EPOCH.
    pub ts_ms: i64,
}

// ---------------------------------------------------------------------------
// E-8 (SCHEMA_VERSION 162) — self-evolving prompts substrate.
// SINGLE-VARIANT Pattern-2 (14th application) per
// workflow/reasoning/E-8-decisions.md DECISION-1. Wire-dark unless
// ELAI_PROMPT_OPTIMIZER_ENABLE=1 AND cfg(feature = "darwinian-evolve").
// ---------------------------------------------------------------------------

/// E-8 — phase discriminator carried by [`PromptOptimizationCompletedPayload`].
///
/// Covers the 7 fire-points the original E-8 plan listed as +7 SsbActionType
/// variants; lifted into a payload-internal enum to preserve the single-variant
/// pattern on [`SsbActionType`] (14th application of Pattern-2; precedent at
/// [`DarwinianPhase`]).
///
/// Paper-grounded phases:
///   - `DspyInit` -- arXiv:2310.03714 (DSPy compile / MIPROv2 boot).
///   - `OproStep` -- arXiv:2309.03409 (OPRO meta-prompt trajectory step).
///   - `RankPromptVote` -- arXiv:2403.12373 (RankPrompt; paper-triager flagged
///     WRONG_REGIME -- retained as a forward-compatible slot per
///     workflow/reasoning/E-8-decisions.md DECISION-6).
///   - `ApeProposal` -- arXiv:2211.01910 (APE Monte-Carlo instruction search).
///   - `GepaReflect` -- arXiv:2507.19457 (GEPA reflective text-feedback step).
///   - `ArchivePromote` -- D-1 + E-7 Two-Gate admit of an optimized template.
///   - `ArchiveRetire` -- ETGPO-/ overfit-rejection retirement (arXiv:2602.00997
///     gotchas injection + transfer-regression block).
///
/// Encoded via `serde rename_all = "snake_case"`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OptimizerPhase {
    /// DSPy MIPROv2 / pipeline init.
    DspyInit,
    /// OPRO meta-prompt trajectory step.
    OproStep,
    /// RankPrompt pairwise vote step (forward-compatible slot; WRONG_REGIME).
    RankPromptVote,
    /// APE Monte-Carlo instruction proposal.
    ApeProposal,
    /// GEPA reflective text-feedback step.
    GepaReflect,
    /// Two-Gate promotion of an optimized template (Candidate -> Promoted).
    ArchivePromote,
    /// Retirement of an optimized template (Candidate -> Retired or
    /// Promoted -> Retired).
    ArchiveRetire,
}

impl OptimizerPhase {
    /// Snake-case wire label.
    #[must_use]
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::DspyInit => "dspy_init",
            Self::OproStep => "opro_step",
            Self::RankPromptVote => "rank_prompt_vote",
            Self::ApeProposal => "ape_proposal",
            Self::GepaReflect => "gepa_reflect",
            Self::ArchivePromote => "archive_promote",
            Self::ArchiveRetire => "archive_retire",
        }
    }
}

/// E-8 (SCHEMA_VERSION 162) — payload for
/// [`SsbActionType::PromptOptimizationCompleted`].
///
/// Wire-dark default; emission gated by `ELAI_PROMPT_OPTIMIZER_ENABLE` env
/// var AND `cfg(feature = "darwinian-evolve")` cargo feature (re-uses the
/// E-7 feature gate per workflow/reasoning/E-8-decisions.md DECISION-3).
///
/// Carries enough scalar context for the dashboard
/// `OptimizerRunPanel.svelte` to render per-iter progress + the final
/// admit / retire verdict without re-issuing a query against the
/// orchestrator-side state.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PromptOptimizationCompletedPayload {
    /// Fire-point discriminator. See [`OptimizerPhase`].
    pub phase: OptimizerPhase,
    /// Per-session monotone counter -- one row per optimization step.
    pub iteration_index: u32,
    /// 8-byte blake3 prefix of the optimization-run id (correlates rows
    /// from the same in-flight run).
    pub run_id_prefix: [u8; 8],
    /// 8-byte blake3 prefix of the optimized template content (set on
    /// `GepaReflect` / `ApeProposal` / `OproStep` / `ArchivePromote`).
    pub template_blake3_prefix: [u8; 8],
    /// Validation-set lift in percentage points vs the parent template
    /// (`Some` on `GepaReflect` / `ArchivePromote` / `ArchiveRetire`).
    pub val_lift_pct: Option<f64>,
    /// Held-out-set lift in percentage points (`Some` on `ArchivePromote`
    /// / `ArchiveRetire`).
    pub held_out_lift_pct: Option<f64>,
    /// blake3 prefix of the reason string (`Some` on `ArchiveRetire`).
    pub reason_hash: Option<[u8; 8]>,
    /// Fire-point wall-clock (millis since UNIX_EPOCH).
    pub ts_ms: i64,
}

// ---------------------------------------------------------------------------
// E-10 (SCHEMA_VERSION 163) -- hostile-critic structural simplifications.
// SINGLE-VARIANT Pattern-2 (15th application) per
// workflow/reasoning/E-10-decisions.md DECISION-1. Wire-dark unless
// ELAI_CRITIC_OF_DESIGN_ENABLE=1.
// ---------------------------------------------------------------------------

/// E-10 -- phase discriminator carried by [`CriticDecisionEventPayload`].
///
/// Covers the 9 fire-points the original E-10 plan listed as +9 SsbActionType
/// variants; lifted into a payload-internal enum to preserve the
/// single-variant pattern on [`SsbActionType`] (15th application of Pattern-2;
/// precedent at [`OptimizerPhase`]).
///
/// Tile mapping for the dashboard (6 tiles, 9 phases): CloudDefault + Cache
/// fold into the Tier tile; GrcBlockquote becomes a header badge;
/// MistakeBookSignal becomes a status badge. See E-10-decisions.md
/// DECISION-15.
///
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CriticTilePhase {
    /// FSM-state ablation: full 9-state vs reduced 3-state profile.
    FsmAblation,
    /// Role ablation: full 8-role vs reduced 3-role Self-MoA composition.
    RoleAblation,
    /// 3-layer RAG challenger MRR vs 7-layer baseline.
    RagChallenger,
    /// MistakeBook structured failure signal stored (ICL-poisoning defense
    /// additive delta on top of existing elai_core::mistake_book).
    MistakeBookSignal,
    /// Typed-CoT proof certificate verified (Curry-Howard mechanical check).
    TypedCotVerified,
    /// Per-VRAM tier matrix decision sheet measured.
    TierMatrix,
    /// Cloud-default backend switch test measurement.
    CloudDefault,
    /// GRC blockquote empirical lift measurement.
    GrcBlockquote,
    /// Anthropic prompt-cache hit-rate probe measurement.
    PromptCacheHitRate,
}

impl CriticTilePhase {
    /// Snake-case wire label.
    #[must_use]
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::FsmAblation => "fsm_ablation",
            Self::RoleAblation => "role_ablation",
            Self::RagChallenger => "rag_challenger",
            Self::MistakeBookSignal => "mistake_book_signal",
            Self::TypedCotVerified => "typed_cot_verified",
            Self::TierMatrix => "tier_matrix",
            Self::CloudDefault => "cloud_default",
            Self::GrcBlockquote => "grc_blockquote",
            Self::PromptCacheHitRate => "prompt_cache_hit_rate",
        }
    }
}

/// E-10 (SCHEMA_VERSION 163) -- payload for
/// [`SsbActionType::CriticDecisionEvent`].
///
/// Wire-dark default; emission gated by `ELAI_CRITIC_OF_DESIGN_ENABLE` env
/// var (no cargo feature gate). Carries enough scalar context for the
/// dashboard `CriticDecisionPanel.svelte` to render 6 ablation tiles +
/// paired-bootstrap p + Simplify/Keep verdict without re-issuing a query
/// against the orchestrator-side state.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CriticDecisionEventPayload {
    /// Fire-point discriminator. See [`CriticTilePhase`].
    pub phase: CriticTilePhase,
    /// 8-byte blake3 prefix of the critic-run id (correlates rows from the
    /// same in-flight ablation pass).
    pub run_id_prefix: [u8; 8],
    /// Baseline arm score (e.g. full-pipeline pass-rate or 7-layer MRR).
    pub baseline_score: f64,
    /// Challenger arm score (e.g. reduced-profile pass-rate or 3-layer MRR).
    pub challenger_score: f64,
    /// Paired-bootstrap p-value (Some when E-9 helper ran; None when
    /// computation deferred / unavailable).
    pub paired_bootstrap_p: Option<f64>,
    /// Sample size in the paired-bootstrap window.
    pub sample_size: u32,
    /// Structural verdict (Simplify vs Keep) -- 0 = Keep, 1 = Simplify.
    pub verdict_code: u8,
    /// 8-byte blake3 prefix of the rationale string (write-side; not the
    /// rationale itself -- raw text omitted per CTIM-Rover defense).
    pub rationale_hash: [u8; 8],
    /// Fire-point wall-clock (millis since UNIX_EPOCH).
    pub ts_ms: i64,
}

// ---------------------------------------------------------------------------
// E-43 (SCHEMA_VERSION 164) -- agent skill systems telemetry types
// ---------------------------------------------------------------------------

/// E-43 -- phase discriminator carried by [`SkillSystemEventPayload`].
///
/// Collapses the plan-proposed 3-variant cascade
/// (SkillInvocationLogged + SkillEvalBelowThreshold +
/// SkillExternalImportScanned) into one variant + 3-arm discriminator
/// per Pattern-2 (16th application).
///
/// Phase mapping:
///   - Loaded: skill discovered + parsed by SkillRegistry on startup.
///   - Invoked: skill invocation logged via PostToolUse hook telemetry.
///   - Failed: eval below threshold OR mcp-scan finding OR validator
///     rejection -- the outcome enum on payload disambiguates which.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SkillEventPhase {
    /// Skill discovered + parsed by registry.
    Loaded,
    /// Skill invocation logged (PostToolUse hook).
    Invoked,
    /// Eval-below-threshold / scan-finding / validator-reject.
    Failed,
}

impl SkillEventPhase {
    /// Snake-case wire label.
    #[must_use]
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Loaded => "loaded",
            Self::Invoked => "invoked",
            Self::Failed => "failed",
        }
    }
}

/// E-43 (SCHEMA_VERSION 164) -- payload for
/// [`SsbActionType::SkillSystemEvent`].
///
/// Wire-dark default; emission gated by `ELAI_SKILL_SYSTEM_ENABLE` env
/// var. Carries scalar context so the dashboard can render per-skill
/// telemetry without re-issuing a query against orchestrator state.
/// Raw prompt text is omitted by design (blake3 8-byte prefix only --
/// CTIM-Rover defense).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SkillSystemEventPayload {
    /// Fire-point discriminator. See [`SkillEventPhase`].
    pub phase: SkillEventPhase,
    /// Skill canonical name (kebab-case, matches dir leaf).
    pub skill_name: String,
    /// 8-byte blake3 prefix of the prompt that triggered the skill
    /// (Invoked phase) OR `None` for Loaded/Failed phases.
    pub prompt_hash_prefix: Option<[u8; 8]>,
    /// Trigger method: "explicit" (user typed $<skill-name>) or
    /// "implicit" (auto-trigger). Empty string for Loaded/Failed.
    pub trigger_method: String,
    /// Outcome label: "passed" / "skipped" / "errored" /
    /// "eval_below_threshold" / "scan_finding" / "validator_reject".
    pub outcome: String,
    /// Optional eval precision (Failed phase + outcome
    /// eval_below_threshold).
    pub eval_precision: Option<f32>,
    /// Optional eval recall (same condition).
    pub eval_recall: Option<f32>,
    /// Optional mcp-scan finding count (Failed phase + outcome
    /// scan_finding).
    pub scan_findings: Option<u32>,
    /// Fire-point wall-clock (millis since UNIX_EPOCH).
    pub ts_ms: i64,
}

// ---------------------------------------------------------------------------
// E-9 (SCHEMA_VERSION 165) -- external agent-eval benchmark suite types
// ---------------------------------------------------------------------------

/// E-9 -- phase discriminator carried by [`BenchmarkRunObservedPayload`].
///
/// Collapses the plan-proposed 12-variant cascade (SweBenchLiveRunCompleted +
/// LiveCodeBenchRunCompleted + Tau2BenchRunCompleted + BfclV3RunCompleted +
/// GaiaRunCompleted + ClawsBenchSplitCompleted + SycophancyProbeCompleted +
/// MathFloorsCompleted + HleCeilingTracked + DeepPlanningCompleted +
/// DsCpCoverageMeasured + BenchPowerCalibrated + LowPowerNoConclusion) into
/// one variant + 5-arm discriminator per Pattern-2 (17th application). The
/// suite_name string field on the payload distinguishes the 11 external
/// benchmark suites.
///
/// Phase mapping:
///   - Loaded: suite registered in `external_benches::list_suites` at boot.
///   - Started: suite run dispatched (cli::eval_cmd) before first item.
///   - ItemEvaluated: one fixture item completed (per-task verdict row).
///   - Completed: run aggregated; carries final score + paired-bootstrap p +
///     low_power flag + (clawsbench-only) model/harness effect pp + (ds-cp-
///     only) coverage_gap_pp.
///   - Aborted: run halted via tripwire / power-calibration refusal.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BenchPhase {
    /// Suite registered in the external_benches registry at boot.
    Loaded,
    /// Suite run dispatched (eval_cmd entrypoint) before first item.
    Started,
    /// One fixture item finished; carries per-item score in payload.
    ItemEvaluated,
    /// Run aggregated; final score + p_value + low_power surface.
    Completed,
    /// Run halted (power-calibration refusal / tripwire / I/O failure).
    Aborted,
}

impl BenchPhase {
    /// Snake-case wire label.
    #[must_use]
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Loaded => "loaded",
            Self::Started => "started",
            Self::ItemEvaluated => "item_evaluated",
            Self::Completed => "completed",
            Self::Aborted => "aborted",
        }
    }
}

/// E-9 (SCHEMA_VERSION 165) -- payload for
/// [`SsbActionType::BenchmarkRunObserved`].
///
/// Wire-dark default; emission gated by `ELAI_EVAL_BENCHMARK_ENABLE` env var.
/// Carries scalar context so the dashboard + CLI can render per-suite verdicts
/// without re-issuing a query against orchestrator state.
///
/// Forward-compat envelope (per D-30 CalibrationBootstrapCompletedPayload
/// precedent): suite-specific fields (model_effect_pp, harness_effect_pp,
/// coverage_gap_pp) ship as Option<f64> with None default. Live RUNPOD fields
/// (required_n / observed_variance) extend the payload as Option fields when
/// the live calibration loop comes online.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BenchmarkRunObservedPayload {
    /// Fire-point discriminator. See [`BenchPhase`].
    pub phase: BenchPhase,
    /// Canonical suite name (kebab-case wire identifier; matches
    /// `external_benches::BenchmarkSuite::name`).
    pub suite_name: String,
    /// Suite version tag (semver or arXiv-id anchor).
    pub suite_version: String,
    /// Aggregate score in `[0.0, 1.0]` on Completed; per-item score on
    /// ItemEvaluated; None on Loaded/Started/Aborted.
    pub score: Option<f64>,
    /// Number of items evaluated. None on Loaded/Started.
    pub n: Option<usize>,
    /// Public SOTA reference for telemetry.
    pub sota_reference: Option<f64>,
    /// Paired-bootstrap p-value vs prior run (Completed only).
    pub p_value: Option<f64>,
    /// True when `n < required_n` for the configured significance target.
    pub low_power: bool,
    /// Required-n surface from the power calibrator (Completed only).
    pub required_n: Option<usize>,
    /// ClawsBench-split-only: per-bucket model effect (percentage points).
    pub model_effect_pp: Option<f64>,
    /// ClawsBench-split-only: harness effect (percentage points).
    pub harness_effect_pp: Option<f64>,
    /// DS-CP-only: worst-case coverage gap across in/near/far buckets (pp).
    pub coverage_gap_pp: Option<f64>,
    /// Runtime lane the suite ran against: "local_sidecar" / "cloud_api" /
    /// "mixed". Empty string on Loaded.
    pub runtime_lane: String,
    /// blake3 prefix of the harness scaffold version (16 hex chars). Empty
    /// string on Loaded.
    pub harness_blake3_prefix: String,
    /// Fire-point wall-clock (millis since UNIX_EPOCH).
    pub ts_ms: i64,
}

// ---------------------------------------------------------------------------
// E-19 (SCHEMA_VERSION 166) -- reasoning quality probe types
// ---------------------------------------------------------------------------

/// E-19 -- fire-point discriminator carried by
/// [`ReasoningQualityProbePayload`].
///
/// Collapses the plan-proposed 11-variant cascade (L2SkippedHighConfidence +
/// L2SelfCertaintyScore + L2TokenCapHit + L1ToL3DirectRoute + L4PrmLiteScore +
/// L4LateralKept + L2TypedCotEmitted + L6VerifierDiversityPass +
/// L6VerifierDiversityDisagree + L4TtcBudgetChosen + L2SkippedNoCotClass)
/// into ONE variant + 6-arm discriminator per Pattern-2 (18th application).
/// The `verdict` and `extras` string fields on the payload carry the
/// per-kind sub-state (e.g. `skip_high_confidence` / `reflect` /
/// `l3_direct` / `no_cot_class` for L2 gate decisions).
/// I-5 adds the 6th arm: `VividnessProbe` (per-branch A/B outcome delta).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ProbeKind {
    /// Task 1+3+9 -- pre-L2 confidence gate verdict. The `verdict` payload
    /// field discriminates Skip / Reflect / L3Direct / NoCotClass.
    L2GateDecision,
    /// Task 4 -- PRM-lite L4 node scoring (one row per AB-MCTS candidate
    /// node). Carries auditor_pass + mean_logprob + cosine in the scalar
    /// fields; `score` carries the weighted combination.
    L4PrmLiteScore,
    /// Task 5 -- LToT lateral branch retained one extra ply at L4.
    L4LateralKept,
    /// Task 8 -- difficulty-conditional TTC budget chosen for L4.
    L4TtcBudgetChosen,
    /// Task 7 -- L6 verifier diversity verdict (fresh-seed L1 + rotated
    /// L3). `verdict` carries pass / disagree; `extras` carries the
    /// reason slug.
    L6VerifierDiversity,
    /// I-5 -- per-branch pristine-context A/B outcome delta. Emitted on a
    /// sampled fraction of branch expansions when the umbrella env gate is
    /// open. `score` carries `outcome_delta` (pristine minus accumulated).
    /// Recomputable under `ELAI_SEED` (GD-2).
    VividnessProbe,
}

impl ProbeKind {
    /// Snake-case wire label.
    #[must_use]
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::L2GateDecision => "l2_gate_decision",
            Self::L4PrmLiteScore => "l4_prm_lite_score",
            Self::L4LateralKept => "l4_lateral_kept",
            Self::L4TtcBudgetChosen => "l4_ttc_budget_chosen",
            Self::L6VerifierDiversity => "l6_verifier_diversity",
            Self::VividnessProbe => "vividness_probe",
        }
    }
}

/// E-19 (SCHEMA_VERSION 166) -- payload for
/// [`SsbActionType::ReasoningQualityProbe`].
///
/// Wire-dark default; emission gated by `ELAI_REASONING_QUALITY_PROBE_ENABLE`
/// env var. Forward-compat envelope (per D-30
/// CalibrationBootstrapCompletedPayload precedent): per-kind fields ship
/// as `Option<_>` with None default.
///
/// Field semantics by `kind`:
///   * L2GateDecision: `score` = self-certainty, `threshold` = gate cutoff,
///     `verdict` = "skip_high_confidence" / "reflect" / "l3_direct" /
///     "no_cot_class".
///   * L4PrmLiteScore: `score` = weighted combine, `extras` carries the
///     `auditor_pass=...|mean_logprob=...|cosine=...` summary string.
///   * L4LateralKept: `extras` carries `branch_id|utility=...|consistent=true`.
///   * L4TtcBudgetChosen: `verdict` = difficulty bucket, `extras` carries
///     `depth=N|n_candidates=N|n_lateral=N`.
///   * L6VerifierDiversity: `verdict` = "pass" / "disagree", `extras`
///     carries the reason slug + the two agreement booleans.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ReasoningQualityProbePayload {
    /// Fire-point discriminator. See [`ProbeKind`].
    pub kind: ProbeKind,
    /// Reasoning-pipeline layer (1..=6) the probe fired in.
    pub layer: u8,
    /// Caller-assigned baby step / task identifier. Empty when not
    /// applicable.
    pub baby_step_id: String,
    /// Snake-case verdict slug. Per-kind interpretation -- see struct
    /// docs above. Empty when not applicable.
    pub verdict: String,
    /// Primary scalar score in `[0.0, 1.0]` (kind-specific:
    /// self-certainty / weighted PRM combine / utility / etc.).
    pub score: Option<f32>,
    /// Threshold the score is compared against, when applicable.
    pub threshold: Option<f32>,
    /// Observed token count (L2 token cap probe). None when not
    /// applicable.
    pub observed_tokens: Option<u32>,
    /// Hard cap the observation is compared against (L2 token cap).
    pub token_cap: Option<u32>,
    /// FailureTaxonomy category slug -- carried on L2 gate decisions
    /// + difficulty-budget rows. Empty when not applicable.
    pub failure_taxonomy_category: String,
    /// Free-form key=value summary (per-kind interpretation). Use
    /// `|`-separated `key=value` pairs to keep the field deterministic
    /// without a HashMap (ELAI no-HashMap rule).
    pub extras: String,
    /// Fire-point wall-clock (millis since UNIX_EPOCH).
    pub ts_ms: i64,
}

// ---------------------------------------------------------------------------
// E-54 (SCHEMA_VERSION 167) -- eval-corpus repair pre-deploy
// ---------------------------------------------------------------------------

/// E-54 -- fire-point discriminator carried by [`EvalCorpusEventPayload`].
///
/// Collapses the plan-proposed 14-variant cascade
/// (CorpusIngestRejectedNgram + CorpusIngestRejectedCosine +
/// CorpusShardCreated + CorpusSchemaViolationNoTests +
/// CorpusTestFailsOnReference + CorpusTestUselessPassesBroken +
/// KorOrthogonalityViolated + KorOrthogonalityPassed +
/// SycophancyFramingGapDetected + SandbaggingAnomaly +
/// GeneratorRotationViolated + PerfDeltaContaminationSuspected +
/// DynamicKnowledgeQaGenerated + ProdTrafficProbeInjected) into one variant +
/// 14-arm discriminator per Pattern-2 (19th application).
///
/// Phase mapping:
///   - IngestRejectedNgram: 13-gram K-consecutive match against prior
///     held-out signature; task rejected.
///   - IngestRejectedCosine: embedding cosine >= threshold (0.85 default);
///     task rejected.
///   - ShardCreated: first task per (family, date) shard.
///   - SchemaViolationNoTests: task body missing/empty `tests` field.
///   - TestFailsOnReference: reference_solution fails >=1 of its own tests.
///   - TestUselessPassesBroken: deliberately-broken reference passes all
///     tests (= tests are too weak).
///   - KorOrthogonalityViolated: cross-family transfer accuracy >
///     random_baseline + 10pp; canary build MUST abort.
///   - KorOrthogonalityPassed: cross-family transfer within tolerance.
///   - SycophancyFramingGapDetected: |score_eval - score_deploy| > 10pp on
///     a D-23 matched-pair probe.
///   - SandbaggingAnomaly: latency / token-count deviation between eval +
///     deploy variants of a matched-pair probe.
///   - GeneratorRotationViolated: corpus growth has <3 distinct generators
///     OR any generator's share < 30%.
///   - PerfDeltaContaminationSuspected: self-mod loop improved on family X
///     by >2-sigma vs rolling baseline (ConStat fallback path).
///   - DynamicKnowledgeQaGenerated: OKBench-style daily batch landed via
///     sidecar SearXNG endpoint; audit log appended.
///   - ProdTrafficProbeInjected: opt-in prod-traffic grade probe injected.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CorpusEventPhase {
    /// 13-gram K-consecutive match -> reject.
    IngestRejectedNgram,
    /// Embedding cosine >= threshold -> reject.
    IngestRejectedCosine,
    /// First task per (family, date) shard.
    ShardCreated,
    /// Task body missing / empty `tests`.
    SchemaViolationNoTests,
    /// reference_solution fails >=1 test.
    TestFailsOnReference,
    /// Tests pass on deliberately-broken solution -> reject.
    TestUselessPassesBroken,
    /// Cross-family transfer > random + 10pp.
    KorOrthogonalityViolated,
    /// Cross-family transfer within tolerance.
    KorOrthogonalityPassed,
    /// Matched-pair framing gap > 10pp.
    SycophancyFramingGapDetected,
    /// Latency / token deviation between framing variants.
    SandbaggingAnomaly,
    /// Generator rotation min-30% or <3 generators violated.
    GeneratorRotationViolated,
    /// ConStat / 2-sigma perf jump suspect.
    PerfDeltaContaminationSuspected,
    /// Daily OKBench-style MCQ batch generated via SearXNG.
    DynamicKnowledgeQaGenerated,
    /// Opt-in prod-traffic silent probe injected.
    ProdTrafficProbeInjected,
}

impl CorpusEventPhase {
    /// Snake-case wire label.
    #[must_use]
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::IngestRejectedNgram => "ingest_rejected_ngram",
            Self::IngestRejectedCosine => "ingest_rejected_cosine",
            Self::ShardCreated => "shard_created",
            Self::SchemaViolationNoTests => "schema_violation_no_tests",
            Self::TestFailsOnReference => "test_fails_on_reference",
            Self::TestUselessPassesBroken => "test_useless_passes_broken",
            Self::KorOrthogonalityViolated => "kor_orthogonality_violated",
            Self::KorOrthogonalityPassed => "kor_orthogonality_passed",
            Self::SycophancyFramingGapDetected => "sycophancy_framing_gap_detected",
            Self::SandbaggingAnomaly => "sandbagging_anomaly",
            Self::GeneratorRotationViolated => "generator_rotation_violated",
            Self::PerfDeltaContaminationSuspected => "perf_delta_contamination_suspected",
            Self::DynamicKnowledgeQaGenerated => "dynamic_knowledge_qa_generated",
            Self::ProdTrafficProbeInjected => "prod_traffic_probe_injected",
        }
    }
}

/// E-54 (SCHEMA_VERSION 167) -- payload for
/// [`SsbActionType::EvalCorpusEvent`].
///
/// Wire-dark default; emission gated by `ELAI_EVAL_CORPUS_ENABLE` env var.
/// Carries scalar / string context so the dashboard's EvalCorpusPanel +
/// SycophancyMatchedPairPanel can render per-phase state without re-issuing
/// a query against orchestrator state. Free-form per-phase data goes into
/// `details_json` (serde_json::Value) so future phases can extend the
/// payload without a schema bump.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EvalCorpusEventPayload {
    /// Fire-point discriminator. See [`CorpusEventPhase`].
    pub phase: CorpusEventPhase,
    /// Optional canonical task id (None for kor / perf_delta / generator
    /// rotation phases which aggregate across tasks).
    pub task_id: Option<String>,
    /// Optional family wire label (canary.rs CanaryFamily::as_str output:
    /// `code_fix` / `math_proof` / `prose_reason` / `planning_decomposition`
    /// / `tool_use`).
    pub family: Option<String>,
    /// Free-form per-phase context (e.g. cosine value, framing gap pp,
    /// offending task id, etc.). Serialized as JSON so future phases can
    /// extend without a schema bump.
    pub details_json: serde_json::Value,
    /// Fire-point wall-clock (millis since UNIX_EPOCH).
    pub ts_ms: i64,
}

// ---------------------------------------------------------------------------
// E-55 (SCHEMA_VERSION 168) -- agentic RL + reward ladder substrate
// ---------------------------------------------------------------------------

/// E-55 -- fire-point discriminator carried by [`RewardLadderEventPayload`].
///
/// Collapses 9 proposed SSB variants (RlaifJudgeRated /
/// RlaifCalibrationCorrelationLow / GepaTemplateEvolved /
/// GepaTemplateEvicted / RewardFusionComputed / RewardHackingSuspected /
/// SftTrainingThresholdReached / BaiArmObservationRecorded /
/// RlWeightUpdateUnlocked) into one variant + 9-arm discriminator per
/// Pattern-2 (20th application).
///
/// Phase mapping:
///   - RlaifJudgeRated: cloud Sonnet judge produced a 5-dim Likert rating
///     for a sampled trajectory.
///   - RlaifCalibrationCorrelationLow: quarterly calibration vs human
///     grades dropped below 0.6; judge downgraded to advisory mode.
///   - GepaTemplateEvolved: GEPA reflection on bottom-quartile produced
///     a new template arm; entered BAI candidate set with Beta(1,1).
///   - GepaTemplateEvicted: arm posterior fell below leader/2 after
///     >=30 trials; evicted from candidate set.
///   - RewardFusionComputed: PRM+ORM fused reward computed for one
///     trajectory.
///   - RewardHackingSuspected: MONA myopic-vs-non-myopic divergence
///     flagged a high-myopic-low-step-validity trajectory.
///   - SftTrainingThresholdReached: 200th GREEN rollout collected; SFT
///     fine-tune patch (E-55.1) unblocked.
///   - BaiArmObservationRecorded: BAI Beta posterior updated with the
///     fused-reward Bernoulli observation.
///   - RlWeightUpdateUnlocked: all 3 unlock conditions TRUE
///     (bai_saturated + orm_coverage >= 0.8 + vram_headroom >= 4GB);
///     DPO/PPO/GRPO/KTO weight-update modules now allowed.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RewardLadderEventKind {
    /// Cloud Sonnet 5-dim Likert rating produced.
    RlaifJudgeRated,
    /// Quarterly calibration correlation < 0.6 -> advisory mode.
    RlaifCalibrationCorrelationLow,
    /// GEPA reflection produced a new template arm.
    GepaTemplateEvolved,
    /// Arm evicted after >=30 trials underperformance.
    GepaTemplateEvicted,
    /// PRM+ORM fused reward computed.
    RewardFusionComputed,
    /// MONA myopic-vs-non-myopic divergence flagged.
    RewardHackingSuspected,
    /// 200th GREEN rollout collected -> SFT patch unblocked.
    SftTrainingThresholdReached,
    /// BAI Beta posterior updated with fused-reward observation.
    BaiArmObservationRecorded,
    /// All 3 unlock conditions TRUE -> weight-update RL allowed.
    RlWeightUpdateUnlocked,
}

impl RewardLadderEventKind {
    /// Snake-case wire label.
    #[must_use]
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::RlaifJudgeRated => "rlaif_judge_rated",
            Self::RlaifCalibrationCorrelationLow => "rlaif_calibration_correlation_low",
            Self::GepaTemplateEvolved => "gepa_template_evolved",
            Self::GepaTemplateEvicted => "gepa_template_evicted",
            Self::RewardFusionComputed => "reward_fusion_computed",
            Self::RewardHackingSuspected => "reward_hacking_suspected",
            Self::SftTrainingThresholdReached => "sft_training_threshold_reached",
            Self::BaiArmObservationRecorded => "bai_arm_observation_recorded",
            Self::RlWeightUpdateUnlocked => "rl_weight_update_unlocked",
        }
    }
}

/// E-55 (SCHEMA_VERSION 168) -- payload for
/// [`SsbActionType::RewardLadderEvent`].
///
/// Wire-dark default; emission gated by `ELAI_AGENTIC_RL_ENABLE` env var.
/// Carries the 9-arm discriminator + session / trajectory ids + free-form
/// JSON details so each kind's specific scalars (rating dims, weights,
/// penalty breakdowns, unlock-condition flags) can ride along without
/// per-kind payload structs (Pattern-2 single-variant scope-lean).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RewardLadderEventPayload {
    /// Fire-point discriminator. See [`RewardLadderEventKind`].
    pub kind: RewardLadderEventKind,
    /// Optional canonical session id (the E-55 substrate emits per-session
    /// rather than per-task, so trajectory_id is the finer-grained key).
    pub session_id: Option<String>,
    /// Optional trajectory id (None for kinds that aggregate across
    /// trajectories such as `RlWeightUpdateUnlocked`).
    pub trajectory_id: Option<String>,
    /// Free-form per-kind context (e.g. rating dims, fusion weights,
    /// penalty breakdown, unlock-condition flags). Serialized as JSON so
    /// future kinds can extend without a schema bump.
    pub details_json: serde_json::Value,
    /// Fire-point wall-clock (millis since UNIX_EPOCH).
    pub ts_ms: i64,
}

// ---------------------------------------------------------------------------
// E-11 (SCHEMA_VERSION 169) -- SaaS prior-art catalog + recommender types
// ---------------------------------------------------------------------------

/// E-11 -- fire-point discriminator carried by
/// [`SaasPriorArtEventPayload`].
///
/// Collapses 2 proposed SSB variants (ArchetypeRecommended +
/// PriorArtConsulted) into one variant + 2-arm discriminator per
/// Pattern-2 (21st application).
///
/// Phase mapping:
///   - ArchetypeRecommended: in-process call to
///     `architecture::recommend_archetype` produced an
///     ArchetypeRecommendation; payload carries primary archetype
///     snake_case + confidence snake_case + clarifying-question count.
///   - PriorArtConsulted: caller (idea-critic / production-grade
///     auditor / dashboard recommender panel) requested
///     `PriorArtRegistry::entries_for_archetype`; payload carries the
///     consulted archetype + the count of matching prior-art entries +
///     a caller-site label.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PriorArtEventKind {
    /// Recommender produced an ArchetypeRecommendation.
    ArchetypeRecommended,
    /// Prior-art registry was queried for archetype-tagged entries.
    PriorArtConsulted,
}

impl PriorArtEventKind {
    /// Snake-case wire label.
    #[must_use]
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::ArchetypeRecommended => "archetype_recommended",
            Self::PriorArtConsulted => "prior_art_consulted",
        }
    }
}

/// E-11 (SCHEMA_VERSION 169) -- payload for
/// [`SsbActionType::SaasPriorArtEvent`].
///
/// Wire-dark default; emission gated by `ELAI_SAAS_PRIOR_ART_ENABLE`
/// env var. Carries the 2-arm discriminator + session id + archetype
/// snake_case label + confidence label + free-form JSON details so the
/// 2 kinds can carry their per-kind scalars (clarifying-question
/// count, prior-art result count, caller-site label) without per-kind
/// payload structs (Pattern-2 single-variant scope-lean).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SaasPriorArtEventPayload {
    /// Fire-point discriminator. See [`PriorArtEventKind`].
    pub kind: PriorArtEventKind,
    /// Canonical session id (optional; the recommender can be called
    /// out-of-band of any orchestrator session).
    pub session_id: Option<String>,
    /// Recommended (or consulted) archetype as the canonical
    /// snake_case label (e.g. `"saas_dashboard"`,
    /// `"b2b_marketplace"`).
    pub archetype: String,
    /// Snake_case confidence label
    /// (`"high"` / `"medium"` / `"low"`) for
    /// `ArchetypeRecommended`. Empty string for `PriorArtConsulted`.
    pub confidence: String,
    /// Free-form per-kind context (e.g. clarifying-question count,
    /// prior-art result count, caller-site label, ProjectClass
    /// snake_case label). Serialized as JSON so future kinds can
    /// extend without a schema bump.
    pub details_json: serde_json::Value,
    /// Fire-point wall-clock (millis since UNIX_EPOCH).
    pub ts_ms: i64,
}

// ---------------------------------------------------------------------------
// E-22 (SCHEMA_VERSION 170) -- may-2026 github trending sweep substrate
// ---------------------------------------------------------------------------

/// E-22 -- per-kind discriminator carried by
/// [`GithubTrendingEventPayload`].
///
/// Collapses 5 proposed SSB variants (skill_export_completed +
/// skill_export_drift + sandbox_microvm_unavailable +
/// mcp_lazy_schema_savings + mcp_transport_negotiated) into one
/// variant + 3-arm discriminator per Pattern-2 (22nd application). The
/// skill_export_completed + skill_export_drift kinds are merged onto
/// `SkillExportNormalized` with a `drift_count` field (0 = clean, >0 =
/// drift). The mcp_transport_negotiated kind is OMITTED -- E-24 already
/// emits `McpLegacySseTransport` which is the only practical fire-point.
///
/// Phase mapping:
///   - `SkillExportNormalized`: `elai skills-mgmt export` ran a normalised
///     SKILL.md write to dist/<target>/skills/<name>/SKILL.md.
///   - `MicroVmFallback`: `orchestrator::sandbox::select_backend` requested
///     the Microsandbox backend but degraded to a fallback (linux+KVM
///     only with MICROSANDBOX_BIN unset, OR windows AppContainer).
///   - `LazySchemaSavings`: `DefaultMcpBridge::http_list_tools` emitted a
///     {name,description} lazy summary instead of the full input_schema.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TrendingEventKind {
    /// `elai skills-mgmt export` emitted a normalised SKILL.md.
    SkillExportNormalized,
    /// `select_backend` requested Microsandbox; degraded to fallback.
    MicroVmFallback,
    /// `http_list_tools` returned a lazy {name,description} shape.
    LazySchemaSavings,
}

impl TrendingEventKind {
    /// Snake-case wire label.
    #[must_use]
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::SkillExportNormalized => "skill_export_normalized",
            Self::MicroVmFallback => "microvm_fallback",
            Self::LazySchemaSavings => "lazy_schema_savings",
        }
    }
}

/// E-22 (SCHEMA_VERSION 170) -- payload for
/// [`SsbActionType::GithubTrendingEvent`].
///
/// Wire-dark default; emission gated by the `ELAI_GITHUB_TRENDING_ENABLE`
/// env var (read at construction time by the slice owner -- skills_cmd
/// for the export kind, orchestrator::sandbox for the fallback kind,
/// plugins::mcp_bridge for the savings kind). Carries the 3-arm
/// discriminator + per-kind scalar fields + free-form JSON details so
/// the dashboard can render telemetry per fire-point without re-issuing
/// any orchestrator query. Empty strings + 0 integers on fields not
/// applicable to the current kind.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GithubTrendingEventPayload {
    /// Fire-point discriminator. See [`TrendingEventKind`].
    pub kind: TrendingEventKind,
    /// Optional session id (the export + microvm + lazy-schema fire-
    /// points can run out-of-band of any orchestrator session).
    pub session_id: Option<String>,
    /// Skill canonical name (kebab-case, matches dir leaf). Empty
    /// string for non-SkillExportNormalized kinds.
    pub skill_name: String,
    /// Reason label for `MicroVmFallback` (e.g. `"missing_bin"`,
    /// `"no_kvm"`, `"windows_native"`). Empty string for other kinds.
    pub reason_label: String,
    /// Fallback backend snake_case label for `MicroVmFallback`
    /// (`"firecracker"` / `"gvisor"` / `"app_container"`). Empty
    /// string for other kinds.
    pub fallback_backend: String,
    /// Estimated full-schema bytes for `LazySchemaSavings`. 0 for
    /// other kinds.
    pub pre_bytes: u64,
    /// Lazy summary bytes actually emitted for `LazySchemaSavings`.
    /// 0 for other kinds.
    pub post_bytes: u64,
    /// Tool count in the lazy listing for `LazySchemaSavings`. 0 for
    /// other kinds.
    pub tool_count: u32,
    /// blake3 8-byte prefix diff count for `SkillExportNormalized`
    /// (0 = clean export against any prior dist copy; >0 = drift
    /// detected). 0 for other kinds.
    pub drift_count: u32,
    /// Free-form per-kind context (e.g. dist_path label, target
    /// vendor, caller_site, microsandbox CLI version when known).
    /// Serialized as JSON so future kinds can extend without a
    /// schema bump.
    pub details_json: serde_json::Value,
    /// Fire-point wall-clock (millis since UNIX_EPOCH).
    pub ts_ms: i64,
}

/// E-22 -- env var that flips the github_trending_event family from
/// wire-dark to live emission. Match-string is `"1"` or `"true"`
/// (case-insensitive); any other value leaves the emitter as a noop.
/// Default-off so existing tests keep their expected SSB row counts.
pub const ELAI_GITHUB_TRENDING_ENABLE_ENV: &str = "ELAI_GITHUB_TRENDING_ENABLE";

// ---------------------------------------------------------------------------
// E-23 (SCHEMA_VERSION 171) -- 2026 developer-discourse positioning + UX-guard
// ---------------------------------------------------------------------------

/// E-23 -- per-kind discriminator carried by
/// [`DeveloperDiscourseEventPayload`].
///
/// Collapses 3 proposed SSB variants (trust_telemetry_aggregated +
/// runtime_choice_badge_emitted + context_state_warning) into one
/// variant + 3-arm discriminator per Pattern-2 (23rd application).
///
/// Phase mapping:
///   - `TrustTelemetryAggregated`: `orchestrator::trust_telemetry::
///     aggregate_session` rolled up productivity / verification / drift
///     counters from existing SSB rows.
///   - `RuntimeChoiceBadgeEmitted`: CommandWindow status-row label
///     transitioned between local / cloud-billed (e.g. local sidecar OOM
///     fallback to Anthropic API).
///   - `ContextStateWarning`: ContextStatePanel crossed a token-budget
///     threshold (70% yellow, 85% red) before PreCompact.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DiscourseEventKind {
    /// `trust_telemetry::aggregate_session` produced a TrustReport.
    TrustTelemetryAggregated,
    /// CommandWindow runtime_choice_badge transitioned to a new label.
    RuntimeChoiceBadgeEmitted,
    /// ContextStatePanel crossed a yellow (70%) or red (85%) threshold.
    ContextStateWarning,
}

impl DiscourseEventKind {
    /// Snake-case wire label.
    #[must_use]
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::TrustTelemetryAggregated => "trust_telemetry_aggregated",
            Self::RuntimeChoiceBadgeEmitted => "runtime_choice_badge_emitted",
            Self::ContextStateWarning => "context_state_warning",
        }
    }
}

/// E-23 (SCHEMA_VERSION 171) -- payload for
/// [`SsbActionType::DeveloperDiscourseEvent`].
///
/// Wire-dark default; emission gated by the
/// `ELAI_DEVELOPER_DISCOURSE_ENABLE` env var (read at construction time by
/// the slice owner -- `trust_telemetry::aggregate_session` for the
/// aggregated kind, dashboard-v2 command path for the badge-emitted kind,
/// `context_state` Tauri command for the threshold-warning kind). Carries
/// the 3-arm discriminator + per-kind scalar fields + free-form JSON
/// details so the dashboard can render telemetry per fire-point without
/// re-issuing any orchestrator query. Empty strings + 0 numbers on fields
/// not applicable to the current kind.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DeveloperDiscourseEventPayload {
    /// Fire-point discriminator. See [`DiscourseEventKind`].
    pub kind: DiscourseEventKind,
    /// Optional session id (telemetry can roll up across sessions).
    pub session_id: Option<String>,
    /// Tasks shipped per hour (productivity proxy from the
    /// TrustTelemetryAggregated rollup). 0.0 for other kinds.
    pub tasks_per_hour: f64,
    /// Auditor findings per shipped task (verification density). 0.0 for
    /// other kinds.
    pub findings_per_task: f64,
    /// Reverted-change ratio (rollback signal). 0.0 for other kinds.
    pub revert_rate: f64,
    /// Count of mcp_think calls with empty `grounding` field over the
    /// session window. 0 for other kinds.
    pub ungrounded_count: u64,
    /// Previous badge label (`"local"` / `"cloud_billed"` / empty for
    /// other kinds). Used by RuntimeChoiceBadgeEmitted to record the
    /// transition source side.
    pub prev_badge: String,
    /// New badge label (same vocabulary as `prev_badge`). Empty string
    /// for other kinds.
    pub next_badge: String,
    /// Token-budget percentage at the warning event (0.0..=100.0). 0.0
    /// for non ContextStateWarning kinds.
    pub token_budget_pct: f32,
    /// Threshold label (`"yellow"` for >=70%, `"red"` for >=85%). Empty
    /// for other kinds.
    pub threshold_label: String,
    /// Free-form per-kind context (e.g. stagnation_dist map, fallback
    /// chain summary, FSM substate at warning time). Serialized as JSON
    /// so future kinds can extend without a schema bump.
    pub details_json: serde_json::Value,
    /// Fire-point wall-clock (millis since UNIX_EPOCH).
    pub ts_ms: i64,
}

/// E-23 -- env var that flips the developer_discourse_event family from
/// wire-dark to live emission. Match-string is `"1"` or `"true"`
/// (case-insensitive); any other value leaves the emitter as a noop.
/// Default-off so existing tests keep their expected SSB row counts.
pub const ELAI_DEVELOPER_DISCOURSE_ENABLE_ENV: &str = "ELAI_DEVELOPER_DISCOURSE_ENABLE";

/// E-23 -- ContextStatePanel yellow-threshold cutoff (token-budget %).
/// Crossing this value emits a [`DiscourseEventKind::ContextStateWarning`]
/// row with `threshold_label="yellow"`. Sourced from the plan body Task 1.
pub const CONTEXT_BUDGET_YELLOW_PCT: f32 = 70.0;

/// E-23 -- ContextStatePanel red-threshold cutoff (token-budget %).
/// Crossing this value emits a [`DiscourseEventKind::ContextStateWarning`]
/// row with `threshold_label="red"`. Sourced from the plan body Task 1.
pub const CONTEXT_BUDGET_RED_PCT: f32 = 85.0;

/// E-23 -- threshold (hours) after which the prompt-injection check is
/// shown as "yellow advisory" in the CommandWindow status row. Sourced
/// from the plan body Task 7. Used by the dashboard ContextStatePanel
/// + injection-check status row to decide which colour state to render.
pub const PROMPT_INJECTION_CHECK_STALE_HOURS: u64 = 24;

// ---------------------------------------------------------------------------
// E-29 Tier-B (SCHEMA_VERSION 173) -- agentic-observability event surface
// ---------------------------------------------------------------------------

/// Discriminator for the agentic-observability SSB event family. Each
/// arm corresponds to one E-29 Tier-B fire-point:
///
///   * `MastFailureClassified` -- offline MAST classifier emitted a
///     hit per [`crate::mast::MastMode`] (14-variant taxonomy from
///     arXiv:2503.13657). DOES NOT collide with the pre-existing
///     `failure_classified` action_type (DRIFT-2; the existing variant
///     carries the 13-category SYMPTOM payload from `crate::regression`).
///   * `ReplayResume` -- `ssb_resume_from(checkpoint_id, patch)`
///     validator call completed. Outcome is encoded in the payload's
///     `replay_outcome` field (`refused` / `predicate_unavailable` /
///     `accepted`).
///   * `SignalMatch` -- Laminar-style natural-language signal
///     backfill produced a hit.
///   * `EvalDatasetMember` -- production trace promoted to an eval
///     dataset (Langfuse / LangSmith pattern).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AgenticObservabilityEventKind {
    MastFailureClassified,
    ReplayResume,
    SignalMatch,
    EvalDatasetMember,
}

impl AgenticObservabilityEventKind {
    /// Snake-case wire label.
    #[must_use]
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::MastFailureClassified => "mast_failure_classified",
            Self::ReplayResume => "replay_resume",
            Self::SignalMatch => "signal_match",
            Self::EvalDatasetMember => "eval_dataset_member",
        }
    }
}

/// E-29 Tier-B (SCHEMA_VERSION 173) -- payload for
/// [`SsbActionType::AgenticObservabilityEvent`].
///
/// Wire-dark default; emission gated by
/// `ELAI_AGENTIC_OBSERVABILITY_ENABLE` (read by the
/// [`crate::observability`] slice owners at construction time). The
/// payload carries the 4-arm discriminator + per-kind scalar fields +
/// free-form JSON details so the dashboard can render telemetry per
/// fire-point without re-issuing any orchestrator query. Empty
/// strings + 0 numbers on fields not applicable to the current kind.
///
/// Cross-references:
///   * `mast_failure_classified` populates `mast_mode_label` (one of
///     the 14 tags from [`MastMode::as_sql_str`]) +
///     `evidence_row_ids` + `confidence_bp` + `session_id`.
///   * `replay_resume` populates `replay_session_id` +
///     `replay_outcome` + `refusal_reason` + `checkpoint_id`.
///   * `signal_match` populates `signal_label` + `matched_row_ids` +
///     `classifier_model` + `confidence_bp`.
///   * `eval_dataset_member` populates `dataset_name` + `trace_ids`
///     (single-element vec when emitted per-row; multi-element when
///     batched) + `dataset_label` + `scoring_rubric`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AgenticObservabilityEventPayload {
    /// Fire-point discriminator. See [`AgenticObservabilityEventKind`].
    pub kind: AgenticObservabilityEventKind,
    /// Optional session id (telemetry rolls up across sessions).
    pub session_id: Option<String>,
    /// MAST mode tag (`fm_1_1_disobey_task_spec`, ...). Empty for
    /// non-MastFailureClassified kinds. See [`MastMode::as_sql_str`].
    pub mast_mode_label: String,
    /// SSB seq ids that drove the classification / signal match.
    /// Empty when not applicable.
    pub evidence_row_ids: Vec<i64>,
    /// Confidence in basis points (0..=10_000). 0 for kinds where
    /// confidence is not a scalar (e.g. ReplayResume).
    pub confidence_bp: u16,
    /// Replay session id (for ReplayResume kind). Empty for other
    /// kinds. Deterministic blake3-prefixed (see
    /// [`crate::observability::replay::ReplaySessionId::derive`]).
    pub replay_session_id: String,
    /// Outcome string for ReplayResume kind: `refused` /
    /// `predicate_unavailable` / `accepted`. Empty for other kinds.
    pub replay_outcome: String,
    /// Reason text for ReplayResume kind when outcome != accepted.
    pub refusal_reason: String,
    /// Parent checkpoint id for ReplayResume kind. 0 for other kinds.
    pub checkpoint_id: i64,
    /// Signal label (free-text) for SignalMatch kind. Empty for
    /// other kinds.
    pub signal_label: String,
    /// Matched SSB row seq ids (for SignalMatch). Empty for other
    /// kinds. Note: `evidence_row_ids` carries the union across kinds;
    /// `matched_row_ids` is the SignalMatch-specific narrow channel.
    pub matched_row_ids: Vec<i64>,
    /// Classifier model id (for SignalMatch / MastFailureClassified
    /// when the LLM-backed alt classifier ran). Empty for the
    /// deterministic mast_classify path.
    pub classifier_model: String,
    /// Eval dataset name (for EvalDatasetMember). Empty for other
    /// kinds.
    pub dataset_name: String,
    /// Trace ids promoted into the dataset (for EvalDatasetMember).
    /// Empty for other kinds.
    pub trace_ids: Vec<i64>,
    /// Optional dataset-side label (e.g. `"hard"`).
    pub dataset_label: String,
    /// Scoring rubric string (one of `binary_pass` / `likert_5` /
    /// `rubric_v1`). Empty for other kinds.
    pub scoring_rubric: String,
    /// Free-form per-kind context. Serialized as JSON so future
    /// fields can extend without a schema bump.
    pub details_json: serde_json::Value,
    /// Fire-point wall-clock (millis since UNIX_EPOCH).
    pub ts_ms: i64,

    // -----------------------------------------------------------------
    // E-29 OTel-style columns (15 fields, all Option<_> with serde
    // default so existing msgpack rows round-trip without re-encoding).
    // Mirrors the W3C trace-context + OpenTelemetry SpanContext +
    // Resource semantic conventions (opentelemetry.io/docs/specs/otel/
    // trace/api/ and resource/sdk/). Live OTLP export is RUNPOD-PENDING;
    // the columns let us populate spans BEFORE any exporter wiring so
    // ingest schema is stable. The `ssb-otel-export` crate consumes
    // these fields and constructs an opentelemetry::trace::SpanContext
    // + Span when the otlp-export feature is enabled.
    // -----------------------------------------------------------------
    /// W3C trace-context trace_id (32 lowercase hex chars per spec).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub trace_id: Option<String>,
    /// W3C trace-context span_id (16 lowercase hex chars per spec).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub span_id: Option<String>,
    /// Parent span_id when this event is part of a nested span chain.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub parent_span_id: Option<String>,
    /// W3C trace-context trace_flags byte rendered as 2 hex chars
    /// (e.g. "01" = sampled).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub trace_flags: Option<String>,
    /// W3C trace-context tracestate header value (comma-separated
    /// key=value vendor-specific list).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub trace_state: Option<String>,
    /// Human-readable span name (e.g. "elai.mast.classify",
    /// "elai.replay.resume").
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub span_name: Option<String>,
    /// OTel SpanKind ("internal" / "server" / "client" / "producer" /
    /// "consumer"). String for cross-language stability.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub span_kind: Option<String>,
    /// OTel status code ("unset" / "ok" / "error").
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub span_status_code: Option<String>,
    /// OTel status description (free text when status = error).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub span_status_message: Option<String>,
    /// OTel resource service.name attribute (e.g. "elai-orchestrator",
    /// "elai-sidecar").
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub service_name: Option<String>,
    /// OTel resource service.namespace attribute (e.g. "elai").
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub service_namespace: Option<String>,
    /// OTel resource service.instance.id attribute (UUID/blake3 of
    /// the running process).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub service_instance_id: Option<String>,
    /// OTel resource attributes as serialized JSON object string
    /// ({"host.name":"...","host.arch":"x86_64",...}). Stored as JSON
    /// rather than typed map for forward compatibility.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub resource_attributes_json: Option<String>,
    /// OTel span attributes as serialized JSON object string
    /// ({"mast.mode":"fm_1_1_disobey_task_spec",
    ///   "confidence_bp":10000, ...}).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub span_attributes_json: Option<String>,
    /// OTel span events as serialized JSON array of
    /// {name, timestamp_ns, attributes}.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub span_events_json: Option<String>,
}

impl AgenticObservabilityEventPayload {
    /// Convenience constructor for the MAST classifier emission path.
    /// Sets `kind = MastFailureClassified` + the `mast_mode_label`
    /// from `mode.as_sql_str()`.
    #[must_use]
    pub fn mast_failure_classified(
        session_id: impl Into<String>,
        mode: MastMode,
        evidence_row_ids: Vec<i64>,
        confidence_bp: u16,
        classifier_model: impl Into<String>,
        ts_ms: i64,
    ) -> Self {
        Self {
            kind: AgenticObservabilityEventKind::MastFailureClassified,
            session_id: Some(session_id.into()),
            mast_mode_label: mode.as_sql_str().to_string(),
            evidence_row_ids,
            confidence_bp,
            replay_session_id: String::new(),
            replay_outcome: String::new(),
            refusal_reason: String::new(),
            checkpoint_id: 0,
            signal_label: String::new(),
            matched_row_ids: Vec::new(),
            classifier_model: classifier_model.into(),
            dataset_name: String::new(),
            trace_ids: Vec::new(),
            dataset_label: String::new(),
            scoring_rubric: String::new(),
            details_json: serde_json::Value::Null,
            ts_ms,
            // OTel cols default to None; callers wanting OTLP export
            // populate via `with_otel_span(...)` (see impl below).
            trace_id: None,
            span_id: None,
            parent_span_id: None,
            trace_flags: None,
            trace_state: None,
            span_name: None,
            span_kind: None,
            span_status_code: None,
            span_status_message: None,
            service_name: None,
            service_namespace: None,
            service_instance_id: None,
            resource_attributes_json: None,
            span_attributes_json: None,
            span_events_json: None,
        }
    }

    /// Convenience constructor for the replay validator emission path.
    #[must_use]
    pub fn replay_resume(
        checkpoint_id: i64,
        replay_session_id: impl Into<String>,
        outcome: impl Into<String>,
        refusal_reason: impl Into<String>,
        ts_ms: i64,
    ) -> Self {
        Self {
            kind: AgenticObservabilityEventKind::ReplayResume,
            session_id: None,
            mast_mode_label: String::new(),
            evidence_row_ids: Vec::new(),
            confidence_bp: 0,
            replay_session_id: replay_session_id.into(),
            replay_outcome: outcome.into(),
            refusal_reason: refusal_reason.into(),
            checkpoint_id,
            signal_label: String::new(),
            matched_row_ids: Vec::new(),
            classifier_model: String::new(),
            dataset_name: String::new(),
            trace_ids: Vec::new(),
            dataset_label: String::new(),
            scoring_rubric: String::new(),
            details_json: serde_json::Value::Null,
            ts_ms,
            // OTel cols default to None on the replay-resume path; the
            // ssb-otel-export crate fills them when the wire path is
            // enabled via `with_otel_span(...)`.
            trace_id: None,
            span_id: None,
            parent_span_id: None,
            trace_flags: None,
            trace_state: None,
            span_name: None,
            span_kind: None,
            span_status_code: None,
            span_status_message: None,
            service_name: None,
            service_namespace: None,
            service_instance_id: None,
            resource_attributes_json: None,
            span_attributes_json: None,
            span_events_json: None,
        }
    }

    /// Populate the OTel-style trace context + resource columns on an
    /// existing payload. Returns `self` for chained construction.
    ///
    /// Wiring: the `ssb-otel-export` crate calls this BEFORE emit when
    /// the `otlp-export` feature is enabled so the SSB row carries the
    /// same trace_id / span_id pair that the OTLP exporter will use,
    /// keeping row-level audit trails consistent with downstream
    /// tracing backends (Jaeger / Tempo / Honeycomb).
    #[must_use]
    pub fn with_otel_span(
        mut self,
        trace_id: impl Into<String>,
        span_id: impl Into<String>,
        parent_span_id: Option<String>,
        span_name: impl Into<String>,
        service_name: impl Into<String>,
    ) -> Self {
        self.trace_id = Some(trace_id.into());
        self.span_id = Some(span_id.into());
        self.parent_span_id = parent_span_id;
        self.span_name = Some(span_name.into());
        self.service_name = Some(service_name.into());
        // Default trace_flags = "01" (sampled) per W3C trace-context
        // spec section 3.3.3 -- our SSB writes are by definition
        // sampled.
        if self.trace_flags.is_none() {
            self.trace_flags = Some("01".to_string());
        }
        // Default span_kind = "internal" -- our spans are intra-process
        // orchestrator events, not RPC servers/clients.
        if self.span_kind.is_none() {
            self.span_kind = Some("internal".to_string());
        }
        // Default status = "unset" per OTel spec; ssb-otel-export
        // upgrades to "ok"/"error" based on outcome.
        if self.span_status_code.is_none() {
            self.span_status_code = Some("unset".to_string());
        }
        // Default namespace = "elai" -- all our services live under it.
        if self.service_namespace.is_none() {
            self.service_namespace = Some("elai".to_string());
        }
        self
    }

    /// Populate the OTel-style attributes columns on an existing
    /// payload. Returns `self` for chained construction.
    #[must_use]
    pub fn with_otel_attributes(
        mut self,
        resource_attributes_json: Option<String>,
        span_attributes_json: Option<String>,
        span_events_json: Option<String>,
    ) -> Self {
        self.resource_attributes_json = resource_attributes_json;
        self.span_attributes_json = span_attributes_json;
        self.span_events_json = span_events_json;
        self
    }
}

/// E-29 Tier-B -- env var that flips the agentic_observability_event
/// family from wire-dark to live emission. Match-string is `"1"` or
/// `"true"` (case-insensitive); any other value leaves the emitter as a
/// noop. Default-off so existing tests keep their expected SSB row
/// counts.
pub const ELAI_AGENTIC_OBSERVABILITY_ENABLE_ENV: &str =
    "ELAI_AGENTIC_OBSERVABILITY_ENABLE";

// ---------------------------------------------------------------------------
// E-31 (SCHEMA_VERSION 174) -- coding-model-leaderboard scraper substrate
// ---------------------------------------------------------------------------

/// E-31 -- per-kind discriminator carried by
/// [`CodingModelLeaderboardEventPayload`].
///
/// Single SsbActionType variant + 3-arm discriminator per Pattern-2 (26th
/// application). Collapses 3 fire-points (scrape_started /
/// scrape_completed / scrape_failed) onto one wire variant so the SCHEMA
/// bump is exactly +1 (173 -> 174) and TOTAL_VARIANTS bump is exactly +1
/// (446 -> 447). Wire-dark unless `ELAI_CODING_MODEL_LEADERBOARD_ENABLE=1`.
///
/// Phase mapping (mirrors the BenchmarkRunObserved scrape-phase pattern at
/// `benches/external/_common.py:59-81`):
///   - `ScrapeStarted`: scraper began an HTTP fetch against a leaderboard
///     URL. Carries `leaderboard_url` + `suite_name` + `suite_version`.
///   - `ScrapeCompleted`: scraper finished parsing one model entry on a
///     leaderboard. Emits one row per (suite, model) tuple. Carries
///     `model_name` + `score_pct_bp` (basis points 0-10000) + `sample_size`
///     + `entry_hash` (blake3 8-byte hex prefix of the canonical row
///     tuple for dedup) + `pricing_label` (snake_case model-pricing tier).
///   - `ScrapeFailed`: scraper hit HTTP non-200, parse error, ENV-gate
///     refusal, or 5-source-rate-limit. Carries `reason_label` +
///     `http_status` (0 if non-HTTP) + `suite_name`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LeaderboardEventKind {
    /// Scraper began HTTP fetch against a leaderboard URL.
    ScrapeStarted,
    /// Scraper finished parsing one model entry on a leaderboard.
    ScrapeCompleted,
    /// Scraper hit HTTP non-200, parse error, or ENV-gate refusal.
    ScrapeFailed,
}

impl LeaderboardEventKind {
    /// Snake-case wire label.
    #[must_use]
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::ScrapeStarted => "scrape_started",
            Self::ScrapeCompleted => "scrape_completed",
            Self::ScrapeFailed => "scrape_failed",
        }
    }
}

/// E-31 (SCHEMA_VERSION 174) -- payload for
/// [`SsbActionType::CodingModelLeaderboardEvent`].
///
/// Wire-dark default; emission gated by the
/// `ELAI_CODING_MODEL_LEADERBOARD_ENABLE` env var (read at construction
/// time by the scraper at `benches/external/coding_model_leaderboard.py`
/// + any future rust shim). Carries the 3-arm discriminator + per-kind
/// scalar fields + free-form `details_json` so future kinds (e.g.
/// `cross_suite_correlation_computed`) can extend without a schema bump.
/// Empty strings + 0 integers on fields not applicable to the current kind.
///
/// The `score_pct_bp` field is basis points (0-10000) so JSON scrapes
/// from sources reporting "46%" land as `4600` -- avoids float drift on
/// the wire and keeps the round-trip test deterministic.
///
/// The `entry_hash` field is a blake3 8-byte hex prefix of the canonical
/// row tuple `(suite_name, suite_version, model_name, score_pct_bp,
/// sample_size, scrape_ts_ms_rounded)` so the dispatcher can dedup
/// (suite, model) tuples on the read side without keeping the full
/// row. blake3 -- NOT blake2/sha2 per CLAUDE.md Anti-Stale-Training.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CodingModelLeaderboardEventPayload {
    /// Fire-point discriminator. See [`LeaderboardEventKind`].
    pub kind: LeaderboardEventKind,
    /// Optional session id (scraper runs out-of-band of any orchestrator
    /// session by default; ENV-gate flips it live).
    pub session_id: Option<String>,
    /// Canonical leaderboard suite name (kebab-case, matches scraper
    /// SUITE_NAME constant). One of: `"bigcodebench"`, `"humaneval-plus"`,
    /// `"mbpp-plus"`, `"swe-bench-pro"`, `"livecodebench"`.
    pub suite_name: String,
    /// Suite version label (e.g. `"v6"` for LiveCodeBench, `"hard-set"`
    /// for BigCodeBench, `"public"` for SWE-Bench Pro). Empty string if
    /// the source does not version its leaderboard.
    pub suite_version: String,
    /// Source URL the scraper fetched. Empty string for `ScrapeFailed`
    /// kinds that never opened a connection (e.g. ENV-gate refusal).
    pub leaderboard_url: String,
    /// Per-run unique id (uuidv4 string preferred; empty for legacy
    /// rows) so all rows from one scrape run join on a single id.
    pub scrape_run_id: String,
    /// Canonical model identifier (kebab-case). Empty for `ScrapeStarted`
    /// + `ScrapeFailed` kinds. Examples: `"claude-opus-4-7"`,
    /// `"claude-sonnet-4-6"`, `"qwen2-5-coder-7b-instruct-abliterated"`,
    /// `"deepseek-v4-flash"`.
    pub model_name: String,
    /// Reported score in basis points (0-10000) for `ScrapeCompleted`.
    /// 0 for other kinds. A source reporting `46%` lands as `4600`.
    pub score_pct_bp: u32,
    /// Sample size the score was computed over (number of problems in
    /// the suite). 0 for kinds other than `ScrapeCompleted`.
    pub sample_size: u32,
    /// blake3 8-byte hex prefix of the canonical row tuple for dedup.
    /// Empty string for non-`ScrapeCompleted` kinds.
    pub entry_hash: String,
    /// Snake_case model-pricing tier label for `ScrapeCompleted`
    /// (`"local"` / `"cheap-volume"` / `"sonnet"` / `"opus"`). Empty
    /// string for other kinds. Cited grounding:
    /// platform.claude.com/docs/en/about-claude/pricing (verified May
    /// 2026 -- Sonnet 4.6 $3/$15, Opus 4.7 $5/$25, Haiku 4.5 $1/$5).
    pub pricing_label: String,
    /// Reason label for `ScrapeFailed` (e.g. `"http_404"`,
    /// `"parse_error"`, `"env_gate_off"`, `"rate_limited"`). Empty
    /// string for other kinds.
    pub reason_label: String,
    /// HTTP status code for `ScrapeFailed` (0 if non-HTTP failure such
    /// as ENV-gate refusal or rate limit). 0 for other kinds.
    pub http_status: u16,
    /// Free-form per-kind context (e.g. raw `score_str` before parse,
    /// `arch_tag` snake_case label for downstream filtering, `caller_site`
    /// label, source `paper_arxiv_id`). Serialized as JSON so future
    /// kinds can extend without a schema bump.
    pub details_json: serde_json::Value,
    /// Fire-point wall-clock (millis since UNIX_EPOCH).
    pub ts_ms: i64,
}

/// E-31 -- env var that flips the coding_model_leaderboard_event family
/// from wire-dark to live emission. Match-string is `"1"` or `"true"`
/// (case-insensitive); any other value leaves the emitter as a noop.
/// Default-off so existing tests keep their expected SSB row counts.
pub const ELAI_CODING_MODEL_LEADERBOARD_ENABLE_ENV: &str =
    "ELAI_CODING_MODEL_LEADERBOARD_ENABLE";

// ---------------------------------------------------------------------------
// E-38 (SCHEMA_VERSION 175) -- 2026 AI roadmap signals + EU AI Act compliance
// pre-gate substrate
// ---------------------------------------------------------------------------

/// E-38 -- per-kind discriminator carried by [`AiRoadmapSignalEventPayload`].
///
/// Single SsbActionType variant + 5-arm discriminator per Pattern-2 (27th
/// application). Collapses 5 fire-points (compliance_blocked /
/// compliance_verified / mcp_transport_version_logged / mrcr_unverified /
/// posture_doc_stale) onto one wire variant so SCHEMA bump is exactly +1
/// (174 -> 175) and TOTAL_VARIANTS bump is exactly +1 (447 -> 448).
/// Wire-dark unless `ELAI_AI_ROADMAP_SIGNALS_ENABLE=1`.
///
/// Phase mapping:
///   - `ComplianceBlocked`: EU AI Act compliance pre-gate refused
///     PLANNING/FEASIBILITY_PROBE -> TESTING_FIRST transition. Payload
///     carries `assertion_label` + `hint` + `plan_session_id`.
///   - `ComplianceVerified`: pre-gate passed; payload carries
///     `plan_session_id` + `assertions_checked` bitmask + scalar 0s on
///     blocked-only fields.
///   - `McpTransportVersionLogged`: DefaultMcpBridge::new emitted a
///     transport-version beacon at construction. Payload carries
///     `mcp_transport_current` + `mcp_transport_target`.
///   - `MrcrUnverified`: a backend declared `declared_max_context_tokens`
///     >= 100_000 without a measured `mrcr_v2_score`. Payload carries
///     `backend_name` + `declared_context_tokens` + `mrcr_score_label`
///     (e.g. "vendor-claim-only").
///   - `PostureDocStale`: docs/strategy/ELAI-2026-POSTURE.md mtime is
///     older than 90 days. Payload carries `posture_doc_path` +
///     `posture_doc_mtime_ms` + `age_days`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RoadmapSignalKind {
    /// EU AI Act compliance pre-gate refused a PLANNING/FEASIBILITY_PROBE
    /// -> TESTING_FIRST transition.
    ComplianceBlocked,
    /// EU AI Act compliance pre-gate accepted the transition.
    ComplianceVerified,
    /// DefaultMcpBridge::new emitted a transport-version beacon at boot.
    McpTransportVersionLogged,
    /// A backend declared >= 100K context without a measured MRCR v2 score.
    MrcrUnverified,
    /// docs/strategy/ELAI-2026-POSTURE.md mtime is > 90 days old.
    PostureDocStale,
}

impl RoadmapSignalKind {
    /// Snake-case wire label.
    #[must_use]
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::ComplianceBlocked => "compliance_blocked",
            Self::ComplianceVerified => "compliance_verified",
            Self::McpTransportVersionLogged => "mcp_transport_version_logged",
            Self::MrcrUnverified => "mrcr_unverified",
            Self::PostureDocStale => "posture_doc_stale",
        }
    }
}

/// E-38 (SCHEMA_VERSION 175) -- payload for [`SsbActionType::AiRoadmapSignalEvent`].
///
/// Wire-dark default; emission gated by `ELAI_AI_ROADMAP_SIGNALS_ENABLE`
/// (read at construction time by the producers:
/// [`crate::compliance`] / [`crate::research::roadmap_signals`] +
/// [`elai_plugins::mcp_bridge::DefaultMcpBridge`]). Carries the 5-arm
/// discriminator + per-kind scalar fields + free-form `details_json` so
/// future kinds can extend without a schema bump. Empty strings + 0
/// numbers on fields not applicable to the current kind.
///
/// Cross-references:
///   * `compliance_blocked` populates `assertion_label` (one of
///     `clock_provider_intact` / `deterministic_seed` /
///     `template_hash_chain` / `audit_trail_live`) + `hint` +
///     `plan_session_id`.
///   * `compliance_verified` populates `plan_session_id` +
///     `assertions_checked` (bitmask: 0x1 clock | 0x2 seed | 0x4 template
///     | 0x8 audit_trail).
///   * `mcp_transport_version_logged` populates `mcp_transport_current`
///     (e.g. `streamable_http`, `sse_legacy`) + `mcp_transport_target`
///     (modelcontextprotocol.io/development/roadmap target string).
///   * `mrcr_unverified` populates `backend_name` +
///     `declared_context_tokens` + `mrcr_score_label`.
///   * `posture_doc_stale` populates `posture_doc_path` +
///     `posture_doc_mtime_ms` + `age_days`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AiRoadmapSignalEventPayload {
    /// Fire-point discriminator. See [`RoadmapSignalKind`].
    pub kind: RoadmapSignalKind,
    /// Plan session id (compliance arms). Empty for transport / mrcr /
    /// posture arms.
    pub plan_session_id: String,
    /// EU AI Act assertion label (compliance_blocked arm). One of
    /// `clock_provider_intact` / `deterministic_seed` /
    /// `template_hash_chain` / `audit_trail_live`. Empty for other kinds.
    pub assertion_label: String,
    /// Remediation hint (compliance_blocked arm). Empty for other kinds.
    pub hint: String,
    /// Bitmask of assertions checked (compliance_verified arm):
    /// 0x1 = clock_provider, 0x2 = seed, 0x4 = template, 0x8 = audit_trail.
    /// 0 for non-verified kinds.
    pub assertions_checked: u32,
    /// MCP transport label (mcp_transport_version_logged arm). One of
    /// `streamable_http` / `sse_legacy` / `stdio`. Empty for other kinds.
    pub mcp_transport_current: String,
    /// MCP transport target (mcp_transport_version_logged arm). Free text
    /// referencing modelcontextprotocol.io/development/roadmap. Empty for
    /// other kinds.
    pub mcp_transport_target: String,
    /// Backend identifier (mrcr_unverified arm). Empty for other kinds.
    pub backend_name: String,
    /// Declared max-context tokens (mrcr_unverified arm). 0 for other
    /// kinds.
    pub declared_context_tokens: u32,
    /// MRCR score label (mrcr_unverified arm). Typically
    /// `vendor-claim-only` per plan Open Question 2 (no canonical arXiv
    /// MRCR v2 baseline; cite arXiv:2409.12640 + Anthropic blog).
    pub mrcr_score_label: String,
    /// Posture doc path (posture_doc_stale arm). Empty for other kinds.
    pub posture_doc_path: String,
    /// Posture doc mtime millis since UNIX_EPOCH (posture_doc_stale arm).
    /// 0 for other kinds.
    pub posture_doc_mtime_ms: i64,
    /// Age in days (posture_doc_stale arm). 0 for other kinds.
    pub age_days: u32,
    /// Free-form per-kind context (e.g. failing-assertion details,
    /// transport-roadmap details, mrcr backend metadata). Serialized as
    /// JSON so future kinds can extend without a schema bump.
    pub details_json: serde_json::Value,
    /// Fire-point wall-clock (millis since UNIX_EPOCH).
    pub ts_ms: i64,
}

/// E-38 -- env var that flips the ai_roadmap_signal_event family from
/// wire-dark to live emission. Match-string is `"1"` or `"true"`
/// (case-insensitive); any other value leaves the emitter as a noop.
/// Default-off so existing tests keep their expected SSB row counts.
pub const ELAI_AI_ROADMAP_SIGNALS_ENABLE_ENV: &str = "ELAI_AI_ROADMAP_SIGNALS_ENABLE";

/// E-38 -- compliance assertion bitmask flags (compliance_verified arm).
pub const COMPLIANCE_BIT_CLOCK_PROVIDER: u32 = 0x1;
pub const COMPLIANCE_BIT_DETERMINISTIC_SEED: u32 = 0x2;
pub const COMPLIANCE_BIT_TEMPLATE_HASH_CHAIN: u32 = 0x4;
pub const COMPLIANCE_BIT_AUDIT_TRAIL_LIVE: u32 = 0x8;

/// E-38 -- posture-doc staleness threshold (90 days). Mtime older than
/// this triggers a `posture_doc_stale` SSB row at session start (or at
/// the next call to `crate::research::roadmap_signals::check_posture_doc_age`).
pub const POSTURE_DOC_STALE_DAYS: u32 = 90;

/// E-38 -- declared-max-context threshold (100K tokens). Backends
/// declaring >= this value MUST also publish a measured mrcr_v2_score
/// (or the label `vendor-claim-only`); otherwise emit an
/// `mrcr_unverified` SSB row at backend init.
pub const MRCR_UNVERIFIED_CONTEXT_THRESHOLD: u32 = 100_000;

// ---------------------------------------------------------------------------
// E-39 (SCHEMA_VERSION 176) -- cost + token economics types
// ---------------------------------------------------------------------------

/// E-39 -- 8-arm fire-point discriminator carried by
/// [`CostTokenEventPayload`]. Mirrors the
/// [`RewardLadderEventKind`] shape exactly (snake_case wire labels +
/// `as_str()` helper). Pattern-2 (28th application).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CostEventKind {
    /// Mid-task cooperative abort because `task_spend_usd_cents >=
    /// max_spend_per_task_cents`. Produced by
    /// [`crate::cost::cloud_abort::abort_task_on_cap_breach`].
    TaskAbortedBySpendCap,
    /// Rolling-window counter exceeded its limit. Produced by
    /// [`crate::cost::velocity_cap::VelocityCapEnforcer::enforce`].
    VelocityCapBreached,
    /// Local Qwen 3B verifier emitted Continue/Retry/Escalate verdict
    /// between L3 failure and HYBRID cloud escalation. Produced by
    /// [`crate::cost::automix_gate::verify_before_escalate`].
    AutomixVerifierVerdict,
    /// Pre-execution difficulty estimator + budget-remaining picked
    /// Local / Sonnet / Haiku / DeepSeekFlash / Opus. Produced by
    /// [`crate::cost::cost_router::route_for_difficulty_and_budget`].
    CostRouterDecision,
    /// Post-call actual / pre-call estimate ratio fell outside
    /// `[0.5, 2.0]`. Calibration feedback for the cost-estimator.
    CostEstimateInaccurate,
    /// Backend at >= 80% of daily quota; warn-only.
    FreeTierQuotaApproaching,
    /// Backend at >= 95% of daily quota; refuse + abort dispatch.
    FreeTierQuotaExceeded,
    /// User invoked `elai cloud override-cap` with typed-confirmation.
    /// Audit-trail composition with E-38 EU AI Act liveness gate.
    LayeredCapManualOverride,
}

impl CostEventKind {
    /// Snake-case wire label.
    #[must_use]
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::TaskAbortedBySpendCap => "task_aborted_by_spend_cap",
            Self::VelocityCapBreached => "velocity_cap_breached",
            Self::AutomixVerifierVerdict => "automix_verifier_verdict",
            Self::CostRouterDecision => "cost_router_decision",
            Self::CostEstimateInaccurate => "cost_estimate_inaccurate",
            Self::FreeTierQuotaApproaching => "free_tier_quota_approaching",
            Self::FreeTierQuotaExceeded => "free_tier_quota_exceeded",
            Self::LayeredCapManualOverride => "layered_cap_manual_override",
        }
    }
}

/// E-39 (SCHEMA_VERSION 176) -- payload for
/// [`SsbActionType::CostTokenEvent`].
///
/// Wire-dark default; emission gated by
/// `ELAI_COST_TOKEN_ECONOMICS_ENABLE` env var (see
/// [`crate::cost::ENV_COST_TOKEN_ECONOMICS_ENABLE`]). Carries the
/// 8-arm discriminator + session / task ids + free-form JSON details
/// so each kind's specific scalars (spent_usd_cents, cap_usd_cents,
/// window, count, limit, verdict, difficulty, p50_usd, p95_usd,
/// remaining_pct, override_amount, etc.) can ride along without per-
/// kind payload structs (Pattern-2 single-variant scope-lean).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CostTokenEventPayload {
    /// Fire-point discriminator. See [`CostEventKind`].
    pub kind: CostEventKind,
    /// Optional canonical session id (the E-39 substrate emits per-
    /// session for `cost_router_decision` / `velocity_cap_breached` /
    /// `task_aborted_by_spend_cap`).
    pub session_id: Option<String>,
    /// Optional canonical task id (finer-grained than session for
    /// per-call rows like `automix_verifier_verdict`).
    pub task_id: Option<String>,
    /// Free-form per-kind context. Serialized as JSON so future kinds
    /// can extend without a schema bump. Conventional fields per kind:
    ///   * task_aborted_by_spend_cap: spent_usd_cents, cap_usd_cents,
    ///     reason, last_action_type, role
    ///   * velocity_cap_breached: window, kind ("requests"|"usd_cents"),
    ///     count, limit
    ///   * automix_verifier_verdict: verdict, reason, retry_count
    ///   * cost_router_decision: difficulty, budget_remaining_usd,
    ///     chosen_backend, criticality, rationale
    ///   * cost_estimate_inaccurate: estimate_usd, actual_usd, ratio,
    ///     p50_usd, p95_usd
    ///   * free_tier_quota_approaching: backend, remaining_pct
    ///   * free_tier_quota_exceeded: backend
    ///   * layered_cap_manual_override: user_session_id,
    ///     override_amount_usd, confirmation_string_hash
    pub details_json: serde_json::Value,
    /// Fire-point wall-clock (millis since UNIX_EPOCH).
    pub ts_ms: i64,
}

/// E-39 -- env var name re-export for callers that prefer to reference
/// the gate via `ssb::ELAI_COST_TOKEN_ECONOMICS_ENABLE_ENV` rather than
/// importing the canonical const from [`crate::cost`].
pub const ELAI_COST_TOKEN_ECONOMICS_ENABLE_ENV: &str = "ELAI_COST_TOKEN_ECONOMICS_ENABLE";

// ---------------------------------------------------------------------------
// E-50 (SCHEMA_VERSION 177) -- SaaS architecture pattern catalog substrate.
// ---------------------------------------------------------------------------

/// E-50 -- per-kind discriminator carried by
/// [`SaasArchPatternEventPayload`]. Mirrors the
/// [`CostEventKind`] / [`PriorArtEventKind`] shape exactly (snake_case
/// wire labels + `as_str()` helper). Pattern-2 (29th application).
///
/// 12 fire-points covering the May 2026 SaaS architecture survey's
/// production-readiness gate + RLS emit + scoped-token enforcement +
/// MoR decision fork + realtime / async / rate-limit / OTel-SLO
/// scaffolding plus the Tranche B-7 triple `SaasArchetypeSelected`
/// extension (E-50 archetype taxonomy refresh -- closes the
/// "archetype_selected SSB absent" master-index framing without
/// a separate top-level variant). Closed enum; adding a 13th kind
/// requires bumping the schema again.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SaasArchPatternEventKind {
    /// One of the 6 day-1 readiness checks failed. Payload carries the
    /// failing check + reason in `details_json`.
    ProductionReadinessGateBlocked,
    /// All applicable day-1 readiness checks passed for the
    /// catalog-selected archetype.
    ProductionReadinessGatePassed,
    /// Designer emitted a per-table RLS baby step. Payload records the
    /// table name + blake3 hash of the generated policy text.
    RlsBabyStepEmitted,
    /// Scoped-token enforcer refused a deploy step using an unscoped
    /// CLI token (PocketOS Apr 2026 9s wipe defense).
    UnscopedTokenBlocked,
    /// (geography, mrr_projection) resolved to a MoR vendor selection
    /// (or Stripe-direct on US-only / `None` MoR).
    BillingMorDecisionMade,
    /// Designer fixed [`crate::architecture::saas_patterns::RealtimeNeed`]
    /// based on the idea-text keyword trigger. Default `None`.
    RealtimeBlockDecided,
    /// Latency budget + archetype resolved to a
    /// [`crate::architecture::saas_patterns::AsyncBackbone`] selection
    /// (Serverless / Inngest / TriggerDev / Modal / Hatchet).
    AsyncBackboneDecided,
    /// Billing tier names mapped to per-tier rate-limit rows
    /// (free=10rpm / pro=100rpm / business=1000rpm / enterprise=5000rpm
    /// + median fallback per E-50 plan).
    PerTierRateLimitDerived,
    /// Per-archetype OTel + SLO + runbook scaffold emitted with the
    /// OneUptime 2026 multi-burn-rate defaults (crit 14x / warn 6x /
    /// notice 1x).
    OtelSloRunbookScaffolded,
    /// Scoped-token enforcer fell back to comment-marker because the
    /// platform (Vercel / AWS) has no first-class scope-pinning field.
    PlatformUnsupportedForScopeCheck,
    /// Designer recorded a manual stack-layer override (vendor swap
    /// inside the chosen archetype; e.g. swap Neon for Supabase on
    /// `SaasDashboard`).
    ArchetypeBlockOverridden,
    /// Tranche B-7 triple (SCHEMA_VERSION 195) -- E-50 archetype taxonomy
    /// refresh: emitted by
    /// [`crate::architecture::recommender::recommend_archetype_with_emit`]
    /// when an archetype is selected (or auto-defaulted) for an
    /// idea-framing brief. Wire-dark unless
    /// `ELAI_E50_ARCHETYPE_SELECTED_EMIT_ENABLE=1`. `details_json` carries
    /// the selection source (recommender_high / recommender_medium /
    /// recommender_low / user_override / default) + confidence_bps
    /// (`u16`, 0..=10_000, avoids f32 round-trip drift per addendum D13)
    /// + selected_at_step. Closes the master-index "archetype_selected
    /// SSB absent" framing without adding a new top-level variant.
    SaasArchetypeSelected,
}

impl SaasArchPatternEventKind {
    /// Snake-case wire label.
    #[must_use]
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::ProductionReadinessGateBlocked => "production_readiness_gate_blocked",
            Self::ProductionReadinessGatePassed => "production_readiness_gate_passed",
            Self::RlsBabyStepEmitted => "rls_baby_step_emitted",
            Self::UnscopedTokenBlocked => "unscoped_token_blocked",
            Self::BillingMorDecisionMade => "billing_mor_decision_made",
            Self::RealtimeBlockDecided => "realtime_block_decided",
            Self::AsyncBackboneDecided => "async_backbone_decided",
            Self::PerTierRateLimitDerived => "per_tier_rate_limit_derived",
            Self::OtelSloRunbookScaffolded => "otel_slo_runbook_scaffolded",
            Self::PlatformUnsupportedForScopeCheck => "platform_unsupported_for_scope_check",
            Self::ArchetypeBlockOverridden => "archetype_block_overridden",
            Self::SaasArchetypeSelected => "saas_archetype_selected",
        }
    }
}

/// E-50 (SCHEMA_VERSION 177) -- payload for
/// [`SsbActionType::SaasArchPatternEvent`].
///
/// Wire-dark default; emission gated by
/// `ELAI_SAAS_ARCH_PATTERNS_ENABLE` env var (see
/// [`crate::architecture::saas_patterns::ELAI_SAAS_ARCH_PATTERNS_ENABLE_VAR`]).
/// Carries the 11-arm discriminator + session id + archetype label +
/// free-form JSON details so each kind's per-kind scalars (check name,
/// failing reason, MoR vendor, realtime trigger keyword, async backbone
/// selection, rate-limit tier values, SLO scaffold tweaks, deploy
/// platform, override layer, etc.) can ride along without per-kind
/// payload structs (Pattern-2 single-variant scope-lean).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SaasArchPatternEventPayload {
    /// Fire-point discriminator. See [`SaasArchPatternEventKind`].
    pub kind: SaasArchPatternEventKind,
    /// Optional canonical session id. Some kinds (e.g. catalog-defaults
    /// recomputation on cold start) can be emitted out-of-band.
    pub session_id: Option<String>,
    /// Archetype snake_case label (e.g. `"saas_dashboard"`,
    /// `"b2b_marketplace"`) the event is associated with. Mirrors the
    /// E-11 `SaasPriorArtEventPayload::archetype` shape.
    pub archetype: String,
    /// Free-form per-kind context. Serialized as JSON so future kinds
    /// can extend without a schema bump. Conventional fields per kind:
    ///   * production_readiness_gate_blocked: check (snake_case),
    ///     reason, caller_site
    ///   * production_readiness_gate_passed: applicable_checks (array),
    ///     skipped_checks (array)
    ///   * rls_baby_step_emitted: table, policy_blake3_hex,
    ///     baby_step_id
    ///   * unscoped_token_blocked: platform, env_var, attempted_verb,
    ///     opt_out_flag
    ///   * billing_mor_decision_made: vendor (or "stripe_direct"),
    ///     geography, mrr_projection
    ///   * realtime_block_decided: need (none / presence / crdt),
    ///     trigger_keyword
    ///   * async_backbone_decided: backbone, latency_budget, brief_hash
    ///   * per_tier_rate_limit_derived: tiers (array of
    ///     {tier, rpm})
    ///   * otel_slo_runbook_scaffolded: availability_bps,
    ///     p95_latency_ms, error_rate_bps, burn_factors
    ///   * platform_unsupported_for_scope_check: platform,
    ///     fallback_marker
    ///   * archetype_block_overridden: layer, original_vendor,
    ///     override_vendor, reason
    ///   * saas_archetype_selected: source (recommender_high /
    ///     recommender_medium / recommender_low / user_override /
    ///     default), confidence_bps (u16 in 0..=10_000), selected_at_step
    ///     (u32), brief_hash (blake3-prefix u64)
    pub details_json: serde_json::Value,
    /// Fire-point wall-clock (millis since UNIX_EPOCH).
    pub ts_ms: i64,
}

/// E-50 -- env var name re-export for callers that prefer to reference
/// the gate via `ssb::ELAI_SAAS_ARCH_PATTERNS_ENABLE_ENV` rather than
/// importing the canonical const from
/// [`crate::architecture::saas_patterns`].
pub const ELAI_SAAS_ARCH_PATTERNS_ENABLE_ENV: &str = "ELAI_SAAS_ARCH_PATTERNS_ENABLE";

// ---------------------------------------------------------------------------
// E-4 (SCHEMA_VERSION 178) -- chat UX critique substrate
// ---------------------------------------------------------------------------

/// E-4 -- per-kind discriminator carried by [`ChatUxCritiqueEventPayload`].
/// Mirrors the [`SaasArchPatternEventKind`] / [`RewardLadderEventKind`]
/// shape exactly (snake_case wire labels + `as_str()` helper).
/// Pattern-2 (30th application).
///
/// 13 fire-points covering the chat-UX critique report's top
/// trust-impact-per-effort fixes. Exceeds the prior empirical max of 11
/// arms (E-50) by 2; documented in workflow/reasoning/E-4-decisions.md
/// as a deliberate single-envelope decision that keeps the
/// `action_type_lock.golden` cascade to ONE ordinal insertion + ONE
/// schema bump (vs. 13 separate variants = 13 ordinal cascades + 13
/// schema bumps).
///
/// Closed enum; adding a 14th kind requires bumping the schema again.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ChatUxCritiqueKind {
    /// AUDITOR stage emitted a per-claim confidence pill
    /// ([`crate::chat_ux::ClaimWithConfidence`]). Details carry the tier
    /// label (verified / read_from_source / from_training / guessed) +
    /// optional `evidence_ref` (file_path:line / ssb_seq) + claim text
    /// blake3 hash for dedup.
    ClaimConfidenceTierEmitted,
    /// AUDITOR overrode the REASONER's stance based on user-disagreement
    /// evidence. Model held position with a citation rather than agreeing
    /// to be agreeable (canonical sycophancy anti-pattern). Details carry
    /// `evidence_ref` + `original_stance` + `user_assertion` (hashed).
    SycophancyPushbackDetected,
    /// User pinned an ad-hoc constraint to the PinnedRulesPanel surface.
    /// Details: `rule_text` (truncated for PII safety), `rule_blake3_hex`,
    /// `pinned_at_ms`.
    UserRulePinned,
    /// User removed a pinned constraint (click-to-remove on the panel).
    /// Details: `rule_blake3_hex` matching the prior pin event.
    UserRuleUnpinned,
    /// After N turns or before a destructive action, system prompt +
    /// first user message + active rules were re-injected. Details:
    /// `trigger` (turn_threshold / destructive_action / manual),
    /// `turn_count`, `active_rules_count`.
    SessionReAnchored,
    /// DoomLoopDetector fired. Details: `pattern` (spinning / oscillation
    /// / no_drift / diminishing_returns), `attempt_count`, `file_or_topic`.
    DoomLoopDetected,
    /// AUDITOR rejected a "done" / "complete" / "finished" output with no
    /// evidence_ref. Details: `claim_text_blake3_hex`, `missing_evidence`
    /// (one of: file_diff / audit_verdict / det_check).
    CompletionClaimRejectedNoEvidence,
    /// TRIAGE stage emitted exactly ONE clarifier card. Details:
    /// `ambiguity_score`, `dimensions_violated` (count), `question_count`.
    OneShotClarifierEmitted,
    /// User answered the clarifier; pipeline proceeds. Details:
    /// `clarifier_id`, `latency_ms`.
    OneShotClarifierResolved,
    /// ELAI refused an action. Details: `refusal_reason`, `refusal_scope`
    /// (`local_codebase` -> overrideable; `outside_project` -> hard
    /// refuse).
    RefusalEmitted,
    /// User clicked the refusal-override button; action re-runs with the
    /// named rule bypassed for that single dispatch. Details:
    /// `refusal_id`, `rule_bypassed`.
    RefusalOverriddenByUser,
    /// User typed a correction mid-stream; in-flight stream was cancelled
    /// (BoxStream drop) and conversation restarted. Details:
    /// `bytes_streamed_before_cancel`, `correction_blake3_hex`.
    MidStreamInterruptCommitted,
    /// ELAI explicitly retracted a prior claim with the canonical
    /// "I was wrong. Correct: X because Y." pattern. Details:
    /// `original_claim_blake3_hex`, `correction_claim_blake3_hex`,
    /// `trigger` (test_failed / doc_disagreed / user_corrected).
    HonestCorrectionEmitted,
}

impl ChatUxCritiqueKind {
    /// Snake-case wire label.
    #[must_use]
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::ClaimConfidenceTierEmitted => "claim_confidence_tier_emitted",
            Self::SycophancyPushbackDetected => "sycophancy_pushback_detected",
            Self::UserRulePinned => "user_rule_pinned",
            Self::UserRuleUnpinned => "user_rule_unpinned",
            Self::SessionReAnchored => "session_re_anchored",
            Self::DoomLoopDetected => "doom_loop_detected",
            Self::CompletionClaimRejectedNoEvidence => "completion_claim_rejected_no_evidence",
            Self::OneShotClarifierEmitted => "one_shot_clarifier_emitted",
            Self::OneShotClarifierResolved => "one_shot_clarifier_resolved",
            Self::RefusalEmitted => "refusal_emitted",
            Self::RefusalOverriddenByUser => "refusal_overridden_by_user",
            Self::MidStreamInterruptCommitted => "mid_stream_interrupt_committed",
            Self::HonestCorrectionEmitted => "honest_correction_emitted",
        }
    }
}

/// E-4 (SCHEMA_VERSION 178) -- payload for
/// [`SsbActionType::ChatUxCritiqueEvent`].
///
/// Wire-dark default; emission gated by `ELAI_CHAT_UX_CRITIQUE_ENABLE`
/// env var (see [`ELAI_CHAT_UX_CRITIQUE_ENABLE_ENV`]).
///
/// Carries the 13-arm discriminator + session id + optional turn id +
/// free-form JSON details so each kind's specific scalars (claim
/// tier label, evidence ref, refusal reason, doom-loop pattern,
/// ambiguity score, correction trigger, etc.) ride along without
/// per-kind payload structs (Pattern-2 single-variant scope-lean).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ChatUxCritiqueEventPayload {
    /// Fire-point discriminator. See [`ChatUxCritiqueKind`].
    pub kind: ChatUxCritiqueKind,
    /// Optional canonical session id (some kinds, e.g. cold-start
    /// catalog defaults, can be emitted out-of-band).
    pub session_id: Option<String>,
    /// Optional turn id within the session (None for kinds that span
    /// multiple turns, e.g. `SessionReAnchored` or `DoomLoopDetected`).
    pub turn_id: Option<String>,
    /// Free-form per-kind context. Conventional fields documented on
    /// each [`ChatUxCritiqueKind`] variant docstring. Serialized as
    /// JSON so future kinds extend without a schema bump.
    pub details_json: serde_json::Value,
    /// Fire-point wall-clock (millis since UNIX_EPOCH).
    pub ts_ms: i64,
}

/// E-4 -- env var name re-export. Wire-dark gate; emission of
/// [`SsbActionType::ChatUxCritiqueEvent`] rows requires this env var
/// equal to `"1"` at the producer's call site.
pub const ELAI_CHAT_UX_CRITIQUE_ENABLE_ENV: &str = "ELAI_CHAT_UX_CRITIQUE_ENABLE";

// ---------------------------------------------------------------------------
// E-16 (SCHEMA_VERSION 179) -- VLM image validation hardening substrate.
// ---------------------------------------------------------------------------

/// E-16 -- 3-arm fire-point discriminator carried by
/// [`VlmImageValidationEventPayload`]. Mirrors the
/// [`ChatUxCritiqueKind`] / [`CostEventKind`] / [`SaasArchPatternEventKind`]
/// shape exactly (snake_case wire labels + `as_str()` helper).
/// Pattern-2 (31st application).
///
/// 3 fire-points covering the E-16 plan's deterministic-check
/// requirements. Notably ABSENT from this enum is a
/// `SycophancyFlag` arm: that fire-point reuses the pre-existing
/// [`SsbActionType::SycophancyFlag`] (E-4 / D-14) so the dashboard's
/// SycophancyFlagStrip / SycophancyGuardBadge surfaces keep
/// rendering without a duplicate variant. Closed enum; adding a 4th
/// kind requires bumping the schema again.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum VlmImageValidationKind {
    /// Pairwise Tier-2 judge returned disagreement when the same
    /// candidate pair was presented in swapped order [A,B] vs [B,A]
    /// (arXiv:2406.07791 position-bias). Veto pathway routes the
    /// request to HumanReview instead of accepting the agreement.
    /// `details_json` carries `judge_a_verdict`, `judge_b_verdict`,
    /// `agreement_required: true`.
    SwapOrderDisagreement,
    /// AXTree YAML structural snapshot
    /// ([Playwright `page.ariaSnapshot()` v1.59+](https://playwright.dev/docs/aria-snapshots))
    /// diverged from the baseline beyond the deterministic-drift
    /// threshold (default 1% per E-16 plan open Q4). `details_json`
    /// carries `yaml_diff_pct: f64`, `baseline_path`, `current_path`.
    AxtreeMismatch,
    /// Tier-2 VLM JSON output failed the XGrammar schema-mask check
    /// (arXiv:2411.15100). The SGLang fallback path emitted free-form
    /// JSON that did not validate against the
    /// [`VlmValidationOutput`](super::media_validator::types::VlmValidationOutput)
    /// schema. `details_json` carries `schema_id`, `violation_kind`.
    VlmSchemaUnconstrained,
}

impl VlmImageValidationKind {
    /// Snake-case wire label.
    #[must_use]
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::SwapOrderDisagreement => "swap_order_disagreement",
            Self::AxtreeMismatch => "axtree_mismatch",
            Self::VlmSchemaUnconstrained => "vlm_schema_unconstrained",
        }
    }
}

/// E-16 (SCHEMA_VERSION 179) -- payload for
/// [`SsbActionType::VlmImageValidationEvent`].
///
/// Wire-dark default; emission gated by
/// `ELAI_VLM_IMAGE_VALIDATION_ENABLE` env var (see
/// [`ELAI_VLM_IMAGE_VALIDATION_ENABLE_ENV`]). Carries the 3-arm
/// discriminator + step / request ids + free-form JSON details so
/// each kind's per-kind scalars (judge verdicts, YAML diff pct,
/// schema violation kind) ride along without per-kind payload structs
/// (Pattern-2 single-variant scope-lean).
///
/// Does NOT derive `Eq` — `details_json` may contain NaN-not-equal
/// `f64` values (e.g. `yaml_diff_pct`) per Pattern-2 28th precedent.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct VlmImageValidationEventPayload {
    /// Fire-point discriminator. See [`VlmImageValidationKind`].
    pub kind: VlmImageValidationKind,
    /// Pipeline step id; mirrors `MediaValidationEntry::step_id` on
    /// the dashboard side so the fold lands on the right row.
    pub step_id: String,
    /// Optional canonical request id (uuid stringified). Some kinds
    /// (e.g. `AxtreeMismatch` emitted during render-only health
    /// probes) can be emitted without a request envelope.
    pub request_id: Option<String>,
    /// Free-form per-kind context. Conventional fields per kind:
    ///   * swap_order_disagreement: judge_a_verdict, judge_b_verdict,
    ///     agreement_required, divergence_axis
    ///   * axtree_mismatch: yaml_diff_pct, baseline_path, current_path,
    ///     threshold_pct
    ///   * vlm_schema_unconstrained: schema_id, violation_kind,
    ///     raw_output_blake3_hex
    pub details_json: serde_json::Value,
    /// Fire-point wall-clock (millis since UNIX_EPOCH).
    pub ts_ms: i64,
}

/// E-16 -- env var name re-export. Wire-dark gate; emission of
/// [`SsbActionType::VlmImageValidationEvent`] rows requires this env
/// var equal to `"1"` at the producer's call site.
pub const ELAI_VLM_IMAGE_VALIDATION_ENABLE_ENV: &str = "ELAI_VLM_IMAGE_VALIDATION_ENABLE";

// ---------------------------------------------------------------------------
// E-20 (SCHEMA_VERSION 180) -- local-cloud routing substrate.
// ---------------------------------------------------------------------------

/// E-20 -- 9-arm fire-point discriminator carried by
/// [`LocalCloudRoutingEventPayload`]. Mirrors the
/// [`CostEventKind`] / [`SaasArchPatternEventKind`] /
/// [`ChatUxCritiqueKind`] / [`VlmImageValidationKind`] shape exactly
/// (snake_case wire labels + `as_str()` helper). Pattern-2 (32nd
/// application).
///
/// 9 fire-points covering the E-20 plan's substrate:
///
///   * `confidence_emitted` -- L1 inference emitted a
///     `confidence_score` (entropy-of-first-token-derived). Details
///     carry `score: f32 in [0, 1]`, `entropy_nats: f32`,
///     `temperature: f32`, `model: String`.
///   * `hybrid_escalation` -- HybridEscalationGate verdict (composes
///     with [`CostEventKind::AutomixVerifierVerdict`] from E-39's
///     [`crate::cost::automix_gate::verify_before_escalate`]). Details
///     carry `verdict: skip_cloud|escalate_cloud|escalate_opus`,
///     `confidence_score: f32`, `threshold: f32`,
///     `reason: confidence|retry_exhausted|privacy_block`,
///     `l_layer: u8` (1-5).
///   * `difficulty_classified` -- pre-execution difficulty classifier
///     emitted easy/medium/hard. Details carry
///     `difficulty: easy|medium|hard`, `score: f32 in [0, 1]`,
///     `cloud_recommended: bool`, `classifier: routellm|heuristic`.
///   * `privacy_blocked` -- secret-scan refused a cloud dispatch.
///     Details carry `match_count: u32`, `severity: warn|block`,
///     `patterns: Vec<String>` (redacted pattern labels, NOT the
///     matched secret values), `cap_period: "policy"`.
///   * `output_filtered` -- post-cloud response was redacted before
///     display. Details carry `redaction_count: u32`,
///     `patterns: Vec<String>`, `output_blake3_hex: String`.
///   * `burn_alert` -- daily-burn-rate exceeded 2x baseline OR
///     projected-monthly exceeded ceiling. Details carry
///     `alert_kind: daily_2x|projected_monthly`,
///     `current_usd: f64`, `baseline_usd: f64`,
///     `projection_usd: f64`, `ceiling_usd: f64`.
///   * `reasoning_mode` -- semantic_reasoning_mode_chooser picked
///     whether to invoke extended-thinking on cloud. Details carry
///     `mode: on|off`, `difficulty: easy|medium|hard`,
///     `task_shape: execution|architecture|math`.
///   * `majority_vote` -- sample-of-3 routing decision tally. Details
///     carry `samples: u8` (always 3 today),
///     `tally: { local: u8, sonnet: u8, opus: u8, ... }`,
///     `winner: String`, `flapping: bool`.
///   * `role_priority_chosen` -- PerRoleRuntime priority +
///     task_shape_hint observed at dispatch. Details carry
///     `role: String`, `priority: cost|latency|quality`,
///     `task_shape: execution|architecture|math`,
///     `backend: String`.
///
/// Closed enum; adding a 10th kind requires bumping the schema again.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RoutingEventKind {
    ConfidenceEmitted,
    HybridEscalation,
    DifficultyClassified,
    PrivacyBlocked,
    OutputFiltered,
    BurnAlert,
    ReasoningMode,
    MajorityVote,
    RolePriorityChosen,
}

impl RoutingEventKind {
    /// Snake-case wire label.
    #[must_use]
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::ConfidenceEmitted => "confidence_emitted",
            Self::HybridEscalation => "hybrid_escalation",
            Self::DifficultyClassified => "difficulty_classified",
            Self::PrivacyBlocked => "privacy_blocked",
            Self::OutputFiltered => "output_filtered",
            Self::BurnAlert => "burn_alert",
            Self::ReasoningMode => "reasoning_mode",
            Self::MajorityVote => "majority_vote",
            Self::RolePriorityChosen => "role_priority_chosen",
        }
    }
}

/// E-20 (SCHEMA_VERSION 180) -- payload for
/// [`SsbActionType::LocalCloudRoutingEvent`].
///
/// Wire-dark default; emission gated by
/// `ELAI_LOCAL_CLOUD_ROUTING_ENABLE` env var (see
/// [`crate::cost::cloud_route::ENV_LOCAL_CLOUD_ROUTING_ENABLE`]).
/// Carries the 9-arm discriminator + session / task ids + free-form
/// JSON details so each kind's per-kind scalars (entropy, threshold,
/// verdict, redaction count, burn USD, vote tally, role+priority)
/// ride along without per-kind payload structs (Pattern-2 single-
/// variant scope-lean).
///
/// Does NOT derive `Eq` -- `details_json` may carry NaN-not-equal
/// `f64` / `f32` values (entropy score / threshold / burn USD) per
/// the Pattern-2 28th + 31st precedents.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LocalCloudRoutingEventPayload {
    /// Fire-point discriminator. See [`RoutingEventKind`].
    pub kind: RoutingEventKind,
    /// Optional canonical session id.
    pub session_id: Option<String>,
    /// Optional canonical task id (finer-grained than session for
    /// per-call rows like `confidence_emitted` /
    /// `majority_vote` / `hybrid_escalation`).
    pub task_id: Option<String>,
    /// Free-form per-kind context. Serialized as JSON so future kinds
    /// can extend without a schema bump. Conventional fields per kind:
    /// see [`RoutingEventKind`] docstrings for the canonical shape.
    pub details_json: serde_json::Value,
    /// Fire-point wall-clock (millis since UNIX_EPOCH).
    pub ts_ms: i64,
}

/// E-20 -- env var name re-export for callers that prefer to reference
/// the gate via `ssb::ELAI_LOCAL_CLOUD_ROUTING_ENABLE_ENV` rather than
/// importing the canonical const from
/// [`crate::cost::cloud_route`].
pub const ELAI_LOCAL_CLOUD_ROUTING_ENABLE_ENV: &str = "ELAI_LOCAL_CLOUD_ROUTING_ENABLE";

// ---------------------------------------------------------------------------
// E-21 (SCHEMA_VERSION 181) -- peer-systems harness substrate.
// ---------------------------------------------------------------------------

/// E-21 -- 4-arm fire-point discriminator carried by
/// [`PeerSystemsEventPayload`]. Mirrors the
/// [`RoutingEventKind`] / [`ChatUxCritiqueKind`] /
/// [`SkillEventPhase`] shape (snake_case wire labels + `as_str()`
/// helper). Pattern-2 (33rd application).
///
/// 4 fire-points covering the E-21 plan's inference-time critic-
/// rerank pipeline (`crates/orchestrator/src/peer_systems/critic_rerank.rs`):
///
///   * `critic_candidates_generated` -- N candidates produced by the
///     parallel-completion fan-out. `details_json` carries
///     `n: u8`, `runtimes: Vec<String>`, `task_id: String`.
///   * `critic_score_computed` -- one critic score emitted for one
///     candidate. `details_json` carries `score: f64 in [0, 1]`,
///     `runtime_label: String`, `critic_runtime: String`,
///     `candidate_idx: u8`.
///   * `critic_pick_committed` -- argmax pick committed downstream.
///     `details_json` carries `winner_idx: u8`, `winner_score: f64`,
///     `winner_runtime: String`, `runner_up_score: Option<f64>`.
///   * `critic_rerank_skipped` -- N=1 fallback fired (only one
///     runtime available OR sidecar reported single worker).
///     `details_json` carries `reason: String`
///     (`single_runtime` / `sidecar_single_worker` / `disabled`),
///     `available_runtimes: Vec<String>`.
///
/// Closed enum; adding a 5th kind requires bumping the schema.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PeerSystemsEventKind {
    /// N candidates generated this round.
    CriticCandidatesGenerated,
    /// One critic score computed for one candidate.
    CriticScoreComputed,
    /// Argmax pick committed downstream.
    CriticPickCommitted,
    /// N=1 fallback fired (single runtime / disabled).
    CriticRerankSkipped,
}

impl PeerSystemsEventKind {
    /// Snake-case wire label.
    #[must_use]
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::CriticCandidatesGenerated => "critic_candidates_generated",
            Self::CriticScoreComputed => "critic_score_computed",
            Self::CriticPickCommitted => "critic_pick_committed",
            Self::CriticRerankSkipped => "critic_rerank_skipped",
        }
    }
}

/// E-21 (SCHEMA_VERSION 181) -- payload for
/// [`SsbActionType::PeerSystemsEvent`].
///
/// Wire-dark default; emission gated by `ELAI_PEER_SYSTEMS_ENABLE` env
/// var (see [`crate::peer_systems::ELAI_PEER_SYSTEMS_ENABLE_ENV`]).
/// Carries the 4-arm discriminator + session / task ids + free-form
/// JSON details so each kind's per-kind scalars (score, n,
/// runtime_label, winner_idx, reason) ride along without per-kind
/// payload structs (Pattern-2 single-variant scope-lean).
///
/// Does NOT derive `Eq` -- `details_json` carries `f64` score / weight
/// values that violate `Eq` (NaN-not-equal). Mirrors the E-20 / E-39 /
/// E-50 / E-4 precedent.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PeerSystemsEventPayload {
    /// Fire-point discriminator. See [`PeerSystemsEventKind`].
    pub kind: PeerSystemsEventKind,
    /// Optional canonical session id.
    pub session_id: Option<String>,
    /// Optional canonical task id (finer-grained than session for
    /// per-call rows like `critic_score_computed`).
    pub task_id: Option<String>,
    /// Free-form per-kind context. Serialized as JSON so future kinds
    /// can extend without a schema bump. Conventional fields per kind:
    /// see [`PeerSystemsEventKind`] docstrings for the canonical shape.
    pub details_json: serde_json::Value,
    /// Fire-point wall-clock (millis since UNIX_EPOCH).
    pub ts_ms: i64,
}

/// E-21 -- env var name re-export for callers that prefer to reference
/// the gate via `ssb::ELAI_PEER_SYSTEMS_ENABLE_ENV` rather than
/// importing the canonical const from [`crate::peer_systems`].
pub const ELAI_PEER_SYSTEMS_ENABLE_ENV: &str = "ELAI_PEER_SYSTEMS_ENABLE";

// ---------------------------------------------------------------------------
// E-36 (SCHEMA_VERSION 182) -- document + vision RAG substrate.
// ---------------------------------------------------------------------------

/// E-36 -- 5-arm fire-point discriminator carried by
/// [`DocumentVisionRagEventPayload`]. Mirrors the
/// [`RoutingEventKind`] / [`PeerSystemsEventKind`] shape (snake_case
/// wire labels + `as_str()` helper). Pattern-2 (34th application).
///
/// 5 fire-points covering the E-36 plan's substrate:
///
///   * `vision_chunk_emitted` — one VisionArtifact row produced by the
///     `ImageArtifactChunker`. `details_json` carries
///     `source_path: String`, `kind: String` (page/screenshot/diagram),
///     `content_hash_hex: String`, `byte_size: u64`,
///     `pooled: Option<bool>`, `multi_vec_count: Option<usize>`.
///   * `vision_fallback_triggered` — at least one page in a
///     `PdfStructuredOutput` had `parser_confidence` below the
///     configured `cloud_fallback_threshold` (default 0.7).
///     `details_json` carries `source_path: String`,
///     `min_confidence: f32`, `threshold: f32`, `page_count: u32`,
///     `cloud_eligible: bool`.
///   * `vision_rag_policy_decision` — per-project enable/disable verdict
///     for the vision-RAG path. `details_json` carries
///     `project_id: String`, `enabled: bool`,
///     `storage_budget_mb: u64`, `actual_storage_mb: Option<u64>`.
///   * `visual_artifact_check_verdict` — outcome of a
///     `VerificationMethod::VisualArtifactCheck` dispatch via the
///     `VisualVerifier`. `details_json` carries
///     `claim: String`, `verdict: String` (confirm/refute/
///     indeterminate), `reliability: String` (vlm_caption/ocr_parse/
///     diagram_source), `evidence_page_count: usize`.
///   * `doc_parser_cloud_fallback_blocked_by_mode` — `--mode local`
///     refused a cloud parser escalation. `details_json` carries
///     `source_path: String`, `attempted_backend: String`
///     (mistral_cloud/reducto_cloud/llamaparse_cloud),
///     `min_confidence: f32`.
///
/// Closed enum; adding a 6th kind requires bumping the schema again.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DocumentVisionRagEventKind {
    VisionChunkEmitted,
    VisionFallbackTriggered,
    VisionRagPolicyDecision,
    VisualArtifactCheckVerdict,
    DocParserCloudFallbackBlockedByMode,
}

impl DocumentVisionRagEventKind {
    /// Snake-case wire label.
    #[must_use]
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::VisionChunkEmitted => "vision_chunk_emitted",
            Self::VisionFallbackTriggered => "vision_fallback_triggered",
            Self::VisionRagPolicyDecision => "vision_rag_policy_decision",
            Self::VisualArtifactCheckVerdict => "visual_artifact_check_verdict",
            Self::DocParserCloudFallbackBlockedByMode => {
                "doc_parser_cloud_fallback_blocked_by_mode"
            }
        }
    }
}

/// E-36 (SCHEMA_VERSION 182) -- payload for
/// [`SsbActionType::DocumentVisionRagEvent`].
///
/// Wire-dark default; emission gated by
/// `ELAI_DOCUMENT_VISION_RAG_ENABLE` env var (see
/// [`ELAI_DOCUMENT_VISION_RAG_ENABLE_ENV`]). Carries the 5-arm
/// discriminator + session / task ids + free-form JSON details so each
/// kind's per-kind scalars ride along without per-kind payload structs
/// (Pattern-2 single-variant scope-lean).
///
/// Does NOT derive `Eq` — `details_json` may carry NaN-not-equal
/// `f32` (parser_confidence) per the Pattern-2 28th / 31st precedents.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DocumentVisionRagEventPayload {
    /// Fire-point discriminator. See [`DocumentVisionRagEventKind`].
    pub kind: DocumentVisionRagEventKind,
    /// Optional canonical session id.
    pub session_id: Option<String>,
    /// Optional canonical task id (finer-grained than session).
    pub task_id: Option<String>,
    /// Free-form per-kind context. Serialized as JSON so future kinds
    /// can extend without a schema bump. Conventional fields per kind:
    /// see [`DocumentVisionRagEventKind`] docstrings for the canonical
    /// shape.
    pub details_json: serde_json::Value,
    /// Fire-point wall-clock (millis since UNIX_EPOCH).
    pub ts_ms: i64,
}

/// E-36 -- env var name. Wire-dark default: emission silently skipped
/// until the operator sets this to a truthy value (`"1"` / `"true"` /
/// `"yes"`). Mirrors the precedent set by
/// `ELAI_PEER_SYSTEMS_ENABLE` / `ELAI_LOCAL_CLOUD_ROUTING_ENABLE` /
/// `ELAI_SKILL_SYSTEM_ENABLE`.
pub const ELAI_DOCUMENT_VISION_RAG_ENABLE_ENV: &str = "ELAI_DOCUMENT_VISION_RAG_ENABLE";

// ---------------------------------------------------------------------------
// E-42 (SCHEMA_VERSION 183) -- tree-sitter grammar health substrate.
// ---------------------------------------------------------------------------

/// E-42 -- 4-arm fire-point discriminator carried by
/// [`TreeSitterGrammarEventPayload`]. Mirrors the
/// [`DocumentVisionRagEventKind`] / [`PeerSystemsEventKind`] shape
/// (snake_case wire labels + `as_str()` helper). Pattern-2 (35th
/// application).
///
/// 4 fire-points covering the E-42 plan's tree-sitter substrate. Each
/// fire-point's `details_json` shape is documented below; the
/// canonical producer for each lives in
/// `elai_knowledge::tree_sitter_health::*`.
///
///   * `abi_probe_mismatch` -- a registered grammar reported an ABI
///     version outside `[14, 15]`. Producer:
///     `elai_knowledge::tree_sitter_health::abi_probe::probe_abi_versions`.
///     `details_json` carries `language_id: String`, `version: usize`,
///     `expected_min: usize`, `expected_max: usize`. Probes that panic
///     surface here with `version: 0`.
///   * `svelte_runes_parse_health` -- one .svelte file was parsed and
///     its ERROR / MISSING node counts + Svelte 5 runes flag are
///     surfaced for telemetry. Producer:
///     `elai_knowledge::tree_sitter_health::svelte_health::compute_health` +
///     `::classify`. `details_json` carries `file: String`,
///     `error_node_count: usize`, `missing_node_count: usize`,
///     `total_node_count: usize`, `runes_detected: bool`,
///     `classification: String` (healthy / fallback_raw_text /
///     no_runes_healthy / no_runes_degraded).
///   * `tier_b_grammar_loaded` -- a Tier-B grammar registered
///     successfully under the `experimental_grammars` Cargo feature.
///     Producer:
///     `elai_knowledge::tree_sitter_health::registration::register_tier_b_languages`.
///     `details_json` carries `language_id: String`,
///     `crate_name: String`. Operators see one row per Tier-B grammar
///     that loaded; failed Tier-B grammars surface via the existing
///     `LanguageGrammarStatus` payload (B-30 SCHEMA 36).
///   * `tree_sitter_skipped_large_file` -- a source file exceeded the
///     2 MiB ceiling and was downgraded to raw-text chunking.
///     Producer:
///     `elai_knowledge::tree_sitter_health::large_file::should_skip_for_size`
///     + LargeFileSkip emission at the DkgEngine source-walk path.
///     `details_json` carries `relative_path: String`,
///     `byte_count: u64`, `threshold_bytes: u64`.
///
/// Closed enum; adding a 5th kind requires bumping the schema again.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TreeSitterGrammarEventKind {
    /// Grammar reported ABI outside `[14, 15]` (or probe panicked).
    AbiProbeMismatch,
    /// .svelte parse health snapshot (runes + ERROR / MISSING counts).
    SvelteRunesParseHealth,
    /// Tier-B grammar registered under `experimental_grammars` feature.
    TierBGrammarLoaded,
    /// Source file above 2 MiB ceiling; raw-text fallback used.
    TreeSitterSkippedLargeFile,
}

impl TreeSitterGrammarEventKind {
    /// Snake-case wire label.
    #[must_use]
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::AbiProbeMismatch => "abi_probe_mismatch",
            Self::SvelteRunesParseHealth => "svelte_runes_parse_health",
            Self::TierBGrammarLoaded => "tier_b_grammar_loaded",
            Self::TreeSitterSkippedLargeFile => "tree_sitter_skipped_large_file",
        }
    }
}

/// E-42 (SCHEMA_VERSION 183) -- payload for
/// [`SsbActionType::TreeSitterGrammarEvent`].
///
/// Wire-dark default; emission gated by
/// `ELAI_TREE_SITTER_GRAMMARS_ENABLE` env var (see
/// [`ELAI_TREE_SITTER_GRAMMARS_ENABLE_ENV`]). Carries the 4-arm
/// discriminator + optional file path + free-form JSON details so each
/// kind's per-kind scalars (ABI version, ERROR / MISSING counts,
/// crate name, byte count) ride along without per-kind payload structs
/// (Pattern-2 single-variant scope-lean).
///
/// Does NOT derive `Eq` -- `details_json` may carry NaN-not-equal
/// `f32` values (e.g. error rate ratios in Svelte health detail
/// blobs) per the Pattern-2 28th / 31st / 34th precedents.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TreeSitterGrammarEventPayload {
    /// Fire-point discriminator. See [`TreeSitterGrammarEventKind`].
    pub kind: TreeSitterGrammarEventKind,
    /// Optional project-relative path of the file the event concerns.
    /// `None` for kinds that are file-agnostic (e.g.
    /// `AbiProbeMismatch`, `TierBGrammarLoaded`).
    pub file: Option<String>,
    /// Free-form per-kind context. Serialized as JSON so future kinds
    /// can extend without a schema bump. Conventional fields per kind:
    /// see [`TreeSitterGrammarEventKind`] docstrings for the canonical
    /// shape.
    pub details_json: serde_json::Value,
    /// Fire-point wall-clock (millis since UNIX_EPOCH).
    pub ts_ms: i64,
}

/// E-42 -- env var name. Wire-dark default: emission silently skipped
/// until the operator sets this to a truthy value (`"1"` / `"true"` /
/// `"yes"`). Mirrors the precedent set by
/// `ELAI_DOCUMENT_VISION_RAG_ENABLE` / `ELAI_PEER_SYSTEMS_ENABLE` /
/// `ELAI_LOCAL_CLOUD_ROUTING_ENABLE`. Re-exported here so callers can
/// reference the gate via `ssb::ELAI_TREE_SITTER_GRAMMARS_ENABLE_ENV`
/// without importing the canonical const from
/// [`elai_knowledge::tree_sitter_health::ELAI_TREE_SITTER_GRAMMARS_ENABLE_ENV`].
pub const ELAI_TREE_SITTER_GRAMMARS_ENABLE_ENV: &str = "ELAI_TREE_SITTER_GRAMMARS_ENABLE";

// ---------------------------------------------------------------------------
// E-46 (SCHEMA_VERSION 184) -- planning-decomposition substrate.
// ---------------------------------------------------------------------------

/// E-46 -- 8-arm fire-point discriminator carried by
/// [`PlanningDecompositionEventPayload`]. Mirrors the
/// [`RoutingEventKind`] / [`ChatUxCritiqueKind`] /
/// [`TreeSitterGrammarEventKind`] shape (snake_case wire labels +
/// `as_str()` helper). Pattern-2 (36th application).
///
/// 8 fire-points covering the E-46 plan's planning-decomposition
/// substrate (`crates/orchestrator/src/planning_decomposition/`):
///
///   * `reflection_appended` -- one reflection pushed into a per-node
///     [`crate::planning_decomposition::reflection_buffer::ReflectionBuffer`].
///     `details_json` carries `node_key: String`, `trust_tier: String`,
///     `source_event_id: String` (blake3 hex), `text_len: u32`,
///     `created_at_unix: u64`.
///   * `low_entropy_decomposition` -- decomposition fork with
///     `H_children / H_parent < ENTROPY_RATIO_THRESHOLD`. `details_json`
///     carries `parent_node_key`, `entropy_ratio: f32`, `h_parent: f32`,
///     `h_children: f32`, `threshold: f32`, `verdict: String`.
///   * `goal_drift_canary_below_threshold` -- phase below jaccard OR
///     cosine threshold. `details_json` carries `phase_node_key`,
///     `jaccard: f32`, `cosine: f32`, `prev_below_threshold: bool`,
///     `fire_cascade: bool`. BOTH jaccard AND cosine populated by
///     struct shape (stub_ban §89 enforced at type level).
///   * `bubble_up_replan_phase` -- the same BabyStep failed twice OR
///     goal-drift canary fired twice consecutively. `details_json`
///     carries `phase_node_key`, `failed_plan: String`,
///     `error_transcript: String`, `original_goal: String`. ALL THREE
///     fields are validated non-empty by the producer
///     (`SsbError::PayloadValidation` on violation; stub_ban §88).
///   * `bubble_up_ledger_reset` -- Magentic-One progress-ledger blocker
///     count crossed [`crate::planning_decomposition::dual_ledger_ext::LEDGER_RESET_BLOCKER_THRESHOLD`].
///     `details_json` carries `concept_or_phase_id: String`,
///     `blocker_count: u32`, `threshold: u32`.
///   * `agentless_shortcut_dispatched` -- a Step level was dispatched
///     through the AGENTLESS 3-phase pipeline (localize / repair /
///     validate) rather than the standard BabyStep decomposition path.
///     `details_json` carries `step_node_key: String`, `reason: String`.
///   * `process_reward_score_emitted` -- LATS-style deterministic
///     scalar score in `[0.0, 1.0]` emitted for one per-level audit
///     verdict. `details_json` carries `tier_node_key: String`,
///     `score: f32`, `verdict_modifier_label: String`,
///     `findings_count: u32`, `max_expected_findings: f32`.
///   * `skill_prior_retrieved` -- one Voyager-style skill prior
///     surfaced to the scope-planner decomposer. `details_json` carries
///     `query_node_key: String`, `prior_id: String`,
///     `similarity: f32`, `ssb_evidence_seqs: Vec<i64>`,
///     `accepted_diff_summary_len: u32`.
///
/// Closed enum; adding a 9th kind requires bumping the schema.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PlanningDecompositionEventKind {
    /// One reflection appended to a per-node ReflectionBuffer.
    ReflectionAppended,
    /// Decomposition fork with low entropy ratio (children carry too
    /// little information differentiation vs the parent goal).
    LowEntropyDecomposition,
    /// Phase below jaccard OR cosine drift threshold (carries both
    /// arms; the cascade trigger requires two consecutive
    /// below-threshold Phases — that flag rides in details_json).
    GoalDriftCanaryBelowThreshold,
    /// Bubble-up REPLAN_PHASE — payload carries failed_plan +
    /// error_transcript + original_goal (all 3 non-empty validated).
    BubbleUpReplanPhase,
    /// Bubble-up LEDGER_RESET — Magentic-One blocker count crossed
    /// threshold; re-decomposition triggered.
    BubbleUpLedgerReset,
    /// AGENTLESS shortcut dispatched at Step level.
    AgentlessShortcutDispatched,
    /// LATS-style process-reward scalar emitted for a per-level audit
    /// verdict (in [0.0, 1.0]).
    ProcessRewardScoreEmitted,
    /// Voyager-style skill prior retrieved for the scope-planner
    /// decomposer prompt.
    SkillPriorRetrieved,
}

impl PlanningDecompositionEventKind {
    /// Snake-case wire label.
    #[must_use]
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::ReflectionAppended => "reflection_appended",
            Self::LowEntropyDecomposition => "low_entropy_decomposition",
            Self::GoalDriftCanaryBelowThreshold => "goal_drift_canary_below_threshold",
            Self::BubbleUpReplanPhase => "bubble_up_replan_phase",
            Self::BubbleUpLedgerReset => "bubble_up_ledger_reset",
            Self::AgentlessShortcutDispatched => "agentless_shortcut_dispatched",
            Self::ProcessRewardScoreEmitted => "process_reward_score_emitted",
            Self::SkillPriorRetrieved => "skill_prior_retrieved",
        }
    }
}

/// E-46 (SCHEMA_VERSION 184) -- payload for
/// [`SsbActionType::PlanningDecompositionEvent`].
///
/// Wire-dark default; emission gated by
/// `ELAI_PLANNING_DECOMPOSITION_ENABLE` env var (see
/// [`ELAI_PLANNING_DECOMPOSITION_ENABLE_ENV`]). Carries the 8-arm
/// discriminator + optional canonical session/task ids + free-form JSON
/// details so each kind's specific scalars ride along without per-kind
/// payload structs (Pattern-2 single-variant scope-lean).
///
/// Does NOT derive `Eq` -- `details_json` may carry NaN-not-equal `f32`
/// / `f64` values (entropy ratio, jaccard, cosine, process-reward score)
/// per the Pattern-2 28th / 31st / 34th precedents.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PlanningDecompositionEventPayload {
    /// Fire-point discriminator. See [`PlanningDecompositionEventKind`].
    pub kind: PlanningDecompositionEventKind,
    /// Optional canonical session id.
    pub session_id: Option<String>,
    /// Optional canonical task / node id (finer-grained than session for
    /// per-node fire-points like `reflection_appended` /
    /// `process_reward_score_emitted`).
    pub task_id: Option<String>,
    /// Free-form per-kind context. Serialized as JSON so future kinds can
    /// extend without a schema bump. Conventional fields per kind: see
    /// [`PlanningDecompositionEventKind`] docstrings.
    pub details_json: serde_json::Value,
    /// Fire-point wall-clock (millis since UNIX_EPOCH).
    pub ts_ms: i64,
}

/// E-46 -- env var name. Wire-dark default: emission silently skipped
/// until the operator sets this to a truthy value (`"1"` / `"true"` /
/// `"yes"` / `"on"`). Mirrors the precedent set by
/// [`ELAI_TREE_SITTER_GRAMMARS_ENABLE_ENV`] /
/// [`ELAI_LOCAL_CLOUD_ROUTING_ENABLE_ENV`].
///
/// CONTRACT: The literal string MUST match
/// [`crate::planning_decomposition::PLANNING_DECOMPOSITION_ENABLE_ENV`]
/// verbatim. Drift would silently disable the producers without any SSB
/// row to point at.
pub const ELAI_PLANNING_DECOMPOSITION_ENABLE_ENV: &str = "ELAI_PLANNING_DECOMPOSITION_ENABLE";

// ---------------------------------------------------------------------------
// E-48 (SCHEMA_VERSION 185) -- web-research orchestrator hardening substrate.
// ---------------------------------------------------------------------------

/// E-48 -- 10-arm fire-point discriminator carried by
/// [`WebResearchEventPayload`]. Pattern-2 (37th application).
///
/// The plan named 17 distinct fire-points in its WIRING_CONTRACT; this
/// enum collapses them into 10 Kind arms by folding outcome discriminators
/// (e.g. citation verification stage labels, tiered-search dispatch status)
/// into the payload's `details_json`. Precedent: E-50 11-arm + E-4 13-arm.
///
/// Wire labels are snake_case via `#[serde(rename_all)]`; `as_str()` is the
/// canonical helper for log-line and SSB dashboard labels.
///
/// Closed enum; adding an 11th kind requires a SCHEMA_VERSION bump.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum WebResearchEventKind {
    /// Editable plan stage produced a `WebResearchPlan` artifact BEFORE
    /// any SEARCH call fired. `details_json` carries
    /// `sub_question_count: u32`, `planned_search_budget: u32`,
    /// `interactive: bool`, `approved_by_user: bool`.
    ResearchPlanEmitted,
    /// One tiered-search dispatch returned. `details_json` carries
    /// `query_hash_prefix: String` (blake3[..8]), `status: String`
    /// (`"ok"` / `"degraded_to_t2_only"` /
    /// `"tier_disabled_quota_exceeded"`),
    /// `tier_t1_results: u32`, `tier_t2_results: u32`,
    /// `tier_t3_results: u32`, `dedup_dropped: u32`.
    TieredSearchDispatched,
    /// Both crawl4ai primary and Jina fallback failed for one URL.
    /// `details_json` carries `url_hash_prefix: String`,
    /// `primary_error: String`, `fallback_error: String`,
    /// `primary_timeout_ms: u32`, `fallback_timeout_ms: u32`.
    ExtractionFailedAllExtractors,
    /// Cross-encoder rerank finished. `details_json` carries
    /// `n_input: u32`, `n_output: u32` (capped at 5),
    /// `reranker_backend: String` (`"local_sidecar"` / `"cohere"` /
    /// `"none"`), `rank_shift_histogram: [u32; 5]` (max-shift buckets).
    CrossEncoderRerankComplete,
    /// FlashResearch speculative branch pruned. `details_json` carries
    /// `branch_id: u64`, `evidence_delta: f32`, `low_gain_streak: u32`,
    /// `threshold: f32`.
    SpeculativeBranchPruned,
    /// Conformal triple-threshold lower bound fired (preemptive-unsolvable).
    /// `details_json` carries `final_confidence: f32`,
    /// `rounds: u32`, `low_delta_streak: u32`,
    /// `threshold_uncalibrated: bool`.
    ResearchPreemptiveStop,
    /// ContradictionNode materialized in the research tree.
    /// `details_json` carries `source_a_url_hash: String`,
    /// `source_b_url_hash: String`, `classification_reason: String`
    /// (truncated to 200 chars), `prune_protected: bool` (always true).
    ContradictionNodeMaterialized,
    /// One citation passed through the 4-stage verifier (URL-liveness,
    /// arXiv-ID, CrossRef-DOI, SemanticCite faithfulness).
    /// `details_json` carries `citation_url_hash: String`,
    /// `outcome: String` (`"pass"` / `"dead_url"` /
    /// `"invalid_arxiv"` / `"invalid_doi"` / `"unfaithful"`),
    /// `url_live: bool`, `arxiv_valid: Option<bool>`,
    /// `doi_valid: Option<bool>`, `faithful: Option<String>`.
    CitationVerificationOutcome,
    /// Lead / orchestrator attempted to cite outside the SEARCH-stage
    /// attribution set; refusal recorded per the Anthropic
    /// lead-never-cites lock. `details_json` carries
    /// `attempted_url_hash: String`, `producer_set_size: u32`.
    CitationAttributionViolationBlocked,
    /// Temporal-marker regex hit on the question; SourceQualityTable
    /// score multiplied by `exp(-ln(2) * age_days / half_life_days)`.
    /// `details_json` carries `half_life_days: u32`,
    /// `n_sources_decayed: u32`, `mean_age_days: f32`,
    /// `temporal_marker: String` (the matched token).
    RecencyDecayApplied,
}

impl WebResearchEventKind {
    /// Snake-case wire label.
    #[must_use]
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::ResearchPlanEmitted => "research_plan_emitted",
            Self::TieredSearchDispatched => "tiered_search_dispatched",
            Self::ExtractionFailedAllExtractors => "extraction_failed_all_extractors",
            Self::CrossEncoderRerankComplete => "cross_encoder_rerank_complete",
            Self::SpeculativeBranchPruned => "speculative_branch_pruned",
            Self::ResearchPreemptiveStop => "research_preemptive_stop",
            Self::ContradictionNodeMaterialized => "contradiction_node_materialized",
            Self::CitationVerificationOutcome => "citation_verification_outcome",
            Self::CitationAttributionViolationBlocked => "citation_attribution_violation_blocked",
            Self::RecencyDecayApplied => "recency_decay_applied",
        }
    }
}

/// E-48 (SCHEMA_VERSION 185) -- payload for
/// [`SsbActionType::WebResearchEvent`].
///
/// Wire-dark default; emission gated by
/// `ELAI_WEB_RESEARCH_ORCHESTRATOR_ENABLE` env var (see
/// [`ELAI_WEB_RESEARCH_ORCHESTRATOR_ENABLE_ENV`]).
///
/// Carries the 10-arm Kind discriminator + optional canonical
/// session/node ids + free-form JSON details so each kind's specific
/// scalars ride along without per-kind payload structs (Pattern-2
/// single-variant scope-lean — same shape as
/// [`PlanningDecompositionEventPayload`]).
///
/// Does NOT derive `Eq` -- `details_json` may carry NaN-not-equal `f32`
/// values (evidence_delta, recency mean_age_days, citation faithfulness
/// scores) per the Pattern-2 28th / 31st / 34th / 36th precedents.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct WebResearchEventPayload {
    /// Fire-point discriminator. See [`WebResearchEventKind`].
    pub kind: WebResearchEventKind,
    /// Optional canonical session id.
    pub session_id: Option<String>,
    /// Optional canonical node id (ResearchTree node ordinal, base-10).
    pub node_id: Option<String>,
    /// Free-form per-kind context. Serialized as JSON so future kinds
    /// can extend without a schema bump. Conventional fields per kind:
    /// see [`WebResearchEventKind`] docstrings.
    pub details_json: serde_json::Value,
    /// Fire-point wall-clock (millis since UNIX_EPOCH).
    pub ts_ms: i64,
}

/// E-48 -- env var name. Wire-dark default: emission silently skipped
/// until the operator sets this to a truthy value (`"1"` / `"true"` /
/// `"yes"` / `"on"`). Mirrors the precedent set by
/// [`ELAI_PLANNING_DECOMPOSITION_ENABLE_ENV`] /
/// [`ELAI_TREE_SITTER_GRAMMARS_ENABLE_ENV`].
///
/// CONTRACT: The literal string MUST match
/// `crate::research::web_orchestrator::WEB_RESEARCH_ORCHESTRATOR_ENABLE_ENV`
/// verbatim. Drift would silently disable the producers without any SSB
/// row to point at.
pub const ELAI_WEB_RESEARCH_ORCHESTRATOR_ENABLE_ENV: &str =
    "ELAI_WEB_RESEARCH_ORCHESTRATOR_ENABLE";

// ---------------------------------------------------------------------------
// E-44 (SCHEMA_VERSION 186) -- durability observability substrate.
// ---------------------------------------------------------------------------

/// E-44 -- 10-arm fire-point discriminator carried by
/// [`DurabilityEventPayload`]. Pattern-2 (38th application).
///
/// Each arm maps 1:1 to one of the plan's promised 10 sibling
/// SsbActionType variants (TASK-E44-013 closure). 5 of 10 Kinds cover
/// task primaries whose producer modules are NOT-SHIPPED at the time of
/// SCHEMA_VERSION 186; their wire tags are reserved here so the
/// follow-on producers (SAGA log, reliability metrics rollup, tool-error
/// taxonomy) ship without a further SCHEMA bump.
///
/// Wire labels are snake_case via `#[serde(rename_all)]`; `as_str()` is
/// the canonical helper for log-line and SSB dashboard labels.
///
/// Closed enum; adding an 11th kind requires a SCHEMA_VERSION bump.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DurabilityEventKind {
    /// IdempotentDispatcher served a cached result for a deterministic
    /// call-shape replay. `details_json` carries
    /// `call_shape_hash: u64` (hex), `tool_id: String`, `cache_age_ms: u64`.
    /// Producer: `crates/orchestrator/src/durability/idempotent_dispatcher.rs`.
    ToolIdempotencyHit,
    /// A cached side-effect-bearing call was elided to preserve at-most-once
    /// semantics. `details_json` carries `call_shape_hash: u64` (hex),
    /// `tool_id: String`, `effect_class: String` (`"file_write"` /
    /// `"network"` / `"db_write"`). Producer: idempotent_dispatcher.rs.
    ToolIdempotencyReplayedIrreversible,
    /// A SAGA compensation action was appended to the compensation_actions
    /// log. `details_json` carries `saga_id: String`, `action_id: String`,
    /// `step_index: u32`, `compensate_tool: String`. Producer: PENDING
    /// (TASK-E44-002 saga.rs not yet shipped); wire tag reserved.
    CompensationLogged,
    /// A recovered session re-executed a logged compensation.
    /// `details_json` carries `saga_id: String`, `action_id: String`,
    /// `outcome: String` (`"ok"` / `"err"`), `duration_ms: u64`. Producer:
    /// PENDING (TASK-E44-002); wire tag reserved.
    CompensationReplayed,
    /// A compensation action errored. `details_json` carries
    /// `saga_id: String`, `action_id: String`, `error_class: String`,
    /// `error_message: String` (truncated to 200 chars). Producer: PENDING
    /// (TASK-E44-002); wire tag reserved.
    CompensationFailed,
    /// The per-session ceiling (event count OR phase count) was hit and
    /// `continue_as_new_if_needed` rolled the session over to a new root.
    /// `details_json` carries `old_session_id: String`,
    /// `new_session_id: String`, `trigger: String` (`"event_count"` /
    /// `"phase_count"`), `event_count: u64`, `phase_count: u32`,
    /// `ceiling: u64`. Producer:
    /// `crates/orchestrator/src/durability/continue_as_new.rs`.
    SessionDagContinueAsNew,
    /// The read-side migrator chain transformed a checkpoint from
    /// `from_version` to CURRENT. `details_json` carries
    /// `from_version: u32`, `to_version: u32`, `transform_count: u32`,
    /// `bytes_before: u64`, `bytes_after: u64`. Producer:
    /// `crates/orchestrator/src/durability/checkpoint_migrator.rs`.
    CheckpointLazyMigrated,
    /// The PreCompact hook and 85% compact path raced; the second
    /// acquirer waited or aborted. `details_json` carries
    /// `lock_path: String`, `holder_pid: u32`, `wait_ms: u64`,
    /// `outcome: String` (`"acquired_after_wait"` / `"aborted"`).
    /// Producer: `crates/orchestrator/src/durability/compact_lock.rs`.
    CompactLockContended,
    /// RDC (Reliability Decay Curve) / VAF / GDS / MOP rollup emission.
    /// `details_json` carries `agent_id: String`, `phase_id: String`,
    /// `rdc: f32`, `vaf: f32`, `gds: f32`, `mop: f32`,
    /// `sample_window: u32`. Producer: PENDING (TASK-E44-008
    /// reliability.rs not yet shipped); wire tag reserved.
    AgentReliabilityMetrics,
    /// A tool error was bucketed into the 3-bucket recovery taxonomy.
    /// `details_json` carries `tool_id: String`, `error_class: String`
    /// (`"transient"` / `"contract_violation"` / `"unrecoverable"`),
    /// `error_message: String` (truncated to 200 chars),
    /// `recovery_action: String` (`"retry"` / `"compensate"` / `"abort"`).
    /// Producer: PENDING (TASK-E44-009 error_taxonomy.rs not yet
    /// shipped); wire tag reserved.
    ToolErrorClassified,
}

impl DurabilityEventKind {
    /// Snake-case wire label.
    #[must_use]
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::ToolIdempotencyHit => "tool_idempotency_hit",
            Self::ToolIdempotencyReplayedIrreversible => "tool_idempotency_replayed_irreversible",
            Self::CompensationLogged => "compensation_logged",
            Self::CompensationReplayed => "compensation_replayed",
            Self::CompensationFailed => "compensation_failed",
            Self::SessionDagContinueAsNew => "session_dag_continue_as_new",
            Self::CheckpointLazyMigrated => "checkpoint_lazy_migrated",
            Self::CompactLockContended => "compact_lock_contended",
            Self::AgentReliabilityMetrics => "agent_reliability_metrics",
            Self::ToolErrorClassified => "tool_error_classified",
        }
    }
}

/// E-44 (SCHEMA_VERSION 186) -- payload for
/// [`SsbActionType::DurabilityEvent`].
///
/// Wire-dark default; emission gated by
/// `ELAI_DURABILITY_OBSERVABILITY_ENABLE` env var (see
/// [`ELAI_DURABILITY_OBSERVABILITY_ENABLE_ENV`]).
///
/// Carries the 10-arm Kind discriminator + optional canonical
/// session/agent ids + free-form JSON details so each kind's specific
/// scalars ride along without per-kind payload structs (Pattern-2
/// single-variant scope-lean -- same shape as
/// [`WebResearchEventPayload`] / [`PlanningDecompositionEventPayload`]).
///
/// Does NOT derive `Eq` -- `details_json` may carry NaN-not-equal `f32`
/// values (reliability metrics rdc / vaf / gds / mop) per the Pattern-2
/// 28th / 31st / 34th / 36th / 37th precedents.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DurabilityEventPayload {
    /// Fire-point discriminator. See [`DurabilityEventKind`].
    pub kind: DurabilityEventKind,
    /// Optional canonical session id.
    pub session_id: Option<String>,
    /// Optional canonical agent id (for arms where it applies).
    pub agent_id: Option<String>,
    /// Free-form per-kind context. Serialized as JSON so future kinds
    /// can extend without a schema bump. Conventional fields per kind:
    /// see [`DurabilityEventKind`] docstrings.
    pub details_json: serde_json::Value,
    /// Fire-point wall-clock (millis since UNIX_EPOCH).
    pub ts_ms: i64,
}

/// E-44 -- env var name. Wire-dark default: emission silently skipped
/// until the operator sets this to a truthy value (`"1"` / `"true"` /
/// `"yes"` / `"on"`). Mirrors the precedent set by
/// [`ELAI_WEB_RESEARCH_ORCHESTRATOR_ENABLE_ENV`] /
/// [`ELAI_PLANNING_DECOMPOSITION_ENABLE_ENV`] /
/// [`ELAI_TREE_SITTER_GRAMMARS_ENABLE_ENV`].
///
/// Sites that consult this gate live in
/// `crates/orchestrator/src/durability/*.rs` producer hooks (helpers
/// like `notify_idempotency_hit()` introduced alongside this
/// SCHEMA_VERSION bump as wire-dark scaffolding).
pub const ELAI_DURABILITY_OBSERVABILITY_ENABLE_ENV: &str =
    "ELAI_DURABILITY_OBSERVABILITY_ENABLE";

// ---------------------------------------------------------------------------
// E-31 (SCHEMA_VERSION 187) -- head_dim invariant verification substrate.
// ---------------------------------------------------------------------------

/// E-31 -- 2-arm fire-point discriminator carried by
/// [`HeadDimInvariantVerifiedPayload`]. Pattern-2 (39th application).
///
/// The boot-time refuse path raises `SidecarBootError` BEFORE any SSB
/// writer is reachable, so the `Refused` arm is reserved for in-band
/// re-check from a hot config swap (producer pending; wire tag reserved
/// here so a follow-on producer ships without a further SCHEMA bump).
///
/// Wire labels are snake_case via `#[serde(rename_all)]`; `as_str()` is
/// the canonical helper for log-line and SSB dashboard labels.
///
/// Closed enum; adding a 3rd kind requires a SCHEMA_VERSION bump.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum HeadDimInvariantVerifiedKind {
    /// `assert_head_dim_invariant` succeeded at sidecar boot: observed
    /// head_dim matched expected (128 for the Qwen2.5-Coder family).
    /// `details_json` carries `{model_id, head_dim_observed,
    /// head_dim_expected, ok: true}`. Producer:
    /// `python/elai_inference/server.py::assert_head_dim_invariant`.
    Verified,
    /// Post-boot health-check re-evaluation detected a mismatch.
    /// `details_json` carries `{model_id, head_dim_observed,
    /// head_dim_expected, ok: false}`. Producer: PENDING (hot-swap
    /// health-check path not yet shipped); wire tag reserved.
    Refused,
}

impl HeadDimInvariantVerifiedKind {
    /// Snake-case wire label.
    #[must_use]
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Verified => "verified",
            Self::Refused => "refused",
        }
    }
}

/// E-31 (SCHEMA_VERSION 187) -- payload for
/// [`SsbActionType::HeadDimInvariantVerified`].
///
/// Wire-dark default; emission gated by
/// `ELAI_HEAD_DIM_INVARIANT_SSB_ENABLE` env var (see
/// [`ELAI_HEAD_DIM_INVARIANT_SSB_ENABLE_ENV`]).
///
/// Mirrors the head_dim_invariant contract test at the runtime layer:
/// the contract test asserts head_dim == 128 from config.json on disk;
/// the sidecar `assert_head_dim_invariant` asserts the same on the
/// loaded model object. Both must agree for the Qwen2.5-Coder family
/// to attain stable SDPA / FA2 / xformers attention performance.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct HeadDimInvariantVerifiedPayload {
    /// Fire-point discriminator. See [`HeadDimInvariantVerifiedKind`].
    pub kind: HeadDimInvariantVerifiedKind,
    /// Model id / path that was checked (e.g.
    /// `"huihui-ai/Qwen2.5-Coder-3B-Instruct-abliterated"`).
    pub model_id: String,
    /// Head dimension observed on the loaded model
    /// (hidden_size / num_attention_heads).
    pub head_dim_observed: u32,
    /// Head dimension expected by the sidecar attention paths.
    /// Default 128 for Qwen2.5-Coder family.
    pub head_dim_expected: u32,
    /// True when observed == expected. False reserved for `Refused` arm.
    pub ok: bool,
    /// Free-form per-fire-point context (extra config diagnostics).
    pub details_json: serde_json::Value,
    /// Fire-point wall-clock (millis since UNIX_EPOCH).
    pub ts_ms: i64,
}

/// E-31 -- env var name. Wire-dark default: emission silently skipped
/// until the operator sets this to a truthy value (`"1"` / `"true"` /
/// `"yes"` / `"on"`). Mirrors the precedent set by
/// [`ELAI_DURABILITY_OBSERVABILITY_ENABLE_ENV`] /
/// [`ELAI_WEB_RESEARCH_ORCHESTRATOR_ENABLE_ENV`].
///
/// The sidecar producer
/// (`python/elai_inference/server.py::assert_head_dim_invariant`)
/// reads the equivalent OS env var directly; the const here is the
/// Rust-side declaration consumed by any future Rust SSB writer
/// (e.g. a dashboard-side hot-swap health check).
pub const ELAI_HEAD_DIM_INVARIANT_SSB_ENABLE_ENV: &str =
    "ELAI_HEAD_DIM_INVARIANT_SSB_ENABLE";

// ---------------------------------------------------------------------------
// E-3 (SCHEMA_VERSION 188) -- role-shown / model-hidden default telemetry.
// ---------------------------------------------------------------------------

/// E-3 (SCHEMA_VERSION 188) -- payload for
/// [`SsbActionType::RoleShownModelHidden`].
///
/// Scope-lean MVP slice of TASK-E03-001: tracks the SettingsWindow
/// "show model details" toggle so the audit pass can verify the default
/// (MODEL hidden, ROLE visible) is reached. No Kind discriminator -- this
/// is a single user-facing toggle, not a multi-fire-point envelope.
///
/// Wire-dark default; emission gated by
/// `ELAI_ROLE_VISIBILITY_TELEMETRY_ENABLE` env var (see
/// [`ELAI_ROLE_VISIBILITY_TELEMETRY_ENABLE_ENV`]).
///
/// Producer: `crates/dashboard-v2/src-tauri/src/commands.rs::record_model_visibility`
/// (Tauri command emitted by `SettingsWindow.svelte` on toggle change).
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct RoleShownModelHiddenPayload {
    /// Agent role string (e.g. `"executor"`, `"auditor"`,
    /// `"prompt-engineer"`). Free-form so the payload survives future
    /// role taxonomy churn without a SCHEMA bump.
    pub role: String,
    /// True when the MODEL name is hidden in the UI for this role
    /// (the E-3 default). False when the user toggled "show model details"
    /// ON and the MODEL string is rendered.
    pub model_hidden: bool,
    /// Optional canonical session id. None when fired before a session
    /// is bound (cold-boot UI render).
    pub session_id: Option<String>,
    /// Fire-point wall-clock (millis since UNIX_EPOCH).
    pub ts_ms: i64,
}

/// E-3 -- env var name. Wire-dark default: emission silently skipped
/// until the operator sets this to a truthy value (`"1"` / `"true"` /
/// `"yes"` / `"on"`). Mirrors the precedent set by
/// [`ELAI_HEAD_DIM_INVARIANT_SSB_ENABLE_ENV`] /
/// [`ELAI_DURABILITY_OBSERVABILITY_ENABLE_ENV`].
///
/// Consulted by `record_model_visibility` in dashboard-v2 commands.rs
/// before write_payload; the UI toggle itself is always reactive (the
/// telemetry is opt-in observation, not a feature flag for the toggle).
pub const ELAI_ROLE_VISIBILITY_TELEMETRY_ENABLE_ENV: &str =
    "ELAI_ROLE_VISIBILITY_TELEMETRY_ENABLE";

// ---------------------------------------------------------------------------
// E-49 (SCHEMA_VERSION 189) -- coding-agent metrics named-token substrate.
// ---------------------------------------------------------------------------

/// E-49 -- 3-arm fire-point discriminator carried by
/// [`CodingMetricNamedTokenPayload`]. Pattern-2 (41st application).
///
/// Covers the 3 spec-named metric tokens the original E-49 ship did not
/// land under their canonical strings (per cross-check
/// E-16_e47_e48_e49.md: zero hits for `stable_pass_at_1` /
/// `harness_attribution_tuple` / `stop_verified` across `.elai_cc/`).
///
/// Wire labels are snake_case via `#[serde(rename_all)]`; `as_str()` is
/// the canonical helper for log-line and SSB dashboard labels.
///
/// Closed enum; adding a 4th kind requires a SCHEMA_VERSION bump.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CodingMetricNamedTokenKind {
    /// `stable_pass_at_1` -- n=5 seed stability metric. The runner
    /// executes the same test 5 times with different deterministic seeds
    /// and asserts identical output across runs. `details_json` carries
    /// `{seed_count, identical_outputs, std_dev, mean_score,
    /// test_id, model_id}`. Replaces single-seed Pass@1 reporting per
    /// the E-49 plan's "credible stack" addition.
    StablePassAt1,
    /// `harness_attribution_tuple` -- per-session record of
    /// `(model_id, harness_version, harness_commit_sha)` so the B-37
    /// harness-vs-model ablation lever is legible at audit time.
    /// `details_json` carries `{model_id, harness_version,
    /// harness_commit_sha, harness_repo, session_id}`. Producer reads
    /// the git commit sha at runtime via `std::process::Command` from
    /// the harness checkout root.
    HarnessAttributionTuple,
    /// `stop_verified` -- result of the CI lint that refuses legacy
    /// HumanEval / MBPP / SWE-bench Verified mention without an
    /// accompanying `STOP-Verified:` guard comment. `details_json`
    /// carries `{files_scanned, violations_count, violations: [{path,
    /// line, token}], guard_present}`. Producer:
    /// `scripts/check_stop_verified.sh` (CI lint). Prevents accidental
    /// re-introduction of contaminated benches; coordinates with C-54
    /// SWE-Bench Verified RUNPOD work.
    StopVerified,
}

impl CodingMetricNamedTokenKind {
    /// Snake-case wire label.
    #[must_use]
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::StablePassAt1 => "stable_pass_at_1",
            Self::HarnessAttributionTuple => "harness_attribution_tuple",
            Self::StopVerified => "stop_verified",
        }
    }
}

/// E-49 (SCHEMA_VERSION 189) -- payload for
/// [`SsbActionType::CodingMetricNamedToken`].
///
/// Wire-dark default; emission gated by
/// `ELAI_CODING_METRIC_NAMED_TOKEN_ENABLE` env var (see
/// [`ELAI_CODING_METRIC_NAMED_TOKEN_ENABLE_ENV`]).
///
/// Closes the 3 SPEC-DRIFT TASK rows (TASK-E49-005 / 006 / 007) by
/// surfacing the named tokens the original E-49 ship omitted under the
/// canonical strings.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CodingMetricNamedTokenPayload {
    /// Fire-point discriminator. See [`CodingMetricNamedTokenKind`].
    pub kind: CodingMetricNamedTokenKind,
    /// Canonical session id this metric row belongs to.
    pub session_id: String,
    /// Optional model id the metric was measured against (None for the
    /// `stop_verified` CI-lint arm which is model-agnostic).
    pub model_id: Option<String>,
    /// Free-form per-fire-point context. Shape depends on `kind`:
    ///   * StablePassAt1 -- `{seed_count, identical_outputs, std_dev,
    ///     mean_score, test_id}`.
    ///   * HarnessAttributionTuple -- `{harness_version,
    ///     harness_commit_sha, harness_repo}`.
    ///   * StopVerified -- `{files_scanned, violations_count,
    ///     violations: [...], guard_present}`.
    pub details_json: serde_json::Value,
    /// Fire-point wall-clock (millis since UNIX_EPOCH).
    pub ts_ms: i64,
}

/// E-49 -- env var name. Wire-dark default: emission silently skipped
/// until the operator sets this to a truthy value (`"1"` / `"true"` /
/// `"yes"` / `"on"`). Mirrors the precedent set by
/// [`ELAI_HEAD_DIM_INVARIANT_SSB_ENABLE_ENV`] /
/// [`ELAI_DURABILITY_OBSERVABILITY_ENABLE_ENV`] /
/// [`ELAI_ROLE_VISIBILITY_TELEMETRY_ENABLE_ENV`].
pub const ELAI_CODING_METRIC_NAMED_TOKEN_ENABLE_ENV: &str =
    "ELAI_CODING_METRIC_NAMED_TOKEN_ENABLE";

// ---------------------------------------------------------------------------
// E-3 sub-phase 4A (SCHEMA_VERSION 190) -- runtime handoff event substrate.
// ---------------------------------------------------------------------------

/// E-3 sub-phase 4A -- 6-arm fire-point discriminator carried by
/// [`RuntimeHandoffEventPayload`]. Pattern-2 (42nd application).
///
/// Collapses the 6 sibling variants the original E-3 plan called out into a
/// single envelope (per cross-check + exploration recommendation in
/// `workflow/sessions/s-21636/exploration/tranche4-E3-dashboard.md`),
/// matching the documented Pattern-2 precedent set by
/// [`CodingMetricNamedTokenKind`] (3-arm) and
/// [`HeadDimInvariantVerifiedKind`] (2-arm).
///
/// Wire labels are snake_case via `#[serde(rename_all)]`; `as_str()` is the
/// canonical helper for log-line and SSB dashboard labels.
///
/// Closed enum; adding a 7th kind requires a `SCHEMA_VERSION` bump.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RuntimeHandoffEventKind {
    /// `unified_preamble_applied` -- emitted by
    /// `RuntimeOrchestrator::try_complete` when `build_role_prompt` wraps
    /// the system message with a [`crate::role_prompt::VoiceAnchor`]
    /// preamble. `details_json` carries
    /// `{base_prompt_len, anchor_blake3, preamble_len}`.
    UnifiedPreambleApplied,
    /// `runtime_prewarm_started` -- emitted by `predict_next_role` before
    /// kicking off a 1-token CUDA-graph warmup on the predicted next
    /// backend. `details_json` carries
    /// `{predicted_role, backend, vram_tier}`. (sub-phase 4B)
    RuntimePrewarmStarted,
    /// `runtime_prewarm_complete` -- emitted after the warmup returns.
    /// `details_json` carries
    /// `{backend, duration_ms, ok, cuda_graph_captured}`. (sub-phase 4B)
    RuntimePrewarmComplete,
    /// `handoff_payload_compressed` -- emitted by
    /// `compress_handoff_payload`. `details_json` carries
    /// `{original_tokens, compressed_tokens, ratio}`. (sub-phase 4B)
    HandoffPayloadCompressed,
    /// `persona_vector_applied` -- emitted by the sidecar persona-vector
    /// hook. `details_json` carries
    /// `{model_id, vector_blake3, strength}`. (sub-phase 4B)
    PersonaVectorApplied,
    /// `style_normalizer_applied` -- emitted by the live_cli style
    /// normalizer post-pass. `details_json` carries
    /// `{source_role, input_blake3, output_blake3, tells_stripped}`.
    /// (sub-phase 4B)
    StyleNormalizerApplied,
}

impl RuntimeHandoffEventKind {
    /// Snake-case wire label.
    #[must_use]
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::UnifiedPreambleApplied => "unified_preamble_applied",
            Self::RuntimePrewarmStarted => "runtime_prewarm_started",
            Self::RuntimePrewarmComplete => "runtime_prewarm_complete",
            Self::HandoffPayloadCompressed => "handoff_payload_compressed",
            Self::PersonaVectorApplied => "persona_vector_applied",
            Self::StyleNormalizerApplied => "style_normalizer_applied",
        }
    }
}

/// E-3 sub-phase 4A (SCHEMA_VERSION 190) -- payload for
/// [`SsbActionType::E3RuntimeHandoffEvent`].
///
/// Wire-dark default; emission gated by
/// `ELAI_E3_RUNTIME_HANDOFF_ENABLE` env var (see
/// [`ELAI_E3_RUNTIME_HANDOFF_ENABLE_ENV`]).
///
/// Covers all 6 fire points of the model-handoff UX plan (E-3). Sub-phase
/// 4A wires only the `UnifiedPreambleApplied` arm via the role-prompt wrap
/// in `runtime_orchestrator.rs`; the other 5 arms are reserved (variant
/// + envelope shipped now so dashboard + CLI consumers can begin building
/// against a stable wire surface in 4B).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RuntimeHandoffEventPayload {
    /// Fire-point discriminator. See [`RuntimeHandoffEventKind`].
    pub kind: RuntimeHandoffEventKind,
    /// Agent role string the handoff applies to (e.g. `"executor"`,
    /// `"auditor"`). Optional because some fire points (style-normalizer
    /// post-pass) are role-agnostic.
    pub role: Option<String>,
    /// Runtime backend identifier (e.g. `"sidecar"`, `"anthropic_api"`).
    /// Optional because the preamble-applied arm fires before backend
    /// selection commits.
    pub backend: Option<String>,
    /// Canonical session id this handoff belongs to. None when fired
    /// before a session is bound (cold-boot dispatch).
    pub session_id: Option<String>,
    /// Free-form per-fire-point context. Shape depends on `kind`:
    ///   * UnifiedPreambleApplied -- `{base_prompt_len, anchor_blake3,
    ///     preamble_len}`.
    ///   * RuntimePrewarmStarted -- `{predicted_role, backend, vram_tier}`.
    ///   * RuntimePrewarmComplete -- `{backend, duration_ms, ok,
    ///     cuda_graph_captured}`.
    ///   * HandoffPayloadCompressed -- `{original_tokens,
    ///     compressed_tokens, ratio}`.
    ///   * PersonaVectorApplied -- `{model_id, vector_blake3, strength}`.
    ///   * StyleNormalizerApplied -- `{source_role, input_blake3,
    ///     output_blake3, tells_stripped}`.
    pub details_json: serde_json::Value,
    /// Fire-point wall-clock (millis since UNIX_EPOCH).
    pub ts_ms: i64,
}

/// E-3 sub-phase 4A -- env var name. Wire-dark default: emission silently
/// skipped until the operator sets this to a truthy value (`"1"` /
/// `"true"` / `"yes"` / `"on"`). Mirrors the precedent set by
/// [`ELAI_HEAD_DIM_INVARIANT_SSB_ENABLE_ENV`] /
/// [`ELAI_DURABILITY_OBSERVABILITY_ENABLE_ENV`] /
/// [`ELAI_ROLE_VISIBILITY_TELEMETRY_ENABLE_ENV`] /
/// [`ELAI_CODING_METRIC_NAMED_TOKEN_ENABLE_ENV`].
///
/// Consulted by `runtime_orchestrator.rs::try_complete` (sub-phase 4A
/// wires the UnifiedPreambleApplied arm via tracing log + a TODO marker
/// for the actual Ssb::write_payload injection in sub-phase 4B).
pub const ELAI_E3_RUNTIME_HANDOFF_ENABLE_ENV: &str =
    "ELAI_E3_RUNTIME_HANDOFF_ENABLE";

// ---------------------------------------------------------------------------
// E-31 sub-phase 6A (SCHEMA_VERSION 191) -- cloud-route decision event substrate.
// ---------------------------------------------------------------------------

/// E-31 sub-phase 6A -- 3-arm fire-point discriminator carried by
/// [`RouteDecisionEventPayload`]. Pattern-2 (43rd application).
///
/// Collapses the E-31 plan's two deferred SSB envelopes
/// (CloudRouteDecision + TierModelSwap) plus a third "kept local" arm
/// into a single Pattern-2 variant. Matches the documented
/// [`CodingMetricNamedTokenKind`] (3-arm) precedent and avoids 2 SCHEMA
/// slot bumps + 2 ALL_VARIANTS rows. The `LocalRouteKept` arm is the
/// sanity-check telemetry surface for the local-first path (operator
/// expects most steps to NOT escalate).
///
/// Wire labels are snake_case via `#[serde(rename_all)]`; `as_str()` is
/// the canonical helper for log-line and SSB dashboard labels.
///
/// Closed enum; adding a 4th kind requires a `SCHEMA_VERSION` bump.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RouteDecisionEventKind {
    /// The runtime routing policy escalated to a cloud arm. The
    /// `route` field of the payload carries the cloud arm label
    /// (`cloud_sonnet` / `cloud_opus` / `cloud_deepseek_flash`).
    /// Producer: `cloud_mode.rs::CloudModeRouter::route`.
    CloudRouteChosen,
    /// The routing policy forced a tier-model swap (e.g. VRAM
    /// headroom too low for the per-role configured model;
    /// swap from 7B local -> 3B local or local -> cloud cheap arm).
    /// Producer: PENDING (E-31 sub-phase 6B; wire tag reserved here).
    TierModelSwapped,
    /// The routing policy evaluated the per-step context and kept
    /// the local sidecar route. Useful for telemetry on the
    /// local-first path. Producer:
    /// `cloud_mode.rs::CloudModeRouter::route`.
    LocalRouteKept,
}

impl RouteDecisionEventKind {
    /// Snake-case wire label.
    #[must_use]
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::CloudRouteChosen => "cloud_route_chosen",
            Self::TierModelSwapped => "tier_model_swapped",
            Self::LocalRouteKept => "local_route_kept",
        }
    }
}

/// E-31 sub-phase 6A (SCHEMA_VERSION 191) -- payload for
/// [`SsbActionType::RouteDecisionEvent`].
///
/// Wire-dark default; emission gated by
/// `ELAI_ROUTE_DECISION_SSB_ENABLE` env var (see
/// [`ELAI_ROUTE_DECISION_SSB_ENABLE_ENV`]).
///
/// Mirrors the
/// `providers::runtime::escalation_policy::RoutingDecision` shape at the
/// SSB row layer; the policy module owns the decision-table while the
/// SSB row owns the audit-trail. The two enums (`RoutingChoice`,
/// `RoutingReason`) are mirrored as snake-case strings here rather than
/// as cross-crate typed enums to keep the orchestrator -> providers
/// dependency edge unidirectional.
///
/// Fields:
///   * `kind` -- 3-arm fire-point discriminator (see
///     [`RouteDecisionEventKind`]).
///   * `route` -- snake-case label of the selected
///     `RoutingChoice` (e.g. `"local_sidecar"`, `"cloud_sonnet"`,
///     `"cloud_opus"`, `"cloud_deepseek_flash"`).
///   * `score` -- difficulty score in [0.0, 1.0] (caller-supplied;
///     mirrors the E-39 `Difficulty` f32). May be 0.0 for fire
///     points that do not carry a difficulty signal
///     (`LocalRouteKept` when the local route was unconditional).
///   * `ms` -- decision latency in millis (caller-measured).
///   * `reason` -- snake-case label of the
///     `RoutingReason` (e.g. `"online_threshold_met"`,
///     `"vram_headroom_low"`).
///   * `session_id` -- canonical session id when known.
///   * `details_json` -- free-form per-fire-point context.
///   * `ts_ms` -- fire-point wall-clock (millis since UNIX_EPOCH).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RouteDecisionEventPayload {
    /// Fire-point discriminator. See [`RouteDecisionEventKind`].
    pub kind: RouteDecisionEventKind,
    /// Snake-case label of the selected `RoutingChoice`.
    pub route: String,
    /// Difficulty score in [0.0, 1.0].
    pub score: f32,
    /// Decision latency in millis.
    pub ms: u64,
    /// Snake-case label of the `RoutingReason`.
    pub reason: String,
    /// Canonical session id this decision belongs to.
    pub session_id: Option<String>,
    /// Free-form per-fire-point context.
    pub details_json: serde_json::Value,
    /// Fire-point wall-clock (millis since UNIX_EPOCH).
    pub ts_ms: i64,
}

/// E-31 sub-phase 6A -- env var name. Wire-dark default: emission
/// silently skipped until the operator sets this to a truthy value
/// (`"1"` / `"true"` / `"yes"` / `"on"`). Mirrors the precedent set by
/// [`ELAI_HEAD_DIM_INVARIANT_SSB_ENABLE_ENV`] /
/// [`ELAI_E3_RUNTIME_HANDOFF_ENABLE_ENV`] /
/// [`ELAI_CODING_METRIC_NAMED_TOKEN_ENABLE_ENV`].
///
/// Consulted by `cloud_mode.rs::CloudModeRouter::route` (sub-phase 6A
/// wires the `CloudRouteChosen` and `LocalRouteKept` arms via tracing
/// log; `TierModelSwapped` producer + `Ssb::write_payload` injection
/// land in sub-phase 6B alongside the dashboard RuntimePanel row).
pub const ELAI_ROUTE_DECISION_SSB_ENABLE_ENV: &str =
    "ELAI_ROUTE_DECISION_SSB_ENABLE";

// ---------------------------------------------------------------------------
// E-47 (SCHEMA_VERSION 192) -- LoRA fine-tuning ship-gate substrate.
// ---------------------------------------------------------------------------

/// E-47 -- 3-arm phase discriminator carried by
/// [`LoraShipGateEventPayload`]. Pattern-2 (44th application).
///
/// Collapses the E-47 plan's nominally three sibling SSB rows
/// (`lora_ship_gate_refused` / `lora_calibration_gate_passed` /
/// `lora_calibration_gate_refused`) into a single Pattern-2 variant
/// per the E-43 [`SkillEventPhase`] precedent. Matches the
/// [`SkillEventPhase`] (3-arm) and [`RouteDecisionEventKind`] (3-arm)
/// shape and avoids 2 SCHEMA slot bumps + 2 ALL_VARIANTS rows.
///
/// Wire labels are snake_case via `#[serde(rename_all)]`; `as_str()`
/// is the canonical helper for log-line and SSB dashboard labels.
///
/// Closed enum; adding a 4th phase requires a `SCHEMA_VERSION` bump.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LoraShipGateEventPhase {
    /// The triple-AND ship-gate refused the adapter load.
    /// `details_json` carries the refusal reason
    /// (`deferral_token_present` / `calibration_regression` / etc.).
    Refused,
    /// The ship-gate is enabled and configured to ship but no
    /// `LoraCalibrationReport` was supplied. Refuse-and-continue
    /// posture (no adapter load).
    CalibrationMissing,
    /// The ship-gate is either wire-dark (`enabled = false`) or the
    /// calibration report passes the no-regression threshold.
    /// Clear-to-proceed posture.
    CalibrationPassed,
}

impl LoraShipGateEventPhase {
    /// Snake-case wire label.
    #[must_use]
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Refused => "refused",
            Self::CalibrationMissing => "calibration_missing",
            Self::CalibrationPassed => "calibration_passed",
        }
    }
}

/// E-47 (SCHEMA_VERSION 192) -- payload for
/// [`SsbActionType::LoraShipGateEvent`].
///
/// Wire-dark default; emission gated by
/// `ELAI_LORA_SHIP_GATE_ENABLE` env var (see
/// [`ELAI_LORA_SHIP_GATE_ENABLE_ENV`]).
///
/// Mirrors the
/// [`crate::lora_ship_gate::LoraShipGateOutcome`] shape at the SSB
/// row layer; the pure-logic module owns the decision function while
/// the SSB row owns the audit trail. `phase` is the canonical
/// outcome tag; `details_json` is a free-form per-fire-point context
/// object (config snapshot, calibration deltas, refusal reason, etc.).
///
/// Fields:
///   * `phase` -- 3-arm outcome discriminator (see
///     [`LoraShipGateEventPhase`]).
///   * `session_id` -- canonical session id when known.
///   * `agent_id` -- emitting agent identifier (CLI or
///     orchestrator subsystem).
///   * `details_json` -- free-form per-fire-point context.
///   * `ts_ms` -- fire-point wall-clock (millis since UNIX_EPOCH).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LoraShipGateEventPayload {
    /// Outcome discriminator. See [`LoraShipGateEventPhase`].
    pub kind: LoraShipGateEventPhase,
    /// Canonical session id this event belongs to.
    pub session_id: Option<String>,
    /// Emitting agent identifier.
    pub agent_id: Option<String>,
    /// Free-form per-fire-point context.
    pub details_json: serde_json::Value,
    /// Fire-point wall-clock (millis since UNIX_EPOCH).
    pub ts_ms: i64,
}

/// E-47 -- env var name. Wire-dark default: emission silently
/// skipped until the operator sets this to a truthy value
/// (`"1"` / `"true"` / `"yes"` / `"on"`). Mirrors the precedent set
/// by [`ELAI_ROUTE_DECISION_SSB_ENABLE_ENV`] /
/// [`ELAI_CODING_METRIC_NAMED_TOKEN_ENABLE_ENV`] /
/// [`ELAI_E3_RUNTIME_HANDOFF_ENABLE_ENV`].
///
/// Consulted by
/// `crate::lora_ship_gate::lora_ship_gate_enabled_from_env` and the
/// `notify_lora_ship_gate` helper.
pub const ELAI_LORA_SHIP_GATE_ENABLE_ENV: &str =
    "ELAI_LORA_SHIP_GATE_ENABLE";

// ---------------------------------------------------------------------------
// E-40 Tranche B-2 (SCHEMA_VERSION 193) -- SymbolGraph augmentation
// substrate. Pattern-2 single-variant with 4-arm discriminator.
// ---------------------------------------------------------------------------

/// E-40 Tranche B-2 -- 4-arm kind discriminator carried by
/// [`SymbolGraphEventPayload`]. Pattern-2 (45th application).
///
/// Collapses the Tranche B-2 plan's nominally four sibling SSB rows
/// (`symbol_dsl_executed` / `dataflow_impact_computed` /
/// `agent_impact_claim_refuted` / `scope_graph_resolved`) into a
/// single Pattern-2 variant per the E-47 [`LoraShipGateEventPhase`]
/// precedent.
///
/// Wire labels are snake_case via `#[serde(rename_all)]`; `as_str()`
/// is the canonical helper for log-line and SSB dashboard labels.
///
/// Closed enum; adding a 5th arm requires a `SCHEMA_VERSION` bump.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SymbolGraphEventKind {
    /// `exec_dsl` returned with a match set.
    SymbolDslExecuted,
    /// `DataflowAnalyzer::impact_analysis` returned (any BFS impact
    /// request, including the 5th-list RRF retriever path).
    DataflowImpactComputed,
    /// The Refine+Filter stage refused an agent's impact claim
    /// because at least one impacted file was NOT in the claim.
    AgentImpactClaimRefuted,
    /// Reserved for Tranche B-3 -- `ScopeGraph` resolved a symbol's
    /// scope chain. The B-3 implementation is RUNPOD-PENDING; the
    /// arm is reserved here to avoid a 2nd SCHEMA bump on B-3 landing.
    ScopeGraphResolved,
}

impl SymbolGraphEventKind {
    /// Snake-case wire label.
    #[must_use]
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::SymbolDslExecuted => "symbol_dsl_executed",
            Self::DataflowImpactComputed => "dataflow_impact_computed",
            Self::AgentImpactClaimRefuted => "agent_impact_claim_refuted",
            Self::ScopeGraphResolved => "scope_graph_resolved",
        }
    }
}

/// E-40 Tranche B-2 (SCHEMA_VERSION 193) -- payload for
/// [`SsbActionType::SymbolGraphEvent`].
///
/// Wire-dark default; emission gated by
/// `ELAI_SYMBOL_GRAPH_5TH_RRF_ENABLE` (for the 5th-list RRF
/// retriever path) or other per-event gates upstream.
///
/// `kind` is the canonical outcome tag; `details_json` is a free-form
/// per-fire-point context object (DSL string, matched symbol names,
/// claim refutation diff, etc.).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SymbolGraphEventPayload {
    /// Outcome discriminator. See [`SymbolGraphEventKind`].
    pub kind: SymbolGraphEventKind,
    /// Canonical session id this event belongs to.
    pub session_id: Option<String>,
    /// Emitting agent identifier.
    pub agent_id: Option<String>,
    /// Free-form per-fire-point context.
    pub details_json: serde_json::Value,
    /// Fire-point wall-clock (millis since UNIX_EPOCH).
    pub ts_ms: i64,
}

/// E-40 Tranche B-2 -- env var name. Wire-dark default: emission
/// silently skipped until the operator sets this to a truthy value
/// (`"1"` / `"true"` / `"yes"` / `"on"`). Mirrors the precedent set
/// by [`ELAI_LORA_SHIP_GATE_ENABLE_ENV`].
///
/// Same env var also gates the
/// `elai_knowledge::pipeline` 5th-list RRF input (see
/// `crates/knowledge/src/pipeline.rs::ELAI_SYMBOL_GRAPH_5TH_RRF_ENABLE_ENV`)
/// so an operator opts the entire substrate on with one flip.
pub const ELAI_SYMBOL_GRAPH_5TH_RRF_ENABLE_ENV: &str =
    "ELAI_SYMBOL_GRAPH_5TH_RRF_ENABLE";

// ---------------------------------------------------------------------------
// Tranche B-6 quad (SCHEMA_VERSION 194) -- four new SSB substrates landing in
// one coordinated orchestrate run. See SCHEMA_VERSION header block for the
// full tranche docs. Layout below mirrors the SymbolGraphEvent pattern
// (single Pattern-2 variant + kind discriminator + payload + env-var const).
// ---------------------------------------------------------------------------

// === C-42 SHIPPABLE-NOW (proc-macro deferred to C-42b) ======================

/// Tranche B-6 quad C-42 -- 3-arm kind discriminator carried by
/// [`McpAuthzEventPayload`]. Pattern-2 (46th application).
///
/// Wire labels are snake_case via `#[serde(rename_all)]`; `as_str()` is
/// the canonical helper for log-line + SSB dashboard labels.
///
/// Closed enum; adding a 4th arm requires a `SCHEMA_VERSION` bump.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum McpAuthzEventKind {
    /// `check_authz` returned [`AuthzVerdict::Allow`] and the composed
    /// gate (ToolGate) also allowed the call -- per-call SSB signal.
    AuthzCheckPassed,
    /// `check_authz` returned [`AuthzVerdict::Deny`] or the composed
    /// ToolGate denied; carries hashed deny_reason in `details_json`.
    AuthzCheckFailed,
    /// One of the deterministic-corpus tuples was replayed against the
    /// composed gate (offline fuzz harness). `details_json.tuple_id`
    /// carries the corpus slot.
    CorpusReplayed,
}

impl McpAuthzEventKind {
    /// Snake-case wire label.
    #[must_use]
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::AuthzCheckPassed => "authz_check_passed",
            Self::AuthzCheckFailed => "authz_check_failed",
            Self::CorpusReplayed => "corpus_replayed",
        }
    }
}

/// Tranche B-6 quad C-42 -- payload for [`SsbActionType::McpAuthzEvent`].
///
/// Wire-dark default; emission gated by
/// [`ELAI_C42_MCP_AUTHZ_EMIT_ENABLE_ENV`].
///
/// `kind` is the canonical outcome tag; `details_json` is a free-form
/// per-fire-point context object (hashed deny_reason, layer label, corpus
/// tuple_id, etc.). Hashed identifiers (no plaintext) so the wire format
/// is privacy-preserving and stable across sessions.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct McpAuthzEventPayload {
    /// Outcome discriminator. See [`McpAuthzEventKind`].
    pub kind: McpAuthzEventKind,
    /// blake3-prefix u64 of the canonical snake_case tool name.
    pub tool_name_hash: u64,
    /// blake3-prefix u64 of the lowercased FSM state string ("planning",
    /// "executing", "auditing", ...).
    pub fsm_state_hash: u64,
    /// blake3-prefix u64 of the snake_case actor/role label ("planner",
    /// "executor", ...).
    pub actor_hash: u64,
    /// blake3-prefix u64 of the layer-attribution label
    /// (`"allow"` / `"authz"` / `"tool_gate"`). Mirrors paper Section 5's
    /// rejecting_layer surface.
    pub layer_hash: u64,
    /// blake3-prefix u64 of the deny reason string when `kind` is
    /// [`McpAuthzEventKind::AuthzCheckFailed`]; `None` for passed checks
    /// and corpus replay rows.
    pub deny_reason_hash: Option<u64>,
    /// Severity: `0` advisory (passed / corpus signal), `1` critical
    /// (authz deny). Stable wire format for dashboard sort.
    pub severity: u8,
    /// Free-form per-fire-point context (corpus tuple_id, fuzz seed,
    /// etc.).
    pub details_json: serde_json::Value,
    /// Fire-point wall-clock (millis since UNIX_EPOCH).
    pub ts_ms: i64,
}

/// Tranche B-6 quad C-42 -- env var name. Wire-dark default: emission
/// silently skipped until the operator sets this to a truthy value
/// (`"1"` / `"true"` / `"yes"` / `"on"`). Mirrors the precedent set by
/// [`ELAI_SYMBOL_GRAPH_5TH_RRF_ENABLE_ENV`] /
/// [`ELAI_LORA_SHIP_GATE_ENABLE_ENV`].
pub const ELAI_C42_MCP_AUTHZ_EMIT_ENABLE_ENV: &str =
    "ELAI_C42_MCP_AUTHZ_EMIT_ENABLE";

/// Tranche B-6 quad C-42 -- on-disk corpus directory for the
/// deterministic-tuple replay harness. Relative to the workspace root.
/// The directory is established with a `.gitkeep` sentinel; the corpus
/// generator (DEFERRED to C-42b alongside the proc-macro) populates it
/// with 200+ JSON tuples (tool * FSM state * 5 arg shapes).
pub const MCP_FUZZ_CORPUS_DIR: &str = ".elai_cc/data/mcp_fuzz_corpus";

/// Tranche M (s-22600) C-42b -- minimum tuple count for the
/// `plugins_macros` fuzz harness corpus. Per D-M8 (see
/// `workflow/reasoning/M-trancheM-decisions.md`), the deterministic
/// corpus generator at `crates/plugins_macros/examples/corpusgen.rs`
/// emits >= 200 `(tool_name, authz_decision, expected_emit_count)`
/// tuples; the contract test at
/// `crates/contracts/tests/m_authz_corpus_loop_contract.rs` pins the
/// floor against this constant so a future generator-tuning regression
/// surfaces as a compile-time test failure.
pub const MCP_FUZZ_CORPUS_MIN_TUPLES: usize = 200;

// === C-47 Pattern-2 collapse ================================================

/// Tranche B-6 quad C-47 -- 6-arm kind discriminator carried by
/// [`ToolCalibrationEventPayload`]. Pattern-2 (47th application).
///
/// Collapses the C-47 case-base surface into one Pattern-2 variant. The
/// 3 pre-existing helpers (`ToolCalibrationHintEmitted` /
/// `ToolCalibrationDriftDetected` / `ToolCallLogged`) stay alive for one
/// release window as deprecation aliases; the new variant + a new
/// notify-helper land here, gradual migration in C-47b.
///
/// Wire labels are snake_case via `#[serde(rename_all)]`; `as_str()` is
/// the canonical helper for log-line + SSB dashboard labels.
///
/// Closed enum; adding a 7th arm requires a `SCHEMA_VERSION` bump.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ToolCalibrationEventKind {
    /// Hinter emitted a calibration hint to the prompt-assembly side
    /// channel (mirrors the legacy `ToolCalibrationHintEmitted`).
    HintEmitted,
    /// Per-tool aggregate error_rate crossed the drift threshold
    /// (mirrors the legacy `ToolCalibrationDriftDetected`).
    DriftDetected,
    /// One completed tool dispatch was logged into the case base
    /// (mirrors the legacy `ToolCallLogged`).
    CallLogged,
    /// A new aggregated case was added to the case base (NEW signal --
    /// surfaces the case-base grow event).
    CaseAdded,
    /// A nearest-neighbor / aggregate-lookup retrieved a case from the
    /// base in service of a calibration decision.
    CaseSelected,
    /// A calibration outcome (hint application, sampling-rate tweak,
    /// etc.) was applied to a downstream tool dispatch.
    CalibrationApplied,
}

impl ToolCalibrationEventKind {
    /// Snake-case wire label.
    #[must_use]
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::HintEmitted => "hint_emitted",
            Self::DriftDetected => "drift_detected",
            Self::CallLogged => "call_logged",
            Self::CaseAdded => "case_added",
            Self::CaseSelected => "case_selected",
            Self::CalibrationApplied => "calibration_applied",
        }
    }
}

/// Tranche B-6 quad C-47 -- payload for
/// [`SsbActionType::ToolCalibrationEvent`].
///
/// Wire-dark default; emission gated by
/// [`ELAI_C47_CASE_BASE_ENABLE_ENV`].
///
/// Hashed identifiers (no plaintext) so the wire format is
/// privacy-preserving and stable across sessions (parity with C-45 /
/// C-46 hashing discipline).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ToolCalibrationEventPayload {
    /// Outcome discriminator. See [`ToolCalibrationEventKind`].
    pub kind: ToolCalibrationEventKind,
    /// blake3-prefix u64 of the snake_case tool name.
    pub tool_name_hash: u64,
    /// blake3-prefix u64 of the canonical-sorted call-shape JSON key set
    /// (mirrors `ToolCallLoggedPayload.call_shape_hash`).
    pub call_shape_hash: u64,
    /// blake3-prefix u64 of the snake_case response_kind string for
    /// `CallLogged`; `None` for kinds that do not carry a per-call
    /// response.
    pub response_kind_hash: Option<u64>,
    /// Free-form per-fire-point context (hint_text_hash, error_rate,
    /// total_calls, case_id, etc.).
    pub details_json: serde_json::Value,
    /// Fire-point wall-clock (millis since UNIX_EPOCH).
    pub ts_ms: i64,
}

/// Tranche B-6 quad C-47 -- env var name. Wire-dark default: emission
/// silently skipped until the operator sets this to a truthy value.
pub const ELAI_C47_CASE_BASE_ENABLE_ENV: &str =
    "ELAI_C47_CASE_BASE_ENABLE";

/// Tranche D (s-22600) C-47b -- env var name for the cold-tier rusqlite
/// persistence layer on top of C-47's in-memory LRU. Wire-dark default:
/// the registry only attempts a durable write when BOTH a
/// `CasePersistStore` has been installed via
/// `ToolCalibrationRegistry::with_persist` AND this env-var is set to a
/// truthy value (`"1"` / `"true"` / `"yes"` / `"on"`). Fail-closed
/// posture per D-trancheD-decisions.md D3: persist error SKIPS the LRU
/// admit so the in-memory view stays consistent with the durable store.
/// The plugins-crate canonical const value is
/// `"ELAI_C47B_CASE_BASE_PERSIST_ENABLE"`; the orchestrator-side const
/// here is a wire mirror so contract tests + dashboard wire layers can
/// reference one symbol regardless of crate boundary.
pub const ELAI_C47B_CASE_BASE_PERSIST_ENABLE_ENV: &str =
    "ELAI_C47B_CASE_BASE_PERSIST_ENABLE";

/// Tranche D (s-22600) C-47b -- canonical on-disk path for the sqlite
/// persistence store. Mirrors the `.elai_cc/data/` convention established
/// by [`MCP_FUZZ_CORPUS_DIR`] (Tranche B-6 quad). The bootstrap site
/// (cli + dashboard-v2) opens a `CasePersistStore` against this path
/// when wiring `ToolCalibrationRegistry::with_persist`. Relative to the
/// workspace root.
pub const CASE_BASE_PERSIST_PATH: &str = ".elai_cc/data/case_base/case_base.sqlite3";

// === B-39 single-add ========================================================

/// Tranche B-6 quad B-39 -- payload for
/// [`SsbActionType::ReflectBudgetSoftWarn`].
///
/// Single-add (NOT Pattern-2; reflect_* family stays
/// alphabetical-contiguous as the 5th sibling). Emitted by
/// `crates/reasoning/src/chat_reflect.rs` ~L454 when
/// `used >= soft_cap && used < hard_cap` where
/// `soft_cap = (hard_cap * REFLECT_BUDGET_SOFT_WARN_PCT) / 100`.
///
/// Wire-dark unless [`ELAI_B39_SOFTWARN_ENABLE_ENV`] is set truthy.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ReflectBudgetSoftWarnPayload {
    /// blake3-prefix u64 of the `turn_id` correlator (raw turn_id stays
    /// off the wire surface; the dashboard joins via SsbRow.agent_id +
    /// ts_ms when needed).
    pub turn_id_hash: u64,
    /// Tokens used at the point the soft-warn fired (sum across all
    /// stages reported by `ReflectTokenSummary`).
    pub used: u32,
    /// Computed soft cap = `(hard_cap * pct) / 100`. Carried explicitly
    /// so the dashboard does not have to re-derive it.
    pub soft_cap: u32,
    /// Hard cap (the existing `ChatReflectConfig.budget_cap_tokens`).
    pub hard_cap: u32,
    /// Threshold percentage that fired (mirrors
    /// `REFLECT_BUDGET_SOFT_WARN_PCT`; 0..=100).
    pub pct: u8,
    /// Fire-point wall-clock (millis since UNIX_EPOCH).
    pub ts_ms: i64,
}

/// Tranche B-6 quad B-39 -- env var name. Wire-dark default: emission
/// silently skipped until the operator sets this to a truthy value.
pub const ELAI_B39_SOFTWARN_ENABLE_ENV: &str =
    "ELAI_B39_SOFTWARN_ENABLE";

// === C-50 Pattern-2 collapse ================================================

/// Tranche B-6 quad C-50 -- 3-arm kind discriminator carried by
/// [`ContractCodingEventPayload`]. Pattern-2 (48th application).
///
/// Wire labels are snake_case via `#[serde(rename_all)]`; `as_str()` is
/// the canonical helper for log-line + SSB dashboard labels.
///
/// Pre-existing `ContractViolationDetected` stays alive (NOT collapsed
/// in this tranche -- separate landing).
///
/// Closed enum; adding a 4th arm requires a `SCHEMA_VERSION` bump.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ContractCodingEventKind {
    /// One fix_and_rerun iteration attempted (validator reported
    /// `Failed`; fixer turn dispatched). `details_json.iterations`
    /// carries the running counter.
    RetryAttempted,
    /// Validator pass after the fixer turn returned `Green` --
    /// retry-loop healed. `details_json.iterations` carries the final
    /// count.
    RetryResolved,
    /// Retry loop exhausted `max_iterations` and escalated to a human
    /// reviewer. `details_json.iterations` carries the cap value;
    /// `details_json.escalation_reason` carries a truncated failure
    /// summary.
    RetryAbandoned,
}

impl ContractCodingEventKind {
    /// Snake-case wire label.
    #[must_use]
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::RetryAttempted => "retry_attempted",
            Self::RetryResolved => "retry_resolved",
            Self::RetryAbandoned => "retry_abandoned",
        }
    }
}

/// Tranche B-6 quad C-50 -- payload for
/// [`SsbActionType::ContractCodingEvent`].
///
/// Wire-dark default; emission gated by
/// [`ELAI_C50_CONTRACT_RETRY_EMIT_ENABLE_ENV`].
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ContractCodingEventPayload {
    /// Outcome discriminator. See [`ContractCodingEventKind`].
    pub kind: ContractCodingEventKind,
    /// blake3-prefix u64 of the canonical step identifier (matches the
    /// `step_id` field on `fix_and_rerun_until_green`).
    pub step_id_hash: u64,
    /// Iterations counter. `u8` is sufficient: per `fix_and_rerun_contract.rs:278`
    /// the production cap is exactly 5; u8 max 255 leaves ample headroom.
    pub iterations: u8,
    /// Severity: `0` advisory (`RetryAttempted` / `RetryResolved`), `1`
    /// critical (`RetryAbandoned` -- human escalation). Stable wire format
    /// for dashboard sort.
    pub severity: u8,
    /// Free-form per-fire-point context (truncated failure_summary,
    /// escalation_reason, final_state label, ...).
    pub details_json: serde_json::Value,
    /// Fire-point wall-clock (millis since UNIX_EPOCH).
    pub ts_ms: i64,
}

/// Tranche B-6 quad C-50 -- env var name. Wire-dark default: emission
/// silently skipped until the operator sets this to a truthy value.
pub const ELAI_C50_CONTRACT_RETRY_EMIT_ENABLE_ENV: &str =
    "ELAI_C50_CONTRACT_RETRY_EMIT_ENABLE";

// === F-1 (SCHEMA_VERSION 196) -- VibeGateBlocked substrate ===============

/// F-1 (SCHEMA_VERSION 196) -- 4-arm kind discriminator carried by
/// [`VibeGateBlockedPayload`]. Pattern-2 (49th application).
///
/// Wire labels are snake_case; `as_sql_str()` is the canonical helper for
/// log-line + SSB dashboard labels and for the Pattern-2 `as_sql_str`
/// delegation in `SsbActionType::VibeGateBlocked`.
///
/// Closed enum; adding a 5th arm requires a `SCHEMA_VERSION` bump.
///
/// DepHallucinated and Skill are RESERVED this session (parse/sql arms
/// only; no producer fires them until the vibe-gate producer wiring lands
/// in a follow-on plan).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum VibeGateBlockedKind {
    /// A known-CWE code-level vulnerability was detected in a candidate
    /// edit (static analysis or SAST-rule match). Producer: vibe gate
    /// SAST arm (F-1 follow-on wiring).
    CodeVuln,
    /// A secret or credential pattern was detected in a candidate edit
    /// (regex or entropy-based scanner). Producer: vibe gate secret-scan
    /// arm (F-1 follow-on wiring).
    Secret,
    /// A dependency name that cannot be resolved in any known registry
    /// was detected (anti-slopsquatting guard). RESERVED -- parse/sql
    /// arm only; no producer this session.
    DepHallucinated,
    /// A third-party skill import failed the mcp-scan allowlist check.
    /// RESERVED -- parse/sql arm only; no producer this session.
    Skill,
}

impl VibeGateBlockedKind {
    /// Snake-case wire label used as the sql_str for each arm.
    #[must_use]
    pub fn as_sql_str(&self) -> &'static str {
        match self {
            Self::CodeVuln       => "vibesec_code_vuln_blocked",
            Self::Secret         => "vibesec_secret_blocked",
            Self::DepHallucinated => "vibesec_dep_hallucinated_blocked",
            Self::Skill          => "vibesec_skill_blocked",
        }
    }
}

/// F-1 (SCHEMA_VERSION 196) -- payload for
/// [`SsbActionType::VibeGateBlocked`].
///
/// Wire-dark default; emission gated by `ELAI_VIBE_GATE_ENABLE` env var
/// (see [`ELAI_VIBE_GATE_ENABLE_ENV`]).
///
/// Fields:
///   * `kind`          -- 4-arm fire-point discriminator (see
///     [`VibeGateBlockedKind`]).
///   * `rule_id`       -- optional identifier of the specific rule or
///     pattern that triggered the block (e.g. CWE ID string, secret-scan
///     rule name, registry slug). `None` when the blocking signal has no
///     named rule (e.g. entropy-only secret detection).
///   * `cwe`           -- optional CWE identifier string (e.g. `"CWE-89"`).
///     Populated only for `CodeVuln` arm; `None` for other arms.
///   * `file_path`     -- project-relative path of the file where the
///     violation was detected. Empty string when path is not applicable.
///   * `source_blake3` -- blake3 hex-prefix (first 16 chars) of the
///     candidate source snippet that triggered the block. Never the full
///     source text (privacy + size discipline).
///   * `citation`      -- optional URL or reference grounding the rule
///     (e.g. OWASP link, CVE advisory URL). `None` for rules with no
///     public citation.
///   * `fsm_state`     -- optional snake-case FSM state label at emission
///     time (e.g. `"executing"`, `"testing_first"`). `None` when the gate
///     fires outside the FSM cycle.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct VibeGateBlockedPayload {
    /// Fire-point discriminator. See [`VibeGateBlockedKind`].
    pub kind: VibeGateBlockedKind,
    /// Optional rule or pattern identifier.
    pub rule_id: Option<String>,
    /// Optional CWE identifier (CodeVuln arm only).
    pub cwe: Option<String>,
    /// Project-relative path of the file with the violation.
    pub file_path: String,
    /// blake3 hex-prefix of the triggering candidate snippet.
    pub source_blake3: String,
    /// Optional citation URL or reference.
    pub citation: Option<String>,
    /// Optional FSM state label at emission time.
    pub fsm_state: Option<String>,
}

/// F-1 (SCHEMA_VERSION 196) -- env var name for [`VibeGateBlockedPayload`]
/// emission. Wire-dark default: emission silently skipped until the
/// operator sets this to a truthy value (`"1"` / `"true"` / `"yes"` /
/// `"on"`). Mirrors the precedent set by [`ELAI_C50_CONTRACT_RETRY_EMIT_ENABLE_ENV`]
/// and other Pattern-2 substrates.
pub const ELAI_VIBE_GATE_ENABLE_ENV: &str = "ELAI_VIBE_GATE_ENABLE";

// === F-2a (SCHEMA_VERSION 196) -- SecurityRuleBlocked substrate ==========

/// F-2a (SCHEMA_VERSION 196) -- payload for
/// [`SsbActionType::SecurityRuleBlocked`].
///
/// Wire-dark default; emission gated by `ELAI_SECURITY_RULE_BLOCKED_ENABLE`
/// env var (see [`ELAI_SECURITY_RULE_BLOCKED_ENABLE_ENV`]).
///
/// Fields:
///   * `rule_id`          -- identifier of the security rule that blocked
///     the candidate (e.g. `"no_raw_exec"`, `"require_input_sanitize"`).
///   * `cwe`              -- CWE identifier string associated with the rule
///     (e.g. `"CWE-78"`). Populated by the rule author at rule definition
///     time.
///   * `file_path`        -- project-relative path of the file where the
///     rule violation was detected.
///   * `language`         -- snake-case language label (e.g. `"rust"`,
///     `"python"`, `"typescript"`). Lowercase; matches the language-tag
///     vocabulary used by the DKG engine.
///   * `candidate_blake3` -- blake3 hex-prefix (first 16 chars) of the
///     candidate edit blob. Never the full source text.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SecurityRuleBlockedPayload {
    /// Identifier of the security rule that blocked.
    pub rule_id: String,
    /// CWE identifier associated with the rule.
    pub cwe: String,
    /// Project-relative path where the rule violation was detected.
    pub file_path: String,
    /// Snake-case language label (e.g. `"rust"`, `"python"`).
    pub language: String,
    /// blake3 hex-prefix of the candidate edit blob.
    pub candidate_blake3: String,
}

/// F-2a (SCHEMA_VERSION 196) -- env var name for
/// [`SecurityRuleBlockedPayload`] emission. Wire-dark default.
pub const ELAI_SECURITY_RULE_BLOCKED_ENABLE_ENV: &str =
    "ELAI_SECURITY_RULE_BLOCKED_ENABLE";

// === F-4 / Slice B-1 (SCHEMA_VERSION 197) -- TestIntegrityEvent substrate ==

/// F-4 / Slice B-1 (SCHEMA_VERSION 197) -- 2-arm kind discriminator carried
/// by [`TestIntegrityEventPayload`]. Pattern-2 (50th application).
///
/// Wire labels are snake_case; `as_sql_str()` is the canonical helper for
/// log-line + SSB dashboard labels and for the Pattern-2 `as_sql_str`
/// delegation in `SsbActionType::TestIntegrityEvent`.
///
/// Closed enum; adding a 3rd arm requires a `SCHEMA_VERSION` bump.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TestIntegrityEventKind {
    /// A test run completed and its stdout hash was recorded for future
    /// comparison. The test PASSED (`passed=true` in the payload).
    /// Emitted by the test-integrity verifier after a confirmed passing run.
    Verified,
    /// A test run whose recorded stdout hash does NOT match the previously
    /// stored baseline -- indicating possible output fabrication or
    /// non-determinism. The pipeline blocks the baby-step from advancing.
    FabricationBlocked,
}

impl TestIntegrityEventKind {
    /// Snake-case wire label used as the sql_str for each arm.
    #[must_use]
    pub fn as_sql_str(&self) -> &'static str {
        match self {
            Self::Verified           => "test_integrity_verified",
            Self::FabricationBlocked => "anti_fabrication_block",
        }
    }
}

/// F-4 / Slice B-1 (SCHEMA_VERSION 197) -- payload for
/// [`SsbActionType::TestIntegrityEvent`].
///
/// Fields:
///   * `test_name`   -- name or identifier of the test that was run
///     (e.g. crate::module::test_fn_name or the test file path).
///   * `passed`      -- `true` if the underlying test passed (exit 0 /
///     assertion success); `false` on test failure. Distinct from the
///     `kind` discriminator -- a `Verified` kind implies `passed=true`;
///     a `FabricationBlocked` kind may have `passed` set to either value
///     depending on whether the output-hash mismatch was detected on a
///     passing or failing run.
///   * `stdout_hash` -- blake3 hex-prefix (first 16 chars) of the
///     captured stdout from the run. `None` when stdout was not captured
///     (e.g. non-output-capture callers, legacy test harness).
///   * `kind`        -- 2-arm fire-point discriminator. See
///     [`TestIntegrityEventKind`].
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TestIntegrityEventPayload {
    /// Name or identifier of the test run.
    pub test_name: String,
    /// `true` if the test passed; `false` on failure.
    pub passed: bool,
    /// blake3 hex-prefix of the captured stdout. `None` when not captured.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub stdout_hash: Option<String>,
    /// 2-arm fire-point discriminator. See [`TestIntegrityEventKind`].
    pub kind: TestIntegrityEventKind,
}

// === F-3p1 / Slice C1 (SCHEMA_VERSION 198) -- ToolFilterEvent substrate =====

/// F-3p1 / Slice C1 (SCHEMA_VERSION 198) -- 2-arm kind discriminator carried
/// by [`ToolFilterPayload`]. Pattern-2 (51st application).
///
/// Wire labels are snake_case; `as_sql_str()` is the canonical helper for
/// log-line + SSB dashboard labels and for the Pattern-2 `as_sql_str`
/// delegation in `SsbActionType::ToolFilterEvent`.
///
/// Closed enum; adding a 3rd arm requires a `SCHEMA_VERSION` bump.
///
/// `CapabilityFlowBlocked` is RESERVED (Tier-3) and is NOT an arm of this
/// enum -- it has no in-plan producer (W-8) and must not be added until a
/// plan ships a caller.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ToolFilterEventKind {
    /// A [`CommittedToolSet`](crate::capability_flow::CommittedToolSet) was
    /// successfully locked for a task. Emitted by
    /// `commit_minimal_set` callers at task-start time.
    Committed,
    /// A variable derived from an untrusted span (non-Clean
    /// [`ClassifierVerdict`](elai_safety::prompt_armor::ClassifierVerdict))
    /// was quarantined before reaching a tool call. Emitted by
    /// `taint_from_verdict` callers when they detect a taint and block
    /// the downstream action.
    UntrustedQuarantined,
}

impl ToolFilterEventKind {
    /// Snake-case wire label used as the sql_str for each arm.
    #[must_use]
    pub fn as_sql_str(&self) -> &'static str {
        match self {
            Self::Committed            => "tool_filter_committed",
            Self::UntrustedQuarantined => "untrusted_var_quarantined",
        }
    }
}

/// F-3p1 / Slice C1 (SCHEMA_VERSION 198) -- payload for
/// [`SsbActionType::ToolFilterEvent`].
///
/// Fields:
///   * `commit_hash`         -- blake3 64-hex digest from the
///     [`CommittedToolSet`](crate::capability_flow::CommittedToolSet).
///     Empty string for `UntrustedQuarantined` rows where no commitment
///     was active.
///   * `allowed_categories`  -- sorted snake_case labels of the committed
///     categories (e.g. `["read", "web"]`). Empty vec for
///     `UntrustedQuarantined` rows.
///   * `denied_tool`         -- the tool name that was denied, present for
///     `UntrustedQuarantined` rows; `None` for `Committed` rows.
///   * `taint_source`        -- verdict label (`"suspect"` / `"malicious"`)
///     that produced the taint, present for `UntrustedQuarantined` rows;
///     `None` for `Committed` rows.
///   * `task_id`             -- opaque task identifier supplied to
///     `commit_minimal_set`.
///   * `kind`                -- 2-arm fire-point discriminator. See
///     [`ToolFilterEventKind`].
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ToolFilterPayload {
    /// blake3 64-hex commit hash from [`CommittedToolSet`](crate::capability_flow::CommittedToolSet).
    pub commit_hash: String,
    /// Sorted snake_case labels of committed categories.
    pub allowed_categories: Vec<String>,
    /// Tool name that was denied (`UntrustedQuarantined` arm only).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub denied_tool: Option<String>,
    /// Taint source verdict label (`UntrustedQuarantined` arm only).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub taint_source: Option<String>,
    /// Opaque task identifier.
    pub task_id: String,
    /// 2-arm fire-point discriminator. See [`ToolFilterEventKind`].
    pub kind: ToolFilterEventKind,
}

// === F-9 / Slice D1 (SCHEMA_VERSION 199) -- WriteTimeSlopsquatBlocked substrate

/// F-9 / Slice D1 (SCHEMA_VERSION 199) -- 3-arm verdict kind for
/// [`WriteTimeSlopsquatBlockedPayload`].
///
/// Stored as a String in the payload (`verdict_kind` field) rather than
/// as a Pattern-2 enum variant so the SSB table CHECK constraint stays flat
/// (`write_time_slopsquat_blocked` single sql_str).
///
/// `RegistryUnreachable` is RESERVED; the gate is fail-open on unreachable
/// and MUST NOT emit a `WriteTimeSlopsquatBlocked` row for that case (W-10).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum WriteTimeSlopVerdictKind {
    /// Package was not found in the registry (slopsquat / hallucination
    /// candidate). The only arm that currently fires in production.
    NotFound,
    /// Package exists but its latest publish is older than the per-ecosystem
    /// staleness threshold (stale/abandoned fork). Produced by the
    /// SubprocessRegistryChecker stale arm (BLOCK-1 reconcile, F-Unit-D).
    TooNew,
    /// Registry probe was unreachable. The gate stays fail-open on unreachable
    /// (verdict Warn/Allow, never Block); revised W-10 emits this arm as an
    /// ADVISORY row so the fail-open leaves an observable trace ("no silent
    /// fail-open without an advisory SSB row").
    RegistryUnreachable,
}

impl WriteTimeSlopVerdictKind {
    /// Snake-case label stored in the `verdict_kind` String field.
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            Self::NotFound            => "not_found",
            Self::TooNew              => "too_new",
            Self::RegistryUnreachable => "registry_unreachable",
        }
    }
}

/// F-9 / Slice D1 (SCHEMA_VERSION 199) -- payload for
/// [`SsbActionType::WriteTimeSlopsquatBlocked`].
///
/// Emitted by `crate::vibesec::dep_writetime_gate::DepWriteTimeGate::analyze`
/// when a dependency resolves to `NotFound` or `TooNew` (blocking), or as an
/// ADVISORY row on `RegistryUnreachable` (fail-open, revised W-10).
///
/// Fields:
///   * `dep_name_hash`      -- 64-bit big-endian blake3 prefix of the
///     dependency name (never the raw name for privacy).
///   * `ecosystem`          -- `"cargo"` / `"pypi"` / `"npm"`.
///   * `verdict_kind`       -- snake_case [`WriteTimeSlopVerdictKind`] label
///     (`"not_found"` / `"too_new"` block the write; `"registry_unreachable"`
///     is emitted as an ADVISORY row on the fail-open path, revised W-10).
///   * `registry_age_days`  -- package age in days when the verdict is
///     `TooNew`; `None` for `NotFound`.
///   * `ts_ms`              -- wall-clock milliseconds since UNIX_EPOCH.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WriteTimeSlopsquatBlockedPayload {
    /// blake3 64-bit prefix of the dependency name.
    pub dep_name_hash: u64,
    /// Package ecosystem (`"cargo"` / `"pypi"` / `"npm"`).
    pub ecosystem: String,
    /// Verdict kind label.  See [`WriteTimeSlopVerdictKind`].
    pub verdict_kind: String,
    /// Package age in days (`TooNew` arm only).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub registry_age_days: Option<u32>,
    /// Wall-clock timestamp (ms since UNIX_EPOCH).
    pub ts_ms: i64,
}

// === F-10 / Slice A1 (SCHEMA_VERSION 200) -- SecurityClassRegression substrate

/// F-10 / Slice A1 (SCHEMA_VERSION 200) -- which production iteration loop
/// raised a security-class regression. This is a PAYLOAD field on
/// [`SecurityClassRegressionBlockedPayload`], NOT a discriminator on the
/// [`SsbActionType`] enum (the action type stays a flat beacon). The three
/// arms map one-to-one to the three production legs that call the guard:
///   * `Refine`      -- the REFINE call-site (a refine iteration would
///     introduce a new CWE class versus the prior candidate).
///   * `AutoAdvance` -- the AUTO-ADVANCE evidence-check (advancing to the
///     next step would carry a regressed security class).
///   * `SelfMod`     -- the SELF-MOD precondition (a self-modification would
///     introduce a CWE class absent from the pre-mod baseline).
///
/// Wire labels are snake_case (`refine` / `auto_advance` / `self_mod`).
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LoopKind {
    /// REFINE call-site leg.
    Refine,
    /// AUTO-ADVANCE evidence-check leg.
    AutoAdvance,
    /// SELF-MOD precondition leg.
    SelfMod,
}

/// F-10 / Slice A1 (SCHEMA_VERSION 200) -- payload for
/// [`SsbActionType::SecurityClassRegressionBlocked`].
///
/// Emitted by the security-monotonicity iteration guard when a candidate
/// iteration would introduce one or more CWE classes that were NOT present
/// in the prior iteration's classification (a non-monotone security
/// regression). Default-ON; emission is suppressed only when
/// `ELAI_SECURITY_MONOTONICITY_DISABLE` is set (kill-switch, never a
/// default-off ENABLE flag).
///
/// Fields:
///   * `delta_blake3`       -- blake3 hex-prefix (16 chars) of the candidate
///     edit blob whose classification regressed. Never the full source text.
///   * `introduced_classes` -- snake/CWE-id labels of the security classes
///     newly introduced by this iteration (the regression set; non-empty
///     when a row is emitted).
///   * `prior_classes`      -- the security-class labels present in the prior
///     iteration (the monotonic baseline being compared against).
///   * `loop_kind`          -- which production leg raised the regression.
///     See [`LoopKind`].
///   * `loop_id`            -- stable identifier of the iteration loop
///     instance (e.g. refine-loop / auto-advance step id) for correlation.
///   * `lang`               -- snake-case language label (e.g. `"rust"`,
///     `"python"`, `"typescript"`). Lowercase; matches the language-tag
///     vocabulary used by the F-1 vibegate scanner.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SecurityClassRegressionBlockedPayload {
    /// blake3 hex-prefix of the regressing candidate edit blob.
    pub delta_blake3: String,
    /// Security-class labels newly introduced by this iteration.
    pub introduced_classes: Vec<String>,
    /// Security-class labels present in the prior iteration.
    pub prior_classes: Vec<String>,
    /// Which production leg raised the regression.
    pub loop_kind: LoopKind,
    /// Stable identifier of the iteration loop instance.
    pub loop_id: String,
    /// Snake-case language label (e.g. `"rust"`, `"python"`).
    pub lang: String,
}

// === F-2b / Slice A1 (SCHEMA_VERSION 201) -- InsecureRateMeasured substrate

/// F-2b / Slice A1 (SCHEMA_VERSION 201) -- payload for
/// [`SsbActionType::InsecureRateMeasured`].
///
/// Emitted ONCE per static-gate insecure-rate benchmark run. The benchmark
/// runs the REAL ELAI static write-gate (`evaluate_vibegate`) over a
/// SeCodePLT-structured CWE corpus and records:
///   * `insecure_rate_before` -- the corpus ground-truth insecure-rate across
///     ALL candidate completions (the baseline insecure fraction).
///   * `insecure_rate_after`  -- the insecure-rate of the gate-APPROVED subset
///     (vulnerable completions the static gate did NOT Block; i.e. the static
///     false-negative residual).
///   * `gate_recall`          -- (before - after) / before; the fraction of
///     vulnerable completions the static gate caught.
///
/// This is an HONEST static-gate insecure-rate. It is DISTINCT from the
/// SeCodePLT dynamic secure-coding rate (sandbox-executed); the dynamic 3-way
/// score, functional pass@1, and the live-model number are a separate,
/// RunPod-gated measurement and are NOT carried here.
///
/// Fields:
///   * `suite`         -- benchmark suite label (e.g. the corpus / harness id).
///   * `model_id`      -- model identifier under test (the static gate is
///     model-independent, so this records the candidate-generation source for
///     provenance, not a gate input).
///   * `n_samples`     -- number of candidate completions evaluated.
///   * `seed`          -- deterministic RNG seed used to assemble the run.
///   * `corpus_blake3` -- blake3 hex-prefix of the corpus blob (provenance;
///     never the full corpus text).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct InsecureRateMeasuredPayload {
    /// Benchmark suite / harness label.
    pub suite: String,
    /// Model identifier of the candidate-generation source (provenance).
    pub model_id: String,
    /// Number of candidate completions evaluated.
    pub n_samples: u32,
    /// Corpus ground-truth insecure-rate across all candidates (before).
    pub insecure_rate_before: f64,
    /// Insecure-rate of the gate-approved subset (after = FN residual).
    pub insecure_rate_after: f64,
    /// Derived gate recall: (before - after) / before.
    pub gate_recall: f64,
    /// Deterministic RNG seed used to assemble the run.
    pub seed: u64,
    /// blake3 hex-prefix of the corpus blob (provenance).
    pub corpus_blake3: String,
}

// === I-9 (SCHEMA_VERSION 263) -- ErrorAmplificationMeasured substrate ========

/// I-9 (SCHEMA_VERSION 263) -- payload for
/// [`SsbActionType::ErrorAmplificationMeasured`].
///
/// Emitted ONCE per `elai bench error-amplification` run. The harness injects
/// a deterministic byte-perturbation at one step of a frozen pipeline fixture
/// and measures whether the verification/repair stack contracts the error
/// (low `alpha_t`) or amplifies it (high `alpha_t`). Flat beacon (no Kind
/// discriminator), mirroring [`InsecureRateMeasuredPayload`].
///
/// `alpha_t` is the fraction of downstream steps that still carry the error
/// after the repair stack acts; `recovered_fraction = 1.0 - alpha_t`. Both are
/// COMPUTED from the fixture walk, never literals. The `--no-repair` arm
/// (`repaired = false`) propagates fully (`alpha_t == 1.0`) so the instrument
/// can scream (R2). ID-5: this is ELAI's OWN measured baseline; no literature
/// alpha is carried as a target.
///
/// Fields:
///   * `suite`               -- benchmark suite label.
///   * `model_id`            -- fixture provenance id (the harness is
///     model-independent; this records the fixture source).
///   * `n_steps`             -- total pipeline steps in the fixture.
///   * `injected_step`       -- 0-indexed step where corruption was injected.
///   * `divergence_step`     -- 0-indexed step where the corrupted output first
///     diverges from canonical (mirrors the DriftComparator first-divergence
///     shape, H-23); `None` on a degenerate empty-output fixture.
///   * `alpha_t`             -- the amplification factor (propagated fraction).
///   * `recovered_fraction`  -- downstream fraction the repair stack recovered.
///   * `repaired`            -- `true` = canonical arm; `false` = no-repair R2
///     negative control.
///   * `annealed_temperature`-- the clamped C-26 temperature the harness drove
///     toward when `alpha_t` crossed the amplifying threshold mid-run; `None`
///     when no tighten fired.
///   * `seed`                -- deterministic RNG seed (`ELAI_SEED`).
///   * `fixture_blake3`      -- blake3 hex of the canonical fixture bytes
///     (provenance; never the full fixture text).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ErrorAmplificationMeasuredPayload {
    /// Benchmark suite / harness label.
    pub suite: String,
    /// Fixture provenance id.
    pub model_id: String,
    /// Total pipeline steps in the fixture.
    pub n_steps: u32,
    /// 0-indexed step where corruption was injected.
    pub injected_step: u32,
    /// 0-indexed first-divergence step (`None` on a degenerate fixture).
    pub divergence_step: Option<u32>,
    /// The amplification factor (downstream propagated fraction), `[0,1]`.
    pub alpha_t: f64,
    /// Downstream fraction the repair stack recovered, `[0,1]`.
    pub recovered_fraction: f64,
    /// `true` = canonical (repaired) arm; `false` = no-repair R2 control.
    pub repaired: bool,
    /// Clamped C-26 annealing temperature when alpha crossed the threshold.
    pub annealed_temperature: Option<f64>,
    /// Deterministic RNG seed used for the byte-perturbation.
    pub seed: u64,
    /// blake3 hex of the canonical fixture bytes (provenance).
    pub fixture_blake3: String,
}

// === F-6 / Slice A1 (SCHEMA_VERSION 202) -- SkillLibraryEvent substrate ======

/// F-6 / Slice A1 (SCHEMA_VERSION 202) -- 3-arm kind discriminator carried by
/// [`SkillLibraryEventPayload`]. Pattern-2 (single variant on
/// [`SsbActionType`]; this 3-arm Kind selects the wire tag).
///
/// Wire labels are snake_case; `as_sql_str()` is the canonical helper for
/// log-line + SSB dashboard labels and for the Pattern-2 `as_sql_str`
/// delegation in `SsbActionType::SkillLibraryEvent`.
///
/// Closed enum; adding a 4th arm requires a `SCHEMA_VERSION` bump.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SkillLibraryEventKind {
    /// A new skill was acquired into the learned-skill library (born at the
    /// S1 floor after a critic-gated success in C6b).
    Acquired,
    /// A skill climbed one trust rung (S1->S2, S2->S3, or the human-granted
    /// S3->S4 apex jump).
    TrustPromoted,
    /// A skill was knocked down one rung by the C6c staleness sweep because
    /// its staleness score crossed the demotion threshold.
    DemotedStale,
}

impl SkillLibraryEventKind {
    /// Snake-case wire label used as the sql_str for each arm.
    #[must_use]
    pub fn as_sql_str(&self) -> &'static str {
        match self {
            Self::Acquired      => "skill_acquired",
            Self::TrustPromoted => "skill_trust_promoted",
            Self::DemotedStale  => "skill_demoted_stale",
        }
    }
}

/// F-6 / Slice A1 (SCHEMA_VERSION 202) -- payload for
/// [`SsbActionType::SkillLibraryEvent`].
///
/// Fields:
///   * `skill_id`     -- stable id of the learned skill.
///   * `description`  -- human-readable skill description.
///   * `tier`         -- snake-case [`crate::skill_trust::SkillTrustTier`]
///     label of the skill AFTER the event (e.g. `"s1"` on acquire, the new
///     rung on promote/demote).
///   * `prior_tier`   -- snake-case tier label BEFORE the event; `None` on
///     `Acquired` (no prior rung).
///   * `reason`       -- short human-readable cause (e.g. the gate that
///     cleared, or the staleness score that fired the demotion).
///   * `invocations`  -- cumulative successful invocations at event time.
///   * `failure_rate` -- failure rate (failures / total) at event time.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SkillLibraryEventPayload {
    /// Stable id of the learned skill.
    pub skill_id: String,
    /// Human-readable skill description.
    pub description: String,
    /// Snake-case skill tier AFTER the event.
    pub tier: String,
    /// Snake-case skill tier BEFORE the event; `None` on `Acquired`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub prior_tier: Option<String>,
    /// Short human-readable cause for the event.
    pub reason: String,
    /// Cumulative successful invocations at event time.
    pub invocations: u32,
    /// Failure rate (failures / total) at event time.
    pub failure_rate: f64,
    /// 3-arm fire-point discriminator. See [`SkillLibraryEventKind`].
    pub kind: SkillLibraryEventKind,
}

// === F-5 / Slice A1 (SCHEMA_VERSION 203) -- StalenessQuarantineBroadcast =====

/// F-5 / Slice A1 (SCHEMA_VERSION 203) -- payload for
/// [`SsbActionType::StalenessQuarantineBroadcast`].
///
/// Emitted ONCE per quarantined fact at the orchestrator/CLI memory-decay-tick
/// boundary. A decay tick that drives a fact's staleness ratio across the decay
/// threshold quarantines the fact; the broadcast cascades the quarantine to the
/// downstream subsystems (DKG `concept_stale=2` write-through + a non-destructive
/// `staleness_quarantined` marker edge in `dkg_edges`, and Cosmos artifact
/// flagging) via a bounded `QuarantineBroadcastQueue` drained by the CLI. The
/// emit lives at the orchestrator/CLI boundary (the decay-scheduler
/// `DecayTickObserver` hook OR the inline `live_cli.rs` decay tick), NOT inside
/// `crates/memory` `decay.rs`: `crates/memory` has no dependency on
/// `crates/knowledge`, so the cascade is applied by the CLI (which depends on
/// both).
///
/// Fields:
///   * `fact_id`                  -- stable id of the quarantined fact.
///   * `staleness_ratio`          -- decay staleness ratio at quarantine time
///     (0.0 fresh .. 1.0+ fully stale); the value that crossed the threshold.
///   * `dependents_demoted`       -- count of DKG concepts demoted to
///     `concept_stale=2` by this cascade.
///   * `cosmos_artifacts_flagged` -- count of Cosmos artifacts flagged stale by
///     this cascade.
///   * `tick_ms`                  -- wall-clock ms of the decay tick that fired
///     the quarantine.
///   * `broadcast_id`             -- unique id for this broadcast (blake3
///     hex-prefix of `fact_id` + `tick_ms`); correlates the SSB row with the
///     queue record and the cascade application.
///   * `dropped_count`           -- number of queue records DROPPED-OLDEST due
///     to `QUARANTINE_QUEUE_CAPACITY` overflow at emit time (W-2 no silent
///     drop; 0 in the steady state).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct StalenessQuarantineBroadcastPayload {
    /// Stable id of the quarantined fact.
    pub fact_id: String,
    /// Decay staleness ratio at quarantine time (the value that crossed the
    /// threshold).
    pub staleness_ratio: f64,
    /// Count of DKG concepts demoted to `concept_stale=2` by this cascade.
    pub dependents_demoted: u32,
    /// Count of Cosmos artifacts flagged stale by this cascade.
    pub cosmos_artifacts_flagged: u32,
    /// Wall-clock ms of the decay tick that fired the quarantine.
    pub tick_ms: u64,
    /// Unique broadcast id (blake3 hex-prefix of `fact_id` + `tick_ms`).
    pub broadcast_id: String,
    /// Queue records DROPPED-OLDEST due to capacity overflow at emit time.
    pub dropped_count: u32,
}

// === F-8 / Slice A1 (SCHEMA_VERSION 204) -- SymbolicConflictDetected =========

/// F-8 / Slice A1 (SCHEMA_VERSION 204) -- payload for
/// [`SsbActionType::SymbolicConflictDetected`].
///
/// Emitted ONCE per flagged self-mod config variant at the `elai self-mod
/// apply` CLI entrypoint (a W-3 production entrypoint) when the LLM-free SCF
/// symbolic detector flags a contradiction between a proposed `ConfigVariant`
/// and an active locked-key invariant (`self_mod_safety::LOCKED_KEYS`) BEFORE
/// archive admission. POLICY is WARN-then-escalate: the variant is held
/// PENDING (a non-blocking held outcome), NEVER hard-fail-closed BLOCK. The
/// 27.9% SCF precision figure is sim-only / UNVERIFIED and is explicitly NOT a
/// target -- it is not encoded anywhere in this payload or its thresholds.
///
/// The detector ALSO composes at the F-10 darwinian-evolve seam
/// (`external_benches.rs`) so the same beacon fires inside the
/// `#[cfg(feature = "darwinian-evolve")]` promotion loop when that ships.
///
/// Fields (all `String` -- a flat symbolic record, no numeric thresholds on the
/// wire):
///   * `fingerprint`  -- blake3 hex-prefix of the conflict tuple
///     (`kind` + `key` + `detail`); stable id correlating the SSB row
///     with the held-pending record. (F-7 doc fix: the impl in
///     `symbolic_conflict.rs::conflict_fingerprint` hashes `kind + key +
///     detail`, NOT `candidate_id`; the doc previously misstated the third
///     component. DOC-ONLY -- the hash itself is unchanged.)
///   * `kind`         -- conflict class (e.g. the `TripwireKind` name, or a
///     locked-key-contradiction label); free-form symbolic tag.
///   * `key`          -- the locked invariant key that the variant contradicts
///     (one of `self_mod_safety::LOCKED_KEYS`, verified via `is_locked`).
///   * `detail`       -- human-readable contradiction detail (what the variant
///     proposed vs the locked invariant); for the WARN message + dashboard.
///   * `candidate_id` -- stable id of the proposed `ConfigVariant` that was
///     flagged and held pending.
///   * `fsm_substate` -- the FSM agent sub-state at flag time (provenance of
///     the admission attempt).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SymbolicConflictPayload {
    /// blake3 hex-prefix of the conflict tuple (`kind` + `key` + `detail`).
    pub fingerprint: String,
    /// Conflict class / symbolic tag (e.g. a `TripwireKind` name or a
    /// locked-key-contradiction label).
    pub kind: String,
    /// The locked invariant key the variant contradicts (one of
    /// `self_mod_safety::LOCKED_KEYS`).
    pub key: String,
    /// Human-readable contradiction detail (proposed vs locked invariant).
    pub detail: String,
    /// Stable id of the flagged-and-held-pending `ConfigVariant`.
    pub candidate_id: String,
    /// FSM agent sub-state at flag time (provenance of the admission attempt).
    pub fsm_substate: String,
}

// === F-7 / Slice A1 (SCHEMA_VERSION 205) -- TemplateUtilityRanked ============

/// F-7 / Slice A1 (SCHEMA_VERSION 205) -- payload for
/// [`SsbActionType::TemplateUtilityRanked`].
///
/// Emitted ONCE per crystallized-pipeline template whose Laplace-smoothed
/// outcome utility is computed at the `try_execute` gate-threshold seam (the
/// W-3 production entrypoint) to decide whether the cached template is reused.
/// The cache holds exactly ONE template per `(concept_class, plan_skeleton_hash)`
/// key, so this is a GATE-THRESHOLD swap (utility vs the raw
/// `first_try_success_rate`), NOT a between-candidates sort.
///
/// `utility` is `laplace_utility(successes, total) = (successes + ALPHA) /
/// (total + ALPHA + BETA)` with `ALPHA = BETA = 1.0` (ELAI-owned constants).
/// Add-one (Laplace) smoothing means a 1/1 template scores `2/3 ~= 0.667`
/// rather than `1.0`, and a 95/100 template scores `~0.941`, so a single lucky
/// firing no longer over-trusts a low-sample template. `successes` and `total`
/// are derived from the PERSISTED `CompiledPipeline.first_try_success_rate` +
/// `use_count` read via the `CompiledPipelineStore` -- NOT from SSB
/// compiled-pipeline events (those carry NO per-firing pass/fail signal).
///
/// `first_try_ratio` is the recorded raw baseline (the persisted
/// `first_try_success_rate`) kept alongside the smoothed `utility` for
/// before/after dashboard display (W-2 no-removal -- the baseline is preserved,
/// not replaced).
///
/// `block_ratio` and `coverage` are DELIBERATELY ABSENT: there is no producer
/// for them on this path, and zero-filling absent signals would be a CSQ
/// footgun (a 0.0 would read as "fully blocked / zero coverage" when the truth
/// is "unknown"). They are documented as FUTURE signals to add only once a real
/// producer exists; this slice ships Laplace over success/total only.
///
/// The Springdrift +10.7% figure is UNGROUNDED and is explicitly NOT a target:
/// it is not encoded anywhere in this payload or its thresholds. F-7 gates on
/// wiring + smoothing-correctness, not a numeric uplift.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TemplateUtilityRankedPayload {
    /// Concept-class key the crystallized template is bucketed under (one half
    /// of the cache key).
    pub concept_class: String,
    /// Stable hash of the plan skeleton (the other half of the cache key).
    pub plan_skeleton_hash: u64,
    /// Laplace-smoothed outcome utility:
    /// `(successes + ALPHA) / (total + ALPHA + BETA)`, `ALPHA = BETA = 1.0`.
    pub utility: f32,
    /// Raw recorded baseline: the persisted
    /// `CompiledPipeline.first_try_success_rate` (kept for before/after
    /// dashboard display, W-2 no-removal).
    pub first_try_ratio: f32,
    /// Sample size (`total` firings) backing the utility, i.e. the persisted
    /// `use_count`; low values are exactly where Laplace smoothing matters.
    pub sample_n: u32,
    /// Stable id of the crystallized pipeline template whose utility was ranked.
    pub chosen_pipeline_id: String,
}

// === F-13 / Slice A1 (SCHEMA_VERSION 206) -- SessionArchived ================

/// F-13 / Slice A1 (SCHEMA_VERSION 206) -- payload for
/// [`SsbActionType::SessionArchived`].
///
/// Emitted ONCE per cold-archived store at the `run_migration` destructive-DROP
/// site (the production SSB open path, `Ssb::new`) when a stale-version DB is
/// opened with `ELAI_SSB_ARCHIVE_ON_BUMP_ENABLE` set. BEFORE the DROP, the
/// stale `ssb.db` (and, precautionarily, the `session_dag.db`) is copied to
/// `workflow/sessions/{id}/archive/ssb-v{N}.sqlite`; the DROP then proceeds
/// unconditionally (the archive is ADDITIVE, killer_risk #5). The row is
/// emitted via a DIRECT synchronous INSERT on the writer connection AFTER the
/// recreate, because the async single-writer task does not exist yet at
/// migration time.
///
/// SINGLE FLAT beacon (no Pattern-2 Kind). `store_kind` is a PAYLOAD FIELD
/// (`"ssb"` | `"session_dag"`), NOT a discriminator — one variant covers both
/// store copies. The `manifest_blake3` integrity prefix is computed from the
/// ARCHIVED bytes (blake3 ONLY, CLAUDE.md HARD RULE; mirrors checkpoint.rs
/// content_hash) so the cold archive is integrity-checkable later.
///
/// This guard ships DEFAULT-OFF by design (forensic opt-in over data the
/// migration comment marks "data loss is acceptable for old sessions"); the
/// W-2 disposition + the wire-dark-allowlist row are recorded in
/// workflow/reasoning/F-13-session-archive-decisions.md.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SessionArchivedPayload {
    /// The session whose stale-version store was archived.
    pub session_id: String,
    /// The on-disk `user_version` the store carried BEFORE the bump (the
    /// version embedded in the archive file name `*-v{stale_version}.sqlite`).
    pub stale_version: i64,
    /// Relative path of the archive file under the per-session workflow dir,
    /// e.g. `workflow/sessions/{id}/archive/ssb-v{N}.sqlite`.
    pub archive_path: String,
    /// blake3 hex-prefix of the archived bytes (integrity check; blake3 ONLY).
    pub manifest_blake3: String,
    /// Which store was archived: `"ssb"` (the destructive journal) or
    /// `"session_dag"` (the precautionary, non-destructive copy).
    pub store_kind: String,
}

// === F-11 / Slice A1 (SCHEMA_VERSION 207) -- DarwinianAdmit ==================

/// F-11 (SCHEMA_VERSION 207) -- 3-arm discriminator for
/// [`DarwinianAdmitPayload`] (Pattern-2 single-variant).
///
/// Lifted into a payload-internal enum to preserve the single-variant pattern
/// on [`SsbActionType`]. Mirrors the E-7 [`DarwinianPhase`] layout. Encoded via
/// `serde rename_all = "snake_case"` so the wire tags are
/// `admitted` / `rejected_by_guard` / `loop_closed`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DarwinianAdmitKind {
    /// Candidate passed all four guards and was inserted into the config
    /// archive (`config_archive::insert_variant`). The FIRST production caller
    /// of the DGM archive admit path.
    Admitted,
    /// A guard rejected the candidate before admit. `guard_rejected_by` on the
    /// payload names the firing guard (`tripwire` / `trap` / `security` /
    /// `two_gate`). The candidate was NOT inserted.
    RejectedByGuard,
    /// `config_archive::select_parent` ran AFTER `insert_variant` for this
    /// generation -- the loop is closed (propose -> evaluate -> guard ->
    /// insert -> select-next-parent). This is the loop-closure proof.
    LoopClosed,
}

impl DarwinianAdmitKind {
    /// Snake-case wire label.
    #[must_use]
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Admitted => "admitted",
            Self::RejectedByGuard => "rejected_by_guard",
            Self::LoopClosed => "loop_closed",
        }
    }
}

/// F-11 (SCHEMA_VERSION 207) -- payload for
/// [`SsbActionType::DarwinianAdmit`].
///
/// Emitted by the config-DGM admit-loop driver
/// (`crates/cli/src/darwin_loop_cmd.rs` -> [`crate::darwin_admit_loop`]) once
/// per candidate routed through the four-guard chain, plus one `LoopClosed`
/// beacon per closed generation. Wire-dark default; emission gated by
/// `ELAI_DARWINIAN_LOOP_ENABLE` env var AND `cfg(feature = "darwinian-evolve")`
/// cargo feature.
///
/// `candidate_blake3` is a blake3 hex-prefix (16 chars) of the candidate
/// `config_blob` -- blake3 ONLY (CLAUDE.md HARD RULE; never blake2/sha2). No
/// hard-coded threshold numbers live in this payload; `alpha` is the real,
/// candidate-derived evaluation score.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DarwinianAdmitPayload {
    /// Admit/reject/loop-closed discriminator. See [`DarwinianAdmitKind`].
    pub kind: DarwinianAdmitKind,
    /// Stable id of the candidate config variant the loop evaluated.
    pub variant_id: String,
    /// Parent variant id the candidate mutated from (`None` for a root seed on
    /// an empty archive).
    pub parent_id: Option<String>,
    /// Real evaluation score in `[0.0, 1.0]` derived from the candidate (the
    /// Two-Gate margin basis). NOT a synthetic/mock constant.
    pub alpha: f64,
    /// Name of the firing guard on `RejectedByGuard` (`tripwire` / `trap` /
    /// `security` / `two_gate`); `None` on `Admitted` / `LoopClosed`.
    pub guard_rejected_by: Option<String>,
    /// blake3 hex-prefix (16 chars) of the candidate `config_blob` (blake3
    /// ONLY).
    pub candidate_blake3: String,
    /// Deterministic seed the generation ran under (`ELAI_SEED` / `--seed`).
    pub seed: u64,
}

// ---------------------------------------------------------------------------
// G-plan payload structs and Kind enums (SCHEMA_VERSION 208-224)
// ---------------------------------------------------------------------------

/// G-4: MCP rug-pull blocked at http_list_tools gate.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct McpRugPullBlockedPayload {
    pub server_url: String,
    pub schema_blake3_before: String,
    pub schema_blake3_after: String,
    pub changed_tools: Vec<String>,
    pub description_diff_count: u32,
    pub ingest_origin: String,
}

/// G-1: taint propagation kind.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum TaintFlowKind {
    Web,
    ToolOutput,
    File,
    Sandbox,
}

impl TaintFlowKind {
    pub fn label(&self) -> &'static str {
        match self {
            Self::Web => "web",
            Self::ToolOutput => "tool_output",
            Self::File => "file",
            Self::Sandbox => "sandbox",
        }
    }
}

/// G-1: data-plane taint flow blocked at destructive sink.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct TaintFlowPayload {
    pub kind: TaintFlowKind,
    pub origin_blake3: String,
    pub trust_level: String,
    pub sink_tool: Option<String>,
    pub blocked: bool,
    pub task_id: String,
}

/// G-3: blast-radius gate block kind.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum BlastRadiusKind {
    FsEscape,
    EgressDenied,
}

impl BlastRadiusKind {
    pub fn label(&self) -> &'static str {
        match self {
            Self::FsEscape => "fs_escape",
            Self::EgressDenied => "egress_denied",
        }
    }
}

impl From<elai_safety::blast_radius::BlastRadiusKind> for BlastRadiusKind {
    fn from(k: elai_safety::blast_radius::BlastRadiusKind) -> Self {
        use elai_safety::blast_radius::BlastRadiusKind as SafetyKind;
        match k {
            SafetyKind::FsEscape => Self::FsEscape,
            SafetyKind::EgressDenied => Self::EgressDenied,
        }
    }
}

/// G-3: blast-radius gate block.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct BlastRadiusBlockedPayload {
    pub kind: BlastRadiusKind,
    pub pre_expansion_command: String,
    pub post_expansion_command: String,
    pub target_or_host: String,
    pub worktree_root_blake3: String,
    pub trust_label: String,
    pub ts_ms: i64,
}

/// G-7: lethal trifecta quarantine.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct TrifectaQuarantinedPayload {
    pub task_id: String,
    pub untrusted_origin_blake3: String,
    pub sink_tool: String,
    pub secret_present: bool,
    pub untrusted_span: bool,
    pub egress_available: bool,
    pub quarantined: bool,
    pub sub_llm_verdict: String,
}

/// G-2: reproducible-run artifact sealed.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct RunArtifactEmittedPayload {
    pub root_blake3: String,
    pub guarantee_level: String,
    pub gate_count: u32,
    pub seed: u64,
    pub session_id: String,
    /// I-45: 16-hex blake3 objective hash sealed at run time. `None` on LEGACY
    /// receipts (pre-I-45 runs where no IdeaFramingStarted row was present).
    #[serde(default)]
    pub objective_hash: Option<String>,
}

/// G-5: reviewer-cockpit DSR score.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct ReviewSurfaceScoredPayload {
    pub session_id: String,
    pub dsr: f64,
    pub total_changed_lines: u32,
    pub attention_lines: u32,
    pub vouched_hunks: u32,
    pub unvouched_hunks: u32,
    pub base_commit: String,
    pub diff_available: bool,
    pub diff_unavailable_reason: Option<String>,
}

/// G-8: signed in-toto provenance attestation emitted.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct ProvenanceAttestationEmittedPayload {
    pub session_id: String,
    pub subject_blake3: String,
    pub run_artifact_root: String,
    pub verified_levels: u32,
    pub all_passed: bool,
    pub signing_model: String,
}

/// G-6: SAFEDELTA gate-ON vs gate-OFF measurement.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct SafetyDeltaMeasuredPayload {
    pub suite: String,
    pub model_id: String,
    pub safety_delta: f64,
    pub ci_low: f64,
    pub ci_high: f64,
    pub n_tasks: u32,
    pub seed: u64,
    pub corpus_blake3: String,
    pub gate_recall: f64,
    pub stdout_hash: Option<String>,
    pub runpod_pending: bool,
}

/// I-38: QDELTA defective-diff rate gate-ON vs gate-OFF measurement.
///
/// Flat beacon (no Kind discriminator). Mirrors [`SafetyDeltaMeasuredPayload`] shape.
/// `delta` = `off_defective_rate` - `on_defective_rate` (positive = gates improve quality).
/// `runpod_pending = true` for Tier-1 fixture runs (no live inference, D-026).
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct QdeltaMeasuredPayload {
    /// Suite label (e.g. "test_qdelta_fixture" for Tier-1).
    pub suite: String,
    /// Model/candidate provenance label.
    pub model_id: String,
    /// Number of paired tasks evaluated.
    pub n_tasks: u32,
    /// Number of independent seeds (floor >= 5).
    pub seed_count: u32,
    /// `true` = same tasks measured gates-ON and gates-OFF (paired design, floor = true).
    pub paired: bool,
    /// Fraction of tasks where gates-ON produced a defective diff.
    pub on_defective_rate: f64,
    /// Fraction of tasks where gates-OFF produced a defective diff.
    pub off_defective_rate: f64,
    /// QDELTA = off_defective_rate - on_defective_rate. Positive = gates improve quality.
    pub delta: f64,
    /// Seeded bootstrap 95% CI lower bound (GD-2: recomputable from pinned seed).
    pub ci_low: f64,
    /// Seeded bootstrap 95% CI upper bound.
    pub ci_high: f64,
    /// Cost per task in USD (GD-3 $/task policy).
    pub dollar_per_task: f64,
    /// D-20 contamination probe result: "not_run" | "honestly_evaluated" | "game_with_memorization".
    pub contamination_probe_result: String,
    /// Per-task surviving mutants from I-20. 0.0 at Tier-1 (I-20 stub, pending I-20 ship).
    #[serde(default)]
    pub surviving_mutants_per_task: f64,
    /// blake3 hex of the config bytes (provenance pin; anti-gaming).
    pub corpus_blake3: String,
    /// `true` = Tier-1 fixture only; live-model leg is deferred (D-026).
    pub runpod_pending: bool,
}

/// I-43: discriminator arm for [`ClaimScopeMeasuredPayload`].
///
/// - `PilotDelta`: reduced-N gate-ON vs gate-OFF pilot; carries `delta`, `reviewer_reaching_rate`,
///   and `trap_fired` (block-everything trap detector).
/// - `CrossLaneDivergence`: local-Qwen vs cloud-Claude action-stream divergence on security-critical
///   paths; carries `divergence_pct` and `claim_wording`.
/// - `ClaimScoped`: consolidated go/no-go verdict; carries both pilot and divergence results +
///   the final `claim_wording` the system may honestly use (GD-5).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ClaimScopeMeasuredKind {
    /// Reduced-N pilot signal: delta sign + magnitude + reviewer-reaching trap detector.
    PilotDelta,
    /// Cross-lane divergence: local vs cloud action-stream divergence on security paths.
    CrossLaneDivergence,
    /// Consolidated go/no-go memo with final honest claim wording (GD-5).
    ClaimScoped,
}

impl ClaimScopeMeasuredKind {
    /// Stable snake_case wire string for the SSB kind arm.
    #[must_use]
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::PilotDelta => "pilot_delta",
            Self::CrossLaneDivergence => "cross_lane_divergence",
            Self::ClaimScoped => "claim_scoped",
        }
    }
}

/// I-43 payload for [`SsbPayload::ClaimScopeMeasured`].
///
/// Emitted by `cli::bench_claim_scope::run_inner` for each fixture scenario.
/// Three arms driven by [`ClaimScopeMeasuredKind`]; fields unused by a given arm are `None`.
///
/// Off-agent artifact (R4): written at the bench call site, not by the logic being measured.
/// Missing rows = FAIL (R1): absence means the kill-gate never ran.
///
/// `reviewer_reaching_rate` is derived as `1.0 - gate_recall` (not a field on any runner report).
/// It detects the block-everything trap: a gate that wins only by blocking near-everything has
/// a very low reviewer-reaching rate and fails the pilot.
///
/// GD-2: all values recompute byte-identically under the same `seed` and fixture data.
/// GD-5: `claim_wording` reflects what the system can HONESTLY claim (never "deterministic replay"
/// on the cloud path if `divergence_pct > 5.0`).
/// TOTAL_VARIANTS 544 -> 545.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct ClaimScopeMeasuredPayload {
    /// Discriminator arm.
    pub kind: ClaimScopeMeasuredKind,
    /// Suite label (e.g. "seccodeplt_claim_scope_fixture" for Tier-1).
    pub suite: String,
    /// ELAI_SEED value at the time of evaluation (GD-2 reproducibility).
    pub seed: u64,
    /// Number of tasks in the pilot run (N, reduced for cheap-first framing).
    pub n_samples: u32,
    /// `PilotDelta` arm: residual-insecure-rate delta (gate-ON minus gate-OFF). Positive = gate helps.
    #[serde(default)]
    pub delta: Option<f64>,
    /// `PilotDelta` arm: proportion of candidates that reach a human reviewer under gate-ON.
    /// Derived as `1.0 - gate_recall`. Near-zero = block-everything trap.
    #[serde(default)]
    pub reviewer_reaching_rate: Option<f64>,
    /// `PilotDelta` arm: `true` if `reviewer_reaching_rate < 0.05` (block-everything trap fired).
    #[serde(default)]
    pub trap_fired: bool,
    /// `CrossLaneDivergence` arm: % of security-path actions that differ between local and cloud lanes.
    #[serde(default)]
    pub divergence_pct: Option<f64>,
    /// `CrossLaneDivergence` / `ClaimScoped` arm: honest claim wording the harness can support (GD-5).
    #[serde(default)]
    pub claim_wording: Option<String>,
    /// `true` = Tier-1 fixture only; live-model leg deferred (D-026).
    pub runpod_pending: bool,
    /// blake3 hex of the fixture bytes (provenance pin, GD-2).
    pub corpus_blake3: String,
}

// ---------------------------------------------------------------------------
// I-10: CacheEconomicsEvent payload structs
// ---------------------------------------------------------------------------

/// I-10: 3-arm discriminator for [`CacheEconomicsEventPayload`].
///
/// Mirrors [`crate::cache_economics::CacheEconomicsEventKind`] (same enum,
/// kept in ssb.rs as the SSB wire type; cache_economics.rs re-exports via
/// `use crate::ssb::CacheEconomicsEventKind`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub enum CacheEconomicsEventKind {
    /// Planner selected a TTL tier for a cloud call.
    TtlPlanned,
    /// Guard detected unexplained miss-rate exceeding threshold (>10%).
    HitRateGuardTripped,
    /// Guard detected a prefix mutation; blake3 byte-range offset dumped.
    PrefixDiffDumped,
}

/// I-10: SSB payload for prefix invariance + TTL tiering + cache-hit hard guard.
///
/// Pattern-2 single-variant; kind discriminator is [`CacheEconomicsEventKind`].
/// Emitted by `orchestrator::cache_economics::emit_cache_economics_event`.
/// Default-ON; kill-switch `ELAI_CACHE_ECONOMICS_DISABLE=1`.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct CacheEconomicsEventPayload {
    /// Which arm of the event.
    pub kind: CacheEconomicsEventKind,
    /// TTL in seconds (3600 = 1h, 300 = 5m, None = no-cache).
    #[serde(default)]
    pub ttl_secs: Option<u32>,
    /// Breakeven read count for the selected tier.
    pub breakeven_reads: u32,
    /// Measured miss-rate at guard evaluation (0.0 when N/A).
    pub miss_rate: f64,
    /// Blake3 8-byte prefix of the detected mutable byte range (all-zeros when N/A).
    pub mutation_hash: [u8; 8],
    /// Call class label (e.g. "scoring", "grounding", "chat").
    pub call_class: String,
    /// Milliseconds since UNIX epoch at emit time.
    pub ts_ms: i64,
}

// ---------------------------------------------------------------------------
// I-8: TrajectoryAborted payload structs
// ---------------------------------------------------------------------------

/// I-8: 3-arm discriminator for [`TrajectoryAbortedPayload`].
///
/// - `Voluntary`: the model explicitly called `acknowledge_infeasible`.
/// - `DeadEndHash`: a pre-dispatch fingerprint match pruned an isomorphic
///   re-attempt before any model call (zero tokens).
/// - `HaltingBound`: the empirical check-runner watchdog fired after the
///   configured bound elapsed (I-17 seam; Wave-1 registers the variant,
///   Wave-3 wires the actual timer).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub enum TrajectoryAbortedKind {
    /// Model called `acknowledge_infeasible` (explicit voluntary abort).
    Voluntary,
    /// Dead-end hash cache collision — isomorphic re-attempt pruned pre-dispatch.
    DeadEndHash,
    /// Halting-bound watchdog fired (I-17 watchdog seam; Wave-1 stub).
    HaltingBound,
}

/// I-8: SSB payload for a voluntarily-aborted or pruned trajectory.
///
/// Pattern-2 single-variant; kind discriminator is [`TrajectoryAbortedKind`].
/// Emitted by:
///   - `production_agent_rpc.rs` `TOOL_ACKNOWLEDGE_INFEASIBLE` arm (Voluntary).
///   - `dead_end_cache.rs` check path after `check_and_insert` returns `true`
///     (DeadEndHash — Wave-1 wiring in task_runner).
///   - I-17 check-runner watchdog hook (HaltingBound — Wave-3).
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct TrajectoryAbortedPayload {
    /// Which arm triggered the abort.
    pub kind: TrajectoryAbortedKind,
    /// Session id at abort time (informational; SSB row also carries it).
    #[serde(default)]
    pub session_id: String,
    /// Optional trajectory id for correlation with earlier SSB rows.
    #[serde(default)]
    pub trajectory_id: Option<String>,
    /// blake3 hex digest of the (task, approach) fingerprint.
    /// `Some` for DeadEndHash; `None` for Voluntary/HaltingBound.
    #[serde(default)]
    pub approach_hash: Option<String>,
    /// Human-readable abort reason from the model's `acknowledge_infeasible` call.
    /// `Some` for Voluntary; `None` for DeadEndHash/HaltingBound.
    #[serde(default)]
    pub abort_reason_text: Option<String>,
    /// Milliseconds since UNIX epoch at emit time.
    pub ts_ms: i64,
}

// ---------------------------------------------------------------------------
// I-1: DirectiveExecuted payload structs
// ---------------------------------------------------------------------------

/// I-1: 3-arm discriminator for [`DirectiveExecutedPayload`].
///
/// - `Executed`: the gate chain passed and the side effect ran cleanly.
/// - `Discarded`: a gate blocked the directive; the side effect never ran
///   (the generator cannot reach the sink — fail-closed CANT-WRECK).
/// - `CompensationFired`: the side effect crashed and E-44 saga compensation
///   restored the pre-op state.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub enum DirectiveExecutedKind {
    /// Gates passed; side effect ran cleanly.
    Executed,
    /// A gate blocked the directive; side effect never ran.
    Discarded,
    /// Side effect crashed; E-44 saga compensation fired to restore state.
    CompensationFired,
}

/// I-1: SSB payload for a directive that reached the supervisor.
///
/// Pattern-2 single-variant; kind discriminator is [`DirectiveExecutedKind`].
/// Emitted by `directive::InProcessSupervisor::execute_directive`. The
/// generator emits pure `Directive` structs; the Supervisor is the sole
/// side-effect executor and the sole producer of this row.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct DirectiveExecutedPayload {
    /// Which terminal outcome the directive reached.
    pub kind: DirectiveExecutedKind,
    /// The tool the directive targeted (e.g. `bash_tool_use`, `file_write`).
    pub tool_name: String,
    /// Correlation id linking back to the intent that produced the directive.
    pub intent_id: String,
    /// Who authored the intent (agent id / role).
    pub initiator: String,
    /// Milliseconds since UNIX epoch at emit time.
    pub ts_ms: i64,
}

// ---------------------------------------------------------------------------
// I-33: GovernancePrimitiveFired payload structs
// ---------------------------------------------------------------------------

/// I-33: 3-arm discriminator for [`GovernancePrimitiveFiredPayload`].
///
/// - `CeflExternalized`: CEFL candidate set was injected into the LLM call;
///   model-invented options outside the schema are structurally unparseable
///   (selection-by-negation foreclosed at the type level).
/// - `I6qRejected`: rationale failed the I6Q length+diversity gate; decision
///   refused; downstream routes to D-16 `RefusalDecision::Abstain`.
/// - `CommitRevealSealed`: blake3 commit row written before reveal; ordering
///   integrity verified (commit_row_id < reveal_row_id = PASS).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub enum GovernancePrimitiveFiredKind {
    /// CEFL candidate externalization fired; schema enforces the domain.
    CeflExternalized,
    /// I6Q rationale validator rejected the response; routes to Abstain.
    I6qRejected,
    /// Commit-reveal sealed; ordering integrity confirmed.
    CommitRevealSealed,
}

/// I-33: SSB payload for a mechanical decision governance primitive that fired.
///
/// Pattern-2 single-variant; kind discriminator is [`GovernancePrimitiveFiredKind`].
/// Emitted by `decision_governance::emit_governance_fired` at the
/// `governed_prompt_decision` call site and by the `bench_governance` fixture driver.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct GovernancePrimitiveFiredPayload {
    /// Which governance primitive fired.
    pub kind: GovernancePrimitiveFiredKind,
    /// Session that triggered the primitive (None in fixture runs).
    pub session_id: Option<String>,
    /// Primitive-specific details as a JSON string (e.g. candidate list,
    /// rationale snippet, commit hash hex). Must not exceed 4 KiB.
    pub details_json: String,
    /// Milliseconds since UNIX epoch at emit time.
    pub ts_ms: i64,
}

/// I-32: Discriminator for a structural role-scope contract application event.
///
/// Pattern-2 single-variant; 3-arm; alphabetical order maintained.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub enum RoleContractAppliedKind {
    /// The A/B evidence harness fired (`--role-ab-probe`); before/after fixture
    /// pass-rate fields + delta are populated in the payload.
    AbDelta,
    /// Narrative "You are..." persona was stripped and replaced with schema-first
    /// role contract text. Emitted for factual/code roles (Executor, Researcher,
    /// ContextCompiler, Designer, MemoryWriter).
    PersonaStripped,
    /// Role is alignment-dependent (Auditor, IdeaCritic, InjectionClassifier) or
    /// unknown; the contract store has no entry; original framing left intact.
    PersonaRetained,
}

/// I-32: SSB payload for a structural role-scope contract application event.
///
/// Pattern-2 single-variant; kind discriminator is [`RoleContractAppliedKind`].
/// Emitted by `report_fns::build_system_prompt_for_role` (PersonaStripped /
/// PersonaRetained) and by the `--role-ab-probe` CLI harness (AbDelta).
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct RoleContractAppliedPayload {
    /// Which outcome occurred.
    pub kind: RoleContractAppliedKind,
    /// Snake-case role name that triggered the event (e.g. `"executor"`).
    pub role: String,
    /// Fixture pass-rate before role-contract application (AbDelta only; 0.0 otherwise).
    pub fixture_pass_rate_old: f32,
    /// Fixture pass-rate after role-contract application (AbDelta only; 0.0 otherwise).
    pub fixture_pass_rate_new: f32,
    /// Delta = new - old (negative = regression; evidence gate: must be >= 0.0 for production flip).
    pub delta: f32,
    /// Milliseconds since UNIX epoch at emit time.
    pub ts_ms: i64,
}

/// I-35: discriminator for a temporal tool-sequence enforcement verdict.
///
/// Pattern-2 single-variant; 3-arm; alphabetical order maintained.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub enum ToolSequenceVerdictKind {
    /// All enabled rules that triggered on this tool class were satisfied.
    /// The call proceeds normally.
    Allowed,
    /// A Halt-enforced rule's predicate was not satisfied; the call is refused
    /// fail-closed (consistent with the `Discarded` directive outcome).
    HaltEnforced,
    /// A WarnOnly rule's predicate was not satisfied; the call proceeds but
    /// an advisory finding is recorded.
    OrderViolation,
}

/// I-35: SSB payload for a temporal tool-sequence enforcement verdict.
///
/// Pattern-2 single-variant; kind discriminator is [`ToolSequenceVerdictKind`].
/// Emitted by `directive::InProcessSupervisor::run_sequence_gate` (the 5th
/// supervisor gate arm) for every dispatch cycle where a sequence rule triggers.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct ToolSequenceVerdictPayload {
    /// Which verdict was produced.
    pub kind: ToolSequenceVerdictKind,
    /// The logical tool class that triggered the evaluation (e.g. `"git_push"`).
    pub tool_class: String,
    /// The `rule_id` of the triggered rule (empty string for `Allowed`).
    pub rule_id: String,
    /// Human-readable description of the unsatisfied predicate (empty for `Allowed`).
    pub unsatisfied_predicate: String,
    /// Milliseconds since UNIX epoch at emit time.
    pub ts_ms: i64,
}

/// I-41: discriminant tag for [`SsbActionType::CritiqueLensFinding`].
///
/// Copy/Eq/Hash C-like enum used exclusively by `SsbActionType` for routing /
/// hashing. The full payload (with `String` and `f32` fields) lives in
/// [`CritiqueLensFindingKind`] inside [`CritiqueLensFindingPayload`].
/// Pattern-2: one tag enum for `SsbActionType`, one rich enum for `SsbPayload`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub enum CritiqueLensFindingTag {
    /// A lens produced a finding with a valid non-empty location anchor.
    AdvisoryFinding,
    /// A finding was dropped because the location field was empty.
    DroppedNoAnchor,
    /// Emitted once per lens at the start of a run (observability beacon).
    LensRun,
}

/// I-41: discriminator for a spec-time critique lens finding (full-payload variant).
///
/// Used in [`CritiqueLensFindingPayload`] inside [`SsbPayload`].
/// Structurally distinct from `idea_framing::Severity` — no `Into<>` impls exist.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub enum CritiqueLensFindingKind {
    /// A lens produced a finding with a valid non-empty `location` anchor.
    AdvisoryFinding {
        /// Lens identifier (e.g. "L1", "L4").
        lens: String,
        /// Severity string ("FLAG", "CONCERN", "NOTE").
        severity: String,
        /// Non-empty location anchor from the finding.
        location: String,
        /// The claim text.
        claim: String,
        /// Confidence in [0.0, 1.0].
        confidence: f32,
    },
    /// A finding was dropped because the `location` field was empty (no anchor).
    /// The DroppedNoAnchor row is the only evidence; the finding is NOT surfaced.
    DroppedNoAnchor {
        /// Lens identifier of the dropped finding.
        lens: String,
    },
    /// Emitted once per lens at the start of a run (observability beacon).
    LensRun {
        /// Lens identifier being run.
        lens: String,
    },
}

/// I-41: SSB payload for a spec-time critique lens advisory beacon.
///
/// Pattern-2 single-variant; kind discriminator is [`CritiqueLensFindingKind`].
/// Emitted by `critique::lenses::LensSet::run` on the CLI (`elai critique`) and
/// as a post-convergence advisory from `idea_framing::run_critic_loop`.
/// `advisory=true` is invariant (compile-time structural isolation from gate plane).
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct CritiqueLensFindingPayload {
    /// Which kind of event this is.
    pub kind: CritiqueLensFindingKind,
    /// Always `true`. Advisory rows never affect any gate or verdict.
    pub advisory: bool,
    /// Session that triggered the lens run.
    pub session_id: String,
}

// ---------------------------------------------------------------------------
// I-46: BashRedirected payload structs
// ---------------------------------------------------------------------------

/// I-46: 3-arm discriminator for [`BashRedirectedPayload`].
///
/// - `Redirected`: the command matched a redirect rule (grep/cat/find/head/tail/less/more/rg).
///   The process was NOT spawned; the model receives a redirect hint.
/// - `FixedUp`: the command had trailing output-pipeline noise stripped (| head / | tail /
///   redundant 2>&1). The rewritten command is returned in the error for the model to retry.
/// - `PrefixBlocked`: the command starts with an allowed prefix but its body contains a banned
///   shell token (control char, unquoted expansion, newline, backslash escape).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub enum BashRedirectedKind {
    /// Command matched a redirect rule; directed to a dedicated ELAI tool instead.
    Redirected,
    /// Command had trailing noise stripped; rewritten command returned for retry.
    FixedUp,
    /// Command starts with an allowed prefix but contains a banned shell token.
    PrefixBlocked,
}

/// I-46: SSB payload for a bash interceptor interception event.
///
/// Pattern-2 single-variant; kind discriminator is [`BashRedirectedKind`].
/// Emitted by `dispatch::in_process::InProcessDispatch::maybe_emit_bash_redirected`
/// when `run_bash` returns `Err` with `error_code` `bash_redirected`,
/// `bash_fixedup`, or `bash_prefix_blocked`.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct BashRedirectedPayload {
    /// Which interception arm fired.
    pub kind: BashRedirectedKind,
    /// The original bash command (before any fixup or redirect).
    pub command: String,
    /// For `Redirected`: the ELAI tool the model should call instead (e.g. `"grep_search"`).
    /// `None` for `FixedUp` and `PrefixBlocked`.
    pub substitute_call: Option<String>,
    /// For `FixedUp`: the rewritten command that was returned to the caller.
    /// `None` for `Redirected` and `PrefixBlocked`.
    pub fixup_applied: Option<String>,
    /// For `PrefixBlocked`: the matched allowed-prefix (first token of the command).
    /// `None` for `Redirected` and `FixedUp`.
    pub prefix_matched: Option<String>,
}

// ---------------------------------------------------------------------------
// I-47: PromptHardeningApplied payload structs
// ---------------------------------------------------------------------------

/// I-47: 3-arm discriminator for [`PromptHardeningAppliedPayload`].
///
/// - `InputsTagged`: auditor/critic dispatch -- untrusted worker output wrapped in
///   `<ELAI_DATA>` boundary tags with standing hardening frame prepended above.
/// - `RulesInjected`: worker dispatch -- frozen per-role Karpathy-distillate rule block
///   appended to CONSTRAINTS section (HD-2 byte-stable per role, cache-prefix safe).
/// - `SelfAttestationStripped`: auditor/critic dispatch -- a self-attestation phrase detected;
///   labelled as claim-not-evidence inside `<ELAI_SELF_ATTESTATION>` sub-tag.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub enum PromptHardeningAppliedKind {
    /// Auditor/critic dispatch: untrusted inputs wrapped in `<ELAI_DATA>` tags.
    InputsTagged,
    /// Worker dispatch: frozen per-role rule block appended to CONSTRAINTS section.
    RulesInjected,
    /// Auditor/critic dispatch: self-attestation phrase detected and labelled.
    SelfAttestationStripped,
}

/// I-47: SSB payload for a prompt-hardening application event.
///
/// Pattern-2 single-variant; kind discriminator is [`PromptHardeningAppliedKind`].
/// Emitted by `tools::execute_agent_with_spawn` once per dispatch where hardening was applied.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct PromptHardeningAppliedPayload {
    /// Which hardening arm fired.
    pub kind: PromptHardeningAppliedKind,
    /// Canonical role string as returned by `normalize_subagent_type`.
    pub role: String,
    /// Session-scoped agent id of the dispatch being hardened.
    pub agent_id: String,
    /// For `SelfAttestationStripped`: the matched attestation phrase. `None` otherwise.
    pub attestation_phrase: Option<String>,
}

// ---------------------------------------------------------------------------
// I-2: BranchWorkspaceEvent payload structs
// ---------------------------------------------------------------------------

/// I-2: 4-arm discriminator for [`BranchWorkspaceEventPayload`].
///
/// - `Spawned`: a private branch workspace root was created (Tier-1: warm git worktree).
/// - `Merged`: the deterministic SSB-seq-ordered barrier merged all branch roots into
///   trunk cleanly (last-seq-wins on non-overlapping relative paths).
/// - `EscapeBlocked`: a branch attempted a write whose target resolves OUTSIDE its root;
///   blocked BEFORE the FS mutation (containment delegated to
///   `elai_safety::blast_radius::check_fs_target`).
/// - `ConflictEscalated`: two branches wrote the SAME relative path with differing bytes;
///   Tier-1 HALTS the merge (no partial write). H-49 trunk-rollback is the Tier-2 deferred path.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub enum BranchWorkspaceEventKind {
    /// A private branch workspace root was created.
    Spawned,
    /// The deterministic barrier merged all branch roots into trunk cleanly.
    Merged,
    /// A branch tried to write outside its root; blocked pre-FS-mutation.
    EscapeBlocked,
    /// Two branches wrote the same path with differing bytes; merge halted.
    ConflictEscalated,
}

/// I-2: SSB payload for a CoW workspace isolation barrier event.
///
/// Pattern-2 single-variant; kind discriminator is [`BranchWorkspaceEventKind`].
/// Emitted by `sandbox::branch_workspace::WorktreePoolImpl` (spawn/escape_check) and the
/// BSP barrier site (`bsp/engine.rs` `barrier_merge`). The `worktree_root_blake3` field
/// uses the same `elai_safety::blast_radius::worktree_root_blake3` helper as the
/// blast-radius SSB adapter so a forensic reader can group rows by root.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct BranchWorkspaceEventPayload {
    /// Which barrier arm fired.
    pub kind: BranchWorkspaceEventKind,
    /// Logical branch identifier (also the git worktree branch name).
    pub branch_id: String,
    /// Session the branch belongs to (empty for barrier-scoped Merged/Conflict rows
    /// where the session is implied by the run).
    pub session_id: String,
    /// blake3 hex-prefix of the branch (or merge-target) root path. An identifier,
    /// not a secret; blake3 ONLY per the CLAUDE.md HARD RULE.
    pub worktree_root_blake3: String,
    /// For `EscapeBlocked`/`ConflictEscalated`: the offending/conflicting path
    /// (post-expansion for escapes, relative path for conflicts). `None` otherwise.
    pub conflict_path: Option<String>,
}

// ---------------------------------------------------------------------------
// I-5: ContextPruned payload structs
// ---------------------------------------------------------------------------

/// I-5: 3-arm discriminator for [`ContextPrunedPayload`].
///
/// - `SiblingExcluded`: sibling branch SSB payloads were hard-excluded from the fresh branch
///   context per the ID-1 stateless-generator doctrine. The raw `excluded_bytes` carried the
///   accumulated sibling outputs; after exclusion they are absent from the rebuilt context.
/// - `TracebackContrasted`: a prior failed branch's raw error traceback (>= threshold bytes) was
///   replaced by a `<=30-token` contrastive constraint phrase. Preserves the causal signal ("attempt
///   A failed at line-42 type X -- avoid X") without the attention-sink payload.
/// - `VividnessProbe`: a sampled A/B comparison arm. Emitted only when
///   `ELAI_REASONING_QUALITY_PROBE_ENABLE=1` (the E-19 umbrella gate). Carries `outcome_delta`
///   (pristine_score - accumulated_score). Recomputable under `ELAI_SEED` (GD-2).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub enum ContextPrunedKind {
    /// Sibling branch payloads excluded from the fresh branch context (ID-1).
    SiblingExcluded,
    /// Failed-attempt traceback replaced by a <=30-token contrastive constraint phrase.
    TracebackContrasted,
    /// Sampled A/B outcome delta (pristine minus accumulated); gated by E-19 umbrella.
    VividnessProbe,
}

/// I-5: SSB payload for a per-branch context rebuild outcome.
///
/// Pattern-2 single-variant; kind discriminator is [`ContextPrunedKind`].
/// Emitted by `pristine_path_filter::PristinePathFilter::build_branch_context`.
///
/// Wire-dark until I-4 `SearchLadder::run_best_of_n` ships (Wave 4).
/// Tier-1 coverage via `crates/contracts/tests/i5_pristine_context_contract.rs`.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct ContextPrunedPayload {
    /// Which pruning arm fired.
    pub kind: ContextPrunedKind,
    /// Logical branch identifier for the branch being rebuilt.
    pub branch_id: String,
    /// Session the branch belongs to.
    pub session_id: String,
    /// For `SiblingExcluded`: total bytes excluded from sibling payloads. 0 otherwise.
    pub excluded_bytes: u64,
    /// For `TracebackContrasted`: byte length of the original traceback. 0 otherwise.
    pub original_traceback_bytes: u64,
    /// For `TracebackContrasted`: the <=30-token contrastive constraint phrase injected.
    /// `None` for other kinds.
    pub constraint_phrase: Option<String>,
    /// For `VividnessProbe`: outcome delta (pristine_score - accumulated_score). `None` for other
    /// kinds. Recomputable under `ELAI_SEED` (GD-2).
    pub outcome_delta: Option<f64>,
}

// ---------------------------------------------------------------------------
// I-11: BeliefStateEvent payload structs
// ---------------------------------------------------------------------------

/// I-11: 3-arm discriminator for [`BeliefStateEventPayload`].
///
/// - `Condensed`: emitted for every `adaptive_reduce` condensation (ID-3 JOURNALED-MIDDLEWARE
///   rule). Un-journaled condensation is a wiring-enforcement FAIL. Carries `token_delta`.
/// - `Handoff`: emitted when `evaluate_handoff` returns `HandoffDecision::Cloud` and the
///   BeliefState payload is injected into the cloud call. Carries `payload_bytes`.
/// - `HonestyDelta`: emitted by the periodic honesty job (every `HONESTY_JOB_INTERVAL`
///   escalations). Compares full-history token count vs BeliefState-compressed count.
///   Deterministic under `ELAI_SEED` (GD-2); recomputable from the SSB stream.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub enum BeliefStateEventKind {
    /// Condensation receipt (ID-3): emitted on every `adaptive_reduce` call.
    Condensed,
    /// Handoff dispatch receipt: emitted when BeliefState payload is sent to cloud.
    Handoff,
    /// Honesty-job delta ledger entry: full-history vs BeliefState delta.
    HonestyDelta,
}

/// I-11: SSB payload for PABU belief-state handoff receipts.
///
/// Pattern-2 single-variant; kind discriminator is [`BeliefStateEventKind`].
/// Emitted by `belief_state::{BeliefState::emit_condensed_receipt,
/// BeliefState::emit_handoff_receipt, HonestyJob::run}`.
/// Production wiring: `task_runner::drive_task` (I-11 block, post `adaptive_reduce`).
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct BeliefStateEventPayload {
    /// Which arm of the belief-state event.
    pub kind: BeliefStateEventKind,
    /// Wire format version of the BeliefState blob (BELIEF_STATE_SCHEMA_VERSION = 1).
    pub schema_version: u32,
    /// Tokens saved by `adaptive_reduce` before encoding (0 for `HonestyDelta`).
    pub token_delta: u32,
    /// Size in bytes of the encoded BeliefState payload (0 for `Condensed` and `HonestyDelta`).
    pub payload_bytes: u32,
    /// Milliseconds since UNIX epoch at emit time.
    pub ts_ms: i64,
}

/// I-13: 3-arm discriminator for [`MemoryGuardEventPayload`].
///
/// - `PoisonBlocked`: emitted when MemoryWriteScanner detects invisible-Unicode or confusable-shell
///   pattern in a candidate memory write. The write is REFUSED (fail-closed). Carries `pattern_class`.
/// - `DualSerialized`: emitted when a ToolResultPayload passes through `to_llm_content` gate before
///   entering the agent context window. Carries `token_delta` (full - summarized token count).
/// - `TraceQueryExpanded`: emitted when the `trace_query` tool returns a line-range expansion.
///   Carries `byte_range` as `"path:start-end"`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub enum MemoryGuardEventKind {
    /// Write blocked by aho-corasick scan: invisible-Unicode or confusable-shell pattern detected.
    PoisonBlocked,
    /// ToolResultPayload serialized through `to_llm_content` dual-serialization gate.
    DualSerialized,
    /// TraceQuery line-range expansion completed.
    TraceQueryExpanded,
}

/// I-13: SSB payload for write-time memory guard events.
///
/// Pattern-2 single-variant; kind discriminator is [`MemoryGuardEventKind`].
/// Emitted by `user_memory_capture` (CHOKEPOINT-1), `memory_cmd::insert` (CHOKEPOINT-2),
/// and `production_agent_rpc::translate_tool_request` (TraceQuery arm).
/// Production wiring: both CLI chokepoints call `MemoryWriteScanner::scan` which triggers
/// the emit; the TraceQuery arm wires directly in `production_agent_rpc.rs`.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct MemoryGuardEventPayload {
    /// Which arm of the memory-guard event.
    pub kind: MemoryGuardEventKind,
    /// For `PoisonBlocked`: the matched pattern class ("invisible_unicode" | "confusable_shell").
    pub pattern_class: Option<String>,
    /// For `DualSerialized`: tokens saved by `to_llm_content` summarization (full - llm count).
    pub token_delta: Option<u32>,
    /// For `TraceQueryExpanded`: the file range as `"<path>:<start>-<end>"`.
    pub byte_range: Option<String>,
    /// Milliseconds since UNIX epoch at emit time.
    pub ts_ms: i64,
}

/// I-14: 3-arm discriminator for [`ContextTransformReceiptPayload`].
///
/// - `LspSlice`: a `rustc`/cargo panic payload was sliced to the located error
///   neighbourhood (the long backtrace tail dropped); the error location is parsed from
///   the tool-output text, never from the LSP oracle (which only probes toolchain
///   presence).
/// - `AstFold`: tree-sitter function-body elision (function bodies collapsed to
///   `/* elided */`; signatures and types preserved so BPE semantic priors survive).
/// - `Tier2Compressed`: reserved discriminator for the DEFERRED neural (LLMLingua-2-class)
///   tier; no Tier-2 encoder ships in the Tier-1 changeset.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub enum ContextTransformKind {
    /// Compiler-error payload sliced to the located error neighbourhood.
    LspSlice,
    /// Tree-sitter function-body elision (signatures kept).
    AstFold,
    /// Reserved discriminator for the deferred neural tier.
    Tier2Compressed,
}

/// I-14: SSB payload for deterministic context-compression middleware events
/// (ID-3 JOURNALED-MIDDLEWARE: a reversible receipt for every transform).
///
/// Pattern-2 single-variant; kind discriminator is [`ContextTransformKind`].
/// Emitted by `ContextTransformer::transform` (context_transform.rs) on EVERY transform,
/// including a graceful skip (toolchain absent / non-UTF-8 / nothing to fold). The
/// `reverse_hash` is the full 32-byte blake3 of the ORIGINAL bytes, hex-encoded: replay
/// reconstructs what the model saw by matching it against the journaled original.
/// Production wiring: the `AgentRpcServer::transform_tool_output` override in
/// `production_agent_rpc.rs`, invoked from `ipc_accept.rs` `run_agent_session` at the
/// tool-result byte seam.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct ContextTransformReceiptPayload {
    /// Which transform strategy produced this receipt.
    pub kind: ContextTransformKind,
    /// Estimated token count of the ORIGINAL (pre-transform) payload.
    pub before_tokens: u32,
    /// Estimated token count of the payload that entered the context window.
    pub after_tokens: u32,
    /// Lowercase-hex full 32-byte blake3 of the original bytes (the reverse pointer).
    pub reverse_hash: String,
    /// True when the strategy degraded gracefully and returned the input verbatim.
    pub skipped: bool,
    /// Milliseconds since UNIX epoch at emit time.
    pub ts_ms: i64,
}

/// I-12: 4-arm discriminator for [`PlaybookEventPayload`].
///
/// - `Promoted`: a rule was advanced to ACTIVE (evidence gate passed, regression_count==0).
/// - `Demoted`: an ACTIVE rule lost ACTIVE status (stale FPR or regression detected).
/// - `ValidationRun`: the curator ran a check without completing promotion
///   (evidence_count < MIN_EVIDENCE_COUNT, or regression_count > 0).
/// - `Exported`: `export_verified` wrote a verified AGENTS.md / skills export block with
///   embedded outcome_row_ids so downstream consumers can grep-verify provenance.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub enum PlaybookEventKind {
    /// Rule advanced to ACTIVE after evidence gate + metric gates + zero regression.
    Promoted,
    /// ACTIVE rule demoted (stale or regressed).
    Demoted,
    /// Promotion-gate check ran without completing promotion.
    ValidationRun,
    /// Verified export written to AGENTS.md / skills block.
    Exported,
}

/// I-12: SSB payload for evidence-gated playbook curator events (ID-6 WEIGHTS-ARE-CONFIGS).
///
/// Pattern-2 single-variant; kind discriminator is [`PlaybookEventKind`].
/// Emitted by `PlaybookCurator::run_promotion_tick` (Promoted/Demoted/ValidationRun)
/// and `PlaybookCurator::export_verified` (Exported).
/// The `outcome_row_ids` field embeds the SSB seq IDs of the outcome rows used as
/// evidence — grep-verifiable and off-agent auditable (R8 gate-freeze).
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct PlaybookEventPayload {
    /// Stable identifier of the rule being promoted/demoted/validated/exported.
    pub rule_id: String,
    /// Which curator event this row represents.
    pub kind: PlaybookEventKind,
    /// SSB seq IDs of the outcome rows counted as evidence.
    /// Non-empty for Promoted and Exported; may be empty for ValidationRun/Demoted.
    pub outcome_row_ids: Vec<String>,
    /// Measured lift from on/off injection (None if not computed this tick).
    pub lift: Option<f32>,
    /// Number of C-28 mutation-suite regressions found during the gate check.
    /// 0 for Promoted/Exported; > 0 when ValidationRun blocks promotion.
    pub regression_count: u32,
    /// Number of SSB outcome links counted for this rule at gate time.
    pub evidence_count: u32,
    /// Milliseconds since UNIX epoch at emit time.
    pub ts_ms: i64,
}

// ---------------------------------------------------------------------------
// I-17: TriLayerVerdict payload structs
// ---------------------------------------------------------------------------

/// I-17: 4-arm discriminator for [`TriLayerVerdictPayload`].
///
/// - `HardVeto`: an L1 deterministic gate (compile / dep-gate / VibeGate / checker /
///   halting-bound) returned a veto. The branch score is a LITERAL `0.0` and neither
///   L2 nor L3 ran for it (`l2_call_count == 0`). ID-4: the veto is supreme — no
///   neural scorer may resurrect a vetoed branch.
/// - `ScoredSurvivor`: the branch cleared every L1 gate; the L2 calibrated scorer
///   (I-23 conformal facade) assigned it a `score` in `[0.0, 1.0]`.
/// - `DisagreementEscalated`: L2 scores among survivors disagreed past the
///   `[DESIGN]` variance threshold; the D-13 cross-family adversary was routed to
///   produce a concrete counter-example (not just a score).
/// - `FlakeQuarantined`: an oracle flipped across seeded reruns and was ledgered out
///   of voting until rehabilitated (blake3(oracle-id + seed-range) key).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub enum TriLayerVerdictKind {
    /// L1 deterministic veto fired — score is literal 0.0, L2/L3 never ran.
    HardVeto,
    /// Branch cleared L1; L2 calibrated scorer assigned a score.
    ScoredSurvivor,
    /// L2 variance among survivors exceeded threshold; D-13 adversary escalated.
    DisagreementEscalated,
    /// Flaky oracle ledgered out of voting (seed-variation flip detected).
    FlakeQuarantined,
}

/// I-17: which gate produced an L1 [`TriLayerVerdictKind::HardVeto`].
///
/// A typed enum (not a magic string) so the dashboard + contract test can group
/// vetoes by gate without parsing free text. `None` on the [`TriLayerVerdictPayload`]
/// `gate` field for non-veto verdicts (ScoredSurvivor / DisagreementEscalated /
/// FlakeQuarantined).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub enum GateId {
    /// Compile/test checker veto (D-11 CompileRustChecker or equivalent).
    L1Compile,
    /// F-9 write-time dependency gate veto (slopsquat / stale dep).
    L1DepGate,
    /// F-1 VibeGate veto (secret axis or code-vuln axis Block).
    L1VibeGate,
    /// D-11 generic constructive-checker veto (non-compile checker Fail).
    L1Checker,
    /// I-8 / I-17 halting-bound watchdog fired (deterministic check overran the bound).
    HaltingBound,
}

impl GateId {
    /// Stable wire string for SSB payloads + dashboard rendering.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::L1Compile => "l1_compile",
            Self::L1DepGate => "l1_dep_gate",
            Self::L1VibeGate => "l1_vibe_gate",
            Self::L1Checker => "l1_checker",
            Self::HaltingBound => "halting_bound",
        }
    }
}

/// I-17: SSB payload for one tri-layer per-branch verdict (ID-4 veto supremacy).
///
/// Pattern-2 single-variant; kind discriminator is [`TriLayerVerdictKind`].
/// Emitted by `verifier_stack::TriLayerStack::run_verdict`, exactly one row per
/// branch evaluated. The `l2_call_count` field is the off-agent observable proof of
/// veto supremacy: for a `HardVeto` row it MUST be `0` (the vetoed branch never
/// reached the L2 calibrated scorer). The auditor's contract test asserts this.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct TriLayerVerdictPayload {
    /// Stable identifier of the branch this verdict scores.
    pub branch_id: String,
    /// Which arm of the tri-layer stack produced this verdict.
    pub kind: TriLayerVerdictKind,
    /// The gate that vetoed (`Some` only for `HardVeto`; `None` otherwise).
    #[serde(default)]
    pub gate: Option<GateId>,
    /// Number of times the L2 calibrated scorer was invoked for this branch.
    /// `0` for `HardVeto` (the load-bearing observable proof L2 never saw a vetoed
    /// branch); `>= 1` for `ScoredSurvivor` / `DisagreementEscalated`.
    pub l2_call_count: u32,
    /// Final branch score in `[0.0, 1.0]`. Literal `0.0` for `HardVeto`
    /// (no soft-veto penalty weight — ID-4); the calibrated L2 score otherwise.
    pub score: f64,
    /// Milliseconds since UNIX epoch at emit time.
    pub ts_ms: i64,
}

/// I-18 — Kind discriminator for [`SsbActionType::RowChainSealed`].
///
/// - `ChainAdvanced`: the rolling blake3 chain_prev_hash advanced to the next
///   position after a sealed RRA. Emitted from `run_subcommand::emit_run_artifact`.
/// - `AibomEmitted`: the AI-BOM/CycloneDX-ML entry was sealed alongside the
///   in-toto Statement. Emitted from `run_subcommand::emit_attestation`.
/// - `ChainBroken`: a Cargo.lock mutation was detected mid-run at the DEPS
///   layer of the 5-layer StackHash. The run is halted with an error; the
///   off-agent beacon fires to signal the break (observable_effect R2 beacon).
///
/// NOTE: DISTINCT from `crate::verification_result::GateRejection::ChainBroken`
/// (a Rust error type in the verify path). Do NOT conflate.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RowChainSealedKind {
    /// Rolling chain_prev_hash advanced after a sealed RRA.
    ChainAdvanced,
    /// AI-BOM/CycloneDX-ML entry sealed alongside the in-toto Statement.
    AibomEmitted,
    /// Cargo.lock mutation detected mid-run at DEPS layer; run halted.
    ChainBroken,
}

impl RowChainSealedKind {
    /// Stable snake_case wire string.
    #[must_use]
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::ChainAdvanced => "chain_advanced",
            Self::AibomEmitted => "aibom_emitted",
            Self::ChainBroken => "chain_broken",
        }
    }
}

/// I-18 payload for [`SsbActionType::RowChainSealed`].
///
/// Emitted from `run_subcommand::emit_run_artifact` (ChainAdvanced arm) and
/// from `run_subcommand::emit_attestation` (AibomEmitted arm).
///
/// `kind` is the arm discriminator.
/// `seq` is the SSB row sequence number this chain seal anchors to.
/// `session_id` is the session that generated this event.
/// `ts_ms` is the wall-clock unix-ms at emit time.
/// `break_seq` is the seq of the broken row (`Some` only on ChainBroken arm).
/// TOTAL_VARIANTS 561 -> 562.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RowChainSealedPayload {
    /// Which arm fired.
    pub kind: RowChainSealedKind,
    /// SSB row sequence number this chain seal anchors to.
    pub seq: i64,
    /// SSB session_id that generated this event.
    pub session_id: String,
    /// Wall-clock unix-ms at emit time.
    pub ts_ms: i64,
    /// Seq of the broken row. `Some` only on the ChainBroken arm.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub break_seq: Option<i64>,
}

// === I-19 (SCHEMA_VERSION 254) -- TddPreflightBlocked substrate ===========

/// I-19 (SCHEMA_VERSION 254) -- discriminator for [`SsbActionType::TddPreflightBlocked`].
///
/// Pattern-2 single-variant 3-arm; each arm maps to its own sql_str.
/// Wire-dark unless `ELAI_TDD_PREFLIGHT_ENABLE=1` (R9 experiment).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TddPreflightBlockedKind {
    /// An Edit introduced a new pub symbol but the test-file set contains no
    /// test symbol whose name substring-matches the new pub symbol. The gate
    /// blocks the write to enforce the "no new pub fn without a test delta" rule.
    MissingTestDelta,
    /// Staged content fails `cargo check --tests` (Rust) or `pytest --collect-only`
    /// (Python) on the compile-only dry-run. The gate blocks the write because even
    /// a symbolically-matched test would fail to compile.
    SyntaxDryRunFailed,
    /// The diff introduces NO new pub symbol (pure refactor/rename/reformat).
    /// Gate is skipped; this arm is emitted as an advisory SSB row only.
    RefactorExempted,
}

impl TddPreflightBlockedKind {
    /// Snake-case wire label used as the sql_str for each arm.
    #[must_use]
    pub fn as_sql_str(&self) -> &'static str {
        match self {
            Self::MissingTestDelta   => "tdd_preflight_missing_test_delta",
            Self::SyntaxDryRunFailed => "tdd_preflight_syntax_dry_run_failed",
            Self::RefactorExempted   => "tdd_preflight_refactor_exempted",
        }
    }
}

/// I-19 (SCHEMA_VERSION 254) -- payload for [`SsbActionType::TddPreflightBlocked`].
///
/// Wire-dark default; emission gated by `ELAI_TDD_PREFLIGHT_ENABLE` env var.
///
/// Fields:
///   * `kind`           -- 3-arm fire-point discriminator.
///   * `file_path`      -- project-relative path of the file being edited.
///   * `pub_symbols`    -- comma-separated new pub symbols extracted by tree-sitter
///     (empty string for RefactorExempted arm).
///   * `fsm_state`      -- optional snake-case FSM state label at emission time.
///   * `dry_run_stderr` -- first 256 bytes of stderr from `cargo check --tests`
///     (SyntaxDryRunFailed arm only; empty string on other arms).
///   * `ts_ms`          -- wall-clock unix-ms at emission time.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TddPreflightBlockedPayload {
    /// Fire-point discriminator. See [`TddPreflightBlockedKind`].
    pub kind: TddPreflightBlockedKind,
    /// Project-relative path of the file being gated.
    pub file_path: String,
    /// Comma-separated new pub symbol names extracted by tree-sitter parse.
    /// Empty string for RefactorExempted arm.
    pub pub_symbols: String,
    /// Optional snake-case FSM state label at emission time.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub fsm_state: Option<String>,
    /// First 256 bytes of `cargo check --tests` stderr (SyntaxDryRunFailed arm only).
    pub dry_run_stderr: String,
    /// Wall-clock unix-ms at emission time.
    pub ts_ms: i64,
}

/// I-19 (SCHEMA_VERSION 254) -- env var name gating `TddPreflightBlockedPayload` emission.
/// Wire-dark default: gate is OFF until GD-4 FP rate bounded on >=3 real orchestrate runs.
/// Set to `"1"` / `"true"` / `"yes"` / `"on"` to activate.
pub const ELAI_TDD_PREFLIGHT_ENABLE_ENV: &str = "ELAI_TDD_PREFLIGHT_ENABLE";

// === I-20 (SCHEMA_VERSION 255) -- MutationBudgetedRun substrate =============

/// I-20 (SCHEMA_VERSION 255) -- discriminator for [`SsbActionType::MutationBudgetedRun`].
///
/// Pattern-2 single-variant 4-arm; each arm maps to its own sql_str. The
/// budgeted mutation scheduler (Track 15 KEYSTONE) emits one row per cone
/// scoring outcome.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MutationBudgetedRunKind {
    /// The Wilson-CI lower bound cleared the session's (TOML-measured)
    /// kill-rate threshold -- pass (early or full).
    CiCleared,
    /// The Wilson-CI upper bound already failed the threshold -- fail-early.
    CiFailed,
    /// A mutant survived; the row carries the structured SurvivorDirective
    /// (operator + line + surviving mutation) fed to the generator.
    SurvivorDirective,
    /// An unchanged (fn-content, bound-test-set) blake3 key returned from the
    /// EquivalentMutantCache with ZERO mutant runs.
    CacheHit,
}

impl MutationBudgetedRunKind {
    /// Snake-case wire label used as the sql_str for each arm.
    #[must_use]
    pub fn as_sql_str(&self) -> &'static str {
        match self {
            Self::CiCleared => "mutation_budgeted_run_ci_cleared",
            Self::CiFailed => "mutation_budgeted_run_ci_failed",
            Self::SurvivorDirective => "mutation_budgeted_run_survivor_directive",
            Self::CacheHit => "mutation_budgeted_run_cache_hit",
        }
    }
}

/// I-20 (SCHEMA_VERSION 255) -- payload for [`SsbActionType::MutationBudgetedRun`].
///
/// The 5 CI params (`n_sampled`, `n_total_cone`, `ci_lower`, `ci_upper`,
/// `time_ms`) are recomputable under `ELAI_SEED` (GD-2). For the
/// `SurvivorDirective` arm the `operator` / `survivor_file` / `survivor_line`
/// / `survivor_mutation` fields carry the structured counterexample; for the
/// `CacheHit` arm `n_sampled == 0` and the CI bounds are `0.0`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MutationBudgetedRunPayload {
    /// Fire-point discriminator. See [`MutationBudgetedRunKind`].
    pub kind: MutationBudgetedRunKind,
    /// Number of mutants actually sampled (0 on a pure cache hit).
    pub n_sampled: u32,
    /// Size of the full diff-cone-selected mutant set.
    pub n_total_cone: u32,
    /// Wilson-CI lower bound on the kill rate (0.0..=1.0).
    pub ci_lower: f64,
    /// Wilson-CI upper bound on the kill rate (0.0..=1.0).
    pub ci_upper: f64,
    /// The (TOML-measured, ID-5) kill-rate threshold the run was scored against.
    pub threshold: f64,
    /// Wall-clock duration of the budgeted run in milliseconds.
    pub time_ms: u64,
    /// SurvivorDirective arm: the surviving operator (empty on other arms).
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub operator: String,
    /// SurvivorDirective arm: project-relative source file (empty on other arms).
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub survivor_file: String,
    /// SurvivorDirective arm: 1-based source line (0 on other arms).
    #[serde(default, skip_serializing_if = "is_zero_u32")]
    pub survivor_line: u32,
    /// SurvivorDirective arm: human-readable surviving-mutation description
    /// (empty on other arms).
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub survivor_mutation: String,
    /// Wall-clock unix-ms at emission time.
    pub ts_ms: i64,
}

/// serde skip helper for the optional `survivor_line` field.
fn is_zero_u32(v: &u32) -> bool {
    *v == 0
}

/// I-20 (SCHEMA_VERSION 255) -- env var name gating `MutationBudgetedRun`
/// emission. Default-ON kill-switch posture: the budget scheduler is the
/// mutation hot-path optimization, so it runs by default; set to `"1"` /
/// `"true"` to DISABLE emission (the gate verdict is unaffected either way).
pub const ELAI_MUTATION_BUDGET_DISABLE_ENV: &str = "ELAI_MUTATION_BUDGET_DISABLE";

// === I-21 (SCHEMA_VERSION 256) -- SprintContractEvent substrate =============

/// I-21 (SCHEMA_VERSION 256) -- discriminator for [`SsbActionType::SprintContractEvent`].
///
/// Pattern-2 single-variant 5-arm; each arm maps to its own sql_str. The
/// sprint-contract + ralph-loop machinery emits one row per lifecycle event:
/// the isolated evaluator's receive/verdict, each bounded reinjection, and the
/// 3rd-failure cross-family adversary escalation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SprintContractEventKind {
    /// The isolated evaluator received the generator's sealed Sprint Contract
    /// (zero shared generator context).
    Proposed,
    /// The evaluator approved the contract (acceptance criteria are
    /// deterministically checkable; scope is bounded).
    Approved,
    /// The evaluator rejected the contract; the payload's `objection` field
    /// carries the concrete, structured reason (the vague-criterion path).
    Rejected,
    /// A TriLayerStack failure reinjected the failing trace to the generator
    /// for one bounded retry (loop cap = 3 total iterations).
    RalphReinjected,
    /// The 3rd consecutive failure escalated to the D-13 cross-family adversary,
    /// which produced a concrete counterexample rather than a bare score.
    AdversaryEscalated,
}

impl SprintContractEventKind {
    /// Snake-case wire label used as the sql_str for each arm.
    #[must_use]
    pub fn as_sql_str(&self) -> &'static str {
        match self {
            Self::Proposed => "sprint_contract_proposed",
            Self::Approved => "sprint_contract_approved",
            Self::Rejected => "sprint_contract_rejected",
            Self::RalphReinjected => "sprint_contract_ralph_reinjected",
            Self::AdversaryEscalated => "sprint_contract_adversary_escalated",
        }
    }
}

/// I-21 (SCHEMA_VERSION 256) -- payload for [`SsbActionType::SprintContractEvent`].
///
/// One row per sprint-contract lifecycle event. The `iteration` field carries
/// the ralph-loop counter (0 at Proposed/Approved/Rejected, 1..=3 on the
/// reinjection/escalation arms); `objection` carries the evaluator's concrete
/// structured rejection reason (empty on non-Rejected arms); `counterexample`
/// carries the D-13 adversary counter text (empty unless AdversaryEscalated).
/// All free-text fields are bounded by the caller before construction.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SprintContractEventPayload {
    /// Fire-point discriminator. See [`SprintContractEventKind`].
    pub kind: SprintContractEventKind,
    /// Stable per-contract identifier (blake3-derived by the producer so rows
    /// recompute under `ELAI_SEED`, GD-2).
    pub contract_id: String,
    /// The contract goal (bounded plaintext).
    pub goal: String,
    /// Ralph-loop iteration counter: 0 on Proposed/Approved/Rejected,
    /// 1..=`MAX_RALPH_REINJECTIONS` on RalphReinjected/AdversaryEscalated.
    pub iteration: u32,
    /// Rejected arm: the concrete structured objection (empty on other arms).
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub objection: String,
    /// AdversaryEscalated arm: the D-13 counterexample text (empty on others).
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub counterexample: String,
    /// Wall-clock unix-ms at emission time.
    pub ts_ms: i64,
}

/// I-21 (SCHEMA_VERSION 256) -- env var name gating `SprintContractEvent`
/// emission. Default-OFF (R9 experiment): set to a truthy literal (`"1"` /
/// `"true"` / `"yes"` / `"on"`) to enable the sprint-contract dispatch path.
/// Promotion plan: `I-21-promotion`. See `wire-dark-allowlist.toml`.
pub const ELAI_SPRINT_CONTRACT_ENABLE_ENV: &str = "ELAI_SPRINT_CONTRACT_ENABLE";

// === I-4 (SCHEMA_VERSION 261) -- SearchNodeEvent substrate ==================

/// I-4 (SCHEMA_VERSION 261) -- discriminator for [`SsbActionType::SearchNodeEvent`].
///
/// Pattern-2 single-variant 4-arm; each arm maps to its own sql_str. The
/// evidence-ranked search ladder emits one row per branch lifecycle transition
/// so the SSB records WHY each branch won or lost (the dashboard branch-board,
/// H-15, is the intended consumer).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SearchNodeEventKind {
    /// A candidate branch was generated, its pristine per-branch context built
    /// (I-5), and run through the I-17 tri-layer stack. The row carries the
    /// branch's tri-layer verdict + score.
    Expanded,
    /// A branch was DROPPED from selection: the tri-layer stack hard-vetoed it
    /// (`VerdictRow::is_vetoed`) or its oracle was flake-quarantined
    /// (`FlakeQuarantineLedger::is_quarantined`). Terminal (ID-4) -- a pruned
    /// branch is never Promoted.
    Pruned,
    /// The best-of-N winner. Carries verifier-derived evidence (tri-layer score
    /// + mutation kill-rate + progress score); a branch promoted on neural score
    /// alone is the ID-4 stub_ban the broken `--llm-judge-only` arm FAILS on.
    Promoted,
    /// Every candidate was vetoed/quarantined; the ladder produced no survivor
    /// and fell back to the linear-retry path (the honest no-winner beacon).
    BudgetCollapsed,
}

impl SearchNodeEventKind {
    /// Snake-case wire label used as the sql_str for each arm.
    #[must_use]
    pub fn as_sql_str(&self) -> &'static str {
        match self {
            Self::Expanded => "search_node_expanded",
            Self::Pruned => "search_node_pruned",
            Self::Promoted => "search_node_promoted",
            Self::BudgetCollapsed => "search_node_budget_collapsed",
        }
    }
}

/// I-4 (SCHEMA_VERSION 261) -- payload for [`SsbActionType::SearchNodeEvent`].
///
/// One row per search-ladder branch lifecycle transition. The verifier-receipt
/// fields are the load-bearing evidence: a `Promoted` row with a neutral
/// `tri_layer_score` and a zero `mutation_kill_rate` is the ID-4 stub_ban the
/// `--llm-judge-only` broken arm produces (no verifier-derived selection). All
/// fields recompute under `ELAI_SEED` (GD-2): `mutation_kill_rate` is the
/// Wilson-CI point estimate, `progress_score` the rank-3 tie-breaker.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SearchNodeEventPayload {
    /// Fire-point discriminator. See [`SearchNodeEventKind`].
    pub kind: SearchNodeEventKind,
    /// The branch this row scores (the I-17 `Branch::branch_id` / `VerdictRow`
    /// key, so the row joins to its `tri_layer_verdict` row).
    pub branch_id: String,
    /// The wire tag of the branch's tri-layer verdict (e.g. `hard_veto`,
    /// `scored_survivor`); the rank-1 selection signal. Empty on
    /// `BudgetCollapsed` (no survivor to rank).
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub tri_layer_verdict: String,
    /// The branch's tri-layer score in `[0.0, 1.0]` (rank-1). Literal `0.0` for
    /// a vetoed (Pruned) branch.
    pub tri_layer_score: f64,
    /// The branch's I-20 mutation kill-rate (Wilson-CI point estimate, rank-2).
    /// `0.0` when no mutation receipt was supplied for the branch.
    pub mutation_kill_rate: f64,
    /// The branch's progress score (rank-3 tie-breaker) in `[0.0, 1.0]`.
    pub progress_score: f64,
    /// Wall-clock unix-ms at emission time.
    pub ts_ms: i64,
}

/// I-4 (SCHEMA_VERSION 261) -- env var name gating `SearchNodeEvent` emission
/// AND the multi-branch ladder dispatch. Default-OFF (R9 experiment): the
/// multi-branch ladder consumes more compute + changes search to best-of-N, so
/// it ships behind a GD-4 measured-first-win gate. Set to a truthy literal
/// (`"1"` / `"true"` / `"yes"` / `"on"`) to enable. Promotion plan:
/// `I-4-promotion`. See `wire-dark-allowlist.toml`.
pub const ELAI_SEARCH_LADDER_ENABLE_ENV: &str = "ELAI_SEARCH_LADDER_ENABLE";

// === I-7 (SCHEMA_VERSION 262) -- EscalationCascade substrate =================

/// I-7 (SCHEMA_VERSION 262) -- discriminator for [`SsbActionType::EscalationCascade`].
///
/// Pattern-2 single-variant 4-arm; each arm maps to its own sql_str. The
/// local-first escalation waterfall emits one row per lifecycle transition.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EscalationCascadeKind {
    /// At least one local branch survived the I-17 tri-layer check; the task
    /// solved on the local lane and the cloud arm NEVER ran (E-39 zero-cloud-spend).
    LocalWin,
    /// Every local branch failed; the failure set was condensed into the typed
    /// I-11 BeliefState (<=2K, NO raw history -- ID-3 / stub_ban). The row carries
    /// the condensed payload byte size.
    Condensed,
    /// The condensate was routed SYNCHRONOUSLY to the H-7 cloud seam
    /// (`CloudModeRouter::route`). The billed cloud architect inference that
    /// consumes the condensate is D-026 PENDING (never fired from the buildable
    /// path). The row carries the condensed payload byte size.
    Escalated,
    /// Journal-completeness ONLY for the OFFLINE H-5 batch lane; the LIVE waterfall
    /// NEVER emits this arm (IC-1: the batch lane is structurally unreachable from
    /// `escalation_waterfall.rs` -- it has no batch-client import). Present so the
    /// offline scheduler has a wire tag without a second action_type.
    BatchRouted,
}

impl EscalationCascadeKind {
    /// Snake-case wire label used as the sql_str for each arm.
    #[must_use]
    pub fn as_sql_str(&self) -> &'static str {
        match self {
            Self::LocalWin    => "escalation_cascade_local_win",
            Self::Condensed   => "escalation_cascade_condensed",
            Self::Escalated   => "escalation_cascade_escalated",
            Self::BatchRouted => "escalation_cascade_batch_routed",
        }
    }
}

/// I-7 (SCHEMA_VERSION 262) -- payload for [`SsbActionType::EscalationCascade`].
///
/// Wire-dark default; emission gated by `ELAI_ESCALATION_WATERFALL_ENABLE`.
///
/// Fields:
///   * `kind`            -- 4-arm lifecycle discriminator.
///   * `condensed_bytes` -- byte length of the condensed I-11 BeliefState wire
///     payload sent on escalation (`0` for the `LocalWin` arm; `<=` the module's
///     `CONDENSED_PAYLOAD_MAX_BYTES` ceiling on `Condensed`/`Escalated`).
///   * `ts_ms`           -- wall-clock unix-ms at emission time.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EscalationCascadePayload {
    /// Fire-point discriminator. See [`EscalationCascadeKind`].
    pub kind: EscalationCascadeKind,
    /// Byte length of the condensed BeliefState wire payload (`0` on `LocalWin`).
    pub condensed_bytes: u32,
    /// Wall-clock unix-ms at emission time.
    pub ts_ms: i64,
}

/// I-7 (SCHEMA_VERSION 262) -- env var name gating `EscalationCascade` emission AND
/// the `maybe_run_escalation_waterfall` drive_task hook. Default-OFF (R9
/// experiment): the local-first waterfall changes the local->cloud transition
/// path, so it ships behind a GD-4 measured-first-win gate. Set to a truthy literal
/// (`"1"` / `"true"` / `"yes"` / `"on"`) to enable. Promotion plan: `I-7-promotion`.
/// See `wire-dark-allowlist.toml`.
pub const ELAI_ESCALATION_WATERFALL_ENABLE_ENV: &str = "ELAI_ESCALATION_WATERFALL_ENABLE";

// === I-22 (SCHEMA_VERSION 264) -- ProgressRewardScored substrate =============

/// I-22 (SCHEMA_VERSION 264) -- discriminator for [`SsbActionType::ProgressRewardScored`].
///
/// Pattern-2 single-variant 3-arm; each arm maps to its own sql_str. Emitted by
/// `crate::search::score::DeltaProgressScorer` from the
/// `task_runner::dispatch_progress_reward` post-task seam.
///
/// - `Delta`: a scored step; carries the shaped reward phi(h_t) - phi(h_{t-1}) and
///   the full phi-component breakdown (test_pass, compile_err_inv, req_covered) so
///   the review is reproducible under `ELAI_SEED` (GD-2 / ID-4).
/// - `FirstErrorTruncated`: a branch was truncated at the first HardVeto (step k-1);
///   the H-49 fork node id at step k-1 is visible in the `fork_node_id` field.
/// - `DecayApplied`: D consecutive zero-delta steps triggered the no-progress decay
///   multiplier; the `decay_factor` field carries the applied multiplier.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ProgressRewardScoredKind {
    /// Shaped reward phi(h_t) - phi(h_{t-1}) scored for one branch step.
    Delta,
    /// Branch truncated at first HardVeto (step k-1); H-49 fork point recorded.
    FirstErrorTruncated,
    /// No-progress decay multiplier applied after D consecutive zero-delta steps.
    DecayApplied,
}

impl ProgressRewardScoredKind {
    /// Snake-case wire label used as the sql_str for each arm.
    #[must_use]
    pub fn as_sql_str(self) -> &'static str {
        match self {
            Self::Delta              => "progress_reward_delta",
            Self::FirstErrorTruncated => "progress_reward_first_error_truncated",
            Self::DecayApplied      => "progress_reward_decay_applied",
        }
    }
}

/// I-22 (SCHEMA_VERSION 264) -- payload for [`SsbActionType::ProgressRewardScored`].
///
/// Wire-dark default; emission gated by `ELAI_PROGRESS_REWARD_ENABLE` env var.
///
/// Fields (all arms):
///   * `kind`           -- 3-arm fire-point discriminator.
///   * `branch_id`      -- stable identifier of the branch being scored.
///   * `step`           -- 0-indexed step index within the branch trajectory.
///   * `ts_ms`          -- wall-clock unix-ms at emission time.
///
/// Fields (Delta arm only):
///   * `phi_t`          -- current phi(h_t) value (weighted sum of deterministic observables).
///   * `phi_t_prev`     -- previous phi(h_{t-1}) value (0.0 at step 0).
///   * `shaped_reward`  -- phi_t - phi_t_prev (the actual shaped reward fed to the PUCT value fn).
///   * `test_pass`      -- count of passing tests at step t (deterministic, from SSB TestOutcome rows).
///   * `compile_err_inv` -- inverted compile-error count (higher = fewer errors). Deterministic.
///   * `req_covered`    -- count of EARS requirements that are Satisfied at step t (from i-37).
///
/// Fields (FirstErrorTruncated arm only):
///   * `fork_node_id`   -- H-49 CAS fork node id at step k-1 (empty string on other arms).
///   * `truncated_at_step` -- the step k where the HardVeto fired (0 on other arms).
///
/// Fields (DecayApplied arm only):
///   * `decay_factor`   -- the multiplier applied (< 1.0); 1.0 on other arms.
///   * `no_progress_window` -- the D parameter from metrics/progress_reward_weights.toml (0 on other arms).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ProgressRewardScoredPayload {
    /// Fire-point discriminator. See [`ProgressRewardScoredKind`].
    pub kind: ProgressRewardScoredKind,
    /// Stable identifier of the branch this scored step belongs to.
    pub branch_id: String,
    /// 0-indexed step index within the branch trajectory.
    pub step: u32,
    // --- Delta arm fields ---
    /// Current phi(h_t). Deterministic observable sum; 0.0 on non-Delta arms.
    #[serde(default, skip_serializing_if = "is_zero_f64")]
    pub phi_t: f64,
    /// Previous phi(h_{t-1}). 0.0 at step 0 and on non-Delta arms.
    #[serde(default, skip_serializing_if = "is_zero_f64")]
    pub phi_t_prev: f64,
    /// Shaped reward = phi_t - phi_t_prev. 0.0 on non-Delta arms.
    #[serde(default, skip_serializing_if = "is_zero_f64")]
    pub shaped_reward: f64,
    /// Count of passing tests at step t (deterministic, ID-4). 0 on non-Delta arms.
    #[serde(default, skip_serializing_if = "is_zero_u32")]
    pub test_pass: u32,
    /// Inverted compile-error count at step t (higher = cleaner). 0 on non-Delta arms.
    #[serde(default, skip_serializing_if = "is_zero_u32")]
    pub compile_err_inv: u32,
    /// Count of Satisfied EARS requirements at step t. 0 on non-Delta arms.
    #[serde(default, skip_serializing_if = "is_zero_u32")]
    pub req_covered: u32,
    // --- FirstErrorTruncated arm fields ---
    /// H-49 CAS fork node id recorded at step k-1. Empty on non-truncation arms.
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub fork_node_id: String,
    /// The step k where the HardVeto fired (branch truncated at k-1). 0 on other arms.
    #[serde(default, skip_serializing_if = "is_zero_u32")]
    pub truncated_at_step: u32,
    // --- DecayApplied arm fields ---
    /// The decay multiplier applied (< 1.0). 1.0 on non-decay arms.
    #[serde(default, skip_serializing_if = "is_one_f64")]
    pub decay_factor: f64,
    /// The no-progress window D (from metrics/progress_reward_weights.toml). 0 on other arms.
    #[serde(default, skip_serializing_if = "is_zero_u32")]
    pub no_progress_window: u32,
    /// Wall-clock unix-ms at emission time.
    pub ts_ms: i64,
}

/// serde skip helper: true when the f64 is exactly 0.0.
fn is_zero_f64(v: &f64) -> bool {
    *v == 0.0
}

/// serde skip helper: true when the f64 is exactly 1.0.
fn is_one_f64(v: &f64) -> bool {
    *v == 1.0
}

/// I-22 (SCHEMA_VERSION 264) -- env var name gating `ProgressRewardScored` emission
/// and the `dispatch_progress_reward` drive_task hook. Default-OFF (R9 experiment):
/// the progress-shaped reward scorer changes how the search ladder values branches,
/// so it ships behind a GD-4 measured-first-win gate. Set to a truthy literal
/// (`"1"` / `"true"` / `"yes"` / `"on"`) to enable. Promotion plan: `I-22-promotion`
/// (FORWARD). See `wire-dark-allowlist.toml`.
pub const ELAI_PROGRESS_REWARD_ENABLE_ENV: &str = "ELAI_PROGRESS_REWARD_ENABLE";

// === I-37 (SCHEMA_VERSION 257) -- RequirementVerified substrate =============

/// I-37 (SCHEMA_VERSION 257) -- discriminator for [`SsbActionType::RequirementVerified`].
///
/// Pattern-2 single-variant 3-arm; each arm maps to its own sql_str. One row is
/// emitted per EARS requirement at the PLANNING -> TESTING_FIRST audit so the
/// G-5 cockpit can render a per-requirement verified-status chip. The emit is
/// additive (independent of the blocking gate flag) so an `Unverified`
/// requirement is always SHOWN, never silently dropped (R1).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RequirementVerifiedKind {
    /// Mutation coverage cleared the (I-20-measured) floor AND every bound
    /// metamorphic relation held AND the requirement has >=1 bound test.
    Satisfied,
    /// The requirement has NO bound tests -- it is unverifiable and is shown as
    /// such in the cockpit (R1: missing evidence is a hard, visible state, never
    /// a silent drop).
    Unverified,
    /// A previously-bound test was deleted (the re-audit status flip), or a
    /// bound MR no longer holds, or coverage fell below the floor.
    Regressed,
}

impl RequirementVerifiedKind {
    /// Snake-case wire label used as the sql_str for each arm.
    #[must_use]
    pub fn as_sql_str(&self) -> &'static str {
        match self {
            Self::Satisfied => "requirement_verified_satisfied",
            Self::Unverified => "requirement_verified_unverified",
            Self::Regressed => "requirement_verified_regressed",
        }
    }
}

/// I-37 (SCHEMA_VERSION 257) -- payload for [`SsbActionType::RequirementVerified`].
///
/// One row per EARS requirement per audit. `content_hash` is the blake3 of the
/// canonical requirement text (the idempotent join key for the cockpit chip);
/// `mutation_coverage` is the I-20 sampled score (`None` until a scoring audit
/// runs; recomputable under `ELAI_SEED`, GD-2 -- never fabricated).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RequirementVerifiedPayload {
    /// Fire-point discriminator. See [`RequirementVerifiedKind`].
    pub kind: RequirementVerifiedKind,
    /// blake3 hex of the canonical requirement text (drift detector + chip key).
    pub content_hash: String,
    /// The EARS stereotype label (snake_case: ubiquitous / event_driven / ...).
    pub pattern: String,
    /// Number of bound tests on this requirement.
    pub bound_test_count: u32,
    /// I-20 sampled mutation score on the bound tests (`None` until scored).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub mutation_coverage: Option<f64>,
    /// Number of bound metamorphic relations.
    pub mr_count: u32,
    /// Wall-clock unix-ms at emission time.
    pub ts_ms: i64,
}

/// I-37 (SCHEMA_VERSION 257) -- env var name gating the blocking `RequirementsGate`.
/// Default-OFF (R9 experiment): with the flag absent the gate degrades to pass
/// and changes no production gate behaviour (only the additive
/// `RequirementVerified` SSB emit runs). Set to a truthy literal (`"1"` /
/// `"true"` / `"yes"` / `"on"`) to enable blocking. Promotion plan:
/// `I-37-promotion`. See `wire-dark-allowlist.toml`.
pub const ELAI_REQUIREMENTS_GATE_ENABLE_ENV: &str = "ELAI_REQUIREMENTS_GATE_ENABLE";

// === I-44 (SCHEMA_VERSION 258) -- RetentionPurged substrate ==================

/// I-44 (SCHEMA_VERSION 258) -- discriminator for [`SsbActionType::RetentionPurged`].
///
/// Pattern-2 single-variant 3-arm; each arm maps to its own sql_str. Emitted by
/// `crate::retention::purge_after_verify` from the `elai run` post-attestation
/// seal seam once a zero-retention purge runs.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RetentionPurgedKind {
    /// The active session's raw managed-transcript files were deleted after the
    /// sealed verdict; the payload's `purged_classes` names the file classes.
    RawInputsPurged,
    /// A blake3 over the redacted retained evidence set -- the survivable digest
    /// the verdict still verifies against (carried in `retained_blake3`).
    EvidenceRetained,
    /// Post-purge beacon: the raw bytes are gone and the redacted evidence
    /// digest survives + verifies (the auditable proof of the purge).
    PurgeVerified,
}

impl RetentionPurgedKind {
    /// Snake-case wire label used as the sql_str for each arm.
    #[must_use]
    pub fn as_sql_str(&self) -> &'static str {
        match self {
            Self::RawInputsPurged => "retention_raw_inputs_purged",
            Self::EvidenceRetained => "retention_evidence_retained",
            Self::PurgeVerified => "retention_purge_verified",
        }
    }
}

/// I-44 (SCHEMA_VERSION 258) -- payload for [`SsbActionType::RetentionPurged`].
///
/// Records WHAT was purged + WHAT was retained so the purge itself is auditable:
/// you can prove the raw bytes are gone AND the verdict still verifies. Secret-
/// free by construction (only file-class extensions + a hash, never raw bytes).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RetentionPurgedPayload {
    /// Fire-point discriminator. See [`RetentionPurgedKind`].
    pub kind: RetentionPurgedKind,
    /// Purged file-class extensions (`jsonl` / `json`, ...). Empty on the
    /// EvidenceRetained / PurgeVerified arms.
    pub purged_classes: Vec<String>,
    /// blake3 hex over the redacted retained evidence set (the survivable
    /// digest). Empty only when nothing was retained.
    pub retained_blake3: String,
    /// The session whose raw inputs were purged.
    pub session_id: String,
    /// Wall-clock unix-ms at emission time (injected; never `SystemTime::now`).
    pub ts_ms: i64,
}

/// I-44 (SCHEMA_VERSION 258) -- env var name gating the destructive zero-
/// retention purge. Default-OFF (R9 experiment, GD-4): with the flag absent the
/// purge performs ZERO filesystem mutation even when `--zero-retention` is
/// passed (defence in depth on an irreversible op). Truthy literals (`"1"` /
/// `"true"` / `"yes"` / `"on"`) enable. Promotion plan: `I-44-promotion`. See
/// `wire-dark-allowlist.toml`. Mirrored by `crate::retention::ZERO_RETENTION_ENV`.
pub const ELAI_ZERO_RETENTION_ENABLE_ENV: &str = "ELAI_ZERO_RETENTION_ENABLE";

// === I-52 (SCHEMA_VERSION 259) -- CheckpointToolEvent substrate =============

/// I-52 (SCHEMA_VERSION 259) -- discriminator for [`SsbActionType::CheckpointToolEvent`].
///
/// Pattern-2 single-variant 4-arm; each arm maps to its own sql_str. Emitted by
/// the agent-callable `checkpoint_save/rewind/diff` tool arms over the H-49
/// time-travel CAS substrate.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CheckpointToolEventKind {
    /// The agent created a labelled checkpoint: a tagged manifest of the named
    /// files captured into the H-49 CAS (`timetravel::capture_file`).
    Saved,
    /// The agent rewound the working tree to a saved label; the restore routed
    /// through `timetravel::materialize` (the only writer; no raw-write bypass).
    Rewound,
    /// The agent diffed the working tree against a saved label (read-only).
    Diffed,
    /// The rewind was REFUSED because the working tree diverged from the
    /// checkpoint since save (RV-1 safety; no silent overwrite over diverged
    /// files). The payload's `diverged_paths` names the diverged set.
    RefusedDiverged,
}

impl CheckpointToolEventKind {
    /// Snake-case wire label used as the sql_str for each arm.
    #[must_use]
    pub fn as_sql_str(&self) -> &'static str {
        match self {
            Self::Saved => "checkpoint_tool_saved",
            Self::Rewound => "checkpoint_tool_rewound",
            Self::Diffed => "checkpoint_tool_diffed",
            Self::RefusedDiverged => "checkpoint_tool_refused_diverged",
        }
    }
}

/// I-52 (SCHEMA_VERSION 259) -- payload for [`SsbActionType::CheckpointToolEvent`].
///
/// One row per agent checkpoint-tool call. The `RefusedDiverged` arm is the
/// load-bearing safety observable (R1: the divergence-refusal must LAND so the
/// auditor can prove the rewind was refused, not silently applied).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CheckpointToolEventPayload {
    /// Fire-point discriminator. See [`CheckpointToolEventKind`].
    pub kind: CheckpointToolEventKind,
    /// The session this checkpoint is scoped to (derived from run context, never
    /// agent-supplied -- the cross-session rewind guard).
    pub session_id: String,
    /// User-supplied checkpoint label (the `tag` the save/rewind/diff resolves).
    pub label: String,
    /// Project-relative paths covered by this checkpoint (the manifest file set).
    pub paths: Vec<String>,
    /// On `RefusedDiverged`: the project-relative paths that diverged since save.
    /// Empty on the Saved / Rewound / Diffed arms.
    pub diverged_paths: Vec<String>,
    /// Wall-clock unix-ms at emission time (injected via the clock; never
    /// `SystemTime::now`).
    pub ts_ms: i64,
}

// === I-54 (SCHEMA_VERSION 260) -- StructuralEditApplied substrate ============

/// I-54 (SCHEMA_VERSION 260) -- discriminator for
/// [`SsbActionType::StructuralEditApplied`].
///
/// Pattern-2 single-variant 5-arm; each arm maps to its own sql_str. Emitted by
/// the agent-callable `structural_search/_preview/_apply` tool arms over the
/// shipped tree-sitter grammars (one version source via
/// `elai_knowledge::TreeSitterAstEvaluator`; ast-grep dep BANNED, SLC-3).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum StructuralEditAppliedKind {
    /// The search arm ran a tree-sitter query and returned a match set.
    Matched,
    /// The preview arm resolved a query + rewrite template into the would-be
    /// diff (the row carries the blake3 diff hash). ZERO bytes written.
    Previewed,
    /// The previewed rewrite set was applied through the SHIPPED PreToolUse edit
    /// gates + `dispatch_edit`.
    Applied,
    /// A matched node's bytes changed between preview and apply (the AST moved);
    /// the apply was REFUSED. The load-bearing safety observable (R1; no silent
    /// corruption).
    RefusedStale,
    /// A broken tree-sitter query / rewrite template was caught at preview; ZERO
    /// bytes (no regex degrade -- stub_ban).
    ParseFailed,
}

impl StructuralEditAppliedKind {
    /// Snake-case wire label used as the sql_str for each arm.
    #[must_use]
    pub fn as_sql_str(&self) -> &'static str {
        match self {
            Self::Matched => "structural_edit_matched",
            Self::Previewed => "structural_edit_previewed",
            Self::Applied => "structural_edit_applied",
            Self::RefusedStale => "structural_edit_refused_stale",
            Self::ParseFailed => "structural_edit_parse_failed",
        }
    }
}

/// I-54 (SCHEMA_VERSION 260) -- payload for
/// [`SsbActionType::StructuralEditApplied`].
///
/// One row per agent structural-edit tool call. The `RefusedStale` arm is the
/// load-bearing safety observable (R1: a stale-match refusal must LAND so the
/// auditor can prove the apply was refused, not silently corrupting the file).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct StructuralEditAppliedPayload {
    /// Fire-point discriminator. See [`StructuralEditAppliedKind`].
    pub kind: StructuralEditAppliedKind,
    /// Source language the query ran against (rust / python / typescript).
    pub language: String,
    /// Number of nodes the tree-sitter query matched (Matched/Previewed/Applied).
    pub match_count: u32,
    /// blake3 hex of the would-be diff (Previewed/Applied); empty otherwise.
    pub diff_hash: String,
    /// Project-relative file paths the edit covered (Applied), or the stale path
    /// (RefusedStale). Empty on Matched/Previewed/ParseFailed.
    pub paths: Vec<String>,
    /// Wall-clock unix-ms at emission time (injected via the clock; never
    /// `SystemTime::now`).
    pub ts_ms: i64,
}

/// G-10: drift signal kind.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum DriftSignalKind {
    FingerprintChanged,
    CanaryRegressed,
    MalformedOutputSpike,
}

impl DriftSignalKind {
    pub fn label(&self) -> &'static str {
        match self {
            Self::FingerprintChanged => "fingerprint_changed",
            Self::CanaryRegressed => "canary_regressed",
            Self::MalformedOutputSpike => "malformed_output_spike",
        }
    }
}

/// G-10: exogenous behavioral regression suspected.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct ModelDriftSuspectedPayload {
    pub identity_key: String,
    pub kind: DriftSignalKind,
    pub signals: Vec<String>,
    pub per_family_delta: Vec<(String, f64)>,
    pub affected_template_ids: Vec<String>,
    pub confirmed_by_fingerprint: bool,
    pub baseline_captured_at: u64,
}

/// G-9: write-time secret/PII redaction advisory.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct SecretRedactedPayload {
    pub session_id: String,
    pub field: String,
    pub tokens: Vec<String>,
    pub match_count: u32,
    pub kind_counts: Vec<(String, u32)>,
}

/// G-7b: per-change conformal confidence decision.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct ClaimConfidenceScoredPayload {
    pub claim_id: String,
    pub claim_score: f64,
    pub logprob_component: Option<f64>,
    pub self_consistency_component: f64,
    pub exec_agreement_component: f64,
    pub lane: String,
    pub tau: f64,
    pub conformal_verdict: String,
    pub abstained: bool,
}

/// G-11: steering-vector refresh triggered.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct SteeringVectorRefreshTriggeredPayload {
    pub pipeline_mode: String,
    pub rules_db_path: String,
    pub reason: String,
}

/// H-10 canvas: canvas op kind.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum CanvasOpKind {
    NodeCreated,
    EdgeConnected,
    /// Node removed from the canvas graph.
    NodeDeleted,
    /// Edge removed from the canvas graph.
    EdgeDeleted,
    LabelEdited,
    AnnotationAdded,
    /// A visual group (cluster) was created around a set of nodes.
    GroupCreated,
    NodeForked,
    LayoutChanged,
    ScreenshotCaptured,
}

impl CanvasOpKind {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::NodeCreated => "node_created",
            Self::EdgeConnected => "edge_connected",
            Self::NodeDeleted => "node_deleted",
            Self::EdgeDeleted => "edge_deleted",
            Self::LabelEdited => "label_edited",
            Self::AnnotationAdded => "annotation_added",
            Self::GroupCreated => "group_created",
            Self::NodeForked => "node_forked",
            Self::LayoutChanged => "layout_changed",
            Self::ScreenshotCaptured => "screenshot_captured",
        }
    }
}

/// H-10 canvas: canvas op applied at dispatcher.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct CanvasOpPayload {
    pub kind: CanvasOpKind,
    pub node_id: String,
    pub op_data: serde_json::Value,
    pub mode: String,
    pub task_id: String,
    pub session_id: String,
}

/// H-10 canvas: canvas view event kind.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum CanvasViewKind {
    ViewSpawned,
    UiAction,
    StateCheckpoint,
}

impl CanvasViewKind {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::ViewSpawned => "view_spawned",
            Self::UiAction => "ui_action",
            Self::StateCheckpoint => "state_checkpoint",
        }
    }
}

/// H-10 canvas: canvas view event.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct CanvasViewPayload {
    pub kind: CanvasViewKind,
    pub view_type: String,
    pub view_id: String,
    pub seed_op_ids: Vec<String>,
    pub node_id: String,
    pub taint_flagged: bool,
    pub task_id: String,
    pub session_id: String,
}

/// H-10 canvas: canvas activity event kind.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum CanvasActivityKind {
    Reading,
    Editing,
    /// Canvas agent is validating a node or edge (running checks, linting, test assertions).
    Validating,
    /// Canvas agent is performing web/doc research for a node.
    Researching,
}

impl CanvasActivityKind {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Reading => "reading",
            Self::Editing => "editing",
            Self::Validating => "validating",
            Self::Researching => "researching",
        }
    }
}

/// H-10 canvas: canvas activity (reading/editing) event.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct CanvasActivityPayload {
    pub kind: CanvasActivityKind,
    pub target_node_id: String,
    pub file_path: String,
    pub symbol_name: Option<String>,
    pub mode: String,
    pub label: Option<String>,
    pub task_id: String,
}

/// H-1: Anthropic batch API lifecycle kind.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum BatchLifecycleKind {
    Submitted,
    Completed,
    /// Batch expired before all requests resolved (Anthropic API expires batches after 24h).
    Expired,
}

impl BatchLifecycleKind {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Submitted => "submitted",
            Self::Completed => "completed",
            Self::Expired => "expired",
        }
    }

    /// Alias for [`as_str`]; kept for call-site compat.
    #[inline]
    pub fn label(&self) -> &'static str {
        self.as_str()
    }
}

/// H-1: Anthropic batch API lifecycle row.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct BatchLifecyclePayload {
    pub kind: BatchLifecycleKind,
    pub batch_id: String,
    pub request_count: u32,
    pub model: String,
    pub spend_cents_half: i64,
    pub task_id: String,
}

// ---------------------------------------------------------------------------
// H-plan verifiability substrate payload structs and Kind enums (SCHEMA_VERSION 225+)
// ---------------------------------------------------------------------------

/// H-17: Sharpe-ratio deflation gate verdict.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct DeflationGateEvaluatedPayload {
    pub objective: String,
    pub data_window: String,
    pub n_used: u64,
    pub dsr: f64,
    pub dsr_pvalue: f64,
    pub sigma_sr: f64,
    pub sr_0: f64,
    pub pbo: f64,
    pub min_trl: f64,
    pub verdict: String,
    pub seed: u64,
    pub corpus_blake3: String,
}

/// H-18: data-leakage blocked kind.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum LeakageBlockedKind {
    NoTestSet,
    PreprocessBeforeSplit,
    FeatureSelectBeforeSplit,
    HumanReviewRequired,
}

impl LeakageBlockedKind {
    pub fn label(&self) -> &'static str {
        match self {
            Self::NoTestSet => "no_test_set",
            Self::PreprocessBeforeSplit => "preprocess_before_split",
            Self::FeatureSelectBeforeSplit => "feature_select_before_split",
            Self::HumanReviewRequired => "human_review_required",
        }
    }
}

/// H-18: data-leakage blocked.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct LeakageBlockedPayload {
    pub kind: LeakageBlockedKind,
    pub kapoor_code: String,
    pub module_path: String,
    pub fit_site_file: String,
    pub fit_site_line: u32,
    pub fit_site_symbol: String,
    pub reachability_path: Vec<String>,
    pub verdict: String,
    pub suppressed: bool,
    pub seed: u64,
    pub corpus_blake3: String,
}

/// H-19: lookahead differential kind.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum LookaheadDifferentialKind {
    Causal,
    FutureLeak,
}

impl LookaheadDifferentialKind {
    pub fn label(&self) -> &'static str {
        match self {
            Self::Causal => "causal",
            Self::FutureLeak => "future_leak",
        }
    }
}

/// H-19: lookahead differential verdict.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct LookaheadDifferentialPayload {
    pub kind: LookaheadDifferentialKind,
    pub strategy_path: String,
    pub first_change_column: String,
    pub first_change_index: u32,
    pub abs_diff: f64,
    pub rel_diff: f64,
    pub tolerance: f64,
    pub verdict: String,
    pub seed: u64,
    pub corpus_blake3: String,
}

/// H-20: trading null-hypothesis test kind.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum TradingNullTestKind {
    EdgeCollapsedToNull,
    EdgeSurvivedNull,
    Unprovable,
}

impl TradingNullTestKind {
    pub fn label(&self) -> &'static str {
        match self {
            Self::EdgeCollapsedToNull => "edge_collapsed_to_null",
            Self::EdgeSurvivedNull => "edge_survived_null",
            Self::Unprovable => "unprovable",
        }
    }
}

/// H-20: trading null-hypothesis test verdict.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct TradingNullTestPayload {
    pub kind: TradingNullTestKind,
    pub strategy_path: String,
    pub statistic: f64,
    pub p_value: f64,
    pub null_band_low: f64,
    pub null_band_high: f64,
    pub n_trials: u64,
    pub k_permutations: u64,
    pub alpha: f64,
    pub verdict: String,
    pub seed: u64,
    pub corpus_blake3: String,
}

/// H-21: rerun hash equality kind.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum RerunHashEqualityKind {
    Equal,
    Divergent,
}

impl RerunHashEqualityKind {
    pub fn label(&self) -> &'static str {
        match self {
            Self::Equal => "equal",
            Self::Divergent => "divergent",
        }
    }
}

/// H-21: rerun hash equality verdict.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct RerunHashEqualityPayload {
    pub kind: RerunHashEqualityKind,
    pub strategy_path: String,
    pub digest_a: String,
    pub digest_b: String,
    pub first_divergence_index: u32,
    pub divergent_metric: String,
    pub declared_metric_count: u32,
    pub verdict: String,
    pub seed: u64,
    pub corpus_blake3: String,
}

/// H-22: sim vs real parity kind.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum SimRealParityKind {
    Parity,
    ReplayGoldenMismatch,
}

impl SimRealParityKind {
    pub fn label(&self) -> &'static str {
        match self {
            Self::Parity => "parity",
            Self::ReplayGoldenMismatch => "replay_golden_mismatch",
        }
    }
}

/// H-22: sim vs real parity verdict.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct SimRealParityPayload {
    pub kind: SimRealParityKind,
    pub strategy_path: String,
    pub golden_digest: String,
    pub replay_digest: String,
    pub first_divergence_index: u32,
    pub divergent_event: String,
    pub declared_event_count: u32,
    pub verdict: String,
    pub seed: u64,
    pub corpus_blake3: String,
}

/// H-23: result manifest verified kind.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum ResultManifestVerifiedKind {
    ReDerived,
    FabricationDetected,
}

impl ResultManifestVerifiedKind {
    pub fn label(&self) -> &'static str {
        match self {
            Self::ReDerived => "re_derived",
            Self::FabricationDetected => "fabrication_detected",
        }
    }
}

/// H-23: result manifest verified verdict.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct ResultManifestVerifiedPayload {
    pub kind: ResultManifestVerifiedKind,
    pub strategy_path: String,
    pub run_id: String,
    pub offending_metric: String,
    pub declared_result_count: u32,
    pub manifest_digest: String,
    pub rederived_digest: String,
    pub verdict: String,
    pub seed: u64,
    pub corpus_blake3: String,
}

/// H-24: differential divergence kind.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum DifferentialDivergenceKind {
    InvariantHeld,
    ShadowDiverged,
    InvariantViolated,
}

impl DifferentialDivergenceKind {
    pub fn label(&self) -> &'static str {
        match self {
            Self::InvariantHeld => "invariant_held",
            Self::ShadowDiverged => "shadow_diverged",
            Self::InvariantViolated => "invariant_violated",
        }
    }
}

/// H-24: differential divergence between primary and shadow.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct DifferentialDivergencePayload {
    pub kind: DifferentialDivergenceKind,
    pub strategy_path: String,
    pub run_id: String,
    pub offending_invariant: String,
    pub declared_trade_count: u32,
    pub primary_digest: String,
    pub shadow_digest: String,
    pub first_divergent_field: String,
    pub divergence_metric: f64,
    pub verdict: String,
    pub seed: u64,
    pub corpus_blake3: String,
}

/// H-25: gate self-validation kind.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum GateSelfValidationKind {
    AllGatesValid,
    EmptyRegistry,
    GateFailedToBlock,
    GateUnreachable,
    GateOverBlocked,
}

impl GateSelfValidationKind {
    pub fn label(&self) -> &'static str {
        match self {
            Self::AllGatesValid => "all_gates_valid",
            Self::EmptyRegistry => "empty_registry",
            Self::GateFailedToBlock => "gate_failed_to_block",
            Self::GateUnreachable => "gate_unreachable",
            Self::GateOverBlocked => "gate_over_blocked",
        }
    }
}

/// H-25: gate self-validation mutation score verdict.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct GateSelfValidationPayload {
    pub kind: GateSelfValidationKind,
    pub gates_total: u32,
    pub gates_killed: u32,
    pub mutation_score_pct: f64,
    pub escaped_gate: String,
    pub escaped_fixture: String,
    pub run_id: String,
    pub seed: u64,
    pub corpus_blake3: String,
}

/// H-26: reward tampering detection kind.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum RewardTamperingKind {
    NoTampering,
    EvaluatorModified,
    ImplausibleRewardDelta,
}

impl RewardTamperingKind {
    pub fn label(&self) -> &'static str {
        match self {
            Self::NoTampering => "no_tampering",
            Self::EvaluatorModified => "evaluator_modified",
            Self::ImplausibleRewardDelta => "implausible_reward_delta",
        }
    }
}

/// H-26: reward tampering detection verdict.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct RewardTamperingPayload {
    pub kind: RewardTamperingKind,
    pub harness_blake3: String,
    pub prior_reward: f64,
    pub new_reward: f64,
    pub reward_delta: f64,
    pub plausible_band: f64,
    pub run_id: String,
    pub seed: u64,
    pub corpus_blake3: String,
}

/// H-27: holdout budget kind.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum HoldoutBudgetKind {
    Running,
    Exhausted,
    QuerySpent,
    Provisioned,
}

impl HoldoutBudgetKind {
    pub fn label(&self) -> &'static str {
        match self {
            Self::Running => "running",
            Self::Exhausted => "exhausted",
            Self::QuerySpent => "query_spent",
            Self::Provisioned => "provisioned",
        }
    }
}

/// H-27: holdout budget exhaustion or running.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct HoldoutBudgetPayload {
    pub kind: HoldoutBudgetKind,
    pub holdout_id: String,
    pub objective: String,
    pub queries_spent: u64,
    pub budget_total: u64,
    pub noisy_statistic: Option<f64>,
    pub data_window: String,
    pub run_id: String,
    pub seed: u64,
    pub corpus_blake3: String,
}

/// H-28: drift interlock kind.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum DriftInterlockKind {
    Running,
    Warning,
    DriftHalted,
    HumanUnlocked,
}

impl DriftInterlockKind {
    pub fn label(&self) -> &'static str {
        match self {
            Self::Running => "running",
            Self::Warning => "warning",
            Self::DriftHalted => "drift_halted",
            Self::HumanUnlocked => "human_unlocked",
        }
    }
}

/// H-28: drift interlock running/warning/halted.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct DriftInterlockPayload {
    pub kind: DriftInterlockKind,
    pub strategy_id: String,
    pub breached_axes: String,
    pub run_id: String,
    pub seed: u64,
    pub evidence_blake3: String,
}

/// H-17: strategy trial count persisted.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct TrialRegisteredPayload {
    pub objective: String,
    pub data_window: String,
    pub config_hash: String,
    pub seed: i64,
    pub n_after: u64,
}

/// C-36 extension: MuCoCo inconsistency detected.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct MucocoInconsistencyDetectedPayload {
    pub operator: String,
    pub language: String,
    pub source_hash: u64,
    pub original_output_hash: u64,
    pub mutated_output_hash: u64,
    pub divergence: String,
    pub timestamp_ms: i64,
}

/// C-36 extension: MuCoCo operator inapplicable skip reason.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct MucocoSkipReasonPayload {
    pub operator: String,
    pub language: String,
    pub source_hash: u64,
    pub skip_reason: String,
    pub timestamp_ms: i64,
}

/// SQL CREATE for the strategy_trials auxiliary table used by H-17 TrialLedger.
pub const CREATE_STRATEGY_TRIALS_TABLE: &str = "\
CREATE TABLE IF NOT EXISTS strategy_trials (
    id          INTEGER PRIMARY KEY AUTOINCREMENT,
    objective   TEXT    NOT NULL,
    data_window TEXT    NOT NULL,
    config_hash TEXT    NOT NULL,
    seed        INTEGER NOT NULL,
    created_at  INTEGER NOT NULL,
    UNIQUE(objective, data_window, config_hash, seed)
);
CREATE INDEX IF NOT EXISTS idx_strategy_trials_obj ON strategy_trials(objective, data_window);
";

/// SQL CREATE for the holdout_budget auxiliary table used by H-19 / H-47.
///
/// Schema updated in H-47 to carry the full per-row provenance columns used by
/// `holdout_budget.rs` (row_id dedup key, kind discriminator, data_window,
/// seed, created_at_ms, session_id). The old schema only had 5 columns;
/// `HoldoutBudgetLedger::open` runs idempotent `ALTER TABLE ADD COLUMN`
/// migrations for existing databases (see `open` in `holdout_budget.rs`).
pub const CREATE_HOLDOUT_BUDGET_TABLE: &str = "\
CREATE TABLE IF NOT EXISTS holdout_budget (
    id             INTEGER PRIMARY KEY AUTOINCREMENT,
    row_id         TEXT    NOT NULL DEFAULT '',
    kind           TEXT    NOT NULL DEFAULT 'spent',
    holdout_id     TEXT    NOT NULL,
    objective      TEXT    NOT NULL,
    data_window    TEXT    NOT NULL DEFAULT '',
    budget_total   INTEGER NOT NULL DEFAULT 0,
    seed           INTEGER NOT NULL DEFAULT 0,
    created_at_ms  INTEGER NOT NULL DEFAULT 0,
    session_id     TEXT    NOT NULL DEFAULT ''
);
CREATE INDEX IF NOT EXISTS idx_holdout_budget_obj ON holdout_budget(objective);
CREATE INDEX IF NOT EXISTS idx_holdout_budget_holdout ON holdout_budget(holdout_id, objective, kind);
";

/// SQL CREATE for the drift_interlock auxiliary table used by H-28.
pub const CREATE_DRIFT_INTERLOCK_TABLE: &str = "\
CREATE TABLE IF NOT EXISTS drift_interlock (
    id           INTEGER PRIMARY KEY AUTOINCREMENT,
    strategy_id  TEXT    NOT NULL,
    kind         TEXT    NOT NULL,
    breached_axes TEXT   NOT NULL,
    created_at   INTEGER NOT NULL
);
CREATE INDEX IF NOT EXISTS idx_drift_interlock_strat ON drift_interlock(strategy_id);
";

// ---------------------------------------------------------------------------
// Additional missing payload types (user-decision, staleness, canvas const)
// ---------------------------------------------------------------------------

/// H-30: user-decision event kind.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum UserDecisionEventKind {
    Captured,
    Superseded,
    ProposalSurfaced,
}

impl UserDecisionEventKind {
    pub fn label(&self) -> &'static str {
        match self {
            Self::Captured => "captured",
            Self::Superseded => "superseded",
            Self::ProposalSurfaced => "proposal_surfaced",
        }
    }
}

/// H-30: user decision captured or superseded.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct UserDecisionSupersededPayload {
    pub kind: UserDecisionEventKind,
    pub old_fact_id: String,
    pub new_fact_id: String,
    pub fact_type: String,
    pub key: String,
    pub evidence_blake3: String,
}

/// H-17 staleness benchmark measured result.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct StalenessEvalMeasuredPayload {
    pub suite: String,
    pub n_cases: u32,
    pub sr_rate: f64,
    pub ipa_rate: f64,
    pub recognition_application_gap: f64,
    pub stale_fact_reached_prompt: u64,
    pub total_retrievals: u64,
    pub broken_arm: bool,
    pub instrument_failed: bool,
    pub seed: u64,
    pub corpus_blake3: String,
}

/// D-27.1 — review status for a discovery-candidate template.
///
/// Variants:
///   - `Ratified` — reviewer accepted the candidate for adversarial verification.
///   - `Rejected`  — reviewer dismissed the candidate (carries a reason string).
///   - `Pending`   — default state; no reviewer action has been recorded yet.
///
/// Serialised as lowercase snake_case via `#[serde(rename_all = "snake_case")]`.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ReviewStatus {
    Ratified,
    Rejected,
    Pending,
}

/// H-32 payload for [`SsbActionType::FactValidationFailed`].
///
/// Emitted by the citation-validity gate when a Tier-A `DomainKnowledge` or
/// `Mistake` fact carries a DKG symbol citation that no longer resolves against
/// the current symbol graph. The fact is HARD-EXCLUDED from the injected context
/// slice; this row is the off-agent artifact (R4 — the agent did not author it).
///
/// `reason` is `"stale"` when the symbol was previously resolvable but has since
/// been renamed or deleted, or `"unknown"` when the citation index was unavailable
/// (treated as FAIL per R2 — missing evidence is a hard FAIL, not SKIP).
/// `evidence_blake3` is the blake3 16-hex citation fingerprint (anti-stale: NEVER
/// sha2/blake2 per CLAUDE.md) of `<name>@<file>:<line>`, or empty string when no
/// parseable `dkg:` pointer was found.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FactValidationFailedPayload {
    /// `fact_id` of the excluded Tier-A fact.
    pub fact_id: String,
    /// The raw `dkg:<name>@<file>:<line>` citation string from `evidence_pointers`,
    /// or empty when the citation pointer was unparseable.
    pub citation: String,
    /// `"stale"` | `"unknown"` (snake_case discriminator; free-form for dashboard
    /// rendering so future reasons can land without a SCHEMA bump).
    pub reason: String,
    /// blake3 16-hex fingerprint of the canonical `<name>@<file>:<line>` string.
    /// Empty string when no parseable `dkg:` pointer existed on the row.
    pub evidence_blake3: String,
}

/// H-33 kind discriminator for [`SsbActionType::MemoryGovernanceEvent`].
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MemoryGovernanceKind {
    /// User explicitly deleted/forgot a fact (`elai memory forget` or dashboard).
    Forgotten,
    /// User pinned a fact to prevent decay-tick from reducing its trust score.
    Pinned,
    /// User unpinned a previously-pinned fact (decay resumes on next tick).
    Unpinned,
}

/// H-33 payload for [`SsbActionType::MemoryGovernanceEvent`].
///
/// Emitted by `elai memory forget|pin|unpin` and the dashboard
/// `memory_forget` / `memory_pin` Tauri commands. The payload is the
/// authoritative off-agent governance beacon (R4) — the fact store
/// soft-close is the primary mutation; this row lets auditors and the decay
/// tick observe the governance intent without polling the store.
///
/// `actor` is `"cli"` or `"dashboard"` to distinguish the write path.
/// `session_id` is the ELAI session that emitted the beacon.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MemoryGovernanceEventPayload {
    /// `fact_id` of the governed fact.
    pub fact_id: String,
    /// Governance action taken.
    pub kind: MemoryGovernanceKind,
    /// `"cli"` | `"dashboard"` — write-path discriminator.
    pub actor: String,
    /// ELAI session that emitted the beacon.
    pub session_id: String,
    /// Wall-clock ms of emission.
    pub created_at_ms: u64,
}

/// H-34 payload for [`SsbActionType::McpToolDeferred`].
///
/// Emitted by `mcp_glue::RuntimeMcpState::call_tool` on each cache-miss
/// (first use of a withheld tool) when `ELAI_MCP_DEFER_SCHEMA_ENABLE=1`.
/// One row per deferred-schema resolve, NOT per subsequent hit.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct McpToolDeferredPayload {
    /// MCP server name (e.g. `"local-rag"`).
    pub server_name: String,
    /// Fully-qualified tool name (e.g. `"local-rag__mcp_search_context"`).
    pub tool_qualified_name: String,
    /// Schema-resolve latency in nanoseconds.
    pub resolve_ns: u64,
    /// `true` when the schema was already cached (hit); `false` on first-resolve (miss).
    pub cache_hit: bool,
    /// Number of tools in the always-floor injected set at time of this resolve.
    pub injected_floor_size: u32,
}

/// H-34 payload for [`SsbActionType::McpSchemaBudgetMeasured`].
///
/// Emitted by `elai bench mcp-schema-budget`. All token counts are COMPUTED
/// from the corpus fixture — no numeric literals in the runner.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct McpSchemaBudgetMeasuredPayload {
    /// Suite label (e.g. `"mcp_schema_budget"`).
    pub suite: String,
    /// Baseline schema tokens per turn (all tools injected eagerly).
    pub baseline_schema_tokens_per_turn: u64,
    /// Injected schema tokens per turn after defer-load (floor + selected only).
    pub injected_schema_tokens_per_turn: u64,
    /// Reduction in tokens: `baseline - injected` (signed; negative = regression).
    pub reduction_tokens: i64,
    /// Number of deferred-schema resolves recorded across the fixture run.
    pub deferred_resolves: u32,
    /// Deterministic RNG seed used (`ELAI_SEED`, default 0).
    pub seed: u64,
    /// blake3 hex of the canonical corpus bytes (provenance / staleness pin).
    pub corpus_blake3: String,
}

/// H-35 payload for [`SsbActionType::SessionRewound`].
///
/// Emitted by `elai rewind <target>` (CLI) and the `rewind_session` Tauri command
/// on each successful rewind. A no-op rewind (target==head) emits NO row.
/// All fields are intent/outcome only -- message content is NEVER stored here.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SessionRewoundPayload {
    /// Session that was rewound.
    pub session_id: String,
    /// SSB seq that was the rewind target (the "go back to here" point).
    pub target_seq: i64,
    /// SSB seq that was the head BEFORE the rewind (the "current" point).
    pub head_seq_before: i64,
    /// Scope: "conversation" | "code" | "both".
    pub scope: String,
    /// Whether the tail past target_seq was summarized via compact_session.
    pub summarized: bool,
    /// Replay-session id minted by ssb_resume_from (code-restore path).
    /// None when scope=="conversation".
    pub replay_session_id: Option<String>,
    /// Outcome of the code-restore predicate check:
    /// "Accepted" | "Refused" | "PredicateUnavailable" | "n/a".
    pub replay_outcome: String,
    /// blake3 hex of the (seq, action_type, event_blake3) triple of the
    /// tail that was moved past. Empty string on no-op (should never reach
    /// this field since no-op emits no row).
    pub tail_fingerprint_blake3: String,
}

/// H-36 discriminator: which trigger fired the notification.
/// `serde(rename_all = "snake_case")` keeps wire tags stable.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AgentNotificationKind {
    /// The task completed (PhaseComplete write succeeded, anti-fabrication gate
    /// did NOT halt the run).
    Completion,
    /// An autonomous decision escalated to the human (AutonomyDecision::RequestHuman,
    /// fix_and_rerun cap exhausted).
    NeedsInput,
}

/// H-36 payload for [`SsbActionType::AgentNotificationEmitted`].
///
/// Written by `notify_dispatch::NotificationDispatcher::notify` once per dispatch
/// event. Never updated; append-only (one row per fire). Token-injection surface is
/// ZERO -- notification text is two fixed short strings, never model-generated.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AgentNotificationEmittedPayload {
    /// Two-arm discriminator: Completion | NeedsInput.
    pub kind: AgentNotificationKind,
    /// Bitset: bit 0 = terminal escape attempted, bit 1 = OS notification attempted.
    /// Both legs are always attempted when applicable (escape when TTY, OS when
    /// ELAI_NOTIFY_ENABLE=1). 0b01 = escape-only (default, no env gate needed).
    pub surfaces_attempted: u8,
    /// Bitset matching `surfaces_attempted`: bit set iff the surface SUCCEEDED.
    /// OS notification bit clear when the platform binary is absent (fail-graceful,
    /// not panic); terminal escape bit clear on non-TTY stdout.
    pub surfaces_succeeded: u8,
    /// Session ID at the time of dispatch (from the SSB writer's session context).
    pub session_id: String,
    /// Unix milliseconds at dispatch time (from ClockProvider).
    pub ts_ms: u64,
}

/// H-37 discriminator: outcome of the plan-approval gate.
/// Pattern-2 single-variant with 4-arm kind discriminator.
/// `serde(rename_all = "snake_case")` keeps wire tags stable.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PlanDecisionKind {
    /// Human (or LOW-risk auto-advance) approved the plan.
    Approve,
    /// Human redirected to a different valid FSM target.
    Redirect,
    /// Human (or fail-closed gate) rejected the plan; FSM stays at source.
    Reject,
    /// Autonomous mode with gate handle absent: auto-advanced without prompt.
    AutoAdvance,
}

impl PlanDecisionKind {
    /// SQL/wire label for the kind (used in SSB payload serialisation).
    pub fn label(&self) -> &'static str {
        match self {
            Self::Approve => "approve",
            Self::Redirect => "redirect",
            Self::Reject => "reject",
            Self::AutoAdvance => "auto_advance",
        }
    }
}

/// H-37 payload for [`SsbActionType::PlanDecisionEvent`].
///
/// Emitted once per PLANNING|FEASIBILITY_PROBE -> TESTING_FIRST transition
/// attempt when `ELAI_PLAN_APPROVAL_GATE_ENABLE=1`. Append-only; one row per
/// gate invocation. A REJECT row is followed by NO PhaseTransition row (FSM
/// stays at source). A REDIRECT row carries `target_state` = the redirected
/// target, not TESTING_FIRST.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PlanDecisionPayload {
    /// Outcome: Approve | Redirect | Reject | AutoAdvance.
    pub decision_kind: PlanDecisionKind,
    /// The FSM target state attempted (typically "TESTING_FIRST"; "PLANNING" on Redirect).
    pub target_state: String,
    /// Risk classification used by the gate (LOW/MEDIUM/HIGH).
    pub risk_level: String,
    /// Human-provided reason for Reject or Redirect; None for Approve/AutoAdvance.
    pub reason: Option<String>,
    /// FSM phase label at the time of the gate invocation.
    pub plan_session_id: String,
}

/// H-38 outcome discriminator for [`StopConfirmationPayload`].
///
/// 3-arm discriminator for the Pattern-2 single-variant `StopConfirmationEvent`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum StopOutcomeKind {
    /// The user confirmed; the stop proceeds.
    ConfirmedProceed,
    /// The user rejected the prompt (or no gate was installed — fail-closed); stop held.
    RejectedHeld,
    /// Cost + unsaved + pod all below thresholds; stop passed with zero friction.
    /// Emitted only when `ELAI_STOP_COST_CONFIRM_ENABLE=1` (SSB-telemetry-gated).
    PassedNoFriction,
}

/// H-38 payload for [`SsbActionType::StopConfirmationEvent`].
///
/// Emitted once per `dispatch_stop` call that reaches the cost-gate path.
/// `ConfirmedProceed` / `RejectedHeld` are emitted unconditionally (the safety
/// behavior); `PassedNoFriction` only when `ELAI_STOP_COST_CONFIRM_ENABLE=1`.
/// The `cost_usd` field mirrors what `SsbViewer::cloud_spend_summary` returns for
/// the current UTC day at stop time — recomputable from the SSB CloudSpend ledger
/// under any `ELAI_SEED` (off-agent, R4-compliant).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct StopConfirmationPayload {
    /// Gate outcome: ConfirmedProceed | RejectedHeld | PassedNoFriction.
    pub outcome: StopOutcomeKind,
    /// Day-spend (USD) at the moment of the stop (from CloudSpend ledger, may be 0.0).
    pub cost_usd: f64,
    /// Number of unsaved artifacts in the run at stop time (0 = fully saved).
    pub unsaved_count: u32,
    /// Wall-clock milliseconds since UNIX epoch at stop time.
    pub ts_ms: i64,
}

// === H-39 (SCHEMA_VERSION 218) -- AbstentionScored substrate

/// H-39 payload for [`SsbActionType::AbstentionScored`].
///
/// Emitted ONCE per bench run after computing the [`AbstentionAggregate`]
/// over the run's outcome triples. Closes the self-improvement loop
/// misalignment: D-16's `RefusalDecision::Abstain` fires at inference but
/// darwin_admit_loop's reward was abstention-blind (GAP-1, H-39 plan §Context).
///
/// Scoring formula (arXiv:2509.04664 Table 2):
///   - correct & answered  -> +1.0
///   - abstained           ->  0.0
///   - answered & wrong    -> -t/(1-t) = -3.0 at default t=0.75
///
/// Off-agent artifact (R4-compliant): the fields are recomputable from the
/// frozen fixture + ELAI_SEED; the row is written by the bench runner, not
/// by the agent under test. Flat beacon (no Kind discriminator).
///
/// Default-ON, kill-switch `ELAI_ABSTENTION_SCORE_DISABLE` (W-2).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AbstentionScoredPayload {
    /// Benchmark suite label (e.g. "seccodeplt", "safedelta").
    pub suite: String,
    /// Mean abstention-aware score over all n items. Range [-penalty, +1.0].
    pub mean_score: f64,
    /// Fraction of items that were abstentions. In [0, 1].
    pub abstention_rate: f64,
    /// Fraction of items that were confident-wrong (answered but incorrect). In [0, 1].
    pub confident_wrong_rate: f64,
    /// Confidence target t used. LOCKED at DEFAULT_CONFIDENCE_TARGET = 0.75.
    pub t: f64,
    /// Number of items scored.
    pub n: u32,
    /// Deterministic RNG seed used to assemble the run.
    pub seed: u64,
    /// blake3 hex-prefix of the corpus blob (provenance; never the full corpus).
    pub corpus_blake3: String,
}

// === H-41 (SCHEMA_VERSION 219) -- CheckerGapDetected substrate

/// H-41 payload for [`SsbActionType::CheckerGapDetected`].
///
/// Emitted at the `route_claim_with_gap` call site (wrapping
/// `agent_pool::route_claim`) when a Factual or Computational claim falls to
/// a non-Constructive grounding mode because no `VerifierKind::Deterministic`
/// verifier is present in `OrchContext.available_verifiers`.
///
/// Off-agent artifact (R4-compliant): written by the routing wrapper, NOT by
/// the agent whose claim is being graded. The backlog row is verifiable from
/// the SSB journal independently of the agent. Missing = FAIL (R1).
///
/// Wire-dark under `ELAI_CHECKER_GAP_ENABLE` kill-switch (R9).
/// Flat beacon (no Kind discriminator). TOTAL_VARIANTS 529 -> 530.
///
/// Grounding:
/// - Kambhampati arXiv:2402.08115 — Mode-A coverage is the verification lever.
/// - Xu arXiv:2401.11817 Table 3 — NP-complete / undecidable = never-LLM-grade.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CheckerGapDetectedPayload {
    /// Identifier of the claim that triggered the gap (mirrors `claim_id` in
    /// the paired `claim_routed` row).
    pub claim_id: String,
    /// Claim type label — one of the [`crate::claim::ClaimType`] `as_str()`
    /// values (`"factual"` or `"computational"` in the gap path).
    pub claim_type: String,
    /// Xu Table-3 complexity taxon — one of the
    /// [`crate::checker_coverage::ComplexityTaxon`] `as_str()` values.
    pub complexity_taxon: String,
    /// The grounding mode the claim FELL to (e.g. `"falsifiability_cross_family"`
    /// or `"conformal_refusal"`). Mirrors the `mode` field in the paired
    /// `claim_routed` row.
    pub fell_to_mode: String,
    /// Wall-clock timestamp (ms since UNIX_EPOCH) from the injected
    /// `ClockProvider`. Recomputable from ELAI_SEED (GD-2).
    pub ts_ms: i64,
}

// === H-42 (SCHEMA_VERSION 220) -- ReasoningBudgetScaled substrate

/// H-42 payload for [`SsbActionType::ReasoningBudgetScaled`].
///
/// Emitted at both dispatch sites (agent_pool::spawn and bsp/engine.rs timeout path)
/// when `reasoning_budget::scale_grc_budget` raises a sub-task's GRC budget above the
/// base per-role config.
///
/// Off-agent artifact (R4-compliant): written at the dispatch call site by the
/// orchestrator, NOT by the agent whose budget is being scaled. The row is verifiable
/// from the SSB journal independently of the agent. Missing = FAIL (R1).
///
/// Default-ON, kill-switch `ELAI_REASONING_BUDGET_SCALE_DISABLE`.
/// Pattern-2 single-variant 2-arm ReasoningBudgetScaledKind: DepthBound / AccessBound.
/// TOTAL_VARIANTS 530 -> 531.
///
/// Grounding:
/// - Merrill & Sabharwal arXiv:2310.07923 — single forward pass is bounded ~TC0;
///   more context buys ACCESS not DEPTH. Formal basis for the depth-not-width guard.
/// - Li et al. arXiv:2402.12875 — T serial CoT steps simulate a size-T circuit.
/// - Yehudai et al. arXiv:2503.01544 — compositional tasks need ~O(n) CoT tokens.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ReasoningBudgetScaledPayload {
    /// Dominant failure class for the SSB Kind arm: "depth_bound", "access_bound",
    /// or "neither".  Derived from [`crate::reasoning_budget::FailureClass::as_ssb_kind_str`].
    /// On a normal dispatch (no prior failure) this is "neither".
    pub kind: String,
    /// Agent role name (e.g. "executor", "auditor"). Mirrors AgentGrcConfig.role.
    pub role: String,
    /// Base (un-scaled) max_calls from the per-role AgentGrcConfig before scaling.
    pub base_calls: u32,
    /// Serial depth d computed by `estimate_serial_depth(level)`.
    /// 0 = Concept, 1 = Phase, 2 = Step, 3 = BabyStep.
    pub serial_depth: u32,
    /// Scaled max_calls after `scale_grc_budget(base, d)`, clamped to
    /// `LOCKED_BUDGET_CEILING × base_calls`.
    pub scaled_calls: u32,
    /// Effective ceiling = `LOCKED_BUDGET_CEILING × base_calls`. Serialised for
    /// auditability — verifiers can confirm no row exceeds its own ceiling.
    pub ceiling: u32,
    /// ELAI_SEED value at the time of dispatch (GD-2 reproducibility).
    /// Recomputed identically from (base_calls, serial_depth) under the same seed.
    pub seed: u64,
    // --- I-26 payload extension (serde(default) + skip_serializing_if preserves HD-2 byte-stability
    //     for all rows emitted before I-26 shipped; old rows decode with None on these fields).
    /// Search-DAG node depth when emitted from the I-26 BudgetController.
    /// None on GRC-dispatch rows (H-42 agent_pool / bsp path); populated only by the I-26 search path.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub depth_level: Option<u32>,
    /// Budget consumed as a fraction 0.0–1.0 across all active siblings.
    /// None on GRC-dispatch rows. Set when the BudgetController's collapse check runs.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub budget_pct: Option<f32>,
    /// True when the plateau detector fired (top-L2 score flat across 5 supersteps).
    /// None on GRC-dispatch rows. Populated only by the I-26 BudgetController collapse path.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub plateau_fired: Option<bool>,
}

// === H-43 (SCHEMA_VERSION 221) -- FitnessEvaluated substrate

/// H-43 Kind discriminator for [`SsbActionType::FitnessEvaluated`].
///
/// - `ExecutionPassRate`: primary signal — pass_rate over the frozen test corpus
///   (CodeRM arXiv:2501.01054). Emitted for candidates that complete evaluation.
/// - `DualAgreement`: secondary signal — dual-execution output-consensus mass
///   (CodeT arXiv:2207.10397). Emitted alongside ExecutionPassRate.
/// - `FitnessRegression`: candidate scored below its parent's fitness. Emitted
///   when a child's fitness < parent's fitness (two-gate margin < 0). Signals
///   the loop is not climbing — the promotion gate sees a negative margin.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FitnessEvaluatedKind {
    /// Primary pass-rate signal (CodeRM); candidate fitness measured by
    /// execution against the frozen test corpus.
    ExecutionPassRate,
    /// Secondary dual-agreement signal (CodeT); output-consensus mass
    /// across solutions over the same test corpus.
    DualAgreement,
    /// Candidate scored below parent — the loop would regress if promoted.
    FitnessRegression,
}

impl FitnessEvaluatedKind {
    /// Stable snake_case wire string for the SSB kind arm.
    #[must_use]
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::ExecutionPassRate => "execution_pass_rate",
            Self::DualAgreement => "dual_agreement",
            Self::FitnessRegression => "fitness_regression",
        }
    }
}

/// H-43 payload for [`SsbActionType::FitnessEvaluated`].
///
/// Emitted once per scored candidate in
/// `darwin_admit_loop::run_one_generation` under
/// `cfg(feature = "darwinian-evolve")` and `ELAI_DARWINIAN_LOOP_ENABLE=1`.
///
/// Off-agent artifact (R4): written at the scoring call site inside the
/// orchestrator, NOT by the candidate whose fitness is being measured.
/// Missing rows = FAIL (R1): absence means the fitness gate never ran.
///
/// Wiring:
/// - `pass_rate`: CodeRM signal — fraction of frozen corpus tests that passed.
/// - `dual_agreement`: CodeT signal — output-consensus mass.
/// - `fitness`: weighted composite = 0.7 * pass_rate + 0.3 * dual_agreement.
///   Weights are LOCKED at compile time (VD-1 / TD-2 freeze); a self-mod
///   loop cannot tune them toward a gameable signal.
/// - `test_set_hash`: blake3 of the frozen test corpus bytes (provenance pin,
///   anti-gaming R8; the corpus is gate-frozen and read-only to the loop).
/// - `candidate_blake3`: the 16-char hex prefix from `candidate_blake3()`.
///
/// GD-2 reproducibility: the `{pass_rate, dual_agreement, fitness}` triple
/// recomputes byte-identically under the same `seed` and `test_set_hash`.
/// TOTAL_VARIANTS 531 -> 532.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FitnessEvaluatedPayload {
    /// Dominant signal arm: `ExecutionPassRate` for normal scoring;
    /// `DualAgreement` for the agreement leg; `FitnessRegression` when the
    /// child scored below the parent.
    pub kind: FitnessEvaluatedKind,
    /// Fraction of frozen corpus tests the candidate passed (CodeRM signal).
    /// In [0.0, 1.0]; 0.0 = no tests passed.
    pub pass_rate: f64,
    /// Dual-execution output-consensus mass (CodeT signal). In [0.0, 1.0].
    pub dual_agreement: f64,
    /// Number of frozen corpus tests evaluated.
    pub tests_total: u32,
    /// Weighted composite fitness = W_PASS * pass_rate + W_AGREE * dual_agreement.
    /// Weights are LOCKED (0.7 / 0.3) per VD-1.
    pub fitness: f64,
    /// ELAI_SEED value at the time of evaluation (GD-2 reproducibility).
    pub seed: u64,
    /// blake3 hash of the frozen test corpus bytes (anti-gaming provenance pin).
    /// A zero array indicates a test-only / fixture context.
    pub test_set_hash: [u8; 32],
    /// blake3 hex-prefix (16 chars) of the candidate `config_blob`.
    pub candidate_blake3: String,
}

// === H-44 (SCHEMA_VERSION 222) -- TrapBehaviorDetected substrate

/// H-44 Kind discriminator for [`SsbActionType::TrapBehaviorDetected`].
///
/// - `ImpossiblePassExploit`: the candidate passed a frozen conflict-spec task
///   where a correct solution CANNOT pass (ImpossibleBench 2510.20270).
///   Pass-rate on an impossible task == cheat-rate.
/// - `IsomorphMismatch`: the candidate passed the original task but failed a
///   logically-isomorphic rewrite (Isomorphic-Perturbation 2604.15149).
///   Passes original but fails isomorph == gaming the verifier.
/// - `ExecutionHallucination`: the candidate looks correct but behaves wrongly
///   at runtime — "looks right, runs wrong" (CodeHalu 2405.00253). Detected
///   via the execution fitness harness (H-43 compose, GD-1).
/// - `KeywordFallback`: the keyword-grep advisory triggered (retained as cheap
///   first-pass). NEVER the sole blocker (VD-2). Emitted only when no
///   behavioral leg fired and the keyword grep found a match.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TrapBehaviorDetectedKind {
    /// ImpossibleBench conflict canary: candidate passed an impossible task
    /// (arXiv:2510.20270, Zhong/Raghunathan/Carlini).
    ImpossiblePassExploit,
    /// Isomorphic-Perturbation mismatch: passes original, fails isomorphic
    /// rewrite (arXiv:2604.15149).
    IsomorphMismatch,
    /// Execution hallucination: looks right but fails execution (CodeHalu
    /// arXiv:2405.00253, composes H-43 ExecutionFitnessHarness).
    ExecutionHallucination,
    /// Keyword-grep advisory triggered (cheap first-pass only, VD-2).
    KeywordFallback,
    /// Obligation-closure escape-hatch detected (H-54): an unregistered
    /// `sorry`/`admit`/`axiom`/`assume`/`#[verifier::external]` token sits
    /// anywhere in the transitive dependency closure of a "discharged" goal
    /// (arXiv:2605.22763 AlphaProof Nexus — the relocated-sorry + false-
    /// known-result evasion classes, the whole-closure defense).
    ///
    /// This is the deterministic FIRST leg of the behavioral trap-detection
    /// stack on the formal-verification lane (E-53). It fires before the
    /// verifier's own discharge and short-circuits when an unregistered escape
    /// hatch is found anywhere in the closure, regardless of how deep or
    /// how renamed the hatch obligation is. NEVER a keyword-only advisory.
    ClosureEscapeHatch,
}

impl TrapBehaviorDetectedKind {
    /// Stable snake_case wire string for the SSB kind arm.
    #[must_use]
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::ImpossiblePassExploit => "impossible_pass_exploit",
            Self::IsomorphMismatch => "isomorph_mismatch",
            Self::ExecutionHallucination => "execution_hallucination",
            Self::KeywordFallback => "keyword_fallback",
            Self::ClosureEscapeHatch => "closure_escape_hatch",
        }
    }
}

/// H-44 payload for [`SsbActionType::TrapBehaviorDetected`].
///
/// Emitted from `darwin_admit_loop::score_candidate` guard chain step (ii)
/// under `cfg(feature = "darwinian-evolve")` and `ELAI_DARWINIAN_LOOP_ENABLE=1`.
///
/// Off-agent artifact (R4): written at the guard chain call site, NOT by the
/// candidate whose config is being evaluated. Missing rows = FAIL (R1).
///
/// The `recall` / `fp_rate` / `n` fields carry the measured recall of the
/// behavioral detector over the frozen TRACE-style corpus (VD-2). A detector
/// with UNMEASURED recall is a FAIL per the wiring contract stub_ban.
///
/// `corpus_hash` is the blake3 hash of the frozen corpus bytes (anti-gaming
/// provenance pin, R8 / TD-2 freeze). NEVER blake2 or sha2.
///
/// GD-2 reproducibility: `recall` and `fp_rate` recompute byte-identically
/// under the same `seed` and `corpus_hash` over the frozen corpus.
/// TOTAL_VARIANTS 532 -> 533.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TrapBehaviorDetectedPayload {
    /// Which behavioral detection arm fired (or keyword fallback).
    pub kind: TrapBehaviorDetectedKind,
    /// Measured recall of the behavioral detector over the frozen TRACE-style
    /// corpus: fraction of planted hacks the detector caught. In [0.0, 1.0].
    pub recall: f64,
    /// Measured false-positive rate over the frozen corpus. In [0.0, 1.0].
    pub fp_rate: f64,
    /// Number of corpus pairs the recall was measured over.
    pub n: usize,
    /// blake3 hash of the frozen corpus bytes (provenance pin, never blake2/sha2).
    pub corpus_hash: [u8; 32],
    /// blake3 hex-prefix (16 chars) of the candidate `config_blob`.
    pub candidate_blake3: String,
    /// ELAI_SEED value at the time of evaluation (GD-2 reproducibility).
    pub seed: u64,
    /// Whether a behavioral leg (not keyword-only) was the deciding signal.
    pub behavioral_flag: bool,
}

// === H-45 (SCHEMA_VERSION 223) -- PromotionScoreAggregated substrate

/// H-45 Kind discriminator for [`SsbActionType::PromotionScoreAggregated`].
///
/// - `WorstCaseScored`: the aggregate is dominated by the worst-case (min)
///   signal; emitted when only one family verdict is available (single-view
///   + penalty) or the variance contribution is below the kappa threshold.
/// - `VariancePenalized`: two or more distinct-family verdicts present; the
///   variance penalty is load-bearing in the aggregate (both WCO + UWO terms
///   are non-trivial; Coste 2310.02743).
/// - `LineageDiverseDisagree`: cross-view disagreement exceeded
///   `DISAGREEMENT_BAND`; promotion is BLOCKED. The payload carries the
///   spread and band for audit (Co-rewarding 2508.00410 two-views stability).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PromotionScoreAggregatedKind {
    /// Worst-case signal from a single-family or low-variance aggregate.
    WorstCaseScored,
    /// Variance penalty load-bearing; multiple distinct families present.
    VariancePenalized,
    /// Cross-view disagreement exceeded the locked band; promotion blocked.
    LineageDiverseDisagree,
}

impl PromotionScoreAggregatedKind {
    /// Stable snake_case wire string for the SSB kind arm.
    #[must_use]
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::WorstCaseScored => "worst_case_scored",
            Self::VariancePenalized => "variance_penalized",
            Self::LineageDiverseDisagree => "lineage_diverse_disagree",
        }
    }
}

/// H-45 payload for [`SsbActionType::PromotionScoreAggregated`].
///
/// Emitted from `darwin_admit_loop::run_one_generation` at the promotion gate
/// call site for every scored candidate under `cfg(feature = "darwinian-evolve")`
/// and `ELAI_DARWINIAN_LOOP_ENABLE=1`.
///
/// Off-agent artifact (R4): written at the guard chain call site, NOT by the
/// candidate whose config is being evaluated. Missing rows = FAIL (R1).
///
/// The `worst_case` field is the value that `two_gate_promote_aggregate` compares
/// to `tau`. A self-mod loop that replaces `worst_case` with `mean` as the margin
/// signal restores the overoptimization path (VD-3 stub_ban).
///
/// `tau` is the PRMBench-calibrated locked constant (arXiv:2501.03124).
/// `calibrated_at` is the locked date string (TD-2 freeze).
///
/// GD-2 reproducibility: `{worst_case, variance, mean, disagreement}` recompute
/// byte-identically under the same `seed` and verdicts.
/// TOTAL_VARIANTS 533 -> 534.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PromotionScoreAggregatedPayload {
    /// Which arm of the aggregation fired.
    pub kind: PromotionScoreAggregatedKind,
    /// Worst-case / variance-penalized aggregate (WCO + UWO; Coste 2310.02743).
    /// This is the margin `two_gate_promote_aggregate` compares to `tau`.
    pub worst_case: f64,
    /// Sample variance across the lineage verdicts.
    pub variance: f64,
    /// Arithmetic mean of the verdicts (NOT the promotion margin — VD-3).
    pub mean: f64,
    /// Spread (max − min) across family verdicts; disagreement diagnostic.
    pub disagreement: f64,
    /// The `tau` value used in the promotion gate at the time of scoring.
    /// PRMBench-calibrated + locked (arXiv:2501.03124).
    pub tau: f64,
    /// ISO-8601 date string when `tau` was last calibrated against PRMBench.
    /// LOCKED — a self-mod loop must not update this field.
    pub calibrated_at: String,
    /// ELAI_SEED value at the time of evaluation (GD-2 reproducibility).
    pub seed: u64,
}

/// H-46 — Kind discriminator for [`SsbActionType::CladeSelected`].
///
/// - `CladeMetaproductivity`: CMP override fired; the selected node had
///   `clade_size >= 1` measured descendants and CMP is the load-bearing signal
///   (HGM 2510.21614 Metaproductivity-Performance Mismatch).
/// - `ImmediateScoreFallback`: the selected node has no measured descendants
///   (`clade_size == 0`); selection fell back to the immediate sigmoid score
///   (cold-start guard — CMP only overrides once a lineage exists).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CladeSelectedKind {
    /// CMP aggregate fired; clade_size >= 1; load-bearing signal is the
    /// aggregated execution-grounded descendant delta.
    CladeMetaproductivity,
    /// No measured descendants; fallback to immediate sigmoid score.
    ImmediateScoreFallback,
}

impl CladeSelectedKind {
    /// Stable snake_case wire string for the SSB kind arm.
    #[must_use]
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::CladeMetaproductivity => "clade_metaproductivity",
            Self::ImmediateScoreFallback => "immediate_score_fallback",
        }
    }
}

/// H-46 payload for [`SsbActionType::CladeSelected`].
///
/// Emitted from `config_archive::select_parent_with_cmp` on the REAL loop
/// selection step (`darwin_admit_loop::run_one_generation` -> `select_parent` ->
/// CMP key) for every parent selection under `cfg(feature = "darwinian-evolve")`
/// and `ELAI_DARWINIAN_LOOP_ENABLE=1`.
///
/// Off-agent artifact (R4): written at the selection call site, NOT by the
/// candidate being evaluated. Missing rows = FAIL (R1).
///
/// VD-1: `cmp` aggregates execution-grounded descendant deltas (H-43 signal),
/// NEVER byte-hash values. A self-mod loop replacing `cmp` with a byte-hash
/// restores the GAP-1 noise path — that is the `ImmediateScoreFallback` stub-ban.
///
/// GD-2 reproducibility: `{cmp, clade_size, immediate_score}` recompute
/// byte-identically under the same `seed` and `fitness_deltas`.
/// TOTAL_VARIANTS 534 -> 535.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CladeSelectedPayload {
    /// Which arm fired.
    pub kind: CladeSelectedKind,
    /// Stable id of the selected parent variant.
    pub node_id: String,
    /// Clade-Metaproductivity aggregate (mean execution-grounded delta of
    /// direct descendants). Equals `immediate_score` on `ImmediateScoreFallback`.
    pub cmp: f64,
    /// Count of descendants that contributed a measured delta to `cmp`.
    /// Zero on `ImmediateScoreFallback`.
    pub clade_size: usize,
    /// Sigmoid score of the selected node's own `alpha` (retained for audit).
    pub immediate_score: f64,
    /// Snake_case wire string of the selection key used
    /// (`"clade_metaproductivity"` or `"immediate_score_fallback"`).
    pub key_used: String,
    /// ELAI_SEED at the time of selection (GD-2 reproducibility).
    pub seed: u64,
}

/// H-47 — Kind discriminator for [`SsbActionType::EvalFirewallEvent`].
///
/// - `PreUpdateItemRejected`: a held-out item's date precedes the last config
///   update (LiveCodeBench 2403.07974 temporal gate); the item is rejected to
///   prevent scoring on memorized content.
/// - `TemplateRegenerated`: a templated eval item was regenerated fresh for this
///   generation (GSM-Symbolic 2410.05229 score-time instance generation).
/// - `IndistinguishabilityProbe`: the eval-awareness distinguishability measurement
///   (VD-5 sandbagging risk surface — high distinguishability = model may detect eval).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EvalFirewallEventKind {
    /// Held-out item rejected: date precedes the last ConfigVariant.created_at.
    PreUpdateItemRejected,
    /// Templated task regenerated fresh via blake3-seeded generation.
    TemplateRegenerated,
    /// Eval-awareness probe measurement (Jaccard distinguishability score).
    IndistinguishabilityProbe,
}

impl EvalFirewallEventKind {
    /// Stable snake_case wire string for the SSB kind arm.
    #[must_use]
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::PreUpdateItemRejected => "pre_update_item_rejected",
            Self::TemplateRegenerated => "template_regenerated",
            Self::IndistinguishabilityProbe => "indistinguishability_probe",
        }
    }
}

/// H-47 payload for [`SsbActionType::EvalFirewallEvent`].
///
/// Emitted ONCE per generation from `darwin_admit_loop::run_one_generation`
/// after the per-item eval firewall pass closes. The `kind` field names the
/// dominant arm that fired for the generation; all counter fields are aggregates.
///
/// Budget queries REUSE `HoldoutBudgetEvent` (GD-1: no second budget variant).
/// The `budget_remaining` field mirrors the ledger's remaining count from the
/// last `query_holdout` outcome in this generation.
///
/// VD-5 honest residue: `distinguishability` near 0.0 reduces sandbagging risk
/// but cannot eliminate it — a strong model can detect evals from semantic cues
/// not captured by n-gram similarity.
///
/// GD-2 reproducibility: `{items_rejected, regen_count}` recompute
/// byte-identically under the same `seed` and `generation_id`.
/// TOTAL_VARIANTS 535 -> 536.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EvalFirewallPayload {
    /// Which arm fired (the dominant event for this generation).
    pub kind: EvalFirewallEventKind,
    /// Count of held-out items that passed the date-gate and were scored.
    pub items_scored: u32,
    /// Count of items rejected by the date-gate (PreUpdateItemRejected).
    pub items_rejected: u32,
    /// Count of items for which template-regen produced a fresh variant.
    pub regen_count: u32,
    /// Remaining Thresholdout budget after this generation (from last query).
    /// Saturates at `i64::MAX` on no-query generations.
    pub budget_remaining: i64,
    /// Jaccard distinguishability score from the indistinguishability probe.
    /// 0.0 = indistinguishable from workload; 1.0 = fully distinct (VD-5 risk).
    pub distinguishability: f64,
    /// ELAI_SEED for this generation (GD-2 reproducibility).
    pub seed: u64,
    /// Loop generation counter (0-indexed).
    pub generation_id: u64,
}

/// H-49 — Kind discriminator for [`SsbActionType::EditHistoryEvent`].
///
/// - `SnapshotCaptured`: a new CAS blob was written for the pre-edit content.
/// - `Deduped`: the pre-edit content was already in the CAS store; no new blob written.
/// - `Redo`: HEAD moved forward to a child node (redo operation).
/// - `Branched`: an edit after an undo created a new sibling child (fork); the prior
///   redo timeline was preserved (RV-3: never destroy a timeline).
/// - `Jumped`: HEAD jumped to an arbitrary labeled node (`elai jump`).
/// - `TooLargeSkipped`: file exceeded the per-blob size cap; content NOT stored
///   (R1: the event is emitted so the caller knows — never silently pretends captured).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EditHistoryEventKind {
    /// New CAS blob written for the pre-edit bytes.
    SnapshotCaptured,
    /// Pre-edit bytes already in CAS store; zero new bytes written.
    Deduped,
    /// HEAD moved forward to an existing child (redo).
    Redo,
    /// An edit after undo created a new sibling; prior redo timeline preserved.
    Branched,
    /// HEAD jumped to an arbitrary labeled node.
    Jumped,
    /// File exceeded the per-blob size cap; content skipped (R1 transparency).
    TooLargeSkipped,
}

impl EditHistoryEventKind {
    /// Stable snake_case wire string for the SSB kind arm.
    #[must_use]
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::SnapshotCaptured => "snapshot_captured",
            Self::Deduped => "deduped",
            Self::Redo => "redo",
            Self::Branched => "branched",
            Self::Jumped => "jumped",
            Self::TooLargeSkipped => "too_large_skipped",
        }
    }
}

/// H-49 payload for [`SsbActionType::EditHistoryEvent`].
///
/// Emitted from `task_runner.rs` on every file pre-edit capture (off-agent
/// artifact, R4) and by `timetravel_cmd.rs` for redo/branch/jump operations.
///
/// The `pre_hash` and `post_hash` fields are blake3 hex strings (64 chars).
/// `grain` is the user-selected granularity ("edit" | "turn" | "babystep").
/// A no-op operation (HEAD already at target) emits ZERO rows (the smoke
/// manifest distinguishes no-op from a dead path per the wiring contract).
///
/// GD-2 reproducibility: `{node_id, pre_hash, post_hash}` are deterministic
/// under the same file content and session state.
/// TOTAL_VARIANTS 536 -> 537.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EditHistoryEventPayload {
    /// Which arm fired for this event.
    pub kind: EditHistoryEventKind,
    /// Undo-tree node id (typically the snap_id of the created/moved-to snapshot).
    pub node_id: String,
    /// Parent node id (None for root captures).
    pub parent_id: Option<String>,
    /// Number of file paths captured / restored in this operation.
    pub paths_n: usize,
    /// blake3 hex hash of the pre-edit content (SnapshotCaptured / Deduped / Redo).
    pub pre_hash: Option<String>,
    /// blake3 hex hash of the post-edit content (Redo / Branched / Jumped).
    pub post_hash: Option<String>,
    /// User-selected grain: "edit" | "turn" | "babystep".
    pub grain: String,
}

/// H-51 — Kind discriminator for [`SsbActionType::HandoffLifecycleEvent`].
///
/// - `DigestWritten`: the sleep-time grounded digest was authored and written to
///   `workflow/handoffs/{slug}.md`. Decisions sourced from SSB intervention rows;
///   files sourced from SSB FileWrite rows (NOT LLM recollection — stub_ban R4).
/// - `ResumeOffered`: a fresh REPL detected an open topic and surfaced the one-line
///   offer ("prior session on <slug> at <fsm_cursor>, N open follow-ups — resume?").
/// - `ResumeConfirmed`: the user confirmed resume with 'y'. Tiered load begins.
/// - `TieredLoad`: the tiered load completed. `loaded_tokens` carries the measured
///   token count; must be <= 5000 for the wiring done-gate to PASS.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum HandoffLifecycleEventKind {
    /// Grounded digest authored and written to workflow/handoffs/{slug}.md.
    DigestWritten,
    /// Fresh REPL detected an open topic and surfaced the resume offer.
    ResumeOffered,
    /// User confirmed the resume offer. Tiered load begins.
    ResumeConfirmed,
    /// Tiered load completed. `loaded_tokens` <= 5000 = PASS.
    TieredLoad,
}

impl HandoffLifecycleEventKind {
    /// Stable snake_case wire string for the SSB kind arm.
    #[must_use]
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::DigestWritten => "digest_written",
            Self::ResumeOffered => "resume_offered",
            Self::ResumeConfirmed => "resume_confirmed",
            Self::TieredLoad => "tiered_load",
        }
    }
}

/// H-51 payload for [`SsbActionType::HandoffLifecycleEvent`].
///
/// Emitted from `handoff_digest::author_topic_digest` at session end (Drop hook),
/// from the fresh-REPL auto-detect path (ResumeOffered), and from the tiered-load
/// confirm path in `handoff.rs` (ResumeConfirmed + TieredLoad).
///
/// `digest_path` carries the resolved path of the written/loaded digest file.
/// `loaded_tokens` is set only on `TieredLoad` (measured token count; must be
/// <= 5000 for the wiring smoke gate to PASS per WIRING_CONTRACT).
/// `session_id` is the SSB session that generated this event.
///
/// GD-2: resume token cost recomputable from the journal row under ELAI_SEED.
/// TOTAL_VARIANTS 537 -> 538.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct HandoffLifecyclePayload {
    /// Which arm fired.
    pub kind: HandoffLifecycleEventKind,
    /// Stable topic slug (e.g. "main-feature" — the problem, NOT the session id).
    pub topic_slug: String,
    /// Resolved path of the digest file (workflow/handoffs/{slug}.md).
    pub digest_path: Option<String>,
    /// Measured loaded-token count (TieredLoad arm only; None otherwise).
    pub loaded_tokens: Option<u32>,
    /// SSB session_id that generated this event.
    pub session_id: String,
}

/// H-52 — Kind discriminator for [`SsbActionType::FindingAnchored`].
///
/// - `Anchored`: snippet found exactly once in the diff's new side (or uniquely on
///   the removed side when no new-side match). `resolved_hunk_id` + `new_line_start`
///   are populated. The model-emitted line number is STRUCTURALLY ABSENT from the
///   finding schema; only deterministic code sets this.
/// - `FileFallback`: snippet is empty / hunk body unavailable; anchor at file level.
///   `resolved_hunk_id` and `new_line_start` are None. DSR counts as needs_attention.
/// - `Ambiguous`: snippet occurs more than once in the search scope; anchoring to a
///   specific line would be a guess. Falls back to file level. `candidate_match_n > 1`.
/// - `Unanchorable`: snippet not found anywhere in the diff after normalization.
///   Falls back to file level. `candidate_match_n == 0`.
/// - `Relocated`: RESERVED for the Tier-2 LLM re-location retry path (DEFERRED).
///   Never produced by the current Tier-1 resolver.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FindingAnchoredKind {
    /// Unique match found; `resolved_hunk_id` + `new_line_start` populated.
    Anchored,
    /// Empty snippet or empty hunk body; file-level fallback.
    FileFallback,
    /// Multiple matches in scope; first-occurrence-wins guess suppressed.
    Ambiguous,
    /// Snippet absent from the diff after normalization.
    Unanchorable,
    /// RESERVED: Tier-2 LLM re-location retry (DEFERRED).
    Relocated,
}

impl FindingAnchoredKind {
    /// Stable snake_case wire string for the SSB kind arm.
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Anchored => "anchored",
            Self::FileFallback => "file_fallback",
            Self::Ambiguous => "ambiguous",
            Self::Unanchorable => "unanchorable",
            Self::Relocated => "relocated",
        }
    }
}

/// H-52 payload for [`SsbActionType::FindingAnchored`].
///
/// Emitted by `review_cmd::open_score_and_emit` (the `elai review --cockpit`
/// production entrypoint) for each finding anchoring attempt.
///
/// `snippet_blake3` is the blake3 hex digest of the normalized snippet bytes
/// (GD-2: verifiable without re-running the anchor; anti-stale — blake3 only,
/// never blake2/sha2 per CLAUDE.md HARD RULE Anti-Stale-Training).
///
/// `resolved_hunk_id` is the stable hunk identifier (`<path>#<old>-<new>`) from
/// `crate::review_surface::hunk_id_for` — the SAME hunk-id space as
/// `C-7 ChangeDecisionPayload.hunk_id` (closes G-5 OQ#2).
///
/// `new_line_start` is the 1-based first line on the new side of the matched
/// run. None when the anchor is file-level (FileFallback / Ambiguous /
/// Unanchorable).
///
/// `finding_id` is a caller-supplied stable identifier for the finding (e.g.
/// `"<session_id>/<file_path>/<FindingKind>"`) so the dashboard can join rows
/// without a seq round-trip.
///
/// GD-2: the anchor kind + hunk_id re-computes byte-identically from the
/// snippet + the diff under `ELAI_SEED`.
/// TOTAL_VARIANTS 538 -> 539.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FindingAnchoredPayload {
    /// Stable caller-supplied finding identifier.
    pub finding_id: String,
    /// blake3 hex digest of the normalized snippet (GD-2 verifiability; blake3
    /// per CLAUDE.md anti-stale rule — never blake2/sha2).
    pub snippet_blake3: String,
    /// Resolved hunk id (C-7 space), or None for file-level anchors.
    pub resolved_hunk_id: Option<String>,
    /// 1-based first line on the new side of the matched run. None for file-level.
    pub new_line_start: Option<u32>,
    /// Why the anchor was (or was not) resolved.
    pub kind: FindingAnchoredKind,
    /// Number of normalized matches found. 0 = Unanchorable, 1 = Anchored,
    /// >1 = Ambiguous (before tiebreak resolution).
    pub candidate_match_n: u32,
}

/// H-53 — Kind discriminator for [`SsbActionType::CitationAnchorsBound`].
///
/// - `Bound`: the evidence set was frozen and the pre-binding stage ran.
///   All `[N]` markers in the answer must be drawn from the pre-bound id space.
/// - `Blocked`: `ELAI_CITATION_PREBIND_DISABLE` is set; the legacy detect-after
///   path is active. This arm fires in the broken-arm gate smoke test to prove
///   the guard was bypassed (R2: a gate that stays "right" when the disable is set
///   is itself a FAIL).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CitationAnchorsBoundKind {
    /// Evidence set frozen; pre-binding stage ran.
    Bound,
    /// ELAI_CITATION_PREBIND_DISABLE set; legacy path active.
    Blocked,
}

impl CitationAnchorsBoundKind {
    /// Stable snake_case wire string for SSB telemetry.
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Bound => "bound",
            Self::Blocked => "blocked",
        }
    }
}

/// H-53 payload for [`SsbActionType::CitationAnchorsBound`].
///
/// Emitted by `prebind_cmd::run_demo_prebind` (the `elai prebind --demo-prebind`
/// REAL CLI entrypoint) and by the /research|/ask answer path AFTER
/// `citation_prebind::freeze_anchors` runs.
///
/// `blake3_set_hash_hex` is the blake3 hex digest of the concatenation of all
/// per-anchor `excerpt_blake3` values in id order (GD-2: verifiable without
/// re-fetching evidence; anti-stale — blake3 only, never blake2/sha2 per
/// CLAUDE.md HARD RULE Anti-Stale-Training).
///
/// `seed` is the `ELAI_SEED` value active when the set was frozen (0 when
/// the env var was absent). Carried for GD-2 reproducibility.
/// TOTAL_VARIANTS 539 -> 540.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CitationAnchorsBoundPayload {
    /// Which arm fired.
    pub kind: CitationAnchorsBoundKind,
    /// Number of anchors in the pre-bound set. 0 when Blocked.
    pub anchor_count: u32,
    /// ELAI_SEED value at freeze time (0 when absent).
    pub seed: u64,
    /// blake3 hex digest of the concatenation of all per-anchor excerpt_blake3
    /// values in id order (set fingerprint; GD-2 verifiability; blake3 only).
    pub blake3_set_hash_hex: String,
}

/// I-39 payload for [`SsbActionType::PlanStateDerived`].
///
/// Emitted by `plan_status_cmd::run_plan_status` (REAL CLI entrypoint:
/// `elai plan status [--letter <L>] [--json]`). Flat beacon; no inner
/// Kind enum. All three evidence sources are represented so off-agent
/// verification is possible (GD-2). Output is byte-identical from the
/// same inputs (deterministic). Default-ON; no kill-switch needed (pure
/// read path). TOTAL_VARIANTS 540 -> 541.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PlanStateDerivedPayload {
    /// Plan letter (e.g. "B", "C", "H"). Empty string when deriving all letters.
    pub letter: String,
    /// Session ID of the plan_status_cmd invocation.
    pub session_id: String,
    /// Derived state string. One of: "SHIPPED_WIRED", "SHIPPED_PARTIAL",
    /// "WIRE_DARK", "IN_PROGRESS", "PLANNED", "UNKNOWN".
    pub derived_state: String,
    /// Evidence anchor: human-readable pointer to the source that drove
    /// the final verdict (e.g. "fragments:s-plan-audit-2026-06-04:B-10",
    /// "wiring_check:PASS", "ssb_journal:0_observable_effect_rows"). Used
    /// for off-agent verification (GD-2). blake3 only for hashes.
    pub evidence_anchor: String,
    /// Wall-clock milliseconds since UNIX epoch at invocation time.
    pub ts_ms: i64,
}

/// I-45 — Kind discriminator for [`SsbActionType::ObjectiveSealed`].
///
/// - `Sealed`: the blake3 objective hash was stamped into the [`RunArtifact`]
///   at `build_rra` time. The receipt is fresh.
/// - `MutationDetected`: at verify time, the re-derived hash from the presented
///   task text MISMATCHES the sealed hash. The objective was redefined after
///   PASS. Hard-FAIL verify verdict.
/// - `ReceiptStale`: the attestation predicate's `objective_hash` mismatches
///   the presented task text. Hard-FAIL verify verdict.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ObjectiveSealedKind {
    /// Objective hash stamped into the RunArtifact at seal time.
    Sealed,
    /// Hash mismatch at verify time: objective redefined after PASS.
    MutationDetected,
    /// Attestation predicate hash mismatch at verify time.
    ReceiptStale,
}

impl ObjectiveSealedKind {
    /// Stable snake_case wire string.
    #[must_use]
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Sealed => "sealed",
            Self::MutationDetected => "mutation_detected",
            Self::ReceiptStale => "receipt_stale",
        }
    }
}

/// I-45 payload for [`SsbActionType::ObjectiveSealed`].
///
/// Emitted from `rra::build_rra` (Sealed arm) and from
/// `rra::verify_artifact_with_objective` (MutationDetected arm).
///
/// `objective_hash` is the 16-hex blake3 prefix sealed into the receipt.
/// `current_hash` is the re-derived hash at verify time (None on Sealed arm).
/// `session_id` is the session that generated this event.
/// `ts_ms` is the wall-clock unix-ms at emit time.
/// TOTAL_VARIANTS 541 -> 542.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ObjectiveSealedPayload {
    /// Which arm fired.
    pub kind: ObjectiveSealedKind,
    /// 16-hex blake3 prefix of blake3(canonical_objective_bytes) sealed into the receipt.
    pub objective_hash: String,
    /// Re-derived hash at verify time. None when kind == Sealed.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub current_hash: Option<String>,
    /// SSB session_id that generated this event.
    pub session_id: String,
    /// Wall-clock unix-ms at emit time.
    pub ts_ms: i64,
}

/// I-34 — Kind discriminator for [`SsbActionType::GrammarGateEvent`].
///
/// Pattern-2 single-variant 3-arm. SCHEMA_VERSION 233 -> 234.
/// - `CranePassed`: constrained decode completed, gate eligibility check passed.
/// - `PinViolation`: grammar file's blake3 digest mismatches the pinned manifest value.
/// - `PostParseRejected`: post-parse re-validation detected silent non-compliance (landmine #7).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum GrammarGateEventKind {
    /// Constrained decode completed successfully; gate eligibility passed.
    CranePassed,
    /// Grammar file's blake3 digest mismatches the pinned manifest value (CDA landmine #8).
    PinViolation,
    /// Post-parse schema re-validation detected a silently non-compliant output (landmine #7).
    PostParseRejected,
}

impl GrammarGateEventKind {
    /// Stable snake_case wire string.
    #[must_use]
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::CranePassed => "crane_passed",
            Self::PinViolation => "pin_violation",
            Self::PostParseRejected => "post_parse_rejected",
        }
    }
}

/// I-34 payload for [`SsbActionType::GrammarGateEvent`].
///
/// Emitted from `orchestrator::grammar_gate::emit_grammar_gate_event`.
/// `grammar_path` is the EBNF grammar path that triggered the event.
/// `pin_violation_expected` / `pin_violation_actual` carry the pinned vs actual
/// blake3 hex on a PinViolation arm (None on other arms).
/// `session_id` is the session that generated this event.
/// `ts_ms` is the wall-clock unix-ms at emit time.
/// TOTAL_VARIANTS 542 -> 543.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GrammarGateEventPayload {
    /// Which arm fired.
    pub kind: GrammarGateEventKind,
    /// Path to the EBNF grammar file involved in this event.
    pub grammar_path: String,
    /// Pinned blake3 hex at pin time (PinViolation arm only).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub pin_violation_expected: Option<String>,
    /// Freshly-computed blake3 hex at verify time (PinViolation arm only).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub pin_violation_actual: Option<String>,
    /// SSB session_id that generated this event.
    pub session_id: String,
    /// Wall-clock unix-ms at emit time.
    pub ts_ms: i64,
}

/// D-27.1 payload for [`SsbActionType::DiscoveryRatified`].
///
/// Emitted by the `ratify_discovery` Tauri command when a human reviewer
/// marks a `discovery_candidate` novelty-classified template as ratified
/// for adversarial verification. `reviewer_id` must be non-empty (enforced
/// at the command layer before this payload is constructed).
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct DiscoveryRatifiedPayload {
    /// ID of the template being ratified (mirrors `NoveltyClassifiedPayload.template_id`).
    pub template_id: String,
    /// Non-empty human-readable reviewer identifier (e.g. session alias or user handle).
    pub reviewer_id: String,
    /// Wall-clock milliseconds since UNIX epoch at the moment the command was invoked.
    pub ts_ms: i64,
}

/// D-27.1 payload for [`SsbActionType::DiscoveryRejected`].
///
/// Emitted by the `reject_discovery` Tauri command when a human reviewer
/// dismisses a `discovery_candidate` novelty-classified template. Both
/// `reviewer_id` and `reason` must be non-empty (enforced at the command
/// layer before this payload is constructed).
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct DiscoveryRejectedPayload {
    /// ID of the template being rejected.
    pub template_id: String,
    /// Non-empty human-readable reviewer identifier.
    pub reviewer_id: String,
    /// Non-empty human-readable reason for rejection (free-form text, max 512 chars
    /// by convention — the command layer may truncate before constructing this payload).
    pub reason: String,
    /// Wall-clock milliseconds since UNIX epoch at the moment the command was invoked.
    pub ts_ms: i64,
}

// === I-24 (SCHEMA_VERSION 266) -- PrmCanaryEvent substrate ===================

/// I-24 (SCHEMA_VERSION 266) -- discriminator for [`SsbActionType::PrmCanaryEvent`].
///
/// Scorer-side anti-gaming canary: isomorphic code pairs are injected into the
/// L2 scoring path; a scorer that rates semantically-identical variants differently
/// is style-biased and (after N consecutive detections) neutralized.
///
/// R8 compliance: the canary pair set is FROZEN in `fixtures/canary_pairs.json`;
/// the running system NEVER generates new canaries from the model under test.
///
/// Emitted from `crate::canary_injector::emit_prm_canary_to_ssb` at the
/// `TriLayerStack::run_verdict` L2 injection site.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PrmCanaryEventKind {
    /// A canary pair was injected and the score delta was below the bias
    /// threshold; no style bias detected on this injection.
    CanaryInjected,
    /// The score delta between the isomorphic pair exceeded the ELAI-measured
    /// threshold (`metrics/canary_bias_threshold.toml`, ID-5); the scorer is
    /// flagged as style-biased for this detection.
    StyleBiasDetected,
    /// N consecutive `StyleBiasDetected` events within one run caused the scorer
    /// weight to be zeroed for the remainder of the run. ID-4: this is an L2-only
    /// effect; it CANNOT override an L1 HardVeto (`VETO_SCORE = 0.0` stands).
    ScorerNeutralized,
}

impl PrmCanaryEventKind {
    /// SQL-compatible string for the CHECK constraint and SSB payload `kind` field.
    #[must_use]
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::CanaryInjected    => "prm_canary_injected",
            Self::StyleBiasDetected => "prm_canary_style_bias_detected",
            Self::ScorerNeutralized => "prm_canary_scorer_neutralized",
        }
    }

    /// Alias required by `SsbActionType::as_sql_str()` dispatch convention.
    /// Returns the same string as [`as_str`].
    #[must_use]
    pub fn as_sql_str(&self) -> &'static str {
        self.as_str()
    }
}

/// I-24 (SCHEMA_VERSION 266) -- payload for [`SsbActionType::PrmCanaryEvent`].
///
/// Carries the canary injection result: which pair was selected, the scores for
/// both variants, the computed delta, the bias threshold, and (on ScorerNeutralized)
/// the consecutive-detection count that triggered neutralization.
///
/// `score_original` and `score_obfuscated` are in `[0.0, 1.0]` (the L2 calibrated
/// score range). `delta_bp` and `threshold_bp` are in basis points (1/10000) so
/// small differences remain representable as integers. `delta_bp` is ALWAYS
/// non-negative (`|score_original - score_obfuscated| * 10_000` rounded).
///
/// ID-4 invariant: `ScorerNeutralized` zeroes the scorer WEIGHT for the run; it
/// does NOT modify any L1-veto decision or resurrect a HardVeto branch.
///
/// GD-2: results are recomputable under the same `ELAI_SEED` + frozen fixture set.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct PrmCanaryEventPayload {
    /// Fire-point discriminator. See [`PrmCanaryEventKind`].
    pub kind: PrmCanaryEventKind,
    /// Opaque scorer identifier (e.g. `"conformal_tau"` for the current L2 stub,
    /// `"ppm_client"` when the neural scorer is wired in post-D-026).
    pub scorer_id: String,
    /// Index of the selected canary pair within `fixtures/canary_pairs.json`.
    pub pair_index: u32,
    /// L2 score of the original (unobfuscated) variant, in `[0.0, 1.0]`.
    pub score_original: f64,
    /// L2 score of the isomorphic (AST-preserving obfuscated) variant, in `[0.0, 1.0]`.
    pub score_obfuscated: f64,
    /// Absolute score delta in basis points: `round(|score_original - score_obfuscated| * 10_000)`.
    pub delta_bp: u32,
    /// Bias detection threshold in basis points (from `metrics/canary_bias_threshold.toml`,
    /// ID-5: ELAI-measured, never a paper-derived constant).
    pub threshold_bp: u32,
    /// Number of consecutive `StyleBiasDetected` events in this run at the time of
    /// emission. `0` on `CanaryInjected` (no bias), `N >= 1` on `StyleBiasDetected`,
    /// equals the neutralization threshold on `ScorerNeutralized`.
    pub consecutive_count: u32,
    /// Wall-clock milliseconds since UNIX epoch at the moment of emission.
    pub ts_ms: i64,
}

/// I-24 (SCHEMA_VERSION 266) -- env var name gating `PrmCanaryEvent` emission
/// and the `CanaryInjector::maybe_inject` hook inside `TriLayerStack::run_verdict`.
/// Default-OFF (R9 experiment): the canary injector adds a second L2 scoring call
/// per sampled branch; ships behind a measured-first-win gate. Promotion plan:
/// `I-24-promotion` (FORWARD). See `wire-dark-allowlist.toml`.
pub const ELAI_CANARY_INJECTOR_ENABLE_ENV: &str = "ELAI_CANARY_INJECTOR_ENABLE";

// === I-27 (SCHEMA_VERSION 265) -- ConvergenceCurveMeasured substrate =========

/// I-27 (SCHEMA_VERSION 265) -- one compute-level point on the convergence curve.
///
/// `compute_level` is an opaque label (e.g. "1x", "2x", "4x", "8x") naming the
/// compute envelope (task_count * seeds multiplier, or a cost-cents ceiling).
/// `mean_rate` is the fraction of tasks that produced a verified outcome at this
/// level, averaged over all seeds. `ci_low`/`ci_high` are the seeded bootstrap
/// 95% CI bounds (GD-2: recomputable under `ELAI_SEED`).
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct ConvergenceCurvePoint {
    /// Human-readable compute level label (e.g. "1x", "2x", "4x", "8x").
    pub compute_level: String,
    /// Mean verified-outcome rate over all seeds at this compute level, in [0.0, 1.0].
    pub mean_rate: f64,
    /// Seeded bootstrap 95% CI lower bound (GD-2: recomputable under ELAI_SEED).
    pub ci_low: f64,
    /// Seeded bootstrap 95% CI upper bound (GD-2: recomputable under ELAI_SEED).
    pub ci_high: f64,
    /// Cost per additional verified task at this level (None if cost data absent).
    /// Pareto view: cost_per_verified_task = dollar_per_task / mean_rate when mean_rate > 0.
    pub cost_per_verified_task: Option<f64>,
}

/// I-27 (SCHEMA_VERSION 265) -- payload for [`SsbActionType::ConvergenceCurveMeasured`].
///
/// Flat beacon (no Kind discriminator). ONE row per `elai bench convergence` run,
/// carrying the full curve inline. The payload is bounded: at Tier-1 fixture leg,
/// task_count and seed_count are both small.
///
/// GD-2: `curve_points` recompute byte-identically from the fixture under `ELAI_SEED`.
/// GD-5: `non_monotonic_at` names every compute level where the rate dipped; NOTHING
/// is smoothed. The instrument is the honest signal for the search bet (NOT a promise
/// of monotonicity beyond the measured sample).
///
/// Composes E-49 (cost_per_task_cents denominator), I-38 (bootstrap CI + seed floor),
/// H-9 (cost cockpit family reads this row for the convergence tile) (GD-1).
///
/// WIRE-DARK[I-27]: the live per-node compute-spend axis sourced from I-4 SearchNodeEvent
/// rows is PENDING-on-I-4 (search/ladder.rs UNBUILT at HEAD). The Tier-1 fixture leg
/// uses cost_per_task_cents as the cost axis and task_count*seeds as the compute proxy.
/// When I-4 ships, the compute_level field will carry per-node spend from SearchNodeEvent;
/// the payload shape is forward-compatible (compute_level is an opaque label).
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct ConvergenceCurveMeasuredPayload {
    /// Fixture suite label (e.g. "convergence_fixture_v1").
    pub fixture_suite: String,
    /// Number of independent seeds used per compute level (floor >= 5, I-38).
    pub seed_count: u32,
    /// Number of tasks in the fixture.
    pub task_count: u32,
    /// The convergence curve: one point per compute level, ordered by compute_level.
    pub curve_points: Vec<ConvergenceCurvePoint>,
    /// The compute level at or before which the marginal gain collapses (the "knee").
    /// Set to the last level's label if no knee is detected.
    pub knee_compute_level: String,
    /// Compute levels where the verified-outcome rate DIPPED compared to the prior level.
    /// Empty if the curve is monotonically non-decreasing (GD-5: report, never smooth).
    pub non_monotonic_at: Vec<String>,
    /// ELAI_SEED used for the bootstrap CI resampling (GD-2: recomputable).
    pub seed: u64,
    /// blake3 hex of the fixture bytes (provenance pin; anti-gaming, GD-2).
    pub fixture_blake3: String,
    /// `true` = Tier-1 fixture only; live per-node leg pending I-4 (D-026).
    pub runpod_pending: bool,
    /// Wall-clock milliseconds since UNIX epoch at the moment of emission.
    pub ts_ms: i64,
}

// ---------------------------------------------------------------------------
// I-36 (SCHEMA_VERSION 267): PromptCompiled substrate
// ---------------------------------------------------------------------------

/// I-36 (SCHEMA_VERSION 267): 3-arm discriminator for [`SsbPayload::PromptCompiled`].
///
/// Pattern-2 single-variant, 3 distinct sql_str values (per-Kind convention).
/// Alphabetical arm order maintained per codebase convention.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub enum PromptCompiledKind {
    /// A promoted compiled variant was substituted at the adapter boundary. The payload
    /// names the variant hash + backend. Emitted when `CompiledPromptStore::lookup` returns
    /// `Some` and the string is placed into `CompletionConfig::compiled_system_override`.
    Compiled,
    /// Source-vs-compiled fixture pass-rate delta was measured and ledgered.
    /// Recomputable under `ELAI_SEED` (GD-2). Tier-2 DEFER (live model budget).
    TransferGapMeasured,
    /// The compiled variant regressed its fixture floor; variant removed from the store
    /// and substitution reverts to the base assembled prompt.
    Rolledback,
}

impl PromptCompiledKind {
    /// Per-Kind sql_str for the SSB CHECK constraint.
    ///
    /// Three distinct values so Compiled / TransferGapMeasured / Rolledback are
    /// distinguishable on the wire (mirrors TddPreflightBlockedKind convention).
    pub fn as_sql_str(self) -> &'static str {
        match self {
            Self::Compiled => "prompt_compiled",
            Self::TransferGapMeasured => "transfer_gap_measured",
            Self::Rolledback => "prompt_compiled_rolledback",
        }
    }
}

/// I-36: SSB payload for a compiled-prompt artifact event.
///
/// Pattern-2 single-variant; kind discriminator is [`PromptCompiledKind`].
///
/// Emitted by:
///   - the `report_fns::build_system_prompt_for_role` seam (Compiled arm) when
///     `CompiledPromptStore::lookup` returns a variant and it is placed into
///     `CompletionConfig::compiled_system_override`.
///   - `prompt_artifacts::run_transfer_gap_fixture` (TransferGapMeasured arm, Tier-2 DEFER).
///   - `prompt_artifacts::rollback_on_regression` (Rolledback arm).
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct PromptCompiledPayload {
    /// Which event occurred.
    pub kind: PromptCompiledKind,
    /// Snake-case role name (e.g. `"executor"`, `"researcher"`).
    pub role: String,
    /// Backend name (e.g. `"anthropic_api"`, `"sidecar"`).
    pub backend: String,
    /// First 16 hex chars of the blake3 variant hash (truncated for log brevity).
    /// Full hash lives in the stored `CompiledPromptVariant::variant_hash`.
    pub variant_hash: String,
    /// `compiled_pass_rate - source_pass_rate` (positive = improvement).
    /// `0.0` for `Compiled` rows (not yet measured); measured value for
    /// `TransferGapMeasured` and negative-and-below-floor for `Rolledback`.
    pub fixture_delta: f32,
    /// Wall-clock milliseconds since UNIX epoch at emit time.
    pub ts_ms: i64,
}

// ---------------------------------------------------------------------------
// I-50 (SCHEMA_VERSION 268): AuthBrokerEvent substrate
// ---------------------------------------------------------------------------

/// I-50 (SCHEMA_VERSION 268): 4-arm discriminator for [`SsbPayload::AuthBrokerEvent`].
///
/// Pattern-2 single-variant, 4 distinct sql_str values (per-Kind convention).
/// Arms ordered by lifecycle: acquisition → refresh → link → failure.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub enum AuthBrokerEventKind {
    /// A valid credential was resolved (env-key, saved OAuth token, or new OAuth exchange).
    /// No token material is logged; only `provider_id` and event timing are recorded.
    TokenAcquired,
    /// An expired saved token was refreshed and re-persisted to the credential store.
    /// Emitted only on a successful refresh; failed refresh emits `RefreshFailed`.
    Refreshed,
    /// A new provider credential was registered with the broker for the first time.
    /// Used by the `AuthBroker::register_provider` path (future multi-provider flow).
    ProviderLinked,
    /// A token-refresh attempt failed with a typed error.
    /// The error kind is recorded in `error_kind`; the error is surfaced as `ApiError`
    /// to the caller -- never silently swallowed.
    RefreshFailed,
}

impl AuthBrokerEventKind {
    /// Per-Kind sql_str for the SSB CHECK constraint.
    ///
    /// Four distinct values so each lifecycle arm is distinguishable on the wire
    /// (mirrors `TddPreflightBlockedKind` and `PromptCompiledKind` conventions).
    pub fn as_sql_str(self) -> &'static str {
        match self {
            Self::TokenAcquired => "auth_broker_token_acquired",
            Self::Refreshed => "auth_broker_refreshed",
            Self::ProviderLinked => "auth_broker_provider_linked",
            Self::RefreshFailed => "auth_broker_refresh_failed",
        }
    }
}

/// I-50: SSB payload for a subscription auth broker lifecycle event.
///
/// Pattern-2 single-variant; kind discriminator is [`AuthBrokerEventKind`].
///
/// SECURITY: this payload deliberately omits token material. `provider_id` names
/// the provider (e.g. `"anthropic"`); no access token, refresh token, or key bytes
/// are included. The `stub_ban` observable_effect from the I-50 WIRING_CONTRACT
/// is structurally enforced here: there is no field for token material.
///
/// Emitted by:
///   - `AuthBroker::resolve()` → `TokenAcquired` arm (happy path).
///   - `AuthBroker::refresh()` → `Refreshed` arm (on successful re-exchange).
///   - `AuthBroker::register_provider()` → `ProviderLinked` arm (future multi-provider).
///   - `AuthBroker::refresh()` error path → `RefreshFailed` arm.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct AuthBrokerEventPayload {
    /// Which auth broker lifecycle event occurred.
    pub kind: AuthBrokerEventKind,
    /// Opaque provider identifier (e.g. `"anthropic"`, `"aws_bedrock"` in future).
    /// Used to correlate rows; never contains token material.
    pub provider_id: String,
    /// For `RefreshFailed`: a short classifier for the error kind (e.g. `"network"`,
    /// `"token_expired_no_refresh"`, `"http_401"`). Empty string for other arms.
    pub error_kind: String,
    /// Wall-clock milliseconds since UNIX epoch at emit time.
    pub ts_ms: i64,
}
