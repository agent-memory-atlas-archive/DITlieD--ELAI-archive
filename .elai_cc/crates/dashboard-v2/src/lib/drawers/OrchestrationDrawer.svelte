<script lang="ts">
  import { onMount } from 'svelte';
  import { invoke } from '@tauri-apps/api/core';
  import { orchestrationData } from '../state/drawers.svelte';
  import { Card, Badge, Button } from '$lib/ui';
  import { logStub, get_sandbox_status } from '../api/stub';
  import { tooltip } from '$lib/ui/tooltip';
  import type {
    FsmState, GateVerdict, AgentLifecycle, BspSuperstep, SandboxStatus, SandboxTier,
    AmbiguityState, AmbiguityComponent,
  } from '../api/types';

  onMount(() => {
    orchestrationData.bootstrap();
    return orchestrationData.teardown;
  });

  const d = orchestrationData;

  // FSM state tone via state-* tokens
  function fsmTone(state: FsmState): string {
    const map: Record<FsmState, string> = {
      IDLE:             'state-idle',
      PLANNING:         'state-planning',
      FEASIBILITY_PROBE:'state-feasibility',
      TESTING_FIRST:    'state-testing',
      EXECUTING:        'state-executing',
      AUDITING:         'state-auditing',
      MEMORIZING:       'state-memorizing',
      COMPLETE:         'state-complete',
      CONFIGURING:      'state-configuring',
      RESEARCHING:      'state-researching',
    };
    return map[state] ?? 'ghost';
  }

  // Last 4 FSM transitions
  const lastTransitions = $derived(d.fsmSnapshot.history.slice(-4));

  function gateTone(v: GateVerdict): string {
    if (v === 'pass')    return 'teal';
    if (v === 'block')   return 'fail';
    if (v === 'warn')    return 'warn';
    return 'ghost';
  }

  function agentLifeTone(lc: AgentLifecycle): string {
    if (lc === 'RUNNING')    return 'teal';
    if (lc === 'DEAD')       return 'fail';
    if (lc === 'RESTARTING') return 'warn';
    if (lc === 'STARTING')   return 'warn';
    return 'ghost';
  }

  function agentLifeLabel(lc: AgentLifecycle): string {
    if (lc === 'RUNNING')    return 'working';
    if (lc === 'IDLE')       return 'idle';
    if (lc === 'DEAD')       return 'dead';
    if (lc === 'RESTARTING') return 'restart';
    return 'starting';
  }

  function superstepTone(s: BspSuperstep): string {
    if (s.status === 'PASS') return 'teal';
    if (s.status === 'FAIL') return 'fail';
    return 'warn';
  }

  function fmtTs(ts: number): string {
    return new Date(ts).toISOString().slice(11, 19);
  }

  function supervisionTone(event: string): string {
    if (event === 'DIED')               return 'fail';
    if (event === 'RESTARTED')          return 'warn';
    if (event === 'HEARTBEAT_TIMEOUT')  return 'warn';
    if (event === 'STARTED')            return 'teal';
    return 'ghost';
  }

  // Context budget: map slot names to display names
  const slotDisplayNames: Record<string, string> = {
    'system-prompt':   'SystemPrompt',
    'memory-bank':     'Rules',
    'plan-context':    'PhaseSpec',
    'active-file':     'RepoMap',
    'activity-stream': 'CodeContext',
    'rag-results':     'SsbEntries',
    'tool-output':     'GrcHistory',
    'reserve':         'OutputReserve',
  };

  const slotColors = [
    'var(--teal-bright)',
    'var(--steel)',
    'var(--warn)',
    'var(--teal)',
    'var(--text-secondary)',
    'var(--state-researching)',
    'var(--state-planning)',
    'var(--text-ghost)',
  ];

  const slots = $derived(
    d.contextBudget.slots.map((s, i) => ({
      ...s,
      displayName: slotDisplayNames[s.name] ?? s.name,
      color: slotColors[i % slotColors.length],
    }))
  );

  const totalPct = $derived(
    Math.round((d.contextBudget.total_used / d.contextBudget.total_cap) * 100)
  );

  // Cache breakpoints at ~33% and ~66% of context window
  const bp1Pct = 33;
  const bp2Pct = 66;

  // Supervision: show last 5 events
  const recentSupervision = $derived(d.supervisionLog.slice(-5));

  // Last 8 BSP supersteps
  const supersteps = $derived(d.bspSupersteps.slice(-8));

  // B-18 sandbox tier badge
  let sandboxStatus = $state<SandboxStatus | null>(null);
  onMount(async () => {
    try {
      sandboxStatus = await get_sandbox_status();
    } catch (_) {
      // non-fatal: badge remains hidden
    }
  });

  // C-2: ambiguity gate snapshot — replaces the legacy B-38 mock with the
  // live SSB-backed `get_ambiguity_state` Tauri command. Refreshed every 5s
  // so the AGENTS/AMBIGUITY/EVIDENCE-GATES rows stay in step.
  let ambig = $state<AmbiguityState | null>(null);
  let ambigError = $state<string | null>(null);

  async function refreshAmbiguity() {
    try {
      ambig = await invoke<AmbiguityState>('get_ambiguity_state');
      ambigError = null;
    } catch (e) {
      ambigError = String(e);
    }
  }

  $effect(() => {
    refreshAmbiguity();
    const id = setInterval(refreshAmbiguity, 5_000);
    return () => clearInterval(id);
  });

  function ambigPct(x: number): string {
    return `${Math.round(x * 100)}%`;
  }

  const ambigRows = $derived.by(() => {
    if (!ambig) return [] as { name: string; c: AmbiguityComponent | null }[];
    const r: { name: string; c: AmbiguityComponent | null }[] = [
      { name: 'Goal', c: ambig.components.goal_clarity },
      { name: 'Constraint', c: ambig.components.constraint_clarity },
      { name: 'Success criteria', c: ambig.components.success_criteria_clarity },
    ];
    if (ambig.brownfield) {
      r.push({ name: 'Context', c: ambig.components.context_clarity });
    }
    return r;
  });

  const ambigScoreColor = $derived.by(() => {
    if (!ambig) return 'var(--text-ghost)';
    return ambig.score <= ambig.threshold ? 'var(--teal)' : 'var(--fail)';
  });

  function sandboxTierTone(tier: SandboxTier): string {
    if (tier === 'C-Windows' || tier === 'C-Linux') return 'teal';
    if (tier === 'A++') return 'warn';
    return 'ghost';
  }

  // C-10: anti-spiral StagnationSection — mirrors the AMBIGUITY card
  // pattern (5s poll, single-DTO row list, color-coded badges via the
  // existing GatesPanel tokens — no new tokens introduced).
  type StagnationEvent = {
    timestamp_ms: number;
    action_type: string;
    agent_id: string;
    pattern: string | null;
    persona: string | null;
    drift_score: number | null;
    elapsed_ms: number | null;
    detail: string;
  };

  let stagEvents = $state<StagnationEvent[]>([]);
  let stagError = $state<string | null>(null);

  async function refreshStagnation() {
    try {
      stagEvents = await invoke<StagnationEvent[]>('get_stagnation_events');
      stagError = null;
    } catch (e) {
      stagError = String(e);
    }
  }

  $effect(() => {
    refreshStagnation();
    const id = setInterval(refreshStagnation, 5_000);
    return () => clearInterval(id);
  });

  function stagBadgeTone(action_type: string): string {
    if (action_type.startsWith('stagnation_')) return 'warn';
    if (action_type.startsWith('lateral_thinking_')) return 'teal';
    if (action_type.startsWith('goal_drift_')) return 'warn';
    if (action_type === 'watchdog_timeout') return 'fail';
    return 'ghost';
  }

  function stagBadgeLabel(action_type: string): string {
    if (action_type === 'stagnation_spinning_detected') return 'spin';
    if (action_type === 'stagnation_oscillation_detected') return 'osc';
    if (action_type === 'lateral_thinking_activated') return 'switch';
    if (action_type === 'lateral_thinking_succeeded') return 'win';
    if (action_type === 'lateral_thinking_exhausted') return 'exhausted';
    if (action_type === 'goal_drift_warning') return 'drift';
    if (action_type === 'goal_drift_block') return 'block';
    if (action_type === 'watchdog_timeout') return 'wd-to';
    return action_type;
  }

  // C-45: CANTANTE contrastive credit attribution — read-only credit
  // table surfaced via the new `credit_attribution_status` Tauri
  // command. Paper-grounded against arXiv:2605.13295 (Zehle 2026,
  // Section 3.2 — K rollouts NOT K^N, scalar c in [-1,+1], g=3).
  // The live runpod pilot is RUNPOD-PENDING; this surface renders the
  // most-recent in-process synthetic-fixture result so dashboard users
  // can see the shape of the eventual real measurement.
  type CreditAttributionRow = {
    role: string;
    score: number;
  };
  type CreditAttributionSnapshot = {
    rows: CreditAttributionRow[];
    ts_ms: number;
    runpod_pending: boolean;
  };

  let creditSnapshot = $state<CreditAttributionSnapshot | null>(null);
  let creditError = $state<string | null>(null);

  async function refreshCredit() {
    try {
      creditSnapshot = await invoke<CreditAttributionSnapshot>('credit_attribution_status');
      creditError = null;
    } catch (e) {
      creditError = String(e);
    }
  }

  $effect(() => {
    refreshCredit();
    const id = setInterval(refreshCredit, 10_000);
    return () => clearInterval(id);
  });

  function creditScoreTone(score: number): string {
    if (score > 0.3) return 'teal';
    if (score < -0.3) return 'fail';
    if (score < 0) return 'warn';
    return 'ghost';
  }

  // C-46: Uno-Orchestra parsimonious agent routing — recent selective-
  // delegation decisions surfaced via the `routing_decisions_status`
  // Tauri command. Paper-grounded against arXiv:2605.05007 (Cui et al
  // 2026, Sections 5 + 6.1 — (model, primitive) action space, alpha in
  // [0, 1] convex blend, S_max <= 0.10 cost-hack bound, direct-answer
  // lazy-mode cost zero). Live runs are RUNPOD-PENDING; this surface
  // renders the synthetic-pilot result so dashboard users can see the
  // shape of the eventual real measurement.
  type RoutingDispatchPair = {
    role: string;
    primitive: string;
  };
  type RoutingDecisionRow = {
    task_id: string;
    difficulty: number;
    alpha: number;
    dispatched: RoutingDispatchPair[];
    is_lazy_mode: boolean;
  };
  type RoutingPolicySnapshot = {
    enabled: boolean;
    alpha: number;
    s_max: number;
    max_turns: number;
    max_workers_per_query: number;
  };
  type RoutingDecisionsSnapshot = {
    rows: RoutingDecisionRow[];
    policy: RoutingPolicySnapshot;
    ts_ms: number;
    runpod_pending: boolean;
  };

  let routingSnapshot = $state<RoutingDecisionsSnapshot | null>(null);
  let routingError = $state<string | null>(null);

  async function refreshRouting() {
    try {
      routingSnapshot = await invoke<RoutingDecisionsSnapshot>('routing_decisions_status');
      routingError = null;
    } catch (e) {
      routingError = String(e);
    }
  }

  $effect(() => {
    refreshRouting();
    const id = setInterval(refreshRouting, 10_000);
    return () => clearInterval(id);
  });

  function difficultyTone(d: number): string {
    if (d >= 0.7) return 'fail';
    if (d >= 0.3) return 'warn';
    return 'teal';
  }
