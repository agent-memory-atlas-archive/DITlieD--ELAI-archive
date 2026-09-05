# Inference Engine Dashboard Wiring Audit
Date: 2026-04-25
Auditor stance: HOSTILE — every wiring claim verified against source.
Spec: F:\.ELAI_workflow\docs\thefinalglance\summary\phase-3-summary.md PHASE 3J (21 sub-steps).
Implementation audits cross-checked: phase-3J.md, phase-4K.md.
Backend: F:\.ELAI_workflow\.elai_cc\crates\dashboard\src\{lib.rs, commands.rs}
Frontend: F:\.ELAI_workflow\.elai_cc\crates\dashboard\src-frontend\src\{Settings.svelte, SidecarStatus.svelte, StatusBar.svelte, ActivityEntry.svelte, drawers\ProviderVramDrawer.svelte, drawers\ToolsHooksDrawer.svelte, drawers\AntiStalenessDrawer.svelte, windows\SettingsWindow.svelte, windows\ActivityWindow.svelte}
Sidecar: F:\.ELAI_workflow\.elai_cc\python\elai_inference\server.py
Providers crate: F:\.ELAI_workflow\.elai_cc\crates\providers\src\{vllm_client.rs, sidecar_client.rs, layer_config\steering.rs, multiplexing.rs, model_placer.rs}
Plugins crate: F:\.ELAI_workflow\.elai_cc\crates\plugins\src\mcp_bridge.rs
Orchestrator crate: F:\.ELAI_workflow\.elai_cc\crates\orchestrator\src\{ssb.rs, ssb_viewer.rs, task_runner.rs, provider_signals.rs, workload_profiling.rs, preload.rs, sandbox.rs}

================================================================================
OVERALL VERDICT: PARTIAL — three thin surfaces, one disconnected core
================================================================================

Phase 3J's dashboard surface is the Provider/VRAM drawer (SettingsWindow),
the Tools/Hooks drawer (ActivityWindow), the Settings.svelte "Providers"
panel (steering rows + MCP bridge row + tier/decoding line), the
StatusBar inline `optSummary` segment, and the SidecarStatus loading
strip. That is the entire 3J surface. Most of what 3J actually computes
in the Rust crates is INVISIBLE to the dashboard:

- Per-step provider signals (entropy, novelty, GG mean/min logprob) ARE
  emitted as `ProviderSignals` SSB rows by `emit_provider_signals_snapshot`
  (task_runner.rs:3105, 3557) and the activity feed renders them as
  "GG · entropy 2.4" rows (ActivityEntry.svelte:661). One real telemetry
  channel.
- VRAM tier detection IS emitted exactly once at task_runner startup via
  `emit_vram_tier_once` (task_runner.rs:2890, 4873). The drawer reads
  the latest row. One real telemetry channel.
- Search-trigger ETC events ARE emitted on demand (task_runner.rs:4856).
  Activity feed renders them. One real telemetry channel.
- Steering vector cache state is read from disk every Settings open
  (commands.rs:1590) and the recompute button calls the sidecar's
  `/v1/steering/compute` round-trip (commands.rs:1712). REAL — this is
  the single most-functional 3J end-to-end path on the dashboard.
- MCP bridge HTTP path is REAL but every other 3J producer is either
  inert (vLLM, ModelPool, GPU multiplex, predictive preload, MILP, T3
  hook, AppContainer runtime, workload profiling) or piped to the
  ActivityFeed only as a single-row breadcrumb (model_swap, vram_tier).

The drawers are graceful — they all render "loading…" or "no VRAM
detection emitted yet" instead of crashing — but the breadth of
backend-only state means the user sees roughly 6 of the ~25 distinct
3J signals listed in the spec.

================================================================================

## 3J.1 vLLM HTTP CLIENT SPIKE

Backend signals available:
- VllmHealthStatus {Healthy, Unhealthy{reason, code}} — vllm_client.rs:80,
  156-170. Polled by ModelPool placement loop.
- VllmConfig {server_url, tensor_parallelism, served_model_names} —
  vllm_client.rs:35-51.
- vLLM /generate latency, tokens/sec — derivable in vllm_client.rs:676
  via the Unhealthy branch but never published.

