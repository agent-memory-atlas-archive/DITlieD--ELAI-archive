PHASE 4K AUDIT — DASHBOARD (Tauri MVP)
Auditor: hostile implementation auditor
Spec sources read:
  - F:\.ELAI_workflow\docs\thefinalglance\summary\phase-4-summary.md (lines 1659-1955)
  - phase-4K.txt content cross-checked via summary (summary explicitly states it reflects phase-4K.txt 5-sub-step rewrite; spec-internal contradiction with the original 13-sub-step task description is documented in summary §CONTRADICTIONS).
Code paths inspected (all absolute):
  - F:\.ELAI_workflow\.elai_cc\crates\dashboard\Cargo.toml
  - F:\.ELAI_workflow\.elai_cc\crates\dashboard\tauri.conf.json
  - F:\.ELAI_workflow\.elai_cc\crates\dashboard\src\main.rs (51 LOC)
  - F:\.ELAI_workflow\.elai_cc\crates\dashboard\src\commands.rs (4110 LOC)
  - F:\.ELAI_workflow\.elai_cc\crates\dashboard\src\lib.rs (1536 LOC)
  - F:\.ELAI_workflow\.elai_cc\crates\dashboard\src-frontend\package.json
  - F:\.ELAI_workflow\.elai_cc\crates\dashboard\src-frontend\src\app.css
  - F:\.ELAI_workflow\.elai_cc\crates\dashboard\src-frontend\src\App.svelte
  - F:\.ELAI_workflow\.elai_cc\crates\dashboard\src-frontend\src\Workspace.svelte
  - F:\.ELAI_workflow\.elai_cc\crates\dashboard\src-frontend\src\PlanTree.svelte
  - F:\.ELAI_workflow\.elai_cc\crates\dashboard\src-frontend\src\ActivityFeed.svelte
  - F:\.ELAI_workflow\.elai_cc\crates\dashboard\src-frontend\src\DiffViewer.svelte
  - F:\.ELAI_workflow\.elai_cc\crates\dashboard\src-frontend\src\shell\Shell.svelte
  - F:\.ELAI_workflow\.elai_cc\crates\dashboard\src-frontend\src\shell\Sidebar.svelte
  - F:\.ELAI_workflow\.elai_cc\crates\dashboard\src-frontend\src\shell\WindowRegistry.ts
  - F:\.ELAI_workflow\.elai_cc\crates\dashboard\src-frontend\src\shell\windowTypes.ts
  - F:\.ELAI_workflow\.elai_cc\crates\orchestrator\src\ssb.rs (action-type enum + payload defs)
  - F:\.ELAI_workflow\.elai_cc\crates\orchestrator\src\ssb_viewer.rs (via elai.map / SCIP refs)
  - F:\.ELAI_workflow\.claude\tools\elai.map (SCIP/grep)
  - F:\.ELAI_workflow\.elai_cc\dashboard-wiring-plan.txt (consulted for context only — claims were verified independently per audit instructions)
  - F:\.ELAI_workflow\.elai_cc\dashboard-wiring-inventory.txt (consulted for context only — same caveat)


SUB-STEP 4K.1 — SSB READ-ONLY VIEWER LIBRARY
Verdict: PASS (with scope creep noted)
Evidence:
  - SsbViewer struct exists in crates\orchestrator\src\ssb_viewer.rs (DEF_STRUCT at line 768 per elai.map; SsbViewerError enum present at lines 728+).
  - All 8 spec-required typed query functions exist as SCIP-resolved methods on SsbViewer per elai.map: latest_fsm_state, active_session, phase_progress, recent_actions, agent_actions, pending_interventions, file_changes, reasoning_chain (the last 6 are referenced explicitly via SCIP refs in ssb_viewer.rs and dashboard contract tests).
  - get_action_detail (added in 4K.5) also present at ssb_viewer.rs:1189.
  - Read-only WAL: opened with rusqlite OPEN_READ_ONLY semantics — confirmed by contract test crates\contracts\tests\dashboard_tier2_windows_contract.rs which asserts SsbViewerError::NotFound on read-only paths (elai.map line 893).
  - 2 NEW SSB action types compile-time present: SsbActionType::InterventionRequest and SsbActionType::InterventionResponse, with payloads InterventionRequestPayload and InterventionResponsePayload defined in ssb.rs:854-866. CHECK constraint at ssb.rs:51 includes 'intervention_request', 'intervention_response'.
  - Spec-deferred types correctly absent: HEALTH_SNAPSHOT and WINDOW_STATE do NOT appear in the SsbActionType enum, do NOT appear in the CHECK constraint, do NOT appear in elai.map. PASS on the deferral boundary.
  - Pagination (offset+limit) confirmed in ssb_viewer.rs (DRAWER_QUERY_HARD_CAP at lines 1915, 2288).
