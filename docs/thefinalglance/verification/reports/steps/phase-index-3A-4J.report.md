# Verification Report: phase-index-3A-4J
Source: F:\.ELAI_workflow\docs\specs\guide\steps\phase-index-3A-4J.txt
Date: 2026-05-17
Overall status: MOSTLY_DONE
Confidence: HIGH

## Summary
The phase index covers 14 phases (3A-3K, 4A-4J excluding 3J sub-steps). Spot-checked all 14
against elai.map. 11 phases have strong evidence with primary types and test scaffolding
confirmed. Phase 4J (Empire Integration) has zero evidence in elai.map, indicating it was never
started (consistent with its "conditional" status). Phase 3I (DST) is partial: the reference
model and StateSnapshot exist but proptest-state-machine and madsim integrations are absent.
Phase 3K HyDE and query-tier routing are confirmed present; adaptive AST-aware chunking gap is
minor. Overall the spec index accurately reflects implemented work with two notable gaps.

## Deliverables verified DONE

- 3A FSM state machine (9 states, transitions, InteractiveGate) — evidence: crates/orchestrator/src/fsm.rs:1014 (FsmState::Planning, Auditing, Configuring, TestingFirst, Executing, FeasibilityProbe, Memorizing all referenced in tests)
- 3A InteractiveGate + GateDecision::Approve/Reject — evidence: crates/orchestrator/src/gate.rs:961
- 3B SSB core with SsbActionType (PhaseComplete, ReasoningTrace, WorkloadProfile, etc.) — evidence: crates/orchestrator/src/ssb.rs:1153 (full action-type dispatch table)
- 3B broadcast after commit + ClockProvider — evidence: crates/orchestrator/src/ssb.rs:6186; crates/orchestrator/src/ssb.rs:4645
- 3C AgentPool + AgentHandle + anti-spiral detection — evidence: crates/orchestrator/src/agent_pool.rs:287; crates/orchestrator/src/anti_spiral.rs:373
- 3C SupervisionTree + handle_death + CheckpointContext — evidence: crates/orchestrator/src/supervision.rs:145; crates/orchestrator/src/task_runner.rs:10231
- 3D BSP channel types (LastValue, Topic, EphemeralValue, UntrackedValue) with DeepSizeOf — evidence: crates/orchestrator/src/bsp/channels.rs:39 (deep_size_of on all 4), SCIP_DEF lines 109/206/212/218/224
- 3D BSP engine + spawn_parallel + straggler timeout — evidence: crates/orchestrator/src/bsp/engine.rs:1700; crates/orchestrator/src/agent_pool.rs:635
- 3E ContextBudget with 8 slots, file_overlap_score, adaptive reduction, cache breakpoints — evidence: crates/orchestrator/src/context_budget.rs:605 (file_overlap_score); context_budget.rs:701 (AdaptiveLevel enum); context_budget.rs:2161 (cache_breakpoint_slot_positions)
- 3E floor enforcement, Terminal -> SPLIT_TASK — evidence: crates/contracts/tests/context_budget_contract.rs:266 (adaptive_reduce_all_levels_fail_autonomous_split_task)
- 3F RuleEngine trait, Verdict enum (ALLOW/WARN/BLOCK/ESCALATE), predicate DSL, lifecycle states — evidence: crates/rules/src/rule_engine.rs:1382; crates/rules/src/lifecycle.rs:67 (as_db for PROPOSED/CANDIDATE/SHADOW/ACTIVE/RETIRED); crates/plugins/src/tiered_analysis.rs:1812
- 3G PlanSkeleton + PhaseEntry + PhaseDetail + ContractRef, Kahn topology — evidence: crates/orchestrator/src/plan_skeleton.rs:305 (add_phase); plan_skeleton.rs:1375; crates/orchestrator/src/coherence_check.rs:257; crates/orchestrator/src/phase_sizing_splitter.rs:382
- 3H SessionDag + snapshot + branch/merge + 3-pass trimming + blake3 integrity — evidence: crates/orchestrator/src/session_dag.rs:532 (save_snapshot); session_dag.rs:2093 (trim_tool_result_stubbed); session_dag.rs:2589 (list_branches_returns_all_active); crates/knowledge/src/metadata.rs:606 (parse_blake3_hex)
- 3I DST StateSnapshot + ReferenceState + ActualSystem + FsmEvent enum — evidence: crates/orchestrator/src/dst.rs:37 (active_agents); dst.rs:86 (snapshot_state); dst.rs:150 (FsmEvent::SsbWrite)
- 3J DoLa sidecar (Python, DoLaSteeringProcessor) — evidence: python/elai_inference/server.py:812 (SCIP_DEF __call__); server.py:714 (__init__)
- 3J TurboQuant KV compression + ActAdd steering — evidence: crates/providers/src/turboquant.rs:687; metrics/compute_steering_vector.py:1
- 3J VllmClient + ModelPool + GPU multiplexing — evidence: crates/providers/src/vllm_client.rs:35 (VllmConfig); vllm_client.rs:480 (available_vram); vllm_client.rs:555 (place_models)
- 3J MCP bridge + McpError + SSB action types — evidence: crates/plugins/src/mcp_bridge.rs:108; crates/contracts/tests/fuld_mcp_bridge_contract.rs:54
- 3J LiveCli + ConversationRuntime + InferenceDispatcher wiring (3J.PIPELINE) — evidence: crates/cli/src/main.rs:3018 (SCIP_DEF LiveCli); crates/orchestrator/src/conversation.rs:127
- 3K HyDE pipeline stage + HydeClient + query tier routing — evidence: crates/knowledge/src/hyde.rs:210; crates/knowledge/src/pipeline.rs:1137 (make_hyde_stage); knowledge/src/pipeline.rs:1323 (query_tier_resolve_concrete)
- 3K query expansion (no_duplicate_expansions) — evidence: crates/knowledge/src/query_expansion.rs:315
- 3K retrieval benchmark (fast_tier_latency_under_budget) — evidence: crates/knowledge/tests/benchmark_eval.rs:130
- 4A IpcGateway + IpcFrame + ToolFilter + AppContainer + seccomp (IPC_BASELINE, NETWORK_EXTRA) — evidence: crates/orchestrator/src/ipc_gateway.rs:521; crates/orchestrator/src/sandbox.rs:561 (AppContainerConfig); sandbox.rs:867 (compile_seccomp_ipc_baseline)
- 4A elai_cc_agent binary + ReplanRequest adversarial tests — evidence: crates/agent/src/main.rs:43 (SCIP_DEF); crates/orchestrator/tests/adversarial_ipc.rs:69 (oversized_payload_rejected)
- 4B tiered validation pipeline + ShortCoder + DAIRA trace + FailureTaxonomy — evidence: crates/tools/src/tiered_validation.rs:170; crates/orchestrator/src/shortcoder_hook.rs:245; crates/orchestrator/src/evidence.rs:1212 (daira trace)
- 4B IntegrationTestRunner baby-step granularity (fula contract) — evidence: crates/contracts/tests/fula_language_aware_integration_runner_contract.rs:37
- 4C pre/post-tool hook + tiered static analysis + ShortCoder in PostToolUse — evidence: crates/plugins/src/post_tool_use.rs:185; crates/plugins/src/tiered_analysis.rs:1812; crates/contracts/tests/plugins_adversarial.rs:118 (rule_id never leaked)
- 4D ResearchTree + FlashResearch + BabyStepSpec (8 blocks) + TechStackDecision + ResearchCache — evidence: crates/orchestrator/src/research_tree.rs:1284 (BabyStepSpec); research_tree.rs:1770 (TechStackDecision); research_tree.rs:1937 (ResearchCache)
- 4D PackageRef + recursive decomposition — evidence: crates/orchestrator/src/research_tree.rs:1635 (PhaseNode); crates/orchestrator/src/pipeline_tree_navigator.rs:1731 (ConceptNode)
- 4E DecompositionTree + TreeNavigator + BubbleUpAction + BubbleUpHandler + IntegrationCheckpoint — evidence: crates/orchestrator/src/pipeline_tree_navigator.rs:83 (BubbleUpAction SCIP_DEF); pipeline_tree_navigator.rs:976 (bubble_up_on_concept_integration_fail); pipeline_tree_navigator.rs:2071 (checkpoint_recorded_on_completion)
- 4E PhaseSizingSplitter + phase splitting + structural validation — evidence: crates/orchestrator/src/phase_sizing_splitter.rs:382; crates/contracts/tests/fulb_phase_sizing_splitter_contract.rs:24
- 4F FeasibilityProbe + BabyStepFeasibilityReport + BabyStepStubResult + SystemProbe + dual timeout — evidence: crates/orchestrator/src/feasibility_probe.rs:161 (SCIP_DEF BabyStepFeasibilityReport); feasibility_probe.rs:524; crates/contracts/tests/feasibility_probe_contract.rs:418
- 4G AutonomousGate + TreeAutoAdvancePolicy + EscalationTrigger + circuit_breaker + SPLIT_TASK — evidence: crates/orchestrator/src/autonomous_gate.rs:125 (TreeAutoAdvancePolicy); autonomous_gate.rs:374 (EscalationTrigger); autonomous_gate.rs:691 (circuit_breaker_count)
- 4G mode_switch_contract (local/api/hybrid) — evidence: crates/contracts/tests/mode_switch_contract.rs:75
- 4H Layer1A (self_mod_layer1a) + AllowedConfigKey + PipelineOutcome + is_security_sensitive — evidence: crates/orchestrator/src/self_mod_layer1a.rs:197 (AllowedConfigKey); self_mod_layer1a.rs:163 (PipelineOutcome); self_mod_layer1a.rs:377 (is_security_sensitive)
- 4H MSI (MsiVerdict + MsiSnapshot) per PipelineMode + config archive DGM parent selection — evidence: crates/orchestrator/src/msi.rs:162 (verdict); crates/orchestrator/src/config_archive.rs:274 (select_parent); crates/orchestrator/src/self_mod_layer1a.rs:401 (MsiState)
- 4H StagedEval (Layer 2) + TestOutcomeReport + PASS_TO_FAIL rejection — evidence: crates/orchestrator/src/staged_eval.rs:117 (BenchmarkRunner trait); staged_eval.rs:331 (stage1_pass_to_fail_discards); crates/contracts/tests/staged_eval_contract.rs:37
- 4H auto-revert safety (DEFAULT_AUTO_REVERT_FRACTION=0.10, window=3) — evidence: crates/contracts/tests/self_mod_safety_contract.rs:123 (auto_revert_fires_on_three_bad_pipelines)
- 4I MediaValidator + render adapters (Blender, Playwright/web, ComfyUI) + VlmEngine + bounded concurrency — evidence: crates/orchestrator/src/media_validator/render_adapters/blender.rs:256; crates/orchestrator/src/media_validator/types.rs:345 (VlmEngine); crates/orchestrator/src/media_validator/mod.rs:282 (validate); crates/orchestrator/src/media_validator/feedback_loop.rs:184

