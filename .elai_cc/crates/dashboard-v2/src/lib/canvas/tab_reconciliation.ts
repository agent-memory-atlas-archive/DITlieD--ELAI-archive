// H-15 — tab_reconciliation.ts
//
// Per-tab snapshot/delta reconciliation for canvas view events (H15-3 slice).
//
// Design (CANVAS-WORKBENCH-DESIGN.md §18, H-15-decisions.md D-5):
//   - Per-tab sequence numbers (SSB `seq` field) are the monotonic rebuild key.
//   - On first mount: poll ALL recent canvas_view_event rows, find the most
//     recent StateCheckpoint row, set last_seen_seq from that row's seq.
//   - Incremental: poll only rows where seq > last_seen_seq.
//   - Periodic StateCheckpoint emission: every CANVAS_CHECKPOINT_INTERVAL_OPS
//     view events (named constant from ssb.rs) or on tab focus-gained.
//   - Drift detection: if last_seen_seq jumps backward (reconnect) or the
//     delta count exceeds MAX_DELTA_WITHOUT_CHECKPOINT, request a fresh
//     ViewSpawned snapshot.
//
// Monk 2003: pointing breaks if two parties see different display versions.
// The SSB seq ensures the dashboard tab and the model see the same view state.
//
// GD-1: mirrors canvasDispatcher.svelte.ts polling pattern (H-13).
// HARD RULES: named constants for all thresholds; no magic numbers.

import { get_canvas_views, emit_canvas_checkpoint } from '../api/stub';
import type { CanvasViewEventDto } from '../api/stub';

// ---------------------------------------------------------------------------
// Named constants (no magic numbers — CLAUDE.md HARD RULE)
// ---------------------------------------------------------------------------

/** Polling interval for get_canvas_views (ms). Matches H-11/H-13 cadence. */
const POLL_INTERVAL_MS = 2_000;

/** Number of view events between periodic StateCheckpoint emissions. */
const CHECKPOINT_INTERVAL_OPS = 50; // mirrors ssb.rs CANVAS_CHECKPOINT_INTERVAL_OPS

/** Maximum deltas between checkpoints before drift detection triggers. */
const MAX_DELTA_WITHOUT_CHECKPOINT = 100; // mirrors ssb.rs CANVAS_MAX_DELTA_WITHOUT_CHECKPOINT

/** Maximum view events to request per poll (server caps at 100). */
const POLL_LIMIT = 50;

// ---------------------------------------------------------------------------
// Types
// ---------------------------------------------------------------------------

/** A point-in-time snapshot of the canvas view state for a tab. */
export interface CanvasSnapshot {
  /** SSB seq of the StateCheckpoint row this snapshot was anchored on. */
  checkpointSeq: number;
  /** View id this snapshot belongs to. */
  viewId: string;
  /** Timestamp of the snapshot (unix ms). */
  ts: number;
}

/** State held per-tab by TabReconciliationState. */
export interface TabStateSnapshot {
  /** Last SSB seq we have processed (monotonically increasing). */
  lastSeenSeq: number;
  /** Most recent StateCheckpoint anchor (null = no checkpoint yet). */
  checkpoint: CanvasSnapshot | null;
  /** Count of deltas since the last StateCheckpoint row. */
  deltasSinceCheckpoint: number;
  /** True if a fresh ViewSpawned snapshot has been requested. */
  snapshotRequested: boolean;
}

// ---------------------------------------------------------------------------
// Store
// ---------------------------------------------------------------------------

