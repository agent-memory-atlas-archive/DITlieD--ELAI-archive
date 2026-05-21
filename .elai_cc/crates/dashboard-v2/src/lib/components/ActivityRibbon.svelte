<script lang="ts">
  // E-33 ActivityRibbon — persistent thin row at the top edge of WindowHost.
  // Compressed slice of ActivityWindow state that survives window navigation
  // (Kiro CLI "Live Unified Status" pattern, Fuselab 2026 transparency
  // principle). Distinct from StatusBar (which lives at the app bottom and
  // shows static session globals): the ribbon shows LIVE operational state.

  import { fsm } from '$lib/state/fsm.svelte';
  import { session } from '$lib/state/session.svelte';

  // Live FSM state — re-rendered on every fsm.svelte.ts poll (5s default).
  let fsmState = $derived(fsm.state);
  let agentSubstate = $derived(fsm.agentSubstate);
  let progressPct = $derived(Math.round(fsm.progress * 100));
  let stepId = $derived(fsm.currentStepId);
  let mode = $derived(session.mode);
  let model = $derived(session.model);

  // FSM-state color mirrors StatusBar mapping. Kept in sync with tokens.css
  // FSM STATE COLORS block; do not introduce hex literals here.
  const STATE_TOKEN_MAP: Record<string, string> = {
    IDLE: 'var(--state-idle)',
    PLANNING: 'var(--state-planning)',
    FEASIBILITY_PROBE: 'var(--state-feasibility)',
    TESTING_FIRST: 'var(--state-testing)',
    EXECUTING: 'var(--state-executing)',
    AUDITING: 'var(--state-auditing)',
    MEMORIZING: 'var(--state-memorizing)',
    COMPLETE: 'var(--state-complete)',
    CONFIGURING: 'var(--state-configuring)',
    RESEARCHING: 'var(--state-researching)',
  };

  let stateColor = $derived(STATE_TOKEN_MAP[fsmState] ?? 'var(--text-secondary)');

  // Visually hide the ribbon when the FSM is fully idle and there's nothing
  // to report — avoids permanent dead-pixel chrome. The mount stays so the
  // a11y region "polite" announcement still fires on first activity.
  let visible = $derived(fsmState !== 'IDLE' || agentSubstate !== '' || progressPct > 0);
</script>

<aside
  class="activity-ribbon"
  class:is-hidden={!visible}
  role="status"
  aria-live="polite"
  aria-label="Agent activity"
  data-fsm-state={fsmState}
>
  <span class="ribbon-cell ribbon-state" style="--state-color: {stateColor};">
    <span class="ribbon-dot" aria-hidden="true"></span>
    <span class="ribbon-state-label">{fsmState.toLowerCase()}</span>
  </span>
  {#if agentSubstate}
    <span class="ribbon-cell ribbon-substate" title="agent substate">
      <span class="ribbon-arrow" aria-hidden="true">›</span>
      {agentSubstate}
    </span>
  {/if}
  {#if stepId}
    <span class="ribbon-cell ribbon-step" title="current step id">
      <span class="ribbon-key">step</span>
      <span class="ribbon-val">{stepId}</span>
    </span>
  {/if}
  {#if progressPct > 0}
    <span class="ribbon-cell ribbon-progress" title="step progress">
      <span class="ribbon-progress-track">
        <span class="ribbon-progress-fill" style="width: {progressPct}%;"></span>
      </span>
      <span class="ribbon-val">{progressPct}%</span>
    </span>
  {/if}
  <span class="ribbon-spacer"></span>
  <span class="ribbon-cell ribbon-mode" title="dispatch mode">
    <span class="ribbon-key">mode</span>
    <span class="ribbon-val">{mode.toLowerCase()}</span>
  </span>
  <span class="ribbon-cell ribbon-model" title="active model">
    <span class="ribbon-key">model</span>
    <span class="ribbon-val">{model}</span>
  </span>
</aside>

<style>
  .activity-ribbon {
    display: flex;
    align-items: center;
    gap: var(--s-3);
    padding: 4px var(--s-3);
    height: 22px;
    min-height: 22px;
    border-bottom: 1px solid var(--border-subtle);
    background: var(--bg-base);
    font-family: var(--font-mono);
    font-size: 10px;
    color: var(--text-secondary);
    user-select: none;
    flex: 0 0 auto;
  }
  .activity-ribbon.is-hidden {
    /* Keep mount for a11y region, collapse visual footprint. */
    height: 0;
    min-height: 0;
    padding: 0;
    border-bottom: 0;
    overflow: hidden;
  }
  .ribbon-cell {
    display: inline-flex;
    align-items: center;
    gap: 4px;
    white-space: nowrap;
  }
  .ribbon-state {
    color: var(--state-color);
    font-weight: 500;
  }
  .ribbon-dot {
    width: 6px;
    height: 6px;
    border-radius: 50%;
    background: var(--state-color);
    box-shadow: 0 0 4px var(--state-color);
  }
  @media (prefers-reduced-motion: reduce) {
    .ribbon-dot {
      box-shadow: none;
    }
  }
  .ribbon-state-label {
    text-transform: lowercase;
    letter-spacing: 0.04em;
  }
  .ribbon-arrow {
    color: var(--text-ghost);
  }
  .ribbon-key {
    color: var(--text-ghost);
  }
  .ribbon-val {
    color: var(--text-primary);
  }
  .ribbon-progress-track {
    width: 60px;
    height: 4px;
    background: var(--bg-input);
    border-radius: var(--r-pill);
    overflow: hidden;
  }
  .ribbon-progress-fill {
    display: block;
    height: 100%;
    background: var(--teal-bright);
    transition: width var(--dur-base) var(--ease-out);
  }
  .ribbon-spacer {
    flex: 1 1 auto;
  }
  .ribbon-substate {
    color: var(--text-primary);
  }
</style>
