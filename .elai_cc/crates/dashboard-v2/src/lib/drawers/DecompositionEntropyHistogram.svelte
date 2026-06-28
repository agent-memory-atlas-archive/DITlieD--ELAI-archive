<script lang="ts">
  // E-46 D-7 — DecompositionEntropyHistogram.
  //
  // Read-only histogram of `low_entropy_decomposition` SSB rows via the
  // `get_entropy_history` Tauri command (commands_e46.rs::get_entropy_history).
  //
  // Renders:
  //   * last 50 low_entropy_decomposition rows sorted most-recent first
  //   * per-row entropy_ratio bar (visual relative to threshold)
  //   * h_parent / h_children / threshold scalars
  //
  // Wire-dark unless ELAI_PLANNING_DECOMPOSITION_ENABLE=1 on the orchestrator.
  // Panel renders an informative empty state when no rows exist.
  // Polls at 15s cadence matching the planning-ledger panel convention.

  import { onMount, onDestroy } from 'svelte';

  type EntropyHistogramRow = {
    parent_node_key: string;
    entropy_ratio: number;
    h_parent: number;
    h_children: number;
    threshold: number;
    ts_ms: number;
  };

  let rows = $state<EntropyHistogramRow[]>([]);
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
      console.warn(`[decomposition-entropy] invoke('${cmd}') failed`, e);
      return undefined;
    }
  }

  async function reload() {
    const out = await callInvoke<EntropyHistogramRow[]>('get_entropy_history', { limit: 50 });
    if (out === undefined) {
      loadError = null;
    } else {
      rows = out;
      loadError = null;
    }
    loaded = true;
  }

  function shortKey(s: string | null | undefined): string {
    if (!s) return '—';
    if (s.length <= 20) return s;
    return `${s.slice(0, 12)}…${s.slice(-6)}`;
  }

  function fmtRatio(n: number): string {
    if (!Number.isFinite(n)) return '—';
    return (n * 100).toFixed(1) + '%';
  }

  function fmtH(n: number): string {
    if (!Number.isFinite(n)) return '—';
    return n.toFixed(3);
  }

  function barWidth(ratio: number, threshold: number): number {
    // Bar is expressed as percentage of the container width.
    // Red when ratio < threshold, amber when ratio >= threshold but close,
    // green when ratio >= 1.0. Clamp [0, 100].
    return Math.min(100, Math.max(0, ratio * 100));
  }

  function barClass(ratio: number, threshold: number): string {
    if (ratio < threshold) return 'bar-low';
    if (ratio < 0.85) return 'bar-warn';
    return 'bar-ok';
  }

  onMount(() => {
    reload();
    pollHandle = setInterval(reload, 15_000);
  });

  onDestroy(() => {
    if (pollHandle) {
      clearInterval(pollHandle);
      pollHandle = null;
    }
  });
</script>

<section class="entropy-histogram-panel">
  <header class="panel-header">
    <h2>Decomposition Entropy</h2>
    <button class="reload-btn" onclick={reload}>Reload</button>
  </header>

  {#if !loaded}
    <p class="muted">Loading entropy history…</p>
  {:else if loadError}
    <div class="error-banner">
      <strong>Failed to load entropy rows.</strong>
      <pre>{loadError}</pre>
    </div>
  {:else if rows.length === 0}
    <p class="muted">
      No low_entropy_decomposition rows recorded yet. Rows are emitted by the
      orchestrator when the entropy ratio H_children/H_parent falls below the
      threshold. Enable via <code>ELAI_PLANNING_DECOMPOSITION_ENABLE=1</code>.
    </p>
  {:else}
    <table>
      <thead>
        <tr>
          <th>Node</th>
          <th>Ratio</th>
          <th>Bar</th>
          <th>H_par</th>
          <th>H_chi</th>
          <th>Thresh</th>
        </tr>
      </thead>
      <tbody>
        {#each rows as row (row.ts_ms + row.parent_node_key)}
          <tr>
            <td><code>{shortKey(row.parent_node_key)}</code></td>
            <td class:ratio-low={row.entropy_ratio < row.threshold}>{fmtRatio(row.entropy_ratio)}</td>
            <td class="bar-cell">
              <div
                class="bar-fill {barClass(row.entropy_ratio, row.threshold)}"
                style="width:{barWidth(row.entropy_ratio, row.threshold)}%"
              ></div>
            </td>
            <td>{fmtH(row.h_parent)}</td>
            <td>{fmtH(row.h_children)}</td>
            <td>{fmtH(row.threshold)}</td>
          </tr>
        {/each}
      </tbody>
    </table>
  {/if}
</section>

<style>
  .entropy-histogram-panel {
    display: flex;
    flex-direction: column;
    gap: 12px;
    padding: 16px;
    font-size: 13px;
  }

  .panel-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
  }

  .panel-header h2 {
    font-size: 15px;
    font-weight: 600;
    margin: 0;
  }

  .reload-btn {
    padding: 4px 10px;
    font-size: 12px;
    cursor: pointer;
    border-radius: 4px;
    border: 1px solid var(--border-color, #ccc);
    background: var(--btn-bg, transparent);
    color: inherit;
  }

  .muted {
    color: var(--text-muted, #888);
    font-size: 12px;
  }

  .error-banner {
    background: var(--error-bg, #ffeaea);
    border: 1px solid var(--error-border, #f99);
    padding: 8px;
    border-radius: 4px;
  }

  table {
    border-collapse: collapse;
    width: 100%;
  }

  th,
  td {
    padding: 4px 8px;
    text-align: left;
    border-bottom: 1px solid var(--border-light, #eee);
    white-space: nowrap;
  }

  th {
    font-weight: 600;
    color: var(--text-muted, #888);
    font-size: 10px;
    text-transform: uppercase;
  }

  .ratio-low {
    color: var(--error-color, #c0392b);
    font-weight: 600;
  }

  .bar-cell {
    width: 80px;
    padding: 2px 8px;
  }

  .bar-fill {
    height: 10px;
    border-radius: 3px;
    min-width: 2px;
  }

  .bar-low {
    background: var(--error-color, #c0392b);
  }

  .bar-warn {
    background: var(--warn-color, #e67e22);
  }

  .bar-ok {
    background: var(--ok-color, #27ae60);
  }
</style>
