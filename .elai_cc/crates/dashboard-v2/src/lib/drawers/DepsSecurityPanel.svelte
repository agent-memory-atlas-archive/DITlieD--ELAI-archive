<script lang="ts">
  // F-9 / Slice D5 -- DepsSecurityPanel.
  //
  // Read-only view of recent WriteTimeSlopsquatBlocked SSB rows via the
  // `write_time_slop_status` Tauri command
  // (see crates/dashboard-v2/src-tauri/src/commands_f9.rs).
  //
  // Renders:
  //   * summary header: total blocked count + per-verdict-kind breakdown
  //   * last 50 WriteTimeSlopsquatBlocked rows, newest first
  //   * per-row: timestamp, ecosystem badge, verdict-kind badge,
  //     dep_name_hash (hex), registry_age_days (when present)
  //   * "env gate inactive" hint when ELAI_HOOKS_DISABLED is set
  //
  // Default-ON (B1/W-2): rows emit with no env override; gate is always
  // active unless ELAI_HOOKS_DISABLED kills the whole hook dispatcher.
  // Polls on a 5s cadence (mirrors VibeGateDrawer sibling pattern).
  // cross-lang-pair: dashboard/tauri/api

  import { onMount, onDestroy } from 'svelte';
  import type { WriteTimeSlopStatusDto, WriteTimeSlopRow } from '../api/types';

  let statusDto = $state<WriteTimeSlopStatusDto | null>(null);
  let loaded = $state(false);
  let loadError = $state<string | null>(null);
  let pollHandle: ReturnType<typeof setInterval> | null = null;

  async function callInvoke<T>(
    cmd: string,
    args?: Record<string, unknown>
  ): Promise<T | undefined> {
    if (typeof window === 'undefined') return undefined;
    const tauriAvail = !!(window as unknown as { __TAURI_INTERNALS__?: unknown })
      .__TAURI_INTERNALS__;
    if (!tauriAvail) return undefined;
    try {
      const mod = await import('@tauri-apps/api/core');
      return (await mod.invoke<T>(cmd, args)) as T;
    } catch (e) {
      console.warn(`[deps-security] invoke('${cmd}') failed`, e);
      return undefined;
    }
  }

  async function reload() {
    const out = await callInvoke<WriteTimeSlopStatusDto>('write_time_slop_status');
    if (out === undefined) {
      statusDto = null;
      loadError = null;
    } else {
      statusDto = out;
      loadError = null;
    }
    loaded = true;
  }

  function fmtTs(ms: number | null | undefined): string {
    if (!ms) return '—';
    try {
      return new Date(ms).toISOString().slice(0, 19).replace('T', ' ');
    } catch {
      return '—';
    }
  }

  function fmtHash(n: number | null | undefined): string {
    if (n == null) return '—';
    return `0x${n.toString(16).padStart(16, '0')}`;
  }

  function verdictLabel(kind: string): string {
    switch (kind) {
      case 'not_found':
        return 'NOT FOUND';
      case 'too_new':
        return 'TOO NEW';
      case 'registry_unreachable':
        return 'UNREACHABLE';
      default:
        return kind.toUpperCase();
    }
  }

  function verdictSeverity(kind: string): 'block' | 'warn' | 'info' {
    switch (kind) {
      case 'not_found':
        return 'block';
      case 'too_new':
        return 'warn';
      case 'registry_unreachable':
        return 'info';
      default:
        return 'info';
    }
  }

  function ecosystemLabel(eco: string): string {
    switch (eco) {
      case 'cargo':
        return 'CARGO';
      case 'pypi':
        return 'PYPI';
      case 'npm':
        return 'NPM';
      default:
        return eco.toUpperCase();
    }
  }

  onMount(() => {
    reload();
    pollHandle = setInterval(reload, 5_000);
  });

  onDestroy(() => {
    if (pollHandle) {
      clearInterval(pollHandle);
      pollHandle = null;
    }
  });
</script>

