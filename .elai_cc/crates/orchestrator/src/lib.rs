pub mod agent_pool;
pub mod agent_template;
pub mod deterministic_audit_gate;
pub mod deterministic_auditor;
pub mod grc;
pub mod anti_spiral;
pub mod autonomous_gate;
pub mod sandbox;
pub mod bsp;
mod bash;
pub mod checkpoint;
pub mod coherence_check;
pub mod dst;
pub mod cloud_mode;
pub mod context_budget;
pub mod dispatcher;
pub mod privacy_redactor;
pub mod branch_lock;
mod compact;
mod conversation;
pub mod evidence;
pub mod fsm;
pub mod gate;
pub mod hooks;
pub mod hook_dispatcher;
pub mod shortcoder_hook;
pub mod provider_signals;
pub mod indexing_service;
pub mod ipc;
pub mod ipc_gateway;
pub mod ipc_types;
pub mod tool_filter;
mod lane_events;
mod mcp;
mod mcp_client;
pub mod mcp_lifecycle_hardened;
mod mcp_stdio;
pub mod mcp_tool_bridge;
pub mod permission_enforcer;
mod permissions;
pub mod plugin_lifecycle;
mod preload;
mod prompt;
pub mod recovery;
pub mod recovery_recipes;
pub mod grounding_client;
pub mod research_budget;
pub mod research_pipeline;
pub mod searx_search_backend;
pub mod research_tier;
pub mod research_tree;
pub mod pipeline_builder;
pub mod phase_sizing_splitter;
pub mod pipeline_tree_navigator;
#[cfg(test)]
mod pipeline_builder_bench;
#[cfg(test)]
mod pipeline_builder_tests;
pub mod web_research_orchestrator;
mod session;
#[cfg(test)]
mod session_control;
pub mod plan_skeleton;
pub mod session_dag;
pub mod workload_profiling;
pub use plan_skeleton::{
    PlanSkeleton, PlanSkeletonError, PhaseEntry, PhaseContext, PhaseDetail,
    ExtractedContracts, AdjacentContracts, ContractRef, StalenessWarning,
};
pub use session_dag::{
    SessionDag, SessionDagError, Snapshot, TrimmedMessage, TrimConfig,
    BranchRef, DagMessageRole, MergeResult, Stub,
};
pub mod irreversible_classifier;
pub mod micro_reflection_emitter;
pub mod mistake_book;
pub mod sidecar_reflection_adapter;
pub mod output_postprocessor;
pub mod profile_selector;
pub mod rule_loader;
pub mod config_archive;
pub mod evolution_gate;
pub mod msi;
pub mod osc;
pub mod regression;
pub mod self_mod_bridge;
pub mod self_mod_layer1a;
pub mod self_mod_proposal;
pub mod self_mod_orchestrator;
pub mod self_mod_safety;
pub mod self_mod_types;
pub mod ssb;
pub mod staged_eval;
pub mod ssb_viewer;
// B-24 Phase 4I (Group C1): media validator skeleton. See
// `media_validator/mod.rs` for the per-group rollout map.
pub mod media_validator;
pub mod system_probe;
pub mod telescope_caller;
pub mod feasibility_probe;
pub mod task_runner;
pub mod supervision;
pub mod task_packet;
pub mod task_registry;
pub mod team_cron_registry;
pub mod worker_boot;

// Re-export modules from external crates under their old names so that
// internal `crate::config::*`, `crate::sandbox::*`, etc. paths still resolve.
pub use elai_core::config;
pub mod json {
    pub use elai_core::json::{JsonError, JsonValue};
}
pub mod usage {
    pub use elai_core::{
        format_usd, pricing_for_model, ModelPricing, TokenUsage, UsageCostEstimate, UsageTracker,
    };
}
pub mod bootstrap {
    pub use elai_core::{BootstrapPhase, BootstrapPlan};
}
pub mod oauth {
    pub use elai_providers::oauth::*;
}
pub mod telemetry_types {
    pub use elai_providers::telemetry_types::*;
}
pub mod sse {
    pub use elai_providers::{parse_frame, SseParser};
}
pub mod green_contract {
    pub use elai_rules::green_contract::*;
}
pub mod policy_engine {
    pub use elai_rules::policy_engine::*;
}
#[cfg(test)]
pub mod trust_resolver {
    pub use elai_rules::trust_resolver::*;
}

// Usage types re-exported at crate root (many files use `crate::TokenUsage` etc.)
pub use elai_core::{
    format_usd, pricing_for_model, ModelPricing, TokenUsage, UsageCostEstimate, UsageTracker,
};

