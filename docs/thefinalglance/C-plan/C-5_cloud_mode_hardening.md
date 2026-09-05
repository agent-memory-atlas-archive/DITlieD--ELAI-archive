# C-5: Cloud Mode Hardening
Objective: close all deferred production-readiness gaps from B-27 — hard
ceiling enforcement, warn-at-percent injection, CLI --provider flag,
HYBRID escalation retry loop, CloudPanel dashboard component,
pre-task cost estimation, LOC sizing constants, and BYOE endpoint.
Estimated effort: 2-3 sessions.
Pre-requisites: C-1 (SSB emission infrastructure), C-4 (get_cloud_spend
Tauri command — live spend read used by ceiling enforcement and
CloudPanel gauge).
Parallel-safe with: C-6/C-7 (dashboard design system — CloudPanel.svelte
must follow any design tokens C-6/C-7 introduce, but the component
scaffold can be written in parallel with those tokens as stubs).

STATUS (post-rebaseline session 2): SHIPPED.
  - All 10 task groups landed across session 1 (s-32952) + session 2 (s-25120).
  - SCHEMA_VERSION = 103 (atomic 3-variant bump: RouteBlocked +
    WarnLimitReached + CostEstimate). bijection 242 -> 245.
    action_type_lock TOTAL_VARIANTS = 306.
  - Drift fixes (SPEC_DRIFT, do NOT re-implement):
    * Task 3 CLI flag: shipped as `--provider local|cloud|hybrid`
      (args.rs:889-913), NOT `--mode`. `--mode` is gate-mode axis. Plan
      amended in place.
    * Task 9 CloudPanel directory: shipped at
      `dashboard-v2/src/lib/screens/CloudPanel.svelte`, NOT
      `lib/components/`. Plan amended in place.
    * Task 9 wording: "REDACTION_APPLIED COMMAND subtype" was sloppy;
      it ships as the distinct `SsbActionType::RedactionApplied` variant.
    * Tasks 2+6: shipped as 3 atomic SsbActionType variants (RouteBlocked
      + WarnLimitReached + CostEstimate), NOT as a generic
      `SsbPayload::Command` container. Atomic SCHEMA bump 102 -> 103.
  - Dashboard wires landed via `get_cloud_route_status` Tauri command +
    `CloudRouteStatus` TS type + CloudPanel banner section. The plan
    Task 9 "redaction event feed" remains DEFERRED — `RedactionApplied`
    rows are written by `RedactingBabyStepExecutor` (verified in
    contract tests) but a dedicated last-5 timeline lands in a
    follow-up that extends `CloudSpendSummary` or adds
    `get_recent_redactions`.
  - Decisions: workflow/reasoning/C-5-decisions.md (Q1-Q6 from session 1
    + DOUBLE-EMIT Option A + UNGROUNDED_HEURISTIC named-const rationale
    + plan drift fixes).

## Context

B-27 shipped routing, spend accumulation, privacy redaction, and
cloud_200k context profile (status=MOSTLY_DONE per verification report).
Eight gaps remain, all explicitly deferred in code comments:

1. Hard ceiling — `route_cloud()` cloud_mode.rs:207: "warning-only, hard
   caps deferred." Variants `DailySpendCapExceeded` / `MonthlySpendCapExceeded`
   defined (cloud_mode.rs:178-191) but `#[allow(dead_code)]` and never returned.
2. warn_at_percent — toml has `warn_at_usd=1.0` but no percent field;
   80% WARN_LIMIT_REACHED injection absent from config and code.
3. CLI --mode flag — absent from /task (args.rs); BackendConfig and ExecMode
   exist in orchestrator; gap is the CLI bridge only.
4. HYBRID escalation — EscalateToCloud type exists (ssb.rs:702), intent
   described at cloud_mode.rs:86-89, but task_runner.rs retry-loop
   re-invoking with ExecMode::Cloud is unconfirmed by contract tests.
5. CloudPanel.svelte — absent; MonitorWindow has a conditional spend gauge
   but no dedicated panel with mode selector + privacy event feed.
6. Pre-task cost estimation — no pre-task function; dialog absent.
7. LOC sizing constants — spec D-2 task 7 (25/40/300/500) not found by grep.
8. BYOE custom_endpoint — deferred at cloud_mode.rs:40 and :238.

Outstanding dependency (non-blocking): patch-R3 wires ToolUse translation
in ProductionAgentRpcServer.handle_infer_request (production_agent_rpc.rs:
400-458). C-5 smoke tests use pure-text tasks if R3 is not yet merged.

