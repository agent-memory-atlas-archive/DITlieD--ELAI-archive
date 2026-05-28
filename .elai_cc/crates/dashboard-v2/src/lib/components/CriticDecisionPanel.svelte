<script lang="ts">
  // E-10 -- hostile-critic structural simplifications decision panel.
  // 6-tile renderer over critic_decision_event SSB rows. CloudDefault +
  // PromptCacheHitRate fold into the Tier tile; GrcBlockquote becomes a
  // header badge; MistakeBookSignal becomes a status badge. Live via SSB
  // subscription with 5s self-polling. See workflow/reasoning/E-10-decisions.md.
  import { invoke } from "@tauri-apps/api/core";
  import { onMount, onDestroy } from "svelte";

  type CriticTileState = {
    phase: string;
    baseline_score: number;
    challenger_score: number;
    paired_bootstrap_p: number | null;
    sample_size: number;
    verdict_code: number;
    ts_ms: number;
  };
  type CriticDecisionState = {
    tiles: CriticTileState[];
    grand_total: number;
  };

  const TILE_PHASES: { key: string; label: string }[] = [
    { key: "fsm_ablation", label: "FSM" },
    { key: "role_ablation", label: "Roles" },
    { key: "rag_challenger", label: "RAG" },
    { key: "typed_cot_verified", label: "Typed CoT" },
    { key: "tier_matrix", label: "Tier" },
    { key: "mistake_book_signal", label: "MistakeBook" },
  ];

  let panelState: CriticDecisionState | null = $state(null);
  let loadError: string | null = $state(null);
  let pollHandle: ReturnType<typeof setInterval> | null = null;

  async function refresh() {
    try {
      panelState = await invoke<CriticDecisionState>("get_critic_decision_state", { limit: 200 });
      loadError = null;
    } catch (e) {
      loadError = String(e);
    }
  }

  function tileFor(phaseKey: string): CriticTileState | null {
    if (!panelState) return null;
    return panelState.tiles.find((t) => t.phase === phaseKey) ?? null;
  }

  function verdictLabel(code: number): string {
    return code === 1 ? "Simplify" : "Keep";
  }

  function fmt(n: number): string {
    return n.toFixed(3);
  }

  onMount(() => {
    refresh();
    pollHandle = setInterval(refresh, 5000);
  });

  onDestroy(() => {
    if (pollHandle) clearInterval(pollHandle);
  });
</script>

<div class="critic-decision-panel">
  <header class="cdp-header">
    <span class="cdp-title">Critic Decisions</span>
    {#if tileFor("grc_blockquote")}
      <span class="cdp-badge">GRC: {verdictLabel(tileFor("grc_blockquote")?.verdict_code ?? 0)}</span>
    {/if}
    {#if panelState}<span class="cdp-count">rows: {panelState.grand_total}</span>{/if}
  </header>
  {#if loadError}
    <div class="cdp-error">load error: {loadError}</div>
  {/if}
  <div class="cdp-grid">
    {#each TILE_PHASES as p}
      {@const t = tileFor(p.key)}
      <div class="cdp-tile" class:simplify={t?.verdict_code === 1}>
        <div class="cdp-tile-label">{p.label}</div>
        {#if t}
          <div class="cdp-tile-scores">
            <span title="baseline">{fmt(t.baseline_score)}</span>
            <span class="cdp-arrow">vs</span>
            <span title="challenger">{fmt(t.challenger_score)}</span>
          </div>
          <div class="cdp-tile-stats">
            {#if t.paired_bootstrap_p !== null}
              <span title="paired-bootstrap p">p={fmt(t.paired_bootstrap_p)}</span>
            {:else}
              <span title="paired-bootstrap p">p=NA</span>
            {/if}
            <span title="sample size">n={t.sample_size}</span>
          </div>
          <div class="cdp-tile-verdict">{verdictLabel(t.verdict_code)}</div>
        {:else}
          <div class="cdp-tile-empty">no data</div>
        {/if}
      </div>
    {/each}
  </div>
</div>

<style>
  .critic-decision-panel { display: flex; flex-direction: column; gap: 0.5rem; padding: 0.75rem; }
  .cdp-header { display: flex; align-items: center; gap: 0.5rem; }
  .cdp-title { font-weight: 600; }
  .cdp-badge { font-size: 0.75rem; padding: 0.125rem 0.375rem; background: var(--color-surface-2, #2a2a2a); border-radius: 4px; }
  .cdp-count { font-size: 0.75rem; opacity: 0.7; margin-left: auto; }
  .cdp-error { color: #f55; font-size: 0.875rem; }
  .cdp-grid { display: grid; grid-template-columns: repeat(3, 1fr); gap: 0.5rem; }
  .cdp-tile { padding: 0.5rem; border: 1px solid var(--color-border, #444); border-radius: 6px; display: flex; flex-direction: column; gap: 0.25rem; }
  .cdp-tile.simplify { border-color: #4ade80; }
  .cdp-tile-label { font-weight: 500; font-size: 0.875rem; }
  .cdp-tile-scores { display: flex; align-items: center; gap: 0.375rem; font-variant-numeric: tabular-nums; font-size: 0.9rem; }
  .cdp-arrow { opacity: 0.6; }
  .cdp-tile-stats { display: flex; gap: 0.5rem; font-size: 0.75rem; opacity: 0.85; font-variant-numeric: tabular-nums; }
  .cdp-tile-verdict { font-size: 0.875rem; font-weight: 500; }
  .cdp-tile-empty { font-size: 0.875rem; opacity: 0.5; }
</style>
