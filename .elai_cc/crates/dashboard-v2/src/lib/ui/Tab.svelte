<script lang="ts">
  import { getContext } from 'svelte';

  interface Props {
    id: string;
    children?: import('svelte').Snippet;
  }

  let { id, children }: Props = $props();

  const ctx = getContext<{ activeId: () => string; setActive: (id: string) => void }>('tabs');
  const active = $derived(ctx.activeId() === id);
</script>

<button
  class="tab"
  class:tab--active={active}
  role="tab"
  aria-selected={active}
  onclick={() => ctx.setActive(id)}
>
  {#if children}
    {@render children()}
  {/if}
</button>

<style>
  .tab {
    display: inline-flex;
    align-items: center;
    padding: 0 10px;
    height: 100%;
    font-family: var(--font-sans);
    font-size: 12px;
    color: var(--text-secondary);
    background: transparent;
    border: none;
    border-bottom: 1px solid transparent;
    cursor: pointer;
    white-space: nowrap;
    transition: color var(--dur-fast) var(--ease-std),
                border-color var(--dur-fast) var(--ease-std);
    outline: none;
  }

  .tab:focus-visible {
    box-shadow: inset 0 0 0 1px var(--teal);
  }

  .tab:hover:not(.tab--active) {
    color: var(--text-primary);
  }

  .tab--active {
    color: var(--text-primary);
    border-bottom-color: var(--teal);
  }
</style>