pub use bash::{execute_bash, BashCommandInput, BashCommandOutput};
pub use fsm::{
    EvidenceArtifact, Fsm, FsmError, FsmState, PersistedFsmState, ReplanLogEntry, ReplanRequest,
    ReplanSeverity, TransitionRecord,
};
pub use evidence::{
    all_gates, check_transition, daira_adaptive_run, gates_for_transition, AuditFinding,
    DairaScope, DairaScopeResult, DkgChangelogGate, EvidenceError, EvidenceGate,
    FindingSeverity, GroundingSweepGate, KnowledgeReadiness, KnowledgeReadinessGate,
    ReplanFastTrackGate, TestFailReasonsGate, TestFlippedGate, TestOutcome,
    TransitionContext, ZeroBlocksGate,
};
pub use gate::{
    default_risk_level, run_interactive_gate, EvidenceSummary, GateDecision, GateResult,
    InteractiveGate, InteractiveGateOutcome, RiskLevel,
};
#[cfg(test)]
pub use gate::MockInteractiveGate;
// Batch 3B (Phase 4G) — AutonomousGate surface.
pub use autonomous_gate::{
    autonomous_gate_disabled_via_env, AutonomousGate, AutonomousGateConfig, AutonomyDecision,
    EscalationTrigger, Evidence as AutonomousEvidence, GateContext as AutonomousGateContext,
    GateMode, PipelineLevel, TreeAutoAdvancePolicy, ELAI_DISABLE_AUTONOMOUS_GATE_ENV,
};
pub use tool_filter::{RuleCategory, ToolCategory};
pub use dispatcher::{
    BackendConfig, BackendMode, DispatchContext, DispatchError, PhaseSlot,
    build_dispatch_context, CONSTRAINTS_BLOCK_HEADER, LOCAL_MODEL_MAX_CONTEXT,
    LOCAL_BUDGET_FRACTION,
};
pub use elai_core::{BootstrapPhase, BootstrapPlan};
// Batch 2B: re-export ToolDispatcher trait + value types so downstream
// callers (dashboard, contracts) program against `elai_orchestrator::{...}`
// without a direct `elai_core::tool_dispatcher::` path. The concrete
// `InProcessToolDispatcher` impl lives in `elai-tools` and is imported
// there directly (elai-tools depends on elai-orchestrator for facade
// re-exports, so the dep direction blocks a tools re-export here).
pub use elai_core::tool_dispatcher::{
    ClassifiedFailure as ToolClassifiedFailure, CommitOutcome, EditMatchTier, EditOutcome,
    EditRequest, FailureInput, FileView, GitBranchPolicy, GitCommitRequest, IncrementalOutcome,
    IncrementalRequest, ToolDispatchError, ToolDispatcher, ToolFailureCategory, ValidateRequest,
    ValidationTier, ValidationTierResult,
};
pub use branch_lock::{detect_branch_lock_collisions, BranchLockCollision, BranchLockIntent};
pub use compact::{
    compact_session, estimate_session_tokens, format_compact_summary,
    get_compact_continuation_message, should_compact, CompactionConfig, CompactionResult,
};
pub use elai_core::config::{
    ConfigEntry, ConfigError, ConfigLoader, ConfigSource, McpConfigCollection,
    McpManagedProxyServerConfig, McpOAuthConfig, McpRemoteServerConfig, McpSdkServerConfig,
    McpServerConfig, McpStdioServerConfig, McpTransport, McpWebSocketServerConfig, OAuthConfig,
    ResolvedPermissionMode, RuntimeConfig, RuntimeFeatureConfig, RuntimeHookConfig,
    RuntimePermissionRuleConfig, RuntimePluginConfig, ScopedMcpServerConfig,
    ELAI_SETTINGS_SCHEMA_NAME,
};
pub use conversation::{
    auto_compaction_threshold_from_env, ApiClient, ApiRequest, AssistantEvent, AutoCompactionEvent,
    ConversationRuntime, PromptCacheEvent, RuntimeError, StaticToolExecutor, ToolError,
    ToolExecutor, TurnSummary,
};
pub use hooks::{
    HookAbortSignal, HookEvent, HookProgressEvent, HookProgressReporter, HookRunResult, HookRunner,
    StopHookDecision, StopHookGate, STOP_HOOK_DOUBLE_PRESS_WINDOW_MS,
};
pub use hook_dispatcher::{
    emit_hook_events_to_ssb, emit_hook_latency_violation_to_ssb,
    emit_shortcoder_compression_to_ssb, emit_tiered_analysis_summary_to_ssb,
    hook_tier_budget_ms, report_hook_latency, sanitized_block_message, HookDispatcher,
    HookError as HookDispatcherError, HookFinding, HookInvocation, HookPoint, HookTier,
    HookVerdictKind, InProcessHookDispatcher, ELAI_HOOKS_DISABLED_ENV, HOOK_EVENT_CATEGORY,
    HOOK_T1_BUDGET_MS, HOOK_T2_BUDGET_MS, HOOK_T3_BUDGET_MS, SANITIZED_BLOCK_MESSAGE,
};
pub use shortcoder_hook::{
    ShortCoderHook, ShortCoderHookError, ShortCoderOutcome, ELAI_DISABLE_SHORTCODER_ENV,
    SHORTCODER_MAX_RULES_EMITTED,
};
pub use provider_signals::{
    provider_signals_disabled_via_env, EtcState, GgSignals, NoveltySignal,
    NullProviderSignalReader, ProviderSignalReader, SearchTrigger, SteeringVectorStatus,
    VramDetectionSource, VramTierConfig as ProviderVramTierConfig,
    ELAI_DISABLE_PROVIDER_SIGNALS_ENV,
};
#[cfg(feature = "sidecar")]
pub use provider_signals::SidecarProviderSignalReader;
pub use lane_events::{
    dedupe_superseded_commit_events, LaneCommitProvenance, LaneEvent, LaneEventBlocker,
    LaneEventName, LaneEventStatus, LaneFailureClass,
};
pub use mcp::{
    mcp_server_signature, mcp_tool_name, mcp_tool_prefix, normalize_name_for_mcp,
    scoped_mcp_config_hash, unwrap_ccr_proxy_url,
};
pub use mcp_client::{
    McpClientAuth, McpClientBootstrap, McpClientTransport, McpManagedProxyTransport,
    McpRemoteTransport, McpSdkTransport, McpStdioTransport,
};
pub use mcp_lifecycle_hardened::{
    McpDegradedReport, McpErrorSurface, McpFailedServer, McpLifecyclePhase, McpLifecycleState,
    McpLifecycleValidator, McpPhaseResult,
};
pub use mcp_stdio::{
    spawn_mcp_stdio_process, JsonRpcError, JsonRpcId, JsonRpcRequest, JsonRpcResponse,
    ManagedMcpTool, McpDiscoveryFailure, McpInitializeClientInfo, McpInitializeParams,
    McpInitializeResult, McpInitializeServerInfo, McpListResourcesParams, McpListResourcesResult,
    McpListToolsParams, McpListToolsResult, McpReadResourceParams, McpReadResourceResult,
    McpResource, McpResourceContents, McpServerManager, McpServerManagerError, McpStdioProcess,
    McpTool, McpToolCallContent, McpToolCallParams, McpToolCallResult, McpToolDiscoveryReport,
    UnsupportedMcpServer,
};
pub use elai_providers::oauth::{
    clear_oauth_credentials, code_challenge_s256, credentials_path, generate_pkce_pair,
    generate_state, load_oauth_credentials, loopback_redirect_uri, parse_oauth_callback_query,
    parse_oauth_callback_request_target, save_oauth_credentials, OAuthAuthorizationRequest,
    OAuthCallbackParams, OAuthRefreshRequest, OAuthTokenExchangeRequest, OAuthTokenSet,
    PkceChallengeMethod, PkceCodePair,
};
pub use permissions::{
    PermissionContext, PermissionMode, PermissionOutcome, PermissionOverride, PermissionPolicy,
    PermissionPromptDecision, PermissionPrompter, PermissionRequest,
};
pub use plugin_lifecycle::{
    DegradedMode, DiscoveryResult, PluginHealthcheck, PluginLifecycle, PluginLifecycleEvent,
    PluginState, ResourceInfo, ServerHealth, ServerStatus, ToolInfo,
};
pub use elai_rules::policy_engine::{
    evaluate, DiffScope, GreenLevel, LaneBlocker, LaneContext, PolicyAction, PolicyCondition,
    PolicyEngine, PolicyRule, ReconcileReason, ReviewStatus,
};
pub use prompt::{
    load_system_prompt, prepend_bullets, ContextFile, ProjectContext, PromptBuildError,
    SystemPromptBuilder, FRONTIER_MODEL_NAME, SYSTEM_PROMPT_DYNAMIC_BOUNDARY,
};
pub use research_pipeline::{
    run_research_pipeline, phase_a_formulate_queries, phase_b_search, phase_c_classify,
    phase_d_extract_facts, phase_e_cross_verify, phase_f_synthesize, phase_g_post_verify,
    ClassifiedResults, CollectedResults, FormulatedQueries, PostVerificationResult,
    SearchBackend, SearchResult, StubSearchBackend,
};
pub use grounding_client::{GroundingClient, SidecarGroundingClient};
pub use searx_search_backend::{SearxSearchBackend, DEFAULT_SIDECAR_SEARCH_URL};
pub use pipeline_builder::{
    PipelineBuilder, PipelineBuilderError, PipelineId, NodeIndex, StateId, DataRef, RuleId,
    ExecutablePipeline, DependencyGraph, DagNode, DagRepairLoop, PipelineFsmState,
    AgentPhaseConfig, RuleBinding, ResourceEstimate, ResourceReq, TestHarnessSkeleton,
    TestCase, FixtureStub, RepairAction, RepairOutcome, ValidationError, ValidationErrorKind,
    StructuralValidator, FsmStateGenerator, TestHarnessGenerator, ResourceEstimator,
    NoveltyDetector, AgentConfigurator, RuleInstaller, ToolLookup, PredicateCompiler,
    PredicateRef, T_DAG, CONTRACT_OVERHEAD, SPLIT_RATIO, MAX_SPLIT_DEPTH,
};
pub use recovery_recipes::{
    attempt_recovery, recipe_for, EscalationPolicy, FailureScenario, RecoveryContext,
    RecoveryEvent, RecoveryRecipe, RecoveryResult, RecoveryStep,
};
pub use elai_core::sandbox::{
    build_linux_sandbox_command, detect_container_environment, detect_container_environment_from,
    resolve_sandbox_status, resolve_sandbox_status_for_request, ContainerEnvironment,
    FilesystemIsolationMode, LinuxSandboxCommand, SandboxConfig, SandboxDetectionInputs,
    SandboxRequest, SandboxStatus,
};
pub use sandbox::{
    AgentEnvironment, AgentProcess, AgentSandbox, AppContainerConfig, JobObjectLimits,
    PlatformSandboxHandle, SandboxError, SandboxHandle, SelfModificationLocks, create_sandbox,
};
#[cfg(target_os = "windows")]
pub use sandbox::{WindowsAppContainerSandbox, WindowsSandboxHandle};
#[cfg(target_os = "linux")]
pub use sandbox::{LinuxSandbox, LinuxSandboxHandle};
pub use ipc_gateway::{
    AgentConnection, FrameRouteResult, GroundingMethod, IpcGateway, IpcGatewayError,
    allowed_grounding_methods, create_session_socket_dir,
};
pub use session::{
    ContentBlock, ConversationMessage, MessageRole, Session, SessionCompaction, SessionError,
    SessionFork,
};
pub use task_packet::{validate_packet, TaskPacket, TaskPacketValidationError, ValidatedPacket};
#[cfg(test)]
pub use elai_rules::trust_resolver::{TrustConfig, TrustDecision, TrustEvent, TrustPolicy, TrustResolver};
pub use elai_providers::telemetry_types::{
    AnalyticsEvent, AnthropicRequestProfile, ClientIdentity, JsonlTelemetrySink,
    MemoryTelemetrySink, SessionTraceRecord, SessionTracer, TelemetryEvent, TelemetrySink,
    DEFAULT_ANTHROPIC_VERSION,
};
pub use anti_spiral::{AntiSpiralDetector, SpiralResult, ESCALATION_DEPTH, ESCALATION_MESSAGE, HARD_STOP_DEPTH, RING_SIZE, SPIRAL_THRESHOLD};
pub use deterministic_audit_gate::{
    ChangedFilesSupplier, DefaultDeterministicAuditGate, DeterministicAuditGate,
    ProjectProfileSupplier, GATE_AGENT_ID, GATE_FINDING_CATEGORY, GATE_SUPERSTEP,
};
pub use deterministic_auditor::{
    CheckKind, DeterministicAuditResult, DeterministicAuditor, DeterministicAuditorConfig,
    DeterministicFinding, DeterministicSeverity,
};
pub use agent_pool::{
    default_grc_config, role_default_allowed_paths, AgentHandle, AgentMessage, AgentPool,
    AgentPoolConfig, AgentPoolError, AgentRole, AgentStatus, InProcessAgentPool, ModelPool,
    VRAM_MIN_BYTES,
};
pub use agent_template::{
    build_agent_prompt, build_agent_system_preamble, AgentOutputFormat, AgentPromptContext,
    AgentTemplateConfig, AgentTypeConfig,
};
pub use grc::{
    pipeline_grc_disabled_via_env, PipelineGrc, PipelineGrcError,
    ELAI_DISABLE_PIPELINE_GRC_ENV,
};
pub use output_postprocessor::{process as postprocess_output, OutputConfig};
pub use worker_boot::{
    Worker, WorkerEvent, WorkerEventKind, WorkerEventPayload, WorkerFailure, WorkerFailureKind,
    WorkerPromptTarget, WorkerReadySnapshot, WorkerRegistry, WorkerStatus, WorkerTrustResolution,
};
pub use ssb::{
    default_ssb_path, default_ssb_path_for_base, DairaTracePayload, DiffSummary, DkgReadyPayload,
    FailureClassifiedPayload, FeasibilityReportPayload, GitCommitPayload, HookEventPayload,
    IndexingProgressPayload, InterventionKeyword, InterventionOption, InterventionRequestPayload,
    InterventionResponsePayload, InterventionRetry, InterventionType, MistakeEntryPayload,
    NovelStepDetectedPayload, PhaseCompletePayload, PreviewAvailablePayload, ProviderSignalsPayload,
    SearchTriggerFiredPayload, Ssb, SsbActionType, SsbConfig, SsbEntry, SsbError, SsbFileEntry,
    FewShotExample, FindingPayload, SsbPayload, SsbReader, SsbWriteRequest, StubRunPayload,
    SystemProbePayload, TaskSubmittedPayload, ToolDispatchPayload, ValidationTierPayload,
    VramTierDetectedPayload, WebResearchStepPayload, SSB_DB_FILE_NAME, SSB_SESSIONS_SUBDIR,
    // Batch 3B (Phase 4G).
    EscalationFiredPayload, GateDecisionPayload, ModeChangedPayload, TaskSplitPayload,
    // Batch 3C (Phase 4H) — Layer 1A.
    ConfigArchivedPayload, MsiUpdatePayload, ProposalAppliedPayload, ProposalGeneratedPayload,
    ProposalRejectedPayload,
    // Batch 4C — scattered A-step wiring.
    ConfigWarningPayload, FsmTransitionPayload, MicroReflectionPayload,
    ProfileSelectedPayload, TelescopeCollapsePayload,
    // Batch-fu — cross-cutting follow-up wiring.
    ConstraintsInjectedPayload, InterventionResolvedPayload,
    // Batch-ful-a — DkgQuerierAdapter hot-swap.
    DkgQuerierSwappedPayload,
    // Batch-ful-c — A-8 freshness steering vector.
    SteeringComputedPayload, SteeringComputeFailedPayload,
    // Batch-ful-d — external MCP server lifecycle.
    McpServerConnectedPayload, McpServerDisconnectedPayload, McpToolCalledPayload,
    // Tier 1 CHAT window — dashboard-originated message + card rows.
    ChatMessagePayload, ChatResponsePayload,
    // Tier 4 Settings reorg (§8) — execution-mode selection row.
    ExecutionModeChangedPayload,
    // Track A.2 — synthesis path selection row.
    ResearchPathChosenPayload,
    // Track A.6 — grounding cache hit/miss event row.
    GroundingCacheEventPayload,
    // B-10 (DECISION-C) — per-dispatch ContextBudget snapshot payload re-exported
    // so dashboard commands.rs can read real per-slot allocations from SSB.
    ContextBudgetComputedPayload,
    // B-6 — orchestration producer wiring payloads.
    AgentLifecyclePayload, AntiSpiralHitPayload, BspSuperstepPayload,
    EvidenceGateEvaluatedPayload, SupervisionEventPayload,
    // B-27 — cloud-mode spend ledger + escalation marker.
    CloudSpendPayload, EscalateToCloudPayload,
    // B-23 Phase 4H Group A — self-mod beacons (Layer 1A growth + Layer 3 +
    // OSC + REG + EVO).
    ConfigOscillationPayload, ConvergenceStagnantPayload, DataTableGrewPayload,
    EvolutionWithheldPayload, L3ProposalPayload, RegressionDetectedPayload,
    // B-39 chat reflect pipeline — per-turn REFLECT stage beacons.
    ReflectAuditVerdictPayload, ReflectCandidatePayload, ReflectRefinedPayload,
    ReflectTriagedPayload,
    // B-24 Phase 4I (Group C1) — media-validator beacons (re-exported so
    // dashboard-v2 can deserialize SSB rows by payload type without pulling
    // in the orchestrator's internal `ssb` module path).
    MediaValidationStartedPayload, MediaTier1CompletePayload,
    MediaTier2CompletePayload, RenderCompletePayload,
    VisualFeedbackIterationPayload, MediaHumanReviewRequiredPayload,
};
// Batch 4C — public surfaces for the new modules.
pub use profile_selector::{
    emit_profile_selected, profile_name, DefaultGenerationProfileSelector, DispatchRole,
    GenerationProfileSelector,
};
pub use telescope_caller::{invoke_telescope, TelescopeResult, TelescopeStats};
pub use micro_reflection_emitter::{
    emit_micro_reflection, parse_reflection_json, DynReflectionAdapter, DynReflectionClient,
    ReflectionClient, ReflectionClientError,
};
pub use sidecar_reflection_adapter::{
    micro_reflection_disabled_via_env, SidecarReflectionAdapter,
    ELAI_DISABLE_MICRO_REFLECTION_ENV,
};
pub use irreversible_classifier::{
    irreversible_classifier_disabled_via_env, IrreversibleActionClassifier, IrreversibleVerdict,
    SharedIrreversibleClassifier, CONFIRM_IRREVERSIBLE_ACTION_KIND,
    ELAI_DISABLE_IRREVERSIBLE_CLASSIFIER_ENV,
};
// Batch 3C (Phase 4H): Self-Modification Layer 1A surface.
pub use self_mod_layer1a::{
    acquire_rules_lock, build_pattern_regex, classify_table, estimate_fpr, is_allowed,
    is_security_sensitive, layer1a_auto_apply_via_env, layer1a_disabled_via_env,
    status_label as msi_status_label, AllowedConfigKey, AppliedProposalRecord, DataTable,
    DataTableProposal, DefaultLayer1aProcessor, FailureRecord, FindingRecord, Layer1aError,
    Layer1aProcessor, MsiStatus, PipelineMode, PipelineOutcome, ProposalOutcome, ProposalSource,
    RulesDbLockGuard, DEFAULT_ARCHIVE_CAP, DEFAULT_MAX_FPR, DEFAULT_MIN_OCCURRENCES,
    DEFAULT_MSI_LAMBDA, DEFAULT_MSI_WINDOW, ELAI_DISABLE_LAYER1A_ENV, ELAI_LAYER1A_AUTO_APPLY_ENV,
    LAYER1A_AGENT_ID, MSI_DIVERGING_THRESHOLD, MSI_IMPROVING_THRESHOLD, RULES_DB_LOCKFILE,
};
// B-23 Phase 4H Group B: DGM ConfigVariant archive (4H.3) + log-driven
// SelfModProposal builder + Class A/B routing (4H.4).
pub use config_archive::{
    exploration_bonus, parent_selection_probabilities, parent_selection_weights,
    sample_variant, sigmoid_perf_score, ConfigArchive, ConfigArchiveConfig,
    ConfigArchiveError, DEFAULT_ARCHIVE_CAP_PER_MODE, DGM_ALPHA_0, DGM_LAMBDA,
};
pub use self_mod_proposal::{
    route_proposal, ProposalClass, ProposalEngine, ProposalEngineError,
};
// B-23 Phase 4H Group C — bridge from FullPipelineOutcome to Layer 1A
// proposals + REG pre-screen stub (real REG lands in Group E).
pub use self_mod_bridge::{proposals_from_full_outcome, regression_pre_screen};
// B-23 Phase 4H Group D — staged evaluation (4H.5) + 4-layer safety (4H.6).
pub use staged_eval::{
    evaluate_stage, run_full_pipeline, BenchmarkRunner, StageOutcome, StageReport, StageVerdict,
    StagedEvalError,
};
pub use self_mod_safety::{
    apply_structural_change, check_lock_list_read, check_locked_key, is_known_parameter,
    is_locked, should_auto_revert, SafetyError, DEFAULT_AUTO_REVERT_FRACTION,
    DEFAULT_AUTO_REVERT_WINDOW, KNOWN_PARAMETER_KEYS, LOCKED_KEYS,
};
// B-23 Phase 4H Group E — MSI per-mode tracker (4H.7), OSC ouroboros port
// (4H.7B), REG cross-pipeline regression detector (4H.7C), EVO evolution
// gate (4H.7D).
pub use msi::{
    msi_verdict, MsiSnapshot, MsiTracker, MsiVerdict, MSI_DIVERGING_THRESHOLD as MSI_DIVERGING,
    MSI_IMPROVING_THRESHOLD as MSI_IMPROVING, MSI_LAMBDA as MSI_LAMBDA_SPEC, MSI_WINDOW,
};
pub use osc::{compute_osc_verdict, compute_similarity};
pub use regression::detect as detect_regressions;
pub use evolution_gate::compute_evolution_gate;
// B-23 Phase 4H Group F — top-level self-modification orchestration (4H.8).
pub use self_mod_orchestrator::{
    SelfModOrchestrator, SelfModOrchestratorError, SelfModUpdate,
};

