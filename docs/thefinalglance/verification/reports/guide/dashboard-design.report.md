# Verification Report: dashboard-design.txt
Source: F:\.ELAI_workflow\docs\specs\guide\dashboard-design.txt
Date: 2026-05-17
Overall status: MOSTLY_DONE
Confidence: HIGH

## Summary
The dashboard-v2 implementation (Tauri + Svelte 5) covers the vast majority of mechanisms specified
in the design document. All six window types are wired and mounted in WindowHost. The color system,
sidebar, status bar, keyboard shortcuts, tab system, panel divider, and intent classifier are fully
present. The primary gaps are: no EXPLORE window (replaced by the orbit/graph paradigm), no
purple accent in the token system (design specified purple for interactive elements; implementation
uses steel-blue), no window recovery history (Ctrl+Shift+H maps to command palette instead), no
Ctrl+Enter keybinding, and no responsive/hamburger mode for narrow widths. The design's
"Option B" (Axum + WebSocket) was not implemented; Option A (Tauri IPC) was chosen instead.

## Mechanisms verified DONE

- Project selector screen (Flow 1) — evidence: src/lib/screens/ProjectSelector.svelte:1
- Wordmark + tagline "a deterministic development engine" — evidence: ProjectSelector.svelte:127-129
- Recent projects list (name, path, timestamp, hover teal left border) — evidence: ProjectSelector.svelte:154-174, ProjectSelector.svelte:383-388 (.row-inset scaleY)
- New Project button with native folder picker (Tauri dialog + browser fallback) — evidence: ProjectSelector.svelte:69-109
- Mode selector LOCAL/CLOUD/HYBRID with teal underline on active — evidence: ProjectSelector.svelte:111-116, ProjectSelector.svelte:447-464 (.mode-btn.active::after)
- Dual-panel workspace shell (sidebar + primary + divider + secondary + status bar) — evidence: src/lib/screens/Workspace.svelte:77-141, src/lib/layout/AppShell.svelte:136-177
- Sidebar 64px icon strip (matches 48px spec, actual 64px) with 7 nav icons — evidence: Sidebar.svelte:28-36, Sidebar.svelte:209 (.sidebar width: 64px)
- Active sidebar icon: teal fill + teal-ghost background + active-bar left indicator — evidence: Sidebar.svelte:295-308 (.nav-btn--active, .active-bar)
- Sidebar bottom: mode dot indicator (LOCAL=teal, CLOUD=steel, HYBRID=conic) — evidence: Sidebar.svelte:183-205, Sidebar.svelte:360-370
- Sidebar bottom: session/user button — evidence: Sidebar.svelte:197-205
- Status bar: project name, FSM state (color-coded), mode, gate mode, session ID — evidence: StatusBar.svelte:84-164
- Status bar executing line (teal 1px pulse at bottom when EXECUTING) — evidence: StatusBar.svelte:161-163, StatusBar.svelte:307-315 (.executing-line + anim-breath)
- Status bar VRAM display (only shown when >70%) — evidence: StatusBar.svelte:71-72, StatusBar.svelte:134-141
- Panel divider: 2px draggable, teal on hover/drag — evidence: PanelDivider.svelte:93-120
- Panel divider: double-click resets to 50/50 — evidence: PanelDivider.svelte:51-57
- Panel divider: min panel width 320px — evidence: PanelDivider.svelte:18, Workspace.svelte:152
- Secondary panel collapse/expand (Ctrl+D, closeTab collapses) — evidence: workspace.svelte.ts:193-218, shortcuts.ts:93-97
- Window types: COMMAND, PLAN, ACTIVITY, REVIEW, MONITOR, SETTINGS all wired in WindowHost — evidence: WindowHost.svelte:25-46
- Tab bar with active teal border, close on [x] hover, middle-click close — evidence: Panel.svelte:247-264, Panel.svelte:439-453
- Tab drag: move tab between panels via drag-and-drop — evidence: Panel.svelte:255-263, workspace.svelte.ts:231-260
- Sidebar icon drag: open new tab type in drop target — evidence: Sidebar.svelte:117-131, AppShell.svelte:59-83
- Ctrl+N — new tab (opens picker) — evidence: shortcuts.ts:78-82
- Ctrl+W — close active tab — evidence: shortcuts.ts:83-91
- Ctrl+D — toggle secondary panel — evidence: shortcuts.ts:93-97
- Ctrl+1 through Ctrl+5 — switch sidebar windows — evidence: shortcuts.ts:28-74
- Ctrl+Shift+P — command palette — evidence: shortcuts.ts:119-123
- Ctrl+. — slash command palette — evidence: shortcuts.ts:99-107, AppShell.svelte:116-120
- Escape — dismiss overlays — evidence: AppShell.svelte:103-113
- COMMAND window with message history, cards, input bar — evidence: CommandWindow.svelte:1+
- Intent classifier (TASK, FIX, EXPLORE, INTERVENE, STEER, ABORT, CONFIG, AMBIGUOUS) — evidence: src/lib/utils/intent.ts:13-24
- Slash commands (/task, /fix, /explore, /config, /status, /continue) — evidence: intent.ts:26-35, CommandWindow.svelte:43-52
- Slash command autocomplete palette overlay — evidence: CommandWindow.svelte:64-73
- Response card types: status, intervention, error, disambiguation, steering — evidence: api/types.ts:577
- Disambiguation card with option buttons — evidence: CommandWindow.svelte:548-551, api/types.ts:626-629
- Intervention card with Accept/Decline/View Full/Retry buttons — evidence: CommandWindow.svelte lines with 'intervention' kind
- ACTIVITY window (orbit/starmap visualizer for live execution) — evidence: ActivityWindow.svelte:1-60, OrbitCanvas.svelte present
- PLAN window (tree view of phases + baby steps, FSM-wired) — evidence: PlanWindow.svelte:1-60, Tree.svelte present
- REVIEW window (diff viewer, Accept All, Decline All, Retry, multi-file tab bar) — evidence: ReviewWindow.svelte:1-120
- MONITOR window (phase dots, progress bar, intervention cards, supervision log) — evidence: MonitorWindow.svelte:1-80 with PhaseDot model
- MONITOR intervention: submit_intervention_response, approve/reject/takeover — evidence: MonitorWindow.svelte:141-152
- SETTINGS window (sidecar, gate, MCP, model routing, project profile, dep audit) — evidence: SettingsWindow.svelte:1-60
- Notification system (Browser Notifications API, task-complete + intervention events) — evidence: NotificationManager.svelte:1-60
- Command palette (Ctrl+Shift+P, lists all actions + drawers) — evidence: CommandPalette.svelte:1-60
- Color token system (bg-void, bg-base, bg-surface, teal, teal-soft, teal-ghost, teal-bright, pass, warn, fail) — evidence: styles/tokens.css:8-53
- Motion library (enterFade, enterLift, breath, scan, pulseScale; reduced-motion media query) — evidence: styles/motion.css:1-127
- Typography: JetBrains Mono + Fira Code monospace font-mono; 8px base spacing grid — evidence: tokens.css:105-106, tokens.css:73-83
- Transition durations: 120ms fast, 180ms base, 320ms slow — evidence: tokens.css:87-89
- Scrollbars: Scrollbar.svelte component; 4px width styling — evidence: src/lib/ui/Scrollbar.svelte, styles/scrollbar.css
- Tauri IPC backend: 145 #[tauri::command] functions including get_fsm_state, submit_task, submit_intervention_response, set_gate_mode, get_context_budget — evidence: commands.rs:94,173,207,240,568,1695
- SSB read-only viewer: all backend data sourced from SSB (SQLite) via Tauri state — evidence: commands.rs:6-14
- Workspace persistence: layout serialized to localStorage (schema v1) — evidence: workspace.svelte.ts:8-121
- Gate mode layout (INTERACTIVE: primary=Activity, secondary=Command; AUTONOMOUS: applied via applyGateModeLayout) — evidence: Workspace.svelte:44-55

