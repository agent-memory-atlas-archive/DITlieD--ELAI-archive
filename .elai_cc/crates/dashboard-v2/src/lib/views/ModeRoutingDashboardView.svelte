<script lang="ts">
  // D-27 — ModeRoutingDashboardView.
  //
  // Full-page integrated view that consolidates the four Wave-2 mode-
  // routing + refusal-SLA panels into a single dashboard surface:
  //
  //   * Top summary row — dominant grounding mode, current refusal
  //     rate (+ healthy-band tint), pending-discovery candidate count,
  //     and conformal tau headline pulled from the existing per-panel
  //     polling. Driven by `get_mode_routing_refusal_combined` so the
  //     row refreshes in a single round-trip.
  //   * Four sub-panels composed below — ModeRoutingPanel (D-10),
  //     NoveltyPanel (D-15), ConformalCalibrationPanel (D-16), and
  //     RefusalRateGauge (D-17). These continue to poll their own
  //     dedicated endpoints; the summary row is the only new client of
  //     the D-27 composite.
  //
  // Read-only. No producer. Pure composition + a single new composite
  // command (`get_mode_routing_refusal_combined`). The "discovery
  // queue" drawer is mounted separately via DrawerHost using the new
  // `discovery-queue` DrawerKey; this view surfaces the count only.
  //
  // Grounding: arXiv:1908.04734 (Everitt current-RF — orchestrator,
  // not policy LLM, picks the grounding mode); arXiv:2503.01345
  // (UniCR honest-refusal SLA).

  import ModeRoutingPanel from '../drawers/ModeRoutingPanel.svelte';
  import NoveltyPanel from '../drawers/NoveltyPanel.svelte';
  import ConformalCalibrationPanel from '../drawers/ConformalCalibrationPanel.svelte';
  import RefusalRateGauge from '../drawers/RefusalRateGauge.svelte';
  // D-27 GAP-1 + GAP-2 closure-pass inline widgets. Mounted in this
  // composite view (and NOT in MonitorWindow/ChatWindow — see
  // workflow/reasoning/D-27-decisions.md KNOWN-RESIDUAL KR-4 + KR-5
  // for per-window mounts that remain deferred).
  import SycophancyFlagStrip from '../components/SycophancyFlagStrip.svelte';
  import CheckerStatusPanel from '../components/CheckerStatusPanel.svelte';
  // D-27 closure (night 2026-05-30): per-class novelty breakdown widget.
  // Closes the cross-check gap noted in workflow/sessions/
  // s-night-2026-05-30/cross-check/D/D-09_d26_d27_d28.md TASK-D27-003.
  // Inline sibling of SycophancyFlagStrip + CheckerStatusPanel; backs the
  // ModeRoutingDashboardView upper widget row.
  import NoveltyBreakdown from '../components/NoveltyBreakdown.svelte';

  type DiscoveryQueueRow = {
    template_id: string;
    class: string;
    dkg_match_count: number;
    near_duplicate_threshold: number;
    ts_ms: number;
    days_waiting: number;
  };

  type NoveltyClassifiedRowD27 = {
    template_id: string;
    class: string;
    dkg_match_count: number;
    near_duplicate_threshold: number;
    ts_ms: number;
  };

  type ModeRoutingRefusalView = {
    mode_breakdown: Record<string, number>;
    refusal_rate_current: number;
    refusal_health_band: string;
    discovery_queue: DiscoveryQueueRow[];
    recent_classifications: NoveltyClassifiedRowD27[];
    dominant_mode: string | null;
    limit_applied: number;
  };

  const MODE_LABELS: Record<string, string> = {
    constructive: 'CONSTRUCTIVE',
    falsifiability_cross_family: 'FALSIFIABILITY',
    conformal_refusal: 'CONFORMAL REFUSAL',
    operational_deferred: 'OPERATIONAL DEFERRED',
    conservative_fallback: 'CONSERVATIVE FALLBACK',
  };

  let summary = $state<ModeRoutingRefusalView | null>(null);
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
      console.warn(`[mode-routing-dashboard] invoke('${cmd}') failed`, e);
      return undefined;
    }
  }

  async function reload() {
    try {
      const out = await callInvoke<ModeRoutingRefusalView>(
        'get_mode_routing_refusal_combined',
        { limit: 50 },
      );
      if (out === undefined) {
        summary = null;
        loadError = null;
      } else {
        summary = out;
        loadError = null;
      }
    } catch (e) {
      loadError = String(e);
    } finally {
      loaded = true;
    }
  }

  $effect(() => {
    reload();
    pollHandle = setInterval(reload, 10_000);
    return () => {
      if (pollHandle) {
        clearInterval(pollHandle);
        pollHandle = null;
      }
    };
  });

  const dominantModeLabel = $derived.by(() => {
    if (!summary || !summary.dominant_mode) return '—';
    return MODE_LABELS[summary.dominant_mode] ?? summary.dominant_mode.toUpperCase();
  });

  const refusalRateTint = $derived.by(() => {
    if (!summary) return 'unknown';
    return summary.refusal_health_band;
  });

  const refusalRatePct = $derived.by(() => {
    if (!summary) return '—';
    return `${(summary.refusal_rate_current * 100).toFixed(1)}%`;
  });

  const discoveryCount = $derived.by(() => {
    if (!summary) return 0;
    return summary.discovery_queue.length;
  });

  const totalClassifications = $derived.by(() => {
    if (!summary) return 0;
    return summary.recent_classifications.length;
  });
