//! Shared State Board (SSB): inter-agent communication via SQLite WAL.
//!
//! Single-writer guarantee via `tokio::mpsc` channel. All writes serialized
//! through one dedicated task. Reads served from a `deadpool-sqlite` async connection pool.
//! Append-only within a session — no UPDATE or DELETE.
//!
//! A single [`Ssb`] instance is permanently bound to one `(project_id, session_id)` pair.
//! Multi-session scenarios require separate instances.

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

// ---------------------------------------------------------------------------
// Schema version — bump on any DDL change
// ---------------------------------------------------------------------------

// B-10: bumped 21 -> 22 for SsbActionType::ContextBudgetComputed +
// SsbFilteredByOverlap. Tier-4 last bumped 20 -> 21 for ExecutionModeChanged.
// B-19A: bumped 22 -> 23 for SsbActionType::HookLatencyViolation
// (per-hook-tier T1/T2/T3 latency budget breach beacon).
// batch-ful-d: bumped 23 -> 24 for research_path_chosen + grounding_cache_event CHECK variants.
// Track B (s-1430428): bumped 24 -> 25 for mcp_server_connected / mcp_tool_called /
// mcp_server_disconnected CHECK variants. Enum + from_str + payloads already landed in
// batch-ful-d; this bump completes the CHECK constraint to match the enum surface.
// B-6 (coordinated schema bump): bumped 25 -> 26 for B-4 circuit_breaker_state_change,
// B-7 bubble_up_typed / integration_checkpoint_success / dag_repair_round /
// concept_decomposition_node, and B-6's own evidence_gate_evaluated / supervision_event /
// bsp_superstep / agent_lifecycle / anti_spiral_hit. All CHECK, enum, as_sql_str,
// from_str, payloads, and SsbPayload variants land atomically in this bump.
// DESTRUCTIVE: existing v25 session DBs are DROPped on first read (no migration logic).
// B-23 Phase 4H Group A: bumped 27 -> 28 for 6 new self-mod SSB action types
// (data_table_grew, l3_proposal, convergence_stagnant, config_oscillation,
//  regression_detected, evolution_withheld). The 5 batch-3C self-mod variants
// (proposal_*, msi_update, config_archived) already shipped in v26/v27.
// B-39 chat reflect pipeline: 4 new SsbActionType variants for per-turn REFLECT events.
// B-24 Phase 4I Group C1: bumped 29 -> 30 for 6 new media-validator SSB action types
// (media_validation_started, media_tier1_complete, media_tier2_complete,
//  render_complete, visual_feedback_iteration, media_human_review_required).
// B-28 E-3 (SCHEMA_VERSION 31): bumped 30 -> 31 for SsbActionType::GenerationModeSelected.
// Single new action_type added in this bump; the other 3 focus-tunnel beacons
// (streaming_early_terminated, pipeline_pre_assembled, dependency_resolved) land
// with their own bumps in E-4..E-7 when their emitters are wired. Co-staging the
// CHECK list without emitters would create dead schema rows the writer never
// produces — declined per B-24 4I.1 precedent.
// B-28 E-4 (SCHEMA_VERSION 32): bumped 31 -> 32 for SsbActionType::StreamingEarlyTerminated.
// Single new action_type. Emitter is the post-E-7 WIRING phase that plugs the
// library-tier StreamingValidator (focus_tunnel::streaming) into run_fim_step /
// run_diff_step; E-4 only defines the library surface + the schema bump. The
// payload variant lands now (not deferred) because the StreamingValidator's
// EarlyTermWireForm projection consumes StreamingEarlyTerminatedPayload directly
// — co-staging is appropriate here even though the live emitter is deferred.
// DESTRUCTIVE: existing v31 session DBs are DROPped on first read (no migration
// logic). Matches B-23 / B-24 / B-27 / B-28 E-3 precedent.
// B-28 WIRING-1 (SCHEMA_VERSION 33 -> 34 -> 35): batched 3-variant bump for the
// remaining focus_tunnel reader-side beacons. Three serial bumps in one atomic
// commit:
//   * 32 -> 33: SsbActionType::PipelinePreAssembled (item A.1). Emitter is the
//     post-WIRING-2 hook in drive_task that calls
//     PipelinePreAssembler::on_step_outcome + spawn prepare(). Payload struct
//     PipelinePreAssembledPayload defined below. Reader aggregator
//     pre_assembly_stats_for_payloads (pre_assembly.rs:498) wires the match arm
//     in this commit.
//   * 33 -> 34: SsbActionType::DependencyResolved (item A.1). Emitter is the
//     post-execute hook scanning changed_files via plugins::import_extractor +
//     DependencyResolver::resolve. Payload struct DependencyResolvedPayload
//     defined below. Reader aggregator dependency_resolution_stats_for_payloads
//     (dep_resolver.rs:807) wires the match arm in this commit.
//   * 34 -> 35: SsbActionType::MicroStepCreated (item A.1). Emitter is the
//     SpeculativeFileRouter::route NeedsMicroStep / MicroStepCapExhausted path.
//     Payload struct MicroStepCreatedPayload defined below. Reader aggregator
//     file_router_stats_for_payloads (file_router.rs:881) wires the match arm
//     in this commit.
// DESTRUCTIVE: existing v32 session DBs are DROPped on first read (no migration
// logic). Matches B-23 / B-24 / B-27 / B-28 E-3 / E-4 precedent.
const SCHEMA_VERSION: i64 = 35;

/// Public re-export of [`SCHEMA_VERSION`] for contract / integration tests
/// outside the orchestrator crate. The const itself stays crate-private so
/// internal callers cannot drift from the canonical value.
pub const SCHEMA_VERSION_PUBLIC: i64 = SCHEMA_VERSION;

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
        'tiered_analysis', 'shortcoder_compression',
        'tool_dispatch', 'validation_tier', 'failure_classified',
        'git_commit', 'daira_trace',
        'system_probe', 'feasibility_report', 'novel_step_detected', 'stub_run',
        'gate_decision', 'task_split', 'escalation_fired', 'mode_changed',
        'proposal_generated', 'proposal_applied', 'proposal_rejected',
        'msi_update', 'config_archived',
        'profile_selected', 'fsm_transition', 'micro_reflection',
        'telescope_collapse', 'config_warning',
        'constraints_injected', 'intervention_resolved',
        'dkg_querier_swapped',
        'steering_computed', 'steering_compute_failed',
        'chat_message', 'chat_response',
        'execution_mode_changed',
        -- B-10 (DECISION-C): per-dispatch ContextBudget snapshot + 3E.5
        -- SSB entry filtering result for the dashboard's per-slot bar chart.
        'context_budget_computed', 'ssb_filtered_by_overlap',
        -- B-19A (T10): hook latency budget violation beacon emitted by
        -- the dispatcher when a T1/T2/T3 hook overruns its tier budget.
        'hook_latency_violation',
        -- batch-ful-d (A.2/A.6): research path selection + grounding cache hit/miss.
        'research_path_chosen', 'grounding_cache_event',
        -- Track B (s-1430428): MCP bridge lifecycle + tool-call beacons.
        -- Enum variants McpServerConnected / McpToolCalled / McpServerDisconnected
        -- already declared at L283-291; from_str arms at L498-500.
        'mcp_server_connected', 'mcp_tool_called', 'mcp_server_disconnected',
        -- B-4: circuit-breaker state transition for embedding/reranker models.
        'circuit_breaker_state_change',
        -- B-7: bubble-up failure cascade, integration checkpoints, DAG repair, decomposition.
        'bubble_up_typed', 'integration_checkpoint_success',
        'dag_repair_round', 'concept_decomposition_node',
        -- B-6: orchestration producer wiring — evidence gates, supervision, BSP, agent lifecycle.
        'evidence_gate_evaluated', 'supervision_event',
        'bsp_superstep', 'agent_lifecycle', 'anti_spiral_hit',
        -- B-27 (SCHEMA_VERSION 27): cloud-mode dual-rate spend ledger row +
        -- one-shot escalation marker emitted when local exhausts retries
        -- and cloud_mode promotes the step. cloud_mode.rs is the sole writer.
        'cloud_spend', 'escalate_to_cloud',
        -- B-23 Phase 4H Group A (SCHEMA_VERSION 28): 6 new self-mod beacons.
        -- data_table_grew    -- Layer 1A row insert event (per spec phase-4H.txt:140)
        -- l3_proposal         -- Layer 3 structural-change proposal (HUMAN-ONLY review)
        -- convergence_stagnant-- OSC verdict (3 consecutive >=0.95 similarity)
        -- config_oscillation  -- OSC period-2 verdict (sim(N,N-2) AND sim(N-1,N-3))
        -- regression_detected -- REG cross-pipeline AC regression report
        -- evolution_withheld  -- EVO gate Withhold (config_blob never evolved)
        'data_table_grew', 'l3_proposal', 'convergence_stagnant',
        'config_oscillation', 'regression_detected', 'evolution_withheld',
        -- B-39 chat reflect pipeline: per-turn REFLECT stage beacons.
        'reflect_triaged', 'reflect_candidate', 'reflect_audit_verdict', 'reflect_refined',
        -- B-24 Phase 4I Group C1: media validator subsystem (Tier 1 / Tier 2 /
        -- Render-Capture-Evaluate / Visual-Feedback / HumanReview branch).
        -- Payloads ship in Group C4 alongside the dashboard wiring; the
        -- enum + SQL CHECK + as_sql_str arms are added now so the SCHEMA_VERSION
        -- bump (29 -> 30) is atomic.
        'media_validation_started', 'media_tier1_complete', 'media_tier2_complete',
        'render_complete', 'visual_feedback_iteration', 'media_human_review_required',
        -- B-28 E-3 (SCHEMA_VERSION 31): focus_tunnel ModeSelector beacon.
        -- generation_mode_selected — emitted once per baby-step decision in
        -- drive_task immediately before the executor dispatch. Carries the
        -- chosen GenerationMode plus the SelectionRule that fired, so the
        -- dashboard's mode_distribution surface and the auditor's
        -- first-match-wins check both read from one source.
        'generation_mode_selected',
        -- B-28 E-4 (SCHEMA_VERSION 32): focus_tunnel StreamingValidator
        -- early-termination beacon. Emitted by the post-E-7 WIRING phase
        -- whenever the library-tier validator returns ValidationDecision
        -- ::EarlyTerm. Carries the wire-form EarlyTermReason (lazy_pattern /
        -- unbalanced_braces / output_too_long / deprecated_import /
        -- unbounded_string_literal) plus the token count at termination so
        -- the dashboard can render an early-term row keyed by reason.
        'streaming_early_terminated',
        -- B-28 WIRING-1 (SCHEMA_VERSION 33): focus_tunnel
        -- PipelinePreAssembler readiness beacon. One row per
        -- PipelinePreAssembler::on_step_outcome -> prepare() invocation.
        -- Carries pass_rate_bp (basis points) + was_discarded + depth +
        -- bytes_assembled so the dashboard can render the parallelism gauge
        -- and waste-rate bar without aggregating across multiple rows.
        'pipeline_pre_assembled',
        -- B-28 WIRING-1 (SCHEMA_VERSION 34): focus_tunnel DependencyResolver
        -- decision beacon. Emitted once per ResolutionDecision after the
        -- post-execute import-scan + resolve cycle. Carries the resolved
        -- language + name + decision string (AutoApprove / QueueForHuman /
        -- Blocked) + the reason that gated the decision. install_command
        -- mirrors the AutoApprove ApplyPlan when present.
        'dependency_resolved',
        -- B-28 WIRING-1 (SCHEMA_VERSION 35): focus_tunnel SpeculativeFileRouter
        -- micro-step creation / cap-exhaustion beacon. Emitted once per
        -- NeedsMicroStep or MicroStepCapExhausted route decision. Carries
        -- the parent step id + created_path + type_hint (Module / Struct /
        -- Function / Const) + reason string (the gating cue) so the
        -- dashboard can render a per-step expansion trail.
        'micro_step_created'
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

