<script lang="ts">
  // OrbitEvidenceGates — 7 gates evenly distributed on a ring between
  // the phase orbit and the step orbit. Verdict drives color + lock state.

  import type { EvidenceGate } from '../api/types';

  interface Props {
    gates: EvidenceGate[];
    cx: number;
    cy: number;
    radius: number;
  }
  let { gates, cx, cy, radius }: Props = $props();

  function color(v: EvidenceGate['verdict']): string {
    switch (v) {
      case 'pass':    return 'var(--gate-pass)';
      case 'block':   return 'var(--gate-block)';
      case 'warn':    return 'var(--gate-warn)';
      case 'pending': return 'var(--gate-pending)';
    }
  }

  let hovered = $state<string | null>(null);

  // distribute gates evenly around the ring, starting at top.
  function angleFor(i: number, n: number): number {
    return -Math.PI / 2 + (i / Math.max(n, 1)) * Math.PI * 2;
  }
</script>

<g class="evidence-gates" aria-label="Evidence gates">
  {#each gates as g, i (g.id)}
    {@const a = angleFor(i, gates.length)}
    {@const x = cx + Math.cos(a) * radius}
    {@const y = cy + Math.sin(a) * radius}
    <g class="gate"
       onmouseenter={() => { hovered = g.id; }} onmouseleave={() => { hovered = null; }}
       role="button" tabindex="0" aria-label="{g.name}: {g.verdict}">
      <!-- chevron-style mark: a tiny hexagonal pip -->
      <circle cx={x} cy={y} r="3.2" fill="var(--bg-base)" stroke={color(g.verdict)} stroke-width="1.2" opacity={g.verdict === 'pending' ? 0.6 : 1} />
      <circle cx={x} cy={y} r={g.verdict === 'pass' ? 1.6 : 0} fill={color(g.verdict)} />
      {#if hovered === g.id}
        <text x={x + 6} y={y + 3} class="gate-label"
              paint-order="stroke" stroke="var(--bg-base)" stroke-width="2" stroke-linejoin="round">
          {g.name.toLowerCase()}
        </text>
      {/if}
    </g>
  {/each}
</g>

<style>
  .gate { cursor: default; }
  .gate-label {
    font-family: var(--font-mono);
    font-size: 8px;
    fill: var(--text-secondary);
  }
</style>
