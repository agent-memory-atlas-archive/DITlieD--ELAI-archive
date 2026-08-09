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


// === Tier-B Move 1: ssb.rs split into a module directory (behavior-preserving). ===
// Submodules live in src/ssb/; everything is re-exported here so every external
// path `elai_orchestrator::ssb::X` is unchanged. SCHEMA_VERSION stays in THIS file
// (literal const line) so the contract path-greps (e12, h51) remain green.
mod schema;
mod action_type;
mod payloads;
mod store;

pub use schema::*;
pub use action_type::*;
pub use payloads::*;
pub use store::*;

/// Canonical, append-only list of every `SsbActionType` variant (a representative
/// `Kind` for data-carrying variants). Single source of truth for the registry size:
/// `SCHEMA_VERSION` below and `action_type_lock::TOTAL_VARIANTS` both derive from it.
/// Append a new variant HERE (alongside the enum / as_sql_str / from_str / SQL CHECK /
/// SsbPayload / action_type() arms); the counters and the golden then update with no
/// hand-edited number, so parallel plans no longer collide on the counters
/// (Tier-B Move 2, Option A).
pub const SSB_ACTION_TYPES: &[SsbActionType] = &[
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
    SsbActionType::PerRoleDecodingApplied,
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
    SsbActionType::BaiPromotionDecision,
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
    // B-19 audit follow-up: the 9 pre-existing variants below (B-3 / B-4 /
    // B-7 era) shipped without being added to this lock array, causing the
    // exhaustive_match_guard to fail to compile. Adding them here in the
    // same edit that adds MarkerReport closes the drift atomically; the
    // golden file is regenerated alongside.
    // B-3 (SCHEMA_VERSION 98): AntiStalenessDrawer + RulesDrawer beacons.
    SsbActionType::DepAuditCompleted,
    SsbActionType::BaselineRunCompleted,
    SsbActionType::DeterministicAuditRun,
    SsbActionType::RuleTransition,
    // B-4 (SCHEMA_VERSION 99): canary drift snapshot.
    SsbActionType::CanaryDrift,
    // B-7 (SCHEMA_VERSION 100): producer-side wiring closure batch.
    SsbActionType::TreePositionChanged,
    SsbActionType::FailureRecorded,
    SsbActionType::PackageRefVerified,
    SsbActionType::TechStackDecided,
    // B-19 (SCHEMA_VERSION 101): per-agent-execution DAIRA marker aggregation.
    SsbActionType::MarkerReport,
    // B-25 (SCHEMA_VERSION 102): Phase 4J Empire integration. 5 atomic variants.
    SsbActionType::EmpireConnected,
    SsbActionType::EmpireDisconnected,
    SsbActionType::EmpireEventReceived,
    SsbActionType::EmpireCapabilityInvoked,
    SsbActionType::EmpireRepairProposed,
    // C-5 follow-on (SCHEMA_VERSION 103): cloud-mode hardening trio.
    SsbActionType::RouteBlocked,
    SsbActionType::WarnLimitReached,
    SsbActionType::CostEstimate,
    // C-36 (SCHEMA_VERSION 104): MuCoCo mutation operator dispatch.
    SsbActionType::MucocoMutationApplied,
    // D-3 closure (SCHEMA_VERSION 105): PpmUnavailable +
    // VerifierDisagreement + CrossContextDispatched. PpmUnavailable carried
    // atomically from D-2 (DECISION-1). VerifierDisagreement = D-4 tripwire
    // input. CrossContextDispatched = cross_context::run emission.
    SsbActionType::PpmUnavailable,
    SsbActionType::VerifierDisagreement,
    SsbActionType::CrossContextDispatched,
    // C-38 (SCHEMA_VERSION 107): pagerank production observability + dkg
    // rebuild lifecycle. PagerankRun emitted from knowledge pipeline Stage
    // 2; DkgRebuildStarted/Complete emitted from the CLI's
    // dkg_lifecycle::ensure_dkg_or_rebuild helper.
    SsbActionType::PagerankRun,
    SsbActionType::DkgRebuildStarted,
    SsbActionType::DkgRebuildComplete,
    // D-4 GAP-2 (SCHEMA_VERSION 108): concurrent-detector tripwire row.
    // Distinct wire variant from `TripwireFired` so D-26 diagnostics can
    // branch on the tag rather than join raw rows.
    SsbActionType::TripwireFiredConcurrent,
    // C-40 (SCHEMA_VERSION 109): GitHub PR adapter producer surface.
    // 4 atomic variants emitted by crate::github_pr::GhCliAdapter on
    // open / status / close / merge. Sensitive fields hashed via blake3.
    SsbActionType::GithubPrOpened,
    SsbActionType::GithubPrStatusFetched,
    SsbActionType::GithubPrClosed,
    SsbActionType::GithubPrMerged,
    // E-41 (SCHEMA_VERSION 135): GitHub PR review surface. 5 atomic
    // variants for review composition / comment polling / slopsquat /
    // semgrep / auto-merge guard. NO new AgentRole (cascade avoided per
    // E-41 brief — uses existing PerRoleRuntime routing).
    SsbActionType::PrReviewComposed,
    SsbActionType::PrCommentWatched,
    SsbActionType::PrPackageVerifyResult,
    SsbActionType::PrSemgrepGateResult,
    SsbActionType::PrAutoMergeBlocked,
    // D-5 closure (SCHEMA_VERSION 110): Thompson-sample beacon
    // (concurrently landed by session s-31708; lock-step entry).
    SsbActionType::ThompsonSampled,
    // C-41 (SCHEMA_VERSION 111): skills-driven workflow registry.
    // SkillTriggered + SkillExecuted pair around the registry's
    // top_k_skills lookup and the harness Skill tool's execute_skill
    // dispatch. See workflow/papers/scientific-workflow-interface.md.
    SsbActionType::SkillTriggered,
    SsbActionType::SkillExecuted,
    // D-6 closure (SCHEMA_VERSION 112): held-out promotion-gate +
    // refresh-event beacons. Land atomically with the held-out closure
    // pass; see workflow/reasoning/D-6-decisions.md.
    SsbActionType::HeldOutPromotionGate,
    SsbActionType::HeldOutRefresh,
    // C-42 (SCHEMA_VERSION 113): Semantic Gateway L2 RBAC zero-trust
    // beacons. Composed-Allow / composed-Deny / L3-pending trio emitted
    // by plugins::mcp_bridge::DefaultMcpBridge::dispatch_tool_call_with_authz.
    // Paper grounding arXiv:2604.25555 (MEDIUM reliability, single-author
    // preprint). The GatewayHitlPending variant exists but is DEFERRED-
    // TO-FOLLOWUP for L3 Crypto-HITL — no producer in session 2.
    SsbActionType::GatewayPolicyAllowed,
    SsbActionType::GatewayPolicyDenied,
    SsbActionType::GatewayHitlPending,
    // D-7 closure (SCHEMA_VERSION 114): cross-domain canary observation
    // beacons. ELAI design hypothesis — NanoFlux (arXiv:2509.23252)
    // provides empirical motivation only; no canary mechanism in the
    // paper itself. See workflow/reasoning/D-7-decisions.md.
    SsbActionType::CrossDomainRegression,
    SsbActionType::CanaryFamilyPass,
    SsbActionType::CanaryInsufficientData,
    // D-8 closure (SCHEMA_VERSION 115): cryptographic archive verify.
    // arxiv:2507.05619 (T1) + DGM §5 are supporting documentation;
    // neither paper specifies the SSB wire layout. See
    // workflow/reasoning/D-8-decisions.md.
    SsbActionType::ArchiveVerifyResult,
    // C-43 closure (SCHEMA_VERSION 116): Phase 4H scope-freeze
    // ratification + violation beacons. ELAI design hypothesis — NO
    // paper grounding. The source plan cited arXiv:2604.21725 (AEL)
    // but two independent WebFetches confirmed that paper is
    // Thompson-Sampling agent learning, NOT scope-freeze. See
    // workflow/sessions/s-25120/mistakes.md for the bad-cite analysis.
    SsbActionType::ScopeFreezeRatified,
    SsbActionType::ScopeFreezeViolated,
    // C-44 (SCHEMA_VERSION 117): Composer 2 paid-backend cost accrual.
    // ELAI design hypothesis — paper arXiv:2603.24477 grounds model
    // identity only; integration source-grounded via @cursor/sdk. See
    // workflow/papers/composer2-interface.md (shallow-cite-fit) and
    // workflow/sessions/s-25120/mistakes.md.
    SsbActionType::BackendCostAccrued,
    // C-45 (SCHEMA_VERSION 120): CANTANTE contrastive credit attribution.
    // Paper-grounded against arXiv:2605.13295 (Zehle 2026, Section 3.2 — K
    // rollouts NOT K^N, scalar c in [-1,+1], g=3 partition default). See
    // workflow/papers/cantante-credit-attribution.md.
    SsbActionType::AgentRoleDropped,
    SsbActionType::CreditAttributionRun,
    SsbActionType::CreditAttributionResult,
    // C-46 (SCHEMA_VERSION 121): Uno-Orchestra parsimonious agent routing
    // (selective delegation). Paper-grounded against arXiv:2605.05007 (Cui
    // et al 2026, Sections 5 + 6.1 — (model, primitive) action space, alpha
    // in [0,1] convex blend, S_max <= 0.10 cost-hack bound, direct-answer
    // lazy-mode cost zero). See workflow/papers/uno-orchestra-interface.md.
    SsbActionType::SelectiveDelegationDecision,
    SsbActionType::SelectiveDelegationEscalated,
    SsbActionType::SelectiveDelegationCostSaved,
    // C-47 (SCHEMA_VERSION 122): Case-Based Calibration of Tool-Use.
    // Paper-grounded against arXiv:2605.15041 (CAST). Three variants emitted
    // by `elai_plugins::mcp_bridge::DefaultMcpBridge`'s shared
    // consult_calibration + record_call helpers (called from BOTH
    // dispatch_tool_call_with_gate AND dispatch_tool_call_with_authz). See
    // workflow/papers/case-based-calibration.md +
    // workflow/reasoning/C-47-decisions.md.
    SsbActionType::ToolCallLogged,
    SsbActionType::ToolCalibrationHintEmitted,
    SsbActionType::ToolCalibrationDriftDetected,
    // C-49 (SCHEMA_VERSION 123): iterative-distillation experience module.
    // ELAI heuristic adaptation of arXiv:2605.14443 (Sayana, Todi, Jash
    // 2026). The full RL-trained prompter (Algorithm 1, GRPO + KL anchor)
    // is RUNPOD-PENDING. Three variants emitted by
    // `elai_orchestrator::experience::distill_fragments` + the wire-dark
    // post-audit hook in `task_runner`. See
    // workflow/papers/iterative-distillation-interface.md.
    SsbActionType::PromptPolicyDistillationRun,
    SsbActionType::PromptFragmentSelected,
    SsbActionType::PromptFragmentDiscarded,
    // D-14 (SCHEMA_VERSION 124): sycophancy probe row pair. ELAI
    // structural-defense adaptation of arXiv:2310.13548 (Sharma et al
    // 2024). See workflow/papers/sharma-sycophancy-2310.13548-interface.md.
    SsbActionType::SycophancyProbeDispatched,
    SsbActionType::SycophancyFlag,
    // D-23 (SCHEMA_VERSION 142): sycophancy detection benchmark.
    SsbActionType::SycophancyBenchRunCompleted,
    // D-24 (SCHEMA_VERSION 143): cross-domain canary regression
    // benchmark. NanoFlux arXiv:2509.23252 provides empirical motivation
    // only; no canary mechanism in the paper itself — D-24 is an ELAI
    // design hypothesis layered on top of D-7's run_canary substrate.
    SsbActionType::CanaryBenchRunCompleted,
    // D-25 (SCHEMA_VERSION 144): determinism + archive tamper-suite
    // benchmark. arxiv:2507.05619 (T1) provides integrity-checksum
    // motivation only; no tamper-suite mechanism in the paper itself —
    // D-25 is an ELAI design hypothesis driving D-8's verify_archive
    // IN-PROCESS across a 10-scenario JSON-manifest matrix with a
    // 5000ms per-scenario wall budget.
    SsbActionType::DeterminismTamperSuiteCompleted,
    // D-28 (SCHEMA_VERSION 145): adversarial example farming run.
    // arXiv:2504.06141 (Adv-RM) §3.2 farming loop. Host-side
    // scaffolding for AdvRmFarmer::run; live PPM scoring loop +
    // ppm_finetune.sh + dual bench_prmbench v1/v2 are RUNPOD-PENDING.
    // Paper-cite calibration: plan's "+5pp / 3-epoch / K>=2"
    // ungrounded — recalibrated to +1pp / 1 epoch / K=1 proxy>>gold
    // gap fallback per workflow/reasoning/D-28-decisions.md.
    SsbActionType::AdversarialFarmRunCompleted,
    // D-29 (SCHEMA_VERSION 146): hierarchical BAI round completion.
    // Wire-dark gated by ELAI_BAI_MODE=hierarchical AND
    // ELAI_BAI_ENABLE=1. arXiv:2111.06929 SUPPORTING EVIDENCE (HierTS;
    // regret minimization, NOT BAI; paper grounds the Beta-hyperprior
    // shape). arXiv:2106.10417 DIRECT GROUNDING (variance-dependent
    // BAI sample complexity bound H_var). Plan's "30-50% regret
    // reduction" claim UNGROUNDED for BAI; live baseline RUNPOD-PENDING.
    // Single-variant by D-19/D-23/D-24/D-25/D-28 precedent.
    SsbActionType::HierarchicalBaiRoundCompleted,
    // D-30 (SCHEMA_VERSION 147): calibration bootstrap warm-start
    // completion. Wire-dark gated by ELAI_CALIBRATION_BOOTSTRAP_ENABLE.
    // arXiv:2509.01455 UniCR COMPOSITION (consumes compute_tau_with_min).
    // arXiv:2109.07958 TruthfulQA DATA ORIGIN (corpus fixture; not
    // algorithm-source). Politis & Romano 1994 JASA Vol 89 §3 DIRECT
    // GROUNDING (stationary bootstrap; only active when SECONDARY runs).
    // Single-variant by D-19/D-23/D-24/D-25/D-28/D-29 precedent (11th
    // application of pattern-2); the SECONDARY block-bootstrap CI path
    // extends the payload via `Option` fields instead of a 2nd variant.
    // Live D-22 validation + 817-row corpus + block-bootstrap activation
    // are RUNPOD-PENDING.
    SsbActionType::CalibrationBootstrapCompleted,
    // C-50 (SCHEMA_VERSION 125): Contract-Coding STRUCTURAL contract
    // test scaffolding. Paper-grounded against arXiv:2604.13100 (Lin,
    // Zhao, Shi 2026; kernel K = <N, Sigma, Delta> §4.2). Full RL
    // Generator -> Discriminator pipeline (Algorithm 1) is
    // RUNPOD-PENDING; this lockstep entry covers the heuristic
    // scaffolder + discriminator row pair only. Citation discipline:
    // the headline "47%" is the Roguelike subset of Greenfield-5
    // (Table 1) and TIES Trae; the differentiator is structural
    // integrity (failures confined to Local Logic Errors) per
    // Section 5.2. See workflow/papers/contract-coding-interface.md.
    SsbActionType::ContractTestScaffolded,
    SsbActionType::ContractViolationDetected,
    // C-52 (SCHEMA_VERSION 126): Dafny Formal Verification Spike host-
    // side scaffolding (heuristic + offline path). Paper-grounded
    // against arXiv:2604.22601 (NL2VC). Single beacon variant —
    // DafnyVerifyCompleted carries tier_hash + dafny_pass +
    // udebug_pass + heal_iter + ts_ms. Live `dafny verify` execution
    // is RUNPOD-PENDING (local dev host lacks the dafny binary;
    // Gemma 4 BLOCKED per memory feedback_no_gemma4). See
    // workflow/papers/nl2vc-dafny-interface.md +
    // workflow/reasoning/C-52-decisions.md.
    SsbActionType::DafnyVerifyCompleted,
    // D-16 closure (SCHEMA_VERSION 127): durable TauUpdater audit row.
    // Paper-grounded against arXiv:2509.01455 (UniCR Eq.5).
    SsbActionType::CalibrationCorpusUpdated,
    // E-12 Tier-B (SCHEMA_VERSION 128): production fullstack codegen.
    // Framework profile selection + legacy linter config strip. Two-
    // variant atomic bump per the QUINTUPLE / 9-tuple lockstep
    // pattern (mod scaled to two variants). Wire-dark posture: live
    // 15-runner production-checklist invocation against
    // representative apps is RUNPOD-PENDING per
    // workflow/reasoning/E-12-tier-b-decisions.md. The 7 framework
    // manifests (Wasp / Encore / RedwoodSDK / T3 / Hono / FastAPI /
    // NextAppRouter) are partially shipped — 4 representative
    // archetypes (nextjs15 / svelte5 / fastapi-modern / hono) land
    // in this commit; the rest are RUNPOD-PENDING source-grounded
    // via paper-ingestor mode=source.
    SsbActionType::FrameworkProfileSelected,
    SsbActionType::LegacyConfigStripped,
    // D-18 closure (SCHEMA_VERSION 129): typed-ObservableEffect 4-variant
    // pack. Plan: docs/thefinalglance/D-plan/D-18_observable_effect_typed.md.
    // Anchors: Voyager arxiv:2305.16291 §3 (skill-with-measurable-effect),
    // Eureka arxiv:2310.12931 §3.3 (component-variation Goodhart check),
    // Coscientist arxiv:2304.05332 §4 (sandbox vs production observation).
    // The plan's arxiv:2412.16367 citation was REJECTED (unrelated
    // battery-thermal-runaway paper) per
    // workflow/reasoning/D-18-decisions.md DECISION-6.
    SsbActionType::ObservableEffectDispatched,
    SsbActionType::GoodhartRiskFlag,
    SsbActionType::SandboxObservation,
    SsbActionType::ProductionObservationProposal,
    // E-13 (SCHEMA_VERSION 130): TDD verifier-agent beacon family —
    // cycle entry + per-phase Red/Green observations. The plan
    // originally proposed 7 variants; scope-lean re-check trimmed
    // to the 3 essential signals. Composes with D-3's
    // ConsensusRule::VerifierKindHeterogeneous via
    // tdd_verifier::consensus::tdd_results_to_verifier_results.
    SsbActionType::TddCycleStarted,
    SsbActionType::TddTestFailed,
    SsbActionType::TddTestPassed,
    // Tranche B-7 triple (SCHEMA_VERSION 195): TDD verification gate
    // substrate. Pattern-2 (46th application). 4-arm Kind discriminator
    // covers PropertyGatePass / MutationGateSurvivorBlock / AntiMockBlock
    // / AssertionOnlyDiffBlock. See
    // workflow/reasoning/B7-triple-tranche-decisions.md.
    SsbActionType::TddVerificationEvent,
    // E-14 (SCHEMA_VERSION 131): browser-e2e agent beacon family.
    // 4 atomic variants — journey run entry + per-assertion + per-snapshot
    // rows. See workflow/reasoning/E-14-decisions.md.
    SsbActionType::BrowserE2eRunStarted,
    SsbActionType::BrowserE2eAssertionFailed,
    SsbActionType::BrowserE2eAssertionPassed,
    SsbActionType::BrowserE2eSnapshotCaptured,
    // E-18 (SCHEMA_VERSION 132): tool-use reliability gate beacon family.
    // 4 atomic variants — Pass / Reject verdict from
    // tool_reliability::ToolReliabilityGate::check, plus the spill writer
    // beacon and the retry escalation beacon. Reuse-heavy posture: composes
    // on D-11 RegistryCheck + E-35 SearchReplaceMatcher / SidecarXGrammarHook
    // + C-47 ToolCalibrationRegistry + C-42 AuthzPolicy + C-35 ToolGate
    // rather than re-implementing those surfaces. See
    // workflow/reasoning/E-18-decisions.md.
    SsbActionType::ToolReliabilityGatePassed,
    SsbActionType::ToolReliabilityGateRejected,
    SsbActionType::ToolCallSpilled,
    SsbActionType::ToolCallRetryEscalated,
    // E-37 (SCHEMA_VERSION 133): prompt-injection defense beacon family.
    // 9 atomic variants. Paper grounding arXiv:2403.14720 (Spotlight) +
    // arXiv:2507.15219 (PromptArmor) + arXiv:2503.18813 (CaMeL) +
    // arXiv:2406.13352 (AgentDojo) + arXiv:2502.06703 (cheap monitors) +
    // OWASP LLM Top 10 LLM01. See workflow/reasoning/E-37-decisions.md.
    SsbActionType::InjectionAttemptDetected,
    SsbActionType::InjectionAttemptBlocked,
    SsbActionType::SpotlightWrapped,
    SsbActionType::SpotlightBreached,
    SsbActionType::ToolDescriptionPinMismatch,
    SsbActionType::CapabilityGateRejected,
    SsbActionType::ProvenanceTagged,
    SsbActionType::PromptArmorVerdict,
    SsbActionType::InjectionFixtureReplayed,
    // E-37 U1: output sanitizer / classifier flagged beacon. Emitted by
    // conversation::run_post_tool_use_hook when strip_markdown_images
    // removed a non-allowlisted host OR classify_output flagged a
    // script / role-tag impersonation pattern. See
    // workflow/reasoning/E-37-decisions.md (U1).
    SsbActionType::OutputClassifierFlagged,
    // E-40 (SCHEMA_VERSION 134): orchestrator-rooted symbol-graph beacon family.
    // 3 atomic variants. Paper grounding deferred per
    // workflow/sessions/s-25120/mistakes.md cluster #6 (Stack Graphs +
    // GraphCoder + CodexGraph + RANGER + Glean Angle). See
    // workflow/reasoning/E-40-decisions.md.
    SsbActionType::SymbolGraphQueried,
    SsbActionType::SymbolGraphCacheLoaded,
    SsbActionType::LspOracleConsulted,
    // E-44 (SCHEMA_VERSION 136): durability checkpoint + idempotent dispatch +
    // compact-lock beacon family. 5 atomic variants emitted by
    // `CheckpointManager::write/read_latest`, `checkpoint_migrator::migrate_checkpoint`,
    // `IdempotentDispatcher` cache-hit path, and `CompactLock::try_acquire` contention.
    // See workflow/reasoning/E-44-decisions.md.
    SsbActionType::CheckpointWritten,
    SsbActionType::CheckpointResumed,
    SsbActionType::CheckpointMigrated,
    SsbActionType::IdempotentDispatchSkipped,
    SsbActionType::DurabilityLockBlocked,
    // E-45 (SCHEMA_VERSION 137): cross-language drift detector beacon family.
    // 3 atomic variants emitted by `cross_lang_drift::DriftDetector::diff_pair`
    // (per pair drift count + severity), `detect_pairs` (per repo scan), and
    // the sidecar SCHEMA_VERSION mismatch at the orchestrator <-> sidecar
    // boundary. See workflow/reasoning/E-45-decisions.md.
    SsbActionType::CrossLangDriftDetected,
    SsbActionType::CrossLangPairScanned,
    SsbActionType::SchemaVersionMismatch,
    // E-49 (SCHEMA_VERSION 138): coding-agent metric suite beacon family
    // (scope-lean ship). 5 atomic variants emitted by
    // `coding_metrics::collector::MetricsCollector::capture_metrics`,
    // `coding_metrics::swebench_pro::run_swebench_pro` (start + completion),
    // `coding_metrics::refinebench::run_refinebench`, and
    // `coding_metrics::aggregator::aggregate_session_metrics`. Live
    // SWE-Bench Pro + RefineBench transports are RUNPOD-PENDING; the
    // variant set is stable. See workflow/reasoning/E-49-decisions.md.
    SsbActionType::CodingAgentMetricsCaptured,
    SsbActionType::SweBenchProRunStarted,
    SsbActionType::SweBenchProRunCompleted,
    SsbActionType::RefineBenchScoreEmitted,
    SsbActionType::MetricsAggregationRun,
    // E-52 Tier-B (SCHEMA_VERSION 139): LSP-as-verifier diagnostic probe +
    // SCIP refresh beacon family. 4 atomic variants emitted by
    // `orchestrator::lsp_oracle::handshake` (JSON-RPC initialize +
    // initialized + shutdown + exit Tier-B path), the future PostToolUse
    // diagnostic-probe dispatcher (Tier-B+1), and the `.claude/tools/
    // scip_refresh.py` map-refresh wrapper. Live rust-analyzer/pylsp
    // handshakes are RUNPOD-PENDING; the variant set is stable. See
    // workflow/reasoning/E-52-decisions.md.
    SsbActionType::DiagnosticProbeDispatched,
    SsbActionType::DiagnosticProbeCompleted,
    SsbActionType::DiagnosticProbeBlock,
    SsbActionType::ScipRefreshCompleted,
    // E-53 (SCHEMA_VERSION 140): formal-verify pipeline boundary
    // beacon. Single atomic +1 bump per the C-52
    // DafnyVerifyCompleted single-variant precedent. Composes C-50
    // contract_coding (StructuralContract emission) + C-52
    // dafny_verify (invoke_dafny_cli dispatch) + new csq.rs
    // classifier (CounterfactualClass gating). Live Z3 / Kani /
    // Verus dispatch is RUNPOD-PENDING; Dafny dispatch reuses C-52.
    // Gemma-4 BLOCKED per memory feedback_no_gemma4 (substrate
    // inheritance from C-52). VerifierKind enum in
    // verification_result.rs extended atomically per the L3-lock
    // escape clause "New kinds land via plan-level schema bumps"
    // (Z3SmtCheck variant added; 4 -> 5 variants). See
    // workflow/reasoning/E-53-decisions.md.
    SsbActionType::FormalVerifierCompleted,
    // E-35 (SCHEMA_VERSION 141): structured-output substrate beacons.
    // 10 atomic variants for XGrammar / INSTRUCTOR / CodeAct /
    // SEARCH/REPLACE / cloud strict-schema / engine-refusal. Lean
    // ship per session-2 autonomous-mode brief: AgentRole::Runner
    // DEFERRED per E41_AGENTROLE_FROZEN_AT_17 (230 match-arm cascade
    // — CodeAct composes through existing role + RuntimeBackend);
    // Python sidecar tool_parser.py + XGrammarLogitsProcessor
    // RUNPOD-PENDING; SearchReplaceMatcher dispatcher tier-0 wire
    // DEFERRED to patch-N; DCCD (Task 9) DEFERRED behind env flag.
    // Composes on the already-shipped Rust substrate (StructuredMode,
    // ToolParserVariant, XGrammarRequest/Response, CodeActAction,
    // SearchReplaceTier1 at crates/ipc-types; SidecarXGrammarHook +
    // xgrammar_cache_key at crates/providers/src/runtime/
    // sidecar_xgrammar_hook.rs; SearchReplaceMatcher +
    // DelimiterCollisionGuard at crates/tools/src/search_replace.rs;
    // GrammarCompilePanel + commands_e35.rs in crates/dashboard-v2).
    // Inline cite ONLY codeact-2402.01030 per cite-discipline cluster
    // s-25120 (mistakes.md cite-incident cluster #13 for XGrammar /
    // JSONSchemaBench / CRANE / Diff-XYZ / "Think Inside the JSON" /
    // DCCD / "When Correct Isn't Usable" name-only references). See
    // workflow/reasoning/E-35-decisions.md.
    SsbActionType::XgrammarMaskApplied,
    SsbActionType::InstructorModeActivated,
    SsbActionType::GrammarModeActivated,
    SsbActionType::CodeactActionExecuted,
    SsbActionType::SearchReplaceTier1Hit,
    SsbActionType::SearchReplaceCollisionFallback,
    SsbActionType::CloudStrictSchemaApplied,
    SsbActionType::GrammarCacheLoaded,
    SsbActionType::StructuredEngineRejected,
    SsbActionType::LegacyJsonObjectRefused,
    // E-6 (SCHEMA_VERSION 148): MAST-grounded multi-agent failure-mode defenses.
    // Four atomic variants per workflow/reasoning/E-6-decisions.md DECISION-5.
    // arXiv:2503.13657 PRIMARY ANCHOR (Cemri et al. 2025, MAST). Citation
    // substitution from plan's bad 2406.12624 cite documented in DECISION-1
    // + gotcha E6_CITATION_2406_12624_ID_COLLISION.
    SsbActionType::AuditMastTagged,
    SsbActionType::IsoBudgetMeasured,
    SsbActionType::PersonaReanchored,
    SsbActionType::CrossFamilyUnavailable,
    // E-7 (SCHEMA_VERSION 149): darwinian self-improvement substrate.
    // SINGLE-VARIANT scope-lean per Pattern-2 (13th application). Paper
    // grounding: arXiv:2510.04399 Statistical Limits / Two-Gate
    // (NOT AlphaEvolve; canonical re-anchor) + arXiv:2305.10718
    // Discounted Thompson Sampling + arXiv:2506.13131 AlphaEvolve
    // MAP-Elites + arXiv:2507.05619 EST + arXiv:2507.19457 GEPA
    // background + OpenEvolve background + Krakovna spec-gaming
    // spreadsheet. REMOVED: arXiv:2602.12413 (wrong paper; see
    // workflow/reasoning/E-7-decisions.md DECISION-3). Wire-dark unless
    // ELAI_DARWINIAN_LOOP_ENABLE=1 AND cfg(feature = "darwinian-evolve").
    SsbActionType::DarwinianGenerationCompleted,
    // batch2-merge-collision recovery (SCHEMA_VERSION 150-156): 9 variants
    // re-applied after parallel-executor write collision in cluster s-25120
    // batch 2 (E-7 clobbered E-2 / E-3 / E-26 / E-28 prior writes). See
    // workflow/sessions/s-25120/completion-drive/audits/batch2-fixer-report.md.
    // E-3 (SCHEMA_VERSION 150): runtime handoff signal beacon.
    SsbActionType::RuntimeHandoffSignal,
    // E-26 (SCHEMA_VERSION 151): inference-engines 2026 pack.
    SsbActionType::SglangBackendRegistered,
    SsbActionType::XgrammarSchemaCompiled,
    SsbActionType::RuntimeColdStartObserved,
    // E-2 (SCHEMA_VERSION 152-153): streaming timing + UX pacing vote.
    SsbActionType::StreamingTimingRecorded,
    SsbActionType::UxPacingVoteRecorded,
    // E-28 (SCHEMA_VERSION 154-156): attention-kernels 2026 pack.
    SsbActionType::StreamingLlmEngaged,
    SsbActionType::SdpaBackendPinned,
    SsbActionType::YarnRopeScalingApplied,
    // E-8 (SCHEMA_VERSION 162): self-evolving prompts substrate.
    // SINGLE-VARIANT Pattern-2; OptimizerPhase discriminator on payload
    // covers 7 fire-points. Paper grounding: arXiv:2507.19457 GEPA +
    // arXiv:2310.03714 DSPy + arXiv:2309.03409 OPRO + arXiv:2211.01910
    // APE + arXiv:2405.10276 small-LM-as-optimizer limitation +
    // arXiv:2504.20355 LPO + arXiv:2602.00997 ETGPO. REMOVED:
    // arXiv:2403.12373 RankPrompt (WRONG_REGIME per paper-triager). See
    // workflow/reasoning/E-8-decisions.md.
    SsbActionType::PromptOptimizationCompleted,
    // E-10 (SCHEMA_VERSION 163): hostile-critic structural simplifications.
    // SINGLE-VARIANT Pattern-2; CriticTilePhase 9-arm discriminator on
    // payload covers 9 fire-points. Paper grounding: arXiv:2502.00674
    // Self-MoA + arXiv:2503.13657 MAST + arXiv:2510.01069 typed-CoT +
    // arXiv:2305.01219 ProAttack + arXiv:2401.05949 ICLAttack +
    // arXiv:2307.13702 Lanham + arXiv:2510.04040 FaithCoT-Bench.
    // See workflow/reasoning/E-10-decisions.md.
    SsbActionType::CriticDecisionEvent,
    // E-43 (SCHEMA_VERSION 164): agent skill systems telemetry.
    // SINGLE-VARIANT Pattern-2; SkillEventPhase 3-arm discriminator on
    // payload covers loaded / invoked / failed. Wire-dark unless
    // ELAI_SKILL_SYSTEM_ENABLE=1. Grounding: agentskills.io/specification +
    // snyk.io/blog/toxicskills + vercel.com/blog/agents-md-outperforms-skills.
    // See workflow/reasoning/E-43-decisions.md.
    SsbActionType::SkillSystemEvent,
    // E-9 (SCHEMA_VERSION 165): external agent-eval benchmark suite.
    // SINGLE-VARIANT Pattern-2 (17th application); BenchPhase 5-arm
    // discriminator on payload covers loaded / started / item_evaluated /
    // completed / aborted. suite_name string field discriminates across
    // the 11 external benchmark suites. Wire-dark unless
    // ELAI_EVAL_BENCHMARK_ENABLE=1. Paper grounding (7 verified):
    // arXiv:2509.16941 SWE-Bench Pro (primary, RAG-indexed) +
    // arXiv:2505.23419 SWE-bench-Live + arXiv:2511.22173 RefineBench +
    // arXiv:2604.05172 ClawsBench + arXiv:2311.12983 GAIA +
    // arXiv:2601.18137 DeepPlanning + arXiv:2506.07982 tau2. REMOVED
    // per workflow/reasoning/E-9-decisions.md DECISION-3: plan's "DS-CP
    // arXiv:2510.05566" + focused.io blog + "MetaMate".
    SsbActionType::BenchmarkRunObserved,
    // E-19 (SCHEMA_VERSION 166): reasoning quality probe.
    // SINGLE-VARIANT Pattern-2 (18th application); ProbeKind 5-arm
    // discriminator on payload covers l2_gate_decision /
    // l4_prm_lite_score / l4_lateral_kept / l4_ttc_budget_chosen /
    // l6_verifier_diversity. Wire-dark unless
    // ELAI_REASONING_QUALITY_PROBE_ENABLE=1. Paper grounding:
    // arXiv:2510.08308 First Try Matters + arXiv:2502.18581
    // Self-Certainty + arXiv:2510.01500 LToT + arXiv:2503.04412 AB-MCTS
    // + arXiv:2502.08788 Vanilla MAD + arXiv:2410.21333 Mind-Your-Step +
    // arXiv:2510.01069 Typed CoT + arXiv:2502.10954 + arXiv:2408.03314
    // + arXiv:2507.02778 Self-Correction Bench. DROPPED per
    // workflow/reasoning/E-19-decisions.md grounding-triage:
    // arXiv:2602.12113 ARLCP (WRONG_REGIME, training-time RL) +
    // arXiv:2307.13702 Lanham (WRONG_REGIME, CoT-text faithfulness).
    SsbActionType::ReasoningQualityProbe,
    // E-54 (SCHEMA_VERSION 167): eval-corpus repair pre-deploy.
    // SINGLE-VARIANT Pattern-2 (19th application); CorpusEventPhase 14-arm
    // discriminator on payload covers ingest_rejected_ngram /
    // ingest_rejected_cosine / shard_created / schema_violation_no_tests /
    // test_fails_on_reference / test_useless_passes_broken /
    // kor_orthogonality_violated / kor_orthogonality_passed /
    // sycophancy_framing_gap_detected / sandbagging_anomaly /
    // generator_rotation_violated / perf_delta_contamination_suspected /
    // dynamic_knowledge_qa_generated / prod_traffic_probe_injected.
    // Wire-dark unless ELAI_EVAL_CORPUS_ENABLE=1. See
    // workflow/reasoning/E-54-decisions.md.
    SsbActionType::EvalCorpusEvent,
    // E-55 (SCHEMA_VERSION 168): agentic RL + reward ladder substrate.
    // SINGLE-VARIANT Pattern-2 (20th application); RewardLadderEventKind
    // 9-arm discriminator covers rlaif_judge_rated /
    // rlaif_calibration_correlation_low / gepa_template_evolved /
    // gepa_template_evicted / reward_fusion_computed /
    // reward_hacking_suspected / sft_training_threshold_reached /
    // bai_arm_observation_recorded / rl_weight_update_unlocked.
    // Wire-dark unless ELAI_AGENTIC_RL_ENABLE=1. See
    // workflow/reasoning/E-55-decisions.md.
    SsbActionType::RewardLadderEvent,
    // E-11 (SCHEMA_VERSION 169): SaaS prior-art catalog + recommender
    // substrate. SINGLE-VARIANT Pattern-2 (21st application);
    // PriorArtEventKind 2-arm discriminator covers
    // archetype_recommended / prior_art_consulted. Wire-dark unless
    // ELAI_SAAS_PRIOR_ART_ENABLE=1. See
    // workflow/reasoning/E-11-decisions.md.
    SsbActionType::SaasPriorArtEvent,
    // E-22 (SCHEMA_VERSION 170): may-2026 github trending sweep
    // substrate (SKILL.md export + microVM sandbox + lazy MCP schema
    // savings). SINGLE-VARIANT Pattern-2 (22nd application);
    // TrendingEventKind 3-arm discriminator covers
    // skill_export_normalized / microvm_fallback / lazy_schema_savings.
    // Wire-dark unless ELAI_GITHUB_TRENDING_ENABLE=1. See
    // workflow/reasoning/E-22-decisions.md.
    SsbActionType::GithubTrendingEvent,
    // E-23 (SCHEMA_VERSION 171): 2026 developer-discourse positioning +
    // UX-guard substrate. SINGLE-VARIANT Pattern-2 (23rd application);
    // DiscourseEventKind 3-arm discriminator covers
    // trust_telemetry_aggregated / runtime_choice_badge_emitted /
    // context_state_warning. Wire-dark unless
    // ELAI_DEVELOPER_DISCOURSE_ENABLE=1. See
    // workflow/reasoning/E-23-decisions.md.
    SsbActionType::DeveloperDiscourseEvent,
    // E-25 (SCHEMA_VERSION 172): claude-code-architecture catalog
    // substrate. SINGLE-VARIANT Pattern-2 (24th application);
    // ClaudeCodeArchitectureEventKind 9-arm discriminator covers
    // compaction_ladder_tier_applied / skill_template_executed /
    // prompt_decision_made / opusplan_overlay_activated / session_forked
    // / edit_checkpoint_snapshotted / bash_complexity_blocked /
    // approval_fatigue_detected / sandbox_template_warned. Wire-dark
    // unless ELAI_CLAUDE_CODE_ARCH_ENABLE=1. See
    // workflow/reasoning/E-25-decisions.md.
    SsbActionType::ClaudeCodeArchitectureEvent,
    // E-29 Tier-B (SCHEMA_VERSION 173): agentic-observability event
    // substrate. SINGLE-VARIANT Pattern-2 (25th application);
    // AgenticObservabilityEventKind 4-arm discriminator covers
    // mast_failure_classified / replay_resume / signal_match /
    // eval_dataset_member. Wire-dark unless
    // ELAI_AGENTIC_OBSERVABILITY_ENABLE=1.
    SsbActionType::AgenticObservabilityEvent,
    // E-31 (SCHEMA_VERSION 174): coding-model-leaderboard scraper substrate.
    // SINGLE-VARIANT Pattern-2 (26th application); LeaderboardEventKind 3-arm
    // discriminator covers scrape_started / scrape_completed / scrape_failed.
    // Wire-dark unless ELAI_CODING_MODEL_LEADERBOARD_ENABLE=1. Scraper at
    // benches/external/coding_model_leaderboard.py emits stdout JSON rows
    // parsed by the rust dispatcher. Live scrape RUNPOD-PENDING. See
    // workflow/reasoning/E-31-leaderboard-decisions.md.
    SsbActionType::CodingModelLeaderboardEvent,
    // E-38 (SCHEMA_VERSION 175): 2026 AI roadmap signals + EU AI Act
    // compliance pre-gate substrate. SINGLE-VARIANT Pattern-2 (27th
    // application); RoadmapSignalKind 5-arm discriminator covers
    // compliance_blocked / compliance_verified /
    // mcp_transport_version_logged / mrcr_unverified / posture_doc_stale.
    // Wire-dark unless ELAI_AI_ROADMAP_SIGNALS_ENABLE=1. See
    // workflow/reasoning/E-38-decisions.md.
    SsbActionType::AiRoadmapSignalEvent,
    // E-39 (SCHEMA_VERSION 176): cost + token economics substrate.
    // SINGLE-VARIANT Pattern-2 (28th application); CostEventKind 8-arm
    // discriminator covers task_aborted_by_spend_cap /
    // velocity_cap_breached / automix_verifier_verdict /
    // cost_router_decision / cost_estimate_inaccurate /
    // free_tier_quota_approaching / free_tier_quota_exceeded /
    // layered_cap_manual_override. Wire-dark unless
    // ELAI_COST_TOKEN_ECONOMICS_ENABLE=1. Producers: orchestrator/
    // cost/{velocity_cap,automix_gate,cost_router,free_tier_guard,
    // cloud_abort}.rs + CLI `elai cloud override-cap`. See
    // workflow/reasoning/E-39-decisions.md.
    SsbActionType::CostTokenEvent,
    // E-50 (SCHEMA_VERSION 177): SaaS architecture pattern catalog
    // substrate. SINGLE-VARIANT Pattern-2 (29th application);
    // SaasArchPatternEventKind N-arm discriminator (11 arms) covers
    // production_readiness_gate_blocked /
    // production_readiness_gate_passed / rls_baby_step_emitted /
    // unscoped_token_blocked / billing_mor_decision_made /
    // realtime_block_decided / async_backbone_decided /
    // per_tier_rate_limit_derived / otel_slo_runbook_scaffolded /
    // platform_unsupported_for_scope_check / archetype_block_overridden.
    // Wire-dark unless ELAI_SAAS_ARCH_PATTERNS_ENABLE=1. Producer:
    // orchestrator/src/architecture/saas_patterns.rs. See
    // workflow/reasoning/E-50-decisions.md.
    SsbActionType::SaasArchPatternEvent,
    // E-4 (SCHEMA_VERSION 178): chat UX critique substrate.
    // SINGLE-VARIANT Pattern-2 (30th application); ChatUxCritiqueKind
    // 13-arm discriminator covers claim_confidence_tier_emitted /
    // sycophancy_pushback_detected / user_rule_pinned /
    // user_rule_unpinned / session_re_anchored / doom_loop_detected /
    // completion_claim_rejected_no_evidence / one_shot_clarifier_emitted /
    // one_shot_clarifier_resolved / refusal_emitted /
    // refusal_overridden_by_user / mid_stream_interrupt_committed /
    // honest_correction_emitted. Wire-dark unless
    // ELAI_CHAT_UX_CRITIQUE_ENABLE=1. Producer:
    // orchestrator/src/chat_ux/* (substrate module). See
    // workflow/reasoning/E-4-decisions.md.
    SsbActionType::ChatUxCritiqueEvent,
    // E-16 (SCHEMA_VERSION 179): VLM image-validation hardening
    // single-variant Pattern-2 (31st application). VlmImageValidationKind
    // 3-arm discriminator covers swap_order_disagreement /
    // axtree_mismatch / vlm_schema_unconstrained. SycophancyFlag
    // (E-4 / D-14 prior ship) is REUSED for the sycophancy_probe
    // inverted-framing fire-point. Wire-dark unless
    // ELAI_VLM_IMAGE_VALIDATION_ENABLE=1. Producer:
    // orchestrator/src/media_validator/{vlm_validator, axtree,
    // sycophancy}. See workflow/reasoning/E-16-decisions.md.
    SsbActionType::VlmImageValidationEvent,
    // E-20 (SCHEMA_VERSION 180): local-cloud routing substrate.
    // SINGLE-VARIANT Pattern-2 (32nd application). RoutingEventKind
    // 9-arm discriminator covers confidence_emitted /
    // hybrid_escalation / difficulty_classified / privacy_blocked /
    // output_filtered / burn_alert / reasoning_mode /
    // majority_vote / role_priority_chosen. Wire-dark unless
    // ELAI_LOCAL_CLOUD_ROUTING_ENABLE=1. Producer:
    // orchestrator/src/cost/cloud_route.rs (sibling to E-39
    // automix_gate + cost_router + cloud_abort under cost/ subtree).
    // PrivacyBlock composes with the pre-existing RouteBlocked
    // (C-5 ordinal 331) via reason="privacy_block" payload value.
    // See workflow/reasoning/E-20-decisions.md.
    SsbActionType::LocalCloudRoutingEvent,
    // E-21 (SCHEMA_VERSION 181): peer-systems harness substrate.
    // SINGLE-VARIANT Pattern-2 (33rd application). PeerSystemsEventKind
    // 4-arm discriminator covers critic_candidates_generated /
    // critic_score_computed / critic_pick_committed /
    // critic_rerank_skipped. Wire-dark unless
    // ELAI_PEER_SYSTEMS_ENABLE=1. Producer:
    // orchestrator/src/peer_systems/{critic_rerank,sbfl,repo_state_store,
    // aci_audit}.rs. Distinct from existing SsbActionType::PagerankRun
    // (C-3/C-38 surface, reused) and crate::mini_critic (role-time
    // signature linter, distinct from inference-time N-candidate
    // reranker). See workflow/reasoning/E-21-decisions.md.
    SsbActionType::PeerSystemsEvent,
    // E-36 (SCHEMA_VERSION 182): document + vision RAG substrate.
    // SINGLE-VARIANT Pattern-2 (34th application).
    // DocumentVisionRagEventKind 5-arm discriminator covers
    // vision_chunk_emitted / vision_fallback_triggered /
    // vision_rag_policy_decision / visual_artifact_check_verdict /
    // doc_parser_cloud_fallback_blocked_by_mode. Wire-dark unless
    // ELAI_DOCUMENT_VISION_RAG_ENABLE=1. Producers:
    // orchestrator/src/document_vision_rag/{visual_verifier, policy,
    // mod}.rs + knowledge::chunking::image_artifact +
    // knowledge::chunking::pdf_structured. 10th VerificationMethod
    // variant (reasoning::types::VerificationMethod::VisualArtifactCheck)
    // dispatches into this substrate; reuses claim_category String
    // instead of a parallel claim_class taxonomy.
    // See workflow/reasoning/E-36-decisions.md.
    SsbActionType::DocumentVisionRagEvent,
    // E-42 (SCHEMA_VERSION 183): tree-sitter grammar health substrate.
    // SINGLE-VARIANT Pattern-2 (35th application).
    // TreeSitterGrammarEventKind 4-arm discriminator covers
    // abi_probe_mismatch / svelte_runes_parse_health /
    // tier_b_grammar_loaded / tree_sitter_skipped_large_file. Wire-dark
    // unless ELAI_TREE_SITTER_GRAMMARS_ENABLE=1. Producers:
    // knowledge::tree_sitter_health::{abi_probe, svelte_health, large_file,
    // registration} + DkgEngine init path. Collapses the plan's
    // demanded 4 sibling variants into one per DRIFT-6 in
    // workflow/reasoning/E-42-decisions.md.
    SsbActionType::TreeSitterGrammarEvent,
    // E-46 (SCHEMA_VERSION 184): planning-decomposition substrate.
    // SINGLE-VARIANT Pattern-2 (36th application);
    // PlanningDecompositionEventKind 8-arm discriminator covers
    // reflection_appended / low_entropy_decomposition /
    // goal_drift_canary_below_threshold / bubble_up_replan_phase /
    // bubble_up_ledger_reset / agentless_shortcut_dispatched /
    // process_reward_score_emitted / skill_prior_retrieved.
    // Wire-dark unless ELAI_PLANNING_DECOMPOSITION_ENABLE=1.
    SsbActionType::PlanningDecompositionEvent,
    // E-48 (SCHEMA_VERSION 185): web-research orchestrator hardening
    // substrate. SINGLE-VARIANT Pattern-2 (37th application);
    // WebResearchEventKind 10-arm discriminator covers
    // research_plan_emitted / tiered_search_dispatched /
    // extraction_failed_all_extractors / cross_encoder_rerank_complete /
    // speculative_branch_pruned / research_preemptive_stop /
    // contradiction_node_materialized / citation_verification_outcome /
    // citation_attribution_violation_blocked / recency_decay_applied.
    // Wire-dark unless ELAI_WEB_RESEARCH_ORCHESTRATOR_ENABLE=1.
    SsbActionType::WebResearchEvent,
    // E-44 (SCHEMA_VERSION 186): durability observability substrate.
    // SINGLE-VARIANT Pattern-2 (38th application);
    // DurabilityEventKind 10-arm discriminator covers
    // tool_idempotency_hit / tool_idempotency_replayed_irreversible /
    // compensation_logged / compensation_replayed /
    // compensation_failed / session_dag_continue_as_new /
    // checkpoint_lazy_migrated / compact_lock_contended /
    // agent_reliability_metrics / tool_error_classified.
    // Wire-dark unless ELAI_DURABILITY_OBSERVABILITY_ENABLE=1.
    // TASK-E44-013 observability gap closure.
    SsbActionType::DurabilityEvent,
    // E-31 (SCHEMA_VERSION 187): head_dim invariant verification substrate.
    // SINGLE-VARIANT Pattern-2 (39th application);
    // HeadDimInvariantVerifiedKind 2-arm discriminator covers
    // verified / refused. Wire-dark unless
    // ELAI_HEAD_DIM_INVARIANT_SSB_ENABLE=1. TASK-E31-001 closure.
    SsbActionType::HeadDimInvariantVerified,
    // E-3 (SCHEMA_VERSION 188): role-shown / model-hidden default
    // telemetry substrate. SINGLE-VARIANT Pattern-2 (40th application);
    // no Kind discriminator (scope-lean MVP). Wire-dark unless
    // ELAI_ROLE_VISIBILITY_TELEMETRY_ENABLE=1. TASK-E03-001 MVP closure.
    SsbActionType::RoleShownModelHidden,
    // E-49 (SCHEMA_VERSION 189): coding-agent metrics named-token
    // substrate. SINGLE-VARIANT Pattern-2 (41st application);
    // CodingMetricNamedTokenKind 3-arm discriminator covers
    // stable_pass_at_1 / harness_attribution_tuple / stop_verified.
    // Wire-dark unless ELAI_CODING_METRIC_NAMED_TOKEN_ENABLE=1.
    // Closes TASK-E49-005 / 006 / 007 SPEC-DRIFT.
    SsbActionType::CodingMetricNamedToken,
    // E-3 sub-phase 4A (SCHEMA_VERSION 190): runtime handoff event
    // substrate. SINGLE-VARIANT Pattern-2 (42nd application);
    // RuntimeHandoffEventKind 6-arm discriminator covers
    // unified_preamble_applied / runtime_prewarm_started /
    // runtime_prewarm_complete / handoff_payload_compressed /
    // persona_vector_applied / style_normalizer_applied. Wire-dark
    // unless ELAI_E3_RUNTIME_HANDOFF_ENABLE=1. TASK-E03-001 4A.
    SsbActionType::E3RuntimeHandoffEvent,
    // E-31 sub-phase 6A (SCHEMA_VERSION 191): cloud-route decision
    // event substrate. SINGLE-VARIANT Pattern-2 (43rd application);
    // RouteDecisionEventKind 3-arm discriminator covers
    // cloud_route_chosen / tier_model_swapped / local_route_kept.
    // Wire-dark unless ELAI_ROUTE_DECISION_SSB_ENABLE=1.
    SsbActionType::RouteDecisionEvent,
    // E-47 (SCHEMA_VERSION 192): LoRA fine-tuning ship-gate substrate.
    // SINGLE-VARIANT Pattern-2 (44th application);
    // LoraShipGateEventPhase 3-arm discriminator covers
    // refused / calibration_missing / calibration_passed. Wire-dark
    // unless ELAI_LORA_SHIP_GATE_ENABLE=1.
    SsbActionType::LoraShipGateEvent,
    // E-40 Tranche B-2 (SCHEMA_VERSION 193): SymbolGraph augmentation
    // substrate. SINGLE-VARIANT Pattern-2 (45th application);
    // SymbolGraphEventKind 4-arm discriminator covers
    // symbol_dsl_executed / dataflow_impact_computed /
    // agent_impact_claim_refuted / scope_graph_resolved. Wire-dark
    // unless ELAI_SYMBOL_GRAPH_5TH_RRF_ENABLE=1.
    SsbActionType::SymbolGraphEvent,
    // Tranche B-6 quad (SCHEMA_VERSION 194): four new SSB substrates.
    // See header SCHEMA_VERSION block for full tranche docs.
    SsbActionType::McpAuthzEvent,
    SsbActionType::ToolCalibrationEvent,
    SsbActionType::ReflectBudgetSoftWarn,
    SsbActionType::ContractCodingEvent,
    // F-1 / F-2a (SCHEMA_VERSION 196, 2026-06-01): vibe-coding security gate
    // variants. VibeGateBlocked is Pattern-2 single variant (4-arm Kind
    // discriminator; counts as ONE variant here). SecurityRuleBlocked is a
    // flat beacon. Session: s-6344.
    SsbActionType::VibeGateBlocked(VibeGateBlockedKind::CodeVuln),
    SsbActionType::SecurityRuleBlocked,
    // F-4 / Slice B-1 (SCHEMA_VERSION 197, 2026-06-02): TestIntegrityEvent
    // Pattern-2 single variant (2-arm Kind discriminator; counts as ONE
    // variant here). Verified arm is the representative arm. Session: s-6344.
    SsbActionType::TestIntegrityEvent(TestIntegrityEventKind::Verified),
    // F-3p1 / Slice C1 (SCHEMA_VERSION 198, 2026-06-02): ToolFilterEvent
    // Pattern-2 single variant (2-arm ToolFilterEventKind discriminator;
    // counts as ONE variant here). Committed arm is the representative arm.
    // Session: s-6344.
    SsbActionType::ToolFilterEvent(ToolFilterEventKind::Committed),
    // F-9 / Slice D1 (SCHEMA_VERSION 199, 2026-06-02): WriteTimeSlopsquatBlocked
    // flat beacon. Not Pattern-2; single sql_str. Session: s-6344.
    SsbActionType::WriteTimeSlopsquatBlocked,
    // F-10 / Slice A1 (SCHEMA_VERSION 200, 2026-06-02): SecurityClassRegressionBlocked
    // flat beacon (security-monotonicity iteration guard). loop_kind is a payload
    // field; not Pattern-2. Single sql_str. Session: s-60824.
    SsbActionType::SecurityClassRegressionBlocked,
    // F-2b / Slice A1 (SCHEMA_VERSION 201, 2026-06-02): InsecureRateMeasured flat
    // beacon (insecure-rate keystone bench). Bench telemetry; not Pattern-2.
    // Single sql_str. Session: s-60824.
    SsbActionType::InsecureRateMeasured,
    // I-9 (SCHEMA_VERSION 263, 2026-06-14): ErrorAmplificationMeasured flat
    // beacon (error-amplification alpha_t telemetry). Bench telemetry; not
    // Pattern-2. Single sql_str "error_amplification_measured". This is the
    // ONLY counter touch -- SCHEMA_VERSION (=-309+len) + TOTAL_VARIANTS + the
    // action_type_lock golden all auto-derive from this append.
    SsbActionType::ErrorAmplificationMeasured,
    // F-6 / Slice A1 (SCHEMA_VERSION 202, 2026-06-02): SkillLibraryEvent
    // Pattern-2 single variant (3-arm SkillLibraryEventKind discriminator;
    // counts as ONE variant here). Acquired arm is the representative arm.
    // Session: s-60824.
    SsbActionType::SkillLibraryEvent(SkillLibraryEventKind::Acquired),
    // F-5 / Slice A1 (SCHEMA_VERSION 203, 2026-06-02): StalenessQuarantineBroadcast
    // SINGLE FLAT beacon (cross-subsystem staleness-quarantine cascade). No Kind
    // discriminator; one sql_str (staleness_quarantine_broadcast). Session: s-60824.
    SsbActionType::StalenessQuarantineBroadcast,
    // F-8 / Slice A1 (SCHEMA_VERSION 204, 2026-06-02): SymbolicConflictDetected
    // SINGLE FLAT beacon (SCF symbolic semantic conflict detector). No Kind
    // discriminator; one sql_str (symbolic_conflict_detected). Session: s-60824.
    SsbActionType::SymbolicConflictDetected,
    // F-7 / Slice A1 (SCHEMA_VERSION 205, 2026-06-02): TemplateUtilityRanked
    // SINGLE FLAT beacon (CBR template Laplace-utility ranking, HALF B only).
    // No Kind discriminator; one sql_str (template_utility_ranked).
    // Session: s-60824.
    SsbActionType::TemplateUtilityRanked,
    // F-13 / Slice A1 (SCHEMA_VERSION 206, 2026-06-03): SessionArchived
    // SINGLE FLAT beacon (session-archive-on-schema-bump). No Kind
    // discriminator; one sql_str (session_archived). Session: s-6312.
    SsbActionType::SessionArchived,
    // F-11 / Slice A1 (SCHEMA_VERSION 207, 2026-06-03): DarwinianAdmit config-DGM
    // admit-loop beacon. Pattern-2 single-variant; the 3-arm DarwinianAdmitKind
    // discriminator (Admitted / RejectedByGuard / LoopClosed) is on the payload,
    // one sql_str (darwinian_admit). SEPARATE from E-7 darwinian_generation_-
    // completed. Wire-dark unless ELAI_DARWINIAN_LOOP_ENABLE=1 AND cfg(feature
    // darwinian-evolve). Session: s-6312.
    SsbActionType::DarwinianAdmit,
    // G-plan + H-plan repair (2026-06-07, s-build-repair): 35 new variants
    // completing G-plan security/verifiability substrate + H-plan canvas/batch.
    SsbActionType::McpRugPullBlocked,
    SsbActionType::TaintFlow,
    SsbActionType::BlastRadiusBlocked,
    SsbActionType::TrifectaQuarantined,
    SsbActionType::RunArtifactEmitted,
    SsbActionType::ReviewSurfaceScored,
    SsbActionType::ProvenanceAttestationEmitted,
    SsbActionType::SafetyDeltaMeasured,
    SsbActionType::ModelDriftSuspected,
    SsbActionType::SecretRedacted,
    SsbActionType::ClaimConfidenceScored,
    SsbActionType::SteeringVectorRefreshTriggered,
    SsbActionType::CanvasOpApplied,
    SsbActionType::CanvasViewEvent,
    SsbActionType::CanvasActivityEvent,
    SsbActionType::BatchLifecycle,
    SsbActionType::DeflationGateEvaluated,
    SsbActionType::LeakageBlocked,
    SsbActionType::LookaheadDifferential,
    SsbActionType::TradingNullTest,
    SsbActionType::RerunHashEquality,
    SsbActionType::SimRealParity,
    SsbActionType::ResultManifestVerified,
    SsbActionType::DifferentialDivergence,
    SsbActionType::GateSelfValidation,
    SsbActionType::RewardTampering,
    SsbActionType::HoldoutBudgetEvent,
    SsbActionType::DriftInterlock,
    SsbActionType::TrialRegistered,
    SsbActionType::MucocoInconsistencyDetected,
    SsbActionType::MucocoSkipReason,
    SsbActionType::UserDecisionSuperseded,
    SsbActionType::StalenessEvalMeasured,
    // D-27.1 (SCHEMA_VERSION 209): discovery-queue reviewer-action beacons.
    SsbActionType::DiscoveryRatified,
    SsbActionType::DiscoveryRejected,
    // H-32 (SCHEMA_VERSION 210): FactValidationFailed flat beacon.
    SsbActionType::FactValidationFailed,
    // H-33 (SCHEMA_VERSION 211): MemoryGovernanceEvent flat beacon.
    SsbActionType::MemoryGovernanceEvent,
    // H-34 (SCHEMA_VERSION 212): McpToolDeferred -- deferred STDIO schema resolve.
    SsbActionType::McpToolDeferred,
    // H-34 (SCHEMA_VERSION 213): McpSchemaBudgetMeasured -- bench result beacon.
    SsbActionType::McpSchemaBudgetMeasured,
    // H-35 (SCHEMA_VERSION 214): SessionRewound -- session-rewind restore beacon.
    SsbActionType::SessionRewound,
    // H-36 (SCHEMA_VERSION 215): AgentNotificationEmitted -- completion/needs-input
    // dev-event beacon. Emitted by NotificationDispatcher::notify.
    SsbActionType::AgentNotificationEmitted,
    // H-37 (SCHEMA_VERSION 216): PlanDecisionEvent -- plan-approval gate decision.
    // Pattern-2 4-arm PlanDecisionKind (Approve/Redirect/Reject/AutoAdvance).
    SsbActionType::PlanDecisionEvent,
    // H-38 (SCHEMA_VERSION 217): StopConfirmationEvent -- cost-confirm gate outcome.
    // Pattern-2 single-variant 3-arm StopOutcomeKind discriminator.
    SsbActionType::StopConfirmationEvent,
    // H-39 (SCHEMA_VERSION 218): AbstentionScored -- abstention-aware scoring bench beacon.
    // Flat beacon. Default-ON, kill-switch ELAI_ABSTENTION_SCORE_DISABLE.
    SsbActionType::AbstentionScored,
    // H-41 (SCHEMA_VERSION 219): CheckerGapDetected -- off-agent backlog beacon.
    // Emitted when Factual/Computational claim falls to non-Constructive.
    // Wire-dark: ELAI_CHECKER_GAP_ENABLE. Flat beacon.
    SsbActionType::CheckerGapDetected,
    // H-42 (SCHEMA_VERSION 220): ReasoningBudgetScaled -- serial-depth-scaled GRC
    // budget decision beacon. Default-ON, kill-switch
    // ELAI_REASONING_BUDGET_SCALE_DISABLE. Pattern-2 single-variant 2-arm
    // ReasoningBudgetScaledKind: DepthBound / AccessBound.
    SsbActionType::ReasoningBudgetScaled,
    // H-43 (SCHEMA_VERSION 221): FitnessEvaluated -- execution-grounded DGM fitness
    // beacon. Replaces blake3-of-bytes derived_alpha (GAP-1). Pattern-2 single-variant
    // 3-arm FitnessEvaluatedKind: ExecutionPassRate | DualAgreement | FitnessRegression.
    SsbActionType::FitnessEvaluated,
    // H-44 (SCHEMA_VERSION 222): TrapBehaviorDetected -- behavioral cheat-detector
    // beacon. Replaces keyword-grep (GAP-2) with ImpossibleBench/IsomorphMismatch/
    // CodeHalu/KeywordFallback detection + frozen TRACE recall. Pattern-2 single-variant
    // 4-arm TrapBehaviorDetectedKind.
    SsbActionType::TrapBehaviorDetected,
    // H-45 (SCHEMA_VERSION 223): PromotionScoreAggregated -- worst-case/variance-penalized
    // aggregate over lineage-diverse promotion verdicts (Coste 2310.02743 WCO+UWO +
    // PRMBench 2501.03124 tau calibration). Replaces candidate mean as the
    // two_gate_promote margin. Pattern-2 single-variant 3-arm
    // PromotionScoreAggregatedKind: WorstCaseScored | VariancePenalized |
    // LineageDiverseDisagree.
    SsbActionType::PromotionScoreAggregated,
    // H-46 (SCHEMA_VERSION 224): CladeSelected -- HGM clade-metaproductivity
    // parent-selection beacon (HGM 2510.21614, ICLR 2026 oral). Emitted from
    // config_archive::select_parent_with_cmp. CMP aggregates H-43 execution-
    // grounded deltas. Pattern-2 single-variant 2-arm CladeSelectedKind:
    // CladeMetaproductivity | ImmediateScoreFallback.
    SsbActionType::CladeSelected,
    // H-47 (SCHEMA_VERSION 225): EvalFirewallEvent -- DGM eval firewall beacon.
    // Composes D-6 date-gate + E-54 template-regen + H-19 Thresholdout budget.
    // Emitted once per generation from run_one_generation. H-19 budget leg
    // REUSES HoldoutBudgetEvent (GD-1). Pattern-2 single-variant 3-arm
    // EvalFirewallEventKind: PreUpdateItemRejected | TemplateRegenerated |
    // IndistinguishabilityProbe.
    SsbActionType::EvalFirewallEvent,
    // H-49 (SCHEMA_VERSION 226): EditHistoryEvent -- reversible LLM edits beacon.
    // Emitted by the default-ON CAS time-travel capture hook (task_runner.rs) and
    // by elai undo/redo/jump CLI ops. Default-ON kill-switch ELAI_TIMETRAVEL_DISABLE.
    // Pattern-2 single-variant 6-arm EditHistoryEventKind:
    // SnapshotCaptured | Deduped | Redo | Branched | Jumped | TooLargeSkipped.
    SsbActionType::EditHistoryEvent,
    // H-51 (SCHEMA_VERSION 227): HandoffLifecycleEvent -- cross-session sleep-time
    // handoff digest lifecycle beacon. Emitted by Drop hook (DigestWritten), REPL
    // startup detect (ResumeOffered), one-tap confirm (ResumeConfirmed), and /continue
    // tiered load (TieredLoad). Default-ON kill-switch ELAI_HANDOFF_AUTORESUME_DISABLE.
    // Pattern-2 single-variant 4-arm HandoffLifecycleEventKind.
    SsbActionType::HandoffLifecycleEvent,
    // H-52 (SCHEMA_VERSION 228): FindingAnchored -- deterministic snippet->hunk
    // finding-anchoring beacon. Emitted by elai review --cockpit (the real CLI
    // entrypoint). Closes G-5 OQ#2. Default-ON kill-switch ELAI_REVIEW_ANCHOR_DISABLE.
    // Pattern-2 single-variant 5-arm FindingAnchoredKind:
    // Anchored | FileFallback | Ambiguous | Unanchorable | Relocated.
    SsbActionType::FindingAnchored,
    // H-53 (SCHEMA_VERSION 229, 2026-06-11): CitationAnchorsBound -- citation pre-binding
    // beacon. Emitted by elai prebind --demo-prebind (REAL CLI entrypoint) and by the
    // /research|/ask answer path after citation_prebind::freeze_anchors runs.
    // Default-ON kill-switch ELAI_CITATION_PREBIND_DISABLE (no allowlist row per R9).
    // Pattern-2 single-variant 2-arm CitationAnchorsBoundKind: Bound | Blocked. 539 -> 540.
    SsbActionType::CitationAnchorsBound,
    // I-39 (SCHEMA_VERSION 230, 2026-06-11): PlanStateDerived -- per-plan state derived
    // beacon. Emitted by elai plan status (REAL CLI entrypoint). Flat beacon;
    // letter + session_id + derived_state + evidence_anchor. Default-ON. 540 -> 541.
    SsbActionType::PlanStateDerived,
    // I-45 (SCHEMA_VERSION 231, 2026-06-11): ObjectiveSealed -- Pattern-2 single-variant
    // 3-arm ObjectiveSealedKind. Sealed representative in this lock array. 541 -> 542.
    SsbActionType::ObjectiveSealed(ObjectiveSealedKind::Sealed),
    // I-34 (SCHEMA_VERSION 234, 2026-06-12): GrammarGateEvent -- Pattern-2 single-variant
    // 3-arm GrammarGateEventKind. CranePassed representative in this lock array. 542 -> 543.
    SsbActionType::GrammarGateEvent(GrammarGateEventKind::CranePassed),
    // I-38 (SCHEMA_VERSION 235, 2026-06-12): QdeltaMeasured -- flat beacon. 543 -> 544.
    SsbActionType::QdeltaMeasured,
    // I-16: KvCacheTuned flat beacon.
    SsbActionType::KvCacheTuned,
    // I-43 (SCHEMA_VERSION 236, 2026-06-12): ClaimScopeMeasured -- Pattern-2 single-variant
    // 3-arm ClaimScopeMeasuredKind. PilotDelta representative in this lock array. 544 -> 545.
    SsbActionType::ClaimScopeMeasured(ClaimScopeMeasuredKind::PilotDelta),
    // I-10 (SCHEMA_VERSION 237, 2026-06-12): CacheEconomicsEvent -- Pattern-2 single-variant
    // 3-arm CacheEconomicsEventKind. TtlPlanned representative in this lock array. 545 -> 546.
    SsbActionType::CacheEconomicsEvent(CacheEconomicsEventKind::TtlPlanned),
    // I-8 (SCHEMA_VERSION 238, 2026-06-12): TrajectoryAborted -- Pattern-2 single-variant
    // 3-arm TrajectoryAbortedKind. Voluntary representative in this lock array. 546 -> 547.
    SsbActionType::TrajectoryAborted(TrajectoryAbortedKind::Voluntary),
    // I-1 (SCHEMA_VERSION 239, 2026-06-12): DirectiveExecuted -- Pattern-2 single-variant
    // 3-arm DirectiveExecutedKind. Executed representative in this lock array. 547 -> 548.
    SsbActionType::DirectiveExecuted(DirectiveExecutedKind::Executed),
    // I-33 (SCHEMA_VERSION 240, 2026-06-12): GovernancePrimitiveFired -- Pattern-2 single-variant
    // 3-arm GovernancePrimitiveFiredKind. CeflExternalized representative in this lock array. 548 -> 549.
    SsbActionType::GovernancePrimitiveFired(GovernancePrimitiveFiredKind::CeflExternalized),
    // I-32 (SCHEMA_VERSION 241, 2026-06-12): RoleContractApplied -- Pattern-2 single-variant
    // 3-arm RoleContractAppliedKind. PersonaStripped representative in this lock array. 549 -> 550.
    SsbActionType::RoleContractApplied(RoleContractAppliedKind::PersonaStripped),
    // I-35 (SCHEMA_VERSION 242, 2026-06-13): ToolSequenceVerdict -- Pattern-2 single-variant
    // 3-arm ToolSequenceVerdictKind: Allowed / HaltEnforced / OrderViolation.
    // Allowed representative in this lock array. 550 -> 551.
    SsbActionType::ToolSequenceVerdict(ToolSequenceVerdictKind::Allowed),
    // I-41 (SCHEMA_VERSION 243, 2026-06-13): CritiqueLensFinding -- spec-time critique lenses
    // advisory beacon. Pattern-2 single-variant 3-arm CritiqueLensFindingTag.
    // AdvisoryFinding representative in this lock array. 551 -> 552.
    SsbActionType::CritiqueLensFinding(CritiqueLensFindingTag::AdvisoryFinding),
    // I-46 (SCHEMA_VERSION 244, 2026-06-13): BashRedirected -- bash interceptor redirect/fixup/
    // allowed-prefix outcome. Pattern-2 single-variant 3-arm BashRedirectedKind.
    // Redirected representative in this lock array. 552 -> 553.
    SsbActionType::BashRedirected(BashRedirectedKind::Redirected),
    // I-47 (SCHEMA_VERSION 245, 2026-06-13): PromptHardeningApplied -- verifier-input XML tagging +
    // worker rule injection beacon. Pattern-2 single-variant 3-arm PromptHardeningAppliedKind.
    // InputsTagged representative in this lock array. 553 -> 554.
    SsbActionType::PromptHardeningApplied(PromptHardeningAppliedKind::InputsTagged),
    // I-2 (SCHEMA_VERSION 246, 2026-06-13): BranchWorkspaceEvent -- CoW workspace isolation barrier
    // outcome. Pattern-2 single-variant 4-arm BranchWorkspaceEventKind. Spawned representative
    // in this lock array. 554 -> 555.
    SsbActionType::BranchWorkspaceEvent(BranchWorkspaceEventKind::Spawned),
    // I-5 (SCHEMA_VERSION 247, 2026-06-13): ContextPruned -- per-branch context rebuild outcome.
    // Pattern-2 single-variant 3-arm ContextPrunedKind. SiblingExcluded representative
    // in this lock array. 555 -> 556.
    SsbActionType::ContextPruned(ContextPrunedKind::SiblingExcluded),
    // I-11 (SCHEMA_VERSION 248, 2026-06-13): BeliefStateEvent -- typed versioned condensate receipt.
    // Pattern-2 single-variant 3-arm BeliefStateEventKind: Condensed / Handoff / HonestyDelta.
    // Condensed is the representative arm (all 3 arms share wire tag "belief_state_event"). 556 -> 557.
    SsbActionType::BeliefStateEvent(BeliefStateEventKind::Condensed),
    // I-13 (SCHEMA_VERSION 249, 2026-06-13): MemoryGuardEvent -- write-time memory guard outcome.
    // Pattern-2 single-variant 3-arm MemoryGuardEventKind: PoisonBlocked / DualSerialized /
    // TraceQueryExpanded. PoisonBlocked is the representative arm. 557 -> 558.
    SsbActionType::MemoryGuardEvent(MemoryGuardEventKind::PoisonBlocked),
    // I-14 (SCHEMA_VERSION 250, 2026-06-13): ContextTransformReceipt -- deterministic
    // context-compression middleware receipt. Pattern-2 single-variant 3-arm
    // ContextTransformKind: LspSlice / AstFold / Tier2Compressed. LspSlice is the
    // representative arm. 558 -> 559.
    SsbActionType::ContextTransformReceipt(ContextTransformKind::LspSlice),
    // I-12 (SCHEMA_VERSION 251, 2026-06-13): PlaybookEvent -- evidence-gated playbook curator
    // outcome beacon. Pattern-2 single-variant 4-arm PlaybookEventKind: Promoted / Demoted /
    // ValidationRun / Exported. Promoted is the representative arm. 559 -> 560.
    SsbActionType::PlaybookEvent(PlaybookEventKind::Promoted),
    // I-17 (SCHEMA_VERSION 252, 2026-06-13): TriLayerVerdict -- tri-layer evaluation-stack
    // per-branch verdict. Pattern-2 single-variant 4-arm TriLayerVerdictKind: HardVeto /
    // ScoredSurvivor / DisagreementEscalated / FlakeQuarantined. HardVeto is the
    // representative arm. 560 -> 561.
    SsbActionType::TriLayerVerdict(TriLayerVerdictKind::HardVeto),
    // I-18 (SCHEMA_VERSION 253, 2026-06-13): RowChainSealed -- per-row rolling blake3
    // chain beacon (uPIP row-chain + AI-BOM). Pattern-2 single-variant 3-arm
    // RowChainSealedKind: ChainAdvanced / AibomEmitted / ChainBroken. ChainAdvanced
    // is the representative arm. 561 -> 562.
    SsbActionType::RowChainSealed(RowChainSealedKind::ChainAdvanced),
    // I-19 (SCHEMA_VERSION 254, 2026-06-14): TddPreflightBlocked -- TDD enforcer preflight
    // gate event. Pattern-2 single-variant 3-arm TddPreflightBlockedKind:
    // MissingTestDelta / SyntaxDryRunFailed / RefactorExempted.
    // Wire-dark default (ELAI_TDD_PREFLIGHT_ENABLE=1 required). MissingTestDelta is
    // the representative arm (primary block reason). 562 -> 563.
    SsbActionType::TddPreflightBlocked(TddPreflightBlockedKind::MissingTestDelta),
    // I-20 (SCHEMA_VERSION 255, 2026-06-14): MutationBudgetedRun -- budgeted mutation
    // scheduler (Track 15 KEYSTONE). Pattern-2 single-variant 4-arm MutationBudgetedRunKind:
    // CiCleared / CiFailed / SurvivorDirective / CacheHit. Default-ON; kill-switch
    // ELAI_MUTATION_BUDGET_DISABLE=1. CiCleared is the representative arm. 563 -> 564.
    SsbActionType::MutationBudgetedRun(MutationBudgetedRunKind::CiCleared),
    // I-21 (SCHEMA_VERSION 256, 2026-06-14): SprintContractEvent -- sprint-contract +
    // ralph-loop lifecycle event (Track 15 Wave 3). Pattern-2 single-variant 5-arm
    // SprintContractEventKind: Proposed / Approved / Rejected / RalphReinjected /
    // AdversaryEscalated. Wire-dark default (ELAI_SPRINT_CONTRACT_ENABLE=1 required).
    // Composes H-37/i-1/i-17/D-13 (GD-1). Proposed is the representative arm. 564 -> 565.
    SsbActionType::SprintContractEvent(SprintContractEventKind::Proposed),
    // I-37 (SCHEMA_VERSION 257, 2026-06-14): RequirementVerified -- per-EARS-requirement
    // verified-status event (Track 18 Wave 3). Pattern-2 single-variant 3-arm
    // RequirementVerifiedKind: Satisfied / Unverified / Regressed. Additive emit (always
    // lands when an EarsRequirementSet is present); the BLOCKING RequirementsGate is
    // wire-dark default (ELAI_REQUIREMENTS_GATE_ENABLE=1 required to block).
    // Composes IS-15/IS-8/I-20 (GD-1). Satisfied is the representative arm. 565 -> 566.
    SsbActionType::RequirementVerified(RequirementVerifiedKind::Satisfied),
    // I-44 (SCHEMA_VERSION 258, 2026-06-14): RetentionPurged -- zero-retention-after-
    // verification export purge receipt (Track 19 Wave 3). Pattern-2 single-variant 3-arm
    // RetentionPurgedKind: RawInputsPurged / EvidenceRetained / PurgeVerified. Destructive +
    // opt-in (elai run --zero-retention + ELAI_ZERO_RETENTION_ENABLE default-OFF kill-switch).
    // Composes G-9 redaction + G-9 TTL purge + F-13 archive + G-2/G-8 evidence (GD-1).
    // Emitted by crate::retention::purge_after_verify. RawInputsPurged is the representative
    // arm. 566 -> 567.
    SsbActionType::RetentionPurged(RetentionPurgedKind::RawInputsPurged),
    // I-52 (SCHEMA_VERSION 259, 2026-06-14): CheckpointToolEvent -- agent-callable
    // checkpoint tool outcome (Track 20 Wave 3). Pattern-2 single-variant 4-arm
    // CheckpointToolEventKind: Saved / Rewound / Diffed / RefusedDiverged. Default-ON
    // over the H-49 time-travel substrate (ELAI_TIMETRAVEL_DISABLE=1 kill-switch; no
    // separate I-52 flag). REUSE-vs-NEW resolved NEW (EditHistoryEvent's 6-arm Kind
    // lacks RefusedDiverged + a user label; R1 requires the refusal observable to land).
    // Composes H-49 timetravel::{capture_file,materialize} (GD-1). Emitted by the
    // checkpoint_save/rewind/diff arms of production_agent_rpc::translate_tool_request.
    // Saved is the representative arm. 567 -> 568.
    SsbActionType::CheckpointToolEvent(CheckpointToolEventKind::Saved),
    // I-54 (SCHEMA_VERSION 260, 2026-06-14): StructuralEditApplied -- agent-callable
    // structural (AST-aware) edit tool outcome (Track 20 Wave 3). Pattern-2 single-
    // variant 5-arm StructuralEditAppliedKind: Matched / Previewed / Applied /
    // RefusedStale / ParseFailed. Reuses elai_knowledge::TreeSitterAstEvaluator (one
    // tree-sitter version source, SLC-3; ast-grep dep BANNED). The taint-sink +
    // FS-blast-radius edit gates are the safety, so no separate I-54 flag. Composes
    // the shipped PreToolUse edit gates + dispatch_edit (GD-1). Emitted by the
    // structural_search/_preview/_apply arms of production_agent_rpc::translate_tool_request.
    // Matched is the representative arm. 568 -> 569.
    SsbActionType::StructuralEditApplied(StructuralEditAppliedKind::Matched),
    // I-4 (SCHEMA_VERSION 261, 2026-06-14): SearchNodeEvent -- evidence-ranked
    // search ladder branch lifecycle (Wave-4 KEYSTONE). Pattern-2 single-variant
    // 4-arm SearchNodeEventKind: Expanded / Pruned / Promoted / BudgetCollapsed.
    // The ladder generates K candidates, builds each branch's pristine context
    // (I-5 build_branch_context), runs the I-17 tri-layer stack, DROPS vetoed
    // (VerdictRow::is_vetoed) + flake-quarantined (FlakeQuarantineLedger::
    // is_quarantined) branches, calls escalate_on_disagreement on the survivor
    // SET, and ranks the rest by (tri-layer score, mutation kill-rate, progress)
    // to pick best-of-N. Selection is purely evidence-ranked (ID-4): a Promoted
    // row MUST carry verifier-derived evidence. Default-OFF
    // (ELAI_SEARCH_LADDER_ENABLE; GD-4 measured-first-win). Composes I-2/I-5/
    // I-17/I-20 (GD-1, never re-implemented). Emitted by
    // crate::search::SearchLadder::run_best_of_n from the
    // task_runner::dispatch_search_ladder post-task seam.
    // Expanded is the representative arm. 569 -> 570.
    SsbActionType::SearchNodeEvent(SearchNodeEventKind::Expanded),
    // I-7 (SCHEMA_VERSION 262, 2026-06-14): EscalationCascade -- cascading escalation
    // waterfall lifecycle (Wave-4). Pattern-2 single-variant 4-arm EscalationCascadeKind:
    // LocalWin / Condensed / Escalated / BatchRouted. Local-first: fan out K branches
    // (I-4 ladder composes I-2/I-17), condense-on-all-fail into an I-11 BeliefState,
    // route synchronously to the H-7 CloudModeRouter. IC-1: NO batch-client import
    // (BatchRouted is offline-journal-only, never fired live). Default-OFF
    // (ELAI_ESCALATION_WATERFALL_ENABLE; GD-4 measured-first-win). LocalWin is the
    // representative arm. 570 -> 571.
    SsbActionType::EscalationCascade(EscalationCascadeKind::LocalWin),
    // I-22 (SCHEMA_VERSION 264, 2026-06-14): ProgressRewardScored -- delta-progress
    // shaped-reward scorer (Track 15, lever ANTI-GAMING). Pattern-2 single-variant
    // 3-arm ProgressRewardScoredKind: Delta / FirstErrorTruncated / DecayApplied.
    // Delta: phi(h_t)-phi(h_{t-1}) from deterministic observables (test_pass,
    //   compile_err_inv, req_covered); ID-4 compliance (no LLM-opinion phi component).
    // FirstErrorTruncated: branch truncated at first i-17 HardVeto (step k-1);
    //   H-49 CAS fork node id recorded for prefix reinvestment.
    // DecayApplied: D consecutive zero-delta steps; D and weights from
    //   metrics/progress_reward_weights.toml (ID-5: measured, not paper-derived).
    // Wire-dark: ELAI_PROGRESS_REWARD_ENABLE=1 required (GD-4 measured-first-win).
    // Composes i-17/H-49/i-37 (GD-1). Emitted from task_runner::dispatch_progress_reward
    // (immediately after dispatch_tri_layer_verdict). Delta is the representative arm.
    // Counter is derived: TOTAL_VARIANTS = -309 + len(REGISTRY); do not hand-count.
    SsbActionType::ProgressRewardScored(ProgressRewardScoredKind::Delta),
    // I-24 (SCHEMA_VERSION 266, 2026-06-14): PrmCanaryEvent -- isomorphic canary
    // anti-gaming event (Track 15, lever ANTI-GAMING). Pattern-2 single-variant
    // 3-arm PrmCanaryEventKind: CanaryInjected / StyleBiasDetected / ScorerNeutralized.
    // CanaryInjected: pair injected, delta below ELAI-measured bias threshold.
    // StyleBiasDetected: delta >= threshold from metrics/canary_bias_threshold.toml
    //   (ID-5: ELAI-measured, never a paper constant); scorer flagged for this run.
    // ScorerNeutralized: N consecutive StyleBiasDetected -> scorer weight zeroed
    //   for the run (ID-4: L2-only; VETO_SCORE=0.0 for HardVeto still stands).
    // R8: canary pair set FROZEN in fixtures/canary_pairs.json (gate-manifest entry);
    //   the running system NEVER generates canaries from the model under test.
    // Wire-dark: ELAI_CANARY_INJECTOR_ENABLE=1 required (GD-4 measured-first-win).
    // Composes i-17 TriLayerStack L2 leg (GD-1; does NOT re-implement L2 invocation).
    // Emitted from canary_injector::emit_prm_canary_to_ssb called inside
    // TriLayerStack::run_verdict L2 path. CanaryInjected is the representative arm.
    // 574 -> 575.
    SsbActionType::PrmCanaryEvent(PrmCanaryEventKind::CanaryInjected),
    // I-27 (SCHEMA_VERSION 265, 2026-06-14): ConvergenceCurveMeasured -- verified-
    // outcome-rate vs compute convergence curve flat beacon (Track 16, lever PROVABLE).
    // Flat beacon (no Kind discriminator): ONE row per `elai bench convergence` run.
    // Carries curve_points[] (compute_level, mean_rate, ci_low, ci_high), knee_compute_level,
    // non_monotonic_at[], seed_count (>=5 I-38 floor), fixture_suite.
    // GD-2: curve_points recomputable under ELAI_SEED. GD-5: non_monotonic_at named,
    // never smoothed. Default-ON (CLI measurement instrument, no kill-switch).
    // WIRE-DARK[I-27]: live per-node compute-spend from I-4 SearchNodeEvent PENDING
    // (I-4 search/ladder.rs UNBUILT at HEAD); fixture leg uses cost_per_task_cents.
    // Emitted from bench_convergence::run_inner from CliAction::BenchConvergence dispatch.
    // 575 -> 576.
    SsbActionType::ConvergenceCurveMeasured,
    // I-36 (SCHEMA_VERSION 267, 2026-06-14): PromptCompiled -- per-(role,backend)
    // compiled-prompt artifact event (Track 18, Wave 4, lever CHEAPER-PORTABLE).
    // Pattern-2 single-variant 3-arm PromptCompiledKind: Compiled / TransferGapMeasured /
    // Rolledback. Each Kind arm has a distinct sql_str (per-Kind convention, mirrors
    // TddPreflightBlocked). Default-ON (no-op pass-through when no variant stored).
    // Kill-switch ELAI_PROMPT_COMPILE_DISABLE=1 (kill_switch type, ops-level exemption).
    // Deliberative-hygiene contract: NO CoT scaffold in compiled strings (landmine #6).
    // Composes I-32 role_contracts (compiled substitution applies AFTER) + B-37
    // PerRoleRuntime (the routing key) + I-12 ledger gates ID-6 (GD-1, never re-impl).
    // promote_variant WIRE-DARK[I-36] (Tier-2 DEFER — I-36-promotion plan);
    // run_transfer_gap_fixture WIRE-DARK[I-36] (Tier-2 DEFER — live cloud budget).
    // Compiled (the representative arm, ONE SSB_ACTION_TYPES entry). 575 -> 576.
    SsbActionType::PromptCompiled(PromptCompiledKind::Compiled),
    // I-50 (SCHEMA_VERSION 268, 2026-06-14): AuthBrokerEvent -- subscription auth broker
    // lifecycle event (Track 20, I-50). Pattern-2 single-variant 4-arm AuthBrokerEventKind:
    // TokenAcquired / Refreshed / ProviderLinked / RefreshFailed.
    // Each Kind arm has a distinct sql_str (per-Kind convention).
    // Default-ON. SSB-emit seam WIRE-DARK[I-50] at runtime_cmd.rs pending Arc<Ssb> threading.
    // Emit path proven reachable by the i50_auth_broker_contract test (real TempDir SQLite).
    // TokenAcquired is the representative arm (ONE SSB_ACTION_TYPES entry). 576 -> 577.
    SsbActionType::AuthBrokerEvent(AuthBrokerEventKind::TokenAcquired),
    // J-1.T4 (SCHEMA_VERSION 269, 2026-06-14): PerceptionScaffoldEvent -- perception-gateway
    // scaffold lifecycle event (J-1.T4). Pattern-2 single-variant 4-arm
    // PerceptionScaffoldEventKind: Preprocessed / ScaffoldEmitted / RoutedToSpecialist /
    // DegradedPassthrough. Single sql_str `perception_scaffold_event` (the Kind discriminator
    // rides inside the payload, mirroring DocumentVisionRagEvent).
    // Wire-dark unless ELAI_PERCEPTION_GATEWAY_ENABLE=1 (gate checked at the call site in
    // handle_infer_request; emit helper does NOT re-gate). T4 wires scaffold + router gates;
    // T5 wires the Preprocessed arm emit at the preprocess call site.
    // ScaffoldEmitted is the representative arm (ONE SSB_ACTION_TYPES entry). 577 -> 578.
    // SCHEMA = -309 + 578 = 269.
    SsbActionType::PerceptionScaffoldEvent(PerceptionScaffoldEventKind::ScaffoldEmitted),
    // K-18a follow-on (SCHEMA_VERSION 270): RecurrenceHitRateMeasured -- PS-5
    // recurrence-hit-rate flat beacon for `elai bench case-reuse`, restoring
    // GD-2 journal-recomputability that K-18a's ABORT-4 explicitly deferred.
    // Flat beacon (no Kind discriminator), mirrors ConvergenceCurveMeasured.
    // Emitted by bench_case_reuse::run_inner from CliAction::BenchCaseReuse.
    // Default-ON (fixture-suite CLI measurement instrument, no kill-switch).
    // 578 -> 579. SCHEMA = -309 + 579 = 270.
    SsbActionType::RecurrenceHitRateMeasured,
    // K-4 SP1 follow-on (SCHEMA_VERSION 271): GateTaxMeasured -- composed
    // gate-tax budget flat beacon for `elai bench gate-tax`, the PS-2 owner
    // number (KEYSTONE of K-T3). Flat beacon (no Kind discriminator), mirrors
    // RecurrenceHitRateMeasured. Emitted by bench_gate_tax::run from
    // CliAction::BenchGateTax. Default-ON, no kill-switch.
    // 579 -> 580. SCHEMA = -309 + 580 = 271.
    SsbActionType::GateTaxMeasured,
    // J-28 follow-on (SCHEMA_VERSION 272): UsageTurnEvent -- the uniform
    // per-turn token-usage row emitted by BOTH the cloud lane
    // (grounding_client.rs, beside CloudSpend) and the local lane
    // (runtime_orchestrator.rs, beside BackendCostAccrued). Flat variant
    // (mirrors RecurrenceHitRateMeasured/GateTaxMeasured); the 3-arm Kind
    // (TurnRecorded/EstimateApplied/NormalizationApplied) rides in
    // UsageTurnEventPayload.kind, not on this enum. Default-ON, no
    // kill-switch. Consumed by the T5 usage_ledger rollup + `elai usage`.
    // 580 -> 581. SCHEMA = -309 + 581 = 272.
    SsbActionType::UsageTurnEvent,
    // J-9 (SCHEMA_VERSION 273): HardwareModelCatalogEvent -- data-driven
    // hardware/model eligibility catalog. Flat action type; the 4-arm Kind
    // rides inside the payload. CatalogBuilt is emitted from the one-shot
    // startup SystemProbe seam. 581 -> 582. SCHEMA = -309 + 582 = 273.
    SsbActionType::HardwareModelCatalogEvent,
    // K-2 (SCHEMA_VERSION 274): deterministic routing economics benchmark.
    // Emitted by `elai bench routing-economics`; 582 -> 583.
    SsbActionType::RoutingEconomicsMeasured,
    // K-5 (SCHEMA_VERSION 275): deterministic risk-signal driven GateIntensity
    // selection. Emitted observe-only from the deterministic audit gate; 583 -> 584.
    SsbActionType::GateIntensitySelected,
    // K-9 (SCHEMA_VERSION 276): model refresh cadence decision row. Flat action
    // type; CandidateEvaluated/DecisionRecorded/IncumbentKept rides in
    // ModelRefreshCadencePayload.kind. 584 -> 585.
    SsbActionType::ModelRefreshCadence,
    // PS-3 (SCHEMA_VERSION 277): hallucination-delta flat beacon. Emitted by
    // `elai bench hallucination-delta`; 585 -> 586.
    SsbActionType::HallucinationDeltaMeasured,
    SsbActionType::LocalLaneE2eMeasured,
    SsbActionType::LocalDeterminismEvent(LocalDeterminismKind::BatchInvariantEnabled),
    // J-14 (SCHEMA_VERSION 282): repro-pack + RRA verify upgrade telemetry event.
    // Pattern-2 single-variant 4-arm ReproPackKind: TierEvaluated / FirstDivergenceLocated /
    // FlowFingerprintMismatch / FixtureExtracted. Emitted from rra::verify_rra (TierEvaluated,
    // FirstDivergenceLocated), compiled_pipeline::TemplateCrystallizer (FlowFingerprintMismatch),
    // and cli rra export-fixture (FixtureExtracted). Default-ON; no kill-switch (verify path).
    // TOTAL_VARIANTS 590 -> 591.
    SsbActionType::ReproPackEvent(ReproPackKind::TierEvaluated),
    // J-16 (SCHEMA_VERSION 283): blast-radius / anti-regression gate event.
    // Pattern-2 single-variant 4-arm BlastRadiusRegressionKind:
    // ConeComputed / ScopeViolationFlagged / RegressionSetSelected /
    // SilentDeletionFlagged. Emitted from blast_radius::BlastRadiusRegressionGate
    // on the EXECUTING -> AUDITING transition. OBSERVE-ONLY (GD-4):
    // flag, do not block, until FP/friction rate is bounded on >=3 real runs.
    // J-90 Leg A will widen the Kind to 6-arm (append-only).
    // TOTAL_VARIANTS 591 -> 592.
    SsbActionType::BlastRadiusRegressionEvent(BlastRadiusRegressionKind::ConeComputed),
    // J-30 T7b (SCHEMA_VERSION 284): provider BYOK lifecycle event. Flat
    // variant (Pattern-2, mirrors UsageTurnEvent); the 3-arm
    // ProviderByokEventKind (KeyStored / EndpointVerified /
    // CustomBackendRegistered) rides inside the payload. Emitted at the
    // T5 configure + T2 custom-backend register seams (T7c deferred).
    // NO key/secret/token material in the payload (plan stub_ban).
    // TOTAL_VARIANTS 592 -> 593.
    SsbActionType::ProviderByokEvent,
    // J-32 (SCHEMA_VERSION 285): front-door intent routing decision. Flat
    // variant (Pattern-2, mirrors ProviderByokEvent); the 5-arm
    // IntentRoutedKind (Classified / Confirmed / Rejected / Bypassed /
    // Clarified) rides inside the payload. Emitted by FrontDoorRouter on
    // every routing decision. Payload NEVER carries rewritten text
    // (arg_spans are byte-offsets). Emission sites land in a later slice
    // (slice-C, deferred like J-30 T7c).
    // TOTAL_VARIANTS 593 -> 594.
    SsbActionType::IntentRouted,
    // J-34 (SCHEMA_VERSION 286): empirical context calibration event. Flat
    // variant (Pattern-2, mirrors PerceptionScaffoldEvent); the 4-arm
    // ContextCalibrationKind (CurveMeasured / ThresholdApplied /
    // PlacementApplied / CalibrationInvalidated) rides inside the payload.
    // Emitted by CalibrationArtifactStore::insert (CurveMeasured) on a
    // bench-produced artifact, and by the leg-2 / leg-3 seams (later slices).
    // TOTAL_VARIANTS 594 -> 595.
    SsbActionType::ContextCalibrationEvent(ContextCalibrationKind::CurveMeasured),
    // J-36 (SCHEMA_VERSION 287): local prefix-KV cache event. Flat
    // variant (Pattern-2, mirrors ContextCalibrationEvent); the 4-arm
    // LocalPrefixCacheKind (HeadCacheHit / HeadCacheMiss / TailReprefill /
    // CacheIneligible) rides inside the payload. Emitted by
    // `emit_local_prefix_cache_event` (prefix_cache_ssb.rs) on the local
    // dispatch path when prefix-KV eligibility is checked.
    // TOTAL_VARIANTS 595 -> 596.
    SsbActionType::LocalPrefixCacheEvent(LocalPrefixCacheKind::HeadCacheHit),
    // J-24 (SCHEMA_VERSION 288): behavior steering event. Flat
    // variant (Pattern-2, mirrors ContextCalibrationEvent); the 3-arm
    // BehaviorSteeringKind (SteeringVectorApplied / SteeringEffectMeasured /
    // ObservationSizePreassessed) rides inside the payload. Emitted by
    // `emit_behavior_steering_event` (behavior_steering_ssb.rs).
    // TOTAL_VARIANTS 596 -> 597.
    SsbActionType::BehaviorSteeringEvent(BehaviorSteeringKind::ObservationSizePreassessed),
    // J-35 intent-control layer: typed intent spec + premise-check + route +
    // journaled assumptions. Flat variant (Pattern-2, mirrors
    // BehaviorSteeringEvent); the 4-arm IntentControlKind
    // (SpecReconstructed / PremiseFlagged / RouteDecided / AssumptionLogged)
    // rides inside the payload. Emitted by `run_intent_control`
    // (intent_control.rs) on the task-lane intake before decomposition.
    // TOTAL_VARIANTS 597 -> 598.
    SsbActionType::IntentControlEvent(IntentControlKind::SpecReconstructed),
    // J-39 T2: node admission gate event. Pattern-2 flat variant (mirrors
    // ContextCalibrationEvent); the 3-arm NodeAdmissionEventKind
    // (Admitted / SplitForced / EscalatedToCloud) rides inside the payload.
    // Emitted by `emit_node_admission_event` (node_admission.rs) at the
    // task_runner SPLIT_TASK phase_sizing branch.
    SsbActionType::NodeAdmissionEvent(NodeAdmissionEventKind::Admitted),
    // J-31: system-composition canvas event. Pattern-2 flat variant; 4-arm
    // SystemContractEventKind rides in the payload. Emitted by
    // emit_system_contract_event (system_contract_event.rs) at the canvas
    // bind_node/declare_edge ops + the auditor verdict tick.
    SsbActionType::SystemContractEvent(SystemContractEventKind::NodeBound),
    // J-12: HarnessInterventionEvent (Pattern-2; Kind in payload).
    SsbActionType::HarnessInterventionEvent(HarnessInterventionKind::ActionGateBlocked),
    // J-54 UNIT8: FreePoolEvent (Pattern-2 flat; 3-arm Kind in payload).
    SsbActionType::FreePoolEvent,
    // J-73 Slice C: RetrievalAbstained (Pattern-2 flat; 3-arm Kind in payload).
    SsbActionType::RetrievalAbstained,
    // J-78: ContextAccessSpec (Pattern-2 flat; 3-arm Kind in payload).
    SsbActionType::ContextAccessSpec,
    SsbActionType::ScopeCeilingAdvisory,
    SsbActionType::DagCreditAssigned,
    // I-30 Slice 1a: SFT exhaust event (Pattern-2 flat; 3-arm Kind in payload).
    SsbActionType::SftExhaustEvent,
    // C-37: snowballing harness summary measurement row.
    SsbActionType::SnowballingMeasured,
];

