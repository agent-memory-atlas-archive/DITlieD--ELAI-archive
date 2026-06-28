<script lang="ts">
  import { onMount } from 'svelte';
  import { session } from '../state/session.svelte';
  import { fsm } from '../state/fsm.svelte';
  import { reindex } from '../state/reindex.svelte';
  import { VRAM_GAUGES } from '../api/mock-data';
  import { get_context_budget } from '../api/stub';

  const BRANCH = 'main';
  // B-9: handoff threshold gauge — poll get_context_budget every 15s.
  // Amber at 70%, red at 85% (SESSION_HANDOFF_THRESHOLD).
  let contextPct = $state(0);

  async function refreshContextBudget() {
    try {
      const budget = await get_context_budget();
      if (budget && budget.total_cap > 0) {
        contextPct = Math.round((budget.total_used / budget.total_cap) * 100);
      }
    } catch {
      // non-fatal — keep last value
    }
  }

  // Reindex feedback comes from the reindex store mounted at App root —
  // the listener registers before any reindex_now() call so the 'started'
  // event isn't lost on first project open.
  let reindexStatus = $derived(reindex.status);
  let reindexCounts = $derived(reindex.counts);
  let reindexError = $derived(reindex.error);

  function fmtCount(n: number): string {
    if (n >= 10_000) return `${Math.round(n / 1000)}k`;
    if (n >= 1_000) return `${(n / 1000).toFixed(1)}k`;
    return `${n}`;
  }

  onMount(() => {
    refreshContextBudget();
    const id = setInterval(refreshContextBudget, 15_000);
    return () => clearInterval(id);
  });

  const STATE_TOKEN_MAP: Record<string, string> = {
    IDLE:             'var(--state-idle)',
    PLANNING:         'var(--state-planning)',
    FEASIBILITY_PROBE: 'var(--state-feasibility)',
    TESTING_FIRST:    'var(--state-testing)',
    EXECUTING:        'var(--state-executing)',
    AUDITING:         'var(--state-auditing)',
    MEMORIZING:       'var(--state-memorizing)',
    COMPLETE:         'var(--state-complete)',
    CONFIGURING:      'var(--state-configuring)',
    RESEARCHING:      'var(--state-researching)',
  };

  let project = $derived(session.currentProject?.name ?? 'no project');
  let fsmState = $derived(fsm.state);
  let mode = $derived(session.mode);
  let gateMode = $derived(session.gateMode);
  let model = $derived(session.model);
  let sessionId = $derived(session.sessionId);

  let vramPct = $derived(
    VRAM_GAUGES.length > 0
      ? Math.round(VRAM_GAUGES[0].used_mb / VRAM_GAUGES[0].total_mb * 100)
      : 0
  );

  let stateColor = $derived(STATE_TOKEN_MAP[fsmState] ?? 'var(--text-secondary)');
  let showVram = $derived(vramPct > 70);
  let showCtx = $derived(contextPct >= 70);
  let isExecuting = $derived(fsmState === 'EXECUTING');

  function copySessionId() {
    navigator.clipboard.writeText(sessionId).then(() => {
      console.info('[stub] session-id copied:', sessionId);
    }).catch((err) => {
      console.warn('[stub] clipboard copy failed:', err);
    });
  }
</script>

