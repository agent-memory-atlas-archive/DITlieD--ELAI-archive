//! Golden-file lock for SsbActionType::as_sql_str() wire tags.
//!
//! Enforces the SSB wire-protocol invariant that the set of action_type strings
//! emitted by the orchestrator never silently shrinks, gets renamed, or has its
//! ordering relative to the lock file scrambled.
//!
//! Three tests:
//!   - `action_type_lock_golden_matches_runtime` -- byte-equality between
//!     the alphabetically-sorted as_sql_str() values and the body lines of
//!     tests/action_type_lock.golden.
//!   - `action_type_lock_count_is_locked` -- runtime variant count equals
//!     `TOTAL_VARIANTS`, catching renames that preserve count.
//!   - `regen_golden_file` (#[ignore]'d helper) -- rewrites the golden from
//!     the current runtime values. Run with `--ignored --nocapture` after a
//!     deliberate, reviewed schema change.
//!
//! Enumeration strategy: ALL_VARIANTS is a hand-locked array in declaration
//! order. `exhaustive_match_guard` is an internal `match` on `SsbActionType`
//! whose every arm returns `()`; rustc requires exhaustivity, so adding a new
//! variant to the enum without updating this test produces a hard compile
//! error pointing back here. No `strum` dep needed and no mutation of
//! ssb.rs (lockstep file -- kept READ ONLY by E-45 SP2 scope).
//!
//! Append-only contract -- see the header of action_type_lock.golden.

use std::collections::BTreeSet;
use std::fs;
use std::path::PathBuf;

use elai_orchestrator::ssb::SsbActionType;

/// Total number of SsbActionType variants currently locked. Bumping this
/// requires regenerating action_type_lock.golden in the SAME commit; the
/// `_count_is_locked` test below holds the line.
const TOTAL_VARIANTS: usize = 288;

