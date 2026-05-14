<script lang="ts">
  import { onMount } from 'svelte';
  import { invoke } from '@tauri-apps/api/core';
  import { plan } from '$lib/state/plan.svelte';
  import { fsm } from '$lib/state/fsm.svelte';
  import { activity } from '$lib/state/activity.svelte';
  import { workspace } from '$lib/state/workspace.svelte';
  import { get_bubble_up_events } from '$lib/api/stub';
  import { Tree, Badge, Button, Empty, Card, Scrollbar } from '$lib/ui';
  import type { TreeNode } from '$lib/ui';
  import type {
    DecompNode,
    BubbleUpEvent,
    BabyStepBlock,
    DecompLevel,
  } from '$lib/api/types';

  // ----------------------------------------------------------------
  // B-28 follow-up — focus-tunnel inline chips per baby-step.
  // Wire shape returned by `query_focus_tunnel_per_baby_step`:
  //   { mode: 'FIM'|'DIFF'|'STRUCTURED'|null,
  //     early_term: { reason: string, tokens_saved: number } | null,
  //     micro_steps: number,
  //     deps_auto: number,
  //     deps_blocked: number }
  // ----------------------------------------------------------------
  type EarlyTermSummary = {
    reason: string;
    tokens_saved: number;
  };
  type FocusTunnelPerStep = {
    mode: 'FIM' | 'DIFF' | 'STRUCTURED' | null;
    early_term: EarlyTermSummary | null;
    micro_steps: number;
    deps_auto: number;
    deps_blocked: number;
  };

  // Extract a numeric baby_step_id from a dashboard node id (e.g. 'bs-1.1.1'
  // -> last numeric component, 1). Returns `null` if no digits found — chips
  // then render zeroed/empty state gracefully.
  function babyStepIdFromNode(nodeId: string): number | null {
    const segments = nodeId.split(/[^0-9]+/).filter((s) => s.length > 0);
    if (segments.length === 0) return null;
    const last = segments[segments.length - 1];
    const n = Number.parseInt(last, 10);
    return Number.isFinite(n) ? n : null;
  }

  let focusTunnel = $state<FocusTunnelPerStep | null>(null);
  let focusTunnelTimer: ReturnType<typeof setInterval> | null = null;

  // Phase D wiring: poll bubble-up events from the backend instead of
  // reading the static mock array. plan + fsm + activity stores already
  // poll on Workspace.onMount.
  let bubbleUpEvents = $state<BubbleUpEvent[]>([]);
  onMount(() => {
    let alive = true;
    async function refresh() {
      try {
        const evts = await get_bubble_up_events();
        if (alive) bubbleUpEvents = evts;
      } catch (err) {
        console.debug('[PlanWindow] get_bubble_up_events failed', err);
      }
    }
    void refresh();
    const t = setInterval(refresh, 5_000);
    return () => { alive = false; clearInterval(t); };
  });

  function handleCtrlSelectNode(id: string) {
    // Ctrl+click: open Activity window in primary panel and scroll to this step
    workspace.setPrimaryWindow('activity');
    activity.scrollTo(id);
  }

  // ----------------------------------------------------------------
  // Convert DecompNode tree to TreeNode for the Tree primitive
  // ----------------------------------------------------------------
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
    if (ms < 1000) return `${ms}ms`;
    if (ms < 60_000) return `${(ms / 1000).toFixed(1)}s`;
    const m = Math.floor(ms / 60_000);
    const s = Math.floor((ms % 60_000) / 1000);
    return s > 0 ? `${m}m${s}s` : `${m}m`;
  }

  const treeRoot = $derived(decompToTree(plan.tree));
  const treeNodes = $derived([treeRoot]);

  // ----------------------------------------------------------------
  // Expand/collapse all
  // ----------------------------------------------------------------
  function collectAllIds(node: DecompNode, acc: string[] = []): string[] {
    acc.push(node.id);
    for (const child of node.children ?? []) {
      collectAllIds(child, acc);
    }
    return acc;
  }

  function expandAll() {
    const all = collectAllIds(plan.tree);
    plan.expandedIds; // reactive touch
    for (const id of all) {
      plan.expandNode(id);
    }
  }

  function collapseAll() {
    // Collapse everything — keep only root
    const next = new Set<string>();
    plan.expandedIds.forEach(() => {}); // reactive touch
    // Replace via toggles — simpler to directly rebuild
    const all = collectAllIds(plan.tree);
    for (const id of all) {
      plan.collapseNode(id);
    }
  }

  // ----------------------------------------------------------------
  // Selected node detail
  // ----------------------------------------------------------------
  function findNode(node: DecompNode, id: string): DecompNode | undefined {
    if (node.id === id) return node;
    for (const child of node.children ?? []) {
      const found = findNode(child, id);
      if (found) return found;
    }
    return undefined;
  }

  const selectedNode = $derived(
    plan.selectedId ? findNode(plan.tree, plan.selectedId) : null
  );

  // B-28 follow-up — poll focus-tunnel per-baby-step signal every 5s while
  // a baby-step node is selected. Clears state when selection changes or
  // the selected node is not a baby-step.
  $effect(() => {
    if (focusTunnelTimer) {
      clearInterval(focusTunnelTimer);
      focusTunnelTimer = null;
    }
    focusTunnel = null;
    const node = selectedNode;
    if (!node || node.level !== 'baby-step') {
      return;
    }
    const numericId = babyStepIdFromNode(node.id);
    if (numericId === null) {
      return;
    }
    let alive = true;
    async function refresh(): Promise<void> {
      try {
        const result = await invoke<FocusTunnelPerStep>(
          'query_focus_tunnel_per_baby_step',
          { babyStepId: numericId },
        );
        if (alive) focusTunnel = result;
      } catch (err) {
        // Tauri reject -> empty chip state; no console noise during dev.
        if (alive) focusTunnel = null;
      }
    }
    void refresh();
    focusTunnelTimer = setInterval(() => void refresh(), 5_000);
    return () => {
      alive = false;
      if (focusTunnelTimer) {
        clearInterval(focusTunnelTimer);
        focusTunnelTimer = null;
      }
    };
  });

  // Hardcoded sample BabyStep detail blocks for active baby step
  const SAMPLE_BLOCKS: { block: BabyStepBlock; status: 'done' | 'active' | 'pending' | 'failed'; detail: string }[] = [
    {
      block: 'GROUNDING',
      status: 'done',
      detail: 'RFC 7519 section 4.1.4: exp claim mandatory for token expiry. Verified via WebSearch. deadpool-redis 0.12 changelog: connection leak CVE fixed. SOURCE: official changelog.',
    },
    {
      block: 'PRE_GATE',
      status: 'done',
      detail: 'Gate 1: contract tests written for RevocationStore — PASS. Gate 2: no .unwrap() in production path — PASS. Gate 3: dep audit cleared — WARN (upgrade pending).',
    },
    {
      block: 'TASK',
      status: 'active',
      detail: 'Add EXIT_CRITERIA validation: revoked token must be rejected in <2ms p99. Increase deadpool max_size from 5 to 20; add connection_timeout 100ms.',
    },
    {
      block: 'TDD',
      status: 'pending',
      detail: 'tests/revocation_tests.rs: test_exit_criteria_latency_p99 — assert p99 < 2ms under 50-concurrent load. Expected: PASS after pool size fix.',
    },
    {
      block: 'DETERMINISTIC_CHECK',
      status: 'pending',
      detail: 'cargo test -p auth -- test_exit_criteria_latency_p99 --exact. Expected: 1 passed, 0 failed.',
    },
    {
      block: 'ACCEPTANCE',
      status: 'pending',
      detail: 'Binary criterion: revoked token latency p99 < 2ms measured over 100 concurrent requests.',
    },
    {
      block: 'INTEGRATION',
      status: 'pending',
      detail: 'Checkpoint level: BABY_STEP. Trigger after DETERMINISTIC_CHECK passes. No phase-level checkpoint yet.',
    },
    {
      block: 'EXIT_CRITERIA',
      status: 'pending',
      detail: '[ ] p99 < 2ms confirmed by latency benchmark. [ ] No CVE-2025-1234 in dependency tree. [ ] All 24 tests green.',
    },
  ];

  function blockStatusColor(s: string): string {
    if (s === 'done') return 'var(--teal)';
    if (s === 'active') return 'var(--teal-bright)';
    if (s === 'failed') return 'var(--fail)';
    return 'var(--text-ghost)';
  }

  function blockStatusLabel(s: string): string {
    return s.toUpperCase();
  }

  // ----------------------------------------------------------------
  // Child count for non-baby-step nodes
  // ----------------------------------------------------------------
  function childCount(node: DecompNode): number {
    return node.children?.length ?? 0;
  }

  // ----------------------------------------------------------------
  // BubbleUp event color
  // ----------------------------------------------------------------
  function bubbleColor(event: BubbleUpEvent): string {
    if (event.escalated) return 'var(--fail)';
    if (event.level === 'baby-step') return 'var(--steel)';
    if (event.level === 'step') return 'var(--steel)';
    if (event.level === 'phase') return 'var(--warn)';
    return 'var(--text-ghost)';
  }

  function levelLabel(l: DecompLevel): string {
    return l.toUpperCase();
  }

  // ----------------------------------------------------------------
  // Task name from root
  // ----------------------------------------------------------------
  const taskName = $derived(plan.tree.label);