<div class="status-bar" class:status-bar--executing={isExecuting}>
  <div class="cluster">
    <!-- Always visible (essentials) -->
    <span class="mono project">{project}</span>
    <span class="sep" aria-hidden="true">·</span>
    <span class="mono state-badge" style="color: {stateColor}" aria-label="FSM state: {fsmState}">{fsmState}</span>
    <span class="sep" aria-hidden="true">·</span>
    <span class="mode-indicator" aria-label="Mode: {mode}">
      <span class="mode-dot mode-dot--{mode.toLowerCase()}" aria-hidden="true"></span>
      <span class="mono mode-label">{mode}</span>
    </span>
    <!-- patch-4G.1: gate-mode badge promoted from .verbose so the active
         autonomy mode is visible without hover. Same lower-case styling as
         mode-label for visual parity. -->
    <span class="sep" aria-hidden="true">·</span>
    <span class="gate-indicator" aria-label="Gate: {gateMode}">
      <span class="mono gate-label gate-label--{gateMode.toLowerCase()}">gate {gateMode.toLowerCase()}</span>
    </span>

    <!-- Reindex feedback (Item 3 — fires for ~indexing duration, fades 3s after done) -->
    {#if reindexStatus !== 'idle'}
      <span class="sep" aria-hidden="true">·</span>
      <span
        class="mono reindex-label"
        class:reindex--running={reindexStatus === 'running'}
        class:reindex--done={reindexStatus === 'done'}
        class:reindex--error={reindexStatus === 'error'}
        aria-live="polite"
        aria-label={
          reindexStatus === 'running' ? 'Reindexing project' :
          reindexStatus === 'done'    ? `Reindex complete: ${reindexCounts?.dkg_files ?? 0} files, ${reindexCounts?.dkg_nodes ?? 0} nodes` :
          reindexError ? `Reindex error: ${reindexError}` : 'Reindex error'
        }
      >
        {#if reindexStatus === 'running'}
          indexing...
        {:else if reindexStatus === 'done' && reindexCounts}
          indexed: {fmtCount(reindexCounts.dkg_files)} files / {fmtCount(reindexCounts.dkg_nodes)} nodes{#if reindexCounts.profile_loaded === false} (profile failed){/if}
        {:else if reindexStatus === 'error'}
          index error
        {/if}
      </span>
    {/if}

    <!-- Verbose group: hidden by default, fade in on hover/focus -->
    <span class="verbose">
      <span class="sep" aria-hidden="true">·</span>
      <span class="mono branch">{BRANCH}</span>
      <span class="sep" aria-hidden="true">·</span>
      <span class="mono model-label">{model}</span>
      {#if showVram}
        <span class="sep" aria-hidden="true">·</span>
        <span
          class="mono vram-label"
          class:vram-warn={vramPct > 85}
          class:vram-fail={vramPct > 95}
          aria-label="VRAM {vramPct}%"
        >vram {vramPct}%</span>
      {/if}
      {#if showCtx}
        <span class="sep" aria-hidden="true">·</span>
        <span
          class="mono ctx-label"
          class:ctx-warn={contextPct >= 85}
          aria-label="Context {contextPct}% — handoff threshold 85%"
        >ctx {contextPct}%</span>
      {/if}
      <span class="sep" aria-hidden="true">·</span>
      <button
        class="session-id"
        onclick={copySessionId}
        title="Click to copy session ID"
        aria-label="Session ID: {sessionId} — click to copy"
      >{sessionId}</button>
    </span>
  </div>

  {#if isExecuting}
    <div class="executing-line anim-breath" aria-hidden="true"></div>
  {/if}
</div>

<style>
  .status-bar {
    position: relative;
    display: flex;
    align-items: center;
    justify-content: flex-start;
    width: 100%;
    height: 22px;
    background: var(--bg-void);
    border-top: 1px solid var(--border-subtle);
    padding: 0 10px;
    box-sizing: border-box;
    overflow: hidden;
  }

  .cluster {
    display: flex;
    align-items: center;
    gap: 5px;
    flex-shrink: 1;
    overflow: hidden;
  }

  /* Verbose status items hidden by default; fade in on bar hover or focus
     anywhere inside it. Layout doesn't shift because the hidden span has
     zero width when display: none is avoided via opacity-only hide. */
  .verbose {
    display: flex;
    align-items: center;
    gap: 5px;
    opacity: 0;
    pointer-events: none;
    transition: opacity 220ms var(--ease-std);
    overflow: hidden;
  }
  .status-bar:hover .verbose,
  .status-bar:focus-within .verbose {
    opacity: 1;
    pointer-events: auto;
  }

  .mono {
    font-family: var(--font-mono);
    font-size: 10.5px;
    line-height: 1;
    white-space: nowrap;
  }

  .project { color: var(--text-secondary); min-width: 0; overflow: hidden; text-overflow: ellipsis; }
  .branch  { color: var(--text-ghost); }

  .sep {
    color: var(--text-ghost);
    font-size: 10px;
    opacity: 0.5;
    user-select: none;
  }

  .state-badge {
    font-size: 10.5px;
    letter-spacing: 0.04em;
    text-transform: lowercase;
    font-weight: 400;
  }

  .mode-indicator { display: flex; align-items: center; gap: 4px; }

  .mode-dot {
    width: 4px;
    height: 4px;
    border-radius: 50%;
    flex-shrink: 0;
  }

  .mode-dot--local  { background: var(--teal); }
  .mode-dot--cloud  { background: var(--steel); }
  .mode-dot--hybrid { background: conic-gradient(var(--teal) 0deg 180deg, var(--steel) 180deg 360deg); }

  .mode-label {
    color: var(--text-ghost);
    font-size: 10px;
    letter-spacing: 0.04em;
    text-transform: lowercase;
  }

  .gate-indicator { display: flex; align-items: center; }

  .gate-label {
    color: var(--text-ghost);
    font-size: 10px;
    letter-spacing: 0.04em;
    text-transform: lowercase;
  }
  /* patch-4G.1: AUTONOMOUS gets a faint teal tint so the user notices
     when the gate is in auto-advance mode without resorting to hover. */
  .gate-label--autonomous { color: var(--teal); }
  .gate-label--interactive { color: var(--text-ghost); }

  .model-label { color: var(--text-ghost); min-width: 0; overflow: hidden; text-overflow: ellipsis; }

  .vram-label {
    color: var(--text-secondary);
  }

  .vram-warn {
    color: var(--warn);
  }

  .vram-fail {
    color: var(--fail);
  }

  .ctx-label {
    color: var(--warn);
  }

  .ctx-warn {
    color: var(--fail);
  }

  .session-id {
    font-family: var(--font-mono);
    font-size: 10px;
    color: var(--text-ghost);
    background: none;
    border: none;
    cursor: pointer;
    padding: 0;
    outline: none;
    transition: color 120ms var(--ease-std);
  }

  .session-id:hover {
    color: var(--text-secondary);
  }

  .session-id:focus-visible {
    box-shadow: var(--ring);
    border-radius: var(--r-xs);
  }

  .executing-line {
    position: absolute;
    bottom: 0;
    left: 0;
    width: 100%;
    height: 1px;
    background: var(--teal);
    pointer-events: none;
  }

  .reindex-label {
    font-size: 10.5px;
    letter-spacing: 0.04em;
    text-transform: lowercase;
    transition: opacity 320ms var(--ease-std);
  }

  .reindex--running {
    color: var(--teal);
    animation: breath 2s ease-in-out infinite;
  }

  .reindex--done {
    color: var(--teal);
  }

  .reindex--error {
    color: var(--fail);
  }
</style>
