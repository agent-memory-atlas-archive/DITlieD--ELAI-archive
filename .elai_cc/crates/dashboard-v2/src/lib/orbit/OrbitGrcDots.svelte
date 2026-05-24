<script lang="ts">
  // OrbitGrcDots — small dots orbiting the active node, one per recent GRC call.
  // Color encodes the step type (--grc-* tokens). Phase A renders a static
  // representation of the *current* step; Phase C will hook into a recent-N
  // history once a producer exists.
  //
  // E-33: a11y label + SVG <title> for the lead dot (hover tooltip surfaces
  // the GRC step name; was aria-hidden so screen readers had nothing to
  // announce). Reduced-motion guard hides the rotation when prefers-reduced
  // is on (CSS @media doesn't reach SVG animateTransform). Per plan task 4
  // wires ConfidenceTag near the lead dot (via foreignObject so the HTML
  // component renders inside SVG coordinate space) when the caller passes a
  // numeric confidence value from the GRC store.

  import type { GrcStep } from './types';
  import { motion } from '../state/motion.svelte';
  import ConfidenceTag from '../components/ConfidenceTag.svelte';

  interface Props {
    cx: number;
    cy: number;
    active_step: GrcStep;
    radius?: number;
    /** Optional 0..1 confidence; when present renders ConfidenceTag overlay. */
    confidence?: number;
  }
  let { cx, cy, active_step, radius = 18, confidence }: Props = $props();

  const STEP_TOKEN: Record<GrcStep, string> = {
    observe:     'var(--grc-observe)',
    hypothesize: 'var(--grc-hypothesize)',
    plan_action: 'var(--grc-plan_action)',
    act:         'var(--grc-act)',
    verify:      'var(--grc-verify)',
    converge:    'var(--grc-converge)',
    status:      'var(--grc-status)',
  };

  // Three dots circling at 120-degree offsets; the lead dot is the
  // current step, the trailing two represent recent history (greyed).
  const DOTS: Array<{ angle: number; opacity: number; lead: boolean }> = [
    { angle: 0,                   opacity: 1.0,  lead: true  },
    { angle: (2 * Math.PI) / 3,   opacity: 0.55, lead: false },
    { angle: (4 * Math.PI) / 3,   opacity: 0.30, lead: false },
  ];

  let allowMotion = $derived(motion.allow);
</script>

<g class="grc-dots" role="img" aria-label="GRC step: {active_step}">
  <title>GRC step: {active_step}</title>
  {#each DOTS as d, i (i)}
    {@const x = cx + Math.cos(d.angle) * radius}
    {@const y = cy + Math.sin(d.angle) * radius}
    <circle cx={x} cy={y} r={d.lead ? 1.8 : 1.4} fill={STEP_TOKEN[active_step]} opacity={d.opacity}
            class={d.lead ? 'grc-dot grc-dot--lead' : 'grc-dot'} />
  {/each}
  {#if allowMotion}
    <!-- Slow rotation around active node — suppressed under reduced-motion. -->
    <animateTransform attributeName="transform" type="rotate" from="0 {cx} {cy}" to="360 {cx} {cy}" dur="8s" repeatCount="indefinite" />
  {/if}
  {#if typeof confidence === 'number'}
    <!-- E-33 task 4: ConfidenceTag overlay near the active node. Rendered
         in a foreignObject so the HTML component lives inside the SVG
         coordinate space; sized generously enough to fit the binary tag. -->
    <foreignObject x={cx + radius + 4} y={cy - 8} width="92" height="20" class="grc-confidence">
      <ConfidenceTag value={confidence} size="sm" label={active_step} />
    </foreignObject>
  {/if}
</g>

<style>
  .grc-dot--lead { filter: drop-shadow(0 0 2px currentColor); }
</style>
