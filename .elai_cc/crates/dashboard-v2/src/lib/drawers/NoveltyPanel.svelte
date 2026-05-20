<script lang="ts">
  // D-15 — NoveltyPanel.
  //
  // Three sections:
  //   1. Per-class distribution (search / distant_search / discovery_candidate)
  //      sourced from `get_novelty_recent` (no second call — the response
  //      carries both rows + distribution).
  //   2. Pie-chart-style summary using inline SVG (the same coarse
  //      geometry the other D-panels use, no chart library dependency).
  //   3. Recent classifications table.
  //
  // No producer logic lives here — `agent_pool::dispatch_novelty` emits
  // the SSB row server-side when the D-10 router routes a retained
  // Mode A or Mode D result. Polls on a 10s cadence matching the other
  // D-{1..16} panels.
  //
  // Grounding: arXiv:2406.05761 (LLM novelty detection),
  // docs/specs/guide/real-problem-solving-grounded-2026-05-19.md
  // §1 Mode C + §4.8.

  import { onMount, onDestroy } from 'svelte';

  type NoveltyClassifiedRow = {
    template_id: string;
    class: string;
    dkg_match_count: number;
    near_duplicate_threshold: number;
    ts_ms: number;
  };

  type NoveltyDistribution = {
    total: number;
    search: number;
    distant_search: number;
    discovery_candidate: number;
  };

  type RecentNovelty = {
    rows: NoveltyClassifiedRow[];
    distribution: NoveltyDistribution;
  };

  const CLASS_KEYS: ReadonlyArray<keyof Omit<NoveltyDistribution, 'total'>> = [
    'search',
    'distant_search',
    'discovery_candidate',
  ];

  const CLASS_LABELS: Record<string, string> = {
    search: 'SEARCH',
    distant_search: 'DISTANT',
    discovery_candidate: 'DISCOVERY',
  };

  const CLASS_COLORS: Record<string, string> = {
    search: '#4a8',
    distant_search: '#a8f',
    discovery_candidate: '#f93',
  };

  let recent = $state<RecentNovelty>({
    rows: [],
    distribution: {
      total: 0,
      search: 0,
      distant_search: 0,
      discovery_candidate: 0,
    },
  });
  let loaded = $state(false);
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
      console.warn(`[novelty] invoke('${cmd}') failed`, e);
      return undefined;
    }
  }

  async function reload() {
    const r = await callInvoke<RecentNovelty>('get_novelty_recent', { limit: 50 });
    if (r !== undefined) recent = r;
    loaded = true;
  }

  function pct(k: keyof Omit<NoveltyDistribution, 'total'>): number {
    if (recent.distribution.total === 0) return 0;
    return Math.round((recent.distribution[k] / recent.distribution.total) * 100);
  }

  function classCount(k: keyof Omit<NoveltyDistribution, 'total'>): number {
    return recent.distribution[k];
  }

  function shortId(id: string): string {
    if (id.length <= 16) return id;
    return `${id.slice(0, 8)}...${id.slice(-4)}`;
  }

  function fmtTs(ms: number | null | undefined): string {
    if (!ms || ms <= 0) return '-';
    try {
      return new Date(ms).toISOString().slice(0, 19).replace('T', ' ');
    } catch {
      return '-';
    }
  }

  function classChipClass(k: string): string {
    switch (k) {
      case 'search':
        return 'chip-search';
      case 'distant_search':
        return 'chip-distant';
      case 'discovery_candidate':
        return 'chip-discovery';
      default:
        return 'chip-unknown';
    }
  }

  // Compute pie-chart-style stacked bar segments as an array of
  // { offset, width, color, key }.
  function segments(): Array<{ offset: number; width: number; color: string; key: string }> {
    const total = recent.distribution.total;
    if (total === 0) return [];
    let offset = 0;
    const out: Array<{ offset: number; width: number; color: string; key: string }> = [];
    for (const k of CLASS_KEYS) {
      const w = recent.distribution[k] / total;
      if (w > 0) {
        out.push({ offset, width: w, color: CLASS_COLORS[k], key: k });
        offset += w;
      }
    }
    return out;
  }

  onMount(() => {
    reload();
    pollHandle = setInterval(reload, 10_000);
  });

  onDestroy(() => {
    if (pollHandle) {
      clearInterval(pollHandle);
      pollHandle = null;
    }
  });
