<script lang="ts">
  // D-9 — LockedKeysPanel.
  //
  // Read-only view of the L3 frozen-surface registry + recent audit
  // runs via the `get_locked_keys_status` +
  // `get_locked_keys_recent_audits` Tauri commands
  // (additive — see crates/dashboard-v2/src-tauri/src/commands_d9.rs).
  //
  // Renders:
  //   * registry header: path, entry count, blake3 (truncated), load ts
  //   * registry entries: path / line_range / reason
  //   * last 50 locked_keys_audit_run rows, sorted by ts_ms desc
  //   * per-row violation breakdown grouped by audit_id
  //
  // No producer logic lives here — the orchestrator producer
  // (`LockedKeysAuditor::audit_diff_ranged`) emits the SSB rows
  // separately. This panel polls on a 10s cadence matching the other
  // D-{1..7} panels.
  //
  // Grounding: arxiv:1908.04734 (Everitt current-RF principle).

  import { onMount, onDestroy } from 'svelte';

  type LockedKeysEntry = {
    path: string;
    line_range: number[];
    reason: string;
  };

  type LockedKeysStatus = {
    registry_path: string;
    n_entries: number;
    registry_blake3: string;
    loaded_ts_ms: number;
    entries: LockedKeysEntry[];
    error: string | null;
  };

  type AuditRunRow = {
    audit_id: string;
    ts_ms: number;
    n_audited: number;
    n_violations: number;
    registry_blake3: string;
    registry_path: string;
  };

  type ViolationRow = {
    audit_id: string;
    ts_ms: number;
    attempted_edit: string;
    attempted_lines: number[];
    locked_key_path: string;
    reason: string;
  };

  type RecentAudits = {
    runs: AuditRunRow[];
    violations_by_audit: Record<string, ViolationRow[]>;
  };

  let status = $state<LockedKeysStatus | null>(null);
  let recent = $state<RecentAudits>({ runs: [], violations_by_audit: {} });
  let loaded = $state(false);
  let pollHandle: ReturnType<typeof setInterval> | null = null;

  async function callInvoke<T>(cmd: string, args?: Record<string, unknown>): Promise<T | undefined> {
    if (typeof window === 'undefined') return undefined;
    const tauriAvail = !!(window as unknown as { __TAURI_INTERNALS__?: unknown }).__TAURI_INTERNALS__;
    if (!tauriAvail) return undefined;
    try {
      const mod = await import('@tauri-apps/api/core');
      return (await mod.invoke<T>(cmd, args)) as T;
    } catch (e) {
      console.warn(`[locked-keys] invoke('${cmd}') failed`, e);
      return undefined;
    }
  }

  async function reload() {
    const s = await callInvoke<LockedKeysStatus>('get_locked_keys_status');
    const r = await callInvoke<RecentAudits>('get_locked_keys_recent_audits', { limit: 50 });
    if (s !== undefined) {
      status = s;
    }
    if (r !== undefined) {
      recent = r;
    }
    loaded = true;
  }

  function shortHash(h: string | null | undefined): string {
    if (!h) return '—';
    if (h.length <= 16) return h;
    return `${h.slice(0, 8)}…${h.slice(-4)}`;
  }

  function fmtTs(ms: number | null | undefined): string {
    if (!ms || ms <= 0) return '—';
    try {
      return new Date(ms).toISOString().slice(0, 19).replace('T', ' ');
    } catch {
      return '—';
    }
  }

  function fmtLineRange(range: number[] | undefined): string {
    if (!range || range.length < 2) return '—';
    if (range[0] === 0 && range[1] >= 4_294_967_295) return 'whole file';
    return `${range[0]}–${range[1]}`;
  }

  function violationsFor(auditId: string): ViolationRow[] {
    return recent.violations_by_audit[auditId] ?? [];
  }

  function runBadgeClass(n: number): string {
    return n > 0 ? 'badge badge-fail' : 'badge badge-pass';
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

<div class="panel">
  <h2>Locked Keys</h2>

  {#if !loaded}
    <p class="muted">Loading…</p>
  {:else if status === null}
    <p class="muted">No registry data available (Tauri not attached).</p>
  {:else if status.error}
    <p class="error">Registry load error: {status.error}</p>
    <p class="muted">Path: <code>{status.registry_path}</code></p>
  {:else}
    <section class="registry">
      <header>
        <strong>Registry</strong>
        <span class="muted">{status.n_entries} entries</span>
        <span class="muted">blake3={shortHash(status.registry_blake3)}</span>
        <span class="muted">loaded={fmtTs(status.loaded_ts_ms)}</span>
      </header>
      <p class="muted path">{status.registry_path}</p>
      {#if status.entries.length === 0}
        <p class="muted">No locked surfaces declared (bootstrap-seed state).</p>
      {:else}
        <table>
          <thead>
            <tr><th>Path</th><th>Range</th><th>Reason</th></tr>
          </thead>
          <tbody>
            {#each status.entries as entry}
              <tr>
                <td><code>{entry.path}</code></td>
                <td>{fmtLineRange(entry.line_range)}</td>
                <td>{entry.reason}</td>
              </tr>
            {/each}
          </tbody>
        </table>
      {/if}
    </section>

    <section class="audits">
      <header>
        <strong>Recent audits</strong>
        <span class="muted">{recent.runs.length} runs</span>
      </header>
      {#if recent.runs.length === 0}
        <p class="muted">No audit runs recorded yet.</p>
      {:else}
        <table>
          <thead>
            <tr><th>When</th><th>Audited</th><th>Violations</th><th>Audit ID</th></tr>
          </thead>
          <tbody>
            {#each recent.runs as run}
              <tr>
                <td>{fmtTs(run.ts_ms)}</td>
                <td>{run.n_audited}</td>
                <td><span class={runBadgeClass(run.n_violations)}>{run.n_violations}</span></td>
                <td><code>{shortHash(run.audit_id)}</code></td>
              </tr>
              {#each violationsFor(run.audit_id) as v}
                <tr class="violation-row">
                  <td></td>
                  <td colspan="3">
                    <span class="violation">VIOLATION</span>
                    <code>{v.attempted_edit}</code>
                    <span class="muted">[{v.attempted_lines.join(',')}]</span>
                    <span class="muted">— {v.reason}</span>
                  </td>
                </tr>
              {/each}
            {/each}
          </tbody>
        </table>
      {/if}
    </section>
  {/if}
</div>

<style>
  .panel {
    padding: 1rem;
    font-family: ui-monospace, SFMono-Regular, Menlo, Consolas, monospace;
    font-size: 0.85rem;
  }

  h2 {
    margin: 0 0 0.75rem 0;
    font-size: 1rem;
    text-transform: uppercase;
    letter-spacing: 0.08em;
  }

  section {
    margin-top: 1rem;
  }

  section > header {
    display: flex;
    gap: 0.75rem;
    align-items: baseline;
    margin-bottom: 0.5rem;
  }

  .muted {
    color: var(--muted, #888);
  }

  .error {
    color: var(--err, #c33);
  }

  .path {
    word-break: break-all;
  }

  table {
    border-collapse: collapse;
    width: 100%;
  }

  th, td {
    padding: 0.35rem 0.5rem;
    text-align: left;
    border-bottom: 1px solid var(--border, #2a2a2a);
    vertical-align: top;
  }

  .badge {
    display: inline-block;
    padding: 0.1rem 0.45rem;
    border-radius: 3px;
    background: var(--badge-bg, #333);
    color: var(--badge-fg, #fff);
    font-size: 0.75rem;
  }

  .badge-pass {
    background: #1f6d3a;
  }

  .badge-fail {
    background: #8a1f1f;
  }

  .violation-row {
    background: rgba(138, 31, 31, 0.06);
  }

  .violation {
    color: #c33;
    font-weight: bold;
    margin-right: 0.5rem;
  }

  code {
    font-family: inherit;
    font-size: inherit;
  }
</style>
