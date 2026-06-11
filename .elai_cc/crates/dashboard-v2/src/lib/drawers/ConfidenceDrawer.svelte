<script lang="ts">
  // G-7 / Slice A -- ConfidenceDrawer.
  //
  // Read-only view of two G-7 SSB feeds:
  //  - claim_confidence_scored rows via the `get_claim_confidence` Tauri command
  //    (the REAL fused per-change confidence: log-prob LOCAL-only +
  //    self-consistency + exec-agreement, feeding the SHIPPED conformal decide()).
  //  - trifecta_quarantined rows via the `get_trifecta_quarantines` command (the
  //    lethal-trifecta dual-LLM quarantine beacon: secret-present + untrusted-span
  //    + egress co-occurred and the untrusted span was held back).
  //
  // The confidence rows feed the G-5 cockpit confidence-triage band (review
  // low-confidence hunks first). The producer is gated by the conformal triple
  // gate; the quarantine gate is default-ON (kill-switch
  // ELAI_TRIFECTA_QUARANTINE_DISABLE only). On a default install both feeds are
  // empty. Polls on a 5s cadence (mirrors the G-2/G-3/G-8/G-9 siblings).
  // cross-lang-pair: dashboard/tauri/api

  import { onMount, onDestroy } from 'svelte';
  import type { ClaimConfidence, TrifectaQuarantine } from '../api/types';

  let confidences = $state<ClaimConfidence[] | null>(null);
  let quarantines = $state<TrifectaQuarantine[] | null>(null);
  let loaded = $state(false);
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
      console.warn(`[confidence] invoke('${cmd}') failed`, e);
      return undefined;
    }
  }

  async function reload() {
    const conf = await callInvoke<ClaimConfidence[]>('get_claim_confidence');
    confidences = conf === undefined ? null : conf;
    const quar = await callInvoke<TrifectaQuarantine[]>('get_trifecta_quarantines');
    quarantines = quar === undefined ? null : quar;
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

  function fmtScore(n: number | null | undefined): string {
    if (n === null || n === undefined) return '—';
    return n.toFixed(3);
  }

  let totalConf = $derived(confidences ? confidences.length : 0);
  let abstainCount = $derived(
    confidences ? confidences.filter((r) => r.abstained).length : 0
  );
  let totalQuar = $derived(quarantines ? quarantines.length : 0);

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

<section class="confidence-panel">
  <header class="panel-header">
    <h2>Per-Change Confidence + Trifecta Quarantine (G-7)</h2>
    <button class="reload-btn" onclick={reload}>Reload</button>
  </header>

  {#if !loaded}
    <p class="muted">Loading confidence + quarantine feeds…</p>
  {:else if confidences === null && quarantines === null}
    <p class="muted">
      Tauri IPC unavailable (browser preview mode). Mount the dashboard in the
      Tauri host to see live data.
    </p>
  {:else}
    <div class="summary-row">
      <span class="summary-label">Decisions:</span>
      <span class="summary-value">{totalConf}</span>
      <span class="summary-label">Abstained:</span>
      <span class="summary-value">{abstainCount}</span>
      <span class="summary-label">Quarantines:</span>
      <span class="summary-value">{totalQuar}</span>
    </div>

    <h3>Per-change confidence</h3>
    {#if !confidences || confidences.length === 0}
      <p class="muted">
        No <code>claim_confidence_scored</code> rows. A row emits per live
        per-change conformal decision carrying the REAL fused claim_score
        (log-prob LOCAL-only + self-consistency + exec-agreement). The producer
        is behind the conformal triple gate.
      </p>
    {:else}
      <table>
        <thead>
          <tr>
            <th>Time</th>
            <th>Claim</th>
            <th>Score</th>
            <th>logp</th>
            <th>self-c</th>
            <th>exec</th>
            <th>Lane</th>
            <th>tau</th>
            <th>Verdict</th>
          </tr>
        </thead>
        <tbody>
          {#each confidences as row (row.seq)}
            <tr class:abstained={row.abstained}>
              <td>{fmtTs(row.ts_ms)}</td>
              <td class="ellipsis" title={row.claim_id}>{row.claim_id}</td>
              <td class="num-cell">{fmtScore(row.claim_score)}</td>
              <td class="num-cell">{fmtScore(row.logprob_component)}</td>
              <td class="num-cell">{fmtScore(row.self_consistency_component)}</td>
              <td class="num-cell">{fmtScore(row.exec_agreement_component)}</td>
              <td>{row.lane}</td>
              <td class="num-cell">{fmtScore(row.tau)}</td>
              <td class="verdict-cell">{row.conformal_verdict}</td>
            </tr>
          {/each}
        </tbody>
      </table>
    {/if}

    <h3>Lethal-trifecta quarantines</h3>
    {#if !quarantines || quarantines.length === 0}
      <p class="muted">
        No <code>trifecta_quarantined</code> rows. A row emits when a secret is
        present AND an untrusted span flows into an egress-capable tool; the
        untrusted span is routed through a no-tool sub-LLM and NEVER reaches the
        tool-caller. A benign run (any leg missing) never trips the trifecta.
      </p>
    {:else}
      <table>
        <thead>
          <tr>
            <th>Time</th>
            <th>Task</th>
            <th>Origin (blake3)</th>
            <th>Sink</th>
            <th>secret</th>
            <th>untrusted</th>
            <th>egress</th>
            <th>Verdict</th>
          </tr>
        </thead>
        <tbody>
          {#each quarantines as row (row.seq)}
            <tr class="quarantine-row">
              <td>{fmtTs(row.ts_ms)}</td>
              <td class="ellipsis" title={row.task_id}>{row.task_id}</td>
              <td class="ellipsis"><code>{row.untrusted_origin_blake3}</code></td>
              <td>{row.sink_tool}</td>
              <td class="bool-cell">{row.secret_present ? '✓' : '·'}</td>
              <td class="bool-cell">{row.untrusted_span ? '✓' : '·'}</td>
              <td class="bool-cell">{row.egress_available ? '✓' : '·'}</td>
              <td class="verdict-cell">{row.sub_llm_verdict}</td>
            </tr>
          {/each}
        </tbody>
      </table>
      <p class="footnote">
        The untrusted-origin is blake3-hashed (never the raw span). The raw
        secret and the raw untrusted text never leave the gate; only the no-tool
        sub-LLM's sanitized summary flows on.
      </p>
    {/if}
  {/if}
</section>

<style>
  .confidence-panel {
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

  h3 {
    margin: 4px 0 0 0;
    font-size: 11px;
    font-weight: 500;
    letter-spacing: 0.04em;
    text-transform: uppercase;
    color: var(--text-secondary);
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

  .ellipsis {
    max-width: 160px;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .num-cell {
    text-align: right;
    font-variant-numeric: tabular-nums;
  }

  .bool-cell {
    text-align: center;
  }

  tr.abstained .verdict-cell {
    color: rgb(220, 120, 80);
    font-weight: 600;
  }

  tr.quarantine-row .verdict-cell {
    color: rgb(220, 120, 80);
    font-weight: 600;
  }

  .muted {
    color: var(--text-secondary);
  }

  .footnote {
    color: var(--text-secondary);
    font-size: 11px;
  }
</style>
