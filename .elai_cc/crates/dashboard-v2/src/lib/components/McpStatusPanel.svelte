<script lang="ts">
  // McpStatusPanel -- surfaces MCP security status from the E-24 hardening layer.
  // Shows: schema pin status, last sanitization events, budget enforcement, LLM-guard decisions.

  import { onMount } from 'svelte';

  interface McpSecurityStatus {
    server_url: string;
    pin_status: 'pinned' | 'verified' | 'violation' | 'unknown';
    last_sanitization_count: number;
    last_budget_dropped: string[];
    last_guard_decision: 'allow' | 'deny' | 'ask' | null;
    transport_mode: 'streamable_http' | 'sse_legacy';
  }

  let statuses: McpSecurityStatus[] = $state([]);
  let loading = $state(true);
  let error: string | null = $state(null);

  async function loadMcpStatus() {
    try {
      loading = true;
      const { invoke } = await import('@tauri-apps/api/core');
      statuses = await invoke<McpSecurityStatus[]>('get_mcp_security_status');
      error = null;
    } catch (e) {
      error = e instanceof Error ? e.message : String(e);
    } finally {
      loading = false;
    }
  }

  onMount(() => {
    loadMcpStatus();
  });

  function pinStatusClass(status: McpSecurityStatus['pin_status']): string {
    switch (status) {
      case 'pinned': return 'status-new';
      case 'verified': return 'status-ok';
      case 'violation': return 'status-error';
      default: return 'status-unknown';
    }
  }

  function transportLabel(mode: McpSecurityStatus['transport_mode']): string {
    return mode === 'sse_legacy' ? 'SSE (legacy)' : 'Streamable HTTP';
  }
</script>

<div class="mcp-status-panel">
  <header>
    <h3>MCP Security Status</h3>
    <button onclick={loadMcpStatus} disabled={loading}>
      {loading ? 'Loading...' : 'Refresh'}
    </button>
  </header>

  {#if error}
    <div class="error-banner">{error}</div>
  {/if}

  {#if statuses.length === 0 && !loading}
    <div class="empty-state">No MCP servers connected.</div>
  {:else}
    <ul class="server-list">
      {#each statuses as status}
        <li class="server-row">
          <div class="server-url">{status.server_url}</div>
          <div class="status-grid">
            <span class="label">Schema pin</span>
            <span class={`badge ${pinStatusClass(status.pin_status)}`}>
              {status.pin_status}
            </span>

            <span class="label">Transport</span>
            <span class={`badge ${status.transport_mode === 'sse_legacy' ? 'status-warn' : 'status-ok'}`}>
              {transportLabel(status.transport_mode)}
            </span>

            <span class="label">Sanitizations</span>
            <span class="value">{status.last_sanitization_count} rules applied</span>

            {#if status.last_budget_dropped.length > 0}
              <span class="label">Budget</span>
              <span class="badge status-warn">
                {status.last_budget_dropped.length} tools dropped
              </span>
            {/if}

            {#if status.last_guard_decision !== null}
              <span class="label">LLM guard</span>
              <span class={`badge ${status.last_guard_decision === 'deny' ? 'status-error' : status.last_guard_decision === 'ask' ? 'status-warn' : 'status-ok'}`}>
                {status.last_guard_decision}
              </span>
            {/if}
          </div>
        </li>
      {/each}
    </ul>
  {/if}
</div>

<style>
  .mcp-status-panel {
    font-family: monospace;
    font-size: 13px;
    padding: 12px;
    border: 1px solid #333;
    border-radius: 4px;
    background: #1a1a1a;
    color: #e0e0e0;
  }

  header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: 10px;
  }

  h3 {
    margin: 0;
    font-size: 14px;
    color: #aaa;
  }

  button {
    font-size: 11px;
    padding: 2px 8px;
    background: #2a2a2a;
    border: 1px solid #444;
    color: #ccc;
    cursor: pointer;
    border-radius: 2px;
  }

  button:hover:not(:disabled) {
    background: #333;
  }

  .error-banner {
    color: #ff6b6b;
    font-size: 12px;
    margin-bottom: 8px;
    padding: 4px 8px;
    background: #2a1a1a;
    border-radius: 2px;
  }

  .empty-state {
    color: #666;
    font-style: italic;
    text-align: center;
    padding: 20px;
  }

  .server-list {
    list-style: none;
    margin: 0;
    padding: 0;
    display: flex;
    flex-direction: column;
    gap: 8px;
  }

  .server-row {
    padding: 8px;
    background: #222;
    border-radius: 3px;
  }

  .server-url {
    font-size: 11px;
    color: #88aaff;
    margin-bottom: 6px;
    word-break: break-all;
  }

  .status-grid {
    display: grid;
    grid-template-columns: 100px 1fr;
    gap: 4px 8px;
    align-items: center;
  }

  .label {
    color: #888;
    font-size: 11px;
  }

  .value {
    font-size: 11px;
    color: #ccc;
  }

  .badge {
    font-size: 10px;
    padding: 1px 6px;
    border-radius: 10px;
    display: inline-block;
  }

  .status-ok { background: #1a3a1a; color: #4caf50; border: 1px solid #2a5a2a; }
  .status-error { background: #3a1a1a; color: #f44336; border: 1px solid #5a2a2a; }
  .status-warn { background: #3a2a1a; color: #ff9800; border: 1px solid #5a3a2a; }
  .status-new { background: #1a2a3a; color: #2196f3; border: 1px solid #2a3a5a; }
  .status-unknown { background: #2a2a2a; color: #888; border: 1px solid #444; }
</style>
