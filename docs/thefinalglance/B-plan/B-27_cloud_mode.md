# B-27: Cloud Mode (D-1 through D-4)
Objective: implement cloud mode per cloud-mode-design.txt and
post-phase4-implementation-order.txt GROUP D — config + provider
routing + cost tracking; cloud context budget profile + baby step sizing;
privacy redaction pipeline + BYOE + air-gapped mode; HYBRID escalation
+ pre-task cost estimation.
Estimated effort: 2-3 sessions.
Pre-requisites: B-10 (ContextBudget profile activation — cloud is just
another profile); B-15..B-20 (production hardening).
Parallel-safe with: B-25, B-26, B-28.

## Context

Per cloud-mode-design.txt and post-phase4-implementation-order.txt
STEPS D-1 through D-4:

3 modes (per-task selection):
- LOCAL (default): full local pipeline, all 6 reasoning layers, $0
- CLOUD: cloud model generates, local harness validates. Layers 2-5
  skipped. Layer 1 deterministic still runs.
- HYBRID: local first, auto-escalate to cloud on Layer 3+ failure for
  that step only

Provider routing reuses existing providers crate.

Cost tracking: CLOUD_SPEND SsbActionType; per-task and per-day spend
ceilings; pre-task cost estimation.

Context budget changes for cloud:
- 200K context vs 8-25K local
- Baby step: 200-500 LOC vs 25-40
- Decomposition depth: 2 vs 4
- Phase split threshold: 50K tok vs 3K
- Context telescope optional vs mandatory

[context_budget.cloud] slots:
  system_prompt = 500
  rules = 200
  phase_spec = 2000
  repo_map = 4000
  code_context = 150000
  ssb_entries = 2000
  grc_history = 1000
  output_reserve = 8000

Privacy controls:
- redact_env_vars (default true)
- redact_patterns (regex list)
- blocked_paths (glob list)
- audit trail per cloud call (timestamp, provider/model, token count, cost,
  files included, redaction count)

BYOE (Bring Your Own Endpoint): user provides API-compatible endpoint
(vLLM, TGI, Ollama on user-owned hardware). Killer feature for enterprise.

Air-gapped mode: explicit opt-out of all external network. Some features
disabled (nightly dep audit, cloud escalation, Oracle Mode web search)
but core works.

HYBRID escalation: when reasoning Layer 3+ fails locally, emit
ESCALATE_TO_CLOUD SSB action; orchestrator switches provider for current
step only; continues. Privacy redaction (D-3) applies to escalated calls.

## Reads

- F:\.ELAI_workflow\docs\specs\guide\cloud-mode-design.txt (FULL)
- F:\.ELAI_workflow\docs\specs\guide\post-phase4-implementation-order.txt
  STEPS D-1 through D-4
- F:\.ELAI_workflow\docs\specs\guide\reasoning-architecture-design.txt
  LAYER 5-6 + CLOUD BUDGET ECONOMICS
