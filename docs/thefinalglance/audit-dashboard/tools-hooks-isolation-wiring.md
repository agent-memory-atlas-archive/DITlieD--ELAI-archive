# Tools / Hooks / Isolation Dashboard Wiring Audit
Date: 2026-04-25
Auditor stance: HOSTILE — every wiring claim verified against source.
Spec: F:\.ELAI_workflow\docs\thefinalglance\summary\phase-4-summary.md (4A L5-188, 4B L192-287, 4C L290-429).
Implementation audits cross-checked: phase-4A.md, phase-4B.md, phase-4C.md, phase-4K.md, _AGGREGATE.md.
Backend: F:\.ELAI_workflow\.elai_cc\crates\dashboard\src\{lib.rs, commands.rs}
Frontend: F:\.ELAI_workflow\.elai_cc\crates\dashboard\src-frontend\src\{ActivityFeed.svelte, ActivityEntry.svelte, StatusBar.svelte, drawers\ToolsHooksDrawer.svelte, drawers\OrchestrationDrawer.svelte, drawers\ReasoningDrawer.svelte, windows\ActivityWindow.svelte}
Producers: F:\.ELAI_workflow\.elai_cc\crates\orchestrator\src\{ssb.rs, ssb_viewer.rs, task_runner.rs, hook_dispatcher.rs, agent_pool.rs, sandbox.rs, ipc_gateway.rs, pipeline_tree_navigator.rs}, crates\plugins\src\{ipc_hooks.rs, tiered_analysis.rs}, crates\tools\src\{dispatcher.rs, tiered_validation.rs, ipc_registry.rs, git_tool.rs, validation.rs, trace.rs, edit_matching.rs, file_read.rs, compile_check.rs, failure_taxonomy.rs, integration_check.rs}, crates\knowledge\src\shortcoder.rs

================================================================================
OVERALL VERDICT: PARTIAL — three live channels, one cosmetic drawer, isolation invisible
================================================================================

The user-facing surface for Phase 4A/4B/4C is exactly:

- ToolsHooksDrawer (hosted in ActivityWindow): renders `hook_verdict_log`
  (last 20 HookEvent rows) + aggregated `TokenReclamationStats`
  (ShortCoder + ContextReduction totals). Real, but text-only and
  collapsed by default.
- ActivityEntry switch cases for tool_dispatch / validation_tier /
  failure_classified / git_commit / daira_trace / shortcoder_compression
  / hook_event. Inline rows in the activity feed.
- ActivityFeed `recent_commits` strip (top of feed) calling
  `get_recent_commits` -> SSB-filtered `git_commit` rows. Real commit
  list, capped at 3 entries.
- StatusBar `git` segment polling `get_git_status` (libgit2 branch +
  ahead-of-origin + dirty flag). Real.
- ReasoningDrawer's "last failure" line via `get_failure_taxonomy_last`
  + GRC budgets via `get_grc_budgets`. Real but shows only the LATEST
  taxonomy row, not the 13-category distribution.
- OrchestrationDrawer's checkpoints + handoffs via
  `get_agent_checkpoints` / `get_session_handoffs`. The agent_id field
  is just a string label — there is no per-agent isolation tier
  indicator anywhere in this drawer.

What is NOT surfaced anywhere in the dashboard:

- Process isolation tier (A / B / C) — no command, no drawer, no badge.
- Sandbox state (enable_sandbox flag, namespaces, AppContainer,
  rlimits, /proc masking) — zero references in dashboard sources.
- IPC frame stats / SO_PEERCRED handshake / accept loop — moot, since
  per the 4A audit the orchestrator has no live accept loop.
- ToolFilter rejections — no IPC ToolRequest path runs in production
  (4B audit), so no ToolFilter SSB row exists to render.
- T1 / T2 / T3 latency budgets — no metric, no drawer panel.
- ShortCoder per-step rows (the `get_shortcoder_effect` command is
  registered at lib.rs:1490 but ToolsHooksDrawer omits it — drawer
  comment L4-6 says "future tiers may add a sub-tab"). Aggregated
  total is shown; per-step detail is invisible.
- DAIRA trace markers — only `get_dair_trace_last` is wired as a
  command (commands.rs:3203). No drawer or panel renders it; a search
  across drawers/ + windows/ for `get_dair_trace_last` returned ZERO
  callers. The 13 of 15 DAIRA markers missing per the 4B audit are
  irrelevant because the dashboard never reads even the 2 that are
  emitted ("enter" / "exit").
