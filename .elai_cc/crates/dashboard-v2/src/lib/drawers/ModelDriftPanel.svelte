<script lang="ts">
  // G-10 (SCHEMA_VERSION 218) -- ModelDriftPanel.
  //
  // Read-only view of the most recent model_drift_suspected SSB row via the
  // `get_model_drift_status` Tauri command
  // (see crates/dashboard-v2/src-tauri/src/commands_g10.rs).
  //
  // Surfaces the exogenous model-version behavioral-regression detection:
  //   * identity_key (blake3 of the model_pin + fingerprint + lane tuple)
  //   * drift kind (fingerprint_changed / canary_regressed / malformed_output_spike)
  //   * per-family regression deltas + affected template domain labels
  //   * fingerprint-confirmed/behavioral badge (GD-5 honesty)
  //
  // HONESTY (GD-5): cloud-lane rows always have confirmed_by_fingerprint=false
  // (Anthropic's system_fingerprint is opaque). The panel renders a
  // "FINGERPRINT-CONFIRMED" badge only for local-lane rows where the
  // fingerprint change was directly observed. Cloud rows render "BEHAVIORAL-SUSPECTED".
  //
  // SURFACE not auto-swap (DD5): this panel shows the drift signal + affected
  // template domain labels for reviewer routing. Auto-pin to last-good is a
  // separate human-gated follow-on (not in the done-gate).
  //
  // cross-lang-pair: dashboard/tauri/api

  import { onMount, onDestroy } from 'svelte';
  import type { ModelDriftStatus } from '../api/types';

  let row = $state<ModelDriftStatus | null>(null);
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
      const result = await callInvoke<ModelDriftStatus | null>(
        'get_model_drift_status'
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

  function kindLabel(kind: string): string {
    switch (kind) {
      case 'fingerprint_changed': return 'Fingerprint Changed';
      case 'canary_regressed': return 'Canary Regressed';
      case 'malformed_output_spike': return 'Malformed Output Spike';
      default: return kind;
    }
  }
</script>

<div class="model-drift-panel">
  <h3>Model Drift (G-10)</h3>
  <p class="honesty-note">
    Exogenous model-version behavioral-regression gate. Composes D-7 canary (GD-1).
    Cloud lane = behavioral-suspected only (Anthropic fingerprint is opaque, GD-5).
    SURFACE only — auto-pin is a separate human-gated follow-on (DD5).
  </p>

  {#if !loaded}
    <p class="loading">Loading&hellip;</p>
  {:else if loadError}
    <p class="error">Error: {loadError}</p>
  {:else if row === null}
    <p class="empty">
      No model_drift_suspected rows yet.
      Run <code>elai model-drift check</code> or wait for a <code>elai run</code> to detect drift.
    </p>
  {:else}
    <div class="summary-card">
      <div class="metric primary">
        <span class="label">Drift Kind</span>
        <span class="value">{kindLabel(row.kind)}</span>

        {#if row.confirmed_by_fingerprint}
          <span class="badge confirmed">FINGERPRINT-CONFIRMED (local lane)</span>
        {:else}
          <span class="badge behavioral">BEHAVIORAL-SUSPECTED (cloud or no fp change)</span>
        {/if}
      </div>

      <div class="metric-row">
        <div class="metric">
          <span class="label">Identity Key</span>
          <span class="value mono">{row.identity_key.substring(0, 12)}&hellip;</span>
        </div>
        <div class="metric">
          <span class="label">Baseline Captured</span>
          <span class="value">{formatTs(row.baseline_captured_at)}</span>
        </div>
        <div class="metric">
          <span class="label">Detected</span>
          <span class="value">{formatTs(row.ts)}</span>
        </div>
      </div>

      {#if row.per_family_delta.length > 0}
        <div class="family-table">
          <span class="label">Per-Family Delta</span>
          <table>
            <thead>
              <tr><th>Family</th><th>Delta</th></tr>
            </thead>
            <tbody>
              {#each row.per_family_delta as [family, delta]}
                <tr class={delta <= -0.05 ? 'regressed' : ''}>
                  <td>{family}</td>
                  <td>{pct(delta)}</td>
                </tr>
              {/each}
            </tbody>
          </table>
        </div>
      {/if}

      {#if row.affected_template_ids.length > 0}
        <div class="meta">
          <span class="label">Affected Templates (review surface)</span>
          <span class="value">{row.affected_template_ids.join(', ')}</span>
        </div>
      {/if}

      {#if row.signals.length > 0}
        <div class="meta">
          <span class="label">Signals</span>
          <span class="value">{row.signals.join(', ')}</span>
        </div>
      {/if}
    </div>
  {/if}
</div>

<style>
  .model-drift-panel {
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
    font-size: 1.0rem;
    font-weight: 600;
  }
  .badge {
    display: inline-block;
    font-size: 0.7rem;
    border-radius: 4px;
    padding: 0.1rem 0.4rem;
    margin-top: 0.2rem;
    font-weight: 600;
  }
  .badge.confirmed {
    background: #003a00;
    color: #00ff88;
  }
  .badge.behavioral {
    background: #3a2800;
    color: #ffaa00;
  }
  .family-table {
    display: flex;
    flex-direction: column;
    gap: 0.25rem;
  }
  table {
    border-collapse: collapse;
    font-size: 0.85rem;
    width: 100%;
  }
  th, td {
    text-align: left;
    padding: 0.2rem 0.5rem;
    border: 1px solid #333;
  }
  th {
    background: #1a1a1a;
    color: #888;
    font-size: 0.7rem;
    text-transform: uppercase;
  }
  tr.regressed {
    color: #ff6666;
  }
  .meta {
    font-size: 0.75rem;
    color: #888;
  }
  .mono {
    font-family: monospace;
  }
</style>
