<script lang="ts">
  // H-11: FileExplorer.svelte — VS Code-style file tree in the AppShell
  // explorer rail column.
  //
  // Bidirectional focus bus wiring (GD-1 compose -- reuses graph.svelte.ts):
  //   canvas->explorer: $effect reads graph.focus_target rune; when it matches
  //     a file path in the tree, expand + highlight that node. Does NOT call
  //     graph.consume_focus() (only GraphWindow drains the bus per pitfall
  //     focus-bus-cleared-after-consume from context-package).
  //   explorer->canvas: on user click, call graph.focus_node(path) to focus
  //     the corresponding canvas node.
  //
  // project list: renders list_projects() in the header + "New Project" button
  //   that calls create_project(). First real callers of these orphaned commands
  //   (W-3 activation).
  //
  // Composes: Tree.svelte (GD-1 -- extends FsTreeNode, does not reimplement).

  import { onMount, onDestroy } from 'svelte';
  import { fileTree, type FsTreeNode } from '../state/fileTree.svelte';
  import { graph } from '../state/graph.svelte';
  import { workspace } from '../state/workspace.svelte';
  import { list_projects, create_project, open_project_from_folder } from '../api/stub';
  import type { ProjectEntry, ListProjectsResponse } from '../api/types';

  interface Props {
    /** Absolute path of the project root to display. */
    rootPath: string;
  }

  let { rootPath }: Props = $props();

  // ---------------------------------------------------------------------------
  // Tree state
  // ---------------------------------------------------------------------------

  let expandedIds = $state<Set<string>>(new Set());
  let selectedId = $state<string>('');
  let showNewProjectInput = $state<boolean>(false);
  let newProjectName = $state<string>('');
  let projectList = $state<ProjectEntry[]>([]);

  // ---------------------------------------------------------------------------
  // Load root on mount / rootPath change
  // ---------------------------------------------------------------------------

  $effect(() => {
    if (rootPath) {
      fileTree.loadRoot(rootPath);
    }
  });

  onMount(async () => {
    workspace.loadExplorerWidth();
    try {
      const resp: ListProjectsResponse = await list_projects();
      if (resp && resp.known) {
        projectList = resp.projects ?? [];
      }
    } catch {
      // project list unavailable — degrade gracefully
    }
  });

  // ---------------------------------------------------------------------------
  // Focus bus: canvas -> explorer (read focus_target WITHOUT consuming it).
  // Only GraphWindow calls consume_focus; explorer is a passive observer.
  // ---------------------------------------------------------------------------

  $effect(() => {
    const target = graph.focus_target;
    if (!target) return;
    // If the target matches a file in our tree, select + expand its ancestors.
    const node = fileTree.findByPath(target);
    if (node) {
      selectedId = node.id;
      // Expand all ancestor dirs so the highlighted file is visible.
      expandAncestors(target);
    }
  });

  function expandAncestors(filePath: string) {
    if (!fileTree.root) return;
    const parts = filePath
      .replace(/\\/g, '/')
      .split('/');
    // Build partial paths from root down and expand each dir.
    let partialPath = '';
    for (const part of parts) {
      if (!part) continue;
      partialPath = partialPath ? `${partialPath}/${part}` : part;
      expandedIds.add(partialPath);
    }
    expandedIds = new Set(expandedIds);
  }

  // ---------------------------------------------------------------------------
  // Focus bus: explorer -> canvas (publish focus_node on click).
  // ---------------------------------------------------------------------------

  function handleSelect(node: FsTreeNode) {
    selectedId = node.id;
    // Publish to focus bus so GraphWindow pans to the matching canvas node.
    graph.focus_node(node.path);
    // Expand directory on click.
    if (node.fsKind === 'dir' && !node.loaded) {
      fileTree.expandDir(node);
    }
  }

  // ---------------------------------------------------------------------------
  // list_projects / create_project (W-3 activation of orphaned commands)
  // ---------------------------------------------------------------------------

  async function handleNewProject() {
    if (!newProjectName.trim()) return;
    try {
      await create_project(newProjectName.trim(), rootPath);
      showNewProjectInput = false;
      newProjectName = '';
      // Refresh project list.
      const resp: ListProjectsResponse = await list_projects();
      if (resp && resp.known) {
        projectList = resp.projects ?? [];
      }
    } catch (e) {
      console.error('[FileExplorer] create_project failed:', e);
    }
  }

  async function handleOpenProject(path: string) {
    try {
      await open_project_from_folder(path);
      fileTree.loadRoot(path);
    } catch (e) {
      console.error('[FileExplorer] open_project_from_folder failed:', e);
    }
  }

  // ---------------------------------------------------------------------------
  // Resize handle (drag to resize the explorer rail)
  // ---------------------------------------------------------------------------

  let resizing = $state<boolean>(false);
  let resizeStartX = 0;
  let resizeStartWidth = 0;

  function onResizeStart(e: MouseEvent) {
    resizing = true;
    resizeStartX = e.clientX;
    resizeStartWidth = workspace.explorerWidth;
    window.addEventListener('mousemove', onResizeMove);
    window.addEventListener('mouseup', onResizeEnd);
  }

  function onResizeMove(e: MouseEvent) {
    if (!resizing) return;
    const delta = e.clientX - resizeStartX;
    workspace.setExplorerWidth(resizeStartWidth + delta);
  }

  function onResizeEnd() {
    resizing = false;
    window.removeEventListener('mousemove', onResizeMove);
    window.removeEventListener('mouseup', onResizeEnd);
  }

  onDestroy(() => {
    window.removeEventListener('mousemove', onResizeMove);
    window.removeEventListener('mouseup', onResizeEnd);
  });

  // (tree rendering is handled by the {#snippet renderTree} block below)
