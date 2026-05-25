<script lang="ts">
  import { onMount } from 'svelte';
  import { reasoningData } from '../state/drawers.svelte';
  import { Card, Badge, Button, UnwiredNote } from '$lib/ui';
  import { logStub } from '../api/stub';
  import FeasibilityPanel from '$lib/components/FeasibilityPanel.svelte';
  import ArchiveVerifyGauge from '$lib/components/ArchiveVerifyGauge.svelte';
  import TripwireStatusStrip from '$lib/components/TripwireStatusStrip.svelte';

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

  <!-- 4. INLINE FEASIBILITY (C-22: rendered via FeasibilityPanel component) -->
  <Card variant="plain" padding={12}>
    <div class="section-label">INLINE FEASIBILITY</div>
    <FeasibilityPanel report={d.feasibility} />
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
    font-size: 20px;
    font-weight: 600;
    line-height: 1;
  }

  .sweep-count--teal { color: var(--teal-bright); }
  .sweep-count--warn { color: var(--warn); }
  .sweep-count--fail { color: var(--fail); }

  .sweep-label {
    font-family: var(--font-mono);
    font-size: 9px;
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
    line-height: 1.4;
  }

  .csq-actions {
    display: flex;
    gap: 4px;
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
    line-height: 1.4;
  }
  .reflect-tl-detail {
    font-family: var(--font-mono);
    font-size: 10px;
    color: var(--text-ghost);
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
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .mini-critic-lang {
    color: var(--text-ghost);
    flex-shrink: 0;
  }
</style>
