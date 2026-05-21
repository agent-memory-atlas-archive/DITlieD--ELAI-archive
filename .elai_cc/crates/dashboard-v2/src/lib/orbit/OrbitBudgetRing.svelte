<script lang="ts">
  // OrbitBudgetRing — context-budget donut around the centre node.
  // 8 slots, each rendered as an arc segment with width proportional to
  // total_tokens and fill proportional to used_tokens.

  import type { ContextBudgetSlot } from '../api/types';
  // E-33: motion guard is a no-op here today — OrbitBudgetRing has only
  // static SVG arcs, no <animate>/<animateTransform> elements. The motion
  // store import is intentionally omitted to keep the file lean; if a
  // future E-N adds arc-fill transitions, mirror the OrbitGrcDots pattern
  // (lines 13/41/52-55): import { motion } + let allowMotion = $derived(motion.allow)
  // + {#if allowMotion}...{/if}.

  interface Props {
    cx: number;
    cy: number;
    radius: number;
    slots: ContextBudgetSlot[];
  }
  let { cx, cy, radius, slots }: Props = $props();

  // Build arc paths for each slot.
  // total_tokens determines arc length; used_tokens / total_tokens determines fill.
  interface Arc {
    name: string;
    startAngle: number;
    endAngle: number;
    fillEnd: number;
    pct: number;
  }

  const arcs: Arc[] = $derived.by(() => {
    const total = slots.reduce((a, s) => a + s.total_tokens, 0);
    if (total === 0) return [];
    const out: Arc[] = [];
    let angle = -Math.PI / 2; // start at top
    for (const s of slots) {
      const span = (s.total_tokens / total) * Math.PI * 2 - 0.04; // small gap between segments
      const fillFrac = Math.max(0, Math.min(1, s.pct / 100));
      out.push({
        name: s.name,
        startAngle: angle,
        endAngle: angle + span,
        fillEnd: angle + span * fillFrac,
        pct: s.pct,
      });
      angle += span + 0.04;
    }
    return out;
  });

  function arcPath(start: number, end: number, r: number): string {
    const x1 = cx + Math.cos(start) * r;
    const y1 = cy + Math.sin(start) * r;
    const x2 = cx + Math.cos(end) * r;
    const y2 = cy + Math.sin(end) * r;
    const large = end - start > Math.PI ? 1 : 0;
    return `M ${x1} ${y1} A ${r} ${r} 0 ${large} 1 ${x2} ${y2}`;
  }
</script>

<g class="budget-ring" aria-label="Context budget" aria-hidden="true">
  {#each arcs as a (a.name)}
    <!-- track -->
    <path d={arcPath(a.startAngle, a.endAngle, radius)} fill="none" stroke="var(--bg-input)" stroke-width="2" stroke-linecap="round" />
    <!-- fill -->
    <path d={arcPath(a.startAngle, a.fillEnd, radius)} fill="none"
          stroke={a.pct > 90 ? 'var(--warn)' : 'var(--teal-line)'}
          stroke-width="2" stroke-linecap="round" />
  {/each}
</g>