/// All SsbActionType variants in enum DECLARATION order. Append-only -- new
/// variants go at the end of this array AND at the end of the enum AND get a
/// new as_sql_str arm AND get a new from_str arm AND get serde rename shims
/// where needed (see ssb.rs lockstep comments at lines 39, 119, 132, 145, etc.).
///
/// This is the 8th lockstep site (1-7 documented in ssb.rs file header).
#[rustfmt::skip]
const ALL_VARIANTS: [SsbActionType; TOTAL_VARIANTS] = [
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
    SsbActionType::WorkloadProfile,
    SsbActionType::DeathEvent,
    SsbActionType::RecoveryEvent,
    SsbActionType::SupervisionExhausted,
    SsbActionType::SpiralDetected,
    SsbActionType::ContextReduction,
    SsbActionType::BranchMerge,
    SsbActionType::MistakeEntry,
    SsbActionType::ResearchStep,
    SsbActionType::ResearchDecomposeLevel,
    SsbActionType::WebResearchStep,
    SsbActionType::PreviewAvailable,
    SsbActionType::InterventionRequest,
    SsbActionType::InterventionResponse,
    SsbActionType::TaskSubmitted,
    SsbActionType::HookEvent,
    SsbActionType::ProviderSignals,
    SsbActionType::SearchTriggerFired,
    SsbActionType::VramTierDetected,
    SsbActionType::IndexingProgress,
    SsbActionType::DkgReady,
    SsbActionType::TieredAnalysis,
    SsbActionType::ShortCoderCompression,
    SsbActionType::ToolDispatch,
    SsbActionType::ValidationTier,
    SsbActionType::FailureClassified,
    SsbActionType::GitCommit,
    SsbActionType::DairaTrace,
    SsbActionType::SystemProbe,
    SsbActionType::FeasibilityReport,
    SsbActionType::NovelStepDetected,
    SsbActionType::StubRun,
    SsbActionType::GateDecision,
    SsbActionType::TaskSplit,
    SsbActionType::EscalationFired,
    SsbActionType::ModeChanged,
    SsbActionType::ProposalGenerated,
    SsbActionType::ProposalApplied,
    SsbActionType::ProposalRejected,
    SsbActionType::MsiUpdate,
    SsbActionType::ConfigArchived,
    SsbActionType::ProfileSelected,
    SsbActionType::FsmTransition,
    SsbActionType::MicroReflection,
    SsbActionType::TelescopeCollapse,
    SsbActionType::ConfigWarning,
    SsbActionType::ConstraintsInjected,
    SsbActionType::InterventionResolved,
    SsbActionType::DkgQuerierSwapped,
    SsbActionType::SteeringComputed,
    SsbActionType::SteeringComputeFailed,
    SsbActionType::McpServerConnected,
    SsbActionType::McpToolCalled,
    SsbActionType::McpServerDisconnected,
    SsbActionType::ChatMessage,
    SsbActionType::ChatResponse,
    SsbActionType::ExecutionModeChanged,
    SsbActionType::ContextBudgetComputed,
    SsbActionType::SsbFilteredByOverlap,
    SsbActionType::HookLatencyViolation,
    SsbActionType::ResearchPathChosen,
    SsbActionType::GroundingCacheEvent,
    SsbActionType::CircuitBreakerStateChange,
    SsbActionType::BubbleUpTyped,
    SsbActionType::IntegrationCheckpointSuccess,
    SsbActionType::DagRepairRound,
    SsbActionType::ConceptDecompositionNode,
    SsbActionType::EvidenceGateEvaluated,
    SsbActionType::SupervisionEvent,
    SsbActionType::BspSuperstep,
    SsbActionType::AgentLifecycle,
    SsbActionType::AntiSpiralHit,
    SsbActionType::CloudSpend,
    SsbActionType::EscalateToCloud,
    SsbActionType::DataTableGrew,
    SsbActionType::L3Proposal,
    SsbActionType::ConvergenceStagnant,
    SsbActionType::ConfigOscillation,
    SsbActionType::RegressionDetected,
    SsbActionType::EvolutionWithheld,
    SsbActionType::ReflectTriaged,
    SsbActionType::ReflectCandidate,
    SsbActionType::ReflectAuditVerdict,
    SsbActionType::ReflectRefined,
    SsbActionType::MediaValidationStarted,
    SsbActionType::MediaTier1Complete,
    SsbActionType::MediaTier2Complete,
    SsbActionType::RenderComplete,
    SsbActionType::VisualFeedbackIteration,
    SsbActionType::MediaHumanReviewRequired,
    SsbActionType::GenerationModeSelected,
    SsbActionType::StreamingEarlyTerminated,
    SsbActionType::PipelinePreAssembled,
    SsbActionType::DependencyResolved,
    SsbActionType::MicroStepCreated,
    SsbActionType::LanguageGrammarStatus,
    SsbActionType::SymbolQuery,
    SsbActionType::MinifiedFileChunked,
    SsbActionType::IndexCheckpoint,
    SsbActionType::IndexResume,
    SsbActionType::IndexCancel,
    SsbActionType::LockAcquired,
    SsbActionType::LockContested,
    SsbActionType::SymbolGraphUpdated,
    SsbActionType::IdeaFramingStarted,
    SsbActionType::ProjectClassClassified,
    SsbActionType::CriticRoundComplete,
    SsbActionType::IdeaFramed,
    SsbActionType::TradeoffCardEmitted,
    SsbActionType::EntryPointsDetected,
    SsbActionType::ArtifactsIndexed,
    SsbActionType::ArtifactSearched,
    SsbActionType::ArtifactStale,
    SsbActionType::AmbiguityScored,
    SsbActionType::AmbiguityFloorFailed,
    SsbActionType::AmbiguityStreakAdvanced,
    SsbActionType::CosmosRefreshStarted,
    SsbActionType::CosmosArtifactUpdated,
    SsbActionType::CosmosArtifactDecayed,
    SsbActionType::CosmosGroundingUsed,
    SsbActionType::PerLevelAuditStarted,
    SsbActionType::PerLevelAuditVerdict,
    SsbActionType::PerLevelAuditCascade,
    SsbActionType::RuntimeFallback,
    SsbActionType::RuntimeHealthChanged,
    SsbActionType::CliSubprocessSpawned,
    SsbActionType::CliSubprocessKilled,
    SsbActionType::MlxSubprocessSpawned,
    SsbActionType::MlxSubprocessKilled,
    SsbActionType::LlamaCppSubprocessSpawned,
    SsbActionType::LlamaCppSubprocessKilled,
    SsbActionType::MlxApplePlatformDetected,
    SsbActionType::LlamaCppEpDetected,
    SsbActionType::RuntimeChainBuilt,
    SsbActionType::DeepPoolCapped,
    SsbActionType::RrfQualityGateFired,
    SsbActionType::HydeSkippedCodeQuery,
    SsbActionType::RrfComponentWeights,
    SsbActionType::GrcStepEmitted,
    SsbActionType::GroundingSweepResult,
    SsbActionType::CsqEvaluated,
    SsbActionType::AuditorRubberStampRejected,
    SsbActionType::HydeQuery,
    SsbActionType::VectorCacheHitRate,
    SsbActionType::QueryTrace,
    SsbActionType::PhaseNodeReady,
    SsbActionType::StepNodeReady,
    SsbActionType::BabyStepReady,
    SsbActionType::DagRepairEscalated,
    SsbActionType::AgentStatusChanged,
    SsbActionType::StagnationSpinningDetected,
    SsbActionType::StagnationOscillationDetected,
    SsbActionType::LateralThinkingActivated,
    SsbActionType::GoalDriftWarning,
    SsbActionType::WatchdogTimeout,
    SsbActionType::ChangeDecision,
    SsbActionType::SearchModeChanged,
    SsbActionType::RedactionApplied,
    SsbActionType::LateralThinkingSucceeded,
    SsbActionType::LateralThinkingExhausted,
    SsbActionType::GoalDriftBlock,
    SsbActionType::WipSquashComplete,
    SsbActionType::AlternativeExplored,
    SsbActionType::FeasibilityEscalated,
    SsbActionType::BranchForked,
    SsbActionType::BranchMerged,
    SsbActionType::BranchClosed,
    SsbActionType::PlanSkeleton,
    SsbActionType::CompiledPipelineFired,
    SsbActionType::TemplateCrystallized,
    SsbActionType::BackgroundTaskStarted,
    SsbActionType::BackgroundTaskComplete,
    SsbActionType::DecompileTriggered,
    SsbActionType::FutilityDetected,
    SsbActionType::FutilityOutputStagnation,
    SsbActionType::WiringEvidence,
    SsbActionType::WiringViolation,
    SsbActionType::DeterminismDrift,
    SsbActionType::MutationSurvived,
    SsbActionType::IntentConfirmed,
    SsbActionType::MrViolated,
    SsbActionType::SurgicalPlanCreated,
    SsbActionType::SurgicalLocWarning,
    SsbActionType::OracleCacheHit,
    SsbActionType::OracleCacheMiss,
    SsbActionType::ProjectDetectionSignal,
    SsbActionType::SkeletonGenerated,
    SsbActionType::SkeletonFlushed,
    SsbActionType::MiniCriticVerdict,
    SsbActionType::StructuralRetryStageAttempt,
    SsbActionType::BridgeFileGenerated,
    SsbActionType::ArchiveEntryAdded,
    SsbActionType::ArchiveEntryRetired,
    SsbActionType::TemplatePromoted,
    SsbActionType::TemplateCandidateRejected,
    SsbActionType::MemoryFactExtracted,
    SsbActionType::MemoryFactSuperseded,
    SsbActionType::PpmScored,
    SsbActionType::PpmGatingFailed,
    SsbActionType::ChatReflectionCompleted,
    SsbActionType::MemoryEvolutionRun,
    SsbActionType::MemoryNoteInserted,
    SsbActionType::MemoryNoteEvolved,
    SsbActionType::MemoryConflictDetected,
    SsbActionType::CoreBlockReplaced,
    SsbActionType::VerifierConsensusPass,
    SsbActionType::VerifierConsensusFail,
    SsbActionType::TripwireFired,
    SsbActionType::TemplateAutoReverted,
    SsbActionType::PrmBenchRunCompleted,
    SsbActionType::BaiArmUpdated,
    SsbActionType::HeldOutTaskAdded,
    SsbActionType::HeldOutTaskUsed,
    SsbActionType::CanaryRunCompleted,
    SsbActionType::CanaryFamilyEvaluated,
    SsbActionType::LockedKeysAuditRun,
    SsbActionType::LockedKeysViolationDetected,
    SsbActionType::TaskAdvanced,
    SsbActionType::AgentlessShortcutTaken,
    SsbActionType::GoalDriftFired,
    SsbActionType::ClaimRouted,
    SsbActionType::ModeRoutingBenchRunCompleted,
    SsbActionType::CheckerResultEmitted,
    SsbActionType::AdversaryGenerated,
    SsbActionType::ConsensusReevaluatedWithCounter,
    SsbActionType::BabyStepDispatched,
    SsbActionType::ConformalTauComputed,
    SsbActionType::ConformalRefusalDecided,
    SsbActionType::NoveltyClassified,
    SsbActionType::ObservableEffectVerified,
    SsbActionType::RefusalRateSnapshot,
    SsbActionType::RefusalSlaBreach,
    SsbActionType::ConformalValidationCompleted,
    SsbActionType::ArcBenchmarkCompleted,
    SsbActionType::ToolGated,
    SsbActionType::ToolEager,
    SsbActionType::ToolLazyResolved,
    SsbActionType::ToolDenied,
    SsbActionType::QuantBackendSelected,
    SsbActionType::KvCacheDtypeSoftwareSim,
    SsbActionType::AwqCheckpointBuilt,
    SsbActionType::CliMcpDispatch,
    SsbActionType::CliMcpServerStarted,
    SsbActionType::CliMcpServerStopped,
    SsbActionType::FirecrackerRestoreSlow,
    SsbActionType::SnapshotUniquenessRegenerated,
    SsbActionType::SnapshotEvicted,
    SsbActionType::SandboxBackendSelected,
    SsbActionType::AgentForkedFromSnapshot,
    SsbActionType::ResearcherNetEgressBlocked,
    SsbActionType::McpSchemaPinViolation,
    SsbActionType::McpSanitizationRuleApplied,
    SsbActionType::McpBudgetEnforced,
    SsbActionType::McpLegacySseTransport,
    SsbActionType::McpLlmGuardBlocked,
    SsbActionType::AnthropicGatewayDetected,
    SsbActionType::MemoryTierAssigned,
    SsbActionType::MemoryQueryFiltered,
    SsbActionType::SafetyCriticalBlocked,
    SsbActionType::ArgvDestructiveDenied,
    SsbActionType::ArgvApprovalRequired,
    SsbActionType::TrustDialogHookDeferred,
    SsbActionType::TrustDialogEnvDeferred,
    SsbActionType::McpPinVersionRefused,
    SsbActionType::AuditorLineageCollision,
    SsbActionType::UntrustedMemoryWriteBlocked,
    SsbActionType::IncidentRecordCreated,
    SsbActionType::FitmCalibrationApplied,
    SsbActionType::DeepModeActivated,
    SsbActionType::DeepModeEscalationChosen,
    SsbActionType::YarnRefused,
];

