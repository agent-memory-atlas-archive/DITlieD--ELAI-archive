<script lang="ts">
  // F-5 / Slice A5 -- StalenessQuarantineDrawer.
  //
  // Read-only view of recent StalenessQuarantineBroadcast SSB rows via the
  // `get_staleness_quarantine` Tauri command
  // (see crates/dashboard-v2/src-tauri/src/commands_f5.rs).
  //
  // Renders:
  //   * summary header: total quarantined fact count + aggregate demotion /
  //     Cosmos-flag counts + total dropped (queue-overflow) records
  //   * last 50 StalenessQuarantineBroadcast rows, newest first
  //   * per-row: timestamp, fact id, staleness ratio, dependents demoted,
  //     Cosmos artifacts flagged, and a drop-pressure badge when dropped_count>0
  //
  // No wire-dark ENABLE gate (W-2): the broadcast writes its row whenever a
  // memory-decay tick quarantines a fact. The dropped_count surfaces the bounded
  // QuarantineBroadcastQueue's DROP-OLDEST overflow so silent drops are visible.
  // Polls on a 5s cadence (mirrors SecurityMonotonicityDrawer sibling pattern).
  // cross-lang-pair: dashboard/tauri/api

  import { onMount, onDestroy } from 'svelte';
  import type {
    StalenessQuarantineResponse,
    StalenessQuarantineRow,
  } from '../api/types';

  let resp = $state<StalenessQuarantineResponse | null>(null);
  let loaded = $state(false);
  let loadError = $state<string | null>(null);
  let pollHandle: ReturnType<typeof setInterval> | null = null;

  async function callInvoke<T>(
    cmd: string,
    args?: Record<string, unknown>
  ): Promise<T | undefined> {
    if (typeof window === 'undefined') return undefined;
    const tauriAvail = !!(window as unknown as { __TAURI_INTERNALS__?: unknown })
      .__TAURI_INTERNALS__;
    if (!tauriAvail) return undefined;
    try {
      const mod = await import('@tauri-apps/api/core');
      return (await mod.invoke<T>(cmd, args)) as T;
    } catch (e) {
      console.warn(`[staleness-quarantine] invoke('${cmd}') failed`, e);
      return undefined;
    }
  }

  async function reload() {
    const out = await callInvoke<StalenessQuarantineResponse>(
      'get_staleness_quarantine'
    );
    if (out === undefined) {
      resp = null;
      loadError = null;
    } else {
      resp = out;
      loadError = null;
    }
    loaded = true;
  }

  function fmtTs(ms: number | null | undefined): string {
    if (!ms) return '—';
    try {
      return new Date(ms).toISOString().slice(0, 19).replace('T', ' ');
    } catch {
      return '—';
    }
  }

  function fmtRatio(r: number): string {
    return r.toFixed(2);
  }

  // Aggregate counts derived from the rows (the command returns only the row
  // list; the panel aggregates client-side, mirroring the
  // SecurityMonotonicityDrawer countsByLoopKind shape).
  let totalQuarantined = $derived(resp ? resp.rows.length : 0);

  let totalDemoted = $derived(
    resp ? resp.rows.reduce((sum, row) => sum + row.dependents_demoted, 0) : 0
  );

  let totalFlagged = $derived(
    resp
      ? resp.rows.reduce((sum, row) => sum + row.cosmos_artifacts_flagged, 0)
      : 0
  );

  let totalDropped = $derived(
    resp ? resp.rows.reduce((sum, row) => sum + row.dropped_count, 0) : 0
  );

  onMount(() => {
    reload();
    pollHandle = setInterval(reload, 5_000);
  });

  onDestroy(() => {
    if (pollHandle) {
      clearInterval(pollHandle);
      pollHandle = null;
    }
  });
</script>

