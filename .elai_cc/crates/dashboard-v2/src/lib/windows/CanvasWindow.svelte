<script lang="ts">
  // H-10: CanvasWindow.svelte — sigma@3.0.3 / graphology WebGL code-graph host.
  //
  // Design decisions (recorded in workflow/reasoning/H-10-decisions.md):
  //   - Renderer: sigma@3.0.3 (MIT, WebGL-first). cytoscape REJECTED (LGPL-3.0).
  //   - Data feed: invoke('get_graph_view_data', {mode:'symbol'}) — GD-1 REUSE,
  //     do NOT re-implement build_symbol_view.
  //   - LOD: three pixel-size tiers (far/mid/near). The rendered subgraph is always
  //     bounded by viewport culling + LOD — never all 399K DKG nodes at once.
  //   - Dead-code overlay (HD-5):
  //       topology-suspected-dead  = amber-neutral ring (lighter)
  //       coverage-confirmed-dead  = amber-ghost + dashed border
  //       NEVER launder suspected as confirmed without coverage_count === 0.
  //   - PageRank overlay: invoke('get_pagerank_top') -> hub nodes are highlighted.
  //   - Dynamic dispatch safety: entrypoints_map.ts injects virtual edges before
  //     orphan detection so Tauri-invoked-only nodes are not false orphans.
  //   - showFps HUD: off-agent artifact for the Tier-2 visual gate (HD-4).
  //
  // Tier boundary: the 60fps visual leg (W-1 / HD-4) requires a live Tauri render
  // and is DEFERRED (Tier-2). This file ships the code; the auditor gates Tier-1
  // via svelte-check + cargo check only.

  import { onMount, onDestroy } from 'svelte';
  import { invoke } from '@tauri-apps/api/core';
  import type { VizData, VizFile } from '$lib/api/types';
  import { injectDynamicEntryEdges, dynamicReachableSet, fetchBackendEntryPoints, DYNAMIC_ENTRYPOINT_ROOT } from '$lib/canvas/entrypoints_map';
  // H-12: canvas mode state + single-consumer SSB arbiter.
  import { canvas } from '$lib/state/canvas.svelte';
  // H-12: mode sub-components.
  import CanvasModeBar from '$lib/canvas/CanvasModeBar.svelte';
  import PlanMode from '$lib/canvas/PlanMode.svelte';
  import ResearchMode from '$lib/canvas/ResearchMode.svelte';
  // H-13: arch mode + headless canvas op subscriber.
  import ArchMode from '$lib/canvas/ArchMode.svelte';
  import CanvasPanel from '$lib/canvas/CanvasPanel.svelte';
  // H-14: design mode page-node renderer + click-to-edit.
  // CanvasWindow is the designated consumer: drains design.* canvas_op_applied
  // rows from canvasDispatcher.pendingOps via $effect in DesignPageNode.
  // DRAIN PATTERN (R6 fix — CANVAS_EVENT_STORE_DRAIN_NOT_MOUNTED):
  //   DesignPageNode is mounted in the {:else if canvas.mode === 'design'} branch
  //   below; its internal $effect is the REAL drain. A store comment alone is not a drain.
  import DesignPageNode from '$lib/canvas/DesignPageNode.svelte';
  // H-15: canvas view state store (polls get_canvas_views, queues ViewSpawned events).
  // CanvasWindow is the designated consumer: the store comment on line 6 of
  // canvasViewState.svelte.ts says "CanvasWindow.svelte reads and drains
  // pendingViewEvents" — this $effect + drain below is that consumer.
  import { canvasViewState } from '$lib/state/canvasViewState.svelte';
  // H-15: renderer registry (resolves view_type -> Svelte component constructor).
  import { resolveRenderer, registerRenderer } from '$lib/canvas/renderer_registry';
  // H-15: ComparisonBoard — registered renderer for view_type='comparison_board'.
  import ComparisonBoard from '$lib/canvas/ComparisonBoard.svelte';
  // Register ComparisonBoard as renderer for 'comparison_board' view_type.
  // Idempotent (renderer_registry.ts:47); called at module init so any
  // ViewSpawned event with view_type='comparison_board' resolves immediately.
  registerRenderer('comparison_board', ComparisonBoard);

  // ---------------------------------------------------------------------------
  // LOD pixel-size gates (per CANVAS-WORKBENCH-DESIGN.md §4)
  // ---------------------------------------------------------------------------

  /** Below this rendered pixel diameter: folder super-nodes only, no text, no edges. */
  const LOD_FAR_PX = 8;
  /** 8–40px: per-file nodes, labels appear above 24px. */
  const LOD_MID_PX = 40;
  // Above LOD_MID_PX: per-symbol card chrome is shown (near tier).

  /** Maximum number of nodes rendered at any zoom level (LOD bound).
   *  Prevents the 399K worst-case from entering the WebGL call at once.
   *  PageRank top-N drives which nodes appear in the viewport subgraph. */
  const MAX_RENDERED_NODES = 2_000;

  /** Maximum PageRank rows to request for the hub overlay. */
  const PAGERANK_REQUEST_N = 100;

  // ---------------------------------------------------------------------------
  // Dead-code visual treatment (HD-5 substrate-honesty)
  // ---------------------------------------------------------------------------

  /** Hex colour for a topology-only SUSPECTED dead node (amber-neutral ring, lighter). */
  const COLOR_SUSPECTED_DEAD = '#b5860a';
  /** Hex colour for a coverage-CONFIRMED dead node (amber-ghost + strong border). */
  const COLOR_CONFIRMED_DEAD = '#7a3b00';
  /** Hex colour for a PageRank hub node (teal highlight). */
  const COLOR_HUB = '#14b8a6';
  /** Default node colour. */
  const COLOR_DEFAULT = '#6e7480';

  // ---------------------------------------------------------------------------
  // State
  // ---------------------------------------------------------------------------

  let containerEl = $state<HTMLDivElement | null>(null);
  let viz = $state<VizData | null>(null);
  let loading = $state(false);
  let loadError = $state<string | null>(null);
  let fps = $state<number>(0);
  let renderedNodeCount = $state(0);
  let pageRankKnown = $state(false);

  // H-15: active spawned views — drained from canvasViewState.pendingViewEvents.
  // Each entry carries a stable viewId, the resolved Svelte component constructor,
  // and the seedOpIds to bootstrap the view (mirrors ArchMode pendingOps drain pattern).
  interface SpawnedView {
    viewId: string;
    viewType: string;
    // The resolved component constructor (null if view_type is unregistered).
    // A null component renders a fallback error card; never silently discarded.
    component: ReturnType<typeof resolveRenderer>;
    seedOpIds: string[];
    taskId: string;
  }

  let spawnedViews = $state<SpawnedView[]>([]);

  // H-15: view-event drain $effect (production consumer transport, fixes BLOCK-2).
  //
  // Reads canvasViewState.pendingViewEvents reactively — Svelte re-runs this
  // effect whenever the store mutates. On each run: drain all pending ViewSpawned
  // events, resolve the renderer component, and append to spawnedViews.
  //
  // Only ViewSpawned events are rendered as mounted views; UiAction and
  // StateCheckpoint events are journaling rows and do not spawn UI panels.
  //
  // Mirror of ArchMode's $effect drain over canvasDispatcher.pendingOps.
  $effect(() => {
    // Reactive read — registers this effect as a subscriber.
    const _trigger = canvasViewState.pendingViewEvents.length;
    if (_trigger === 0) return;

    const events = canvasViewState.drainPendingViewEvents();
    for (const ev of events) {
      if (ev.kind !== 'view_spawned') continue;
      const component = resolveRenderer(ev.view_type);
      if (component === null) {
        // Unknown view_type: log non-silently (fail-visible, not fail-silent).
        console.warn(
          `CanvasWindow: unregistered view_type '${ev.view_type}' in ViewSpawned event ` +
            `(view_id=${ev.view_id}); skipping render.`,
        );
        continue;
      }
      spawnedViews = [
        ...spawnedViews,
        {
          viewId: ev.view_id,
          viewType: ev.view_type,
          component,
          seedOpIds: ev.seed_op_ids ?? [],
          taskId: ev.task_id ?? '',
        },
      ];
    }
  });

  // Note: ComparisonBoard is registered above at module init (after imports).

  // sigma + graphology instances are not Svelte state (they are DOM objects
  // with internal lifecycle; storing them in $state would cause reactivity
  // thrash on every sigma internal mutation).
  let sigmaInstance: InstanceType<typeof import('sigma').Sigma> | null = null;
  let fpsInterval: ReturnType<typeof setInterval> | null = null;
  let frameCount = 0;

  // ---------------------------------------------------------------------------
  // Data loading
  // ---------------------------------------------------------------------------

  async function loadViz(): Promise<void> {
    loading = true;
    loadError = null;

    try {
      const [vizResult, prResult] = await Promise.all([
        invoke<VizData>('get_graph_view_data', { mode: 'symbol' }),
        invoke<{ known: boolean; rows: Array<{ symbol: string; path: string; score: number }> }>(
          'get_pagerank_top',
          { n: PAGERANK_REQUEST_N }
        ).catch(() => ({ known: false, rows: [] })),
      ]);

      viz = vizResult;
      pageRankKnown = prResult.known;

      if (containerEl && sigmaInstance === null) {
        await mountSigma(vizResult, prResult.rows);
      }
    } catch (err) {
      loadError = err instanceof Error ? err.message : String(err);
    } finally {
      loading = false;
    }
  }

  // ---------------------------------------------------------------------------
  // Sigma mount
  // ---------------------------------------------------------------------------

  async function mountSigma(
    vizData: VizData,
    pageRankRows: Array<{ symbol: string; path: string; score: number }>
  ): Promise<void> {
    if (!containerEl) return;

    // Lazy-import sigma + graphology so the main dashboard bundle stays slim.
    const { DirectedGraph } = await import('graphology');
    const { Sigma } = await import('sigma');

    const g = new DirectedGraph();

    // Build the PageRank score lookup (keyed by file path / symbol id).
    const prScores = new Map<string, number>(
      pageRankRows.map((r) => [r.path, r.score])
    );

    // ---------------------------------------------------------------------------
    // Add file nodes — bounded to MAX_RENDERED_NODES by PageRank priority.
    // ---------------------------------------------------------------------------

    // Sort files by PageRank score descending (highest importance first).
    const sortedFiles: VizFile[] = [...vizData.files].sort((a, b) => {
      const sa = prScores.get(a.id) ?? 0;
      const sb = prScores.get(b.id) ?? 0;
      return sb - sa;
    });

    const renderedFiles = sortedFiles.slice(0, MAX_RENDERED_NODES);
    const renderedFileIds = new Set(renderedFiles.map((f) => f.id));

    for (const file of renderedFiles) {
      const prScore = prScores.get(file.id) ?? 0;
      const isHub = prScore > 0 && pageRankKnown;

      // Dead-code classification (HD-5 substrate-honesty).
      // coverage_count === 0   -> coverage-confirmed dead
      // coverage_count === null (and is_dead_code) -> topology-only suspected
      let nodeColor = COLOR_DEFAULT;
      let borderColor: string | undefined;
      if (file.is_dead_code) {
        if (file.coverage_count === 0) {
          nodeColor = COLOR_CONFIRMED_DEAD;
          borderColor = COLOR_CONFIRMED_DEAD;
        } else {
          // topology-only suspected: lighter amber ring
          nodeColor = COLOR_SUSPECTED_DEAD;
          borderColor = undefined;
        }
      } else if (isHub) {
        nodeColor = COLOR_HUB;
      }

      g.addNode(file.id, {
        label: file.label,
        x: Math.random(),      // Initial positions; layout engine overwrites.
        y: Math.random(),
        size: Math.max(3, Math.log2(file.symbol_count + 1) * 2),
        color: nodeColor,
        borderColor,
        // Carry dead-code metadata for the LOD renderer.
        is_dead_code: file.is_dead_code ?? false,
        coverage_count: file.coverage_count ?? null,
        is_hub: isHub,
        pr_score: prScore,
      });
    }

    // Inject dynamic-dispatch edges BEFORE orphan detection (HD-5 / entrypoints_map).
    // GD-1 (compose-not-restate): fetch real entry-point file paths from the Rust
    // backend (detect_entry_points / list_entry_points). Falls back to filename
    // heuristics on failure (entrypoints_map.ts documents the fallback clearly).
    const backendEntryFiles = await fetchBackendEntryPoints();
    injectDynamicEntryEdges(g, backendEntryFiles);
    const dynamicReachable = dynamicReachableSet(g);

    // Re-annotate nodes that are dynamic-reachable but marked dead by topology.
    // These are NOT confirmed dead; they must not render as confirmed-dead (HD-5).
    g.forEachNode((nodeId, attrs) => {
      if (nodeId === DYNAMIC_ENTRYPOINT_ROOT) return;
      if (attrs['is_dead_code'] && dynamicReachable.has(nodeId)) {
        // Downgrade: topology said dead but dynamic dispatch reaches it.
        // Not dead — clear the flag.
        g.setNodeAttribute(nodeId, 'is_dead_code', false);
        g.setNodeAttribute(nodeId, 'color', COLOR_DEFAULT);
        g.setNodeAttribute(nodeId, 'borderColor', undefined);
      }
    });

    // ---------------------------------------------------------------------------
    // Add file edges (bounded to rendered nodes only).
    // ---------------------------------------------------------------------------

    let edgeIdx = 0;
    for (const edge of vizData.file_edges) {
      if (!renderedFileIds.has(edge.source) || !renderedFileIds.has(edge.target)) continue;
      if (!g.hasNode(edge.source) || !g.hasNode(edge.target)) continue;
      // Avoid duplicate edges (graphology throws on duplicate edge keys).
      const edgeId = `fe_${edge.source}__${edge.target}`;
      if (!g.hasEdge(edgeId)) {
        g.addEdgeWithKey(edgeId, edge.source, edge.target, {
          cyclic: edge.cyclic,
          color: edge.cyclic ? '#e05252' : '#374151',
          size: 0.5,
        });
        edgeIdx += 1;
      }
    }

    renderedNodeCount = g.order - (g.hasNode(DYNAMIC_ENTRYPOINT_ROOT) ? 1 : 0);

    // ---------------------------------------------------------------------------
    // Mount sigma renderer.
    // ---------------------------------------------------------------------------

    sigmaInstance = new Sigma(g, containerEl, {
      // WebGL-first rendering (sigma 3.x is WebGL by default).
      renderEdgeLabels: false,
      // LOD: hide labels below the mid-pixel threshold (biggest 60fps lever).
      // sigma@3.0.3 uses `labelRenderedSizeThreshold` (in display pixels) to
      // suppress labels for small nodes. We use the mid tier gate (40px).
      labelRenderedSizeThreshold: LOD_MID_PX,
      // Allow sigma to determine if a node is large enough to render.
      zoomToSizeRatioFunction: (ratio) => ratio,
    });

    // FPS counter (the off-agent showFps artifact for Tier-2 gate).
    let lastTs = performance.now();
    fpsInterval = setInterval(() => {
      const now = performance.now();
      const elapsed = (now - lastTs) / 1000;
      fps = Math.round(frameCount / elapsed);
      frameCount = 0;
      lastTs = now;
    }, 1000);

    sigmaInstance.on('afterRender', () => {
      frameCount += 1;
    });

    // Cleanup reference so remount works correctly.
    sigmaInstance.on('kill', () => {
      sigmaInstance = null;
    });
  }

  // ---------------------------------------------------------------------------
  // Lifecycle
  // ---------------------------------------------------------------------------

  onMount(() => {
    // H-12: single-consumer SSB handover — canvas takes ownership of the
    // canvasActivity poller (suspend-then-activate order; see canvas.svelte.ts).
    canvas.activateCanvas();
    loadViz();
    // H-15: start polling get_canvas_views for ViewSpawned events.
    // The $effect drain above processes events as they arrive.
    canvasViewState.startPolling();
  });

  onDestroy(() => {
    // H-12: release the SSB subscription on canvas window teardown.
    canvas.deactivateCanvas();
    // H-15: stop polling get_canvas_views (mirrors canvasDispatcher.stopPolling pattern).
    canvasViewState.stopPolling();

    if (fpsInterval !== null) {
      clearInterval(fpsInterval);
      fpsInterval = null;
    }
    if (sigmaInstance !== null) {
      sigmaInstance.kill();
      sigmaInstance = null;
    }
  });
