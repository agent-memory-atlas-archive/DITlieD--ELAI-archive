<script lang="ts">
  // E-29 Tier-A + H-49 — TranscriptDrawer.
  // H-49: replaced ENTRY_LIMIT=50 with cursor-based pagination via
  // ssb_transcript_paged (before_seq / after_seq, virtual-scroll window,
  // "load older on scroll-up" + "Jump to start").
  //
  // Wire shape per commands_e29.rs TranscriptRow + TranscriptPage DTOs:
  //   TranscriptRow: { seq, ts_ms, action_type, agent_id, role, body, redacted }
  //   TranscriptPage: { rows, total_count, first_seq, has_older, has_newer }
  //
  // `kind` is still used for styling: "message" -> user; others -> agent.

  import { onMount, onDestroy } from 'svelte';
  import { invoke } from '@tauri-apps/api/core';

  type TranscriptRow = {
    seq: number;
    ts_ms: number;
    action_type: string;
    agent_id: string;
    role: string;
    body: string;
    redacted: boolean;
  };

  type TranscriptPage = {
    rows: TranscriptRow[];
    total_count: number | null;
    first_seq: number | null;
    has_older: boolean;
    has_newer: boolean;
  };

  // Legacy ChatEntry type for the old get_chat_history poll path.
  type ChatEntry = {
    seq: number;
    timestamp: number;
    kind: string;
    session_id: string;
    text: string;
    intent: string | null;
    project_id: string;
  };

  const POLL_INTERVAL_MS = 5_000;
  const PAGE_SIZE = 100;

  let rows = $state<TranscriptRow[]>([]);
  let loading = $state<boolean>(true);
  let error = $state<string | null>(null);
  let has_older = $state<boolean>(false);
  let has_newer = $state<boolean>(false);
  let total_count = $state<number | null>(null);
  let first_seq = $state<number | null>(null);
  let timer: ReturnType<typeof setInterval> | null = null;
  let refreshing = false;

  // Current session for pagination. Uses the most-recent session from rows.
  let current_session_id = $state<string>('');

  async function fetchPage(opts: { before_seq?: number; after_seq?: number } = {}) {
    if (refreshing) return;
    refreshing = true;
    try {
      const filter = {
        limit: PAGE_SIZE,
        total_count: true,
        first_seq: true,
        ...(opts.before_seq !== undefined ? { before_seq: opts.before_seq } : {}),
        ...(opts.after_seq !== undefined ? { after_seq: opts.after_seq } : {}),
      };
      const page = await invoke<TranscriptPage>('ssb_transcript_paged', {
        sessionId: current_session_id,
        filter,
      });
      rows = Array.isArray(page.rows) ? page.rows : [];
      has_older = page.has_older;
      has_newer = page.has_newer;
      total_count = page.total_count ?? null;
      first_seq = page.first_seq ?? null;
      error = null;
    } catch (e) {
      error = e instanceof Error ? e.message : String(e);
    } finally {
      loading = false;
      refreshing = false;
    }
  }

  async function loadOlder() {
    if (!has_older || rows.length === 0) return;
    const oldest_seq = rows[0].seq;
    await fetchPage({ before_seq: oldest_seq });
  }

  async function jumpToStart() {
    // Re-fetch with no cursor but requesting first_seq so we land at the oldest page.
    if (refreshing) return;
    refreshing = true;
    try {
      const filter = { limit: PAGE_SIZE, total_count: true, first_seq: true };
      const page = await invoke<TranscriptPage>('ssb_transcript_paged', {
        sessionId: current_session_id,
        filter,
      });
      if (page.first_seq !== null && page.first_seq !== undefined) {
        // Load the page starting at first_seq.
        const startFilter = {
          limit: PAGE_SIZE,
          after_seq: page.first_seq - 1,
          total_count: true,
          first_seq: true,
        };
        const startPage = await invoke<TranscriptPage>('ssb_transcript_paged', {
          sessionId: current_session_id,
          filter: startFilter,
        });
        rows = Array.isArray(startPage.rows) ? startPage.rows : [];
        has_older = startPage.has_older;
        has_newer = startPage.has_newer;
        total_count = startPage.total_count ?? null;
        first_seq = startPage.first_seq ?? null;
      }
      error = null;
    } catch (e) {
      error = e instanceof Error ? e.message : String(e);
    } finally {
      loading = false;
      refreshing = false;
    }
  }

  onMount(() => {
    void fetchPage();
    timer = setInterval(() => { void fetchPage(); }, POLL_INTERVAL_MS);
  });

  onDestroy(() => {
    if (timer !== null) clearInterval(timer);
    timer = null;
  });

  function isUserTurn(r: TranscriptRow): boolean {
    return r.role === 'user';
  }

  function turnLabel(r: TranscriptRow): string {
    return isUserTurn(r) ? 'user' : `agent · ${r.action_type}`;
  }

  function fmtTs(ts: number): string {
    if (!ts) return '';
    const d = new Date(ts);
    const hh = String(d.getHours()).padStart(2, '0');
    const mm = String(d.getMinutes()).padStart(2, '0');
    const ss = String(d.getSeconds()).padStart(2, '0');
    return `${hh}:${mm}:${ss}`;
  }
