ELAI TOKEN-USAGE AND COST ACCOUNTING: CURRENT STATE AUDIT
Produced by: audit agent X1 / 2026-06-12
Scope: crates/core, crates/providers, crates/orchestrator, crates/dashboard-v2, python/elai_inference/server.py

All file:line references are verified from live source (codegraph + ranged reads).
WIRED = real producer running in production.
DARK  = struct/variant/module exists, env-gated or no production caller.
ABSENT = does not exist yet.


1. TOKEN CAPTURE
================

1.1 Core struct
---------------
File: crates/core/src/usage.rs:15

  pub struct TokenUsage {
      pub input_tokens: u32,
      pub output_tokens: u32,
      pub cache_creation_input_tokens: u32,
      pub cache_read_input_tokens: u32,
  }

UsageTracker (usage.rs:34): wraps a running cumulative + latest_turn pair.
record() called after each assistant turn. cumulative_usage() / current_turn_usage() exposed.
No cost field — pure counters, intentionally narrow per the C-14 comment.

1.2 Cloud lane — Anthropic-compat API (WIRED)
---------------------------------------------
Provider source: crates/providers/src/providers/anthropic.rs

The MessageStream SSE parser at anthropic.rs:1313-1334 listens to
StreamEvent::MessageDelta and captures usage from the Anthropic
message_delta event into self.latest_usage (anthropic.rs:1316).
The providers::types::Usage struct (types.rs:275) holds:
  input_tokens, output_tokens, cache_creation_input_tokens, cache_read_input_tokens.

In conversation.rs, build_assistant_message() at line 1020:
  AssistantEvent::Usage(value) => usage = Some(value)
...then at line 1040:
  ConversationMessage::assistant_with_usage(blocks, usage)
...and the UsageTracker.record(usage) is called on every successful turn
(conversation.rs:228-230 during runtime construction from persisted messages;
and on each new turn in the run loop).

The full usage pipeline from the wire for cloud:
  SSE stream -> MessageDeltaEvent { usage: { input_tokens, output_tokens,
  cache_creation_input_tokens, cache_read_input_tokens } } -> AssistantEvent::Usage(TokenUsage)
  -> UsageTracker.record() -> CloudSpend SSB row (see section 2 and 3).

Note: Anthropic streaming spec returns input_tokens only in message_start;
output_tokens only in message_delta. ELAI accumulates across both via the
Usage struct (types.rs:276-288 documents this explicitly). The full four-field
struct is populated before it hits SpendAccumulator.

1.3 Local sidecar lane (PARTIAL - known accounting divergence)
--------------------------------------------------------------
File: python/elai_inference/server.py

Streaming path (the main production local path, ~line 4720-4868):
  message_start event (line 4728-4733):
    "usage": { "input_tokens": input_token_count, "output_tokens": 0,
               "cache_creation_input_tokens": 0, "cache_read_input_tokens": 0 }

  message_delta event (line 4856-4863):
    "usage": { "output_tokens": actual_output_tokens }

  NOTE: message_delta does NOT carry input_tokens, cache_creation_input_tokens,
  or cache_read_input_tokens in the streaming path. Only output_tokens is in
  message_delta. This is Anthropic-spec-compliant for streaming, but ELAI's
  cost accumulator reads from the message_delta event (anthropic.rs:1315).
  The input token count from message_start is captured in a separate
  MessageStart event, not via AssistantEvent::Usage.

Non-streaming path (server.py:4964-4968) returns a full usage block:
  "usage": { "input_tokens": input_token_count, "output_tokens": len(tokens),
             "cache_creation_input_tokens": cache_creation_tokens,
             "cache_read_input_tokens": cache_read_tokens }

KNOWN ACCOUNTING DIVERGENCE (server.py:4959-4963 comment confirms):
  "input_tokens emits the FULL prompt token count (cache_creation + cache_read
  = input_tokens). Anthropic's spec emits input_tokens as the suffix-only
  (uncached) portion." ELAI consumers use the full count — intentional but
  diverges from the Anthropic billing definition, making local token counts
  non-comparable to cloud token counts.

