<script lang="ts">
  // D-27 GAP-1: SycophancyFlagStrip.
  //
  // Horizontal strip showing the most recent SycophancyFlag rows from
  // the D-14 substrate. Polls `get_sycophancy_flags_recent` every
  // `pollMs` ms (default 10s, matches TripwireStatusStrip cadence).
  //
  // Color semantics (mirrors the D-14 ProbeVerdict three-arm collapse):
  //   * red   — flip_detected = true (divergent_sycophancy_flag verdict)
  //   * gray  — flip_detected = false AND verdict = convergent
  //   * yellow— verdict = inconclusive
  //
  // Read-only — pure projection over D-14 SsbActionType::SycophancyFlag
  // rows. No producer.
  //
  // Used inline (NOT a drawer) — composable into ModeRoutingDashboardView,
  // ReasoningDrawer, or any future ChatWindow side strip. Mirrors the
  // TripwireStatusStrip pattern from D-26 GAP-2.
  //
  // Grounding: arXiv:2310.13548 (Sharma sycophancy — §3.2 flip
  // definition + §4 PM residual sycophancy floor).

  import { onMount, onDestroy } from 'svelte';
  import type { SycophancyFlagRow } from '../api/types';

  interface Props {
    limit?: number;
    pollMs?: number;
  }
  const { limit = 10, pollMs = 10_000 }: Props = $props();

  let rows = $state<SycophancyFlagRow[]>([]);
  let loadError = $state<string | null>(null);
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
      const msg = e instanceof Error ? e.message : String(e);
      console.warn(`[sycophancy-strip] invoke('${cmd}') failed`, e);
      loadError = msg;
      return undefined;
    }
  }

  async function reload() {
    const out = await callInvoke<SycophancyFlagRow[]>('get_sycophancy_flags_recent', { limit });
    if (out !== undefined) {
      rows = out.slice(0, limit);
      loadError = null;
    }
  }

  function fmtAgo(tsMs: number): string {
    if (!Number.isFinite(tsMs) || tsMs <= 0) return '—';
    const deltaMs = Date.now() - tsMs;
    if (deltaMs < 0 || !Number.isFinite(deltaMs)) return '—';
    const sec = Math.floor(deltaMs / 1000);
    if (sec < 60) return `${sec}s`;
    const min = Math.floor(sec / 60);
    if (min < 60) return `${min}m`;
    const hr = Math.floor(min / 60);
    if (hr < 24) return `${hr}h`;
    return `${Math.floor(hr / 24)}d`;
  }

  // Map row to a chip color class per the GAP-1 spec:
  //   red   — flip_detected = true
  //   yellow — verdict === 'inconclusive'
  //   gray  — convergent (or unknown)
  function chipClass(r: SycophancyFlagRow): string {
    if (r.flip_detected) return 'sfs-chip-flip';
    if (r.verdict === 'inconclusive') return 'sfs-chip-inconclusive';
    return 'sfs-chip-convergent';
  }

  function verdictBadge(r: SycophancyFlagRow): string {
    if (r.flip_detected) return 'FLIP';
    if (r.verdict === 'inconclusive') return 'INCONCLUSIVE';
    return 'CONVERGENT';
  }

  function fmtDelta(d: number): string {
    if (!Number.isFinite(d)) return '—';
    return d.toFixed(2);
  }

  function shortClaimId(id: string): string {
    if (!id) return '—';
    if (id.length <= 14) return id;
    return id.slice(0, 14) + '…';
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

<div class="sfs-root" aria-label="Recent sycophancy flags">
  <div class="sfs-header">
    <span class="sfs-title">SYCOPHANCY FLAGS</span>
    <span class="sfs-count">{rows.length}</span>
  </div>

  {#if rows.length === 0}
    <span class="sfs-empty">no sycophancy flags</span>
  {:else}
    <div class="sfs-strip">
      {#each rows as r (r.seq)}
        <div
          class="sfs-chip {chipClass(r)}"
          title={`seq ${r.seq} · claim ${r.claim_id} · delta ${fmtDelta(r.delta)} · verdict ${r.verdict}`}
        >
          <span class="sfs-chip-kind">{verdictBadge(r)}</span>
          <span class="sfs-chip-claim">{shortClaimId(r.claim_id)}</span>
          <span class="sfs-chip-delta">Δ {fmtDelta(r.delta)}</span>
          <span class="sfs-chip-ago">{fmtAgo(r.ts_ms)}</span>
        </div>
      {/each}
    </div>
  {/if}

  {#if loadError !== null}
    <div class="sfs-load-error" role="status">backend error: {loadError}</div>
  {/if}
</div>

<style>
  .sfs-root {
    display: flex;
    flex-direction: column;
    gap: 6px;
    padding: 8px 10px;
    border: 1px solid var(--border-subtle, #2a2a2a);
    border-radius: var(--r-sm, 4px);
    background: var(--bg-input, #181818);
  }

  .sfs-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 8px;
  }

  .sfs-title {
    font-family: var(--font-mono, monospace);
    font-size: 10px;
    color: var(--text-ghost, #888);
    letter-spacing: 0.08em;
    text-transform: uppercase;
  }

  .sfs-count {
    font-family: var(--font-mono, monospace);
    font-size: 10px;
    color: var(--text-ghost, #888);
  }

  .sfs-empty {
    font-family: var(--font-mono, monospace);
    font-size: 10px;
    color: var(--text-ghost, #888);
  }

  .sfs-strip {
    display: flex;
    flex-wrap: wrap;
    gap: 4px;
  }

  .sfs-chip {
    display: inline-flex;
    align-items: center;
    gap: 5px;
    padding: 2px 6px;
    border-radius: var(--r-xs, 3px);
    border-left: 2px solid transparent;
    background: rgba(255, 255, 255, 0.04);
  }

  /* red — flip detected (divergent_sycophancy_flag) */
  .sfs-chip.sfs-chip-flip {
    border-left-color: var(--fail, #dc3c3c);
    background: rgba(220, 60, 60, 0.10);
  }
  /* yellow — inconclusive */
  .sfs-chip.sfs-chip-inconclusive {
    border-left-color: var(--warn, #f9c44d);
    background: rgba(249, 196, 77, 0.08);
  }
  /* gray — convergent (no flip) */
  .sfs-chip.sfs-chip-convergent {
    border-left-color: var(--text-ghost, #6a6a6a);
  }

  .sfs-chip-kind {
    font-family: var(--font-mono, monospace);
    font-size: 9px;
    padding: 1px 4px;
    border-radius: var(--r-xs, 3px);
    letter-spacing: 0.04em;
  }

  .sfs-chip.sfs-chip-flip .sfs-chip-kind {
    color: var(--fail, #dc3c3c);
    background: rgba(220, 60, 60, 0.18);
    font-weight: 600;
  }
  .sfs-chip.sfs-chip-inconclusive .sfs-chip-kind {
    color: var(--warn, #f9c44d);
    background: rgba(249, 196, 77, 0.18);
  }
  .sfs-chip.sfs-chip-convergent .sfs-chip-kind {
    color: var(--text-secondary, #aaa);
    background: rgba(120, 120, 120, 0.18);
  }

  .sfs-chip-claim {
    font-family: var(--font-mono, monospace);
    font-size: 10px;
    color: var(--text-secondary, #aaa);
    max-width: 140px;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .sfs-chip-delta {
    font-family: var(--font-mono, monospace);
    font-size: 10px;
    color: var(--text-ghost, #888);
    font-variant-numeric: tabular-nums;
  }

  .sfs-chip-ago {
    font-family: var(--font-mono, monospace);
    font-size: 9px;
    color: var(--text-ghost, #888);
  }

  .sfs-load-error {
    font-family: var(--font-mono, monospace);
    font-size: 10px;
    color: var(--warn, #f9c44d);
  }
</style>
