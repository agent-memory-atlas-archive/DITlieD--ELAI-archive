<script lang="ts">
  // I-38 (SCHEMA_VERSION 235) -- QdeltaPanel.
  //
  // Read-only view of the most recent qdelta_measured SSB flat-beacon row via
  // the `get_qdelta_latest` Tauri command
  // (see crates/dashboard-v2/src-tauri/src/commands_i38.rs).
  //
  // Surfaces QDELTA (off_defective_rate - on_defective_rate), paired 95%
  // bootstrap CI, on/off rates, corpus_blake3, contamination probe result,
  // and runpod_pending status.
  //
  // HONESTY (D-026): Tier-1 runs the embedded fixture corpus only. The
  // live-model dynamic leg is RunPod-gated. The panel renders
  // "RUNPOD-PENDING" when runpod_pending is true so no live claim is implied.
  // surviving_mutants_per_task is 0.0 (I-20 stub) until I-20 ships.
  //
  // cross-lang-pair: dashboard/tauri/api

  import { onMount, onDestroy } from 'svelte';
  import type { QdeltaLatest } from '../api/types';

  let row = $state<QdeltaLatest | null>(null);
  let loaded = $state(false);
  let loadError = $state<string | null>(null);
  let pollHandle: ReturnType<typeof setInterval> | null = null;

  async function callInvoke<T>(
    cmd: string,
    args?: Record<string, unknown>
  ): Promise<T> {
    const w = window as unknown as {
      __TAURI__?: { invoke?: (cmd: string, args?: unknown) => Promise<T> };
    };
    const invoke = w.__TAURI__?.invoke;
    if (!invoke) throw new Error('Tauri invoke not available');
    return invoke(cmd, args);
  }

  async function load() {
    try {
      const result = await callInvoke<QdeltaLatest | null>('get_qdelta_latest');
      row = result;
      loadError = null;
    } catch (e) {
      loadError = String(e);
    } finally {
      loaded = true;
    }
  }

  onMount(() => {
    load();
    pollHandle = setInterval(load, 30_000);
  });

  onDestroy(() => {
    if (pollHandle !== null) clearInterval(pollHandle);
  });

  function formatTs(ts: number): string {
    return new Date(ts).toLocaleString();
  }

  function pct(v: number): string {
    return (v * 100).toFixed(2) + '%';
  }
</script>

<div class="qdelta-panel">
  <h3>QDELTA — Defective-Diff Rate (I-38)</h3>
  <p class="honesty-note">
    QDELTA = gate-OFF defective-diff rate minus gate-ON defective-diff rate (positive = gates reduce defects).
    Tier-1 fixture corpus only; live-model leg is RunPod-gated (D-026).
    surviving_mutants_per_task = 0.0 until I-20 ships.
  </p>

  {#if !loaded}
    <p class="loading">Loading&hellip;</p>
  {:else if loadError}
    <p class="error">Error: {loadError}</p>
  {:else if row === null}
    <p class="empty">No qdelta_measured rows yet. Run <code>elai bench qdelta --config &lt;PATH&gt;</code> to populate.</p>
  {:else}
    <div class="summary-card">
      <div class="metric primary">
        <span class="label">QDELTA</span>
        <span class="value">{pct(row.delta)}</span>
        <span class="ci">[{pct(row.ci_low)}, {pct(row.ci_high)}] 95% CI (paired bootstrap)</span>
        {#if row.runpod_pending}
          <span class="pending-badge">RUNPOD-PENDING — Tier-1 fixture only (D-026)</span>
        {/if}
      </div>

      <div class="metric-row">
        <div class="metric">
          <span class="label">On rate</span>
          <span class="value">{pct(row.on_defective_rate)}</span>
        </div>
        <div class="metric">
          <span class="label">Off rate</span>
          <span class="value">{pct(row.off_defective_rate)}</span>
        </div>
        <div class="metric">
          <span class="label">N tasks</span>
          <span class="value">{row.n_tasks}</span>
        </div>
        <div class="metric">
          <span class="label">Seeds</span>
          <span class="value">{row.seed_count}</span>
        </div>
      </div>

      <div class="metric-row">
        <div class="metric">
          <span class="label">Contamination probe</span>
          <span class="value probe">{row.contamination_probe_result}</span>
        </div>
        <div class="metric">
          <span class="label">Mutants / task</span>
          <span class="value">{row.surviving_mutants_per_task.toFixed(3)}</span>
        </div>
        <div class="metric">
          <span class="label">$/task</span>
          <span class="value">{row.dollar_per_task.toFixed(4)}</span>
        </div>
      </div>

      <div class="meta">
        <span class="label">Suite</span>
        <span class="value">{row.suite}</span>
        &nbsp;&bull;&nbsp;
        <span class="label">Corpus blake3</span>
        <span class="value mono">{row.corpus_blake3.substring(0, 16)}&hellip;</span>
        &nbsp;&bull;&nbsp;
        <span class="ts">{formatTs(row.ts)}</span>
      </div>
    </div>
  {/if}
</div>

<style>
  .qdelta-panel {
    padding: 1rem;
    font-family: inherit;
  }
  h3 {
    margin: 0 0 0.5rem;
    font-size: 1.1rem;
  }
  .honesty-note {
    font-size: 0.75rem;
    color: #888;
    margin: 0 0 1rem;
    font-style: italic;
  }
  .loading, .empty, .error {
    color: #888;
    font-size: 0.9rem;
  }
  .error {
    color: #c44;
  }
  .summary-card {
    border: 1px solid #333;
    border-radius: 6px;
    padding: 0.75rem;
    display: flex;
    flex-direction: column;
    gap: 0.5rem;
  }
  .metric.primary {
    display: flex;
    flex-direction: column;
    gap: 0.2rem;
  }
  .metric-row {
    display: flex;
    gap: 1.5rem;
    flex-wrap: wrap;
  }
  .metric {
    display: flex;
    flex-direction: column;
  }
  .label {
    font-size: 0.7rem;
    color: #888;
    text-transform: uppercase;
    letter-spacing: 0.05em;
  }
  .value {
    font-size: 1.1rem;
    font-weight: 600;
  }
  .probe {
    font-size: 0.85rem;
    font-weight: 500;
  }
  .ci {
    font-size: 0.75rem;
    color: #aaa;
  }
  .pending-badge {
    display: inline-block;
    font-size: 0.7rem;
    background: #3a2800;
    color: #ffaa00;
    border-radius: 4px;
    padding: 0.1rem 0.4rem;
    margin-top: 0.2rem;
    font-weight: 600;
  }
  .meta {
    font-size: 0.75rem;
    color: #888;
  }
  .mono {
    font-family: monospace;
  }
  .ts {
    color: #666;
  }
</style>
