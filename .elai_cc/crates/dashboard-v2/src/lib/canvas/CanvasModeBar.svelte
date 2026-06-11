<script lang="ts">
  // CanvasModeBar.svelte — shared chrome for the canvas: mode bar + zoom +
  // search + inspector + follow-agent toggle. (H-12)
  //
  // The mode bar is a segmented control (hover-reveal per CANVAS-WORKBENCH-DESIGN.md
  // §7 / §8). It sets the plain mode flag on canvas.svelte.ts — NOT an FSM
  // substate. fsm.rs is UNTOUCHED (CANVAS §11).
  //
  // Modes wired in H-12: code / plan / research.
  // Modes wired in H-13/H-14: arch / design (shown dimmed until wired).

  import type { CanvasMode } from '../state/canvas.svelte';
  import { canvas } from '../state/canvas.svelte';
  import { canvasActivity } from '../state/canvasActivity.svelte';

  // ---------------------------------------------------------------------------
  // Named constants (no magic strings)
  // ---------------------------------------------------------------------------

  /** Mode bar items — wired flag controls which are interactive now. */
  const MODE_ITEMS: Array<{ mode: CanvasMode; label: string; wired: boolean }> = [
    { mode: 'code',     label: 'Code',     wired: true  },
    { mode: 'plan',     label: 'Plan',     wired: true  },
    { mode: 'research', label: 'Research', wired: true  },
    { mode: 'arch',     label: 'Arch',     wired: true  }, // H-13: wired
    { mode: 'design',   label: 'Design',   wired: false },
  ];

  // ---------------------------------------------------------------------------
  // Props
  // ---------------------------------------------------------------------------

  interface Props {
    /** Callback when the user requests a zoom-in. */
    onZoomIn?: () => void;
    /** Callback when the user requests a zoom-out. */
    onZoomOut?: () => void;
    /** Callback when the user requests a zoom-fit. */
    onZoomFit?: () => void;
    /** Callback when the user opens search (Cmd+F). */
    onSearch?: () => void;
  }

  let {
    onZoomIn,
    onZoomOut,
    onZoomFit,
    onSearch,
  }: Props = $props();

  // ---------------------------------------------------------------------------
  // State
  // ---------------------------------------------------------------------------

  /** Hover-reveal state: true when the user hovers over the chrome bar. */
  let barHovered = $state<boolean>(false);

  // ---------------------------------------------------------------------------
  // Mode switch
  // ---------------------------------------------------------------------------

  function switchMode(m: CanvasMode, wired: boolean) {
    if (!wired) return; // deferred mode — H-13/H-14 scope
    canvas.setMode(m);
  }

  // ---------------------------------------------------------------------------
  // Follow-agent toggle
  // ---------------------------------------------------------------------------

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
  <!-- Mode segmented control -->
  <div class="mode-bar" role="group" aria-label="Canvas mode">
    {#each MODE_ITEMS as item (item.mode)}
      <button
        class="mode-btn"
        class:active={canvas.mode === item.mode}
        class:dimmed={!item.wired}
        disabled={!item.wired}
        aria-pressed={canvas.mode === item.mode}
        aria-disabled={!item.wired}
        title={item.wired ? `Switch to ${item.label} mode` : `${item.label} mode — coming in H-13/H-14`}
        onclick={() => switchMode(item.mode, item.wired)}
      >
        {item.label}
      </button>
    {/each}
  </div>

  <!-- Zoom controls -->
  <div class="chrome-group zoom-group" role="group" aria-label="Zoom">
    <button
      class="chrome-btn"
      title="Zoom in"
      aria-label="Zoom in"
      onclick={() => onZoomIn?.()}
    >+</button>
    <button
      class="chrome-btn"
      title="Zoom out"
      aria-label="Zoom out"
      onclick={() => onZoomOut?.()}
    >−</button>
    <button
      class="chrome-btn chrome-btn-text"
      title="Fit graph to window"
      aria-label="Fit to window"
      onclick={() => onZoomFit?.()}
    >fit</button>
  </div>

  <!-- Search -->
  <button
    class="chrome-btn chrome-btn-text"
    title="Search nodes (Cmd+F)"
    aria-label="Search canvas"
    onclick={() => onSearch?.()}
  >search</button>

  <!-- Follow-agent toggle -->
  <button
    class="chrome-btn chrome-btn-text"
    class:follow-active={canvasActivity.followAgent}
    title={canvasActivity.followAgent ? 'Following agent — click to stop' : 'Follow agent'}
    aria-label="Follow agent"
    aria-pressed={canvasActivity.followAgent}
    onclick={toggleFollowAgent}
  >
    {canvasActivity.followAgent ? 'following' : 'follow'}
  </button>
</div>

<style>
  .canvas-chrome {
    position: absolute;
    top: 0;
    left: 0;
    right: 0;
    z-index: 10;
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 6px 10px;
    background: var(--bg-surface);
    border-bottom: 1px solid var(--border-subtle);
    opacity: 0.15;
    transition: opacity 150ms ease;
    pointer-events: none;
  }

  .canvas-chrome:hover,
  .canvas-chrome.visible {
    opacity: 1;
    pointer-events: auto;
  }

  /* Mode segmented control */
  .mode-bar {
    display: flex;
    gap: 2px;
    border: 1px solid var(--border-subtle);
    border-radius: var(--r-sm, 4px);
    overflow: hidden;
  }

  .mode-btn {
    padding: 3px 8px;
    font-size: 10px;
    font-family: var(--font-mono);
    font-weight: 500;
    text-transform: uppercase;
    letter-spacing: 0.06em;
    background: var(--bg-base);
    color: var(--text-secondary);
    border: none;
    cursor: pointer;
    transition: background 100ms, color 100ms;
  }

  .mode-btn:hover:not(:disabled) {
    background: var(--bg-hover);
    color: var(--text-primary);
  }

  .mode-btn.active {
    background: var(--teal-ghost, rgba(20, 184, 166, 0.15));
    color: var(--teal-bright, #14b8a6);
    font-weight: 600;
  }

  .mode-btn.dimmed,
  .mode-btn:disabled {
    opacity: 0.35;
    cursor: not-allowed;
  }

  /* Generic chrome groups */
  .chrome-group {
    display: flex;
    gap: 2px;
  }

  .chrome-btn {
    padding: 3px 7px;
    font-size: 11px;
    font-family: var(--font-mono);
    background: var(--bg-hover);
    color: var(--text-secondary);
    border: 1px solid var(--border-subtle);
    border-radius: var(--r-xs, 3px);
    cursor: pointer;
    transition: background 100ms, color 100ms;
    line-height: 1;
  }

  .chrome-btn:hover {
    background: var(--bg-surface);
    color: var(--text-primary);
  }

  .chrome-btn-text {
    font-size: 10px;
    text-transform: lowercase;
    letter-spacing: 0.04em;
  }

  .follow-active {
    background: var(--teal-ghost, rgba(20, 184, 166, 0.12));
    color: var(--teal-bright, #14b8a6);
    border-color: var(--teal-line, rgba(20, 184, 166, 0.35));
  }
</style>
