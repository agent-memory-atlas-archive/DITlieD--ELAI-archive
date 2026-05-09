<script lang="ts">
  import { onMount } from 'svelte';
  import { orchestrationData } from '../state/drawers.svelte';
  import { Card, Badge, Button } from '$lib/ui';
  import { logStub, get_sandbox_status } from '../api/stub';
  import { tooltip } from '$lib/ui/tooltip';
  import type { FsmState, GateVerdict, AgentLifecycle, BspSuperstep, SandboxStatus, SandboxTier } from '../api/types';

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

  function sandboxTierTone(tier: SandboxTier): string {
    if (tier === 'C-Windows' || tier === 'C-Linux') return 'teal';
    if (tier === 'A++') return 'warn';
    return 'ghost';
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

</div>

<style>
  .orchestration-drawer {
    display: flex;
    flex-direction: column;
    gap: 8px;
  }

  .section-label {
    font-family: var(--font-mono);
    font-size: 11px;
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
    font-size: 11px;
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
    font-size: 11px;
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
</style>
