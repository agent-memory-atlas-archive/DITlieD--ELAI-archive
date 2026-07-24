<script lang="ts">
  // ProcessLayer.svelte — J-60 slice-2 process canvas mode.
  // Renders the live process-canvas registry (nodes + worktree diff entries)
  // from get_process_canvas. Minimal: lists nodes with status glyphs and a
  // diff panel. AgentTab is mounted when a node is selected.

  import { onMount } from 'svelte';
  import { get_process_canvas, type ProcessCanvasRegistry, type ProcessNode } from '../api/stub';
  import AgentTab from './AgentTab.svelte';

  let registry = $state<ProcessCanvasRegistry>({ nodes: [], diff_entries: [], fingerprint: '' });
  let loading = $state(true);
  let error = $state<string | null>(null);
  let selected = $state<ProcessNode | null>(null);

  onMount(async () => {
    try {
      registry = await get_process_canvas();
    } catch (e) {
      error = e instanceof Error ? e.message : String(e);
    } finally {
      loading = false;
    }
  });

  function statusGlyph(status: string): string {
    if (status === 'completed') return '✓';
    if (status === 'active') return '●';
    return '○';
  }

  function bindingLabel(node: ProcessNode): string {
    const b = node.binding;
    if ('fsm_session' in b) return `fsm:${b.fsm_session}`;
    if ('agent_worker' in b) return `agent:${b.agent_worker}`;
    return `phase:${b.phase_step}`;
  }

  function selectNode(node: ProcessNode) {
    selected = selected?.node_id === node.node_id ? null : node;
  }
</script>

<div class="proc-layer">
  <div class="proc-header">
    <span class="proc-title">Process</span>
    <span class="proc-meta">
      {registry.nodes.length} nodes · {registry.diff_entries.length} edits
    </span>
  </div>

  {#if loading}
    <div class="proc-state">Loading process canvas…</div>
  {:else if error}
    <div class="proc-state proc-error">Error: {error}</div>
  {:else if registry.nodes.length === 0}
    <div class="proc-state">No process nodes yet.</div>
  {:else}
    <div class="proc-body">
      <ul class="proc-node-list">
        {#each registry.nodes as node (node.node_id)}
          <li>
            <button
              class="proc-node"
              class:selected={selected?.node_id === node.node_id}
              onclick={() => selectNode(node)}
            >
              <span class="proc-glyph" data-status={node.display.status}>
                {statusGlyph(node.display.status)}
              </span>
              <span class="proc-node-label">{node.display.label}</span>
              <span class="proc-node-binding">{bindingLabel(node)}</span>
            </button>
          </li>
        {/each}
      </ul>
      {#if selected}
        <AgentTab node={selected} diffEntries={registry.diff_entries} />
      {/if}
    </div>
  {/if}
</div>

<style>
  .proc-layer {
    display: flex;
    flex-direction: column;
    height: 100%;
    padding: 0.75rem;
    gap: 0.5rem;
    overflow: auto;
  }
  .proc-header {
    display: flex;
    align-items: baseline;
    gap: 0.5rem;
  }
  .proc-title { font-weight: 600; }
  .proc-meta { opacity: 0.7; font-size: 0.85rem; }
  .proc-state { opacity: 0.6; padding: 1rem; }
  .proc-error { color: #c0392b; }
  .proc-body { display: flex; flex-direction: column; gap: 0.5rem; }
  .proc-node-list {
    list-style: none;
    padding: 0;
    margin: 0;
    display: flex;
    flex-direction: column;
    gap: 0.25rem;
  }
  .proc-node {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    width: 100%;
    padding: 0.4rem 0.5rem;
    background: transparent;
    border: 1px solid rgba(127, 127, 127, 0.2);
    border-radius: 4px;
    cursor: pointer;
    text-align: left;
    color: inherit;
    font: inherit;
  }
  .proc-node.selected { border-color: rgba(127, 127, 127, 0.6); }
  .proc-node:hover { background: rgba(127, 127, 127, 0.08); }
  .proc-glyph { font-size: 0.9rem; }
  .proc-glyph[data-status='completed'] { color: #27ae60; }
  .proc-glyph[data-status='active'] { color: #2980b9; }
  .proc-glyph[data-status='idle'] { color: #95a5a6; }
  .proc-node-label { flex: 1; }
  .proc-node-binding { opacity: 0.5; font-size: 0.8rem; }
</style>