Local sidecar cost: ZERO. cost_model_for(RuntimeBackend::Sidecar) returns None
(crates/providers/src/runtime/cost.rs:103). The runtime_orchestrator emits a
BackendCostAccrued row with est_cost_micros=0 for local backends
(runtime_orchestrator.rs:366-398). No dollar computation, no SpendAccumulator
path. Local inference is treated as free.

1.4 Where input_tokens is captured from the streaming sidecar
-------------------------------------------------------------
For the streaming local path, the message_start event carries input_tokens.
In ELAI's stream consumer (anthropic.rs, crates/providers/src/providers/anthropic.rs),
the SSE parser processes MessageStart events; these carry a Usage struct.
The parser's observe_event (anthropic.rs:1313) only reacts to MessageDelta to
capture usage. The MessageStart usage (with input_tokens) is carried in the
StreamEvent::MessageStart variant but the provider code does not explicitly
accumulate it into latest_usage via observe_event.

Consequence for local lane UsageTracker: in streaming mode, the AssistantEvent::Usage
produced from the sidecar's message_delta only carries output_tokens. The four-field
TokenUsage in the UsageTracker will have input_tokens=0 for streaming local calls
(only output_tokens is set). The BackendCostAccrued row at runtime_orchestrator.rs:372
uses response.usage tuple which comes from the runtime layer's (tokens_in, tokens_out)
pair — this is populated from the runtime's own usage parsing, not the SSE layer.


2. COST COMPUTATION
===================

2.1 Pricing model
-----------------
File: crates/providers/src/pricing.rs

ModelPricing struct (pricing.rs:32): input, output, cache_creation, cache_read
  costs in $ per million tokens.
pricing_for_model(model: &str) -> Option<ModelPricing> (pricing.rs:82):
  family-match on "haiku" / "opus" / "sonnet" substring in model id.
  Returns None for non-Anthropic families (e.g. "qwen", "gpt-4o").

Default pricing (Sonnet-tier, pricing.rs:25-28):
  input: $15/M, output: $75/M, cache_creation: $18.75/M, cache_read: $1.50/M

Haiku pricing (hardcoded fallback, pricing.rs:113-117):
  input: $1/M, output: $5/M, cache_creation: $1.25/M, cache_read: $0.10/M

Catalog source: config/models.toml embedded at compile time via
ModelCatalog::load_or_default() (model_catalog.rs:27, 155).
I-51: pricing.rs is now a VIEW over models.toml; constants stay as fallbacks.
Unknown model (e.g. any Qwen model id) -> pricing_for_model returns None
-> estimate_cost_usd falls back to default sonnet pricing (pricing.rs:163-164).

2.2 SpendAccumulator — cloud-lane cost tracking
------------------------------------------------
File: crates/orchestrator/src/cloud_mode.rs:646-753

One SpendAccumulator per cloud task. Four-rate accumulation:
  input_price_per_token, output_price_per_token,
  cache_read_price_per_token, cache_write_price_per_token.
SpendAccumulator::for_model() (cloud_mode.rs:683):
  only "claude-sonnet-4-6" has an explicit rate; all other model ids fall
  back to claude-sonnet-4-6 pricing (cloud_mode.rs:686-691).
  GAP: no per-model rate for haiku, opus, deepseek, or any local model.

cost_usd in SpendSnapshot (cloud_mode.rs:737) is the RUNNING TOTAL for the
task, NOT the per-turn delta. Each CloudSpendPayload row carries the
accumulated total so the dashboard can render "latest task" as the most
recent row.

2.3 BackendCostModel — multi-backend micro-dollar accounting
------------------------------------------------------------
File: crates/providers/src/runtime/cost.rs

BackendCostModel: integer micro-dollar (1e-6 USD) rates per 1k tokens.
Defined for Composer2 only (cost.rs:89-95):
  input: 500 micros/1k, output: 2500 micros/1k.
