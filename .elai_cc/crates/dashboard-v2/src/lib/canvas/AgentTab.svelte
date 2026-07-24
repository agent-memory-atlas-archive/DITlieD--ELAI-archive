<script lang="ts">
  // AgentTab.svelte — J-60 slice-2 scoped journal for a selected process node.
  // Shows the worktree diff entries scoped to the selected node's binding key,
  // plus the node's status. Minimal: a header + a filtered diff list.

  import type { ProcessNode, WorktreeDiffEntry } from '../api/stub';

  let { node, diffEntries }: { node: ProcessNode; diffEntries: WorktreeDiffEntry[] } = $props();

  let scoped = $derived(diffEntries);
</script>

<div class="agent-tab">
  <div class="agent-tab-header">
    <span class="agent-tab-title">{node.display.label}</span>
    <span class="agent-tab-status" data-status={node.display.status}>
      {node.display.status}
    </span>
  </div>
  <div class="agent-tab-binding">{node.node_id}</div>
  {#if scoped.length === 0}
    <div class="agent-tab-empty">No edit history for this node.</div>
  {:else}
    <ul class="agent-tab-diffs">
      {#each scoped as entry (entry.seq + entry.path)}
        <li>
          <span class="agent-tab-seq">#{entry.seq}</span>
          <span class="agent-tab-path">{entry.path}</span>
        </li>
      {/each}
    </ul>
  {/if}
</div>

<style>
  .agent-tab {
    display: flex;
    flex-direction: column;
    gap: 0.4rem;
    padding: 0.6rem;
    border: 1px solid rgba(127, 127, 127, 0.2);
    border-radius: 4px;
  }
  .agent-tab-header {
    display: flex;
    align-items: baseline;
    justify-content: space-between;
    gap: 0.5rem;
  }
  .agent-tab-title { font-weight: 600; }
  .agent-tab-status {
    font-size: 0.75rem;
    text-transform: uppercase;
    letter-spacing: 0.05em;
  }
  .agent-tab-status[data-status='completed'] { color: #27ae60; }
  .agent-tab-status[data-status='active'] { color: #2980b2; }
  .agent-tab-status[data-status='idle'] { color: #95a5a6; }
  .agent-tab-binding { opacity: 0.5; font-size: 0.8rem; }
  .agent-tab-empty { opacity: 0.6; padding: 0.5rem 0; }
  .agent-tab-diffs {
    list-style: none;
    padding: 0;
    margin: 0;
    display: flex;
    flex-direction: column;
    gap: 0.2rem;
    max-height: 240px;
    overflow: auto;
  }
  .agent-tab-diffs li {
    display: flex;
    gap: 0.5rem;
    font-size: 0.82rem;
  }
  .agent-tab-seq { opacity: 0.5; min-width: 3rem; }
  .agent-tab-path { font-family: monospace; }
</style>