## Reads

- F:\.ELAI_workflow\.elai_cc\crates\orchestrator\src\cloud_mode.rs
  (CloudModeRouter::route_cloud, CloudConfig, SpendAccumulator,
   DailySpendCapExceeded, MonthlySpendCapExceeded variants)
- F:\.ELAI_workflow\.elai_cc\crates\orchestrator\src\task_runner.rs
  (ExecMode selection, BackendConfig wiring, escalation call sites)
- F:\.ELAI_workflow\.elai_cc\crates\cli\src\args.rs
  (task subcommand definition, existing flag patterns)
- F:\.ELAI_workflow\.elai_cc\crates\orchestrator\src\dispatcher.rs
  (BackendMode, BackendConfig structs)
- F:\.ELAI_workflow\.elai_cc\crates\dashboard-v2\src\lib\windows\MonitorWindow.svelte
  (existing cloud spend gauge — CloudPanel extends this)
- F:\.ELAI_workflow\.elai_cc\crates\dashboard-v2\src\lib\api\types.ts
  (CloudSpendInfo interface)
- F:\.ELAI_workflow\.elai_cc\crates\dashboard-v2\src-tauri\src\commands.rs
  (get_cloud_spend command wired in C-4)
- F:\.ELAI_workflow\.elai_cc\elai_cc.toml (cloud.spend schema)
- F:\.ELAI_workflow\docs\thefinalglance\verification\reports\B-plan\B-27_cloud_mode.report.md

## Tasks

1. **Ceiling enforcement in CloudModeRouter::route_cloud()**
   Read the current accumulated spend from the SSB (via the
   `get_cloud_spend` path introduced in C-4) before routing. If
   `spent_day_usd >= spend_cap_daily_usd`, return
   `Err(CloudModeError::DailySpendCapExceeded { spent_usd, cap_usd })`.
   Same for monthly. Emit `SsbActionType::ROUTE_BLOCKED` row (new
   action_type — string "route_blocked") with the reason. The two
   existing dead-code variants (`DailySpendCapExceeded`,
   `MonthlySpendCapExceeded`) are already defined; remove their
   `#[allow(dead_code)]` annotations.

2. **warn_at_percent field in CloudConfig + elai_cc.toml**
   Add `warn_at_percent: Option<u8>` to `CloudConfig`; wire from
   `CloudConfigToml` in `crates/cli/src/runtime_config.rs`. Default:
   `Some(80)`. In `route_cloud()`, before the cap check: if
   `spent_day_usd >= cap * (warn_at_percent / 100.0)` and no prior
   WARN_LIMIT_REACHED row exists this calendar day, push a
   `SsbPayload::Command` row with `action="WARN_LIMIT_REACHED"` and
   `detail="cloud spend at {pct:.0}% of daily cap"`.

3. **CLI --provider local|cloud|hybrid flag for /task** (SHIPPED)
   SPEC_DRIFT (session 1 closed this task under the name `--provider`
   to avoid collision with the gate-mode `--mode` axis at args.rs:865-887;
   plan amended in place). In `crates/cli/src/args.rs` (L889-913) the
   `--provider <value>` flag is parsed on the task/run subcommand path.
   Mapping: "cloud" -> `BackendMode::Api`, "hybrid"|"auto" -> `BackendMode::Hybrid`,
   "local" -> `BackendMode::Local` (dispatcher.rs::BackendMode::FromStr).
   Bridge: `resolve_backend_mode` in runtime_setup.rs maps CLI override
   over toml `[cloud].default_mode`. If flag absent, toml default is used.

4. **HYBRID escalation retry loop in task_runner.rs**
   After local baby-step retries exhaust (L3 failure path), check if
   `backend_config.mode == BackendMode::Auto` (Hybrid). If yes: emit
   `SsbPayload::EscalateToCloud(EscalateToCloudPayload { step_id, .. })`,
   then re-invoke the step with `ExecMode::Cloud`. Cap cloud retries at
   1 to prevent runaway spend. Existing EscalateToCloud SSB type
   (ssb.rs:702) and ExecMode::Cloud routing path are already wired.

