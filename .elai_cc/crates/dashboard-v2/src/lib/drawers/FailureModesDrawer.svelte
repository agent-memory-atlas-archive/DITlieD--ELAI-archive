<script lang="ts">
  // E-29 Tier-B — FailureModesDrawer.
  // Consumes the E-6 SHIPPED `get_mast_breakdown` Tauri command for the
  // 14-mode MAST taxonomy counts (arXiv:2503.13657 Cemri et al. NeurIPS
  // 2025) plus the E-29 Tier-B observability rollups for the per-category
  // sparkline. The MastFailurePanel (E-6) shows the at-a-glance count;
  // this drawer is the richer view with category bands + filters.
  //
  // Sibling drawer of TranscriptDrawer (Tier-A) / SpendDrawer (Tier-A) /
  // QueryDrawer (Tier-A). Mount via DrawerHost.

  import { onMount, onDestroy } from 'svelte';
  import { invoke } from '@tauri-apps/api/core';

  type BreakdownRow = {
    mast_mode: string;
    agent_role: string;
    count: number;
  };

  type ModeTotal = {
    mast_mode: string;
    total: number;
  };

  type BreakdownResponse = {
    rows: BreakdownRow[];
    totals: ModeTotal[];
    grand_total: number;
  };

  const POLL_INTERVAL_MS = 10_000;
  const ROW_LIMIT = 500;

  // Canonical MAST tag -> category mapping (mirrors crate::mast::MastMode::mode_category).
  const SPECIFICATION = new Set([
    'fm_1_1_disobey_task_spec',
    'fm_1_2_disobey_role_spec',
    'fm_1_3_step_repetition',
    'fm_1_4_loss_of_conversation_history',
    'fm_1_5_unaware_of_termination_conditions',
  ]);
  const INTER_AGENT = new Set([
    'fm_2_1_conversation_reset',
    'fm_2_2_fail_to_ask_for_clarification',
    'fm_2_3_task_derailment',
    'fm_2_4_information_withholding',
    'fm_2_5_ignored_other_agent_input',
    'fm_2_6_reasoning_action_mismatch',
  ]);
  const VERIFICATION = new Set([
    'fm_3_1_premature_termination',
    'fm_3_2_no_or_incomplete_verification',
    'fm_3_3_incorrect_verification',
  ]);

  let totals = $state<ModeTotal[]>([]);
  let rows = $state<BreakdownRow[]>([]);
  let grandTotal = $state<number>(0);
  let loading = $state<boolean>(true);
  let error = $state<string | null>(null);
  let categoryFilter = $state<'all' | 'specification' | 'inter_agent' | 'verification'>('all');
  let timer: ReturnType<typeof setInterval> | null = null;
  let refreshing = false;

  function categoryOf(tag: string): 'specification' | 'inter_agent' | 'verification' | 'unknown' {
    if (SPECIFICATION.has(tag)) return 'specification';
    if (INTER_AGENT.has(tag)) return 'inter_agent';
    if (VERIFICATION.has(tag)) return 'verification';
    return 'unknown';
  }

  async function refresh() {
    if (refreshing) return;
    refreshing = true;
    try {
      const resp = await invoke<BreakdownResponse>('get_mast_breakdown', { limit: ROW_LIMIT });
      totals = Array.isArray(resp?.totals) ? resp.totals : [];
      rows = Array.isArray(resp?.rows) ? resp.rows : [];
      grandTotal = typeof resp?.grand_total === 'number' ? resp.grand_total : 0;
      error = null;
    } catch (e) {
      error = e instanceof Error ? e.message : String(e);
    } finally {
      loading = false;
      refreshing = false;
    }
  }

  $effect(() => {
    if (timer === null) {
      void refresh();
      timer = setInterval(refresh, POLL_INTERVAL_MS);
    }
  });

  onDestroy(() => {
    if (timer !== null) {
      clearInterval(timer);
      timer = null;
    }
  });

  let filteredTotals = $derived(
    categoryFilter === 'all'
      ? totals
      : totals.filter((t) => categoryOf(t.mast_mode) === categoryFilter),
  );

  let categorySum = $derived.by(() => {
    const sum: Record<string, number> = { specification: 0, inter_agent: 0, verification: 0 };
    for (const t of totals) {
      const cat = categoryOf(t.mast_mode);
      if (cat !== 'unknown') sum[cat] += t.total;
    }
    return sum;
  });
</script>

