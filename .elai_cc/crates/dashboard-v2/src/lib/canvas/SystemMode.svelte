<script lang="ts">
  // SystemMode.svelte — J-31 system-composition mode (UI shell + mock fixtures).
  //
  // NODE = a binding to a real plan-unit / contract / MCP service (NOT a
  // drawing). EDGE = a wiring contract whose verdict (declared -> wired ->
  // verified -> broken) is a DERIVED state, never LLM-set. This renders the
  // intent-side system the user composes; CODE mode renders what exists.
  //
  // BACKEND: the orchestrator side (system_canvas.rs binding registry + verdict
  // derivation, SystemContractEvent SSB, the 'system' dispatcher namespace) is
  // landed; the populated branch reads live via get_system_canvas. The other
  // demoStates (empty/loading/error/disconnected) render synthetic surfaces for
  // state coverage without a backend.
  //
  // Layout law (DESIGN / J-31): the algorithm owns geometry — nodes sit on a
  // fixed col/row grid (a stand-in for dagre/ELK), NEVER LLM-placed coords.
  // Flat-matte work surface on --bg; quinary state language; role tokens only.

  import { onMount } from 'svelte';
  import NodeStateGlyph from './NodeStateGlyph.svelte';
  import { nodeStory } from './node_story_store.svelte';
  import { get_system_canvas as getSystemCanvas } from '../api/stub';
  import {
    SYSTEM_CANVAS_FIXTURE,
    BINDING_LABEL,
    VERDICT_CAPTION,
    verdictToQuinary,
    lifecycleToQuinary,
    type SystemCanvas,
    type SystemNode,
    type SystemEdge,
  } from './system_mode_fixtures';

  // ---------------------------------------------------------------------------
  // Layout constants (named — no magic numbers)
  // ---------------------------------------------------------------------------
  const COL_W = 220;   // px between grid columns
  const ROW_H = 132;   // px between grid rows
  const NODE_W = 176;  // node card width
  const NODE_H = 88;   // node card height
  const PAD = 40;      // surface padding around the grid

  // ---------------------------------------------------------------------------
  // Data source. The fixture stands in for a get_system_canvas read-command.
  // demoState lets the surface exercise its full state set (DESIGN §9) without
  // a backend: populated / empty / loading / error / disconnected.
  // ---------------------------------------------------------------------------
  type DemoState = 'populated' | 'empty' | 'loading' | 'error' | 'disconnected';
  let demoState = $state<DemoState>('populated');

  let liveCanvas = $state<SystemCanvas | null>(null);

  onMount(async () => {
    try {
      const c = await getSystemCanvas();
      if (c && (c.nodes.length > 0 || c.edges.length > 0)) {
        liveCanvas = c;
      }
    } catch {
      // offline / no backend — liveCanvas stays null, fixture is used.
    }
  });

  const canvasData = $derived(
    demoState === 'populated' && liveCanvas ? liveCanvas : SYSTEM_CANVAS_FIXTURE,
  );

  // selection — a node OR an edge inspector
  let selected = $state<
    | { kind: 'node'; node: SystemNode }
    | { kind: 'edge'; edge: SystemEdge }
    | null
  >(null);

  function nodeX(n: SystemNode): number { return PAD + n.col * COL_W; }
  function nodeY(n: SystemNode): number { return PAD + n.row * ROW_H; }
  function nodeCx(n: SystemNode): number { return nodeX(n) + NODE_W / 2; }
  function nodeCy(n: SystemNode): number { return nodeY(n) + NODE_H / 2; }

  function nodeById(id: string): SystemNode | undefined {
    return canvasData.nodes.find((n) => n.node_id === id);
  }

  const surfaceW = $derived(
    PAD * 2 + (Math.max(0, ...canvasData.nodes.map((n) => n.col)) + 1) * COL_W,
  );
  const surfaceH = $derived(
    PAD * 2 + (Math.max(0, ...canvasData.nodes.map((n) => n.row)) + 1) * ROW_H,
  );

  // Derived counts for the header summary (n/m wiring fraction — DESIGN §8).
  const verifiedEdges = $derived(canvasData.edges.filter((e) => e.verdict === 'verified').length);
  const brokenEdges = $derived(canvasData.edges.filter((e) => e.verdict === 'broken').length);

  function selectNode(n: SystemNode) { selected = { kind: 'node', node: n }; }
  function selectEdge(e: SystemEdge) { selected = { kind: 'edge', edge: e }; }
  function clearSelection() { selected = null; }
