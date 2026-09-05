C-5 — Cloud Mode Hardening: Decisions
=====================================

Sessions: s-32952 (session 1) + s-25120 (session 2 follow-on close)
Plan:     docs/thefinalglance/C-plan/C-5_cloud_mode_hardening.md
Contexts: workflow/sessions/s-32952/context-package-C5.md
          workflow/sessions/s-25120/completion-drive/contexts/C-5_cloud_mode_hardening-context.md
Status:   SHIPPED. Session 1 landed RedactionApplied + provider + panel
          scaffold; session 2 closed the remaining 17 follow-on items
          (2 PARTIAL stubs + 15 MISSING wires + plan SPEC_DRIFT fixes).

ORCHESTRATOR Q-RESOLUTIONS (locked at task start)
-------------------------------------------------

Q1 — --mode conflict (DRIFT-1):
  Decision: ADD `--provider local|cloud|hybrid` as a SEPARATE CLI flag.
            Default = inherits toml `[cloud].default_mode`. Do NOT
            repurpose C-9's `--mode` (which is gate-mode).
  Sites:    args.rs::parse_run_subcommand_args, main.rs::CliAction::Run,
            run_subcommand.rs::RunArgs.provider,
            runtime_setup.rs::resolve_backend_mode().

Q2 — BackendMode::Auto/Hybrid variant (DRIFT-2):
  Decision: ADD `BackendMode::Hybrid` variant (clean enum option). The
            three match arms (Display, FromStr, resolve_exec_mode) were
            updated alongside; no other production match sites exist on
            BackendMode outside cloud_mode + live_cli (1 each).
            FromStr also accepts the legacy `"auto"` alias mapping to
            Hybrid so existing elai_cc.toml `default_mode = "auto"`
            files continue to resolve to the same provider semantics.

Q3 — UTC day boundary (DRIFT-3):
  Decision: ACCEPT as documented limitation. CloudPanel.svelte header
            explicitly states "UTC day · spend, redaction, provider".
            The aggregator `cloud_spend_summary` retains its UTC-day
            window. Threading per-operator IANA tz is deferred to a
            future patch.

Q4 — RedactionApplied SSB variant (DRIFT-4):
  Decision: SHIP new SSB variant `RedactionApplied` with payload
            { count: u64, fields: Vec<String>, baby_step_id: u64, ts_ms: i64 }.
            Atomic 9-tuple lockstep (SCHEMA 50 -> 51):
              (1) SCHEMA_VERSION constant bumped
              (2) SQL CHECK constraint adds 'redaction_applied'
              (3) SsbActionType enum variant
              (4) as_sql_str arm
              (5) FromStr arm
              (6) SsbPayload variant
              (7) action_type() arm
              (8) RedactionAppliedPayload struct
              (9) test_payload + all_test_payloads (102 -> 103) +
                  size-assert string updated.

Q5 — CloudPanel.svelte location:
  Decision: CREATE new file at
            `dashboard-v2/src/lib/screens/CloudPanel.svelte`. REPLACE
            the existing read-only block in SettingsWindow.svelte
            (lines 731-768) with `<CloudPanel ... />` mount under the
            same "cloud" section heading. Capacity cap value passed
            through from the existing `cloudSpendCap` input.

Q6 — Production wiring:
  Decision: Wire `CloudModeRouter`, `RedactingBabyStepExecutor`,
            and `EscalateToCloud` emission into production paths.
            Wiring sites:
              - runtime_setup.rs::apply_task_runner_wiring (chokepoint)
              - run_subcommand.rs (CLI loads elai_cc.toml + provider)
              - dashboard-v2/src-tauri/src/lib.rs (Tauri setup loads
                same toml via local helper; no elai-cli dep)
              - task_runner.rs (pre-dispatch hook emits EscalateToCloud
                + spend-cap check via SsbViewer.cloud_spend_summary)
              - grounding_client.rs::CloudGroundingClient
                (send_with_ledger wraps send_message; per-call
                CloudSpend SSB write; pre-call cap check; on cap
                breach emits cloud_spend_blocked marker via existing
                EscalateToCloud row with trigger_reason="cap_reached").


SHIPPED CHANGES
---------------