- summary/integration-summary.md
- crates/providers/src/* (existing Anthropic/OpenAI adapters)

## Tasks

### D-1: Config + provider routing + cost tracking

1. **Config schema** in elai_cc.toml:
   ```
   [cloud]
   provider = "anthropic"
   model = "claude-sonnet-4-6"
   escalation_model = "claude-opus-4-6"
   api_key_env = "ANTHROPIC_API_KEY"
   max_spend_per_task_usd = 2.00
   max_spend_per_day_usd = 10.00
   warn_at_percent = 80
   
   [execution]
   default_mode = "local"
   ```

2. **Provider routing** logic (5-step):
   - Read [cloud] config
   - Verify API key
   - Probe model availability (<100ms tokenize)
   - Fall back to [cloud.fallback] if probe fails
   - Degrade to LOCAL with notification if no fallback

3. **CLOUD_SPEND SSB action_type** + per-task/per-day accumulation +
   ceiling enforcement.

4. **CLI flags**: /task --mode cloud|hybrid|local. /config to change
   default.

5. **Session metadata**: execution_mode field on session.

### D-2: Cloud context budget + baby step sizing

6. **[context_budget.cloud] profile** with all 8 slot values per spec.
7. **Baby step sizing per mode**: local_3b=25, local_7b=40, local_32b=100,
   cloud_sonnet=300, cloud_opus=500.
8. **Decomposition depth per mode**: LOCAL max 4, CLOUD max 2.
9. **Reasoning layer collapse**: CLOUD skips L2-L5; uses L1 deterministic +
   L6 cloud generation. On L1 failure: inject findings, retry max 3,
   escalate to escalation_model.
10. **HYBRID**: L1-L4 local; L3+ failure → single cloud call.

### D-3: Privacy redaction

11. **Config** [cloud.privacy] schema:
    ```
    allow_source_upload = true
    allow_test_upload = true
    redact_env_vars = true
    redact_patterns = ["(?i)api[_-]?key\\s*=\\s*\\S+", ...]
    blocked_paths = [".env", "*.pem", "*.key", "credentials.*"]
    ```

12. **Redaction pipeline** runs BEFORE context assembly. Replaces matches
    with `[REDACTED:type]` placeholder.

13. **Audit trail** per cloud call (timestamp, provider/model, token
    counts, cost, file paths only — not content, redaction count).

14. **BYOE**: [cloud] provider = "custom", custom_endpoint = URL.
    Reuses OpenAI-compatible adapter.

15. **Air-gapped mode**: [execution] air_gapped = true. Disables ALL
    external network. Features disabled gracefully.

### D-4: HYBRID escalation + cost estimation

16. **ESCALATE_TO_CLOUD SSB action** when L3+ fails. Orchestrator
    switches provider for current step. Privacy redaction applies.

17. **Pre-task cost estimation**: phase_count * avg_tokens * pricing.
    Display to user: "Estimated cost: $0.12-$0.35. Proceed? [y/n]".

18. **Per-task cost dashboard**: MONITOR window status snapshot shows
    running spend total ("$0.00" if zero).

19. **Ceiling warnings**: at warn_at_percent (default 80%), inject
    warning into COMMAND. At 100%: HYBRID degrades, CLOUD fails step.

20. **Dashboard CloudPanel**: provider/model selection UI; current spend
    per task + per day gauges; redaction stats; air-gapped toggle.

## WIRING_CONTRACT

produces_symbols:
  - crates/orchestrator/src/cloud_mode.rs::CloudModeRouter
  - crates/orchestrator/src/cloud_mode.rs::CloudConfig
  - crates/orchestrator/src/cloud_mode.rs::ExecMode
  - crates/orchestrator/src/cloud_mode.rs::RoutedProvider
  - crates/orchestrator/src/cloud_mode.rs::SpendAccumulator
  - crates/orchestrator/src/cloud_mode.rs::RedactingBabyStepExecutor
  - crates/orchestrator/src/cloud_mode.rs::estimate_task_cost_usd
  - crates/orchestrator/src/cloud_mode.rs::resolve_exec_mode
  - crates/orchestrator/src/cloud_mode.rs::baby_step_loc_for_mode
  - crates/orchestrator/src/cloud_mode.rs::route_with_spend
  - crates/orchestrator/src/cloud_mode.rs::escalate_hybrid_step_to_cloud
  - crates/orchestrator/src/cloud_mode.rs::prompt_cost_estimate_gate
  - crates/orchestrator/src/privacy_redactor.rs::redact_context
  - crates/orchestrator/src/privacy_redactor.rs::PrivacyConfig
  - crates/orchestrator/src/context_budget.rs::cloud_200k
  - crates/orchestrator/src/ssb.rs::CloudSpend
  - crates/orchestrator/src/ssb.rs::EscalateToCloud
  - crates/orchestrator/src/ssb.rs::RouteBlocked
  - crates/orchestrator/src/ssb.rs::CostEstimate
  - crates/orchestrator/src/ssb.rs::WarnLimitReached
  - crates/orchestrator/src/ssb.rs::RedactionApplied
  - crates/orchestrator/src/ssb.rs::ExecutionModeChanged
  - crates/cli/src/runtime_config.rs::CloudConfigToml
  - crates/dashboard-v2/src/lib/screens/CloudPanel.svelte
  - crates/dashboard-v2/src-tauri/src/commands.rs::set_execution_mode
  - crates/dashboard-v2/src-tauri/src/commands.rs::get_execution_mode
  - crates/dashboard-v2/src-tauri/src/commands.rs::submit_task
consumes_symbols:
  - crates/orchestrator/src/runtime_setup.rs::apply_task_runner_wiring
  - crates/orchestrator/src/task_runner.rs::start_task_runner
reachable_from:
  - crates/orchestrator/src/runtime_setup.rs::apply_task_runner_wiring
  - crates/orchestrator/src/task_runner.rs::start_task_runner
  - crates/cli/src/run_subcommand.rs::run_async
  - crates/cli/src/task_cmd.rs::drive_task
  - crates/dashboard-v2/src-tauri/src/commands.rs::submit_task
stub_ban:
  - "cloud_calls_without_redaction: None disallowed in RedactingBabyStepExecutor"
  - "spend_cap_unenforced: route_with_spend must enforce daily/monthly caps"
  - "secrets_leaking_to_cloud: redact_context must scrub env-var tokens"
frontend_pair: CloudPanel + cost gauges
observable_effect:
  trigger: cd /d F:\.ELAI_workflow\.elai_cc && cargo test -p elai_cc_contracts --test b27_cloud_mode_smoke_ssb -- --nocapture
  expect_ssb: cloud_spend
  expect_artifact: dashboard shows running spend

## DETERMINISTIC_CHECK

```bash
# 1. Config schema present
rg "\\[cloud\\]" .elai_cc/elai_cc.toml
# 2. CLOUD_SPEND action
rg "CloudSpend" .elai_cc/crates/orchestrator/src/ssb.rs
# 3. Redaction tests
cargo test --package elai-orchestrator privacy_redaction
```

## ACCEPTANCE

- [ ] All 4 D-steps deliverables present
- [ ] 3 modes (LOCAL/CLOUD/HYBRID) selectable
- [ ] Cost tracking + ceilings work
- [ ] [context_budget.cloud] profile
- [ ] Baby step sizing per mode
- [ ] Reasoning layer collapse in CLOUD mode
- [ ] Privacy redaction default + BYOE + air-gapped
- [ ] HYBRID escalation works (single cloud call per failed step)
- [ ] Pre-task cost estimation displayed
- [ ] dashboard CloudPanel functional

## Files touched

- ADD: crates/orchestrator/src/cloud_mode.rs
- ADD: crates/orchestrator/src/privacy_redactor.rs
- MODIFY: crates/orchestrator/src/context_budget.rs (cloud profile)
- MODIFY: crates/orchestrator/src/task_runner.rs (mode-aware decomposition)
- MODIFY: crates/orchestrator/src/ssb.rs (CLOUD_SPEND, ESCALATE_TO_CLOUD)
- MODIFY: crates/cli/src/commands.rs (--mode flag)
- MODIFY: crates/dashboard-v2/src-tauri/src/lib.rs (V2 — was V1
  crates/dashboard/src/commands.rs); ADD crates/dashboard-v2/src/lib/
  components/CloudPanel.svelte (consumed by SettingsWindow.svelte —
  cloud key, spend cap, BYOE config). V2 SettingsWindow already has the
  Execution group placeholder per DOMAIN-CONTEXT §2 SETTINGS
- MODIFY: elai_cc.toml schema [cloud] + [cloud.privacy] + [execution]
- CREATE: workflow/reasoning/B-27-decisions.md

## Next

B-28 (Focus Tunnel).
