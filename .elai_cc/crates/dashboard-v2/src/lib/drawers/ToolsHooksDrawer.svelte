<script lang="ts">
  import { onMount } from 'svelte';
  import { toolsHooksData } from '$lib/state/drawers.svelte';
  import Badge from '$lib/ui/Badge.svelte';
  import Divider from '$lib/ui/Divider.svelte';
  import UnwiredNote from '$lib/ui/UnwiredNote.svelte';
  import type { HookType, HookVerdict } from '$lib/api/types';

  onMount(() => {
    toolsHooksData.bootstrap();
    return toolsHooksData.teardown;
  });

  const NOW = Date.now();
  const hooks = $derived(toolsHooksData.hookExecutions);
  const sc = $derived(toolsHooksData.shortcoderStats);
  const lastTaxonomy = $derived(toolsHooksData.failureTaxonomy);
  const lastClassification = $derived(lastTaxonomy?.classification ?? 'NONE');

  // Tiered analysis static mock
  const tiers = [
    { label: 'T1', desc: 'deterministic <1ms', invocations: 284, median_ms: '0.3ms', allow: 278, warn: 4, block: 2 },
    { label: 'T2', desc: 'AST <10ms',          invocations: 91,  median_ms: '4.1ms', allow: 88,  warn: 2, block: 1 },
    { label: 'T3', desc: 'pattern <5ms',       invocations: 37,  median_ms: '2.8ms', allow: 35,  warn: 2, block: 0 },
  ];

  // Shortcoder rules
  const shortcoderRules: { label: string; chips: number }[] = [
    { label: 'Collapse empty match arms',  chips: 8  },
    { label: 'Inline single-use let',      chips: 12 },
    { label: 'Remove redundant clones',    chips: 7  },
    { label: 'Fold if-let chains',          chips: 5  },
    { label: 'Deduplicate imports',        chips: 9  },
    { label: 'Remove dead assignments',    chips: 4  },
    { label: 'Collapse nested options',    chips: 3  },
  ];

  // Hidden tools list — derived from toolsHooksData.hiddenTools (wired via get_tool_visibility in PART11).
  const hiddenTools = $derived(toolsHooksData.hiddenTools);

  // Failure-taxonomy categories — only rendered when toolsHooksData.unwired.taxonomyDistribution is false
  // (currently always true; kept here so the {:else} branch type-checks).
  const taxonomyCategories: { name: string; count: number }[] = [
    { name: 'LOGIC_ERROR',         count: 0 },
    { name: 'TYPE_MISMATCH',       count: 0 },
    { name: 'RESOURCE_EXHAUSTED',  count: 0 },
    { name: 'NETWORK_TIMEOUT',     count: 0 },
    { name: 'DEPENDENCY_MISSING',  count: 0 },
    { name: 'PERMISSION_DENIED',   count: 0 },
    { name: 'ASSERTION_FAILED',    count: 0 },
    { name: 'PANIC',               count: 0 },
    { name: 'UNCLASSIFIED',        count: 0 },
  ];

  function hookTone(verdict: HookVerdict): string {
    if (verdict === 'ALLOW') return 'teal';
    if (verdict === 'BLOCK') return 'fail';
    return 'warn';
  }

  function hookTypeTone(_hook: HookType): string {
    return 'steel';
  }

  function fmtAgo(ts: number): string {
    const diff = NOW - ts;
    const mins = Math.floor(diff / 60_000);
    if (mins < 60) return `${mins}m ago`;
    const hrs = Math.floor(mins / 60);
    return `${hrs}h ago`;
  }

  function tierBarWidth(n: number, total: number): string {
    return total > 0 ? `${Math.round((n / total) * 100)}%` : '0%';
  }
</script>

