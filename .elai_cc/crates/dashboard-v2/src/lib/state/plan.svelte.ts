// ============================================================
// plan.svelte.ts — Decomposition tree state
//
// Phase B wiring: drop the mock DECOMP_TREE seed. Initial tree is
// an empty placeholder; bootstrap() polls get_decomposition_tree()
// every 5s and replaces local state. Selection/expansion are
// preserved across refreshes when the node id still exists.
// ============================================================

import type { DecompNode } from '../api/types';
import { get_decomposition_tree, logStub } from '../api/stub';

const POLL_INTERVAL_MS = 5_000;

const EMPTY_TREE: DecompNode = {
  id: 'empty',
  level: 'concept',
  label: 'No active task',
  status: 'pending',
  elapsed_ms: 0,
  children: [],
};

function collectIds(node: DecompNode | null | undefined, acc: Set<string>) {
  if (!node) return;
  acc.add(node.id);
  for (const child of node.children ?? []) collectIds(child, acc);
}

function createPlan() {
  let tree = $state<DecompNode>({ ...EMPTY_TREE });
  let selectedId = $state<string | null>(null);
  let expandedIds = $state<Set<string>>(new Set());

  let pollTimer: ReturnType<typeof setInterval> | null = null;
  let alive = false;

  async function refresh() {
    try {
      const next = await get_decomposition_tree();
      tree = next;
      // Drop selection / expanded ids that no longer exist in the new tree.
      const ids = new Set<string>();
      collectIds(next, ids);
      if (selectedId && !ids.has(selectedId)) selectedId = null;
      const filtered = new Set<string>();
      for (const id of expandedIds) if (ids.has(id)) filtered.add(id);
      expandedIds = filtered;
    } catch (err) {
      console.debug('[plan] refresh failed', err);
    }
  }

  return {
    get tree() { return tree; },
    get selectedId() { return selectedId; },
    get expandedIds() { return expandedIds; },

    bootstrap() {
      if (alive) return;
      alive = true;
      void refresh();
      pollTimer = setInterval(() => { void refresh(); }, POLL_INTERVAL_MS);
      logStub('plan.bootstrap');
    },

    teardown() {
      if (pollTimer !== null) {
        clearInterval(pollTimer);
        pollTimer = null;
      }
      alive = false;
    },

    setTree(newTree: DecompNode) { tree = newTree; },
    selectNode(id: string) { selectedId = id; },
    expandNode(id: string) { expandedIds = new Set([...expandedIds, id]); },
    collapseNode(id: string) {
      const next = new Set(expandedIds);
      next.delete(id);
      expandedIds = next;
    },
    toggleNode(id: string) {
      if (expandedIds.has(id)) {
        const next = new Set(expandedIds);
        next.delete(id);
        expandedIds = next;
      } else {
        expandedIds = new Set([...expandedIds, id]);
      }
    },
  };
}

export const plan = createPlan();
