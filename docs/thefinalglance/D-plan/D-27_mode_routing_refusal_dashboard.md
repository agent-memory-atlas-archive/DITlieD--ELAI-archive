# D-27: Mode-routing + refusal-SLA dashboard panels

Objective: surface Wave-2 modes (mode routing distribution, novelty class
breakdown, refusal-rate gauge, sycophancy-flag stream) in dashboard-v2.
ModeRoutingPanel.svelte + RefusalRateGauge.svelte + NoveltyBreakdown.svelte
+ SycophancyFlagStrip.svelte + Tauri commands.
Estimated effort: 2 sessions.
Pre-requisites: D-10..D-17 shipped.
Parallel-safe with: D-26 (orthogonal panels).

## Context

D-26 covered Wave-1 substrate; D-27 covers Wave-2 modes. Together they
complete D-plan dashboard surface.

Partial foundation present and usable:
- D-26 dashboard pattern (Tauri commands + Svelte panels).
- C-7 design tokens.
- D-10..D-17 SSB rows.

## Reads

- F:\.ELAI_workflow\docs\thefinalglance\C-plan\C-3_tauri_commands_knowledge_surface.md
- F:\.ELAI_workflow\docs\thefinalglance\C-plan\C-7_dashboard_ux_completeness.md
- All Wave-2 D-N files (artifacts being rendered)

## Tasks

1. **Tauri commands** in
   .elai_cc/crates/dashboard-v2/src-tauri/src/commands.rs:
   - `get_mode_breakdown(session_id: Option<String>, window_minutes: u32)
     -> ModeBreakdown`: counts ModeSelected rows per mode over window.
   - `get_refusal_rate(session_id: Option<String>) -> RefusalRateSnapshot`:
     reuses D-17 snapshot directly.
   - `get_novelty_breakdown(window_minutes: u32) -> NoveltyBreakdown`: counts
     NoveltyLabeled rows per NoveltyClass.
   - `get_sycophancy_flags(limit: u32) -> Vec<SycophancyFlagSummary>`:
     recent SycophancyFlag rows + claim text.
   - `get_discovery_queue() -> Vec<DiscoveryQueueEntrySummary>`: pending
     human-review claims from D-15 queue dir.
   - `ratify_discovery(claim_id: ClaimId, reviewer: String) -> Result<(),
     String>`: write DiscoveryRatified event; update queue file.
   - `reject_discovery(claim_id: ClaimId, reviewer: String, reason: String)
     -> Result<(), String>`: write DiscoveryRejected event.

2. **ModeRoutingPanel.svelte** at
   .elai_cc/crates/dashboard-v2/src/lib/views/ModeRoutingPanel.svelte:
   - Pie chart or stacked bar showing per-session mode distribution.
   - Live-update on SSB ModeSelected event.
   - Hover per-slice → percentage + raw count.

3. **RefusalRateGauge.svelte** at
   .elai_cc/crates/dashboard-v2/src/lib/views/RefusalRateGauge.svelte:
   - Linear or radial gauge from 0% to 100%.
   - Healthy-band [5%, 50%] highlighted green; out-of-band red.
   - Current rate as needle / fill level.
   - Click → opens session detail with recent RefusalDecision rows.

4. **NoveltyBreakdown.svelte** at
   .elai_cc/crates/dashboard-v2/src/lib/views/NoveltyBreakdown.svelte:
   - Bar chart: Search / DistantSearch / DiscoveryCandidate counts.
   - Click DiscoveryCandidate bar → opens DiscoveryQueueDrawer.

5. **DiscoveryQueueDrawer.svelte** at
   .elai_cc/crates/dashboard-v2/src/lib/views/DiscoveryQueueDrawer.svelte:
   - List of pending DiscoveryQueueEntry; per-entry: claim, evidence,
     classified_at.
   - Buttons: Ratify (with reviewer name input) / Reject (with reason).
   - Calls ratify_discovery / reject_discovery Tauri commands.

6. **SycophancyFlagStrip.svelte** at
   .elai_cc/crates/dashboard-v2/src/lib/views/SycophancyFlagStrip.svelte:
   - Real-time stream of SycophancyFlag rows in ChatWindow side strip.
   - Per-flag: claim_id, jaccard score, threshold, link to quarantined
     claim review.

7. **CheckerStatusPanel.svelte** at
   .elai_cc/crates/dashboard-v2/src/lib/views/CheckerStatusPanel.svelte:
   - Lists D-11 CheckerRegistry entries; per-checker: last 10 dispatches,
     pass/fail ratio, last_failure timestamp + evidence snippet.
   - Surfaces D-11 health to operator.

8. **Drawer integration**: extend MonitorWindow.svelte to include
   ModeRoutingPanel + RefusalRateGauge; ChatWindow includes
   SycophancyFlagStrip; new "Reasoning" drawer tab for NoveltyBreakdown +
   DiscoveryQueueDrawer + CheckerStatusPanel.

9. **Real-time SSB subscription** for mode + refusal + sycophancy events.

