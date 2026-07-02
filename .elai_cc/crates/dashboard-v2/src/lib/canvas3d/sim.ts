// canvas3d/sim.ts — seeded live-activity simulator (the standalone feed).
//
// Stands in for the SSB live layer until wiring: grows a root model tree,
// walks activity through it (pending -> active -> done/warn/fail), and
// occasionally has an active model/agent spawn a sub-layer with its own
// tree. Deterministic under a fixed seed (mulberry32) so captures are
// reproducible. Later wiring replaces THIS FILE ONLY: anything that
// mutates C3Graph and bumps rev drives the identical render path.

import type { C3Graph, C3Node, C3NodeKind } from './types';
import { emptyGraph } from './types';

const KIND_LABELS: Record<C3NodeKind, string[]> = {
  model: ['qwen-coder', 'planner-core', 'critic-loop', 'synth-unit', 'eval-head'],
  agent: ['auditor', 'explorer', 'compiler', 'fixer', 'grounder', 'memory-writer', 'mapper'],
  task: ['parse_spec', 'route_ops', 'diff_scan', 'wire_check', 'grc_step', 'index_pass',
    'probe_seam', 'gate_eval', 'trace_join', 'rank_nodes', 'fold_ctx', 'emit_ssb'],
  tool: ['grep', 'read', 'edit', 'bash', 'search', 'raycast', 'ingest'],
  file: ['grc.rs', 'ssb.rs', 'fsm.rs', 'lib.rs', 'canvas.ts', 'server.py', 'runner.rs'],
};

function mulberry32(seed: number): () => number {
  let a = seed >>> 0;
  return () => {
    a = (a + 0x6d2b79f5) >>> 0;
    let t = a;
    t = Math.imul(t ^ (t >>> 15), t | 1);
    t ^= t + Math.imul(t ^ (t >>> 7), t | 61);
    return ((t ^ (t >>> 14)) >>> 0) / 4294967296;
  };
}

export interface SimOptions {
  seed?: number;
  maxLayers?: number;
  maxNodes?: number;
}

export class CanvasSim {
  readonly graph: C3Graph;
  private rnd: () => number;
  private nextId = 0;
  private maxLayers: number;
  private maxNodes: number;
  /** ticks a node has spent active, keyed by id */
  private activeAge = new Map<string, number>();

  constructor(opts: SimOptions = {}) {
    this.rnd = mulberry32(opts.seed ?? 0xe1a1);
    this.maxLayers = opts.maxLayers ?? 6;
    this.maxNodes = opts.maxNodes ?? 420;
    this.graph = emptyGraph();
    this.bootstrap();
  }

  private id(prefix: string): string {
    return `${prefix}-${(this.nextId++).toString(36)}`;
  }

  private pick<T>(arr: T[]): T {
    return arr[Math.floor(this.rnd() * arr.length)];
  }

  private addNode(
    kind: C3NodeKind,
    layerId: string,
    parentId: string | null,
  ): C3Node {
    const n: C3Node = {
      id: this.id(kind),
      label: this.pick(KIND_LABELS[kind]),
      kind,
      state: 'pending',
      layerId,
      parentId,
      childIds: [],
      spawnedLayerId: null,
      expanded: true,
    };
    this.graph.nodes.set(n.id, n);
    if (parentId !== null) {
      const p = this.graph.nodes.get(parentId);
      if (p) {
        p.childIds.push(n.id);
        const eid = `e-${parentId}-${n.id}`;
        this.graph.edges.set(eid, { id: eid, kind: 'tree', a: parentId, b: n.id });
      }
    }
    return n;
  }

  private addLayer(title: string, spawnerNodeId: string | null): string {
    const depth = this.graph.layerOrder.length;
    const layerId = this.id('layer');
    const root = this.addNode('model', layerId, null);
    root.state = 'active';
    root.label = title;
    this.graph.layers.set(layerId, { id: layerId, depth, title, rootId: root.id, spawnerNodeId });
    this.graph.layerOrder.push(layerId);
    if (spawnerNodeId !== null) {
      const spawner = this.graph.nodes.get(spawnerNodeId);
      if (spawner) spawner.spawnedLayerId = layerId;
      const eid = `e-spawn-${spawnerNodeId}-${root.id}`;
      this.graph.edges.set(eid, { id: eid, kind: 'spawn', a: spawnerNodeId, b: root.id });
    }
    // seed a starting brood so a fresh layer is never a lone dot
    const brood = 3 + Math.floor(this.rnd() * 4);
    for (let i = 0; i < brood; i++) {
      this.addNode(this.rnd() < 0.5 ? 'agent' : 'task', layerId, root.id);
    }
    return layerId;
  }

  private bootstrap(): void {
    this.addLayer('elai-root', null);
    // pre-warm: a few growth ticks so the first frame already looks alive
    for (let i = 0; i < 26; i++) this.tick();
    this.graph.rev++;
  }

  private childKind(parent: C3Node): C3NodeKind {
    if (parent.kind === 'model') return this.rnd() < 0.45 ? 'agent' : 'task';
    if (parent.kind === 'agent') return this.rnd() < 0.55 ? 'task' : (this.rnd() < 0.5 ? 'tool' : 'file');
    return this.rnd() < 0.5 ? 'tool' : 'file';
  }

  /** One simulation step. Returns true when anything changed. */
  tick(): boolean {
    const g = this.graph;
    let changed = false;
    const nodes = [...g.nodes.values()];

    for (const n of nodes) {
      if (n.state === 'active') {
        const age = (this.activeAge.get(n.id) ?? 0) + 1;
        this.activeAge.set(n.id, age);

        // grow: active non-leaf kinds sprout children
        const canGrow =
          g.nodes.size < this.maxNodes &&
          n.kind !== 'tool' && n.kind !== 'file' &&
          n.childIds.length < 9;
        if (canGrow && this.rnd() < 0.22) {
          this.addNode(this.childKind(n), n.layerId, n.id);
          changed = true;
        }

        // spawn: an active model/agent may fork its own sub-layer
        const canSpawn =
          (n.kind === 'model' || n.kind === 'agent') &&
          n.spawnedLayerId === null &&
          g.layerOrder.length < this.maxLayers &&
          g.nodes.size < this.maxNodes;
        if (canSpawn && this.rnd() < 0.018) {
          this.addLayer(`${n.label}/sub`, n.id);
          changed = true;
        }

        // settle: long-active nodes resolve
        if (age > 4 && this.rnd() < 0.3) {
          const roll = this.rnd();
          n.state = roll < 0.87 ? 'done' : roll < 0.94 ? 'warn' : 'fail';
          this.activeAge.delete(n.id);
          changed = true;
        }
      } else if (n.state === 'pending') {
        // activate off a live frontier: parent active or done
        const p = n.parentId !== null ? (g.nodes.get(n.parentId) ?? null) : null;
        const frontier = p === null || p.state === 'active' || p.state === 'done';
        if (frontier && this.rnd() < 0.16) {
          n.state = 'active';
          changed = true;
        }
      } else if (n.state === 'warn' && this.rnd() < 0.04) {
        n.state = 'active'; // retried
        changed = true;
      }
    }
    if (changed) g.rev++;
    return changed;
  }
}