/// Compile-time exhaustiveness check: if a new variant lands in ssb.rs without
/// being added to ALL_VARIANTS, this match still compiles (it is exhaustive on
/// the enum), but ALL_VARIANTS will be shorter than the runtime count and the
/// `_count_is_locked` test fires. Inverse: removing a variant from the enum
/// without removing it from ALL_VARIANTS produces an "unknown variant" build
/// error at this match site. Either direction surfaces drift loudly.
#[allow(dead_code)]
fn exhaustive_match_guard(v: SsbActionType) {
    match v {
        SsbActionType::FileWrite => (),
        SsbActionType::FileRead => (),
        SsbActionType::ToolResult => (),
        SsbActionType::TestOutcome => (),
        SsbActionType::Finding => (),
        SsbActionType::Claim => (),
        SsbActionType::Checkpoint => (),
        SsbActionType::ModelSwap => (),
        SsbActionType::PhaseTransition => (),
        SsbActionType::KnowledgeDegradation => (),
        SsbActionType::IsolationMetric => (),
        SsbActionType::PlanUpdate => (),
        SsbActionType::ReplanRequest => (),
        SsbActionType::StubReference => (),
        SsbActionType::ReasoningTrace => (),
        SsbActionType::PhaseComplete => (),
        SsbActionType::MemoryMonitoring => (),
        SsbActionType::WorkloadProfile => (),
        SsbActionType::DeathEvent => (),
        SsbActionType::RecoveryEvent => (),
        SsbActionType::SupervisionExhausted => (),
        SsbActionType::SpiralDetected => (),
        SsbActionType::ContextReduction => (),
        SsbActionType::BranchMerge => (),
        SsbActionType::MistakeEntry => (),
        SsbActionType::ResearchStep => (),
        SsbActionType::ResearchDecomposeLevel => (),
        SsbActionType::WebResearchStep => (),
        SsbActionType::PreviewAvailable => (),
        SsbActionType::InterventionRequest => (),
        SsbActionType::InterventionResponse => (),
        SsbActionType::TaskSubmitted => (),
        SsbActionType::HookEvent => (),
        SsbActionType::ProviderSignals => (),
        SsbActionType::SearchTriggerFired => (),
        SsbActionType::VramTierDetected => (),
        SsbActionType::IndexingProgress => (),
        SsbActionType::DkgReady => (),
        SsbActionType::TieredAnalysis => (),
        SsbActionType::ShortCoderCompression => (),
        SsbActionType::ToolDispatch => (),
        SsbActionType::ValidationTier => (),
        SsbActionType::FailureClassified => (),
        SsbActionType::GitCommit => (),
        SsbActionType::DairaTrace => (),
        SsbActionType::SystemProbe => (),
        SsbActionType::FeasibilityReport => (),
        SsbActionType::NovelStepDetected => (),
        SsbActionType::StubRun => (),
        SsbActionType::GateDecision => (),
        SsbActionType::TaskSplit => (),
        SsbActionType::EscalationFired => (),
        SsbActionType::ModeChanged => (),
        SsbActionType::ProposalGenerated => (),
        SsbActionType::ProposalApplied => (),
        SsbActionType::ProposalRejected => (),
        SsbActionType::MsiUpdate => (),
        SsbActionType::ConfigArchived => (),
        SsbActionType::ProfileSelected => (),
        SsbActionType::FsmTransition => (),
        SsbActionType::MicroReflection => (),
        SsbActionType::TelescopeCollapse => (),
        SsbActionType::ConfigWarning => (),
        SsbActionType::ConstraintsInjected => (),
        SsbActionType::InterventionResolved => (),
        SsbActionType::DkgQuerierSwapped => (),
        SsbActionType::SteeringComputed => (),
        SsbActionType::SteeringComputeFailed => (),
        SsbActionType::McpServerConnected => (),
        SsbActionType::McpToolCalled => (),
        SsbActionType::McpServerDisconnected => (),
        SsbActionType::ChatMessage => (),
        SsbActionType::ChatResponse => (),
        SsbActionType::ExecutionModeChanged => (),
        SsbActionType::ContextBudgetComputed => (),
        SsbActionType::SsbFilteredByOverlap => (),
        SsbActionType::HookLatencyViolation => (),
        SsbActionType::ResearchPathChosen => (),
        SsbActionType::GroundingCacheEvent => (),
        SsbActionType::CircuitBreakerStateChange => (),
        SsbActionType::BubbleUpTyped => (),
        SsbActionType::IntegrationCheckpointSuccess => (),
        SsbActionType::DagRepairRound => (),
        SsbActionType::ConceptDecompositionNode => (),
        SsbActionType::EvidenceGateEvaluated => (),
        SsbActionType::SupervisionEvent => (),
        SsbActionType::BspSuperstep => (),
        SsbActionType::AgentLifecycle => (),
        SsbActionType::AntiSpiralHit => (),
        SsbActionType::CloudSpend => (),
        SsbActionType::EscalateToCloud => (),
        SsbActionType::DataTableGrew => (),
        SsbActionType::L3Proposal => (),
        SsbActionType::ConvergenceStagnant => (),
        SsbActionType::ConfigOscillation => (),
        SsbActionType::RegressionDetected => (),
        SsbActionType::EvolutionWithheld => (),
        SsbActionType::ReflectTriaged => (),
        SsbActionType::ReflectCandidate => (),
        SsbActionType::ReflectAuditVerdict => (),
        SsbActionType::ReflectRefined => (),
        SsbActionType::MediaValidationStarted => (),
        SsbActionType::MediaTier1Complete => (),
        SsbActionType::MediaTier2Complete => (),
        SsbActionType::RenderComplete => (),
        SsbActionType::VisualFeedbackIteration => (),
        SsbActionType::MediaHumanReviewRequired => (),
        SsbActionType::GenerationModeSelected => (),
        SsbActionType::StreamingEarlyTerminated => (),
        SsbActionType::PipelinePreAssembled => (),
        SsbActionType::DependencyResolved => (),
        SsbActionType::MicroStepCreated => (),
        SsbActionType::LanguageGrammarStatus => (),
        SsbActionType::SymbolQuery => (),
        SsbActionType::MinifiedFileChunked => (),
        SsbActionType::IndexCheckpoint => (),
        SsbActionType::IndexResume => (),
        SsbActionType::IndexCancel => (),
        SsbActionType::LockAcquired => (),
        SsbActionType::LockContested => (),
        SsbActionType::SymbolGraphUpdated => (),
        SsbActionType::IdeaFramingStarted => (),
        SsbActionType::ProjectClassClassified => (),
        SsbActionType::CriticRoundComplete => (),
        SsbActionType::IdeaFramed => (),
        SsbActionType::TradeoffCardEmitted => (),
        SsbActionType::EntryPointsDetected => (),
        SsbActionType::ArtifactsIndexed => (),
        SsbActionType::ArtifactSearched => (),
        SsbActionType::ArtifactStale => (),
        SsbActionType::AmbiguityScored => (),
        SsbActionType::AmbiguityFloorFailed => (),
        SsbActionType::AmbiguityStreakAdvanced => (),
        SsbActionType::CosmosRefreshStarted => (),
        SsbActionType::CosmosArtifactUpdated => (),
        SsbActionType::CosmosArtifactDecayed => (),
        SsbActionType::CosmosGroundingUsed => (),
        SsbActionType::PerLevelAuditStarted => (),
        SsbActionType::PerLevelAuditVerdict => (),
        SsbActionType::PerLevelAuditCascade => (),
        SsbActionType::RuntimeFallback => (),
        SsbActionType::RuntimeHealthChanged => (),
        SsbActionType::CliSubprocessSpawned => (),
        SsbActionType::CliSubprocessKilled => (),
        SsbActionType::MlxSubprocessSpawned => (),
        SsbActionType::MlxSubprocessKilled => (),
        SsbActionType::LlamaCppSubprocessSpawned => (),
        SsbActionType::LlamaCppSubprocessKilled => (),
        SsbActionType::MlxApplePlatformDetected => (),
        SsbActionType::LlamaCppEpDetected => (),
        SsbActionType::RuntimeChainBuilt => (),
        SsbActionType::DeepPoolCapped => (),
        SsbActionType::RrfQualityGateFired => (),
        SsbActionType::HydeSkippedCodeQuery => (),
        SsbActionType::RrfComponentWeights => (),
        SsbActionType::GrcStepEmitted => (),
        SsbActionType::GroundingSweepResult => (),
        SsbActionType::CsqEvaluated => (),
        SsbActionType::AuditorRubberStampRejected => (),
        SsbActionType::HydeQuery => (),
        SsbActionType::VectorCacheHitRate => (),
        SsbActionType::QueryTrace => (),
        SsbActionType::PhaseNodeReady => (),
        SsbActionType::StepNodeReady => (),
        SsbActionType::BabyStepReady => (),
        SsbActionType::DagRepairEscalated => (),
        SsbActionType::AgentStatusChanged => (),
        SsbActionType::StagnationSpinningDetected => (),
        SsbActionType::StagnationOscillationDetected => (),
        SsbActionType::LateralThinkingActivated => (),
        SsbActionType::GoalDriftWarning => (),
        SsbActionType::WatchdogTimeout => (),
        SsbActionType::ChangeDecision => (),
        SsbActionType::SearchModeChanged => (),
        SsbActionType::RedactionApplied => (),
        SsbActionType::LateralThinkingSucceeded => (),
        SsbActionType::LateralThinkingExhausted => (),
        SsbActionType::GoalDriftBlock => (),
        SsbActionType::WipSquashComplete => (),
        SsbActionType::AlternativeExplored => (),
        SsbActionType::FeasibilityEscalated => (),
        SsbActionType::BranchForked => (),
        SsbActionType::BranchMerged => (),
        SsbActionType::BranchClosed => (),
        SsbActionType::PlanSkeleton => (),
        SsbActionType::CompiledPipelineFired => (),
        SsbActionType::TemplateCrystallized => (),
        SsbActionType::BackgroundTaskStarted => (),
        SsbActionType::BackgroundTaskComplete => (),
        SsbActionType::DecompileTriggered => (),
        SsbActionType::FutilityDetected => (),
        SsbActionType::FutilityOutputStagnation => (),
        SsbActionType::WiringEvidence => (),
        SsbActionType::WiringViolation => (),
        SsbActionType::DeterminismDrift => (),
        SsbActionType::MutationSurvived => (),
        SsbActionType::IntentConfirmed => (),
        SsbActionType::MrViolated => (),
        SsbActionType::SurgicalPlanCreated => (),
        SsbActionType::SurgicalLocWarning => (),
        SsbActionType::OracleCacheHit => (),
        SsbActionType::OracleCacheMiss => (),
        SsbActionType::ProjectDetectionSignal => (),
        SsbActionType::SkeletonGenerated => (),
        SsbActionType::SkeletonFlushed => (),
        SsbActionType::MiniCriticVerdict => (),
        SsbActionType::StructuralRetryStageAttempt => (),
        SsbActionType::BridgeFileGenerated => (),
        SsbActionType::ArchiveEntryAdded => (),
        SsbActionType::ArchiveEntryRetired => (),
        SsbActionType::TemplatePromoted => (),
        SsbActionType::TemplateCandidateRejected => (),
        SsbActionType::MemoryFactExtracted => (),
        SsbActionType::MemoryFactSuperseded => (),
        SsbActionType::PpmScored => (),
        SsbActionType::PpmGatingFailed => (),
        SsbActionType::ChatReflectionCompleted => (),
        SsbActionType::MemoryEvolutionRun => (),
        SsbActionType::MemoryNoteInserted => (),
        SsbActionType::MemoryNoteEvolved => (),
        SsbActionType::MemoryConflictDetected => (),
        SsbActionType::CoreBlockReplaced => (),
        SsbActionType::VerifierConsensusPass => (),
        SsbActionType::VerifierConsensusFail => (),
        SsbActionType::TripwireFired => (),
        SsbActionType::TemplateAutoReverted => (),
        SsbActionType::PrmBenchRunCompleted => (),
        SsbActionType::BaiArmUpdated => (),
        SsbActionType::HeldOutTaskAdded => (),
        SsbActionType::HeldOutTaskUsed => (),
        SsbActionType::CanaryRunCompleted => (),
        SsbActionType::CanaryFamilyEvaluated => (),
        SsbActionType::LockedKeysAuditRun => (),
        SsbActionType::LockedKeysViolationDetected => (),
        SsbActionType::TaskAdvanced => (),
        SsbActionType::AgentlessShortcutTaken => (),
        SsbActionType::GoalDriftFired => (),
        SsbActionType::ClaimRouted => (),
        SsbActionType::ModeRoutingBenchRunCompleted => (),
        SsbActionType::CheckerResultEmitted => (),
        SsbActionType::AdversaryGenerated => (),
        SsbActionType::ConsensusReevaluatedWithCounter => (),
        SsbActionType::BabyStepDispatched => (),
        SsbActionType::ConformalTauComputed => (),
        SsbActionType::ConformalRefusalDecided => (),
        SsbActionType::NoveltyClassified => (),
        SsbActionType::ObservableEffectVerified => (),
        SsbActionType::RefusalRateSnapshot => (),
        SsbActionType::RefusalSlaBreach => (),
        SsbActionType::ConformalValidationCompleted => (),
        SsbActionType::ArcBenchmarkCompleted => (),
        SsbActionType::ToolGated => (),
        SsbActionType::ToolEager => (),
        SsbActionType::ToolLazyResolved => (),
        SsbActionType::ToolDenied => (),
        SsbActionType::QuantBackendSelected => (),
        SsbActionType::KvCacheDtypeSoftwareSim => (),
        SsbActionType::AwqCheckpointBuilt => (),
        SsbActionType::CliMcpDispatch => (),
        SsbActionType::CliMcpServerStarted => (),
        SsbActionType::CliMcpServerStopped => (),
        SsbActionType::FirecrackerRestoreSlow => (),
        SsbActionType::SnapshotUniquenessRegenerated => (),
        SsbActionType::SnapshotEvicted => (),
        SsbActionType::SandboxBackendSelected => (),
        SsbActionType::AgentForkedFromSnapshot => (),
        SsbActionType::ResearcherNetEgressBlocked => (),
        SsbActionType::McpSchemaPinViolation => (),
        SsbActionType::McpSanitizationRuleApplied => (),
        SsbActionType::McpBudgetEnforced => (),
        SsbActionType::McpLegacySseTransport => (),
        SsbActionType::McpLlmGuardBlocked => (),
        SsbActionType::AnthropicGatewayDetected => (),
        SsbActionType::MemoryTierAssigned => (),
        SsbActionType::MemoryQueryFiltered => (),
        SsbActionType::SafetyCriticalBlocked => (),
        SsbActionType::ArgvDestructiveDenied => (),
        SsbActionType::ArgvApprovalRequired => (),
        SsbActionType::TrustDialogHookDeferred => (),
        SsbActionType::TrustDialogEnvDeferred => (),
        SsbActionType::McpPinVersionRefused => (),
        SsbActionType::AuditorLineageCollision => (),
        SsbActionType::UntrustedMemoryWriteBlocked => (),
        SsbActionType::IncidentRecordCreated => (),
        SsbActionType::FitmCalibrationApplied => (),
        SsbActionType::DeepModeActivated => (),
        SsbActionType::DeepModeEscalationChosen => (),
        SsbActionType::YarnRefused => (),
    }
}