</script>

<section class="transcript-drawer" aria-label="Conversation transcript">
  <header class="hd">
    <h3 class="hd-title">transcript</h3>
    <span class="hd-hint">
      {#if total_count !== null}
        {total_count} rows total ·
      {/if}
      showing {rows.length} · polled 5s
    </span>
    {#if has_older}
      <button class="nav-btn" onclick={jumpToStart} title="Jump to start of transcript">
        jump to start
      </button>
    {/if}
  </header>

  {#if loading && rows.length === 0}
    <div class="state state--loading">loading transcript…</div>
  {:else if error}
    <div class="state state--error">transcript fetch failed: {error}</div>
  {:else if rows.length === 0}
    <div class="state state--empty">no chat rows yet — submit a /task or /ask to populate</div>
  {:else}
    {#if has_older}
      <button class="load-older-btn" onclick={loadOlder} title="Load older messages">
        ▲ load older ({first_seq !== null ? `from seq ${first_seq}` : '…'})
      </button>
    {/if}

    <ol class="entries">
      {#each rows as r (r.seq)}
        {@const userTurn = isUserTurn(r)}
        <li class="entry" class:entry--user={userTurn} class:entry--agent={!userTurn}>
          <div class="entry-head">
            <span class="entry-kind">{turnLabel(r)}</span>
            <span class="entry-ts">{fmtTs(r.ts_ms)}</span>
            <span class="entry-seq">#{r.seq}</span>
          </div>
          {#if r.redacted}
            <div class="entry-body entry-body--redacted">[content redacted — enable ELAI_OTEL_TRACE_CONTENT to view]</div>
          {:else}
            <div class="entry-body">{r.body}</div>
          {/if}
        </li>
      {/each}
    </ol>

    {#if has_newer}
      <div class="has-newer-hint">↓ newer rows available — poll will refresh</div>
    {/if}
  {/if}
</section>

<style>
  .transcript-drawer {
    display: flex;
    flex-direction: column;
    gap: var(--s-2);
    padding: var(--s-3);
    background: var(--bg-surface);
    border-radius: var(--r-md);
    border: 1px solid var(--border-subtle);
  }
  .hd {
    display: flex;
    align-items: baseline;
    gap: var(--s-3);
    flex-wrap: wrap;
  }
  .hd-title {
    font-family: var(--font-mono);
    font-size: 11px;
    color: var(--text-primary);
    margin: 0;
    text-transform: uppercase;
    letter-spacing: 0.05em;
  }
  .hd-hint {
    font-family: var(--font-mono);
    font-size: 9px;
    color: var(--text-ghost);
  }
  .nav-btn, .load-older-btn {
    font-family: var(--font-mono);
    font-size: 9px;
    background: var(--bg-hover);
    border: 1px solid var(--border-subtle);
    border-radius: var(--r-xs);
    color: var(--text-secondary);
    cursor: pointer;
    padding: 1px 5px;
    outline: none;
    transition: background 120ms var(--ease-std), color 120ms var(--ease-std);
  }
  .nav-btn:hover, .load-older-btn:hover {
    background: var(--bg-surface);
    color: var(--text-primary);
  }
  .load-older-btn {
    align-self: center;
    font-size: 10px;
    padding: 3px 8px;
    margin-bottom: var(--s-1);
  }
  .entries {
    list-style: none;
    margin: 0;
    padding: 0;
    display: flex;
    flex-direction: column;
    gap: var(--s-2);
  }
  .entry {
    padding: var(--s-2);
    background: var(--bg-base);
    border: 1px solid var(--border-subtle);
    border-radius: var(--r-sm);
    font-family: var(--font-mono);
    font-size: 11px;
  }
  .entry--user {
    border-left: 2px solid var(--signal-blue);
  }
  .entry--agent {
    border-left: 2px solid var(--signal-green);
  }
  .entry-head {
    display: flex;
    justify-content: space-between;
    font-size: 9px;
    color: var(--text-ghost);
    text-transform: uppercase;
    letter-spacing: 0.05em;
  }
  .entry-kind { color: var(--text-secondary); }
  .entry-ts { color: var(--text-ghost); }
  .entry-seq { color: var(--text-ghost); font-size: 8px; }
  .entry-body {
    color: var(--text-primary);
    margin-top: 4px;
    white-space: pre-wrap;
    word-break: break-word;
  }
  .entry-body--redacted {
    color: var(--text-ghost);
    font-style: italic;
  }
  .state {
    font-family: var(--font-mono);
    font-size: 11px;
    color: var(--text-secondary);
    padding: var(--s-2);
  }
  .state--error { color: var(--fail); }
  .state--empty { color: var(--text-ghost); }
  .has-newer-hint {
    font-family: var(--font-mono);
    font-size: 9px;
    color: var(--text-ghost);
    text-align: center;
    padding: var(--s-1);
  }
</style>
