<script lang="ts">
  import { workspace } from '../state/workspace.svelte';

  interface Props {
    onSplitChange: (primaryPx: number) => void;
    containerRef?: HTMLElement | undefined;
  }

  let { onSplitChange, containerRef }: Props = $props();

  function resolveContainer(): HTMLElement | null {
    return containerRef ?? document.querySelector<HTMLElement>('.panels-area');
  }

  let isDragging = $state(false);
  let dividerEl = $state<HTMLElement | undefined>(undefined);

  const MIN_PANEL_PX = 320;

  function clampSplit(px: number, totalWidth: number): number {
    const max = totalWidth - 2 - MIN_PANEL_PX;
    return Math.min(Math.max(px, MIN_PANEL_PX), max);
  }

  function onPointerDown(e: PointerEvent) {
    e.preventDefault();
    isDragging = true;
    document.body.style.cursor = 'col-resize';
    document.body.style.userSelect = 'none';
    (e.target as HTMLElement).setPointerCapture(e.pointerId);
  }

  function onPointerMove(e: PointerEvent) {
    if (!isDragging) return;
    const container = resolveContainer();
    if (!container) return;
    const rect = container.getBoundingClientRect();
    const totalWidth = rect.width;
    const relX = e.clientX - rect.left;
    const clamped = clampSplit(relX, totalWidth);
    onSplitChange(clamped);
  }

  function onPointerUp() {
    if (!isDragging) return;
    isDragging = false;
    document.body.style.cursor = '';
    document.body.style.userSelect = '';
  }

  function onDoubleClick() {
    const container = resolveContainer();
    if (!container) return;
    const totalWidth = container.offsetWidth;
    onSplitChange(Math.floor((totalWidth - 2) / 2));
    workspace.resetSplit();
  }

  function onKeyDown(e: KeyboardEvent) {
    const container = resolveContainer();
    if (!container) return;
    const totalWidth = container.offsetWidth;
    const currentSplit = container.querySelector<HTMLElement>('.panel:first-child')?.offsetWidth ?? Math.floor(totalWidth / 2);

    if (e.key === 'ArrowLeft') {
      e.preventDefault();
      onSplitChange(clampSplit(currentSplit - 16, totalWidth));
    } else if (e.key === 'ArrowRight') {
      e.preventDefault();
      onSplitChange(clampSplit(currentSplit + 16, totalWidth));
    }
  }
</script>

<!-- svelte-ignore a11y_no_noninteractive_tabindex -->
<!-- svelte-ignore a11y_no_noninteractive_element_interactions -->
<div
  class="panel-divider"
  class:panel-divider--dragging={isDragging}
  bind:this={dividerEl}
  role="separator"
  aria-orientation="vertical"
  aria-label="Resize panels — arrow keys shift by 16px, double-click to reset"
  tabindex={0}
  onpointerdown={onPointerDown}
  onpointermove={onPointerMove}
  onpointerup={onPointerUp}
  onpointercancel={onPointerUp}
  ondblclick={onDoubleClick}
  onkeydown={onKeyDown}
></div>

<style>
  .panel-divider {
    width: 2px;
    align-self: stretch;
    flex-shrink: 0;
    background: transparent;
    cursor: col-resize;
    outline: none;
    transition: background 120ms var(--ease-std);
    position: relative;
    z-index: 5;
  }

  .panel-divider::before {
    content: '';
    position: absolute;
    inset: 0 -4px;
  }

  .panel-divider:hover,
  .panel-divider:focus-visible {
    background: var(--teal-line);
  }

  .panel-divider--dragging {
    background: var(--teal-line);
  }
</style>
