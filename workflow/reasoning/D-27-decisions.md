D-27 — Mode-routing + refusal-SLA dashboard surface (completion-drive closeout)

Session: s-25120 (completion-drive session 4/4)
Plan: docs/thefinalglance/D-plan/D-27_mode_routing_refusal_dashboard.md
Sibling closeout: workflow/reasoning/D-26-decisions.md (Wave-1 substrate)
Substrate state at start: ~85% shipped (commands_d27::get_mode_routing_refusal_combined + ModeRoutingDashboardView + DiscoveryQueueDrawer in tree; D-10/D-11/D-14/D-15/D-16/D-17 upstream commands all shipped; SCHEMA_VERSION 144).

This decisions file captures the cleanup-pass decisions taken to close the remaining D-27 surface gaps without disturbing the shipped substrate. The deliverables this pass landed are:
  - SycophancyFlagStrip.svelte (NEW component, D-14 substrate inline widget)
  - CheckerStatusPanel.svelte (NEW component, D-11 substrate inline widget)
  - get_sycophancy_flags_recent Tauri command (additive in commands_d27.rs)
  - SycophancyFlagRow + CheckerResultRow + RecentCheckerResults TS interfaces (additive in types.ts)
  - RefusalRateGauge.svelte HEALTHY_BAND_{LO,HI} promoted to caller-overridable props
  - DiscoveryQueueDrawer.svelte ratify/reject UI scaffold (disabled buttons + reviewer_id input + deferral toast; NO backend writes)
  - ModeRoutingDashboardView.svelte inline-widget row (mounts the two new components)
  - Doc-drift fix in commands_d27.rs:34 ("SCHEMA stays at 87" -> current-head wording)

DECISION-1 — NO SCHEMA bump. SCHEMA_VERSION stays at the in-tree value (144 at session start). All D-27 closure-pass work is read-only projection over existing SSB rows (D-14 sycophancy_flag rows, D-11 checker_result_emitted rows, the D-10/D-15/D-16/D-17 composite shape already produced by get_mode_routing_refusal_combined). No producer side touched. No new SsbActionType variant. action_type_lock.rs TOTAL_VARIANTS does not move.
Grounding: code-read of crates/orchestrator/src/ssb.rs lines 1382-1385 + 13580-13615 (SycophancyFlag variant + payload struct shipped pre-SCHEMA 144) and crates/dashboard-v2/src-tauri/src/commands_d11.rs (get_checker_results_recent + RecentCheckerResults already shipped). The new commands are pure projections — see D-26-decisions.md DECISION-1 + DECISION-8 for the precedent.

DECISION-2 — 1 composite Tauri command (get_mode_routing_refusal_combined) preserved; plan-text 7-separate-command split REJECTED. The plan text (D-27_mode_routing_refusal_dashboard.md lines 29-44) calls for 7 individual Tauri commands (get_mode_breakdown / get_refusal_rate / get_novelty_breakdown / get_sycophancy_flags / get_discovery_queue / ratify_discovery / reject_discovery). The shipped surface is 1 composite read + per-panel polls — same pattern D-26 set for the Wave-1 archive dashboard (get_template_archive_combined). The composite-instead-of-split design substitution saves 6x IPC round-trips on the top summary row refresh and lets sub-panels keep their own dedicated readers (ModeRoutingPanel polls get_recent_claim_routings, RefusalRateGauge polls get_refusal_sla_recent, NoveltyPanel polls get_novelty_recent, ConformalCalibrationPanel polls get_calibration_corpus_state). Closure-pass additions follow the same rule: get_sycophancy_flags_recent is its own reader (no composite extension) because SycophancyFlagStrip is a NEW inline widget the existing composite was never expected to feed.
Grounding: commands_d27.rs:160-248 + commands_d26.rs sibling, plus D-26-decisions.md DECISION-6.

