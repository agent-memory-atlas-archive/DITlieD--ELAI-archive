D-26 — Reasoning template archive dashboard surface (completion-drive closeout)

Session: s-25120 (completion-drive)
Plan: docs/thefinalglance/D-plan/D-26_reasoning_template_dashboard.md
Substrate state at start: ~85% shipped (LineageGraphView + ArchiveExplorerView + commands_d26.rs::{get_archive_lineage, get_template_archive_combined} + 5 upstream commands_dN.rs already in tree at SCHEMA_VERSION 144).

This decisions file captures the cleanup-pass decisions taken to close the remaining D-26 surface gaps without disturbing the shipped substrate. The 4 deliverables this pass landed are:
  - ArchiveVerifyGauge.svelte (NEW component, not drawer)
  - TripwireStatusStrip.svelte (NEW component, not drawer)
  - get_archive_verify_status Tauri command (additive in commands_d26.rs)
  - ArchiveVerifyStatus TS interface (additive in types.ts)
  - ReasoningDrawer wiring (2 new component mounts at top of drawer)

DECISION-1 — NO SCHEMA bump. SCHEMA_VERSION stays at the in-tree value (144 at session start). All D-26 cleanup is read-only projection over existing SSB rows (D-1 ArchiveChain on disk, D-4 tripwire_fired / tripwire_fired_concurrent / template_auto_reverted rows in SSB, D-8 archive_verify_result row in SSB). No producer side touched. No new variant. The action_type_lock.rs TOTAL_VARIANTS does not move.
Grounding: code-read of crates/orchestrator/src/ssb.rs lines 1234-1252 (D-8 SCHEMA_VERSION 115 already lists archive_verify_result in the SQL allowlist + the SsbActionType enum at line 5565 + the SsbPayload enum at line 9157), and existing commands_d26.rs::get_template_archive_combined which already composes recent_rows_of_type('tripwire_fired'). The new command is a 4th projection over the same surface.

DECISION-2 — ArchiveVerifyGauge + TripwireStatusStrip live in components/, not drawers/. These are inline widgets composable into any panel — they have no DrawerHost route, no `bootstrap()` / `teardown()` lifecycle, no global state hook, and no drawer-level routing entry. The shipped precedent for inline status widgets in components/ (FeasibilityPanel.svelte, RefusalRateGauge.svelte, ConfidenceTag.svelte) settled the convention. The plan-text said "drawer" for both — drift accepted in favour of the shipped convention.
Grounding: ls of crates/dashboard-v2/src/lib/components/ vs drawers/ at session start. Components contains 23 inline widgets; drawers contains 40 entries, all of which are drawer-tier (BabyStepPanel, KnowledgeDrawer, ProvidersDrawer, etc., each routed via DrawerHost).

DECISION-3 — LineageGraphView shipped uses cytoscape, plan text said plain-SVG. Drift accepted. The plan-text was authored before D-1 shipped its chain-walker DTO; the shipped LineageGraphView consumes ArchiveLineage.entries directly and renders a richer DAG view than plain-SVG would allow. No code change this pass (the file is shipped); decision recorded so future plan-walkers don't try to "fix" it back to SVG.

DECISION-4 — Drawers/views location split. LineageGraphView + ArchiveExplorerView shipped to views/ (not drawers/). This was the D-26 author's call at original ship time. ArchiveVerifyGauge + TripwireStatusStrip ship to components/. The location convention is therefore:
  - views/        — full-screen content surfaces routed by App-level navigation
  - drawers/      — DrawerHost-routed side drawers with bootstrap/teardown lifecycles
  - components/   — inline widgets composable into any of the above
The D-26 surface deliberately spans views/ (full archive explorer) + components/ (inline health gauges) — both layers, no drawer.

DECISION-5 — get_archive_verify_status is non-blocking, read-only SSB projection. The Tauri command consumes the single most-recent archive_verify_result row via viewer.recent_rows_of_type('archive_verify_result', 1) and returns the empty default (verified=false, last_verified_ts_ms=None) when no row exists or the viewer is unavailable (fresh repo). No separate state machine. No producer logic. The producer remains crates/cli/src/archive_verify.rs (D-8 CLI driver), unchanged by this pass.

