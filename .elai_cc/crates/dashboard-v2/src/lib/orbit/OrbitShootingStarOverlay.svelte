<script lang="ts">
  // OrbitShootingStarOverlay — deterministic SVG shooting-star idle effect.
  // Fires every ~14s. Position seeded from hashId(task_id) + fire_counter.
  // No Math.random(). No SSB feed. Local clock only.
  //
  // Replaces the deleted OrbitShootingStar.svelte (removed in G3 session).
  // ShootingStarConfig type is in orbit/types.ts:175-180.

  import { lcg, hashId } from './lcg';

  interface Props {
    task_id: string;
    enabled?: boolean;
    fire_period_seconds?: number;
  }
  let {
    task_id,
    enabled = true,
    fire_period_seconds = 14,
  }: Props = $props();

  // LCG-seeded position jitter. Counter increments each fire.
  // Seed = hashId(task_id) XOR (counter * 0x9E3779B9) for unique star per fire.
  let fire_counter = $state(0);

  // Animate flag: true during the 1.5s arc; reset after.
  let animating = $state(false);

  // Star path — recomputed each fire from counter+seed.
  type StarPath = { x1: number; y1: number; x2: number; y2: number; cx: number; cy: number };
  let starPath = $state<StarPath>({ x1: 0, y1: 0, x2: 0, y2: 0, cx: 0, cy: 0 });

  // SVG viewport: 640×560 (matches OrbitCanvas reference size).
  const VW = 640;
  const VH = 560;
  const TRAIL = 90; // arc chord length in svg units

  function computePath(counter: number): StarPath {
    const seed = (hashId(task_id) ^ Math.imul(counter, 0x9E3779B9)) >>> 0;
    const rng = lcg(seed);
    // Start point: upper-left quadrant with jitter, avoiding the centre bloom.
    const x1 = 40  + rng() * (VW * 0.45);
    const y1 = 20  + rng() * (VH * 0.35);
    // Arc direction: mostly diagonal (down-right) ±15°.
    const angle = (Math.PI / 4) + (rng() - 0.5) * (Math.PI / 6);
    const x2 = x1 + TRAIL * Math.cos(angle);
    const y2 = y1 + TRAIL * Math.sin(angle);
    // Control point for slight curve (perpendicular offset).
    const mx = (x1 + x2) / 2;
    const my = (y1 + y2) / 2;
    const cx = mx - (y2 - y1) * 0.18;
    const cy = my + (x2 - x1) * 0.18;
    return { x1, y1, x2, y2, cx, cy };
  }

  function fire() {
    if (!enabled) return;
    fire_counter += 1;
    starPath = computePath(fire_counter);
    animating = true;
    // Clear after 1.6s (animation is 1.5s + 100ms safety margin).
    setTimeout(() => { animating = false; }, 1600);
  }

  $effect(() => {
    if (!enabled) return;
    const ms = fire_period_seconds * 1000;
    // Initial fire after one full period so the star doesn't appear immediately on mount.
    const id = setInterval(fire, ms);
    return () => clearInterval(id);
  });

  // Svelte-computed path string for the quadratic bezier arc.
  const pathD = $derived(
    `M ${starPath.x1} ${starPath.y1} Q ${starPath.cx} ${starPath.cy} ${starPath.x2} ${starPath.y2}`
  );
</script>

{#if animating}
  <svg
    class="shooting-star-overlay"
    viewBox="0 0 {VW} {VH}"
    xmlns="http://www.w3.org/2000/svg"
    aria-hidden="true"
  >
    <path
      class="star-arc"
      d={pathD}
    />
  </svg>
{/if}

<style>
  .shooting-star-overlay {
    position: absolute;
    inset: 0;
    width: 100%;
    height: 100%;
    pointer-events: none;
    z-index: 2;
    /* suppress sub-pixel AA flash on Chrome */
    will-change: opacity;
  }

  .star-arc {
    fill: none;
    stroke: var(--teal-line);
    stroke-width: 1.5px;
    stroke-linecap: round;
    /* opacity arc: 0 → 1 → 0 over 1.5s */
    animation: star-fade 1.5s ease-in-out forwards;
  }

  @keyframes star-fade {
    0%   { opacity: 0; stroke-dashoffset: 110; stroke-dasharray: 110; }
    20%  { opacity: 1; }
    80%  { opacity: 1; }
    100% { opacity: 0; stroke-dashoffset: 0;   stroke-dasharray: 110; }
  }
</style>