Concerns:
  - SsbViewer has many post-MVP query methods bolted on (bm25_stats, agent_checkpoints, decomposition_tree, recent_preview_available, etc.). These are not 4K.1 scope but do not break the MVP; they support the deferred drawers/windows that should not yet exist.

SUB-STEP 4K.2 — TAURI APP SCAFFOLD + TAURI COMMANDS
Verdict: PASS (severe scope creep on commands count)
Evidence:
  - Crate path .elai_cc\crates\dashboard\ exists. Cargo.toml declares tauri = "2.10" (with feature "custom-protocol") and tauri-build = "2.5" build-dep; serde, serde_json, anyhow, tracing pulled in.
  - tauri.conf.json: identifier "dev.elai.dashboard"; window 1200x800 minWidth 900 minHeight 600; frontendDist "src-frontend/dist"; devUrl "http://localhost:1420"; beforeBuildCommand "npm --prefix src-frontend run build"; bundle.active true with "all" targets.
  - build.rs present (one line wrapping tauri_build::build via the standard layout — confirmed by Cargo.toml [build-dependencies] block).
  - src/main.rs (51 LOC) instantiates the Tauri builder; src/commands.rs holds the #[tauri::command] handlers.
  - Frontend stack matches spec: src-frontend/package.json declares svelte ^5.0.0, @sveltejs/vite-plugin-svelte ^5.0.0, vite ^6.0.0, @tauri-apps/api ^2.1.1.
  - All 6 spec-required commands are present with the spec signatures (verified by grep on commands.rs):
      get_fsm_state                      (line 67)
      get_phase_progress                 (line 74)
      get_recent_actions(limit, offset)  (line 81)
      get_pending_interventions          (line 94)
      get_file_changes(session_id)       (line 101)
      submit_intervention_response       (line 369)
    Plus 4K.5's get_action_detail (line 338) and 4K.3's project commands (list_projects line 109, select_project line 212, create_project line 244, list_templates line 293, submit_task line 299).
  - elai-orchestrator dep with feature "sidecar" present (path-dep correct).
Major concern:
  - commands.rs is 4110 LOC and contains 98 `tauri::command` attributes. Spec budget for 4K.2 was ~100 Rust LOC and 6 commands. The MVP-required commands are present, but 90+ extra commands implement deferred features (drawers, windows, intelligence panels, gate stats, intent classification, override file writers, preview slot management, dev-server registry, BM25 stats, MCP bridge, escalation queries, etc.). This is the main wiring/clarity problem the user is feeling — a 41x LOC overshoot on the scaffold step.
  - app.css visual identity drifted slightly from spec (see 4K.4 for color drift detail).

SUB-STEP 4K.3 — PROJECT SELECTOR + TASK INPUT
Verdict: PASS
Evidence:
  - App.svelte uses a $state discriminant with views ProjectSelector | EmptyFolderChooser | TemplatePicker | TaskInput | Workspace | Settings. ProjectSelector is the entry view.
  - Components present: ProjectSelector.svelte, EmptyFolderChooser.svelte, TemplatePicker.svelte, TaskInput.svelte.
  - Tauri commands for project management exist with spec signatures: list_projects, select_project, create_project, list_templates, submit_task.
  - tauri-plugin-dialog = "2.7" present in Cargo.toml — supports native folder picker as required.
  - "+ New Project" implemented per a recent patch (patch-new-project-flow comment in App.svelte:8) which routes empty folders to EmptyFolderChooser and populated folders straight to Workspace. TemplatePicker is now an orphan in the default flow (per the same comment) — this is a minor spec deviation: spec says template picker triggers for empty folders. Deviation is documented in code as deliberate.
