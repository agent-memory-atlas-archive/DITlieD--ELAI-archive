<script lang="ts">
  // D-6 — HeldOutPanel.
  //
  // Read-only view of the time-windowed held-out task store + 9-gram
  // contamination filter status via the `get_held_out_recent` +
  // `get_held_out_stats` Tauri commands (additive — see
  // crates/dashboard-v2/src-tauri/src/commands_d6.rs).
  //
  // Renders:
  //   * top stats banner: total added / used / utilization / per-source counts
  //   * last 50 held_out_task_added rows (recent ingest first)
  //   * per-row: short task-id, source label, ngram count, ingest age (days),
  //     eligibility window (active / pending)
  //
  // No producer logic lives here — the orchestrator producer
  // (`held_out::insert_and_emit` + `held_out::mark_used_and_emit`) emits the
  // SSB rows separately. This panel polls on a 10s cadence matching the
  // other D-{1..5} panels.

  import { onMount, onDestroy } from 'svelte';

  type HeldOutTaskRow = {
    task_id: string;
    source: string;
    content_blake3: string;
    ngram_count: number;
    ingested_ts_ms: number;
    eligibility_window_ms: number;
  };

  type HeldOutStatsRow = {
    total_added: number;
    total_used: number;
    by_source: Record<string, number>;
    utilization_pct: number;
  };

  let rows = $state<HeldOutTaskRow[]>([]);
  let stats = $state<HeldOutStatsRow | null>(null);
  let loaded = $state(false);
  let loadError = $state<string | null>(null);
  let pollHandle: ReturnType<typeof setInterval> | null = null;

  async function callInvoke<T>(cmd: string, args?: Record<string, unknown>): Promise<T | undefined> {
    if (typeof window === 'undefined') return undefined;
    const tauriAvail = !!(window as unknown as { __TAURI_INTERNALS__?: unknown }).__TAURI_INTERNALS__;
    if (!tauriAvail) return undefined;
    try {
      const mod = await import('@tauri-apps/api/core');
      return (await mod.invoke<T>(cmd, args)) as T;
    } catch (e) {
      console.warn(`[held-out] invoke('${cmd}') failed`, e);
      return undefined;
    }
  }

  async function reload() {
    const out = await callInvoke<HeldOutTaskRow[]>('get_held_out_recent', { limit: 50 });
    const st = await callInvoke<HeldOutStatsRow>('get_held_out_stats');
    if (out === undefined) {
      rows = [];
    } else {
      rows = out;
    }
    if (st !== undefined) {
      stats = st;
    }
    loadError = null;
    loaded = true;
  }

  function shortId(s: string | null | undefined): string {
    if (!s) return '—';
    if (s.length <= 16) return s;
    return `${s.slice(0, 8)}…${s.slice(-4)}`;
  }

  function ageDays(ts: number): string {
    if (!Number.isFinite(ts) || ts <= 0) return '—';
    const ms = Date.now() - ts;
    if (ms < 0) return '0d';
    const days = Math.floor(ms / 86_400_000);
    if (days === 0) {
      const hours = Math.floor(ms / 3_600_000);
      return `${hours}h`;
    }
    return `${days}d`;
  }

  function eligibilityLabel(row: HeldOutTaskRow): string {
    if (row.eligibility_window_ms <= 0) return 'active';
    const ageMs = Date.now() - row.ingested_ts_ms;
    if (ageMs >= row.eligibility_window_ms) return 'active';
    const remainMs = row.eligibility_window_ms - ageMs;
    const days = Math.ceil(remainMs / 86_400_000);
    return `pending ${days}d`;
  }

  function utilizationPct(stats: HeldOutStatsRow | null): string {
    if (!stats) return '—';
    return `${(stats.utilization_pct * 100).toFixed(1)}%`;
  }

  function sourceList(stats: HeldOutStatsRow | null): Array<[string, number]> {
    if (!stats) return [];
    return Object.entries(stats.by_source).sort((a, b) => b[1] - a[1]);
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

<section class="held-out-panel">
  <header class="panel-header">
    <h2>Held-Out Tasks</h2>
    <button class="reload-btn" onclick={reload}>Reload</button>
  </header>

  {#if !loaded}
    <p class="muted">Loading held-out tasks…</p>
  {:else if loadError}
    <div class="error-banner">
      <strong>Failed to load held-out task rows.</strong>
      <pre>{loadError}</pre>
    </div>
  {:else}
    <div class="stats-row">
      <div class="stat">
        <div class="stat-label">Added</div>
        <div class="stat-value">{stats?.total_added ?? 0}</div>
      </div>
      <div class="stat">
        <div class="stat-label">Used</div>
        <div class="stat-value">{stats?.total_used ?? 0}</div>
      </div>
      <div class="stat">
        <div class="stat-label">Utilization</div>
        <div class="stat-value">{utilizationPct(stats)}</div>
      </div>
    </div>

    {#if sourceList(stats).length > 0}
      <div class="source-row">
        {#each sourceList(stats) as [src, count] (src)}
          <span class="source-chip">
            <span class="source-name">{src}</span>
            <span class="source-count">{count}</span>
          </span>
        {/each}
      </div>
    {/if}

    {#if rows.length === 0}
      <p class="muted">
        No held-out tasks recorded yet. Rows are emitted by the orchestrator
        after <code>held_out::insert_and_emit</code> ingests a task that
        passed the 9-gram contamination filter.
      </p>
    {:else}
      <table>
        <thead>
          <tr>
            <th>Task</th>
            <th>Source</th>
            <th>9-grams</th>
            <th>Age</th>
            <th>Window</th>
          </tr>
        </thead>
        <tbody>
          {#each rows as row (row.task_id)}
            <tr>
              <td><code>{shortId(row.task_id)}</code></td>
              <td>{row.source}</td>
              <td>{row.ngram_count}</td>
              <td>{ageDays(row.ingested_ts_ms)}</td>
              <td>
                {#if eligibilityLabel(row) === 'active'}
                  <span class="badge badge-active">active</span>
                {:else}
                  <span class="badge badge-pending">{eligibilityLabel(row)}</span>
                {/if}
              </td>
            </tr>
          {/each}
        </tbody>
      </table>
    {/if}
  {/if}
</section>

<style>
  .held-out-panel {
    display: flex;
    flex-direction: column;
    gap: 12px;
    font-family: var(--font-mono);
    font-size: 12px;
  }

  .panel-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
  }

  .panel-header h2 {
    margin: 0;
    font-size: 13px;
    font-weight: 500;
    letter-spacing: 0.04em;
    text-transform: uppercase;
  }

  .reload-btn {
    font-family: var(--font-mono);
    font-size: 10px;
    padding: 2px 8px;
    background: var(--bg-hover);
    border: 1px solid var(--border-default);
    border-radius: var(--r-xs);
    color: var(--text-primary);
    cursor: pointer;
  }

  .reload-btn:hover {
    background: var(--bg-surface);
  }

  .stats-row {
    display: flex;
    gap: 12px;
  }

  .stat {
    flex: 1;
    border: 1px solid var(--border-subtle);
    border-radius: var(--r-xs);
    padding: 6px 8px;
  }

  .stat-label {
    color: var(--text-secondary);
    font-size: 10px;
    letter-spacing: 0.04em;
    text-transform: uppercase;
  }

  .stat-value {
    font-size: 15px;
    font-weight: 500;
    color: var(--text-primary);
  }

  .source-row {
    display: flex;
    gap: 6px;
    flex-wrap: wrap;
  }

  .source-chip {
    display: inline-flex;
    align-items: center;
    gap: 4px;
    padding: 2px 6px;
    background: var(--bg-hover);
    border: 1px solid var(--border-subtle);
    border-radius: var(--r-xs);
    font-size: 10px;
  }

  .source-name {
    color: var(--text-primary);
    letter-spacing: 0.02em;
  }

  .source-count {
    color: var(--text-secondary);
  }

  table {
    width: 100%;
    border-collapse: collapse;
  }

  th, td {
    text-align: left;
    padding: 4px 6px;
    border-bottom: 1px solid var(--border-subtle);
  }

  th {
    font-weight: 500;
    color: var(--text-secondary);
    letter-spacing: 0.04em;
    text-transform: uppercase;
    font-size: 10px;
  }

  .badge {
    display: inline-block;
    padding: 1px 6px;
    border-radius: var(--r-xs);
    font-size: 10px;
    letter-spacing: 0.04em;
    text-transform: uppercase;
    background: var(--bg-hover);
    border: 1px solid var(--border-subtle);
  }

  .badge-active {
    background: rgba(80, 200, 120, 0.18);
    border-color: rgba(80, 200, 120, 0.45);
    color: rgb(120, 220, 150);
  }

  .badge-pending {
    background: rgba(220, 180, 80, 0.18);
    border-color: rgba(220, 180, 80, 0.45);
    color: rgb(220, 200, 140);
  }

  .muted {
    color: var(--text-secondary);
  }

  .error-banner {
    background: rgba(220, 80, 80, 0.1);
    border: 1px solid rgba(220, 80, 80, 0.4);
    padding: 8px;
    border-radius: var(--r-sm);
  }
</style>
