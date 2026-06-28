<script lang="ts">
  // G-3 / Slice A -- BlastRadiusDrawer.
  //
  // Read-only view of recent blast_radius_blocked SSB rows via the
  // `get_blast_radius_blocks` Tauri command
  // (see crates/dashboard-v2/src-tauri/src/commands_g3.rs).
  //
  // Each row is a deterministic runtime gate BLOCK. Two kinds:
  //  - fs_escape: a shell-EXPANDED filesystem-mutating target resolved OUTSIDE
  //    the per-run worktree root (the rm -rf ~/x / glob / symlink escape),
  //    emitted from translate_tool_request / run_bash.
  //  - egress_denied: an outbound host carried a NUL/control char or resolved to
  //    an IP outside the pinned allow-set, emitted from fetch_and_extract.
  //
  // The drawer renders the pre- vs post-expansion command diff (the #10077
  // forensics fix: the EXPANDED command -- the one the shell would actually have
  // run -- was never logged before this gate). Both producers are DEFAULT-ON;
  // the only opt-out is the kill-switch ELAI_BLAST_RADIUS_DISABLE. On the
  // default install with no block the feed is empty. Polls on a 5s cadence
  // (mirrors the G-4 sibling). cross-lang-pair: dashboard/tauri/api

  import { onMount, onDestroy } from 'svelte';
  import type { BlastRadiusBlock } from '../api/types';

  let rows = $state<BlastRadiusBlock[] | null>(null);
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
      console.warn(`[blast-radius] invoke('${cmd}') failed`, e);
      return undefined;
    }
  }

  async function reload() {
    const out = await callInvoke<BlastRadiusBlock[]>('get_blast_radius_blocks');
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

  function kindLabel(kind: string): string {
    if (kind === 'fs_escape') return 'FS ESCAPE';
    if (kind === 'egress_denied') return 'EGRESS DENIED';
    return kind.toUpperCase();
  }

  let total = $derived(rows ? rows.length : 0);
  let fsEscapes = $derived(rows ? rows.filter((r) => r.kind === 'fs_escape').length : 0);
  let egressDenials = $derived(
    rows ? rows.filter((r) => r.kind === 'egress_denied').length : 0
  );

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

<section class="blast-radius-panel">
  <header class="panel-header">
    <h2>Blast Radius (G-3)</h2>
    <button class="reload-btn" onclick={reload}>Reload</button>
  </header>

  {#if !loaded}
    <p class="muted">Loading blast-radius events…</p>
  {:else if loadError}
    <div class="error-banner">
      <strong>Failed to load blast-radius events.</strong>
      <pre>{loadError}</pre>
    </div>
  {:else if rows === null}
    <p class="muted">
      Tauri IPC unavailable (running in browser preview mode). Mount the
      dashboard in the Tauri host to see live blast-radius data.
    </p>
  {:else}
    <div class="summary-row">
      <span class="summary-label">Blocks:</span>
      <span class="summary-value">{total}</span>
      <span class="summary-label">FS escapes:</span>
      <span class="summary-value">{fsEscapes}</span>
      <span class="summary-label">Egress denials:</span>
      <span class="summary-value">{egressDenials}</span>
    </div>

    {#if rows.length === 0}
      <p class="muted">
        No <code>blast_radius_blocked</code> rows recorded. Rows emit when a
        deterministic runtime gate refuses an operation fail-closed: a
        shell-EXPANDED filesystem target that resolves OUTSIDE the worktree root
        (<code>fs_escape</code>), or an outbound host with a NUL/control char or
        an IP outside the pinned allow-set (<code>egress_denied</code>). Both
        gates are default-ON (kill-switch <code>ELAI_BLAST_RADIUS_DISABLE</code>
        only).
      </p>
    {:else}
      <table>
        <thead>
          <tr>
            <th>Time</th>
            <th>Kind</th>
            <th>Pre-expansion (issued)</th>
            <th>Post-expansion (resolved)</th>
            <th>Target / Host</th>
          </tr>
        </thead>
        <tbody>
          {#each rows as row (row.seq)}
            <tr class="row-block" class:row-fs={row.kind === 'fs_escape'} class:row-egress={row.kind === 'egress_denied'}>
              <td>{fmtTs(row.ts_ms)}</td>
              <td><span class="badge badge-kind">{kindLabel(row.kind)}</span></td>
              <td class="cmd-cell pre" title={row.pre_expansion_command}>
                <code>{row.pre_expansion_command}</code>
              </td>
              <td class="cmd-cell post" title={row.post_expansion_command}>
                <code>{row.post_expansion_command}</code>
              </td>
              <td class="target-cell" title={row.target_or_host}>{row.target_or_host}</td>
            </tr>
          {/each}
        </tbody>
      </table>
      <p class="footnote">
        The verb never ran / the connect never happened. The pre- vs
        post-expansion pair is the <code>#10077</code> forensics fix: the
        EXPANDED command (the one the shell would actually have executed) was
        never logged before this gate.
      </p>
    {/if}
  {/if}
</section>

<style>
  .blast-radius-panel {
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

  .row-fs {
    background: rgba(220, 120, 80, 0.07);
  }

  .row-egress {
    background: rgba(120, 140, 220, 0.07);
  }

  .cmd-cell {
    max-width: 240px;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .cmd-cell code {
    font-size: 10px;
  }

  .cmd-cell.pre code {
    color: var(--text-secondary);
  }

  .cmd-cell.post code {
    color: rgb(220, 120, 80);
  }

  .target-cell {
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

  .badge-kind {
    background: rgba(220, 120, 80, 0.15);
    border-color: rgba(220, 120, 80, 0.4);
  }

  .muted {
    color: var(--text-secondary);
  }

  .footnote {
    color: var(--text-secondary);
    font-size: 10px;
  }

  .error-banner {
    background: rgba(220, 80, 80, 0.1);
    border: 1px solid rgba(220, 80, 80, 0.4);
    padding: 8px;
    border-radius: var(--r-sm);
  }
</style>
