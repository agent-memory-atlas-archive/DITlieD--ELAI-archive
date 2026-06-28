<script lang="ts">
  // D-5 — BaiArmsPanel.
  //
  // Read-only view of recent BAI (Best-Arm Identification) Thompson-sampling
  // arm-update rows via the `get_bai_arms` Tauri command (additive — see
  // crates/dashboard-v2/src-tauri/src/commands_d5.rs).
  //
  // Renders:
  //   * last 50 bai_arm_updated rows, sorted by mean_estimate descending
  //   * per-row Beta(alpha, beta) posterior + n_pulls + posterior mean
  //   * promotion flag as a green/red badge
  //
  // No producer logic lives here — the orchestrator producer
  // (`bai::emit_bai_round`) emits the SSB rows separately. This panel
  // polls on a 10s cadence matching the other D-{1..4} panels.

  import { onMount, onDestroy } from 'svelte';

  type BaiArmRow = {
    template_id: string;
    alpha: number;
    beta: number;
    n_pulls: number;
    was_promoted: boolean;
    mean_estimate: number;
  };

  let rows = $state<BaiArmRow[]>([]);
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
      console.warn(`[bai-arms] invoke('${cmd}') failed`, e);
      return undefined;
    }
  }

  async function reload() {
    const out = await callInvoke<BaiArmRow[]>('get_bai_arms', { limit: 50 });
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

  function fmtMean(n: number | null | undefined): string {
    if (n === null || n === undefined) return '—';
    if (!Number.isFinite(n)) return String(n);
    return n.toFixed(3);
  }

  function fmtBeta(n: number | null | undefined): string {
    if (n === null || n === undefined) return '—';
    if (!Number.isFinite(n)) return String(n);
    // Posterior counts are integer-valued in the Bernoulli case but f64
    // on the wire (callers can feed soft rewards). 2 dp keeps the column
    // narrow without losing soft-reward fidelity.
    return n.toFixed(2);
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

<section class="bai-arms-panel">
  <header class="panel-header">
    <h2>BAI Arms</h2>
    <button class="reload-btn" onclick={reload}>Reload</button>
  </header>

  {#if !loaded}
    <p class="muted">Loading BAI arm rows…</p>
  {:else if loadError}
    <div class="error-banner">
      <strong>Failed to load BAI arm rows.</strong>
      <pre>{loadError}</pre>
    </div>
  {:else if rows.length === 0}
    <p class="muted">
      No BAI arm rows recorded yet. Rows are emitted by the orchestrator
      after <code>bai::emit_bai_round</code> updates a candidate's
      Beta(alpha, beta) posterior and runs Thompson sampling.
    </p>
  {:else}
    <table>
      <thead>
        <tr>
          <th>Template</th>
          <th>α</th>
          <th>β</th>
          <th>n</th>
          <th>Mean</th>
          <th>Promoted</th>
        </tr>
      </thead>
      <tbody>
        {#each rows as row (row.template_id)}
          <tr class:row-promoted={row.was_promoted}>
            <td><code>{shortId(row.template_id)}</code></td>
            <td>{fmtBeta(row.alpha)}</td>
            <td>{fmtBeta(row.beta)}</td>
            <td>{row.n_pulls}</td>
            <td>{fmtMean(row.mean_estimate)}</td>
            <td>
              {#if row.was_promoted}
                <span class="badge badge-promoted">PROMOTED</span>
              {:else}
                <span class="badge badge-held">HELD</span>
              {/if}
            </td>
          </tr>
        {/each}
      </tbody>
    </table>
  {/if}
</section>

<style>
  .bai-arms-panel {
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

  .row-promoted {
    background: rgba(80, 200, 120, 0.07);
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

  .badge-promoted {
    background: rgba(80, 200, 120, 0.18);
    border-color: rgba(80, 200, 120, 0.45);
    color: rgb(120, 220, 150);
  }

  .badge-held {
    background: rgba(180, 180, 180, 0.10);
    border-color: rgba(180, 180, 180, 0.35);
    color: var(--text-secondary);
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
