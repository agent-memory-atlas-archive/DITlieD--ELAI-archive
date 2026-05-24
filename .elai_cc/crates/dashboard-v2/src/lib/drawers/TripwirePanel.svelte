<script lang="ts">
  // D-4 — TripwirePanel.
  //
  // Read-only view of recent tripwire rows via the
  // `get_recent_tripwires` Tauri command (additive — see
  // crates/dashboard-v2/src-tauri/src/commands_d4.rs).
  //
  // Renders:
  //   * last 50 tripwire rows (mix of `fired` + `auto_reverted`), newest first
  //   * per-row kind badge (KL / Δρ / IQR / PPL / DDM)
  //   * candidate_id + observed / threshold values for fired rows
  //   * template_id + retirement_reason + lineage for auto-revert rows
  //
  // No producer logic lives here — the orchestrator producer
  // (`agent_pool::observe_post_consensus_tripwires` calling
  // `tripwire::TripwireMonitor::observe` + `tripwire::auto_revert`) emits
  // the SSB rows separately. This panel polls on a 10s cadence.

  import { onMount, onDestroy } from 'svelte';
  // D-4 GAP-2: TripwireRow lifted to $lib/api/types.ts (shared surface
  // between commands_d4.rs and this panel). Local copy removed so the
  // 'fired_concurrent' kind admitted by SCHEMA_VERSION 108 +
  // TripwireFiredConcurrent stays in sync with the Rust DTO + the rest
  // of the dashboard. Widening the kind union here required updating
  // the Rust DTO in src-tauri/src/commands_d4.rs in the same commit.
  import type { TripwireRow } from '$lib/api/types';

  let rows = $state<TripwireRow[]>([]);
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
      console.warn(`[tripwires] invoke('${cmd}') failed`, e);
      return undefined;
    }
  }

  async function reload() {
    const out = await callInvoke<TripwireRow[]>('get_recent_tripwires', { limit: 50 });
    if (out === undefined) {
      rows = [];
      loadError = null;
    } else {
      rows = out;
      loadError = null;
    }
    loaded = true;
  }

  function shortId(s: string | null | undefined): string {
    if (!s) return '—';
    if (s.length <= 16) return s;
    return `${s.slice(0, 8)}…${s.slice(-4)}`;
  }

  function fmtTs(ms: number | null | undefined): string {
    if (!ms) return '—';
    try {
      return new Date(ms).toISOString().slice(0, 19).replace('T', ' ');
    } catch {
      return '—';
    }
  }

  function fmtNum(n: number | null | undefined): string {
    if (n === null || n === undefined) return '—';
    if (!Number.isFinite(n)) return String(n);
    if (Math.abs(n) >= 1000 || (Math.abs(n) > 0 && Math.abs(n) < 0.01)) {
      return n.toExponential(2);
    }
    return n.toFixed(3);
  }

  onMount(() => {
    reload();
    pollHandle = setInterval(reload, 10_000);
  });

  onDestroy(() => {
    if (pollHandle) {
      clearInterval(pollHandle);
      pollHandle = null;
    }
  });
</script>

