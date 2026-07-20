<script lang="ts">
  // J-29 SLICE-1: first dashboard consumer of the J-28 usage_ledger rollup.
  // Polls `get_usage_rollup` (Tauri command -> usage_ledger::rollup_usage_turns
  // over SsbViewer::usage_turn_events) on mount + every 5s, mirroring the
  // MonitorWindow / CloudPanel polling cadence. Local lane rows surface token
  // counts with `no rate` in place of cost_micros (no rate card for local
  // inference); cloud rows surface cost_micros as USD. Never fakes $0.00.
  //
  // J-29 SLICE-2: also polls `get_backend_cost_rollup` (BackendCostAccrued
  // rollup) and `get_pricing_rollup` (per-phase / per-agent / per-tool USD)
  // on the same cadence and renders them as extra sections below the usage
  // table. `unknown_backend` rows are labelled explicitly; an
  // `observability_gate_off` pricing response surfaces the disabled-gate
  // notice rather than a silent empty table.

  import { Empty } from '$lib/ui';
  import * as stub from '$lib/api/stub';
  import type {
    UsageRollupRow,
    UsageRollupResponse,
    BackendCostRollupRow,
    BackendCostRollupResponse,
    PricingRollupResponse,
    AutoAdvanceStats,
  } from '$lib/api/types';

  let rows = $state<UsageRollupRow[]>([]);
  let backendRows = $state<BackendCostRollupRow[]>([]);
  let pricing = $state<PricingRollupResponse | null>(null);
  let autoAdvance = $state<AutoAdvanceStats | null>(null);
  let loading = $state<boolean>(true);
  let error = $state<string | null>(null);

  async function poll(): Promise<void> {
    try {
      const [usageResp, backendResp, pricingResp, autoResp] = await Promise.all([
        stub.get_usage_rollup(500),
        stub.get_backend_cost_rollup(2000),
        stub.get_pricing_rollup(500),
        stub.get_auto_advance_stats(),
      ]);
      rows = usageResp.rows;
      backendRows = backendResp.rows;
      pricing = pricingResp;
      autoAdvance = autoResp;
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
  // inference). Cloud rows surface the USD cost or `no rate for <model>` when
  // the lane has no rate card; local rows render raw token counts inline
  // (`in N / out M`) in the cost cell instead of a fake USD value.
  function formatCost(row: UsageRollupRow): string {
    if (row.cost_micros == null) return `no rate for ${row.model}`;
    const usd = row.cost_micros / 1_000_000;
    return `$${usd.toFixed(4)}`;
  }

  function formatMicros(micros: number): string {
    const usd = micros / 1_000_000;
    return `$${usd.toFixed(4)}`;
  }

  function formatUsd(usd: number): string {
    return `$${usd.toFixed(4)}`;
  }

  function formatTokens(n: number): string {
    if (n === 0) return '—';
    if (n >= 1_000_000) return `${(n / 1_000_000).toFixed(2)}M`;
    if (n >= 1_000) return `${(n / 1_000).toFixed(1)}k`;
    return String(n);
  }

  function backendLabel(row: BackendCostRollupRow): string {
    return row.unknown_backend ? `unknown (${row.backend_name})` : row.backend_name;
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
    {#if autoAdvance}
      <span
        class="autonomy-chip"
        title={`DERIVED: auto_executed_steps / total_steps = ${autoAdvance.auto_advanced}/${autoAdvance.total_steps}`}
      >autonomy {autoAdvance.auto_advanced}/{autoAdvance.total_steps}</span>
    {/if}
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
                {#if row.lane !== 'cloud'}in {row.input_tokens} / out {row.output_tokens}{:else}{formatCost(row)}{/if}
              </td>
              <td class="num">{row.row_count}</td>
            </tr>
          {/each}
        </tbody>
      </table>
    </div>
  {/if}

  <section class="rollup-section" aria-label="Backend cost rollup">
    <header class="rollup-header">
      <h3>Backend cost</h3>
      <span class="rollup-subtitle">
        {#if loading}
          loading…
        {:else}
          {backendRows.length} row{backendRows.length === 1 ? '' : 's'}
        {/if}
      </span>
    </header>
    {#if !loading && backendRows.length === 0 && !error}
      <div class="rollup-empty">
        <Empty
          title="no BackendCostAccrued rows yet"
          subtitle="no backend cost ledger rows to roll up"
        />
      </div>
    {:else if backendRows.length > 0}
      <div class="usage-table-wrap">
        <table class="usage-table">
          <thead>
            <tr>
              <th>backend</th>
              <th class="num">in</th>
              <th class="num">out</th>
              <th class="num">cost</th>
              <th class="num">invocations</th>
            </tr>
          </thead>
          <tbody>
            {#each backendRows as row (row.backend_name_hash)}
              <tr>
                <td class="mono" class:backend-unknown={row.unknown_backend}>
                  {backendLabel(row)}
                </td>
                <td class="num">{formatTokens(row.tokens_in)}</td>
                <td class="num">{formatTokens(row.tokens_out)}</td>
                <td class="num">{formatMicros(row.est_cost_micros)}</td>
                <td class="num">{row.invocations}</td>
              </tr>
            {/each}
          </tbody>
        </table>
      </div>
    {/if}
  </section>

  <section class="rollup-section" aria-label="Pricing rollup">
    <header class="rollup-header">
      <h3>Pricing rollup</h3>
      <span class="rollup-subtitle">
        {#if loading}
          loading…
        {:else if pricing?.observability_gate_off}
          <span class="rollup-notice" role="status">
            observability gate off (ELAI_AGENTIC_OBSERVABILITY_ENABLE)
          </span>
        {:else}
          {#if pricing}
            {pricing.by_phase.length + pricing.by_agent.length + pricing.by_tool.length} row{pricing.by_phase.length + pricing.by_agent.length + pricing.by_tool.length === 1 ? '' : 's'}
          {/if}
        {/if}
      </span>
    </header>
    {#if !loading && pricing?.observability_gate_off}
      <div class="rollup-empty">
        <Empty
          title="observability gate off (ELAI_AGENTIC_OBSERVABILITY_ENABLE)"
          subtitle="pricing rollup is not collected; enable the gate to populate"
        />
      </div>
    {:else if !loading && pricing && pricing.by_phase.length === 0 && pricing.by_agent.length === 0 && pricing.by_tool.length === 0}
      <div class="rollup-empty">
        <Empty
          title="no pricing rollup rows yet"
          subtitle="observability gate is on but no priced events have landed"
        />
      </div>
    {:else if pricing && !pricing.observability_gate_off}
      <div class="usage-table-wrap">
        <table class="usage-table">
          <thead>
            <tr>
              <th>bucket</th>
              <th>key</th>
              <th>category</th>
              <th class="num">cost</th>
              <th class="num">tokens</th>
            </tr>
          </thead>
          <tbody>
            {#each pricing.by_phase as row (`phase:${row.key}`)}
              <tr>
                <td class="mono">phase</td>
                <td class="mono">{row.key}</td>
                <td class="mono">{row.category}</td>
                <td class="num">{formatUsd(row.cost_usd)}</td>
                <td class="num">{formatTokens(row.input_tokens + row.output_tokens + row.cache_creation_input_tokens + row.cache_read_input_tokens)}</td>
              </tr>
            {/each}
            {#each pricing.by_agent as row (`agent:${row.key}`)}
              <tr>
                <td class="mono">agent</td>
                <td class="mono">{row.key}</td>
                <td class="mono">{row.category}</td>
                <td class="num">{formatUsd(row.cost_usd)}</td>
                <td class="num">{formatTokens(row.input_tokens + row.output_tokens + row.cache_creation_input_tokens + row.cache_read_input_tokens)}</td>
              </tr>
            {/each}
            {#each pricing.by_tool as row (`tool:${row.key}`)}
              <tr>
                <td class="mono">tool</td>
                <td class="mono">{row.key}</td>
                <td class="mono">{row.category}</td>
                <td class="num">{formatUsd(row.cost_usd)}</td>
                <td class="num">{formatTokens(row.input_tokens + row.output_tokens + row.cache_creation_input_tokens + row.cache_read_input_tokens)}</td>
              </tr>
            {/each}
          </tbody>
        </table>
      </div>
    {/if}
  </section>
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

  .autonomy-chip {
    font-family: var(--font-mono);
    font-size: 11px;
    color: var(--text-secondary);
    padding: 1px 6px;
    border: 1px solid var(--border-subtle);
    border-radius: 3px;
    white-space: nowrap;
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

  .rollup-section {
    display: flex;
    flex-direction: column;
    gap: 6px;
    flex-shrink: 0;
    border-top: 1px solid var(--border-subtle);
    padding-top: 10px;
  }

  .rollup-header {
    display: flex;
    align-items: baseline;
    justify-content: space-between;
    gap: 12px;
  }

  .rollup-header h3 {
    margin: 0;
    font-family: var(--font-display);
    font-weight: 300;
    font-size: 14px;
    color: var(--text-primary);
    letter-spacing: 0.04em;
  }

  .rollup-subtitle {
    font-family: var(--font-mono);
    font-size: 11px;
    color: var(--text-secondary);
  }

  .rollup-notice {
    color: var(--text-secondary);
    font-style: italic;
  }

  .rollup-empty {
    display: flex;
    align-items: center;
    justify-content: center;
    padding: 8px 0;
  }

  .usage-table tbody td.backend-unknown {
    color: var(--text-secondary);
    font-style: italic;
  }
</style>
