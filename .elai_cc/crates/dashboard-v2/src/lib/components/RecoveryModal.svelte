<script lang="ts">
  // ============================================================
  // RecoveryModal — closed-tab recovery surface (C-7)
  // ============================================================
  // Lists workspace.closedTabs (newest first). Each row's Restore
  // button calls workspace.openTab('primary', entry.type) to re-open
  // a fresh tab of the same WindowType.
  //
  // Identity-by-type semantics (DRIFT-5): we do NOT restore the
  // closed tab's original id / per-window internal state. The plan
  // task #3 stores only {type, title, closedAt} per entry.
  //
  // Mounted inside Workspace.svelte overlay snippet alongside
  // CommandPalette (DRIFT-1).

  import { X, RotateCcw } from '@lucide/svelte';
  import { workspace } from '../state/workspace.svelte';

  function close() {
    workspace.closeWindowRecovery();
  }

  function restore(idx: number) {
    const entry = workspace.closedTabs[idx];
    if (!entry) return;
    workspace.openTab('primary', entry.type);
    close();
  }

  function handleKeydown(e: KeyboardEvent) {
    if (e.key === 'Escape') {
      e.preventDefault();
      close();
    }
  }

  function handleBackdropClick(e: MouseEvent) {
    if ((e.target as HTMLElement).classList.contains('recovery-backdrop')) {
      close();
    }
  }

  function relativeTime(ts: number): string {
    const diffMs = Date.now() - ts;
    if (diffMs < 1_000) return 'just now';
    const sec = Math.floor(diffMs / 1_000);
    if (sec < 60) return `${sec}s ago`;
    const min = Math.floor(sec / 60);
    if (min < 60) return `${min}m ago`;
    const hr = Math.floor(min / 60);
    if (hr < 24) return `${hr}h ago`;
    const d = Math.floor(hr / 24);
    return `${d}d ago`;
  }
</script>

<!-- svelte-ignore a11y_no_static_element_interactions -->
<div
  class="recovery-backdrop"
  role="presentation"
  onclick={handleBackdropClick}
  onkeydown={handleKeydown}
  data-testid="recovery-modal"
>
  <div
    class="recovery"
    role="dialog"
    aria-modal="true"
    aria-label="Recover closed window"
  >
    <div class="recovery__header">
      <span class="recovery__title">Recover closed window</span>
      <button
        class="recovery__close"
        onclick={close}
        aria-label="Close recovery modal"
      >
        <X size={14} stroke-width={1.5} />
      </button>
    </div>

    <div class="recovery__list" role="list">
      {#if workspace.closedTabs.length === 0}
        <div class="recovery__empty">
          <span class="empty-label">No recently closed windows</span>
        </div>
      {:else}
        {#each workspace.closedTabs as entry, idx (entry.closedAt)}
          <button
            class="recovery__row"
            onclick={() => restore(idx)}
            data-testid="recovery-row"
          >
            <span class="recovery__type">{entry.type}</span>
            <span class="recovery__name">{entry.title}</span>
            <span class="recovery__ts">{relativeTime(entry.closedAt)}</span>
            <span class="recovery__action">
              <RotateCcw size={12} stroke-width={1.5} />
              <span>Restore</span>
            </span>
          </button>
        {/each}
      {/if}
    </div>
  </div>
</div>

<style>
  .recovery-backdrop {
    position: absolute;
    inset: 0;
    background: var(--bg-overlay);
    backdrop-filter: blur(4px);
    display: flex;
    justify-content: center;
    align-items: flex-start;
    padding-top: 64px;
    pointer-events: auto;
    z-index: 100;
  }

  .recovery {
    width: min(540px, 92vw);
    max-height: 70vh;
    background: var(--bg-surface);
    border: 1px solid var(--border-default);
    border-radius: var(--r-sm);
    box-shadow: 0 12px 32px rgba(0, 0, 0, 0.4);
    display: flex;
    flex-direction: column;
    overflow: hidden;
  }

  .recovery__header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: var(--s-2) var(--s-3);
    border-bottom: 1px solid var(--border-subtle);
    background: var(--bg-surface);
    flex-shrink: 0;
  }

  .recovery__title {
    font-family: var(--font-mono);
    font-size: 12px;
    color: var(--teal);
    text-transform: uppercase;
    letter-spacing: 0.15em;
  }

  .recovery__close {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 24px;
    height: 24px;
    background: transparent;
    border: none;
    border-radius: var(--r-xs);
    color: var(--text-secondary);
    cursor: pointer;
    outline: none;
    transition: color var(--dur-fast) var(--ease-std);
  }

  .recovery__close:hover {
    color: var(--text-primary);
  }

  .recovery__close:focus-visible {
    box-shadow: var(--ring);
  }

  .recovery__list {
    display: flex;
    flex-direction: column;
    overflow-y: auto;
    flex: 1;
  }

  .recovery__empty {
    display: flex;
    align-items: center;
    justify-content: center;
    padding: var(--s-4);
    font-family: var(--font-mono);
    font-size: 11px;
    color: var(--text-ghost);
  }

  .recovery__row {
    display: grid;
    grid-template-columns: 96px 1fr auto auto;
    align-items: center;
    gap: var(--s-3);
    padding: var(--s-2) var(--s-3);
    background: transparent;
    border: none;
    border-bottom: 1px solid var(--border-subtle);
    cursor: pointer;
    text-align: left;
    outline: none;
    transition: background var(--dur-fast) var(--ease-std);
  }

  .recovery__row:hover {
    background: var(--bg-input);
  }

  .recovery__row:focus-visible {
    background: var(--bg-input);
    box-shadow: inset 0 0 0 1px var(--teal);
  }

  .recovery__type {
    font-family: var(--font-mono);
    font-size: 10px;
    color: var(--teal);
    text-transform: uppercase;
    letter-spacing: 0.1em;
  }

  .recovery__name {
    font-family: var(--font-mono);
    font-size: 12px;
    color: var(--text-primary);
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    min-width: 0;
  }

  .recovery__ts {
    font-family: var(--font-mono);
    font-size: 10px;
    color: var(--text-ghost);
  }

  .recovery__action {
    display: inline-flex;
    align-items: center;
    gap: 4px;
    font-family: var(--font-mono);
    font-size: 10px;
    color: var(--teal);
    text-transform: uppercase;
    letter-spacing: 0.1em;
  }
</style>