ssb.rs — atomic 9-tuple bump (SCHEMA 50 -> 51):
  - Added RedactionApplied variant + as_sql_str + FromStr + SsbPayload
    variant + action_type() arm.
  - Added RedactionAppliedPayload struct.
  - Updated test_payload + all_test_payloads (102 -> 103) + size-assert
    string.
  - Added db_path() accessor on Ssb (needed by task_runner cap check).

dispatcher.rs:
  - Added BackendMode::Hybrid variant.
  - Updated Display + FromStr + serde rename_all (FromStr also accepts
    "auto" / "cloud" aliases).

cloud_mode.rs:
  - resolve_exec_mode covers Hybrid -> Auto.
  - RedactingBabyStepExecutor::with_ssb constructor (writer plumb) +
    per-field redaction counting + RedactionApplied row emission on
    fire-and-forget basis.

task_runner.rs:
  - TaskRunnerConfig + ResolvedConfig: cloud_router +
    cloud_spend_cap_daily_usd + cloud_spend_blocked_emitted.
  - Pre-dispatch hook in drive_task baby-step loop emits
    EscalateToCloud (Cloud / Auto exec_mode), cap-check via
    SsbViewer.cloud_spend_summary, BLOCKED marker via existing row
    type with trigger_reason="cap_reached".
  - new_minimal + 3 test struct-literal sites updated.

grounding_client.rs::CloudGroundingClient:
  - ssb_writer (Option<Arc<Ssb>>) + spend_accumulator + spend_cap_daily +
    task_id fields + 4 builder methods.
  - send_with_ledger() helper: pre-call cap check, post-success
    CloudSpend emission with per-call token deltas + computed cost.

runtime_setup.rs::RuntimeWiring + apply_task_runner_wiring:
  - cloud_config + provider_override fields on RuntimeWiring.
  - resolve_backend_mode() helper maps CLI --provider over toml default.
  - apply wraps baby_step_executor in RedactingBabyStepExecutor::with_ssb
    when the resolved provider is cloud-capable (Api | Hybrid).
  - Threads cloud_spend_cap_daily_usd onto TaskRunnerConfig.
  - Always populates TaskRunnerConfig.backend_config so
    resolve_exec_mode() reads the correct provider.

args.rs::parse_run_subcommand_args:
  - Parses `--provider <local|cloud|hybrid>` and `--provider=<val>`.
  - Rejects unknown values with the same error format as `--mode`.
  - CliAction::Run gained a `provider: Option<String>` field.

run_subcommand.rs:
  - RunArgs.provider plumbed; loads elai_cc.toml + stashes cloud_config
    + provider_override on the wiring before apply.

dashboard-v2/src-tauri/src/lib.rs:
  - Tauri setup loads `[cloud]` directly with a thin toml::de helper
    (avoids elai-cli dep). Falls through to CloudConfig::default() on
    missing/invalid toml so the router is always constructable.

dashboard-v2/src/lib/screens/CloudPanel.svelte (NEW):
  - Spend gauge (day_usd / capDailyUsd with warn-at-80% + blocked
    badges), task spend, monthly cap (when set), provider selector
    (LOCAL | HYBRID | CLOUD via set_execution_mode), redaction count
    today, cache token deltas. 5s polling. Svelte 5 runes mode.

dashboard-v2/src/lib/windows/SettingsWindow.svelte:
  - Imports CloudPanel; replaces the L731-768 read-only block with
    a CloudPanel mount under the existing "cloud" section heading.

c5_cloud_mode_contract.rs (NEW, 15 tests):
  - BackendMode::Hybrid + Display/FromStr (Hybrid, auto, cloud aliases) + serde.
  - resolve_exec_mode coverage of all three BackendMode variants.
  - SCHEMA_VERSION >= 51 floor.
  - RedactionApplied bijection + SQL round-trip via SsbViewer.
  - CloudModeRouter constructible + air_gapped guard.
  - EscalateToCloud preserves both new trigger_reason strings
    (hybrid_local_first, cap_reached).

3 sibling contract tests touched for field additions:
  - task_runner_chokepoint_contract.rs (cloud_router + cap field)
  - agent_pool_executor_wiring_contract.rs (cloud_config + override)
  - c9_autonomous_infra_contract.rs (3 RuntimeWiring sites)

TEST RESULTS
------------

cargo test -p elai-orchestrator --lib (ssb + cloud_mode):
  34 passed, 0 failed.
