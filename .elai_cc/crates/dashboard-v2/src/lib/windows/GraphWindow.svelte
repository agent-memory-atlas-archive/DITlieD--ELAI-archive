<script lang="ts">
  // B-33 — Interactive graph viewer (cytoscape-backed).
  //
  // Svelte 5 runes ONLY (NO `$:`).
  // Cytoscape + dagre + fcose are lazy-imported via dynamic import() so the
  // initial dashboard bundle stays slim.
  //
  // Mode toggle (file/symbol/decomposition) and export buttons sit in the
  // header. The right-click handler (cxttap) on a node fetches an impact
  // overlay and applies a red-gradient class by hop depth.
  //
  // Live updates: SymbolGraphUpdated SSB variant ships in B-33 follow-up
  // (SCHEMA_VERSION 40). We listen for the Tauri "symbol_graph_updated"
  // event re-broadcast by spawn_symbol_graph_subscriber in lib.rs and
  // refetch on receipt. The 30s polling fallback is KEPT as a backstop
  // in case the listener registration races with a hot-reload or a row
  // is dropped under broadcast lag.

  import { onMount } from 'svelte';
  import { invoke } from '@tauri-apps/api/core';
  import { save as saveDialog } from '@tauri-apps/plugin-dialog';
  import { graph, type GraphMode } from '$lib/state/graph.svelte';
  import type {
    VizData,
    VizFile,
    VizSymbol,
    SymbolMode,
    ImpactOverlay,
  } from '$lib/api/types';
  import GraphSidebarPanel from './GraphSidebarPanel.svelte';
  import MermaidExportModal from './MermaidExportModal.svelte';

  // ----- Config ----------------------------------------------------------
  // 6 layouts per spec D11; dagre + fcose register dynamically.
  const LAYOUTS = ['dagre', 'fcose', 'concentric', 'breadthfirst', 'grid', 'circle'] as const;
  type LayoutName = typeof LAYOUTS[number];
  const POLL_MS = 30_000;

  // ----- State -----------------------------------------------------------
  let canvasEl = $state<HTMLDivElement | null>(null);
  let cy = $state<any>(null);
  let viz = $state<VizData | null>(null);
  let loading = $state<boolean>(false);
  let loadError = $state<string | null>(null);
  let selectedFile = $state<VizFile | null>(null);
  let selectedSymbols = $state<VizSymbol[]>([]);
  let layout = $state<LayoutName>('dagre');
  let searchTerm = $state<string>('');
  let mermaidString = $state<string | null>(null);
  let showMermaid = $state<boolean>(false);

  // ----- Derived ---------------------------------------------------------
  let mode = $derived<GraphMode>(graph.mode);
  let symbolModeDisabled = $derived(symbolModeFromViz(viz) !== 'Full');
  let symbolModeReason = $derived(reasonForMode(viz));

  function symbolModeFromViz(v: VizData | null): 'Full' | 'Capped' | 'Omitted' {
    if (!v) return 'Full';
    if (v.symbol_mode === 'Full') return 'Full';
    if (typeof v.symbol_mode === 'object' && 'Capped' in v.symbol_mode) return 'Capped';
    if (typeof v.symbol_mode === 'object' && 'Omitted' in v.symbol_mode) return 'Omitted';
    return 'Full';
  }

  function reasonForMode(v: VizData | null): string {
    if (!v) return '';
    const sm: SymbolMode = v.symbol_mode;
    if (sm === 'Full') return '';
    if ('Capped' in sm) return sm.Capped.reason;
    if ('Omitted' in sm) return sm.Omitted.reason;
    return '';
  }

  // ----- Loading ---------------------------------------------------------
  // FIX-A2 (B-33 follow-up): when fetchData detects an Omitted symbol-mode
  // result and downgrades the store to 'file', the mode-change $effect
  // (line ~325) would normally re-fire fetchData with the new mode. That
  // double-invoke is wasted work. We set `suppressNextModeFetch` to mark
  // the next mode-change as a no-op driven by us, not the user, so the
  // effect can skip its fetchData call exactly once.
  let suppressNextModeFetch = false;

  async function fetchData(forMode: GraphMode) {
    loading = true;
    loadError = null;
    try {
      const data = await invoke<VizData>('get_graph_view_data', {
        mode: forMode,
      });
      viz = data;
      // Block symbol mode + downgrade if commands.rs reported Omitted.
      // Guard against pointless transitions when we're already in 'file' so
      // we don't re-trigger the mode-change $effect for an idempotent set.
      if (
        forMode === 'symbol'
        && symbolModeFromViz(data) === 'Omitted'
        && graph.mode !== 'file'
      ) {
        suppressNextModeFetch = true;
        graph.set_mode('file');
      }
      rebuildCytoscape(data);
    } catch (err) {
      console.warn('[GraphWindow] get_graph_view_data failed', err);
      loadError = String(err);
      viz = null;
    } finally {
      loading = false;
    }
  }

  // ----- Cytoscape lifecycle --------------------------------------------
  async function ensureCytoscape() {
    if (cy || !canvasEl) return cy;
    // Lazy-load cytoscape + layout extensions so the initial bundle is thin.
    const [cytoscapeMod, dagreMod, fcoseMod] = await Promise.all([
      import('cytoscape'),
      import('cytoscape-dagre'),
      import('cytoscape-fcose'),
    ]);
    const cytoscape = cytoscapeMod.default;
    cytoscape.use(dagreMod.default);
    cytoscape.use(fcoseMod.default);
    cy = cytoscape({
      container: canvasEl,
      elements: [],
      wheelSensitivity: 0.2,
      style: [
        {
          selector: 'node',
          style: {
            'background-color': '#3578e5',
            'label': 'data(label)',
            'color': '#fff',
            'font-size': 9,
            'text-outline-color': '#000',
            'text-outline-width': 1,
            'text-valign': 'center',
            'width': 24,
            'height': 24,
          },
        },
        {
          selector: 'node[language = "rust"]',
          style: { 'background-color': '#dea584' },
        },
        {
          selector: 'node[language = "python"]',
          style: { 'background-color': '#3572a5' },
        },
        {
          selector: 'node[language = "typescript"], node[language = "ts"]',
          style: { 'background-color': '#3178c6' },
        },
        {
          selector: 'node[language = "svelte"]',
          style: { 'background-color': '#ff3e00' },
        },
        {
          selector: 'edge',
          style: {
            'width': 1,
            'line-color': '#888',
            'target-arrow-color': '#888',
            'target-arrow-shape': 'triangle',
            'curve-style': 'bezier',
          },
        },
        {
          selector: 'edge[cyclic = "true"]',
          style: {
            'line-color': '#ff5252',
            'target-arrow-color': '#ff5252',
            'line-style': 'dashed',
          },
        },
        {
          selector: 'node.impact-1',
          style: { 'background-color': '#ffb199', 'border-width': 2, 'border-color': '#ff5252' },
        },
        {
          selector: 'node.impact-2',
          style: { 'background-color': '#ff8a73', 'border-width': 2, 'border-color': '#d6342b' },
        },
        {
          selector: 'node.impact-3',
          style: { 'background-color': '#ff5247', 'border-width': 2, 'border-color': '#a4140e' },
        },
        {
          selector: 'node:selected',
          style: { 'border-color': '#0bf', 'border-width': 3 },
        },
        // -------------------------------------------------------------
        // C-24 (DRIFT-7 Path A) — decomposition-mode shape selectors.
        //
        // VizFile.level is populated by b33_build_decomposition_view
        // (Concept | Phase | Step | BabyStep). rebuildCytoscape passes
        // it through as `data.level`. The selectors are scoped to the
        // node[level=...] attribute, so they apply only when the
        // decomposition view publishes the field — file + symbol modes
        // leave level = undefined and these selectors are inert.
        // -------------------------------------------------------------
        {
          selector: 'node[level = "Concept"]',
          style: {
            'shape': 'round-rectangle',
            'width': 180,
            'height': 60,
            'border-width': 3,
            'background-color': '#283c5a',
            'color': '#fff',
            'font-size': 12,
          },
        },
        {
          selector: 'node[level = "Phase"]',
          style: {
            'shape': 'round-rectangle',
            'width': 140,
            'height': 48,
            'border-width': 2,
            'background-color': '#3b5a8a',
            'color': '#fff',
            'font-size': 11,
          },
        },
        {
          selector: 'node[level = "Step"]',
          style: {
            'shape': 'round-rectangle',
            'width': 100,
            'height': 36,
            'border-width': 1,
            'background-color': '#4f7eb7',
            'color': '#fff',
            'font-size': 10,
          },
        },
        {
          selector: 'node[level = "BabyStep"]',
          style: {
            'shape': 'ellipse',
            'width': 60,
            'height': 60,
            'border-width': 1,
            'background-color': '#7fa9d9',
            'color': '#fff',
            'font-size': 9,
          },
        },
        // C-24 (DRIFT-7 Path A) — per-status color classes for the
        // decomposition view. VizFile.status (when present) maps to one
        // of pending|running|passed|failed|blocked; rebuildCytoscape
        // adds the matching `.status-*` class so these overrides win
        // over the level-shape background-color above.
        {
          selector: 'node.status-pending',
          style: { 'background-color': '#9e9e9e' },
        },
        {
          selector: 'node.status-running',
          style: {
            'background-color': '#1976d2',
            'border-color': '#0bf',
            'border-width': 3,
            // cytoscape transitions: cycle border-color for a pulse hint.
            'transition-property': 'border-color',
            'transition-duration': 500,
          },
        },
        {
          selector: 'node.status-passed',
          style: { 'background-color': '#388e3c' },
        },
        {
          selector: 'node.status-failed',
          style: { 'background-color': '#d32f2f' },
        },
        {
          selector: 'node.status-blocked',
          style: { 'background-color': '#f57c00' },
        },
      ],
    });
    cy.on('tap', 'node', (evt: any) => onNodeTap(evt.target));
    cy.on('cxttap', 'node', (evt: any) => onNodeRightClick(evt.target));
    return cy;
  }

  function rebuildCytoscape(data: VizData) {
    if (!cy) return;
    // C-24 (DRIFT-7 Path A) — publish optional `level` + `status` data
    // attributes so the cytoscape `node[level = "Concept" | ...]` and
    // `.status-*` selectors can match. Both fields are populated only by
    // the decomposition view today (file/symbol views leave them
    // undefined and the selectors are inert). `classes` is set in the
    // initial add() so cytoscape applies the status background-color
    // override before the first paint.
    const nodeElements = data.files.map((f) => {
      const ndata: Record<string, unknown> = {
        id: f.id,
        label: f.label,
        language: f.language,
      };
      // VizFile.level / status are Option<String> from Rust — they may be
      // undefined / null / "". Only forward truthy strings so the
      // cytoscape selector `node[level = ...]` predicate stays clean.
      if (typeof f.level === 'string' && f.level.length > 0) {
        ndata.level = f.level;
      }
      if (typeof f.status === 'string' && f.status.length > 0) {
        ndata.status = f.status;
      }
      const classes =
        typeof f.status === 'string' && f.status.length > 0
          ? `status-${f.status}`
          : undefined;
      return classes !== undefined
        ? { data: ndata, classes }
        : { data: ndata };
    });
    const elements = [
      ...nodeElements,
      ...data.file_edges.map((e, i) => ({
        data: {
          id: `e-${i}-${e.source}->${e.target}`,
          source: e.source,
          target: e.target,
          cyclic: String(e.cyclic),
        },
      })),
    ];
    cy.elements().remove();
    cy.add(elements as any);
    runLayout(layout);
  }

  function runLayout(name: LayoutName) {
    if (!cy) return;
    cy.layout({ name, animate: false, fit: true, padding: 30 }).run();
  }

  function onNodeTap(node: any) {
    const id = node.id() as string;
    if (!viz) return;
    selectedFile = viz.files.find((f) => f.id === id) ?? null;
    selectedSymbols = viz.symbols_by_file?.[id] ?? [];
  }

  async function onNodeRightClick(node: any) {
    const id = node.id() as string;
    await applyImpactOverlay(id);
  }

  async function applyImpactOverlay(target: string) {
    try {
      const overlay = await invoke<ImpactOverlay>('get_impact_overlay', {
        target,
        depth: 3,
      });
      // Strip prior impact classes.
      cy?.nodes().removeClass('impact-1 impact-2 impact-3');
      for (const [depthKey, files] of Object.entries(overlay.by_depth ?? {})) {
        const depth = Number.parseInt(depthKey, 10);
        if (!Number.isFinite(depth) || depth === 0) continue;
        const cls = depth >= 3 ? 'impact-3' : depth === 2 ? 'impact-2' : 'impact-1';
        for (const f of files) {
          cy?.getElementById(f).addClass(cls);
        }
      }
    } catch (err) {
      console.warn('[GraphWindow] get_impact_overlay failed', err);
    }
  }

  function onSearchInput() {
    if (!cy) return;
    const term = searchTerm.trim().toLowerCase();
    // B-33 follow-up FIX-A1: class name must match the global CSS selector
    // `:global(.cy-faded)` at the bottom of this file. Previously we toggled
    // 'faded' which had no matching CSS rule, so the search filter visually
    // did nothing.
    if (term === '') {
      cy.elements().removeClass('cy-faded');
      return;
    }
    cy.nodes().forEach((n: any) => {
      const label = String(n.data('label') ?? '').toLowerCase();
      if (label.includes(term)) {
        n.removeClass('cy-faded');
      } else {
        n.addClass('cy-faded');
      }
    });
  }

  async function exportMermaid() {
    try {
      const text = await invoke<string>('export_graph_mermaid', {
        mode,
        maxNodes: 200,
      });
      mermaidString = text;
      showMermaid = true;
    } catch (err) {
      console.warn('[GraphWindow] export_graph_mermaid failed', err);
    }
  }

  async function exportPng() {
    if (!cy) return;
    try {
      const blob: Blob = cy.png({ output: 'blob', full: true, scale: 2 });
      const path = await saveDialog({
        filters: [{ name: 'PNG', extensions: ['png'] }],
        defaultPath: 'graph.png',
      });
      if (!path) return;
      // Browser/webview download — same trick as MermaidExportModal.
      const url = URL.createObjectURL(blob);
      const a = document.createElement('a');
      a.href = url;
      a.download = path.split(/[\\/]/).pop() ?? 'graph.png';
      document.body.appendChild(a);
      a.click();
      document.body.removeChild(a);
      URL.revokeObjectURL(url);
    } catch (err) {
      console.warn('[GraphWindow] PNG export failed', err);
    }
  }

  function setMode(next: GraphMode) {
    if (next === 'symbol' && symbolModeDisabled) return;
    graph.set_mode(next);
  }

  // ----- Effects ---------------------------------------------------------
  let mountedOnce = $state(false);

  onMount(() => {
    let alive = true;
    let pollTimer: ReturnType<typeof setInterval> | null = null;
    let unlisten: (() => void) | null = null;
    (async () => {
      await ensureCytoscape();
      if (!alive) return;
      mountedOnce = true;
      await fetchData(graph.mode);
      // B-33 follow-up FIX-B6: listen for the Tauri event re-broadcast by
      // spawn_symbol_graph_subscriber in lib.rs. On receipt, refetch the
      // current mode so the graph reflects the new build. Keep the 30s
      // poll as a backstop (defense-in-depth) for hot-reload / lagged
      // subscribers.
      try {
        const { listen } = await import('@tauri-apps/api/event');
        const u = await listen('symbol_graph_updated', () => {
          if (alive) void fetchData(graph.mode);
        });
        if (alive) {
          unlisten = u;
        } else {
          // Lost the race: tear down the listener immediately.
          try { u(); } catch (_) { /* unlisten failed — drop */ }
        }
      } catch (err) {
        console.warn('[GraphWindow] symbol_graph_updated listen() failed', err);
      }
      pollTimer = setInterval(() => {
        if (alive) void fetchData(graph.mode);
      }, POLL_MS);
    })();
    return () => {
      alive = false;
      if (pollTimer) clearInterval(pollTimer);
      if (unlisten) {
        try { unlisten(); } catch (_) { /* unlisten failed — drop */ }
      }
      cy?.destroy();
      cy = null;
    };
  });

  // React to mode changes from the store (e.g. from PlanWindow).
  $effect(() => {
    // Re-read on every store mutation; if the most recent mode-change was
    // driven by an internal auto-downgrade (FIX-A2 above), skip the fetch
    // exactly once to avoid a redundant invoke.
    const currentMode = graph.mode;
    if (!mountedOnce) return;
    if (suppressNextModeFetch) {
      suppressNextModeFetch = false;
      return;
    }
    void fetchData(currentMode);
  });

  // React to refresh ticks (manual refresh from store).
  $effect(() => {
    if (!mountedOnce) return;
    if (graph.refresh_tick > 0) void fetchData(graph.mode);
  });

  // Focus target consumer (other windows push a node id to focus on).
  $effect(() => {
    if (!cy) return;
    const id = graph.focus_target;
    if (!id) return;
    const node = cy.getElementById(id);
    if (node && node.length > 0) {
      cy.elements().unselect();
      node.select();
      cy.center(node);
    }
    // Drain the request so subsequent renders don't re-focus.
    graph.consume_focus();
  });

  // Impact target consumer (other windows push a file path to overlay).
  $effect(() => {
    const tgt = graph.impact_target;
    if (!tgt || !cy) return;
    void applyImpactOverlay(tgt);
    graph.consume_impact();
  });
