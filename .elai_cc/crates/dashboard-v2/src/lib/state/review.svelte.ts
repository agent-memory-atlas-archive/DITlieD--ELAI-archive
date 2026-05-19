// ============================================================
// review.svelte.ts — ReviewWindow shared state (C-7)
// ============================================================
//
// Lightweight Svelte 5 store that exposes accept-all / decline-all
// signals so the global Ctrl+Shift+A / Ctrl+Shift+R shortcuts can
// trigger actions inside the ReviewWindow component (which owns the
// per-hunk hunkState / fileState local maps).
//
// Why a "signal counter" instead of direct method invocation?
//   ReviewWindow re-mounts on tab switch + lives inside an {#if} branch.
//   A direct method registry would race with mount/unmount. A monotonically
//   incrementing counter ($state number) lets the window subscribe via
//   $effect and react on every increment regardless of mount timing.
//
// `focusedHunkIndex` is reserved for future Ctrl+ArrowDown navigation
// (defined but currently only updated by ReviewWindow on hunk click).

function createReviewStore() {
  // Monotonic counters — every increment triggers ReviewWindow's $effect.
  // Reset to 0 on dashboard reload (state is in-memory only by design).
  let acceptAllSignal = $state<number>(0);
  let rejectAllSignal = $state<number>(0);
  let focusedHunkIndex = $state<number>(-1);

  function acceptAllHunks() {
    acceptAllSignal = acceptAllSignal + 1;
  }

  function rejectAllHunks() {
    rejectAllSignal = rejectAllSignal + 1;
  }

  function setFocusedHunkIndex(idx: number) {
    focusedHunkIndex = idx;
  }

  return {
    get acceptAllSignal() {
      return acceptAllSignal;
    },
    get rejectAllSignal() {
      return rejectAllSignal;
    },
    get focusedHunkIndex() {
      return focusedHunkIndex;
    },
    acceptAllHunks,
    rejectAllHunks,
    setFocusedHunkIndex,
  };
}

export const reviewStore = createReviewStore();
