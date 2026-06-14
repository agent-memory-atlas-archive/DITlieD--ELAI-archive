<script lang="ts">
  // ============================================================
  // ExploreWindow — Q&A oracle surface (C-7)
  // ============================================================
  // Submitting a query appends a placeholder oracle response row. The
  // backend `get_codebase_search` Tauri command is a deferred C-3+
  // deliverable; the UI is wired end-to-end so the surface is usable
  // (and visible) the moment the live command lands.
  //
  // No mock data is pre-rendered — the response list starts empty and
  // shows the Empty state. Each submit produces one user-row + one
  // oracle-placeholder row so the threading is observable without a
  // backend.

  import { Button, Empty, Scrollbar } from '$lib/ui';
  import * as stub from '$lib/api/stub';
  import SkeletonList from '$lib/components/SkeletonList.svelte';
  // KNOWLEDGE tab: the notes/ideas "brain" canvas (LLM drafts, user connects,
  // research accrues). Lives alongside the Q&A oracle behind a view toggle.
  import KnowledgeBrain from '$lib/canvas/KnowledgeBrain.svelte';

  // Two views on the KNOWLEDGE surface: the notes BRAIN (default — the
  // curatable concept graph) and the Q&A oracle (ask the codebase).
  type KnowledgeView = 'brain' | 'qa';
  let view = $state<KnowledgeView>('brain');

  interface OracleRow {
    id: string;
    kind: 'query' | 'response';
    text: string;
    ts: number;
  }

  let inputValue = $state<string>('');
  let rows = $state<OracleRow[]>([]);
  // E-33: transient loading flag covers the synthesizing window between
  // submit() append and the next paint. Visible as a one-frame SkeletonList
  // shimmer; once the C-3+ get_codebase_search backend lands, this flag
  // will gate on the real async fetch instead of a queueMicrotask.
  let loading = $state<boolean>(false);

  function nextId(): string {
    if (typeof crypto !== 'undefined' && typeof crypto.randomUUID === 'function') {
      return crypto.randomUUID();
    }
    return `${Date.now().toString(36)}-${Math.random().toString(36).slice(2, 10)}`;
  }

  function submit() {
    const trimmed = inputValue.trim();
    if (trimmed.length === 0) return;
    const ts = Date.now();
    const queryRow: OracleRow = { id: nextId(), kind: 'query', text: trimmed, ts };
    // C-7 DRIFT-note: backend codebase-search command deferred to C-3+
    // (no get_codebase_search Tauri command shipped yet). We log the stub
    // call so the harness logStub stream still shows the user intent,
    // and render an explicit placeholder so the threading is observable.
    stub.logStub('explore.query', { query: trimmed });
    // E-33: flip loading=true for one microtask so SkeletonList renders
    // a single shimmer frame between query append and placeholder reply.
    rows = [...rows, queryRow];
    inputValue = '';
    loading = true;
    queueMicrotask(() => {
      const placeholder: OracleRow = {
        id: nextId(),
        kind: 'response',
        text:
          'Codebase oracle is offline. The live search backend lands in a follow-up patch (get_codebase_search Tauri command). Your query has been logged.',
        ts: ts + 1,
      };
      rows = [...rows, placeholder];
      loading = false;
    });
  }

  function handleKeydown(e: KeyboardEvent) {
    // Ctrl+Enter and bare Enter both submit; Shift+Enter inserts a newline
    // (textarea default). This mirrors CommandWindow's submit ergonomics.
    if (e.key === 'Enter' && !e.shiftKey) {
      e.preventDefault();
      submit();
    }
  }

  function fmtTs(ts: number): string {
    const d = new Date(ts);
    const hh = String(d.getHours()).padStart(2, '0');
    const mm = String(d.getMinutes()).padStart(2, '0');
    const ss = String(d.getSeconds()).padStart(2, '0');
    return `${hh}:${mm}:${ss}`;
  }
</script>