<div class="drawer">
  <!-- HOOKS FIRED -->
  <section class="section">
    <h3 class="section-header">HOOKS FIRED (recent)</h3>
    <div class="hook-list">
      {#each hooks.slice(0, 6) as h}
        <div class="hook-row">
          <Badge tone={hookTypeTone(h.hook)} size="xs">{h.hook}</Badge>
          <Badge tone={hookTone(h.verdict)} size="xs">{h.verdict}</Badge>
          <span class="tool-name">{h.tool ?? '—'}</span>
          <span class="ts-ghost">{fmtAgo(h.ts)}</span>
        </div>
      {/each}
    </div>
  </section>

  <Divider />

  <!-- TIERED ANALYSIS -->
  <section class="section">
    <h3 class="section-header">TIERED ANALYSIS</h3>
    {#if toolsHooksData.unwired.tieredAnalysis}
      <UnwiredNote
        phase="4C"
        feature="T1 / T2 / T3 invocation counts + ALLOW/WARN/BLOCK split per tier"
        note="Tiered hook stats not aggregated in SSB; current rows are synthetic"
      />
    {:else}
      <div class="tier-grid">
        {#each tiers as tier}
          <div class="tier-card">
            <div class="tier-top">
              <span class="tier-label">{tier.label}</span>
              <span class="tier-count">{tier.invocations}</span>
            </div>
            <span class="tier-desc">{tier.desc} · {tier.median_ms} median</span>
            <div class="tier-bar-wrap">
              <div class="tier-bar">
                <div class="tier-seg seg-allow" style="width:{tierBarWidth(tier.allow, tier.invocations)}"></div>
                <div class="tier-seg seg-warn"  style="width:{tierBarWidth(tier.warn,  tier.invocations)}"></div>
                <div class="tier-seg seg-block" style="width:{tierBarWidth(tier.block, tier.invocations)}"></div>
              </div>
            </div>
          </div>
        {/each}
      </div>
    {/if}
  </section>

  <Divider />

  <!-- SHORTCODER -->
  <section class="section">
    <h3 class="section-header">SHORTCODER</h3>
    <div class="sc-stat">
      <span class="sc-value">{sc.savings_pct}%</span>
      <span class="sc-label">avg token savings</span>
    </div>
    {#if toolsHooksData.unwired.shortcoderRules}
      <UnwiredNote
        phase="A-24"
        feature="per-rule hit counts (collapse / inline / fold / ...)"
        note="ShortCoder per-rule histogram not exposed; only the aggregate savings_pct comes from get_shortcoder_stats"
      />
    {:else}
      <div class="sc-rules">
        {#each shortcoderRules as rule}
          <div class="sc-rule-row">
            <span class="sc-rule-name">{rule.label}</span>
            <span class="sc-chip-count">{rule.chips}</span>
          </div>
        {/each}
      </div>
    {/if}
  </section>

  <Divider />

  <!-- EDIT MATCHING -->
  <section class="section">
    <h3 class="section-header">EDIT MATCHING</h3>
    <div class="em-rows">
      {#each [
        { label: 'exact',                count: sc.exact_hits },
        { label: 'whitespace-normalized', count: sc.whitespace_hits },
        { label: 'ellipsis-elision',      count: sc.ellipsis_hits },
      ] as em}
        <div class="em-row">
          <span class="em-label">{em.label}</span>
          <div class="em-bar-wrap">
            <div
              class="em-bar-fill"
              style="width:{Math.round((em.count / sc.total_edits) * 100)}%"
            ></div>
          </div>
          <span class="em-count">{em.count}</span>
        </div>
      {/each}
    </div>
  </section>

  <Divider />

  <!-- FAILURE TAXONOMY -->
  <section class="section">
    <h3 class="section-header">FAILURE TAXONOMY (last)</h3>
    <div class="tax-last">
      <span class="tax-last-label">last classification</span>
      <Badge tone="fail" size="xs">{lastClassification}</Badge>
    </div>
    {#if toolsHooksData.unwired.taxonomyDistribution}
      <UnwiredNote
        phase="4B"
        feature="per-category counts across the 9 FailureTaxonomy categories"
        note="Per-category histogram not aggregated; only the most-recent classification comes from get_failure_taxonomy_last"
      />
    {:else}
      <div class="tax-list">
        {#each taxonomyCategories as cat}
          <div class="tax-row">
            <span class="tax-name">{cat.name}</span>
            <span class="tax-count" class:tax-zero={cat.count === 0}>{cat.count}</span>
          </div>
        {/each}
      </div>
    {/if}
  </section>

  <Divider />

  <!-- TOOL FILTER -->
  <section class="section">
    <h3 class="section-header">TOOL FILTER</h3>
    {#if toolsHooksData.unwired.toolFilter}
      <UnwiredNote
        phase="4A"
        feature="per-agent allow/deny tool list"
        note="Backend AgentDispatchConfig is authoritative but not exposed via Tauri yet; chips above are hardcoded"
      />
    {:else}
      <div class="tool-chips">
        {#each hiddenTools as tool}
          <span class="tool-chip">{tool}</span>
        {/each}
      </div>
    {/if}
  </section>
</div>

<style>
  .drawer {
    display: flex;
    flex-direction: column;
    gap: 0;
    padding-bottom: var(--s-6);
  }

  .section {
    padding: var(--s-4) var(--s-4) var(--s-3);
  }

  .section-header {
    font-family: var(--font-mono);
    font-size: 11px;
    font-weight: 500;
    letter-spacing: 0.08em;
    color: var(--text-ghost);
    text-transform: uppercase;
    margin: 0 0 var(--s-3) 0;
  }

  /* HOOKS */
  .hook-list {
    display: flex;
    flex-direction: column;
    gap: 6px;
  }

  .hook-row {
    display: flex;
    align-items: center;
    gap: var(--s-2);
    flex-wrap: nowrap;
  }

  .tool-name {
    font-family: var(--font-mono);
    font-size: 10px;
    color: var(--text-ghost);
    flex: 1;
  }

  .ts-ghost {
    font-family: var(--font-mono);
    font-size: 10px;
    color: var(--text-ghost);
    white-space: nowrap;
  }

  /* TIERS */
  .tier-grid {
    display: flex;
    flex-direction: column;
    gap: var(--s-3);
  }

  .tier-card {
    background: var(--bg-surface);
    border: 1px solid var(--border-subtle);
    border-radius: var(--r-sm);
    padding: var(--s-2) var(--s-3);
    display: flex;
    flex-direction: column;
    gap: 4px;
  }

  .tier-top {
    display: flex;
    align-items: baseline;
    justify-content: space-between;
  }

  .tier-label {
    font-family: var(--font-mono);
    font-size: 11px;
    color: var(--text-ghost);
    letter-spacing: 0.06em;
  }

  .tier-count {
    font-family: var(--font-mono);
    font-size: 18px;
    color: var(--text-primary);
    font-weight: 500;
  }

  .tier-desc {
    font-family: var(--font-mono);
    font-size: 10px;
    color: var(--text-ghost);
  }

  .tier-bar-wrap {
    margin-top: 4px;
  }

  .tier-bar {
    display: flex;
    height: 4px;
    border-radius: var(--r-pill);
    overflow: hidden;
    background: var(--bg-input);
  }

  .tier-seg {
    height: 100%;
    transition: width var(--dur-base) var(--ease-out);
  }

  .seg-allow { background: var(--teal); }
  .seg-warn  { background: var(--warn); }
  .seg-block { background: var(--fail); }

  /* SHORTCODER */
  .sc-stat {
    display: flex;
    align-items: baseline;
    gap: var(--s-2);
    margin-bottom: var(--s-3);
  }

  .sc-value {
    font-family: var(--font-mono);
    font-size: 18px;
    color: var(--teal-bright);
    font-weight: 500;
  }

  .sc-label {
    font-family: var(--font-mono);
    font-size: 11px;
    color: var(--text-ghost);
  }

  .sc-rules {
    display: flex;
    flex-direction: column;
    gap: 5px;
  }

  .sc-rule-row {
    display: flex;
    align-items: center;
    justify-content: space-between;
  }

  .sc-rule-name {
    font-family: var(--font-mono);
    font-size: 11px;
    color: var(--text-secondary);
  }

  .sc-chip-count {
    font-family: var(--font-mono);
    font-size: 11px;
    color: var(--text-ghost);
  }

  /* EDIT MATCHING */
  .em-rows {
    display: flex;
    flex-direction: column;
    gap: 6px;
  }

  .em-row {
    display: flex;
    align-items: center;
    gap: var(--s-2);
  }

  .em-label {
    font-family: var(--font-mono);
    font-size: 10px;
    color: var(--text-ghost);
    width: 14ch;
    white-space: nowrap;
    flex-shrink: 0;
  }

  .em-bar-wrap {
    flex: 1;
    height: 4px;
    background: var(--bg-input);
    border-radius: var(--r-pill);
    overflow: hidden;
  }

  .em-bar-fill {
    height: 100%;
    background: var(--teal);
    border-radius: var(--r-pill);
    transition: width var(--dur-base) var(--ease-out);
  }

  .em-count {
    font-family: var(--font-mono);
    font-size: 10px;
    color: var(--text-ghost);
    width: 3ch;
    text-align: right;
  }

  /* FAILURE TAXONOMY */
  .tax-last {
    display: flex;
    align-items: center;
    gap: var(--s-2);
    margin-bottom: var(--s-2);
  }

  .tax-last-label {
    font-family: var(--font-mono);
    font-size: 11px;
    color: var(--text-ghost);
  }

  .tax-list {
    display: flex;
    flex-direction: column;
    gap: 4px;
  }

  .tax-row {
    display: flex;
    align-items: center;
    justify-content: space-between;
  }

  .tax-name {
    font-family: var(--font-mono);
    font-size: 10px;
    color: var(--text-secondary);
  }

  .tax-count {
    font-family: var(--font-mono);
    font-size: 11px;
    color: var(--text-primary);
  }

  .tax-zero {
    color: var(--text-ghost);
  }

  /* TOOL FILTER */
  .tool-chips {
    display: flex;
    flex-wrap: wrap;
    gap: var(--s-1);
  }

  .tool-chip {
    font-family: var(--font-mono);
    font-size: 10px;
    color: var(--teal);
    background: var(--teal-ghost);
    border: 1px solid var(--teal-line);
    border-radius: var(--r-pill);
    padding: 2px 8px;
  }
</style>
