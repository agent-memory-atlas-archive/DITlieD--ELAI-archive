<script lang="ts">
  // Canvas3DView — the standalone 3D layered canvas (PROCESS-altitude
  // preview). Layers = spawn depth: the root model's tree on the top slab,
  // every spawned sub-model on its own slab below, tethered by a bright
  // vertical trace. Self-contained: feeds itself from CanvasSim until the
  // SSB live layer is wired (swap `sim` for a store-driven C3Graph then).
  //
  // Interactions: drag = orbit, wheel = zoom, right-drag = pan,
  // hover = probe, click = select, double-click = expand/collapse subtree.

  import { onMount } from 'svelte';
  import { Canvas3DScene } from './scene';
  import { CanvasSim } from './sim';
  import type { C3Node } from './types';
  import type { QuinaryState } from '../canvas/canvas_state';
  import { QUINARY_STATES } from '../canvas/canvas_state';

  interface Props {
    /** sim seed; fixed default keeps captures reproducible */
    seed?: number;
    /** sim step interval in ms */
    tickMs?: number;
  }
  let { seed = 0xe1a1, tickMs = 420 }: Props = $props();

  let canvasEl: HTMLCanvasElement;
  let hostEl: HTMLDivElement;
  let scene: Canvas3DScene | null = null;
  // seed is intentionally construction-time only: the sim is deterministic
  // from its seed and never reseeds mid-flight.
  // svelte-ignore state_referenced_locally
  const sim = new CanvasSim({ seed });

  let paused = $state(false);
  let follow = $state(false);
  let fps = $state(0);
  let nodeCount = $state(0);
  let layerCount = $state(0);

  let hoverNode = $state<C3Node | null>(null);
  let hoverX = $state(0);
  let hoverY = $state(0);
  let selected = $state<C3Node | null>(null);

  const STATE_LABEL: Record<QuinaryState, string> = {
    done: 'DONE',
    active: 'ACTIVE',
    pending: 'PENDING',
    warn: 'WARN',
    fail: 'FAIL',
  };

  function refreshCounts(): void {
    nodeCount = sim.graph.nodes.size;
    layerCount = sim.graph.layerOrder.length;
  }

  function toggleExpand(nodeId: string): void {
    const n = sim.graph.nodes.get(nodeId);
    if (!n) return;
    n.expanded = !n.expanded;
    sim.graph.rev++;
    if (selected?.id === nodeId) selected = n;
  }

  function descend(n: C3Node): void {
    if (n.spawnedLayerId === null) return;
    const layer = sim.graph.layers.get(n.spawnedLayerId);
    if (layer) scene?.focusNode(layer.rootId);
  }

  onMount(() => {
    scene = new Canvas3DScene(canvasEl, sim.graph, {
      onHover(id, x, y) {
        hoverNode = id !== null ? (sim.graph.nodes.get(id) ?? null) : null;
        hoverX = x;
        hoverY = y;
      },
      onSelect(id) {
        selected = id !== null ? (sim.graph.nodes.get(id) ?? null) : null;
      },
      onToggleExpand: toggleExpand,
    });

    const ro = new ResizeObserver(() => {
      scene?.resize(hostEl.clientWidth, hostEl.clientHeight);
    });
    ro.observe(hostEl);
    scene.resize(hostEl.clientWidth, hostEl.clientHeight);
    refreshCounts();

    const tick = setInterval(() => {
      if (!paused) {
        sim.tick();
        refreshCounts();
      }
      fps = scene?.fps ?? 0;
    }, tickMs);

    return () => {
      clearInterval(tick);
      ro.disconnect();
      scene?.dispose();
      scene = null;
    };
  });

  $effect(() => {
    scene?.setFollowActive(follow);
  });
</script>

