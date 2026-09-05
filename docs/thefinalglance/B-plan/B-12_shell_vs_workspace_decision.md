# B-12: Tier-0 Shell vs Spec Workspace Decision
Objective: consolidate the two competing dashboard shells. The spec MVP
Workspace is buried as the "PLAN window" inside Tier-0 Shell. Make the
spec MVP Workspace the primary landing; Tier-0 Shell becomes optional
power-user mode.
Estimated effort: ~30 min (most of the decision is already DONE in V2).
Pre-requisites: B-3 (PlanWindow registered), B-7 (decomposition tree
populated), B-11 (design tokens fixed).
Parallel-safe with: B-13, B-14, B-15..B-20.

## V2 REBASELINE (2026-05-07) — decision already executed in V2

dashboard-v2 (`.elai_cc/crates/dashboard-v2/`) ships Workspace as the
canonical primary shell:

  - Entry: `src/main.ts` mounts `App.svelte` via Svelte 5 `mount(App,
    { target })` (per commit f6f96b0).
  - `App.svelte` -> `lib/screens/ProjectSelector.svelte` (first run) ->
    `lib/screens/Workspace.svelte` (primary).
  - Workspace hosts `lib/layout/{AppShell, Sidebar, Panel, PanelDivider,
    StatusBar, WindowHost, Drawer, CommandPalette, Rail}` plus the orbit
    canvas.
  - Tier-0 Shell concept from V1 is RETIRED. There is no Tier-0/Workspace
    competition in V2.

Remaining work for B-12 (small):

  1. Verify `lib/layout/StatusBar.svelte` lives inside Workspace (not
     outside it). If it's a top-level App.svelte sibling, move it into
     Workspace per spec.
  2. First-run wizard at `lib/screens/ProjectSelector.svelte` already
     covers project selection. Confirm it does NOT also ask
     "tier0/workspace" — if it does, drop that branch.
  3. Sidebar audit: spec calls for 5 icons (terminal/layers/play/search/
     settings) at 48px. V2's `lib/layout/Sidebar.svelte` exists; verify
     dimensions + icon count.
  4. Drop the `elai_cc.toml [dashboard] primary_shell` config knob from
     the original B-12 task (no longer relevant — only one shell now).

Path map (V1 -> V2):
  V1: crates/dashboard/src-frontend/src/Shell.svelte         -> retired
  V1: crates/dashboard/src-frontend/src/Workspace.svelte     ->
       crates/dashboard-v2/src/lib/screens/Workspace.svelte
  V1: src/shell/Sidebar.svelte                               ->
       crates/dashboard-v2/src/lib/layout/Sidebar.svelte
  V1: src/shell/WindowRegistry.ts                            ->
       (retired; routing via AppShell + Panel + WindowHost)
  V1: src/StatusBar.svelte                                   ->
       crates/dashboard-v2/src/lib/layout/StatusBar.svelte
  V1: src/main.ts                                            ->
       crates/dashboard-v2/src/main.ts (Svelte 5 mount(App, ...))

## Context

Per audit-implementation/phase-4K.md and audit-dashboard/_AGGREGATE.md
"DASHBOARD VISUAL DRIFT": "Two competing shells coexist. Tier-0 Shell
(sidebar + dual-PanelHost) is what the user lands on; the spec MVP
Workspace exists but reachable only as the 'PLAN window' inside Shell."
This is the second-largest "doesn't feel like wanted" gap.

Spec MVP per dashboard-design.txt: single Workspace as primary surface,
status bar inside Workspace, sidebar with 5 mode icons (terminal/layers/
play/search/settings), primary + secondary panel split. The Tier-0 Shell
adds 9 windows + 10 drawers + autonomous-mode UI + ProposalsPanel — all
of which are deferred to post-MVP per spec but were built early.

## Reads

- audit-implementation/phase-4K.md (full DASHBOARD QUALITY ASSESSMENT)
- F:\.ELAI_workflow\docs\specs\guide\dashboard-design.txt FLOW 2 (MAIN
  WORKSPACE) and FLOW 3 (WINDOWS)
- crates/dashboard/src-frontend/src/Shell.svelte (current Tier-0 shell)
- crates/dashboard/src-frontend/src/Workspace.svelte (the spec MVP)
- crates/dashboard/src-frontend/src/shell/WindowRegistry.ts:27

## Tasks

1. **Decision: Workspace primary, Shell optional.** Refactor:
   - Workspace becomes default landing on dashboard launch
   - Tier-0 Shell becomes accessible via Ctrl+Shift+W (or similar)
   - User preference persisted in elai_cc.toml [dashboard] primary_shell =
     "workspace" | "tier0"
   - First-run wizard asks: "Spec MVP layout (recommended) or full
     power-user shell?"

2. **Move status bar inside Workspace.** Spec required status bar inside
   the Workspace primary surface, not in the outer Shell chrome.

3. **Sidebar follows spec.** 48px wide, icon-only, 5 icons:
   [terminal] Command, [layers] Plan, [play] Execution, [search] Explore,
   [settings] Settings. Bottom: [moon/sun] mode indicator (LOCAL=teal dot,
   CLOUD=purple dot), [user] session info.

