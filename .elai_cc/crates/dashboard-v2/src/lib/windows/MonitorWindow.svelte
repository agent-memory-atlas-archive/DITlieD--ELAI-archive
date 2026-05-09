<script lang="ts">
  import { Empty } from '$lib/ui';
  import { tooltip } from '$lib/ui/tooltip';
  import * as stub from '$lib/api/stub';
  import type {
    FsmSnapshot,
    SystemStatus,
    InterventionRequest,
    SupervisionEvent,
    AutoAdvanceStats,
  } from '$lib/api/types';
  import { workspace } from '$lib/state/workspace.svelte';
  import { session } from '$lib/state/session.svelte';

  // ------------------------------------------------------------------
  // Phase dot model — derived from FSM history when live, static stub otherwise
  // ------------------------------------------------------------------
  type DotStatus = 'done' | 'active' | 'skipped' | 'pending' | 'intervention';

  interface PhaseDot {
    num: number;
    label: string;
    steps: number;
    status: DotStatus;
  }

  function dotChar(status: DotStatus): string {
    switch (status) {
      case 'done':         return '.';
      case 'active':       return '*';
      case 'skipped':      return 'o';
      case 'pending':      return '-';
      case 'intervention': return '!';
    }
  }

  // ------------------------------------------------------------------
  // Live Tauri state — polled every 5s; graceful empty fallback
  // ------------------------------------------------------------------
  let fsmSnap = $state<FsmSnapshot | null>(null);
  let sysStatus = $state<SystemStatus | null>(null);
  let supervisionLog = $state<SupervisionEvent[]>([]);
  let autoStats = $state<AutoAdvanceStats | null>(null);
  let loadError = $state<string | null>(null);

  // Derived from live data, fallback to empty/defaults
  let isRunning = $derived(
    fsmSnap !== null &&
    (fsmSnap.state === 'EXECUTING' || fsmSnap.state === 'PLANNING' || fsmSnap.state === 'AUDITING')
  );
  let progress = $derived(fsmSnap?.progress ?? 0);
  let taskName = $derived(fsmSnap?.agent_substate ? fsmSnap.agent_substate : (fsmSnap?.current_step_id ?? '—'));

  // Status snapshot from SystemStatus
  let statusSnap = $derived({
    currently: fsmSnap?.current_step_id ?? '—',
    health: sysStatus?.sidecar.loaded ? `${sysStatus.sidecar.tokens_per_sec} tok/s` : 'sidecar offline',
    model: sysStatus?.sidecar.model ?? '—',
  });

  // Elapsed — start_ts from FSM history first IDLE->PLANNING transition
  let startedAt = $derived(
    fsmSnap?.history?.find((h) => h.trigger === 'submit_task')?.ts ?? Date.now()
  );
  function fmtElapsed(startMs: number): string {
    const elapsed = Date.now() - startMs;
    const mins = Math.floor(elapsed / 60_000);
    const hrs = Math.floor(mins / 60);
    if (hrs > 0) return `running for ${hrs}h ${mins % 60}m`;
    return `running for ${mins} min`;
  }

  // Phase dots — stub 5-phase model (real phase data requires plan-level SSB query, deferred)
  let PHASES: PhaseDot[] = $derived(
    fsmSnap === null
      ? [
          { num: 1, label: 'Plan', steps: 0, status: 'pending' },
          { num: 2, label: 'Feasibility', steps: 0, status: 'pending' },
          { num: 3, label: 'Execute', steps: 0, status: 'pending' },
          { num: 4, label: 'Audit', steps: 0, status: 'pending' },
          { num: 5, label: 'Memorize', steps: 0, status: 'pending' },
        ]
      : (() => {
          const s = fsmSnap.state;
          const stateOrder = ['PLANNING', 'FEASIBILITY_PROBE', 'TESTING_FIRST', 'EXECUTING', 'AUDITING', 'MEMORIZING', 'COMPLETE'];
          const currentIdx = stateOrder.indexOf(s);
          const phases: PhaseDot[] = [
            { num: 1, label: 'Plan', steps: 0, status: 'pending' },
            { num: 2, label: 'Feasibility', steps: 0, status: 'pending' },
            { num: 3, label: 'Execute', steps: 0, status: 'pending' },
            { num: 4, label: 'Audit', steps: 0, status: 'pending' },
            { num: 5, label: 'Memorize', steps: 0, status: 'pending' },
          ];
          const map: Record<string, number> = { PLANNING: 0, FEASIBILITY_PROBE: 1, TESTING_FIRST: 2, EXECUTING: 2, AUDITING: 3, MEMORIZING: 4, COMPLETE: 4 };
          for (const [state, idx] of Object.entries(map)) {
            const sIdx = stateOrder.indexOf(state);
            if (sIdx < currentIdx) phases[idx].status = 'done';
          }
          const activeIdx = map[s];
          if (activeIdx !== undefined) phases[activeIdx].status = 'active';
          return phases;
        })()
  );

  // ------------------------------------------------------------------
  // Interventions — from SystemStatus pending_interventions or empty
  // ------------------------------------------------------------------
  // SystemStatus doesn't expose pending_interventions directly; use local state
  // wired from polled data. B-36 will confirm the SSB writer path.
  let interventions = $state<InterventionRequest[]>([]);
  let interventionBusy = $state<Record<string, boolean>>({});

  async function handleIntervention(id: string, action: 'approve' | 'reject') {
    if (interventionBusy[id]) return;
    interventionBusy = { ...interventionBusy, [id]: true };
    try {
      await stub.submit_intervention_response(id, action);
      interventions = interventions.filter((i) => i.id !== id);
    } finally {
      interventionBusy = { ...interventionBusy, [id]: false };
    }
  }

  async function takeover(id: string) {
    stub.logStub('monitor.takeover', { id }); // telemetry trace
    stub.submit_intervention_response(id, 'takeover').catch((err) => console.warn('[monitor] takeover failed', err));
    interventions = interventions.filter((i) => i.id !== id);
    workspace.openTab('primary', 'command');
  }

  // ------------------------------------------------------------------
  // Timeline — adapted from SupervisionEvent (proxy for activity, B-36 gap)
  // SupervisionEvent: { ts, agent, event, restart_count, reason? }
  // Displayed as timeline rows with appropriate tones.
  // ------------------------------------------------------------------
  let TIMELINE = $derived(
    supervisionLog.slice().reverse().slice(0, 15)
  );

  function fmtTime(ts: number): string {
    const d = new Date(ts);
    const hh = String(d.getUTCHours()).padStart(2, '0');
    const mm = String(d.getUTCMinutes()).padStart(2, '0');
    return `${hh}:${mm}`;
  }

  function supervisionLabel(entry: SupervisionEvent): string {
    const base = `${entry.agent}  ${entry.event.toLowerCase()}`;
    if (entry.reason) return `${base}  ${entry.reason.slice(0, 50)}${entry.reason.length > 50 ? '…' : ''}`;
    if (entry.restart_count > 0) return `${base}  restart #${entry.restart_count}`;
    return base;
  }

  function supervisionTone(entry: SupervisionEvent): 'pass' | 'fail' | 'warn' | 'default' {
    if (entry.event === 'DIED') return 'fail';
    if (entry.event === 'RESTARTED') return 'warn';
    if (entry.event === 'HEALTHY' || entry.event === 'STARTED') return 'pass';
    return 'default';
  }

  function openActivity(entryId: string) {
    stub.logStub('monitor.openActivity', { entryId });
    workspace.openTab('primary', 'activity');
  }

  // ------------------------------------------------------------------
  // Polling — every 5s; cleans up on unmount
  // ------------------------------------------------------------------
  $effect(() => {
    let alive = true;

    async function poll() {
      try {
        const [fsm, sys, log, stats] = await Promise.all([
          stub.get_fsm_state(),
          stub.get_system_status(),
          stub.get_supervision_log(),
          stub.get_auto_advance_stats(),
        ]);
        if (!alive) return;
        fsmSnap = fsm;
        sysStatus = sys;
        supervisionLog = log;
        autoStats = stats;
        loadError = null;
      } catch (e) {
        if (!alive) return;
        loadError = String(e);
      }
    }

    poll();
    const interval = setInterval(poll, 5_000);

    return () => {
      alive = false;
      clearInterval(interval);
    };
  });
