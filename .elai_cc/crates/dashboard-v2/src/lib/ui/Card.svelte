<script lang="ts">
  type Variant = 'status' | 'intervention' | 'error' | 'info' | 'plain';

  interface Props {
    variant?: Variant;
    padding?: number;
    children?: import('svelte').Snippet;
  }

  let {
    variant = 'plain',
    padding = 12,
    children,
  }: Props = $props();
</script>

<div class="card card--{variant} anim-enter-lift" style="padding: {padding}px;">
  {#if children}
    {@render children()}
  {/if}
</div>

<style>
  .card {
    background: var(--bg-surface);
    border-radius: var(--r-md);
    position: relative;
    overflow: hidden;
  }

  /* Left bar via pseudo-element */
  .card::before {
    content: '';
    position: absolute;
    left: 0;
    top: 0;
    bottom: 0;
    width: 0;
    border-radius: var(--r-md) 0 0 var(--r-md);
  }

  .card--status::before {
    width: 2px;
    background: var(--teal);
  }

  .card--intervention::before {
    width: 2px;
    background: var(--teal-bright);
  }

  .card--error::before {
    width: 2px;
    background: var(--fail);
  }

  .card--info::before {
    width: 2px;
    background: var(--steel);
  }

  .card--plain::before {
    display: none;
  }

  /* When has a left bar, add slight left padding to avoid content overlap */
  .card--status,
  .card--intervention,
  .card--error,
  .card--info {
    padding-left: calc(var(--s-3) + 2px) !important;
  }
</style>
