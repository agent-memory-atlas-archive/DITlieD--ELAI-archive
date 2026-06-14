<script lang="ts">
  // CanvasModeBar.svelte — shared chrome for the canvas: mode bar + zoom +
  // search + follow-agent toggle. (H-12; restyled to DESIGN.md 2026-06-14)
  //
  // Restyle: flat-matte work surface on --bg, role tokens only, and the
  // center->out UNDERLINE idiom for the mode nav (DESIGN §13 / brief I3 —
  // no dot, no box; hover draws a teal line from center via scaleX; the
  // active mode keeps the full line). The mode bar sets the plain mode flag
  // on canvas.svelte.ts — NOT an FSM substate. fsm.rs is UNTOUCHED.
  //
  // Modes: code / plan / research / arch / design wired; system is the
  // J-31 UI shell (mock-fixture surface, backend pending — see SystemMode).

  import type { CanvasMode } from '../state/canvas.svelte';
  import { canvas } from '../state/canvas.svelte';
  import { canvasActivity } from '../state/canvasActivity.svelte';

  /** Mode bar items — wired flag controls which are interactive now. */
  const MODE_ITEMS: Array<{ mode: CanvasMode; label: string; wired: boolean }> = [
    { mode: 'code',     label: 'CODE',     wired: true },
    { mode: 'plan',     label: 'PLAN',     wired: true },
    { mode: 'research', label: 'RESEARCH', wired: true },
    { mode: 'arch',     label: 'ARCH',     wired: true },
    { mode: 'design',   label: 'DESIGN',   wired: true },
    { mode: 'system',   label: 'SYSTEM',   wired: true }, // J-31 UI shell
  ];

  interface Props {
    onZoomIn?: () => void;
    onZoomOut?: () => void;
    onZoomFit?: () => void;
    onSearch?: () => void;
  }

  let { onZoomIn, onZoomOut, onZoomFit, onSearch }: Props = $props();

  /** Hover-reveal state: true when the user hovers over the chrome bar. */
  let barHovered = $state<boolean>(false);

  function switchMode(m: CanvasMode, wired: boolean) {
    if (!wired) return;
    canvas.setMode(m);
  }

  function toggleFollowAgent() {
    canvasActivity.setFollowAgent(!canvasActivity.followAgent);
  }
</script>

<!-- CanvasModeBar — hover-reveal per CANVAS §7 -->
<div
  class="canvas-chrome"
  class:visible={barHovered}
  role="toolbar"
  tabindex="-1"
  aria-label="Canvas mode bar"
  onmouseenter={() => { barHovered = true; }}
  onmouseleave={() => { barHovered = false; }}
>
  <!-- Mode nav — center->out underline idiom (no dot, no box) -->
  <div class="mode-bar" role="group" aria-label="Canvas mode">
    {#each MODE_ITEMS as item (item.mode)}
      <button
        class="mode-btn"
        class:active={canvas.mode === item.mode}
        class:dimmed={!item.wired}
        disabled={!item.wired}
        aria-pressed={canvas.mode === item.mode}
        title={`Switch to ${item.label} mode`}
        onclick={() => switchMode(item.mode, item.wired)}
      >
        {item.label}
        <span class="underline" aria-hidden="true"></span>
      </button>
    {/each}
  </div>

  <div class="spacer"></div>

  <!-- Zoom controls -->
  <div class="chrome-group" role="group" aria-label="Zoom">
    <button class="chrome-btn" title="Zoom in" aria-label="Zoom in" onclick={() => onZoomIn?.()}>+</button>
    <button class="chrome-btn" title="Zoom out" aria-label="Zoom out" onclick={() => onZoomOut?.()}>−</button>
    <button class="chrome-btn chrome-btn-text" title="Fit graph to window" aria-label="Fit to window" onclick={() => onZoomFit?.()}>fit</button>
  </div>

  <button class="chrome-btn chrome-btn-text" title="Search nodes (Cmd+F)" aria-label="Search canvas" onclick={() => onSearch?.()}>search</button>

  <button
    class="chrome-btn chrome-btn-text"
    class:follow-active={canvasActivity.followAgent}
    title={canvasActivity.followAgent ? 'Following agent — click to stop' : 'Follow agent'}
    aria-label="Follow agent"
    aria-pressed={canvasActivity.followAgent}
    onclick={toggleFollowAgent}
  >{canvasActivity.followAgent ? 'following' : 'follow'}</button>
</div>

<style>
  .canvas-chrome {
    position: absolute;
    top: 0;
    left: 0;
    right: 0;
    z-index: var(--z-shell-chrome, 10);
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 6px 12px;
    background: var(--bg);
    border-bottom: 1px solid var(--line);
    opacity: 0.2;
    transition: opacity var(--dur-base) var(--ease-out);
    pointer-events: none;
  }
  .canvas-chrome:hover,
  .canvas-chrome.visible {
    opacity: 1;
    pointer-events: auto;
  }

  .spacer { flex: 1 1 auto; }

  /* Mode nav — underline idiom */
  .mode-bar {
    display: flex;
    align-items: center;
    gap: 2px;
  }

  .mode-btn {
    position: relative;
    padding: 4px 11px 6px;
    font-family: var(--font-mono);
    font-size: 10px;
    letter-spacing: 0.16em;
    background: none;
    color: var(--dim);
    border: none;
    cursor: pointer;
    transition: color var(--dur-fast) var(--ease-out);
  }
  .mode-btn:hover:not(:disabled):not(.active) { color: var(--text); }
  .mode-btn.active { color: var(--d-teal-bright); }
  .mode-btn.dimmed,
  .mode-btn:disabled { opacity: 0.4; cursor: not-allowed; }

  /* center->out underline */
  .underline {
    position: absolute;
    left: 9px;
    right: 9px;
    bottom: 1px;
    height: 1.5px;
    background: var(--d-teal-bright);
    transform: scaleX(0);
    transform-origin: center;
    transition: transform var(--dur-base) var(--ease-out);
  }
  .mode-btn:hover:not(:disabled) .underline { transform: scaleX(1); }
  .mode-btn.active .underline { transform: scaleX(1); }

  .chrome-group { display: flex; gap: 2px; }

  .chrome-btn {
    padding: 3px 7px;
    font-family: var(--font-mono);
    font-size: 11px;
    background: var(--panel);
    color: var(--dim);
    border: 1px solid var(--line);
    border-radius: var(--r-xs);
    cursor: pointer;
    transition: background var(--dur-fast) var(--ease-out), color var(--dur-fast) var(--ease-out);
    line-height: 1;
  }
  .chrome-btn:hover { background: var(--raised); color: var(--text); }

  .chrome-btn-text {
    font-size: 10px;
    letter-spacing: 0.04em;
  }

  .follow-active {
    background: var(--d-teal-ghost);
    color: var(--d-teal-bright);
    border-color: var(--d-teal-soft);
  }

  @media (prefers-reduced-motion: reduce) {
    .underline { transition: none; }
  }
</style>
