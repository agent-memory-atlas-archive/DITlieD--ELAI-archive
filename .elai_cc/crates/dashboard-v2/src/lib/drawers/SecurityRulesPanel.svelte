<script lang="ts">
  // F-2a / A5 -- SecurityRulesPanel.
  //
  // Read-only view of recent SecurityRuleBlocked SSB rows via the
  // `get_security_blocks_recent` Tauri command
  // (see crates/dashboard-v2/src-tauri/src/commands_f2.rs).
  //
  // Renders:
  //   * the F-2b static-gate insecure-rate keystone tile (before -> after +
  //     gate_recall), via get_insecure_rate_latest
  //   * last 50 security_rule_blocked flat-beacon rows, newest first
  //   * per-row rule_id + cwe + file_path + language + candidate_blake3
  //   * "env gate inactive" hint when ELAI_SECURITY_RULE_BLOCKED_ENABLE is unset
  //
  // Wire-dark: when the env gate is off, no rows are written and this
  // panel renders the inactive-gate hint. The Tauri command still
  // returns cleanly so the panel mounts without errors.
  //
  // The insecure-rate tile shows the REAL ELAI static write-gate
  // (evaluate_vibegate) measured over a SeCodePLT-structured CWE corpus.
  // It is the STATIC-gate insecure-rate, DISTINCT from the SeCodePLT
  // dynamic secure-coding rate (sandbox-executed); that dynamic number is a
  // separate RunPod-gated measurement and is NOT shown here.
  //
  // Polls on a 5s cadence (mirrors IncidentsDrawer's sibling pattern).
  // cross-lang-pair: dashboard/tauri/api

  import { onMount, onDestroy } from 'svelte';
  import type { SecurityRuleRow, InsecureRateRow } from '../api/types';

  interface SecurityBlocksResponse {
    rows: SecurityRuleRow[];
    env_active: boolean;
  }

  let rows = $state<SecurityRuleRow[]>([]);
  let envActive = $state(false);
  let insecureRate = $state<InsecureRateRow | null>(null);
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
      console.warn(`[security-rules] invoke('${cmd}') failed`, e);
      return undefined;
    }
  }

  async function reload() {
    const out = await callInvoke<SecurityBlocksResponse>('get_security_blocks_recent', {
      limit: 50,
    });
    if (out === undefined) {
      rows = [];
      envActive = false;
      loadError = null;
    } else {
      rows = out.rows;
      envActive = out.env_active;
      loadError = null;
    }

    const rate = await callInvoke<InsecureRateRow | null>('get_insecure_rate_latest');
    insecureRate = rate ?? null;

    loaded = true;
  }

  function fmtPct(v: number | null | undefined): string {
    if (v === null || v === undefined || Number.isNaN(v)) return '—';
    return `${(v * 100).toFixed(1)}%`;
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

<section class="security-rules-panel">
  <header class="panel-header">
    <h2>Security Rules (F-2a)</h2>
    <button class="reload-btn" onclick={reload}>Reload</button>
  </header>

  {#if insecureRate}
    <div class="insecure-tile">
      <div class="tile-title">
        Static-gate insecure-rate
        <span class="tile-sub"
          >evaluate_vibegate over a SeCodePLT-structured CWE corpus (NOT the
          SeCodePLT dynamic secure-coding rate)</span
        >
      </div>
      <div class="ba-row">
        <div class="ba-cell">
          <div class="ba-label">before</div>
          <div class="ba-value before">{fmtPct(insecureRate.insecure_rate_before)}</div>
        </div>
        <div class="ba-arrow">→</div>
        <div class="ba-cell">
          <div class="ba-label">after (gate-approved)</div>
          <div class="ba-value after">{fmtPct(insecureRate.insecure_rate_after)}</div>
        </div>
        <div class="ba-cell recall">
          <div class="ba-label">gate recall</div>
          <div class="ba-value">{fmtPct(insecureRate.gate_recall)}</div>
        </div>
      </div>
      <div class="tile-meta">
        suite {insecureRate.suite} · model {insecureRate.model_id} · n={insecureRate.n_samples}
        · {fmtTs(insecureRate.ts)}
      </div>
    </div>
  {/if}

  {#if !loaded}
    <p class="muted">Loading security rule blocks…</p>
  {:else if loadError}
    <div class="error-banner">
      <strong>Failed to load security rule blocks.</strong>
      <pre>{loadError}</pre>
    </div>
  {:else if !envActive}
    <p class="muted">
      SecurityRuleBlocked env gate inactive. Set
      <code>ELAI_SECURITY_RULE_BLOCKED_ENABLE=1</code> to enable security-rule
      enforcement telemetry. No rows are written until the gate is active.
    </p>
  {:else if rows.length === 0}
    <p class="muted">
      No security-rule blocks recorded yet. Blocks are emitted by the
      orchestrator rules engine when a candidate edit violates a security
      rule predicate.
    </p>
  {:else}
    <table>
      <thead>
        <tr>
          <th>Time</th>
          <th>Rule / CWE</th>
          <th>File</th>
          <th>Language</th>
          <th>Candidate Hash</th>
        </tr>
      </thead>
      <tbody>
        {#each rows as row (row.seq)}
          <tr class="row-block">
            <td>{fmtTs(row.ts)}</td>
            <td>
              <span class="rule-chip">
                {row.rule_id}{row.cwe ? ` / ${row.cwe}` : ''}
              </span>
            </td>
            <td class="path" title={row.file_path}>{row.file_path}</td>
            <td>{row.language}</td>
            <td>
              <code title={row.candidate_blake3}>{shortHash(row.candidate_blake3)}</code>
            </td>
          </tr>
        {/each}
      </tbody>
    </table>
  {/if}
</section>

<style>
  .security-rules-panel {
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

  .path {
    max-width: 200px;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    color: var(--text-secondary);
  }

  .rule-chip {
    font-size: 10px;
    color: var(--text-secondary);
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

  .insecure-tile {
    border: 1px solid var(--border-default);
    border-radius: var(--r-sm);
    padding: 10px 12px;
    background: var(--bg-surface);
    display: flex;
    flex-direction: column;
    gap: 8px;
  }

  .tile-title {
    display: flex;
    flex-direction: column;
    gap: 2px;
    font-weight: 500;
    letter-spacing: 0.04em;
    text-transform: uppercase;
    font-size: 10px;
    color: var(--text-primary);
  }

  .tile-sub {
    font-weight: 400;
    text-transform: none;
    letter-spacing: 0;
    font-size: 10px;
    color: var(--text-secondary);
  }

  .ba-row {
    display: flex;
    align-items: center;
    gap: 16px;
  }

  .ba-cell {
    display: flex;
    flex-direction: column;
    gap: 2px;
  }

  .ba-cell.recall {
    margin-left: auto;
  }

  .ba-label {
    font-size: 10px;
    color: var(--text-secondary);
    text-transform: uppercase;
    letter-spacing: 0.04em;
  }

  .ba-value {
    font-size: 18px;
    font-weight: 600;
    font-variant-numeric: tabular-nums;
    color: var(--text-primary);
  }

  .ba-value.before {
    color: rgba(220, 120, 80, 0.95);
  }

  .ba-value.after {
    color: rgba(90, 190, 120, 0.95);
  }

  .ba-arrow {
    font-size: 15px;
    color: var(--text-secondary);
  }

  .tile-meta {
    font-size: 10px;
    color: var(--text-secondary);
  }
</style>