fn golden_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("action_type_lock.golden")
}

/// Build the sorted (alphabetical by wire tag) list of `"NNNN <tag>"` lines
/// that the golden file body should contain -- header comment lines excluded.
fn runtime_body_lines() -> Vec<String> {
    let mut tags: Vec<&'static str> =
        ALL_VARIANTS.iter().map(|v| v.as_sql_str()).collect();
    tags.sort();
    tags.iter()
        .enumerate()
        .map(|(i, t)| format!("{:04} {}", i, t))
        .collect()
}

/// Split the golden file into (header_comments, body_lines). Comment lines
/// start with `#`; body lines are "<ordinal> <wire_tag>".
fn split_golden(text: &str) -> (Vec<String>, Vec<String>) {
    let mut header = Vec::new();
    let mut body = Vec::new();
    for raw in text.lines() {
        if raw.starts_with('#') || raw.is_empty() {
            header.push(raw.to_string());
        } else {
            body.push(raw.to_string());
        }
    }
    (header, body)
}

#[test]
fn action_type_lock_count_is_locked() {
    assert_eq!(
        ALL_VARIANTS.len(),
        TOTAL_VARIANTS,
        "TOTAL_VARIANTS const ({}) and ALL_VARIANTS array length ({}) drift. \
         A variant was added/removed without updating both. Update TOTAL_VARIANTS \
         and regen action_type_lock.golden via \
         `cargo test -p elai-orchestrator --test action_type_lock regen_golden_file -- --ignored --nocapture`.",
        TOTAL_VARIANTS,
        ALL_VARIANTS.len(),
    );

    // Defensive: also assert no duplicate wire tags. as_sql_str() must be
    // a 1:1 function on variants. A collision here would be a rename gone wrong.
    let mut seen: BTreeSet<&'static str> = BTreeSet::new();
    for v in ALL_VARIANTS.iter() {
        let tag = v.as_sql_str();
        assert!(
            seen.insert(tag),
            "duplicate wire tag '{}' produced by SsbActionType::as_sql_str -- \
             two variants map to the same SQL string (rename collision?)",
            tag
        );
    }
}

