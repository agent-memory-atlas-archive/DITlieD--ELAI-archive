<script lang="ts">
  // E-6 — MAST-grounded multi-agent failure-mode tracker panel.
  //
  // Renders per-MAST-mode counts in 3 grouped columns (Specification /
  // InterAgent / Verification) with one row per (mode, agent_role) cell.
  // Live via SSB subscription on AuditMastTagged + IsoBudgetMeasured.
  //
  // Grounding: arXiv:2503.13657 (Cemri et al. 2025, MAST). Original plan
  // cite 2406.12624 was an ID_COLLISION (Judging the Judges); substituted
  // per workflow/reasoning/E-6-decisions.md DECISION-1 + gotchas.md
  // E6_CITATION_2406_12624_ID_COLLISION.

  import { invoke } from "@tauri-apps/api/core";
  import { onMount, onDestroy } from "svelte";

  type MastBreakdownRow = {
    mast_mode: string;
    agent_role: string;
    count: number;
  };
  type MastModeTotal = {
    mast_mode: string;
    total: number;
  };
  type MastBreakdownResponse = {
    rows: MastBreakdownRow[];
    totals: MastModeTotal[];
    grand_total: number;
  };
  type IsoBudgetMeasuredDto = {
    task_fixture: string;
    budget_tokens: number;
    full_pipeline_pass_rate: number;
    single_agent_wait_pass_rate: number;
    insufficient_lift: boolean;
    runpod_pending: boolean;
    mast_mode_counts: { mast_mode: string; count: number }[];
  };

  // 14 MAST modes in canonical declaration order (FM-1.1 -> FM-3.3).
  // Mirrors orchestrator::mast::MastMode::ALL.
  const FC1_MODES = [
    "fm_1_1_disobey_task_spec",
    "fm_1_2_disobey_role_spec",
    "fm_1_3_step_repetition",
    "fm_1_4_loss_of_conversation_history",
    "fm_1_5_unaware_of_termination_conditions",
  ];
  const FC2_MODES = [
    "fm_2_1_conversation_reset",
    "fm_2_2_fail_to_ask_for_clarification",
    "fm_2_3_task_derailment",
    "fm_2_4_information_withholding",
    "fm_2_5_ignored_other_agent_input",
    "fm_2_6_reasoning_action_mismatch",
  ];
  const FC3_MODES = [
    "fm_3_1_premature_termination",
    "fm_3_2_no_or_incomplete_verification",
    "fm_3_3_incorrect_verification",
  ];

  let breakdown: MastBreakdownResponse | null = null;
  let isoBudget: IsoBudgetMeasuredDto[] = [];
  let loadError: string | null = null;
  let pollHandle: ReturnType<typeof setInterval> | null = null;

  async function refresh() {
    try {
      const [bd, ib] = await Promise.all([
        invoke<MastBreakdownResponse>("get_mast_breakdown", {
          limit: 500,
        }),
        invoke<IsoBudgetMeasuredDto[]>("get_iso_budget_recent", {
          limit: 20,
        }),
      ]);
      breakdown = bd;
      isoBudget = ib;
      loadError = null;
    } catch (e) {
      loadError = String(e);
    }
  }

  function totalForMode(mode: string): number {
    if (!breakdown) return 0;
    const t = breakdown.totals.find((x) => x.mast_mode === mode);
    return t ? t.total : 0;
  }

  function formatModeLabel(mode: string): string {
    // fm_1_1_disobey_task_spec -> "FM-1.1 disobey task spec"
    const m = mode.match(/^fm_(\d)_(\d)_(.+)$/);
    if (!m) return mode;
    return `FM-${m[1]}.${m[2]} ${m[3].replace(/_/g, " ")}`;
  }

  onMount(() => {
    refresh();
    // Poll every 5s for live updates. The Tauri command is cheap (single
    // SsbViewer scan capped at MAX_LIMIT=1000 rows).
    pollHandle = setInterval(() => {
      refresh();
    }, 5000);
  });

  onDestroy(() => {
    if (pollHandle) clearInterval(pollHandle);
  });
</script>

