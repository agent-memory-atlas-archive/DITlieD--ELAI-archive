<script lang="ts">
  import { Search } from '@lucide/svelte';
  import { workspace } from '../state/workspace.svelte';
  import { query_knowledge_engine, logStub, type KnowledgeAnswer } from '../api/stub';

  // EXPLORE_CODE wire (B-explore-code): rewired from the SearXNG /search
  // endpoint to the local BM25 index of the active project root. Same
  // backend as CommandWindow's auto-detect chip for EXPLORE_CODE intent.
  // Seed queries removed — they hit nothing useful before a project is
  // indexed, and the empty state already explains what to type.

  // Answers: newest at bottom (chat-style stream, mirrors CommandWindow)
  let answers = $state<KnowledgeAnswer[]>([]);
  let recentQueries = $state<string[]>([]);
  let queryText = $state('');
  let pending = $state(false);
  let inputFocused = $state(false);

  let streamEl = $state<HTMLElement | undefined>(undefined);
  let inputEl = $state<HTMLInputElement | undefined>(undefined);

  // Auto-scroll to bottom whenever a new answer lands
  $effect(() => {
    const _ = answers.length;
    if (!streamEl) return;
    requestAnimationFrame(() => {
      requestAnimationFrame(() => {
        if (streamEl) streamEl.scrollTop = streamEl.scrollHeight;
      });
    });
  });

  async function runQuery(q: string) {
    const trimmed = q.trim();
    if (!trimmed || pending) return;
    pending = true;
    queryText = '';

    // Track recent queries (dedup, newest first, max 5)
    recentQueries = [trimmed, ...recentQueries.filter((r) => r !== trimmed)].slice(0, 5);

    try {
      const answer = await query_knowledge_engine(trimmed);
      answers = [...answers, answer];
    } finally {
      pending = false;
    }
  }

  function handleQueryKeydown(e: KeyboardEvent) {
    if (e.key === 'Enter' && !e.shiftKey) {
      e.preventDefault();
      runQuery(queryText);
    }
  }

  function handleFileRefClick(path: string) {
    workspace.openTab('primary', 'review');
    logStub('open-file', path);
  }
</script>