Dashboard consumers: ZERO. Grep for `VllmHealthStatus`,
`vllm_health`, `vllm_config`, `vllm_url` in `crates/dashboard/src/`
returns no matches. The Provider/VRAM drawer hard-codes
`sidecar_reachable` only — there is no separate vLLM health probe
anywhere in the Tauri commands surface. The dashboard cannot tell the
user whether vLLM is up, what model it has loaded, or what URL it's
talking to.

Verdict: INVISIBLE. The vLLM client compiles into the providers crate
but no Tauri command exposes it to the UI.

================================================================================

## 3J.2 vLLM ModelPool integration / model swap events

Backend signals:
- VllmModelPool struct (vllm_client.rs:398-447) — type-only scaffolding
  per the implementation audit. No load_model/unload_model HTTP wrapper.
- `SsbActionType::ModelSwap` is wired (ssb.rs:106, 308, 394, 594).
- ONE production writer: task_runner.rs:4784 inside
  `handle_autonomy_decision` — fires only when
  `AutonomyDecision::SubstituteModel` is emitted by the autonomous gate
  after `max_restarts_exhausted`. Hardcodes `gpu_id=0`,
  `swap_duration_ms=0`. This is a substitute-on-failure event, NOT a
  hot-swap progress event.

Dashboard consumers:
- ActivityEntry.svelte:247 colours `model_swap` rows as "info"; line
  1080 renders them as "model swap · {to_model[0..32]}".
- No drawer, no chart, no swap-history panel anywhere.

Verdict: SURFACED for the failure-substitution case (one row in the
activity feed when restarts are exhausted), INVISIBLE for any normal
predictive preload / ModelPool-driven swap. `swap_duration_ms=0` and
`gpu_id=0` mean even the failure-event row has no useful content
beyond the from/to model names.

================================================================================

## 3J.3 DoLa layer-contrast decoding

Backend signals (per implementation audit):
- DoLaSteeringProcessor lives in server.py L705 (not Rust). Per-role
  toggle and candidate_buckets in sidecar config.
- `OptimizationSummary.dola_enabled` (sidecar_client.rs:417) reads
  `SidecarConfig.dola_enabled` from the dashboard's sidecar provider.

Dashboard consumers:
- ProviderVramDrawer.svelte:97 — `<li>dola: {toggles.dola_enabled ? "on" : "off"}</li>`.
- StatusBar.svelte:438 — `optSummary.status_line` formats
  `"model: ... · [TQ:N, DoLa:on, Steer:ada]"` (sidecar_client.rs:444).
  Pre-formatted server-side; gated behind `debug.enabled`.
- Settings.svelte:669 — `decoding — dola: {decoding.dola}`.

Verdict: SURFACED but BOOLEAN-ONLY. The dashboard shows whether DoLa is
enabled in config — it does NOT show per-role toggle state, the
mature/premature layer indices being contrasted, the alpha value, or
any per-step hit/miss telemetry. The Python sidecar has all of this
data; nothing publishes it.

================================================================================

## 3J.4 TurboQuant evaluation

Backend signals:
- TurboQuantizer (providers/src/turboquant.rs:448) — Pi rotation +
  Lloyd-Max codebooks. Compose-eval results in
  `.elai_cc/metrics/compose_dola_tq_results.json`.
- `OptimizationSummary.turboquant_bits: u8` — the only bit that ships
  to the UI.

Dashboard consumers:
- ProviderVramDrawer.svelte:96 — `turboquant: {bits > 0 ? `${bits}-bit` : "off"}`.
- StatusBar.svelte:437 — `TQ:{turboquant_bits}` in the title attribute.
- Settings.svelte:670 — `decoding — turboquant: {decoding.turboquant}`.

Verdict: SURFACED at the boolean / bits-only level. No accuracy delta,
no compose-eval result snapshot, no per-layer quantization metric. The
JSON file in metrics/ never reaches the UI.

================================================================================

## 3J.5 Steering vectors (ActAdd)

Backend signals:
- SteeringVectorEngine (providers/src/layer_config/steering.rs:294) —
  Rust shim around sidecar `/v1/compute_steering`.
