<script lang="ts">
  // F-13 / Slice A5 -- SessionArchiveDrawer.
  //
  // Read-only view of recent SessionArchived SSB rows via the
  // `get_session_archives` Tauri command
  // (see crates/dashboard-v2/src-tauri/src/commands_f13.rs).
  //
  // Each row is a stale-version store that was cold-archived to
  // workflow/sessions/{id}/archive/ssb-v{N}.sqlite BEFORE the destructive
  // run_migration DROP, when the wire-dark ELAI_SSB_ARCHIVE_ON_BUMP_ENABLE gate
  // was set. The drawer surfaces the archived session id, the stale schema
  // version, the archive path, and the blake3 manifest prefix so an operator
  // can locate + integrity-check the cold archive.
  //
  // The producer gate is DEFAULT-OFF (forensic opt-in); on the default install
  // the feed is empty. Polls on a 5s cadence (mirrors the F-7 sibling).
  // cross-lang-pair: dashboard/tauri/api

  import { onMount, onDestroy } from 'svelte';
  import type { SessionArchive } from '../api/types';

  let rows = $state<SessionArchive[] | null>(null);
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
      console.warn(`[session-archive] invoke('${cmd}') failed`, e);
      return undefined;
    }
  }

  async function reload() {
    const out = await callInvoke<SessionArchive[]>('get_session_archives');
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

<section class="session-archive-panel">
  <header class="panel-header">
    <h2>Session Archives (F-13)</h2>
    <button class="reload-btn" onclick={reload}>Reload</button>
  </header>

  {#if !loaded}
    <p class="muted">Loading session archives…</p>
  {:else if loadError}
    <div class="error-banner">
      <strong>Failed to load session archives.</strong>
      <pre>{loadError}</pre>
    </div>
  {:else if rows === null}
    <p class="muted">
      Tauri IPC unavailable (running in browser preview mode). Mount the
      dashboard in the Tauri host to see live session-archive data.
    </p>
  {:else}
    <div class="summary-row">
      <span class="summary-label">Archived:</span>
      <span class="summary-value">{total}</span>
    </div>

    {#if rows.length === 0}
      <p class="muted">
        No <code>session_archived</code> rows recorded yet. Rows are emitted at
        the <code>run_migration</code> destructive-DROP site when a stale-version
        <code>ssb.db</code> is opened with
        <code>ELAI_SSB_ARCHIVE_ON_BUMP_ENABLE</code> set: the stale journal is
        cold-archived to
        <code>workflow/sessions/&lbrace;id&rbrace;/archive/ssb-v&lbrace;N&rbrace;.sqlite</code>
        before the DROP. The gate is default-OFF (forensic opt-in).
      </p>
    {:else}
      <table>
        <thead>
          <tr>
            <th>Time</th>
            <th>Session</th>
            <th class="num">Stale v</th>
            <th>Store</th>
            <th>Archive path</th>
            <th>blake3</th>
          </tr>
        </thead>
        <tbody>
          {#each rows as row (row.seq)}
            <tr class="row-archive">
              <td>{fmtTs(row.ts_ms)}</td>
              <td class="session-cell" title={row.session_id}>{row.session_id}</td>
              <td class="num">{row.stale_version}</td>
              <td>
                <span class="badge badge-store">{row.store_kind}</span>
              </td>
              <td class="path-cell" title={row.archive_path}>{row.archive_path}</td>
              <td class="hash-cell" title={row.manifest_blake3}>{row.manifest_blake3}</td>
            </tr>
          {/each}
        </tbody>
      </table>
    {/if}
  {/if}
</section>

<style>
  .session-archive-panel {
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

  .row-archive {
    background: rgba(110, 150, 220, 0.05);
  }

  .session-cell,
  .path-cell,
  .hash-cell {
    max-width: 200px;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .badge {
    display: inline-block;
    padding: 1px 6px;
    border-radius: var(--r-xs);
    font-size: 10px;
    letter-spacing: 0.04em;
    background: var(--bg-hover);
    border: 1px solid var(--border-subtle);
  }

  .badge-store {
    background: rgba(110, 150, 220, 0.15);
    border-color: rgba(110, 150, 220, 0.4);
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
