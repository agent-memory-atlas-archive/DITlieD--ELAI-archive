<script lang="ts">
  // B-42 — Cosmos Artifact Tier drawer.
  //
  // Surfaces the read-only per-artifact freshness snapshot from
  // ~/.elai/cosmos/. The Refresh button calls `triggerCosmosRefresh`, which
  // currently returns an explicit NotImplemented error pending the production
  // ResearchDispatcher (see workflow/reasoning/B-42-decisions.md BLOCK-5).
  // We render that error inline next to the row so the UI is honest about
  // the half-wired state — no fake success animations.

  import { onMount, onDestroy } from 'svelte';
  import { Card, Badge } from '$lib/ui';
  import type { CosmosArtifactStatus } from '$lib/api/types';
  import {
    getCosmosStatus,
    triggerCosmosRefresh,
    subscribeCosmosEvents,
    type CosmosEvent,
  } from '$lib/api/stub';

  let artifacts = $state<CosmosArtifactStatus[]>([]);
  let loaded = $state(false);
  let loadError = $state<string | null>(null);
  let refreshBusy = $state<Record<string, boolean>>({});
  let refreshError = $state<Record<string, string>>({});

  // C-16 (DECISION-C): cosmos SSB event tail + last_seq cursor.
  let recentEvents = $state<CosmosEvent[]>([]);
  let lastSeq = $state<number>(0);

  async function reload() {
    try {
      artifacts = await getCosmosStatus();
      loadError = null;
    } catch (e) {
      loadError = String(e);
    } finally {
      loaded = true;
    }
  }

  // C-16 (DECISION-C): poll subscribe_cosmos_events. New rows arriving
  // means tick() / trigger_cosmos_refresh emitted SSB activity — reload
  // the status snapshot so the drawer reflects the post-refresh state
  // without waiting for the 30s status poll.
  async function pollEvents() {
    try {
      const batch = await subscribeCosmosEvents(lastSeq);
      if (batch.events.length > 0) {
        // Append, cap displayed tail at 16 rows.
        recentEvents = [...recentEvents, ...batch.events].slice(-16);
        lastSeq = batch.last_seq;
        // If we saw artifact_updated or refresh_started, refresh status.
        const triggersReload = batch.events.some(
          (e) => e.kind === 'artifact_updated' || e.kind === 'refresh_started',
        );
        if (triggersReload) {
          await reload();
        }
      } else if (lastSeq === 0) {
        // Cold-start: bump last_seq to the returned baseline so the next
        // poll only returns true deltas.
        lastSeq = batch.last_seq;
      }
    } catch (_e) {
      // Silent failure — drawer remains on the last good snapshot.
    }
  }

  let statusInterval: ReturnType<typeof setInterval> | null = null;
  let eventsInterval: ReturnType<typeof setInterval> | null = null;

  onMount(() => {
    reload();
    pollEvents();
    // Status snapshot poll (slow). Live updates come from the events poll.
    statusInterval = setInterval(reload, 30_000);
    // C-16 (DECISION-C): 5s cadence for the SSB event subscription.
    eventsInterval = setInterval(pollEvents, 5_000);
  });

  onDestroy(() => {
    if (statusInterval !== null) clearInterval(statusInterval);
    if (eventsInterval !== null) clearInterval(eventsInterval);
  });

  async function onRefresh(name: string) {
    if (refreshBusy[name]) return;
    refreshBusy = { ...refreshBusy, [name]: true };
    refreshError = { ...refreshError, [name]: '' };
    try {
      const res = await triggerCosmosRefresh(name);
      if (!res.ok) {
        refreshError = { ...refreshError, [name]: res.error ?? 'unknown error' };
      } else {
        // C-16: real success path. Reload status + event tail immediately.
        await Promise.all([reload(), pollEvents()]);
      }
    } finally {
      refreshBusy = { ...refreshBusy, [name]: false };
    }
  }

  function statusTone(status: string): 'teal' | 'warn' | 'fail' | 'ghost' {
    if (status === 'fresh') return 'teal';
    if (status === 'stale') return 'warn';
    if (status === 'decayed') return 'fail';
    return 'ghost';
  }

  /** Humanize an RFC3339 timestamp as a coarse relative string. */
  function humanizeRelative(iso: string | null): string {
    if (!iso) return 'never';
    const t = Date.parse(iso);
    if (Number.isNaN(t)) return iso;
    const delta = Math.max(0, Date.now() - t);
    const secs = Math.floor(delta / 1000);
    if (secs < 60) return `${secs}s ago`;
    const mins = Math.floor(secs / 60);
    if (mins < 60) return `${mins} min ago`;
    const hrs = Math.floor(mins / 60);
    if (hrs < 24) return `${hrs} hour${hrs === 1 ? '' : 's'} ago`;
    const days = Math.floor(hrs / 24);
    if (days < 30) return `${days} day${days === 1 ? '' : 's'} ago`;
    const months = Math.floor(days / 30);
    if (months < 12) return `${months} month${months === 1 ? '' : 's'} ago`;
    const years = Math.floor(months / 12);
    return `${years} year${years === 1 ? '' : 's'} ago`;
  }

  function fmtBytes(n: number): string {
    if (n <= 0) return '—';
    if (n < 1024) return `${n} B`;
    if (n < 1024 * 1024) return `${(n / 1024).toFixed(1)} KiB`;
    return `${(n / (1024 * 1024)).toFixed(2)} MiB`;
  }
