<script lang="ts">
  // OrbitTendrils — tendrils extending OUTSIDE the orbit during web research.
  // Each tendril is a curved path from the orbit's outer ring to a point in
  // deep space; a particle returns along the path coloured by source tier.
  // Phase D fleshes this out; Phase A is a passthrough that renders nothing
  // unless the fixture actually emits tendril events.

  import type { TendrilEvent } from './types';
  import { curveCubic } from './geometry';

  interface Props {
    cx: number;
    cy: number;
    outerRadius: number;
    events: TendrilEvent[];
  }
  let { cx, cy, outerRadius, events }: Props = $props();

  const TIER_TOKEN: Record<TendrilEvent['tier'], string> = {
    T1: 'var(--tier-1)',
    T2: 'var(--tier-2)',
    T3: 'var(--tier-3)',
    T4: 'var(--tier-4)',
    T5: 'var(--tier-5)',
  };

  // Deterministic angle per tendril id so the same source always extends
  // toward the same deep-space direction.
  function fnvHash(s: string): number {
    let h = 0x811c9dc5 >>> 0;
    for (let i = 0; i < s.length; i++) {
      h ^= s.charCodeAt(i);
      h = Math.imul(h, 0x01000193) >>> 0;
    }
    return h >>> 0;
  }
  function angleFor(id: string): number {
    return (fnvHash(id) / 0xFFFFFFFF) * Math.PI * 2;
  }

  interface Geo {
    e: TendrilEvent;
    sx: number; sy: number; tx: number; ty: number; path: string;
  }
  const geos = $derived<Geo[]>(events.map((e) => {
    const a = angleFor(e.id);
    const sx = cx + Math.cos(a) * outerRadius;
    const sy = cy + Math.sin(a) * outerRadius;
    const tx = cx + Math.cos(a) * (outerRadius + 160);
    const ty = cy + Math.sin(a) * (outerRadius + 160);
    return { e, sx, sy, tx, ty, path: curveCubic(sx, sy, tx, ty, 0.12) };
  }));
</script>

<g class="tendrils" aria-hidden="true">
  {#each geos as g (g.e.id)}
    <path d={g.path} fill="none" stroke={TIER_TOKEN[g.e.tier]} stroke-width={g.e.returned ? 1.4 : 0.8} opacity={g.e.returned ? 0.7 : 0.45} class="tendril" />
    {#if g.e.returned}
      <circle r="2" fill={TIER_TOKEN[g.e.tier]} class="particle" filter="url(#bloom-soft)">
        <animateMotion dur="1.6s" repeatCount="1" path={g.path} />
      </circle>
    {/if}
  {/each}
</g>

<style>
  .tendril { stroke-dasharray: 2 4; }
</style>
