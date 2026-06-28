<script lang="ts">
  // UnwiredNote — placeholder shown in the dashboard for any data section
  // whose backend producer is not yet implemented.
  //
  // Rationale: rather than render a mock fixture (misleading) or hide the
  // section entirely (loses the discoverability of what the dashboard is
  // *meant* to surface), we render a discreet note with the phase id where
  // the producer is scheduled to land. See B-0_INDEX.md for phase numbers.
  interface Props {
    /** Phase number where the producer is scheduled (e.g. "B-4", "4G", "1I.6"). */
    phase: string;
    /** Short human-readable description of what would appear here once wired. */
    feature?: string;
    /** Optional one-line note explaining the gap (which command returns empty/zeros). */
    note?: string;
  }
  let { phase, feature, note }: Props = $props();
</script>

<div class="unwired" role="status" aria-label="data source not yet wired">
  <div class="header">
    <span class="badge">UNWIRED</span>
    <span class="phase">phase {phase}</span>
  </div>
  {#if feature}
    <span class="feature">{feature}</span>
  {/if}
  {#if note}
    <span class="note">{note}</span>
  {/if}
  <span class="hint">producer pending — UI surface ready, backend signal not yet emitted</span>
</div>

<style>
  .unwired {
    display: flex;
    flex-direction: column;
    gap: 4px;
    padding: 8px 10px;
    background: rgba(195, 154, 58, 0.04);
    border: 1px dashed rgba(195, 154, 58, 0.3);
    border-radius: var(--r-sm);
    color: var(--text-ghost);
  }
  .header {
    display: flex;
    align-items: center;
    gap: 8px;
  }
  .badge {
    font-family: var(--font-mono);
    font-size: 10px;
    font-weight: 600;
    color: var(--warn);
    letter-spacing: 0.10em;
    padding: 1px 6px;
    border: 1px solid rgba(195, 154, 58, 0.4);
    border-radius: var(--r-pill);
    background: rgba(195, 154, 58, 0.08);
  }
  .phase {
    font-family: var(--font-mono);
    font-size: 10px;
    color: var(--warn);
    letter-spacing: 0.06em;
  }
  .feature {
    font-family: var(--font-sans);
    font-size: 10px;
    color: var(--text-secondary);
  }
  .note {
    font-family: var(--font-mono);
    font-size: 10px;
    color: var(--text-ghost);
    line-height: var(--leading-code);
  }
  .hint {
    font-family: var(--font-mono);
    font-size: 10px;
    color: var(--text-ghost);
    font-style: italic;
    line-height: var(--leading-code);
  }
</style>
