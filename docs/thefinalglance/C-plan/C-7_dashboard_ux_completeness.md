# C-7: Dashboard UX Completeness — ExploreWindow, Window Recovery, Review Controls, Responsive CSS, Visual Regression
Objective: close the dashboard-design.report.md and B-26 MISSING/PARTIAL gaps for window coverage,
keyboard shortcuts, per-hunk review controls, responsive breakpoints, visual regression tests, and
live ReviewWindow data wiring.
Estimated effort: 1-2 sessions.
Pre-requisites: C-6 (design tokens aligned for visual baselines), C-2 (SSB DispatchEditHunks/FileWrite rows wired).
Parallel-safe with: C-5 (cloud mode), C-1/C-2/C-3/C-4 (data layer).

## Context

dashboard-design.report.md (2026-05-17, HIGH confidence) records these MISSING items:
- No ExploreWindow component; WindowHost.svelte lists 9 types, 'explore' absent.
- No closedTabs history stack; Ctrl+Shift+H routes to command palette (no-op stub at shortcuts.ts:114-118).
- Ctrl+Shift+A / Ctrl+Shift+R not implemented; ReviewWindow has acceptAll/declineAll but no shortcuts.
- No responsive CSS breakpoints (<1000px hamburger, <800px single-panel collapse).
- No per-hunk Accept/Decline/Retry controls in ReviewWindow template (only file-level).
- ReviewWindow shows static stub data with explicit UnwiredNote banner (ReviewWindow.svelte:139-145).

B-26.report.md confirms: Ctrl+Shift+A/R absent, recover-closed entry (shortcuts.ts:114-118) misleading,
T6 responsive breakpoints entirely absent.

phase-4K.report.md confirms: ReviewWindow live wiring gap (no get_file_changes / accept_change /
decline_change Tauri commands; UnwiredNote at ReviewWindow.svelte:139-145).

No existing DispatchEditHunks SSB variant found (grep returned empty); C-2 is the prerequisite that
adds it. FileWrite (SsbActionType::FileWrite, ssb.rs:442) is the live data source for applied edits.
VALID_WINDOW_TYPES in workspace.svelte.ts:9 does not include 'explore'; types.ts WindowType union
at lines 45-52 does not include 'explore'.

## Reads

- F:\.ELAI_workflow\docs\specs\guide\dashboard-design.txt (EXPLORE section, responsive layout §7, shortcuts §6)
- F:\.ELAI_workflow\docs\specs\guide\steps\phase-4K.txt (4K.5 diff viewer live wiring spec)
- F:\.ELAI_workflow\docs\thefinalglance\verification\reports\B-plan\B-26_phase_4K_6_to_13_dashboard_completion.report.md
- F:\.ELAI_workflow\docs\thefinalglance\verification\reports\guide\dashboard-design.report.md
- F:\.ELAI_workflow\docs\thefinalglance\verification\reports\steps\phase-4K.report.md

## Tasks

1. Add WindowType 'explore' to types.ts union; add 'explore' to VALID_WINDOW_TYPES Set and
   SIDEBAR_WINDOWS array in workspace.svelte.ts; register ExploreWindow.svelte in WindowHost.svelte;
   add Ctrl+8 sidebar shortcut to shortcuts.ts SHORTCUTS array.

2. Create ExploreWindow.svelte at .elai_cc/crates/dashboard-v2/src/lib/windows/ExploreWindow.svelte.
   Render a Q&A oracle surface: text input, submit button, scrollable response area. On submit,
   call stub.logStub('explore.query', { query }) and append a placeholder oracle response row.
   UnwiredNote: live codebase-search Tauri command deferred to C-3 get_codebase_search phase.
   No pass bodies, no TODO comments — render the UI fully; only the backend call is stubbed.

3. Implement window recovery history in workspace.svelte.ts:
   - Add closedTabs stack (max 20 entries) of type { type: WindowType; title: string; closedAt: number }.
   - In closeTab(), push the closing tab's type + title + Date.now() before removal.
   - Export openWindowRecovery() that sets a new boolean state flag showRecoveryModal.
   - Export the closedTabs array and showRecoveryModal flag from the workspace store.

4. Create RecoveryModal.svelte at .elai_cc/crates/dashboard-v2/src/lib/components/RecoveryModal.svelte.
   Show the closedTabs list (most-recent first); each row has window type label, title, relative
   timestamp, and a "Restore" button that calls workspace.openTab('primary', entry.type).
   Mount it inside AppShell.svelte alongside CommandPalette.svelte.