/// Non-variant schema epoch. Hand-bump ONLY for a DDL change that does NOT add an
/// action_type (a new column / CHECK tweak / index). Chosen as -309 so SCHEMA_VERSION
/// stays 253 at the current 562 variants (continuity with the prior hand-bumped value)
/// and auto-increments by 1 per new variant. SSB migration is equality-based
/// (drop+recreate on any mismatch), so a non-monotonic derived value is migration-safe.
const SCHEMA_VERSION_DDL_EPOCH: i64 = -309;

/// SSB schema version, DERIVED (Tier-B Move 2, Option A): no longer a hand-edited
/// literal, so two parallel plans adding action types never collide on this line.
/// Stays crate-private; external callers use [`SCHEMA_VERSION_PUBLIC`].
const SCHEMA_VERSION: i64 = SCHEMA_VERSION_DDL_EPOCH + SSB_ACTION_TYPES.len() as i64;

/// Public re-export of [`SCHEMA_VERSION`] for contract / integration tests
/// outside the orchestrator crate. The const itself stays crate-private so
/// internal callers cannot drift from the canonical value.
pub const SCHEMA_VERSION_PUBLIC: i64 = SCHEMA_VERSION;

/// Maximum serialized payload size in bytes (1 MiB).
/// `write_payload()` rejects payloads exceeding this limit with `SsbError::PayloadTooLarge`.
const PAYLOAD_MAX_BYTES: usize = 1_048_576;

