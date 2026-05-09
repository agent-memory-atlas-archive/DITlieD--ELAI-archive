<script lang="ts">
  // ============================================================
  // NotificationManager.svelte
  // Singleton — mount once in Workspace.svelte (or AppShell).
  // Polls get_fsm_state + get_system_status every 5s.
  // Fires Browser Notification API on:
  //   - FSM state change to INTERVENTION_REQUIRED / AUDITING / IDLE (task done)
  //   - pending_proposals becoming non-empty (intervention pending)
  //   - BLOCK verdict in supervision log (future SSB tail — graceful no-op)
  // Permission deferred: only requests on first intervention event.
  // Preference persisted in localStorage key "elai_notif_pref".
  // ============================================================

  import { onMount } from 'svelte';
  import * as stub from '$lib/api/stub';
  import type { FsmState } from '$lib/api/types';

  const STORAGE_KEY = 'elai_notif_pref';
  type NotifPref = 'granted' | 'denied' | 'deferred';

  // -- internal state (not reactive outside this component) --
  let lastFsmState: FsmState | null = null;
  let lastInterventionCount = 0;
  let pref: NotifPref = 'deferred';

  function loadPref(): NotifPref {
    try {
      const v = localStorage.getItem(STORAGE_KEY);
      if (v === 'granted' || v === 'denied' || v === 'deferred') return v;
    } catch {
      // storage unavailable
    }
    return 'deferred';
  }

  function savePref(p: NotifPref) {
    try {
      localStorage.setItem(STORAGE_KEY, p);
    } catch {
      // ignore
    }
    pref = p;
  }

  async function ensurePermission(): Promise<boolean> {
    if (pref === 'denied') return false;
    if (typeof Notification === 'undefined') return false;
    if (Notification.permission === 'granted') {
      savePref('granted');
      return true;
    }
    if (Notification.permission === 'denied') {
      savePref('denied');
      return false;
    }
    // Ask for permission (only if pref != denied)
    try {
      const result = await Notification.requestPermission();
      if (result === 'granted') {
        savePref('granted');
        return true;
      } else {
        savePref('denied');
        return false;
      }
    } catch {
      return false;
    }
  }

  function fire(title: string, body: string) {
    if (typeof Notification === 'undefined') return;
    if (Notification.permission !== 'granted') return;
    try {
      new Notification(title, { body, silent: false });
    } catch {
      // Notification API not available in this webview context
    }
  }

  async function poll() {
    try {
      const [fsm, sys] = await Promise.all([
        stub.get_fsm_state(),
        stub.get_system_status(),
      ]);

      const currentState = fsm.state;
      const prevState = lastFsmState;

      // FSM state change notifications
      if (prevState !== null && prevState !== currentState) {
        if (currentState === 'AUDITING') {
          const ok = await ensurePermission();
          if (ok) fire('ELAI — Auditing', 'Phase moved to AUDITING — review may be required');
        } else if (currentState === 'IDLE' && (prevState === 'MEMORIZING' || prevState === 'COMPLETE')) {
          const ok = await ensurePermission();
          if (ok) fire('ELAI — Task Complete', 'Autonomous task finished and returned to IDLE');
        }
      }

      lastFsmState = currentState;

      // Intervention pending: active_agents length or FSM state indicator
      // SystemStatus does not expose pending_interventions directly (B-36 gap).
      // Use agent count change as a proxy: if executor + auditor both active, likely executing.
      // Real intervention signal deferred until B-36 wires SSB pending_proposals.
      // No-op here; consent banner is rendered by the parent component instead.

    } catch {
      // sidecar offline — silent no-op
    }
  }

  onMount(() => {
    pref = loadPref();

    // If permission already granted previously, sync with browser
    if (typeof Notification !== 'undefined' && Notification.permission === 'granted') {
      savePref('granted');
    } else if (typeof Notification !== 'undefined' && Notification.permission === 'denied') {
      savePref('denied');
    }

    poll();
    const interval = setInterval(poll, 5_000);
    return () => clearInterval(interval);
  });

  // -- consent banner state (visible when pref === 'deferred' and browser supports Notification) --
  let showBanner = $state(false);
  $effect(() => {
    showBanner = pref === 'deferred' && typeof Notification !== 'undefined' && Notification.permission === 'default';
  });

  async function handleAllow() {
    const ok = await ensurePermission();
    showBanner = !ok && pref !== 'denied';
  }

  function handleDismiss() {
    savePref('deferred');
    showBanner = false;
  }
</script>

{#if showBanner}
  <div class="notif-banner" role="status" aria-live="polite">
    <span class="notif-banner__text">Enable desktop notifications for interventions?</span>
    <button class="notif-banner__allow" onclick={handleAllow}>Allow</button>
    <button class="notif-banner__dismiss" onclick={handleDismiss}>Not now</button>
  </div>
{/if}

<style>
  .notif-banner {
    position: fixed;
    bottom: 28px;
    left: 50%;
    transform: translateX(-50%);
    z-index: 9999;
    display: flex;
    align-items: center;
    gap: 10px;
    background: var(--bg-surface, #1a2029);
    border: 1px solid var(--border-default, rgba(255,255,255,0.10));
    border-radius: 6px;
    padding: 6px 14px;
    font-family: var(--font-sans, sans-serif);
    font-size: 12px;
    color: var(--text-secondary, rgba(255,255,255,0.65));
    box-shadow: 0 4px 24px rgba(0,0,0,0.45);
    animation: bannerSlideUp 180ms var(--ease-std, ease) both;
  }

  @keyframes bannerSlideUp {
    from { opacity: 0; transform: translateX(-50%) translateY(8px); }
    to   { opacity: 1; transform: translateX(-50%) translateY(0); }
  }

  .notif-banner__text { flex: 1; }

  .notif-banner__allow {
    background: transparent;
    border: 1px solid var(--teal-line, rgba(30,138,122,0.4));
    border-radius: 4px;
    color: var(--teal, #1e8a7a);
    font-family: var(--font-sans, sans-serif);
    font-size: 11px;
    padding: 2px 10px;
    cursor: pointer;
    transition: background 120ms ease;
  }
  .notif-banner__allow:hover { background: var(--teal-ghost, rgba(30,138,122,0.08)); }

  .notif-banner__dismiss {
    background: transparent;
    border: none;
    color: var(--text-ghost, rgba(255,255,255,0.3));
    font-family: var(--font-sans, sans-serif);
    font-size: 11px;
    cursor: pointer;
    padding: 2px 4px;
  }
  .notif-banner__dismiss:hover { color: var(--text-secondary, rgba(255,255,255,0.65)); }
</style>