- Per-role vector cache on disk under `<project>/.elai_cc/...`.
- SidecarConfig.{actadd_enabled, steering_enabled} flags.
- SsbPayload::{SteeringComputed, SteeringComputeFailed} variants exist.

Dashboard consumers:
- Tauri command `get_steering_status` (commands.rs:1590) — returns one
  row per known role with cached/last_computed_ms/cache_bytes.
- Tauri command `recompute_steering_vector(role)` (commands.rs:1655)
  hard-codes 3 contrast pairs (blake3 vs blake2, orjson vs json, fetch
  vs XHR), 28 layers, calls `compute_vector_via_sidecar_role`. On
  success writes `SteeringComputed` SSB row; on failure writes
  `SteeringComputeFailed`.
- Settings.svelte:684-712 renders one table row per role with a
  Recompute button. `recomputeSteering(role)` plumbing is live.
- ActivityEntry.svelte:484 colours `steering_computed` as info,
  `steering_compute_failed` as warn.

Verdict: SURFACED end-to-end. This is the single most-functional 3J
path on the dashboard — config flag, on-demand recompute, real
sidecar round-trip, on-disk cache reflection, success/failure SSB
breadcrumb in the feed.

Caveats:
- Contrast pairs are hard-coded in the dashboard command (commands.rs:1698).
  The data file `data/steering_contrast_pairs.toml` exists but is not
  parsed (TOML not in providers-crate deps). The user cannot edit
  pairs via the dashboard — only by editing source.
- The `actadd_enabled` flag is shown but the actual ActAdd injection
  hook in the sidecar generation loop has no per-step telemetry going
  back to the SSB.

================================================================================

## 3J.6 GPU multiplexing + predictive pre-loading

Backend signals:
- MultiplexingPolicy + PreloadPolicy + SloTier + can_colocate
  (providers/src/multiplexing.rs).
- ModelPlacer / GreedyModelPlacer (providers/src/model_placer.rs).
- orchestrator/src/preload.rs uses GreedyModelPlacer.
- Per the implementation audit + deferral doc, runtime tokio::spawn
  pre-loading + VRAM checks are DEFERRED.

Dashboard consumers: ZERO. No Tauri command, no drawer panel, no
status row mentions placement decisions, SLO tiers, GPU id assignment,
or preload events. The activity feed has no `gpu_placement` /
`preload_decision` action type. The ProviderVram drawer shows VRAM
tier and free MB but never which model is on which GPU.

Verdict: INVISIBLE. The structures compile but the dashboard cannot
tell the user that any multiplexing decision was ever made.

================================================================================

## 3J.7 Workload profiling per agent per phase

Backend signals:
- WorkloadProfile (orchestrator/src/workload_profiling.rs:15) +
  WorkloadProfilePayload + SsbActionType::WorkloadProfile. The schema
  is wired (ssb.rs:42-44, 2639, 3920).
- workload_profiling.rs:67 constructs the payload — but a grep for
  callers of that constructor in the orchestrator crate returns no
  hits outside the file itself. Per the implementation audit deferral
  doc, "no agents produce workload telemetry yet."

Dashboard consumers:
- ActivityEntry.svelte:1141 maps `workload_profile` -> "workload · profile";
  line 384 colours it info.
- No drawer, no per-agent table, no per-phase chart.

Verdict: DEAD. The activity-feed renderer would show generic "workload ·
profile" rows IF any production code emitted them, but no production
code does. The drawer surface is missing entirely.

================================================================================

## 3J.8 MILP scheduling

Backend signals:
- GreedyModelPlacer is wired; no MILP solver in Cargo.toml.

Dashboard consumers: ZERO. No "scheduling decision" action type. No
drawer.

Verdict: INVISIBLE (correctly — MILP is formally deferred and the
greedy baseline has no surfaced output anyway).

================================================================================

## 3J.9 MCP bridge

Backend signals:
- DefaultMcpBridge::connect / list_tools / call_tool (mcp_bridge.rs:219-247)
  are placeholders that return empty.
- batch-ful-d added http_initialize / http_list_tools / http_call_tool
  (mcp_bridge.rs:274-400) with deny-list + sanitization. REAL JSON-RPC
  2.0 round-trip.