<div class="failure-modes-drawer">
  <header class="drawer-header">
    <h2>Failure modes (MAST)</h2>
    <p class="grounding">
      arXiv:2503.13657 — Cemri et al. NeurIPS 2025. 14-variant taxonomy. Wire-stable tags.
    </p>
  </header>

  {#if loading}
    <p class="status">Loading…</p>
  {:else if error}
    <p class="status error">Error: {error}</p>
  {:else if grandTotal === 0}
    <p class="status">No MAST-tagged failure rows in window.</p>
  {:else}
    <div class="category-bands">
      <button
        class:active={categoryFilter === 'all'}
        onclick={() => (categoryFilter = 'all')}
      >
        All ({grandTotal})
      </button>
      <button
        class:active={categoryFilter === 'specification'}
        onclick={() => (categoryFilter = 'specification')}
      >
        Specification ({categorySum.specification})
      </button>
      <button
        class:active={categoryFilter === 'inter_agent'}
        onclick={() => (categoryFilter = 'inter_agent')}
      >
        Inter-Agent ({categorySum.inter_agent})
      </button>
      <button
        class:active={categoryFilter === 'verification'}
        onclick={() => (categoryFilter = 'verification')}
      >
        Verification ({categorySum.verification})
      </button>
    </div>

    <table class="totals-grid">
      <thead>
        <tr>
          <th>Mode</th>
          <th>Category</th>
          <th>Count</th>
        </tr>
      </thead>
      <tbody>
        {#each filteredTotals as t (t.mast_mode)}
          <tr>
            <td class="mode-tag">{t.mast_mode}</td>
            <td class="cat-cell cat-{categoryOf(t.mast_mode)}">{categoryOf(t.mast_mode)}</td>
            <td class="count">{t.total}</td>
          </tr>
        {/each}
      </tbody>
    </table>

    <details>
      <summary>Per-(mode, role) breakdown ({rows.length} rows)</summary>
      <table class="rows-grid">
        <thead>
          <tr>
            <th>Mode</th>
            <th>Agent role</th>
            <th>Count</th>
          </tr>
        </thead>
        <tbody>
          {#each rows as r (r.mast_mode + r.agent_role)}
            <tr>
              <td class="mode-tag">{r.mast_mode}</td>
              <td>{r.agent_role}</td>
              <td class="count">{r.count}</td>
            </tr>
          {/each}
        </tbody>
      </table>
    </details>
  {/if}
</div>

<style>
  .failure-modes-drawer {
    display: flex;
    flex-direction: column;
    gap: 0.75rem;
    padding: 0.75rem;
    font-family: var(--font-mono, ui-monospace, monospace);
    font-size: 12px;
  }
  .drawer-header h2 {
    margin: 0 0 0.25rem 0;
    font-size: 14px;
    font-weight: 600;
  }
  .grounding {
    margin: 0;
    color: var(--muted-fg, #888);
    font-size: 11px;
  }
  .status {
    color: var(--muted-fg, #888);
  }
  .status.error {
    color: var(--err-fg, #c33);
  }
  .category-bands {
    display: flex;
    gap: 0.5rem;
    flex-wrap: wrap;
  }
  .category-bands button {
    padding: 0.25rem 0.5rem;
    border: 1px solid var(--border, #444);
    background: var(--bg, #1a1a1a);
    color: var(--fg, #ddd);
    cursor: pointer;
    font-family: inherit;
    font-size: 11px;
  }
  .category-bands button.active {
    background: var(--accent-bg, #2a4a6a);
    border-color: var(--accent-fg, #5c8cb8);
  }
  table {
    width: 100%;
    border-collapse: collapse;
  }
  th,
  td {
    text-align: left;
    padding: 0.2rem 0.4rem;
    border-bottom: 1px solid var(--border, #333);
  }
  th {
    font-weight: 600;
    color: var(--muted-fg, #888);
  }
  .mode-tag {
    font-family: var(--font-mono, ui-monospace, monospace);
    font-size: 11px;
  }
  .count {
    text-align: right;
    font-variant-numeric: tabular-nums;
  }
  .cat-cell {
    font-size: 10px;
    text-transform: uppercase;
  }
  .cat-specification {
    color: #d4a017;
  }
  .cat-inter_agent {
    color: #5c8cb8;
  }
  .cat-verification {
    color: #c33;
  }
  .cat-unknown {
    color: #888;
  }
  details summary {
    cursor: pointer;
    color: var(--muted-fg, #888);
    margin-top: 0.5rem;
  }
</style>
