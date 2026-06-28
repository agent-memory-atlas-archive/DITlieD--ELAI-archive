<script lang="ts">
  // E-23 — ContextStatePanel.
  //
  // Surfaces the live context-state warning the dashboard shows BEFORE
  // PreCompact truncates the conversation. Addresses the #1 complaint
  // from the 2026 developer-discourse report (Anthropic issue #13171:
  // "Context loss without warning breaks trust"). Polls
  // get_context_state every 5s and renders:
  //
  //   * a badge with token_budget_pct
  //       0..70   = green (text-secondary)
  //       70..85  = yellow (warning border)
  //       85+     = red (fail) and surfaces a "compact now" hint
  //
  //   * the active FSM state + substate
  //   * the prompt_injection_check_status row (from C-42 producer when
  //     it ships; defaults to "not_run")
  //   * the runtime_choice_badge (local / cloud_billed) derived from
  //     the B-37 runtime_fallback chain
  //
  // The 70% / 85% thresholds match
  // orchestrator::ssb::CONTEXT_BUDGET_YELLOW_PCT / RED_PCT (named
  // constants, not magic numbers — Anti-stale-training HARD RULE).

  import { onMount, onDestroy } from 'svelte';
  import { get_context_state } from '../api/stub';
  import type { ContextState } from '../api/types';

  interface Props {
    pollMs?: number;
  }
  const { pollMs = 5_000 }: Props = $props();

  // Named constants mirror the orchestrator ssb.rs values. If the
  // backend constants change, update both sites in lockstep.
  const YELLOW_THRESHOLD_PCT = 70;
  const RED_THRESHOLD_PCT = 85;

  let panelState = $state<ContextState>({
    fsm_state: 'IDLE',
    substate: null,
    token_budget_pct: 0,
    mcp_think_topics: [],
    last_precompact_thresholds: [],
    prompt_injection_check_status: 'not_run',
    runtime_choice_badge: '',
    current_role: null,
    current_runtime: null,
  });
  let loadError = $state<string | null>(null);
  let pollHandle: ReturnType<typeof setInterval> | null = null;

  async function reload() {
    try {
      const next = await get_context_state();
      panelState = next;
      loadError = null;
    } catch (e) {
      loadError = e instanceof Error ? e.message : String(e);
    }
  }

  const badgeKind = $derived.by(() => {
    const pct = panelState.token_budget_pct;
    if (pct >= RED_THRESHOLD_PCT) return 'red';
    if (pct >= YELLOW_THRESHOLD_PCT) return 'yellow';
    return 'green';
  });

  const runtimeLabel = $derived.by(() => {
    if (panelState.runtime_choice_badge === 'local') return 'local';
    if (panelState.runtime_choice_badge === 'cloud_billed') return 'cloud - billed';
    return '';
  });

  onMount(() => {
    void reload();
    pollHandle = setInterval(reload, pollMs);
  });

  onDestroy(() => {
    if (pollHandle !== null) {
      clearInterval(pollHandle);
      pollHandle = null;
    }
  });
</script>

<section
  class="context-state-panel"
  title="ContextStatePanel — Anthropic issue #13171 context-loss warning at 70 / 85 thresholds. Threshold constants live in orchestrator::ssb::CONTEXT_BUDGET_*_PCT."
>
  <header class="header mono">
    <span class="label-ghost">context state</span>
    <span class="badge badge--{badgeKind}">
      {panelState.token_budget_pct.toFixed(0)}%
    </span>
  </header>

  <div class="row mono">
    <span class="row-label">fsm</span>
    <span class="row-val">{panelState.fsm_state.toLowerCase()}{panelState.substate ? ' / ' + panelState.substate.toLowerCase() : ''}</span>
  </div>

  <div class="row mono">
    <span class="row-label">runtime</span>
    {#if runtimeLabel}
      <span class="row-val runtime-{panelState.runtime_choice_badge}">{runtimeLabel}</span>
    {:else}
      <span class="row-val row-val--ghost">not yet selected</span>
    {/if}
  </div>

  <div class="row mono">
    <span class="row-label">injection</span>
    <span class="row-val injection-{panelState.prompt_injection_check_status}">
      {panelState.prompt_injection_check_status}
    </span>
  </div>

  {#if badgeKind === 'red'}
    <div class="warning mono">
      context at {panelState.token_budget_pct.toFixed(0)}%. compact now or risk truncation.
    </div>
  {:else if badgeKind === 'yellow'}
    <div class="advisory mono">
      context approaching {RED_THRESHOLD_PCT}%. consider /continue to checkpoint.
    </div>
  {/if}

  {#if loadError}
    <div class="error mono">load error: {loadError}</div>
  {/if}
</section>

<style>
  .context-state-panel {
    display: flex;
    flex-direction: column;
    gap: 4px;
    padding: 8px 10px;
    border: 1px solid var(--border-default);
    border-radius: var(--r-sm);
    background: var(--bg-surface);
  }

  .header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 8px;
  }

  .label-ghost {
    font-size: 10px;
    color: var(--text-ghost);
    text-transform: uppercase;
    letter-spacing: 0.08em;
  }

  .badge {
    display: inline-flex;
    align-items: center;
    padding: 2px 8px;
    border-radius: var(--r-pill);
    font-size: 10px;
    font-weight: 600;
    border: 1px solid transparent;
  }

  .badge--green {
    color: var(--text-secondary);
    border-color: var(--border-subtle);
  }

  .badge--yellow {
    color: #d4a017;
    border-color: rgba(212, 160, 23, 0.5);
    background: rgba(212, 160, 23, 0.08);
  }

  .badge--red {
    color: var(--fail);
    border-color: rgba(176, 69, 69, 0.5);
    background: rgba(176, 69, 69, 0.08);
  }

  .row {
    display: flex;
    align-items: baseline;
    justify-content: space-between;
    gap: 8px;
    font-size: 10px;
  }

  .row-label {
    color: var(--text-ghost);
    text-transform: lowercase;
  }

  .row-val {
    color: var(--text-secondary);
  }

  .row-val--ghost {
    color: var(--text-ghost);
    font-style: italic;
  }

  .runtime-local {
    color: var(--teal);
  }

  .runtime-cloud_billed {
    color: #d4a017;
  }

  .injection-passed {
    color: var(--teal);
  }

  .injection-blocked {
    color: var(--fail);
  }

  .injection-not_run {
    color: var(--text-ghost);
  }

  .warning {
    font-size: 10px;
    color: var(--fail);
    padding-top: 4px;
    border-top: 1px solid rgba(176, 69, 69, 0.2);
    margin-top: 4px;
  }

  .advisory {
    font-size: 10px;
    color: #d4a017;
    padding-top: 4px;
    border-top: 1px solid rgba(212, 160, 23, 0.2);
    margin-top: 4px;
  }

  .error {
    font-size: 10px;
    color: var(--fail);
    font-style: italic;
  }

  .mono {
    font-family: var(--font-mono);
  }
</style>