</script>

<div class="monitor">
  <!-- Section 6: AutoAdvanceStats — always visible (historical totals shown even when idle).
       Phase 4G §4G.6 requires the stats panel to render outside the running gate
       so users can see the cumulative auto-advance health between tasks.
       patch-4G.1: header now mirrors the live gate mode so users see whether
       the gate is currently auto-advancing or asking humans. -->
  <section class="monitor-section auto-advance" aria-label="Auto-advance stats">
    <div class="auto-advance__header">
      <span class="auto-advance__label mono">auto-advance</span>
      <span
        class="auto-advance__mode mono auto-advance__mode--{session.gateMode.toLowerCase()}"
        aria-label="Active gate mode: {session.gateMode}"
      >gate {session.gateMode.toLowerCase()}</span>
    </div>
    <div class="auto-advance__row">
      <span class="auto-advance__metric mono">
        <span class="auto-advance__num">{autoStats?.auto_advanced ?? 0}</span>
        <span class="auto-advance__sep">/</span>
        <span class="auto-advance__num">{autoStats?.total_steps ?? 0}</span>
        <span class="auto-advance__unit">steps</span>
      </span>
      <span class="auto-advance__metric mono" title="Auto-advance rate (overall)">
        <span class="auto-advance__pct">{Math.round((autoStats?.auto_advance_rate ?? 0) * 100)}%</span>
        <span class="auto-advance__unit">overall</span>
      </span>
      <span class="auto-advance__metric mono" title="Auto-advance rate over the trailing 24 hours">
        <span class="auto-advance__pct">{Math.round((autoStats?.last_24h_rate ?? 0) * 100)}%</span>
        <span class="auto-advance__unit">24h</span>
      </span>
      {#if (autoStats?.intervention_required ?? 0) > 0}
        <span class="auto-advance__metric auto-advance__metric--warn mono" title="Pending interventions">
          <span class="auto-advance__num">{autoStats?.intervention_required ?? 0}</span>
          <span class="auto-advance__unit">human</span>
        </span>
      {/if}
    </div>
  </section>

  {#if !isRunning}
    <div class="monitor__empty">
      <Empty
        title="no autonomous task running"
        subtitle="the engine is idle"
      />
    </div>
  {:else}
    <div class="monitor__inner">

      <!-- Section 1: Identity — task line + elapsed ghost text -->
      <section class="monitor-section identity">
        <span class="identity-task">{taskName}</span>
        <span class="identity-elapsed mono">{fmtElapsed(startedAt)}</span>
      </section>

      <!-- Section 2: Progress — thin teal line, no fill bg, no rounded corners -->
      <section class="monitor-section progress">
        <div
          class="progress-line"
          role="progressbar"
          aria-valuenow={Math.round(progress * 100)}
          aria-valuemin={0}
          aria-valuemax={100}
        >
          <div class="progress-line__fill" style="width: {progress * 100}%"></div>
        </div>

        <!-- Phase dots: ASCII chars in mono, teal active, dim ghost others -->
        <div class="phase-dots" role="list" aria-label="Phases">
          {#each PHASES as phase}
            <div class="phase-dot-wrap" role="listitem">
              <button
                class="phase-dot phase-dot--{phase.status}"
                use:tooltip={{ label: `Phase ${phase.num}: ${phase.label} (${phase.steps} steps)`, side: 'top' }}
                aria-label={`Phase ${phase.num}: ${phase.label}`}
              >{dotChar(phase.status)}</button>
              <span class="phase-dot-num mono">{phase.num}</span>
            </div>
          {/each}
        </div>
      </section>

      <!-- Thinking pulse — 1px teal breath line during EXECUTING per DESIGN.md §8 -->
      {#if fsmSnap?.state === 'EXECUTING'}
        <div class="thinking-pulse" aria-label="executing"></div>
      {/if}

      <!-- Section 3: Status snapshot — prose lines, no headers, no chrome -->
      <section class="monitor-section snapshot">
        <div class="snap-line">
          <span class="snap-label mono">step</span>
          <span class="snap-value snap-value--accent">{statusSnap.currently}</span>
        </div>
        <div class="snap-line">
          <span class="snap-label mono">health</span>
          <span class="snap-value">{statusSnap.health}</span>
        </div>
        <div class="snap-line">
          <span class="snap-label mono">model</span>
          <span class="snap-value mono">{statusSnap.model}</span>
        </div>
        {#if loadError}
          <div class="snap-line">
            <span class="snap-label mono">status</span>
            <span class="snap-value snap-value--warn mono">offline</span>
          </div>
        {/if}
      </section>

      <!-- Section 4: Interventions inbox — orbit-card floating-in-space -->
      {#if interventions.length > 0}
        <section class="monitor-section interventions">
          {#each interventions as iv}
            <div class="orbit-card anim-enter-lift">
              <div class="card-inner">
                <div class="iv-title">{iv.title}</div>
                <div class="iv-desc">{iv.description}</div>
                {#if iv.diff_summary}
                  <div class="diff-block">
                    <div class="diff-left-strip" aria-hidden="true"></div>
                    <pre class="diff-pre mono">{iv.diff_summary}</pre>
                  </div>
                {/if}
                <div class="iv-actions">
                  <button
                    class="btn-approve"
                    disabled={!!interventionBusy[iv.id]}
                    onclick={() => handleIntervention(iv.id, 'approve')}
                  >Approve</button>
                  <button
                    class="btn-outlined"
                    disabled={!!interventionBusy[iv.id]}
                    onclick={() => handleIntervention(iv.id, 'reject')}
                  >Reject</button>
                  <button
                    class="btn-outlined"
                    onclick={() => takeover(iv.id)}
                  >Take over</button>
                </div>
              </div>
            </div>
          {/each}
        </section>
      {/if}

      <!-- Section 5: Timeline — supervision log rows (proxy for activity; B-36 gap noted) -->
      <section class="monitor-section timeline">
        <div class="timeline-label mono">supervision</div>
        <div class="timeline-list" role="list">
          {#if TIMELINE.length === 0}
            <span class="tl-empty mono">no events yet</span>
          {:else}
            {#each TIMELINE as entry}
              {@const tone = supervisionTone(entry)}
              <button
                class="tl-row tl-row--{tone}"
                onclick={() => openActivity(`sup-${entry.ts}`)}
              >
                <span class="tl-time mono">{fmtTime(entry.ts)}</span>
                <span class="tl-body mono">{supervisionLabel(entry)}</span>
              </button>
            {/each}
          {/if}
        </div>
      </section>

    </div>
  {/if}
</div>

<style>
  /* ================================================================
     MonitorWindow — orbit aesthetic.
     Phase dots = mono ASCII. Progress = 1px teal stroke.
     Interventions = floating cards (no bordered containers).
     Timeline = ghost-text mono rows.
  ================================================================ */

  .monitor {
    display: flex;
    flex-direction: column;
    height: 100%;
    background: var(--bg-void);
    overflow: hidden;
  }

  .monitor__empty {
    flex: 1;
    display: flex;
    align-items: center;
    justify-content: center;
  }

  .monitor__inner {
    flex: 1;
    overflow-y: auto;
    scrollbar-width: thin;
    scrollbar-color: var(--border-default) transparent;
    display: flex;
    flex-direction: column;
    gap: var(--s-8);
    padding: var(--s-6) var(--s-6) var(--s-8);
  }

  .monitor__inner::-webkit-scrollbar { width: 4px; }
  .monitor__inner::-webkit-scrollbar-track { background: transparent; }
  .monitor__inner::-webkit-scrollbar-thumb { background: var(--border-default); border-radius: var(--r-pill); }
  .monitor__inner::-webkit-scrollbar-thumb:hover { background: var(--border-strong); }

  .monitor-section {
    display: flex;
    flex-direction: column;
  }

  /* ---- Identity ---- */
  .identity {
    align-items: center;
    gap: 4px;
    padding-top: var(--s-2);
  }
  .identity-task {
    font-family: var(--font-sans);
    font-size: 16px;
    font-weight: 500;
    color: var(--text-primary);
    text-align: center;
  }
  .identity-elapsed {
    font-size: 11px;
    color: var(--text-ghost);
  }

  /* ---- Progress: thin teal stroke, no bg, no rounded corners ---- */
  .progress { gap: var(--s-4); }

  .progress-line {
    height: 1px;
    background: transparent;
    position: relative;
    overflow: hidden;
  }
  .progress-line::before {
    content: '';
    position: absolute;
    inset: 0;
    background: var(--border-subtle);
  }
  .progress-line__fill {
    position: relative;
    height: 1px;
    background: var(--teal);
    transition: width var(--dur-slow) var(--ease-std);
  }

  /* ---- Phase dots: ASCII chars in mono. No fills, no borders. ---- */
  .phase-dots {
    display: flex;
    flex-direction: row;
    gap: var(--s-3);
    align-items: flex-end;
    justify-content: center;
  }
  .phase-dot-wrap {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 2px;
  }
  .phase-dot {
    background: transparent;
    border: none;
    font-family: var(--font-mono);
    font-size: 16px;
    line-height: 1;
    cursor: pointer;
    padding: 0;
    color: var(--text-ghost);
    outline: none;
    transition: color var(--dur-fast) var(--ease-std);
  }
  .phase-dot:focus-visible { box-shadow: var(--ring); border-radius: var(--r-xs); }
  .phase-dot:hover { color: var(--text-secondary); }

  .phase-dot--done    { color: var(--teal); }
  .phase-dot--active  {
    color: var(--teal-bright);
    animation: phaseBreath 3s ease-in-out infinite;
  }
  .phase-dot--skipped { color: var(--text-disabled); }
  .phase-dot--pending { color: var(--text-ghost); }
  .phase-dot--intervention { color: var(--warn); }

  /* DESIGN.md §8: progress dots pulse — scale 100→110→100 over 3s */
  @keyframes phaseBreath {
    0%, 100% { transform: scale(1); }
    50%       { transform: scale(1.10); }
  }

  .phase-dot-num {
    font-size: 10px;
    color: var(--text-ghost);
  }

  /* ---- Thinking pulse: 1px teal stroke during EXECUTING (mirrors ExploreWindow) ---- */
  .thinking-pulse {
    height: 1px;
    background: var(--teal);
    flex-shrink: 0;
    animation: monitorBreath 2s ease-in-out infinite;
  }
  @keyframes monitorBreath {
    0%, 100% { opacity: 0.2; }
    50%       { opacity: 0.5; }
  }

  /* ---- Snapshot: prose lines, no headers, no chrome ---- */
  .snapshot { gap: var(--s-1); }

  .snap-line {
    display: flex;
    flex-direction: row;
    gap: var(--s-3);
    align-items: baseline;
  }
  .snap-label {
    font-size: 11px;
    color: var(--text-ghost);
    min-width: 72px;
    flex-shrink: 0;
  }
  .snap-value {
    font-family: var(--font-sans);
    font-size: 12px;
    color: var(--text-secondary);
    line-height: 1.5;
  }
  .snap-value--accent { color: var(--teal); }
  .snap-value--warn   { color: var(--warn); }

  /* ---- Interventions: orbit-card floating-in-space (mirrors CommandWindow) ---- */
  .interventions {
    gap: var(--s-3);
  }

  .orbit-card {
    position: relative;
    padding: var(--s-4) var(--s-5);
    border-radius: var(--r-md);
    background: transparent;
    outline: 1px solid transparent;
    transition:
      outline-color var(--dur-fast) var(--ease-std),
      background var(--dur-fast) var(--ease-std);
    animation: cardGlow 1.6s var(--ease-out) both;
  }

  .orbit-card:hover {
    outline-color: rgba(255,255,255,0.05);
    background: rgba(16,20,26,0.35);
  }

  @keyframes cardGlow {
    0%   { outline-color: rgba(30,138,122,0.28); }
    60%  { outline-color: rgba(30,138,122,0.12); }
    100% { outline-color: transparent; }
  }

  .card-inner {
    display: flex;
    flex-direction: column;
    gap: var(--s-2);
  }

  .iv-title {
    font-family: var(--font-sans);
    font-size: 14px;
    font-weight: 500;
    color: var(--text-primary);
    line-height: 1.3;
  }

  .iv-desc {
    font-family: var(--font-sans);
    font-size: 12px;
    color: var(--text-secondary);
    line-height: 1.5;
  }

  .diff-block {
    position: relative;
    background: rgba(26, 32, 41, 0.6);
    border: 1px solid var(--border-subtle);
    border-radius: var(--r-sm);
    overflow: hidden;
    margin: var(--s-1) 0;
  }
  .diff-left-strip {
    position: absolute;
    left: 0;
    top: 0;
    bottom: 0;
    width: 1px;
    background: var(--teal-line);
    opacity: 0.5;
    pointer-events: none;
  }
  .diff-pre {
    font-size: 11px;
    color: var(--text-secondary);
    padding: var(--s-2) var(--s-3) var(--s-2) calc(var(--s-3) + 3px);
    white-space: pre-wrap;
    word-break: break-all;
    line-height: 1.5;
    margin: 0;
    background: none;
  }

  .iv-actions {
    display: flex;
    flex-wrap: wrap;
    gap: var(--s-2);
    margin-top: var(--s-2);
    align-items: center;
  }

  /* Approve: outlined teal — primary feeling, never solid fill */
  .btn-approve {
    font-family: var(--font-sans);
    font-size: 12px;
    font-weight: 500;
    color: var(--teal);
    background: transparent;
    border: 1px solid var(--teal-line);
    border-radius: var(--r-sm);
    padding: 4px 12px;
    cursor: pointer;
    outline: none;
    transition:
      background var(--dur-fast) var(--ease-std),
      color var(--dur-fast) var(--ease-std),
      border-color var(--dur-fast) var(--ease-std);
  }
  .btn-approve:hover:not(:disabled) {
    background: var(--teal-ghost);
    color: var(--teal-bright);
    border-color: var(--teal);
  }
  .btn-approve:disabled { opacity: 0.5; cursor: not-allowed; }
  .btn-approve:focus-visible { box-shadow: var(--ring); }

  /* Outlined: secondary action buttons */
  .btn-outlined {
    font-family: var(--font-sans);
    font-size: 12px;
    color: var(--text-secondary);
    background: transparent;
    border: 1px solid var(--border-default);
    border-radius: var(--r-sm);
    padding: 4px 12px;
    cursor: pointer;
    outline: none;
    transition:
      background var(--dur-fast) var(--ease-std),
      color var(--dur-fast) var(--ease-std),
      border-color var(--dur-fast) var(--ease-std);
  }
  .btn-outlined:hover:not(:disabled) {
    background: var(--teal-ghost);
    color: var(--text-primary);
    border-color: var(--teal-line);
  }
  .btn-outlined:disabled { opacity: 0.5; cursor: not-allowed; }
  .btn-outlined:focus-visible { box-shadow: var(--ring); }

  /* ---- Timeline: ghost-text mono rows ---- */
  .timeline { gap: var(--s-2); }

  .timeline-label {
    font-size: 10px;
    text-transform: uppercase;
    letter-spacing: 0.10em;
    color: var(--text-ghost);
  }

  .timeline-list {
    display: flex;
    flex-direction: column;
  }

  .tl-row {
    display: flex;
    gap: var(--s-3);
    padding: 2px var(--s-1);
    background: transparent;
    border: none;
    cursor: pointer;
    text-align: left;
    outline: none;
    border-radius: var(--r-xs);
    transition: background var(--dur-fast) var(--ease-std), color var(--dur-fast) var(--ease-std);
  }
  .tl-row:hover .tl-body { color: var(--teal); }
  .tl-row:focus-visible { box-shadow: var(--ring); }

  .tl-time {
    font-size: 11px;
    color: var(--text-disabled);
    flex-shrink: 0;
    min-width: 36px;
  }
  .tl-body {
    font-size: 11px;
    color: var(--text-ghost);
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    transition: color var(--dur-fast) var(--ease-std);
  }
  .tl-row--pass .tl-body { color: var(--teal); opacity: 0.85; }
  .tl-row--fail .tl-body { color: var(--fail); opacity: 0.85; }
  .tl-row--warn .tl-body { color: var(--warn); opacity: 0.85; }

  /* Failure: animated 2px left border (200ms fade) per DESIGN.md §8 */
  .tl-row--fail {
    animation: failBorder 200ms ease-out both;
  }
  @keyframes failBorder {
    0%   { border-left: 2px solid transparent; }
    100% { border-left: 2px solid var(--fail); }
  }

  /* Step completion flash: 100ms teal-ghost per DESIGN.md §8 */
  @keyframes step-flash {
    0%, 100% { background: transparent; }
    50%       { background: var(--teal-ghost); }
  }
  .tl-row--pass { animation: step-flash 100ms ease-in-out; }

  .tl-empty {
    font-size: 11px;
    color: var(--text-ghost);
    padding: 4px var(--s-1);
  }

  /* ---- Section 6: AutoAdvanceStats — Phase 4G ---- */
  .auto-advance {
    gap: var(--s-2);
    padding: var(--s-3) var(--s-6) var(--s-2);
    border-bottom: 1px solid var(--border-subtle);
    flex-shrink: 0;
  }
  .auto-advance__header {
    display: flex;
    align-items: baseline;
    justify-content: space-between;
    gap: var(--s-3);
  }
  .auto-advance__label {
    font-size: 10px;
    text-transform: uppercase;
    letter-spacing: 0.10em;
    color: var(--text-ghost);
  }
  /* patch-4G.1: mode mini-badge mirrors StatusBar styling so users
     read the active gate mode in the auto-advance card too. */
  .auto-advance__mode {
    font-size: 10px;
    letter-spacing: 0.05em;
    color: var(--text-ghost);
  }
  .auto-advance__mode--autonomous { color: var(--teal); }
  .auto-advance__mode--interactive { color: var(--text-ghost); }
  .auto-advance__row {
    display: flex;
    flex-direction: row;
    flex-wrap: wrap;
    gap: var(--s-4);
    align-items: baseline;
  }
  .auto-advance__metric {
    display: inline-flex;
    align-items: baseline;
    gap: 4px;
    font-size: 12px;
  }
  .auto-advance__num {
    color: var(--teal);
    font-weight: 500;
  }
  .auto-advance__pct {
    color: var(--teal-bright);
    font-weight: 500;
  }
  .auto-advance__sep {
    color: var(--text-ghost);
  }
  .auto-advance__unit {
    color: var(--text-ghost);
    font-size: 10px;
    text-transform: uppercase;
    letter-spacing: 0.05em;
  }
  .auto-advance__metric--warn .auto-advance__num {
    color: var(--warn);
  }
</style>
