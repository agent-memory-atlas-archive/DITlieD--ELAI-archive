<script lang="ts">
  // E-1 dashboard slice — Chat-memory drawer.
  //
  // Read-only surface for the A-MEM `memory_notes` Zettelkasten store
  // (arXiv:2502.12110 §3.2) plus the MemGPT-style `core_blocks`
  // pinned-context body (arXiv:2310.08560 §3.2). Pairs with the four
  // SSB beacons added in SCHEMA_VERSION 68 (MemoryNoteInserted /
  // MemoryNoteEvolved / MemoryConflictDetected / CoreBlockReplaced) —
  // those rows surface in the SsbActionList; this panel surfaces the
  // underlying store state.

  import { onMount, onDestroy } from 'svelte';
  import { invoke } from '@tauri-apps/api/core';

  type MemoryNoteRow = {
    note_id: string;
    scope: string;
    tags: string[];
    body: string;
    links: string[];
    link_count: number;
    created_at_ms: number;
    last_accessed_ms: number;
    trust_score: number;
  };

  type MemoryNotesResponse = {
    store_path: string;
    store_just_created: boolean;
    notes: MemoryNoteRow[];
  };

  type ChatMemoryState = {
    core_block_text: string;
    core_block_bytes: number;
    last_reflection_ms: number | null;
    sleep_compute_active: boolean;
    recent_notes_count: number;
  };

  let storePath = $state<string>('');
  let storeJustCreated = $state(false);
  let notes = $state<MemoryNoteRow[]>([]);
  let loaded = $state(false);
  let loadError = $state<string | null>(null);
  let scopeFilter = $state<string>('all');
  let chatState = $state<ChatMemoryState | null>(null);
  let chatStateError = $state<string | null>(null);
  let pollInterval: ReturnType<typeof setInterval> | null = null;

  const SCOPES = ['session', 'project', 'global'];

  async function reload() {
    // Notes list.
    try {
      const res = await invoke<MemoryNotesResponse>('get_memory_notes', {
        scope: scopeFilter === 'all' ? null : scopeFilter,
        limit: 50,
      });
      storePath = res.store_path;
      storeJustCreated = res.store_just_created;
      notes = res.notes;
      loadError = null;
    } catch (e) {
      loadError = String(e);
    } finally {
      loaded = true;
    }
    // Composite header snapshot.
    try {
      chatState = await invoke<ChatMemoryState>('get_chat_memory_state');
      chatStateError = null;
    } catch (e) {
      chatStateError = String(e);
    }
  }

  $effect(() => {
    void scopeFilter;
    reload();
  });

  onMount(() => {
    reload();
    pollInterval = setInterval(reload, 15_000);
  });

  onDestroy(() => {
    if (pollInterval !== null) clearInterval(pollInterval);
  });

  function humanizeRelative(ms: number): string {
    if (!ms) return 'never';
    const delta = Math.max(0, Date.now() - ms);
    const secs = Math.floor(delta / 1000);
    if (secs < 60) return `${secs}s ago`;
    const mins = Math.floor(secs / 60);
    if (mins < 60) return `${mins} min ago`;
    const hrs = Math.floor(mins / 60);
    if (hrs < 24) return `${hrs}h ago`;
    const days = Math.floor(hrs / 24);
    if (days < 30) return `${days}d ago`;
    const months = Math.floor(days / 30);
    if (months < 12) return `${months}mo ago`;
    return `${Math.floor(months / 12)}y ago`;
  }

  function truncBody(s: string): string {
    if (s.length <= 200) return s;
    return s.slice(0, 197) + '…';
  }

  function truncCore(s: string): string {
    if (s.length <= 600) return s;
    return s.slice(0, 597) + '…';
  }
</script>

