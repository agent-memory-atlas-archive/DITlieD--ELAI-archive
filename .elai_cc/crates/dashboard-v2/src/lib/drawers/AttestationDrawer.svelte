<script lang="ts">
  // G-8 / Slice A -- AttestationDrawer.
  //
  // Read-only view of recent provenance_attestation_emitted SSB rows via the
  // `get_attestations` Tauri command
  // (see crates/dashboard-v2/src-tauri/src/commands_g8.rs).
  //
  // Each row is ONE signed in-toto v1 provenance attestation built at the
  // `elai run` exit from the sealed G-2 Reproducible-Run Artifact: the F-gate
  // verified_levels rollup re-expressed as a custom in-toto predicate and signed
  // with a LOCAL blake3-Merkle proof so it re-verifies OFFLINE without re-running
  // ELAI (trust-without-re-running, SLSA-VSA). The drawer renders the
  // verifiedLevels rollup -- the subject digest (the value the git trailer
  // carries + `elai verify-attestation` recomputes), the G-2 run-artifact root
  // it chains to, the F-gate verdict count, the all-passed flag, and the honest
  // signing model -- so an operator can read the run's off-machine trust posture.
  //
  // The producer is DEFAULT-ON; the only opt-out is the kill-switch
  // ELAI_ATTESTATION_DISABLE. On the default install with no run the feed is
  // empty. Polls on a 5s cadence (mirrors the G-2/G-3 siblings).
  // cross-lang-pair: dashboard/tauri/api

  import { onMount, onDestroy } from 'svelte';
  import type { Attestation } from '../api/types';

  let rows = $state<Attestation[] | null>(null);
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
      console.warn(`[attestation] invoke('${cmd}') failed`, e);
      return undefined;
    }
  }

  async function reload() {
    const out = await callInvoke<Attestation[]>('get_attestations');
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
  let allPassed = $derived(rows ? rows.filter((r) => r.all_passed).length : 0);

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

<section class="attestation-panel">
  <header class="panel-header">
    <h2>Provenance Attestations (G-8)</h2>
    <button class="reload-btn" onclick={reload}>Reload</button>
  </header>

  {#if !loaded}
    <p class="muted">Loading provenance attestations…</p>
  {:else if loadError}
    <div class="error-banner">
      <strong>Failed to load provenance attestations.</strong>
      <pre>{loadError}</pre>
    </div>
  {:else if rows === null}
    <p class="muted">
      Tauri IPC unavailable (running in browser preview mode). Mount the
      dashboard in the Tauri host to see live attestation data.
    </p>
  {:else}
    <div class="summary-row">
      <span class="summary-label">Attestations:</span>
      <span class="summary-value">{total}</span>
      <span class="summary-label">All-passed:</span>
      <span class="summary-value">{allPassed}</span>
    </div>

    {#if rows.length === 0}
      <p class="muted">
        No <code>provenance_attestation_emitted</code> rows recorded. A row emits
        at the <code>elai run</code> exit when a signed in-toto v1 provenance
        attestation is built from the sealed G-2 Reproducible-Run Artifact and
        signed with a LOCAL blake3-Merkle proof (re-verifiable OFFLINE via
        <code>elai verify-attestation</code>). Default-ON (kill-switch
        <code>ELAI_ATTESTATION_DISABLE</code> only).
      </p>
    {:else}
      <table>
        <thead>
          <tr>
            <th>Time</th>
            <th>Session</th>
            <th>Subject digest (blake3)</th>
            <th>Run-artifact root</th>
            <th>F-gates</th>
            <th>All passed</th>
            <th>Signing</th>
          </tr>
        </thead>
        <tbody>
          {#each rows as row (row.seq)}
            <tr class="row-attestation" class:row-passed={row.all_passed}>
              <td>{fmtTs(row.ts_ms)}</td>
              <td class="session-cell" title={row.session_id}>{row.session_id}</td>
              <td class="digest-cell" title={row.subject_blake3}>
                <code>{row.subject_blake3}</code>
              </td>
              <td class="digest-cell" title={row.run_artifact_root}>
                <code>{row.run_artifact_root}</code>
              </td>
              <td class="num-cell">{row.verified_levels}</td>
              <td>
                <span class="badge" class:badge-pass={row.all_passed} class:badge-warn={!row.all_passed}>
                  {row.all_passed ? 'PASS' : 'PARTIAL'}
                </span>
              </td>
              <td class="signing-cell" title={row.signing_model}>{row.signing_model}</td>
            </tr>
          {/each}
        </tbody>
      </table>
      <p class="footnote">
        Each attestation re-verifies OFFLINE: <code>elai verify-attestation
        &lt;commit&gt;</code> reads the digest-only <code>ELAI-Provenance-Digest</code>
        commit trailer + the sidecar <code>attestation.intoto.json</code>,
        recomputes the blake3-Merkle root, and confirms it matches -- without
        re-running ELAI. Designed to support EU AI Act Article-50 transparency
        marking (Article-99 Tier-2); not a certified-compliance claim.
      </p>
    {/if}
  {/if}
</section>

<style>
  .attestation-panel {
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
    color: rgb(120, 200, 140);
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

  .row-passed {
    background: rgba(120, 200, 140, 0.07);
  }

  .digest-cell {
    max-width: 180px;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .digest-cell code {
    font-size: 11px;
  }

  .session-cell,
  .signing-cell {
    max-width: 160px;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .num-cell {
    text-align: right;
    font-variant-numeric: tabular-nums;
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

  .badge-pass {
    background: rgba(120, 200, 140, 0.15);
    border-color: rgba(120, 200, 140, 0.4);
  }

  .badge-warn {
    background: rgba(220, 180, 80, 0.15);
    border-color: rgba(220, 180, 80, 0.4);
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
