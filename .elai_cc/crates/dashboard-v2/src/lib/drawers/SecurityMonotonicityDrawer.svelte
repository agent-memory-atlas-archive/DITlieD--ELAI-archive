<script lang="ts">
  // F-10 / Slice A5 -- SecurityMonotonicityDrawer.
  //
  // Read-only view of recent SecurityClassRegressionBlocked SSB rows via the
  // `get_security_monotonicity_blocks` Tauri command
  // (see crates/dashboard-v2/src-tauri/src/commands_f10.rs).
  //
  // Renders:
  //   * summary header: total blocked count + per-loop-kind breakdown
  //   * last 50 SecurityClassRegressionBlocked rows, newest first
  //   * per-row: timestamp, loop-kind badge, introduced CWE classes,
  //     prior classes, language, delta_blake3
  //   * "guard inactive" hint when ELAI_SECURITY_MONOTONICITY_DISABLE is set
  //
  // Default-ON (W-2): rows emit with no env override; the guard is always
  // active unless ELAI_SECURITY_MONOTONICITY_DISABLE kills it. The three
  // production legs (REFINE / AUTO-ADVANCE / SELF-MOD) share this one row type
  // discriminated by loop_kind.
  // Polls on a 5s cadence (mirrors DepsSecurityPanel sibling pattern).
  // cross-lang-pair: dashboard/tauri/api

  import { onMount, onDestroy } from 'svelte';
  import type {
    SecurityMonotonicityResponse,
    SecurityMonotonicityRow,
  } from '../api/types';

  let resp = $state<SecurityMonotonicityResponse | null>(null);
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
      console.warn(`[security-monotonicity] invoke('${cmd}') failed`, e);
      return undefined;
    }
  }

  async function reload() {
    const out = await callInvoke<SecurityMonotonicityResponse>(
      'get_security_monotonicity_blocks'
    );
    if (out === undefined) {
      resp = null;
      loadError = null;
    } else {
      resp = out;
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

  function loopKindLabel(kind: string): string {
    switch (kind) {
      case 'refine':
        return 'REFINE';
      case 'auto_advance':
        return 'AUTO-ADVANCE';
      case 'self_mod':
        return 'SELF-MOD';
      default:
        return kind.toUpperCase();
    }
  }

  // Total + per-loop-kind breakdown derived from the rows (the command returns
  // only the row list + env flag; the panel aggregates client-side, mirroring
  // the DepsSecurityPanel counts_by_verdict shape).
  let totalBlocked = $derived(resp ? resp.rows.length : 0);

  let countsByLoopKind = $derived.by(() => {
    const map = new Map<string, number>();
    if (resp) {
      for (const row of resp.rows) {
        map.set(row.loop_kind, (map.get(row.loop_kind) ?? 0) + 1);
      }
    }
    return [...map.entries()]
      .map(([loop_kind, count]) => ({ loop_kind, count }))
      .sort((a, b) => b.count - a.count);
  });

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

<section class="security-monotonicity-panel">
  <header class="panel-header">
    <h2>Security Monotonicity (F-10)</h2>
    <button class="reload-btn" onclick={reload}>Reload</button>
  </header>

  {#if !loaded}
    <p class="muted">Loading security-monotonicity rows…</p>
  {:else if loadError}
    <div class="error-banner">
      <strong>Failed to load security-monotonicity rows.</strong>
      <pre>{loadError}</pre>
    </div>
  {:else if resp === null}
    <p class="muted">
      Tauri IPC unavailable (running in browser preview mode). Mount the
      dashboard in the Tauri host to see live security-monotonicity data.
    </p>
  {:else if !resp.env_active}
    <p class="muted">
      Security-monotonicity guard inactive. The kill-switch
      (<code>ELAI_SECURITY_MONOTONICITY_DISABLE=1</code>) is set. No
      <code>SecurityClassRegressionBlocked</code> rows are written until the
      guard is re-enabled.
    </p>
  {:else}
    <div class="summary-row">
      <span class="summary-label">Total blocked:</span>
      <span class="summary-value">{totalBlocked}</span>
      {#each countsByLoopKind as ck (ck.loop_kind)}
        <span class="badge badge-loop">{loopKindLabel(ck.loop_kind)} &times;{ck.count}</span>
      {/each}
    </div>

    {#if resp.rows.length === 0}
      <p class="muted">
        No <code>SecurityClassRegressionBlocked</code> rows recorded yet. Rows
        are emitted by the security-monotonicity iteration guard when an
        iteration introduces a CWE class absent from the prior iteration
        (REFINE call-site, AUTO-ADVANCE evidence-check, or SELF-MOD
        precondition).
      </p>
    {:else}
      <table>
        <thead>
          <tr>
            <th>Time</th>
            <th>Loop</th>
            <th>Introduced</th>
            <th>Prior</th>
            <th>Lang</th>
          </tr>
        </thead>
        <tbody>
          {#each resp.rows as row (row.seq)}
            <tr class="row-block">
              <td>{fmtTs(row.ts)}</td>
              <td><span class="badge badge-loop">{loopKindLabel(row.loop_kind)}</span></td>
              <td class="cwe-cell">
                {#if row.introduced_classes.length === 0}
                  <span class="muted">—</span>
                {:else}
                  {#each row.introduced_classes as cwe (cwe)}
                    <span class="badge badge-cwe">{cwe}</span>
                  {/each}
                {/if}
              </td>
              <td class="cwe-cell">
                {#if row.prior_classes.length === 0}
                  <span class="muted">—</span>
                {:else}
                  {#each row.prior_classes as cwe (cwe)}
                    <span class="badge badge-prior">{cwe}</span>
                  {/each}
                {/if}
              </td>
              <td><span class="badge badge-lang">{row.lang.toUpperCase()}</span></td>
            </tr>
          {/each}
        </tbody>
      </table>
    {/if}
  {/if}
</section>

<style>
  .security-monotonicity-panel {
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
    vertical-align: top;
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

  .cwe-cell {
    display: flex;
    flex-wrap: wrap;
    gap: 3px;
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

  .badge-loop {
    background: rgba(150, 110, 220, 0.15);
    border-color: rgba(150, 110, 220, 0.4);
  }

  .badge-cwe {
    background: rgba(220, 80, 80, 0.15);
    border-color: rgba(220, 80, 80, 0.4);
  }

  .badge-prior {
    background: rgba(120, 130, 145, 0.15);
    border-color: rgba(120, 130, 145, 0.4);
  }

  .badge-lang {
    background: rgba(80, 200, 150, 0.12);
    border-color: rgba(80, 200, 150, 0.35);
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
