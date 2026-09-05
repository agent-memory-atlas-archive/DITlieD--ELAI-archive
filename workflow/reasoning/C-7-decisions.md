C-7 — Dashboard UX Completeness — Decisions
============================================
Session: s-32952
Date: 2026-05-18
Plan: docs/thefinalglance/C-plan/C-7_dashboard_ux_completeness.md
Prior session context: workflow/sessions/s-32952/context-package-C7.md
Prior attempt: C-7 first attempt was rate-limited; this rewrite resumes from a
              partial state where the SSB ChangeDecision variant was already
              landed in orchestrator/src/ssb.rs (SCHEMA_VERSION 49) but the
              corresponding Tauri commands, frontend wiring, ExploreWindow,
              RecoveryModal, responsive CSS, and contract test were not.

Scope shipped (W1-W6 + ChangeDecision wiring)
---------------------------------------------
W1  ExploreWindow.svelte (new) + 'explore' WindowType registered in
    types.ts + workspace.svelte.ts VALID_WINDOW_TYPES + shortcuts.ts
    SIDEBAR_WINDOWS + Ctrl+8 binding + WindowHost.svelte branch + Sidebar
    nav item with Compass icon. Q&A oracle UI: textarea + Ask button +
    scrollable response list with synthetic placeholder responses. No
    backend call yet (codebase-search command deferred to a follow-up);
    submit fires stub.logStub('explore.query', {query}) and appends a
    placeholder oracle row so the threading is observable.

W2  closedTabs ring buffer (max 10, FIFO eviction) added to
    workspace.svelte.ts. closeTab() captures {type, title, closedAt}
    BEFORE removing the tab, then pushes to the buffer. New
    openWindowRecovery()/closeWindowRecovery() actions toggle the
    showRecoveryModal flag. RecoveryModal.svelte mounted in
    screens/Workspace.svelte overlay snippet (DRIFT-1 correction: NOT
    layout/AppShell.svelte). Ctrl+Shift+H now calls
    workspace.openWindowRecovery() (replaces the prior no-op stub that
    pointed at openCommandPalette).

W3  review.svelte.ts store created with acceptAllSignal /
    rejectAllSignal monotonic counters + focusedHunkIndex.
    Ctrl+Shift+A / Ctrl+Shift+R added to shortcuts.ts, both guarded by
    primaryActiveTabIsReview() so they only fire when the primary
    panel's active tab is 'review' — no global bulk-accept from any
    context. ReviewWindow.svelte subscribes via $effect blocks that
    fire on each counter increment.

W4  ReviewWindow.svelte per-hunk row rewritten (DRIFT-4): hover-only
    inline buttons removed; replaced with an always-visible button row
    BELOW each hunk separator with Accept/Decline/Retry. Retry toggles
    a per-hunk note input (mirroring the existing per-file pattern).
    Hunk state keyed by `${path}#${hunkLabel}` so the same hunk on
    different files doesn't alias.

W5  base.css gained four responsive breakpoints:
      - @media (max-width: 1099px): hover tooltips disabled.
      - @media (max-width: 999px):  hamburger sidebar tier; panels-area
                                    min-width relaxed.
      - @media (max-width: 799px):  status bar tightens to 20px;
                                    paired with Workspace.svelte
                                    ResizeObserver that auto-collapses
                                    the secondary panel below 800px.
      - @media (max-width: 720px):  wordmark shrinks for very narrow.
    Sidebar.svelte added a Lucide Menu hamburger button (rendered only
    when viewport < 1000px); toggling sidebarCollapsed hides nav-list
    + bottom controls via the `.sidebar--narrow.sidebar--collapsed`
    selectors. Window resize listener keeps viewportIsNarrow in sync
    with the JS-side state for the hamburger button visibility.

W6  ReviewWindow.svelte wired to live data:
      - UnwiredNote banner removed.
      - FILES static const dropped; `files` is now a $state<FileChangeRow[]>
        populated via stub.get_file_changes(session.sessionId) on mount.
      - Empty branch fires when files.length === 0 (no static fallback).
      - acceptHunk / declineHunk / retryHunk / acceptAll / declineAll /
        retryFile invoke stub.accept_change / decline_change / retry_audit
        — the 3 new Tauri commands added in src-tauri/src/commands.rs.

