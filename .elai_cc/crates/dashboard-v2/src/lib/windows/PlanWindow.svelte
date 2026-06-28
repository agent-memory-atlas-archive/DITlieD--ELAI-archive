<script lang="ts">
  import { onMount } from 'svelte';
  import { invoke } from '@tauri-apps/api/core';
  import { plan } from '$lib/state/plan.svelte';
  import { fsm } from '$lib/state/fsm.svelte';
  import { activity } from '$lib/state/activity.svelte';
  import { workspace } from '$lib/state/workspace.svelte';
  import { graph } from '$lib/state/graph.svelte';
  import { get_bubble_up_events, get_package_verifications, get_executable_pipeline, get_phase_sized_splits, get_tree_positions, get_failure_records } from '$lib/api/stub';
  import { Tree, Badge, Button, Empty, Card, Scrollbar } from '$lib/ui';
  import SkeletonList from '$lib/components/SkeletonList.svelte';
  import NodeStateGlyph from '$lib/canvas/NodeStateGlyph.svelte';
  import { toQuinary } from '$lib/canvas/canvas_state';
  import type { TreeNode } from '$lib/ui';
  import type {
    DecompNode,
    BubbleUpEvent,
    BabyStepBlock,
    PackageRefVerified,
    PhaseSizedSplit,
    ExecutablePipelineDag,
    PipelineNode,
    RepairLogEntry,
    TreePosition,
    FailureRecord,
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

  // ----------------------------------------------------------------
  // C-29: Inline IntentPairEditor (per-BabyStep T-2 intent grounding).
  // Reads via `get_intent_pairs(step_id)`, writes via `add_intent_pair`
  // + `confirm_intent_pairs`. The Confirm button enables only when
  // pair count >= MIN_INTENT_PAIRS (default 3 per [testing.intent]
  // min_pairs). Char bounds enforced client-side mirror the Rust
  // constructor (120/120/80) so the editor surfaces violations before
  // the round-trip rejection.
  // ----------------------------------------------------------------
  const MIN_INTENT_PAIRS = 3;
  const INTENT_MAX_CHARS = 120;
  const ASSERTION_MAX_CHARS = 120;

  type IntentPairLite = {
    natural_language_intent: string;
    concrete_test_assertion: string;
    baby_step_ref: string;
  };
  type IntentPairSetLite = {
    pairs: IntentPairLite[];
    confirmed: boolean;
    confirmed_at_ms: number | null;
  };

  let intentPairs = $state<IntentPairSetLite>({
    pairs: [],
    confirmed: false,
    confirmed_at_ms: null,
  });
  let intentPairsTimer: ReturnType<typeof setInterval> | null = null;
  let intentPairForm = $state({ intent: '', assertion: '' });
  let intentPairError = $state<string | null>(null);
  let intentPairBusy = $state(false);

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

  // B-7 (SCHEMA_VERSION 100): producer-wire — poll PackageRefVerified
  // rows from the orchestrator SSB viewer. Rendered as a PACKAGES VERIFIED
  // row in the concept / phase / step detail pane (skipped at baby-step
  // level because the row applies to the parent decision-source context,
  // not to an individual baby step).
  let packageVerifications = $state<PackageRefVerified[]>([]);
  onMount(() => {
    let alive = true;
    async function refresh() {
      try {
        const rows = await get_package_verifications();
        if (alive) packageVerifications = rows;
      } catch (err) {
        console.debug('[PlanWindow] get_package_verifications failed', err);
      }
    }
    void refresh();
    const t = setInterval(refresh, 5_000);
    return () => { alive = false; clearInterval(t); };
  });

  // Derived rollup: total verified + total checked, used by the
  // detail-pane PACKAGES VERIFIED row.
  const packageVerifyStats = $derived({
    total: packageVerifications.length,
    verified: packageVerifications.filter((p) => p.verified).length,
  });

  // B-7 Task #8: PhaseSizingSplitter split-badge surface. Polls
  // get_phase_sized_splits from the orchestrator SSB viewer. Rendered
  // as a SPLIT badge on phase nodes whose phase_id matches a split row.
  let phaseSizedSplits = $state<PhaseSizedSplit[]>([]);
  onMount(() => {
    let alive = true;
    async function refresh() {
      try {
        const rows = await get_phase_sized_splits();
        if (alive) phaseSizedSplits = rows;
      } catch (err) {
        console.debug('[PlanWindow] get_phase_sized_splits failed', err);
      }
    }
    void refresh();
    const t = setInterval(refresh, 5_000);
    return () => { alive = false; clearInterval(t); };
  });

  // Returns the PhaseSizedSplit row for a given phase node, or null.
  // Phase node ids in the dashboard follow the pattern 'phase-N' where
  // N is the numeric phase_id. We extract the trailing number and match
  // it against the split row's phase_id field.
  function splitForNode(nodeId: string): PhaseSizedSplit | null {
    const segments = nodeId.split(/[^0-9]+/).filter((s) => s.length > 0);
    if (segments.length === 0) return null;
    const n = Number.parseInt(segments[segments.length - 1], 10);
    if (!Number.isFinite(n)) return null;
    return phaseSizedSplits.find((s) => s.phase_id === n) ?? null;
  }

  // ----------------------------------------------------------------
  // B-7 Step 9: TreeNavigator state cursor. Polls get_tree_positions
  // from the orchestrator SSB viewer. The latest row's current_node
  // is passed to the Tree component as cursorId to highlight the node
  // the navigator is currently processing (distinct from user selection).
  // ----------------------------------------------------------------
  let treePositions = $state<TreePosition[]>([]);
  onMount(() => {
    let alive = true;
    async function refresh() {
      try {
        const rows = await get_tree_positions();
        if (alive) treePositions = rows;
      } catch (err) {
        console.debug('[PlanWindow] get_tree_positions failed', err);
      }
    }
    void refresh();
    const t = setInterval(refresh, 5_000);
    return () => { alive = false; clearInterval(t); };
  });

  // Latest cursor node id (empty string when no rows or traversal exhausted).
  const cursorNodeId = $derived(
    treePositions.length > 0 ? treePositions[0].current_node : '',
  );

  // ----------------------------------------------------------------
  // B-7 Step 10: per-baby-step failure retry history. Polls
  // get_failure_records from the orchestrator SSB viewer. The baby-step
  // detail pane filters these by baby_step_id to show the retry trail
  // (reason + attempts per FailureRecord upsert inside handle_failure).
  // ----------------------------------------------------------------
  let failureRecords = $state<FailureRecord[]>([]);
  onMount(() => {
    let alive = true;
    async function refresh() {
      try {
        const rows = await get_failure_records();
        if (alive) failureRecords = rows;
      } catch (err) {
        console.debug('[PlanWindow] get_failure_records failed', err);
      }
    }
    void refresh();
    const t = setInterval(refresh, 5_000);
    return () => { alive = false; clearInterval(t); };
  });

  // Derived: failure records for the currently selected baby-step node.
  // Uses babyStepIdFromNode to extract the numeric id from the dashboard
  // node id (e.g. 'bs-1.1.3' -> 3) and matches against baby_step_id.
  const selectedBabyStepFailures = $derived.by(() => {
    const node = selectedNode;
    if (!node || node.level !== 'baby-step') return [];
    const numericId = babyStepIdFromNode(node.id);
    if (numericId === null) return [];
    return failureRecords.filter((f) => f.baby_step_id === numericId);
  });

  // ----------------------------------------------------------------
  // B-7 STEP 7: ExecutablePipeline DAG visualizer. Polls the
  // get_executable_pipeline Tauri command (which returns the full DAG
  // + validation_errors + repair_log) and renders a lightweight SVG
  // graph in the detail pane. No d3/cytoscape dependency — the DAG is
  // a strict hierarchy (concept → phase → step → baby_step), so a
  // level-based SVG layout suffices.
  // ----------------------------------------------------------------
  let pipelineDag = $state<ExecutablePipelineDag>({
    nodes: [],
    validation_errors: [],
    repair_log: [],
  });
  let dagExpanded = $state(false);

  onMount(() => {
    let alive = true;
    async function refresh() {
      try {
        const dag = await get_executable_pipeline();
        if (alive) pipelineDag = dag;
      } catch (err) {
        console.debug('[PlanWindow] get_executable_pipeline failed', err);
      }
    }
    void refresh();
    const t = setInterval(refresh, 5_000);
    return () => { alive = false; clearInterval(t); };
  });

  // SVG layout: assign each node an (x, y) position based on its depth
  // in the dependency hierarchy. Root nodes (no deps) are at level 0.
  // Nodes are spaced horizontally within each level.
  const DAG_NODE_W = 120;
  const DAG_NODE_H = 28;
  const DAG_LEVEL_GAP = 50;
  const DAG_NODE_GAP = 12;

  const dagLayout = $derived.by(() => {
    const nodes = pipelineDag.nodes;
    if (nodes.length === 0) return { positioned: [], edges: [], width: 0, height: 0 };

    // Build adjacency: dep -> dependents
    const nodeMap = new Map<string, PipelineNode>();
    for (const n of nodes) nodeMap.set(n.id, n);

    // Compute depth (longest path from any root) via topological levels.
    const depthCache = new Map<string, number>();
    function depth(id: string): number {
      if (depthCache.has(id)) return depthCache.get(id)!;
      const node = nodeMap.get(id);
      if (!node || node.deps.length === 0) {
        depthCache.set(id, 0);
        return 0;
      }
      const d = 1 + Math.max(...node.deps.map((dep) => {
        const depNode = nodeMap.get(dep);
        return depNode ? depth(dep) : 0;
      }));
      depthCache.set(id, d);
      return d;
    }

    // Group nodes by depth level.
    const levels = new Map<number, PipelineNode[]>();
    for (const n of nodes) {
      const d = depth(n.id);
      if (!levels.has(d)) levels.set(d, []);
      levels.get(d)!.push(n);
    }

    const maxDepth = Math.max(...levels.keys(), 0);
    const positioned: { node: PipelineNode; x: number; y: number }[] = [];
    const edges: { from: string; to: string; x1: number; y1: number; x2: number; y2: number }[] = [];

    let maxWidth = 0;
    for (let lvl = 0; lvl <= maxDepth; lvl++) {
      const levelNodes = levels.get(lvl) ?? [];
      const levelWidth = levelNodes.length * (DAG_NODE_W + DAG_NODE_GAP) - DAG_NODE_GAP;
      if (levelWidth > maxWidth) maxWidth = levelWidth;
    }

    const posMap = new Map<string, { x: number; y: number }>();
    for (let lvl = 0; lvl <= maxDepth; lvl++) {
      const levelNodes = levels.get(lvl) ?? [];
      const levelWidth = levelNodes.length * (DAG_NODE_W + DAG_NODE_GAP) - DAG_NODE_GAP;
      const startX = (maxWidth - levelWidth) / 2;
      for (let i = 0; i < levelNodes.length; i++) {
        const x = startX + i * (DAG_NODE_W + DAG_NODE_GAP);
        const y = lvl * (DAG_NODE_H + DAG_LEVEL_GAP);
        posMap.set(levelNodes[i].id, { x, y });
        positioned.push({ node: levelNodes[i], x, y });
      }
    }

    for (const n of nodes) {
      const to = posMap.get(n.id);
      if (!to) continue;
      for (const dep of n.deps) {
        const from = posMap.get(dep);
        if (!from) continue;
        edges.push({
          from: dep,
          to: n.id,
          x1: from.x + DAG_NODE_W / 2,
          y1: from.y + DAG_NODE_H,
          x2: to.x + DAG_NODE_W / 2,
          y2: to.y,
        });
      }
    }

    return {
      positioned,
      edges,
      width: maxWidth,
      height: (maxDepth + 1) * (DAG_NODE_H + DAG_LEVEL_GAP) - DAG_LEVEL_GAP,
    };
  });

  function dagNodeStatusColor(status: string): string {
    switch (status) {
      case 'done': return 'var(--d-green, #4ade80)';
      case 'active': return 'var(--d-teal, #2dd4bf)';
      case 'failed': return 'var(--d-red, #f87171)';
      case 'skipped': return 'var(--dim, #6b7280)';
      default: return 'var(--raised, #374151)';
    }
  }

  function dagNodeKindLabel(kind: string): string {
    return kind.toUpperCase();
  }

  function repairLogClass(entry: RepairLogEntry): string {
    return entry.escalated ? 'repair-entry repair-escalated' : 'repair-entry';
  }

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

  // ----------------------------------------------------------------
  // C-29: IntentPair poller — mirrors the focus-tunnel pattern.
  // Polls get_intent_pairs(step_id) every 5s; rebuilds on selection.
  // ----------------------------------------------------------------
  $effect(() => {
    if (intentPairsTimer) {
      clearInterval(intentPairsTimer);
      intentPairsTimer = null;
    }
    intentPairs = { pairs: [], confirmed: false, confirmed_at_ms: null };
    intentPairError = null;
    intentPairForm = { intent: '', assertion: '' };
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
        const result = await invoke<IntentPairSetLite>(
          'get_intent_pairs',
          { step_id: numericId },
        );
        if (alive && result) intentPairs = result;
      } catch (_err) {
        // Tauri reject -> keep last good state; surface in editor only on user action.
      }
    }
    void refresh();
    intentPairsTimer = setInterval(() => void refresh(), 5_000);
    return () => {
      alive = false;
      if (intentPairsTimer) {
        clearInterval(intentPairsTimer);
        intentPairsTimer = null;
      }
    };
  });

  async function handleAddIntentPair(): Promise<void> {
    const node = selectedNode;
    if (!node || node.level !== 'baby-step') return;
    const numericId = babyStepIdFromNode(node.id);
    if (numericId === null) return;
    const intent = intentPairForm.intent.trim();
    const assertion = intentPairForm.assertion.trim();
    if (intent.length === 0 || assertion.length === 0) {
      intentPairError = 'both fields are required';
      return;
    }
    if (intent.length > INTENT_MAX_CHARS) {
      intentPairError = `natural_language_intent > ${INTENT_MAX_CHARS} chars`;
      return;
    }
    if (assertion.length > ASSERTION_MAX_CHARS) {
      intentPairError = `concrete_test_assertion > ${ASSERTION_MAX_CHARS} chars`;
      return;
    }
    intentPairBusy = true;
    intentPairError = null;
    try {
      await invoke('add_intent_pair', {
        step_id: numericId,
        natural_language_intent: intent,
        concrete_test_assertion: assertion,
      });
      intentPairForm = { intent: '', assertion: '' };
      // Optimistic refresh — the next poller tick will reconcile.
      const result = await invoke<IntentPairSetLite>(
        'get_intent_pairs',
        { step_id: numericId },
      );
      if (result) intentPairs = result;
    } catch (err) {
      intentPairError = String(err);
    } finally {
      intentPairBusy = false;
    }
  }

  async function handleConfirmIntentPairs(): Promise<void> {
    const node = selectedNode;
    if (!node || node.level !== 'baby-step') return;
    const numericId = babyStepIdFromNode(node.id);
    if (numericId === null) return;
    if (intentPairs.pairs.length < MIN_INTENT_PAIRS) {
      intentPairError = `need ${MIN_INTENT_PAIRS - intentPairs.pairs.length} more pair(s)`;
      return;
    }
    intentPairBusy = true;
    intentPairError = null;
    try {
      await invoke('confirm_intent_pairs', {
        step_id: numericId,
        min_pairs: MIN_INTENT_PAIRS,
      });
      const result = await invoke<IntentPairSetLite>(
        'get_intent_pairs',
        { step_id: numericId },
      );
      if (result) intentPairs = result;
    } catch (err) {
      intentPairError = String(err);
    } finally {
      intentPairBusy = false;
    }
  }

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

  function blockStatusLabel(s: string): string {
    return s.toUpperCase();
  }

  // ----------------------------------------------------------------
  // Child count for non-baby-step nodes
  // ----------------------------------------------------------------
  function childCount(node: DecompNode): number {
    return node.children?.length ?? 0;
  }

  // B-7 STEP 11: BubbleUp cascade hierarchy visualization.
  // The typed payload carries from_level / to_level / action_kind / reason.
  // We render a 4-level cascade bar: BabyStep → Step → Phase → Concept,
  // with each event shown as an arrow chip on the segment it traverses.
  // Same-level events (from_level == to_level) are shown on the level node
  // itself as a threshold-miss indicator.

  // State severity from action_kind — escalate is the loud one (fail);
  // replan_phase / replan_concept are warn; lower levels stay quiet.
  function bubbleState(event: BubbleUpEvent): 'fail' | 'warn' | 'pending' {
    if (event.action_kind === 'escalate') return 'fail';
    if (event.action_kind === 'replan_phase' || event.action_kind === 'replan_concept') return 'warn';
    return 'pending';
  }

  // Short label for the action_kind discriminant.
  function actionLabel(kind: string): string {
    const map: Record<string, string> = {
      'research_alternative': 'RESEARCH',
      'redecompose_step': 'REDECOMP',
      'replan_phase': 'REPLAN-P',
      'replan_concept': 'REPLAN-C',
      'diagnose_and_retry': 'RETRY',
      'escalate': 'ESCALATE',
    };
    return map[kind] ?? kind.toUpperCase();
  }

  // The 4 cascade levels in order from lowest to highest.
  const CASCADE_LEVELS = ['BabyStep', 'Step', 'Phase', 'Concept'] as const;

  // Short label for a cascade level node.
  function cascadeNodeLabel(level: string): string {
    const map: Record<string, string> = {
      'BabyStep': 'BS',
      'Step': 'S',
      'Phase': 'P',
      'Concept': 'C',
    };
    return map[level] ?? level.slice(0, 2).toUpperCase();
  }

  // Count of events at each level node (same-level threshold misses).
  function sameLevelCount(level: string): number {
    return bubbleUpEvents.filter((e) => e.from_level === level && e.to_level === level).length;
  }

  // Count of events on a cascade segment (from → to).
  function segmentCount(from: string, to: string): number {
    return bubbleUpEvents.filter((e) => e.from_level === from && e.to_level === to).length;
  }

  // Severity of a cascade segment or level node.
  function segmentSeverity(from: string, to: string): 'fail' | 'warn' | 'pending' {
    const events = bubbleUpEvents.filter((e) => e.from_level === from && e.to_level === to);
    if (events.some((e) => e.action_kind === 'escalate')) return 'fail';
    if (events.some((e) => e.action_kind === 'replan_phase' || e.action_kind === 'replan_concept')) return 'warn';
    return 'pending';
  }

  // Tooltip text for a cascade segment.
  function segmentTooltip(from: string, to: string): string {
    const events = bubbleUpEvents.filter((e) => e.from_level === from && e.to_level === to);
    return events.map((e) => `${actionLabel(e.action_kind)}: ${e.reason} (${e.failure_count}/${e.threshold})`).join('\n');
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
      <!-- B-7 STEP 11: Cascade hierarchy visualization with arrows
           BabyStep → Step → Phase → Concept. Each level node shows a
           count badge for same-level threshold misses; each arrow
           segment shows a count badge for cascade lifts. -->
      <div class="cascade-bar" aria-label="Bubble-up cascade hierarchy">
        {#if bubbleUpEvents.length === 0}
          <span class="cascade-empty">no bubble-up events</span>
        {:else}
          {#each CASCADE_LEVELS as lvl, i (lvl)}
            <div
              class="cascade-node"
              class:has-miss={sameLevelCount(lvl) > 0}
              class:sev-fail={sameLevelCount(lvl) > 0 && segmentSeverity(lvl, lvl) === 'fail'}
              class:sev-warn={sameLevelCount(lvl) > 0 && segmentSeverity(lvl, lvl) === 'warn'}
              title={sameLevelCount(lvl) > 0 ? segmentTooltip(lvl, lvl) : `${lvl} level`}
            >
              <span class="cascade-node-label">{cascadeNodeLabel(lvl)}</span>
              {#if sameLevelCount(lvl) > 0}
                <span class="cascade-node-count">{sameLevelCount(lvl)}</span>
              {/if}
            </div>
            {#if i < CASCADE_LEVELS.length - 1}
              {@const nextLvl = CASCADE_LEVELS[i + 1]}
              {@const cnt = segmentCount(lvl, nextLvl)}
              {@const sev = segmentSeverity(lvl, nextLvl)}
              <div
                class="cascade-arrow"
                class:has-events={cnt > 0}
                class:sev-fail={cnt > 0 && sev === 'fail'}
                class:sev-warn={cnt > 0 && sev === 'warn'}
                title={cnt > 0 ? segmentTooltip(lvl, nextLvl) : ''}
              >
                <span class="cascade-arrow-glyph">→</span>
                {#if cnt > 0}
                  <span class="cascade-arrow-count">{cnt}</span>
                {/if}
              </div>
            {/if}
          {/each}
        {/if}
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
        <SkeletonList rows={4} loading={true} label="loading plan rows" />
        <Empty title="no plan loaded" subtitle="submit a task to generate a decomposition tree" />
      {:else}
        <div class="tree-scroll">
          <Tree
            nodes={treeNodes}
            expandedIds={plan.expandedIds}
            selectedId={plan.selectedId ?? ''}
            cursorId={cursorNodeId}
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
            <!-- B-33: cross-window link → focus this decomposition node in GraphWindow -->
            <button
              class="text-btn graph-link"
              type="button"
              title="Show this node in the Graph window"
              onclick={() => { graph.set_mode('decomposition'); graph.focus_node(selectedNode.id); workspace.openTab('primary', 'graph'); }}
            >Show in Graph</button>
          </div>

          {#if selectedNode.level !== 'baby-step'}
            <!-- Concept / Phase / Step detail -->
            <div class="detail-section">
              <div class="meta-row">
                <span class="meta-label">STATUS</span>
                <span class="meta-value meta-status">
                  <NodeStateGlyph state={toQuinary(selectedNode.status)} size={12} />
                  <span class="meta-status-text">{selectedNode.status.toUpperCase()}</span>
                </span>
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
              <!-- B-7 (SCHEMA_VERSION 100): producer-wire row. Rendered
                   only when the SSB has at least one PackageRefVerified
                   row so the detail pane stays clean on cold start. -->
              {#if packageVerifyStats.total > 0}
                <div
                  class="meta-row"
                  data-testid="packages-verified-row"
                  title="package_ref_verified rows from SSB (session-scoped)"
                >
                  <span class="meta-label">PACKAGES VERIFIED</span>
                  <span
                    class="meta-value"
                    class:all-verified={packageVerifyStats.verified === packageVerifyStats.total}
                  >{packageVerifyStats.verified}/{packageVerifyStats.total}</span>
                </div>
              {/if}
              <!-- B-7 Task #8: PhaseSizingSplitter split badge. Shown only
                   on phase nodes that have a matching phase_sized_split SSB
                   row. The badge displays the sub-task count + complexity
                   reason tooltip. -->
              {#if splitForNode(selectedNode.id)}
                {@const split = splitForNode(selectedNode.id)!}
                <div
                  class="meta-row"
                  data-testid="phase-split-row"
                  title={split.complexity_reason ?? 'phase_sized_split'}
                >
                  <span class="meta-label">SPLIT</span>
                  <span class="meta-value split-badge">
                    <span class="split-count">{split.sub_task_ids.length}</span>
                    <span class="split-unit">sub-tasks</span>
                  </span>
                </div>
              {/if}
              <!-- B-7 Step 9: TreeNavigator cursor state. Shown when the
                   selected node is the current navigator cursor, surfacing
                   the visited/blocked/skipped drift trail. -->
              {#if cursorNodeId !== '' && cursorNodeId === selectedNode.id && treePositions.length > 0}
                {@const pos = treePositions[0]}
                <div
                  class="meta-row"
                  data-testid="tree-cursor-row"
                  title="tree_position_changed: navigator cursor state"
                >
                  <span class="meta-label">CURSOR</span>
                  <span class="meta-value cursor-state">
                    <span class="cursor-chip cursor-visited">{pos.visited.length} visited</span>
                    <span class="cursor-chip cursor-blocked">{pos.blocked.length} blocked</span>
                    <span class="cursor-chip cursor-skipped">{pos.skipped.length} skipped</span>
                  </span>
                </div>
              {/if}
            </div>

            {#if selectedNode.children && selectedNode.children.length > 0}
              <div class="children-list">
                {#each selectedNode.children as child (child.id)}
                  <div class="child-row">
                    <NodeStateGlyph state={toQuinary(child.status)} size={11} />
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

            <!-- C-29: Inline IntentPairEditor (T-2 intent grounding).
                 PLANNING -> TESTING_FIRST gate requires >= MIN_INTENT_PAIRS
                 confirmed pairs per baby step. -->
            <div class="intent-pair-editor" data-testid="intent-pair-editor">
              <div class="ipe-header">
                <span class="ipe-title">INTENT PAIRS (T-2)</span>
                <span class="ipe-count">
                  {intentPairs.pairs.length}/{MIN_INTENT_PAIRS}
                  {#if intentPairs.confirmed}<span class="ipe-confirmed">CONFIRMED</span>{/if}
                </span>
              </div>
              {#if intentPairs.pairs.length > 0}
                <ul class="ipe-list">
                  {#each intentPairs.pairs as pair, idx (idx)}
                    <li class="ipe-row">
                      <div class="ipe-row-intent">{pair.natural_language_intent}</div>
                      <div class="ipe-row-assertion">{pair.concrete_test_assertion}</div>
                    </li>
                  {/each}
                </ul>
              {:else}
                <p class="ipe-empty">no intent pairs yet — add at least {MIN_INTENT_PAIRS} before confirming</p>
              {/if}
              {#if !intentPairs.confirmed}
                <div class="ipe-form">
                  <textarea
                    class="ipe-input"
                    placeholder="natural language intent (max {INTENT_MAX_CHARS} chars)"
                    rows="2"
                    maxlength={INTENT_MAX_CHARS}
                    bind:value={intentPairForm.intent}
                    disabled={intentPairBusy}
                  ></textarea>
                  <textarea
                    class="ipe-input"
                    placeholder="concrete test assertion (max {ASSERTION_MAX_CHARS} chars)"
                    rows="2"
                    maxlength={ASSERTION_MAX_CHARS}
                    bind:value={intentPairForm.assertion}
                    disabled={intentPairBusy}
                  ></textarea>
                  <div class="ipe-actions">
                    <button
                      class="ipe-btn"
                      type="button"
                      onclick={() => void handleAddIntentPair()}
                      disabled={intentPairBusy}
                    >add pair</button>
                    <button
                      class="ipe-btn ipe-btn-confirm"
                      type="button"
                      onclick={() => void handleConfirmIntentPairs()}
                      disabled={intentPairBusy || intentPairs.pairs.length < MIN_INTENT_PAIRS}
                    >confirm ({intentPairs.pairs.length}/{MIN_INTENT_PAIRS})</button>
                  </div>
                  {#if intentPairError}
                    <p class="ipe-error" role="alert">{intentPairError}</p>
                  {/if}
                </div>
              {/if}
            </div>

            <!-- BabyStep detail: 8 block spec sections -->
            {@const blocks = selectedNode.detail?.blocks ?? SAMPLE_BLOCKS.map(b => ({ block: b.block, status: b.status }))}
            <div class="blocks-list">
              {#each SAMPLE_BLOCKS as spec (spec.block)}
                {@const liveBlock = blocks.find(b => b.block === spec.block)}
                {@const status = liveBlock?.status ?? 'pending'}
                <div class="block-section" class:is-done={status === 'done'}>
                  <div class="block-header">
                    <span class="block-name">{spec.block}</span>
                    <span class="block-status">
                      <NodeStateGlyph state={toQuinary(status)} size={11} />
                      <span class="block-status-text">{blockStatusLabel(status)}</span>
                    </span>
                  </div>
                  <p class="block-detail">{spec.detail}</p>
                </div>
              {/each}
            </div>

            <!-- B-7 Step 10: per-baby-step failure retry history. Renders
                 failure_recorded SSB rows matching this baby step's numeric
                 id. Each row shows the attempt count and failure reason.
                 Hidden when no failure rows exist for this step. -->
            {#if selectedBabyStepFailures.length > 0}
              <div class="retry-history" data-testid="retry-history">
                <div class="retry-header">
                  <span class="retry-title">RETRY HISTORY</span>
                  <span class="retry-count">{selectedBabyStepFailures.length}</span>
                </div>
                <ul class="retry-list">
                  {#each selectedBabyStepFailures as rec (rec.attempts + rec.reason)}
                    <li class="retry-row">
                      <span class="retry-attempts" title="attempts">#{rec.attempts}</span>
                      <span class="retry-reason">{rec.reason}</span>
                    </li>
                  {/each}
                </ul>
              </div>
            {/if}
          {/if}

          <!-- B-7 STEP 7: ExecutablePipeline DAG visualizer. Renders the
               full pipeline DAG (nodes + edges) as a lightweight SVG graph,
               plus the repair log and validation error history. Collapsible
               to keep the detail pane clean when not needed. -->
          {#if pipelineDag.nodes.length > 0}
            <div class="dag-section" data-testid="executable-pipeline-dag">
              <button
                class="dag-toggle text-btn"
                type="button"
                onclick={() => dagExpanded = !dagExpanded}
              >
                <span class="dag-toggle-icon">{dagExpanded ? '▾' : '▸'}</span>
                EXECUTABLE PIPELINE DAG ({pipelineDag.nodes.length} nodes)
              </button>

              {#if dagExpanded}
                <div class="dag-svg-wrap">
                  <svg
                    class="dag-svg"
                    width={dagLayout.width + 20}
                    height={dagLayout.height + 10}
                    viewBox={`-10 -5 ${dagLayout.width + 20} ${dagLayout.height + 10}`}
                    role="img"
                    aria-label="Executable pipeline dependency graph"
                  >
                    {#each dagLayout.edges as edge (`${edge.from}-${edge.to}`)}
                      <path
                        class="dag-edge"
                        d={`M ${edge.x1} ${edge.y1} C ${edge.x1} ${(edge.y1 + edge.y2) / 2}, ${edge.x2} ${(edge.y1 + edge.y2) / 2}, ${edge.x2} ${edge.y2}`}
                      />
                    {/each}
                    {#each dagLayout.positioned as item (item.node.id)}
                      {@const isSelected = selectedNode?.id === item.node.id}
                      <g transform={`translate(${item.x}, ${item.y})`}>
                        <rect
                          class="dag-node"
                          class:dag-node-selected={isSelected}
                          width={DAG_NODE_W}
                          height={DAG_NODE_H}
                          rx="3"
                          fill={dagNodeStatusColor(item.node.status)}
                          fill-opacity="0.15"
                          stroke={dagNodeStatusColor(item.node.status)}
                          stroke-width="1"
                        />
                        <text
                          class="dag-node-kind"
                          x="4"
                          y="11"
                        >{dagNodeKindLabel(item.node.kind)}</text>
                        <text
                          class="dag-node-label"
                          x="4"
                          y="23"
                        >{item.node.label.length > 18 ? item.node.label.slice(0, 17) + '…' : item.node.label}</text>
                      </g>
                    {/each}
                  </svg>
                </div>

                {#if pipelineDag.repair_log.length > 0}
                  <div class="dag-repair-log" data-testid="dag-repair-log">
                    <div class="dag-subhead">REPAIR LOG</div>
                    {#each pipelineDag.repair_log as entry, idx (idx)}
                      <div class={repairLogClass(entry)}>
                        <span class="repair-round">R{entry.round}</span>
                        <span class="repair-kind">{entry.action_kind}</span>
                        <span class="repair-reason">{entry.reason}</span>
                        {#if entry.escalated}
                          <span class="repair-badge" title="T_DAG=10 exhausted">ESCALATED</span>
                        {/if}
                      </div>
                    {/each}
                  </div>
                {/if}

                {#if pipelineDag.validation_errors.length > 0}
                  <div class="dag-validation-errors" data-testid="dag-validation-errors">
                    <div class="dag-subhead">VALIDATION ERROR HISTORY</div>
                    {#each pipelineDag.validation_errors as err, idx (idx)}
                      <div class="val-err-entry">
                        <span class="val-err-kind">{err.kind}</span>
                        <span class="val-err-round">R{err.round}</span>
                        <span class="val-err-msg">{err.message}</span>
                      </div>
                    {/each}
                  </div>
                {/if}
              {/if}
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
    background: var(--bg);
  }

  /* ---- Toolbar ---- */
  .toolbar {
    height: 28px;
    min-height: 28px;
    display: flex;
    align-items: center;
    gap: var(--s-3);
    padding: 0 var(--s-3);
    background: var(--bg);
    border-bottom: 1px solid var(--line);
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
    font-family: var(--font-mono);
    font-size: 12px;
    font-weight: 500;
    color: var(--text);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
    max-width: 200px;
  }

  .toolbar-center {
    flex: 1;
    overflow: hidden;
  }

  /* B-7 STEP 11: Cascade hierarchy bar — BabyStep → Step → Phase → Concept */
  .cascade-bar {
    display: flex;
    align-items: center;
    gap: 2px;
    overflow-x: auto;
    scrollbar-width: none;
    padding: 2px 0;
  }

  .cascade-bar::-webkit-scrollbar {
    display: none;
  }

  .cascade-empty {
    font-family: var(--font-mono);
    font-size: 10px;
    color: var(--text-ghost);
    letter-spacing: 0.06em;
  }

  .cascade-node {
    display: inline-flex;
    align-items: center;
    gap: 3px;
    height: 18px;
    padding: 0 5px;
    border: 1px solid var(--line);
    border-radius: var(--r-xs);
    font-family: var(--font-mono);
    font-size: 10px;
    color: var(--text-secondary);
    background: transparent;
    flex-shrink: 0;
    cursor: default;
  }

  .cascade-node.has-miss {
    border-color: var(--steel);
  }

  .cascade-node.sev-warn {
    border-color: var(--warn);
  }

  .cascade-node.sev-fail {
    border-color: var(--fail);
  }

  .cascade-node-label {
    font-weight: 600;
    letter-spacing: 0.06em;
    color: var(--dim);
  }

  .cascade-node-count {
    font-size: 9px;
    color: var(--text-ghost);
    background: rgba(255, 255, 255, 0.06);
    border-radius: var(--r-pill);
    padding: 0 4px;
    min-width: 14px;
    text-align: center;
  }

  .cascade-arrow {
    display: inline-flex;
    align-items: center;
    gap: 2px;
    font-family: var(--font-mono);
    font-size: 10px;
    color: var(--text-ghost);
    flex-shrink: 0;
    cursor: default;
  }

  .cascade-arrow.has-events {
    color: var(--steel);
  }

  .cascade-arrow.sev-warn {
    color: var(--warn);
  }

  .cascade-arrow.sev-fail {
    color: var(--fail);
  }

  .cascade-arrow-glyph {
    font-size: 12px;
    line-height: 1;
  }

  .cascade-arrow-count {
    font-size: 9px;
    color: var(--text-ghost);
    background: rgba(255, 255, 255, 0.06);
    border-radius: var(--r-pill);
    padding: 0 4px;
    min-width: 14px;
    text-align: center;
  }

  .toolbar-right {
    display: flex;
    align-items: center;
    gap: 2px;
    flex-shrink: 0;
  }

  /* Interactive controls = §13/§14 center->out underline idiom (no box). */
  .text-btn {
    position: relative;
    background: transparent;
    border: none;
    color: var(--dim);
    font-family: var(--font-mono);
    font-size: 10px;
    cursor: pointer;
    padding: 2px 4px 3px;
    transition: color var(--dur-fast) var(--ease-out);
  }

  .text-btn::after {
    content: '';
    position: absolute;
    left: 4px;
    right: 4px;
    bottom: 1px;
    height: 1px;
    background: var(--d-teal);
    transform: scaleX(0);
    transform-origin: center;
    transition: transform var(--dur-base) var(--ease-out);
  }

  .text-btn:hover {
    color: var(--text);
  }

  .text-btn:hover::after,
  .text-btn:focus-visible::after {
    transform: scaleX(1);
  }

  @media (prefers-reduced-motion: reduce) {
    .text-btn::after { transition: none; }
  }

  .divider-char {
    color: var(--ghost);
    font-family: var(--font-mono);
    font-size: 10px;
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
    scrollbar-color: var(--raised) transparent;
  }

  .tree-scroll::-webkit-scrollbar {
    width: 4px;
  }

  .tree-scroll::-webkit-scrollbar-thumb {
    background: var(--raised);
    border-radius: 2px;
  }

  /* ---- Column divider ---- */
  .col-divider {
    width: 1px;
    background: var(--line);
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
    font-size: 10px;
    color: var(--dim);
  }

  .detail-scroll {
    flex: 1;
    overflow-y: auto;
    padding: var(--s-3);
    scrollbar-width: thin;
    scrollbar-color: var(--raised) transparent;
    display: flex;
    flex-direction: column;
    gap: var(--s-2);
  }

  .detail-scroll::-webkit-scrollbar {
    width: 4px;
  }

  .detail-scroll::-webkit-scrollbar-thumb {
    background: var(--raised);
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
    color: var(--dim);
  }

  .detail-title {
    font-family: var(--font-mono);
    font-size: 13px;
    font-weight: 500;
    color: var(--text);
    margin: 0;
    line-height: var(--leading-code);
  }

  .detail-elapsed {
    font-family: var(--font-mono);
    font-size: 10px;
    color: var(--dim);
    margin-top: 2px;
  }

  /* ---- Concept/Phase/Step detail section (inset well below --bg) ---- */
  .detail-section {
    background: var(--panel);
    border: 1px solid var(--line);
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
    color: var(--dim);
    min-width: 64px;
    flex-shrink: 0;
  }

  .meta-value {
    font-family: var(--font-mono);
    font-size: 10px;
    color: var(--read);
    min-width: 0;
    overflow-wrap: anywhere;
  }

  .meta-status {
    display: inline-flex;
    align-items: center;
    gap: 6px;
  }

  .meta-status-text {
    color: var(--read);
    letter-spacing: 0.04em;
  }

  /* all-verified reads in the one accent; partial stays neutral (not alarm) */
  .meta-value.all-verified {
    color: var(--d-teal);
  }

  /* B-7 Task #8: split badge — amber accent to distinguish from
     verified (teal) and failed (red). */
  .split-badge {
    display: inline-flex;
    align-items: baseline;
    gap: 2px;
  }
  .split-badge .split-count {
    color: var(--d-amber, #fbbf24);
    font-weight: 600;
  }
  .split-badge .split-unit {
    color: var(--dim);
    font-size: 0.85em;
  }

  /* B-7 Step 9: TreeNavigator cursor state chips. */
  .cursor-state {
    display: inline-flex;
    gap: 6px;
  }
  .cursor-chip {
    font-size: 0.85em;
    color: var(--dim);
  }
  .cursor-chip.cursor-visited { color: var(--d-teal, #2dd4bf); }
  .cursor-chip.cursor-blocked { color: var(--d-red, #f87171); }
  .cursor-chip.cursor-skipped { color: var(--dim); }

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

  .child-label {
    font-family: var(--font-mono);
    font-size: 12px;
    color: var(--read);
    flex: 1;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .child-elapsed {
    font-family: var(--font-mono);
    font-size: 10px;
    color: var(--dim);
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
    border: 1px solid var(--line);
    background: transparent;
    font-family: var(--font-mono);
    font-size: 10px;
    line-height: var(--leading-code);
  }

  .chip-label {
    color: var(--dim);
    letter-spacing: 0.04em;
    text-transform: uppercase;
  }

  .chip-value {
    color: var(--read);
    font-weight: 500;
  }

  /* the focus mode carries the one teal accent; other values stay neutral */
  .chip-mode .chip-value { color: var(--d-teal); }

  /* ---- BabyStep blocks ---- */
  .blocks-list {
    display: flex;
    flex-direction: column;
    gap: var(--s-2);
  }

  .block-section {
    background: var(--panel);
    border: 1px solid var(--line);
    border-radius: var(--r-sm);
    padding: var(--s-2) var(--s-3);
  }

  /* §6 done: content RECEDES — slight fade + a teal-deep left bar. */
  .block-section.is-done {
    opacity: 0.8;
    border-left: 2px solid var(--d-teal-deep);
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
    color: var(--dim);
  }

  .block-status {
    display: inline-flex;
    align-items: center;
    gap: 6px;
    font-family: var(--font-mono);
    font-size: 10px;
    letter-spacing: 0.06em;
  }

  .block-status-text {
    color: var(--read);
  }

  .block-detail {
    font-family: var(--font-mono);
    font-size: 12px;
    color: var(--read);
    line-height: var(--leading-code);
    margin: 0;
    word-break: break-word;
  }

  /* ---- B-7 Step 10: per-baby-step retry history ---- */
  .retry-history {
    border: 1px solid var(--line);
    border-left: 2px solid var(--d-red, #f87171);
    border-radius: var(--r-sm);
    padding: var(--s-3);
    margin-top: var(--s-3);
    background: var(--panel);
  }

  .retry-header {
    display: flex;
    align-items: center;
    gap: var(--s-2);
    margin-bottom: var(--s-2);
  }

  .retry-title {
    font-family: var(--font-mono);
    font-size: 10px;
    font-weight: 600;
    letter-spacing: 0.08em;
    color: var(--dim);
  }

  .retry-count {
    font-family: var(--font-mono);
    font-size: 10px;
    color: var(--d-red, #f87171);
    background: color-mix(in srgb, var(--d-red, #f87171) 12%, transparent);
    padding: 1px 6px;
    border-radius: var(--r-xs, 2px);
  }

  .retry-list {
    list-style: none;
    margin: 0;
    padding: 0;
    display: flex;
    flex-direction: column;
    gap: 4px;
  }

  .retry-row {
    display: flex;
    align-items: baseline;
    gap: var(--s-2);
  }

  .retry-attempts {
    font-family: var(--font-mono);
    font-size: 10px;
    font-weight: 600;
    color: var(--d-red, #f87171);
    flex-shrink: 0;
    min-width: 28px;
  }

  .retry-reason {
    font-family: var(--font-mono);
    font-size: 11px;
    color: var(--read);
    line-height: var(--leading-code);
    word-break: break-word;
  }

  /* ---- C-29 IntentPairEditor (inset well below --bg, hairline edge) ---- */
  .intent-pair-editor {
    border: 1px solid var(--line);
    border-radius: var(--r-sm);
    padding: var(--s-3);
    margin-bottom: var(--s-3);
    background: var(--panel);
  }
  .ipe-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: var(--s-2);
  }
  .ipe-title {
    font-family: var(--font-mono);
    font-size: 10px;
    font-weight: 600;
    letter-spacing: 0.08em;
    color: var(--read);
  }
  .ipe-count {
    font-family: var(--font-mono);
    font-size: 10px;
    color: var(--dim);
  }
  .ipe-confirmed {
    color: var(--d-teal);
    margin-left: 4px;
  }
  .ipe-list {
    list-style: none;
    margin: 0 0 var(--s-2) 0;
    padding: 0;
    display: flex;
    flex-direction: column;
    gap: var(--s-1);
  }
  .ipe-row {
    border-left: 2px solid var(--line);
    padding-left: 6px;
  }
  .ipe-row-intent {
    font-size: 10px;
    color: var(--text);
    line-height: var(--leading-code);
  }
  .ipe-row-assertion {
    font-size: 10px;
    color: var(--dim);
    font-family: var(--font-mono);
    line-height: var(--leading-code);
  }
  .ipe-empty {
    font-size: 10px;
    color: var(--dim);
    margin: 0 0 var(--s-2) 0;
  }
  .ipe-form {
    display: flex;
    flex-direction: column;
    gap: 6px;
  }
  .ipe-input {
    width: 100%;
    background: var(--sink);
    border: 1px solid var(--line);
    border-radius: var(--r-xs);
    padding: 6px;
    font-family: var(--font-mono);
    font-size: 10px;
    color: var(--text);
    resize: vertical;
  }
  .ipe-input:focus-visible {
    border-color: var(--d-teal-soft);
  }
  .ipe-actions {
    display: flex;
    gap: var(--s-2);
  }
  .ipe-btn {
    padding: 4px 8px;
    background: var(--sink);
    border: 1px solid var(--line);
    border-radius: var(--r-xs);
    color: var(--text);
    font-family: var(--font-mono);
    font-size: 10px;
    cursor: pointer;
    transition: border-color var(--dur-fast) var(--ease-out),
      background var(--dur-fast) var(--ease-out);
  }
  .ipe-btn:hover:not(:disabled) {
    background: var(--raised);
    border-color: var(--line2);
  }
  .ipe-btn:disabled {
    color: var(--ghost);
    cursor: not-allowed;
  }
  .ipe-btn-confirm:not(:disabled) {
    border-color: var(--d-teal);
    color: var(--d-teal);
  }
  /* §6 error language: mono-inversion, never a foreign red. */
  .ipe-error {
    font-size: 10px;
    color: var(--void);
    background: var(--text);
    border-radius: 2px;
    padding: 3px 6px;
    margin: 0;
    align-self: flex-start;
    font-weight: 600;
  }

  /* ---- B-7 STEP 7: ExecutablePipeline DAG visualizer ---- */
  .dag-section {
    border: 1px solid var(--line);
    border-radius: var(--r-sm);
    padding: var(--s-2) var(--s-3);
    margin-top: var(--s-3);
    background: var(--panel);
  }

  .dag-toggle {
    display: flex;
    align-items: center;
    gap: 4px;
    width: 100%;
    text-align: left;
    font-family: var(--font-mono);
    font-size: 10px;
    font-weight: 600;
    letter-spacing: 0.06em;
    color: var(--read);
    padding: 2px 0;
  }

  .dag-toggle-icon {
    font-size: 10px;
    color: var(--dim);
  }

  .dag-svg-wrap {
    overflow-x: auto;
    overflow-y: hidden;
    margin-top: var(--s-2);
    padding: 4px 0;
    scrollbar-width: thin;
    scrollbar-color: var(--raised) transparent;
  }

  .dag-svg-wrap::-webkit-scrollbar {
    height: 4px;
  }

  .dag-svg-wrap::-webkit-scrollbar-thumb {
    background: var(--raised);
    border-radius: 2px;
  }

  .dag-svg {
    display: block;
  }

  .dag-edge {
    fill: none;
    stroke: var(--line2);
    stroke-width: 1;
  }

  .dag-node {
    cursor: default;
    transition: stroke-width var(--dur-fast) var(--ease-out);
  }

  .dag-node-selected {
    stroke-width: 2.5;
  }

  .dag-node-kind {
    font-family: var(--font-mono);
    font-size: 8px;
    font-weight: 600;
    letter-spacing: 0.04em;
    fill: var(--dim);
  }

  .dag-node-label {
    font-family: var(--font-mono);
    font-size: 9px;
    fill: var(--text);
  }

  .dag-subhead {
    font-family: var(--font-mono);
    font-size: 9px;
    font-weight: 600;
    letter-spacing: 0.08em;
    color: var(--dim);
    margin-bottom: 4px;
    margin-top: var(--s-2);
  }

  .dag-repair-log,
  .dag-validation-errors {
    margin-top: var(--s-2);
  }

  .repair-entry {
    display: flex;
    align-items: center;
    gap: 6px;
    padding: 2px 0;
    font-family: var(--font-mono);
    font-size: 10px;
    color: var(--read);
  }

  .repair-entry.repair-escalated {
    color: var(--void);
    background: var(--text);
    border-radius: 2px;
    padding: 3px 6px;
  }

  .repair-round {
    color: var(--dim);
    font-weight: 600;
    flex-shrink: 0;
  }

  .repair-kind {
    color: var(--d-teal);
    flex-shrink: 0;
  }

  .repair-reason {
    color: var(--read);
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .repair-badge {
    flex-shrink: 0;
    font-size: 8px;
    font-weight: 700;
    letter-spacing: 0.06em;
    color: var(--d-red, #f87171);
    border: 1px solid var(--d-red, #f87171);
    border-radius: 2px;
    padding: 0 3px;
  }

  .val-err-entry {
    display: flex;
    align-items: center;
    gap: 6px;
    padding: 2px 0;
    font-family: var(--font-mono);
    font-size: 10px;
    color: var(--read);
  }

  .val-err-kind {
    color: var(--d-teal);
    flex-shrink: 0;
  }

  .val-err-round {
    color: var(--dim);
    font-weight: 600;
    flex-shrink: 0;
  }

  .val-err-msg {
    color: var(--read);
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }
</style>
