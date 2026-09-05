# Verification Report: post-phase4-implementation-order
Source: F:\.ELAI_workflow\docs\specs\guide\post-phase4-implementation-order.txt
Date: 2026-05-17
Overall status: PARTIAL
Confidence: HIGH

## Summary
The input file defines 30 post-Phase-4 implementation steps across 7 groups (C,D,E,F,G,H). Group E (Focus Tunnel: E-1 through E-7) is fully shipped with dedicated modules, contract tests, and SSB wiring. Group D cloud infrastructure (ExecMode, CloudModeRouter, PrivacyConfig, EscalateToCloud, cloud_200k budget) is substantially present. Group C has MonitorWindow and tab system but EXPLORE window is missing as a standalone component and C-3/C-4 are partial. Groups F and G are largely missing dedicated implementations. H-1 through H-4 (wiring harness types, DkgReachabilityAuditor, StubBanAuditor, WiringContract struct, SSB action types 19/20) are absent — the term WIRING_CONTRACT appears only as a comment convention in existing tests, not as a structural Rust type.


## Mechanisms verified DONE

- E-1 FIM assembly: FimPromptAssembler struct and module — crates/orchestrator/src/focus_tunnel/fim.rs (USE at mod.rs:64); contract tests at crates/contracts/tests/focus_tunnel_e1_contract.rs
- E-1 ModelCapabilities / supports_fim field: FimContext wired into orchestrator at crates/orchestrator/src/focus_tunnel/orchestrator.rs:205
- E-2 DiffParser + DiffHunk types: crates/orchestrator/src/focus_tunnel/diff.rs (struct DiffHunk; DiffParser::new().parse()); contract tests at crates/contracts/tests/focus_tunnel_e2_contract.rs
- E-3 ModeSelector + GenerationMode: struct ModeSelector at crates/orchestrator/src/focus_tunnel/mode.rs:154; GenerationModeSelectedPayload at ssb.rs:3282; SSB action_type "generation_mode_selected" at ssb.rs:1258; contract tests at crates/contracts/tests/focus_tunnel_e3_contract.rs
- E-4 StreamingValidator + EarlyTermination: StreamingValidator at crates/orchestrator/src/focus_tunnel/streaming.rs:941; SsbPayload::StreamingEarlyTerminated at commands.rs:5922; contract tests at crates/contracts/tests/focus_tunnel_e4_contract.rs
- E-5 PipelinePreAssembler: struct PipelinePreAssembler at crates/orchestrator/src/focus_tunnel/pre_assembly.rs:204; PreAssemblyStats at pre_assembly.rs:483; contract tests at crates/contracts/tests/focus_tunnel_e5_contract.rs
- E-6 DependencyResolver + Allowlist + AutoApprove: struct DependencyResolver at crates/orchestrator/src/focus_tunnel/dep_resolver.rs:455; AutoApproveReason at dep_resolver.rs:161; contract tests at crates/contracts/tests/focus_tunnel_e6_contract.rs
- E-7 SpeculativeFileRouter + MicroStepType + MicroStepCreated: struct SpeculativeFileRouter at crates/orchestrator/src/focus_tunnel/file_router.rs:133; MicroStepType enum at file_router.rs:65; SsbActionType::MicroStepCreated at ssb.rs:1264; contract tests at crates/contracts/tests/focus_tunnel_e7_contract.rs
- D-1 ExecMode enum + CloudModeRouter: enum ExecMode at crates/orchestrator/src/cloud_mode.rs:75; CloudModeError at cloud_mode.rs:171; SpendAccumulator at cloud_mode.rs:302
- D-1 CLOUD_SPEND / EscalateToCloud SSB types: SsbActionType::EscalateToCloud at ssb.rs:703; EscalateToCloudPayload at ssb.rs:1900; contract test at crates/contracts/tests/cloud_spend_ssb_contract.rs
- D-2 Cloud context budget profile: ModelCapabilityProfile::cloud_200k() at crates/orchestrator/src/context_budget.rs:398; ModelTier::Cloud at context_budget.rs:195; attention_ceiling=200_000, output_reserve=8_000
- D-3 PrivacyConfig + RedactionProfile: struct PrivacyConfig at crates/orchestrator/src/privacy_redactor.rs:34; redact_context() at privacy_redactor.rs:126; air_gapped field at cloud_mode.rs:112
- D-4 EscalateToCloud SSB action + UsageCostEstimate: EscalateToCloud roundtrip tested at crates/contracts/tests/cloud_spend_ssb_contract.rs:88; UsageCostEstimate at crates/core/src/usage.rs:41; estimate_cost_usd at usage.rs:91
- G-5 ExtractedFact.exact_quote + has_exact_quote: field exact_quote at crates/core/src/research_types.rs:80; has_exact_quote() at research_types.rs:143; used in cross-verify pipeline at crates/orchestrator/src/research_pipeline.rs:314
- G-5 Constrained decoding (LMFE): opt-in JSON-schema constrained decoding wired in grounding_client.rs:161; contract test at crates/contracts/tests/research_decompose_constrained.rs
- C-1 MonitorWindow component: crates/dashboard-v2/src/lib/windows/MonitorWindow.svelte; intervention queue rendering, thinking pulse, timeline (last 15), phase dots, status snapshot all present
- C-1 Intervention SSB wiring: SsbActionType::InterventionRequest at ssb.rs:1172; submit_intervention_response in stub.ts
- C-2 Intervention controls (dashboard side): InterventionRequestPayload at ssb.rs:859; contract tests at crates/contracts/tests/dashboard_4k3_4k4_4k5_contract.rs and fu_intervention_request_emission_contract.rs; Approve/Reject/Takeover buttons in MonitorWindow.svelte:142-151
- C-4 Thinking pulse animation: @keyframes monitorBreath at MonitorWindow.svelte:722; step-flash at MonitorWindow.svelte:944; failBorder at MonitorWindow.svelte:936; phase breath at MonitorWindow.svelte:705
- C-4 JetBrains Mono + Lucide icons: declared in dashboard-v2/DESIGN-BRIEF.md and package.json; used throughout Svelte components
- C-3 Tab system (basic): workspace.svelte.ts; openTab/closeTab/moveTab functions; drag-drop via Panel.svelte; localStorage persistence via STORAGE_KEY at workspace.svelte.ts:8


