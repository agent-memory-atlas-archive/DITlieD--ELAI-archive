<script lang="ts">
  // D-7 — CanaryPanel.
  //
  // Read-only view of recent cross-domain transfer canary runs via the
  // `get_canary_recent` + `get_canary_per_family_breakdown` Tauri commands
  // (additive — see crates/dashboard-v2/src-tauri/src/commands_d7.rs).
  //
  // Renders:
  //   * last 50 canary_run_completed rows, sorted by ts_ms desc
  //   * per-row decision badge (pass / marginal_pass / fail)
  //   * per-family pass/fail + transfer_score breakdown for each run
  //
  // No producer logic lives here — the orchestrator producer
  // (`canary::emit_canary_run`) emits the SSB rows separately. This panel
  // polls on a 10s cadence matching the other D-{1..6} panels.

  import { onMount, onDestroy } from 'svelte';

  type CanaryRunRow = {
    report_id: string;
    ts_ms: number;
    overall_transfer_score: number;
    decision_kind: string;
    dominant_fail_family: string;
  };

  type CanaryFamilyRow = {
    report_id: string;
    family: string;
    n_evaluated: number;
    pass_count: number;
    fail_count: number;
    transfer_score: number;
    baseline_score: number;
  };

  type CanaryBreakdown = {
    by_report: Record<string, CanaryFamilyRow[]>;
  };

  let runs = $state<CanaryRunRow[]>([]);
  let breakdown = $state<CanaryBreakdown>({ by_report: {} });
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
      console.warn(`[canary] invoke('${cmd}') failed`, e);
      return undefined;
    }
  }

  async function reload() {
    const out = await callInvoke<CanaryRunRow[]>('get_canary_recent', { limit: 50 });
    const br = await callInvoke<CanaryBreakdown>('get_canary_per_family_breakdown', { limit: 50 });
    if (out === undefined) {
      runs = [];
      loadError = null;
    } else {
      runs = out;
      loadError = null;
    }
    if (br !== undefined) {
      breakdown = br;
    }
    loaded = true;
  }

  function shortId(s: string | null | undefined): string {
    if (!s) return '—';
    if (s.length <= 16) return s;
    return `${s.slice(0, 8)}…${s.slice(-4)}`;
  }

  function fmtDelta(n: number | null | undefined): string {
    if (n === null || n === undefined) return '—';
    if (!Number.isFinite(n)) return String(n);
    const sign = n > 0 ? '+' : '';
    return `${sign}${(n * 100).toFixed(1)}%`;
  }

  function fmtRate(n: number | null | undefined): string {
    if (n === null || n === undefined) return '—';
    if (!Number.isFinite(n)) return String(n);
    return `${(n * 100).toFixed(0)}%`;
  }

  function fmtFamilyLabel(f: string): string {
    return f.replace(/_/g, ' ').toUpperCase();
  }

  function decisionBadgeClass(kind: string): string {
    if (kind === 'pass') return 'badge badge-pass';
    if (kind === 'marginal_pass') return 'badge badge-marginal';
    if (kind === 'fail') return 'badge badge-fail';
    return 'badge';
  }

  function familyRows(reportId: string): CanaryFamilyRow[] {
    return breakdown.by_report[reportId] ?? [];
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

<section class="canary-panel">
  <header class="panel-header">
    <h2>Cross-Domain Canary</h2>
    <button class="reload-btn" onclick={reload}>Reload</button>
  </header>

  {#if !loaded}
    <p class="muted">Loading canary rows…</p>
  {:else if loadError}
    <div class="error-banner">
      <strong>Failed to load canary rows.</strong>
      <pre>{loadError}</pre>
    </div>
  {:else if runs.length === 0}
    <p class="muted">
      No canary runs recorded yet. Rows are emitted by the orchestrator
      via <code>canary::emit_canary_run</code> when the BAI promotion
      gate evaluates a candidate template against the 5 task families.
    </p>
  {:else}
    {#each runs as run (run.report_id)}
      <article class="run">
        <header class="run-header">
          <span class="run-id"><code>{shortId(run.report_id)}</code></span>
          <span class={decisionBadgeClass(run.decision_kind)}>
            {run.decision_kind.toUpperCase()}
          </span>
          <span class="overall">overall {fmtDelta(run.overall_transfer_score)}</span>
        </header>
        {#if run.decision_kind === 'fail' && run.dominant_fail_family}
          <p class="dominant">
            dominant regressor: <strong>{fmtFamilyLabel(run.dominant_fail_family)}</strong>
          </p>
        {/if}
        {#if familyRows(run.report_id).length > 0}
          <table class="families">
            <thead>
              <tr>
                <th>Family</th>
                <th>n</th>
                <th>Pass</th>
                <th>Fail</th>
                <th>Δ</th>
                <th>Base</th>
              </tr>
            </thead>
            <tbody>
              {#each familyRows(run.report_id) as fam (fam.family)}
                <tr
                  class:row-regression={fam.transfer_score < 0}
                  class:row-improvement={fam.transfer_score > 0}
                >
                  <td>{fmtFamilyLabel(fam.family)}</td>
                  <td>{fam.n_evaluated}</td>
                  <td>{fam.pass_count}</td>
                  <td>{fam.fail_count}</td>
                  <td>{fmtDelta(fam.transfer_score)}</td>
                  <td>{fmtRate(fam.baseline_score)}</td>
                </tr>
              {/each}
            </tbody>
          </table>
        {/if}
      </article>
    {/each}
  {/if}
</section>

<style>
  .canary-panel {
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

  .run {
    display: flex;
    flex-direction: column;
    gap: 6px;
    padding: 8px;
    border: 1px solid var(--border-subtle);
    border-radius: var(--r-sm);
    background: var(--bg-surface);
  }

  .run-header {
    display: flex;
    align-items: center;
    gap: 12px;
  }

  .run-id {
    color: var(--text-secondary);
  }

  .overall {
    color: var(--text-secondary);
    font-size: 11px;
    margin-left: auto;
  }

  .dominant {
    margin: 0;
    color: rgb(220, 120, 120);
    font-size: 11px;
  }

  table.families {
    width: 100%;
    border-collapse: collapse;
  }

  table.families th,
  table.families td {
    text-align: left;
    padding: 3px 6px;
    border-bottom: 1px solid var(--border-subtle);
  }

  table.families th {
    font-weight: 500;
    color: var(--text-secondary);
    letter-spacing: 0.04em;
    text-transform: uppercase;
    font-size: 10px;
  }

  .row-regression {
    background: rgba(220, 80, 80, 0.06);
  }

  .row-improvement {
    background: rgba(80, 200, 120, 0.06);
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

  .badge-pass {
    background: rgba(80, 200, 120, 0.18);
    border-color: rgba(80, 200, 120, 0.45);
    color: rgb(120, 220, 150);
  }

  .badge-marginal {
    background: rgba(220, 180, 80, 0.18);
    border-color: rgba(220, 180, 80, 0.45);
    color: rgb(220, 200, 120);
  }

  .badge-fail {
    background: rgba(220, 80, 80, 0.18);
    border-color: rgba(220, 80, 80, 0.45);
    color: rgb(230, 130, 130);
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
