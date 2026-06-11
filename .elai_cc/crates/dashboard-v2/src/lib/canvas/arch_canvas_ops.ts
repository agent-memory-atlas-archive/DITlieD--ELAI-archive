// H-13: arch_canvas_ops.ts — pure op-application logic for the arch canvas.
//
// Extracted from ArchMode.svelte so it can be unit-tested without a Svelte
// runtime or DOM (node:test, no JSDOM required).
//
// The graphology DirectedGraph interface used here is a structural duck-type:
// any object that implements hasNode/hasEdge/addNode/addEdgeWithKey/dropNode/
// dropEdge/setNodeAttribute/getNodeAttributes is accepted. This lets tests pass
// a plain JS object mock without loading the full graphology bundle.
//
// Two-lane constraint note (matching canvas_grammar.rs module doc):
//   Cloud lane: tool input_schema is the structural constraint (active).
//   Local lane: GBNF grammar (canvas_grammar.rs) is Tier-2/sidecar.
//   This module is the drain transport — it applies already-validated ops to
//   the graphology graph regardless of which constraint lane produced them.

import type { CanvasOpDto } from '../api/stub';

// ---------------------------------------------------------------------------
// Minimal structural interface for a graphology DirectedGraph
// (allows injecting a mock in unit tests without importing graphology)
// ---------------------------------------------------------------------------

export interface GraphLike {
  hasNode(id: string): boolean;
  hasEdge(id: string): boolean;
  addNode(id: string, attrs: Record<string, unknown>): void;
  addEdgeWithKey(key: string, source: string, target: string, attrs: Record<string, unknown>): void;
  dropNode(id: string): void;
  dropEdge(id: string): void;
  setNodeAttribute(id: string, attr: string, value: unknown): void;
  getNodeAttributes(id: string): Record<string, unknown>;
}

// ---------------------------------------------------------------------------
// Result type
// ---------------------------------------------------------------------------

export type ApplyOpResult =
  | { ok: true }
  | { ok: false; reason: string };

// ---------------------------------------------------------------------------
// applyOpToGraph
//
// Apply a single validated CanvasOpDto to the graph.
//
// Dangling-id rejection: ops that reference a node_id or edge id NOT in the
// graph are rejected with a non-silent reason (wiring-enforcement R1 / WARN-1).
// A non-empty but non-existent id is a hallucinated reference that the Rust
// sanitize layer (canvas_dispatcher.rs::sanitize_node_id) cannot catch — it
// only checks empty/whitespace. This function is the correct detection site
// because it has access to the live graph state.
//
// Returns { ok: true } on success, { ok: false, reason } on rejection.
// Callers MUST log/surface rejection reasons — never silently drop.
// ---------------------------------------------------------------------------

