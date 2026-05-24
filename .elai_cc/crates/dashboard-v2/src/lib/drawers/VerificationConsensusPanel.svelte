<script lang="ts">
  // D-3 — VerificationConsensusPanel.
  //
  // Read-only view of recent multi-verifier consensus rows via the
  // `get_recent_verifications` Tauri command (additive — see
  // crates/dashboard-v2/src-tauri/src/commands_d3.rs).
  //
  // Renders:
  //   * last 50 consensus rows (mix of `pass` + `fail`), newest first
  //   * per-row kind badge + rule id (k_of_n / all_pass / weighted / heterogeneous)
  //   * truncated candidate_id + template_id (12 hex chars)
  //   * verifier_count / pass_count for `pass` rows
  //   * failing_verifier_ids + reason for `fail` rows
  //
  // No producer logic lives here — the orchestrator producer
  // (`agent_pool::run_consensus_pass`) emits the SSB rows separately. This
  // panel polls on a 10s cadence.

  import { onMount, onDestroy } from 'svelte';

  // D-3 closure: widened to accept the 3 new SSB variants (ppm_unavailable,
  // verifier_disagreement, cross_context_dispatched). Existing 'pass' /
  // 'fail' branches remain intact; the new kinds render via the same
  // table with a per-kind reason summary. The Rust-side
  // `commands_d3::VerificationConsensusRow` is the canonical schema;
  // optional fields below are populated only for the relevant kinds.
  type VerificationConsensusRow = {
    seq: number;
    ts: number;
    kind: 'pass' | 'fail' | 'ppm_unavailable' | 'verifier_disagreement' | 'cross_context_dispatched';
    candidate_id: string;
    template_id: string;
    verdict: string;
    rule_id: string;
    verifier_count: number | null;
    pass_count: number | null;
    failing_verifier_ids: string[];
    reason: string | null;
  };

  let rows = $state<VerificationConsensusRow[]>([]);
  let loaded = $state(false);
  let loadError = $state<string | null>(null);
  let pollHandle: ReturnType<typeof setInterval> | null = null;

  async function callInvoke<T>(cmd: string, args?: Record<string, unknown>): Promise<T | undefined> {
    if (typeof window === 'undefined') return undefined;
    const tauriAvail = !!(window as unknown as { __TAURI_INTERNALS__?: unknown }).__TAURI_INTERNALS__;
    if (!tauriAvail) return undefined;
    try {
      const mod = await import('@tauri-apps/api/core');
      return (await mod.invoke<T>(cmd, args)) as T;
    } catch (e) {
      console.warn(`[verification-consensus] invoke('${cmd}') failed`, e);
      return undefined;
    }
  }

  async function reload() {
    const out = await callInvoke<VerificationConsensusRow[]>('get_recent_verifications', { limit: 50 });
    if (out === undefined) {
      rows = [];
      loadError = null;
    } else {
      rows = out;
      loadError = null;
    }
    loaded = true;
  }

  function shortId(s: string): string {
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
    pollHandle = setInterval(reload, 10_000);
  });

  onDestroy(() => {
    if (pollHandle) {
      clearInterval(pollHandle);
      pollHandle = null;
    }
  });
</script>

<section class="verification-consensus-panel">
  <header class="panel-header">
    <h2>Verifier Consensus</h2>
    <button class="reload-btn" onclick={reload}>Reload</button>
  </header>

  {#if !loaded}
    <p class="muted">Loading consensus rows…</p>
  {:else if loadError}
    <div class="error-banner">
      <strong>Failed to load consensus rows.</strong>
      <pre>{loadError}</pre>
    </div>
  {:else if rows.length === 0}
    <p class="muted">
      No consensus rows recorded yet. Rows are emitted by the orchestrator
      after <code>agent_pool::run_consensus_pass</code> runs at least one
      multi-verifier consensus pass.
    </p>
  {:else}
    <table>
      <thead>
        <tr>
          <th>Time</th>
          <th>Kind</th>
          <th>Candidate</th>
          <th>Template</th>
          <th>Rule</th>
          <th>Verifiers</th>
          <th>Pass</th>
          <th>Failing / Reason</th>
        </tr>
      </thead>
      <tbody>
        {#each rows as row (row.seq)}
          <tr class="row-{row.kind}">
            <td>{fmtTs(row.ts)}</td>
            <td><span class="badge badge-kind-{row.kind}">{row.kind}</span></td>
            <td><code>{shortId(row.candidate_id)}</code></td>
            <td><code>{shortId(row.template_id)}</code></td>
            <td><span class="badge badge-rule">{row.rule_id}</span></td>
            <td>{row.verifier_count ?? '—'}</td>
            <td>{row.pass_count ?? '—'}</td>
            <td>
              {#if row.kind === 'fail'}
                <span class="reason">{row.reason ?? ''}</span>
                {#if row.failing_verifier_ids.length > 0}
                  <span class="failing">
                    [{row.failing_verifier_ids.map(shortId).join(', ')}]
                  </span>
                {/if}
              {:else if row.kind === 'ppm_unavailable' || row.kind === 'verifier_disagreement' || row.kind === 'cross_context_dispatched'}
                <span class="reason">{row.reason ?? row.kind}</span>
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
  .verification-consensus-panel {
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

  table {
    width: 100%;
    border-collapse: collapse;
  }

  th, td {
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

  .row-fail {
    background: rgba(255, 80, 80, 0.05);
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

  .badge-kind-pass {
    background: rgba(80, 200, 120, 0.15);
    border-color: rgba(80, 200, 120, 0.4);
  }

  .badge-kind-fail {
    background: rgba(220, 80, 80, 0.15);
    border-color: rgba(220, 80, 80, 0.4);
  }

  /* D-3 closure: badge styles for the 3 new SSB variants surfaced by
     commands_d3 once the Rust-side `consensus_detail_to_row` matches them. */
  .badge-kind-ppm_unavailable {
    background: rgba(255, 170, 60, 0.15);
    border-color: rgba(255, 170, 60, 0.4);
  }

  .badge-kind-verifier_disagreement {
    background: rgba(255, 200, 60, 0.15);
    border-color: rgba(255, 200, 60, 0.4);
  }

  .badge-kind-cross_context_dispatched {
    background: rgba(120, 140, 255, 0.15);
    border-color: rgba(120, 140, 255, 0.4);
  }

  .badge-rule {
    background: var(--bg-surface);
  }

  .reason {
    color: var(--text-secondary);
  }

  .failing {
    color: var(--text-secondary);
    font-size: 10px;
    margin-left: 4px;
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
