<script lang="ts">
  // ==========================================================================
  // B-41 — Per-Level Auditor Timeline
  //
  // Visualises the `per_level_audit_verdict` SSB stream as a vertical timeline
  // keyed on the 6 AuditorMode values (Concepts/Phases/Steps/BabySteps/
  // BabyStepDetail/Plan). Cascade arrows are drawn between verdicts where
  // `cascade_to_parent === true`. Per-verdict popover surfaces the Finding
  // list (category, severity, source).
  //
  // Data: `getPerLevelVerdicts(taskId)` (live tryInvoke, no mock per brief).
  // The backing Tauri command currently returns an empty Vec on cold start
  // (see commands.rs::get_per_level_verdicts) which renders as the
  // "no verdicts yet" empty state — NOT a mock.
  // ==========================================================================

  import { onMount } from 'svelte';
  import type {
    LevelAuditVerdictJson,
    AuditorMode,
    AuditFinding,
    TestIntegrityEventJson,
  } from '../api/types';
  import { getPerLevelVerdicts, getTestIntegrityEvents } from '../api/stub';

  // Active task id — sourced from the workspace store in a follow-up wiring.
  // Until then we accept it via a prop with a sentinel default so the window
  // renders the empty-state in browser mode and is connected in tauri mode.
  interface Props {
    taskId?: string;
  }
  let { taskId = 'active' }: Props = $props();

  let verdicts = $state<LevelAuditVerdictJson[]>([]);
  let fabricationEvents = $state<TestIntegrityEventJson[]>([]);
  let loading = $state(true);
  let error = $state<string | null>(null);
  let selectedFindings = $state<AuditFinding[] | null>(null);
  let selectedLabel = $state<string>('');

  // Mode-keyed colour mapping per brief.
  const modeColour: Record<AuditorMode, string> = {
    concepts: '#3b82f6', // blue
    phases: '#22c55e', // green
    steps: '#eab308', // yellow
    babysteps: '#f97316', // orange
    babystep_detail: '#ef4444', // red
    plan: '#a855f7', // purple
    production_grade: '#ec4899', // pink
  };

  const modeLabel: Record<AuditorMode, string> = {
    concepts: 'Concepts',
    phases: 'Phases',
    steps: 'Steps',
    babysteps: 'BabySteps',
    babystep_detail: 'BabyStepDetail',
    plan: 'Plan',
    production_grade: 'ProductionGrade',
  };

  async function load() {
    loading = true;
    error = null;
    try {
      const [rows, fabRows] = await Promise.all([
        getPerLevelVerdicts(taskId),
        getTestIntegrityEvents(taskId),
      ]);
      verdicts = rows;
      fabricationEvents = fabRows;
    } catch (e) {
      error = String(e);
    } finally {
      loading = false;
    }
  }

  onMount(load);

  function showFindings(v: LevelAuditVerdictJson) {
    selectedFindings = v.findings;
    selectedLabel = `${modeLabel[v.mode]} · ${v.tier_node_id.tier} #${v.tier_node_id.id}`;
  }
  function closeFindings() {
    selectedFindings = null;
    selectedLabel = '';
  }

  // Group counts per mode for badge summary at the top.
  let modeCounts = $derived.by(() => {
    const counts: Record<string, number> = {};
    for (const v of verdicts) {
      counts[v.mode] = (counts[v.mode] ?? 0) + 1;
    }
    return counts;
  });

  let cascadeCount = $derived(verdicts.filter((v) => v.cascade_to_parent).length);
  let blockCount = $derived(verdicts.filter((v) => v.verdict === 'block').length);
  let fabricationBlocks = $derived(
    fabricationEvents.filter((e) => e.kind === 'fabrication_blocked').length,
  );
</script>

