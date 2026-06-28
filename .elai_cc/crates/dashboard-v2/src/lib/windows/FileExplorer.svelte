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
      <!-- loading: skeleton rows (nothing under ~200ms is the host's job) -->
      <div class="tree-skeleton" aria-busy="true" aria-label="loading file tree">
        {#each Array(6) as _, i}
          <div class="skeleton-row" style="width: {72 - i * 7}%"></div>
        {/each}
      </div>
    {:else if fileTree.error}
      <!-- error: mono-inversion fail chip + recovery path (§6) -->
      <div class="tree-error" role="alert">
        <span class="fail-chip">✕</span>
        <span class="tree-error__msg">{fileTree.error}</span>
        <button class="tree-error__retry" onclick={() => fileTree.loadRoot(rootPath)}>
          retry
        </button>
      </div>
    {:else if fileTree.root}
      {@render renderTree(fileTree.root.children ?? [], 0)}
    {:else}
      <!-- empty: focal next action, not a bare "no data" (§9) -->
      <div class="tree-empty">
        <span class="tree-empty__glyph">○</span>
        <span class="tree-empty__msg">no folder open</span>
      </div>
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
    font-family: var(--font-mono);
    font-size: 12px;
    color: var(--read);
    user-select: none;
  }

  .explorer-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 6px 8px;
    border-bottom: 1px solid var(--line);
    flex-shrink: 0;
  }

  /* static section label — §4 dot-prefix */
  .explorer-title {
    display: inline-flex;
    align-items: center;
    gap: 6px;
    font-size: 10px;
    letter-spacing: 0.2em;
    text-transform: uppercase;
    color: var(--dim);
  }
  .explorer-title::before {
    content: '';
    width: 5px;
    height: 5px;
    border-radius: 50%;
    background: var(--d-teal);
    flex-shrink: 0;
  }

  .explorer-btn {
    background: none;
    border: none;
    color: var(--read);
    cursor: pointer;
    padding: 2px 4px;
    font-family: var(--font-mono);
    font-size: 13px;
    line-height: 1;
    border-radius: var(--r-sm);
    transition: color var(--dur-fast) var(--ease-out),
      background-color var(--dur-fast) var(--ease-out);
  }

  .explorer-btn:hover {
    background: var(--raised);
    color: var(--text);
  }

  .new-project-row {
    display: flex;
    gap: 4px;
    padding: 4px 8px;
    flex-shrink: 0;
  }

  .new-project-input {
    flex: 1;
    background: var(--sink);
    border: 1px solid var(--line);
    color: var(--text);
    font-family: var(--font-mono);
    padding: 3px 6px;
    font-size: 10px;
    border-radius: var(--r-sm);
    outline: none;
    transition: border-color var(--dur-fast) var(--ease-out);
  }
  .new-project-input:focus { border-color: var(--d-teal); }
  .new-project-input::placeholder { color: var(--dim); }

  .project-list {
    border-bottom: 1px solid var(--line);
    flex-shrink: 0;
  }

  .project-item {
    position: relative;
    display: flex;
    align-items: center;
    gap: 6px;
    width: 100%;
    padding: 4px 8px;
    background: none;
    border: none;
    color: var(--read);
    cursor: pointer;
    text-align: left;
    font-family: var(--font-mono);
    font-size: 10px;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
    transition: background-color var(--dur-fast) var(--ease-out),
      color var(--dur-fast) var(--ease-out);
  }

  .project-item:hover {
    background: var(--raised);
    color: var(--text);
  }

  /* active project: one teal accent as text, no fill */
  .project-item--active {
    color: var(--d-teal);
  }
  .project-item--active .project-icon { color: var(--d-teal); }

  .project-icon {
    font-size: 10px;
    flex-shrink: 0;
    color: var(--dim);
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
    position: relative;
    display: flex;
    align-items: center;
    gap: 4px;
    height: 22px;
    cursor: pointer;
    white-space: nowrap;
    overflow: hidden;
    background: var(--row);
    transition: background-color var(--dur-fast) var(--ease-out),
      color var(--dur-fast) var(--ease-out);
  }

  /* hairline guides between rows */
  .tree-node + .tree-node { box-shadow: inset 0 1px 0 var(--line); }

  .tree-node:hover {
    background: var(--raised);
  }

  /* selected: teal name + left teal bar, no flat fill */
  .tree-node--selected {
    color: var(--d-teal);
  }
  .tree-node--selected::before {
    content: '';
    position: absolute;
    left: 0;
    top: 0;
    bottom: 0;
    width: 2px;
    background: var(--d-teal);
  }
  .tree-node--selected .tree-node-label,
  .tree-node--selected .tree-node-label--dir {
    color: var(--d-teal);
  }

  .tree-node-icon {
    font-size: 10px;
    width: 14px;
    flex-shrink: 0;
    text-align: center;
    color: var(--dim);
  }

  /* file names read brighter than meta; dirs strongest */
  .tree-node-label {
    overflow: hidden;
    text-overflow: ellipsis;
    flex: 1;
    color: var(--read);
  }

  .tree-node-label--dir {
    color: var(--text);
  }

  /* ---- state surfaces (§9) ---- */
  .tree-skeleton {
    display: flex;
    flex-direction: column;
    gap: 8px;
    padding: 12px 8px;
  }
  .skeleton-row {
    height: 12px;
    border-radius: var(--r-sm);
    background: var(--raised);
    border: 1px solid var(--line);
    opacity: 0.55;
    animation: skeleton-pulse 1.2s var(--ease-out) infinite alternate;
  }
  @keyframes skeleton-pulse {
    from { opacity: 0.35; }
    to   { opacity: 0.7; }
  }

  .tree-empty {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 6px;
    padding: 24px 8px;
    color: var(--dim);
    font-size: 10px;
  }
  .tree-empty__glyph {
    font-size: 15px;
    color: var(--ghost);
  }

  .tree-error {
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 12px 8px;
    font-size: 10px;
    color: var(--read);
  }
  /* §6 fail = mono-inversion chip, never red */
  .fail-chip {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    width: 16px;
    height: 16px;
    flex-shrink: 0;
    background: var(--text);
    color: var(--void);
    border-radius: var(--r-xs);
    font-size: 10px;
    line-height: 1;
  }
  .tree-error__msg {
    flex: 1;
    overflow: hidden;
    text-overflow: ellipsis;
  }
  .tree-error__retry {
    background: transparent;
    border: 1px solid var(--d-teal-soft);
    border-radius: var(--r-xs);
    color: var(--d-teal);
    font-family: var(--font-mono);
    font-size: 10px;
    padding: 1px 8px;
    cursor: pointer;
    flex-shrink: 0;
    transition: background-color var(--dur-fast) var(--ease-out);
  }
  .tree-error__retry:hover { background: var(--d-teal-ghost); }

  /* Resize handle: right edge drag strip */
  .resize-handle {
    position: absolute;
    top: 0;
    right: 0;
    width: 4px;
    height: 100%;
    cursor: col-resize;
    background: transparent;
    z-index: var(--z-shell-chrome);
  }

  .resize-handle:hover {
    background: var(--d-teal-soft);
  }

  @media (prefers-reduced-motion: reduce) {
    .skeleton-row { animation: none; opacity: 0.5; }
  }
</style>
