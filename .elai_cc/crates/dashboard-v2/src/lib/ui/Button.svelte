<script lang="ts">
  import type { Component } from 'svelte';
  import type { IconProps } from '@lucide/svelte';

  type Variant = 'primary' | 'ghost' | 'text' | 'danger';
  type Size = 'sm' | 'md' | 'lg';
  type BtnType = 'button' | 'submit';

  interface Props {
    variant?: Variant;
    size?: Size;
    disabled?: boolean;
    type?: BtnType;
    onclick?: (e: MouseEvent) => void;
    iconLeft?: Component<IconProps>;
    iconRight?: Component<IconProps>;
    children?: import('svelte').Snippet;
  }

  let {
    variant = 'ghost',
    size = 'md',
    disabled = false,
    type = 'button',
    onclick,
    iconLeft: IconLeft,
    iconRight: IconRight,
    children,
  }: Props = $props();

  const iconSizeMap: Record<Size, number> = { sm: 12, md: 13, lg: 14 };
  const iconSize = $derived(iconSizeMap[size]);
</script>

<button
  {type}
  {disabled}
  class="btn btn--{variant} btn--{size}"
  onclick={onclick}
  aria-disabled={disabled}
>
  {#if IconLeft}
    <span class="btn__icon btn__icon--left">
      <IconLeft size={iconSize} strokeWidth={1.5} />
    </span>
  {/if}
  {#if children}
    {@render children()}
  {/if}
  {#if IconRight}
    <span class="btn__icon btn__icon--right">
      <IconRight size={iconSize} strokeWidth={1.5} />
    </span>
  {/if}
</button>

<style>
  .btn {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    gap: 5px;
    border: none;
    border-radius: var(--r-sm);
    font-family: var(--font-sans);
    font-weight: 500;
    letter-spacing: 0.01em;
    cursor: pointer;
    user-select: none;
    white-space: nowrap;
    transition: background-color var(--dur-fast) var(--ease-std),
                color var(--dur-fast) var(--ease-std),
                border-color var(--dur-fast) var(--ease-std);
    outline: none;
  }

  .btn:focus-visible {
    box-shadow: var(--ring);
  }

  .btn:active:not(:disabled) {
    transform: scale(0.98);
    transition: transform 0ms, background-color var(--dur-fast) var(--ease-std), color var(--dur-fast) var(--ease-std);
  }

  .btn:not(:active) {
    transition: transform var(--dur-fast) var(--ease-std),
                background-color var(--dur-fast) var(--ease-std),
                color var(--dur-fast) var(--ease-std),
                border-color var(--dur-fast) var(--ease-std);
  }

  /* Sizes */
  .btn--sm {
    height: 24px;
    padding: 0 10px;
    font-size: 10px;
  }

  .btn--md {
    height: 28px;
    padding: 0 12px;
    font-size: 12px;
  }

  .btn--lg {
    height: 36px;
    padding: 0 16px;
    font-size: 13px;
  }

  /* Primary — outlined teal accent (matches the .btn-approve aesthetic
     in CommandWindow). The teal-line border is the affordance, never a
     solid fill. */
  .btn--primary {
    background: transparent;
    color: var(--teal);
    border: 1px solid var(--teal-line);
  }
  .btn--primary:hover:not(:disabled) {
    background: var(--teal-ghost);
    color: var(--teal-bright);
    border-color: var(--teal);
  }

  /* Ghost — subtle outline (matches the .btn-outlined aesthetic used
     for Reject / Modify / Take-over / disambiguation pills). The
     default border is --border-default; on hover it lifts to teal-line. */
  .btn--ghost {
    background: transparent;
    color: var(--text-secondary);
    border: 1px solid var(--border-default);
  }
  .btn--ghost:hover:not(:disabled) {
    border-color: var(--teal-line);
    background: var(--teal-ghost);
    color: var(--text-primary);
  }

  /* Text — link-style, used for tertiary/inline actions only. */
  .btn--text {
    background: transparent;
    color: var(--teal);
    border: 1px solid transparent;
    padding-left: 4px;
    padding-right: 4px;
  }
  .btn--text:hover:not(:disabled) {
    color: var(--teal-bright);
    text-decoration: underline;
    text-underline-offset: 3px;
  }

  /* Danger */
  .btn--danger {
    background: transparent;
    color: var(--fail);
    border: 1px solid transparent;
  }
  .btn--danger:hover:not(:disabled) {
    background: rgba(176, 69, 69, 0.12);
  }

  /* Disabled */
  .btn:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }

  .btn__icon {
    display: flex;
    align-items: center;
    flex-shrink: 0;
  }
</style>
