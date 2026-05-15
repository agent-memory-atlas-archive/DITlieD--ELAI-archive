<script lang="ts">
  import { onMount } from 'svelte';
  import { knowledgeData } from '../state/drawers.svelte';
  import { Card, Badge, Toggle, UnwiredNote } from '$lib/ui';
  import IndexingProgressPanel from '../components/IndexingProgressPanel.svelte';
  import type { BreakerState, SearchMode } from '../api/types';

  onMount(() => {
    knowledgeData.bootstrap();
    return knowledgeData.teardown;
  });

  const d = knowledgeData;

  // Maximum grammar entries to display inline before showing "+N more".
  const MAX_DISPLAY_LANGUAGES = 8;

  // Freshness decay enabled (from mock: no dedicated freshness store key, use dkgStats)
  let freshnessEnabled = $state(true);
  const lastRecalcTs = new Date(Date.now() - 18 * 60 * 1000).toISOString().replace('T', ' ').slice(0, 19) + 'Z';

  // Search mode transitions (simulated from known query trace modes in mock-data)
  const searchModeTransitions = [
    { ts: '11:20:01', from: 'HYBRID' as SearchMode, to: 'BM25_ONLY' as SearchMode, reason: 'graphcodebert breaker open' },
    { ts: '11:35:44', from: 'BM25_ONLY' as SearchMode, to: 'SPARSE_ONLY' as SearchMode, reason: 'vector index rebuild' },
    { ts: '12:01:23', from: 'SPARSE_ONLY' as SearchMode, to: 'HYBRID' as SearchMode, reason: 'all breakers closed' },
  ];

  function fmtTs(ts: number): string {
    const d = new Date(ts);
    return d.toISOString().slice(11, 19);
  }

  function breakerTone(state: BreakerState): string {
    if (state === 'CLOSED') return 'teal';
    if (state === 'HALF_OPEN') return 'warn';
    return 'fail';
  }

  function modeTone(mode: SearchMode): string {
    if (mode === 'HYBRID') return 'teal';
    return 'warn';
  }

  function verdictTone(pass: boolean): string {
    return pass ? 'teal' : 'fail';
  }

  // Compute stage bar segments for the most recent trace
  const trace = $derived(d.recentTraces[0]);
  const totalMs = $derived(trace?.total_ms ?? 1);
  const segments = $derived(
    trace?.stages.map(s => ({
      name: s.name,
      pass: s.pass,
      pct: Math.round((s.latency_ms / totalMs) * 100),
    })) ?? []
  );

  // Recall tier highlight: DEEP MRR < STANDARD MRR
  const recall = $derived(d.recallMetrics);
  const deepMrr = $derived(recall.find(r => r.tier === 'DEEP')?.mrr ?? 0);
  const stdMrr = $derived(recall.find(r => r.tier === 'STANDARD')?.mrr ?? 0);
</script>