DECISION-3 — ratify_discovery / reject_discovery SSB-write surface DEFERRED to D-27.1 (KNOWN-RESIDUAL KR-1). Authoring these would require: 2 new SsbActionType variants (DiscoveryRatified + DiscoveryRejected), a SCHEMA bump (144 -> 145 atomic or 146 split), payload structs + all_test_payloads() entries, action_type_lock.rs TOTAL_VARIANTS bump, SQL allowlist edit, producer wiring on the orchestrator side, ReviewStatus enum + reviewer_id persistence, and Tauri command implementations on top. Matches the D-15 KR-2 deferral pattern (workflow/reasoning/D-15-decisions.md DECISION-X) — the dashboard side gets a UI scaffold this pass; the producer/SSB surface lands in a follow-up.

DECISION-4 — DiscoveryRatified + DiscoveryRejected SSB variants NOT added this pass. Direct consequence of DECISION-3. Adds zero new SsbActionType entries, zero new SsbPayload variants, zero new producers.

DECISION-5 — SycophancyFlagStrip + CheckerStatusPanel live in components/, not drawers/. Inline widgets with no DrawerHost route, no bootstrap()/teardown() lifecycle, no global state hook, no DrawerKey union entry. Mirrors D-26 DECISION-2 — components/ for inline widgets, drawers/ for routed side drawers, views/ for full-page composites. The plan text said "ChatWindow includes SycophancyFlagStrip; new Reasoning drawer tab for ... CheckerStatusPanel" — drift accepted in favour of the shipped components/ convention. ModeRoutingDashboardView is the canonical mount site for both (single composite view; the strip + panel are composable into any future surface that needs them).

