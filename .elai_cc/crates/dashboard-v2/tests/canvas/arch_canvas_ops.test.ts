// Tests for src/lib/canvas/arch_canvas_ops.ts
//
// Exercises the drain op-application logic and dangling-id rejection
// without requiring a Svelte runtime, JSDOM, or graphology bundle.
// A plain JS object mock satisfies the GraphLike structural interface.
//
// Run via:
//   node --experimental-strip-types --test tests/canvas/arch_canvas_ops.test.ts

import { test } from 'node:test';
import * as assert from 'node:assert/strict';
import { applyOpToGraph } from '../../src/lib/canvas/arch_canvas_ops.ts';
import type { GraphLike } from '../../src/lib/canvas/arch_canvas_ops.ts';
import type { CanvasOpDto } from '../../src/lib/api/stub.ts';

// ---------------------------------------------------------------------------
// Mock GraphLike factory (no graphology dependency in tests)
// ---------------------------------------------------------------------------

function makeGraph(): GraphLike & {
  nodes: Map<string, Record<string, unknown>>;
  edges: Map<string, { source: string; target: string; attrs: Record<string, unknown> }>;
} {
  const nodes = new Map<string, Record<string, unknown>>();
  const edges = new Map<string, { source: string; target: string; attrs: Record<string, unknown> }>();
  return {
    nodes,
    edges,
    hasNode: (id) => nodes.has(id),
    hasEdge: (id) => edges.has(id),
    addNode: (id, attrs) => { nodes.set(id, { ...attrs }); },
    addEdgeWithKey: (key, src, tgt, attrs) => { edges.set(key, { source: src, target: tgt, attrs }); },
    dropNode: (id) => { nodes.delete(id); },
    dropEdge: (id) => { edges.delete(id); },
    setNodeAttribute: (id, attr, value) => {
      const a = nodes.get(id);
      if (a) a[attr] = value;
    },
    getNodeAttributes: (id) => nodes.get(id) ?? {},
  };
}

function makeOp(kind: string, node_id: string, op_data: Record<string, unknown> = {}): CanvasOpDto {
  return { kind, node_id, op_data, mode: 'arch', task_id: 't-test', session_id: 's-test', ts: Date.now() };
}

// ---------------------------------------------------------------------------
// node_created: adds node to graph
// ---------------------------------------------------------------------------

test('node_created adds node with label', () => {
  const g = makeGraph();
  const result = applyOpToGraph(g, makeOp('node_created', 'svc-api', { label: 'API Service', x: 0.1, y: 0.2 }));
  assert.equal(result.ok, true, 'node_created should succeed');
  assert.ok(g.nodes.has('svc-api'), 'node svc-api must be in graph');
  assert.equal(g.nodes.get('svc-api')?.['label'], 'API Service');
});

test('node_created with empty node_id is rejected (dangling-id)', () => {
  const g = makeGraph();
  const result = applyOpToGraph(g, makeOp('node_created', '', { label: 'Ghost' }));
  assert.equal(result.ok, false, 'empty node_id must be rejected');
  assert.ok(!result.ok && result.reason.length > 0, 'rejection must carry a reason');
  assert.equal(g.nodes.size, 0, 'no node must be added');
});

// ---------------------------------------------------------------------------
// edge_connected: adds edge, rejects dangling node references
// ---------------------------------------------------------------------------

test('edge_connected adds edge between existing nodes', () => {
  const g = makeGraph();
  applyOpToGraph(g, makeOp('node_created', 'a', { label: 'A' }));
  applyOpToGraph(g, makeOp('node_created', 'b', { label: 'B' }));
  const result = applyOpToGraph(g, makeOp('edge_connected', '', { from_id: 'a', to_id: 'b' }));
  assert.equal(result.ok, true);
  assert.ok(g.edges.has('arch_a__b'), 'edge arch_a__b must exist');
});

