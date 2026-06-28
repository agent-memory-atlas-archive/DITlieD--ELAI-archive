<!-- E-8 OptimizerRunPanel.svelte: live progress of an in-flight prompt -->
<!-- optimization run. Reads the prompt_optimization_completed SSB stream -->
<!-- via the get_optimizer_run Tauri command. Surfaces the 7-arm           -->
<!-- OptimizerPhase discriminator (dspy_init / opro_step / rank_prompt_vote -->
<!-- / ape_proposal / gepa_reflect / archive_promote / archive_retire) +   -->
<!-- val_lift_pct + held_out_lift_pct + overfit-risk red flag.            -->
<!-- Paper grounding: GEPA arXiv:2507.19457 (Pareto frontier surface) +    -->
<!-- LPO arXiv:2504.20355 (frozen-block enforcement Rust-side) +           -->
<!-- arXiv:2405.10276 cloud-only reflection guard.                         -->
<!-- See workflow/reasoning/E-8-decisions.md.                              -->
<script lang="ts">
  import { onMount, onDestroy } from "svelte";
  import { invoke } from "@tauri-apps/api/core";

  interface OptimizerRunRowDto {
    seq: number;
    ts: number;
    phase: string;
    iteration_index: number;
    val_lift_pct: number | null;
    held_out_lift_pct: number | null;
  }

  interface OptimizerRunStateDto {
    rows: OptimizerRunRowDto[];
    latest_phase: string | null;
    latest_val_lift_pct: number | null;
    latest_held_out_lift_pct: number | null;
    overfit_risk: boolean;
    reflection_lm_too_small: boolean;
  }

  let runState = $state<OptimizerRunStateDto | null>(null);
  let loadError = $state<string | null>(null);
  let lastRefreshTs = $state<number | null>(null);
  let pollHandle: number | null = null;

  async function refresh() {
    try {
      const s = await invoke<OptimizerRunStateDto>("get_optimizer_run", {
        n: 50,
      });
      runState = s;
      loadError = null;
      lastRefreshTs = Date.now();
    } catch (e) {
      loadError = String(e);
    }
  }

  onMount(() => {
    refresh();
    pollHandle = window.setInterval(refresh, 5000);
  });

  onDestroy(() => {
    if (pollHandle !== null) {
      window.clearInterval(pollHandle);
      pollHandle = null;
    }
  });

  function formatLift(v: number | null): string {
    if (v === null || v === undefined) return "-";
    return `${v.toFixed(1)}pp`;
  }
</script>

<div class="optimizer-run-panel">
  {#if loadError}
    <div class="error-banner">Optimizer snapshot unavailable: {loadError}</div>
  {:else if runState === null}
    <div class="empty">Loading optimizer run...</div>
  {:else if runState.rows.length === 0}
    <div class="empty">
      No optimization runs recorded yet. The prompt_optimization_completed
      SSB stream is empty (wire-dark unless
      ELAI_PROMPT_OPTIMIZER_ENABLE=1 + cfg darwinian-evolve).
    </div>
  {:else}
    <div class="status-row">
      <span class="label">Latest phase:</span>
      <code>{runState.latest_phase ?? "-"}</code>
    </div>
    <div class="status-row">
      <span class="label">val_lift_pct:</span>
      <code>{formatLift(runState.latest_val_lift_pct)}</code>
    </div>
    <div class="status-row">
      <span class="label">held_out_lift_pct:</span>
      <code>{formatLift(runState.latest_held_out_lift_pct)}</code>
    </div>
    {#if runState.overfit_risk}
      <div class="red-flag">
        OVERFIT RISK: held-out lift is more than 2pp below val lift
        (DSPy 20/80 split / arXiv:2310.03714 enforcement).
      </div>
    {/if}
    {#if runState.reflection_lm_too_small}
      <div class="red-flag">
        Reflection-LM too small: local Qwen 7B class is REJECTED per
        arXiv:2405.10276. Use cloud Sonnet 4.6 / GPT class only.
      </div>
    {/if}
    <table class="rows">
      <thead>
        <tr>
          <th>seq</th>
          <th>iter</th>
          <th>phase</th>
          <th>val_lift</th>
          <th>held_out</th>
        </tr>
      </thead>
      <tbody>
        {#each runState.rows as r}
          <tr>
            <td>{r.seq}</td>
            <td>{r.iteration_index}</td>
            <td><code>{r.phase}</code></td>
            <td>{formatLift(r.val_lift_pct)}</td>
            <td>{formatLift(r.held_out_lift_pct)}</td>
          </tr>
        {/each}
      </tbody>
    </table>
    <div class="footer">
      {runState.rows.length} row(s)
      {#if lastRefreshTs}
        · refreshed {new Date(lastRefreshTs).toLocaleTimeString()}
      {/if}
    </div>
  {/if}
</div>

<style>
  .optimizer-run-panel {
    font-size: 12px;
    padding: 8px 0;
  }
  .error-banner {
    color: #d05050;
    font-size: 10px;
    padding: 4px 0;
  }
  .empty {
    color: #888;
    font-style: italic;
    padding: 6px 0;
  }
  .status-row {
    display: flex;
    gap: 8px;
    padding: 2px 0;
  }
  .status-row .label {
    color: #888;
  }
  .red-flag {
    color: #d05050;
    font-weight: bold;
    padding: 4px 0;
    border-left: 3px solid #d05050;
    padding-left: 6px;
    margin: 4px 0;
  }
  table.rows {
    width: 100%;
    border-collapse: collapse;
    margin-top: 6px;
  }
  table.rows th,
  table.rows td {
    text-align: left;
    padding: 2px 6px;
    border-bottom: 1px solid #2a2a2a;
    font-size: 10px;
  }
  .footer {
    margin-top: 6px;
    font-size: 10px;
    color: #888;
  }
  code {
    background: #1a1a1a;
    padding: 1px 4px;
    border-radius: 2px;
    font-size: 10px;
  }
</style>
