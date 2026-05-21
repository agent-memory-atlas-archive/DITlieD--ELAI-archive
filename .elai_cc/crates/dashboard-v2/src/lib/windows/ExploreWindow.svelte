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
    <span class="explore__title">explore</span>
    <span class="explore__hint">ask the codebase oracle — Enter to submit</span>
  </div>

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
</div>

<style>
  .explore {
    display: flex;
    flex-direction: column;
    height: 100%;
    background: var(--bg-base);
    overflow: hidden;
  }

  .explore__header {
    display: flex;
    align-items: baseline;
    gap: var(--s-3);
    padding: var(--s-2) var(--s-3);
    background: var(--bg-surface);
    border-bottom: 1px solid var(--border-subtle);
    flex-shrink: 0;
  }

  .explore__title {
    font-family: var(--font-mono);
    font-size: 12px;
    color: var(--teal);
    text-transform: uppercase;
    letter-spacing: 0.15em;
  }

  .explore__hint {
    font-family: var(--font-mono);
    font-size: 10px;
    color: var(--text-ghost);
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
    border-radius: var(--r-xs);
    border: 1px solid var(--border-subtle);
  }

  .explore-row--query {
    background: var(--bg-input);
    border-left: 2px solid var(--teal-line);
  }

  .explore-row--response {
    background: var(--bg-surface);
    border-left: 2px solid var(--border-subtle);
  }

  .explore-row__meta {
    display: flex;
    gap: var(--s-2);
    align-items: baseline;
  }

  .explore-row__kind {
    font-family: var(--font-mono);
    font-size: 10px;
    color: var(--teal);
    text-transform: uppercase;
    letter-spacing: 0.1em;
  }

  .explore-row__ts {
    font-family: var(--font-mono);
    font-size: 10px;
    color: var(--text-ghost);
  }

  .explore-row__text {
    font-family: var(--font-mono);
    font-size: 12px;
    color: var(--text-primary);
    white-space: pre-wrap;
    word-break: break-word;
  }

  .explore__composer {
    display: flex;
    gap: var(--s-2);
    padding: var(--s-2) var(--s-3);
    background: var(--bg-surface);
    border-top: 1px solid var(--border-subtle);
    align-items: flex-end;
    flex-shrink: 0;
  }

  .explore__input {
    flex: 1;
    min-height: 32px;
    max-height: 96px;
    background: var(--bg-input);
    border: 1px solid var(--border-subtle);
    border-radius: var(--r-xs);
    color: var(--text-primary);
    font-family: var(--font-mono);
    font-size: 12px;
    padding: 6px 8px;
    resize: vertical;
    outline: none;
    transition: border-color var(--dur-fast) var(--ease-std);
  }

  .explore__input:focus {
    border-color: var(--teal-line);
  }

  .explore__input::placeholder {
    color: var(--text-ghost);
  }
</style>
