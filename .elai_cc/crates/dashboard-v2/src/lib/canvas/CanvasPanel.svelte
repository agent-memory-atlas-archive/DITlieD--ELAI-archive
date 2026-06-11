<script lang="ts">
  // H-13: CanvasPanel.svelte — headless canvas op subscriber component.
  //
  // Wraps canvasDispatcher.svelte.ts: starts and stops the get_canvas_ops
  // poll loop for the current canvas mode. CanvasWindow.svelte mounts this
  // alongside the sigma renderer; the canvasDispatcher store's pendingOps
  // is drained by CanvasWindow to apply mutations to the graphology instance.
  //
  // Design doc: CANVAS-WORKBENCH-DESIGN.md §7 (op stream) + §11 (architecture).
  //
  // Pitfall (SVELTE_ONDESTROY_READS_STALE_REACTIVE_USE_BOOTSTRAPPED_FLAG):
  //   onDestroy uses _bootstrapped, NOT canvas.mode.
  //
  // Pitfall (SINGLE_CONSUMER_SSB_HANDOVER_ONE_OWNER):
  //   CanvasPanel polls canvas_op_applied; canvasActivity polls
  //   canvas_activity_event. Two parallel polls, no shared arbiter.

  import { onMount, onDestroy } from 'svelte';
  import { canvasDispatcher } from '../state/canvasDispatcher.svelte';

  interface Props {
    /** Current canvas mode (from canvas.svelte.ts store). */
    mode: string;
  }

  let { mode }: Props = $props();

  let _bootstrapped = false; // onDestroy guard

  onMount(() => {
    _bootstrapped = true;
    canvasDispatcher.startPolling(mode);
  });

  onDestroy(() => {
    // Use _bootstrapped flag (not canvas.mode — pitfall).
    if (_bootstrapped) {
      canvasDispatcher.stopPolling();
    }
  });
</script>

<!--
  CanvasPanel renders no visible UI. It is a lifecycle controller for the
  canvasDispatcher poll loop. CanvasWindow.svelte mounts this component and
  reads canvasDispatcher.pendingOps to apply canvas_op_applied rows to sigma.
-->