ChangeDecision SSB variant + 3 new Tauri commands (DRIFT-2 Option A)
--------------------------------------------------------------------
SCHEMA_VERSION 48 -> 49 (already landed in a prior attempt and was kept
intact — verified via grep `SCHEMA_VERSION: i64 = 49`). 9-tuple lockstep
sites all present:
  1. SsbActionType enum variant (ssb.rs:1112)
  2. CHECK constraint string in SQL DDL (ssb.rs:467)
  3. as_sql_str arm (ssb.rs:1303)
  4. ssb_action_type_from_str arm (ssb.rs:1501)
  5. SsbPayload enum variant (ssb.rs:3902)
  6. action_type() match arm (ssb.rs:4092)
  7. as_default_typed_payload arm (ssb.rs:6763)
  8. all_test_payloads entry + count assert 101 (ssb.rs:8216, 8233)
  9. ChangeDecisionPayload struct (ssb.rs:5611)

Three new Tauri commands appended to commands.rs:
  - accept_change(state, path, hunkId?) -> emits ChangeDecision row with
    decision="accepted".
  - decline_change(state, path, hunkId?) -> decision="declined".
  - retry_audit(state, path, hunkId?, note?) -> decision="retried" with
    optional reviewer note.
All three route through a shared `emit_change_decision()` async helper
that constructs the ChangeDecisionPayload, calls writer.write_payload(),
and returns {ok: true, seq}. Error path: writer unavailable returns
"ssb writer unavailable; no session db open" — same string and signal
as the existing submit_task / set_gate_mode commands.

Registered in src-tauri/src/lib.rs invoke_handler list (3 new lines
after get_research_node_history).

Frontend wrappers added to api/stub.ts (accept_change / decline_change /
retry_audit) following the existing C-4 invoke-wrapper pattern: logStub
+ tryInvoke + graceful {ok: false, error} fallback for non-Tauri runtimes.
Return type `ChangeDecisionResponse = {ok, seq?, error?}`.

DRIFTs and how each was resolved
--------------------------------
DRIFT-1 (RecoveryModal mount path):
  Plan said `src/lib/screens/AppShell.svelte`. Actual: AppShell lives at
  `src/lib/layout/AppShell.svelte` and only plumbs snippets — the
  CommandPalette mount sits in `src/lib/screens/Workspace.svelte` overlay
  snippet. RecoveryModal mounted next to CommandPalette there. AppShell
  itself was NOT edited.

DRIFT-2 (3 new Tauri commands):
  Plan task #8 cited C-3 as the prerequisite. C-3 shipped 8 read-only
  commands; C-4 added get_file_changes; no plan ever added the mutation
  commands. Picked Option A from context-package §6 — add the 3 commands
  in C-7 with a single ChangeDecision SSB variant. Acceptable scope creep:
  payload is a 4-field struct, command bodies are 3-line wrappers, the
  SSB lockstep was already in place from the prior attempt.

DRIFT-3 (DispatchEditHunks SSB variant):
  Confirmed via grep that no DispatchEditHunks exists. Dropped entirely.
  The live data source for applied edits remains FileWrite rows
  (SsbActionType::FileWrite); ReviewWindow consumes get_file_changes
  which already filters to FileWrite-typed rows via
  SsbViewer::file_changes. The new ChangeDecision rows are separate
  verdict events — they do NOT carry diff payload.

DRIFT-4 (per-hunk buttons):
  ReviewWindow had hover-only Accept/Decline buttons rendered inline on
  the hunk separator. REWROTE per spec: removed the hover guard +
  inline render, added an always-visible `<div class="hunk-row">`
  beneath each hunk separator with three buttons (Accept/Decline/Retry).
  Retry toggles a per-hunk note input that mirrors the existing
  per-file retry-note bar; submitting the note fires retry_audit with
  the trimmed note (or undefined if empty).

