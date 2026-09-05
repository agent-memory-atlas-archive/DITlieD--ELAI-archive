# Verification Report: B-27_cloud_mode.md
Source: F:\.ELAI_workflow\docs\thefinalglance\B-plan\B-27_cloud_mode.md
Date: 2026-05-17
Overall status: MOSTLY_DONE
Confidence: HIGH

## Summary
B-27 core routing, spend accumulation, privacy redaction, and cloud_200k context profile are fully implemented and covered by contract tests. The dashboard MonitorWindow shows cloud spend gauges (wired), but get_cloud_spend() is a stub returning zeros (SSB aggregation deferred to B-27.5). BYOE custom_endpoint and per-task/per-day ceiling hard-enforcement are explicitly deferred in code comments. The CLI --mode flag and the dashboard CloudPanel component are absent; cloud mode selection is done via elai_cc.toml or the dashboard's CommandPalette setMode call. The B-27-decisions.md file was never created.

## Claims verified DONE

- cloud config schema [cloud] in elai_cc.toml — evidence: F:\.ELAI_workflow\.elai_cc\elai_cc.toml:308-328
- [cloud.privacy] schema in elai_cc.toml — evidence: F:\.ELAI_workflow\.elai_cc\elai_cc.toml:317-321
- [cloud.spend] with warn_at_usd, hard_cap_daily_usd, hard_cap_monthly_usd — evidence: F:\.ELAI_workflow\.elai_cc\elai_cc.toml:323-328
- CloudConfigToml struct (provider, model, mode, air_gapped, privacy, spend) — evidence: F:\.ELAI_workflow\.elai_cc\crates\cli\src\runtime_config.rs:184-248
- ExecMode enum (Local, Cloud, Auto) — evidence: F:\.ELAI_workflow\.elai_cc\crates\orchestrator\src\cloud_mode.rs:75-103
- CloudModeRouter::route() 5-step routing (air_gapped check, cred check, provider validation, spend pre-flight, decision) — evidence: F:\.ELAI_workflow\.elai_cc\crates\orchestrator\src\cloud_mode.rs:205-251
- AirGappedConflict error on ExecMode::Cloud when air_gapped=true — evidence: F:\.ELAI_workflow\.elai_cc\crates\orchestrator\src\cloud_mode.rs:232-233
- NoCredentials error when credentials absent — evidence: F:\.ELAI_workflow\.elai_cc\crates\orchestrator\src\cloud_mode.rs:236
- CLOUD_SPEND SSB action_type (CloudSpend) — evidence: F:\.ELAI_workflow\.elai_cc\crates\orchestrator\src\ssb.rs:694,1880,3609,5642
- ESCALATE_TO_CLOUD SSB action_type (EscalateToCloud) — evidence: F:\.ELAI_workflow\.elai_cc\crates\orchestrator\src\ssb.rs:702,1900,3610,5653
- SpendAccumulator dual-rate per-step cost (D-4:303-305) — evidence: F:\.ELAI_workflow\.elai_cc\crates\orchestrator\src\cloud_mode.rs:297-396
- SpendAccumulator.for_claude_sonnet_4_6() with named pricing constants — evidence: F:\.ELAI_workflow\.elai_cc\crates\orchestrator\src\cloud_mode.rs:48-67, 316-332
- CloudContextBudgetProfile cloud_200k() — attention_ceiling 200K, output_reserve 8K — evidence: F:\.ELAI_workflow\.elai_cc\crates\orchestrator\src\context_budget.rs:389-437
- ModelTier::Cloud enum variant — evidence: F:\.ELAI_workflow\.elai_cc\crates\orchestrator\src\context_budget.rs:195
- privacy_redactor.rs with PrivacyConfig, RedactionProfile (Minimal/Standard/Strict) — evidence: F:\.ELAI_workflow\.elai_cc\crates\orchestrator\src\privacy_redactor.rs:1-68
- redact_context() runs BEFORE context assembly (RedactingBabyStepExecutor wraps inner executor) — evidence: F:\.ELAI_workflow\.elai_cc\crates\orchestrator\src\cloud_mode.rs:411-461
- Redaction of email, IPv4, API tokens (Anthropic/OpenAI/GitHub shapes) — evidence: F:\.ELAI_workflow\.elai_cc\crates\orchestrator\src\privacy_redactor.rs:75-99
- Strict profile adds Windows + POSIX path redaction — evidence: F:\.ELAI_workflow\.elai_cc\crates\orchestrator\src\privacy_redactor.rs:105-120
- [REDACTED:type] placeholder pattern (REDACTED_EMAIL, REDACTED_IPV4, REDACTED_TOKEN, REDACTED_PATH) — evidence: F:\.ELAI_workflow\.elai_cc\crates\orchestrator\src\privacy_redactor.rs:70-73
- Air-gapped mode: air_gapped field in CloudConfig + AirGappedConflict enforcement — evidence: F:\.ELAI_workflow\.elai_cc\crates\orchestrator\src\cloud_mode.rs:112, 232-233
- CloudGroundingClient exists with async from_env_async() + auto-refresh via resolve_auth_with_auto_refresh — evidence: F:\.ELAI_workflow\.elai_cc\crates\orchestrator\src\grounding_client.rs:375,426-427
- B3 cloud-mode variant: e2e_full_pipeline_local.rs detects cloud mode via ELAI_E2E_FULL_PIPELINE_CLOUD=1 and routes through CloudGroundingClient — evidence: F:\.ELAI_workflow\.elai_cc\crates\orchestrator\tests\e2e_full_pipeline_local.rs:338-358
- Dashboard MonitorWindow cloud spend gauge (conditional on task_usd > 0 or day_usd > 0) — evidence: F:\.ELAI_workflow\.elai_cc\crates\dashboard-v2\src\lib\windows\MonitorWindow.svelte:60-62, 386-408
- CloudSpendInfo TypeScript interface — evidence: F:\.ELAI_workflow\.elai_cc\crates\dashboard-v2\src\lib\api\types.ts:1427
- set_execution_mode / get_execution_mode Tauri commands — evidence: F:\.ELAI_workflow\.elai_cc\crates\dashboard-v2\src\lib\api\stub.ts:1228 + commands.rs
- resolve_exec_mode() bridges BackendConfig to ExecMode for task_runner — evidence: F:\.ELAI_workflow\.elai_cc\crates\orchestrator\src\cloud_mode.rs:289-295
- exec_mode field on SSB output rows (session metadata) — evidence: F:\.ELAI_workflow\.elai_cc\crates\orchestrator\src\task_runner.rs:5191, 5290
- Contract tests for B-27 routing + spend accumulator + cloud_200k profile — evidence: F:\.ELAI_workflow\.elai_cc\crates\contracts\tests\cloud_mode_routing_contract.rs:1-113
- Contract tests for CloudSpend + EscalateToCloud SSB payload roundtrips — evidence: F:\.ELAI_workflow\.elai_cc\crates\contracts\tests\cloud_spend_ssb_contract.rs
- cloud config schema ADD: crates/orchestrator/src/cloud_mode.rs — evidence: file exists
- cloud config schema ADD: crates/orchestrator/src/privacy_redactor.rs — evidence: file exists
- MODIFY: crates/orchestrator/src/context_budget.rs (cloud profile) — evidence: cloud_200k() at line 398
- cloud mode selection available via dashboard CommandPalette setMode('HYBRID'/'CLOUD'/'LOCAL') — evidence: F:\.ELAI_workflow\.elai_cc\crates\dashboard-v2\src\lib\layout\CommandPalette.svelte:72

