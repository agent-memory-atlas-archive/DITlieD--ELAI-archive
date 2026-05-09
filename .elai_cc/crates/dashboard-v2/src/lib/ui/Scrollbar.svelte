<script lang="ts">
  type Axis = 'y' | 'x' | 'both';

  interface Props {
    axis?: Axis;
    children?: import('svelte').Snippet;
  }

  let {
    axis = 'y',
    children,
  }: Props = $props();
</script>

<div
  class="scrollbar-wrap"
  class:scroll-y={axis === 'y' || axis === 'both'}
  class:scroll-x={axis === 'x' || axis === 'both'}
>
  {#if children}
    {@render children()}
  {/if}
</div>

<style>
  .scrollbar-wrap {
    overflow: hidden;
    scrollbar-gutter: stable;
    min-height: 0;
    min-width: 0;
    flex: 1;
  }

  .scroll-y {
    overflow-y: auto;
  }

  .scroll-x {
    overflow-x: auto;
  }
</style>