/// All ~100 action types recorded in SSB entries.
/// B-19A added `HookLatencyViolation` (was 74 in B-10). The count is
/// approximate because per-batch additions land asynchronously; the
/// canonical inventory lives in [`SsbActionType::as_sql_str`] and the
/// CHECK constraint above. B-23 Phase 4H Group A is the latest tranche
/// of additions (6 self-mod beacons). B-39 adds 4 reflect variants
/// (reflect_triaged, reflect_candidate, reflect_audit_verdict, reflect_refined).
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
    /// Batch 2B: per-dispatch summary for every `InProcessToolDispatcher`
    /// call (edit / validate / classify / git_commit / file_read / incremental).
    /// Carries a [`ToolDispatchPayload`].
    ToolDispatch,
    /// Batch 2B: per-tier validation verdict row emitted once per
    /// [`elai_core::ValidationTier`] execution. Carries a
    /// [`ValidationTierPayload`].
    ValidationTier,
    /// Batch 2B: failure taxonomy classification row emitted by
    /// `dispatch_classify_failure`. Carries a [`FailureClassifiedPayload`].
    FailureClassified,
    /// Batch 2B: git2-backed commit boundary. Carries a
    /// [`GitCommitPayload`].
    GitCommit,
    /// Batch 2B: DAIRA call-graph breadcrumb. Emitted on entry/exit of
    /// every dispatch method so the activity feed can render call graphs.
    /// Carries a [`DairaTracePayload`].
    DairaTrace,
    /// Batch 3A (Phase 4F): system probe snapshot. Emitted once per task
    /// runner lifetime and replayed on every dashboard subscribe. Carries
    /// a [`SystemProbePayload`].
    SystemProbe,
    /// Batch 3A (Phase 4F): per-concept / per-task feasibility verdict.
    /// Carries a [`FeasibilityReportPayload`].
    FeasibilityReport,
    /// Batch 3A (Phase 4F): one row per novel step detected by the
    /// feasibility prober. Carries a [`NovelStepDetectedPayload`].
    NovelStepDetected,
    /// Batch 3A (Phase 4F): one row per stub execution inside the
    /// feasibility probe. Carries a [`StubRunPayload`].
    StubRun,
    /// Batch 3B (Phase 4G): autonomous-gate decision row. Emitted on every
    /// call to [`crate::autonomous_gate::AutonomousGate::decide`]. Carries a
    /// [`GateDecisionPayload`].
    GateDecision,
    /// Batch 3B (Phase 4G): task-split announcement emitted when the
    /// autonomous gate decides to partition a task under context pressure.
    /// Carries a [`TaskSplitPayload`].
    TaskSplit,
    /// Batch 3B (Phase 4G): escalation-fired row. Carries an
    /// [`EscalationFiredPayload`] with the trigger + step_id.
    EscalationFired,
    /// Batch 3B (Phase 4G): gate-mode transition (Interactive ↔ Autonomous).
    /// Carries a [`ModeChangedPayload`].
    ModeChanged,
    /// Batch 3C (Phase 4H) — Layer 1A proposal generated. Carries a
    /// [`ProposalGeneratedPayload`].
    ProposalGenerated,
    /// Batch 3C (Phase 4H) — Layer 1A proposal applied. Carries a
    /// [`ProposalAppliedPayload`].
    ProposalApplied,
    /// Batch 3C (Phase 4H) — Layer 1A proposal rejected. Carries a
    /// [`ProposalRejectedPayload`].
    ProposalRejected,
    /// Batch 3C (Phase 4H) — per-apply MSI update. Carries a
    /// [`MsiUpdatePayload`].
    MsiUpdate,
    /// Batch 3C (Phase 4H) — archive capping event. Carries a
    /// [`ConfigArchivedPayload`].
    ConfigArchived,
    /// Batch 4C (A-18) — per-dispatch GenerationProfile selection. Carries a
    /// [`ProfileSelectedPayload`].
    ProfileSelected,
    /// Batch 4C (A-22) — FSM transition emitted on RESEARCHING entry/exit.
    /// Carries an [`FsmTransitionPayload`].
    FsmTransition,
    /// Batch 4C (A-23) — micro-reflection diagnosis on baby-step failure.
    /// Carries a [`MicroReflectionPayload`].
    MicroReflection,
    /// Batch 4C (A-46) — context telescope collapse event. Carries a
    /// [`TelescopeCollapsePayload`].
    TelescopeCollapse,
    /// Batch 4C (A-45) — config-load warning (e.g. source-tiers.toml missing).
    /// Carries a [`ConfigWarningPayload`].
    ConfigWarning,
    /// Batch-fu (FU.2 / A-7) — constitutional CONSTRAINTS block injected into
    /// the assembled PhaseSpec. Emitted once per dispatch that carried a
    /// non-empty `ProjectProfile.constraints` list. Carries a
    /// [`ConstraintsInjectedPayload`].
    ConstraintsInjected,
    /// Batch-fu (FU.5 / A-31) — resolution row emitted after an
    /// `InterventionRequest` has been answered by the dashboard via
    /// `respond_to_confirmation`. Carries an [`InterventionResolvedPayload`].
    InterventionResolved,
    /// Batch-ful-a (FUL-A.1) — live upgrade of the rule engine's DkgQuerier
    /// after the first successful `IndexingService::run_full_index`. Carries
    /// a [`DkgQuerierSwappedPayload`]. Emitted once per project per runner
    /// lifetime, immediately after the structured `IndexingProgress=complete`
    /// + `DkgReady` rows. Lets the dashboard activity feed render "DKG querier
    /// upgraded · empty → adapter" without polling the rule engine.
    DkgQuerierSwapped,
    /// Batch-ful-c (FUL-C / A-8) — freshness steering vector computed for a
    /// role. Emitted on successful `/v1/steering/compute` round-trip +
    /// cache-file write. Carries a [`SteeringComputedPayload`].
    SteeringComputed,
    /// Batch-ful-c (FUL-C / A-8) — freshness steering vector compute failed
    /// for a role (sidecar unreachable, HTTP error, validation rejected, or
    /// cache write failed). Carries a [`SteeringComputeFailedPayload`].
    SteeringComputeFailed,
    /// Batch-ful-d (FUL-D / phase-3J S9) — external MCP server connection
    /// established via the `DefaultMcpBridge`. Emitted once per successful
    /// `initialize` handshake + `tools/list` response. Carries a
    /// [`McpServerConnectedPayload`].
    McpServerConnected,
    /// Batch-ful-d (FUL-D / phase-3J S9) — per-call summary row emitted on
    /// every `tools/call` round-trip. Carries a [`McpToolCalledPayload`].
    McpToolCalled,
    /// Batch-ful-d (FUL-D / phase-3J S9) — MCP server disconnected, either
    /// by user toggle, heartbeat timeout, or peer-closed TCP. Carries a
    /// [`McpServerDisconnectedPayload`].
    McpServerDisconnected,
    /// Tier 1 CHAT window (dashboard-wiring-plan.txt §5) — user-submitted
    /// chat-surface message after client-side intent classification.
    /// Dashboard-originated row; no orchestrator consumer today (same
    /// deferral pattern as `TaskSubmitted`). Carries a [`ChatMessagePayload`].
    ChatMessage,
    /// Tier 1 CHAT window — system-emitted card response rendered in the
    /// chat message list (StatusCard / ErrorCard / DisambiguationCard /
    /// SteeringAckCard). Dashboard-originated. Carries a
    /// [`ChatResponsePayload`].
    ChatResponse,
    /// Tier 4 Settings reorg (dashboard-wiring-plan.txt §8) — execution-mode
    /// selection from the Settings window (LOCAL / CLOUD / HYBRID). Emitted
    /// by the dashboard `set_execution_mode` Tauri command. Carries a
    /// [`ExecutionModeChangedPayload`]. Dashboard-originated row; CLOUD /
    /// HYBRID dispatch paths are not yet implemented (§9 group 20) — the row
    /// is accepted but a follow-up `Finding` row is emitted to flag the gap.
    ExecutionModeChanged,
    /// B-10 (DECISION-C): per-dispatch ContextBudget snapshot. Emitted after
    /// `ContextBudget::compute_with_profile` succeeds at every
    /// `build_dispatch_context` callsite so the dashboard's per-slot bar
    /// chart can render real allocation values matching the active tier
    /// profile (e.g. 12.5/66.7/12.5/8.3 for 7B-12GB). Carries a
    /// [`ContextBudgetComputedPayload`]. Schema version 22.
    ContextBudgetComputed,
    /// B-10 / 3E.5 (DECISION-C): SSB entry filtering summary emitted when
    /// the SsbEntries slot truncation drops candidates below
    /// [`crate::context_budget::FILE_OVERLAP_THRESHOLD`]. Carries a
    /// [`SsbFilteredByOverlapPayload`] with `entries_total` / `entries_kept`
    /// / `overlap_threshold`. Schema version 22.
    SsbFilteredByOverlap,
    /// B-19A (T10) — emitted by the hook dispatcher whenever a T1/T2/T3
    /// hook overruns its tier latency budget
    /// (T1 <= 1ms, T2 <= 10ms, T3 <= 5ms). Carries a
    /// [`HookLatencyViolationPayload`]. Schema version 23.
    HookLatencyViolation,
    /// Track A.2 — research synthesis path selection. Emitted once per
    /// `phase_f_synthesize` call. Carries a [`ResearchPathChosenPayload`].
    ResearchPathChosen,
    /// Track A.6 — grounding cache hit or miss. Emitted by `ground_or_cached`
    /// on each call. Carries a [`GroundingCacheEventPayload`].
    GroundingCacheEvent,
    /// B-4: circuit-breaker state transition for embedding/reranker models.
    /// Emitted by CrossEncoderReranker + OrtVectorModule on CLOSED->OPEN or HALF_OPEN transitions.
    /// Dashboard get_embedding_breakers reads the latest row per `model` field.
    /// Carries a [`CircuitBreakerStateChangePayload`].
    CircuitBreakerStateChange,
    /// B-7: one row per BubbleUpEvent push in handle_failure() / bubble_up_on_*_integration_fail().
    /// Carries a [`BubbleUpTypedPayload`] with from/to level, action variant, and reason.
    BubbleUpTyped,
    /// B-7: emitted at all 4 DecompositionLevel checkpoints by IntegrationCheckpointEmitter.
    /// Carries a [`IntegrationCheckpointSuccessPayload`] with level, node_id, timing, success flag.
    IntegrationCheckpointSuccess,
    /// B-7: one row per RepairAction emitted by DagRepairLoop::repair() via DagRepairEmitter.
    /// Carries a [`DagRepairRoundPayload`] with action kind, round number, and reason string.
    DagRepairRound,
    /// B-7: one row per concept/phase/step/baby_step built inside research_decompose().
    /// Carries a [`ConceptDecompositionNodePayload`] with node_kind, node_id, parent_id, name.
    ConceptDecompositionNode,
    /// B-6: emitted after each EvidenceGate::check() call in the FSM transition path.
    /// Carries an [`EvidenceGateEvaluatedPayload`] with gate_id, verdict, and ts_ms.
    EvidenceGateEvaluated,
    /// B-6: emitted by the supervision layer on each restart or kill event.
    /// Carries a [`SupervisionEventPayload`] with agent_id, event kind, and attempt count.
    SupervisionEvent,
    /// B-6: one row per completed BSP superstep written by the BSP engine.
    /// Carries a [`BspSuperstepPayload`] with snapshot_id, ts_ms, and channels_count.
    BspSuperstep,
    /// B-6: agent lifecycle state change (spawned / ready / exited).
    /// Carries an [`AgentLifecyclePayload`] with agent_id, role, state, and optional death_class.
    AgentLifecycle,
    /// B-6: anti-spiral escalation signal — step hash repeated past threshold.
    /// Carries an [`AntiSpiralHitPayload`] with agent_id, step_hash, and count.
    AntiSpiralHit,
    /// B-27: per-call cloud-mode spend ledger row. Carries a [`CloudSpendPayload`]
    /// with provider, model, dual-rate token counts, and computed cost_usd.
    /// One row per AnthropicClient::send_message round-trip routed through
    /// cloud_mode.rs. Read by `get_cloud_spend` Tauri command for the
    /// MonitorWindow spend gauges (spec D-4:307).
    CloudSpend,
    /// B-27: one-shot marker emitted when cloud_mode.rs promotes a baby-step
    /// from local execution to cloud after the local model exhausts retries
    /// or trips an explicit escalation rule. Carries an [`EscalateToCloudPayload`].
    EscalateToCloud,
    /// B-23 Phase 4H Group A — Layer 1A row insert event. Carries a
    /// [`DataTableGrewPayload`]. Distinct from `ConfigArchived` (capping) and
    /// `ProposalApplied` (the action) — `DataTableGrew` records the actual
    /// table-row growth so the dashboard can render "deprecated_patterns +1
    /// (cargo, blake2->blake3)" without aggregating apply rows.
    DataTableGrew,
    /// B-23 Phase 4H — Layer 3 structural-change proposal emitted for HUMAN
    /// review only. Carries an [`L3ProposalPayload`]. Self-mod agent NEVER
    /// applies these; the dashboard surfaces them for an operator to approve.
    L3Proposal,
    /// B-23 Phase 4H 4H.7B (OSC) — emitted when ConfigVariantSimilarity over
    /// the last `stagnation_window` consecutive variants is >= 0.95 for that
    /// PipelineMode. Carries a [`ConvergenceStagnantPayload`].
    ConvergenceStagnant,
    /// B-23 Phase 4H 4H.7B (OSC) — period-2 oscillation detected
    /// (sim(N,N-2) AND sim(N-1,N-3) both >= 0.95). Carries a
    /// [`ConfigOscillationPayload`].
    ConfigOscillation,
    /// B-23 Phase 4H 4H.7C (REG) — cross-pipeline AC regression detected
    /// (AC failed in latest pipeline, passed in some prior pipeline).
    /// Carries a [`RegressionDetectedPayload`].
    RegressionDetected,
    /// B-23 Phase 4H 4H.7D (EVO) — evolution gate Withhold result emitted
    /// when the latest N variants never structurally evolved (similarity >=
    /// 0.95 across all consecutive pairs). Carries an
    /// [`EvolutionWithheldPayload`]. Forces MSI verdict to STABILIZING for
    /// the affected mode.
    EvolutionWithheld,
    // B-39 chat reflect pipeline: 4 new SsbActionType variants for per-turn REFLECT events.
    /// B-39: TRIAGE stage verdict. Carries a [`ReflectTriagedPayload`].
    ReflectTriaged,
    /// B-39: REASONER candidate emitted. Carries a [`ReflectCandidatePayload`].
    ReflectCandidate,
    /// B-39: AUDITOR best-of-N verdict. Carries a [`ReflectAuditVerdictPayload`].
    ReflectAuditVerdict,
    /// B-39: SYNTHESIZE final synthesis. Carries a [`ReflectRefinedPayload`].
    ReflectRefined,
    // B-24 Phase 4I Group C1: media validator beacons. Payloads land in
    // Group C4 with the dashboard surface; the enum surface is added now
    // so the schema bump is atomic and downstream readers (SsbViewer)
    // can already match on the variant.
    /// B-24: media validation started for a media artifact. Carries
    /// [`MediaValidationStartedPayload`] (Group C4).
    MediaValidationStarted,
    /// B-24: Tier-1 deterministic checks complete. Carries
    /// [`MediaTier1CompletePayload`] (Group C4).
    MediaTier1Complete,
    /// B-24: Tier-2 (VLM) advisory complete. Carries
    /// [`MediaTier2CompletePayload`] (Group C4).
    MediaTier2Complete,
    /// B-24: render adapter finished producing a captured frame set.
    /// Carries [`RenderCompletePayload`] (Group C4).
    RenderComplete,
    /// B-24: one iteration of the visual-feedback loop completed.
    /// Carries [`VisualFeedbackIterationPayload`] (Group C4).
    VisualFeedbackIteration,
    /// B-24: validator escalates to a human-review verdict. Carries
    /// [`MediaHumanReviewRequiredPayload`] (Group C4).
    MediaHumanReviewRequired,
    /// B-28 E-3 (SCHEMA_VERSION 31): focus-tunnel ModeSelector decision
    /// beacon. Emitted once per baby-step in `drive_task` immediately
    /// before the executor dispatch. Carries a
    /// [`GenerationModeSelectedPayload`] with the chosen
    /// `GenerationMode`, the `SelectionRule` that fired, and the active
    /// execution-mode tag — feeds the dashboard `mode_distribution`
    /// surface and the auditor's first-match-wins assertion.
    GenerationModeSelected,
    /// B-28 E-4 (SCHEMA_VERSION 32): focus-tunnel StreamingValidator
    /// early-termination beacon. Emitted by the post-E-7 WIRING phase
    /// whenever the synchronous library-tier validator returns
    /// `ValidationDecision::EarlyTerm`. Carries a
    /// [`StreamingEarlyTerminatedPayload`] with the wire-form
    /// `EarlyTermReason`, the token count at termination, and the active
    /// generation `mode` — feeds the dashboard's "early-term" surface and
    /// the auditor's reason-distribution check. The library validator
    /// itself does NOT emit the row; the wiring layer translates
    /// `EarlyTermWireForm` into this payload.
    StreamingEarlyTerminated,
    /// B-28 WIRING-1 (SCHEMA_VERSION 33): focus-tunnel PipelinePreAssembler
    /// readiness beacon. One row per `PipelinePreAssembler::on_step_outcome`
    /// + `prepare()` invocation. Carries a [`PipelinePreAssembledPayload`]
    /// with `pass_rate_bp` (basis points) + `was_discarded` + `depth_at_time`
    /// + `bytes_assembled` so the dashboard can render the parallelism gauge
    /// and waste-rate bar without aggregating across multiple rows.
    PipelinePreAssembled,
    /// B-28 WIRING-1 (SCHEMA_VERSION 34): focus-tunnel DependencyResolver
    /// decision beacon. Emitted once per `ResolutionDecision` after the
    /// post-execute import-scan + resolve cycle. Carries a
    /// [`DependencyResolvedPayload`] with the wire-form decision string
    /// (`auto_approve` / `queue_for_human` / `blocked`) and gating reason.
    DependencyResolved,
    /// B-28 WIRING-1 (SCHEMA_VERSION 35): focus-tunnel SpeculativeFileRouter
    /// micro-step beacon. Emitted once per `NeedsMicroStep` or
    /// `MicroStepCapExhausted` route decision. Carries a
    /// [`MicroStepCreatedPayload`] with the parent step id + created path +
    /// type_hint (Module / Struct / Function / Const) + reason string.
    MicroStepCreated,
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
            Self::ToolDispatch => "tool_dispatch",
            Self::ValidationTier => "validation_tier",
            Self::FailureClassified => "failure_classified",
            Self::GitCommit => "git_commit",
            Self::DairaTrace => "daira_trace",
            Self::SystemProbe => "system_probe",
            Self::FeasibilityReport => "feasibility_report",
            Self::NovelStepDetected => "novel_step_detected",
            Self::StubRun => "stub_run",
            Self::GateDecision => "gate_decision",
            Self::TaskSplit => "task_split",
            Self::EscalationFired => "escalation_fired",
            Self::ModeChanged => "mode_changed",
            Self::ProposalGenerated => "proposal_generated",
            Self::ProposalApplied => "proposal_applied",
            Self::ProposalRejected => "proposal_rejected",
            Self::MsiUpdate => "msi_update",
            Self::ConfigArchived => "config_archived",
            Self::ProfileSelected => "profile_selected",
            Self::FsmTransition => "fsm_transition",
            Self::MicroReflection => "micro_reflection",
            Self::TelescopeCollapse => "telescope_collapse",
            Self::ConfigWarning => "config_warning",
            Self::ConstraintsInjected => "constraints_injected",
            Self::InterventionResolved => "intervention_resolved",
            Self::DkgQuerierSwapped => "dkg_querier_swapped",
            Self::SteeringComputed => "steering_computed",
            Self::SteeringComputeFailed => "steering_compute_failed",
            Self::McpServerConnected => "mcp_server_connected",
            Self::McpToolCalled => "mcp_tool_called",
            Self::McpServerDisconnected => "mcp_server_disconnected",
            Self::ChatMessage => "chat_message",
            Self::ChatResponse => "chat_response",
            Self::ExecutionModeChanged => "execution_mode_changed",
            Self::ContextBudgetComputed => "context_budget_computed",
            Self::SsbFilteredByOverlap => "ssb_filtered_by_overlap",
            Self::HookLatencyViolation => "hook_latency_violation",
            Self::ResearchPathChosen => "research_path_chosen",
            Self::GroundingCacheEvent => "grounding_cache_event",
            Self::CircuitBreakerStateChange => "circuit_breaker_state_change",
            Self::BubbleUpTyped => "bubble_up_typed",
            Self::IntegrationCheckpointSuccess => "integration_checkpoint_success",
            Self::DagRepairRound => "dag_repair_round",
            Self::ConceptDecompositionNode => "concept_decomposition_node",
            // B-6: orchestration producer wiring.
            Self::EvidenceGateEvaluated => "evidence_gate_evaluated",
            Self::SupervisionEvent => "supervision_event",
            Self::BspSuperstep => "bsp_superstep",
            Self::AgentLifecycle => "agent_lifecycle",
            Self::AntiSpiralHit => "anti_spiral_hit",
            Self::CloudSpend => "cloud_spend",
            Self::EscalateToCloud => "escalate_to_cloud",
            // B-23 Phase 4H Group A.
            Self::DataTableGrew => "data_table_grew",
            Self::L3Proposal => "l3_proposal",
            Self::ConvergenceStagnant => "convergence_stagnant",
            Self::ConfigOscillation => "config_oscillation",
            Self::RegressionDetected => "regression_detected",
            Self::EvolutionWithheld => "evolution_withheld",
            // B-39 chat reflect pipeline.
            Self::ReflectTriaged => "reflect_triaged",
            Self::ReflectCandidate => "reflect_candidate",
            Self::ReflectAuditVerdict => "reflect_audit_verdict",
            Self::ReflectRefined => "reflect_refined",
            // B-24 Phase 4I Group C1: media validator beacons.
            Self::MediaValidationStarted => "media_validation_started",
            Self::MediaTier1Complete => "media_tier1_complete",
            Self::MediaTier2Complete => "media_tier2_complete",
            Self::RenderComplete => "render_complete",
            Self::VisualFeedbackIteration => "visual_feedback_iteration",
            Self::MediaHumanReviewRequired => "media_human_review_required",
            // B-28 E-3 (SCHEMA_VERSION 31): focus-tunnel ModeSelector beacon.
            Self::GenerationModeSelected => "generation_mode_selected",
            // B-28 E-4 (SCHEMA_VERSION 32): focus-tunnel streaming early-term beacon.
            Self::StreamingEarlyTerminated => "streaming_early_terminated",
            // B-28 WIRING-1 (SCHEMA_VERSION 33): pre-assembly readiness beacon.
            Self::PipelinePreAssembled => "pipeline_pre_assembled",
            // B-28 WIRING-1 (SCHEMA_VERSION 34): dependency-resolver decision beacon.
            Self::DependencyResolved => "dependency_resolved",
            // B-28 WIRING-1 (SCHEMA_VERSION 35): micro-step creation beacon.
            Self::MicroStepCreated => "micro_step_created",
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
        "tool_dispatch" => Ok(SsbActionType::ToolDispatch),
        "validation_tier" => Ok(SsbActionType::ValidationTier),
        "failure_classified" => Ok(SsbActionType::FailureClassified),
        "git_commit" => Ok(SsbActionType::GitCommit),
        "daira_trace" => Ok(SsbActionType::DairaTrace),
        "system_probe" => Ok(SsbActionType::SystemProbe),
        "feasibility_report" => Ok(SsbActionType::FeasibilityReport),
        "novel_step_detected" => Ok(SsbActionType::NovelStepDetected),
        "stub_run" => Ok(SsbActionType::StubRun),
        "gate_decision" => Ok(SsbActionType::GateDecision),
        "task_split" => Ok(SsbActionType::TaskSplit),
        "escalation_fired" => Ok(SsbActionType::EscalationFired),
        "mode_changed" => Ok(SsbActionType::ModeChanged),
        "proposal_generated" => Ok(SsbActionType::ProposalGenerated),
        "proposal_applied" => Ok(SsbActionType::ProposalApplied),
        "proposal_rejected" => Ok(SsbActionType::ProposalRejected),
        "msi_update" => Ok(SsbActionType::MsiUpdate),
        "config_archived" => Ok(SsbActionType::ConfigArchived),
        "profile_selected" => Ok(SsbActionType::ProfileSelected),
        "fsm_transition" => Ok(SsbActionType::FsmTransition),
        "micro_reflection" => Ok(SsbActionType::MicroReflection),
        "telescope_collapse" => Ok(SsbActionType::TelescopeCollapse),
        "config_warning" => Ok(SsbActionType::ConfigWarning),
        "constraints_injected" => Ok(SsbActionType::ConstraintsInjected),
        "intervention_resolved" => Ok(SsbActionType::InterventionResolved),
        "dkg_querier_swapped" => Ok(SsbActionType::DkgQuerierSwapped),
        "steering_computed" => Ok(SsbActionType::SteeringComputed),
        "steering_compute_failed" => Ok(SsbActionType::SteeringComputeFailed),
        "mcp_server_connected" => Ok(SsbActionType::McpServerConnected),
        "mcp_tool_called" => Ok(SsbActionType::McpToolCalled),
        "mcp_server_disconnected" => Ok(SsbActionType::McpServerDisconnected),
        "chat_message" => Ok(SsbActionType::ChatMessage),
        "chat_response" => Ok(SsbActionType::ChatResponse),
        "execution_mode_changed" => Ok(SsbActionType::ExecutionModeChanged),
        "context_budget_computed" => Ok(SsbActionType::ContextBudgetComputed),
        "ssb_filtered_by_overlap" => Ok(SsbActionType::SsbFilteredByOverlap),
        "hook_latency_violation" => Ok(SsbActionType::HookLatencyViolation),
        "research_path_chosen" => Ok(SsbActionType::ResearchPathChosen),
        "grounding_cache_event" => Ok(SsbActionType::GroundingCacheEvent),
        // B-4 (v26 bump completed).
        "circuit_breaker_state_change" => Ok(SsbActionType::CircuitBreakerStateChange),
        // B-7 (v26 bump completed).
        "bubble_up_typed" => Ok(SsbActionType::BubbleUpTyped),
        "integration_checkpoint_success" => Ok(SsbActionType::IntegrationCheckpointSuccess),
        "dag_repair_round" => Ok(SsbActionType::DagRepairRound),
        "concept_decomposition_node" => Ok(SsbActionType::ConceptDecompositionNode),
        // B-6: orchestration producer wiring.
        "evidence_gate_evaluated" => Ok(SsbActionType::EvidenceGateEvaluated),
        "supervision_event" => Ok(SsbActionType::SupervisionEvent),
        "bsp_superstep" => Ok(SsbActionType::BspSuperstep),
        "agent_lifecycle" => Ok(SsbActionType::AgentLifecycle),
        "anti_spiral_hit" => Ok(SsbActionType::AntiSpiralHit),
        // B-27 cloud-mode action types.
        "cloud_spend" => Ok(SsbActionType::CloudSpend),
        "escalate_to_cloud" => Ok(SsbActionType::EscalateToCloud),
        // B-23 Phase 4H Group A self-mod action types.
        "data_table_grew" => Ok(SsbActionType::DataTableGrew),
        "l3_proposal" => Ok(SsbActionType::L3Proposal),
        "convergence_stagnant" => Ok(SsbActionType::ConvergenceStagnant),
        "config_oscillation" => Ok(SsbActionType::ConfigOscillation),
        "regression_detected" => Ok(SsbActionType::RegressionDetected),
        "evolution_withheld" => Ok(SsbActionType::EvolutionWithheld),
        // B-39 chat reflect pipeline.
        "reflect_triaged" => Ok(SsbActionType::ReflectTriaged),
        "reflect_candidate" => Ok(SsbActionType::ReflectCandidate),
        "reflect_audit_verdict" => Ok(SsbActionType::ReflectAuditVerdict),
        "reflect_refined" => Ok(SsbActionType::ReflectRefined),
        // B-24 Phase 4I Group C1: media validator beacons.
        "media_validation_started" => Ok(SsbActionType::MediaValidationStarted),
        "media_tier1_complete" => Ok(SsbActionType::MediaTier1Complete),
        "media_tier2_complete" => Ok(SsbActionType::MediaTier2Complete),
        "render_complete" => Ok(SsbActionType::RenderComplete),
        "visual_feedback_iteration" => Ok(SsbActionType::VisualFeedbackIteration),
        "media_human_review_required" => Ok(SsbActionType::MediaHumanReviewRequired),
        // B-28 E-3 (SCHEMA_VERSION 31).
        "generation_mode_selected" => Ok(SsbActionType::GenerationModeSelected),
        // B-28 E-4 (SCHEMA_VERSION 32).
        "streaming_early_terminated" => Ok(SsbActionType::StreamingEarlyTerminated),
        // B-28 WIRING-1 (SCHEMA_VERSION 33 -> 35).
        "pipeline_pre_assembled" => Ok(SsbActionType::PipelinePreAssembled),
        "dependency_resolved" => Ok(SsbActionType::DependencyResolved),
        "micro_step_created" => Ok(SsbActionType::MicroStepCreated),
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
    /// Model identifier: "graphcodebert" | "granite-embed" | "ms-marco-l12" | "ms-marco-l6" | "splade-v3"
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

