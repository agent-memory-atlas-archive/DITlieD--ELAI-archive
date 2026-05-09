<script lang="ts">
  import Tree from './Tree.svelte';
  import type { TreeNode } from './tree-types';
  import { ChevronRight } from '@lucide/svelte';

  interface Props {
    nodes: TreeNode[];
    expandedIds?: Set<string>;
    selectedId?: string;
    onselect?: (id: string) => void;
    /** Called when user Ctrl+clicks a node — navigates to it in Activity. */
    onctrselect?: (id: string) => void;
    depth?: number;
  }

  let {
    nodes,
    expandedIds = $bindable(new Set<string>()),
    selectedId = $bindable(''),
    onselect,
    onctrselect,
    depth = 0,
  }: Props = $props();

  function toggleExpand(id: string) {
    const next = new Set(expandedIds);
    if (next.has(id)) {
      next.delete(id);
    } else {
      next.add(id);
    }
    expandedIds = next;
  }

  function handleClick(id: string, e: MouseEvent) {
    if ((e.ctrlKey || e.metaKey) && onctrselect) {
      e.preventDefault();
      selectedId = id;
      onselect?.(id);
      onctrselect(id);
    } else {
      handleSelect(id);
    }
  }

  function handleSelect(id: string) {
    selectedId = id;
    onselect?.(id);
  }

  function statusColor(status?: string): string {
    switch (status) {
      case 'done': return 'var(--teal)';
      case 'active': return 'var(--teal)';
      case 'failed': return 'var(--fail)';
      default: return 'var(--text-ghost)';
    }
  }

  function labelColor(status?: string): string {
    switch (status) {
      case 'active': return 'var(--text-primary)';
      case 'done': return 'var(--teal)';
      case 'failed': return 'var(--fail)';
      default: return 'var(--text-secondary)';
    }
  }
</script>

<div class="tree" style="padding-left: {depth * 16}px;">
  {#each nodes as node (node.id)}
    {@const hasChildren = node.children && node.children.length > 0}
    {@const expanded = expandedIds.has(node.id)}
    {@const isSelected = selectedId === node.id}
    {@const isActive = node.status === 'active'}

    <div
      class="tree-node"
      class:tree-node--selected={isSelected}
      class:tree-node--active={isActive}
      onclick={(e) => handleClick(node.id, e)}
      role="treeitem"
      aria-selected={isSelected}
      aria-expanded={hasChildren ? expanded : undefined}
      tabindex="0"
      onkeydown={(e) => { if (e.key === 'Enter' || e.key === ' ') { e.preventDefault(); handleSelect(node.id); } }}
    >
      {#if hasChildren}
        <button
          class="tree-caret tree-caret--visible"
          class:tree-caret--expanded={expanded}
          tabindex="-1"
          aria-label={expanded ? 'Collapse' : 'Expand'}
          onclick={(e) => { e.stopPropagation(); toggleExpand(node.id); }}
          type="button"
        >
          <ChevronRight size={12} strokeWidth={1.5} />
        </button>
      {:else}
        <span class="tree-caret"></span>
      {/if}

      <span
        class="tree-dot"
        class:tree-dot--breath={isActive}
        style="background: {statusColor(node.status)};"
      ></span>

      <span class="tree-label" style="color: {labelColor(node.status)};">
        {node.label}
      </span>

      {#if node.meta}
        <span class="tree-meta">{node.meta}</span>
      {/if}
    </div>

    {#if hasChildren && expanded}
      <Tree
        nodes={node.children ?? []}
        bind:expandedIds
        bind:selectedId
        {onselect}
        {onctrselect}
        depth={depth + 1}
      />
    {/if}
  {/each}
</div>

<style>
  .tree {
    display: flex;
    flex-direction: column;
  }

  .tree-node {
    display: flex;
    align-items: center;
    gap: 4px;
    height: 24px;
    padding: 0 8px;
    border-radius: var(--r-xs);
    cursor: pointer;
    position: relative;
    outline: none;
    transition: background-color var(--dur-fast) var(--ease-std);
  }

  .tree-node:hover {
    background: var(--bg-hover);
  }

  .tree-node:focus-visible {
    box-shadow: inset 0 0 0 1px var(--teal);
  }

  .tree-node--selected {
    background: var(--bg-hover);
  }

  /* 2-px teal left border for active node, scales in from top */
  .tree-node--active::before {
    content: '';
    position: absolute;
    left: 0;
    top: 0;
    bottom: 0;
    width: 2px;
    background: var(--teal);
    border-radius: 1px;
    animation: scaleBarIn 140ms var(--ease-out) both;
    transform-origin: top;
  }

  @keyframes scaleBarIn {
    from { transform: scaleY(0); }
    to   { transform: scaleY(1); }
  }

  .tree-caret {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 14px;
    height: 14px;
    flex-shrink: 0;
    color: var(--text-ghost);
    background: transparent;
    border: none;
    padding: 0;
    cursor: pointer;
    transition: transform 120ms var(--ease-std);
    outline: none;
  }

  .tree-caret:not(.tree-caret--visible) {
    pointer-events: none;
  }

  .tree-caret--expanded {
    transform: rotate(90deg);
  }

  .tree-dot {
    width: 6px;
    height: 6px;
    border-radius: 50%;
    flex-shrink: 0;
  }

  .tree-dot--breath {
    animation: breath 2.2s ease-in-out infinite;
  }

  @keyframes breath {
    0%   { opacity: 0.2; }
    50%  { opacity: 0.55; }
    100% { opacity: 0.2; }
  }

  .tree-label {
    font-family: var(--font-sans);
    font-size: 12px;
    flex: 1;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .tree-meta {
    font-family: var(--font-mono);
    font-size: 11px;
    color: var(--text-ghost);
    flex-shrink: 0;
    margin-left: auto;
  }
</style>
