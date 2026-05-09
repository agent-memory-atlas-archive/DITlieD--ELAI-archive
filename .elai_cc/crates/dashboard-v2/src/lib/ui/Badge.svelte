<script lang="ts">
  type Size = 'xs' | 'sm';

  interface Props {
    tone?: string; // 'teal' | 'steel' | 'warn' | 'fail' | 'ghost' | 'state-<name>'
    size?: Size;
    children?: import('svelte').Snippet;
  }

  let {
    tone = 'ghost',
    size = 'sm',
    children,
  }: Props = $props();

  // Resolve tone to CSS vars
  const colorMap: Record<string, { fg: string; bg: string }> = {
    teal:    { fg: 'var(--teal)',           bg: 'var(--teal-ghost)' },
    steel:   { fg: 'var(--steel)',          bg: 'var(--steel-ghost)' },
    warn:    { fg: 'var(--warn)',           bg: 'rgba(195,154,58,0.12)' },
    fail:    { fg: 'var(--fail)',           bg: 'rgba(176,69,69,0.12)' },
    ghost:   { fg: 'var(--text-ghost)',     bg: 'transparent' },
  };

  // State-prefixed tones map to --state-* tokens
  const resolved = $derived(() => {
    if (tone.startsWith('state-')) {
      const state = tone.slice(6);
      return { fg: `var(--state-${state})`, bg: 'var(--teal-ghost)' };
    }
    return colorMap[tone] ?? colorMap['ghost'];
  });

  const fg = $derived(resolved().fg);
  const bg = $derived(resolved().bg);
</script>

<span
  class="badge badge--{size}"
  style="color:{fg}; background:{bg};"
>
  {#if children}
    {@render children()}
  {/if}
</span>

<style>
  .badge {
    display: inline-flex;
    align-items: center;
    border-radius: var(--r-pill);
    font-family: var(--font-mono);
    font-weight: 500;
    letter-spacing: 0.06em;
    white-space: nowrap;
  }

  .badge--xs {
    font-size: 10px;
    padding: 1px 5px;
  }

  .badge--sm {
    font-size: 11px;
    padding: 2px 7px;
  }
</style>
