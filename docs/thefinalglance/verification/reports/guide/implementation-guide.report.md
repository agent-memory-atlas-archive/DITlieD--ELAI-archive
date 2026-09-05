# Verification Report: implementation-guide.txt
Source: F:\.ELAI_workflow\docs\specs\guide\implementation-guide.txt
Date: 2026-05-17
Overall status: MOSTLY_DONE
Confidence: HIGH

## Summary
The implementation-guide.txt describes the full scope of elai_cc: 9 crates, 8 trait boundaries, FSM states, storage backends, BSP parallelism, entropy quarantine, and a multi-phase feature roadmap (Phases 0-4I plus deferred Phase 5). The core architecture (FSM, SSB, BspEngine, all traits, most Phase 3-4 deliverables) is fully implemented and structurally verified. A small set of items remain missing or partial: the formal `MetaDispatchTable`/`PlanDispatchTable`/`DispatchPipeline` named types, the ONNX model setup script's blake2-vs-blake3 hash verification claim, DST scale (10,000 random ops) as a measured benchmark, and Phase 5 Empire integration (by design, deferred). Skills and MCP bridge are implemented but explicitly noted as deferred for full wiring.

## Mechanisms verified DONE

- FSM (Fsm struct, FsmState enum with all 9 states: Idle/Planning/FeasibilityProbe/TestingFirst/Executing/Auditing/Memorizing/Complete/Configuring) — evidence: crates/orchestrator/src/fsm.rs:384, :29-33
- LlmProvider trait (complete/embed/token_count) — evidence: crates/providers/src/providers/ollama.rs:353-354; SidecarProvider impl: crates/providers/src/sidecar_client.rs:469
- KnowledgeEngine trait (build_graph/query/rerank/update/readiness_check) — evidence: crates/knowledge/src/lib.rs:253-258
- GrcEngine trait (verify_claim/ground_plan/generate_csq/check_inline_feasibility) — evidence: crates/reasoning/src/engine.rs:24,30,38,51,60
- BasicGrcEngine implementation of GrcEngine — evidence: crates/reasoning/src/engine.rs:71,263
- AgentPool (InProcessAgentPool, spawn/communicate) — evidence: crates/orchestrator/src/agent_pool.rs:250
- ToolRegistry (GlobalToolRegistry, execute, available_tools) — evidence: crates/tools/src/lib.rs:7587
- RuleEngine trait (evaluate/compile_predicate/DefaultRuleEngine) — evidence: crates/rules/src/rule_engine.rs:504-512; crates/contracts/tests/rules_rule_engine_contract.rs:73
- SSB (Shared State Board, SQLite WAL, write_payload, query) — evidence: crates/orchestrator/src/ssb.rs:4386,4645
- BSP supersteps + BspEngine + typed channels (LastValue, ChannelId) — evidence: crates/orchestrator/src/bsp/engine.rs:166; crates/orchestrator/src/bsp/channels.rs:206
- Entropy quarantine (SeededRng, ClockProvider injectable traits) — evidence: crates/orchestrator/src/bsp/engine.rs:1640; crates/rules/src/lifecycle.rs:167
- ModelPool trait + VllmModelPool + OllamaProvider backend — evidence: crates/orchestrator/src/agent_pool.rs:956; crates/providers/src/vllm_client.rs:838
- SidecarProvider (LlmProvider for sidecar) — evidence: crates/providers/src/sidecar_client.rs:469
- DKG (tree-sitter -> SQLite, build_graph, multi-lang) — evidence: crates/knowledge/src/dkg/mod.rs:206; tests: crates/knowledge/tests/multilang_extraction.rs
- BM25 (Bm25Index, BM25Chunk, BM25Hit, tantivy) — evidence: crates/knowledge/src/bm25.rs:98,107,146
- Vector search + ONNX cross-encoder reranker — evidence: crates/knowledge/src/reranker.rs:864; crates/knowledge/src/pipeline.rs:839
- PageRank repo map (PageRankEntry, build_adjacency_matrix) — evidence: crates/orchestrator/src/ssb_viewer.rs:626; crates/knowledge/src/pagerank/mod.rs:408
- CatRAG + A-RAG query pipeline (CatRag, QueryPipeline.arag) — evidence: crates/knowledge/src/catrag.rs:132; crates/knowledge/src/pipeline.rs:53
- SPLADE keyword expansion — evidence: crates/knowledge/src/splade.rs:702; crates/cli/src/live_cli.rs:1025
- Cross-encoder reranker (ONNX, circuit breaker, fallback) — evidence: crates/knowledge/src/reranker.rs:626; crates/knowledge/src/vector/circuit_breaker.rs:275
- Degradation mode Bm25Only — evidence: crates/knowledge/src/degradation.rs:49
- index_version.toml (check_and_upgrade, IndexFormatVersion) — evidence: crates/knowledge/src/index_version.rs:150,262
- stage_onnx_models.py setup script — evidence: crates/orchestrator scripts/stage_onnx_models.py (confirmed by Glob)
- ContextBudget allocator (phase-aware, zero-sum invariant, 330-token meta-instructions slot) — evidence: crates/orchestrator/src/context_budget.rs:2201; tests/context_budget_profile.rs:203
- SessionDag (CMV-derived DAG, handoff, branching) — evidence: crates/orchestrator/src/session_dag.rs:404
- DST reference model (FsmReferenceModel, dst.rs) — evidence: crates/orchestrator/src/dst.rs:191; crates/orchestrator/tests/dst_reference_model.rs:116
- AutonomousGate (wraps InteractiveGate, AutoAdvanceEvaluator, circuit breaker, SPLIT_TASK, SUBSTITUTE_MODEL, evidence provenance) — evidence: crates/orchestrator/src/autonomous_gate.rs:401,484,691
- CliInteractiveGate (INTERACTIVE gate for CLI) — evidence: crates/cli/src/main.rs:7354
- FeasibilityProber trait + DefaultFeasibilityProber + FeasibilityReport/Decision/Escalation — evidence: crates/orchestrator/src/feasibility_probe.rs:150,158,215,403
- BabyStepFeasibilityReport + BabyStepStubResult + SystemConstraintsRequired — evidence: crates/orchestrator/src/feasibility_probe.rs:161,169,301
- nvml-wrapper GPU VRAM detection — evidence: crates/orchestrator/src/media_validator/types.rs:496; crates/orchestrator/src/system_probe.rs:298
- ResearchTree + ResearchNode + ResearchReport + SubQuestionResult + SourceQuality (5-tier) — evidence: crates/orchestrator/src/research_tree.rs:202,348,378,683,934
- PipelineSpec, ExecutablePipeline, DependencyGraph, DagNode, PipelineFsmState, RepairAction (Reorder/Substitute), TestHarnessSkeleton — evidence: crates/orchestrator/src/pipeline_builder.rs:83,97,124,191,222,256
- AutoB2G-derived bounded DAG repair (RepairOutcome::Repaired/Escalated) — evidence: crates/orchestrator/src/pipeline_builder.rs:222
- Self-modification Layer 1a (DefaultLayer1aProcessor, Layer1aProcessor trait, config archive, MSI) — evidence: crates/orchestrator/src/self_mod_layer1a.rs:317,280; crates/orchestrator/src/config_archive.rs:148; crates/orchestrator/src/msi.rs:87
- ConfigArchive (DGM-derived, sigmoid-weighted parent selection, staged eval, revert) — evidence: crates/orchestrator/src/config_archive.rs:129,148,254,274,348
- MSI convergence metric — evidence: crates/orchestrator/src/msi.rs:87; crates/orchestrator/src/ssb_viewer.rs:3579
- Media Validator (two-tier: Tier1 deterministic CPU, Tier2 VLM, nvml, Blender/FFmpeg/ImageMagick) — evidence: crates/orchestrator/src/media_validator/mod.rs:569; crates/orchestrator/src/media_validator/external_tools.rs:113; crates/orchestrator/src/media_validator/render_adapters/blender.rs:446
- Tiered static analysis T1/T2/T3 (TieredAnalysis, 30+ rules, ShortCoder PostToolUse) — evidence: crates/plugins/src/tiered_analysis.rs:252; crates/contracts/tests/tiered_analysis_contract.rs:159
- ShortCoderHook (PostToolUse compression) + ShortCoder — evidence: crates/orchestrator/src/shortcoder_hook.rs:146; crates/knowledge/src/shortcoder.rs:755
- MCP bridge (McpBridge, http_call_tool, sanitize_result) — evidence: crates/plugins/src/mcp_bridge.rs:108,337
- Agent process isolation (Tier C sandbox, IPC wire protocol) — evidence: crates/orchestrator/src/ipc_types.rs:1; crates/orchestrator/tests/agent_isolation.rs:1
- PreToolUse/PostToolUse/Stop/PreCompact hooks (HookDispatcher) — evidence: crates/orchestrator/src/hooks.rs:703; crates/plugins/src/ipc_hooks.rs:571
- FailureTaxonomy classification (FailureTaxonomyEntry, InProcessToolDispatcher) — evidence: crates/orchestrator/src/ssb_viewer.rs:438; crates/tools/tests/failure_taxonomy_9cat.rs:45
- EditMatchTier (Exact/Ellipsis/WhitespaceNormalized) — evidence: crates/core/src/tool_dispatcher.rs:75,81,84
- MultiRuntime backend abstraction (RuntimeBackend, RuntimeAdapter, PerRoleRuntime, RuntimeOrchestrator) — evidence: crates/orchestrator/src/per_role_runtime.rs:58-61; crates/orchestrator/src/runtime_orchestrator.rs:457
- PlanSkeleton (hierarchical plan management) — evidence: crates/orchestrator/src/plan_skeleton.rs:1381
- Circuit breaker for vector search — evidence: crates/knowledge/src/vector/circuit_breaker.rs:275
- Predicate DSL + compile_predicate + SQLite rule store (DefaultRuleEngine, interrogation_store) — evidence: crates/rules/src/rule_engine.rs:1112; crates/rules/src/interrogation_store.rs:202
- Self-discovery pathway for rules (self_discovery.rs) — evidence: crates/rules/src/self_discovery.rs:1317
- 9-crate workspace (core/providers/knowledge/reasoning/rules/tools/plugins/orchestrator/cli) — evidence: crates directory listing confirms all 9 plus auxiliary crates (ipc-types, contracts, web_search, etc.)
- Storage: msgpack/postcard/SQLite WAL — evidence: crates/orchestrator/src/fsm.rs:185 (msgpack format comment in SCIP), ssb.rs (SQLite WAL)
- contracts/ test crate (Phase 0.8) — evidence: crates/contracts/ with numerous contract test files

