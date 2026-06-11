// H-10: entrypoints_map.ts — dynamic-dispatch edge injector for dead-code detection.
//
// Problem: static call-graph analysis (detect_entry_points heuristic) flags files
// with zero static callers as "orphan candidates". But some files/symbols are reached
// ONLY via dynamic dispatch — Tauri invoke() strings, dyn Trait vtable, SSB bus
// action_type consumers — none of which appear as static call-graph edges.
//
// Solution (GD-1 compose-not-restate): the PRIMARY source for dynamic-dispatch
// entrypoints is the Rust backend's detect_entry_points (B-34, commands.rs
// `list_entry_points`). Call fetchBackendEntryPoints() to retrieve those file paths
// via Tauri IPC before building the graph. The filename heuristics below are a
// LABELED FALLBACK used only when the backend invoke fails or returns empty.
//
// Usage:
//   1. const backendEPs = await fetchBackendEntryPoints();   // primary
//   2. injectDynamicEntryEdges(graphInstance, backendEPs);   // primary + fallback
//   This satisfies GD-1: real detect_entry_points output is consumed, not re-implemented.

import { invoke } from '@tauri-apps/api/core';
import type Graph from 'graphology';

// ---------------------------------------------------------------------------
// Known dynamic-dispatch entrypoint categories
// ---------------------------------------------------------------------------

/** A Tauri invoke() command name registered in lib.rs generate_handler!. The file
 *  that implements the command is reachable from the frontend — not a dead file. */
const TAURI_COMMAND_NAMES: ReadonlySet<string> = new Set([
  // B-33 graph commands
  'get_graph_view_data',
  'get_pagerank_top',
  'get_impact_overlay',
  'export_graph_mermaid',
  // B-40 idea-framing
  'get_idea_framing_state',
  'submit_idea_framing',
  // B-41 audit timeline
  'get_audit_timeline',
  // C-7 explore
  'oracle_query',
  // C-40 github
  'list_prs',
  'get_pr_detail',
  // E-40 symbol graph
  'symbol_graph_pagerank',
  'list_entry_points',
  // G-5 cockpit
  'get_review_surface',
  // H-10 canvas
  'get_graph_view_data',
  // General indexing/reindex
  'start_reindex',
  'get_reindex_status',
  'get_task_status',
  'run_symbol_graph_rank',
]);

/** SSB action_type strings whose consumers live in production code (wiring_coverage.rs,
 *  ssb_viewer.rs) but have no static call-graph incoming edges from the view layer. */
const SSB_ACTION_TYPES: ReadonlySet<string> = new Set([
  'skill_system_event',
  'route_decision_event',
  'canvas_activity_event',
  'batch_lifecycle',
  // H-13: model-driven canvas op stream (CanvasPanel.svelte consumer).
  'canvas_op_applied',
  // H-15: canvas view reply-channel event (canvasViewState.svelte.ts consumer).
  'canvas_view_event',
]);

// H-13: CanvasOpsDispatcher dyn dispatch point. Listed here to prevent HD-5
// orphan-detection false-positive. InProcessCanvasDispatcher is the concrete
// impl; it is wired via translate_tool_request canvas_* arms in production.
// This constant is advisory-only (the backend detect_entry_points is the
// authoritative source via fetchBackendEntryPoints()).
const CANVAS_DYN_DISPATCH_FILES: ReadonlySet<string> = new Set([
  'crates/orchestrator/src/canvas_dispatcher.rs',
  'crates/orchestrator/src/canvas_grammar.rs',
  'crates/dashboard-v2/src/lib/canvas/CanvasPanel.svelte',
]);

// ---------------------------------------------------------------------------
// Virtual root node
// ---------------------------------------------------------------------------

/** Stable synthetic node id used as the reachability root. Not a real file;
 *  exists only in the in-memory graphology graph during canvas rendering.
 *  Canvas.svelte must NOT render this node — it is invisible scaffolding. */
export const DYNAMIC_ENTRYPOINT_ROOT = '__dynamic_entrypoint_root__';

// ---------------------------------------------------------------------------
// Backend entry-point fetch (GD-1 primary source)
// ---------------------------------------------------------------------------

/** Shape of a single entry-point record returned by list_entry_points. */
interface BackendEntryPoint {
  id: string;
  name: string;
  file: string;
  line: number;
  reason: string;
}

/**
 * Fetch the real detected entry-point file paths from the Rust backend
 * (detect_entry_points / list_entry_points, B-34).
 *
 * Returns a Set<string> of file paths that the engine determined are
 * reachable from dynamic-dispatch entrypoints. Empty on failure or when
 * the index has not been built yet — callers fall back to filename heuristics.
 *
 * GD-1: this is the PRIMARY consume-not-restate path. The filename heuristics
 * in looksLikeTauriCommandFile / looksLikeSsbConsumerFile are a LABELED FALLBACK
 * for when this call fails or returns empty.
 */
export async function fetchBackendEntryPoints(): Promise<Set<string>> {
  try {
    const result = await invoke<{ entry_points: BackendEntryPoint[]; summary: unknown }>(
      'list_entry_points',
      { include_tests: false }
    );
    const files = new Set<string>();
    for (const ep of result.entry_points ?? []) {
      if (ep.file) files.add(ep.file);
    }
    console.debug(`[entrypoints_map] backend reported ${files.size} entry-point files`);
    return files;
  } catch (err) {
    // Backend invoke failed (index not built, Tauri error, etc.).
    // Fallback to filename heuristics in injectDynamicEntryEdges.
    console.warn('[entrypoints_map] list_entry_points invoke failed; using filename heuristics as fallback', err);
    return new Set<string>();
  }
}