</script>

<div class="graph-window">
  <header class="header">
    <div class="title">
      <h2>{viz?.project.name ?? 'Graph'}</h2>
      {#if viz}
        <span class="meta">{viz.project.files} files · {viz.project.edges} edges{viz.project.symbols !== null ? ` · ${viz.project.symbols} symbols` : ''}</span>
      {/if}
    </div>
    <div class="modes" role="group" aria-label="Graph mode">
      <button type="button" class:active={mode === 'file'} onclick={() => setMode('file')}>File</button>
      <button
        type="button"
        class:active={mode === 'symbol'}
        disabled={symbolModeDisabled}
        title={symbolModeDisabled ? symbolModeReason : ''}
        onclick={() => setMode('symbol')}
      >Symbol</button>
      <button type="button" class:active={mode === 'decomposition'} onclick={() => setMode('decomposition')}>Decomposition</button>
    </div>
  </header>

  <div class="body">
    <div class="canvas-wrap">
      <!-- svelte-ignore a11y_no_static_element_interactions -->
      <div class="canvas" bind:this={canvasEl}></div>
      {#if loading}
        <div class="overlay">Loading graph…</div>
      {:else if loadError}
        <div class="overlay overlay--error">Error: {loadError}</div>
      {:else if viz && viz.files.length === 0}
        <div class="overlay">
          {symbolModeReason || 'Run /index to populate the graph.'}
        </div>
      {/if}
    </div>
    <GraphSidebarPanel selectedNode={selectedFile} symbols={selectedSymbols} />
  </div>

  <footer class="footer">
    <input
      class="search"
      type="search"
      placeholder="Filter nodes…"
      bind:value={searchTerm}
      oninput={onSearchInput}
    />
    <label class="layout">
      Layout
      <select bind:value={layout} onchange={() => runLayout(layout)}>
        {#each LAYOUTS as L}
          <option value={L}>{L}</option>
        {/each}
      </select>
    </label>
    <button type="button" onclick={exportMermaid}>Export Mermaid</button>
    <button type="button" onclick={exportPng}>Export PNG</button>
  </footer>
</div>

{#if showMermaid && mermaidString !== null}
  <MermaidExportModal mermaid={mermaidString} onClose={() => { showMermaid = false; }} />
{/if}

<style>
  .graph-window {
    display: flex;
    flex-direction: column;
    height: 100%;
    width: 100%;
    background: var(--bg-base);
    color: var(--text-primary);
    font-family: var(--font-mono);
    font-size: 10px;
    overflow: hidden;
  }
  .header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 8px 12px;
    border-bottom: 1px solid var(--border-subtle);
  }
  .title h2 {
    margin: 0;
    font-size: 13px;
  }
  .title .meta {
    color: var(--text-secondary);
    font-size: 10px;
  }
  .modes {
    display: flex;
    gap: 4px;
  }
  .modes button {
    padding: 4px 10px;
    background: transparent;
    border: 1px solid var(--teal-line);
    color: var(--teal);
    border-radius: var(--r-sm);
    font-family: inherit;
    font-size: 10px;
    cursor: pointer;
  }
  .modes button.active {
    background: var(--teal-ghost);
    color: var(--teal-bright);
    border-color: var(--teal);
  }
  .modes button:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }
  .body {
    display: flex;
    flex: 1 1 auto;
    min-height: 0;
  }
  .canvas-wrap {
    position: relative;
    flex: 1 1 auto;
    min-width: 0;
  }
  .canvas {
    position: absolute;
    inset: 0;
    background: var(--bg-base);
  }
  .overlay {
    position: absolute;
    inset: 0;
    display: flex;
    align-items: center;
    justify-content: center;
    background: rgba(0, 0, 0, 0.35);
    color: var(--text-primary);
    pointer-events: none;
    text-align: center;
    padding: 16px;
  }
  .overlay--error {
    color: #ff7676;
  }
  .footer {
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 8px 12px;
    border-top: 1px solid var(--border-subtle);
  }
  .footer .search {
    flex: 1 1 auto;
    padding: 4px 8px;
    background: var(--bg-surface);
    border: 1px solid var(--border-subtle);
    border-radius: var(--r-sm);
    color: var(--text-primary);
    font-family: inherit;
    font-size: 10px;
  }
  .footer .layout {
    display: flex;
    align-items: center;
    gap: 6px;
    color: var(--text-secondary);
  }
  .footer select {
    background: var(--bg-surface);
    border: 1px solid var(--border-subtle);
    border-radius: var(--r-sm);
    color: var(--text-primary);
    font-family: inherit;
    font-size: 10px;
    padding: 2px 6px;
  }
  .footer button {
    padding: 4px 10px;
    background: transparent;
    border: 1px solid var(--teal-line);
    color: var(--teal);
    border-radius: var(--r-sm);
    cursor: pointer;
    font-family: inherit;
    font-size: 10px;
  }
  .footer button:hover {
    background: var(--teal-ghost);
  }
  /* faded class used by search filter */
  :global(.cy-faded) { opacity: 0.18; }
</style>