<div class="c3-host" bind:this={hostEl}>
  <canvas bind:this={canvasEl}></canvas>

  <!-- chrome bar -->
  <div class="c3-bar">
    <span class="c3-title">ELAI // CANVAS-3D</span>
    <span class="c3-stat">{layerCount} LAYERS · {nodeCount} NODES · {fps} FPS</span>
    <span class="c3-spacer"></span>
    <button class="c3-btn" class:on={follow} onclick={() => (follow = !follow)}>FOLLOW</button>
    <button class="c3-btn" class:on={paused} onclick={() => (paused = !paused)}>
      {paused ? 'RESUME' : 'PAUSE'}
    </button>
    <button class="c3-btn" onclick={() => scene?.resetCamera()}>RESET CAM</button>
  </div>

  <!-- legend -->
  <div class="c3-legend">
    {#each QUINARY_STATES as s (s)}
      <span class="c3-key"><i class="dot {s}"></i>{STATE_LABEL[s]}</span>
    {/each}
    <span class="c3-key"><i class="dot beam"></i>SPAWN BEAM</span>
  </div>

  <!-- hover tooltip -->
  {#if hoverNode}
    <div class="c3-tip" style:left="{hoverX + 14}px" style:top="{hoverY + 10}px">
      <div class="tip-label">{hoverNode.label}</div>
      <div class="tip-row">{hoverNode.kind.toUpperCase()} · {STATE_LABEL[hoverNode.state]}</div>
      <div class="tip-row dim">
        {hoverNode.childIds.length} children{hoverNode.spawnedLayerId !== null ? ' · spawned a layer' : ''}
      </div>
      <div class="tip-row dim">dbl-click: {hoverNode.expanded ? 'collapse' : 'expand'}</div>
    </div>
  {/if}

  <!-- selection panel -->
  {#if selected}
    <div class="c3-panel">
      <div class="p-head">
        <span class="p-label">{selected.label}</span>
        <button class="c3-btn" onclick={() => (selected = null)}>×</button>
      </div>
      <div class="p-row">KIND <b>{selected.kind.toUpperCase()}</b></div>
      <div class="p-row">STATE <b class="st-{selected.state}">{STATE_LABEL[selected.state]}</b></div>
      <div class="p-row">CHILDREN <b>{selected.childIds.length}</b></div>
      <div class="p-actions">
        <button class="c3-btn" onclick={() => toggleExpand(selected!.id)}>
          {selected.expanded ? 'COLLAPSE' : 'EXPAND'}
        </button>
        <button class="c3-btn" onclick={() => scene?.focusNode(selected!.id)}>FOCUS</button>
        {#if selected.spawnedLayerId !== null}
          <button class="c3-btn" onclick={() => descend(selected!)}>DESCEND ▾</button>
        {/if}
      </div>
    </div>
  {/if}
</div>

<style>
  .c3-host {
    position: relative;
    width: 100%;
    height: 100%;
    min-height: 320px;
    background: var(--bg-void);
    overflow: hidden;
    font-family: var(--font-sans);
  }
  canvas {
    display: block;
    width: 100%;
    height: 100%;
  }

  .c3-bar {
    position: absolute;
    top: 0;
    left: 0;
    right: 0;
    display: flex;
    align-items: center;
    gap: 12px;
    padding: 6px 12px;
    background: var(--bg-overlay);
    border-bottom: var(--border-width) solid var(--teal-line);
    font-size: 11px;
    letter-spacing: 0.12em;
    color: var(--text-primary);
    user-select: none;
  }
  .c3-title {
    color: var(--teal-bright);
    font-weight: 700;
  }
  .c3-stat { color: var(--text-secondary); }
  .c3-spacer { flex: 1; }

  .c3-btn {
    background: transparent;
    border: var(--border-width) solid var(--teal-line);
    color: var(--teal-bright);
    font: inherit;
    letter-spacing: inherit;
    padding: 3px 9px;
    cursor: pointer;
  }
  .c3-btn:hover { background: var(--teal-ghost); }
  .c3-btn.on { background: var(--teal-soft); color: var(--text-primary); }

  .c3-legend {
    position: absolute;
    left: 12px;
    bottom: 10px;
    display: flex;
    gap: 14px;
    font-size: 10px;
    letter-spacing: 0.1em;
    color: var(--text-secondary);
    background: var(--bg-overlay);
    border: var(--border-width) solid var(--border-default);
    padding: 5px 10px;
    user-select: none;
  }
  .c3-key { display: inline-flex; align-items: center; gap: 5px; }
  /* DESIGN.md §6: one teal family + mono inversion; red/amber are dead */
  .dot { width: 7px; height: 7px; display: inline-block; }
  .dot.done { background: var(--teal); }
  .dot.active { background: var(--teal-bright); box-shadow: 0 0 6px var(--teal-bright); }
  .dot.pending { background: var(--text-secondary); opacity: 0.55; }
  .dot.warn { background: transparent; border: 1px solid var(--text-primary); }
  .dot.fail { background: var(--text-primary); box-shadow: 0 0 6px var(--text-primary); }
  .dot.beam { background: var(--teal-bright); height: 2px; width: 14px; }

  .c3-tip {
    position: absolute;
    pointer-events: none;
    background: var(--bg-overlay);
    border: var(--border-width) solid var(--teal-line);
    padding: 6px 9px;
    font-size: 11px;
    color: var(--text-primary);
    max-width: 240px;
    z-index: 3;
  }
  .tip-label { color: var(--teal-bright); font-weight: 700; }
  .tip-row { margin-top: 2px; letter-spacing: 0.06em; }
  .tip-row.dim { color: var(--text-secondary); font-size: 10px; }

  .c3-panel {
    position: absolute;
    right: 12px;
    top: 42px;
    width: 220px;
    background: var(--bg-overlay);
    border: var(--border-width) solid var(--teal-line);
    padding: 8px 10px;
    font-size: 11px;
    color: var(--text-primary);
    letter-spacing: 0.08em;
  }
  .p-head {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: 6px;
  }
  .p-label { color: var(--teal-bright); font-weight: 700; }
  .p-row {
    display: flex;
    justify-content: space-between;
    color: var(--text-secondary);
    margin: 3px 0;
  }
  .p-row b { color: var(--text-primary); font-weight: 600; }
  .p-row .st-active { color: var(--teal-bright); }
  .p-row .st-done { color: var(--teal); }
  .p-row .st-warn { color: var(--text-primary); }
  .p-row .st-fail { background: var(--text-primary); color: var(--bg-void); padding: 0 4px; }
  .p-actions { display: flex; gap: 6px; margin-top: 8px; flex-wrap: wrap; }
</style>