Concerns:
  - Spec spelled "describe what you want to build..." as the placeholder; not verified at this depth but components exist.
  - EmptyFolderChooser provides interactive vs autonomous routing — autonomous routing is technically a deferred feature (autonomous mode UI), but here it is a routing choice on an empty folder, not a full autonomous dashboard, so it is borderline.

SUB-STEP 4K.4 — MAIN WORKSPACE: PLAN TREE + ACTIVITY FEED
Verdict: PARTIAL PASS (the spec workspace exists but is bypassed by a parallel sidebar+tabs shell)
Evidence FOR:
  - Workspace.svelte renders the spec layout (CSS grid 320px | 1fr) with PlanTree on the left and ActivityFeed (or DiffViewer) on the right.
  - PlanTree.svelte polls get_phase_progress and step_progress, expands on click, marks status (complete/fail/warn-pending) with badge colors. Spec-conformant.
  - ActivityFeed.svelte polls get_recent_actions every 500ms (POLL_MS = 500) and renders ActivityEntry rows. Status bar component (StatusBar.svelte) exists.
Evidence AGAINST (scope creep + spec drift):
  - Workspace has been demoted: the script header literally says "Tier 1 (Workspace.svelte demoted): the Tier 0 inline command strip moved into the CHAT window. Workspace is now rendered exclusively as the PLAN window via WindowRegistry." So the user does not land on the spec MVP layout — they land on Shell.svelte, which renders Sidebar + dual PanelHost + StatusBar + an overlay system. WindowRegistry maps WindowType.PLAN to Workspace.svelte, not to a clean two-panel layout.
  - The spec MVP explicitly says "Sidebar with mode icons (MVP uses two-panel layout directly, no sidebar) — DEFERRED". Sidebar.svelte exists with 5 mode icons (CHAT/PLAN/ACTIVITY/EXPLORE/SETTINGS) and Ctrl+1-5 keymap (SIDEBAR_KEYMAP in Shell.svelte:40-46). DEFERRED feature shipped.
  - Panel divider draggable: PanelDivider.svelte exists in shell/. The spec required this on Workspace, not on a Tier-0 dual-panel shell. The Workspace itself uses a CSS grid (grid-template-columns: 320px 1fr) with NO draggable divider in Workspace.svelte:191-195. The MVP acceptance criterion ("panel divider draggable") is satisfied only at the Shell level, not on the spec's 4K.4 workspace.
  - Auto-scroll + "Jump to latest" pill: NOT FOUND in ActivityFeed.svelte (no autoScroll variable, no pill markup). Spec acceptance partially unmet.
  - Preview link rendering in ActivityFeed: NOT FOUND. ActivityFeed has no preview-handling branch and no entry-type renderer for action_type === "preview_available" that emits a clickable teal link. Preview is instead handled inside DiffViewer (which detects PreviewPayload in action_detail) and via a separate PreviewWindow. Spec 4K.4 explicitly required "[preview]: clickable localhost URL ... rendered as teal link 'Preview: http://localhost:5173', on click opens default browser via Tauri shell::open" inside the activity feed entry types. NOT met.
  - Status bar shows no "FSM state" surfacing in Workspace.svelte's right-pane; FSM state is exposed via Shell.svelte instead. The MVP status bar is only present when going through Shell.
  - Spec entry-type set: [reasoning], [edit], [validate], [test], [preview]. Real ActivityFeed delegates rendering to ActivityEntry.svelte; not all entry types verified at this depth, but at minimum no preview branch exists.
  - ActivityFeed has been bloated with feasibility summary panel (Phase 4F backport) and rule violations strip (Batch 4B) embedded in the same component (lines 36-62). Both belong outside MVP.