<div class="explore" data-testid="explore-window">
  <div class="explore__header">
    <span class="explore__title">knowledge</span>
    <!-- view toggle — center->out underline idiom (DESIGN §13) -->
    <div class="explore__views" role="tablist" aria-label="Knowledge view">
      <button class="vbtn" class:on={view === 'brain'} role="tab" aria-selected={view === 'brain'} onclick={() => { view = 'brain'; }}>
        BRAIN<span class="vu" aria-hidden="true"></span>
      </button>
      <button class="vbtn" class:on={view === 'qa'} role="tab" aria-selected={view === 'qa'} onclick={() => { view = 'qa'; }}>
        Q&amp;A<span class="vu" aria-hidden="true"></span>
      </button>
    </div>
    {#if view === 'qa'}
      <span class="explore__hint">ask the codebase oracle — Enter to submit</span>
    {/if}
  </div>

  {#if view === 'brain'}
    <KnowledgeBrain />
  {:else}
  <div class="explore__body">
    {#if rows.length === 0 && !loading}
      <div class="explore__empty">
        <Empty
          title="no queries yet"
          subtitle="ask anything about the codebase — symbols, dependencies, history"
        />
      </div>
    {:else}
      <SkeletonList rows={2} loading={loading} label="oracle thinking" />
      <Scrollbar>
        <div class="explore__rows">
          {#each rows as row (row.id)}
            <div class="explore-row explore-row--{row.kind}">
              <div class="explore-row__meta">
                <span class="explore-row__kind">{row.kind === 'query' ? 'you' : 'oracle'}</span>
                <span class="explore-row__ts">{fmtTs(row.ts)}</span>
              </div>
              <div class="explore-row__text">{row.text}</div>
            </div>
          {/each}
        </div>
      </Scrollbar>
    {/if}
  </div>

  <div class="explore__composer">
    <textarea
      class="explore__input mono"
      placeholder="ask the codebase oracle…"
      rows="2"
      bind:value={inputValue}
      onkeydown={handleKeydown}
      data-testid="explore-input"
    ></textarea>
    <Button
      variant="primary"
      size="sm"
      onclick={submit}
      disabled={inputValue.trim().length === 0}
    >
      Ask
    </Button>
  </div>
  {/if}
</div>

<style>
  .explore {
    display: flex;
    flex-direction: column;
    height: 100%;
    background: var(--bg);
    overflow: hidden;
  }

  .explore__header {
    display: flex;
    align-items: center;
    gap: 14px;
    padding: var(--s-2) var(--s-3);
    background: var(--bg);
    border-bottom: 1px solid var(--line);
    flex-shrink: 0;
  }

  .explore__title {
    font-family: var(--font-mono);
    font-size: 10px;
    color: var(--text);
    text-transform: uppercase;
    letter-spacing: 0.16em;
  }

  /* view toggle — center->out underline idiom */
  .explore__views { display: flex; gap: 2px; }
  .vbtn {
    position: relative;
    padding: 4px 9px 6px;
    font-family: var(--font-mono);
    font-size: 9px;
    letter-spacing: 0.14em;
    background: none;
    border: none;
    color: var(--dim);
    cursor: pointer;
    transition: color var(--dur-fast) var(--ease-out);
  }
  .vbtn:hover:not(.on) { color: var(--text); }
  .vbtn.on { color: var(--d-teal-bright); }
  .vu {
    position: absolute;
    left: 7px; right: 7px; bottom: 1px;
    height: 1.5px;
    background: var(--d-teal-bright);
    transform: scaleX(0);
    transform-origin: center;
    transition: transform var(--dur-base) var(--ease-out);
  }
  .vbtn:hover .vu, .vbtn.on .vu { transform: scaleX(1); }

  .explore__hint {
    font-family: var(--font-mono);
    font-size: 10px;
    color: var(--dim);
    margin-left: auto;
  }

  .explore__body {
    flex: 1;
    overflow: hidden;
    min-height: 0;
  }

  .explore__empty {
    height: 100%;
    display: flex;
    align-items: center;
    justify-content: center;
  }

  .explore__rows {
    display: flex;
    flex-direction: column;
    gap: var(--s-2);
    padding: var(--s-3);
  }

  .explore-row {
    display: flex;
    flex-direction: column;
    gap: 4px;
    padding: var(--s-2);
    border-radius: var(--r-sm);
    border: 1px solid var(--line);
  }

  .explore-row--query {
    background: var(--panel);
    border-left: 2px solid var(--d-teal-soft);
  }

  .explore-row--response {
    background: var(--row);
    border-left: 2px solid var(--line2);
  }

  .explore-row__meta {
    display: flex;
    gap: var(--s-2);
    align-items: baseline;
  }

  .explore-row__kind {
    font-family: var(--font-mono);
    font-size: 10px;
    color: var(--d-teal-bright);
    text-transform: uppercase;
    letter-spacing: 0.1em;
  }

  .explore-row__ts {
    font-family: var(--font-mono);
    font-size: 10px;
    color: var(--dim);
  }

  .explore-row__text {
    font-family: var(--font-mono);
    font-size: 12px;
    color: var(--read);
    white-space: pre-wrap;
    word-break: break-word;
  }

  .explore__composer {
    display: flex;
    gap: var(--s-2);
    padding: var(--s-2) var(--s-3);
    background: var(--bg);
    border-top: 1px solid var(--line);
    align-items: flex-end;
    flex-shrink: 0;
  }

  .explore__input {
    flex: 1;
    min-height: 32px;
    max-height: 96px;
    background: var(--panel);
    border: 1px solid var(--line);
    border-radius: var(--r-sm);
    color: var(--text);
    font-family: var(--font-mono);
    font-size: 12px;
    padding: 6px 8px;
    resize: vertical;
    outline: none;
    transition: border-color var(--dur-fast) var(--ease-out);
  }

  .explore__input:focus {
    border-color: var(--d-teal-soft);
  }

  .explore__input::placeholder {
    color: var(--dim);
  }

  @media (prefers-reduced-motion: reduce) {
    .vu { transition: none; }
  }
</style>
