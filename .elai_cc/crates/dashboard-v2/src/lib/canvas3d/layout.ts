// canvas3d/layout.ts — depth-plane square adaptive layout + trace metas.
//
// v2 model: a SLAB is a DEPTH PLANE, and every spawned tree whose spawner
// sits at depth d-1 is planted on the SAME plane d — so multiple vertical
// beams drop between adjacent slabs (one per spawning node), matching the
// reference image. The footprint is a SQUARE whose side adapts to the
// densest plane: few nodes = tight square, growth = the square grows.
// Placement stays deterministic (FNV ring-scan); the scene lerps nodes
// between successive placements so relocations glide instead of jump.
//
// Routes have FIXED TOPOLOGY (tree = 3 waypoints, spawn = 4, degenerate
// segments allowed) so the trace buffers can follow animated node
// positions every frame without reallocation.

import { fnvHash, visibleNodeIds } from './types';
import type { C3Graph } from './types';

export const CELL = 10;
export const SLAB_THICK = 16;
/** top-surface to top-surface distance; gap between slabs = LAYER_DY - SLAB_THICK */
export const LAYER_DY = 28;
export const TRACE_LIFT = 2.0;
const MIN_SIDE = 8;
const MAX_SIDE = 44;
/** grid cells provisioned per node on the densest plane */
const SPREAD = 1.4;

export interface NodePlacement {
  col: number;
  row: number;
  depth: number;
  pos: [number, number, number];
}

export interface TraceMeta {
  id: string;
  kind: 'tree' | 'spawn';
  aId: string;
  bId: string;
  /** manhattan bend order for tree traces */
  xFirst: boolean;
}

export interface LayoutResult {
  placements: Map<string, NodePlacement>;
  traces: TraceMeta[];
  visible: Set<string>;
  /** square side in cells (uniform across planes this rev) */
  side: number;
  worldSize: number;
  /** number of occupied depth planes */
  depthCount: number;
}

export function layerTopY(depth: number): number {
  return -depth * LAYER_DY;
}

function cellToWorld(col: number, row: number, depth: number, side: number): [number, number, number] {
  const half = (side * CELL) / 2;
  // nodes live INSIDE the slab volume (mid-thickness), not on its surface
  return [(col + 0.5) * CELL - half, layerTopY(depth) - SLAB_THICK * 0.5, (row + 0.5) * CELL - half];
}

function worldToCell(x: number, z: number, side: number): [number, number] {
  const half = (side * CELL) / 2;
  const clampCell = (v: number) => Math.min(side - 2, Math.max(1, Math.floor(v)));
  return [clampCell((x + half) / CELL), clampCell((z + half) / CELL)];
}

function nearestFreeCell(
  pc: number,
  pr: number,
  hash: number,
  occupied: Set<number>,
  side: number,
): [number, number] | null {
  const inBounds = (c: number, r: number) => c >= 1 && c < side - 1 && r >= 1 && r < side - 1;
  if (inBounds(pc, pr) && !occupied.has(pr * side + pc)) return [pc, pr];
  // start at radius 2: one clear cell between parent and child keeps the
  // manhattan traces readable instead of glow-on-glow adjacency
  for (let radius = 2; radius < side; radius++) {
    const ring: [number, number][] = [];
    for (let dc = -radius; dc <= radius; dc++) {
      for (let dr = -radius; dr <= radius; dr++) {
        if (Math.max(Math.abs(dc), Math.abs(dr)) !== radius) continue;
        ring.push([pc + dc, pr + dr]);
      }
    }
    const start = hash % ring.length;
    for (let i = 0; i < ring.length; i++) {
      const [c, r] = ring[(start + i) % ring.length];
      if (inBounds(c, r) && !occupied.has(r * side + c)) return [c, r];
    }
  }
  return null;
}

export function layoutGraph(g: C3Graph): LayoutResult {
  const visible = visibleNodeIds(g);
  const placements = new Map<string, NodePlacement>();

  // ---- adaptive square side from the densest depth plane
  const perDepth = new Map<number, number>();
  for (const id of visible) {
    const n = g.nodes.get(id);
    const layer = n ? g.layers.get(n.layerId) : undefined;
    if (!layer) continue;
    perDepth.set(layer.depth, (perDepth.get(layer.depth) ?? 0) + 1);
  }
  let maxCount = 0;
  let depthCount = 0;
  for (const [d, c] of perDepth) {
    maxCount = Math.max(maxCount, c);
    depthCount = Math.max(depthCount, d + 1);
  }
  // even side reduces re-layout churn as counts creep up
  let side = Math.ceil(Math.sqrt(maxCount * SPREAD));
  side = Math.min(MAX_SIDE, Math.max(MIN_SIDE, side + (side % 2)));

  // ---- place plane by plane (top down) so spawner cells exist first
  const depths = [...perDepth.keys()].sort((a, b) => a - b);
  for (const depth of depths) {
    const occupied = new Set<number>();
    const queue: string[] = [];
    const put = (id: string, c: number, r: number) => {
      occupied.add(r * side + c);
      placements.set(id, { col: c, row: r, depth, pos: cellToWorld(c, r, depth, side) });
      queue.push(id);
    };
    // roots of every tree living on this plane, in stable layer order
    for (const layerId of g.layerOrder) {
      const layer = g.layers.get(layerId);
      if (!layer || layer.depth !== depth || !visible.has(layer.rootId)) continue;
      let pc: number;
      let pr: number;
      if (layer.spawnerNodeId !== null) {
        const sp = placements.get(layer.spawnerNodeId);
        if (!sp) continue; // spawner hidden this rev
        [pc, pr] = worldToCell(sp.pos[0], sp.pos[2], side);
      } else {
        const h = fnvHash(layer.rootId);
        pc = Math.floor(side / 2) + ((h % 5) - 2);
        pr = Math.floor(side / 2) + ((Math.floor(h / 5) % 5) - 2);
      }
      const cell = nearestFreeCell(pc, pr, fnvHash(layer.rootId), occupied, side);
      if (cell) put(layer.rootId, cell[0], cell[1]);
    }
    // BFS the plane's trees together over the shared occupancy grid
    while (queue.length > 0) {
      const id = queue.shift() as string;
      const n = g.nodes.get(id);
      const pp = placements.get(id);
      if (!n || !pp) continue;
      for (const childId of n.childIds) {
        if (!visible.has(childId) || placements.has(childId)) continue;
        const cell = nearestFreeCell(pp.col, pp.row, fnvHash(childId), occupied, side);
        if (cell === null) continue;
        put(childId, cell[0], cell[1]);
      }
    }
  }

  const traces: TraceMeta[] = [];
  for (const e of g.edges.values()) {
    if (!placements.has(e.a) || !placements.has(e.b)) continue;
    traces.push({ id: e.id, kind: e.kind, aId: e.a, bId: e.b, xFirst: (fnvHash(e.id) & 1) === 0 });
  }
  return { placements, traces, visible, side, worldSize: side * CELL, depthCount };
}