cost_model_for(backend) returns None for Sidecar/Mlx/LlamaCpp/Ollama/Vllm
and ALL other backends except Composer2. AnthropicApi's cost lives in
pricing.rs (acknowledged as "legacy, not consolidated here" at cost.rs:108-112).

2.4 Per-session rollup helpers (DARK)
-------------------------------------
File: crates/orchestrator/src/observability/pricing_rollup.rs

PricingRollupRow, rollup_cost_by_phase/agent/tool (pricing_rollup.rs).
No Tauri command exposes this rollup to the dashboard (grep of commands.rs
confirmed zero matches for "pricing_rollup"). The module is exported in
observability/mod.rs:75 but has no production consumer beyond unit tests.
Wire-dark: gated by ELAI_AGENTIC_OBSERVABILITY_ENABLE (observability/mod.rs:82).

2.5 CLI cost report
-------------------
File: crates/cli/src/resume.rs:328-342

format_cost_report(usage: TokenUsage) prints raw token counts only.
NO dollar figure in the CLI cost report — tokens only, no model-aware
pricing call. The live_cli.rs print_cost (live_cli.rs:2683) calls this.
Per-model dollar breakdown is absent from the CLI display.


3. SSB TELEMETRY
================

3.1 CloudSpend (WIRED, production)
------------------------------------
SSB action_type: "cloud_spend" (ssb.rs:9190)
Payload struct: CloudSpendPayload (ssb.rs:11522)
  provider, model, input_tokens, output_tokens,
  cache_creation_tokens, cache_read_tokens, cost_usd, task_id, ts_ms

Producer: grounding_client.rs:950-969
  Emitted after every successful cloud (Anthropic) send_message round-trip.
  cost_usd = SpendAccumulator running total (accumulates across the task).
  SSB writer attached via .with_ssb_writer() chained on CloudGroundingClient
  at task_runner.rs:6469-6475 (H-9 C10 keystone fix).
  Contract verified by h9_cost_cockpit_contract.rs.

CloudSpend is WIRED and emits real rows per cloud call since H-9.

3.2 BackendCostAccrued (WIRED for paid backends; WIRED with est_cost_micros=0 for local)
-----------------------------------------------------------------------------------------
SSB action_type: "backend_cost_accrued" (ssb.rs:9527)
Payload struct: BackendCostAccruedPayload (ssb.rs:18477)
  backend_name_hash (u64), role_name_hash (u64), tokens_in, tokens_out,
  est_cost_micros, ts_ms

Producer: runtime_orchestrator.rs:331-398
  Emitted after every successful RuntimeOrchestrator::try_complete call.
  For paid backend (Composer2 only): est_cost_micros from BackendCostModel.
  For local backends: est_cost_micros=0, fires when tokens_in>0 || tokens_out>0.
  Backend + role identity encoded as 64-bit hashes (not plaintext).

Dashboard consumer: NONE confirmed. No Tauri command reads backend_cost_accrued rows.
The payload exists, rows are written, but no panel surfaces them.

3.3 CostTokenEvent (DARK)
--------------------------
SSB action_type: "cost_token_event" (ssb.rs:9728)
8-arm discriminator: TaskAbortedBySpendCap / VelocityCapBreached /
  AutomixVerifierVerdict / CostRouterDecision / CostEstimateInaccurate /
  FreeTierQuotaApproaching / FreeTierQuotaExceeded / LayeredCapManualOverride

Gate: ELAI_COST_TOKEN_ECONOMICS_ENABLE=1 (cost/mod.rs:65)
Default: OFF (wire-dark). Dashboard CostPanel reads cost_token_event rows via
Tauri commands get_velocity_cap_status / get_cost_router_recent / etc.,
but those commands return empty until the env gate is set.

3.4 RouteBlocked (DARK)
------------------------
SSB action_type: referenced in cloud_mode.rs routing comments and EscalationReason::PrivacyBlock
but no dedicated RouteBlocked payload found; privacy blocks surface as
CloudRouteStatus.route_blocked in the SSB viewer read path.
CloudRouteStatus struct (ssb_viewer.rs) aggregates from existing SSB rows.