DECISION-6 — D-26 follows the additive commands_dN.rs pattern, not the plan-text "edit-of-commands.rs" pattern. The shipped commands_d26.rs already established the pattern; the new get_archive_verify_status appends to the same file rather than migrating into commands.rs. Registration in lib.rs lands alongside the two existing D-26 registrations.

DECISION-7 — Polling-based refresh, not push-subscription. ArchiveVerifyGauge polls every 30s (matches archive-verify cadence — verify is a manual CLI invocation, not a hot stream). TripwireStatusStrip polls every 10s (matches the CanaryPanel / commands_d{1..6} precedent — D-7-decisions.md DECISION-3 captured the original 10s settling). Both polling intervals are tunable via component props (pollMs).
Grounding: drawers/CanaryPanel.svelte line 112 sets `pollHandle = setInterval(reload, 10_000)` — the canonical D-tier dashboard cadence.

DECISION-8 — NO new SSB variants. Composite reads only. The full D-26 surface is realisable as projections over the 4 existing variant families (ArchiveEntry on-disk, TripwireFired / TripwireFiredConcurrent / TemplateAutoReverted, ArchiveVerifyResult, BaiArmUpdated). Adding a new variant would mean: action_type_lock.rs bump, SCHEMA_VERSION bump, SQL allowlist edit, payload struct, all_test_payloads() entry, and a producer wiring. None of that is needed — the existing producer surface is complete; the dashboard just lacked the read projections.

DECISION-9 — `errors` field projection: short kind labels only. The on-wire `ArchiveVerifyError` is an externally-tagged enum (e.g. `{"HashMismatch": {"line": 2}}`). The gauge only renders a few error-kind chips, so the projection extracts the variant name (first object key) into a `Vec<String>`. The richer MutationArchivePanel (already shipped) can read the full typed payload via the raw SSB row when a deep view is needed — the gauge does not duplicate that surface.

DECISION-10 — ReasoningDrawer mount is the chosen home for both new widgets (GAP-6 affirmative). The drawer already surfaces archive-chain reasoning state (GRC budgets, mistake book, REFLECT timeline), and the archive verify + tripwire glances are part of the same operational health story. Mounted at the top of the drawer template, before the GRC BUDGETS section, so they act as a header-level health glance.

OUT-OF-SCOPE for this pass:
  - commands_d26.rs::{get_archive_lineage, get_template_archive_combined} — shipped, no change
  - LineageGraphView.svelte / ArchiveExplorerView.svelte — shipped, no change
  - D-4 TripwireFiredConcurrent SSB variant — shipped at SCHEMA_VERSION 108
  - D-8 ArchiveVerifyResult SSB variant — shipped at SCHEMA_VERSION 116
  - Any producer-side work — all 5 producer surfaces remain unchanged

Files touched this pass:
  - .elai_cc/crates/dashboard-v2/src-tauri/src/commands_d26.rs (extended: ArchiveVerifyStatus struct + project_archive_verify_status fn + get_archive_verify_status command + 4 tests)
  - .elai_cc/crates/dashboard-v2/src-tauri/src/lib.rs (1 invoke_handler row added for the new command)
  - .elai_cc/crates/dashboard-v2/src/lib/api/types.ts (ArchiveVerifyStatus interface added at file tail)
  - .elai_cc/crates/dashboard-v2/src/lib/components/ArchiveVerifyGauge.svelte (NEW, ~210 LOC)
  - .elai_cc/crates/dashboard-v2/src/lib/components/TripwireStatusStrip.svelte (NEW, ~245 LOC)
  - .elai_cc/crates/dashboard-v2/src/lib/drawers/ReasoningDrawer.svelte (2 import lines + 2 component mounts at the top of the template)
  - workflow/reasoning/D-26-decisions.md (this file)
