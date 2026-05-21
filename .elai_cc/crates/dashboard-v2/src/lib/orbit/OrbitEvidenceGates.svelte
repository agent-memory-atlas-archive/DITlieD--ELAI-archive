<script lang="ts">
  // OrbitEvidenceGates — 7 gates evenly distributed on a ring between
  // the phase orbit and the step orbit. Verdict drives color + lock state.
  //
  // E-33: shape-distinct verdict glyph + Wong-palette color so the signal
  // never depends on hue alone (WCAG 2.2 1.4.1).

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
      // E-33: signal layer uses Wong tones, not the legacy gate-* mapping.
      case 'pass':    return 'var(--confidence-verified)';   // green
      case 'block':   return 'var(--confidence-needs-evidence)'; // vermilion
      case 'warn':    return 'var(--confidence-low)';        // orange
      case 'pending': return 'var(--gate-pending)';          // ghost
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
      <title>{g.name}: {g.verdict}</title>
      <!-- E-33: shape-distinct verdict glyph (color is a REDUNDANT signal). -->
      <circle cx={x} cy={y} r="3.2" fill="var(--bg-base)" stroke={color(g.verdict)} stroke-width="1.2" opacity={g.verdict === 'pending' ? 0.6 : 1} />
      {#if g.verdict === 'pass'}
        <!-- check mark -->
        <path d="M {x - 1.6} {y} L {x - 0.3} {y + 1.2} L {x + 1.6} {y - 1.4}" stroke={color(g.verdict)} stroke-width="1.0" fill="none" stroke-linecap="round" stroke-linejoin="round" />
      {:else if g.verdict === 'block'}
        <!-- cross / X -->
        <path d="M {x - 1.4} {y - 1.4} L {x + 1.4} {y + 1.4} M {x + 1.4} {y - 1.4} L {x - 1.4} {y + 1.4}" stroke={color(g.verdict)} stroke-width="1.0" stroke-linecap="round" />
      {:else if g.verdict === 'warn'}
        <!-- exclamation: vertical bar + dot -->
        <rect x={x - 0.4} y={y - 1.6} width="0.8" height="2.0" fill={color(g.verdict)} />
        <circle cx={x} cy={y + 1.4} r="0.5" fill={color(g.verdict)} />
      {:else}
        <!-- pending: faint hollow ring (rendered above by the outer circle) -->
        <circle cx={x} cy={y} r="0.6" fill={color(g.verdict)} opacity="0.6" />
      {/if}
      {#if hovered === g.id}
        <text x={x + 6} y={y + 3} class="gate-label"
              paint-order="stroke" stroke="var(--bg-base)" stroke-width="2" stroke-linejoin="round">
          {g.name.toLowerCase()}: {g.verdict}
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