<section class="tripwire-panel">
  <header class="panel-header">
    <h2>Tripwires</h2>
    <button class="reload-btn" onclick={reload}>Reload</button>
  </header>

  {#if !loaded}
    <p class="muted">Loading tripwire rows…</p>
  {:else if loadError}
    <div class="error-banner">
      <strong>Failed to load tripwire rows.</strong>
      <pre>{loadError}</pre>
    </div>
  {:else if rows.length === 0}
    <p class="muted">
      No tripwire rows recorded yet. Rows are emitted by the orchestrator
      after <code>agent_pool::observe_post_consensus_tripwires</code> trips
      one of the five detectors (KL / Δρ / IQR / perplexity / DDM).
    </p>
  {:else}
    <table>
      <thead>
        <tr>
          <th>Time</th>
          <th>Kind</th>
          <th>Detector</th>
          <th>Candidate / Template</th>
          <th>Observed</th>
          <th>Threshold</th>
          <th>Detail</th>
        </tr>
      </thead>
      <tbody>
        {#each rows as row (row.seq)}
          <tr class="row-{row.kind}">
            <td>{fmtTs(row.ts)}</td>
            <td><span class="badge badge-kind-{row.kind}">{row.kind}</span></td>
            <td>
              {#if row.tripwire_kind}
                <span class="badge badge-detector">{row.tripwire_kind}</span>
              {:else}
                —
              {/if}
            </td>
            <td>
              {#if row.kind === 'auto_reverted'}
                <code>{shortId(row.template_id)}</code>
              {:else}
                <code>{shortId(row.candidate_id)}</code>
              {/if}
            </td>
            <td>{fmtNum(row.observed_value)}</td>
            <td>{fmtNum(row.threshold)}</td>
            <td>
              {#if row.kind === 'fired'}
                <span class="event">id <code>{shortId(row.event_id)}</code></span>
                {#if row.source}
                  <span class="lineage"> · src <code>{row.source}</code></span>
                {/if}
              {:else if row.kind === 'fired_concurrent'}
                <span class="event">
                  <code>{row.tripwire_a ?? '—'}</code>
                  + <code>{row.tripwire_b ?? '—'}</code>
                </span>
                {#if row.fired_within_ms !== null && row.fired_within_ms !== undefined}
                  <span class="lineage"> within {row.fired_within_ms}ms</span>
                {/if}
              {:else}
                <span class="reason">{row.retirement_reason ?? ''}</span>
                {#if row.parent_template_id}
                  <span class="lineage"> ⟶ parent <code>{shortId(row.parent_template_id)}</code></span>
                {/if}
              {/if}
            </td>
          </tr>
        {/each}
      </tbody>
    </table>
  {/if}
</section>

<style>
  .tripwire-panel {
    display: flex;
    flex-direction: column;
    gap: 12px;
    font-family: var(--font-mono);
    font-size: 12px;
  }

  .panel-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
  }

  .panel-header h2 {
    margin: 0;
    font-size: 13px;
    font-weight: 500;
    letter-spacing: 0.04em;
    text-transform: uppercase;
  }

  .reload-btn {
    font-family: var(--font-mono);
    font-size: 11px;
    padding: 2px 8px;
    background: var(--bg-hover);
    border: 1px solid var(--border-default);
    border-radius: var(--r-xs);
    color: var(--text-primary);
    cursor: pointer;
  }

  .reload-btn:hover {
    background: var(--bg-surface);
  }

  table {
    width: 100%;
    border-collapse: collapse;
  }

  th, td {
    text-align: left;
    padding: 4px 6px;
    border-bottom: 1px solid var(--border-subtle);
  }

  th {
    font-weight: 500;
    color: var(--text-secondary);
    letter-spacing: 0.04em;
    text-transform: uppercase;
    font-size: 10px;
  }

  .row-auto_reverted {
    background: rgba(255, 80, 80, 0.07);
  }

  .badge {
    display: inline-block;
    padding: 1px 6px;
    border-radius: var(--r-xs);
    font-size: 10px;
    letter-spacing: 0.04em;
    text-transform: uppercase;
    background: var(--bg-hover);
    border: 1px solid var(--border-subtle);
  }

  .badge-kind-fired {
    background: rgba(220, 180, 80, 0.15);
    border-color: rgba(220, 180, 80, 0.4);
  }

  .badge-kind-auto_reverted {
    background: rgba(220, 80, 80, 0.15);
    border-color: rgba(220, 80, 80, 0.4);
  }

  /* D-4 GAP-2: concurrent-fire row styled distinctly from single-fire +
     auto_revert so the new SCHEMA_VERSION 108 variant surfaces at a
     glance. */
  .badge-kind-fired_concurrent {
    background: rgba(220, 120, 80, 0.18);
    border-color: rgba(220, 120, 80, 0.5);
  }

  .row-fired_concurrent {
    background: rgba(220, 120, 80, 0.06);
  }

  .badge-detector {
    background: var(--bg-surface);
  }

  .reason {
    color: var(--text-secondary);
  }

  .lineage {
    color: var(--text-secondary);
    font-size: 10px;
    margin-left: 4px;
  }

  .event {
    color: var(--text-secondary);
    font-size: 10px;
  }

  .muted {
    color: var(--text-secondary);
  }

  .error-banner {
    background: rgba(220, 80, 80, 0.1);
    border: 1px solid rgba(220, 80, 80, 0.4);
    padding: 8px;
    border-radius: var(--r-sm);
  }
</style>
