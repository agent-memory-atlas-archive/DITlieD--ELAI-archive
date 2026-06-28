<script lang="ts">
  // E-33: Binary-confidence tag. Per Fuselab 2026, users decided faster
  // with "confident" vs "uncertain" than with raw numeric scores. We map
  // four states (needs-evidence, low, ok, verified) using shape + text
  // + Wong-palette color so the signal is redundant (WCAG 2.2 1.4.1).
  // Hover tooltip surfaces the underlying numeric value for power users.

  type State = 'needs-evidence' | 'low' | 'ok' | 'verified';

  interface Props {
    value: number; // 0..1
    label?: string; // optional override label
    title?: string; // hover tooltip override; default = raw value
    size?: 'sm' | 'md';
  }

  let { value, label, title, size = 'sm' }: Props = $props();

  function classify(v: number): State {
    if (v < 0.25) return 'needs-evidence';
    if (v < 0.5) return 'low';
    if (v < 0.85) return 'ok';
    return 'verified';
  }

  const STATE_LABEL: Record<State, string> = {
    'needs-evidence': 'needs evidence',
    low: 'low',
    ok: 'ok',
    verified: 'verified',
  };

  // Shape-distinct glyph per state. Color alone is never the signal.
  const STATE_GLYPH: Record<State, string> = {
    'needs-evidence': '?', // open question
    low: '·', // small dot
    ok: '•', // filled dot
    verified: '✓', // check
  };

  let state: State = $derived(classify(value));
  let tooltip: string = $derived(title ?? `confidence: ${value.toFixed(2)}`);
  let displayLabel: string = $derived(label ?? STATE_LABEL[state]);
</script>

<span
  class="ctag ctag--{state} ctag--{size}"
  title={tooltip}
  role="status"
  aria-label="confidence {STATE_LABEL[state]} ({value.toFixed(2)})"
>
  <span class="ctag-glyph" aria-hidden="true">{STATE_GLYPH[state]}</span>
  <span class="ctag-label">{displayLabel}</span>
</span>

<style>
  .ctag {
    display: inline-flex;
    align-items: center;
    gap: 4px;
    padding: 2px 6px;
    border-radius: var(--r-pill);
    font-family: var(--font-mono);
    font-size: 10px;
    line-height: 1.2;
    border: 1px solid currentColor;
    background: var(--bg-surface);
    cursor: default;
    user-select: none;
    white-space: nowrap;
  }
  .ctag--md {
    font-size: 10px;
    padding: 3px 8px;
  }
  .ctag-glyph {
    font-weight: 600;
  }
  .ctag-label {
    color: var(--text-primary);
  }
  .ctag--needs-evidence {
    color: var(--confidence-needs-evidence);
  }
  .ctag--low {
    color: var(--confidence-low);
  }
  .ctag--ok {
    color: var(--confidence-ok);
  }
  .ctag--verified {
    color: var(--confidence-verified);
  }
</style>
