<script lang="ts">
  // E-21 Tier-A — PagerankPanel.
  // Surfaces the existing `get_pagerank_top` Tauri command (commands.rs:1178)
  // which reads dkg_nodes_pagerank via SsbViewer::pagerank_top (ssb_viewer.rs:2897).
  // Producer is knowledge::pipeline::compute_repo_map / knowledge::pagerank
  // (C-3 ship). When the table is absent, the command returns
  // `{known: false, rows: []}` — we render the empty-state hint, not an error.
  //
  // Aligns with E-29 carry-overs: re-entrancy guard + Wong tokens + direct
  // `invoke` from @tauri-apps/api/core (tryInvoke is private in stub.ts).

  import { onMount, onDestroy } from 'svelte';
  import { invoke } from '@tauri-apps/api/core';

  type PagerankRow = {
    symbol: string;
    path: string;
    score: number;
  };

  type PagerankResponse = {
    known: boolean;
    rows: PagerankRow[];
  };

  const POLL_INTERVAL_MS = 15_000;
  const ROW_LIMIT = 20;

  let rows = $state<PagerankRow[]>([]);
  let known = $state<boolean>(false);
  let loading = $state<boolean>(true);
  let error = $state<string | null>(null);
  let timer: ReturnType<typeof setInterval> | null = null;
  let refreshing = false;

  async function refresh() {
    if (refreshing) return; // dedupe overlapping polls
    refreshing = true;
    try {
      const live = await invoke<PagerankResponse>('get_pagerank_top', { n: ROW_LIMIT });
      if (live && typeof live === 'object') {
        rows = Array.isArray(live.rows) ? live.rows : [];
        known = Boolean(live.known);
      } else {
        rows = [];
        known = false;
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
    void refresh();
    timer = setInterval(() => { void refresh(); }, POLL_INTERVAL_MS);
  });

  onDestroy(() => {
    if (timer !== null) clearInterval(timer);
    timer = null;
  });

  // Score-tone scale per Wong palette: top quartile green, mid-upper blue,
  // mid-lower orange, bottom quartile vermilion.
  function scoreTone(idx: number, total: number): string {
    if (total === 0) return 'var(--text-ghost)';
    const q = idx / total;
    if (q < 0.25) return 'var(--signal-green)';
    if (q < 0.5) return 'var(--signal-blue)';
    if (q < 0.75) return 'var(--signal-orange)';
    return 'var(--signal-vermilion)';
  }

  function truncate(s: string, max: number): string {
    if (s.length <= max) return s;
    return s.slice(0, max - 1) + '…';
  }
</script>

<section class="pagerank-panel" aria-label="PageRank top symbols">
  <header class="hd">
    <h3 class="hd-title">pagerank top {ROW_LIMIT}</h3>
    <span class="hd-hint">dkg_nodes_pagerank · polled 15s</span>
  </header>

  {#if loading && rows.length === 0}
    <div class="state state--loading">loading pagerank…</div>
  {:else if error}
    <div class="state state--error">pagerank query failed: {error}</div>
  {:else if !known || rows.length === 0}
    <div class="state state--empty">
      no pagerank data — run knowledge::pipeline::compute_repo_map to populate
      <div class="state-hint">cli: <code>elai symbol_graph rank --top {ROW_LIMIT}</code></div>
    </div>
  {:else}
    <ol class="rows" aria-label="top {rows.length} symbols by pagerank score">
      {#each rows as r, i (r.symbol)}
        <li class="row">
          <span class="row-rank" style="color: {scoreTone(i, rows.length)};">#{i + 1}</span>
          <span class="row-symbol" title={r.symbol}>{truncate(r.symbol, 48)}</span>
          <span class="row-score" title="pagerank score">{r.score.toFixed(5)}</span>
          <span class="row-path" title={r.path}>{truncate(r.path, 36)}</span>
        </li>
      {/each}
    </ol>
  {/if}
</section>

<style>
  .pagerank-panel {
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
  .rows {
    list-style: none;
    margin: 0;
    padding: 0;
    display: flex;
    flex-direction: column;
    gap: 2px;
  }
  .row {
    display: grid;
    grid-template-columns: 32px 1fr 80px 180px;
    align-items: center;
    gap: var(--s-2);
    padding: 4px var(--s-2);
    background: var(--bg-base);
    border: 1px solid var(--border-subtle);
    border-radius: var(--r-sm);
    font-family: var(--font-mono);
    font-size: 11px;
  }
  .row-rank {
    font-weight: 500;
  }
  .row-symbol {
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    color: var(--text-primary);
  }
  .row-score {
    text-align: right;
    color: var(--signal-blue);
  }
  .row-path {
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    color: var(--text-ghost);
    font-size: 10px;
  }
  .state {
    font-family: var(--font-mono);
    font-size: 11px;
    color: var(--text-secondary);
    padding: var(--s-2);
  }
  .state--error { color: var(--fail); }
  .state--empty {
    color: var(--text-ghost);
    display: flex;
    flex-direction: column;
    gap: 4px;
  }
  .state-hint code {
    color: var(--text-secondary);
    background: var(--bg-input);
    padding: 1px 4px;
    border-radius: var(--r-xs);
  }
</style>