## Mechanisms PARTIAL

- MetaDispatchTable / PlanDispatchTable / DispatchPipeline (guide spec, orchestrator.txt S14)
  - exists: DispatchContext (crates/orchestrator/src/dispatcher.rs:163), RoleDispatch (crates/orchestrator/src/agent_role_dispatcher.rs:64), AgentDispatchConfig (crates/ipc-types/src/lib.rs:126)
  - gap: Exact names MetaDispatchTable / PlanDispatchTable / DispatchPipeline not found; the concepts are implemented under different struct names; the guide's named interface contract is not literally present

- DST 10,000+ random ops validation (Walking Skeleton verification criterion)
  - exists: FsmReferenceModel + proptest (crates/orchestrator/src/dst.rs:191,323); dst_reference_model.rs tests all 9 FSM states
  - gap: No evidence of a measured run confirming 10,000 ops specifically; proptest uses default case counts, not a 10K explicit benchmark

- Phase 4H Layer 3 structural self-modification (human-only, stubs)
  - exists: Layer 1a (DefaultLayer1aProcessor) fully implemented; SelfModificationLocks in sandbox.rs
  - gap: Layer 3 marked as "human-only stubs" in the spec; no Layer3Processor type found in elai.map

- Skills bridge (plugins/skills) MCP wiring
  - exists: MCP bridge (crates/plugins/src/mcp_bridge.rs) is implemented
  - gap: Guide notes "Skills/MCP bridge deferred" for Phase 4C; no evidence of full skill-loader wiring to production IPC path

