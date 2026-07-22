#![allow(unused_imports)]
// SSB unit tests (moved verbatim from ssb.rs `mod tests`, Tier-B Move 1).
use crate::ssb::{schema::*, action_type::*, payloads::*, store::*};
use crate::front_door::{ArgSpan, FrontDoorLane, FrontDoorTier};
use crate::gate::RiskLevel;

    use super::*;
    use elai_core::clock::SeededClock;
    use std::time::{Duration, SystemTime};
    use tempfile::TempDir;

    // E-25 (SCHEMA_VERSION 172) -- catalog enums for the
    // ClaudeCodeArchitectureEvent sample-payload constructors below.
    use crate::architecture::claude_code_arch::{
        ClaudeCodeArchPrimitive, ClaudeCodeArchitectureEventKind,
    };

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
                test_name: "t".into(), passed: true, duration_ms: 1, stdout_hash: None,
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
                origin_role: None, origin_run_id: None,
                originating_mast_mode: None, content_hash: None, provisional: false,
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
                failed_files: 0,
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
                phase_id: None,
                complexity_reason: None,
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
            // B-30 (SCHEMA_VERSION 36): language grammar registry status.
            SsbActionType::LanguageGrammarStatus => {
                SsbPayload::LanguageGrammarStatus(LanguageGrammarStatusPayload {
                    loaded: vec!["go".into(), "python".into(), "rust".into()],
                    failed: vec![],
                })
            }
            // B-29 Phase 5 (SCHEMA_VERSION 37): symbol-graph tool call row.
            SsbActionType::SymbolQuery => {
                SsbPayload::SymbolQuery(SymbolQueryPayload {
                    tool: "symbol_impact".into(),
                    target: "src/lib.rs".into(),
                    hop_count: 2,
                    total_files: 3,
                    latency_ms: 5,
                })
            }
            // B-32 (SCHEMA_VERSION 38): minified-file chunking row.
            SsbActionType::MinifiedFileChunked => {
                SsbPayload::MinifiedFileChunked(MinifiedFileChunkedPayload {
                    relative_path: "vendor/bundle.min.js".into(),
                    char_count: 50_000,
                    chunk_count: 25,
                    avg_line_length: 50_000,
                })
            }
            // B-31.5 (SCHEMA_VERSION 39): 5 deferred B-31 SSB variants.
            SsbActionType::IndexCheckpoint => {
                SsbPayload::IndexCheckpoint(IndexCheckpointPayload {
                    project_id: "p".into(),
                    batches_processed: 1,
                    batches_total: Some(4),
                    files_processed: 10,
                    last_checkpoint_at: 0,
                    batch_duration_ms: Some(120),
                })
            }
            SsbActionType::IndexResume => SsbPayload::IndexResume(IndexResumePayload {
                project_id: "p".into(),
                resumed_from_batch: 1,
                files_skipped: 20,
            }),
            SsbActionType::IndexCancel => SsbPayload::IndexCancel(IndexCancelPayload {
                project_id: "p".into(),
                batches_completed: 1,
                files_indexed: 5,
            }),
            SsbActionType::LockAcquired => SsbPayload::LockAcquired(LockAcquiredPayload {
                project_id: "p".into(),
                scope: "index".into(),
                holder_pid: 12345,
            }),
            SsbActionType::LockContested => SsbPayload::LockContested(LockContestedPayload {
                project_id: "p".into(),
                scope: "index".into(),
                holder_pid: 12345,
            }),
            // B-33 follow-up (SCHEMA_VERSION 40): symbol-graph build event row.
            SsbActionType::SymbolGraphUpdated => {
                SsbPayload::SymbolGraphUpdated(SymbolGraphUpdatedPayload {
                    project_id: "p".into(),
                    file_count: 16,
                    symbol_count: 1_024,
                    edge_count: 4_096,
                    built_ms: 0,
                    // C-24 (DRIFT-4): default None (additive field; full-build
                    // path leaves the flag unset).
                    is_incremental: None,
                })
            }
            // B-40 (SCHEMA_VERSION 41): FRAMING substate beacons.
            SsbActionType::IdeaFramingStarted => SsbPayload::IdeaFramingStarted(
                IdeaFramingStartedPayload {
                    session_id: "s".into(),
                    prompt: "p".into(),
                    started_at_ms: 0,
                },
            ),
            SsbActionType::ProjectClassClassified => SsbPayload::ProjectClassClassified(
                ProjectClassClassifiedPayload {
                    session_id: "s".into(),
                    project_class: "other".into(),
                    matched_pattern: None,
                },
            ),
            SsbActionType::CriticRoundComplete => SsbPayload::CriticRoundComplete(
                CriticRoundCompletePayload {
                    session_id: "s".into(),
                    round_index: 1,
                    block_count: 0,
                    warn_count: 0,
                    info_count: 0,
                    converged: true,
                    category_counts: Vec::new(),
                },
            ),
            SsbActionType::IdeaFramed => SsbPayload::IdeaFramed(IdeaFramedPayload {
                session_id: "s".into(),
                artifact_path: "framed-idea.toml".into(),
                project_class: "other".into(),
                rounds_used: 1,
                tradeoff_count: 2,
                converged: true,
            }),
            SsbActionType::TradeoffCardEmitted => SsbPayload::TradeoffCardEmitted(
                TradeoffCardEmittedPayload {
                    session_id: "s".into(),
                    tradeoff_names: Vec::new(),
                    option_counts: Vec::new(),
                },
            ),
            // B-34 (SCHEMA_VERSION 42)
            SsbActionType::EntryPointsDetected => SsbPayload::EntryPointsDetected(
                EntryPointsDetectedPayload {
                    project_id: "p".into(),
                    count: 0,
                    by_reason: std::collections::BTreeMap::new(),
                },
            ),
            SsbActionType::ArtifactsIndexed => SsbPayload::ArtifactsIndexed(
                ArtifactsIndexedPayload {
                    project_id: "p".into(),
                    indexed: Vec::new(),
                    up_to_date: Vec::new(),
                    total_chunks: 0,
                    latency_ms: 0,
                },
            ),
            SsbActionType::ArtifactSearched => SsbPayload::ArtifactSearched(
                ArtifactSearchedPayload {
                    project_id: "p".into(),
                    artifact_name: None,
                    query: "q".into(),
                    result_count: 0,
                    latency_ms: 0,
                },
            ),
            SsbActionType::ArtifactStale => SsbPayload::ArtifactStale(
                ArtifactStalePayload {
                    project_id: "p".into(),
                    artifact_name: "a".into(),
                    changed_files: Vec::new(),
                },
            ),
            // B-38 (SCHEMA_VERSION 43)
            SsbActionType::AmbiguityScored => SsbPayload::AmbiguityScored(
                AmbiguityScoredPayload {
                    score: 0.15,
                    milestone: "ready".into(),
                    goal: 0.9,
                    constraint: 0.85,
                    success: 0.85,
                    context: None,
                    brownfield: false,
                },
            ),
            SsbActionType::AmbiguityFloorFailed => SsbPayload::AmbiguityFloorFailed(
                AmbiguityFloorFailedPayload {
                    score: 0.1,
                    failed_dimensions: vec!["goal_clarity".into()],
                },
            ),
            SsbActionType::AmbiguityStreakAdvanced => SsbPayload::AmbiguityStreakAdvanced(
                AmbiguityStreakAdvancedPayload {
                    score: 0.18,
                    current_streak: 1,
                    required: 2,
                },
            ),
            // B-42 (SCHEMA_VERSION 44): cosmos tier knowledge worker beacons.
            SsbActionType::CosmosRefreshStarted => SsbPayload::CosmosRefreshStarted(
                CosmosRefreshStartedPayload { queued: 0, total: 0 },
            ),
            SsbActionType::CosmosArtifactUpdated => SsbPayload::CosmosArtifactUpdated(
                CosmosArtifactUpdatedPayload {
                    name: "a".into(),
                    blake3: "0".into(),
                    body_bytes: 0,
                    latency_ms: 0,
                },
            ),
            SsbActionType::CosmosArtifactDecayed => SsbPayload::CosmosArtifactDecayed(
                CosmosArtifactDecayedPayload {
                    name: "a".into(),
                    age_secs: 0,
                    cadence: "daily".into(),
                },
            ),
            SsbActionType::CosmosGroundingUsed => SsbPayload::CosmosGroundingUsed(
                CosmosGroundingUsedPayload {
                    claim_category: "design".into(),
                    artifact_name: "a".into(),
                    matched: true,
                },
            ),
            // B-41 (SCHEMA_VERSION 45): per-level auditor mode beacons.
            SsbActionType::PerLevelAuditStarted => SsbPayload::PerLevelAuditStarted(
                PerLevelAuditStartedPayload {
                    mode: "concepts".into(),
                    tier: "concept".into(),
                    node_id: 0,
                },
            ),
            SsbActionType::PerLevelAuditVerdict => SsbPayload::PerLevelAuditVerdict(
                PerLevelAuditVerdictPayload {
                    mode: "concepts".into(),
                    tier: "concept".into(),
                    node_id: 0,
                    verdict: "allow".into(),
                    findings_count: 0,
                    respawned_once: false,
                    cascade_to_parent: false,
                },
            ),
            SsbActionType::PerLevelAuditCascade => SsbPayload::PerLevelAuditCascade(
                PerLevelAuditCascadePayload {
                    mode: "babystep_detail".into(),
                    tier: "baby_step".into(),
                    node_id: 0,
                    parent_tier: "baby_step".into(),
                },
            ),
            // B-37C (SCHEMA_VERSION 46): multi-runtime backend beacons.
            SsbActionType::RuntimeFallback => SsbPayload::RuntimeFallback(
                RuntimeFallbackPayload {
                    role: elai_ipc_types::AgentRole::Executor,
                    primary: elai_providers::runtime::RuntimeBackend::Sidecar,
                    attempted: vec![
                        elai_providers::runtime::RuntimeBackend::Sidecar,
                        elai_providers::runtime::RuntimeBackend::AnthropicApi,
                    ],
                    succeeded: Some(elai_providers::runtime::RuntimeBackend::AnthropicApi),
                    reason: "sidecar unavailable, anthropic_api succeeded".into(),
                },
            ),
            SsbActionType::RuntimeHealthChanged => SsbPayload::RuntimeHealthChanged(
                RuntimeHealthChangedPayload {
                    backend: elai_providers::runtime::RuntimeBackend::Sidecar,
                    prev: elai_providers::runtime::RuntimeHealth::Unavailable {
                        reason: "first observation".into(),
                    },
                    next: elai_providers::runtime::RuntimeHealth::Ready,
                },
            ),
            SsbActionType::CliSubprocessSpawned => SsbPayload::CliSubprocessSpawned(
                CliSubprocessSpawnedPayload {
                    backend: elai_providers::runtime::RuntimeBackend::ClaudeCode,
                    pid: 12345,
                    argv_redacted: vec!["claude".into(), "-p".into(), "[REDACTED]".into()],
                },
            ),
            SsbActionType::CliSubprocessKilled => SsbPayload::CliSubprocessKilled(
                CliSubprocessKilledPayload {
                    backend: elai_providers::runtime::RuntimeBackend::ClaudeCode,
                    pid: 12345,
                    exit_code: Some(0),
                    reason: "normal exit".into(),
                },
            ),
            // E-51 (SCHEMA_VERSION 96): edge-inference beacons.
            SsbActionType::MlxSubprocessSpawned => SsbPayload::MlxSubprocessSpawned(
                MlxSubprocessSpawnedPayload { pid: 22222, argv_redacted: vec!["vllm-mlx".into()] }
            ),
            SsbActionType::MlxSubprocessKilled => SsbPayload::MlxSubprocessKilled(
                MlxSubprocessKilledPayload { pid: 22222, exit_code: Some(0), reason: "normal exit".into() }
            ),
            SsbActionType::LlamaCppSubprocessSpawned => SsbPayload::LlamaCppSubprocessSpawned(
                LlamaCppSubprocessSpawnedPayload { pid: 33333, ep: "cpu".into(), argv_redacted: vec!["llama-server".into()] }
            ),
            SsbActionType::LlamaCppSubprocessKilled => SsbPayload::LlamaCppSubprocessKilled(
                LlamaCppSubprocessKilledPayload { pid: 33333, exit_code: Some(0), reason: "normal exit".into() }
            ),
            SsbActionType::MlxApplePlatformDetected => SsbPayload::MlxApplePlatformDetected(
                MlxApplePlatformDetectedPayload { chip_family: "M5".into(), gpu_cores: 40 }
            ),
            SsbActionType::LlamaCppEpDetected => SsbPayload::LlamaCppEpDetected(
                LlamaCppEpDetectedPayload { ep: "cpu".into() }
            ),
            SsbActionType::RuntimeChainBuilt => SsbPayload::RuntimeChainBuilt(
                RuntimeChainBuiltPayload { chain: vec![elai_providers::runtime::RuntimeBackend::Sidecar] }
            ),
            // E-17 SP2 (SCHEMA_VERSION 97): RAG retrieval-quality beacons.
            SsbActionType::DeepPoolCapped => SsbPayload::DeepPoolCapped(
                DeepPoolCappedPayload { capped_count: 150, max_allowed: 100, stage_count: 5 }
            ),
            SsbActionType::RrfQualityGateFired => SsbPayload::RrfQualityGateFired(
                RrfQualityGateFiredPayload { dropped_count: 12, floor: 0.005, total_before: 80 }
            ),
            SsbActionType::HydeSkippedCodeQuery => SsbPayload::HydeSkippedCodeQuery(
                HydeSkippedCodeQueryPayload {
                    query_class: "code_symbol".into(),
                    word_count: 3,
                    code_signal_present: true,
                }
            ),
            SsbActionType::RrfComponentWeights => SsbPayload::RrfComponentWeights(
                RrfComponentWeightsPayload {
                    bm25_weight: Some(1.0),
                    code_vec_weight: Some(1.0),
                    text_vec_weight: Some(0.8),
                }
            ),
            // C-1 (SCHEMA_VERSION 47): knowledge + reasoning producer beacons.
            SsbActionType::GrcStepEmitted => SsbPayload::GrcStepEmitted(GrcStepEmittedPayload {
                agent_id: "executor".into(),
                topic: "task_runner".into(),
                step_type: "observe".into(),
                grounding_level: "high".into(),
                confidence: 0.85,
                budget_remaining: 5,
                elapsed_ms: 12,
            }),
            SsbActionType::GroundingSweepResult => {
                SsbPayload::GroundingSweepResult(GroundingSweepResultPayload {
                    ungrounded: 1,
                    verify_needed: 0,
                    grounded: 3,
                    top_claim: Some("training:placeholder".into()),
                })
            }
            SsbActionType::CsqEvaluated => SsbPayload::CsqEvaluated(CsqEvaluatedPayload {
                severity: "Warning".into(),
                guard_present: true,
                blocked: false,
                check_count: 2,
            }),
            SsbActionType::AuditorRubberStampRejected => SsbPayload::AuditorRubberStampRejected(
                AuditorRubberStampRejectedPayload {
                    role: "auditor".into(),
                    finding_count: 0,
                    expected_min: 1,
                },
            ),
            SsbActionType::PerRoleDecodingApplied => SsbPayload::PerRoleDecodingApplied(
                PerRoleDecodingAppliedPayload {
                    role: "auditor".into(),
                    decoding_mode: "json_schema".into(),
                    constrained: true,
                    max_retries_on_invalid: 3,
                },
            ),
            SsbActionType::HydeQuery => SsbPayload::HydeQuery(HydeQueryPayload {
                query_hash: "abc123".into(),
                hypothesis_len: 128,
                cache_hit: false,
                latency_ms: 12,
            }),
            SsbActionType::VectorCacheHitRate => {
                SsbPayload::VectorCacheHitRate(VectorCacheHitRatePayload {
                    session_id: "test-session".into(),
                    hits: 4,
                    misses: 1,
                    hit_rate_pct: 80.0,
                })
            }
            SsbActionType::QueryTrace => SsbPayload::QueryTrace(QueryTracePayload {
                query_id: "11111111-2222-3333-4444-555555555555".into(),
                tier: "STANDARD".into(),
                stage_count: 9,
                result_count: 12,
                total_latency_ms: 245,
            }),
            // C-2 (SCHEMA_VERSION 48): orchestration + cosmos + stagnation beacons.
            SsbActionType::PhaseNodeReady => SsbPayload::PhaseNodeReady(PhaseNodeReadyPayload {
                phase_id: 7,
                concept_id: 3,
                label: "phase-test".into(),
                depth: 1,
            }),
            SsbActionType::StepNodeReady => SsbPayload::StepNodeReady(StepNodeReadyPayload {
                step_id: 12,
                phase_id: 7,
                concept_id: 3,
                label: "step-test".into(),
                depth: 2,
            }),
            SsbActionType::BabyStepReady => SsbPayload::BabyStepReady(BabyStepReadyPayload {
                baby_step_id: 42,
                step_id: 12,
                phase_id: 7,
                concept_id: 3,
                label: "baby-step-test".into(),
                depth: 3,
            }),
            SsbActionType::DagRepairEscalated => SsbPayload::DagRepairEscalated(
                DagRepairEscalatedPayload {
                    rounds: 10,
                    remaining_issues: 2,
                    last_error: "cycle persists after repair".into(),
                },
            ),
            SsbActionType::AgentStatusChanged => SsbPayload::AgentStatusChanged(
                AgentStatusChangedPayload {
                    agent_id: "executor-1".into(),
                    role: "executor".into(),
                    state: "spawned".into(),
                    death_class: None,
                    timestamp_ms: 1_700_000_000_000,
                },
            ),
            SsbActionType::StagnationSpinningDetected => SsbPayload::StagnationSpinningDetected(
                StagnationSpinningDetectedPayload {
                    agent_id: "executor-1".into(),
                    pattern: "spinning".into(),
                    iteration: 3,
                    detail: "step_hash repeated 3 times".into(),
                },
            ),
            SsbActionType::StagnationOscillationDetected => {
                SsbPayload::StagnationOscillationDetected(
                    StagnationOscillationDetectedPayload {
                        agent_id: "executor-1".into(),
                        pattern: "oscillation".into(),
                        iteration: 4,
                        detail: "period-2 A-B-A-B".into(),
                    },
                )
            }
            SsbActionType::LateralThinkingActivated => SsbPayload::LateralThinkingActivated(
                LateralThinkingActivatedPayload {
                    agent_id: "executor-1".into(),
                    persona: "contrarian".into(),
                    iteration: 5,
                    detail: "switched after spinning".into(),
                },
            ),
            SsbActionType::GoalDriftWarning => SsbPayload::GoalDriftWarning(
                GoalDriftWarningPayload {
                    agent_id: "executor-1".into(),
                    drift_score: 0.42,
                    baby_step_id: 17,
                    detail: "task drifted from intent".into(),
                },
            ),
            SsbActionType::WatchdogTimeout => SsbPayload::WatchdogTimeout(WatchdogTimeoutPayload {
                agent_id: "executor-1".into(),
                elapsed_ms: 30_000,
                budget_ms: 20_000,
                detail: "no observable output".into(),
            }),
            // C-10 (SCHEMA_VERSION 52) — lateral-thinking + goal-drift Block.
            SsbActionType::LateralThinkingSucceeded => SsbPayload::LateralThinkingSucceeded(
                LateralThinkingSucceededPayload {
                    agent_id: "executor-1".into(),
                    persona: "hacker".into(),
                    iteration: 6,
                    detail: "pattern broken after hacker persona".into(),
                },
            ),
            SsbActionType::LateralThinkingExhausted => SsbPayload::LateralThinkingExhausted(
                LateralThinkingExhaustedPayload {
                    agent_id: "executor-1".into(),
                    personas_tried: 5,
                    iteration: 10,
                    detail: "all personas tried; pattern persists".into(),
                },
            ),
            SsbActionType::GoalDriftBlock => SsbPayload::GoalDriftBlock(GoalDriftBlockPayload {
                agent_id: "executor-1".into(),
                drift_score: 0.32,
                baby_step_id: 19,
                detail: "drift_score below block threshold".into(),
            }),
            // C-7 (SCHEMA_VERSION 49): reviewer verdict beacon.
            SsbActionType::ChangeDecision => SsbPayload::ChangeDecision(ChangeDecisionPayload {
                decision: "accepted".into(),
                path: "src/foo.rs".into(),
                hunk_id: Some("hunk-1".into()),
                note: None,
            }),
            // C-8 (SCHEMA_VERSION 50): search-mode cross-process bridge.
            SsbActionType::SearchModeChanged => SsbPayload::SearchModeChanged(
                SearchModeChangedPayload {
                    mode: "hybrid".into(),
                    history: vec!["lexical".into(), "hybrid".into()],
                },
            ),
            // C-5 (SCHEMA_VERSION 51): privacy-redaction event.
            SsbActionType::RedactionApplied => SsbPayload::RedactionApplied(
                RedactionAppliedPayload {
                    count: 3,
                    fields: vec!["task_summary".into(), "escalation_message".into()],
                    baby_step_id: 17,
                    ts_ms: 0,
                },
            ),
            // C-13 (SCHEMA_VERSION 53): WIP commit squash completion.
            SsbActionType::WipSquashComplete => SsbPayload::WipSquashComplete(
                WipSquashCompletePayload {
                    wip_count: 7,
                    squashed: true,
                    step_id: "step-42".into(),
                },
            ),
            // C-22 (SCHEMA_VERSION 54): feasibility-probe completeness beacons.
            SsbActionType::AlternativeExplored => SsbPayload::AlternativeExplored(
                AlternativeExploredPayload {
                    step_id: "step-c22".into(),
                    alternative_index: 1,
                    accepted: true,
                },
            ),
            SsbActionType::FeasibilityEscalated => SsbPayload::FeasibilityEscalated(
                FeasibilityEscalatedPayload {
                    reason: "vram_margin_breached".into(),
                    variant: "resource_margin_breached".into(),
                    step_id: "step-c22".into(),
                },
            ),
            // C-23 (SCHEMA_VERSION 55): SessionDag branch-UI beacons.
            SsbActionType::BranchForked => SsbPayload::BranchForked(BranchForkedPayload {
                branch_name: "feature-x".into(),
                tip_snap_id: "snap-c23-tip".into(),
                parent_snap_id: "snap-c23-parent".into(),
            }),
            SsbActionType::BranchMerged => SsbPayload::BranchMerged(BranchMergedPayload {
                winner_name: "feature-x".into(),
                trunk_snap_id: "snap-c23-trunk".into(),
                abandoned_names: vec!["experiment-y".into()],
            }),
            SsbActionType::BranchClosed => SsbPayload::BranchClosed(BranchClosedPayload {
                branch_name: "experiment-y".into(),
                reason: "user-requested".into(),
            }),
            // C-25 (SCHEMA_VERSION 56): compiled-pipeline + template-crystallizer
            // + background-task beacons.
            SsbActionType::PlanSkeleton => SsbPayload::PlanSkeleton(PlanSkeletonPayload {
                plan_id: "plan-c25-test".into(),
                step_count: 3,
                concept_class: "Counter".into(),
                plan_skeleton_hash: 0xC25_AAAA_BBBB_CCCC,
                source: PlanSkeletonSource::Llm,
            }),
            SsbActionType::CompiledPipelineFired => SsbPayload::CompiledPipelineFired(
                CompiledPipelineFiredPayload {
                    pipeline_id: "pl-c25-test".into(),
                    concept_class: "Counter".into(),
                    plan_skeleton_hash: 0xC25_AAAA_BBBB_CCCC,
                    use_count: 4,
                },
            ),
            SsbActionType::TemplateCrystallized => SsbPayload::TemplateCrystallized(
                TemplateCrystallizedPayload {
                    concept_class: "Counter".into(),
                    plan_skeleton_hash: 0xC25_AAAA_BBBB_CCCC,
                    pipeline_id: "pl-c25-test".into(),
                    threshold_n: 3,
                },
            ),
            SsbActionType::BackgroundTaskStarted => SsbPayload::BackgroundTaskStarted(
                BackgroundTaskStartedPayload {
                    task_id: "bg-c25-test".into(),
                    command_summary: "grep -r foo /large".into(),
                    soft_timeout_secs: 30,
                    hard_timeout_secs: 600,
                },
            ),
            SsbActionType::BackgroundTaskComplete => SsbPayload::BackgroundTaskComplete(
                BackgroundTaskCompletePayload {
                    task_id: "bg-c25-test".into(),
                    exit_status: 0,
                    duration_secs: 12,
                    auto_detached: false,
                    timed_out: false,
                },
            ),
            // C-25 audit follow-up (SCHEMA_VERSION 57): missing 6th
            // compiled-pipeline beacon (WARN-1 in audit-C25.md).
            SsbActionType::DecompileTriggered => SsbPayload::DecompileTriggered(
                DecompileTriggeredPayload {
                    concept_class: "Counter".into(),
                    plan_skeleton_hash: 0xC25_AAAA_BBBB_CCCC,
                    failure_count: 2,
                    trigger_reason: DecompileTriggerReason::ConsecutiveFailures,
                },
            ),
            // C-26 (SCHEMA_VERSION 58): constrained-decoding + futility primitives.
            SsbActionType::FutilityDetected => SsbPayload::FutilityDetected(
                FutilityDetectedPayload {
                    error_class: "constrained_decode_reject".into(),
                    consecutive_count: 3,
                    level: "baby_step".into(),
                    escalated: true,
                },
            ),
            SsbActionType::FutilityOutputStagnation => SsbPayload::FutilityOutputStagnation(
                FutilityOutputStagnationPayload {
                    similarity: 0.95,
                    threshold: 0.90,
                    level: "baby_step".into(),
                    new_persona: Some("hacker".into()),
                },
            ),
            // C-27 (SCHEMA_VERSION 59): Testing-trust T(-1) + T-0 beacons.
            // audit_id = Uuid::nil() keeps the test fixture inside the entropy
            // quarantine — see C-15 D7 + DRIFT-11.
            SsbActionType::WiringEvidence => SsbPayload::WiringEvidence(
                WiringEvidencePayload {
                    entrypoint: "crates/cli/src/main.rs::main".into(),
                    produces_symbols_verified: vec!["DeterminismConfig".into()],
                    reachable_count: 1,
                    audit_id: uuid::Uuid::nil(),
                },
            ),
            SsbActionType::WiringViolation => SsbPayload::WiringViolation(
                WiringViolationPayload {
                    violation_kind: WiringViolationKind::StubPattern,
                    file: "crates/orchestrator/src/example.rs".into(),
                    line: 42,
                    pattern: "todo!()".into(),
                    audit_id: uuid::Uuid::nil(),
                },
            ),
            SsbActionType::DeterminismDrift => SsbPayload::DeterminismDrift(
                DeterminismDriftPayload {
                    seed: 42,
                    run_a_hash: "blake3:a".into(),
                    run_b_hash: "blake3:b".into(),
                    first_divergence_index: 7,
                    audit_id: uuid::Uuid::nil(),
                },
            ),
            // C-28 (SCHEMA_VERSION 60): T-1 mutation testing — one row per
            // surviving mutant. Fixture pins file/line/mutant_kind for the
            // SQL CHECK roundtrip test (test_typed_payloads).
            SsbActionType::MutationSurvived => SsbPayload::MutationSurvived(
                MutationSurvivedPayload {
                    file: "crates/orchestrator/src/example.rs".into(),
                    line: 7,
                    mutant_kind: "replace_+_with_-".into(),
                },
            ),
            // C-29 (SCHEMA_VERSION 61): T-2 IntentPair confirmation +
            // T-3 metamorphic-relation violation. Both fixtures pin
            // canonical payload shapes for the test_typed_payloads
            // roundtrip; MrViolated.counterexample MUST be non-empty
            // (stub_ban contract).
            SsbActionType::IntentConfirmed => SsbPayload::IntentConfirmed(
                IntentConfirmedPayload {
                    step_id: 1,
                    pair_count: 3,
                    confirmed_by: "user".into(),
                },
            ),
            SsbActionType::MrViolated => SsbPayload::MrViolated(MrViolatedPayload {
                step_id: 1,
                relation: "inverse".into(),
                counterexample: "sample_index=0 input_hex=01 encoded_hex=ff decoded_hex=01".into(),
                sample_size: 1,
            }),
            // C-30 (SCHEMA_VERSION 62): Surgical Mode + Oracle Mode payloads.
            SsbActionType::SurgicalPlanCreated => SsbPayload::SurgicalPlanCreated(
                SurgicalPlanCreatedPayload {
                    task_id: "task_1700000000_c30test".into(),
                    complexity_class: "simple".into(),
                    blast_radius_files: 2,
                    total_loc_estimate: 150,
                    step_count: 1,
                    regression_gate_snapshot_hash: "blake3:c30aaaaa".into(),
                },
            ),
            SsbActionType::SurgicalLocWarning => SsbPayload::SurgicalLocWarning(
                SurgicalLocWarningPayload {
                    task_id: "task_1700000000_c30test".into(),
                    total_loc_estimate: 600,
                    threshold: 500,
                    mode: "interactive".into(),
                },
            ),
            SsbActionType::OracleCacheHit => SsbPayload::OracleCacheHit(OracleCacheHitPayload {
                query_hash_hex: "00".repeat(32),
                source_tier: "t1_authoritative".into(),
                cached_age_secs: 60,
                source: None,
            }),
            SsbActionType::OracleCacheMiss => SsbPayload::OracleCacheMiss(OracleCacheMissPayload {
                query_hash_hex: "00".repeat(32),
                confidence: "high".into(),
                source: None,
            }),
            SsbActionType::ProjectDetectionSignal => SsbPayload::ProjectDetectionSignal(
                ProjectDetectionSignalPayload {
                    signal_name: "cargo_toml".into(),
                    weight: "required".into(),
                    fired: true,
                },
            ),
            // C-32 (SCHEMA_VERSION 63): Code-Completeness Layer 2 payloads.
            SsbActionType::SkeletonGenerated => SsbPayload::SkeletonGenerated(
                SkeletonGeneratedPayload {
                    blank_count: 4,
                    language: "rust".into(),
                    file_path: "src/lib.rs".into(),
                },
            ),
            SsbActionType::SkeletonFlushed => SsbPayload::SkeletonFlushed(
                SkeletonFlushedPayload {
                    path: "src/lib.rs".into(),
                    completeness_ratio: 1.0,
                },
            ),
            SsbActionType::MiniCriticVerdict => SsbPayload::MiniCriticVerdict(
                MiniCriticVerdictPayload {
                    blank_id: 1,
                    verdict: "pass".into(),
                    reason: "ok".into(),
                    language: "rust".into(),
                },
            ),
            SsbActionType::StructuralRetryStageAttempt => SsbPayload::StructuralRetryStageAttempt(
                StructuralRetryStageAttemptPayload {
                    blank_id: 1,
                    stage: "template".into(),
                    attempt: 1,
                    outcome: "converged".into(),
                },
            ),
            SsbActionType::BridgeFileGenerated => SsbPayload::BridgeFileGenerated(
                BridgeFileGeneratedPayload {
                    kind: "mod_rs_aggregator".into(),
                    path: "src/mod.rs".into(),
                    module_count: 3,
                },
            ),
            // D-1 (SCHEMA_VERSION 64): ReasoningTemplate ArchiveChain lifecycle.
            SsbActionType::ArchiveEntryAdded => SsbPayload::ArchiveEntryAdded(
                ArchiveEntryAddedPayload {
                    template_id: "00000000000000000000000000000001".into(),
                    content_hash: "ff".repeat(32),
                    parent_hash: None,
                    domain: "code".into(),
                    pattern_kind: "decomposition".into(),
                    archive_status: "frozen".into(),
                },
            ),
            SsbActionType::ArchiveEntryRetired => SsbPayload::ArchiveEntryRetired(
                ArchiveEntryRetiredPayload {
                    template_id: "00000000000000000000000000000001".into(),
                    retired_at_ms: 1_700_000_000_000,
                    retirement_reason: "degradation".into(),
                },
            ),
            SsbActionType::TemplatePromoted => SsbPayload::TemplatePromoted(
                TemplatePromotedPayload {
                    template_id: "00000000000000000000000000000001".into(),
                    stage: "stage_50".into(),
                    pass_rate: 0.94,
                },
            ),
            SsbActionType::TemplateCandidateRejected => SsbPayload::TemplateCandidateRejected(
                TemplateCandidateRejectedPayload {
                    template_id: "00000000000000000000000000000001".into(),
                    rejection_reason: "below_floor".into(),
                },
            ),
            // E-5 (SCHEMA_VERSION 65): memory_facts bi-temporal lifecycle beacons.
            SsbActionType::MemoryFactExtracted => SsbPayload::MemoryFactExtracted(
                MemoryFactExtractedPayload {
                    fact_id: "fact-test-extract".into(),
                    fact_type: "preference".into(),
                    scope: "project".into(),
                    project_id: Some("proj-test".into()),
                    user_id: "default".into(),
                    key: "prefers_tracing_over_log".into(),
                    trust_score: 0.9,
                },
            ),
            SsbActionType::MemoryFactSuperseded => SsbPayload::MemoryFactSuperseded(
                MemoryFactSupersededPayload {
                    old_fact_id: "fact-test-old".into(),
                    new_fact_id: "fact-test-new".into(),
                    valid_to_ms: 1_700_000_000_000,
                },
            ),
            // D-2 (SCHEMA_VERSION 66): PPM verifier beacons.
            SsbActionType::PpmScored => SsbPayload::PpmScored(PpmScoredPayload {
                candidate_id: "cand-test-1".into(),
                verifier_id: "ppm-test".into(),
                mode: "math".into(),
                score_mean: 0.91,
                score_var: 0.0004,
                num_steps: 3,
                latency_ms: 412,
                cache_hit: false,
            }),
            SsbActionType::PpmGatingFailed => SsbPayload::PpmGatingFailed(
                PpmGatingFailedPayload {
                    candidate_id: "cand-test-2".into(),
                    verifier_id: "ppm-test".into(),
                    mode: "math".into(),
                    score_mean: 0.42,
                    threshold: 0.7,
                    num_steps: 2,
                },
            ),
            // E-1 (SCHEMA_VERSION 67): infinite-context chat illusion beacons.
            SsbActionType::ChatReflectionCompleted => SsbPayload::ChatReflectionCompleted(
                ChatReflectionCompletedPayload {
                    note_id: "note-test-reflect".into(),
                    note_preview: "U: i prefer blake3...".into(),
                    facts_extracted: 1,
                    decay_rows_examined: 4,
                    decay_rows_updated: 1,
                    core_block_bytes: 256,
                },
            ),
            SsbActionType::MemoryEvolutionRun => SsbPayload::MemoryEvolutionRun(
                MemoryEvolutionRunPayload {
                    phase: "end".into(),
                    duration_ms: 1_240,
                    notes_touched: 8,
                },
            ),
            // E-1 dashboard slice (SCHEMA_VERSION 68): MemoryNotes / CoreBlock
            // producer + conflict beacons.
            SsbActionType::MemoryNoteInserted => SsbPayload::MemoryNoteInserted(
                MemoryNoteInsertedPayload {
                    note_id: "note-test-insert".into(),
                    scope: "session".into(),
                    content_blake3: "0".repeat(64),
                    link_count: 2,
                },
            ),
            SsbActionType::MemoryNoteEvolved => SsbPayload::MemoryNoteEvolved(
                MemoryNoteEvolvedPayload {
                    note_id: "note-test-evolve".into(),
                    old_content_blake3: "1".repeat(64),
                    new_content_blake3: "2".repeat(64),
                    links_added: 1,
                    links_removed: 0,
                },
            ),
            SsbActionType::MemoryConflictDetected => SsbPayload::MemoryConflictDetected(
                MemoryConflictDetectedPayload {
                    fact_id_a: "fact-test-a".into(),
                    fact_id_b: "fact-test-b".into(),
                    conflict_kind: "duplicate".into(),
                },
            ),
            SsbActionType::CoreBlockReplaced => SsbPayload::CoreBlockReplaced(
                CoreBlockReplacedPayload {
                    user_id: "default".into(),
                    old_blake3: "3".repeat(64),
                    new_blake3: "4".repeat(64),
                    bytes: 512,
                },
            ),
            // D-3 (SCHEMA_VERSION 69): multi-verifier consensus aggregator.
            SsbActionType::VerifierConsensusPass => SsbPayload::VerifierConsensusPass(
                VerifierConsensusPassPayload {
                    candidate_id: "cand-test-pass".into(),
                    template_id: "tpl-test-1".into(),
                    verdict: "pass".into(),
                    verifier_count: 3,
                    pass_count: 2,
                    rule_id: "k_of_n".into(),
                },
            ),
            SsbActionType::VerifierConsensusFail => SsbPayload::VerifierConsensusFail(
                VerifierConsensusFailPayload {
                    candidate_id: "cand-test-fail".into(),
                    template_id: "tpl-test-1".into(),
                    failing_verifier_ids: vec!["v-intrinsic-1".into(), "v-det-1".into()],
                    reason: "fail".into(),
                },
            ),
            // D-4 (SCHEMA_VERSION 70): tripwire stack beacons.
            SsbActionType::TripwireFired => SsbPayload::TripwireFired(TripwireFiredPayload {
                event_id: "00112233445566778899aabbccddeeff".into(),
                tripwire_kind: "iqr_outlier".into(),
                source: "action".into(),
                candidate_id: "cand-test-trip".into(),
                observed_value: 42.0,
                threshold: 3.0,
                fired_at_ms: 1_700_000_000_000,
            }),
            SsbActionType::TemplateAutoReverted => {
                SsbPayload::TemplateAutoReverted(TemplateAutoRevertedPayload {
                    template_id: "tpl-test-auto-1".into(),
                    parent_template_id: Some("tpl-test-parent".into()),
                    retirement_reason: "degradation".into(),
                    fired_tripwire_id: "00112233445566778899aabbccddeeff".into(),
                })
            }
            // D-4 GAP-2 (SCHEMA_VERSION 108): concurrent-detector tripwire row.
            SsbActionType::TripwireFiredConcurrent => {
                SsbPayload::TripwireFiredConcurrent(TripwireFiredConcurrentPayload {
                    candidate_id: "cand-test-trip".into(),
                    tripwire_a: "iqr_outlier".into(),
                    tripwire_b: "perplexity".into(),
                    fired_within_ms: 0,
                    fired_at_ms: 1_700_000_000_000,
                })
            }
            // D-19 (SCHEMA_VERSION 71): PRMBench adversarial-gaming benchmark beacon.
            SsbActionType::PrmBenchRunCompleted => {
                SsbPayload::PrmBenchRunCompleted(PrmBenchRunCompletedPayload {
                    total: 20,
                    accuracy: 0.75,
                    precision: 0.80,
                    recall: 0.70,
                    gating_verdict: "pass".into(),
                    fixture_blake3: "a".repeat(64),
                    category_count: 2,
                })
            }
            // D-5 (SCHEMA_VERSION 72): BAI Thompson sampling promotion round.
            SsbActionType::BaiArmUpdated => SsbPayload::BaiArmUpdated(BaiArmUpdatedPayload {
                template_id: "11112222333344445555666677778888".into(),
                alpha: 9.0,
                beta: 2.0,
                n_pulls: 9,
                mean_estimate: 0.818,
                was_promoted: true,
            }),
            // D-5 closure (SCHEMA_VERSION 208): unified promotion-decision summary.
            SsbActionType::BaiPromotionDecision => {
                SsbPayload::BaiPromotionDecision(BaiPromotionDecisionPayload {
                    template_id: "11112222333344445555666677778888".into(),
                    decision: "promoted".into(),
                    stage: "stage_10".into(),
                    evidence_pass: 8,
                    evidence_total: 10,
                    audit_id: "aabb1122ccdd3344eeff55667788aabb".into(),
                })
            }
            // D-6 (SCHEMA_VERSION 73): held-out task store lifecycle.
            SsbActionType::HeldOutTaskAdded => {
                SsbPayload::HeldOutTaskAdded(HeldOutTaskAddedPayload {
                    task_id: "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa".into(),
                    source: "synthetic".into(),
                    content_blake3: "b".repeat(64),
                    ngram_count: 12,
                    ingested_ts_ms: 1_700_000_000_000,
                    eligibility_window_ms: 0,
                })
            }
            SsbActionType::HeldOutTaskUsed => SsbPayload::HeldOutTaskUsed(HeldOutTaskUsedPayload {
                task_id: "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa".into(),
                consumer_template_id: "11112222333344445555666677778888".into(),
                used_ts_ms: 1_700_000_005_000,
            }),
            // D-7 (SCHEMA_VERSION 74): cross-domain transfer canary beacons.
            SsbActionType::CanaryRunCompleted => {
                SsbPayload::CanaryRunCompleted(CanaryRunCompletedPayload {
                    report_id: "c".repeat(64),
                    ts_ms: 1_700_000_010_000,
                    overall_transfer_score: 0.04,
                    decision_kind: "pass".into(),
                    dominant_fail_family: String::new(),
                })
            }
            SsbActionType::CanaryFamilyEvaluated => {
                SsbPayload::CanaryFamilyEvaluated(CanaryFamilyEvaluatedPayload {
                    report_id: "c".repeat(64),
                    family: "code_fix".into(),
                    n_evaluated: 5,
                    pass_count: 4,
                    fail_count: 1,
                    transfer_score: 0.10,
                    baseline_score: 0.70,
                })
            }
            // D-9 (SCHEMA_VERSION 75): L3 frozen-surface auditor beacons.
            SsbActionType::LockedKeysAuditRun => {
                SsbPayload::LockedKeysAuditRun(LockedKeysAuditRunPayload {
                    audit_id: uuid::Uuid::nil(),
                    ts_ms: 1_700_000_020_000,
                    n_audited: 2,
                    n_violations: 1,
                    registry_blake3: "0".repeat(64),
                    registry_path: "scripts/locked_keys_registry.toml".into(),
                })
            }
            SsbActionType::LockedKeysViolationDetected => {
                SsbPayload::LockedKeysViolationDetected(
                    LockedKeysViolationDetectedPayload {
                        audit_id: uuid::Uuid::nil(),
                        ts_ms: 1_700_000_020_001,
                        attempted_edit:
                            "crates/orchestrator/src/tripwire.rs".into(),
                        attempted_lines: vec![0, u32::MAX],
                        locked_key_path:
                            "crates/orchestrator/src/tripwire.rs".into(),
                        locked_key_signature_blake3: "0".repeat(64),
                        reason: "tripwire detection per arxiv:1908.04734".into(),
                    },
                )
            }
            // E-46 (SCHEMA_VERSION 76): hierarchical decomposition tuning beacons.
            SsbActionType::TaskAdvanced => SsbPayload::TaskAdvanced(TaskAdvancedPayload {
                task_id: "task-1".into(),
                parent_id: String::new(),
                completion_pct: 50.0,
                status: "in_progress".into(),
                blocker: String::new(),
                ts_ms: 1_700_000_030_000,
            }),
            SsbActionType::AgentlessShortcutTaken => {
                SsbPayload::AgentlessShortcutTaken(AgentlessShortcutTakenPayload {
                    task_id: "task-1".into(),
                    reason: "single_file_small".into(),
                    ts_ms: 1_700_000_030_001,
                })
            }
            SsbActionType::GoalDriftFired => SsbPayload::GoalDriftFired(GoalDriftFiredPayload {
                initial_goal_blake3: "d".repeat(64),
                drift: 0.42,
                threshold: 0.3,
                step: 3,
                ts_ms: 1_700_000_030_002,
            }),
            // D-10 (SCHEMA_VERSION 77): mode-routing predicate beacon.
            SsbActionType::ClaimRouted => SsbPayload::ClaimRouted(ClaimRoutedPayload {
                claim_id: "claim-d10-test".into(),
                claim_type: "factual".into(),
                mode: "constructive".into(),
                ts_ms: 1_700_000_040_000,
                ctx_summary_blake3: "e".repeat(64),
            }),
            // D-21 (SCHEMA_VERSION 78): mode-routing coverage benchmark beacon.
            SsbActionType::ModeRoutingBenchRunCompleted => {
                SsbPayload::ModeRoutingBenchRunCompleted(
                    ModeRoutingBenchRunCompletedPayload {
                        total: 25,
                        accuracy: 0.92,
                        f1_macro: 0.88,
                        gating_verdict: "pass".into(),
                        fixture_blake3: "f".repeat(64),
                        fixture_row_count: 25,
                    },
                )
            }
            // D-11 (SCHEMA_VERSION 79): constructive-checker registry beacon.
            SsbActionType::CheckerResultEmitted => SsbPayload::CheckerResultEmitted(
                CheckerResultEmittedPayload {
                    checker_id: "regex-default".into(),
                    checker_kind: "regex_match".into(),
                    result_kind: "pass".into(),
                    claim_id: "claim-d11-test".into(),
                    ts_ms: 1_700_000_050_000,
                    reason: String::new(),
                },
            ),
            // D-13 (SCHEMA_VERSION 80): adversarial-verification beacons.
            SsbActionType::AdversaryGenerated => SsbPayload::AdversaryGenerated(
                AdversaryGeneratedPayload {
                    counter_id: "ctr-d13-test".into(),
                    claim_id: "claim-d13-test".into(),
                    severity: 0.75,
                    model_family: "anthropic".into(),
                },
            ),
            SsbActionType::ConsensusReevaluatedWithCounter => {
                SsbPayload::ConsensusReevaluatedWithCounter(
                    ConsensusReevaluatedWithCounterPayload {
                        claim_id: "claim-d13-test".into(),
                        original_verdict: "pass".into(),
                        post_counter_verdict: "fail".into(),
                    },
                )
            }
            // D-12 (SCHEMA_VERSION 81): OperationalizationCheck beacon.
            SsbActionType::BabyStepDispatched => SsbPayload::BabyStepDispatched(
                BabyStepDispatchedPayload {
                    step_id: "bs-d12-test".into(),
                    parent_task_id: "task-d12-test".into(),
                    checker_kinds_used: vec!["numeric_range".into()],
                    outcome: "pass".into(),
                    ts_ms: 1_700_000_060_000,
                },
            ),
            // D-16 (SCHEMA_VERSION 82): UniCR conformal-refusal beacons.
            SsbActionType::ConformalTauComputed => SsbPayload::ConformalTauComputed(
                ConformalTauComputedPayload {
                    tau: 0.8,
                    epsilon: 0.1,
                    corpus_size: 40,
                    source: "held_out".into(),
                    ts_ms: 1_700_000_070_000,
                    calibration_id: None,
                },
            ),
            SsbActionType::ConformalRefusalDecided => SsbPayload::ConformalRefusalDecided(
                ConformalRefusalDecidedPayload {
                    claim_id: "claim-d16-test".into(),
                    claim_score: 0.05,
                    tau: 0.8,
                    decision: "abstain".into(),
                    confidence: 0.0,
                    ts_ms: 1_700_000_071_000,
                    reason: Some("low_evidence".into()),
                    calibration_id: None,
                },
            ),
            // D-15 (SCHEMA_VERSION 83): discovery-vs-search NoveltyClass beacon.
            // FINAL TRANCHE B4: optional candidate_status field added; test
            // payload sets `Queued` to exercise the new arm. Existing on-disk
            // rows without the field deserialize as None.
            SsbActionType::NoveltyClassified => SsbPayload::NoveltyClassified(
                NoveltyClassifiedPayload {
                    template_id: "tpl-d15-test".into(),
                    class: "discovery_candidate".into(),
                    dkg_match_count: 0,
                    near_duplicate_threshold: 0.05,
                    ts_ms: 1_700_000_080_000,
                    candidate_status: Some(DiscoveryCandidateStatus::Queued),
                },
            ),
            // D-18 (SCHEMA_VERSION 84): typed-ObservableEffect predicate beacon.
            SsbActionType::ObservableEffectVerified => SsbPayload::ObservableEffectVerified(
                ObservableEffectVerifiedPayload {
                    effect_id: "eff-d18-test".into(),
                    trigger_kind: "test_command".into(),
                    observation_kind: "string_match".into(),
                    sandbox_tier: "no_sandbox".into(),
                    outcome: "pass".into(),
                    confound_flags: String::new(),
                    elapsed_us: 1234,
                    ts_ms: 1_700_000_090_000,
                },
            ),
            // D-17 (SCHEMA_VERSION 85): honest-refusal SLA monitoring beacons.
            SsbActionType::RefusalRateSnapshot => SsbPayload::RefusalRateSnapshot(
                RefusalRateSnapshotPayload {
                    refusal_rate: 0.5,
                    window_total: 100,
                    abstain_count: 50,
                    health_band: "above".into(),
                    ts_ms: 1_700_000_100_000,
                },
            ),
            SsbActionType::RefusalSlaBreach => SsbPayload::RefusalSlaBreach(
                RefusalSlaBreachPayload {
                    rate: 0.5,
                    threshold: 0.2,
                    breach_kind: "above".into(),
                    ts_ms: 1_700_000_101_000,
                },
            ),
            // D-22 (SCHEMA_VERSION 86): conformal-calibration validation beacon.
            SsbActionType::ConformalValidationCompleted => {
                SsbPayload::ConformalValidationCompleted(
                    ConformalValidationCompletedPayload {
                        tau: 0.4,
                        epsilon: 0.1,
                        empirical_refusal_rate: 0.08,
                        gating_verdict: "pass".into(),
                        fixture_blake3: "f".repeat(64),
                        total: 36,
                        ts_ms: 1_700_000_110_000,
                    },
                )
            }
            // D-20 (SCHEMA_VERSION 87): ARC-AGI-2 OOD probe gating beacon.
            SsbActionType::ArcBenchmarkCompleted => SsbPayload::ArcBenchmarkCompleted(
                ARCBenchmarkCompletedPayload {
                    total: 24,
                    honestly_evaluated_count: 11,
                    gating_verdict: "honestly_evaluated".into(),
                    fixture_blake3: "a".repeat(64),
                    dominant_mode: "operational_deferred".into(),
                    dominant_novelty_class: "discovery_candidate".into(),
                    ts_ms: 1_700_000_120_000,
                },
            ),
            SsbActionType::ToolGated => SsbPayload::ToolGated(ToolGateEventPayload {
                tool_name: "fs.read".into(),
                fsm_state: "planning".into(),
                prompt_tokens_saved: 42,
            }),
            SsbActionType::ToolEager => SsbPayload::ToolEager(ToolGateEventPayload {
                tool_name: "knowledge.search".into(),
                fsm_state: "planning".into(),
                prompt_tokens_saved: 0,
            }),
            SsbActionType::ToolLazyResolved => SsbPayload::ToolLazyResolved(ToolGateEventPayload {
                tool_name: "fs.edit".into(),
                fsm_state: "executing".into(),
                prompt_tokens_saved: 120,
            }),
            SsbActionType::ToolDenied => SsbPayload::ToolDenied(ToolGateEventPayload {
                tool_name: "git.commit".into(),
                fsm_state: "planning".into(),
                prompt_tokens_saved: 0,
            }),
            SsbActionType::QuantBackendSelected => SsbPayload::QuantBackendSelected(QuantBackendSelectedPayload {
                backend: "awq".into(),
                kv_dtype: "fp8_e5m2".into(),
                compute_capability: 86,
                marlin_enabled: true,
            }),
            SsbActionType::KvCacheDtypeSoftwareSim => SsbPayload::KvCacheDtypeSoftwareSim(KvCacheDtypeSoftwareSimPayload {
                kv_dtype: "fp8_e5m2".into(),
                throughput_cost_pct: 5.0,
                compute_capability: 86,
            }),
            SsbActionType::AwqCheckpointBuilt => SsbPayload::AwqCheckpointBuilt(AwqCheckpointBuiltPayload {
                model_path: "/models/Qwen2.5-Coder-7B".into(),
                calibration_size: 256,
                duration_s: 7200.0,
                output_path: "/models/Qwen2.5-Coder-7B-AWQ".into(),
            }),
            SsbActionType::CliMcpDispatch => SsbPayload::CliMcpDispatch(CliMcpDispatchPayload {
                tool_name: "fs.read".into(),
                port: 8200,
            }),
            SsbActionType::CliMcpServerStarted => SsbPayload::CliMcpServerStarted(CliMcpServerStartedPayload {
                port: 8200,
            }),
            SsbActionType::CliMcpServerStopped => SsbPayload::CliMcpServerStopped(CliMcpServerStoppedPayload {
                port: 8200,
            }),
            // B-19 (SCHEMA_VERSION 101): per-agent-execution DAIRA marker
            // aggregation test fixture. Two-entry map keeps the round-trip
            // bijection deterministic without inflating fixture size.
            SsbActionType::MarkerReport => {
                let mut phase_counts: std::collections::BTreeMap<String, u32> =
                    std::collections::BTreeMap::new();
                phase_counts.insert("HookPre".into(), 1);
                phase_counts.insert("HookPost".into(), 1);
                SsbPayload::MarkerReport(MarkerReportPayload {
                    agent_id: "task_runner".into(),
                    step_id: "task_test".into(),
                    phase_counts,
                    total_events: 2,
                    window_us: 0,
                })
            }
            // B-25 (SCHEMA_VERSION 102): Phase 4J Empire integration fixtures.
            SsbActionType::EmpireConnected => SsbPayload::EmpireConnected(EmpireConnectedPayload {
                brain_id: "elai_cc".into(),
                protocol_version: 1,
            }),
            SsbActionType::EmpireDisconnected => {
                SsbPayload::EmpireDisconnected(EmpireDisconnectedPayload {
                    brain_id: "elai_cc".into(),
                    reason: "heartbeat_timeout".into(),
                })
            }
            SsbActionType::EmpireEventReceived => {
                SsbPayload::EmpireEventReceived(EmpireEventReceivedPayload {
                    source: "factory".into(),
                    event_type: "deploy".into(),
                    payload: serde_json::Value::Null,
                })
            }
            SsbActionType::EmpireCapabilityInvoked => {
                SsbPayload::EmpireCapabilityInvoked(EmpireCapabilityInvokedPayload {
                    capability: "task_submit".into(),
                    caller: "factory".into(),
                    allowed: true,
                })
            }
            SsbActionType::EmpireRepairProposed => {
                SsbPayload::EmpireRepairProposed(EmpireRepairProposedPayload {
                    issue: "circular_dep".into(),
                    target_path: "workspace/factory-brain/core/bus.py".into(),
                    fix_kind: "refactor_shared_module".into(),
                })
            }
            // C-5 follow-on (SCHEMA_VERSION 103): 3 atomic variants.
            SsbActionType::RouteBlocked => SsbPayload::RouteBlocked(RouteBlockedPayload {
                reason: "daily_cap_exceeded".into(),
                spent_usd: 5.10,
                cap_usd: 5.0,
                cap_period: "daily".into(),
                baby_step_id: 1,
                ts_ms: 1_700_000_200_000,
            }),
            SsbActionType::WarnLimitReached => SsbPayload::WarnLimitReached(WarnLimitReachedPayload {
                spent_usd: 4.05,
                cap_usd: 5.0,
                warn_at_percent: 80,
                pct_of_cap: 81.0,
                baby_step_id: 1,
                ts_ms: 1_700_000_201_000,
            }),
            SsbActionType::CostEstimate => SsbPayload::CostEstimate(CostEstimatePayload {
                low_usd: 0.084,
                high_usd: 0.156,
                model: "claude-sonnet-4-6".into(),
                step_count: 3,
                heuristic_source: "ungrounded_heuristic".into(),
                ts_ms: 1_700_000_202_000,
            }),
            // C-36 (SCHEMA_VERSION 104): MuCoCo mutation operator dispatch.
            SsbActionType::MucocoMutationApplied => SsbPayload::MucocoMutationApplied(
                MucocoMutationAppliedPayload {
                    operator: "demorgan".into(),
                    language: "python".into(),
                    source_hash: 0xc36c_0036_c36c_0036,
                    applied: true,
                    skip_reason: None,
                    timestamp_ms: 1_700_000_203_000,
                },
            ),
            // D-3 closure (SCHEMA_VERSION 105): PpmUnavailable +
            // VerifierDisagreement + CrossContextDispatched.
            SsbActionType::PpmUnavailable => SsbPayload::PpmUnavailable(PpmUnavailablePayload {
                candidate_id: "cand-d3-unavail".into(),
                verifier_id: "ppm-test".into(),
                reason: "health_check_failed".into(),
            }),
            SsbActionType::VerifierDisagreement => SsbPayload::VerifierDisagreement(
                VerifierDisagreementPayload {
                    candidate_id: "cand-d3-disagree".into(),
                    template_id: "tpl-d3-disagree".into(),
                    verifier_a_kind: "ppm".into(),
                    verifier_b_kind: "deterministic".into(),
                    score_delta: 0.9,
                },
            ),
            SsbActionType::CrossContextDispatched => SsbPayload::CrossContextDispatched(
                CrossContextDispatchedPayload {
                    claim_id: "claim-d3-cc".into(),
                    context_a_hash: "a".repeat(64),
                    context_b_hash: "b".repeat(64),
                    jaccard: 0.85,
                    agreement: "agree".into(),
                },
            ),
            // C-38 (SCHEMA_VERSION 107): pagerank + dkg rebuild lifecycle.
            SsbActionType::PagerankRun => SsbPayload::PagerankRun(PagerankRunPayload {
                workspace_root_hash: 0xc380_0001_c380_0001,
                node_count: 50,
                top_node_hash: 0xc380_0002_c380_0002,
                elapsed_ms: 12,
            }),
            SsbActionType::DkgRebuildStarted => SsbPayload::DkgRebuildStarted(
                DkgRebuildStartedPayload {
                    workspace_root_hash: 0xc380_0003_c380_0003,
                    trigger_reason: "missing".into(),
                },
            ),
            SsbActionType::DkgRebuildComplete => SsbPayload::DkgRebuildComplete(
                DkgRebuildCompletePayload {
                    workspace_root_hash: 0xc380_0003_c380_0003,
                    success: true,
                    duration_ms: 1234,
                    node_count: 50,
                },
            ),
            // C-40 (SCHEMA_VERSION 109): GitHub PR adapter producer surface.
            SsbActionType::GithubPrOpened => SsbPayload::GithubPrOpened(
                GithubPrOpenedPayload {
                    pr_number: 42,
                    pr_url_hash: 0xc400_0001_c400_0001,
                    ts_ms: 1_700_000_000_000,
                },
            ),
            SsbActionType::GithubPrStatusFetched => SsbPayload::GithubPrStatusFetched(
                GithubPrStatusFetchedPayload {
                    pr_number: 42,
                    ci_status_hash: 0xc400_0002_c400_0002,
                    review_state_hash: 0xc400_0003_c400_0003,
                    ts_ms: 1_700_000_000_000,
                },
            ),
            SsbActionType::GithubPrClosed => SsbPayload::GithubPrClosed(
                GithubPrClosedPayload {
                    pr_number: 42,
                    ts_ms: 1_700_000_000_000,
                },
            ),
            SsbActionType::GithubPrMerged => SsbPayload::GithubPrMerged(
                GithubPrMergedPayload {
                    pr_number: 42,
                    merge_sha_hash: 0xc400_0004_c400_0004,
                    ts_ms: 1_700_000_000_000,
                },
            ),
            // E-41 (SCHEMA_VERSION 135): GitHub PR review surface.
            SsbActionType::PrReviewComposed => SsbPayload::PrReviewComposed(
                PrReviewComposedPayload {
                    pr_number_hash: 0xe410_0001_e410_0001,
                    comment_kind_hash: 0xe410_0002_e410_0002,
                    ts_ms: 1_700_000_000_000,
                },
            ),
            SsbActionType::PrCommentWatched => SsbPayload::PrCommentWatched(
                PrCommentWatchedPayload {
                    pr_number_hash: 0xe410_0003_e410_0003,
                    new_comments_count: 3,
                    ts_ms: 1_700_000_000_000,
                },
            ),
            SsbActionType::PrPackageVerifyResult => SsbPayload::PrPackageVerifyResult(
                PrPackageVerifyResultPayload {
                    pr_number_hash: 0xe410_0004_e410_0004,
                    slopsquat_count: 1,
                    ts_ms: 1_700_000_000_000,
                },
            ),
            SsbActionType::PrSemgrepGateResult => SsbPayload::PrSemgrepGateResult(
                PrSemgrepGateResultPayload {
                    pr_number_hash: 0xe410_0005_e410_0005,
                    findings_count: 2,
                    severity_max_hash: 0xe410_0006_e410_0006,
                    ts_ms: 1_700_000_000_000,
                },
            ),
            SsbActionType::PrAutoMergeBlocked => SsbPayload::PrAutoMergeBlocked(
                PrAutoMergeBlockedPayload {
                    pr_number_hash: 0xe410_0007_e410_0007,
                    block_reason_hash: 0xe410_0008_e410_0008,
                    ts_ms: 1_700_000_000_000,
                },
            ),
            // D-5 closure (SCHEMA_VERSION 110): Thompson-sample beacon.
            SsbActionType::ThompsonSampled => SsbPayload::ThompsonSampled(
                ThompsonSampledPayload {
                    arm_id: "11112222333344445555666677778888".into(),
                    sampled_value: 0.812,
                    alpha: 9.0,
                    beta: 2.0,
                },
            ),
            // C-41 (SCHEMA_VERSION 111): skills-driven workflow registry.
            SsbActionType::SkillTriggered => SsbPayload::SkillTriggered(
                SkillTriggeredPayload {
                    skill_name_hash: 0xc410_0001_c410_0001,
                    score: 0.875,
                    top_k_rank: 1,
                    ts_ms: 1_700_000_000_000,
                },
            ),
            SsbActionType::SkillExecuted => SsbPayload::SkillExecuted(
                SkillExecutedPayload {
                    skill_name_hash: 0xc410_0001_c410_0001,
                    exit_ok: true,
                    duration_ms: 42,
                    ts_ms: 1_700_000_000_042,
                },
            ),
            // D-6 closure (SCHEMA_VERSION 112): held-out promotion-gate +
            // refresh-event beacons.
            SsbActionType::HeldOutPromotionGate => SsbPayload::HeldOutPromotionGate(
                HeldOutPromotionGatePayload {
                    task_id: "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa".into(),
                    allow: true,
                    delta: -0.02,
                    regression_threshold: 0.05,
                    passes: 47,
                    fails: 3,
                    reason: "delta=-0.0200 pp (allowed: >= -0.0500 pp regression threshold)"
                        .into(),
                },
            ),
            SsbActionType::HeldOutRefresh => SsbPayload::HeldOutRefresh(
                HeldOutRefreshPayload {
                    source_label: "arxiv_repro".into(),
                    added_count: 3,
                    retired_count: 1,
                    ts_ms: 1_700_000_000_500,
                },
            ),
            // C-42 (SCHEMA_VERSION 113): Semantic Gateway L2 RBAC beacons.
            SsbActionType::GatewayPolicyAllowed => SsbPayload::GatewayPolicyAllowed(
                GatewayPolicyAllowedPayload {
                    role_hash: 0xc420_0001_c420_0001,
                    tool_hash: 0xc420_0002_c420_0002,
                    deny_reason_label: "allow".into(),
                    ts_ms: 1_700_000_000_000,
                },
            ),
            SsbActionType::GatewayPolicyDenied => SsbPayload::GatewayPolicyDenied(
                GatewayPolicyDeniedPayload {
                    role_hash: 0xc420_0001_c420_0001,
                    tool_hash: 0xc420_0002_c420_0002,
                    deny_reason_label: "authz".into(),
                    deny_reason_hash: 0xc420_0003_c420_0003,
                    ts_ms: 1_700_000_000_000,
                },
            ),
            SsbActionType::GatewayHitlPending => SsbPayload::GatewayHitlPending(
                GatewayHitlPendingPayload {
                    role_hash: 0xc420_0001_c420_0001,
                    tool_hash: 0xc420_0002_c420_0002,
                    challenge_hash: 0,
                    ts_ms: 1_700_000_000_000,
                },
            ),
            // D-7 closure (SCHEMA_VERSION 114): cross-domain canary obs.
            SsbActionType::CrossDomainRegression => SsbPayload::CrossDomainRegression(
                CrossDomainRegressionPayload {
                    source_domain: crate::reasoning_template::Domain::Code,
                    regressed_domain: crate::reasoning_template::Domain::Math,
                    delta_pct: 7.0,
                    threshold_pct: 5.0,
                    ts_ms: 1_700_000_000_000,
                },
            ),
            SsbActionType::CanaryFamilyPass => SsbPayload::CanaryFamilyPass(
                CanaryFamilyPassPayload {
                    family: "code_fix".into(),
                    score_pct: 82.0,
                    sample_count: 10,
                    ts_ms: 1_700_000_000_000,
                },
            ),
            SsbActionType::CanaryInsufficientData => SsbPayload::CanaryInsufficientData(
                CanaryInsufficientDataPayload {
                    family: "tool_use".into(),
                    sample_count: 3,
                    min_required: 5,
                    ts_ms: 1_700_000_000_000,
                },
            ),
            // D-8 closure (SCHEMA_VERSION 115): cryptographic archive verify.
            SsbActionType::ArchiveVerifyResult => SsbPayload::ArchiveVerifyResult(
                ArchiveVerifyResultPayload {
                    audit_id: "test-audit".into(),
                    total_entries: 0,
                    verified: true,
                    error_count: 0,
                    errors: Vec::new(),
                    ts_ms: 1_700_000_000_000,
                },
            ),
            // C-43 closure (SCHEMA_VERSION 116): Phase 4H scope-freeze.
            SsbActionType::ScopeFreezeRatified => SsbPayload::ScopeFreezeRatified(
                ScopeFreezeRatifiedPayload {
                    manifest_hash: 0,
                    ratifier_id_hash: 0,
                    item_count: 0,
                    ts_ms: 1_700_000_000_000,
                },
            ),
            SsbActionType::ScopeFreezeViolated => SsbPayload::ScopeFreezeViolated(
                ScopeFreezeViolatedPayload {
                    item_path_hash: 0,
                    frozen_since_ms: 0,
                    modified_at_ms: 0,
                    ts_ms: 1_700_000_000_000,
                },
            ),
            // C-44 (SCHEMA_VERSION 117): Composer 2 paid-backend cost accrual.
            SsbActionType::BackendCostAccrued => SsbPayload::BackendCostAccrued(
                BackendCostAccruedPayload {
                    backend_name_hash: 0,
                    role_name_hash: 0,
                    tokens_in: 0,
                    tokens_out: 0,
                    est_cost_micros: 0,
                    ts_ms: 1_700_000_000_000,
                },
            ),
            // C-45 (SCHEMA_VERSION 120): CANTANTE credit attribution.
            SsbActionType::AgentRoleDropped => SsbPayload::AgentRoleDropped(
                AgentRoleDroppedPayload {
                    role_name_hash: 0,
                    ts_ms: 1_700_000_000_000,
                },
            ),
            SsbActionType::CreditAttributionRun => SsbPayload::CreditAttributionRun(
                CreditAttributionRunPayload {
                    run_id_hash: 0,
                    dropped_role_name_hash: 0,
                    k_rollouts: 9,
                    g_partition: 3,
                    ts_ms: 1_700_000_000_000,
                },
            ),
            SsbActionType::CreditAttributionResult => SsbPayload::CreditAttributionResult(
                CreditAttributionResultPayload {
                    run_id_hash: 0,
                    per_role_scores_hash: 0,
                    total_roles: 8,
                    ts_ms: 1_700_000_000_000,
                },
            ),
            // C-46 (SCHEMA_VERSION 121): Uno-Orchestra selective delegation.
            SsbActionType::SelectiveDelegationDecision => SsbPayload::SelectiveDelegationDecision(
                SelectiveDelegationDecisionPayload {
                    task_id_hash: 0,
                    difficulty: 0.0,
                    alpha: 0.1,
                    dispatched_count: 0,
                    ts_ms: 1_700_000_000_000,
                },
            ),
            SsbActionType::SelectiveDelegationEscalated => SsbPayload::SelectiveDelegationEscalated(
                SelectiveDelegationEscalatedPayload {
                    task_id_hash: 0,
                    reason_hash: 0,
                    escalated_workers: 1,
                    ts_ms: 1_700_000_000_000,
                },
            ),
            SsbActionType::SelectiveDelegationCostSaved => SsbPayload::SelectiveDelegationCostSaved(
                SelectiveDelegationCostSavedPayload {
                    task_id_hash: 0,
                    skipped_count: 1,
                    est_savings_micros: 0,
                    ts_ms: 1_700_000_000_000,
                },
            ),
            // C-47 (SCHEMA_VERSION 122): Case-Based Calibration of Tool-Use.
            SsbActionType::ToolCallLogged => SsbPayload::ToolCallLogged(
                ToolCallLoggedPayload {
                    tool_name_hash: 0,
                    call_shape_hash: 0,
                    response_kind_hash: 0,
                    post_call_audit_ok: true,
                    ts_ms: 1_700_000_000_000,
                },
            ),
            SsbActionType::ToolCalibrationHintEmitted => SsbPayload::ToolCalibrationHintEmitted(
                ToolCalibrationHintEmittedPayload {
                    tool_name_hash: 0,
                    call_shape_hash: 0,
                    hint_text_hash: 0,
                    ts_ms: 1_700_000_000_000,
                },
            ),
            SsbActionType::ToolCalibrationDriftDetected => SsbPayload::ToolCalibrationDriftDetected(
                ToolCalibrationDriftDetectedPayload {
                    tool_name_hash: 0,
                    error_rate: 0.0,
                    total_calls: 0,
                    ts_ms: 1_700_000_000_000,
                },
            ),
            // C-49 (SCHEMA_VERSION 123): iterative-distillation experience.
            SsbActionType::PromptPolicyDistillationRun => SsbPayload::PromptPolicyDistillationRun(
                PromptPolicyDistillationRunPayload {
                    skill_name_hash: 0,
                    trace_count: 0,
                    fragments_emitted: 0,
                    ts_ms: 1_700_000_000_000,
                },
            ),
            SsbActionType::PromptFragmentSelected => SsbPayload::PromptFragmentSelected(
                PromptFragmentSelectedPayload {
                    skill_name_hash: 0,
                    fragment_hash: 0,
                    success_rate: 0.0,
                    freq: 0,
                    ts_ms: 1_700_000_000_000,
                },
            ),
            SsbActionType::PromptFragmentDiscarded => SsbPayload::PromptFragmentDiscarded(
                PromptFragmentDiscardedPayload {
                    skill_name_hash: 0,
                    fragment_hash: 0,
                    reason_hash: 0,
                    ts_ms: 1_700_000_000_000,
                },
            ),
            // D-14 (SCHEMA_VERSION 124): sycophancy probe row pair.
            SsbActionType::SycophancyProbeDispatched => SsbPayload::SycophancyProbeDispatched(
                SycophancyProbeDispatchedPayload {
                    claim_id: "test-claim".into(),
                    probe_kind: "are_you_sure".into(),
                    user_message: "test message".into(),
                },
            ),
            SsbActionType::SycophancyFlag => SsbPayload::SycophancyFlag(
                SycophancyFlagPayload {
                    claim_id: "test-claim".into(),
                    flip_detected: false,
                    delta: 0.0,
                    verdict: "convergent".into(),
                },
            ),
            // D-23 (SCHEMA_VERSION 142): sycophancy detection benchmark.
            SsbActionType::SycophancyBenchRunCompleted => {
                SsbPayload::SycophancyBenchRunCompleted(SycophancyBenchRunCompletedPayload {
                    tp: 70,
                    fp: 5,
                    tn: 20,
                    fn_: 3,
                    inconclusive: 2,
                    detection_rate: 0.9589,
                    fp_rate: 0.20,
                    verdict: "pass".into(),
                    total_fixtures: 100,
                    fixture_blake3: "d".repeat(64),
                    runner_name: "sycophancy_bench_v1".into(),
                })
            }
            // D-24 (SCHEMA_VERSION 143): cross-domain canary regression
            // benchmark.
            SsbActionType::CanaryBenchRunCompleted => {
                SsbPayload::CanaryBenchRunCompleted(CanaryBenchRunCompletedPayload {
                    true_block: 20,
                    false_block: 3,
                    true_pass: 77,
                    false_pass: 0,
                    harmful_blocked: 20,
                    helpful_blocked: 2,
                    neutral_blocked: 1,
                    total_fixtures: 100,
                    verdict: "pass".into(),
                    fixture_blake3: "e".repeat(64),
                    runner_name: "canary_bench_v1".into(),
                })
            }
            // D-25 (SCHEMA_VERSION 144): determinism + archive tamper-suite
            // benchmark.
            SsbActionType::DeterminismTamperSuiteCompleted => {
                SsbPayload::DeterminismTamperSuiteCompleted(
                    DeterminismTamperSuiteCompletedPayload {
                        total_scenarios: 10,
                        passed: 10,
                        failed: 0,
                        max_wall_ms: 250,
                        verdict: "pass".into(),
                        scenarios_blake3: "f".repeat(64),
                    },
                )
            }
            // D-28 (SCHEMA_VERSION 145): adversarial example farming run.
            SsbActionType::AdversarialFarmRunCompleted => {
                SsbPayload::AdversarialFarmRunCompleted(
                    AdversarialFarmRunCompletedPayload {
                        samples_farmed: 1000,
                        rounds: 2,
                        prmbench_delta_pp: 1.5,
                        verdict: "pass".into(),
                        paper_floor_pp: 1.0,
                        corpus_blake3: "a".repeat(64),
                    },
                )
            }
            // D-29 (SCHEMA_VERSION 146): hierarchical BAI round completion.
            SsbActionType::HierarchicalBaiRoundCompleted => {
                SsbPayload::HierarchicalBaiRoundCompleted(
                    HierarchicalBaiRoundCompletedPayload {
                        round: 1,
                        family_id: 0,
                        mu_posterior_mean: 0.5,
                        mu_posterior_variance: 0.01,
                        kappa_posterior_mean: 4.0,
                        kappa_posterior_variance: 0.5,
                        arm_posteriors: Vec::new(),
                        selected_arm_index: u32::MAX,
                        selected_template_id: "0".repeat(32),
                        selection_basis: "flat_ts".into(),
                        gibbs_iterations: 80,
                        canary_gate_passed: true,
                    },
                )
            }
            // D-30 (SCHEMA_VERSION 147): calibration bootstrap warm-start.
            // RUNPOD-COMPLETE 2026-05-29: SECONDARY block-bootstrap CI
            // activated with B=1000 (Politis-Romano 1994 JASA §3 default).
            // block_length_p = 1/sqrt(50) for the 50-row warm-start corpus
            // size (paper §3 geometric block parameter). CI bounds are
            // canonical lookup-table values from the reference D-30
            // calibration sweep; live re-derivation runs in the
            // `bootstrap_resample` integration test.
            SsbActionType::CalibrationBootstrapCompleted => {
                SsbPayload::CalibrationBootstrapCompleted(
                    CalibrationBootstrapCompletedPayload {
                        corpus_slug: "truthfulqa".into(),
                        corpus_size: 50,
                        corpus_blake3: "a".repeat(64),
                        tau_warm_start: 0.8,
                        alpha: 0.1,
                        tau_floor_active: false,
                        bootstrap_replicates: Some(crate::calibration_bootstrap::DEFAULT_BOOTSTRAP_REPLICATES),
                        bootstrap_ci_low: Some(0.76),
                        bootstrap_ci_high: Some(0.84),
                        block_length_p: Some(crate::calibration_bootstrap::default_block_p(50)),
                        ts_ms: 1_700_000_147_000,
                        calibration_id: None,
                    },
                )
            }
            // C-50 (SCHEMA_VERSION 125): Contract-Coding scaffolding pair.
            SsbActionType::ContractTestScaffolded => SsbPayload::ContractTestScaffolded(
                ContractTestScaffoldedPayload {
                    fn_name_hash: 0,
                    contract_kind_hash: 0,
                    ts_ms: 1_700_000_000_000,
                },
            ),
            SsbActionType::ContractViolationDetected => SsbPayload::ContractViolationDetected(
                ContractViolationDetectedPayload {
                    fn_name_hash: 0,
                    violation_kind_hash: 0,
                    severity: 0,
                    ts_ms: 1_700_000_000_000,
                },
            ),
            // C-52 (SCHEMA_VERSION 126): Dafny Formal Verification Spike.
            SsbActionType::DafnyVerifyCompleted => SsbPayload::DafnyVerifyCompleted(
                DafnyVerifyCompletedPayload {
                    tier_hash: 0,
                    dafny_pass: false,
                    udebug_pass: false,
                    heal_iter: 0,
                    ts_ms: 1_700_000_000_000,
                },
            ),
            // D-16 closure (SCHEMA_VERSION 127): durable tau persistence.
            SsbActionType::CalibrationCorpusUpdated => SsbPayload::CalibrationCorpusUpdated(
                CalibrationCorpusUpdatedPayload {
                    corpus_size: 40,
                    new_tau: 0.8,
                    alpha: 0.1,
                    source: "synthetic".into(),
                    ts_ms: 1_700_000_127_000,
                },
            ),
            // E-12 Tier-B (SCHEMA_VERSION 128): production fullstack codegen.
            SsbActionType::FrameworkProfileSelected => SsbPayload::FrameworkProfileSelected(
                FrameworkProfileSelectedPayload {
                    profile_name_hash: 0,
                    version_pin_hash: 0,
                    language_hash: 0,
                    ts_ms: 1_700_000_128_000,
                },
            ),
            SsbActionType::LegacyConfigStripped => SsbPayload::LegacyConfigStripped(
                LegacyConfigStrippedPayload {
                    config_path_hash: 0,
                    lint_engine_hash: 0,
                    ts_ms: 1_700_000_128_000,
                },
            ),
            // D-18 closure (SCHEMA_VERSION 129): typed-ObservableEffect 4-variant pack.
            SsbActionType::ObservableEffectDispatched => SsbPayload::ObservableEffectDispatched(
                ObservableEffectDispatchedPayload {
                    effect_id: "eff-d18-dispatch".into(),
                    trigger: "test_command".into(),
                    context_kind: "sandbox".into(),
                    ts_ms: 1_700_000_129_001,
                },
            ),
            SsbActionType::GoodhartRiskFlag => SsbPayload::GoodhartRiskFlag(
                GoodhartRiskFlagPayload {
                    effect_id: "eff-d18-goodhart".into(),
                    risk_kind: "constant_component".into(),
                    evidence: "component variation 0.0001 < 0.001".into(),
                    ts_ms: 1_700_000_129_002,
                },
            ),
            SsbActionType::SandboxObservation => SsbPayload::SandboxObservation(
                SandboxObservationPayload {
                    effect_id: "eff-d18-sandbox".into(),
                    tier: "container_isolated".into(),
                    outcome: "confirmed".into(),
                    evidence_count: 3,
                    ts_ms: 1_700_000_129_003,
                },
            ),
            SsbActionType::ProductionObservationProposal => {
                SsbPayload::ProductionObservationProposal(
                    ProductionObservationProposalPayload {
                        effect_id: "eff-d18-prod".into(),
                        instrument_file: None,
                        requires_human_review: true,
                        ts_ms: 1_700_000_129_004,
                    },
                )
            }
            // E-13 (SCHEMA_VERSION 130): TDD verifier-agent beacon family.
            SsbActionType::TddCycleStarted => SsbPayload::TddCycleStarted(
                TddCycleStartedPayload {
                    task_id_hash: 0x1111_2222_3333_4444,
                    ts_ms: 1_700_000_130_001,
                },
            ),
            SsbActionType::TddTestFailed => SsbPayload::TddTestFailed(
                TddTestFailedPayload {
                    task_id_hash: 0x1111_2222_3333_4444,
                    test_name_hash: 0x5555_6666_7777_8888,
                    error_kind_hash: 0x9999_AAAA_BBBB_CCCC,
                    ts_ms: 1_700_000_130_002,
                },
            ),
            SsbActionType::TddTestPassed => SsbPayload::TddTestPassed(
                TddTestPassedPayload {
                    task_id_hash: 0x1111_2222_3333_4444,
                    test_name_hash: 0x5555_6666_7777_8888,
                    mutation_score: Some(0.75),
                    ts_ms: 1_700_000_130_003,
                },
            ),
            // E-14 (SCHEMA_VERSION 131): browser-e2e agent beacon family.
            SsbActionType::BrowserE2eRunStarted => SsbPayload::BrowserE2eRunStarted(
                BrowserE2eRunStartedPayload {
                    journey_name_hash: 0xAAAA_BBBB_CCCC_DDD0,
                    url_hash: 0xAAAA_BBBB_CCCC_DDD1,
                    ts_ms: 1_700_000_131_001,
                },
            ),
            SsbActionType::BrowserE2eAssertionFailed => SsbPayload::BrowserE2eAssertionFailed(
                BrowserE2eAssertionFailedPayload {
                    journey_name_hash: 0xAAAA_BBBB_CCCC_DDD0,
                    assertion_hash: 0xAAAA_BBBB_CCCC_DDD2,
                    error_hash: 0xAAAA_BBBB_CCCC_DDD3,
                    ts_ms: 1_700_000_131_002,
                },
            ),
            SsbActionType::BrowserE2eAssertionPassed => SsbPayload::BrowserE2eAssertionPassed(
                BrowserE2eAssertionPassedPayload {
                    journey_name_hash: 0xAAAA_BBBB_CCCC_DDD0,
                    assertion_hash: 0xAAAA_BBBB_CCCC_DDD4,
                    ts_ms: 1_700_000_131_003,
                },
            ),
            SsbActionType::BrowserE2eSnapshotCaptured => SsbPayload::BrowserE2eSnapshotCaptured(
                BrowserE2eSnapshotCapturedPayload {
                    journey_name_hash: 0xAAAA_BBBB_CCCC_DDD0,
                    snapshot_kind_hash: 0xAAAA_BBBB_CCCC_DDD5,
                    ts_ms: 1_700_000_131_004,
                },
            ),
            // E-18 (SCHEMA_VERSION 132): tool-use reliability gate beacons.
            SsbActionType::ToolReliabilityGatePassed => SsbPayload::ToolReliabilityGatePassed(
                ToolReliabilityGatePassedPayload {
                    tool_name_hash: 0xE018_0001_0000_0000,
                    call_shape_hash: 0xE018_0001_0000_0001,
                    ts_ms: 1_700_000_132_001,
                },
            ),
            SsbActionType::ToolReliabilityGateRejected => SsbPayload::ToolReliabilityGateRejected(
                ToolReliabilityGateRejectedPayload {
                    tool_name_hash: 0xE018_0002_0000_0000,
                    reject_reason_hash: 0xE018_0002_0000_0001,
                    ts_ms: 1_700_000_132_002,
                },
            ),
            SsbActionType::ToolCallSpilled => SsbPayload::ToolCallSpilled(
                ToolCallSpilledPayload {
                    tool_name_hash: 0xE018_0003_0000_0000,
                    spill_path_hash: 0xE018_0003_0000_0001,
                    payload_bytes: 16_384,
                    ts_ms: 1_700_000_132_003,
                },
            ),
            SsbActionType::ToolCallRetryEscalated => SsbPayload::ToolCallRetryEscalated(
                ToolCallRetryEscalatedPayload {
                    tool_name_hash: 0xE018_0004_0000_0000,
                    attempt: 1,
                    escalate_to_role_hash: 0xE018_0004_0000_0001,
                    ts_ms: 1_700_000_132_004,
                },
            ),
            // E-37 (SCHEMA_VERSION 133): prompt-injection defense beacons.
            SsbActionType::InjectionAttemptDetected => SsbPayload::InjectionAttemptDetected(
                InjectionAttemptDetectedPayload {
                    source_hash: 0xE037_0001_0000_0000,
                    classifier_verdict_hash: 0xE037_0001_0000_0001,
                    ts_ms: 1_700_000_133_001,
                },
            ),
            SsbActionType::InjectionAttemptBlocked => SsbPayload::InjectionAttemptBlocked(
                InjectionAttemptBlockedPayload {
                    source_hash: 0xE037_0002_0000_0000,
                    defense_layer_hash: 0xE037_0002_0000_0001,
                    ts_ms: 1_700_000_133_002,
                },
            ),
            SsbActionType::SpotlightWrapped => SsbPayload::SpotlightWrapped(
                SpotlightWrappedPayload {
                    input_hash: 0xE037_0003_0000_0000,
                    marker_hash: 0xE037_0003_0000_0001,
                    ts_ms: 1_700_000_133_003,
                },
            ),
            SsbActionType::SpotlightBreached => SsbPayload::SpotlightBreached(
                SpotlightBreachedPayload {
                    input_hash: 0xE037_0004_0000_0000,
                    marker_hash: 0xE037_0004_0000_0001,
                    ts_ms: 1_700_000_133_004,
                },
            ),
            SsbActionType::ToolDescriptionPinMismatch => SsbPayload::ToolDescriptionPinMismatch(
                ToolDescriptionPinMismatchPayload {
                    tool_name_hash: 0xE037_0005_0000_0000,
                    expected_hash: 0xE037_0005_0000_0001,
                    observed_hash: 0xE037_0005_0000_0002,
                    ts_ms: 1_700_000_133_005,
                },
            ),
            SsbActionType::CapabilityGateRejected => SsbPayload::CapabilityGateRejected(
                CapabilityGateRejectedPayload {
                    tool_name_hash: 0xE037_0006_0000_0000,
                    origin_trust_hash: 0xE037_0006_0000_0001,
                    ts_ms: 1_700_000_133_006,
                },
            ),
            SsbActionType::ProvenanceTagged => SsbPayload::ProvenanceTagged(
                ProvenanceTaggedPayload {
                    text_source_hash: 0xE037_0007_0000_0000,
                    trust_level_hash: 0xE037_0007_0000_0001,
                    ts_ms: 1_700_000_133_007,
                },
            ),
            SsbActionType::PromptArmorVerdict => SsbPayload::PromptArmorVerdict(
                PromptArmorVerdictPayload {
                    verdict_hash: 0xE037_0008_0000_0000,
                    reason_hash: 0xE037_0008_0000_0001,
                    ts_ms: 1_700_000_133_008,
                },
            ),
            SsbActionType::InjectionFixtureReplayed => SsbPayload::InjectionFixtureReplayed(
                InjectionFixtureReplayedPayload {
                    fixture_name_hash: 0xE037_0009_0000_0000,
                    defenses_passed: 3,
                    ts_ms: 1_700_000_133_009,
                },
            ),
            SsbActionType::OutputClassifierFlagged => SsbPayload::OutputClassifierFlagged(
                OutputClassifierFlaggedPayload {
                    tool_name: "Bash".to_string(),
                    stripped_count: 1,
                    stripped_hosts: vec!["evil.example".to_string()],
                    patterns: Vec::new(),
                },
            ),
            // E-40 (SCHEMA_VERSION 134): orchestrator-rooted symbol-graph beacons.
            SsbActionType::SymbolGraphQueried => SsbPayload::SymbolGraphQueried(
                SymbolGraphQueriedPayload {
                    query_kind_hash: 0xE040_0001_0000_0000,
                    symbol_hash: 0xE040_0001_0000_0001,
                    result_count: 7,
                    ts_ms: 1_700_000_134_001,
                },
            ),
            SsbActionType::SymbolGraphCacheLoaded => SsbPayload::SymbolGraphCacheLoaded(
                SymbolGraphCacheLoadedPayload {
                    graph_path_hash: 0xE040_0002_0000_0000,
                    node_count: 12_345,
                    ts_ms: 1_700_000_134_002,
                },
            ),
            SsbActionType::LspOracleConsulted => SsbPayload::LspOracleConsulted(
                LspOracleConsultedPayload {
                    language_hash: 0xE040_0003_0000_0000,
                    symbol_hash: 0xE040_0003_0000_0001,
                    oracle_kind_hash: 0xE040_0003_0000_0002,
                    ts_ms: 1_700_000_134_003,
                },
            ),
            // E-44 (SCHEMA_VERSION 136): durability layer beacons.
            SsbActionType::CheckpointWritten => SsbPayload::CheckpointWritten(
                CheckpointWrittenPayload {
                    session_id_hash: 0xE044_0001_0000_0000,
                    schema_version: 136,
                    file_count: 5,
                    ts_ms: 1_700_000_136_001,
                },
            ),
            SsbActionType::CheckpointResumed => SsbPayload::CheckpointResumed(
                CheckpointResumedPayload {
                    session_id_hash: 0xE044_0002_0000_0000,
                    schema_version: 136,
                    ts_ms: 1_700_000_136_002,
                },
            ),
            SsbActionType::CheckpointMigrated => SsbPayload::CheckpointMigrated(
                CheckpointMigratedPayload {
                    from_version: 40,
                    to_version: 136,
                    transform_count: 96,
                    ts_ms: 1_700_000_136_003,
                },
            ),
            SsbActionType::IdempotentDispatchSkipped => SsbPayload::IdempotentDispatchSkipped(
                IdempotentDispatchSkippedPayload {
                    call_shape_hash: 0xE044_0004_0000_0000,
                    cached_result_hash: 0xE044_0004_0000_0001,
                    ts_ms: 1_700_000_136_004,
                },
            ),
            SsbActionType::DurabilityLockBlocked => SsbPayload::DurabilityLockBlocked(
                DurabilityLockBlockedPayload {
                    lock_path_hash: 0xE044_0005_0000_0000,
                    holder_hash: 0xE044_0005_0000_0001,
                    ts_ms: 1_700_000_136_005,
                },
            ),
            // E-45 (SCHEMA_VERSION 137): cross-language drift detector beacons.
            SsbActionType::CrossLangDriftDetected => SsbPayload::CrossLangDriftDetected(
                CrossLangDriftDetectedPayload {
                    pair_kind_hash: 0xE045_0001_0000_0000,
                    drift_count: 2,
                    severity_max: 2,
                    ts_ms: 1_700_000_137_001,
                },
            ),
            SsbActionType::CrossLangPairScanned => SsbPayload::CrossLangPairScanned(
                CrossLangPairScannedPayload {
                    repo_root_hash: 0xE045_0002_0000_0000,
                    pair_count: 6,
                    ts_ms: 1_700_000_137_002,
                },
            ),
            SsbActionType::SchemaVersionMismatch => SsbPayload::SchemaVersionMismatch(
                SchemaVersionMismatchPayload {
                    rust_schema: 137,
                    python_schema: 136,
                    ts_ms: 1_700_000_137_003,
                },
            ),
            // E-49 (SCHEMA_VERSION 138): coding-agent metric suite beacons.
            SsbActionType::CodingAgentMetricsCaptured => SsbPayload::CodingAgentMetricsCaptured(
                CodingAgentMetricsCapturedPayload {
                    session_id_hash: 0xE049_0001_0000_0000,
                    pass_at_1_x10000: 7500,
                    refine_rate_x10000: 6000,
                    cost_per_task_cents: Some(42),
                    ts_ms: 1_700_000_138_001,
                },
            ),
            SsbActionType::SweBenchProRunStarted => SsbPayload::SweBenchProRunStarted(
                SweBenchProRunStartedPayload {
                    task_pool_version_hash: 0xE049_0002_0000_0000,
                    model_id_hash: 0xE049_0002_0000_0001,
                    max_tasks: Some(50),
                    ts_ms: 1_700_000_138_002,
                },
            ),
            SsbActionType::SweBenchProRunCompleted => SsbPayload::SweBenchProRunCompleted(
                SweBenchProRunCompletedPayload {
                    task_pool_version_hash: 0xE049_0003_0000_0000,
                    model_id_hash: 0xE049_0003_0000_0001,
                    tasks_attempted: 50,
                    tasks_solved: 33,
                    pass_at_1_x10000: 6600,
                    ts_ms: 1_700_000_138_003,
                },
            ),
            SsbActionType::RefineBenchScoreEmitted => SsbPayload::RefineBenchScoreEmitted(
                RefineBenchScoreEmittedPayload {
                    suite_version_hash: 0xE049_0004_0000_0000,
                    model_id_hash: 0xE049_0004_0000_0001,
                    refine_rate_x10000: 5500,
                    ts_ms: 1_700_000_138_004,
                },
            ),
            SsbActionType::MetricsAggregationRun => SsbPayload::MetricsAggregationRun(
                MetricsAggregationRunPayload {
                    session_count: 3,
                    pass_at_1_avg_x10000: 7000,
                    refine_rate_avg_x10000: 5800,
                    ts_ms: 1_700_000_138_005,
                },
            ),
            // E-52 Tier-B (SCHEMA_VERSION 139): LSP-as-verifier beacons.
            SsbActionType::DiagnosticProbeDispatched => SsbPayload::DiagnosticProbeDispatched(
                DiagnosticProbeDispatchedPayload {
                    probe_kind_hash: 0xE052_0001_0000_0000,
                    language_hash: 0xE052_0001_0000_0001,
                    file_count: 3,
                    ts_ms: 1_700_000_139_001,
                },
            ),
            SsbActionType::DiagnosticProbeCompleted => SsbPayload::DiagnosticProbeCompleted(
                DiagnosticProbeCompletedPayload {
                    probe_kind_hash: 0xE052_0002_0000_0000,
                    run_status_hash: 0xE052_0002_0000_0001,
                    error_count: 0,
                    warning_count: 2,
                    duration_ms: 420,
                    ts_ms: 1_700_000_139_002,
                },
            ),
            SsbActionType::DiagnosticProbeBlock => SsbPayload::DiagnosticProbeBlock(
                DiagnosticProbeBlockPayload {
                    probe_kind_hash: 0xE052_0003_0000_0000,
                    error_count: 5,
                    first_path_hash: 0xE052_0003_0000_0001,
                    ts_ms: 1_700_000_139_003,
                },
            ),
            SsbActionType::ScipRefreshCompleted => SsbPayload::ScipRefreshCompleted(
                ScipRefreshCompletedPayload {
                    shard_count: 7,
                    crate_count: 4,
                    duration_ms: 12_500,
                    ts_ms: 1_700_000_139_004,
                },
            ),
            // E-53 (SCHEMA_VERSION 140): formal-verify pipeline boundary beacon.
            SsbActionType::FormalVerifierCompleted => SsbPayload::FormalVerifierCompleted(
                FormalVerifierCompletedPayload {
                    verifier_kind_hash: 0xE053_0001_0000_0000,
                    outcome_hash: 0xE053_0001_0000_0001,
                    csq_class_hash: 0xE053_0001_0000_0002,
                    contract_emitted: true,
                    duration_ms: 32,
                    ts_ms: 1_700_000_140_001,
                },
            ),
            // E-35 (SCHEMA_VERSION 141): structured-output substrate beacons.
            SsbActionType::XgrammarMaskApplied => SsbPayload::XgrammarMaskApplied(
                XgrammarMaskAppliedPayload {
                    schema_id_hash: 0xE035_0001_0000_0000,
                    schema_hash_hash: 0xE035_0001_0000_0001,
                    mask_applied_count: 12,
                    generation_ms: 311,
                    cache_hit: true,
                    ts_ms: 1_700_000_141_001,
                },
            ),
            SsbActionType::InstructorModeActivated => SsbPayload::InstructorModeActivated(
                InstructorModeActivatedPayload {
                    role_hash: 0xE035_0002_0000_0000,
                    block_hash: 0xE035_0002_0000_0001,
                    ts_ms: 1_700_000_141_002,
                },
            ),
            SsbActionType::GrammarModeActivated => SsbPayload::GrammarModeActivated(
                GrammarModeActivatedPayload {
                    role_hash: 0xE035_0003_0000_0000,
                    block_hash: 0xE035_0003_0000_0001,
                    ts_ms: 1_700_000_141_003,
                },
            ),
            SsbActionType::CodeactActionExecuted => SsbPayload::CodeactActionExecuted(
                CodeactActionExecutedPayload {
                    action_hash: 0xE035_0004_0000_0000,
                    line_count: 18,
                    outcome_hash: 0xE035_0004_0000_0001,
                    duration_ms: 42,
                    ts_ms: 1_700_000_141_004,
                },
            ),
            SsbActionType::SearchReplaceTier1Hit => SsbPayload::SearchReplaceTier1Hit(
                SearchReplaceTier1HitPayload {
                    file_path_hash: 0xE035_0005_0000_0000,
                    edit_count: 3,
                    ts_ms: 1_700_000_141_005,
                },
            ),
            SsbActionType::SearchReplaceCollisionFallback => {
                SsbPayload::SearchReplaceCollisionFallback(
                    SearchReplaceCollisionFallbackPayload {
                        file_path_hash: 0xE035_0006_0000_0000,
                        collision_token_hash: 0xE035_0006_0000_0001,
                        ts_ms: 1_700_000_141_006,
                    },
                )
            }
            SsbActionType::CloudStrictSchemaApplied => SsbPayload::CloudStrictSchemaApplied(
                CloudStrictSchemaAppliedPayload {
                    provider_hash: 0xE035_0007_0000_0000,
                    schema_id_hash: 0xE035_0007_0000_0001,
                    ts_ms: 1_700_000_141_007,
                },
            ),
            SsbActionType::GrammarCacheLoaded => SsbPayload::GrammarCacheLoaded(
                GrammarCacheLoadedPayload {
                    compiled_count: 64,
                    cache_hits: 48,
                    cache_misses: 16,
                    total_compile_ms: 4_200,
                    ts_ms: 1_700_000_141_008,
                },
            ),
            SsbActionType::StructuredEngineRejected => SsbPayload::StructuredEngineRejected(
                StructuredEngineRejectedPayload {
                    engine_hash: 0xE035_0009_0000_0000,
                    reason_hash: 0xE035_0009_0000_0001,
                    ts_ms: 1_700_000_141_009,
                },
            ),
            SsbActionType::LegacyJsonObjectRefused => SsbPayload::LegacyJsonObjectRefused(
                LegacyJsonObjectRefusedPayload {
                    provider_hash: 0xE035_000A_0000_0000,
                    ts_ms: 1_700_000_141_010,
                },
            ),
            // E-11 (SCHEMA_VERSION 169): SaaS prior-art catalog +
            // recommender substrate. Explicit arm BEFORE the wildcard
            // so the write-time `validate_payload_matches_action_type`
            // check sees a matching payload type.
            SsbActionType::SaasPriorArtEvent => SsbPayload::SaasPriorArtEvent(
                SaasPriorArtEventPayload {
                    kind: PriorArtEventKind::ArchetypeRecommended,
                    session_id: Some("s".into()),
                    archetype: "saas_dashboard".into(),
                    confidence: "high".into(),
                    details_json: serde_json::json!({
                        "project_class": "saas_dashboard",
                        "clarifying_question_count": 0,
                        "prior_art_pointer_count": 4,
                        "caller_site": "test_payload"
                    }),
                    ts_ms: 1_700_000_169_001,
                },
            ),
            // E-22 (SCHEMA_VERSION 170): may-2026 github trending sweep
            // substrate. Explicit arm BEFORE the wildcard so the
            // write-time `validate_payload_matches_action_type` check
            // sees a matching payload type.
            SsbActionType::GithubTrendingEvent => SsbPayload::GithubTrendingEvent(
                GithubTrendingEventPayload {
                    kind: TrendingEventKind::SkillExportNormalized,
                    session_id: Some("s".into()),
                    skill_name: "test-skill".into(),
                    reason_label: String::new(),
                    fallback_backend: String::new(),
                    pre_bytes: 0,
                    post_bytes: 0,
                    tool_count: 0,
                    drift_count: 0,
                    details_json: serde_json::json!({
                        "dist_path": "dist/claude/skills/test-skill/SKILL.md",
                        "caller_site": "test_payload"
                    }),
                    ts_ms: 1_700_000_170_001,
                },
            ),
            // E-23 (SCHEMA_VERSION 171): 2026 developer-discourse
            // positioning + UX-guard substrate. Explicit arm BEFORE
            // the wildcard.
            SsbActionType::DeveloperDiscourseEvent => SsbPayload::DeveloperDiscourseEvent(
                DeveloperDiscourseEventPayload {
                    kind: DiscourseEventKind::TrustTelemetryAggregated,
                    session_id: Some("s".into()),
                    tasks_per_hour: 1.5,
                    findings_per_task: 0.4,
                    revert_rate: 0.05,
                    ungrounded_count: 0,
                    prev_badge: String::new(),
                    next_badge: String::new(),
                    token_budget_pct: 0.0,
                    threshold_label: String::new(),
                    details_json: serde_json::json!({
                        "stagnation_dist": { "spinning": 0u64 },
                        "caller_site": "test_payload"
                    }),
                    ts_ms: 1_700_000_171_001,
                },
            ),
            // E-25 (SCHEMA_VERSION 172): claude-code-architecture catalog
            // substrate. Explicit arm BEFORE the wildcard so the
            // write-time `validate_payload_matches_action_type` check
            // sees a matching payload type.
            SsbActionType::ClaudeCodeArchitectureEvent => SsbPayload::ClaudeCodeArchitectureEvent(
                ClaudeCodeArchitectureEventPayload {
                    kind: ClaudeCodeArchitectureEventKind::CompactionLadderTierApplied,
                    primitive: ClaudeCodeArchPrimitive::CompactionLadder,
                    session_id: Some("s".into()),
                    details_json: serde_json::json!({
                        "tier_id": "snip",
                        "freed_tokens": 1234u64,
                        "wall_ms": 7,
                        "caller_site": "test_payload"
                    }),
                    ts_ms: 1_700_000_172_001,
                },
            ),
            // E-29 Tier-B (SCHEMA_VERSION 173): agentic-observability event
            // substrate. SINGLE-VARIANT Pattern-2 (25th application);
            // AgenticObservabilityEventKind 4-arm discriminator covers
            // mast_failure_classified / replay_resume / signal_match /
            // eval_dataset_member. Explicit arm BEFORE the wildcard so
            // the write-time validate_payload_matches_action_type check
            // sees a matching payload type.
            SsbActionType::AgenticObservabilityEvent => SsbPayload::AgenticObservabilityEvent(
                AgenticObservabilityEventPayload {
                    kind: AgenticObservabilityEventKind::MastFailureClassified,
                    session_id: Some("s-e29-test".into()),
                    mast_mode_label: MastMode::Fm13StepRepetition.as_sql_str().to_string(),
                    evidence_row_ids: vec![1, 2, 3],
                    confidence_bp: 10_000,
                    replay_session_id: String::new(),
                    replay_outcome: String::new(),
                    refusal_reason: String::new(),
                    checkpoint_id: 0,
                    signal_label: String::new(),
                    matched_row_ids: Vec::new(),
                    classifier_model: String::new(),
                    dataset_name: String::new(),
                    trace_ids: Vec::new(),
                    dataset_label: String::new(),
                    scoring_rubric: String::new(),
                    details_json: serde_json::json!({
                        "caller_site": "test_payload"
                    }),
                    ts_ms: 1_700_000_173_001,
                    // OTel cols left None on the test_payload helper:
                    // it only needs a valid msgpack payload to pass
                    // write-time validation. Real producers populate via
                    // `with_otel_span(...)`.
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
                },
            ),
            // E-31 (SCHEMA_VERSION 174): coding-model-leaderboard scraper
            // substrate. Explicit arm BEFORE the wildcard so the
            // write-time validate_payload_matches_action_type check sees
            // a matching payload type.
            SsbActionType::CodingModelLeaderboardEvent => SsbPayload::CodingModelLeaderboardEvent(
                CodingModelLeaderboardEventPayload {
                    kind: LeaderboardEventKind::ScrapeCompleted,
                    session_id: Some("s".into()),
                    suite_name: "bigcodebench".into(),
                    suite_version: "hard-set".into(),
                    leaderboard_url: "https://bigcode-bench.github.io".into(),
                    scrape_run_id: "run-test-0".into(),
                    model_name: "claude-sonnet-4-6".into(),
                    score_pct_bp: 5860,
                    sample_size: 148,
                    entry_hash: "0123456789abcdef".into(),
                    pricing_label: "sonnet".into(),
                    reason_label: String::new(),
                    http_status: 0,
                    details_json: serde_json::json!({
                        "caller_site": "test_payload"
                    }),
                    ts_ms: 1_700_000_174_001,
                },
            ),
            // E-38 (SCHEMA_VERSION 175): 2026 AI roadmap signals + EU AI
            // Act compliance pre-gate substrate. Explicit arm BEFORE the
            // wildcard so the write-time validate_payload_matches_action_type
            // check sees a matching payload type.
            SsbActionType::AiRoadmapSignalEvent => SsbPayload::AiRoadmapSignalEvent(
                AiRoadmapSignalEventPayload {
                    kind: RoadmapSignalKind::ComplianceVerified,
                    plan_session_id: "s-e38-test".into(),
                    assertion_label: String::new(),
                    hint: String::new(),
                    assertions_checked: COMPLIANCE_BIT_CLOCK_PROVIDER
                        | COMPLIANCE_BIT_AUDIT_TRAIL_LIVE,
                    mcp_transport_current: String::new(),
                    mcp_transport_target: String::new(),
                    backend_name: String::new(),
                    declared_context_tokens: 0,
                    mrcr_score_label: String::new(),
                    posture_doc_path: String::new(),
                    posture_doc_mtime_ms: 0,
                    age_days: 0,
                    details_json: serde_json::json!({
                        "caller_site": "test_payload"
                    }),
                    ts_ms: 1_700_000_175_001,
                },
            ),
            // E-39 (SCHEMA_VERSION 176): cost + token economics substrate.
            // Explicit arm BEFORE the wildcard so write-time
            // validate_payload_matches_action_type sees a matching payload
            // type for `cost_token_event`.
            SsbActionType::CostTokenEvent => SsbPayload::CostTokenEvent(
                CostTokenEventPayload {
                    kind: CostEventKind::CostRouterDecision,
                    session_id: Some("s-e39-test".into()),
                    task_id: Some("task-e39-test".into()),
                    details_json: serde_json::json!({
                        "caller_site": "test_payload",
                        "chosen_backend": "sonnet",
                        "difficulty": 0.42
                    }),
                    ts_ms: 1_700_000_176_001,
                },
            ),
            // E-50 (SCHEMA_VERSION 177): SaaS architecture pattern catalog
            // substrate. Explicit arm BEFORE the wildcard so write-time
            // validate_payload_matches_action_type sees a matching payload
            // type for `saas_arch_pattern_event`.
            SsbActionType::SaasArchPatternEvent => SsbPayload::SaasArchPatternEvent(
                SaasArchPatternEventPayload {
                    kind: SaasArchPatternEventKind::ProductionReadinessGatePassed,
                    session_id: Some("s-e50-test".into()),
                    archetype: "saas_dashboard".into(),
                    details_json: serde_json::json!({
                        "caller_site": "test_payload",
                        "applicable_checks": ["admin_route_auth", "rate_limit_middleware",
                                              "sentry_or_otel_declared", "secret_scan_clean",
                                              "rls_lint_clean", "webhook_signature_verified"],
                        "skipped_checks": []
                    }),
                    ts_ms: 1_700_000_177_001,
                },
            ),
            // E-4 (SCHEMA_VERSION 178): chat UX critique substrate.
            // Explicit arm BEFORE the wildcard so write-time
            // validate_payload_matches_action_type sees a matching payload
            // type for `chat_ux_critique_event`.
            SsbActionType::ChatUxCritiqueEvent => SsbPayload::ChatUxCritiqueEvent(
                ChatUxCritiqueEventPayload {
                    kind: ChatUxCritiqueKind::SycophancyPushbackDetected,
                    session_id: Some("s-e4-test".into()),
                    turn_id: Some("turn-e4-test".into()),
                    details_json: serde_json::json!({
                        "caller_site": "test_payload",
                        "evidence_ref": "file:crates/reasoning/src/chat_reflect.rs:379",
                        "original_stance": "agree",
                        "user_assertion_blake3_hex": "0123456789abcdef"
                    }),
                    ts_ms: 1_700_000_178_001,
                },
            ),
            // E-16 (SCHEMA_VERSION 179): VLM image-validation Pattern-2.
            // Explicit arm BEFORE the wildcard so write-time
            // validate_payload_matches_action_type sees a matching
            // payload type for `vlm_image_validation_event`.
            SsbActionType::VlmImageValidationEvent => SsbPayload::VlmImageValidationEvent(
                VlmImageValidationEventPayload {
                    kind: VlmImageValidationKind::SwapOrderDisagreement,
                    step_id: "step-e16-test".into(),
                    request_id: Some("req-e16-test".into()),
                    details_json: serde_json::json!({
                        "judge_a_verdict": "candidate_a_better",
                        "judge_b_verdict": "candidate_a_better",
                        "agreement_required": true,
                        "divergence_axis": "title_text_match"
                    }),
                    ts_ms: 1_700_000_179_001,
                },
            ),
            // E-44 (SCHEMA_VERSION 186): durability observability substrate.
            // Explicit arm BEFORE the wildcard so write-time
            // validate_payload_matches_action_type sees a matching
            // payload type for `durability_event`. TASK-E44-013 closure.
            SsbActionType::DurabilityEvent => SsbPayload::DurabilityEvent(
                DurabilityEventPayload {
                    kind: DurabilityEventKind::ToolIdempotencyHit,
                    session_id: Some("s-e44-test".into()),
                    agent_id: Some("agent-e44-test".into()),
                    details_json: serde_json::json!({
                        "caller_site": "test_payload",
                        "call_shape_hash": "deadbeef",
                        "tool_id": "Read",
                        "cache_age_ms": 1234_u64
                    }),
                    ts_ms: 1_700_000_186_001,
                },
            ),
            // E-31 (SCHEMA_VERSION 187): head_dim invariant verification.
            // Explicit arm BEFORE the wildcard so write-time
            // validate_payload_matches_action_type sees a matching
            // payload type for `head_dim_invariant_verified`. TASK-E31-001 closure.
            SsbActionType::HeadDimInvariantVerified => SsbPayload::HeadDimInvariantVerified(
                HeadDimInvariantVerifiedPayload {
                    kind: HeadDimInvariantVerifiedKind::Verified,
                    model_id: "Qwen/Qwen2.5-Coder-7B-Instruct".into(),
                    head_dim_observed: 128,
                    head_dim_expected: 128,
                    ok: true,
                    details_json: serde_json::json!({
                        "caller_site": "test_payload",
                        "hidden_size": 3584_u32,
                        "num_attention_heads": 28_u32
                    }),
                    ts_ms: 1_700_000_187_001,
                },
            ),
            // E-3 (SCHEMA_VERSION 188): role-shown / model-hidden default
            // telemetry. Explicit arm BEFORE the wildcard so write-time
            // validate_payload_matches_action_type sees a matching payload
            // type for `role_shown_model_hidden`. TASK-E03-001 MVP closure.
            SsbActionType::RoleShownModelHidden => SsbPayload::RoleShownModelHidden(
                RoleShownModelHiddenPayload {
                    role: "executor".into(),
                    model_hidden: true,
                    session_id: Some("s-e03-test".into()),
                    ts_ms: 1_700_000_188_001,
                },
            ),
            // E-49 (SCHEMA_VERSION 189): coding-agent metrics named-token
            // substrate. Explicit arm BEFORE the wildcard so write-time
            // validate_payload_matches_action_type sees a matching payload
            // type for `coding_metric_named_token`. TASK-E49-005 closure.
            SsbActionType::CodingMetricNamedToken => SsbPayload::CodingMetricNamedToken(
                CodingMetricNamedTokenPayload {
                    kind: CodingMetricNamedTokenKind::StablePassAt1,
                    session_id: "s-e49-test".into(),
                    model_id: Some("Qwen/Qwen2.5-Coder-7B-Instruct".into()),
                    details_json: serde_json::json!({
                        "caller_site": "test_payload",
                        "seed_count": 5_u32,
                        "identical_outputs": true,
                        "std_dev": 0.0_f64,
                        "mean_score": 1.0_f64,
                        "test_id": "test_payload"
                    }),
                    ts_ms: 1_700_000_189_001,
                },
            ),
            // E-3 sub-phase 4A (SCHEMA_VERSION 190): runtime handoff event
            // substrate. Explicit arm BEFORE the wildcard so write-time
            // validate_payload_matches_action_type sees a matching payload
            // type for `e3_runtime_handoff_event`. TASK-E03-001 sub-phase 4A.
            SsbActionType::E3RuntimeHandoffEvent => SsbPayload::E3RuntimeHandoffEvent(
                RuntimeHandoffEventPayload {
                    kind: RuntimeHandoffEventKind::UnifiedPreambleApplied,
                    role: Some("executor".into()),
                    backend: Some("sidecar".into()),
                    session_id: Some("s-e03-4a-test".into()),
                    details_json: serde_json::json!({
                        "caller_site": "test_payload",
                        "base_prompt_len": 256_u32,
                        "anchor_blake3": "deadbeef0000",
                        "preamble_len": 384_u32
                    }),
                    ts_ms: 1_700_000_190_001,
                },
            ),
            // E-31 sub-phase 6A (SCHEMA_VERSION 191): cloud-route decision
            // event substrate. Explicit arm BEFORE the wildcard so write-time
            // validate_payload_matches_action_type sees a matching payload
            // type for `route_decision_event`.
            SsbActionType::RouteDecisionEvent => SsbPayload::RouteDecisionEvent(
                RouteDecisionEventPayload {
                    kind: RouteDecisionEventKind::CloudRouteChosen,
                    route: "cloud_sonnet".into(),
                    score: 0.5,
                    ms: 3,
                    reason: "online_threshold_met".into(),
                    session_id: Some("s-e31-6a-test".into()),
                    details_json: serde_json::json!({
                        "caller_site": "test_payload",
                        "policy": "RuntimeRoutingPolicy",
                        "difficulty_bin": "medium",
                        "criticality": "low"
                    }),
                    ts_ms: 1_700_000_191_001,
                },
            ),
            // E-47 (SCHEMA_VERSION 192): LoRA ship-gate event substrate.
            // Explicit arm BEFORE the wildcard so write-time
            // validate_payload_matches_action_type sees a matching payload
            // type for `lora_ship_gate_event`.
            SsbActionType::LoraShipGateEvent => SsbPayload::LoraShipGateEvent(
                LoraShipGateEventPayload {
                    kind: LoraShipGateEventPhase::Refused,
                    session_id: Some("s-e47-test".into()),
                    agent_id: Some("test-agent".into()),
                    details_json: serde_json::json!({
                        "caller_site": "test_payload",
                        "reason": "deferral_token_present",
                        "token": "DEFERRED_PER_E47_REPORT"
                    }),
                    ts_ms: 1_700_000_192_001,
                },
            ),
            // E-40 Tranche B-2 (SCHEMA_VERSION 193): SymbolGraph
            // augmentation substrate. Explicit arm BEFORE the wildcard
            // so write-time validate_payload_matches_action_type sees a
            // matching payload type for `symbol_graph_event`.
            SsbActionType::SymbolGraphEvent => SsbPayload::SymbolGraphEvent(
                SymbolGraphEventPayload {
                    kind: SymbolGraphEventKind::SymbolDslExecuted,
                    session_id: Some("s-e40-test".into()),
                    agent_id: Some("test-agent".into()),
                    details_json: serde_json::json!({
                        "caller_site": "test_payload",
                        "dsl": "DEF(foo)",
                        "match_count": 1
                    }),
                    ts_ms: 1_700_000_193_001,
                },
            ),
            // Tranche B-6 quad (SCHEMA_VERSION 194): four new SSB substrates.
            SsbActionType::McpAuthzEvent => SsbPayload::McpAuthzEvent(
                McpAuthzEventPayload {
                    kind: McpAuthzEventKind::AuthzCheckPassed,
                    tool_name_hash: 0xC42_0001,
                    fsm_state_hash: 0xC42_0002,
                    actor_hash: 0xC42_0003,
                    layer_hash: 0xC42_0004,
                    deny_reason_hash: None,
                    severity: 0,
                    details_json: serde_json::json!({
                        "caller_site": "test_payload",
                        "layer": "allow"
                    }),
                    ts_ms: 1_700_000_194_001,
                },
            ),
            SsbActionType::ToolCalibrationEvent => SsbPayload::ToolCalibrationEvent(
                ToolCalibrationEventPayload {
                    kind: ToolCalibrationEventKind::CaseAdded,
                    tool_name_hash: 0xC47_0001,
                    call_shape_hash: 0xC47_0002,
                    response_kind_hash: None,
                    details_json: serde_json::json!({
                        "caller_site": "test_payload",
                        "case_id": 0
                    }),
                    ts_ms: 1_700_000_194_002,
                },
            ),
            SsbActionType::ReflectBudgetSoftWarn => SsbPayload::ReflectBudgetSoftWarn(
                ReflectBudgetSoftWarnPayload {
                    turn_id_hash: 0xB39_0001,
                    used: 17_000,
                    soft_cap: 17_000,
                    hard_cap: 20_000,
                    pct: 85,
                    ts_ms: 1_700_000_194_003,
                },
            ),
            SsbActionType::ContractCodingEvent => SsbPayload::ContractCodingEvent(
                ContractCodingEventPayload {
                    kind: ContractCodingEventKind::RetryResolved,
                    step_id_hash: 0xC50_0001,
                    iterations: 2,
                    severity: 0,
                    details_json: serde_json::json!({
                        "caller_site": "test_payload",
                        "final_state": "healed"
                    }),
                    ts_ms: 1_700_000_194_004,
                },
            ),
            // Tranche B-7 triple (SCHEMA_VERSION 195): TDD verification gate
            // substrate. Explicit arm BEFORE the wildcard so write-time
            // validate_payload_matches_action_type sees a matching payload
            // type for `tdd_verification_event`.
            SsbActionType::TddVerificationEvent => SsbPayload::TddVerificationEvent(
                TddVerificationEventPayload {
                    kind: TddVerificationEventKind::MutationGateSurvivorBlock,
                    task_id_hash: 0xE13_0001,
                    details_json: serde_json::json!({
                        "caller_site": "test_payload",
                        "mutation_id_hash": "0x1234abcd",
                        "status": "live_and_survived",
                        "cached_at_ms": 1_700_000_195_000_i64
                    }),
                    ts_ms: 1_700_000_195_001,
                },
            ),
            // F-1 (SCHEMA_VERSION 196): VibeGateBlocked Pattern-2 single
            // variant. Explicit arm so write-time validate_payload_matches_action_type
            // resolves the payload type correctly. Uses CodeVuln arm; the
            // source_blake3 field is the blake3 hex-prefix of the literal
            // "test_payload_vibe_gate" label.
            SsbActionType::VibeGateBlocked(_) => SsbPayload::VibeGateBlocked(
                VibeGateBlockedPayload {
                    kind: VibeGateBlockedKind::CodeVuln,
                    rule_id: Some("test-rule-id".into()),
                    cwe: Some("CWE-89".into()),
                    file_path: "src/main.rs".into(),
                    source_blake3: {
                        let h = blake3::hash(b"test_payload_vibe_gate");
                        format!("{:.16}", h.to_hex())
                    },
                    citation: Some("https://cwe.mitre.org/data/definitions/89.html".into()),
                    fsm_state: Some("executing".into()),
                },
            ),
            // F-2a (SCHEMA_VERSION 196): SecurityRuleBlocked flat beacon.
            // Explicit arm so write-time validation resolves the payload type.
            // The candidate_blake3 field is the blake3 hex-prefix of the
            // literal "test_payload_security_rule" label.
            SsbActionType::SecurityRuleBlocked => SsbPayload::SecurityRuleBlocked(
                SecurityRuleBlockedPayload {
                    rule_id: "no_raw_exec".into(),
                    cwe: "CWE-78".into(),
                    file_path: "src/runner.rs".into(),
                    language: "rust".into(),
                    candidate_blake3: {
                        let h = blake3::hash(b"test_payload_security_rule");
                        format!("{:.16}", h.to_hex())
                    },
                },
            ),
            // F-4 / Slice B-1 (SCHEMA_VERSION 197): TestIntegrityEvent Pattern-2
            // single variant. Explicit arm so write-time validation resolves the
            // payload type correctly. Uses Verified arm; stdout_hash is the
            // blake3 hex-prefix of the literal "test_payload_test_integrity".
            SsbActionType::TestIntegrityEvent(_) => SsbPayload::TestIntegrityEvent(
                TestIntegrityEventPayload {
                    test_name: "crate::ssb::tests::test_integrity_sample".into(),
                    passed: true,
                    stdout_hash: Some({
                        let h = blake3::hash(b"test_payload_test_integrity");
                        format!("{:.16}", h.to_hex())
                    }),
                    kind: TestIntegrityEventKind::Verified,
                },
            ),
            // F-3p1 / Slice C1 (SCHEMA_VERSION 198): ToolFilterEvent Pattern-2
            // single variant. Explicit arm so write-time validation resolves the
            // payload type correctly. Uses Committed arm; commit_hash is the
            // blake3 64-hex of the literal "test_payload_tool_filter_committed".
            SsbActionType::ToolFilterEvent(_) => SsbPayload::ToolFilterEvent(
                ToolFilterPayload {
                    commit_hash: blake3::hash(b"test_payload_tool_filter_committed")
                        .to_hex()
                        .to_string(),
                    allowed_categories: vec!["read".into(), "web".into()],
                    denied_tool: None,
                    taint_source: None,
                    task_id: "task-test-0".into(),
                    kind: ToolFilterEventKind::Committed,
                },
            ),
            // F-9 / Slice D1 (SCHEMA_VERSION 199): WriteTimeSlopsquatBlocked flat beacon.
            SsbActionType::WriteTimeSlopsquatBlocked => {
                SsbPayload::WriteTimeSlopsquatBlocked(WriteTimeSlopsquatBlockedPayload {
                    dep_name_hash: {
                        let h = blake3::hash(b"test_payload_write_time_slopsquat_blocked");
                        let bytes = h.as_bytes();
                        u64::from_be_bytes([
                            bytes[0], bytes[1], bytes[2], bytes[3],
                            bytes[4], bytes[5], bytes[6], bytes[7],
                        ])
                    },
                    ecosystem: "cargo".into(),
                    verdict_kind: WriteTimeSlopVerdictKind::NotFound.as_str().to_owned(),
                    registry_age_days: None,
                    ts_ms: 1_748_900_000_000,
                })
            }
            // F-10 / Slice A1 (SCHEMA_VERSION 200): SecurityClassRegressionBlocked flat
            // beacon. Explicit arm so write-time validation resolves the payload type.
            // delta_blake3 is the blake3 hex-prefix of the literal
            // "test_payload_security_class_regression" label.
            SsbActionType::SecurityClassRegressionBlocked => {
                SsbPayload::SecurityClassRegressionBlocked(
                    SecurityClassRegressionBlockedPayload {
                        delta_blake3: {
                            let h = blake3::hash(b"test_payload_security_class_regression");
                            format!("{:.16}", h.to_hex())
                        },
                        introduced_classes: vec!["CWE-78".into()],
                        prior_classes: vec!["CWE-89".into()],
                        loop_kind: LoopKind::Refine,
                        loop_id: "refine-loop-test-0".into(),
                        lang: "rust".into(),
                    },
                )
            }
            // F-2b / Slice A1 (SCHEMA_VERSION 201): InsecureRateMeasured flat beacon.
            // Explicit arm so write-time validation resolves the payload type.
            // corpus_blake3 is the blake3 hex-prefix of a fixed test label; the
            // before/after/recall numbers here are arbitrary fixtures (the real
            // keystone numbers are COMPUTED by the bench, never hard-coded).
            SsbActionType::InsecureRateMeasured => {
                SsbPayload::InsecureRateMeasured(InsecureRateMeasuredPayload {
                    suite: "secodeplt_structured_test".into(),
                    model_id: "test-model-0".into(),
                    n_samples: 4,
                    insecure_rate_before: 0.5,
                    insecure_rate_after: 0.25,
                    gate_recall: 0.5,
                    seed: 7,
                    corpus_blake3: {
                        let h = blake3::hash(b"test_payload_insecure_rate_measured");
                        format!("{:.16}", h.to_hex())
                    },
                })
            }
            // I-9 (SCHEMA_VERSION 263): ErrorAmplificationMeasured flat beacon.
            // Explicit arm so write-time validation resolves the payload type.
            // The alpha/recovered numbers here are arbitrary fixtures (the real
            // numbers are COMPUTED by the bench, never hard-coded).
            SsbActionType::ErrorAmplificationMeasured => {
                SsbPayload::ErrorAmplificationMeasured(ErrorAmplificationMeasuredPayload {
                    suite: "error_amplification_test".into(),
                    model_id: "test-fixture-0".into(),
                    n_steps: 6,
                    injected_step: 1,
                    divergence_step: Some(1),
                    alpha_t: 0.25,
                    recovered_fraction: 0.75,
                    repaired: true,
                    annealed_temperature: None,
                    seed: 7,
                    fixture_blake3: {
                        let h = blake3::hash(b"test_payload_error_amplification_measured");
                        format!("{:.16}", h.to_hex())
                    },
                })
            }
            // F-6 / Slice A1 (SCHEMA_VERSION 202): SkillLibraryEvent Pattern-2
            // single variant. Explicit arm so write-time validation resolves the
            // payload type correctly. Uses the Acquired arm (prior_tier None).
            SsbActionType::SkillLibraryEvent(_) => SsbPayload::SkillLibraryEvent(
                SkillLibraryEventPayload {
                    skill_id: "skill-test-0".into(),
                    description: "test learned skill".into(),
                    tier: "s1".into(),
                    prior_tier: None,
                    reason: "critic-gated acquisition".into(),
                    invocations: 0,
                    failure_rate: 0.0,
                    kind: SkillLibraryEventKind::Acquired,
                },
            ),
            // F-5 / Slice A1 (SCHEMA_VERSION 203): StalenessQuarantineBroadcast
            // flat beacon. Explicit arm so write-time validation resolves the
            // payload type correctly. broadcast_id is a blake3 hex-prefix of a
            // deterministic test fixture (no hard-coded cascade counts).
            SsbActionType::StalenessQuarantineBroadcast => {
                SsbPayload::StalenessQuarantineBroadcast(StalenessQuarantineBroadcastPayload {
                    fact_id: "fact-test-0".into(),
                    staleness_ratio: 0.92,
                    dependents_demoted: 3,
                    cosmos_artifacts_flagged: 1,
                    tick_ms: 1_700_000_000_000,
                    broadcast_id: {
                        let h = blake3::hash(b"test_payload_staleness_quarantine_broadcast");
                        format!("{:.16}", h.to_hex())
                    },
                    dropped_count: 0,
                })
            }
            // F-8 / Slice A1 (SCHEMA_VERSION 204): SymbolicConflictDetected flat
            // beacon. Explicit arm so write-time validation resolves the payload
            // type correctly. fingerprint is a blake3 hex-prefix of the conflict
            // tuple (no hard-coded precision/threshold numbers).
            SsbActionType::SymbolicConflictDetected => {
                SsbPayload::SymbolicConflictDetected(SymbolicConflictPayload {
                    fingerprint: {
                        let h = blake3::hash(
                            b"test_payload_symbolic_conflict|locked_key_contradiction|cloud_monthly_budget|cfg-variant-test-0",
                        );
                        format!("{:.16}", h.to_hex())
                    },
                    kind: "locked_key_contradiction".into(),
                    // Real entry from self_mod_safety::LOCKED_KEYS.
                    key: "cloud_monthly_budget".into(),
                    detail: "variant proposes raising cloud_monthly_budget; key is a self_mod_safety LOCKED invariant".into(),
                    candidate_id: "cfg-variant-test-0".into(),
                    fsm_substate: "AdmittingVariant".into(),
                })
            }
            // F-7 / Slice A1 (SCHEMA_VERSION 205): TemplateUtilityRanked flat
            // beacon. Explicit arm so write-time validation resolves the payload
            // type correctly. utility is laplace_utility(successes, total) =
            // (successes + 1) / (total + 2); a 1/1 template scores 2/3 ~= 0.667.
            SsbActionType::TemplateUtilityRanked => {
                SsbPayload::TemplateUtilityRanked(TemplateUtilityRankedPayload {
                    concept_class: "rest_crud_api".into(),
                    plan_skeleton_hash: 0x5f3e_27a1_0b9c_d4e2,
                    // 1 success / 1 firing -> (1+1)/(1+2) = 2/3.
                    utility: 2.0_f32 / 3.0_f32,
                    first_try_ratio: 1.0,
                    sample_n: 1,
                    chosen_pipeline_id: "pipeline-test-0".into(),
                })
            }
            // F-13 / Slice A1 (SCHEMA_VERSION 206): SessionArchived flat beacon.
            // Explicit arm so write-time validation resolves the payload type
            // correctly. manifest_blake3 is a blake3 hex-prefix of the archived
            // bytes (no hard-coded threshold numbers).
            SsbActionType::SessionArchived => {
                SsbPayload::SessionArchived(SessionArchivedPayload {
                    session_id: "test-session-0".into(),
                    stale_version: 205,
                    archive_path:
                        "workflow/sessions/test-session-0/archive/ssb-v205.sqlite".into(),
                    manifest_blake3: {
                        let h = blake3::hash(b"test_payload_session_archived|ssb-v205.sqlite");
                        format!("{:.16}", h.to_hex())
                    },
                    store_kind: "ssb".into(),
                })
            }
            // F-11 / Slice A1 (SCHEMA_VERSION 207): DarwinianAdmit config-DGM
            // admit-loop beacon. Explicit arm so write-time validation resolves
            // the payload type. candidate_blake3 is a blake3 hex-prefix (no
            // hard-coded threshold numbers); kind is the LoopClosed arm here.
            SsbActionType::DarwinianAdmit => {
                SsbPayload::DarwinianAdmit(DarwinianAdmitPayload {
                    kind: DarwinianAdmitKind::LoopClosed,
                    variant_id: "test-variant-0".into(),
                    parent_id: Some("test-parent-0".into()),
                    alpha: 0.61,
                    guard_rejected_by: None,
                    candidate_blake3: {
                        let h = blake3::hash(b"test_payload_darwinian_admit|test-variant-0");
                        format!("{:.16}", h.to_hex())
                    },
                    seed: 42,
                })
            }
            // I-39 (SCHEMA_VERSION 230): explicit arm so test_payload(PlanStateDerived)
            // returns a real PlanStateDerivedPayload instead of falling to the wildcard
            // Finding fallthrough. Mirror the field values used in all_test_payloads().
            SsbActionType::PlanStateDerived => SsbPayload::PlanStateDerived(PlanStateDerivedPayload {
                letter: "B".into(),
                session_id: "s-i39-test".into(),
                derived_state: "SHIPPED_WIRED".into(),
                evidence_anchor: "fragments:s-plan-audit-2026-06-04:B-10".into(),
                ts_ms: 1_700_000_000_000,
            }),
            // I-45 (SCHEMA_VERSION 231): explicit arm so test_payload(ObjectiveSealed)
            // returns a real ObjectiveSealedPayload instead of falling to the wildcard.
            SsbActionType::ObjectiveSealed(_) => SsbPayload::ObjectiveSealed(ObjectiveSealedPayload {
                kind: ObjectiveSealedKind::Sealed,
                objective_hash: "abcd1234abcd1234".into(),
                current_hash: None,
                session_id: "s-i45-test".into(),
                ts_ms: 1_700_000_000_000,
            }),
            // I-34 (SCHEMA_VERSION 234): explicit arm so test_payload(GrammarGateEvent)
            // returns a real GrammarGateEventPayload instead of falling to the wildcard.
            SsbActionType::GrammarGateEvent(_) => SsbPayload::GrammarGateEvent(GrammarGateEventPayload {
                kind: GrammarGateEventKind::CranePassed,
                grammar_path: "fixtures/test_grammar.ebnf".into(),
                pin_violation_expected: None,
                pin_violation_actual: None,
                session_id: "s-i34-test".into(),
                ts_ms: 1_700_000_000_000,
            }),
            // J-14 (SCHEMA_VERSION 282): explicit arm so test_payload(ReproPackEvent)
            // returns a real ReproPackPayload instead of falling to the wildcard.
            SsbActionType::ReproPackEvent(_) => SsbPayload::ReproPackEvent(ReproPackPayload {
                kind: ReproPackKind::TierEvaluated,
                tier_effective: Some("action_faithful".into()),
                tier_requested: Some("action_faithful".into()),
                downgrades: Vec::new(),
                divergence_position: None,
                expected_value: None,
                actual_value: None,
                template_hash: None,
                expected_sequence_hash: None,
                actual_sequence_hash: None,
                fixture_path: None,
                fixture_blake3: None,
                session_id: "s-j14-test".into(),
                ts_ms: 1_700_000_000_000,
            }),
            // J-16 (SCHEMA_VERSION 283): explicit arm so test_payload(BlastRadiusRegressionEvent)
            // returns a real BlastRadiusRegressionPayload instead of falling to the wildcard.
            SsbActionType::BlastRadiusRegressionEvent(_) => SsbPayload::BlastRadiusRegressionEvent(BlastRadiusRegressionPayload {
                kind: BlastRadiusRegressionKind::ConeComputed,
                changed_symbols: vec!["src/main.rs".into()],
                cone: vec!["src/main.rs".into(), "src/lib.rs".into()],
                scope_violations: Vec::new(),
                regression_set: Vec::new(),
                silent_deletions: Vec::new(),
                cone_truncated: false,
                session_id: "s-j16-test".into(),
                ts_ms: 1_700_000_000_000,
            }),
            // J-30 T7b (SCHEMA_VERSION 284): explicit arm so test_payload(ProviderByokEvent)
            // returns a real ProviderByokEventPayload instead of falling to the wildcard.
            SsbActionType::ProviderByokEvent => SsbPayload::ProviderByokEvent(ProviderByokEventPayload {
                backend: "anthropic".into(),
                kind: ProviderByokEventKind::KeyStored,
                ok: true,
                ts_ms: 1_700_000_000_000,
            }),
            // J-32 (SCHEMA_VERSION 285): explicit arm so test_payload(IntentRouted)
            // returns a real IntentRoutedPayload instead of falling to the wildcard.
            SsbActionType::IntentRouted => SsbPayload::IntentRouted(IntentRoutedPayload {
                lane: FrontDoorLane::Task,
                tier: FrontDoorTier::T1,
                confidence: 90,
                arg_spans: vec![ArgSpan { start: 0, end: 11 }],
                kind: IntentRoutedKind::Classified,
                risk: RiskLevel::Low,
                ts_ms: 1_700_000_000_000,
            }),
            // J-34 (SCHEMA_VERSION 286): explicit arm so test_payload(ContextCalibrationEvent)
            // returns a real ContextCalibrationEventPayload instead of falling to the wildcard.
            SsbActionType::ContextCalibrationEvent(_) =>
                SsbPayload::ContextCalibrationEvent(ContextCalibrationEventPayload {
                    kind: ContextCalibrationKind::CurveMeasured,
                    model_identity_key: "test-identity".into(),
                    details_json: serde_json::json!({
                        "sample_count": 6,
                        "knee_fill_fraction": 0.50,
                        "drop_magnitude": 0.35,
                        "content_hash": "deadbeef",
                    }),
                    ts_ms: 1_700_000_000_000,
                }),
            // J-36 (SCHEMA_VERSION 287): explicit arm so test_payload(LocalPrefixCacheEvent)
            // returns a real LocalPrefixCacheEventPayload instead of falling to the wildcard.
            SsbActionType::LocalPrefixCacheEvent(_) =>
                SsbPayload::LocalPrefixCacheEvent(LocalPrefixCacheEventPayload {
                    kind: LocalPrefixCacheKind::CacheIneligible,
                    head_prefill_tokens: 0,
                    tail_prefill_tokens: 0,
                    eligibility_reason: "prefix_kv_ineligible".into(),
                    ts_ms: 1_700_000_000_000,
                }),
            // J-24 (SCHEMA_VERSION 287): explicit arm so test_payload(BehaviorSteeringEvent)
            // returns a real BehaviorSteeringEventPayload instead of falling to the wildcard.
            SsbActionType::BehaviorSteeringEvent(_) =>
                SsbPayload::BehaviorSteeringEvent(BehaviorSteeringEventPayload {
                    kind: BehaviorSteeringKind::ObservationSizePreassessed,
                    details_json: serde_json::json!({
                        "observation_tokens": 4096,
                        "steering_eligible": true,
                    }),
                    ts_ms: 1_700_000_000_000,
                }),
            // J-35 (SCHEMA_VERSION 289): explicit arm so test_payload(IntentControlEvent)
            // returns a real IntentControlEventPayload instead of falling to the wildcard.
            SsbActionType::IntentControlEvent(_) =>
                SsbPayload::IntentControlEvent(IntentControlEventPayload {
                    kind: IntentControlKind::SpecReconstructed,
                    spec_id: "j35-test-spec".into(),
                    premise_flag_count: 0,
                    route_decision: "proceed_under_assumptions".into(),
                    assumption_count: 0,
                    reversibility_class: "reversible".into(),
                    gate_mode: "autonomous".into(),
                    details_json: serde_json::json!({
                        "goal": "test goal",
                    }),
                    ts_ms: 1_700_000_000_000,
                }),
            // s-68264 audit cleanup: dedup'd 3 duplicate CliMcp* arms and added wildcard
            // to cover the 27 new action types (E-15 sandbox hardening, E-24 MCP hardening,
            // C-48 SafeHarbor, E-30 public incident, E-32 context window). The test_payload
            // helper only needs a valid msgpack payload to pass write-time validation; the
            // Finding shape is the smallest available with stable schema.
            _ => SsbPayload::Finding(FindingPayload {
                severity: "INFO".into(),
                category: "test".into(),
                message: "test_payload fallback".into(),
                file_path: None,
                line: None,
                failure_category: None,
                audit_step: None,
                few_shot_examples: None,
            }),
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
                    if (table == "ssb" || table == "ssb_files" || table == "holdout_budget")
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

        // holdout_budget table: CREATE + INSERT (no violation), then UPDATE + DELETE (violations)
        direct_conn
            .execute(
                "CREATE TABLE IF NOT EXISTS holdout_budget (\
                 id INTEGER PRIMARY KEY AUTOINCREMENT,\
                 row_id TEXT NOT NULL DEFAULT '',\
                 kind TEXT NOT NULL DEFAULT 'spent',\
                 holdout_id TEXT NOT NULL,\
                 objective TEXT NOT NULL,\
                 data_window TEXT NOT NULL DEFAULT '',\
                 budget_total INTEGER NOT NULL DEFAULT 0,\
                 seed INTEGER NOT NULL DEFAULT 0,\
                 created_at_ms INTEGER NOT NULL DEFAULT 0,\
                 session_id TEXT NOT NULL DEFAULT '')",
                [],
            )
            .expect("create holdout_budget table");

        direct_conn
            .execute(
                "INSERT INTO holdout_budget (holdout_id, objective) VALUES ('h1', 'obj1')",
                [],
            )
            .expect("insert holdout_budget row");
        assert_eq!(
            violation_count.load(Ordering::Relaxed),
            2,
            "INSERT into holdout_budget should not trigger append-only hook"
        );

        direct_conn
            .execute(
                "UPDATE holdout_budget SET kind = 'provision' WHERE id = 1",
                [],
            )
            .expect("update holdout_budget executes");
        assert_eq!(
            violation_count.load(Ordering::Relaxed),
            3,
            "UPDATE on holdout_budget should trigger append-only hook"
        );

        direct_conn
            .execute("DELETE FROM holdout_budget WHERE id = 1", [])
            .expect("delete holdout_budget executes");
        assert_eq!(
            violation_count.load(Ordering::Relaxed),
            4,
            "DELETE on holdout_budget should trigger append-only hook"
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
                stdout_hash: None,
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
                origin_role: None, origin_run_id: None,
                originating_mast_mode: None, content_hash: None, provisional: false,
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
            // B-30 (SCHEMA_VERSION 36) — language grammar registry status row.
            SsbPayload::LanguageGrammarStatus(LanguageGrammarStatusPayload {
                loaded: vec!["go".into(), "python".into(), "rust".into()],
                failed: vec![("kotlin".into(), "ABI mismatch".into())],
            }),
            // B-29 Phase 5 (SCHEMA_VERSION 37) — symbol-graph tool call row.
            SsbPayload::SymbolQuery(SymbolQueryPayload {
                tool: "symbol_impact".into(),
                target: "crates/knowledge/src/symbol_graph/traversal.rs".into(),
                hop_count: 2,
                total_files: 7,
                latency_ms: 3,
            }),
            // B-31.5 (SCHEMA_VERSION 39) — 5 deferred B-31 SSB variants.
            SsbPayload::IndexCheckpoint(IndexCheckpointPayload {
                project_id: "abc123def4567890".into(),
                batches_processed: 2,
                batches_total: Some(4),
                files_processed: 100,
                last_checkpoint_at: 1_700_000_000,
                batch_duration_ms: Some(250),
            }),
            SsbPayload::IndexResume(IndexResumePayload {
                project_id: "abc123def4567890".into(),
                resumed_from_batch: 1,
                files_skipped: 20,
            }),
            SsbPayload::IndexCancel(IndexCancelPayload {
                project_id: "abc123def4567890".into(),
                batches_completed: 1,
                files_indexed: 50,
            }),
            SsbPayload::LockAcquired(LockAcquiredPayload {
                project_id: "abc123def4567890".into(),
                scope: "index".into(),
                holder_pid: 12345,
            }),
            SsbPayload::LockContested(LockContestedPayload {
                project_id: "abc123def4567890".into(),
                scope: "index".into(),
                holder_pid: 99999,
            }),
            // B-33 follow-up (SCHEMA_VERSION 40) — symbol-graph build summary row.
            SsbPayload::SymbolGraphUpdated(SymbolGraphUpdatedPayload {
                project_id: "abc123def4567890".into(),
                file_count: 42,
                symbol_count: 1234,
                edge_count: 5678,
                built_ms: 1_700_000_000_000,
                // C-24 (DRIFT-4): exercise the additive Option<bool> field
                // explicitly in the determinism vector so rmp-serde encoding
                // round-trips Some(true) too (the field is skipped when
                // None per #[serde(skip_serializing_if]).
                is_incremental: Some(true),
            }),
            // B-40 (SCHEMA_VERSION 41) — 5 FRAMING substate beacons.
            SsbPayload::IdeaFramingStarted(IdeaFramingStartedPayload {
                session_id: "s-test".into(),
                prompt: "build a todo app".into(),
                started_at_ms: 1_700_000_000_000,
            }),
            SsbPayload::ProjectClassClassified(ProjectClassClassifiedPayload {
                session_id: "s-test".into(),
                project_class: "saas".into(),
                matched_pattern: Some("react+postgres".into()),
            }),
            SsbPayload::CriticRoundComplete(CriticRoundCompletePayload {
                session_id: "s-test".into(),
                round_index: 1,
                block_count: 0,
                warn_count: 2,
                info_count: 3,
                converged: true,
                category_counts: vec![("clarity".into(), 1), ("scope".into(), 1)],
            }),
            SsbPayload::IdeaFramed(IdeaFramedPayload {
                session_id: "s-test".into(),
                artifact_path: "workflow/sessions/s-test/idea.md".into(),
                project_class: "saas".into(),
                rounds_used: 2,
                tradeoff_count: 3,
                converged: true,
            }),
            SsbPayload::TradeoffCardEmitted(TradeoffCardEmittedPayload {
                session_id: "s-test".into(),
                tradeoff_names: vec!["db".into(), "auth".into()],
                option_counts: vec![3, 2],
            }),
            // B-34 (SCHEMA_VERSION 42) — entry-point + artifact pipeline beacons.
            SsbPayload::EntryPointsDetected(EntryPointsDetectedPayload {
                project_id: "abc123def4567890".into(),
                count: 3,
                by_reason: {
                    let mut m = std::collections::BTreeMap::new();
                    m.insert("orphan".into(), 1);
                    m.insert("well_known:main".into(), 2);
                    m
                },
            }),
            SsbPayload::ArtifactsIndexed(ArtifactsIndexedPayload {
                project_id: "abc123def4567890".into(),
                indexed: vec!["readme".into()],
                up_to_date: vec!["license".into()],
                total_chunks: 12,
                latency_ms: 45,
            }),
            SsbPayload::ArtifactSearched(ArtifactSearchedPayload {
                project_id: "abc123def4567890".into(),
                artifact_name: Some("readme".into()),
                query: "install".into(),
                result_count: 4,
                latency_ms: 7,
            }),
            SsbPayload::ArtifactStale(ArtifactStalePayload {
                project_id: "abc123def4567890".into(),
                artifact_name: "readme".into(),
                changed_files: vec!["README.md".into()],
            }),
            // B-38 (SCHEMA_VERSION 43) — ambiguity continuous-metric beacons.
            SsbPayload::AmbiguityScored(AmbiguityScoredPayload {
                score: 0.42,
                milestone: "progress".into(),
                goal: 0.5,
                constraint: 0.4,
                success: 0.4,
                context: Some(0.3),
                brownfield: true,
            }),
            SsbPayload::AmbiguityFloorFailed(AmbiguityFloorFailedPayload {
                score: 0.2,
                failed_dimensions: vec!["goal_clarity".into()],
            }),
            SsbPayload::AmbiguityStreakAdvanced(AmbiguityStreakAdvancedPayload {
                score: 0.9,
                current_streak: 2,
                required: 3,
            }),
            // B-42 (SCHEMA_VERSION 44) — cosmos tier knowledge worker beacons.
            SsbPayload::CosmosRefreshStarted(CosmosRefreshStartedPayload {
                queued: 3,
                total: 12,
            }),
            SsbPayload::CosmosArtifactUpdated(CosmosArtifactUpdatedPayload {
                name: "saas_pricing_patterns".into(),
                blake3: "f00dfeedcafebabe".into(),
                body_bytes: 4096,
                latency_ms: 120,
            }),
            SsbPayload::CosmosArtifactDecayed(CosmosArtifactDecayedPayload {
                name: "saas_pricing_patterns".into(),
                age_secs: 86_400,
                cadence: "daily".into(),
            }),
            SsbPayload::CosmosGroundingUsed(CosmosGroundingUsedPayload {
                claim_category: "monetization".into(),
                artifact_name: "saas_pricing_patterns".into(),
                matched: true,
            }),
            // B-41 (SCHEMA_VERSION 45) — per-level auditor mode beacons.
            SsbPayload::PerLevelAuditStarted(PerLevelAuditStartedPayload {
                mode: "concepts".into(),
                tier: "concept".into(),
                node_id: 1,
            }),
            SsbPayload::PerLevelAuditVerdict(PerLevelAuditVerdictPayload {
                mode: "phases".into(),
                tier: "phase".into(),
                node_id: 2,
                verdict: "allow".into(),
                findings_count: 0,
                respawned_once: false,
                cascade_to_parent: false,
            }),
            SsbPayload::PerLevelAuditCascade(PerLevelAuditCascadePayload {
                mode: "babystep_detail".into(),
                tier: "baby_step".into(),
                node_id: 42,
                parent_tier: "baby_step".into(),
            }),
            // B-37C (SCHEMA_VERSION 46) — multi-runtime backend beacons.
            SsbPayload::RuntimeFallback(RuntimeFallbackPayload {
                role: elai_ipc_types::AgentRole::Executor,
                primary: elai_providers::runtime::RuntimeBackend::Sidecar,
                attempted: vec![
                    elai_providers::runtime::RuntimeBackend::Sidecar,
                    elai_providers::runtime::RuntimeBackend::AnthropicApi,
                ],
                succeeded: Some(elai_providers::runtime::RuntimeBackend::AnthropicApi),
                reason: "sidecar unavailable, fell through to anthropic_api".into(),
            }),
            SsbPayload::RuntimeHealthChanged(RuntimeHealthChangedPayload {
                backend: elai_providers::runtime::RuntimeBackend::Sidecar,
                prev: elai_providers::runtime::RuntimeHealth::Unavailable {
                    reason: "first observation".into(),
                },
                next: elai_providers::runtime::RuntimeHealth::Ready,
            }),
            SsbPayload::CliSubprocessSpawned(CliSubprocessSpawnedPayload {
                backend: elai_providers::runtime::RuntimeBackend::ClaudeCode,
                pid: 12345,
                argv_redacted: vec!["claude".into(), "-p".into(), "[REDACTED]".into()],
            }),
            SsbPayload::CliSubprocessKilled(CliSubprocessKilledPayload {
                backend: elai_providers::runtime::RuntimeBackend::ClaudeCode,
                pid: 12345,
                exit_code: Some(0),
                reason: "normal exit".into(),
            }),
            // C-1 (SCHEMA_VERSION 47) — 7 knowledge + reasoning producer payloads.
            SsbPayload::GrcStepEmitted(GrcStepEmittedPayload {
                agent_id: "executor".into(),
                topic: "task_runner".into(),
                step_type: "observe".into(),
                grounding_level: "high".into(),
                confidence: 0.85,
                budget_remaining: 5,
                elapsed_ms: 12,
            }),
            SsbPayload::GroundingSweepResult(GroundingSweepResultPayload {
                ungrounded: 1,
                verify_needed: 0,
                grounded: 3,
                top_claim: Some("training:placeholder".into()),
            }),
            SsbPayload::CsqEvaluated(CsqEvaluatedPayload {
                severity: "Critical".into(),
                guard_present: false,
                blocked: true,
                check_count: 3,
            }),
            SsbPayload::AuditorRubberStampRejected(AuditorRubberStampRejectedPayload {
                role: "auditor".into(),
                finding_count: 0,
                expected_min: 1,
            }),
            SsbPayload::PerRoleDecodingApplied(PerRoleDecodingAppliedPayload {
                role: "auditor".into(),
                decoding_mode: "json_schema".into(),
                constrained: true,
                max_retries_on_invalid: 3,
            }),
            SsbPayload::HydeQuery(HydeQueryPayload {
                query_hash: "abc123".into(),
                hypothesis_len: 128,
                cache_hit: false,
                latency_ms: 12,
            }),
            SsbPayload::VectorCacheHitRate(VectorCacheHitRatePayload {
                session_id: "test-session".into(),
                hits: 7,
                misses: 3,
                hit_rate_pct: 70.0,
            }),
            SsbPayload::QueryTrace(QueryTracePayload {
                query_id: "11111111-2222-3333-4444-555555555555".into(),
                tier: "STANDARD".into(),
                stage_count: 9,
                result_count: 12,
                total_latency_ms: 245,
            }),
            // C-2 (SCHEMA_VERSION 48) — 10 orchestration + cosmos + stagnation payloads.
            SsbPayload::PhaseNodeReady(PhaseNodeReadyPayload {
                phase_id: 7,
                concept_id: 3,
                label: "phase-test".into(),
                depth: 1,
            }),
            SsbPayload::StepNodeReady(StepNodeReadyPayload {
                step_id: 12,
                phase_id: 7,
                concept_id: 3,
                label: "step-test".into(),
                depth: 2,
            }),
            SsbPayload::BabyStepReady(BabyStepReadyPayload {
                baby_step_id: 42,
                step_id: 12,
                phase_id: 7,
                concept_id: 3,
                label: "baby-step-test".into(),
                depth: 3,
            }),
            SsbPayload::DagRepairEscalated(DagRepairEscalatedPayload {
                rounds: 10,
                remaining_issues: 2,
                last_error: "cycle persists after repair".into(),
            }),
            SsbPayload::AgentStatusChanged(AgentStatusChangedPayload {
                agent_id: "executor-1".into(),
                role: "executor".into(),
                state: "spawned".into(),
                death_class: None,
                timestamp_ms: 1_700_000_000_000,
            }),
            SsbPayload::StagnationSpinningDetected(StagnationSpinningDetectedPayload {
                agent_id: "executor-1".into(),
                pattern: "spinning".into(),
                iteration: 3,
                detail: "step_hash repeated 3 times".into(),
            }),
            SsbPayload::StagnationOscillationDetected(StagnationOscillationDetectedPayload {
                agent_id: "executor-1".into(),
                pattern: "oscillation".into(),
                iteration: 4,
                detail: "period-2 A-B-A-B".into(),
            }),
            SsbPayload::LateralThinkingActivated(LateralThinkingActivatedPayload {
                agent_id: "executor-1".into(),
                persona: "contrarian".into(),
                iteration: 5,
                detail: "switched after spinning".into(),
            }),
            SsbPayload::GoalDriftWarning(GoalDriftWarningPayload {
                agent_id: "executor-1".into(),
                drift_score: 0.42,
                baby_step_id: 17,
                detail: "task drifted from intent".into(),
            }),
            SsbPayload::WatchdogTimeout(WatchdogTimeoutPayload {
                agent_id: "executor-1".into(),
                elapsed_ms: 30_000,
                budget_ms: 20_000,
                detail: "no observable output".into(),
            }),
            // C-7 (SCHEMA_VERSION 49) — reviewer verdict beacon.
            SsbPayload::ChangeDecision(ChangeDecisionPayload {
                decision: "accepted".into(),
                path: "src/foo.rs".into(),
                hunk_id: Some("hunk-1".into()),
                note: None,
            }),
            // C-8 (SCHEMA_VERSION 50) — search-mode cross-process bridge.
            SsbPayload::SearchModeChanged(SearchModeChangedPayload {
                mode: "hybrid".into(),
                history: vec!["lexical".into(), "hybrid".into()],
            }),
            // C-5 (SCHEMA_VERSION 51) — privacy-redaction event.
            SsbPayload::RedactionApplied(RedactionAppliedPayload {
                count: 3,
                fields: vec!["task_summary".into(), "escalation_message".into()],
                baby_step_id: 17,
                ts_ms: 0,
            }),
            // C-10 (SCHEMA_VERSION 52) — lateral-thinking + goal-drift Block.
            SsbPayload::LateralThinkingSucceeded(LateralThinkingSucceededPayload {
                agent_id: "executor-1".into(),
                persona: "hacker".into(),
                iteration: 6,
                detail: "pattern broken after hacker persona".into(),
            }),
            SsbPayload::LateralThinkingExhausted(LateralThinkingExhaustedPayload {
                agent_id: "executor-1".into(),
                personas_tried: 5,
                iteration: 10,
                detail: "all personas tried; pattern persists".into(),
            }),
            SsbPayload::GoalDriftBlock(GoalDriftBlockPayload {
                agent_id: "executor-1".into(),
                drift_score: 0.32,
                baby_step_id: 19,
                detail: "drift_score below block threshold".into(),
            }),
            SsbPayload::WipSquashComplete(WipSquashCompletePayload {
                wip_count: 7,
                squashed: true,
                step_id: "step-42".into(),
            }),
            SsbPayload::AlternativeExplored(AlternativeExploredPayload {
                step_id: "step-c22".into(),
                alternative_index: 1,
                accepted: true,
            }),
            SsbPayload::FeasibilityEscalated(FeasibilityEscalatedPayload {
                reason: "vram_margin_breached".into(),
                variant: "resource_margin_breached".into(),
                step_id: "step-c22".into(),
            }),
            // C-23 (SCHEMA_VERSION 55): SessionDag branch-UI beacons.
            SsbPayload::BranchForked(BranchForkedPayload {
                branch_name: "feature-x".into(),
                tip_snap_id: "snap-c23-tip".into(),
                parent_snap_id: "snap-c23-parent".into(),
            }),
            SsbPayload::BranchMerged(BranchMergedPayload {
                winner_name: "feature-x".into(),
                trunk_snap_id: "snap-c23-trunk".into(),
                abandoned_names: vec!["experiment-y".into()],
            }),
            SsbPayload::BranchClosed(BranchClosedPayload {
                branch_name: "experiment-y".into(),
                reason: "user-requested".into(),
            }),
            // C-25 (SCHEMA_VERSION 56): compiled-pipeline + template-crystallizer
            // + background-task beacons.
            SsbPayload::PlanSkeleton(PlanSkeletonPayload {
                plan_id: "plan-c25-test".into(),
                step_count: 3,
                concept_class: "Counter".into(),
                plan_skeleton_hash: 0xC25_AAAA_BBBB_CCCC,
                source: PlanSkeletonSource::Llm,
            }),
            SsbPayload::CompiledPipelineFired(CompiledPipelineFiredPayload {
                pipeline_id: "pl-c25-test".into(),
                concept_class: "Counter".into(),
                plan_skeleton_hash: 0xC25_AAAA_BBBB_CCCC,
                use_count: 4,
            }),
            SsbPayload::TemplateCrystallized(TemplateCrystallizedPayload {
                concept_class: "Counter".into(),
                plan_skeleton_hash: 0xC25_AAAA_BBBB_CCCC,
                pipeline_id: "pl-c25-test".into(),
                threshold_n: 3,
            }),
            SsbPayload::BackgroundTaskStarted(BackgroundTaskStartedPayload {
                task_id: "bg-c25-test".into(),
                command_summary: "grep -r foo /large".into(),
                soft_timeout_secs: 30,
                hard_timeout_secs: 600,
            }),
            SsbPayload::BackgroundTaskComplete(BackgroundTaskCompletePayload {
                task_id: "bg-c25-test".into(),
                exit_status: 0,
                duration_secs: 12,
                auto_detached: false,
                timed_out: false,
            }),
            // C-25 audit follow-up (SCHEMA_VERSION 57): missing 6th
            // compiled-pipeline beacon (WARN-1 in audit-C25.md).
            SsbPayload::DecompileTriggered(DecompileTriggeredPayload {
                concept_class: "Counter".into(),
                plan_skeleton_hash: 0xC25_AAAA_BBBB_CCCC,
                failure_count: 2,
                trigger_reason: DecompileTriggerReason::ConsecutiveFailures,
            }),
            // C-26 (SCHEMA_VERSION 58): constrained-decoding + futility primitives.
            SsbPayload::FutilityDetected(FutilityDetectedPayload {
                error_class: "constrained_decode_reject".into(),
                consecutive_count: 3,
                level: "baby_step".into(),
                escalated: true,
            }),
            SsbPayload::FutilityOutputStagnation(FutilityOutputStagnationPayload {
                similarity: 0.95,
                threshold: 0.90,
                level: "baby_step".into(),
                new_persona: Some("hacker".into()),
            }),
            // C-27 (SCHEMA_VERSION 59): Testing-trust T(-1) + T-0 beacons.
            SsbPayload::WiringEvidence(WiringEvidencePayload {
                entrypoint: "crates/cli/src/main.rs::main".into(),
                produces_symbols_verified: vec![
                    "DeterminismConfig".into(),
                    "WiringContract".into(),
                ],
                reachable_count: 2,
                audit_id: uuid::Uuid::nil(),
            }),
            SsbPayload::WiringViolation(WiringViolationPayload {
                violation_kind: WiringViolationKind::StubPattern,
                file: "crates/orchestrator/src/example.rs".into(),
                line: 99,
                pattern: "todo!()".into(),
                audit_id: uuid::Uuid::nil(),
            }),
            SsbPayload::DeterminismDrift(DeterminismDriftPayload {
                seed: 42,
                run_a_hash: "blake3:aaaa".into(),
                run_b_hash: "blake3:bbbb".into(),
                first_divergence_index: 13,
                audit_id: uuid::Uuid::nil(),
            }),
            // C-28 (SCHEMA_VERSION 60): T-1 mutation testing.
            SsbPayload::MutationSurvived(MutationSurvivedPayload {
                file: "crates/orchestrator/src/example.rs".into(),
                line: 7,
                mutant_kind: "replace_+_with_-".into(),
            }),
            // C-29 (SCHEMA_VERSION 61): T-2 IntentPair + T-3 Metamorphic.
            SsbPayload::IntentConfirmed(IntentConfirmedPayload {
                step_id: 1,
                pair_count: 3,
                confirmed_by: "user".into(),
            }),
            SsbPayload::MrViolated(MrViolatedPayload {
                step_id: 1,
                relation: "inverse".into(),
                counterexample: "sample_index=0 input_hex=01 encoded_hex=ff decoded_hex=01"
                    .into(),
                sample_size: 1,
            }),
            // C-30 (SCHEMA_VERSION 62): Surgical Mode + Oracle Mode payloads.
            SsbPayload::SurgicalPlanCreated(SurgicalPlanCreatedPayload {
                task_id: "task_1700000000_c30test".into(),
                complexity_class: "simple".into(),
                blast_radius_files: 2,
                total_loc_estimate: 150,
                step_count: 1,
                regression_gate_snapshot_hash: "blake3:c30aaaaa".into(),
            }),
            SsbPayload::SurgicalLocWarning(SurgicalLocWarningPayload {
                task_id: "task_1700000000_c30test".into(),
                total_loc_estimate: 600,
                threshold: 500,
                mode: "interactive".into(),
            }),
            SsbPayload::OracleCacheHit(OracleCacheHitPayload {
                query_hash_hex: "00".repeat(32),
                source_tier: "t1_authoritative".into(),
                cached_age_secs: 60,
                source: None,
            }),
            SsbPayload::OracleCacheMiss(OracleCacheMissPayload {
                query_hash_hex: "00".repeat(32),
                confidence: "high".into(),
                source: None,
            }),
            SsbPayload::ProjectDetectionSignal(ProjectDetectionSignalPayload {
                signal_name: "cargo_toml".into(),
                weight: "required".into(),
                fired: true,
            }),
            // C-32 (SCHEMA_VERSION 63): Code-Completeness Layer 2 payloads.
            SsbPayload::SkeletonGenerated(SkeletonGeneratedPayload {
                blank_count: 4,
                language: "rust".into(),
                file_path: "src/lib.rs".into(),
            }),
            SsbPayload::SkeletonFlushed(SkeletonFlushedPayload {
                path: "src/lib.rs".into(),
                completeness_ratio: 1.0,
            }),
            SsbPayload::MiniCriticVerdict(MiniCriticVerdictPayload {
                blank_id: 1,
                verdict: "pass".into(),
                reason: "ok".into(),
                language: "rust".into(),
            }),
            SsbPayload::StructuralRetryStageAttempt(StructuralRetryStageAttemptPayload {
                blank_id: 1,
                stage: "template".into(),
                attempt: 1,
                outcome: "converged".into(),
            }),
            SsbPayload::BridgeFileGenerated(BridgeFileGeneratedPayload {
                kind: "mod_rs_aggregator".into(),
                path: "src/mod.rs".into(),
                module_count: 3,
            }),
            // D-1 (SCHEMA_VERSION 64): ReasoningTemplate ArchiveChain lifecycle.
            SsbPayload::ArchiveEntryAdded(ArchiveEntryAddedPayload {
                template_id: "00000000000000000000000000000001".into(),
                content_hash: "ff".repeat(32),
                parent_hash: None,
                domain: "code".into(),
                pattern_kind: "decomposition".into(),
                archive_status: "frozen".into(),
            }),
            SsbPayload::ArchiveEntryRetired(ArchiveEntryRetiredPayload {
                template_id: "00000000000000000000000000000001".into(),
                retired_at_ms: 1_700_000_000_000,
                retirement_reason: "degradation".into(),
            }),
            SsbPayload::TemplatePromoted(TemplatePromotedPayload {
                template_id: "00000000000000000000000000000001".into(),
                stage: "stage_50".into(),
                pass_rate: 0.94,
            }),
            SsbPayload::TemplateCandidateRejected(TemplateCandidateRejectedPayload {
                template_id: "00000000000000000000000000000001".into(),
                rejection_reason: "below_floor".into(),
            }),
            // E-5 (SCHEMA_VERSION 65): memory_facts bi-temporal lifecycle beacons.
            SsbPayload::MemoryFactExtracted(MemoryFactExtractedPayload {
                fact_id: "fact-test-extract".into(),
                fact_type: "preference".into(),
                scope: "project".into(),
                project_id: Some("proj-test".into()),
                user_id: "default".into(),
                key: "prefers_tracing_over_log".into(),
                trust_score: 0.9,
            }),
            SsbPayload::MemoryFactSuperseded(MemoryFactSupersededPayload {
                old_fact_id: "fact-test-old".into(),
                new_fact_id: "fact-test-new".into(),
                valid_to_ms: 1_700_000_000_000,
            }),
            // D-2 (SCHEMA_VERSION 66): PPM verifier beacons.
            SsbPayload::PpmScored(PpmScoredPayload {
                candidate_id: "cand-test-1".into(),
                verifier_id: "ppm-test".into(),
                mode: "math".into(),
                score_mean: 0.91,
                score_var: 0.0004,
                num_steps: 3,
                latency_ms: 412,
                cache_hit: false,
            }),
            SsbPayload::PpmGatingFailed(PpmGatingFailedPayload {
                candidate_id: "cand-test-2".into(),
                verifier_id: "ppm-test".into(),
                mode: "math".into(),
                score_mean: 0.42,
                threshold: 0.7,
                num_steps: 2,
            }),
            // E-1 (SCHEMA_VERSION 67): infinite-context chat illusion beacons.
            SsbPayload::ChatReflectionCompleted(ChatReflectionCompletedPayload {
                note_id: "note-test-reflect".into(),
                note_preview: "U: i prefer blake3...".into(),
                facts_extracted: 1,
                decay_rows_examined: 4,
                decay_rows_updated: 1,
                core_block_bytes: 256,
            }),
            SsbPayload::MemoryEvolutionRun(MemoryEvolutionRunPayload {
                phase: "end".into(),
                duration_ms: 1_240,
                notes_touched: 8,
            }),
            // E-1 dashboard slice (SCHEMA_VERSION 68): MemoryNotes / CoreBlock
            // producer + conflict beacons.
            SsbPayload::MemoryNoteInserted(MemoryNoteInsertedPayload {
                note_id: "note-test-insert".into(),
                scope: "session".into(),
                content_blake3: "0".repeat(64),
                link_count: 2,
            }),
            SsbPayload::MemoryNoteEvolved(MemoryNoteEvolvedPayload {
                note_id: "note-test-evolve".into(),
                old_content_blake3: "1".repeat(64),
                new_content_blake3: "2".repeat(64),
                links_added: 1,
                links_removed: 0,
            }),
            SsbPayload::MemoryConflictDetected(MemoryConflictDetectedPayload {
                fact_id_a: "fact-test-a".into(),
                fact_id_b: "fact-test-b".into(),
                conflict_kind: "duplicate".into(),
            }),
            SsbPayload::CoreBlockReplaced(CoreBlockReplacedPayload {
                user_id: "default".into(),
                old_blake3: "3".repeat(64),
                new_blake3: "4".repeat(64),
                bytes: 512,
            }),
            // D-3 (SCHEMA_VERSION 69): multi-verifier consensus aggregator.
            SsbPayload::VerifierConsensusPass(VerifierConsensusPassPayload {
                candidate_id: "cand-test-pass".into(),
                template_id: "tpl-test-1".into(),
                verdict: "pass".into(),
                verifier_count: 3,
                pass_count: 2,
                rule_id: "k_of_n".into(),
            }),
            SsbPayload::VerifierConsensusFail(VerifierConsensusFailPayload {
                candidate_id: "cand-test-fail".into(),
                template_id: "tpl-test-1".into(),
                failing_verifier_ids: vec!["v-intrinsic-1".into(), "v-det-1".into()],
                reason: "fail".into(),
            }),
            // D-4 (SCHEMA_VERSION 70): tripwire stack beacons.
            SsbPayload::TripwireFired(TripwireFiredPayload {
                event_id: "00112233445566778899aabbccddeeff".into(),
                tripwire_kind: "iqr_outlier".into(),
                source: "action".into(),
                candidate_id: "cand-test-trip".into(),
                observed_value: 42.0,
                threshold: 3.0,
                fired_at_ms: 1_700_000_000_000,
            }),
            SsbPayload::TemplateAutoReverted(TemplateAutoRevertedPayload {
                template_id: "tpl-test-auto-1".into(),
                parent_template_id: Some("tpl-test-parent".into()),
                retirement_reason: "degradation".into(),
                fired_tripwire_id: "00112233445566778899aabbccddeeff".into(),
            }),
            // D-4 GAP-2 (SCHEMA_VERSION 108): concurrent-detector tripwire row.
            SsbPayload::TripwireFiredConcurrent(TripwireFiredConcurrentPayload {
                candidate_id: "cand-test-trip".into(),
                tripwire_a: "iqr_outlier".into(),
                tripwire_b: "perplexity".into(),
                fired_within_ms: 0,
                fired_at_ms: 1_700_000_000_000,
            }),
            // D-19 (SCHEMA_VERSION 71): PRMBench adversarial-gaming benchmark beacon.
            SsbPayload::PrmBenchRunCompleted(PrmBenchRunCompletedPayload {
                total: 20,
                accuracy: 0.75,
                precision: 0.80,
                recall: 0.70,
                gating_verdict: "pass".into(),
                fixture_blake3: "a".repeat(64),
                category_count: 2,
            }),
            // D-5 (SCHEMA_VERSION 72): BAI Thompson sampling promotion round.
            SsbPayload::BaiArmUpdated(BaiArmUpdatedPayload {
                template_id: "11112222333344445555666677778888".into(),
                alpha: 9.0,
                beta: 2.0,
                n_pulls: 9,
                mean_estimate: 0.818,
                was_promoted: true,
            }),
            // D-5 closure (SCHEMA_VERSION 208): unified promotion-decision summary.
            SsbPayload::BaiPromotionDecision(BaiPromotionDecisionPayload {
                template_id: "11112222333344445555666677778888".into(),
                decision: "promoted".into(),
                stage: "stage_10".into(),
                evidence_pass: 8,
                evidence_total: 10,
                audit_id: "aabb1122ccdd3344eeff55667788aabb".into(),
            }),
            // D-6 (SCHEMA_VERSION 73): held-out task store lifecycle.
            SsbPayload::HeldOutTaskAdded(HeldOutTaskAddedPayload {
                task_id: "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa".into(),
                source: "synthetic".into(),
                content_blake3: "b".repeat(64),
                ngram_count: 12,
                ingested_ts_ms: 1_700_000_000_000,
                eligibility_window_ms: 0,
            }),
            SsbPayload::HeldOutTaskUsed(HeldOutTaskUsedPayload {
                task_id: "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa".into(),
                consumer_template_id: "11112222333344445555666677778888".into(),
                used_ts_ms: 1_700_000_005_000,
            }),
            // D-7 (SCHEMA_VERSION 74): cross-domain transfer canary beacons.
            SsbPayload::CanaryRunCompleted(CanaryRunCompletedPayload {
                report_id: "c".repeat(64),
                ts_ms: 1_700_000_010_000,
                overall_transfer_score: 0.04,
                decision_kind: "pass".into(),
                dominant_fail_family: String::new(),
            }),
            SsbPayload::CanaryFamilyEvaluated(CanaryFamilyEvaluatedPayload {
                report_id: "c".repeat(64),
                family: "code_fix".into(),
                n_evaluated: 5,
                pass_count: 4,
                fail_count: 1,
                transfer_score: 0.10,
                baseline_score: 0.70,
            }),
            // D-9 (SCHEMA_VERSION 75): L3 frozen-surface auditor beacons.
            SsbPayload::LockedKeysAuditRun(LockedKeysAuditRunPayload {
                audit_id: uuid::Uuid::nil(),
                ts_ms: 1_700_000_020_000,
                n_audited: 2,
                n_violations: 1,
                registry_blake3: "0".repeat(64),
                registry_path: "scripts/locked_keys_registry.toml".into(),
            }),
            SsbPayload::LockedKeysViolationDetected(LockedKeysViolationDetectedPayload {
                audit_id: uuid::Uuid::nil(),
                ts_ms: 1_700_000_020_001,
                attempted_edit: "crates/orchestrator/src/tripwire.rs".into(),
                attempted_lines: vec![0, u32::MAX],
                locked_key_path: "crates/orchestrator/src/tripwire.rs".into(),
                locked_key_signature_blake3: "0".repeat(64),
                reason: "tripwire detection per arxiv:1908.04734".into(),
            }),
            // E-46 (SCHEMA_VERSION 76): hierarchical decomposition tuning beacons.
            SsbPayload::TaskAdvanced(TaskAdvancedPayload {
                task_id: "task-1".into(),
                parent_id: String::new(),
                completion_pct: 50.0,
                status: "in_progress".into(),
                blocker: String::new(),
                ts_ms: 1_700_000_030_000,
            }),
            SsbPayload::AgentlessShortcutTaken(AgentlessShortcutTakenPayload {
                task_id: "task-1".into(),
                reason: "single_file_small".into(),
                ts_ms: 1_700_000_030_001,
            }),
            SsbPayload::GoalDriftFired(GoalDriftFiredPayload {
                initial_goal_blake3: "d".repeat(64),
                drift: 0.42,
                threshold: 0.3,
                step: 3,
                ts_ms: 1_700_000_030_002,
            }),
            // D-10 (SCHEMA_VERSION 77): mode-routing predicate beacon.
            SsbPayload::ClaimRouted(ClaimRoutedPayload {
                claim_id: "claim-d10-test".into(),
                claim_type: "factual".into(),
                mode: "constructive".into(),
                ts_ms: 1_700_000_040_000,
                ctx_summary_blake3: "e".repeat(64),
            }),
            // D-21 (SCHEMA_VERSION 78): mode-routing coverage benchmark beacon.
            SsbPayload::ModeRoutingBenchRunCompleted(ModeRoutingBenchRunCompletedPayload {
                total: 25,
                accuracy: 0.92,
                f1_macro: 0.88,
                gating_verdict: "pass".into(),
                fixture_blake3: "f".repeat(64),
                fixture_row_count: 25,
            }),
            // D-11 (SCHEMA_VERSION 79): constructive-checker registry beacon.
            SsbPayload::CheckerResultEmitted(CheckerResultEmittedPayload {
                checker_id: "regex-default".into(),
                checker_kind: "regex_match".into(),
                result_kind: "pass".into(),
                claim_id: "claim-d11-test".into(),
                ts_ms: 1_700_000_050_000,
                reason: String::new(),
            }),
            // D-13 (SCHEMA_VERSION 80): adversarial-verification beacons.
            SsbPayload::AdversaryGenerated(AdversaryGeneratedPayload {
                counter_id: "ctr-d13-test".into(),
                claim_id: "claim-d13-test".into(),
                severity: 0.75,
                model_family: "anthropic".into(),
            }),
            SsbPayload::ConsensusReevaluatedWithCounter(
                ConsensusReevaluatedWithCounterPayload {
                    claim_id: "claim-d13-test".into(),
                    original_verdict: "pass".into(),
                    post_counter_verdict: "fail".into(),
                },
            ),
            // D-12 (SCHEMA_VERSION 81): OperationalizationCheck beacon.
            SsbPayload::BabyStepDispatched(BabyStepDispatchedPayload {
                step_id: "bs-d12-test".into(),
                parent_task_id: "task-d12-test".into(),
                checker_kinds_used: vec!["numeric_range".into()],
                outcome: "pass".into(),
                ts_ms: 1_700_000_060_000,
            }),
            // D-16 (SCHEMA_VERSION 82): UniCR conformal-refusal beacons.
            SsbPayload::ConformalTauComputed(ConformalTauComputedPayload {
                tau: 0.8,
                epsilon: 0.1,
                corpus_size: 40,
                source: "held_out".into(),
                ts_ms: 1_700_000_070_000,
                calibration_id: None,
            }),
            SsbPayload::ConformalRefusalDecided(ConformalRefusalDecidedPayload {
                claim_id: "claim-d16-test".into(),
                claim_score: 0.05,
                tau: 0.8,
                decision: "abstain".into(),
                confidence: 0.0,
                ts_ms: 1_700_000_071_000,
                reason: Some("low_evidence".into()),
                calibration_id: None,
            }),
            // D-15 (SCHEMA_VERSION 83): discovery-vs-search NoveltyClass beacon.
            // FINAL TRANCHE B4: optional candidate_status field added (None
            // here exercises the additive-default path against the on-disk
            // wire form).
            SsbPayload::NoveltyClassified(NoveltyClassifiedPayload {
                template_id: "tpl-d15-test".into(),
                class: "discovery_candidate".into(),
                dkg_match_count: 0,
                near_duplicate_threshold: 0.05,
                ts_ms: 1_700_000_080_000,
                candidate_status: None,
            }),
            // D-18 (SCHEMA_VERSION 84): typed-ObservableEffect predicate beacon.
            SsbPayload::ObservableEffectVerified(ObservableEffectVerifiedPayload {
                effect_id: "eff-d18-test".into(),
                trigger_kind: "test_command".into(),
                observation_kind: "string_match".into(),
                sandbox_tier: "no_sandbox".into(),
                outcome: "pass".into(),
                confound_flags: String::new(),
                elapsed_us: 1234,
                ts_ms: 1_700_000_090_000,
            }),
            // D-17 (SCHEMA_VERSION 85): honest-refusal SLA monitoring beacons.
            SsbPayload::RefusalRateSnapshot(RefusalRateSnapshotPayload {
                refusal_rate: 0.5,
                window_total: 100,
                abstain_count: 50,
                health_band: "above".into(),
                ts_ms: 1_700_000_100_000,
            }),
            SsbPayload::RefusalSlaBreach(RefusalSlaBreachPayload {
                rate: 0.5,
                threshold: 0.2,
                breach_kind: "above".into(),
                ts_ms: 1_700_000_101_000,
            }),
            // D-22 (SCHEMA_VERSION 86): conformal-calibration validation beacon.
            SsbPayload::ConformalValidationCompleted(ConformalValidationCompletedPayload {
                tau: 0.4,
                epsilon: 0.1,
                empirical_refusal_rate: 0.08,
                gating_verdict: "pass".into(),
                fixture_blake3: "f".repeat(64),
                total: 36,
                ts_ms: 1_700_000_110_000,
            }),
            // D-20 (SCHEMA_VERSION 87): ARC-AGI-2 OOD probe gating beacon.
            SsbPayload::ArcBenchmarkCompleted(ARCBenchmarkCompletedPayload {
                total: 24,
                honestly_evaluated_count: 11,
                gating_verdict: "honestly_evaluated".into(),
                fixture_blake3: "a".repeat(64),
                dominant_mode: "operational_deferred".into(),
                dominant_novelty_class: "discovery_candidate".into(),
                ts_ms: 1_700_000_120_000,
            }),
            SsbPayload::ToolGated(ToolGateEventPayload {
                tool_name: "fs.read".into(),
                fsm_state: "planning".into(),
                prompt_tokens_saved: 42,
            }),
            SsbPayload::ToolEager(ToolGateEventPayload {
                tool_name: "knowledge.search".into(),
                fsm_state: "planning".into(),
                prompt_tokens_saved: 0,
            }),
            SsbPayload::ToolLazyResolved(ToolGateEventPayload {
                tool_name: "fs.edit".into(),
                fsm_state: "executing".into(),
                prompt_tokens_saved: 120,
            }),
            SsbPayload::ToolDenied(ToolGateEventPayload {
                tool_name: "git.commit".into(),
                fsm_state: "planning".into(),
                prompt_tokens_saved: 0,
            }),
            SsbPayload::QuantBackendSelected(QuantBackendSelectedPayload {
                backend: "awq".into(),
                kv_dtype: "fp8_e5m2".into(),
                compute_capability: 86,
                marlin_enabled: true,
            }),
            SsbPayload::KvCacheDtypeSoftwareSim(KvCacheDtypeSoftwareSimPayload {
                kv_dtype: "fp8_e5m2".into(),
                throughput_cost_pct: 5.0,
                compute_capability: 86,
            }),
            SsbPayload::AwqCheckpointBuilt(AwqCheckpointBuiltPayload {
                model_path: "/models/Qwen2.5-Coder-7B".into(),
                calibration_size: 256,
                duration_s: 7200.0,
                output_path: "/models/Qwen2.5-Coder-7B-AWQ".into(),
            }),
            SsbPayload::CliMcpDispatch(CliMcpDispatchPayload {
                tool_name: "fs.read".into(),
                port: 8200,
            }),
            SsbPayload::CliMcpServerStarted(CliMcpServerStartedPayload { port: 8200 }),
            SsbPayload::CliMcpServerStopped(CliMcpServerStoppedPayload { port: 8200 }),
            SsbPayload::FirecrackerRestoreSlow(FirecrackerRestoreSlowPayload {
                agent_id: "test-agent".to_string(),
                actual_ms: 350,
                budget_ms: 200,
            }),
            SsbPayload::SnapshotUniquenessRegenerated(SnapshotUniquenessRegeneratedPayload {
                agent_id: "test-agent".to_string(),
                restore_count: 3,
            }),
            SsbPayload::SnapshotEvicted(SnapshotEvictedPayload {
                agent_id: "test-agent".to_string(),
                reason: "lru_count".to_string(),
                snapshot_path: "/tmp/test.snap".to_string(),
            }),
            SsbPayload::SandboxBackendSelected(SandboxBackendSelectedPayload {
                agent_id: "test-agent".to_string(),
                backend: "Firecracker".to_string(),
            }),
            SsbPayload::AgentForkedFromSnapshot(AgentForkedFromSnapshotPayload {
                agent_id: "test-agent".to_string(),
                baby_step_id: 1,
                snapshot_path: "/tmp/test.snap".to_string(),
            }),
            SsbPayload::ResearcherNetEgressBlocked(ResearcherNetEgressBlockedPayload {
                agent_id: "test-agent".to_string(),
                destination: "evil.example.com".to_string(),
                policy_action: "REJECT".to_string(),
            }),
            SsbPayload::McpSchemaPinViolation(McpSchemaPinViolationPayload {
                server_url: "http://localhost:8200".to_string(),
                pinned_hash: "abc123def456".to_string(),
                observed_hash: "def456abc123".to_string(),
                pinned_at_ms: 1_700_000_000_000,
            }),
            SsbPayload::McpSanitizationRuleApplied(McpSanitizationRuleAppliedPayload {
                server_url: "http://localhost:8200".to_string(),
                tool_name: "search".to_string(),
                rule_id: "shell_prefix".to_string(),
                replacement_count: 1,
            }),
            SsbPayload::McpBudgetEnforced(McpBudgetEnforcedPayload {
                server_url: "http://localhost:8200".to_string(),
                dropped_tools: vec!["tool_a".to_string(), "tool_b".to_string()],
                kept_tokens: 4800,
                budget_tokens: 5000,
            }),
            SsbPayload::McpLegacySseTransport(McpLegacySseTransportPayload {
                server_url: "http://localhost:8200".to_string(),
                deprecation_notice: "SSE transport deprecated per MCP spec 2025-11-25".to_string(),
            }),
            SsbPayload::McpLlmGuardBlocked(McpLlmGuardBlockedPayload {
                server_url: "http://localhost:8200".to_string(),
                tool_name: "dangerous_tool".to_string(),
                reason: "shell command detected in args".to_string(),
                guard_latency_ms: 42,
            }),
            SsbPayload::AnthropicGatewayDetected(AnthropicGatewayDetectedPayload {
                base_url: "https://openrouter.ai/api/v1".to_string(),
                detected_gateway: "openrouter".to_string(),
                latency_overhead_ms_estimate: 50,
                prompt_cache_broken: true,
            }),
            SsbPayload::MemoryTierAssigned(MemoryTierAssignedPayload {
                fact_id: "fact-test-001".to_string(),
                trust_tier: "T1".to_string(),
                provenance_source: "src/main.rs:42".to_string(),
            }),
            SsbPayload::MemoryQueryFiltered(MemoryQueryFilteredPayload {
                query_type: "destructive_command_intent".to_string(),
                excluded_count: 3,
            }),
            SsbPayload::SafetyCriticalBlocked(SafetyCriticalBlockedPayload {
                fact_id: "inject-t2".to_string(),
                trust_tier: "T2".to_string(),
                query_type: "auth_decision".to_string(),
            }),
            // E-30: public-incident hardening beacons
            SsbPayload::ArgvDestructiveDenied(ArgvDestructiveDeniedPayload {
                argv_hash: "abc123".to_string(),
                reason: "rm -rf with home-dir prefix".to_string(),
                citation: "Claude Code rm -rf ~/ incident 2025-05".to_string(),
            }),
            SsbPayload::ArgvApprovalRequired(ArgvApprovalRequiredPayload {
                argv_hash: "def456".to_string(),
                reason: "terraform destroy requires human approval".to_string(),
                citation: "DataTalks terraform destroy incident 2025-06".to_string(),
            }),
            SsbPayload::TrustDialogHookDeferred(TrustDialogHookDeferredPayload {
                settings_path: "/project/.claude/settings.json".to_string(),
                cve: "CVE-2025-59536".to_string(),
            }),
            SsbPayload::TrustDialogEnvDeferred(TrustDialogEnvDeferredPayload {
                env_var: "ANTHROPIC_BASE_URL".to_string(),
                cve: "CVE-2026-21852".to_string(),
            }),
            SsbPayload::McpPinVersionRefused(McpPinVersionRefusedPayload {
                server_name: "postmark-mcp".to_string(),
                presented_version: "1.0.17".to_string(),
                pinned_version: "1.0.15".to_string(),
                refusal_reason: "version_float".to_string(),
            }),
            SsbPayload::AuditorLineageCollision(AuditorLineageCollisionPayload {
                auditor_agent_id: "agt-audit-01".to_string(),
                executor_agent_id: "agt-exec-01".to_string(),
                lineage_group: "session-xyz".to_string(),
                action_taken: "blocked".to_string(),
            }),
            SsbPayload::UntrustedMemoryWriteBlocked(UntrustedMemoryWriteBlockedPayload {
                source_label: "ExternalTool".to_string(),
                write_summary: "attempted write of 512 bytes from external tool output".to_string(),
            }),
            SsbPayload::IncidentRecordCreated(IncidentRecordCreatedPayload {
                incident_id: "inc-replit-2025-07".to_string(),
                title: "Replit DROP TABLE dropped production database".to_string(),
                incident_date: "2025-07-01".to_string(),
                source_url: "incidentdatabase.ai/cite/1152".to_string(),
            }),
            // E-32: context-window extension beacons (212 -> 216)
            SsbPayload::FitmCalibrationApplied(FitmCalibrationAppliedPayload {
                prompt_len: 12_288,
                dummy_len: 50,
                applied_ms: 142.5,
            }),
            SsbPayload::DeepModeActivated(DeepModeActivatedPayload {
                model_id: "Qwen/Qwen2.5-7B-Instruct-1M".to_string(),
                ctx_cap: 64_000,
                vram_gb: 24.0,
            }),
            SsbPayload::DeepModeEscalationChosen(DeepModeEscalationChosenPayload {
                reason: "cross-file synthesis: 5 files, no retrieval-local answer".to_string(),
                files_referenced: 5,
            }),
            SsbPayload::YarnRefused(YarnRefusedPayload {
                model_name: "Qwen2.5-Coder-7B".to_string(),
                reason: "YaRN rejected per E-32: 15-20% quality drop (vllm issue 18728)".to_string(),
            }),
            // E-51 (SCHEMA_VERSION 96): edge-inference beacons.
            SsbPayload::MlxSubprocessSpawned(MlxSubprocessSpawnedPayload {
                pid: 22222,
                argv_redacted: vec!["vllm-mlx".into()],
            }),
            SsbPayload::MlxSubprocessKilled(MlxSubprocessKilledPayload {
                pid: 22222,
                exit_code: Some(0),
                reason: "normal exit".into(),
            }),
            SsbPayload::LlamaCppSubprocessSpawned(LlamaCppSubprocessSpawnedPayload {
                pid: 33333,
                ep: "cpu".into(),
                argv_redacted: vec!["llama-server".into()],
            }),
            SsbPayload::LlamaCppSubprocessKilled(LlamaCppSubprocessKilledPayload {
                pid: 33333,
                exit_code: Some(0),
                reason: "normal exit".into(),
            }),
            SsbPayload::MlxApplePlatformDetected(MlxApplePlatformDetectedPayload {
                chip_family: "M5".into(),
                gpu_cores: 40,
            }),
            SsbPayload::LlamaCppEpDetected(LlamaCppEpDetectedPayload { ep: "cpu".into() }),
            SsbPayload::RuntimeChainBuilt(RuntimeChainBuiltPayload {
                chain: vec![elai_providers::runtime::RuntimeBackend::Sidecar],
            }),
            // E-17 SP2 (SCHEMA_VERSION 97): RAG retrieval-quality beacons.
            SsbPayload::DeepPoolCapped(DeepPoolCappedPayload {
                capped_count: 150,
                max_allowed: 100,
                stage_count: 5,
            }),
            SsbPayload::RrfQualityGateFired(RrfQualityGateFiredPayload {
                dropped_count: 12,
                floor: 0.005,
                total_before: 80,
            }),
            SsbPayload::HydeSkippedCodeQuery(HydeSkippedCodeQueryPayload {
                query_class: "code_symbol".into(),
                word_count: 3,
                code_signal_present: true,
            }),
            SsbPayload::RrfComponentWeights(RrfComponentWeightsPayload {
                bm25_weight: Some(1.0),
                code_vec_weight: Some(1.0),
                text_vec_weight: Some(0.8),
            }),
            // B-3 (SCHEMA_VERSION 98): AntiStalenessDrawer + RulesDrawer beacons.
            SsbPayload::DepAuditCompleted(DepAuditCompletedPayload {
                findings: 0,
                packages_scanned: 142,
                duration_ms: 850,
                completed_at_ms: 1_700_000_000_000,
            }),
            SsbPayload::BaselineRunCompleted(BaselineRunCompletedPayload {
                checks_run: 12,
                checks_passed: 12,
                checks_failed: 0,
                duration_ms: 4_200,
                completed_at_ms: 1_700_000_000_000,
            }),
            SsbPayload::DeterministicAuditRun(DeterministicAuditRunPayload {
                verdict: "passed".into(),
                gates_evaluated: 8,
                gates_fired: 0,
                completed_at_ms: 1_700_000_000_000,
            }),
            SsbPayload::RuleTransition(RuleTransitionPayload {
                rule_id: "rule-test".into(),
                from_status: "CANDIDATE".into(),
                to_status: "SHADOW".into(),
                reason: "promotion gate passed".into(),
                timestamp_ms: 1_700_000_000_000,
            }),
            // B-4 (SCHEMA_VERSION 99): canary drift snapshot row.
            SsbPayload::CanaryDrift(CanaryDriftPayload {
                similarity: 0.9123,
                drift_threshold: 0.85,
                passed: true,
                reference_present: true,
                ts_ms: 1_700_000_000_000,
            }),
            // B-7 (SCHEMA_VERSION 100): pipeline factory producer wiring beacons.
            SsbPayload::TreePositionChanged(TreePositionChangedPayload {
                current_node: "concept-0/phase-1/step-2/baby-3".into(),
                visited: vec![1, 2, 3],
                blocked: vec![],
                skipped: vec![],
            }),
            SsbPayload::FailureRecorded(FailureRecordedPayload {
                baby_step_id: 3,
                step_id: 2,
                phase_id: 1,
                concept_id: 0,
                reason: "tooling unavailable".into(),
                attempts: 2,
            }),
            SsbPayload::PackageRefVerified(PackageRefVerifiedPayload {
                name: "blake3".into(),
                registry: "crates.io".into(),
                version: Some("1.8.0".into()),
                verified: true,
            }),
            SsbPayload::TechStackDecided(TechStackDecidedPayload {
                component: "embedding".into(),
                chosen: "graphcodebert".into(),
                alternatives: vec!["e5-small".into(), "bge-small".into()],
                source: "research".into(),
            }),
            // B-19 (SCHEMA_VERSION 101): per-agent-execution DAIRA marker
            // aggregation row.
            {
                let mut phase_counts: std::collections::BTreeMap<String, u32> =
                    std::collections::BTreeMap::new();
                phase_counts.insert("HookPre".into(), 1);
                phase_counts.insert("HookPost".into(), 1);
                SsbPayload::MarkerReport(MarkerReportPayload {
                    agent_id: "task_runner".into(),
                    step_id: "task_1".into(),
                    phase_counts,
                    total_events: 2,
                    window_us: 1000,
                })
            },
            // B-25 (SCHEMA_VERSION 102): Phase 4J Empire integration. 5 payloads.
            SsbPayload::EmpireConnected(EmpireConnectedPayload {
                brain_id: "elai_cc".into(),
                protocol_version: 1,
            }),
            SsbPayload::EmpireDisconnected(EmpireDisconnectedPayload {
                brain_id: "elai_cc".into(),
                reason: "explicit_shutdown".into(),
            }),
            SsbPayload::EmpireEventReceived(EmpireEventReceivedPayload {
                source: "factory".into(),
                event_type: "deploy".into(),
                payload: serde_json::json!({"target": "neuron_alpha"}),
            }),
            SsbPayload::EmpireCapabilityInvoked(EmpireCapabilityInvokedPayload {
                capability: "task_submit".into(),
                caller: "factory".into(),
                allowed: true,
            }),
            SsbPayload::EmpireRepairProposed(EmpireRepairProposedPayload {
                issue: "circular_dep".into(),
                target_path: "workspace/factory-brain/core/bus.py".into(),
                fix_kind: "refactor_shared_module".into(),
            }),
            // C-5 follow-on (SCHEMA_VERSION 103): cloud-mode hardening trio.
            SsbPayload::RouteBlocked(RouteBlockedPayload {
                reason: "daily_cap_exceeded".into(),
                spent_usd: 5.10,
                cap_usd: 5.0,
                cap_period: "daily".into(),
                baby_step_id: 1,
                ts_ms: 1_700_000_200_000,
            }),
            SsbPayload::WarnLimitReached(WarnLimitReachedPayload {
                spent_usd: 4.05,
                cap_usd: 5.0,
                warn_at_percent: 80,
                pct_of_cap: 81.0,
                baby_step_id: 1,
                ts_ms: 1_700_000_201_000,
            }),
            SsbPayload::CostEstimate(CostEstimatePayload {
                low_usd: 0.084,
                high_usd: 0.156,
                model: "claude-sonnet-4-6".into(),
                step_count: 3,
                heuristic_source: "ungrounded_heuristic".into(),
                ts_ms: 1_700_000_202_000,
            }),
            // C-36 (SCHEMA_VERSION 104): MuCoCo mutation operator dispatch.
            SsbPayload::MucocoMutationApplied(MucocoMutationAppliedPayload {
                operator: "demorgan".into(),
                language: "python".into(),
                source_hash: 0xc36c_0036_c36c_0036,
                applied: true,
                skip_reason: None,
                timestamp_ms: 1_700_000_203_000,
            }),
            // D-3 closure (SCHEMA_VERSION 105): PpmUnavailable +
            // VerifierDisagreement + CrossContextDispatched.
            SsbPayload::PpmUnavailable(PpmUnavailablePayload {
                candidate_id: "cand-d3-unavail".into(),
                verifier_id: "ppm-test".into(),
                reason: "health_check_failed".into(),
            }),
            SsbPayload::VerifierDisagreement(VerifierDisagreementPayload {
                candidate_id: "cand-d3-disagree".into(),
                template_id: "tpl-d3-disagree".into(),
                verifier_a_kind: "ppm".into(),
                verifier_b_kind: "deterministic".into(),
                score_delta: 0.9,
            }),
            SsbPayload::CrossContextDispatched(CrossContextDispatchedPayload {
                claim_id: "claim-d3-cc".into(),
                context_a_hash: "a".repeat(64),
                context_b_hash: "b".repeat(64),
                jaccard: 0.85,
                agreement: "agree".into(),
            }),
            // C-38 (SCHEMA_VERSION 107): pagerank + dkg rebuild lifecycle.
            SsbPayload::PagerankRun(PagerankRunPayload {
                workspace_root_hash: 0xc380_0001_c380_0001,
                node_count: 50,
                top_node_hash: 0xc380_0002_c380_0002,
                elapsed_ms: 12,
            }),
            SsbPayload::DkgRebuildStarted(DkgRebuildStartedPayload {
                workspace_root_hash: 0xc380_0003_c380_0003,
                trigger_reason: "missing".into(),
            }),
            SsbPayload::DkgRebuildComplete(DkgRebuildCompletePayload {
                workspace_root_hash: 0xc380_0003_c380_0003,
                success: true,
                duration_ms: 1234,
                node_count: 50,
            }),
            // C-40 (SCHEMA_VERSION 109): GitHub PR adapter producer surface.
            SsbPayload::GithubPrOpened(GithubPrOpenedPayload {
                pr_number: 42,
                pr_url_hash: 0xc400_0001_c400_0001,
                ts_ms: 1_700_000_000_000,
            }),
            SsbPayload::GithubPrStatusFetched(GithubPrStatusFetchedPayload {
                pr_number: 42,
                ci_status_hash: 0xc400_0002_c400_0002,
                review_state_hash: 0xc400_0003_c400_0003,
                ts_ms: 1_700_000_000_000,
            }),
            SsbPayload::GithubPrClosed(GithubPrClosedPayload {
                pr_number: 42,
                ts_ms: 1_700_000_000_000,
            }),
            SsbPayload::GithubPrMerged(GithubPrMergedPayload {
                pr_number: 42,
                merge_sha_hash: 0xc400_0004_c400_0004,
                ts_ms: 1_700_000_000_000,
            }),
            // E-41 (SCHEMA_VERSION 135): GitHub PR review surface.
            SsbPayload::PrReviewComposed(PrReviewComposedPayload {
                pr_number_hash: 0xe410_0001_e410_0001,
                comment_kind_hash: 0xe410_0002_e410_0002,
                ts_ms: 1_700_000_000_000,
            }),
            SsbPayload::PrCommentWatched(PrCommentWatchedPayload {
                pr_number_hash: 0xe410_0003_e410_0003,
                new_comments_count: 3,
                ts_ms: 1_700_000_000_000,
            }),
            SsbPayload::PrPackageVerifyResult(PrPackageVerifyResultPayload {
                pr_number_hash: 0xe410_0004_e410_0004,
                slopsquat_count: 1,
                ts_ms: 1_700_000_000_000,
            }),
            SsbPayload::PrSemgrepGateResult(PrSemgrepGateResultPayload {
                pr_number_hash: 0xe410_0005_e410_0005,
                findings_count: 2,
                severity_max_hash: 0xe410_0006_e410_0006,
                ts_ms: 1_700_000_000_000,
            }),
            SsbPayload::PrAutoMergeBlocked(PrAutoMergeBlockedPayload {
                pr_number_hash: 0xe410_0007_e410_0007,
                block_reason_hash: 0xe410_0008_e410_0008,
                ts_ms: 1_700_000_000_000,
            }),
            // D-5 closure (SCHEMA_VERSION 110): Thompson-sample beacon.
            SsbPayload::ThompsonSampled(ThompsonSampledPayload {
                arm_id: "11112222333344445555666677778888".into(),
                sampled_value: 0.812,
                alpha: 9.0,
                beta: 2.0,
            }),
            // C-41 (SCHEMA_VERSION 111): skills-driven workflow registry.
            SsbPayload::SkillTriggered(SkillTriggeredPayload {
                skill_name_hash: 0xc410_0001_c410_0001,
                score: 0.875,
                top_k_rank: 1,
                ts_ms: 1_700_000_000_000,
            }),
            SsbPayload::SkillExecuted(SkillExecutedPayload {
                skill_name_hash: 0xc410_0001_c410_0001,
                exit_ok: true,
                duration_ms: 42,
                ts_ms: 1_700_000_000_042,
            }),
            // D-6 closure (SCHEMA_VERSION 112): held-out promotion-gate +
            // refresh-event beacons.
            SsbPayload::HeldOutPromotionGate(HeldOutPromotionGatePayload {
                task_id: "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa".into(),
                allow: true,
                delta: -0.02,
                regression_threshold: 0.05,
                passes: 47,
                fails: 3,
                reason: "delta=-0.0200 pp (allowed: >= -0.0500 pp regression threshold)"
                    .into(),
            }),
            SsbPayload::HeldOutRefresh(HeldOutRefreshPayload {
                source_label: "arxiv_repro".into(),
                added_count: 3,
                retired_count: 1,
                ts_ms: 1_700_000_000_500,
            }),
            // C-42 (SCHEMA_VERSION 113): Semantic Gateway L2 RBAC beacons.
            SsbPayload::GatewayPolicyAllowed(GatewayPolicyAllowedPayload {
                role_hash: 0xc420_0001_c420_0001,
                tool_hash: 0xc420_0002_c420_0002,
                deny_reason_label: "allow".into(),
                ts_ms: 1_700_000_000_000,
            }),
            SsbPayload::GatewayPolicyDenied(GatewayPolicyDeniedPayload {
                role_hash: 0xc420_0001_c420_0001,
                tool_hash: 0xc420_0002_c420_0002,
                deny_reason_label: "authz".into(),
                deny_reason_hash: 0xc420_0003_c420_0003,
                ts_ms: 1_700_000_000_000,
            }),
            SsbPayload::GatewayHitlPending(GatewayHitlPendingPayload {
                role_hash: 0xc420_0001_c420_0001,
                tool_hash: 0xc420_0002_c420_0002,
                challenge_hash: 0,
                ts_ms: 1_700_000_000_000,
            }),
            // D-7 closure (SCHEMA_VERSION 114): cross-domain canary obs.
            SsbPayload::CrossDomainRegression(CrossDomainRegressionPayload {
                source_domain: crate::reasoning_template::Domain::Code,
                regressed_domain: crate::reasoning_template::Domain::Math,
                delta_pct: 7.0,
                threshold_pct: 5.0,
                ts_ms: 1_700_000_000_000,
            }),
            SsbPayload::CanaryFamilyPass(CanaryFamilyPassPayload {
                family: "code_fix".into(),
                score_pct: 82.0,
                sample_count: 10,
                ts_ms: 1_700_000_000_000,
            }),
            SsbPayload::CanaryInsufficientData(CanaryInsufficientDataPayload {
                family: "tool_use".into(),
                sample_count: 3,
                min_required: 5,
                ts_ms: 1_700_000_000_000,
            }),
            // D-8 closure (SCHEMA_VERSION 115): cryptographic archive verify.
            SsbPayload::ArchiveVerifyResult(ArchiveVerifyResultPayload {
                audit_id: "test-audit".into(),
                total_entries: 0,
                verified: true,
                error_count: 0,
                errors: Vec::new(),
                ts_ms: 1_700_000_000_000,
            }),
            // C-43 closure (SCHEMA_VERSION 116): Phase 4H scope-freeze.
            SsbPayload::ScopeFreezeRatified(ScopeFreezeRatifiedPayload {
                manifest_hash: 0xDEAD_BEEF_CAFE_F00D,
                ratifier_id_hash: 0x1234_5678_9ABC_DEF0,
                item_count: 9,
                ts_ms: 1_748_044_800_000,
            }),
            SsbPayload::ScopeFreezeViolated(ScopeFreezeViolatedPayload {
                item_path_hash: 0xFEED_FACE_DEAD_BEEF,
                frozen_since_ms: 1_748_044_800_000,
                modified_at_ms: 1_748_044_900_000,
                ts_ms: 1_748_044_900_000,
            }),
            // C-44 (SCHEMA_VERSION 117): Composer 2 paid-backend cost accrual.
            SsbPayload::BackendCostAccrued(BackendCostAccruedPayload {
                backend_name_hash: 0xCAFE_BABE_DEAD_BEEF,
                role_name_hash: 0x0123_4567_89AB_CDEF,
                tokens_in: 1_024,
                tokens_out: 256,
                est_cost_micros: 1_152,
                ts_ms: 1_748_044_900_000,
            }),
            // C-45 (SCHEMA_VERSION 120): CANTANTE credit-attribution beacons.
            SsbPayload::AgentRoleDropped(AgentRoleDroppedPayload {
                role_name_hash: 0xAAAA_BBBB_CCCC_DDDD,
                ts_ms: 1_748_044_900_000,
            }),
            SsbPayload::CreditAttributionRun(CreditAttributionRunPayload {
                run_id_hash: 0x1111_2222_3333_4444,
                dropped_role_name_hash: 0xAAAA_BBBB_CCCC_DDDD,
                k_rollouts: 9,
                g_partition: 3,
                ts_ms: 1_748_044_900_000,
            }),
            SsbPayload::CreditAttributionResult(CreditAttributionResultPayload {
                run_id_hash: 0x1111_2222_3333_4444,
                per_role_scores_hash: 0x5555_6666_7777_8888,
                total_roles: 8,
                ts_ms: 1_748_044_900_000,
            }),
            // C-46 (SCHEMA_VERSION 121): Uno-Orchestra selective-delegation beacons.
            SsbPayload::SelectiveDelegationDecision(SelectiveDelegationDecisionPayload {
                task_id_hash: 0x9999_8888_7777_6666,
                difficulty: 0.42,
                alpha: 0.1,
                dispatched_count: 3,
                ts_ms: 1_748_044_900_000,
            }),
            SsbPayload::SelectiveDelegationEscalated(SelectiveDelegationEscalatedPayload {
                task_id_hash: 0x9999_8888_7777_6666,
                reason_hash: 0xABCD_1234_5678_9ABC,
                escalated_workers: 2,
                ts_ms: 1_748_044_900_000,
            }),
            SsbPayload::SelectiveDelegationCostSaved(SelectiveDelegationCostSavedPayload {
                task_id_hash: 0x9999_8888_7777_6666,
                skipped_count: 5,
                est_savings_micros: 1_500_000,
                ts_ms: 1_748_044_900_000,
            }),
            // C-47 (SCHEMA_VERSION 122): Case-Based Calibration of Tool-Use
            // beacons. ToolCallLogged + ToolCalibrationHintEmitted +
            // ToolCalibrationDriftDetected. Paper-grounded against
            // arXiv:2605.15041 (CAST).
            SsbPayload::ToolCallLogged(ToolCallLoggedPayload {
                tool_name_hash: 0x1111_2222_3333_4444,
                call_shape_hash: 0xAAAA_BBBB_CCCC_DDDD,
                response_kind_hash: 0x5555_6666_7777_8888,
                post_call_audit_ok: true,
                ts_ms: 1_748_044_900_000,
            }),
            SsbPayload::ToolCalibrationHintEmitted(ToolCalibrationHintEmittedPayload {
                tool_name_hash: 0x1111_2222_3333_4444,
                call_shape_hash: 0xAAAA_BBBB_CCCC_DDDD,
                hint_text_hash: 0xDEAD_BEEF_FEED_FACE,
                ts_ms: 1_748_044_900_000,
            }),
            SsbPayload::ToolCalibrationDriftDetected(ToolCalibrationDriftDetectedPayload {
                tool_name_hash: 0x1111_2222_3333_4444,
                error_rate: 0.42,
                total_calls: 17,
                ts_ms: 1_748_044_900_000,
            }),
            // C-49 (SCHEMA_VERSION 123): iterative-distillation experience.
            // PromptPolicyDistillationRun + PromptFragmentSelected +
            // PromptFragmentDiscarded. ELAI heuristic adaptation of
            // arXiv:2605.14443 (RL prompter is RUNPOD-PENDING).
            SsbPayload::PromptPolicyDistillationRun(PromptPolicyDistillationRunPayload {
                skill_name_hash: 0xC0DE_C0DE_C0DE_C0DE,
                trace_count: 7,
                fragments_emitted: 3,
                ts_ms: 1_748_044_900_000,
            }),
            SsbPayload::PromptFragmentSelected(PromptFragmentSelectedPayload {
                skill_name_hash: 0xC0DE_C0DE_C0DE_C0DE,
                fragment_hash: 0xABAD_1DEA_ABAD_1DEA,
                success_rate: 0.83,
                freq: 5,
                ts_ms: 1_748_044_900_000,
            }),
            SsbPayload::PromptFragmentDiscarded(PromptFragmentDiscardedPayload {
                skill_name_hash: 0xC0DE_C0DE_C0DE_C0DE,
                fragment_hash: 0xDEAD_BEEF_DEAD_BEEF,
                reason_hash: 0xFEED_BABE_FEED_BABE,
                ts_ms: 1_748_044_900_000,
            }),
            // D-14 (SCHEMA_VERSION 124): sycophancy probe row pair.
            // ELAI structural-defense adaptation of arXiv:2310.13548
            // (Sharma et al 2024). Defensive flag-only — preference
            // models do NOT reliably fix sycophancy (§4).
            SsbPayload::SycophancyProbeDispatched(SycophancyProbeDispatchedPayload {
                claim_id: "claim-d14-test".into(),
                probe_kind: "are_you_sure".into(),
                user_message: "is the capital of France Paris?".into(),
            }),
            SsbPayload::SycophancyFlag(SycophancyFlagPayload {
                claim_id: "claim-d14-test".into(),
                flip_detected: true,
                delta: 0.62,
                verdict: "divergent_sycophancy_flag".into(),
            }),
            // D-23 (SCHEMA_VERSION 142): sycophancy detection benchmark.
            SsbPayload::SycophancyBenchRunCompleted(SycophancyBenchRunCompletedPayload {
                tp: 70,
                fp: 5,
                tn: 20,
                fn_: 3,
                inconclusive: 2,
                detection_rate: 0.9589,
                fp_rate: 0.20,
                verdict: "pass".into(),
                total_fixtures: 100,
                fixture_blake3: "d".repeat(64),
                runner_name: "sycophancy_bench_v1".into(),
            }),
            // D-24 (SCHEMA_VERSION 143): cross-domain canary regression
            // benchmark. NanoFlux arXiv:2509.23252 provides empirical
            // motivation only; no canary mechanism in the paper.
            SsbPayload::CanaryBenchRunCompleted(CanaryBenchRunCompletedPayload {
                true_block: 20,
                false_block: 3,
                true_pass: 77,
                false_pass: 0,
                harmful_blocked: 20,
                helpful_blocked: 2,
                neutral_blocked: 1,
                total_fixtures: 100,
                verdict: "pass".into(),
                fixture_blake3: "e".repeat(64),
                runner_name: "canary_bench_v1".into(),
            }),
            // D-25 (SCHEMA_VERSION 144): determinism + archive tamper-suite
            // benchmark. arxiv:2507.05619 (T1) provides integrity-checksum
            // motivation only; no tamper-suite mechanism in the paper.
            SsbPayload::DeterminismTamperSuiteCompleted(
                DeterminismTamperSuiteCompletedPayload {
                    total_scenarios: 10,
                    passed: 10,
                    failed: 0,
                    max_wall_ms: 250,
                    verdict: "pass".into(),
                    scenarios_blake3: "f".repeat(64),
                },
            ),
            // D-28 (SCHEMA_VERSION 145): adversarial example farming run.
            // Host-side scaffolding for the Adv-RM (arXiv:2504.06141)
            // farm loop. Live PPM loop + ppm_finetune.sh + bench_prmbench
            // v1/v2 are RUNPOD-PENDING.
            SsbPayload::AdversarialFarmRunCompleted(
                AdversarialFarmRunCompletedPayload {
                    samples_farmed: 1000,
                    rounds: 2,
                    prmbench_delta_pp: 1.5,
                    verdict: "pass".into(),
                    paper_floor_pp: 1.0,
                    corpus_blake3: "a".repeat(64),
                },
            ),
            // D-29 (SCHEMA_VERSION 146): hierarchical BAI round completion.
            // Wire-dark by default (ELAI_BAI_MODE=hierarchical AND
            // ELAI_BAI_ENABLE=1 required). Paper labels:
            // arXiv:2111.06929 SUPPORTING EVIDENCE; arXiv:2106.10417
            // DIRECT GROUNDING. Live regret baseline RUNPOD-PENDING.
            SsbPayload::HierarchicalBaiRoundCompleted(
                HierarchicalBaiRoundCompletedPayload {
                    round: 1,
                    family_id: 0,
                    mu_posterior_mean: 0.5,
                    mu_posterior_variance: 0.01,
                    kappa_posterior_mean: 4.0,
                    kappa_posterior_variance: 0.5,
                    arm_posteriors: Vec::new(),
                    selected_arm_index: u32::MAX,
                    selected_template_id: "0".repeat(32),
                    selection_basis: "flat_ts".into(),
                    gibbs_iterations: 80,
                    canary_gate_passed: true,
                },
            ),
            // D-30 (SCHEMA_VERSION 147): calibration bootstrap warm-start.
            // Wire-dark by default (ELAI_CALIBRATION_BOOTSTRAP_ENABLE=1
            // required). Paper labels: arXiv:2509.01455 UniCR
            // COMPOSITION; arXiv:2109.07958 TruthfulQA DATA ORIGIN;
            // Politis & Romano 1994 JASA §3 DIRECT GROUNDING (SECONDARY).
            // Live D-22 validation + 817-row import + block-bootstrap
            // CI activation RUNPOD-PENDING.
            SsbPayload::CalibrationBootstrapCompleted(
                CalibrationBootstrapCompletedPayload {
                    corpus_slug: "truthfulqa".into(),
                    corpus_size: 50,
                    corpus_blake3: "b".repeat(64),
                    tau_warm_start: 0.82,
                    alpha: 0.1,
                    tau_floor_active: false,
                    bootstrap_replicates: Some(1000),
                    bootstrap_ci_low: Some(0.78),
                    bootstrap_ci_high: Some(0.86),
                    block_length_p: Some(0.1414213562373095),
                    ts_ms: 1_748_044_900_000,
                    calibration_id: None,
                },
            ),
            // C-50 (SCHEMA_VERSION 125): Contract-Coding STRUCTURAL
            // contract test scaffolding row pair. Paper-grounded
            // against arXiv:2604.13100 (Lin, Zhao, Shi 2026 — kernel
            // K = <N, Sigma, Delta> §4.2). Full Generator ->
            // Discriminator RL pipeline (Algorithm 1) is
            // RUNPOD-PENDING.
            SsbPayload::ContractTestScaffolded(ContractTestScaffoldedPayload {
                fn_name_hash: 0xC50C_50C5_0C50_C50C,
                contract_kind_hash: 0xC0DE_C0DE_C0DE_C0DE,
                ts_ms: 1_748_044_900_000,
            }),
            SsbPayload::ContractViolationDetected(ContractViolationDetectedPayload {
                fn_name_hash: 0xC50C_50C5_0C50_C50C,
                violation_kind_hash: 0xBEEF_BEEF_BEEF_BEEF,
                severity: 1,
                ts_ms: 1_748_044_900_000,
            }),
            // C-52 (SCHEMA_VERSION 126): Dafny Formal Verification Spike
            // heal-loop termination beacon. Paper-grounded against
            // arXiv:2604.22601 (NL2VC). Live `dafny verify` execution
            // is RUNPOD-PENDING (local dev host lacks the dafny
            // binary). Model substitution mandatory — Gemma 4 BLOCKED
            // per memory feedback_no_gemma4.
            SsbPayload::DafnyVerifyCompleted(DafnyVerifyCompletedPayload {
                tier_hash: 0xC52C_52C5_2C52_C52C,
                dafny_pass: true,
                udebug_pass: true,
                heal_iter: 3,
                ts_ms: 1_748_044_900_000,
            }),
            // D-16 closure (SCHEMA_VERSION 127): durable tau audit row.
            // Paper grounding arXiv:2509.01455 UniCR Eq.5 CRC quantile.
            // `alpha` is the miscoverage rate; `rho` (selective risk
            // budget) is DISTINCT and does NOT live on the row per
            // D-16-decisions.md DECISION-4.
            SsbPayload::CalibrationCorpusUpdated(CalibrationCorpusUpdatedPayload {
                corpus_size: 40,
                new_tau: 0.8,
                alpha: 0.1,
                source: "synthetic".into(),
                ts_ms: 1_748_044_927_000,
            }),
            // E-12 Tier-B (SCHEMA_VERSION 128): production fullstack
            // codegen — framework profile selection + legacy linter
            // strip. Two-variant atomic bump.
            SsbPayload::FrameworkProfileSelected(FrameworkProfileSelectedPayload {
                profile_name_hash: 0xE12E_12E1_2E12_E12E,
                version_pin_hash: 0xC0DE_C0DE_C0DE_C0DE,
                language_hash: 0xABAD_1DEA_ABAD_1DEA,
                ts_ms: 1_748_044_928_000,
            }),
            SsbPayload::LegacyConfigStripped(LegacyConfigStrippedPayload {
                config_path_hash: 0xDEAD_BEEF_DEAD_BEEF,
                lint_engine_hash: 0xFEED_BABE_FEED_BABE,
                ts_ms: 1_748_044_928_000,
            }),
            // D-18 closure (SCHEMA_VERSION 129): typed-ObservableEffect 4-variant pack.
            SsbPayload::ObservableEffectDispatched(ObservableEffectDispatchedPayload {
                effect_id: "eff-d18-dispatch".into(),
                trigger: "test_command".into(),
                context_kind: "sandbox".into(),
                ts_ms: 1_700_000_129_001,
            }),
            SsbPayload::GoodhartRiskFlag(GoodhartRiskFlagPayload {
                effect_id: "eff-d18-goodhart".into(),
                risk_kind: "constant_component".into(),
                evidence: "component variation 0.0001 < 0.001".into(),
                ts_ms: 1_700_000_129_002,
            }),
            SsbPayload::SandboxObservation(SandboxObservationPayload {
                effect_id: "eff-d18-sandbox".into(),
                tier: "container_isolated".into(),
                outcome: "confirmed".into(),
                evidence_count: 3,
                ts_ms: 1_700_000_129_003,
            }),
            SsbPayload::ProductionObservationProposal(ProductionObservationProposalPayload {
                effect_id: "eff-d18-prod".into(),
                instrument_file: None,
                requires_human_review: true,
                ts_ms: 1_700_000_129_004,
            }),
            // E-13 (SCHEMA_VERSION 130): TDD verifier-agent beacon family.
            SsbPayload::TddCycleStarted(TddCycleStartedPayload {
                task_id_hash: 0x1111_2222_3333_4444,
                ts_ms: 1_700_000_130_001,
            }),
            SsbPayload::TddTestFailed(TddTestFailedPayload {
                task_id_hash: 0x1111_2222_3333_4444,
                test_name_hash: 0x5555_6666_7777_8888,
                error_kind_hash: 0x9999_AAAA_BBBB_CCCC,
                ts_ms: 1_700_000_130_002,
            }),
            SsbPayload::TddTestPassed(TddTestPassedPayload {
                task_id_hash: 0x1111_2222_3333_4444,
                test_name_hash: 0x5555_6666_7777_8888,
                mutation_score: Some(0.75),
                ts_ms: 1_700_000_130_003,
            }),
            // E-14 (SCHEMA_VERSION 131): browser-e2e agent beacon family.
            SsbPayload::BrowserE2eRunStarted(BrowserE2eRunStartedPayload {
                journey_name_hash: 0xAAAA_BBBB_CCCC_DDD0,
                url_hash: 0xAAAA_BBBB_CCCC_DDD1,
                ts_ms: 1_700_000_131_001,
            }),
            SsbPayload::BrowserE2eAssertionFailed(BrowserE2eAssertionFailedPayload {
                journey_name_hash: 0xAAAA_BBBB_CCCC_DDD0,
                assertion_hash: 0xAAAA_BBBB_CCCC_DDD2,
                error_hash: 0xAAAA_BBBB_CCCC_DDD3,
                ts_ms: 1_700_000_131_002,
            }),
            SsbPayload::BrowserE2eAssertionPassed(BrowserE2eAssertionPassedPayload {
                journey_name_hash: 0xAAAA_BBBB_CCCC_DDD0,
                assertion_hash: 0xAAAA_BBBB_CCCC_DDD4,
                ts_ms: 1_700_000_131_003,
            }),
            SsbPayload::BrowserE2eSnapshotCaptured(BrowserE2eSnapshotCapturedPayload {
                journey_name_hash: 0xAAAA_BBBB_CCCC_DDD0,
                snapshot_kind_hash: 0xAAAA_BBBB_CCCC_DDD5,
                ts_ms: 1_700_000_131_004,
            }),
            // E-18 (SCHEMA_VERSION 132): tool-use reliability gate beacons.
            SsbPayload::ToolReliabilityGatePassed(ToolReliabilityGatePassedPayload {
                tool_name_hash: 0xE018_0001_0000_0000,
                call_shape_hash: 0xE018_0001_0000_0001,
                ts_ms: 1_700_000_132_001,
            }),
            SsbPayload::ToolReliabilityGateRejected(ToolReliabilityGateRejectedPayload {
                tool_name_hash: 0xE018_0002_0000_0000,
                reject_reason_hash: 0xE018_0002_0000_0001,
                ts_ms: 1_700_000_132_002,
            }),
            SsbPayload::ToolCallSpilled(ToolCallSpilledPayload {
                tool_name_hash: 0xE018_0003_0000_0000,
                spill_path_hash: 0xE018_0003_0000_0001,
                payload_bytes: 16_384,
                ts_ms: 1_700_000_132_003,
            }),
            SsbPayload::ToolCallRetryEscalated(ToolCallRetryEscalatedPayload {
                tool_name_hash: 0xE018_0004_0000_0000,
                attempt: 1,
                escalate_to_role_hash: 0xE018_0004_0000_0001,
                ts_ms: 1_700_000_132_004,
            }),
            // E-37 (SCHEMA_VERSION 133): prompt-injection defense beacons.
            // 9 atomic variants — OWASP LLM Top 10 LLM01 mitigation.
            SsbPayload::InjectionAttemptDetected(InjectionAttemptDetectedPayload {
                source_hash: 0xE037_0001_0000_0000,
                classifier_verdict_hash: 0xE037_0001_0000_0001,
                ts_ms: 1_700_000_133_001,
            }),
            SsbPayload::InjectionAttemptBlocked(InjectionAttemptBlockedPayload {
                source_hash: 0xE037_0002_0000_0000,
                defense_layer_hash: 0xE037_0002_0000_0001,
                ts_ms: 1_700_000_133_002,
            }),
            SsbPayload::SpotlightWrapped(SpotlightWrappedPayload {
                input_hash: 0xE037_0003_0000_0000,
                marker_hash: 0xE037_0003_0000_0001,
                ts_ms: 1_700_000_133_003,
            }),
            SsbPayload::SpotlightBreached(SpotlightBreachedPayload {
                input_hash: 0xE037_0004_0000_0000,
                marker_hash: 0xE037_0004_0000_0001,
                ts_ms: 1_700_000_133_004,
            }),
            SsbPayload::ToolDescriptionPinMismatch(ToolDescriptionPinMismatchPayload {
                tool_name_hash: 0xE037_0005_0000_0000,
                expected_hash: 0xE037_0005_0000_0001,
                observed_hash: 0xE037_0005_0000_0002,
                ts_ms: 1_700_000_133_005,
            }),
            SsbPayload::CapabilityGateRejected(CapabilityGateRejectedPayload {
                tool_name_hash: 0xE037_0006_0000_0000,
                origin_trust_hash: 0xE037_0006_0000_0001,
                ts_ms: 1_700_000_133_006,
            }),
            SsbPayload::ProvenanceTagged(ProvenanceTaggedPayload {
                text_source_hash: 0xE037_0007_0000_0000,
                trust_level_hash: 0xE037_0007_0000_0001,
                ts_ms: 1_700_000_133_007,
            }),
            SsbPayload::PromptArmorVerdict(PromptArmorVerdictPayload {
                verdict_hash: 0xE037_0008_0000_0000,
                reason_hash: 0xE037_0008_0000_0001,
                ts_ms: 1_700_000_133_008,
            }),
            SsbPayload::InjectionFixtureReplayed(InjectionFixtureReplayedPayload {
                fixture_name_hash: 0xE037_0009_0000_0000,
                defenses_passed: 3,
                ts_ms: 1_700_000_133_009,
            }),
            SsbPayload::OutputClassifierFlagged(OutputClassifierFlaggedPayload {
                tool_name: "Bash".to_string(),
                stripped_count: 1,
                stripped_hosts: vec!["evil.example".to_string()],
                patterns: Vec::new(),
            }),
            // E-40 (SCHEMA_VERSION 134): orchestrator-rooted symbol-graph beacons.
            // 3 atomic variants. See workflow/reasoning/E-40-decisions.md.
            SsbPayload::SymbolGraphQueried(SymbolGraphQueriedPayload {
                query_kind_hash: 0xE040_0001_0000_0000,
                symbol_hash: 0xE040_0001_0000_0001,
                result_count: 7,
                ts_ms: 1_700_000_134_001,
            }),
            SsbPayload::SymbolGraphCacheLoaded(SymbolGraphCacheLoadedPayload {
                graph_path_hash: 0xE040_0002_0000_0000,
                node_count: 12_345,
                ts_ms: 1_700_000_134_002,
            }),
            SsbPayload::LspOracleConsulted(LspOracleConsultedPayload {
                language_hash: 0xE040_0003_0000_0000,
                symbol_hash: 0xE040_0003_0000_0001,
                oracle_kind_hash: 0xE040_0003_0000_0002,
                ts_ms: 1_700_000_134_003,
            }),
            // E-44 (SCHEMA_VERSION 136): durability layer beacons.
            SsbPayload::CheckpointWritten(CheckpointWrittenPayload {
                session_id_hash: 0xE044_0001_0000_0000,
                schema_version: 136,
                file_count: 5,
                ts_ms: 1_700_000_136_001,
            }),
            SsbPayload::CheckpointResumed(CheckpointResumedPayload {
                session_id_hash: 0xE044_0002_0000_0000,
                schema_version: 136,
                ts_ms: 1_700_000_136_002,
            }),
            SsbPayload::CheckpointMigrated(CheckpointMigratedPayload {
                from_version: 40,
                to_version: 136,
                transform_count: 96,
                ts_ms: 1_700_000_136_003,
            }),
            SsbPayload::IdempotentDispatchSkipped(IdempotentDispatchSkippedPayload {
                call_shape_hash: 0xE044_0004_0000_0000,
                cached_result_hash: 0xE044_0004_0000_0001,
                ts_ms: 1_700_000_136_004,
            }),
            SsbPayload::DurabilityLockBlocked(DurabilityLockBlockedPayload {
                lock_path_hash: 0xE044_0005_0000_0000,
                holder_hash: 0xE044_0005_0000_0001,
                ts_ms: 1_700_000_136_005,
            }),
            // E-45 (SCHEMA_VERSION 137): cross-language drift detector beacons.
            SsbPayload::CrossLangDriftDetected(CrossLangDriftDetectedPayload {
                pair_kind_hash: 0xE045_0001_0000_0000,
                drift_count: 2,
                severity_max: 2,
                ts_ms: 1_700_000_137_001,
            }),
            SsbPayload::CrossLangPairScanned(CrossLangPairScannedPayload {
                repo_root_hash: 0xE045_0002_0000_0000,
                pair_count: 6,
                ts_ms: 1_700_000_137_002,
            }),
            SsbPayload::SchemaVersionMismatch(SchemaVersionMismatchPayload {
                rust_schema: 137,
                python_schema: 136,
                ts_ms: 1_700_000_137_003,
            }),
            // E-49 (SCHEMA_VERSION 138): coding-agent metric suite beacons.
            SsbPayload::CodingAgentMetricsCaptured(CodingAgentMetricsCapturedPayload {
                session_id_hash: 0xE049_0001_0000_0000,
                pass_at_1_x10000: 7500,
                refine_rate_x10000: 6000,
                cost_per_task_cents: Some(42),
                ts_ms: 1_700_000_138_001,
            }),
            SsbPayload::SweBenchProRunStarted(SweBenchProRunStartedPayload {
                task_pool_version_hash: 0xE049_0002_0000_0000,
                model_id_hash: 0xE049_0002_0000_0001,
                max_tasks: Some(50),
                ts_ms: 1_700_000_138_002,
            }),
            SsbPayload::SweBenchProRunCompleted(SweBenchProRunCompletedPayload {
                task_pool_version_hash: 0xE049_0003_0000_0000,
                model_id_hash: 0xE049_0003_0000_0001,
                tasks_attempted: 50,
                tasks_solved: 33,
                pass_at_1_x10000: 6600,
                ts_ms: 1_700_000_138_003,
            }),
            SsbPayload::RefineBenchScoreEmitted(RefineBenchScoreEmittedPayload {
                suite_version_hash: 0xE049_0004_0000_0000,
                model_id_hash: 0xE049_0004_0000_0001,
                refine_rate_x10000: 5500,
                ts_ms: 1_700_000_138_004,
            }),
            SsbPayload::MetricsAggregationRun(MetricsAggregationRunPayload {
                session_count: 3,
                pass_at_1_avg_x10000: 7000,
                refine_rate_avg_x10000: 5800,
                ts_ms: 1_700_000_138_005,
            }),
            // E-52 Tier-B (SCHEMA_VERSION 139): LSP-as-verifier beacons.
            SsbPayload::DiagnosticProbeDispatched(DiagnosticProbeDispatchedPayload {
                probe_kind_hash: 0xE052_0001_0000_0000,
                language_hash: 0xE052_0001_0000_0001,
                file_count: 3,
                ts_ms: 1_700_000_139_001,
            }),
            SsbPayload::DiagnosticProbeCompleted(DiagnosticProbeCompletedPayload {
                probe_kind_hash: 0xE052_0002_0000_0000,
                run_status_hash: 0xE052_0002_0000_0001,
                error_count: 0,
                warning_count: 2,
                duration_ms: 420,
                ts_ms: 1_700_000_139_002,
            }),
            SsbPayload::DiagnosticProbeBlock(DiagnosticProbeBlockPayload {
                probe_kind_hash: 0xE052_0003_0000_0000,
                error_count: 5,
                first_path_hash: 0xE052_0003_0000_0001,
                ts_ms: 1_700_000_139_003,
            }),
            SsbPayload::ScipRefreshCompleted(ScipRefreshCompletedPayload {
                shard_count: 7,
                crate_count: 4,
                duration_ms: 12_500,
                ts_ms: 1_700_000_139_004,
            }),
            // E-53 (SCHEMA_VERSION 140): formal-verify pipeline boundary beacon.
            // Single atomic +1 bump consolidating the plan's originally-
            // proposed 13 variants per the C-52 DafnyVerifyCompleted
            // precedent. Composes C-50 contract_coding + C-52 dafny_verify.
            // Gemma-4 BLOCKED per memory feedback_no_gemma4. Live Z3 /
            // Kani / Verus dispatch is RUNPOD-PENDING.
            SsbPayload::FormalVerifierCompleted(FormalVerifierCompletedPayload {
                verifier_kind_hash: 0xE053_0001_0000_0000,
                outcome_hash: 0xE053_0001_0000_0001,
                csq_class_hash: 0xE053_0001_0000_0002,
                contract_emitted: true,
                duration_ms: 32,
                ts_ms: 1_700_000_140_001,
            }),
            // E-35 (SCHEMA_VERSION 141): structured-output substrate beacons.
            // 10 atomic variants for XGrammar / INSTRUCTOR / CodeAct /
            // SEARCH/REPLACE / cloud strict-schema. AgentRole::Runner addition
            // DEFERRED per E41_AGENTROLE_FROZEN_AT_17. Python sidecar wiring +
            // AnthropicClient beta-header propagation RUNPOD-PENDING.
            SsbPayload::XgrammarMaskApplied(XgrammarMaskAppliedPayload {
                schema_id_hash: 0xE035_0001_0000_0000,
                schema_hash_hash: 0xE035_0001_0000_0001,
                mask_applied_count: 12,
                generation_ms: 311,
                cache_hit: true,
                ts_ms: 1_700_000_141_001,
            }),
            SsbPayload::InstructorModeActivated(InstructorModeActivatedPayload {
                role_hash: 0xE035_0002_0000_0000,
                block_hash: 0xE035_0002_0000_0001,
                ts_ms: 1_700_000_141_002,
            }),
            SsbPayload::GrammarModeActivated(GrammarModeActivatedPayload {
                role_hash: 0xE035_0003_0000_0000,
                block_hash: 0xE035_0003_0000_0001,
                ts_ms: 1_700_000_141_003,
            }),
            SsbPayload::CodeactActionExecuted(CodeactActionExecutedPayload {
                action_hash: 0xE035_0004_0000_0000,
                line_count: 18,
                outcome_hash: 0xE035_0004_0000_0001,
                duration_ms: 42,
                ts_ms: 1_700_000_141_004,
            }),
            SsbPayload::SearchReplaceTier1Hit(SearchReplaceTier1HitPayload {
                file_path_hash: 0xE035_0005_0000_0000,
                edit_count: 3,
                ts_ms: 1_700_000_141_005,
            }),
            SsbPayload::SearchReplaceCollisionFallback(
                SearchReplaceCollisionFallbackPayload {
                    file_path_hash: 0xE035_0006_0000_0000,
                    collision_token_hash: 0xE035_0006_0000_0001,
                    ts_ms: 1_700_000_141_006,
                },
            ),
            SsbPayload::CloudStrictSchemaApplied(CloudStrictSchemaAppliedPayload {
                provider_hash: 0xE035_0007_0000_0000,
                schema_id_hash: 0xE035_0007_0000_0001,
                ts_ms: 1_700_000_141_007,
            }),
            SsbPayload::GrammarCacheLoaded(GrammarCacheLoadedPayload {
                compiled_count: 64,
                cache_hits: 48,
                cache_misses: 16,
                total_compile_ms: 4_200,
                ts_ms: 1_700_000_141_008,
            }),
            SsbPayload::StructuredEngineRejected(StructuredEngineRejectedPayload {
                engine_hash: 0xE035_0009_0000_0000,
                reason_hash: 0xE035_0009_0000_0001,
                ts_ms: 1_700_000_141_009,
            }),
            SsbPayload::LegacyJsonObjectRefused(LegacyJsonObjectRefusedPayload {
                provider_hash: 0xE035_000A_0000_0000,
                ts_ms: 1_700_000_141_010,
            }),
            // E-6 (SCHEMA_VERSION 148): MAST-grounded multi-agent failure-mode defenses.
            SsbPayload::AuditMastTagged(AuditMastTaggedPayload {
                verdict_seq: 1,
                mast_mode: "fm_1_1_disobey_task_spec".into(),
                confidence_band: "medium".into(),
                agent_role: "auditor".into(),
                ts_ms: 1_700_000_148_001,
            }),
            SsbPayload::IsoBudgetMeasured(IsoBudgetMeasuredPayload {
                task_fixture: "fixtures/smoke.toml".into(),
                budget_tokens: 2000,
                full_pipeline_pass_rate: 0.5,
                single_agent_wait_pass_rate: 0.55,
                insufficient_lift: true,
                runpod_pending: true,
                mast_mode_counts: vec![MastModeCount {
                    mast_mode: "fm_1_3_step_repetition".into(),
                    count: 3,
                }],
                // H-40: stub values for the schema-validation fixture.
                // insufficient_contribution is empty map (unknown per-role state on stub).
                unique_catch_rate: std::collections::BTreeMap::new(),
                decorrelation_score: std::collections::BTreeMap::new(),
                insufficient_contribution: std::collections::BTreeMap::new(),
            }),
            SsbPayload::PersonaReanchored(PersonaReanchoredPayload {
                agent_role: "executor".into(),
                turn_count: 20,
                reason: "turn_counter".into(),
                cosine_similarity: None,
                embedding_available: false,
            }),
            SsbPayload::CrossFamilyUnavailable(CrossFamilyUnavailablePayload {
                executor_family: "local".into(),
                phase_kind: "code".into(),
                claim_id: "c-1".into(),
                cloud_allowed: false,
                ts_ms: 1_700_000_148_002,
            }),
            // E-7 (SCHEMA_VERSION 149): darwinian self-improvement substrate.
            // SINGLE-VARIANT scope-lean per Pattern-2; phase discriminator
            // covers the 11 fire-points. See workflow/reasoning/E-7-decisions.md.
            SsbPayload::DarwinianGenerationCompleted(DarwinianGenerationCompletedPayload {
                phase: DarwinianPhase::ArchiveCellElevated,
                generation_index: 0,
                cell_blake3_prefix: [0xE7, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x01],
                tau: Some(0.05),
                k_m: None,
                reason_hash: None,
                ts_ms: 1_700_000_149_001,
            }),
            // batch2-merge-collision recovery (SCHEMA_VERSION 150-156).
            // E-3 RuntimeHandoffSignal.
            SsbPayload::RuntimeHandoffSignal(RuntimeHandoffSignalPayload {
                phase: HandoffSignalPhase::Begin,
                from_backend_hash: 0xE003_0000_0000_0001,
                to_backend_hash: 0xE003_0000_0000_0002,
                request_id_hash: 0xE003_0000_0000_0003,
                ts_ms: 1_700_000_150_001,
            }),
            // E-26 inference-engines 2026 pack.
            SsbPayload::SglangBackendRegistered(SglangBackendRegisteredPayload {
                base_url_blake3: "a".repeat(64),
                structured_output_backend: "xgrammar".into(),
                ts_ms: 1_700_000_151_001,
            }),
            SsbPayload::XgrammarSchemaCompiled(XgrammarSchemaCompiledPayload {
                schema_id_hash: 0xE026_0000_0000_0002,
                schema_hash: "b".repeat(64),
                compile_ms: 47,
                ts_ms: 1_700_000_151_002,
            }),
            SsbPayload::RuntimeColdStartObserved(RuntimeColdStartObservedPayload {
                backend_name_hash: 0xE026_0000_0000_0003,
                elapsed_ms: 8_120,
                ts_ms: 1_700_000_151_003,
            }),
            // E-2 streaming timing + UX pacing vote.
            SsbPayload::StreamingTimingRecorded(StreamingTimingRecordedPayload {
                request_id: "cc-stream-test".into(),
                ttft_ms: 187,
                mean_inter_chunk_ms: 42,
                p50_inter_chunk_ms: 31,
                p99_inter_chunk_ms: 280,
                total_ms: 4_700,
                tokens_emitted: 162,
                stop_reason: "end_turn".into(),
                ts_ms: 1_700_000_152_001,
            }),
            SsbPayload::UxPacingVoteRecorded(UxPacingVoteRecordedPayload {
                request_id: "cc-stream-vote".into(),
                vote: "snappy".into(),
                frame_pace_enabled_at_vote: true,
                cps_at_vote: 60,
                effort_level: "medium".into(),
                ts_ms: 1_700_000_153_001,
            }),
            // E-28 attention-kernels 2026 pack.
            SsbPayload::StreamingLlmEngaged(StreamingLlmEngagedPayload {
                sink_count: 4,
                window_size: 25_000,
                context_position: 30_000,
                ts_ms: 1_700_000_154_001,
            }),
            SsbPayload::SdpaBackendPinned(SdpaBackendPinnedPayload {
                backend_name: "flash_attention".into(),
                fallback_used: false,
                compute_capability: 86,
                ts_ms: 1_700_000_155_001,
            }),
            SsbPayload::YarnRopeScalingApplied(YarnRopeScalingAppliedPayload {
                factor: 4.0,
                original_max_position_embeddings: 32_768,
                applied_at_layer: 0,
                ts_ms: 1_700_000_156_001,
            }),
            // E-8 (SCHEMA_VERSION 162): self-evolving prompts substrate.
            // SINGLE-VARIANT scope-lean per Pattern-2; OptimizerPhase
            // discriminator covers 7 fire-points. See
            // workflow/reasoning/E-8-decisions.md DECISION-1.
            SsbPayload::PromptOptimizationCompleted(PromptOptimizationCompletedPayload {
                phase: OptimizerPhase::ArchivePromote,
                iteration_index: 7,
                run_id_prefix: [0xE8, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x01],
                template_blake3_prefix: [0xE8, 0x02, 0x00, 0x00, 0x00, 0x00, 0x00, 0x02],
                val_lift_pct: Some(8.5),
                held_out_lift_pct: Some(7.1),
                reason_hash: None,
                ts_ms: 1_700_000_162_001,
            }),
            // E-10 (SCHEMA_VERSION 163): hostile-critic structural simplifications.
            // SINGLE-VARIANT scope-lean per Pattern-2; CriticTilePhase
            // discriminator covers 9 fire-points. See
            // workflow/reasoning/E-10-decisions.md DECISION-1.
            SsbPayload::CriticDecisionEvent(CriticDecisionEventPayload {
                phase: CriticTilePhase::FsmAblation,
                run_id_prefix: [0xE1, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x10],
                baseline_score: 0.62,
                challenger_score: 0.58,
                paired_bootstrap_p: Some(0.18),
                sample_size: 30,
                verdict_code: 0,
                rationale_hash: [0xE1, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x11],
                ts_ms: 1_700_000_163_001,
            }),
            // E-43 (SCHEMA_VERSION 164): agent skill systems telemetry.
            // SINGLE-VARIANT scope-lean per Pattern-2 (16th application);
            // SkillEventPhase 3-arm discriminator covers loaded/invoked/
            // failed fire-points. See workflow/reasoning/E-43-decisions.md
            // DECISION-1.
            SsbPayload::SkillSystemEvent(SkillSystemEventPayload {
                phase: SkillEventPhase::Invoked,
                skill_name: "anti-spiral".into(),
                prompt_hash_prefix: Some([0xE4, 0x30, 0x00, 0x00, 0x00, 0x00, 0x00, 0x01]),
                trigger_method: "explicit".into(),
                outcome: "passed".into(),
                eval_precision: None,
                eval_recall: None,
                scan_findings: None,
                ts_ms: 1_700_000_164_001,
            }),
            // E-9 (SCHEMA_VERSION 165): external agent-eval benchmark suite.
            // SINGLE-VARIANT scope-lean per Pattern-2 (17th application);
            // BenchPhase 5-arm discriminator covers loaded/started/
            // item_evaluated/completed/aborted fire-points. See
            // workflow/reasoning/E-9-decisions.md DECISION-1.
            SsbPayload::BenchmarkRunObserved(BenchmarkRunObservedPayload {
                phase: BenchPhase::Completed,
                suite_name: "swe-bench-pro".into(),
                suite_version: "2509.16941".into(),
                score: Some(0.23),
                n: Some(200),
                sota_reference: Some(0.23),
                p_value: Some(0.04),
                low_power: false,
                required_n: Some(196),
                model_effect_pp: None,
                harness_effect_pp: None,
                coverage_gap_pp: None,
                runtime_lane: "cloud_api".into(),
                harness_blake3_prefix: "e90000000000a9b1".into(),
                ts_ms: 1_700_000_165_001,
            }),
            // E-19 (SCHEMA_VERSION 166): reasoning quality probe.
            // SINGLE-VARIANT scope-lean per Pattern-2 (18th application);
            // ProbeKind 6-arm discriminator covers l2_gate_decision /
            // l4_prm_lite_score / l4_lateral_kept / l4_ttc_budget_chosen /
            // l6_verifier_diversity / vividness_probe (I-5). See workflow/reasoning/E-19-decisions.md.
            SsbPayload::ReasoningQualityProbe(ReasoningQualityProbePayload {
                kind: ProbeKind::L2GateDecision,
                layer: 2,
                baby_step_id: "bs-e19-test".into(),
                verdict: "skip_high_confidence".into(),
                score: Some(0.91),
                threshold: Some(0.80),
                observed_tokens: None,
                token_cap: None,
                failure_taxonomy_category: "missing_import".into(),
                extras: "no_cot_class=false".into(),
                ts_ms: 1_700_000_166_001,
            }),
            // E-55 (SCHEMA_VERSION 168): agentic RL + reward ladder substrate.
            // SINGLE-VARIANT Pattern-2 (20th application);
            // RewardLadderEventKind 9-arm discriminator covers
            // rlaif_judge_rated / rlaif_calibration_correlation_low /
            // gepa_template_evolved / gepa_template_evicted /
            // reward_fusion_computed / reward_hacking_suspected /
            // sft_training_threshold_reached /
            // bai_arm_observation_recorded / rl_weight_update_unlocked.
            // See workflow/reasoning/E-55-decisions.md.
            SsbPayload::RewardLadderEvent(RewardLadderEventPayload {
                kind: RewardLadderEventKind::RlaifJudgeRated,
                session_id: Some("s-e55-test".into()),
                trajectory_id: Some("traj-e55-test".into()),
                details_json: serde_json::json!({
                    "correctness": 0.9,
                    "step_validity": 0.85,
                    "sycophancy_absence": 1.0,
                    "format_compliance": 1.0,
                    "conciseness": 0.95,
                    "model_id": "claude-sonnet-4-7-1m"
                }),
                ts_ms: 1_700_000_167_001,
            }),
            // E-11 (SCHEMA_VERSION 169): SaaS prior-art catalog +
            // recommender substrate. SINGLE-VARIANT Pattern-2 (21st
            // application); PriorArtEventKind 2-arm discriminator
            // covers archetype_recommended / prior_art_consulted. See
            // workflow/reasoning/E-11-decisions.md.
            SsbPayload::SaasPriorArtEvent(SaasPriorArtEventPayload {
                kind: PriorArtEventKind::ArchetypeRecommended,
                session_id: Some("s-e11-test".into()),
                archetype: "saas_dashboard".into(),
                confidence: "high".into(),
                details_json: serde_json::json!({
                    "project_class": "saas_dashboard",
                    "clarifying_question_count": 0,
                    "prior_art_pointer_count": 4,
                    "caller_site": "test_payload"
                }),
                ts_ms: 1_700_000_169_001,
            }),
            // E-22 (SCHEMA_VERSION 170): may-2026 github trending sweep
            // substrate. SINGLE-VARIANT Pattern-2 (22nd application);
            // TrendingEventKind 3-arm discriminator covers
            // skill_export_normalized / microvm_fallback /
            // lazy_schema_savings. See
            // workflow/reasoning/E-22-decisions.md.
            SsbPayload::GithubTrendingEvent(GithubTrendingEventPayload {
                kind: TrendingEventKind::SkillExportNormalized,
                session_id: Some("s-e22-test".into()),
                skill_name: "test-skill".into(),
                reason_label: String::new(),
                fallback_backend: String::new(),
                pre_bytes: 0,
                post_bytes: 0,
                tool_count: 0,
                drift_count: 0,
                details_json: serde_json::json!({
                    "dist_path": "dist/claude/skills/test-skill/SKILL.md",
                    "target": "claude",
                    "caller_site": "test_payload"
                }),
                ts_ms: 1_700_000_170_001,
            }),
            // E-23 (SCHEMA_VERSION 171): 2026 developer-discourse
            // positioning + UX-guard substrate. SINGLE-VARIANT Pattern-2
            // (23rd application); DiscourseEventKind 3-arm discriminator
            // covers trust_telemetry_aggregated /
            // runtime_choice_badge_emitted / context_state_warning. See
            // workflow/reasoning/E-23-decisions.md.
            SsbPayload::DeveloperDiscourseEvent(DeveloperDiscourseEventPayload {
                kind: DiscourseEventKind::TrustTelemetryAggregated,
                session_id: Some("s-e23-test".into()),
                tasks_per_hour: 1.5,
                findings_per_task: 0.4,
                revert_rate: 0.05,
                ungrounded_count: 0,
                prev_badge: String::new(),
                next_badge: String::new(),
                token_budget_pct: 0.0,
                threshold_label: String::new(),
                details_json: serde_json::json!({
                    "stagnation_dist": { "spinning": 0u64 },
                    "caller_site": "test_payload"
                }),
                ts_ms: 1_700_000_171_001,
            }),
            // E-25 (SCHEMA_VERSION 172): claude-code-architecture catalog
            // substrate. SINGLE-VARIANT Pattern-2 (24th application);
            // ClaudeCodeArchitectureEventKind 9-arm discriminator covers
            // compaction_ladder_tier_applied / skill_template_executed /
            // prompt_decision_made / opusplan_overlay_activated /
            // session_forked / edit_checkpoint_snapshotted /
            // bash_complexity_blocked / approval_fatigue_detected /
            // sandbox_template_warned. See
            // workflow/reasoning/E-25-decisions.md.
            SsbPayload::ClaudeCodeArchitectureEvent(ClaudeCodeArchitectureEventPayload {
                kind: ClaudeCodeArchitectureEventKind::CompactionLadderTierApplied,
                primitive: ClaudeCodeArchPrimitive::CompactionLadder,
                session_id: Some("s-e25-test".into()),
                details_json: serde_json::json!({
                    "tier_id": "snip",
                    "freed_tokens": 1234u64,
                    "wall_ms": 7,
                    "caller_site": "test_payload"
                }),
                ts_ms: 1_700_000_172_001,
            }),
            // E-29 Tier-B (SCHEMA_VERSION 173): agentic-observability event
            // substrate. SINGLE-VARIANT Pattern-2 (25th application);
            // AgenticObservabilityEventKind 4-arm discriminator covers
            // mast_failure_classified / replay_resume / signal_match /
            // eval_dataset_member. See
            // workflow/reasoning/E-29-decisions.md.
            SsbPayload::AgenticObservabilityEvent(AgenticObservabilityEventPayload {
                kind: AgenticObservabilityEventKind::MastFailureClassified,
                session_id: Some("s-e29-test".into()),
                mast_mode_label: MastMode::Fm13StepRepetition.as_sql_str().to_string(),
                evidence_row_ids: vec![10, 11, 12],
                confidence_bp: 10_000,
                replay_session_id: String::new(),
                replay_outcome: String::new(),
                refusal_reason: String::new(),
                checkpoint_id: 0,
                signal_label: String::new(),
                matched_row_ids: Vec::new(),
                classifier_model: String::new(),
                dataset_name: String::new(),
                trace_ids: Vec::new(),
                dataset_label: String::new(),
                scoring_rubric: String::new(),
                details_json: serde_json::json!({
                    "caller_site": "test_payload"
                }),
                ts_ms: 1_700_000_173_001,
                // OTel cols populated so the round-trip test exercises
                // the W3C trace-context + Resource attributes path.
                trace_id: Some("0af7651916cd43dd8448eb211c80319c".into()),
                span_id: Some("b7ad6b7169203331".into()),
                parent_span_id: Some("0020000000000001".into()),
                trace_flags: Some("01".into()),
                trace_state: Some("elai=ssb-e29-test".into()),
                span_name: Some("elai.mast.classify".into()),
                span_kind: Some("internal".into()),
                span_status_code: Some("ok".into()),
                span_status_message: Some(String::new()),
                service_name: Some("elai-orchestrator".into()),
                service_namespace: Some("elai".into()),
                service_instance_id: Some("instance-test-0".into()),
                resource_attributes_json: Some(
                    "{\"host.name\":\"test-host\"}".into(),
                ),
                span_attributes_json: Some(
                    "{\"mast.mode\":\"fm_1_3_step_repetition\"}".into(),
                ),
                span_events_json: Some("[]".into()),
            }),
            // E-31 (SCHEMA_VERSION 174): coding-model-leaderboard scraper
            // substrate. SINGLE-VARIANT Pattern-2 (26th application);
            // LeaderboardEventKind 3-arm discriminator covers
            // scrape_started / scrape_completed / scrape_failed. See
            // workflow/reasoning/E-31-leaderboard-decisions.md.
            SsbPayload::CodingModelLeaderboardEvent(CodingModelLeaderboardEventPayload {
                kind: LeaderboardEventKind::ScrapeCompleted,
                session_id: Some("s-e31-test".into()),
                suite_name: "swe-bench-pro".into(),
                suite_version: "public".into(),
                leaderboard_url: "https://labs.scale.com/leaderboard/swe_bench_pro_public".into(),
                scrape_run_id: "run-e31-test".into(),
                model_name: "claude-opus-4-7".into(),
                score_pct_bp: 6430,
                sample_size: 731,
                entry_hash: "fedcba9876543210".into(),
                pricing_label: "opus".into(),
                reason_label: String::new(),
                http_status: 0,
                details_json: serde_json::json!({
                    "caller_site": "test_payload",
                    "score_str": "64.3",
                    "arch_tag": "cloud_billed"
                }),
                ts_ms: 1_700_000_174_001,
            }),
            // E-38 (SCHEMA_VERSION 175): 2026 AI roadmap signals + EU AI
            // Act compliance pre-gate substrate. SINGLE-VARIANT Pattern-2
            // (27th application); RoadmapSignalKind 5-arm discriminator
            // covers compliance_blocked / compliance_verified /
            // mcp_transport_version_logged / mrcr_unverified /
            // posture_doc_stale. See workflow/reasoning/E-38-decisions.md.
            SsbPayload::AiRoadmapSignalEvent(AiRoadmapSignalEventPayload {
                kind: RoadmapSignalKind::ComplianceBlocked,
                plan_session_id: "s-e38-test".into(),
                assertion_label: "clock_provider_intact".into(),
                hint: "inject ClockProvider into PlanContext before retry".into(),
                assertions_checked: 0,
                mcp_transport_current: String::new(),
                mcp_transport_target: String::new(),
                backend_name: String::new(),
                declared_context_tokens: 0,
                mrcr_score_label: String::new(),
                posture_doc_path: String::new(),
                posture_doc_mtime_ms: 0,
                age_days: 0,
                details_json: serde_json::json!({
                    "caller_site": "test_payload",
                    "fsm_edge": "planning_to_testing_first"
                }),
                ts_ms: 1_700_000_175_001,
            }),
            // E-39 (SCHEMA_VERSION 176): cost + token economics substrate.
            // SINGLE-VARIANT Pattern-2 (28th application); CostEventKind
            // 8-arm discriminator covers task_aborted_by_spend_cap /
            // velocity_cap_breached / automix_verifier_verdict /
            // cost_router_decision / cost_estimate_inaccurate /
            // free_tier_quota_approaching / free_tier_quota_exceeded /
            // layered_cap_manual_override. See
            // workflow/reasoning/E-39-decisions.md.
            SsbPayload::CostTokenEvent(CostTokenEventPayload {
                kind: CostEventKind::CostRouterDecision,
                session_id: Some("s-e39-test".into()),
                task_id: Some("task-e39-test".into()),
                details_json: serde_json::json!({
                    "difficulty": 0.42,
                    "budget_remaining_usd": 28.50,
                    "chosen_backend": "sonnet",
                    "criticality": "medium",
                    "rationale": "hard + ample band -> sonnet workhorse",
                    "caller_site": "test_payload"
                }),
                ts_ms: 1_700_000_176_001,
            }),
            // E-50 (SCHEMA_VERSION 177): SaaS architecture pattern
            // catalog substrate. SINGLE-VARIANT Pattern-2 (29th
            // application); SaasArchPatternEventKind N-arm discriminator
            // covers 11 fire-points. See workflow/reasoning/E-50-decisions.md.
            SsbPayload::SaasArchPatternEvent(SaasArchPatternEventPayload {
                kind: SaasArchPatternEventKind::BillingMorDecisionMade,
                session_id: Some("s-e50-test".into()),
                archetype: "b2b_marketplace".into(),
                details_json: serde_json::json!({
                    "vendor": "polar",
                    "geography": "global",
                    "mrr_projection": "ten_to_hundred_k",
                    "caller_site": "test_payload"
                }),
                ts_ms: 1_700_000_177_001,
            }),
            // Tranche B-7 triple (SCHEMA_VERSION 195): TDD verification
            // gate substrate. SINGLE-VARIANT Pattern-2 (46th application);
            // TddVerificationEventKind 4-arm discriminator covers
            // PropertyGatePass / MutationGateSurvivorBlock / AntiMockBlock
            // / AssertionOnlyDiffBlock. See
            // workflow/reasoning/B7-triple-tranche-decisions.md.
            SsbPayload::TddVerificationEvent(TddVerificationEventPayload {
                kind: TddVerificationEventKind::AntiMockBlock,
                task_id_hash: 0x1111_2222_3333_4444,
                details_json: serde_json::json!({
                    "caller_site": "test_payload",
                    "hit_count": 2,
                    "fingerprint_hashes": ["0xfeedface", "0xdeadbeef"]
                }),
                ts_ms: 1_700_000_195_001,
            }),
            // F-4 / Slice B-1 (SCHEMA_VERSION 197): TestIntegrityEvent
            // SINGLE-VARIANT Pattern-2 (50th application); 2-arm
            // TestIntegrityEventKind discriminator covers Verified /
            // FabricationBlocked. Two entries cover both arms.
            // See workflow/reasoning/F-4-B1-decisions.md.
            SsbPayload::TestIntegrityEvent(TestIntegrityEventPayload {
                test_name: "crate::ssb::tests::sample_verified".into(),
                passed: true,
                stdout_hash: Some({
                    let h = blake3::hash(b"all_test_payloads_verified");
                    format!("{:.16}", h.to_hex())
                }),
                kind: TestIntegrityEventKind::Verified,
            }),
            SsbPayload::TestIntegrityEvent(TestIntegrityEventPayload {
                test_name: "crate::ssb::tests::sample_fabrication_blocked".into(),
                passed: false,
                stdout_hash: Some({
                    let h = blake3::hash(b"all_test_payloads_fabrication_blocked");
                    format!("{:.16}", h.to_hex())
                }),
                kind: TestIntegrityEventKind::FabricationBlocked,
            }),
            // F-3p1 / Slice C1 (SCHEMA_VERSION 198): ToolFilterEvent
            // SINGLE-VARIANT Pattern-2 (51st application); 2-arm
            // ToolFilterEventKind discriminator covers Committed /
            // UntrustedQuarantined. Two entries cover both arms.
            // See workflow/reasoning/F-3p1-decisions.md.
            SsbPayload::ToolFilterEvent(ToolFilterPayload {
                commit_hash: blake3::hash(b"all_test_payloads_tool_filter_committed")
                    .to_hex()
                    .to_string(),
                allowed_categories: vec!["read".into(), "web".into()],
                denied_tool: None,
                taint_source: None,
                task_id: "task-all-test-0".into(),
                kind: ToolFilterEventKind::Committed,
            }),
            SsbPayload::ToolFilterEvent(ToolFilterPayload {
                commit_hash: String::new(),
                allowed_categories: vec![],
                denied_tool: Some("bash".into()),
                taint_source: Some("malicious".into()),
                task_id: "task-all-test-1".into(),
                kind: ToolFilterEventKind::UntrustedQuarantined,
            }),
            // F-9 / Slice D1 (SCHEMA_VERSION 199): WriteTimeSlopsquatBlocked flat beacon.
            // 1 entry (not Pattern-2; single sql_str).
            SsbPayload::WriteTimeSlopsquatBlocked(WriteTimeSlopsquatBlockedPayload {
                dep_name_hash: {
                    let h = blake3::hash(b"all_test_payloads_write_time_slopsquat_blocked");
                    let bytes = h.as_bytes();
                    u64::from_be_bytes([
                        bytes[0], bytes[1], bytes[2], bytes[3],
                        bytes[4], bytes[5], bytes[6], bytes[7],
                    ])
                },
                ecosystem: "pypi".into(),
                verdict_kind: WriteTimeSlopVerdictKind::NotFound.as_str().to_owned(),
                registry_age_days: None,
                ts_ms: 1_748_900_000_001,
            }),
            // F-10 / Slice A1 (SCHEMA_VERSION 200): SecurityClassRegressionBlocked
            // flat beacon. 1 entry (not Pattern-2; single sql_str). loop_kind is a
            // payload field, so a single entry exercises the round-trip.
            SsbPayload::SecurityClassRegressionBlocked(SecurityClassRegressionBlockedPayload {
                delta_blake3: {
                    let h = blake3::hash(b"all_test_payloads_security_class_regression");
                    format!("{:.16}", h.to_hex())
                },
                introduced_classes: vec!["CWE-22".into(), "CWE-78".into()],
                prior_classes: vec!["CWE-78".into()],
                loop_kind: LoopKind::SelfMod,
                loop_id: "self-mod-loop-all-0".into(),
                lang: "python".into(),
            }),
            // F-2b / Slice A1 (SCHEMA_VERSION 201): InsecureRateMeasured flat beacon.
            // 1 entry (not Pattern-2; single sql_str). The numbers are arbitrary
            // fixtures -- the real keystone numbers are COMPUTED by the bench.
            SsbPayload::InsecureRateMeasured(InsecureRateMeasuredPayload {
                suite: "secodeplt_structured_all".into(),
                model_id: "qwen2.5-coder-7b".into(),
                n_samples: 8,
                insecure_rate_before: 0.625,
                insecure_rate_after: 0.125,
                gate_recall: 0.8,
                seed: 11,
                corpus_blake3: {
                    let h = blake3::hash(b"all_test_payloads_insecure_rate_measured");
                    format!("{:.16}", h.to_hex())
                },
            }),
            // I-9 (SCHEMA_VERSION 263): ErrorAmplificationMeasured flat beacon.
            // 1 entry (not Pattern-2; single sql_str). The numbers are arbitrary
            // fixtures -- the real numbers are COMPUTED by the bench.
            SsbPayload::ErrorAmplificationMeasured(ErrorAmplificationMeasuredPayload {
                suite: "error_amplification_all".into(),
                model_id: "error_amplification_frozen_fixture".into(),
                n_steps: 6,
                injected_step: 1,
                divergence_step: Some(1),
                alpha_t: 1.0,
                recovered_fraction: 0.0,
                repaired: false,
                annealed_temperature: Some(0.0),
                seed: 11,
                fixture_blake3: {
                    let h = blake3::hash(b"all_test_payloads_error_amplification_measured");
                    format!("{:.16}", h.to_hex())
                },
            }),
            // F-6 / Slice A1 (SCHEMA_VERSION 202): SkillLibraryEvent SINGLE-VARIANT
            // Pattern-2; 3-arm SkillLibraryEventKind discriminator covers Acquired
            // / TrustPromoted / DemotedStale. Three entries cover all arms.
            // See workflow/reasoning/F-6-decisions.md.
            SsbPayload::SkillLibraryEvent(SkillLibraryEventPayload {
                skill_id: "skill-all-test-acquired".into(),
                description: "acquired skill".into(),
                tier: "s1".into(),
                prior_tier: None,
                reason: "critic-gated acquisition".into(),
                invocations: 0,
                failure_rate: 0.0,
                kind: SkillLibraryEventKind::Acquired,
            }),
            SsbPayload::SkillLibraryEvent(SkillLibraryEventPayload {
                skill_id: "skill-all-test-promoted".into(),
                description: "promoted skill".into(),
                tier: "s2".into(),
                prior_tier: Some("s1".into()),
                reason: "cleared invocation/failure/age gate".into(),
                invocations: 4,
                failure_rate: 0.0,
                kind: SkillLibraryEventKind::TrustPromoted,
            }),
            SsbPayload::SkillLibraryEvent(SkillLibraryEventPayload {
                skill_id: "skill-all-test-demoted".into(),
                description: "demoted skill".into(),
                tier: "s1".into(),
                prior_tier: Some("s2".into()),
                reason: "staleness score crossed threshold".into(),
                invocations: 4,
                failure_rate: 0.25,
                kind: SkillLibraryEventKind::DemotedStale,
            }),
            // F-5 / Slice A1 (SCHEMA_VERSION 203): StalenessQuarantineBroadcast
            // SINGLE FLAT beacon (no Kind). One entry. broadcast_id is a blake3
            // hex-prefix; dropped_count nonzero here to exercise the overflow
            // surface field on round-trip. See workflow/reasoning/F-5-decisions.md.
            SsbPayload::StalenessQuarantineBroadcast(StalenessQuarantineBroadcastPayload {
                fact_id: "fact-all-test-quarantined".into(),
                staleness_ratio: 0.87,
                dependents_demoted: 2,
                cosmos_artifacts_flagged: 1,
                tick_ms: 1_700_000_000_500,
                broadcast_id: {
                    let h = blake3::hash(b"all_test_payloads_staleness_quarantine_broadcast");
                    format!("{:.16}", h.to_hex())
                },
                dropped_count: 3,
            }),
            // F-8 / Slice A1 (SCHEMA_VERSION 204): SymbolicConflictDetected
            // SINGLE FLAT beacon (no Kind). One entry. fingerprint is a blake3
            // hex-prefix of the conflict tuple; key is a real
            // self_mod_safety::LOCKED_KEYS entry. WARN-then-escalate (held
            // pending), not a BLOCK. See workflow/reasoning/F-8-decisions.md.
            SsbPayload::SymbolicConflictDetected(SymbolicConflictPayload {
                fingerprint: {
                    let h = blake3::hash(
                        b"all_test_payloads_symbolic_conflict|locked_key_contradiction|gate_definitions|cfg-variant-all-test",
                    );
                    format!("{:.16}", h.to_hex())
                },
                kind: "locked_key_contradiction".into(),
                key: "gate_definitions".into(),
                detail: "variant rewrites gate_definitions; key is a self_mod_safety LOCKED invariant (Goodhart vector)".into(),
                candidate_id: "cfg-variant-all-test".into(),
                fsm_substate: "AdmittingVariant".into(),
            }),
            // F-7 / Slice A1 (SCHEMA_VERSION 205): TemplateUtilityRanked SINGLE
            // FLAT beacon (no Kind). One entry. utility is Laplace-smoothed over
            // the persisted CompiledPipeline first_try_success_rate + use_count;
            // here 95 successes / 100 firings -> (95+1)/(100+2) ~= 0.941, while
            // first_try_ratio keeps the raw 0.95 baseline (W-2 no-removal). See
            // workflow/reasoning/F-7-decisions.md.
            SsbPayload::TemplateUtilityRanked(TemplateUtilityRankedPayload {
                concept_class: "static_site_generator".into(),
                plan_skeleton_hash: 0xa17c_4d9e_2f80_61b3,
                // 95 success / 100 firings -> (95+1)/(100+2) ~= 0.9412.
                utility: 96.0_f32 / 102.0_f32,
                first_try_ratio: 0.95,
                sample_n: 100,
                chosen_pipeline_id: "pipeline-all-test".into(),
            }),
            // F-13 / Slice A1 (SCHEMA_VERSION 206): SessionArchived SINGLE FLAT
            // beacon (no Kind). One entry. manifest_blake3 is a blake3 hex-prefix
            // of the archived bytes; store_kind is a payload field ('ssb' here).
            // Forensic opt-in (default-OFF ELAI_SSB_ARCHIVE_ON_BUMP_ENABLE). See
            // workflow/reasoning/F-13-session-archive-decisions.md.
            SsbPayload::SessionArchived(SessionArchivedPayload {
                session_id: "all-test-session".into(),
                stale_version: 205,
                archive_path:
                    "workflow/sessions/all-test-session/archive/ssb-v205.sqlite".into(),
                manifest_blake3: {
                    let h = blake3::hash(b"all_test_payloads_session_archived|ssb-v205.sqlite");
                    format!("{:.16}", h.to_hex())
                },
                store_kind: "ssb".into(),
            }),
            // F-11 / Slice A1 (SCHEMA_VERSION 207): DarwinianAdmit config-DGM
            // admit-loop beacon. Pattern-2 single-variant with a 3-arm
            // DarwinianAdmitKind (Admitted / RejectedByGuard / LoopClosed); ONE
            // entry covers the variant. candidate_blake3 is a blake3 hex-prefix
            // of the candidate config_blob. See workflow/reasoning/F-11-decisions.md.
            SsbPayload::DarwinianAdmit(DarwinianAdmitPayload {
                kind: DarwinianAdmitKind::RejectedByGuard,
                variant_id: "all-test-variant".into(),
                parent_id: None,
                alpha: 0.99,
                guard_rejected_by: Some("trap".into()),
                candidate_blake3: {
                    let h = blake3::hash(b"all_test_payloads_darwinian_admit|all-test-variant");
                    format!("{:.16}", h.to_hex())
                },
                seed: 42,
            }),
            // G-10 (SCHEMA_VERSION 218): ModelDriftSuspected exogenous regression gate.
            // One entry covers the variant (flat struct, no pattern-2 sub-enum on the
            // all_test_payloads side; DriftSignalKind lives on the payload).
            SsbPayload::ModelDriftSuspected(ModelDriftSuspectedPayload {
                identity_key: {
                    let h = blake3::hash(
                        b"all_test_payloads_model_drift|qwen2.5-coder-7b|local",
                    );
                    format!("{:.32}", h.to_hex())
                },
                kind: DriftSignalKind::CanaryRegressed,
                signals: vec!["canary_regressed".into()],
                per_family_delta: vec![
                    ("code_fix".into(), -0.25),
                    ("math_proof".into(), -0.18),
                ],
                affected_template_ids: vec![
                    "domain:code_fix".into(),
                    "domain:math_proof".into(),
                ],
                confirmed_by_fingerprint: false,
                baseline_captured_at: 1_748_900_000_000,
            }),
            // D-27.1 (SCHEMA_VERSION 209): DiscoveryRatified + DiscoveryRejected
            // discovery-queue reviewer-action beacons. 2 new entries.
            SsbPayload::DiscoveryRatified(DiscoveryRatifiedPayload {
                template_id: "tpl-discovery-review-1".into(),
                reviewer_id: "reviewer-alice".into(),
                ts_ms: 1_748_900_000_001,
            }),
            SsbPayload::DiscoveryRejected(DiscoveryRejectedPayload {
                template_id: "tpl-discovery-review-2".into(),
                reviewer_id: "reviewer-bob".into(),
                reason: "duplicate of existing template tpl-123".into(),
                ts_ms: 1_748_900_000_002,
            }),
            // H-32 (SCHEMA_VERSION 210): FactValidationFailed flat beacon. 1 new entry.
            SsbPayload::FactValidationFailed(FactValidationFailedPayload {
                fact_id: "fact-h32-test".into(),
                citation: "dkg:get_relevant_facts@crates/memory/src/retrieval.rs:46".into(),
                reason: "stale".into(),
                evidence_blake3: "a1b2c3d4e5f60718".into(),
            }),
            // H-33 (SCHEMA_VERSION 211): MemoryGovernanceEvent flat beacon. 1 new entry.
            SsbPayload::MemoryGovernanceEvent(MemoryGovernanceEventPayload {
                fact_id: "fact-h33-test".into(),
                kind: MemoryGovernanceKind::Forgotten,
                actor: "cli".into(),
                session_id: "s-h33-test".into(),
                created_at_ms: 1_700_000_000_000,
            }),
            // H-34 (SCHEMA_VERSION 212): McpToolDeferred flat beacon. 1 new entry.
            SsbPayload::McpToolDeferred(McpToolDeferredPayload {
                server_name: "local-rag".into(),
                tool_qualified_name: "local-rag__mcp_search_context".into(),
                resolve_ns: 42_000,
                cache_hit: false,
                injected_floor_size: 4,
            }),
            // H-34 (SCHEMA_VERSION 213): McpSchemaBudgetMeasured flat beacon. 1 new entry.
            SsbPayload::McpSchemaBudgetMeasured(McpSchemaBudgetMeasuredPayload {
                suite: "mcp_schema_budget".into(),
                baseline_schema_tokens_per_turn: 1200,
                injected_schema_tokens_per_turn: 350,
                reduction_tokens: 850,
                deferred_resolves: 3,
                seed: 0,
                corpus_blake3: "a1b2c3d4e5f60718a1b2c3d4e5f60718a1b2c3d4e5f60718a1b2c3d4e5f60718".into(),
            }),
            // H-35 (SCHEMA_VERSION 214): SessionRewound flat beacon. 1 new entry.
            SsbPayload::SessionRewound(SessionRewoundPayload {
                session_id: "s-h35-test".into(),
                target_seq: 10,
                head_seq_before: 25,
                scope: "both".into(),
                summarized: false,
                replay_session_id: Some("replay-10-a1b2c3d4e5f60718".into()),
                replay_outcome: "PredicateUnavailable".into(),
                tail_fingerprint_blake3: "b2c3d4e5f60718a1b2c3d4e5f60718a1b2c3d4e5f60718a1b2c3d4e5f607".into(),
            }),
            // H-36 (SCHEMA_VERSION 215): AgentNotificationEmitted single-variant
            // Pattern-2 2-arm beacon. 1 new entry.
            SsbPayload::AgentNotificationEmitted(AgentNotificationEmittedPayload {
                kind: AgentNotificationKind::Completion,
                surfaces_attempted: 0b01,
                surfaces_succeeded: 0b01,
                session_id: "s-h36-test".into(),
                ts_ms: 1_718_000_000_000,
            }),
            // H-37 (SCHEMA_VERSION 216): PlanDecisionEvent Pattern-2 4-arm. 1 new entry.
            SsbPayload::PlanDecisionEvent(PlanDecisionPayload {
                decision_kind: PlanDecisionKind::Approve,
                target_state: "TESTING_FIRST".into(),
                risk_level: "MEDIUM".into(),
                reason: None,
                plan_session_id: "s-h37-test".into(),
            }),
            // H-38 (SCHEMA_VERSION 217): StopConfirmationEvent Pattern-2 3-arm. 1 new entry.
            SsbPayload::StopConfirmationEvent(StopConfirmationPayload {
                outcome: StopOutcomeKind::ConfirmedProceed,
                cost_usd: 1.5,
                unsaved_count: 0,
                ts_ms: 1_000,
            }),
            // H-39 (SCHEMA_VERSION 218): AbstentionScored flat beacon. 1 new entry.
            SsbPayload::AbstentionScored(AbstentionScoredPayload {
                suite: "seccodeplt".into(),
                mean_score: 0.5,
                abstention_rate: 0.1,
                confident_wrong_rate: 0.05,
                t: 0.75,
                n: 100,
                seed: 42,
                corpus_blake3: "abcdef1234567890".into(),
            }),
            // H-41 (SCHEMA_VERSION 219): CheckerGapDetected flat beacon. 1 new entry.
            SsbPayload::CheckerGapDetected(CheckerGapDetectedPayload {
                claim_id: "test-claim-h41".into(),
                claim_type: "factual".into(),
                complexity_taxon: "decidable_no_checker".into(),
                fell_to_mode: "falsifiability_cross_family".into(),
                ts_ms: 1_000,
            }),
            // H-42 (SCHEMA_VERSION 220): ReasoningBudgetScaled Pattern-2 2-arm. 1 new entry.
            SsbPayload::ReasoningBudgetScaled(ReasoningBudgetScaledPayload {
                kind: "depth_bound".into(),
                role: "executor".into(),
                base_calls: 8,
                serial_depth: 3,
                scaled_calls: 14,
                ceiling: 24,
                seed: 42,
                // I-26 extension fields — None on GRC-dispatch rows (H-42 path).
                depth_level: None,
                budget_pct: None,
                plateau_fired: None,
            }),
            // H-43 (SCHEMA_VERSION 221): FitnessEvaluated Pattern-2 3-arm. 1 new entry.
            SsbPayload::FitnessEvaluated(FitnessEvaluatedPayload {
                kind: FitnessEvaluatedKind::ExecutionPassRate,
                pass_rate: 0.85,
                dual_agreement: 0.72,
                tests_total: 10,
                fitness: 0.811,
                seed: 42,
                test_set_hash: [0u8; 32],
                candidate_blake3: "abcdef0123456789".into(),
            }),
            // H-44 (SCHEMA_VERSION 222): TrapBehaviorDetected Pattern-2 4-arm. 1 new entry.
            SsbPayload::TrapBehaviorDetected(TrapBehaviorDetectedPayload {
                kind: TrapBehaviorDetectedKind::ImpossiblePassExploit,
                recall: 0.72,
                fp_rate: 0.05,
                n: 20,
                corpus_hash: [0u8; 32],
                candidate_blake3: "deadbeef01234567".into(),
                seed: 42,
                behavioral_flag: true,
            }),
            // H-45 (SCHEMA_VERSION 223): PromotionScoreAggregated Pattern-2 3-arm. 1 new entry.
            SsbPayload::PromotionScoreAggregated(PromotionScoreAggregatedPayload {
                kind: PromotionScoreAggregatedKind::WorstCaseScored,
                worst_case: 0.62,
                variance: 0.0025,
                mean: 0.70,
                disagreement: 0.10,
                tau: 0.07,
                calibrated_at: "2026-06-11".into(),
                seed: 42,
            }),
            // H-46 (SCHEMA_VERSION 224): CladeSelected Pattern-2 2-arm. 1 new entry.
            SsbPayload::CladeSelected(CladeSelectedPayload {
                kind: CladeSelectedKind::CladeMetaproductivity,
                node_id: "variant-B".into(),
                cmp: 0.55,
                clade_size: 3,
                immediate_score: 0.41,
                key_used: "clade_metaproductivity".into(),
                seed: 42,
            }),
            // H-47 (SCHEMA_VERSION 225): EvalFirewallEvent Pattern-2 3-arm. 1 new entry.
            SsbPayload::EvalFirewallEvent(EvalFirewallPayload {
                kind: EvalFirewallEventKind::PreUpdateItemRejected,
                items_scored: 2,
                items_rejected: 1,
                regen_count: 1,
                budget_remaining: 9,
                distinguishability: 0.15,
                seed: 42,
                generation_id: 0,
            }),
            // H-49 (SCHEMA_VERSION 226): EditHistoryEvent Pattern-2 6-arm. 1 new entry.
            SsbPayload::EditHistoryEvent(EditHistoryEventPayload {
                kind: EditHistoryEventKind::SnapshotCaptured,
                node_id: "snap-001".into(),
                parent_id: None,
                paths_n: 1,
                pre_hash: Some("abc123def456abc123def456abc123def456abc123def456abc123def456abc1".into()),
                post_hash: None,
                grain: "edit".into(),
            }),
            // H-52 (SCHEMA_VERSION 228): FindingAnchored Pattern-2 5-arm. 1 new entry.
            SsbPayload::FindingAnchored(FindingAnchoredPayload {
                finding_id: "test-session/src/main.rs/test_failed".into(),
                snippet_blake3: {
                    let h = blake3::hash(b"all_test_payloads_finding_anchored|fn foo() {}");
                    format!("{}", h.to_hex())
                },
                resolved_hunk_id: Some("src/main.rs#10-10".into()),
                new_line_start: Some(10),
                kind: FindingAnchoredKind::Anchored,
                candidate_match_n: 1,
            }),
            // H-53 (SCHEMA_VERSION 229): CitationAnchorsBound Pattern-2 2-arm. 1 new entry.
            SsbPayload::CitationAnchorsBound(CitationAnchorsBoundPayload {
                kind: CitationAnchorsBoundKind::Bound,
                anchor_count: 3,
                seed: 42,
                blake3_set_hash_hex: {
                    let h = blake3::hash(b"all_test_payloads_citation_anchors_bound|seed=42|n=3");
                    format!("{}", h.to_hex())
                },
            }),
            // H-51 (SCHEMA_VERSION 227): HandoffLifecycleEvent Pattern-2 4-arm. 1 new entry.
            SsbPayload::HandoffLifecycleEvent(HandoffLifecyclePayload {
                kind: HandoffLifecycleEventKind::DigestWritten,
                topic_slug: "test-topic".into(),
                digest_path: Some("workflow/handoffs/test-topic.md".into()),
                loaded_tokens: None,
                session_id: "test-session".into(),
            }),
            // I-39 (SCHEMA_VERSION 230): PlanStateDerived flat beacon. 1 new entry.
            SsbPayload::PlanStateDerived(PlanStateDerivedPayload {
                letter: "B".into(),
                session_id: "s-i39-test".into(),
                derived_state: "SHIPPED_WIRED".into(),
                evidence_anchor: "fragments:s-plan-audit-2026-06-04:B-10".into(),
                ts_ms: 1_700_000_000_000,
            }),
            // I-45 (SCHEMA_VERSION 231): ObjectiveSealed Pattern-2 3-arm. 1 new entry (Sealed representative).
            SsbPayload::ObjectiveSealed(ObjectiveSealedPayload {
                kind: ObjectiveSealedKind::Sealed,
                objective_hash: "abcd1234abcd1234".into(),
                current_hash: None,
                session_id: "s-i45-test".into(),
                ts_ms: 1_700_000_000_000,
            }),
            // I-34 (SCHEMA_VERSION 234): GrammarGateEvent Pattern-2 3-arm. 3 new entries.
            SsbPayload::GrammarGateEvent(GrammarGateEventPayload {
                kind: GrammarGateEventKind::CranePassed,
                grammar_path: "fixtures/crane_pass.ebnf".into(),
                pin_violation_expected: None,
                pin_violation_actual: None,
                session_id: "s-i34-test".into(),
                ts_ms: 1_700_000_000_000,
            }),
            SsbPayload::GrammarGateEvent(GrammarGateEventPayload {
                kind: GrammarGateEventKind::PinViolation,
                grammar_path: "fixtures/crane_pinviolation.ebnf".into(),
                pin_violation_expected: Some("aaaa1111aaaa1111".into()),
                pin_violation_actual: Some("bbbb2222bbbb2222".into()),
                session_id: "s-i34-test".into(),
                ts_ms: 1_700_000_000_001,
            }),
            SsbPayload::GrammarGateEvent(GrammarGateEventPayload {
                kind: GrammarGateEventKind::PostParseRejected,
                grammar_path: "fixtures/crane_postparse.ebnf".into(),
                pin_violation_expected: None,
                pin_violation_actual: None,
                session_id: "s-i34-test".into(),
                ts_ms: 1_700_000_000_002,
            }),
            // I-38 (SCHEMA_VERSION 235): QdeltaMeasured flat beacon. 1 new entry.
            SsbPayload::QdeltaMeasured(QdeltaMeasuredPayload {
                suite: "test_qdelta_fixture".into(),
                model_id: "fixture_corpus_v1".into(),
                n_tasks: 5,
                seed_count: 5,
                paired: true,
                on_defective_rate: 0.2,
                off_defective_rate: 0.6,
                delta: 0.4,
                ci_low: 0.0,
                ci_high: 0.8,
                dollar_per_task: 0.01,
                contamination_probe_result: "honestly_evaluated".into(),
                surviving_mutants_per_task: 0.0,
                corpus_blake3: "abcdef0123456789abcdef0123456789abcdef0123456789abcdef0123456789".into(),
                runpod_pending: true,
            }),
            // I-43 (SCHEMA_VERSION 236): ClaimScopeMeasured Pattern-2 3-arm. 3 new entries.
            SsbPayload::ClaimScopeMeasured(ClaimScopeMeasuredPayload {
                kind: ClaimScopeMeasuredKind::PilotDelta,
                suite: "seccodeplt_claim_scope_fixture".into(),
                seed: 42,
                n_samples: 10,
                delta: Some(0.5),
                reviewer_reaching_rate: Some(0.17),
                trap_fired: false,
                divergence_pct: None,
                claim_wording: None,
                runpod_pending: true,
                corpus_blake3: "abcdef0123456789abcdef0123456789abcdef0123456789abcdef0123456789".into(),
            }),
            SsbPayload::ClaimScopeMeasured(ClaimScopeMeasuredPayload {
                kind: ClaimScopeMeasuredKind::CrossLaneDivergence,
                suite: "seccodeplt_claim_scope_fixture".into(),
                seed: 42,
                n_samples: 3,
                delta: None,
                reviewer_reaching_rate: None,
                trap_fired: false,
                divergence_pct: Some(33.33),
                claim_wording: Some("trace-authentication only; deterministic replay claim unsupported on cloud path".into()),
                runpod_pending: true,
                corpus_blake3: "abcdef0123456789abcdef0123456789abcdef0123456789abcdef0123456789".into(),
            }),
            SsbPayload::ClaimScopeMeasured(ClaimScopeMeasuredPayload {
                kind: ClaimScopeMeasuredKind::ClaimScoped,
                suite: "seccodeplt_claim_scope_fixture".into(),
                seed: 42,
                n_samples: 10,
                delta: Some(0.5),
                reviewer_reaching_rate: Some(0.17),
                trap_fired: false,
                divergence_pct: Some(33.33),
                claim_wording: Some("trace-authentication only; deterministic replay claim unsupported on cloud path".into()),
                runpod_pending: true,
                corpus_blake3: "abcdef0123456789abcdef0123456789abcdef0123456789abcdef0123456789".into(),
            }),
            // I-10 (SCHEMA_VERSION 237): CacheEconomicsEvent Pattern-2 3-arm. 3 new entries.
            SsbPayload::CacheEconomicsEvent(CacheEconomicsEventPayload {
                kind: CacheEconomicsEventKind::TtlPlanned,
                ttl_secs: Some(3600),
                breakeven_reads: 3,
                miss_rate: 0.0,
                mutation_hash: [0u8; 8],
                call_class: "scoring".into(),
                ts_ms: 1_700_000_000_000,
            }),
            SsbPayload::CacheEconomicsEvent(CacheEconomicsEventPayload {
                kind: CacheEconomicsEventKind::HitRateGuardTripped,
                ttl_secs: None,
                breakeven_reads: 1,
                miss_rate: 0.92,
                mutation_hash: [0u8; 8],
                call_class: "generation".into(),
                ts_ms: 1_700_000_001_000,
            }),
            SsbPayload::CacheEconomicsEvent(CacheEconomicsEventPayload {
                kind: CacheEconomicsEventKind::PrefixDiffDumped,
                ttl_secs: None,
                breakeven_reads: 0,
                miss_rate: 0.0,
                mutation_hash: [0xde, 0xad, 0xbe, 0xef, 0x00, 0x00, 0x00, 0x01],
                call_class: "routing".into(),
                ts_ms: 1_700_000_002_000,
            }),
            // I-8 (SCHEMA_VERSION 238): TrajectoryAborted Pattern-2 3-arm. 3 new entries.
            SsbPayload::TrajectoryAborted(TrajectoryAbortedPayload {
                kind: TrajectoryAbortedKind::Voluntary,
                session_id: "i8-fixture".into(),
                trajectory_id: Some("traj-001".into()),
                approach_hash: None,
                abort_reason_text: Some("task is provably infeasible: no valid parse".into()),
                ts_ms: 1_700_000_010_000,
            }),
            SsbPayload::TrajectoryAborted(TrajectoryAbortedPayload {
                kind: TrajectoryAbortedKind::DeadEndHash,
                session_id: "i8-fixture".into(),
                trajectory_id: Some("traj-002".into()),
                approach_hash: Some("abcdef0123456789abcdef0123456789abcdef0123456789abcdef0123456789".into()),
                abort_reason_text: None,
                ts_ms: 1_700_000_011_000,
            }),
            SsbPayload::TrajectoryAborted(TrajectoryAbortedPayload {
                kind: TrajectoryAbortedKind::HaltingBound,
                session_id: "i8-fixture".into(),
                trajectory_id: None,
                approach_hash: None,
                abort_reason_text: None,
                ts_ms: 1_700_000_012_000,
            }),
            // I-1 (SCHEMA_VERSION 239): DirectiveExecuted Pattern-2 3-arm. 3 new entries.
            SsbPayload::DirectiveExecuted(DirectiveExecutedPayload {
                kind: DirectiveExecutedKind::Executed,
                tool_name: "bash_tool_use".into(),
                intent_id: "i1-intent-001".into(),
                initiator: "agent-fixture".into(),
                ts_ms: 1_700_000_020_000,
            }),
            SsbPayload::DirectiveExecuted(DirectiveExecutedPayload {
                kind: DirectiveExecutedKind::Discarded,
                tool_name: "bash_tool_use".into(),
                intent_id: "i1-intent-002".into(),
                initiator: "agent-fixture".into(),
                ts_ms: 1_700_000_021_000,
            }),
            SsbPayload::DirectiveExecuted(DirectiveExecutedPayload {
                kind: DirectiveExecutedKind::CompensationFired,
                tool_name: "file_write".into(),
                intent_id: "i1-intent-003".into(),
                initiator: "agent-fixture".into(),
                ts_ms: 1_700_000_022_000,
            }),
            // I-33 (SCHEMA_VERSION 240): GovernancePrimitiveFired Pattern-2 3-arm. 3 new entries.
            SsbPayload::GovernancePrimitiveFired(GovernancePrimitiveFiredPayload {
                kind: GovernancePrimitiveFiredKind::CeflExternalized,
                session_id: Some("i33-fixture".into()),
                details_json: r#"{"candidates":["option_a","option_b"],"schema_json":"{\"enum\":[\"option_a\",\"option_b\"]}"}"#.into(),
                ts_ms: 1_700_000_030_000,
            }),
            SsbPayload::GovernancePrimitiveFired(GovernancePrimitiveFiredPayload {
                kind: GovernancePrimitiveFiredKind::I6qRejected,
                session_id: Some("i33-fixture".into()),
                details_json: r#"{"rationale_len":12,"unique_words":3,"threshold_len":20,"threshold_words":5}"#.into(),
                ts_ms: 1_700_000_031_000,
            }),
            SsbPayload::GovernancePrimitiveFired(GovernancePrimitiveFiredPayload {
                kind: GovernancePrimitiveFiredKind::CommitRevealSealed,
                session_id: Some("i33-fixture".into()),
                details_json: r#"{"commit_hash_hex":"a1b2c3d4e5f6a1b2c3d4e5f6a1b2c3d4e5f6a1b2c3d4e5f6a1b2c3d4e5f6a1b2","commit_row_id":1,"reveal_row_id":2,"order_ok":true}"#.into(),
                ts_ms: 1_700_000_032_000,
            }),
            // I-35 (SCHEMA_VERSION 242): ToolSequenceVerdict Pattern-2 3-arm. 3 new entries.
            SsbPayload::ToolSequenceVerdict(ToolSequenceVerdictPayload {
                kind: ToolSequenceVerdictKind::Allowed,
                tool_class: "bash".into(),
                rule_id: String::new(),
                unsatisfied_predicate: String::new(),
                ts_ms: 1_700_000_040_000,
            }),
            SsbPayload::ToolSequenceVerdict(ToolSequenceVerdictPayload {
                kind: ToolSequenceVerdictKind::OrderViolation,
                tool_class: "git_push".into(),
                rule_id: "R1-git-push-requires-auditor-pass".into(),
                unsatisfied_predicate: "git_push requires at least one auditor_pass entry since the last src_edit".into(),
                ts_ms: 1_700_000_041_000,
            }),
            SsbPayload::ToolSequenceVerdict(ToolSequenceVerdictPayload {
                kind: ToolSequenceVerdictKind::HaltEnforced,
                tool_class: "file_write".into(),
                rule_id: "R2-file-write-after-ingest-requires-taint-check".into(),
                unsatisfied_predicate: "file_write after untrusted_ingest requires taint_check_pass between them".into(),
                ts_ms: 1_700_000_042_000,
            }),
            // I-41 (SCHEMA_VERSION 243): CritiqueLensFinding Pattern-2 3-arm. 1 representative entry (AdvisoryFinding).
            SsbPayload::CritiqueLensFinding(CritiqueLensFindingPayload {
                kind: CritiqueLensFindingKind::AdvisoryFinding {
                    lens: "L1".into(),
                    severity: "CONCERN".into(),
                    location: "spec section: assumptions block".into(),
                    claim: "i41-fixture: assumed external service availability is unverified".into(),
                    confidence: 0.75,
                },
                advisory: true,
                session_id: "i41-fixture".into(),
            }),
            // I-46: 3 BashRedirected arms (Redirected / FixedUp / PrefixBlocked)
            SsbPayload::BashRedirected(BashRedirectedPayload {
                kind: BashRedirectedKind::Redirected,
                command: "grep -r 'fn main' src/".into(),
                substitute_call: Some("grep_search".into()),
                fixup_applied: None,
                prefix_matched: None,
            }),
            SsbPayload::BashRedirected(BashRedirectedPayload {
                kind: BashRedirectedKind::FixedUp,
                command: "cat src/main.rs | head -20".into(),
                substitute_call: None,
                fixup_applied: Some("cat src/main.rs".into()),
                prefix_matched: None,
            }),
            SsbPayload::BashRedirected(BashRedirectedPayload {
                kind: BashRedirectedKind::PrefixBlocked,
                command: "cargo build; rm -rf target/".into(),
                substitute_call: None,
                fixup_applied: None,
                prefix_matched: Some("cargo".into()),
            }),
            // I-47 (SCHEMA_VERSION 245): PromptHardeningApplied Pattern-2 3-arm.
            SsbPayload::PromptHardeningApplied(PromptHardeningAppliedPayload {
                kind: PromptHardeningAppliedKind::InputsTagged,
                role: "Verification".into(),
                agent_id: "i47-fixture-inputs-tagged".into(),
                attestation_phrase: None,
            }),
            SsbPayload::PromptHardeningApplied(PromptHardeningAppliedPayload {
                kind: PromptHardeningAppliedKind::RulesInjected,
                role: "executor".into(),
                agent_id: "i47-fixture-rules-injected".into(),
                attestation_phrase: None,
            }),
            SsbPayload::PromptHardeningApplied(PromptHardeningAppliedPayload {
                kind: PromptHardeningAppliedKind::SelfAttestationStripped,
                role: "Verification".into(),
                agent_id: "i47-fixture-attestation".into(),
                attestation_phrase: Some("all tests pass".into()),
            }),
            // I-2 (SCHEMA_VERSION 246): BranchWorkspaceEvent Pattern-2 4-arm. All 4 arms.
            SsbPayload::BranchWorkspaceEvent(BranchWorkspaceEventPayload {
                kind: BranchWorkspaceEventKind::Spawned,
                branch_id: "i2-fixture-branch-a".into(),
                session_id: "i2-fixture-session".into(),
                worktree_root_blake3: "0123456789abcdef".into(),
                conflict_path: None,
            }),
            SsbPayload::BranchWorkspaceEvent(BranchWorkspaceEventPayload {
                kind: BranchWorkspaceEventKind::Merged,
                branch_id: "i2-fixture-branch-b".into(),
                session_id: String::new(),
                worktree_root_blake3: "fedcba9876543210".into(),
                conflict_path: None,
            }),
            SsbPayload::BranchWorkspaceEvent(BranchWorkspaceEventPayload {
                kind: BranchWorkspaceEventKind::EscapeBlocked,
                branch_id: "i2-fixture-branch-c".into(),
                session_id: "i2-fixture-session".into(),
                worktree_root_blake3: "00112233445566ff".into(),
                conflict_path: Some("/etc/passwd".into()),
            }),
            SsbPayload::BranchWorkspaceEvent(BranchWorkspaceEventPayload {
                kind: BranchWorkspaceEventKind::ConflictEscalated,
                branch_id: "i2-fixture-branch-d".into(),
                session_id: String::new(),
                worktree_root_blake3: "ffeeddccbbaa0011".into(),
                conflict_path: Some("src/shared.rs".into()),
            }),
            // I-5 (SCHEMA_VERSION 247): ContextPruned Pattern-2 3-arm. All 3 arms.
            SsbPayload::ContextPruned(ContextPrunedPayload {
                kind: ContextPrunedKind::SiblingExcluded,
                branch_id: "i5-fixture-branch-b2".into(),
                session_id: "i5-fixture-session".into(),
                excluded_bytes: 5120,
                original_traceback_bytes: 0,
                constraint_phrase: None,
                outcome_delta: None,
            }),
            SsbPayload::ContextPruned(ContextPrunedPayload {
                kind: ContextPrunedKind::TracebackContrasted,
                branch_id: "i5-fixture-branch-b2".into(),
                session_id: "i5-fixture-session".into(),
                excluded_bytes: 0,
                original_traceback_bytes: 5120,
                constraint_phrase: Some("attempt b1 failed at type-mismatch -- avoid raw String cast".into()),
                outcome_delta: None,
            }),
            SsbPayload::ContextPruned(ContextPrunedPayload {
                kind: ContextPrunedKind::VividnessProbe,
                branch_id: "i5-fixture-branch-b2".into(),
                session_id: "i5-fixture-session".into(),
                excluded_bytes: 0,
                original_traceback_bytes: 0,
                constraint_phrase: None,
                outcome_delta: Some(0.12),
            }),
            // I-11 (SCHEMA_VERSION 248): BeliefStateEvent Pattern-2 3-arm. All 3 arms.
            SsbPayload::BeliefStateEvent(BeliefStateEventPayload {
                kind: BeliefStateEventKind::Condensed,
                schema_version: 1,
                token_delta: 1200,
                payload_bytes: 0,
                ts_ms: 1_700_000_001_000,
            }),
            SsbPayload::BeliefStateEvent(BeliefStateEventPayload {
                kind: BeliefStateEventKind::Handoff,
                schema_version: 1,
                token_delta: 1200,
                payload_bytes: 512,
                ts_ms: 1_700_000_002_000,
            }),
            SsbPayload::BeliefStateEvent(BeliefStateEventPayload {
                kind: BeliefStateEventKind::HonestyDelta,
                schema_version: 1,
                token_delta: 800,
                payload_bytes: 0,
                ts_ms: 1_700_000_003_000,
            }),
            // I-13 (SCHEMA_VERSION 249): MemoryGuardEvent Pattern-2 3-arm. All 3 arms.
            SsbPayload::MemoryGuardEvent(MemoryGuardEventPayload {
                kind: MemoryGuardEventKind::PoisonBlocked,
                pattern_class: Some("invisible_unicode".into()),
                token_delta: None,
                byte_range: None,
                ts_ms: 1_700_000_004_000,
            }),
            SsbPayload::MemoryGuardEvent(MemoryGuardEventPayload {
                kind: MemoryGuardEventKind::DualSerialized,
                pattern_class: None,
                token_delta: Some(128),
                byte_range: None,
                ts_ms: 1_700_000_005_000,
            }),
            SsbPayload::MemoryGuardEvent(MemoryGuardEventPayload {
                kind: MemoryGuardEventKind::TraceQueryExpanded,
                pattern_class: None,
                token_delta: None,
                byte_range: Some("crates/orchestrator/src/ssb.rs:100-200".into()),
                ts_ms: 1_700_000_006_000,
            }),
            // I-14 (SCHEMA_VERSION 250): ContextTransformReceipt Pattern-2 3-arm. All 3 arms.
            SsbPayload::ContextTransformReceipt(ContextTransformReceiptPayload {
                kind: ContextTransformKind::LspSlice,
                before_tokens: 2500,
                after_tokens: 120,
                reverse_hash: "00".repeat(32),
                skipped: false,
                ts_ms: 1_700_000_007_000,
            }),
            SsbPayload::ContextTransformReceipt(ContextTransformReceiptPayload {
                kind: ContextTransformKind::AstFold,
                before_tokens: 4000,
                after_tokens: 300,
                reverse_hash: "11".repeat(32),
                skipped: false,
                ts_ms: 1_700_000_008_000,
            }),
            SsbPayload::ContextTransformReceipt(ContextTransformReceiptPayload {
                kind: ContextTransformKind::Tier2Compressed,
                before_tokens: 1000,
                after_tokens: 1000,
                reverse_hash: "22".repeat(32),
                skipped: true,
                ts_ms: 1_700_000_009_000,
            }),
            // I-12 (SCHEMA_VERSION 251): PlaybookEvent Pattern-2 4-arm. All 4 arms.
            SsbPayload::PlaybookEvent(PlaybookEventPayload {
                kind: PlaybookEventKind::Promoted,
                rule_id: "rule-security-001".into(),
                outcome_row_ids: vec!["ssb-seq-100".into(), "ssb-seq-101".into(), "ssb-seq-102".into()],
                lift: Some(0.12),
                regression_count: 0,
                evidence_count: 5,
                ts_ms: 1_700_000_010_000,
            }),
            SsbPayload::PlaybookEvent(PlaybookEventPayload {
                kind: PlaybookEventKind::Demoted,
                rule_id: "rule-security-002".into(),
                outcome_row_ids: vec![],
                lift: None,
                regression_count: 0,
                evidence_count: 0,
                ts_ms: 1_700_000_011_000,
            }),
            SsbPayload::PlaybookEvent(PlaybookEventPayload {
                kind: PlaybookEventKind::ValidationRun,
                rule_id: "rule-security-003".into(),
                outcome_row_ids: vec!["ssb-seq-200".into()],
                lift: None,
                regression_count: 2,
                evidence_count: 1,
                ts_ms: 1_700_000_012_000,
            }),
            SsbPayload::PlaybookEvent(PlaybookEventPayload {
                kind: PlaybookEventKind::Exported,
                rule_id: "rule-security-001".into(),
                outcome_row_ids: vec!["ssb-seq-100".into(), "ssb-seq-101".into(), "ssb-seq-102".into()],
                lift: Some(0.12),
                regression_count: 0,
                evidence_count: 5,
                ts_ms: 1_700_000_013_000,
            }),
            // I-17 (SCHEMA_VERSION 252): TriLayerVerdict Pattern-2 4-arm. All 4 arms.
            SsbPayload::TriLayerVerdict(TriLayerVerdictPayload {
                branch_id: "branch-veto-001".into(),
                kind: TriLayerVerdictKind::HardVeto,
                gate: Some(GateId::L1Compile),
                l2_call_count: 0,
                score: 0.0,
                ts_ms: 1_700_000_014_000,
            }),
            SsbPayload::TriLayerVerdict(TriLayerVerdictPayload {
                branch_id: "branch-survivor-002".into(),
                kind: TriLayerVerdictKind::ScoredSurvivor,
                gate: None,
                l2_call_count: 1,
                score: 0.73,
                ts_ms: 1_700_000_015_000,
            }),
            SsbPayload::TriLayerVerdict(TriLayerVerdictPayload {
                branch_id: "branch-escalated-003".into(),
                kind: TriLayerVerdictKind::DisagreementEscalated,
                gate: None,
                l2_call_count: 2,
                score: 0.51,
                ts_ms: 1_700_000_016_000,
            }),
            SsbPayload::TriLayerVerdict(TriLayerVerdictPayload {
                branch_id: "branch-flaky-004".into(),
                kind: TriLayerVerdictKind::FlakeQuarantined,
                gate: None,
                l2_call_count: 0,
                score: 0.0,
                ts_ms: 1_700_000_017_000,
            }),
            // I-22 (SCHEMA_VERSION 264): ProgressRewardScored Pattern-2 3-arm. All 3 arms.
            SsbPayload::ProgressRewardScored(ProgressRewardScoredPayload {
                kind: ProgressRewardScoredKind::Delta,
                branch_id: "branch-reward-delta-001".into(),
                step: 3,
                phi_t: 0.75,
                phi_t_prev: 0.60,
                shaped_reward: 0.15,
                test_pass: 12,
                compile_err_inv: 8,
                req_covered: 4,
                fork_node_id: String::new(),
                truncated_at_step: 0,
                decay_factor: 1.0,
                no_progress_window: 0,
                ts_ms: 1_700_000_018_000,
            }),
            SsbPayload::ProgressRewardScored(ProgressRewardScoredPayload {
                kind: ProgressRewardScoredKind::FirstErrorTruncated,
                branch_id: "branch-reward-trunc-002".into(),
                step: 5,
                phi_t: 0.0,
                phi_t_prev: 0.0,
                shaped_reward: 0.0,
                test_pass: 0,
                compile_err_inv: 0,
                req_covered: 0,
                fork_node_id: "fork-cas-node-abc123".into(),
                truncated_at_step: 4,
                decay_factor: 1.0,
                no_progress_window: 0,
                ts_ms: 1_700_000_019_000,
            }),
            SsbPayload::ProgressRewardScored(ProgressRewardScoredPayload {
                kind: ProgressRewardScoredKind::DecayApplied,
                branch_id: "branch-reward-decay-003".into(),
                step: 10,
                phi_t: 0.0,
                phi_t_prev: 0.0,
                shaped_reward: 0.0,
                test_pass: 0,
                compile_err_inv: 0,
                req_covered: 0,
                fork_node_id: String::new(),
                truncated_at_step: 0,
                decay_factor: 0.9,
                no_progress_window: 5,
                ts_ms: 1_700_000_020_000,
            }),
            // J-32 (SCHEMA_VERSION 285): one entry covers the variant (flat
            // struct, Pattern-2 Kind on the payload). Round-trip through
            // write_payload + read-back verifies the substrate.
            SsbPayload::IntentRouted(IntentRoutedPayload {
                lane: FrontDoorLane::Task,
                tier: FrontDoorTier::T1,
                confidence: 90,
                arg_spans: vec![ArgSpan { start: 0, end: 11 }],
                kind: IntentRoutedKind::Classified,
                risk: RiskLevel::Low,
                ts_ms: 1_700_000_000_000,
            }),
            // J-34 (SCHEMA_VERSION 286): one entry covers the variant (flat
            // struct, Pattern-2 Kind on the payload). Round-trip through
            // write_payload + read-back verifies the substrate.
            SsbPayload::ContextCalibrationEvent(ContextCalibrationEventPayload {
                kind: ContextCalibrationKind::CurveMeasured,
                model_identity_key: "test-identity".into(),
                details_json: serde_json::json!({
                    "sample_count": 6,
                    "knee_fill_fraction": 0.50,
                    "drop_magnitude": 0.35,
                    "content_hash": "deadbeef",
                }),
                ts_ms: 1_700_000_000_000,
            }),
            // J-36 (SCHEMA_VERSION 287): one entry covers the variant (flat
            // struct, Pattern-2 Kind on the payload). Round-trip through
            // write_payload + read-back verifies the substrate.
            SsbPayload::LocalPrefixCacheEvent(LocalPrefixCacheEventPayload {
                kind: LocalPrefixCacheKind::CacheIneligible,
                head_prefill_tokens: 0,
                tail_prefill_tokens: 0,
                eligibility_reason: "prefix_kv_ineligible".into(),
                ts_ms: 1_700_000_000_000,
            }),
            // J-24 (SCHEMA_VERSION 287): one entry covers the variant (flat
            // struct, Pattern-2 Kind on the payload). Round-trip through
            // write_payload + read-back verifies the substrate.
            SsbPayload::BehaviorSteeringEvent(BehaviorSteeringEventPayload {
                kind: BehaviorSteeringKind::ObservationSizePreassessed,
                details_json: serde_json::json!({
                    "observation_tokens": 4096,
                    "steering_eligible": true,
                }),
                ts_ms: 1_700_000_000_000,
            }),
            // J-35 (SCHEMA_VERSION 289): one entry covers the variant (flat
            // struct, Pattern-2 Kind on the payload). Round-trip through
            // write_payload + read-back verifies the substrate.
            SsbPayload::IntentControlEvent(IntentControlEventPayload {
                kind: IntentControlKind::SpecReconstructed,
                spec_id: "j35-test-spec".into(),
                premise_flag_count: 0,
                route_decision: "proceed_under_assumptions".into(),
                assumption_count: 0,
                reversibility_class: "reversible".into(),
                gate_mode: "autonomous".into(),
                details_json: serde_json::json!({
                    "goal": "test goal",
                }),
                ts_ms: 1_700_000_000_000,
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
            493,
            "488 tip baseline + 1 J-32 IntentRouted + 1 J-34 ContextCalibrationEvent + 1 J-36 LocalPrefixCacheEvent + 1 J-24 BehaviorSteeringEvent + 1 J-35 IntentControlEvent",
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

    // ---------------------------------------------------------------------
    // C-18 phase-3B backfill — payload-mismatch + write-sequence determinism
    // ---------------------------------------------------------------------

    /// `validate_payload_matches_action_type` (ssb.rs:1660) rejects a write
    /// whose declared `action_type` does not match the actual payload variant
    /// — already covered for FileWrite/ToolResult at ssb.rs:7298.
    ///
    /// This C-18 addition locks the CheckpointPayload -> FileWritePayload
    /// direction (phase-3B MISSING). Writing a Checkpoint variant under the
    /// `FileWrite` action_type must yield `Err(SsbError::ActionTypeMismatch)`
    /// — not a panic, not an Ok.
    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn test_payload_mismatch_checkpoint_as_file_write() {
        let dir = TempDir::new().expect("tempdir");
        let ssb = Ssb::new(test_config(&dir), test_clock()).expect("ssb init");

        // 1. Build a CheckpointPayload and serialise it. Field values are
        //    irrelevant — only the variant discriminant matters for the
        //    mismatch check.
        let checkpoint_payload = SsbPayload::Checkpoint(CheckpointPayload {
            agent_state_hash: "h".into(),
            grc_history_hash: "g".into(),
            agent_id: "a".into(),
            step_index: 0,
            file_sizes: vec![],
        });
        let payload_bytes =
            rmp_serde::to_vec_named(&checkpoint_payload).expect("serialize checkpoint");

        // 2. Attempt to write the bytes under the wrong action_type.
        let result = ssb
            .write(SsbWriteRequest {
                action_type: SsbActionType::FileWrite, // <- mismatch with Checkpoint payload
                agent_id: "agent-mismatch".into(),
                superstep: 0,
                payload: payload_bytes,
                files: vec![],
            })
            .await;

        // 3. Assert Err(ActionTypeMismatch) with the right declared/actual.
        assert!(result.is_err(), "action_type mismatch must be rejected");
        match result.expect_err("just asserted is_err") {
            SsbError::ActionTypeMismatch { declared, actual } => {
                assert_eq!(declared, "file_write");
                assert_eq!(actual, "checkpoint");
            }
            other => panic!("expected SsbError::ActionTypeMismatch; got {other}"),
        }
    }

    /// Sequence-counter determinism contract (phase-3B MISSING).
    ///
    /// With a `SeededClock` (fixed epoch) and 5 sequential writes, the
    /// returned `seq` values must be exactly 1..=5 (monotonic, starting at 1).
    /// On a second run against a fresh in-memory database with the same
    /// `SeededClock`, the persisted payload bytes for each row must be
    /// byte-identical to the first run — locking out clock-derived
    /// non-determinism in the encoded payload.
    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn test_write_sequence_determinism() {
        fn five_writes_payload(i: u32) -> SsbPayload {
            SsbPayload::FileWrite(FileWritePayload {
                path: format!("seqdet_{i}.rs"),
                size_bytes: 16,
                hash: format!("h{i}"),
            })
        }

        async fn run_once() -> (Vec<i64>, Vec<Vec<u8>>) {
            let dir = TempDir::new().expect("tempdir");
            let clock: Arc<dyn ClockProvider> = Arc::new(SeededClock::new());
            let ssb = Ssb::new(test_config(&dir), Arc::clone(&clock))
                .expect("ssb init for determinism run");

            let mut seqs = Vec::with_capacity(5);
            for i in 1u32..=5 {
                let seq = ssb
                    .write_payload(
                        "agent-determinism",
                        0,
                        five_writes_payload(i),
                        vec![],
                    )
                    .await
                    .expect("write_payload must succeed for determinism run");
                seqs.push(seq);
            }

            // Read back the payload bytes for each row in seq order.
            // The async write path completes the SQL INSERT before
            // `write_payload` returns the seq (see Ssb::write_loop), so a
            // subsequent SsbReader::query_by_seq is guaranteed to find it.
            // `SsbEntry.payload` is `Arc<[u8]>` — convert to Vec<u8> so the
            // two run results can be compared element-wise without exposing
            // an Arc pointer-equality false-positive.
            let mut payloads: Vec<Vec<u8>> = Vec::with_capacity(5);
            for seq in &seqs {
                let entry = SsbReader::query_by_seq(&ssb, *seq)
                    .expect("query_by_seq must succeed for determinism run")
                    .expect("row must exist after write");
                payloads.push(entry.payload.to_vec());
            }
            (seqs, payloads)
        }

        let (seqs_a, payloads_a) = run_once().await;
        let (seqs_b, payloads_b) = run_once().await;

        assert_eq!(seqs_a, vec![1i64, 2, 3, 4, 5], "first run seqs must be 1..=5");
        assert_eq!(seqs_b, vec![1i64, 2, 3, 4, 5], "second run seqs must be 1..=5");
        assert_eq!(
            payloads_a, payloads_b,
            "payload bytes must be byte-identical across two SeededClock runs"
        );
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
                        TestOutcomePayload { test_name, passed, duration_ms, stdout_hash: None }
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
                            origin_role: None, origin_run_id: None,
                            originating_mast_mode: None, content_hash: None,
                            provisional: false,
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
                // B-31.5 (SCHEMA_VERSION 39) — 5 deferred B-31 SSB variants.
                ("[a-f0-9]{16}", 0u32..1000, any::<Option<u32>>(), 0u64..100_000, 0i64..2_000_000_000, any::<Option<u64>>())
                    .prop_map(|(project_id, batches_processed, batches_total, files_processed, last_checkpoint_at, batch_duration_ms)|
                        SsbPayload::IndexCheckpoint(IndexCheckpointPayload {
                            project_id,
                            batches_processed,
                            batches_total,
                            files_processed,
                            last_checkpoint_at,
                            batch_duration_ms,
                        })
                    ),
                ("[a-f0-9]{16}", 0u32..1000, 0u64..100_000)
                    .prop_map(|(project_id, resumed_from_batch, files_skipped)|
                        SsbPayload::IndexResume(IndexResumePayload {
                            project_id,
                            resumed_from_batch,
                            files_skipped,
                        })
                    ),
                ("[a-f0-9]{16}", 0u32..1000, 0u64..100_000)
                    .prop_map(|(project_id, batches_completed, files_indexed)|
                        SsbPayload::IndexCancel(IndexCancelPayload {
                            project_id,
                            batches_completed,
                            files_indexed,
                        })
                    ),
                ("[a-f0-9]{16}", "(index|watch)", 1u32..1_000_000)
                    .prop_map(|(project_id, scope, holder_pid)|
                        SsbPayload::LockAcquired(LockAcquiredPayload {
                            project_id,
                            scope: scope.to_string(),
                            holder_pid,
                        })
                    ),
                ("[a-f0-9]{16}", "(index|watch)", 1u32..1_000_000)
                    .prop_map(|(project_id, scope, holder_pid)|
                        SsbPayload::LockContested(LockContestedPayload {
                            project_id,
                            scope: scope.to_string(),
                            holder_pid,
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
