<script lang="ts">
  // G-4 / Slice A -- McpRugPullDrawer.
  //
  // Read-only view of recent mcp_rug_pull_blocked SSB rows via the
  // `get_mcp_rug_pulls` Tauri command
  // (see crates/dashboard-v2/src-tauri/src/commands_g4.rs).
  //
  // Each row is a LIVE MCP rug-pull / line-jump: a reconnect-time tools/list
  // re-fetch from a previously-approved server whose blake3 schema pin no longer
  // matched. The changed tool(s) were BLOCKED fail-closed until operator
  // re-approval (update_pin). The drawer surfaces the server, the before/after
  // schema hashes (the diff), the changed tool names (the line-jump set), and
  // the description-diff count so an operator can decide whether to re-approve.
  //
  // The producer (DefaultMcpBridge::http_list_tools re-pin gate) is DEFAULT-ON;
  // the only opt-out is the kill-switch ELAI_MCP_REPIN_DISABLE. On the default
  // install with no rug-pull the feed is empty. Polls on a 5s cadence (mirrors
  // the F-13 sibling). cross-lang-pair: dashboard/tauri/api

  import { onMount, onDestroy } from 'svelte';
  import type { McpRugPull } from '../api/types';

  let rows = $state<McpRugPull[] | null>(null);
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
      console.warn(`[mcp-rug-pull] invoke('${cmd}') failed`, e);
      return undefined;
    }
  }

  async function reload() {
    const out = await callInvoke<McpRugPull[]>('get_mcp_rug_pulls');
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

<section class="mcp-rugpull-panel">
  <header class="panel-header">
    <h2>MCP Rug-Pulls (G-4)</h2>
    <button class="reload-btn" onclick={reload}>Reload</button>
  </header>

  {#if !loaded}
    <p class="muted">Loading MCP rug-pull events…</p>
  {:else if loadError}
    <div class="error-banner">
      <strong>Failed to load MCP rug-pull events.</strong>
      <pre>{loadError}</pre>
    </div>
  {:else if rows === null}
    <p class="muted">
      Tauri IPC unavailable (running in browser preview mode). Mount the
      dashboard in the Tauri host to see live rug-pull data.
    </p>
  {:else}
    <div class="summary-row">
      <span class="summary-label">Rug-pulls blocked:</span>
      <span class="summary-value">{total}</span>
    </div>

    {#if rows.length === 0}
      <p class="muted">
        No <code>mcp_rug_pull_blocked</code> rows recorded. Rows emit when a
        reconnect-time <code>tools/list</code> re-fetch from a previously-approved
        MCP server no longer matches its blake3 schema pin (a rug-pull /
        line-jump). The changed tool(s) are blocked fail-closed until operator
        re-approval. The gate is default-ON (kill-switch
        <code>ELAI_MCP_REPIN_DISABLE</code> only).
      </p>
    {:else}
      <table>
        <thead>
          <tr>
            <th>Time</th>
            <th>Server</th>
            <th>Changed tools</th>
            <th class="num">Diffs</th>
            <th>Before -&gt; After (blake3)</th>
            <th>Origin</th>
          </tr>
        </thead>
        <tbody>
          {#each rows as row (row.seq)}
            <tr class="row-rugpull">
              <td>{fmtTs(row.ts_ms)}</td>
              <td class="server-cell" title={row.server_url}>{row.server_url}</td>
              <td class="tools-cell">
                {#each row.changed_tools as t}
                  <span class="badge badge-tool" title={t}>{t}</span>
                {/each}
              </td>
              <td class="num">{row.description_diff_count}</td>
              <td class="diff-cell">
                <span class="hash hash-before" title={row.schema_blake3_before}
                  >{shortHash(row.schema_blake3_before)}</span
                >
                <span class="arrow">-&gt;</span>
                <span class="hash hash-after" title={row.schema_blake3_after}
                  >{shortHash(row.schema_blake3_after)}</span
                >
              </td>
              <td class="origin-cell" title={row.ingest_origin}>{row.ingest_origin}</td>
            </tr>
          {/each}
        </tbody>
      </table>
      <p class="footnote">
        Blocked tools stay blocked until an operator re-approves the new schema
        (<code>update_pin</code>). The pin is never auto-updated on a rug-pull.
      </p>
    {/if}
  {/if}
</section>

<style>
  .mcp-rugpull-panel {
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

  th.num,
  td.num {
    text-align: right;
    font-variant-numeric: tabular-nums;
  }

  th {
    font-weight: 500;
    color: var(--text-secondary);
    letter-spacing: 0.04em;
    text-transform: uppercase;
    font-size: 10px;
  }

  .row-rugpull {
    background: rgba(220, 120, 80, 0.07);
  }

  .server-cell,
  .origin-cell {
    max-width: 200px;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .tools-cell {
    max-width: 220px;
  }

  .diff-cell {
    display: flex;
    align-items: center;
    gap: 6px;
    white-space: nowrap;
  }

  .hash {
    font-variant-numeric: tabular-nums;
  }

  .hash-before {
    color: var(--text-secondary);
  }

  .hash-after {
    color: rgb(220, 120, 80);
  }

  .arrow {
    color: var(--text-secondary);
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

  .badge-tool {
    background: rgba(220, 120, 80, 0.15);
    border-color: rgba(220, 120, 80, 0.4);
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
