<script lang="ts">
  type Mode = 'dot' | 'line';

  interface Props {
    mode?: Mode;
    color?: string;
    size?: number;
  }

  let {
    mode = 'dot',
    color = '--teal',
    size,
  }: Props = $props();

  const dotSize = $derived(size ?? 6);
  const lineHeight = $derived(size ?? 1);
</script>

{#if mode === 'dot'}
  <span
    class="pulse-dot anim-breath"
    style="width:{dotSize}px; height:{dotSize}px; background:var({color});"
  ></span>
{:else}
  <span
    class="pulse-line anim-breath"
    style="height:{lineHeight}px; background:var({color});"
  ></span>
{/if}

<style>
  .pulse-dot {
    display: inline-block;
    border-radius: 50%;
    flex-shrink: 0;
  }

  .pulse-line {
    display: block;
    width: 100%;
    border-radius: 1px;
  }
</style>
