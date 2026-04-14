pub mod agent_pool;
pub mod deterministic_auditor;
pub mod grc;
pub mod anti_spiral;
pub mod sandbox;
pub mod bsp;
mod bash;
pub mod checkpoint;
pub mod coherence_check;
pub mod dst;
pub mod context_budget;
pub mod dispatcher;
pub mod bash_validation;
pub mod branch_lock;
mod compact;
mod conversation;
mod file_ops;
pub mod evidence;
pub mod fsm;
pub mod gate;
mod hooks;
pub mod ipc;
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
mod remote;
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
pub mod mistake_book;
pub mod output_postprocessor;
pub mod ssb;
pub mod supervision;
pub mod validated_ssb;
pub mod stale_branch;
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
pub use tool_filter::{RuleCategory, ToolCategory};
pub use dispatcher::{
    BackendConfig, BackendMode, DispatchContext, DispatchError, build_dispatch_context,
    LOCAL_MODEL_MAX_CONTEXT, LOCAL_BUDGET_FRACTION,
};
pub use elai_core::{BootstrapPhase, BootstrapPlan};
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
pub use file_ops::{
    edit_file, glob_search, grep_search, read_file, write_file, EditFileOutput, GlobSearchOutput,
    GrepSearchInput, GrepSearchOutput, ReadFileOutput, StructuredPatchHunk, TextFilePayload,
    WriteFileOutput,
};
pub use hooks::{
    HookAbortSignal, HookEvent, HookProgressEvent, HookProgressReporter, HookRunResult, HookRunner,
};
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
pub use recovery_recipes::{
    attempt_recovery, recipe_for, EscalationPolicy, FailureScenario, RecoveryContext,
    RecoveryEvent, RecoveryRecipe, RecoveryResult, RecoveryStep,
};
pub use remote::{
    inherited_upstream_proxy_env, no_proxy_list, read_token, upstream_proxy_ws_url,
    RemoteSessionContext, UpstreamProxyBootstrap, UpstreamProxyState, DEFAULT_REMOTE_BASE_URL,
    DEFAULT_SESSION_TOKEN_PATH, DEFAULT_SYSTEM_CA_BUNDLE, NO_PROXY_HOSTS, UPSTREAM_PROXY_ENV_KEYS,
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
pub use session::{
    ContentBlock, ConversationMessage, MessageRole, Session, SessionCompaction, SessionError,
    SessionFork,
};
pub use stale_branch::{
    apply_policy, check_freshness, BranchFreshness, StaleBranchAction, StaleBranchEvent,
    StaleBranchPolicy,
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
pub use deterministic_auditor::{
    CheckKind, DeterministicAuditResult, DeterministicAuditor, DeterministicAuditorConfig,
    DeterministicFinding, DeterministicSeverity,
};
pub use agent_pool::{
    default_grc_config, AgentHandle, AgentMessage, AgentPool, AgentPoolConfig, AgentPoolError,
    AgentRole, AgentStatus, InProcessAgentPool, ModelPool, VRAM_MIN_BYTES,
};
pub use grc::{PipelineGrc, PipelineGrcError};
pub use output_postprocessor::{process as postprocess_output, OutputConfig};
pub use worker_boot::{
    Worker, WorkerEvent, WorkerEventKind, WorkerEventPayload, WorkerFailure, WorkerFailureKind,
    WorkerPromptTarget, WorkerReadySnapshot, WorkerRegistry, WorkerStatus, WorkerTrustResolution,
};
pub use ssb::{
    Ssb, SsbActionType, SsbConfig, SsbEntry, SsbError, SsbFileEntry,
    SsbPayload, SsbReader, SsbWriteRequest, MistakeEntryPayload,
};
pub use supervision::{
    EscalationAction, SupervisionError, SupervisionManager, SupervisionOutcome, SupervisionPolicy,
};

#[cfg(test)]
pub(crate) fn test_env_lock() -> std::sync::MutexGuard<'static, ()> {
    static LOCK: std::sync::OnceLock<std::sync::Mutex<()>> = std::sync::OnceLock::new();
    LOCK.get_or_init(|| std::sync::Mutex::new(()))
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
}