- Edit-matching tier hits (exact / whitespace / ellipsis) — emitted
  inside `tool_dispatch` rows as substring data only; ActivityEntry's
  parser at L740-769 extracts `tool` + `outcome` + `elapsed_ms` but
  never separates the EditMatchTier discriminant.
- Failure taxonomy histogram — only the last entry renders. The 13
  categories live in ActivityEntry's substring-decode list (L796-810)
  but no aggregate counter exists.
- IntegrationTestRunner per-level results — `IntegrationCheckpoint`
  and `IntegrationRunResult` (pipeline_tree_navigator.rs L1322-1342)
  live ONLY in `TreeNavigatorState` in memory. No SsbActionType
  variant, no Tauri command, no drawer panel reads them. The
  CommandIntegrationRunner's per-level timeouts (5/30/60/120s) are
  enforced silently.
- Stop / PreCompact verdicts as distinct events — both fire as
  generic `hook_event` rows that the ToolsHooksDrawer renders
  identically to PreToolUse/PostToolUse rows. The verdict-trigger
  semantics ("Stop blocks non-IDLE", spec) live in the dead
  IpcHookDispatcher path, not in the live InProcessHookDispatcher,
  so the dashboard cannot show "Stop blocked because FSM != IDLE."

The drawers are graceful — they render "loading…" or "no hook
verdicts yet" instead of crashing — but the user sees roughly 6 of
the ~20 distinct 4A/4B/4C signal channels in the spec.

================================================================================

## 4A.1 IPC types + agent binary skeleton

Backend signals available:
- `AgentDispatchConfig` with role, model_preference, tool_filter,
  rule_domains, grc_budget, sandbox_config (ipc-types/src/lib.rs L125-134).
- `IpcMessageType` (15 variants enforced by const at L182).
- `AgentHeartbeat` every 5 s, frame protocol 1 byte version + 4 byte LE
  length + postcard payload, MAX_FRAME_SIZE 10 MiB.
- `AgentPoolBabyStepExecutor` spawns `elai_cc_agent` via
  `Command::spawn` from dashboard/src/lib.rs L1191-1196.

Dashboard consumers: ZERO.
- No Tauri command exposes the spawned agent's pid, role, dispatch
  config, heartbeat status, or model_preference. The
  `OrchestrationDrawer` agent_id row (line 108) is a string label
  pulled from `AgentCheckpointEntry.agent_id` — not a live agent
  inventory.
- No "Agents" panel anywhere; grep across `crates/dashboard/src/` for
  `AgentDispatchConfig`, `IpcMessageType`, `AgentHeartbeat`,
  `agent_pool` returned zero hits beyond the lib.rs spawn block.

Verdict: INVISIBLE. The dashboard cannot tell the user (a) which
agents are running, (b) what role each holds, (c) when the last
heartbeat arrived, or (d) whether the elai_cc_agent binary was
located. The spawn is logged via `tracing::info!` (lib.rs:1197)
but no UI consumer reads logs.

================================================================================

## 4A.5 Sandbox (Linux clone+seccomp / Windows AppContainer) + IPC accept

Backend signals:
- `LinuxSandbox::launch_agent` (sandbox.rs L934-1039) implements
  clone(NEWNS|NEWPID|NEWNET|NEWUTS) + seccomp BPF.
- `WindowsAppContainerSandbox::create()` (sandbox.rs L326-693) +
  CreateRestrictedToken + Job object.
- `enable_sandbox: bool` on AgentPoolConfig.

Dashboard consumers:
- dashboard/src/lib.rs L1183 hard-codes `enable_sandbox: false`. There
  is no Tauri command to read this, no Settings toggle, no warning
  banner.
- The `IsolationMetric` SSB action type IS registered (ssb.rs:109,
  311, 397, 1580) and ActivityEntry has the case at L1135-1136
  rendering `"isolation · metric"` as info colour. BUT: grep across
  the orchestrator crate for non-test emit sites of
  `SsbPayload::IsolationMetric(...)` returned ZERO hits — the only
  constructions are the SSB module's own serde fixtures (ssb.rs:3834)
  and proptest generators (L4194). `compute_isolation_metric`
  (ssb.rs:499) is a pure helper that no production code calls.

Verdict: DEAD. The sandbox code exists in the orchestrator crate but
is bypassed at spawn time, the `enable_sandbox` flag is hard-coded
off in the dashboard wiring without a UI surface, and the
`IsolationMetric` action_type renderer in the activity feed has no
producer to render. A user looking at the dashboard CANNOT determine
whether they are running Tier A, Tier B, or Tier C — the answer
("Tier A++") is not surfaced anywhere.