DRIFT-5 (window recovery semantics):
  workspace.openTab(panelId, type) opens a fresh tab — it does NOT
  restore the closed tab's original id or window-internal state.
  Documented in the ClosedTabEntry doc-comment + this decisions file.
  Acceptable first-cut: closedTabs only stores {type, title, closedAt}
  per plan task #3.

Plan task #10 (Playwright visual.spec.ts): DEFERRED
---------------------------------------------------
Per CLAUDE.md "No new dependencies without confirmation", adding
@playwright/test as a devDependency requires explicit user approval.
The ExploreWindow / RecoveryModal / hunk-row elements ALL carry
data-testid attributes so the deferred Playwright suite can target them
cleanly when the dependency is approved:
  - data-testid="explore-window"
  - data-testid="explore-input"
  - data-testid="recovery-modal"
  - data-testid="recovery-row"
  - data-testid="sidebar-hamburger"
  - data-testid="review-window"
  - data-testid="hunk-row"
  - data-testid="hunk-accept" / "hunk-decline" / "hunk-retry"
Follow-up patch-N can add the suite once approved.

Plan task #11 (Ctrl+Shift+W shell): commented placeholder only
--------------------------------------------------------------
grep "'shell'"/"tier_0"/"Tier-0" returned 0 hits in workspace.svelte.ts.
shortcuts.ts gained a 5-line block comment documenting the deferral.
No SHORTCUTS entry added.

Files touched
-------------
MODIFY:
  - .elai_cc/crates/dashboard-v2/src/lib/api/types.ts
      added 'explore' to WindowType union.
  - .elai_cc/crates/dashboard-v2/src/lib/api/stub.ts
      added accept_change / decline_change / retry_audit wrappers +
      ChangeDecisionResponse interface.
  - .elai_cc/crates/dashboard-v2/src/lib/state/workspace.svelte.ts
      added 'explore' to VALID_WINDOW_TYPES; added ClosedTabEntry +
      closedTabs ring buffer (max 10) + showRecoveryModal flag +
      openWindowRecovery / closeWindowRecovery / setSecondaryCollapsed
      actions; closeTab() now pushes to closedTabs before removal.
  - .elai_cc/crates/dashboard-v2/src/lib/state/shortcuts.ts
      added 'explore' to SIDEBAR_WINDOWS; added Ctrl+8 / Ctrl+Shift+A /
      Ctrl+Shift+R SHORTCUTS entries; rewrote recover-closed action to
      call workspace.openWindowRecovery(); imported reviewStore;
      added primaryActiveTabIsReview() guard; documented Ctrl+Shift+W
      deferral.
  - .elai_cc/crates/dashboard-v2/src/lib/windows/ReviewWindow.svelte
      full rewrite — removed UnwiredNote banner, removed static FILES
      const, replaced with live get_file_changes wiring; rewrote
      per-hunk row to always-visible Accept/Decline/Retry; subscribed
      to reviewStore.acceptAllSignal / rejectAllSignal via $effect;
      wired all 6 action paths (file accept/decline/retry,
      hunk accept/decline/retry) to the 3 new Tauri commands.
  - .elai_cc/crates/dashboard-v2/src/lib/layout/Sidebar.svelte
      added Compass icon import + 'explore' nav item; added Menu icon
      import + hamburger button (visible only when viewport < 1000px)
      + viewportIsNarrow / sidebarCollapsed local state + window
      resize listener; added CSS for hamburger-btn + collapsed-narrow
      hides nav-list / sidebar-bottom.
  - .elai_cc/crates/dashboard-v2/src/lib/layout/WindowHost.svelte
      added ExploreWindow import + branch.
  - .elai_cc/crates/dashboard-v2/src/lib/screens/Workspace.svelte
      added RecoveryModal import + overlay-snippet mount; added
      ResizeObserver in onMount that calls
      workspace.setSecondaryCollapsed(true) when panels-area width
      drops below 800px (and undo when it rises again, if secondary
      has tabs).
  - .elai_cc/crates/dashboard-v2/src/styles/base.css
      appended 4 @media blocks (1099 / 999 / 799 / 720).
  - .elai_cc/crates/dashboard-v2/src-tauri/src/commands.rs
      added ChangeDecisionPayload import; appended accept_change /
      decline_change / retry_audit Tauri commands + shared
      emit_change_decision helper.
  - .elai_cc/crates/dashboard-v2/src-tauri/src/lib.rs
      registered the 3 new commands in invoke_handler.

