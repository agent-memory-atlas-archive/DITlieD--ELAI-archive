<script lang="ts">
  import { onMount } from 'svelte';
  import { knowledgeData } from '../state/drawers.svelte';
  import { Card, Badge, Toggle, UnwiredNote } from '$lib/ui';
  import IndexingProgressPanel from '../components/IndexingProgressPanel.svelte';
  // B-35 Phase C: Context Artifacts panel.
  import ArtifactsPanel from '../components/ArtifactsPanel.svelte';
  import type { BreakerState, SearchMode } from '../api/types';
  import { get_impact_analysis, submit_task, type ImpactAnalysis } from '../api/stub';
  // B-33 — Visualize button cross-link: open GraphWindow + push impact target.
  import { graph } from '../state/graph.svelte';
  import { workspace } from '../state/workspace.svelte';

  function visualizeImpact(target: string) {
    if (!target) return;
    graph.set_mode('file');
    graph.request_impact(target);
    workspace.openTab('primary', 'graph');
  }

  // C-24 (Task 3) — /symbol_impact target input. Sends a task to the agent
  // via submit_task("/symbol_impact target=<...>") so the agent's tool
  // surface (which now includes symbol_impact per build_tool_catalog C-24
  // expansion) can self-select the tool. The next poll of
  // get_impact_analysis() picks up the new SymbolQuery row and populates
  // the IMPACT ANALYSIS card body below.
  let symbolImpactInput = $state<string>('');
  let symbolImpactRunning = $state<boolean>(false);

  async function runSymbolImpact() {
    const target = symbolImpactInput.trim();
    if (target.length === 0) return;
    symbolImpactRunning = true;
    try {
      await submit_task(`/symbol_impact target=${target}`);
    } finally {
      symbolImpactRunning = false;
    }
  }

  let impact = $state<ImpactAnalysis>({ target: '', total_files: 0, hop_bands: [], unresolved_pct: 0.0 });
  let impactLoaded = $state(false);

  onMount(() => {
    knowledgeData.bootstrap();
    // B-29 Phase 5: poll the most recent symbol_query SSB row every 5s
    // so the IMPACT ANALYSIS panel reflects fresh tool output without
    // forcing the user to reload the drawer.
    const refresh = async () => {
      try {
        impact = await get_impact_analysis();
      } catch (e) {
        // get_impact_analysis falls back to empty shape; no console spam.
      } finally {
        impactLoaded = true;
      }
    };
    refresh();
    const interval = setInterval(refresh, 5000);
    return () => {
      clearInterval(interval);
      knowledgeData.teardown();
    };
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

  // ----------------------------------------------------------------
  // E-17 SP4: RAG retrieval-quality beacons
  //
  // Each card consumes a feed off knowledgeData (10s polling) and
  // renders four explicit branches in this priority order:
  //   1. error  — show retry banner (red), keeps last-known data visible
  //   2. loading on first cycle — skeleton placeholder
  //   3. has rows — populated card
  //   4. empty — informative ghost-text per producer
  // ----------------------------------------------------------------

  // Format an SSB epoch-ms ts to HH:MM:SS for the chip strip.
  function fmtTsShort(ts: number): string {
    if (!ts) return '--:--:--';
    return new Date(ts).toISOString().slice(11, 19);
  }

  // Format an Option<f64> RRF weight: null -> "off", number -> 3-decimal.
  function fmtWeight(w: number | null | undefined): string {
    if (w === null || w === undefined) return 'off';
    return w.toFixed(3);
  }

  // DEEP pool capped derivations.
  const deepPoolCapped = $derived(d.deepPoolCapped);
  const deepPoolLatest = $derived(deepPoolCapped[0]);
  const deepPoolMeanCapped = $derived(
    deepPoolCapped.length === 0
      ? 0
      : Math.round(
          deepPoolCapped.reduce((s, r) => s + r.capped_count, 0) / deepPoolCapped.length,
        ),
  );
  const deepPoolMaxCapped = $derived(
    deepPoolCapped.reduce((m, r) => Math.max(m, r.capped_count), 0),
  );

  // RRF quality gate derivations.
  const rrfGate = $derived(d.rrfQualityGate);
  const rrfGateLatest = $derived(rrfGate[0]);
  const rrfGateMeanDropped = $derived(
    rrfGate.length === 0
      ? 0
      : Math.round(rrfGate.reduce((s, r) => s + r.dropped_count, 0) / rrfGate.length),
  );

  // HyDE skipped derivations.
  const hydeSkipped = $derived(d.hydeSkippedCodeQuery);
  const hydeClassCounts = $derived.by(() => {
    const counts = new Map<string, number>();
    for (const row of hydeSkipped) {
      const key = row.query_class ?? '(unknown)';
      counts.set(key, (counts.get(key) ?? 0) + 1);
    }
    return Array.from(counts.entries()).sort((a, b) => b[1] - a[1]);
  });

  // RRF component weights — last 10 rows. The producer can emit f64::NaN
  // -> JSON null path is handled by fmtWeight().
  const rrfWeights = $derived(d.rrfComponentWeights);
  const rrfWeightsLatest = $derived(rrfWeights[0]);

  // Shared loading / error flags so each card branches consistently.
  const ragLoadState = $derived(d.ragQualityLoadState);
  const ragLastError = $derived(d.ragQualityLastError);
  const ragShowSkeleton = $derived(ragLoadState === 'loading');
  const ragShowError = $derived(ragLoadState === 'error');

  async function retryRagQuality() {
    try {
      await d.refresh();
    } catch (e) {
      console.warn('[KnowledgeDrawer] manual retry failed', e);
    }
  }
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

  <!-- B-31 Phase 4 (Cycle B): indexing progress + cancel surface.
       C-17 (DRIFT-J): IndexingProgressPanel also renders the resilience
       sub-block now — auto_resume status, Watch-scope lock count, last
       IndexResume banner, LockContested banner, and per-batch duration
       parsed from the C-17 batch_duration_ms field. -->
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

  <!-- E-17 SP4 — RAG retrieval-quality beacons (4 cards).
       Each card surfaces one of the SP2 producer-side events; rows are
       SSB-backed via SsbViewer::recent_rows_of_type and polled on the
       knowledgeData 10s cadence. Empty arrays are steady state for most
       sessions (not "unwired"); error state renders a retry button. -->

  <!-- DEEP POOL CAPPED -->
  <Card variant="plain" padding={12}>
    <div class="section-label">DEEP POOL CAPPED</div>
    {#if ragShowError}
      <div class="rag-error-banner">
        <span class="rag-error-text">refresh failed{ragLastError ? `: ${ragLastError}` : ''}</span>
        <button class="rag-retry-btn" type="button" onclick={() => void retryRagQuality()}>Retry</button>
      </div>
    {:else if ragShowSkeleton}
      <div class="rag-skeleton">loading...</div>
    {:else if deepPoolCapped.length > 0}
      <div class="rag-stat-row">
        <span class="rag-stat-label">latest</span>
        <span class="rag-stat-value">
          capped {deepPoolLatest?.capped_count ?? 0}/{deepPoolLatest?.max_allowed ?? 0}
          across {deepPoolLatest?.stage_count ?? 0} stage{(deepPoolLatest?.stage_count ?? 0) === 1 ? '' : 's'}
        </span>
      </div>
      <div class="rag-stat-row">
        <span class="rag-stat-label">mean / max (last {deepPoolCapped.length})</span>
        <span class="rag-stat-value">{deepPoolMeanCapped} / {deepPoolMaxCapped}</span>
      </div>
      <div class="rag-chip-strip">
        {#each deepPoolCapped as r}
          <span class="rag-chip" title="seq {r.seq} @ {fmtTsShort(r.ts)} — capped {r.capped_count}/{r.max_allowed} across {r.stage_count} stages">
            {fmtTsShort(r.ts)} <span class="rag-chip-meta">{r.capped_count}</span>
          </span>
        {/each}
      </div>
    {:else}
      <span class="ghost-text">No DEEP queries capped yet (pool &lt; max_allowed). Threshold is configurable via [knowledge.deep] max_candidates.</span>
    {/if}
  </Card>

  <!-- RRF QUALITY GATE -->
  <Card variant="plain" padding={12}>
    <div class="section-label">RRF QUALITY GATE</div>
    {#if ragShowError}
      <div class="rag-error-banner">
        <span class="rag-error-text">refresh failed{ragLastError ? `: ${ragLastError}` : ''}</span>
        <button class="rag-retry-btn" type="button" onclick={() => void retryRagQuality()}>Retry</button>
      </div>
    {:else if ragShowSkeleton}
      <div class="rag-skeleton">loading...</div>
    {:else if rrfGate.length > 0}
      <div class="rag-stat-row">
        <span class="rag-stat-label">latest</span>
        <span class="rag-stat-value">
          dropped {rrfGateLatest?.dropped_count ?? 0}/{rrfGateLatest?.total_before ?? 0}
          below floor {(rrfGateLatest?.floor ?? 0).toFixed(4)}
        </span>
      </div>
      <div class="rag-stat-row">
        <span class="rag-stat-label">mean dropped (last {rrfGate.length})</span>
        <span class="rag-stat-value">{rrfGateMeanDropped}</span>
      </div>
      <div class="rag-chip-strip">
        {#each rrfGate as r}
          <span class="rag-chip" title="seq {r.seq} @ {fmtTsShort(r.ts)} — dropped {r.dropped_count}/{r.total_before} below {r.floor.toFixed(4)}">
            {fmtTsShort(r.ts)} <span class="rag-chip-meta">{r.dropped_count}</span>
          </span>
        {/each}
      </div>
    {:else}
      <span class="ghost-text">No RRF candidates dropped (all above floor).</span>
    {/if}
  </Card>

  <!-- HYDE SKIPPED (CODE QUERY) -->
  <Card variant="plain" padding={12}>
    <div class="section-label">HYDE SKIPPED (CODE QUERY)</div>
    {#if ragShowError}
      <div class="rag-error-banner">
        <span class="rag-error-text">refresh failed{ragLastError ? `: ${ragLastError}` : ''}</span>
        <button class="rag-retry-btn" type="button" onclick={() => void retryRagQuality()}>Retry</button>
      </div>
    {:else if ragShowSkeleton}
      <div class="rag-skeleton">loading...</div>
    {:else if hydeSkipped.length > 0}
      <div class="rag-stat-row">
        <span class="rag-stat-label">class distribution (last {hydeSkipped.length})</span>
        <span class="rag-stat-value">
          {#each hydeClassCounts as [cls, n], i}
            {cls}: {n}{i < hydeClassCounts.length - 1 ? ', ' : ''}
          {/each}
        </span>
      </div>
      <div class="rag-event-list">
        {#each hydeSkipped.slice(0, 5) as r}
          <div class="rag-event-row">
            <span class="rag-event-ts">{fmtTsShort(r.ts)}</span>
            <Badge tone={r.code_signal_present ? 'warn' : 'ghost'} size="xs">{r.query_class ?? 'unknown'}</Badge>
            <span class="rag-event-meta">{r.word_count}w</span>
          </div>
        {/each}
      </div>
    {:else}
      <span class="ghost-text">HyDE was applied to every recent query (none classified as code-symbol).</span>
    {/if}
  </Card>

  <!-- RRF COMPONENT WEIGHTS -->
  <Card variant="plain" padding={12}>
    <div class="section-label">RRF COMPONENT WEIGHTS</div>
    {#if ragShowError}
      <div class="rag-error-banner">
        <span class="rag-error-text">refresh failed{ragLastError ? `: ${ragLastError}` : ''}</span>
        <button class="rag-retry-btn" type="button" onclick={() => void retryRagQuality()}>Retry</button>
      </div>
    {:else if ragShowSkeleton}
      <div class="rag-skeleton">loading...</div>
    {:else if rrfWeights.length > 0}
      <div class="rag-stat-row">
        <span class="rag-stat-label">latest</span>
        <span class="rag-stat-value">
          bm25={fmtWeight(rrfWeightsLatest?.bm25_weight)} /
          code={fmtWeight(rrfWeightsLatest?.code_vec_weight)} /
          text={fmtWeight(rrfWeightsLatest?.text_vec_weight)}
        </span>
      </div>
      <div class="rag-weight-table">
        <div class="rag-weight-header">
          <span>ts</span>
          <span>bm25</span>
          <span>code</span>
          <span>text</span>
        </div>
        {#each rrfWeights as r}
          <div class="rag-weight-row">
            <span class="rag-weight-ts">{fmtTsShort(r.ts)}</span>
            <span class="rag-weight-cell">{fmtWeight(r.bm25_weight)}</span>
            <span class="rag-weight-cell">{fmtWeight(r.code_vec_weight)}</span>
            <span class="rag-weight-cell">{fmtWeight(r.text_vec_weight)}</span>
          </div>
        {/each}
      </div>
    {:else}
      <span class="ghost-text">No recent queries -- run a query to populate.</span>
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

  <!-- B-35 Phase C: CONTEXT ARTIFACTS panel.
       db_path is left blank here so the panel renders gracefully in browser mode.
       When Tauri commands are registered (Phase C2), pass the project db_path prop. -->
  <ArtifactsPanel db_path="" />

  <!-- B-29 Phase 5: IMPACT ANALYSIS panel -->
  <Card variant="plain" padding={12}>
    <div class="section-label">IMPACT ANALYSIS</div>

    <!-- C-24 (Task 3) — /symbol_impact runner. The agent's tool catalog
         includes symbol_impact (C-24 build_tool_catalog expansion), so the
         submit_task() helper drops the agent into a 1-iteration tool call
         that emits a SymbolQuery SSB row. The 5s poll picks it up and
         re-renders the card body below. -->
    <div class="impact-runner">
      <input
        class="impact-runner__input mono"
        type="text"
        placeholder="symbol target (e.g. SsbActionType)"
        bind:value={symbolImpactInput}
        onkeydown={(e) => {
          if (e.key === 'Enter') {
            e.preventDefault();
            void runSymbolImpact();
          }
        }}
        data-testid="impact-target-input"
        disabled={symbolImpactRunning}
      />
      <button
        class="impact-runner__btn"
        onclick={() => void runSymbolImpact()}
        disabled={symbolImpactRunning || symbolImpactInput.trim().length === 0}
        data-testid="impact-run"
      >
        {symbolImpactRunning ? 'running...' : 'Run /symbol_impact'}
      </button>
    </div>

    {#if !impactLoaded}
      <div class="muted-small">loading...</div>
    {:else if !impact.target}
      <div class="muted-small">No symbol_query rows yet — type a symbol above + Run, or invoke the agent's `symbol_impact` tool elsewhere.</div>
    {:else}
      <div class="impact-summary">
        <div class="impact-row">
          <span class="impact-label">Target</span>
          <span class="impact-value mono">{impact.target}</span>
        </div>
        <div class="impact-row">
          <button
            class="b33-visualize"
            type="button"
            onclick={() => visualizeImpact(impact.target)}
            title="Open in Graph window with red-gradient blast-radius overlay"
          >Visualize blast radius</button>
        </div>
        <div class="impact-row">
          <span class="impact-label">Total files impacted</span>
          <span class="impact-value">{impact.total_files}</span>
        </div>
        <div class="impact-row">
          <span class="impact-label">Unresolved edges</span>
          <span class="impact-value">{impact.unresolved_pct.toFixed(1)}%</span>
        </div>
        {#if impact.hop_bands.length > 0}
          <div class="impact-hop-table">
            <div class="hop-header">
              <span>Hop</span>
              <span>Files</span>
            </div>
            {#each impact.hop_bands as band}
              <div class="hop-row">
                <span class="hop-depth">{band.depth}</span>
                <span class="hop-files">
                  {#if band.files.length === 0}
                    <span class="muted-small">(per-band file list materialised via /symbol_impact tool)</span>
                  {:else}
                    {band.files.length} file{band.files.length === 1 ? '' : 's'}
                  {/if}
                </span>
              </div>
            {/each}
          </div>
        {/if}
      </div>
    {/if}
  </Card>

</div>

<style>
  .knowledge-drawer {
    display: flex;
    flex-direction: column;
    gap: 8px;
  }

  .impact-summary {
    display: flex;
    flex-direction: column;
    gap: 6px;
  }
  .impact-row {
    display: flex;
    justify-content: space-between;
    align-items: baseline;
    font-size: 12px;
  }
  .impact-label {
    color: var(--text-secondary);
  }
  .b33-visualize {
    padding: 4px 10px;
    background: transparent;
    border: 1px solid var(--teal-line);
    color: var(--teal);
    border-radius: var(--r-sm);
    font-family: var(--font-mono);
    font-size: 10px;
    cursor: pointer;
  }
  .b33-visualize:hover {
    background: var(--teal-ghost);
    color: var(--teal-bright);
  }
  /* C-24 (Task 3) — /symbol_impact runner row. */
  .impact-runner {
    display: flex;
    gap: 6px;
    margin-bottom: 6px;
  }
  .impact-runner__input {
    flex: 1;
    background: var(--bg-input);
    border: 1px solid var(--border-subtle);
    border-radius: var(--r-xs);
    color: var(--text-primary);
    font-size: 11px;
    padding: 3px 8px;
    outline: none;
    transition: border-color var(--dur-fast) var(--ease-std);
  }
  .impact-runner__input:focus { border-color: var(--teal); }
  .impact-runner__input::placeholder { color: var(--text-ghost); }
  .impact-runner__input:disabled { opacity: 0.6; }
  .impact-runner__btn {
    padding: 3px 10px;
    background: transparent;
    border: 1px solid var(--teal-line);
    color: var(--teal);
    border-radius: var(--r-xs);
    font-family: var(--font-mono);
    font-size: 10px;
    cursor: pointer;
    white-space: nowrap;
  }
  .impact-runner__btn:hover:not(:disabled) {
    background: var(--teal-ghost);
    color: var(--teal-bright);
  }
  .impact-runner__btn:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }
  .impact-value {
    color: var(--text-primary);
    font-weight: 500;
  }
  .impact-value.mono {
    font-family: var(--font-mono);
    font-size: 10px;
  }
  .impact-hop-table {
    margin-top: 6px;
    display: flex;
    flex-direction: column;
    gap: 2px;
    border-top: 1px solid var(--surface-divider);
    padding-top: 6px;
  }
  .hop-header {
    display: flex;
    justify-content: space-between;
    font-family: var(--font-mono);
    font-size: 10px;
    color: var(--text-secondary);
    letter-spacing: 0.05em;
  }
  .hop-row {
    display: flex;
    justify-content: space-between;
    font-size: 12px;
  }
  .hop-depth {
    font-family: var(--font-mono);
    color: var(--text-secondary);
  }
  .hop-files {
    color: var(--text-primary);
  }
  .muted-small {
    color: var(--text-secondary);
    font-size: 10px;
    font-style: italic;
  }

  .section-label {
    font-family: var(--font-mono);
    font-size: 10px;
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
    font-size: 10px;
    color: var(--text-primary);
    width: 56px;
    flex-shrink: 0;
  }

  .index-metric {
    font-family: var(--font-mono);
    font-size: 10px;
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
    font-size: 10px;
    color: var(--text-ghost);
  }

  .trace-latency {
    font-family: var(--font-mono);
    font-size: 10px;
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
    font-size: 10px;
    color: var(--text-primary);
  }

  .lang-failed {
    font-family: var(--font-mono);
    font-size: 10px;
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

  /* E-17 SP4: RAG retrieval-quality cards. */
  .rag-stat-row {
    display: flex;
    justify-content: space-between;
    align-items: baseline;
    gap: 8px;
    font-family: var(--font-mono);
    font-size: 10px;
    margin-bottom: 4px;
  }
  .rag-stat-label {
    color: var(--text-secondary);
    letter-spacing: 0.05em;
    text-transform: uppercase;
    flex-shrink: 0;
  }
  .rag-stat-value {
    color: var(--text-primary);
    text-align: right;
  }
  .rag-chip-strip {
    display: flex;
    flex-wrap: wrap;
    gap: 4px;
    margin-top: 6px;
  }
  .rag-chip {
    font-family: var(--font-mono);
    font-size: 10px;
    color: var(--text-secondary);
    background: var(--bg-input);
    border-radius: var(--r-xs);
    padding: 1px 5px;
    display: inline-flex;
    align-items: baseline;
    gap: 4px;
  }
  .rag-chip-meta {
    color: var(--text-primary);
  }
  .rag-event-list {
    display: flex;
    flex-direction: column;
    gap: 3px;
    margin-top: 6px;
  }
  .rag-event-row {
    display: flex;
    align-items: center;
    gap: 8px;
    font-family: var(--font-mono);
    font-size: 10px;
  }
  .rag-event-ts {
    color: var(--text-ghost);
    width: 64px;
    flex-shrink: 0;
  }
  .rag-event-meta {
    color: var(--text-secondary);
    margin-left: auto;
  }
  .rag-weight-table {
    margin-top: 6px;
    display: flex;
    flex-direction: column;
    gap: 2px;
    border-top: 1px solid var(--surface-divider);
    padding-top: 6px;
  }
  .rag-weight-header,
  .rag-weight-row {
    display: grid;
    grid-template-columns: 72px 1fr 1fr 1fr;
    font-family: var(--font-mono);
    font-size: 10px;
    gap: 6px;
  }
  .rag-weight-header {
    color: var(--text-secondary);
    letter-spacing: 0.05em;
    text-transform: uppercase;
  }
  .rag-weight-ts {
    color: var(--text-ghost);
  }
  .rag-weight-cell {
    color: var(--text-primary);
  }
  .rag-skeleton {
    font-family: var(--font-mono);
    font-size: 10px;
    color: var(--text-ghost);
    font-style: italic;
    padding: 4px 0;
  }
  .rag-error-banner {
    display: flex;
    justify-content: space-between;
    align-items: center;
    gap: 8px;
    padding: 4px 6px;
    background: color-mix(in srgb, var(--fail) 8%, transparent);
    border: 1px solid color-mix(in srgb, var(--fail) 30%, transparent);
    border-radius: var(--r-xs);
  }
  .rag-error-text {
    font-family: var(--font-mono);
    font-size: 10px;
    color: var(--fail);
    flex: 1;
    min-width: 0;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }
  .rag-retry-btn {
    padding: 2px 8px;
    background: transparent;
    border: 1px solid var(--teal-line);
    color: var(--teal);
    border-radius: var(--r-xs);
    font-family: var(--font-mono);
    font-size: 10px;
    cursor: pointer;
    flex-shrink: 0;
  }
  .rag-retry-btn:hover {
    background: var(--teal-ghost);
    color: var(--teal-bright);
  }
</style>
