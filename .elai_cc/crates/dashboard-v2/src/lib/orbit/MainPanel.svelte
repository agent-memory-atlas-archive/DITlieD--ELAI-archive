<script lang="ts">
  // MainPanel — primary information surface for the V3 NodePanel design.
  // 480×300px, anchored to a "south of node" projection point. Always
  // upright (camera-facing) with a slight rotateX(-4deg) tilt so it
  // reads as a "tablet held up under the planet". 2×2 quadrant grid
  // + breadcrumb header strip + scan-line accent + bracketed corners
  // + thin connector to the node.
  //
  // Per-frame anchor updates mutate transform via a Svelte derived; no
  // CSS transition on transform — only on opacity, same pattern as the
  // legacy NodePanel.

  import type { MainPanelContent, MainQuadrant, MiniCardLine } from './node-panel-variants';

  interface Props {
    content: MainPanelContent;
    /** Top-centre anchor point (canvas-local px). The panel hangs
     *  vertically below this point — position.x is the horizontal
     *  centre, position.y is the panel's TOP edge. */
    position: { x: number; y: number; visible: boolean };
    /** Hidden during a camera warp (gates the fade-in). */
    cameraWarping?: boolean;
    /** True when the panel is "pinned" by click; renders the close button. */
    pinned?: boolean;
    /** Caller closes the pinned panel. */
    onClose?: () => void;
  }

  let {
    content,
    position,
    cameraWarping = false,
    pinned = false,
    onClose,
  }: Props = $props();

  const PANEL_W = 560;
  const PANEL_H = 220;
  // The anchor point is the TOP-CENTRE of the panel. position.x is the
  // viewport horizontal centre; position.y is the panel's top edge.
  const shouldShow = $derived(position.visible && !cameraWarping);

  function classForLine(l: MiniCardLine): string {
    return `mp-line mp-line--${l.classMod ?? 'mono'}`;
  }

  // Radial progress (BR quadrant). SVG circle with stroke-dasharray
  // driven by `pathLength=100` so the arc length math is just frac*100,
  // independent of the actual circumference.
  const RADIAL_R = 28;
  const RADIAL_PATH_LEN = 100;
  function radialDash(frac: number): string {
    const f = Math.max(0, Math.min(1, frac));
    return `${(f * RADIAL_PATH_LEN).toFixed(2)} ${RADIAL_PATH_LEN}`;
  }

  // Tier dot color helper (research SOURCES quadrant).
  function tierColor(t: 'T1' | 'T2' | 'T3'): string {
    if (t === 'T1') return 'var(--teal-bright)';
    if (t === 'T2') return 'var(--teal)';
    return 'var(--teal-line)';
  }

  function handleClose(ev: MouseEvent): void {
    ev.stopPropagation();
    onClose?.();
  }

  // Renders a single quadrant — kind-aware. The LIVE ACTIVITY / DIFF /
  // IDENTITY quadrants are plain line lists; PROGRESS adds the radial
  // arc + meta grid; SOURCES adds the tier-dot strip; PHASES is a
  // numbered list (already styled as mono-bright when active).
  function radialMeta(q: MainQuadrant): Array<[string, string]> {
    return q.metaRows ?? [];
  }
</script>

<div
  class="main-panel"
  class:main-panel--visible={shouldShow}
  class:main-panel--pinned={pinned}
  style:transform="translate3d({position.x - PANEL_W / 2}px, {position.y}px, 0) perspective(1600px) rotateX(-3deg)"
  data-variant={content.quadrants[0]?.kind ?? ''}
  role="dialog"
  aria-label="Main node detail"
  aria-hidden={!shouldShow}