/// Batch 3A — snapshot of the [`crate::system_probe::SystemProbeReport`] at
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
                .map(|n| u32::try_from(n.get()).unwrap_or(u32::MAX))
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
            // NARROWING-OK: u32 pool size always fits usize on supported 32/64-bit targets.
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
    let created_at = i64::try_from(
        clock
            .system_now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or(Duration::ZERO)
            .as_secs(),
    )
    .unwrap_or(i64::MAX);

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
        match tokio::runtime::Handle::try_current() {
            Ok(handle) => match handle.runtime_flavor() {
                tokio::runtime::RuntimeFlavor::MultiThread => {
                    // Multi-thread runtime: block_in_place is safe.
                    tokio::task::block_in_place(|| {
                        handle.block_on(async move {
                            let obj = pool
                                .get()
                                .await
                                .map_err(|e| SsbError::Pool(format!("{e}")))?;
                            f(&obj)
                        })
                    })
                }
                _ => {
                    // CurrentThread runtime context (e.g. rt.block_on in
                    // try_save_snapshot): block_in_place panics here, and
                    // re-entering the same runtime deadlocks. Spawn an OS
                    // thread with its own transient runtime to drive the work.
                    std::thread::spawn(move || {
                        let rt = tokio::runtime::Builder::new_current_thread()
                            .enable_all()
                            .build()
                            .map_err(|e| SsbError::Pool(format!("rt build: {e}")))?;
                        rt.block_on(async move {
                            let obj = pool
                                .get()
                                .await
                                .map_err(|e| SsbError::Pool(format!("{e}")))?;
                            f(&obj)
                        })
                    })
                    .join()
                    .map_err(|_| SsbError::Pool("read worker panicked".into()))?
                }
            },
            Err(_) => {
                // No async context: spin transient current-thread runtime.
                let rt = tokio::runtime::Builder::new_current_thread()
                    .enable_all()
                    .build()
                    .map_err(|e| SsbError::Pool(format!("rt build: {e}")))?;
                rt.block_on(async move {
                    let obj = pool
                        .get()
                        .await
                        .map_err(|e| SsbError::Pool(format!("{e}")))?;
                    f(&obj)
                })
            }
        }
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
// B-1: canonical SSB sessions subdir now under .elai_cc/sessions.
// SSB_SESSIONS_SUBDIR kept for source-compat (external callers); value updated.
pub const SSB_SESSIONS_SUBDIR: &str = ".elai_cc/sessions";

