<script lang="ts">
  // B-31 Phase 4 (Cycle B): Indexing Progress + Cancel surface.
  //
  // Mounts inside KnowledgeDrawer.svelte. Polls `get_indexing_progress`
  // every POLL_INTERVAL_MS while the panel is mounted. Renders a status
  // badge, progress bar (batches_processed / batches_total), files counter,
  // last-checkpoint relative time, and a Cancel button.

  import { onMount, onDestroy } from 'svelte';
  import { Card, Badge, Button } from '$lib/ui';

  type IndexingProgressDto = {
    status: 'in-progress' | 'completed' | 'cancelled' | 'unknown';
    project_id: string;
    project_path?: string;
    indexed_files: number;
    total_files: number;
    batch_size?: number;
    batches_processed: number;
    batches_total: number | null;
    last_checkpoint_at: number;
    schema_version?: number;
  };

  const POLL_INTERVAL_MS = 1000;

  let progress = $state<IndexingProgressDto | null>(null);
  let lastError = $state<string | null>(null);
  let cancelInFlight = $state(false);
  let pollTimer: ReturnType<typeof setInterval> | null = null;
  let tauriInvoke: (<T>(cmd: string, args?: Record<string, unknown>) => Promise<T>) | null = null;

  async function ensureInvoke(): Promise<typeof tauriInvoke> {
    if (tauriInvoke !== null) return tauriInvoke;
    const w = typeof window !== 'undefined' ? (window as unknown as Record<string, unknown>) : {};
    if (!w.__TAURI_INTERNALS__ && !w.__TAURI__) {
      return null;
    }
    try {
      const mod = await import('@tauri-apps/api/core');
      tauriInvoke = mod.invoke;
    } catch (err) {
      console.warn('[IndexingProgressPanel] @tauri-apps/api/core import failed', err);
      tauriInvoke = null;
    }
    return tauriInvoke;
  }

  async function refresh(): Promise<void> {
    try {
      const inv = await ensureInvoke();
      if (!inv) {
        progress = null;
        return;
      }
      const result = await inv<IndexingProgressDto>('get_indexing_progress', {});
      progress = result;
      lastError = null;
    } catch (err) {
      lastError = err instanceof Error ? err.message : String(err);
      console.debug('[IndexingProgressPanel] refresh failed', err);
    }
  }

  async function cancel(): Promise<void> {
    if (cancelInFlight) return;
    cancelInFlight = true;
    try {
      const inv = await ensureInvoke();
      if (!inv) {
        lastError = 'tauri unavailable';
        return;
      }
      await inv<{ ok: boolean; was_running: boolean }>('cancel_indexing', {});
      // Immediate refresh so the badge updates without waiting for poll.
      await refresh();
    } catch (err) {
      lastError = err instanceof Error ? err.message : String(err);
      console.warn('[IndexingProgressPanel] cancel_indexing failed', err);
    } finally {
      cancelInFlight = false;
    }
  }

  onMount(() => {
    void refresh();
    pollTimer = setInterval(() => {
      void refresh();
    }, POLL_INTERVAL_MS);
  });

  onDestroy(() => {
    if (pollTimer !== null) {
      clearInterval(pollTimer);
      pollTimer = null;
    }
  });

  const statusTone = $derived.by((): string => {
    const s = progress?.status ?? 'unknown';
    if (s === 'completed') return 'teal';
    if (s === 'in-progress') return 'warn';
    if (s === 'cancelled') return 'fail';
    return 'ghost';
  });

  const progressPct = $derived.by((): number => {
    if (!progress) return 0;
    const total = progress.batches_total ?? 0;
    if (total === 0) return 0;
    const pct = (progress.batches_processed / total) * 100;
    if (!Number.isFinite(pct)) return 0;
    return Math.max(0, Math.min(100, pct));
  });

  function fmtRelativeTime(unixSecs: number): string {
    if (!unixSecs || unixSecs <= 0) return 'never';
    const nowSecs = Math.floor(Date.now() / 1000);
    const delta = nowSecs - unixSecs;
    if (delta < 0) return 'just now';
    if (delta < 60) return `${delta}s ago`;
    if (delta < 3600) return `${Math.floor(delta / 60)}m ago`;
    if (delta < 86400) return `${Math.floor(delta / 3600)}h ago`;
    return `${Math.floor(delta / 86400)}d ago`;
  }

  const canCancel = $derived(progress?.status === 'in-progress');
</script>

<Card variant="plain" padding={12}>
  <div class="section-label">INDEXING PROGRESS</div>
  {#if progress === null}
    <span class="ghost-text">no indexing run recorded</span>
  {:else}
    <div class="row">
      <Badge tone={statusTone} size="xs">{progress.status.toUpperCase()}</Badge>
      <span class="metric">
        batch {progress.batches_processed}{progress.batches_total !== null
          ? ` / ${progress.batches_total}`
          : ''}
      </span>
      <span class="ghost-text">checkpoint {fmtRelativeTime(progress.last_checkpoint_at)}</span>
    </div>
    <div class="bar-wrap">
      <div class="bar">
        <div class="bar-fill" style="width: {progressPct}%;"></div>
      </div>
    </div>
    <div class="row">
      <span class="metric">files {progress.indexed_files} / {progress.total_files}</span>
      <Button variant="ghost" size="sm" disabled={!canCancel || cancelInFlight} onclick={() => void cancel()}>
        {cancelInFlight ? 'cancelling…' : 'Cancel'}
      </Button>
    </div>
    {#if lastError !== null}
      <div class="error-row">error: {lastError}</div>
    {/if}
  {/if}
</Card>

<style>
  .section-label {
    font-family: var(--font-mono);
    font-size: 11px;
    color: var(--text-secondary);
    letter-spacing: 0.08em;
    text-transform: uppercase;
    margin-bottom: 8px;
  }

  .row {
    display: flex;
    align-items: center;
    gap: 8px;
    margin-top: 4px;
  }

  .metric {
    font-family: var(--font-mono);
    font-size: 11px;
    color: var(--text-primary);
  }

  .ghost-text {
    font-family: var(--font-mono);
    font-size: 10px;
    color: var(--text-ghost);
    margin-left: auto;
  }

  .bar-wrap {
    margin: 6px 0;
  }

  .bar {
    width: 100%;
    height: 4px;
    background: var(--bg-input);
    border-radius: var(--r-xs);
    overflow: hidden;
  }

  .bar-fill {
    height: 100%;
    background: var(--teal-bright);
    transition: width 0.3s ease;
  }

  .error-row {
    font-family: var(--font-mono);
    font-size: 10px;
    color: var(--fail);
    margin-top: 4px;
  }
</style>