test('edge_connected rejects dangling from_id', () => {
  const g = makeGraph();
  applyOpToGraph(g, makeOp('node_created', 'b', { label: 'B' }));
  const result = applyOpToGraph(g, makeOp('edge_connected', '', { from_id: 'hallucinated-node-xyz', to_id: 'b' }));
  assert.equal(result.ok, false, 'dangling from_id must be rejected');
  assert.ok(!result.ok && result.reason.includes('dangling-id'), 'reason must cite dangling-id');
  assert.equal(g.edges.size, 0, 'no edge must be added');
});

test('edge_connected rejects dangling to_id', () => {
  const g = makeGraph();
  applyOpToGraph(g, makeOp('node_created', 'a', { label: 'A' }));
  const result = applyOpToGraph(g, makeOp('edge_connected', '', { from_id: 'a', to_id: 'ghost-node' }));
  assert.equal(result.ok, false, 'dangling to_id must be rejected');
  assert.ok(!result.ok && result.reason.includes('dangling-id'), 'reason must cite dangling-id');
});

// ---------------------------------------------------------------------------
// node_deleted: removes node, rejects missing node
// ---------------------------------------------------------------------------

test('node_deleted removes existing node', () => {
  const g = makeGraph();
  applyOpToGraph(g, makeOp('node_created', 'x', {}));
  const result = applyOpToGraph(g, makeOp('node_deleted', 'x'));
  assert.equal(result.ok, true);
  assert.ok(!g.nodes.has('x'), 'node x must be removed');
});

test('node_deleted with non-existent node_id is rejected (dangling-id)', () => {
  const g = makeGraph();
  const result = applyOpToGraph(g, makeOp('node_deleted', 'does-not-exist'));
  assert.equal(result.ok, false);
  assert.ok(!result.ok && result.reason.includes('dangling-id'));
});

// ---------------------------------------------------------------------------
// label_edited: updates label, rejects dangling
// ---------------------------------------------------------------------------

test('label_edited updates node label', () => {
  const g = makeGraph();
  applyOpToGraph(g, makeOp('node_created', 'n1', { label: 'Old Label' }));
  const result = applyOpToGraph(g, makeOp('label_edited', 'n1', { label: 'New Label' }));
  assert.equal(result.ok, true);
  assert.equal(g.nodes.get('n1')?.['label'], 'New Label');
});

test('label_edited with non-existent node_id is rejected (dangling-id)', () => {
  const g = makeGraph();
  const result = applyOpToGraph(g, makeOp('label_edited', 'ghost', { label: 'X' }));
  assert.equal(result.ok, false);
  assert.ok(!result.ok && result.reason.includes('dangling-id'));
});

// ---------------------------------------------------------------------------
// Drain-sequence integration: multiple ops applied in order
// ---------------------------------------------------------------------------

test('drain sequence: create two nodes then connect them', () => {
  const g = makeGraph();
  const ops: CanvasOpDto[] = [
    makeOp('node_created', 'svc-auth', { label: 'Auth Service' }),
    makeOp('node_created', 'svc-db',   { label: 'Database' }),
    makeOp('edge_connected', '', { from_id: 'svc-auth', to_id: 'svc-db' }),
  ];
  for (const op of ops) {
    const r = applyOpToGraph(g, op);
    assert.equal(r.ok, true, `op ${op.kind} must succeed`);
  }
  assert.ok(g.nodes.has('svc-auth'));
  assert.ok(g.nodes.has('svc-db'));
  assert.ok(g.edges.has('arch_svc-auth__svc-db'));
});

test('drain sequence stops on dangling-id: edge before nodes is rejected', () => {
  const g = makeGraph();
  // Try to connect nodes that do not yet exist.
  const result = applyOpToGraph(g, makeOp('edge_connected', '', { from_id: 'not-yet-a', to_id: 'not-yet-b' }));
  assert.equal(result.ok, false, 'connecting non-existent nodes must fail');
  assert.equal(g.edges.size, 0);
  assert.equal(g.nodes.size, 0);
});
