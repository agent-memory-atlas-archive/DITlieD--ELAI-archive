<script lang="ts">
  // G-9 / Slice A -- RedactionPanel.
  //
  // Read-only view of recent secret_redacted SSB advisory rows via the
  // `get_redaction_stats` Tauri command
  // (see crates/dashboard-v2/src-tauri/src/commands_g9.rs).
  //
  // Each row is ONE write-time secret+PII redaction that fired on a typed
  // SsbPayload free-text field at the top of Ssb::write_payload (BEFORE msgpack
  // serialization; the SSB append-only invariant forces write-time). The row
  // records the redacted field identity, the stable blake3-keyed token(s) that
  // replaced the match(es), the match count, and the per-kind (label, count)
  // summary -- NEVER the raw secret. The panel renders the counts so an operator
  // can see redaction is firing on the write path without ever seeing a value.
  //
  // The producer is DEFAULT-ON (redaction is always on; there is no enable flag).
  // On the default install with no redaction fired the feed is empty. Polls on a
  // 5s cadence (mirrors the G-2/G-3/G-8 siblings).
  // cross-lang-pair: dashboard/tauri/api

  import { onMount, onDestroy } from 'svelte';
  import type { RedactionStat } from '../api/types';

  let rows = $state<RedactionStat[] | null>(null);
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
      console.warn(`[redaction] invoke('${cmd}') failed`, e);
      return undefined;
    }
  }

  async function reload() {
    const out = await callInvoke<RedactionStat[]>('get_redaction_stats');
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

  function fmtKinds(kinds: [string, number][]): string {
    return kinds.map(([label, count]) => `${label}×${count}`).join(', ');
  }

  let total = $derived(rows ? rows.length : 0);
  let totalMatches = $derived(
    rows ? rows.reduce((acc, r) => acc + r.match_count, 0) : 0
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

<section class="redaction-panel">
  <header class="panel-header">
    <h2>Secret Redactions (G-9)</h2>
    <button class="reload-btn" onclick={reload}>Reload</button>
  </header>

  {#if !loaded}
    <p class="muted">Loading secret-redaction advisories…</p>
  {:else if loadError}
    <div class="error-banner">
      <strong>Failed to load redaction advisories.</strong>
      <pre>{loadError}</pre>
    </div>
  {:else if rows === null}
    <p class="muted">
      Tauri IPC unavailable (running in browser preview mode). Mount the
      dashboard in the Tauri host to see live redaction data.
    </p>
  {:else}
    <div class="summary-row">
      <span class="summary-label">Redaction events:</span>
      <span class="summary-value">{total}</span>
      <span class="summary-label">Total matches:</span>
      <span class="summary-value">{totalMatches}</span>
    </div>

    {#if rows.length === 0}
      <p class="muted">
        No <code>secret_redacted</code> rows recorded. A row emits whenever the
        write-time secret+PII redactor fires on a typed payload free-text field
        at the top of <code>write_payload</code> (BEFORE serialization). The row
        carries the redacted field + stable blake3-keyed token(s) + match count,
        NEVER the raw secret. Redaction is always on (no enable flag).
      </p>
    {:else}
      <table>
        <thead>
          <tr>
            <th>Time</th>
            <th>Session</th>
            <th>Field</th>
            <th>Matches</th>
            <th>Kinds</th>
            <th>Tokens</th>
          </tr>
        </thead>
        <tbody>
          {#each rows as row (row.seq)}
            <tr class="row-redaction">
              <td>{fmtTs(row.ts_ms)}</td>
              <td class="session-cell" title={row.session_id}>{row.session_id}</td>
              <td class="field-cell" title={row.field}><code>{row.field}</code></td>
              <td class="num-cell">{row.match_count}</td>
              <td class="kinds-cell" title={fmtKinds(row.kind_counts)}>
                {fmtKinds(row.kind_counts)}
              </td>
              <td class="tokens-cell" title={row.tokens.join(' ')}>
                <code>{row.tokens.join(' ')}</code>
              </td>
            </tr>
          {/each}
        </tbody>
      </table>
      <p class="footnote">
        Tokens are blake3-keyed (<code>[REDACTED:&lt;kind&gt;:&lt;blake3-16&gt;]</code>):
        the same secret maps to the same token (correlatable) and the token is
        never reversible to the secret. The redactor runs LOCALLY (no cloud
        egress) and is fail-closed.
      </p>
    {/if}
  {/if}
</section>

<style>
  .redaction-panel {
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
    color: rgb(220, 180, 80);
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

  .field-cell,
  .tokens-cell,
  .kinds-cell {
    max-width: 200px;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .field-cell code,
  .tokens-cell code {
    font-size: 11px;
  }

  .session-cell {
    max-width: 160px;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .num-cell {
    text-align: right;
    font-variant-numeric: tabular-nums;
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
