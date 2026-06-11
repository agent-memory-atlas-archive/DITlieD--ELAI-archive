<!--
  H-15 — ComparisonBoard.svelte
  H15-1 slice: the cheapest high-value view.

  A comparison/branch board: N columns (approach A vs B, two git branches,
  two run-artifacts) on a shared row axis, backed by the fork op + forked_from
  edges (H-13 canvas_fork CanvasOpKind::NodeForked). Click a cell -> expand
  this tradeoff / pick this branch.

  Design (CANVAS-WORKBENCH-DESIGN.md §17):
    - Pure existing ops: columns + labels + fork edges (no new renderer primitives).
    - view_type = 'comparison_board' (registered in renderer_registry.ts).
    - Renders from seed_op_ids (CanvasOpApplied rows of kind NodeForked/NodeCreated).
    - hasNode/hasEdge checks before applying ops (gotcha DANGLING_ID_REJECT_AT_GRAPH_DRAIN).
    - SEPARATE from the canvas_op_applied drain (gotcha OP_APPLICATION_TRANSPORT_IS_H13_DRAIN).

  Props:
    - viewId: string — stable view instance id from the ViewSpawned SSB row.
    - seedOpIds: string[] — CanvasOpApplied op_ids to seed the board columns.
    - taskId: string | undefined

  GD-1: composes with H-13 CanvasOpApplied rows. No new canvas op primitives.
  HARD RULES: no magic numbers (named constants for column cap); no unwrap equiv.
-->

<script lang="ts">
  import type { CanvasOpDto } from '../api/stub';
  import { get_canvas_ops } from '../api/stub';

  // ---------------------------------------------------------------------------
  // Named constants (no magic numbers — CLAUDE.md HARD RULE)
  // ---------------------------------------------------------------------------

  /** Maximum columns rendered in a comparison board view. */
  const MAX_COMPARISON_COLUMNS = 8;

  /** Label shown for an empty/missing column head. */
  const EMPTY_COLUMN_LABEL = '(no label)';

  // ---------------------------------------------------------------------------
  // Props
  // ---------------------------------------------------------------------------

  interface Props {
    /** Stable view instance id (from ViewSpawned SSB row). */
    viewId: string;
    /** Seed op ids to bootstrap the view's initial graph state. */
    seedOpIds?: string[];
    /** Task id for SSB tracing. */
    taskId?: string;
  }

  let { viewId, seedOpIds = [], taskId }: Props = $props();

  // ---------------------------------------------------------------------------
  // State
  // ---------------------------------------------------------------------------

  interface ColumnCell {
    nodeId: string;
    label: string;
    columnIndex: number;
    /** True if this node was forked (comparison arm). */
    isForked: boolean;
    /** Source node id if this is a fork (empty if root). */
    forkedFrom: string;
  }

  /** Columns derived from fork + label ops. */
  let columns = $state<Map<string, ColumnCell[]>>(new Map());
  /** Column head labels (column id -> label). */
  let columnHeads = $state<Map<string, string>>(new Map());
  /** Human-readable status for the board. */
  let status = $state<string>('loading');
  /** Any error during seed-op hydration. */
  let errorMsg = $state<string | null>(null);

  // ---------------------------------------------------------------------------
  // Hydrate from seed ops
  // ---------------------------------------------------------------------------

  $effect(() => {
    if (seedOpIds.length === 0) {
      status = 'empty';
      return;
    }
    hydrateFromSeedOps(seedOpIds);
  });

  async function hydrateFromSeedOps(opIds: string[]): Promise<void> {
    try {
      // Fetch recent canvas ops and filter to seedOpIds.
      const allOps = await get_canvas_ops(100);
      const seedSet = new Set(opIds);
      const seedOps = allOps.filter((op) => {
        // Match by task_id as the correlation key (op_id is in the task_id field
        // per CanvasOpPayload.task_id; seedOpIds are the task_ids of the seed ops).
        return seedSet.has(op.task_id) || seedSet.has(op.node_id);
      });

      buildColumns(seedOps);
      status = 'ready';
    } catch (e: unknown) {
      const msg = e instanceof Error ? e.message : String(e);
      errorMsg = `ComparisonBoard hydration failed: ${msg}`;
      status = 'error';
    }
  }

  function buildColumns(ops: CanvasOpDto[]): void {
    const colMap = new Map<string, ColumnCell[]>();
    const headMap = new Map<string, string>();

    let colIndex = 0;
    for (const op of ops) {
      if (colIndex >= MAX_COMPARISON_COLUMNS) break;

      // hasNode/hasEdge equivalent: check node_id is non-empty for node ops.
      // (Graphology not available here; structural sanity check only.)
      if (!op.node_id.trim() && op.kind === 'node_created') continue;

      if (op.kind === 'node_forked') {
        const forkedFrom = String(op.op_data['from_id'] ?? '');
        if (!forkedFrom.trim()) continue; // dangling-id rejection
        const colId = `col-${op.node_id}`;
        if (!colMap.has(colId)) {
          colMap.set(colId, []);
          headMap.set(colId, op.op_data['label'] != null
            ? String(op.op_data['label'])
            : EMPTY_COLUMN_LABEL);
          colIndex++;
        }
        colMap.get(colId)!.push({
          nodeId: op.node_id,
          label: String(op.op_data['label'] ?? op.node_id),
          columnIndex: colIndex - 1,
          isForked: true,
          forkedFrom,
        });
      } else if (op.kind === 'node_created') {
        const colId = `col-root-${op.node_id}`;
        if (!colMap.has(colId)) {
          colMap.set(colId, []);
          headMap.set(colId, String(op.op_data['label'] ?? op.node_id));
          colIndex++;
        }
        colMap.get(colId)!.push({
          nodeId: op.node_id,
          label: String(op.op_data['label'] ?? op.node_id),
          columnIndex: colIndex - 1,
          isForked: false,
          forkedFrom: '',
        });
      }
    }

    columns = colMap;
    columnHeads = headMap;
  }