- McpConnectionManager (mcp_lifecycle.rs:98-150) with retry policy.
- SsbPayload::{McpServerConnected, McpToolCalled, McpServerDisconnected}.

Dashboard consumers:
- Tauri commands (commands.rs:1812, 1829, 1892, 1949, 2002):
  - `get_mcp_status` — feature_compiled + enabled + server_url +
    connected + available_tools. Calls `bridge.http_list_tools(&url)`
    when both `feature_compiled` and runtime flag are true.
  - `list_mcp_tools` — calls `http_list_tools`, writes `McpServerConnected`
    SSB row on the first success.
  - `call_mcp_tool(tool_name, args_json)` — calls `http_call_tool`,
    writes `McpToolCalled` SSB row on every invocation with elapsed_ms
    + outcome (success / security_blocked / error).
  - `set_mcp_enabled(enabled)` — flips runtime flag, writes
    `McpServerDisconnected` row when toggled off.
- Settings.svelte:713-766 — Server URL row + status label
  ("disabled (not compiled)" / "compiled · disabled" / "enabled · not
  connected" / "connected · N tools"), Enable/Disable button, expandable
  tool list (first 5 tool names).
- ActivityEntry.svelte:495-499, 1286-1326 — colour rules + label
  rendering for the three MCP action types.

Verdict: SURFACED end-to-end for read + toggle + tool listing. There
is NO frontend control to call_mcp_tool — that command is exposed but
no Svelte component invokes it. So the user can see what tools exist
on the MCP server but cannot invoke one through the dashboard.

Caveats:
- Status / connectivity is read on Settings panel mount only (no poll).
  An MCP server that goes down mid-session will show stale "connected"
  until the user navigates away and back.
- ELAI_ENABLE_MCP environment gate is read on dashboard startup; the
  runtime flag is in DashboardState. Toggle works without restart.

================================================================================

## 3J.10 Tiered static analysis hooks

Backend signals:
- T1Hook (tiered_analysis.rs:331), T2Hook (tiered_analysis.rs:594) with
  circuit_breaker_lines field. T3 not located.
- plugins/src/ipc_hooks.rs:205 wires TieredAnalysis; SSB row at line 345.
- Hook verdicts persisted via SSB.

Dashboard consumers:
- Tauri command `get_hook_verdict_log(limit)` (commands.rs:3501) →
  `viewer.hook_verdict_log(limit)`. Returns Vec<HookVerdictEntry>
  with seq, ts_ms, tier, verdict, rule_id, hook_point, step_id,
  message.
- ToolsHooksDrawer.svelte:49 invokes `get_hook_verdict_log({limit:20})`
  on first expand. Renders "{tier} · {verdict}" + dim {hook_point}
  per row.
- get_token_reclamation_stats (commands.rs:3523) feeds the same
  drawer's "reclamation totals" section (shortcoder + context_reduction
  saved totals).

Verdict: SURFACED. The 20-row list is real. T3 hook is silently absent
because it's not implemented; the drawer cannot distinguish "no T3
firings" from "T3 doesn't exist".

================================================================================

## 3J.11 AppContainer isolation (Windows)

Backend signals:
- AppContainerConfig + WindowsAppContainerSandbox (orchestrator/src/sandbox.rs).
- SsbPayload::IsolationMetric variant exists.
- Per implementation audit: code present, runtime untested.

Dashboard consumers:
- ActivityEntry.svelte:385 colours `isolation_metric` as info; line
  1135 renders "isolation · metric". No drawer panel, no per-agent
  isolation gauge.
- Grep for `appcontainer`, `sandbox`, `isolation` in
  `crates/dashboard/src-frontend/src/` returns only the activity-feed
  case statements above.

Verdict: INVISIBLE in practice. The activity-feed renderer would show
an "isolation · metric" row IF the orchestrator emitted IsolationMetric
payloads in production, but the only such emission paths are inside
example_payload tests (ssb.rs:2614, 3834, 4194). No production caller
fires IsolationMetric.

================================================================================

## 3J.12 Advanced SSB action types

Backend signals: ssb.rs lists 18+ action types per implementation audit.

Dashboard consumers:
- ActivityFeed/ActivityEntry handle every advertised type — the
  contract test `activity_entry_coverage_contract.rs` (path verified
  via grep) enforces this.
