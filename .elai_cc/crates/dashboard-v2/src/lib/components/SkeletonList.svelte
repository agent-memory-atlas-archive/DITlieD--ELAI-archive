<script lang="ts">
  // E-33: Generic skeleton-row placeholder. Per NN/g + LogRocket +
  // OneThing, skeleton screens deliver ~20-30% perceived-speed gain
  // over indeterminate spinners. Use for any async list pane.
  //
  // API:
  //   <SkeletonList rows={5} loading={isLoading} />
  // While loading, renders N placeholder rows with shimmer. When loading
  // flips false, the parent renders the real list — replace row-by-row,
  // not all-at-once, to preserve continuity (see PlanWindow / ReviewWindow).

  interface Props {
    rows?: number;
    loading: boolean;
    rowHeight?: number; // px
    label?: string; // a11y label e.g. "loading audit findings"
  }
  let { rows = 4, loading, rowHeight = 28, label = 'loading' }: Props = $props();

  // Pseudo-random widths derived from index — keep visual deterministic so
  // tests can snapshot. No Math.random() to avoid hydration mismatch.
  function widthAt(i: number, n: number): number {
    const t = (i + 1) / (n + 1);
    return 55 + Math.round(40 * (0.5 + 0.5 * Math.sin(t * Math.PI * 3)));
  }
</script>

{#if loading}
  <ul class="skeleton-list" aria-busy="true" aria-label={label}>
    {#each Array(rows) as _, i (i)}
      <li class="skeleton-row" style="height: {rowHeight}px;">
        <span class="skeleton-bar" style="width: {widthAt(i, rows)}%;"></span>
      </li>
    {/each}
  </ul>
{/if}

<style>
  .skeleton-list {
    list-style: none;
    margin: 0;
    padding: 0;
    display: flex;
    flex-direction: column;
    gap: var(--s-1);
  }
  .skeleton-row {
    display: flex;
    align-items: center;
    padding: var(--s-1) var(--s-2);
    border-radius: var(--r-sm);
  }
  .skeleton-bar {
    display: inline-block;
    height: 10px;
    border-radius: var(--r-sm);
    background: linear-gradient(
      90deg,
      var(--bg-input) 0%,
      var(--bg-hover) 50%,
      var(--bg-input) 100%
    );
    background-size: 200% 100%;
    animation: skeleton-shimmer 1.6s linear infinite;
  }
  @keyframes skeleton-shimmer {
    0% {
      background-position: 200% 0;
    }
    100% {
      background-position: -200% 0;
    }
  }
  @media (prefers-reduced-motion: reduce) {
    .skeleton-bar {
      animation: none;
      background: var(--bg-input);
    }
  }
</style>
