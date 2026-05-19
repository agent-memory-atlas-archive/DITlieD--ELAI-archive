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
    CloudSpendInfo,
    // C-28: T-1 mutation testing gauge surfaced in the Monitor.
    MutationGaugeInfo,
    // C-32: Code-Completeness Layer 2 completeness_ratio gauge.
    CompletenessRatioInfo,
    // B-40: Idea-Framer state (read for critic-rounds badge).
    IdeaFramingState,
    // B-42: cosmos artifact freshness counts for the monitor badge.
    CosmosArtifactStatus,
    // B-41: per-level audit verdicts mini-badges.
    LevelAuditVerdictJson,
  } from '$lib/api/types';
  import { workspace } from '$lib/state/workspace.svelte';
  import { session } from '$lib/state/session.svelte';
  // B-23 Phase 4H Group F (4H.8) — self-mod surfaces wired into MonitorWindow.
  import ProposalsPanel from '$lib/components/ProposalsPanel.svelte';
  import MsiPanel from '$lib/components/MsiPanel.svelte';
  // B-24 Phase 4I Group C8 — media validator pipeline + RCE feedback loop.
  import MediaValidationPanel from '$lib/components/MediaValidationPanel.svelte';
  // B-28 follow-up — Focus tunnel surface moved to inline chips on the
  // PlanWindow baby-step detail card. No sibling panel is mounted here.

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
  // B-40: latest IdeaFraming snapshot for the critic-rounds badge.
  let framingState = $state<IdeaFramingState | null>(null);
  let sysStatus = $state<SystemStatus | null>(null);
  let supervisionLog = $state<SupervisionEvent[]>([]);
  let autoStats = $state<AutoAdvanceStats | null>(null);
  // B-27: cloud-mode spend gauge. Spec D-4:308 — rows are conditionally
  // hidden when both task_usd and day_usd are 0 (no "$0.00" display).
  let cloudSpend = $state<CloudSpendInfo | null>(null);
  // C-28: T-1 mutation gauge. Null = no mutation run yet (renders a grey
  // placeholder row); populated value renders green (score >= threshold)
  // or amber (score < threshold). Polled every 5s with the rest.
  let mutationGauge = $state<MutationGaugeInfo | null>(null);
  // C-32: Code-Completeness Layer 2 completeness_ratio gauge. Audit-C32
  // W2 close-out: the wire shape is now an envelope `{ known: false }` |
  // `{ known: true, ratio, blank_count, last_path }`. `{ known: false }`
  // renders the grey placeholder; `{ known: true }` drives the gauge
  // colour per workflow/reasoning/C-32-decisions.md D3 / plan §Task 6.
  // Polled every 5s with the rest.
  let completenessRatio = $state<CompletenessRatioInfo>({ known: false });
  // B-42: cosmos artifact tier counts (fresh/stale/decayed), polled every 30s
  // while the FSM is in a PLANNING-adjacent substate.
  let cosmosArtifacts = $state<CosmosArtifactStatus[]>([]);
  // B-41: per-level audit verdict feed for the auditor mini-badges.
  let perLevelVerdicts = $state<LevelAuditVerdictJson[]>([]);
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

  // B-41: per-mode audit counts derived from perLevelVerdicts. Renders 5
  // mini-badges (one per per-level mode) in the auto-advance header. Plan
  // mode is folded into the existing legacy auditor pass — no separate
  // badge.
  let auditCounts = $derived.by(() => {
    const counts = { concepts: 0, phases: 0, steps: 0, babysteps: 0, babystep_detail: 0 };
    let anyCascade = false;
    let anyBlock = false;
    for (const v of perLevelVerdicts) {
      if (v.mode in counts) {
        // @ts-expect-error narrow record keyed by AuditorMode subset.
        counts[v.mode]++;
      }
      if (v.cascade_to_parent) anyCascade = true;
      if (v.verdict === 'block') anyBlock = true;
    }
    const status: 'red' | 'yellow' | 'green' =
      anyCascade ? 'red' : anyBlock ? 'yellow' : 'green';
    return { ...counts, status, total: perLevelVerdicts.length };
  });

  // B-41: poll the per-level verdict feed every 5s (matches FSM poll cadence).
  $effect(() => {
    let alive = true;
    async function poll() {
      try {
        perLevelVerdicts = await stub.getPerLevelVerdicts('active');
      } catch {
        // graceful: keep last snapshot
      }
    }
    poll();
    const interval = setInterval(() => {
      if (!alive) return;
      poll();
    }, 5000);
    return () => {
      alive = false;
      clearInterval(interval);
    };
  });

  // B-42: cosmos counts derived from cosmosArtifacts.
  let cosmosCounts = $derived.by(() => {
    let fresh = 0;
    let stale = 0;
    let decayed = 0;
    for (const a of cosmosArtifacts) {
      if (a.status === 'fresh') fresh++;
      else if (a.status === 'decayed') decayed++;
      else stale++;
    }
    return { fresh, stale, decayed, total: cosmosArtifacts.length };
  });

  // B-42: poll cosmos status every 30s when the FSM is in PLANNING or in the
  // FRAMING substate (cosmos briefings are consumed during plan composition).
  $effect(() => {
    let alive = true;
    async function pollCosmos() {
      try {
        cosmosArtifacts = await stub.getCosmosStatus();
      } catch {
        // graceful: keep last snapshot
      }
    }
    pollCosmos();
    const interval = setInterval(() => {
      if (!alive) return;
      const inPlanning = fsmSnap?.state === 'PLANNING';
      const inFraming = fsmSnap?.agent_substate === 'Framing';
      if (inPlanning || inFraming || cosmosArtifacts.length === 0) {
        pollCosmos();
      }
    }, 30_000);
    return () => {
      alive = false;
      clearInterval(interval);
    };
  });

  // ------------------------------------------------------------------
  // Polling — every 5s; cleans up on unmount
  // ------------------------------------------------------------------
  $effect(() => {
    let alive = true;

    async function poll() {
      try {
        const [fsm, sys, log, stats, spend, framing, mutation, completeness] = await Promise.all([
          stub.get_fsm_state(),
          stub.get_system_status(),
          stub.get_supervision_log(),
          stub.get_auto_advance_stats(),
          stub.get_cloud_spend(),
          // B-40: critic-rounds badge data source.
          stub.get_idea_framing_state(),
          // C-28: T-1 mutation gauge readback.
          stub.get_mutation_gauge(),
          // C-32: Code-Completeness Layer 2 completeness_ratio gauge.
          stub.get_completeness_ratio(),
        ]);
        if (!alive) return;
        fsmSnap = fsm;
        sysStatus = sys;
        supervisionLog = log;
        autoStats = stats;
        cloudSpend = spend;
        framingState = framing;
        mutationGauge = mutation;
        completenessRatio = completeness;
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
  <!-- B-23 Phase 4H Group F (4H.8) — Self-mod surfaces. -->
  <section class="monitor-section self-mod-msi" aria-label="MSI per pipeline mode">
    <MsiPanel />
  </section>
  <section class="monitor-section self-mod-proposals" aria-label="Layer 1A proposals">
    <ProposalsPanel />
  </section>

  <!-- B-24 Phase 4I Group C8 — media validator pipeline + RCE feedback loop.
       Surfaces per-step Tier1 results, Tier2 advisory, render preview
       thumbnails, iteration count, and HumanReview alerts. Mount lives
       above the auto-advance bar so HumanReview rows are immediately
       visible when the gate stops the run. -->
  <section class="monitor-section media-validation" aria-label="Media validation">
    <MediaValidationPanel />
  </section>

  <!-- B-28 follow-up — Focus tunnel signals moved inline onto the PlanWindow
       baby-step detail card (Mode / Early-term / Micro-steps / Deps chips).
       The dedicated panel surface was retired because the focus tunnel is
       middleware and is not user-visible at task submission time. -->

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
      {#if cosmosCounts.total > 0}
        <span
          class="cosmos-badge mono"
          title="Cosmos artifact freshness (fresh / stale / decayed). Click rail's COSMOS drawer for detail."
          aria-label="Cosmos artifacts: {cosmosCounts.fresh} fresh, {cosmosCounts.stale} stale, {cosmosCounts.decayed} decayed"
        >cosmos: {cosmosCounts.fresh}/{cosmosCounts.stale}/{cosmosCounts.decayed}</span>
      {/if}
      <!-- B-41: per-mode auditor mini-badges + overall status dot. Always
           rendered (vacuous-default-zero) so a 0/0/0/0/0 baseline is visible. -->
      <span
        class="audit-badge mono audit-badge--{auditCounts.status}"
        title="Per-level audit counts (concepts/phases/steps/babysteps/babystep_detail). Status: {auditCounts.status}."
        aria-label="Per-level audits: concepts {auditCounts.concepts}, phases {auditCounts.phases}, steps {auditCounts.steps}, babysteps {auditCounts.babysteps}, babystep_detail {auditCounts.babystep_detail}; status {auditCounts.status}"
      >audit {auditCounts.concepts}/{auditCounts.phases}/{auditCounts.steps}/{auditCounts.babysteps}/{auditCounts.babystep_detail}</span>
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

  <!-- B-27: cloud-mode spend gauge (spec D-4:307). Conditional on > 0;
       no "$0.00" rendered when no cloud spend has been recorded. -->
  {#if cloudSpend && (cloudSpend.task_usd > 0 || cloudSpend.day_usd > 0)}
    <section class="monitor-section cloud-spend" aria-label="Cloud spend">
      <div class="auto-advance__header">
        <span class="auto-advance__label mono">cloud spend</span>
      </div>
      <div class="auto-advance__row">
        {#if cloudSpend.task_usd > 0}
          <span
            class="auto-advance__metric mono"
            title="USD spent on cloud calls during the current task"
          >
            <span class="auto-advance__pct">${cloudSpend.task_usd.toFixed(4)}</span>
            <span class="auto-advance__unit">task</span>
          </span>
        {/if}
        {#if cloudSpend.day_usd > 0}
          <span
            class="auto-advance__metric mono"
            title="USD spent on cloud calls today"
          >
            <span class="auto-advance__pct">${cloudSpend.day_usd.toFixed(2)}</span>
            <span class="auto-advance__unit">today</span>
          </span>
        {/if}
        {#if cloudSpend.cache_read_tokens > 0}
          <span
            class="auto-advance__metric mono"
            title="Tokens served from Anthropic's prompt cache (10× cheaper than fresh input)"
          >
            <span class="auto-advance__num">{cloudSpend.cache_read_tokens}</span>
            <span class="auto-advance__unit">cache hit</span>
          </span>
        {/if}
        {#if cloudSpend.redaction_count > 0}
          <span
            class="auto-advance__metric mono"
            title="Number of substitutions applied by the privacy redactor before each cloud send"
          >
            <span class="auto-advance__num">{cloudSpend.redaction_count}</span>
            <span class="auto-advance__unit">redacted</span>
          </span>
        {/if}
      </div>
    </section>
  {/if}

  <!-- C-32: Code-Completeness Layer 2 completeness_ratio gauge. Four states:
       (a) null     -> grey "no skeleton flushed yet" placeholder.
       (b) >= 0.9   -> green "ok" metric class.
       (c) 0.5..0.9 -> amber "warn" metric class.
       (d) < 0.5    -> red "fail" tone (re-uses --warn class — there is no
                       --fail class on auto-advance__metric today; the red
                       tone is the dashboard's current "below threshold"
                       treatment, see workflow/reasoning/C-32-decisions.md
                       D3). The gauge updates on every poll (5s); the
                       underlying source of truth is the latest
                       `skeleton_flushed` SSB row. -->
  <section class="monitor-section completeness-gauge" aria-label="Completeness ratio">
    <div class="auto-advance__header">
      <span class="auto-advance__label mono">completeness</span>
    </div>
    {#if !completenessRatio.known}
      <div class="auto-advance__row">
        <span class="auto-advance__metric mono" title="No skeleton flushed for this session yet">
          <span class="auto-advance__unit">no skeleton flushed yet</span>
        </span>
      </div>
    {:else}
      <div class="auto-advance__row">
        <span
          class="auto-advance__metric mono"
          class:auto-advance__metric--warn={completenessRatio.ratio < 0.9}
          class:auto-advance__metric--ok={completenessRatio.ratio >= 0.9}
          title="Completeness ratio (non-blocked fills / total blanks). Red < 0.5, yellow 0.5..0.9, green >= 0.9."
        >
          <span class="auto-advance__pct">{(completenessRatio.ratio * 100).toFixed(1)}%</span>
          <span class="auto-advance__unit">ratio ({completenessRatio.blank_count} blanks)</span>
        </span>
        {#if completenessRatio.last_path}
          <span
            class="auto-advance__metric mono"
            title="Most recent skeleton flush path"
          >
            <span class="auto-advance__unit">{completenessRatio.last_path}</span>
          </span>
        {/if}
      </div>
    {/if}
  </section>

  <!-- C-28: T-1 mutation testing gauge. Three states:
       (a) null      -> grey "no mutation run yet" placeholder.
       (b) score >= threshold -> green "ok" metric class.
       (c) score < threshold  -> amber "warn" metric class. -->
  <section class="monitor-section mutation-gauge" aria-label="Mutation score">
    <div class="auto-advance__header">
      <span class="auto-advance__label mono">mutation</span>
    </div>
    {#if mutationGauge === null}
      <div class="auto-advance__row">
        <span class="auto-advance__metric mono" title="No mutation run recorded for this session yet">
          <span class="auto-advance__unit">no mutation run yet</span>
        </span>
      </div>
    {:else}
      <div class="auto-advance__row">
        <span
          class="auto-advance__metric mono"
          class:auto-advance__metric--warn={mutationGauge.score_pct < mutationGauge.threshold_pct}
          class:auto-advance__metric--ok={mutationGauge.score_pct >= mutationGauge.threshold_pct}
          title="Mutation score (caught / (caught + survived + timeout))"
        >
          <span class="auto-advance__pct">{mutationGauge.score_pct.toFixed(1)}%</span>
          <span class="auto-advance__unit">score (threshold {mutationGauge.threshold_pct.toFixed(0)}%)</span>
        </span>
        {#if mutationGauge.cache_hits > 0}
          <span
            class="auto-advance__metric mono"
            title="Mutants served from EquivalentMutantCache (skipped re-run)"
          >
            <span class="auto-advance__num">{mutationGauge.cache_hits}</span>
            <span class="auto-advance__unit">cache hit</span>
          </span>
        {/if}
        <span class="auto-advance__metric mono" title="Total mutants run this session">
          <span class="auto-advance__num">{mutationGauge.total_run}</span>
          <span class="auto-advance__unit">total</span>
        </span>
      </div>
    {/if}
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

      <!-- B-40: Critic rounds badge.
           Surfaces "Critic rounds: N/3" when the FRAMING substate is active
           (PLANNING + agent_substate=Framing). Source: framingState polled
           every 5s via get_idea_framing_state. -->
      {#if fsmSnap?.state === 'PLANNING' && fsmSnap?.agent_substate === 'Framing' && framingState}
        <div class="framing-badge mono" aria-label="critic rounds">
          Critic rounds: {framingState.critic_rounds_used} / 3
          {#if framingState.is_converged}
            <span class="framing-badge-status badge-ok">converged</span>
          {:else if framingState.critic_rounds_used >= 3}
            <span class="framing-badge-status badge-block">BLOCK</span>
          {:else}
            <span class="framing-badge-status badge-active">in progress</span>
          {/if}
        </div>
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
    font-size: 10px;
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
    font-size: 10px;
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
    font-size: 10px;
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
    font-size: 10px;
    color: var(--text-disabled);
    flex-shrink: 0;
    min-width: 36px;
  }
  .tl-body {
    font-size: 10px;
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
    font-size: 10px;
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
  /* B-42 — cosmos artifact freshness badge. */
  .cosmos-badge {
    margin-left: 8px;
    font-size: 10px;
    color: var(--text-secondary);
    padding: 1px 6px;
    border-radius: var(--r-xs);
    background: var(--bg-input);
  }
  /* B-41: per-mode audit mini-badge (overall status from any cascade flag). */
  .audit-badge {
    margin-left: 8px;
    font-size: 10px;
    padding: 1px 6px;
    border-radius: var(--r-xs);
    background: var(--bg-input);
    color: var(--text-secondary);
    border-left: 2px solid transparent;
  }
  .audit-badge--green { border-left-color: #22c55e; }
  .audit-badge--yellow { border-left-color: #eab308; }
  .audit-badge--red { border-left-color: #ef4444; color: #fca5a5; }
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

  /* B-40: critic-rounds badge — surfaces during FRAMING substate. */
  .framing-badge {
    display: inline-flex;
    align-items: center;
    gap: 8px;
    margin: 8px 0;
    padding: 4px 10px;
    font-size: 10px;
    border-radius: 3px;
    background: rgba(95, 168, 255, 0.08);
    color: #5fa8ff;
    border: 1px solid rgba(95, 168, 255, 0.25);
  }
  .framing-badge-status {
    font-size: 10px;
    padding: 1px 6px;
    border-radius: 2px;
  }
  .badge-ok {
    background: rgba(111, 192, 111, 0.18);
    color: #6fc06f;
  }
  .badge-block {
    background: rgba(255, 107, 107, 0.18);
    color: #ff6b6b;
  }
  .badge-active {
    background: rgba(95, 168, 255, 0.18);
    color: #5fa8ff;
  }
</style>
