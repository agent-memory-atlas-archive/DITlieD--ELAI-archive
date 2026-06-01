<script lang="ts">
  // D-27 closure (night 2026-05-30): NoveltyBreakdown.
  //
  // Compact inline widget showing per-class novelty distribution as
  // horizontal bars with computed percentages. Polls
  // `get_novelty_breakdown` every `pollMs` ms (default 10s, matches the
  // sibling D-15 NoveltyPanel + D-27 SycophancyFlagStrip cadence).
  //
  // Color semantics mirror the D-15 NoveltyPanel pie chart:
  //   * search             -> green  (#4a8)
  //   * distant_search     -> purple (#a8f)
  //   * discovery_candidate-> orange (#f93)
  //
  // Read-only -- pure projection over the orchestrator
  // `novelty::novelty_distribution` helper. No producer.
  //
  // Used inline (NOT a drawer) -- composable into ModeRoutingDashboardView
  // alongside RefusalRateGauge + SycophancyFlagStrip, or any future
  // research-mode side panel. Mirrors the SycophancyFlagStrip pattern
  // from D-27 GAP-1.
  //
  // Closes cross-check gap noted in workflow/sessions/s-night-2026-05-30/
  // cross-check/D/D-09_d26_d27_d28.md TASK-D27-003 + TASK-D27-005.
  //
  // Grounding: arXiv:2406.05761 (LLM novelty detection -- three-bucket
  // taxonomy: search vs distant_search vs discovery_candidate);
  // docs/specs/guide/real-problem-solving-grounded-2026-05-19.md
  // section 1 Mode C + section 4.8.

  import { onMount, onDestroy } from 'svelte';
  import type {
    DiscoveryCandidateStatus,
    NoveltyClassifiedRow,
    RecentNovelty,
  } from '$lib/api/types';

  type NoveltyBreakdown = {
    total: number;
    search: number;
    distant_search: number;
    discovery_candidate: number;
    search_pct: number;
    distant_search_pct: number;
    discovery_candidate_pct: number;
    dominant_class: string | null;
    limit_applied: number;
  };

  type CandidateStatusCounts = {
    queued: number;
    ratified: number;
    rejected: number;
  };

  interface Props {
    limit?: number;
    pollMs?: number;
  }
  const { limit = 50, pollMs = 10_000 }: Props = $props();

  const CLASS_LABELS: Record<string, string> = {
    search: 'SEARCH',
    distant_search: 'DISTANT',
    discovery_candidate: 'DISCOVERY',
  };

  let snapshot = $state<NoveltyBreakdown>({
    total: 0,
    search: 0,
    distant_search: 0,
    discovery_candidate: 0,
    search_pct: 0,
    distant_search_pct: 0,
    discovery_candidate_pct: 0,
    dominant_class: null,
    limit_applied: limit,
  });
  // F4 (D-15 enrichment): per-status counts for the discovery-candidate
  // lifecycle, aggregated client-side from the get_novelty_recent rows.
  // Wire-default zero; the badge cluster is hidden when the discovery
  // bucket is empty so the panel stays compact on cold-start sessions.
  let candidateStatusCounts = $state<CandidateStatusCounts>({
    queued: 0,
    ratified: 0,
    rejected: 0,
  });
  let loaded = $state(false);
  let loadError = $state<string | null>(null);
  let pollHandle: ReturnType<typeof setInterval> | null = null;

  async function callInvoke<T>(
    cmd: string,
    args?: Record<string, unknown>,
  ): Promise<T | undefined> {
    if (typeof window === 'undefined') return undefined;
    const tauriAvail = !!(window as unknown as { __TAURI_INTERNALS__?: unknown })
      .__TAURI_INTERNALS__;
    if (!tauriAvail) return undefined;
    try {
      const mod = await import('@tauri-apps/api/core');
      return (await mod.invoke<T>(cmd, args)) as T;
    } catch (e) {
      const msg = e instanceof Error ? e.message : String(e);
      console.warn(`[novelty-breakdown] invoke('${cmd}') failed`, e);
      loadError = msg;
      return undefined;
    }
  }

  function aggregateCandidateStatus(
    rows: NoveltyClassifiedRow[],
  ): CandidateStatusCounts {
    const counts: CandidateStatusCounts = { queued: 0, ratified: 0, rejected: 0 };
    for (const r of rows) {
      const s = r.candidate_status;
      if (s === 'queued' || s === 'ratified' || s === 'rejected') {
        counts[s] += 1;
      }
    }
    return counts;
  }

  async function reload() {
    const out = await callInvoke<NoveltyBreakdown>('get_novelty_breakdown', {
      limit,
    });
    if (out !== undefined) {
      snapshot = out;
      loadError = null;
    }
    // F4 (D-15 enrichment): pull per-status counts from the recent-rows
    // feed. Failure is non-fatal — the badge cluster simply stays at
    // zeros and the rest of the panel renders normally.
    const recent = await callInvoke<RecentNovelty>('get_novelty_recent', {
      limit,
    });
    if (recent !== undefined) {
      candidateStatusCounts = aggregateCandidateStatus(recent.rows);
    }
    loaded = true;
  }

  function statusBadgeClass(s: DiscoveryCandidateStatus): string {
    // Style consistent with ContextStatePanel's badge--{kind} pattern.
    return `nb-status-badge nb-status-${s}`;
  }

  function fmtPct(p: number): string {
    if (!Number.isFinite(p)) return '—';
    return `${p.toFixed(1)}%`;
  }

  function barWidth(p: number): string {
    if (!Number.isFinite(p)) return '0%';
    const clamped = Math.max(0, Math.min(100, p));
    return `${clamped}%`;
  }

  function dominantLabel(cls: string | null): string {
    if (cls === null) return '—';
    return CLASS_LABELS[cls] ?? cls.toUpperCase();
  }

  onMount(() => {
    reload();
    pollHandle = setInterval(reload, pollMs);
  });

  onDestroy(() => {
    if (pollHandle) {
      clearInterval(pollHandle);
      pollHandle = null;
    }
  });
