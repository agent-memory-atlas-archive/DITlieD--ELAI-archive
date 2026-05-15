// ============================================================
// graph.svelte.ts — Cross-window store for the GraphWindow
//
// Svelte 5 runes ONLY (NO `$:` reactive labels). Other windows
// (PlanWindow, ActivityWindow, KnowledgeDrawer) push focus and
// impact-target requests through this store; GraphWindow listens
// via $effect and reacts.
//
// Why a store rather than a Tauri event-bus: cross-window state
// already passes through the Svelte runtime (workspace.svelte.ts
// pattern). A focus signal that bounces through Tauri serialise/
// deserialise pays dispatch cost without any benefit — both ends
// live in the same webview process.
// ============================================================

export type GraphMode = 'file' | 'symbol' | 'decomposition';

function createGraphStore() {
  // Node id (file path or symbol id) the GraphWindow should pan/select to
  // on its next render. Cleared after consumption to avoid loops.
  let focus_target = $state<string | null>(null);
  // File path or symbol name for which an impact overlay should be fetched.
  let impact_target = $state<string | null>(null);
  // Active graph mode. Persists across re-mounts (re-opening the GraphWindow tab keeps the mode).
  let mode = $state<GraphMode>('file');
  // Bumped whenever a consumer wants to force a refetch (live SSB variant
  // is deferred per B-33 D3; the GraphWindow currently polls every 30s
  // and reacts to this counter for explicit refresh).
  let refresh_tick = $state<number>(0);

  function focus_node(id: string) {
    focus_target = id;
  }

  function consume_focus(): string | null {
    const v = focus_target;
    focus_target = null;
    return v;
  }

  function request_impact(target: string) {
    impact_target = target;
  }

  function consume_impact(): string | null {
    const v = impact_target;
    impact_target = null;
    return v;
  }

  function set_mode(next: GraphMode) {
    mode = next;
  }

  function trigger_refresh() {
    refresh_tick = refresh_tick + 1;
  }

  function reset() {
    focus_target = null;
    impact_target = null;
    mode = 'file';
    refresh_tick = 0;
  }

  return {
    get focus_target() { return focus_target; },
    get impact_target() { return impact_target; },
    get mode() { return mode; },
    get refresh_tick() { return refresh_tick; },

    focus_node,
    consume_focus,
    request_impact,
    consume_impact,
    set_mode,
    trigger_refresh,
    reset,
  };
}

export const graph = createGraphStore();
