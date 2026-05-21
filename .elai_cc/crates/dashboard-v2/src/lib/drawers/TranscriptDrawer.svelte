<script lang="ts">
  // E-29 Tier-A — TranscriptDrawer.
  // Joins user-turn ChatMessage rows with agent-turn ChatResponse rows from
  // the SSB via the existing `get_chat_history` Tauri command (commands.rs:4479).
  // Wire shape per ssb_viewer.rs:209-217 ChatEntry struct:
  //   { seq, timestamp, kind, session_id, text, intent?, project_id }
  //
  // `kind` is "message" for user-submitted chat_message rows or one of the
  // chat_response card kinds ("status" | "error" | "disambiguation" |
  // "steering_ack") for agent-emitted rows (ssb_viewer.rs:202-207). Sorted by
  // seq (chronological) by the viewer. The natural "transcript" surface for
  // OTel GenAI semconv invoke_workflow spans — reusing the existing chat
  // surface means zero new backend producers for Tier-A.

  import { onMount, onDestroy } from 'svelte';
  import { invoke } from '@tauri-apps/api/core';

  type ChatEntry = {
    seq: number;
    timestamp: number; // ms since epoch
    kind: string; // "message" | card_kind variants
    session_id: string;
    text: string;
    intent: string | null;
    project_id: string;
  };

  const POLL_INTERVAL_MS = 5_000;
  const ENTRY_LIMIT = 50;

  let entries = $state<ChatEntry[]>([]);
  let loading = $state<boolean>(true);
  let error = $state<string | null>(null);
  let timer: ReturnType<typeof setInterval> | null = null;
  let refreshing = false;

  async function refresh() {
    if (refreshing) return; // dedupe overlapping polls
    refreshing = true;
    try {
      const live = await invoke<ChatEntry[]>('get_chat_history', { limit: ENTRY_LIMIT });
      entries = Array.isArray(live) ? live : [];
      error = null;
    } catch (e) {
      error = e instanceof Error ? e.message : String(e);
    } finally {
      loading = false;
      refreshing = false;
    }
  }

  onMount(() => {
    void refresh();
    timer = setInterval(() => { void refresh(); }, POLL_INTERVAL_MS);
  });

  onDestroy(() => {
    if (timer !== null) clearInterval(timer);
    timer = null;
  });

  // Per ssb_viewer.rs:204-207, kind == "message" -> user turn; everything
  // else (status / error / disambiguation / steering_ack) is an agent card.
  function isUserTurn(e: ChatEntry): boolean {
    return e.kind === 'message';
  }

  function turnLabel(e: ChatEntry): string {
    return isUserTurn(e) ? 'user' : `agent · ${e.kind}`;
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
    <span class="hd-hint">chat_message + chat_response SSB · last {ENTRY_LIMIT} · polled 5s</span>
  </header>

  {#if loading && entries.length === 0}
    <div class="state state--loading">loading transcript…</div>
  {:else if error}
    <div class="state state--error">transcript fetch failed: {error}</div>
  {:else if entries.length === 0}
    <div class="state state--empty">no chat rows yet — submit a /task or /ask to populate</div>
  {:else}
    <ol class="entries">
      {#each entries as e (e.seq)}
        {@const userTurn = isUserTurn(e)}
        <li class="entry" class:entry--user={userTurn} class:entry--agent={!userTurn}>
          <div class="entry-head">
            <span class="entry-kind">{turnLabel(e)}</span>
            <span class="entry-ts">{fmtTs(e.timestamp)}</span>
          </div>
          <div class="entry-body">{e.text}</div>
          {#if e.intent}
            <div class="entry-meta">intent: {e.intent}</div>
          {/if}
        </li>
      {/each}
    </ol>
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
  .entry-body {
    color: var(--text-primary);
    margin-top: 4px;
    white-space: pre-wrap;
    word-break: break-word;
  }
  .entry-meta {
    font-size: 9px;
    color: var(--text-ghost);
    margin-top: 4px;
  }
  .state {
    font-family: var(--font-mono);
    font-size: 11px;
    color: var(--text-secondary);
    padding: var(--s-2);
  }
  .state--error { color: var(--fail); }
  .state--empty { color: var(--text-ghost); }
</style>
