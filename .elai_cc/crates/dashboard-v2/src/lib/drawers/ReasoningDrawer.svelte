<script lang="ts">
  import { onMount } from 'svelte';
  import { reasoningData } from '../state/drawers.svelte';
  import { Card, Badge, Button, UnwiredNote } from '$lib/ui';
  import { logStub } from '../api/stub';
  import FeasibilityPanel from '$lib/components/FeasibilityPanel.svelte';
  import ArchiveVerifyGauge from '$lib/components/ArchiveVerifyGauge.svelte';
  import TripwireStatusStrip from '$lib/components/TripwireStatusStrip.svelte';
  // B-5: red intervention card for unguarded Critical CSQ verdicts.
  import CsqBlockCard from '$lib/components/CsqBlockCard.svelte';
  // B-5: per-role GRC config table (get_agent_grc_config consumer).
  // C-3: raw feasibility-report log consumer (get_feasibility_log).
  import {
    get_agent_grc_config,
    get_feasibility_log,
    type FeasibilityLogRow,
  } from '../api/stub';
  import type { GrcConfigRow } from '../api/types';

  // B-39D: live REFLECT pipeline timeline. Subscribes to the singleton
  // `reflect-progress` Tauri event emitted by TauriEventSink. Each stage
  // pushes a stamped row; we keep the last 12. Turn boundary is detected
  // by stream_id change — when a new stream_id appears, clear the buffer.
  type ReflectStage =
    | { stage: 'triage'; ts: number; stream_id: string; class: string }
    | { stage: 'candidate'; ts: number; stream_id: string; candidate_index: number; content_preview: string }
    | { stage: 'audit_verdict'; ts: number; stream_id: string; best_index: number; rationale_preview: string }
    | { stage: 'refined'; ts: number; stream_id: string; total_tokens_used: number; synthesis_preview: string };

  let reflectTimeline = $state<ReflectStage[]>([]);
  let reflectCurrentStream = $state<string | null>(null);

  // C-32 MINI-CRITIC -- per-blank verdict rows. Subscribes to the
  // `mini_critic_verdict` SSB event surface (mounted alongside the existing
  // SSB activity listener that the dashboard already plumbs). Buffer is
  // bounded to the last 24 rows so a long-running session does not run the
  // drawer out of memory. Toggle defaults to ON per workflow/reasoning/
  // C-32-decisions.md and plan §Task 7.
  type MiniCriticRow = {
    ts: number;
    blank_id: number;
    verdict: 'pass' | 'warn' | 'block';
    reason: string;
    language: string;
  };
  let miniCriticRows = $state<MiniCriticRow[]>([]);
  let showMiniCritic = $state(true);

  function miniCriticVerdictTone(v: 'pass' | 'warn' | 'block'): 'teal' | 'warn' | 'fail' {
    switch (v) {
      case 'pass': return 'teal';
      case 'warn': return 'warn';
      case 'block': return 'fail';
    }
  }

  onMount(() => {
    reasoningData.bootstrap();

    let unlistenReflect: (() => void) | null = null;
    let unlistenMiniCritic: (() => void) | null = null;
    (async () => {
      try {
        const { listen } = await import('@tauri-apps/api/event');
        unlistenReflect = await listen<ReflectStage>('reflect-progress', (e) => {
          const incoming = e.payload as ReflectStage & { stream_id?: string };
          const sid = incoming.stream_id ?? '';
          // Turn boundary: new stream_id resets the timeline.
          if (reflectCurrentStream !== sid) {
            reflectCurrentStream = sid;
            reflectTimeline = [];
          }
          reflectTimeline = [
            ...reflectTimeline,
            { ...incoming, ts: Date.now() } as ReflectStage,
          ].slice(-12);
        });
      } catch (err) {
        console.debug('[ReasoningDrawer] reflect-progress listen failed', err);
      }

      // C-32: subscribe to mini_critic_verdict SSB events. Backend wires
      // these through the existing ssb-event broadcast channel; the
      // dashboard receives a serialized MiniCriticVerdictPayload.
      try {
        const { listen } = await import('@tauri-apps/api/event');
        unlistenMiniCritic = await listen<MiniCriticRow>('mini-critic-verdict', (e) => {
          const incoming = e.payload as MiniCriticRow;
          miniCriticRows = [
            ...miniCriticRows,
            { ...incoming, ts: Date.now() },
          ].slice(-24);
        });
      } catch (err) {
        console.debug('[ReasoningDrawer] mini-critic-verdict listen failed', err);
      }
    })();

    return () => {
      try { unlistenReflect?.(); } catch (err) { console.debug('reflect unlisten', err); }
      try { unlistenMiniCritic?.(); } catch (err) { console.debug('mini-critic unlisten', err); }
      reasoningData.teardown();
    };
  });

  function reflectStageLabel(s: ReflectStage): string {
    switch (s.stage) {
      case 'triage':        return `triage → ${s.class}`;
      case 'candidate':     return `reasoner #${s.candidate_index}`;
      case 'audit_verdict': return `auditor → pick #${s.best_index}`;
      case 'refined':       return `synthesize · ${s.total_tokens_used}t`;
    }
  }

  function reflectStageDetail(s: ReflectStage): string {
    switch (s.stage) {
      case 'triage':        return '';
      case 'candidate':     return s.content_preview;
      case 'audit_verdict': return s.rationale_preview;
      case 'refined':       return s.synthesis_preview;
    }
  }

  function reflectStageTone(s: ReflectStage): 'teal' | 'steel' | 'warn' | 'neutral' {
    switch (s.stage) {
      case 'triage':        return 'steel';
      case 'candidate':     return 'neutral';
      case 'audit_verdict': return 'warn';
      case 'refined':       return 'teal';
    }
  }

  const d = reasoningData;

  // B-5: per-role GRC config table. Sourced from get_agent_grc_config
  // (compile-time default_grc_config table, NOT SSB). Collapsible so it
  // does not crowd the live budget bars above. Loaded once on mount —
  // the config is static for the process lifetime.
  let grcConfig = $state<GrcConfigRow[]>([]);
  let showGrcConfig = $state(false);

  onMount(() => {
    void (async () => {
      try {
        grcConfig = await get_agent_grc_config();
      } catch (err) {
        console.debug('[ReasoningDrawer] get_agent_grc_config failed', err);
      }
    })();
  });

  // C-3: raw feasibility-report log (every feasibility_report SSB row, not
  // just the v2 PASS/FAIL summary the FeasibilityPanel renders). Polled on a
  // 5s cadence; falls back to [] in browser-dev mode. Bounded by the backend
  // FEASIBILITY_LOG_QUERY_LIMIT (20 rows).
  let feasibilityLog = $state<FeasibilityLogRow[]>([]);
  let showFeasibilityLog = $state(false);
  let feasLogTimer: ReturnType<typeof setInterval> | null = null;

  onMount(() => {
    async function refreshFeasLog(): Promise<void> {
      try {
        feasibilityLog = await get_feasibility_log();
      } catch (err) {
        console.debug('[ReasoningDrawer] get_feasibility_log failed', err);
      }
    }
    void refreshFeasLog();
    feasLogTimer = setInterval(() => void refreshFeasLog(), 5_000);
    return () => {
      if (feasLogTimer) {
        clearInterval(feasLogTimer);
        feasLogTimer = null;
      }
    };
  });

  function feasVerdictTone(v: string): 'teal' | 'warn' | 'fail' {
    const lv = v.toLowerCase();
    if (lv === 'pass' || lv === 'feasible') return 'teal';
    if (lv === 'fail' || lv === 'infeasible' || lv === 'blocked') return 'fail';
    return 'warn';
  }

  function callsPct(used: number, total: number): number {
    return total > 0 ? Math.round((used / total) * 100) : 0;
  }

  function charsPct(used: number, total: number): number {
    return total > 0 ? Math.round((used / total) * 100) : 0;
  }

  function fmtTs(ts: number): string {
    return new Date(ts).toISOString().slice(11, 19);
  }

  // C-22: feasibilityTone / feasOutcomes / feasRows / hasFeasResults removed —
  // the dedicated FeasibilityPanel component now owns the rendering logic.

  // CSQ entries
  const hasCsqs = $derived(d.csqPending.length > 0);

  // B-5 GAP-1 — anti-rubber-stamp rejection count. The counter badge
  // renders "Auditor: N rubber stamps rejected this session" from the
  // reasoningData.auditorRubberStampsRejected feed (10s poll).
  const rubberStampRejectCount = $derived(d.auditorRubberStampsRejected.length);

  // F-8 / Slice A5 — held-pending SCF symbolic conflicts.
  const hasSymbolicConflicts = $derived(d.symbolicConflicts.length > 0);

  // F-11 — config-DGM admit-loop rows (Admitted / RejectedByGuard / LoopClosed).
  const hasDarwinianAdmits = $derived(d.darwinianAdmits.length > 0);

  // H-43 — execution-grounded DGM fitness rows (FitnessEvaluated per candidate).
  // Wire-dark: same gate as darwinianAdmits (darwinian-evolve + ELAI_DARWINIAN_LOOP_ENABLE=1).
  const hasFitnessRows = $derived(
    Array.isArray((d as any).fitnessEvaluated) && (d as any).fitnessEvaluated.length > 0
  );

  // H-46 — CMP parent-selection rows (CladeSelected per selection event).
  // Wire-dark: same gate as darwinianAdmits (darwinian-evolve + ELAI_DARWINIAN_LOOP_ENABLE=1).
  const hasCladeSelections = $derived(d.cladeSelections.length > 0);

  // Badge tone per admit kind: loop_closed=teal (success), rejected_by_guard=
  // warn, admitted=steel (neutral/info). Tones constrained to the Badge
  // component's set ('teal' | 'steel' | 'warn' | 'fail' | 'ghost').
  function darwinianAdmitTone(kind: string): 'teal' | 'warn' | 'steel' {
    switch (kind) {
      case 'loop_closed': return 'teal';
      case 'rejected_by_guard': return 'warn';
      default: return 'steel';
    }
  }

  // H-43: Badge tone per fitness kind: fitness_regression=warn (candidate regressed),
  // execution_pass_rate=teal (causal signal), dual_agreement=steel (agreement mass).
  function fitnessTone(kind: string): 'teal' | 'warn' | 'steel' {
    switch (kind) {
      case 'fitness_regression': return 'warn';
      case 'execution_pass_rate': return 'teal';
      default: return 'steel';
    }
  }

  // H-46: Badge tone per clade selection key:
  // clade_metaproductivity=teal (CMP override), immediate_score_fallback=steel (no lineage yet).
  function cladeSelectionTone(kind: string): 'teal' | 'steel' {
    return kind === 'clade_metaproductivity' ? 'teal' : 'steel';
  }

  // GG signals as ordered tiles
  const ggTiles = $derived([
    { label: 'mean_logprob', value: d.ggSignals.mean_logprob.toFixed(3) },
    { label: 'min_logprob',  value: d.ggSignals.min_logprob.toFixed(3) },
    { label: 'entropy_mean', value: d.ggSignals.entropy_mean.toFixed(3) },
    { label: 'entropy_max',  value: d.ggSignals.entropy_max.toFixed(3) },
  ]);

  // REFLECT derived display values
  const reflectEnabled = $derived(d.reflectState.enabled);
  const reflectTriageClass = $derived(d.reflectState.last_triage?.class ?? '—');
  const reflectTriageTurnId = $derived(d.reflectState.last_triage?.turn_id ?? null);
  const reflectVerdictIndex = $derived(
    d.reflectState.last_verdict === null ? null : d.reflectState.last_verdict.best_index
  );
  const reflectVerdictRationale = $derived(
    d.reflectState.last_verdict === null
      ? '—'
      : d.reflectState.last_verdict.rationale.slice(0, 80)
  );
  const reflectSummaryTokens = $derived(
    d.reflectState.last_summary === null ? null : d.reflectState.last_summary.total_tokens_used
  );
  const reflectSummarySynthesis = $derived(
    d.reflectState.last_summary === null
      ? '—'
      : d.reflectState.last_summary.synthesis.slice(0, 100)
  );
