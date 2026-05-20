<script lang="ts">
  // D-19 — PrmBenchPanel.
  //
  // Read-only view of recent PRMBench (adversarial-gaming benchmark)
  // runs via the `get_prmbench_recent` Tauri command (additive — see
  // crates/dashboard-v2/src-tauri/src/commands_d19.rs).
  //
  // Renders:
  //   * last 50 PRMBench runs, newest first
  //   * accuracy / precision / recall (3 dp) per run
  //   * gating verdict badge (pass / fail)
  //   * truncated fixture blake3 (12 hex chars)
  //   * total fixture rows + category count per run
  //
  // No producer logic lives here — the orchestrator producer
  // (`PRMBenchRunner::run_all`) emits the SSB rows separately. This
  // panel polls on a 10s cadence.

  import { onMount, onDestroy } from 'svelte';

  type PrmBenchRow = {
    seq: number;
    ts: number;
    total: number;
    accuracy: number;
    precision: number;
    recall: number;
    gating_verdict: string;
    fixture_blake3: string;
    category_count: number;
  };

  let rows = $state<PrmBenchRow[]>([]);
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
      console.warn(`[prmbench] invoke('${cmd}') failed`, e);
      return undefined;
    }
  }

  async function reload() {
    const out = await callInvoke<PrmBenchRow[]>('get_prmbench_recent', { limit: 50 });
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

<section class="prmbench-panel">
  <header class="panel-header">
    <h2>PRMBench Runs</h2>
    <button class="reload-btn" onclick={reload}>Reload</button>
  </header>

  {#if !loaded}
    <p class="muted">Loading PRMBench runs…</p>
  {:else if loadError}
    <div class="error-banner">
      <strong>Failed to load PRMBench runs.</strong>
      <pre>{loadError}</pre>
    </div>
  {:else if rows.length === 0}
    <p class="muted">
      No PRMBench runs recorded yet. Rows are emitted by the orchestrator
      after <code>elai bench prmbench</code> completes, or after
      <code>PRMBenchRunner::run_all</code> is invoked from a task_runner.
    </p>
  {:else}
    <table>
      <thead>
        <tr>
          <th>Time</th>
          <th>Verdict</th>
          <th>Total</th>
          <th>Accuracy</th>
          <th>Precision</th>
          <th>Recall</th>
          <th>Cats</th>
          <th>Fixture</th>
        </tr>
      </thead>
      <tbody>
        {#each rows as row (row.seq)}
          <tr class="row-{row.gating_verdict}">
            <td>{fmtTs(row.ts)}</td>
            <td><span class="badge badge-verdict-{row.gating_verdict}">{row.gating_verdict}</span></td>
            <td>{row.total}</td>
            <td><code>{fmtScore(row.accuracy)}</code></td>
            <td><code>{fmtScore(row.precision)}</code></td>
            <td><code>{fmtScore(row.recall)}</code></td>
            <td>{row.category_count}</td>
            <td><code>{shortHash(row.fixture_blake3)}</code></td>
          </tr>
        {/each}
      </tbody>
    </table>
  {/if}
</section>

<style>
  .prmbench-panel {
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