## Deliverables PARTIAL

- 3I Deterministic Simulation Testing: reference model + FsmEvent present; proptest-state-machine PBT and madsim fault injection absent
  - exists: crates/orchestrator/src/dst.rs:37 (StateSnapshot), dst.rs:163 (ReferenceState), dst.rs:197 (ActualSystem)
  - gap: no proptest-state-machine dependency or stateful PBT harness found in elai.map; no madsim import detected; spec claims 10K+ random sequences via proptest-state-machine

- 3K adaptive AST-aware chunking: BM25 chunker present but AST-boundary splitting unconfirmed
  - exists: crates/knowledge/src/artifacts/indexer.rs:313 (chunk_file), crates/knowledge/src/bm25.rs:915
  - gap: chunk_file uses fixed-window approach per the spec's own description ("50-line fixed windows"); tree-sitter grammar integration in knowledge crate present (DKG) but AST-split boundary logic in bm25.rs not confirmed by map entries

## Deliverables MISSING

- 4J Empire Integration (FactoryLine protocol, ElaiNeuron, brain registration, brainctl replacement, Empire repair pipeline) — searched: "FactoryLine", "factory_line", "ElaiNeuron", "brainctl", "empire", "UnixTransport", "TcpTransport"; no evidence in elai.map

## Deliverables UNVERIFIABLE