10. **Decision doc** workflow/reasoning/D-27-decisions.md (3 decisions):
    - (a) Why human-review buttons in dashboard (not CLI only): operator UX
      friction matters; per-claim CLI invocation is tedious.
    - (b) Why pie chart for modes (vs bar / line): 5-mode distribution is
      a categorical breakdown; pie is canonical.
    - (c) Why gauge for refusal rate: single-value + healthy-band; gauge is
      universal pattern.

## WIRING_CONTRACT

produces_symbols:
  - Tauri commands: get_mode_breakdown, get_refusal_rate,
    get_novelty_breakdown, get_sycophancy_flags, get_discovery_queue,
    ratify_discovery, reject_discovery
  - 6 Svelte components (ModeRoutingPanel, RefusalRateGauge,
    NoveltyBreakdown, DiscoveryQueueDrawer, SycophancyFlagStrip,
    CheckerStatusPanel)
consumes_symbols:
  - ModeSelected (D-10), RefusalDecision (D-16), NoveltyLabeled (D-15),
    SycophancyFlag (D-14), CheckerDispatched/Pass/Fail (D-11)
  - RefusalRateMonitor (D-17)
  - DiscoveryQueueEntry (D-15)
  - SsbViewer + SsbWriter
reachable_from:
  - dashboard-v2 MonitorWindow + ChatWindow + ReasoningDrawer tabs
stub_ban:
  - ratify_discovery + reject_discovery MUST require reviewer field (no
    anonymous human ratification)
  - SycophancyFlagStrip MUST link to quarantined claim
  - Healthy-band thresholds load from D-17 elai_cc.toml; not hardcoded in
    Svelte
observable_effect:
  trigger: cloud /task mix of claims across modes
  expect_ssb: existing rows from D-10..D-17 ingest via SsbViewer
  expect_artifact: ModeRoutingPanel renders distribution; RefusalRateGauge
                   shows rate; DiscoveryQueueDrawer lists pending entries

## DETERMINISTIC_CHECK

```bash
rg "fn get_mode_breakdown|fn get_refusal_rate|fn get_novelty_breakdown" \
  .elai_cc/crates/dashboard-v2/src-tauri/src/commands.rs
ls .elai_cc/crates/dashboard-v2/src/lib/views/ | grep -E "ModeRouting|RefusalRate|NoveltyBreakdown|DiscoveryQueue|SycophancyFlag|CheckerStatus"
cd .elai_cc/crates/dashboard-v2 && npx svelte-check 2>&1 | tail -3
cargo check -p elai_dashboard_v2 2>&1 | tail -3
```

## ACCEPTANCE

- [ ] All 7 Tauri commands compile + type-check
- [ ] 6 Svelte components compile (svelte-check zero errors)
- [ ] ts-rs bindings drift check passes
- [ ] cargo check -p elai_dashboard_v2: zero new errors
- [ ] Cloud smoke test A: cloud /task with mixed claims → ModeRoutingPanel
      shows realistic 5-mode distribution
- [ ] Cloud smoke test B: synthetic high-refusal session →
      RefusalRateGauge flips red (out of healthy band)
- [ ] Cloud smoke test C: claim referencing not-in-DKG primitive →
      NoveltyBreakdown shows DiscoveryCandidate bar increment + queue entry
      in DiscoveryQueueDrawer
- [ ] Cloud smoke test D: synthetic sycophancy fixture → SycophancyFlagStrip
      shows flag within poll interval
- [ ] Cloud smoke test E: dashboard ratify_discovery on queue entry →
      DiscoveryRatified SSB row written + entry status updated
- [ ] Decision doc workflow/reasoning/D-27-decisions.md

## Files touched

- MODIFY: .elai_cc/crates/dashboard-v2/src-tauri/src/commands.rs (7 new
    commands)
- ADD: .elai_cc/crates/dashboard-v2/src/lib/views/ModeRoutingPanel.svelte
- ADD: .elai_cc/crates/dashboard-v2/src/lib/views/RefusalRateGauge.svelte
- ADD: .elai_cc/crates/dashboard-v2/src/lib/views/NoveltyBreakdown.svelte
- ADD: .elai_cc/crates/dashboard-v2/src/lib/views/DiscoveryQueueDrawer.svelte
- ADD: .elai_cc/crates/dashboard-v2/src/lib/views/SycophancyFlagStrip.svelte
- ADD: .elai_cc/crates/dashboard-v2/src/lib/views/CheckerStatusPanel.svelte
- MODIFY: .elai_cc/crates/dashboard-v2/src/lib/views/MonitorWindow.svelte
- MODIFY: .elai_cc/crates/dashboard-v2/src/lib/views/ChatWindow.svelte
- MODIFY: .elai_cc/crates/dashboard-v2/src/lib/views/ReasoningDrawer.svelte
- ADD: workflow/reasoning/D-27-decisions.md

## Out-of-scope

- TUI for discovery review (DEFERRED; dashboard sufficient)
- Multi-reviewer ratification workflow (DEFERRED)
- Cross-plan progress overlay (DEFERRED to potential D-28+ session)

## Next

Wave 4 complete. Wave 5 (D-28..D-30) are research-grade deferred.
