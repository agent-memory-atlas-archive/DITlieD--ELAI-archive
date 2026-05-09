<script lang="ts">
  interface Props {
    checked?: boolean;
    disabled?: boolean;
    label?: string;
    onchange?: (checked: boolean) => void;
  }

  let {
    checked = $bindable(false),
    disabled = false,
    label = '',
    onchange,
  }: Props = $props();

  function toggle() {
    if (disabled) return;
    checked = !checked;
    onchange?.(checked);
  }
</script>

<label class="toggle-wrap" class:toggle-wrap--disabled={disabled}>
  <button
    class="toggle"
    class:toggle--on={checked}
    role="switch"
    aria-checked={checked}
    aria-disabled={disabled}
    aria-label={label || 'toggle'}
    {disabled}
    onclick={toggle}
    type="button"
  >
    <span class="toggle-thumb"></span>
  </button>
  {#if label}
    <span class="toggle-label">{label}</span>
  {/if}
</label>

<style>
  .toggle-wrap {
    display: inline-flex;
    align-items: center;
    gap: 8px;
    cursor: pointer;
    user-select: none;
  }

  .toggle-wrap--disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }

  .toggle {
    position: relative;
    width: 32px;
    height: 18px;
    border-radius: var(--r-pill);
    border: none;
    background: var(--bg-input);
    cursor: pointer;
    padding: 0;
    outline: none;
    transition: background-color var(--dur-fast) var(--ease-std);
    flex-shrink: 0;
  }

  .toggle:focus-visible {
    box-shadow: var(--ring);
  }

  .toggle--on {
    background: var(--teal);
  }

  .toggle-thumb {
    position: absolute;
    top: 2px;
    left: 2px;
    width: 14px;
    height: 14px;
    border-radius: 50%;
    background: var(--text-secondary);
    transition: transform 120ms var(--ease-std), background-color var(--dur-fast) var(--ease-std);
  }

  .toggle--on .toggle-thumb {
    transform: translateX(14px);
    background: var(--bg-void);
  }

  .toggle-label {
    font-family: var(--font-sans);
    font-size: 12px;
    color: var(--text-secondary);
  }
</style>
