<script lang="ts">
  // H-14: DesignPageNode.svelte — APP/WEBSITE DESIGN mode canvas surface.
  //
  // Renders a design.page node that loads the ELAI-controlled proxied origin
  // in an iframe for live-preview. Nav edges connecting pages into the real
  // navigation graph are canvas_op_applied ops with mode='design'.
  //
  // Design decisions (workflow/reasoning/H-14-decisions.md):
  //   - The iframe loads the ELAI-controlled proxied origin ONLY (never the raw
  //     dev server). The dev-preview proxy strips X-Frame-Options + rewrites
  //     frame-ancestors (dev_preview_proxy.rs). For Tier-1, the iframe src is a
  //     placeholder when the proxy is not yet live (stub-8).
  //   - Click-to-edit: clicking an element reads its data-oid attr and calls
  //     resolve_oid to get {file, range, scope}. The Tauri command applies the
  //     patch via apply_recast_patch (deterministic tier).
  //   - GD-1: design.* ops ride canvas_op_applied (no new transport).
  //   - Design ops (design.page_node, design.nav_edge) are drained from
  //     canvasDispatcher.pendingOps in the $effect below. Only ops with
  //     mode='design' pass the dispatcher's mode-namespace gate.
  //
  // R6 wiring (CANVAS_EVENT_STORE_DRAIN_NOT_MOUNTED gotcha fix):
  //   CanvasWindow.svelte mounts this component AND the CanvasPanel in the
  //   {:else if canvas.mode === 'design'} branch. The $effect drain below is
  //   the REAL consumer of design mode canvas_op_applied rows — a store
  //   comment is NOT a drain (the bug that fired in H-13 + H-15).
  //
  // Tier boundary:
  //   - Tier-1: the drain logic + OidMap resolution + click-to-edit wiring.
  //     DesignPageNode shape, op tracking, and patch dispatch are live.
  //   - Tier-2 (stub-8): the LIVE painted-preview iframe (requires the
  //     dev-preview proxy to be running + the painted-preview screenshot
  //     off-agent done-gate to pass). The iframe src is a placeholder in Tier-1.
  //   - Tier-2 (stub-7): the LLM structural writeback tier (GD-5 gate).
  //   - Tier-2 (T2-D): the writeback-corruption rate measurement on >= 3 runs.
  //
  // Honest limit (GD-5): the live proxy only fronts dev servers ELAI controls.
  // An arbitrary deployed third-party hard-CSP site is out of scope. This is
  // stated in the UI as an informational note, not hidden.

  import { onMount, onDestroy } from 'svelte';
  import { canvasDispatcher } from '../state/canvasDispatcher.svelte';

  // ---------------------------------------------------------------------------
  // Named constants
  // ---------------------------------------------------------------------------

  /** Design mode op namespace (matches DESIGN_NAMESPACE in canvas_dispatcher.rs). */
  const DESIGN_NAMESPACE = 'design';

  /** Maximum number of design page nodes tracked (LOD bound). */
  const MAX_DESIGN_NODES = 50;

  /** Maximum number of nav edges tracked. */
  const MAX_DESIGN_EDGES = 200;

  // ---------------------------------------------------------------------------
  // Types
  // ---------------------------------------------------------------------------

  interface DesignPageNodeData {
    nodeId: string;
    url: string;
    label: string;
    /** Opaque data-oid of the selected element (null when no selection). */
    selectedOid: string | null;
  }

  interface DesignNavEdge {
    fromId: string;
    toId: string;
    label: string;
  }

  // ---------------------------------------------------------------------------
  // State
  // ---------------------------------------------------------------------------

  /** Registered design page nodes (design.page_node ops). */
  let pageNodes = $state<DesignPageNodeData[]>([]);

  /** Navigation edges connecting page nodes (design.nav_edge ops). */
  let navEdges = $state<DesignNavEdge[]>([]);

  /** Count of rejected ops (dangling-id or wrong mode). */
  let rejectedOps = $state<number>(0);

  /** Currently selected page node id (null = no selection). */
  let selectedNodeId = $state<string | null>(null);

  /** Whether the resolve_oid call is in flight. */
  let resolving = $state<boolean>(false);

  /** Last resolve_oid result for display. */
  let resolveResult = $state<{ file: string; range: number[]; scope: string } | null>(null);

  /** Last resolve_oid error (null when clean). */
  let resolveError = $state<string | null>(null);

  // ---------------------------------------------------------------------------
  // Design op drain $effect (R6 fix — CANVAS_EVENT_STORE_DRAIN_NOT_MOUNTED)
  //
  // This $effect is the REAL consumer of design mode canvas_op_applied rows.
  // It mirrors the ArchMode $effect drain pattern exactly (H-13 audit lesson).
  // Drains canvasDispatcher.pendingOps, filters ops with mode='design', and
  // applies each to the design data model (page nodes + nav edges).
  //
  // R6 wiring is confirmed: CanvasWindow.svelte mounts <DesignPageNode /> in
  // the {:else if canvas.mode === 'design'} branch, so this $effect runs
  // whenever the design mode is active and pendingOps mutates.
  // ---------------------------------------------------------------------------

  $effect(() => {
    // Reactive read — registers this effect as a subscriber.
    const _trigger = canvasDispatcher.pendingOps.length;
    if (_trigger === 0) return;

    const ops = canvasDispatcher.drainPendingOps();
    for (const op of ops) {
      // Filter: only process design.* ops (defensive; dispatcher already gates by mode).
      if (op.mode !== DESIGN_NAMESPACE) {
        // Ops from other modes should not reach this drain. Log non-silently.
        console.warn(
          `DesignPageNode: unexpected op mode='${op.mode}' (expected '${DESIGN_NAMESPACE}'); ` +
            `kind='${op.kind}' node_id='${op.node_id}' — skipping`,
        );
        rejectedOps += 1;
        continue;
      }
      applyDesignOp(op);
    }
  });

  /**
   * Apply a single validated design canvas op to the local state.
   *
   * Supported op kinds (riding existing CanvasOpKind arms per GD-1):
   *   - node_created (mode='design') -> design.page_node: register a page node.
   *   - edge_connected (mode='design') -> design.nav_edge: register a nav edge.
   *   - label_edited (mode='design') -> update page node label.
   *   - annotation_added (mode='design') -> element selection (sets selectedOid).
   *
   * Unrecognised kinds are logged non-silently (never silently dropped).
   */
  function applyDesignOp(op: import('../api/stub').CanvasOpDto): void {
    switch (op.kind) {
      case 'node_created': {
        if (pageNodes.length >= MAX_DESIGN_NODES) {
          console.warn(
            `DesignPageNode: MAX_DESIGN_NODES (${MAX_DESIGN_NODES}) reached; ` +
              `dropping page node op for node_id='${op.node_id}'`,
          );
          rejectedOps += 1;
          return;
        }
        const url = typeof op.op_data['url'] === 'string' ? (op.op_data['url'] as string) : '';
        const label = typeof op.op_data['label'] === 'string' ? (op.op_data['label'] as string) : op.node_id;
        const existing = pageNodes.find((n) => n.nodeId === op.node_id);
        if (!existing) {
          pageNodes = [
            ...pageNodes,
            { nodeId: op.node_id, url, label, selectedOid: null },
          ];
        }
        break;
      }

      case 'edge_connected': {
        if (navEdges.length >= MAX_DESIGN_EDGES) {
          console.warn(`DesignPageNode: MAX_DESIGN_EDGES (${MAX_DESIGN_EDGES}) reached`);
          rejectedOps += 1;
          return;
        }
        const fromId = typeof op.op_data['from_id'] === 'string' ? (op.op_data['from_id'] as string) : '';
        const toId = typeof op.op_data['to_id'] === 'string' ? (op.op_data['to_id'] as string) : '';
        const edgeLabel = typeof op.op_data['label'] === 'string' ? (op.op_data['label'] as string) : '';
        if (fromId && toId) {
          navEdges = [...navEdges, { fromId, toId, label: edgeLabel }];
        }
        break;
      }

      case 'label_edited': {
        // Update an existing page node's label.
        const newLabel = typeof op.op_data['label'] === 'string' ? (op.op_data['label'] as string) : '';
        pageNodes = pageNodes.map((n) =>
          n.nodeId === op.node_id ? { ...n, label: newLabel } : n,
        );
        break;
      }

      case 'annotation_added': {
        // Element selection: op_data.oid carries the selected element's data-oid.
        const oid = typeof op.op_data['oid'] === 'string' ? (op.op_data['oid'] as string) : null;
        if (oid) {
          // Find the page node for the current selection (by task_id association).
          const nodeId = typeof op.op_data['node_id'] === 'string'
            ? (op.op_data['node_id'] as string)
            : op.node_id;
          pageNodes = pageNodes.map((n) =>
            n.nodeId === nodeId ? { ...n, selectedOid: oid } : n,
          );
          // Trigger resolve_oid for the selected element.
          void resolveSelectedOid(oid);
        }
        break;
      }

      default: {
        console.warn(
          `DesignPageNode: unrecognised design op kind='${op.kind}' ` +
            `node_id='${op.node_id}' — non-silent skip`,
        );
        rejectedOps += 1;
      }
    }
  }

  // ---------------------------------------------------------------------------
  // Click-to-edit: resolve oid -> AST range via Tauri command
  // ---------------------------------------------------------------------------

  /**
   * Resolve a data-oid to its {file, range, scope} via the server-side OidMap.
   * Called when an element inside the preview iframe is clicked (annotation_added
   * op) or when the user clicks a node in the design canvas.
   *
   * Uses the resolve_oid Tauri command (commands_h14.rs).
   */
  async function resolveSelectedOid(oid: string): Promise<void> {
    resolving = true;
    resolveError = null;
    resolveResult = null;
    try {
      const { invoke } = await import('@tauri-apps/api/core');
      const result = await invoke<{ oid: string; file: string; range: number[]; scope: string }>(
        'resolve_oid',
        { oid },
      );
      resolveResult = { file: result.file, range: result.range, scope: result.scope };
    } catch (err) {
      resolveError = String(err);
      console.warn(`DesignPageNode: resolve_oid failed for oid='${oid}': ${err}`);
    } finally {
      resolving = false;
    }
  }

  // ---------------------------------------------------------------------------
  // Lifecycle
  // ---------------------------------------------------------------------------

  onMount(() => {
    // Nothing to initialise in Tier-1 (no graphology instance; no sigma mount).
    // The $effect drain starts automatically when pendingOps mutates.
  });

  onDestroy(() => {
    // Clear local state on unmount.
    pageNodes = [];
    navEdges = [];
    selectedNodeId = null;
    resolveResult = null;
  });
