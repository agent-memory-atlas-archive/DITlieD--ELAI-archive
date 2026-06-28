<!--
  C-43 — Phase 4H scope-freeze status badge.

  Minimal widget: renders a green or red badge plus the last-ratified
  wall-clock timestamp. Reads via Tauri `scope_freeze_status` (backed
  by crates/dashboard-v2/src-tauri/src/commands_c43.rs). When Tauri is
  unavailable (browser preview), renders a neutral "offline" badge.

  Grounding posture: ELAI design hypothesis. The source plan cited
  arXiv:2604.21725 (AEL) as ground truth but two independent
  WebFetches confirmed that paper is Thompson-Sampling agent learning,
  NOT scope-freeze. The citation has been removed across all C-43
  deliverables. See workflow/sessions/s-25120/mistakes.md for the
  bad-cite analysis.
-->
<script lang="ts">
  import { onDestroy, onMount } from 'svelte';

  type FreezeViolation = {
    item_path: string;
    item_path_hash: number;
    frozen_since_ms: number;
    modified_at_ms: number;
  };

  type FreezeStatus = {
    clean: boolean;
    ratified_at_ms: number;
    verified_at_ms: number;
    violation_count: number;
    violations: FreezeViolation[];
    error: string | null;
  };

  const REFRESH_INTERVAL_MS = 15_000;

  let status: FreezeStatus | null = $state(null);
  let loading = $state(true);
  let pollHandle: ReturnType<typeof setInterval> | null = null;

  async function fetchStatus(): Promise<void> {
    try {
      const mod = await import('@tauri-apps/api/core');
      const result = await mod.invoke<FreezeStatus>('scope_freeze_status', {});
      status = result;
    } catch (e) {
      // Browser preview or Tauri unavailable; surface as offline state.
      status = {
        clean: false,
        ratified_at_ms: 0,
        verified_at_ms: 0,
        violation_count: 0,
        violations: [],
        error: 'tauri unavailable',
      };
    } finally {
      loading = false;
    }
  }

  onMount(() => {
    void fetchStatus();
    pollHandle = setInterval(() => void fetchStatus(), REFRESH_INTERVAL_MS);
  });

  onDestroy(() => {
    if (pollHandle !== null) {
      clearInterval(pollHandle);
      pollHandle = null;
    }
  });

  function formatTs(ms: number): string {
    if (!ms) return 'never';
    try {
      return new Date(ms).toISOString().replace('T', ' ').slice(0, 19);
    } catch {
      return String(ms);
    }
  }

  const badgeLabel = $derived.by(() => {
    if (loading) return 'freeze: loading';
    if (status === null) return 'freeze: unknown';
    if (status.error) return `freeze: ${status.error}`;
    if (status.clean) return 'freeze: clean';
    return `freeze: ${status.violation_count} violation(s)`;
  });

  const badgeClass = $derived.by(() => {
    if (loading || status === null) return 'badge badge-neutral';
    if (status.clean) return 'badge badge-clean';
    return 'badge badge-violated';
  });

  const ratifiedLabel = $derived.by(() => {
    if (status === null) return '';
    return `ratified: ${formatTs(status.ratified_at_ms)}`;
  });
</script>

<div class="freeze-status-bar" title={ratifiedLabel}>
  <span class={badgeClass}>{badgeLabel}</span>
  {#if status !== null && status.ratified_at_ms > 0}
    <span class="meta">{ratifiedLabel}</span>
  {/if}
</div>

<style>
  .freeze-status-bar {
    display: inline-flex;
    align-items: center;
    gap: 0.5rem;
    font-family: var(--font-mono, monospace);
    font-size: 0.75rem;
    line-height: 1;
  }

  .badge {
    padding: 2px 6px;
    border-radius: 4px;
    font-weight: 600;
  }

  .badge-clean {
    background: var(--pass);
    color: var(--text-primary);
  }

  .badge-violated {
    background: var(--fail);
    color: var(--text-primary);
  }

  .badge-neutral {
    background: var(--text-disabled);
    color: var(--text-primary);
  }

  .meta {
    color: var(--text-secondary);
  }
</style>
