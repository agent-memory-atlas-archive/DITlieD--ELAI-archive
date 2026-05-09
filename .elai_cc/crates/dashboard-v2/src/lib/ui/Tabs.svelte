<script lang="ts">
  import { setContext } from 'svelte';

  interface Props {
    value?: string;
    children?: import('svelte').Snippet;
  }

  let {
    value = $bindable(''),
    children,
  }: Props = $props();

  setContext('tabs', {
    activeId: () => value,
    setActive: (id: string) => { value = id; },
  });
</script>

<div class="tabs" role="tablist">
  {#if children}
    {@render children()}
  {/if}
</div>

<style>
  .tabs {
    display: flex;
    align-items: stretch;
    height: 28px;
    border-bottom: 1px solid var(--border-subtle);
  }
</style>
