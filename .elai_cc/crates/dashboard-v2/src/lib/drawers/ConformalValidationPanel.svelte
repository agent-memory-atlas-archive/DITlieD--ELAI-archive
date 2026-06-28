<script lang="ts">
  // D-22 — ConformalValidationPanel.
  //
  // Read-only view of recent conformal calibration validation runs via the
  // `get_conformal_validation_recent` Tauri command (additive — see
  // crates/dashboard-v2/src-tauri/src/commands_d22.rs).
  //
  // The runner replays a held-out fixture (in_domain / near_ood / far_ood
  // buckets) through D-16's `decide` predicate and reports the empirical
  // refusal rate + the UniCR (arXiv:2503.01345) band gate. This panel
  // surfaces:
  //   * last 50 runs, newest first
  //   * tau / epsilon / empirical refusal rate (3 dp) per run
  //   * gating verdict badge (pass / fail)
  //   * truncated fixture blake3 (12 hex chars)
  //   * total rows scored
  //
  // No producer logic lives here — the orchestrator producer
  // (`ConformalValidationRunner::run_all`) emits the SSB rows separately.
  // This panel polls on a 10s cadence; mirrors ModeRoutingBenchPanel.

  import { onMount, onDestroy } from 'svelte';

  type ConformalValidationRow = {
    seq: number;
    ts: number;
    tau: number;
    epsilon: number;
    empirical_refusal_rate: number;
    gating_verdict: string;
    fixture_blake3: string;
    total: number;
  };

  let rows = $state<ConformalValidationRow[]>([]);
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
      console.warn(`[conformal-validation] invoke('${cmd}') failed`, e);
      return undefined;
    }
  }

  async function reload() {
    const out = await callInvoke<ConformalValidationRow[]>(
      'get_conformal_validation_recent',
      { limit: 50 },
    );
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

  function bandLabel(row: ConformalValidationRow): string {
    const lower = Math.max(0, row.epsilon - row.epsilon);
    const upper = Math.min(1, row.epsilon + row.epsilon);
    return `[${lower.toFixed(2)}, ${upper.toFixed(2)}]`;
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

<section class="conformal-validation-panel">
  <header class="panel-header">
    <h2>Conformal Validation</h2>
    <button class="reload-btn" onclick={reload}>Reload</button>
  </header>

  {#if !loaded}
    <p class="muted">Loading conformal validation runs…</p>
  {:else if loadError}
    <div class="error-banner">
      <strong>Failed to load conformal validation runs.</strong>
      <pre>{loadError}</pre>
    </div>
  {:else if rows.length === 0}
    <p class="muted">
      No conformal validation runs recorded yet. Rows are emitted by the
      orchestrator after <code>elai bench conformal</code> completes, or
      after <code>ConformalValidationRunner::run_all</code> is invoked
      from a task_runner.
    </p>
  {:else}
    <div class="legend">
      <span class="legend-label">Buckets:</span>
      in_domain · near_ood · far_ood (per-bucket refusal rate must stay
      within <code>[target ± epsilon]</code>, per UniCR §3).
    </div>
    <table>
      <thead>
        <tr>
          <th>Time</th>
          <th>Verdict</th>
          <th>Tau</th>
          <th>Epsilon</th>
          <th>Empirical</th>
          <th>Band</th>
          <th>Rows</th>
          <th>Fixture</th>
        </tr>
      </thead>
      <tbody>
        {#each rows as row (row.seq)}
          <tr class="row-{row.gating_verdict}">
            <td>{fmtTs(row.ts)}</td>
            <td>
              <span class="badge badge-verdict-{row.gating_verdict}"
                >{row.gating_verdict}</span
              >
            </td>
            <td><code>{fmtScore(row.tau)}</code></td>
            <td><code>{fmtScore(row.epsilon)}</code></td>
            <td><code>{fmtScore(row.empirical_refusal_rate)}</code></td>
            <td><code>{bandLabel(row)}</code></td>
            <td>{row.total}</td>
            <td><code>{shortHash(row.fixture_blake3)}</code></td>
          </tr>
        {/each}
      </tbody>
    </table>
    <p class="footnote">
      Gating: per-bucket empirical refusal rate must satisfy
      <code>|rate − target| ≤ epsilon</code> AND every bucket must contain
      at least <code>min_rows_per_bucket</code> rows (no class collapse).
      The full per-domain breakdown is in the JSON report on disk; this
      panel surfaces the aggregate row so regressions are visible at a
      glance.
    </p>
  {/if}
</section>

<style>
  .conformal-validation-panel {
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

  th,
  td {
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
    border-radius: var(--r-xs);
  }
</style>
