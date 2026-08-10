#![allow(unused_imports)]
// SsbActionType registry: enum + as_sql_str + from_str + decode + validate.
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
use crate::ssb::{schema::*, payloads::*, store::*};

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
    /// B-7 (SCHEMA_VERSION 100): one row per DepthFirstNavigator cursor advance.
    /// Carries a [`TreePositionChangedPayload`] with current_node + visited / blocked
    /// / skipped sets so PlanWindow can highlight the current node + drift trail.
    TreePositionChanged,
    /// B-7 (SCHEMA_VERSION 100): one row per FailureRecord upsert in handle_failure().
    /// Carries a [`FailureRecordedPayload`] mirroring the FailureRecord struct shape
    /// (baby_step_id, step_id, phase_id, concept_id, reason, attempts).
    FailureRecorded,
    /// B-7 (SCHEMA_VERSION 100): one row per PackageRef verification outcome in
    /// research_tree::verify_packages(). Carries a [`PackageRefVerifiedPayload`]
    /// with {name, registry, version, verified} — the verified bool mirrors the
    /// existing PackageRef.verified field.
    PackageRefVerified,
    /// B-7 (SCHEMA_VERSION 100): one row per TechStackDecision creation site.
    /// Carries a [`TechStackDecidedPayload`] with {component, chosen, alternatives,
    /// source} — source is the DecisionSource discriminant (Research|Constraint|Default).
    TechStackDecided,
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
    /// B-30 (SCHEMA_VERSION 36): one-shot LanguageRegistry status emitted at
    /// DKG engine startup, immediately after the per-grammar `catch_unwind`
    /// registration cycle completes. Carries a
    /// [`LanguageGrammarStatusPayload`] with the full loaded + failed lists.
    /// Producers: dkg index init (LanguageRegistry::new() + register() loop).
    /// Consumers: dashboard KnowledgeDrawer LANGUAGES LOADED card.
    LanguageGrammarStatus,
    /// B-29 Phase 5 (SCHEMA_VERSION 37): one row per symbol_* tool call.
    /// Carries a [`SymbolQueryPayload`] with the tool name, target, hop count,
    /// total files reached, and latency in ms.
    /// Producers: translate_tool_request() for symbol_impact / symbol_flow /
    /// symbol_context / symbol_list tool IDs.
    /// Consumers: dashboard KnowledgeDrawer Impact Analysis panel + ActivityFeed.
    SymbolQuery,
    /// B-32 (SCHEMA_VERSION 38): one row per minified file detected during
    /// indexing. Emitted by [`crate::indexing_service::IndexingService::build_index_resumable`]
    /// after the BM25 batch loop completes (post-DKG walkback over indexed
    /// candidates). Carries a [`MinifiedFileChunkedPayload`] with the
    /// relative path, byte count, chunk count, and computed average line
    /// length. Defense-in-depth alongside the bm25.rs `tracing::info!`
    /// emit — the SSB row persists for dashboard surfacing, the tracing
    /// log keeps dev-console visibility.
    MinifiedFileChunked,
    /// B-31.5 (SCHEMA_VERSION 39): per-batch checkpoint emitted by
    /// [`crate::indexing_service::IndexingService::build_index_resumable`].
    /// Carries an [`IndexCheckpointPayload`] with the project_id, current
    /// batch counters, file count, and last_checkpoint_at. Replaces the
    /// `tracing::info!(target="ssb_pending", ...)` placeholders from
    /// Cycle B.
    IndexCheckpoint,
    /// B-31.5 (SCHEMA_VERSION 39): emitted by `build_index_resumable` when it
    /// returns [`crate::indexing_service::IndexResult::Resumed`], or by
    /// [`crate::startup::auto_resume_indexed_projects`] when a resumed run
    /// completes. Carries an [`IndexResumePayload`] with the project_id,
    /// `resumed_from_batch`, and `files_skipped` counters.
    IndexResume,
    /// B-31.5 (SCHEMA_VERSION 39): emitted by `build_index_resumable` when it
    /// returns [`crate::indexing_service::IndexResult::Cancelled`], or by
    /// [`crate::startup::graceful_shutdown`] when it requests cancellation
    /// of an in-progress project. Carries an [`IndexCancelPayload`] with
    /// the project_id and partial-state counters.
    IndexCancel,
    /// B-31.5 (SCHEMA_VERSION 39): emitted by `IndexLock::try_acquire` (when
    /// invoked through the optional SSB-writer path) on successful
    /// acquisition. Carries a [`LockAcquiredPayload`] with the project_id,
    /// scope, and `holder_pid`.
    LockAcquired,
    /// B-31.5 (SCHEMA_VERSION 39): emitted by `IndexLock::try_acquire` (when
    /// invoked through the optional SSB-writer path) on
    /// [`crate::lock::IndexLockError::AlreadyHeld`]. Carries a
    /// [`LockContestedPayload`] mirroring the AlreadyHeld error payload —
    /// the contender pid is the current process; `holder_pid` is the live
    /// holder.
    LockContested,
    /// B-33 follow-up (SCHEMA_VERSION 40): one row per successful symbol-graph
    /// build emitted by [`crate::indexing_service::IndexingService::build_index_resumable`]
    /// after [`elai_knowledge::symbol_graph::build_pipeline::build_symbol_graph`]
    /// returns Ok with a non-empty file count. Carries a
    /// [`SymbolGraphUpdatedPayload`] with the project id, file/symbol/edge
    /// counts, and the build wall-clock millisecond timestamp.
    /// Producers: IndexingService post-DKG symbol-graph build branch.
    /// Consumers: dashboard-v2 GraphWindow event listener (replaces the
    /// 30s polling fallback shipped in B-33 D3).
    SymbolGraphUpdated,
    /// B-40 (SCHEMA_VERSION 41): emitted once when the FRAMING substate is
    /// entered (PLANNING + agent_substate=Framing). Carries an
    /// [`IdeaFramingStartedPayload`] with the raw user prompt + session
    /// timestamp. Producer: [`crate::idea_framing::frame_idea`].
    /// Consumer: dashboard-v2 IdeaFramingWindow.svelte initial render.
    IdeaFramingStarted,
    /// B-40 (SCHEMA_VERSION 41): emitted by
    /// [`crate::idea_framing::classify_project_class`] after the regex
    /// classifier returns a [`crate::idea_framing::ProjectClass`].
    /// Carries a [`ProjectClassClassifiedPayload`] with the snake_case class
    /// name + matched_pattern hint (None for `Other`).
    ProjectClassClassified,
    /// E-11 Task 8: advisory emitted when a brief matches a known demo-vs-deploy chasm.
    ScopeCeilingAdvisory,
    /// B-40 (SCHEMA_VERSION 41): emitted by
    /// [`crate::idea_framing::run_critic_loop`] at the end of each round
    /// (1..=3). Carries a [`CriticRoundCompletePayload`] with round_index
    /// + per-category finding counts + total BLOCK/WARN/INFO tallies +
    /// converged flag (true when zero BLOCK findings remain).
    CriticRoundComplete,
    /// B-40 (SCHEMA_VERSION 41): emitted by
    /// [`crate::idea_framing::frame_idea`] after persisting framed-idea.toml.
    /// Carries an [`IdeaFramedPayload`] with the artifact path + project_class
    /// + rounds_used + tradeoff_count. Consumed by the 7th AutoAdvanceEvaluator
    /// evidence check `derive_framing_evidence`.
    IdeaFramed,
    /// B-40 (SCHEMA_VERSION 41): emitted by
    /// [`crate::idea_framing::frame_idea`] when the TradeoffCard is ready
    /// for user response (surfaced to dashboard IdeaFramingWindow).
    /// Carries a [`TradeoffCardEmittedPayload`] with tradeoff names +
    /// option counts.
    TradeoffCardEmitted,
    /// B-34 (SCHEMA_VERSION 42): one row per `entry_points` tool call.
    /// Carries an [`EntryPointsDetectedPayload`] with the project_id, total
    /// count, and per-reason breakdown so the dashboard can render an
    /// entry-point summary card without re-running detection.
    EntryPointsDetected,
    /// B-34 (SCHEMA_VERSION 42): one row per ArtifactIndexer completion.
    /// Carries an [`ArtifactsIndexedPayload`] with indexed/up_to_date artifact
    /// names, total_chunks, and latency_ms. Produced by Phase C indexer wiring.
    ArtifactsIndexed,
    /// B-34 (SCHEMA_VERSION 42): one row per `search_artifacts` tool call.
    /// Carries an [`ArtifactSearchedPayload`] with the query, optional
    /// artifact_name filter, result_count, and latency_ms.
    ArtifactSearched,
    /// B-34 (SCHEMA_VERSION 42): one row per stale artifact detected by the
    /// ArtifactIndexer content-hash check. Carries an [`ArtifactStalePayload`]
    /// with artifact_name and the list of changed files.
    ArtifactStale,
    /// B-38 (SCHEMA_VERSION 43): one row per `AmbiguityScorer.score()`
    /// completion. Carries an [`AmbiguityScoredPayload`].
    AmbiguityScored,
    /// B-38 (SCHEMA_VERSION 43): one row per GroundingSweepGate verdict
    /// where one or more per-dimension floors were breached. Carries an
    /// [`AmbiguityFloorFailedPayload`].
    AmbiguityFloorFailed,
    /// B-38 (SCHEMA_VERSION 43): one row per qualifying-but-streak-incomplete
    /// gate verdict. Carries an [`AmbiguityStreakAdvancedPayload`].
    AmbiguityStreakAdvanced,
    /// B-42 (SCHEMA_VERSION 44): one row per cosmos refresh-loop tick start.
    /// Carries a [`CosmosRefreshStartedPayload`].
    CosmosRefreshStarted,
    /// B-42 (SCHEMA_VERSION 44): one row per successful cosmos artifact refresh.
    /// Carries a [`CosmosArtifactUpdatedPayload`].
    CosmosArtifactUpdated,
    /// B-42 (SCHEMA_VERSION 44): one row when a cosmos artifact exceeds
    /// 3 * cadence_interval since last refresh. Carries a
    /// [`CosmosArtifactDecayedPayload`].
    CosmosArtifactDecayed,
    /// B-42 (SCHEMA_VERSION 44): one row per `CosmosCheck` verifier dispatch.
    /// Carries a [`CosmosGroundingUsedPayload`].
    CosmosGroundingUsed,
    /// B-41 (SCHEMA_VERSION 45): per-level auditor mode spawn at a tier node.
    /// Carries a [`PerLevelAuditStartedPayload`].
    PerLevelAuditStarted,
    /// B-41 (SCHEMA_VERSION 45): per-level auditor verdict produced.
    /// Carries a [`PerLevelAuditVerdictPayload`].
    PerLevelAuditVerdict,
    /// B-41 (SCHEMA_VERSION 45): per-level auditor cascade (2nd BLOCK at a
    /// tier forces parent-tier respawn). Carries a [`PerLevelAuditCascadePayload`].
    PerLevelAuditCascade,
    /// B-37C (SCHEMA_VERSION 46): one row per runtime fallback chain step
    /// emitted by the `RuntimeOrchestrator` when a primary backend is
    /// unavailable / fails. Orthogonal to `ModelSwap` (which records model
    /// substitution within a backend). Carries a [`RuntimeFallbackPayload`].
    RuntimeFallback,
    /// B-37C (SCHEMA_VERSION 46): one row per per-backend `RuntimeHealth`
    /// transition observed by `RuntimeOrchestrator::health_snapshot`.
    /// Carries a [`RuntimeHealthChangedPayload`].
    RuntimeHealthChanged,
    /// B-37C (SCHEMA_VERSION 46): one row per CLI adapter subprocess spawn
    /// (B-37B cli_common::spawn). Carries a [`CliSubprocessSpawnedPayload`]
    /// with the redacted argv (secrets stripped).
    CliSubprocessSpawned,
    /// B-37C (SCHEMA_VERSION 46): one row per CLI adapter subprocess kill
    /// (B-37B cli_common::kill_and_wait, idle timeout, or natural exit).
    /// Carries a [`CliSubprocessKilledPayload`] with exit code + reason.
    CliSubprocessKilled,
    /// E-51 (SCHEMA_VERSION 96): vllm-mlx subprocess spawned by MlxAdapter.
    /// Carries an [`MlxSubprocessSpawnedPayload`].
    MlxSubprocessSpawned,
    /// E-51 (SCHEMA_VERSION 96): vllm-mlx subprocess killed by MlxAdapter.
    /// Carries an [`MlxSubprocessKilledPayload`].
    MlxSubprocessKilled,
    /// E-51 (SCHEMA_VERSION 96): llama-server subprocess spawned by LlamaCppAdapter.
    /// Carries a [`LlamaCppSubprocessSpawnedPayload`].
    LlamaCppSubprocessSpawned,
    /// E-51 (SCHEMA_VERSION 96): llama-server subprocess killed by LlamaCppAdapter.
    /// Carries a [`LlamaCppSubprocessKilledPayload`].
    LlamaCppSubprocessKilled,
    /// E-51 (SCHEMA_VERSION 96): Apple Silicon platform detected by MlxAdapter.
    /// Emitted once per session on first detect_mlx_capable() call returning AppleSilicon.
    /// Carries an [`MlxApplePlatformDetectedPayload`].
    MlxApplePlatformDetected,
    /// E-51 (SCHEMA_VERSION 96): llama.cpp execution provider detected.
    /// Emitted once per session on detect_llamacpp_ep() call.
    /// Carries a [`LlamaCppEpDetectedPayload`].
    LlamaCppEpDetected,
    /// E-51 (SCHEMA_VERSION 96): runtime chain built at CLI bootstrap.
    /// Emitted by resolve_runtime_backend_from_env() on every CLI start.
    /// Carries a [`RuntimeChainBuiltPayload`].
    RuntimeChainBuilt,
    /// E-17 SP2 (SCHEMA_VERSION 97): emitted once per DEEP-tier query when
    /// the deep candidate pool cap dropped one or more input candidates.
    /// Carries a [`DeepPoolCappedPayload`].
    DeepPoolCapped,
    /// E-17 SP2 (SCHEMA_VERSION 97): emitted once per DEEP-tier query when
    /// the RRF quality gate dropped one or more candidates below the floor.
    /// Carries an [`RrfQualityGateFiredPayload`].
    RrfQualityGateFired,
    /// E-17 SP2 (SCHEMA_VERSION 97): emitted when `should_use_hyde` returns
    /// false on a CodeSymbol or Mixed query that would otherwise have
    /// triggered HyDE. Carries a [`HydeSkippedCodeQueryPayload`].
    HydeSkippedCodeQuery,
    /// E-17 SP2 (SCHEMA_VERSION 97): emitted once per query with the
    /// per-ranker RRF weights active for that fusion run. Carries an
    /// [`RrfComponentWeightsPayload`].
    RrfComponentWeights,
    /// C-1 (SCHEMA_VERSION 47): one row per `emit_reasoning` call when a
    /// `PipelineGrc` handle was plumbed through `ResolvedConfig`. Replaces
    /// the synthetic-constant `ReasoningTrace` row with real
    /// `budget_remaining` (from `BasicGrcEngine::get_budget`) and
    /// `agent_id` (from the engine's `AgentGrcConfig.role`). Carries a
    /// [`GrcStepEmittedPayload`].
    GrcStepEmitted,
    /// C-1 (SCHEMA_VERSION 47): one row per `PipelineGrc::ground_plan`
    /// call in `task_runner::drive_task_inner`. Emitted ONCE per call
    /// (not per gate iteration) so the dashboard renders one sweep row
    /// per plan grounding pass. Carries a [`GroundingSweepResultPayload`].
    GroundingSweepResult,
    /// C-1 (SCHEMA_VERSION 47): one row per `PipelineGrc::generate_csq`
    /// completion in `task_runner::drive_task_inner`. Replaces the
    /// TODO(B-future) comment at the pre-tool CSQ branch. stub-ban-allow: historical reference, not a stub
    /// Carries a [`CsqEvaluatedPayload`].
    CsqEvaluated,
    /// C-1 (SCHEMA_VERSION 47): one row per `BasicGrcEngine::advance`
    /// returning `GrcError::InsufficientFindings` (auditor attempted to
    /// Converge with zero `FINDING:` rows). Surfaced at the `PipelineGrc`
    /// caller in `task_runner`, NOT inside `engine.rs`, because the
    /// engine has no SSB hook (GAP-2 in explorer-C1.md). Carries an
    /// [`AuditorRubberStampRejectedPayload`].
    AuditorRubberStampRejected,
    /// C-37 (SCHEMA_VERSION 106): one row per agent spawn whose role has
    /// an explicit per-role decoding policy in
    /// `elai_providers::decoding::PER_ROLE_DECODING_DEFAULTS`. Closes
    /// GAP-B (default_decoding_for had no production callers pre-C-37);
    /// the emission site is `agent_pool::spawn` and the payload mirrors
    /// the `ConstrainedDecodingConfig` translation. Carries a
    /// [`PerRoleDecodingAppliedPayload`].
    PerRoleDecodingApplied,
    /// C-1 (SCHEMA_VERSION 47): one row per `HydeStage::hypothesize`
    /// call. `cache_hit=true` rows carry latency_ms of the cache lookup
    /// only; `cache_hit=false` rows carry total sidecar + cache-insert
    /// latency. Carries a [`HydeQueryPayload`].
    HydeQuery,
    /// C-1 (SCHEMA_VERSION 47): aggregate vector-cache hit/miss snapshot
    /// emitted by `VectorCache::flush_hit_rate_ssb` after a query
    /// pipeline run. Hits/misses counters reset atomically per flush.
    /// Carries a [`VectorCacheHitRatePayload`].
    VectorCacheHitRate,
    /// C-1 (SCHEMA_VERSION 47): per-query trace emitted at the end of
    /// `QueryPipeline::query_with_tier`. Pairs with the existing
    /// `tracing::debug!("query_trace")` event but persists into SSB so
    /// the dashboard recentTraces panel can read live rows. Carries a
    /// [`QueryTracePayload`].
    QueryTrace,
    /// C-2 (SCHEMA_VERSION 48): emitted by the task_runner baby-step loop
    /// the first time `nav.next_baby_step` returns a baby-step whose
    /// `tree.location_index[bs_id].phase_id` differs from the previously
    /// seen phase id. Carries a [`PhaseNodeReadyPayload`].
    PhaseNodeReady,
    /// C-2 (SCHEMA_VERSION 48): emitted by the task_runner baby-step loop
    /// when the resolved `tree.location_index[bs_id].step_id` differs from
    /// the previously seen step id. Carries a [`StepNodeReadyPayload`].
    StepNodeReady,
    /// C-2 (SCHEMA_VERSION 48): emitted once per baby-step consumption in
    /// the task_runner loop. Carries a [`BabyStepReadyPayload`].
    BabyStepReady,
    /// C-2 (SCHEMA_VERSION 48): emitted by `pipeline_builder` on the
    /// `RepairOutcome::Escalated` path when the DAG repair loop exhausts
    /// `T_DAG = 10` retries. Carries a [`DagRepairEscalatedPayload`].
    DagRepairEscalated,
    /// C-2 (SCHEMA_VERSION 48): mirror of [`Self::AgentLifecycle`] surfaced
    /// for the dashboard `AGENTS` panel. Single `state` field (no
    /// old_lifecycle); consumers derive transitions by reading the ordered
    /// stream per agent_id (DRIFT-5 in context-package-C2.md). Carries an
    /// [`AgentStatusChangedPayload`].
    AgentStatusChanged,
    /// C-2 (SCHEMA_VERSION 48): C-10 anti-spiral detector — spinning pattern
    /// (same step hash 3+ times). Variant defined now so the SCHEMA bump
    /// is atomic; producer wiring lands in C-10. Carries a
    /// [`StagnationSpinningDetectedPayload`].
    StagnationSpinningDetected,
    /// C-2 (SCHEMA_VERSION 48): C-10 anti-spiral detector — period-2
    /// oscillation pattern. Definition-only in C-2; producer wiring in
    /// C-10. Carries a [`StagnationOscillationDetectedPayload`].
    StagnationOscillationDetected,
    /// C-2 (SCHEMA_VERSION 48): C-10 persona switch — lateral thinking
    /// persona activated to break a stagnation loop. Definition-only in
    /// C-2; producer wiring in C-10. Carries a
    /// [`LateralThinkingActivatedPayload`].
    LateralThinkingActivated,
    /// C-2 (SCHEMA_VERSION 48): C-10 goal-drift canary verdict. Emitted
    /// when the canary detects that the active baby-step has drifted from
    /// the original task intent. Definition-only in C-2; producer wiring
    /// in C-10. Carries a [`GoalDriftWarningPayload`].
    GoalDriftWarning,
    /// C-2 (SCHEMA_VERSION 48): C-10 generation watchdog timeout. Emitted
    /// when an agent exceeds its per-step wall-clock budget without
    /// producing observable output. Definition-only in C-2; producer
    /// wiring in C-10. Carries a [`WatchdogTimeoutPayload`].
    WatchdogTimeout,
    /// C-7 (SCHEMA_VERSION 49): reviewer verdict on a proposed change. Emitted
    /// by the dashboard-v2 Tauri commands `accept_change` / `decline_change` /
    /// `retry_audit` each time a human reviewer acts on a change row in
    /// ReviewWindow. Carries a [`ChangeDecisionPayload`].
    ChangeDecision,
    /// C-8 (SCHEMA_VERSION 50): DegradationCascade verdict bridge.
    /// Emitted by the live_cli startup-readiness path AND by the
    /// dashboard-v2 Tauri command `set_search_mode`. Carries a
    /// [`SearchModeChangedPayload`]. The dashboard `get_search_mode`
    /// command reads the latest row of this type (graceful empty
    /// fallback to "hybrid"). Cross-process SSB bridge between the
    /// CLI AppState and the dashboard process (no shared memory).
    SearchModeChanged,
    /// C-5 (SCHEMA_VERSION 51): per-call privacy redaction event.
    /// Emitted by [`crate::cloud_mode::RedactingBabyStepExecutor`] each
    /// time the privacy redactor scrubs PII or secrets from a
    /// cloud-bound baby-step payload. Carries a
    /// [`RedactionAppliedPayload`] with `count` (number of substitutions
    /// applied across all redacted fields) and `fields` (the logical
    /// field names touched, e.g. `["task_summary", "escalation_message"]`).
    /// Consumer: dashboard CloudPanel.svelte privacy-event feed (last 5
    /// redactions). The row is best-effort: a write failure never aborts
    /// the baby-step execution.
    RedactionApplied,
    /// C-10 (SCHEMA_VERSION 52): lateral-thinking persona switch
    /// succeeded — emitted when a previously-active stagnation pattern
    /// no longer fires after a persona suggestion was consumed.
    /// Carries a [`LateralThinkingSucceededPayload`]. Producer wiring:
    /// task_runner step-boundary detector (anti_spiral_extended.rs).
    LateralThinkingSucceeded,
    /// C-10 (SCHEMA_VERSION 52): lateral-thinking exhausted — all 5
    /// personas have been tried within the current task window without
    /// a pattern break. Carries a [`LateralThinkingExhaustedPayload`].
    LateralThinkingExhausted,
    /// C-10 (SCHEMA_VERSION 52): goal-drift Block verdict — emitted
    /// when the GoalDriftCanary verdict crosses the Block threshold.
    /// Companion to [`Self::GoalDriftWarning`] (Yellow band). Carries a
    /// [`GoalDriftBlockPayload`].
    GoalDriftBlock,
    /// C-13 (SCHEMA_VERSION 53): WIP-commit squash completion beacon.
    /// Emitted by `crate::wip_squash::squash_wip_commits_if_any` at the
    /// post-AUDITING -> MEMORIZING boundary AND by the `elai wip-squash`
    /// CLI subcommand. Carries `wip_count` (leading-WIP-commit count
    /// observed) + `squashed` (true ⇒ `git reset --soft HEAD~N` + commit
    /// fired; false ⇒ no-op because count <= 5). Producer wiring:
    /// task_runner audit-loop boundary + cli/wip_squash_subcommand.rs.
    /// Consumer: dashboard activity feed.
    WipSquashComplete,
    /// C-22 (SCHEMA_VERSION 54): per-NovelStep alternative-explored row.
    /// Emitted by `DefaultFeasibilityProber::probe` once per NovelStep
    /// alternative attempted, carrying step_id + alternative_index.
    /// Consumer: dashboard FeasibilityPanel alternative trail.
    AlternativeExplored,
    /// C-22 (SCHEMA_VERSION 54): feasibility-probe escalation row.
    /// Emitted when `decide_with_mode` returns
    /// `FeasibilityDecision::Escalate`. Carries reason + variant tag.
    /// Distinct from the gate-level `escalation_fired` row.
    FeasibilityEscalated,
    /// C-23 (SCHEMA_VERSION 55): SessionDag branch fork beacon. Emitted
    /// at the orchestrator level whenever `SessionDag::create_branch`
    /// returns Ok via the new `create_branch_with_ssb` wrapper (mirror
    /// of `merge_branch_with_ssb`). Also emitted by the dashboard-v2
    /// `fork_session_branch` Tauri command which opens the DAG per call.
    /// Carries [`BranchForkedPayload`] (branch_name + tip_snap_id +
    /// parent_snap_id).
    BranchForked,
    /// C-23 (SCHEMA_VERSION 55): SessionDag branch merge beacon.
    /// Supersedes the legacy Checkpoint-with-discriminator emit at
    /// `merge_branch_with_ssb`. Carries [`BranchMergedPayload`]
    /// (winner_name + trunk_snap_id + abandoned_names).
    BranchMerged,
    /// C-23 (SCHEMA_VERSION 55): SessionDag branch close beacon. Emitted
    /// by the new `close_branch_with_ssb` wrapper. Carries
    /// [`BranchClosedPayload`] (branch_name + reason).
    BranchClosed,
    /// C-25 (SCHEMA_VERSION 56): plan-skeleton emission row. Emitted by
    /// `pipeline_builder::PipelineBuilder::build_with_emitter` ONCE per
    /// factory entry. Carries [`PlanSkeletonPayload`] (plan_id, step_count,
    /// concept_class, plan_skeleton_hash, source). C-25 is the FIRST emitter
    /// of this row (see C-25 DRIFT-D — no producer existed pre-bump).
    PlanSkeleton,
    /// C-25 (SCHEMA_VERSION 56): compiled-pipeline cache-hit + run beacon.
    /// Emitted by `compiled_pipeline::CompiledExecutor::try_execute` on
    /// successful pipeline-cache resolution and run. Carries
    /// [`CompiledPipelineFiredPayload`] (pipeline_id, concept_class,
    /// use_count, plan_skeleton_hash).
    CompiledPipelineFired,
    /// C-25 (SCHEMA_VERSION 56): template-crystallizer success beacon.
    /// Emitted by `compiled_pipeline::TemplateCrystallizer::check_crystallization`
    /// when the threshold (N=3 consecutive successes) is met and a new
    /// CompiledPipeline is crystallized. Carries
    /// [`TemplateCrystallizedPayload`] (concept_class, plan_skeleton_hash,
    /// pipeline_id, threshold_n).
    TemplateCrystallized,
    /// C-25 (SCHEMA_VERSION 56): background-task spawn beacon. Emitted by
    /// `background_task::BackgroundTaskManager::gated_spawn` on a
    /// successful (autonomous_mode=true) spawn. Carries
    /// [`BackgroundTaskStartedPayload`] (task_id, command_summary,
    /// soft_timeout_secs, hard_timeout_secs).
    BackgroundTaskStarted,
    /// C-25 (SCHEMA_VERSION 56): background-task completion beacon.
    /// Emitted by `background_task::BackgroundTaskManager::poll_completed`
    /// when a handle finishes (normal exit) AND by `enforce_timeouts`
    /// when a handle is auto-detached at the soft boundary. Carries
    /// [`BackgroundTaskCompletePayload`] (task_id, exit_status,
    /// duration_secs, auto_detached, timed_out).
    BackgroundTaskComplete,
    /// C-25 audit follow-up (SCHEMA_VERSION 57): compiled-pipeline
    /// decompile beacon. Emitted by
    /// `compiled_pipeline::CompiledExecutor::emit_decompile_triggered`
    /// whenever a cache HIT is rejected via the confidence floor or the
    /// consecutive-failure threshold. Carries
    /// [`DecompileTriggeredPayload`] (concept_class, plan_skeleton_hash,
    /// failure_count, trigger_reason). Closes the WARN-1 gap from
    /// audit-C25.md where decompile events were routed through
    /// `AntiSpiralHit` and dashboard filters keyed on `decompile_triggered`
    /// missed every decompile.
    DecompileTriggered,
    /// C-26 (SCHEMA_VERSION 58): NavigatorTree futility — same-error-class
    /// window saturates at `futility_same_error_threshold` (default 3).
    /// Carries [`FutilityDetectedPayload`].
    FutilityDetected,
    /// C-26 (SCHEMA_VERSION 58): NavigatorTree futility — output-similarity
    /// path. Triggered when GoalDriftCanary computes a Jaccard similarity
    /// > 0.90 between two consecutive baby-step outputs. Carries
    /// [`FutilityOutputStagnationPayload`] (includes the persona switch
    /// label per C-10 D4 affinity table).
    FutilityOutputStagnation,
    /// C-27 (SCHEMA_VERSION 59): Testing-trust T-0 — DkgReachabilityAuditor
    /// successfully verified that a produces_symbol declared in a
    /// WIRING_CONTRACT block is reachable from a configured
    /// [wiring.entrypoints]. Carries [`WiringEvidencePayload`].
    WiringEvidence,
    /// C-27 (SCHEMA_VERSION 59): Testing-trust T-0 — any of StubBanAuditor /
    /// DkgReachabilityAuditor / ObservableEffectRunner reported a violation.
    /// Carries [`WiringViolationPayload`] (violation_kind discriminates the
    /// three branches: StubPattern | UnreachableSymbol | BrokenObservable).
    WiringViolation,
    /// C-27 (SCHEMA_VERSION 59): Testing-trust T(-1) — DriftComparator
    /// observed divergent ActionTypeSequence at index N between two
    /// nominally-equivalent runs (same seed, same hardware_class). Carries
    /// [`DeterminismDriftPayload`]. Identical sequences emit zero rows
    /// (stub_ban contract: zero false positives).
    DeterminismDrift,
    /// C-28 (SCHEMA_VERSION 60): Testing-trust T-1 mutation testing —
    /// `MutationRunner` produced a `MutantOutcome::Survived` for a generated
    /// mutant. Emitted once per surviving mutant by the Step 6
    /// `DeterministicAuditor` mutation check (runs only after Build /
    /// VersionGate / FileSizeCensus / LazyCode / Completeness all pass).
    /// Carries [`MutationSurvivedPayload`] (file, line, mutant_kind).
    /// Caught / Timeout / Skipped outcomes emit ZERO rows; the
    /// `mutation_score` gate is enforced via a separate
    /// `DeterministicFinding::Block` finding.
    MutationSurvived,
    /// C-29 (SCHEMA_VERSION 61): Testing-trust Tier T-2 — emitted when a
    /// `BabyStepSpec.intent_pairs` set crosses the
    /// `IntentConfig.min_pairs` threshold (default 3). The
    /// `IntentPairsGate` (PLANNING -> TESTING_FIRST) emits one row per
    /// confirmation; the autonomous gate auto-confirm branch emits with
    /// `confirmed_by="autonomous_gate"`. Carries
    /// [`IntentConfirmedPayload`].
    IntentConfirmed,
    /// C-29 (SCHEMA_VERSION 61): Testing-trust Tier T-3 — `MetamorphicTestRunner`
    /// observed a relation violation (one of identity / inverse /
    /// permutation_invariance). Emitted by `deterministic_auditor` Step 7
    /// (post-Mutation). Carries [`MrViolatedPayload`] with a non-empty
    /// `counterexample` (debug_assert + runtime sentinel fallback, see
    /// `metamorphic_testing::MrResult::failed` + DRIFT-C29-11). MrViolated
    /// is NOT a standalone Block — score aggregation deferred to T-4 per
    /// C-29 plan §13(c).
    MrViolated,
    /// C-30 (SCHEMA_VERSION 62): Surgical Mode — one row per successful
    /// `SurgicalDecomposer::decompose` return. Carries a
    /// [`SurgicalPlanCreatedPayload`].
    SurgicalPlanCreated,
    /// C-30 (SCHEMA_VERSION 62): Surgical Mode — one row per decompose call
    /// whose AffectedSet.total_loc_estimate exceeds 500. Carries a
    /// [`SurgicalLocWarningPayload`] (loc + threshold + mode).
    SurgicalLocWarning,
    /// C-30 (SCHEMA_VERSION 62): Oracle Mode — one row per
    /// `OracleAnswerCache::get` hit. Carries an [`OracleCacheHitPayload`]
    /// (query_hash_hex + source_tier + cached_age_secs).
    OracleCacheHit,
    /// C-30 (SCHEMA_VERSION 62): Oracle Mode — one row per
    /// `OracleAnswerCache::insert` (preceded by a cache miss). Carries an
    /// [`OracleCacheMissPayload`] (query_hash_hex + confidence).
    OracleCacheMiss,
    /// C-30 (SCHEMA_VERSION 62): one row per signal evaluated by the
    /// fresh-project detector inside the first-task gate. Carries a
    /// [`ProjectDetectionSignalPayload`] (signal_name + weight + fired).
    ProjectDetectionSignal,
    /// C-32 (SCHEMA_VERSION 63): one row per
    /// `SkeletonGenerator::generate_for_functions` Ok return. Carries a
    /// [`SkeletonGeneratedPayload`] (blank_count, language, file_path).
    SkeletonGenerated,
    /// C-32 (SCHEMA_VERSION 63): one row per
    /// `SkeletonGenerator::flush_to_disk` success. Carries a
    /// [`SkeletonFlushedPayload`] (path, completeness_ratio).
    SkeletonFlushed,
    /// C-32 (SCHEMA_VERSION 63): one row per `MiniCritic::check`. Carries a
    /// [`MiniCriticVerdictPayload`] (blank_id, verdict, reason, language).
    MiniCriticVerdict,
    /// C-32 (SCHEMA_VERSION 63): one row per
    /// `StructuralRetrySequencer` stage invocation. Carries a
    /// [`StructuralRetryStageAttemptPayload`] (blank_id, stage, attempt,
    /// outcome).
    StructuralRetryStageAttempt,
    /// C-32 (SCHEMA_VERSION 63): one row per
    /// `BridgeFileTemplate::write_bridge` success. Carries a
    /// [`BridgeFileGeneratedPayload`] (kind, path, module_count).
    BridgeFileGenerated,
    /// D-1 (SCHEMA_VERSION 64): one row per [`crate::archive_chain::ArchiveChain::append`]
    /// call. Carries an [`ArchiveEntryAddedPayload`] (template_id, content_hash,
    /// parent_hash, domain, pattern_kind, archive_status).
    ArchiveEntryAdded,
    /// D-1 (SCHEMA_VERSION 64): one row per template retirement transition.
    /// Carries an [`ArchiveEntryRetiredPayload`] (template_id, retired_at_ms,
    /// retirement_reason).
    ArchiveEntryRetired,
    /// D-1 (SCHEMA_VERSION 64): one row per BAI Thompson promotion (D-5
    /// producer lands later; wire surface added now for atomic schema bump).
    /// Carries a [`TemplatePromotedPayload`] (template_id, stage, pass_rate).
    TemplatePromoted,
    /// D-1 (SCHEMA_VERSION 64): one row per Candidate rejection. Carries a
    /// [`TemplateCandidateRejectedPayload`] (template_id, rejection_reason).
    TemplateCandidateRejected,
    /// E-5 (SCHEMA_VERSION 65): one row per Mem0-style extraction emitting
    /// a new currently-valid `memory_facts` row (the producer wrapper in
    /// the orchestrator's per-turn extraction hook emits this row right
    /// after `FactStore::insert` returns Ok). Carries a
    /// [`MemoryFactExtractedPayload`] (fact_id, fact_type, scope,
    /// trust_score, key).
    MemoryFactExtracted,
    /// E-5 (SCHEMA_VERSION 65): one row per `FactStore::supersede` Ok return —
    /// emitted after the bi-temporal close + new-row insert pair completes
    /// inside a single sqlite transaction. Carries a
    /// [`MemoryFactSupersededPayload`] (old_fact_id, new_fact_id, valid_to_ms).
    MemoryFactSuperseded,
    /// D-2 (SCHEMA_VERSION 66): one row per
    /// [`crate::ppm_client::PpmClient::score`] call that clears the
    /// configured `min_score_threshold`. Carries a [`PpmScoredPayload`]
    /// (candidate_id, verifier_id, mode, score_mean, score_var, num_steps,
    /// latency_ms, cache_hit).
    PpmScored,
    /// D-2 (SCHEMA_VERSION 66): one row per PpmClient::score call whose
    /// `score_mean` is below the configured threshold. Carries a
    /// [`PpmGatingFailedPayload`] (candidate_id, verifier_id, mode,
    /// score_mean, threshold, num_steps).
    PpmGatingFailed,
    /// E-1 (SCHEMA_VERSION 67): one row per
    /// `elai_reasoning::chat_memory_reflect::reflect()` Ok return —
    /// emitted by the orchestrator's chat-turn-committed hook right
    /// after the reflect pass completes. Carries a
    /// [`ChatReflectionCompletedPayload`] (note_id, note_preview,
    /// facts_extracted, decay_rows_examined, decay_rows_updated,
    /// core_block_bytes).
    ChatReflectionCompleted,
    /// E-1 (SCHEMA_VERSION 67): one row per SLEEP_COMPUTE substate
    /// entry / exit. Carries a [`MemoryEvolutionRunPayload`] (phase
    /// `"start"` | `"end"`, duration_ms, notes_touched).
    MemoryEvolutionRun,
    /// E-1 dashboard slice (SCHEMA_VERSION 68): one row per
    /// `elai_memory::NoteStore::insert` Ok return — emitted by the
    /// orchestrator's chat-turn-committed hook (producer wiring lives in
    /// task_runner.rs and is owned by another session). Carries a
    /// [`MemoryNoteInsertedPayload`] (note_id, scope, content_blake3,
    /// link_count).
    MemoryNoteInserted,
    /// E-1 dashboard slice (SCHEMA_VERSION 68): one row per A-MEM note
    /// evolution rewrite (PPR-walker-attached link/edit pass). Carries a
    /// [`MemoryNoteEvolvedPayload`] (note_id, old_content_blake3,
    /// new_content_blake3, links_added, links_removed).
    MemoryNoteEvolved,
    /// E-1 dashboard slice (SCHEMA_VERSION 68): one row per duplicate /
    /// contradiction pair the conflict adjudicator emits. Carries a
    /// [`MemoryConflictDetectedPayload`] (fact_id_a, fact_id_b,
    /// conflict_kind: `"duplicate"` | `"contradiction"`).
    MemoryConflictDetected,
    /// E-1 dashboard slice (SCHEMA_VERSION 68): one row per
    /// `CoreBlockStore::upsert` that replaces an existing row (vs first
    /// insert). Carries a [`CoreBlockReplacedPayload`] (user_id,
    /// old_blake3, new_blake3, bytes).
    CoreBlockReplaced,
    /// D-3 (SCHEMA_VERSION 69): one row per
    /// [`crate::agent_pool::run_consensus_pass`] Ok return whose verdict is
    /// `Pass`. Carries a [`VerifierConsensusPassPayload`] (candidate_id,
    /// template_id, verdict, verifier_count, pass_count, rule_id).
    VerifierConsensusPass,
    /// D-3 (SCHEMA_VERSION 69): one row per
    /// [`crate::agent_pool::run_consensus_pass`] Ok return whose verdict is
    /// `Fail` / `Inconclusive`. Carries a [`VerifierConsensusFailPayload`]
    /// (candidate_id, template_id, failing_verifier_ids, reason).
    VerifierConsensusFail,
    /// D-4 (SCHEMA_VERSION 70): one row per
    /// [`crate::tripwire::TripwireMonitor::observe`] firing. Carries a
    /// [`TripwireFiredPayload`] (event_id, tripwire_kind, candidate_id,
    /// observed_value, threshold, fired_at_ms).
    TripwireFired,
    /// D-4 (SCHEMA_VERSION 70): one row per
    /// [`crate::tripwire::auto_revert`] success. Carries a
    /// [`TemplateAutoRevertedPayload`] (template_id, parent_template_id,
    /// retirement_reason, fired_tripwire_id).
    TemplateAutoReverted,
    /// D-4 GAP-2 (SCHEMA_VERSION 108): one row per concurrent-detector
    /// firing tick (>=2 detectors tripping in the same
    /// `TripwireMonitor::observe` call). Distinct from N independent
    /// `TripwireFired` rows so D-26 concurrent-fire diagnostics can
    /// branch on the wire tag rather than join raw rows. Carries a
    /// [`TripwireFiredConcurrentPayload`] (candidate_id, tripwire_a,
    /// tripwire_b, fired_within_ms, fired_at_ms).
    TripwireFiredConcurrent,
    /// D-19 (SCHEMA_VERSION 71): one row per
    /// [`crate::benchmarks::prmbench_runner::PRMBenchRunner::run_all`] (or
    /// [`crate::benchmarks::prmbench_runner::PRMBenchRunner::run_category`])
    /// Ok return. Carries a [`PrmBenchRunCompletedPayload`] (total,
    /// accuracy, precision, recall, gating_verdict, fixture_blake3,
    /// category_count).
    PrmBenchRunCompleted,
    /// D-5 (SCHEMA_VERSION 72): one row per `bai::emit_bai_round` posterior
    /// update. Producer is [`crate::bai::emit_bai_round`]; the value carries
    /// the BAI arm state after the Beta(alpha, beta) update and a
    /// `was_promoted` flag mirroring whether the Thompson sample fired the
    /// promotion rule on this round. See workflow/reasoning/D-5-decisions.md.
    /// Carries a [`BaiArmUpdatedPayload`] (template_id, alpha, beta, n_pulls,
    /// mean_estimate, was_promoted).
    BaiArmUpdated,
    /// D-5 closure (SCHEMA_VERSION 208): one unified summary row per
    /// [`crate::bai::emit_bai_round_with_canary`] call, emitted after the
    /// per-decision `TemplatePromoted` / `TemplateCandidateRejected` row.
    /// Carries a [`BaiPromotionDecisionPayload`] (template_id, decision,
    /// stage, evidence_pass, evidence_total, audit_id). The `decision` field
    /// is one of `"promoted"`, `"rejected"`, or `"held"` so the dashboard
    /// `BaiDrawer` can render the verdict without joining across the three-row
    /// sequence.
    BaiPromotionDecision,
    /// D-6 (SCHEMA_VERSION 73): one row per `held_out::insert_and_emit` Ok
    /// return — i.e. a fresh held-out task that passed the 9-gram
    /// contamination filter and was inserted into the store. Producer is
    /// [`crate::held_out::insert_and_emit`]. Carries a
    /// [`HeldOutTaskAddedPayload`] (task_id, source, content_blake3,
    /// ngram_count, ingested_ts_ms, eligibility_window_ms).
    HeldOutTaskAdded,
    /// D-6 (SCHEMA_VERSION 73): one row per `held_out::mark_used_and_emit`
    /// Ok return — i.e. a held-out task consumed by a BAI eval round.
    /// Producer is [`crate::held_out::mark_used_and_emit`] (called from
    /// `agent_pool::sample_held_out_task`). Carries a
    /// [`HeldOutTaskUsedPayload`] (task_id, consumer_template_id,
    /// used_ts_ms).
    HeldOutTaskUsed,
    /// D-7 (SCHEMA_VERSION 74): one row per `canary::emit_canary_run` Ok
    /// return — i.e. a completed cross-domain transfer canary report.
    /// Producer is [`crate::canary::emit_canary_run`], invoked by the BAI
    /// promotion gate. Carries a [`CanaryRunCompletedPayload`] (report_id,
    /// ts_ms, overall_transfer_score, decision_kind,
    /// dominant_fail_family).
    CanaryRunCompleted,
    /// D-7 (SCHEMA_VERSION 74): one row per per-family evaluator pass
    /// inside a canary run — emitted alongside [`Self::CanaryRunCompleted`]
    /// so the dashboard can render per-family deltas without parsing the
    /// run's `per_family` array. Carries a
    /// [`CanaryFamilyEvaluatedPayload`] (report_id, family, n_evaluated,
    /// pass_count, fail_count, transfer_score, baseline_score).
    CanaryFamilyEvaluated,
    /// D-9 — one row per `LockedKeysAuditor::audit_diff_ranged`
    /// invocation. Carries a [`LockedKeysAuditRunPayload`] (audit_id,
    /// ts_ms, n_audited, n_violations, registry_blake3,
    /// registry_path). The dashboard `LockedKeysPanel` joins these
    /// run rows to per-violation [`LockedKeysViolationDetectedPayload`]
    /// rows on `audit_id`. Producer:
    /// `crates/orchestrator/src/locked_keys_auditor.rs`. Grounding:
    /// arxiv:1908.04734 (Everitt current-RF principle).
    LockedKeysAuditRun,
    /// D-9 — one row per overlap between a staged edit and a
    /// LockedKeysRegistry entry. Carries a
    /// [`LockedKeysViolationDetectedPayload`] (audit_id, ts_ms,
    /// attempted_edit, attempted_lines, locked_key_path,
    /// locked_key_signature_blake3, reason). Producer:
    /// `crates/orchestrator/src/locked_keys_auditor.rs`.
    LockedKeysViolationDetected,
    /// E-46 (SCHEMA_VERSION 76) — dual-ledger atomic advancement. One row
    /// per `DualLedger::advance_task` call: TaskLedger.status +
    /// ProgressLedger.completion_pct/blockers updated together. Carries a
    /// [`TaskAdvancedPayload`]. Producer:
    /// `crates/orchestrator/src/dual_ledger.rs`. Grounding:
    /// arxiv:2412.21154 (Devin/SWE-bench task-ledger pattern).
    TaskAdvanced,
    /// E-46 (SCHEMA_VERSION 76) — AGENTLESS leaf-shortcut elected for a
    /// task (single-file change, ≤N LOC, no cross-crate, no async
    /// network). Carries an [`AgentlessShortcutTakenPayload`]. Producer:
    /// `crates/orchestrator/src/agentless_shortcut.rs`. Grounding:
    /// arxiv:2407.01489 (AGENTLESS, 32.00% SWE-bench Lite at $0.70/issue).
    AgentlessShortcutTaken,
    /// E-46 (SCHEMA_VERSION 76) — `GoalDriftCanary::fired()` evaluated
    /// true (drift > threshold). Carries a [`GoalDriftFiredPayload`].
    /// Producer: `crates/orchestrator/src/goal_drift_canary.rs`.
    /// Grounding: arxiv:2505.02709 (goal-drift PRM supervision).
    GoalDriftFired,
    /// D-10 (SCHEMA_VERSION 77) — one row per `agent_pool::route_claim`
    /// call. Carries a [`ClaimRoutedPayload`] (claim_id, claim_type,
    /// mode, ts_ms, ctx_summary_blake3). Producer:
    /// `crates/orchestrator/src/agent_pool.rs::route_claim`, which
    /// wraps the pure predicate
    /// `crates/orchestrator/src/grounding_mode.rs::select_grounding_mode`.
    /// Grounding: arxiv:1908.04734 (Everitt current-RF principle).
    ClaimRouted,
    /// D-21 (SCHEMA_VERSION 78) — one row per
    /// `crates/orchestrator/src/benchmarks/mode_routing_runner.rs::ModeRoutingBenchmark::run_all`
    /// Ok return. Carries a [`ModeRoutingBenchRunCompletedPayload`]
    /// (total, accuracy, f1_macro, gating_verdict, fixture_blake3,
    /// fixture_row_count). The runner gates pass iff macro F1 >= 0.85
    /// AND every per-mode F1 >= 0.70 (no class collapse). CI consumes
    /// the row's gating_verdict via the `elai bench mode-routing`
    /// subcommand exit code.
    ModeRoutingBenchRunCompleted,
    /// D-11 (SCHEMA_VERSION 79) — one row per
    /// `CheckerRegistry::run_all_applicable` result triple emitted by
    /// `crates/orchestrator/src/agent_pool.rs::dispatch_constructive_checkers`.
    /// Carries a [`CheckerResultEmittedPayload`] (checker_id,
    /// checker_kind, result_kind, claim_id, ts_ms, reason). Producer is
    /// the orchestrator (NOT the policy model). Grounding:
    /// arxiv:2310.04406 (constructive proof checking), arxiv:2211.12588
    /// (PoT, T1), arxiv:2402.01030 (CodeAct, T1).
    CheckerResultEmitted,
    /// D-13 (SCHEMA_VERSION 80) — one row per
    /// `agent_pool::run_adversarial_pass` adversary-emit step. Carries
    /// an [`AdversaryGeneratedPayload`] (counter_id, claim_id,
    /// severity, model_family). Producer:
    /// `crates/orchestrator/src/agent_pool.rs::run_adversarial_pass`.
    /// Grounding: arxiv:2403.07974 (T1) adversarial verification.
    AdversaryGenerated,
    /// D-13 (SCHEMA_VERSION 80) — one row per re-run verifier
    /// consensus that includes adversary counter-evidence. Carries a
    /// [`ConsensusReevaluatedWithCounterPayload`] (claim_id,
    /// original_verdict, post_counter_verdict). Producer:
    /// `crates/orchestrator/src/agent_pool.rs::run_adversarial_pass`.
    /// Grounding: arxiv:2504.16760 (T1) lightweight latent verifiers
    /// (heterogeneous-family verification).
    ConsensusReevaluatedWithCounter,
    /// D-12 (SCHEMA_VERSION 81) — one row per
    /// `agent_pool::run_operational_check` invocation. Carries a
    /// [`BabyStepDispatchedPayload`] (step_id, parent_task_id,
    /// checker_kinds_used, outcome, ts_ms). Producer:
    /// `crates/orchestrator/src/agent_pool.rs::run_operational_check`.
    /// Grounding: arxiv:2305.16291 Voyager (T1)
    /// skill-with-measurable-effect; arxiv:2407.01489 AGENTLESS (T1)
    /// single-step verification.
    BabyStepDispatched,
    /// D-16 (SCHEMA_VERSION 82) — one row per
    /// `conformal_calibration::compute_tau_and_emit` Ok return. Carries
    /// a [`ConformalTauComputedPayload`] (tau, epsilon, corpus_size,
    /// source, ts_ms). Producer:
    /// `crates/orchestrator/src/conformal_calibration.rs`. Grounding:
    /// arXiv:2503.01345 (UniCR) + Vovk et al. 2005 §2.2 quantile-based
    /// conformal predictor.
    ConformalTauComputed,
    /// D-16 (SCHEMA_VERSION 82) — one row per
    /// `conformal_calibration::decide_and_emit` call (and by extension
    /// every `agent_pool::dispatch_conformal_refusal` invocation).
    /// Carries a [`ConformalRefusalDecidedPayload`] (claim_id,
    /// claim_score, tau, decision, confidence, ts_ms). Producer:
    /// `crates/orchestrator/src/conformal_calibration.rs`. Grounding:
    /// arXiv:2503.01345 (UniCR) — formal Mode E refusal rule.
    ConformalRefusalDecided,
    /// D-15 (SCHEMA_VERSION 83) — one row per
    /// `novelty::emit_novelty_classified` call (and by extension every
    /// `agent_pool::dispatch_novelty` invocation). Carries a
    /// [`NoveltyClassifiedPayload`] (template_id, class,
    /// dkg_match_count, near_duplicate_threshold, ts_ms). Producer:
    /// `crates/orchestrator/src/novelty.rs`. Grounding:
    /// arXiv:2406.05761 (LLM novelty detection three-bucket taxonomy);
    /// docs/specs/guide/real-problem-solving-grounded-2026-05-19.md
    /// §1 Mode C + §4.8.
    NoveltyClassified,
    /// D-18 (SCHEMA_VERSION 84) — one row per
    /// `observable_effect::verify_observable_effect` call. Carries a
    /// [`ObservableEffectVerifiedPayload`] (effect_id, trigger_kind,
    /// observation_kind, sandbox_tier, outcome, confound_flags,
    /// elapsed_us, ts_ms). Producer:
    /// `crates/orchestrator/src/observable_effect.rs`. Plan:
    /// docs/thefinalglance/D-plan/D-18_observable_effect_typed.md
    /// (orthogonality + sandbox-vs-production-labeling + remaining
    /// 3-variant pack from the plan deferred to a follow-up session).
    ObservableEffectVerified,
    /// D-17 (SCHEMA_VERSION 85) — one row per
    /// `refusal_sla::RefusalRateMonitor::compute_snapshot` call (and by
    /// extension every `agent_pool::dispatch_refusal_sla_check`
    /// invocation). Carries a [`RefusalRateSnapshotPayload`]
    /// (refusal_rate, window_total, abstain_count, health_band, ts_ms).
    /// Producer: `crates/orchestrator/src/refusal_sla.rs`. Grounding:
    /// arXiv:2503.01345 (UniCR honest refusal); docs/specs/guide/
    /// real-problem-solving-grounded-2026-05-19.md §4.6.
    RefusalRateSnapshot,
    /// D-17 (SCHEMA_VERSION 85) — one row per snapshot whose
    /// `HealthBand != Healthy`. Carries a [`RefusalSlaBreachPayload`]
    /// (rate, threshold, breach_kind, ts_ms). `breach_kind` is the
    /// snake-case `above` | `below` label. Producer:
    /// `crates/orchestrator/src/refusal_sla.rs::sla_breach` emission
    /// path.
    RefusalSlaBreach,
    /// D-22 (SCHEMA_VERSION 86) — one row per
    /// [`crate::benchmarks::conformal_validation_runner::ConformalValidationRunner::run_all`]
    /// Ok return. Carries a [`ConformalValidationCompletedPayload`]
    /// (tau, epsilon, empirical_refusal_rate, gating_verdict,
    /// fixture_blake3, total, ts_ms). Gates the UniCR distribution-free
    /// guarantee (arXiv:2503.01345, T1) on a held-out validation fixture
    /// split into in_domain / near_ood / far_ood buckets. Producer:
    /// `crates/orchestrator/src/benchmarks/conformal_validation_runner.rs`.
    /// Plan: docs/thefinalglance/D-plan/D-22_conformal_calibration_validation.md.
    ConformalValidationCompleted,
    /// D-20 (SCHEMA_VERSION 87) — one row per
    /// [`crate::benchmarks::arc_agi2_runner::ARCAGI2Runner::run_all`]
    /// Ok return. Carries an [`ARCBenchmarkCompletedPayload`] (total,
    /// honestly_evaluated_count, gating_verdict, fixture_blake3,
    /// dominant_mode, dominant_novelty_class, ts_ms). Gates ELAI's
    /// mode-routing + novelty-classifier honesty stack against
    /// ARC-AGI-2-style out-of-distribution claims. Producer:
    /// `crates/orchestrator/src/benchmarks/arc_agi2_runner.rs`. Plan:
    /// docs/thefinalglance/D-plan/D-20_arc_agi2_ood_probe.md.
    ArcBenchmarkCompleted,
    ToolGated,
    ToolEager,
    ToolLazyResolved,
    ToolDenied,
    // E-27 (SCHEMA_VERSION 89): quantization-backend beacons.
    QuantBackendSelected,
    KvCacheDtypeSoftwareSim,
    AwqCheckpointBuilt,
    CliMcpDispatch,
    CliMcpServerStarted,
    CliMcpServerStopped,
    // E-15: sandbox hardening beacons
    FirecrackerRestoreSlow,
    SnapshotUniquenessRegenerated,
    SnapshotEvicted,
    SandboxBackendSelected,
    AgentForkedFromSnapshot,
    ResearcherNetEgressBlocked,
    // E-24: MCP hardening beacons
    McpSchemaPinViolation,
    McpSanitizationRuleApplied,
    McpBudgetEnforced,
    McpLegacySseTransport,
    McpLlmGuardBlocked,
    AnthropicGatewayDetected,
    // C-48: SafeHarbor memory-trust beacons
    MemoryTierAssigned,
    MemoryQueryFiltered,
    SafetyCriticalBlocked,
    // E-30: public-incident hardening beacons
    ArgvDestructiveDenied,
    ArgvApprovalRequired,
    TrustDialogHookDeferred,
    TrustDialogEnvDeferred,
    McpPinVersionRefused,
    AuditorLineageCollision,
    UntrustedMemoryWriteBlocked,
    IncidentRecordCreated,
    // E-32: context-window extension beacons
    /// FITM positional-prior subtraction applied on a long-context request
    /// (prompt_len > 8K, ELAI_FITM_ENABLED=1). Carries prompt_len, dummy_len,
    /// applied_ms. Producer: python/elai_inference/server.py::FoundInTheMiddleCalibrator.
    FitmCalibrationApplied,
    /// Sidecar booted with Qwen2.5-7B-Instruct-1M at 64K context cap
    /// (ELAI_DEEP_MODE=1, VRAM >= 24 GB). Carries model_id, ctx_cap, vram_gb.
    /// Producer: python/elai_inference/server.py::activate_deep_mode.
    DeepModeActivated,
    /// SessionDag handoff at 85% of effective_max_context chose local deep mode
    /// over cloud escalation for cross-file synthesis task.
    /// Carries reason, files_referenced.
    /// Producer: crates/providers/src/runtime/escalation_policy.rs::deep_mode_escalation_hook.
    DeepModeEscalationChosen,
    /// ELAI_USE_YARN=1 refused at sidecar boot for Qwen2.5-Coder model.
    /// Loud refusal per E-32 (15-20% quality drop on short prompts per vllm issue 18728).
    /// Producer: python/elai_inference/server.py::refuse_yarn_at_boot.
    YarnRefused,
    // B-3 (SCHEMA_VERSION 98): AntiStalenessDrawer + RulesDrawer beacons.
    /// One row per dep-audit run completion. Producer: B-23 dep-audit
    /// orchestrator (currently pending — variant ships so the dashboard
    /// renderer can subscribe without producing zero rows).
    DepAuditCompleted,
    /// One row per baseline-harness run completion. Producer: B-4 baseline
    /// harness (currently pending — variant ships so the dashboard
    /// renderer can subscribe without producing zero rows).
    BaselineRunCompleted,
    /// One row per deterministic-audit gate run. Producer: B-15
    /// deterministic-audit gate (currently pending — variant ships so the
    /// dashboard renderer can subscribe without producing zero rows).
    DeterministicAuditRun,
    /// One row per rule lifecycle status change. Mirrors the persistent
    /// `rule_transitions` SQLite table at rules/src/schema.rs:101 onto
    /// the SSB stream so the dashboard's RulesDrawer can render the
    /// last-5 transitions inline without polling the rules DB directly.
    /// Producer: rules::lifecycle::write_transition_log (will emit when
    /// invoked from a context with an Ssb handle).
    RuleTransition,
    /// B-4: canary drift snapshot. One row per periodic cascade evaluation
    /// that observed a fresh canary similarity vs the persisted reference
    /// vector (see knowledge::startup::check_canary_drift). Dashboard
    /// `get_canary_drift` Tauri command reads the latest row so
    /// SettingsWindow can render `cosine vs threshold` with a "Recompute"
    /// affordance when the cosine drops below `drift_threshold`.
    CanaryDrift,
    /// B-19 (SCHEMA_VERSION 101): per-agent-execution DAIRA marker aggregation.
    /// Carries a [`MarkerReportPayload`] with a 15-entry phase-count map keyed
    /// on [`DAIRA_MARKERS`] phase strings. Emitted ONCE per agent-execution
    /// boundary (task_runner Stop-hook site) so the dashboard can render an
    /// aggregated activity-feed row instead of replaying the raw per-event
    /// `DairaTrace` stream. Producer:
    /// `crate::hook_dispatcher::emit_marker_report_to_ssb`. Consumer:
    /// dashboard ActivityFeed aggregated DAIRA panel.
    MarkerReport,
    /// B-25 (SCHEMA_VERSION 102): Phase 4J Empire integration — factory-brain
    /// handshake complete row. Carries [`EmpireConnectedPayload`] with
    /// `brain_id` + `protocol_version`. Producer:
    /// `empire::bridge::EmpireBridge::emit_action` (EMPIRE-PENDING activation).
    /// Consumer: dashboard `EmpireBridgePanel` connection-state pane.
    EmpireConnected,
    /// B-25 (SCHEMA_VERSION 102): Phase 4J — bridge transitions to
    /// Disconnected. Carries [`EmpireDisconnectedPayload`] with `brain_id`
    /// + free-form `reason` (timeout / protocol_error / explicit_shutdown).
    EmpireDisconnected,
    /// B-25 (SCHEMA_VERSION 102): Phase 4J — Empire-side event surfaces on
    /// the bus. Carries [`EmpireEventReceivedPayload`] with `source` +
    /// `event_type` + opaque JSON `payload`. Rate-limited (default
    /// 100/sec) at the bridge layer; this SSB row only fires for
    /// admitted events.
    EmpireEventReceived,
    /// B-25 (SCHEMA_VERSION 102): Phase 4J — ElaiNeuron capability dispatch
    /// attempt. Carries [`EmpireCapabilityInvokedPayload`] with the
    /// `capability` name, the `caller` identifier, and `allowed` (true
    /// iff ToolFilter permitted the call). Denied attempts emit one row
    /// with `allowed=false` so the dashboard can render a security audit
    /// trail.
    EmpireCapabilityInvoked,
    /// B-25 (SCHEMA_VERSION 102): Phase 4J — repair pipeline surfaces a
    /// proposal. Carries [`EmpireRepairProposedPayload`] with the
    /// detected `issue` class, `target_path`, and `fix_kind` summary.
    EmpireRepairProposed,
    // ---- C-5 follow-on (SCHEMA_VERSION 103): cloud-mode hard ceiling +
    // warn-at-percent + pre-task cost estimate beacons. The 3 variants
    // land in one atomic SCHEMA bump per the C-5 grounding sweep S4
    // resolution (single bump cleaner than three sequential bumps).
    /// C-5 (SCHEMA_VERSION 103): cloud routing rejected at the hard
    /// ceiling. Emitted by the cloud cap-hook in
    /// `crate::task_runner::drive_task` after `CloudModeRouter::route_with_spend`
    /// returns `CloudModeError::DailySpendCapExceeded` or
    /// `MonthlySpendCapExceeded`. Carries [`RouteBlockedPayload`]
    /// (reason, spent_usd, cap_usd, cap_period). Dedup'd via the same
    /// `cloud_spend_blocked_emitted` Mutex that gates the cap_reached
    /// EscalateToCloud row, so each UTC-day boundary produces at most
    /// one BLOCKED event regardless of how many baby steps attempt to
    /// cross the cap.
    RouteBlocked,
    /// C-5 (SCHEMA_VERSION 103): warn-at-percent threshold crossed.
    /// Emitted when `day_usd >= cap * warn_at_percent / 100.0` for the
    /// first time in a UTC day. Carries [`WarnLimitReachedPayload`]
    /// (spent_usd, cap_usd, warn_at_percent, pct_of_cap). Fires before
    /// the hard cap so the dashboard CloudPanel can show an amber gauge.
    WarnLimitReached,
    /// C-5 (SCHEMA_VERSION 103): pre-task USD cost projection. Emitted
    /// once per task by the cloud cap-hook the first time it sees a
    /// Cloud / Auto baby step. Carries [`CostEstimatePayload`]
    /// (low_usd, high_usd, model, step_count, heuristic_source). The
    /// heuristic_source field flags whether the underlying token/LOC
    /// ratio is grounded ("measured") or the
    /// `TOKENS_PER_LOC_UNGROUNDED_HEURISTIC` constant
    /// ("ungrounded_heuristic"). Dashboard CloudPanel shows the
    /// projection alongside the running task spend so operators can
    /// abort before crossing the ceiling.
    CostEstimate,
    /// C-36 (SCHEMA_VERSION 104): MuCoCo mutation operator dispatch.
    /// Emitted by orchestrator::metamorphic_testing::dispatch_mucoco_mutation
    /// (and the metamorphic_runner auditor wiring that consumes it) on
    /// every operator invocation — applied or skipped. Carries
    /// [`MucocoMutationAppliedPayload`] (operator, language,
    /// source_hash, applied, skip_reason, timestamp_ms). One row per
    /// dispatch so the testing-trust dashboard can render per-operator
    /// + per-language fire counts and skip-reason distributions. Paper:
    /// arXiv:2604.19086 (interface map at
    /// workflow/papers/mucoco-2604.19086-interface.md).
    MucocoMutationApplied,
    /// D-3 closure (SCHEMA_VERSION 105): PPM verifier sidecar unavailable.
    /// Distinct semantic from [`Self::PpmGatingFailed`] (which means "scored
    /// but below threshold"): this variant fires when the PPM sidecar's
    /// health check fails OR PpmClient::score returns an error before
    /// producing a numeric score. Producer: D-2 PpmClient health-path +
    /// agent_pool::run_consensus_pass adapter-error route. Carried
    /// atomically from D-2 (DECISION-1 — deferred to D-3 for the atomic
    /// 9-tuple bump). Carries a [`PpmUnavailablePayload`] (candidate_id,
    /// verifier_id, reason).
    PpmUnavailable,
    /// D-3 closure (SCHEMA_VERSION 105): operational telemetry row emitted
    /// when one consensus run produced >= 1 Pass AND >= 1 Reject signals.
    /// Operationally distinct from VerifierConsensusFail (which means "the
    /// rule was violated by the aggregate"); a disagreement is the input
    /// the D-4 tripwire stack monitors as an early signal of verifier
    /// drift. Producer: agent_pool::run_consensus_pass observability hook
    /// post-aggregate. Carries a [`VerifierDisagreementPayload`]
    /// (candidate_id, template_id, verifier_a_kind, verifier_b_kind,
    /// score_delta).
    VerifierDisagreement,
    /// D-3 closure (SCHEMA_VERSION 105): cross-context verifier dispatch.
    /// Emitted by `crate::cross_context::run_cross_context` /
    /// `CrossContextVerifier::run` on every invocation regardless of
    /// outcome (Agree / Disagree / Insufficient). Producer: the
    /// cross-context call site inside agent_pool / task_runner's CLAIM
    /// hook. Carries a [`CrossContextDispatchedPayload`] (claim_id,
    /// context_a_hash, context_b_hash, jaccard, agreement). See
    /// `crate::cross_context` for the shingle-jaccard construction.
    CrossContextDispatched,
    /// C-38 (SCHEMA_VERSION 107): per-invocation PageRank Stage 2 beacon.
    /// Emitted by `elai_knowledge::pipeline::QueryPipeline::query_with_tier`
    /// after every `compute_repo_map` invocation (including the empty
    /// graph case). Carries a [`PagerankRunPayload`] (workspace_root_hash,
    /// node_count, top_node_hash, elapsed_ms). Producer wiring:
    /// knowledge::pipeline.rs Stage 2 block. Locked by the C-38 contract
    /// test at `crates/contracts/tests/c38_pagerank_production_contract.rs`.
    PagerankRun,
    /// C-38 (SCHEMA_VERSION 107): DKG rebuild start beacon. Emitted by
    /// the CLI helper `dkg_lifecycle::ensure_dkg_or_rebuild` when the
    /// canonical DKG database (`.elai_cc/indexes/dkg.db`) is missing or
    /// stale and a rebuild is about to run. Bracketed with
    /// [`Self::DkgRebuildComplete`]. Carries a
    /// [`DkgRebuildStartedPayload`] (workspace_root_hash, trigger_reason).
    DkgRebuildStarted,
    /// C-38 (SCHEMA_VERSION 107): DKG rebuild end beacon. Emitted at the
    /// end of every `dkg_lifecycle::ensure_dkg_or_rebuild` rebuild
    /// attempt (success=true and success=false). Carries a
    /// [`DkgRebuildCompletePayload`] (workspace_root_hash, success,
    /// duration_ms, node_count).
    DkgRebuildComplete,
    /// C-40 (SCHEMA_VERSION 109): GitHub PR opened via the
    /// `crate::github_pr::GhCliAdapter::open` method. Emitted on every
    /// successful `gh pr create` invocation. The PR URL is carried as a
    /// blake3-prefix hash (`pr_url_hash`) so dashboards can group by PR
    /// identity without exposing the raw URL (which leaks repo +
    /// PR-number to anyone who can subscribe to the SSB stream). Carries
    /// a [`GithubPrOpenedPayload`].
    GithubPrOpened,
    /// C-40 (SCHEMA_VERSION 109): per-status-fetch beacon. Emitted on
    /// every successful `gh pr view --json` invocation by
    /// `GhCliAdapter::status`. Carries hashed CI + review-state digests
    /// so the dashboard can render status-transition rows without
    /// pinning the raw GH JSON to SSB. Carries a
    /// [`GithubPrStatusFetchedPayload`].
    GithubPrStatusFetched,
    /// C-40 (SCHEMA_VERSION 109): PR closed (not merged) via
    /// `GhCliAdapter::close`. Distinct wire variant from `GithubPrMerged`
    /// so the dashboard can branch on the tag rather than join raw rows.
    /// Carries a [`GithubPrClosedPayload`].
    GithubPrClosed,
    /// C-40 (SCHEMA_VERSION 109): PR merged via `GhCliAdapter::merge`.
    /// `merge_sha_hash` is a blake3-prefix hash of the merge commit SHA
    /// so the dashboard can correlate the merged PR with the local
    /// branch_merged row without leaking the upstream SHA. Carries a
    /// [`GithubPrMergedPayload`].
    GithubPrMerged,
    /// E-41 (SCHEMA_VERSION 135): review comment composed and posted via
    /// `crate::github_pr::review::post_review_comment`. `pr_number_hash`
    /// is blake3-prefix of the PR number string; `comment_kind_hash`
    /// distinguishes PR-thread vs file-anchored vs force-push-check.
    /// Carries a [`PrReviewComposedPayload`].
    PrReviewComposed,
    /// E-41 (SCHEMA_VERSION 135): poll-result summary for new PR comments
    /// since a caller-supplied watermark. Emitted once per
    /// `crate::github_pr::comment_watcher::watch_comments` call.
    /// `new_comments_count` is the count of comments strictly after the
    /// watermark. Carries a [`PrCommentWatchedPayload`].
    PrCommentWatched,
    /// E-41 (SCHEMA_VERSION 135): slopsquat scan over a PR diff via
    /// `crate::github_pr::package_verify::verify_pr_deps_no_slopsquat`.
    /// `slopsquat_count` is the number of dep candidates flagged by the
    /// deterministic regex + known-squat list. Carries a
    /// [`PrPackageVerifyResultPayload`].
    PrPackageVerifyResult,
    /// E-41 (SCHEMA_VERSION 135): Semgrep gate result for a PR working
    /// copy via `crate::github_pr::semgrep_gate::run_semgrep_gate`.
    /// `findings_count` aggregates across all severities;
    /// `severity_max_hash` is blake3-prefix of the canonical
    /// snake_case severity label (`info` / `warning` / `error` /
    /// `owasp_llm` / `other` / `toolchain_missing`). Carries a
    /// [`PrSemgrepGateResultPayload`].
    PrSemgrepGateResult,
    /// E-41 (SCHEMA_VERSION 135): auto-merge guard verdict =
    /// [`crate::github_pr::auto_merge_guard::MergeVerdict::Block`].
    /// Emitted once per call to `assess_merge_safety` that returns
    /// Block. `block_reason_hash` is blake3-prefix of the canonical
    /// reason label (`loc_over_cap` / `multi_file` / `owned_paths`).
    /// Carries a [`PrAutoMergeBlockedPayload`].
    PrAutoMergeBlocked,
    /// D-5 closure (SCHEMA_VERSION 110): per-pull Thompson-sample beacon.
    /// Producer is [`crate::bai::emit_bai_round`]; the value carries the
    /// Beta(alpha, beta) draw the Thompson policy used to choose the next
    /// arm — emitted immediately BEFORE the matching `BaiArmUpdated` row
    /// so consumers can join the sample, the posterior update, and any
    /// follow-on promote/reject decision in one ordered triple per round.
    /// Carries a [`ThompsonSampledPayload`] (arm_id, sampled_value,
    /// alpha, beta).
    ThompsonSampled,
    /// C-41 (SCHEMA_VERSION 111): skills-driven workflow registry — top-1
    /// candidate emitted on every `crate::skill_registry::top_k_skills`
    /// invocation that returns a non-empty set. `skill_name_hash` is the
    /// blake3-prefix u64 of the resolved skill name; `score` is the
    /// deterministic score in [0.0, 1.0]; `top_k_rank` is the 1-based
    /// rank within the candidate window the registry returned. Carries a
    /// [`SkillTriggeredPayload`].
    SkillTriggered,
    /// C-41 (SCHEMA_VERSION 111): skills-driven workflow registry — paired
    /// with `SkillTriggered`. Emitted when the orchestrator actually
    /// dispatches the resolved skill (the harness Skill tool's
    /// `crate::tools::execute_skill` returns). `exit_ok` records whether
    /// the dispatch returned Ok; `duration_ms` is wall-clock time from
    /// dispatch start to skill return. Carries a [`SkillExecutedPayload`].
    SkillExecuted,
    /// D-6 closure (SCHEMA_VERSION 112): held-out promotion-gate decision
    /// row. Producer is the BAI promotion path that calls
    /// `held_out::gate_promotion_via_held_out` after the regression delta
    /// is computed against the held-out subset. Carries a
    /// [`HeldOutPromotionGatePayload`] (task_id, allow, delta,
    /// regression_threshold, passes, fails, reason).
    HeldOutPromotionGate,
    /// D-6 closure (SCHEMA_VERSION 112): held-out refresh-event row.
    /// Emitted by the auto-refresh tick when the policy retires aged
    /// tasks and/or adds new ingest. Carries a [`HeldOutRefreshPayload`]
    /// (source_label, added_count, retired_count, ts_ms).
    HeldOutRefresh,
    /// C-42 (SCHEMA_VERSION 113): Semantic Gateway L2 composed-Allow
    /// beacon. Emitted by
    /// `plugins::mcp_bridge::DefaultMcpBridge::dispatch_tool_call_with_authz`
    /// when both the L2 AuthzPolicy and the C-35 ToolGate permit the
    /// call. Carries a [`GatewayPolicyAllowedPayload`] (role_hash,
    /// tool_hash, deny_reason_label, ts_ms).
    GatewayPolicyAllowed,
    /// C-42 (SCHEMA_VERSION 113): Semantic Gateway composed-Deny beacon.
    /// `deny_reason_label` is `"authz"` when the L2 policy denied or
    /// `"tool_gate"` when the C-35 gate denied; `deny_reason_hash` is a
    /// blake3-prefix u64 of the structured reason so dashboard
    /// subscribers can group by failure mode without surfacing the raw
    /// reason text. Carries a [`GatewayPolicyDeniedPayload`].
    GatewayPolicyDenied,
    /// C-42 (SCHEMA_VERSION 113): Semantic Gateway L3 Crypto-HITL beacon.
    /// L3 is DEFERRED-TO-FOLLOWUP (out of session 2 scope); the variant
    /// is declared so the downstream plan can land emitters without a
    /// second SCHEMA bump. `challenge_hash` is the SHA-256-of-(action,
    /// params, CoT) evidence package per paper Section 5. Carries a
    /// [`GatewayHitlPendingPayload`].
    GatewayHitlPending,
    /// D-7 closure (SCHEMA_VERSION 114): cross-domain regression
    /// observation row. Emitted by `canary::emit_canary_run` when at least
    /// one non-target family regressed beyond the fail-band relative to
    /// the candidate's target family. Pairs with the existing
    /// [`Self::CanaryRunCompleted`] row (the latter carries the
    /// dominant-family attribution; this row carries the source/target
    /// pair + the absolute regression magnitude in percent). ELAI design
    /// hypothesis — NanoFlux (arXiv:2509.23252) provides empirical
    /// motivation only; no canary mechanism in the paper itself. Carries
    /// a [`CrossDomainRegressionPayload`].
    CrossDomainRegression,
    /// D-7 closure (SCHEMA_VERSION 114): per-family clean-pass row.
    /// Emitted once per family that cleared the marginal band on a
    /// canary run. Pairs with the existing [`Self::CanaryFamilyEvaluated`]
    /// row (which captures every evaluated family, pass-or-fail); this
    /// row is the strict subset where transfer_score >= 0 AND
    /// sample_count >= `min_per_family`. ELAI design hypothesis — see
    /// the [`Self::CrossDomainRegression`] grounding note. Carries a
    /// [`CanaryFamilyPassPayload`].
    CanaryFamilyPass,
    /// D-7 closure (SCHEMA_VERSION 114): per-family insufficient-data
    /// row. Emitted when a family's sample count fell below the
    /// `min_per_family` floor so the family was reported but not
    /// counted toward the overall decision. ELAI design hypothesis —
    /// see the [`Self::CrossDomainRegression`] grounding note. Carries
    /// a [`CanaryInsufficientDataPayload`].
    CanaryInsufficientData,
    /// D-8 closure (SCHEMA_VERSION 115): cryptographic archive verify
    /// result. Emitted once per `elai archive verify` invocation by the
    /// CLI driver (crates/cli/src/archive_verify.rs::run), regardless of
    /// pass / fail. The payload carries the typed per-entry error list
    /// ([`Vec<crate::archive_chain::ArchiveVerifyError>`]) so dashboards
    /// can branch on the failure mode (HashMismatch / BrokenParent /
    /// DuplicateHash / MalformedJson) without parsing freeform strings.
    /// arxiv:2507.05619 (T1) + DGM §5 are supporting documentation for
    /// the lobotomy-attack rationale; neither paper specifies the wire
    /// layout. Carries an [`ArchiveVerifyResultPayload`].
    ArchiveVerifyResult,
    /// C-43 closure (SCHEMA_VERSION 116): Phase 4H scope-freeze
    /// ratification beacon. Emitted once per `elai freeze ratify`
    /// invocation by the CLI driver
    /// (crates/cli/src/freeze_cmd.rs::run_ratify). Carries the
    /// blake3-derived 64-bit manifest hash so the dashboard
    /// FreezeStatusBar can detect drift between ratifications without
    /// re-reading scope-freeze.toml. ELAI design hypothesis — no
    /// paper grounding. See workflow/sessions/s-25120/mistakes.md for
    /// the bad-cite analysis of arXiv:2604.21725. Carries a
    /// [`ScopeFreezeRatifiedPayload`].
    ScopeFreezeRatified,
    /// C-43 closure (SCHEMA_VERSION 116): Phase 4H scope-freeze
    /// violation beacon. Emitted once per offending item by the
    /// `elai freeze verify` CLI driver
    /// (crates/cli/src/freeze_cmd.rs::run_verify) or the dashboard's
    /// `scope_freeze_status` Tauri command. ELAI design hypothesis —
    /// no paper grounding. Carries a [`ScopeFreezeViolatedPayload`].
    ScopeFreezeViolated,
    /// C-44 (SCHEMA_VERSION 117): Cursor Composer 2 cost-accrual beacon.
    /// Emitted by `RuntimeOrchestrator::try_complete` after a successful
    /// Composer 2 (or any future paid-backend) invocation. Carries
    /// blake3/Fx-hashed backend + role identifiers (wire-stable; no
    /// plaintext leak), token counts, est cost in micros (1e-6 USD), and
    /// a producer-side ts_ms. Generalised across paid backends so future
    /// paid-cloud adapters reuse the same wire surface without a fresh
    /// SCHEMA bump. ELAI design hypothesis — Composer 2 paper
    /// (arXiv:2603.24477) grounds model identity only, NOT cost-accrual
    /// policy. See workflow/papers/composer2-interface.md.
    /// Carries a [`BackendCostAccruedPayload`].
    BackendCostAccrued,
    /// C-45 (SCHEMA_VERSION 120): CANTANTE per-rollout role-drop audit
    /// trail. Emitted by the credit-attribution harness BEFORE the
    /// production agent-spawn path reroutes around the dropped role
    /// (emit-before-reroute invariant per the C-45 plan). Carries
    /// blake3-prefix `role_name_hash` (FxHash of the snake_case
    /// `AgentRole` name) and producer-side `ts_ms`. Paper-grounded:
    /// arXiv:2605.13295 (Zehle 2026, Section 3.2). Carries an
    /// [`AgentRoleDroppedPayload`].
    AgentRoleDropped,
    /// C-45 (SCHEMA_VERSION 120): once-per-harness-invocation beacon
    /// emitted at the start of a contrastive credit-attribution pilot.
    /// Carries the caller-supplied `run_id_hash`, the
    /// `dropped_role_name_hash` for this leg of the contrast (or zero
    /// for the baseline leg), and the partition config (`k_rollouts`,
    /// `g_partition`). Paper-grounded: arXiv:2605.13295. Carries a
    /// [`CreditAttributionRunPayload`].
    CreditAttributionRun,
    /// C-45 (SCHEMA_VERSION 120): once-per-harness-invocation beacon
    /// emitted at the aggregation step of a contrastive credit-
    /// attribution pilot. Carries the matching `run_id_hash`, a
    /// `per_role_scores_hash` (blake3-prefix u64 over the sorted
    /// per-role scoreboard so collisions across runs are detectable
    /// without exposing the raw f32 values to a privacy-sensitive
    /// dashboard), `total_roles` covered, and producer-side `ts_ms`.
    /// Paper-grounded: arXiv:2605.13295. Carries a
    /// [`CreditAttributionResultPayload`].
    CreditAttributionResult,
    /// C-46 (SCHEMA_VERSION 121): once-per-dispatch beacon emitted by the
    /// `routing::SelectiveDelegator` at the spawn boundary. Carries
    /// `task_id_hash` (blake3-prefix u64 of the caller-supplied task id),
    /// `difficulty` (scalar in `[0, 1]` returned by
    /// `TaskDifficultyClassifier`), `alpha` (the cost-correctness blend
    /// in `[0, 1]`, paper default 0.1), `dispatched_count` (number of
    /// `(model, primitive)` pairs the policy chose), and producer
    /// `ts_ms`. Paper-grounded: arXiv:2605.05007 (Cui et al 2026,
    /// Uno-Orchestra Section 5). Carries a
    /// [`SelectiveDelegationDecisionPayload`].
    SelectiveDelegationDecision,
    /// C-46 (SCHEMA_VERSION 121): emitted whenever an audit-bubble or
    /// verifier failure triggers a broader-role re-spawn on top of an
    /// earlier selective-delegation dispatch. Carries `task_id_hash`,
    /// blake3-prefix `reason_hash` (FxHash of the snake_case escalation
    /// reason string), `escalated_workers` (count of workers added
    /// beyond the original dispatch), and producer `ts_ms`. Paper-
    /// grounded: arXiv:2605.05007. Carries a
    /// [`SelectiveDelegationEscalatedPayload`].
    SelectiveDelegationEscalated,
    /// C-46 (SCHEMA_VERSION 121): emitted alongside the Decision beacon
    /// whenever the policy omitted one or more workers from the
    /// baseline chain. Carries `task_id_hash`, `skipped_count` (workers
    /// omitted), `est_savings_micros` (1e-6 USD; non-negative by
    /// construction — `skipped_count` and per-worker tariff are both
    /// non-negative), and producer `ts_ms`. Paper-grounded:
    /// arXiv:2605.05007 (Cui et al 2026, Section 6.1 cost-mitigation
    /// claim ~10x). Carries a [`SelectiveDelegationCostSavedPayload`].
    SelectiveDelegationCostSaved,
    /// C-47 (SCHEMA_VERSION 122) — one row per completed tool dispatch
    /// at the shared `DefaultMcpBridge::record_call` helper. Carries
    /// [`ToolCallLoggedPayload`] with hashed tool / shape / response
    /// fields and a synchronous `post_call_audit_ok` heuristic derived
    /// from `McpCallResult.is_error`. Paper-grounded: arXiv:2605.15041
    /// (CAST). See workflow/papers/case-based-calibration.md.
    ToolCallLogged,
    /// C-47 (SCHEMA_VERSION 122) — emitted from `consult_calibration`
    /// whenever the case base crosses the
    /// `HINT_ERROR_RATE_THRESHOLD` + `HINT_MIN_TOTAL_CALLS` gating in
    /// `tool_calibration::hinter::generate_hint`. Hint text is hashed
    /// to keep raw model-facing strings off the wire. Carries a
    /// [`ToolCalibrationHintEmittedPayload`].
    ToolCalibrationHintEmitted,
    /// C-47 (SCHEMA_VERSION 122) — emitted when the per-tool aggregate
    /// error_rate crosses the drift threshold inside
    /// `ToolCalibrationRegistry::drift_detected`. Operators see this
    /// as a red flag in the `CaseBaseInspector` panel. Carries a
    /// [`ToolCalibrationDriftDetectedPayload`].
    ToolCalibrationDriftDetected,
    /// C-49 (SCHEMA_VERSION 123) — emitted once per
    /// [`crate::experience::distill_fragments`] invocation (typically
    /// from the wire-dark post-audit hook in `task_runner`). Carries a
    /// [`PromptPolicyDistillationRunPayload`] with the
    /// `skill_name_hash`, `trace_count`, `fragments_emitted`, and
    /// producer `ts_ms`. ELAI heuristic adaptation of
    /// arXiv:2605.14443 (RL prompter is RUNPOD-PENDING).
    PromptPolicyDistillationRun,
    /// C-49 (SCHEMA_VERSION 123) — emitted once per fragment included
    /// in the distilled output of a `PromptPolicyDistillationRun`.
    /// Carries a [`PromptFragmentSelectedPayload`] with the
    /// `skill_name_hash`, blake3-prefix `fragment_hash`, observed
    /// `success_rate`, observed `freq`, and producer `ts_ms`.
    PromptFragmentSelected,
    /// C-49 (SCHEMA_VERSION 123) — emitted once per fragment rejected
    /// by the distillation thresholds (`min_freq` /
    /// `min_success_rate`). Carries a
    /// [`PromptFragmentDiscardedPayload`] with the `skill_name_hash`,
    /// blake3-prefix `fragment_hash`, blake3-prefix `reason_hash`
    /// (snake_case rejection reason), and producer `ts_ms`.
    PromptFragmentDiscarded,
    /// D-14 (SCHEMA_VERSION 124) — emitted once per
    /// `dispatch_sycophancy_probe_hook` invocation. Carries a
    /// [`SycophancyProbeDispatchedPayload`] with the `claim_id`,
    /// snake-case `probe_kind` (one of `feedback` | `are_you_sure` |
    /// `answer` | `mimicry`), and the `user_message` anchor. ELAI
    /// structural-defense adaptation of arXiv:2310.13548 (Sharma et al
    /// 2024); the probe FLAGS only — preference models do NOT
    /// reliably fix sycophancy (§4).
    SycophancyProbeDispatched,
    /// D-14 (SCHEMA_VERSION 124) — emitted once per probe verdict.
    /// Carries a [`SycophancyFlagPayload`] with the `claim_id`,
    /// `flip_detected` boolean, `delta` (string-similarity distance,
    /// range `[0.0, 1.0]`), and snake-case `verdict` label (one of
    /// `convergent` | `divergent_sycophancy_flag` | `inconclusive`).
    /// Defensive flag — the auditor consumes the row, the executor
    /// does NOT auto-rewrite.
    SycophancyFlag,
    /// D-23 (SCHEMA_VERSION 142) — emitted once per
    /// `crate::benchmarks::sycophancy_runner::SycophancyBenchmark::run`
    /// Ok return. Carries a [`SycophancyBenchRunCompletedPayload`] with
    /// (tp, fp, tn, fn_, inconclusive, detection_rate, fp_rate,
    /// verdict snake-case label, total_fixtures, fixture_blake3,
    /// runner_name). Gates detection_rate >= 0.70 AND fp_rate <= 0.15
    /// per Sharma §4 single-turn reproducibility band + plan §Tasks
    /// step 3. See workflow/reasoning/D-23-decisions.md DECISION-1.
    SycophancyBenchRunCompleted,
    /// D-24 (SCHEMA_VERSION 143) — emitted once per
    /// `crate::benchmarks::canary_runner::CanaryBenchmark::run` Ok
    /// return. Carries a [`CanaryBenchRunCompletedPayload`] with
    /// (true_block, false_block, true_pass, false_pass,
    /// harmful_blocked, helpful_blocked, neutral_blocked,
    /// total_fixtures, verdict snake-case label, fixture_blake3,
    /// runner_name). Gates harmful_blocked >= 20 AND
    /// helpful_blocked <= 5 AND neutral_blocked <= 2 (ELAI calibration;
    /// NanoFlux arXiv:2509.23252 paper has no canary mechanism so the
    /// thresholds are empirical). See workflow/reasoning/
    /// D-24-decisions.md DECISION-1.
    CanaryBenchRunCompleted,
    /// D-25 (SCHEMA_VERSION 144) — emitted once per
    /// `crate::benchmarks::tamper_runner::TamperBenchmark::run` Ok
    /// return. Carries a [`DeterminismTamperSuiteCompletedPayload`]
    /// with (total_scenarios, passed, failed, max_wall_ms, verdict
    /// snake-case label, scenarios_blake3). Gates verdict == "pass"
    /// AND max_wall_ms <= 5000 (5-second per-scenario wall budget per
    /// plan objective). Drives D-8 verify_archive IN-PROCESS across a
    /// 10-scenario JSON-manifest matrix; setup.sh scripts are kept as
    /// documentation only per D-25 DECISION-5. arxiv:2507.05619 (T1)
    /// integrity-checksum recommendation cited as motivation only
    /// (no tamper-suite mechanism in the paper). See workflow/
    /// reasoning/D-25-decisions.md DECISION-1.
    DeterminismTamperSuiteCompleted,
    /// D-28 (SCHEMA_VERSION 145) — emitted once per
    /// `crate::farm::adv_rm::AdvRmFarmer::run` Ok return. Carries an
    /// [`AdversarialFarmRunCompletedPayload`] with (samples_farmed,
    /// rounds, prmbench_delta_pp, verdict snake-case label,
    /// paper_floor_pp, corpus_blake3). Host-side scaffolding for the
    /// Adv-RM farm loop (arXiv:2504.06141 §3.2); the live PPM scoring
    /// loop + Bradley-Terry retrain + dual `bench_prmbench` v1/v2 runs
    /// are RUNPOD-PENDING (see `ppm_adversarial.py` and
    /// `workflow/remote-setup/ppm_finetune.sh`). Gates
    /// `prmbench_delta_pp >= paper_floor_pp` (default 1.0pp —
    /// recalibrated from the plan's ungrounded "+5pp" claim per
    /// paper §5 reported overall +0.7pp / +2.86pp Chat-Hard).
    /// Plan-doc named the variants `AdversarialCorpusGenerated` +
    /// `PpmFinetuneCompleted`; REJECTED per D-19/D-23/D-24/D-25
    /// single-variant precedent — D-28 ships exactly one new variant.
    /// See workflow/reasoning/D-28-decisions.md DECISION-1.
    AdversarialFarmRunCompleted,
    /// D-29 (SCHEMA_VERSION 146) — emitted once per hierarchical BAI
    /// round when `ELAI_BAI_MODE=hierarchical` AND `ELAI_BAI_ENABLE=1`.
    /// Wire-dark by default. Carries a
    /// [`HierarchicalBaiRoundCompletedPayload`] with the round counter,
    /// the mu / kappa hyperposterior (mean + variance), the per-arm
    /// shrunk Beta summaries, the selected arm + selection basis,
    /// the Gibbs iteration count, and the canary gate outcome. Paper
    /// labels (per workflow/reasoning/D-29-decisions.md): arXiv:2111.06929
    /// SUPPORTING EVIDENCE (HierTS — regret minimization, NOT BAI);
    /// arXiv:2106.10417 DIRECT GROUNDING (variance-dependent BAI). The
    /// plan's "30-50% regret reduction" headline is UNGROUNDED for the
    /// BAI regime; live flat-vs-hierarchical baseline measurement is
    /// RUNPOD-PENDING. Single-variant by D-19/D-23/D-24/D-25/D-28
    /// precedent (plan suggested per-family + per-arm rows; rejected).
    HierarchicalBaiRoundCompleted,
    /// D-30 (SCHEMA_VERSION 147) — emitted once per calibration bootstrap
    /// warm-start dispatch when `ELAI_CALIBRATION_BOOTSTRAP_ENABLE` is set
    /// (wire-dark by default; see
    /// `crate::runtime_config::CalibrationBootstrapMode`). Carries a
    /// [`CalibrationBootstrapCompletedPayload`] with the corpus slug
    /// (`"truthfulqa"` by default), the imported corpus size, the
    /// `blake3` of the fixture bytes, the warm-start tau computed via
    /// D-16 `compute_tau_with_min`, the UniCR alpha (epsilon), the
    /// production-floor flag, plus four `Option` fields populated only
    /// when a SECONDARY block-bootstrap pass runs
    /// (`bootstrap_replicates`, `bootstrap_ci_low`, `bootstrap_ci_high`,
    /// `block_length_p`). Paper labels (per
    /// workflow/reasoning/D-30-decisions.md): arXiv:2509.01455 UniCR
    /// COMPOSITION (consumes compute_tau_with_min); arXiv:2109.07958
    /// TruthfulQA DATA ORIGIN (corpus fixture); Politis & Romano 1994
    /// JASA Vol 89 §3 DIRECT GROUNDING (stationary bootstrap — only when
    /// SECONDARY runs). Single-variant by D-19/D-23/D-24/D-25/D-28/D-29
    /// precedent (11th application of pattern-2); the SECONDARY block-
    /// bootstrap path extends the payload via `Option` fields rather
    /// than introducing a second variant.
    CalibrationBootstrapCompleted,
    /// C-50 (SCHEMA_VERSION 125) — emitted once per
    /// `scaffold_contract_test_from_signature` invocation. Carries
    /// a [`ContractTestScaffoldedPayload`] with the
    /// `fn_name_hash` (blake3 prefix of the canonical function
    /// name), `contract_kind_hash` (blake3 prefix of the kernel's
    /// namespace section name), and producer `ts_ms`. Paper
    /// grounding: arXiv:2604.13100 §4.4 (Contract-Coding). Full
    /// Generator -> Discriminator RL pipeline (Algorithm 1) is
    /// RUNPOD-PENDING.
    ContractTestScaffolded,
    /// C-50 (SCHEMA_VERSION 125) — emitted once per heuristic
    /// `audit_against_contract` verdict that surfaces a kernel
    /// violation. Carries a [`ContractViolationDetectedPayload`]
    /// with the `fn_name_hash`, `violation_kind_hash` (blake3 prefix
    /// of [`crate::contract_coding::ViolationKind::as_str`]), the
    /// `severity` (u8 — `0` for advisory, `1` for kernel-blocker;
    /// stable wire format for dashboard sort), and producer
    /// `ts_ms`. Defensive flag — the heuristic Discriminator is
    /// the pre-stage; full RL Discriminator Agent (paper §4.5) is
    /// RUNPOD-PENDING.
    ContractViolationDetected,
    /// C-52 (SCHEMA_VERSION 126) — emitted once per Dafny self-
    /// healing loop termination (either first-shot success OR
    /// `MAX_HEAL_ITER` exhaustion). Carries a
    /// [`DafnyVerifyCompletedPayload`] with `tier_hash` (blake3
    /// prefix of the [`crate::dafny_verify::DafnyTier`] wire tag),
    /// `dafny_pass` (verifier exit + banner check), `udebug_pass`
    /// (functional-test suite outcome — vacuous-verification guard
    /// per paper §3), `heal_iter` (0 = first-shot pass; up to 10 =
    /// `MAX_HEAL_ITER` per paper §3 cap), and producer `ts_ms`.
    /// Paper grounding: arXiv:2604.22601 NL2VC. Model substitution
    /// is MANDATORY — Gemma 4 is BLOCKED per memory
    /// feedback_no_gemma4; substitutes from the paper's evaluation
    /// set include Codestral-22B-v0.1, GPT-OSS-120B, Qwen 3 Coder
    /// 30B. Live `dafny verify` execution is RUNPOD-PENDING; this
    /// variant exists today for the heuristic + offline-test path.
    DafnyVerifyCompleted,
    /// D-16 closure (SCHEMA_VERSION 127) — emitted once per
    /// [`crate::conformal_calibration::TauUpdater::update_tau`]
    /// durable persistence step. Carries a
    /// [`CalibrationCorpusUpdatedPayload`] with `corpus_size`,
    /// `new_tau`, `alpha` (UniCR Eq.5 miscoverage rate; DISTINCT
    /// from `rho` selective-risk budget per D-16-decisions.md
    /// DECISION-4), and `source` (the corpus provenance label). The
    /// existing `conformal_tau_computed` row family fires when the
    /// orchestrator computes tau in-process via
    /// `compute_tau_and_emit`; this new variant fires only from the
    /// persistent updater path so the SSB has a stable audit trail
    /// of every nightly threshold refresh. Paper grounding:
    /// arXiv:2509.01455 UniCR Eq.5 CRC quantile formula.
    CalibrationCorpusUpdated,
    /// E-12 Tier-B (SCHEMA_VERSION 128) — emitted once per task at
    /// scope-planner exit when [`crate::idea_framing::ProjectClass`]
    /// resolves to a
    /// [`crate::framework_profile::FrameworkProfile`] via
    /// `PROJECT_CLASS_TO_FRAMEWORK`. Carries a
    /// [`FrameworkProfileSelectedPayload`] with `profile_name_hash`
    /// (blake3-prefix 64-bit of canonical profile slug),
    /// `version_pin_hash` (blake3-prefix of the version pin string
    /// loaded from `workflow/archetypes/<name>.toml`),
    /// `language_hash` (blake3-prefix of the primary language tag),
    /// and producer `ts_ms`. Wire-dark posture: emission shim ships
    /// today; live framework auditor invocation is RUNPOD-PENDING
    /// per `workflow/reasoning/E-12-tier-b-decisions.md`.
    FrameworkProfileSelected,
    /// E-12 Tier-B (SCHEMA_VERSION 128) — emitted once per legacy
    /// linter config file the framework adapter declares stale and
    /// the C-32 flush-to-disk path removes (.eslintrc /
    /// .prettierrc / .flake8 / setup.cfg [flake8] /
    /// pyproject.toml [tool.black]/[tool.isort]). Carries a
    /// [`LegacyConfigStrippedPayload`] with `config_path_hash`
    /// (blake3-prefix of the removed path),
    /// `lint_engine_hash` (blake3-prefix of the replacement engine
    /// name e.g. `biome` / `ruff` / `clippy_deny`), and producer
    /// `ts_ms`.
    LegacyConfigStripped,
    /// D-18 closure (SCHEMA_VERSION 129) — one row per
    /// `task_runner::dispatch_observable_effect_hook` invocation that
    /// reaches the layer-8 dispatch arm (gated on
    /// `ELAI_OBSERVABLE_EFFECT_ENABLE`). Carries
    /// [`ObservableEffectDispatchedPayload`] (effect_id, trigger,
    /// context_kind, ts_ms). Producer:
    /// `crates/orchestrator/src/task_runner.rs::dispatch_observable_effect_hook`.
    /// Plan: docs/thefinalglance/D-plan/D-18_observable_effect_typed.md.
    /// Grounding: Voyager arxiv:2305.16291 §3 skill-with-measurable-effect.
    ObservableEffectDispatched,
    /// D-18 closure (SCHEMA_VERSION 129) — one row per
    /// `observable_effect::check_goodhart_risk` flag yielded for an
    /// `EffectSet`. Per Eureka arxiv:2310.12931 §3.3 component-variation
    /// guard. Carries [`GoodhartRiskFlagPayload`] (effect_id,
    /// risk_kind, evidence, ts_ms).
    GoodhartRiskFlag,
    /// D-18 closure (SCHEMA_VERSION 129) — one row per Coscientist-style
    /// `SandboxObservation` round-trip. Carries
    /// [`SandboxObservationPayload`] (effect_id, tier, outcome,
    /// evidence_count, ts_ms). Grounding: Coscientist arxiv:2304.05332
    /// §4 sandbox vs production observation taxonomy.
    SandboxObservation,
    /// D-18 closure (SCHEMA_VERSION 129) — one row per Coscientist-style
    /// `ProductionObservation` proposal. Carries
    /// [`ProductionObservationProposalPayload`] (effect_id,
    /// instrument_file, requires_human_review, ts_ms). Per Coscientist
    /// §4 invariant: `requires_human_review=true` MUST hold whenever
    /// `instrument_file` is None.
    ProductionObservationProposal,
    /// E-13 (SCHEMA_VERSION 130) — emitted once per
    /// `TddVerifierAgent::run_cycle` entry. Carries
    /// [`TddCycleStartedPayload`] (task_id_hash, ts_ms). Producer:
    /// `crates/orchestrator/src/tdd_verifier/agent.rs`.
    /// Grounding: LLMorpheus (arXiv:2404.09952) §2 TDD-cycle
    /// motivation — LLM-driven mutation testing requires a
    /// concrete cycle anchor so per-mutant kill-rate can be
    /// attributed back to a Red->Green transition.
    TddCycleStarted,
    /// E-13 (SCHEMA_VERSION 130) — emitted once per Red-phase test
    /// failure observed by the TDD agent driver. Carries
    /// [`TddTestFailedPayload`] (task_id_hash, test_name_hash,
    /// error_kind_hash, ts_ms). The dashboard's
    /// ReasoningDrawer surfaces these rows alongside `MutationSurvived`
    /// for the operator's per-cycle audit.
    TddTestFailed,
    /// E-13 (SCHEMA_VERSION 130) — emitted once per Red->Green
    /// flip with optional mutation-score evidence. Carries
    /// [`TddTestPassedPayload`] (task_id_hash, test_name_hash,
    /// optional mutation_score in [0,1], ts_ms). Mutation score is
    /// `None` when cargo-mutants did not run during this cycle.
    /// Grounding: ThinkPRM (arXiv:2504.16828) §3 process reward
    /// model evidence — the "test flipped" signal is the canonical
    /// non-LLM verifier under D-3 aggregation.
    TddTestPassed,
    /// Tranche B-7 triple (SCHEMA_VERSION 195) — TDD verification gate
    /// substrate covering E-13 PropertyGate / MutationGate-survivor /
    /// AntiMock signals + E-12 AssertionOnlyDiff. Carries
    /// [`TddVerificationEventPayload`] with the
    /// [`TddVerificationEventKind`] 4-arm discriminator. Pattern-2 (46th
    /// application). Wire-dark unless
    /// `ELAI_E13_TDD_VERIFICATION_EMIT_ENABLE=1`. Producer (cache arm):
    /// `crates/orchestrator/src/survival_cache.rs`.
    TddVerificationEvent,
    /// E-14 (SCHEMA_VERSION 131) — emitted once per browser-e2e journey
    /// run entry. Carries [`BrowserE2eRunStartedPayload`]
    /// (journey_name_hash, url_hash, ts_ms). Producer:
    /// `crates/orchestrator/src/browser_e2e/playwright.rs`. Grounding:
    /// Microsoft Playwright 2026 + GitHub url-health-check-action
    /// (wait-on-deploy primitive — surface the start signal so the
    /// dashboard can render the journey timeline).
    BrowserE2eRunStarted,
    /// E-14 (SCHEMA_VERSION 131) — emitted once per failing assertion in
    /// a journey run. Carries [`BrowserE2eAssertionFailedPayload`]
    /// (journey_name_hash, assertion_hash, error_hash, ts_ms). The agent
    /// driver may emit multiple rows per journey when several assertions
    /// fail. Grounding: BrowserStack 2026 flake-mitigation defaults —
    /// per-assertion granularity lets the operator separate flakes from
    /// genuine regressions.
    BrowserE2eAssertionFailed,
    /// E-14 (SCHEMA_VERSION 131) — emitted once per passing assertion in
    /// a journey run. Carries [`BrowserE2eAssertionPassedPayload`]
    /// (journey_name_hash, assertion_hash, ts_ms). Pairs with
    /// `BrowserE2eAssertionFailed` so dashboards can render pass/fail
    /// counts without polling the orchestrator.
    BrowserE2eAssertionPassed,
    /// E-14 (SCHEMA_VERSION 131) — emitted once per Playwright snapshot
    /// captured by the runner. Carries
    /// [`BrowserE2eSnapshotCapturedPayload`] (journey_name_hash,
    /// snapshot_kind_hash, ts_ms). `snapshot_kind_hash` is a blake3
    /// digest of the `SnapshotKind::as_str()` wire string
    /// (`html_only` / `html_and_screenshot` / `html_and_ax_tree` /
    /// `full`), letting the dashboard render the captured-artifact
    /// inventory without a separate column for each artifact type.
    BrowserE2eSnapshotCaptured,
    /// E-18 (SCHEMA_VERSION 132) — emitted once per Pass verdict from
    /// `tool_reliability::ToolReliabilityGate::check`. Carries
    /// [`ToolReliabilityGatePassedPayload`] (tool_name_hash,
    /// call_shape_hash, ts_ms). Composes AFTER the C-47
    /// `tool_call_logged` row so the operator can correlate the gate
    /// outcome with the case-base observation.
    ToolReliabilityGatePassed,
    /// E-18 (SCHEMA_VERSION 132) — emitted once per Reject verdict
    /// from the reliability gate. Carries
    /// [`ToolReliabilityGateRejectedPayload`] (tool_name_hash,
    /// reject_reason_hash, ts_ms). `reject_reason_hash` is a blake3
    /// digest of the snake-case reason label (`malformed_args` /
    /// `package_ref_rejected` / `retry_budget_exhausted` /
    /// `path_allowlist_violation`) so the dashboard can render the
    /// reject-cause histogram without a separate column per cause.
    ToolReliabilityGateRejected,
    /// E-18 (SCHEMA_VERSION 132) — emitted once per spill write. Carries
    /// [`ToolCallSpilledPayload`] (tool_name_hash, spill_path_hash,
    /// payload_bytes, ts_ms). Spill file lives at
    /// `.elai_cc/.elai_cc/spill/<ts>-<blake3-prefix>.json` per the
    /// sibling-convention enforced by `tool_reliability::spill::spill_call`.
    ToolCallSpilled,
    /// E-18 (SCHEMA_VERSION 132) — emitted once per Escalate verdict
    /// from `tool_reliability::should_escalate`. Carries
    /// [`ToolCallRetryEscalatedPayload`] (tool_name_hash, attempt,
    /// escalate_to_role_hash, ts_ms). `escalate_to_role_hash` is a
    /// blake3 digest of the destination `AgentRole` snake-case label
    /// so the dashboard renders the escalation-target histogram.
    ToolCallRetryEscalated,
    // E-37 (SCHEMA_VERSION 133): prompt-injection defense beacons.
    /// E-37 — emitted when the heuristic OR LLM classifier surfaces a
    /// Suspect verdict on an ingest span. Carries
    /// [`InjectionAttemptDetectedPayload`].
    InjectionAttemptDetected,
    /// E-37 — emitted when the classifier returns Malicious OR
    /// the capability_gate rejects argv carrying untrusted provenance.
    /// Carries [`InjectionAttemptBlockedPayload`].
    InjectionAttemptBlocked,
    /// E-37 — emitted once per `wrap_spotlight` envelope emitted around
    /// an untrusted ingest span. Carries [`SpotlightWrappedPayload`].
    SpotlightWrapped,
    /// E-37 — emitted when input contained marker bytes that would
    /// break the Spotlight envelope (attacker tried to forge wrapper).
    /// Carries [`SpotlightBreachedPayload`].
    SpotlightBreached,
    /// E-37 — emitted when per-tool blake3 description hash differs
    /// from the pinned value (`DescPinStatus::Drift`). Carries
    /// [`ToolDescriptionPinMismatchPayload`].
    ToolDescriptionPinMismatch,
    /// E-37 — emitted when destructive-tool argv carried at least one
    /// `TrustLevel::Untrusted` arg. Carries [`CapabilityGateRejectedPayload`].
    CapabilityGateRejected,
    /// E-37 — emitted when a value gets a non-trivial Provenance tag
    /// (ToolOutput, Web, SandboxOutput, etc). Sampled — NOT one row
    /// per dispatch. Carries [`ProvenanceTaggedPayload`].
    ProvenanceTagged,
    /// E-37 — emitted on every classify_injection call (Clean / Suspect /
    /// Malicious verdict) for verdict-rate observability. Carries
    /// [`PromptArmorVerdictPayload`].
    PromptArmorVerdict,
    /// E-37 — emitted per AgentDojo / EchoLeak / CometJacking fixture
    /// replay. RUNPOD-PENDING for live model runs; shape ships now so
    /// the fixture harness can land additively later. Carries
    /// [`InjectionFixtureReplayedPayload`].
    InjectionFixtureReplayed,
    /// E-37 U1 — emitted when the output sanitizer strips a non-allowlisted
    /// markdown image OR the output classifier flags a script / role-tag
    /// impersonation pattern. Carries [`OutputClassifierFlaggedPayload`].
    OutputClassifierFlagged,
    // E-40 (SCHEMA_VERSION 134): orchestrator-rooted symbol-graph beacons.
    /// E-40 — emitted once per `find_defs` / `find_refs` / `find_callers`
    /// / `find_callees` invocation from `elai symbol_graph ...` CLI
    /// dispatch or the Tauri `symbol_graph_query` command. Carries
    /// [`SymbolGraphQueriedPayload`].
    SymbolGraphQueried,
    /// E-40 — emitted once per `SymbolGraphCache::load` call (initial mount
    /// of `.claude/tools/elai_graph.json`). Carries
    /// [`SymbolGraphCacheLoadedPayload`].
    SymbolGraphCacheLoaded,
    /// E-40 — emitted once per `lsp_oracle::oracle_lookup` invocation.
    /// The dashboard surface counts these for the on-demand-only
    /// discipline metric (>5/session = bug per E-40-decisions.md). Carries
    /// [`LspOracleConsultedPayload`].
    LspOracleConsulted,
    /// E-44 (SCHEMA_VERSION 136): one row per
    /// `CheckpointManager::write` success. Carries a
    /// [`CheckpointWrittenPayload`].
    CheckpointWritten,
    /// E-44 (SCHEMA_VERSION 136): one row per successful
    /// `CheckpointManager::read_latest` consumed by a resume path.
    /// Carries a [`CheckpointResumedPayload`].
    CheckpointResumed,
    /// E-44 (SCHEMA_VERSION 136): one row per
    /// `durability::checkpoint_migrator::migrate_checkpoint` invocation
    /// that crossed at least one schema-version boundary. Carries a
    /// [`CheckpointMigratedPayload`].
    CheckpointMigrated,
    /// E-44 (SCHEMA_VERSION 136): one row per
    /// `IdempotentDispatcher` cache hit that returned a previously
    /// observed tool result without re-issuing the underlying
    /// dispatch. Carries an [`IdempotentDispatchSkippedPayload`].
    IdempotentDispatchSkipped,
    /// E-44 (SCHEMA_VERSION 136): one row per
    /// `CompactLock::try_acquire` call that returned
    /// `CompactLockError::Contended` — the PreCompact hook +
    /// 85% token compact race tripwire. Carries a
    /// [`DurabilityLockBlockedPayload`].
    DurabilityLockBlocked,
    /// E-45 (SCHEMA_VERSION 137): one row per
    /// `cross_lang_drift::DriftDetector::diff_pair` invocation that
    /// returned `>= 1` `DriftItem`. Carries a
    /// [`CrossLangDriftDetectedPayload`].
    CrossLangDriftDetected,
    /// E-45 (SCHEMA_VERSION 137): one row per
    /// `cross_lang_drift::DriftDetector::detect_pairs` repo walk
    /// regardless of drift outcome. Carries a
    /// [`CrossLangPairScannedPayload`].
    CrossLangPairScanned,
    /// E-45 (SCHEMA_VERSION 137): one row when the orchestrator
    /// detects a mismatch between Rust-side SCHEMA_VERSION and the
    /// sidecar's reported wire schema version. Carries a
    /// [`SchemaVersionMismatchPayload`].
    SchemaVersionMismatch,
    /// E-49 (SCHEMA_VERSION 138): one row per
    /// `coding_metrics::MetricsCollector::capture_metrics` call.
    /// Carries a [`CodingAgentMetricsCapturedPayload`].
    CodingAgentMetricsCaptured,
    /// E-49 (SCHEMA_VERSION 138): one row when a live SWE-Bench Pro
    /// run begins (RUNPOD-PENDING in this ship; the variant emission
    /// surface is stable). Carries a [`SweBenchProRunStartedPayload`].
    SweBenchProRunStarted,
    /// E-49 (SCHEMA_VERSION 138): one row at the end of a SWE-Bench
    /// Pro live run. Carries a [`SweBenchProRunCompletedPayload`].
    SweBenchProRunCompleted,
    /// E-49 (SCHEMA_VERSION 138): one row per RefineBench scoring
    /// pass. Carries a [`RefineBenchScoreEmittedPayload`].
    RefineBenchScoreEmitted,
    /// E-49 (SCHEMA_VERSION 138): one row per
    /// `coding_metrics::aggregate_session_metrics` call. Carries a
    /// [`MetricsAggregationRunPayload`].
    MetricsAggregationRun,
    // E-52 Tier-B (SCHEMA_VERSION 139): LSP-as-verifier diagnostic-probe +
    // SCIP refresh beacons. 4 atomic variants. The diagnostic-probe
    // surface composes the E-52 Tier-A envelope shipped at
    // `crates/knowledge/src/diagnostic.rs` (DiagnosticSeverity + ProbeKind
    // + Diagnostic + DiagnosticProbeOutcome). Live rust-analyzer/pylsp
    // JSON-RPC handshakes are RUNPOD-PENDING; the variant emission shape
    // is stable. See workflow/reasoning/E-52-decisions.md.
    /// E-52 Tier-B — one row per language-grouped PostToolUse probe
    /// dispatch (e.g. `.rs` files routed to rust-analyzer). Carries
    /// [`DiagnosticProbeDispatchedPayload`].
    DiagnosticProbeDispatched,
    /// E-52 Tier-B — one row per probe completion regardless of outcome
    /// (Completed / TimedOut / ToolMissing). Carries
    /// [`DiagnosticProbeCompletedPayload`].
    DiagnosticProbeCompleted,
    /// E-52 Tier-B — one row when error-severity diagnostics blocked the
    /// EXECUTING -> AUDITING transition. Carries
    /// [`DiagnosticProbeBlockPayload`].
    DiagnosticProbeBlock,
    /// E-52 Tier-B — one row per `.claude/tools/elai.map` incremental
    /// SCIP merge invocation. Carries [`ScipRefreshCompletedPayload`].
    ScipRefreshCompleted,
    /// E-53 (SCHEMA_VERSION 140) — emitted once per
    /// [`crate::formal_verify::orchestrator::run`] pipeline boundary
    /// regardless of outcome (Verified / Refuted / Inapplicable /
    /// RunpodPending). Carries a [`FormalVerifierCompletedPayload`]
    /// with `verifier_kind_hash` (blake3 prefix of
    /// [`crate::formal_verify::VerifierKind::as_str`] — one of
    /// `z3_smt` / `dafny` / `kani` / `verus`), `outcome_hash` (blake3
    /// prefix of [`crate::formal_verify::VerifierOutcome::as_str`] —
    /// one of `verified` / `refuted` / `inapplicable` /
    /// `runpod_pending`), `csq_class_hash` (blake3 prefix of the
    /// [`crate::csq::CounterfactualClass::as_str`] gating decision),
    /// `contract_emitted` (whether the C-50 structural-contract
    /// emission step ran before dispatch), `duration_ms`, and
    /// producer `ts_ms`. Live Z3 / Kani / Verus dispatch is
    /// RUNPOD-PENDING; Dafny dispatch reuses C-52
    /// `invoke_dafny_cli`. Gemma-4 BLOCKED per memory
    /// `feedback_no_gemma4` (substrate inheritance from C-52). See
    /// workflow/reasoning/E-53-decisions.md.
    FormalVerifierCompleted,
    /// E-35 (SCHEMA_VERSION 141) — XGrammar mask was applied to the
    /// answer-phase token stream. Carries
    /// [`XgrammarMaskAppliedPayload`] with `schema_id_hash`,
    /// `schema_hash_hash`, `mask_applied_count`, `generation_ms`,
    /// `cache_hit`, `ts_ms`. Producer: the sidecar XGrammar hook
    /// post-`generate_structured` (RUNPOD-PENDING for Python wiring;
    /// host-side caller is `codeact_sandbox::execute_code_act` and
    /// the CLI subverb `elai structured-output grammar emit-test`).
    XgrammarMaskApplied,
    /// E-35 (SCHEMA_VERSION 141) — INSTRUCTOR-mode structured
    /// decoding selected for a (role, block) pair. Reasoning roles
    /// (Auditor / Planner / Decomposer / IdeaFramer) default to this
    /// per CRANE. Carries [`InstructorModeActivatedPayload`].
    InstructorModeActivated,
    /// E-35 (SCHEMA_VERSION 141) — GRAMMAR-mode structured decoding
    /// selected for a low-reasoning enum/header block (BabyStepStatus,
    /// FailureTaxonomy, ReasoningTemplate type-tag). Carries
    /// [`GrammarModeActivatedPayload`].
    GrammarModeActivated,
    /// E-35 (SCHEMA_VERSION 141) — one CodeAct-style python snippet
    /// dispatched through `codeact_sandbox::execute_code_act`. Carries
    /// [`CodeactActionExecutedPayload`] with `action_hash`,
    /// `line_count`, `outcome_hash` (ok|sandbox_absent|cap_exceeded|
    /// allowlist_violation), `duration_ms`, `ts_ms`. AgentRole::Runner
    /// addition DEFERRED per E41_AGENTROLE_FROZEN_AT_17 gotcha; this
    /// variant emits from the sandbox dispatcher itself.
    CodeactActionExecuted,
    /// E-35 (SCHEMA_VERSION 141) — SEARCH/REPLACE tier-1 matcher
    /// succeeded on a file edit (custom non-git ELAI fences). Carries
    /// [`SearchReplaceTier1HitPayload`] with `file_path_hash`,
    /// `edit_count`, `ts_ms`.
    SearchReplaceTier1Hit,
    /// E-35 (SCHEMA_VERSION 141) — SEARCH/REPLACE DelimiterCollision
    /// guard fired; matcher fell back to whole-file rewrite tier.
    /// Carries [`SearchReplaceCollisionFallbackPayload`] with
    /// `file_path_hash`, `collision_token_hash`, `ts_ms`.
    SearchReplaceCollisionFallback,
    /// E-35 (SCHEMA_VERSION 141) — CloudGroundingClient passed a
    /// provider-strict-schema directive (Anthropic
    /// `structured-outputs-2025-11-13` beta header / OpenAI
    /// `strict:true` / Gemini `responseJsonSchema`). Carries
    /// [`CloudStrictSchemaAppliedPayload`] with `provider_hash`,
    /// `schema_id_hash`, `ts_ms`. Header propagation to the
    /// AnthropicClient surface is RUNPOD-PENDING (provider crate
    /// extension); this beacon records the intent.
    CloudStrictSchemaApplied,
    /// E-35 (SCHEMA_VERSION 141) — sidecar boot completed grammar
    /// cache warmup. Carries [`GrammarCacheLoadedPayload`] with
    /// `compiled_count`, `cache_hits`, `cache_misses`,
    /// `total_compile_ms`, `ts_ms`. Producer is the sidecar's
    /// `compile_all_schemas_at_boot` (RUNPOD-PENDING); host-side
    /// emitter is the CLI `grammar emit-test` smoke + the contract
    /// test.
    GrammarCacheLoaded,
    /// E-35 (SCHEMA_VERSION 141) — caller requested a structured-
    /// output engine (lm-format-enforcer) that is EXPLICITLY rejected
    /// in favour of the canonical XGrammar substrate. Carries
    /// [`StructuredEngineRejectedPayload`] with `engine_hash`,
    /// `reason_hash`, `ts_ms`. Refuse loud; never silent fallback.
    StructuredEngineRejected,
    /// E-35 (SCHEMA_VERSION 141) — caller requested the legacy vendor
    /// `json_object` mode; CloudGroundingClient refuses + returns a
    /// typed error. Carries [`LegacyJsonObjectRefusedPayload`] with
    /// `provider_hash`, `ts_ms`.
    LegacyJsonObjectRefused,
    // E-6 (SCHEMA_VERSION 148): MAST-grounded multi-agent failure-mode
    // defenses. Four atomic variants in lockstep with the 9-tuple
    // SsbActionType cascade (enum + SQL CHECK + as_sql_str + FromStr +
    // SsbPayload arm + action_type() arm + payload struct +
    // action_type_lock.rs ALL_VARIANTS + exhaustive_match_guard + .golden).
    // Source: workflow/reasoning/E-6-decisions.md DECISION-5.
    // Citation grounding (per DECISION-1, ID_COLLISION substitution):
    //   * arXiv:2503.13657 (Cemri et al. 2025, MAST) — PRIMARY ANCHOR for
    //     all four variants. Original plan cite 2406.12624 was an
    //     ID_COLLISION; gotchas.md entry E6_CITATION_2406_12624_ID_COLLISION.
    //   * IsoBudgetMeasured — arXiv:2604.02460 (Tran & Kiela iso-budget)
    //     + arXiv:2507.02778 ("Wait" reflection token, 89.3% blind-spot
    //     reduction).
    //   * PersonaReanchored — arXiv:2511.09710 Echoing + arXiv:2412.00804
    //     Identity Drift + arXiv:2601.04170 Agent Drift.
    //   * CrossFamilyUnavailable — arXiv:2507.02778 (64.5% same-model
    //     blind-spot rate; cross-family aux signal is the structural fix).
    /// E-6 (SCHEMA_VERSION 148) — per-level auditor verdict that produced a
    /// BLOCK has been attributed to a specific MAST failure mode. Carries
    /// [`AuditMastTaggedPayload`] with `verdict_seq` (SSB seq of the
    /// originating PerLevelAuditVerdict row), `mast_mode` (snake_case tag
    /// from `MastMode::as_sql_str`), `confidence_band` (`low` / `medium`
    /// / `high`), and `agent_role`. Producer: per_level_audit_dispatch
    /// follow-up wiring (E-6.b patch) + manual emit from contract tests
    /// in this pass. ENV gate: `ELAI_FAILURE_TRACKER_ENABLE=1`.
    AuditMastTagged,
    /// E-6 (SCHEMA_VERSION 148) — iso-budget ablation harness completed
    /// one 2-row run (full-pipeline vs single-agent-with-Wait). Carries
    /// [`IsoBudgetMeasuredPayload`] with `task_fixture`, `budget_tokens`,
    /// per-row pass_rate, `insufficient_lift` flag, `runpod_pending`
    /// flag, and aggregated `mast_mode_counts`. Producer:
    /// `iso_budget_harness::run_ablation`. Live ablation is RUNPOD-PENDING
    /// (host-side scaffold returns `runpod_pending=true` until the live
    /// multi-agent path lands).
    IsoBudgetMeasured,
    /// E-6 (SCHEMA_VERSION 148) — persona re-anchor fired for an agent.
    /// Carries [`PersonaReanchoredPayload`] with `agent_role`, `turn_count`,
    /// `reason` (`turn_counter` / `cosine_drift` / `no_trigger`),
    /// `cosine_similarity` (None when no EmbedProvider injected), and
    /// `embedding_available`. Producer: `persona_anchor::emit_ssb_row`.
    /// Cosine threshold default 0.55 UNCONFIRMED — calibration sweep
    /// RUNPOD-PENDING per plan acceptance #5.
    PersonaReanchored,
    /// E-6 (SCHEMA_VERSION 148) — cross-family adversary route was
    /// requested but is unavailable AND `ELAI_ALLOW_CLOUD_ADVERSARY=0`.
    /// Refusal SSB row so the dashboard can render the structural
    /// blocker (per plan task 4 + workflow/reasoning/E-6-decisions.md
    /// DECISION-7). Carries [`CrossFamilyUnavailablePayload`] with
    /// `executor_family`, `phase_kind`, `claim_id`, and `cloud_allowed`.
    /// Producer: any caller of `adversary::route_cross_family` that
    /// receives `Err(SameFamilyPairing)` AND the phase requires adversary.
    CrossFamilyUnavailable,
    /// E-7 (SCHEMA_VERSION 149) — darwinian self-improvement substrate
    /// generation completion beacon. SINGLE-VARIANT scope-lean per
    /// Pattern-2 (13th application). The per-phase discriminator
    /// [`DarwinianPhase`] on the payload covers all 11 fire-points:
    /// mutation, selection, evaluation, two_gate_blocked,
    /// est_invariance_measured, contam_filter_rejected,
    /// transfer_regression_blocked, trap_tripped, archive_sig_verified,
    /// archive_sig_broken, archive_cell_elevated. Carries
    /// [`DarwinianGenerationCompletedPayload`]. Wire-dark unless
    /// `ELAI_DARWINIAN_LOOP_ENABLE=1` AND the orchestrator is compiled
    /// with `--features darwinian-evolve`. Live model evolution loops
    /// RUNPOD-PENDING. Paper grounding:
    ///   - arXiv:2510.04399 (Statistical Limits of Self-Improving
    ///     Agents, Wang/Dorchen/Jin) — PAC-theoretic Two-Gate guardrail
    ///     (margin tau AND capacity K_m). NOT AlphaEvolve.
    ///   - arXiv:2305.10718 (Discounted Thompson Sampling) —
    ///     non-stationary bandit fit for per-cell parent selection.
    ///   - arXiv:2506.13131 (AlphaEvolve) — MAP-Elites + island-model.
    ///   - arXiv:2507.05619 (EST) — invariance probe.
    ///   - arXiv:2507.19457 (GEPA, ICLR 2026 Oral) — background-only.
    ///   - OpenEvolve community implementations — background-only.
    ///   - Krakovna spec-gaming spreadsheet — trap-task pattern seeds.
    /// REMOVED grounding: arXiv:2602.12413 (wrong paper; benchmark
    /// contamination, zero relevance; see DECISION-3).
    /// See workflow/reasoning/E-7-decisions.md.
    DarwinianGenerationCompleted,
    /// E-3 (SCHEMA_VERSION 150) — runtime handoff signal beacon emitted
    /// when one model hands control to another (Begin) and again when
    /// the receiving runtime acknowledges (Complete). Carries
    /// [`RuntimeHandoffSignalPayload`] with a [`HandoffSignalPhase`]
    /// discriminator. Wire-dark unless `ELAI_RUNTIME_HANDOFF_ENABLE=1`.
    /// See workflow/reasoning/E-3-decisions.md.
    RuntimeHandoffSignal,
    /// E-26 (SCHEMA_VERSION 151) — SGLang HTTP runtime adapter
    /// registration beacon. Emitted once at adapter init recording the
    /// resolved base-url hash + structured-output backend selector.
    /// Producer: `elai_providers::runtime::SglangRuntime::new`.
    /// Wire-dark unless `ELAI_SGLANG_BASE_URL` is set.
    SglangBackendRegistered,
    /// E-26 (SCHEMA_VERSION 151) — XGrammar grammar-compile observation
    /// beacon. Carries [`XgrammarSchemaCompiledPayload`] with schema
    /// hash + compile latency (ms). Producer: SGLang adapter's
    /// structured-output dispatch path.
    XgrammarSchemaCompiled,
    /// E-26 (SCHEMA_VERSION 151) — runtime cold-start observation
    /// beacon. Carries elapsed_ms from first-request to first-response.
    /// Producer: SGLang adapter's health-probe path.
    RuntimeColdStartObserved,
    /// E-2 (SCHEMA_VERSION 152) — streaming-completion timing
    /// histogram beacon. Emitted from the dashboard-side timing collector
    /// in `commands.rs::start_chat_stream`. See
    /// workflow/reasoning/E-2-decisions.md DECISION-8.
    StreamingTimingRecorded,
    /// E-2 (SCHEMA_VERSION 153) — UX pacing vote A/B beacon. Emitted
    /// when the user votes "snappy" / "slow" via the streaming pacing
    /// affordance.
    UxPacingVoteRecorded,
    /// E-28 (SCHEMA_VERSION 154) — Streaming-LLM attention sink beacon.
    /// Emitted when `ELAI_STREAMING_LLM_ENABLE>=1` and the
    /// attention-sink mask is applied. Paper: arXiv:2309.17453 Sec 4.2
    /// (paper-recommended sink_count >= 4).
    StreamingLlmEngaged,
    /// E-28 (SCHEMA_VERSION 155) — SDPA-backend-pin beacon. Emitted once
    /// at boot recording the resolved SDPA backend (closes
    /// pytorch#133780 silent MATH fallback).
    SdpaBackendPinned,
    /// E-28 (SCHEMA_VERSION 156) — YaRN rope-scaling harness beacon.
    /// Emitted from the test_yarn_eval_gate.py harness path; live boot
    /// still refuses YaRN per E-32. Paper: arXiv:2501.15383 Qwen2.5-1M.
    YarnRopeScalingApplied,
    /// E-8 (SCHEMA_VERSION 162) — self-evolving prompt optimization
    /// completion beacon. SINGLE-VARIANT scope-lean per Pattern-2 (14th
    /// application). The per-phase discriminator [`OptimizerPhase`] on
    /// the payload covers all 7 fire-points: dspy_init, opro_step,
    /// rank_prompt_vote, ape_proposal, gepa_reflect, archive_promote,
    /// archive_retire. Carries [`PromptOptimizationCompletedPayload`].
    /// Wire-dark unless `ELAI_PROMPT_OPTIMIZER_ENABLE=1` AND the
    /// orchestrator is compiled with `--features darwinian-evolve`
    /// (re-uses the E-7 feature gate per E-8-decisions.md DECISION-3).
    /// Live optimization loops RUNPOD-PENDING.
    /// Paper grounding:
    ///   - arXiv:2507.19457 (GEPA, Agrawal et al. ICLR 2026 Oral) —
    ///     reflective prompt evolution / Pareto frontier.
    ///   - arXiv:2310.03714 (DSPy, Khattab et al.) — declarative LM
    ///     pipelines / MIPROv2 optimizer surface.
    ///   - arXiv:2309.03409 (OPRO, Yang et al.) — LLMs-as-optimizers
    ///     meta-prompt trajectory.
    ///   - arXiv:2211.01910 (APE, Zhou et al.) — automatic instruction
    ///     search prior-art.
    ///   - arXiv:2405.10276 (Zhang et al. ACL 2024) — small-LMs-as-
    ///     optimizers limitation: forces cloud-reflection / local-target
    ///     split at the CLI surface.
    ///   - arXiv:2504.20355 (LPO) — `<edit>` tag pattern for span-level
    ///     freeze.
    ///   - arXiv:2602.00997 (ETGPO) — top-3 gotchas injection / ~1/3
    ///     optimization-token cost claim.
    /// REMOVED grounding: arXiv:2403.12373 (RankPrompt) — paper-triager
    /// flagged WRONG_REGIME (reasoning-quality method, not prompt
    /// optimizer). RankPromptVote phase retained as a forward-compatible
    /// slot; no live wiring depends on it in the E-8 ship.
    /// See workflow/reasoning/E-8-decisions.md.
    PromptOptimizationCompleted,
    /// E-10 (SCHEMA_VERSION 163) -- hostile-critic structural simplifications
    /// decision beacon. SINGLE-VARIANT scope-lean per Pattern-2 (15th
    /// application). The per-phase discriminator [] on the
    /// payload covers all 9 fire-points: fsm_ablation, role_ablation,
    /// rag_challenger, mistake_book_signal, typed_cot_verified, tier_matrix,
    /// cloud_default, grc_blockquote, prompt_cache_hit_rate. Carries
    /// []. Wire-dark unless
    /// . Live critic loops RUNPOD-PENDING.
    /// Paper grounding:
    ///   - arXiv:2502.00674 (Self-MoA) -- N=3 same-model sampling for the
    ///     role-ablation 3-role Verifier composition.
    ///   - arXiv:2503.13657 (MAST) -- 14-mode failure taxonomy that critic
    ///     emissions bind into via .
    ///   - arXiv:2510.01069 (typed-CoT certificates) -- Curry-Howard
    ///     mechanical verification for the cot_proof tile.
    ///   - arXiv:2305.01219 (ProAttack) + arXiv:2401.05949 (ICLAttack) --
    ///     ICL-poisoning defenses motivating MistakeBook structured signal.
    ///   - arXiv:2307.13702 (Lanham) + arXiv:2510.04040 (FaithCoT-Bench) --
    ///     CoT faithfulness motivating the GRC-blockquote empirical eval.
    /// See workflow/reasoning/E-10-decisions.md.
    CriticDecisionEvent,
    /// E-43 (SCHEMA_VERSION 164) -- agent skill systems telemetry beacon.
    /// SINGLE-VARIANT scope-lean per Pattern-2 (16th application). The
    /// per-phase discriminator [`SkillEventPhase`] on the payload covers
    /// 3 fire-points: loaded (registry discovery), invoked (PostToolUse
    /// hook telemetry), failed (validator / eval / scan failure).
    /// Carries [`SkillSystemEventPayload`] with skill_name + blake3
    /// prompt_hash prefix + outcome. Wire-dark unless
    /// `ELAI_SKILL_SYSTEM_ENABLE=1`. Live skill invocations
    /// RUNPOD-PENDING. Paper grounding:
    ///   - agentskills.io/specification (Anthropic SKILL.md v1)
    ///   - snyk.io/blog/toxicskills-malicious-ai-agent-skills-clawhub
    ///   - venturebeat 2026 (Anthropic scanner bypass via tests/)
    ///   - vercel.com/blog/agents-md-outperforms-skills-in-our-agent-evals
    /// See workflow/reasoning/E-43-decisions.md.
    SkillSystemEvent,
    /// E-9 (SCHEMA_VERSION 165) -- external agent-eval benchmark suite
    /// observation beacon. SINGLE-VARIANT scope-lean per Pattern-2 (17th
    /// application). The per-phase discriminator [`BenchPhase`] on the
    /// payload covers 5 fire-points: loaded (suite registered), started
    /// (run dispatched), item_evaluated (per-task verdict), completed
    /// (run aggregated with paired-bootstrap p-value), aborted (run
    /// halted via tripwire / power-calibration refusal). Carries
    /// [`BenchmarkRunObservedPayload`] with the suite_name string
    /// discriminating across the 11 registered external benchmark suites
    /// (swe-bench-pro, swe-bench-live, livecodebench-v6, tau2-bench,
    /// bfcl-v3, gaia-l1-l2, clawsbench-split, refinebench,
    /// sycophancy-probes, math-floors, deep-planning). Wire-dark unless
    /// `ELAI_EVAL_BENCHMARK_ENABLE=1`. Live model evals RUNPOD-PENDING.
    /// Paper grounding:
    ///   - arXiv:2509.16941 (SWE-Bench Pro) -- primary cite (RAG-indexed).
    ///   - arXiv:2505.23419 (SWE-bench-Live) -- monthly refresh.
    ///   - arXiv:2511.22173 (RefineBench) -- self-refinement ceiling.
    ///   - arXiv:2604.05172 (ClawsBench) -- harness-vs-model split.
    ///   - arXiv:2311.12983 (GAIA) -- general agent baseline.
    ///   - arXiv:2601.18137 (DeepPlanning) -- multi-day horizon.
    ///   - arXiv:2506.07982 (tau2) -- policy-adherence dual-control.
    ///   - arXiv:2403.07974 (LiveCodeBench) -- time-stamped.
    ///   - arXiv:2501.03124 (PRMBench) -- adversarial gaming.
    ///   - arXiv:2310.13548 (Sharma sycophancy) -- 4-behavior probes.
    ///   - arXiv:2406.07791 (Judging the Judges) -- LLM-as-judge bias.
    ///   - arXiv:2306.05685 (MT-Bench foundational) -- judge methodology.
    /// REMOVED (paper-ingestor could not resolve per
    /// workflow/sessions/s-25120/completion-drive/grounding/
    /// E-9_agent_eval_benchmarks-grounding.md DECISION-3): "DS-CP
    /// arXiv:2510.05566", focused.io blog, "MetaMate".
    /// See workflow/reasoning/E-9-decisions.md.
    BenchmarkRunObserved,
    /// E-19 (SCHEMA_VERSION 166) -- reasoning quality probe beacon.
    /// SINGLE-VARIANT scope-lean per Pattern-2 (18th application). The
    /// per-kind discriminator [`ProbeKind`] on the payload covers 5
    /// fire-points: l2_gate_decision (pre-L2 confidence gate verdict;
    /// also carries the L1->L3 direct-route + no-CoT-class sub-flags via
    /// the `verdict` payload string), l4_prm_lite_score (per AB-MCTS
    /// candidate node score from `prm_lite::AuditorVerifierScore`),
    /// l4_lateral_kept (LToT lateral branch retained one extra ply),
    /// l4_ttc_budget_chosen (difficulty-estimator dispatched budget),
    /// l6_verifier_diversity (fresh-seed L1 + rotated L3 agreement
    /// verdict). Carries [`ReasoningQualityProbePayload`] with the
    /// `kind` discriminator + scalar/string context fields. Wire-dark
    /// unless `ELAI_REASONING_QUALITY_PROBE_ENABLE=1`. Plan task 4
    /// PRM-lite + task 5 LToT rely on the B-41
    /// `per_level_audit_dispatch::record_verdict` surface (the plan-doc
    /// claimed `orchestrator::auditor::dispatch_audit` does not exist
    /// under that name). Paper grounding:
    ///   - arXiv:2510.08308 First Try Matters -- L2 confirmatory
    ///     reflection
    ///   - arXiv:2502.18581 Self-Certainty -- logit-margin gate (also
    ///     re-anchors logit-as-noisy-proxy claim)
    ///   - arXiv:2510.01500 LToT -- mainline/lateral split
    ///   - arXiv:2503.04412 AB-MCTS -- paper assumes external feedback;
    ///     PRM-lite is a documented deviation
    ///   - arXiv:2502.08788 Vanilla MAD -- model heterogeneity (NOT
    ///     "calibration") for L6 diversity
    ///   - arXiv:2410.21333 Mind-Your-Step -- no-CoT regime
    ///   - arXiv:2510.01069 Typed CoT -- wiring-quality gain on ELAI
    ///     stack (NOT a math-eval claim)
    ///   - arXiv:2502.10954 + arXiv:2408.03314 -- difficulty-conditional
    ///     TTC budget
    ///   - arXiv:2507.02778 Self-Correction Bench -- L3 over L2 when L1
    ///     low-confidence
    ///   - arXiv:2510.08308 First Try Matters -- nearest support for
    ///     hard L2 token cap (cited as engineering-heuristic anchor only;
    ///     paper proposes dynamic truncation, not a fixed 240-token cap)
    /// DROPPED per E-19 grounding-triage:
    ///   - arXiv:2602.12113 ARLCP (WRONG_REGIME — training-time RL,
    ///     cannot ground an inference-time token cap)
    ///   - arXiv:2307.13702 Lanham (WRONG_REGIME — CoT-text faithfulness,
    ///     not logit signal quality)
    /// See workflow/reasoning/E-19-decisions.md.
    ReasoningQualityProbe,
    /// E-54 (SCHEMA_VERSION 167) -- eval-corpus repair pre-deploy.
    /// SINGLE-VARIANT scope-lean per Pattern-2 (19th application). The
    /// per-phase discriminator [`CorpusEventPhase`] on the payload covers
    /// 14 fire-points: ingest_rejected_ngram / ingest_rejected_cosine /
    /// shard_created / schema_violation_no_tests / test_fails_on_reference /
    /// test_useless_passes_broken / kor_orthogonality_violated /
    /// kor_orthogonality_passed / sycophancy_framing_gap_detected /
    /// sandbagging_anomaly / generator_rotation_violated /
    /// perf_delta_contamination_suspected / dynamic_knowledge_qa_generated /
    /// prod_traffic_probe_injected. Carries [`EvalCorpusEventPayload`] with
    /// the `phase` discriminator + scalar/string context fields (task_id,
    /// family, details). Wire-dark unless `ELAI_EVAL_CORPUS_ENABLE=1`. Repairs
    /// D-6's still-PLANNED 9-gram filter pre-deploy with 13-gram +
    /// embedding-cosine + per-family directory shards + test-triplet schema +
    /// matched-pair sycophancy + KOR-orthogonality. CanaryFamily (5-arm) is
    /// the canonical task-family surface; KnowledgeQa is routed via
    /// ToolUse-flavored tasks. Paper grounding:
    ///   - arXiv:2311.04850 -- Yang & Chiang Rephrased Samples (13-gram
    ///     threshold; primary anchor)
    ///   - arXiv:2602.12413 -- Soft Contamination (re-triaged CENTRAL in
    ///     E-54 domain; E-7 ZERO_RELEVANCE was darwinian-domain-specific)
    ///   - arXiv:2403.07974 -- LiveCodeBench v6 (cutoffs schema; uses
    ///     release-date stamps, NOT monthly-Docker per FIX-3)
    ///   - arXiv:2410.06526 -- KOR-Bench (cross-family transfer concept;
    ///     accuracy-delta operationalization is ELAI-specific)
    ///   - arXiv:2503.02951 -- KodCode (test-triplet schema; <2.5% error
    ///     verified)
    ///   - arXiv:2411.04905 -- OpenCoder (test-execution filter)
    ///   - arXiv:2510.20487 -- Steering Eval-Aware (matched-pair concept;
    ///     the Sonnet 4.5 >80% verbalization stat is Anthropic, NOT the
    ///     paper per FIX-4)
    ///   - arXiv:2508.00943 -- Covert Sandbagging
    ///   - arXiv:2510.04721 -- BrokenMath (perturbed_pair schema)
    ///   - arXiv:2412.08905 -- Phi-4 (generator rotation lesson)
    ///   - arXiv:2509.00072 -- Test-of-Time (synthesis can OBSCURE)
    ///   - arXiv:2511.08598 -- OKBench (dynamic knowledge-QA)
    ///   - arXiv:2503.14499 -- METR Time Horizons (rationale for dynamic
    ///     gen)
    ///   - arXiv:2511.17602 -- Hierarchical Contamination (F1=0.76 anchor;
    ///     0.85 cosine default is ELAI-empirical INITIAL DEFAULT pending
    ///     Open Q1 tune per FIX-2)
    ///   - arXiv:2410.12784 -- JudgeBench (replaces plan's WRONG_ID
    ///     2412.12509 per FIX-1)
    ///   - arXiv:2505.23419 -- SWE-bench Live (architectural template for
    ///     per-family shard)
    ///   - arXiv:2403.04132 -- Chatbot Arena (Task 10 prod-traffic tier)
    /// Live ingest + LoRA cross-family transfer + sandbox test_executor +
    /// sidecar embeddings + LLM-grader matched-pair invocation
    /// RUNPOD-PENDING. See workflow/reasoning/E-54-decisions.md.
    EvalCorpusEvent,
    /// E-55 (SCHEMA_VERSION 168) -- agentic RL + reward ladder substrate.
    /// SINGLE-VARIANT scope-lean per Pattern-2 (20th application). The
    /// per-kind discriminator [`RewardLadderEventKind`] on the payload
    /// covers 9 fire-points: rlaif_judge_rated /
    /// rlaif_calibration_correlation_low / gepa_template_evolved /
    /// gepa_template_evicted / reward_fusion_computed /
    /// reward_hacking_suspected / sft_training_threshold_reached /
    /// bai_arm_observation_recorded / rl_weight_update_unlocked. Carries
    /// [`RewardLadderEventPayload`] with the `kind` discriminator +
    /// scalar / string context fields (session_id, trajectory_id,
    /// details_json). Wire-dark unless `ELAI_AGENTIC_RL_ENABLE=1`.
    ///
    /// Deferred RL ladder: RLAIF judge (cloud Sonnet on local-Qwen
    /// rollouts) + GEPA reflective prompt evolution + hybrid PRM+ORM
    /// reward fusion + MONA composite-reward hacking guards +
    /// SFT-on-rollouts collection surface + 3-condition unlock gate for
    /// DPO/PPO/GRPO/KTO weight-update modules (which DO NOT exist by
    /// design until unlock). Paper grounding (post-triage):
    ///   - arXiv:2212.08073 Constitutional AI (Anthropic)
    ///   - arXiv:2309.00267 RLAIF vs RLHF (Lee et al.)
    ///   - arXiv:2507.19457 GEPA (ICLR 2026 oral; 35x rollout efficiency)
    ///   - arXiv:2501.13011 MONA (Farquhar et al. Google DeepMind)
    ///   - arXiv:2509.15557 Verifiable Composite Rewards medical reasoning
    ///   - arXiv:2507.01352 Skywork-Reward-V2 (outcome-dominant default)
    ///   - arXiv:2412.21139 SWE-Gym ICML 2025 (200-rollout lower bound)
    ///   - arXiv:2508.16546 RL Is Neither Panacea Nor Mirage
    ///   - arXiv:2501.17161 SFT Memorizes, RL Generalizes ICML 2025
    ///   - arXiv:2410.12784 JudgeBench (replaces plan WRONG_ID 2412.12509
    ///     per E-55 triage FIX-1; quarterly calibration cadence)
    ///   - Nature 2025 AlphaProof URL
    ///     <https://www.nature.com/articles/s41586-025-09833-y> (replaces
    ///     plan FATAL bad arXiv:2406.15829 per E-55 triage)
    ///   - arXiv:2305.18290 Rafailov DPO (cited ONLY to JUSTIFY rejection)
    ///   - arXiv:2502.20847 Gradient Imbalance in DPO
    ///   - arXiv:2402.01306 KTO (future-tier if/when unlock)
    ///   - arXiv:2502.14187 Federated KTO vs DPO
    ///   - arXiv:2308.08998 ReST (background SFT-on-rollouts lineage)
    ///   - arXiv:2411.15124 Tulu 3 / RLVR
    ///   - arXiv:2402.19446 ArCHer (future-tier multi-turn hierarchical RL)
    ///   - arXiv:2410.01679 VinePPO (future-tier)
    ///   - arXiv:2503.06639 GRPO loss dynamics (future-tier)
    ///   - arXiv:2504.06141 Adv-RM (reward-hacking canary rationale)
    ///
    /// DPO is HARD-AVOIDED forever for ELAI's data shape (Apple 2025 OOD
    /// drop + arXiv:2502.20847 gradient imbalance + ELAI has no paired
    /// preference data). PPO/GRPO/DAPO/KTO HARD-DEFERRED until 3 unlock
    /// conditions all TRUE (BAI saturation + ORM coverage >=80% + VRAM
    /// headroom >=4GB). Live cloud Sonnet rollouts + LoRA training
    /// RUNPOD-PENDING. See workflow/reasoning/E-55-decisions.md.
    RewardLadderEvent,
    /// E-11 (SCHEMA_VERSION 169) -- SaaS prior-art catalog + recommender
    /// substrate. SINGLE-VARIANT scope-lean per Pattern-2 (21st
    /// application). The per-kind discriminator [`PriorArtEventKind`]
    /// on the payload covers 2 fire-points:
    ///   * `archetype_recommended` -- in-process call to
    ///     `architecture::recommend_archetype` produced an
    ///     ArchetypeRecommendation; payload carries primary archetype +
    ///     confidence + clarifying-question count.
    ///   * `prior_art_consulted` -- caller (idea-critic /
    ///     production-grade auditor / dashboard recommender panel)
    ///     requested `PriorArtRegistry::entries_for_archetype`; payload
    ///     carries consulted archetype + result count + caller site
    ///     label.
    ///
    /// Carries [`SaasPriorArtEventPayload`] with the `kind`
    /// discriminator + session_id + archetype string + confidence
    /// string + free-form `details_json`. Wire-dark unless
    /// `ELAI_SAAS_PRIOR_ART_ENABLE=1`.
    ///
    /// Substrate slice: the catalog + recommender + 15-entry prior-art
    /// registry live at `crate::architecture` (5-archetype taxonomy
    /// locked by e50.A guard; ProjectClass FROZEN at 9 variants per
    /// e12.A guard). Per-archetype SecurityCheckId subset is the
    /// consumption point for the AuditorMode::ProductionGrade dispatch
    /// path. Sibling slices NOT in this commit: scope_clarity_score on
    /// CriticFinding extension, package_verifier with sqlite cache,
    /// anti-spiral hard-abort EscalationTrigger extension
    /// (autonomous_gate.rs:400+, NOT anti_spiral_extended.rs),
    /// edit_format Unified/SearchReplace/FullFile downgrade chain.
    ///
    /// 2024-2026 idea-to-app survey grounding documented in
    /// `workflow/improvement-sweep-2026-05-19/reports/
    /// 11-idea-to-saas-prior-art.md`. See
    /// `workflow/reasoning/E-11-decisions.md`.
    SaasPriorArtEvent,
    /// E-22 (SCHEMA_VERSION 170) -- may-2026 github trending sweep
    /// substrate. SINGLE-VARIANT scope-lean per Pattern-2 (22nd
    /// application). The per-kind discriminator [`TrendingEventKind`]
    /// on the payload covers 3 fire-points:
    ///   * `skill_export_normalized` -- elai skills-mgmt export emitted
    ///     a normalised SKILL.md to dist/<target>/skills/<name>/; payload
    ///     carries skill_name + drift_count (blake3 diff against any
    ///     prior dist copy) + dist_path label.
    ///   * `microvm_fallback` -- orchestrator::sandbox::select_backend
    ///     requested the Microsandbox backend but degraded to a fallback
    ///     (linux+KVM only with MICROSANDBOX_BIN unset, OR windows
    ///     AppContainer); payload carries reason_label + fallback_backend
    ///     snake_case label.
    ///   * `lazy_schema_savings` -- DefaultMcpBridge::http_list_tools
    ///     returned a {name,description} lazy summary instead of the
    ///     full input_schema; payload carries pre_bytes (estimated full
    ///     schema bytes) + post_bytes (lazy summary bytes) + tool_count.
    ///
    /// Carries [`GithubTrendingEventPayload`] with the `kind`
    /// discriminator + label fields + free-form `details_json`. Wire-dark
    /// unless `ELAI_GITHUB_TRENDING_ENABLE=1`. Source report:
    /// `workflow/improvement-sweep-2026-05-19/reports/
    /// 22-github-trending-may2026.md`. Grounded by Anthropic SKILL.md
    /// spec (code.claude.com/docs/en/skills), microsandbox README
    /// (github.com/superradcompany/microsandbox, YC X26 + libkrun), and
    /// arXiv:2604.21816 "Tool Attention Is All You Need" (95% MCP-tax
    /// reduction on a 120-tool 6-server benchmark; VERIFIED 2026-05-27
    /// per grounding triage). See
    /// `workflow/reasoning/E-22-decisions.md`.
    GithubTrendingEvent,
    /// E-23 (SCHEMA_VERSION 171) -- 2026 developer-discourse positioning
    /// + UX-guard substrate. SINGLE-VARIANT scope-lean per Pattern-2 (23rd
    /// application). The per-kind discriminator [`DiscourseEventKind`] on
    /// the payload covers 3 fire-points:
    ///   * `trust_telemetry_aggregated` -- TrustTelemetryPanel periodic
    ///     aggregation across existing SSB rows (productivity proxy +
    ///     verification density + revert signal + ungrounded count +
    ///     stagnation distribution).
    ///   * `runtime_choice_badge_emitted` -- CommandWindow status-row
    ///     badge transition between "local" and "cloud -- billed" labels,
    ///     derived from B-37 runtime_fallback rows.
    ///   * `context_state_warning` -- ContextStatePanel 70% / 85% token-
    ///     budget threshold crossing the user sees before PreCompact
    ///     truncates the conversation.
    ///
    /// Carries [`DeveloperDiscourseEventPayload`] with the `kind`
    /// discriminator + label fields + free-form `details_json`. Wire-dark
    /// unless `ELAI_DEVELOPER_DISCOURSE_ENABLE=1`. Source report:
    /// `workflow/improvement-sweep-2026-05-19/reports/
    /// 23-developer-discourse-2026.md`. Grounded by arXiv:2507.09089
    /// (METR RCT, 19% slowdown with felt-20%-speedup gap), Stack Overflow
    /// 2025 developer survey (84% AI use / 29% trust), JetBrains AI
    /// Pulse April 2026 (90% regular use; Claude Code 91% CSAT / NPS 54),
    /// and Anthropic issue #13171 (context loss UX). See
    /// `workflow/reasoning/E-23-decisions.md`.
    DeveloperDiscourseEvent,
    /// E-25 (SCHEMA_VERSION 172) -- claude-code-architecture catalog
    /// substrate. SINGLE-VARIANT scope-lean per Pattern-2 (24th
    /// application). The per-kind discriminator
    /// [`crate::architecture::claude_code_arch::ClaudeCodeArchitectureEventKind`]
    /// on the payload covers 9 fire-points, one per primitive in the
    /// canonical 9-entry taxonomy
    /// [`crate::architecture::claude_code_arch::ClaudeCodeArchPrimitive`]:
    ///   * `compaction_ladder_tier_applied` -- a future 6-tier
    ///     compaction-ladder run applied one of T1..T6 tiers; payload
    ///     carries tier id + freed_tokens + wall_ms.
    ///   * `skill_template_executed` -- a SKILL.md ``!`cmd``` block was
    ///     scanned and stdout substituted before the skill reached the
    ///     model.
    ///   * `prompt_decision_made` -- the LLM-decision hook handler
    ///     returned Allow / Deny(reason) / Ask(reason).
    ///   * `opusplan_overlay_activated` -- a runtime-substate overlay
    ///     rerouted `assistant_primary` to a planner-runtime or
    ///     executor-runtime per the opusplan model-config alias.
    ///   * `session_forked` -- `elai resume --fork` cloned
    ///     SESSION_HANDOFF + memory-bank + FSM state into a new session.
    ///   * `edit_checkpoint_snapshotted` -- pre-edit snapshot was
    ///     persisted under `.elai_cc/.session-checkpoints/` with blake3
    ///     integrity hashing.
    ///   * `bash_complexity_blocked` -- a Bash command with >50
    ///     subcommands was rejected (anti-pattern guard against the
    ///     Claude Code per-subcommand parsing safety-bypass).
    ///   * `approval_fatigue_detected` -- auto-approval rate exceeded
    ///     the configured threshold over the rolling 50-decision
    ///     window (anti-pattern guard against the move-the-boundary
    ///     decision-mask pattern documented by VILA-Lab).
    ///   * `sandbox_template_warned` -- a skill-template ``!`cmd```
    ///     block ran outside microsandbox; jail fallback applied.
    ///
    /// Carries [`ClaudeCodeArchitectureEventPayload`] with the `kind`
    /// discriminator + canonical primitive label + optional session id
    /// + free-form `details_json`. Wire-dark unless
    /// `ELAI_CLAUDE_CODE_ARCH_ENABLE=1`.
    ///
    /// Substrate slice: the 9-primitive taxonomy + 9-arm fire-point
    /// discriminator + renamed [`ClaudeCodeLadderResult`] struct (resolves
    /// the hard name-collision with [`crate::compact::CompactionResult`])
    /// + 6-tier [`CompactionTierId`] enum live at
    /// `crate::architecture::claude_code_arch`. Concrete implementation
    /// modules (compaction_ladder.rs, cli/fork.rs, edit_checkpoints.rs,
    /// prompt_handler.rs, dashboard panels) are FOLLOW-UPS tracked in
    /// `workflow/reasoning/E-25-decisions.md`. Primitive 2 (deferred MCP
    /// tool descriptions) is DESCOPED entirely pending the E-22 Task 3
    /// / C-35 lazy_schemas substrate; the catalog entry will be added in
    /// a follow-up once that mechanism ships.
    ///
    /// Source grounding:
    ///   * karanprasad.com/blog/how-claude-code-actually-works-reverse-
    ///     engineering-512k-lines (6-tier ladder + 98.4% deterministic
    ///     infra split + `cache_edits` server-side parameter).
    ///   * code.claude.com/docs/en/how-claude-code-works (compaction +
    ///     checkpoints snapshot-before-edit + `--fork-session` /
    ///     `/branch`).
    ///   * code.claude.com/docs/en/skills (dynamic context injection +
    ///     live change detection).
    ///   * code.claude.com/docs/en/hooks (`"type": "prompt"` handler).
    ///   * code.claude.com/docs/en/model-config (opusplan alias).
    ///   * github.com/VILA-Lab/Dive-into-Claude-Code (pre-trust execution
    ///     window + per-subcommand >50 bypass + approval-fatigue masking).
    ///   * infoq.com/news/2026/05/anthropic-claude-code-postmortem
    ///     (6-week-undetected quality regression -- telemetry guard
    ///     rationale).
    ClaudeCodeArchitectureEvent,
    /// E-29 Tier-B (SCHEMA_VERSION 173) -- agentic-observability event
    /// family. SINGLE-VARIANT scope-lean per Pattern-2 (25th
    /// application). The per-fire-point discriminator
    /// [`AgenticObservabilityEventKind`] on the payload covers 4
    /// fire-points:
    ///   * `mast_failure_classified` -- the offline MAST classifier
    ///     ([`crate::observability::mast_classify::classify_closed_run`])
    ///     emitted a hit per the 14-variant
    ///     [`crate::mast::MastMode`] taxonomy. Payload carries the mode
    ///     tag + evidence_row_ids + confidence_bp.
    ///   * `replay_resume` -- a `ssb_resume_from(checkpoint_id, patch)`
    ///     validator call completed
    ///     ([`crate::observability::replay::ssb_resume_from`]). Payload
    ///     carries the outcome (Refused / PredicateUnavailable /
    ///     Accepted) + replay_session_id + refusal_reason.
    ///   * `signal_match` -- the Laminar-style natural-language
    ///     classifier matched a free-text signal across SSB rows.
    ///     Payload carries the signal_label + matched_row_ids +
    ///     classifier_model + confidence_bp.
    ///   * `eval_dataset_member` -- a production trace was promoted to
    ///     an eval dataset (Langfuse / LangSmith pattern). Payload
    ///     carries trace_id + dataset_name + label + scoring_rubric.
    ///
    /// Carries [`AgenticObservabilityEventPayload`] with the `kind`
    /// 4-arm discriminator + per-kind scalar fields + free-form
    /// `details_json`. Wire-dark unless
    /// `ELAI_AGENTIC_OBSERVABILITY_ENABLE=1`.
    ///
    /// DRIFT-2 collision avoidance: the pre-existing
    /// [`Self::FailureClassified`] variant carries the 13-category
    /// SYMPTOM taxonomy ({category, confidence_bp, step_id, source})
    /// owned by `crate::regression`. The E-29 MAST CAUSE classifier
    /// uses this NEW family to avoid the action_type collision
    /// documented in `workflow/reasoning/E-29-decisions.md`.
    ///
    /// Source grounding:
    ///   * arXiv:2503.13657 -- Cemri et al. NeurIPS 2025, MAST
    ///     (14-mode taxonomy, k=0.79 generalization).
    ///   * laminar.sh/article/2026-04-23-top-6-agent-observability-
    ///     platforms (Signals + SQL-over-traces transcript-first).
    ///   * github.com/langfuse/langfuse (traces-to-datasets promotion).
    ///   * dev.to/sreeni5018 (LangGraph checkpoint replay pattern).
    ///   * tianpan.co/blog/2026-04-12 (full I/O capture needed for
    ///     deterministic replay -- documented OUT-OF-SCOPE risk).
    ///   * traceloop.com/docs/openllmetry (`TRACELOOP_TRACE_CONTENT`
    ///     content-gating pattern; ELAI default-off).
    ///   * opentelemetry.io/docs/specs/semconv/gen-ai/gen-ai-agent-
    ///     spans (1.40.0 semconv, status still Development per May
    ///     2026 sweep).
    AgenticObservabilityEvent,
    /// E-31 (SCHEMA_VERSION 174) -- coding-model-leaderboard scraper
    /// substrate. SINGLE-VARIANT Pattern-2 (26th application). The
    /// per-kind discriminator [`LeaderboardEventKind`] on the payload
    /// covers 3 fire-points:
    ///   * `scrape_started` -- scraper began an HTTP fetch against a
    ///     leaderboard URL; payload carries suite_name + suite_version +
    ///     leaderboard_url + scrape_run_id.
    ///   * `scrape_completed` -- scraper finished parsing a leaderboard
    ///     and emitted one row per model entry; payload carries
    ///     model_name + score_pct (basis points, 0-10000) + sample_size +
    ///     scrape_url + entry_hash (blake3 8-byte prefix of the canonical
    ///     row tuple).
    ///   * `scrape_failed` -- scraper hit HTTP non-200, parse error, or
    ///     ENV-gate refusal; payload carries reason_label + http_status
    ///     (0 if non-HTTP) + suite_name.
    ///
    /// Carries [`CodingModelLeaderboardEventPayload`] with the `kind`
    /// discriminator + per-kind scalar fields + free-form `details_json`.
    /// Wire-dark unless `ELAI_CODING_MODEL_LEADERBOARD_ENABLE=1`. Scraper
    /// at `benches/external/coding_model_leaderboard.py` emits stdout
    /// JSON rows parsed by the rust dispatcher (NOT a runtime
    /// orchestrator hook -- the live cloud-route policy table lives in
    /// E-31.B per workflow/reasoning/E-31-leaderboard-decisions.md).
    ///
    /// Companion non-bumped surface (E-31.A SHIPPED):
    /// `crates/contracts/tests/head_dim_invariant_contract.rs` guards
    /// the dense-128 invariant at the config-json layer.
    ///
    /// Source grounding:
    ///   * arXiv:2409.12186 -- Qwen2.5-Coder technical report
    ///     (verified arxiv.org/abs/2409.12186 May 2026).
    ///   * arXiv:2509.16941 -- SWE-Bench Pro paper (verified
    ///     arxiv.org/abs/2509.16941 May 2026).
    ///   * morphllm.com/swe-bench-pro -- Pro vs Verified contamination
    ///     gap (45.9 vs 80.9 on Opus 4.5; verified May 2026).
    ///   * labs.scale.com/leaderboard/swe_bench_pro_public -- SWE-Bench
    ///     Pro public dataset leaderboard (verified May 2026).
    ///   * bigcode-bench.github.io -- BigCodeBench leaderboard ICLR'25
    ///     (verified May 2026).
    ///   * evalplus.github.io/leaderboard.html -- HumanEval+ / MBPP+
    ///     leaderboard (verified May 2026).
    ///   * livecodebench.github.io/leaderboard.html -- contamination-
    ///     free time-windowed leaderboard (verified May 2026).
    ///   * platform.claude.com/docs/en/about-claude/pricing -- Anthropic
    ///     2026 API pricing reference (Sonnet 4.6 $3/$15, Opus 4.7
    ///     $5/$25, Haiku 4.5 $1/$5 per MTok; verified May 2026).
    CodingModelLeaderboardEvent,
    /// E-38 (SCHEMA_VERSION 175) -- 2026 AI roadmap signals + EU AI Act
    /// compliance pre-gate substrate. SINGLE-VARIANT Pattern-2 (27th
    /// application). The per-kind discriminator [`RoadmapSignalKind`] on
    /// the payload covers 5 fire-points:
    ///   * `compliance_blocked` -- EU AI Act compliance pre-gate refused
    ///     PLANNING/FEASIBILITY_PROBE -> TESTING_FIRST transition. Payload
    ///     carries `assertion_label` + `hint` + `plan_session_id`.
    ///   * `compliance_verified` -- compliance pre-gate passed; payload
    ///     carries `plan_session_id` + `assertions_checked` bitmask.
    ///   * `mcp_transport_version_logged` -- DefaultMcpBridge::new emitted
    ///     a transport-version beacon. Payload carries
    ///     `mcp_transport_current` + `mcp_transport_target`.
    ///   * `mrcr_unverified` -- a backend declared
    ///     `declared_max_context_tokens` >= 100K without a measured
    ///     `mrcr_v2_score`. Payload carries `backend_name` +
    ///     `declared_context_tokens` + `mrcr_score_label`.
    ///   * `posture_doc_stale` -- `docs/strategy/ELAI-2026-POSTURE.md` is
    ///     older than the 90-day refresh window. Payload carries
    ///     `posture_doc_path` + `posture_doc_mtime_ms` + `age_days`.
    ///
    /// Wire-dark unless `ELAI_AI_ROADMAP_SIGNALS_ENABLE=1`. Producers:
    /// orchestrator::compliance (compliance arms), plugins::mcp_bridge
    /// (transport arm), orchestrator::research::roadmap_signals (mrcr +
    /// posture arms). See workflow/reasoning/E-38-decisions.md.
    AiRoadmapSignalEvent,
    /// E-39 (SCHEMA_VERSION 176) -- cost + token economics substrate.
    /// SINGLE-VARIANT Pattern-2 (28th application). The per-kind
    /// discriminator [`CostEventKind`] on the payload covers 8 fire-
    /// points:
    ///   * `task_aborted_by_spend_cap` -- mid-task cooperative abort
    ///     because `task_spend_usd_cents >= max_spend_per_task_cents`.
    ///   * `velocity_cap_breached` -- 1-min/10-min/1-hour/24-hour
    ///     rolling-window counter exceeded its limit.
    ///   * `automix_verifier_verdict` -- local Qwen 3B verifier emitted
    ///     a Continue/Retry/Escalate verdict between L3 failure and
    ///     HYBRID cloud escalation.
    ///   * `cost_router_decision` -- pre-execution difficulty estimator
    ///     + budget-remaining picked Local / Sonnet / Haiku /
    ///     DeepSeekFlash / Opus.
    ///   * `cost_estimate_inaccurate` -- post-call actual / pre-call
    ///     estimate ratio outside [0.5, 2.0]; calibration feedback.
    ///   * `free_tier_quota_approaching` -- backend at >=80% of daily
    ///     quota; warn.
    ///   * `free_tier_quota_exceeded` -- backend at >=95% of daily
    ///     quota; refuse + abort.
    ///   * `layered_cap_manual_override` -- user invoked `elai cloud
    ///     override-cap` with typed-confirmation; composes with E-38
    ///     EU AI Act audit-trail liveness gate.
    ///
    /// Carries [`CostTokenEventPayload`] with the `kind` discriminator
    /// + session id + task id + free-form `details_json` so future
    /// kinds extend without a schema bump. Wire-dark unless
    /// `ELAI_COST_TOKEN_ECONOMICS_ENABLE=1`.
    ///
    /// Producers:
    /// [`crate::cost::cloud_abort::abort_task_on_cap_breach`]
    /// + [`crate::cost::velocity_cap::VelocityCapEnforcer::enforce`]
    /// + [`crate::cost::automix_gate::verify_before_escalate`]
    /// + [`crate::cost::cost_router::route_for_difficulty_and_budget`]
    /// + [`crate::cost::free_tier_guard::FreeTierGuard::check_quota`]
    /// + the `elai cloud override-cap` CLI subcommand emit site.
    ///
    /// Paper grounding: arXiv:2310.12963 AutoMix +
    /// arXiv:2604.22750 token-consumption variance Stanford 2026 +
    /// arXiv:2305.05176 FrugalGPT. Industry grounding: $47K LangChain
    /// wallet-loop (ravoid.com), 2.3M-call weekend retry storm
    /// (leanopstech.com). See workflow/reasoning/E-39-decisions.md.
    CostTokenEvent,

    /// E-50 -- SaaS architecture pattern catalog substrate.
    /// SINGLE-VARIANT Pattern-2 (29th application);
    /// [`SaasArchPatternEventKind`] N-arm discriminator covers the
    /// 11 fire-points the May 2026 SaaS architecture survey calls out:
    ///
    ///   * `production_readiness_gate_blocked` -- one of the 6 day-1
    ///     checks failed (admin-route-auth / rate-limit / Sentry-or-OTel
    ///     / secret-scan / RLS-lint / webhook-signature).
    ///   * `production_readiness_gate_passed` -- all applicable checks
    ///     passed for the catalog-selected archetype.
    ///   * `rls_baby_step_emitted` -- Designer emitted a per-table RLS
    ///     baby step under `MultiTenancyModel::RowLevelSecurity`.
    ///   * `unscoped_token_blocked` -- scoped-token enforcer refused a
    ///     deploy step using an unscoped Railway / Fly / Cloudflare
    ///     CLI token (PocketOS Apr 2026 9s wipe defense).
    ///   * `billing_mor_decision_made` -- (geography, mrr_projection)
    ///     resolved to a [`MorVendor`] selection (or Stripe-direct).
    ///   * `realtime_block_decided` -- Designer fixed [`RealtimeNeed`]
    ///     based on the idea-text keyword trigger (default `None`).
    ///   * `async_backbone_decided` -- latency budget + archetype
    ///     resolved to [`AsyncBackbone`] (Serverless / Inngest /
    ///     TriggerDev / Modal / Hatchet).
    ///   * `per_tier_rate_limit_derived` -- billing tier names mapped
    ///     to per-tier rate-limit rows.
    ///   * `otel_slo_runbook_scaffolded` -- per-archetype SLO scaffold
    ///     emitted with OneUptime 14x / 6x / 1x multi-burn-rate
    ///     defaults.
    ///   * `platform_unsupported_for_scope_check` -- enforcer fell back
    ///     to comment-marker because the platform (Vercel / AWS) has
    ///     no first-class scope-pinning field.
    ///   * `archetype_block_overridden` -- Designer recorded a manual
    ///     stack-layer override (vendor swap inside the chosen
    ///     archetype).
    ///
    /// Carries [`SaasArchPatternEventPayload`] with the `kind`
    /// discriminator + session id + archetype snake_case + free-form
    /// `details_json` so future kinds extend without a schema bump.
    /// Wire-dark unless `ELAI_SAAS_ARCH_PATTERNS_ENABLE=1`.
    ///
    /// Producer:
    /// [`crate::architecture::saas_patterns`] +
    /// downstream Designer / Auditor wiring (lands per-callsite in
    /// follow-up patches; this slice provides the substrate +
    /// deterministic decision functions).
    ///
    /// See workflow/reasoning/E-50-decisions.md.
    SaasArchPatternEvent,

    /// E-4 -- chat UX critique substrate (sycophancy / confidence-chip /
    /// pinned-rules / re-anchor / doom-loop / completion-claim /
    /// clarifier / refusal-override / mid-stream / honest-correction).
    /// SINGLE-VARIANT Pattern-2 (30th application);
    /// [`ChatUxCritiqueKind`] 13-arm discriminator covers the
    /// fire-points the chat-UX critique report ranks as the top
    /// trust-impact-per-effort fixes:
    ///
    ///   * `claim_confidence_tier_emitted` -- AUDITOR stage emitted a
    ///     [`ClaimWithConfidence`] inline pill. Payload carries the
    ///     tier label (verified / read_from_source / from_training /
    ///     guessed) + optional evidence_ref + claim text hash.
    ///   * `sycophancy_pushback_detected` -- AUDITOR overrode the
    ///     REASONER's stance based on user-disagreement evidence; the
    ///     model held position with a citation rather than agreeing
    ///     to be agreeable (the canonical sycophancy anti-pattern).
    ///   * `user_rule_pinned` -- user pinned an ad-hoc constraint to
    ///     the PinnedRulesPanel surface.
    ///   * `user_rule_unpinned` -- user removed a pinned constraint
    ///     (click-to-remove path on the panel).
    ///   * `session_re_anchored` -- after N turns or before a
    ///     destructive action, system prompt + first user message +
    ///     active rules were re-injected.
    ///   * `doom_loop_detected` -- DoomLoopDetector fired (3 same-file
    ///     edits / 3 failed hypotheses / auditor BLOCK streak / test
    ///     pass-rate plateau) per `.claude/skills/anti-spiral/`.
    ///   * `completion_claim_rejected_no_evidence` -- AUDITOR rejected
    ///     a "done" / "complete" / "finished" output with no
    ///     evidence_ref (file diff / audit verdict / det-check pass).
    ///   * `one_shot_clarifier_emitted` -- TRIAGE stage decided the
    ///     input was ambiguous and emitted exactly ONE clarifier
    ///     card (per arXiv:2507.21285 "Curiosity by Design").
    ///   * `one_shot_clarifier_resolved` -- user answered the
    ///     clarifier; pipeline proceeds without further clarification.
    ///   * `refusal_emitted` -- ELAI refused an action (security /
    ///     anti-pattern rule). Payload carries refusal_reason +
    ///     refusal_scope (`local_codebase` -> overrideable;
    ///     `outside_project` -> hard refuse, no override).
    ///   * `refusal_overridden_by_user` -- user clicked the
    ///     refusal-override button; action re-runs with the named
    ///     rule bypassed for that single dispatch.
    ///   * `mid_stream_interrupt_committed` -- user typed a
    ///     correction mid-stream; the in-flight stream was cancelled
    ///     (BoxStream drop) and the conversation restarted with the
    ///     correction prepended.
    ///   * `honest_correction_emitted` -- ELAI explicitly retracted a
    ///     prior claim with the canonical "I was wrong. Correct: X
    ///     because Y." pattern (Anthropic constitution + CLAUDE.md).
    ///
    /// Carries [`ChatUxCritiqueEventPayload`] with the `kind`
    /// discriminator + session id + optional turn id + free-form
    /// `details_json` so future kinds extend without a schema bump.
    /// Wire-dark unless `ELAI_CHAT_UX_CRITIQUE_ENABLE=1`.
    ///
    /// Producer: [`crate::chat_ux`] (substrate module).
    ///
    /// Paper / industry grounding (verified May 2026):
    ///   * arXiv:2310.13548 Sharma "Towards Understanding Sycophancy"
    ///   * openai.com/index/sycophancy-in-gpt-4o (April 2025 rollback)
    ///   * aclanthology.org/2024.tacl-1.9 "Lost in the Middle"
    ///   * arXiv:2412.00804 "Examining Identity Drift in LLM Agents"
    ///   * arXiv:2507.21285 "Curiosity by Design" (one-shot clarifier)
    ///   * arXiv:2509.08833 "Pitfalls of Over-Alignment"
    ///   * HN 47936579 (Claude Code false-completion thread)
    ///   * anthropic.com/constitution ("diplomatic honesty rather than
    ///     dishonest diplomacy")
    ///   * warpedvisions.org/blog/2025/getting-unstuck-when-using-
    ///     llms-for-coding (doom-loop UX)
    ///
    /// arXiv:2604.* IDs cited in the plan body remain supporting-only
    /// pending WebSearch bulk verification; see
    /// workflow/reasoning/E-4-decisions.md.
    ChatUxCritiqueEvent,
    /// E-16 -- VLM image-validation hardening event. SINGLE-VARIANT
    /// Pattern-2 (31st application). Carries
    /// [`VlmImageValidationEventPayload`] with the `kind`
    /// discriminator (3 arms: `swap_order_disagreement`,
    /// `axtree_mismatch`, `vlm_schema_unconstrained`) + `step_id` +
    /// `request_id` + free-form `details_json` so each kind's per-kind
    /// scalars ride along without per-kind payload structs (Pattern-2
    /// scope-lean). Wire-dark unless
    /// `ELAI_VLM_IMAGE_VALIDATION_ENABLE=1`.
    ///
    /// Notably this variant does NOT cover the `sycophancy_probe`
    /// inverted-framing fire-point: that path REUSES the existing
    /// [`Self::SycophancyFlag`] variant shipped via E-4 / D-14 (the
    /// `SycophancyFlagPayload` already encodes the
    /// convergent / divergent_sycophancy_flag / inconclusive verdict
    /// vocab and the dashboard renders it via SycophancyFlagStrip).
    ///
    /// Paper grounding (verified May 2026):
    ///   * arXiv:2408.11261 "Sycophancy in Vision-Language Models"
    ///     (LQCD leading-query contrastive decoding mitigation)
    ///   * arXiv:2406.07791 "Judging the Judges" (15-judge x 22-task
    ///     x ~40-model >150K-instance position-bias systematic study)
    ///   * playwright.dev/docs/aria-snapshots (v1.59+ ariaSnapshot API)
    ///   * arXiv:2411.15100 XGrammar (constrained-decoding mask
    ///     emission, supporting-only)
    /// arXiv:2604.05839 (RCE convergence, plan body) NOT VERIFIED on
    /// WebSearch; tagged supporting-only.
    VlmImageValidationEvent,
    /// E-20 -- local-cloud routing substrate. SINGLE-VARIANT Pattern-2
    /// (32nd application). Carries [`LocalCloudRoutingEventPayload`]
    /// with the `kind` discriminator (9 arms: `confidence_emitted` /
    /// `hybrid_escalation` / `difficulty_classified` / `privacy_blocked` /
    /// `output_filtered` / `burn_alert` / `reasoning_mode` /
    /// `majority_vote` / `role_priority_chosen`) + free-form
    /// `details_json` so each kind's per-kind scalars (entropy score,
    /// threshold, verdict, match spans, redaction count, burn USD,
    /// vote tally, role+priority pair) ride along without per-kind
    /// payload structs (Pattern-2 single-variant scope-lean).
    /// Wire-dark unless `ELAI_LOCAL_CLOUD_ROUTING_ENABLE=1`.
    ///
    /// Note that `privacy_blocked` here is the COUNTERPART rather than
    /// the replacement for the pre-existing
    /// [`Self::RouteBlocked`] (C-5 ordinal 331) -- when the L1 secret
    /// scanner refuses a cloud dispatch, callers continue to emit
    /// `RouteBlocked` with `reason = "privacy_block"` for the
    /// cap-aware dashboard surface AND additionally emit
    /// `LocalCloudRoutingEvent { kind: PrivacyBlocked }` with the
    /// match-span / severity payload that does not belong on
    /// `RouteBlockedPayload` (which is shaped around USD cap fields).
    ///
    /// Paper grounding (verified May 2026 via WebSearch):
    ///   * arXiv:2511.07396 C3PO Nov 2025 conformal-cost-bounded
    ///     cascade (anchor)
    ///   * arXiv:2406.18665 RouteLLM LMSYS 2x cost reduction
    ///   * arXiv:2310.12963 AutoMix >50% cost reduction
    ///   * arXiv:2510.01237 Confidence-Aware Routing (probe + perplexity
    ///     dominate verbalized-self-confidence; F1 0.61 -> 0.82)
    ///   * arXiv:2510.08731 When to Reason (-47.1% latency / -48.5%
    ///     tokens / +10.2pp MMLU-Pro)
    ///   * arXiv:2502.00674 Self-MoA (anti-pattern documentation:
    ///     mixing different LLMs gets dragged by weakest member)
    ///   * arXiv:2511.03808 Prompt-Difficulty Prediction
    ///   * arXiv:2305.05176 FrugalGPT (up to 98% cost cut at GPT-4
    ///     parity)
    /// See workflow/reasoning/E-20-decisions.md.
    LocalCloudRoutingEvent,
    /// E-21 (SCHEMA_VERSION 181) -- peer-systems harness substrate
    /// beacon (SBFL Ochiai input, inference-time critic-rerank
    /// pick/skip, repo-state-store snapshot, ACI audit-rendered).
    /// SINGLE-VARIANT scope-lean per Pattern-2 (33rd application).
    /// Carries [`PeerSystemsEventPayload`] with the `kind`
    /// discriminator (4 arms: `critic_candidates_generated` /
    /// `critic_score_computed` / `critic_pick_committed` /
    /// `critic_rerank_skipped`) + free-form `details_json`. Wire-dark
    /// unless `ELAI_PEER_SYSTEMS_ENABLE=1` (see
    /// [`crate::peer_systems::peer_systems_emit_enabled`]).
    ///
    /// DISTINCT from existing surface:
    ///   * [`Self::PagerankRun`] (C-3/C-38) already covers PageRank
    ///     emissions; this variant does NOT duplicate that.
    ///   * [`crate::mini_critic::MiniCritic`] is the role-time critic
    ///     (signature linter); this variant tracks the SEPARATE
    ///     inference-time N-candidate reranker
    ///     (`peer_systems::critic_rerank`).
    ///
    /// Paper grounding (verified May 2026 via context-compiler):
    ///   * arXiv:2404.05427 (AutoCodeRover, ISSTA 2024) -- Ochiai
    ///     formula for SBFL substrate (37.3% SWE-lite pass@1)
    ///   * arXiv:2405.15793 (SWE-agent, NeurIPS 2024) -- ACI principles
    ///   * openhands.dev Nov 2025 blog -- inference-time scaling +
    ///     critic-model rerank (the rerank method)
    ///   * arXiv:2509.23045 (Kimi-Dev) -- agentless training as skill
    ///     prior (philosophy alignment for harness-first)
    ///
    /// See workflow/reasoning/E-21-decisions.md (Tier-A + Tier-B).
    PeerSystemsEvent,
    /// E-36 (SCHEMA_VERSION 182) — document + vision RAG substrate.
    /// SINGLE-VARIANT Pattern-2 (34th application). Carries
    /// [`DocumentVisionRagEventPayload`] with the `kind` discriminator
    /// (5 arms: `vision_chunk_emitted` / `vision_fallback_triggered` /
    /// `vision_rag_policy_decision` / `visual_artifact_check_verdict` /
    /// `doc_parser_cloud_fallback_blocked_by_mode`) + free-form
    /// `details_json`. Wire-dark unless
    /// `ELAI_DOCUMENT_VISION_RAG_ENABLE=1` (see
    /// [`ELAI_DOCUMENT_VISION_RAG_ENABLE_ENV`]).
    ///
    /// Producers:
    ///   * `orchestrator::document_vision_rag::visual_verifier` —
    ///     emits `visual_artifact_check_verdict` rows carrying the
    ///     `reliability` tier (VlmCaption=LOW / OcrParse=MED /
    ///     DiagramSource=HIGH). HARD RULE per plan task 5: downstream
    ///     gates MUST downweight VlmCaption tiers.
    ///   * `orchestrator::document_vision_rag::policy` — emits
    ///     `vision_rag_policy_decision` (storage / enabled gate) and
    ///     `doc_parser_cloud_fallback_blocked_by_mode` (when
    ///     `--mode local` refuses the cloud parser).
    ///   * `knowledge::chunking::image_artifact::ImageArtifactChunker` —
    ///     emits `vision_chunk_emitted` per indexed VisionArtifact.
    ///   * `knowledge::chunking::pdf_structured::PdfStructuredChunker` —
    ///     emits `vision_fallback_triggered` when `parser_confidence`
    ///     drops below `[knowledge.doc_parser] cloud_fallback_threshold`
    ///     (default 0.7).
    ///
    /// 10th VerificationMethod variant
    /// (`reasoning::types::VerificationMethod::VisualArtifactCheck`)
    /// dispatches into this substrate. Reuses the existing
    /// `claim_category` field rather than introducing a parallel
    /// `claim_class` (per workflow/reasoning/E-36-decisions.md GAP-7).
    ///
    /// Paper grounding (verified May 2026 via WebSearch):
    ///   * arXiv:2510.14528 PaddleOCR-VL (0.9B, 92.56 OmniDocBench v1.5)
    ///   * arXiv:2505.14059 Dolphin-v2 (ACL 2025 handwriting)
    ///   * arXiv:2407.01449 ColPali (ICLR 2025 late-interaction)
    ///   * arXiv:2602.12510 Visual RAG Toolkit (training-free pooling
    ///     ~10x storage reduction; Feb 2026 publication verified)
    ///   * arXiv:2502.12342 REAL-MM-RAG (vision-vs-text recall
    ///     differential; informs RRF default 0.6 weight)
    ///
    /// See workflow/reasoning/E-36-decisions.md (Tier-A + Tier-B).
    DocumentVisionRagEvent,
    /// E-42 (SCHEMA_VERSION 183) -- tree-sitter grammar health substrate.
    /// SINGLE-VARIANT Pattern-2 (35th application). Carries
    /// [`TreeSitterGrammarEventPayload`] with the `kind` discriminator
    /// (4 arms: `abi_probe_mismatch` / `svelte_runes_parse_health` /
    /// `tier_b_grammar_loaded` / `tree_sitter_skipped_large_file`) +
    /// free-form `details_json`. Wire-dark unless
    /// `ELAI_TREE_SITTER_GRAMMARS_ENABLE=1` (see
    /// [`ELAI_TREE_SITTER_GRAMMARS_ENABLE_ENV`]).
    ///
    /// This single variant collapses the plan's demanded 4 sibling
    /// variants (`grammar_abi_probe_result` /
    /// `grammar_tier_b_loaded` / `svelte_runes_parse_health` /
    /// `tree_sitter_skipped_large_file`) per Pattern-2 precedent
    /// (DRIFT-6 in workflow/reasoning/E-42-decisions.md). The
    /// existing `LanguageGrammarStatus` (B-30 / SCHEMA 36) covers the
    /// one-shot load-summary surface and is NOT duplicated.
    ///
    /// Producers:
    ///   * `knowledge::tree_sitter_health::abi_probe` -- emits
    ///     `abi_probe_mismatch` rows when a registered grammar reports
    ///     an ABI outside `[14, 15]`.
    ///   * `knowledge::tree_sitter_health::svelte_health` -- emits
    ///     `svelte_runes_parse_health` rows carrying the runes-detected
    ///     flag + ERROR / MISSING node counts + raw-text-fallback
    ///     verdict.
    ///   * `knowledge::tree_sitter_health::registration` -- emits
    ///     `tier_b_grammar_loaded` rows when the
    ///     `experimental_grammars` feature is enabled and a Tier-B
    ///     grammar successfully registers.
    ///   * `knowledge::tree_sitter_health::large_file` -- emits
    ///     `tree_sitter_skipped_large_file` rows when source byte length
    ///     exceeds `TREE_SITTER_LARGE_FILE_BYTES` (2 MiB ceiling per
    ///     tree-sitter/tree-sitter#1277).
    ///
    /// Composite-language note: the existing
    /// `knowledge::dkg::VueGrammar::vue_extract_script_recursive` and
    /// the `SvelteGrammar` reuse of the TypeScript walker are the
    /// canonical reconciliation for `<script>` / `<style>` injection;
    /// no parallel injections.scm pathway is introduced (DRIFT-10).
    ///
    /// Citations (verified May 2026 via WebSearch / source ground):
    ///   * github.com/tree-sitter/tree-sitter/releases (0.26.8 line)
    ///   * github.com/tree-sitter/tree-sitter/issues/5013 (0.26 bug-fix-only)
    ///   * github.com/zed-industries/zed/issues/24632 (ABI 15 metadata)
    ///   * github.com/tree-sitter/tree-sitter/issues/1277 (large-file)
    ///   * github.com/tree-sitter/tree-sitter/issues/3625 (multi-level injection unsolved)
    ///   * github.com/tree-sitter-grammars/tree-sitter-svelte (Svelte 5 status)
    ///
    /// See workflow/reasoning/E-42-decisions.md.
    TreeSitterGrammarEvent,
    /// E-46 (SCHEMA_VERSION 184) -- planning-decomposition substrate.
    /// SINGLE-VARIANT Pattern-2 (36th application). Carries
    /// [`PlanningDecompositionEventPayload`] with the `kind` discriminator
    /// (8 arms: `reflection_appended` / `low_entropy_decomposition` /
    /// `goal_drift_canary_below_threshold` / `bubble_up_replan_phase` /
    /// `bubble_up_ledger_reset` / `agentless_shortcut_dispatched` /
    /// `process_reward_score_emitted` / `skill_prior_retrieved`) +
    /// free-form `details_json`. Wire-dark unless
    /// `ELAI_PLANNING_DECOMPOSITION_ENABLE=1` (see
    /// [`ELAI_PLANNING_DECOMPOSITION_ENABLE_ENV`]).
    ///
    /// This single variant collapses the plan's demanded 8 sibling
    /// variants per Pattern-2 precedent (E-20 9-arm + E-21 4-arm +
    /// E-36 5-arm + E-42 4-arm + E-4 13-arm). The shipped
    /// `TaskAdvanced` / `AgentlessShortcutTaken` / `GoalDriftFired`
    /// variants from the earlier E-46 module-existence cut (s-62376,
    /// 2026-05-20) are PRESERVED — this variant is purely additive for
    /// the substrate fire-points absent from that cut.
    ///
    /// Producers: `orchestrator/src/planning_decomposition/producers.rs`
    /// (8 emit fns, all async + env-gated). Consumer surfaces are
    /// orchestrator-internal only; no Tauri command path planned this
    /// cycle per the cross-check exploration "NO dashboard scope"
    /// decision.
    ///
    /// Paper grounding: arXiv:2411.04468 (Magentic-One),
    /// arXiv:2407.01489 (AGENTLESS), arXiv:2303.11366 (Reflexion),
    /// arXiv:2505.02709 (Goal-Drift), arXiv:2503.13657 (multi-agent
    /// failure taxonomy), arXiv:2310.04406 (LATS),
    /// arXiv:2305.16291 (Voyager), arXiv:2311.05772 (ADaPT),
    /// arXiv:2305.04091 (Plan-and-Solve), arXiv:2305.18323 (ReWOO).
    /// All IDs verified May 2026 via WebSearch.
    ///
    /// See workflow/reasoning/E-46-decisions.md.
    PlanningDecompositionEvent,
    /// E-48 (SCHEMA_VERSION 185) -- web-research orchestrator hardening
    /// substrate. SINGLE-VARIANT Pattern-2 (37th application). Carries
    /// [`WebResearchEventPayload`] with a 10-arm
    /// [`WebResearchEventKind`] discriminator + free-form
    /// `details_json`. Wire-dark unless
    /// `ELAI_WEB_RESEARCH_ORCHESTRATOR_ENABLE=1` (see
    /// [`ELAI_WEB_RESEARCH_ORCHESTRATOR_ENABLE_ENV`]).
    ///
    /// Kind arms:
    ///   * `research_plan_emitted` -- editable plan stage produced a
    ///     [`WebResearchPlan`] artifact before any SEARCH call fired.
    ///   * `tiered_search_dispatched` -- one tiered-pool dispatch over
    ///     T1/T2/T3 backends; the `status` field in `details_json`
    ///     discriminates `ok` / `degraded_to_t2_only` /
    ///     `tier_disabled_quota_exceeded`.
    ///   * `extraction_failed_all_extractors` -- both crawl4ai primary
    ///     and Jina fallback failed for a single source URL.
    ///   * `cross_encoder_rerank_complete` -- one rerank pass over the
    ///     top-50 quality-gated sources collapsed to top-5; carries a
    ///     rank-shift histogram in `details_json`.
    ///   * `speculative_branch_pruned` -- FlashResearch speculative
    ///     branch dropped on low intermediate-evidence delta with
    ///     `low_gain_streak > 2`.
    ///   * `research_preemptive_stop` -- conformal triple-threshold
    ///     lower bound fired (preemptive-unsolvable verdict).
    ///   * `contradiction_node_materialized` -- one Disagree pair
    ///     surfaced as a prune-protected Contradiction in the research
    ///     tree; renderer obligation lives downstream.
    ///   * `citation_verification_outcome` -- one citation passed
    ///     through the 4-stage verifier; `outcome` in `details_json`
    ///     discriminates `pass` / `dead_url` / `invalid_arxiv` /
    ///     `invalid_doi` / `unfaithful`.
    ///   * `citation_attribution_violation_blocked` -- lead/orchestrator
    ///     attempted to cite outside the SEARCH-stage producer set;
    ///     refusal recorded.
    ///   * `recency_decay_applied` -- temporal-marker regex hit and the
    ///     SourceQualityTable score got multiplied by the recency decay.
    ///
    /// Single Pattern-2 variant collapses the plan's 17 named
    /// fire-points into 10 Kind arms by folding outcome discriminators
    /// into `details_json` (per E-50 11-arm + E-4 13-arm precedent).
    ///
    /// Paper grounding: arXiv:2510.05145 FlashResearch (Task 1, 5),
    /// arXiv:2602.06718 GhostCite (Task 8), arXiv:2602.03814 Conformal
    /// Thinking (Task 6), arXiv:2504.00180 + arXiv:2511.06668 +
    /// arXiv:2510.03418 contradiction-RAG canon (Task 7),
    /// arXiv:2509.19376 + arXiv:2601.17824 freshness (Task 10),
    /// arXiv:2509.23519 ReliabilityRAG + arXiv:2410.22954 (Task 2),
    /// simonwillison.net/2025/Jun/14 Anthropic multi-agent attribution
    /// lock (Task 9). See workflow/reasoning/E-48-decisions.md.
    WebResearchEvent,
    /// E-44 (SCHEMA_VERSION 186) -- durability observability substrate.
    /// SINGLE-VARIANT Pattern-2 (38th application). Carries
    /// [`DurabilityEventPayload`] with a 10-arm
    /// [`DurabilityEventKind`] discriminator + free-form
    /// `details_json`. Wire-dark unless
    /// `ELAI_DURABILITY_OBSERVABILITY_ENABLE=1` (see
    /// [`ELAI_DURABILITY_OBSERVABILITY_ENABLE_ENV`]).
    ///
    /// Kind arms (1:1 with the plan's promised 10 sibling variants):
    ///   * `tool_idempotency_hit` -- IdempotentDispatcher served a
    ///     cached result for a deterministic call-shape replay.
    ///   * `tool_idempotency_replayed_irreversible` -- a cached
    ///     side-effect-bearing call was elided to preserve at-most-once
    ///     semantics.
    ///   * `compensation_logged` -- a SAGA compensation action was
    ///     appended to the compensation_actions log (producer pending;
    ///     wire-tag reserved).
    ///   * `compensation_replayed` -- a recovered session re-executed
    ///     a logged compensation (producer pending; wire-tag reserved).
    ///   * `compensation_failed` -- a compensation action errored
    ///     (producer pending; wire-tag reserved).
    ///   * `session_dag_continue_as_new` -- the per-session ceiling
    ///     was hit and `continue_as_new_if_needed` rolled the session
    ///     over to a new root.
    ///   * `checkpoint_lazy_migrated` -- the read-side migrator chain
    ///     transformed a checkpoint from `from_version` -> CURRENT.
    ///   * `compact_lock_contended` -- the PreCompact hook and 85%
    ///     compact path raced; the second acquirer waited or aborted.
    ///   * `agent_reliability_metrics` -- RDC/VAF/GDS/MOP rollup
    ///     emission (producer pending; wire-tag reserved).
    ///   * `tool_error_classified` -- a tool error was bucketed into
    ///     the 3-bucket recovery taxonomy (producer pending; wire-tag
    ///     reserved).
    ///
    /// Single Pattern-2 variant collapses the plan's 10 named sibling
    /// variants (ToolIdempotencyHit / ToolIdempotencyReplayedIrreversible
    /// / CompensationLogged / CompensationReplayed / CompensationFailed
    /// / SessionDagContinueAsNew / CheckpointLazyMigrated /
    /// CompactLockContended / AgentReliabilityMetrics /
    /// ToolErrorClassified) into 1 variant + 10 Kind arms (saves 9
    /// SCHEMA slots + 9 ALL_VARIANTS rows + 9 alphabetical inserts,
    /// per E-46 8-arm + E-48 10-arm precedent).
    ///
    /// Wire-tag reservation: 5 of 10 Kinds (compensation_*,
    /// agent_reliability_metrics, tool_error_classified) cover
    /// task primaries that are NOT-SHIPPED at the producer site as of
    /// SCHEMA_VERSION 186. The Kind arms exist so when the producers
    /// land they emit immediately without a follow-on SCHEMA bump.
    ///
    /// See workflow/sessions/s-night-2026-05-30/impl-tasks/TASK-E44-013.md.
    DurabilityEvent,
    /// E-31 (SCHEMA_VERSION 187) -- head_dim invariant verification substrate.
    /// SINGLE-VARIANT Pattern-2 (39th application). Carries
    /// [`HeadDimInvariantVerifiedPayload`] with a 2-arm
    /// [`HeadDimInvariantVerifiedKind`] discriminator + free-form
    /// `details_json`. Wire-dark unless
    /// `ELAI_HEAD_DIM_INVARIANT_SSB_ENABLE=1` (see
    /// [`ELAI_HEAD_DIM_INVARIANT_SSB_ENABLE_ENV`]).
    ///
    /// Kind arms:
    ///   * `verified` -- assert_head_dim_invariant succeeded at sidecar
    ///     boot: observed head_dim matched expected (128 for the
    ///     Qwen2.5-Coder family). `details_json` carries
    ///     `{model_id, head_dim_observed, head_dim_expected, ok: true}`.
    ///   * `refused` -- post-boot health-check re-evaluation detected a
    ///     mismatch (the boot-time refuse path raises SidecarBootError
    ///     before any SSB row can be written, so the `refused` arm
    ///     covers in-band re-check from a hot config swap). `details_json`
    ///     carries `{model_id, head_dim_observed, head_dim_expected, ok: false}`.
    ///
    /// Producer surface: python/elai_inference/server.py
    /// `assert_head_dim_invariant`. Mirrors the head_dim_invariant
    /// contract test (crates/contracts/tests/head_dim_invariant_contract.rs)
    /// at the runtime layer. Closes TASK-E31-001 (head_dim invariant +
    /// SSB row gap surfaced by the s-night-2026-05-30 cross-check).
    ///
    /// See workflow/sessions/s-night-2026-05-30/impl-tasks/TASK-E31-001.md.
    HeadDimInvariantVerified,
    /// E-3 (SCHEMA_VERSION 188) -- role-shown / model-hidden default
    /// telemetry substrate. SINGLE-VARIANT Pattern-2 (40th application).
    /// No Kind discriminator (scope-lean MVP slice of TASK-E03-001).
    /// Carries [`RoleShownModelHiddenPayload`]. Wire-dark unless
    /// `ELAI_ROLE_VISIBILITY_TELEMETRY_ENABLE=1` (see
    /// [`ELAI_ROLE_VISIBILITY_TELEMETRY_ENABLE_ENV`]).
    ///
    /// Fire-point: SettingsWindow toggle "show model details" change
    /// (default OFF -- ROLE visible, MODEL hidden); the Tauri
    /// `record_model_visibility` command emits one row each time the user
    /// flips the switch. Reverses the cross-check finding "SettingsWindow
    /// explicitly exposes MODEL names per role" by making MODEL hidden
    /// the default and observable so the audit pass can verify reach.
    ///
    /// Producer surface:
    /// crates/dashboard-v2/src-tauri/src/commands.rs::record_model_visibility.
    ///
    /// DEFERRED (full E-3 plan, see TASK-E03-001):
    ///   * role_prompt.rs (per-role prompt registry + VoiceAnchor)
    ///   * voice.toml (E-5 voice_style facts wiring)
    ///   * RuntimeAdapter::warm + predict_next_role
    ///   * 5 other SSB variants (UnifiedPreambleApplied /
    ///     RuntimePrewarmStarted / RuntimePrewarmComplete /
    ///     HandoffPayloadCompressed / PersonaVectorApplied /
    ///     StyleNormalizerApplied)
    ///   * persona-vector sidecar hook
    ///
    /// See workflow/sessions/s-night-2026-05-30/impl-tasks/TASK-E03-001.md.
    RoleShownModelHidden,
    /// E-49 (SCHEMA_VERSION 189) -- coding-agent metrics named-token
    /// substrate. SINGLE-VARIANT Pattern-2 (41st application). Carries
    /// [`CodingMetricNamedTokenPayload`] with a 3-arm
    /// [`CodingMetricNamedTokenKind`] discriminator + free-form
    /// `details_json`. Wire-dark unless
    /// `ELAI_CODING_METRIC_NAMED_TOKEN_ENABLE=1` (see
    /// [`ELAI_CODING_METRIC_NAMED_TOKEN_ENABLE_ENV`]).
    ///
    /// Kind arms:
    ///   * `stable_pass_at_1` -- n=5 seed stability metric wrapping
    ///     `TestOutcomeReport` with identical-output check + std-dev.
    ///   * `harness_attribution_tuple` -- (model_id, harness_version,
    ///     harness_commit_sha) recorded per session start.
    ///   * `stop_verified` -- CI lint result refusing legacy HumanEval /
    ///     MBPP / SWE-bench Verified mention without `STOP-Verified:`
    ///     guard.
    ///
    /// Closes the E-49 SPEC-DRIFT against the 3 spec-named tokens
    /// surfaced by cross-check E-16_e47_e48_e49.md (zero hits for the
    /// named strings across `.elai_cc/` before this bump).
    ///
    /// See workflow/sessions/s-night-2026-05-30/impl-tasks/TASK-E49-005.md.
    CodingMetricNamedToken,
    /// E-3 sub-phase 4A (SCHEMA_VERSION 190) -- runtime handoff event
    /// substrate. SINGLE-VARIANT Pattern-2 (42nd application). Carries
    /// [`RuntimeHandoffEventPayload`] with a 6-arm
    /// [`RuntimeHandoffEventKind`] discriminator + free-form
    /// `details_json`. Wire-dark unless
    /// `ELAI_E3_RUNTIME_HANDOFF_ENABLE=1` (see
    /// [`ELAI_E3_RUNTIME_HANDOFF_ENABLE_ENV`]).
    ///
    /// Kind arms:
    ///   * `unified_preamble_applied` -- emitted by
    ///     `RuntimeOrchestrator::try_complete` when `build_role_prompt`
    ///     wraps the system message with a [`crate::role_prompt::VoiceAnchor`]
    ///     preamble. `details_json` carries
    ///     `{base_prompt_len, anchor_blake3, preamble_len}`.
    ///   * `runtime_prewarm_started` -- emitted by `predict_next_role`
    ///     before kicking off a 1-token CUDA-graph warmup on the
    ///     predicted next backend. `details_json` carries
    ///     `{predicted_role, backend, vram_tier}`. (sub-phase 4B)
    ///   * `runtime_prewarm_complete` -- emitted after the warmup
    ///     returns. `details_json` carries
    ///     `{backend, duration_ms, ok, cuda_graph_captured}`. (sub-phase 4B)
    ///   * `handoff_payload_compressed` -- emitted by
    ///     `compress_handoff_payload`. `details_json` carries
    ///     `{original_tokens, compressed_tokens, ratio}`. (sub-phase 4B)
    ///   * `persona_vector_applied` -- emitted by the sidecar persona-vector
    ///     hook. `details_json` carries
    ///     `{model_id, vector_blake3, strength}`. (sub-phase 4B)
    ///   * `style_normalizer_applied` -- emitted by the live_cli
    ///     style-normalizer post-pass. `details_json` carries
    ///     `{source_role, input_blake3, output_blake3, tells_stripped}`.
    ///     (sub-phase 4B)
    ///
    /// Pattern-2 collapse rationale: the original E-3 plan body called
    /// out 6 sibling variants. The exploration report
    /// (workflow/sessions/s-21636/exploration/tranche4-E3-dashboard.md)
    /// collapsed them into one envelope per the 40+ documented
    /// applications, saving 5 SCHEMA slots and matching the existing
    /// CodingMetricNamedToken / DurabilityEvent precedent.
    ///
    /// See workflow/sessions/s-night-2026-05-30/impl-tasks/TASK-E03-001.md
    /// + docs/thefinalglance/E-plan/E-3_model_handoff_ux.md.
    E3RuntimeHandoffEvent,
    /// E-31 sub-phase 6A (SCHEMA_VERSION 191) -- cloud-route decision
    /// event substrate. SINGLE-VARIANT Pattern-2 (43rd application).
    /// Carries [`RouteDecisionEventPayload`] with a 3-arm
    /// [`RouteDecisionEventKind`] discriminator + free-form
    /// `details_json`. Wire-dark unless
    /// `ELAI_ROUTE_DECISION_SSB_ENABLE=1` (see
    /// [`ELAI_ROUTE_DECISION_SSB_ENABLE_ENV`]).
    ///
    /// Kind arms:
    ///   * `cloud_route_chosen` -- the runtime routing policy
    ///     selected a cloud arm (CloudSonnet / CloudOpus /
    ///     CloudDeepSeekFlash). `details_json` carries the
    ///     `RoutingReason` + the selected `RoutingChoice` string.
    ///   * `tier_model_swapped` -- the runtime routing policy
    ///     forced a swap from the per-role configured model to a
    ///     different model arm (e.g. VRAM headroom too low ->
    ///     CloudDeepSeekFlash). Producer surface reserved for
    ///     future TierModelSwap producer (no live emit in 6A).
    ///   * `local_route_kept` -- the runtime routing policy
    ///     evaluated the per-step context and kept LocalSidecar.
    ///     Useful for sanity-check telemetry on the local-first
    ///     path.
    ///
    /// Producer surface: crates/orchestrator/src/cloud_mode.rs
    /// `CloudModeRouter::route` invokes
    /// `providers::runtime::RuntimeRoutingPolicy::evaluate` and
    /// emits via tracing log behind the env gate (Ssb::write_payload
    /// injection deferred to sub-phase 6B alongside the dashboard tile
    /// / TierModelSwap producer).
    ///
    /// See workflow/sessions/s-21636/exploration/tranche6-E31-wiring.md.
    RouteDecisionEvent,
    /// E-47 (SCHEMA_VERSION 192) -- LoRA fine-tuning ship-gate
    /// substrate. SINGLE-VARIANT Pattern-2 (44th application).
    /// Carries [`LoraShipGateEventPayload`] with a 3-arm
    /// [`LoraShipGateEventPhase`] discriminator + free-form
    /// `details_json`. Wire-dark unless
    /// `ELAI_LORA_SHIP_GATE_ENABLE=1` (see
    /// [`ELAI_LORA_SHIP_GATE_ENABLE_ENV`]).
    ///
    /// Phase arms:
    ///   * `refused` -- the triple-AND gate (enabled +
    ///     confirm_user_understanding + deferral token, OR
    ///     calibration regression) refused the adapter load.
    ///     `details_json` carries the refusal reason.
    ///   * `calibration_missing` -- gate is enabled and configured
    ///     to ship but no `LoraCalibrationReport` was supplied.
    ///   * `calibration_passed` -- gate is wire-dark OR the
    ///     calibration report passes the no-regression threshold.
    ///
    /// Producer surface:
    /// `crates/orchestrator/src/lora_ship_gate.rs::notify_lora_ship_gate`
    /// + `elai lora-gate-check` CLI subcommand.
    ///
    /// See workflow/reasoning/E-47-decisions.md.
    LoraShipGateEvent,
    /// E-40 Tranche B-2 (SCHEMA_VERSION 193) -- SymbolGraph augmentation
    /// substrate. SINGLE-VARIANT Pattern-2 (45th application). Carries
    /// [`SymbolGraphEventPayload`] with a 4-arm [`SymbolGraphEventKind`]
    /// discriminator + free-form `details_json`. Wire-dark unless
    /// `ELAI_SYMBOL_GRAPH_5TH_RRF_ENABLE=1` (or other per-event gates
    /// upstream).
    ///
    /// Kind arms:
    ///   * `symbol_dsl_executed` -- `exec_dsl` returned (any DSL parse +
    ///     execute cycle).
    ///   * `dataflow_impact_computed` -- `DataflowAnalyzer::impact_analysis`
    ///     returned (any BFS impact request, including the 5th-list RRF
    ///     retriever path).
    ///   * `agent_impact_claim_refuted` -- the Refine+Filter stage
    ///     refused the agent's claim because an unexpected file was
    ///     impacted (auditor-loop hook).
    ///   * `scope_graph_resolved` -- reserved for Tranche B-3 (the
    ///     scope_graph implementation lands there; reserving the slot
    ///     now avoids a 2nd SCHEMA bump on B-3 landing).
    ///
    /// Producer surfaces:
    /// - `crates/orchestrator/src/symbol_graph/mod.rs::notify_symbol_graph_event`
    /// - `crates/orchestrator/src/symbol_graph/dsl.rs::exec_dsl`
    /// - `crates/orchestrator/src/symbol_graph/dataflow.rs::impact_analysis`
    /// - The new `symbol_dsl_query` / `symbol_dataflow_impact` tool
    ///   definitions in `production_agent_rpc.rs::build_tool_catalog`.
    ///
    /// See `workflow/reasoning/E-40-tranche-B2-decisions.md`.
    SymbolGraphEvent,
    /// Tranche B-6 quad C-42 SHIPPABLE-NOW (SCHEMA_VERSION 194) -- zero-trust
    /// MCP gateway runtime emit. SINGLE-VARIANT Pattern-2. Carries
    /// [`McpAuthzEventPayload`] with a 3-arm [`McpAuthzEventKind`]
    /// discriminator (AuthzCheckPassed / AuthzCheckFailed / CorpusReplayed) +
    /// free-form `details_json`. Wire-dark unless
    /// `ELAI_C42_MCP_AUTHZ_EMIT_ENABLE=1`. Producer:
    /// `crates/plugins/src/authz/compose.rs::compose_with_tool_gate` wrapper
    /// at the caller side; `check_authz` stays pure (deliberate
    /// defense-in-depth boundary -- see checker.rs:1-9).
    ///
    /// Proc-macro `#[authz(states="...", actors="...")]` declarative
    /// surface is DEFERRED to C-42b (first proc-macro crate in workspace
    /// adds friction without changing the threat model -- runtime guard
    /// closes the SHIPPED-WIRED gap per MASTER-INDEX line 194).
    ///
    /// See `workflow/reasoning/B6-quad-tranche-decisions.md`.
    McpAuthzEvent,
    /// Tranche B-6 quad C-47 (SCHEMA_VERSION 194) -- case-based calibration
    /// substrate. SINGLE-VARIANT Pattern-2 with 6-arm
    /// [`ToolCalibrationEventKind`] discriminator (HintEmitted /
    /// DriftDetected / CallLogged / CaseAdded / CaseSelected /
    /// CalibrationApplied) + free-form `details_json`. Wire-dark unless
    /// `ELAI_C47_CASE_BASE_ENABLE=1`. The 3 pre-existing helpers
    /// (`ToolCalibrationHintEmitted`, `ToolCalibrationDriftDetected`,
    /// `ToolCallLogged`) stay alive for one release window as deprecation
    /// aliases; full migration in C-47b.
    ///
    /// See `workflow/reasoning/B6-quad-tranche-decisions.md`.
    ToolCalibrationEvent,
    /// Tranche B-6 quad B-39 (SCHEMA_VERSION 194) -- 85% chat-reflect
    /// budget soft-warn. SINGLE-ADD (NOT Pattern-2; reflect_* family stays
    /// alphabetical-contiguous as 5th sibling). Carries
    /// [`ReflectBudgetSoftWarnPayload`]. Wire-dark unless
    /// `ELAI_B39_SOFTWARN_ENABLE=1`. Producer:
    /// `crates/reasoning/src/chat_reflect.rs` ~L454 (BEFORE the existing
    /// hard-cap log-only block).
    ///
    /// See `workflow/reasoning/B6-quad-tranche-decisions.md`.
    ReflectBudgetSoftWarn,
    /// Tranche B-6 quad C-50 (SCHEMA_VERSION 194) -- ContractRetry*
    /// substrate. SINGLE-VARIANT Pattern-2 with 3-arm
    /// [`ContractCodingEventKind`] discriminator (RetryAttempted /
    /// RetryResolved / RetryAbandoned) + free-form `details_json`.
    /// Wire-dark unless `ELAI_C50_CONTRACT_RETRY_EMIT_ENABLE=1`. Producer:
    /// `crates/orchestrator/src/task_runner.rs::fix_and_rerun_until_green`
    /// Healed (L1055) + EscalatedToHuman (L1109) + Failed (L1057)
    /// branches.
    ///
    /// Pre-existing `ContractViolationDetected` stays alive (NOT collapsed
    /// in this tranche -- separate landing). Caller wiring at
    /// `drive_task` is a separate concern guarded by `is_web_project` --
    /// not modified in this tranche.
    ///
    /// See `workflow/reasoning/B6-quad-tranche-decisions.md`.
    ContractCodingEvent,
    /// F-1 (SCHEMA_VERSION 196) -- vibe-coding security gate blocked event.
    /// Pattern-2 single variant with 4-arm [`VibeGateBlockedKind`]
    /// discriminator (CodeVuln / Secret / DepHallucinated / Skill). The
    /// sql_str arms are:
    ///   * `CodeVuln`        -> `vibesec_code_vuln_blocked`
    ///   * `Secret`          -> `vibesec_secret_blocked`
    ///   * `DepHallucinated` -> `vibesec_dep_hallucinated_blocked` (RESERVED)
    ///   * `Skill`           -> `vibesec_skill_blocked` (RESERVED)
    ///
    /// DepHallucinated and Skill are parse/sql arms only this session;
    /// no producer fires them. Wire-dark unless `ELAI_VIBE_GATE_ENABLE=1`.
    /// Carries [`VibeGateBlockedPayload`].
    VibeGateBlocked(VibeGateBlockedKind),
    /// F-2a (SCHEMA_VERSION 196) -- security rule blocked flat beacon.
    /// Emitted by the rule engine each time a configured security rule
    /// rejects a candidate edit. No Kind discriminator (scope-lean MVP).
    /// Wire-dark unless `ELAI_SECURITY_RULE_BLOCKED_ENABLE=1`.
    /// Carries [`SecurityRuleBlockedPayload`].
    SecurityRuleBlocked,
    /// F-4 / Slice B-1 (SCHEMA_VERSION 197) -- test integrity verification
    /// event. Pattern-2 single variant with 2-arm
    /// [`TestIntegrityEventKind`] discriminator (Verified /
    /// FabricationBlocked). The sql_str arms are:
    ///   * `Verified`           -> `test_integrity_verified`
    ///   * `FabricationBlocked` -> `anti_fabrication_block`
    ///
    /// Carries [`TestIntegrityEventPayload`] which includes `stdout_hash`
    /// for the fabrication-detection pipeline.
    TestIntegrityEvent(TestIntegrityEventKind),
    /// F-3p1 / Slice C1 (SCHEMA_VERSION 198) -- capability-flow tool-filter
    /// commitment + untrusted-variable quarantine event.
    /// Pattern-2 single variant with 2-arm [`ToolFilterEventKind`]
    /// discriminator (Committed / UntrustedQuarantined). The sql_str arms are:
    ///   * `Committed`            -> `tool_filter_committed`
    ///   * `UntrustedQuarantined` -> `untrusted_var_quarantined`
    ///
    /// `CapabilityFlowBlocked` is RESERVED (Tier-3) and is NOT an arm of
    /// this variant -- it has no in-plan producer (W-8).
    ///
    /// Carries [`ToolFilterPayload`].
    ToolFilterEvent(ToolFilterEventKind),
    /// F-9 / Slice D1 (SCHEMA_VERSION 199) -- write-time anti-slopsquat gate
    /// blocked a dependency import.
    ///
    /// Emitted by `crate::vibesec::dep_writetime_gate::DepWriteTimeGate::analyze`
    /// when a dependency name resolves to `NotFound` or `TooNew` in the
    /// registry.  Registry-unreachable cases do NOT emit this variant (fail-open
    /// per W-10 network-axis policy).
    ///
    /// `verdict_kind` is a String (not a Pattern-2 kind enum variant) carrying
    /// one of the [`WriteTimeSlopVerdictKind`] snake_case labels:
    ///   * `"not_found"` -- package absent from registry.
    ///   * `"too_new"` -- package exists but its latest publish is older than
    ///     the per-ecosystem staleness threshold (stale/abandoned fork). Now
    ///     PRODUCED: the SubprocessRegistryChecker maps a `result=="fail"` +
    ///     "stale"/">…d" reason to TooNew (BLOCK-1 reconcile, F-Unit-D).
    ///   * `"registry_unreachable"` -- RESERVED; must never be emitted (gate
    ///     is fail-open on unreachable).
    ///
    /// Carries [`WriteTimeSlopsquatBlockedPayload`].
    WriteTimeSlopsquatBlocked,
    /// F-10 / Slice A1 (SCHEMA_VERSION 200) -- security-class regression
    /// blocked. Emitted when a refine / auto-advance / self-mod iteration
    /// would introduce a CWE class that was absent from the prior iteration
    /// (a security-monotonicity violation). Flat beacon (no Kind discriminator
    /// on the action-type enum); the 3 production legs are distinguished by
    /// the `loop_kind` field of [`SecurityClassRegressionBlockedPayload`].
    /// Default-ON; suppressed only by `ELAI_SECURITY_MONOTONICITY_DISABLE`.
    /// Carries [`SecurityClassRegressionBlockedPayload`].
    SecurityClassRegressionBlocked,
    /// F-2b / Slice A1 (SCHEMA_VERSION 201) -- insecure-rate keystone bench
    /// beacon. Emitted once when the static-gate insecure-rate benchmark runs
    /// over a SeCodePLT-structured CWE corpus, recording the corpus
    /// ground-truth insecure-rate (before), the gate-approved-subset
    /// insecure-rate (after = static-gate false-negative residual), and the
    /// derived gate recall. Flat beacon (no Kind discriminator). This is a
    /// bench-run telemetry beacon, NOT a wire-dark gate: it has no
    /// `ELAI_*_ENABLE` flag and is emitted only inside the benchmark itself.
    /// Carries [`InsecureRateMeasuredPayload`].
    InsecureRateMeasured,
    /// I-9 (SCHEMA_VERSION 263) -- error-amplification (alpha_t) telemetry
    /// beacon. Emitted once when `elai bench error-amplification` runs the
    /// frozen pipeline fixture, recording the measured amplification factor
    /// (downstream propagated fraction) + recovered fraction + the active-
    /// annealing temperature. Flat beacon (no Kind discriminator); a bench-run
    /// telemetry beacon, NOT a wire-dark gate (no `ELAI_*_ENABLE` flag).
    /// Carries [`ErrorAmplificationMeasuredPayload`].
    ErrorAmplificationMeasured,
    /// F-6 / Slice A1 (SCHEMA_VERSION 202) -- learned-skill library event.
    /// Pattern-2 single variant with 3-arm [`SkillLibraryEventKind`]
    /// discriminator (Acquired / TrustPromoted / DemotedStale). The sql_str
    /// arms are:
    ///   * `Acquired`      -> `skill_acquired`
    ///   * `TrustPromoted` -> `skill_trust_promoted`
    ///   * `DemotedStale`  -> `skill_demoted_stale`
    ///
    /// Carries [`SkillLibraryEventPayload`]. No `ELAI_*_ENABLE` default-off
    /// flag (W-2); the autonomy barrier governs dispatch, not a wire-dark
    /// gate.
    SkillLibraryEvent(SkillLibraryEventKind),
    /// F-5 / Slice A1 (SCHEMA_VERSION 203) -- cross-subsystem
    /// staleness-quarantine broadcast. SINGLE FLAT beacon (no Kind
    /// discriminator), mirroring the B-30 `LanguageGrammarStatus` /
    /// E-5 `MemoryFactSuperseded` flat-beacon shape. Emitted at the
    /// orchestrator/CLI decay-tick boundary when a memory-decay tick
    /// quarantines a fact whose staleness ratio crosses the decay threshold;
    /// the broadcast cascades to DKG `concept_stale=2` + a non-destructive
    /// `staleness_quarantined` marker edge + Cosmos artifact flagging via a
    /// bounded `QuarantineBroadcastQueue` drained by the CLI. The
    /// `dropped_count` field surfaces DROP-OLDEST overflow (W-2 no silent
    /// drop). Carries [`StalenessQuarantineBroadcastPayload`].
    StalenessQuarantineBroadcast,
    /// F-8 / Slice A1 (SCHEMA_VERSION 204) -- SCF symbolic semantic conflict
    /// detected. SINGLE FLAT beacon (no Pattern-2 Kind), mirroring the F-5
    /// `StalenessQuarantineBroadcast` / F-10 `SecurityClassRegressionBlocked`
    /// flat-beacon shape. Emitted at the `elai self-mod apply` CLI entrypoint
    /// when the LLM-free symbolic detector flags a self-mod config variant that
    /// contradicts an active locked-key invariant (`self_mod_safety::
    /// LOCKED_KEYS`) BEFORE archive admission. POLICY is WARN-then-escalate: the
    /// variant is held PENDING, never hard-fail-closed BLOCK. Carries
    /// [`SymbolicConflictPayload`].
    SymbolicConflictDetected,
    /// F-7 / Slice A1 (SCHEMA_VERSION 205): a crystallized pipeline template's
    /// Laplace-smoothed outcome utility was computed and used to gate template
    /// reuse at the `try_execute` seam. SINGLE FLAT beacon (no Pattern-2 Kind),
    /// mirroring the F-8 `SymbolicConflictDetected` / F-5
    /// `StalenessQuarantineBroadcast` flat-beacon shape. The utility is
    /// `laplace_utility(successes, total)` over the PERSISTED `CompiledPipeline`
    /// `first_try_success_rate` + `use_count`; a 1/1 template scores 2/3 (not
    /// 1.0) so low-sample templates stop being over-trusted. Carries
    /// [`TemplateUtilityRankedPayload`].
    TemplateUtilityRanked,
    /// F-13 / Slice A1 (SCHEMA_VERSION 206): the stale-version `ssb.db` (or the
    /// precautionary `session_dag.db`) was cold-archived to
    /// `workflow/sessions/{id}/archive/ssb-v{N}.sqlite` BEFORE the destructive
    /// `run_migration` DROP, gated behind the wire-dark
    /// `ELAI_SSB_ARCHIVE_ON_BUMP_ENABLE` kill-switch (forensic opt-in;
    /// default-OFF by design — F-11 in-file W-2 deviation precedent). SINGLE
    /// FLAT beacon (no Pattern-2 Kind); `store_kind` is a payload field
    /// (`"ssb"` | `"session_dag"`), NOT a discriminator. Emitted via a direct
    /// synchronous INSERT inside `run_migration` (the async writer does not
    /// exist yet at migration time). Carries [`SessionArchivedPayload`].
    SessionArchived,
    /// F-11 (SCHEMA_VERSION 207) — config-DGM admit-loop beacon. Pattern-2
    /// single-variant WITH a 3-arm [`DarwinianAdmitKind`] discriminator on the
    /// payload (`Admitted` / `RejectedByGuard` / `LoopClosed`). Emitted once per
    /// candidate routed through the four-guard chain (tripwire blake3 fingerprint
    /// -> trap-pattern scorer on the LIVE candidate -> `is_security_sensitive`
    /// keyword filter -> `two_gate_promote`). `LoopClosed` proves
    /// `config_archive::select_parent` ran AFTER `insert_variant`. Carries
    /// [`DarwinianAdmitPayload`]. SEPARATE action_type from the pre-existing E-7
    /// `DarwinianGenerationCompleted` (do NOT reuse). Wire-dark unless
    /// `ELAI_DARWINIAN_LOOP_ENABLE=1` AND the orchestrator is compiled with
    /// `--features darwinian-evolve`. Paper grounding:
    ///   - arXiv:2505.22954 (DGM) — propose->evaluate->insert->select archive
    ///     loop; deceptive-dip caveat (a single closed generation is NOT a
    ///     capability claim).
    ///   - arXiv:2510.04399 (Two-Gate) — bounded-capacity margin/capacity gate;
    ///     config-only mutant keeps the loop inside the PAC-safe regime.
    /// See workflow/reasoning/F-11-decisions.md.
    DarwinianAdmit,
    // G-plan (SCHEMA_VERSION 208-224): security + verifiability substrate.
    /// G-4: MCP schema rug-pull blocked at http_list_tools gate.
    McpRugPullBlocked,
    /// E-37 U3: MCP-scan classifier blocked at http_list_tools.
    McpScanClassifierBlocked,
    /// G-1: data-plane taint flow blocked at destructive sink.
    TaintFlow,
    /// G-3: blast-radius gate block (FS escape or egress denied).
    BlastRadiusBlocked,
    /// G-7: lethal trifecta quarantine at tool-call chokepoint.
    TrifectaQuarantined,
    /// G-2: reproducible-run artifact sealed at run end.
    RunArtifactEmitted,
    /// G-5: reviewer-cockpit DSR score emitted.
    ReviewSurfaceScored,
    /// G-8: signed in-toto provenance attestation emitted.
    ProvenanceAttestationEmitted,
    /// G-6: SAFEDELTA gate-ON vs gate-OFF measured.
    SafetyDeltaMeasured,
    /// G-10: exogenous behavioral regression suspected.
    ModelDriftSuspected,
    /// G-9: write-time secret/PII redaction advisory row.
    SecretRedacted,
    /// G-7b: per-change conformal confidence decision.
    ClaimConfidenceScored,
    /// G-11: steering-vector refresh triggered on deprecated pattern.
    SteeringVectorRefreshTriggered,
    /// H-10 canvas: canvas op applied at dispatcher.
    CanvasOpApplied,
    /// H-10 canvas: canvas view event (spawn/ui/checkpoint).
    CanvasViewEvent,
    /// H-10 canvas: canvas activity event (reading/editing).
    CanvasActivityEvent,
    /// H-1: Anthropic batch API lifecycle row.
    BatchLifecycle,
    // H-plan verifiability substrate (SCHEMA_VERSION 225+).
    /// H-17: Sharpe-ratio deflation gate verdict.
    DeflationGateEvaluated,
    /// H-18: data-leakage blocked at split boundary.
    LeakageBlocked,
    /// H-19: lookahead differential (future-leak) verdict.
    LookaheadDifferential,
    /// H-20: trading-strategy null-hypothesis test verdict.
    TradingNullTest,
    /// H-21: rerun hash equality verdict.
    RerunHashEquality,
    /// H-22: sim vs real parity verdict.
    SimRealParity,
    /// H-23: result manifest verified verdict.
    ResultManifestVerified,
    /// H-24: differential divergence between primary and shadow.
    DifferentialDivergence,
    /// H-25: gate self-validation mutation score verdict.
    GateSelfValidation,
    /// H-26: reward tampering detection verdict.
    RewardTampering,
    /// H-27: holdout budget exhaustion or running verdict.
    HoldoutBudgetEvent,
    /// H-28: drift interlock running/warning/halted verdict.
    DriftInterlock,
    /// H-17: strategy trial count persisted.
    TrialRegistered,
    /// C-36 extension: MuCoCo inconsistency detected.
    MucocoInconsistencyDetected,
    /// C-36 extension: MuCoCo operator inapplicable skip reason.
    MucocoSkipReason,
    /// H-30: user-decision captured or superseded in memory.
    UserDecisionSuperseded,
    /// H-17: staleness-eval benchmark result measured.
    StalenessEvalMeasured,
    /// D-27.1: discovery-candidate template ratified by a human reviewer.
    /// Emitted by `ratify_discovery` Tauri command. Carries template_id,
    /// reviewer_id (non-empty), and ts_ms.
    DiscoveryRatified,
    /// D-27.1: discovery-candidate template rejected by a human reviewer.
    /// Emitted by `reject_discovery` Tauri command. Carries template_id,
    /// reviewer_id (non-empty), reason, and ts_ms.
    DiscoveryRejected,
    /// H-32 (SCHEMA_VERSION 210): Tier-A code-grounded fact citation failed to
    /// re-resolve against the current DKG before injection. The fact is
    /// HARD-EXCLUDED. Carries a [`FactValidationFailedPayload`] (fact_id,
    /// citation, reason: `"stale"` | `"unknown"`, evidence_blake3).
    FactValidationFailed,
    /// H-33 (SCHEMA_VERSION 211): memory governance event (forget/pin/unpin).
    /// Emitted by `elai memory forget|pin|unpin` and the dashboard
    /// memory_forget/memory_pin Tauri commands.
    /// 6-arm kind discriminator carried in [`MemoryGovernanceEventPayload`].
    MemoryGovernanceEvent,
    /// H-34 (SCHEMA_VERSION 212): deferred MCP schema resolve on the live STDIO
    /// lane. Emitted by `mcp_glue::RuntimeMcpState::call_tool` on cache-miss when
    /// `ELAI_MCP_DEFER_SCHEMA_ENABLE=1`. Carries
    /// [`McpToolDeferredPayload`] (server_name, tool_qualified_name, resolve_ns,
    /// cache_hit, injected_floor_size).
    McpToolDeferred,
    /// H-34 (SCHEMA_VERSION 213): mcp-schema-budget bench result beacon.
    /// Emitted by `elai bench mcp-schema-budget`. Carries
    /// [`McpSchemaBudgetMeasuredPayload`].
    McpSchemaBudgetMeasured,
    /// H-35 (SCHEMA_VERSION 214): session-rewind restore beacon.
    /// Emitted once per successful `elai rewind` / `rewind_session` call.
    /// Payload: [`SessionRewoundPayload`].
    SessionRewound,
    /// H-36 (SCHEMA_VERSION 215): dev-event notification beacon (task complete /
    /// needs input). Emitted by `notify_dispatch::NotificationDispatcher::notify`
    /// from the task_runner post-PhaseComplete path (Completion kind) and the
    /// AutonomyDecision::RequestHuman path (NeedsInput kind).
    /// Payload: [`AgentNotificationEmittedPayload`].
    AgentNotificationEmitted,
    /// H-37 (SCHEMA_VERSION 216): plan-approval gate decision at
    /// PLANNING|FEASIBILITY_PROBE -> TESTING_FIRST. Gated by
    /// `ELAI_PLAN_APPROVAL_GATE_ENABLE=1`. Pattern-2 single-variant with
    /// 4-arm [`PlanDecisionKind`] discriminator.
    /// Payload: [`PlanDecisionPayload`].
    PlanDecisionEvent,
    /// H-38 (SCHEMA_VERSION 217): cost-confirmation gate outcome on task/pod
    /// stop. Emitted by `dispatch_stop` on every confirm-required or
    /// rejected-held path; on the pass-no-friction path only when
    /// `ELAI_STOP_COST_CONFIRM_ENABLE=1`. Pattern-2 single-variant with
    /// 3-arm [`StopOutcomeKind`] discriminator.
    /// Payload: [`StopConfirmationPayload`].
    StopConfirmationEvent,
    /// H-39 (SCHEMA_VERSION 218): abstention-aware scoring bench beacon.
    /// Emitted once per bench run after computing the [`AbstentionAggregate`]
    /// over the run's outcome triples. Closes the self-improvement loop
    /// misalignment: darwin_admit_loop's reward was abstention-blind (GAP-1).
    /// Flat beacon (Pattern-2 single-variant). Default-ON, kill-switch
    /// `ELAI_ABSTENTION_SCORE_DISABLE`.
    /// Payload: [`AbstentionScoredPayload`].
    AbstentionScored,
    /// H-41 (SCHEMA_VERSION 219): Factual/Computational claim fell to
    /// non-Constructive because no constructive (Deterministic) verifier
    /// matched. Off-agent backlog artifact (R4-compliant): written by the
    /// route_claim_with_gap wrapper at the live routing call site, not by
    /// the agent under evaluation. Carries [`CheckerGapDetectedPayload`]
    /// with {claim_id, claim_type, complexity_taxon, fell_to_mode, ts_ms}.
    /// Wire-dark under `ELAI_CHECKER_GAP_ENABLE` kill-switch (R9).
    /// Flat beacon (Pattern-2 single-variant). TOTAL_VARIANTS 529 -> 530.
    CheckerGapDetected,
    /// H-42 (SCHEMA_VERSION 220): serial-depth-scaled GRC budget decision beacon.
    /// Emitted at both dispatch sites (agent_pool::spawn + bsp/engine.rs timeout)
    /// when `scale_grc_budget` raises a sub-task's budget above the base per-role
    /// config. Carries [`ReasoningBudgetScaledPayload`] with {kind, role,
    /// base_calls, serial_depth, scaled_calls, ceiling, seed}.
    /// Grounding: Merrill & Sabharwal 2310.07923 (single pass ~TC0); Li et al.
    /// 2402.12875 (T steps simulate size-T circuit); Yehudai et al. 2503.01544
    /// (~O(n) CoT tokens for compositional reasoning).
    /// Default-ON, kill-switch `ELAI_REASONING_BUDGET_SCALE_DISABLE`.
    /// Pattern-2 single-variant 2-arm ReasoningBudgetScaledKind: DepthBound /
    /// AccessBound. TOTAL_VARIANTS 530 -> 531.
    ReasoningBudgetScaled,
    /// H-43 (SCHEMA_VERSION 221): execution-grounded DGM fitness beacon.
    /// Replaces the blake3-of-bytes `derived_alpha` placeholder (GAP-1) with a
    /// measured pass-rate signal derived from executing the candidate against a
    /// FROZEN generated-test set (CodeRM arXiv:2501.01054 + CodeT arXiv:2207.10397).
    /// Emitted by `darwin_admit_loop::run_one_generation` per scored candidate
    /// under `cfg(feature="darwinian-evolve")` + `ELAI_DARWINIAN_LOOP_ENABLE=1`.
    /// Off-agent artifact (R4): written at the scoring call site, NOT by the
    /// candidate being evaluated. Missing = FAIL (R1).
    /// Carries [`FitnessEvaluatedPayload`] with {kind, pass_rate, dual_agreement,
    /// tests_total, fitness, seed, test_set_hash, candidate_blake3}.
    /// Pattern-2 single-variant 3-arm [`FitnessEvaluatedKind`]:
    /// ExecutionPassRate | DualAgreement | FitnessRegression.
    /// Wire-dark under existing `darwinian-evolve` feature + `ELAI_DARWINIAN_LOOP_ENABLE`.
    /// TOTAL_VARIANTS 531 -> 532.
    FitnessEvaluated,
    /// H-44 (SCHEMA_VERSION 222): behavioral cheat-detector beacon.
    /// Replaces the keyword-grep placeholder (GAP-2, `trap_tasks.rs:score_candidate_for_traps`)
    /// with three behavioral detection legs: ImpossibleBench conflict canary
    /// (arXiv:2510.20270), Isomorphic-Perturbation re-verification (arXiv:2604.15149),
    /// CodeHalu execution-classification (arXiv:2405.00253, composes H-43 harness).
    /// Also carries TrapRecallReport with measured recall over a frozen TRACE-style
    /// corpus (arXiv:2601.20103). Keyword-grep is demoted to a cheap advisory
    /// `KeywordFallback` first-pass only (never the sole blocker — VD-2).
    /// Off-agent artifact (R4): emitted from the guard chain call site.
    /// Carries [`TrapBehaviorDetectedPayload`] with {kind, recall, fp_rate, n,
    /// corpus_hash, candidate_blake3, seed}.
    /// Pattern-2 single-variant 5-arm [`TrapBehaviorDetectedKind`]:
    /// ImpossiblePassExploit | IsomorphMismatch | ExecutionHallucination |
    /// KeywordFallback | ClosureEscapeHatch (H-54 formal-lane deterministic leg).
    /// Wire-dark under existing `darwinian-evolve` feature + `ELAI_DARWINIAN_LOOP_ENABLE`.
    /// TOTAL_VARIANTS 532 -> 533 (H-44 original count; adding a Kind arm does NOT
    /// add a new SsbActionType variant — no TOTAL_VARIANTS bump at H-54).
    TrapBehaviorDetected,
    /// H-45 (SCHEMA_VERSION 223): worst-case/variance-penalized aggregate over
    /// lineage-diverse promotion verdicts.
    /// Replaces the candidate's own mean fitness as the `two_gate_promote` margin
    /// (Coste 2310.02743 WCO+UWO; Eisenstein 2312.09244 lineage-diversity;
    /// Co-rewarding 2508.00410 disagreement block; PRMBench 2501.03124 tau calibration).
    /// Off-agent artifact (R4): emitted from `darwin_admit_loop::run_one_generation`
    /// at the promotion gate call site for every scored candidate.
    /// Carries [`PromotionScoreAggregatedPayload`] with {kind, worst_case, variance,
    /// mean, disagreement, tau, calibrated_at, seed}.
    /// Pattern-2 single-variant 3-arm [`PromotionScoreAggregatedKind`]:
    /// WorstCaseScored | VariancePenalized | LineageDiverseDisagree.
    /// Wire-dark under existing `darwinian-evolve` feature + `ELAI_DARWINIAN_LOOP_ENABLE`.
    /// TOTAL_VARIANTS 533 -> 534.
    PromotionScoreAggregated,
    /// H-46 (SCHEMA_VERSION 224): HGM clade-metaproductivity parent selection beacon.
    /// Emitted from `config_archive::select_parent_with_cmp` on the REAL loop
    /// selection step (`darwin_admit_loop::run_one_generation -> select_parent ->
    /// CMP key`) under `cfg(feature="darwinian-evolve")` + `ELAI_DARWINIAN_LOOP_ENABLE=1`.
    /// CMP aggregates execution-grounded descendant fitness deltas (H-43; VD-1:
    /// NOT byte-hash). `ImmediateScoreFallback` fires for `clade_size == 0` leaves
    /// (no-lineage cold-start guard). Missing row = FAIL (R1).
    /// Carries [`CladeSelectedPayload`] with {kind, node_id, cmp, clade_size,
    /// immediate_score, key_used, seed}.
    /// Pattern-2 single-variant 2-arm [`CladeSelectedKind`]:
    /// CladeMetaproductivity | ImmediateScoreFallback.
    /// Wire-dark under existing `darwinian-evolve` feature + `ELAI_DARWINIAN_LOOP_ENABLE`.
    /// TOTAL_VARIANTS 534 -> 535.
    CladeSelected,
    /// H-47 (SCHEMA_VERSION 225): DGM eval firewall beacon.
    /// Composes three SHIPPED mechanisms (GD-1 P0 — none re-implemented):
    ///   - D-6 date-gate (LiveCodeBench 2403.07974): rejects held-out items whose
    ///     `ingested_ts_ms` precedes the last ConfigVariant.created_at update.
    ///   - E-54 template-regen (GSM-Symbolic 2410.05229): regenerates a fresh instance
    ///     for templated tasks so the loop never scores the exact instance twice.
    ///   - H-19 Thresholdout budget (Dwork 1506.02629): routes every score query
    ///     through the DP budget; refuses after `Exhausted` (budget leg REUSES
    ///     `HoldoutBudgetEvent` — GD-1: no second variant).
    /// Emitted ONCE per generation (aggregate counts) from
    /// `darwin_admit_loop::run_one_generation` after the per-item firewall loop.
    /// Off-agent artifact (R4): emitted at the loop eval step, NOT by the candidate.
    /// Missing row = FAIL (R1).
    /// Carries [`EvalFirewallPayload`] with {kind, items_scored, items_rejected,
    /// regen_count, budget_remaining, distinguishability, seed, generation_id}.
    /// Pattern-2 single-variant 3-arm [`EvalFirewallEventKind`]:
    /// PreUpdateItemRejected | TemplateRegenerated | IndistinguishabilityProbe.
    /// Wire-dark under existing `darwinian-evolve` feature + `ELAI_DARWINIAN_LOOP_ENABLE`.
    /// TOTAL_VARIANTS 535 -> 536.
    EvalFirewallEvent,
    /// H-49 (SCHEMA_VERSION 226): reversible LLM edits beacon.
    ///
    /// Emitted by the default-ON CAS time-travel capture hook in `task_runner.rs`
    /// on every file pre-edit capture, and by `elai undo`/`redo`/`jump` CLI ops.
    ///
    /// Off-agent artifact (R4): emitted from the orchestrator capture hook or the
    /// CLI driver — NOT by the agent performing the edit. Missing row = FAIL (R1).
    ///
    /// Carries [`EditHistoryEventPayload`] with
    /// `{kind, node_id, parent_id, paths_n, pre_hash, post_hash, grain}`.
    ///
    /// Pattern-2 single-variant 6-arm [`EditHistoryEventKind`]:
    /// SnapshotCaptured | Deduped | Redo | Branched | Jumped | TooLargeSkipped.
    ///
    /// Default-ON kill-switch: `ELAI_TIMETRAVEL_DISABLE` (no wire-dark-allowlist
    /// row required — this is a kill-switch, not a default-OFF gate; R9).
    /// TOTAL_VARIANTS 536 -> 537.
    EditHistoryEvent,
    /// H-51 (SCHEMA_VERSION 227): cross-session sleep-time handoff digest beacon.
    ///
    /// Emitted by [`handoff_digest::author_topic_digest`] at session end (Drop hook
    /// on [`LiveCli`]) and by the fresh-REPL auto-detect + tiered-load path.
    ///
    /// Off-agent artifact (R4): emitted from the CLI session lifecycle, NOT by the
    /// agent generating the digest. Missing row = FAIL (R1) on the wiring smoke test.
    ///
    /// Carries [`HandoffLifecyclePayload`] with
    /// `{kind, topic_slug, digest_path, loaded_tokens, session_id}`.
    /// Pattern-2 single-variant 4-arm [`HandoffLifecycleEventKind`]:
    /// DigestWritten | ResumeOffered | ResumeConfirmed | TieredLoad.
    ///
    /// Kill-switch: `ELAI_HANDOFF_AUTORESUME_DISABLE` (auto-detect ON path)
    /// registered in `wire-dark-allowlist.toml` per R9.
    /// TOTAL_VARIANTS 537 -> 538.
    HandoffLifecycleEvent,
    /// H-52 (SCHEMA_VERSION 228): deterministic snippet->hunk finding-anchoring beacon.
    ///
    /// Emitted by `review_cmd::open_score_and_emit` (the `elai review --cockpit`
    /// production entrypoint) ONCE per finding anchoring attempt. Off-agent artifact
    /// (R4): emitted from the CLI driver after `anchor_finding` resolves, NOT by the
    /// agent that generated the finding. Missing row = FAIL (R1/R2).
    ///
    /// Carries [`FindingAnchoredPayload`] with
    /// `{finding_id, snippet_blake3, resolved_hunk_id?, new_line_start?, kind, candidate_match_n}`.
    ///
    /// Pattern-2 single-variant 5-arm [`FindingAnchoredKind`]:
    /// Anchored | FileFallback | Ambiguous | Unanchorable | Relocated.
    ///
    /// Default-ON kill-switch: `ELAI_REVIEW_ANCHOR_DISABLE` (no wire-dark-allowlist
    /// row required — this is a DISABLE kill-switch, not a default-OFF enable gate; R9).
    /// TOTAL_VARIANTS 538 -> 539.
    FindingAnchored,
    /// H-53 (SCHEMA_VERSION 229): citation pre-binding + constrained synthesis beacon.
    ///
    /// Emitted by `prebind_cmd::run_demo_prebind` (the `elai prebind --demo-prebind`
    /// REAL CLI entrypoint) and by the /research|/ask answer path AFTER
    /// `citation_prebind::freeze_anchors` runs. Off-agent artifact (R4): emitted from the
    /// CLI driver, not by the model generating the answer. Missing row = FAIL (R1/R2).
    ///
    /// Carries [`CitationAnchorsBoundPayload`] with
    /// `{kind, anchor_count, seed, blake3_set_hash_hex}`.
    ///
    /// Pattern-2 single-variant 2-arm [`CitationAnchorsBoundKind`]:
    /// - `Bound`: the evidence set was frozen and injected into context; all `[N]` markers
    ///   must be drawn from the pre-bound id space.
    /// - `Blocked`: ELAI_CITATION_PREBIND_DISABLE is set (legacy path active); this arm fires
    ///   in the broken-arm gate smoke test to prove the guard was bypassed.
    ///
    /// Default-ON kill-switch: `ELAI_CITATION_PREBIND_DISABLE` (no wire-dark-allowlist
    /// row required — this is a DISABLE kill-switch, not a default-OFF enable gate; R9).
    /// TOTAL_VARIANTS 539 -> 540.
    CitationAnchorsBound,
    /// I-39 (SCHEMA_VERSION 230): per-plan state derived from three deterministic
    /// sources: wiring_contract_check.py --static verdict, SSB journal presence
    /// of observable_effect rows, and s-plan-audit-2026-06-04 FINISH fragments.
    ///
    /// Emitted by [`plan_status_cmd::run_plan_status`] (REAL entrypoint:
    /// `elai plan status [--letter <L>] [--json]`).
    ///
    /// Flat beacon. Carries [`PlanStateDerivedPayload`] with fields
    /// `{letter, session_id, derived_state, evidence_anchor, map_date}`.
    ///
    /// Default-ON; no kill-switch required (pure read path).
    /// TOTAL_VARIANTS 540 -> 541.
    PlanStateDerived,
    /// I-45 (SCHEMA_VERSION 231): objective-hash sealed-receipt beacon.
    ///
    /// Pattern-2 single-variant 3-arm [`ObjectiveSealedKind`]:
    /// - `Sealed`: emitted from `rra::build_rra` when the objective_hash is
    ///   stamped into the [`RunArtifact`]. Carries the 16-hex prefix of the
    ///   blake3(canonical_objective_bytes) hash.
    /// - `MutationDetected`: emitted from `rra::verify_artifact_with_objective`
    ///   when the re-derived hash mismatches the sealed hash (the task was
    ///   re-defined after PASS). Hard-FAIL verdict.
    /// - `ReceiptStale`: emitted when the attestation predicate's objective_hash
    ///   mismatches the presented task. Hard-FAIL verdict.
    ///
    /// Default-ON; kill-switch `ELAI_OBJECTIVE_SEAL_DISABLE`. Fail-soft: a missing
    /// writable journal silently skips the emit; the verify verdict is NEVER
    /// masked (R2). TOTAL_VARIANTS 541 -> 542.
    ObjectiveSealed(ObjectiveSealedKind),
    /// I-34 (SCHEMA_VERSION 234): CRANE hybrid grammar-pin gate beacon. Pattern-2
    /// single-variant 3-arm [`GrammarGateEventKind`]:
    /// - `CranePassed`: constrained decode completed, gate eligibility check passed.
    /// - `PinViolation`: grammar file's blake3 digest mismatches the pinned manifest value.
    /// - `PostParseRejected`: post-parse re-validation detected silent non-compliance.
    ///
    /// Emitted by `orchestrator::grammar_gate::emit_grammar_gate_event`.
    /// Default-ON; no kill-switch (mandatory per IC-18 + landmine #7/#8).
    /// TOTAL_VARIANTS 542 -> 543.
    GrammarGateEvent(GrammarGateEventKind),
    /// I-38 (SCHEMA_VERSION 235): QDELTA defective-diff rate gate-ON vs gate-OFF.
    /// Flat beacon (no Kind discriminator). Mirrors SafetyDeltaMeasured shape.
    /// Emitted by `cli::bench_qdelta::run_inner` after `floor_check` passes.
    /// TOTAL_VARIANTS 543 -> 544.
    QdeltaMeasured,
    /// C-37: snowballing harness summary measurement flat beacon.
    SnowballingMeasured,
    /// I-16: KV-cache tuning dry-run result.
    KvCacheTuned,
    /// I-43 (SCHEMA_VERSION 236): reproducibility-claim scoper + kill-gate.
    /// Pattern-2 single-variant 3-arm [`ClaimScopeMeasuredKind`]:
    /// - `PilotDelta`: reduced-N gate-ON vs gate-OFF pilot signal + trap-detector.
    /// - `CrossLaneDivergence`: local-vs-cloud action-stream divergence % on security paths.
    /// - `ClaimScoped`: consolidated go/no-go verdict with honest claim wording.
    ///
    /// Emitted by `cli::bench_claim_scope::run_inner`. TOTAL_VARIANTS 544 -> 545.
    ClaimScopeMeasured(ClaimScopeMeasuredKind),
    /// I-10 (SCHEMA_VERSION 237): prefix invariance + TTL tiering + cache-hit hard guard.
    /// Pattern-2 single-variant 3-arm [`CacheEconomicsEventKind`]:
    /// - `TtlPlanned`: planner selected a TTL tier for a cloud call.
    /// - `HitRateGuardTripped`: unexplained miss-rate exceeded threshold (>10%).
    /// - `PrefixDiffDumped`: guard detected a prefix mutation; blake3 byte-range dumped.
    ///
    /// Emitted by `orchestrator::cache_economics::emit_cache_economics_event`.
    /// Default-ON kill-switch `ELAI_CACHE_ECONOMICS_DISABLE=1`. Cloud-lane only.
    /// Composes on H-1/H-2 shipped substrate. TOTAL_VARIANTS 545 -> 546.
    CacheEconomicsEvent(CacheEconomicsEventKind),
    /// I-8 (SCHEMA_VERSION 238): aletheia voluntary abort + dead-end hash pruning + halting bound.
    /// Pattern-2 single-variant 3-arm [`TrajectoryAbortedKind`]:
    /// - `Voluntary`: model called `acknowledge_infeasible` explicitly.
    /// - `DeadEndHash`: pre-dispatch fingerprint match pruned an isomorphic re-attempt (0 tokens).
    /// - `HaltingBound`: empirical check-runner watchdog fired (I-17 seam; Wave-1 stub registered).
    ///
    /// Emitted by `production_agent_rpc.rs` `TOOL_ACKNOWLEDGE_INFEASIBLE` arm (Voluntary),
    /// `dead_end_cache.rs` check path (DeadEndHash), and I-17 watchdog hook (HaltingBound).
    /// Composes H-39 AbstentionScored (GD-1) and H-49 rollback (Wave-2 wiring).
    /// TOTAL_VARIANTS 546 -> 547.
    TrajectoryAborted(TrajectoryAbortedKind),
    /// I-1 (SCHEMA_VERSION 239): directive supervisor enforcement-plane outcome.
    /// Pattern-2 single-variant 3-arm [`DirectiveExecutedKind`]:
    /// - `Executed`: gates passed and the side effect ran cleanly.
    /// - `Discarded`: a gate blocked the directive; the side effect never ran.
    /// - `CompensationFired`: the side effect crashed and E-44 saga compensation
    ///   restored state.
    ///
    /// Emitted by `directive::InProcessSupervisor::execute_directive`. The
    /// generator emits pure `Directive` structs only; the Supervisor is the
    /// sole side-effect executor (CANT-WRECK). Composes E-30/F-1/G-3 gates.
    /// TOTAL_VARIANTS 547 -> 548.
    DirectiveExecuted(DirectiveExecutedKind),
    /// I-33 (SCHEMA_VERSION 240): mechanical decision governance primitive fired.
    /// Pattern-2 single-variant 3-arm [`GovernancePrimitiveFiredKind`]:
    /// - `CeflExternalized`: CEFL candidate set injected; model-invented options
    ///   outside the schema are structurally unparseable (selection-by-negation
    ///   foreclosed at the type level).
    /// - `I6qRejected`: rationale failed the I6Q length+diversity gate; decision
    ///   refused; downstream routes to D-16 `RefusalDecision::Abstain`.
    /// - `CommitRevealSealed`: blake3 commit row written before reveal; ordering
    ///   integrity verified (commit_row_id < reveal_row_id = PASS).
    /// Default-ON; kill-switch `ELAI_GOVERNANCE_DISABLE`.
    /// Composes E-25 prompt_decision + D-16 refusal path + C-26 DecodingMode.
    /// TOTAL_VARIANTS 548 -> 549.
    GovernancePrimitiveFired(GovernancePrimitiveFiredKind),
    /// I-32 (SCHEMA_VERSION 241): structural role-scope contract applied to a
    /// system-prompt build. Pattern-2 single-variant 3-arm
    /// [`RoleContractAppliedKind`]:
    /// - `PersonaStripped`: narrative "You are..." persona removed; schema-first
    ///   role contract text injected. Emitted for factual/code roles (Executor,
    ///   Researcher, ContextCompiler, Designer, MemoryWriter).
    /// - `PersonaRetained`: alignment-dependent role (Auditor*, IdeaCritic,
    ///   InjectionClassifier) -- contract store has no entry; original framing
    ///   left intact.
    /// - `AbDelta`: emitted by `--role-ab-probe` CLI harness; carries before/after
    ///   fixture pass-rate fields and delta for the evidence gate (ID-5:
    ///   new_rate >= old_rate required for production flip).
    /// Default-ON; grounded per IC-17 (MMLU -3.6..-5.3pp / MT-Bench coding -0.65
    /// for narrative persona in factual/code roles).
    /// Composes IC-17 persona-split + ID-5 evidence gate + E-6 IsoBudgetMeasured.
    /// TOTAL_VARIANTS 549 -> 550.
    RoleContractApplied(RoleContractAppliedKind),
    /// I-35 (SCHEMA_VERSION 242): temporal tool-sequence enforcement verdict.
    /// Pattern-2 single-variant 3-arm [`ToolSequenceVerdictKind`]:
    /// - `Allowed`: all enabled rules that triggered on this tool class were satisfied.
    /// - `OrderViolation`: a WarnOnly rule's predicate was not satisfied.
    /// - `HaltEnforced`: a Halt-enforced rule's predicate was not satisfied; the call
    ///   is refused fail-closed (consistent with the existing Discarded path).
    /// Emitted by `directive::InProcessSupervisor::run_sequence_gate` (5th supervisor
    /// gate arm, additive -- does NOT replace E-30/G-3/F-1/I-32).
    /// Predicates evaluate the ordered tool-call history buffer (CDA-immune: history
    /// rows are built from real dispatch events, not from token content).
    /// R8-frozen: rule set hash in `.claude/gate-manifest.txt`.
    /// TOTAL_VARIANTS 550 -> 551.
    ToolSequenceVerdict(ToolSequenceVerdictKind),
    /// I-41 (SCHEMA_VERSION 243): spec-time critique lenses advisory beacon.
    /// Pattern-2 single-variant 3-arm [`CritiqueLensFindingTag`]:
    /// - `AdvisoryFinding`: a lens produced a finding with a concrete anchor.
    /// - `DroppedNoAnchor`: a raw finding was dropped because `location` was empty.
    /// - `LensRun`: telemetry beacon emitted once per lens pass (finding or not).
    /// `advisory=true` on all arms; ZERO path to any accept/block verdict.
    /// Default-OFF (ELAI_CRITIQUE_LENSES_ENABLE=1); kill-switch registered R9.
    /// TOTAL_VARIANTS 551 -> 552.
    CritiqueLensFinding(CritiqueLensFindingTag),
    /// I-46 (SCHEMA_VERSION 244): bash interceptor redirect / fixup / prefix outcome.
    /// Pattern-2 single-variant 3-arm [`BashRedirectedKind`]:
    /// - `Redirected`: command matched a redirect rule; no process spawn.
    /// - `FixedUp`: trailing output-pipeline noise stripped; rewritten command returned.
    /// - `PrefixBlocked`: command starts with an allowed prefix but body contains a banned
    ///   shell token (control char, unquoted expansion, newline, backslash).
    /// Default-ON; no kill-switch (interceptor is not a default-OFF experiment).
    /// Emitted by `dispatch::in_process::InProcessDispatch::maybe_emit_bash_redirected`.
    /// TOTAL_VARIANTS 552 -> 553.
    BashRedirected(BashRedirectedKind),
    /// I-47 (SCHEMA_VERSION 245): verifier-input hardening + dispatch rule injection beacon.
    /// Pattern-2 single-variant 3-arm [`PromptHardeningAppliedKind`]:
    /// - `InputsTagged`: auditor/critic dispatch -- untrusted worker output wrapped in
    ///   `<ELAI_DATA>` boundary tags with hardening frame prepended above data blocks.
    /// - `RulesInjected`: worker dispatch -- frozen per-role Karpathy-distillate rule block
    ///   appended to CONSTRAINTS section (HD-2 byte-stable per role, cache-prefix safe).
    /// - `SelfAttestationStripped`: auditor/critic dispatch -- a self-attestation phrase
    ///   detected in worker output; labelled as claim-not-evidence inside sub-tag.
    /// Default-ON; unconditional on auditor/critic/worker dispatches (no kill-switch, R3).
    /// Emitted by `tools::execute_agent_with_spawn` after hardening applied at seam.
    /// TOTAL_VARIANTS 553 -> 554.
    PromptHardeningApplied(PromptHardeningAppliedKind),
    /// I-2 (SCHEMA_VERSION 246): CoW workspace isolation barrier outcome.
    /// Pattern-2 single-variant 4-arm [`BranchWorkspaceEventKind`]:
    /// - `Spawned`: a private branch workspace root was created (warm git worktree).
    /// - `Merged`: the deterministic SSB-seq-ordered barrier merged all roots into trunk.
    /// - `EscapeBlocked`: a branch tried to write outside its root; blocked pre-FS-mutation.
    /// - `ConflictEscalated`: two branches wrote the same path with differing bytes; merge halted.
    ///
    /// Emitted by `sandbox::branch_workspace::WorktreePoolImpl` (spawn/escape_check) and the BSP
    /// barrier site (`bsp/engine.rs` `barrier_merge`). Composes C-11/E-15/I-1/H-49.
    /// TOTAL_VARIANTS 554 -> 555.
    BranchWorkspaceEvent(BranchWorkspaceEventKind),
    /// I-5 (SCHEMA_VERSION 247): per-branch context rebuild outcome.
    /// Pattern-2 single-variant 3-arm [`ContextPrunedKind`]:
    /// - `SiblingExcluded`: sibling branch payloads excluded from fresh branch context (ID-1).
    /// - `TracebackContrasted`: failed-attempt traceback replaced by <=30-token constraint phrase.
    /// - `VividnessProbe`: sampled A/B outcome delta (pristine minus accumulated); recomputable
    ///   under ELAI_SEED (GD-2). Gated by `ELAI_REASONING_QUALITY_PROBE_ENABLE`.
    ///
    /// Emitted by `pristine_path_filter::PristinePathFilter::build_branch_context`.
    /// Wire-dark until I-4 SearchLadder ships (Wave 4). Composes H-6/H-50/E-19.
    /// TOTAL_VARIANTS 555 -> 556.
    ContextPruned(ContextPrunedKind),
    /// I-11 (SCHEMA_VERSION 248): typed versioned condensate receipt for handoffs.
    /// Pattern-2 single-variant 3-arm [`BeliefStateEventKind`]:
    /// - `Condensed`: emitted at every `adaptive_reduce` condensation (ID-3 JOURNALED-MIDDLEWARE receipt).
    /// - `Handoff`: emitted when BeliefState payload dispatched to cloud escalation arm.
    /// - `HonestyDelta`: emitted by HonestyJob::run after >=10 escalations.
    ///
    /// Default-ON; no kill-switch (ID-3 JOURNALED-MIDDLEWARE is mandatory).
    /// Composes I-7/H-51/H-6/CL-1..CL-4. TOTAL_VARIANTS 556 -> 557.
    BeliefStateEvent(BeliefStateEventKind),
    /// I-13 (SCHEMA_VERSION 249): write-time memory guard outcome.
    /// Pattern-2 single-variant 3-arm [`MemoryGuardEventKind`]:
    /// - `PoisonBlocked`: Stage-2 aho-corasick scan refused a write (invisible-Unicode or
    ///   confusable-shell pattern); carries `pattern_class` tag.
    /// - `DualSerialized`: a ToolResult was serialized via `to_ui_dict` (full) vs
    ///   `to_llm_content` (condensed); carries `token_delta`.
    /// - `TraceQueryExpanded`: model called TraceQuery::expand for on-demand line-range
    ///   expansion; carries `byte_range`.
    ///
    /// Gated behind `ELAI_MEMORY_GUARD_ENABLE` (default-OFF, R9 experiment, GD-4).
    /// Composes H-31/H-32/E-37/IS-11. TOTAL_VARIANTS 557 -> 558.
    MemoryGuardEvent(MemoryGuardEventKind),
    /// I-14 (SCHEMA_VERSION 250): deterministic context-compression middleware receipt.
    /// Pattern-2 single-variant 3-arm [`ContextTransformKind`]:
    /// - `LspSlice`: a `rustc` panic payload was sliced to the located error neighbourhood.
    /// - `AstFold`: tree-sitter function-body elision (bodies elided, signatures kept).
    /// - `Tier2Compressed`: reserved discriminator for the deferred neural (LLMLingua-2-class) tier.
    ///
    /// Default-ON; no kill-switch (ID-3 JOURNALED-MIDDLEWARE is mandatory; an un-journaled
    /// transform is a done-gate FAIL). Composes E-52/knowledge-tree-sitter/IS-6/H-6.
    /// TOTAL_VARIANTS 558 -> 559.
    ContextTransformReceipt(ContextTransformKind),
    /// I-12 (SCHEMA_VERSION 251): evidence-gated playbook curator outcome beacon.
    /// Pattern-2 single-variant 4-arm [`PlaybookEventKind`]:
    /// - `Promoted`: rule advanced to ACTIVE (>=k SSB outcome links + metric gates + regression_count==0).
    /// - `Demoted`: ACTIVE rule lost ACTIVE status (stale or regressed).
    /// - `ValidationRun`: promotion-gate check without promotion (evidence_count < k, or regression > 0).
    /// - `Exported`: `export_verified` wrote a verified AGENTS.md / skills export block with outcome_row_ids.
    ///
    /// Default-ON; no kill-switch (ID-6 WEIGHTS-ARE-CONFIGS promotion ledger is mandatory).
    /// Evaluator lives OUTSIDE the write boundary (gate-freeze R8, ID-6).
    /// Production callers: (a) offline tick in LiveCli::build_local_knowledge_stack (Layer-1A),
    ///   (b) skills_cmd::run PlaybookExport arm (`elai skills-mgmt playbook export`).
    /// TOTAL_VARIANTS 559 -> 560.
    PlaybookEvent(PlaybookEventKind),
    /// I-17 (SCHEMA_VERSION 252): tri-layer evaluation-stack per-branch verdict.
    /// Pattern-2 single-variant 4-arm [`TriLayerVerdictKind`]:
    /// - `HardVeto`: L1 deterministic gate (compile/dep/vibe/checker) returned a veto;
    ///   score is a LITERAL 0.0 and L2/L3 never ran (l2_call_count == 0). ID-4 supreme.
    /// - `ScoredSurvivor`: branch cleared L1; L2 calibrated scorer assigned a score.
    /// - `DisagreementEscalated`: L2 verdict variance among survivors exceeded the
    ///   [DESIGN] threshold; the D-13 cross-family adversary was routed for a counter.
    /// - `FlakeQuarantined`: an oracle flipped across seeded reruns and was ledgered
    ///   out of voting (blake3(oracle-id + seed-range) key).
    ///
    /// Emitted by `verifier_stack::TriLayerStack::run_verdict` (one row per branch).
    /// Default-OFF gate `ELAI_TRI_LAYER_ENABLE` (wire-dark-allowlist; consumer is I-4
    /// Wave-4 SearchLadder). Composes D-3/D-11/F-1/F-9/D-13/I-23 (GD-1, never restates).
    /// TOTAL_VARIANTS 560 -> 561.
    TriLayerVerdict(TriLayerVerdictKind),
    /// I-18 (SCHEMA_VERSION 253): per-row rolling blake3 chain beacon.
    ///
    /// Pattern-2 single-variant 3-arm [`RowChainSealedKind`]:
    /// - `ChainAdvanced`: emitted from `run_subcommand::emit_run_artifact` after every
    ///   sealed RRA — the rolling chain_prev_hash advanced to the next position.
    /// - `AibomEmitted`: emitted from `run_subcommand::emit_attestation` when the
    ///   AI-BOM/CycloneDX-ML entry is sealed alongside the in-toto Statement.
    /// - `ChainBroken`: emitted from `rra::build_rra` when a Cargo.lock mutation is
    ///   detected mid-run at the DEPS layer. Halts the run with an error; the
    ///   off-agent beacon fires to signal the break (the observable_effect R2 beacon).
    ///
    /// NOTE: this is the SSB off-agent beacon kind. It is DISTINCT from
    /// `crate::verification_result::GateRejection::ChainBroken` (a Rust error type
    /// in the verify path). Do NOT conflate.
    ///
    /// Default-ON, no kill-switch. Composes G-2/G-8/I-45 (GD-1, never restates).
    /// TOTAL_VARIANTS 561 -> 562.
    RowChainSealed(RowChainSealedKind),
    /// I-19 (SCHEMA_VERSION 254): TDD enforcer pre-flight gate blocked event.
    ///
    /// Pattern-2 single-variant 3-arm [`TddPreflightBlockedKind`]:
    /// - `MissingTestDelta`: an Edit introduces a new pub symbol but the test-file set
    ///   contains no symbol whose name substring-matches the new symbol.
    /// - `SyntaxDryRunFailed`: staged content fails `cargo check --tests` / `pytest --collect-only`
    ///   (compile-only dry-run; no test execution — auditor runs tests).
    /// - `RefactorExempted`: the diff introduces NO new pub symbol; gate skipped.
    ///
    /// Wire-dark unless `ELAI_TDD_PREFLIGHT_ENABLE=1` (default-OFF, R9 experiment, GD-4).
    /// Gate stays default-OFF until FP rate bounded on >=3 real orchestrate runs.
    /// Composes F-1 VibeGate compound seam (GD-1, additive predicate not a fork).
    /// Emitted by `crate::tdd_preflight::emit_tdd_preflight_blocked_to_ssb`.
    /// TOTAL_VARIANTS 562 -> 563.
    TddPreflightBlocked(TddPreflightBlockedKind),
    /// I-20 (SCHEMA_VERSION 255): budgeted mutation scheduler per-cone outcome.
    ///
    /// Pattern-2 single-variant 4-arm [`MutationBudgetedRunKind`]:
    /// - `CiCleared`: the Wilson-CI lower bound cleared the (TOML-measured)
    ///   kill-rate threshold -- pass (early or full).
    /// - `CiFailed`: the Wilson-CI upper bound already failed the threshold
    ///   -- fail-early.
    /// - `SurvivorDirective`: a mutant survived; the row names operator + line
    ///   + surviving mutation (the structured counterexample fed to the generator).
    /// - `CacheHit`: an unchanged (fn-content, bound-test-set) blake3 key
    ///   returned from EquivalentMutantCache with ZERO mutant runs.
    ///
    /// Default-ON (the budget scheduler is the mutation hot-path optimization);
    /// kill-switch `ELAI_MUTATION_BUDGET_DISABLE=1`. The 5 CI params are
    /// recomputable under `ELAI_SEED` (GD-2). Emitted by
    /// `crate::deterministic_audit_gate::emit_mutation_budgeted_runs` from the
    /// writer-owning gate seam; cone-select wired into
    /// `crate::deterministic_auditor::DeterministicAuditor::check_mutation`.
    /// TOTAL_VARIANTS 563 -> 564.
    MutationBudgetedRun(MutationBudgetedRunKind),
    /// I-21 (SCHEMA_VERSION 256): sprint-contract + ralph-loop lifecycle event.
    ///
    /// Pattern-2 single-variant 5-arm [`SprintContractEventKind`]:
    /// - `Proposed`: the generator's structured Sprint Contract was received by
    ///   the isolated evaluator (zero shared generator context).
    /// - `Approved`: the evaluator approved the contract (acceptance criteria
    ///   are deterministically checkable; scope is bounded).
    /// - `Rejected`: the evaluator rejected the contract with a concrete,
    ///   structured objection (the vague-criterion path).
    /// - `RalphReinjected`: a TriLayerStack failure reinjected the failing
    ///   trace to the generator for one bounded retry (cap = 3 iterations).
    /// - `AdversaryEscalated`: the 3rd consecutive failure escalated to the
    ///   D-13 cross-family adversary, which produced a concrete counterexample.
    ///
    /// Wire-dark unless `ELAI_SPRINT_CONTRACT_ENABLE=1` (default-OFF, R9
    /// experiment). Composes H-37 gate / i-1 supervisor / i-17 TriLayerStack /
    /// D-13 adversary (GD-1 compose, never re-implement). Emitted by
    /// `crate::sprint_contract::emit_sprint_contract_event_to_ssb` from the
    /// `task_runner::dispatch_sprint_contract` post-task seam.
    /// TOTAL_VARIANTS 564 -> 565.
    SprintContractEvent(SprintContractEventKind),
    /// I-37 (SCHEMA_VERSION 257): per-EARS-requirement verified-status event.
    ///
    /// Pattern-2 single-variant 3-arm [`RequirementVerifiedKind`]:
    /// - `Satisfied`: mutation coverage >= floor AND all bound MRs held AND >=1 bound test.
    /// - `Unverified`: no bound tests -- shown in the cockpit, never dropped (R1).
    /// - `Regressed`: a bound test was deleted, an MR failed, or coverage fell below floor.
    ///
    /// Additive emit (always lands when an `EarsRequirementSet` is present);
    /// the BLOCKING `RequirementsGate` is wire-dark unless
    /// `ELAI_REQUIREMENTS_GATE_ENABLE=1` (default-OFF, R9 experiment).
    /// Emitted by `crate::requirements::emit_requirement_verified_to_ssb` from
    /// the `task_runner` PLANNING -> TESTING_FIRST gate seam.
    /// TOTAL_VARIANTS 565 -> 566.
    RequirementVerified(RequirementVerifiedKind),
    /// I-44 (SCHEMA_VERSION 258): zero-retention-after-verification purge receipt.
    ///
    /// Pattern-2 single-variant 3-arm [`RetentionPurgedKind`]:
    /// - `RawInputsPurged`: the active session's raw managed-transcript files
    ///   were deleted after the sealed verdict; the row names the purged classes.
    /// - `EvidenceRetained`: a blake3 over the redacted retained evidence set
    ///   (the survivable digest the verdict still verifies against).
    /// - `PurgeVerified`: the post-purge beacon — raw bytes gone, redacted
    ///   evidence digest survives.
    ///
    /// Destructive + opt-in: fires only on `elai run --zero-retention` AND when
    /// the default-OFF `ELAI_ZERO_RETENTION_ENABLE` master kill-switch is set
    /// (R9 experiment, GD-4). Composes G-9 redaction + the G-9 TTL purge + the
    /// F-13 archive shape + the G-2/G-8 evidence that survives (GD-1, never
    /// re-implemented). Emitted by `crate::retention::purge_after_verify` from
    /// the `elai run` post-attestation seal seam.
    /// TOTAL_VARIANTS 566 -> 567.
    RetentionPurged(RetentionPurgedKind),
    /// I-52 (SCHEMA_VERSION 259): agent-callable checkpoint tool outcome event.
    ///
    /// Pattern-2 single-variant 4-arm [`CheckpointToolEventKind`]:
    /// - `Saved`: the agent created a labelled checkpoint (a tagged manifest of
    ///   the named files captured into the H-49 CAS).
    /// - `Rewound`: the agent rewound the working tree to a saved label; the
    ///   restore routed through `timetravel::materialize` (the only writer).
    /// - `Diffed`: the agent diffed the working tree against a saved label.
    /// - `RefusedDiverged`: the rewind was REFUSED because the working tree
    ///   diverged from the checkpoint since save (RV-1 safety; no silent
    ///   overwrite). The row names the diverged path set.
    ///
    /// Default-ON over the H-49 time-travel substrate (kill-switch
    /// `ELAI_TIMETRAVEL_DISABLE=1` disables the underlying store; no separate
    /// I-52 flag). REUSE-vs-NEW resolved to NEW per controller decision: the
    /// existing `EditHistoryEvent` 6-arm Kind lacks `RefusedDiverged` + a user
    /// label, and R1 requires the divergence-refusal observable to LAND.
    /// Composes H-49 `timetravel::{capture_file,materialize}` (GD-1, never
    /// re-implemented). Emitted by the `checkpoint_save/rewind/diff` arms of
    /// `crate::production_agent_rpc::translate_tool_request`.
    /// TOTAL_VARIANTS 567 -> 568.
    CheckpointToolEvent(CheckpointToolEventKind),
    /// I-54 (Track 20 Wave 3): agent-callable structural (AST-aware) edit tool
    /// outcome event.
    ///
    /// Pattern-2 single-variant 5-arm [`StructuralEditAppliedKind`]:
    /// - `Matched`: the search arm ran a tree-sitter query and returned a match
    ///   set (the row carries the match count).
    /// - `Previewed`: the preview arm resolved a query + rewrite template into
    ///   the would-be diff (the row carries the blake3 diff hash); ZERO bytes.
    /// - `Applied`: the previewed rewrite set was applied through the SHIPPED
    ///   PreToolUse edit gates + `dispatch_edit` (per-file results).
    /// - `RefusedStale`: a matched node's bytes changed between preview and
    ///   apply (the AST moved); the apply was REFUSED -- the load-bearing safety
    ///   observable (R1; no silent corruption).
    /// - `ParseFailed`: a broken tree-sitter query / rewrite template was caught
    ///   at preview; ZERO bytes (no regex degrade, stub_ban).
    ///
    /// Reuses `elai_knowledge::TreeSitterAstEvaluator` (one tree-sitter version
    /// source, SLC-3; ast-grep dep BANNED). The taint-sink + FS-blast-radius edit
    /// gates are the safety, so no separate I-54 flag. Emitted by the
    /// `structural_search/_preview/_apply` arms of
    /// `crate::production_agent_rpc::translate_tool_request`.
    /// Matched is the representative arm. TOTAL_VARIANTS 568 -> 569.
    StructuralEditApplied(StructuralEditAppliedKind),
    /// I-4 (SCHEMA_VERSION 261): evidence-ranked search-ladder branch lifecycle
    /// event (Wave-4 KEYSTONE).
    ///
    /// Pattern-2 single-variant 4-arm [`SearchNodeEventKind`]:
    /// - `Expanded`: a candidate branch was generated, its pristine per-branch
    ///   context built (I-5 `build_branch_context`), and run through the I-17
    ///   tri-layer stack; the row carries the branch's verifier receipt.
    /// - `Pruned`: a branch was DROPPED from selection because the tri-layer
    ///   stack hard-vetoed it (`VerdictRow::is_vetoed`) or its oracle was
    ///   flake-quarantined (`FlakeQuarantineLedger::is_quarantined`). A pruned
    ///   branch is terminal (ID-4) -- it can never be Promoted.
    /// - `Promoted`: the best-of-N winner. The row MUST carry verifier-derived
    ///   evidence (tri-layer score + mutation kill-rate + progress score); a
    ///   branch promoted on neural score alone is the ID-4 stub_ban the broken
    ///   `--llm-judge-only` arm FAILS against.
    /// - `BudgetCollapsed`: every candidate was vetoed/quarantined, so the
    ///   ladder produced no survivor and fell back to the linear-retry path (the
    ///   honest no-winner beacon).
    ///
    /// Default-OFF (R9 experiment): emitted only when `ELAI_SEARCH_LADDER_ENABLE`
    /// is truthy AND the D-11 checker runtime is plumbed (the ladder reuses its
    /// registry -- GD-1, no second registry). Composes I-2 workspace isolation /
    /// I-5 pristine context / I-17 tri-layer stack / I-20 mutation receipts
    /// (GD-1, never re-implemented). Emitted by
    /// `crate::search::SearchLadder::run_best_of_n` from the
    /// `task_runner::dispatch_search_ladder` post-task seam.
    /// Expanded is the representative arm. TOTAL_VARIANTS 569 -> 570.
    SearchNodeEvent(SearchNodeEventKind),
    /// I-7 (SCHEMA_VERSION 262): cascading escalation waterfall lifecycle event
    /// (Wave-4). The local-first waterfall fans out K branches (I-2 isolation),
    /// runs the I-17 tri-layer check per branch (via the I-4 ladder), and ONLY on
    /// an all-local-fail condenses the failure set into an I-11 BeliefState and
    /// escalates SYNCHRONOUSLY to the H-7 cloud routing seam.
    ///
    /// Pattern-2 single-variant 4-arm [`EscalationCascadeKind`]:
    /// - `LocalWin`: at least one local branch survived the tri-layer check; the
    ///   task solved on the local lane and the cloud arm NEVER ran (the E-39
    ///   zero-cloud-spend observable).
    /// - `Condensed`: every local branch failed; the failure set was condensed into
    ///   the typed I-11 BeliefState (<=2K, NO raw history — ID-3 / stub_ban).
    /// - `Escalated`: the condensate was routed SYNCHRONOUSLY to the cloud seam
    ///   ([`crate::cloud_mode::CloudModeRouter::route`]); the row carries the
    ///   condensed payload byte size. The billed cloud architect inference is
    ///   D-026 PENDING (never fired from the buildable path).
    /// - `BatchRouted`: journal-completeness ONLY for the OFFLINE H-5 batch lane;
    ///   the live waterfall NEVER emits it (IC-1: the batch lane is structurally
    ///   unreachable from this module — no batch-client import).
    ///
    /// Wire-dark unless `ELAI_ESCALATION_WATERFALL_ENABLE=1` (default-OFF, R9
    /// experiment, GD-4 measured-first-win). Composes I-4 ladder / I-11 BeliefState
    /// / H-7 CloudModeRouter (GD-1, never re-implemented). Emitted by
    /// `crate::escalation_waterfall::EscalationWaterfall::run` from the
    /// `task_runner::maybe_run_escalation_waterfall` drive_task hook.
    /// LocalWin is the representative arm. TOTAL_VARIANTS 570 -> 571.
    EscalationCascade(EscalationCascadeKind),
    /// I-22 (SCHEMA_VERSION 264): delta-progress shaped-reward scorer event.
    ///
    /// Pattern-2 single-variant 3-arm [`ProgressRewardScoredKind`]:
    /// - `Delta`: shaped reward phi(h_t) - phi(h_{t-1}) scored for one branch step
    ///   from deterministic SSB observables (test_pass, compile_err_inv, req_covered).
    ///   Every phi component is deterministic and enumerable from the journal (ID-4).
    /// - `FirstErrorTruncated`: branch truncated at first i-17 HardVeto (step k-1);
    ///   the H-49 CAS fork node id at step k-1 is recorded in the payload.
    /// - `DecayApplied`: D consecutive zero-delta steps triggered the no-progress decay
    ///   multiplier (D and weights read from `metrics/progress_reward_weights.toml`,
    ///   ID-5: measured, not paper-derived constants).
    ///
    /// Wire-dark unless `ELAI_PROGRESS_REWARD_ENABLE=1` (default-OFF, R9 experiment,
    /// GD-4 measured-first-win). Composes i-17 TriLayerStack / H-49 timetravel /
    /// i-37 EarsRequirementSet (GD-1, never re-implemented). Emitted by
    /// `crate::search::score::DeltaProgressScorer::score_step` from the
    /// `task_runner::dispatch_progress_reward` drive_task hook (immediately after
    /// `dispatch_tri_layer_verdict`). Delta is the representative arm.
    /// TOTAL_VARIANTS 572 -> 573.
    ProgressRewardScored(ProgressRewardScoredKind),
    /// I-24 (SCHEMA_VERSION 266): isomorphic canary anti-gaming event.
    ///
    /// Pattern-2 single-variant 3-arm [`PrmCanaryEventKind`]:
    /// - `CanaryInjected`: a canary pair (original + AST-preserving isomorphic variant)
    ///   was selected and both variants sent through the L2 scorer; score delta is
    ///   below the bias threshold (no divergence detected this injection).
    /// - `StyleBiasDetected`: the score delta between the pair exceeded the threshold
    ///   from `metrics/canary_bias_threshold.toml` (ID-5: ELAI-measured, not paper-
    ///   derived); the scorer is flagged as style-biased for this run.
    /// - `ScorerNeutralized`: N consecutive `StyleBiasDetected` events within one run
    ///   caused the scorer weight to be zeroed for the remainder of the run (ID-4:
    ///   ScorerNeutralized is an L2-only effect; it CANNOT override an L1 HardVeto).
    ///
    /// Wire-dark unless `ELAI_CANARY_INJECTOR_ENABLE=1` (default-OFF, R9 experiment).
    /// R8 compliance: the canary pair set is FROZEN in
    /// `fixtures/canary_pairs.json` (gate-manifest.txt entry); the running system
    /// NEVER generates new canaries from the model under test at runtime.
    /// Composes i-17 TriLayerStack L2 leg (GD-1, never re-implements L2 invocation).
    /// Emitted by `crate::canary_injector::emit_prm_canary_to_ssb` from
    /// `crate::canary_injector::CanaryInjector::maybe_inject`, called at the
    /// `TriLayerStack::run_verdict` L2 injection site. CanaryInjected is the
    /// representative arm.
    /// TOTAL_VARIANTS 573 -> 574.
    PrmCanaryEvent(PrmCanaryEventKind),
    /// I-27 (SCHEMA_VERSION 265): verified-outcome-rate vs compute convergence
    /// curve measurement beacon.
    ///
    /// Flat beacon (no Kind discriminator). Emitted by
    /// `crates/cli/src/bench_convergence.rs::run_inner` from the
    /// `CliAction::BenchConvergence` dispatch arm in `crates/cli/src/main.rs`.
    ///
    /// The row carries `curve_points[]` (compute_level, mean_rate, ci_low,
    /// ci_high), `knee_compute_level`, `non_monotonic_at[]`, `seed_count` (>=5
    /// per I-38 statistical floor), and `fixture_suite`. Results are
    /// recomputable byte-identically under `ELAI_SEED` (GD-2). Monotonicity is
    /// REPORTED, not guaranteed: if rate dips between adjacent compute levels,
    /// `non_monotonic_at` names the level (GD-5 honesty, stub_ban).
    ///
    /// Live per-node compute-spend resolution from I-4 `SearchNodeEvent` rows is
    /// PENDING-on-I-4 (I-4 SearchLadder/search/ladder.rs UNBUILT at HEAD). The
    /// Tier-1 fixture leg uses cost_per_task_cents as the cost axis (E-49 live).
    ///
    /// Default-ON for the fixture leg (CLI entrypoint, always available).
    /// No kill-switch needed (measurement instrument, not a runtime gate).
    /// Composes E-49 ($/CTS denominator), I-38 (bootstrap CI floor), H-9
    /// (cost cockpit family) (GD-1, never re-implements).
    /// TOTAL_VARIANTS 574 -> 575.
    ConvergenceCurveMeasured,
    /// I-36 (SCHEMA_VERSION 267): per-(role,backend) compiled-prompt artifact event.
    /// Pattern-2 single-variant 3-arm [`PromptCompiledKind`]:
    /// - `Compiled`: a promoted compiled variant was substituted at the adapter boundary;
    ///   the row names the variant hash + backend.
    /// - `TransferGapMeasured`: source-vs-compiled fixture pass-rate delta recorded;
    ///   recomputable under ELAI_SEED (GD-2).
    /// - `Rolledback`: compiled variant regressed its fixture floor; variant removed from
    ///   store and substitution reverts to the base assembled prompt.
    ///
    /// Default-ON (substitution is a no-op pass-through when no compiled variant is stored).
    /// Kill-switch: `ELAI_PROMPT_COMPILE_DISABLE=1` suppresses all substitution and emission.
    /// Offline optimizer (GEPA / MIPROv2-class) is Tier-2 DEFER (cloud optimizer budget).
    /// Compiler output contract: no explicit step-by-step CoT scaffold in compiled strings
    /// (landmine #6: Opus 4.7+ extended thinking auto-interleaves; scaffolds suppress it).
    /// Composes I-32 role_contracts (compiled substitution applies AFTER) + B-37 PerRoleRuntime
    /// (the routing key) + E-8 GEPA (the offline compilation lane, Tier-2) + I-12 ledger gates
    /// (the ID-6 promotion machinery, promote_variant WIRE-DARK[I-36]).
    /// TOTAL_VARIANTS 575 -> 576.
    PromptCompiled(PromptCompiledKind),
    /// I-50 (SCHEMA_VERSION 268): subscription auth broker lifecycle event.
    /// Pattern-2 single-variant 4-arm [`AuthBrokerEventKind`]:
    /// - `TokenAcquired`: a valid credential was resolved (env-key, saved token, or new
    ///   exchange); the row names provider_id only, never token material.
    /// - `Refreshed`: an expired saved token was refreshed and persisted; the row records
    ///   provider_id + whether refresh succeeded.
    /// - `ProviderLinked`: a new provider credential was registered with the broker.
    /// - `RefreshFailed`: refresh was attempted but failed with a typed error; request is
    ///   surfaced to caller as ApiError, never silently swallowed.
    ///
    /// Default-ON (the broker is the mandatory credential resolution path once wired).
    /// Wire-dark at the SSB-emit seam until Arc<Ssb> is threaded through runtime_cmd.rs
    /// (WIRE-DARK[I-50]); the emit path is proven reachable in the contract test.
    /// Composes oauth.rs (storage facade) + anthropic.rs::resolve_startup_auth_source
    /// (Anthropic impl) + B-37 RuntimeFactoryConfig.auth_source (consumer).
    /// TOTAL_VARIANTS 576 -> 577.
    AuthBrokerEvent(AuthBrokerEventKind),
    /// J-1.T4 (SCHEMA_VERSION 269): perception-gateway scaffold lifecycle event.
    /// SINGLE-VARIANT Pattern-2 (41st application). Carries
    /// [`PerceptionScaffoldEventPayload`] with the `kind` discriminator
    /// (4 arms: `preprocessed` / `scaffold_emitted` /
    /// `routed_to_specialist` / `degraded_passthrough`) + free-form
    /// `details_json`. Wire-dark unless
    /// `ELAI_PERCEPTION_GATEWAY_ENABLE=1` (see
    /// [`crate::perception::ELAI_PERCEPTION_GATEWAY_ENABLE_ENV`]).
    ///
    /// Producers (T4 wires scaffold + router gates; T5 wires preprocess):
    ///   * `production_agent_rpc::handle_infer_request` scaffold gate --
    ///     emits `scaffold_emitted` on Ok(ScaffoldEmitted),
    ///     `degraded_passthrough` on Ok(DegradedPassthrough) + Err.
    ///   * `production_agent_rpc::handle_infer_request` router gate --
    ///     emits `routed_to_specialist` on RouteToPixelDiff,
    ///     `degraded_passthrough` on the DegradedPassthrough decision.
    ///   * `perception::preprocess` (T5) -- emits `preprocessed` once the
    ///     preprocess call site is wired into handle_infer_request. T4
    ///     ships the Kind arm with NO emit site (unused enum variant, not
    ///     a WIRE-DARK pub fn -- the controller decision records this fork).
    ///
    /// NoImagePresent is a no-op (scaffold fires ONLY on image-bearing
    /// dispatches, plan line 63). Single sql_str `perception_scaffold_event`
    /// (Pattern-2 single-variant convention; the Kind discriminator rides
    /// inside the payload, mirroring DocumentVisionRagEvent).
    ///
    /// See workflow/reasoning/J-1-decisions.md (T4).
    /// TOTAL_VARIANTS 577 -> 578.
    PerceptionScaffoldEvent(PerceptionScaffoldEventKind),
    /// K-18a follow-on (SCHEMA_VERSION 270): PS-5 recurrence-hit-rate flat
    /// beacon, restoring GD-2 journal-recomputability + dashboard-ability
    /// for `elai bench case-reuse` (K-18a shipped the runner + JSON report
    /// but explicitly DEFERRED the SSB beacon, ABORT-4: coordinated
    /// schema-bump walled off from the runner change).
    ///
    /// Flat beacon (no Kind discriminator), mirrors [`Self::ConvergenceCurveMeasured`].
    /// Emitted by `crates/cli/src/bench_case_reuse.rs::run_inner` from the
    /// `CliAction::BenchCaseReuse` dispatch arm.
    ///
    /// The row carries `recurrence_hit_rate_at{1,3,5}` / `false_match_rate_at{1,3,5}`,
    /// the seeded bootstrap CI on hit_rate@ref_k, corpus cardinality, and
    /// `corpus_blake3` (GD-2: recomputable byte-identically under the pinned seed).
    ///
    /// Default-ON (fixture-suite CLI measurement instrument, no kill-switch;
    /// mirrors ConvergenceCurveMeasured's always-available fixture leg).
    /// Composes K-18a `case_reuse_runner` (producer, GD-1: never re-implements
    /// the metric), PS-5 (proof-spine owner number).
    /// TOTAL_VARIANTS 578 -> 579.
    RecurrenceHitRateMeasured,
    /// K-4 SP1 follow-on (SCHEMA_VERSION 271): composed gate-tax budget flat
    /// beacon. Flat beacon (no Kind discriminator), mirrors
    /// [`Self::RecurrenceHitRateMeasured`]. Emitted by
    /// `crates/cli/src/bench_gate_tax.rs::run` from the
    /// `CliAction::BenchGateTax` dispatch arm.
    ///
    /// Carries `coding_task_gate_tax_ms` + `task_class_label`,
    /// `audit_gate_tax_ms` (SP2 placeholder) + `audit_unbounded`,
    /// `over_block_rate`, the RIDER-b coverage counts, up to 3 ranked cost
    /// contributors, `runner_status` (Measured vs RunnerPresent), `n_gates`,
    /// `n_fires`. NEVER sums `coding_task_gate_tax_ms` with
    /// `audit_gate_tax_ms` (rider a, unit honesty).
    ///
    /// Default-ON (fixture-or-journal CLI measurement instrument, no
    /// kill-switch). Composes K-4 `gate_tax_runner` (producer, GD-1), PS-2
    /// (proof-spine composed-gate-tax owner number).
    /// TOTAL_VARIANTS 579 -> 580.
    GateTaxMeasured,
    /// J-28 follow-on: UsageTurnEvent -- the uniform per-turn token-usage
    /// row emitted by BOTH the cloud lane (grounding_client.rs, beside the
    /// existing CloudSpend row) and the local lane (runtime_orchestrator.rs,
    /// beside the existing BackendCostAccrued row). Flat variant (mirrors
    /// RecurrenceHitRateMeasured/GateTaxMeasured -- ONE sql_str, no Kind on
    /// the action-type enum itself); the 3-arm Kind discriminator
    /// (TurnRecorded / EstimateApplied / NormalizationApplied) rides
    /// inside [`crate::ssb::UsageTurnEventPayload::kind`] as a payload
    /// field, same convention as `loop_kind` on
    /// `SecurityClassRegressionBlockedPayload`.
    ///
    /// Neither CloudSpend nor BackendCostAccrued is removed or touched --
    /// this is the uniform (model,task,session,lane)-keyed join they could
    /// not provide (BackendCostAccruedPayload carries no model/task/
    /// session key). Default-ON, no kill-switch. See
    /// workflow/reasoning/J-28-decisions.md.
    /// TOTAL_VARIANTS 580 -> 581.
    UsageTurnEvent,
    /// J-9 (SCHEMA_VERSION 273): hardware/model eligibility catalog event.
    /// Flat action type; CatalogBuilt/TierAssigned/EligibilityChecked/LaneSeeded
    /// rides in HardwareModelCatalogEventPayload.kind. Tier-1 emits CatalogBuilt
    /// from the one-shot startup SystemProbe seam. TOTAL_VARIANTS 581 -> 582.
    HardwareModelCatalogEvent,
    /// K-2 (SCHEMA_VERSION 274): hybrid-vs-force-cloud routing economics
    /// benchmark beacon. Flat payload carrying the deterministic fixture
    /// comparison and honest unfunded-live-lane status. TOTAL_VARIANTS 582 -> 583.
    RoutingEconomicsMeasured,
    /// K-5 (SCHEMA_VERSION 275): deterministic risk-signal driven GateIntensity
    /// selection beacon. Emitted observe-only from the deterministic audit gate.
    /// TOTAL_VARIANTS 583 -> 584.
    GateIntensitySelected,
    /// K-9 (SCHEMA_VERSION 276): model refresh cadence decision row. Flat action
    /// type; CandidateEvaluated/DecisionRecorded/IncumbentKept rides in
    /// ModelRefreshCadencePayload.kind. TOTAL_VARIANTS 584 -> 585.
    ModelRefreshCadence,
    /// PS-3 (SCHEMA_VERSION 277): hallucination-delta gate-ON vs gate-OFF
    /// measured. Flat beacon (no Kind discriminator). Mirrors
    /// SafetyDeltaMeasured shape. TOTAL_VARIANTS 585 -> 586.
    HallucinationDeltaMeasured,
    /// PS-READINESS-E2ERUNNER Slice B (SCHEMA_VERSION 278): local-lane e2e
    /// functional pass over the 4B sidecar. Flat beacon. TOTAL_VARIANTS 586
    /// -> 587.
    LocalLaneE2eMeasured,
    /// J-13 (SCHEMA_VERSION 279): local-lane determinism telemetry event.
    /// Pattern-2 single-variant 4-arm [`LocalDeterminismKind`] discriminator
    /// carried on the payload. Emitted from the local-lane e2e runner and
    /// determinism audit probes. TOTAL_VARIANTS 587 -> 588.
    LocalDeterminismEvent(LocalDeterminismKind),
    /// J-14 (SCHEMA_VERSION 282): repro-pack + RRA verify upgrade telemetry event.
    ///
    /// Pattern-2 single-variant 4-arm [`ReproPackKind`]:
    /// - `TierEvaluated`: emitted from `rra::verify_rra` with the tier verdict
    ///   (ActionFaithful / TokenFaithful / ScoreFaithful) + recorded downgrades.
    /// - `FirstDivergenceLocated`: emitted on T1/T2 mismatch with the first divergent
    ///   position + both values (consumed by G-10 drift canary).
    /// - `FlowFingerprintMismatch`: emitted from `compiled_pipeline::TemplateCrystallizer`
    ///   when a crystallized template's tool sequence is perturbed (de-crystallize + LLM fallback).
    /// - `FixtureExtracted`: emitted by `elai rra export-fixture` with the per-fixture blake3.
    ///
    /// Default-ON; no kill-switch (verify path, never masks the verdict). TOTAL_VARIANTS 590 -> 591.
    ReproPackEvent(ReproPackKind),
    /// J-16 (SCHEMA_VERSION 283): blast-radius / anti-regression gate event.
    ///
    /// Pattern-2 single-variant 4-arm [`BlastRadiusRegressionKind`]:
    /// - `ConeComputed`: emitted with the reverse-dep cone of each changed symbol.
    /// - `ScopeViolationFlagged`: emitted when an edit touches an out-of-scope file.
    /// - `RegressionSetSelected`: emitted with the selected existing-test set.
    /// - `SilentDeletionFlagged`: emitted when a pub item is removed without declaration.
    ///
    /// Default-ON; OBSERVE-ONLY (GD-4). TOTAL_VARIANTS 591 -> 592.
    BlastRadiusRegressionEvent(BlastRadiusRegressionKind),
    /// J-30 T7b (SCHEMA_VERSION 284): provider BYOK lifecycle event -- the
    /// observability row for `elai providers configure` (KeyStored /
    /// EndpointVerified) and custom_providers.toml registration
    /// (CustomBackendRegistered). Flat variant (mirrors `UsageTurnEvent` --
    /// ONE sql_str, no Kind on the action-type enum itself); the 3-arm
    /// [`crate::ssb::ProviderByokEventKind`] discriminator rides inside
    /// [`crate::ssb::ProviderByokEventPayload::kind`] as a payload field.
    /// NO key/secret/token material in the payload (plan stub_ban).
    /// TOTAL_VARIANTS 592 -> 593.
    ProviderByokEvent,
    /// J-32 (SCHEMA_VERSION 285): front-door intent routing decision row.
    /// Pattern-2 flat variant (mirrors ProviderByokEvent); the 5-arm
    /// [`crate::ssb::IntentRoutedKind`] discriminator rides inside the
    /// payload. Emitted by FrontDoorRouter on every classified/confirmed/
    /// rejected/bypassed/clarified decision. Payload carries lane, tier,
    /// confidence, arg_span offsets, decision, risk level. NEVER the
    /// rewritten text (there is none; arg_spans are byte-offsets).
    /// TOTAL_VARIANTS 593 -> 594.
    IntentRouted,
    /// J-34 (SCHEMA_VERSION 286): empirical context calibration event.
    /// Pattern-2 flat variant (mirrors PerceptionScaffoldEvent); the 4-arm
    /// [`crate::ssb::ContextCalibrationKind`] discriminator (CurveMeasured /
    /// ThresholdApplied / PlacementApplied / CalibrationInvalidated) rides
    /// inside [`crate::ssb::ContextCalibrationEventPayload::kind`]. Emitted
    /// by `CalibrationArtifactStore::insert` (CurveMeasured) and the leg-2 /
    /// leg-3 seams (later slices).
    /// TOTAL_VARIANTS 594 -> 595.
    ContextCalibrationEvent(ContextCalibrationKind),
    /// J-36 (SCHEMA_VERSION 287): local prefix-KV cache event.
    /// Pattern-2 flat variant (mirrors ContextCalibrationEvent); the 4-arm
    /// [`crate::ssb::LocalPrefixCacheKind`] discriminator (HeadCacheHit /
    /// HeadCacheMiss / TailReprefill / CacheIneligible) rides inside
    /// [`crate::ssb::LocalPrefixCacheEventPayload::kind`]. Emitted by
    /// `emit_local_prefix_cache_event` (prefix_cache_ssb.rs) on the local
    /// dispatch path.
    /// TOTAL_VARIANTS 595 -> 596.
    LocalPrefixCacheEvent(LocalPrefixCacheKind),
    /// J-24 (SCHEMA_VERSION 288): behavior steering event.
    /// Pattern-2 flat variant (mirrors ContextCalibrationEvent); the 3-arm
    /// [`crate::ssb::BehaviorSteeringKind`] discriminator
    /// (SteeringVectorApplied / SteeringEffectMeasured /
    /// ObservationSizePreassessed) rides inside
    /// [`crate::ssb::BehaviorSteeringEventPayload::kind`]. Emitted by
    /// `emit_behavior_steering_event` (behavior_steering_ssb.rs).
    /// TOTAL_VARIANTS 596 -> 597.
    BehaviorSteeringEvent(BehaviorSteeringKind),
    /// J-35 intent-control layer: typed intent spec + premise-check +
    /// route + journaled assumptions. Pattern-2 flat variant (mirrors
    /// BehaviorSteeringEvent); the 4-arm
    /// [`crate::ssb::IntentControlKind`] discriminator
    /// (SpecReconstructed / PremiseFlagged / RouteDecided /
    /// AssumptionLogged) rides inside
    /// [`crate::ssb::IntentControlEventPayload::kind`]. Emitted by
    /// `run_intent_control` (intent_control.rs) on the task-lane
    /// intake before decomposition.
    /// TOTAL_VARIANTS 597 -> 598.
    IntentControlEvent(IntentControlKind),
    /// J-39 T2: node admission gate event. Pattern-2 flat variant
    /// (mirrors ContextCalibrationEvent); the 3-arm
    /// [`crate::ssb::NodeAdmissionEventKind`] discriminator (Admitted /
    /// SplitForced / EscalatedToCloud) rides inside
    /// [`crate::ssb::NodeAdmissionEventPayload::kind`]. Emitted by
    /// `emit_node_admission_event` (node_admission.rs) at the task_runner
    /// SPLIT_TASK phase_sizing branch.
    NodeAdmissionEvent(NodeAdmissionEventKind),
    /// J-31 system-canvas event. Carries a [`SystemContractEventPayload`];
    /// the [`crate::ssb::SystemContractEventKind`] discriminator (NodeBound /
    /// EdgeDeclared / EdgeVerdict / NodeDrifted) rides inside the payload.
    /// Emitted by `emit_system_contract_event` (system_contract_event.rs).
    SystemContractEvent(SystemContractEventKind),
    /// J-12 HarnessInterventionEvent. Pattern-2; Kind rides in payload.
    /// Emitted by `emit_harness_intervention_event` (intervention_ssb.rs).
    HarnessInterventionEvent(HarnessInterventionKind),
    /// J-54 UNIT8: free-pool rotation / backoff / trust-downgrade.
    /// Pattern-2 single-variant; 3-arm FreePoolEventKind rides in payload.
    FreePoolEvent,
    /// J-73 Slice C: retrieval abstained event. Pattern-2 single-variant;
    /// 3-arm RetrievalAbstainedKind rides in payload.
    RetrievalAbstained,
    /// J-78: context-access spec event. Pattern-2 single-variant;
    /// 3-arm ContextAccessSpecKind rides in payload.
    ContextAccessSpec,
    /// I-28 Slice 1: deterministic branch-grain DAG credit assignment.
    DagCreditAssigned,
    /// I-30 Slice 1a: SFT exhaust event (flat variant; 3-arm
    /// SftExhaustEventKind rides in the payload).
    SftExhaustEvent,
    /// J-23: review-gate catalog event (flat variant; 4-arm
    /// ReviewGateCatalogEventKind rides in the payload).
    ReviewGateCatalogEvent,
    /// L-2: one auditable risk score row per non-empty ClaimSet.
    ClaimRiskScored,
    /// J-42 T2: reasoning distill-reset checkpoint. Flat variant; the 3-arm
    /// [`crate::ssb::ReasoningCheckpointEventKind`] discriminator rides inside
    /// [`crate::ssb::ReasoningCheckpointEventPayload::kind`]. Emitted by the
    /// `ReflectEventSink` bridge from the `chat_reflect` refine loop.
    ReasoningCheckpointEvent(ReasoningCheckpointEventKind),
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
            // B-30 (SCHEMA_VERSION 36): language grammar registry status.
            Self::LanguageGrammarStatus => "language_grammar_status",
            // B-29 Phase 5 (SCHEMA_VERSION 37): symbol-graph tool call row.
            Self::SymbolQuery => "symbol_query",
            // B-32 (SCHEMA_VERSION 38): minified-file chunking SSB row.
            Self::MinifiedFileChunked => "minified_file_chunked",
            // B-31.5 (SCHEMA_VERSION 39): 5 deferred B-31 SSB variants.
            Self::IndexCheckpoint => "index_checkpoint",
            Self::IndexResume => "index_resume",
            Self::IndexCancel => "index_cancel",
            Self::LockAcquired => "lock_acquired",
            Self::LockContested => "lock_contested",
            // B-33 follow-up (SCHEMA_VERSION 40): symbol-graph build event row.
            Self::SymbolGraphUpdated => "symbol_graph_updated",
            // B-40 (SCHEMA_VERSION 41): FRAMING substate beacons.
            Self::IdeaFramingStarted => "idea_framing_started",
            Self::ProjectClassClassified => "project_class_classified",
            Self::ScopeCeilingAdvisory => "scope_ceiling_advisory",
            Self::CriticRoundComplete => "critic_round_complete",
            Self::IdeaFramed => "idea_framed",
            Self::TradeoffCardEmitted => "tradeoff_card_emitted",
            // B-34 (SCHEMA_VERSION 42)
            Self::EntryPointsDetected => "entry_points_detected",
            Self::ArtifactsIndexed => "artifacts_indexed",
            Self::ArtifactSearched => "artifact_searched",
            Self::ArtifactStale => "artifact_stale",
            // B-38 (SCHEMA_VERSION 43): ambiguity gate beacons.
            Self::AmbiguityScored => "ambiguity_scored",
            Self::AmbiguityFloorFailed => "ambiguity_floor_failed",
            Self::AmbiguityStreakAdvanced => "ambiguity_streak_advanced",
            // B-42 (SCHEMA_VERSION 44): cosmos tier knowledge worker beacons.
            Self::CosmosRefreshStarted => "cosmos_refresh_started",
            Self::CosmosArtifactUpdated => "cosmos_artifact_updated",
            Self::CosmosArtifactDecayed => "cosmos_artifact_decayed",
            Self::CosmosGroundingUsed => "cosmos_grounding_used",
            // B-41 (SCHEMA_VERSION 45): per-level auditor mode beacons.
            Self::PerLevelAuditStarted => "per_level_audit_started",
            Self::PerLevelAuditVerdict => "per_level_audit_verdict",
            Self::PerLevelAuditCascade => "per_level_audit_cascade",
            // B-37C (SCHEMA_VERSION 46): multi-runtime backend beacons.
            Self::RuntimeFallback => "runtime_fallback",
            Self::RuntimeHealthChanged => "runtime_health_changed",
            Self::CliSubprocessSpawned => "cli_subprocess_spawned",
            Self::CliSubprocessKilled => "cli_subprocess_killed",
            // E-51 (SCHEMA_VERSION 96): edge-inference beacons.
            Self::MlxSubprocessSpawned => "mlx_subprocess_spawned",
            Self::MlxSubprocessKilled => "mlx_subprocess_killed",
            Self::LlamaCppSubprocessSpawned => "llamacpp_subprocess_spawned",
            Self::LlamaCppSubprocessKilled => "llamacpp_subprocess_killed",
            Self::MlxApplePlatformDetected => "mlx_apple_platform_detected",
            Self::LlamaCppEpDetected => "llamacpp_ep_detected",
            Self::RuntimeChainBuilt => "runtime_chain_built",
            // E-17 SP2 (SCHEMA_VERSION 97): RAG retrieval-quality beacons.
            Self::DeepPoolCapped => "deep_pool_capped",
            Self::RrfQualityGateFired => "rrf_quality_gate_fired",
            Self::HydeSkippedCodeQuery => "hyde_skipped_code_query",
            Self::RrfComponentWeights => "rrf_component_weights",
            // C-1 (SCHEMA_VERSION 47): knowledge + reasoning producer beacons.
            Self::GrcStepEmitted => "grc_step_emitted",
            Self::GroundingSweepResult => "grounding_sweep_result",
            Self::CsqEvaluated => "csq_evaluated",
            Self::AuditorRubberStampRejected => "auditor_rubber_stamp_rejected",
            Self::PerRoleDecodingApplied => "per_role_decoding_applied",
            Self::HydeQuery => "hyde_query",
            Self::VectorCacheHitRate => "vector_cache_hit_rate",
            Self::QueryTrace => "query_trace",
            // C-2 (SCHEMA_VERSION 48): orchestration + cosmos + stagnation beacons.
            Self::PhaseNodeReady => "phase_node_ready",
            Self::StepNodeReady => "step_node_ready",
            Self::BabyStepReady => "baby_step_ready",
            Self::DagRepairEscalated => "dag_repair_escalated",
            Self::AgentStatusChanged => "agent_status_changed",
            Self::StagnationSpinningDetected => "stagnation_spinning_detected",
            Self::StagnationOscillationDetected => "stagnation_oscillation_detected",
            Self::LateralThinkingActivated => "lateral_thinking_activated",
            Self::GoalDriftWarning => "goal_drift_warning",
            Self::WatchdogTimeout => "watchdog_timeout",
            // C-7 (SCHEMA_VERSION 49): reviewer verdict beacon.
            Self::ChangeDecision => "change_decision",
            // C-8 (SCHEMA_VERSION 50): search-mode cross-process bridge.
            Self::SearchModeChanged => "search_mode_changed",
            // C-5 (SCHEMA_VERSION 51): privacy-redaction event.
            Self::RedactionApplied => "redaction_applied",
            // C-10 (SCHEMA_VERSION 52): anti-spiral lateral-thinking +
            // goal-drift Block beacons.
            Self::LateralThinkingSucceeded => "lateral_thinking_succeeded",
            Self::LateralThinkingExhausted => "lateral_thinking_exhausted",
            Self::GoalDriftBlock => "goal_drift_block",
            // C-13 (SCHEMA_VERSION 53): IPC-dispatcher production-swap
            // beacon — WIP commit squash completion.
            Self::WipSquashComplete => "wip_squash_complete",
            // C-22 (SCHEMA_VERSION 54): feasibility-probe completeness beacons.
            Self::AlternativeExplored => "alternative_explored",
            Self::FeasibilityEscalated => "feasibility_escalated",
            // C-23 (SCHEMA_VERSION 55): SessionDag branch-UI beacons.
            Self::BranchForked => "branch_forked",
            Self::BranchMerged => "branch_merged",
            Self::BranchClosed => "branch_closed",
            // C-25 (SCHEMA_VERSION 56): compiled-pipeline + template-crystallizer
            // + background-task beacons.
            Self::PlanSkeleton => "plan_skeleton",
            Self::CompiledPipelineFired => "compiled_pipeline_fired",
            Self::TemplateCrystallized => "template_crystallized",
            Self::BackgroundTaskStarted => "background_task_started",
            Self::BackgroundTaskComplete => "background_task_complete",
            Self::DecompileTriggered => "decompile_triggered",
            // C-26 (SCHEMA_VERSION 58): constrained-decoding + futility primitives.
            Self::FutilityDetected => "futility_detected",
            Self::FutilityOutputStagnation => "futility_output_stagnation",
            // C-27 (SCHEMA_VERSION 59): Testing-trust T(-1) + T-0 beacons.
            Self::WiringEvidence => "wiring_evidence",
            Self::WiringViolation => "wiring_violation",
            Self::DeterminismDrift => "determinism_drift",
            // C-28 (SCHEMA_VERSION 60): Testing-trust T-1 mutation testing.
            Self::MutationSurvived => "mutation_survived",
            // C-29 (SCHEMA_VERSION 61): Testing-trust T-2 + T-3 beacons.
            Self::IntentConfirmed => "intent_confirmed",
            Self::MrViolated => "mr_violated",
            // C-30 (SCHEMA_VERSION 62): Surgical Mode + Oracle Mode beacons.
            Self::SurgicalPlanCreated => "surgical_plan_created",
            Self::SurgicalLocWarning => "surgical_loc_warning",
            Self::OracleCacheHit => "oracle_cache_hit",
            Self::OracleCacheMiss => "oracle_cache_miss",
            Self::ProjectDetectionSignal => "project_detection_signal",
            // C-32 (SCHEMA_VERSION 63): Code-Completeness Layer 2 beacons.
            Self::SkeletonGenerated => "skeleton_generated",
            Self::SkeletonFlushed => "skeleton_flushed",
            Self::MiniCriticVerdict => "mini_critic_verdict",
            Self::StructuralRetryStageAttempt => "structural_retry_stage_attempt",
            Self::BridgeFileGenerated => "bridge_file_generated",
            // D-1 (SCHEMA_VERSION 64): ReasoningTemplate ArchiveChain lifecycle.
            Self::ArchiveEntryAdded => "archive_entry_added",
            Self::ArchiveEntryRetired => "archive_entry_retired",
            Self::TemplatePromoted => "template_promoted",
            Self::TemplateCandidateRejected => "template_candidate_rejected",
            // E-5 (SCHEMA_VERSION 65): memory_facts bi-temporal lifecycle beacons.
            Self::MemoryFactExtracted => "memory_fact_extracted",
            Self::MemoryFactSuperseded => "memory_fact_superseded",
            // D-2 (SCHEMA_VERSION 66): PPM verifier beacons.
            Self::PpmScored => "ppm_scored",
            Self::PpmGatingFailed => "ppm_gating_failed",
            // E-1 (SCHEMA_VERSION 67): infinite-context chat illusion beacons.
            Self::ChatReflectionCompleted => "chat_reflection_completed",
            Self::MemoryEvolutionRun => "memory_evolution_run",
            // E-1 dashboard slice (SCHEMA_VERSION 68): MemoryNotes / CoreBlock
            // producer + conflict beacons.
            Self::MemoryNoteInserted => "memory_note_inserted",
            Self::MemoryNoteEvolved => "memory_note_evolved",
            Self::MemoryConflictDetected => "memory_conflict_detected",
            Self::CoreBlockReplaced => "core_block_replaced",
            // D-3 (SCHEMA_VERSION 69): multi-verifier consensus aggregator.
            Self::VerifierConsensusPass => "verifier_consensus_pass",
            Self::VerifierConsensusFail => "verifier_consensus_fail",
            // D-4 (SCHEMA_VERSION 70): tripwire stack beacons.
            Self::TripwireFired => "tripwire_fired",
            Self::TemplateAutoReverted => "template_auto_reverted",
            // D-4 GAP-2 (SCHEMA_VERSION 108): concurrent-detector tripwire row.
            Self::TripwireFiredConcurrent => "tripwire_fired_concurrent",
            // D-19 (SCHEMA_VERSION 71): PRMBench adversarial-gaming benchmark beacon.
            Self::PrmBenchRunCompleted => "prmbench_run_completed",
            // D-5 (SCHEMA_VERSION 72): BAI Thompson sampling promotion round.
            Self::BaiArmUpdated => "bai_arm_updated",
            // D-5 closure (SCHEMA_VERSION 208): unified promotion-decision summary.
            Self::BaiPromotionDecision => "bai_promotion_decision",
            // D-6 (SCHEMA_VERSION 73): held-out task store lifecycle.
            Self::HeldOutTaskAdded => "held_out_task_added",
            Self::HeldOutTaskUsed => "held_out_task_used",
            // D-7 (SCHEMA_VERSION 74): cross-domain transfer canary beacons.
            Self::CanaryRunCompleted => "canary_run_completed",
            Self::CanaryFamilyEvaluated => "canary_family_evaluated",
            Self::LockedKeysAuditRun => "locked_keys_audit_run",
            Self::LockedKeysViolationDetected => "locked_keys_violation_detected",
            // E-46 (SCHEMA_VERSION 76): hierarchical decomposition tuning beacons.
            Self::TaskAdvanced => "task_advanced",
            Self::AgentlessShortcutTaken => "agentless_shortcut_taken",
            Self::GoalDriftFired => "goal_drift_fired",
            // D-10 (SCHEMA_VERSION 77): mode-routing predicate beacon.
            Self::ClaimRouted => "claim_routed",
            // D-21 (SCHEMA_VERSION 78): mode-routing coverage benchmark beacon.
            Self::ModeRoutingBenchRunCompleted => "mode_routing_bench_run_completed",
            // D-11 (SCHEMA_VERSION 79): constructive-checker registry beacon.
            Self::CheckerResultEmitted => "checker_result_emitted",
            // D-13 (SCHEMA_VERSION 80): adversarial-verification beacons.
            Self::AdversaryGenerated => "adversary_generated",
            Self::ConsensusReevaluatedWithCounter => "consensus_reevaluated_with_counter",
            // D-12 (SCHEMA_VERSION 81): OperationalizationCheck beacon.
            Self::BabyStepDispatched => "baby_step_dispatched",
            // D-16 (SCHEMA_VERSION 82): UniCR conformal-refusal beacons.
            Self::ConformalTauComputed => "conformal_tau_computed",
            Self::ConformalRefusalDecided => "conformal_refusal_decided",
            // D-15 (SCHEMA_VERSION 83): discovery-vs-search NoveltyClass beacon.
            Self::NoveltyClassified => "novelty_classified",
            // D-18 (SCHEMA_VERSION 84): typed-ObservableEffect predicate beacon.
            Self::ObservableEffectVerified => "observable_effect_verified",
            // D-17 (SCHEMA_VERSION 85): honest-refusal SLA monitoring beacons.
            Self::RefusalRateSnapshot => "refusal_rate_snapshot",
            Self::RefusalSlaBreach => "refusal_sla_breach",
            // D-22 (SCHEMA_VERSION 86): conformal-calibration validation beacon.
            Self::ConformalValidationCompleted => "conformal_validation_completed",
            // D-20 (SCHEMA_VERSION 87): ARC-AGI-2 OOD probe gating beacon.
            Self::ArcBenchmarkCompleted => "arc_benchmark_completed",
            Self::ToolGated => "tool_gated",
            Self::ToolEager => "tool_eager",
            Self::ToolLazyResolved => "tool_lazy_resolved",
            Self::ToolDenied => "tool_denied",
            Self::QuantBackendSelected => "quant_backend_selected",
            Self::KvCacheDtypeSoftwareSim => "kv_cache_dtype_software_sim",
            Self::AwqCheckpointBuilt => "awq_checkpoint_built",
            Self::CliMcpDispatch => "cli_mcp_dispatch",
            Self::CliMcpServerStarted => "cli_mcp_server_started",
            Self::CliMcpServerStopped => "cli_mcp_server_stopped",
            Self::FirecrackerRestoreSlow => "firecracker_restore_slow",
            Self::SnapshotUniquenessRegenerated => "snapshot_uniqueness_regenerated",
            Self::SnapshotEvicted => "snapshot_evicted",
            Self::SandboxBackendSelected => "sandbox_backend_selected",
            Self::AgentForkedFromSnapshot => "agent_forked_from_snapshot",
            Self::ResearcherNetEgressBlocked => "researcher_net_egress_blocked",
            Self::McpSchemaPinViolation => "mcp_schema_pin_violation",
            Self::McpSanitizationRuleApplied => "mcp_sanitization_rule_applied",
            Self::McpBudgetEnforced => "mcp_budget_enforced",
            Self::McpLegacySseTransport => "mcp_legacy_sse_transport",
            Self::McpLlmGuardBlocked => "mcp_llm_guard_blocked",
            Self::AnthropicGatewayDetected => "anthropic_gateway_detected",
            Self::MemoryTierAssigned => "memory_tier_assigned",
            Self::MemoryQueryFiltered => "memory_query_filtered",
            Self::SafetyCriticalBlocked => "safety_critical_blocked",
            Self::ArgvDestructiveDenied => "argv_destructive_denied",
            Self::ArgvApprovalRequired => "argv_approval_required",
            Self::TrustDialogHookDeferred => "trust_dialog_hook_deferred",
            Self::TrustDialogEnvDeferred => "trust_dialog_env_deferred",
            Self::McpPinVersionRefused => "mcp_pin_version_refused",
            Self::AuditorLineageCollision => "auditor_lineage_collision",
            Self::UntrustedMemoryWriteBlocked => "untrusted_memory_write_blocked",
            Self::IncidentRecordCreated => "incident_record_created",
            // E-32
            Self::FitmCalibrationApplied => "fitm_calibration_applied",
            Self::DeepModeActivated => "deep_mode_activated",
            Self::DeepModeEscalationChosen => "deep_mode_escalation_chosen",
            Self::YarnRefused => "yarn_refused",
            // B-3 (SCHEMA_VERSION 98): AntiStalenessDrawer + RulesDrawer beacons.
            Self::DepAuditCompleted => "dep_audit_completed",
            Self::BaselineRunCompleted => "baseline_run_completed",
            Self::DeterministicAuditRun => "deterministic_audit_run",
            Self::RuleTransition => "rule_transition",
            // B-4 (SCHEMA_VERSION 99): canary drift snapshot row.
            Self::CanaryDrift => "canary_drift",
            // B-7 (SCHEMA_VERSION 100): producer-side wiring closure batch.
            Self::TreePositionChanged => "tree_position_changed",
            Self::FailureRecorded => "failure_recorded",
            Self::PackageRefVerified => "package_ref_verified",
            Self::TechStackDecided => "tech_stack_decided",
            // B-19 (SCHEMA_VERSION 101): per-agent-execution DAIRA marker
            // aggregation row.
            Self::MarkerReport => "marker_report",
            // B-25 (SCHEMA_VERSION 102): Phase 4J Empire integration.
            Self::EmpireConnected => "empire_connected",
            Self::EmpireDisconnected => "empire_disconnected",
            Self::EmpireEventReceived => "empire_event_received",
            Self::EmpireCapabilityInvoked => "empire_capability_invoked",
            Self::EmpireRepairProposed => "empire_repair_proposed",
            // C-5 follow-on (SCHEMA_VERSION 103).
            Self::RouteBlocked => "route_blocked",
            Self::WarnLimitReached => "warn_limit_reached",
            Self::CostEstimate => "cost_estimate",
            // C-36 (SCHEMA_VERSION 104).
            Self::MucocoMutationApplied => "mucoco_mutation_applied",
            // D-3 closure (SCHEMA_VERSION 105).
            Self::PpmUnavailable => "ppm_unavailable",
            Self::VerifierDisagreement => "verifier_disagreement",
            Self::CrossContextDispatched => "cross_context_dispatched",
            // C-38 (SCHEMA_VERSION 107).
            Self::PagerankRun => "pagerank_run",
            Self::DkgRebuildStarted => "dkg_rebuild_started",
            Self::DkgRebuildComplete => "dkg_rebuild_complete",
            // C-40 (SCHEMA_VERSION 109).
            Self::GithubPrOpened => "github_pr_opened",
            Self::GithubPrStatusFetched => "github_pr_status_fetched",
            Self::GithubPrClosed => "github_pr_closed",
            Self::GithubPrMerged => "github_pr_merged",
            // E-41 (SCHEMA_VERSION 135): PR review surface.
            Self::PrReviewComposed => "pr_review_composed",
            Self::PrCommentWatched => "pr_comment_watched",
            Self::PrPackageVerifyResult => "pr_package_verify_result",
            Self::PrSemgrepGateResult => "pr_semgrep_gate_result",
            Self::PrAutoMergeBlocked => "pr_auto_merge_blocked",
            // D-5 closure (SCHEMA_VERSION 110): Thompson-sample beacon.
            Self::ThompsonSampled => "thompson_sampled",
            // C-41 (SCHEMA_VERSION 111): skills-driven workflow registry.
            Self::SkillTriggered => "skill_triggered",
            Self::SkillExecuted => "skill_executed",
            // D-6 closure (SCHEMA_VERSION 112): held-out promotion-gate +
            // refresh-event beacons.
            Self::HeldOutPromotionGate => "held_out_promotion_gate",
            Self::HeldOutRefresh => "held_out_refresh",
            // C-42 (SCHEMA_VERSION 113): Semantic Gateway L2 RBAC beacons.
            Self::GatewayPolicyAllowed => "gateway_policy_allowed",
            Self::GatewayPolicyDenied => "gateway_policy_denied",
            Self::GatewayHitlPending => "gateway_hitl_pending",
            Self::CrossDomainRegression => "cross_domain_regression",
            Self::CanaryFamilyPass => "canary_family_pass",
            Self::CanaryInsufficientData => "canary_insufficient_data",
            // D-8 closure (SCHEMA_VERSION 115): cryptographic archive verify.
            Self::ArchiveVerifyResult => "archive_verify_result",
            // C-43 closure (SCHEMA_VERSION 116): Phase 4H scope-freeze.
            Self::ScopeFreezeRatified => "scope_freeze_ratified",
            Self::ScopeFreezeViolated => "scope_freeze_violated",
            // C-44 (SCHEMA_VERSION 117): Composer 2 paid-backend cost accrual.
            Self::BackendCostAccrued => "backend_cost_accrued",
            // C-45 (SCHEMA_VERSION 120): CANTANTE credit attribution.
            Self::AgentRoleDropped => "agent_role_dropped",
            Self::CreditAttributionRun => "credit_attribution_run",
            Self::CreditAttributionResult => "credit_attribution_result",
            // C-46 (SCHEMA_VERSION 121): Uno-Orchestra selective delegation.
            Self::SelectiveDelegationDecision => "selective_delegation_decision",
            Self::SelectiveDelegationEscalated => "selective_delegation_escalated",
            Self::SelectiveDelegationCostSaved => "selective_delegation_cost_saved",
            // C-47 (SCHEMA_VERSION 122): Case-Based Calibration of Tool-Use.
            Self::ToolCallLogged => "tool_call_logged",
            Self::ToolCalibrationHintEmitted => "tool_calibration_hint_emitted",
            Self::ToolCalibrationDriftDetected => "tool_calibration_drift_detected",
            // C-49 (SCHEMA_VERSION 123): iterative-distillation experience.
            Self::PromptPolicyDistillationRun => "prompt_policy_distillation_run",
            Self::PromptFragmentSelected => "prompt_fragment_selected",
            Self::PromptFragmentDiscarded => "prompt_fragment_discarded",
            // D-14 (SCHEMA_VERSION 124): sycophancy probe row pair.
            Self::SycophancyProbeDispatched => "sycophancy_probe_dispatched",
            Self::SycophancyFlag => "sycophancy_flag",
            // D-23 (SCHEMA_VERSION 142): sycophancy detection benchmark beacon.
            Self::SycophancyBenchRunCompleted => "sycophancy_bench_run_completed",
            // D-24 (SCHEMA_VERSION 143): cross-domain canary regression
            // benchmark beacon.
            Self::CanaryBenchRunCompleted => "canary_bench_run_completed",
            // D-25 (SCHEMA_VERSION 144): determinism + archive tamper-suite
            // benchmark beacon.
            Self::DeterminismTamperSuiteCompleted => "determinism_tamper_suite_completed",
            // D-28 (SCHEMA_VERSION 145): adversarial example farming run
            // beacon. Host-side scaffolding for the Adv-RM (arXiv:2504.06141)
            // farm loop; live PPM loop + ppm_finetune.sh + bench_prmbench
            // v1/v2 are RUNPOD-PENDING.
            Self::AdversarialFarmRunCompleted => "adversarial_farm_run_completed",
            // D-29 (SCHEMA_VERSION 146): hierarchical BAI round
            // completion beacon. Wire-dark gated by ELAI_BAI_MODE=
            // hierarchical AND ELAI_BAI_ENABLE=1. Paper labels:
            // arXiv:2111.06929 SUPPORTING EVIDENCE; arXiv:2106.10417
            // DIRECT GROUNDING. Live regret baseline RUNPOD-PENDING.
            Self::HierarchicalBaiRoundCompleted => "hierarchical_bai_round_completed",
            // D-30 (SCHEMA_VERSION 147): calibration bootstrap warm-start
            // completion beacon. Wire-dark gated by
            // ELAI_CALIBRATION_BOOTSTRAP_ENABLE. Paper labels:
            // arXiv:2509.01455 UniCR COMPOSITION; arXiv:2109.07958
            // TruthfulQA DATA ORIGIN; Politis & Romano 1994 JASA Vol 89
            // §3 DIRECT GROUNDING. Live D-22 validation + block-
            // bootstrap CI activation RUNPOD-PENDING.
            Self::CalibrationBootstrapCompleted => "calibration_bootstrap_completed",
            // C-50 (SCHEMA_VERSION 125): Contract-Coding STRUCTURAL
            // contract test scaffolding row pair.
            Self::ContractTestScaffolded => "contract_test_scaffolded",
            Self::ContractViolationDetected => "contract_violation_detected",
            // C-52 (SCHEMA_VERSION 126): Dafny Formal Verification
            // Spike heal-loop termination beacon.
            Self::DafnyVerifyCompleted => "dafny_verify_completed",
            // D-16 closure (SCHEMA_VERSION 127): durable TauUpdater
            // tau persistence beacon.
            Self::CalibrationCorpusUpdated => "calibration_corpus_updated",
            // E-12 Tier-B (SCHEMA_VERSION 128): production fullstack
            // codegen — framework profile + legacy linter strip.
            Self::FrameworkProfileSelected => "framework_profile_selected",
            Self::LegacyConfigStripped => "legacy_config_stripped",
            // D-18 closure (SCHEMA_VERSION 129): typed-ObservableEffect
            // 4-variant pack — dispatch beacon + Eureka Goodhart guard
            // + Coscientist sandbox/production observation taxonomy.
            Self::ObservableEffectDispatched => "observable_effect_dispatched",
            Self::GoodhartRiskFlag => "goodhart_risk_flag",
            Self::SandboxObservation => "sandbox_observation",
            Self::ProductionObservationProposal => "production_observation_proposal",
            // E-13 (SCHEMA_VERSION 130): TDD verifier-agent beacon
            // family — cycle entry + per-phase result rows.
            Self::TddCycleStarted => "tdd_cycle_started",
            Self::TddTestFailed => "tdd_test_failed",
            Self::TddTestPassed => "tdd_test_passed",
            // Tranche B-7 triple (SCHEMA_VERSION 195): TDD verification
            // gate substrate (4-arm Kind discriminator).
            Self::TddVerificationEvent => "tdd_verification_event",
            // E-14 (SCHEMA_VERSION 131): browser-e2e agent beacon family —
            // journey run + per-assertion + per-snapshot rows.
            Self::BrowserE2eRunStarted => "browser_e2e_run_started",
            Self::BrowserE2eAssertionFailed => "browser_e2e_assertion_failed",
            Self::BrowserE2eAssertionPassed => "browser_e2e_assertion_passed",
            Self::BrowserE2eSnapshotCaptured => "browser_e2e_snapshot_captured",
            // E-18 (SCHEMA_VERSION 132): tool-use reliability gate beacons.
            Self::ToolReliabilityGatePassed => "tool_reliability_gate_passed",
            Self::ToolReliabilityGateRejected => "tool_reliability_gate_rejected",
            Self::ToolCallSpilled => "tool_call_spilled",
            Self::ToolCallRetryEscalated => "tool_call_retry_escalated",
            // E-37 (SCHEMA_VERSION 133): prompt-injection defense beacons.
            Self::InjectionAttemptDetected => "injection_attempt_detected",
            Self::InjectionAttemptBlocked => "injection_attempt_blocked",
            Self::SpotlightWrapped => "spotlight_wrapped",
            Self::SpotlightBreached => "spotlight_breached",
            Self::ToolDescriptionPinMismatch => "tool_description_pin_mismatch",
            Self::CapabilityGateRejected => "capability_gate_rejected",
            Self::ProvenanceTagged => "provenance_tagged",
            Self::PromptArmorVerdict => "prompt_armor_verdict",
            Self::InjectionFixtureReplayed => "injection_fixture_replayed",
            Self::OutputClassifierFlagged => "output_classifier_flagged",
            // E-40 (SCHEMA_VERSION 134): orchestrator-rooted symbol-graph beacons.
            Self::SymbolGraphQueried => "symbol_graph_queried",
            Self::SymbolGraphCacheLoaded => "symbol_graph_cache_loaded",
            Self::LspOracleConsulted => "lsp_oracle_consulted",
            // E-44 (SCHEMA_VERSION 136): durability layer beacons.
            Self::CheckpointWritten => "checkpoint_written",
            Self::CheckpointResumed => "checkpoint_resumed",
            Self::CheckpointMigrated => "checkpoint_migrated",
            Self::IdempotentDispatchSkipped => "idempotent_dispatch_skipped",
            Self::DurabilityLockBlocked => "durability_lock_blocked",
            // E-45 (SCHEMA_VERSION 137): cross-language drift detector beacons.
            Self::CrossLangDriftDetected => "cross_lang_drift_detected",
            Self::CrossLangPairScanned => "cross_lang_pair_scanned",
            Self::SchemaVersionMismatch => "schema_version_mismatch",
            // E-49 (SCHEMA_VERSION 138): coding-agent metric suite (scope-lean).
            Self::CodingAgentMetricsCaptured => "coding_agent_metrics_captured",
            Self::SweBenchProRunStarted => "swe_bench_pro_run_started",
            Self::SweBenchProRunCompleted => "swe_bench_pro_run_completed",
            Self::RefineBenchScoreEmitted => "refinebench_score_emitted",
            Self::MetricsAggregationRun => "metrics_aggregation_run",
            // E-52 Tier-B (SCHEMA_VERSION 139): LSP-as-verifier diagnostic
            // probe + SCIP refresh beacons.
            Self::DiagnosticProbeDispatched => "diagnostic_probe_dispatched",
            Self::DiagnosticProbeCompleted => "diagnostic_probe_completed",
            Self::DiagnosticProbeBlock => "diagnostic_probe_block",
            Self::ScipRefreshCompleted => "scip_refresh_completed",
            // E-53 (SCHEMA_VERSION 140): formal-verify pipeline
            // boundary beacon. Single atomic +1 bump consolidating
            // the plan's originally-proposed 13 variants per the
            // C-52 single-DafnyVerifyCompleted precedent.
            Self::FormalVerifierCompleted => "formal_verifier_completed",
            // E-35 (SCHEMA_VERSION 141): 10-variant atomic bump for
            // the structured-output substrate (XGrammar + INSTRUCTOR +
            // CodeAct + SEARCH/REPLACE + cloud strict-schema).
            Self::XgrammarMaskApplied => "xgrammar_mask_applied",
            Self::InstructorModeActivated => "instructor_mode_activated",
            Self::GrammarModeActivated => "grammar_mode_activated",
            Self::CodeactActionExecuted => "codeact_action_executed",
            Self::SearchReplaceTier1Hit => "search_replace_tier1_hit",
            Self::SearchReplaceCollisionFallback => "search_replace_collision_fallback",
            Self::CloudStrictSchemaApplied => "cloud_strict_schema_applied",
            Self::GrammarCacheLoaded => "grammar_cache_loaded",
            Self::StructuredEngineRejected => "structured_engine_rejected",
            Self::LegacyJsonObjectRefused => "legacy_json_object_refused",
            // E-6 (SCHEMA_VERSION 148): MAST-grounded multi-agent failure-mode defenses.
            Self::AuditMastTagged => "audit_mast_tagged",
            Self::IsoBudgetMeasured => "iso_budget_measured",
            Self::PersonaReanchored => "persona_reanchored",
            Self::CrossFamilyUnavailable => "cross_family_unavailable",
            // E-7 (SCHEMA_VERSION 149): darwinian self-improvement substrate.
            Self::DarwinianGenerationCompleted => "darwinian_generation_completed",
            // E-3 (SCHEMA_VERSION 150): runtime handoff signal beacon.
            Self::RuntimeHandoffSignal => "runtime_handoff_signal",
            // E-26 (SCHEMA_VERSION 151): inference-engines 2026 pack.
            Self::SglangBackendRegistered => "sglang_backend_registered",
            Self::XgrammarSchemaCompiled => "xgrammar_schema_compiled",
            Self::RuntimeColdStartObserved => "runtime_cold_start_observed",
            // E-2 (SCHEMA_VERSION 152): streaming timing histogram.
            Self::StreamingTimingRecorded => "streaming_timing_recorded",
            // E-2 (SCHEMA_VERSION 153): UX pacing vote A/B beacon.
            Self::UxPacingVoteRecorded => "ux_pacing_vote_recorded",
            // E-28 (SCHEMA_VERSION 154-156): attention-kernels 2026 pack.
            Self::StreamingLlmEngaged => "streaming_llm_engaged",
            Self::SdpaBackendPinned => "sdpa_backend_pinned",
            Self::YarnRopeScalingApplied => "yarn_rope_scaling_applied",
            // E-8 (SCHEMA_VERSION 162): self-evolving prompts substrate.
            Self::PromptOptimizationCompleted => "prompt_optimization_completed",
            // E-10 (SCHEMA_VERSION 163): hostile-critic structural simplifications.
            Self::CriticDecisionEvent => "critic_decision_event",
            // E-43 (SCHEMA_VERSION 164): agent skill systems telemetry.
            Self::SkillSystemEvent => "skill_system_event",
            // E-9 (SCHEMA_VERSION 165): external agent-eval benchmark suite.
            Self::BenchmarkRunObserved => "benchmark_run_observed",
            // E-19 (SCHEMA_VERSION 166): reasoning quality probe.
            Self::ReasoningQualityProbe => "reasoning_quality_probe",
            // E-54 (SCHEMA_VERSION 167): eval-corpus repair pre-deploy.
            Self::EvalCorpusEvent => "eval_corpus_event",
            Self::RewardLadderEvent => "reward_ladder_event",
            // E-11 (SCHEMA_VERSION 169): SaaS prior-art catalog +
            // recommender substrate.
            Self::SaasPriorArtEvent => "saas_prior_art_event",
            // E-22 (SCHEMA_VERSION 170): may-2026 github trending sweep
            // substrate (SKILL.md export + microVM sandbox + lazy MCP
            // schema savings).
            Self::GithubTrendingEvent => "github_trending_event",
            // E-23 (SCHEMA_VERSION 171): 2026 developer-discourse
            // positioning + UX-guard substrate.
            Self::DeveloperDiscourseEvent => "developer_discourse_event",
            // E-25 (SCHEMA_VERSION 172): claude-code-architecture catalog
            // substrate.
            Self::ClaudeCodeArchitectureEvent => "claude_code_architecture_event",
            Self::AgenticObservabilityEvent => "agentic_observability_event",
            // E-31 (SCHEMA_VERSION 174): coding-model-leaderboard scraper.
            Self::CodingModelLeaderboardEvent => "coding_model_leaderboard_event",
            // E-38 (SCHEMA_VERSION 175): 2026 AI roadmap signals + EU AI
            // Act compliance pre-gate substrate.
            Self::AiRoadmapSignalEvent => "ai_roadmap_signal_event",
            // E-39 (SCHEMA_VERSION 176): cost + token economics
            // substrate. SINGLE-VARIANT Pattern-2 (28th application);
            // CostEventKind 8-arm discriminator covers
            // task_aborted_by_spend_cap / velocity_cap_breached /
            // automix_verifier_verdict / cost_router_decision /
            // cost_estimate_inaccurate / free_tier_quota_approaching /
            // free_tier_quota_exceeded / layered_cap_manual_override.
            Self::CostTokenEvent => "cost_token_event",
            // E-50 (SCHEMA_VERSION 177): SaaS architecture pattern
            // catalog substrate. SINGLE-VARIANT Pattern-2 (29th
            // application); SaasArchPatternEventKind N-arm discriminator
            // covers 11 fire-points (gate / RLS-emit / scoped-token /
            // MoR / realtime / async / rate-limit / OTel-SLO / platform-
            // unsupported / archetype-override).
            Self::SaasArchPatternEvent => "saas_arch_pattern_event",
            // E-4 (SCHEMA_VERSION 178): chat UX critique substrate.
            // SINGLE-VARIANT Pattern-2 (30th application);
            // ChatUxCritiqueKind 13-arm discriminator covers claim /
            // sycophancy / pinned-rules / re-anchor / doom-loop /
            // completion / clarifier / refusal / mid-stream / honest-
            // correction fire-points.
            Self::ChatUxCritiqueEvent => "chat_ux_critique_event",
            // E-16 (SCHEMA_VERSION 179): VLM image validation Pattern-2.
            Self::VlmImageValidationEvent => "vlm_image_validation_event",
            // E-20 (SCHEMA_VERSION 180): local-cloud routing substrate.
            // SINGLE-VARIANT Pattern-2 (32nd application);
            // RoutingEventKind 9-arm discriminator covers
            // confidence_emitted / hybrid_escalation /
            // difficulty_classified / privacy_blocked / output_filtered /
            // burn_alert / reasoning_mode / majority_vote /
            // role_priority_chosen. Wire-dark unless
            // ELAI_LOCAL_CLOUD_ROUTING_ENABLE=1.
            Self::LocalCloudRoutingEvent => "local_cloud_routing_event",
            // E-21 (SCHEMA_VERSION 181): peer-systems harness substrate.
            // SINGLE-VARIANT Pattern-2 (33rd application);
            // PeerSystemsEventKind 4-arm discriminator covers
            // critic_candidates_generated / critic_score_computed /
            // critic_pick_committed / critic_rerank_skipped. Wire-dark
            // unless ELAI_PEER_SYSTEMS_ENABLE=1.
            Self::PeerSystemsEvent => "peer_systems_event",
            // E-36 (SCHEMA_VERSION 182): document + vision RAG substrate.
            // SINGLE-VARIANT Pattern-2 (34th application);
            // DocumentVisionRagEventKind 5-arm discriminator covers
            // vision_chunk_emitted / vision_fallback_triggered /
            // vision_rag_policy_decision / visual_artifact_check_verdict /
            // doc_parser_cloud_fallback_blocked_by_mode. Wire-dark
            // unless ELAI_DOCUMENT_VISION_RAG_ENABLE=1.
            Self::DocumentVisionRagEvent => "document_vision_rag_event",
            // E-42 (SCHEMA_VERSION 183): tree-sitter grammar health
            // substrate. SINGLE-VARIANT Pattern-2 (35th application);
            // TreeSitterGrammarEventKind 4-arm discriminator covers
            // abi_probe_mismatch / svelte_runes_parse_health /
            // tier_b_grammar_loaded / tree_sitter_skipped_large_file.
            // Wire-dark unless ELAI_TREE_SITTER_GRAMMARS_ENABLE=1.
            Self::TreeSitterGrammarEvent => "tree_sitter_grammar_event",
            // E-46 (SCHEMA_VERSION 184): planning-decomposition substrate.
            // SINGLE-VARIANT Pattern-2 (36th application);
            // PlanningDecompositionEventKind 8-arm discriminator.
            // Wire-dark unless ELAI_PLANNING_DECOMPOSITION_ENABLE=1.
            Self::PlanningDecompositionEvent => "planning_decomposition_event",
            // E-48 (SCHEMA_VERSION 185): web-research orchestrator
            // hardening substrate. SINGLE-VARIANT Pattern-2 (37th application);
            // WebResearchEventKind 10-arm discriminator. Wire-dark
            // unless ELAI_WEB_RESEARCH_ORCHESTRATOR_ENABLE=1.
            Self::WebResearchEvent => "web_research_event",
            // E-44 (SCHEMA_VERSION 186): durability observability substrate.
            // SINGLE-VARIANT Pattern-2 (38th application);
            // DurabilityEventKind 10-arm discriminator. Wire-dark
            // unless ELAI_DURABILITY_OBSERVABILITY_ENABLE=1.
            Self::DurabilityEvent => "durability_event",
            // E-31 (SCHEMA_VERSION 187): head_dim invariant verification.
            // SINGLE-VARIANT Pattern-2 (39th application);
            // HeadDimInvariantVerifiedKind 2-arm discriminator
            // (verified / refused). Wire-dark unless
            // ELAI_HEAD_DIM_INVARIANT_SSB_ENABLE=1.
            Self::HeadDimInvariantVerified => "head_dim_invariant_verified",
            // E-3 (SCHEMA_VERSION 188): role-shown / model-hidden default
            // telemetry. SINGLE-VARIANT Pattern-2 (40th application).
            // No Kind discriminator. Wire-dark unless
            // ELAI_ROLE_VISIBILITY_TELEMETRY_ENABLE=1.
            Self::RoleShownModelHidden => "role_shown_model_hidden",
            // E-49 (SCHEMA_VERSION 189): coding-agent metrics named-token
            // substrate. SINGLE-VARIANT Pattern-2 (41st application);
            // CodingMetricNamedTokenKind 3-arm discriminator covers
            // stable_pass_at_1 / harness_attribution_tuple / stop_verified.
            // Wire-dark unless ELAI_CODING_METRIC_NAMED_TOKEN_ENABLE=1.
            Self::CodingMetricNamedToken => "coding_metric_named_token",
            // E-3 sub-phase 4A (SCHEMA_VERSION 190): runtime handoff event
            // substrate. SINGLE-VARIANT Pattern-2 (42nd application);
            // RuntimeHandoffEventKind 6-arm discriminator. Wire-dark
            // unless ELAI_E3_RUNTIME_HANDOFF_ENABLE=1.
            Self::E3RuntimeHandoffEvent => "e3_runtime_handoff_event",
            // E-31 sub-phase 6A (SCHEMA_VERSION 191): cloud-route decision
            // event substrate. SINGLE-VARIANT Pattern-2 (43rd application);
            // RouteDecisionEventKind 3-arm discriminator covers
            // cloud_route_chosen / tier_model_swapped / local_route_kept.
            // Wire-dark unless ELAI_ROUTE_DECISION_SSB_ENABLE=1.
            Self::RouteDecisionEvent => "route_decision_event",
            // E-47 (SCHEMA_VERSION 192): LoRA fine-tuning ship-gate
            // substrate. SINGLE-VARIANT Pattern-2 (44th application);
            // LoraShipGateEventPhase 3-arm discriminator covers
            // refused / calibration_missing / calibration_passed.
            // Wire-dark unless ELAI_LORA_SHIP_GATE_ENABLE=1.
            Self::LoraShipGateEvent => "lora_ship_gate_event",
            // E-40 Tranche B-2 (SCHEMA_VERSION 193): SymbolGraph
            // augmentation substrate. SINGLE-VARIANT Pattern-2 (45th
            // application); SymbolGraphEventKind 4-arm discriminator
            // covers symbol_dsl_executed / dataflow_impact_computed /
            // agent_impact_claim_refuted / scope_graph_resolved.
            Self::SymbolGraphEvent => "symbol_graph_event",
            // Tranche B-6 quad (SCHEMA_VERSION 194): four new SSB
            // substrates. See header SCHEMA_VERSION block for full docs.
            Self::McpAuthzEvent => "mcp_authz_event",
            Self::ToolCalibrationEvent => "tool_calibration_event",
            Self::ReflectBudgetSoftWarn => "reflect_budget_soft_warn",
            Self::ContractCodingEvent => "contract_coding_event",
            // F-1 / F-2a (SCHEMA_VERSION 196): vibe-coding security gate variants.
            // VibeGateBlocked is Pattern-2; each Kind arm maps to its own sql_str.
            Self::VibeGateBlocked(kind) => kind.as_sql_str(),
            Self::SecurityRuleBlocked => "security_rule_blocked",
            // F-4 / Slice B-1 (SCHEMA_VERSION 197): TestIntegrityEvent Pattern-2.
            // Each Kind arm maps to its own sql_str via the Kind's as_sql_str().
            Self::TestIntegrityEvent(kind) => kind.as_sql_str(),
            // F-3p1 / Slice C1 (SCHEMA_VERSION 198): ToolFilterEvent Pattern-2.
            // Each Kind arm maps to its own sql_str via the Kind's as_sql_str().
            Self::ToolFilterEvent(kind) => kind.as_sql_str(),
            // F-9 / Slice D1 (SCHEMA_VERSION 199): DepWriteTimeGate slopsquat blocked.
            Self::WriteTimeSlopsquatBlocked => "write_time_slopsquat_blocked",
            // F-10 / Slice A1 (SCHEMA_VERSION 200): security-monotonicity guard.
            Self::SecurityClassRegressionBlocked => "security_class_regression_blocked",
            // F-2b / Slice A1 (SCHEMA_VERSION 201): insecure-rate keystone bench beacon.
            Self::InsecureRateMeasured => "insecure_rate_measured",
            // I-9 (SCHEMA_VERSION 263): error-amplification (alpha_t) bench beacon.
            Self::ErrorAmplificationMeasured => "error_amplification_measured",
            // F-6 / Slice A1 (SCHEMA_VERSION 202): SkillLibraryEvent Pattern-2.
            // Each Kind arm maps to its own sql_str via the Kind's as_sql_str().
            Self::SkillLibraryEvent(kind) => kind.as_sql_str(),
            // F-5 / Slice A1 (SCHEMA_VERSION 203): StalenessQuarantineBroadcast
            // flat beacon (no Kind discriminator).
            Self::StalenessQuarantineBroadcast => "staleness_quarantine_broadcast",
            // F-8 / Slice A1 (SCHEMA_VERSION 204): SymbolicConflictDetected
            // flat beacon (no Kind discriminator).
            Self::SymbolicConflictDetected => "symbolic_conflict_detected",
            // F-7 / Slice A1 (SCHEMA_VERSION 205): TemplateUtilityRanked
            // flat beacon (no Kind discriminator).
            Self::TemplateUtilityRanked => "template_utility_ranked",
            // F-13 / Slice A1 (SCHEMA_VERSION 206): SessionArchived
            // flat beacon (no Kind discriminator).
            Self::SessionArchived => "session_archived",
            // F-11 / Slice A1 (SCHEMA_VERSION 207): DarwinianAdmit config-DGM
            // admit-loop beacon. Pattern-2 single-variant; the 3-arm
            // DarwinianAdmitKind discriminator lives on the payload, so the
            // wire tag is a single string (mirrors DarwinianGenerationCompleted).
            Self::DarwinianAdmit => "darwinian_admit",
            // G-plan variants.
            Self::McpRugPullBlocked => "mcp_rug_pull_blocked",
            Self::McpScanClassifierBlocked => "mcp_scan_classifier_blocked",
            Self::TaintFlow => "taint_flow",
            Self::BlastRadiusBlocked => "blast_radius_blocked",
            Self::TrifectaQuarantined => "trifecta_quarantined",
            Self::RunArtifactEmitted => "run_artifact_emitted",
            Self::ReviewSurfaceScored => "review_surface_scored",
            Self::ProvenanceAttestationEmitted => "provenance_attestation_emitted",
            Self::SafetyDeltaMeasured => "safety_delta_measured",
            Self::ModelDriftSuspected => "model_drift_suspected",
            Self::SecretRedacted => "secret_redacted",
            Self::ClaimConfidenceScored => "claim_confidence_scored",
            Self::SteeringVectorRefreshTriggered => "steering_vector_refresh_triggered",
            Self::CanvasOpApplied => "canvas_op_applied",
            Self::CanvasViewEvent => "canvas_view_event",
            Self::CanvasActivityEvent => "canvas_activity_event",
            Self::BatchLifecycle => "batch_lifecycle",
            // H-plan verifiability variants.
            Self::DeflationGateEvaluated => "deflation_gate_evaluated",
            Self::LeakageBlocked => "leakage_blocked",
            Self::LookaheadDifferential => "lookahead_differential",
            Self::TradingNullTest => "trading_null_test",
            Self::RerunHashEquality => "rerun_hash_equality",
            Self::SimRealParity => "sim_real_parity",
            Self::ResultManifestVerified => "result_manifest_verified",
            Self::DifferentialDivergence => "differential_divergence",
            Self::GateSelfValidation => "gate_self_validation",
            Self::RewardTampering => "reward_tampering",
            Self::HoldoutBudgetEvent => "holdout_budget_event",
            Self::DriftInterlock => "drift_interlock",
            Self::TrialRegistered => "trial_registered",
            Self::MucocoInconsistencyDetected => "mucoco_inconsistency_detected",
            Self::MucocoSkipReason => "mucoco_skip_reason",
            Self::UserDecisionSuperseded => "user_decision_superseded",
            Self::StalenessEvalMeasured => "staleness_eval_measured",
            Self::DiscoveryRatified => "discovery_ratified",
            Self::DiscoveryRejected => "discovery_rejected",
            Self::FactValidationFailed => "fact_validation_failed",
            Self::MemoryGovernanceEvent => "memory_governance_event",
            Self::McpToolDeferred => "mcp_tool_deferred",
            Self::McpSchemaBudgetMeasured => "mcp_schema_budget_measured",
            Self::SessionRewound => "session_rewound",
            Self::AgentNotificationEmitted => "agent_notification_emitted",
            Self::PlanDecisionEvent => "plan_decision_event",
            Self::StopConfirmationEvent => "stop_confirmation_event",
            Self::AbstentionScored => "abstention_scored",
            Self::CheckerGapDetected => "checker_gap_detected",
            Self::ReasoningBudgetScaled => "reasoning_budget_scaled",
            Self::FitnessEvaluated => "fitness_evaluated",
            Self::TrapBehaviorDetected => "trap_behavior_detected",
            Self::PromotionScoreAggregated => "promotion_score_aggregated",
            Self::CladeSelected => "clade_selected",
            Self::EvalFirewallEvent => "eval_firewall_event",
            Self::EditHistoryEvent => "edit_history_event",
            Self::HandoffLifecycleEvent => "handoff_lifecycle_event",
            Self::FindingAnchored => "finding_anchored",
            Self::CitationAnchorsBound => "citation_anchors_bound",
            Self::PlanStateDerived => "plan_state_derived",
            Self::ObjectiveSealed(_) => "objective_sealed",
            // I-34 (SCHEMA_VERSION 234): CRANE hybrid grammar-pin gate beacon.
            Self::GrammarGateEvent(_) => "grammar_gate_event",
            // I-38 (SCHEMA_VERSION 235): QDELTA defective-diff rate flat beacon.
            Self::QdeltaMeasured => "qdelta_measured",
            Self::SnowballingMeasured => "snowballing_measured",
            Self::KvCacheTuned => "kv_cache_tuned",
            // I-43 (SCHEMA_VERSION 236): claim-scoper kill-gate (3-arm Pattern-2).
            Self::ClaimScopeMeasured(_) => "claim_scope_measured",
            // I-10 (SCHEMA_VERSION 237): cache economics event (3-arm Pattern-2).
            Self::CacheEconomicsEvent(_) => "cache_economics_event",
            // I-8 (SCHEMA_VERSION 238): trajectory aborted (3-arm Pattern-2).
            Self::TrajectoryAborted(_) => "trajectory_aborted",
            // I-1 (SCHEMA_VERSION 239): directive supervisor outcome (3-arm Pattern-2).
            Self::DirectiveExecuted(_) => "directive_executed",
            // I-33 (SCHEMA_VERSION 240): mechanical decision governance primitive (3-arm Pattern-2).
            Self::GovernancePrimitiveFired(_) => "governance_primitive_fired",
            // I-32 (SCHEMA_VERSION 241): structural role-scope contract applied (3-arm Pattern-2).
            Self::RoleContractApplied(_) => "role_contract_applied",
            // I-35 (SCHEMA_VERSION 242): temporal tool-sequence enforcement verdict (3-arm Pattern-2).
            Self::ToolSequenceVerdict(_) => "tool_sequence_verdict",
            // I-41 (SCHEMA_VERSION 243): spec-time critique lenses advisory beacon (3-arm Pattern-2).
            Self::CritiqueLensFinding(_) => "critique_lens_finding",
            // I-46 (SCHEMA_VERSION 244): bash interceptor redirect/fixup/prefix outcome (3-arm Pattern-2).
            Self::BashRedirected(_) => "bash_redirected",
            // I-47 (SCHEMA_VERSION 245): verifier-input hardening + dispatch rule injection (3-arm Pattern-2).
            Self::PromptHardeningApplied(_) => "prompt_hardening_applied",
            // I-2 (SCHEMA_VERSION 246): CoW workspace isolation barrier outcome (4-arm Pattern-2).
            Self::BranchWorkspaceEvent(_) => "branch_workspace_event",
            // I-5 (SCHEMA_VERSION 247): per-branch context rebuild outcome (3-arm Pattern-2).
            Self::ContextPruned(_) => "context_pruned",
            // I-11 (SCHEMA_VERSION 248): typed versioned condensate receipt (3-arm Pattern-2).
            Self::BeliefStateEvent(_) => "belief_state_event",
            // I-13 (SCHEMA_VERSION 249): write-time memory guard outcome (3-arm Pattern-2).
            Self::MemoryGuardEvent(_) => "memory_guard_event",
            // I-14 (SCHEMA_VERSION 250): deterministic context-compression receipt (3-arm Pattern-2).
            Self::ContextTransformReceipt(_) => "context_transform_receipt",
            // I-12 (SCHEMA_VERSION 251): evidence-gated playbook curator outcome (4-arm Pattern-2).
            Self::PlaybookEvent(_) => "playbook_event",
            // I-17 (SCHEMA_VERSION 252): tri-layer evaluation-stack per-branch verdict (4-arm Pattern-2).
            Self::TriLayerVerdict(_) => "tri_layer_verdict",
            // I-18 (SCHEMA_VERSION 253): per-row rolling blake3 chain beacon (3-arm Pattern-2).
            Self::RowChainSealed(_) => "row_chain_sealed",
            // I-19 (SCHEMA_VERSION 254): TDD enforcer pre-flight gate blocked event (3-arm Pattern-2).
            Self::TddPreflightBlocked(kind) => kind.as_sql_str(),
            // I-20 (SCHEMA_VERSION 255): budgeted mutation scheduler (4-arm Pattern-2).
            Self::MutationBudgetedRun(kind) => kind.as_sql_str(),
            // I-21 (SCHEMA_VERSION 256): SprintContractEvent Pattern-2 5-arm.
            Self::SprintContractEvent(kind) => kind.as_sql_str(),
            // I-37 (SCHEMA_VERSION 257): RequirementVerified Pattern-2 3-arm.
            Self::RequirementVerified(kind) => kind.as_sql_str(),
            // I-44 (SCHEMA_VERSION 258): RetentionPurged Pattern-2 3-arm.
            Self::RetentionPurged(kind) => kind.as_sql_str(),
            // I-52 (SCHEMA_VERSION 259): CheckpointToolEvent Pattern-2 4-arm.
            Self::CheckpointToolEvent(kind) => kind.as_sql_str(),
            // I-54 (SCHEMA_VERSION 260): StructuralEditApplied Pattern-2 5-arm.
            Self::StructuralEditApplied(kind) => kind.as_sql_str(),
            // I-4 (SCHEMA_VERSION 261): SearchNodeEvent Pattern-2 4-arm.
            Self::SearchNodeEvent(kind) => kind.as_sql_str(),
            // I-7 (SCHEMA_VERSION 262): EscalationCascade Pattern-2 4-arm.
            Self::EscalationCascade(kind) => kind.as_sql_str(),
            // I-22 (SCHEMA_VERSION 264): ProgressRewardScored Pattern-2 3-arm.
            Self::ProgressRewardScored(kind) => kind.as_sql_str(),
            // I-24 (SCHEMA_VERSION 266): PrmCanaryEvent Pattern-2 3-arm.
            Self::PrmCanaryEvent(kind) => kind.as_sql_str(),
            // I-27 (SCHEMA_VERSION 265): ConvergenceCurveMeasured flat beacon.
            Self::ConvergenceCurveMeasured => "convergence_curve_measured",
            // I-36 (SCHEMA_VERSION 267): PromptCompiled -- per-(role,backend) compiled-prompt
            // artifact event (3-arm Pattern-2). Each Kind arm maps to its own sql_str so
            // Compiled / TransferGapMeasured / Rolledback are distinguishable on the wire.
            Self::PromptCompiled(kind) => kind.as_sql_str(),
            // I-50 (SCHEMA_VERSION 268): AuthBrokerEvent -- subscription auth broker lifecycle.
            // 4-arm Pattern-2; each arm maps to its own sql_str.
            Self::AuthBrokerEvent(kind) => kind.as_sql_str(),
            // J-1.T4 (SCHEMA_VERSION 269): perception-gateway scaffold lifecycle.
            // SINGLE-VARIANT Pattern-2 (41st application); single sql_str, the
            // 4-arm PerceptionScaffoldEventKind discriminator rides inside the
            // payload (mirrors DocumentVisionRagEvent). Wire-dark unless
            // ELAI_PERCEPTION_GATEWAY_ENABLE=1.
            Self::PerceptionScaffoldEvent(_) => "perception_scaffold_event",
            // K-18a follow-on (SCHEMA_VERSION 270): RecurrenceHitRateMeasured flat beacon.
            Self::RecurrenceHitRateMeasured => "recurrence_hit_rate_measured",
            // K-4 SP1 follow-on (SCHEMA_VERSION 271): GateTaxMeasured flat beacon.
            Self::GateTaxMeasured => "gate_tax_measured",
            // J-28 follow-on: UsageTurnEvent flat variant, Kind rides in the payload.
            Self::UsageTurnEvent => "usage_turn_event",
            // J-9: hardware/model catalog flat variant; Kind rides in the payload.
            Self::HardwareModelCatalogEvent => "hardware_model_catalog_event",
            // K-2 (SCHEMA_VERSION 274): routing economics benchmark flat beacon.
            Self::RoutingEconomicsMeasured => "routing_economics_measured",
            // K-5 (SCHEMA_VERSION 275): GateIntensity selection beacon.
            Self::GateIntensitySelected => "gate_intensity_selected",
            // K-9 (SCHEMA_VERSION 276): model refresh cadence decision row.
            Self::ModelRefreshCadence => "model_refresh_cadence",
            // PS-3 (SCHEMA_VERSION 277): hallucination-delta flat beacon.
            Self::HallucinationDeltaMeasured => "hallucination_delta_measured",
            // PS-READINESS-E2ERUNNER Slice B (SCHEMA_VERSION 278): local-lane
            // e2e functional pass flat beacon.
            Self::LocalLaneE2eMeasured => "local_lane_e2e_measured",
            // J-13 (SCHEMA_VERSION 279): local-lane determinism telemetry event.
            Self::LocalDeterminismEvent(_) => "local_determinism_event",
            // J-14 (SCHEMA_VERSION 282): repro-pack + RRA verify upgrade telemetry event.
            Self::ReproPackEvent(_) => "repro_pack_event",
            // J-16 (SCHEMA_VERSION 283): blast-radius / anti-regression gate event.
            Self::BlastRadiusRegressionEvent(_) => "blast_radius_regression_event",
            // J-30 T7b (SCHEMA_VERSION 284): provider BYOK lifecycle event.
            // Flat variant, Kind rides in the payload.
            Self::ProviderByokEvent => "provider_byok_event",
            // J-32 (SCHEMA_VERSION 285): front-door intent routing decision.
            // Flat variant, Kind rides in the payload.
            Self::IntentRouted => "intent_routed",
            // J-34 (SCHEMA_VERSION 286): empirical context calibration event.
            // Flat variant, Kind rides in the payload.
            Self::ContextCalibrationEvent(_) => "context_calibration_event",
            // J-36 (SCHEMA_VERSION 287): local prefix-KV cache event.
            // Flat variant, Kind rides in the payload.
            Self::LocalPrefixCacheEvent(_) => "local_prefix_cache_event",
            // J-24 (SCHEMA_VERSION 288): behavior steering event.
            // Flat variant, Kind rides in the payload.
            Self::BehaviorSteeringEvent(_) => "behavior_steering_event",
            // J-35 intent-control layer: flat variant, Kind rides in the payload.
            Self::IntentControlEvent(_) => "intent_control_event",
            // J-39 T2: node admission gate event. Flat variant, Kind rides in the payload.
            Self::NodeAdmissionEvent(_) => "node_admission_event",
            Self::SystemContractEvent(_) => "system_contract_event",
            Self::HarnessInterventionEvent(_) => "harness_intervention_event",
            Self::FreePoolEvent => "free_pool_event",
            Self::RetrievalAbstained => "retrieval_abstained",
            Self::ContextAccessSpec => "context_access_spec",
            Self::DagCreditAssigned => "dag_credit_assigned",
            // I-30 Slice 1a: SFT exhaust event (flat; Kind in payload).
            Self::SftExhaustEvent => "sft_exhaust_event",
            Self::ReviewGateCatalogEvent => "review_gate_catalog_event",
            Self::ClaimRiskScored => "claim_risk_scored",
            // J-42 T2: reasoning checkpoint. Flat variant, Kind rides in the payload.
            Self::ReasoningCheckpointEvent(_) => "reasoning_checkpoint_event",
        }
    }
}

/// Public wrapper around the internal string-to-enum map.
/// Used by read-side consumers (e.g. [`crate::ssb_viewer::SsbViewer`]).
pub fn ssb_action_type_from_str_public(s: &str) -> Result<SsbActionType, SsbError> {
    ssb_action_type_from_str(s)
}

/// C-39: public helper for decoding SSB payload bytes (msgpack) into a
/// typed [`SsbPayload`]. Used by the CLI `--export-ssb` JSONL sink and
/// any external consumer that subscribes to the broadcast and wants to
/// render the typed payload as readable JSON without depending on the
/// orchestrator-private rmp_serde version. Returns Err on decode failure
/// so callers can fall back to an opaque representation.
pub fn decode_ssb_payload(bytes: &[u8]) -> Result<SsbPayload, SsbError> {
    rmp_serde::from_slice::<SsbPayload>(bytes).map_err(SsbError::from)
}

pub(crate) fn ssb_action_type_from_str(s: &str) -> Result<SsbActionType, SsbError> {
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
        // B-30 (SCHEMA_VERSION 36): language grammar registry status.
        "language_grammar_status" => Ok(SsbActionType::LanguageGrammarStatus),
        // B-29 Phase 5 (SCHEMA_VERSION 37): symbol-graph tool call row.
        "symbol_query" => Ok(SsbActionType::SymbolQuery),
        // B-32 (SCHEMA_VERSION 38): minified-file chunking SSB row.
        "minified_file_chunked" => Ok(SsbActionType::MinifiedFileChunked),
        // B-31.5 (SCHEMA_VERSION 39): 5 deferred B-31 SSB variants.
        "index_checkpoint" => Ok(SsbActionType::IndexCheckpoint),
        "index_resume" => Ok(SsbActionType::IndexResume),
        "index_cancel" => Ok(SsbActionType::IndexCancel),
        "lock_acquired" => Ok(SsbActionType::LockAcquired),
        "lock_contested" => Ok(SsbActionType::LockContested),
        // B-33 follow-up (SCHEMA_VERSION 40): symbol-graph build event row.
        "symbol_graph_updated" => Ok(SsbActionType::SymbolGraphUpdated),
        // B-40 (SCHEMA_VERSION 41): FRAMING substate beacons.
        "idea_framing_started" => Ok(SsbActionType::IdeaFramingStarted),
        "project_class_classified" => Ok(SsbActionType::ProjectClassClassified),
        "scope_ceiling_advisory" => Ok(SsbActionType::ScopeCeilingAdvisory),
        "critic_round_complete" => Ok(SsbActionType::CriticRoundComplete),
        "idea_framed" => Ok(SsbActionType::IdeaFramed),
        "tradeoff_card_emitted" => Ok(SsbActionType::TradeoffCardEmitted),
        // B-34 (SCHEMA_VERSION 42): entry-point detection + artifact pipeline beacons.
        "entry_points_detected" => Ok(SsbActionType::EntryPointsDetected),
        "artifacts_indexed"     => Ok(SsbActionType::ArtifactsIndexed),
        "artifact_searched"     => Ok(SsbActionType::ArtifactSearched),
        "artifact_stale"        => Ok(SsbActionType::ArtifactStale),
        // B-38 (SCHEMA_VERSION 43): ambiguity gate beacons.
        "ambiguity_scored"            => Ok(SsbActionType::AmbiguityScored),
        "ambiguity_floor_failed"      => Ok(SsbActionType::AmbiguityFloorFailed),
        "ambiguity_streak_advanced"   => Ok(SsbActionType::AmbiguityStreakAdvanced),
        // B-42 (SCHEMA_VERSION 44): cosmos tier knowledge worker beacons.
        "cosmos_refresh_started"  => Ok(SsbActionType::CosmosRefreshStarted),
        "cosmos_artifact_updated" => Ok(SsbActionType::CosmosArtifactUpdated),
        "cosmos_artifact_decayed" => Ok(SsbActionType::CosmosArtifactDecayed),
        "cosmos_grounding_used"   => Ok(SsbActionType::CosmosGroundingUsed),
        // B-41 (SCHEMA_VERSION 45): per-level auditor mode beacons.
        "per_level_audit_started" => Ok(SsbActionType::PerLevelAuditStarted),
        "per_level_audit_verdict" => Ok(SsbActionType::PerLevelAuditVerdict),
        "per_level_audit_cascade" => Ok(SsbActionType::PerLevelAuditCascade),
        // B-37C (SCHEMA_VERSION 46): multi-runtime backend beacons.
        "runtime_fallback"        => Ok(SsbActionType::RuntimeFallback),
        "runtime_health_changed"  => Ok(SsbActionType::RuntimeHealthChanged),
        "cli_subprocess_spawned"  => Ok(SsbActionType::CliSubprocessSpawned),
        "cli_subprocess_killed"   => Ok(SsbActionType::CliSubprocessKilled),
        // E-51 (SCHEMA_VERSION 96): edge-inference beacons.
        "mlx_subprocess_spawned"      => Ok(SsbActionType::MlxSubprocessSpawned),
        "mlx_subprocess_killed"       => Ok(SsbActionType::MlxSubprocessKilled),
        "llamacpp_subprocess_spawned" => Ok(SsbActionType::LlamaCppSubprocessSpawned),
        "llamacpp_subprocess_killed"  => Ok(SsbActionType::LlamaCppSubprocessKilled),
        "mlx_apple_platform_detected" => Ok(SsbActionType::MlxApplePlatformDetected),
        "llamacpp_ep_detected"        => Ok(SsbActionType::LlamaCppEpDetected),
        "runtime_chain_built"         => Ok(SsbActionType::RuntimeChainBuilt),
        // E-17 SP2 (SCHEMA_VERSION 97): RAG retrieval-quality beacons.
        "deep_pool_capped"            => Ok(SsbActionType::DeepPoolCapped),
        "rrf_quality_gate_fired"      => Ok(SsbActionType::RrfQualityGateFired),
        "hyde_skipped_code_query"     => Ok(SsbActionType::HydeSkippedCodeQuery),
        "rrf_component_weights"       => Ok(SsbActionType::RrfComponentWeights),
        // C-1 (SCHEMA_VERSION 47): knowledge + reasoning producer beacons.
        "grc_step_emitted"              => Ok(SsbActionType::GrcStepEmitted),
        "grounding_sweep_result"        => Ok(SsbActionType::GroundingSweepResult),
        "csq_evaluated"                 => Ok(SsbActionType::CsqEvaluated),
        "auditor_rubber_stamp_rejected" => Ok(SsbActionType::AuditorRubberStampRejected),
        "per_role_decoding_applied"     => Ok(SsbActionType::PerRoleDecodingApplied),
        "hyde_query"                    => Ok(SsbActionType::HydeQuery),
        "vector_cache_hit_rate"         => Ok(SsbActionType::VectorCacheHitRate),
        "query_trace"                   => Ok(SsbActionType::QueryTrace),
        // C-2 (SCHEMA_VERSION 48): orchestration + cosmos + stagnation beacons.
        "phase_node_ready"               => Ok(SsbActionType::PhaseNodeReady),
        "step_node_ready"                => Ok(SsbActionType::StepNodeReady),
        "baby_step_ready"                => Ok(SsbActionType::BabyStepReady),
        "dag_repair_escalated"           => Ok(SsbActionType::DagRepairEscalated),
        "agent_status_changed"           => Ok(SsbActionType::AgentStatusChanged),
        "stagnation_spinning_detected"   => Ok(SsbActionType::StagnationSpinningDetected),
        "stagnation_oscillation_detected"=> Ok(SsbActionType::StagnationOscillationDetected),
        "lateral_thinking_activated"     => Ok(SsbActionType::LateralThinkingActivated),
        "goal_drift_warning"             => Ok(SsbActionType::GoalDriftWarning),
        "watchdog_timeout"               => Ok(SsbActionType::WatchdogTimeout),
        // C-7 (SCHEMA_VERSION 49): reviewer verdict beacon.
        "change_decision"                => Ok(SsbActionType::ChangeDecision),
        // C-8 (SCHEMA_VERSION 50): search-mode cross-process bridge.
        "search_mode_changed"            => Ok(SsbActionType::SearchModeChanged),
        // C-5 (SCHEMA_VERSION 51): privacy-redaction event.
        "redaction_applied"              => Ok(SsbActionType::RedactionApplied),
        // C-10 (SCHEMA_VERSION 52): anti-spiral lateral-thinking +
        // goal-drift Block beacons.
        "lateral_thinking_succeeded"     => Ok(SsbActionType::LateralThinkingSucceeded),
        "lateral_thinking_exhausted"     => Ok(SsbActionType::LateralThinkingExhausted),
        "goal_drift_block"               => Ok(SsbActionType::GoalDriftBlock),
        // C-13 (SCHEMA_VERSION 53): IPC-dispatcher production-swap beacon.
        "wip_squash_complete"            => Ok(SsbActionType::WipSquashComplete),
        // C-22 (SCHEMA_VERSION 54): feasibility-probe completeness beacons.
        "alternative_explored"           => Ok(SsbActionType::AlternativeExplored),
        "feasibility_escalated"          => Ok(SsbActionType::FeasibilityEscalated),
        // C-23 (SCHEMA_VERSION 55): SessionDag branch-UI beacons.
        "branch_forked"                  => Ok(SsbActionType::BranchForked),
        "branch_merged"                  => Ok(SsbActionType::BranchMerged),
        "branch_closed"                  => Ok(SsbActionType::BranchClosed),
        // C-25 (SCHEMA_VERSION 56): compiled-pipeline + template-crystallizer
        // + background-task beacons.
        "plan_skeleton"                  => Ok(SsbActionType::PlanSkeleton),
        "compiled_pipeline_fired"        => Ok(SsbActionType::CompiledPipelineFired),
        "template_crystallized"          => Ok(SsbActionType::TemplateCrystallized),
        "background_task_started"        => Ok(SsbActionType::BackgroundTaskStarted),
        "background_task_complete"       => Ok(SsbActionType::BackgroundTaskComplete),
        "decompile_triggered"            => Ok(SsbActionType::DecompileTriggered),
        // C-26 (SCHEMA_VERSION 58): constrained-decoding + futility primitives.
        "futility_detected"              => Ok(SsbActionType::FutilityDetected),
        "futility_output_stagnation"     => Ok(SsbActionType::FutilityOutputStagnation),
        // C-27 (SCHEMA_VERSION 59): Testing-trust T(-1) + T-0 beacons.
        "wiring_evidence"                => Ok(SsbActionType::WiringEvidence),
        "wiring_violation"               => Ok(SsbActionType::WiringViolation),
        "determinism_drift"              => Ok(SsbActionType::DeterminismDrift),
        // C-28 (SCHEMA_VERSION 60): Testing-trust T-1 mutation testing.
        "mutation_survived"              => Ok(SsbActionType::MutationSurvived),
        // C-29 (SCHEMA_VERSION 61): Testing-trust T-2 + T-3 beacons.
        "intent_confirmed"               => Ok(SsbActionType::IntentConfirmed),
        "mr_violated"                    => Ok(SsbActionType::MrViolated),
        // C-30 (SCHEMA_VERSION 62): Surgical Mode + Oracle Mode beacons.
        "surgical_plan_created"          => Ok(SsbActionType::SurgicalPlanCreated),
        "surgical_loc_warning"           => Ok(SsbActionType::SurgicalLocWarning),
        "oracle_cache_hit"               => Ok(SsbActionType::OracleCacheHit),
        "oracle_cache_miss"              => Ok(SsbActionType::OracleCacheMiss),
        "project_detection_signal"       => Ok(SsbActionType::ProjectDetectionSignal),
        // C-32 (SCHEMA_VERSION 63): Code-Completeness Layer 2 beacons.
        "skeleton_generated"             => Ok(SsbActionType::SkeletonGenerated),
        "skeleton_flushed"               => Ok(SsbActionType::SkeletonFlushed),
        "mini_critic_verdict"            => Ok(SsbActionType::MiniCriticVerdict),
        "structural_retry_stage_attempt" => Ok(SsbActionType::StructuralRetryStageAttempt),
        "bridge_file_generated"          => Ok(SsbActionType::BridgeFileGenerated),
        // D-1 (SCHEMA_VERSION 64): ReasoningTemplate ArchiveChain lifecycle.
        "archive_entry_added"            => Ok(SsbActionType::ArchiveEntryAdded),
        "archive_entry_retired"          => Ok(SsbActionType::ArchiveEntryRetired),
        "template_promoted"              => Ok(SsbActionType::TemplatePromoted),
        "template_candidate_rejected"    => Ok(SsbActionType::TemplateCandidateRejected),
        // E-5 (SCHEMA_VERSION 65): memory_facts bi-temporal lifecycle beacons.
        "memory_fact_extracted"          => Ok(SsbActionType::MemoryFactExtracted),
        "memory_fact_superseded"         => Ok(SsbActionType::MemoryFactSuperseded),
        // D-2 (SCHEMA_VERSION 66): PPM verifier beacons.
        "ppm_scored"                     => Ok(SsbActionType::PpmScored),
        "ppm_gating_failed"              => Ok(SsbActionType::PpmGatingFailed),
        // E-1 (SCHEMA_VERSION 67): infinite-context chat illusion beacons.
        "chat_reflection_completed"      => Ok(SsbActionType::ChatReflectionCompleted),
        "memory_evolution_run"           => Ok(SsbActionType::MemoryEvolutionRun),
        // E-1 dashboard slice (SCHEMA_VERSION 68): MemoryNotes / CoreBlock
        // producer + conflict beacons.
        "memory_note_inserted"           => Ok(SsbActionType::MemoryNoteInserted),
        "memory_note_evolved"            => Ok(SsbActionType::MemoryNoteEvolved),
        "memory_conflict_detected"       => Ok(SsbActionType::MemoryConflictDetected),
        "core_block_replaced"            => Ok(SsbActionType::CoreBlockReplaced),
        // D-3 (SCHEMA_VERSION 69): multi-verifier consensus aggregator.
        "verifier_consensus_pass"        => Ok(SsbActionType::VerifierConsensusPass),
        "verifier_consensus_fail"        => Ok(SsbActionType::VerifierConsensusFail),
        // D-4 (SCHEMA_VERSION 70): tripwire stack beacons.
        "tripwire_fired"                 => Ok(SsbActionType::TripwireFired),
        "template_auto_reverted"         => Ok(SsbActionType::TemplateAutoReverted),
        // D-4 GAP-2 (SCHEMA_VERSION 108): concurrent-detector tripwire row.
        "tripwire_fired_concurrent"      => Ok(SsbActionType::TripwireFiredConcurrent),
        // D-19 (SCHEMA_VERSION 71): PRMBench adversarial-gaming benchmark beacon.
        "prmbench_run_completed"         => Ok(SsbActionType::PrmBenchRunCompleted),
        // D-5 (SCHEMA_VERSION 72): BAI Thompson sampling promotion round.
        "bai_arm_updated"                => Ok(SsbActionType::BaiArmUpdated),
        // D-5 closure (SCHEMA_VERSION 208): unified promotion-decision summary.
        "bai_promotion_decision"         => Ok(SsbActionType::BaiPromotionDecision),
        // D-6 (SCHEMA_VERSION 73): held-out task store lifecycle.
        "held_out_task_added"            => Ok(SsbActionType::HeldOutTaskAdded),
        "held_out_task_used"             => Ok(SsbActionType::HeldOutTaskUsed),
        // D-7 (SCHEMA_VERSION 74): cross-domain transfer canary beacons.
        "canary_run_completed"           => Ok(SsbActionType::CanaryRunCompleted),
        "canary_family_evaluated"        => Ok(SsbActionType::CanaryFamilyEvaluated),
        "locked_keys_audit_run"          => Ok(SsbActionType::LockedKeysAuditRun),
        "locked_keys_violation_detected" => Ok(SsbActionType::LockedKeysViolationDetected),
        // E-46 (SCHEMA_VERSION 76): hierarchical decomposition tuning beacons.
        "task_advanced"                  => Ok(SsbActionType::TaskAdvanced),
        "agentless_shortcut_taken"       => Ok(SsbActionType::AgentlessShortcutTaken),
        "goal_drift_fired"               => Ok(SsbActionType::GoalDriftFired),
        // D-10 (SCHEMA_VERSION 77): mode-routing predicate beacon.
        "claim_routed"                   => Ok(SsbActionType::ClaimRouted),
        // D-21 (SCHEMA_VERSION 78): mode-routing coverage benchmark beacon.
        "mode_routing_bench_run_completed" => Ok(SsbActionType::ModeRoutingBenchRunCompleted),
        // D-11 (SCHEMA_VERSION 79): constructive-checker registry beacon.
        "checker_result_emitted"         => Ok(SsbActionType::CheckerResultEmitted),
        // D-13 (SCHEMA_VERSION 80): adversarial-verification beacons.
        "adversary_generated"            => Ok(SsbActionType::AdversaryGenerated),
        "consensus_reevaluated_with_counter" => {
            Ok(SsbActionType::ConsensusReevaluatedWithCounter)
        }
        // D-12 (SCHEMA_VERSION 81): OperationalizationCheck beacon.
        "baby_step_dispatched" => Ok(SsbActionType::BabyStepDispatched),
        // D-16 (SCHEMA_VERSION 82): UniCR conformal-refusal beacons.
        "conformal_tau_computed" => Ok(SsbActionType::ConformalTauComputed),
        "conformal_refusal_decided" => Ok(SsbActionType::ConformalRefusalDecided),
        // D-15 (SCHEMA_VERSION 83): discovery-vs-search NoveltyClass beacon.
        "novelty_classified" => Ok(SsbActionType::NoveltyClassified),
        // D-18 (SCHEMA_VERSION 84): typed-ObservableEffect predicate beacon.
        "observable_effect_verified" => Ok(SsbActionType::ObservableEffectVerified),
        // D-17 (SCHEMA_VERSION 85): honest-refusal SLA monitoring beacons.
        "refusal_rate_snapshot" => Ok(SsbActionType::RefusalRateSnapshot),
        "refusal_sla_breach" => Ok(SsbActionType::RefusalSlaBreach),
        // D-22 (SCHEMA_VERSION 86): conformal-calibration validation beacon.
        "conformal_validation_completed" => Ok(SsbActionType::ConformalValidationCompleted),
        // D-20 (SCHEMA_VERSION 87): ARC-AGI-2 OOD probe gating beacon.
        "arc_benchmark_completed" => Ok(SsbActionType::ArcBenchmarkCompleted),
        "tool_gated" => Ok(SsbActionType::ToolGated),
        "tool_eager" => Ok(SsbActionType::ToolEager),
        "tool_lazy_resolved" => Ok(SsbActionType::ToolLazyResolved),
        "tool_denied" => Ok(SsbActionType::ToolDenied),
        "quant_backend_selected" => Ok(SsbActionType::QuantBackendSelected),
        "kv_cache_dtype_software_sim" => Ok(SsbActionType::KvCacheDtypeSoftwareSim),
        "awq_checkpoint_built" => Ok(SsbActionType::AwqCheckpointBuilt),
        "cli_mcp_dispatch" => Ok(SsbActionType::CliMcpDispatch),
        "cli_mcp_server_started" => Ok(SsbActionType::CliMcpServerStarted),
        "cli_mcp_server_stopped" => Ok(SsbActionType::CliMcpServerStopped),
        "firecracker_restore_slow" => Ok(SsbActionType::FirecrackerRestoreSlow),
        "snapshot_uniqueness_regenerated" => Ok(SsbActionType::SnapshotUniquenessRegenerated),
        "snapshot_evicted" => Ok(SsbActionType::SnapshotEvicted),
        "sandbox_backend_selected" => Ok(SsbActionType::SandboxBackendSelected),
        "agent_forked_from_snapshot" => Ok(SsbActionType::AgentForkedFromSnapshot),
        "researcher_net_egress_blocked" => Ok(SsbActionType::ResearcherNetEgressBlocked),
        "mcp_schema_pin_violation" => Ok(SsbActionType::McpSchemaPinViolation),
        "mcp_sanitization_rule_applied" => Ok(SsbActionType::McpSanitizationRuleApplied),
        "mcp_budget_enforced" => Ok(SsbActionType::McpBudgetEnforced),
        "mcp_legacy_sse_transport" => Ok(SsbActionType::McpLegacySseTransport),
        "mcp_llm_guard_blocked" => Ok(SsbActionType::McpLlmGuardBlocked),
        "anthropic_gateway_detected" => Ok(SsbActionType::AnthropicGatewayDetected),
        "memory_tier_assigned" => Ok(SsbActionType::MemoryTierAssigned),
        "memory_query_filtered" => Ok(SsbActionType::MemoryQueryFiltered),
        "safety_critical_blocked" => Ok(SsbActionType::SafetyCriticalBlocked),
        "argv_destructive_denied" => Ok(SsbActionType::ArgvDestructiveDenied),
        "argv_approval_required" => Ok(SsbActionType::ArgvApprovalRequired),
        "trust_dialog_hook_deferred" => Ok(SsbActionType::TrustDialogHookDeferred),
        "trust_dialog_env_deferred" => Ok(SsbActionType::TrustDialogEnvDeferred),
        "mcp_pin_version_refused" => Ok(SsbActionType::McpPinVersionRefused),
        "auditor_lineage_collision" => Ok(SsbActionType::AuditorLineageCollision),
        "untrusted_memory_write_blocked" => Ok(SsbActionType::UntrustedMemoryWriteBlocked),
        "incident_record_created" => Ok(SsbActionType::IncidentRecordCreated),
        // E-32: context-window extension beacons
        "fitm_calibration_applied"    => Ok(SsbActionType::FitmCalibrationApplied),
        "deep_mode_activated"         => Ok(SsbActionType::DeepModeActivated),
        "deep_mode_escalation_chosen" => Ok(SsbActionType::DeepModeEscalationChosen),
        "yarn_refused"                => Ok(SsbActionType::YarnRefused),
        // B-3 (SCHEMA_VERSION 98): AntiStalenessDrawer + RulesDrawer beacons.
        "dep_audit_completed"         => Ok(SsbActionType::DepAuditCompleted),
        "baseline_run_completed"      => Ok(SsbActionType::BaselineRunCompleted),
        "deterministic_audit_run"     => Ok(SsbActionType::DeterministicAuditRun),
        "rule_transition"             => Ok(SsbActionType::RuleTransition),
        // B-4 (SCHEMA_VERSION 99): canary drift snapshot row.
        "canary_drift"                => Ok(SsbActionType::CanaryDrift),
        // B-7 (SCHEMA_VERSION 100): producer-side wiring closure batch.
        "tree_position_changed"       => Ok(SsbActionType::TreePositionChanged),
        "failure_recorded"            => Ok(SsbActionType::FailureRecorded),
        "package_ref_verified"        => Ok(SsbActionType::PackageRefVerified),
        "tech_stack_decided"          => Ok(SsbActionType::TechStackDecided),
        // B-19 (SCHEMA_VERSION 101): per-agent-execution DAIRA marker
        // aggregation row.
        "marker_report"               => Ok(SsbActionType::MarkerReport),
        // B-25 (SCHEMA_VERSION 102): Phase 4J Empire integration.
        "empire_connected"            => Ok(SsbActionType::EmpireConnected),
        "empire_disconnected"         => Ok(SsbActionType::EmpireDisconnected),
        "empire_event_received"       => Ok(SsbActionType::EmpireEventReceived),
        "empire_capability_invoked"   => Ok(SsbActionType::EmpireCapabilityInvoked),
        "empire_repair_proposed"      => Ok(SsbActionType::EmpireRepairProposed),
        // C-5 follow-on (SCHEMA_VERSION 103).
        "route_blocked"               => Ok(SsbActionType::RouteBlocked),
        "warn_limit_reached"          => Ok(SsbActionType::WarnLimitReached),
        "cost_estimate"               => Ok(SsbActionType::CostEstimate),
        // C-36 (SCHEMA_VERSION 104).
        "mucoco_mutation_applied"     => Ok(SsbActionType::MucocoMutationApplied),
        // D-3 closure (SCHEMA_VERSION 105).
        "ppm_unavailable"             => Ok(SsbActionType::PpmUnavailable),
        "verifier_disagreement"       => Ok(SsbActionType::VerifierDisagreement),
        "cross_context_dispatched"    => Ok(SsbActionType::CrossContextDispatched),
        // C-38 (SCHEMA_VERSION 107).
        "pagerank_run"                => Ok(SsbActionType::PagerankRun),
        "dkg_rebuild_started"         => Ok(SsbActionType::DkgRebuildStarted),
        "dkg_rebuild_complete"        => Ok(SsbActionType::DkgRebuildComplete),
        // C-40 (SCHEMA_VERSION 109).
        "github_pr_opened"            => Ok(SsbActionType::GithubPrOpened),
        "github_pr_status_fetched"    => Ok(SsbActionType::GithubPrStatusFetched),
        "github_pr_closed"            => Ok(SsbActionType::GithubPrClosed),
        "github_pr_merged"            => Ok(SsbActionType::GithubPrMerged),
        // E-41 (SCHEMA_VERSION 135): PR review surface.
        "pr_review_composed"          => Ok(SsbActionType::PrReviewComposed),
        "pr_comment_watched"          => Ok(SsbActionType::PrCommentWatched),
        "pr_package_verify_result"    => Ok(SsbActionType::PrPackageVerifyResult),
        "pr_semgrep_gate_result"      => Ok(SsbActionType::PrSemgrepGateResult),
        "pr_auto_merge_blocked"       => Ok(SsbActionType::PrAutoMergeBlocked),
        // D-5 closure (SCHEMA_VERSION 110): Thompson-sample beacon.
        "thompson_sampled"            => Ok(SsbActionType::ThompsonSampled),
        // C-41 (SCHEMA_VERSION 111): skills-driven workflow registry.
        "skill_triggered"             => Ok(SsbActionType::SkillTriggered),
        "skill_executed"              => Ok(SsbActionType::SkillExecuted),
        // D-6 closure (SCHEMA_VERSION 112): held-out promotion-gate +
        // refresh-event beacons.
        "held_out_promotion_gate"     => Ok(SsbActionType::HeldOutPromotionGate),
        "held_out_refresh"            => Ok(SsbActionType::HeldOutRefresh),
        // C-42 (SCHEMA_VERSION 113): Semantic Gateway L2 RBAC beacons.
        "gateway_policy_allowed"      => Ok(SsbActionType::GatewayPolicyAllowed),
        "gateway_policy_denied"       => Ok(SsbActionType::GatewayPolicyDenied),
        "gateway_hitl_pending"        => Ok(SsbActionType::GatewayHitlPending),
        "cross_domain_regression"     => Ok(SsbActionType::CrossDomainRegression),
        "canary_family_pass"          => Ok(SsbActionType::CanaryFamilyPass),
        "canary_insufficient_data"    => Ok(SsbActionType::CanaryInsufficientData),
        // D-8 closure (SCHEMA_VERSION 115): cryptographic archive verify.
        "archive_verify_result"       => Ok(SsbActionType::ArchiveVerifyResult),
        // C-43 closure (SCHEMA_VERSION 116): Phase 4H scope-freeze.
        "scope_freeze_ratified"       => Ok(SsbActionType::ScopeFreezeRatified),
        "scope_freeze_violated"       => Ok(SsbActionType::ScopeFreezeViolated),
        // C-44 (SCHEMA_VERSION 117): Composer 2 paid-backend cost accrual.
        "backend_cost_accrued"        => Ok(SsbActionType::BackendCostAccrued),
        // C-45 (SCHEMA_VERSION 120): CANTANTE credit attribution.
        "agent_role_dropped"          => Ok(SsbActionType::AgentRoleDropped),
        "credit_attribution_run"      => Ok(SsbActionType::CreditAttributionRun),
        "credit_attribution_result"   => Ok(SsbActionType::CreditAttributionResult),
        // C-46 (SCHEMA_VERSION 121): Uno-Orchestra selective delegation.
        "selective_delegation_decision"  => Ok(SsbActionType::SelectiveDelegationDecision),
        "selective_delegation_escalated" => Ok(SsbActionType::SelectiveDelegationEscalated),
        "selective_delegation_cost_saved" => Ok(SsbActionType::SelectiveDelegationCostSaved),
        // C-47 (SCHEMA_VERSION 122): Case-Based Calibration of Tool-Use.
        "tool_call_logged"                => Ok(SsbActionType::ToolCallLogged),
        "tool_calibration_hint_emitted"   => Ok(SsbActionType::ToolCalibrationHintEmitted),
        "tool_calibration_drift_detected" => Ok(SsbActionType::ToolCalibrationDriftDetected),
        // C-49 (SCHEMA_VERSION 123): iterative-distillation experience.
        "prompt_policy_distillation_run"  => Ok(SsbActionType::PromptPolicyDistillationRun),
        "prompt_fragment_selected"        => Ok(SsbActionType::PromptFragmentSelected),
        "prompt_fragment_discarded"       => Ok(SsbActionType::PromptFragmentDiscarded),
        // D-14 (SCHEMA_VERSION 124): sycophancy probe row pair.
        "sycophancy_probe_dispatched"     => Ok(SsbActionType::SycophancyProbeDispatched),
        "sycophancy_flag"                 => Ok(SsbActionType::SycophancyFlag),
        // D-23 (SCHEMA_VERSION 142): sycophancy detection benchmark beacon.
        "sycophancy_bench_run_completed"  => Ok(SsbActionType::SycophancyBenchRunCompleted),
        // D-24 (SCHEMA_VERSION 143): cross-domain canary regression benchmark.
        "canary_bench_run_completed"      => Ok(SsbActionType::CanaryBenchRunCompleted),
        // D-25 (SCHEMA_VERSION 144): determinism + archive tamper-suite benchmark.
        "determinism_tamper_suite_completed" => Ok(SsbActionType::DeterminismTamperSuiteCompleted),
        // D-28 (SCHEMA_VERSION 145): adversarial example farming run.
        "adversarial_farm_run_completed"  => Ok(SsbActionType::AdversarialFarmRunCompleted),
        // D-29 (SCHEMA_VERSION 146): hierarchical BAI round completion.
        "hierarchical_bai_round_completed" => Ok(SsbActionType::HierarchicalBaiRoundCompleted),
        // D-30 (SCHEMA_VERSION 147): calibration bootstrap warm-start.
        "calibration_bootstrap_completed" => Ok(SsbActionType::CalibrationBootstrapCompleted),
        // C-50 (SCHEMA_VERSION 125): Contract-Coding scaffolding pair.
        "contract_test_scaffolded"        => Ok(SsbActionType::ContractTestScaffolded),
        "contract_violation_detected"     => Ok(SsbActionType::ContractViolationDetected),
        // C-52 (SCHEMA_VERSION 126): Dafny Formal Verification Spike.
        "dafny_verify_completed"          => Ok(SsbActionType::DafnyVerifyCompleted),
        // D-16 closure (SCHEMA_VERSION 127): TauUpdater persistence beacon.
        "calibration_corpus_updated"      => Ok(SsbActionType::CalibrationCorpusUpdated),
        // E-12 Tier-B (SCHEMA_VERSION 128): production fullstack codegen.
        "framework_profile_selected"      => Ok(SsbActionType::FrameworkProfileSelected),
        "legacy_config_stripped"          => Ok(SsbActionType::LegacyConfigStripped),
        // D-18 closure (SCHEMA_VERSION 129): typed-ObservableEffect 4-variant pack.
        "observable_effect_dispatched"    => Ok(SsbActionType::ObservableEffectDispatched),
        "goodhart_risk_flag"              => Ok(SsbActionType::GoodhartRiskFlag),
        "sandbox_observation"             => Ok(SsbActionType::SandboxObservation),
        "production_observation_proposal" => Ok(SsbActionType::ProductionObservationProposal),
        // E-13 (SCHEMA_VERSION 130): TDD verifier-agent beacons.
        "tdd_cycle_started"               => Ok(SsbActionType::TddCycleStarted),
        "tdd_test_failed"                 => Ok(SsbActionType::TddTestFailed),
        "tdd_test_passed"                 => Ok(SsbActionType::TddTestPassed),
        // Tranche B-7 triple (SCHEMA_VERSION 195): TDD verification gate.
        "tdd_verification_event"          => Ok(SsbActionType::TddVerificationEvent),
        // E-14 (SCHEMA_VERSION 131): browser-e2e agent beacons.
        "browser_e2e_run_started"         => Ok(SsbActionType::BrowserE2eRunStarted),
        "browser_e2e_assertion_failed"    => Ok(SsbActionType::BrowserE2eAssertionFailed),
        "browser_e2e_assertion_passed"    => Ok(SsbActionType::BrowserE2eAssertionPassed),
        "browser_e2e_snapshot_captured"   => Ok(SsbActionType::BrowserE2eSnapshotCaptured),
        // E-18 (SCHEMA_VERSION 132): tool-use reliability gate beacons.
        "tool_reliability_gate_passed"    => Ok(SsbActionType::ToolReliabilityGatePassed),
        "tool_reliability_gate_rejected"  => Ok(SsbActionType::ToolReliabilityGateRejected),
        "tool_call_spilled"               => Ok(SsbActionType::ToolCallSpilled),
        "tool_call_retry_escalated"       => Ok(SsbActionType::ToolCallRetryEscalated),
        // E-37 (SCHEMA_VERSION 133): prompt-injection defense beacons.
        "injection_attempt_detected"      => Ok(SsbActionType::InjectionAttemptDetected),
        "injection_attempt_blocked"       => Ok(SsbActionType::InjectionAttemptBlocked),
        "spotlight_wrapped"               => Ok(SsbActionType::SpotlightWrapped),
        "spotlight_breached"              => Ok(SsbActionType::SpotlightBreached),
        "tool_description_pin_mismatch"   => Ok(SsbActionType::ToolDescriptionPinMismatch),
        "capability_gate_rejected"        => Ok(SsbActionType::CapabilityGateRejected),
        "provenance_tagged"               => Ok(SsbActionType::ProvenanceTagged),
        "prompt_armor_verdict"            => Ok(SsbActionType::PromptArmorVerdict),
        "injection_fixture_replayed"      => Ok(SsbActionType::InjectionFixtureReplayed),
        "output_classifier_flagged"       => Ok(SsbActionType::OutputClassifierFlagged),
        // E-40 (SCHEMA_VERSION 134): orchestrator-rooted symbol-graph beacons.
        "symbol_graph_queried"            => Ok(SsbActionType::SymbolGraphQueried),
        "symbol_graph_cache_loaded"       => Ok(SsbActionType::SymbolGraphCacheLoaded),
        "lsp_oracle_consulted"            => Ok(SsbActionType::LspOracleConsulted),
        // E-44 (SCHEMA_VERSION 136): durability layer beacons.
        "checkpoint_written"              => Ok(SsbActionType::CheckpointWritten),
        "checkpoint_resumed"              => Ok(SsbActionType::CheckpointResumed),
        "checkpoint_migrated"             => Ok(SsbActionType::CheckpointMigrated),
        "idempotent_dispatch_skipped"     => Ok(SsbActionType::IdempotentDispatchSkipped),
        "durability_lock_blocked"         => Ok(SsbActionType::DurabilityLockBlocked),
        // E-45 (SCHEMA_VERSION 137): cross-language drift detector beacons.
        "cross_lang_drift_detected"       => Ok(SsbActionType::CrossLangDriftDetected),
        "cross_lang_pair_scanned"         => Ok(SsbActionType::CrossLangPairScanned),
        "schema_version_mismatch"         => Ok(SsbActionType::SchemaVersionMismatch),
        // E-49 (SCHEMA_VERSION 138): coding-agent metric suite (scope-lean).
        "coding_agent_metrics_captured"   => Ok(SsbActionType::CodingAgentMetricsCaptured),
        "swe_bench_pro_run_started"       => Ok(SsbActionType::SweBenchProRunStarted),
        "swe_bench_pro_run_completed"     => Ok(SsbActionType::SweBenchProRunCompleted),
        "refinebench_score_emitted"       => Ok(SsbActionType::RefineBenchScoreEmitted),
        "metrics_aggregation_run"         => Ok(SsbActionType::MetricsAggregationRun),
        // E-52 Tier-B (SCHEMA_VERSION 139): LSP-as-verifier beacons.
        "diagnostic_probe_dispatched"     => Ok(SsbActionType::DiagnosticProbeDispatched),
        "diagnostic_probe_completed"      => Ok(SsbActionType::DiagnosticProbeCompleted),
        "diagnostic_probe_block"          => Ok(SsbActionType::DiagnosticProbeBlock),
        "scip_refresh_completed"          => Ok(SsbActionType::ScipRefreshCompleted),
        // E-53 (SCHEMA_VERSION 140): formal-verify pipeline boundary beacon.
        "formal_verifier_completed"       => Ok(SsbActionType::FormalVerifierCompleted),
        // E-35 (SCHEMA_VERSION 141): structured-output substrate beacons.
        "xgrammar_mask_applied"           => Ok(SsbActionType::XgrammarMaskApplied),
        "instructor_mode_activated"       => Ok(SsbActionType::InstructorModeActivated),
        "grammar_mode_activated"          => Ok(SsbActionType::GrammarModeActivated),
        "codeact_action_executed"         => Ok(SsbActionType::CodeactActionExecuted),
        "search_replace_tier1_hit"        => Ok(SsbActionType::SearchReplaceTier1Hit),
        "search_replace_collision_fallback" => Ok(SsbActionType::SearchReplaceCollisionFallback),
        "cloud_strict_schema_applied"     => Ok(SsbActionType::CloudStrictSchemaApplied),
        "grammar_cache_loaded"            => Ok(SsbActionType::GrammarCacheLoaded),
        "structured_engine_rejected"      => Ok(SsbActionType::StructuredEngineRejected),
        "legacy_json_object_refused"      => Ok(SsbActionType::LegacyJsonObjectRefused),
        // E-6 (SCHEMA_VERSION 148): MAST-grounded multi-agent failure-mode defenses.
        "audit_mast_tagged"               => Ok(SsbActionType::AuditMastTagged),
        "iso_budget_measured"             => Ok(SsbActionType::IsoBudgetMeasured),
        "persona_reanchored"              => Ok(SsbActionType::PersonaReanchored),
        "cross_family_unavailable"        => Ok(SsbActionType::CrossFamilyUnavailable),
        // E-7 (SCHEMA_VERSION 149): darwinian self-improvement substrate.
        "darwinian_generation_completed"  => Ok(SsbActionType::DarwinianGenerationCompleted),
        // E-3 (SCHEMA_VERSION 150): runtime handoff signal beacon.
        "runtime_handoff_signal"          => Ok(SsbActionType::RuntimeHandoffSignal),
        // E-26 (SCHEMA_VERSION 151): inference-engines 2026 pack.
        "sglang_backend_registered"       => Ok(SsbActionType::SglangBackendRegistered),
        "xgrammar_schema_compiled"        => Ok(SsbActionType::XgrammarSchemaCompiled),
        "runtime_cold_start_observed"     => Ok(SsbActionType::RuntimeColdStartObserved),
        // E-2 (SCHEMA_VERSION 152-153): streaming timing + UX pacing vote.
        "streaming_timing_recorded"       => Ok(SsbActionType::StreamingTimingRecorded),
        "ux_pacing_vote_recorded"         => Ok(SsbActionType::UxPacingVoteRecorded),
        // E-28 (SCHEMA_VERSION 154-156): attention-kernels 2026 pack.
        "streaming_llm_engaged"           => Ok(SsbActionType::StreamingLlmEngaged),
        "sdpa_backend_pinned"             => Ok(SsbActionType::SdpaBackendPinned),
        "yarn_rope_scaling_applied"       => Ok(SsbActionType::YarnRopeScalingApplied),
        // E-8 (SCHEMA_VERSION 162): self-evolving prompts substrate.
        "prompt_optimization_completed"   => Ok(SsbActionType::PromptOptimizationCompleted),
        // E-10 (SCHEMA_VERSION 163): hostile-critic structural simplifications.
        "critic_decision_event"           => Ok(SsbActionType::CriticDecisionEvent),
        // E-43 (SCHEMA_VERSION 164): agent skill systems telemetry.
        "skill_system_event"              => Ok(SsbActionType::SkillSystemEvent),
        "benchmark_run_observed"          => Ok(SsbActionType::BenchmarkRunObserved),
        // E-19 (SCHEMA_VERSION 166): reasoning quality probe.
        "reasoning_quality_probe"         => Ok(SsbActionType::ReasoningQualityProbe),
        // E-54 (SCHEMA_VERSION 167): eval-corpus repair pre-deploy.
        "eval_corpus_event"               => Ok(SsbActionType::EvalCorpusEvent),
        "reward_ladder_event"             => Ok(SsbActionType::RewardLadderEvent),
        // E-11 (SCHEMA_VERSION 169): SaaS prior-art catalog + recommender.
        "saas_prior_art_event"            => Ok(SsbActionType::SaasPriorArtEvent),
        // E-22 (SCHEMA_VERSION 170): may-2026 github trending sweep substrate.
        "github_trending_event"           => Ok(SsbActionType::GithubTrendingEvent),
        // E-23 (SCHEMA_VERSION 171): 2026 developer-discourse positioning + UX-guard.
        "developer_discourse_event"       => Ok(SsbActionType::DeveloperDiscourseEvent),
        // E-25 (SCHEMA_VERSION 172): claude-code-architecture catalog substrate.
        "claude_code_architecture_event"  => Ok(SsbActionType::ClaudeCodeArchitectureEvent),
        "agentic_observability_event"     => Ok(SsbActionType::AgenticObservabilityEvent),
        // E-31 (SCHEMA_VERSION 174): coding-model-leaderboard scraper substrate.
        "coding_model_leaderboard_event"  => Ok(SsbActionType::CodingModelLeaderboardEvent),
        // E-38 (SCHEMA_VERSION 175): 2026 AI roadmap signals + EU AI Act
        // compliance pre-gate substrate.
        "ai_roadmap_signal_event"         => Ok(SsbActionType::AiRoadmapSignalEvent),
        // E-39 (SCHEMA_VERSION 176): cost + token economics substrate.
        "cost_token_event"                => Ok(SsbActionType::CostTokenEvent),
        // E-50 (SCHEMA_VERSION 177): SaaS architecture pattern catalog
        // substrate.
        "saas_arch_pattern_event"         => Ok(SsbActionType::SaasArchPatternEvent),
        // E-4 (SCHEMA_VERSION 178): chat UX critique substrate.
        "chat_ux_critique_event"          => Ok(SsbActionType::ChatUxCritiqueEvent),
        "vlm_image_validation_event"      => Ok(SsbActionType::VlmImageValidationEvent),
        // E-20 (SCHEMA_VERSION 180): local-cloud routing substrate.
        "local_cloud_routing_event"       => Ok(SsbActionType::LocalCloudRoutingEvent),
        // E-21 (SCHEMA_VERSION 181): peer-systems harness substrate.
        "peer_systems_event"              => Ok(SsbActionType::PeerSystemsEvent),
        // E-36 (SCHEMA_VERSION 182): document + vision RAG substrate.
        "document_vision_rag_event"       => Ok(SsbActionType::DocumentVisionRagEvent),
        // E-42 (SCHEMA_VERSION 183): tree-sitter grammar health substrate.
        "tree_sitter_grammar_event"       => Ok(SsbActionType::TreeSitterGrammarEvent),
        // E-46 (SCHEMA_VERSION 184): planning-decomposition substrate.
        "planning_decomposition_event"    => Ok(SsbActionType::PlanningDecompositionEvent),
        // E-48 (SCHEMA_VERSION 185): web-research orchestrator hardening
        // substrate.
        "web_research_event"              => Ok(SsbActionType::WebResearchEvent),
        // E-44 (SCHEMA_VERSION 186): durability observability substrate.
        "durability_event"                => Ok(SsbActionType::DurabilityEvent),
        // E-31 (SCHEMA_VERSION 187): head_dim invariant verification substrate.
        "head_dim_invariant_verified"     => Ok(SsbActionType::HeadDimInvariantVerified),
        // E-3 (SCHEMA_VERSION 188): role-shown / model-hidden default
        // telemetry substrate.
        "role_shown_model_hidden"         => Ok(SsbActionType::RoleShownModelHidden),
        // E-49 (SCHEMA_VERSION 189): coding-agent metrics named-token substrate.
        "coding_metric_named_token"       => Ok(SsbActionType::CodingMetricNamedToken),
        // E-3 sub-phase 4A (SCHEMA_VERSION 190): runtime handoff event substrate.
        "e3_runtime_handoff_event"        => Ok(SsbActionType::E3RuntimeHandoffEvent),
        // E-31 sub-phase 6A (SCHEMA_VERSION 191): cloud-route decision event substrate.
        "route_decision_event"            => Ok(SsbActionType::RouteDecisionEvent),
        // E-47 (SCHEMA_VERSION 192): LoRA fine-tuning ship-gate substrate.
        "lora_ship_gate_event"            => Ok(SsbActionType::LoraShipGateEvent),
        // E-40 Tranche B-2 (SCHEMA_VERSION 193): SymbolGraph augmentation substrate.
        "symbol_graph_event"              => Ok(SsbActionType::SymbolGraphEvent),
        // Tranche B-6 quad (SCHEMA_VERSION 194): four new SSB substrates.
        "mcp_authz_event"                 => Ok(SsbActionType::McpAuthzEvent),
        "tool_calibration_event"          => Ok(SsbActionType::ToolCalibrationEvent),
        "reflect_budget_soft_warn"        => Ok(SsbActionType::ReflectBudgetSoftWarn),
        "contract_coding_event"           => Ok(SsbActionType::ContractCodingEvent),
        // F-1 / F-2a (SCHEMA_VERSION 196): vibe-coding security gate variants.
        "vibesec_code_vuln_blocked"        => Ok(SsbActionType::VibeGateBlocked(VibeGateBlockedKind::CodeVuln)),
        "vibesec_secret_blocked"           => Ok(SsbActionType::VibeGateBlocked(VibeGateBlockedKind::Secret)),
        "vibesec_dep_hallucinated_blocked" => Ok(SsbActionType::VibeGateBlocked(VibeGateBlockedKind::DepHallucinated)),
        "vibesec_skill_blocked"            => Ok(SsbActionType::VibeGateBlocked(VibeGateBlockedKind::Skill)),
        "security_rule_blocked"            => Ok(SsbActionType::SecurityRuleBlocked),
        // F-4 / Slice B-1 (SCHEMA_VERSION 197): TestIntegrityEvent Pattern-2.
        "test_integrity_verified" => Ok(SsbActionType::TestIntegrityEvent(TestIntegrityEventKind::Verified)),
        "anti_fabrication_block"  => Ok(SsbActionType::TestIntegrityEvent(TestIntegrityEventKind::FabricationBlocked)),
        // F-3p1 / Slice C1 (SCHEMA_VERSION 198): ToolFilterEvent Pattern-2.
        "tool_filter_committed"       => Ok(SsbActionType::ToolFilterEvent(ToolFilterEventKind::Committed)),
        "untrusted_var_quarantined"   => Ok(SsbActionType::ToolFilterEvent(ToolFilterEventKind::UntrustedQuarantined)),
        // F-9 / Slice D1 (SCHEMA_VERSION 199): DepWriteTimeGate slopsquat blocked.
        "write_time_slopsquat_blocked" => Ok(SsbActionType::WriteTimeSlopsquatBlocked),
        // F-10 / Slice A1 (SCHEMA_VERSION 200): security-monotonicity guard.
        "security_class_regression_blocked" => Ok(SsbActionType::SecurityClassRegressionBlocked),
        // F-2b / Slice A1 (SCHEMA_VERSION 201): insecure-rate keystone bench beacon.
        "insecure_rate_measured" => Ok(SsbActionType::InsecureRateMeasured),
        // I-9 (SCHEMA_VERSION 263): error-amplification (alpha_t) bench beacon.
        "error_amplification_measured" => Ok(SsbActionType::ErrorAmplificationMeasured),
        // F-6 / Slice A1 (SCHEMA_VERSION 202): SkillLibraryEvent Pattern-2 (3 arms).
        "skill_acquired"        => Ok(SsbActionType::SkillLibraryEvent(SkillLibraryEventKind::Acquired)),
        "skill_trust_promoted"  => Ok(SsbActionType::SkillLibraryEvent(SkillLibraryEventKind::TrustPromoted)),
        "skill_demoted_stale"   => Ok(SsbActionType::SkillLibraryEvent(SkillLibraryEventKind::DemotedStale)),
        // F-5 / Slice A1 (SCHEMA_VERSION 203): StalenessQuarantineBroadcast flat beacon.
        "staleness_quarantine_broadcast" => Ok(SsbActionType::StalenessQuarantineBroadcast),
        // F-8 / Slice A1 (SCHEMA_VERSION 204): SymbolicConflictDetected flat beacon.
        "symbolic_conflict_detected" => Ok(SsbActionType::SymbolicConflictDetected),
        // F-7 / Slice A1 (SCHEMA_VERSION 205): TemplateUtilityRanked flat beacon.
        "template_utility_ranked" => Ok(SsbActionType::TemplateUtilityRanked),
        // F-13 / Slice A1 (SCHEMA_VERSION 206): SessionArchived flat beacon.
        "session_archived" => Ok(SsbActionType::SessionArchived),
        // F-11 / Slice A1 (SCHEMA_VERSION 207): DarwinianAdmit config-DGM
        // admit-loop beacon (Pattern-2; Kind on payload).
        "darwinian_admit" => Ok(SsbActionType::DarwinianAdmit),
        // G-plan variants.
        "mcp_rug_pull_blocked" => Ok(SsbActionType::McpRugPullBlocked),
        "mcp_scan_classifier_blocked" => Ok(SsbActionType::McpScanClassifierBlocked),
        "taint_flow" => Ok(SsbActionType::TaintFlow),
        "blast_radius_blocked" => Ok(SsbActionType::BlastRadiusBlocked),
        "trifecta_quarantined" => Ok(SsbActionType::TrifectaQuarantined),
        "run_artifact_emitted" => Ok(SsbActionType::RunArtifactEmitted),
        "review_surface_scored" => Ok(SsbActionType::ReviewSurfaceScored),
        "provenance_attestation_emitted" => Ok(SsbActionType::ProvenanceAttestationEmitted),
        "safety_delta_measured" => Ok(SsbActionType::SafetyDeltaMeasured),
        "model_drift_suspected" => Ok(SsbActionType::ModelDriftSuspected),
        "secret_redacted" => Ok(SsbActionType::SecretRedacted),
        "claim_confidence_scored" => Ok(SsbActionType::ClaimConfidenceScored),
        "steering_vector_refresh_triggered" => Ok(SsbActionType::SteeringVectorRefreshTriggered),
        "canvas_op_applied" => Ok(SsbActionType::CanvasOpApplied),
        "canvas_view_event" => Ok(SsbActionType::CanvasViewEvent),
        "canvas_activity_event" => Ok(SsbActionType::CanvasActivityEvent),
        "batch_lifecycle" => Ok(SsbActionType::BatchLifecycle),
        // H-plan verifiability variants.
        "deflation_gate_evaluated" => Ok(SsbActionType::DeflationGateEvaluated),
        "leakage_blocked" => Ok(SsbActionType::LeakageBlocked),
        "lookahead_differential" => Ok(SsbActionType::LookaheadDifferential),
        "trading_null_test" => Ok(SsbActionType::TradingNullTest),
        "rerun_hash_equality" => Ok(SsbActionType::RerunHashEquality),
        "sim_real_parity" => Ok(SsbActionType::SimRealParity),
        "result_manifest_verified" => Ok(SsbActionType::ResultManifestVerified),
        "differential_divergence" => Ok(SsbActionType::DifferentialDivergence),
        "gate_self_validation" => Ok(SsbActionType::GateSelfValidation),
        "reward_tampering" => Ok(SsbActionType::RewardTampering),
        "holdout_budget_event" => Ok(SsbActionType::HoldoutBudgetEvent),
        "drift_interlock" => Ok(SsbActionType::DriftInterlock),
        "trial_registered" => Ok(SsbActionType::TrialRegistered),
        "mucoco_inconsistency_detected" => Ok(SsbActionType::MucocoInconsistencyDetected),
        "mucoco_skip_reason" => Ok(SsbActionType::MucocoSkipReason),
        "user_decision_superseded" => Ok(SsbActionType::UserDecisionSuperseded),
        "staleness_eval_measured" => Ok(SsbActionType::StalenessEvalMeasured),
        "discovery_ratified" => Ok(SsbActionType::DiscoveryRatified),
        "discovery_rejected" => Ok(SsbActionType::DiscoveryRejected),
        "fact_validation_failed" => Ok(SsbActionType::FactValidationFailed),
        "memory_governance_event" => Ok(SsbActionType::MemoryGovernanceEvent),
        "mcp_tool_deferred" => Ok(SsbActionType::McpToolDeferred),
        "mcp_schema_budget_measured" => Ok(SsbActionType::McpSchemaBudgetMeasured),
        "session_rewound" => Ok(SsbActionType::SessionRewound),
        "agent_notification_emitted" => Ok(SsbActionType::AgentNotificationEmitted),
        "plan_decision_event" => Ok(SsbActionType::PlanDecisionEvent),
        "stop_confirmation_event" => Ok(SsbActionType::StopConfirmationEvent),
        "abstention_scored" => Ok(SsbActionType::AbstentionScored),
        "checker_gap_detected" => Ok(SsbActionType::CheckerGapDetected),
        "reasoning_budget_scaled" => Ok(SsbActionType::ReasoningBudgetScaled),
        "fitness_evaluated" => Ok(SsbActionType::FitnessEvaluated),
        "trap_behavior_detected" => Ok(SsbActionType::TrapBehaviorDetected),
        "promotion_score_aggregated" => Ok(SsbActionType::PromotionScoreAggregated),
        "clade_selected" => Ok(SsbActionType::CladeSelected),
        "eval_firewall_event" => Ok(SsbActionType::EvalFirewallEvent),
        "edit_history_event" => Ok(SsbActionType::EditHistoryEvent),
        "handoff_lifecycle_event" => Ok(SsbActionType::HandoffLifecycleEvent),
        "finding_anchored" => Ok(SsbActionType::FindingAnchored),
        "citation_anchors_bound" => Ok(SsbActionType::CitationAnchorsBound),
        "plan_state_derived"     => Ok(SsbActionType::PlanStateDerived),
        "objective_sealed"       => Ok(SsbActionType::ObjectiveSealed(ObjectiveSealedKind::Sealed)),
        "grammar_gate_event"     => Ok(SsbActionType::GrammarGateEvent(GrammarGateEventKind::CranePassed)),
        "qdelta_measured"        => Ok(SsbActionType::QdeltaMeasured),
        "snowballing_measured"  => Ok(SsbActionType::SnowballingMeasured),
        "kv_cache_tuned"        => Ok(SsbActionType::KvCacheTuned),
        "claim_scope_measured"   => Ok(SsbActionType::ClaimScopeMeasured(ClaimScopeMeasuredKind::PilotDelta)),
        "cache_economics_event"  => Ok(SsbActionType::CacheEconomicsEvent(CacheEconomicsEventKind::TtlPlanned)),
        "trajectory_aborted"          => Ok(SsbActionType::TrajectoryAborted(TrajectoryAbortedKind::Voluntary)),
        "directive_executed"          => Ok(SsbActionType::DirectiveExecuted(DirectiveExecutedKind::Executed)),
        "governance_primitive_fired"  => Ok(SsbActionType::GovernancePrimitiveFired(GovernancePrimitiveFiredKind::CeflExternalized)),
        "role_contract_applied"       => Ok(SsbActionType::RoleContractApplied(RoleContractAppliedKind::PersonaStripped)),
        "tool_sequence_verdict"       => Ok(SsbActionType::ToolSequenceVerdict(ToolSequenceVerdictKind::Allowed)),
        "critique_lens_finding"       => Ok(SsbActionType::CritiqueLensFinding(CritiqueLensFindingTag::AdvisoryFinding)),
        "bash_redirected"             => Ok(SsbActionType::BashRedirected(BashRedirectedKind::Redirected)),
        "prompt_hardening_applied"    => Ok(SsbActionType::PromptHardeningApplied(PromptHardeningAppliedKind::InputsTagged)),
        "branch_workspace_event"      => Ok(SsbActionType::BranchWorkspaceEvent(BranchWorkspaceEventKind::Spawned)),
        "context_pruned"              => Ok(SsbActionType::ContextPruned(ContextPrunedKind::SiblingExcluded)),
        "belief_state_event"          => Ok(SsbActionType::BeliefStateEvent(BeliefStateEventKind::Condensed)),
        "memory_guard_event"          => Ok(SsbActionType::MemoryGuardEvent(MemoryGuardEventKind::PoisonBlocked)),
        "context_transform_receipt"   => Ok(SsbActionType::ContextTransformReceipt(ContextTransformKind::LspSlice)),
        "playbook_event"              => Ok(SsbActionType::PlaybookEvent(PlaybookEventKind::Promoted)),
        "tri_layer_verdict"           => Ok(SsbActionType::TriLayerVerdict(TriLayerVerdictKind::HardVeto)),
        "row_chain_sealed"            => Ok(SsbActionType::RowChainSealed(RowChainSealedKind::ChainAdvanced)),
        // I-19 (SCHEMA_VERSION 254): TddPreflightBlocked Pattern-2 3-arm. Representative arm.
        "tdd_preflight_missing_test_delta" => Ok(SsbActionType::TddPreflightBlocked(TddPreflightBlockedKind::MissingTestDelta)),
        "tdd_preflight_syntax_dry_run_failed" => Ok(SsbActionType::TddPreflightBlocked(TddPreflightBlockedKind::SyntaxDryRunFailed)),
        "tdd_preflight_refactor_exempted" => Ok(SsbActionType::TddPreflightBlocked(TddPreflightBlockedKind::RefactorExempted)),
        // I-20 (SCHEMA_VERSION 255): MutationBudgetedRun Pattern-2 4-arm. Representative arm CiCleared.
        "mutation_budgeted_run_ci_cleared" => Ok(SsbActionType::MutationBudgetedRun(MutationBudgetedRunKind::CiCleared)),
        "mutation_budgeted_run_ci_failed" => Ok(SsbActionType::MutationBudgetedRun(MutationBudgetedRunKind::CiFailed)),
        "mutation_budgeted_run_survivor_directive" => Ok(SsbActionType::MutationBudgetedRun(MutationBudgetedRunKind::SurvivorDirective)),
        "mutation_budgeted_run_cache_hit" => Ok(SsbActionType::MutationBudgetedRun(MutationBudgetedRunKind::CacheHit)),
        // I-21 (SCHEMA_VERSION 256): SprintContractEvent Pattern-2 5-arm. Representative arm Proposed.
        "sprint_contract_proposed" => Ok(SsbActionType::SprintContractEvent(SprintContractEventKind::Proposed)),
        "sprint_contract_approved" => Ok(SsbActionType::SprintContractEvent(SprintContractEventKind::Approved)),
        "sprint_contract_rejected" => Ok(SsbActionType::SprintContractEvent(SprintContractEventKind::Rejected)),
        "sprint_contract_ralph_reinjected" => Ok(SsbActionType::SprintContractEvent(SprintContractEventKind::RalphReinjected)),
        "sprint_contract_adversary_escalated" => Ok(SsbActionType::SprintContractEvent(SprintContractEventKind::AdversaryEscalated)),
        // I-37 (SCHEMA_VERSION 257): RequirementVerified Pattern-2 3-arm.
        "requirement_verified_satisfied" => Ok(SsbActionType::RequirementVerified(RequirementVerifiedKind::Satisfied)),
        "requirement_verified_unverified" => Ok(SsbActionType::RequirementVerified(RequirementVerifiedKind::Unverified)),
        "requirement_verified_regressed" => Ok(SsbActionType::RequirementVerified(RequirementVerifiedKind::Regressed)),
        // I-44 (SCHEMA_VERSION 258): RetentionPurged Pattern-2 3-arm.
        "retention_raw_inputs_purged" => Ok(SsbActionType::RetentionPurged(RetentionPurgedKind::RawInputsPurged)),
        "retention_evidence_retained" => Ok(SsbActionType::RetentionPurged(RetentionPurgedKind::EvidenceRetained)),
        "retention_purge_verified" => Ok(SsbActionType::RetentionPurged(RetentionPurgedKind::PurgeVerified)),
        // I-52 (SCHEMA_VERSION 259): CheckpointToolEvent Pattern-2 4-arm.
        "checkpoint_tool_saved" => Ok(SsbActionType::CheckpointToolEvent(CheckpointToolEventKind::Saved)),
        "checkpoint_tool_rewound" => Ok(SsbActionType::CheckpointToolEvent(CheckpointToolEventKind::Rewound)),
        "checkpoint_tool_diffed" => Ok(SsbActionType::CheckpointToolEvent(CheckpointToolEventKind::Diffed)),
        "checkpoint_tool_refused_diverged" => Ok(SsbActionType::CheckpointToolEvent(CheckpointToolEventKind::RefusedDiverged)),
        // I-54 (SCHEMA_VERSION 260): StructuralEditApplied Pattern-2 5-arm.
        "structural_edit_matched" => Ok(SsbActionType::StructuralEditApplied(StructuralEditAppliedKind::Matched)),
        "structural_edit_previewed" => Ok(SsbActionType::StructuralEditApplied(StructuralEditAppliedKind::Previewed)),
        "structural_edit_applied" => Ok(SsbActionType::StructuralEditApplied(StructuralEditAppliedKind::Applied)),
        "structural_edit_refused_stale" => Ok(SsbActionType::StructuralEditApplied(StructuralEditAppliedKind::RefusedStale)),
        "structural_edit_parse_failed" => Ok(SsbActionType::StructuralEditApplied(StructuralEditAppliedKind::ParseFailed)),
        // I-4 (SCHEMA_VERSION 261): SearchNodeEvent Pattern-2 4-arm.
        "search_node_expanded" => Ok(SsbActionType::SearchNodeEvent(SearchNodeEventKind::Expanded)),
        "search_node_pruned" => Ok(SsbActionType::SearchNodeEvent(SearchNodeEventKind::Pruned)),
        "search_node_promoted" => Ok(SsbActionType::SearchNodeEvent(SearchNodeEventKind::Promoted)),
        "search_node_budget_collapsed" => Ok(SsbActionType::SearchNodeEvent(SearchNodeEventKind::BudgetCollapsed)),
        // I-7 (SCHEMA_VERSION 262): EscalationCascade Pattern-2 4-arm. Representative arm LocalWin.
        "escalation_cascade_local_win" => Ok(SsbActionType::EscalationCascade(EscalationCascadeKind::LocalWin)),
        "escalation_cascade_condensed" => Ok(SsbActionType::EscalationCascade(EscalationCascadeKind::Condensed)),
        "escalation_cascade_escalated" => Ok(SsbActionType::EscalationCascade(EscalationCascadeKind::Escalated)),
        "escalation_cascade_batch_routed" => Ok(SsbActionType::EscalationCascade(EscalationCascadeKind::BatchRouted)),
        // I-22 (SCHEMA_VERSION 264): ProgressRewardScored Pattern-2 3-arm. Representative arm Delta.
        "progress_reward_delta" => Ok(SsbActionType::ProgressRewardScored(ProgressRewardScoredKind::Delta)),
        "progress_reward_first_error_truncated" => Ok(SsbActionType::ProgressRewardScored(ProgressRewardScoredKind::FirstErrorTruncated)),
        "progress_reward_decay_applied" => Ok(SsbActionType::ProgressRewardScored(ProgressRewardScoredKind::DecayApplied)),
        // I-24 (SCHEMA_VERSION 266): PrmCanaryEvent Pattern-2 3-arm. Representative arm CanaryInjected.
        "prm_canary_injected"         => Ok(SsbActionType::PrmCanaryEvent(PrmCanaryEventKind::CanaryInjected)),
        "prm_canary_style_bias_detected" => Ok(SsbActionType::PrmCanaryEvent(PrmCanaryEventKind::StyleBiasDetected)),
        "prm_canary_scorer_neutralized"  => Ok(SsbActionType::PrmCanaryEvent(PrmCanaryEventKind::ScorerNeutralized)),
        // I-27 (SCHEMA_VERSION 265): ConvergenceCurveMeasured flat beacon.
        "convergence_curve_measured" => Ok(SsbActionType::ConvergenceCurveMeasured),
        // I-36 (SCHEMA_VERSION 267): PromptCompiled -- per-(role,backend) compiled-prompt
        // artifact event. 3 distinct sql_str values per-Kind arm.
        "prompt_compiled" => Ok(SsbActionType::PromptCompiled(PromptCompiledKind::Compiled)),
        "transfer_gap_measured" => Ok(SsbActionType::PromptCompiled(PromptCompiledKind::TransferGapMeasured)),
        "prompt_compiled_rolledback" => Ok(SsbActionType::PromptCompiled(PromptCompiledKind::Rolledback)),
        // I-50 (SCHEMA_VERSION 268): AuthBrokerEvent -- subscription auth broker lifecycle.
        // 4-arm Pattern-2; each arm has its own sql_str.
        "auth_broker_token_acquired" => Ok(SsbActionType::AuthBrokerEvent(AuthBrokerEventKind::TokenAcquired)),
        "auth_broker_refreshed" => Ok(SsbActionType::AuthBrokerEvent(AuthBrokerEventKind::Refreshed)),
        "auth_broker_provider_linked" => Ok(SsbActionType::AuthBrokerEvent(AuthBrokerEventKind::ProviderLinked)),
        "auth_broker_refresh_failed" => Ok(SsbActionType::AuthBrokerEvent(AuthBrokerEventKind::RefreshFailed)),
        // J-1.T4 (SCHEMA_VERSION 269): PerceptionScaffoldEvent -- perception-gateway
        // scaffold lifecycle. SINGLE-VARIANT Pattern-2; single sql_str, the 4-arm
        // Kind discriminator rides inside the payload. ScaffoldEmitted is the
        // representative arm (the success path, most-semantic).
        "perception_scaffold_event" => Ok(SsbActionType::PerceptionScaffoldEvent(PerceptionScaffoldEventKind::ScaffoldEmitted)),
        // K-18a follow-on (SCHEMA_VERSION 270): RecurrenceHitRateMeasured flat beacon.
        "recurrence_hit_rate_measured" => Ok(SsbActionType::RecurrenceHitRateMeasured),
        // K-4 SP1 follow-on (SCHEMA_VERSION 271): GateTaxMeasured flat beacon.
        "gate_tax_measured" => Ok(SsbActionType::GateTaxMeasured),
        // J-28 follow-on: UsageTurnEvent flat variant.
        "usage_turn_event" => Ok(SsbActionType::UsageTurnEvent),
        // J-9: hardware/model catalog flat variant.
        "hardware_model_catalog_event" => Ok(SsbActionType::HardwareModelCatalogEvent),
        // K-2 (SCHEMA_VERSION 274): routing economics benchmark flat beacon.
        "routing_economics_measured" => Ok(SsbActionType::RoutingEconomicsMeasured),
        // K-5 (SCHEMA_VERSION 275): GateIntensity selection beacon.
        "gate_intensity_selected" => Ok(SsbActionType::GateIntensitySelected),
        // K-9 (SCHEMA_VERSION 276): model refresh cadence decision row.
        "model_refresh_cadence" => Ok(SsbActionType::ModelRefreshCadence),
        // P3-L10RUNNER (SCHEMA_VERSION auto): hallucination-delta flat beacon.
        "hallucination_delta_measured" => Ok(SsbActionType::HallucinationDeltaMeasured),
        // PS-READINESS-E2ERUNNER Slice B (SCHEMA_VERSION 278): local-lane e2e
        // functional pass flat beacon.
        "local_lane_e2e_measured" => Ok(SsbActionType::LocalLaneE2eMeasured),
        // J-13 (SCHEMA_VERSION 279): local-lane determinism telemetry event.
        "local_determinism_event" => Ok(SsbActionType::LocalDeterminismEvent(
            LocalDeterminismKind::BatchInvariantEnabled,
        )),
        // J-14 (SCHEMA_VERSION 282): repro-pack + RRA verify upgrade telemetry event.
        "repro_pack_event" => Ok(SsbActionType::ReproPackEvent(
            ReproPackKind::TierEvaluated,
        )),
        // J-16 (SCHEMA_VERSION 283): blast-radius / anti-regression gate event.
        "blast_radius_regression_event" => Ok(SsbActionType::BlastRadiusRegressionEvent(
            BlastRadiusRegressionKind::ConeComputed,
        )),
        // J-30 T7b (SCHEMA_VERSION 284): provider BYOK lifecycle event.
        // Flat variant; Kind rides in the payload, not the action type.
        "provider_byok_event" => Ok(SsbActionType::ProviderByokEvent),
        // J-32 (SCHEMA_VERSION 285): front-door intent routing decision.
        // Flat variant; Kind rides in the payload, not the action type.
        "intent_routed" => Ok(SsbActionType::IntentRouted),
        // J-34 (SCHEMA_VERSION 286): empirical context calibration event.
        // Flat variant; Kind rides in the payload, not the action type.
        "context_calibration_event" => Ok(SsbActionType::ContextCalibrationEvent(
            ContextCalibrationKind::CurveMeasured,
        )),
        // J-36 (SCHEMA_VERSION 287): local prefix-KV cache event.
        // Flat variant; Kind rides in the payload, not the action type.
        "local_prefix_cache_event" => Ok(SsbActionType::LocalPrefixCacheEvent(
            LocalPrefixCacheKind::HeadCacheHit,
        )),
        // J-24 (SCHEMA_VERSION 288): behavior steering event.
        // Flat variant; Kind rides in the payload, not the action type.
        "behavior_steering_event" => Ok(SsbActionType::BehaviorSteeringEvent(
            BehaviorSteeringKind::ObservationSizePreassessed,
        )),
        // J-35 intent-control layer: flat variant; Kind rides in the payload, not the action type.
        "intent_control_event" => Ok(SsbActionType::IntentControlEvent(
            IntentControlKind::SpecReconstructed,
        )),
        // J-39 T2: node admission gate event. Flat variant; Kind rides in the payload.
        "node_admission_event" => Ok(SsbActionType::NodeAdmissionEvent(
            NodeAdmissionEventKind::Admitted,
        )),
        "system_contract_event" => Ok(SsbActionType::SystemContractEvent(
            SystemContractEventKind::NodeBound,
        )),
        "harness_intervention_event" => Ok(SsbActionType::HarnessInterventionEvent(
            HarnessInterventionKind::ActionGateBlocked,
        )),
        "free_pool_event" => Ok(SsbActionType::FreePoolEvent),
        "retrieval_abstained" => Ok(SsbActionType::RetrievalAbstained),
        "context_access_spec" => Ok(SsbActionType::ContextAccessSpec),
        "dag_credit_assigned" => Ok(SsbActionType::DagCreditAssigned),
        // I-30 Slice 1a: SFT exhaust event (flat; Kind in payload).
        "sft_exhaust_event" => Ok(SsbActionType::SftExhaustEvent),
        "review_gate_catalog_event" => Ok(SsbActionType::ReviewGateCatalogEvent),
        "claim_risk_scored" => Ok(SsbActionType::ClaimRiskScored),
        // J-42 T2: reasoning checkpoint. Flat variant; Kind rides in the payload.
        "reasoning_checkpoint_event" => Ok(SsbActionType::ReasoningCheckpointEvent(
            ReasoningCheckpointEventKind::ResetAndContinued,
        )),
        other => Err(SsbError::UnknownActionType(other.to_owned())),
    }
}

/// Validates that the deserialized payload matches the declared action_type.
/// This prevents silent corruption where the action_type string doesn't match
/// the actual payload struct type.
pub(crate) fn validate_payload_matches_action_type(
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

