<script lang="ts">
  // D-11 — CheckerRegistryPanel.
  //
  // Two sections:
  //   1. Registered checker kinds (closed enum — sympy, z3, compile_rust,
  //      regex_match, json_schema_validate, numeric_range). Sourced from
  //      `get_checker_registry_state` (static table — always available).
  //   2. Recent `checker_result_emitted` SSB rows + per-result-kind
  //      histogram. Sourced from `get_checker_results_recent`.
  //
  // No producer logic lives here — `agent_pool::dispatch_constructive_checkers`
  // emits the SSB row server-side when the D-10 router selects
  // GroundingMode::Constructive. Polls on a 10s cadence matching the
  // other D-{1..10} panels.
  //
  // Grounding: arxiv:2310.04406 (constructive proof checking),
  // arxiv:2211.12588 (PoT, T1), arxiv:2402.01030 (CodeAct, T1).

  import { onMount, onDestroy } from 'svelte';

  type CheckerResultRow = {
    checker_id: string;
    checker_kind: string;
    result_kind: string;
    claim_id: string;
    ts_ms: number;
    reason: string;
  };

  type RecentCheckerResults = {
    rows: CheckerResultRow[];
    counts_by_kind: Record<string, number>;
    total: number;
  };

  type CheckerKindEntry = {
    kind: string;
    description: string;
    builtin_available: boolean;
  };

  type CheckerRegistryState = {
    kinds: CheckerKindEntry[];
  };

  const RESULT_KINDS: ReadonlyArray<string> = ['pass', 'fail', 'inapplicable', 'error'];

  const RESULT_LABELS: Record<string, string> = {
    pass: 'PASS',
    fail: 'FAIL',
    inapplicable: 'INAPPLICABLE',
    error: 'ERROR',
  };

  let results = $state<RecentCheckerResults>({
    rows: [],
    counts_by_kind: {},
    total: 0,
  });
  let registry = $state<CheckerRegistryState>({ kinds: [] });
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
      console.warn(`[checker-registry] invoke('${cmd}') failed`, e);
      return undefined;
    }
  }

  async function reload() {
    const r = await callInvoke<RecentCheckerResults>('get_checker_results_recent', {
      limit: 50,
    });
    if (r !== undefined) results = r;
    const s = await callInvoke<CheckerRegistryState>('get_checker_registry_state');
    if (s !== undefined) registry = s;
    loaded = true;
  }

  function pct(kind: string): number {
    if (results.total === 0) return 0;
    const n = results.counts_by_kind[kind] ?? 0;
    return Math.round((n / results.total) * 100);
  }

  function kindCount(kind: string): number {
    return results.counts_by_kind[kind] ?? 0;
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

  function resultClass(kind: string): string {
    switch (kind) {
      case 'pass':
        return 'result-pass';
      case 'fail':
        return 'result-fail';
      case 'inapplicable':
        return 'result-inapplicable';
      case 'error':
        return 'result-error';
      default:
        return 'result-unknown';
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

<div class="panel">
  <h2>Checker Registry</h2>

  {#if !loaded}
    <p class="muted">Loading…</p>
  {:else}
    <section class="registry">
      <header>
        <strong>Registered checker kinds</strong>
        <span class="muted">{registry.kinds.length} kinds</span>
      </header>
      {#if registry.kinds.length === 0}
        <p class="muted">No checker kinds reported.</p>
      {:else}
        <table>
          <thead>
            <tr><th>Kind</th><th>Available</th><th>Description</th></tr>
          </thead>
          <tbody>
            {#each registry.kinds as entry}
              <tr>
                <td><code>{entry.kind}</code></td>
                <td>{entry.builtin_available ? 'yes' : 'deferred'}</td>
                <td>{entry.description}</td>
              </tr>
            {/each}
          </tbody>
        </table>
      {/if}
    </section>

    <section class="histogram">
      <header>
        <strong>Result distribution</strong>
        <span class="muted">{results.total} results</span>
      </header>
      {#if results.total === 0}
        <p class="muted">No checker results recorded.</p>
      {:else}
        <table>
          <thead>
            <tr><th>Kind</th><th>Count</th><th>%</th><th>Bar</th></tr>
          </thead>
          <tbody>
            {#each RESULT_KINDS as kind}
              <tr>
                <td><span class={resultClass(kind)}>{RESULT_LABELS[kind] ?? kind}</span></td>
                <td>{kindCount(kind)}</td>
                <td>{pct(kind)}%</td>
                <td>
                  <div class="bar" style="width: {pct(kind)}%"></div>
                </td>
              </tr>
            {/each}
          </tbody>
        </table>
      {/if}
    </section>

    <section class="recent">
      <header>
        <strong>Recent results</strong>
        <span class="muted">{results.rows.length} rows</span>
      </header>
      {#if results.rows.length === 0}
        <p class="muted">No checker_result_emitted rows recorded.</p>
      {:else}
        <table>
          <thead>
            <tr>
              <th>When</th>
              <th>Checker</th>
              <th>Kind</th>
              <th>Result</th>
              <th>Claim</th>
              <th>Reason</th>
            </tr>
          </thead>
          <tbody>
            {#each results.rows as row}
              <tr>
                <td>{fmtTs(row.ts_ms)}</td>
                <td><code>{row.checker_id}</code></td>
                <td><code>{row.checker_kind}</code></td>
                <td><span class={resultClass(row.result_kind)}>{RESULT_LABELS[row.result_kind] ?? row.result_kind}</span></td>
                <td><code>{shortId(row.claim_id)}</code></td>
                <td class="reason">{row.reason}</td>
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
    height: 8px;
    background: var(--accent, #4a8);
    border-radius: 2px;
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
  .result-error {
    color: #f93;
  }
  .result-unknown {
    color: var(--muted, #888);
  }

  .reason {
    max-width: 30rem;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  code {
    font-family: inherit;
    font-size: inherit;
  }
</style>