</script>

<div class="cosmos-drawer">
  <Card variant="plain" padding={12}>
    <div class="section-label">COSMOS ARTIFACT TIER</div>
    <div class="cosmos-note">
      Cross-project knowledge cache rooted at <span class="mono">~/.elai/cosmos/</span>.
      Live SSB subscription active (5s poll, C-16 DECISION-C).
    </div>

    {#if !loaded}
      <div class="muted-small">loading…</div>
    {:else if loadError}
      <div class="muted-small error">load failed: {loadError}</div>
    {:else if artifacts.length === 0}
      <div class="muted-small">
        No cosmos artifacts on disk yet. Manifest is installed lazily on first
        boot; nothing to display until <span class="mono">~/.elai/cosmos/manifest.toml</span>
        is populated.
      </div>
    {:else}
      <div class="rows">
        {#each artifacts as a}
          <div class="row">
            <div class="row-head">
              <span class="name mono">{a.name}</span>
              <Badge tone={statusTone(a.status)} size="xs">{a.status.toUpperCase()}</Badge>
            </div>
            <div class="row-meta">
              <span class="ghost-text">refreshed {humanizeRelative(a.refreshed_at)}</span>
              <span class="ghost-text">{fmtBytes(a.size_bytes)}</span>
            </div>
            {#if a.project_classes.length > 0}
              <div class="class-chips">
                {#each a.project_classes as cls}
                  <span class="class-chip">{cls}</span>
                {/each}
              </div>
            {/if}
            <div class="row-actions">
              <button
                class="refresh-btn"
                type="button"
                disabled={refreshBusy[a.name]}
                onclick={() => onRefresh(a.name)}
              >
                {refreshBusy[a.name] ? 'refreshing…' : 'Refresh'}
              </button>
              {#if refreshError[a.name]}
                <span class="refresh-error mono" title={refreshError[a.name]}>
                  {refreshError[a.name]}
                </span>
              {/if}
            </div>
          </div>
        {/each}
      </div>
    {/if}
  </Card>
</div>

<style>
  .cosmos-drawer {
    display: flex;
    flex-direction: column;
    gap: 8px;
  }
  .section-label {
    font-family: var(--font-mono);
    font-size: 10px;
    color: var(--text-secondary);
    letter-spacing: 0.08em;
    text-transform: uppercase;
    margin-bottom: 8px;
  }
  .cosmos-note {
    font-size: 10px;
    color: var(--text-secondary);
    margin-bottom: 10px;
    line-height: 1.5;
  }
  .rows {
    display: flex;
    flex-direction: column;
    gap: 10px;
  }
  .row {
    display: flex;
    flex-direction: column;
    gap: 4px;
    padding: 8px;
    background: var(--bg-input);
    border-radius: var(--r-sm);
  }
  .row-head {
    display: flex;
    align-items: center;
    gap: 8px;
    justify-content: space-between;
  }
  .row-meta {
    display: flex;
    gap: 12px;
  }
  .name {
    font-family: var(--font-mono);
    font-size: 12px;
    color: var(--text-primary);
    min-width: 0;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }
  .mono {
    font-family: var(--font-mono);
  }
  .class-chips {
    display: flex;
    flex-wrap: wrap;
    gap: 4px;
    margin-top: 2px;
  }
  .class-chip {
    font-family: var(--font-mono);
    font-size: 10px;
    color: var(--text-secondary);
    background: var(--surface-elev);
    border-radius: var(--r-xs);
    padding: 1px 5px;
  }
  .row-actions {
    display: flex;
    align-items: center;
    gap: 8px;
    margin-top: 2px;
    flex-wrap: wrap;
  }
  .refresh-btn {
    padding: 3px 10px;
    background: transparent;
    border: 1px solid var(--teal-line);
    color: var(--teal);
    border-radius: var(--r-sm);
    font-family: var(--font-mono);
    font-size: 10px;
    cursor: pointer;
  }
  .refresh-btn:hover:not(:disabled) {
    background: var(--teal-ghost);
    color: var(--teal-bright);
  }
  .refresh-btn:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }
  .refresh-error {
    font-size: 10px;
    color: var(--fail);
    overflow: hidden;
    text-overflow: ellipsis;
    flex: 1;
    min-width: 0;
  }
  .muted-small {
    color: var(--text-secondary);
    font-size: 10px;
    font-style: italic;
  }
  .muted-small.error {
    color: var(--fail);
  }
  .ghost-text {
    font-family: var(--font-mono);
    font-size: 10px;
    color: var(--text-ghost);
  }
</style>
