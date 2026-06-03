<script lang="ts">
  import type { Component } from 'svelte';
  import type { IconProps } from '@lucide/svelte';
  import Kbd from './Kbd.svelte';

  interface Props {
    label: string;
    kbd?: string[];
    icon?: Component<IconProps>;
    selected?: boolean;
    onclick?: (e: MouseEvent) => void;
  }

  let {
    label,
    kbd,
    icon: Icon,
    selected = false,
    onclick,
  }: Props = $props();
</script>

<div
  class="cmd-row"
  class:cmd-row--selected={selected}
  role="option"
  aria-selected={selected}
  tabindex="0"
  onclick={onclick}
  onkeydown={(e) => { if (e.key === 'Enter' || e.key === ' ') { e.preventDefault(); onclick?.(e as unknown as MouseEvent); } }}
>
  {#if Icon}
    <span class="cmd-row__icon">
      <Icon size={14} strokeWidth={1.5} />
    </span>
  {:else}
    <span class="cmd-row__icon-placeholder"></span>
  {/if}

  <span class="cmd-row__label">{label}</span>

  {#if kbd && kbd.length > 0}
    <span class="cmd-row__kbd">
      <Kbd keys={kbd} />
    </span>
  {/if}
</div>

<style>
  .cmd-row {
    display: flex;
    align-items: center;
    gap: 8px;
    height: 32px;
    padding: 0 10px;
    border-radius: var(--r-sm);
    cursor: pointer;
    outline: none;
    transition: background-color var(--dur-fast) var(--ease-std);
  }

  .cmd-row:hover,
  .cmd-row:focus-visible {
    background: var(--bg-hover);
  }

  .cmd-row--selected {
    background: var(--teal-ghost);
  }

  .cmd-row__icon {
    display: flex;
    align-items: center;
    color: var(--text-secondary);
    flex-shrink: 0;
    width: 16px;
  }

  .cmd-row--selected .cmd-row__icon {
    color: var(--teal);
  }

  .cmd-row__icon-placeholder {
    width: 16px;
    flex-shrink: 0;
  }

  .cmd-row__label {
    flex: 1;
    min-width: 0;
    font-family: var(--font-sans);
    font-size: 12px;
    color: var(--text-primary);
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .cmd-row__kbd {
    flex-shrink: 0;
    margin-left: auto;
  }
</style>
