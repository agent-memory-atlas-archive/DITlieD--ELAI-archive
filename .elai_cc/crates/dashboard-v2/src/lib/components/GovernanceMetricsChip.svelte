<script lang="ts">
  // I-33 (SCHEMA_VERSION 240): governance metrics chip.
  //
  // Inline chip rendering CDL / DIU / FSR governance metrics sourced from
  // GovernancePrimitiveFired SSB rows (commands_i33.rs::get_governance_metrics).
  //
  // Three metric bars:
  //   CDL = Cosmetic-Deadlock Rate (CeflExternalized / total)
  //   DIU = Deferral Info Utilisation (I6qRejected / total)
  //   FSR = Framing Success Rate (clean CommitRevealSealed / sealed total)
  //
  // Cross-lang pair: commands_i33.rs::get_governance_metrics
  //                  types.ts::GovernanceMetrics
  //
  // Svelte 5 runes only. No createEventDispatcher, no $: labels.

  import type { GovernanceMetrics } from '$lib/api/types';

  interface Props {
    metrics: GovernanceMetrics | null;
    loading?: boolean;
  }

  const { metrics, loading = false }: Props = $props();

  const fmt = (v: number) => (v * 100).toFixed(1) + '%';

  let cdlText = $derived(metrics ? fmt(metrics.cdl) : '--');
  let diuText = $derived(metrics ? fmt(metrics.diu) : '--');
  let fsrText = $derived(metrics ? fmt(metrics.fsr) : '--');
  let rowText = $derived(metrics ? String(metrics.row_count) : '0');

  // FSR colour: green >=90%, amber >=70%, red below.
  let fsrClass = $derived.by(() => {
    if (!metrics) return 'gmc__bar--neutral';
    if (metrics.fsr >= 0.9) return 'gmc__bar--good';
    if (metrics.fsr >= 0.7) return 'gmc__bar--warn';
    return 'gmc__bar--fail';
  });
</script>

<div class="gmc" aria-label="Governance metrics" role="region">
  {#if loading}
    <span class="gmc__loading">loading&hellip;</span>
  {:else if !metrics}
    <span class="gmc__empty">no governance rows</span>
  {:else}
    <ul class="gmc__list">
      <li class="gmc__item">
        <span class="gmc__label" title="Cosmetic-Deadlock Rate: CeflExternalized / total decisions">CDL</span>
        <span class="gmc__bar gmc__bar--neutral" style="--pct:{(metrics.cdl * 100).toFixed(1)}%">
          <span class="gmc__val">{cdlText}</span>
        </span>
      </li>
      <li class="gmc__item">
        <span class="gmc__label" title="Deferral Info Utilisation: I6qRejected / total decisions">DIU</span>
        <span class="gmc__bar gmc__bar--neutral" style="--pct:{(metrics.diu * 100).toFixed(1)}%">
          <span class="gmc__val">{diuText}</span>
        </span>
      </li>
      <li class="gmc__item">
        <span class="gmc__label" title="Framing Success Rate: clean commit-reveal seals / total seals">FSR</span>
        <span class={`gmc__bar ${fsrClass}`} style="--pct:{(metrics.fsr * 100).toFixed(1)}%">
          <span class="gmc__val">{fsrText}</span>
        </span>
      </li>
    </ul>
    <span class="gmc__rows">{rowText} rows</span>
  {/if}
</div>

<style>
  .gmc {
    display: inline-flex;
    flex-direction: column;
    gap: 0.25rem;
    padding: 0.375rem 0.5rem;
    border: 1px solid var(--border-muted, oklch(45% 0 0));
    border-radius: 0.375rem;
    background: var(--bg-surface, oklch(18% 0 0));
    font-size: 0.75rem;
    font-family: var(--font-mono, monospace);
    color: var(--text-primary, oklch(85% 0 0));
    min-width: 10rem;
  }

  .gmc__loading,
  .gmc__empty {
    color: var(--text-muted, oklch(55% 0 0));
    font-style: italic;
  }

  .gmc__list {
    list-style: none;
    margin: 0;
    padding: 0;
    display: flex;
    flex-direction: column;
    gap: 0.125rem;
  }

  .gmc__item {
    display: flex;
    align-items: center;
    gap: 0.375rem;
  }

  .gmc__label {
    width: 2rem;
    color: var(--text-muted, oklch(55% 0 0));
    flex-shrink: 0;
    cursor: default;
  }

  .gmc__bar {
    position: relative;
    flex: 1;
    height: 1rem;
    border-radius: 2px;
    background: var(--bg-elevated, oklch(22% 0 0));
    overflow: hidden;
  }

  .gmc__bar::before {
    content: '';
    position: absolute;
    inset-block: 0;
    inset-inline-start: 0;
    width: var(--pct, 0%);
    border-radius: 2px;
    background: currentColor;
    opacity: 0.35;
  }

  .gmc__bar--neutral { color: var(--text-secondary, oklch(65% 0 0)); }
  .gmc__bar--good    { color: oklch(72% 0.18 150); }
  .gmc__bar--warn    { color: oklch(78% 0.18 75); }
  .gmc__bar--fail    { color: oklch(65% 0.22 25); }

  .gmc__val {
    position: relative;
    z-index: 1;
    padding-inline-start: 0.25rem;
    font-variant-numeric: tabular-nums;
    line-height: 1rem;
  }

  .gmc__rows {
    color: var(--text-muted, oklch(55% 0 0));
    font-size: 0.6875rem;
    text-align: right;
  }

  @media (prefers-reduced-motion: no-preference) {
    .gmc__bar::before {
      transition: width 0.25s ease-out;
    }
  }
</style>
