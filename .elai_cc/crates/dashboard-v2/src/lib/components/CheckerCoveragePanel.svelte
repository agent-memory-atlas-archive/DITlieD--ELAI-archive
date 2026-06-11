<script lang="ts">
  // H-41: CheckerCoveragePanel.
  //
  // Displays two pieces of information from the H-41 checker-gap instrument:
  //   1. Mode-A coverage fraction tile — what fraction of Factual/Computational
  //      claims reached a sound Constructive checker this session.
  //   2. Ranked checker-gap backlog — claim-class x complexity-taxon pairs that
  //      fell to non-Constructive for lack of a Deterministic verifier, sorted
  //      by frequency x taxon-priority. HIGH-priority rows (NP-complete /
  //      undecidable) appear at the top and are flagged "never-LLM-grade" per
  //      Xu arXiv:2401.11817 RD-3.
  //
  // Wire-dark: rows only appear when ELAI_CHECKER_GAP_ENABLE=1 is set on the
  // orchestrator writer side. The panel renders empty/zero when the gate is
  // OFF — that is the CORRECT signal per GD-5 (coverage = REACH, not
  // checker correctness). An empty backlog is also correct when all
  // Factual/Computational claims have a Deterministic verifier available.
  //
  // Polls get_checker_gap_backlog + get_coverage_report every `pollMs` ms.
  //
  // Grounding:
  //   Kambhampati arXiv:2402.08115 — Mode-A coverage is the verification lever.
  //   Xu arXiv:2401.11817 Table 3 — NP-complete / undecidable = never-LLM-grade.

  import { onMount, onDestroy } from 'svelte';
  import type { CheckerGapBacklogRow, CoverageReport } from '../api/types';
  import { invoke } from '@tauri-apps/api/core';

  interface Props {
    limit?: number;
    pollMs?: number;
    sessionId?: string;
  }
  const { limit = 20, pollMs = 15_000, sessionId = undefined }: Props = $props();

  let coverage: CoverageReport = $state({
    mode_a_fraction: 0,
    constructive_count: 0,
    total_factual_computational: 0,
    session_id: 'all'
  });
  let backlog: CheckerGapBacklogRow[] = $state([]);
  let loading = $state(true);
  let error: string | null = $state(null);
  let interval: ReturnType<typeof setInterval> | null = null;

  async function refresh() {
    try {
      const [cov, gaps] = await Promise.all([
        invoke<CoverageReport>('get_coverage_report', {
          sessionId: sessionId ?? null
        }),
        invoke<CheckerGapBacklogRow[]>('get_checker_gap_backlog', { limit })
      ]);
      coverage = cov;
      backlog = gaps;
      error = null;
    } catch (e) {
      error = String(e);
    } finally {
      loading = false;
    }
  }

  onMount(() => {
    refresh();
    interval = setInterval(refresh, pollMs);
  });

  onDestroy(() => {
    if (interval !== null) clearInterval(interval);
  });

  function pct(f: number): string {
    return (f * 100).toFixed(1) + '%';
  }

  function priorityLabel(p: number): string {
    return p >= 2 ? 'HIGH' : 'normal';
  }

  function priorityClass(p: number): string {
    return p >= 2 ? 'priority-high' : 'priority-normal';
  }
</script>

<div class="checker-coverage-panel">
  <h3 class="panel-title">Checker Coverage (H-41)</h3>

  {#if loading}
    <p class="status">Loading…</p>
  {:else if error}
    <p class="status error">Error: {error}</p>
  {:else}
    <!-- Mode-A fraction tile -->
    <div class="coverage-tile">
      <span class="coverage-label">Mode-A coverage</span>
      <span class="coverage-value">{pct(coverage.mode_a_fraction)}</span>
      <span class="coverage-detail">
        {coverage.constructive_count} Constructive / {coverage.total_factual_computational} F+C claims
      </span>
    </div>

    <!-- Gap backlog -->
    {#if backlog.length === 0}
      <p class="status">
        No checker-gap rows recorded.
        {#if coverage.total_factual_computational === 0}
          (No Factual/Computational claims seen yet.)
        {:else}
          (ELAI_CHECKER_GAP_ENABLE=1 not set, or all F/C claims have a Deterministic verifier.)
        {/if}
      </p>
    {:else}
      <table class="gap-table">
        <thead>
          <tr>
            <th>Claim type</th>
            <th>Complexity taxon</th>
            <th class="num">Count</th>
            <th>Priority</th>
          </tr>
        </thead>
        <tbody>
          {#each backlog as row (row.claim_type + '|' + row.complexity_taxon)}
            <tr class={priorityClass(row.priority)}>
              <td>{row.claim_type}</td>
              <td>{row.complexity_taxon}</td>
              <td class="num">{row.count}</td>
              <td>
                <span class="priority-badge {priorityClass(row.priority)}">
                  {priorityLabel(row.priority)}
                </span>
              </td>
            </tr>
          {/each}
        </tbody>
      </table>
    {/if}
  {/if}
</div>

<style>
  .checker-coverage-panel {
    font-family: var(--font-mono, monospace);
    font-size: 0.85rem;
    color: var(--color-text, #cdd6f4);
    background: var(--color-surface, #1e1e2e);
    border: 1px solid var(--color-border, #313244);
    border-radius: 4px;
    padding: 0.75rem 1rem;
    min-width: 340px;
  }

  .panel-title {
    font-size: 0.9rem;
    font-weight: 600;
    margin: 0 0 0.5rem;
    color: var(--color-accent, #89b4fa);
  }

  .coverage-tile {
    display: flex;
    flex-direction: column;
    margin-bottom: 0.75rem;
    background: var(--color-mantle, #181825);
    border-radius: 4px;
    padding: 0.5rem 0.75rem;
  }

  .coverage-label {
    font-size: 0.75rem;
    color: var(--color-subtext, #a6adc8);
    text-transform: uppercase;
    letter-spacing: 0.05em;
  }

  .coverage-value {
    font-size: 1.6rem;
    font-weight: 700;
    color: var(--color-green, #a6e3a1);
    line-height: 1.2;
  }

  .coverage-detail {
    font-size: 0.75rem;
    color: var(--color-subtext, #a6adc8);
    margin-top: 0.2rem;
  }

  .status {
    color: var(--color-subtext, #a6adc8);
    font-size: 0.8rem;
    margin: 0.25rem 0;
  }

  .status.error {
    color: var(--color-red, #f38ba8);
  }

  .gap-table {
    width: 100%;
    border-collapse: collapse;
    font-size: 0.8rem;
  }

  .gap-table th {
    text-align: left;
    color: var(--color-subtext, #a6adc8);
    font-weight: 500;
    padding: 0.25rem 0.4rem;
    border-bottom: 1px solid var(--color-border, #313244);
    white-space: nowrap;
  }

  .gap-table th.num,
  .gap-table td.num {
    text-align: right;
  }

  .gap-table td {
    padding: 0.2rem 0.4rem;
    border-bottom: 1px solid var(--color-overlay, #2a2a3e);
    vertical-align: middle;
  }

  .priority-badge {
    display: inline-block;
    border-radius: 3px;
    padding: 0.1rem 0.35rem;
    font-size: 0.7rem;
    font-weight: 600;
    text-transform: uppercase;
  }

  .priority-badge.priority-high {
    background: var(--color-red, #f38ba8);
    color: #1e1e2e;
  }

  .priority-badge.priority-normal {
    background: var(--color-surface1, #313244);
    color: var(--color-text, #cdd6f4);
  }
</style>