3.5 LocalCloudRoutingEvent (DARK)
----------------------------------
Gate: ELAI_LOCAL_CLOUD_ROUTING_ENABLE=1 (cost/cloud_route.rs:82)
Default: OFF.

3.6 EscalateToCloud (WIRED when cloud path fires)
--------------------------------------------------
SSB action_type: "escalate_to_cloud" implied from EscalateToCloudPayload (ssb.rs:11542)
Emitted when a baby-step is promoted from local to cloud execution.

3.7 Streaming real-time: NOT available
----------------------------------------
CloudSpend rows are written AFTER each complete round-trip (non-streaming with
respect to the token-count stream). The dashboard polls get_cloud_spend every
5-10 seconds. There is no SSE/event push for mid-stream token counts.
In-flight token progress (e.g. "currently generating N tokens") is not tracked.


4. DASHBOARD SURFACE
====================

4.1 CloudPanel (crates/dashboard-v2/src/lib/screens/CloudPanel.svelte)
-----------------------------------------------------------------------
Polls: get_cloud_spend (5s), get_execution_mode (5s), get_cloud_route_status (5s)
Data shown: task_usd, day_usd, cache_creation_tokens, cache_read_tokens, redaction_count
Cache hit rate (H-9): cacheHitRate = cache_read_tokens / (cache_read_tokens + cache_creation_tokens)
Route status banners: RouteBlocked, WarnLimitReached, CostEstimate
Daily cap gauge: dayPct = day_usd / capDailyUsd * 100
Provider switcher: LOCAL / HYBRID / CLOUD
WIRED: reads real CloudSpendSummary from SSB via SsbViewer::cloud_spend_summary.
PER MODEL: NO — only shows aggregate day/task totals. No per-model breakdown.
PER TASK: YES — task_usd = most recent CloudSpendPayload.cost_usd.
PER SESSION: NO — no session-scoped accumulation (day window only).
REAL-TIME DURING STREAMING: NO — poll every 5s only.

4.2 SpendDrawer (crates/dashboard-v2/src/lib/drawers/SpendDrawer.svelte)
--------------------------------------------------------------------------
Polls: get_cloud_spend (10s)
Data shown: task_usd, day_usd, cache_creation/read token counts, redaction_count
Same data source as CloudPanel, different layout. No per-model, no per-agent.
WIRED: yes.

4.3 CostPanel (crates/dashboard-v2/src/lib/components/CostPanel.svelte)
-------------------------------------------------------------------------
Polls: get_velocity_cap_status, get_cost_router_recent, get_automix_verdicts_recent,
  get_free_tier_quotas via direct Tauri invoke (not via stub.ts).
Data shown: velocity windows, cost-router decisions, AutoMix verdicts, free-tier quotas
GATE: ELAI_COST_TOKEN_ECONOMICS_ENABLE=1 required on orchestrator side for data.
WIRE-DARK by default. Does NOT show dollar totals or per-model token counts.

4.4 ContextStatePanel (crates/dashboard-v2/src/lib/components/ContextStatePanel.svelte)
------------------------------------------------------------------------------------------
Polls: get_context_state (5s)
Shows: token_budget_pct (0-100%), FSM state, runtime badge (local vs cloud_billed),
  prompt injection status
token_budget_pct derived from most recent context_budget_computed SSB row (usage_pct field).
Does NOT show absolute token count — only percentage. No cost information.
WIRED: yes.

4.5 MonitorWindow (crates/dashboard-v2/src/lib/windows/MonitorWindow.svelte)
------------------------------------------------------------------------------
Polls: get_cloud_spend, plus cloudSpend state variable shown as gauge row.
Condition: rows hidden when both task_usd and day_usd are 0.
Same cloud spend data as CloudPanel.

4.6 BackendCostPanel
---------------------
ABSENT: No component named BackendCostPanel found in dashboard-v2/src.
BackendCostAccrued rows exist in SSB but no UI panel surfaces them.

