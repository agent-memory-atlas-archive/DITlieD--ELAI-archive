<script lang="ts">
  import type { SessionBranch, BranchStatus } from '../api/types';

  // B-9: compact branch status table + H-49: undo-tree timeline extension.
  //
  // H-49: when `timetravelNodes` is provided, renders the undo-tree timeline
  // in addition to (or instead of) the branch status table. The HEAD node is
  // marked, sibling branches are indicated, and Undo/Redo affordances are
  // surfaced on the current HEAD. A per-node manifest diff count is shown when
  // the node has a manifest_hash.
  //
  // Cross-lang pair: commands_h49.rs TimeTravelNode + stub.ts.

  import { timetravel_undo, timetravel_redo, timetravel_jump } from '../api/stub';

  type TimeTravelNode = {
    snap_id: string;
    parent_id: string | null;
    is_head: boolean;
    children_count: number;
    tag: string | null;
    created_at_ms: number;
    manifest_hash: string | null;
  };

  interface Props {
    branches: SessionBranch[];
    timetravelNodes?: TimeTravelNode[];
    sessionId?: string;
    onfork?: () => void;
    onmerge?: (id: string) => void;
    onclose?: (id: string) => void;
    onundoredo?: () => void;
  }

  let { branches, timetravelNodes, sessionId, onfork, onmerge, onclose, onundoredo }: Props = $props();

  let ttError = $state<string | null>(null);
  let ttBusy = $state<boolean>(false);

  function statusColor(status: BranchStatus): string {
    switch (status) {
      case 'active':    return 'var(--teal)';
      case 'merged':    return 'var(--steel)';
      case 'abandoned': return 'var(--text-ghost)';
    }
  }

  function relativeTime(ts: number): string {
    const diff = Date.now() - ts;
    const mins = Math.floor(diff / 60_000);
    if (mins < 1)  return 'just now';
    if (mins < 60) return `${mins}m ago`;
    const hrs = Math.floor(mins / 60);
    if (hrs < 24)  return `${hrs}h ago`;
    return `${Math.floor(hrs / 24)}d ago`;
  }

  async function handleUndo() {
    if (ttBusy || !sessionId) return;
    ttBusy = true;
    ttError = null;
    try {
      await timetravel_undo(sessionId, 'code');
      onundoredo?.();
    } catch (e) {
      ttError = e instanceof Error ? e.message : String(e);
    } finally {
      ttBusy = false;
    }
  }

  async function handleRedo() {
    if (ttBusy || !sessionId) return;
    ttBusy = true;
    ttError = null;
    try {
      await timetravel_redo(sessionId, 'code');
      onundoredo?.();
    } catch (e) {
      ttError = e instanceof Error ? e.message : String(e);
    } finally {
      ttBusy = false;
    }
  }

  async function handleJump(snapId: string) {
    if (ttBusy || !sessionId) return;
    ttBusy = true;
    ttError = null;
    try {
      await timetravel_jump({ session_id: sessionId, target: snapId, scope: 'code', dry_run: false });
      onundoredo?.();
    } catch (e) {
      ttError = e instanceof Error ? e.message : String(e);
    } finally {
      ttBusy = false;
    }
  }

  // Determine if any node in the tree is the HEAD and has children (redo available).
  let headNode = $derived(timetravelNodes?.find(n => n.is_head));
  let redoAvailable = $derived((headNode?.children_count ?? 0) > 0);
</script>

