<script lang="ts">
  // H-13: ArchMode.svelte — ARCHITECTURE mode canvas surface.
  //
  // Renders the model-driven architecture diagram as a graphology canvas graph.
  // The model emits canvas_create_node / canvas_connect / canvas_render_diagram
  // ops via the tool dispatch path; this component owns the graphology
  // DirectedGraph instance for the arch canvas, imports canvasDispatcher, and
  // drains pendingOps in a $effect (the production op-application transport).
  //
  // Op-application transport (BLOCK-1 fix, H-13 core deliverable):
  //   The $effect below reads canvasDispatcher.pendingOps, calls drainPendingOps(),
  //   and applies each op to the graphology instance g. Dangling-id ops (referencing
  //   a node_id or edge id that does not exist in g) are REJECTED non-silently:
  //   a console.warn is emitted and the rejectedOps counter increments.
  //
  // Tier boundary:
  //   The live sigma WebGL render (on-screen pixels) is Tier-2 (requires a live
  //   Tauri window). The drain logic + graphology mutations are Tier-1 and
  //   fully testable without a live window — the graphology instance `g` is at
  //   component scope, not inside a closure.
  //
  // Mermaid import bridge (CANVAS-WORKBENCH-DESIGN.md §8):
  //   - Mermaid/plantuml are IMPORT-ONLY via the existing server-side
  //     `export_graph_mermaid` Tauri command (commands.rs:2645). We do NOT
  //     add the @tldraw/mermaid NPM dep (source-available, license hazard) or
  //     the mermaid NPM package in Tier-1.
  //   - The model NEVER emits raw mermaid as its authoring channel (unreliable;
  //     MermaidSeqBench). The model emits validated canvas ops instead.
  //   - For diagram text export: MermaidExportModal.svelte is reused (existing).
  //
  // The 'arch' mode placeholder (H-12, wired:false) is NOW live (wired:true).
  // CanvasModeBar.svelte and canvas.svelte.ts are updated accordingly.

  import { onMount, onDestroy } from 'svelte';
  import { invoke } from '@tauri-apps/api/core';
  import { canvasDispatcher } from '../state/canvasDispatcher.svelte';
  import { applyOpToGraph } from './arch_canvas_ops';

  // ---------------------------------------------------------------------------
  // Props
  // ---------------------------------------------------------------------------

  interface Props {
    /** When true, show the mermaid text export panel. */
    showExport?: boolean;
  }

  let { showExport = false }: Props = $props();

  // ---------------------------------------------------------------------------
  // Graphology + sigma instances for arch canvas (Tier-1 drain target)
  //
  // g is at component scope so the drain $effect can apply ops without a
  // closure boundary (the BLOCK-1 issue: mountSigma kept `g` closure-local).
  // sigmaInstance is initialised in onMount when the container element is ready.
  // Both are plain variables (not $state): sigma/graphology have their own
  // internal lifecycle; storing them as reactive state causes thrash.
  // ---------------------------------------------------------------------------

  type DirectedGraphType = InstanceType<typeof import('graphology').DirectedGraph>;
  type SigmaType = InstanceType<typeof import('sigma').Sigma>;

  let g: DirectedGraphType | null = null;
  let sigmaInstance: SigmaType | null = null;
  let archContainerEl = $state<HTMLDivElement | null>(null);

  /** Count of ops rejected due to dangling-id (visible in UI as a health signal). */
  let rejectedOps = $state<number>(0);

  // ---------------------------------------------------------------------------
  // Op-application drain $effect (BLOCK-1 fix — the production transport)
  //
  // Reads canvasDispatcher.pendingOps reactively; when ops arrive, drains and
  // applies each to the graphology graph.  Dangling-id references are rejected
  // non-silently (WARN-1 fix folded in here per audit instruction).
  // ---------------------------------------------------------------------------

  $effect(() => {
    // Reading pendingOps registers this effect as a subscriber; Svelte will
    // re-run whenever pendingOps changes.
    const _trigger = canvasDispatcher.pendingOps.length;
    if (_trigger === 0) return;
    if (g === null) return;

    const ops = canvasDispatcher.drainPendingOps();
    for (const op of ops) {
      applyOp(g, op);
    }
    // Refresh the sigma renderer if it is live (Tier-2 path; no-op if null).
    sigmaInstance?.refresh();
  });

  /**
   * Apply a single validated CanvasOpDto to the graphology graph.
   *
   * Delegates to applyOpToGraph (arch_canvas_ops.ts) for the pure logic.
   * On rejection: console.warn (non-silent) + increment rejectedOps counter.
   * Dangling-id detection (WARN-1 fix) lives in arch_canvas_ops.ts.
   */
  function applyOp(graph: DirectedGraphType, op: import('../api/stub').CanvasOpDto): void {
    const result = applyOpToGraph(graph, op);
    if (!result.ok) {
      console.warn(`ArchMode: rejected op kind='${op.kind}' node_id='${op.node_id}': ${result.reason}`);
      rejectedOps += 1;
    }
  }

  // ---------------------------------------------------------------------------
  // Sigma mount (Tier-2 live render; no-op if container not available in test)
  // ---------------------------------------------------------------------------

  async function mountArchSigma(): Promise<void> {
    if (!archContainerEl || g === null) return;
    const { Sigma } = await import('sigma');
    sigmaInstance = new Sigma(g, archContainerEl, {
      renderEdgeLabels: false,
      labelRenderedSizeThreshold: 20,
    });
    sigmaInstance.on('kill', () => { sigmaInstance = null; });
  }

  // ---------------------------------------------------------------------------
  // Lifecycle
  // ---------------------------------------------------------------------------

  onMount(async () => {
    // Lazy-import graphology so the main bundle stays slim (mirrors mountSigma
    // pattern in CanvasWindow.svelte — GD-1 reuse of the same sigma/graphology
    // dynamic-import approach).
    const { DirectedGraph } = await import('graphology');
    g = new DirectedGraph();
    // Tier-2: mount sigma if the container element is available (live Tauri window).
    await mountArchSigma();
  });

  onDestroy(() => {
    if (sigmaInstance !== null) {
      sigmaInstance.kill();
      sigmaInstance = null;
    }
    g = null;
  });

  // ---------------------------------------------------------------------------
  // Mermaid text export (server-side reuse, GD-1)
  // ---------------------------------------------------------------------------

  let mermaidText = $state<string>('');
  let exportLoading = $state<boolean>(false);
  let exportError = $state<string | null>(null);

  async function loadMermaidExport(): Promise<void> {
    exportLoading = true;
    exportError = null;
    try {
      // Reuse the existing export_graph_mermaid Tauri command (commands.rs:2645).
      // This calls elai_knowledge::visualization::mermaid::generate_mermaid on
      // the server side. No new Rust dep required (GD-1).
      const result = await invoke<{ mermaid: string }>('export_graph_mermaid', {
        mode: 'symbol',
        maxNodes: 100,
      });
      mermaidText = result.mermaid;
    } catch (err) {
      exportError = String(err);
    } finally {
      exportLoading = false;
    }
  }
