# B-26: Phase 4K.6 through 4K.13 — Dashboard Completion (V2 final pass)

Objective: close the remaining dashboard polish, intervention webhook, and
keyboard-shortcut gaps for V2. The previous scope of "build the windows"
is obsolete — V2 ships full MonitorWindow / ExploreWindow / ReviewWindow /
NotificationManager / tab system already. B-26 now narrows to the six
items below.

Estimated effort: 1 session.

Pre-requisites:
  - B-22 (Autonomous mode signals consumed by MonitorWindow) — shipped.
  - B-7 (decomposition tree for PLAN) — shipped.
  - B-11..B-14 (visual restoration) — shipped.
  - B-36 (Track D V2 wiring — real Tauri invokes behind the stubs).
    B-26 does NOT block on B-36; the stubs already produce realistic
    payloads via tryInvoke + mock fallback.

Parallel-safe with: B-25, B-27, B-28.

## V2 REBASELINE LOG

  2026-05-07 — first rebaseline. Re-scoped from "build new windows" to
              "polish + intervention lifecycle + animations".
  2026-05-11 — second rebaseline. Most rebaseline items from 2026-05-07
              are now shipped. B-26 narrows to the six real remaining
              gaps. V1 task list 1-30 and V1 DETERMINISTIC CHECK deleted
              (they referenced crates/dashboard/, which no longer
              exists).
  2026-05-11 — decisions recorded:
                D1 (WINDOW_STATE SSB vs localStorage) = (a) DROP. V2
                    persists workspace layout via localStorage key
                    `elai.dashboard-v2.workspace.v1`; cross-device sync
                    is not in the backlog. WINDOW_STATE +
                    HEALTH_SNAPSHOT NOT added to ssb.rs.
                T4 (extract InterventionCard.svelte) = DROP. Inline
                    orbit-card in MonitorWindow.svelte:376-409 has
                    exactly one consumer; CLAUDE.md prefers no
                    premature abstraction. Reopen if a second consumer
                    (CommandWindow takeover, MONITOR overlay, etc.)
                    appears.

## What is already shipped (do NOT re-implement)

  S1. MonitorWindow.svelte — progress bar, phase dots (.*-!o encoding),
      5s polling of get_fsm_state / get_system_status /
      get_supervision_log / get_auto_advance_stats / get_cloud_spend,
      inline intervention cards with Approve / Reject / Take over,
      supervision timeline, MSI / Proposals panels, thinking pulse,
      step-flash 100ms, fail-border 200ms, phaseBreath 3s, cardGlow.
  S2. ExploreWindow.svelte — Q&A chat stream through run_explore_query
      (crates/dashboard-v2/src/lib/api/stub.ts:1051), file refs, code
      blocks, citations, pending breath 2.2s.
  S3. ReviewWindow.svelte — per-file tabs, sticky file header, per-hunk
      hover Accept / Decline, retry-note bar, file-level Accept All /
      Decline All / Retry.
  S4. NotificationManager.svelte — Browser Notification API with
      localStorage permission preference, FSM state-change triggers,
      consent banner. Mounted in Workspace.svelte.
  S5. SSB types — InterventionRequest + InterventionResponse already in
      crates/orchestrator/src/ssb.rs:199-200 and
      crates/orchestrator/src/ssb_viewer.rs:148.
  S6. Tauri commands — submit_intervention_response,
      set_webhook_url, get_webhook_url all wired in
      crates/dashboard-v2/src-tauri/src/commands.rs (lines 199, 2552,
      2566). Stub wrappers in stub.ts:239, 1252, 1260.
  S7. Tab system — openTab / closeTab / moveTab in
      crates/dashboard-v2/src/lib/state/workspace.svelte.ts. Drag tabs
      between panels via Panel.svelte + AppShell.svelte drop-strip.
      Persistence to localStorage with schema v1 validation.
  S8. Keyboard shortcuts (partial) — Ctrl+1..7 sidebar, Ctrl+N new tab,
      Ctrl+W close, Ctrl+D toggle secondary, Ctrl+. slash palette,
      Ctrl+, settings, Ctrl+Shift+P command palette, Esc dismiss.
      See crates/dashboard-v2/src/lib/state/shortcuts.ts.
  S9. Session recovery — client-side via localStorage
      `elai.dashboard-v2.workspace.v1`. Per D1=(a), this is the final
      design; SSB persistence intentionally not added.