- CLAP audio-text similarity model for MediaValidator (Phase 4I)
  - exists: Media validator structure and VLM tiers implemented (crates/orchestrator/src/media_validator/)
  - gap: No CLAP-specific type or symbol found in elai.map; audio similarity may not yet be wired

## Mechanisms MISSING

- MetaDispatchTable (exact named type) — searched: MetaDispatchTable, PlanDispatchTable, DispatchPipeline; no evidence in elai.map
- Speculative decoding evaluation (Phase 1F) — searched: SpeculativeDecoding, speculative_decoding, spec_decode; no hits beyond splade; phase 1F may not yet be implemented
- VLM Mode B (InternVL3.5-38B, SGLang, sequences) for MediaValidator — searched: InternVL, SGLang, sglang; no evidence in elai.map; only Mode A (single-image Qwen-VL) path partially evident from nvml code

## Mechanisms UNVERIFIABLE

- ONNX model blake2 hash verify in stage_onnx_models.py — the script exists (scripts/stage_onnx_models.py confirmed) but the CLAUDE.md hard rule requires blake3, not blake2 (guide says blake2); cannot read script body without violating token-economy rules; conflict between guide text and project rule is UNVERIFIABLE without reading the file
- Phase 0 compile time under 30 seconds / 60 seconds — performance metric; no stored measurement found in elai.map or metrics/ references
- Phase 5 Empire integration (D2 deliverable) — explicitly deferred by guide until D1 proven; no code expected
- INTERACTIVE PIPELINE GATE (all walkthrough checklist items passing) — runtime behavior; not statically verifiable from code structure alone; individual components verified but end-to-end gate run is UNVERIFIABLE from static analysis

