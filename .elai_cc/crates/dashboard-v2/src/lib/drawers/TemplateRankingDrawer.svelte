<script lang="ts">
  // F-7 / Slice A5 -- TemplateRankingDrawer.
  //
  // Read-only view of recent TemplateUtilityRanked SSB rows via the
  // `get_template_ranking` Tauri command
  // (see crates/dashboard-v2/src-tauri/src/commands_f7.rs).
  //
  // Renders the F-7 before/after side-by-side:
  //   * summary header: total ranked templates + the count whose smoothed
  //     utility dropped below their raw first_try_ratio (low-sample distrust)
  //   * last 50 TemplateUtilityRanked rows, sorted by utility DESC (the
  //     "ranking" surface), each showing utility (after) AND first_try_ratio
  //     (before) SIDE-BY-SIDE plus the sample size that drives the gap
  //   * a delta badge per row = utility - first_try_ratio (negative = the
  //     template was being over-trusted on thin evidence; Laplace pulled it down)
  //
  // No wire-dark ENABLE gate (W-2): the beacon is default-ON; any
  // ELAI_TEMPLATE_CBR_* env is a producer-side kill-switch. Polls on a 5s
  // cadence (mirrors the StalenessQuarantineDrawer sibling pattern).
  // cross-lang-pair: dashboard/tauri/api

  import { onMount, onDestroy } from 'svelte';
  import type {
    TemplateRankingResponse,
    TemplateRankingRow,
  } from '../api/types';

  let resp = $state<TemplateRankingResponse | null>(null);
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
      console.warn(`[template-ranking] invoke('${cmd}') failed`, e);
      return undefined;
    }
  }

  async function reload() {
    const out = await callInvoke<TemplateRankingResponse>(
      'get_template_ranking'
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

  function fmtScore(r: number): string {
    return r.toFixed(3);
  }

  function fmtDelta(d: number): string {
    const s = d.toFixed(3);
    return d > 0 ? `+${s}` : s;
  }

  // The CLI sort is the canonical "ranking" surface; the drawer re-sorts the
  // returned rows by smoothed utility DESC so the panel reads top-down highest
  // utility first regardless of SSB seq order.
  let ranked = $derived<TemplateRankingRow[]>(
    resp ? [...resp.rows].sort((a, b) => b.utility - a.utility) : []
  );

  let totalRanked = $derived(ranked.length);

  // Count of templates whose Laplace-smoothed utility came out BELOW their raw
  // first_try_ratio -- these are the low-sample templates F-7 stopped
  // over-trusting (the whole point of the change).
  let distrustedCount = $derived(
    ranked.filter((row) => row.utility < row.first_try_ratio).length
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

<section class="template-ranking-panel">
  <header class="panel-header">
    <h2>Template Ranking (F-7)</h2>
    <button class="reload-btn" onclick={reload}>Reload</button>
  </header>

  {#if !loaded}
    <p class="muted">Loading template-ranking rows…</p>
  {:else if loadError}
    <div class="error-banner">
      <strong>Failed to load template-ranking rows.</strong>
      <pre>{loadError}</pre>
    </div>
  {:else if resp === null}
    <p class="muted">
      Tauri IPC unavailable (running in browser preview mode). Mount the
      dashboard in the Tauri host to see live template-ranking data.
    </p>
  {:else}
    <div class="summary-row">
      <span class="summary-label">Ranked:</span>
      <span class="summary-value">{totalRanked}</span>
      {#if distrustedCount > 0}
        <span class="badge badge-distrust">{distrustedCount} smoothed-down</span>
      {/if}
    </div>

    {#if ranked.length === 0}
      <p class="muted">
        No <code>template_utility_ranked</code> rows recorded yet. Rows are
        emitted at the <code>try_execute</code> gate-threshold seam and by the
        <code>elai template-ranking list</code> CLI when a crystallized-pipeline
        template's Laplace-smoothed outcome utility is computed. A 1/1 template
        scores utility ≈ 0.667 while its first-try ratio is 1.0; the two
        converge as the sample size grows.
      </p>
    {:else}
      <table>
        <thead>
          <tr>
            <th>Time</th>
            <th>Concept</th>
            <th class="num">Utility</th>
            <th class="num">First-try</th>
            <th class="num">Δ</th>
            <th class="num">Samples</th>
            <th>Pipeline</th>
          </tr>
        </thead>
        <tbody>
          {#each ranked as row (row.seq)}
            <tr class="row-template">
              <td>{fmtTs(row.ts)}</td>
              <td class="concept-cell" title={row.concept_class}>{row.concept_class}</td>
              <td class="num">
                <span class="badge badge-utility">{fmtScore(row.utility)}</span>
              </td>
              <td class="num">
                <span class="badge badge-baseline">{fmtScore(row.first_try_ratio)}</span>
              </td>
              <td class="num">
                {#if row.utility < row.first_try_ratio}
                  <span class="badge badge-delta-down">{fmtDelta(row.utility - row.first_try_ratio)}</span>
                {:else}
                  <span class="muted">{fmtDelta(row.utility - row.first_try_ratio)}</span>
                {/if}
              </td>
              <td class="num">{row.sample_n}</td>
              <td class="pipeline-cell" title={row.chosen_pipeline_id}>{row.chosen_pipeline_id}</td>
            </tr>
          {/each}
        </tbody>
      </table>
    {/if}
  {/if}
</section>

<style>
  .template-ranking-panel {
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
    font-size: 10px;
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
    font-size: 10px;
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

  th.num,
  td.num {
    text-align: right;
    font-variant-numeric: tabular-nums;
  }

  th {
    font-weight: 500;
    color: var(--text-secondary);
    letter-spacing: 0.04em;
    text-transform: uppercase;
    font-size: 10px;
  }

  .row-template {
    background: rgba(150, 110, 220, 0.05);
  }

  .concept-cell,
  .pipeline-cell {
    max-width: 160px;
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
    background: var(--bg-hover);
    border: 1px solid var(--border-subtle);
    font-variant-numeric: tabular-nums;
  }

  .badge-utility {
    background: rgba(80, 200, 150, 0.15);
    border-color: rgba(80, 200, 150, 0.4);
  }

  .badge-baseline {
    background: rgba(120, 140, 180, 0.12);
    border-color: rgba(120, 140, 180, 0.35);
  }

  .badge-delta-down {
    background: rgba(220, 160, 60, 0.15);
    border-color: rgba(220, 160, 60, 0.4);
    text-transform: none;
  }

  .badge-distrust {
    background: rgba(220, 160, 60, 0.15);
    border-color: rgba(220, 160, 60, 0.4);
    text-transform: uppercase;
    padding: 1px 6px;
    border-radius: var(--r-xs);
    font-size: 10px;
    letter-spacing: 0.04em;
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