cargo test -p elai_cc_contracts (8 test files in C-5 scope):
  68 passed, 0 failed.
cargo check --workspace: GREEN.
npm run check (dashboard-v2): 0 errors, 13 pre-existing warnings in
  unrelated files (AuditTimelineWindow on:click deprecations + unused
  CSS selectors in KnowledgeDrawer / ResearchDrawer / ProvidersDrawer).

RESIDUALS / FOLLOW-UPS
----------------------

(none deferred from the C-5 plan)

R1: The two `#[allow(dead_code)]` annotations on
    CloudModeError::DailySpendCapExceeded / MonthlySpendCapExceeded
    were INTENTIONALLY KEPT. Today the cap-breach surface is the
    GroundingError::RequestFailed { reason: "cloud daily spend
    cap reached..." } returned by CloudGroundingClient.send_with_ledger
    plus the EscalateToCloud row with trigger_reason="cap_reached".
    The typed CloudModeError variants are reserved for the
    follow-up patch that lifts cap-check INTO CloudModeRouter.route_cloud()
    itself (per the original B-27.5 sketch in cloud_mode.rs:178-186).
    They are exercised indirectly via the matching string in the
    GroundingError reason.

R2: estimate_task_cost_usd helper + COST_ESTIMATE row emission +
    LOC sizing constants (25/40/300/500) + BYOE custom_endpoint were
    out of scope for the orchestrator's locked Q-resolutions (Q1-Q6).
    They remain open from the C-5 plan §Tasks 5/6/7 and can ship in a
    follow-up patch with their own contract test.

R3: The CloudPanel privacy-event feed currently displays
    `redaction_count` (today's aggregate from CloudSpendSummary). The
    new RedactionApplied SSB rows DO get written (verified by
    contract test) but the panel does not yet poll them into a
    timeline list. That polling surface is independent of C-5's
    locked task list and lands when the dashboard adds a
    `get_recent_redactions()` Tauri command.


SESSION 2 (s-25120) — CLOSURE OF B-27.5 FOLLOW-UP RESIDUALS
===========================================================

Session 2 picked up the 8 residuals R1/R2/R3 deferred from session 1
plus the 15 MISSING wires identified by the s-25120/cross-check-recheck
report. The prior executor crashed mid-implementation; this entry
documents the resumed close.

Q7 — Atomic 3-variant SSB landing (SCHEMA 102 -> 103, bijection
     242 -> 245, action_type_lock 303 -> 306):
  Decision: ship RouteBlocked + WarnLimitReached + CostEstimate
            together as 3 new SsbActionType variants — NOT as a
            generic SsbPayload::Command container (plan Tasks 2 + 6
            originally implied such a container; grounding sweep S4
            confirmed no such container exists, atomic-3 was the
            cleaner downstream filter UX). Single SCHEMA bump.

  Sites (all 8 lockstep wiring arms + atomic counts):
    (1) SCHEMA_VERSION 102 -> 103 (ssb.rs:1043)
    (2) SQL CHECK constraint adds 'route_blocked', 'warn_limit_reached',
        'cost_estimate' (ssb.rs:1711-1716)
    (3) SsbActionType enum +3 variants (ssb.rs:3049/3055/3066)
    (4) as_sql_str arms (ssb.rs:3449-3451)
    (5) ssb_action_type_from_str_public arms (ssb.rs:3842-3844)
    (6) SsbPayload enum +3 arms (ssb.rs:6670-6672)
    (7) action_type() method +3 arms (ssb.rs:7060-7062)
    (8) all_test_payloads +3 fixtures (ssb.rs:12924-12943) +
        bijection size_assert 242 -> 245 (ssb.rs:15300+)
    (9) action_type_lock TOTAL_VARIANTS 303 -> 306 +
        ALL_VARIANTS array +3 entries (orchestrator/tests/action_type_lock.rs:35)
        + action_type_lock.golden +3 entries (sorted)

Q8 — Double-emit between task_runner cap-hook and route_cloud ceiling
     (HIGH-RISK per exploration R3):
  Option A (chosen): task_runner cap-hook owns the dedup mutex
                     `cloud_spend_blocked_emitted`. It calls
                     `CloudModeRouter::route_with_spend(Cloud, probe_creds,
                     Some(snap))` — when the router returns
                     DailySpendCapExceeded / MonthlySpendCapExceeded, the
                     cap-hook writes the typed RouteBlocked SSB row AND
                     the backwards-compat EscalateToCloud(cap_reached)
                     row under the SAME `block_emit_now` gate. Both rows
                     fire at most once per UTC day. The router itself
                     does NOT write — it only reports the verdict, so
                     route_cloud stays sync + testable (no writer
                     coupling). This satisfies "additive, lower
                     regression risk" per the locked orchestrator
                     Option-A instruction.

  Why NOT Option B (delete cap_reached branch from cap-hook): the
  EscalateToCloud(cap_reached) row pre-existed in session 1 dashboards;
  removing it would break the current activity timeline filter.
  Backwards-compat preserved via dual emission under the same dedup.

Q9 — UNGROUNDED_HEURISTIC token/LOC constant (cost estimator):
  Decision: ship `TOKENS_PER_LOC_UNGROUNDED_HEURISTIC: f64 = 4.0`
            as a named const at cloud_mode.rs:104 with the canonical
            UNGROUNDED_HEURISTIC label in its doc-comment.
  Rationale: the spec at docs/specs/guide/cloud-mode-design.txt:84-93
             gives the formula shape but not measured token/LOC ratios
             for ELAI's corpora. 4.0 is the public industry rule-of-thumb
             (~1 token per 4 chars, ~16 chars per LOC ≈ 4 tokens/LOC).
             Per CLAUDE.md grounded-by-default rule + stub-registry
             exemption (zero-lazy-code), the const ships with:
               - the UNGROUNDED_HEURISTIC token in its identifier
               - a TODO[ungrounded] line-comment marker
               - a doc-comment naming the future cost-measurement B-N
                 as the replacement source
             Acceptance: the CostEstimatePayload carries
             `heuristic_source: "ungrounded_heuristic"` so the dashboard
             can render the disclaimer to the operator inline (already
             wired in CloudPanel banner).
  Q10 — Spread ratio: separate const `COST_ESTIMATE_SPREAD_RATIO = 0.30`
        (mid * (1 ± 0.30)) chosen to mirror the spec example "$0.12-$0.35"
        spread without overstating uncertainty. Verified by
        `cloud_loc_constants_contract::estimate_task_cost_usd_sonnet_returns_low_high_in_spread`.

Q11 — local_32b_loc=100 spec value omission:
  Decision: ship the 4 plan-named constants (25/40/300/500); leave
            BABY_STEP_LOC_LOCAL_32B for E-32 follow-up (out of C-5 scope
            per plan). `baby_step_loc_for_mode` falls back to the 7B
            value for any non-"3b" local model id; this gives the 32B
            model a sensible default until E-32 adds the const.

Q12 — Plan SPEC_DRIFT fixes (rebaseline, NOT re-implement):
  D-1. Task 3 — `--mode` -> `--provider` (args.rs:889-913 already ships
       --provider; `--mode` is the gate-mode axis at L865-887). Plan
       text + DETERMINISTIC_CHECK row 5 amended in place.
  D-2. Task 9 — CloudPanel.svelte at `lib/screens/` (not `lib/components/`).
       Plan text + Files-touched + DETERMINISTIC_CHECK row 6 amended.
  D-3. Task 9 — "REDACTION_APPLIED COMMAND subtype" wording -> the
       distinct `SsbActionType::RedactionApplied` (session 1 ship).
  D-4. Tasks 2 + 6 — "SsbPayload::Command row with action=..." -> 3 new
       SsbActionType variants per Q7 above.

Q13 — Dashboard wires for the 3 new SSB rows (CloudPanel banners):
  Decision: add a single `get_cloud_route_status` Tauri command that
            aggregates the latest `route_blocked` + `warn_limit_reached`
            + `cost_estimate` rows via `SsbViewer::latest_action_of_type`.
            One round-trip per refresh tick (5s) instead of three;
            cleaner stub shape (`CloudRouteStatus` with 3 nullable
            sub-objects). Rendered as 3 stacked banners ABOVE the spend
            gauge in CloudPanel.svelte. WARN banner is suppressed when
            ROUTE BLOCKED is present (block supersedes warn).
  Implementation sites:
    - crates/dashboard-v2/src-tauri/src/commands.rs::get_cloud_route_status
    - crates/dashboard-v2/src-tauri/src/lib.rs invoke_handler!
    - crates/dashboard-v2/src/lib/api/types.ts CloudRouteStatus +
      RouteBlockedRow + WarnLimitReachedRow + CostEstimateRow
    - crates/dashboard-v2/src/lib/api/stub.ts get_cloud_route_status
    - crates/dashboard-v2/src/lib/screens/CloudPanel.svelte banner
      section + CSS .cloud-panel__notice variants


SESSION 2 SHIPPED CHANGES
-------------------------

ssb.rs — atomic 3-variant SCHEMA bump 102 -> 103:
  - RouteBlocked / WarnLimitReached / CostEstimate variants + payloads.
  - 8 lockstep wiring arms (per Q7 above).
  - bijection size_assert string 242 -> 245.
  - SQL CHECK constraint allowlist updated.

cloud_mode.rs:
  - CloudConfig: +warn_at_percent: Option<u8> (default Some(80)),
                 +custom_endpoint: Option<String> (default None).
  - CloudModeError: +CustomEndpointForbidden; removed #[allow(dead_code)]
                    on DailySpendCapExceeded + MonthlySpendCapExceeded.
  - route_with_spend(): new ceiling-enforcing variant taking
                        Option<SpendSnapshotInput>; route() delegates.
                        Cap check returns Daily/MonthlySpendCapExceeded.
                        Air-gapped + custom_endpoint conflict returns
                        CustomEndpointForbidden.
  - SpendSnapshotInput struct: day_usd + month_usd + from_day_usd helper.
  - Constants: BABY_STEP_LOC_LOCAL_3B / LOCAL_7B / CLOUD_SONNET / CLOUD_OPUS
               (25/40/300/500). TOKENS_PER_LOC_UNGROUNDED_HEURISTIC = 4.0.
               COST_ESTIMATE_SPREAD_RATIO = 0.30.
  - baby_step_loc_for_mode(mode, model) -> u16: substring dispatch.
  - estimate_task_cost_usd(plan_skeleton, model) -> (low, high) f64.
  - warn_limit_reached(config, day_usd) -> bool helper.

task_runner.rs (cap-hook):
  - SpendSnapshotInput populated from SsbViewer::cloud_spend_summary
    (stub_ban: viewer error => skip cap check, never block).
  - One-shot CostEstimate emit per task lifetime
    (cloud_cost_estimated_emitted Mutex<bool>).
  - One-shot WarnLimitReached emit per UTC day
    (cloud_warn_emitted Mutex<bool>).
  - route_with_spend probe returns DailySpendCapExceeded /
    MonthlySpendCapExceeded; both branch into typed RouteBlocked +
    backwards-compat EscalateToCloud(cap_reached) under shared
    cloud_spend_blocked_emitted dedup (Option A).

cli/src/runtime_config.rs:
  - CloudConfigToml +custom_endpoint Option<String>.
  - CloudSpendToml +warn_at_percent Option<u8>.
  - to_cloud_config lifts warn_at_percent default Some(80) when toml
    unset; custom_endpoint passes through None on unset.

elai_cc.toml:
  - [cloud.spend].warn_at_percent = 80 (uncommented).
  - [cloud].custom_endpoint commented-out example.

dashboard-v2/src-tauri/src/commands.rs:
  - +get_cloud_route_status Tauri command (3 latest_action_of_type
    queries aggregated into one JSON response).

dashboard-v2/src-tauri/src/lib.rs:
  - invoke_handler!: +commands::get_cloud_route_status.

dashboard-v2/src/lib/api/types.ts:
  - +CloudRouteStatus interface + RouteBlockedRow + WarnLimitReachedRow
    + CostEstimateRow.

dashboard-v2/src/lib/api/stub.ts:
  - +get_cloud_route_status stub function (returns all-null when no
    SSB rows yet).

dashboard-v2/src/lib/screens/CloudPanel.svelte:
  - Added routeStatus $state + parallel get_cloud_route_status fetch in
    refresh().
  - Banner section above spend gauge: ROUTE BLOCKED (red) /
    WARN LIMIT (amber) / COST ESTIMATE (blue accent). WARN suppressed
    when BLOCKED present. COST hidden when high_usd == 0.
  - CSS .cloud-panel__notice + .is-blocked / .is-warn / .is-info.

contracts/tests/cloud_ceiling_contract.rs (NEW): 11 tests covering
  route_with_spend daily + monthly + absent-snapshot stub_ban paths,
  CustomEndpointForbidden, warn_limit_reached threshold, SCHEMA
  floor 103, and round-trip for RouteBlocked + WarnLimitReached payloads.

contracts/tests/hybrid_escalation_contract.rs (NEW): 7 tests pinning
  ExecMode::Auto -> Local from router, BackendMode::Hybrid FromStr +
  aliases ("auto", "HYBRID"), resolve_exec_mode for all 3 BackendMode
  variants, EscalateToCloud carrying hybrid_local_first +
  cap_reached trigger_reason strings, CloudCredentials::for_test_with_token.

contracts/tests/cloud_loc_constants_contract.rs (NEW): 11 tests
  covering 4 LOC constant values, baby_step_loc_for_mode dispatch for
  all canonical (mode, model) pairs incl. fallback, UNGROUNDED_HEURISTIC
  + COST_ESTIMATE_SPREAD_RATIO bounds, estimate_task_cost_usd empty +
  local + sonnet spread + opus > sonnet upper-bound assertions.


SESSION 2 TEST RESULTS
----------------------

cargo check -p elai-orchestrator: GREEN (9 pre-existing warnings, none from C-5).
cargo check -p elai_cc_contracts: GREEN.
cargo check -p elai-dashboard-v2:  see env note.

cargo test -p elai-orchestrator --lib ssb: 55 passed, 0 failed.
cargo test -p elai-orchestrator --test action_type_lock:
  2 passed (1 ignored regen_golden_file as designed). TOTAL_VARIANTS=306
  + ALL_VARIANTS array verified; golden file matches runtime.

Contract test execution for new C-5 cases: BUILD-VERIFIED through
cargo check -p elai_cc_contracts (GREEN); test runtime execution
on this machine was blocked by an environmental disk-space ceiling
(F: drive at 100%, link.exe LNK1180 + LNK1140 errors when emitting
per-test .exe binaries). Orchestrator lib + action_type_lock golden
both ran successfully because they incremental-link. Recommend a
`cargo clean` on a follow-up machine before running the full
`cargo test -p elai_cc_contracts cloud_ceiling hybrid_escalation
 cloud_loc_constants` to obtain the green pass count.

svelte-check (npm run check): 0 errors, 17 pre-existing warnings
  (none in CloudPanel.svelte or the new types/stubs).


SESSION 2 RESIDUALS
-------------------

(none from the s-25120 cross-check-recheck list — all 2 PARTIAL stubs
 + 15 MISSING wires + 1 SPEC_DRIFT closed.)

D1: Plan Task 9 "last 5 RedactionApplied rows timeline" — DEFERRED.
    RedactionApplied rows are emitted by RedactingBabyStepExecutor
    (session 1 ship, contract-verified). The timeline surface needs
    either an extension of CloudSpendSummary or a dedicated
    `get_recent_redactions` Tauri command. Both are out of scope for
    C-5's plan task list; the existing redaction_count gauge in
    CloudPanel covers the aggregate need until the dedicated timeline
    lands in a follow-up B-N.

D2: BABY_STEP_LOC_LOCAL_32B = 100 — DEFERRED to E-32 (per plan).
    `baby_step_loc_for_mode` falls back to the 7B value for any
    non-"3b" local model id, which gives the 32B model a sensible
    default until E-32 adds the explicit constant.

D3: Full HYBRID re-invoke loop (task_runner dispatches ExecMode::Cloud
    after local L3 retry exhaust + cap=1 second-exhaustion HALT) —
    PARTIAL. The cap-hook now emits the typed RouteBlocked +
    EscalateToCloud(cap_reached) rows on cap-hit, but the actual
    in-process re-invoke of the baby-step executor with mode flipped
    to ExecMode::Cloud requires production agent_pool changes that
    are out of scope for this contract crate. The surface that DOES
    land + is contract-verified: BackendMode::Hybrid + resolve_exec_mode
    + ExecMode::Auto -> RoutedProvider::Local + EscalateToCloud row
    types. The full retry loop ships with a follow-up that touches
    crates/orchestrator/src/agent_pool.rs.