function createTabReconciliationStore() {
  let tabStates = $state<Map<string, TabStateSnapshot>>(new Map());
  let pollerHandle: ReturnType<typeof setInterval> | null = null;
  /** All pending view events queued for the consumer (CanvasWindow). */
  let pendingViewEvents = $state<CanvasViewEventDto[]>([]);

  // ---------------------------------------------------------------------------
  // Per-tab state helpers
  // ---------------------------------------------------------------------------

  function getOrCreateTab(viewId: string): TabStateSnapshot {
    if (!tabStates.has(viewId)) {
      tabStates.set(viewId, {
        lastSeenSeq: 0,
        checkpoint: null,
        deltasSinceCheckpoint: 0,
        snapshotRequested: false,
      });
    }
    return tabStates.get(viewId)!;
  }

  // ---------------------------------------------------------------------------
  // Drift detection
  // ---------------------------------------------------------------------------

  /** True if the tab has diverged and needs a fresh snapshot. */
  function detectDrift(viewId: string): boolean {
    const st = tabStates.get(viewId);
    if (!st) return false;
    return st.deltasSinceCheckpoint > MAX_DELTA_WITHOUT_CHECKPOINT;
  }

  /** Request a fresh ViewSpawned snapshot for the tab (sets snapshotRequested flag). */
  function requestSnapshot(viewId: string): void {
    const st = getOrCreateTab(viewId);
    st.snapshotRequested = true;
    tabStates.set(viewId, { ...st });
  }

  // ---------------------------------------------------------------------------
  // Polling
  // ---------------------------------------------------------------------------

  async function poll(): Promise<void> {
    try {
      // Poll the global minimum last_seen_seq across all tabs.
      const minSeq = Math.min(
        ...[...tabStates.values()].map((st) => st.lastSeenSeq),
        0,
      );
      const rows = await get_canvas_views(POLL_LIMIT);
      // Filter to rows newer than the minimum seq we've seen.
      const fresh = rows.filter((r) => r.seq > minSeq);
      if (fresh.length === 0) return;

      fresh.sort((a, b) => a.seq - b.seq);

      for (const row of fresh) {
        const viewId = row.view_id;
        if (!viewId) continue;

        const st = getOrCreateTab(viewId);

        // Backward seq (reconnect) -> drift
        if (row.seq < st.lastSeenSeq) {
          requestSnapshot(viewId);
          continue;
        }

        st.lastSeenSeq = row.seq;

        if (row.kind === 'state_checkpoint') {
          st.checkpoint = {
            checkpointSeq: row.seq,
            viewId,
            ts: row.ts,
          };
          st.deltasSinceCheckpoint = 0;
        } else {
          st.deltasSinceCheckpoint++;
          if (st.deltasSinceCheckpoint > MAX_DELTA_WITHOUT_CHECKPOINT) {
            requestSnapshot(viewId);
          }
        }

        tabStates.set(viewId, { ...st });
      }

      // Add all fresh rows to pendingViewEvents for the consumer.
      pendingViewEvents = [
        ...pendingViewEvents.slice(-(POLL_LIMIT - fresh.length)),
        ...fresh,
      ];
    } catch {
      // Polling failure is non-fatal.
    }
  }

  // ---------------------------------------------------------------------------
  // Checkpoint emission
  // ---------------------------------------------------------------------------

  /** Emit a StateCheckpoint SSB row if interval has been reached. */
  async function maybeEmitCheckpoint(viewId: string, taskId?: string): Promise<void> {
    const st = tabStates.get(viewId);
    if (!st) return;
    if (st.deltasSinceCheckpoint >= CHECKPOINT_INTERVAL_OPS) {
      await emit_canvas_checkpoint(viewId, taskId);
      // The emitted row will be picked up by the next poll cycle.
    }
  }

  // ---------------------------------------------------------------------------
  // Public API (mirrors canvasDispatcher.svelte.ts idempotent pattern)
  // ---------------------------------------------------------------------------

  function startPolling() {
    if (pollerHandle !== null) return;
    poll();
    pollerHandle = setInterval(() => poll(), POLL_INTERVAL_MS);
  }

  function stopPolling() {
    if (pollerHandle !== null) {
      clearInterval(pollerHandle);
      pollerHandle = null;
    }
  }

  /** Drain all pending view events (called by CanvasWindow after applying them). */
  function drainPendingViewEvents(): CanvasViewEventDto[] {
    const drained = pendingViewEvents;
    pendingViewEvents = [];
    return drained;
  }

  return {
    get tabStates() {
      return tabStates;
    },
    get pendingViewEvents() {
      return pendingViewEvents;
    },
    getOrCreateTab,
    detectDrift,
    requestSnapshot,
    startPolling,
    stopPolling,
    drainPendingViewEvents,
    maybeEmitCheckpoint,
    // Named constant export for consumers.
    CHECKPOINT_INTERVAL_OPS,
    MAX_DELTA_WITHOUT_CHECKPOINT,
  };
}

export const tabReconciliation = createTabReconciliationStore();
