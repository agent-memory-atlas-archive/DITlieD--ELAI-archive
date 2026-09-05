# Verification Report: B-28_focus_tunnel.md
Source: F:\.ELAI_workflow\docs\thefinalglance\B-plan\B-28_focus_tunnel.md
Date: 2026-05-17
Overall status: MOSTLY_DONE
Confidence: HIGH

## Summary
All seven E-step modules exist under crates/orchestrator/src/focus_tunnel/ and
are covered by per-step contract tests (focus_tunnel_e1 through focus_tunnel_e7
plus wiring + dashboard inline contracts). The four SSB action types and all
core structs match the WIRING_CONTRACT. The standalone FocusTunnelPanel.svelte
was intentionally replaced by inline chips on PlanWindow (dashboard_focus_tunnel
_inline_contract confirms this refactor), so its absence is by design.
The only unresolved gap is StarCoder/DeepSeek-Coder explicit FIM capability
detection: the plan states those models should have supports_fim=true, but only
the Qwen path (via ModelFeatures::none() default=false + resolved from system
probe) is confirmed in code; no StarCoder/DeepSeek-Coder named constants found.

## Claims verified DONE

- focus_tunnel module directory exists with all E-step files
  — evidence: crates/orchestrator/src/focus_tunnel/{fim,diff,mode,streaming,pre_assembly,dep_resolver,file_router,orchestrator,mod}.rs confirmed via Glob

- E-1: FimPromptAssembler struct + QWEN FIM token support
  — evidence: crates/orchestrator/src/focus_tunnel/fim.rs:91 (pub struct FimPromptAssembler), :218-300 (tests with QWEN_FIM_PREFIX/SUFFIX/MIDDLE)

- E-1: ModelFeatures.supports_fim bool field, defaults false
  — evidence: crates/orchestrator/src/context_budget.rs:251 (pub supports_fim: bool); crates/contracts/tests/focus_tunnel_e1_contract.rs:27 (ModelFeatures::none() must default supports_fim=false)

- E-1: Cloud execution forces supports_fim=false (fallback to STRUCTURED)
  — evidence: crates/orchestrator/src/focus_tunnel/mode.rs:466 (ExecMode::Cloud -> STRUCTURED via CloudOverride rule); focus_tunnel_e3_contract.rs:99 (cloud_execution_returns_structured)

- E-2: DiffParser struct exists + 3-tier edit matching integration
  — evidence: crates/orchestrator/src/focus_tunnel/diff.rs:7 (3-tier edit matcher referenced), :613 (pub async fn replay_hunks), :45 (EditMatchTier, SimilarityHint imports); focus_tunnel_e2_contract.rs:341

- E-2: Fallback on parse fail -> STRUCTURED present
  — evidence: crates/orchestrator/src/focus_tunnel/diff.rs comments confirm fallback; DiffApplyError surface used by E-3 router

- E-3: ModeSelector struct, deterministic 5-rule table
  — evidence: crates/orchestrator/src/focus_tunnel/mode.rs:154 (pub struct ModeSelector), :64-80 (SelectionRule enum with 5 rules), :168 (rule comments matching spec)

- E-3: GenerationMode enum (Fim/Diff/Structured)
  — evidence: crates/orchestrator/src/focus_tunnel/mode.rs:44 (pub enum GenerationMode)

- E-3: preferred_mode advisory field on step context
  — evidence: crates/contracts/tests/focus_tunnel_e3_contract.rs:412 (preferred_mode: Some(GenerationMode::Diff)); mode.rs:79-80 (HintedByPreferredMode rule)

- E-3: SSB action_type generation_mode_selected
  — evidence: crates/orchestrator/src/ssb.rs:1258 ("generation_mode_selected" parse arm); focus_tunnel_e3_contract.rs:466 (sql round-trip test)

- E-4: StreamingValidator struct with token-level import validation + lazy code detection
  — evidence: crates/orchestrator/src/focus_tunnel/streaming.rs:537 (StreamingValidator), :666 (import + lazy-code gate), tree-sitter partial parse :820

- E-4: Chunk-level tree-sitter partial parse every ~50 tokens
  — evidence: crates/orchestrator/src/focus_tunnel/streaming.rs:543 ("every ~50 tokens (~200 chars)" doc comment)

- E-4: SSB action_type streaming_early_terminated
  — evidence: crates/orchestrator/src/ssb.rs:1260 ("streaming_early_terminated" parse arm); focus_tunnel_e4_contract.rs:254 (sql round-trip test)

- E-5: PipelinePreAssembler struct with adaptive depth=1 cap
  — evidence: crates/orchestrator/src/focus_tunnel/pre_assembly.rs:192 ("depth=1 cap"), :204 (pub struct PipelinePreAssembler), :615 (depth_cap_blocks_second_prepare test)

- E-5: Adaptive threshold: disable if pass rate < 80%
  — evidence: crates/orchestrator/src/focus_tunnel/pre_assembly.rs:595-597 (set_adaptive_threshold clamping); focus_tunnel_e5_contract.rs:77 (set_adaptive_threshold(0.80))

