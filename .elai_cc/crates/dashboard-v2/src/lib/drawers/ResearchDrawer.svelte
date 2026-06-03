<script lang="ts">
  import { onMount } from 'svelte';
  import { invoke } from '@tauri-apps/api/core';
  import { Tree, Badge, Scrollbar, UnwiredNote } from '$lib/ui';
  import SkeletonList from '$lib/components/SkeletonList.svelte';
  import { researchData } from '$lib/state/drawers.svelte';
  import { plan } from '$lib/state/plan.svelte';
  import { get_research_node_history } from '$lib/api/stub';
  import type { TreeNode } from '$lib/ui/tree-types';
  import type {
    DecompNode,
    ResearchTreeNode,
    ResearchNodeStage,
    SourceTierDistribution,
    ResearchNodeHistoryRow,
  } from '$lib/api/types';

  // E-33: cold-start loading flag — true on first mount until either
  // researchData populates a node or the polled fetch returns at least
  // once. SkeletonList renders during this window; switches to the
  // "No research nodes" empty-note after the first resolve when the
  // session truly has zero rows.
  let loading = $state<boolean>(true);

  onMount(() => {
    researchData.bootstrap();
    // Flip loading false on next paint — researchData.bootstrap() schedules
    // its own polling; we just need to clear the cold-start shimmer once
    // the store has had a chance to publish whatever it has.
    queueMicrotask(() => { loading = false; });
    return researchData.teardown;
  });

  // ------------------------------------------------------------------
  // Section 1 — DECOMPOSITION TREE (mini-map) — sourced from the live
  // `plan` store (already polling get_decomposition_tree every 5s in
  // Workspace.onMount). No separate fetch here.
  // ------------------------------------------------------------------
  function decompToTree(node: DecompNode): TreeNode {
    return {
      id: node.id,
      label: node.label,
      status: node.status,
      meta: node.elapsed_ms && node.elapsed_ms > 0
        ? formatElapsed(node.elapsed_ms)
        : undefined,
      children: node.children?.map(decompToTree),
    };
  }

  function formatElapsed(ms: number): string {
    if (ms >= 60_000) return `${Math.round(ms / 60_000)}m`;
    return `${Math.round(ms / 1000)}s`;
  }

  const decompRoot = $derived(plan.tree);
  const treeNodes = $derived<TreeNode[]>([decompToTree(decompRoot)]);

  let expandedIds = $state(new Set<string>());
  let selectedId = $state('');

  $effect(() => {
    // Expand root + any active/done phase whenever the tree changes.
    const next = new Set<string>();
    next.add(decompRoot.id);
    decompRoot.children?.forEach((p) => {
      if (p.status === 'active' || p.status === 'done') next.add(p.id);
    });
    expandedIds = next;
  });

  function handleTreeSelect(id: string) {
    selectedId = id;
    plan.selectNode(id);
  }

  // ------------------------------------------------------------------
  // Section 2 — RESEARCH TREE
  // ------------------------------------------------------------------
  const STAGES: ResearchNodeStage[] = ['FORMULATE', 'SEARCH', 'EXTRACT', 'REFLECT', 'CONVERGE'];

  function stageDotColor(stage: ResearchNodeStage, nodeStage: ResearchNodeStage, converged: boolean): string {
    const stageIdx = STAGES.indexOf(stage);
    const nodeIdx = STAGES.indexOf(nodeStage);
    if (stageIdx < nodeIdx) return 'var(--teal)';
    if (stageIdx === nodeIdx && converged) return 'var(--teal)';
    if (stageIdx === nodeIdx) return 'var(--warn)';
    return 'var(--text-ghost)';
  }

  const researchRoot = $derived(researchData.researchTree);
  const researchNodes = $derived<ResearchTreeNode[]>([
    researchRoot,
    ...(researchRoot.children ?? []),
  ]);

  // B-7 (SCHEMA_VERSION 100): per-node stage timeline. Rewires the previous
  // C-4 DRIFT-6 deferral (commands.rs::get_research_node_history) so the
  // ResearchTree drawer renders a per-stage history when the user expands
  // a research node. The active session id is derived from the FSM /
  // workspace store once available; for now we pass an empty string and
  // let the backend scope to the active session db.
  let stageHistory = $state<Record<string, ResearchNodeHistoryRow[]>>({});
  let stageHistoryTimer: ReturnType<typeof setInterval> | null = null;

  async function refreshStageHistoryForNode(nodeId: string): Promise<void> {
    // The Tauri command parses `nodeId` as a u64. Non-numeric ids return
    // an empty Vec (covered by the backend). For wire calls we use the
    // stub wrapper which falls back to invoke() under the hood and also
    // returns [] on failure, so this poller never throws past the Promise.
    try {
      const rows = await invoke<ResearchNodeHistoryRow[]>(
        'get_research_node_history',
        { sessionId: '', nodeId },
      );
      stageHistory = { ...stageHistory, [nodeId]: rows ?? [] };
    } catch (_err) {
      // Fall back to the stub wrapper which logs + returns [] on cold
      // start so the drawer still renders.
      const rows = await get_research_node_history('', nodeId);
      stageHistory = { ...stageHistory, [nodeId]: rows };
    }
  }

  $effect(() => {
    if (stageHistoryTimer) {
      clearInterval(stageHistoryTimer);
      stageHistoryTimer = null;
    }
    const ids = researchNodes.map((n) => n.id).filter((id) => id.length > 0);
    if (ids.length === 0) return;
    async function refreshAll() {
      for (const id of ids) {
        await refreshStageHistoryForNode(id);
      }
    }
    void refreshAll();
    stageHistoryTimer = setInterval(() => void refreshAll(), 5_000);
    return () => {
      if (stageHistoryTimer) {
        clearInterval(stageHistoryTimer);
        stageHistoryTimer = null;
      }
    };
  });

  function stagesForNode(nodeId: string): ResearchNodeHistoryRow[] {
    return stageHistory[nodeId] ?? [];
  }

  // ------------------------------------------------------------------
  // Section 3 — SOURCE QUALITY (last research run)
  // ------------------------------------------------------------------
  const sourceTiers = $derived<SourceTierDistribution[]>(researchData.sourceTiers);
  const tierTotal = $derived(sourceTiers.reduce((s, t) => s + t.count, 0) || 1);

  const TIER_COLORS: Record<string, string> = {
    T1: 'var(--teal)',
    T2: 'var(--steel)',
    T3: 'var(--text-secondary)',
    T4: 'var(--warn)',
    T5: 'var(--text-ghost)',
  };

  // ------------------------------------------------------------------
  // Section 5 — CONVERGENCE THRESHOLDS (phi / psi)
  // ------------------------------------------------------------------
  const phi = $derived(researchRoot.phi ?? 0);
  const psi = $derived(researchRoot.psi ?? 0);
  const PHI_TARGET = 0.75;
  const PSI_TARGET = 0.65;