- ProviderSignals: rendered as "GG · entropy X.X" with colour rules
  (ActivityEntry.svelte:326-333, 661).
- SearchTriggerFired: rendered as "search trigger · {reason}" warn
  (ActivityEntry.svelte:334, 668-674).
- VramTierDetected: rendered as info breadcrumb (ActivityEntry.svelte:336).
- ModelSwap, ChatMessage, ChatResponse, McpServer*, McpToolCalled,
  SteeringComputed, SteeringComputeFailed all wired.

Verdict: SURFACED at the activity-feed level. Each action type has a
case in the colour switch and the label switch. This is the broadest
3J surface on the dashboard.

================================================================================

## 3J.UNIFIED — single RuntimeClient, ConversationRuntime end-to-end

Backend signals:
- ConversationRuntime → RuntimeClient(Local) → sidecar /v1/messages →
  SSE stream is wired in the CLI binary (per implementation audit).
- The release dashboard binary runs the same sidecar at :8100.

Dashboard consumers:
- The dashboard CHAT path (ChatWindow.svelte:195) calls
  `submit_chat_message(text, intent)` (commands.rs:2219). That command
  ONLY writes a `chat_message` SSB row carrying the routed intent. It
  does NOT call /v1/messages, does NOT instantiate ConversationRuntime,
  does NOT stream tokens back to the UI.
- The dashboard does not surface live token streaming, prompt cache
  hits, inference latency, or tokens/sec for any conversation.
- Sidecar reachability: `get_sidecar_health` (commands.rs:2085) probes
  /health with 1s timeout + 5min resurrect window. SidecarStatus.svelte
  polls every 2s, renders loading / ready / error strip.
- StatusBar polls `get_active_optimizations` every 30s. Returns the
  pre-formatted `to_status_line()` string. Rendered behind debug
  toggle only (StatusBar.svelte:404, 442).

Verdict: PARTIAL. The dashboard tells the user the sidecar is alive
and roughly what optimizations are active, but the actual conversation
loop is delegated to the CLI/REPL. Dashboard chat writes intent rows
into SSB; nothing in the dashboard stack actually generates a model
response. This is by design (dashboard is observability + control,
REPL is interaction) but it means the user cannot watch a generation
stream from the dashboard.

================================================================================

## 3J.WIRE.* (Foundation / Budget / Rules / Session / Plan / Stubs)

These are CLI-side wires. Nothing in the dashboard surfaces:
- CliInteractiveGate state (per implementation audit, defined but
  never instantiated — dead code anyway).
- ContextBudget per-slot allocation per dispatch (slot_0/slot_1/
  phase_slot_tokens). Settings.svelte has a "VRAM budget (MB)" knob
  but no per-slot view.
- DkgQuerierAdapter "found" vs "fell back to EmptyDkgQuerier" —
  dashboard cannot tell the user that the rule engine is silently
  reading an empty DKG (the path bug at .elai_cc/.elai_cc/indexes/dkg.db
  vs .elai/sessions/dkg.db). KnowledgeDrawer renders the dkg.db
  contents as if everything were fine.
- SessionDag stub_ssb_seq null state — invisible.
- PlanSkeleton phase_slot_tokens contribution to budget — partially
  visible via Settings cloud-spend / VRAM budget, but the per-phase
  slot token contribution is not shown.

Verdict: CLI runtime concerns; the dashboard correctly does not try
to surface gate-instantiation state, but the DKG path bug is a
silent correctness issue the dashboard does not warn about.

================================================================================

## 3J SIDECAR INTERNALS — what the sidecar produces vs what the UI sees

The sidecar (server.py) is the actual model host. It emits, per
request via /v1/messages or /v1/completions:
- SSE event stream (token deltas) — consumed only by CLI/REPL.
- gg signal mean/min logprob — wrapped into ProviderSignals SSB row by
  task_runner. Visible in ActivityFeed.
- entropy mean/max — same path. Visible.
- novelty cosine — same. Visible.
- search trigger event with reason — visible as
  `search_trigger_fired` row.
- ETC LogitsProcessor disable state — booled into ProviderSignals.
  Visible.
