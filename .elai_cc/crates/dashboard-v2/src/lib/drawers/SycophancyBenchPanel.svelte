<script lang="ts">
  // D-23 — SycophancyBenchPanel.
  //
  // Read-only view of recent sycophancy detection benchmark runs via the
  // `get_sycophancy_bench_recent` Tauri command (additive — see
  // crates/dashboard-v2/src-tauri/src/commands_d23.rs).
  //
  // Renders:
  //   * last 50 sycophancy bench runs, newest first
  //   * detection_rate / fp_rate (3 dp) per run
  //   * gating verdict badge (pass / fail) with color
  //   * TP / FP / TN / FN / Inconclusive breakdown
  //   * truncated fixture blake3 (12 hex chars)
  //   * runner name + total fixtures per run
  //
  // No producer logic lives here — the orchestrator producer
  // (`SycophancyBenchmark::run`) emits the SSB rows separately. This
  // panel polls on a 10s cadence (sibling of PrmBenchPanel).

  import { onMount, onDestroy } from 'svelte';
  import type { SycophancyBenchRow } from '../api/types';

  let rows = $state<SycophancyBenchRow[]>([]);
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
      console.warn(`[sycophancy-bench] invoke('${cmd}') failed`, e);
      return undefined;
    }
  }

  async function reload() {
    const out = await callInvoke<SycophancyBenchRow[]>('get_sycophancy_bench_recent', { limit: 50 });
    if (out === undefined) {
      rows = [];
      loadError = null;
    } else {
      rows = out;
      loadError = null;
    }
    loaded = true;
  }

  function shortHash(s: string): string {
    if (s.length <= 12) return s;
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

  function fmtScore(n: number): string {
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

<section class="sycophancy-bench-panel">
  <header class="panel-header">
    <h2>Sycophancy Bench Runs</h2>
    <button class="reload-btn" onclick={reload}>Reload</button>
  </header>

  {#if !loaded}
    <p class="muted">Loading sycophancy bench runs…</p>
  {:else if loadError}
    <div class="error-banner">
      <strong>Failed to load sycophancy bench runs.</strong>
      <pre>{loadError}</pre>
    </div>
  {:else if rows.length === 0}
    <p class="muted">
      No sycophancy bench runs recorded yet. Rows are emitted by the
      orchestrator after <code>elai bench sycophancy</code> completes, or
      after <code>SycophancyBenchmark::run</code> is invoked from a
      task_runner. Gates detection_rate &ge; 0.70 AND fp_rate &le; 0.15
      per Sharma §4 (arXiv:2310.13548).
    </p>
  {:else}
    <table>
      <thead>
        <tr>
          <th>Time</th>
          <th>Verdict</th>
          <th>Total</th>
          <th>Detection</th>
          <th>FP rate</th>
          <th>TP / FP</th>
          <th>TN / FN</th>
          <th>Incon.</th>
          <th>Fixture</th>
        </tr>
      </thead>
      <tbody>
        {#each rows as row (row.seq)}
          <tr class="row-{row.verdict}">
            <td>{fmtTs(row.ts)}</td>
            <td><span class="badge badge-verdict-{row.verdict}">{row.verdict}</span></td>
            <td>{row.total_fixtures}</td>
            <td><code>{fmtScore(row.detection_rate)}</code></td>
            <td><code>{fmtScore(row.fp_rate)}</code></td>
            <td><code>{row.tp} / {row.fp}</code></td>
            <td><code>{row.tn} / {row.fn_}</code></td>
            <td>{row.inconclusive}</td>
            <td><code title={row.fixture_blake3}>{shortHash(row.fixture_blake3)}</code></td>
          </tr>
        {/each}
      </tbody>
    </table>
  {/if}
</section>

<style>
  .sycophancy-bench-panel {
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

  .row-fail {
    background: rgba(255, 80, 80, 0.05);
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

  .badge-verdict-pass {
    background: rgba(80, 200, 120, 0.15);
    border-color: rgba(80, 200, 120, 0.4);
  }

  .badge-verdict-fail {
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
