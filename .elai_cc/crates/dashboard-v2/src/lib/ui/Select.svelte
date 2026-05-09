<script lang="ts">
  import { ChevronDown } from '@lucide/svelte';

  interface Option {
    value: string;
    label: string;
  }

  interface Props {
    value?: string;
    options: Option[];
    placeholder?: string;
  }

  let {
    value = $bindable(''),
    options,
    placeholder = 'Select…',
  }: Props = $props();

  let open = $state(false);
  let triggerEl = $state<HTMLButtonElement | null>(null);
  let focusedIndex = $state(-1);

  const selectedLabel = $derived(
    options.find((o) => o.value === value)?.label ?? placeholder
  );

  function toggle() {
    open = !open;
    if (open) focusedIndex = options.findIndex((o) => o.value === value);
  }

  function select(opt: Option) {
    value = opt.value;
    open = false;
    triggerEl?.focus();
  }

  function handleClickOutside(e: MouseEvent) {
    const target = e.target as Node;
    if (triggerEl && !triggerEl.closest('.select-wrap')?.contains(target)) {
      open = false;
    }
  }

  function handleKeydown(e: KeyboardEvent) {
    if (!open) {
      if (e.key === 'Enter' || e.key === ' ' || e.key === 'ArrowDown') {
        e.preventDefault();
        open = true;
        focusedIndex = options.findIndex((o) => o.value === value);
        if (focusedIndex < 0) focusedIndex = 0;
      }
      return;
    }
    if (e.key === 'Escape') {
      open = false;
      triggerEl?.focus();
    } else if (e.key === 'ArrowDown') {
      e.preventDefault();
      focusedIndex = Math.min(focusedIndex + 1, options.length - 1);
    } else if (e.key === 'ArrowUp') {
      e.preventDefault();
      focusedIndex = Math.max(focusedIndex - 1, 0);
    } else if (e.key === 'Enter' || e.key === ' ') {
      e.preventDefault();
      if (focusedIndex >= 0) select(options[focusedIndex]);
    }
  }
</script>

<svelte:window onclick={handleClickOutside} />

<div class="select-wrap">
  <button
    bind:this={triggerEl}
    class="select-trigger"
    role="combobox"
    aria-expanded={open}
    aria-haspopup="listbox"
    aria-controls="select-dropdown"
    onclick={toggle}
    onkeydown={handleKeydown}
  >
    <span class="select-value" class:select-value--placeholder={!value}>{selectedLabel}</span>
    <span class="select-chevron" class:select-chevron--open={open}>
      <ChevronDown size={12} strokeWidth={1.5} />
    </span>
  </button>

  {#if open}
    <div class="select-dropdown anim-enter-lift" id="select-dropdown" role="listbox" aria-label="Options">
      {#each options as opt, i}
        <div
          role="option"
          aria-selected={opt.value === value}
          class="select-option"
          class:select-option--active={opt.value === value}
          class:select-option--focused={i === focusedIndex}
          tabindex="-1"
          onclick={() => select(opt)}
          onkeydown={(e) => { if (e.key === 'Enter' || e.key === ' ') { e.preventDefault(); select(opt); } }}
          onmouseenter={() => { focusedIndex = i; }}
        >
          {opt.label}
        </div>
      {/each}
    </div>
  {/if}
</div>

<style>
  .select-wrap {
    position: relative;
    display: inline-block;
  }

  .select-trigger {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 8px;
    width: 100%;
    height: 32px;
    padding: 0 10px;
    background: var(--bg-input);
    border: 1px solid var(--border-default);
    border-radius: var(--r-sm);
    color: var(--text-primary);
    font-family: var(--font-sans);
    font-size: 12px;
    cursor: pointer;
    outline: none;
    transition: border-color var(--dur-fast) var(--ease-std);
  }

  .select-trigger:focus-visible {
    border-color: var(--teal);
    box-shadow: var(--ring);
  }

  .select-value--placeholder {
    color: var(--text-ghost);
  }

  .select-chevron {
    display: flex;
    align-items: center;
    color: var(--text-secondary);
    flex-shrink: 0;
    transition: transform var(--dur-fast) var(--ease-std);
  }

  .select-chevron--open {
    transform: rotate(180deg);
  }

  .select-dropdown {
    position: absolute;
    top: calc(100% + 4px);
    left: 0;
    right: 0;
    min-width: 100%;
    background: var(--bg-surface);
    border: 1px solid var(--border-default);
    border-radius: var(--r-sm);
    padding: 4px 0;
    z-index: 100;
  }

  .select-option {
    display: flex;
    align-items: center;
    height: 28px;
    padding: 0 12px;
    font-family: var(--font-sans);
    font-size: 12px;
    color: var(--text-primary);
    cursor: pointer;
    transition: background-color var(--dur-fast) var(--ease-std);
  }

  .select-option:hover,
  .select-option--focused {
    background: var(--bg-hover);
  }

  .select-option--active {
    color: var(--teal);
  }
</style>
