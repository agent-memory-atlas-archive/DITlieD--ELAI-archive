<script lang="ts">
  // D-21 — ModeRoutingBenchPanel.
  //
  // Read-only view of recent mode-routing coverage benchmark runs via the
  // `get_mode_routing_bench_recent` Tauri command (additive — see
  // crates/dashboard-v2/src-tauri/src/commands_d21.rs).
  //
  // The runner replays a hand-labeled fixture through D-10's
  // `select_grounding_mode` predicate and reports macro F1 + per-mode F1
  // + 5x5 confusion matrix. This panel surfaces:
  //   * last 50 runs, newest first
  //   * accuracy / f1_macro (3 dp) per run
  //   * gating verdict badge (pass / fail)
  //   * truncated fixture blake3 (12 hex chars)
  //   * row count
  //
  // No producer logic lives here — the orchestrator producer
  // (`ModeRoutingBenchmark::run_all`) emits the SSB rows separately.
  // This panel polls on a 10s cadence; mirrors PrmBenchPanel structure.

  import { onMount, onDestroy } from 'svelte';

  type ModeRoutingBenchRow = {
    seq: number;
    ts: number;
    total: number;
    accuracy: number;
    f1_macro: number;
    gating_verdict: string;
    fixture_blake3: string;
    fixture_row_count: number;
  };

  let rows = $state<ModeRoutingBenchRow[]>([]);
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
      console.warn(`[mode-routing-bench] invoke('${cmd}') failed`, e);
      return undefined;
    }
  }

  async function reload() {
    const out = await callInvoke<ModeRoutingBenchRow[]>('get_mode_routing_bench_recent', { limit: 50 });
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

  // 5 GroundingMode variants — must match GroundingMode::all() declaration
  // order from crates/orchestrator/src/grounding_mode.rs. Used to label
  // the confusion-matrix grid headers. The grid itself is sourced from
  // the most recent report in the runs list; older runs render only the
  // summary row.
  const MODE_LABELS: string[] = [
    'CON',     // constructive
    'FCF',     // falsifiability_cross_family
    'REF',     // conformal_refusal
    'ODF',     // operational_deferred
    'CFB',     // conservative_fallback
  ];

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

<section class="mode-routing-bench-panel">
  <header class="panel-header">
    <h2>Mode-Routing Bench</h2>
    <button class="reload-btn" onclick={reload}>Reload</button>
  </header>

  {#if !loaded}
    <p class="muted">Loading mode-routing benchmark runs…</p>
  {:else if loadError}
    <div class="error-banner">
      <strong>Failed to load mode-routing benchmark runs.</strong>
      <pre>{loadError}</pre>
    </div>
  {:else if rows.length === 0}
    <p class="muted">
      No mode-routing benchmark runs recorded yet. Rows are emitted by
      the orchestrator after <code>elai bench mode-routing</code> completes,
      or after <code>ModeRoutingBenchmark::run_all</code> is invoked from a
      task_runner.
    </p>
  {:else}
    <div class="legend">
      <span class="legend-label">Modes:</span>
      <code>CON</code> constructive ·
      <code>FCF</code> falsifiability_cross_family ·
      <code>REF</code> conformal_refusal ·
      <code>ODF</code> operational_deferred ·
      <code>CFB</code> conservative_fallback
    </div>
    <table>
      <thead>
        <tr>
          <th>Time</th>
          <th>Verdict</th>
          <th>Total</th>
          <th>Accuracy</th>
          <th>F1 Macro</th>
          <th>Rows</th>
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
            <td><code>{fmtScore(row.f1_macro)}</code></td>
            <td>{row.fixture_row_count}</td>
            <td><code>{shortHash(row.fixture_blake3)}</code></td>
          </tr>
        {/each}
      </tbody>
    </table>
    <p class="footnote">
      Gating: macro F1 ≥ 0.85 AND every per-mode F1 ≥ 0.70 (no class
      collapse). The full per-mode F1 + 5x5 confusion matrix is in the
      JSON report on disk; this panel surfaces the aggregate row so
      regressions are visible at a glance.
    </p>
  {/if}
</section>

<style>
  .mode-routing-bench-panel {
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

  .legend {
    font-size: 10px;
    color: var(--text-secondary);
    letter-spacing: 0.02em;
    line-height: var(--leading-code);
  }

  .legend-label {
    text-transform: uppercase;
    margin-right: 6px;
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

  .footnote {
    color: var(--text-secondary);
    font-size: 10px;
    line-height: var(--leading-text);
  }

  .error-banner {
    background: rgba(220, 80, 80, 0.1);
    border: 1px solid rgba(220, 80, 80, 0.4);
    padding: 8px;
    border-radius: var(--r-sm);
  }
</style>
