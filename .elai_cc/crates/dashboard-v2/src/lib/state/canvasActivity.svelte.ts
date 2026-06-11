// ============================================================
// canvasActivity.svelte.ts — Canvas live overlay store (H-11)
//
// Subscribes to CanvasActivityEvent SSB rows via periodic polling of the
// get_canvas_activity Tauri command. Resolves each event's target_node_id
// against the canvas node set and drives:
//
//   (a) halo-glow state on matched nodes in the active mode;
//   (b) off-frame indicator for nodes in a non-active mode;
//   (c) ghost-node creation + reconcile-or-fade (30s) for un-indexed files.
//
// focus-bus integration (W-3 / GD-1):
//   - When followAgent is ON, calls graph.focus_node() on each new event so
//     the canvas stays centered on the active node.
//   - Does NOT call graph.consume_focus() — only GraphWindow drains the bus
//     (pitfall from context-package: focus-bus-cleared-after-consume).
//
// Ghost-node reconciliation design (CANVAS-WORKBENCH-DESIGN.md §5):
//   - Un-indexed file -> transient GhostNode placed in the store.
//   - H-10 LOD graph data (VizFile ids from get_graph_view_data) is consumed
//     to find the "near nearest resolvable ancestor" (findNearestAncestor).
//     Ghost.ancestorId carries this result; null = root anchor fallback.
//   - After GHOST_FADE_MS (30_000ms) without reconciliation, ghost fades to
//     DroppedMarker. NEVER a silent gap.
//
// HD-5 (substrate honesty): the rendered overlay is NOT proof that the event
// stream is correct. The off-agent CanvasActivityEvent SSB rows are the
// authoritative backstop (verified in h11_live_layer_fs_contract.rs).
// ============================================================

import { get_canvas_activity } from '../api/stub';
import type { CanvasActivityEventDto } from '../api/stub';
import { graph } from './graph.svelte';
import { invoke } from '@tauri-apps/api/core';
import type { VizData } from '../api/types';

// ---------------------------------------------------------------------------
// Named constants (no magic numbers)
// ---------------------------------------------------------------------------

/** Ghost node fades to a dropped-marker after this many milliseconds without
 *  reconciliation. Per CANVAS-WORKBENCH-DESIGN.md §5: "30 seconds". */
const GHOST_FADE_MS = 30_000;

/** Polling interval for get_canvas_activity (milliseconds). */
const POLL_INTERVAL_MS = 2_000;

/** Maximum events to request per poll (server caps at 100). */
const POLL_LIMIT = 20;

// ---------------------------------------------------------------------------
// Types
// ---------------------------------------------------------------------------

export type GhostState = 'active' | 'dropped';

/** A transient ghost node for an un-indexed file. */
export interface GhostNode {
  nodeId: string;
  filePath: string;
  symbolName: string | null;
  mode: string;
  label: string | null;
  createdAtMs: number;
  state: GhostState;
  /** Nearest indexed ancestor node id from the H-10 graph, used for placement.
   *  Null when no ancestor is found in the indexed file set (ghost placed at canvas root). */
  ancestorId: string | null;
}

// ---------------------------------------------------------------------------
// Store
// ---------------------------------------------------------------------------