#[test]
fn action_type_lock_golden_matches_runtime() {
    let path = golden_path();
    let golden_text = match fs::read_to_string(&path) {
        Ok(t) => t,
        Err(e) => panic!(
            "cannot read golden file {}: {}. Regenerate via \
             `cargo test -p elai-orchestrator --test action_type_lock regen_golden_file -- --ignored --nocapture`.",
            path.display(),
            e
        ),
    };
    let (_header, golden_body) = split_golden(&golden_text);
    let runtime_body = runtime_body_lines();

    if golden_body == runtime_body {
        return;
    }

    // Build a structured diff: lines only in runtime (additions), lines
    // only in golden (removals/renames), and ordinal-shifted lines.
    let golden_set: BTreeSet<&str> = golden_body.iter().map(|s| s.as_str()).collect();
    let runtime_set: BTreeSet<&str> = runtime_body.iter().map(|s| s.as_str()).collect();
    let only_runtime: Vec<&&str> = runtime_set.difference(&golden_set).collect();
    let only_golden: Vec<&&str> = golden_set.difference(&runtime_set).collect();

    let mut msg = String::new();
    msg.push_str("SsbActionType golden-file drift detected.\n\n");
    if !only_runtime.is_empty() {
        msg.push_str(
            "Lines present in runtime but missing from golden \
             (additions OR renames -- confirm via PR review):\n",
        );
        for line in &only_runtime {
            msg.push_str("  + ");
            msg.push_str(line);
            msg.push('\n');
        }
    }
    if !only_golden.is_empty() {
        msg.push_str(
            "Lines present in golden but missing from runtime \
             (REMOVALS or RENAMES -- append-only contract violated unless \
             you also added #[serde(rename = ..)] shim):\n",
        );
        for line in &only_golden {
            msg.push_str("  - ");
            msg.push_str(line);
            msg.push('\n');
        }
    }
    if only_runtime.is_empty() && only_golden.is_empty() {
        msg.push_str(
            "Set-equal but ordering / ordinals differ -- alphabetical sort \
             invariant broken. This should be impossible unless the test \
             itself was edited.\n",
        );
    }
    msg.push_str("\nTo accept this change (after confirming it is NOT a rename or removal):\n");
    msg.push_str("  cargo test -p elai-orchestrator --test action_type_lock regen_golden_file -- --ignored --nocapture\n");
    msg.push_str(
        "Then review the diff in `git diff -- crates/orchestrator/tests/action_type_lock.golden` before committing.\n",
    );
    panic!("{}", msg);
}