<section class="deps-security-panel">
  <header class="panel-header">
    <h2>Deps Security (F-9)</h2>
    <button class="reload-btn" onclick={reload}>Reload</button>
  </header>

  {#if !loaded}
    <p class="muted">Loading dep security rows…</p>
  {:else if loadError}
    <div class="error-banner">
      <strong>Failed to load dep security rows.</strong>
      <pre>{loadError}</pre>
    </div>
  {:else if statusDto === null}
    <p class="muted">
      Tauri IPC unavailable (running in browser preview mode). Mount the
      dashboard in the Tauri host to see live dep-security data.
    </p>
  {:else if !statusDto.env_active}
    <p class="muted">
      Dep-writetime gate inactive. The global hook dispatcher is disabled
      (<code>ELAI_HOOKS_DISABLED=1</code>). No
      <code>WriteTimeSlopsquatBlocked</code> rows are written until the hooks
      are re-enabled.
    </p>
  {:else}
    <div class="summary-row">
      <span class="summary-label">Total blocked:</span>
      <span class="summary-value">{statusDto.total_blocked}</span>
      {#each statusDto.counts_by_verdict as cv (cv.verdict_kind)}
        {@const sev = verdictSeverity(cv.verdict_kind)}
        <span class="badge badge-{sev}">{verdictLabel(cv.verdict_kind)} &times;{cv.count}</span>
      {/each}
    </div>

    {#if statusDto.recent_rows.length === 0}
      <p class="muted">
        No <code>WriteTimeSlopsquatBlocked</code> rows recorded yet. Rows are
        emitted by <code>DepWriteTimeGate::analyze</code> in the orchestrator
        <code>InProcessHookDispatcher</code> when a dependency import resolves
        to <code>NotFound</code> or <code>TooNew</code> in the registry.
      </p>
    {:else}
      <table>
        <thead>
          <tr>
            <th>Time</th>
            <th>Ecosystem</th>
            <th>Verdict</th>
            <th>Dep Hash</th>
            <th>Age (days)</th>
          </tr>
        </thead>
        <tbody>
          {#each statusDto.recent_rows as row (row.seq)}
            {@const sev = verdictSeverity(row.verdict_kind)}
            <tr class="row-{sev}">
              <td>{fmtTs(row.ts_ms)}</td>
              <td><span class="badge badge-eco">{ecosystemLabel(row.ecosystem)}</span></td>
              <td><span class="badge badge-{sev}">{verdictLabel(row.verdict_kind)}</span></td>
              <td><code class="hash">{fmtHash(row.dep_name_hash)}</code></td>
              <td>{row.registry_age_days != null ? row.registry_age_days : '—'}</td>
            </tr>
          {/each}
        </tbody>
      </table>
    {/if}
  {/if}
</section>

<style>
  .deps-security-panel {
    display: flex;
    flex-direction: column;
    gap: 12px;
    font-family: var(--font-mono);
    font-size: 12px;
  }

  .panel-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
  }

  .panel-header h2 {
    margin: 0;
    font-size: 13px;
    font-weight: 500;
    letter-spacing: 0.04em;
    text-transform: uppercase;
  }

  .reload-btn {
    font-family: var(--font-mono);
    font-size: 10px;
    padding: 2px 8px;
    background: var(--bg-hover);
    border: 1px solid var(--border-default);
    border-radius: var(--r-xs);
    color: var(--text-primary);
    cursor: pointer;
  }

  .reload-btn:hover {
    background: var(--bg-surface);
  }

  .summary-row {
    display: flex;
    align-items: center;
    gap: 8px;
    flex-wrap: wrap;
  }

  .summary-label {
    color: var(--text-secondary);
    font-size: 10px;
    text-transform: uppercase;
    letter-spacing: 0.04em;
  }

  .summary-value {
    font-weight: 600;
    font-size: 13px;
  }

  table {
    width: 100%;
    border-collapse: collapse;
  }

  th,
  td {
    text-align: left;
    padding: 4px 6px;
    border-bottom: 1px solid var(--border-subtle);
  }

  th {
    font-weight: 500;
    color: var(--text-secondary);
    letter-spacing: 0.04em;
    text-transform: uppercase;
    font-size: 10px;
  }

  .row-block {
    background: rgba(220, 80, 80, 0.07);
  }

  .row-warn {
    background: rgba(220, 180, 80, 0.05);
  }

  .badge {
    display: inline-block;
    padding: 1px 6px;
    border-radius: var(--r-xs);
    font-size: 10px;
    letter-spacing: 0.04em;
    text-transform: uppercase;
    background: var(--bg-hover);
    border: 1px solid var(--border-subtle);
  }

  .badge-block {
    background: rgba(220, 80, 80, 0.15);
    border-color: rgba(220, 80, 80, 0.4);
  }

  .badge-warn {
    background: rgba(220, 180, 80, 0.15);
    border-color: rgba(220, 180, 80, 0.4);
  }

  .badge-info {
    background: rgba(80, 150, 220, 0.15);
    border-color: rgba(80, 150, 220, 0.4);
  }

  .badge-eco {
    background: rgba(80, 200, 150, 0.12);
    border-color: rgba(80, 200, 150, 0.35);
  }

  .hash {
    font-size: 10px;
    color: var(--text-secondary);
    letter-spacing: 0.02em;
  }

  .muted {
    color: var(--text-secondary);
  }

  .error-banner {
    background: rgba(220, 80, 80, 0.1);
    border: 1px solid rgba(220, 80, 80, 0.4);
    padding: 8px;
    border-radius: var(--r-sm);
  }
</style>
