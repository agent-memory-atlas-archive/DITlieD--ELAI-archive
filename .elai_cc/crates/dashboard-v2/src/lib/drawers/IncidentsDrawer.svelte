<script lang="ts">
  // E-30 -- IncidentsDrawer.
  //
  // Read-only view of recent E-30 public-agent-incident rows via the
  // `get_recent_incidents` Tauri command
  // (see crates/dashboard-v2/src-tauri/src/commands_e30.rs).
  //
  // Renders:
  //   * last 50 incident rows across 8 action_types, newest first
  //   * per-row kind badge with severity color
  //   * argv events: argv_hash + reason + citation
  //   * trust dialog events: defer_reason
  //   * mcp pin events: server_name + pinned vs observed version
  //   * lineage events: auditor_lineage vs executor_lineage
  //   * memory events: text_source
  //   * postmortems: incident_id + title + source_url
  //
  // No producer logic lives here -- backend wiring lives across
  // crates/orchestrator/src/hook_dispatcher.rs (argv allowlist),
  // crates/safety (trust_dialog + check_argv), crates/plugins (mcp pins),
  // crates/orchestrator/src/per_role_runtime.rs (lineage guard),
  // crates/orchestrator/src/conversation.rs (TextSource), and
  // crates/cli/src/main.rs (incident record cmd).
  //
  // Polls on a 10s cadence to mirror the sibling TripwirePanel.

  import { onMount, onDestroy } from 'svelte';

  type IncidentKind =
    | 'argv_destructive_denied'
    | 'argv_approval_required'
    | 'trust_dialog_hook_deferred'
    | 'trust_dialog_env_deferred'
    | 'mcp_pin_version_refused'
    | 'auditor_lineage_collision'
    | 'untrusted_memory_write_blocked'
    | 'incident_record_created';

  type IncidentRow = {
    seq: number;
    ts: number;
    kind: IncidentKind;
    argv_hash: string | null;
    reason: string | null;
    citation: string | null;
    defer_reason: string | null;
    server_name: string | null;
    pinned_version: string | null;
    observed_version: string | null;
    auditor_lineage: string | null;
    executor_lineage: string | null;
    text_source: string | null;
    incident_id: string | null;
    title: string | null;
    source_url: string | null;
  };

  let rows = $state<IncidentRow[]>([]);
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
    // Tauri is present: let a real invoke error PROPAGATE so reload() can surface
    // it. Swallowing here would make a backend failure indistinguishable from
    // "no incidents" on a security surface.
    const mod = await import('@tauri-apps/api/core');
    return (await mod.invoke<T>(cmd, args)) as T;
  }

  async function reload() {
    try {
      const out = await callInvoke<IncidentRow[]>('get_recent_incidents', {
        limit: 50,
      });
      if (out === undefined) {
        // No Tauri runtime (browser-dev / SSR): benign, render the empty state.
        rows = [];
        loadError = null;
      } else {
        rows = out;
        loadError = null;
      }
    } catch (e) {
      // Tauri present but the command failed: surface it, do not mask as empty.
      loadError = String(e);
    } finally {
      loaded = true;
    }
  }

  function shortHash(s: string | null | undefined): string {
    if (!s) return '—';
    if (s.length <= 16) return s;
    return `${s.slice(0, 8)}…${s.slice(-4)}`;
  }

  function fmtTs(ms: number | null | undefined): string {
    if (!ms) return '—';
    try {
      return new Date(ms).toISOString().slice(0, 19).replace('T', ' ');
    } catch {
      return '—';
    }
  }

  function kindLabel(kind: IncidentKind): string {
    switch (kind) {
      case 'argv_destructive_denied':
        return 'argv DENY';
      case 'argv_approval_required':
        return 'argv APPROVAL';
      case 'trust_dialog_hook_deferred':
        return 'trust HOOK';
      case 'trust_dialog_env_deferred':
        return 'trust ENV';
      case 'mcp_pin_version_refused':
        return 'mcp PIN';
      case 'auditor_lineage_collision':
        return 'lineage COLLISION';
      case 'untrusted_memory_write_blocked':
        return 'memory UNTRUSTED';
      case 'incident_record_created':
        return 'postmortem';
      default:
        return kind;
    }
  }

  function kindSeverity(kind: IncidentKind): 'block' | 'warn' | 'info' {
    switch (kind) {
      case 'argv_destructive_denied':
      case 'mcp_pin_version_refused':
      case 'untrusted_memory_write_blocked':
        return 'block';
      case 'argv_approval_required':
      case 'trust_dialog_hook_deferred':
      case 'trust_dialog_env_deferred':
      case 'auditor_lineage_collision':
        return 'warn';
      case 'incident_record_created':
        return 'info';
      default:
        return 'info';
    }
  }

  function rowDetail(row: IncidentRow): string {
    switch (row.kind) {
      case 'argv_destructive_denied':
      case 'argv_approval_required':
        return row.reason ?? '';
      case 'trust_dialog_hook_deferred':
      case 'trust_dialog_env_deferred':
        return row.defer_reason ?? row.reason ?? '';
      case 'mcp_pin_version_refused':
        return `${row.server_name ?? '?'}: pinned=${row.pinned_version ?? '?'} observed=${row.observed_version ?? '?'}`;
      case 'auditor_lineage_collision':
        return `auditor=${row.auditor_lineage ?? '?'} executor=${row.executor_lineage ?? '?'}`;
      case 'untrusted_memory_write_blocked':
        return `source=${row.text_source ?? '?'}`;
      case 'incident_record_created':
        return row.title ?? row.incident_id ?? '';
      default:
        return '';
    }
  }

  onMount(() => {
    reload();
    pollHandle = setInterval(reload, 10_000);
  });

  onDestroy(() => {
    if (pollHandle) {
      clearInterval(pollHandle);
      pollHandle = null;
    }
  });