5. Rewrite the 'recover-closed' shortcut entry in shortcuts.ts (lines 114-118):
   - Replace action: () => workspace.openCommandPalette() with action: () => workspace.openWindowRecovery().
   - The shortcut id, combo Ctrl+Shift+H, and description remain; only the action changes.
   - The stale no-op (pointing to command palette) is eliminated.

6. Add Ctrl+Shift+A (accept-all hunks) and Ctrl+Shift+R (reject-all hunks) to shortcuts.ts.
   Both entries guarded by: only fires when the primary panel's active tab type is 'review'.
   Actions call reviewStore.acceptAllHunks() and reviewStore.rejectAllHunks() respectively.
   Create .elai_cc/crates/dashboard-v2/src/lib/state/review.svelte.ts as a minimal Svelte 5 store
   exposing acceptAllHunks() / rejectAllHunks() that ReviewWindow can subscribe to.

7. Add per-hunk Accept/Decline/Retry button row to ReviewWindow.svelte diff template.
   Each hunk-separator line (kind === 'hunk') gets a button row below it: Accept, Decline, Retry.
   Button states toggle hunkState[hunkContent] using the existing acceptHunk()/declineHunk() functions.
   Retry shows an inline note input (reuse toggleRetryNote pattern). No new logic; wire to existing
   local state functions already present at ReviewWindow.svelte:130-135.

8. Wire ReviewWindow to live SSB data. Prerequisite: C-2 adds DispatchEditHunks SSB rows;
   C-3 adds get_file_changes / accept_change / decline_change Tauri commands.
   In this task: remove UnwiredNote banner; replace FILES static const with a $derived that
   reads from a live reviewStore.files reactive list; call invoke('get_file_changes', { session_id })
   on mount; invoke('accept_change' / 'decline_change' / 'retry_audit') on hunk/file actions.
   If get_file_changes returns empty array, show an <Empty> state (not the static stub data).
   Guard: if window is EXECUTING and no file_changes yet, show loading spinner row.

9. Add responsive CSS breakpoints to .elai_cc/crates/dashboard-v2/src/styles/base.css:
   @media (max-width: 999px): sidebar collapses to hamburger icon strip; nav-list hidden;
   hamburger-btn shown (hamburger toggle state managed via Sidebar.svelte ResizeObserver).
   @media (max-width: 799px): Workspace.svelte auto-sets secondaryCollapsed = true via
   ResizeObserver; primary panel takes full width.
   Add hamburger-btn element to Sidebar.svelte (shown only when viewport < 1000px).
   Wire ResizeObserver in Workspace.svelte to set workspace.secondaryCollapsed on narrow viewports.

10. Create Playwright visual regression test at .elai_cc/crates/dashboard-v2/tests/visual.spec.ts.
    Tests: (a) dashboard loads and ProjectSelector renders; (b) ExploreWindow renders at Ctrl+8;
    (c) ReviewWindow renders with hunk controls visible; (d) RecoveryModal renders after window close
    + Ctrl+Shift+H; (e) sidebar collapses at 800px viewport width. Each test calls
    page.screenshot({ path: 'tests/snapshots/<name>.png' }) for baseline. Use Playwright's
    expect(page).toHaveScreenshot() for regression on second run. Add playwright as devDependency
    to package.json if not already present — verify via package.json before adding.

11. Add Ctrl+Shift+W shortcut to shortcuts.ts only if Tier-0 shell window type exists in VALID_WINDOW_TYPES.
    Grep workspace.svelte.ts for 'shell' or 'tier_0' before writing. If absent, add a comment in
    shortcuts.ts: "// Ctrl+Shift+W (Tier-0 shell): deferred — shell WindowType not yet registered."
    No code addition if the window type is unregistered.

## WIRING_CONTRACT

produces_symbols:
  - WindowType 'explore' variant (types.ts)
  - ExploreWindow.svelte component
  - SHORTCUTS entry: sidebar-8 (Ctrl+8 -> 'explore')
  - SHORTCUTS entry: review-accept-all (Ctrl+Shift+A -> reviewStore.acceptAllHunks)
  - SHORTCUTS entry: review-reject-all (Ctrl+Shift+R -> reviewStore.rejectAllHunks)
  - SHORTCUTS entry: recover-closed (Ctrl+Shift+H -> workspace.openWindowRecovery) — replaces stub
  - workspace.closedTabs stack + workspace.openWindowRecovery()
  - RecoveryModal.svelte component
  - review.svelte.ts store (acceptAllHunks, rejectAllHunks)
  - per-hunk button row in ReviewWindow.svelte template
  - ReviewWindow.svelte live data wiring (removes UnwiredNote)
  - responsive @media breakpoints in base.css (<1000px hamburger, <800px single-panel)
  - hamburger-btn in Sidebar.svelte + ResizeObserver in Workspace.svelte
  - visual.spec.ts Playwright test suite (5 snapshot tests)
