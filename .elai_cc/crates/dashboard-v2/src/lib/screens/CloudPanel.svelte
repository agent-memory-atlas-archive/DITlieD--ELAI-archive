<script lang="ts">
  // C-5 — Cloud Mode panel.
  //
  // Live view of cloud-mode state: per-task spend, daily spend with cap
  // gauge, provider selector (local | hybrid | cloud), redaction stats.
  //
  // Reads:
  //   - get_cloud_spend  -> CloudSpendInfo (task_usd, day_usd, cache_*,
  //                                          redaction_count)
  //   - get_execution_mode -> { mode: "LOCAL" | "HYBRID" | "CLOUD" }
  //
  // Writes:
  //   - set_execution_mode("LOCAL" | "HYBRID" | "CLOUD") on selector change
  //
  // The spend cap is sourced from the elai_cc.toml `[cloud.spend].hard_cap_daily_usd`
  // value. We expose a prop `capDailyUsd` so the parent (SettingsWindow)
  // can pass the persisted setting; absent a cap, the gauge renders as
  // "no cap configured".
  //
  // Day boundary is UTC (matches the cloud_spend_summary aggregator's
  // UTC-day window). Label states this explicitly so operators in
  // non-UTC timezones know when "today" resets.
  import { onMount, onDestroy } from 'svelte';
  import * as stub from '$lib/api/stub';
  import type { CloudSpendInfo, CloudRouteStatus } from '$lib/api/types';

  // Props (Svelte 5 runes mode).
  type Props = {
    /// Optional cap surface from elai_cc.toml. `null` => no cap configured.
    capDailyUsd?: number | null;
    /// Optional monthly cap. `null` => unset.
    capMonthlyUsd?: number | null;
    /// Active privacy profile from elai_cc.toml (read-only mirror).
    privacyProfile?: 'minimal' | 'standard' | 'strict';
    /// Air-gapped flag from elai_cc.toml (read-only mirror).
    airGapped?: boolean;
  };
  let {
    capDailyUsd = null,
    capMonthlyUsd = null,
    privacyProfile = 'standard',
    airGapped = false,
  }: Props = $props();

  type Provider = 'LOCAL' | 'HYBRID' | 'CLOUD';

  let spend = $state<CloudSpendInfo>({
    task_usd: 0,
    day_usd: 0,
    cache_creation_tokens: 0,
    cache_read_tokens: 0,
    redaction_count: 0,
  });
  // C-5 follow-on (SCHEMA_VERSION 103): latest routing status banners.
  // Each sub-field is null until the runtime emits a row of that type.
  let routeStatus = $state<CloudRouteStatus>({
    route_blocked: null,
    warn_limit_reached: null,
    cost_estimate: null,
  });
  let provider = $state<Provider>('LOCAL');
  let loading = $state(true);
  let timer: number | null = null;

  async function refresh(): Promise<void> {
    try {
      const [s, m, rs] = await Promise.all([
        stub.get_cloud_spend(),
        stub.get_execution_mode(),
        stub.get_cloud_route_status(),
      ]);
      spend = s;
      const raw = (m?.mode ?? 'LOCAL').toUpperCase();
      provider = raw === 'CLOUD' || raw === 'HYBRID' || raw === 'LOCAL' ? raw : 'LOCAL';
      routeStatus = rs;
    } catch {
      // best-effort polling: silent on transient failure, keep prior state
    } finally {
      loading = false;
    }
  }

  async function onProviderChange(next: Provider): Promise<void> {
    provider = next;
    try {
      await stub.set_execution_mode(next);
    } catch {
      // revert UI on failure
      await refresh();
    }
  }

  onMount(() => {
    refresh();
    // 5s polling: matches MonitorWindow's spend-gauge refresh cadence.
    // Lower than the 10s RuntimePanel cadence because cloud spend +
    // redaction events are higher-velocity than runtime-backend health.
    timer = window.setInterval(refresh, 5_000);
  });
  onDestroy(() => {
    if (timer !== null) window.clearInterval(timer);
  });

  const dayPct = $derived(
    capDailyUsd && capDailyUsd > 0
      ? Math.min(100, (spend.day_usd / capDailyUsd) * 100)
      : null,
  );
  const warnAt80 = $derived(dayPct !== null && dayPct >= 80);
  const blocked = $derived(dayPct !== null && dayPct >= 100);

  // H-9: the MEASURED cache hit-rate — the day-aggregate cache-read fraction
  // (cache_read / total cached input). The believe-the-number instrument for
  // H-1/H-2; never a literature figure (HD-1). Zero-guarded.
  const cacheTotal = $derived(spend.cache_read_tokens + spend.cache_creation_tokens);
  const cacheHitRate = $derived(
    cacheTotal > 0 ? (spend.cache_read_tokens / cacheTotal) * 100 : 0,
  );