/// H-10 canvas: number of ops between automatic state checkpoints.
pub const CANVAS_CHECKPOINT_INTERVAL_OPS: u32 = 20;

/// H-10 canvas: maximum number of ops that may accumulate between checkpoints
/// before a forced checkpoint is triggered. Must be >= CANVAS_CHECKPOINT_INTERVAL_OPS.
pub const CANVAS_MAX_DELTA_WITHOUT_CHECKPOINT: u32 = 50;

/// F-10 / Slice A1 (SCHEMA_VERSION 200) -- kill-switch env var name for the
/// security-monotonicity iteration guard. Default-ON: the guard runs unless
/// this var is set to a truthy value. There is intentionally NO default-off
/// ENABLE flag (W-2 -- security gates must not be wire-dark by default).
pub const ELAI_SECURITY_MONOTONICITY_DISABLE_ENV: &str =
    "ELAI_SECURITY_MONOTONICITY_DISABLE";

/// Tranche K / B.2 Option-B -- non-web retry emit env gate. Wire-dark
/// default: when set to truthy ("1"/"true"/"yes"/"on"), enables a
/// SECOND `fix_and_rerun_until_green` call site in
/// `task_runner::drive_task` (inserted AFTER the web `is_web_project`
/// branch) that fires for non-web projects (where `is_web_project=false`
/// and the baby step did not pass). The web-side gate
/// (`ELAI_C50_CONTRACT_RETRY_EMIT_ENABLE`) is unchanged. Both gates are
/// TRULY INDEPENDENT observability switches: per the W3 finding in the
/// Tranche K context package, gate selection is funneled through
/// [`crate::contract_coding::retry_emit::c50_emit_gate_truthy`], so
/// flipping only the non-web gate does NOT widen web-path emission, and
/// vice versa. Operators flip per-mode without coupling.
pub const ELAI_C50_NON_WEB_RETRY_EMIT_ENABLE_ENV: &str =
    "ELAI_C50_NON_WEB_RETRY_EMIT_ENABLE";

