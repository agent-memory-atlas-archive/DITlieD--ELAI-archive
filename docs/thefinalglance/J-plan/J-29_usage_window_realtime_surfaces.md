# J-29: usage window — the dedicated token-usage surface, the live mid-stream meter, and the canvas badge (the display half of the token-consumption window)

STATUS: PLANNED (WAVE GSW-2; dashboard-v2 + CLI tick + product canvas; BUILD, read-side — zero new SSB action types; depends J-28 for the full (model,task,session) keying, with a degraded mode over existing rows)

- Track J (repo-steal extension) | lever AUDITABLE + EFFICIENT | grounding STRONG (dashboard
  surfaces audited X1 2026-06-12; goose UI patterns via goose-main-INDEX.md, MEDIUM, ideas only)
  | nature BUILD | effort M-L | WAVE GSW-2 | depends J-28 (rollup keying; gauge + existing-row
  tabs are independent)

- Premise: every number the user asked to SEE already has (or gets, via J-28) a producer, and
  almost none has a view: CloudPanel/SpendDrawer show aggregate day/task dollars only — per-model
  NO, per-session NO, real-time NO (audit 4.1-4.2, C-5); BackendCostAccrued rows have NO panel
  (audit 4.6 BackendCostPanel ABSENT, C-44 feature-map entry); pricing_rollup has ZERO consumers
  (audit C-3); ContextStatePanel shows a percentage with no absolute counts (audit 5.2). J-29 is
  the activation + the one genuinely new mechanism: a live in-process gauge so the chat shows
  tokens DURING generation instead of a 5-10s poll after it. Steal file:
  workflow/reasoning/steal-goose-factory-2026-06-12.md (S-07, S-10, S-12..S-15).

## Steal/build
T1 USAGE WINDOW (S-13). New dashboard-v2 window UsageWindow.svelte: per-model / per-task /
  per-session / per-lane token + cost breakdown (J-28 rollup), cache effectiveness (read vs
  creation, the existing CloudPanel hit-rate formula reused), tool-call + file-op counts
  (read-side over existing SSB action types), and the per-turn record view (S-01 surfacing).
  Goose's CostTracker/ContextWindowIndicator are the pattern source (INDEX theme A/E), clean-room
  — no goose names, no goose code.
T2 TAURI READ COMMANDS (the C-3/C-4 activation). get_usage_rollup (J-28 ledger),
  get_backend_cost_rollup (BackendCostAccrued rows joined hash->name by iterating
  RuntimeBackend::all() — the resolver the audit says is missing), and the pricing_rollup
  read (per-phase/agent/tool dollar breakdown — FIRST consumer; the module's existing
  ELAI_AGENTIC_OBSERVABILITY_ENABLE gate is respected: gate off => the tab renders an explicit
  "observability gate off" state, never silent empty. No new gate is added; the existing one
  already has its allowlist row or keeps its DARK status honestly).
T3 LIVE USAGE GAUGE (S-07, audit C-5). In-process atomic counters updated by the stream
  consumers (cloud SSE + local streaming path) — running output-token count + turn input
  estimate. Surfaces: a get_live_usage Tauri command polled at 1s by the chat status line
  (CommandWindow) + a CLI tick on the live streaming path (a single rewritten status line, no
  scroll spam). Mid-stream values NEVER touch SSB (journal bloat ban — J-28 stub_ban); the
  per-turn UsageTurnEvent row remains the durable record; the gauge resets on turn end.
T4 LOCAL DISPLAY RULE (S-12). lane != cloud renders token counts ("in N / out M"), never a fake
  $0.00 (matches audit L-2 reality: cost_model_for(Sidecar) = None). Unknown-rate cloud models
  render "no rate for <model>" (J-28 T4 explicit None), never silent sonnet defaults.
T5 AUTONOMY RATIO (S-10). One derived read-side stat: auto-executed steps / total steps from
  existing gate/approval/intervention action types — labeled DERIVED with its formula in the
  tooltip; an LLM never computes it (TD-3 analog); pure journal arithmetic (GD-2 recomputable).
  Exact source action types enumerated at edit from the live ssb.rs registry.
T6 CANVAS BADGE (S-14). The product canvas (CANVAS WORKBENCH, Track 2) gets a usage badge op via
  the existing E-40 3-arm canvas_* dispatcher pattern (CanvasDispatcher): session token total +
  live gauge value when streaming. Harness canvas (.claude/mcp/canvas) is OUT of scope
  (recorded; separate artifact per the feature map).
T7 ABSOLUTE CONTEXT COUNTS (S-15, audit 5.2). ContextStatePanel additionally renders the
  absolute token count + limit beside the existing token_budget_pct badge — fields already in
  the context_budget_computed payload read path (commands.rs:8840-8845 audit; verify field
  availability at edit, else extend the read not the producer).
