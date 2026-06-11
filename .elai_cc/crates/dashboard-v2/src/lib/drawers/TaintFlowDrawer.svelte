<script lang="ts">
  // G-1 / Slice A -- TaintFlowDrawer.
  //
  // Read-only view of recent taint_flow SSB rows via the `get_taint_flows`
  // Tauri command (see crates/dashboard-v2/src-tauri/src/commands_g1.rs).
  //
  // Each row is a LIVE data-plane taint propagation that was BLOCKED at a
  // destructive sink: an untrusted ingest (web body / MCP tool output /
  // external file read / sandbox stdout) flowed into a destructive sink (git
  // commit / file write) and the allow_tool_with_origin capability gate
  // REJECTED the call. The drawer surfaces the ingest Kind, the origin blake3,
  // the joined trust level, and the blocked sink so an operator can see exactly
  // which untrusted->sink flow was stopped.
  //
  // The producer (production_agent_rpc git_commit / edit taint gate) is
  // DEFAULT-ON; the only opt-out is the kill-switch ELAI_TAINT_PROPAGATE_DISABLE.
  // On the default install with no blocked flow the feed is empty. Polls on a
  // 5s cadence (mirrors the G-4 sibling). cross-lang-pair: dashboard/tauri/api

  import { onMount, onDestroy } from 'svelte';
  import type { TaintFlow } from '../api/types';

  let rows = $state<TaintFlow[] | null>(null);
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
      console.warn(`[taint-flow] invoke('${cmd}') failed`, e);
      return undefined;
    }
  }

  async function reload() {
    const out = await callInvoke<TaintFlow[]>('get_taint_flows');
    if (out === undefined) {
      rows = null;
      loadError = null;
    } else {
      rows = out;
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

  function kindLabel(k: string | null | undefined): string {
    switch (k) {
      case 'web':
        return 'Web';
      case 'tool_output':
        return 'MCP tool';
      case 'file':
        return 'File';
      case 'sandbox_output':
        return 'Sandbox';
      default:
        return k ?? '—';
    }
  }

  function shortHash(h: string | null | undefined): string {
    if (!h) return '—';
    return h.length > 12 ? `${h.slice(0, 12)}…` : h;
  }

  let total = $derived(rows ? rows.length : 0);

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

<section class="taint-flow-panel">
  <header class="panel-header">
    <h2>Taint Flows (G-1)</h2>
    <button class="reload-btn" onclick={reload}>Reload</button>
  </header>

  {#if !loaded}
    <p class="muted">Loading taint-flow events…</p>
  {:else if loadError}
    <div class="error-banner">
      <strong>Failed to load taint-flow events.</strong>
      <pre>{loadError}</pre>
    </div>
  {:else if rows === null}
    <p class="muted">
      Tauri IPC unavailable (running in browser preview mode). Mount the
      dashboard in the Tauri host to see live taint-flow data.
    </p>
  {:else}
    <div class="summary-row">
      <span class="summary-label">Tainted flows blocked:</span>
      <span class="summary-value">{total}</span>
    </div>

    {#if rows.length === 0}
      <p class="muted">
        No <code>taint_flow</code> rows recorded. Rows emit when an untrusted
        ingest (web body, MCP tool output, external file read, or sandbox
        stdout) flows into a destructive sink (git commit / file write) and the
        <code>allow_tool_with_origin</code> capability gate blocks it
        fail-closed. The gate is default-ON (kill-switch
        <code>ELAI_TAINT_PROPAGATE_DISABLE</code> only).
      </p>
    {:else}
      <table>
        <thead>
          <tr>
            <th>Time</th>
            <th>Ingest</th>
            <th>Origin (blake3)</th>
            <th>Trust</th>
            <th>Blocked sink</th>
            <th>Task</th>
          </tr>
        </thead>
        <tbody>
          {#each rows as row (row.seq)}
            <tr class="row-taint">
              <td>{fmtTs(row.ts_ms)}</td>
              <td>
                <span class="badge badge-kind">{kindLabel(row.kind)}</span>
              </td>
              <td class="origin-cell" title={row.origin_blake3}
                >{shortHash(row.origin_blake3)}</td
              >
              <td>
                <span class="badge badge-trust">{row.trust_level}</span>
              </td>
              <td class="sink-cell" title={row.sink_tool}>
                <span class="badge badge-sink">{row.sink_tool || '—'}</span>
                {#if row.blocked}<span class="badge badge-blocked">blocked</span>{/if}
              </td>
              <td class="task-cell" title={row.task_id}>{row.task_id}</td>
            </tr>
          {/each}
        </tbody>
      </table>
      <p class="footnote">
        Each row is an untrusted ingest that reached a destructive sink and was
        blocked by the capability gate (CaMeL-lite, arXiv:2503.18813). The join
        is dependency-screened (RTBAS arXiv:2502.08966): a clean argv that does
        not derive from untrusted content is never blocked.
      </p>
    {/if}
  {/if}
</section>

<style>
  .taint-flow-panel {
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
    font-size: 11px;
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
    font-size: 11px;
    text-transform: uppercase;
    letter-spacing: 0.04em;
  }

  .summary-value {
    font-weight: 600;
    font-size: 13px;
    color: rgb(220, 120, 80);
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
    vertical-align: top;
  }

  th {
    font-weight: 500;
    color: var(--text-secondary);
    letter-spacing: 0.04em;
    text-transform: uppercase;
    font-size: 10px;
  }

  .row-taint {
    background: rgba(220, 120, 80, 0.07);
  }

  .origin-cell,
  .task-cell {
    max-width: 200px;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    font-variant-numeric: tabular-nums;
  }

  .badge {
    display: inline-block;
    margin: 0 4px 4px 0;
    padding: 1px 6px;
    border-radius: var(--r-xs);
    font-size: 10px;
    letter-spacing: 0.04em;
    background: var(--bg-hover);
    border: 1px solid var(--border-subtle);
  }

  .badge-kind {
    background: rgba(120, 160, 220, 0.15);
    border-color: rgba(120, 160, 220, 0.4);
  }

  .badge-trust {
    background: rgba(220, 120, 80, 0.15);
    border-color: rgba(220, 120, 80, 0.4);
  }

  .badge-sink {
    background: var(--bg-hover);
  }

  .badge-blocked {
    background: rgba(220, 80, 80, 0.15);
    border-color: rgba(220, 80, 80, 0.4);
    color: rgb(220, 120, 80);
  }

  .muted {
    color: var(--text-secondary);
  }

  .footnote {
    color: var(--text-secondary);
    font-size: 11px;
  }

  .error-banner {
    background: rgba(220, 80, 80, 0.1);
    border: 1px solid rgba(220, 80, 80, 0.4);
    padding: 8px;
    border-radius: var(--r-sm);
  }
</style>