- E-5: SSB action_type pipeline_pre_assembled
  — evidence: crates/orchestrator/src/ssb.rs (PipelinePreAssembled enum variant); ssb.rs:1086 (sql string "pipeline_pre_assembled"); focus_tunnel_e5_contract.rs:443

- E-6: DependencyResolver struct + Allowlist
  — evidence: crates/orchestrator/src/focus_tunnel/dep_resolver.rs:283 (pub struct Allowlist), :456 (DependencyResolver constructor taking Allowlist), :885 (build_resolver test helper)

- E-6: Pre-approved allowlist + transitive auto-approve
  — evidence: dep_resolver.rs:506 (AutoApprove::Allowlist branch skips registry), :564 (reason: AutoApproveReason::Allowlist); focus_tunnel_e6_contract.rs:237 (auto_approve_allowlist test)

- E-6: Model never runs cargo add / pip install directly
  — evidence: dep_resolver.rs:11 ("DO NOT run cargo add / pip install from model output"), install_command_for() at :771 produces the string but caller dispatches it

- E-6: SSB action_type dependency_resolved
  — evidence: crates/orchestrator/src/ssb.rs:1263 ("dependency_resolved" parse arm); focus_tunnel_e6_contract.rs:1005 (sql round-trip test)

- E-7: SpeculativeFileRouter struct with max_micro_steps=2
  — evidence: crates/orchestrator/src/focus_tunnel/file_router.rs:133 (pub struct SpeculativeFileRouter), :141 (max_micro_steps = 2 matches spec), :252 (cap enforcement)

- E-7: Micro-step creation: SSB MicroStepCreatedPayload
  — evidence: crates/orchestrator/src/ssb.rs:3418 (pub struct MicroStepCreatedPayload); task_runner.rs:2934 (SsbPayload::MicroStepCreated emit); focus_tunnel_e7_contract.rs:640

- task_runner.rs integration: ModeSelector called per baby step
  — evidence: crates/orchestrator/src/task_runner.rs:5174 (SsbPayload::GenerationModeSelected emit); :5071 (target_file + edit_region extracted from spec); map line 99630 (crate::focus_tunnel::Ge call in drive_task)

- Dashboard: inline focus-tunnel chips on PlanWindow (replaces FocusTunnelPanel.svelte)
  — evidence: crates/dashboard-v2/src/lib/windows/PlanWindow.svelte:172 (query_focus_tunnel_per_baby_step invocation); commands.rs:5860 (pub async fn query_focus_tunnel_per_baby_step); dashboard_focus_tunnel_inline_contract.rs:7 (design rationale documented)

- Dashboard: old 5-panel commands removed
  — evidence: dashboard_focus_tunnel_inline_contract.rs:46-52 (REMOVED_COMMANDS list + enforcement test)

- mode_distribution_for_payloads aggregator exported
  — evidence: crates/dashboard-v2/src-tauri/src/commands.rs:34 (use elai_orchestrator::focus_tunnel::{mode_distribution_for_payloads, GenerationMode})

- Contract tests: all 7 E-steps have dedicated contract test files
  — evidence: focus_tunnel_e1_contract.rs through focus_tunnel_e7_contract.rs + focus_tunnel_wiring_contract.rs + dashboard_focus_tunnel_inline_contract.rs confirmed via Glob

## Claims PARTIAL

- E-1: supports_fim=true for StarCoder and DeepSeek-Coder
  - exists: ModelFeatures::supports_fim field at context_budget.rs:251; Qwen path wired via system probe (task_runner.rs:5187 "supports_fim: resolved")
  - gap: No named constants or match arms for "StarCoder" or "DeepSeek-Coder" found in elai.map or context_budget.rs; only Qwen probe path is evidence-grounded

## Claims MISSING

- None confirmed missing beyond the StarCoder/DeepSeek partial above

## Claims UNVERIFIABLE

- Token budget impact claim (~6,800 tokens saved, 27% of 25K, combined 45-65%)
  — reason: performance/savings figures require runtime measurement; no benchmark results or test fixtures assert this figure; it is a design estimate

- Pipeline parallelism saves 100-500ms per step (~88% validation pass rate)
  — reason: latency figures are design targets; no benchmark or integration test in codebase asserts the millisecond range or pass rate percentage

- Streaming validation saves ~90% time when bad gen caught at token 50 vs 500
  — reason: performance claim requires runtime measurement; no test asserts this ratio

## What must be done (action list)

- [ ] Verify or add StarCoder/DeepSeek-Coder supports_fim=true capability detection in crates/orchestrator/src/context_budget.rs (ModelCapabilityProfile or resolve_model_features path)
- [ ] If token-budget savings claims are load-bearing for acceptance, add a benchmark or deterministic test fixture asserting approximate token counts in crates/orchestrator/src/focus_tunnel/

## Notes
The FocusTunnelPanel.svelte absence is intentional: dashboard_focus_tunnel_inline_contract.rs:252
(old_focus_tunnel_panel_svelte_file_removed test) explicitly asserts the file does NOT exist,
confirming the refactor from a dedicated side-panel to inline chips on PlanWindow is a
completed design decision, not a missing deliverable.