</script>

<section class="cloud-panel" aria-label="Cloud mode panel">
  <header class="cloud-panel__header">
    <h3 class="cloud-panel__title mono">cloud mode</h3>
    <span class="cloud-panel__hint mono">UTC day · spend, redaction, provider</span>
  </header>

  <!-- C-5 follow-on: route-blocked notice (SCHEMA_VERSION 103). Renders
       only when CloudModeRouter rejected the most recent cloud probe with
       a Daily/MonthlySpendCapExceeded error. -->
  {#if routeStatus.route_blocked}
    <div class="cloud-panel__notice is-blocked" role="alert">
      <span class="cloud-panel__notice-tag mono">ROUTE BLOCKED</span>
      <span class="cloud-panel__notice-body mono">
        {routeStatus.route_blocked.cap_period} cap reached:
        ${routeStatus.route_blocked.spent_usd.toFixed(4)} /
        ${routeStatus.route_blocked.cap_usd.toFixed(2)}
        ({routeStatus.route_blocked.reason})
      </span>
    </div>
  {/if}

  <!-- C-5 follow-on: warn-limit-reached banner (SCHEMA_VERSION 103).
       Fires once per UTC day when day_usd crosses warn_at_percent of the
       daily cap. Sits ABOVE the hard-block notice so operators can act
       before the cap fires. -->
  {#if routeStatus.warn_limit_reached && !routeStatus.route_blocked}
    <div class="cloud-panel__notice is-warn" role="status">
      <span class="cloud-panel__notice-tag mono">WARN LIMIT</span>
      <span class="cloud-panel__notice-body mono">
        cloud spend at {routeStatus.warn_limit_reached.pct_of_cap.toFixed(0)}% of daily cap
        (${routeStatus.warn_limit_reached.spent_usd.toFixed(4)} /
        ${routeStatus.warn_limit_reached.cap_usd.toFixed(2)},
        threshold {routeStatus.warn_limit_reached.warn_at_percent}%)
      </span>
    </div>
  {/if}

  <!-- C-5 follow-on: cost estimate (SCHEMA_VERSION 103). Pre-task USD
       projection from plan skeleton. Hides when both bounds are zero so
       local-only tasks don't surface noise. heuristic_source carries the
       UNGROUNDED_HEURISTIC disclaimer; display it inline so operators
       know the bounds are coarse. -->
  {#if routeStatus.cost_estimate && routeStatus.cost_estimate.high_usd > 0}
    <div class="cloud-panel__notice is-info" role="status">
      <span class="cloud-panel__notice-tag mono">COST ESTIMATE</span>
      <span class="cloud-panel__notice-body mono">
        ${routeStatus.cost_estimate.low_usd.toFixed(4)} -
        ${routeStatus.cost_estimate.high_usd.toFixed(4)}
        ({routeStatus.cost_estimate.model},
        {routeStatus.cost_estimate.step_count} step{routeStatus.cost_estimate.step_count === 1 ? '' : 's'};
        {routeStatus.cost_estimate.heuristic_source})
      </span>
    </div>
  {/if}

  <!-- Spend gauge: day_usd / cap_usd -->
  <div class="cloud-panel__row">
    <span class="cloud-panel__label mono">Day spend</span>
    <div class="cloud-panel__control">
      <span class="mono cloud-panel__metric" class:is-warn={warnAt80} class:is-blocked={blocked}>
        ${spend.day_usd.toFixed(4)}
        {#if capDailyUsd !== null && capDailyUsd > 0}
          / ${capDailyUsd.toFixed(2)}
        {:else}
          <span class="cloud-panel__no-cap mono">(no cap configured)</span>
        {/if}
      </span>
      {#if dayPct !== null}
        <div class="cloud-panel__gauge" role="meter" aria-valuemin="0" aria-valuemax="100" aria-valuenow={dayPct}>
          <div class="cloud-panel__gauge-fill" class:is-warn={warnAt80} class:is-blocked={blocked} style="width: {dayPct.toFixed(1)}%"></div>
        </div>
        {#if blocked}
          <span class="cloud-panel__badge mono is-blocked">BLOCKED · cap reached</span>
        {:else if warnAt80}
          <span class="cloud-panel__badge mono is-warn">WARN · &ge;80%</span>
        {/if}
      {/if}
    </div>
  </div>

  <div class="cloud-panel__row">
    <span class="cloud-panel__label mono">Task spend</span>
    <div class="cloud-panel__control">
      <span class="mono cloud-panel__metric">${spend.task_usd.toFixed(4)}</span>
    </div>
  </div>

  {#if capMonthlyUsd !== null && capMonthlyUsd > 0}
    <div class="cloud-panel__row">
      <span class="cloud-panel__label mono">Monthly cap</span>
      <div class="cloud-panel__control">
        <span class="mono cloud-panel__metric">${capMonthlyUsd.toFixed(2)}</span>
      </div>
    </div>
  {/if}

  <!-- Provider selector -->
  <div class="cloud-panel__row">
    <span class="cloud-panel__label mono">Provider</span>
    <div class="cloud-panel__control" role="radiogroup" aria-label="Provider routing">
      <button
        type="button"
        class="cloud-panel__seg mono"
        class:is-active={provider === 'LOCAL'}
        role="radio"
        aria-checked={provider === 'LOCAL'}
        onclick={() => onProviderChange('LOCAL')}
      >local</button>
      <button
        type="button"
        class="cloud-panel__seg mono"
        class:is-active={provider === 'HYBRID'}
        role="radio"
        aria-checked={provider === 'HYBRID'}
        onclick={() => onProviderChange('HYBRID')}
      >hybrid</button>
      <button
        type="button"
        class="cloud-panel__seg mono"
        class:is-active={provider === 'CLOUD'}
        role="radio"
        aria-checked={provider === 'CLOUD'}
        onclick={() => onProviderChange('CLOUD')}
        disabled={airGapped}
        title={airGapped ? 'Air-gapped mode blocks cloud routing' : 'Pure cloud routing'}
      >cloud</button>
    </div>
  </div>

  <!-- Redaction stats -->
  <div class="cloud-panel__row">
    <span class="cloud-panel__label mono">Redactions today</span>
    <div class="cloud-panel__control">
      <span class="mono cloud-panel__metric">{spend.redaction_count}</span>
      <span class="cloud-panel__hint mono">profile: {privacyProfile}</span>
    </div>
  </div>

  <!-- Cache stats -->
  <div class="cloud-panel__row">
    <span class="cloud-panel__label mono">Cache tokens today</span>
    <div class="cloud-panel__control">
      <span class="mono cloud-panel__metric">
        {spend.cache_read_tokens} read · {spend.cache_creation_tokens} write
      </span>
    </div>
  </div>

  <!-- H-9 cost cockpit: the MEASURED cache hit-rate (cache-read fraction of
       cached input). The believe-the-number instrument for H-1/H-2. -->
  <div class="cloud-panel__row">
    <span class="cloud-panel__label mono">Cache hit rate</span>
    <div class="cloud-panel__control">
      <span class="mono cloud-panel__metric">{cacheHitRate.toFixed(1)}%</span>
      <span class="cloud-panel__hint mono">measured · cache-read fraction (today)</span>
    </div>
  </div>

  {#if loading}
    <div class="cloud-panel__loading mono">loading…</div>
  {/if}
</section>

<style>
  .cloud-panel {
    display: flex;
    flex-direction: column;
    gap: 8px;
    padding: 8px 0;
    font-family: var(--font-mono, ui-monospace, SFMono-Regular, monospace);
  }
  .cloud-panel__header {
    display: flex;
    align-items: baseline;
    gap: 12px;
  }
  .cloud-panel__title {
    margin: 0;
    font-size: 12px;
    text-transform: uppercase;
    letter-spacing: 0.06em;
    color: var(--text-mute, #8a8a8a);
  }
  .cloud-panel__hint {
    font-size: 11px;
    color: var(--text-dim, #6a6a6a);
  }
  .cloud-panel__row {
    display: grid;
    grid-template-columns: minmax(140px, 1fr) 3fr;
    align-items: center;
    gap: 12px;
    padding: 6px 0;
    border-bottom: 1px solid var(--border-soft, rgba(255, 255, 255, 0.06));
  }
  .cloud-panel__label {
    color: var(--text-mute, #8a8a8a);
    font-size: 11px;
  }
  .cloud-panel__control {
    display: flex;
    align-items: center;
    gap: 10px;
    flex-wrap: wrap;
  }
  .cloud-panel__metric {
    font-size: 12px;
  }
  .cloud-panel__metric.is-warn {
    color: var(--warn, #d99a3c);
  }
  .cloud-panel__metric.is-blocked {
    color: var(--err, #d94c4c);
  }
  .cloud-panel__no-cap {
    color: var(--text-dim, #6a6a6a);
    font-size: 11px;
    margin-left: 4px;
  }
  .cloud-panel__gauge {
    flex: 1;
    height: 4px;
    min-width: 80px;
    background: var(--bg-soft, rgba(255, 255, 255, 0.06));
    border-radius: 2px;
    overflow: hidden;
  }
  .cloud-panel__gauge-fill {
    height: 100%;
    background: var(--ok, #4ca97a);
    transition: width 200ms ease;
  }
  .cloud-panel__gauge-fill.is-warn {
    background: var(--warn, #d99a3c);
  }
  .cloud-panel__gauge-fill.is-blocked {
    background: var(--err, #d94c4c);
  }
  .cloud-panel__badge {
    font-size: 10px;
    padding: 2px 6px;
    border-radius: 3px;
    text-transform: uppercase;
    letter-spacing: 0.08em;
  }
  .cloud-panel__badge.is-warn {
    background: rgba(217, 154, 60, 0.12);
    color: var(--warn, #d99a3c);
  }
  .cloud-panel__badge.is-blocked {
    background: rgba(217, 76, 76, 0.12);
    color: var(--err, #d94c4c);
  }
  .cloud-panel__seg {
    background: transparent;
    border: 1px solid var(--border-soft, rgba(255, 255, 255, 0.12));
    color: var(--text, #d0d0d0);
    padding: 3px 10px;
    font-size: 11px;
    cursor: pointer;
    border-radius: 3px;
    font-family: inherit;
    text-transform: lowercase;
  }
  .cloud-panel__seg.is-active {
    background: var(--accent-soft, rgba(255, 255, 255, 0.12));
    border-color: var(--accent, #88aacc);
    color: var(--accent, #88aacc);
  }
  .cloud-panel__seg:disabled {
    opacity: 0.4;
    cursor: not-allowed;
  }
  .cloud-panel__loading {
    font-size: 11px;
    color: var(--text-dim, #6a6a6a);
    padding: 4px 0;
  }
  /* C-5 follow-on (SCHEMA_VERSION 103) — route-blocked / warn-limit /
     cost-estimate notice banners. Block + warn share the gauge palette
     so the operator's eye lands on the right severity at a glance. */
  .cloud-panel__notice {
    display: flex;
    align-items: center;
    gap: 8px;
    flex-wrap: wrap;
    padding: 6px 8px;
    border-radius: 3px;
    font-size: 11px;
    border: 1px solid var(--border-soft, rgba(255, 255, 255, 0.08));
  }
  .cloud-panel__notice.is-blocked {
    background: rgba(217, 76, 76, 0.12);
    color: var(--err, #d94c4c);
    border-color: var(--err, #d94c4c);
  }
  .cloud-panel__notice.is-warn {
    background: rgba(217, 154, 60, 0.12);
    color: var(--warn, #d99a3c);
    border-color: var(--warn, #d99a3c);
  }
  .cloud-panel__notice.is-info {
    background: rgba(136, 170, 204, 0.10);
    color: var(--accent, #88aacc);
    border-color: var(--accent, #88aacc);
  }
  .cloud-panel__notice-tag {
    font-size: 10px;
    text-transform: uppercase;
    letter-spacing: 0.08em;
    font-weight: 600;
  }
  .cloud-panel__notice-body {
    font-size: 11px;
  }
  .mono {
    font-family: inherit;
  }
</style>