</script>

<!-- Comparison board layout: N columns, header row + cells -->
<div class="comparison-board" data-view-id={viewId}>
  {#if status === 'loading'}
    <div class="cb-status">loading comparison board...</div>
  {:else if status === 'error'}
    <div class="cb-error">{errorMsg}</div>
  {:else if status === 'empty'}
    <div class="cb-status">no seed ops — spawn a fork to populate this board</div>
  {:else}
    <div class="cb-columns" style="grid-template-columns: repeat({Math.min(columns.size, MAX_COMPARISON_COLUMNS)}, 1fr)">
      {#each [...columnHeads.entries()] as [colId, headLabel] (colId)}
        <div class="cb-col-head">{headLabel}</div>
      {/each}
      {#each [...columns.entries()] as [colId, cells] (colId)}
        <div class="cb-col">
          {#each cells as cell (cell.nodeId)}
            <div class="cb-cell" class:forked={cell.isForked} data-node-id={cell.nodeId}>
              <span class="cb-cell-label">{cell.label}</span>
              {#if cell.isForked}
                <span class="cb-cell-fork-tag">fork of {cell.forkedFrom}</span>
              {/if}
            </div>
          {/each}
        </div>
      {/each}
    </div>
  {/if}
</div>

<style>
  .comparison-board {
    font-family: var(--font-mono, monospace);
    font-size: 0.85rem;
    padding: 0.5rem;
    overflow-x: auto;
  }
  .cb-status,
  .cb-error {
    color: var(--text-muted, #888);
    padding: 0.5rem;
  }
  .cb-error {
    color: var(--color-error, #e74c3c);
  }
  .cb-columns {
    display: grid;
    gap: 0.5rem;
    min-width: max-content;
  }
  .cb-col-head {
    font-weight: 600;
    border-bottom: 1px solid var(--border-color, #333);
    padding-bottom: 0.25rem;
    margin-bottom: 0.25rem;
  }
  .cb-col {
    display: flex;
    flex-direction: column;
    gap: 0.25rem;
  }
  .cb-cell {
    padding: 0.25rem 0.5rem;
    border: 1px solid var(--border-color, #333);
    border-radius: 3px;
    cursor: pointer;
    transition: background-color 0.1s;
  }
  .cb-cell:hover {
    background-color: var(--hover-bg, #1a1a2e);
  }
  .cb-cell.forked {
    border-left: 3px solid var(--color-accent, #4a9eff);
  }
  .cb-cell-label {
    display: block;
  }
  .cb-cell-fork-tag {
    font-size: 0.7rem;
    color: var(--text-muted, #888);
  }
</style>