</script>

<div class="orchestration-drawer">

  <!-- 1. FSM -->
  <Card variant="plain" padding={12}>
    <div class="section-label">FSM</div>
    <div class="fsm-current">
      <Badge tone={fsmTone(d.fsmSnapshot.state)} size="sm">{d.fsmSnapshot.state}</Badge>
      {#if d.fsmSnapshot.agent_substate}
        <span class="fsm-sub">{d.fsmSnapshot.agent_substate}</span>
      {/if}
    </div>
    <div class="fsm-chain">
      {#each lastTransitions as t, i}
        <span class="chain-state" class:chain-state--current={i === lastTransitions.length - 1}>{t.to}</span>
        {#if i < lastTransitions.length - 1}
          <span class="chain-arrow">→</span>
        {/if}
      {/each}
    </div>
  </Card>

  <!-- 2. EVIDENCE GATES -->
  <Card variant="plain" padding={12}>
    <div class="section-label">EVIDENCE GATES</div>
    <div class="gate-rows">
      {#each d.evidenceGates as gate}
        <div class="gate-row" class:gate-row--block={gate.verdict === 'block'}>
          <span class="gate-name">{gate.name}</span>
          <Badge tone={gateTone(gate.verdict)} size="xs">{gate.verdict}</Badge>
          {#if gate.reason}
            <span class="gate-reason">{gate.reason.slice(0, 60)}{gate.reason.length > 60 ? '...' : ''}</span>
          {/if}
        </div>
      {/each}
    </div>
  </Card>

  <!-- 2b. AMBIGUITY GATE (C-2) — mirrors GatesPanel layout at
       lib/components/GatesPanel.svelte:91-148 (DRIFT-4 corrected path). -->
  <Card variant="plain" padding={12}>
    <div class="section-label">AMBIGUITY</div>
    {#if ambigError}
      <p class="ambig-error">{ambigError}</p>
    {:else if !ambig}
      <p class="ambig-loading">Loading…</p>
    {:else}
      <div class="ambig-overall">
        <span class="ambig-score-label">overall</span>
        <span class="ambig-score" style="color: {ambigScoreColor};">
          {ambig.score.toFixed(4)}
        </span>
        <span class="ambig-threshold">≤ {ambig.threshold.toFixed(2)}</span>
        <Badge tone={ambig.qualifies_for_advance ? 'teal' : 'ghost'} size="xs">
          {ambig.milestone}
        </Badge>
      </div>
      <table class="ambig-components">
        <thead>
          <tr>
            <th>dimension</th>
            <th>clarity</th>
            <th>weight</th>
            <th>floor</th>
          </tr>
        </thead>
        <tbody>
          {#each ambigRows as row (row.name)}
            {#if row.c}
              <tr class:floor-fail={row.c.score < row.c.floor}>
                <td>{row.name}</td>
                <td>
                  <div class="ambig-bar">
                    <div
                      class="ambig-bar-fill"
                      style="width: {ambigPct(row.c.score)};"
                    ></div>
                    <span>{ambigPct(row.c.score)}</span>
                  </div>
                </td>
                <td>{ambigPct(row.c.weight)}</td>
                <td>{ambigPct(row.c.floor)}</td>
              </tr>
            {/if}
          {/each}
        </tbody>
      </table>
      {#if ambig.floor_failures.length > 0}
        <p class="ambig-floor-warning">
          floors breached: {ambig.floor_failures.join(', ')}
        </p>
      {/if}
      <div class="ambig-streak">
        <span>streak</span>
        <strong>{ambig.streak.current}/{ambig.streak.required}</strong>
        <em>
          {#if ambig.qualifies_for_advance}
            ready to advance
          {:else}
            needs more readiness
          {/if}
        </em>
      </div>
    {/if}
  </Card>

  <!-- 2c. STAGNATION (C-10 anti-spiral primitives) — mirrors the AMBIGUITY
       card layout. Sources: get_stagnation_events Tauri command flattens
       8 SsbActionType variants into a single DTO. Badge tones reuse the
       existing GatesPanel.svelte tokens (warn=amber, teal=lateral,
       fail=red); no new color tokens introduced. -->
  <Card variant="plain" padding={12}>
    <div class="section-label">STAGNATION</div>
    {#if stagError}
      <p class="stag-error">{stagError}</p>
    {:else if stagEvents.length === 0}
      <p class="stag-empty">no anti-spiral events</p>
    {:else}
      <div class="stag-rows">
        {#each stagEvents.slice(0, 8) as ev}
          <div class="stag-row">
            <span class="stag-ts">{fmtTs(ev.timestamp_ms)}</span>
            <Badge tone={stagBadgeTone(ev.action_type)} size="xs">{stagBadgeLabel(ev.action_type)}</Badge>
            {#if ev.pattern}
              <span class="stag-tag">{ev.pattern}</span>
            {/if}
            {#if ev.persona}
              <span class="stag-tag">{ev.persona}</span>
            {/if}
            {#if ev.drift_score !== null && ev.drift_score !== undefined}
              <span class="stag-tag">j={ev.drift_score.toFixed(2)}</span>
            {/if}
            {#if ev.elapsed_ms !== null && ev.elapsed_ms !== undefined}
              <span class="stag-tag">{Math.round(ev.elapsed_ms / 1000)}s</span>
            {/if}
            {#if ev.detail}
              <span class="stag-detail">{ev.detail.slice(0, 60)}{ev.detail.length > 60 ? '...' : ''}</span>
            {/if}
          </div>
        {/each}
      </div>
    {/if}
  </Card>

  <!-- 3. AGENTS -->
  <Card variant="plain" padding={12}>
    <div class="section-label">AGENTS</div>
    <div class="agent-rows">
      {#each d.agents as agent}
        <div class="agent-row">
          <Badge tone="ghost" size="xs">{agent.role}</Badge>
          <Badge tone={agentLifeTone(agent.lifecycle)} size="xs">{agentLifeLabel(agent.lifecycle)}</Badge>
          <div class="agent-budget-bar" title="GRC budget {agent.grc_budget_used_pct}%">
            <div class="agent-budget-fill" style="width: {agent.grc_budget_used_pct}%;"></div>
          </div>
          <span class="agent-hb">{Math.round(agent.last_heartbeat_ms / 1000)}s</span>
        </div>
      {/each}
    </div>
  </Card>

  <!-- 4. SUPERVISION -->
  <Card variant="plain" padding={12}>
    <div class="section-label">SUPERVISION</div>
    <div class="supervision-rows">
      {#each recentSupervision as ev}
        <div class="supervision-row">
          <span class="supervision-ts">{fmtTs(ev.ts)}</span>
          <Badge tone={supervisionTone(ev.event)} size="xs">{ev.event}</Badge>
          <span class="supervision-agent">{ev.agent}</span>
          {#if ev.reason}
            <span class="supervision-reason">{ev.reason.slice(0, 40)}</span>
          {/if}
        </div>
      {/each}
    </div>
  </Card>

  <!-- 5. BSP SUPERSTEPS -->
  <Card variant="plain" padding={12}>
    <div class="section-label">BSP SUPERSTEPS</div>
    <div class="superstep-row">
      {#each supersteps as s}
        <div
          class="superstep-dot"
          class:superstep-dot--teal={s.status === 'PASS'}
          class:superstep-dot--warn={s.status === 'SKIP'}
          class:superstep-dot--fail={s.status === 'FAIL'}
          use:tooltip={{ label: `#${s.id} ${s.baby_step_id} ${s.block} ${s.duration_ms}ms` }}
        ></div>
      {/each}
    </div>
  </Card>

  <!-- 6. SANDBOX TIER -->
  {#if sandboxStatus}
    <Card variant="plain" padding={12}>
      <div class="section-label">SANDBOX</div>
      <div class="sandbox-tier-row">
        <Badge tone={sandboxTierTone(sandboxStatus.tier)} size="sm">{sandboxStatus.tier}</Badge>
        <span class="sandbox-profiles">{sandboxStatus.profile_count} profiles</span>
        {#if sandboxStatus.last_visibility_filter_hit_ms !== null}
          <span class="sandbox-vf-hit" use:tooltip={{ label: 'Last visibility filter hit' }}>{sandboxStatus.last_visibility_filter_hit_ms}ms</span>
        {/if}
      </div>
    </Card>
  {/if}

  <!-- 7. CONTEXT BUDGET -->
  <Card variant="plain" padding={12}>
    <div class="section-label">CONTEXT BUDGET</div>
    <div class="ctx-used">{d.contextBudget.total_used.toLocaleString()} / {d.contextBudget.total_cap.toLocaleString()} tokens ({totalPct}%)</div>
    <div class="ctx-bar-wrap">
      <div class="ctx-bar">
        {#each slots as s}
          <div
            class="ctx-seg"
            style="width: {s.pct}%; background: {s.color};"
            title="{s.displayName}: {s.used_tokens} tokens ({s.pct}%)"
          ></div>
        {/each}
      </div>
      <div class="ctx-breakpoints">
        <div class="ctx-bp" style="left: {bp1Pct}%;" title="Cache breakpoint 1"></div>
        <div class="ctx-bp" style="left: {bp2Pct}%;" title="Cache breakpoint 2"></div>
      </div>
    </div>
    <div class="ctx-legend">
      {#each slots as s}
        <div class="ctx-legend-item">
          <span class="ctx-dot" style="background: {s.color};"></span>
          <span class="ctx-slot-name">{s.displayName}</span>
          <span class="ctx-slot-pct">{s.pct}%</span>
        </div>
      {/each}
    </div>
  </Card>

  <!-- C-45: CANTANTE contrastive credit attribution table.
       Paper-grounded against arXiv:2605.13295 (Zehle 2026, Section 3.2).
       The current rows reflect the synthetic-fixture harness; real
       credits land via the runpod-pending pilot. -->
  <Card variant="plain" padding={12}>
    <div class="section-label">CREDIT ATTRIBUTION (CANTANTE)</div>
    {#if creditError}
      <p class="ambig-error">{creditError}</p>
    {:else if !creditSnapshot}
      <p class="ambig-loading">Loading…</p>
    {:else}
      <div class="credit-meta">
        <span class="credit-ts">ts {creditSnapshot.ts_ms}</span>
        {#if creditSnapshot.runpod_pending}
          <Badge tone="warn" size="xs">runpod-pending</Badge>
        {:else}
          <Badge tone="teal" size="xs">live</Badge>
        {/if}
      </div>
      {#if creditSnapshot.rows.length === 0}
        <p class="ambig-loading">No credit rows yet.</p>
      {:else}
        <table class="credit-table">
          <thead>
            <tr>
              <th>role</th>
              <th>credit</th>
            </tr>
          </thead>
          <tbody>
            {#each creditSnapshot.rows as row (row.role)}
              <tr>
                <td>{row.role}</td>
                <td>
                  <Badge tone={creditScoreTone(row.score)} size="xs">
                    {row.score >= 0 ? '+' : ''}{row.score.toFixed(3)}
                  </Badge>
                </td>
              </tr>
            {/each}
          </tbody>
        </table>
      {/if}
    {/if}
  </Card>

  <!-- C-46: Uno-Orchestra parsimonious agent routing (selective
       delegation) decisions table. Paper-grounded against
       arXiv:2605.05007 (Cui et al 2026, Sections 5 + 6.1). The current
       rows reflect the synthetic-pilot harness; real decisions land
       via the runpod-pending pilot. -->
  <Card variant="plain" padding={12}>
    <div class="section-label">ROUTING DECISIONS (UNO-ORCHESTRA)</div>
    {#if routingError}
      <p class="ambig-error">{routingError}</p>
    {:else if !routingSnapshot}
      <p class="ambig-loading">Loading…</p>
    {:else}
      <div class="credit-meta">
        <span class="credit-ts">ts {routingSnapshot.ts_ms}</span>
        {#if routingSnapshot.runpod_pending}
          <Badge tone="warn" size="xs">runpod-pending</Badge>
        {:else}
          <Badge tone="teal" size="xs">live</Badge>
        {/if}
        <span class="credit-ts">
          alpha={routingSnapshot.policy.alpha.toFixed(2)} s_max={routingSnapshot.policy.s_max.toFixed(2)}
        </span>
      </div>
      {#if routingSnapshot.rows.length === 0}
        <p class="ambig-loading">No routing decisions yet.</p>
      {:else}
        <table class="credit-table">
          <thead>
            <tr>
              <th>task</th>
              <th>difficulty</th>
              <th>dispatched</th>
              <th>mode</th>
            </tr>
          </thead>
          <tbody>
            {#each routingSnapshot.rows as row (row.task_id)}
              <tr>
                <td>{row.task_id}</td>
                <td>
                  <Badge tone={difficultyTone(row.difficulty)} size="xs">
                    {row.difficulty.toFixed(3)}
                  </Badge>
                </td>
                <td>
                  {#if row.dispatched.length === 0}
                    <span class="ambig-loading">none</span>
                  {:else}
                    {row.dispatched.length}
                  {/if}
                </td>
                <td>
                  {#if row.is_lazy_mode}
                    <Badge tone="teal" size="xs">lazy-mode</Badge>
                  {:else if row.dispatched.length === 0}
                    <Badge tone="ghost" size="xs">disabled</Badge>
                  {:else}
                    <Badge tone="warn" size="xs">dispatch</Badge>
                  {/if}
                </td>
              </tr>
            {/each}
          </tbody>
        </table>
      {/if}
    {/if}
  </Card>

</div>

<style>
  .orchestration-drawer {
    display: flex;
    flex-direction: column;
    gap: 8px;
  }

  .section-label {
    font-family: var(--font-mono);
    font-size: 10px;
    color: var(--text-secondary);
    letter-spacing: 0.08em;
    text-transform: uppercase;
    margin-bottom: 8px;
  }

  /* FSM */
  .fsm-current {
    display: flex;
    align-items: center;
    gap: 8px;
    margin-bottom: 8px;
  }

  .fsm-sub {
    font-family: var(--font-mono);
    font-size: 10px;
    color: var(--text-ghost);
  }

  .fsm-chain {
    display: flex;
    align-items: center;
    flex-wrap: wrap;
    gap: 3px;
  }

  .chain-state {
    font-family: var(--font-mono);
    font-size: 10px;
    color: var(--text-ghost);
    animation: chainFade 600ms var(--ease-out) both;
  }

  .chain-state--current {
    color: var(--text-secondary);
  }

  .chain-arrow {
    font-family: var(--font-mono);
    font-size: 10px;
    color: var(--text-ghost);
  }

  @keyframes chainFade {
    from { opacity: 0.3; }
    to   { opacity: 1; }
  }

  /* C-2: Ambiguity gate section (mirrors GatesPanel.svelte layout) */
  .ambig-error,
  .ambig-loading {
    font-family: var(--font-mono);
    font-size: 10px;
    color: var(--text-ghost);
    margin: 4px 0;
  }

  .ambig-overall {
    display: flex;
    align-items: center;
    gap: 8px;
    margin-bottom: 6px;
  }

  .ambig-score-label {
    font-family: var(--font-mono);
    font-size: 10px;
    color: var(--text-ghost);
    text-transform: uppercase;
  }

  .ambig-score {
    font-family: var(--font-mono);
    font-size: 14px;
    font-weight: 600;
  }

  .ambig-threshold {
    font-family: var(--font-mono);
    font-size: 10px;
    color: var(--text-ghost);
  }

  .ambig-components {
    width: 100%;
    border-collapse: collapse;
    font-family: var(--font-mono);
    font-size: 10px;
    margin-bottom: 6px;
  }

  .ambig-components th {
    text-align: left;
    color: var(--text-ghost);
    text-transform: uppercase;
    font-weight: 400;
    padding: 2px 4px;
  }

  .ambig-components td {
    padding: 2px 4px;
    color: var(--text-primary);
  }

  .ambig-components tr.floor-fail {
    background: rgba(176, 69, 69, 0.08);
  }

  .ambig-bar {
    position: relative;
    width: 100%;
    height: 10px;
    background: rgba(255, 255, 255, 0.04);
    border-radius: 2px;
    overflow: hidden;
  }

  .ambig-bar-fill {
    position: absolute;
    top: 0;
    left: 0;
    height: 100%;
    background: var(--teal);
    opacity: 0.6;
  }

  .ambig-bar span {
    position: absolute;
    inset: 0;
    display: flex;
    align-items: center;
    justify-content: center;
    color: var(--text-primary);
    font-size: 9px;
  }

  .ambig-floor-warning {
    font-family: var(--font-mono);
    font-size: 10px;
    color: var(--fail);
    margin: 4px 0;
  }

  .ambig-streak {
    display: flex;
    align-items: center;
    gap: 6px;
    font-family: var(--font-mono);
    font-size: 10px;
    color: var(--text-secondary);
  }

  .ambig-streak strong {
    color: var(--text-primary);
    font-weight: 600;
  }

  .ambig-streak em {
    color: var(--text-ghost);
    font-style: normal;
  }

  /* C-10: STAGNATION section — reuse the agent-row / supervision-row
     visual cadence (no new tokens). */
  .stag-error,
  .stag-empty {
    font-family: var(--font-mono);
    font-size: 10px;
    color: var(--text-ghost);
    margin: 4px 0;
  }

  .stag-rows {
    display: flex;
    flex-direction: column;
    gap: 3px;
  }

  .stag-row {
    display: flex;
    align-items: center;
    gap: 6px;
    flex-wrap: wrap;
    font-family: var(--font-mono);
    font-size: 10px;
    color: var(--text-secondary);
  }

  .stag-ts {
    color: var(--text-ghost);
    font-size: 9px;
  }

  .stag-tag {
    color: var(--text-primary);
    font-size: 10px;
  }

  .stag-detail {
    color: var(--text-ghost);
    font-size: 10px;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  /* Evidence gates */
  .gate-rows {
    display: flex;
    flex-direction: column;
    gap: 4px;
  }

  .gate-row {
    display: flex;
    align-items: flex-start;
    gap: 6px;
    padding: 3px 4px;
    border-radius: var(--r-xs);
    flex-wrap: wrap;
  }

  .gate-row--block {
    background: rgba(176, 69, 69, 0.06);
  }

  .gate-name {
    font-family: var(--font-mono);
    font-size: 10px;
    color: var(--text-primary);
    flex: 1;
    min-width: 120px;
  }

  .gate-reason {
    font-family: var(--font-mono);
    font-size: 10px;
    color: var(--text-ghost);
    width: 100%;
  }

  /* Agents */
  .agent-rows {
    display: flex;
    flex-direction: column;
    gap: 5px;
  }

  .agent-row {
    display: flex;
    align-items: center;
    gap: 6px;
  }

  .agent-budget-bar {
    flex: 1;
    height: 4px;
    background: var(--bg-input);
    border-radius: var(--r-xs);
    overflow: hidden;
    min-width: 0;
  }

  .agent-budget-fill {
    height: 100%;
    background: var(--teal-bright);
    border-radius: var(--r-xs);
    transition: width var(--dur-base) var(--ease-out);
  }

  .agent-hb {
    font-family: var(--font-mono);
    font-size: 10px;
    color: var(--text-ghost);
    flex-shrink: 0;
    width: 28px;
    text-align: right;
  }

  /* Supervision */
  .supervision-rows {
    display: flex;
    flex-direction: column;
    gap: 4px;
  }

  .supervision-row {
    display: flex;
    align-items: center;
    gap: 6px;
    flex-wrap: wrap;
  }

  .supervision-ts {
    font-family: var(--font-mono);
    font-size: 10px;
    color: var(--text-ghost);
    flex-shrink: 0;
  }

  .supervision-agent {
    font-family: var(--font-mono);
    font-size: 10px;
    color: var(--text-secondary);
  }

  .supervision-reason {
    font-family: var(--font-mono);
    font-size: 10px;
    color: var(--text-ghost);
    width: 100%;
  }

  /* BSP Supersteps */
  .superstep-row {
    display: flex;
    align-items: center;
    gap: 6px;
    flex-wrap: wrap;
  }

  .superstep-dot {
    width: 10px;
    height: 10px;
    border-radius: 50%;
    cursor: default;
    flex-shrink: 0;
  }

  .superstep-dot--teal { background: var(--teal-bright); }
  .superstep-dot--warn { background: var(--warn); }
  .superstep-dot--fail { background: var(--fail); }

  /* Context budget */
  .ctx-used {
    font-family: var(--font-mono);
    font-size: 10px;
    color: var(--text-secondary);
    margin-bottom: 6px;
  }

  .ctx-bar-wrap {
    position: relative;
    margin-bottom: 4px;
  }

  .ctx-bar {
    display: flex;
    width: 100%;
    height: 8px;
    border-radius: var(--r-xs);
    overflow: hidden;
    background: var(--bg-input);
    gap: 1px;
  }

  .ctx-seg {
    height: 100%;
    min-width: 1px;
  }

  .ctx-breakpoints {
    position: relative;
    height: 8px;
    margin-top: 2px;
  }

  .ctx-bp {
    position: absolute;
    top: 0;
    width: 0;
    height: 8px;
    border-left: 1px dashed var(--text-ghost);
    transform: translateX(-0.5px);
  }

  .ctx-legend {
    display: flex;
    flex-direction: column;
    gap: 2px;
    margin-top: 6px;
  }

  .ctx-legend-item {
    display: flex;
    align-items: center;
    gap: 5px;
  }

  .ctx-dot {
    width: 6px;
    height: 6px;
    border-radius: 1px;
    flex-shrink: 0;
  }

  .ctx-slot-name {
    font-family: var(--font-mono);
    font-size: 10px;
    color: var(--text-ghost);
    flex: 1;
  }

  .ctx-slot-pct {
    font-family: var(--font-mono);
    font-size: 10px;
    color: var(--text-ghost);
  }

  /* Sandbox tier badge (B-18) */
  .sandbox-tier-row {
    display: flex;
    align-items: center;
    gap: 8px;
  }

  .sandbox-profiles {
    font-family: var(--font-mono);
    font-size: 10px;
    color: var(--text-ghost);
  }

  .sandbox-vf-hit {
    font-family: var(--font-mono);
    font-size: 10px;
    color: var(--text-ghost);
  }

  /* C-45: CANTANTE contrastive credit-attribution table styles. */
  .credit-meta {
    display: flex;
    gap: 8px;
    align-items: center;
    font-family: var(--font-mono);
    font-size: 10px;
    color: var(--text-ghost);
    margin-bottom: 6px;
  }
  .credit-ts {
    color: var(--text-ghost);
  }
  .credit-table {
    width: 100%;
    border-collapse: collapse;
    font-family: var(--font-mono);
    font-size: 11px;
  }
  .credit-table th {
    text-align: left;
    font-weight: normal;
    color: var(--text-ghost);
    padding-bottom: 4px;
  }
  .credit-table td {
    padding: 2px 0;
  }
</style>