## Mechanisms PARTIAL

- C-1 EXPLORE window
  - exists: CommandWindow.svelte references /explore slash command (line 48, 389, 586); DESIGN-BRIEF.md:165 specifies ExploreWindow
  - gap: No ExploreWindow.svelte component file found; explore query routing through Oracle Mode pipeline not implemented as a separate window; file references are not clickable to open REVIEW

- C-3 Window history / session recovery via SSB
  - exists: Ctrl+Shift+H shortcut declared at shortcuts.ts:114-118; workspace.svelte.ts persists to localStorage (STORAGE_KEY)
  - gap: Ctrl+Shift+H action calls openCommandPalette() (stub), not a real closed-window restore list; no SSB WINDOW_STATE action type found; no last-20-closed list; session recovery is localStorage only, not SSB-backed

- D-2 Baby step LOC sizing per mode (local_3b=25, local_7b=40, cloud_sonnet=300, cloud_opus=500)
  - exists: ModelTier::Cloud and ModelTier::Micro/Small defined at context_budget.rs:190; cloud_200k profile present
  - gap: No explicit per-mode LOC sizing table found (no constants for 25/40/100/300/500 LOC); decomposition depth limits (LOCAL max 4, CLOUD max 2) not found as named config

- D-4 Pre-task cost estimation display ("Estimated cost: $X-$Y. Proceed? [y/n]")
  - exists: UsageCostEstimate/estimate_cost_usd at core/src/usage.rs; cloud_spend SSB wiring present
  - gap: No pre-task cost prompt to user found in CLI or orchestrator; D-4 cost ceiling warnings at 80% not found

- G-5 Safety sandwich full implementation (Phase D+G oracle pipeline wiring)
  - exists: exact_quote field and has_exact_quote() on ExtractedFact; cross-verify in research_pipeline.rs:314; LMFE constrained decoding in grounding_client.rs
  - gap: fabrication_rate per-model metric not found; LRU source content cache (100 pages, 50KB) not found; fuzzy Levenshtein quote fallback not found


## Mechanisms MISSING