</script>

<div class="panel">
  <h2>Novelty Classification</h2>

  {#if !loaded}
    <p class="muted">Loading...</p>
  {:else}
    <section class="distribution">
      <header>
        <strong>Class distribution</strong>
        <span class="muted">{recent.distribution.total} classifications</span>
      </header>
      {#if recent.distribution.total === 0}
        <p class="muted">No novelty_classified rows recorded.</p>
      {:else}
        <div class="stacked-bar" role="img" aria-label="Novelty class distribution">
          {#each segments() as seg}
            <div
              class="stacked-seg"
              style="left: {seg.offset * 100}%; width: {seg.width * 100}%; background: {seg.color};"
              title="{CLASS_LABELS[seg.key] ?? seg.key}: {Math.round(seg.width * 100)}%"
            ></div>
          {/each}
        </div>
        <table>
          <thead>
            <tr><th>Class</th><th>Count</th><th>%</th><th>Bar</th></tr>
          </thead>
          <tbody>
            {#each CLASS_KEYS as k}
              <tr>
                <td><span class={classChipClass(k)}>{CLASS_LABELS[k] ?? k}</span></td>
                <td>{classCount(k)}</td>
                <td>{pct(k)}%</td>
                <td>
                  <div class="bar" style="width: {pct(k)}%; background: {CLASS_COLORS[k]};"></div>
                </td>
              </tr>
            {/each}
          </tbody>
        </table>
      {/if}
    </section>

    <section class="recent">
      <header>
        <strong>Recent classifications</strong>
        <span class="muted">{recent.rows.length} rows</span>
      </header>
      {#if recent.rows.length === 0}
        <p class="muted">No novelty_classified rows recorded.</p>
      {:else}
        <table>
          <thead>
            <tr>
              <th>When</th>
              <th>Template</th>
              <th>Class</th>
              <th>DKG matches</th>
              <th>Near-dup</th>
            </tr>
          </thead>
          <tbody>
            {#each recent.rows as row}
              <tr>
                <td>{fmtTs(row.ts_ms)}</td>
                <td><code>{shortId(row.template_id)}</code></td>
                <td><span class={classChipClass(row.class)}>{CLASS_LABELS[row.class] ?? row.class}</span></td>
                <td>{row.dkg_match_count}</td>
                <td>{row.near_duplicate_threshold.toFixed(3)}</td>
              </tr>
            {/each}
          </tbody>
        </table>
      {/if}
    </section>
  {/if}
</div>

<style>
  .panel {
    padding: 1rem;
    font-family: ui-monospace, SFMono-Regular, Menlo, Consolas, monospace;
    font-size: 0.85rem;
  }

  h2 {
    margin: 0 0 0.75rem 0;
    font-size: 1rem;
    text-transform: uppercase;
    letter-spacing: 0.08em;
  }

  section {
    margin-top: 1rem;
  }

  section > header {
    display: flex;
    gap: 0.75rem;
    align-items: baseline;
    margin-bottom: 0.5rem;
  }

  .muted {
    color: var(--muted, #888);
  }

  table {
    border-collapse: collapse;
    width: 100%;
  }

  th, td {
    padding: 0.35rem 0.5rem;
    text-align: left;
    border-bottom: 1px solid var(--border, #2a2a2a);
    vertical-align: middle;
  }

  .bar {
    height: 8px;
    border-radius: 2px;
  }

  .stacked-bar {
    position: relative;
    width: 100%;
    height: 14px;
    background: var(--border, #2a2a2a);
    border-radius: 3px;
    overflow: hidden;
    margin-bottom: 0.75rem;
  }

  .stacked-seg {
    position: absolute;
    top: 0;
    bottom: 0;
  }

  .chip-search {
    color: #4a8;
  }
  .chip-distant {
    color: #a8f;
  }
  .chip-discovery {
    color: #f93;
  }
  .chip-unknown {
    color: var(--muted, #888);
  }

  code {
    font-family: inherit;
    font-size: inherit;
  }
</style>
