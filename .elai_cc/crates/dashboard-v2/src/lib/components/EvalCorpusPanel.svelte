<script lang="ts">
  // E-54 EvalCorpusPanel.
  //
  // Reads `get_eval_corpus_status` (commands_e54.rs) and renders:
  //   - per-family task count (shard_created / rejected)
  //   - cumulative ngram-reject + cosine-reject + schema-violation +
  //     test-fails-on-reference + test-useless + generator-rotation-violation
  //     + perf-delta counters
  //   - the configured ngram_n + cosine threshold
  //
  // Wire-dark default: `known === false` -> render "(no data yet)".
  // Producer side is the eval-corpus crate; gated by
  // ELAI_EVAL_CORPUS_ENABLE=1 on the orchestrator.
  import { onMount, onDestroy } from 'svelte';
  import { invoke } from '@tauri-apps/api/core';

  type FamilyCount = {
    family: string;
    shard_created_count: number;
    rejected_count: number;
  };

  type EvalCorpusStatus = {
    known: boolean;
    families: FamilyCount[];
    ngram_rejected_count: number;
    cosine_rejected_count: number;
    schema_violation_count: number;
    test_fails_reference_count: number;
    test_useless_count: number;
    generator_rotation_violation_count: number;
    perf_delta_count: number;
    ngram_n: number;
    cosine_threshold: number;
    grand_total: number;
    last_ts_ms: number;
  };

  let status: EvalCorpusStatus | null = $state(null);
  let pollTimer: ReturnType<typeof setInterval> | null = null;

  async function refresh() {
    try {
      const r = await invoke<EvalCorpusStatus>('get_eval_corpus_status', {
        limit: 500,
      });
      status = r;
    } catch (e) {
      // Wire-dark default: silent failure leaves the (no data yet) panel.
      console.warn('[EvalCorpusPanel] refresh failed:', e);
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

<section class="eval-corpus-panel" aria-label="Eval corpus (E-54)">
  <header class="ec-header">
    <span class="ec-title mono">eval corpus</span>
    {#if status && status.known}
      <span class="ec-thresholds mono"
        title="13-gram baseline + cosine threshold echoed from the eval-corpus crate constants."
      >n={status.ngram_n} cos={status.cosine_threshold.toFixed(2)}</span>
    {/if}
  </header>
  {#if !status || !status.known}
    <p class="ec-empty mono">(no eval-corpus rows yet)</p>
  {:else}
    <div class="ec-counters">
      <span class="ec-counter mono" title="ingest_rejected_ngram">
        ngram-reject: {status.ngram_rejected_count}
      </span>
      <span class="ec-counter mono" title="ingest_rejected_cosine">
        cos-reject: {status.cosine_rejected_count}
      </span>
      <span class="ec-counter mono" title="corpus_schema_violation_no_tests">
        no-tests: {status.schema_violation_count}
      </span>
      <span class="ec-counter mono" title="corpus_test_fails_on_reference">
        fail-ref: {status.test_fails_reference_count}
      </span>
      <span class="ec-counter mono" title="corpus_test_useless_passes_broken">
        useless: {status.test_useless_count}
      </span>
      <span class="ec-counter mono" title="generator_rotation_violated">
        gen-rot: {status.generator_rotation_violation_count}
      </span>
      <span class="ec-counter mono" title="perf_delta_contamination_suspected">
        perf-delta: {status.perf_delta_count}
      </span>
    </div>
    <table class="ec-families" aria-label="per family counts">
      <thead>
        <tr>
          <th class="mono">family</th>
          <th class="mono">shards</th>
          <th class="mono">rejects</th>
        </tr>
      </thead>
      <tbody>
        {#each status.families as f (f.family)}
          <tr>
            <td class="mono">{f.family}</td>
            <td class="mono">{f.shard_created_count}</td>
            <td class="mono">{f.rejected_count}</td>
          </tr>
        {/each}
      </tbody>
    </table>
  {/if}
</section>

<style>
  .eval-corpus-panel {
    padding: 8px 10px;
    border: 1px solid #2a2a2a;
    border-radius: 4px;
    background: #0f0f0f;
  }
  .ec-header {
    display: flex;
    align-items: center;
    gap: 10px;
    margin-bottom: 6px;
  }
  .ec-title {
    color: #9ad4ff;
    font-weight: 600;
  }
  .ec-thresholds {
    color: #7a7a7a;
    font-size: 10px;
  }
  .ec-empty {
    color: #555;
    font-size: 12px;
    margin: 4px 0;
  }
  .ec-counters {
    display: flex;
    flex-wrap: wrap;
    gap: 6px;
    margin-bottom: 6px;
  }
  .ec-counter {
    padding: 1px 6px;
    border: 1px solid #2a2a2a;
    border-radius: 2px;
    color: #c0c0c0;
    font-size: 10px;
  }
  .ec-families {
    width: 100%;
    border-collapse: collapse;
    font-size: 10px;
  }
  .ec-families th,
  .ec-families td {
    text-align: left;
    padding: 2px 4px;
    border-bottom: 1px solid #1a1a1a;
  }
  .ec-families th {
    color: #888;
    font-weight: normal;
  }
  .mono {
    font-family: 'Fira Code', 'Cascadia Code', monospace;
  }
</style>
