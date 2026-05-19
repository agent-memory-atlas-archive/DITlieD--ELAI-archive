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

  // C-17 (DRIFT-J): resilience state DTO surfaced by get_indexing_resilience.
  // Mirrors crates/orchestrator/src/watcher_service.rs lock guard +
  // crates/orchestrator/src/startup.rs::SHUTDOWN_DEADLINE.
  type IndexingResilienceDto = {
    auto_resume_enabled: boolean;
    watch_locks_held: number;
    last_checkpoint_unix_secs: number;
    last_resume_from_batch: number | null;
    last_resume_files_skipped: number | null;
    lock_contested_count: number;
    shutdown_deadline_secs: number;
    last_batch_duration_ms: number | null;
  };

  const POLL_INTERVAL_MS = 1000;

  let progress = $state<IndexingProgressDto | null>(null);
  let resilience = $state<IndexingResilienceDto | null>(null);
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
        resilience = null;
        return;
      }
      const result = await inv<IndexingProgressDto>('get_indexing_progress', {});
      progress = result;
      // C-17: best-effort resilience read; falls back to null if the
      // tauri handler is not yet wired (older dashboard builds).
      try {
        resilience = await inv<IndexingResilienceDto>('get_indexing_resilience', {});
      } catch (resErr) {
        resilience = null;
        // No console spam — the handler may not exist on older builds.
        void resErr;
      }
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

  <!-- C-17 (DRIFT-J): indexing resilience state. Renders auto_resume
       toggle status, active Watch-scope lock count, last checkpoint
       relative time, IndexResume banner (when applicable), and
       LockContested banner (when applicable). -->
  {#if resilience !== null}
    <div class="resilience-divider"></div>
    <div class="section-label">RESILIENCE</div>
    <div class="row">
      <Badge tone={resilience.auto_resume_enabled ? 'teal' : 'ghost'} size="xs">
        {resilience.auto_resume_enabled ? 'AUTO-RESUME' : 'NO AUTO-RESUME'}
      </Badge>
      <span class="metric">watch locks {resilience.watch_locks_held}</span>
      <span class="ghost-text">deadline {resilience.shutdown_deadline_secs}s</span>
    </div>
    {#if resilience.last_resume_from_batch !== null}
      <div class="resume-banner">
        <Badge tone="teal" size="xs">RESUMED</Badge>
        <span class="metric">
          from batch {resilience.last_resume_from_batch}
          (skipped {resilience.last_resume_files_skipped ?? 0} files)
        </span>
      </div>
    {/if}
    {#if resilience.lock_contested_count > 0}
      <div class="lock-banner">
        <Badge tone="warn" size="xs">LOCK CONTESTED</Badge>
        <span class="metric">
          another ELAI process is indexing ({resilience.lock_contested_count} event{resilience.lock_contested_count === 1 ? '' : 's'})
        </span>
      </div>
    {/if}
    {#if resilience.last_batch_duration_ms !== null}
      <div class="row">
        <span class="metric">last batch {resilience.last_batch_duration_ms} ms</span>
        <span class="ghost-text">
          ckpt {fmtRelativeTime(resilience.last_checkpoint_unix_secs)}
        </span>
      </div>
    {/if}
  {/if}
</Card>

<style>
  .section-label {
    font-family: var(--font-mono);
    font-size: 10px;
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
    font-size: 10px;
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

  /* C-17 (DRIFT-J): resilience banner styling — re-uses existing tokens
     so no new design tokens are introduced. */
  .resilience-divider {
    margin: 12px 0 6px;
    border-top: 1px solid var(--border-subtle);
  }

  .resume-banner,
  .lock-banner {
    display: flex;
    align-items: center;
    gap: 8px;
    margin-top: 4px;
  }
</style>
