# D-26: ReasoningTemplate + archive dashboard surface

Objective: surface Wave-1 substrate (typed templates, archive, tripwires,
BAI, held-out, canary, archive verify) in dashboard-v2. Adds
MutationArchivePanel.svelte + LineageGraphView.svelte (cytoscape lineage
graph); extends ReasoningDrawer with PPM score + consensus rule + tripwire
status; adds Tauri commands get_archive_recent / get_archive_verify_status /
get_tripwire_status / get_bai_arms / get_canary_report.
Estimated effort: 2-3 sessions.
Pre-requisites: D-1..D-9 shipped (the rows to render exist).
Parallel-safe with: D-27 (orthogonal dashboard panels).

## Context

C-3 + C-4 (Tauri commands) established the dashboard-v2 surface pattern.
C-7 (UX completeness) defines size/font conventions; D-26 follows. D-26
exposes Wave-1 D-N artifacts to the operator without yet adding routing-
side panels (D-27).

Partial foundation present and usable:
- C-3/C-4 Tauri command pattern.
- C-6 design tokens (color + typography).
- C-7 ReasoningDrawer / MonitorWindow layout.
- C-24 cytoscape decomposition styles (reused for LineageGraphView).
- Wave-1 SSB rows are queryable via SsbViewer.

## Reads

- F:\.ELAI_workflow\docs\thefinalglance\C-plan\C-3_tauri_commands_knowledge_surface.md
- F:\.ELAI_workflow\docs\thefinalglance\C-plan\C-7_dashboard_ux_completeness.md
- F:\.ELAI_workflow\docs\thefinalglance\C-plan\C-24_symbol_impact_dashboard_surface.md
- .elai_cc/crates/dashboard-v2/src-tauri/src/commands.rs
- .elai_cc/crates/dashboard-v2/src/lib/state/drawers.svelte.ts
- All Wave-1 D-N files (artifacts being rendered)

## Tasks

1. **Tauri commands** in
   .elai_cc/crates/dashboard-v2/src-tauri/src/commands.rs:
   - `get_archive_recent(limit: u32) -> Vec<ArchiveEntrySummary>`: read
     ArchiveChain (D-1); return latest N entries.
   - `get_archive_verify_status() -> ArchiveVerifyStatus`: invoke
     ArchiveChain.verify_chain (D-1); return summary { total, verified,
     errors_count }.
   - `get_tripwire_status(window_minutes: u32) -> TripwireStatusReport`:
     query TripwireFired rows from D-4 over window.
   - `get_bai_arms() -> Vec<ArmPosteriorSummary>`: load BAI state (D-5);
     return active arm summaries.
   - `get_canary_report(template_id: Option<TemplateId>) -> Vec<CanaryReportSummary>`:
     load CanaryReport (D-7) for given template or last N.
   - `get_held_out_status() -> HeldOutStatusReport`: HeldOutStore (D-6)
     summary { active_count, retired_count, last_refresh_at }.

2. **MutationArchivePanel.svelte** at
   .elai_cc/crates/dashboard-v2/src/lib/views/MutationArchivePanel.svelte:
   - Lists recent ArchiveEntry rows: hash prefix + domain + pattern_kind
     + archive_status + RetirementReason (if Retired).
   - Per-row: click to expand → shows template_body excerpt + parent_id link.
   - Filter dropdown: archive_status (Candidate / Promoted / Retired / Frozen).
   - Color coding: Promoted=green, Candidate=yellow, Retired=red, Frozen=grey.

3. **LineageGraphView.svelte** at
   .elai_cc/crates/dashboard-v2/src/lib/views/LineageGraphView.svelte:
   - Cytoscape canvas showing ArchiveEntry parent-chain DAG.
   - Nodes = ArchiveEntry (color per status); edges = parent_hash links.
   - Click node → flip to MutationArchivePanel detail.
   - Style reuses cytoscape decomposition theme (C-24).

4. **TripwireStatusStrip.svelte** at
   .elai_cc/crates/dashboard-v2/src/lib/views/TripwireStatusStrip.svelte:
   - Live-polling strip in ReasoningDrawer; shows last 5 TripwireFired
     events with kind, timestamp, value vs threshold.
   - Color: red on TripwireFiredConcurrent / AutoRevert; yellow on individual
     TripwireFired.

5. **BaiArmsTable.svelte** at
   .elai_cc/crates/dashboard-v2/src/lib/views/BaiArmsTable.svelte:
   - Tabular display of active ArmPosteriors: template_id (prefix),
     pass_count, fail_count, posterior mean, 95% credible interval.
   - Sortable by mean / credible_lower; highlight current Promoted in green.

6. **HeldOutPanel.svelte** at
   .elai_cc/crates/dashboard-v2/src/lib/views/HeldOutPanel.svelte:
   - Per-family active count + recent HeldOutAutoRetire events.
   - Filter by Domain.

7. **ArchiveVerifyGauge.svelte** at
   .elai_cc/crates/dashboard-v2/src/lib/views/ArchiveVerifyGauge.svelte:
   - Live polls get_archive_verify_status; green dot + "Clean" or red dot
     + "Tampered (N errors)".
   - Click → opens full ArchiveVerifyReport.