{#if timetravelNodes && timetravelNodes.length > 0}
  <!-- H-49 undo-tree timeline -->
  <div class="tt-tree" role="region" aria-label="Time-travel undo-tree">
    <div class="tt-header">
      <span class="tt-title mono">undo tree</span>
      {#if sessionId}
        <div class="tt-controls">
          <button
            class="act-btn"
            onclick={handleUndo}
            disabled={ttBusy}
            title="Undo last LLM edit (restore files to pre-edit state)"
          >undo</button>
          <button
            class="act-btn"
            onclick={handleRedo}
            disabled={ttBusy || !redoAvailable}
            title={redoAvailable ? 'Redo forward' : 'No redo available (at tip)'}
          >redo</button>
        </div>
      {/if}
    </div>

    {#if ttError}
      <div class="tt-error">{ttError}</div>
    {/if}

    <div class="tt-nodes">
      {#each timetravelNodes as node (node.snap_id)}
        <div
          class="tt-node"
          class:tt-node--head={node.is_head}
          class:tt-node--branch={node.children_count > 1}
          role="row"
        >
          <span class="tt-node-marker" aria-hidden="true">
            {#if node.is_head}●{:else if node.children_count > 1}⑂{:else}○{/if}
          </span>
          <span class="tt-node-id mono" title={node.snap_id}>
            {node.snap_id.slice(0, 8)}
          </span>
          {#if node.tag}
            <span class="tt-node-tag mono">{node.tag}</span>
          {/if}
          <span class="tt-node-time muted">{relativeTime(node.created_at_ms)}</span>
          {#if node.manifest_hash}
            <span class="tt-node-manifest muted" title="manifest: {node.manifest_hash.slice(0, 12)}">
              files
            </span>
          {/if}
          {#if node.children_count > 1}
            <span class="tt-branch-count muted">{node.children_count} branches</span>
          {/if}
          {#if !node.is_head && sessionId}
            <button
              class="act-btn act-btn--xs"
              onclick={() => handleJump(node.snap_id)}
              disabled={ttBusy}
              title="Jump to this snapshot"
            >jump</button>
          {:else if node.is_head}
            <span class="tt-head-label">HEAD</span>
          {/if}
        </div>
      {/each}
    </div>
  </div>
{/if}

<!-- B-9 branch status table (unchanged) -->
<div class="branch-tree" role="table" aria-label="Session branches">
  <div class="bt-header" role="row">
    <span class="col-name" role="columnheader">branch</span>
    <span class="col-status" role="columnheader">status</span>
    <span class="col-time" role="columnheader">activity</span>
    <span class="col-actions" role="columnheader" aria-label="actions"></span>
  </div>

  {#if branches.length === 0}
    <div class="bt-empty">no branch history</div>
  {/if}

  {#each branches as branch (branch.id)}
    <div class="bt-row" role="row">
      <span class="col-name mono" role="cell" title={branch.id}>{branch.label}</span>
      <span class="col-status" role="cell">
        <span class="status-dot" style="background: {statusColor(branch.status)};" aria-hidden="true"></span>
        <span class="status-label mono" style="color: {statusColor(branch.status)};">{branch.status}</span>
      </span>
      <span class="col-time mono muted" role="cell">{relativeTime(branch.last_activity_ts)}</span>
      <span class="col-actions" role="cell">
        {#if branch.status === 'active'}
          <button
            class="act-btn"
            onclick={() => onmerge?.(branch.id)}
            title="Merge branch (CLI-delegated)"
            aria-label="Merge {branch.label}"
          >merge</button>
          <button
            class="act-btn act-btn--danger"
            onclick={() => onclose?.(branch.id)}
            title="Close branch (CLI-delegated)"
            aria-label="Close {branch.label}"
          >close</button>
        {/if}
      </span>
    </div>
  {/each}

  <div class="bt-footer">
    <button class="act-btn" onclick={() => onfork?.()} title="Fork new branch (CLI-delegated)">
      + fork
    </button>
  </div>
</div>

<style>
  /* H-49 undo-tree styles */
  .tt-tree {
    display: flex;
    flex-direction: column;
    gap: var(--s-1);
    margin-bottom: var(--s-3);
    border-bottom: 1px solid var(--border-subtle);
    padding-bottom: var(--s-2);
  }
  .tt-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: var(--s-2);
  }
  .tt-title {
    font-size: 10px;
    color: var(--text-ghost);
    text-transform: lowercase;
    letter-spacing: 0.04em;
  }
  .tt-controls {
    display: flex;
    gap: var(--s-1);
  }
  .tt-error {
    font-family: var(--font-mono);
    font-size: 10px;
    color: var(--fail);
    padding: 2px 4px;
  }
  .tt-nodes {
    display: flex;
    flex-direction: column;
    gap: 2px;
  }
  .tt-node {
    display: flex;
    align-items: center;
    gap: var(--s-1);
    padding: 2px 4px;
    border-radius: var(--r-xs);
    font-family: var(--font-mono);
    font-size: 10px;
    transition: background 80ms var(--ease-std);
  }
  .tt-node:hover {
    background: var(--bg-hover);
  }
  .tt-node--head {
    background: color-mix(in srgb, var(--teal) 8%, transparent);
    border-left: 2px solid var(--teal);
  }
  .tt-node--branch {
    border-left: 2px solid var(--signal-amber, var(--text-secondary));
  }
  .tt-node-marker {
    width: 12px;
    text-align: center;
    color: var(--text-secondary);
  }
  .tt-node--head .tt-node-marker { color: var(--teal); }
  .tt-node-id {
    color: var(--text-primary);
    min-width: 60px;
  }
  .tt-node-tag {
    color: var(--text-ghost);
    font-size: 10px;
    background: var(--bg-hover);
    padding: 0 3px;
    border-radius: 2px;
  }
  .tt-node-time { color: var(--text-ghost); font-size: 10px; }
  .tt-node-manifest { font-size: 10px; }
  .tt-branch-count { font-size: 10px; color: var(--text-secondary); }
  .tt-head-label {
    font-size: 10px;
    color: var(--teal);
    text-transform: uppercase;
    letter-spacing: 0.06em;
    margin-left: auto;
  }

  /* B-9 branch table styles (unchanged) */
  .branch-tree {
    display: flex;
    flex-direction: column;
    width: 100%;
    font-size: 10px;
  }

  .bt-header, .bt-row {
    display: grid;
    grid-template-columns: 1fr 80px 72px 80px;
    align-items: center;
    height: 26px;
    padding: 0 8px;
    gap: 4px;
  }

  .bt-header {
    border-bottom: 1px solid var(--border-subtle);
    color: var(--text-ghost);
    font-family: var(--font-mono);
    font-size: 10px;
    text-transform: lowercase;
    letter-spacing: 0.04em;
  }

  .bt-row {
    border-bottom: 1px solid var(--border-subtle);
    transition: background 120ms var(--ease-std);
  }

  .bt-row:hover {
    background: var(--bg-hover);
  }

  .bt-empty {
    padding: 12px 8px;
    font-family: var(--font-mono);
    font-size: 10px;
    color: var(--text-ghost);
  }

  .bt-footer {
    padding: 6px 8px;
  }

  .mono { font-family: var(--font-mono); }
  .muted { color: var(--text-ghost); }

  .col-name {
    min-width: 0;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    color: var(--text-primary);
  }

  .col-status {
    display: flex;
    align-items: center;
    gap: 4px;
  }

  .status-dot {
    width: 5px;
    height: 5px;
    border-radius: 50%;
    flex-shrink: 0;
  }

  .status-label {
    font-size: 10px;
    letter-spacing: 0.03em;
  }

  .col-time {
    font-size: 10px;
    color: var(--text-ghost);
    white-space: nowrap;
  }

  .col-actions {
    display: flex;
    align-items: center;
    gap: 4px;
    justify-content: flex-end;
  }

  .act-btn {
    font-family: var(--font-mono);
    font-size: 10px;
    background: var(--bg-hover);
    border: 1px solid var(--border-subtle);
    border-radius: var(--r-xs);
    color: var(--text-secondary);
    cursor: pointer;
    padding: 1px 5px;
    outline: none;
    transition: background 120ms var(--ease-std), color 120ms var(--ease-std);
  }

  .act-btn--xs {
    font-size: 10px;
    padding: 0 4px;
    margin-left: auto;
  }

  .act-btn:hover {
    background: var(--bg-surface);
    color: var(--text-primary);
  }

  .act-btn:focus-visible {
    box-shadow: var(--ring);
  }

  .act-btn--danger:hover {
    color: var(--fail);
  }

  .act-btn:disabled {
    opacity: 0.4;
    cursor: not-allowed;
  }
</style>
