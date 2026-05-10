<script lang="ts">
  import { onMount } from 'svelte';
  import { reasoningData } from '../state/drawers.svelte';
  import { Card, Badge, Button, UnwiredNote } from '$lib/ui';
  import { logStub } from '../api/stub';
  import type { FeasibilityOutcome } from '../api/types';

  onMount(() => {
    reasoningData.bootstrap();
    return reasoningData.teardown;
  });

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

  function feasibilityTone(outcome: FeasibilityOutcome): string {
    if (outcome === 'PASS') return 'teal';
    if (outcome === 'FAIL') return 'fail';
    return 'warn';
  }

  // Last 3 feasibility outcomes from results (most recent last)
  const feasOutcomes = $derived(
    d.feasibility.results.slice(-3).map(r => ({
      outcome: r.outcome,
      ts: fmtTs(r.ts),
    }))
  );

  // Full live feasibility table (most recent first, capped at 12 rows so the
  // drawer stays scrollable). null detail renders as em-dash.
  const feasRows = $derived(
    [...d.feasibility.results]
      .reverse()
      .slice(0, 12)
      .map(r => ({
        id: r.id,
        check: r.check,
        outcome: r.outcome,
        // PITFALL RUST_OPTION_SERDE_JSON_NULL_VS_TS_UNDEFINED: detail can be null.
        detail: r.detail == null ? '—' : r.detail,
        ts: fmtTs(r.ts),
      }))
  );

  const hasFeasResults = $derived(d.feasibility.results.length > 0);

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

  <!-- 4. INLINE FEASIBILITY (B-21 4F: live, SSB-backed) -->
  <Card variant="plain" padding={12}>
    <div class="section-label">INLINE FEASIBILITY</div>
    {#if d.unwired.feasibility}
      <UnwiredNote
        phase="4F (B-21)"
        feature="last 3 inline-feasibility outcomes (PASS / FAIL / BLOCKED)"
        note="get_feasibility_report returns static-shape PASS; FeasibilityProbe not yet implemented"
      />
    {:else if !hasFeasResults}
      <div class="feasibility-empty">
        <Badge tone={feasibilityTone(d.feasibility.overall)} size="xs">{d.feasibility.overall}</Badge>
        <span class="ghost-text">no feasibility checks recorded yet</span>
      </div>
    {:else}
      <div class="feasibility-overall">
        <span class="feasibility-overall-label">overall</span>
        <Badge tone={feasibilityTone(d.feasibility.overall)} size="xs">{d.feasibility.overall}</Badge>
        <span class="ghost-text">probed {fmtTs(d.feasibility.probed_at)}</span>
      </div>
      <div class="feasibility-chips">
        {#each feasOutcomes as f}
          <div class="feasibility-chip">
            <Badge tone={feasibilityTone(f.outcome)} size="xs">{f.outcome}</Badge>
            <span class="ghost-text">{f.ts}</span>
          </div>
        {/each}
      </div>
      <div class="feasibility-rows">
        {#each feasRows as r (r.id)}
          <div class="feasibility-row">
            <span class="feasibility-check">{r.check}</span>
            <Badge tone={feasibilityTone(r.outcome)} size="xs">{r.outcome}</Badge>
            <span class="feasibility-detail" title={r.detail}>{r.detail}</span>
            <span class="ghost-text">{r.ts}</span>
          </div>
        {/each}
      </div>
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
    font-size: 11px;
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
    font-size: 11px;
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
    font-size: 11px;
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
    font-size: 11px;
    color: var(--text-secondary);
    line-height: 1.4;
  }

  .csq-actions {
    display: flex;
    gap: 4px;
  }

  /* Feasibility */
  .feasibility-chips {
    display: flex;
    gap: 8px;
    flex-wrap: wrap;
  }

  .feasibility-chip {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 3px;
    padding: 6px 8px;
    background: var(--bg-input);
    border-radius: var(--r-sm);
  }

  .feasibility-empty {
    display: flex;
    align-items: center;
    gap: 8px;
  }

  .feasibility-overall {
    display: flex;
    align-items: center;
    gap: 8px;
    margin-bottom: 6px;
  }

  .feasibility-overall-label {
    font-family: var(--font-mono);
    font-size: 10px;
    color: var(--text-ghost);
    letter-spacing: 0.06em;
    text-transform: uppercase;
  }

  .feasibility-rows {
    display: flex;
    flex-direction: column;
    gap: 4px;
    margin-top: 8px;
    max-height: 220px;
    overflow-y: auto;
  }

  .feasibility-row {
    display: grid;
    grid-template-columns: minmax(80px, max-content) auto 1fr max-content;
    align-items: center;
    gap: 8px;
    padding: 4px 6px;
    background: var(--bg-input);
    border-radius: var(--r-xs);
  }

  .feasibility-check {
    font-family: var(--font-mono);
    font-size: 11px;
    color: var(--text-secondary);
    white-space: nowrap;
  }

  .feasibility-detail {
    font-family: var(--font-mono);
    font-size: 11px;
    color: var(--text-ghost);
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

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
    font-size: 11px;
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
    font-size: 11px;
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

  .reflect-row {
    display: flex;
    align-items: flex-start;
    gap: 8px;
  }

  .reflect-key {
    font-family: var(--font-mono);
    font-size: 11px;
    color: var(--text-ghost);
    width: 90px;
    flex-shrink: 0;
  }

  .reflect-val {
    font-family: var(--font-mono);
    font-size: 11px;
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
</style>