## Claims PARTIAL

- get_cloud_spend Tauri command (D-4 per-task + per-day dashboard spend gauges)
  - exists: F:\.ELAI_workflow\.elai_cc\crates\dashboard-v2\src-tauri\src\commands.rs:3305
  - gap: returns hardcoded zeros; SSB aggregation (cloud_spend_summary method) is a TODO stub per comment at line 3306. Spec calls for live aggregation from SSB CloudSpend rows.

- Per-task/per-day spend ceiling enforcement (D-4 task 19)
  - exists: DailySpendCapExceeded and MonthlySpendCapExceeded variants in CloudModeError — F:\.ELAI_workflow\.elai_cc\crates\orchestrator\src\cloud_mode.rs:180-186; CloudSpendToml.hard_cap_daily_usd field in runtime_config.rs:227
  - gap: ceiling enforcement is explicitly deferred ("warning-only, hard caps deferred") per code comment at cloud_mode.rs:179, 207, 245. warn_at_percent logic (fire at 80%) is not implemented; only warn_at_usd exists in toml.

- Audit trail per cloud call (D-3 task 13)
  - exists: EscalateToCloudPayload provides step_id + provider reasoning — ssb.rs:1897; CloudSpendPayload carries token counts + cost — ssb.rs:1880
  - gap: per-call audit trail with timestamp, provider/model, file-paths-included, and redaction_count is not a standalone structure. Spec calls for audit trail entries; actual implementation uses SSB rows (CloudSpend + EscalateToCloud) which cover a subset of the required fields. No redaction_count is tracked per-call.

- HYBRID escalation (D-4 task 16, L3+ failure -> ESCALATE_TO_CLOUD)
  - exists: EscalateToCloud SSB action_type defined — ssb.rs:702; ExecMode::Auto starts local and re-invokes with Cloud after escalation — cloud_mode.rs:86-89
  - gap: the actual retry-loop logic that re-invokes route() with ExecMode::Cloud after L3+ local failure is in the orchestrator (task_runner.rs), but no test confirms the escalation path is triggered by layer-3 failure specifically. The comment at cloud_mode.rs:86-89 describes the intent but the wiring in task_runner.rs is not confirmed by contract tests.