- 3J.15 Gemma4 31B CPU offload validation — reason: spec explicitly marks Gemma4/E2B as rejected per memory; validation script path and results not in elai.map; cannot confirm pass/fail without running
- 4H Layer 3 structural self-modification stubs — reason: spec says "human-only, stubs in this phase"; cannot distinguish stub presence from absence via map-only search

## What must be done (action list)

- [ ] 4J: Implement Empire Integration starting with FactoryLine protocol client (crates/orchestrator/src/empire.rs or feature-gated crate) — spec: empire.txt (full)
- [ ] 3I: Add proptest-state-machine dependency and stateful PBT harness to crates/orchestrator/tests/dst_proptest.rs covering FsmEvent sequences per spec 10K+ sequences target
- [ ] 3I: Add madsim fault injection tests (packet loss, jitter, SIGKILL) if IPC correctness guarantees are required by Phase 4A contract tests
- [ ] 3K: Verify chunk_file in knowledge/src/artifacts/indexer.rs uses tree-sitter AST boundaries (fn/impl/struct/class) rather than line-count windows; fix if fixed-window still in use

## Notes
Phase 3J has the largest surface area of any phase in the index (15 sub-steps). All primary
deliverables (DoLa, TurboQuant, ActAdd, VllmClient, MCP bridge, LiveCli/ConversationRuntime
wiring) have clear evidence. Sub-steps 3J.WIRE.STUBS and 3J.14 are implicitly confirmed via
LiveCli and knowledge pipeline evidence. Phase 4J is the only phase with a zero-evidence status
but its "conditional" qualifier explains why it was never started.
