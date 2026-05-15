<script lang="ts">
  // B-33 — sidebar panel for the GraphWindow.
  // Shows language badge, in/out edge counts, symbols list (capped) and
  // action buttons that push events back through the graph store.

  import { graph } from '$lib/state/graph.svelte';
  import { workspace } from '$lib/state/workspace.svelte';
  import type { VizFile, VizSymbol } from '$lib/api/types';

  interface Props {
    selectedNode: VizFile | null;
    symbols: VizSymbol[];
    /** Defaults to MAX_VIZ_SYMS_PER_FILE = 200 */
    symbolsCap?: number;
  }

  let { selectedNode, symbols, symbolsCap = 200 }: Props = $props();

  let displayed = $derived(symbols.slice(0, symbolsCap));
  let truncated = $derived(symbols.length > symbolsCap);

  function showBlastRadius() {
    if (!selectedNode) return;
    graph.request_impact(selectedNode.id);
    workspace.openTab('primary', 'graph');
  }

  function focusInGraph(id: string) {
    graph.focus_node(id);
  }
</script>

<aside class="graph-sidebar" aria-label="Selected node details">
  {#if !selectedNode}
    <p class="empty">Select a node to inspect.</p>
  {:else}
    <header class="header">
      <span class="lang-badge" data-language={selectedNode.language}>
        {selectedNode.language}
      </span>
      <h3 class="label" title={selectedNode.id}>{selectedNode.label}</h3>
    </header>
    <dl class="meta">
      <div><dt>Deps out</dt><dd>{selectedNode.deps}</dd></div>
      <div><dt>Dependents in</dt><dd>{selectedNode.dependents}</dd></div>
      <div><dt>Symbols</dt><dd>{selectedNode.symbol_count}</dd></div>
    </dl>
    <div class="actions">
      <button type="button" onclick={showBlastRadius}>Show blast radius</button>
      <button type="button" onclick={() => focusInGraph(selectedNode.id)}>Centre on node</button>
    </div>
    <section class="symbols">
      <h4>Symbols ({displayed.length}{truncated ? ` of ${symbols.length}` : ''})</h4>
      {#if displayed.length === 0}
        <p class="empty">No indexed symbols yet for this file.</p>
      {:else}
        <ul>
          {#each displayed as s (s.id)}
            <li>
              <button class="sym" type="button" onclick={() => focusInGraph(s.id)}>
                <span class="sym-kind">{s.kind}</span>
                <span class="sym-name">{s.name}</span>
                <span class="sym-line">L{s.line}</span>
              </button>
            </li>
          {/each}
        </ul>
        {#if truncated}
          <p class="truncated">truncated: showing {displayed.length} of {symbols.length} (cap {symbolsCap})</p>
        {/if}
      {/if}
    </section>
  {/if}
</aside>

<style>
  .graph-sidebar {
    width: 320px;
    display: flex;
    flex-direction: column;
    gap: 12px;
    padding: 12px;
    background: var(--bg-surface);
    border-left: 1px solid var(--border-subtle);
    overflow-y: auto;
    font-family: var(--font-mono);
    font-size: 11px;
    color: var(--text-primary);
  }
  .empty {
    color: var(--text-secondary);
    margin: 0;
  }
  .header {
    display: flex;
    flex-direction: column;
    gap: 6px;
  }
  .lang-badge {
    align-self: flex-start;
    padding: 2px 6px;
    border-radius: var(--r-sm);
    background: var(--teal-ghost);
    color: var(--teal);
    text-transform: uppercase;
    font-size: 10px;
    letter-spacing: 0.05em;
  }
  .label {
    margin: 0;
    font-size: 13px;
    word-break: break-all;
  }
  .meta {
    display: grid;
    grid-template-columns: 1fr 1fr 1fr;
    gap: 8px;
    margin: 0;
  }
  .meta div {
    display: flex;
    flex-direction: column;
  }
  .meta dt {
    color: var(--text-secondary);
    font-size: 10px;
    text-transform: uppercase;
  }
  .meta dd {
    margin: 0;
    font-size: 13px;
  }
  .actions {
    display: flex;
    flex-direction: column;
    gap: 6px;
  }
  .actions button {
    padding: 6px 10px;
    background: transparent;
    border: 1px solid var(--teal-line);
    border-radius: var(--r-sm);
    color: var(--teal);
    cursor: pointer;
    font-family: inherit;
    font-size: 11px;
  }
  .actions button:hover {
    background: var(--teal-ghost);
    color: var(--teal-bright);
  }
  .symbols h4 {
    margin: 0 0 6px 0;
    text-transform: uppercase;
    font-size: 10px;
    letter-spacing: 0.05em;
    color: var(--text-secondary);
  }
  .symbols ul {
    list-style: none;
    margin: 0;
    padding: 0;
  }
  .sym {
    display: flex;
    width: 100%;
    gap: 8px;
    padding: 4px 6px;
    background: transparent;
    border: 1px solid transparent;
    border-radius: var(--r-xs);
    color: var(--text-primary);
    cursor: pointer;
    text-align: left;
    font-family: inherit;
    font-size: 11px;
  }
  .sym:hover {
    border-color: var(--border-default);
    background: var(--bg-hover);
  }
  .sym-kind {
    color: var(--text-secondary);
    width: 60px;
  }
  .sym-name {
    flex: 1 1 auto;
  }
  .sym-line {
    color: var(--text-secondary);
  }
  .truncated {
    margin: 6px 0 0 0;
    color: var(--text-secondary);
    font-size: 10px;
  }
</style>
