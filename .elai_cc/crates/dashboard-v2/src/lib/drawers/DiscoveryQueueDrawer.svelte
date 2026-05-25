<script lang="ts">
  // D-27 — DiscoveryQueueDrawer.
  //
  // Lists recent DiscoveryCandidate-classified templates pending
  // adversarial verification. Pure read off the D-27 composite
  // `get_mode_routing_refusal_combined`; we filter the
  // `discovery_queue` field locally for the drawer table.
  //
  // Each row shows: template_id, novelty class, dkg_match_count,
  // days_waiting, and a Quick-promote button. The button is a no-op
  // signal-emitter for now (console.log) — promotion semantics will
  // wire to a follow-up adversarial-verification dispatcher; the
  // affordance lives in the dashboard so operators can flag the
  // candidate without paying CLI friction.
  //
  // Read-only producer-side. No new SSB variant.
  //
  // Grounding: arXiv:2406.05761 (LLM novelty detection — three-bucket
  // taxonomy); docs/specs/guide/real-problem-solving-grounded-2026-
  // 05-19.md §1 Mode C + §4.8.

  type DiscoveryQueueRow = {
    template_id: string;
    class: string;
    dkg_match_count: number;
    near_duplicate_threshold: number;
    ts_ms: number;
    days_waiting: number;
  };

  type ModeRoutingRefusalView = {
    mode_breakdown: Record<string, number>;
    refusal_rate_current: number;
    refusal_health_band: string;
    discovery_queue: DiscoveryQueueRow[];
    recent_classifications: unknown[];
    dominant_mode: string | null;
    limit_applied: number;
  };

  // D-27 GAP-5: ratify/reject UI scaffold state.
  //
  // Backend (DiscoveryRatified / DiscoveryRejected SSB variants +
  // ratify_discovery / reject_discovery Tauri commands + ReviewStatus
  // enum + reviewer_id persistence) is DEFERRED to D-27.1 per the
  // KNOWN-RESIDUAL list in workflow/reasoning/D-27-decisions.md and
  // mirrors the D-15 KR-2 deferral pattern (see
  // workflow/reasoning/D-15-decisions.md). Buttons are rendered but
  // disabled-by-default with an explanatory tooltip; click handlers
  // are pure no-op signal-emitters that surface a toast + console
  // entry so the operator knows the request was received but not
  // persisted.
  let reviewerId = $state('');
  let pendingNotice = $state<string | null>(null);
  let pendingNoticeHandle: ReturnType<typeof setTimeout> | null = null;

  const DEFERRED_TOOLTIP =
    'Ratify/reject backend deferred to D-27.1 (see D-27-decisions.md DECISION-7). ' +
    'Button is UI-only scaffold; clicks surface the request but do NOT write a SSB row.';

  let rows = $state<DiscoveryQueueRow[]>([]);
  let loaded = $state(false);
  let loadError = $state<string | null>(null);
  let pollHandle: ReturnType<typeof setInterval> | null = null;

  async function callInvoke<T>(cmd: string, args?: Record<string, unknown>): Promise<T | undefined> {
    if (typeof window === 'undefined') return undefined;
    const tauriAvail = !!(window as unknown as { __TAURI_INTERNALS__?: unknown }).__TAURI_INTERNALS__;
    if (!tauriAvail) return undefined;
    try {
      const mod = await import('@tauri-apps/api/core');
      return (await mod.invoke<T>(cmd, args)) as T;
    } catch (e) {
      console.warn(`[discovery-queue] invoke('${cmd}') failed`, e);
      return undefined;
    }
  }

  async function reload() {
    try {
      const out = await callInvoke<ModeRoutingRefusalView>(
        'get_mode_routing_refusal_combined',
        { limit: 100 },
      );
      if (out === undefined) {
        rows = [];
        loadError = null;
      } else {
        rows = out.discovery_queue;
        loadError = null;
      }
    } catch (e) {
      loadError = String(e);
    } finally {
      loaded = true;
    }
  }

  $effect(() => {
    reload();
    pollHandle = setInterval(reload, 10_000);
    return () => {
      if (pollHandle) {
        clearInterval(pollHandle);
        pollHandle = null;
      }
      if (pendingNoticeHandle) {
        clearTimeout(pendingNoticeHandle);
        pendingNoticeHandle = null;
      }
    };
  });

  function fmtTs(ms: number): string {
    if (!ms) return '—';
    try {
      return new Date(ms).toISOString().slice(0, 19).replace('T', ' ');
    } catch {
      return '—';
    }
  }

  // D-27 GAP-5: scaffold notifier — logs to console + sets a short-
  // lived toast banner the template renders inline. Pure UI signal,
  // does NOT write a SSB row (deferred per DECISION-7).
  function notifyReviewerActionDeferred(
    rowId: string,
    action: 'ratify' | 'reject',
  ): void {
    const reviewer = reviewerId.trim();
    console.log('[discovery-queue] reviewer-action-deferred', {
      action,
      template_id: rowId,
      reviewer_id: reviewer || '(unset)',
      reason: 'Ratify/reject backend deferred per D-15 KR-2 + D-27 DECISION-7',
    });
    const label = action === 'ratify' ? 'Ratify' : 'Reject';
    pendingNotice =
      `${label} request received for ${rowId} ` +
      `(reviewer: ${reviewer || 'unset'}) — backend deferred per ` +
      `D-15 KR-2 + D-27 DECISION-7. No SSB row written.`;
    if (pendingNoticeHandle !== null) {
      clearTimeout(pendingNoticeHandle);
    }
    // Auto-dismiss after ~6s so the banner doesn't pile up over a
    // sequence of clicks.
    pendingNoticeHandle = setTimeout(() => {
      pendingNotice = null;
      pendingNoticeHandle = null;
    }, 6_000);
  }
