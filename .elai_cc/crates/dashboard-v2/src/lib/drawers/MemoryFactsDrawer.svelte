<script lang="ts">
  // E-5 — User modeling + long-term memory drawer.
  //
  // Read-only surface that lists currently-valid `memory_facts` rows
  // grouped by fact_type, with trust_score + the per-fact-type
  // retrieval-threshold marker so the user can see at-a-glance which
  // rows would actually be retrieved for the next prompt assembly.
  //
  // Mutation surface (forget / promote / contradiction adjudicate)
  // reserved for a follow-up E-5 vertical slice (per the executor
  // brief).

  import { onMount, onDestroy } from 'svelte';
  import { invoke } from '@tauri-apps/api/core';

  type MemoryFactDto = {
    fact_id: string;
    user_id: string;
    project_id: string | null;
    scope: string;
    fact_type: string;
    key: string;
    value: unknown;
    confidence: number;
    trust_score: number;
    trust_threshold: number;
    ingested_at_ms: number;
    valid_from_ms: number;
    valid_to_ms: number | null;
    decay_rate_per_day: number;
    last_observed_at_ms: number;
    evidence_pointers: string[];
    superseded_by: string | null;
  };

  type MemoryFactsResponse = {
    store_path: string;
    store_just_created: boolean;
    facts: MemoryFactDto[];
  };

  let storePath = $state<string>('');
  let storeJustCreated = $state(false);
  let facts = $state<MemoryFactDto[]>([]);
  let loaded = $state(false);
  let loadError = $state<string | null>(null);
  let scopeFilter = $state<string>('all');
  let typeFilter = $state<string>('all');
  let pollInterval: ReturnType<typeof setInterval> | null = null;

  const FACT_TYPES = [
    'voice_style',
    'expertise',
    'risk_tol',
    'opinion',
    'preference',
    'domain_knowledge',
    'session_pattern',
    'mistake',
  ];

  async function reload() {
    try {
      const res = await invoke<MemoryFactsResponse>('get_memory_facts', {
        userId: null,
        scope: scopeFilter === 'all' ? null : scopeFilter,
        factType: typeFilter === 'all' ? null : typeFilter,
        limit: 200,
      });
      storePath = res.store_path;
      storeJustCreated = res.store_just_created;
      facts = res.facts;
      loadError = null;
    } catch (e) {
      loadError = String(e);
    } finally {
      loaded = true;
    }
  }

  $effect(() => {
    // Refire on filter changes.
    void scopeFilter;
    void typeFilter;
    reload();
  });

  onMount(() => {
    reload();
    pollInterval = setInterval(reload, 15_000);
  });

  onDestroy(() => {
    if (pollInterval !== null) clearInterval(pollInterval);
  });

  function bandFor(fact: MemoryFactDto): 'above' | 'near' | 'below' {
    if (fact.trust_score >= fact.trust_threshold + 0.1) return 'above';
    if (fact.trust_score >= fact.trust_threshold) return 'near';
    return 'below';
  }

  function humanizeRelative(ms: number): string {
    if (!ms) return 'never';
    const delta = Math.max(0, Date.now() - ms);
    const secs = Math.floor(delta / 1000);
    if (secs < 60) return `${secs}s ago`;
    const mins = Math.floor(secs / 60);
    if (mins < 60) return `${mins} min ago`;
    const hrs = Math.floor(mins / 60);
    if (hrs < 24) return `${hrs}h ago`;
    const days = Math.floor(hrs / 24);
    if (days < 30) return `${days}d ago`;
    const months = Math.floor(days / 30);
    if (months < 12) return `${months}mo ago`;
    return `${Math.floor(months / 12)}y ago`;
  }

  function summarizeValue(v: unknown): string {
    if (v === null || v === undefined) return '—';
    if (typeof v === 'string') return v;
    if (typeof v === 'number' || typeof v === 'boolean') return String(v);
    try {
      const s = JSON.stringify(v);
      return s.length > 120 ? s.slice(0, 117) + '…' : s;
    } catch {
      return '[unserializable]';
    }
  }

  function groupedByType(rows: MemoryFactDto[]): Array<{ type: string; rows: MemoryFactDto[] }> {
    const buckets = new Map<string, MemoryFactDto[]>();
    for (const r of rows) {
      const list = buckets.get(r.fact_type) ?? [];
      list.push(r);
      buckets.set(r.fact_type, list);
    }
    // Preserve canonical ordering from FACT_TYPES, then any leftover types.
    const out: Array<{ type: string; rows: MemoryFactDto[] }> = [];
    for (const t of FACT_TYPES) {
      const xs = buckets.get(t);
      if (xs && xs.length > 0) out.push({ type: t, rows: xs });
    }
    for (const [k, v] of buckets) {
      if (!FACT_TYPES.includes(k)) out.push({ type: k, rows: v });
    }
    return out;
  }
</script>

