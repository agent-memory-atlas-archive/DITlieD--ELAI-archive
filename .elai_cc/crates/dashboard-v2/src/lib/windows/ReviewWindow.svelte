<script lang="ts">
  // ============================================================
  // ReviewWindow — live SSB-backed reviewer surface (C-7)
  // ============================================================
  // Wired to:
  //   - invoke('get_file_changes', { sessionId })    — read FileWrite rows
  //   - invoke('accept_change',  { path, hunkId? })  — record verdict
  //   - invoke('decline_change', { path, hunkId? })  — record verdict
  //   - invoke('retry_audit',    { path, hunkId?, note? }) — record verdict
  //
  // Empty-state path: when get_file_changes returns [] (cold-start session
  // or no edits yet) we show the <Empty> surface — NO static fallback.
  //
  // Per-hunk action row (DRIFT-4): always-visible Accept/Decline/Retry
  // buttons rendered BELOW each hunk separator (replaces prior hover-only
  // inline buttons).
  //
  // Global shortcuts (Ctrl+Shift+A / Ctrl+Shift+R) fire via reviewStore
  // signal counters which we observe with $effect.

  import { onMount } from 'svelte';
  import { Button, Empty, Scrollbar } from '$lib/ui';
  import * as stub from '$lib/api/stub';
  import type { FileChangeRow } from '$lib/api/types';
  import { session } from '../state/session.svelte';
  import { reviewStore } from '../state/review.svelte';
  // C-24 (DRIFT-8) — cross-link FileWrite rows to GraphWindow with a
  // blast-radius overlay. Mirrors the visualizeImpact() pattern in
  // KnowledgeDrawer.svelte:14-19. ActivityWindow.svelte does not render
  // file-write rows (it's an OrbitCanvas host) so the plan's task #7 was
  // retargeted here per workflow/reasoning/C-24-decisions.md DECISION-7.
  import { graph } from '../state/graph.svelte';
  import { workspace } from '../state/workspace.svelte';

  function showBlastRadius(path: string) {
    if (!path) return;
    // Drop into file mode + push the impact target. GraphWindow's
    // $effect on graph.impact_target picks this up after openTab, calls
    // get_impact_overlay, and applies the red-gradient class set.
    graph.set_mode('file');
    graph.request_impact(path);
    workspace.openTab('primary', 'graph');
    stub.logStub('review.showBlastRadius', { path });
  }

  // ------------------------------------------------------------------
  // Live file-changes wiring
  // ------------------------------------------------------------------

  let files = $state<FileChangeRow[]>([]);
  let loading = $state<boolean>(true);
  let activeIndex = $state(0);
  let activeFile = $derived(files[activeIndex]);

  async function loadFileChanges() {
    loading = true;
    try {
      const rows = await stub.get_file_changes(session.sessionId);
      files = rows;
      if (activeIndex >= files.length) activeIndex = 0;
    } finally {
      loading = false;
    }
  }

  onMount(() => {
    void loadFileChanges();
  });

  // ------------------------------------------------------------------
  // Per-hunk + per-file verdict state (in-memory)
  //
  // hunkState / fileState keys are the path or path#hunkLabel; the
  // backend ChangeDecision SSB rows are the source of truth across
  // reloads. Local state lets the UI react instantly without re-polling.
  // ------------------------------------------------------------------

  let hunkState = $state<Record<string, 'accepted' | 'declined' | null>>({});
  let fileState = $state<Record<string, 'accepted' | 'declined' | null>>({});
  let retryNote = $state<Record<string, string>>({});
  let retryNoteVisible = $state<Record<string, boolean>>({});

  function hunkKey(path: string, hunkLabel: string): string {
    return `${path}#${hunkLabel}`;
  }

  function acceptHunk(path: string, hunkLabel: string) {
    const key = hunkKey(path, hunkLabel);
    hunkState = { ...hunkState, [key]: 'accepted' };
    void stub.accept_change(path, hunkLabel);
  }

  function declineHunk(path: string, hunkLabel: string) {
    const key = hunkKey(path, hunkLabel);
    hunkState = { ...hunkState, [key]: 'declined' };
    void stub.decline_change(path, hunkLabel);
  }

  function retryHunk(path: string, hunkLabel: string) {
    const key = hunkKey(path, hunkLabel);
    const note = retryNote[key]?.trim() ?? '';
    hunkState = { ...hunkState, [key]: null };
    retryNoteVisible = { ...retryNoteVisible, [key]: false };
    void stub.retry_audit(path, hunkLabel, note.length > 0 ? note : undefined);
  }

  function toggleHunkRetryNote(path: string, hunkLabel: string) {
    const key = hunkKey(path, hunkLabel);
    retryNoteVisible = { ...retryNoteVisible, [key]: !retryNoteVisible[key] };
  }

  function acceptAll(path: string) {
    fileState = { ...fileState, [path]: 'accepted' };
    void stub.accept_change(path);
  }

  function declineAll(path: string) {
    fileState = { ...fileState, [path]: 'declined' };
    void stub.decline_change(path);
  }

  function retryFile(path: string) {
    const note = retryNote[path]?.trim() ?? '';
    fileState = { ...fileState, [path]: null };
    retryNoteVisible = { ...retryNoteVisible, [path]: false };
    void stub.retry_audit(path, undefined, note.length > 0 ? note : undefined);
  }

  function toggleRetryNote(path: string) {
    retryNoteVisible = { ...retryNoteVisible, [path]: !retryNoteVisible[path] };
  }

  // ------------------------------------------------------------------
  // Hunk parsing — turn the FileChangeRow.summary into a hunk list.
  //
  // FileChangeRow today carries {path, action, summary}; the raw diff
  // body lives in the FileWrite SSB row, retrievable via
  // get_action_detail(seq) — that detail-fetch is a follow-up. For now
  // we render one synthetic hunk per file labeled with the action +
  // summary so the per-hunk action row is exercised on every file.
  // ------------------------------------------------------------------

  interface HunkLabel {
    label: string;
  }

  function hunkLabelsFor(file: FileChangeRow): HunkLabel[] {
    return [{ label: `${file.action}: ${file.summary}` }];
  }

  // ------------------------------------------------------------------
  // Global shortcut subscriptions (Ctrl+Shift+A / Ctrl+Shift+R)
  //
  // reviewStore exposes monotonic counters; we observe via $effect and
  // fire bulk actions on every increment. Both shortcuts are guarded
  // by shortcuts.ts to only fire when primary active tab is 'review'.
  // ------------------------------------------------------------------

  let lastAcceptSignal = 0;
  let lastRejectSignal = 0;

  $effect(() => {
    const sig = reviewStore.acceptAllSignal;
    if (sig !== lastAcceptSignal) {
      lastAcceptSignal = sig;
      if (sig > 0) {
        // Accept every file's every synthetic hunk.
        for (const f of files) {
          fileState = { ...fileState, [f.path]: 'accepted' };
          for (const h of hunkLabelsFor(f)) {
            hunkState = { ...hunkState, [hunkKey(f.path, h.label)]: 'accepted' };
          }
          void stub.accept_change(f.path);
        }
        stub.logStub('review.bulkAcceptAll', { count: files.length });
      }
    }
  });

  $effect(() => {
    const sig = reviewStore.rejectAllSignal;
    if (sig !== lastRejectSignal) {
      lastRejectSignal = sig;
      if (sig > 0) {
        for (const f of files) {
          fileState = { ...fileState, [f.path]: 'declined' };
          for (const h of hunkLabelsFor(f)) {
            hunkState = { ...hunkState, [hunkKey(f.path, h.label)]: 'declined' };
          }
          void stub.decline_change(f.path);
        }
        stub.logStub('review.bulkRejectAll', { count: files.length });
      }
    }
  });