>
  <span class="mp-corner mp-corner--tl" aria-hidden="true"></span>
  <span class="mp-corner mp-corner--tr" aria-hidden="true"></span>
  <span class="mp-corner mp-corner--bl" aria-hidden="true"></span>
  <span class="mp-corner mp-corner--br" aria-hidden="true"></span>

  <div class="mp-scan" aria-hidden="true"></div>

  <div class="mp-breadcrumb">{content.breadcrumb}</div>

  <div class="mp-grid">
    {#each content.quadrants as q, i (i)}
      <div class="mp-quad mp-quad--{q.kind}">
        <div class="mp-quad-tag">[{q.tag}]</div>
        {#if q.kind === 'progress'}
          <div class="mp-progress-row">
            <svg class="mp-radial" viewBox="-32 -32 64 64" aria-hidden="true">
              <circle
                class="mp-radial-track"
                r={RADIAL_R} cx="0" cy="0"
                fill="none"
                pathLength={RADIAL_PATH_LEN}
              />
              <circle
                class="mp-radial-arc"
                r={RADIAL_R} cx="0" cy="0"
                fill="none"
                pathLength={RADIAL_PATH_LEN}
                stroke-dasharray={radialDash(q.progressFrac ?? 0)}
                transform="rotate(-90)"
              />
              <text class="mp-radial-text" x="0" y="0" text-anchor="middle" dominant-baseline="central"
              >{Math.round((q.progressFrac ?? 0) * 100)}%</text>
            </svg>
            <div class="mp-meta">
              {#each radialMeta(q) as [k, v] (k)}
                <div class="mp-meta-k">{k}</div>
                <div class="mp-meta-v">{v}</div>
              {/each}
            </div>
          </div>
        {:else if q.kind === 'sourcesTier'}
          <div class="mp-tier-row">
            {#each q.tierDots ?? [] as t, di (di)}
              <span class="mp-tier-dot" style:background-color={tierColor(t)} title={t}></span>
            {/each}
          </div>
          {#each q.lines as l, i (i)}
            <div class={classForLine(l)}>{l.text}</div>
          {/each}
        {:else}
          {#each q.lines as l, i (i)}
            <div class={classForLine(l)}>{l.text}</div>
          {/each}
        {/if}
      </div>
    {/each}
  </div>

  {#if pinned}
    <button class="mp-close" onclick={handleClose} aria-label="Close panel" type="button">×</button>
  {/if}
</div>

<style>
  .main-panel {
    --mp-fade-in-ms: 420ms;
    --mp-fade-out-ms: 200ms;
    --mp-fade-ease: cubic-bezier(0.16, 1, 0.3, 1);

    position: absolute;
    top: 0;
    left: 0;
    z-index: 6;
    width: 560px;
    height: 220px;
    pointer-events: auto;
    border: 1.5px solid var(--teal-line);
    background: rgba(8, 16, 14, 0.62);
    color: var(--text-primary);
    font-family: var(--font-sans);
    font-size: 12px;
    line-height: 1.4;
    transform-origin: 50% 0%;
    opacity: 0;
    transition: opacity var(--mp-fade-in-ms) var(--mp-fade-ease);
    will-change: transform, opacity;
    display: flex;
    flex-direction: column;
    /* Subtle outer glow so the panel reads as its own object against
       the bloom — same treatment as the mini-cards. */
    box-shadow: 0 0 32px rgba(30, 138, 122, 0.10);
  }
  .main-panel--visible { opacity: 1; }
  .main-panel:not(.main-panel--visible) {
    transition: opacity var(--mp-fade-out-ms) cubic-bezier(0.4, 0, 0.2, 1);
  }

  /* Bracket corners — 16×16. */
  .mp-corner {
    position: absolute;
    width: 16px;
    height: 16px;
    pointer-events: none;
  }
  .mp-corner--tl { top: -1.5px; left: -1.5px;
    border-top: 1.5px solid var(--teal-bright);
    border-left: 1.5px solid var(--teal-bright);
  }
  .mp-corner--tr { top: -1.5px; right: -1.5px;
    border-top: 1.5px solid var(--teal-bright);
    border-right: 1.5px solid var(--teal-bright);
  }
  .mp-corner--bl { bottom: -1.5px; left: -1.5px;
    border-bottom: 1.5px solid var(--teal-bright);
    border-left: 1.5px solid var(--teal-bright);
  }
  .mp-corner--br { bottom: -1.5px; right: -1.5px;
    border-bottom: 1.5px solid var(--teal-bright);
    border-right: 1.5px solid var(--teal-bright);
  }

  .mp-scan {
    position: absolute;
    top: 8px;
    right: 24px;
    width: 28px;
    height: 1px;
    background: var(--teal-soft);
    opacity: 0.55;
    pointer-events: none;
    animation: mp-scan 2400ms linear infinite;
  }
  @keyframes mp-scan {
    0%   { transform: translateY(0); }
    50%  { transform: translateY(12px); }
    100% { transform: translateY(0); }
  }
  @media (prefers-reduced-motion: reduce) {
    .mp-scan { animation: none; }
  }

  .mp-breadcrumb {
    flex: 0 0 auto;
    padding: 7px 14px 6px;
    font-family: var(--font-mono);
    font-size: 9.5px;
    text-transform: uppercase;
    letter-spacing: 0.14em;
    color: var(--text-ghost);
    border-bottom: 1px solid var(--teal-line);
  }

  .mp-grid {
    flex: 1 1 auto;
    display: grid;
    grid-template-columns: 1fr 1fr;
    grid-template-rows: 1fr 1fr;
    /* Hairline dividers via background-image on the grid container. */
    background:
      linear-gradient(to right, transparent 0, transparent calc(50% - 0.5px),
                                 var(--teal-line) calc(50% - 0.5px),
                                 var(--teal-line) calc(50% + 0.5px),
                                 transparent calc(50% + 0.5px), transparent 100%) no-repeat,
      linear-gradient(to bottom, transparent 0, transparent calc(50% - 0.5px),
                                 var(--teal-line) calc(50% - 0.5px),
                                 var(--teal-line) calc(50% + 0.5px),
                                 transparent calc(50% + 0.5px), transparent 100%) no-repeat;
    background-size: 100% 100%, 100% 100%;
  }
  .mp-quad {
    padding: 8px 12px 10px;
    overflow: hidden;
    min-width: 0;
  }
  .mp-quad-tag {
    font-family: var(--font-mono);
    font-size: 9.5px;
    text-transform: uppercase;
    letter-spacing: 0.12em;
    color: var(--teal-bright);
    margin-bottom: 5px;
  }

  .mp-line {
    font-family: var(--font-mono);
    font-size: 10px;
    line-height: 1.4;
    color: var(--text-secondary);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
    margin-bottom: 1px;
  }
  .mp-line--title {
    font-family: var(--font-sans);
    font-size: 13px;
    color: var(--text-primary);
    letter-spacing: 0.01em;
    margin-bottom: 3px;
  }
  .mp-line--mono { color: var(--text-secondary); }
  .mp-line--mono-bright { color: var(--teal-bright); }
  .mp-line--sans-dim {
    font-family: var(--font-sans);
    color: var(--text-ghost);
    font-size: 11.5px;
  }

  /* PROGRESS quadrant: SVG radial + meta grid side-by-side. */
  .mp-progress-row {
    display: grid;
    grid-template-columns: 72px 1fr;
    column-gap: 12px;
    align-items: center;
  }
  .mp-radial {
    width: 72px;
    height: 72px;
  }
  .mp-radial-track {
    stroke: var(--teal-line);
    stroke-width: 2;
    opacity: 0.45;
  }
  .mp-radial-arc {
    stroke: var(--teal-bright);
    stroke-width: 2.5;
    stroke-linecap: round;
    transform-origin: 0 0;
  }
  .mp-radial-text {
    font-family: var(--font-mono);
    font-size: 10px;
    fill: var(--text-primary);
    letter-spacing: 0.04em;
  }
  .mp-meta {
    display: grid;
    grid-template-columns: minmax(0, auto) 1fr;
    column-gap: 8px;
    row-gap: 1px;
    font-family: var(--font-mono);
    font-size: 10px;
  }
  .mp-meta-k { color: var(--text-ghost); }
  .mp-meta-v { color: var(--teal-bright); white-space: nowrap; overflow: hidden; text-overflow: ellipsis; }

  /* SOURCES tier-dot strip. */
  .mp-tier-row {
    display: flex;
    align-items: center;
    gap: 5px;
    margin-bottom: 5px;
  }
  .mp-tier-dot {
    width: 8px;
    height: 8px;
    border-radius: 50%;
    box-shadow: 0 0 0 1px rgba(0, 0, 0, 0.35) inset;
  }

  .mp-close {
    position: absolute;
    top: 4px;
    right: 6px;
    width: 18px;
    height: 18px;
    background: transparent;
    border: 0;
    color: var(--text-ghost);
    font: 14px/1 var(--font-mono);
    cursor: pointer;
    padding: 0;
    z-index: 2;
  }
  .mp-close:hover { color: var(--text-primary); }
  .mp-close:focus-visible { outline: 1px solid var(--teal-line); outline-offset: 1px; }

  @media (prefers-reduced-motion: reduce) {
    .main-panel { transition: opacity 80ms linear; }
    .main-panel:not(.main-panel--visible) { transition: opacity 80ms linear; }
  }
</style>
