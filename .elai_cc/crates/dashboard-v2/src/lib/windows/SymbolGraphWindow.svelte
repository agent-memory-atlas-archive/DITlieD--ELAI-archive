<script lang="ts">
  // E-40 — Orchestrator-rooted symbol-graph operator window.
  //
  // cross-lang-pair: dashboard/tauri/api
  //
  // Three read-only Tauri commands (commands_e40.rs):
  //   - symbol_graph_query  (defs / refs / callers / callees)
  //   - symbol_graph_pagerank
  //   - symbol_graph_status
  //
  // Reads .claude/tools/elai_graph.json on the Rust side. Mock-mode
  // fallback degrades gracefully: empty results render with a "no
  // graph mounted" empty state.
  //
  // Paper grounding deferred per workflow/sessions/s-25120/mistakes.md
  // cluster #6 (Stack Graphs / GraphCoder / CodexGraph / RANGER /
  // Glean Angle). See workflow/reasoning/E-40-decisions.md.

  import { onMount } from 'svelte';
  import {
    symbolGraphQuery,
    symbolGraphPageRank,
    symbolGraphStatus,
    type SymbolGraphQueryReport,
    type SymbolGraphPageRankReport,
    type SymbolGraphStatusReport,
  } from '../api/stub';

  let queryInput = $state<string>('');
  let queryKind = $state<'defs' | 'refs' | 'callers' | 'callees'>('defs');
  let pagerankSeed = $state<string>('');
  let pagerankK = $state<number>(10);

  let statusReport = $state<SymbolGraphStatusReport | null>(null);
  let queryReport = $state<SymbolGraphQueryReport | null>(null);
  let pagerankReport = $state<SymbolGraphPageRankReport | null>(null);

  let queryBusy = $state<boolean>(false);
  let pagerankBusy = $state<boolean>(false);
  let errorMessage = $state<string | null>(null);

  onMount(() => {
    void refreshStatus();
  });

  async function refreshStatus(): Promise<void> {
    try {
      statusReport = await symbolGraphStatus();
    } catch (err) {
      errorMessage = `status: ${String(err)}`;
    }
  }

  async function runQuery(): Promise<void> {
    if (queryInput.trim().length === 0) {
      errorMessage = 'query: symbol must be non-empty';
      return;
    }
    queryBusy = true;
    errorMessage = null;
    try {
      queryReport = await symbolGraphQuery(queryInput.trim(), queryKind);
    } catch (err) {
      errorMessage = `query: ${String(err)}`;
    } finally {
      queryBusy = false;
    }
  }

  async function runPageRank(): Promise<void> {
    if (pagerankSeed.trim().length === 0) {
      errorMessage = 'pagerank: seed must be non-empty';
      return;
    }
    pagerankBusy = true;
    errorMessage = null;
    try {
      pagerankReport = await symbolGraphPageRank(pagerankSeed.trim(), pagerankK);
    } catch (err) {
      errorMessage = `pagerank: ${String(err)}`;
    } finally {
      pagerankBusy = false;
    }
  }
</script>

<div class="symbol-graph-window">
  <header class="sg-header">
    <h2>Symbol Graph</h2>
    {#if statusReport}
      <span class="sg-status">
        nodes: {statusReport.node_count}
        · lru: {statusReport.lru_capacity}
        · {statusReport.graph_present ? 'mounted' : 'no graph'}
      </span>
    {/if}
    <button type="button" onclick={() => void refreshStatus()}>Refresh</button>
  </header>

  {#if errorMessage}
    <div class="sg-error">{errorMessage}</div>
  {/if}

  <section class="sg-section">
    <h3>Query</h3>
    <div class="sg-row">
      <label for="sg-query-input">Symbol</label>
      <input
        id="sg-query-input"
        type="text"
        placeholder="e.g. compute_impact_radius"
        bind:value={queryInput}
      />
      <label for="sg-kind">Kind</label>
      <select id="sg-kind" bind:value={queryKind}>
        <option value="defs">defs</option>
        <option value="refs">refs</option>
        <option value="callers">callers</option>
        <option value="callees">callees</option>
      </select>
      <button type="button" disabled={queryBusy} onclick={() => void runQuery()}>
        {queryBusy ? '...' : 'Run'}
      </button>
    </div>
    {#if queryReport}
      <p class="sg-count">{queryReport.result_count} result(s)</p>
      <ul class="sg-list">
        {#each queryReport.results as match (match.id)}
          <li>
            <span class="sg-name">{match.name}</span>
            <span class="sg-file">{match.file}:{match.line}</span>
            <span class="sg-kind">{match.kind}</span>
          </li>
        {/each}
      </ul>
    {/if}
  </section>

  <section class="sg-section">
    <h3>Personalized PageRank</h3>
    <div class="sg-row">
      <label for="sg-pagerank-seed">Seed</label>
      <input
        id="sg-pagerank-seed"
        type="text"
        placeholder="e.g. SymbolGraphCache"
        bind:value={pagerankSeed}
      />
      <label for="sg-pagerank-k">k</label>
      <input
        id="sg-pagerank-k"
        type="number"
        min="1"
        max="100"
        bind:value={pagerankK}
      />
      <button
        type="button"
        disabled={pagerankBusy}
        onclick={() => void runPageRank()}
      >
        {pagerankBusy ? '...' : 'Rank'}
      </button>
    </div>
    {#if pagerankReport}
      <ul class="sg-list">
        {#each pagerankReport.ranked as ranked (ranked.id)}
          <li>
            <span class="sg-name">{ranked.id}</span>
            <span class="sg-file">score: {ranked.score.toFixed(6)}</span>
          </li>
        {/each}
      </ul>
    {/if}
  </section>
</div>

<style>
  .symbol-graph-window {
    display: flex;
    flex-direction: column;
    height: 100%;
    width: 100%;
    overflow: auto;
    padding: 0.75rem 1rem;
    gap: 0.75rem;
  }
  .sg-header {
    display: flex;
    align-items: center;
    gap: 0.75rem;
  }
  .sg-status {
    font-size: 0.85rem;
    opacity: 0.8;
  }
  .sg-error {
    background: #5b1c1c;
    color: #fcd7d7;
    padding: 0.4rem 0.6rem;
    border-radius: 4px;
    font-size: 0.85rem;
  }
  .sg-section {
    display: flex;
    flex-direction: column;
    gap: 0.4rem;
  }
  .sg-row {
    display: flex;
    align-items: center;
    gap: 0.4rem;
    flex-wrap: wrap;
  }
  .sg-row input[type='text'] {
    flex: 1 1 12rem;
    min-width: 6rem;
  }
  .sg-row input[type='number'] {
    width: 4rem;
  }
  .sg-count {
    margin: 0;
    font-size: 0.85rem;
    opacity: 0.75;
  }
  .sg-list {
    list-style: none;
    padding: 0;
    margin: 0;
    display: flex;
    flex-direction: column;
    gap: 0.2rem;
    max-height: 24rem;
    overflow-y: auto;
  }
  .sg-list li {
    display: flex;
    gap: 0.6rem;
    font-size: 0.85rem;
    border-bottom: 1px solid var(--surface-2, #2a2a2a);
    padding: 0.2rem 0;
  }
  .sg-name {
    font-weight: 600;
    min-width: 8rem;
  }
  .sg-file {
    opacity: 0.7;
  }
  .sg-kind {
    opacity: 0.6;
    font-style: italic;
  }
</style>