</script>

<div class="review" data-testid="review-window">
  {#if loading}
    <div class="review__loading">
      <span class="review__loading-text">loading file changes…</span>
    </div>
  {:else if files.length === 0}
    <div class="review__empty">
      <Empty
        title="no changes to review"
        subtitle="review surfaces here when the auditor proposes a change"
      />
    </div>
  {:else}

  <!-- Tab strip -->
  <div class="tab-strip" role="tablist" aria-label="Changed files">
    {#each files as file, i (file.path)}
      <button
        class="file-tab"
        class:file-tab--active={i === activeIndex}
        role="tab"
        aria-selected={i === activeIndex}
        onclick={() => { activeIndex = i; }}
      >
        {file.path.split('/').pop()}
      </button>
    {/each}
  </div>

  {#if activeFile}
    <!-- Sticky file header -->
    <div class="file-header">
      <span class="file-path">{activeFile.path}</span>
      <span class="file-action">{activeFile.action}</span>
      <div class="file-actions">
        <Button
          variant="primary"
          size="sm"
          onclick={() => acceptAll(activeFile.path)}
        >
          Accept All
        </Button>
        <Button
          variant="ghost"
          size="sm"
          onclick={() => declineAll(activeFile.path)}
        >
          Decline All
        </Button>
        <Button
          variant="ghost"
          size="sm"
          onclick={() => toggleRetryNote(activeFile.path)}
        >
          Retry
        </Button>
        <!-- C-24 (DRIFT-8): cross-link to GraphWindow with blast-radius
             overlay. Mirrors KnowledgeDrawer.svelte:307-312. The Button
             component does not accept arbitrary HTML attributes, so a
             plain <button> is used so the test selector
             [data-testid=review-blast-radius] can find it. -->
        <button
          class="blast-radius-btn"
          type="button"
          onclick={() => showBlastRadius(activeFile.path)}
          data-testid="review-blast-radius"
        >
          Show blast radius
        </button>
      </div>
    </div>

    {#if retryNoteVisible[activeFile.path]}
      <div class="retry-note-bar">
        <input
          class="retry-note-input mono"
          type="text"
          placeholder="optional note for auditor (Enter to confirm)"
          bind:value={retryNote[activeFile.path]}
          onkeydown={(e) => { if (e.key === 'Enter') { e.preventDefault(); retryFile(activeFile.path); } }}
        />
        <button class="retry-note-confirm" onclick={() => retryFile(activeFile.path)}>retry</button>
      </div>
    {/if}

    {#if fileState[activeFile.path]}
      <div class="file-verdict file-verdict--{fileState[activeFile.path]}">
        {fileState[activeFile.path] === 'accepted' ? 'All changes accepted' : 'All changes declined'}
        <button class="file-verdict__undo" onclick={() => retryFile(activeFile.path)}>Undo</button>
      </div>
    {/if}

    <!-- Diff body: synthetic hunks from FileChangeRow summary -->
    <Scrollbar>
      <div class="diff-body">
        {#each hunkLabelsFor(activeFile) as hunk}
          {@const key = hunkKey(activeFile.path, hunk.label)}
          <div class="diff-hunk" role="group" aria-label={hunk.label}>
            <span class="diff-hunk__label">{hunk.label}</span>
            {#if hunkState[key]}
              <span class="diff-hunk__verdict diff-hunk__verdict--{hunkState[key]}">
                {hunkState[key]}
              </span>
            {/if}
          </div>

          <!-- Always-visible per-hunk action row (DRIFT-4 rewrite) -->
          <div class="hunk-row" data-testid="hunk-row">
            <button
              class="hunk-btn"
              class:hunk-btn--active={hunkState[key] === 'accepted'}
              onclick={() => acceptHunk(activeFile.path, hunk.label)}
              data-testid="hunk-accept"
            >Accept</button>
            <button
              class="hunk-btn"
              class:hunk-btn--active={hunkState[key] === 'declined'}
              onclick={() => declineHunk(activeFile.path, hunk.label)}
              data-testid="hunk-decline"
            >Decline</button>
            <button
              class="hunk-btn"
              onclick={() => toggleHunkRetryNote(activeFile.path, hunk.label)}
              data-testid="hunk-retry"
            >Retry</button>
          </div>

          {#if retryNoteVisible[key]}
            <div class="retry-note-bar">
              <input
                class="retry-note-input mono"
                type="text"
                placeholder="optional note for auditor (Enter to confirm)"
                bind:value={retryNote[key]}
                onkeydown={(e) => { if (e.key === 'Enter') { e.preventDefault(); retryHunk(activeFile.path, hunk.label); } }}
              />
              <button
                class="retry-note-confirm"
                onclick={() => retryHunk(activeFile.path, hunk.label)}
              >retry</button>
            </div>
          {/if}
        {/each}
      </div>
    </Scrollbar>
  {/if}
  {/if}
</div>

<style>
  .review {
    display: flex;
    flex-direction: column;
    height: 100%;
    background: var(--bg-base);
    overflow: hidden;
  }

  .review__loading {
    flex: 1;
    display: flex;
    align-items: center;
    justify-content: center;
  }

  .review__loading-text {
    font-family: var(--font-mono);
    font-size: 11px;
    color: var(--text-ghost);
  }

  /* Tab strip */
  .tab-strip {
    display: flex;
    flex-shrink: 0;
    overflow-x: auto;
    background: var(--bg-surface);
    border-bottom: 1px solid var(--border-subtle);
    scrollbar-width: none;
  }
  .tab-strip::-webkit-scrollbar { display: none; }

  .file-tab {
    flex-shrink: 0;
    height: 32px;
    padding: 0 14px;
    background: transparent;
    border: none;
    border-bottom: 2px solid transparent;
    color: var(--text-secondary);
    font-family: var(--font-mono);
    font-size: 10px;
    cursor: pointer;
    white-space: nowrap;
    transition: color var(--dur-fast) var(--ease-std), border-color var(--dur-fast) var(--ease-std);
    outline: none;
  }
  .file-tab:hover { color: var(--text-primary); }
  .file-tab--active {
    color: var(--teal);
    border-bottom-color: var(--teal);
  }
  .file-tab:focus-visible { box-shadow: inset 0 0 0 1px var(--teal); }

  /* File header */
  .file-header {
    display: flex;
    align-items: center;
    gap: var(--s-3);
    height: 32px;
    padding: 0 var(--s-3);
    background: var(--bg-surface);
    border-bottom: 1px solid var(--border-subtle);
    flex-shrink: 0;
  }
  .file-path {
    font-family: var(--font-mono);
    font-size: 12px;
    color: var(--text-primary);
    flex: 1;
    min-width: 0;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }
  .file-action {
    font-family: var(--font-mono);
    font-size: 10px;
    color: var(--text-secondary);
    text-transform: uppercase;
    letter-spacing: 0.1em;
    flex-shrink: 0;
  }
  .file-actions {
    display: flex;
    gap: var(--s-1);
    flex-shrink: 0;
  }
  /* C-24 (DRIFT-8) — blast-radius cross-link button. Mirrors the ghost
     variant of the Button component (the latter doesn't accept arbitrary
     HTML attrs, so we render a plain button + the matching styles). */
  .blast-radius-btn {
    background: transparent;
    border: 1px solid var(--border-subtle);
    border-radius: var(--r-xs);
    color: var(--text-secondary);
    font-family: var(--font-mono);
    font-size: 10px;
    padding: 2px 8px;
    cursor: pointer;
    transition: background-color var(--dur-fast) var(--ease-std), color var(--dur-fast) var(--ease-std), border-color var(--dur-fast) var(--ease-std);
  }
  .blast-radius-btn:hover {
    background: var(--teal-ghost);
    color: var(--teal);
    border-color: var(--teal-line);
  }

  /* File verdict banner */
  .file-verdict {
    display: flex;
    align-items: center;
    gap: var(--s-2);
    padding: 4px 12px;
    font-family: var(--font-mono);
    font-size: 10px;
    flex-shrink: 0;
  }
  .file-verdict--accepted { background: var(--teal-ghost); color: var(--teal); }
  .file-verdict--declined { background: rgba(176,69,69,0.10); color: var(--fail); }
  .file-verdict__undo {
    background: transparent;
    border: none;
    color: var(--text-ghost);
    font-family: var(--font-mono);
    font-size: 10px;
    cursor: pointer;
    padding: 0 4px;
    text-decoration: underline;
  }
  .file-verdict__undo:hover { color: var(--text-secondary); }

  /* Retry note bar (file + hunk) */
  .retry-note-bar {
    display: flex;
    align-items: center;
    gap: var(--s-2);
    padding: 4px 12px;
    background: var(--bg-input);
    border-bottom: 1px solid var(--border-subtle);
    flex-shrink: 0;
  }
  .retry-note-input {
    flex: 1;
    background: transparent;
    border: none;
    border-bottom: 1px solid var(--border-subtle);
    color: var(--text-primary);
    font-size: 10px;
    padding: 2px 0;
    outline: none;
    transition: border-color var(--dur-fast) var(--ease-std);
  }
  .retry-note-input:focus { border-bottom-color: var(--teal); }
  .retry-note-input::placeholder { color: var(--text-ghost); }
  .retry-note-confirm {
    background: transparent;
    border: 1px solid var(--teal-line);
    border-radius: var(--r-xs);
    color: var(--teal);
    font-family: var(--font-mono);
    font-size: 10px;
    padding: 1px 8px;
    cursor: pointer;
    flex-shrink: 0;
  }
  .retry-note-confirm:hover { background: var(--teal-ghost); }

  /* Diff body */
  .diff-body {
    display: flex;
    flex-direction: column;
    font-family: var(--font-mono);
    font-size: 12px;
  }

  /* Hunk header (separator line) */
  .diff-hunk {
    display: flex;
    align-items: center;
    background: var(--bg-input);
    padding: 4px 12px;
    color: var(--text-secondary);
    font-size: 10px;
    gap: var(--s-2);
    position: relative;
    min-height: 24px;
  }
  .diff-hunk__label { flex: 1; }
  .diff-hunk__verdict {
    font-size: 10px;
  }
  .diff-hunk__verdict--accepted { color: var(--teal); }
  .diff-hunk__verdict--declined { color: var(--fail); }

  /* Always-visible per-hunk action row (C-7 DRIFT-4 rewrite) */
  .hunk-row {
    display: flex;
    align-items: center;
    gap: var(--s-1);
    padding: 4px 12px;
    background: var(--bg-base);
    border-bottom: 1px solid var(--border-subtle);
  }

  .hunk-btn {
    background: transparent;
    border: 1px solid var(--border-subtle);
    border-radius: var(--r-xs);
    color: var(--text-secondary);
    font-family: var(--font-mono);
    font-size: 10px;
    padding: 2px 8px;
    cursor: pointer;
    transition: background-color var(--dur-fast) var(--ease-std), color var(--dur-fast) var(--ease-std), border-color var(--dur-fast) var(--ease-std);
  }
  .hunk-btn:hover { background: var(--bg-hover); color: var(--text-primary); }
  .hunk-btn--active { color: var(--teal); border-color: var(--teal-line); background: var(--teal-ghost); }

  .review__empty {
    flex: 1;
    display: flex;
    align-items: center;
    justify-content: center;
  }
</style>