/// Tranche B-6 quad -- shared env-gate predicate. Returns `true` when the
/// named env var is set to one of `"1"` / `"true"` / `"yes"` / `"on"`
/// (case-insensitive). Mirrors the precedent set by other Pattern-2
/// substrates (E-47 LoraShipGate, E-40 SymbolGraph). Reads via
/// `std::env::var`; cheap enough for emit-hot-paths (microseconds).
#[must_use]
pub fn env_gate_truthy(name: &str) -> bool {
    match std::env::var(name) {
        Ok(v) => {
            let trimmed = v.trim();
            trimmed.eq_ignore_ascii_case("1")
                || trimmed.eq_ignore_ascii_case("true")
                || trimmed.eq_ignore_ascii_case("yes")
                || trimmed.eq_ignore_ascii_case("on")
        }
        Err(_) => false,
    }
}

/// Tranche B-6 quad -- shared blake3-prefix label hasher. Returns the
/// first 8 bytes of `blake3(label.as_bytes())` packed LE as `u64`.
/// Matches the prefix-hashing discipline used by C-45 / C-46 / C-47
/// payloads. blake3 (NOT blake2 / sha2) per CLAUDE.md HARD RULE.
#[must_use]
pub fn blake3_label_prefix(label: &str) -> u64 {
    let bytes = blake3::hash(label.as_bytes());
    let prefix: [u8; 8] = bytes.as_bytes()[..8].try_into().unwrap_or([0u8; 8]);
    u64::from_le_bytes(prefix)
}

/// Tranche B-6 quad -- system clock ms helper. Saturating-to-`i64::MAX`
/// on the `u128 -> i64` cast (year 292278994 + safe by construction);
/// `UNIX_EPOCH` shift errors collapse to `0` (clock set before 1970 is
/// advisory-only at the SSB ts boundary).
#[must_use]
pub fn system_time_ms_i64() -> i64 {
    // This helper IS the canonical timestamp source for ssb rows; the
    // ssb writer is by design where ClockProvider injection bottoms out.
    use std::time::{SystemTime, UNIX_EPOCH};
    let ms_u128 = SystemTime::now() // entropy-allow: ssb-canonical-timestamp-source-saturating-on-failure
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis())
        .unwrap_or(0);
    i64::try_from(ms_u128).unwrap_or(i64::MAX)
}


#[cfg(test)]
mod tests;