================================================================================

## 4A.5 IPC accept loop / SO_PEERCRED / frame stats

Backend signals: NONE — per the 4A audit, no UnixListener or
NamedPipeServer accept loop exists in `crates/orchestrator/src/`.
`IpcGateway::run_pre_tool_use_hook` and `run_post_tool_use_hook`
have ZERO non-test callers. SO_PEERCRED helper at
ipc_gateway.rs L605-647 is dead.

Dashboard consumers: N/A — there is nothing to consume.

Verdict: INVISIBLE (correctly — there is no IPC channel to
introspect). The cost: anything the spec promised would flow through
ToolRequest/ToolResult/AgentHeartbeat (per-tool latency, rule
violations attributed to a tool call, heartbeat-timeout kill
reasons) cannot reach the dashboard via that path. Whatever the
dashboard does show comes from the second, in-process dispatcher
path described under 4B and 4C.

================================================================================

## 4B.1-4B.4 Tool dispatcher (git2 / 5-tier validation / DAIRA / edit / ShortCoder / failure taxonomy / file read / incremental compile)

### git auto-commit per baby step

Backend signals:
- `InProcessToolDispatcher::dispatch_git_commit` (dispatcher.rs
  L374-509) using git2 native, deterministic message
  `"elai baby_step_{n}: {count} files"` (task_runner.rs:3723).
- SsbActionType::GitCommit + GitCommitPayload with commit_sha +
  files_count.

Dashboard consumers:
- Tauri `get_recent_commits` (commands.rs:741) filters
  `recent_actions` by `SsbActionType::GitCommit`, default page 32.
- ActivityFeed.svelte L80-86 polls `get_recent_commits` every
  500 ms (POLL_MS), trims to 3 entries (`RECENT_COMMITS`), renders
  inline strip above the main feed.
- ActivityEntry.svelte L816-826 parses the row summary into
  `commit · {sha8} · {n} files`, colour teal (L370).
- StatusBar.svelte L184 calls `get_git_status` for branch +
  ahead-of-origin + dirty flag via libgit2 directly (commands.rs:658).

Verdict: SURFACED end-to-end. THE strongest 4B path on the
dashboard. The user sees real commit SHAs flowing in real time after
each baby step.

Caveats:
- No commit graph; no parent linkage; no diff-on-click on a commit
  row. The ActivityEntry click opens DiffViewer for `file_write`
  rows, not for `git_commit` rows.
- The "WIP commits for large phases (>5 edits)" branch (per spec
  4B.4) is not implemented in the producer (4B audit), so the
  dashboard naturally cannot show it.

### 5-tier validation pipeline

Backend signals:
- `DefaultTieredValidator` (tiered_validation.rs L102-320) with 5
  tiers, short-circuit on first failure.
- SsbActionType::ValidationTier + ValidationTierPayload (passed,
  elapsed_ms, tier discriminant).

Dashboard consumers:
- ActivityEntry.svelte L771-794 parses tier ∈ {determinism,
  quality_ast, unit, integration, stress} + passed bool + elapsed_ms,
  renders `{tier} · ok|fail · {n}ms`. Colour pass / warn (L363-366).
- Tauri `get_recent_validations` (commands.rs:716) filters
  `recent_actions` by `SsbActionType::ValidationTier`. NO frontend
  consumer for this command — grep across `src-frontend/src/` returned
  zero hits. The dedicated "validations strip" implied by the
  command name does not exist.

Verdict: PARTIAL. Inline activity-feed rows render correctly, but
the dashboard has no per-tier histogram, no stress-tier flakiness
counter, no per-tier latency p50/p95. The user can see "unit · fail
· 240 ms" land but cannot drill into "which tier failed most this
session."

### DAIRA structured trace

Backend signals:
- `task_runner.rs::emit_daira_trace` (L4314) writes
  `SsbPayload::DairaTrace { phase, tool, args_hash, elapsed_us,
  outcome, step_id }`. Production phase values: only "enter" /
  "exit" (4B audit confirmed 13 of 15 spec markers absent).
- AdaptiveTool<T> wrapper not implemented per 4B audit.

Dashboard consumers:
- ActivityEntry.svelte L827-851 parses phase ∈ {enter, exit} + tool,
  renders `daira · enter · edit` / `daira · exit · edit`. Colour
  info (L371-374).