</script>

<div class="plan-window">
  <!-- Toolbar -->
  <div class="toolbar">
    <div class="toolbar-left">
      <span class="task-name">{taskName}</span>
      <Badge tone="teal" size="xs">{plan.tree.status.toUpperCase()}</Badge>
    </div>
    <div class="toolbar-center">
      <div class="bubble-strip">
        {#each bubbleUpEvents as ev (ev.ts)}
          <div
            class="bubble-chip"
            title="{ev.message}"
            style="border-color:{bubbleColor(ev)};"
          >
            <span class="bubble-level" style="color:{bubbleColor(ev)};">{levelLabel(ev.level)}</span>
            <span class="bubble-node">{ev.node_id}</span>
          </div>
        {/each}
      </div>
    </div>
    <div class="toolbar-right">
      <button class="text-btn" onclick={expandAll} type="button">expand all</button>
      <span class="divider-char">/</span>
      <button class="text-btn" onclick={collapseAll} type="button">collapse all</button>
    </div>
  </div>

  <!-- Body: tree + detail -->
  <div class="body">
    <!-- Tree column -->
    <div class="tree-col">
      {#if !plan.tree || !plan.tree.children}
        <Empty title="no plan loaded" subtitle="submit a task to generate a decomposition tree" />
      {:else}
        <div class="tree-scroll">
          <Tree
            nodes={treeNodes}
            expandedIds={plan.expandedIds}
            selectedId={plan.selectedId ?? ''}
            onselect={(id) => plan.selectNode(id)}
            onctrselect={handleCtrlSelectNode}
          />
        </div>
      {/if}
    </div>

    <!-- Vertical divider -->
    <div class="col-divider" aria-hidden="true"></div>

    <!-- Detail column -->
    <div class="detail-col">
      {#if !selectedNode}
        <div class="detail-empty">
          <span class="detail-hint">select a node to inspect</span>
        </div>
      {:else}
        <div class="detail-scroll">
          <div class="detail-header">
            <span class="detail-level">{selectedNode.level.toUpperCase()}</span>
            <h3 class="detail-title">{selectedNode.label}</h3>
            {#if selectedNode.elapsed_ms && selectedNode.elapsed_ms > 0}
              <span class="detail-elapsed">{formatElapsed(selectedNode.elapsed_ms)}</span>
            {/if}
          </div>

          {#if selectedNode.level !== 'baby-step'}
            <!-- Concept / Phase / Step detail -->
            <div class="detail-section">
              <div class="meta-row">
                <span class="meta-label">STATUS</span>
                <span
                  class="meta-value"
                  style="color:{selectedNode.status === 'done' ? 'var(--teal)' : selectedNode.status === 'active' ? 'var(--teal-bright)' : selectedNode.status === 'failed' ? 'var(--fail)' : 'var(--text-ghost)'};"
                >{selectedNode.status.toUpperCase()}</span>
              </div>
              {#if childCount(selectedNode) > 0}
                <div class="meta-row">
                  <span class="meta-label">CHILDREN</span>
                  <span class="meta-value">{childCount(selectedNode)}</span>
                </div>
              {/if}
              {#if selectedNode.elapsed_ms}
                <div class="meta-row">
                  <span class="meta-label">ELAPSED</span>
                  <span class="meta-value">{formatElapsed(selectedNode.elapsed_ms)}</span>
                </div>
              {/if}
            </div>

            {#if selectedNode.children && selectedNode.children.length > 0}
              <div class="children-list">
                {#each selectedNode.children as child (child.id)}
                  <div class="child-row">
                    <span
                      class="child-dot"
                      style="background:{child.status === 'done' ? 'var(--teal)' : child.status === 'active' ? 'var(--teal-bright)' : child.status === 'failed' ? 'var(--fail)' : 'var(--text-ghost)'};"
                    ></span>
                    <span class="child-label">{child.label}</span>
                    {#if child.elapsed_ms && child.elapsed_ms > 0}
                      <span class="child-elapsed">{formatElapsed(child.elapsed_ms)}</span>
                    {/if}
                  </div>
                {/each}
              </div>
            {/if}

          {:else}
            <!-- B-28 follow-up: focus-tunnel inline chips. Render only when
                 the per-baby-step query returned non-trivial data so the
                 baby-step card stays clean when the step has not yet
                 emitted any focus-tunnel beacons. -->
            {#if focusTunnel && (focusTunnel.mode || focusTunnel.early_term || focusTunnel.micro_steps > 0 || focusTunnel.deps_auto > 0 || focusTunnel.deps_blocked > 0)}
              <div class="focus-chips" aria-label="Focus tunnel signals">
                {#if focusTunnel.mode}
                  <span class="chip chip-mode chip-mode-{focusTunnel.mode.toLowerCase()}">
                    <span class="chip-label">mode</span>
                    <span class="chip-value">{focusTunnel.mode}</span>
                  </span>
                {/if}
                {#if focusTunnel.early_term}
                  <span class="chip chip-term" title={`Tokens saved: ${focusTunnel.early_term.tokens_saved}`}>
                    <span class="chip-label">early-term</span>
                    <span class="chip-value">{focusTunnel.early_term.reason}</span>
                  </span>
                {/if}
                {#if focusTunnel.micro_steps > 0}
                  <span class="chip chip-micro">
                    <span class="chip-label">micro-steps</span>
                    <span class="chip-value">{focusTunnel.micro_steps}</span>
                  </span>
                {/if}
                {#if focusTunnel.deps_auto > 0 || focusTunnel.deps_blocked > 0}
                  <span class="chip chip-deps" title="Task-level dependency resolution totals">
                    <span class="chip-label">deps</span>
                    <span class="chip-value">{focusTunnel.deps_auto} auto / {focusTunnel.deps_blocked} blocked</span>
                  </span>
                {/if}
              </div>
            {/if}

            <!-- BabyStep detail: 8 block spec sections -->
            {@const blocks = selectedNode.detail?.blocks ?? SAMPLE_BLOCKS.map(b => ({ block: b.block, status: b.status }))}
            <div class="blocks-list">
              {#each SAMPLE_BLOCKS as spec (spec.block)}
                {@const liveBlock = blocks.find(b => b.block === spec.block)}
                {@const status = liveBlock?.status ?? 'pending'}
                <div class="block-section">
                  <div class="block-header">
                    <span class="block-name">{spec.block}</span>
                    <span
                      class="block-status"
                      style="color:{blockStatusColor(status)};"
                    >{blockStatusLabel(status)}</span>
                  </div>
                  <p class="block-detail">{spec.detail}</p>
                </div>
              {/each}
            </div>
          {/if}
        </div>
      {/if}
    </div>
  </div>
</div>

<style>
  .plan-window {
    display: flex;
    flex-direction: column;
    height: 100%;
    overflow: hidden;
    background: var(--bg-base);
  }

  /* ---- Toolbar ---- */
  .toolbar {
    height: 28px;
    min-height: 28px;
    display: flex;
    align-items: center;
    gap: var(--s-3);
    padding: 0 var(--s-3);
    background: var(--bg-surface);
    border-bottom: 1px solid var(--border-subtle);
    flex-shrink: 0;
    overflow: hidden;
  }

  .toolbar-left {
    display: flex;
    align-items: center;
    gap: var(--s-2);
    flex-shrink: 0;
    min-width: 0;
  }

  .task-name {
    font-family: var(--font-sans);
    font-size: 12px;
    font-weight: 500;
    color: var(--text-primary);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
    max-width: 200px;
  }

  .toolbar-center {
    flex: 1;
    overflow: hidden;
  }

  .bubble-strip {
    display: flex;
    align-items: center;
    gap: 4px;
    overflow-x: auto;
    scrollbar-width: none;
    padding: 2px 0;
  }

  .bubble-strip::-webkit-scrollbar {
    display: none;
  }

  .bubble-chip {
    display: inline-flex;
    align-items: center;
    gap: 4px;
    height: 18px;
    padding: 0 6px;
    border-radius: var(--r-xs);
    border: 1px solid var(--border-subtle);
    flex-shrink: 0;
    cursor: default;
  }

  .bubble-level {
    font-family: var(--font-mono);
    font-size: 9px;
    font-weight: 600;
    letter-spacing: 0.06em;
  }

  .bubble-node {
    font-family: var(--font-mono);
    font-size: 10px;
    color: var(--text-ghost);
  }

  .toolbar-right {
    display: flex;
    align-items: center;
    gap: 2px;
    flex-shrink: 0;
  }

  .text-btn {
    background: transparent;
    border: none;
    color: var(--text-ghost);
    font-family: var(--font-mono);
    font-size: 11px;
    cursor: pointer;
    padding: 2px 4px;
    border-radius: var(--r-xs);
    transition: color var(--dur-fast) var(--ease-std);
  }

  .text-btn:hover {
    color: var(--text-primary);
  }

  .divider-char {
    color: var(--text-ghost);
    font-family: var(--font-mono);
    font-size: 11px;
    user-select: none;
  }

  /* ---- Body ---- */
  .body {
    flex: 1;
    display: flex;
    overflow: hidden;
  }

  /* ---- Tree column ---- */
  .tree-col {
    width: 60%;
    min-width: 0;
    display: flex;
    flex-direction: column;
    overflow: hidden;
  }

  .tree-scroll {
    flex: 1;
    overflow-y: auto;
    padding: var(--s-2);
    scrollbar-width: thin;
    scrollbar-color: var(--bg-hover) transparent;
  }

  .tree-scroll::-webkit-scrollbar {
    width: 4px;
  }

  .tree-scroll::-webkit-scrollbar-thumb {
    background: var(--bg-hover);
    border-radius: 2px;
  }

  /* ---- Column divider ---- */
  .col-divider {
    width: 1px;
    background: var(--border-subtle);
    flex-shrink: 0;
  }

  /* ---- Detail column ---- */
  .detail-col {
    width: 40%;
    min-width: 0;
    display: flex;
    flex-direction: column;
    overflow: hidden;
  }

  .detail-empty {
    flex: 1;
    display: flex;
    align-items: center;
    justify-content: center;
  }

  .detail-hint {
    font-family: var(--font-mono);
    font-size: 11px;
    color: var(--text-ghost);
  }

  .detail-scroll {
    flex: 1;
    overflow-y: auto;
    padding: var(--s-3);
    scrollbar-width: thin;
    scrollbar-color: var(--bg-hover) transparent;
    display: flex;
    flex-direction: column;
    gap: var(--s-2);
  }

  .detail-scroll::-webkit-scrollbar {
    width: 4px;
  }

  .detail-scroll::-webkit-scrollbar-thumb {
    background: var(--bg-hover);
    border-radius: 2px;
  }

  /* ---- Detail header ---- */
  .detail-header {
    display: flex;
    flex-direction: column;
    gap: 2px;
    margin-bottom: var(--s-2);
  }

  .detail-level {
    font-family: var(--font-mono);
    font-size: 10px;
    letter-spacing: 0.08em;
    color: var(--text-ghost);
  }

  .detail-title {
    font-family: var(--font-sans);
    font-size: 14px;
    font-weight: 500;
    color: var(--text-primary);
    margin: 0;
    line-height: 1.3;
  }

  .detail-elapsed {
    font-family: var(--font-mono);
    font-size: 11px;
    color: var(--text-ghost);
    margin-top: 2px;
  }

  /* ---- Concept/Phase/Step detail section ---- */
  .detail-section {
    background: var(--bg-surface);
    border-radius: var(--r-sm);
    padding: var(--s-2) var(--s-3);
    display: flex;
    flex-direction: column;
    gap: 4px;
  }

  .meta-row {
    display: flex;
    align-items: center;
    gap: var(--s-2);
  }

  .meta-label {
    font-family: var(--font-mono);
    font-size: 10px;
    letter-spacing: 0.08em;
    color: var(--text-ghost);
    min-width: 64px;
    flex-shrink: 0;
  }

  .meta-value {
    font-family: var(--font-mono);
    font-size: 11px;
    color: var(--text-secondary);
  }

  .children-list {
    display: flex;
    flex-direction: column;
    gap: 2px;
    padding: 0 var(--s-1);
  }

  .child-row {
    display: flex;
    align-items: center;
    gap: 6px;
    padding: 2px 0;
  }

  .child-dot {
    width: 5px;
    height: 5px;
    border-radius: 50%;
    flex-shrink: 0;
  }

  .child-label {
    font-family: var(--font-sans);
    font-size: 12px;
    color: var(--text-secondary);
    flex: 1;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .child-elapsed {
    font-family: var(--font-mono);
    font-size: 10px;
    color: var(--text-ghost);
    flex-shrink: 0;
  }

  /* ---- B-28 follow-up: focus-tunnel inline chips ---- */
  .focus-chips {
    display: flex;
    flex-wrap: wrap;
    gap: var(--s-1);
    margin-bottom: var(--s-2);
  }

  .chip {
    display: inline-flex;
    align-items: baseline;
    gap: 4px;
    padding: 2px 6px;
    border-radius: var(--r-xs);
    border: 1px solid var(--border-subtle);
    background: transparent;
    font-family: var(--font-mono);
    font-size: 10px;
    line-height: 1.2;
  }

  .chip-label {
    color: var(--text-ghost);
    letter-spacing: 0.04em;
    text-transform: uppercase;
  }

  .chip-value {
    color: var(--text-secondary);
    font-weight: 500;
  }

  .chip-mode-fim .chip-value { color: var(--teal); }
  .chip-mode-diff .chip-value { color: var(--warn); }
  .chip-mode-structured .chip-value { color: var(--teal-bright); }
  .chip-term .chip-value { color: var(--warn); }
  .chip-micro .chip-value { color: var(--teal); }
  .chip-deps .chip-value { color: var(--text-secondary); }

  /* ---- BabyStep blocks ---- */
  .blocks-list {
    display: flex;
    flex-direction: column;
    gap: var(--s-2);
  }

  .block-section {
    background: var(--bg-surface);
    border-radius: var(--r-sm);
    padding: var(--s-2) var(--s-3);
  }

  .block-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    margin-bottom: 4px;
  }

  .block-name {
    font-family: var(--font-mono);
    font-size: 10px;
    font-weight: 600;
    letter-spacing: 0.08em;
    color: var(--text-ghost);
  }

  .block-status {
    font-family: var(--font-mono);
    font-size: 10px;
    letter-spacing: 0.06em;
  }

  .block-detail {
    font-family: var(--font-sans);
    font-size: 12px;
    color: var(--text-secondary);
    line-height: 1.5;
    margin: 0;
    word-break: break-word;
  }
</style>
