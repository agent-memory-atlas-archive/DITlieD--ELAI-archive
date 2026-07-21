<script lang="ts">
  import { onMount } from 'svelte';
  import { antiStalenessData } from '$lib/state/drawers.svelte';
  import Badge from '$lib/ui/Badge.svelte';
  import Button from '$lib/ui/Button.svelte';
  import Divider from '$lib/ui/Divider.svelte';
  import UnwiredNote from '$lib/ui/UnwiredNote.svelte';
  import { logStub, recompute_steering_vector, get_steering_effect } from '$lib/api/stub';
  import type { SteeringEffect } from '$lib/api/types';

  onMount(() => {
    antiStalenessData.bootstrap();
    get_steering_effect().then((e: SteeringEffect | null) => { steeringEffect = e; });
    return antiStalenessData.teardown;
  });

  const NOW = Date.now();
  const deps = $derived(antiStalenessData.deprecatedPatterns);
  const audit = $derived(antiStalenessData.depAudit);

  // Static steering status derived from mock
  const steeringActive = $state(true);
  const steeringLastRefresh = NOW - 18 * 60 * 1000; // 18 mins ago
  let steeringEffect = $state<SteeringEffect | null>(null);

  function fmtAgo(ts: number): string {
    const diff = NOW - ts;
    const mins = Math.floor(diff / 60_000);
    if (mins < 60) return `${mins}m ago`;
    const hrs = Math.floor(mins / 60);
    if (hrs < 24) return `${hrs}h ago`;
    return `${Math.floor(hrs / 24)}d ago`;
  }

  function fmtTs(ts: number): string {
    return new Date(ts).toISOString().slice(0, 16).replace('T', ' ');
  }

  function runAudit() {
    // No backend command for "run audit on demand" yet; the nightly cron path
    // is the only producer (A-15). Kept as a stub until exposed via Tauri.
    logStub('anti_staleness.run_audit');
  }

  async function recomputeSteering() {
    // Real: posts to sidecar /v1/steering/compute (Bucket A).
    try {
      await recompute_steering_vector();
    } catch (err) {
      console.warn('[AntiStalenessDrawer] recompute_steering_vector failed', err);
    }
  }

  // Audit summary stats
  const lastAuditTs = NOW - 8 * 60 * 60 * 1000;
  const auditOutdatedCount = $derived(audit.filter(d => d.outdated).length);
  const auditStatus = $derived(
    audit.some(d => d.security_advisory) ? 'FAIL' :
    auditOutdatedCount > 0 ? 'WARN' : 'PASS'
  );

  function auditTone(s: string): string {
    if (s === 'PASS') return 'teal';
    if (s === 'WARN') return 'warn';
    return 'fail';
  }

  // Baseline metrics to display (7 rows) — values are static from the fixed mock
  const baselineRows = [
    { key: 'attention_ceiling_tokens', label: 'attention_ceiling_tokens', value: '25 000' },
    { key: 'retry_rate_pct',          label: 'retry_rate_pct',           value: '4.2 %' },
    { key: 'few_shot_delta_pct',      label: 'few_shot_delta_pct',       value: '8.0 %' },
    { key: 'format_compliance_pct',   label: 'format_compliance_pct',    value: '97.3 %' },
    { key: 'first_pass_pct',          label: 'first_pass_pct',           value: '91.0 %' },
    { key: 'cosine_drift',            label: 'cosine_drift',             value: '0.031' },
    { key: 'latency_ms_p50',          label: 'latency_ms_p50',           value: '1.0 ms' },
  ];

  function ecosystemFromLang(p: { language: string }): string {
    return p.language.toLowerCase();
  }
</script>