</script>

<section class="incidents-panel">
  <header class="panel-header">
    <h2>Incidents (E-30)</h2>
    <button class="reload-btn" onclick={reload}>Reload</button>
  </header>

  {#if !loaded}
    <p class="muted">Loading incident rows…</p>
  {:else if loadError}
    <div class="error-banner">
      <strong>Failed to load incident rows.</strong>
      <pre>{loadError}</pre>
    </div>
  {:else if rows.length === 0}
    <p class="muted">
      No public-agent-incident rows recorded yet. Rows are emitted by the
      orchestrator after argv-allowlist denies a destructive verb, trust
      dialog defers a hook/env load, MCP pin enforcement refuses a server,
      auditor/executor lineage collides, or an untrusted memory write is
      blocked. Postmortems land via <code>elai incident record</code>.
    </p>
  {:else}
    <table>
      <thead>
        <tr>
          <th>Time</th>
          <th>Kind</th>
          <th>Detail</th>
          <th>Hash / Ref</th>
          <th>Citation / Source</th>
        </tr>
      </thead>
      <tbody>
        {#each rows as row (row.seq)}
          {@const sev = kindSeverity(row.kind)}
          <tr class="row-{sev}">
            <td>{fmtTs(row.ts)}</td>
            <td><span class="badge badge-{sev}">{kindLabel(row.kind)}</span></td>
            <td>{rowDetail(row)}</td>
            <td>
              {#if row.argv_hash}
                <code title={row.argv_hash}>{shortHash(row.argv_hash)}</code>
              {:else if row.incident_id}
                <code title={row.incident_id}>{shortHash(row.incident_id)}</code>
              {:else}
                —
              {/if}
            </td>
            <td>
              {#if row.citation}
                <span class="citation">{row.citation}</span>
              {:else if row.source_url}
                <a class="citation" href={row.source_url} target="_blank" rel="noreferrer"
                  >{row.source_url}</a
                >
              {:else}
                —
              {/if}
            </td>
          </tr>
        {/each}
      </tbody>
    </table>
  {/if}
</section>

<style>
  .incidents-panel {
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

  .citation {
    color: var(--text-secondary);
    font-size: 10px;
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
