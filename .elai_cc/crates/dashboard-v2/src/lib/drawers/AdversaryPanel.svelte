<script lang="ts">
  // D-13 — AdversaryPanel.
  //
  // Read-only view of recent adversarial-verification runs:
  //   * counter-evidence emitted by the cross-family adversary
  //     (severity, model family, claim_id)
  //   * the consensus verdict transition (original -> post-counter)
  //
  // Two Tauri commands (additive — see
  // crates/dashboard-v2/src-tauri/src/commands_d13.rs):
  //   * get_adversarial_runs_recent(limit)  -> rows + total + inversions
  //   * get_adversary_config()              -> active AdversaryConfig
  //
  // The orchestrator (NOT the policy model) emits the underlying SSB
  // rows via agent_pool::run_adversarial_pass. Polls on a 10s cadence
  // matching the other D-{1..11} panels.
  //
  // Grounding: arxiv:2403.07974 (T1) adversarial verification;
  // arxiv:2504.16760 (T1) lightweight latent verifiers (heterogeneous-
  // family verification as the documented defense against same-family
  // verifier-transfer failure).

  import { onMount, onDestroy } from 'svelte';

  type AdversaryRunRow = {
    counter_id: string;
    claim_id: string;
    severity: number;
    model_family: string;
    original_verdict: string;
    post_counter_verdict: string;
    ts_ms: number;
  };

  type RecentAdversarialRuns = {
    rows: AdversaryRunRow[];
    total: number;
    verdict_inversions: number;
  };

  type AdversaryConfigDto = {
    adversary_model_family: string;
    max_counter_attempts: number;
    severity_threshold: number;
  };

  let runs = $state<RecentAdversarialRuns>({
    rows: [],
    total: 0,
    verdict_inversions: 0,
  });
  let config = $state<AdversaryConfigDto | null>(null);
  let loaded = $state(false);
  let pollHandle: ReturnType<typeof setInterval> | null = null;

  async function callInvoke<T>(
    cmd: string,
    args?: Record<string, unknown>,
  ): Promise<T | undefined> {
    if (typeof window === 'undefined') return undefined;
    const tauriAvail = !!(window as unknown as { __TAURI_INTERNALS__?: unknown }).__TAURI_INTERNALS__;
    if (!tauriAvail) return undefined;
    try {
      const mod = await import('@tauri-apps/api/core');
      return (await mod.invoke<T>(cmd, args)) as T;
    } catch (e) {
      console.warn(`[adversary] invoke('${cmd}') failed`, e);
      return undefined;
    }
  }

  async function reload() {
    const r = await callInvoke<RecentAdversarialRuns>('get_adversarial_runs_recent', {
      limit: 50,
    });
    if (r !== undefined) {
      runs = r;
    }
    const c = await callInvoke<AdversaryConfigDto>('get_adversary_config');
    if (c !== undefined) {
      config = c;
    }
    loaded = true;
  }

  function shortId(id: string): string {
    if (id.length <= 16) return id;
    return `${id.slice(0, 8)}…${id.slice(-4)}`;
  }

  function fmtTs(ms: number | null | undefined): string {
    if (!ms || ms <= 0) return '—';
    try {
      return new Date(ms).toISOString().slice(0, 19).replace('T', ' ');
    } catch {
      return '—';
    }
  }

  function fmtSeverity(s: number): string {
    if (!Number.isFinite(s)) return '—';
    return s.toFixed(2);
  }

  function verdictClass(v: string): string {
    switch (v) {
      case 'pass':
        return 'verdict-pass';
      case 'fail':
        return 'verdict-fail';
      case 'inconclusive':
        return 'verdict-inc';
      default:
        return 'verdict-unknown';
    }
  }

  function inverted(row: AdversaryRunRow): boolean {
    return (
      row.original_verdict !== '' &&
      row.original_verdict !== row.post_counter_verdict
    );
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
  <h2>Adversary (Cross-Family)</h2>

  {#if !loaded}
    <p class="muted">Loading…</p>
  {:else}
    <section class="config">
      <header>
        <strong>Active configuration</strong>
      </header>
      {#if config === null}
        <p class="muted">No adversary configured (Mode B claims will fall to Mode E).</p>
      {:else}
        <table>
          <tbody>
            <tr>
              <th>Adversary family</th>
              <td><code>{config.adversary_model_family}</code></td>
            </tr>
            <tr>
              <th>Max counter attempts</th>
              <td>{config.max_counter_attempts}</td>
            </tr>
            <tr>
              <th>Severity threshold</th>
              <td>{fmtSeverity(config.severity_threshold)}</td>
            </tr>
          </tbody>
        </table>
      {/if}
    </section>

    <section class="summary">
      <header>
        <strong>Recent runs</strong>
        <span class="muted">{runs.total} total, {runs.verdict_inversions} inverted</span>
      </header>
      {#if runs.rows.length === 0}
        <p class="muted">No adversarial runs recorded yet.</p>
      {:else}
        <table>
          <thead>
            <tr>
              <th>When</th>
              <th>Claim</th>
              <th>Family</th>
              <th>Severity</th>
              <th>Original</th>
              <th>Post-counter</th>
            </tr>
          </thead>
          <tbody>
            {#each runs.rows as row}
              <tr class:inverted={inverted(row)}>
                <td>{fmtTs(row.ts_ms)}</td>
                <td><code>{shortId(row.claim_id)}</code></td>
                <td><code>{row.model_family}</code></td>
                <td>{fmtSeverity(row.severity)}</td>
                <td><span class={verdictClass(row.original_verdict)}>{row.original_verdict || '—'}</span></td>
                <td><span class={verdictClass(row.post_counter_verdict)}>{row.post_counter_verdict || '—'}</span></td>
              </tr>
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

  table {
    border-collapse: collapse;
    width: 100%;
  }

  th, td {
    padding: 0.35rem 0.5rem;
    text-align: left;
    border-bottom: 1px solid var(--border, #2a2a2a);
    vertical-align: middle;
  }

  tr.inverted {
    background: rgba(255, 153, 51, 0.08);
  }

  .verdict-pass {
    color: #4a8;
  }

  .verdict-fail {
    color: #c66;
  }

  .verdict-inc {
    color: #fc6;
  }

  .verdict-unknown {
    color: var(--muted, #888);
  }

  code {
    font-family: inherit;
    font-size: inherit;
  }
</style>
