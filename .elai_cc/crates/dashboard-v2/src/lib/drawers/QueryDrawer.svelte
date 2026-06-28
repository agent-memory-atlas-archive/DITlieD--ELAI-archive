<script lang="ts">
  // E-29 Tier-A — QueryDrawer.
  // Surfaces the existing `get_query_trace_recent` Tauri command
  // (commands.rs:924) which reads C-1 QueryTrace SSB rows via
  // SsbViewer::recent_rows_of_type. Wire shape per commands.rs:944-957:
  //   { seq, ts, query_id, tier, stage_count, result_count, total_latency_ms }
  // The TS QueryTrace interface in api/types.ts is stale (drafted for a
  // pre-C-1 shape with {id, query, mode, stages[]}) — drawer matches the
  // CURRENT wire shape that the Rust command actually emits, not the type
  // interface. Refresh of api/types.ts is a separate cleanup not in scope here.

  import { onMount, onDestroy } from 'svelte';
  import { invoke } from '@tauri-apps/api/core';

  type QueryTraceRow = {
    seq: number;
    ts: number;
    query_id: string | null;
    tier: string | null;
    stage_count: number;
    result_count: number;
    total_latency_ms: number;
  };

  const POLL_INTERVAL_MS = 5_000;
  const ROW_LIMIT = 20;

  let rows = $state<QueryTraceRow[]>([]);
  let loading = $state<boolean>(true);
  let error = $state<string | null>(null);
  let timer: ReturnType<typeof setInterval> | null = null;
  let refreshing = false;

  async function refresh() {
    if (refreshing) return; // dedupe overlapping polls
    refreshing = true;
    try {
      const live = await invoke<QueryTraceRow[]>('get_query_trace_recent', { n: ROW_LIMIT });
      rows = Array.isArray(live) ? live : [];
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

  function fmtMs(ms: number): string {
    if (ms >= 1000) return `${(ms / 1000).toFixed(2)}s`;
    return `${Math.round(ms)}ms`;
  }

  function fmtTs(ts: number): string {
    if (!ts) return '';
    const d = new Date(ts);
    const hh = String(d.getHours()).padStart(2, '0');
    const mm = String(d.getMinutes()).padStart(2, '0');
    const ss = String(d.getSeconds()).padStart(2, '0');
    return `${hh}:${mm}:${ss}`;
  }

  // C-1 tier strings observed in production: 'FAST' | 'STANDARD' | 'DEEP'
  // (plus null when payload missing). Wong tones; unknown falls through ghost.
  function tierTone(tier: string | null): string {
    if (!tier) return 'var(--text-ghost)';
    const upper = tier.toUpperCase();
    if (upper === 'DEEP') return 'var(--signal-green)';
    if (upper === 'STANDARD') return 'var(--signal-blue)';
    if (upper === 'FAST') return 'var(--signal-orange)';
    return 'var(--text-ghost)';
  }

  function tierLabel(tier: string | null): string {
    if (!tier) return 'unknown';
    return tier.toLowerCase();
  }

  function queryIdLabel(id: string | null): string {
    if (!id) return '(unidentified)';
    if (id.length > 28) return id.slice(0, 12) + '…' + id.slice(-12);
    return id;
  }
</script>

<section class="query-drawer" aria-label="Query trace">
  <header class="hd">
    <h3 class="hd-title">query traces</h3>
    <span class="hd-hint">query_trace SSB · last {ROW_LIMIT} · polled 5s</span>
  </header>

  {#if loading && rows.length === 0}
    <div class="state state--loading">loading query traces…</div>
  {:else if error}
    <div class="state state--error">query trace fetch failed: {error}</div>
  {:else if rows.length === 0}
    <div class="state state--empty">no query_trace rows yet — issue a query to populate</div>
  {:else}
    <div class="rows">
      {#each rows as r (r.seq)}
        <div class="row">
          <span class="row-tier" style="color: {tierTone(r.tier)};">{tierLabel(r.tier)}</span>
          <span class="row-qid" title={r.query_id ?? ''}>{queryIdLabel(r.query_id)}</span>
          <span class="row-meta">
            <span class="row-cell" title="stages">{r.stage_count} stg</span>
            <span class="row-cell" title="results">{r.result_count} hits</span>
            <span class="row-ms">{fmtMs(r.total_latency_ms)}</span>
            <span class="row-ts">{fmtTs(r.ts)}</span>
          </span>
        </div>
      {/each}
    </div>
  {/if}
</section>

<style>
  .query-drawer {
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
    font-size: 10px;
    color: var(--text-primary);
    margin: 0;
    text-transform: uppercase;
    letter-spacing: 0.05em;
  }
  .hd-hint {
    font-family: var(--font-mono);
    font-size: 10px;
    color: var(--text-ghost);
  }
  .rows {
    display: flex;
    flex-direction: column;
    gap: var(--s-1);
  }
  .row {
    display: grid;
    grid-template-columns: 64px 1fr auto;
    align-items: center;
    gap: var(--s-2);
    padding: var(--s-2);
    background: var(--bg-base);
    border: 1px solid var(--border-subtle);
    border-radius: var(--r-sm);
    font-family: var(--font-mono);
    font-size: 10px;
  }
  .row-tier {
    font-size: 10px;
    text-transform: uppercase;
    letter-spacing: 0.05em;
  }
  .row-qid {
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    color: var(--text-primary);
  }
  .row-meta {
    display: inline-flex;
    gap: var(--s-3);
    color: var(--text-secondary);
    font-size: 10px;
  }
  .row-cell { color: var(--text-secondary); }
  .row-ms { color: var(--signal-blue); }
  .row-ts { color: var(--text-ghost); }
  .state {
    font-family: var(--font-mono);
    font-size: 10px;
    color: var(--text-secondary);
    padding: var(--s-2);
  }
  .state--error { color: var(--fail); }
  .state--empty { color: var(--text-ghost); }
</style>