<section class="mast-failure-panel" aria-label="MAST multi-agent failure-mode tracker">
  <header class="panel-header">
    <h2>MAST Failure-Mode Tracker</h2>
    <p class="subtitle">
      14-mode taxonomy per arXiv:2503.13657 (Cemri et al. 2025).
      <span class="grand-total">
        {breakdown?.grand_total ?? 0} total BLOCK verdicts tagged
      </span>
    </p>
  </header>

  {#if loadError}
    <div class="error">Error loading MAST data: {loadError}</div>
  {/if}

  <div class="three-columns">
    <!-- FC1 Specification (41.77% of MAST corpus) -->
    <div class="category-column">
      <h3 class="category-header fc1">Specification (41.77%)</h3>
      {#each FC1_MODES as mode}
        <div class="mode-row" data-mode={mode}>
          <span class="mode-label">{formatModeLabel(mode)}</span>
          <span class="mode-count">{totalForMode(mode)}</span>
        </div>
      {/each}
    </div>

    <!-- FC2 Inter-Agent Misalignment (36.94%) -->
    <div class="category-column">
      <h3 class="category-header fc2">Inter-Agent (36.94%)</h3>
      {#each FC2_MODES as mode}
        <div class="mode-row" data-mode={mode}>
          <span class="mode-label">{formatModeLabel(mode)}</span>
          <span class="mode-count">{totalForMode(mode)}</span>
        </div>
      {/each}
    </div>

    <!-- FC3 Task Verification (21.30%) -->
    <div class="category-column">
      <h3 class="category-header fc3">Verification (21.30%)</h3>
      {#each FC3_MODES as mode}
        <div class="mode-row" data-mode={mode}>
          <span class="mode-label">{formatModeLabel(mode)}</span>
          <span class="mode-count">{totalForMode(mode)}</span>
        </div>
      {/each}
    </div>
  </div>

  <section class="iso-budget" aria-label="Iso-budget ablation results">
    <h3>Iso-budget ablation (full pipeline vs single-agent + Wait)</h3>
    {#if isoBudget.length === 0}
      <p class="muted">No iso-budget runs yet. Invoke via the CLI to populate.</p>
    {:else}
      <table>
        <thead>
          <tr>
            <th>Task fixture</th>
            <th>Budget (tokens)</th>
            <th>Full pipeline</th>
            <th>Single + Wait</th>
            <th>Lift</th>
            <th>Status</th>
          </tr>
        </thead>
        <tbody>
          {#each isoBudget as run}
            <tr>
              <td>{run.task_fixture}</td>
              <td>{run.budget_tokens}</td>
              <td>{(run.full_pipeline_pass_rate * 100).toFixed(1)}%</td>
              <td>{(run.single_agent_wait_pass_rate * 100).toFixed(1)}%</td>
              <td class:insufficient={run.insufficient_lift}>
                {run.insufficient_lift ? "insufficient" : "sufficient"}
              </td>
              <td>{run.runpod_pending ? "RUNPOD-PENDING" : "live"}</td>
            </tr>
          {/each}
        </tbody>
      </table>
    {/if}
  </section>
</section>

<style>
  .mast-failure-panel {
    padding: 1rem;
    border: 1px solid #2a2a2a;
    border-radius: 6px;
    background: #0e0e0e;
    color: #d4d4d4;
    font-family: ui-monospace, "SF Mono", monospace;
  }
  .panel-header h2 {
    margin: 0 0 0.25rem 0;
    font-size: 1.1rem;
    color: #e4e4e4;
  }
  .subtitle {
    margin: 0 0 1rem 0;
    font-size: 0.8rem;
    color: #888;
  }
  .grand-total {
    margin-left: 0.5rem;
    color: #c0c0c0;
  }
  .error {
    color: #ff8080;
    padding: 0.5rem;
    background: rgba(255, 0, 0, 0.06);
    border-radius: 4px;
    margin-bottom: 0.5rem;
  }
  .three-columns {
    display: grid;
    grid-template-columns: repeat(3, 1fr);
    gap: 1rem;
    margin-bottom: 1.5rem;
  }
  .category-column {
    padding: 0.5rem;
    background: #161616;
    border-radius: 4px;
  }
  .category-header {
    margin: 0 0 0.5rem 0;
    font-size: 0.9rem;
    font-weight: bold;
    padding-bottom: 0.25rem;
    border-bottom: 1px solid #333;
  }
  .category-header.fc1 {
    color: #ffd070;
  }
  .category-header.fc2 {
    color: #a0c8ff;
  }
  .category-header.fc3 {
    color: #ff9090;
  }
  .mode-row {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 0.2rem 0;
    font-size: 0.78rem;
  }
  .mode-label {
    color: #b0b0b0;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    margin-right: 0.5rem;
  }
  .mode-count {
    color: #f0f0f0;
    font-weight: bold;
    min-width: 2ch;
    text-align: right;
  }
  .iso-budget h3 {
    font-size: 0.9rem;
    margin: 0 0 0.5rem 0;
  }
  .iso-budget table {
    width: 100%;
    border-collapse: collapse;
    font-size: 0.78rem;
  }
  .iso-budget th,
  .iso-budget td {
    text-align: left;
    padding: 0.25rem 0.5rem;
    border-bottom: 1px solid #222;
  }
  .iso-budget th {
    color: #888;
    font-weight: normal;
  }
  .insufficient {
    color: #ffb070;
  }
  .muted {
    color: #666;
    font-size: 0.85rem;
  }
</style>