<div class="knowledge-drawer">

  <!-- 1. DKG / BM25 / VECTORS -->
  <Card variant="plain" padding={12}>
    <div class="section-label">DKG / BM25 / VECTORS</div>
    <div class="index-rows">
      <div class="index-row">
        <span class="index-name">DKG</span>
        <Badge tone="teal" size="xs">READY</Badge>
        <span class="index-metric">{d.dkgStats.nodes} nodes / {d.dkgStats.edges} edges / 9.67s build</span>
      </div>
      <div class="index-row">
        <span class="index-name">BM25</span>
        <Badge tone="teal" size="xs">READY</Badge>
        <span class="index-metric">{d.dkgStats.bm25_chunks.toLocaleString()} chunks indexed</span>
      </div>
      <div class="index-row">
        <span class="index-name">VECTORS</span>
        <Badge tone={d.dkgStats.vector_status === 'READY' ? 'teal' : d.dkgStats.vector_status === 'DEGRADED' ? 'warn' : 'ghost'} size="xs">{d.dkgStats.vector_status}</Badge>
        <span class="index-metric">decay {d.dkgStats.freshness_decay_pct}%</span>
      </div>
    </div>
  </Card>

  <!-- B-31 Phase 4 (Cycle B): indexing progress + cancel surface. -->
  <IndexingProgressPanel />

  <!-- 2. LANGUAGES LOADED -->
  <Card variant="plain" padding={12}>
    <div class="section-label">LANGUAGES LOADED</div>
    {#if d.unwired.languageStatus}
      <UnwiredNote
        phase="B-30"
        feature="per-grammar load status (loaded / failed grammar list)"
        note="get_language_grammar_status not yet wired; LanguageRegistry emitter lands with B-30 Pass A.2"
      />
    {:else if d.languageStatus}
      {@const ls = d.languageStatus}
      {@const displayLoaded = ls.loaded.slice(0, MAX_DISPLAY_LANGUAGES)}
      {@const hiddenCount = ls.loaded.length - displayLoaded.length}
      <div class="lang-summary">
        <span class="lang-count">{ls.loaded.length} loaded</span>
        {#if ls.failed.length > 0}
          <span class="lang-failed">{ls.failed.length} failed</span>
        {/if}
      </div>
      <div class="lang-list">
        {#each displayLoaded as lang}
          <span class="lang-chip">{lang}</span>
        {/each}
        {#if hiddenCount > 0}
          <span class="lang-chip lang-chip--more">+{hiddenCount} more</span>
        {/if}
      </div>
      {#if ls.failed.length > 0}
        <div class="lang-list lang-list--failed">
          {#each ls.failed as [langId, errMsg]}
            <span class="lang-chip lang-chip--failed" title={errMsg}>{langId}</span>
          {/each}
        </div>
      {/if}
    {:else}
      <span class="ghost-text">no grammar status yet</span>
    {/if}
  </Card>

  <!-- 3. QUERY TRACE (most recent) -->
  <Card variant="plain" padding={12}>
    <div class="section-label">QUERY TRACE (most recent)</div>
    {#if d.unwired.recentTraces}
      <UnwiredNote
        phase="B-4"
        feature="per-query stage trace + hit count"
        note="get_query_trace_recent returns []; SSB QueryTrace row type not yet defined"
      />
    {:else if trace}
      <div class="query-text">{trace.query}</div>
      <div class="stage-bar-wrap">
        <div class="stage-bar">
          {#each segments as seg}
            <div
              class="stage-seg"
              class:stage-seg--pass={seg.pass}
              class:stage-seg--fail={!seg.pass}
              style="width: {seg.pct}%;"
              title="{seg.name} {seg.pct}%"
            ></div>
          {/each}
        </div>
        <div class="stage-names">
          {#each segments as seg}
            <span class="stage-name">{seg.name}</span>
          {/each}
        </div>
      </div>
      <div class="trace-meta">
        <span class="trace-hits">{trace.hit_count} hits</span>
        <span class="trace-latency">{trace.total_ms} ms total</span>
      </div>
    {:else}
      <span class="ghost-text">no recent traces</span>
    {/if}
  </Card>

  <!-- 3. SEARCH MODE -->
  <Card variant="plain" padding={12}>
    <div class="section-label">SEARCH MODE</div>
    {#if d.unwired.searchMode}
      <UnwiredNote
        phase="B-17"
        feature="HYBRID / SPARSE_ONLY / BM25_ONLY cascade + transition history"
        note="get_search_mode returns hardcoded HYBRID; SearchMode cascade not wired into query path"
      />
    {:else}
      <div class="mode-current">
        <Badge tone={modeTone(d.searchMode)} size="sm">{d.searchMode}</Badge>
      </div>
      <div class="mode-transitions">
        {#each searchModeTransitions.slice(-3) as t}
          <div class="mode-row">
            <span class="ghost-text">{t.ts}</span>
            <span class="mode-path">{t.from} → {t.to}</span>
            <span class="ghost-text">reason: {t.reason}</span>
          </div>
        {/each}
      </div>
    {/if}
  </Card>

  <!-- 4. EMBEDDING BREAKERS -->
  <Card variant="plain" padding={12}>
    <div class="section-label">EMBEDDING BREAKERS</div>
    <div class="breaker-grid">
      {#each d.breakers as b}
        <div class="breaker-chip">
          <span class="breaker-name">{b.model.split('-').slice(0, 2).join('-')}</span>
          <Badge tone={breakerTone(b.state)} size="xs">{b.state}</Badge>
          <span class="breaker-fails">{b.failures}f</span>
        </div>
      {/each}
    </div>
  </Card>

  <!-- 5. FRESHNESS DECAY -->
  <Card variant="plain" padding={12}>
    <div class="section-label">FRESHNESS DECAY</div>
    <div class="freshness-row">
      <Toggle
        bind:checked={freshnessEnabled}
        label="enabled"
        onchange={(v) => { freshnessEnabled = v; }}
      />
      <span class="ghost-text">last-recalc {lastRecalcTs}</span>
    </div>
  </Card>

  <!-- 6. RECALL @ TIER -->
  <Card variant="plain" padding={12}>
    <div class="section-label">RECALL @ TIER</div>
    {#if d.unwired.recallMetrics}
      <UnwiredNote
        phase="B-4"
        feature="live R@5 / R@10 / MRR per tier"
        note="get_recall_metrics returns static toml constants; live retrieval-quality benchmark not wired"
      />
    {:else}
      <div class="recall-rows">
        {#each recall as r}
          {@const isDeepMrrWarn = r.tier === 'DEEP' && r.mrr < stdMrr}
          <div class="recall-row">
            <span class="tier-label">{r.tier}</span>
            <div class="recall-badges">
              <Badge tone="ghost" size="xs">R@5 {r.r_at_5.toFixed(3)}</Badge>
              <Badge tone="ghost" size="xs">R@10 {r.r_at_10.toFixed(3)}</Badge>
              <Badge
                tone={isDeepMrrWarn ? 'warn' : 'ghost'}
                size="xs"
              >MRR {r.mrr.toFixed(3)}{isDeepMrrWarn ? ' !' : ''}</Badge>
            </div>
          </div>
        {/each}
      </div>
    {/if}
  </Card>

  <!-- B-29 Phase 5: IMPACT ANALYSIS panel -->
  <Card variant="plain" padding={12}>
    <div class="section-label">IMPACT ANALYSIS</div>
    <UnwiredNote
      phase="B-29"
      feature="symbol-graph impact analysis (BFS reverse-dependency traversal)"
      note="get_impact_analysis Tauri command wired in B-29 Phase 5; submit_task delegation pending B-29 Phase 6"
    />
  </Card>

</div>

<style>
  .knowledge-drawer {
    display: flex;
    flex-direction: column;
    gap: 8px;
  }

  .section-label {
    font-family: var(--font-mono);
    font-size: 11px;
    color: var(--text-secondary);
    letter-spacing: 0.08em;
    text-transform: uppercase;
    margin-bottom: 8px;
  }

  /* Index rows */
  .index-rows {
    display: flex;
    flex-direction: column;
    gap: 4px;
  }

  .index-row {
    display: flex;
    align-items: center;
    gap: 8px;
  }

  .index-name {
    font-family: var(--font-mono);
    font-size: 11px;
    color: var(--text-primary);
    width: 56px;
    flex-shrink: 0;
  }

  .index-metric {
    font-family: var(--font-mono);
    font-size: 11px;
    color: var(--text-secondary);
  }

  /* Query trace */
  .query-text {
    font-family: var(--font-mono);
    font-size: 12px;
    font-style: italic;
    color: var(--text-primary);
    margin-bottom: 8px;
  }

  .stage-bar-wrap {
    margin-bottom: 6px;
  }

  .stage-bar {
    display: flex;
    width: 100%;
    height: 4px;
    border-radius: var(--r-xs);
    overflow: hidden;
    gap: 1px;
    margin-bottom: 4px;
  }

  .stage-seg {
    height: 100%;
    min-width: 2px;
  }

  .stage-seg--pass {
    background: var(--teal-bright);
  }

  .stage-seg--fail {
    background: var(--fail);
  }

  .stage-names {
    display: flex;
    gap: 8px;
  }

  .stage-name {
    font-family: var(--font-mono);
    font-size: 10px;
    color: var(--text-ghost);
  }

  .trace-meta {
    display: flex;
    justify-content: space-between;
    margin-top: 4px;
  }

  .trace-hits {
    font-family: var(--font-mono);
    font-size: 11px;
    color: var(--text-ghost);
  }

  .trace-latency {
    font-family: var(--font-mono);
    font-size: 11px;
    color: var(--text-ghost);
  }

  /* Search mode */
  .mode-current {
    margin-bottom: 6px;
  }

  .mode-transitions {
    display: flex;
    flex-direction: column;
    gap: 3px;
  }

  .mode-row {
    display: flex;
    flex-direction: column;
    gap: 1px;
  }

  .mode-path {
    font-family: var(--font-mono);
    font-size: 10px;
    color: var(--text-secondary);
  }

  /* Breakers */
  .breaker-grid {
    display: flex;
    flex-direction: column;
    gap: 4px;
  }

  .breaker-chip {
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 4px 6px;
    background: var(--bg-input);
    border-radius: var(--r-sm);
  }

  .breaker-name {
    font-family: var(--font-mono);
    font-size: 10px;
    color: var(--text-secondary);
    flex: 1;
    min-width: 0;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .breaker-fails {
    font-family: var(--font-mono);
    font-size: 10px;
    color: var(--text-ghost);
    flex-shrink: 0;
  }

  /* Freshness */
  .freshness-row {
    display: flex;
    align-items: center;
    gap: 12px;
    flex-wrap: wrap;
  }

  /* Recall */
  .recall-rows {
    display: flex;
    flex-direction: column;
    gap: 6px;
  }

  .recall-row {
    display: flex;
    align-items: center;
    gap: 8px;
  }

  .tier-label {
    font-family: var(--font-mono);
    font-size: 10px;
    color: var(--text-secondary);
    letter-spacing: 0.08em;
    text-transform: uppercase;
    width: 56px;
    flex-shrink: 0;
  }

  .recall-badges {
    display: flex;
    gap: 4px;
    flex-wrap: wrap;
  }

  .recall-note {
    margin-top: 6px;
    font-family: var(--font-mono);
    font-size: 10px;
  }

  /* Languages loaded */
  .lang-summary {
    display: flex;
    align-items: center;
    gap: 8px;
    margin-bottom: 6px;
  }

  .lang-count {
    font-family: var(--font-mono);
    font-size: 11px;
    color: var(--text-primary);
  }

  .lang-failed {
    font-family: var(--font-mono);
    font-size: 11px;
    color: var(--fail);
  }

  .lang-list {
    display: flex;
    flex-wrap: wrap;
    gap: 4px;
    margin-bottom: 4px;
  }

  .lang-list--failed {
    margin-top: 4px;
  }

  .lang-chip {
    font-family: var(--font-mono);
    font-size: 10px;
    color: var(--text-secondary);
    background: var(--bg-input);
    border-radius: var(--r-xs);
    padding: 1px 5px;
  }

  .lang-chip--more {
    color: var(--text-ghost);
  }

  .lang-chip--failed {
    color: var(--fail);
    border: 1px solid color-mix(in srgb, var(--fail) 40%, transparent);
  }

  /* Shared */
  .ghost-text {
    font-family: var(--font-mono);
    font-size: 10px;
    color: var(--text-ghost);
  }
</style>