RIDERS (recorded, not tasks): goose's SSE-embedded TokenState (every server event carries usage)
  is the architectural sibling of T3 — ELAI's Tauri poll-on-gauge is the native equivalent; a
  push-event upgrade is a later option if 1s poll proves laggy (decide-on-evidence, GD-4 style).

## Existing ground truth (audit X1 2026-06-12 unless marked grep; re-grep at edit, W-7)
- CloudPanel.svelte polls get_cloud_spend 5s; day/task USD + cache tokens; NO per-model, NO
  per-session, NO real-time (audit 4.1). SpendDrawer same source 10s (4.2). MonitorWindow gauge
  row (4.5). These are KEPT — UsageWindow is the deep view, not a replacement (no-removal rule).
- CostPanel.svelte exists, polls velocity/cost-router/automix/free-tier (E-39 family), DARK
  behind ELAI_COST_TOKEN_ECONOMICS_ENABLE (audit 4.3) — NOT this unit's surface; untouched.
- BackendCostPanel ABSENT; BackendCostAccrued rows landing with hashed backend ids (audit 4.6 +
  grep runtime_orchestrator.rs:331-395 this session).
- pricing_rollup module exported, zero consumers, gated ELAI_AGENTIC_OBSERVABILITY_ENABLE
  (audit 2.4).
- ContextStatePanel shows pct only (audit 4.4/5.2); thresholds 70/85 from ssb.rs constants
  (commands.rs:8799-8803 audit).
- ssb.rs:3096 SCHEMA_VERSION 234 / action_type_lock.rs:676 TOTAL_VARIANTS 543 (grep this
  session) — J-29 adds NOTHING to either.
- Goose ground truth (W-3, via INDEX): CostTracker.tsx free-provider rule + tooltip split;
  ContextWindowIndicator.tsx thresholds; reply.rs:129-177 TokenState-in-every-event;
  output.rs:1367-1445 CLI bar + cost line.

## REVISED scope (W-10 tiers)
SHIP NOW Tier-1: T1/T2/T4/T5/T7 (pure read-side over journal rows + the J-28 ledger; fixture
  rows drive svelte-check + dashboard-smoker verification); T3 gauge wiring on the cloud path +
  the CLI tick (fixture-stream testable in-process).
DEFER Tier-2 (remote-sidecar-gated, W-10): the local-lane LIVE gauge confirmation (real
  streaming turn on the remote sidecar; same boot discipline as J-28 Tier-2 — flag before boot).

## Integration
- NEW crates/dashboard-v2/src/lib/windows/UsageWindow.svelte (+ window registration in the
  existing window host seam).
- crates/dashboard-v2/src-tauri (or the commands module the audit cites): get_usage_rollup /
  get_backend_cost_rollup / get_live_usage / pricing-rollup read (~200 LOC Rust total).
- stub.ts + types.ts updated in the same unit (Tauri<->Svelte symmetry; cross_lang_diff is the
  Layer-2 gate).
- crates/orchestrator: LiveUsageGauge (atomic counters + reset, ~80 LOC) hooked at the two
  stream-consumer seams; crates/cli live path tick (~30 LOC).
- CanvasDispatcher op + canvas surface badge (~60 LOC across the E-40 seam).
- ContextStatePanel.svelte absolute-count addition (~15 LOC).
- frontend_pair: this unit IS the frontend pair (of J-28); the Rust commands + the Svelte
  surfaces ship together.

## WIRING_CONTRACT
produces_symbols:
- UsageWindow.svelte (per-model/per-task/per-session/per-lane + cache + tool/file-op + per-turn
  view + autonomy ratio)
- Tauri commands: get_usage_rollup, get_backend_cost_rollup (hash->name resolver),
  get_live_usage, pricing-rollup read
- LiveUsageGauge (in-process, mid-stream, never journaled)
- canvas usage badge op (E-40 dispatcher arm)
- ContextStatePanel absolute token counts
consumes_symbols:
- usage_ledger rollup + UsageTurnEvent rows (J-28 — the primary feed; do NOT re-aggregate in TS,
  the Rust command returns the rollup)
- SsbViewer cloud_spend_summary path (existing; CloudPanel keeps it)
- observability/pricing_rollup.rs (first consumer; its gate respected, not duplicated)
- RuntimeBackend::all() (the hash->name join)
- CanvasDispatcher (E-40 3-arm op pattern)
reachable_from:
- the dashboard window host (UsageWindow registered + openable)  [PRODUCTION dashboard]
- the live chat status line (CommandWindow polling get_live_usage)  [PRODUCTION chat]
- the live CLI streaming tick  [PRODUCTION CLI]
- contracts/-only callers never satisfy this (R7)
stub_ban:
- NO SSB writes from the gauge or any mid-stream path
- NO TS-side re-aggregation of journal rows (the Rust rollup is the one source — GD-2)
- NO fabricated zeros: empty journal => explicit empty state; unresolvable backend hash =>
  "unknown backend (<hash>)" row, never a crash or a blank