4.7 Per-user accounting
-----------------------
ABSENT: No per-user token accounting. ELAI runs as a single-user orchestrator.
CloudSpend rows have a task_id but no user_id field.


5. CONTEXT-WINDOW FILL
=======================

5.1 Context budget computation
--------------------------------
File: crates/orchestrator/src/context_budget.rs

CumulativeTokenTracker (context_budget.rs:795):
  prompt_tokens_total, completion_tokens_total, cache_read_tokens, cache_write_tokens
  Accumulates via record_response(ProviderUsage) (context_budget.rs:828-839).
  Also tracks last_response_prompt (for attention ceiling recalibration).
  Populated from grounding_client.rs:932-942 (cloud lane only, with SSB writer attached).

ContextBudgetComputedPayload emits "context_budget_computed" SSB rows.
The dashboard reads usage_pct from the most recent row
(commands.rs:8840-8845: recent_rows_of_type("context_budget_computed", 1)).

5.2 Dashboard display
----------------------
ContextStatePanel renders token_budget_pct as a percentage badge (green/yellow/red).
Thresholds: 70% = yellow, 85% = red (constants match ssb.rs CONTEXT_BUDGET_YELLOW_PCT /
CONTEXT_BUDGET_RED_PCT — verified via commands.rs:8799-8803).
NO absolute token count is shown. The panel shows the fill % only.
NOT live during streaming — the context_budget_computed row is emitted per-turn
after the full response, not mid-stream.

5.3 Auto-compaction threshold
------------------------------
auto_compaction_input_tokens_threshold (conversation.rs:171):
  read from ELAI_AUTO_COMPACTION_INPUT_TOKENS env var at startup.
  Fires when cumulative input_tokens crosses the threshold.
  Default: DEFAULT_AUTO_COMPACTION_INPUT_TOKENS_THRESHOLD constant.

5.4 Local vs cloud context tracking
-------------------------------------
The CumulativeTokenTracker is populated via the grounding_client code path only
(grounding_client.rs:932-942, inside `if let Some(ssb) = self.ssb_writer`).
For local sidecar calls routed through RuntimeOrchestrator (not grounding_client),
there is no CumulativeTokenTracker write — local lane context fill is not tracked
in the same SSB path. ContextStatePanel may show 0% for sessions using only the
local lane without SSB-writer-attached cloud calls.


6. GAP LIST
===========

CLOUD LANE GAPS
---------------
C-1 [PARTIAL] No per-model pricing for non-Anthropic cloud backends.
    SpendAccumulator::for_model() falls back to claude-sonnet-4-6 rates for
    any model id that is not literally "claude-sonnet-4-6" (cloud_mode.rs:686-691).
    haiku, opus, deepseek, composer2 are all mispriced in the cloud spend ledger.
    pricing.rs has haiku/opus rates but SpendAccumulator does not call pricing.rs.

C-2 [DARK] No per-session cost total.
    CloudSpendSummary.task_usd = most-recent row's cost_usd (not session total).
    day_usd = today's rows summed. There is no session-scoped or plan-scoped
    accumulation field. A session that runs multiple tasks cannot be totalled
    from the existing wire shape without scanning all rows and correlating by task_id.

C-3 [DARK] pricing_rollup (per-phase / per-agent / per-tool cost breakdown) has no
    Tauri command consumer. The module compiles and is exported but no dashboard
    panel reads it. Wire-dark until ELAI_AGENTIC_OBSERVABILITY_ENABLE=1 and a Tauri
    command is added.

C-4 [DARK] BackendCostAccrued rows (runtime_orchestrator.rs) are written but no
    dashboard panel surfaces them. The backend identity is hashed (u64), requiring
    resolution via RuntimeBackend::all() iteration — adding a panel requires a
    Tauri command that joins the hash back to a name.

C-5 [ABSENT] No real-time (mid-stream) token counter. Token counts only available
    after a complete turn. No SSE push for "currently at N tokens" during streaming.
    The dashboard poll cadence is 5-10 seconds.

