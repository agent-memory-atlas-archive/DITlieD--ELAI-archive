<script lang="ts">
  // PlanMode.svelte — PLAN clipping-frame mode on the canvas. (H-12)
  //
  // Renders the decomposition tree from plan.svelte.ts as a flat list of
  // nested nodes in a clipping frame: concept > phase > step > baby-step,
  // with `contains` nesting and `depends_on` edges between siblings.
  //
  // Active node: lit via node.status === 'active' (the real plan store uses
  // status-on-node, NOT separate active_concept_id/active_phase_id/etc. props
  // — those exist only in the OrbitFixture mock. Per H-12-grounding.md:
  // "The active-node identification uses status='active' on the node itself").
  //
  // Data feed: plan.svelte.ts (GD-1: REUSE the real store, do not re-implement).
  // The plan store is already bootstrapped by ActivityWindow; PlanMode is a
  // passive consumer — it reads plan.tree reactively without owning the poll.
  //
  // This is a 2D clipping-frame renderer, NOT a 3D orbit widget. No StarCanvas,
  // no sigma — just a structured HTML canvas-mode list with visual hierarchy.
  // Sigma rendering of the plan graph is Tier-2 (full layout engine needed).

  import { plan } from '../state/plan.svelte';
  import type { DecompNode } from '../api/types';

  // ---------------------------------------------------------------------------
  // Named constants (no magic numbers)
  // ---------------------------------------------------------------------------

  /** Status values on DecompNode (per api/types.ts). */
  const STATUS_ACTIVE = 'active';
  const STATUS_DONE = 'done';
  const STATUS_FAILED = 'failed';
  const STATUS_PENDING = 'pending';

  /** Maximum nesting depth to render (beyond this, content is clipped). */
  const MAX_DEPTH = 5;

  // ---------------------------------------------------------------------------
  // Helpers
  // ---------------------------------------------------------------------------

  /** Recursively count all nodes in a DecompNode tree. */
  function countNodes(node: DecompNode): number {
    let n = 1;
    for (const child of node.children ?? []) {
      n += countNodes(child);
    }
    return n;
  }

  /** Find the first node with status === 'active' in the tree (BFS). */
  function findActiveNode(root: DecompNode): DecompNode | null {
    if (root.status === STATUS_ACTIVE) return root;
    for (const child of root.children ?? []) {
      const found = findActiveNode(child);
      if (found !== null) return found;
    }
    return null;
  }

  // ---------------------------------------------------------------------------
  // Reactive state from the plan store
  // ---------------------------------------------------------------------------

  const tree = $derived(plan.tree);
  const nodeCount = $derived(countNodes(tree));
  const activeNode = $derived(findActiveNode(tree));

  /** True when the plan is non-empty (has real data beyond the empty placeholder). */
  const hasData = $derived(
    tree.id !== 'empty' && (tree.children?.length ?? 0) > 0
  );
</script>