8. **Drawer integration**: extend ReasoningDrawer.svelte to include the
   new panels (tabs or stacked sections).

9. **Real-time SSB subscription** for tripwire + archive events: reuse
   existing SSB event streaming pattern from C-7.

10. **Decision doc** workflow/reasoning/D-26-decisions.md (3 decisions):
    - (a) Why cytoscape (not D3 or chartjs): C-24 already uses cytoscape;
      consistency + reuse.
    - (b) Why per-component Svelte (not one mega-component): operator may
      want to dock individual panels separately; modular Svelte allows.
    - (c) Why color-coded archive_status: visual scan of archive health
      faster than text-only.

## WIRING_CONTRACT

produces_symbols:
  - Tauri commands: get_archive_recent, get_archive_verify_status,
    get_tripwire_status, get_bai_arms, get_canary_report, get_held_out_status
  - MutationArchivePanel.svelte + LineageGraphView.svelte +
    TripwireStatusStrip.svelte + BaiArmsTable.svelte + HeldOutPanel.svelte +
    ArchiveVerifyGauge.svelte
consumes_symbols:
  - ArchiveChain (D-1) + verify_chain (D-1 + D-8)
  - TripwireOrchestrator (D-4)
  - BaiPromotionEngine (D-5)
  - HeldOutStore (D-6)
  - CanaryRunner (D-7)
  - SsbViewer
reachable_from:
  - dashboard-v2 ReasoningDrawer + dedicated archive sub-window
stub_ban:
  - get_archive_recent MUST honor limit (defends against accidental full-
    archive return)
  - get_archive_verify_status MUST NOT block UI; verify_chain runs async
  - All Tauri commands MUST type-check via existing ts-rs bindings drift
    workflow
observable_effect:
  trigger: live cloud /task with template promotion + tripwire-fire injection
  expect_ssb: rows ingested by dashboard via SsbViewer subscription
  expect_artifact: MutationArchivePanel shows new entry within poll interval;
                   TripwireStatusStrip flips red on injection;
                   ArchiveVerifyGauge stays green on clean repo

## DETERMINISTIC_CHECK

```bash
rg "fn get_archive_recent|fn get_tripwire_status|fn get_bai_arms" \
  .elai_cc/crates/dashboard-v2/src-tauri/src/commands.rs
ls .elai_cc/crates/dashboard-v2/src/lib/views/ | grep -E "MutationArchive|Lineage|TripwireStatus|BaiArms|HeldOut|ArchiveVerify"
cd .elai_cc/crates/dashboard-v2 && npx svelte-check 2>&1 | tail -3
cargo check -p elai_dashboard_v2 2>&1 | tail -3
```

## ACCEPTANCE

- [ ] All 6 Tauri commands compile + type-check
- [ ] 6 Svelte components compile (svelte-check zero errors)
- [ ] ts-rs bindings drift check passes (existing CI gate)
- [ ] cargo check -p elai_dashboard_v2: zero new errors
- [ ] Cloud smoke test A: live cloud /task with template promotion →
      MutationArchivePanel shows new entry; LineageGraphView shows new node
      + edge to parent
- [ ] Cloud smoke test B: synthetic tripwire-fire injection (test harness)
      → TripwireStatusStrip flips red within poll interval (3s)
- [ ] Cloud smoke test C: ArchiveVerifyGauge green on clean repo; red after
      manual tamper
- [ ] Cloud smoke test D: HeldOutPanel shows ≥25 active tasks (per D-6 seed)
- [ ] Cloud smoke test E: BaiArmsTable shows active ArmPosterior rows
      sortable by credible_lower
- [ ] Decision doc workflow/reasoning/D-26-decisions.md

## Files touched

- MODIFY: .elai_cc/crates/dashboard-v2/src-tauri/src/commands.rs (6 new
    commands)
- ADD: .elai_cc/crates/dashboard-v2/src/lib/views/MutationArchivePanel.svelte
- ADD: .elai_cc/crates/dashboard-v2/src/lib/views/LineageGraphView.svelte
- ADD: .elai_cc/crates/dashboard-v2/src/lib/views/TripwireStatusStrip.svelte
- ADD: .elai_cc/crates/dashboard-v2/src/lib/views/BaiArmsTable.svelte
- ADD: .elai_cc/crates/dashboard-v2/src/lib/views/HeldOutPanel.svelte
- ADD: .elai_cc/crates/dashboard-v2/src/lib/views/ArchiveVerifyGauge.svelte
- MODIFY: .elai_cc/crates/dashboard-v2/src/lib/views/ReasoningDrawer.svelte
    (include new panels)
- MODIFY: ts-rs bindings (auto-generated)
- ADD: workflow/reasoning/D-26-decisions.md

## Out-of-scope

- D-27 mode-routing + refusal SLA dashboard
- Cross-plan dashboard (B+C+D plan progress overlay) — partially covered
  by bn_status_report.py (--plan=D flag in D-0_INDEX notes)

## Next

D-27 (mode-routing + refusal-SLA dashboard).