/// Regenerate the golden file in place. `#[ignore]` so it never runs in CI.
/// Invoke manually after a deliberate SsbActionType addition:
///
///   cargo test -p elai-orchestrator --test action_type_lock \
///       regen_golden_file -- --ignored --nocapture
///
/// Review the resulting `git diff` BEFORE committing -- the diff is the audit
/// trail for the schema change.
#[test]
#[ignore]
fn regen_golden_file() {
    let path = golden_path();
    let mut out = String::new();
    out.push_str("# action_type_lock.golden -- SsbActionType wire-tag lock file\n");
    out.push_str("# Generated by: cargo test -p elai-orchestrator --test action_type_lock regen_golden_file -- --ignored --nocapture\n");
    out.push_str("# Format: <4-digit ordinal> <wire_tag>\n");
    out.push_str("# Ordinal is the index in the ALPHABETICALLY-sorted list of as_sql_str() values.\n");
    out.push_str(
        "# Alphabetical sort gives stable PR diffs: adding a variant only inserts new lines and shifts ordinals from the insertion point forward.\n",
    );
    out.push_str(
        "# Append-only contract: removing or renaming a wire tag requires adding a #[serde(rename = \"<old>\")] shim AND updating this file in the same commit.\n",
    );
    out.push_str(
        "# 8th lockstep site for SsbActionType (alongside enum / SQL CHECK / as_sql_str / from_str / SsbPayload / action_type() / variant tests).\n",
    );
    out.push_str(
        "# Last regenerated: rerun this test to refresh; review the diff before committing.\n",
    );
    out.push_str(&format!("# Total variants: {}\n", ALL_VARIANTS.len()));
    out.push_str("#\n");
    for line in runtime_body_lines() {
        out.push_str(&line);
        out.push('\n');
    }
    match fs::write(&path, out) {
        Ok(_) => {}
        Err(e) => panic!(
            "regen_golden_file: failed to write {}: {}",
            path.display(),
            e
        ),
    }
    eprintln!(
        "regen_golden_file: wrote {} runtime variant lines to {}",
        ALL_VARIANTS.len(),
        path.display()
    );
}