consumes_symbols:
  - workspace.openTab, workspace.closeTab, workspace.openCommandPalette (workspace.svelte.ts — existing)
  - WorkspaceWindowType 'review' guard in shortcut action
  - invoke('get_file_changes'), invoke('accept_change'), invoke('decline_change'), invoke('retry_audit') — from C-3
  - SSB FileWrite + DispatchEditHunks rows — from C-2
  - ExploreWindow registered in WindowHost.svelte (AppShell or WindowHost mount point)
reachable_from:
  - Keyboard: Ctrl+8 (explore), Ctrl+Shift+H (recovery), Ctrl+Shift+A (accept-all), Ctrl+Shift+R (reject-all)
  - ReviewWindow per-hunk Accept/Decline/Retry buttons
  - Viewport resize (hamburger sidebar, single-panel collapse)
  - Playwright test runner (npm run test:visual)
stub_ban:
  - ExploreWindow MUST render a full UI; only the backend search call is stubbed via logStub
  - RecoveryModal MUST show real closedTabs entries; MUST NOT hard-code fake entries
  - ReviewWindow UnwiredNote MUST be removed; live data path MUST replace static FILES const
  - Ctrl+Shift+A/R MUST be guarded to 'review' window only — no global accept-all
frontend_pair:
  - ExploreWindow.svelte + WindowHost.svelte + types.ts
  - RecoveryModal.svelte + AppShell.svelte + workspace.svelte.ts
  - review.svelte.ts + ReviewWindow.svelte + shortcuts.ts
observable_effect:
  trigger: elai /task --mode cloud "modify hello.rs to add a goodbye function" -> navigate to ReviewWindow
  expect_ssb: FileWrite rows appear after task completes; get_file_changes returns populated array
  expect_artifact: each hunk in ReviewWindow has Accept/Decline/Retry buttons visible; Ctrl+Shift+A
                   accepts all hunks; file applied state reflected on next get_file_changes poll;
                   ExploreWindow opens at Ctrl+8 and renders (not blank); closing a window then
                   pressing Ctrl+Shift+H shows RecoveryModal with that window listed; resize to
                   <800px collapses secondary panel; visual.spec.ts baseline screenshots written

## DETERMINISTIC_CHECK

```bash
# 1. WindowType 'explore' in types.ts
grep "'explore'" .elai_cc/crates/dashboard-v2/src/lib/api/types.ts

# 2. ExploreWindow component exists
test -f .elai_cc/crates/dashboard-v2/src/lib/windows/ExploreWindow.svelte

# 3. Ctrl+8 shortcut registered
grep "Ctrl+8" .elai_cc/crates/dashboard-v2/src/lib/state/shortcuts.ts

# 4. Ctrl+Shift+A and Ctrl+Shift+R registered
grep -E "Ctrl\+Shift\+A|Ctrl\+Shift\+R" .elai_cc/crates/dashboard-v2/src/lib/state/shortcuts.ts

# 5. recover-closed action no longer calls openCommandPalette
grep -A3 "recover-closed" .elai_cc/crates/dashboard-v2/src/lib/state/shortcuts.ts | grep -v openCommandPalette

# 6. closedTabs in workspace.svelte.ts
grep "closedTabs" .elai_cc/crates/dashboard-v2/src/lib/state/workspace.svelte.ts

# 7. RecoveryModal exists and mounted in AppShell
test -f .elai_cc/crates/dashboard-v2/src/lib/components/RecoveryModal.svelte
grep "RecoveryModal" .elai_cc/crates/dashboard-v2/src/lib/screens/AppShell.svelte

# 8. UnwiredNote removed from ReviewWindow
grep -c "UnwiredNote" .elai_cc/crates/dashboard-v2/src/lib/windows/ReviewWindow.svelte
# Expected: 0

# 9. Per-hunk buttons in ReviewWindow template
grep -E "acceptHunk|declineHunk" .elai_cc/crates/dashboard-v2/src/lib/windows/ReviewWindow.svelte

# 10. Responsive breakpoints in base.css
grep -E "max-width: 999px|max-width: 799px" .elai_cc/crates/dashboard-v2/src/styles/base.css

# 11. Playwright visual spec exists
test -f .elai_cc/crates/dashboard-v2/tests/visual.spec.ts

# 12. Smoke test: cloud task -> ReviewWindow live hunks
elai /task --mode cloud "modify hello.rs to add a goodbye function"
# Manual: navigate to ReviewWindow; confirm hunk Accept/Decline/Retry buttons present;
# press Ctrl+Shift+A; confirm all hunks show accepted state
```