- Baby step sizing per mode (D-2 task 7: local_3b=25, local_7b=40, cloud_sonnet=300, cloud_opus=500)
  - exists: ModelTier::Cloud enum variant — context_budget.rs:195; cloud_200k() uses ModelTier::Cloud — context_budget.rs:401
  - gap: the per-mode LOC sizing constants (25/40/100/300/500) are not found in the codebase. No grep match for "cloud_sonnet.*300" or "local_3b.*25". Sizing may be inferred from ModelTier downstream, but not as named constants matching spec D-2 task 7.

- Reasoning layer collapse in CLOUD mode (D-2 task 9: CLOUD skips L2-L5)
  - exists: ExecMode::Cloud routes to RoutedProvider::Cloud — cloud_mode.rs:223-250; context_budget.rs ModelTier::Cloud exists
  - gap: no explicit L2-L5 skip logic found in orchestrator code under cloud_mode. The focus_tunnel ModeSelector uses ExecMode to select STRUCTURED generation mode for Cloud (mode.rs line 466), but there is no code that explicitly gates or skips reasoning layers 2-5 based on ExecMode::Cloud.

## Claims MISSING

- CLI --mode cloud|hybrid|local flag on /task command — searched: "--mode", "task.*--mode", "cli.*cloud.*flag" in crates/cli/src/; no match. Dashboard CommandPalette and elai_cc.toml are the only mode-selection surfaces.
- CloudPanel Svelte component (dashboard CloudPanel functional per WIRING_CONTRACT frontend_pair) — searched: Glob CloudPanel.svelte, grep CloudPanel in dashboard-v2/src; no match. SettingsWindow.svelte has cloud_api_key and cloud model selection but no dedicated CloudPanel component.
- Pre-task cost estimation dialog ("Estimated cost: $0.12-$0.35. Proceed? [y/n]") — searched: "pre.task.*cost", "Estimated cost", "proceed.*y.n" in crates/; only general UsageCostEstimate infrastructure found, no pre-task user-facing prompt.
- workflow/reasoning/B-27-decisions.md — searched: Glob B-27-decisions.md in workflow/reasoning/; file not present.
- BYOE custom_endpoint support — searched: "BYOE", "custom_endpoint", "bring_your_own" in crates/; cloud_mode.rs:40 and :238 explicitly state BYOE is deferred.

## Claims UNVERIFIABLE

- Decomposition depth per mode (D-2 task 8: LOCAL max 4, CLOUD max 2) — the plan claims CLOUD limits decomposition depth to 2; the codebase has ModelTier::Cloud and the cloud_200k profile, but decomposition depth capping by ExecMode is not visible from grep results without reading the full plan_skeleton or task_runner decomposition logic exhaustively. Cannot confirm or deny from available evidence.
- Pre-task cost estimation formula (phase_count * avg_tokens * pricing) — code comment at cloud_mode.rs:20-23 notes the B-27 original formula was superseded by dual-rate accumulation (D-4:303-305), but no pre-task estimation function implementing either formula was found. Status: MISSING rather than implemented differently.

## What must be done (action list)

- [ ] Implement SsbViewer::cloud_spend_summary() and wire it into get_cloud_spend() in commands.rs:3306 to replace the hardcoded-zero stub (B-27.5 follow-up)
- [ ] Implement warn_at_percent (80%) ceiling warning injection into COMMAND when accumulated spend approaches cap; currently only warn_at_usd is present in toml but not enforced
- [ ] Implement hard ceiling enforcement in CloudModeRouter::route() using spend_cap_daily_usd / spend_cap_monthly_usd (currently deferred per cloud_mode.rs:207)
- [ ] Add named baby-step LOC sizing constants matching spec D-2 task 7 (local_3b=25, local_7b=40, cloud_sonnet=300, cloud_opus=500) or document that sizing is handled implicitly via ModelTier
- [ ] Add CLI --mode cloud|hybrid|local flag to /task command in crates/cli/src/
- [ ] Create CloudPanel.svelte dashboard component per WIRING_CONTRACT frontend_pair requirement
- [ ] Implement pre-task cost estimation user dialog (phase_count * pricing display before cloud task submission)
- [ ] Track per-call redaction_count in RedactingBabyStepExecutor and surface in audit trail / SSB
- [ ] Create workflow/reasoning/B-27-decisions.md (required by decision-doc soft enforcement rule)
- [ ] Add BYOE custom_endpoint support (deferred in B-27, needs follow-up B-plan entry)

## Notes

The B-27 implementation is solid on the routing, privacy, and spend-accounting infrastructure. The three deferred items (BYOE, hard-ceiling enforcement, live SSB aggregation in the dashboard) are explicitly documented in code comments with "B-27.5 follow-up" tags, making the deferred scope traceable. The B3 cloud-mode variant is wired via environment variable (ELAI_E2E_FULL_PIPELINE_CLOUD=1) and proven in e2e_full_pipeline_local.rs. CloudGroundingClient.from_env_async() with OAuth auto-refresh is confirmed in grounding_client.rs:426-427.

REPORT_WRITTEN=F:\.ELAI_workflow\docs\thefinalglance\verification\reports\B-plan\B-27_cloud_mode.report.md | status=MOSTLY_DONE | done=29 | partial=6 | missing=5 | unverifiable=2