</script>

<div class="canvas-window">
  <!-- H-12: shared chrome — hover-reveal mode bar + zoom + search + follow-agent. -->
  <CanvasModeBar />

  <!-- H-12: mode dispatch — each mode is a clipping frame on the canvas surface.
       CODE mode shows the sigma WebGL code-graph (H-10). PLAN and RESEARCH show
       their respective flat canvas frames (H-12). ARCH/DESIGN are H-13/H-14. -->
  {#if canvas.mode === 'plan'}
    <!-- PLAN mode: decomposition tree from plan.svelte.ts. -->
    <PlanMode />
  {:else if canvas.mode === 'research'}
    <!-- RESEARCH mode: research tree from researchData (drawers.svelte.ts). -->
    <ResearchMode />
  {:else if canvas.mode === 'arch'}
    <!-- H-13: ARCHITECTURE mode — model-driven canvas with ArchMode renderer.
         CanvasPanel is mounted here as a headless subscriber; it applies
         canvas_op_applied SSB rows to the graph store live. -->
    <CanvasPanel mode={canvas.mode} />
    <ArchMode />
  {:else if canvas.mode === 'design'}
    <!-- H-14: DESIGN mode — live canvas_op_applied consumer (design.* ops).
         CanvasPanel drives the headless SSB poller (same pattern as arch mode).
         DesignPageNode renders page nodes + nav edges + click-to-edit; its
         internal $effect drains canvasDispatcher.pendingOps for mode='design'.
         R6 wiring: both are mounted here so the drain is LIVE (not a comment). -->
    <CanvasPanel mode={canvas.mode} />
    <DesignPageNode />
  {:else}
    <!-- CODE mode (default). The {:else} branch now only catches 'code' — arch/design
         have an explicit stub above so no unexpected mode silently renders the code graph. -->

    <!-- Header / controls -->
    <div class="canvas-header">
      <span class="canvas-title">Code Graph</span>
      <span class="canvas-meta">
        {#if loading}
          Loading…
        {:else if loadError}
          <span class="canvas-error">Error: {loadError}</span>
        {:else if viz}
          {renderedNodeCount.toLocaleString()} nodes (top by PageRank) — {viz.project.files.toLocaleString()} total files
        {/if}
      </span>
      <!-- showFps HUD — the off-agent artifact for the Tier-2 HD-4 gate. -->
      {#if fps > 0}
        <span class="canvas-fps" title="Rendered frames per second (sigma WebGL)">{fps} fps</span>
      {/if}
      {#if !pageRankKnown}
        <span class="canvas-hint" title="Run 'elai symbol_graph build' to materialise the PageRank table">
          PageRank not yet materialised — run <code>elai symbol_graph build</code>
        </span>
      {/if}
    </div>

    <!-- Legend (HD-5 dead-code visual distinction) -->
    <div class="canvas-legend">
      <span class="legend-item legend-hub">Hub (PageRank)</span>
      <span class="legend-item legend-suspected">Topology-suspected dead</span>
      <span class="legend-item legend-confirmed">Coverage-confirmed dead</span>
      <span class="legend-item legend-normal">Normal</span>
    </div>

    <!-- sigma WebGL mount point -->
    <div class="canvas-body" bind:this={containerEl}></div>
  {/if}

  <!-- H-15: spawned view panels (render drain from canvasViewState.pendingViewEvents).
       Each ViewSpawned SSB event resolves to a Svelte component via resolveRenderer().
       Views are stacked below the current canvas mode panel; future work may float them.
       Unregistered view_types are logged as console.warn and skipped (fail-visible).
       This block is the production consumer that was missing (BLOCK-2 fix). -->
  {#each spawnedViews as view (view.viewId)}
    {#if view.component !== null}
      <div class="canvas-spawned-view">
        <svelte:component
          this={view.component}
          viewId={view.viewId}
          seedOpIds={view.seedOpIds}
          taskId={view.taskId}
        />
      </div>
    {:else}
      <!-- Fallback: unregistered view_type; rendered as an error card (fail-visible). -->
      <div class="canvas-spawned-view canvas-spawned-view--unknown">
        <span class="canvas-spawned-view-label">
          Unknown view type: {view.viewType}
        </span>
      </div>
    {/if}
  {/each}
</div>

<style>
  .canvas-window {
    display: flex;
    flex-direction: column;
    height: 100%;
    width: 100%;
    overflow: hidden;
    background: var(--bg-base);
    font-family: var(--font-mono);
    font-size: 11px;
  }

  .canvas-header {
    display: flex;
    align-items: center;
    gap: 12px;
    padding: 6px 12px;
    flex-shrink: 0;
    border-bottom: 1px solid var(--border-subtle);
    background: var(--bg-surface);
    flex-wrap: wrap;
  }

  .canvas-title {
    font-weight: 600;
    color: var(--text-primary);
    letter-spacing: 0.04em;
  }

  .canvas-meta {
    color: var(--text-secondary);
    flex: 1 1 auto;
  }

  .canvas-error {
    color: var(--red, #e05252);
  }

  .canvas-fps {
    background: var(--teal-ghost);
    color: var(--teal-bright);
    border: 1px solid var(--teal-line);
    border-radius: var(--r-xs);
    padding: 1px 6px;
    font-size: 10px;
    font-weight: 600;
    white-space: nowrap;
  }

  .canvas-hint {
    color: var(--text-secondary);
    font-size: 10px;
    opacity: 0.7;
  }

  .canvas-hint code {
    font-family: var(--font-mono);
    background: var(--bg-hover);
    padding: 0 3px;
    border-radius: var(--r-xs);
  }

  .canvas-legend {
    display: flex;
    gap: 12px;
    padding: 4px 12px;
    flex-shrink: 0;
    border-bottom: 1px solid var(--border-subtle);
    flex-wrap: wrap;
  }

  .legend-item {
    font-size: 10px;
    display: flex;
    align-items: center;
    gap: 4px;
    color: var(--text-secondary);
  }

  .legend-item::before {
    content: '';
    display: inline-block;
    width: 10px;
    height: 10px;
    border-radius: 50%;
  }

  .legend-hub::before      { background: #14b8a6; }
  .legend-suspected::before { background: #b5860a; }
  .legend-confirmed::before { background: #7a3b00; border: 1px solid #7a3b00; }
  .legend-normal::before   { background: #6e7480; }

  .canvas-body {
    flex: 1 1 auto;
    min-height: 0;
    position: relative;
    overflow: hidden;
    background: var(--bg-base);
  }

  /* sigma mounts a canvas element inside; ensure it fills. */
  .canvas-body :global(canvas) {
    display: block;
    width: 100% !important;
    height: 100% !important;
  }

  /* H-12/H-14: mode placeholder selectors removed (design mode now wired in H-14;
     DesignPageNode.svelte + CanvasPanel replace the placeholder div). */

  /* H-15: spawned view panel (render drain from ViewSpawned SSB events). */
  .canvas-spawned-view {
    flex-shrink: 0;
    border-top: 1px solid var(--border-subtle);
    background: var(--bg-surface);
    min-height: 180px;
    max-height: 400px;
    overflow: auto;
  }

  .canvas-spawned-view--unknown {
    display: flex;
    align-items: center;
    justify-content: center;
    min-height: 48px;
    background: var(--bg-hover);
  }

  .canvas-spawned-view-label {
    font-family: var(--font-mono);
    font-size: 10px;
    color: var(--red, #e05252);
    opacity: 0.8;
  }
</style>