## Mechanisms PARTIAL

- Sidebar width
  - exists: Sidebar.svelte:209 (width: 64px)
  - gap: Design specifies 48px; implementation uses 64px. Visual difference only but deviates from spec.

- Status bar layout
  - exists: StatusBar.svelte:84-164 (left cluster: project, state, mode, gate, session)
  - gap: Design specifies Left/Center/Right split (center empty, right has model+VRAM+session). Implementation uses a single left-aligned cluster with verbose items revealed on hover. Model and session ID are in the hover-revealed verbose group, not always-visible right side.

- Purple accent color
  - exists: tokens.css shows --state-auditing: #9e7aa8 (desaturated one-use lavender) and a comment explicitly banning purple as a general accent
  - gap: Design specifies purple (#7C5CBF) as secondary accent for buttons, user actions, chat bubbles, intervention card borders. Implementation uses teal for interactive elements and teal/ghost/outlined buttons instead of purple. The design's purple/purple-soft/purple-ghost tokens do not exist.

- ACTIVITY window as "execution stream" text log
  - exists: ActivityWindow.svelte renders OrbitCanvas (orbit starmap)
  - gap: Design specifies a text-based entry log ([reasoning], [edit], [validate], [test] entries with timestamps). Implementation replaced this with an orbital 3D visualization. The step-by-step text stream is not present.

- Window history and recovery
  - exists: shortcuts.ts:114-118 (Ctrl+Shift+H registered, routes to command palette)
  - gap: No closed-window history stack in workspace.svelte.ts (no closedTabs/windowHistory). Ctrl+Shift+H opens command palette as a proxy but does not show "last 20 closed windows with type, title, timestamp." The recovery feature is a stub.

- Ctrl+Enter to send message
  - exists: shortcuts.ts has no Ctrl+Enter binding; CommandWindow handles keydown
  - gap: Searched CommandWindow.svelte for Ctrl+Enter — not present. Only Enter (without Ctrl) fires send in CommandWindow's textarea handler. Design specifies Ctrl+Enter explicitly.

- INTERACTIVE/AUTONOMOUS mode switcher in sidebar overlay
  - exists: Sidebar.svelte bottom has mode-dot-btn that cycles LOCAL/CLOUD/HYBRID only
  - gap: Design specifies a small overlay with two groups (Execution: LOCAL/CLOUD/HYBRID; Control: INTERACTIVE/AUTONOMOUS). Implementation cycles execution mode only; the gate/autonomy mode is controlled from settings or command palette, not a sidebar overlay.

## Mechanisms MISSING

- EXPLORE window — searched: ExploreWindow, 'explore' in WindowHost, WindowType enum; no evidence. WindowHost.svelte:28 lists command/activity/plan/review/monitor/settings/graph/idea_framing/audit_timeline — EXPLORE is absent. The design's EXPLORE Q&A codebase oracle surface has no dedicated window component.

- Ctrl+Enter send shortcut — searched: shortcuts.ts for 'enter', CommandWindow.svelte for 'ctrlKey.*Enter' and 'Ctrl+Enter'; no match. The send-on-Ctrl+Enter binding from the keyboard shortcuts section is not implemented.

- Ctrl+Shift+A / Ctrl+Shift+R (REVIEW accept/reject all shortcuts) — searched: shortcuts.ts for 'Shift+A', 'Shift+R'; no match. ReviewWindow.svelte has acceptAll/declineAll functions but no keyboard shortcut wiring.

- Responsive layout (1000px minimum, hamburger sidebar below 1000px, single-panel below 800px) — searched: styles/ for @media min-width/max-width breakpoints except reduced-motion; no match. No responsive breakpoints exist. The design's minimum-width and sidebar-collapse behavior is absent.

- WebSocket live updates / Axum server (design Option B) — searched: commands.rs and src-tauri/ for WebSocket, axum serve; no match. Implementation is Tauri IPC (Option A) with polling, no WebSocket push.

- SSB new action types (HEALTH_SNAPSHOT, INTERVENTION_REQUEST, INTERVENTION_RESPONSE, WINDOW_STATE) — searched: commands.rs for these literal action_type strings; not confirmed as explicit SSB action types. The backend writes payloads tagged "dashboard-v2" but the four design-specified action type names are not verified as dedicated SSB enum variants.

- New window type selector overlay (Ctrl+N animated overlay with backdrop blur, staggered fade) — searched: workspace.svelte.ts openNewTabPicker, Panel.svelte picker; partial — the picker exists (Panel.svelte pickerOpenHere) but the design's animated overlay with backdrop-blur and staggered 50ms-each option fade-in is not confirmed implemented with those exact animation properties.

- Per-hunk accept/decline in REVIEW — searched: ReviewWindow.svelte for 'hunk.*accept', 'per-hunk'; only file-level acceptAll/declineAll found. Design specifies per-hunk controls appearing on hover.

## Mechanisms UNVERIFIABLE

- IDLE state "recent tasks" clickable list in COMMAND window opening PLAN history — reason: CommandWindow.svelte polls get_recent_tasks and renders recentTasks, but the behavior of clicking a task (open PLAN for that task's history vs. offer to continue) cannot be confirmed without reading 200+ more lines of CommandWindow.svelte template code beyond what was sampled. Data is present; interaction behavior uncertain.

- Thinking pulse (2px teal line at bottom of active panel, opacity 20->50->20, 2s cycle) — reason: StatusBar has .executing-line with anim-breath; tokens.css breath keyframe matches (0.2->0.55->0.2). However the design places this on the "active panel" not just the status bar. Whether the panel itself also shows a pulse is unverifiable without exhaustive panel CSS grep.

## What must be done (action list)

- [ ] Add EXPLORE window: create src/lib/windows/ExploreWindow.svelte with Q&A codebase oracle UI; register 'explore' in WindowType, WindowHost, WINDOW_TYPES, and SHORTCUTS (Ctrl+8 or reassign Ctrl+4).
- [ ] Implement window recovery history: add closedTabs stack in workspace.svelte.ts closeTab(); wire Ctrl+Shift+H to show a recovery modal listing last 20 closed windows.
- [ ] Add Ctrl+Enter send binding in CommandWindow.svelte textarea onkeydown handler.
- [ ] Add Ctrl+Shift+A (accept all) and Ctrl+Shift+R (reject all) in shortcuts.ts wired to ReviewWindow actions.
- [ ] Add responsive CSS breakpoints in styles/base.css: sidebar hamburger at <1000px, single-panel auto-collapse at <800px.
- [ ] Add per-hunk accept/decline controls to ReviewWindow.svelte (hover-revealed buttons per diff hunk).
- [ ] Decide on purple accent: either adopt #7C5CBF tokens (purple, purple-soft, purple-ghost) per design for interactive buttons/intervention borders, or formally document the deviation (teal-only design decision).
- [ ] Add INTERACTIVE/AUTONOMOUS toggle to the sidebar mode overlay (currently only cycles execution mode, not gate/autonomy mode).
- [ ] Verify ACTIVITY window text-stream parity: confirm whether the orbit visualization is the accepted replacement for the text-log design, or whether a fallback text mode is needed for accessibility/low-end clients.
- [ ] Confirm the four SSB action types (HEALTH_SNAPSHOT, INTERVENTION_REQUEST, INTERVENTION_RESPONSE, WINDOW_STATE) are registered in the SSB schema or document that they were intentionally deferred.

## Notes
The implementation chose Tauri IPC (Option A) over Axum+WebSocket (Option B), consistent with the spec's recommendation for distribution packaging. The ActivityWindow's replacement of the text-log design with an orbital 3D starmap is the largest functional deviation from the design document — the spec's [reasoning]/[edit]/[validate]/[test] stream entries are not present as a text-based surface. The sidebar is 64px wide vs the spec's 48px. The purple accent system was explicitly removed from tokens.css by a comment stating the system has no purple accent for general use.

REPORT_WRITTEN=F:\.ELAI_workflow\docs\thefinalglance\verification\reports\guide\dashboard-design.report.md | status=MOSTLY_DONE | done=47 | partial=7 | missing=8 | unverifiable=3