5. **LOC sizing constants**
   In `cloud_mode.rs`, add named public constants matching spec D-2:
   `BABY_STEP_LOC_LOCAL_3B: u16 = 25`
   `BABY_STEP_LOC_LOCAL_7B: u16 = 40`
   `BABY_STEP_LOC_CLOUD_SONNET: u16 = 300`
   `BABY_STEP_LOC_CLOUD_OPUS: u16 = 500`
   Add a `baby_step_loc_for_mode(mode: ExecMode, model: &str) -> u16`
   helper that dispatches on mode + model substring ("3b" / "7b" /
   "sonnet" / "opus"). Wire into PipelineBuilder's step-size selector.

6. **Pre-task cost estimation**
   Add `fn estimate_task_cost_usd(plan_skeleton: &PlanSkeleton,
   model: &str) -> (f64, f64)` (returns (low, high) bound) to
   `cloud_mode.rs`. Formula: `step_count * avg_tokens * price_per_token`
   where avg_tokens is `baby_step_loc_for_mode * 4.0` (token/LOC heuristic
   — label as UNGROUNDED_HEURISTIC constant, flag for replacement when
   measured data available). Before the first cloud call in
   `task_runner.rs`, emit `SsbPayload::Command` with
   `action="COST_ESTIMATE"` and `detail="$low-$high"`. For the
   interactive path: surface this as a COMMAND row the dashboard
   CloudPanel reads; autonomous path proceeds without blocking.

7. **BYOE custom_endpoint in CloudConfig**
   Add `custom_endpoint: Option<String>` to `CloudConfig`. When set,
   override the `AnthropicClient` base URL. Wire `custom_endpoint` from
   `[cloud] custom_endpoint = "https://..."` in toml. Guard: reject if
   air_gapped=true. Emit a tracing::info! on first use noting the
   endpoint override (never log the value to SSB to avoid leaking
   internal infra URLs).

8. **ROUTE_BLOCKED SSB action type**
   Add `SsbActionType::RouteBlocked` variant (sql string
   "route_blocked") to ssb.rs. Add `RouteBlockedPayload { reason:
   String, spent_usd: f64, cap_usd: f64, cap_period: String }`.
   Add `SsbPayload::RouteBlocked(RouteBlockedPayload)` arm. Bijection
   test in the activity_entry_coverage_contract.rs table.