</script>

<div class="research-drawer">
  <Scrollbar>
    <div class="drawer-body">

      <!-- ── DECOMPOSITION TREE MINI-MAP ─────────────────────── -->
      <section class="section">
        <div class="section-heading">DECOMPOSITION TREE</div>
        <div class="mini-tree">
          <Tree
            nodes={treeNodes}
            bind:expandedIds
            bind:selectedId
            onselect={handleTreeSelect}
          />
        </div>
      </section>

      <!-- ── RESEARCH TREE ──────────────────────────────────── -->
      <section class="section">
        <div class="section-heading">RESEARCH TREE</div>
        {#if researchNodes.length === 0 && loading}
          <SkeletonList rows={5} loading={loading} label="loading research stages" />
        {:else if researchNodes.length === 0}
          <div class="empty-note">No research nodes for this session.</div>
        {:else}
          <div class="research-nodes">
            {#each researchNodes as node, i}
              <div class="research-node" class:research-node--child={i > 0}>
                <div class="rn-query">{node.query}</div>
                <div class="rn-stages">
                  {#each STAGES as stage}
                    <div
                      class="rn-stage-dot"
                      class:rn-stage-dot--active={stage === node.stage && !node.converged}
                      style="background:{stageDotColor(stage, node.stage, node.converged)};"
                      title={stage}
                    ></div>
                  {/each}
                  <span class="rn-stage-label">{node.stage}</span>
                </div>
                <div class="rn-meta">
                  {node.source_count} sources
                  {#if node.phi !== undefined}
                    &middot; phi {node.phi.toFixed(2)}
                  {/if}
                  {#if node.psi !== undefined}
                    &middot; psi {node.psi.toFixed(2)}
                  {/if}
                  {#if node.converged}
                    <Badge tone="teal" size="xs">converged</Badge>
                  {/if}
                </div>
                <!-- B-7 (SCHEMA_VERSION 100): per-stage timeline rows
                     from rewired get_research_node_history. Renders only
                     when at least one stage-populated row exists for this
                     node so the card stays clean on cold start. -->
                {#if stagesForNode(node.id).length > 0}
                  <ul
                    class="rn-stage-timeline"
                    data-testid="research-stage-timeline"
                  >
                    {#each stagesForNode(node.id) as row, idx (idx)}
                      <li class="rn-stage-row">
                        <span class="rn-stage-row-stage">{row.stage ?? ''}</span>
                        <span class="rn-stage-row-phase">{row.stage_phase ?? ''}</span>
                        <span class="rn-stage-row-iter">iter {row.iteration}</span>
                        <span class="rn-stage-row-sources">{row.sources_used}/{row.sources_fetched}</span>
                      </li>
                    {/each}
                  </ul>
                {/if}
              </div>
            {/each}
          </div>
        {/if}
      </section>

      <!-- ── SOURCE QUALITY (LAST RESEARCH RUN) ─────────────── -->
      <section class="section">
        <div class="section-heading">SOURCE QUALITY</div>
        {#if researchData.unwired.sourceTiers}
          <UnwiredNote
            phase="A-45"
            feature="T1..T5 source-tier counts of last research run"
            note="get_source_quality_tiers returns zeros; SourceQualityTable not yet wired into research pipeline"
          />
        {:else}
          {#each sourceTiers as t}
            <div class="tier-row">
              <span class="tier-badge" style="color:{TIER_COLORS[t.tier] ?? 'var(--text-ghost)'};">{t.tier}</span>
              <div class="tier-bar-wrap">
                <div
                  class="tier-bar"
                  style="width:{(t.count / tierTotal) * 100}%; background:{TIER_COLORS[t.tier] ?? 'var(--text-ghost)'};"
                ></div>
              </div>
              <span class="tier-count">{t.count}</span>
            </div>
          {/each}
        {/if}
      </section>

      <!-- ── WEB RESEARCH BUDGET ────────────────────────────── -->
      <section class="section">
        <div class="section-heading">WEB RESEARCH BUDGET</div>
        <UnwiredNote
          phase="4D.16"
          feature="tokens used / sources fetched / budget % over last research run"
          note="ResearchBudgetManager not exposed via Tauri command; backend producer pending"
        />
      </section>

      <!-- ── CONVERGENCE THRESHOLDS ─────────────────────────── -->
      <section class="section section--last">
        <div class="section-heading">CONVERGENCE THRESHOLDS</div>
        {#if researchData.unwired.convergence || (phi === 0 && psi === 0)}
          <UnwiredNote
            phase="4D"
            feature="phi (goal-satisfaction) / psi (quality) thresholds with target markers"
            note="phi and psi populated only when /research is executed; no active research run yet"
          />
        {:else}
          <div class="threshold-row">
            <span class="threshold-label">phi (goal)</span>
            <div class="threshold-bar-wrap">
              <div
                class="threshold-fill"
                class:threshold-fill--pass={phi >= PHI_TARGET}
                style="width:{Math.min(phi * 100, 100)}%;"
              ></div>
              <div class="threshold-marker" style="left:{PHI_TARGET * 100}%;"></div>
            </div>
            <span class="threshold-vals">{phi.toFixed(2)} / {PHI_TARGET}</span>
          </div>

          <div class="threshold-row">
            <span class="threshold-label">psi (quality)</span>
            <div class="threshold-bar-wrap">
              <div
                class="threshold-fill"
                class:threshold-fill--pass={psi >= PSI_TARGET}
                style="width:{Math.min(psi * 100, 100)}%;"
              ></div>
              <div class="threshold-marker" style="left:{PSI_TARGET * 100}%;"></div>
            </div>
            <span class="threshold-vals">{psi.toFixed(2)} / {PSI_TARGET}</span>
          </div>
        {/if}
      </section>

    </div>
  </Scrollbar>
</div>

<style>
  .research-drawer {
    display: flex;
    flex-direction: column;
    height: 100%;
    overflow: hidden;
  }

  .drawer-body {
    padding: var(--s-4);
    display: flex;
    flex-direction: column;
    gap: var(--s-6);
  }

  .section-heading {
    font-family: var(--font-mono);
    font-size: 10px;
    font-weight: 500;
    letter-spacing: 0.08em;
    color: var(--text-secondary);
    text-transform: uppercase;
    margin-bottom: var(--s-3);
  }

  /* ── MINI-TREE ──────────────────────────────── */
  .mini-tree {
    overflow-y: auto;
    max-height: 220px;
  }

  /* Override Tree primitive to 20px rows / 10px font */
  .mini-tree :global(.tree-node) {
    height: 20px;
  }

  .mini-tree :global(.tree-label) {
    font-size: 10px;
  }

  .mini-tree :global(.tree-meta) {
    font-size: 10px;
  }

  /* ── RESEARCH NODES ─────────────────────────── */
  .research-nodes {
    display: flex;
    flex-direction: column;
    gap: var(--s-2);
  }

  .research-node {
    padding: var(--s-2) var(--s-3);
    border-radius: var(--r-sm);
    background: var(--bg-surface);
    border: 1px solid var(--border-subtle);
    display: flex;
    flex-direction: column;
    gap: 4px;
    transition: background-color var(--dur-fast) var(--ease-std);
  }

  .research-node--child {
    margin-left: var(--s-4);
    background: var(--bg-base);
  }

  .research-node:hover {
    background: var(--bg-hover);
  }

  .rn-query {
    font-family: var(--font-mono);
    font-size: 12px;
    color: var(--text-primary);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .rn-stages {
    display: flex;
    align-items: center;
    gap: 4px;
  }

  .rn-stage-dot {
    width: 6px;
    height: 6px;
    border-radius: 50%;
    flex-shrink: 0;
  }

  .rn-stage-dot--active {
    animation: breath 2.2s ease-in-out infinite;
  }

  @keyframes breath {
    0%   { opacity: 0.2; }
    50%  { opacity: 0.55; }
    100% { opacity: 0.2; }
  }

  .rn-stage-label {
    font-family: var(--font-mono);
    font-size: 10px;
    color: var(--text-secondary);
    letter-spacing: 0.04em;
    margin-left: var(--s-1);
  }

  .rn-meta {
    font-family: var(--font-mono);
    font-size: 10px;
    color: var(--text-ghost);
    display: flex;
    align-items: center;
    gap: var(--s-2);
    flex-wrap: wrap;
  }

  /* B-7 (SCHEMA_VERSION 100): per-stage timeline rows */
  .rn-stage-timeline {
    list-style: none;
    padding: 0;
    margin: var(--s-1) 0 0 0;
    display: flex;
    flex-direction: column;
    gap: 2px;
  }

  .rn-stage-row {
    display: grid;
    grid-template-columns: 1fr 0.8fr 0.6fr 0.6fr;
    gap: var(--s-2);
    font-family: var(--font-mono);
    font-size: 10px;
    color: var(--text-ghost);
  }

  .rn-stage-row > * {
    min-width: 0;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .rn-stage-row-stage {
    color: var(--text-secondary);
    letter-spacing: 0.04em;
  }

  .rn-stage-row-phase,
  .rn-stage-row-iter,
  .rn-stage-row-sources {
    text-align: right;
  }

  /* ── SOURCE QUALITY ─────────────────────────── */
  .tier-row {
    display: flex;
    align-items: center;
    gap: var(--s-2);
    padding: 3px 0;
  }

  .tier-badge {
    font-family: var(--font-mono);
    font-size: 10px;
    font-weight: 500;
    width: 20px;
    flex-shrink: 0;
  }

  .tier-bar-wrap {
    flex: 1;
    height: 4px;
    background: var(--bg-input);
    border-radius: var(--r-xs);
    overflow: hidden;
  }

  .tier-bar {
    height: 100%;
    border-radius: var(--r-xs);
    transition: width var(--dur-base) var(--ease-out);
  }

  .tier-count {
    font-family: var(--font-mono);
    font-size: 10px;
    color: var(--text-ghost);
    width: 20px;
    text-align: right;
    flex-shrink: 0;
  }

  /* ── BUDGET ─────────────────────────────────── */
  .budget-bar-wrap {
    margin-bottom: var(--s-2);
  }

  .budget-bar {
    height: 8px;
    background: var(--bg-input);
    border-radius: var(--r-xs);
    overflow: hidden;
  }

  .budget-fill {
    height: 100%;
    background: var(--teal);
    border-radius: var(--r-xs);
    transition: width var(--dur-base) var(--ease-out);
  }

  .budget-meta {
    font-family: var(--font-mono);
    font-size: 10px;
    color: var(--text-ghost);
  }

  /* ── THRESHOLDS ─────────────────────────────── */
  .threshold-row {
    display: flex;
    align-items: center;
    gap: var(--s-2);
    padding: 5px 0;
  }

  .threshold-label {
    font-family: var(--font-mono);
    font-size: 10px;
    color: var(--text-secondary);
    width: 90px;
    flex-shrink: 0;
  }

  .threshold-bar-wrap {
    flex: 1;
    height: 6px;
    background: var(--bg-input);
    border-radius: var(--r-xs);
    overflow: visible;
    position: relative;
  }

  .threshold-fill {
    height: 100%;
    background: var(--steel);
    border-radius: var(--r-xs);
    transition: width var(--dur-base) var(--ease-out);
  }

  .threshold-fill--pass {
    background: var(--teal);
  }

  .threshold-marker {
    position: absolute;
    top: -3px;
    bottom: -3px;
    width: 2px;
    background: var(--text-secondary);
    border-radius: 1px;
    transform: translateX(-50%);
  }

  .threshold-vals {
    font-family: var(--font-mono);
    font-size: 10px;
    color: var(--text-ghost);
    width: 72px;
    text-align: right;
    flex-shrink: 0;
  }

  .empty-note {
    font-family: var(--font-mono);
    font-size: 10px;
    color: var(--text-ghost);
    padding: var(--s-2) 0;
  }

  .section--last {
    padding-bottom: var(--s-6);
  }
</style>