<div class="plan-mode">
  <!-- Plan frame header -->
  <div class="plan-header">
    <span class="plan-title">Plan</span>
    <span class="plan-meta">
      {#if hasData}
        {nodeCount} nodes
        {#if activeNode}
          — active: <span class="active-label">{activeNode.label}</span>
        {/if}
      {:else}
        no active plan
      {/if}
    </span>
  </div>

  <!-- Plan tree — nested node list -->
  <div class="plan-body">
    {#if !hasData}
      <div class="plan-empty">
        <span>No active plan. Start a task to see the decomposition tree here.</span>
      </div>
    {:else}
      {@render planNodeTree(tree, 0)}
    {/if}
  </div>
</div>

<!-- Recursive snippet for rendering each DecompNode (lowercase name avoids
     component-name collision in svelte-check TypeScript inference). -->
{#snippet planNodeTree(node: DecompNode, depth: number)}
  {#if depth < MAX_DEPTH}
    <div
      class="plan-node"
      class:active={node.status === STATUS_ACTIVE}
      class:done={node.status === STATUS_DONE}
      class:failed={node.status === STATUS_FAILED}
      class:pending={node.status === STATUS_PENDING}
      style:padding-left="{depth * 16 + 8}px"
      aria-current={node.status === STATUS_ACTIVE ? 'step' : undefined}
    >
      <!-- Level badge -->
      <span class="node-level">{node.level}</span>

      <!-- Node label -->
      <span class="node-label">{node.label}</span>

      <!-- Status indicator -->
      <span class="node-status" aria-label="status: {node.status}">
        {#if node.status === STATUS_ACTIVE}
          <span class="status-dot status-active" title="active"></span>
        {:else if node.status === STATUS_DONE}
          <span class="status-dot status-done" title="done"></span>
        {:else if node.status === STATUS_FAILED}
          <span class="status-dot status-failed" title="failed"></span>
        {:else}
          <span class="status-dot status-pending" title="pending"></span>
        {/if}
      </span>

      <!-- Elapsed (if non-zero) -->
      {#if (node.elapsed_ms ?? 0) > 0}
        <span class="node-elapsed">{Math.round((node.elapsed_ms ?? 0) / 1000)}s</span>
      {/if}
    </div>

    <!-- Recurse into children -->
    {#each node.children ?? [] as child (child.id)}
      {@render planNodeTree(child, depth + 1)}
    {/each}
  {/if}
{/snippet}

<style>
  .plan-mode {
    display: flex;
    flex-direction: column;
    height: 100%;
    width: 100%;
    overflow: hidden;
    font-family: var(--font-mono);
    font-size: 11px;
    background: var(--bg-base);
  }

  .plan-header {
    display: flex;
    align-items: center;
    gap: 10px;
    padding: 6px 12px;
    border-bottom: 1px solid var(--border-subtle);
    background: var(--bg-surface);
    flex-shrink: 0;
  }

  .plan-title {
    font-weight: 600;
    font-size: 11px;
    text-transform: uppercase;
    letter-spacing: 0.08em;
    color: var(--text-primary);
  }

  .plan-meta {
    color: var(--text-secondary);
    font-size: 10px;
  }

  .active-label {
    color: var(--teal-bright, #14b8a6);
    font-weight: 600;
  }

  .plan-body {
    flex: 1 1 auto;
    overflow-y: auto;
    overflow-x: hidden;
    padding: 6px 0;
  }

  .plan-empty {
    padding: 24px 16px;
    color: var(--text-ghost);
    font-size: 10px;
    text-align: center;
  }

  /* Node rows */
  .plan-node {
    display: flex;
    align-items: center;
    gap: 6px;
    padding: 3px 8px;
    min-height: 22px;
    border-left: 2px solid transparent;
    transition: background 80ms, border-color 80ms;
  }

  .plan-node:hover {
    background: var(--bg-hover);
  }

  /* Active node highlight (status === 'active') */
  .plan-node.active {
    background: var(--teal-ghost, rgba(20, 184, 166, 0.1));
    border-left-color: var(--teal-bright, #14b8a6);
  }

  .plan-node.done {
    opacity: 0.55;
  }

  .plan-node.failed {
    border-left-color: var(--red, #e05252);
    background: rgba(224, 82, 82, 0.05);
  }

  .plan-node.pending {
    opacity: 0.75;
  }

  /* Level badge */
  .node-level {
    font-size: 8px;
    text-transform: uppercase;
    letter-spacing: 0.1em;
    color: var(--text-ghost);
    min-width: 46px;
    flex-shrink: 0;
  }

  /* Node label */
  .node-label {
    flex: 1 1 auto;
    color: var(--text-primary);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .plan-node.active .node-label {
    color: var(--teal-bright, #14b8a6);
    font-weight: 600;
  }

  /* Status dot */
  .node-status {
    flex-shrink: 0;
  }

  .status-dot {
    display: inline-block;
    width: 6px;
    height: 6px;
    border-radius: 50%;
  }

  .status-active  { background: var(--teal-bright, #14b8a6); box-shadow: 0 0 4px var(--teal-bright, #14b8a6); }
  .status-done    { background: var(--text-ghost); }
  .status-failed  { background: var(--red, #e05252); }
  .status-pending { background: var(--border-subtle); }

  /* Elapsed */
  .node-elapsed {
    font-size: 9px;
    color: var(--text-ghost);
    flex-shrink: 0;
  }
</style>
