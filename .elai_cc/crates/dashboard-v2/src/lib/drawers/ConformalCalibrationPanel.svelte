<script lang="ts">
  // D-16 — ConformalCalibrationPanel.
  //
  // Read-only view of the UniCR conformal-refusal calibration:
  //   * recent tau-compute rows (the threshold + corpus size + epsilon)
  //   * recent per-claim refusal decisions (abstain vs answer)
  //   * a tiny histogram of decisions across the returned refusal rows
  //
  // Two Tauri commands (additive — see
  // crates/dashboard-v2/src-tauri/src/commands_d16.rs):
  //   * get_conformal_recent(limit)        -> tau + refusal rows + counts
  //   * get_calibration_corpus_state()     -> corpus snapshot + epsilon
  //
  // The orchestrator (NOT the policy model) emits the underlying SSB
  // rows via conformal_calibration::compute_tau_and_emit +
  // decide_and_emit, both reachable through
  // agent_pool::dispatch_conformal_refusal. Polls on a 10s cadence
  // matching the other D-{1..13} panels.
  //
  // Grounding: arXiv:2503.01345 (UniCR — unified conformal refusal);
  // Vovk et al. *Algorithmic Learning in a Random World* (2005) §2.2.

  import { onMount, onDestroy } from 'svelte';

  type ConformalTauRow = {
    tau: number;
    epsilon: number;
    corpus_size: number;
    source: string;
    ts_ms: number;
  };

  type ConformalRefusalRow = {
    claim_id: string;
    claim_score: number;
    tau: number;
    decision: string;
    confidence: number;
    ts_ms: number;
  };

  type RecentConformal = {
    tau_rows: ConformalTauRow[];
    refusal_rows: ConformalRefusalRow[];
    abstain_count: number;
    answer_count: number;
  };

  type CalibrationCorpusStateDto = {
    source: string;
    size: number;
    frozen: boolean;
    epsilon: number;
  };

  let recent = $state<RecentConformal>({
    tau_rows: [],
    refusal_rows: [],
    abstain_count: 0,
    answer_count: 0,
  });
  let corpus = $state<CalibrationCorpusStateDto | null>(null);
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
      console.warn(`[conformal] invoke('${cmd}') failed`, e);
      return undefined;
    }
  }

  async function reload() {
    const r = await callInvoke<RecentConformal>('get_conformal_recent', {
      limit: 50,
    });
    if (r !== undefined) {
      recent = r;
    }
    const c = await callInvoke<CalibrationCorpusStateDto>('get_calibration_corpus_state');
    if (c !== undefined) {
      corpus = c;
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

  function fmt3(n: number): string {
    if (!Number.isFinite(n)) return '—';
    return n.toFixed(3);
  }

  function decisionClass(d: string): string {
    switch (d) {
      case 'abstain':
        return 'decision-abstain';
      case 'answer':
        return 'decision-answer';
      default:
        return 'decision-unknown';
    }
  }

  function histogramPct(count: number, total: number): number {
    if (total <= 0) return 0;
    const v = (count * 100) / total;
    return Math.max(0, Math.min(100, Math.round(v)));
  }

  const totalDecisions = $derived(recent.abstain_count + recent.answer_count);

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
  <h2>Conformal Calibration (UniCR — Mode E)</h2>

  {#if !loaded}
    <p class="muted">Loading…</p>
  {:else}
    <section class="corpus">
      <header>
        <strong>Active corpus</strong>
      </header>
      {#if corpus === null}
        <p class="muted">No calibration corpus configured (Mode E claims will fall back to ConservativeFallback).</p>
      {:else}
        <table>
          <tbody>
            <tr>
              <th>Source</th>
              <td><code>{corpus.source}</code></td>
            </tr>
            <tr>
              <th>Size</th>
              <td>{corpus.size}</td>
            </tr>
            <tr>
              <th>Frozen</th>
              <td>{corpus.frozen ? 'yes' : 'no'}</td>
            </tr>
            <tr>
              <th>Epsilon (ε)</th>
              <td>{fmt3(corpus.epsilon)}</td>
            </tr>
          </tbody>
        </table>
      {/if}
    </section>

    <section class="tau">
      <header>
        <strong>Recent tau computations</strong>
        <span class="muted">{recent.tau_rows.length} rows</span>
      </header>
      {#if recent.tau_rows.length === 0}
        <p class="muted">No tau computations recorded yet.</p>
      {:else}
        <table>
          <thead>
            <tr>
              <th>When</th>
              <th>τ</th>
              <th>ε</th>
              <th>Corpus</th>
              <th>Source</th>
            </tr>
          </thead>
          <tbody>
            {#each recent.tau_rows as row}
              <tr>
                <td>{fmtTs(row.ts_ms)}</td>
                <td>{fmt3(row.tau)}</td>
                <td>{fmt3(row.epsilon)}</td>
                <td>{row.corpus_size}</td>
                <td><code>{row.source}</code></td>
              </tr>
            {/each}
          </tbody>
        </table>
      {/if}
    </section>

    <section class="histogram">
      <header>
        <strong>Decision histogram</strong>
        <span class="muted">
          {recent.abstain_count} abstain / {recent.answer_count} answer
          ({totalDecisions} total)
        </span>
      </header>
      <div class="bar">
        <div
          class="bar-segment bar-abstain"
          style="width: {histogramPct(recent.abstain_count, totalDecisions)}%"
          aria-label="abstain"
        ></div>
        <div
          class="bar-segment bar-answer"
          style="width: {histogramPct(recent.answer_count, totalDecisions)}%"
          aria-label="answer"
        ></div>
      </div>
    </section>

    <section class="refusals">
      <header>
        <strong>Recent refusal decisions</strong>
        <span class="muted">{recent.refusal_rows.length} rows</span>
      </header>
      {#if recent.refusal_rows.length === 0}
        <p class="muted">No refusal decisions recorded yet.</p>
      {:else}
        <table>
          <thead>
            <tr>
              <th>When</th>
              <th>Claim</th>
              <th>Score</th>
              <th>τ</th>
              <th>Decision</th>
              <th>Confidence</th>
            </tr>
          </thead>
          <tbody>
            {#each recent.refusal_rows as row}
              <tr>
                <td>{fmtTs(row.ts_ms)}</td>
                <td><code>{shortId(row.claim_id)}</code></td>
                <td>{fmt3(row.claim_score)}</td>
                <td>{fmt3(row.tau)}</td>
                <td><span class={decisionClass(row.decision)}>{row.decision || '—'}</span></td>
                <td>{fmt3(row.confidence)}</td>
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

  .bar {
    display: flex;
    width: 100%;
    height: 0.75rem;
    background: var(--bar-bg, #1a1a1a);
    border: 1px solid var(--border, #2a2a2a);
    overflow: hidden;
  }

  .bar-segment {
    height: 100%;
  }

  .bar-abstain {
    background: #c66;
  }

  .bar-answer {
    background: #4a8;
  }

  .decision-abstain {
    color: #c66;
  }

  .decision-answer {
    color: #4a8;
  }

  .decision-unknown {
    color: var(--muted, #888);
  }

  code {
    font-family: inherit;
    font-size: inherit;
  }
</style>
