<script lang="ts">
  import type { Component } from 'svelte';
  import type { IconProps } from '@lucide/svelte';

  type InputType = 'text' | 'search' | 'password' | 'number';
  type Size = 'sm' | 'md';

  interface Props {
    value?: string | number;
    placeholder?: string;
    type?: InputType;
    disabled?: boolean;
    mono?: boolean;
    onenter?: (value: string | number) => void;
    iconLeft?: Component<IconProps>;
    iconRight?: Component<IconProps>;
    size?: Size;
  }

  let {
    value = $bindable(''),
    placeholder = '',
    type = 'text',
    disabled = false,
    mono = false,
    onenter,
    iconLeft: IconLeft,
    iconRight: IconRight,
    size = 'md',
  }: Props = $props();

  function handleKeydown(e: KeyboardEvent) {
    if (e.key === 'Enter' && onenter) {
      onenter(value);
    }
  }
</script>

<div class="input-wrap input-wrap--{size}" class:input-wrap--mono={mono} class:input-wrap--disabled={disabled}>
  {#if IconLeft}
    <span class="input-icon input-icon--left">
      <IconLeft size={14} strokeWidth={1.5} />
    </span>
  {/if}
  <input
    bind:value
    {type}
    {placeholder}
    {disabled}
    class="input"
    class:input--has-left={IconLeft}
    class:input--has-right={IconRight}
    onkeydown={handleKeydown}
  />
  {#if IconRight}
    <span class="input-icon input-icon--right">
      <IconRight size={14} strokeWidth={1.5} />
    </span>
  {/if}
</div>

<style>
  .input-wrap {
    position: relative;
    display: flex;
    align-items: center;
    background: var(--bg-input);
    border: 1px solid var(--border-default);
    border-radius: var(--r-sm);
    height: 32px;
    transition: border-color var(--dur-fast) var(--ease-std);
  }

  .input-wrap--sm {
    height: 26px;
  }

  .input-wrap:focus-within {
    border-color: var(--teal);
    box-shadow: 0 0 0 1px var(--teal-soft);
  }

  .input-wrap--disabled {
    opacity: 0.5;
  }

  .input-wrap--mono .input {
    font-family: var(--font-mono);
  }

  .input {
    flex: 1;
    background: transparent;
    border: none;
    outline: none;
    box-shadow: none;
    color: var(--text-primary);
    font-family: var(--font-sans);
    font-size: 12px;
    padding: 0 12px;
    height: 100%;
    width: 100%;
  }

  .input::placeholder {
    color: var(--text-ghost);
  }

  .input--has-left {
    padding-left: 32px;
  }

  .input--has-right {
    padding-right: 32px;
  }

  .input-icon {
    position: absolute;
    display: flex;
    align-items: center;
    color: var(--text-ghost);
    pointer-events: none;
  }

  .input-icon--left {
    left: 10px;
  }

  .input-icon--right {
    right: 10px;
  }
</style>