C-6 [PARTIAL] CLI cost report (resume.rs:format_cost_report) shows token counts
    only — no dollar amount. pricing.rs is not called from the CLI cost path.

C-7 [ABSENT] No per-user accounting. No user_id in CloudSpendPayload. Entire system
    is assumed single-user.

C-8 [PARTIAL] CostTokenEvent (ELAI_COST_TOKEN_ECONOMICS_ENABLE) is DARK by default.
    Velocity caps, spend-cap abort, AutoMix verdicts, free-tier quotas are all
    behind the env gate. The CostPanel component exists and polls for this data
    but returns empty in a default installation.

C-9 [ABSENT] No per-task elapsed compute time tracking alongside token counts
    (wall-clock per inference call is not stored in the CloudSpendPayload).

LOCAL LANE GAPS
---------------
L-1 [PARTIAL / KNOWN DIVERGENCE] Sidecar streaming message_delta carries only
    output_tokens; input_tokens arrives in message_start but is not accumulated
    into AssistantEvent::Usage for the streaming path. The BackendCostAccrued row
    at runtime_orchestrator.rs uses (tokens_in, tokens_out) from response.usage tuple
    — whether this includes input_tokens from the message_start event depends on
    how the providers layer assembles the final Usage from both message_start +
    message_delta. This needs confirmation at the provider SSE-to-Usage assembly path.

L-2 [DARK] No local-lane cost. cost_model_for(Sidecar) = None. BackendCostAccrued
    rows have est_cost_micros=0. There is no electricity / GPU-time cost model for
    local inference. A "cost" for the local lane would require a custom rate table
    (e.g. $/GPU-hour * tokens/hour) — entirely absent.

L-3 [ABSENT] No tokenizer-based token count for non-inference paths (e.g. prompt
    construction, context summaries). The sidecar only reports token counts after
    an actual generation request. Pre-flight context size estimation uses the
    model's attention_ceiling / CumulativeTokenTracker but no tokenizer call.

L-4 [PARTIAL] Local lane context budget (CumulativeTokenTracker) is populated only
    via the grounding_client path (which requires the SSB writer to be attached).
    RuntimeOrchestrator-routed local calls do not feed CumulativeTokenTracker,
    so ContextStatePanel may show stale or zero context fill for local-only sessions.

L-5 [KNOWN DIVERGENCE] Sidecar input_tokens = FULL prompt token count (server.py:4959).
    Anthropic spec input_tokens = uncached suffix only. This makes token counts
    from the local lane non-comparable to cloud lane counts. No normalization
    or documentation surfaces this to the user at the dashboard level.

L-6 [ABSENT] No per-call latency tracking in the token accounting path. The sidecar
    returns elapsed_ms as a top-level field in non-streaming responses (server.py:4984)
    but ELAI does not store it in any SSB row or dashboard metric.

FEATURE-MAP CROSS-REFERENCE
----------------------------
H-9 COST COCKPIT [PARTIAL]: CloudSpend wired (H-9 C10 keystone done, task_runner.rs:6469).
  Cache hit-rate surfaced in CloudPanel. Missing: per-model breakdown, per-session total,
  real-time streaming counts. Tier-2 (live cloud run $/resolved-task measurement) DEFERRED.

BackendCostAccrued C-44 [WIRED but DARK on dashboard]: rows emitted by runtime_orchestrator.
  No Tauri command or UI panel reads them. Hashed backend IDs need resolver before display.

CloudSpend [WIRED]: grounding_client is the sole production emit site since H-9.

COST ECONOMICS HARD CEILING E-39 [DARK]: TaskAbortedBySpendCap, VelocityCapBreached
  etc. all require ELAI_COST_TOKEN_ECONOMICS_ENABLE=1. Default-off.

CONTEXT BUDGET ALLOCATOR phase-3E [PARTIAL]: ContextBudgetComputedPayload rows are
  written; ContextStatePanel shows pct badge. Missing: absolute token count display,
  per-lane (local vs cloud) context tracking, real-time during streaming.
