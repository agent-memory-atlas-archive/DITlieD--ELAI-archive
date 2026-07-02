<script lang="ts">
  // D-26 GAP-1: ArchiveVerifyGauge.
  //
  // Compact inline status widget showing the most recent
  // archive_verify_result row (producer: `elai archive verify` CLI
  // driver, D-8). Read-only — polls the get_archive_verify_status
  // Tauri command on a 30-second cadence.
  //
  // Rendered inline within ReasoningDrawer (or any other panel that
  // surfaces archive-chain health). NOT a standalone drawer.
  //
  // Empty state (no archive_verify_result row yet):
  //   verified=false, last_verified_ts_ms=null. We distinguish this
  //   from a true verify failure with a separate "NEVER VERIFIED"
  //   badge.
  //
  // Polling cadence is tunable via the `pollMs` prop (default 30s,
  // matches the D-7 / D-26 polling precedent).

  import { onMount, onDestroy } from 'svelte';
  import type { ArchiveVerifyStatus } from '../api/types';

  interface Props {
    pollMs?: number;
  }
  const { pollMs = 30_000 }: Props = $props();

  let status = $state<ArchiveVerifyStatus | null>(null);
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
      console.warn(`[archive-verify-gauge] invoke('${cmd}') failed`, e);
      loadError = msg;
      return undefined;
    }
  }

  async function reload() {
    const out = await callInvoke<ArchiveVerifyStatus>('get_archive_verify_status');
    if (out !== undefined) {
      status = out;
      loadError = null;
    }
  }

  function fmtTs(ms: number | null): string {
    if (ms === null) return 'never';
    const d = new Date(ms);
    if (Number.isNaN(d.getTime())) return 'invalid';
    return d.toISOString().slice(0, 19).replace('T', ' ');
  }

  const stateLabel = $derived.by(() => {
    if (status === null) return 'LOADING';
    if (status.last_verified_ts_ms === null) return 'NEVER VERIFIED';
    return status.verified ? 'VERIFIED' : 'FAILED';
  });

  const stateTone = $derived.by(() => {
    if (status === null) return 'neutral';
    if (status.last_verified_ts_ms === null) return 'neutral';
    return status.verified ? 'pass' : 'fail';
  });

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

<div class="avg-root" aria-label="Archive chain verification gauge">
  <div class="avg-header">
    <span class="avg-title">ARCHIVE VERIFY</span>
    <span class="avg-badge avg-badge--{stateTone}">{stateLabel}</span>
  </div>

  {#if status !== null}
    <div class="avg-grid">
      <div class="avg-cell">
        <span class="avg-cell-label">entries</span>
        <span class="avg-cell-value">{status.total_entries}</span>
      </div>
      <div class="avg-cell">
        <span class="avg-cell-label">errors</span>
        <span class="avg-cell-value" class:avg-cell-value--fail={status.error_count > 0}>
          {status.error_count}
        </span>
      </div>
      <div class="avg-cell avg-cell--wide">
        <span class="avg-cell-label">last</span>
        <span class="avg-cell-value avg-cell-value--mono">
          {fmtTs(status.last_verified_ts_ms)}
        </span>
      </div>
    </div>

    {#if status.errors.length > 0}
      <div class="avg-errors">
        {#each status.errors.slice(0, 6) as kind, i (i + '-' + kind)}
          <span class="avg-err-chip">{kind}</span>
        {/each}
        {#if status.errors.length > 6}
          <span class="avg-err-more">+{status.errors.length - 6}</span>
        {/if}
      </div>
    {/if}
  {/if}

  {#if loadError !== null}
    <div class="avg-load-error" role="status">backend error: {loadError}</div>
  {/if}
</div>

<style>
  .avg-root {
    display: flex;
    flex-direction: column;
    gap: 6px;
    padding: 8px 10px;
    border: 1px solid var(--border-subtle);
    border-radius: var(--r-sm);
    background: var(--bg-input);
  }

  .avg-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 8px;
  }

  .avg-title {
    font-family: var(--font-mono);
    font-size: 10px;
    color: var(--text-ghost);
    letter-spacing: 0.08em;
    text-transform: uppercase;
  }

  .avg-badge {
    font-family: var(--font-mono);
    font-size: 10px;
    padding: 2px 6px;
    border-radius: var(--r-xs);
    text-transform: uppercase;
    letter-spacing: 0.06em;
  }

  .avg-badge--pass {
    background: rgba(0, 180, 130, 0.15);
    color: var(--teal-bright);
  }

  .avg-badge--fail {
    background: rgba(220, 60, 60, 0.18);
    color: var(--fail);
  }

  .avg-badge--neutral {
    background: rgba(120, 120, 120, 0.18);
    color: var(--text-ghost);
  }

  .avg-grid {
    display: grid;
    grid-template-columns: repeat(2, minmax(0, 1fr));
    gap: 4px 8px;
  }

  .avg-cell {
    display: flex;
    flex-direction: column;
    gap: 1px;
    min-width: 0;
  }

  .avg-cell--wide {
    grid-column: span 2;
  }

  .avg-cell-label {
    font-family: var(--font-mono);
    font-size: 10px;
    color: var(--text-ghost);
    letter-spacing: 0.06em;
  }

  .avg-cell-value {
    font-family: var(--font-mono);
    font-size: 12px;
    color: var(--text-secondary);
  }

  .avg-cell-value--mono {
    font-size: 10px;
  }

  .avg-cell-value--fail {
    color: var(--fail);
  }

  .avg-errors {
    display: flex;
    flex-wrap: wrap;
    gap: 4px;
  }

  .avg-err-chip {
    font-family: var(--font-mono);
    font-size: 10px;
    padding: 1px 5px;
    border-radius: var(--r-xs);
    background: rgba(220, 60, 60, 0.12);
    color: var(--fail);
  }

  .avg-err-more {
    font-family: var(--font-mono);
    font-size: 10px;
    color: var(--text-ghost);
  }

  .avg-load-error {
    font-family: var(--font-mono);
    font-size: 10px;
    color: var(--warn);
  }
</style>