<div class="explore-window">
  <!-- Answers stream — chat-style, anchored to bottom -->
  <div class="stream" bind:this={streamEl}>
    <div class="stream-inner">
      {#if answers.length === 0 && !pending}
        <div class="empty-state">
          <span class="empty-label">explore the codebase</span>
          <span class="empty-sub">read-only BM25 queries over the active project's local index</span>
          {#if recentQueries.length > 0}
            <div class="empty-hints">
              {#each recentQueries.slice(0, 3) as q (q)}
                <button class="mono-ghost-btn" onclick={() => runQuery(q)}>{q}</button>
              {/each}
            </div>
          {/if}
        </div>
      {:else}
        {#each answers as answer (answer.id)}
          <div class="answer-wrapper anim-enter-lift">
            <!-- Query echo: mono ghost arrow + question text -->
            <div class="answer-query">
              <span class="answer-query-arrow">&gt;</span>
              <span class="answer-query-text">{answer.query}</span>
            </div>

            {#if answer.status === 'not_indexed'}
              <div class="answer-body answer-hint mono">project not indexed — reopen it from the picker to trigger indexing</div>
            {:else if answer.status === 'error'}
              <div class="answer-body answer-hint mono">{answer.message ?? 'knowledge query failed'}</div>
            {:else if answer.hits.length === 0}
              <div class="answer-body answer-hint mono">no matches for "{answer.query}"</div>
            {:else}
              <ul class="hit-list">
                {#each answer.hits as h, i (h.path + ':' + h.line_start + ':' + i)}
                  <li class="hit-row">
                    <button
                      class="file-ref-btn mono"
                      onclick={() => handleFileRefClick(h.path)}
                      title="Open {h.path} in Review"
                    >
                      {h.path}{h.line_start ? ':' + h.line_start : ''}{h.line_end && h.line_end !== h.line_start ? '-' + h.line_end : ''}
                      <span class="file-ref-symbol"> — {h.score.toFixed(2)} · {h.chunk_kind.toLowerCase()}</span>
                    </button>
                    {#if h.snippet}
                      <pre class="hit-snippet mono">{h.snippet}</pre>
                    {/if}
                  </li>
                {/each}
              </ul>
            {/if}

            <!-- Citation: dim ghost prose -->
            <div class="citation-line mono">
              {answer.hits.length} hit{answer.hits.length === 1 ? '' : 's'} · BM25 (vector + reranker degraded — models not staged)
            </div>
          </div>
        {/each}
      {/if}
    </div>
  </div>

  <!-- Pending breath line — single 1px teal stroke, only when querying -->
  {#if pending}
    <div class="pending-line" aria-label="querying oracle"></div>
  {/if}

  <!-- Input bar at bottom — mirrors CommandWindow's outlined chat shell -->
  <div class="input-bar">
    <div class="input-shell" class:input-shell--focused={inputFocused}>
      <div class="input-bar-line" aria-hidden="true"></div>
      <div class="input-row">
        <span class="input-icon" aria-hidden="true">
          <Search size={14} strokeWidth={1.5} />
        </span>
        <input
          bind:this={inputEl}
          class="input-field mono"
          type="text"
          bind:value={queryText}
          placeholder="ask the codebase a question"
          onkeydown={handleQueryKeydown}
          onfocus={() => (inputFocused = true)}
          onblur={() => (inputFocused = false)}
          disabled={pending}
        />
        {#if queryText.trim().length > 0}
          <button class="send-btn" aria-label="Run query" onclick={() => runQuery(queryText)}>
            run
          </button>
        {/if}
      </div>
    </div>
  </div>
</div>

<style>
  /* ================================================================
     ExploreWindow — mirrors CommandWindow's orbit aesthetic.
     Floating prose answers; outlined input shell; no card chrome.
  ================================================================ */

  .explore-window {
    display: flex;
    flex-direction: column;
    height: 100%;
    overflow: hidden;
    background: var(--bg-void);
    position: relative;
  }

  /* ---- Stream ---- */
  .stream {
    flex: 1 1 auto;
    overflow-y: auto;
    display: flex;
    flex-direction: column;
    scrollbar-width: thin;
    scrollbar-color: var(--border-default) transparent;
    position: relative;
    z-index: 1;
  }

  .stream-inner {
    margin-top: auto;
    display: flex;
    flex-direction: column;
    gap: var(--s-6);
    padding: var(--s-4) var(--s-6) var(--s-3);
  }

  .stream::-webkit-scrollbar { width: 4px; }
  .stream::-webkit-scrollbar-track { background: transparent; }
  .stream::-webkit-scrollbar-thumb { background: var(--border-default); border-radius: var(--r-pill); }
  .stream::-webkit-scrollbar-thumb:hover { background: var(--border-strong); }

  /* ---- Empty state ---- */
  .empty-state {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: var(--s-2);
    padding-top: var(--s-10);
  }

  .empty-label {
    font-family: var(--font-sans);
    font-size: 14px;
    color: var(--text-secondary);
  }

  .empty-sub {
    font-family: var(--font-sans);
    font-size: 12px;
    color: var(--text-ghost);
    line-height: 1.5;
    text-align: center;
    max-width: 380px;
  }

  .empty-hints {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: var(--s-1);
    margin-top: var(--s-3);
  }

  .mono-ghost-btn {
    font-family: var(--font-mono);
    font-size: 12px;
    color: var(--text-ghost);
    background: none;
    border: none;
    cursor: pointer;
    padding: 0;
    outline: none;
    transition: color var(--dur-fast) var(--ease-std);
  }
  .mono-ghost-btn:hover { color: var(--teal); }
  .mono-ghost-btn:focus-visible { box-shadow: var(--ring); border-radius: var(--r-xs); }

  /* ---- Answer wrapper — no card chrome, just spaced prose ---- */
  .answer-wrapper {
    display: flex;
    flex-direction: column;
    gap: var(--s-2);
  }

  .answer-query {
    display: flex;
    align-items: baseline;
    gap: var(--s-2);
  }

  .answer-query-arrow {
    font-family: var(--font-mono);
    font-size: 13px;
    color: var(--teal);
    line-height: 1;
    flex-shrink: 0;
  }

  .answer-query-text {
    font-family: var(--font-mono);
    font-size: 12px;
    color: var(--text-secondary);
    line-height: 1.5;
  }

  .answer-body {
    font-family: var(--font-sans);
    font-size: 13px;
    color: var(--text-primary);
    line-height: 1.6;
  }

  /* Empty / not-indexed / error inline message inside the answer area */
  .answer-hint {
    color: var(--text-ghost);
    font-style: italic;
    font-size: 12px;
  }

  /* ---- Hit list (BM25 results) ---- */
  .hit-list {
    list-style: none;
    margin: 0;
    padding: 0;
    display: flex;
    flex-direction: column;
    gap: var(--s-2);
  }
  .hit-row {
    display: flex;
    flex-direction: column;
    gap: 4px;
    border-left: 2px solid var(--border-subtle);
    padding-left: var(--s-2);
    transition: border-color var(--dur-fast) var(--ease-std);
  }
  .hit-row:hover { border-left-color: var(--teal-line); }
  .hit-snippet {
    font-size: 11px;
    color: var(--text-secondary);
    background: rgba(26, 32, 41, 0.45);
    border-radius: var(--r-xs);
    padding: var(--s-1) var(--s-2);
    margin: 0;
    white-space: pre-wrap;
    word-break: break-word;
    line-height: 1.45;
    max-height: 10em;
    overflow: hidden;
  }

  /* ---- File refs — ghost text by default; teal underline on hover ---- */
  .file-refs {
    display: flex;
    flex-direction: column;
    gap: 2px;
  }

  .file-ref-btn {
    display: inline-block;
    background: none;
    border: none;
    cursor: pointer;
    font-size: 11px;
    color: var(--text-ghost);
    padding: 1px 0;
    outline: none;
    text-align: left;
    transition: color var(--dur-fast) var(--ease-std);
  }

  .file-ref-btn:hover {
    color: var(--teal);
    text-decoration: underline;
    text-underline-offset: 2px;
  }

  .file-ref-btn:focus-visible {
    box-shadow: var(--ring);
    border-radius: var(--r-xs);
  }

  .file-ref-symbol {
    color: var(--text-disabled);
    font-size: 10px;
  }

  /* ---- File refs empty state ---- */
  .file-refs-empty {
    font-size: 11px;
    color: var(--text-disabled);
    opacity: 0.6;
    line-height: 1.5;
  }

  /* ---- Citation line — dim ghost prose ---- */
  .citation-line {
    font-size: 11px;
    color: var(--text-ghost);
    opacity: 0.55;
    line-height: 1.5;
  }

  /* ---- Pending breath line — 1px teal stroke ---- */
  .pending-line {
    height: 1px;
    background: var(--teal);
    flex-shrink: 0;
    animation: breathLine 2.2s ease-in-out infinite;
  }

  @keyframes breathLine {
    0%, 100% { opacity: 0.2; }
    50%       { opacity: 0.55; }
  }

  /* ---- Input bar — outlined chat shell mirrored from CommandWindow ---- */
  .input-bar {
    flex-shrink: 0;
    position: relative;
    background: transparent;
    z-index: 2;
  }

  .input-bar-line {
    height: 1px;
    background: var(--teal-line);
    opacity: 0;
    pointer-events: none;
    transition: opacity 180ms var(--ease-std);
  }
  .input-bar:focus-within .input-bar-line { opacity: 0.45; }

  .input-shell {
    position: relative;
    margin: var(--s-3) var(--s-4) var(--s-3);
    background: var(--bg-input);
    border: 1px solid var(--border-default);
    border-radius: var(--r-md);
    padding: var(--s-1) var(--s-2);
    transition:
      border-color var(--dur-fast) var(--ease-std),
      background var(--dur-fast) var(--ease-std);
  }
  .input-shell--focused {
    border-color: var(--teal-line);
    background: rgba(26, 32, 41, 0.85);
  }

  .input-row {
    display: flex;
    align-items: center;
    gap: var(--s-2);
    padding: 2px 8px;
  }

  .input-icon {
    display: flex;
    align-items: center;
    color: var(--text-ghost);
    flex-shrink: 0;
  }

  .input-field {
    flex: 1 1 auto;
    background: transparent;
    border: none;
    color: var(--text-primary);
    font-size: 12px;
    line-height: 1.4;
    padding: 4px 0;
    outline: none;
    border-bottom: 1px solid transparent;
    transition: border-color var(--dur-fast) var(--ease-std);
  }
  .input-field::placeholder { color: var(--text-ghost); }
  .input-field:focus { border-bottom-color: var(--teal); }
  .input-field:disabled { opacity: 0.5; cursor: not-allowed; }

  /* Send: outlined teal — never solid fill */
  .send-btn {
    font-family: var(--font-sans);
    font-size: 11px;
    color: var(--teal);
    background: transparent;
    border: 1px solid var(--teal-line);
    border-radius: var(--r-sm);
    padding: 3px 10px;
    cursor: pointer;
    outline: none;
    flex-shrink: 0;
    transition:
      background var(--dur-fast) var(--ease-std),
      color var(--dur-fast) var(--ease-std),
      border-color var(--dur-fast) var(--ease-std);
  }
  .send-btn:hover {
    background: var(--teal-ghost);
    border-color: var(--teal);
    color: var(--teal-bright);
  }
  .send-btn:focus-visible { box-shadow: var(--ring); }
</style>
