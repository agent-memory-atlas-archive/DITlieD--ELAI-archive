<script lang="ts">
  import type { SessionBranch, BranchStatus } from '../api/types';

  // B-9: compact branch status table. Props: branches to display.
  // Emits: onfork(), onmerge(id), onclose(id) — callers should handle
  // the CLI-delegation error from backend commands gracefully.
  // Note: fork/merge/close Tauri commands return errors in B-9 scope
  // (no SessionDag in AppState). Callers should display the error message.

  interface Props {
    branches: SessionBranch[];
    onfork?: () => void;
    onmerge?: (id: string) => void;
    onclose?: (id: string) => void;
  }

  let { branches, onfork, onmerge, onclose }: Props = $props();

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
</script>

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
</style>
