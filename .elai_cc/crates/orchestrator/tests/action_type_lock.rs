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
const TOTAL_VARIANTS: usize = 399;

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
        SsbActionType::PerRoleDecodingApplied => (),
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
        // B-19 audit follow-up: close the 9-variant pre-existing drift.
        SsbActionType::DepAuditCompleted => (),
        SsbActionType::BaselineRunCompleted => (),
        SsbActionType::DeterministicAuditRun => (),
        SsbActionType::RuleTransition => (),
        SsbActionType::CanaryDrift => (),
        SsbActionType::TreePositionChanged => (),
        SsbActionType::FailureRecorded => (),
        SsbActionType::PackageRefVerified => (),
        SsbActionType::TechStackDecided => (),
        SsbActionType::MarkerReport => (),
        SsbActionType::EmpireConnected => (),
        SsbActionType::EmpireDisconnected => (),
        SsbActionType::EmpireEventReceived => (),
        SsbActionType::EmpireCapabilityInvoked => (),
        SsbActionType::EmpireRepairProposed => (),
        // C-5 follow-on (SCHEMA_VERSION 103).
        SsbActionType::RouteBlocked => (),
        SsbActionType::WarnLimitReached => (),
        SsbActionType::CostEstimate => (),
        // C-36 (SCHEMA_VERSION 104).
        SsbActionType::MucocoMutationApplied => (),
        // D-3 closure (SCHEMA_VERSION 105).
        SsbActionType::PpmUnavailable => (),
        SsbActionType::VerifierDisagreement => (),
        SsbActionType::CrossContextDispatched => (),
        // C-38 (SCHEMA_VERSION 107).
        SsbActionType::PagerankRun => (),
        SsbActionType::DkgRebuildStarted => (),
        SsbActionType::DkgRebuildComplete => (),
        // D-4 GAP-2 (SCHEMA_VERSION 108): concurrent-detector tripwire row.
        SsbActionType::TripwireFiredConcurrent => (),
        // C-40 (SCHEMA_VERSION 109): GitHub PR adapter producer surface.
        SsbActionType::GithubPrOpened => (),
        SsbActionType::GithubPrStatusFetched => (),
        SsbActionType::GithubPrClosed => (),
        SsbActionType::GithubPrMerged => (),
        // E-41 (SCHEMA_VERSION 135): GitHub PR review surface.
        SsbActionType::PrReviewComposed => (),
        SsbActionType::PrCommentWatched => (),
        SsbActionType::PrPackageVerifyResult => (),
        SsbActionType::PrSemgrepGateResult => (),
        SsbActionType::PrAutoMergeBlocked => (),
        // D-5 closure (SCHEMA_VERSION 110): Thompson-sample beacon.
        SsbActionType::ThompsonSampled => (),
        // C-41 (SCHEMA_VERSION 111): skills-driven workflow registry.
        SsbActionType::SkillTriggered => (),
        SsbActionType::SkillExecuted => (),
        // D-6 closure (SCHEMA_VERSION 112): held-out promotion-gate +
        // refresh-event beacons.
        SsbActionType::HeldOutPromotionGate => (),
        SsbActionType::HeldOutRefresh => (),
        // C-42 (SCHEMA_VERSION 113): Semantic Gateway L2 RBAC beacons.
        SsbActionType::GatewayPolicyAllowed => (),
        SsbActionType::GatewayPolicyDenied => (),
        SsbActionType::GatewayHitlPending => (),
        // D-7 closure (SCHEMA_VERSION 114): cross-domain canary observation
        // beacons. ELAI design hypothesis (NanoFlux arXiv:2509.23252
        // empirical motivation only; no canary mechanism in paper).
        SsbActionType::CrossDomainRegression => (),
        SsbActionType::CanaryFamilyPass => (),
        SsbActionType::CanaryInsufficientData => (),
        // D-8 closure (SCHEMA_VERSION 115): cryptographic archive verify.
        SsbActionType::ArchiveVerifyResult => (),
        // C-43 closure (SCHEMA_VERSION 116): Phase 4H scope-freeze.
        SsbActionType::ScopeFreezeRatified => (),
        SsbActionType::ScopeFreezeViolated => (),
        // C-44 (SCHEMA_VERSION 117): Composer 2 paid-backend cost accrual.
        SsbActionType::BackendCostAccrued => (),
        // C-45 (SCHEMA_VERSION 120): CANTANTE credit attribution.
        SsbActionType::AgentRoleDropped => (),
        SsbActionType::CreditAttributionRun => (),
        SsbActionType::CreditAttributionResult => (),
        // C-46 (SCHEMA_VERSION 121): Uno-Orchestra selective delegation.
        SsbActionType::SelectiveDelegationDecision => (),
        SsbActionType::SelectiveDelegationEscalated => (),
        SsbActionType::SelectiveDelegationCostSaved => (),
        // C-47 (SCHEMA_VERSION 122): Case-Based Calibration of Tool-Use.
        SsbActionType::ToolCallLogged => (),
        SsbActionType::ToolCalibrationHintEmitted => (),
        SsbActionType::ToolCalibrationDriftDetected => (),
        // C-49 (SCHEMA_VERSION 123): iterative-distillation experience.
        SsbActionType::PromptPolicyDistillationRun => (),
        SsbActionType::PromptFragmentSelected => (),
        SsbActionType::PromptFragmentDiscarded => (),
        // D-14 (SCHEMA_VERSION 124): sycophancy probe row pair.
        SsbActionType::SycophancyProbeDispatched => (),
        SsbActionType::SycophancyFlag => (),
        // C-50 (SCHEMA_VERSION 125): Contract-Coding scaffolding pair.
        SsbActionType::ContractTestScaffolded => (),
        SsbActionType::ContractViolationDetected => (),
        // C-52 (SCHEMA_VERSION 126): Dafny Formal Verification Spike.
        SsbActionType::DafnyVerifyCompleted => (),
        // D-16 closure (SCHEMA_VERSION 127): durable TauUpdater audit row.
        SsbActionType::CalibrationCorpusUpdated => (),
        // E-12 Tier-B (SCHEMA_VERSION 128): production fullstack codegen.
        SsbActionType::FrameworkProfileSelected => (),
        SsbActionType::LegacyConfigStripped => (),
        // D-18 closure (SCHEMA_VERSION 129): typed-ObservableEffect 4-pack.
        SsbActionType::ObservableEffectDispatched => (),
        SsbActionType::GoodhartRiskFlag => (),
        SsbActionType::SandboxObservation => (),
        SsbActionType::ProductionObservationProposal => (),
        // E-13 (SCHEMA_VERSION 130): TDD verifier-agent beacons.
        SsbActionType::TddCycleStarted => (),
        SsbActionType::TddTestFailed => (),
        SsbActionType::TddTestPassed => (),
        // E-14 (SCHEMA_VERSION 131): browser-e2e agent beacons.
        SsbActionType::BrowserE2eRunStarted => (),
        SsbActionType::BrowserE2eAssertionFailed => (),
        SsbActionType::BrowserE2eAssertionPassed => (),
        SsbActionType::BrowserE2eSnapshotCaptured => (),
        // E-18 (SCHEMA_VERSION 132): tool-use reliability gate beacons.
        SsbActionType::ToolReliabilityGatePassed => (),
        SsbActionType::ToolReliabilityGateRejected => (),
        SsbActionType::ToolCallSpilled => (),
        SsbActionType::ToolCallRetryEscalated => (),
        // E-37 (SCHEMA_VERSION 133): prompt-injection defense beacons.
        SsbActionType::InjectionAttemptDetected => (),
        SsbActionType::InjectionAttemptBlocked => (),
        SsbActionType::SpotlightWrapped => (),
        SsbActionType::SpotlightBreached => (),
        SsbActionType::ToolDescriptionPinMismatch => (),
        SsbActionType::CapabilityGateRejected => (),
        SsbActionType::ProvenanceTagged => (),
        SsbActionType::PromptArmorVerdict => (),
        SsbActionType::InjectionFixtureReplayed => (),
        // E-40 (SCHEMA_VERSION 134): orchestrator-rooted symbol-graph beacons.
        SsbActionType::SymbolGraphQueried => (),
        SsbActionType::SymbolGraphCacheLoaded => (),
        SsbActionType::LspOracleConsulted => (),
        // E-44 (SCHEMA_VERSION 136): durability checkpoint + idempotent dispatch +
        // compact-lock beacons.
        SsbActionType::CheckpointWritten => (),
        SsbActionType::CheckpointResumed => (),
        SsbActionType::CheckpointMigrated => (),
        SsbActionType::IdempotentDispatchSkipped => (),
        SsbActionType::DurabilityLockBlocked => (),
        // E-45 (SCHEMA_VERSION 137): cross-language drift detector beacons.
        SsbActionType::CrossLangDriftDetected => (),
        SsbActionType::CrossLangPairScanned => (),
        SsbActionType::SchemaVersionMismatch => (),
        // E-49 (SCHEMA_VERSION 138): coding-agent metric suite beacons.
        SsbActionType::CodingAgentMetricsCaptured => (),
        SsbActionType::SweBenchProRunStarted => (),
        SsbActionType::SweBenchProRunCompleted => (),
        SsbActionType::RefineBenchScoreEmitted => (),
        SsbActionType::MetricsAggregationRun => (),
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
