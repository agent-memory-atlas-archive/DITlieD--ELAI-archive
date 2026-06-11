<script lang="ts">
  // G-6 (SCHEMA_VERSION 217) -- SafeDeltaPanel.
  //
  // Read-only view of the most recent safety_delta_measured SSB row via the
  // `get_safety_delta_latest` Tauri command
  // (see crates/dashboard-v2/src-tauri/src/commands_g6.rs).
  //
  // Surfaces the derived SAFEDELTA (gate-ON-vs-OFF marginal residual rate)
  // + a seeded bootstrap 95% CI + per-CWE delta + corpus_blake3 + gate_recall
  // + runpod_pending status.
  //
  // HONESTY (GD-5): Tier-1 runs the static-gate ablation over the embedded
  // SeCodePLT corpus only. The Tier-2 live dynamic exploit-exec arm
  // (BaxBench-392 arXiv:2502.11844, SUSVIBES-200 arXiv:2512.03262) is
  // separately RunPod-gated. The panel explicitly renders "RUNPOD-PENDING"
  // when runpod_pending is true so no live claim is implied.
  //
  // cross-lang-pair: dashboard/tauri/api

  import { onMount, onDestroy } from 'svelte';
  import type { SafetyDeltaLatest } from '../api/types';

  let row = $state<SafetyDeltaLatest | null>(null);
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
      const result = await callInvoke<SafetyDeltaLatest | null>(
        'get_safety_delta_latest'
      );
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

<div class="safe-delta-panel">
  <h3>Safety Delta (G-6)</h3>
  <p class="honesty-note">
    SAFEDELTA = gate-ON vs gate-OFF marginal insecure-rate (static gate, embedded SeCodePLT corpus).
    DISTINCT from SeCodePLT dynamic secure-coding rate (sandbox + live model, RunPod-gated).
    Suite: SeCodePLT arXiv:2410.11096. Cite only arXiv:2404.13161 for insecure-code-gen.
  </p>

  {#if !loaded}
    <p class="loading">Loading&hellip;</p>
  {:else if loadError}
    <p class="error">Error: {loadError}</p>
  {:else if row === null}
    <p class="empty">No safety_delta_measured rows yet. Run <code>elai bench safedelta</code> to populate.</p>
  {:else}
    <div class="summary-card">
      <div class="metric primary">
        <span class="label">SAFEDELTA</span>
        <span class="value">{pct(row.safety_delta)}</span>
        <span class="ci">[{pct(row.ci_low)}, {pct(row.ci_high)}] 95% CI (seeded bootstrap)</span>
        {#if row.runpod_pending}
          <span class="pending-badge">RUNPOD-PENDING — Tier-1 static-gate only</span>
        {/if}
      </div>

      <div class="metric-row">
        <div class="metric">
          <span class="label">Gate recall</span>
          <span class="value">{pct(row.gate_recall)}</span>
        </div>
        <div class="metric">
          <span class="label">N tasks</span>
          <span class="value">{row.n_tasks}</span>
        </div>
        <div class="metric">
          <span class="label">Seed</span>
          <span class="value">{row.seed}</span>
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
  .safe-delta-panel {
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