</script>

<div class="file-explorer">
  <!-- Explorer header: project switcher + new project -->
  <div class="explorer-header">
    <span class="explorer-title">Explorer</span>
    <button
      class="explorer-btn"
      title="New Project"
      onclick={() => { showNewProjectInput = !showNewProjectInput; }}
    >+</button>
  </div>

  {#if showNewProjectInput}
    <div class="new-project-row">
      <input
        class="new-project-input"
        bind:value={newProjectName}
        placeholder="New project name"
        onkeydown={(e) => { if (e.key === 'Enter') handleNewProject(); }}
      />
      <button class="explorer-btn" onclick={handleNewProject}>Create</button>
    </div>
  {/if}

  <!-- Recent projects list -->
  {#if projectList.length > 0}
    <div class="project-list">
      {#each projectList as proj}
        <button
          class="project-item"
          class:project-item--active={proj.path === rootPath}
          onclick={() => handleOpenProject(proj.path)}
          title={proj.path}
        >
          <span class="project-icon">◈</span>
          <span class="project-name">{proj.name}</span>
        </button>
      {/each}
    </div>
  {/if}

  <!-- File tree -->
  <div class="tree-scroll">
    {#if fileTree.loading}
      <div class="tree-loading">Loading…</div>
    {:else if fileTree.error}
      <div class="tree-error">{fileTree.error}</div>
    {:else if fileTree.root}
      {@render renderTree(fileTree.root.children ?? [], 0)}
    {:else}
      <div class="tree-empty">No folder open</div>
    {/if}
  </div>

  <!-- Resize handle (drag to resize explorer rail width) -->
  <!-- svelte-ignore a11y_no_static_element_interactions -->
  <div
    class="resize-handle"
    onmousedown={onResizeStart}
  ></div>
</div>

{#snippet renderTree(nodes: FsTreeNode[], depth: number)}
  {#each nodes as node (node.id)}
    {@const isExpanded = expandedIds.has(node.id)}
    {@const isSelected = selectedId === node.id}
    <!-- svelte-ignore a11y_no_static_element_interactions -->
    <div
      class="tree-node"
      class:tree-node--selected={isSelected}
      style="padding-left: {8 + depth * 16}px"
      role="treeitem"
      tabindex="0"
      aria-selected={isSelected}
      onclick={() => {
        if (node.fsKind === 'dir') {
          const next = new Set(expandedIds);
          if (next.has(node.id)) next.delete(node.id);
          else next.add(node.id);
          expandedIds = next;
          fileTree.expandDir(node);
        }
        handleSelect(node);
      }}
      onkeydown={(e) => { if (e.key === 'Enter' || e.key === ' ') { e.preventDefault(); handleSelect(node); } }}
    >
      <span class="tree-node-icon">
        {#if node.fsKind === 'dir'}
          {isExpanded ? '▾' : '▸'}
        {:else}
          <span style="display:inline-block;width:12px"></span>
        {/if}
      </span>
      <span class="tree-node-label" class:tree-node-label--dir={node.fsKind === 'dir'}>
        {node.label}
      </span>
    </div>
    {#if node.fsKind === 'dir' && isExpanded && node.children}
      {@render renderTree(node.children, depth + 1)}
    {/if}
  {/each}
{/snippet}

<style>
  .file-explorer {
    display: flex;
    flex-direction: column;
    height: 100%;
    position: relative;
    overflow: hidden;
    font-size: 12px;
    color: var(--text-secondary, #9da5b4);
    user-select: none;
  }

  .explorer-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 6px 8px;
    border-bottom: 1px solid var(--border-subtle, #2d2d2d);
    flex-shrink: 0;
  }

  .explorer-title {
    font-size: 10px;
    letter-spacing: 0.08em;
    text-transform: uppercase;
    color: var(--text-ghost, #5c6370);
  }

  .explorer-btn {
    background: none;
    border: none;
    color: var(--text-secondary, #9da5b4);
    cursor: pointer;
    padding: 2px 4px;
    font-size: 14px;
    line-height: 1;
    border-radius: 3px;
  }

  .explorer-btn:hover {
    background: var(--bg-hover, #2c313a);
  }

  .new-project-row {
    display: flex;
    gap: 4px;
    padding: 4px 8px;
    flex-shrink: 0;
  }

  .new-project-input {
    flex: 1;
    background: var(--bg-input, #1d1f23);
    border: 1px solid var(--border-subtle, #2d2d2d);
    color: var(--text-primary, #abb2bf);
    padding: 3px 6px;
    font-size: 11px;
    border-radius: 3px;
    outline: none;
  }

  .project-list {
    border-bottom: 1px solid var(--border-subtle, #2d2d2d);
    flex-shrink: 0;
  }

  .project-item {
    display: flex;
    align-items: center;
    gap: 6px;
    width: 100%;
    padding: 4px 8px;
    background: none;
    border: none;
    color: var(--text-secondary, #9da5b4);
    cursor: pointer;
    text-align: left;
    font-size: 11px;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .project-item:hover {
    background: var(--bg-hover, #2c313a);
  }

  .project-item--active {
    color: var(--teal, #14b8a6);
  }

  .project-icon {
    font-size: 10px;
    flex-shrink: 0;
  }

  .project-name {
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .tree-scroll {
    flex: 1;
    overflow-y: auto;
    overflow-x: hidden;
    min-height: 0;
  }

  .tree-node {
    display: flex;
    align-items: center;
    gap: 4px;
    height: 22px;
    cursor: pointer;
    white-space: nowrap;
    overflow: hidden;
  }

  .tree-node:hover {
    background: var(--bg-hover, #2c313a);
  }

  .tree-node--selected {
    background: var(--bg-selected, #2c313a);
    color: var(--text-primary, #abb2bf);
  }

  .tree-node-icon {
    font-size: 10px;
    width: 14px;
    flex-shrink: 0;
    text-align: center;
    color: var(--text-ghost, #5c6370);
  }

  .tree-node-label {
    overflow: hidden;
    text-overflow: ellipsis;
    flex: 1;
  }

  .tree-node-label--dir {
    color: var(--text-primary, #abb2bf);
  }

  .tree-loading,
  .tree-empty,
  .tree-error {
    padding: 12px 8px;
    color: var(--text-ghost, #5c6370);
    font-size: 11px;
  }

  .tree-error {
    color: var(--fail, #e06c75);
  }

  /* Resize handle: right edge drag strip */
  .resize-handle {
    position: absolute;
    top: 0;
    right: 0;
    width: 4px;
    height: 100%;
    cursor: col-resize;
    background: transparent;
    z-index: 10;
  }

  .resize-handle:hover {
    background: var(--teal-line, rgba(20, 184, 166, 0.3));
  }
</style>
