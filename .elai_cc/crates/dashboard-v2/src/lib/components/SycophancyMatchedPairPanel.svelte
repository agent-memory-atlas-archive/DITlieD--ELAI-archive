<script lang="ts">
  // E-54 SycophancyMatchedPairPanel.
  //
  // Distinct from SycophancyFlagStrip (D-14 surface). Reads matched-pair
  // eval-vs-deploy framing probe results + sandbagging-anomaly flags from
  // the EvalCorpusEvent SSB stream.
  //
  // Wire-dark default: `known === false` -> render "(no data yet)".
  import { onMount, onDestroy } from 'svelte';
  import { invoke } from '@tauri-apps/api/core';

  type MatchedPairProbeView = {
    task_id: string;
    family: string;
    phase: string;
    gap_pp: number | null;
    sandbagging_anomaly: boolean;
    ts_ms: number;
  };

  type SycophancyMatchedPairResults = {
    known: boolean;
    probes: MatchedPairProbeView[];
    framing_gap_count: number;
    sandbagging_anomaly_count: number;
    grand_total: number;
  };

  let results: SycophancyMatchedPairResults | null = $state(null);
  let pollTimer: ReturnType<typeof setInterval> | null = null;

  async function refresh() {
    try {
      results = await invoke<SycophancyMatchedPairResults>(
        'get_sycophancy_matched_pair_results',
        { limit: 200 },
      );
    } catch (e) {
      console.warn('[SycophancyMatchedPairPanel] refresh failed:', e);
    }
  }

  onMount(() => {
    refresh();
    pollTimer = setInterval(refresh, 30_000);
  });
  onDestroy(() => {
    if (pollTimer !== null) {
      clearInterval(pollTimer);
      pollTimer = null;
    }
  });
</script>

<section
  class="sycophancy-matched-pair-panel"
  aria-label="Sycophancy matched-pair probes (E-54)"
>
  <header class="smp-header">
    <span class="smp-title mono">matched-pair sycophancy</span>
    {#if results && results.known}
      <span class="smp-summary mono" title="framing-gap > 10pp / sandbag anomaly counters"
        >gap: {results.framing_gap_count} | sandbag: {results.sandbagging_anomaly_count}</span
      >
    {/if}
  </header>
  {#if !results || !results.known}
    <p class="smp-empty mono">(no matched-pair rows yet)</p>
  {:else if results.probes.length === 0}
    <p class="smp-empty mono">(no framing-gap / sandbag anomalies)</p>
  {:else}
    <table class="smp-probes" aria-label="recent probes">
      <thead>
        <tr>
          <th class="mono">task</th>
          <th class="mono">family</th>
          <th class="mono">phase</th>
          <th class="mono">gap pp</th>
        </tr>
      </thead>
      <tbody>
        {#each results.probes.slice(0, 20) as p (p.task_id + p.ts_ms)}
          <tr class:smp-anomaly={p.sandbagging_anomaly}>
            <td class="mono" title={p.task_id}>{p.task_id.slice(0, 12)}</td>
            <td class="mono">{p.family}</td>
            <td class="mono">{p.phase.replace(/_/g, ' ')}</td>
            <td class="mono">
              {#if p.gap_pp !== null}{p.gap_pp.toFixed(1)}{:else}-{/if}
            </td>
          </tr>
        {/each}
      </tbody>
    </table>
  {/if}
</section>

<style>
  .sycophancy-matched-pair-panel {
    padding: 8px 10px;
    border: 1px solid #2a2a2a;
    border-radius: 4px;
    background: #0f0f0f;
  }
  .smp-header {
    display: flex;
    align-items: center;
    gap: 10px;
    margin-bottom: 6px;
  }
  .smp-title {
    color: #ffb87a;
    font-weight: 600;
  }
  .smp-summary {
    color: #7a7a7a;
    font-size: 11px;
  }
  .smp-empty {
    color: #555;
    font-size: 12px;
    margin: 4px 0;
  }
  .smp-probes {
    width: 100%;
    border-collapse: collapse;
    font-size: 11px;
  }
  .smp-probes th,
  .smp-probes td {
    text-align: left;
    padding: 2px 4px;
    border-bottom: 1px solid #1a1a1a;
  }
  .smp-probes th {
    color: #888;
    font-weight: normal;
  }
  .smp-anomaly td {
    color: #ff8888;
  }
  .mono {
    font-family: 'Fira Code', 'Cascadia Code', monospace;
  }
</style>