4. **Window types per spec.** Per dashboard-design.txt: 6 spec window
   types are COMMAND, PLAN, ACTIVITY, REVIEW, EXPLORE, MONITOR.
   Tier-0 Shell's 9 windows (Chat, Activity, Explore, Monitor, Plan,
   Preview, Research, Review, Settings) include extras (Preview, Research,
   Settings). Decision: keep extras as POST-MVP (B-26) but hide unless
   feature flag set.

5. **Default panel arrangement.** INTERACTIVE mode default:
   - Primary: ACTIVITY
   - Secondary: COMMAND
   AUTONOMOUS mode default (when B-22 lands):
   - Primary: MONITOR
   - Secondary: ACTIVITY

6. **Drawer reduction.** 10 drawers → spec mentions Settings as the only
   sidebar-mode drawer. Consolidate: AntiStaleness, Knowledge, Orchestration,
   ProjectIntelligence, ProviderVram, Reasoning, Research, Rules, ToolsHooks,
   WebQuality become tabs WITHIN Settings (or hide behind power-user mode).
   Keep individual drawers as power-user surfaces accessible from Tier-0 Shell.

7. **Slash command palette stays.** Ctrl+. trigger remains — useful and
   consistent with spec.

8. **Window history + tabs** — defer to B-26 (post-MVP per spec).

## WIRING_CONTRACT

produces_symbols:
  - crates/dashboard-v2/src/lib/screens/Workspace.svelte
consumes_symbols:
  - first-run wizard
  - Ctrl+Shift+W toggle
reachable_from:
  - crates/dashboard-v2/src/main.ts::mount
stub_ban:
  - Tier-0 Shell not the default landing
frontend_pair: B-12 itself
observable_effect:
  trigger: cd .elai_cc && cargo test -p elai_cc_contracts --test fsm_phase_transition_smoke_ssb -- --nocapture
  expect_ssb: phase_transition
  expect_artifact: spec MVP Workspace renders; Tier-0 Shell accessible via
                   keybinding

## DETERMINISTIC_CHECK

```bash
# 1. Default mount is Workspace
rg "App\\.svelte|main\\.ts" .elai_cc/crates/dashboard/src-frontend/src
# Read entry point; confirm default child is Workspace, not Shell.

# 2. Config schema
rg "primary_shell" .elai_cc/crates/dashboard/src
# Expected: ≥1 hit.
```

## ACCEPTANCE

Per V2 REBASELINE CLOSURE (workflow/reasoning/B-12-decisions.md, 2026-07-02).
Verified against active V2 source on 2026-07-02.

- [x] Workspace is default landing — SHIPPED. App.svelte:30-36 routes
      `session.screen` to <Workspace/> via the {:else} branch; default
      landing is Workspace.
- [ ] Tier-0 Shell available via keybinding — DEFERRED to B-26. V2
      REBASELINE retires Tier-0 Shell; Ctrl+Shift+W 'shell' WindowType
      toggle not implemented (three-site sync risk avoided). See
      B-12-decisions.md DEFERRED section.
- [x] Status bar inside Workspace (not Shell chrome) — SHIPPED.
      WorkflowShell.svelte:127-129 renders <ShellStatusBar/> at row 3 of
      the 3-row grid; StatusBar lives inside the Workspace shell, not at
      App level.
- [ ] Sidebar matches spec (5 icons, 48px wide) — DEFERRED to B-26.
      Active IconRail.svelte ships 3 buttons (menu/files/settings) at
      48px; the active V2 IA is the 4-question-tab model, which
      supersedes the spec's 5-mode sidebar model. Icon remap pending
      controller resolution of the IA fork. See B-12-decisions.md
      DEFERRED section.
- [ ] First-run wizard asks layout preference — DEFERRED to B-26. V2
      REBASELINE locks a single primary shell, making a layout-preference
      wizard meaningless. ProjectSelector.svelte asks project only. See
      B-12-decisions.md DEFERRED section.
- [ ] Drawers consolidated under Settings (or hidden behind power-user
      mode) — DEFERRED to B-26. V2 ships ~60 drawer panels
      (lib/drawers/*), far exceeding the plan's assumed 10. Scope is
      materially larger than B-12 assumed. See B-12-decisions.md
      DEFERRED section.

## Files touched

- MODIFY: crates/dashboard/src-frontend/src/main.ts or App.svelte
- MODIFY: crates/dashboard/src-frontend/src/Workspace.svelte
- MODIFY: crates/dashboard/src-frontend/src/Shell.svelte
- MODIFY: crates/dashboard/src-frontend/src/shell/Sidebar.svelte
- MODIFY: crates/dashboard/src-frontend/src/StatusBar.svelte
- ADD: crates/dashboard/src-frontend/src/wizard/FirstRunWizard.svelte
- MODIFY: elai_cc.toml schema [dashboard] section
- CREATE: workflow/reasoning/B-12-decisions.md

## Next

B-13 (Activity feed restoration).
