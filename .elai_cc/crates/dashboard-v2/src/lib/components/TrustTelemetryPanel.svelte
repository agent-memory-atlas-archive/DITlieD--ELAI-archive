<script lang="ts">
  // E-23 — TrustTelemetryPanel.
  //
  // 5-card grid surfacing the productivity / verification / trust roll-up
  // from orchestrator::trust_telemetry::aggregate_session. Wire-dark
  // unless ELAI_DEVELOPER_DISCOURSE_ENABLE=1 on the orchestrator side;
  // when off, the panel renders zeroed cards plus a "telemetry disabled"
  // hint so the user can see why the numbers are flat.
  //
  // Source signals (consumed read-only from existing SSB rows):
  //   1. tasks_per_hour       — phase_transition rows into COMPLETE
  //                             divided by elapsed-session hours.
  //   2. findings_per_task    — auditor finding rows / completed task.
  //   3. revert_rate          — file_write rows with `revert: true` /
  //                             total file_write rows.
  //   4. ungrounded_count     — mcp_think claim rows with empty
  //                             grounding (HARD RULE per CLAUDE.md).
  //   5. stagnation_dist      — C-2 / B-22b spinning + oscillation +
  //                             no_drift + diminishing_returns counts.
  //
  // PER memory feedback_verify_semantics: these are BASELINE numbers,
  // not "we beat METR" claims. The METR RCT (arXiv:2507.09089) reports
  // 19% slowdown on Cursor Pro + Sonnet 3.5/3.7; ELAI's numbers are not
  // directly comparable without paired-task data. Panel-header tooltip
  // documents this.

  import { onMount, onDestroy } from 'svelte';
  import { get_trust_telemetry } from '../api/stub';
  import type { TrustReport } from '../api/types';

  interface Props {
    pollMs?: number;
  }
  const { pollMs = 30_000 }: Props = $props();

  let report = $state<TrustReport>({
    tasks_per_hour: 0,
    findings_per_task: 0,
    revert_rate: 0,
    ungrounded_count: 0,
    stagnation_dist: {},
  });
  let loadError = $state<string | null>(null);
  let pollHandle: ReturnType<typeof setInterval> | null = null;

  async function reload() {
    try {
      report = await get_trust_telemetry();
      loadError = null;
    } catch (e) {
      loadError = e instanceof Error ? e.message : String(e);
    }
  }

  const isWireDark = $derived.by(() => {
    // Heuristic: when the gate is off, every numeric field is zero AND
    // the stagnation_dist is empty. The Rust side returns an empty
    // shape in that case, so this catches it without a separate flag.
    return (
      report.tasks_per_hour === 0 &&
      report.findings_per_task === 0 &&
      report.revert_rate === 0 &&
      report.ungrounded_count === 0 &&
      Object.keys(report.stagnation_dist).length === 0
    );
  });

  const stagnationLabels = $derived.by(() =>
    Object.entries(report.stagnation_dist).sort(([a], [b]) => a.localeCompare(b)),
  );

  onMount(() => {
    void reload();
    pollHandle = setInterval(reload, pollMs);
  });

  onDestroy(() => {
    if (pollHandle !== null) {
      clearInterval(pollHandle);
      pollHandle = null;
    }
  });
</script>

<section
  class="trust-telemetry-panel"
  title="TrustTelemetryPanel — baseline productivity / verification / trust roll-up. NOT a counter-claim to METR arXiv:2507.09089; paired-task data required for direct comparison. Wire-dark unless ELAI_DEVELOPER_DISCOURSE_ENABLE=1."
>
  <header class="header mono">
    <span class="label-ghost">trust telemetry</span>
    {#if isWireDark}
      <span class="gate-hint mono">telemetry disabled (set ELAI_DEVELOPER_DISCOURSE_ENABLE=1)</span>
    {/if}
  </header>

  <div class="grid">
    <div class="card">
      <span class="card-label mono">tasks / hour</span>
      <span class="card-val">{report.tasks_per_hour.toFixed(2)}</span>
      <span class="card-hint mono">phase_transition to COMPLETE</span>
    </div>
    <div class="card">
      <span class="card-label mono">findings / task</span>
      <span class="card-val">{report.findings_per_task.toFixed(2)}</span>
      <span class="card-hint mono">verification density</span>
    </div>
    <div class="card">
      <span class="card-label mono">revert rate</span>
      <span class="card-val">{(report.revert_rate * 100).toFixed(1)}%</span>
      <span class="card-hint mono">file_write revert ratio</span>
    </div>
    <div class="card" class:card--warn={report.ungrounded_count > 0}>
      <span class="card-label mono">ungrounded</span>
      <span class="card-val">{report.ungrounded_count}</span>
      <span class="card-hint mono">claims with empty grounding</span>
    </div>
    <div class="card card--wide">
      <span class="card-label mono">stagnation</span>
      <div class="stagnation-row">
        {#each stagnationLabels as [label, count] (label)}
          <span class="stagnation-pair mono" class:stagnation-pair--hot={count > 0}>
            {label}: {count}
          </span>
        {/each}
        {#if stagnationLabels.length === 0}
          <span class="card-hint mono">no signals yet</span>
        {/if}
      </div>
    </div>
  </div>

  {#if loadError}
    <div class="error mono">load error: {loadError}</div>
  {/if}
</section>

<style>
  .trust-telemetry-panel {
    display: flex;
    flex-direction: column;
    gap: 8px;
    padding: 10px 12px;
    border: 1px solid var(--border-default);
    border-radius: var(--r-sm);
    background: var(--bg-surface);
  }

  .header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 8px;
    flex-wrap: wrap;
  }

  .label-ghost {
    font-size: 10px;
    color: var(--text-ghost);
    text-transform: uppercase;
    letter-spacing: 0.08em;
  }

  .gate-hint {
    font-size: 10px;
    color: var(--text-ghost);
    font-style: italic;
  }

  .grid {
    display: grid;
    grid-template-columns: repeat(4, 1fr);
    gap: 8px;
  }

  .card {
    display: flex;
    flex-direction: column;
    gap: 2px;
    padding: 6px 8px;
    border: 1px solid var(--border-subtle);
    border-radius: var(--r-xs);
    background: var(--bg-input);
  }

  .card--wide {
    grid-column: span 4;
  }

  .card--warn {
    border-color: rgba(212, 160, 23, 0.5);
    background: rgba(212, 160, 23, 0.05);
  }

  .card-label {
    font-size: 10px;
    color: var(--text-ghost);
    text-transform: lowercase;
  }

  .card-val {
    font-family: var(--font-mono);
    font-size: 16px;
    color: var(--text-primary);
    font-weight: 500;
  }

  .card-hint {
    font-size: 9px;
    color: var(--text-disabled);
    font-style: italic;
  }

  .stagnation-row {
    display: flex;
    flex-wrap: wrap;
    gap: 6px;
    padding-top: 2px;
  }

  .stagnation-pair {
    font-size: 10px;
    color: var(--text-secondary);
    padding: 2px 6px;
    border: 1px solid var(--border-subtle);
    border-radius: var(--r-pill);
  }

  .stagnation-pair--hot {
    color: var(--fail);
    border-color: rgba(176, 69, 69, 0.4);
    background: rgba(176, 69, 69, 0.06);
  }

  .error {
    font-size: 10px;
    color: var(--fail);
    font-style: italic;
  }

  .mono {
    font-family: var(--font-mono);
  }
</style>
