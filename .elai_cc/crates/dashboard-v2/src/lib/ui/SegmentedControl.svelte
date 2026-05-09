<script lang="ts">
  interface Option {
    value: string;
    label: string;
  }

  interface Props {
    value?: string;
    options: Option[];
  }

  let {
    value = $bindable(''),
    options,
  }: Props = $props();

  let containerEl = $state<HTMLDivElement | null>(null);
  let buttonEls = $state<HTMLButtonElement[]>([]);

  const activeIndex = $derived(options.findIndex((o) => o.value === value));

  // Compute underline position from button measurements
  let lineLeft = $state(0);
  let lineWidth = $state(0);

  $effect(() => {
    if (activeIndex >= 0 && buttonEls[activeIndex]) {
      const btn = buttonEls[activeIndex];
      const parent = containerEl;
      if (parent) {
        const pRect = parent.getBoundingClientRect();
        const bRect = btn.getBoundingClientRect();
        lineLeft = bRect.left - pRect.left;
        lineWidth = bRect.width;
      }
    }
  });

  function select(opt: Option) {
    value = opt.value;
  }
</script>

<div class="seg" bind:this={containerEl}>
  {#each options as opt, i}
    <button
      bind:this={buttonEls[i]}
      class="seg-btn"
      class:seg-btn--active={opt.value === value}
      onclick={() => select(opt)}
    >
      {opt.label}
    </button>
  {/each}

  <!-- Sliding underline -->
  {#if activeIndex >= 0}
    <span
      class="seg-line"
      style="left:{lineLeft}px; width:{lineWidth}px;"
    ></span>
  {/if}
</div>

<style>
  .seg {
    position: relative;
    display: inline-flex;
    align-items: center;
    border-bottom: 1px solid var(--border-subtle);
  }

  .seg-btn {
    position: relative;
    display: inline-flex;
    align-items: center;
    justify-content: center;
    padding: 4px 12px;
    font-family: var(--font-sans);
    font-size: 12px;
    color: var(--text-secondary);
    background: transparent;
    border: none;
    cursor: pointer;
    outline: none;
    letter-spacing: 0.06em;
    text-transform: uppercase;
    font-size: 11px;
    transition: color var(--dur-fast) var(--ease-std);
  }

  .seg-btn:focus-visible {
    box-shadow: inset 0 0 0 1px var(--teal);
    border-radius: var(--r-xs);
  }

  .seg-btn--active {
    color: var(--teal);
  }

  .seg-line {
    position: absolute;
    bottom: -1px;
    height: 1px;
    background: var(--teal);
    border-radius: 1px;
    transition: left var(--dur-base) var(--ease-std),
                width var(--dur-base) var(--ease-std);
  }
</style>