- NO deletion/replacement of CloudPanel / SpendDrawer / CostPanel (no-removal rule)
- NO new env gate; NO LLM in any displayed-number path (TD-3 analog)
- NO .unwrap()/.expect() on prod paths
frontend_pair:
  this unit is the pair (J-28's dashboard half). stub.ts/types.ts updated in-unit;
  cross_lang_diff clean. Justified.
observable_effect:
  trigger: "dashboard-smoker flow: drive one cloud fixture turn + one local fixture turn (J-28
    triggers), open UsageWindow; then a streaming fixture turn with the chat status line visible"
  expect_ssb: "no NEW rows from this unit (read-side); the J-28 UsageTurnEvent rows it renders
    are recomputable from the window's displayed numbers (GD-2 spot-check)"
  expect_artifact: "UsageWindow renders per-model rows matching the journal; the local row shows
    token counts not dollars; during the streaming fixture the status line value visibly
    increments BEFORE turn end (C-5 closed). NEGATIVE (R2): zero-row journal renders the explicit
    empty state; a seeded unknown backend hash renders the unknown-backend row; observability
    gate off renders the gate-off notice; svelte-check + cross_lang_diff clean."

## DETERMINISTIC_CHECK
```bash
python .claude/tools/wiring_contract_check.py --bn-file docs/thefinalglance/J-plan/J-29_usage_window_realtime_surfaces.md --run-trigger
rg "UsageWindow" .elai_cc/crates/dashboard-v2/src -l                      # window exists + registered
rg "get_live_usage|get_usage_rollup|get_backend_cost_rollup" .elai_cc/crates/dashboard-v2 -l   # commands + TS stubs both sides
python .claude/tools/cross_lang_diff.py                                   # Tauri<->Svelte pair clean
rg -c "write_payload" <LiveUsageGauge file>                               # MUST be 0 (gauge never journals; bind real path at edit)
test -f workflow/reasoning/J-29-decisions.md
```

## Grounding
- ELAI: audit X1 2026-06-12 (dashboard sections 4-5 + gap list C-3/C-4/C-5) + this session's
  greps (runtime_orchestrator.rs emission, ssb.rs/action_type_lock.rs baseline). Anchors
  PLAN-asserted where audit-only; re-grep at edit (W-7).
- Goose: INDEX themes A/E (CostTracker free-provider rule, TokenState-per-event, CLI bar) —
  pattern source only, clean-room. S-46 DO-NOT-COPY: cost display ships ON.
- SSB baseline: LIVE 234/543 (grep 2026-06-12). J-29 = NET ZERO new action types (W-9 re-grep at
  edit anyway — J-28 lands +1 first in GSW sequencing).
- No new dependency (charts: reuse whatever the existing panels use — verify at edit; a chart
  dep that is not already in dashboard-v2 package.json requires user confirmation, default =
  hand-rolled bars like goose's CLI bar idea).
- Doctrine: W-4/W-6 (this IS the dashboard item), GD-2 (recomputable), GD-4 (poll-vs-push decided
  on evidence), TD-3, R7, no-removal.

## COMPOSES-WITH
J-28 (the feed), H-9 CloudPanel (kept, shallow view), C-44 (its dark rows finally surfaced —
the BackendCostPanel absence closed), E-39 CostPanel (untouched, separate concern), E-40 canvas
dispatcher (the badge op), phase-3E ContextStatePanel (absolute counts), H-13/Track-2 canvas
(surface host), dashboard-handoff skill (the verification flow). Restates none.

## Open questions / risks
- Poll (1s) vs Tauri event push for the gauge: start with poll (simplest, matches existing 5s
  panel idiom), upgrade on observed lag only (GD-4-style decide-on-evidence).
- Chart rendering: existing panels' approach first; no new chart dep without user confirmation.
- Autonomy-ratio source action types: enumerate from the live registry at edit; if the
  approval-shaped rows prove too sparse to be meaningful, ship the stat behind an "experimental"
  label or drop it (honest residue, GD-5).
- The window's per-task view depends on task_id propagation through J-28 rows on BOTH lanes —
  verify the local seam carries a task id at edit (runtime_orchestrator context).

## Next
J-29 makes the token window REAL for the user: the dedicated window, the chat live meter, the
canvas badge, and the CLI tick all render the same journal-derived numbers. After it, H-9's
remaining tier ($/resolved-task measurement) and E-39's gated economics panel have a home
surface to land on.

End of J-29.
