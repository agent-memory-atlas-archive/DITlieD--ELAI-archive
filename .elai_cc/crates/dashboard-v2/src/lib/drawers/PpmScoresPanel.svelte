<script lang="ts">
  // D-2 — PpmScoresPanel.
  //
  // Read-only view of recent PPM (Process Preference Model) verifier
  // rows via the `get_ppm_recent_scores` Tauri command (additive — see
  // crates/dashboard-v2/src-tauri/src/commands_d2.rs).
  //
  // Renders:
  //   * last 50 PPM rows (mix of `scored` + `gating_failed`), newest first
  //   * per-row kind badge + mode badge (math / code)
  //   * truncated candidate_id + verifier_id (12 hex chars)
  //   * score_mean with threshold delta (when `gating_failed`)
  //   * cache_hit indicator for scored rows
  //
  // No producer logic lives here — the orchestrator producer
  // (`ppm_client::emit_ppm_score`) emits the SSB rows separately. This
  // panel polls on a 10s cadence.

  import { onMount, onDestroy } from 'svelte';

  type PpmScoreRow = {
    seq: number;
    ts: number;
    kind: 'scored' | 'gating_failed';
    candidate_id: string;
    verifier_id: string;
    mode: string;
    score_mean: number;
    score_var: number | null;
    num_steps: number;
    threshold: number | null;
    latency_ms: number | null;
    cache_hit: boolean | null;
  };

  let rows = $state<PpmScoreRow[]>([]);
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
      console.warn(`[ppm-scores] invoke('${cmd}') failed`, e);
      return undefined;
    }
  }

  async function reload() {
    const out = await callInvoke<PpmScoreRow[]>('get_ppm_recent_scores', { limit: 50 });
    if (out === undefined) {
      rows = [];
      loadError = null;
    } else {
      rows = out;
      loadError = null;
    }
    loaded = true;
  }

  function shortId(s: string): string {
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

  function fmtScore(n: number): string {
    return n.toFixed(3);
  }

  function fmtDelta(score: number, threshold: number | null): string {
    if (threshold === null || threshold === undefined) return '';
    const delta = score - threshold;
    const sign = delta >= 0 ? '+' : '';
    return `${sign}${delta.toFixed(3)}`;
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

<section class="ppm-scores-panel">
  <header class="panel-header">
    <h2>PPM Verifier Scores</h2>
    <button class="reload-btn" onclick={reload}>Reload</button>
  </header>

  {#if !loaded}
    <p class="muted">Loading PPM scores…</p>
  {:else if loadError}
    <div class="error-banner">
      <strong>Failed to load PPM scores.</strong>
      <pre>{loadError}</pre>
    </div>
  {:else if rows.length === 0}
    <p class="muted">
      No PPM scores recorded yet. PPM rows are emitted by the
      orchestrator after the second sidecar is booted (port 8101) and
      <code>ELAI_PPM_ENABLE=1</code> is set.
    </p>
  {:else}
    <table>
      <thead>
        <tr>
          <th>Time</th>
          <th>Kind</th>
          <th>Candidate</th>
          <th>Verifier</th>
          <th>Mode</th>
          <th>Score</th>
          <th>Var</th>
          <th>Steps</th>
          <th>Threshold</th>
          <th>Latency</th>
          <th>Cache</th>
        </tr>
      </thead>
      <tbody>
        {#each rows as row (row.seq)}
          <tr class="row-{row.kind}">
            <td>{fmtTs(row.ts)}</td>
            <td><span class="badge badge-kind-{row.kind}">{row.kind}</span></td>
            <td><code>{shortId(row.candidate_id)}</code></td>
            <td><code>{shortId(row.verifier_id)}</code></td>
            <td><span class="badge badge-mode">{row.mode}</span></td>
            <td><code>{fmtScore(row.score_mean)}</code></td>
            <td>{row.score_var === null ? '—' : fmtScore(row.score_var)}</td>
            <td>{row.num_steps}</td>
            <td>
              {#if row.threshold !== null}
                <code>{fmtScore(row.threshold)}</code>
                <span class="delta">({fmtDelta(row.score_mean, row.threshold)})</span>
              {:else}
                —
              {/if}
            </td>
            <td>{row.latency_ms === null ? '—' : `${row.latency_ms} ms`}</td>
            <td>
              {#if row.cache_hit === true}
                <span class="badge badge-cache-hit">hit</span>
              {:else if row.cache_hit === false}
                <span class="badge badge-cache-miss">miss</span>
              {:else}
                —
              {/if}
            </td>
          </tr>
        {/each}
      </tbody>
    </table>
  {/if}
</section>

<style>
  .ppm-scores-panel {
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

  .row-gating_failed {
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

  .badge-kind-scored {
    background: rgba(80, 200, 120, 0.15);
    border-color: rgba(80, 200, 120, 0.4);
  }

  .badge-kind-gating_failed {
    background: rgba(220, 80, 80, 0.15);
    border-color: rgba(220, 80, 80, 0.4);
  }

  .badge-mode {
    background: var(--bg-surface);
  }

  .badge-cache-hit {
    background: rgba(80, 140, 220, 0.15);
    border-color: rgba(80, 140, 220, 0.4);
  }

  .badge-cache-miss {
    background: var(--bg-surface);
  }

  .delta {
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