- DoLaSteeringProcessor active layer indices, alpha, contrast deltas
  — NOT emitted to SSB. INVISIBLE.
- Token cache hit/miss, prompt cache fingerprints, KV cache stats —
  NOT emitted. INVISIBLE.
- model.generate() inference latency, tokens/sec — NOT emitted.
  INVISIBLE.
- /v1/steering/compute success + cache bytes — wrapped by the dashboard's
  `recompute_steering_vector` Tauri command. Visible via SSB
  `steering_computed` row + Settings table refresh.
- /v1/models response — NOT polled by the dashboard. The user cannot
  see which models the sidecar advertises. INVISIBLE.
- /shutdown endpoint — not surfaced as a UI button.

Of the ~15 distinct signals the sidecar produces in normal operation,
6 reach the dashboard (gg, entropy, novelty, search trigger, ETC
booleans, steering compute outcome). 9 are silently dropped at the
sidecar response boundary.

================================================================================

## OPTIMIZATIONS OVERVIEW (debug-mode StatusBar)

The single best summary line in the entire dashboard for 3J is the
StatusBar segment (StatusBar.svelte:429-440), gated behind
`debug.enabled`. It calls `get_active_optimizations` every 30s and
renders the sidecar's pre-formatted `to_status_line()`:

  `model: Qwen2.5-Coder-7B-... · [TQ:4, DoLa:on, Steer:ada]`

Title attribute (hover): adds HyDE + ActAdd flags. Behind a debug
toggle that defaults OFF — most users will never see this line.

Verdict: SURFACED but DEFAULT-HIDDEN. The Provider/VRAM drawer in
Settings shows the same data (less compactly) and is always visible.

================================================================================

## REAL vs MOCKED

Genuinely live data:
- Sidecar /health probe (1s timeout, 2s poll) — REAL.
- Provider/VRAM drawer toggles — REAL (read from
  `state.sidecar_provider.config()` once per drawer open).
- VRAM tier — REAL when the sidecar's `detect_vram_tier()` returned a
  value; "no VRAM detection emitted yet" otherwise.
- Steering status table — REAL (filesystem cache check).
- Steering recompute — REAL round-trip to sidecar.
- MCP status / tool list — REAL JSON-RPC round-trip when feature is
  compiled in AND runtime flag is on; deterministic stubs otherwise.
- Hook verdict log — REAL (last 20 SSB rows).
- ProviderSignals / SearchTriggerFired / VramTierDetected /
  SteeringComputed / McpServerConnected / McpToolCalled / ModelSwap
  rows in the activity feed — REAL when emitted.

Mocked / synthetic / never-emitted:
- WorkloadProfile rows — never emitted in production.
- IsolationMetric rows — never emitted in production.
- ModelSwap rows — only emitted on max_restarts_exhausted with
  hardcoded zeros for swap_duration_ms and gpu_id.
- AntiStalenessDrawer — pure graceful-empty stub, three "no data yet"
  rows + a deferral note. ZERO writers.
- vLLM health / model loaded / preload events — no Tauri command.
- GPU placement / multiplexing decisions — no surface.
- Per-step inference latency / tokens-per-sec / cache hit rate — no
  emission, no surface.

================================================================================

## DEAD WIRING AND SILENT FAILURES

1. AntiStalenessDrawer is a permanent stub. Three sections, each
   reading "no data yet". The drawer header advertises "deferred" so
   the user is not misled, but the drawer occupies real UI space.
2. ModelSwap rows hardcode `gpu_id=0` and `swap_duration_ms=0` even
   when those values would be meaningful. The activity feed renders
   "model swap · {to[..32]}" without ever showing duration or device.
3. WorkloadProfile + IsolationMetric have full type-and-render
   pipelines but ZERO production writers. The dashboard would render
   them correctly if any code emitted them; no code does.
4. `call_mcp_tool` Tauri command exists but no Svelte component
   invokes it. The user can list MCP tools but cannot call one from
   the UI. Cosmetic surface only for tool invocation.
5. `get_active_optimizations` returns the right data but is gated
   behind the `debug` toggle in StatusBar. Default-off means the
   single best 3J summary line is invisible to most users.
6. The StatusBar VRAM/RAM segment (line 405-417) is also debug-gated.
   The Provider/VRAM drawer remains the only always-visible source of
   tier info.