- Tauri `get_dair_trace_last` (commands.rs:3203) returns the last
  N DAIRA rows. NO frontend consumer — grep across drawers/ +
  windows/ returned zero callers. The Reasoning drawer holds a
  comment claiming DAIRA "fetched lazily on sub-tab click" but no
  sub-tab exists.

Verdict: PARTIAL (renderer present; aggregate panel dead). The
"developer-facing breadcrumb" colour (info per ActivityEntry comment
L372-373) means rows render but blend in. The 15-marker spec is
both unimplemented at the producer side AND unrendered on the
dashboard side; if the producer ever emits MUTATION / BRANCH /
PANIC / TIMEOUT / FINDING / PATTERN / etc., ActivityEntry will fall
through to the default renderer with no decoration.

### Edit-matching 3-tier (exact / whitespace / ellipsis)

Backend signals:
- `try_edit_tiers` (dispatcher.rs L131-198) with EditMatchTier
  enum returned through `EditOutcome`.
- Surfaced as `tool_dispatch` rows where `tool == "edit"`.

Dashboard consumers:
- ActivityEntry.svelte L740-769 extracts tool name + outcome +
  elapsed_ms. Does NOT parse the EditMatchTier discriminant —
  that field is dropped in the substring decode.
- DiffViewer renders the resulting file diff but has no "matched
  via whitespace tier" indicator.

Verdict: PARTIAL (operation surfaced; tier hit invisible). The
user sees "tool · edit · success · 12ms" but cannot tell whether
the agent's exact match landed cleanly or had to fall back to
ellipsis splice. NotMatched outcomes surface as
"tool · edit · error" with no SequenceMatcher hint (4B audit
confirmed the hint is not produced).

### ShortCoder R1-R10

Backend signals:
- `crates/knowledge/src/shortcoder.rs` with all 10 rules + tree-sitter
  AST traversal.
- `task_runner.rs::3633-3651` invokes ShortCoder after PostToolUse
  per changed file; emits `SsbPayload::ShortCoderCompression` via
  `emit_shortcoder_compression_to_ssb`.
- Aggregated counts via `SsbViewer::token_reclamation_stats`
  (tokens_in_total, tokens_out_total, saved_total, row_count) +
  `SsbViewer::shortcoder_effects` (per-row tokens_in / tokens_out /
  reduction_pct).

Dashboard consumers:
- Tauri `get_token_reclamation_stats` (commands.rs:3522) -> ToolsHooksDrawer
  L50, L78 ("{N} tok saved" header) + L86-90 (reclamation totals
  list section).
- Tauri `get_shortcoder_effect` (commands.rs:3511) is REGISTERED in
  invoke_handler at lib.rs:1490 but NO Svelte component invokes it —
  ToolsHooksDrawer header comment L4-6 admits "ShortCoder per-step
  rows are available via get_shortcoder_effect but not rendered by
  default to keep the drawer under its size budget; future tiers
  may add a sub-tab."
- ActivityEntry.svelte L645 also has a `shortcoder_compression`
  inline-row case (L317 colour pass).

Verdict: SURFACED-AGGREGATE / INVISIBLE-PER-ROW. The user sees a
running total of saved tokens at the drawer header and inline pass
rows whenever a compression occurs; the per-step reduction percentages
that would let a user see "this file compressed 80%, that one only
12%" are computed and queryable but not rendered.

### Failure taxonomy