export function applyOpToGraph(
  graph: GraphLike,
  op: CanvasOpDto,
): ApplyOpResult {
  switch (op.kind) {
    case 'node_created': {
      if (!op.node_id.trim()) {
        return { ok: false, reason: 'node_created requires non-empty node_id' };
      }
      if (!graph.hasNode(op.node_id)) {
        const label = String(op.op_data['label'] ?? op.node_id);
        const x = typeof op.op_data['x'] === 'number' ? op.op_data['x'] : Math.random();
        const y = typeof op.op_data['y'] === 'number' ? op.op_data['y'] : Math.random();
        graph.addNode(op.node_id, { label, x, y, size: 8, color: '#6e7480' });
      }
      return { ok: true };
    }

    case 'edge_connected': {
      const fromId = String(op.op_data['from_id'] ?? '');
      const toId = String(op.op_data['to_id'] ?? '');
      if (!graph.hasNode(fromId)) {
        return {
          ok: false,
          reason: `edge_connected: from_id '${fromId}' not in graph (dangling-id)`,
        };
      }
      if (!graph.hasNode(toId)) {
        return {
          ok: false,
          reason: `edge_connected: to_id '${toId}' not in graph (dangling-id)`,
        };
      }
      const edgeId = `arch_${fromId}__${toId}`;
      if (!graph.hasEdge(edgeId)) {
        graph.addEdgeWithKey(edgeId, fromId, toId, { size: 1, color: '#374151' });
      }
      return { ok: true };
    }

    case 'node_deleted': {
      if (!graph.hasNode(op.node_id)) {
        return {
          ok: false,
          reason: `node_deleted: node_id '${op.node_id}' not in graph (dangling-id)`,
        };
      }
      graph.dropNode(op.node_id);
      return { ok: true };
    }

    case 'edge_deleted': {
      const edgeId = String(op.op_data['edge_id'] ?? '');
      if (!graph.hasEdge(edgeId)) {
        return {
          ok: false,
          reason: `edge_deleted: edge_id '${edgeId}' not in graph (dangling-id)`,
        };
      }
      graph.dropEdge(edgeId);
      return { ok: true };
    }

    case 'annotation_added': {
      if (!graph.hasNode(op.node_id)) {
        return {
          ok: false,
          reason: `annotation_added: node_id '${op.node_id}' not in graph (dangling-id)`,
        };
      }
      const annotation = String(op.op_data['annotation'] ?? '');
      graph.setNodeAttribute(op.node_id, 'annotation', annotation);
      return { ok: true };
    }

    case 'label_edited': {
      if (!graph.hasNode(op.node_id)) {
        return {
          ok: false,
          reason: `label_edited: node_id '${op.node_id}' not in graph (dangling-id)`,
        };
      }
      const newLabel = String(op.op_data['label'] ?? op.node_id);
      graph.setNodeAttribute(op.node_id, 'label', newLabel);
      return { ok: true };
    }

    case 'group_created': {
      const groupId = String(op.op_data['group_id'] ?? op.node_id);
      if (!groupId.trim()) {
        return {
          ok: false,
          reason: 'group_created requires non-empty group_id or node_id',
        };
      }
      if (!graph.hasNode(groupId)) {
        const label = String(op.op_data['label'] ?? groupId);
        graph.addNode(groupId, {
          label,
          x: Math.random(),
          y: Math.random(),
          size: 12,
          color: '#14b8a6',
          isGroup: true,
        });
      }
      return { ok: true };
    }

    case 'node_forked': {
      if (!graph.hasNode(op.node_id)) {
        return {
          ok: false,
          reason: `node_forked: source node_id '${op.node_id}' not in graph (dangling-id)`,
        };
      }
      const forkId = String(op.op_data['fork_id'] ?? '');
      if (!forkId.trim()) {
        return { ok: false, reason: 'node_forked requires non-empty fork_id in op_data' };
      }
      const srcAttrs = graph.getNodeAttributes(op.node_id);
      if (!graph.hasNode(forkId)) {
        graph.addNode(forkId, {
          ...srcAttrs,
          x: (srcAttrs['x'] as number ?? Math.random()) + 0.05,
          y: (srcAttrs['y'] as number ?? Math.random()) + 0.05,
          label: String(op.op_data['label'] ?? `${srcAttrs['label']}-fork`),
        });
      }
      return { ok: true };
    }

    case 'layout_changed': {
      const positions = op.op_data['positions'];
      if (positions && typeof positions === 'object' && !Array.isArray(positions)) {
        for (const [nodeId, pos] of Object.entries(positions as Record<string, unknown>)) {
          if (!graph.hasNode(nodeId)) {
            // Non-silent rejection for each dangling id in a layout op.
            // We continue to process other positions but surface the first rejection.
            return {
              ok: false,
              reason: `layout_changed: node_id '${nodeId}' not in graph (dangling-id)`,
            };
          }
          if (pos && typeof pos === 'object') {
            const p = pos as Record<string, unknown>;
            if (typeof p['x'] === 'number') graph.setNodeAttribute(nodeId, 'x', p['x']);
            if (typeof p['y'] === 'number') graph.setNodeAttribute(nodeId, 'y', p['y']);
          }
        }
      }
      return { ok: true };
    }

    default:
      // Unknown or non-mutating op kind (e.g. screenshot_captured): no graph mutation.
      return { ok: true };
  }
}
