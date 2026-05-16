<!--
  B-38 GatesPanel — surfaces the continuous ambiguity-score gate.

  cross-lang-pair: dashboard/tauri/api
  Pairs with `get_ambiguity_state` in src-tauri/src/commands.rs and the
  `AmbiguityState` TypeScript type in src/lib/api/types.ts.

  The backend command returns mock data for B-38; the real producer wires
  `AmbiguityScorer` into the gate in a follow-up B-N. The `mock: true` flag
  drives the "(mock data)" banner so the surface is honest about its source.

  Svelte 5 runes: $state / $derived / $effect.
-->
<script lang="ts">
  import { invoke } from '@tauri-apps/api/core';
  import type {
    AmbiguityState,
    AmbiguityComponent,
    AmbiguityMilestone,
  } from '$lib/api/types';

  let ambig: AmbiguityState | null = $state(null);
  let error: string | null = $state(null);

  async function refresh() {
    try {
      ambig = await invoke<AmbiguityState>('get_ambiguity_state');
      error = null;
    } catch (e) {
      error = String(e);
    }
  }

  $effect(() => {
    refresh();
    const id = setInterval(refresh, 5_000);
    return () => clearInterval(id);
  });

  // Tightest-first milestone label colours.
  const milestoneColor: Record<AmbiguityMilestone, string> = {
    initial: '#d33',
    progress: '#d80',
    refined: '#aa3',
    ready: '#1a7',
  };

  const scoreColor = $derived.by(() => {
    if (!ambig) return '#888';
    return ambig.score <= ambig.threshold ? '#1a7' : '#d33';
  });

  function pct(x: number): string {
    return `${Math.round(x * 100)}%`;
  }

  function componentRow(name: string, c: AmbiguityComponent | null) {
    return { name, c };
  }

  const rows = $derived.by(() => {
    if (!ambig) return [];
    const r = [
      componentRow('Goal clarity', ambig.components.goal_clarity),
      componentRow('Constraint clarity', ambig.components.constraint_clarity),
      componentRow('Success criteria', ambig.components.success_criteria_clarity),
    ];
    if (ambig.brownfield) {
      r.push(componentRow('Context clarity', ambig.components.context_clarity));
    }
    return r;
  });
</script>

<section class="gates-panel" aria-label="Ambiguity gate">
  <header class="gates-panel__header">
    <h3>Ambiguity gate</h3>
    {#if ambig?.mock}
      <span class="gates-panel__mock-badge" title="Real producer not yet wired (B-38 follow-up)">
        mock data
      </span>
    {/if}
  </header>

  {#if error}
    <p class="gates-panel__error">{error}</p>
  {:else if !ambig}
    <p class="gates-panel__loading">Loading…</p>
  {:else}
    <div class="gates-panel__overall">
      <span class="gates-panel__score-label">overall</span>
      <span class="gates-panel__score" style="color: {scoreColor};">
        {ambig.score.toFixed(4)}
      </span>
      <span class="gates-panel__threshold">≤ {ambig.threshold.toFixed(2)}</span>
      <span
        class="gates-panel__milestone"
        style="background: {milestoneColor[ambig.milestone]};"
        title="milestone band"
      >
        {ambig.milestone}
      </span>
    </div>

    <table class="gates-panel__components">
      <thead>
        <tr>
          <th>dimension</th>
          <th>clarity</th>
          <th>weight</th>
          <th>floor</th>
        </tr>
      </thead>
      <tbody>
        {#each rows as row (row.name)}
          {#if row.c}
            <tr class:floor-fail={row.c.score < row.c.floor}>
              <td>{row.name}</td>
              <td>
                <div class="gates-panel__bar">
                  <div
                    class="gates-panel__bar-fill"
                    style="width: {pct(row.c.score)};"
                  ></div>
                  <span>{pct(row.c.score)}</span>
                </div>
              </td>
              <td>{pct(row.c.weight)}</td>
              <td>{pct(row.c.floor)}</td>
            </tr>
          {/if}
        {/each}
      </tbody>
    </table>

    {#if ambig.floor_failures.length > 0}
      <p class="gates-panel__floor-warning">
        floors breached: {ambig.floor_failures.join(', ')}
      </p>
    {/if}

    <div class="gates-panel__streak">
      <span>streak</span>
      <strong>{ambig.streak.current}/{ambig.streak.required}</strong>
      <em>
        {#if ambig.qualifies_for_advance}
          ready to advance
        {:else}
          needs one more snapshot
        {/if}
      </em>
    </div>
  {/if}
</section>

<style>
  .gates-panel {
    border: 1px solid #2a2a2a;
    border-radius: 6px;
    padding: 0.75rem 1rem;
    background: #111;
    color: #eee;
    font-family: ui-sans-serif, system-ui, sans-serif;
    font-size: 0.875rem;
  }
  .gates-panel__header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 0.5rem;
    margin-bottom: 0.5rem;
  }
  .gates-panel__header h3 {
    margin: 0;
    font-size: 0.95rem;
  }
  .gates-panel__mock-badge {
    font-size: 0.7rem;
    padding: 0.1rem 0.4rem;
    background: #553;
    color: #ffc;
    border-radius: 3px;
  }
  .gates-panel__overall {
    display: flex;
    align-items: baseline;
    gap: 0.6rem;
    margin-bottom: 0.5rem;
  }
  .gates-panel__score-label {
    text-transform: uppercase;
    font-size: 0.7rem;
    color: #888;
  }
  .gates-panel__score {
    font-size: 1.4rem;
    font-weight: 600;
    font-variant-numeric: tabular-nums;
  }
  .gates-panel__threshold {
    color: #888;
    font-size: 0.75rem;
  }
  .gates-panel__milestone {
    margin-left: auto;
    text-transform: uppercase;
    font-size: 0.7rem;
    font-weight: 600;
    color: #fff;
    padding: 0.15rem 0.5rem;
    border-radius: 3px;
  }
  .gates-panel__components {
    width: 100%;
    border-collapse: collapse;
    margin-bottom: 0.5rem;
  }
  .gates-panel__components th {
    text-align: left;
    color: #888;
    font-weight: 500;
    font-size: 0.7rem;
    text-transform: uppercase;
    padding: 0.2rem 0.3rem;
  }
  .gates-panel__components td {
    padding: 0.25rem 0.3rem;
    border-top: 1px solid #222;
    font-variant-numeric: tabular-nums;
  }
  .gates-panel__components tr.floor-fail td {
    color: #f88;
  }
  .gates-panel__bar {
    position: relative;
    width: 100%;
    height: 14px;
    background: #222;
    border-radius: 2px;
    overflow: hidden;
  }
  .gates-panel__bar-fill {
    position: absolute;
    inset: 0 auto 0 0;
    background: #2a7;
  }
  .gates-panel__bar span {
    position: relative;
    z-index: 1;
    display: block;
    text-align: center;
    line-height: 14px;
    font-size: 0.7rem;
    color: #fff;
  }
  .gates-panel__floor-warning {
    color: #f88;
    margin: 0.25rem 0;
    font-size: 0.8rem;
  }
  .gates-panel__streak {
    display: flex;
    align-items: baseline;
    gap: 0.5rem;
    font-size: 0.8rem;
  }
  .gates-panel__streak span {
    color: #888;
    text-transform: uppercase;
    font-size: 0.7rem;
  }
  .gates-panel__streak em {
    color: #aaa;
    font-style: normal;
  }
  .gates-panel__error {
    color: #f88;
  }
  .gates-panel__loading {
    color: #888;
  }
</style>
