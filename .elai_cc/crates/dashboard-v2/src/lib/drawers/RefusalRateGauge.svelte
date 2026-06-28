<script lang="ts">
  // D-17 — RefusalRateGauge.
  //
  // Read-only view of the honest-refusal SLA monitoring surface:
  //   * current refusal rate + healthy-band visualization
  //   * last breach chip
  //   * sparkline of recent refusal_rate snapshots
  //
  // Two Tauri commands (additive — see
  // crates/dashboard-v2/src-tauri/src/commands_d17.rs):
  //   * get_refusal_sla_recent(limit)       -> snapshot + breach rows + is_breaching
  //   * get_refusal_rate_current()          -> single most-recent snapshot (or null)
  //
  // The orchestrator (NOT the policy model) emits the underlying SSB
  // rows via refusal_sla::RefusalRateMonitor::emit_snapshot, reachable
  // through agent_pool::dispatch_refusal_sla_check. Polls on a 10s
  // cadence matching the other D-{1..18} panels.
  //
  // Grounding: arXiv:2503.01345 (UniCR honest refusal); docs/specs/
  // guide/real-problem-solving-grounded-2026-05-19.md §4.6 (refusal
  // rate as first-class quality metric).

  import { onMount, onDestroy } from 'svelte';

  type RefusalRateSnapshotRow = {
    refusal_rate: number;
    window_total: number;
    abstain_count: number;
    health_band: string;
    ts_ms: number;
  };

  type RefusalSlaBreachRow = {
    rate: number;
    threshold: number;
    breach_kind: string;
    ts_ms: number;
  };

  type RecentRefusalSla = {
    snapshot_rows: RefusalRateSnapshotRow[];
    breach_rows: RefusalSlaBreachRow[];
    is_breaching: boolean;
  };

  // D-27 GAP-4: HEALTHY_BAND_LO / _HI are now caller-overridable props
  // with defaults that mirror the orchestrator's RefusalSlaConfig::
  // default() (DEFAULT_HEALTHY_BAND_LO=0.05 + DEFAULT_HEALTHY_BAND_HI=
  // 0.20 — see crates/orchestrator/src/refusal_sla.rs:79+84). The
  // visual band is a hint only; classification is driven by the
  // backend `health_band` string carried on each snapshot row, so the
  // producer remains the source of truth. Callers that need to mirror
  // a non-default config can thread their own values through, or fall
  // back to these defaults when no override is available. See
  // workflow/reasoning/D-27-decisions.md DECISION-6.
  interface Props {
    healthyBandLo?: number;
    healthyBandHi?: number;
  }
  const {
    healthyBandLo = 0.05,
    healthyBandHi = 0.20,
  }: Props = $props();
  // Local aliases preserve the original constant names used throughout
  // the template below — keeps the diff scoped to the source line and
  // avoids a sweep edit across 5 template sites. $derived so updates
  // to the props re-flow through the template even though the defaults
  // are static.
  const HEALTHY_BAND_LO = $derived(healthyBandLo);
  const HEALTHY_BAND_HI = $derived(healthyBandHi);

  let recent = $state<RecentRefusalSla>({
    snapshot_rows: [],
    breach_rows: [],
    is_breaching: false,
  });
  let loaded = $state(false);
  let pollHandle: ReturnType<typeof setInterval> | null = null;

  async function callInvoke<T>(
    cmd: string,
    args?: Record<string, unknown>,
  ): Promise<T | undefined> {
    if (typeof window === 'undefined') return undefined;
    const tauriAvail = !!(window as unknown as { __TAURI_INTERNALS__?: unknown }).__TAURI_INTERNALS__;
    if (!tauriAvail) return undefined;
    try {
      const mod = await import('@tauri-apps/api/core');
      return (await mod.invoke<T>(cmd, args)) as T;
    } catch (e) {
      console.warn(`[refusal-sla] invoke('${cmd}') failed`, e);
      return undefined;
    }
  }

  async function reload() {
    const r = await callInvoke<RecentRefusalSla>('get_refusal_sla_recent', {
      limit: 50,
    });
    if (r !== undefined) {
      recent = r;
    }
    loaded = true;
  }

  function fmtTs(ms: number | null | undefined): string {
    if (!ms || ms <= 0) return '—';
    try {
      return new Date(ms).toISOString().slice(0, 19).replace('T', ' ');
    } catch {
      return '—';
    }
  }

  function fmtPct(rate: number): string {
    if (!Number.isFinite(rate)) return '—';
    return `${(rate * 100).toFixed(1)}%`;
  }

  function bandClass(band: string): string {
    switch (band) {
      case 'below':
        return 'band-below';
      case 'healthy':
        return 'band-healthy';
      case 'above':
        return 'band-above';
      default:
        return 'band-unknown';
    }
  }

  // Anchor the rate marker on a 0-100% horizontal axis.
  function ratePct(rate: number): number {
    if (!Number.isFinite(rate)) return 0;
    return Math.max(0, Math.min(100, Math.round(rate * 100)));
  }

  // Build a simple SVG path for the rate sparkline. Polyline points
  // are positioned across 200x40 viewBox; oldest on left, newest on
  // right. `snapshot_rows` is newest-first, so reverse before
  // plotting.
  function sparklinePoints(rows: RefusalRateSnapshotRow[]): string {
    if (rows.length === 0) return '';
    const sorted = [...rows].reverse();
    const n = sorted.length;
    const w = 200;
    const h = 40;
    const stepX = n > 1 ? w / (n - 1) : 0;
    return sorted
      .map((row, i) => {
        const x = (i * stepX).toFixed(1);
        const r = Number.isFinite(row.refusal_rate) ? row.refusal_rate : 0;
        const clamped = Math.max(0, Math.min(1, r));
        // y axis is inverted in SVG; high rate = low y.
        const y = (h - clamped * h).toFixed(1);
        return `${x},${y}`;
      })
      .join(' ');
  }

  const currentSnapshot = $derived(recent.snapshot_rows[0] ?? null);
  const lastBreach = $derived(recent.breach_rows[0] ?? null);
  const sparkline = $derived(sparklinePoints(recent.snapshot_rows));

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
  <h2>Refusal Rate SLA (Honest-Refusal Monitor)</h2>

  {#if !loaded}
    <p class="muted">Loading…</p>
  {:else}
    <section class="current">
      <header>
        <strong>Current refusal rate</strong>
      </header>
      {#if currentSnapshot === null}
        <p class="muted">No refusal snapshots recorded yet.</p>
      {:else}
        <div class="rate-row">
          <span class="rate-value {bandClass(currentSnapshot.health_band)}">
            {fmtPct(currentSnapshot.refusal_rate)}
          </span>
          <span class={'band-chip ' + bandClass(currentSnapshot.health_band)}>
            {currentSnapshot.health_band || 'unknown'}
          </span>
          <span class="muted">
            {currentSnapshot.abstain_count} / {currentSnapshot.window_total} window
          </span>
        </div>

        <div class="band-axis" aria-label="refusal rate axis 0 to 100 percent">
          <div
            class="band-healthy-region"
            style="left: {HEALTHY_BAND_LO * 100}%; width: {(HEALTHY_BAND_HI - HEALTHY_BAND_LO) * 100}%"
          ></div>
          <div
            class="rate-marker"
            style="left: {ratePct(currentSnapshot.refusal_rate)}%"
            aria-label="current rate marker"
          ></div>
        </div>
        <div class="axis-legend">
          <span>0%</span>
          <span class="muted">healthy: {fmtPct(HEALTHY_BAND_LO)}–{fmtPct(HEALTHY_BAND_HI)}</span>
          <span>100%</span>
        </div>
      {/if}
    </section>

    <section class="last-breach">
      <header>
        <strong>Last breach</strong>
      </header>
      {#if lastBreach === null}
        <p class="muted">No SLA breaches recorded yet.</p>
      {:else}
        <table>
          <tbody>
            <tr>
              <th>When</th>
              <td>{fmtTs(lastBreach.ts_ms)}</td>
            </tr>
            <tr>
              <th>Kind</th>
              <td><span class={'band-chip band-' + lastBreach.breach_kind}>{lastBreach.breach_kind || '—'}</span></td>
            </tr>
            <tr>
              <th>Rate</th>
              <td>{fmtPct(lastBreach.rate)}</td>
            </tr>
            <tr>
              <th>Threshold</th>
              <td>{fmtPct(lastBreach.threshold)}</td>
            </tr>
          </tbody>
        </table>
      {/if}
    </section>

    <section class="sparkline">
      <header>
        <strong>Recent rate sparkline</strong>
        <span class="muted">{recent.snapshot_rows.length} snapshots</span>
      </header>
      {#if recent.snapshot_rows.length === 0}
        <p class="muted">No snapshots in the recent window.</p>
      {:else}
        <svg viewBox="0 0 200 40" class="spark" preserveAspectRatio="none" aria-label="refusal rate sparkline">
          <line x1="0" y1="{40 - HEALTHY_BAND_HI * 40}" x2="200" y2="{40 - HEALTHY_BAND_HI * 40}" class="band-line" />
          <line x1="0" y1="{40 - HEALTHY_BAND_LO * 40}" x2="200" y2="{40 - HEALTHY_BAND_LO * 40}" class="band-line" />
          <polyline points={sparkline} class="spark-line" />
        </svg>
      {/if}
    </section>

    <section class="snapshots">
      <header>
        <strong>Recent snapshots</strong>
        <span class="muted">{recent.snapshot_rows.length} rows</span>
      </header>
      {#if recent.snapshot_rows.length === 0}
        <p class="muted">No snapshots recorded yet.</p>
      {:else}
        <table>
          <thead>
            <tr>
              <th>When</th>
              <th>Rate</th>
              <th>Window</th>
              <th>Abstain</th>
              <th>Band</th>
            </tr>
          </thead>
          <tbody>
            {#each recent.snapshot_rows as row}
              <tr>
                <td>{fmtTs(row.ts_ms)}</td>
                <td>{fmtPct(row.refusal_rate)}</td>
                <td>{row.window_total}</td>
                <td>{row.abstain_count}</td>
                <td><span class={'band-chip ' + bandClass(row.health_band)}>{row.health_band || '—'}</span></td>
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
    border-bottom: 1px solid var(--border-default);
    vertical-align: middle;
  }

  .rate-row {
    display: flex;
    gap: 0.75rem;
    align-items: baseline;
    margin-bottom: 0.5rem;
  }

  .rate-value {
    font-size: 1.5rem;
    font-weight: 600;
  }

  .band-axis {
    position: relative;
    height: 1.25rem;
    background: var(--bg-input);
    border: 1px solid var(--border-default);
    overflow: hidden;
  }

  .band-healthy-region {
    position: absolute;
    top: 0;
    bottom: 0;
    background: var(--pass-ghost);
  }

  .rate-marker {
    position: absolute;
    top: 0;
    bottom: 0;
    width: 2px;
    background: var(--rate-marker, #fff);
    transform: translateX(-1px);
  }

  .axis-legend {
    display: flex;
    justify-content: space-between;
    font-size: 0.7rem;
    margin-top: 0.25rem;
  }

  .band-chip {
    display: inline-block;
    padding: 0.1rem 0.4rem;
    border-radius: 0.25rem;
    font-size: 0.75rem;
    text-transform: uppercase;
    letter-spacing: 0.05em;
  }

  .band-below {
    background: rgba(204, 102, 102, 0.2);
    color: #c66;
  }

  .band-healthy {
    background: rgba(74, 136, 102, 0.2);
    color: #4a8;
  }

  .band-above {
    background: rgba(204, 102, 102, 0.2);
    color: #c66;
  }

  .band-unknown {
    color: var(--muted, #888);
  }

  .spark {
    width: 100%;
    height: 3rem;
    display: block;
    background: var(--bg-input);
    border: 1px solid var(--border-default);
  }

  .band-line {
    stroke: var(--pass-soft);
    stroke-width: 0.5;
    stroke-dasharray: 2 2;
  }

  .spark-line {
    fill: none;
    stroke: #4a8;
    stroke-width: 1.5;
    vector-effect: non-scaling-stroke;
  }
</style>