- H-1 WiringContract struct type — searched: WiringContract, struct WiringContract, wiring_contract; no Rust struct definition found (only comment text in tests)
- H-1 StubBanEntry, ObservableEffect, EntrypointSet, WiringEvidence, WiringViolation types — searched: StubBanEntry, ObservableEffect, EntrypointSet, WiringEvidence, WiringViolation; no definitions
- H-1 DkgReachabilityAuditor, StubBanAuditor, WiringContractValidator — searched: DkgReachabilityAuditor, StubBanAuditor, WiringContractValidator; not found
- H-1 SSB action types 19/20 (WiringEvidence, WiringViolation) — searched: wiring_evidence, wiring_violation in ssb.rs; not found
- H-1 EntrypointSetLoader + [wiring.entrypoints] config schema — searched: EntrypointSetLoader, wiring.entrypoints; not found
- H-2 BabyStepSpec 9th block (WiringContract field) — confirmed: BabyStepSpec has 8 fields (id, task, estimated_tokens, grounding, tdd, acceptance, preferred_mode, target_file, edit_region); no wiring_contract field at research_tree.rs:1667
- H-2 Designer USER_FACING classifier + frontend_pair enforcement — searched: USER_FACING, frontend_pair; no deterministic classifier found
- H-2 WIRING_CHECK stage in baby step cycle — searched: WIRING_CHECK, wiring_fail, WiringFail; not found
- H-3 ObservableEffectRunner / SmokeRunner — searched: ObservableEffectRunner, smoke_runner, SmokeGate; not found
- H-4 elai wiring audit CLI subcommand — searched: wiring audit, wiring_audit, subcommand wiring; not found; no [wiring.entrypoints] in elai_cc.toml
- H-4 Closure of 10 MVP wiring gaps (baby_step_executor None, anti_spiral_enabled false, etc.) — searched: stub_ban seed table in rules.db; no wiring audit harness to verify closures
- F-1 Adaptive model selection / task complexity classifier — searched: adaptive_model, per_task_model, task_complexity_classifier; VRAM tier detection exists (vram_tier.rs) but no per-task model switching logic found
- F-1 7B as explicit 12GB default + dual-model config — searched: 12GB tier default 7B, dual_model; VramTier detect exists but no elai_cc.toml tier=12GB 7B default config found
- F-2 Progressive indexing (4-phase: BM25 <5s, DKG imports <30s, hot files <5min, background) — searched: progressive_index, IndexPhase, hot_files, git_aware_index; indexing_service.rs has BM25 path but no phased progressive init found
- F-3 Weaker evidence gates for legacy (ZERO/LOW test coverage detection) — searched: WeakerGate, TestCoverageLevel, ZeroCoverage, LowCoverage; evidence.rs has EvidenceGate but no coverage-level relaxation logic found
- F-4 LspDkgProvider / LSP client for Python, TypeScript — searched: LspDkgProvider, lsp_dkg, Pyright, tsserver; not found
- F-5 Per-database connection pool separation + batch SSB writes + in-memory rule cache — searched: pool_separation, BatchSsbWrite, RuleCache, connection_pool per db; not found
- F-6 Hardware detection first-run wizard + auto-mode selection table — searched: first_run_wizard, hardware_probe, AutoModeSelect, Docker sidecar; partial NVML use exists for media validator but no first-run wizard or GPU->mode mapping table found
- G-1 CumulativeTokenTracker type — searched: CumulativeTokenTracker, prompt_tokens_total; not found; cache_read_tokens used in SpendAccumulator but no dedicated tracker type
- G-2 CompiledPipeline / TemplateCrystallizer / CompiledStep — searched: CompiledPipeline, TemplateCrystallizer, CompiledStep, COMPILED_PIPELINE; not found
- G-3 CompiledExecutor + decompilation logic — searched: CompiledExecutor, decompilation; not found
- G-4 BackgroundTaskManager (max 3 concurrent, soft/hard timeout, BACKGROUND_TASK_STARTED/COMPLETE SSB) — searched: BackgroundTaskManager; background_task_id field exists in BashCommandOutput but no manager/detach infrastructure found
- C-2 Browser Notification API + optional webhook — searched: requestPermission, NotificationAPI, webhook_url in dashboard; not found in Svelte components


## Mechanisms UNVERIFIABLE

- C-4 Scrollbar styling (4px, teal thumb) — would require CSS audit of all Svelte scoped styles across 11 window components; elai.map does not index CSS custom properties
- D-2 Reasoning layer collapse (CLOUD skips Layers 2-5) — spec references reasoning-architecture-design.txt layers; mapping layers to actual orchestrator paths requires reading reasoning pipeline source not indexed with clear layer tags
- D-3 BYOE (custom_endpoint provider adapter) — cloud_mode.rs has ExecMode but custom_endpoint TOML schema not verified; would need full elai_cc.toml schema read


