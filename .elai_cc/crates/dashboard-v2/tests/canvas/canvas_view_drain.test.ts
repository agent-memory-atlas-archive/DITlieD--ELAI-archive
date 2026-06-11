// H-15 — canvas_view_drain.test.ts
//
// Tests the renderer_registry.ts resolveRenderer / registerRenderer primitives
// and the CanvasWindow drain logic (simulated without Svelte runtime).
//
// This is the frontend analog of the Rust h15_spawn_view_contract.rs contract test.
// It proves:
//   (1) resolveRenderer maps view_type='comparison_board' to a non-null component
//       after registerRenderer is called (the wiring drain test).
//   (2) resolveRenderer returns null for unknown view_types (fallback path).
//   (3) The drain simulation: given a ViewSpawned event with a registered view_type,
//       the resolved component is non-null and seed_op_ids are passed through.
//   (4) A ViewSpawned event with an unregistered view_type yields null component
//       (fail-visible fallback — console.warn path in CanvasWindow).
//
// Run via:
//   node --experimental-strip-types --test tests/canvas/canvas_view_drain.test.ts

import { test } from 'node:test';
import * as assert from 'node:assert/strict';
import {
  registerRenderer,
  resolveRenderer,
  registeredViewTypes,
} from '../../src/lib/canvas/renderer_registry.ts';
import type { CanvasViewRenderer } from '../../src/lib/canvas/renderer_registry.ts';

// ---------------------------------------------------------------------------
// Mock component factory (no Svelte runtime needed for registry tests)
//
// CanvasViewRenderer is a Svelte Component type. For unit testing the registry
// logic, any object satisfying the structural shape is sufficient.
// We use a plain function stub — the registry stores and returns it by reference.
// ---------------------------------------------------------------------------

function mockRenderer(name: string): CanvasViewRenderer {
  // The registry type-checks CanvasViewRenderer as Component<Props>.
  // Cast via unknown to satisfy TypeScript in the test context.
  return { name } as unknown as CanvasViewRenderer;
}

// ---------------------------------------------------------------------------
// Helper: simulate the CanvasWindow drain logic for a list of ViewSpawned events
// Returns the list of resolved views (component + seed_op_ids pairs).
// Mirrors the $effect in CanvasWindow.svelte:118-146 in pure TS.
// ---------------------------------------------------------------------------

interface MockCanvasViewEvent {
  kind: string;
  view_type: string;
  view_id: string;
  seed_op_ids: string[];
  task_id: string;
}

interface DrainResult {
  viewId: string;
  viewType: string;
  component: CanvasViewRenderer | null;
  seedOpIds: string[];
  taskId: string;
}

function simulateDrain(events: MockCanvasViewEvent[]): DrainResult[] {
  const results: DrainResult[] = [];
  for (const ev of events) {
    if (ev.kind !== 'view_spawned') continue;
    const component = resolveRenderer(ev.view_type);
    results.push({
      viewId: ev.view_id,
      viewType: ev.view_type,
      component,
      seedOpIds: ev.seed_op_ids ?? [],
      taskId: ev.task_id ?? '',
    });
  }
  return results;
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

test('resolveRenderer returns null for unregistered view_type', () => {
  const result = resolveRenderer('h15-test-view-type-not-registered-9f3a');
  assert.equal(result, null, 'unknown view_type must return null (fallback path)');
});

test('registerRenderer + resolveRenderer round-trip for comparison_board', () => {
  const comp = mockRenderer('comparison_board-test');
  registerRenderer('comparison_board', comp);
  const resolved = resolveRenderer('comparison_board');
  assert.equal(
    resolved,
    comp,
    'resolveRenderer must return the registered component for comparison_board',
  );
});

test('registeredViewTypes includes comparison_board after registration', () => {
  registerRenderer('comparison_board', mockRenderer('comparison_board-check'));
  const types = registeredViewTypes();
  assert.ok(
    types.includes('comparison_board'),
    `registeredViewTypes must include 'comparison_board'; got: ${types.join(', ')}`,
  );
});

test('drain simulation: ViewSpawned with registered view_type yields non-null component', () => {
  const comp = mockRenderer('comparison_board-drain');
  registerRenderer('comparison_board', comp);

  const events: MockCanvasViewEvent[] = [
    {
      kind: 'view_spawned',
      view_type: 'comparison_board',
      view_id: 'view-h15-drain-test-1',
      seed_op_ids: ['op-seed-a', 'op-seed-b'],
      task_id: 't-drain-test',
    },
  ];

  const results = simulateDrain(events);

  assert.equal(results.length, 1, 'one ViewSpawned event must produce one drain result');
  assert.equal(results[0]?.component, comp, 'drain must resolve the registered component');
  assert.deepEqual(
    results[0]?.seedOpIds,
    ['op-seed-a', 'op-seed-b'],
    'seed_op_ids must pass through the drain unchanged',
  );
  assert.equal(results[0]?.viewId, 'view-h15-drain-test-1', 'viewId must be preserved');
  assert.equal(results[0]?.taskId, 't-drain-test', 'taskId must be preserved');
});

test('drain simulation: ViewSpawned with unregistered view_type yields null component', () => {
  const events: MockCanvasViewEvent[] = [
    {
      kind: 'view_spawned',
      view_type: 'h15-unknown-view-type-drain-2a7f',
      view_id: 'view-unknown-1',
      seed_op_ids: [],
      task_id: 't-unknown',
    },
  ];

  const results = simulateDrain(events);

  assert.equal(results.length, 1, 'unregistered ViewSpawned must still produce a drain result');
  assert.equal(
    results[0]?.component,
    null,
    'unregistered view_type must yield null component (console.warn fallback path in CanvasWindow)',
  );
});

test('drain simulation: non-ViewSpawned events (UiAction, StateCheckpoint) are skipped', () => {
  registerRenderer('comparison_board', mockRenderer('comparison_board-skip-test'));

  const events: MockCanvasViewEvent[] = [
    {
      kind: 'ui_action',
      view_type: '',
      view_id: '',
      seed_op_ids: [],
      task_id: 't-ui-action',
    },
    {
      kind: 'state_checkpoint',
      view_type: '',
      view_id: '',
      seed_op_ids: [],
      task_id: 't-checkpoint',
    },
    {
      kind: 'view_spawned',
      view_type: 'comparison_board',
      view_id: 'view-only-spawned',
      seed_op_ids: ['op-1'],
      task_id: 't-only-spawned',
    },
  ];

  const results = simulateDrain(events);

  assert.equal(
    results.length,
    1,
    'only ViewSpawned events are drained to UI panels; ui_action + state_checkpoint are skipped',
  );
  assert.equal(results[0]?.viewId, 'view-only-spawned');
});

test('registerRenderer is idempotent (re-register replaces)', () => {
  const comp1 = mockRenderer('replace-test-v1');
  const comp2 = mockRenderer('replace-test-v2');

  registerRenderer('h15-test-idempotent', comp1);
  assert.equal(resolveRenderer('h15-test-idempotent'), comp1);

  registerRenderer('h15-test-idempotent', comp2);
  assert.equal(
    resolveRenderer('h15-test-idempotent'),
    comp2,
    're-registration must replace the previous renderer',
  );
});
