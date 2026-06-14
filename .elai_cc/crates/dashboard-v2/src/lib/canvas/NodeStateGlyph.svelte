<script lang="ts">
  // NodeStateGlyph.svelte — the shared quinary state glyph (DESIGN §6).
  //
  // Shape carries state, color only whispers; the palette has NO status
  // colors. Encoded redundantly (glyph + fill/border + position) so the
  // state survives a squint and a monochrome display (WCAG 1.4.1):
  //   done     ✓  teal-deep filled, content RECEDES
  //   active   ●  teal outline + slow pulse — the only moving thing
  //   pending  ○  dashed hairline border, dim
  //   warn     ▲  shape only, drawn in --text (no own color; amber is dead)
  //   fail     ✕  MONO INVERSION: light chip (--text bg, --void glyph)
  //
  // This is the one canonical glyph component — never ad-hoc unicode per
  // view (DESIGN §11). Every canvas surface (PlanMode, ResearchMode, the
  // node-story panel, the system-mode renderers) renders state through it.

  // The five canonical states (source of truth: canvas_state.ts, so plain
  // tsc can resolve the type). Callers normalise via toQuinary().
  import type { QuinaryState } from './canvas_state';
  export type { QuinaryState };

  interface Props {
    state: QuinaryState;
    /** Glyph box size in px (default 14). */
    size?: number;
    /** Optional accessible label override (defaults to the state name). */
    label?: string;
  }

  let { state, size = 14, label }: Props = $props();

  const GLYPH: Record<QuinaryState, string> = {
    done: '✓',
    active: '●',
    pending: '○',
    warn: '▲',
    fail: '✕',
  };
</script>

<span
  class="qstate qstate--{state}"
  style:--q-size="{size}px"
  role="img"
  aria-label={label ?? state}
  title={label ?? state}
>{GLYPH[state]}</span>

<style>
  .qstate {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    width: var(--q-size);
    height: var(--q-size);
    flex: none;
    font-family: var(--font-mono);
    font-size: calc(var(--q-size) * 0.72);
    line-height: 1;
    border-radius: 50%;
    box-sizing: border-box;
  }

  /* done — teal-deep filled, glyph recedes onto the fill */
  .qstate--done {
    background: var(--d-teal-deep);
    color: var(--bg);
    opacity: 0.8;
  }

  /* active — teal outline ring + the single sanctioned pulse */
  .qstate--active {
    color: var(--d-teal-bright);
    border: 1px solid var(--d-teal-soft);
    background: var(--d-teal-faint);
    animation: qpulse 1.6s var(--ease-std) infinite;
  }

  /* pending — dashed hairline, dim */
  .qstate--pending {
    color: var(--dim);
    border: 1px dashed var(--line2);
    background: transparent;
  }

  /* warn — shape only, drawn in --text, no own hue (amber is dead) */
  .qstate--warn {
    color: var(--text);
    background: transparent;
    border-radius: 0;
  }

  /* fail — MONO INVERSION: light chip, dark glyph (the loudest thing) */
  .qstate--fail {
    background: var(--text);
    color: var(--void, #030404);
    border-radius: 2px;
    font-weight: 700;
  }

  @keyframes qpulse {
    0%, 100% { opacity: 1; }
    50%      { opacity: 0.4; }
  }

  @media (prefers-reduced-motion: reduce) {
    .qstate--active { animation: none; }
  }
</style>
