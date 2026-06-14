<script lang="ts">
  // E-33 ActivityRibbon — persistent thin row at the top edge of WindowHost.
  // Compressed slice of ActivityWindow state that survives window navigation
  // (Kiro CLI "Live Unified Status" pattern, Fuselab 2026 transparency
  // principle). Distinct from StatusBar (which lives at the app bottom and
  // shows static session globals): the ribbon shows LIVE operational state.

  import { fsm } from '$lib/state/fsm.svelte';
  import { session } from '$lib/state/session.svelte';
  import { uxPrefs } from '$lib/state/ux_prefs.svelte';

  // Live FSM state — re-rendered on every fsm.svelte.ts poll (5s default).
  let fsmState = $derived(fsm.state);
  let agentSubstate = $derived(fsm.agentSubstate);
  // DESIGN §6: progress is an explicit fraction on a segmented bar, never a
  // lone percentage. The store exposes only a normalized 0..1 proportion (no
  // backend n/m step counts), so the denominator is the visible segment count
  // of the bar itself — the mission-control segmented form. filled/total IS
  // the fraction shown; no fabricated step data.
  const PROGRESS_SEGMENTS = 8;
  let progress01 = $derived(Math.max(0, Math.min(1, fsm.progress)));
  let segFilled = $derived(Math.round(progress01 * PROGRESS_SEGMENTS));
  let stepId = $derived(fsm.currentStepId);
  let mode = $derived(session.mode);
  let model = $derived(session.model);
  // E-33 task 1: SettingsWindow toggle ui.activity_ribbon.enabled. When
  // disabled the visual chrome collapses to 0 height (mount stays so the
  // a11y polite region still fires on first activity).
  let userEnabled = $derived(uxPrefs.activityRibbonEnabled);

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

  let stateColor = $derived(STATE_TOKEN_MAP[fsmState] ?? 'var(--read)');

  // Visually hide the ribbon when the FSM is fully idle and there's nothing
  // to report — avoids permanent dead-pixel chrome. The mount stays so the
  // a11y region "polite" announcement still fires on first activity.
  let visible = $derived(
    userEnabled && (fsmState !== 'IDLE' || agentSubstate !== '' || progress01 > 0),
  );
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
  {#if progress01 > 0}
    <span
      class="ribbon-cell ribbon-progress"
      title="step progress {segFilled}/{PROGRESS_SEGMENTS}"
    >
      <span class="ribbon-progress-track" aria-hidden="true">
        {#each { length: PROGRESS_SEGMENTS } as _, i (i)}
          <span class="ribbon-seg" class:is-filled={i < segFilled}></span>
        {/each}
      </span>
      <span class="ribbon-val">{segFilled}/{PROGRESS_SEGMENTS}</span>
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
    border-bottom: 1px solid var(--line);
    background: var(--bg);
    font-family: var(--font-mono);
    font-size: 10px;
    color: var(--read);
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
  /* §6 active glyph: a teal-family state dot with the single sanctioned
     pulse (the only moving thing). No glow — stars/glow are banned on work
     surfaces (§10); the hairline ring + pulse carry the live signal. */
  .ribbon-dot {
    width: 6px;
    height: 6px;
    border-radius: 50%;
    background: var(--state-color);
    animation: ribbon-pulse 1.6s var(--ease-std) infinite;
  }
  @keyframes ribbon-pulse {
    0%, 100% { opacity: 1; }
    50%      { opacity: 0.45; }
  }
  @media (prefers-reduced-motion: reduce) {
    .ribbon-dot {
      animation: none;
    }
  }
  .ribbon-state-label {
    text-transform: lowercase;
    letter-spacing: 0.04em;
  }
  .ribbon-arrow {
    color: var(--dim);
  }
  .ribbon-key {
    color: var(--dim);
  }
  .ribbon-val {
    color: var(--text);
  }
  /* §6 segmented progress bar: discrete cells, filled count == fraction. */
  .ribbon-progress-track {
    display: inline-flex;
    align-items: center;
    gap: 2px;
  }
  .ribbon-seg {
    width: 6px;
    height: 4px;
    background: var(--sink);
    border: 1px solid var(--line);
    border-radius: 1px;
    transition: background var(--dur-base) var(--ease-out);
  }
  .ribbon-seg.is-filled {
    background: var(--d-teal-deep);
    border-color: var(--d-teal-soft);
  }
  .ribbon-spacer {
    flex: 1 1 auto;
  }
  .ribbon-substate {
    color: var(--text);
  }
</style>