</script>

<div class="nb-panel">
  <header class="nb-header">
    <strong>NOVELTY</strong>
    <span class="nb-dominant" class:nb-dom-search={snapshot.dominant_class === 'search'} class:nb-dom-distant={snapshot.dominant_class === 'distant_search'} class:nb-dom-discovery={snapshot.dominant_class === 'discovery_candidate'}>
      {dominantLabel(snapshot.dominant_class)}
    </span>
    <span class="nb-muted">{snapshot.total} classified</span>
  </header>

  {#if !loaded}
    <p class="nb-muted">Loading…</p>
  {:else if loadError !== null}
    <p class="nb-error">Error: {loadError}</p>
  {:else if snapshot.total === 0}
    <p class="nb-muted">No novelty classifications recorded yet.</p>
  {:else}
    <div class="nb-row">
      <span class="nb-label">SEARCH</span>
      <div class="nb-bar-wrap"><div class="nb-bar nb-bar-search" style="width: {barWidth(snapshot.search_pct)}"></div></div>
      <span class="nb-count">{snapshot.search}</span>
      <span class="nb-pct">{fmtPct(snapshot.search_pct)}</span>
    </div>
    <div class="nb-row">
      <span class="nb-label">DISTANT</span>
      <div class="nb-bar-wrap"><div class="nb-bar nb-bar-distant" style="width: {barWidth(snapshot.distant_search_pct)}"></div></div>
      <span class="nb-count">{snapshot.distant_search}</span>
      <span class="nb-pct">{fmtPct(snapshot.distant_search_pct)}</span>
    </div>
    <div class="nb-row">
      <span class="nb-label">DISCOVERY</span>
      <div class="nb-bar-wrap"><div class="nb-bar nb-bar-discovery" style="width: {barWidth(snapshot.discovery_candidate_pct)}"></div></div>
      <span class="nb-count">{snapshot.discovery_candidate}</span>
      <span class="nb-pct">{fmtPct(snapshot.discovery_candidate_pct)}</span>
    </div>
    {#if snapshot.discovery_candidate > 0 && (candidateStatusCounts.queued + candidateStatusCounts.ratified + candidateStatusCounts.rejected) > 0}
      <div class="nb-status-row" title="Discovery candidate lifecycle counts (queued / ratified / rejected) from recent rows">
        <span class="nb-label">STATUS</span>
        <div class="nb-status-cluster">
          <span class={statusBadgeClass('queued')}>queued {candidateStatusCounts.queued}</span>
          <span class={statusBadgeClass('ratified')}>ratified {candidateStatusCounts.ratified}</span>
          <span class={statusBadgeClass('rejected')}>rejected {candidateStatusCounts.rejected}</span>
        </div>
      </div>
    {/if}
  {/if}
</div>

<style>
  .nb-panel {
    padding: 0.5rem 0.75rem;
    font-family: ui-monospace, SFMono-Regular, Menlo, Consolas, monospace;
    font-size: 0.8rem;
  }

  .nb-header {
    display: flex;
    align-items: baseline;
    gap: 0.6rem;
    margin-bottom: 0.4rem;
  }

  .nb-header strong {
    letter-spacing: 0.08em;
  }

  .nb-dominant {
    display: inline-block;
    padding: 0.05rem 0.35rem;
    border-radius: 0.2rem;
    font-size: 0.7rem;
    letter-spacing: 0.05em;
    background: rgba(140, 140, 140, 0.2);
    color: var(--muted, #aaa);
  }

  .nb-dom-search {
    background: rgba(74, 136, 102, 0.25);
    color: #4a8;
  }

  .nb-dom-distant {
    background: rgba(170, 136, 255, 0.25);
    color: #a8f;
  }

  .nb-dom-discovery {
    background: rgba(255, 153, 51, 0.25);
    color: #f93;
  }

  .nb-muted {
    color: var(--muted, #888);
  }

  .nb-error {
    color: #c66;
  }

  .nb-row {
    display: grid;
    grid-template-columns: 5.5rem 1fr 2.5rem 3rem;
    gap: 0.5rem;
    align-items: center;
    margin-top: 0.25rem;
  }

  .nb-label {
    font-size: 0.7rem;
    letter-spacing: 0.05em;
    color: var(--muted, #aaa);
  }

  .nb-bar-wrap {
    height: 0.6rem;
    background: var(--bar-bg, #1a1a1a);
    border: 1px solid var(--border, #2a2a2a);
    overflow: hidden;
  }

  .nb-bar {
    height: 100%;
    transition: width 200ms ease-out;
  }

  .nb-bar-search {
    background: #4a8;
  }

  .nb-bar-distant {
    background: #a8f;
  }

  .nb-bar-discovery {
    background: #f93;
  }

  .nb-count {
    text-align: right;
    color: var(--text, #ddd);
  }

  .nb-pct {
    text-align: right;
    color: var(--muted, #aaa);
    font-size: 0.75rem;
  }

  /* F4 (D-15 enrichment): candidate-status badge cluster. Layout
     mirrors .nb-row but spans the bar + count + pct columns so the
     three badges can flow without crowding. */
  .nb-status-row {
    display: grid;
    grid-template-columns: 5.5rem 1fr;
    gap: 0.5rem;
    align-items: center;
    margin-top: 0.3rem;
  }

  .nb-status-cluster {
    display: flex;
    flex-wrap: wrap;
    gap: 0.35rem;
  }

  .nb-status-badge {
    display: inline-block;
    padding: 0.05rem 0.35rem;
    border-radius: 0.2rem;
    font-size: 0.65rem;
    letter-spacing: 0.05em;
    background: rgba(140, 140, 140, 0.2);
    color: var(--muted, #aaa);
  }

  .nb-status-queued {
    background: rgba(74, 136, 168, 0.25);
    color: #4a9;
  }

  .nb-status-ratified {
    background: rgba(74, 168, 102, 0.25);
    color: #6c8;
  }

  .nb-status-rejected {
    background: rgba(198, 102, 102, 0.25);
    color: #c66;
  }
</style>