<div class="memory-drawer">
  <div class="header">
    <div class="section-label">CHAT MEMORY (E-1)</div>
    <div class="filters">
      <label>
        scope
        <select bind:value={scopeFilter}>
          <option value="all">all</option>
          {#each SCOPES as s}
            <option value={s}>{s}</option>
          {/each}
        </select>
      </label>
      <button class="refresh-btn" type="button" onclick={reload}>refresh</button>
    </div>
  </div>

  <div class="store-meta">
    store: <span class="mono">{storePath || '(unresolved)'}</span>
    {#if storeJustCreated}
      <span class="ghost">(just created)</span>
    {/if}
  </div>

  <!-- CoreBlock / chat-memory state header -->
  <div class="core-card">
    <div class="core-head">
      <span class="type-tag">CORE BLOCK</span>
      {#if chatState}
        <span class="count-pill">{chatState.core_block_bytes} bytes</span>
        <span class="count-pill">
          notes {chatState.recent_notes_count}
        </span>
        <span class="count-pill">
          reflect {humanizeRelative(chatState.last_reflection_ms ?? 0)}
        </span>
        {#if chatState.sleep_compute_active}
          <span class="count-pill active">SLEEP_COMPUTE</span>
        {/if}
      {/if}
    </div>
    {#if chatStateError}
      <div class="muted-small error">chat-memory state failed: {chatStateError}</div>
    {:else if chatState}
      {#if chatState.core_block_text.length === 0}
        <div class="muted-small">
          No CoreBlock content yet. The reflection loop populates this once
          a user style / preference / opinion signal lands.
        </div>
      {:else}
        <div class="core-body mono">{truncCore(chatState.core_block_text)}</div>
      {/if}
    {/if}
  </div>

  {#if !loaded}
    <div class="muted-small">loading…</div>
  {:else if loadError}
    <div class="muted-small error">load failed: {loadError}</div>
  {:else if notes.length === 0}
    <div class="muted-small">
      No notes yet. The chat-reflect REFLECT stage inserts one row per
      committed turn once the producer wiring lands in
      task_runner.rs.
    </div>
  {:else}
    <div class="rows">
      {#each notes as n}
        <div class="row">
          <div class="row-head">
            <span class="scope-pill {n.scope}">{n.scope}</span>
            <span class="ghost mono">{n.note_id.slice(0, 8)}</span>
          </div>
          <div class="body">{truncBody(n.body)}</div>
          {#if n.tags.length > 0}
            <div class="tags">
              {#each n.tags as t}
                <span class="tag-pill">{t}</span>
              {/each}
            </div>
          {/if}
          <div class="row-meta">
            <span>links <strong>{n.link_count}</strong></span>
            <span>trust <strong>{n.trust_score.toFixed(2)}</strong></span>
            <span>added {humanizeRelative(n.created_at_ms)}</span>
            <span>touched {humanizeRelative(n.last_accessed_ms)}</span>
          </div>
        </div>
      {/each}
    </div>
  {/if}
</div>

<style>
  .memory-drawer {
    display: flex;
    flex-direction: column;
    gap: 12px;
    padding: 12px;
    font-family: var(--font-sans);
    color: var(--fg);
  }
  .header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 12px;
  }
  .section-label {
    font-size: 11px;
    letter-spacing: 0.12em;
    color: var(--fg-muted, #888);
  }
  .filters {
    display: flex;
    align-items: center;
    gap: 8px;
    font-size: 12px;
  }
  .filters select {
    margin-left: 4px;
    background: transparent;
    color: var(--fg);
    border: 1px solid var(--border, #333);
    padding: 2px 4px;
  }
  .store-meta {
    font-size: 11px;
    color: var(--fg-muted, #888);
  }
  .store-meta .ghost {
    margin-left: 6px;
    color: var(--fg-muted, #666);
  }
  .mono {
    font-family: var(--font-mono);
  }
  .muted-small {
    font-size: 12px;
    color: var(--fg-muted, #888);
  }
  .muted-small.error {
    color: var(--danger, #e26d6d);
  }
  .core-card {
    border-top: 1px solid var(--border, #1f1f1f);
    padding-top: 8px;
  }
  .core-head {
    display: flex;
    align-items: center;
    gap: 8px;
    font-size: 12px;
    margin-bottom: 6px;
    flex-wrap: wrap;
  }
  .type-tag {
    font-family: var(--font-mono);
    text-transform: uppercase;
    letter-spacing: 0.08em;
    color: var(--accent, #6dafe2);
  }
  .count-pill {
    padding: 0 6px;
    border: 1px solid var(--border, #333);
    border-radius: 8px;
    font-size: 10px;
    color: var(--fg-muted, #888);
  }
  .count-pill.active {
    color: var(--ok, #6de29c);
    border-color: var(--ok, #6de29c);
  }
  .core-body {
    margin-top: 4px;
    padding: 6px 8px;
    background: var(--bg-elevated, #161616);
    font-size: 11px;
    color: var(--fg-muted, #aaa);
    overflow-wrap: anywhere;
    white-space: pre-wrap;
    max-height: 160px;
    overflow-y: auto;
  }
  .rows {
    display: flex;
    flex-direction: column;
    gap: 6px;
  }
  .row {
    padding: 6px 8px;
    background: var(--bg-elevated, #161616);
    border-left: 2px solid var(--accent, #6dafe2);
  }
  .row-head {
    display: flex;
    justify-content: space-between;
    align-items: center;
    font-size: 12px;
  }
  .scope-pill {
    font-size: 10px;
    text-transform: uppercase;
    padding: 0 6px;
    border-radius: 8px;
    background: var(--bg, #0d0d0d);
    color: var(--fg-muted, #888);
  }
  .scope-pill.global {
    color: var(--accent, #6dafe2);
  }
  .body {
    margin-top: 2px;
    font-size: 11px;
    color: var(--fg-muted, #aaa);
    overflow-wrap: anywhere;
    white-space: pre-wrap;
  }
  .tags {
    margin-top: 4px;
    display: flex;
    flex-wrap: wrap;
    gap: 4px;
  }
  .tag-pill {
    font-size: 10px;
    padding: 0 6px;
    border-radius: 8px;
    background: var(--bg, #0d0d0d);
    color: var(--fg-muted, #888);
  }
  .row-meta {
    margin-top: 4px;
    display: flex;
    flex-wrap: wrap;
    gap: 12px;
    font-size: 10px;
    color: var(--fg-muted, #888);
  }
  .row-meta strong {
    color: var(--fg, #ddd);
  }
  .refresh-btn {
    background: transparent;
    color: var(--fg);
    border: 1px solid var(--border, #333);
    padding: 2px 8px;
    cursor: pointer;
    font-size: 11px;
  }
  .refresh-btn:hover {
    background: var(--bg-elevated, #161616);
  }
  .ghost {
    color: var(--fg-muted, #666);
  }
</style>