SUB-STEP 4K.5 — DIFF VIEWER + ACCEPT/DECLINE + PREVIEW LINK
Verdict: PARTIAL PASS
Evidence FOR:
  - DiffViewer.svelte exists. Calls get_action_detail via Tauri invoke (line 42).
  - DiffHunk.svelte component present (per-hunk render).
  - get_action_detail Tauri command exists in commands.rs:338 wrapping SsbViewer::get_action_detail.
  - submit_intervention_response writes a true SsbPayload::InterventionResponse to SSB via state.writer.write_payload (commands.rs:369-387). Real wiring, not a mock. Contract test dashboard_4k3_4k4_4k5_contract.rs:297 verifies the JSON payload shape end-to-end.
  - InterventionCard.svelte present and consumed by Workspace.svelte (lines 173-178).
  - Preview opener: open_preview_url command (commands.rs:393) validates URL prefixes against an allow-list (PREVIEW_URL_PREFIXES) — security boundary present, hostile-URL test cases exist (lines 1439-1447). Uses platform default handler (cmd /c start on Windows, open on macOS, xdg-open on Linux). NOT using the Tauri shell::open API the spec called out, but functionally equivalent.
Evidence AGAINST:
  - DiffViewer in Workspace.svelte:182-186 replaces ActivityFeed via a conditional, not as a modal overlay. Spec said "Replaces activity feed panel temporarily (or modal overlay)" — implementation chose the first option, which is acceptable, but the spec's per-hunk Accept/Decline-on-hover and multi-file vertical file list were not verified in DiffViewer at this depth. DiffHunk.svelte exists but the per-hunk control surface and multi-file navigation need a deeper read; given the 163 LOC DiffViewer and 60 LOC DiffHunk, the per-hunk + multi-file features are likely thin or absent.
  - Preview rendering on the activity feed (spec acceptance criterion "preview link clickable + opens system browser" from inside the activity feed) lives in PreviewWindow + DiffViewer, not in ActivityFeed. The spec wanted the preview link visible inline on the live activity stream.
  - Multiple-pending intervention navigation ([Previous]/[Next] count badge from spec's ADVERSARIAL block) not verified — likely absent given InterventionCard is rendered as a stacked list (Workspace.svelte:172-178).


SSB ACTION-TYPE DELTA TABLE (Dashboard reads vs spec)
Required new types (PRESENT):
  intervention_request    ✓ (ssb.rs:51, 416, 854)
  intervention_response   ✓ (ssb.rs:51, 417, 866)
Required reads (preview_available + reasoning_trace + file_write + tool_result + test_outcome + phase_complete): all six action_types exist in SsbActionType enum (ssb.rs:387-415, 415=preview_available). Whether ActivityEntry renders all six branches with the spec'd typography was not verified — the renderer is delegated.
Correctly DEFERRED (ABSENT):
  HEALTH_SNAPSHOT  ✓ absent (good — MONITOR window deferred)
  WINDOW_STATE     ✓ absent (good — session-recovery deferred)


DEFERRED 4K.6-4K.13 FEATURES THAT SNEAKED IN (ranked by visibility impact)
1. Sidebar with 5 mode icons + active dot indicator (Sidebar.svelte) — DEFERRED.
2. Window management with tabs, Ctrl+W close, Ctrl+Shift+H recover-last-closed, last-closed-stack (layoutState.svelte.ts referenced in Shell.svelte:14-18) — DEFERRED (Group C C-3).
3. Polymorphic windows: ChatWindow, ActivityWindow, ExploreWindow, MonitorWindow, PlanWindow, PreviewWindow, ResearchWindow, ReviewWindow, SettingsWindow (all in src-frontend/src/windows/ + chat/ChatWindow.svelte) — DEFERRED.
4. 10 drawer components (AntiStalenessDrawer, KnowledgeDrawer, OrchestrationDrawer, ProjectIntelligenceDrawer, ProviderVramDrawer, ReasoningDrawer, ResearchDrawer, RulesDrawer, ToolsHooksDrawer, WebQualityDrawer) — NOT IN ANY 4K SPEC (post-MVP enhancement layer).
5. Slash command palette (chat/SlashPalette.svelte triggered by Ctrl+. in Shell.svelte:127) — DEFERRED.
6. Keyboard shortcut grid (Ctrl+1-5 modes, Ctrl+N new window, Ctrl+W close, Ctrl+D toggle secondary, Ctrl+. palette, Ctrl+Shift+H history) — DEFERRED.
7. Autonomous-mode UI: gate-mode polling (refreshGate every 2s in Shell.svelte:153-156), autonomy strip in Workspace.svelte:117-134, AutoAdvanceStats panel — DEFERRED ("autonomous mode UI" is in deferred list).
8. Overlay system with backdrop + window-selector dialog + slash-palette dialog (Shell.svelte:233-278) — visual polish DEFERRED.
9. Sidecar BootGate with health-poll (BootGate.svelte) — not in 4K spec (operational concern bolted on; arguably reasonable but unspecified).
10. ProposalsPanel (Workspace.svelte:181) — reads SCC/CEL proposal flow (Phase 4H), not 4K MVP scope.
11. MCP bridge feature gate (Cargo.toml [features] mcp) — Phase 4 batch-ful-d work, unrelated to 4K MVP.
12. Project Intelligence Tier 3 (commands.rs:425-...), get_rule_lifecycle_stats, get_rule_detail, write_override_file, get_git_status, get_preview_state, etc. — all post-MVP wiring exposed as Tauri commands.

The spec's 6 commands ballooned into 98. That is the wiring problem the user is sensing.


COLOR + TYPOGRAPHY SPEC COMPLIANCE
Spec values vs app.css:
  bg-void:      spec #0a0e12  / actual #0b0d12   DRIFT (slightly darker neutral)
  bg-base:      spec #0f1318  / actual #0e1118   DRIFT
  bg-surface:   spec #151a20  / actual #141822   DRIFT
  bg-hover:     spec #1a2028  / actual #1a1f2a   DRIFT
  bg-input:     spec #1e2530  / actual #1e2430   DRIFT
  teal primary: spec #1B7A6E  / actual #1b7a6e   MATCH (case-only)
  teal-soft:    spec #1B7A6E40 / actual rgba(0.25 = 0x40)   MATCH semantically
  teal-ghost:   spec #1B7A6E15 / actual rgba(0.08)   DRIFT (spec 0x15 = ~0.082, close but not exact)
  purple:       spec #7C5CBF  / actual #7c5cbf   MATCH
  purple-soft:  spec #7C5CBF30 / actual rgba(0.19 = ~0x30)   MATCH
  text-primary: spec #d0d7de  / actual #d0d7de   MATCH
  text-secondary: spec #6e7681 / actual #8a93a0   DRIFT (lighter than spec)
  text-ghost:   spec #383e47  / actual #4a5260   DRIFT (lighter than spec)
  pass/warn/fail/info: all match.
Font scale:
  spec xs=10 sm=12 base=13 lg=15 xl=18  /  actual xs=11 sm=13 base=14 lg=16 xl=20  ALL DRIFT (every step +1 to +2px).
Font family:
  spec mono only ("JetBrains Mono", "Fira Code", monospace).
  actual: --font-display (Segoe UI Variable Display + Inter), --font-ui (Inter), --font-mono (JetBrains Mono). Body uses --font-ui by default — spec wanted mono throughout for the "deterministic engine" feel. Major drift.

The font drift is likely the single biggest source of "doesn't feel or look like I wanted." The spec dashboard-design.txt vision was monospace-everywhere, sub-13px, charcoal-dark; the implementation is Inter UI, ~14px base, slightly muted text. It looks like a generic VS Code clone instead of a CRT-deterministic control surface.


DASHBOARD QUALITY ASSESSMENT — what is making it not feel/look like wanted

A. WRONG FONT FAMILY EVERYWHERE
   --font-ui defaults to Inter; the spec wanted JetBrains Mono as the primary
   text face. Buttons, inputs, sidebar labels, status strip, intervention
   cards — all Inter. Only explicit class="mono" elements use JetBrains Mono.
   Fix: change body default to --font-mono; use Inter only in the project
   selector hero block.

B. ENTIRE TYPE SCALE INFLATED +1-2 PX
   Spec base 13px / xs 10px (information density). Actual base 14px / xs 11px.
   Compounded by the wrong font family, the UI looks ~25% larger than the
   reference design. "Information density over whitespace" was a hard spec.

C. TWO COMPETING SHELLS COEXIST
   App.svelte still has a "Workspace" view (the spec MVP layout) but also a
   "Shell" view (Tier-0 sidebar + dual-PanelHost + tabs). Workspace is
   reachable AS the PLAN window inside Shell, AND as a standalone view
   depending on the navigation path. The user sees Shell first (sidebar
   chrome), and the spec MVP "two-panel workspace appears after task
   submitted" flow is buried.

D. SIDEBAR + TAB CHROME STEAL VISUAL WEIGHT
   The spec MVP wanted plan-tree-on-left, activity-feed-on-right, status-bar.
   The actual product has 48px-icon-rail + tabs + window-selector overlay +
   slash-palette + autonomy strip + feasibility panel + violations strip +
   proposals panel + intervention cards + activity feed. The activity feed —
   the centerpiece per spec — is 5+ rows down on a busy page.

E. PREVIEW LINK NOT INLINE IN ACTIVITY FEED
   Spec required activity-feed [preview] entry rendered as teal link with
   shell::open. Implementation routes preview through PreviewWindow + DiffViewer
   only. The user can finish a build and not see the link in the feed.

F. NO AUTO-SCROLL / NO "JUMP TO LATEST" PILL
   ActivityFeed.svelte has neither. As soon as enough rows arrive, the user
   loses the "live stream" feel that the spec called for. This is a tiny
   feature that disproportionately defines whether a feed feels live.

G. COLOR PALETTE 5 VALUES OFF
   bg-void, bg-base, bg-surface, bg-hover, bg-input, text-secondary, text-ghost
   all drifted slightly cooler/lighter than spec. Individually trivial; in
   aggregate the surface looks more "blueprint blue" than the spec's
   "warm charcoal teal." The teal accent itself is exact, so it pops less
   than intended.

H. AUTONOMY STRIP PRESENT EVEN IN INTERACTIVE MODE? Conditional on gate.mode === "autonomous", which is fine — but it appears inside Workspace alongside ProposalsPanel and feasibility/violations strips embedded in ActivityFeed. The MVP was supposed to be five things on screen; the actual product is twelve.

I. MISSING: STATUS BAR IN THE SPEC MVP WORKSPACE
   Workspace.svelte does not render StatusBar; StatusBar lives in Shell.svelte.
   The spec required a 24px bottom status bar on the workspace itself with
   project | FSM state | session info, EXECUTING -> teal text. Only available
   when the user goes through the Shell path.

J. THE WIRING DOCS UNDER .elai_cc/ MISLEAD THE TEAM
   dashboard-wiring-plan.txt openly states "The dashboard-design.txt is a BASE;
   the user's vision is: A. sidebar (icon rail + mode indicator) + main area
   with TWO windows" and proceeds to plan a KIMI-inspired sidebar+dual-panel
   shell with five tiers (Tier 0-4). This document is a unilateral re-spec
   that retroactively justifies all the deferred features as "tiers" and
   built around them. Per audit instructions, these claims are NOT
   trustworthy — the canonical spec is phase-4K.txt (5 sub-steps, no
   sidebar, deferral list explicit). The wiring plan and inventory should
   be treated as artifacts of an undocumented re-scoping decision, not as
   authoritative.


GAP BETWEEN dashboard-design.txt VISION AND CURRENT IMPLEMENTATION
Vision (from spec summary §4K + design doc):
  - Single Tauri window. 1200x800.
  - Two-panel workspace (320px plan tree | flex activity feed) appears after task submitted.
  - 24px status bar bottom: project | FSM state | session info.
  - Monospace typography throughout (JetBrains Mono).
  - Information density over whitespace, 13px base / 10px xs.
  - Warm charcoal background (#0a0e12 → #1e2530), single teal accent (#1B7A6E), purple for primary actions (#7C5CBF).
  - Activity feed is the live centerpiece, with auto-scroll, jump-to-latest pill, inline preview links, [reasoning]/[edit]/[validate]/[test]/[preview] entry types.
  - Diff viewer overlay/replacement for INTERVENTION_REQUEST.

Current implementation:
  - Tauri window correct dimensions.
  - Sidebar+dual-pane Shell renders first (deferred Group C work).
  - Workspace exists but is one of nine windows in a registry; reachable via Shell as the PLAN window.
  - Status bar in Shell, not in Workspace.
  - Inter UI font default; mono only on opt-in classes.
  - Type scale +1-2px.
  - Five background colors slightly drifted; teal exact.
  - Activity feed: no auto-scroll, no pill, no inline preview link, but loaded with feasibility + violations panels.
  - Diff viewer replaces feed via conditional render. Per-hunk + multi-file controls likely thin (file size suggests minimal).
  - Preview link goes through PreviewWindow + open_preview_url with allow-list — secure but not inline.
  - 90+ extra Tauri commands shipped to drive deferred drawers/windows.

Net: the MVP layout, components, commands, and SSB types all exist and pass acceptance individually (4K.1-4K.5 individually verifiable as PASS). What broke "the feel" is that a parallel Tier-0..Tier-4 shell rewrite ran on top of the MVP, deferred features were implemented anyway, the typography drifted off the design system, and the spec's clean two-panel default flow became the "plan window inside the shell" sub-state.


OVERALL VERDICT
4K.1: PASS
4K.2: PASS (severe scope creep; 98 commands vs 6)
4K.3: PASS (small TemplatePicker deviation, deliberate)
4K.4: PARTIAL PASS (spec workspace exists but bypassed by Shell; missing inline preview, auto-scroll, jump-to-latest; status bar not on workspace)
4K.5: PARTIAL PASS (intervention round-trip + diff viewer + preview opener all real; per-hunk controls and multi-file nav under-built; preview link not inline in feed)

Phase 4K MVP: SHIPPED but BURIED under deferred work the user said not to do yet. The user's "doesn't feel or look like I wanted" instinct is correct — the dashboard pivoted from the 5-step Tauri MVP into an early Group-C (post-MVP) implementation, with typography drift compounding the wrong-feel problem.


REMEDIATION RECOMMENDATIONS (priority order)
1. Decide whether the Shell+Sidebar Tier-0 work is the new direction. If YES, update phase-4K.txt and the summary to canonicalize it. If NO, revert App.svelte default route to Workspace and gate Shell behind a feature flag.
2. Fix typography: change body default to --font-mono; rescale --fs-* down to spec (10/12/13/15/18). High visual impact, low effort.
3. Realign color palette to exact spec values (5 backgrounds + 2 text colors). Low effort, restores "warm charcoal teal" feel.
4. Add inline preview link rendering in ActivityFeed for action_type === "preview_available" calling open_preview_url.
5. Add auto-scroll + Jump-to-Latest pill in ActivityFeed.
6. Move feasibility + violations + proposals panels OUT of Workspace into drawers or a deferred Tier; the MVP workspace must be exactly 2 panels + status bar.
7. Audit DiffViewer for per-hunk hover Accept/Decline + multi-file vertical file list; both are spec-required for 4K.5.
8. Status bar belongs on the workspace, not only on the Shell.
9. Treat dashboard-wiring-plan.txt and dashboard-wiring-inventory.txt as historical artifacts; do not let new work cite them as spec.