## Remaining tasks

  T1. Webhook firing.
      Add crates/dashboard-v2/src/lib/notifications/webhook.ts that
      POSTs a Discord/Slack-shaped JSON body to the stored webhook URL
      (read via get_webhook_url) when any of these fire from
      NotificationManager:
        - FSM enters INTERVENTION_REQUIRED / AUDITING / IDLE-after-task.
        - pending_interventions becomes non-empty.
        - Optional: BLOCK verdict from supervision log (graceful no-op
          if not surfaced yet).
      NotificationManager calls into webhook.ts in parallel with
      Notification.fire(); both gated on user preference. Errors are
      logged + swallowed (no UI failure on bad webhook URL).

  T2. SettingsWindow webhook URL field.
      Add a single-line input bound to set_webhook_url / get_webhook_url
      in the existing notifications row (SettingsWindow.svelte around
      line 785). Persist on blur, validate http(s):// only, show a
      small inline status (saved / invalid).

  T3. REVIEW window-scoped keyboard shortcuts.
      Add Ctrl+Shift+A (accept all) and Ctrl+Shift+R (reject all) in
      shortcuts.ts, guarded by a check that the active primary window
      is `review`. Single source of truth — do NOT add per-window
      keydown handlers.
      Also fix Ctrl+Shift+H — currently maps to openCommandPalette,
      which is misleading (the description says "Recover closed
      window"). Either:
        (a) remove the entry (cleanest);
        (b) implement a recently-closed ring buffer in
            workspace.svelte.ts and reopen on the shortcut.
      Default: (a). Defer (b) per V2 design.

  T4. Phase-dot model — real plan progression.
      Replace the static 5-phase FSM-state-to-phase map in
      MonitorWindow.svelte:80-110 with phase data from a plan-level SSB
      query. Either expose plan progress in FsmSnapshot, or add a
      lightweight get_plan_progress Tauri command + stub wrapper. The
      code comment at line 80 already flags this as deferred; B-26
      closes it.

  T5. Panel-transition animation backfill.
      Verify (and add where missing) fade-out 80ms / fade-in 120ms on
      tab switches inside Panel.svelte. Other DESIGN.md §8 timings
      (thinking pulse 2s, step completion flash 100ms, fail border
      200ms, progress dots 3s) are already in place — verify only,
      patch if drift.

  T6. Responsive breakpoints.
      <1000px viewport: collapse Sidebar.svelte into a hamburger toggle.
      <800px viewport: force workspace.secondaryCollapsed = true and
      hide PanelDivider. Test in tauri:dev by resizing the window.

## WIRING_CONTRACT

produces_symbols:
  - crates/dashboard-v2/src/lib/notifications/webhook.ts (NEW)
  - Ctrl+Shift+A / Ctrl+Shift+R shortcuts in shortcuts.ts (guarded by
    active window == 'review')
  - SettingsWindow.svelte webhook URL input
  - Phase-dot data path (extension of FsmSnapshot OR new
    get_plan_progress command)

consumes_symbols:
  - set_webhook_url / get_webhook_url Tauri commands (already shipped)
  - Notification API permission preference (already shipped)
  - pending_interventions from ssb_viewer.rs:1169 (already shipped)
  - workspace.primary.activeTabId for shortcut guard
  - Oracle Mode RESEARCHING coupling — used by ExploreWindow already;
    no new dependency in B-26 itself.

reachable_from:
  - Workspace.svelte (NotificationManager already mounted; webhook fires
    transparently as a side-effect of notification events).

stub_ban:
  - webhook.ts must call the real fetch — no logStub-only path.

frontend_pair: B-26 itself (no Rust pair).

observable_effect:
  trigger:
    - configure a webhook URL in SettingsWindow.
    - start a task that triggers an intervention.
  expect_artifact:
    - the webhook receives a POST with the intervention payload.
    - the Browser Notification API fires concurrently.
    - Ctrl+Shift+A / Ctrl+Shift+R works only when REVIEW is the active
      tab; no-op (or other shortcut wins) elsewhere.

## DETERMINISTIC_CHECK

```bash
# 1. Webhook frontend module exists and is imported by NotificationManager
test -f .elai_cc/crates/dashboard-v2/src/lib/notifications/webhook.ts
rg -q "from .*notifications/webhook" \
  .elai_cc/crates/dashboard-v2/src/lib/notifications/NotificationManager.svelte

# 2. SettingsWindow has a webhook URL input bound to set_webhook_url
rg -q "set_webhook_url" \
  .elai_cc/crates/dashboard-v2/src/lib/windows/SettingsWindow.svelte

# 3. REVIEW shortcuts registered + guarded by active window
rg -q "Ctrl\+Shift\+A" \
  .elai_cc/crates/dashboard-v2/src/lib/state/shortcuts.ts
rg -q "Ctrl\+Shift\+R" \
  .elai_cc/crates/dashboard-v2/src/lib/state/shortcuts.ts
rg -q "review" \
  .elai_cc/crates/dashboard-v2/src/lib/state/shortcuts.ts

# 4. Misleading Ctrl+Shift+H removed (or backed by a recently-closed list)
#    Default expectation: the shortcut entry is gone.
! rg -q "recover-closed" \
  .elai_cc/crates/dashboard-v2/src/lib/state/shortcuts.ts

# 5. Phase-dot data path no longer hard-codes the 5-phase map
rg -q "get_plan_progress|fsmSnap\.phases" \
  .elai_cc/crates/dashboard-v2/src/lib/windows/MonitorWindow.svelte

# 6. D1 = (a) — WINDOW_STATE / HEALTH_SNAPSHOT MUST NOT appear in ssb.rs
! rg -q "WindowState|HealthSnapshot" \
  .elai_cc/crates/orchestrator/src/ssb.rs
```

## ACCEPTANCE

- [ ] Webhook POST fires on every event that fires a Browser
      Notification (verified by pointing the URL at a local listener).
- [ ] SettingsWindow accepts, validates, and persists a webhook URL.
- [ ] Ctrl+Shift+A and Ctrl+Shift+R operate only inside REVIEW.
- [ ] Ctrl+Shift+H removed from shortcuts.ts (per default).
- [ ] Phase dots reflect real plan progress.
- [ ] tauri:dev resize: sidebar collapses to hamburger <1000px,
      secondary collapses <800px.
- [ ] Panel tab switches fade 80ms / 120ms per DESIGN.md §8.

## Files touched

- ADD:    crates/dashboard-v2/src/lib/notifications/webhook.ts
- MODIFY: crates/dashboard-v2/src/lib/notifications/NotificationManager.svelte
          (import + fire webhook in parallel with Notification API)
- MODIFY: crates/dashboard-v2/src/lib/windows/SettingsWindow.svelte
          (webhook URL input)
- MODIFY: crates/dashboard-v2/src/lib/windows/MonitorWindow.svelte
          (phase-dot data source)
- MODIFY: crates/dashboard-v2/src/lib/state/shortcuts.ts
          (Ctrl+Shift+A / Ctrl+Shift+R guarded by active window;
          remove Ctrl+Shift+H entry)
- MODIFY: crates/dashboard-v2/src/lib/layout/Sidebar.svelte
          (responsive collapse <1000px)
- MODIFY: crates/dashboard-v2/src/lib/screens/Workspace.svelte
          (responsive secondary collapse <800px)
- MODIFY: crates/dashboard-v2/src/lib/layout/Panel.svelte
          (verify tab fade 80ms / 120ms)
- MAYBE-MODIFY: crates/orchestrator/src/lib.rs OR src-tauri/src/commands.rs
          (only if T4 is implemented as a new get_plan_progress
          command rather than extending FsmSnapshot)
- CREATE: workflow/reasoning/B-26-decisions.md

## Next

B-27 (Cloud Mode), B-28 (Focus Tunnel) — both parallel-safe.
