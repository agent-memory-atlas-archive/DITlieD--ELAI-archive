B-12 Tier-0 Shell vs Spec Workspace Decision — DECISIONS
Date: 2026-06-05 (backfill)
Source: workflow/sessions/s-plan-audit-2026-06-04/plan-status/B/B-12_shell_vs_workspace_decision.md

CORE DECISION
- Workspace is the single primary shell. App.svelte (27-29) routes ProjectSelector when
  session.screen==='selector' else Workspace; the V1 Shell.svelte is retired (absent from the V2
  filesystem). main.ts -> mount(App) -> App.svelte -> Workspace is the production entry chain.

SHIPPED
- StatusBar mounted INSIDE Workspace (Workspace.svelte:8 import, :180 render) rather than as App-level
  chrome.
- Sidebar 48px confirmed (Sidebar.svelte width:48px).

KEY DECISIONS
- elai_cc.toml [dashboard] primary_shell config knob: deliberately DROPPED. With only one shell the
  knob is meaningless (plan V2 REBASELINE line 38 says so). Zero hits across .elai_cc — correct.

DEFERRED / NOT DONE (deferred but not formally closed in the plan file)
- FirstRunWizard.svelte (tier0/workspace layout preference): NOT built. ProjectSelector handles
  project selection only, no layout-preference prompt. Defer to B-26 or close acceptance.
- Ctrl+Shift+W Tier-0 shell toggle: DEFERRED (shortcuts.ts:40 "'shell' WindowType not yet"). The
  'shell' WindowType is unimplemented.
- Drawer consolidation under Settings / power-user mode: 10 drawers still present; no consolidation.
- Sidebar icon count: spec says 5 (terminal/layers/play/search/settings); shipped has 7 (added
  monitor + explore). Resolve by updating spec acceptance or removing the two extras.

NOTES
- No ELAI_*_ENABLE flag (no allowlist row). wiring_contract_check FAIL on "Workspace as default
  mount" is a false negative (Svelte routing pattern, not a Rust/TS symbol). The recommendation is
  to formally defer the three open acceptance items (FirstRunWizard, Ctrl+Shift+W, drawer
  consolidation) to B-26 rather than build them under B-12.

## 2026-07-02 V2 REBASELINE CLOSURE

Formal closure of B-12 acceptance items against the locked V2 REBASELINE (plan lines 11-51).
Verified against active V2 source on 2026-07-02.

SHIPPED
- Workspace default mount: App.svelte:30-36 routes `session.screen` to <Workspace/> via the
  {:else} branch (default landing). session.screen discriminant at lib/state/session.svelte.ts:121.
- ShellStatusBar inside WorkflowShell: WorkflowShell.svelte:127-129 renders <ShellStatusBar/> at
  row 3 of the 3-row grid. StatusBar is NOT App-level chrome; it lives inside the Workspace shell.
- primary_shell config knob DROPPED: `rg "primary_shell" .elai_cc/elai_cc.toml` returns 0 hits.
  V2 REBASELINE (plan line 38) declares the knob meaningless with a single shell; none was added.
- ProjectSelector project-only: lib/screens/ProjectSelector.svelte asks project only; no
  tier0/workspace layout-preference branch. session.openWorkspace(project) at
  lib/state/session.svelte.ts:173 is the transition fn called from ProjectSelector.openProject().

DEFERRED to B-26
- FirstRunWizard.svelte: NOT built. `rg "FirstRunWizard" .elai_cc/crates/dashboard-v2/src` returns
  0 hits. Conflicts with V2 REBASELINE (single primary shell makes a layout-preference wizard
  meaningless). Defer to B-26.
- Ctrl+Shift+W 'shell' WindowType toggle: NOT implemented. shortcuts.ts retains the placeholder
  comment; types.ts WindowType union, workspace.svelte.ts VALID_WINDOW_TYPES, and WindowHost.svelte
  switch are all untouched (three-site sync risk avoided by not adding 'shell'). Defer to B-26.
- Drawer consolidation: V2 ships ~60 drawer panels (lib/drawers/*), far exceeding the plan's
  assumed 10. Scope is materially larger than B-12 assumed; defer the full consolidation to B-26.
- Sidebar 5-icon remap: ACTIVE IconRail.svelte ships 3 buttons (menu/files/settings); LEGACY
  Sidebar.svelte ships 7; neither matches spec's 5-mode model (terminal/layers/play/search/
  settings). The active V2 IA is the 4-question-tab model, which supersedes the spec's 5-mode
  sidebar model. Icon remap deferred to B-26 pending controller resolution of the IA fork.

INVARIANTS VERIFIED (2026-07-02)
- `rg "primary_shell" .elai_cc/elai_cc.toml` -> 0 hits (knob must NOT exist). PASS.
- `rg "FirstRunWizard" .elai_cc/crates/dashboard-v2/src` -> 0 hits (wizard must NOT be created).
  PASS.
- `rg "session.screen" .elai_cc/crates/dashboard-v2/src/App.svelte` -> 2 hits (lines 30, 32);
  routing intact, Workspace is the {:else} default. PASS.