Backend signals:
- 13 categories (drift from spec's 9 per 4B audit).
- SsbActionType::FailureClassified rows.

Dashboard consumers:
- ActivityEntry.svelte L795-815 substring-decodes 13 category
  strings, renders `classified · {lower_category}`. Colour warn
  (L367-368).
- ReasoningDrawer.svelte L96-101 calls `get_failure_taxonomy_last`,
  shows ONLY the latest single row as
  `{category} · {confidence%} · {step_id}`.

Verdict: PARTIAL. Inline row rendering catches every classification
event; the only aggregate view is "the most recent one." A user
who fails 7 different categories in a session cannot see the
distribution from the dashboard.

### File-read outline mode + incremental compile

Backend signals:
- `dispatcher.rs::generate_file_view` outline_only flag (L901-919)
  triggered when total_lines > 300.
- `dispatch_incremental_compile` 3-attempt retry per language
  (dispatcher.rs L671-731).
- Both surface as `tool_dispatch` rows with tool ∈
  {file_read_outline, incremental_compile}.

Dashboard consumers:
- ActivityEntry tool_dispatch parser at L741-748 lists both tool
  names; renders the generic `tool · file_read_outline · success ·
  Nms` row.

Verdict: PARTIAL. Operations surface; tier-internal state (outline
threshold hit, retry attempt index, deferred-classification reason
"can't find crate") is dropped at the typeLabel parser. The user
cannot tell from the dashboard that an incremental_compile took 2
retries before passing.

### IpcToolRegistry (DEAD CODE)

Per 4B audit, `IpcToolRegistry::register` is invoked nowhere in
production; the dashboard wires the parallel `InProcessToolDispatcher`.
The TypeScript-side `ToolFilter` rejection events (BLOCK rows when
an agent calls a forbidden tool) cannot exist because the path is
never reached.

Verdict: DEAD across the wiring stack. Spec-implied "ToolFilter
rejected: agent-X called bash but allow-list is [edit, validate]"
breadcrumbs are not generatable, and the dashboard has no
renderer for them.

### IntegrationTestRunner per-level results

Backend signals:
- `IntegrationCheckpoint` (pipeline_tree_navigator.rs L166) +
  `IntegrationRunResult` (L1334) appended to in-memory
  `TreeNavigatorState.integration_checkpoints` (L203).
- `run_integration_checkpoint` invoked by task_runner after each
  baby step (L3534).
- 5/30/60/120 s per-level timeouts via tokio::time::timeout
  (tiered_validation.rs L138).

Dashboard consumers:
- NO `SsbActionType::IntegrationCheckpoint` exists. grep across
  ssb.rs returned zero results.
- NO Tauri command reads `TreeNavigatorState.integration_checkpoints`.
- NO drawer panel renders the per-level pass/fail histogram.
- Failure rows DO surface as `failure_classified` with
  `category == "integration_checkpoint_failed"` (ActivityEntry L48,
  L260, L269) — that is the ONLY way the dashboard learns an
  integration checkpoint failed.

Verdict: INVISIBLE. The 4-level (BabyStep/Step/Phase/Concept)
runner is the centerpiece of 4B.6-4B.7 in the spec; the dashboard
only ever sees its failures as a generic taxonomy bucket. Pass
events are silent. The "60-second Phase-level cargo test" the
runner enforces never tells the user it ran.

================================================================================

## 4C Hooks (PreToolUse / PostToolUse / Stop / PreCompact + T1/T2/T3)

Backend signals:
- `InProcessHookDispatcher` (orchestrator/src/hook_dispatcher.rs)
  is the LIVE path. Wires `T1Hook` + `T2Hook` + `T3Hook` from
  `plugins/src/tiered_analysis.rs` + ShortCoder from
  `crates/orchestrator/src/shortcoder_hook.rs`.
- task_runner.rs production calls (per 4C audit):
  - L3338 `dispatch_pre_tool_use(invocation)` once per baby step
    against the natural-language step description (NOT against an
    IPC ToolRequest).
  - L3589 `dispatch_post_tool_use(invocation)` per changed file
    (NOT against an IPC ToolResult, NO ruff subprocess).
  - L3877 `dispatch_stop(invocation)` once per task — telemetry
    only; does NOT enforce non-IDLE block (lives only in dead
    `IpcHookDispatcher`).
  - L3898 `dispatch_pre_compact(invocation)` immediately after Stop
    — "MVP no-op" per code comment.
- Findings flow into SSB via `emit_hook_events_to_ssb`
  (hook_dispatcher.rs L609-650), one `SsbPayload::HookEvent` row
  per finding with `tier`, `verdict`, `rule_id`, `hook_point`,
  `step_id`, `message`.
- Error sanitization via `sanitized_block_message(category)`
  returns `"tool call rejected by policy (<category>)"` — category
  bounded to lowercase ASCII + underscore, max 64 chars
  (hook_dispatcher.rs L65-71, L526-541).

Dashboard consumers:
- Tauri `get_hook_verdict_log(limit)` (commands.rs:3500-3509) ->
  `SsbViewer::hook_verdict_log` -> filters SSB by
  `SsbActionType::HookEvent`, returns `Vec<HookVerdictEntry>` with
  `{seq, ts_ms, tier, verdict, rule_id, hook_point, step_id,
  message}`.
- ToolsHooksDrawer.svelte L43-65 calls it with `limit: 20` on
  drawer expand. Renders `{tier} · {verdict}` + dim `{hook_point}`
  per row.
- ActivityEntry.svelte L86-100 has a `hookTier(s)` helper that
  substring-matches T1 / T2 / T3 / Composite from the row summary
  and a `hookVerdict(s)` helper for ALLOW / WARN / BLOCK. L617-625
  renders inline `hook · T1 BLOCK` style rows.

### Per-event verdicts

Verdict: PARTIAL — drawer shows the live hook stream but loses
the event-type axis.

- The 4 hook events all serialize to the same `hook_event`
  action_type. The drawer line `{tier} · {verdict}` exposes T1/T2/T3
  but the `hook_point` field carries the spec's PreToolUse /
  PostToolUse / Stop / PreCompact name — and the drawer renders it
  as dimmed text only.
- A user trying to answer "did Stop fire? did it block anything?"
  must scan the dim hook_point column for "stop" entries. There is
  no count, no filter, no per-event panel.
- The semantic flaw bleeds through from the producer: the live
  Stop hook never blocks (telemetry-only per 4C audit), so the
  drawer can render `T1 ALLOW · stop` but never `T1 BLOCK · stop`
  even when FSM ≠ IDLE. The dashboard cannot show the spec-required
  Stop enforcement because the producer doesn't enforce it.
- PreCompact rows enter the same stream as no-op `T1 ALLOW`
  entries, indistinguishable from a routine PreToolUse pass except
  by the hook_point label.
- ruff subprocess (spec 4C.5 PostToolUse mandate) does not exist
  per 4C audit — the dashboard naturally has no ruff verdict to
  render.

### T1 / T2 / T3 latency budgets

Backend signals: `T1_LATENCY_TARGET_US = 5_000` constant in
ipc_hooks.rs:34 (5 ms — drift from spec's <1 ms per 4C audit). No
runtime measurement, no Criterion bench, no SSB row carries per-tier
elapsed time.

Dashboard consumers: NONE. No drawer field, no chart, no command.

Verdict: INVISIBLE.

### ShortCoder reduction percentages

Already documented under 4B above. Aggregate via
`get_token_reclamation_stats` is rendered. Per-row reduction_pct
via `get_shortcoder_effect` is NOT rendered.

### Cross-session lock

Backend signals: `IpcHookDispatcher::pre_tool_use` checks
`CrossSessionLockChecker` BEFORE tiered analysis (ipc_hooks.rs
L278-296), but `IpcHookDispatcher` is dead code. The live
`InProcessHookDispatcher` path does not consult the cross-session
lock.

Dashboard consumers: NONE. No "file locked by other session"
banner, no per-file lock indicator. Spec test c22-c24 covers the
behaviour in the dead path only.

Verdict: DEAD.

### Error sanitization

Backend: `sanitized_block_message(category)` is correctly bounded
(64-char ASCII allowlist) so escape sequences cannot reach the
agent. The ToolsHooksDrawer renders the sanitized `verdict` +
`hook_point` strings; if the producer ever emitted a row with a
malformed message, the drawer would render it literally without
HTML escaping (Svelte 5 escapes by default in `{...}` interpolations,
so XSS is not a worry, but a malformed control character could
break the row layout).

Verdict: SURFACED via the same drawer line; no separate
sanitization indicator.

================================================================================

## Hook-Wiring Truth Table (Dashboard view)

Hook event       | Live producer call                | Dashboard renderer
-----------------|-----------------------------------|--------------------------
PreToolUse       | task_runner.rs:3338 baby-step     | ToolsHooksDrawer line +
                 | (text, NOT IPC ToolRequest)       |   ActivityEntry hook · row
PostToolUse      | task_runner.rs:3589 per-file      | same drawer / feed
                 | (NOT IPC ToolResult, NO ruff)     |
Stop             | task_runner.rs:3877 telemetry     | same drawer / feed —
                 | (no non-IDLE block)               |   never BLOCK
PreCompact       | task_runner.rs:3898 no-op         | same drawer / feed —
                 | (no token_used > 0.85 trigger)    |   indistinguishable
                 |                                   |   from PreToolUse ALLOW

The drawer renders the LIVE, non-spec-compliant path. The
spec-compliant `IpcHookDispatcher` path has no production caller and
therefore no SSB rows for the dashboard to read.

================================================================================

## Per-Phase Visibility Score

| Phase      | Producer ready | Live wiring | Dashboard surface |
|------------|----------------|-------------|-------------------|
| 4A IPC     | Types + frame  | Skeleton    | INVISIBLE         |
| 4A sandbox | Code present   | Disabled    | INVISIBLE         |
| 4A IsoMet  | Helper exists  | No emit     | DEAD (renderer ok)|
| 4B git2    | git2 native    | Per babystep| SURFACED end-to-end|
| 4B 5-tier  | 5 tiers + s.c. | Per babystep| PARTIAL (rows only)|
| 4B DAIRA   | 2/15 markers   | enter/exit  | PARTIAL (no panel)|
| 4B edit    | 3 tiers        | Live        | PARTIAL (tier lost)|
| 4B Short   | R1-R10         | PostToolUse | SURFACED-aggregate|
| 4B failtax | 13 cats        | Live        | PARTIAL (last only)|
| 4B file    | outline >300   | Live        | PARTIAL (gen tool row)|
| 4B compile | 3 retries      | Live        | PARTIAL (gen tool row)|
| 4B ipcReg  | Code present   | DEAD        | INVISIBLE         |
| 4B integ   | 4 levels       | TreeNavigator only| INVISIBLE (failures only as taxonomy)|
| 4C hooks   | T1/T2/T3       | Live        | SURFACED via drawer (event axis lost)|
| 4C Stop    | telemetry only | Live        | SURFACED but never BLOCK|
| 4C PreCmp  | no-op          | Live        | DEAD (no trigger) |
| 4C cslock  | Dead path only | DEAD        | INVISIBLE         |
| 4C ruff    | Not implemented| -           | INVISIBLE         |

================================================================================

## Answers to report-back questions

(1) Path written: F:\.ELAI_workflow\docs\thefinalglance\audit-dashboard\tools-hooks-isolation-wiring.md

(2) Overall verdict: PARTIAL — three live channels (git_commit
inline + StatusBar git status, hook_event drawer, tool_dispatch /
validation_tier / failure_classified / daira_trace / shortcoder
inline rows), one cosmetic drawer (ToolsHooksDrawer = 124 LOC, two
lists), and a large invisible mass (process isolation, sandbox tier,
per-event hook axis, integration runner per-level results, edit
tiers, taxonomy histogram, T1/T2/T3 latency, ShortCoder per-row,
ToolFilter rejections).

(3) ToolsHooksDrawer renders real hook activity? YES — but only as
a flat list of the most recent 20 `hook_event` rows formatted as
`{tier} · {verdict}` + dim `{hook_point}`. The drawer is collapsed
by default, fetches on first expand, never re-polls. The 4 distinct
hook events (PreToolUse / PostToolUse / Stop / PreCompact) all
share the same render line; the only way to tell them apart is the
dimmed `hook_point` substring. Aggregated `TokenReclamationStats`
shows shortcoder + context_reduction totals — real, polled once on
expand. No per-step ShortCoder rows despite the command being
registered. No T1/T2/T3 latency budget. No live-stream / pulse
indicator on the drawer itself; the parent ActivityWindow has the
EXECUTING-state pulse but it targets activity feed rows, not
drawer entries.

(4) Git commits visible (commit graph or list)? YES — TWO surfaces.
ActivityFeed renders the top 3 commits as a strip above the main
feed via `get_recent_commits` polled every 500 ms; each row shows
`commit · {sha8} · {n} files` in teal monospace. StatusBar shows
the live git branch + ahead-of-origin count + dirty flag via
libgit2 (`get_git_status`). Inline `git_commit` action_type rows
also appear in the main activity feed with the same parser. NO
commit graph (no parent linkage), NO diff-on-click for commit rows
(DiffViewer hooks only `file_write` rows), NO commit body / message
detail beyond the deterministic `"elai baby_step_{n}: {count} files"`
auto-generated message. Per-commit list capped at 32 by the
backend `RECENT_PAGE_DEFAULT` constant; UI shows top 3.

(5) Sandbox / isolation tier visible to user? NO. There is zero UI
indication that:
- the `enable_sandbox` flag is hard-coded `false` in
  dashboard/src/lib.rs:1183;
- the orchestrator has no IPC accept loop (per 4A audit);
- the spawned elai_cc_agent runs in plain `Command::spawn` with no
  namespaces, no AppContainer, no rlimits;
- the `IsolationMetric` SSB action_type has no production emitter.
The `OrchestrationDrawer` shows `agent_id` strings from
`AgentCheckpointEntry` but no isolation tier badge, no "running
unsandboxed" warning, no sandbox-state column. A user reading the
dashboard would reasonably conclude the system is running at the
spec-promised Tier C; the ground truth (Tier A++ per 4A audit) is
not visible anywhere.

(6) 4A/4B/4C signals INVISIBLE in dashboard:

4A:
- agent process spawn events (binary location, pid, role, model_pref)
- isolation tier (A/B/C) — no badge anywhere
- sandbox state (enable_sandbox flag, namespaces present, AppContainer
  active, rlimits applied, /proc masked, bind mounts active)
- IPC frame stats (frames sent/received, frame size dist, version
  handshake outcome)
- SO_PEERCRED handshake result
- AgentHeartbeat liveness / timeout kills
- ToolFilter rejections — no IPC path runs in production
- AgentDispatchConfig (role, tool_filter, rule_domains, grc_budget)
- IsolationMetric (renderer present, producer absent)

4B:
- IpcToolRegistry handler dispatches (DEAD path)
- IntegrationTestRunner per-level pass results (failures surface as
  generic taxonomy rows; passes are silent)
- IntegrationCheckpoint state inside TreeNavigatorState (no SSB
  variant, no Tauri command)
- per-level integration latency targets vs actual (5/30/60/120 s
  budgets enforced silently)
- DAIRA markers beyond enter/exit (13 of 15 absent on producer side
  AND no aggregate panel exists)
- AdaptiveTool budget-based scope reduction (not implemented)
- failure taxonomy histogram across the 13 categories (only the
  latest entry is rendered in ReasoningDrawer)
- edit-matching tier hits (whitespace/ellipsis fallback) — discriminant
  dropped in ActivityEntry parser
- ShortCoder per-step reduction_pct (command registered, no UI
  consumer)
- incremental compile retry attempt index + DEFERRED_CLASSIFICATION
  reason
- file_read outline_threshold trigger reason
- 5-tier validator histogram / per-tier latency

4C:
- per-event hook axis (PreToolUse / PostToolUse / Stop / PreCompact
  rows are rendered identically; only dim hook_point distinguishes)
- T1 / T2 / T3 latency budgets and actual measurements
- HookVerdict::Escalate{ProviderTier} (variant doesn't exist; spec
  SCHEMA_REQUIRED unimplemented)
- cross-session lock blocks (DEAD path)
- ruff subprocess outcomes (not implemented)
- PreCompact "token_used > 0.85" trigger (no real trigger exists; no
  UI probe shows context budget vs limit ratio)
- circular-trust prevention rule promotion gate (no UI for
  PROPOSED-source=AUDITOR independent-validation requirement)
- pattern_hash determinism / collision indicator
- T2 5000-line circuit-breaker activation count

================================================================================

## SUMMARY

Phase 4A's dashboard surface is essentially a single "isolation_metric"
ActivityEntry case (L1135) with no producer behind it. The user
cannot see the tier, the sandbox state, the agent inventory, or any
IPC channel because the spec's Tier C plumbing has no live
orchestrator-side accept loop and the dashboard has no probe for
the in-process spawn-and-wait substitute.

Phase 4B's dashboard surface is the strongest of the three: git
commits flow in real time on TWO surfaces (feed strip + status bar),
validation_tier / tool_dispatch / failure_classified / daira_trace
/ shortcoder_compression all have ActivityEntry cases with
substring-decoded summaries, and ShortCoder's aggregated
TokenReclamationStats lands at the ToolsHooksDrawer header. What is
missing is everything that requires aggregation across rows
(taxonomy histogram, per-tier validation latency, per-level
integration pass/fail counts, edit-tier discriminants, retry
counts) and the 4-level IntegrationTestRunner is functionally
silent — failures only surface as a single generic taxonomy bucket.

Phase 4C's dashboard surface is the ToolsHooksDrawer's 20-row
HookEvent list. The 4 hook events all serialize to one
action_type (`hook_event`) and render to one line format
(`{tier} · {verdict}` + dim `{hook_point}`); a user cannot answer
"did Stop fire? did it BLOCK?" without scanning the dimmed column.
Stop never blocks because the live producer doesn't enforce
non-IDLE; PreCompact never fires meaningfully because the live
producer is a no-op; cross-session lock is invisible because that
path is dead code. ShortCoder PostToolUse wiring IS live and
flows into the same drawer header (token totals).

Net: the user sees real git commits + real hook verdicts + real
ShortCoder savings + real failure taxonomy (latest only) + real
tier-discriminated validation rows. The user CANNOT see process
isolation status, integration runner per-level pass rates, the
event-distinct hook activity, edit-tier matching falls-back, T1/T2/T3
latency vs budget, ToolFilter rejections, ruff subprocess outcomes,
cross-session locks, or any of the structural DAIRA markers
beyond enter/exit. The dashboard's per-spec coverage of Phase 4A is
near-zero, of 4B is moderate-with-aggregation-gaps, and of 4C is
moderate-with-event-axis-collapsed.