</script>

<div class="reasoning-drawer">

  <!-- D-26 GAP-1/GAP-2: archive verify gauge + tripwire status strip.
       Inline widgets (not drawers) — they consume read-only Tauri
       commands and self-poll. Mounted at the top so they act as a
       header-level health glance for the archive substrate. -->
  <ArchiveVerifyGauge />
  <TripwireStatusStrip />

  <!-- B-5: CSQ BLOCK intervention card. Self-renders (returns nothing when
       no unguarded Critical CSQ verdict is active). Mounted at the top so a
       block is the most prominent thing in the drawer when it fires. -->
  <CsqBlockCard />

  <!-- 1. GRC BUDGETS PER ROLE -->
  <Card variant="plain" padding={12}>
    <div class="section-label">GRC BUDGETS PER ROLE</div>
    <div class="grc-rows">
      {#each d.grcBudgets as b}
        <div class="grc-row">
          <span class="grc-role">{b.role}</span>
          <div class="grc-bars">
            <div class="mini-bar-wrap" title="calls">
              <div class="mini-bar">
                <div class="mini-bar-fill mini-bar-fill--teal" style="width: {callsPct(b.calls_used, b.total_calls)}%;"></div>
              </div>
            </div>
            <div class="mini-bar-wrap" title="chars">
              <div class="mini-bar">
                <div class="mini-bar-fill mini-bar-fill--steel" style="width: {charsPct(b.chars_used, b.total_chars)}%;"></div>
              </div>
            </div>
          </div>
          <span class="grc-stat">{b.calls_used}/{b.total_calls} calls · {b.chars_used}/{b.total_chars} chars</span>
        </div>
      {/each}
    </div>

    <!-- B-5: per-role GRC CONFIG table (get_agent_grc_config). Collapsible.
         Distinct from the live budget bars above: this is the configured
         per-role max_calls / max_chars / GRC topic key, read once from the
         compile-time default_grc_config table. -->
    <div class="grc-config">
      <button
        class="grc-config-toggle"
        type="button"
        onclick={() => { showGrcConfig = !showGrcConfig; }}
        aria-expanded={showGrcConfig}
      >
        <span class="grc-config-caret">{showGrcConfig ? '▾' : '▸'}</span>
        PER-ROLE GRC CONFIG ({grcConfig.length})
      </button>
      {#if showGrcConfig}
        {#if grcConfig.length === 0}
          <span class="ghost-text">no GRC config (browser-dev mode — get_agent_grc_config returns [] without a Tauri runtime)</span>
        {:else}
          <div class="grc-config-table">
            <div class="grc-config-header">
              <span>role</span>
              <span>max calls</span>
              <span>max chars</span>
              <span>topic</span>
            </div>
            {#each grcConfig as c (c.role)}
              <div class="grc-config-row">
                <span class="grc-config-role">{c.role}</span>
                <span class="grc-config-cell">{c.max_calls}</span>
                <span class="grc-config-cell">{c.max_chars}</span>
                <span class="grc-config-topic mono">{c.topic_prefix}</span>
              </div>
            {/each}
          </div>
        {/if}
      {/if}
    </div>
  </Card>

  <!-- 2. GROUNDING SWEEP -->
  <Card variant="plain" padding={12}>
    <div class="section-label">GROUNDING SWEEP</div>
    {#if d.unwired.groundingSweep}
      <UnwiredNote
        phase="B-17"
        feature="GROUNDED / VERIFY / UNGROUNDED claim counts + ungrounded list"
        note="get_grounding_sweep returns hardcoded zeros; GroundingSweepGate producer is not wired"
      />
    {:else}
      <div class="sweep-tiles">
        <div class="sweep-tile">
          <span class="sweep-count sweep-count--teal">{d.groundingSweep.grounded}</span>
          <span class="sweep-label">GROUNDED</span>
        </div>
        <div class="sweep-tile">
          <span class="sweep-count sweep-count--warn">{d.groundingSweep.verify_needed}</span>
          <span class="sweep-label">VERIFY</span>
        </div>
        <div class="sweep-tile">
          <span class="sweep-count sweep-count--fail">{d.groundingSweep.ungrounded}</span>
          <span class="sweep-label">UNGROUNDED</span>
        </div>
      </div>
      <div class="sweep-ungrounded-list">
        {#each d.groundingSweep.claims.filter(c => c.classification === 'UNGROUNDED').slice(0, 5) as claim}
          <div class="sweep-claim">{claim.text}</div>
        {/each}
      </div>
    {/if}
  </Card>

  <!-- 3. CSQ CRITICAL PENDING -->
  <Card variant={hasCsqs ? 'error' : 'plain'} padding={12}>
    <div class="section-label">CSQ CRITICAL PENDING</div>
    {#if d.unwired.csqPending}
      <UnwiredNote
        phase="B-17"
        feature="CRITICAL/WARNING CSQ entries with [Add guard] / [Dismiss] actions"
        note="get_csq_pending returns []; CSQ enforcement not wired in pre-tool dispatch"
      />
    {:else if hasCsqs}
      <div class="csq-list">
        {#each d.csqPending as csq}
          <div class="csq-entry">
            <Badge tone="fail" size="xs">{csq.severity}</Badge>
            <span class="csq-question">{csq.question}</span>
            <div class="csq-actions">
              <Button
                variant="ghost"
                size="sm"
                onclick={() => logStub('csq.add-guard', { id: csq.id })}
              >[Add guard]</Button>
              <Button
                variant="text"
                size="sm"
                onclick={() => logStub('csq.dismiss', { id: csq.id })}
              >[Dismiss]</Button>
            </div>
          </div>
        {/each}
      </div>
    {:else}
      <span class="ghost-text">no pending CSQs</span>
    {/if}
  </Card>

  <!-- 3a. B-5 GAP-1 ANTI-RUBBER-STAMP COUNTER — surfaces the count of
       auditor_rubber_stamp_rejected SSB rows fired by
       run_per_level_audit_hook when a synthetic Allow is recorded for an
       Auditor* tier boundary without any prior FINDING: Verify step.
       The BasicGrcEngine InsufficientFindings enforcement (engine.rs)
       rejects the converge; this counter makes that rejection visible
       in the dashboard. Sourced from reasoningData.auditorRubberStampsRejected
       (get_auditor_rubber_stamp_rejections, 10s poll). Empty = steady
       state (no rubber-stamp rejections this session). -->
  <Card variant={rubberStampRejectCount > 0 ? 'error' : 'plain'} padding={12}>
    <div class="section-label">AUDITOR ANTI-RUBBER-STAMP</div>
    <div class="rubber-stamp-counter">
      <Badge
        tone={rubberStampRejectCount > 0 ? 'fail' : 'steel'}
        size="sm"
      >{rubberStampRejectCount}</Badge>
      <span class="rubber-stamp-label">
        Auditor: {rubberStampRejectCount} rubber stamp{rubberStampRejectCount === 1 ? '' : 's'} rejected this session
      </span>
    </div>
    {#if rubberStampRejectCount > 0}
      <div class="rubber-stamp-list">
        {#each d.auditorRubberStampsRejected as r (r.seq)}
          <div class="rubber-stamp-entry">
            <span class="rubber-stamp-role">{r.role}</span>
            <span class="rubber-stamp-detail">
              {r.finding_count}/{r.expected_min} findings (min not met)
            </span>
          </div>
        {/each}
      </div>
    {/if}
  </Card>

  <!-- 3b. F-8 / Slice A5 SYMBOLIC CONFLICTS — held-pending self-mod config
       variants the SCF symbolic detector flagged against an active locked-key
       invariant (self_mod_safety::LOCKED_KEYS). POLICY is WARN-then-escalate,
       never hard fail-closed BLOCK — these rows are held-pending variants
       awaiting human escalation, NOT rejections. Sourced from the independent
       reasoningData.symbolicConflicts feed (get_symbolic_conflicts, 10s
       poll). Empty in steady state. -->
  <Card variant={hasSymbolicConflicts ? 'error' : 'plain'} padding={12}>
    <div class="section-label">SYMBOLIC CONFLICTS</div>
    {#if hasSymbolicConflicts}
      <div class="conflict-list">
        {#each d.symbolicConflicts as c (c.seq)}
          <div class="conflict-entry">
            <div class="conflict-head">
              <Badge tone="warn" size="xs">HELD-PENDING</Badge>
              <span class="conflict-kind">{c.kind}</span>
              <span class="conflict-key">{c.key}</span>
            </div>
            <span class="conflict-detail">{c.detail}</span>
            <div class="conflict-meta">
              <span class="conflict-mono">{c.candidate_id}</span>
              <span class="conflict-mono">{c.fsm_substate}</span>
              <span class="conflict-mono">{fmtTs(c.ts_ms)}</span>
            </div>
          </div>
        {/each}
      </div>
    {:else}
      <span class="ghost-text">no symbolic conflicts</span>
    {/if}
  </Card>

  <!-- 3b. F-11 DARWINIAN ADMITS — config-DGM admit-loop events. Each row is one
       candidate routed through the four-guard chain (tripwire blake3 fingerprint
       -> trap-pattern scorer -> is_security_sensitive -> Two-Gate):
       Admitted (passed all guards + inserted), RejectedByGuard (a guard fired,
       guard_rejected_by names it), or LoopClosed (select_parent ran after
       insert_variant). Wire-dark producer (the darwin-loop CLI, gated by feature
       darwinian-evolve + ELAI_DARWINIAN_LOOP_ENABLE=1); empty in steady state.
       Sourced from reasoningData.darwinianAdmits (get_darwinian_admits, 10s
       poll). -->
  <Card variant="plain" padding={12}>
    <div class="section-label">DARWINIAN ADMITS</div>
    {#if hasDarwinianAdmits}
      <div class="conflict-list">
        {#each d.darwinianAdmits as a (a.seq)}
          <div class="conflict-entry">
            <div class="conflict-head">
              <Badge tone={darwinianAdmitTone(a.kind)} size="xs">{a.kind}</Badge>
              <span class="conflict-kind">{a.variant_id}</span>
              {#if a.guard_rejected_by}
                <span class="conflict-key">guard: {a.guard_rejected_by}</span>
              {/if}
            </div>
            <span class="conflict-detail">alpha={a.alpha.toFixed(4)}</span>
            <div class="conflict-meta">
              <span class="conflict-mono">{a.candidate_blake3}</span>
              {#if a.parent_id}
                <span class="conflict-mono">parent: {a.parent_id}</span>
              {/if}
              <span class="conflict-mono">{fmtTs(a.ts_ms)}</span>
            </div>
          </div>
        {/each}
      </div>
    {:else}
      <span class="ghost-text">no darwinian admits</span>
    {/if}
  </Card>

  <!-- H-43: DGM FITNESS (execution-grounded pass-rate per candidate).
       Wire-dark: same gate as DARWINIAN ADMITS (darwinian-evolve +
       ELAI_DARWINIAN_LOOP_ENABLE=1). A fitness_regression badge means the
       candidate scored below its parent — it lost promotion. -->
  <Card variant="plain" padding={12}>
    <div class="section-label">DGM FITNESS</div>
    {#if hasFitnessRows}
      <div class="conflict-list">
        {#each (d as any).fitnessEvaluated as f (f.seq)}
          <div class="conflict-entry">
            <div class="conflict-head">
              <Badge tone={fitnessTone(f.kind)} size="xs">{f.kind}</Badge>
              <span class="conflict-kind">{f.candidate_blake3}</span>
            </div>
            <span class="conflict-detail">
              fitness={f.fitness.toFixed(4)} pass={f.pass_rate.toFixed(3)} agree={f.dual_agreement.toFixed(3)}
            </span>
            <div class="conflict-meta">
              <span class="conflict-mono">tests={f.tests_total}</span>
              <span class="conflict-mono">{fmtTs(f.ts_ms)}</span>
            </div>
          </div>
        {/each}
      </div>
    {:else}
      <span class="ghost-text">no fitness rows (wire-dark: darwinian-evolve + ELAI_DARWINIAN_LOOP_ENABLE=1)</span>
    {/if}
  </Card>

  <!-- H-46: CMP PARENT SELECTION (clade-metaproductivity key per selection event).
       Wire-dark: same gate as DARWINIAN ADMITS (darwinian-evolve +
       ELAI_DARWINIAN_LOOP_ENABLE=1); empty in steady state.
       Sourced from reasoningData.cladeSelections (get_clade_selections, 10s poll).
       Shows node_id, cmp vs immediate_score, clade_size, and key badge. -->
  <Card variant="plain" padding={12}>
    <div class="section-label">CMP SELECTION</div>
    {#if hasCladeSelections}
      <div class="conflict-list">
        {#each d.cladeSelections as c (c.seq)}
          <div class="conflict-row">
            <div class="conflict-head">
              <Badge tone={cladeSelectionTone(c.kind)} size="xs">{c.key_used}</Badge>
              <span class="conflict-kind">{c.node_id}</span>
            </div>
            <span class="conflict-detail">
              cmp={c.cmp.toFixed(4)} imm={c.immediate_score.toFixed(4)} clade={c.clade_size}
            </span>
            <div class="conflict-meta">
              <span class="conflict-mono">{fmtTs(c.ts_ms)}</span>
            </div>
          </div>
        {/each}
      </div>
    {:else}
      <span class="ghost-text">no clade selections (wire-dark: darwinian-evolve + ELAI_DARWINIAN_LOOP_ENABLE=1)</span>
    {/if}
  </Card>

  <!-- 4. INLINE FEASIBILITY (C-22: rendered via FeasibilityPanel component) -->
  <Card variant="plain" padding={12}>
    <div class="section-label">INLINE FEASIBILITY</div>
    <FeasibilityPanel report={d.feasibility} />

    <!-- C-3: raw feasibility-report log (get_feasibility_log consumer).
         Collapsible so it does not crowd the summary panel above. -->
    <button
      type="button"
      class="feas-log-toggle"
      onclick={() => (showFeasibilityLog = !showFeasibilityLog)}
      aria-expanded={showFeasibilityLog}
    >
      {showFeasibilityLog ? '▾' : '▸'} feasibility log ({feasibilityLog.length})
    </button>
    {#if showFeasibilityLog}
      {#if feasibilityLog.length === 0}
        <span class="ghost-text">no feasibility probes recorded yet</span>
      {:else}
        <ul class="feas-log-rows" data-testid="feasibility-log">
          {#each feasibilityLog as row (row.seq)}
            <li class="feas-log-row">
              <Badge tone={feasVerdictTone(row.verdict)} size="xs">{row.verdict}</Badge>
              <span class="feas-log-id">#{row.task_id}</span>
              <span class="feas-log-margins">
                vram {row.vram_margin_pct.toFixed(0)}% · ram {row.ram_margin_pct.toFixed(0)}% · disk {row.disk_margin_pct.toFixed(0)}%
              </span>
              {#if row.novel_step_count > 0}
                <span class="feas-log-novel">{row.novel_step_count} novel</span>
              {/if}
              {#if row.escalations_count > 0}
                <span class="feas-log-esc">{row.escalations_count} esc</span>
              {/if}
            </li>
          {/each}
        </ul>
      {/if}
    {/if}
  </Card>

  <!-- 5. MISTAKE BOOK -->
  <Card variant="plain" padding={12}>
    <div class="section-label">MISTAKE BOOK</div>
    <div class="mistake-list">
      {#each d.mistakePreview as m}
        <button
          class="mistake-row"
          onclick={() => logStub('open-mistake', { id: m.id })}
          title="Open mistake {m.id}"
        >
          <span class="mistake-line">{m.baby_step_id} · {m.pattern_id} · {m.description}</span>
        </button>
      {/each}
    </div>
  </Card>

  <!-- 5b. C-32 MINI-CRITIC VERDICTS -- per-blank PASS / WARN / BLOCK rows.
       Toggle defaults to ON per plan §Task 7. Color-coded via Badge tone:
       PASS=teal, WARN=warn, BLOCK=fail. -->
  <Card variant="plain" padding={12}>
    <div class="section-label">
      MINI-CRITIC
      <label class="mini-critic-toggle">
        <input type="checkbox" bind:checked={showMiniCritic} />
        Show MiniCritic
      </label>
    </div>
    {#if showMiniCritic}
      {#if miniCriticRows.length === 0}
        <span class="ghost-text">no mini_critic_verdict rows yet</span>
      {:else}
        <div class="mini-critic-list">
          {#each miniCriticRows as row, i (i + '-' + row.ts + '-' + row.blank_id)}
            <div class="mini-critic-row">
              <Badge tone={miniCriticVerdictTone(row.verdict)} size="xs">{row.verdict.toUpperCase()}</Badge>
              <span class="mini-critic-blank">BLANK_{row.blank_id}</span>
              <span class="mini-critic-reason">{row.reason}</span>
              <span class="mini-critic-lang">{row.language}</span>
            </div>
          {/each}
        </div>
      {/if}
    {/if}
  </Card>

  <!-- 6. GG SIGNALS -->
  <Card variant="plain" padding={12}>
    <div class="section-label">GG SIGNALS</div>
    <div class="gg-tiles">
      {#each ggTiles as tile}
        <div class="gg-tile">
          <span class="gg-label">{tile.label}</span>
          <span class="gg-value">{tile.value}</span>
        </div>
      {/each}
    </div>
  </Card>

  <!-- 7. REFLECT -->
  <Card variant="plain" padding={12}>
    <div class="section-label">REFLECT</div>

    <!-- B-39D live timeline: each stage transition emitted by the
         ChatReflectPipeline pushes a row here. Cleared on turn boundary
         (new stream_id from the singleton reflect-progress event). -->
    {#if reflectTimeline.length > 0}
      <div class="reflect-timeline" aria-label="Live REFLECT pipeline stages">
        {#each reflectTimeline as st, i (i + '-' + st.stage + '-' + st.ts)}
          <div class="reflect-tl-row">
            <Badge tone={reflectStageTone(st)} size="xs">{reflectStageLabel(st)}</Badge>
            {#if reflectStageDetail(st)}
              <span class="reflect-tl-detail">{reflectStageDetail(st)}</span>
            {/if}
          </div>
        {/each}
      </div>
    {/if}

    <div class="reflect-rows">
      <div class="reflect-row">
        <span class="reflect-key">Enabled</span>
        <Badge tone={reflectEnabled ? 'teal' : 'neutral'} size="xs">{reflectEnabled ? 'yes' : 'no'}</Badge>
      </div>
      <div class="reflect-row">
        <span class="reflect-key">Last triage</span>
        <span class="reflect-val">
          {reflectTriageClass}
          {#if reflectTriageTurnId !== null}
            <span class="reflect-mono">{reflectTriageTurnId}</span>
          {/if}
        </span>
      </div>
      <div class="reflect-row">
        <span class="reflect-key">Last verdict</span>
        <span class="reflect-val">
          {reflectVerdictIndex === null ? '—' : `candidate ${reflectVerdictIndex}`}
          {#if reflectVerdictIndex !== null}
            <span class="reflect-rationale">{reflectVerdictRationale}</span>
          {/if}
        </span>
      </div>
      <div class="reflect-row">
        <span class="reflect-key">Last summary</span>
        <span class="reflect-val">
          {reflectSummaryTokens === null ? '—' : `${reflectSummaryTokens} tokens`}
          {#if reflectSummaryTokens !== null}
            <span class="reflect-rationale">{reflectSummarySynthesis}</span>
          {/if}
        </span>
      </div>
    </div>
  </Card>

</div>

<style>
  .reasoning-drawer {
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

  /* GRC rows */
  .grc-rows {
    display: flex;
    flex-direction: column;
    gap: 6px;
  }

  .grc-row {
    display: flex;
    align-items: center;
    gap: 8px;
  }

  .grc-role {
    font-family: var(--font-mono);
    font-size: 10px;
    color: var(--text-secondary);
    width: 100px;
    flex-shrink: 0;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .grc-bars {
    display: flex;
    flex-direction: column;
    gap: 2px;
    flex: 1;
    min-width: 0;
  }

  .mini-bar-wrap {
    width: 100%;
  }

  .mini-bar {
    width: 100%;
    height: 4px;
    background: var(--bg-input);
    border-radius: var(--r-xs);
    overflow: hidden;
  }

  .mini-bar-fill {
    height: 100%;
    border-radius: var(--r-xs);
    transition: width var(--dur-base) var(--ease-out);
  }

  .mini-bar-fill--teal {
    background: var(--teal-bright);
  }

  .mini-bar-fill--steel {
    background: var(--steel);
  }

  .grc-stat {
    font-family: var(--font-mono);
    font-size: 10px;
    color: var(--text-ghost);
    white-space: nowrap;
    flex-shrink: 0;
  }

  /* B-5: per-role GRC config collapsible table. */
  .grc-config {
    margin-top: 8px;
    padding-top: 8px;
    border-top: 1px dashed var(--border-subtle);
  }
  .grc-config-toggle {
    display: flex;
    align-items: center;
    gap: 6px;
    width: 100%;
    background: none;
    border: none;
    padding: 2px 0;
    cursor: pointer;
    font-family: var(--font-mono);
    font-size: 10px;
    letter-spacing: 0.06em;
    text-transform: uppercase;
    color: var(--text-secondary);
    text-align: left;
  }
  .grc-config-toggle:hover {
    color: var(--text-primary);
  }
  .grc-config-toggle:focus-visible {
    box-shadow: var(--ring);
  }
  .grc-config-caret {
    color: var(--text-ghost);
  }
  .grc-config-table {
    margin-top: 6px;
    display: flex;
    flex-direction: column;
    gap: 2px;
  }
  .grc-config-header,
  .grc-config-row {
    display: grid;
    grid-template-columns: 1.4fr 0.8fr 0.8fr 1.4fr;
    gap: 6px;
    font-family: var(--font-mono);
    font-size: 10px;
  }
  .grc-config-header {
    color: var(--text-secondary);
    letter-spacing: 0.05em;
    text-transform: uppercase;
  }
  .grc-config-role {
    color: var(--text-primary);
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }
  .grc-config-cell {
    color: var(--text-secondary);
  }
  .grc-config-topic {
    color: var(--text-ghost);
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }
  .mono {
    font-family: var(--font-mono);
  }

  /* Grounding sweep */
  .sweep-tiles {
    display: flex;
    gap: 8px;
    margin-bottom: 8px;
  }

  .sweep-tile {
    display: flex;
    flex-direction: column;
    align-items: center;
    flex: 1;
    padding: 6px 4px;
    background: var(--bg-input);
    border-radius: var(--r-sm);
  }

  .sweep-count {
    font-family: var(--font-mono);
    font-size: 18px;
    font-weight: 600;
    line-height: 1;
  }

  .sweep-count--teal { color: var(--teal-bright); }
  .sweep-count--warn { color: var(--warn); }
  .sweep-count--fail { color: var(--fail); }

  .sweep-label {
    font-family: var(--font-mono);
    font-size: 10px;
    color: var(--text-ghost);
    letter-spacing: 0.06em;
    margin-top: 3px;
  }

  .sweep-ungrounded-list {
    display: flex;
    flex-direction: column;
    gap: 3px;
  }

  .sweep-claim {
    font-family: var(--font-mono);
    font-size: 10px;
    color: var(--text-ghost);
    padding: 3px 0;
    border-bottom: 1px solid var(--border-subtle);
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  /* CSQ */
  .csq-list {
    display: flex;
    flex-direction: column;
    gap: 8px;
  }

  .csq-entry {
    display: flex;
    flex-direction: column;
    gap: 4px;
  }

  .csq-question {
    font-family: var(--font-mono);
    font-size: 10px;
    color: var(--text-secondary);
    line-height: var(--leading-code);
  }

  .csq-actions {
    display: flex;
    gap: 4px;
  }

  /* B-5 GAP-1 — anti-rubber-stamp counter */
  .rubber-stamp-counter {
    display: flex;
    align-items: center;
    gap: 8px;
  }

  .rubber-stamp-label {
    font-size: 10px;
    color: var(--text-secondary);
  }

  .rubber-stamp-list {
    display: flex;
    flex-direction: column;
    gap: 4px;
    margin-top: 8px;
  }

  .rubber-stamp-entry {
    display: flex;
    gap: 8px;
    font-family: var(--font-mono);
    font-size: 10px;
    color: var(--text-secondary);
  }

  .rubber-stamp-role {
    color: var(--text-primary);
    font-weight: 600;
  }

  .rubber-stamp-detail {
    color: var(--text-secondary);
  }

  /* F-8 / Slice A5 — symbolic conflicts */
  .conflict-list {
    display: flex;
    flex-direction: column;
    gap: 8px;
  }

  .conflict-entry {
    display: flex;
    flex-direction: column;
    gap: 3px;
    padding-bottom: 6px;
    border-bottom: 1px solid var(--border-subtle);
  }

  .conflict-head {
    display: flex;
    align-items: center;
    gap: 6px;
  }

  .conflict-kind {
    font-family: var(--font-mono);
    font-size: 10px;
    color: var(--warn);
    font-weight: 600;
  }

  .conflict-key {
    font-family: var(--font-mono);
    font-size: 10px;
    color: var(--text-secondary);
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .conflict-detail {
    font-family: var(--font-mono);
    font-size: 10px;
    color: var(--text-secondary);
    line-height: var(--leading-code);
  }

  .conflict-meta {
    display: flex;
    gap: 8px;
    flex-wrap: wrap;
  }

  .conflict-mono {
    font-family: var(--font-mono);
    font-size: 10px;
    color: var(--text-ghost);
  }

  /* C-22: feasibility CSS moved into FeasibilityPanel.svelte. */

  /* Mistake book */
  .mistake-list {
    display: flex;
    flex-direction: column;
    gap: 3px;
  }

  .mistake-row {
    display: block;
    width: 100%;
    text-align: left;
    background: none;
    border: none;
    padding: 3px 0;
    cursor: pointer;
    border-bottom: 1px solid var(--border-subtle);
  }

  .mistake-row:hover .mistake-line {
    color: var(--text-secondary);
  }

  .mistake-row:focus-visible {
    box-shadow: var(--ring);
  }

  .mistake-line {
    font-family: var(--font-mono);
    font-size: 10px;
    color: var(--text-ghost);
    display: block;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    transition: color var(--dur-fast) var(--ease-std);
  }

  /* GG signals */
  .gg-tiles {
    display: grid;
    grid-template-columns: 1fr 1fr;
    gap: 6px;
  }

  .gg-tile {
    display: flex;
    flex-direction: column;
    gap: 3px;
    padding: 6px 8px;
    background: var(--bg-input);
    border-radius: var(--r-sm);
  }

  .gg-label {
    font-family: var(--font-mono);
    font-size: 10px;
    color: var(--text-ghost);
  }

  .gg-value {
    font-family: var(--font-mono);
    font-size: 12px;
    color: var(--teal-bright);
  }

  /* Shared */
  .ghost-text {
    font-family: var(--font-mono);
    font-size: 10px;
    color: var(--text-ghost);
  }

  /* C-3 feasibility log */
  .feas-log-toggle {
    margin-top: var(--s-2);
    background: none;
    border: none;
    padding: 0;
    cursor: pointer;
    font-family: var(--font-mono);
    font-size: 10px;
    color: var(--text-secondary);
    letter-spacing: 0.04em;
  }
  .feas-log-toggle:hover {
    color: var(--text-primary);
  }
  .feas-log-rows {
    list-style: none;
    padding: 0;
    margin: var(--s-2) 0 0 0;
    display: flex;
    flex-direction: column;
    gap: 3px;
  }
  .feas-log-row {
    display: flex;
    align-items: center;
    gap: var(--s-2);
    font-family: var(--font-mono);
    font-size: 10px;
    color: var(--text-ghost);
    flex-wrap: wrap;
  }
  .feas-log-id {
    color: var(--text-secondary);
  }

  /* REFLECT */
  .reflect-rows {
    display: flex;
    flex-direction: column;
    gap: 6px;
  }
  /* B-39D live timeline rows above the static end-of-turn rows */
  .reflect-timeline {
    display: flex;
    flex-direction: column;
    gap: 4px;
    margin-bottom: 8px;
    padding-bottom: 8px;
    border-bottom: 1px dashed var(--border-subtle);
  }
  .reflect-tl-row {
    display: flex;
    align-items: baseline;
    gap: 8px;
    line-height: var(--leading-code);
  }
  .reflect-tl-detail {
    font-family: var(--font-mono);
    font-size: 10px;
    color: var(--text-ghost);
    min-width: 0;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    flex: 1 1 auto;
  }

  .reflect-row {
    display: flex;
    align-items: flex-start;
    gap: 8px;
  }

  .reflect-key {
    font-family: var(--font-mono);
    font-size: 10px;
    color: var(--text-ghost);
    width: 90px;
    flex-shrink: 0;
  }

  .reflect-val {
    font-family: var(--font-mono);
    font-size: 10px;
    color: var(--text-secondary);
    display: flex;
    flex-direction: column;
    gap: 2px;
    min-width: 0;
  }

  .reflect-mono {
    font-family: var(--font-mono);
    font-size: 10px;
    color: var(--text-ghost);
  }

  .reflect-rationale {
    font-family: var(--font-mono);
    font-size: 10px;
    color: var(--text-ghost);
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  /* C-32 MiniCritic verdict rows */
  .mini-critic-toggle {
    margin-left: 8px;
    font-family: var(--font-mono);
    font-size: 10px;
    color: var(--text-ghost);
    display: inline-flex;
    align-items: center;
    gap: 4px;
    cursor: pointer;
  }

  .mini-critic-list {
    display: flex;
    flex-direction: column;
    gap: 4px;
  }

  .mini-critic-row {
    display: flex;
    align-items: center;
    gap: 8px;
    font-family: var(--font-mono);
    font-size: 10px;
  }

  .mini-critic-blank {
    color: var(--text-secondary);
    font-weight: 600;
    min-width: 64px;
  }

  .mini-critic-reason {
    color: var(--text-secondary);
    flex: 1 1 auto;
    min-width: 0;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .mini-critic-lang {
    color: var(--text-ghost);
    flex-shrink: 0;
  }
</style>