<div class="memory-drawer">
  <div class="header">
    <div class="section-label">MEMORY FACTS (E-5)</div>
    <div class="filters">
      <label>
        scope
        <select bind:value={scopeFilter}>
          <option value="all">all</option>
          <option value="project">project</option>
          <option value="global">global</option>
        </select>
      </label>
      <label>
        type
        <select bind:value={typeFilter}>
          <option value="all">all</option>
          {#each FACT_TYPES as t}
            <option value={t}>{t}</option>
          {/each}
        </select>
      </label>
      <button class="refresh-btn" type="button" onclick={reload}>refresh</button>
    </div>
  </div>

  <div class="store-meta">
    store: <span class="mono">{storePath || '(unresolved)'}</span>
    {#if storeJustCreated}
      <span class="ghost">(just created)</span>
    {/if}
  </div>

  {#if !loaded}
    <div class="muted-small">loading…</div>
  {:else if loadError}
    <div class="muted-small error">load failed: {loadError}</div>
  {:else if facts.length === 0}
    <div class="muted-small">
      No facts yet. The Mem0-style extractor populates this drawer once a
      preference / opinion / expertise signal is detected in a chat turn.
    </div>
  {:else}
    {#each groupedByType(facts) as group}
      <div class="group">
        <div class="group-head">
          <span class="type-tag">{group.type}</span>
          <span class="count-pill">{group.rows.length}</span>
        </div>
        <div class="rows">
          {#each group.rows as f}
            <div class="row band-{bandFor(f)}">
              <div class="row-head">
                <span class="key mono">{f.key}</span>
                <span class="scope-pill {f.scope}">{f.scope}</span>
              </div>
              <div class="value">{summarizeValue(f.value)}</div>
              <div class="row-meta">
                <span>
                  trust <strong>{f.trust_score.toFixed(2)}</strong>
                  / floor {f.trust_threshold.toFixed(2)}
                </span>
                <span>confidence {f.confidence.toFixed(2)}</span>
                <span>decay {f.decay_rate_per_day.toExponential(1)}/d</span>
                <span>seen {humanizeRelative(f.last_observed_at_ms)}</span>
                {#if f.evidence_pointers.length > 0}
                  <span class="ghost">evidence x{f.evidence_pointers.length}</span>
                {/if}
              </div>
            </div>
          {/each}
        </div>
      </div>
    {/each}
  {/if}
</div>

<style>
  .memory-drawer {
    display: flex;
    flex-direction: column;
    gap: 12px;
    padding: 12px;
    font-family: var(--font-sans);
    color: var(--fg);
  }
  .header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 12px;
  }
  .section-label {
    font-size: 11px;
    letter-spacing: 0.12em;
    color: var(--fg-muted, #888);
  }
  .filters {
    display: flex;
    align-items: center;
    gap: 8px;
    font-size: 12px;
  }
  .filters select {
    margin-left: 4px;
    background: transparent;
    color: var(--fg);
    border: 1px solid var(--border, #333);
    padding: 2px 4px;
  }
  .store-meta {
    font-size: 11px;
    color: var(--fg-muted, #888);
  }
  .store-meta .ghost {
    margin-left: 6px;
    color: var(--fg-muted, #666);
  }
  .mono {
    font-family: var(--font-mono);
  }
  .muted-small {
    font-size: 12px;
    color: var(--fg-muted, #888);
  }
  .muted-small.error {
    color: var(--danger, #e26d6d);
  }
  .group {
    border-top: 1px solid var(--border, #1f1f1f);
    padding-top: 8px;
  }
  .group-head {
    display: flex;
    align-items: center;
    gap: 8px;
    font-size: 12px;
    margin-bottom: 6px;
  }
  .type-tag {
    font-family: var(--font-mono);
    text-transform: uppercase;
    letter-spacing: 0.08em;
    color: var(--accent, #6dafe2);
  }
  .count-pill {
    padding: 0 6px;
    border: 1px solid var(--border, #333);
    border-radius: 8px;
    font-size: 10px;
    color: var(--fg-muted, #888);
  }
  .rows {
    display: flex;
    flex-direction: column;
    gap: 6px;
  }
  .row {
    padding: 6px 8px;
    background: var(--bg-elevated, #161616);
    border-left: 2px solid transparent;
  }
  .row.band-above {
    border-left-color: var(--ok, #6de29c);
  }
  .row.band-near {
    border-left-color: var(--warn, #e2c66d);
  }
  .row.band-below {
    border-left-color: var(--danger, #e26d6d);
  }
  .row-head {
    display: flex;
    justify-content: space-between;
    align-items: center;
    font-size: 12px;
  }
  .key {
    color: var(--fg, #ddd);
  }
  .scope-pill {
    font-size: 10px;
    text-transform: uppercase;
    padding: 0 6px;
    border-radius: 8px;
    background: var(--bg, #0d0d0d);
    color: var(--fg-muted, #888);
  }
  .scope-pill.global {
    color: var(--accent, #6dafe2);
  }
  .value {
    margin-top: 2px;
    font-size: 11px;
    color: var(--fg-muted, #aaa);
    font-family: var(--font-mono);
    overflow-wrap: anywhere;
  }
  .row-meta {
    margin-top: 4px;
    display: flex;
    flex-wrap: wrap;
    gap: 12px;
    font-size: 10px;
    color: var(--fg-muted, #888);
  }
  .row-meta strong {
    color: var(--fg, #ddd);
  }
  .row-meta .ghost {
    color: var(--fg-muted, #666);
  }
  .refresh-btn {
    background: transparent;
    color: var(--fg);
    border: 1px solid var(--border, #333);
    padding: 2px 8px;
    cursor: pointer;
    font-size: 11px;
  }
  .refresh-btn:hover {
    background: var(--bg-elevated, #161616);
  }
</style>