## ACCEPTANCE

- [ ] WindowType 'explore' added to types.ts union and VALID_WINDOW_TYPES Set
- [ ] ExploreWindow.svelte renders Q&A oracle UI: input, submit, response area (no pass bodies)
- [ ] Ctrl+8 opens ExploreWindow in primary panel; sidebar-8 in SHORTCUTS array
- [ ] workspace.closedTabs stack populated on closeTab(); max 20 entries; FIFO eviction
- [ ] Ctrl+Shift+H opens RecoveryModal listing closed windows (not command palette)
- [ ] RecoveryModal Restore button re-opens window via workspace.openTab
- [ ] Old recover-closed -> openCommandPalette no-op action removed
- [ ] Ctrl+Shift+A fires reviewStore.acceptAllHunks() only when primary active tab is 'review'
- [ ] Ctrl+Shift+R fires reviewStore.rejectAllHunks() only when primary active tab is 'review'
- [ ] Per-hunk Accept/Decline/Retry button row renders below each hunk separator in ReviewWindow
- [ ] ReviewWindow UnwiredNote banner removed; live get_file_changes wired on mount
- [ ] ReviewWindow shows <Empty> state when get_file_changes returns [] (no static stub fallback)
- [ ] accept_change / decline_change / retry_audit invoked on hunk/file actions
- [ ] @media (max-width: 999px) hamburger sidebar in base.css; hamburger-btn in Sidebar.svelte
- [ ] @media (max-width: 799px) auto-collapse secondary panel via ResizeObserver in Workspace.svelte
- [ ] visual.spec.ts has 5 Playwright snapshot tests covering: selector, explore, review, recovery, responsive
- [ ] Ctrl+Shift+W entry: present if 'shell' WindowType exists; commented-out with explanation if absent
- [ ] Cloud smoke test: task produces hunks in ReviewWindow; Ctrl+Shift+A accepts all; refresh shows applied
- [ ] svelte-check passes on all modified .svelte files
- [ ] cargo check --workspace passes (no Rust changes expected; check for accidental drift)

## Files touched

- MODIFY: .elai_cc/crates/dashboard-v2/src/lib/api/types.ts (add 'explore' to WindowType)
- MODIFY: .elai_cc/crates/dashboard-v2/src/lib/state/workspace.svelte.ts (closedTabs, openWindowRecovery, VALID_WINDOW_TYPES, SIDEBAR_WINDOWS)
- MODIFY: .elai_cc/crates/dashboard-v2/src/lib/state/shortcuts.ts (Ctrl+8, Ctrl+Shift+A/R, fix Ctrl+Shift+H, optional Ctrl+Shift+W)
- ADD: .elai_cc/crates/dashboard-v2/src/lib/state/review.svelte.ts (acceptAllHunks, rejectAllHunks store)
- ADD: .elai_cc/crates/dashboard-v2/src/lib/windows/ExploreWindow.svelte
- MODIFY: .elai_cc/crates/dashboard-v2/src/lib/windows/ReviewWindow.svelte (per-hunk buttons, live wiring, remove UnwiredNote)
- ADD: .elai_cc/crates/dashboard-v2/src/lib/components/RecoveryModal.svelte
- MODIFY: .elai_cc/crates/dashboard-v2/src/lib/screens/AppShell.svelte (mount RecoveryModal, register ExploreWindow in WindowHost if needed)
- MODIFY: .elai_cc/crates/dashboard-v2/src/lib/components/Sidebar.svelte (hamburger-btn, ResizeObserver or @media visibility)
- MODIFY: .elai_cc/crates/dashboard-v2/src/lib/screens/Workspace.svelte (ResizeObserver auto-collapse at <800px)
- MODIFY: .elai_cc/crates/dashboard-v2/src/styles/base.css (responsive @media breakpoints)
- ADD: .elai_cc/crates/dashboard-v2/tests/visual.spec.ts (Playwright 5-test visual regression suite)
- CREATE: workflow/reasoning/C-7-decisions.md

## Next

C-8 (if defined) or return to plan-index for remaining gaps.
