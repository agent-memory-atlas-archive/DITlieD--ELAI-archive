<script lang="ts">
  import { Search } from '@lucide/svelte';
  import { workspace } from '../state/workspace.svelte';
  import { run_explore_query, logStub } from '../api/stub';
  import type { ExploreAnswer } from '../api/stub';

  // Pre-seeded answers
  const SEED_QUERIES = [
    'how does the auth module validate tokens?',
    'what calls Token::validate?',
  ];

  // Answers: newest at bottom (chat-style stream, mirrors CommandWindow)
  let answers = $state<ExploreAnswer[]>([]);
  let recentQueries = $state<string[]>([...SEED_QUERIES]);
  let queryText = $state('');
  let pending = $state(false);
  let inputFocused = $state(false);

  let streamEl = $state<HTMLElement | undefined>(undefined);
  let inputEl = $state<HTMLInputElement | undefined>(undefined);

  // Seed on mount — preserve original order (oldest first → newest last)
  $effect(() => {
    (async () => {
      const results = await Promise.all(SEED_QUERIES.map((q) => run_explore_query(q)));
      answers = results;
    })();
  });

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
      const answer = await run_explore_query(trimmed);
      // Append newest at bottom
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

  // Render code blocks: wrap ``` fenced blocks in <pre><code>
  function renderAnswer(text: string): { html: string } {
    const parts = text.split(/```(?:\w+)?\n?/);
    let html = '';
    parts.forEach((part, i) => {
      if (i % 2 === 0) {
        const escaped = part
          .replace(/&/g, '&amp;')
          .replace(/</g, '&lt;')
          .replace(/>/g, '&gt;')
          .replace(/\*\*(.*?)\*\*/g, '<strong>$1</strong>')
          .replace(/`([^`]+)`/g, '<code class="inline-code">$1</code>')
          .replace(/\n/g, '<br>');
        html += escaped;
      } else {
        const escaped = part
          .replace(/&/g, '&amp;')
          .replace(/</g, '&lt;')
          .replace(/>/g, '&gt;');
        html += `<pre class="code-block"><code>${escaped}</code></pre>`;
      }
    });
    return { html };
  }
</script>

<div class="explore-window">
  <!-- Answers stream — chat-style, anchored to bottom -->
  <div class="stream" bind:this={streamEl}>
    <div class="stream-inner">
      {#if answers.length === 0 && !pending}
        <div class="empty-state">
          <span class="empty-label">ask the oracle</span>
          <span class="empty-sub">read-only queries · grounded in DKG and the knowledge engine</span>
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

            <!-- Floating prose answer -->
            <div class="answer-body">
              {@html renderAnswer(answer.answer).html}
            </div>

            <!-- File refs as ghost-text underlined-on-hover links -->
            <!-- WARN: file_refs always [] until DKG enrichment (B-7 follow-up) -->
            {#if answer.file_refs.length > 0}
              <div class="file-refs">
                {#each answer.file_refs as ref (ref.path + (ref.line ?? ''))}
                  <button
                    class="file-ref-btn mono"
                    onclick={() => handleFileRefClick(ref.path)}
                    title="Open {ref.path} in Review"
                  >
                    {ref.path}{ref.line ? ':' + ref.line : ''}{#if ref.symbol}<span class="file-ref-symbol"> — {ref.symbol}</span>{/if}
                  </button>
                {/each}
              </div>
            {:else if answer.sources_count > 0}
              <div class="file-refs-empty mono">no file references · DKG enrichment pending</div>
            {/if}

            <!-- Citation: dim ghost prose, no border, no chrome -->
            <div class="citation-line mono">
              {#if answer.sources_count > 0}
                from /search · {answer.sources_count} sources
              {:else}
                sidecar offline · no sources
              {/if}
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

  .answer-body :global(.inline-code) {
    font-family: var(--font-mono);
    font-size: 11px;
    color: var(--teal-bright);
    background: rgba(26, 32, 41, 0.6);
    padding: 1px 4px;
    border-radius: var(--r-xs);
  }

  .answer-body :global(.code-block) {
    font-family: var(--font-mono);
    font-size: 11px;
    background: rgba(26, 32, 41, 0.6);
    border: 1px solid var(--border-subtle);
    border-radius: var(--r-sm);
    padding: var(--s-2) var(--s-3);
    margin: var(--s-2) 0;
    overflow-x: auto;
    line-height: 1.5;
    color: var(--text-primary);
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