</script>

<div class="design-mode">
  <div class="design-mode__header">
    <span class="design-mode__label">Design</span>
    {#if rejectedOps > 0}
      <span class="design-mode__rejected" title="Ops rejected (dangling-id or wrong mode)">
        {rejectedOps} op{rejectedOps === 1 ? '' : 's'} rejected
      </span>
    {/if}
    <!-- Honest limit: proxy is Tier-2 deferred (stub-8). -->
    <span class="design-mode__tier-note" title="Live preview requires the dev-preview proxy (Tier-2 stub-8; GD-5 honest limit)">
      Live preview: Tier-2 pending
    </span>
  </div>

  <!-- Page node list -->
  {#if pageNodes.length > 0}
    <div class="design-mode__nodes">
      <span class="design-mode__section-label">Pages ({pageNodes.length})</span>
      <ul class="design-mode__node-list">
        {#each pageNodes as node (node.nodeId)}
          <div
            class="design-mode__node-item"
            class:design-mode__node-item--selected={selectedNodeId === node.nodeId}
            role="button"
            tabindex="0"
            onclick={() => { selectedNodeId = node.nodeId; }}
            onkeydown={(e) => { if (e.key === 'Enter' || e.key === ' ') selectedNodeId = node.nodeId; }}
          >
            <span class="design-mode__node-label">{node.label}</span>
            {#if node.url}
              <span class="design-mode__node-url" title={node.url}>{node.url}</span>
            {/if}
            {#if node.selectedOid}
              <span class="design-mode__node-oid" title="Selected element data-oid">
                oid: {node.selectedOid}
              </span>
            {/if}
          </div>
        {/each}
      </ul>
    </div>
  {:else}
    <div class="design-mode__empty">
      <p class="design-mode__empty-text">
        No design page nodes yet. The model emits <code>canvas_create_node</code> with
        <code>mode='design'</code> to add pages to the design canvas.
      </p>
    </div>
  {/if}

  <!-- Nav edges -->
  {#if navEdges.length > 0}
    <div class="design-mode__edges">
      <span class="design-mode__section-label">Nav edges ({navEdges.length})</span>
      <ul class="design-mode__edge-list">
        {#each navEdges as edge, i (i)}
          <li class="design-mode__edge-item">
            {edge.fromId} → {edge.toId}
            {#if edge.label}
              <span class="design-mode__edge-label">({edge.label})</span>
            {/if}
          </li>
        {/each}
      </ul>
    </div>
  {/if}

  <!-- Click-to-edit resolve result -->
  {#if resolving}
    <div class="design-mode__resolve-status">Resolving element…</div>
  {:else if resolveResult}
    <div class="design-mode__resolve-result">
      <span class="design-mode__resolve-file">{resolveResult.file}</span>
      <span class="design-mode__resolve-range">
        L{resolveResult.range[0]}:{resolveResult.range[1]}–L{resolveResult.range[2]}:{resolveResult.range[3]}
      </span>
      <span class="design-mode__resolve-scope">{resolveResult.scope}</span>
    </div>
  {:else if resolveError}
    <div class="design-mode__resolve-error" title={resolveError}>
      Resolve failed: {resolveError}
    </div>
  {/if}

  <!-- Tier-2 deferred: live iframe preview placeholder (stub-8).
       The iframe loads the ELAI-controlled proxied origin; for Tier-1 the src
       is a placeholder because the proxy TCP listener is not yet live.
       TODO[H-14.proxy.live]: replace src with the live proxy URL from
       start_dev_proxy (commands_h14.rs). stub-8. -->
  <div class="design-mode__preview-placeholder" title="Live preview requires dev-preview proxy (Tier-2 stub-8)">
    <span class="design-mode__preview-placeholder-text">
      Live iframe preview — Tier-2 pending (stub-8)<br />
      Point DESIGN mode at a running dev server to activate.
    </span>
  </div>
</div>

<style>
  /* Flat-matte work surface, role tokens only. Blue REMOVED (DESIGN §10
     ban) — the one accent is teal. */
  .design-mode {
    display: flex;
    flex-direction: column;
    height: 100%;
    min-height: 0;
    padding: 0.75rem;
    gap: 0.75rem;
    color: var(--read);
    font-family: var(--font-mono);
    font-size: 0.75rem;
    background: var(--bg);
  }

  .design-mode__header {
    display: flex;
    align-items: center;
    gap: 0.75rem;
    padding-bottom: 0.5rem;
    border-bottom: 1px solid var(--line);
  }

  .design-mode__label {
    font-weight: 500;
    color: var(--text);
    letter-spacing: 0.16em;
    text-transform: uppercase;
    font-size: 0.7rem;
  }

  /* warn reads via triangle + --text, no amber */
  .design-mode__rejected {
    color: var(--text);
    font-size: 0.65rem;
  }
  .design-mode__rejected::before { content: '▲ '; }

  .design-mode__tier-note {
    margin-left: auto;
    color: var(--dim);
    font-size: 0.65rem;
    font-style: italic;
  }

  .design-mode__section-label {
    display: block;
    font-size: 0.65rem;
    color: var(--dim);
    text-transform: uppercase;
    letter-spacing: 0.12em;
    margin-bottom: 0.25rem;
  }

  .design-mode__node-list,
  .design-mode__edge-list {
    list-style: none;
    padding: 0;
    margin: 0;
    display: flex;
    flex-direction: column;
    gap: 0.25rem;
  }

  /* nodes are inset wells (--panel below --bg), rising to --raised on hover */
  .design-mode__node-item {
    display: flex;
    flex-direction: column;
    padding: 0.35rem 0.5rem;
    border-radius: var(--r-sm);
    border: 1px solid var(--line);
    cursor: pointer;
    background: var(--panel);
    transition: border-color var(--dur-fast) var(--ease-out), background var(--dur-fast) var(--ease-out);
  }
  .design-mode__node-item:hover {
    border-color: var(--d-teal-soft);
    background: var(--raised);
  }
  .design-mode__node-item--selected {
    border-color: var(--d-teal);
    background: var(--d-teal-faint);
  }

  .design-mode__node-label {
    font-weight: 500;
    color: var(--text);
  }

  .design-mode__node-url,
  .design-mode__node-oid {
    color: var(--dim);
    font-size: 0.65rem;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .design-mode__edge-item {
    padding: 0.2rem 0.4rem;
    color: var(--dim);
  }

  .design-mode__edge-label {
    color: var(--read);
  }

  .design-mode__empty {
    flex: 1;
    display: flex;
    align-items: center;
    justify-content: center;
  }

  .design-mode__empty-text {
    color: var(--dim);
    text-align: center;
    line-height: 1.6;
    max-width: 32rem;
  }

  .design-mode__empty-text code {
    font-family: inherit;
    background: var(--sink);
    padding: 0.1rem 0.3rem;
    border-radius: var(--r-xs);
    color: var(--d-teal-bright);
  }

  .design-mode__resolve-status {
    color: var(--dim);
    font-style: italic;
    font-size: 0.7rem;
  }

  .design-mode__resolve-result {
    display: flex;
    flex-direction: column;
    gap: 0.15rem;
    padding: 0.4rem 0.5rem;
    border: 1px solid var(--d-teal-soft);
    border-radius: var(--r-sm);
    background: var(--panel);
    font-size: 0.7rem;
  }

  .design-mode__resolve-file {
    color: var(--text);
    font-weight: 500;
  }

  .design-mode__resolve-range { color: var(--dim); }
  .design-mode__resolve-scope { color: var(--read); }

  /* resolve error reads via text (mono-inversion lives at the glyph layer) */
  .design-mode__resolve-error {
    color: var(--text);
    font-size: 0.7rem;
  }

  .design-mode__preview-placeholder {
    flex: 1;
    min-height: 8rem;
    display: flex;
    align-items: center;
    justify-content: center;
    border: 1px dashed var(--line2);
    border-radius: var(--r-md);
    background: var(--sink);
  }

  .design-mode__preview-placeholder-text {
    text-align: center;
    color: var(--dim);
    line-height: 1.8;
    font-style: italic;
  }
</style>