</script>

<section class="discovery-queue">
  <header class="dq-header">
    <h3>Discovery Queue</h3>
    <span class="dq-sub">{rows.length} pending</span>
    {#if loadError}
      <span class="dq-error">{loadError}</span>
    {/if}
  </header>

  <!--
    D-27 GAP-5 UI scaffold:
      - reviewer_id input (free-form; persistence deferred)
      - Ratify / Reject buttons per row (disabled-by-default w/ tooltip)
      - inline pendingNotice banner (auto-dismissed after ~6s)
    Backend write path deferred to D-27.1.
  -->
  <div class="dq-reviewer-row">
    <label class="dq-reviewer-label" for="dq-reviewer-id">reviewer_id</label>
    <input
      id="dq-reviewer-id"
      class="dq-reviewer-input"
      type="text"
      placeholder="(unset — deferred)"
      bind:value={reviewerId}
      title="Reviewer identifier (UI scaffold; persistence deferred to D-27.1)"
    />
    <span class="dq-deferred-note" title={DEFERRED_TOOLTIP}>
      ratify/reject backend deferred → D-27.1
    </span>
  </div>

  {#if pendingNotice !== null}
    <div class="dq-pending-banner" role="status">{pendingNotice}</div>
  {/if}

  {#if !loaded}
    <p class="dq-empty">loading discovery queue…</p>
  {:else if rows.length === 0}
    <p class="dq-empty">no DiscoveryCandidate classifications in the recent window.</p>
  {:else}
    <table class="dq-table">
      <thead>
        <tr>
          <th>template_id</th>
          <th>class</th>
          <th>dkg matches</th>
          <th>days waiting</th>
          <th>classified at</th>
          <th>review</th>
        </tr>
      </thead>
      <tbody>
        {#each rows as row (row.template_id + ':' + row.ts_ms)}
          <tr>
            <td class="dq-id">{row.template_id}</td>
            <td>{row.class}</td>
            <td class="dq-num">{row.dkg_match_count}</td>
            <td class="dq-num">{row.days_waiting}</td>
            <td>{fmtTs(row.ts_ms)}</td>
            <td class="dq-actions">
              <button
                class="dq-action dq-ratify"
                type="button"
                disabled
                title={DEFERRED_TOOLTIP}
                onclick={() => notifyReviewerActionDeferred(row.template_id, 'ratify')}
              >Ratify</button>
              <button
                class="dq-action dq-reject"
                type="button"
                disabled
                title={DEFERRED_TOOLTIP}
                onclick={() => notifyReviewerActionDeferred(row.template_id, 'reject')}
              >Reject</button>
            </td>
          </tr>
        {/each}
      </tbody>
    </table>
  {/if}
</section>

<style>
  .discovery-queue {
    display: flex;
    flex-direction: column;
    gap: 8px;
    padding: 8px;
  }
  .dq-header {
    display: flex;
    align-items: baseline;
    gap: 12px;
    border-bottom: 1px solid var(--border-subtle, #2a2a2a);
    padding-bottom: 4px;
  }
  .dq-header h3 {
    margin: 0;
    font-size: 12px;
    letter-spacing: 0.05em;
    text-transform: uppercase;
  }
  .dq-sub {
    font-size: 10px;
    color: var(--text-muted, #888);
  }
  .dq-error {
    font-size: 10px;
    color: #f66;
  }
  .dq-empty {
    font-size: 11px;
    color: var(--text-muted, #888);
    margin: 4px 0;
  }
  .dq-table {
    width: 100%;
    border-collapse: collapse;
    font-size: 11px;
  }
  .dq-table th,
  .dq-table td {
    border-bottom: 1px solid var(--border-subtle, #232323);
    padding: 4px 6px;
    text-align: left;
  }
  .dq-table th {
    font-weight: 500;
    color: var(--text-muted, #888);
    text-transform: uppercase;
    font-size: 10px;
    letter-spacing: 0.05em;
  }
  .dq-id {
    font-family: var(--font-mono, monospace);
    font-size: 10px;
  }
  .dq-num {
    text-align: right;
    font-variant-numeric: tabular-nums;
  }
  /* D-27 GAP-5 — ratify/reject UI scaffold styles. Buttons are
     rendered disabled-by-default with a deferral tooltip. */
  .dq-reviewer-row {
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 4px 0;
  }
  .dq-reviewer-label {
    font-family: var(--font-mono, monospace);
    font-size: 10px;
    letter-spacing: 0.05em;
    text-transform: uppercase;
    color: var(--text-muted, #888);
  }
  .dq-reviewer-input {
    font-family: var(--font-mono, monospace);
    font-size: 11px;
    padding: 2px 6px;
    background: var(--surface-2, #1f1f1f);
    color: inherit;
    border: 1px solid var(--border-subtle, #2a2a2a);
    border-radius: var(--r-xs, 3px);
    flex: 0 1 200px;
  }
  .dq-deferred-note {
    font-family: var(--font-mono, monospace);
    font-size: 10px;
    color: var(--warn, #f9c44d);
    margin-left: auto;
  }
  .dq-pending-banner {
    font-family: var(--font-mono, monospace);
    font-size: 11px;
    padding: 6px 8px;
    background: rgba(249, 196, 77, 0.08);
    border: 1px solid var(--warn, #f9c44d);
    color: var(--warn, #f9c44d);
    border-radius: var(--r-xs, 3px);
  }
  .dq-actions {
    display: flex;
    gap: 4px;
  }
  .dq-action {
    font-family: var(--font-mono, monospace);
    font-size: 10px;
    padding: 2px 8px;
    border: 1px solid var(--border-subtle, #2a2a2a);
    background: var(--surface-2, #1f1f1f);
    color: inherit;
    cursor: pointer;
    border-radius: var(--r-xs, 3px);
  }
  .dq-action:disabled {
    cursor: not-allowed;
    opacity: 0.55;
  }
  .dq-ratify:not(:disabled):hover {
    border-color: #4a8;
  }
  .dq-reject:not(:disabled):hover {
    border-color: var(--fail, #dc3c3c);
  }
</style>