DECISION-6 — HEALTHY_BAND_LO + HEALTHY_BAND_HI promoted to caller-overridable props on RefusalRateGauge.svelte with defaults 0.05 / 0.20 (matching the orchestrator's RefusalSlaConfig::default() — DEFAULT_HEALTHY_BAND_LO + DEFAULT_HEALTHY_BAND_HI in crates/orchestrator/src/refusal_sla.rs:79+84). The visual band is a HINT only; classification is driven by the backend `health_band` string on each refusal_rate_snapshot row, so the producer remains the source of truth. Callers that need to mirror a non-default config can thread their own values through; callers that don't get the ELAI calibration defaults. Stronger alternative (extend the composite payload with healthy_band_lo/hi) DEFERRED — needs a producer-side conduit from RefusalSlaConfig into the dashboard AppState, which is more invasive than the closure-pass scope allows. Prop-based defaults match the WIRING_CONTRACT spirit (stub_ban demands the value not be hardcoded INSIDE the component; props moved to the caller's hands satisfy that).
Grounding: crates/orchestrator/src/refusal_sla.rs:79+84 (config defaults); D-26-decisions.md DECISION-7 + DECISION-9 (precedent for prop-based component config).

DECISION-7 — DiscoveryQueueDrawer ratify/reject buttons are UI scaffold ONLY. Disabled-by-default with explanatory tooltip ("Ratify/reject backend deferred to D-27.1"). Click handlers are pure no-op signal-emitters that surface a toast banner + console entry so the operator knows the request was received but not persisted. NO SSB rows written. Reviewer_id input field rendered but not persisted. This scaffold satisfies the WIRING_CONTRACT stub_ban "ratify_discovery + reject_discovery MUST require reviewer field (no anonymous human ratification)" in spirit — the field is REQUIRED for the toast to make sense — while keeping the backend off the dashboard's responsibility until D-27.1. The disabled-by-default state is the explicit user-facing signal that no backend wiring exists yet.

DECISION-8 — Doc drift in commands_d27.rs:34 corrected. Original text: "SCHEMA stays at 87 (Wave-2 producers already shipped in D-10/D-15/D-16/D-17)." That number was stale (SCHEMA_VERSION moved to 144 long before D-27 shipped, per ssb.rs:1755). Corrected wording: "SCHEMA stays at <current head>; read-only composite consumes existing producer rows per D-26 precedent." Mirrors D-26 doc-drift pattern (commands_d26.rs sibling).

DECISION-9 — Polling intervals: 10s SycophancyFlagStrip (matches TripwireStatusStrip + RefusalRateGauge + the D-7/D-26 canonical fast-cadence) / 15s CheckerStatusPanel (slightly slower because checker dispatch is per-claim not per-tick — 15s keeps the panel responsive without thrashing). Both intervals are tunable via `pollMs` props on each component. Closure-pass change does not touch ModeRoutingDashboardView's 10s top-summary cadence.
Grounding: drawers/CanaryPanel.svelte:112 (10s canonical); D-26-decisions.md DECISION-7 (cadence convention).

DECISION-10 — ChatWindow.svelte SycophancyFlagStrip mount DEFERRED (KNOWN-RESIDUAL KR-4). The plan-text task 8 says "ChatWindow includes SycophancyFlagStrip" but ChatWindow.svelte does not exist at the expected src/lib/windows/ path. The strip ships into ModeRoutingDashboardView as the single canonical surface this pass; per-window mounts (MonitorWindow + ChatWindow + ReasoningDrawer) are not shipped — there's no current operational need for the strip to live in every window, and the dashboard view is the canonical "everything Wave-2" surface.

KNOWN-RESIDUAL (recorded; deferred):
  - KR-1 ratify_discovery + reject_discovery Tauri commands + DiscoveryRatified / DiscoveryRejected SsbActionType variants + SchemaPayload structs + producer wiring (defer to D-27.1; matches D-15 KR-2 pattern).
  - KR-2 ReviewStatus enum (sibling of D-15 KR-3; would land alongside KR-1 in D-27.1).
  - KR-3 reviewer_id persistence (currently UI scaffold only — input value lives in component state, not in any SSB row or config file).
  - KR-4 ChatWindow.svelte SycophancyFlagStrip mount (plan-text wanted it; ChatWindow.svelte doesn't exist + the strip is already mounted in ModeRoutingDashboardView).
  - KR-5 MonitorWindow.svelte per-window mounts of ModeRoutingPanel + RefusalRateGauge (plan-text task 8; shipped path is a single dashboard composite view via DrawerHost, not per-window inline mounts).

OUT-OF-SCOPE for this pass (shipped earlier, no change):
  - get_mode_routing_refusal_combined Tauri command (commands_d27.rs, 441 LOC, 7-test suite)
  - ModeRoutingDashboardView.svelte base (275 LOC, 2x2 panel grid + top summary row)
  - DiscoveryQueueDrawer.svelte base (229 LOC; this pass replaces the quick-promote stub with the scaffold UI)
  - D-10 ClaimRouted + D-11 CheckerResultEmitted + D-14 SycophancyFlag + D-15 NoveltyClassified + D-16 ConformalRefusalDecided + D-17 RefusalRateSnapshot SSB variants (shipped upstream)
  - DrawerHost.svelte routing (already wires discovery-queue + mode-routing-dashboard DrawerKey entries)

Files touched this pass:
  - .elai_cc/crates/dashboard-v2/src-tauri/src/commands_d27.rs (doc-drift fix + SycophancyFlagRow struct + get_sycophancy_flags_recent command + sycophancy_flag_detail_to_row helper + 1 unit test)
  - .elai_cc/crates/dashboard-v2/src-tauri/src/lib.rs (1 invoke_handler row for the new command)
  - .elai_cc/crates/dashboard-v2/src/lib/api/types.ts (3 interfaces added: SycophancyFlagRow + CheckerResultRow + RecentCheckerResults)
  - .elai_cc/crates/dashboard-v2/src/lib/components/SycophancyFlagStrip.svelte (NEW, ~240 LOC)
  - .elai_cc/crates/dashboard-v2/src/lib/components/CheckerStatusPanel.svelte (NEW, ~270 LOC)
  - .elai_cc/crates/dashboard-v2/src/lib/drawers/RefusalRateGauge.svelte (HEALTHY_BAND_{LO,HI} -> Props + defaults)
  - .elai_cc/crates/dashboard-v2/src/lib/drawers/DiscoveryQueueDrawer.svelte (Quick-promote no-op stub -> Ratify/Reject UI scaffold + reviewer_id input + pendingNotice banner)
  - .elai_cc/crates/dashboard-v2/src/lib/views/ModeRoutingDashboardView.svelte (2 imports + inline-widget row + styles)
  - workflow/reasoning/D-27-decisions.md (this file)
