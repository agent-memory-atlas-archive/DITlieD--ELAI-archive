<script lang="ts">
  // D-18 — ObservableEffectPanel.
  //
  // Three sections:
  //   1. Outcome histogram (pass / fail / skipped) over the recent
  //      `observable_effect_verified` SSB rows.
  //   2. Per-sandbox-tier histogram — how many dispatches ran under
  //      each SandboxTier label (no_sandbox / process_isolated /
  //      container_isolated / network_blocked).
  //   3. Recent dispatches table with per-row confound-flag chips.
  //
  // No producer logic lives here — the orchestrator producer
  // (`elai_orchestrator::observable_effect::verify_observable_effect`)
  // emits the SSB row server-side. Polls on a 10s cadence matching the
  // sibling D-{10,11,12,13,16} panels.
  //
  // Grounding: see crates/orchestrator/src/observable_effect.rs
  // TODO_DOC[D-18] for the paper-ingestor follow-up (candidate anchors
  // Voyager arxiv:2305.16291 + Coscientist arxiv:2304.05332).

  import { onMount, onDestroy } from 'svelte';

  // D-18 closure: import the canonical wire types from
  // $lib/api/types so dashboard renderers stay in lockstep with the
  // Tauri command DTO surface.
  import type {
    ObservableEffectRow,
    RecentObservableEffects,
    ObservableEffectDispatched,
    GoodhartFlag,
    SandboxObservation,
    ProductionObservationProposal,
  } from '$lib/api/types';

  const OUTCOMES: ReadonlyArray<string> = ['pass', 'fail', 'skipped'];
  const OUTCOME_LABELS: Record<string, string> = {
    pass: 'PASS',
    fail: 'FAIL',
    skipped: 'SKIPPED',
  };

  const SANDBOX_TIERS: ReadonlyArray<string> = [
    'no_sandbox',
    'process_isolated',
    'container_isolated',
    'network_blocked',
  ];

  let recent = $state<RecentObservableEffects>({
    rows: [],
    counts_by_outcome: {},
    counts_by_sandbox_tier: {},
    total: 0,
    dispatched_rows: [],
    goodhart_flags: [],
    sandbox_observations: [],
    production_proposals: [],
  });
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
      console.warn(`[observable-effect] invoke('${cmd}') failed`, e);
      return undefined;
    }
  }

  async function reload() {
    const r = await callInvoke<RecentObservableEffects>(
      'get_observable_effects_recent',
      { limit: 50 },
    );
    if (r !== undefined) recent = r;
    loaded = true;
  }

  function outcomeCount(o: string): number {
    return recent.counts_by_outcome[o] ?? 0;
  }

  function outcomePct(o: string): number {
    if (recent.total === 0) return 0;
    return Math.round((outcomeCount(o) / recent.total) * 100);
  }

  function tierCount(t: string): number {
    return recent.counts_by_sandbox_tier[t] ?? 0;
  }

  function tierPct(t: string): number {
    if (recent.total === 0) return 0;
    return Math.round((tierCount(t) / recent.total) * 100);
  }

  function passRatePct(): number {
    return outcomePct('pass');
  }

  function fmtTs(ms: number | null | undefined): string {
    if (!ms || ms <= 0) return '—';
    try {
      return new Date(ms).toISOString().slice(0, 19).replace('T', ' ');
    } catch {
      return '—';
    }
  }

  function fmtElapsedUs(us: number | null | undefined): string {
    if (!us || us <= 0) return '—';
    if (us < 1000) return `${us}us`;
    if (us < 1_000_000) return `${(us / 1000).toFixed(1)}ms`;
    return `${(us / 1_000_000).toFixed(2)}s`;
  }

  function outcomeClass(o: string): string {
    switch (o) {
      case 'pass':
        return 'result-pass';
      case 'fail':
        return 'result-fail';
      case 'skipped':
        return 'result-skipped';
      default:
        return 'result-unknown';
    }
  }

  function confoundFlagsList(s: string): string[] {
    if (!s) return [];
    return s.split(',').filter((x) => x.length > 0);
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
  <h2>Observable Effects</h2>

  {#if !loaded}
    <p class="muted">Loading…</p>
  {:else}
    <section class="summary">
      <header>
        <strong>Pass rate</strong>
        <span class="muted">{recent.total} verifications</span>
      </header>
      {#if recent.total === 0}
        <p class="muted">No observable_effect_verified rows recorded.</p>
      {:else}
        <p class="passrate">
          <strong>{passRatePct()}%</strong>
          <span class="muted">pass</span>
        </p>
      {/if}
    </section>

    <section class="outcome-hist">
      <header>
        <strong>Outcome distribution</strong>
      </header>
      <table>
        <thead>
          <tr><th>Outcome</th><th>Count</th><th>%</th><th>Bar</th></tr>
        </thead>
        <tbody>
          {#each OUTCOMES as o}
            <tr>
              <td><span class={outcomeClass(o)}>{OUTCOME_LABELS[o] ?? o}</span></td>
              <td>{outcomeCount(o)}</td>
              <td>{outcomePct(o)}%</td>
              <td><div class="bar" style="width: {outcomePct(o)}%"></div></td>
            </tr>
          {/each}
        </tbody>
      </table>
    </section>

    <section class="tier-hist">
      <header>
        <strong>Sandbox tier usage</strong>
      </header>
      <table>
        <thead>
          <tr><th>Tier</th><th>Count</th><th>%</th><th>Bar</th></tr>
        </thead>
        <tbody>
          {#each SANDBOX_TIERS as t}
            <tr>
              <td><code>{t}</code></td>
              <td>{tierCount(t)}</td>
              <td>{tierPct(t)}%</td>
              <td><div class="bar" style="width: {tierPct(t)}%"></div></td>
            </tr>
          {/each}
        </tbody>
      </table>
    </section>

    <section class="rows">
      <header>
        <strong>Recent verifications</strong>
        <span class="muted">{recent.rows.length} rows</span>
      </header>
      {#if recent.rows.length === 0}
        <p class="muted">No observable_effect_verified rows recorded.</p>
      {:else}
        <table>
          <thead>
            <tr>
              <th>When</th>
              <th>Effect</th>
              <th>Trigger</th>
              <th>Observation</th>
              <th>Tier</th>
              <th>Outcome</th>
              <th>Elapsed</th>
              <th>Confounds</th>
            </tr>
          </thead>
          <tbody>
            {#each recent.rows as row}
              <tr>
                <td>{fmtTs(row.ts_ms)}</td>
                <td><code>{row.effect_id}</code></td>
                <td><code>{row.trigger_kind}</code></td>
                <td><code>{row.observation_kind}</code></td>
                <td><code>{row.sandbox_tier}</code></td>
                <td><span class={outcomeClass(row.outcome)}>{OUTCOME_LABELS[row.outcome] ?? row.outcome}</span></td>
                <td>{fmtElapsedUs(row.elapsed_us)}</td>
                <td>
                  {#if confoundFlagsList(row.confound_flags).length === 0}
                    <span class="muted">—</span>
                  {:else}
                    {#each confoundFlagsList(row.confound_flags) as f}
                      <span class="confound-chip">{f}</span>
                    {/each}
                  {/if}
                </td>
              </tr>
            {/each}
          </tbody>
        </table>
      {/if}
    </section>

    <section class="dispatched">
      <header>
        <strong>Dispatched effects</strong>
        <span class="muted">{recent.dispatched_rows?.length ?? 0} rows</span>
      </header>
      {#if !recent.dispatched_rows || recent.dispatched_rows.length === 0}
        <p class="muted">No observable_effect_dispatched rows yet.</p>
      {:else}
        <table>
          <thead>
            <tr><th>When</th><th>Effect</th><th>Trigger</th><th>Context</th></tr>
          </thead>
          <tbody>
            {#each recent.dispatched_rows as row}
              <tr>
                <td>{fmtTs(row.ts_ms)}</td>
                <td><code>{row.effect_id}</code></td>
                <td><code>{row.trigger}</code></td>
                <td><code>{row.context_kind}</code></td>
              </tr>
            {/each}
          </tbody>
        </table>
      {/if}
    </section>

    <section class="goodhart">
      <header>
        <strong>Goodhart flags</strong>
        <span class="muted">{recent.goodhart_flags?.length ?? 0} flags</span>
      </header>
      {#if !recent.goodhart_flags || recent.goodhart_flags.length === 0}
        <p class="muted">No goodhart_risk_flag rows yet.</p>
      {:else}
        <table>
          <thead>
            <tr><th>When</th><th>Effect</th><th>Risk kind</th><th>Evidence</th></tr>
          </thead>
          <tbody>
            {#each recent.goodhart_flags as flag}
              <tr>
                <td>{fmtTs(flag.ts_ms)}</td>
                <td><code>{flag.effect_id}</code></td>
                <td><code>{flag.risk_kind}</code></td>
                <td>{flag.evidence}</td>
              </tr>
            {/each}
          </tbody>
        </table>
      {/if}
    </section>

    <section class="sandbox-obs">
      <header>
        <strong>Sandbox observations</strong>
        <span class="muted">{recent.sandbox_observations?.length ?? 0} rows</span>
      </header>
      {#if !recent.sandbox_observations || recent.sandbox_observations.length === 0}
        <p class="muted">No sandbox_observation rows yet.</p>
      {:else}
        <table>
          <thead>
            <tr><th>When</th><th>Effect</th><th>Tier</th><th>Outcome</th><th>Evidence</th></tr>
          </thead>
          <tbody>
            {#each recent.sandbox_observations as row}
              <tr>
                <td>{fmtTs(row.ts_ms)}</td>
                <td><code>{row.effect_id}</code></td>
                <td><code>{row.tier}</code></td>
                <td><code>{row.outcome}</code></td>
                <td>{row.evidence_count}</td>
              </tr>
            {/each}
          </tbody>
        </table>
      {/if}
    </section>

    <section class="production-proposals">
      <header>
        <strong>Production observation proposals</strong>
        <span class="muted">{recent.production_proposals?.length ?? 0} rows</span>
      </header>
      {#if !recent.production_proposals || recent.production_proposals.length === 0}
        <p class="muted">No production_observation_proposal rows yet.</p>
      {:else}
        <table>
          <thead>
            <tr><th>When</th><th>Effect</th><th>Instrument file</th><th>Requires review</th></tr>
          </thead>
          <tbody>
            {#each recent.production_proposals as row}
              <tr>
                <td>{fmtTs(row.ts_ms)}</td>
                <td><code>{row.effect_id}</code></td>
                <td>
                  {#if row.instrument_file}
                    <code>{row.instrument_file}</code>
                  {:else}
                    <span class="muted">—</span>
                  {/if}
                </td>
                <td>
                  {#if row.requires_human_review}
                    <span class="result-fail">REVIEW</span>
                  {:else}
                    <span class="result-pass">OK</span>
                  {/if}
                </td>
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
    margin-bottom: 1.25rem;
  }

  section > header {
    display: flex;
    align-items: baseline;
    gap: 0.6rem;
    margin-bottom: 0.35rem;
  }

  table {
    width: 100%;
    border-collapse: collapse;
  }

  th, td {
    text-align: left;
    padding: 0.25rem 0.5rem;
    border-bottom: 1px solid var(--border, #2a2a2a);
    vertical-align: top;
  }

  th {
    font-weight: 600;
    opacity: 0.75;
  }

  code {
    font-size: 0.8rem;
    opacity: 0.92;
  }

  .muted {
    opacity: 0.55;
  }

  .passrate {
    margin: 0.2rem 0 0 0;
    font-size: 1.2rem;
  }

  .bar {
    height: 6px;
    background: var(--accent, #4f8cff);
    border-radius: 2px;
    max-width: 100%;
  }

  .result-pass { color: #6fdb6f; }
  .result-fail { color: #ff6b6b; }
  .result-skipped { color: #d9b04e; }
  .result-unknown { color: #888; }

  .confound-chip {
    display: inline-block;
    padding: 0.05rem 0.4rem;
    margin: 0 0.2rem 0.2rem 0;
    border: 1px solid var(--border, #2a2a2a);
    border-radius: 8px;
    font-size: 0.72rem;
    opacity: 0.85;
  }
</style>
