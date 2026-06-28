<script lang="ts">
  interface Props {
    keys: string[];
  }

  let { keys }: Props = $props();
</script>

<span class="kbd-wrap">
  {#each keys as key, i}
    {#if i > 0}
      <span class="kbd-sep">+</span>
    {/if}
    <kbd class="kbd-key">{key}</kbd>
  {/each}
</span>

<style>
  .kbd-wrap {
    display: inline-flex;
    align-items: center;
    gap: 2px;
  }

  .kbd-sep {
    font-family: var(--font-mono);
    font-size: 10px;
    color: var(--text-ghost);
  }

  .kbd-key {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    background: var(--bg-surface);
    border: 1px solid var(--border-subtle);
    border-radius: var(--r-xs);
    padding: 1px 5px;
    font-family: var(--font-mono);
    font-size: 10px;
    color: var(--steel);
    line-height: var(--leading-code);
  }
</style>