function createCanvasActivityStore() {
  let activeEvents = $state<CanvasActivityEventDto[]>([]);
  let ghostNodes = $state<Map<string, GhostNode>>(new Map());
  let followAgent = $state<boolean>(true);
  let lastSeenTs = $state<number>(0);
  let pollerHandle: ReturnType<typeof setInterval> | null = null;
  let ghostTimers: Map<string, ReturnType<typeof setTimeout>> = new Map();

  // ---------------------------------------------------------------------------
  // H-10 LOD: indexed file-id set (lazy-loaded from get_graph_view_data)
  // ---------------------------------------------------------------------------

  /** Set of file-node ids from the H-10 graph (file paths as indexed by the DKG).
   *  Used for "near nearest indexed ancestor" ghost placement. Loaded once on
   *  first startPolling() via invoke('get_graph_view_data'). Empty until loaded. */
  let indexedFileIds: Set<string> = new Set();
  let indexLoaded = false;

  async function loadIndexedFileIds(): Promise<void> {
    if (indexLoaded) return;
    indexLoaded = true; // set early to prevent concurrent loads
    try {
      const viz = await invoke<VizData>('get_graph_view_data', { mode: 'file' });
      indexedFileIds = new Set(viz.files.map((f) => f.id));
    } catch {
      // Non-fatal: if the graph is not built yet, ghost placement falls back
      // to root anchor (same behaviour as before this wiring).
      indexLoaded = false; // allow retry on next poll start
    }
  }

  /** Walk the ghost file path upward (by directory segments) and return the
   *  deepest ancestor id found in indexedFileIds, or null if none matches.
   *
   *  Example: ghost path "src/lib/state/canvasActivity.svelte.ts"
   *    checks: "src/lib/state" -> "src/lib" -> "src" -> (root not checked)
   *  Returns the deepest match. Path separators are normalised to '/'. */
  function findNearestAncestor(filePath: string): string | null {
    if (indexedFileIds.size === 0) return null;
    // Normalise to forward-slash.
    const normalised = filePath.replace(/\\/g, '/');
    const parts = normalised.split('/');
    // Walk from deepest parent upward (excluding the filename itself).
    for (let i = parts.length - 2; i >= 1; i--) {
      const candidate = parts.slice(0, i + 1).join('/');
      if (indexedFileIds.has(candidate)) return candidate;
    }
    return null;
  }

  // ---------------------------------------------------------------------------
  // Ghost lifecycle helpers
  // ---------------------------------------------------------------------------

  function scheduleGhostFade(nodeId: string) {
    // Clear any existing timer for this ghost.
    const existing = ghostTimers.get(nodeId);
    if (existing !== undefined) {
      clearTimeout(existing);
    }
    const handle = setTimeout(() => {
      ghostTimers.delete(nodeId);
      ghostNodes = new Map(ghostNodes);
      const ghost = ghostNodes.get(nodeId);
      if (ghost && ghost.state === 'active') {
        ghostNodes.set(nodeId, { ...ghost, state: 'dropped' });
        ghostNodes = new Map(ghostNodes);
      }
    }, GHOST_FADE_MS);
    ghostTimers.set(nodeId, handle);
  }

  function reconcileGhost(nodeId: string) {
    const handle = ghostTimers.get(nodeId);
    if (handle !== undefined) {
      clearTimeout(handle);
      ghostTimers.delete(nodeId);
    }
    ghostNodes.delete(nodeId);
    ghostNodes = new Map(ghostNodes);
  }

  function createGhost(event: CanvasActivityEventDto) {
    // Consume H-10 LOD graph data: find the deepest ancestor of the un-indexed
    // file that IS present in the H-10 indexed file-node set. This satisfies
    // the "near nearest resolvable ancestor" placement contract from
    // CANVAS-WORKBENCH-DESIGN.md §5 using the same DKG data source as
    // CanvasWindow (invoke get_graph_view_data). Null = no ancestor found,
    // which results in root-anchor placement (the original Tier-1 fallback).
    const ancestorId = findNearestAncestor(event.file_path);
    const ghost: GhostNode = {
      nodeId: event.target_node_id,
      filePath: event.file_path,
      symbolName: event.symbol_name,
      mode: event.mode,
      label: event.label,
      createdAtMs: Date.now(),
      state: 'active',
      ancestorId,
    };
    ghostNodes.set(event.target_node_id, ghost);
    ghostNodes = new Map(ghostNodes);
    scheduleGhostFade(event.target_node_id);
  }

  // ---------------------------------------------------------------------------
  // Event processing
  // ---------------------------------------------------------------------------

  function processEvent(event: CanvasActivityEventDto) {
    // Update lastSeenTs to avoid re-processing.
    if (event.ts > lastSeenTs) {
      lastSeenTs = event.ts;
    }

    // Resolution case (c): ghost reconciliation. If a ghost exists for this
    // target_node_id, the real node has now surfaced — reconcile.
    if (ghostNodes.has(event.target_node_id)) {
      reconcileGhost(event.target_node_id);
    }

    // Follow-agent: push focus_node so the canvas centers on the active node.
    // Does NOT consume the focus signal (GraphWindow owns consume_focus).
    if (followAgent) {
      graph.focus_node(event.target_node_id);
    }

    // TODO[H-11.tier2.1]: resolution case (a) halo-glow requires the H-10
    // sigma renderer to accept external highlight commands (no exported API in
    // Tier-1; sigma instance is local to CanvasWindow.svelte mountSigma()).
    // For Tier-1 we track the event in activeEvents so the overlay component
    // can query it. Tier-2: export a sigmaRef + setNodeHighlight() API.
    // TODO[H-11.tier2.2]: resolution case (b) off-frame indicator requires
    // knowledge of which mode each node belongs to (H-10 graph mode store,
    // not yet exported). Tier-2: add a shared modeOf(nodeId) store accessor.
  }

  // ---------------------------------------------------------------------------
  // Polling
  // ---------------------------------------------------------------------------

  async function poll() {
    try {
      const events = await get_canvas_activity(POLL_LIMIT);
      // Only process events newer than what we've already seen.
      const fresh = events.filter((e) => e.ts > lastSeenTs);
      if (fresh.length === 0) return;

      // Sort ascending so we process oldest-first.
      fresh.sort((a, b) => a.ts - b.ts);
      activeEvents = [...activeEvents.slice(-100), ...fresh]; // bounded ring
      for (const event of fresh) {
        processEvent(event);
      }
    } catch {
      // Polling failure is non-fatal; overlay degrades gracefully.
    }
  }

  function startPolling() {
    if (pollerHandle !== null) return;
    // Load H-10 indexed file-ids on first start for ancestor placement.
    loadIndexedFileIds();
    poll(); // immediate first poll
    pollerHandle = setInterval(poll, POLL_INTERVAL_MS);
  }

  function stopPolling() {
    if (pollerHandle !== null) {
      clearInterval(pollerHandle);
      pollerHandle = null;
    }
  }

  // ---------------------------------------------------------------------------
  // Public API
  // ---------------------------------------------------------------------------

  function setFollowAgent(on: boolean) {
    followAgent = on;
  }

  function dismissGhost(nodeId: string) {
    reconcileGhost(nodeId);
  }

  return {
    get activeEvents() { return activeEvents; },
    get ghostNodes() { return ghostNodes; },
    get followAgent() { return followAgent; },
    setFollowAgent,
    dismissGhost,
    startPolling,
    stopPolling,
  };
}

export const canvasActivity = createCanvasActivityStore();