/// B-23 Phase 4H Group F — msgpack helper for callers that don't want to
/// depend on `rmp_serde` directly. Used by dashboard-v2 Tauri commands
/// to encode SSB payloads.
pub fn encode_ssb_payload(payload: &ssb::SsbPayload) -> Result<Vec<u8>, rmp_serde::encode::Error> {
    rmp_serde::to_vec_named(payload)
}

/// B-24 Phase 4I Group C8 audit WARN 2 — inner-payload msgpack helper.
/// B-24 Phase 4I Group C8 — generic msgpack decode helper for callers
/// (currently dashboard-v2) that consume SSB payload bytes from the
/// broadcast subscriber but do not want a direct `rmp_serde` dependency.
/// Returns the deserialization error string so the caller can log/propagate
/// without coupling to the underlying msgpack lib version.
///
/// Decision 92 (B-24 Phase 4I Group C8 audit-fix): the dashboard
/// subscriber must instantiate `T = SsbPayload` so the externally-tagged
/// enum decode matches the production wire format produced by
/// `Ssb::write_payload` (`rmp_serde::to_vec_named(&payload: SsbPayload)`,
/// see ssb.rs L2988). Decoding into the inner payload struct directly
/// fails on real production bytes — the previous helper
/// `encode_inner_payload` was removed because it documented a
/// test-only encoding that did not exist in production.
pub fn decode_ssb_payload_bytes<T: serde::de::DeserializeOwned>(
    bytes: &[u8],
) -> Result<T, String> {
    rmp_serde::from_slice::<T>(bytes).map_err(|e| e.to_string())
}
// B-23 Phase 4H Group A: shared self-modification types used across the
// new modules (config_archive, msi, osc, regression, evolution_gate) and
// the dashboard wiring layer.
//
// `PipelineId` and `RuleId` are intentionally NOT re-exported here — they
// collide with existing pipeline_builder + rules-crate types of the same
// name. Consumers that need the self-mod alias use the fully-qualified
// `crate::self_mod_types::PipelineId` path.
pub use self_mod_types::{
    AcRegression, AgentId, BabyStepId, CatchLayer, CatchRate, ConfigVariant,
    ConfigVariantSimilarity, DataTableValidation, DataTableVerdict, EvolutionGateResult,
    FullPipelineOutcome, GrcStepType, Language, OscVerdict, PhaseId,
    PipelineOutcomeId, ProjectScope, ProposedChange, ReasoningTraceRef, RegressionReport,
    ScoreWeights, SelfModProposal, Slot, SsbLogEntry, StagedEvaluation, StaleApiCatch,
    StructuralChange, StructuralChangeType, TemplateId, TestId, TestOutcomeReport, TriggerCount,
    UnclassifiedError, VariantId, VariantStatus, ProjectId,
    DEFAULT_SCORE_WEIGHTS, SIM_STAGNATION_THRESHOLD, SIM_STAGNATION_WINDOW,
    SIM_WEIGHT_EXACT_MATCH, SIM_WEIGHT_KEY_OVERLAP, SIM_WEIGHT_TYPE_MATCH,
    STAGE1_DEFAULT_TASKS, STAGE2_DEFAULT_TASKS, STAGE3_DEFAULT_TASKS,
    STAGE3_PROMOTION_THRESHOLD, STAGE3_RANKING_SUBSET,
};
pub use system_probe::{
    classify_vram_tier, system_probe_disabled_via_env, DefaultSystemProber, DetectionMethod,
    GpuInfo, SystemProbeReport, SystemProber, ELAI_DISABLE_SYSTEM_PROBE_ENV,
    ELAI_FREE_DISK_GB_ENV, ELAI_FREE_RAM_GB_ENV, ELAI_VRAM_FREE_GB_ENV, ELAI_VRAM_GB_ENV,
};
pub use feasibility_probe::{
    alternative_within_cap, check_constraints, decide_with_mode, enforce_stub_line_limit,
    feasibility_probe_disabled_via_env, BabyStepEscalation, BabyStepFeasibilityReport,
    BabyStepStubResult, ConstraintResult, DefaultFeasibilityProber, FeasibilityDecision,
    FeasibilityEscalation, FeasibilityExecutionMode, FeasibilityProber, FeasibilityReport,
    NovelStep, ResourceMargins, StubResult, SystemConstraintsRequired, TaskFeasibility,
    ELAI_DISABLE_FEASIBILITY_PROBE_ENV, MAX_ALTERNATIVES_PER_STEP, RESOURCE_MARGIN_THRESHOLD_PCT,
    STUB_LINE_LIMIT, STUB_TIMEOUT_GRACEFUL, STUB_TIMEOUT_HARD,
};
pub use ssb_viewer::{
    ChatEntry, FeasibilityReportRow, FeasibilitySummary, FileChange, FsmStateSnapshot,
    InterventionRequest, NovelStepRow, PhaseStatus, SessionInfo,
    RuleViolationSummary, SsbActionDetail, SsbActionSummary, SsbViewer, SsbViewerError,
    StepStatus, StubRunRow, TaskSummary, ThinkStepSummary, validate_request_id,
    // Tier 2 Track B (dashboard-wiring §6) — PlanWindow / PreviewWindow DTOs.
    // `PhaseDetail` is renamed on re-export to `DashboardPhaseDetail` because
    // `plan_skeleton::PhaseDetail` (re-exported above) already occupies the
    // bare name in this module. Dashboard callers import the alias.
    BabyStepDetail, BabyStepRef, ConceptNodeRef, DecompositionTreeSnapshot,
    PhaseDetail as DashboardPhaseDetail, PhaseNodeRef, PreviewAvailableRow, StepNodeRef,
    // Tier 3 partial (session s-84064) — drawer DTOs consumed by
    // elai-dashboard::commands. Zero new SsbActionType variants; all DTOs
    // are typed views over existing payload structs.
    AgentCheckpointEntry, Bm25StatsSnapshot, ContextBudgetSnapshot,
    DairTraceEntry, DkgStatsSnapshot, FailureTaxonomyEntry, FreshnessState,
    GgSignalEntry, GrcBudgetEntry, HookVerdictEntry, KnowledgeReadinessSnapshot,
    MistakeBookEntry, PageRankEntry, QueryTierStats, SessionHandoffEntry,
    ShortCoderEffectEntry, TokenReclamationStats, ValidationTierEntry,
    VectorStatsSnapshot, VramTierSnapshot,
    // Tier 3 complete (session s-456340) — Research drawer DTOs. Typed
    // views over existing payload structs; zero new SsbActionType variants.
    ResearchBudgetSnapshot, ResearchStepEntry, SourceTierDistribution,
    // B-22 Phase 4G — auto-advance counters DTO consumed by dashboard-v2.
    AutoAdvanceStats,
    // B-39 chat reflect pipeline — reflect state snapshot DTO.
    ReflectState,
};
pub use task_runner::{
    detect_scaffold_template, emit_integration_checkpoint_for_test,
    enforce_file_read_budget_guard, evaluate_post_execution_block_found, is_web_research_task,
    is_web_scaffold_task, new_dev_server_registry, project_dir_is_web,
    serialize_decomposition_for_grounding, start_task_runner,
    AgentPoolBabyStepExecutor, BabyStepExecutionCtx, BabyStepExecutionResult, BabyStepExecutor,
    CancelHandle, CwdProjectDirResolver, DefaultWebResearchRunner, DevServerLauncher,
    DevServerRegistry, FileReadBudgetVerdict, LaunchedPreview, ProjectDirResolver,
    RuleEvaluator, ScaffoldApplier, SsbWebResearchStepEmitter, SyntheticBabyStepExecutor,
    TaskRunnerConfig, TaskRunnerError, WebResearchRunner, WebResearchSummary,
    WebValidatorLauncher, AGENT_FAILED_CATEGORY, BABY_STEP_PRE_GATE_ACTION,
    DEFAULT_BABY_STEP_TIMEOUT_MS, DEFAULT_DEV_SERVER_COMMAND, DEFAULT_DEV_SERVER_PORT,
    DEFAULT_ROOT_QUESTION_MAX_CHARS, DEFAULT_SCAFFOLD_TEMPLATE, DEFAULT_SOURCE_TIERS_TOML,
    DETERMINISTIC_AUDIT_CATEGORY, DEV_SERVER_FAILED_CATEGORY,
    ELAI_DISABLE_SHORTCODER_POSTOOLUSE_ENV, INDEXING_SUMMARY_CATEGORY,
    INTEGRATION_CHECKPOINT_FAILED_CATEGORY, INTEGRATION_CHECKPOINT_SUCCESS_CATEGORY,
    MAX_BABY_STEPS_PER_TASK, POST_EXECUTION_RULES_AUDIT_STEP, PREVIEW_SLOT_LIVE,
    RULE_VIOLATION_CATEGORY, RULE_VIOLATION_POST_CATEGORY, RUNNER_AGENT_ID,
    SPIRAL_HARDSTOP_CATEGORY, WEB_RESEARCH_FAILED_CATEGORY, WEB_SCAFFOLD_FAILED_CATEGORY,
    WEB_SCAFFOLD_TEMPLATE_ASTRO, WEB_SCAFFOLD_TEMPLATE_REACT, WEB_SCAFFOLD_TEMPLATE_SVELTE,
    WEB_SCAFFOLD_TEMPLATE_VANILLA,
};
pub use rule_loader::load_seed_rules_from_dir;
pub use supervision::{
    EscalationAction, SupervisionError, SupervisionManager, SupervisionOutcome, SupervisionPolicy,
};
// B-24 Phase 4I (Group C1) — top-level media-validator surface. Only the
// three spec-mandated entry types are re-exported here; the deeper
// surface lives under `elai_orchestrator::media_validator::*`.
pub use media_validator::{MediaValidationRequest, MediaValidationResult, Verdict};

#[cfg(test)]
pub(crate) fn test_env_lock() -> std::sync::MutexGuard<'static, ()> {
    static LOCK: std::sync::OnceLock<std::sync::Mutex<()>> = std::sync::OnceLock::new();
    LOCK.get_or_init(|| std::sync::Mutex::new(()))
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
}
