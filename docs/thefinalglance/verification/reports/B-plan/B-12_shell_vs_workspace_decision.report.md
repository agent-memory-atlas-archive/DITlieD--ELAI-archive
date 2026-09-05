# Verification Report: B-12_shell_vs_workspace_decision
Source: F:\.ELAI_workflow\docs\thefinalglance\B-plan\B-12_shell_vs_workspace_decision.md
Date: 2026-05-17
Overall status: MOSTLY_DONE
Confidence: HIGH

## Summary
The V2 rebaseline is substantively implemented: dashboard-v2 ships Workspace as the canonical primary shell, App.svelte correctly routes selector->ProjectSelector else->Workspace, StatusBar lives inside Workspace (passed as a snippet to AppShell), and the default panel layout (INTERACTIVE=Activity+Command, AUTONOMOUS=Monitor+Activity) is wired in workspace.svelte.ts. Two gaps remain: the sidebar has 7 icons at 64px instead of the spec-required 5 icons at 48px (missing explore/search, extra graph; dimensions wrong), and no Ctrl+Shift+W toggle for a Tier-0 shell exists (though Tier-0 is retired per V2, so this may be intentionally absent).

## Claims verified DONE

- Entry point uses Svelte 5 mount(App, { target }) — evidence: .elai_cc/crates/dashboard-v2/src/main.ts:10
- App.svelte routes session.screen==='selector' -> ProjectSelector, else -> Workspace — evidence: .elai_cc/crates/dashboard-v2/src/App.svelte:25-29
- Workspace.svelte is primary landing (default else-branch) — evidence: .elai_cc/crates/dashboard-v2/src/App.svelte:27-29
- Workspace hosts AppShell, Sidebar, Panel, PanelDivider, StatusBar, Drawer, CommandPalette, Rail — evidence: .elai_cc/crates/dashboard-v2/src/lib/screens/Workspace.svelte:3-11
- StatusBar is inside Workspace (passed as {#snippet statusBar()} to AppShell, not in App.svelte root) — evidence: .elai_cc/crates/dashboard-v2/src/lib/screens/Workspace.svelte:123-125
- AppShell renders statusBar in .statusbar-row (grid-row:2, inside the shell grid) — evidence: .elai_cc/crates/dashboard-v2/src/lib/layout/AppShell.svelte:168-170
- Tier-0 Shell concept is retired: crates/dashboard/ has only gen/schemas/ subdirectory; no src-frontend/src/Shell.svelte exists — evidence: Glob result showing only .elai_cc/crates/dashboard/gen/schemas/*.json files
- No Tier-0/Workspace competition in V2: no references to "tier0", "Tier-0", or "primary_shell" config key in .elai_cc — evidence: Grep on primary_shell returned no files
- ProjectSelector does NOT ask tier0/workspace layout preference — evidence: .elai_cc/crates/dashboard-v2/src/lib/screens/ProjectSelector.svelte (full read, no tier mention)
- FirstRunWizard.svelte not created; plan's remaining work item 2 is satisfied by absence — evidence: Glob of lib/screens/ shows only ProjectSelector.svelte, Workspace.svelte, RemoteSidecarBootOverlay.svelte
- elai_cc.toml primary_shell config knob dropped (plan task 4 item): no such key exists anywhere in codebase — evidence: Grep on primary_shell returned no hits
- Default panel layout wired: INTERACTIVE -> primary=ACTIVITY, secondary=COMMAND; AUTONOMOUS -> primary=MONITOR, secondary=ACTIVITY — evidence: .elai_cc/crates/dashboard-v2/src/lib/state/workspace.svelte.ts:344-357
- applyGateModeLayout called at Workspace mount and on reconcileGateMode — evidence: .elai_cc/crates/dashboard-v2/src/lib/screens/Workspace.svelte:54; .elai_cc/crates/dashboard-v2/src/lib/state/session.svelte.ts:222,244
- Ctrl+. slash command palette trigger wired — evidence: .elai_cc/crates/dashboard-v2/src/lib/state/shortcuts.ts:99-106
- Path map V1->V2 migrations accurate: StatusBar at lib/layout/StatusBar.svelte, Workspace at lib/screens/Workspace.svelte, Sidebar at lib/layout/Sidebar.svelte — all confirmed by Glob

## Claims PARTIAL

- Sidebar matches spec (5 icons, 48px wide)
  - exists: .elai_cc/crates/dashboard-v2/src/lib/layout/Sidebar.svelte — sidebar is 64px wide (width:64px in CSS at line 213); navItems has 7 entries: command/plan/activity/review/monitor/settings/graph
  - gap: spec calls for 5 icons (terminal/layers/play/search/settings) at 48px. Actual: 7 icons at 64px. The "explore/search" icon is absent; "review", "monitor", and "graph" are extra relative to spec's 5-icon list. Width is 64px vs spec's 48px.

## Claims MISSING

- Ctrl+Shift+W keybinding to toggle Tier-0 Shell — searched: shortcuts.ts (full read), AppShell.svelte (full read); no Ctrl+Shift+W binding exists. Note: since Tier-0 is retired per V2 rebaseline this may be intentionally absent, but the WIRING_CONTRACT lists it as a consumes_symbol.

## Claims UNVERIFIABLE

- V1 crates/dashboard/src-frontend/src/Shell.svelte "retired" (not deleted, just no src-frontend) — reason: the V1 src-frontend directory is entirely absent (only gen/schemas/ remains); cannot confirm whether it was deleted or never committed to this branch. Effectively retired from the build but source history not verifiable here.
- WindowRegistry.ts retired in favor of AppShell+Panel+WindowHost routing — reason: no WindowRegistry.ts found under dashboard-v2, consistent with claim, but V1 file history unverifiable.

## What must be done (action list)

- [ ] Fix Sidebar.svelte width: change .sidebar { width } from 64px to 48px if spec requires it (.elai_cc/crates/dashboard-v2/src/lib/layout/Sidebar.svelte:213)
- [ ] Reconcile sidebar icon count with spec: spec demands 5 icons (terminal/layers/play/search/settings); current has 7. Determine whether "explore" replaces one of review/monitor/graph, or whether graph/monitor/review are now also spec-accepted (update B-12 spec or Sidebar.svelte accordingly)
- [ ] Decide on Ctrl+Shift+W: since Tier-0 is retired, either remove this consumes_symbol from the WIRING_CONTRACT or document explicitly that it is N/A in V2

## Notes
The sidebar icon set diverged from spec after B-37 (graph window added). The 64px vs 48px discrepancy was not flagged in the V2 rebaseline section; AppShell.svelte's grid-template-columns also hard-codes 64px for the sidebar column (line 186). Any fix requires updating both files in sync.
