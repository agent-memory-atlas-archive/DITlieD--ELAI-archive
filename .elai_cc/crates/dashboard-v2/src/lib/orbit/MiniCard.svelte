<script lang="ts">
  // MiniCard — small bracketed DOM card pinned at a fixed VIEWPORT
  // corner position (NOT chasing the 3D marker). The connector to the
  // marker is rendered separately by OrbitCanvas's SVG overlay so all
  // connectors share one cohesive layer.
  //
  // Card always upright (camera-facing). The only "tilt" is a small CSS
  // perspective rotateY computed from the card's horizontal screen
  // position (caller passes the angle, capped ±5°).

  interface Line {
    text: string;
    classMod?: 'title' | 'mono' | 'mono-bright' | 'sans-dim';
  }

  interface Props {
    /** Header label rendered as `[TAG]`. Uppercase mono. */
    headerTag: string;
    /** 1-2 content lines. */
    lines: Line[];
    /** Absolute screen-space position (top-left corner of card, canvas-local px). */
    position: { x: number; y: number; visible: boolean };
    /** Y-axis perspective tilt in degrees (capped at ±5 by caller). */
    perspectiveAngle: number;
    /** Hidden during a camera warp — gates the fade-in. */
    cameraWarping?: boolean;
  }

  let {
    headerTag,
    lines,
    position,
    perspectiveAngle,
    cameraWarping = false,
  }: Props = $props();

  // Effective visibility — anchor must be on-screen AND camera not in a warp.
  const shouldShow = $derived(position.visible && !cameraWarping);

  const cardClass = $derived(`mini-card mini-card--${headerTag.toLowerCase().replace(/[^a-z]/g, '')}`);
</script>

<div
  class={cardClass}
  class:mini-card--visible={shouldShow}
  style:transform="translate3d({position.x}px, {position.y}px, 0) perspective(1400px) rotateY({perspectiveAngle}deg)"
  aria-hidden={!shouldShow}
>
  <span class="mc-corner mc-corner--tl" aria-hidden="true"></span>
  <span class="mc-corner mc-corner--tr" aria-hidden="true"></span>
  <span class="mc-corner mc-corner--bl" aria-hidden="true"></span>
  <span class="mc-corner mc-corner--br" aria-hidden="true"></span>
  <div class="mc-header">[{headerTag}]</div>
  {#each lines as line, i (i)}
    <div class="mc-line mc-line--{line.classMod ?? 'mono'}">{line.text}</div>
  {/each}
</div>

<style>
  /* Card itself — absolute, no fill, 1px teal-line border, bracket
     corners. Slightly wider so content breathes. */
  .mini-card {
    --mc-fade-in-ms: 420ms;
    --mc-fade-out-ms: 200ms;
    --mc-fade-ease: cubic-bezier(0.16, 1, 0.3, 1);

    position: absolute;
    top: 0;
    left: 0;
    z-index: 5;
    width: 220px;
    max-width: 220px;
    min-height: 56px;
    pointer-events: auto;
    border: 1px solid var(--teal-line);
    background: rgba(8, 16, 14, 0.62);
    padding: 8px 12px 9px;
    font-family: var(--font-mono);
    font-size: 11px;
    line-height: 1.4;
    color: var(--text-secondary);
    opacity: 0;
    transform-origin: 50% 50%;
    transition: opacity var(--mc-fade-in-ms) var(--mc-fade-ease);
    will-change: transform, opacity;
    /* Subtle outer glow so the card has weight against the bloom. */
    box-shadow: 0 0 24px rgba(30, 138, 122, 0.10);
  }
  .mini-card--visible { opacity: 1; }
  .mini-card:not(.mini-card--visible) {
    transition: opacity var(--mc-fade-out-ms) cubic-bezier(0.4, 0, 0.2, 1);
  }

  /* Bracket corners — 10×10 L-shapes, slightly larger so they read at
     viewport edges. */
  .mc-corner {
    position: absolute;
    width: 10px;
    height: 10px;
    pointer-events: none;
  }
  .mc-corner--tl { top: -1px; left: -1px;
    border-top: 1px solid var(--teal-bright);
    border-left: 1px solid var(--teal-bright);
  }
  .mc-corner--tr { top: -1px; right: -1px;
    border-top: 1px solid var(--teal-bright);
    border-right: 1px solid var(--teal-bright);
  }
  .mc-corner--bl { bottom: -1px; left: -1px;
    border-bottom: 1px solid var(--teal-bright);
    border-left: 1px solid var(--teal-bright);
  }
  .mc-corner--br { bottom: -1px; right: -1px;
    border-bottom: 1px solid var(--teal-bright);
    border-right: 1px solid var(--teal-bright);
  }

  .mc-header {
    font-family: var(--font-mono);
    font-size: 9.5px;
    text-transform: uppercase;
    letter-spacing: 0.14em;
    color: var(--teal-bright);
    margin-bottom: 4px;
  }
  .mc-line {
    font-family: var(--font-mono);
    font-size: 11px;
    color: var(--text-secondary);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }
  .mc-line--title {
    font-family: var(--font-sans);
    font-size: 13px;
    color: var(--text-primary);
    letter-spacing: 0.01em;
  }
  .mc-line--mono {
    color: var(--text-secondary);
  }
  .mc-line--mono-bright {
    color: var(--teal-bright);
  }
  .mc-line--sans-dim {
    font-family: var(--font-sans);
    color: var(--text-ghost);
    font-size: 11.5px;
  }

  @media (prefers-reduced-motion: reduce) {
    .mini-card { transition: opacity 80ms linear; }
    .mini-card:not(.mini-card--visible) { transition: opacity 80ms linear; }
  }
</style>