</script>

<section class="mode-routing-dashboard">
  <header class="mrd-header">
    <h2>Mode Routing &amp; Refusal SLA</h2>
    {#if loadError}
      <span class="mrd-error">composite read error: {loadError}</span>
    {/if}
  </header>

  <!-- Top summary row — single composite read. -->
  <div class="mrd-summary">
    <div class="mrd-summary-cell">
      <span class="mrd-label">DOMINANT MODE</span>
      <span class="mrd-value">{dominantModeLabel}</span>
    </div>
    <div class="mrd-summary-cell" data-band={refusalRateTint}>
      <span class="mrd-label">REFUSAL RATE</span>
      <span class="mrd-value">{refusalRatePct}</span>
      <span class="mrd-sub">band: {refusalRateTint}</span>
    </div>
    <div class="mrd-summary-cell">
      <span class="mrd-label">DISCOVERY QUEUE</span>
      <span class="mrd-value">{discoveryCount}</span>
      <span class="mrd-sub">{totalClassifications} total classifications</span>
    </div>
    <div class="mrd-summary-cell">
      <span class="mrd-label">WINDOW</span>
      <span class="mrd-value">{summary?.limit_applied ?? 0}</span>
      <span class="mrd-sub">most recent rows</span>
    </div>
  </div>

  {#if !loaded}
    <p class="mrd-loading">loading composite…</p>
  {/if}

  <!-- D-27 GAP-1 + GAP-2 + night-2026-05-30 closure-pass inline widgets.
       SycophancyFlagStrip (D-14 substrate), CheckerStatusPanel (D-11
       substrate), and NoveltyBreakdown (D-15 substrate, night closure)
       round out the Wave-2 surface alongside the 2x2 panel grid below. -->
  <div class="mrd-inline-row">
    <div class="mrd-inline-cell">
      <SycophancyFlagStrip />
    </div>
    <div class="mrd-inline-cell">
      <CheckerStatusPanel />
    </div>
    <div class="mrd-inline-cell">
      <NoveltyBreakdown />
    </div>
  </div>

  <!-- 2x2 panel grid, composing the existing D-10/D-15/D-16/D-17 panels. -->
  <div class="mrd-grid">
    <div class="mrd-panel-wrap">
      <ModeRoutingPanel />
    </div>
    <div class="mrd-panel-wrap">
      <RefusalRateGauge />
    </div>
    <div class="mrd-panel-wrap">
      <NoveltyPanel />
    </div>
    <div class="mrd-panel-wrap">
      <ConformalCalibrationPanel />
    </div>
  </div>
</section>

<style>
  .mode-routing-dashboard {
    display: flex;
    flex-direction: column;
    gap: 12px;
    padding: 12px;
    height: 100%;
    overflow: auto;
  }
  .mrd-header {
    display: flex;
    align-items: baseline;
    gap: 16px;
    border-bottom: 1px solid var(--border-subtle);
    padding-bottom: 6px;
  }
  .mrd-header h2 {
    margin: 0;
    font-size: 13px;
    letter-spacing: 0.05em;
    text-transform: uppercase;
  }
  .mrd-error {
    font-size: 10px;
    color: var(--fail);
  }
  .mrd-summary {
    display: grid;
    grid-template-columns: repeat(4, minmax(0, 1fr));
    gap: 8px;
  }
  .mrd-summary-cell {
    display: flex;
    flex-direction: column;
    gap: 2px;
    padding: 8px 10px;
    border: 1px solid var(--border-subtle);
    background: var(--bg-base);
  }
  .mrd-summary-cell[data-band='healthy'] {
    border-color: var(--pass);
  }
  .mrd-summary-cell[data-band='above'] {
    border-color: var(--warn);
  }
  .mrd-summary-cell[data-band='below'] {
    border-color: var(--info);
  }
  .mrd-label {
    font-size: 10px;
    letter-spacing: 0.08em;
    text-transform: uppercase;
    color: var(--text-muted, #888);
  }
  .mrd-value {
    font-size: 18px;
    font-variant-numeric: tabular-nums;
  }
  .mrd-sub {
    font-size: 10px;
    color: var(--text-muted, #888);
  }
  .mrd-loading {
    font-size: 10px;
    color: var(--text-muted, #888);
    margin: 0;
  }
  /* D-27 GAP-6 + night-2026-05-30 — inline widgets row
     (SycophancyFlagStrip + CheckerStatusPanel + NoveltyBreakdown).
     Three columns on wide screens, two on mid, one on narrow. */
  .mrd-inline-row {
    display: grid;
    grid-template-columns: repeat(3, minmax(0, 1fr));
    gap: 12px;
  }
  .mrd-inline-cell {
    min-width: 0;
  }
  @media (max-width: 1100px) {
    .mrd-inline-row {
      grid-template-columns: repeat(2, minmax(0, 1fr));
    }
  }
  @media (max-width: 800px) {
    .mrd-inline-row {
      grid-template-columns: minmax(0, 1fr);
    }
  }
  .mrd-grid {
    display: grid;
    grid-template-columns: repeat(2, minmax(0, 1fr));
    grid-template-rows: repeat(2, minmax(0, 1fr));
    gap: 12px;
    flex: 1;
    min-height: 0;
  }
  .mrd-panel-wrap {
    border: 1px solid var(--border-subtle);
    background: var(--bg-base);
    overflow: auto;
    min-height: 0;
  }
</style>
