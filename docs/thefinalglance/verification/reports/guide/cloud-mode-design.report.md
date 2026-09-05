# Verification Report: cloud-mode-design.txt
Source: F:\.ELAI_workflow\docs\specs\guide\cloud-mode-design.txt
Date: 2026-05-17
Overall status: MOSTLY_DONE
Confidence: HIGH

## Summary
The core cloud-mode infrastructure is well-implemented: provider routing (CloudModeRouter), spend accumulation (SpendAccumulator), the privacy redaction pipeline (privacy_redactor.rs), and the EscalateToCloud SSB signal are all present and tested. The 200K context budget profile (cloud_200k) and OAuth auto-refresh (CloudGroundingClient::from_env_async) are wired. However, several design items are explicitly deferred or absent: the CLI per-task --mode flag, the escalation_model fallback, the pre-task cost-estimation display, the spend-cap hard enforcement (currently warning-only), the per-file blocked_paths/allow_source_upload privacy controls, and the pipeline-designer complexity-to-mode recommendation. The get_cloud_spend Tauri command is a confirmed stub (TODO comment in source).

## Mechanisms verified DONE

- ExecMode enum (LOCAL/CLOUD/AUTO) with from_config_str — evidence: crates/orchestrator/src/cloud_mode.rs:75-103
- CloudModeRouter::route with air_gapped check, credential check, provider validation — evidence: crates/orchestrator/src/cloud_mode.rs:199-267
- CloudConfig struct reading [cloud] section from elai_cc.toml — evidence: crates/cli/src/runtime_config.rs:184-248
- CloudSpendToml with hard_cap_daily_usd, hard_cap_monthly_usd, warn_at_usd fields — evidence: crates/cli/src/runtime_config.rs:222-228
- SpendAccumulator dual-rate cost formula (input+output+cache tokens) — evidence: crates/orchestrator/src/cloud_mode.rs:297-396
- SpendAccumulator::for_model() factory with sonnet-4-6 pricing constants — evidence: crates/orchestrator/src/cloud_mode.rs:316-348
- SSB CloudSpendPayload struct with provider/model/tokens/cost/task_id/ts_ms — evidence: crates/orchestrator/src/ssb.rs:1880-1890
- SSB EscalateToCloudPayload struct with step_id, trigger_reason, layer_failed — evidence: crates/orchestrator/src/ssb.rs:1899-1907
- SsbActionType::EscalateToCloud wired to SsbPayload — evidence: crates/orchestrator/src/ssb.rs:3609
- RedactingBabyStepExecutor wrapping inner executor for pre-send redaction — evidence: crates/orchestrator/src/cloud_mode.rs:420-462
- PrivacyConfig with RedactionProfile (Minimal/Standard/Strict) — evidence: crates/orchestrator/src/privacy_redactor.rs:34-68
- redact_context() covering email, IPv4, API tokens (Standard) plus Windows/POSIX paths (Strict) — evidence: crates/orchestrator/src/privacy_redactor.rs:126-146
- [REDACTED_type] placeholder replacement (not blank erasure) — evidence: crates/orchestrator/src/privacy_redactor.rs:70-73
- ModelCapabilityProfile::cloud_200k() with 200K attention_ceiling, 8K output_reserve, slot weights — evidence: crates/orchestrator/src/context_budget.rs:398-437
- CloudGroundingClient with auto-refresh OAuth via from_env_async() / resolve_auth_with_auto_refresh — evidence: crates/orchestrator/src/grounding_client.rs:381-445
- resolve_exec_mode() mapping BackendMode::Api -> ExecMode::Cloud in task_runner — evidence: crates/orchestrator/src/cloud_mode.rs:289-295
- Dashboard CloudSpendInfo type + redaction_count wire shape — evidence: crates/dashboard-v2/src/lib/api/types.ts:1427-1432
- Contract tests for CloudSpend and EscalateToCloud SSB roundtrips — evidence: crates/contracts/tests/cloud_spend_ssb_contract.rs:37-130
- Contract test for cloud_200k profile validity (attention_ceiling=200000) — evidence: crates/contracts/tests/cloud_mode_routing_contract.rs:66-71

## Mechanisms PARTIAL

- Spend cap enforcement
  - exists: CloudModeError::DailySpendCapExceeded and MonthlySpendCapExceeded variants defined (crates/orchestrator/src/cloud_mode.rs:181-187); hard_cap_daily_usd/hard_cap_monthly_usd in CloudConfig
  - gap: Enforcement explicitly deferred per code comment ("Reserved for B-27.5 follow-up that wires the daily ceiling enforcement"; L178-180); route_cloud does NOT check caps — they are warning-only. Design requires CLOUD fails the step and asks user to raise limit.

- EscalateToCloud (HYBRID auto-escalation)
  - exists: EscalateToCloudPayload in SSB, ExecMode::Auto enum variant, contract tests — evidence: crates/orchestrator/src/ssb.rs:1900, crates/orchestrator/src/cloud_mode.rs:86-89
  - gap: Auto mode in route() returns RoutedProvider::Local unconditionally (cloud_mode.rs:219-222); the orchestrator retry loop that re-invokes with ExecMode::Cloud after local failure is not wired in task_runner.rs (no HYBRID escalation trigger found in task_runner.rs grep).