CREATE:
  - .elai_cc/crates/dashboard-v2/src/lib/state/review.svelte.ts
  - .elai_cc/crates/dashboard-v2/src/lib/windows/ExploreWindow.svelte
  - .elai_cc/crates/dashboard-v2/src/lib/components/RecoveryModal.svelte
  - .elai_cc/crates/contracts/tests/c7_change_decision_contract.rs
  - workflow/reasoning/C-7-decisions.md (this file)

NOT TOUCHED (intentionally):
  - .elai_cc/crates/orchestrator/src/ssb.rs
      ChangeDecision SSB variant + SCHEMA_VERSION 49 + count 101 were
      already landed by the prior attempt; verified intact and reused.
  - .elai_cc/crates/dashboard-v2/src/lib/layout/AppShell.svelte
      DRIFT-1: overlay snippet mount lives in Workspace.svelte, not
      AppShell.
  - .elai_cc/crates/dashboard-v2/package.json
      Playwright dep not added (plan task #10 deferred).
  - .elai_cc/crates/dashboard-v2/tests/visual.spec.ts
      Not created (depends on @playwright/test).

Acceptance checklist (plan §ACCEPTANCE)
---------------------------------------
[x] WindowType 'explore' added to types.ts + VALID_WINDOW_TYPES.
[x] ExploreWindow renders Q&A oracle UI (input, submit, scrollable response).
[x] Ctrl+8 opens ExploreWindow; sidebar-8 in SHORTCUTS.
[x] workspace.closedTabs populated on closeTab(); max 10 entries; FIFO.
    (Plan said max 20; reduced to 10 per ORCHESTRATOR DECISIONS DRIFT-2
    user instruction "ring buffer max 10".)
[x] Ctrl+Shift+H opens RecoveryModal.
[x] RecoveryModal Restore button re-opens window via workspace.openTab.
[x] Old recover-closed -> openCommandPalette no-op removed.
[x] Ctrl+Shift+A fires reviewStore.acceptAllHunks() guarded to review tab.
[x] Ctrl+Shift+R fires reviewStore.rejectAllHunks() guarded to review tab.
[x] Per-hunk Accept/Decline/Retry button row below each hunk separator.
[x] ReviewWindow UnwiredNote removed; live get_file_changes on mount.
[x] Empty state when get_file_changes returns [] (no static stub).
[x] accept_change / decline_change / retry_audit invoked on actions.
[x] @media (max-width: 999px) hamburger sidebar; hamburger-btn in Sidebar.
[x] @media (max-width: 799px) auto-collapse secondary via ResizeObserver.
[ ] visual.spec.ts — DEFERRED (Playwright dep gate, per task #10).
[x] Ctrl+Shift+W — commented-out (5-line deferral block).
[ ] Cloud smoke test — DEFERRED (requires running dashboard + cloud OAuth).
[x] svelte-check passes — see verification log.
[x] cargo check (scoped) passes — see verification log.

Residuals (follow-up patch-N candidates)
----------------------------------------
1. Playwright visual regression suite (plan task #10) — needs user
   approval for @playwright/test devDependency. All target elements
   already carry data-testid attributes.
2. ExploreWindow live backend (codebase-search Tauri command) — UI is
   already wired; only the backend invoke needs to land.
3. ReviewWindow detail-fetch — today we synthesise one hunk per
   FileChangeRow from action+summary. To render real diff bodies the
   window needs to call get_action_detail(seq) per file and parse the
   FileWrite payload's `lines` field. Deferred to a follow-up patch
   so C-7 stays focused on the UX surface + verdict wiring.
4. Cloud smoke test (plan §observable_effect trigger) — orchestration
   cannot execute this; user runs it manually after merging.