<section class="staleness-quarantine-panel">
  <header class="panel-header">
    <h2>Staleness Quarantine (F-5)</h2>
    <button class="reload-btn" onclick={reload}>Reload</button>
  </header>

  {#if !loaded}
    <p class="muted">Loading staleness-quarantine rows…</p>
  {:else if loadError}
    <div class="error-banner">
      <strong>Failed to load staleness-quarantine rows.</strong>
      <pre>{loadError}</pre>
    </div>
  {:else if resp === null}
    <p class="muted">
      Tauri IPC unavailable (running in browser preview mode). Mount the
      dashboard in the Tauri host to see live staleness-quarantine data.
    </p>
  {:else}
    <div class="summary-row">
      <span class="summary-label">Quarantined:</span>
      <span class="summary-value">{totalQuarantined}</span>
      <span class="badge badge-demoted">{totalDemoted} demoted</span>
      <span class="badge badge-flagged">{totalFlagged} cosmos flagged</span>
      {#if totalDropped > 0}
        <span class="badge badge-dropped">{totalDropped} dropped</span>
      {/if}
    </div>

    {#if resp.rows.length === 0}
      <p class="muted">
        No <code>StalenessQuarantineBroadcast</code> rows recorded yet. Rows are
        emitted at the orchestrator/CLI memory-decay-tick boundary when a fact's
        staleness ratio crosses the decay threshold; the cascade demotes
        downstream DKG concepts (<code>concept_stale=2</code>) and flags Cosmos
        artifacts.
      </p>
    {:else}
      <table>
        <thead>
          <tr>
            <th>Time</th>
            <th>Fact</th>
            <th>Staleness</th>
            <th>Demoted</th>
            <th>Cosmos</th>
            <th>Dropped</th>
          </tr>
        </thead>
        <tbody>
          {#each resp.rows as row (row.seq)}
            <tr class="row-quarantine">
              <td>{fmtTs(row.ts)}</td>
              <td class="fact-cell" title={row.fact_id}>{row.fact_id}</td>
              <td><span class="badge badge-ratio">{fmtRatio(row.staleness_ratio)}</span></td>
              <td>{row.dependents_demoted}</td>
              <td>{row.cosmos_artifacts_flagged}</td>
              <td>
                {#if row.dropped_count > 0}
                  <span class="badge badge-dropped">{row.dropped_count}</span>
                {:else}
                  <span class="muted">0</span>
                {/if}
              </td>
            </tr>
          {/each}
        </tbody>
      </table>
    {/if}
  {/if}
</section>

<style>
  .staleness-quarantine-panel {
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

  .summary-row {
    display: flex;
    align-items: center;
    gap: 8px;
    flex-wrap: wrap;
  }

  .summary-label {
    color: var(--text-secondary);
    font-size: 11px;
    text-transform: uppercase;
    letter-spacing: 0.04em;
  }

  .summary-value {
    font-weight: 600;
    font-size: 13px;
  }

  table {
    width: 100%;
    border-collapse: collapse;
  }

  th,
  td {
    text-align: left;
    padding: 4px 6px;
    border-bottom: 1px solid var(--border-subtle);
    vertical-align: top;
  }

  th {
    font-weight: 500;
    color: var(--text-secondary);
    letter-spacing: 0.04em;
    text-transform: uppercase;
    font-size: 10px;
  }

  .row-quarantine {
    background: rgba(220, 160, 60, 0.07);
  }

  .fact-cell {
    max-width: 180px;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
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

  .badge-ratio {
    background: rgba(220, 160, 60, 0.15);
    border-color: rgba(220, 160, 60, 0.4);
  }

  .badge-demoted {
    background: rgba(150, 110, 220, 0.15);
    border-color: rgba(150, 110, 220, 0.4);
  }

  .badge-flagged {
    background: rgba(80, 200, 150, 0.12);
    border-color: rgba(80, 200, 150, 0.35);
  }

  .badge-dropped {
    background: rgba(220, 80, 80, 0.15);
    border-color: rgba(220, 80, 80, 0.4);
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
