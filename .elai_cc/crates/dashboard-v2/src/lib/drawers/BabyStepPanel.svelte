<script lang="ts">
  // D-12 — BabyStepPanel.
  //
  // Three sections:
  //   1. Outcome histogram (pass / fail / inapplicable) over the recent
  //      `baby_step_dispatched` SSB rows.
  //   2. Per-checker-kind histogram — how many dispatches used each
  //      CheckerKind label (a single dispatch can contribute to more
  //      than one kind when the procedure had no `checker_kind_hint`).
  //   3. Recent dispatches table grouped by parent_task_id, so the
  //      user can see whether a whole task family is failing
  //      OperationalizationCheck.
  //
  // No producer logic lives here — `agent_pool::run_operational_check`
  // emits the SSB row server-side. Polls on a 10s cadence matching
  // the other D-{10,11,13} panels.
  //
  // Grounding: arxiv:2305.16291 Voyager (T1) skill-with-measurable-effect;
  // arxiv:2407.01489 AGENTLESS (T1) single-step verification.

  import { onMount, onDestroy } from 'svelte';

  type BabyStepDispatchedRow = {
    step_id: string;
    parent_task_id: string;
    checker_kinds_used: string[];
    outcome: string;
    ts_ms: number;
  };

  type RecentBabyStepDispatches = {
    rows: BabyStepDispatchedRow[];
    counts_by_outcome: Record<string, number>;
    counts_by_kind: Record<string, number>;
    total: number;
  };

  const OUTCOMES: ReadonlyArray<string> = ['pass', 'fail', 'inapplicable'];

  const OUTCOME_LABELS: Record<string, string> = {
    pass: 'PASS',
    fail: 'FAIL',
    inapplicable: 'INAPPLICABLE',
  };

  let dispatches = $state<RecentBabyStepDispatches>({
    rows: [],
    counts_by_outcome: {},
    counts_by_kind: {},
    total: 0,
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
      console.warn(`[baby-step] invoke('${cmd}') failed`, e);
      return undefined;
    }
  }

  async function reload() {
    const r = await callInvoke<RecentBabyStepDispatches>(
      'get_baby_step_dispatches_recent',
      { limit: 50 },
    );
    if (r !== undefined) dispatches = r;
    loaded = true;
  }

  function outcomeCount(o: string): number {
    return dispatches.counts_by_outcome[o] ?? 0;
  }

  function outcomePct(o: string): number {
    if (dispatches.total === 0) return 0;
    return Math.round((outcomeCount(o) / dispatches.total) * 100);
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

  function outcomeClass(o: string): string {
    switch (o) {
      case 'pass':
        return 'result-pass';
      case 'fail':
        return 'result-fail';
      case 'inapplicable':
        return 'result-inapplicable';
      default:
        return 'result-unknown';
    }
  }

  function groupRowsByParent(
    rows: BabyStepDispatchedRow[],
  ): Array<{ parent: string; rows: BabyStepDispatchedRow[] }> {
    const groups = new Map<string, BabyStepDispatchedRow[]>();
    for (const r of rows) {
      const key = r.parent_task_id || '(unknown)';
      const list = groups.get(key) ?? [];
      list.push(r);
      groups.set(key, list);
    }
    const out: Array<{ parent: string; rows: BabyStepDispatchedRow[] }> = [];
    for (const [parent, list] of groups.entries()) {
      out.push({ parent, rows: list });
    }
    out.sort((a, b) => a.parent.localeCompare(b.parent));
    return out;
  }

  function sortedKindKeys(): string[] {
    return Object.keys(dispatches.counts_by_kind).sort();
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
  <h2>Baby-Step Dispatches</h2>

  {#if !loaded}
    <p class="muted">Loading…</p>
  {:else}
    <section class="summary">
      <header>
        <strong>Pass rate</strong>
        <span class="muted">{dispatches.total} dispatches</span>
      </header>
      {#if dispatches.total === 0}
        <p class="muted">No baby_step_dispatched rows recorded.</p>
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
              <td>
                <div class="bar" style="width: {outcomePct(o)}%"></div>
              </td>
            </tr>
          {/each}
        </tbody>
      </table>
    </section>

    <section class="kind-hist">
      <header>
        <strong>Checker kinds used</strong>
        <span class="muted">{sortedKindKeys().length} kinds</span>
      </header>
      {#if sortedKindKeys().length === 0}
        <p class="muted">No checker kinds recorded.</p>
      {:else}
        <table>
          <thead>
            <tr><th>Kind</th><th>Count</th></tr>
          </thead>
          <tbody>
            {#each sortedKindKeys() as k}
              <tr>
                <td><code>{k}</code></td>
                <td>{dispatches.counts_by_kind[k] ?? 0}</td>
              </tr>
            {/each}
          </tbody>
        </table>
      {/if}
    </section>

    <section class="grouped">
      <header>
        <strong>Recent dispatches by parent task</strong>
        <span class="muted">{dispatches.rows.length} rows</span>
      </header>
      {#if dispatches.rows.length === 0}
        <p class="muted">No baby_step_dispatched rows recorded.</p>
      {:else}
        {#each groupRowsByParent(dispatches.rows) as group}
          <div class="group">
            <h3>{group.parent}</h3>
            <table>
              <thead>
                <tr>
                  <th>When</th>
                  <th>Step</th>
                  <th>Outcome</th>
                  <th>Checker kinds</th>
                </tr>
              </thead>
              <tbody>
                {#each group.rows as row}
                  <tr>
                    <td>{fmtTs(row.ts_ms)}</td>
                    <td><code>{row.step_id}</code></td>
                    <td><span class={outcomeClass(row.outcome)}>{OUTCOME_LABELS[row.outcome] ?? row.outcome}</span></td>
                    <td>
                      {#if row.checker_kinds_used.length === 0}
                        <span class="muted">—</span>
                      {:else}
                        {#each row.checker_kinds_used as k, i}
                          <code>{k}</code>{#if i < row.checker_kinds_used.length - 1},&nbsp;{/if}
                        {/each}
                      {/if}
                    </td>
                  </tr>
                {/each}
              </tbody>
            </table>
          </div>
        {/each}
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

  h3 {
    margin: 0.75rem 0 0.35rem 0;
    font-size: 0.85rem;
    text-transform: uppercase;
    letter-spacing: 0.06em;
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
    height: 8px;
    background: var(--accent, #4a8);
    border-radius: 2px;
  }

  .passrate {
    margin: 0;
    font-size: 1.2rem;
  }

  .result-pass {
    color: #4a8;
  }
  .result-fail {
    color: #c66;
  }
  .result-inapplicable {
    color: #a8f;
  }
  .result-unknown {
    color: var(--muted, #888);
  }

  .group {
    margin-bottom: 0.75rem;
  }

  code {
    font-family: inherit;
    font-size: inherit;
  }
</style>
