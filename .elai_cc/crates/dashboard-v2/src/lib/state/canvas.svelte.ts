// ============================================================
// canvas.svelte.ts — Canvas mode flag + single-consumer SSB arbiter (H-12)
//
// Design decisions (workflow/reasoning/H-12-decisions.md):
//
// MODE FLAG: a plain string union (CanvasMode), NOT an FSM substate.
// fsm.rs is UNTOUCHED. The mode flag gates which op-namespace is admissible
// in the CanvasDispatcher (H-13), not which FSM state is active. This removes
// compound-state bugs: an FSM substate would silently create states like
// "AUDITING + design_mode_active" whose transitions are specified nowhere.
// Source: CANVAS-WORKBENCH-DESIGN.md §7 and §11.
//
// SINGLE-CONSUMER SSB RULE (CANVAS §12):
// The active window OWNS the live SSB subscription; the inactive one is PAUSED.
// The handover is ATOMIC in suspend-then-activate order:
//   1. stopPolling() on the outgoing consumer (canvasActivity)
//   2. startPolling() on the incoming consumer (canvasActivity)
// Never activate-then-suspend — that leaves a race window where both
// poll simultaneously and double-apply events into a shared store.
//
// The orbit (ActivityWindow -> OrbitCanvas) is NOT a separate SSB poller —
// it reads plan.svelte.ts (5s poll, always-on, unaffected) and fsm.state
// reactively. The only consumer that must be arbitrated is canvasActivity
// (the H-11 live overlay store). So the handover is:
//   canvas active   -> canvasActivity.startPolling()
//   canvas inactive -> canvasActivity.stopPolling()
// ============================================================

import { canvasActivity } from './canvasActivity.svelte';

// ---------------------------------------------------------------------------
// CanvasMode type — plain string union, NOT an FSM substate.
// code / plan / research are wired in H-12.
// arch / design are wired in H-13 / H-14 respectively — include enum values
// now so downstream code (CanvasDispatcher, CanvasModeBar) can type-check
// against the full union without needing to patch the type later.
// ---------------------------------------------------------------------------

// J-31: 'system' added — the system-composition mode (nodes = bound plan
// units / contracts / MCP services, edges = wiring contracts). The frontend
// UI shell + mock fixtures ship now; the orchestrator backend
// (system_canvas.rs + SystemContractEvent SSB) lands later. The mode flag is
// still a plain string union, NOT an FSM substate.
export type CanvasMode = 'code' | 'plan' | 'research' | 'arch' | 'design' | 'system';

/** Named constants for each mode (avoids magic strings at call sites). */
export const CANVAS_MODE_CODE: CanvasMode = 'code';
export const CANVAS_MODE_PLAN: CanvasMode = 'plan';
export const CANVAS_MODE_RESEARCH: CanvasMode = 'research';
export const CANVAS_MODE_ARCH: CanvasMode = 'arch';
export const CANVAS_MODE_DESIGN: CanvasMode = 'design';
export const CANVAS_MODE_SYSTEM: CanvasMode = 'system';

/** Modes that are fully wired. arch wired in H-13; design wired in H-14.
 *  H-14: 'design' is now a wired mode (DesignPageNode + CanvasPanel mounted;
 *  DESIGN_NAMESPACE const added in canvas_dispatcher.rs). The live proxy
 *  done-gate (painted preview screenshot) is Tier-2 deferred (stub-8) but
 *  the mode infrastructure is live. */
const WIRED_MODES = new Set<CanvasMode>(['code', 'plan', 'research', 'arch', 'design', 'system']);

// ---------------------------------------------------------------------------
// Single-consumer SSB arbiter
//
// Tracks whether the canvas window is currently the active owner of the
// canvasActivity poller. The orbit does NOT need explicit tracking here
// because it has no separate SSB poller — its plan.tree feed is always-on
// and unaffected by the handover.
// ---------------------------------------------------------------------------

function createCanvas() {
  let mode = $state<CanvasMode>('code');

  // Whether the canvas window currently holds the SSB subscription.
  // Managed by activateCanvas() / deactivateCanvas() called from CanvasWindow.
  let isActive = $state<boolean>(false);

  // ---------------------------------------------------------------------------
  // Mode operations
  // ---------------------------------------------------------------------------

  /** Set the canvas mode. Silently ignores unknown values. */
  function setMode(next: CanvasMode) {
    mode = next;
  }

  /** True if this mode's full rendering is wired (H-12). */
  function isModeWired(m: CanvasMode): boolean {
    return WIRED_MODES.has(m);
  }

  // ---------------------------------------------------------------------------
  // Single-consumer SSB handover — ATOMIC, suspend-then-activate order.
  //
  // Called from CanvasWindow.svelte onMount / onDestroy.
  //
  // INVARIANT: stopPolling() is always called BEFORE startPolling().
  // Violation = double-application of SSB events into a shared store.
  // ---------------------------------------------------------------------------

  /** Canvas window is becoming active — take ownership of the SSB subscription.
   *  Suspend-then-activate order: we are the incoming consumer; the outgoing
   *  consumer (orbit) has no SSB poller, so we call startPolling() directly.
   *  If canvasActivity is already polling (previous activateCanvas() without
   *  a matching deactivateCanvas()), startPolling() is idempotent (H-11:
   *  startPolling() returns early if pollerHandle !== null). */
  function activateCanvas() {
    // stopPolling first — ensure no double-poll if called twice without deactivate.
    canvasActivity.stopPolling();
    canvasActivity.startPolling();
    isActive = true;
  }

  /** Canvas window is going dormant — release the SSB subscription.
   *  The orbit does not consume canvasActivity, so stopping here means
   *  NO consumer holds the live overlay poll while the orbit is visible.
   *  This is correct: the live overlay is a canvas-specific feature.
   *  The orbit's own plan.tree feed (plan.svelte.ts 5s poll) runs always. */
  function deactivateCanvas() {
    canvasActivity.stopPolling();
    isActive = false;
  }

  // ---------------------------------------------------------------------------
  // Public API
  // ---------------------------------------------------------------------------

  return {
    get mode(): CanvasMode { return mode; },
    get isActive(): boolean { return isActive; },
    setMode,
    isModeWired,
    activateCanvas,
    deactivateCanvas,
  };
}

export const canvas = createCanvas();