## What must be done (action list)

- [ ] Implement H-1: Add WiringContract, StubBanEntry, ObservableEffect, EntrypointSet, WiringEvidence, WiringViolation types in crates/tools/src/ or crates/orchestrator/src/
- [ ] Implement H-1: Add DkgReachabilityAuditor, StubBanAuditor, WiringContractValidator structs with SQL-backed DKG path queries
- [ ] Implement H-1: Add SSB action types 19 (WiringEvidence) and 20 (WiringViolation) to SsbActionType enum in ssb.rs
- [ ] Implement H-1: Add EntrypointSetLoader + [wiring.entrypoints] TOML schema to elai_cc.toml
- [ ] Implement H-2: Add wiring_contract: Option<WiringContract> as 9th field to BabyStepSpec in research_tree.rs
- [ ] Implement H-2: Add USER_FACING classifier and WIRING_CHECK stage to baby step cycle in task_runner.rs or pipeline_tree_navigator.rs
- [ ] Implement H-3: Add ObservableEffectRunner struct integrating BashSandbox + SSB read-only connection
- [ ] Implement H-4: Add `elai wiring audit` CLI subcommand in crates/cli/src/; declare [wiring.entrypoints] in elai_cc.toml
- [ ] Implement G-1: Add CumulativeTokenTracker type to orchestrator/ with prompt_tokens_total and completion_tokens_total fields; wire to provider response.usage
- [ ] Implement G-2: Add CompiledPipeline, CompiledStep, TemplateVariable, TemplateCrystallizer to orchestrator/
- [ ] Implement G-3: Add CompiledExecutor with decompilation logic and CLI offer
- [ ] Implement G-4: Add BackgroundTaskManager (max 3 concurrent) with BACKGROUND_TASK_STARTED/COMPLETE SSB types; wire to BashSandbox detach
- [ ] Implement F-1: Add adaptive model selection (task complexity classifier) and explicit 12GB-tier 7B default config in elai_cc.toml
- [ ] Implement F-2: Add 4-phase progressive indexing to indexing_service.rs with git-aware hot-files prioritization
- [ ] Implement F-3: Add coverage-level detection (ZERO/LOW/ADEQUATE) and relaxed evidence gate variants in evidence.rs
- [ ] Implement F-4: Add LspDkgProvider trait + LSP client adapters for Pyright and tsserver
- [ ] Implement F-5: Separate per-database connection pools; add batch SSB write buffer; add in-memory rule cache
- [ ] Implement F-6: Add first-run hardware wizard to CLI with GPU->mode mapping table
- [ ] Implement C-1 (ExploreWindow): Create ExploreWindow.svelte as standalone component routing queries through Oracle Mode pipeline
- [ ] Implement C-2: Add Browser Notification API (requestPermission) and optional webhook POST to MonitorWindow or a notification service module
- [ ] Implement C-3: Replace Ctrl+Shift+H stub (openCommandPalette) with real closed-window history list; add WINDOW_STATE SSB action type for cross-session layout persistence
- [ ] Complete D-2: Add explicit per-mode LOC sizing constants (local_3b=25 LOC, local_7b=40, cloud_sonnet=300, cloud_opus=500) in context_budget.rs or decomposer
- [ ] Complete D-4: Add pre-task cost estimation prompt in CLI and 80%/100% ceiling warning injection
- [ ] Complete G-5: Add fabrication_rate per-model metric; add LRU source content cache (100 pages, 50KB); add Levenshtein fuzzy fallback in research_pipeline.rs Phase G


## Notes
Group E (Focus Tunnel) is the most complete group — all 7 steps (E-1 through E-7) have dedicated source modules, SSB action types, and contract test files. Group D cloud infrastructure is substantially present but D-4 pre-task UX and D-2 LOC sizing are incomplete. Group H (wiring harness) has zero structural implementation despite being designated TIER 0 and the load-bearing prerequisite for all other groups. Groups G-2, G-3, G-4 and all of F (except partial NVML presence for F-6) are absent from the codebase. The WIRING_CONTRACT text appearing in test comments refers to the spec design doc block, not to a realized Rust type.

REPORT_WRITTEN=F:\.ELAI_workflow\docs\thefinalglance\verification\reports\guide\post-phase4-implementation-order.report.md | status=PARTIAL | done=20 | partial=6 | missing=24 | unverifiable=3
