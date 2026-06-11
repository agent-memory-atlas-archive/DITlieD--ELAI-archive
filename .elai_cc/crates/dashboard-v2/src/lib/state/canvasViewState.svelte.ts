// ============================================================
// canvasViewState.svelte.ts — Canvas view event store (H-15)
//
// Polls get_canvas_views at POLL_INTERVAL_MS and queues validated
// CanvasViewEventDto entries in pendingViewEvents. CanvasWindow.svelte
// reads and drains pendingViewEvents to dispatch to the appropriate renderer.
//
// Design: mirrors the H-13 canvasDispatcher.svelte.ts polling pattern (GD-1).
// SEPARATE from canvasDispatcher: this store drains canvas_view_event rows
// only. The canvas_op_applied drain in canvasDispatcher is NEVER mixed with
// this store (gotcha OP_APPLICATION_TRANSPORT_IS_H13_DRAIN_PENDINGOPS).
//
// Snapshot/delta reconstruction (H15-3):
//   - On first mount: poll ALL recent canvas_view_event rows, record
//     last_seen_seq from the SSB `seq` field of the most recent row.
//   - Incremental: poll only rows where seq > last_seen_seq.
//   - On kill+reopen: last_seen_seq resets to 0; the first poll fetches the
//     full recent tail and rebuilds from the most recent StateCheckpoint.
//   - Drift detection: delegated to tab_reconciliation.ts.
//
// Monk 2003: pointing breaks if two parties see different display versions.
// The monotonic seq ensures consistent tab state across model and dashboard.
// ============================================================

import { get_canvas_views } from '../api/stub';
import type { CanvasViewEventDto } from '../api/stub';

// ---------------------------------------------------------------------------
// Named constants (no magic numbers — CLAUDE.md HARD RULE)
// ---------------------------------------------------------------------------

/** Polling interval for get_canvas_views (ms). Matches H-11/H-13 cadence. */
const POLL_INTERVAL_MS = 2_000;

/** Maximum view events to request per poll. Server caps at 100. */
const POLL_LIMIT = 50;

/** Ring buffer cap: keep at most this many pending view events. */
const MAX_PENDING_VIEW_EVENTS = 200;

// ---------------------------------------------------------------------------
// Store
// ---------------------------------------------------------------------------

function createCanvasViewStateStore() {
  /** Pending view events ready for CanvasWindow/renderer consumers. */
  let pendingViewEvents = $state<CanvasViewEventDto[]>([]);
  /** Last seen SSB seq (monotonic; used for incremental delta polling). */
  let lastSeenSeq = $state<number>(0);
  let pollerHandle: ReturnType<typeof setInterval> | null = null;

  // ---------------------------------------------------------------------------
  // Polling
  // ---------------------------------------------------------------------------

  async function poll(): Promise<void> {
    try {
      const rows = await get_canvas_views(POLL_LIMIT);
      const fresh = rows.filter((r) => r.seq > lastSeenSeq);
      if (fresh.length === 0) return;

      fresh.sort((a, b) => a.seq - b.seq);
      for (const row of fresh) {
        if (row.seq > lastSeenSeq) lastSeenSeq = row.seq;
      }

      pendingViewEvents = [
        ...pendingViewEvents.slice(-(MAX_PENDING_VIEW_EVENTS - fresh.length)),
        ...fresh,
      ];
    } catch {
      // Polling failure is non-fatal (SSB not attached or transient error).
    }
  }

  // ---------------------------------------------------------------------------
  // Public API (mirrors canvasDispatcher pattern — GD-1)
  // ---------------------------------------------------------------------------

  function startPolling() {
    if (pollerHandle !== null) return;
    // First poll immediately (captures all recent rows from mount).
    poll();
    pollerHandle = setInterval(() => poll(), POLL_INTERVAL_MS);
  }

  function stopPolling() {
    if (pollerHandle !== null) {
      clearInterval(pollerHandle);
      pollerHandle = null;
    }
  }

  /** Reset seq counter (use on tab kill+reopen to force full tail reconstruction). */
  function resetSeq() {
    lastSeenSeq = 0;
  }

  /** Drain all pending view events (called by CanvasWindow after routing to renderers). */
  function drainPendingViewEvents(): CanvasViewEventDto[] {
    const drained = pendingViewEvents;
    pendingViewEvents = [];
    return drained;
  }

  return {
    get pendingViewEvents() {
      return pendingViewEvents;
    },
    get lastSeenSeq() {
      return lastSeenSeq;
    },
    startPolling,
    stopPolling,
    resetSeq,
    drainPendingViewEvents,
  };
}

export const canvasViewState = createCanvasViewStateStore();