<div class="audit-timeline">
  <header class="atl-header">
    <h2>Auditor Timeline</h2>
    <div class="atl-summary">
      <span class="atl-stat" data-label="verdicts">{verdicts.length}</span>
      <span class="atl-stat atl-block" data-label="blocks">{blockCount}</span>
      <span class="atl-stat atl-cascade" data-label="cascades">{cascadeCount}</span>
      {#if fabricationBlocks > 0}
        <span class="atl-stat atl-fabrication" data-label="fabrication blocks" aria-label="{fabricationBlocks} anti-fabrication block{fabricationBlocks === 1 ? '' : 's'}">{fabricationBlocks}</span>
      {/if}
      <button type="button" class="atl-refresh" on:click={load}>Refresh</button>
    </div>
  </header>

  <div class="atl-badges">
    {#each Object.entries(modeLabel) as [mode, label]}
      <span
        class="atl-badge"
        style:--badge-color={modeColour[mode as AuditorMode]}
      >
        {label} · {modeCounts[mode] ?? 0}
      </span>
    {/each}
  </div>

  {#if error}
    <div class="atl-error" role="alert">{error}</div>
  {/if}

  {#if loading}
    <div class="atl-loading">Loading verdicts…</div>
  {:else if verdicts.length === 0}
    <div class="atl-empty">
      No per-level audit verdicts for task <code>{taskId}</code> yet. Verdicts
      surface here as soon as the orchestrator runs the first Auditor* agent.
    </div>
  {:else}
    <ol class="atl-list">
      {#each verdicts as v, i (i)}
        <li
          class="atl-row"
          class:atl-verdict-block={v.verdict === 'block'}
          class:atl-cascade-row={v.cascade_to_parent}
        >
          <button
            type="button"
            class="atl-dot"
            style:--row-color={modeColour[v.mode]}
            aria-label="Show findings for {modeLabel[v.mode]} {v.tier_node_id.tier} {v.tier_node_id.id}"
            on:click={() => showFindings(v)}
          ></button>
          <div class="atl-meta">
            <span class="atl-mode" style:color={modeColour[v.mode]}>
              {modeLabel[v.mode]}
            </span>
            <span class="atl-tier">{v.tier_node_id.tier} #{v.tier_node_id.id}</span>
            <span
              class="atl-verdict atl-verdict-{v.verdict}"
              aria-label="Verdict {v.verdict}"
            >{v.verdict}</span>
            {#if v.respawned_once}
              <span class="atl-flag">respawned</span>
            {/if}
            {#if v.cascade_to_parent}
              <span class="atl-flag atl-flag-cascade" aria-label="cascade-to-parent">
                cascade →
              </span>
            {/if}
            <span class="atl-count">{v.findings.length} finding{v.findings.length === 1 ? '' : 's'}</span>
          </div>
        </li>
      {/each}
    </ol>
  {/if}

  {#if selectedFindings}
    <aside class="atl-popover" role="dialog" aria-label="Findings for {selectedLabel}">
      <header>
        <h3>{selectedLabel}</h3>
        <button type="button" on:click={closeFindings} aria-label="Close popover">×</button>
      </header>
      {#if selectedFindings.length === 0}
        <p class="atl-empty">No findings recorded for this verdict.</p>
      {:else}
        <ul class="atl-findings">
          {#each selectedFindings as f, i (i)}
            <li class="atl-finding atl-sev-{f.severity}">
              <strong>[{f.severity}] {f.category}</strong>
              <p>{f.message}</p>
              {#if f.source}
                <small>source: {f.source}</small>
              {/if}
            </li>
          {/each}
        </ul>
      {/if}
    </aside>
  {/if}
</div>

<style>
  .audit-timeline {
    height: 100%;
    width: 100%;
    overflow: auto;
    padding: 1rem;
    font-family: system-ui, -apple-system, sans-serif;
    color: var(--fg, #e5e7eb);
    background: var(--bg, #0f172a);
    box-sizing: border-box;
  }

  .atl-header {
    display: flex;
    align-items: baseline;
    justify-content: space-between;
    margin-bottom: 1rem;
  }

  .atl-header h2 {
    margin: 0;
    font-size: 1.125rem;
    font-weight: 600;
  }

  .atl-summary {
    display: flex;
    gap: 0.75rem;
    align-items: center;
  }

  .atl-stat {
    font-variant-numeric: tabular-nums;
    font-size: 0.875rem;
  }

  .atl-stat::after {
    content: ' ' attr(data-label);
    font-size: 0.75rem;
    opacity: 0.7;
    margin-left: 0.25rem;
  }

  .atl-block { color: #ef4444; }
  .atl-cascade { color: #f97316; }
  .atl-fabrication { color: #dc2626; font-weight: 600; }

  .atl-refresh {
    background: transparent;
    border: 1px solid #334155;
    color: inherit;
    padding: 0.25rem 0.75rem;
    border-radius: 4px;
    cursor: pointer;
    font-size: 0.75rem;
  }
  .atl-refresh:hover { background: #1e293b; }
  .atl-refresh:focus { outline: 2px solid #3b82f6; outline-offset: 2px; }

  .atl-badges {
    display: flex;
    gap: 0.5rem;
    margin-bottom: 1rem;
    flex-wrap: wrap;
  }

  .atl-badge {
    padding: 0.125rem 0.625rem;
    border-radius: 999px;
    border: 1px solid var(--badge-color);
    color: var(--badge-color);
    font-size: 0.75rem;
    font-variant-numeric: tabular-nums;
  }

  .atl-error {
    border-left: 3px solid #ef4444;
    padding: 0.5rem 0.75rem;
    margin-bottom: 1rem;
    background: rgba(239, 68, 68, 0.08);
  }

  .atl-loading,
  .atl-empty {
    opacity: 0.7;
    font-style: italic;
    padding: 1.5rem 0;
  }

  .atl-empty code {
    background: rgba(255, 255, 255, 0.08);
    padding: 0 0.25rem;
    border-radius: 3px;
  }

  .atl-list {
    list-style: none;
    margin: 0;
    padding: 0;
    border-left: 1px solid #1e293b;
  }

  .atl-row {
    display: flex;
    gap: 0.75rem;
    padding: 0.5rem 0 0.5rem 0.5rem;
    align-items: center;
    position: relative;
  }

  .atl-row.atl-cascade-row {
    border-bottom: 1px dashed #f97316;
  }

  .atl-dot {
    width: 1rem;
    height: 1rem;
    border-radius: 50%;
    border: 2px solid var(--row-color);
    background: var(--row-color);
    margin-left: -1.25rem;
    cursor: pointer;
    flex-shrink: 0;
    padding: 0;
  }
  .atl-dot:focus { outline: 2px solid #3b82f6; outline-offset: 2px; }

  .atl-verdict-block .atl-dot {
    background: transparent;
  }

  .atl-meta {
    display: flex;
    gap: 0.75rem;
    align-items: center;
    font-size: 0.875rem;
    flex-wrap: wrap;
  }

  .atl-mode {
    font-weight: 600;
  }

  .atl-tier {
    font-variant-numeric: tabular-nums;
    opacity: 0.8;
  }

  .atl-verdict {
    text-transform: uppercase;
    font-size: 0.6875rem;
    padding: 0.125rem 0.5rem;
    border-radius: 3px;
    font-weight: 600;
    letter-spacing: 0.05em;
  }
  .atl-verdict-allow {
    color: #22c55e;
    background: rgba(34, 197, 94, 0.12);
  }
  .atl-verdict-block {
    color: #ef4444;
    background: rgba(239, 68, 68, 0.12);
  }

  .atl-flag {
    font-size: 0.6875rem;
    padding: 0.125rem 0.375rem;
    border-radius: 3px;
    background: rgba(255, 255, 255, 0.06);
  }

  .atl-flag-cascade {
    color: #f97316;
    background: rgba(249, 115, 22, 0.12);
  }

  .atl-count {
    opacity: 0.65;
    font-size: 0.75rem;
  }

  .atl-popover {
    position: fixed;
    right: 1rem;
    top: 4rem;
    width: min(420px, 90vw);
    background: #1e293b;
    border: 1px solid #334155;
    border-radius: 6px;
    padding: 1rem;
    z-index: 1000;
    box-shadow: 0 8px 24px rgba(0, 0, 0, 0.4);
  }
  .atl-popover header {
    display: flex;
    justify-content: space-between;
    align-items: baseline;
    margin-bottom: 0.75rem;
  }
  .atl-popover h3 {
    margin: 0;
    font-size: 0.9375rem;
  }
  .atl-popover button {
    background: transparent;
    border: none;
    color: inherit;
    cursor: pointer;
    font-size: 1.25rem;
  }
  .atl-popover button:focus { outline: 2px solid #3b82f6; }

  .atl-findings {
    list-style: none;
    margin: 0;
    padding: 0;
    max-height: 60vh;
    overflow: auto;
  }

  .atl-finding {
    padding: 0.5rem 0;
    border-bottom: 1px solid #334155;
  }
  .atl-finding:last-child { border-bottom: none; }
  .atl-finding p {
    margin: 0.25rem 0;
    font-size: 0.875rem;
  }
  .atl-finding small {
    opacity: 0.6;
    font-size: 0.75rem;
  }

  .atl-sev-BLOCK strong { color: #ef4444; }
  .atl-sev-WARN strong { color: #eab308; }
  .atl-sev-INFO strong { color: #3b82f6; }
</style>
