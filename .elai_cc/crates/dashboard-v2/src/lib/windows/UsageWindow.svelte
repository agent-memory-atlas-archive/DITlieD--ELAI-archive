<script lang="ts">
  // J-29 SLICE-1: first dashboard consumer of the J-28 usage_ledger rollup.
  // Polls `get_usage_rollup` (Tauri command -> usage_ledger::rollup_usage_turns
  // over SsbViewer::usage_turn_events) on mount + every 5s, mirroring the
  // MonitorWindow / CloudPanel polling cadence. Local lane rows surface token
  // counts with `no rate` in place of cost_micros (no rate card for local
  // inference); cloud rows surface cost_micros as USD. Never fakes $0.00.

  import { Empty } from '$lib/ui';
  import * as stub from '$lib/api/stub';
  import type { UsageRollupRow, UsageRollupResponse } from '$lib/api/types';

  let rows = $state<UsageRollupRow[]>([]);
  let loading = $state<boolean>(true);
  let error = $state<string | null>(null);

  async function poll(): Promise<void> {
    try {
      const resp: UsageRollupResponse = await stub.get_usage_rollup(500);
      rows = resp.rows;
      error = null;
    } catch (e) {
      error = e instanceof Error ? e.message : String(e);
    } finally {
      loading = false;
    }
  }

  $effect(() => {
    let alive = true;
    poll();
    const interval = setInterval(() => {
      if (!alive) return;
      poll();
    }, 5000);
    return () => {
      alive = false;
      clearInterval(interval);
    };
  });

  // cost_micros is null when no rate could be derived for the lane (e.g. local
  // inference). The UI MUST surface "no rate" rather than a silent $0.00.
  function formatCost(row: UsageRollupRow): string {
    if (row.cost_micros == null) return 'no rate';
    const usd = row.cost_micros / 1_000_000;
    return `$${usd.toFixed(4)}`;
  }

  function formatTokens(n: number): string {
    if (n === 0) return '—';
    if (n >= 1_000_000) return `${(n / 1_000_000).toFixed(2)}M`;
    if (n >= 1_000) return `${(n / 1_000).toFixed(1)}k`;
    return String(n);
  }
</script>

<section class="usage-window" aria-label="Usage rollup">
  <header class="usage-header">
    <h2>Usage</h2>
    <span class="usage-subtitle">
      {#if loading}
        loading…
      {:else if error}
        <span class="usage-error" role="alert">error: {error}</span>
      {:else}
        {rows.length} row{rows.length === 1 ? '' : 's'}
      {/if}
    </span>
  </header>

  {#if !loading && rows.length === 0 && !error}
    <div class="usage-empty">
      <Empty
        title="no UsageTurnEvent rows yet"
        subtitle="usage_ledger has no rollup to surface"
      />
    </div>
  {:else if rows.length > 0}
    <div class="usage-table-wrap">
      <table class="usage-table">
        <thead>
          <tr>
            <th>model</th>
            <th>lane</th>
            <th>task</th>
            <th>session</th>
            <th class="num">in</th>
            <th class="num">out</th>
            <th class="num">cache_r</th>
            <th class="num">cache_c</th>
            <th class="num">cost</th>
            <th class="num">rows</th>
          </tr>
        </thead>
        <tbody>
          {#each rows as row (row.identity)}
            <tr>
              <td class="mono">{row.model}</td>
              <td class="lane" class:lane-local={row.lane !== 'cloud'}>
                {row.lane}
              </td>
              <td class="mono">{row.task_id ?? '—'}</td>
              <td class="mono">{row.session_id ?? '—'}</td>
              <td class="num">{formatTokens(row.input_tokens)}</td>
              <td class="num">{formatTokens(row.output_tokens)}</td>
              <td class="num">{formatTokens(row.cache_read_tokens)}</td>
              <td class="num">{formatTokens(row.cache_creation_tokens)}</td>
              <td class="num" class:cost-no-rate={row.cost_micros == null}>
                {formatCost(row)}
              </td>
              <td class="num">{row.row_count}</td>
            </tr>
          {/each}
        </tbody>
      </table>
    </div>
  {/if}
</section>

<style>
  .usage-window {
    display: flex;
    flex-direction: column;
    height: 100%;
    width: 100%;
    padding: 16px 20px;
    gap: 12px;
    overflow: hidden;
  }

  .usage-header {
    display: flex;
    align-items: baseline;
    justify-content: space-between;
    gap: 12px;
    flex-shrink: 0;
  }

  .usage-header h2 {
    margin: 0;
    font-family: var(--font-display);
    font-weight: 300;
    font-size: 18px;
    color: var(--text-primary);
    letter-spacing: 0.04em;
  }

  .usage-subtitle {
    font-family: var(--font-mono);
    font-size: 11px;
    color: var(--text-secondary);
  }

  .usage-error {
    color: var(--danger, #e66);
  }

  .usage-empty {
    flex: 1 1 auto;
    display: flex;
    align-items: center;
    justify-content: center;
  }

  .usage-table-wrap {
    flex: 1 1 auto;
    overflow: auto;
    min-height: 0;
  }

  .usage-table {
    width: 100%;
    border-collapse: collapse;
    font-family: var(--font-mono);
    font-size: 11px;
  }

  .usage-table thead th {
    position: sticky;
    top: 0;
    background: var(--bg-base);
    color: var(--text-secondary);
    font-weight: 500;
    text-align: left;
    padding: 6px 10px;
    border-bottom: 1px solid var(--border-subtle);
    white-space: nowrap;
  }

  .usage-table thead th.num {
    text-align: right;
  }

  .usage-table tbody td {
    padding: 5px 10px;
    border-bottom: 1px solid var(--border-subtle);
    color: var(--text-primary);
    white-space: nowrap;
  }

  .usage-table tbody td.num {
    text-align: right;
  }

  .usage-table tbody td.mono {
    color: var(--text-secondary);
  }

  .usage-table tbody td.lane-local {
    color: var(--teal);
  }

  .usage-table tbody td.cost-no-rate {
    color: var(--text-secondary);
    font-style: italic;
  }
</style>
