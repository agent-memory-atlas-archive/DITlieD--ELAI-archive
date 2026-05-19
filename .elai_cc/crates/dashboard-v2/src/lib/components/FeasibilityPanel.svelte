<!--
  C-22 FeasibilityPanel — live FeasibilityReport surface for the
  ReasoningDrawer.

  cross-lang-pair: dashboard/tauri/api
  Pairs with `get_feasibility_report` in src-tauri/src/commands.rs and the
  `FeasibilityReport` TypeScript type in src/lib/api/types.ts.

  Renders:
   - overall PASS/FAIL/BLOCKED badge + probed_at timestamp
   - aggregate counts: probe_count, alternatives_explored, novel_step_count
   - per-result list: each result's check name + outcome + detail + timestamp
   - skip_reason advisory when populated
   - graceful "no feasibility checks recorded yet" empty state

  Data path: Tauri `get_feasibility_report` -> SsbViewer::feasibility_summary
  -> decoded SSB feasibility_report + novel_step_detected + stub_run rows.
  No mock data; if the backend returns the empty PASS shape (no probe yet)
  the empty-state branch fires.

  Svelte 5 runes: $state / $derived / $effect.
-->
<script lang="ts">
  import { Badge } from '$lib/ui';
  import type { FeasibilityReport, FeasibilityOutcome } from '$lib/api/types';

  interface Props {
    report: FeasibilityReport | null;
  }

  let { report }: Props = $props();

  function toneFor(outcome: FeasibilityOutcome): string {
    if (outcome === 'PASS') return 'teal';
    if (outcome === 'FAIL') return 'fail';
    return 'warn';
  }

  function fmtTs(ms: number | null | undefined): string {
    if (!ms) return '';
    return new Date(ms).toISOString().slice(11, 19);
  }

  const hasResults = $derived(!!report && report.results.length > 0);
  const skipReason = $derived(report?.skip_reason ?? null);

  // Aggregate counts default to 0 when the older backend shape is in play.
  const probeCount = $derived(report?.probe_count ?? 0);
  const alternativesExplored = $derived(report?.alternatives_explored ?? 0);
  const novelStepCount = $derived(report?.novel_step_count ?? 0);
</script>

<section class="feasibility-panel" aria-label="Feasibility probe">
  {#if !report}
    <div class="feasibility-empty">
      <span class="ghost-text">Feasibility probe not run</span>
    </div>
  {:else}
    <div class="feasibility-header">
      <span class="feasibility-overall-label">overall</span>
      <Badge tone={toneFor(report.overall)} size="xs">{report.overall}</Badge>
      <span class="ghost-text">probed {fmtTs(report.probed_at)}</span>
    </div>

    <div class="feasibility-aggregates">
      <span class="aggregate">
        <span class="aggregate-label">probes</span>
        <span class="aggregate-value">{probeCount}</span>
      </span>
      <span class="aggregate">
        <span class="aggregate-label">novel</span>
        <span class="aggregate-value">{novelStepCount}</span>
      </span>
      <span class="aggregate">
        <span class="aggregate-label">alts</span>
        <span class="aggregate-value">{alternativesExplored}</span>
      </span>
    </div>

    {#if skipReason}
      <div class="feasibility-skip">
        <Badge tone="warn" size="xs">SKIP</Badge>
        <span class="skip-reason">{skipReason}</span>
      </div>
    {/if}

    {#if hasResults}
      <ul class="feasibility-rows">
        {#each report.results as r (r.id)}
          <li class="feasibility-row">
            <span class="feasibility-check">{r.check}</span>
            <Badge tone={toneFor(r.outcome)} size="xs">{r.outcome}</Badge>
            <span class="feasibility-detail" title={r.detail ?? ''}>
              {r.detail ?? ''}
            </span>
            <span class="ghost-text">{fmtTs(r.ts)}</span>
          </li>
        {/each}
      </ul>
    {:else if !skipReason}
      <div class="feasibility-empty">
        <span class="ghost-text">no feasibility checks recorded yet</span>
      </div>
    {/if}
  {/if}
</section>

<style>
  .feasibility-panel {
    display: flex;
    flex-direction: column;
    gap: 6px;
  }

  .feasibility-header {
    display: flex;
    align-items: center;
    gap: 8px;
  }

  .feasibility-overall-label {
    font-size: 0.72rem;
    text-transform: uppercase;
    letter-spacing: 0.04em;
    color: var(--text-muted, #888);
  }

  .feasibility-aggregates {
    display: flex;
    gap: 12px;
    flex-wrap: wrap;
  }

  .aggregate {
    display: flex;
    align-items: baseline;
    gap: 4px;
  }

  .aggregate-label {
    font-size: 0.7rem;
    text-transform: uppercase;
    letter-spacing: 0.03em;
    color: var(--text-muted, #888);
  }

  .aggregate-value {
    font-variant-numeric: tabular-nums;
    font-weight: 600;
  }

  .feasibility-skip {
    display: flex;
    align-items: center;
    gap: 6px;
  }

  .skip-reason {
    font-family: var(--font-mono, monospace);
    font-size: 0.78rem;
    color: var(--text-muted, #aaa);
  }

  .feasibility-rows {
    list-style: none;
    margin: 0;
    padding: 0;
    display: flex;
    flex-direction: column;
    gap: 4px;
  }

  .feasibility-row {
    display: grid;
    grid-template-columns: minmax(80px, max-content) max-content minmax(0, 1fr) max-content;
    align-items: center;
    gap: 8px;
    font-size: 0.78rem;
  }

  .feasibility-check {
    font-family: var(--font-mono, monospace);
    color: var(--text-muted, #aaa);
  }

  .feasibility-detail {
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    color: var(--text-default, #ddd);
  }

  .feasibility-empty {
    display: flex;
    align-items: center;
    gap: 6px;
    padding: 6px 0;
  }

  .ghost-text {
    color: var(--text-muted, #888);
    font-size: 0.72rem;
  }
</style>