9. **CloudPanel.svelte** (SHIPPED at lib/screens/, NOT lib/components/)
   SPEC_DRIFT (session 1 + session 2 closed this under
   `dashboard-v2/src/lib/screens/CloudPanel.svelte`, NOT `lib/components/`;
   plan amended in place). Reads from `get_cloud_spend` + `get_execution_mode`
   + (session 2 add) `get_cloud_route_status`. Writes via `set_execution_mode`.
   Sections (shipped):
   - Spend gauge: day_usd / cap_usd bar with warn-at-80 + blocked badges
     and monthly cap row when set.
   - Provider selector: segmented control local/hybrid/cloud calling
     `set_execution_mode` on change; cloud disabled when `airGapped=true`.
   - Redaction count gauge (today's aggregate).
   - C-5 follow-on session-2 banners (SCHEMA_VERSION 103):
       * ROUTE BLOCKED notice — pulls latest `route_blocked` SSB row.
       * WARN LIMIT banner — pulls latest `warn_limit_reached` SSB row
         (hidden when route_blocked supersedes it).
       * COST ESTIMATE banner — pulls latest `cost_estimate` SSB row
         with the UNGROUNDED_HEURISTIC disclaimer inline.
   - DEFERRED: "Recent privacy events: last 5 SSB RedactionApplied rows"
     timeline. `RedactionApplied` rows are emitted by
     `RedactingBabyStepExecutor` (cloud_mode.rs:436+, contract-verified)
     but the timeline surface needs a `get_recent_redactions` Tauri
     command which lands in a follow-up.
   Mount: SettingsWindow.svelte L6 (import) + L849 (instance) under
   the existing Cloud section heading.

10. **Contract tests**
    - `crates/contracts/tests/cloud_ceiling_contract.rs`: verify
      route_cloud returns DailySpendCapExceeded when spent >= cap;
      verify warn_at_percent 80% fires WARN_LIMIT_REACHED row; verify
      RouteBlocked SSB round-trip.
    - `crates/contracts/tests/hybrid_escalation_contract.rs`: verify
      EscalateToCloud SSB row fires after local retry exhaust; verify
      cloud retry succeeds with ExecMode::Cloud; verify cloud retry cap
      of 1 (second exhaustion → HALT, not second escalation).
    - `crates/contracts/tests/cloud_loc_constants_contract.rs`: verify
      the four LOC constants match spec values; verify
      baby_step_loc_for_mode dispatches correctly for 4 mode/model
      combos.

## WIRING_CONTRACT

produces_symbols:
  - `CloudConfig.warn_at_percent` (Option<u8>, default Some(80))
  - `CloudConfig.custom_endpoint` (Option<String>)
  - `CloudModeError::DailySpendCapExceeded` (now returned, was dead code)
  - `CloudModeError::MonthlySpendCapExceeded` (now returned, was dead code)
  - `SsbActionType::RouteBlocked` + `RouteBlockedPayload`
  - `BABY_STEP_LOC_LOCAL_3B`, `BABY_STEP_LOC_LOCAL_7B`,
    `BABY_STEP_LOC_CLOUD_SONNET`, `BABY_STEP_LOC_CLOUD_OPUS` constants
  - `baby_step_loc_for_mode(mode, model) -> u16`
  - `estimate_task_cost_usd(plan_skeleton, model) -> (f64, f64)`
  - CloudPanel.svelte (spend gauge + mode selector + redaction event feed)
  - `--mode cloud|hybrid|local` CLI flag on /task
consumes_symbols:
  - `get_cloud_spend` Tauri command (C-4) — spend read for ceiling check
  - `EscalateToCloudPayload` (ssb.rs:1900) — existing, used by retry loop
  - `SpendAccumulator` (cloud_mode.rs:302) — existing spend accounting
  - `BackendConfig`, `BackendMode` (dispatcher.rs:104) — CLI bridge
  - `set_execution_mode` Tauri command (existing, CommandPalette-wired)
  - C-1 SSB emission infrastructure — RouteBlocked + WARN row writes
reachable_from:
  - elai_cc.toml [cloud.spend] warn_at_percent, hard_cap_daily_usd
  - elai_cc.toml [cloud] custom_endpoint
  - CLI /task --mode flag
  - Dashboard SettingsWindow > Cloud > CloudPanel
stub_ban:
  - spend ceiling check MUST read live SSB data (not a hardcoded zero);
    if get_cloud_spend returns error, log warning and SKIP the cap check
    rather than blocking all cloud calls
  - custom_endpoint MUST be rejected when air_gapped=true
  - HYBRID cloud retry MUST cap at 1 to prevent runaway spend
frontend_pair: CloudPanel.svelte + SettingsWindow.svelte mount point
observable_effect:
  trigger: set spend_cap_daily_usd=0.05, run task that crosses $0.04
  expect_ssb: WARN_LIMIT_REACHED COMMAND row appears once; RouteBlocked
              row appears when $0.05 crossed
  expect_artifact: CloudPanel spend gauge shows warn indicator; mode
                   selector reflects current ExecMode

## DETERMINISTIC_CHECK

```bash
# 1. Hard ceiling variants are no longer dead code
rg "DailySpendCapExceeded|MonthlySpendCapExceeded" .elai_cc/crates/orchestrator/src/cloud_mode.rs
# Expect: no allow(dead_code) annotation on either variant

# 2. RouteBlocked SSB action type
rg "RouteBlocked|route_blocked" .elai_cc/crates/orchestrator/src/ssb.rs
# Expect: RouteBlocked variant + "route_blocked" string in as_sql_str

# 3. LOC sizing constants
rg "BABY_STEP_LOC_LOCAL_3B|BABY_STEP_LOC_CLOUD_SONNET" .elai_cc/crates/orchestrator/src/cloud_mode.rs
# Expect: constants with values 25 and 300

# 4. warn_at_percent field
rg "warn_at_percent" .elai_cc/crates/orchestrator/src/cloud_mode.rs
# Expect: field in CloudConfig struct

# 5. --provider flag on CLI (spec_drift corrected from --mode; --mode is
#    the gate-mode axis at args.rs:865-887 and is independent)
rg '\"--provider\"' .elai_cc/crates/cli/src/args.rs
# Expect: at least one match in task/run subcommand parse path

# 6. CloudPanel component exists (spec_drift corrected: lib/screens/, not
#    lib/components/ — already mounted in SettingsWindow.svelte)
test -f .elai_cc/crates/dashboard-v2/src/lib/screens/CloudPanel.svelte

# 7. custom_endpoint field
rg "custom_endpoint" .elai_cc/crates/orchestrator/src/cloud_mode.rs
# Expect: field in CloudConfig struct

# 8. Contract tests
cargo test -p elai_contracts -- cloud_ceiling 2>&1 | grep "test result"
cargo test -p elai_contracts -- hybrid_escalation 2>&1 | grep "test result"
cargo test -p elai_contracts -- cloud_loc_constants 2>&1 | grep "test result"
# Expected: all pass

# 9. EscalateToCloud fired by hybrid path (integration)
cargo test -p elai_orchestrator -- hybrid_escalation 2>&1 | grep "test result"
```

## ACCEPTANCE

- [ ] DailySpendCapExceeded returned by route_cloud() when day spend >= cap
- [ ] MonthlySpendCapExceeded returned when monthly spend >= cap
- [ ] RouteBlocked SSB row emitted on ceiling hit; bijection test passes
- [ ] WARN_LIMIT_REACHED COMMAND row emitted at warn_at_percent (default 80%)
- [ ] warn_at_percent field in CloudConfig, wired from toml
- [ ] --mode cloud|hybrid|local flag parsed by /task CLI subcommand
- [ ] CLI flag bridges to BackendConfig (Cloud->Api, Hybrid->Auto, Local->Local)
- [ ] EscalateToCloud SSB row fires after local retry exhaust in hybrid mode
- [ ] Cloud retry cap of 1 enforced (second exhaustion halts, no second escalation)
- [ ] Four LOC sizing constants at correct spec values (25/40/300/500)
- [ ] baby_step_loc_for_mode dispatches correctly for all 4 combos
- [ ] estimate_task_cost_usd returns (low, high) tuple; heuristic constant labeled
- [ ] COST_ESTIMATE COMMAND row emitted before first cloud call in task_runner
- [ ] custom_endpoint wired from toml; rejected when air_gapped=true
- [ ] CloudPanel.svelte renders spend gauge, mode selector, redaction event feed
- [ ] CloudPanel mounted in SettingsWindow.svelte under "Cloud" heading
- [ ] cloud_ceiling_contract tests pass
- [ ] hybrid_escalation_contract tests pass
- [ ] cloud_loc_constants_contract tests pass
- [ ] patch-R3 dependency noted in Context; C-5 tests use text-only tasks if R3 absent
- [ ] Smoke 1: `elai /task --mode hybrid "tic-tac-toe in Rust"` with cap=0.05 and
       warn_at_percent=80; inject synthetic $0.04 spend fixture; assert one
       WARN_LIMIT_REACHED COMMAND row at $0.04 and RouteBlocked SSB row with
       clean error message (no panic) at $0.05.
- [ ] Smoke 2: cap=10.0; deliberately-failing local task (uninstalled lib) exhausts
       local retries; EscalateToCloud SSB row fires with matching step_id; Sonnet 4.6
       response succeeds (ELAI_E2E_FULL_PIPELINE_CLOUD=1); cloud retry cap=1 confirmed
       (second exhaustion halts, does not re-escalate).
- [ ] Smoke 3: dashboard SettingsWindow > Cloud: spend gauge visible (>=$0), mode
       selector shows local/hybrid/cloud, redaction event feed renders (empty state
       must not crash).

## Files touched

- MODIFY: crates/orchestrator/src/cloud_mode.rs
  (ceiling enforcement, warn_at_percent, LOC constants, cost estimate,
   custom_endpoint, remove dead_code allow on cap error variants)
- MODIFY: crates/orchestrator/src/ssb.rs
  (RouteBlocked variant + payload + bijection entry)
- MODIFY: crates/orchestrator/src/task_runner.rs
  (HYBRID escalation retry loop, COST_ESTIMATE row emit)
- MODIFY: crates/cli/src/args.rs (--mode flag on task/run subcommand)
- MODIFY: crates/cli/src/runtime_config.rs
  (wire warn_at_percent + custom_endpoint into CloudConfigToml)
- ADD: crates/dashboard-v2/src/lib/components/CloudPanel.svelte
- MODIFY: crates/dashboard-v2/src/lib/windows/SettingsWindow.svelte
  (mount CloudPanel under Cloud section)
- ADD: crates/contracts/tests/cloud_ceiling_contract.rs
- ADD: crates/contracts/tests/hybrid_escalation_contract.rs
- ADD: crates/contracts/tests/cloud_loc_constants_contract.rs
- NOTE: workflow/reasoning/B-27-decisions.md is gitignored; document
  key decisions in the MEMORIZING handoff instead.

## Next

C-6 (dashboard design system) or C-7 (dashboard navigation) depending
on prioritization. CloudPanel.svelte should adopt C-6 design tokens
if C-6 lands first.