</script>

<div class="sys-mode">
  <!-- Header: mode label + wiring fraction + the demo-state switcher (so the
       full state set is reachable without a backend). -->
  <div class="sys-header">
    <span class="sys-title">System</span>
    <span class="sys-meta">
      {canvasData.nodes.length} units · {verifiedEdges}/{canvasData.edges.length} contracts verified
      {#if brokenEdges > 0}
        <span class="sys-broken">▲ {brokenEdges} broken</span>
      {/if}
    </span>
    <span class="sys-pending-note" title="J-31 orchestrator backend landed; populated branch reads live via get_system_canvas">
      backend: live on populated
    </span>
    <div class="sys-statepick" role="group" aria-label="Preview state">
      {#each (['populated', 'empty', 'loading', 'error', 'disconnected'] as DemoState[]) as st}
        <button
          class="state-btn"
          class:on={demoState === st}
          onclick={() => { demoState = st; clearSelection(); }}
        >{st}</button>
      {/each}
    </div>
  </div>

  <div class="sys-body">
    <!-- The composition surface -->
    <div class="sys-surface-wrap">
      {#if demoState === 'loading'}
        <div class="sys-skel" aria-label="loading system canvas">
          {#each Array(4) as _, i (i)}<div class="skel-card" style:--d="{i * 90}ms"></div>{/each}
        </div>
      {:else if demoState === 'error'}
        <!-- error obeys the state language: mono-inversion chip, not red -->
        <div class="sys-fault">
          <NodeStateGlyph state="fail" size={18} label="error" />
          <span class="fault-msg">could not derive system canvas</span>
          <button class="retry-btn" onclick={() => { demoState = 'populated'; }}>retry</button>
        </div>
      {:else if demoState === 'disconnected'}
        <div class="sys-fault">
          <NodeStateGlyph state="warn" size={18} label="disconnected" />
          <span class="fault-msg">orchestrator disconnected — last derivation shown stale</span>
          <button class="retry-btn" onclick={() => { demoState = 'populated'; }}>reconnect</button>
        </div>
      {:else if demoState === 'empty'}
        <!-- empty = focal next action, not a bare "no data" (DESIGN §9) -->
        <div class="sys-empty">
          <div class="empty-box">+ bind a unit</div>
          <p class="empty-hint">drop a plan unit, contract, or MCP service onto the surface to start composing the system</p>
        </div>
      {:else}
        <div class="sys-surface" style:width="{surfaceW}px" style:height="{surfaceH}px">
          <!-- edges first (under nodes); each is a contract with a derived verdict -->
          <svg class="edge-layer" width={surfaceW} height={surfaceH} aria-hidden="true">
            {#each canvasData.edges as e (e.producer_node + '->' + e.consumer_node)}
              {@const p = nodeById(e.producer_node)}
              {@const c = nodeById(e.consumer_node)}
              {#if p && c}
                <line
                  class="edge edge--{e.verdict}"
                  x1={nodeCx(p)} y1={nodeCy(p)}
                  x2={nodeCx(c)} y2={nodeCy(c)}
                />
              {/if}
            {/each}
          </svg>

          <!-- clickable edge hit-targets + verdict glyph at the midpoint -->
          {#each canvasData.edges as e (e.producer_node + '~' + e.consumer_node)}
            {@const p = nodeById(e.producer_node)}
            {@const c = nodeById(e.consumer_node)}
            {#if p && c}
              <button
                class="edge-handle edge-handle--{e.verdict}"
                style:left="{(nodeCx(p) + nodeCx(c)) / 2}px"
                style:top="{(nodeCy(p) + nodeCy(c)) / 2}px"
                class:sel={selected?.kind === 'edge' && selected.edge === e}
                title="{e.contract.produces} → {e.contract.consumes} ({e.verdict})"
                onclick={() => selectEdge(e)}
              >
                <NodeStateGlyph state={verdictToQuinary(e.verdict)} size={13} label={e.verdict} />
              </button>
            {/if}
          {/each}

          <!-- nodes — inset wells; bound vs DRAFT grey; quinary lifecycle glyph -->
          {#each canvasData.nodes as n (n.node_id)}
            <button
              class="node"
              class:draft={n.binding.kind === 'draft'}
              class:sel={selected?.kind === 'node' && selected.node === n}
              style:left="{nodeX(n)}px"
              style:top="{nodeY(n)}px"
              style:width="{NODE_W}px"
              style:height="{NODE_H}px"
              onclick={() => selectNode(n)}
            >
              <div class="node-top">
                <NodeStateGlyph state={lifecycleToQuinary(n.lifecycle)} size={13} label={n.lifecycle} />
                <span class="node-binding">{BINDING_LABEL[n.binding.kind]}</span>
                {#if n.tainted}<span class="node-taint" title="ingress carries G-1 provenance taint">taint</span>{/if}
              </div>
              <div class="node-name">{n.display}</div>
              <div class="node-ref">
                {#if n.binding.kind === 'draft'}
                  <span class="node-ref-draft">unbound — click to bind</span>
                {:else}
                  {n.binding.ref}
                {/if}
              </div>
              {#if n.schemaPin}
                <div class="node-pin node-pin--{n.schemaPin}">
                  {n.schemaPin === 'rug_pull' ? '✕ schema rug-pull' : n.schemaPin === 'repinned_ok' ? '✓ re-pinned' : 'pinned'}
                </div>
              {/if}
            </button>
          {/each}
        </div>
      {/if}
    </div>

    <!-- Inspector: node-story or edge-contract detail -->
    {#if selected}
      <aside class="sys-inspector">
        <button class="insp-close" aria-label="close inspector" onclick={clearSelection}>✕</button>
        {#if selected.kind === 'node'}
          <span class="insp-kind">{BINDING_LABEL[selected.node.binding.kind]}</span>
          <h3 class="insp-title">{selected.node.display}</h3>
          <div class="insp-row">
            <NodeStateGlyph state={lifecycleToQuinary(selected.node.lifecycle)} size={13} />
            <span class="insp-life">{selected.node.lifecycle}</span>
          </div>
          {#if selected.node.binding.kind !== 'draft'}
            <div class="insp-field"><span class="insp-label">BINDING</span><span class="insp-val">{selected.node.binding.ref}</span></div>
          {:else}
            <p class="insp-note">This node is a DRAFT. Bind it to a real plan unit, contract, or MCP service before it can build.</p>
          {/if}
          {#if selected.node.schemaPin}
            <div class="insp-field"><span class="insp-label">SCHEMA PIN</span><span class="insp-val">{selected.node.schemaPin}</span></div>
          {/if}
          <div class="insp-actions">
            <button class="insp-btn" disabled title="dispatches the bound unit through the normal FSM (backend pending)">build unit</button>
            <button class="insp-btn insp-btn--story" onclick={() => nodeStory.openFor(selected?.kind === 'node' ? selected.node.node_id : '')}>node story</button>
          </div>
        {:else}
          <span class="insp-kind">WIRING CONTRACT</span>
          <h3 class="insp-title">
            {nodeById(selected.edge.producer_node)?.display} → {nodeById(selected.edge.consumer_node)?.display}
          </h3>
          <div class="insp-row">
            <NodeStateGlyph state={verdictToQuinary(selected.edge.verdict)} size={13} />
            <span class="insp-verdict insp-verdict--{selected.edge.verdict}">{selected.edge.verdict}</span>
          </div>
          <!-- EXPLAIN layer: cached plain-language caption for the verdict -->
          <p class="insp-caption">{VERDICT_CAPTION[selected.edge.verdict]}</p>
          <div class="insp-field"><span class="insp-label">PRODUCES</span><span class="insp-val">{selected.edge.contract.produces}</span></div>
          <div class="insp-field"><span class="insp-label">CONSUMES</span><span class="insp-val">{selected.edge.contract.consumes}</span></div>
          <div class="insp-field"><span class="insp-label">REACHABLE</span><span class="insp-val">{selected.edge.contract.reachable_from}</span></div>
          <div class="insp-field"><span class="insp-label">EFFECT</span><span class="insp-val">{selected.edge.contract.observable_effect}</span></div>
          {#if selected.edge.contract.evidenceRow}
            <div class="insp-field"><span class="insp-label">EVIDENCE</span><span class="insp-val insp-evidence">{selected.edge.contract.evidenceRow}</span></div>
          {/if}
        {/if}
      </aside>
    {/if}
  </div>
</div>

<style>
  .sys-mode {
    display: flex;
    flex-direction: column;
    height: 100%;
    width: 100%;
    overflow: hidden;
    background: var(--bg);
    font-family: var(--font-mono);
    font-size: 10px;
  }

  .sys-header {
    display: flex;
    align-items: center;
    gap: 12px;
    padding: 6px 12px;
    border-bottom: 1px solid var(--line);
    flex-shrink: 0;
    flex-wrap: wrap;
  }
  .sys-title {
    font-weight: 500;
    font-size: 10px;
    text-transform: uppercase;
    letter-spacing: 0.16em;
    color: var(--text);
  }
  .sys-meta { color: var(--dim); font-size: 10px; }
  .sys-broken { color: var(--text); margin-left: 6px; }
  .sys-pending-note {
    color: var(--dim);
    font-size: 10px;
    font-style: italic;
  }
  .sys-statepick { margin-left: auto; display: flex; gap: 1px; }
  .state-btn {
    font-family: var(--font-mono);
    font-size: 10px;
    padding: 2px 6px;
    background: none;
    color: var(--ghost);
    border: none;
    cursor: pointer;
    transition: color var(--dur-fast) var(--ease-out);
  }
  .state-btn:hover { color: var(--text); }
  .state-btn.on { color: var(--d-teal-bright); }

  .sys-body { flex: 1 1 auto; min-height: 0; display: flex; overflow: hidden; }

  .sys-surface-wrap {
    flex: 1 1 auto;
    min-width: 0;
    overflow: auto;
    position: relative;
  }

  .sys-surface { position: relative; }

  .edge-layer { position: absolute; top: 0; left: 0; pointer-events: none; }
  .edge {
    stroke-width: 1.4;
    fill: none;
  }
  /* verdict reads via stroke style, not foreign hue */
  .edge--verified { stroke: var(--d-teal-deep); }
  .edge--wired    { stroke: var(--d-teal); }
  .edge--declared { stroke: var(--line2); stroke-dasharray: 3 4; }
  .edge--broken   { stroke: var(--text); stroke-dasharray: 2 3; }

  .edge-handle {
    position: absolute;
    transform: translate(-50%, -50%);
    background: var(--bg);
    border: 1px solid var(--line);
    border-radius: 50%;
    padding: 2px;
    cursor: pointer;
    line-height: 0;
    transition: border-color var(--dur-fast) var(--ease-out);
  }
  .edge-handle:hover, .edge-handle.sel { border-color: var(--d-teal-soft); }

  /* nodes — inset wells (--panel below --bg) rising to --raised on hover */
  .node {
    position: absolute;
    display: flex;
    flex-direction: column;
    gap: 3px;
    text-align: left;
    padding: 8px 10px;
    background: var(--panel);
    border: 1px solid var(--line);
    border-radius: var(--r-md);
    cursor: pointer;
    overflow: hidden;
    transition: background var(--dur-fast) var(--ease-out), border-color var(--dur-fast) var(--ease-out);
  }
  .node:hover { background: var(--raised); }
  .node.sel { border-color: var(--d-teal); }
  /* DRAFT renders grey (J-31 leg 2) */
  .node.draft {
    border-style: dashed;
    border-color: var(--line2);
    background: var(--sink);
    opacity: 0.85;
  }

  .node-top { display: flex; align-items: center; gap: 6px; }
  .node-binding {
    font-size: 8px;
    letter-spacing: 0.1em;
    color: var(--dim);
  }
  .node-taint {
    margin-left: auto;
    font-size: 8px;
    letter-spacing: 0.06em;
    color: var(--text);
    border: 1px solid var(--line2);
    border-radius: var(--r-xs);
    padding: 0 3px;
  }
  .node-name {
    font-size: 12px;
    color: var(--text);
    font-weight: 500;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }
  .node-ref {
    font-size: 10px;
    color: var(--dim);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }
  .node-ref-draft { color: var(--ghost); font-style: italic; }
  .node-pin {
    font-size: 8px;
    letter-spacing: 0.04em;
    color: var(--dim);
  }
  .node-pin--rug_pull { color: var(--text); }
  .node-pin--repinned_ok { color: var(--d-teal); }

  /* inspector */
  .sys-inspector {
    position: relative;
    flex: none;
    width: 280px;
    border-left: 1px solid var(--line);
    background: var(--bg);
    padding: 14px 14px 16px;
    overflow-y: auto;
  }
  .insp-close {
    position: absolute;
    top: 8px; right: 8px;
    background: none;
    border: none;
    color: var(--dim);
    font-size: 10px;
    cursor: pointer;
  }
  .insp-close:hover { color: var(--text); }
  .insp-kind {
    font-size: 10px;
    letter-spacing: 0.14em;
    color: var(--dim);
  }
  .insp-title {
    font-size: 13px;
    color: var(--text);
    font-weight: 500;
    margin: 3px 0 8px;
    line-height: 1.3;
  }
  .insp-row { display: flex; align-items: center; gap: 7px; margin-bottom: 8px; }
  .insp-life { font-size: 10px; color: var(--read); letter-spacing: 0.06em; }
  .insp-verdict { font-size: 10px; letter-spacing: 0.08em; color: var(--read); }
  .insp-verdict--verified { color: var(--d-teal-bright); }
  .insp-verdict--broken { color: var(--text); }
  .insp-caption {
    font-size: 10px;
    color: var(--read);
    line-height: var(--leading-code);
    margin: 0 0 10px;
    padding-left: 8px;
    border-left: 2px solid var(--d-teal-soft);
  }
  .insp-field {
    display: flex;
    gap: 8px;
    padding: 3px 0;
    border-bottom: 1px solid var(--line);
  }
  .insp-label {
    font-size: 10px;
    letter-spacing: 0.1em;
    color: var(--ghost);
    min-width: 72px;
    flex: none;
  }
  .insp-val { font-size: 10px; color: var(--read); overflow-wrap: anywhere; }
  .insp-evidence { color: var(--d-teal-bright); }
  .insp-note { font-size: 10px; color: var(--dim); line-height: var(--leading-code); }
  .insp-actions { margin-top: 12px; display: flex; gap: 6px; flex-wrap: wrap; }
  .insp-btn {
    font-family: var(--font-mono);
    font-size: 10px;
    padding: 4px 10px;
    background: var(--panel);
    border: 1px solid var(--line);
    border-radius: var(--r-sm);
    color: var(--dim);
    cursor: not-allowed;
  }
  .insp-btn--story {
    color: var(--read);
    cursor: pointer;
    transition: border-color var(--dur-fast) var(--ease-out), color var(--dur-fast) var(--ease-out);
  }
  .insp-btn--story:hover { border-color: var(--d-teal); color: var(--text); }

  /* state coverage */
  .sys-skel { display: flex; flex-wrap: wrap; gap: 16px; padding: 40px; }
  .skel-card {
    width: 176px; height: 88px;
    border-radius: var(--r-md);
    background: var(--panel);
    border: 1px solid var(--line);
    animation: skelpulse 1.4s var(--ease-std) infinite;
    animation-delay: var(--d);
  }
  .sys-fault, .sys-empty {
    height: 100%;
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    gap: 10px;
    padding: 24px;
    text-align: center;
  }
  .fault-msg { color: var(--read); font-size: 10px; }
  .retry-btn, .empty-box {
    font-family: var(--font-mono);
    font-size: 10px;
    padding: 6px 14px;
    background: none;
    border: 1px solid var(--line2);
    border-radius: var(--r-sm);
    color: var(--text);
    cursor: pointer;
    transition: border-color var(--dur-fast) var(--ease-out);
  }
  .retry-btn:hover, .empty-box:hover { border-color: var(--d-teal); }
  .empty-hint { color: var(--dim); font-size: 10px; max-width: 280px; line-height: var(--leading-text); margin: 0; }

  @keyframes skelpulse { 0%,100% { opacity: 1; } 50% { opacity: 0.5; } }
  @media (prefers-reduced-motion: reduce) {
    .skel-card { animation: none; }
  }
</style>
