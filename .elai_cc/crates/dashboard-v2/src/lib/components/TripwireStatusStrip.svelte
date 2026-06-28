<script lang="ts">
  // D-26 GAP-2: TripwireStatusStrip.
  //
  // Horizontal strip showing the most recent tripwire fires from the
  // D-4 surface (`get_recent_tripwires`). Distinguishes:
  //   * fired            — single-detector fire (TripwireFired row)
  //   * fired_concurrent — >=2 detectors fired in same observe() tick
  //                        (TripwireFiredConcurrent — D-4 GAP-2)
  //   * auto_reverted    — template auto-revert applied
  //
  // Read-only — polls every 10s by default (matches CanaryPanel).
  // Color-coded by TripwireKindLabel (kl_divergence / correlation_drop
  // / iqr_outlier / perplexity / ddm) and by row kind.
  //
  // Used inline within ReasoningDrawer (or any panel that surfaces
  // tripwire activity). NOT a standalone drawer.

  import { onMount, onDestroy } from 'svelte';
  import type { TripwireRow, TripwireRowKind } from '../api/types';

  interface Props {
    limit?: number;
    pollMs?: number;
  }
  const { limit = 10, pollMs = 10_000 }: Props = $props();

  let rows = $state<TripwireRow[]>([]);
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
      const msg = e instanceof Error ? e.message : String(e);
      console.warn(`[tripwire-strip] invoke('${cmd}') failed`, e);
      loadError = msg;
      return undefined;
    }
  }

  async function reload() {
    const out = await callInvoke<TripwireRow[]>('get_recent_tripwires', { limit });
    if (out !== undefined) {
      rows = out.slice(0, limit);
      loadError = null;
    }
  }

  function fmtAgo(ts: number): string {
    const deltaMs = Date.now() - ts;
    if (deltaMs < 0 || !Number.isFinite(deltaMs)) return '—';
    const sec = Math.floor(deltaMs / 1000);
    if (sec < 60) return `${sec}s`;
    const min = Math.floor(sec / 60);
    if (min < 60) return `${min}m`;
    const hr = Math.floor(min / 60);
    if (hr < 24) return `${hr}h`;
    return `${Math.floor(hr / 24)}d`;
  }

  // Map snake_case TripwireKindLabel to a CSS-class suffix so the strip
  // can color-code each chip by detector family. Unknown labels fall
  // back to neutral.
  function kindClass(label: string | null | undefined): string {
    switch (label) {
      case 'kl_divergence':    return 'k-kl';
      case 'correlation_drop': return 'k-corr';
      case 'iqr_outlier':      return 'k-iqr';
      case 'perplexity':       return 'k-perp';
      case 'ddm':              return 'k-ddm';
      default:                  return 'k-neutral';
    }
  }

  function rowKindBadge(k: TripwireRowKind): string {
    switch (k) {
      case 'fired':            return 'FIRED';
      case 'fired_concurrent': return 'CONCURRENT';
      case 'auto_reverted':    return 'REVERTED';
    }
  }

  function rowKindClass(k: TripwireRowKind): string {
    switch (k) {
      case 'fired':            return 'rk-fired';
      case 'fired_concurrent': return 'rk-concurrent';
      case 'auto_reverted':    return 'rk-reverted';
    }
  }

  function detectorLabel(r: TripwireRow): string {
    if (r.kind === 'fired_concurrent') {
      const a = r.tripwire_a ?? '?';
      const b = r.tripwire_b ?? '?';
      return `${a} + ${b}`;
    }
    return r.tripwire_kind ?? r.retirement_reason ?? '—';
  }

  onMount(() => {
    reload();
    pollHandle = setInterval(reload, pollMs);
  });

  onDestroy(() => {
    if (pollHandle !== null) {
      clearInterval(pollHandle);
      pollHandle = null;
    }
  });
</script>

<div class="tss-root" aria-label="Recent tripwire fires">
  <div class="tss-header">
    <span class="tss-title">TRIPWIRES</span>
    <span class="tss-count">{rows.length}</span>
  </div>

  {#if rows.length === 0}
    <span class="tss-empty">no tripwires</span>
  {:else}
    <div class="tss-strip">
      {#each rows as r, i (i + '-' + r.seq)}
        <div class="tss-chip {kindClass(r.tripwire_kind)}" title={`seq ${r.seq} · ${detectorLabel(r)}`}>
          <span class="tss-chip-kind {rowKindClass(r.kind)}">{rowKindBadge(r.kind)}</span>
          <span class="tss-chip-detector">{detectorLabel(r)}</span>
          <span class="tss-chip-ago">{fmtAgo(r.ts)}</span>
        </div>
      {/each}
    </div>
  {/if}

  {#if loadError !== null}
    <div class="tss-load-error" role="status">backend error: {loadError}</div>
  {/if}
</div>

<style>
  .tss-root {
    display: flex;
    flex-direction: column;
    gap: 6px;
    padding: 8px 10px;
    border: 1px solid var(--border-subtle);
    border-radius: var(--r-sm);
    background: var(--bg-input);
  }

  .tss-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 8px;
  }

  .tss-title {
    font-family: var(--font-mono);
    font-size: 10px;
    color: var(--text-ghost);
    letter-spacing: 0.08em;
    text-transform: uppercase;
  }

  .tss-count {
    font-family: var(--font-mono);
    font-size: 10px;
    color: var(--text-ghost);
  }

  .tss-empty {
    font-family: var(--font-mono);
    font-size: 10px;
    color: var(--text-ghost);
  }

  .tss-strip {
    display: flex;
    flex-wrap: wrap;
    gap: 4px;
  }

  .tss-chip {
    display: inline-flex;
    align-items: center;
    gap: 5px;
    padding: 2px 6px;
    border-radius: var(--r-xs);
    border-left: 2px solid transparent;
    background: rgba(255, 255, 255, 0.04);
  }

  /* Detector-family color stripes (left border). */
  .tss-chip.k-kl       { border-left-color: var(--teal-bright); }
  .tss-chip.k-corr     { border-left-color: var(--steel); }
  .tss-chip.k-iqr      { border-left-color: var(--warn); }
  .tss-chip.k-perp     { border-left-color: #b97cff; }
  .tss-chip.k-ddm      { border-left-color: #ff9c5f; }
  .tss-chip.k-neutral  { border-left-color: var(--text-ghost); }

  .tss-chip-kind {
    font-family: var(--font-mono);
    font-size: 10px;
    padding: 1px 4px;
    border-radius: var(--r-xs);
    letter-spacing: 0.04em;
  }

  .tss-chip-kind.rk-fired      { background: rgba(220, 60, 60, 0.18); color: var(--fail); }
  .tss-chip-kind.rk-concurrent { background: rgba(220, 60, 60, 0.30); color: var(--fail); font-weight: 600; }
  .tss-chip-kind.rk-reverted   { background: rgba(120, 120, 120, 0.18); color: var(--text-secondary); }

  .tss-chip-detector {
    font-family: var(--font-mono);
    font-size: 10px;
    color: var(--text-secondary);
    max-width: 140px;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .tss-chip-ago {
    font-family: var(--font-mono);
    font-size: 10px;
    color: var(--text-ghost);
  }

  .tss-load-error {
    font-family: var(--font-mono);
    font-size: 10px;
    color: var(--warn);
  }
</style>
