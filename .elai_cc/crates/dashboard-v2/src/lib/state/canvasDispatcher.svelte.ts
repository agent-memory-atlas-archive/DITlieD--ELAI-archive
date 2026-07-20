// ============================================================
// canvasDispatcher.svelte.ts — Canvas op dispatcher store (H-13)
//
// Polls get_canvas_ops at POLL_INTERVAL_MS and queues validated
// CanvasOpDto entries in pendingOps. CanvasWindow.svelte reads and
// drains pendingOps to apply mutations to the sigma/graphology instance.
//
// Design: mirrors the H-11 canvasActivity.svelte.ts polling pattern (GD-1).
// The graphology instance is local to CanvasWindow.svelte; this store
// exposes reactive state that CanvasWindow reads via $effect.
//
// sanitizeCanvasOp:
//   - Mode-namespace gating: op.mode must match currentMode.
//   - Structural dangling-id rejection: empty node_id on node-referencing ops.
//   - NEVER silently drops a bad op (wiring-enforcement R1 + HD-3).
//   - CanvasWindow does the live graph-existence checks (hasNode/hasEdge)
//     because the graphology instance is not accessible from this store.
//
// Pitfall (SVELTE_ONDESTROY_READS_STALE_REACTIVE_USE_BOOTSTRAPPED_FLAG):
//   stopPolling() uses a local _bootstrapped flag, not canvas.mode.
// ============================================================

import { get_canvas_ops } from '../api/stub';
import type { CanvasOpDto } from '../api/stub';
import { registerUsageBadgeSetter, type UsageBadgeState } from '../canvas/arch_canvas_ops';

// ---------------------------------------------------------------------------
// Named constants (no magic numbers — CLAUDE.md HARD RULE)
// ---------------------------------------------------------------------------

/** Polling interval for get_canvas_ops (milliseconds). Matches H-11 cadence. */
const POLL_INTERVAL_MS = 2_000;

/** Maximum ops to request per poll (server caps at 100). */
const POLL_LIMIT = 20;

/** Ring buffer cap: keep at most this many ops in pendingOps. */
const MAX_PENDING_OPS = 200;

// ---------------------------------------------------------------------------
// Store
// ---------------------------------------------------------------------------

function createCanvasDispatcherStore() {
  /** Ops that have passed sanitizeCanvasOp and are ready for CanvasWindow. */
  let pendingOps = $state<CanvasOpDto[]>([]);
  let lastSeenTs = $state<number>(0);
  let pollerHandle: ReturnType<typeof setInterval> | null = null;

  // J-29 SLICE-4: usage badge state. Updated by applyOpToGraph (via the
  // registered setter) when a usage_badge AnnotationAdded op is drained,
  // and by CanvasWindow's get_live_usage poll fallback. CanvasWindow reads
  // usageBadge reactively to render the chrome badge.
  let usageBadge = $state<UsageBadgeState>({ session_total: 0, live_value: 0 });

  function setUsageBadge(state: UsageBadgeState): void {
    usageBadge = { ...state };
  }

  // Register the setter so arch_canvas_ops (a pure module without Svelte
  // runtime access) can update this store without importing it directly.
  registerUsageBadgeSetter(setUsageBadge);

  // ---------------------------------------------------------------------------
  // sanitizeCanvasOp
  // ---------------------------------------------------------------------------

  function sanitizeCanvasOp(
    op: CanvasOpDto,
    currentMode: string,
  ): { ok: boolean; reason?: string } {
    if (op.mode !== currentMode) {
      return {
        ok: false,
        reason: `mode mismatch: op.mode='${op.mode}' but canvas is in mode='${currentMode}'`,
      };
    }
    const opsThatNeedNodeId: string[] = ['node_deleted', 'label_edited', 'node_forked'];
    if (opsThatNeedNodeId.includes(op.kind) && !op.node_id.trim()) {
      return {
        ok: false,
        reason: `dangling-id rejected: op '${op.kind}' requires a non-empty node_id`,
      };
    }
    if (op.kind === 'edge_connected') {
      const fromId = String(op.op_data['from_id'] ?? '');
      const toId = String(op.op_data['to_id'] ?? '');
      if (!fromId.trim()) {
        return { ok: false, reason: 'dangling-id rejected: edge_connected missing from_id' };
      }
      if (!toId.trim()) {
        return { ok: false, reason: 'dangling-id rejected: edge_connected missing to_id' };
      }
    }
    return { ok: true };
  }

  // ---------------------------------------------------------------------------
  // Polling
  // ---------------------------------------------------------------------------

  async function poll(currentMode: string): Promise<void> {
    try {
      const ops = await get_canvas_ops(POLL_LIMIT);
      const fresh = ops.filter((op) => op.ts > lastSeenTs);
      if (fresh.length === 0) return;

      fresh.sort((a, b) => a.ts - b.ts);
      const validated: CanvasOpDto[] = [];
      for (const op of fresh) {
        if (op.ts > lastSeenTs) lastSeenTs = op.ts;
        const check = sanitizeCanvasOp(op, currentMode);
        if (!check.ok) {
          // Non-silent rejection (wiring-enforcement R1 + HD-3).
          console.warn(
            `canvasDispatcher: rejected op kind='${op.kind}' ts=${op.ts}: ${check.reason}`,
          );
          continue;
        }
        validated.push(op);
      }
      if (validated.length > 0) {
        pendingOps = [
          ...pendingOps.slice(-(MAX_PENDING_OPS - validated.length)),
          ...validated,
        ];
      }
    } catch {
      // Polling failure is non-fatal.
    }
  }

  // ---------------------------------------------------------------------------
  // Public API
  // ---------------------------------------------------------------------------

  function startPolling(currentMode: string) {
    if (pollerHandle !== null) return;
    poll(currentMode);
    pollerHandle = setInterval(() => poll(currentMode), POLL_INTERVAL_MS);
  }

  function stopPolling() {
    if (pollerHandle !== null) {
      clearInterval(pollerHandle);
      pollerHandle = null;
    }
  }

  /** Drain all pending ops (called by CanvasWindow after applying them). */
  function drainPendingOps(): CanvasOpDto[] {
    const drained = pendingOps;
    pendingOps = [];
    return drained;
  }

  return {
    get pendingOps() {
      return pendingOps;
    },
    get usageBadge() {
      return usageBadge;
    },
    setUsageBadge,
    startPolling,
    stopPolling,
    drainPendingOps,
  };
}

export const canvasDispatcher = createCanvasDispatcherStore();
