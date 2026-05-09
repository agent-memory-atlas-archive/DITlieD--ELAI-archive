<script lang="ts">
  import type { Component } from 'svelte';
  import type { IconProps } from '@lucide/svelte';

  type Variant = 'default' | 'accent';
  type Size = 'sm' | 'md' | 'lg';

  interface Props {
    icon: Component<IconProps>;
    label: string;
    onclick?: (e: MouseEvent) => void;
    size?: Size;
    variant?: Variant;
    active?: boolean;
    disabled?: boolean;
  }

  let {
    icon: Icon,
    label,
    onclick,
    size = 'md',
    variant = 'default',
    active = false,
    disabled = false,
  }: Props = $props();

  const pxMap: Record<Size, number> = { sm: 24, md: 28, lg: 32 };
  const iconPxMap: Record<Size, number> = { sm: 14, md: 16, lg: 18 };
  const px = $derived(pxMap[size]);
  const iconPx = $derived(iconPxMap[size]);
</script>

<button
  class="ibtn ibtn--{variant}"
  class:ibtn--active={active}
  title={label}
  aria-label={label}
  {disabled}
  onclick={onclick}
  style="width:{px}px; height:{px}px;"
>
  <Icon size={iconPx} strokeWidth={1.5} />
</button>

<style>
  .ibtn {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    background: transparent;
    border: none;
    border-radius: var(--r-sm);
    cursor: pointer;
    color: var(--text-secondary);
    flex-shrink: 0;
    transition: background-color var(--dur-fast) var(--ease-std),
                color var(--dur-fast) var(--ease-std);
    outline: none;
  }

  .ibtn:focus-visible {
    box-shadow: var(--ring);
  }

  .ibtn:active:not(:disabled) {
    transform: scale(0.96);
  }

  .ibtn:hover:not(:disabled) {
    color: var(--text-primary);
    background: var(--bg-hover);
  }

  .ibtn--accent {
    color: var(--teal);
  }

  .ibtn--active,
  .ibtn--active:hover {
    color: var(--teal);
    background: var(--teal-ghost);
  }

  .ibtn:disabled {
    opacity: 0.4;
    cursor: not-allowed;
  }
</style>