/// File name of the canonical SSB SQLite database inside
/// [`SSB_SESSIONS_SUBDIR`].
pub const SSB_DB_FILE_NAME: &str = "ssb.db";

/// Canonical SSB database path for the given project/base directory.
///
/// The CLI's sidecar bootstrap uses `<session-persistence-dir>/ssb.db`, and
/// managed session persistence files live under `<base>/.elai_cc/sessions/`
/// (see `session_control::managed_sessions_dir_for`). The dashboard must
/// open the SSB at the **same** path so it and a running CLI REPL share
/// telemetry instead of silently creating a second, empty database.
///
/// Returns `<base_dir>/.elai_cc/sessions/ssb.db` as a pure path computation
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
// B-39: ReflectEventSink bridge — translates reasoning crate events to SSB rows.
// ---------------------------------------------------------------------------

impl ReflectEventSink for Ssb {
    /// Translates each [`ReflectEvent`] variant into the corresponding
    /// [`SsbActionType`] + [`SsbPayload`] and appends to the SSB.
    ///
    /// Write is fire-and-forget via `tokio::spawn`: the pipeline never awaits
    /// SSB persistence. Write failures are logged at WARN level and silently
    /// discarded — the pipeline must never propagate SSB errors to the caller.
    fn emit(&self, event: ReflectEvent) {
        let (action_type, payload) = match event {
            ReflectEvent::Triaged { turn_id, class } => (
                SsbActionType::ReflectTriaged,
                SsbPayload::ReflectTriaged(ReflectTriagedPayload {
                    turn_id,
                    class: class.as_wire_str().to_owned(),
                }),
            ),
            ReflectEvent::Candidate {
                turn_id,
                candidate_index,
                content,
            } => (
                SsbActionType::ReflectCandidate,
                SsbPayload::ReflectCandidate(ReflectCandidatePayload {
                    turn_id,
                    candidate_index,
                    content,
                }),
            ),
            ReflectEvent::AuditVerdict {
                turn_id,
                best_index,
                rationale,
            } => (
                SsbActionType::ReflectAuditVerdict,
                SsbPayload::ReflectAuditVerdict(ReflectAuditVerdictPayload {
                    turn_id,
                    best_index,
                    rationale,
                }),
            ),
            ReflectEvent::Refined {
                turn_id,
                total_tokens_used,
                synthesis,
            } => (
                SsbActionType::ReflectRefined,
                SsbPayload::ReflectRefined(ReflectRefinedPayload {
                    turn_id,
                    total_tokens_used,
                    synthesis,
                }),
            ),
        };

        // Serialize payload eagerly on the caller's thread; spawn write async.
        let bytes = match rmp_serde::to_vec_named(&payload) {
            Ok(b) => b,
            Err(e) => {
                tracing::warn!(
                    target: "orchestrator::ssb",
                    error = %e,
                    action_type = action_type.as_sql_str(),
                    "ReflectEventSink: payload serialization failed — dropping event"
                );
                return;
            }
        };

        if bytes.len() > PAYLOAD_MAX_BYTES {
            tracing::warn!(
                target: "orchestrator::ssb",
                action_type = action_type.as_sql_str(),
                size = bytes.len(),
                max = PAYLOAD_MAX_BYTES,
                "ReflectEventSink: payload exceeds size limit — dropping event"
            );
            return;
        }

        let write_tx = match &self.write_tx {
            Some(tx) => tx.clone(),
            None => {
                tracing::warn!(
                    target: "orchestrator::ssb",
                    action_type = action_type.as_sql_str(),
                    "ReflectEventSink: SSB is shut down — dropping event"
                );
                return;
            }
        };

        let request = SsbWriteRequest {
            action_type,
            // Reflect events are emitted by the CLI chat loop; agent_id is
            // not meaningful here, so we use a stable sentinel.
            agent_id: "reflect_pipeline".to_owned(),
            superstep: 0,
            payload: bytes,
            files: vec![],
        };

        let (respond, _rx) = tokio::sync::oneshot::channel();
        if let Err(e) = write_tx.send(WriteMessage { request, respond }) {
            tracing::warn!(
                target: "orchestrator::ssb",
                error = %e,
                action_type = action_type.as_sql_str(),
                "ReflectEventSink: write channel send failed — dropping event"
            );
        }
        // _rx dropped intentionally: fire-and-forget; no await on the response.
    }
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
                file_path: None, line: None, failure_category: None,
                audit_step: None, few_shot_examples: None,
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
                fetcher: "reqwest".into(),
                stage: String::new(), stage_phase: String::new(), stage_payload: None,
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
                error_kind: None,
                error_message: None,
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
            SsbActionType::ToolDispatch => SsbPayload::ToolDispatch(ToolDispatchPayload {
                tool: "edit".into(),
                args_summary: "path=src/foo.rs tier=exact".into(),
                elapsed_ms: 3,
                step_id: "step-1".into(),
                outcome: "success".into(),
                error_category: None,
            }),
            SsbActionType::ValidationTier => {
                SsbPayload::ValidationTier(ValidationTierPayload {
                    tier: "determinism".into(),
                    passed: true,
                    elapsed_ms: 1200,
                    rules_fired: 1,
                    step_id: "step-1".into(),
                })
            }
            SsbActionType::FailureClassified => {
                SsbPayload::FailureClassified(FailureClassifiedPayload {
                    category: "COMPILATION_ERROR".into(),
                    confidence_bp: 10000,
                    step_id: "step-1".into(),
                    source: "validator".into(),
                })
            }
            SsbActionType::GitCommit => SsbPayload::GitCommit(GitCommitPayload {
                commit_sha: "abc123def456".into(),
                branch: "elai/plan-demo".into(),
                files_count: 2,
                step_id: "step-1".into(),
                message: "batch-2b: test commit".into(),
            }),
            SsbActionType::DairaTrace => SsbPayload::DairaTrace(DairaTracePayload {
                phase: "enter".into(),
                tool: "edit".into(),
                args_hash: "deadbeef".into(),
                elapsed_us: 0,
                outcome: "started".into(),
                step_id: "step-1".into(),
            }),
            SsbActionType::SystemProbe => SsbPayload::SystemProbe(SystemProbePayload {
                gpu_name: Some("test".into()),
                vram_total_mb: Some(12_288),
                vram_free_mb: Some(6_144),
                ram_total_mb: 32_000,
                ram_free_mb: 16_000,
                disk_total_mb: 1_000_000,
                disk_free_mb: 500_000,
                detection_method: "shell_out".into(),
                tier: "12GB".into(),
            }),
            SsbActionType::FeasibilityReport => {
                SsbPayload::FeasibilityReport(FeasibilityReportPayload {
                    task_feasibility: "feasible".into(),
                    skip_reason: None,
                    novel_step_count: 0,
                    escalations_count: 0,
                    vram_margin_bp: 5_000,
                    ram_margin_bp: 5_000,
                    disk_margin_bp: 5_000,
                })
            }
            SsbActionType::NovelStepDetected => {
                SsbPayload::NovelStepDetected(NovelStepDetectedPayload {
                    step_id: "step-1".into(),
                    reason: "unverified".into(),
                    alternative_count: 0,
                })
            }
            SsbActionType::StubRun => SsbPayload::StubRun(StubRunPayload {
                step_id: "step-1".into(),
                passed: true,
                elapsed_ms: 12,
            }),
            SsbActionType::GateDecision => SsbPayload::GateDecision(GateDecisionPayload {
                decision: "auto_advance".into(),
                risk_level: Some("LOW".into()),
                reason: None,
                evidence_summary: "kr=1 zu=1 tfc=1 tf=1 zb=1 dcg=1".into(),
                step_id: "step-1".into(),
            }),
            SsbActionType::TaskSplit => SsbPayload::TaskSplit(TaskSplitPayload {
                original_task_id: "task-1".into(),
                sub_task_ids: vec!["task-1.1".into(), "task-1.2".into()],
                depth: 1,
                sub_task_summaries: None,
                splitter: None,
                complexity_estimate: None,
            }),
            SsbActionType::EscalationFired => SsbPayload::EscalationFired(EscalationFiredPayload {
                trigger: "anti_spiral".into(),
                step_id: "step-1".into(),
                details: "spiral_depth=2".into(),
            }),
            SsbActionType::ModeChanged => SsbPayload::ModeChanged(ModeChangedPayload {
                from: "interactive".into(),
                to: "autonomous".into(),
            }),
            // Batch 3C (Phase 4H) — Layer 1A.
            SsbActionType::ProposalGenerated => SsbPayload::ProposalGenerated(ProposalGeneratedPayload {
                proposal_id: "pp-0000-1".into(),
                table: "lazy_code_regex".into(),
                new_row_summary: "todo!".into(),
                fpr_estimate: 0.0,
                source: "auto_from_outcome".into(),
            }),
            SsbActionType::ProposalApplied => SsbPayload::ProposalApplied(ProposalAppliedPayload {
                proposal_id: "pp-0000-1".into(),
                table: "lazy_code_regex".into(),
                applied_by: "layer1a".into(),
                commit_sha: None,
            }),
            SsbActionType::ProposalRejected => SsbPayload::ProposalRejected(ProposalRejectedPayload {
                proposal_id: "pp-0000-1".into(),
                table: "lazy_code_regex".into(),
                reason: "fpr too high".into(),
            }),
            SsbActionType::MsiUpdate => SsbPayload::MsiUpdate(MsiUpdatePayload {
                pipeline_mode: "surgical".into(),
                msi_value: 0.5,
                status: "IMPROVING".into(),
            }),
            SsbActionType::ConfigArchived => SsbPayload::ConfigArchived(ConfigArchivedPayload {
                pipeline_mode: "surgical".into(),
                archived_count: 1,
                dropped_id: None,
            }),
            // Batch 4C.
            SsbActionType::ProfileSelected => SsbPayload::ProfileSelected(ProfileSelectedPayload {
                profile: "FirstAttempt".into(),
                role: "executor".into(),
                fsm_state: "EXECUTING".into(),
                step_id: "step-1".into(),
            }),
            SsbActionType::FsmTransition => SsbPayload::FsmTransition(FsmTransitionPayload {
                from: "IDLE".into(),
                to: "RESEARCHING".into(),
                reason: "web_research_task".into(),
            }),
            SsbActionType::MicroReflection => SsbPayload::MicroReflection(MicroReflectionPayload {
                step_id: "step-1".into(),
                root_cause: "missing import".into(),
                proposed_fix: "add use crate::foo".into(),
                confidence: 0.8,
            }),
            SsbActionType::TelescopeCollapse => {
                SsbPayload::TelescopeCollapse(TelescopeCollapsePayload {
                    from_tokens: 1800,
                    to_tokens: 1400,
                    layers_collapsed: 1,
                    step_id: "step-1".into(),
                })
            }
            SsbActionType::ConfigWarning => SsbPayload::ConfigWarning(ConfigWarningPayload {
                config: "source-tiers.toml".into(),
                reason: "file_not_found".into(),
                detail: "path=/missing".into(),
            }),
            // Batch-fu.
            SsbActionType::ConstraintsInjected => {
                SsbPayload::ConstraintsInjected(ConstraintsInjectedPayload {
                    project_id: "proj-test".into(),
                    constraint_count: 2,
                    step_id: "step-1".into(),
                })
            }
            SsbActionType::InterventionResolved => {
                SsbPayload::InterventionResolved(InterventionResolvedPayload {
                    intervention_id: "iv-0000-0001".into(),
                    approved: true,
                    elapsed_ms: 250,
                })
            }
            SsbActionType::DkgQuerierSwapped => {
                SsbPayload::DkgQuerierSwapped(DkgQuerierSwappedPayload {
                    project_id: "p".into(),
                    from: "empty".into(),
                    to: "adapter".into(),
                })
            }
            SsbActionType::SteeringComputed => {
                SsbPayload::SteeringComputed(SteeringComputedPayload {
                    role: "executor".into(),
                    vector_dim: 2048,
                    norm: 12.34,
                    elapsed_ms: 150,
                    cache_path: "/tmp/.elai_cc/steering/p/executor.bin".into(),
                })
            }
            SsbActionType::SteeringComputeFailed => {
                SsbPayload::SteeringComputeFailed(SteeringComputeFailedPayload {
                    role: "executor".into(),
                    reason: "sidecar_unreachable".into(),
                })
            }
            SsbActionType::McpServerConnected => {
                SsbPayload::McpServerConnected(McpServerConnectedPayload {
                    server_url: "http://localhost:8200".into(),
                    tool_count: 3,
                })
            }
            SsbActionType::McpToolCalled => {
                SsbPayload::McpToolCalled(McpToolCalledPayload {
                    tool_name: "echo".into(),
                    server_url: "http://localhost:8200".into(),
                    elapsed_ms: 42,
                    outcome: "success".into(),
                })
            }
            SsbActionType::McpServerDisconnected => {
                SsbPayload::McpServerDisconnected(McpServerDisconnectedPayload {
                    server_url: "http://localhost:8200".into(),
                    reason: "user_disabled".into(),
                })
            }
            SsbActionType::ChatMessage => SsbPayload::ChatMessage(ChatMessagePayload {
                session_id: "s-test".into(),
                text: "hello".into(),
                intent: "TASK".into(),
                project_id: "p".into(),
                submitted_at_ms: 0,
            }),
            SsbActionType::ChatResponse => SsbPayload::ChatResponse(ChatResponsePayload {
                session_id: "s-test".into(),
                card_kind: "status".into(),
                body: "ok".into(),
                project_id: "p".into(),
                emitted_at_ms: 0,
            }),
            SsbActionType::ExecutionModeChanged => SsbPayload::ExecutionModeChanged(ExecutionModeChangedPayload {
                from_mode: "LOCAL".into(),
                to_mode: "CLOUD".into(),
                project_id: "p".into(),
                changed_at_ms: 0,
            }),
            SsbActionType::ContextBudgetComputed => {
                SsbPayload::ContextBudgetComputed(ContextBudgetComputedPayload {
                    tier: "micro_7b_12gb".into(),
                    raw_max: 25000,
                    slots: vec![0; 8],
                    allocations: vec![330, 400, 800, 1500, 8000, 1500, 1000, 512],
                    adapt_levels_fired: 0,
                })
            }
            SsbActionType::SsbFilteredByOverlap => {
                SsbPayload::SsbFilteredByOverlap(SsbFilteredByOverlapPayload {
                    entries_total: 10,
                    entries_kept: 4,
                    overlap_threshold: 0.10,
                })
            }
            SsbActionType::HookLatencyViolation => {
                SsbPayload::HookLatencyViolation(HookLatencyViolationPayload {
                    hook_name: "PreToolUse".into(),
                    tier: "T1".into(),
                    budget_ms: 1,
                    actual_ms: 12,
                })
            }
            SsbActionType::ResearchPathChosen => {
                SsbPayload::ResearchPathChosen(ResearchPathChosenPayload {
                    path: "llm".into(),
                    reason: "llm".into(),
                    ts_ms: 0,
                })
            }
            SsbActionType::GroundingCacheEvent => {
                SsbPayload::GroundingCacheEvent(GroundingCacheEventPayload {
                    kind: "miss".into(),
                    key_fingerprint: "00000000".into(),
                    level: "Idea".into(),
                    elapsed_us: 0,
                })
            }
            // B-4 (v26 bump completed).
            SsbActionType::CircuitBreakerStateChange => {
                SsbPayload::CircuitBreakerStateChange(CircuitBreakerStateChangePayload {
                    model: "graphcodebert".into(),
                    state: "CLOSED".into(),
                    previous_state: "CLOSED".into(),
                    failures: 0,
                    success_rate: 1.0,
                    last_failure_ts: None,
                })
            }
            // B-7 variants.
            SsbActionType::BubbleUpTyped => SsbPayload::BubbleUpTyped(BubbleUpTypedPayload {
                from_level: "BabyStep".into(),
                to_level: "Step".into(),
                action_kind: "diagnose_and_retry".into(),
                reason: "test".into(),
                failure_count: 1,
                threshold: 3,
            }),
            SsbActionType::IntegrationCheckpointSuccess => {
                SsbPayload::IntegrationCheckpointSuccess(IntegrationCheckpointSuccessPayload {
                    level: "BabyStep".into(),
                    node_id: 0,
                    wiring_target: "test-target".into(),
                    success: true,
                    failure_reason: None,
                    duration_ms: 1,
                })
            }
            SsbActionType::DagRepairRound => SsbPayload::DagRepairRound(DagRepairRoundPayload {
                action_kind: "reorder".into(),
                round: 1,
                reason: "test".into(),
            }),
            SsbActionType::ConceptDecompositionNode => {
                SsbPayload::ConceptDecompositionNode(ConceptDecompositionNodePayload {
                    node_kind: "concept".into(),
                    node_id: 0,
                    parent_id: 0,
                    name: "test-concept".into(),
                })
            }
            // B-6 variants.
            SsbActionType::EvidenceGateEvaluated => {
                SsbPayload::EvidenceGateEvaluated(EvidenceGateEvaluatedPayload {
                    gate_id: "knowledge_readiness_gate".into(),
                    verdict: "pass".into(),
                    ts_ms: 0,
                })
            }
            SsbActionType::SupervisionEvent => {
                SsbPayload::SupervisionEvent(SupervisionEventPayload {
                    agent_id: "a".into(),
                    event: "restart".into(),
                    attempt: 1,
                })
            }
            SsbActionType::BspSuperstep => SsbPayload::BspSuperstep(BspSuperstepPayload {
                snapshot_id: "s-test:superstep:1".into(),
                ts_ms: 0,
                channels_count: 1,
            }),
            SsbActionType::AgentLifecycle => SsbPayload::AgentLifecycle(AgentLifecyclePayload {
                agent_id: "a".into(),
                role: "executor".into(),
                state: "spawned".into(),
                death_class: None,
            }),
            SsbActionType::AntiSpiralHit => SsbPayload::AntiSpiralHit(AntiSpiralHitPayload {
                agent_id: "a".into(),
                step_hash: "0000000000000000000000000000000000000000000000000000000000000000".into(),
                count: 4,
            }),
            SsbActionType::CloudSpend => SsbPayload::CloudSpend(CloudSpendPayload {
                provider: "anthropic".into(),
                model: "claude-sonnet-4-6".into(),
                input_tokens: 10,
                output_tokens: 2,
                cache_creation_tokens: 0,
                cache_read_tokens: 0,
                cost_usd: 0.000_06,
                task_id: "t".into(),
                ts_ms: 0,
            }),
            SsbActionType::EscalateToCloud => SsbPayload::EscalateToCloud(EscalateToCloudPayload {
                step_id: "step-1".into(),
                trigger_reason: "test".into(),
                layer_failed: 1,
                ts_ms: 0,
            }),
            // B-23 Phase 4H Group A.
            SsbActionType::DataTableGrew => SsbPayload::DataTableGrew(DataTableGrewPayload {
                proposal_id: "prop-1".into(),
                table: "deprecated_patterns".into(),
                pipeline_mode: "surgical".into(),
                ecosystem: Some("cargo".into()),
                language: Some("Rust".into()),
                rowcount_after: 12,
                ts_ms: 0,
            }),
            SsbActionType::L3Proposal => SsbPayload::L3Proposal(L3ProposalPayload {
                proposal_id: "l3-1".into(),
                change_type: "allowed_config_key_addition".into(),
                description: "add EtcThreshold variant".into(),
                rationale: "test".into(),
                ts_ms: 0,
            }),
            SsbActionType::ConvergenceStagnant => {
                SsbPayload::ConvergenceStagnant(ConvergenceStagnantPayload {
                    pipeline_mode: "surgical".into(),
                    window: 3,
                    similarity: 0.96,
                    variant_id: "v-7".into(),
                    ts_ms: 0,
                })
            }
            SsbActionType::ConfigOscillation => {
                SsbPayload::ConfigOscillation(ConfigOscillationPayload {
                    pipeline_mode: "surgical".into(),
                    period: 2,
                    frozen_keys: vec!["etc_threshold".into()],
                    freeze_pipelines: 5,
                    ts_ms: 0,
                })
            }
            SsbActionType::RegressionDetected => {
                SsbPayload::RegressionDetected(RegressionDetectedPayload {
                    pipeline_id: "pipe-99".into(),
                    pipeline_mode: "greenfield".into(),
                    regressed_ac_indices: vec![0, 3, 7],
                    max_consecutive_failures: 2,
                    ts_ms: 0,
                })
            }
            SsbActionType::EvolutionWithheld => {
                SsbPayload::EvolutionWithheld(EvolutionWithheldPayload {
                    pipeline_mode: "surgical".into(),
                    identical_streak: 5,
                    reason: "ontology never evolved".into(),
                    ts_ms: 0,
                })
            }
            // B-39 chat reflect pipeline.
            SsbActionType::ReflectTriaged => SsbPayload::ReflectTriaged(ReflectTriagedPayload {
                turn_id: "turn-1".into(),
                class: "problem".into(),
            }),
            SsbActionType::ReflectCandidate => SsbPayload::ReflectCandidate(ReflectCandidatePayload {
                turn_id: "turn-1".into(),
                candidate_index: 0,
                content: "test-content".into(),
            }),
            SsbActionType::ReflectAuditVerdict => SsbPayload::ReflectAuditVerdict(ReflectAuditVerdictPayload {
                turn_id: "turn-1".into(),
                best_index: 0,
                rationale: "most direct".into(),
            }),
            SsbActionType::ReflectRefined => SsbPayload::ReflectRefined(ReflectRefinedPayload {
                turn_id: "turn-1".into(),
                total_tokens_used: 0,
                synthesis: "test-synthesis".into(),
            }),
            // B-24 Phase 4I Group C1: media validator beacons (test fixtures).
            SsbActionType::MediaValidationStarted => {
                SsbPayload::MediaValidationStarted(MediaValidationStartedPayload {
                    step_id: "step-1".into(),
                    media_type: "image".into(),
                    file_path: "/tmp/x.png".into(),
                })
            }
            SsbActionType::MediaTier1Complete => {
                SsbPayload::MediaTier1Complete(MediaTier1CompletePayload {
                    step_id: "step-1".into(),
                    pass: true,
                    check_count: 0,
                    latency_ms: 0,
                })
            }
            SsbActionType::MediaTier2Complete => {
                SsbPayload::MediaTier2Complete(MediaTier2CompletePayload {
                    step_id: "step-1".into(),
                    advisory_pass: true,
                    model_used: "mock-vlm".into(),
                    latency_ms: 0,
                })
            }
            SsbActionType::RenderComplete => SsbPayload::RenderComplete(RenderCompletePayload {
                step_id: "step-1".into(),
                adapter_name: "web".into(),
                frame_count: 1,
                latency_ms: 0,
            }),
            SsbActionType::VisualFeedbackIteration => {
                SsbPayload::VisualFeedbackIteration(VisualFeedbackIterationPayload {
                    step_id: "step-1".into(),
                    iteration: 1,
                    issue_count: 0,
                    converged: true,
                })
            }
            SsbActionType::MediaHumanReviewRequired => {
                SsbPayload::MediaHumanReviewRequired(MediaHumanReviewRequiredPayload {
                    step_id: "step-1".into(),
                    reason: "test-reason".into(),
                })
            }
            SsbActionType::GenerationModeSelected => {
                SsbPayload::GenerationModeSelected(GenerationModeSelectedPayload {
                    step_id: "baby_step_1".into(),
                    mode: "structured".into(),
                    rule_fired: "structured_fallback".into(),
                    reason: "rule_5_structured_fallback".into(),
                    preferred_was: None,
                    supports_fim: false,
                    exec_mode: "local".into(),
                })
            }
            SsbActionType::StreamingEarlyTerminated => {
                SsbPayload::StreamingEarlyTerminated(StreamingEarlyTerminatedPayload {
                    step_id: "baby_step_1".into(),
                    mode: "fim".into(),
                    reason: "lazy_pattern".into(),
                    reason_detail: Some("rust_todo_macro".into()),
                    token_count_at_term: 17,
                    total_tokens_received: 17,
                    language_hint: "rust".into(),
                })
            }
            SsbActionType::PipelinePreAssembled => {
                SsbPayload::PipelinePreAssembled(PipelinePreAssembledPayload {
                    step_idx: 0,
                    pass_rate_bp: 10_000,
                    was_discarded: false,
                    depth_at_time: 1,
                    bytes_assembled: 1024,
                })
            }
            SsbActionType::DependencyResolved => {
                SsbPayload::DependencyResolved(DependencyResolvedPayload {
                    language: "rust".into(),
                    name: "tokio".into(),
                    decision: "auto_approve".into(),
                    reason: "allowlist_match".into(),
                    install_command: Some("cargo add tokio".into()),
                })
            }
            SsbActionType::MicroStepCreated => {
                SsbPayload::MicroStepCreated(MicroStepCreatedPayload {
                    parent_step_id: 1,
                    created_path: "crates/foo/src/bar.rs".into(),
                    type_hint: "Module".into(),
                    reason: "undefined_symbol".into(),
                    count: 1,
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
            SsbActionType::ExecutionModeChanged,
            SsbActionType::ContextBudgetComputed,
            SsbActionType::SsbFilteredByOverlap,
            SsbActionType::HookLatencyViolation,
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
        // B-10 (DECISION-C): subset count is 35 (Tier-4 added 1, B-10 added 2).
        // B-19A: subset count is 36 (B-19A added HookLatencyViolation).
        // The assertion grows as variants are added to the test's `types` vec
        // above; both must move in lockstep.
        assert_eq!(count, 36, "must have inserted all 36 types in this test list");
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
                failure_category: None,
                audit_step: None,
                few_shot_examples: None,
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
                fetcher: "reqwest".into(),
                stage: String::new(), stage_phase: String::new(), stage_payload: None,
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
            SsbPayload::ChatMessage(ChatMessagePayload {
                session_id: "s-dash".into(),
                text: "/task rebuild the dashboard".into(),
                intent: "TASK".into(),
                project_id: "dash-mvp".into(),
                submitted_at_ms: 1_713_300_000_100,
            }),
            SsbPayload::ChatResponse(ChatResponsePayload {
                session_id: "s-dash".into(),
                card_kind: "status".into(),
                body: "routed to TASK".into(),
                project_id: "dash-mvp".into(),
                emitted_at_ms: 1_713_300_000_150,
            }),
            SsbPayload::ExecutionModeChanged(ExecutionModeChangedPayload {
                from_mode: "LOCAL".into(),
                to_mode: "HYBRID".into(),
                project_id: "dash-mvp".into(),
                changed_at_ms: 1_713_300_000_200,
            }),
            // B-10 (DECISION-C) — 2 new variants for the dashboard's per-slot
            // budget bar chart and SSB entry filter transparency.
            SsbPayload::ContextBudgetComputed(ContextBudgetComputedPayload {
                tier: "micro_7b_12gb".into(),
                raw_max: 25_000,
                slots: vec![0, 0, 0, 1250, 6670, 1250, 830, 0],
                allocations: vec![330, 400, 800, 1500, 8000, 1500, 1000, 512],
                adapt_levels_fired: 0,
            }),
            SsbPayload::SsbFilteredByOverlap(SsbFilteredByOverlapPayload {
                entries_total: 12,
                entries_kept: 4,
                overlap_threshold: 0.10,
            }),
            // B-39 chat reflect pipeline.
            SsbPayload::ReflectTriaged(ReflectTriagedPayload {
                turn_id: "turn-1".into(),
                class: "problem".into(),
            }),
            SsbPayload::ReflectCandidate(ReflectCandidatePayload {
                turn_id: "turn-1".into(),
                candidate_index: 0,
                content: "test-content".into(),
            }),
            SsbPayload::ReflectAuditVerdict(ReflectAuditVerdictPayload {
                turn_id: "turn-1".into(),
                best_index: 0,
                rationale: "most direct".into(),
            }),
            SsbPayload::ReflectRefined(ReflectRefinedPayload {
                turn_id: "turn-1".into(),
                total_tokens_used: 0,
                synthesis: "test-synthesis".into(),
            }),
            // B-24 Phase 4I Group C1: media validator beacons.
            SsbPayload::MediaValidationStarted(MediaValidationStartedPayload {
                step_id: "step-1".into(),
                media_type: "image".into(),
                file_path: "/tmp/x.png".into(),
            }),
            SsbPayload::MediaTier1Complete(MediaTier1CompletePayload {
                step_id: "step-1".into(),
                pass: true,
                check_count: 0,
                latency_ms: 0,
            }),
            SsbPayload::MediaTier2Complete(MediaTier2CompletePayload {
                step_id: "step-1".into(),
                advisory_pass: true,
                model_used: "mock-vlm".into(),
                latency_ms: 0,
            }),
            SsbPayload::RenderComplete(RenderCompletePayload {
                step_id: "step-1".into(),
                adapter_name: "web".into(),
                frame_count: 1,
                latency_ms: 0,
            }),
            SsbPayload::VisualFeedbackIteration(VisualFeedbackIterationPayload {
                step_id: "step-1".into(),
                iteration: 1,
                issue_count: 0,
                converged: true,
            }),
            SsbPayload::MediaHumanReviewRequired(MediaHumanReviewRequiredPayload {
                step_id: "step-1".into(),
                reason: "test-reason".into(),
            }),
            SsbPayload::GenerationModeSelected(GenerationModeSelectedPayload {
                step_id: "baby_step_99".into(),
                mode: "fim".into(),
                rule_fired: "fim_all_conditions_met".into(),
                reason: "rule_3_fim_eligible".into(),
                preferred_was: None,
                supports_fim: true,
                exec_mode: "local".into(),
            }),
            // B-28 E-4 (SCHEMA_VERSION 32) — focus-tunnel StreamingValidator early-term.
            SsbPayload::StreamingEarlyTerminated(StreamingEarlyTerminatedPayload {
                step_id: "baby_step_77".into(),
                mode: "fim".into(),
                reason: "lazy_pattern".into(),
                reason_detail: Some("rust_todo_macro".into()),
                token_count_at_term: 17,
                total_tokens_received: 17,
                language_hint: "rust".into(),
            }),
            // B-28 WIRING-1 (SCHEMA_VERSION 33) — pre-assembly readiness row.
            SsbPayload::PipelinePreAssembled(PipelinePreAssembledPayload {
                step_idx: 5,
                pass_rate_bp: 8500,
                was_discarded: false,
                depth_at_time: 1,
                bytes_assembled: 4096,
            }),
            // B-28 WIRING-1 (SCHEMA_VERSION 34) — dep-resolver decision row.
            SsbPayload::DependencyResolved(DependencyResolvedPayload {
                language: "rust".into(),
                name: "tokio".into(),
                decision: "auto_approve".into(),
                reason: "allowlist_match".into(),
                install_command: Some("cargo add tokio".into()),
            }),
            // B-28 WIRING-1 (SCHEMA_VERSION 35) — file-router micro-step row.
            SsbPayload::MicroStepCreated(MicroStepCreatedPayload {
                parent_step_id: 11,
                created_path: "crates/foo/src/bar.rs".into(),
                type_hint: "Module".into(),
                reason: "undefined_symbol".into(),
                count: 1,
            }),
        ]
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn test_typed_payloads() {
        let dir = TempDir::new().expect("tempdir");
        let ssb = Ssb::new(test_config(&dir), test_clock()).expect("ssb init");

        let payloads = all_test_payloads();
        assert_eq!(
            payloads.len(),
            52,
            "must cover original 32 + 2 CHAT + 1 Tier 4 EXECUTION_MODE_CHANGED + 2 B-10 (ContextBudgetComputed + SsbFilteredByOverlap) + 4 B-39 (ReflectTriaged/Candidate/AuditVerdict/Refined) + 6 B-24 4I.C1 (media validator beacons) + 1 B-28 E-3 (GenerationModeSelected) + 1 B-28 E-4 (StreamingEarlyTerminated) + 3 B-28 WIRING-1 (PipelinePreAssembled / DependencyResolved / MicroStepCreated)",
        );

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
                        FindingPayload { severity, category, message, file_path, line, failure_category: None, audit_step: None, few_shot_examples: None }
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
                            fetcher: "reqwest".to_owned(),
                            stage: String::new(),
                            stage_phase: String::new(),
                            stage_payload: None,
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
                ("[a-z0-9-]{1,20}", "[a-z/ ]{1,40}", "[A-Z_]{3,12}",
                 "[a-z0-9-]{1,20}", 0i64..1_000_000_000_000)
                    .prop_map(|(session_id, text, intent, project_id, submitted_at_ms)|
                        SsbPayload::ChatMessage(ChatMessagePayload {
                            session_id, text, intent, project_id, submitted_at_ms,
                        })
                    ),
                ("[a-z0-9-]{1,20}", "(status|error|disambiguation|steering_ack)",
                 "[a-z ]{1,40}", "[a-z0-9-]{1,20}", 0i64..1_000_000_000_000)
                    .prop_map(|(session_id, card_kind, body, project_id, emitted_at_ms)|
                        SsbPayload::ChatResponse(ChatResponsePayload {
                            session_id, card_kind, body, project_id, emitted_at_ms,
                        })
                    ),
                ("(LOCAL|CLOUD|HYBRID)", "(LOCAL|CLOUD|HYBRID)",
                 "[a-z0-9-]{1,20}", 0i64..1_000_000_000_000)
                    .prop_map(|(from_mode, to_mode, project_id, changed_at_ms)|
                        SsbPayload::ExecutionModeChanged(ExecutionModeChangedPayload {
                            from_mode, to_mode, project_id, changed_at_ms,
                        })
                    ),
                // B-10 (DECISION-C) — ContextBudgetComputed + SsbFilteredByOverlap
                // arbitraries. tier kept short; slots/allocations 8-element vec.
                ("[a-z0-9_]{1,30}", 0usize..200000,
                 proptest::collection::vec(0u64..10000, 8..=8),
                 proptest::collection::vec(0usize..50000, 8..=8),
                 0u8..=4)
                    .prop_map(|(tier, raw_max, slots, allocations, adapt_levels_fired)|
                        SsbPayload::ContextBudgetComputed(ContextBudgetComputedPayload {
                            tier, raw_max, slots, allocations, adapt_levels_fired,
                        })
                    ),
                (0usize..1000, 0usize..1000, 0.0f32..1.0)
                    .prop_map(|(entries_total, entries_kept, overlap_threshold)|
                        SsbPayload::SsbFilteredByOverlap(SsbFilteredByOverlapPayload {
                            entries_total, entries_kept, overlap_threshold,
                        })
                    ),
                // B-19A (T10) — hook latency violation arbitrary. hook_name
                // / tier kept short ASCII to mirror the discriminant strings.
                ("(PreToolUse|PostToolUse|Stop|PreCompact)", "(T1|T2|T3|Composite)",
                 0u64..1000, 0u64..10000)
                    .prop_map(|(hook_name, tier, budget_ms, actual_ms)|
                        SsbPayload::HookLatencyViolation(HookLatencyViolationPayload {
                            hook_name: hook_name.to_string(),
                            tier: tier.to_string(),
                            budget_ms,
                            actual_ms,
                        })
                    ),
                // B-39 chat reflect pipeline arbitraries.
                ("[a-z0-9-]{1,20}", "(chat|problem|code_fix)")
                    .prop_map(|(turn_id, class)|
                        SsbPayload::ReflectTriaged(ReflectTriagedPayload {
                            turn_id,
                            class: class.to_string(),
                        })
                    ),
                ("[a-z0-9-]{1,20}", 0u32..1024, "[a-z ]{1,40}")
                    .prop_map(|(turn_id, candidate_index, content)|
                        SsbPayload::ReflectCandidate(ReflectCandidatePayload {
                            turn_id,
                            candidate_index,
                            content,
                        })
                    ),
                ("[a-z0-9-]{1,20}", 0u32..1024, "[a-z ]{1,40}")
                    .prop_map(|(turn_id, best_index, rationale)|
                        SsbPayload::ReflectAuditVerdict(ReflectAuditVerdictPayload {
                            turn_id,
                            best_index,
                            rationale,
                        })
                    ),
                ("[a-z0-9-]{1,20}", 0u32..1024, "[a-z ]{1,40}")
                    .prop_map(|(turn_id, total_tokens_used, synthesis)|
                        SsbPayload::ReflectRefined(ReflectRefinedPayload {
                            turn_id,
                            total_tokens_used,
                            synthesis,
                        })
                    ),
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