- Cloud spend read-back in dashboard (get_cloud_spend Tauri command)
  - exists: command signature, wire shape, MonitorWindow.svelte consumption — evidence: crates/dashboard-v2/src-tauri/src/commands.rs:3305-3321
  - gap: Command body is a confirmed TODO stub returning zeros; comment says "TODO(B-27.5): replace this default-zero stub with an SSB query" (L3306-3308).

## Mechanisms MISSING

- CLI per-task --mode flag ("/task --mode cloud") — searched: --mode cloud, exec_mode flag, task mode flag in crates/cli/src/args.rs and live_cli.rs; comment in commands.rs:3298 explicitly says "CLI --mode cloud flag bridge is still under follow-up"; no flag parsing found.

- escalation_model for MAJOR replan / hard failures after 3 retries — searched: escalation_model, escalate.*model, retry.*escalate in elai.map and orchestrator src; design specifies "claude-opus-4-6" as escalation_model config key with fallback on 3-retry failure; no implementation found.

- Pre-task cost estimation display ("Estimated cost: $X-$Y. Proceed? [y/n]") — searched: pre_task_estimate, cost estimate display, estimate.*display, warn.*percent in elai.map and cli; token counting from plan skeleton + user-facing confirm prompt are absent.

- blocked_paths / allow_source_upload / allow_test_upload / redact_env_vars privacy config fields — searched: blocked_paths, allow_source_upload, allow_test_upload, redact_env_vars in entire elai_cc/; only RedactionProfile (Minimal/Standard/Strict) is implemented; per-file path exclusion is absent.

- Pipeline-designer complexity-to-mode recommendation (TRIVIAL/SMALL -> local, MEDIUM -> hybrid, LARGE -> cloud) — searched: ComplexityClass, complexity_class, TRIVIAL, MEDIUM.*HYBRID, LARGE.*CLOUD, pipeline.*designer.*recommend in elai_cc/; no per-complexity mode recommendation found.

- Cloud audit trail logging files-in-context (paths only, not content) — searched: files_included, context_files, cloud.*audit.*files in elai_cc/; CloudSpendPayload has no files_included field; only token counts and cost are logged.

## Mechanisms UNVERIFIABLE

- Model availability probe ("single tokenize call, <100ms") before routing — the cloud_mode.rs route_cloud() makes no network call by design (noted in docstring L192-193). Whether a probe fires at a higher level (cli startup) was not confirmed from map search; code comment says probe is not in route_cloud.

- Baby step LOC sizing per mode (local_3b_loc=25, cloud_sonnet_loc=300, etc.) — no BabyStepSizing config struct or constant block found in elai.map search; the cloud_200k() profile adjusts context window but does not embed per-mode LOC targets as named config.

## What must be done (action list)

- [ ] Implement CLI per-task --mode flag in crates/cli/src/args.rs; bridge parsed ExecMode to task_runner BackendConfig
- [ ] Wire HYBRID escalation retry loop in crates/orchestrator/src/task_runner.rs: after local retries exhaust, emit EscalateToCloud SSB row and re-invoke with ExecMode::Cloud
- [ ] Implement spend cap hard enforcement in CloudModeRouter::route_cloud (B-27.5): check accumulated daily/monthly spend against caps before routing, return DailySpendCapExceeded when exceeded
- [ ] Replace get_cloud_spend stub in crates/dashboard-v2/src-tauri/src/commands.rs:3305 with real SSB aggregation query (sum today's CloudSpend rows)
- [ ] Implement escalation_model config key and retry-to-escalation-model path after 3 step failures
- [ ] Add pre-task cost estimation display: estimate tokens from plan skeleton, display "$X-$Y. Proceed?" before first cloud call
- [ ] Extend CloudPrivacyToml with blocked_paths ([]string), allow_source_upload (bool), allow_test_upload (bool), redact_env_vars (bool); implement path-exclusion in knowledge engine queries under cloud mode
- [ ] Add files_included field to CloudSpendPayload (path list only) and populate at call site for audit trail
- [ ] Add pipeline-designer complexity classification -> mode recommendation (TRIVIAL/SMALL -> local, MEDIUM -> auto, LARGE -> cloud)
- [ ] Add per-mode baby-step LOC sizing config struct and named constants

## Notes
The comment at crates/dashboard-v2/src-tauri/src/commands.rs:3298 is the clearest admission of follow-up scope: "because the CLI --mode cloud flag bridge is still under follow-up". The spend-cap deferred comment at cloud_mode.rs:178 explicitly labels the variants as "Reserved for B-27.5 follow-up". All core data structures, SSB wire formats, redactor, and accumulator are solid — the gaps are in the user-facing surface (CLI flag, HYBRID retry loop, dashboard read-back, pre-task display) and advanced privacy controls.