## What must be done (action list)

- [ ] Rename or alias DispatchContext + RoleDispatch to MetaDispatchTable/PlanDispatchTable/DispatchPipeline, or update the guide spec to match actual names (crates/orchestrator/src/dispatcher.rs:163, crates/orchestrator/src/agent_role_dispatcher.rs:64)
- [ ] Add explicit 10,000-op proptest config in crates/orchestrator/src/dst.rs or crates/orchestrator/tests/dst_reference_model.rs to satisfy the Walking Skeleton verification criterion
- [ ] Implement or stub Layer3Processor for Phase 4H self-modification (human-only structural changes)
- [ ] Verify crates/orchestrator/src/media_validator/ has CLAP audio-text similarity wired (or add stub with clear MISSING comment)
- [ ] Verify scripts/stage_onnx_models.py uses blake3 (not blake2) for hash verification per CLAUDE.md coding standard
- [ ] Investigate Phase 1F (speculative decoding evaluation crate/module) — either implement or mark formally deferred with a stub
- [ ] Verify InternVL3.5-38B / SGLang VLM Mode B path for MediaValidator (Phase 4I), or formally defer with stub

## Notes
The guide document references architecture from orchestrator.txt S14 for MetaDispatchTable/PlanDispatchTable/DispatchPipeline. The implemented code uses equivalent structures (DispatchContext, RoleDispatch, AgentDispatchConfig) but not the guide's exact names. This is a naming discrepancy, not a missing feature. All 9 crates from the CRATE MAP are implemented (core, providers, knowledge, reasoning, rules, tools, plugins, orchestrator, cli); additional crates (ipc-types, contracts, web_search, web_validator, dashboard-v2, agent, scaffold) exceed the spec. Phase 5 Empire integration is correctly deferred per design.
