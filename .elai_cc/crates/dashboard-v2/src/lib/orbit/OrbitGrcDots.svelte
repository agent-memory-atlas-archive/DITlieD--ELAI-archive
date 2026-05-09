<script lang="ts">
  // OrbitGrcDots — small dots orbiting the active node, one per recent GRC call.
  // Color encodes the step type (--grc-* tokens). Phase A renders a static
  // representation of the *current* step; Phase C will hook into a recent-N
  // history once a producer exists.

  import type { GrcStep } from './types';

  interface Props {
    cx: number;
    cy: number;
    active_step: GrcStep;
    radius?: number;
  }
  let { cx, cy, active_step, radius = 18 }: Props = $props();

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
</script>

<g class="grc-dots" aria-hidden="true">
  {#each DOTS as d, i (i)}
    {@const x = cx + Math.cos(d.angle) * radius}
    {@const y = cy + Math.sin(d.angle) * radius}
    <circle cx={x} cy={y} r={d.lead ? 1.8 : 1.4} fill={STEP_TOKEN[active_step]} opacity={d.opacity}
            class={d.lead ? 'grc-dot grc-dot--lead' : 'grc-dot'} />
  {/each}
  <!-- Slow rotation around active node -->
  <animateTransform attributeName="transform" type="rotate" from="0 {cx} {cy}" to="360 {cx} {cy}" dur="8s" repeatCount="indefinite" />
</g>

<style>
  .grc-dot--lead { filter: drop-shadow(0 0 2px currentColor); }
</style>