</script>

<div class="arch-mode">
  <div class="arch-mode__header">
    <span class="arch-mode__label">Architecture</span>
    <button
      class="arch-mode__export-btn"
      onclick={() => { showExport = !showExport; if (showExport) loadMermaidExport(); }}
      title="Export as Mermaid diagram"
    >
      Mermaid export
    </button>
    {#if rejectedOps > 0}
      <span class="arch-mode__rejected" title="Ops rejected due to dangling-id (node/edge not in graph)">
        {rejectedOps} op{rejectedOps === 1 ? '' : 's'} rejected
      </span>
    {/if}
  </div>

  <!-- Arch canvas: sigma mounts here when a live Tauri window is available (Tier-2).
       The graphology instance `g` is always present after onMount (Tier-1 drain target). -->
  <div class="arch-mode__canvas" bind:this={archContainerEl}></div>

  {#if showExport}
    <div class="arch-mode__export-panel">
      {#if exportLoading}
        <p class="arch-mode__loading">Generating diagram...</p>
      {:else if exportError}
        <p class="arch-mode__error">Export failed: {exportError}</p>
      {:else if mermaidText}
        <pre class="arch-mode__mermaid-text">{mermaidText}</pre>
        <button
          class="arch-mode__copy-btn"
          onclick={() => navigator.clipboard.writeText(mermaidText)}
        >
          Copy to clipboard
        </button>
      {:else}
        <p class="arch-mode__empty">No diagram generated yet. Ask the model to draw an architecture.</p>
      {/if}
    </div>
  {/if}
</div>

<style>
  /* Flat-matte work surface — role tokens only (DESIGN §1/§3). */
  .arch-mode {
    display: flex;
    flex-direction: column;
    gap: 8px;
    padding: 8px;
    height: 100%;
    min-height: 0;
    background: var(--bg);
    font-family: var(--font-mono);
  }

  .arch-mode__header {
    display: flex;
    align-items: center;
    gap: 12px;
  }

  .arch-mode__label {
    font-size: 10px;
    font-weight: 500;
    color: var(--text);
    text-transform: uppercase;
    letter-spacing: 0.16em;
  }

  .arch-mode__export-btn,
  .arch-mode__copy-btn {
    font-family: var(--font-mono);
    font-size: 10px;
    padding: 3px 8px;
    border: 1px solid var(--line);
    border-radius: var(--r-xs);
    background: var(--panel);
    color: var(--read);
    cursor: pointer;
    transition: background var(--dur-fast) var(--ease-out), color var(--dur-fast) var(--ease-out);
  }
  .arch-mode__export-btn:hover,
  .arch-mode__copy-btn:hover {
    background: var(--raised);
    color: var(--text);
  }

  .arch-mode__export-panel {
    border: 1px solid var(--line);
    border-radius: var(--r-sm);
    padding: 8px;
    background: var(--panel);
    max-height: 300px;
    overflow-y: auto;
  }

  .arch-mode__mermaid-text {
    font-family: var(--font-mono);
    font-size: 10px;
    white-space: pre-wrap;
    color: var(--read);
    margin: 0 0 8px;
  }

  .arch-mode__loading,
  .arch-mode__empty {
    font-size: 10px;
    color: var(--dim);
    margin: 0;
  }

  /* error reads via text + the surrounding state language, not red */
  .arch-mode__error {
    font-size: 10px;
    color: var(--text);
    margin: 0;
  }

  .arch-mode__canvas {
    flex: 1 1 auto;
    min-height: 200px;
    position: relative;
    overflow: hidden;
    background: var(--bg);
    border: 1px solid var(--line);
    border-radius: var(--r-sm);
  }
  .arch-mode__canvas :global(canvas) {
    display: block;
    width: 100% !important;
    height: 100% !important;
  }

  /* rejected-ops health signal — warn reads as a triangle prefix + --text,
     no amber (DESIGN §6: amber is dead). */
  .arch-mode__rejected {
    font-size: 10px;
    color: var(--text);
    background: transparent;
    border: 1px solid var(--line2);
    border-radius: var(--r-xs);
    padding: 1px 6px 1px 4px;
    cursor: default;
  }
  .arch-mode__rejected::before {
    content: '▲ ';
    color: var(--text);
  }
</style>
