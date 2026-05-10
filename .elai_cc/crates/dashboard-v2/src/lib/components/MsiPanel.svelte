<script lang="ts">
  // B-23 Phase 4H Group F (4H.8) — MSI per-PipelineMode dashboard surface.
  // Renders the latest msi_update SSB row per mode (surgical / greenfield /
  // oracle). Polls every 5s. DIVERGING is highlighted to surface
  // self-modification pause events to the operator quickly.

  import { onMount, onDestroy } from 'svelte';
  import * as stub from '$lib/api/stub';
  import type { SelfModMsiSnapshot } from '$lib/api/types';

  let snapshots = $state<SelfModMsiSnapshot[]>([]);
  let loadError = $state<string | null>(null);
  let timer: ReturnType<typeof setInterval> | null = null;

  function isSelfModMsi(s: unknown): s is SelfModMsiSnapshot {
    return !!s && typeof s === 'object' && 'pipeline_mode' in (s as Record<string, unknown>) &&
      'msi_value' in (s as Record<string, unknown>);
  }

  async function load(): Promise<void> {
    try {
      const result = await stub.get_msi_status();
      if (Array.isArray(result)) {
        snapshots = result.filter(isSelfModMsi);
      } else {
        // Legacy multi-session-isolation shape — render nothing.
        snapshots = [];
      }
      loadError = null;
    } catch (e) {
      loadError = String(e);
    }
  }

  function statusClass(status: string | undefined): string {
    if (!status) return 'unknown';
    const s = status.toLowerCase();
    if (s.includes('improving')) return 'improving';
    if (s.includes('diverging')) return 'diverging';
    if (s.includes('stabilizing')) return 'stabilizing';
    return 'unknown';
  }

  onMount(() => {
    void load();
    timer = setInterval(() => void load(), 5_000);
  });

  onDestroy(() => {
    if (timer) clearInterval(timer);
  });
</script>

<div class="msi-panel">
  <h3>Modification Solidification Index (4H.7)</h3>
  {#if loadError}
    <div class="err">load error: {loadError}</div>
  {:else if snapshots.length === 0}
    <div class="empty">no MSI snapshots emitted yet</div>
  {:else}
    <ul>
      {#each snapshots as snap (snap.pipeline_mode)}
        <li class={statusClass(snap.status)}>
          <span class="mode">{snap.pipeline_mode}</span>
          <span class="value" class:negative={snap.msi_value < 0}>
            {snap.msi_value.toFixed(3)}
          </span>
          <span class="status">{snap.status ?? '—'}</span>
        </li>
      {/each}
    </ul>
  {/if}
</div>

<style>
  .msi-panel {
    font-family: var(--mono, ui-monospace, monospace);
    font-size: 12px;
    padding: 8px 12px;
  }
  h3 {
    font-size: 13px;
    margin: 0 0 8px;
    letter-spacing: 0.04em;
    text-transform: uppercase;
  }
  .err {
    color: var(--accent-red, #d33);
  }
  .empty {
    color: var(--text-muted, #666);
    font-style: italic;
  }
  ul {
    list-style: none;
    padding: 0;
    margin: 0;
  }
  li {
    display: grid;
    grid-template-columns: 110px 80px 1fr;
    gap: 12px;
    padding: 6px 0;
    border-bottom: 1px solid var(--border-soft, #2a2a2a);
    align-items: center;
  }
  .mode {
    color: var(--text-secondary, #aaa);
    text-transform: uppercase;
    letter-spacing: 0.04em;
  }
  .value {
    text-align: right;
    color: var(--accent-blue, #4a90e2);
  }
  .value.negative {
    color: var(--accent-orange, #d97706);
  }
  .status {
    color: var(--text-muted, #888);
  }
  li.improving .status { color: var(--accent-green, #50c878); }
  li.stabilizing .status { color: var(--accent-blue, #4a90e2); }
  li.diverging {
    background: var(--bg-warn, rgba(217, 119, 6, 0.06));
  }
  li.diverging .status { color: var(--accent-orange, #d97706); font-weight: bold; }
</style>