<div class="drawer">
  <!-- DEPRECATED PATTERNS -->
  <section class="section">
    <h3 class="section-header">DEPRECATED PATTERNS</h3>
    <div class="count-tile">
      <span class="count-value">{deps.length}</span>
      <span class="count-label">total patterns</span>
    </div>
    <div class="pattern-list">
      {#each deps.slice(0, 5) as pat}
        <div class="pattern-row">
          <span class="pattern-name">{pat.pattern}</span>
          <Badge tone="steel" size="xs">{ecosystemFromLang(pat)}</Badge>
          <span class="pattern-ts">{fmtAgo(pat.added_ts)}</span>
        </div>
      {/each}
    </div>
  </section>

  <Divider />

  <!-- NIGHTLY DEP AUDIT -->
  <section class="section">
    <h3 class="section-header">NIGHTLY DEP AUDIT</h3>
    {#if antiStalenessData.unwired.depAudit}
      <UnwiredNote
        phase="A-15"
        feature="last-run timestamp + new-deprecation count + run-now trigger"
        note="get_dep_audit_results returns []; nightly registry-poll cron not yet running"
      />
    {:else}
      <div class="audit-meta">
        <span class="meta-label">last run</span>
        <span class="meta-value">{fmtTs(lastAuditTs)}</span>
        <Badge tone={auditTone(auditStatus)} size="xs">{auditStatus}</Badge>
      </div>
      <div class="audit-meta" style="margin-top:4px">
        <span class="meta-label">new deprecations</span>
        <span class="meta-value">{auditOutdatedCount}</span>
      </div>
      <div class="audit-action">
        <Button variant="ghost" size="sm" onclick={runAudit}>Run now (stub)</Button>
      </div>
    {/if}
  </section>

  <Divider />

  <!-- FRESHNESS STEERING VECTOR -->
  <section class="section">
    <h3 class="section-header">FRESHNESS STEERING VECTOR</h3>
    <div class="steering-row">
      <Badge tone={steeringActive ? 'teal' : 'ghost'} size="xs">
        {steeringActive ? 'ACTIVE' : 'INACTIVE'}
      </Badge>
      <span class="ts-ghost">last refresh {fmtAgo(steeringLastRefresh)}</span>
    </div>
    <div class="steering-action">
      <Button variant="ghost" size="sm" onclick={recomputeSteering}>Recompute</Button>
    </div>
    {#if steeringEffect}
      <div class="steering-effect">
        <span class="meta-label">behavior delta</span>
        <span class="meta-value">{steeringEffect.behavior_delta.toFixed(2)}</span>
      </div>
    {/if}
  </section>

  <Divider />

  <!-- BASELINE HARNESS (A-1) -->
  <section class="section">
    <h3 class="section-header">BASELINE HARNESS (A-1)</h3>
    {#if antiStalenessData.unwired.baselineResults}
      <UnwiredNote
        phase="A-1"
        feature="7 measured model-capability metrics (attention ceiling, retry rate, few-shot delta, ...)"
        note="get_baseline_results returns static spec values; the live A-1 standalone bench is not invoked from the dashboard"
      />
    {:else}
      <div class="metric-table">
        {#each baselineRows as row}
          <span class="metric-label">{row.label}</span>
          <span class="metric-value">{row.value}</span>
        {/each}
      </div>
    {/if}
  </section>
</div>

<style>
  .drawer {
    display: flex;
    flex-direction: column;
    gap: 0;
    padding-bottom: var(--s-6);
  }

  .section {
    padding: var(--s-4) var(--s-4) var(--s-3);
  }

  .section-header {
    font-family: var(--font-mono);
    font-size: 10px;
    font-weight: 500;
    letter-spacing: 0.08em;
    color: var(--text-ghost);
    text-transform: uppercase;
    margin: 0 0 var(--s-3) 0;
  }

  .count-tile {
    display: flex;
    align-items: baseline;
    gap: var(--s-2);
    margin-bottom: var(--s-3);
  }

  .count-value {
    font-family: var(--font-mono);
    font-size: 18px;
    color: var(--teal-bright);
    font-weight: 500;
  }

  .count-label {
    font-family: var(--font-mono);
    font-size: 10px;
    color: var(--text-ghost);
  }

  .pattern-list {
    display: flex;
    flex-direction: column;
    gap: 6px;
  }

  .pattern-row {
    display: flex;
    align-items: center;
    gap: var(--s-2);
    flex-wrap: wrap;
  }

  .pattern-name {
    font-family: var(--font-mono);
    font-size: 10px;
    color: var(--text-secondary);
    flex: 1;
    min-width: 0;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .pattern-ts {
    font-family: var(--font-mono);
    font-size: 10px;
    color: var(--text-ghost);
    white-space: nowrap;
    margin-left: auto;
  }

  .audit-meta {
    display: flex;
    align-items: center;
    gap: var(--s-2);
  }

  .meta-label {
    font-family: var(--font-mono);
    font-size: 10px;
    color: var(--text-ghost);
  }

  .meta-value {
    font-family: var(--font-mono);
    font-size: 10px;
    color: var(--text-secondary);
  }

  .audit-action {
    margin-top: var(--s-3);
  }

  .steering-row {
    display: flex;
    align-items: center;
    gap: var(--s-3);
    margin-bottom: var(--s-2);
  }

  .ts-ghost {
    font-family: var(--font-mono);
    font-size: 10px;
    color: var(--text-ghost);
  }

  .steering-action {
    margin-top: var(--s-1);
  }

  .steering-effect {
    display: flex;
    align-items: center;
    gap: var(--s-2);
    margin-top: var(--s-2);
  }

  .metric-table {
    display: grid;
    grid-template-columns: 18ch 1fr;
    row-gap: 6px;
    column-gap: var(--s-3);
  }

  .metric-label {
    font-family: var(--font-mono);
    font-size: 10px;
    color: var(--text-ghost);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .metric-value {
    font-family: var(--font-mono);
    font-size: 12px;
    color: var(--text-primary);
  }
</style>