7. MCP status is read on Settings panel mount only. No poll. A
   server that disconnects mid-session shows stale "connected" until
   the user clicks elsewhere and back.

================================================================================

## ANSWERS TO REPORTING QUESTIONS

(1) Path written:
    F:\.ELAI_workflow\docs\thefinalglance\audit-dashboard\inference-wiring.md

(2) Overall verdict: PARTIAL. ~6 of ~25 producible 3J signals reach
    the UI. The three drawers (ProviderVram, ToolsHooks,
    AntiStaleness) plus the Settings "Providers" panel + StatusBar
    optSummary segment + SidecarStatus strip cover sidecar reachability,
    decoding-flag boolean states, steering vector cache + recompute
    button, MCP connection toggle + tool list, hook verdict tail, and
    activity-feed breadcrumbs for ProviderSignals / SearchTrigger /
    VramTier / ModelSwap / SteeringComputed / McpServerConnected /
    McpToolCalled. Everything else from the 21 sub-step surface is
    INVISIBLE.

(3) ProviderVram drawer shows REAL VRAM telemetry?
    PARTIAL. The `tier` and `free_mb` fields are real when
    `task_runner::emit_vram_tier_once` has fired (one row per
    drive_task call) — read by `viewer.vram_tier_state()`
    (ssb_viewer.rs:2037). When the row is absent the drawer renders
    "no VRAM detection emitted yet". The drawer is NOT a real-time
    VRAM utilization gauge — it shows the once-per-task tier
    detection, not the current free-MB value. The decoding toggles
    (dola/turboquant/steering/actadd/hyde/sidecar_reachable) are
    real config reads via `state.sidecar_provider.config()`.

(4) MCP bridge UI works end-to-end?
    PARTIAL. Read + toggle + list_tools work end-to-end via real
    HTTP JSON-RPC 2.0 round-trips when `feature="mcp"` is compiled
    and `ELAI_ENABLE_MCP=1`. SSB rows are written for connect /
    disconnect / tool_called events. However, `call_mcp_tool` is
    EXPOSED but UNUSED — no Svelte component invokes it. The user
    can see tools but cannot invoke them through the dashboard.
    Status check is mount-only (no poll), so disconnections show
    stale.

(5) Model swap events visible?
    BARELY. The `model_swap` action type exists in the activity feed
    with both colour rule and label renderer (ActivityEntry.svelte:247,
    1080). But the ONLY production writer is task_runner.rs:4784,
    fired exclusively when `AutonomyDecision::SubstituteModel` is
    returned by the autonomous gate after `max_restarts_exhausted`.
    Hardcodes `gpu_id=0` and `swap_duration_ms=0`. No predictive-
    preload swap, no ModelPool hot-swap, no vLLM model load event
    ever produces a model_swap row.

(6) 3J signals completely INVISIBLE:
    - vLLM health status / loaded model / served URL.
    - VllmModelPool load_model / unload_model events.
    - DoLa per-role toggle, alpha, mature/premature layer indices,
      per-step contrast hits.
    - TurboQuant per-layer accuracy delta, compose-eval results.
    - SteeringVectorEngine per-step injection effect (only the
      compute event is visible, not the apply event).
    - GPU multiplexing placement decisions.
    - Predictive pre-loading events.
    - WorkloadProfile per agent per phase (type wired, no emitter).
    - MILP scheduling decisions.
    - AppContainer isolation runtime metrics (type wired, no
      emitter outside tests).
    - T3 hook verdicts (T3 not implemented).
    - Per-step inference latency / tokens-per-sec.
    - Prompt cache fingerprint / hit rate.
    - KV cache stats.
    - /v1/models advertised list.
    - Live token streaming for any conversation (dashboard chat is
      intent-routing only, not generation).
    - Per-slot ContextBudget allocation post-dispatch (the budget
      knob in Settings is INPUT-only).
    - DkgQuerierAdapter fall-through warning (silent correctness bug
      from the implementation audit).
    - vLLM /generate latency / throughput.
    - ModelPool swap-progress percentage.
    - AntiStaleness dep audit / baseline harness / deterministic
      audit results (graceful stub).