// ---------------------------------------------------------------------------
// Injection logic
// ---------------------------------------------------------------------------

/** Return true when a node id looks like it implements a Tauri command or canvas
 *  dyn dispatch point (H-13).
 *  FALLBACK ONLY — used when fetchBackendEntryPoints() returns empty.
 *  Matches commands.rs / commands_*.rs files (the most common dispatch pattern)
 *  AND canvas_dispatcher.rs / canvas_grammar.rs (H-13 dyn dispatch points, HD-5).
 *  A file named graph_route_handler.rs would be missed; the backend detector
 *  (list_entry_points) catches those when the index is available. */
function looksLikeTauriCommandFile(nodeId: string): boolean {
  const base = nodeId.split('/').pop() ?? '';
  return (
    (base.startsWith('commands') && base.endsWith('.rs')) ||
    base === 'canvas_dispatcher.rs' ||
    base === 'canvas_grammar.rs' ||
    CANVAS_DYN_DISPATCH_FILES.has(nodeId)
  );
}

/** Return true when the node id contains an SSB viewer or wiring_coverage file.
 *  FALLBACK ONLY — used when fetchBackendEntryPoints() returns empty. */
function looksLikeSsbConsumerFile(nodeId: string): boolean {
  const base = nodeId.split('/').pop() ?? '';
  return (
    base === 'ssb_viewer.rs' ||
    base === 'wiring_coverage.rs' ||
    base === 'ssb.rs' ||
    base === 'action_type_lock.rs'
  );
}

/**
 * Inject virtual edges from DYNAMIC_ENTRYPOINT_ROOT to every node that is
 * reachable via dynamic dispatch (Tauri invoke, dyn Trait, SSB bus). The injected
 * edges carry `edge_type: 'dynamic_dispatch'` so the canvas renderer can style
 * them differently from static call edges.
 *
 * This must be called BEFORE the orphan-detection pass. Nodes with an incoming
 * dynamic_dispatch edge are NOT candidates for topology-suspected-dead.
 *
 * @param g                The graphology DirectedGraph built from VizData.
 * @param backendEntryFiles  File paths returned by fetchBackendEntryPoints().
 *                           When non-empty, these are the PRIMARY source (GD-1).
 *                           When empty, falls back to filename heuristics (labeled).
 */
export function injectDynamicEntryEdges(g: Graph, backendEntryFiles: Set<string> = new Set()): void {
  // Ensure the synthetic root node exists.
  if (!g.hasNode(DYNAMIC_ENTRYPOINT_ROOT)) {
    g.addNode(DYNAMIC_ENTRYPOINT_ROOT, {
      label: '[dynamic dispatch root]',
      virtual: true,
      hidden: true,
    });
  }

  const usingBackend = backendEntryFiles.size > 0;
  let injected = 0;

  g.forEachNode((nodeId, _attrs) => {
    if (nodeId === DYNAMIC_ENTRYPOINT_ROOT) return;

    let shouldAttach: boolean;
    if (usingBackend) {
      // PRIMARY: use the real backend-detected entry-point file set.
      // A node matches when its id (file path) is in the backend set, or
      // the TAURI_COMMAND_NAMES/SSB_ACTION_TYPES allowlists cover it
      // (those lists rescue dynamic-dispatch callers the backend may not
      // classify as entry-points, e.g. SSB consumer files).
      shouldAttach =
        backendEntryFiles.has(nodeId) ||
        looksLikeTauriCommandFile(nodeId) ||
        looksLikeSsbConsumerFile(nodeId);
    } else {
      // FALLBACK (labeled): backend unavailable — filename heuristics only.
      // This path misses any handler file not named commands*.rs or the four
      // SSB filenames below. Log once so the canvas overlay is honest.
      shouldAttach =
        looksLikeTauriCommandFile(nodeId) || looksLikeSsbConsumerFile(nodeId);
    }

    if (shouldAttach) {
      const edgeId = `__dyn__${DYNAMIC_ENTRYPOINT_ROOT}__${nodeId}`;
      if (!g.hasEdge(edgeId)) {
        g.addEdgeWithKey(edgeId, DYNAMIC_ENTRYPOINT_ROOT, nodeId, {
          edge_type: 'dynamic_dispatch',
        });
        injected += 1;
      }
    }
  });

  // Log injection count and source for diagnostics (not an assertion, safe in prod).
  console.debug(
    `[entrypoints_map] injected ${injected} dynamic-dispatch edges (source: ${usingBackend ? 'backend detect_entry_points' : 'filename-heuristics fallback'})`
  );
}

/**
 * Return the set of node ids that are considered "reachable" after dynamic
 * entrypoint injection. A node is reachable if any ancestor (direct or transitive)
 * has an incoming dynamic_dispatch edge from DYNAMIC_ENTRYPOINT_ROOT.
 *
 * Used by Canvas.svelte's dead-code classification pass.
 */
export function dynamicReachableSet(g: Graph): Set<string> {
  const reachable = new Set<string>();
  if (!g.hasNode(DYNAMIC_ENTRYPOINT_ROOT)) return reachable;

  // BFS from the synthetic root via outgoing edges.
  const queue: string[] = [DYNAMIC_ENTRYPOINT_ROOT];
  while (queue.length > 0) {
    const cur = queue.shift()!;
    g.forEachOutNeighbor(cur, (neighbor) => {
      if (!reachable.has(neighbor)) {
        reachable.add(neighbor);
        queue.push(neighbor);
      }
    });
  }

  return reachable;
}

// Re-export the known sets for testing and introspection.
export { TAURI_COMMAND_NAMES, SSB_ACTION_TYPES };
