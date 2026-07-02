// canvas3d/layout.ts — deterministic slab-grid layout + manhattan routing.
//
// Every layer is a COLS x ROWS grid of cells on a slab. Nodes snap to cell
// centers (the pixel-aligned circuit-board look); children are placed on
// the nearest free cell ring-scanned around their parent, with the scan
// start angle derived from the node id hash, so the same graph always
// lays out identically. Tree traces run manhattan (axis-aligned with one
// bend) just above the slab surface; spawn traces drop vertically to the
// spawned layer, whose root is seeded at the spawner's cell.

import { fnvHash, visibleNodeIds } from './types';
import type { C3Graph, C3Layer } from './types';

export const COLS = 30;
export const ROWS = 20;
export const CELL = 10;
export const SLAB_W = COLS * CELL;
export const SLAB_D = ROWS * CELL;
export const SLAB_THICK = 7;
export const LAYER_DY = 66;
/** traces hover this far above the slab top surface */
export const TRACE_LIFT = 2.2;

export interface NodePlacement {
  col: number;
  row: number;
  /** world position of the node center (sits on the slab top) */
  pos: [number, number, number];
}

export interface RoutedEdge {
  id: string;
  kind: 'tree' | 'spawn';
  aId: string;
  bId: string;
  /** polyline waypoints in world space */
  points: [number, number, number][];
}

export interface LayoutResult {
  placements: Map<string, NodePlacement>;
  routes: RoutedEdge[];
  visible: Set<string>;
}

export function layerTopY(depth: number): number {
  return -depth * LAYER_DY;
}

function cellToWorld(col: number, row: number, depth: number): [number, number, number] {
  return [
    (col + 0.5) * CELL - SLAB_W / 2,
    layerTopY(depth),
    (row + 0.5) * CELL - SLAB_D / 2,
  ];
}

const inBounds = (c: number, r: number) => c >= 1 && c < COLS - 1 && r >= 1 && r < ROWS - 1;

/** Ring-scan outward from (pc,pr); start direction from hash. Returns
 *  first free in-bounds cell, or null when the slab is saturated. */
function nearestFreeCell(
  pc: number,
  pr: number,
  hash: number,
  occupied: Set<number>,
): [number, number] | null {
  // start at radius 2: one clear cell between parent and child keeps the
  // manhattan traces readable instead of glow-on-glow adjacency
  for (let radius = 2; radius < Math.max(COLS, ROWS); radius++) {
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
      if (inBounds(c, r) && !occupied.has(r * COLS + c)) return [c, r];
    }
  }
  return null;
}

function placeLayer(
  g: C3Graph,
  layer: C3Layer,
  visible: Set<string>,
  placements: Map<string, NodePlacement>,
  preferred: [number, number] | null,
): void {
  const occupied = new Set<number>();
  const rootHash = fnvHash(layer.rootId);
  let rc: number;
  let rr: number;
  if (preferred !== null && inBounds(preferred[0], preferred[1])) {
    [rc, rr] = preferred;
  } else {
    rc = Math.floor(COLS / 2) + ((rootHash % 7) - 3);
    rr = Math.floor(ROWS / 2) + ((Math.floor(rootHash / 7) % 5) - 2);
  }
  const put = (id: string, c: number, r: number) => {
    occupied.add(r * COLS + c);
    placements.set(id, { col: c, row: r, pos: cellToWorld(c, r, layer.depth) });
  };
  if (!visible.has(layer.rootId)) return;
  put(layer.rootId, rc, rr);

  const queue = [layer.rootId];
  while (queue.length > 0) {
    const id = queue.shift() as string;
    const n = g.nodes.get(id);
    const parentPlace = placements.get(id);
    if (!n || !parentPlace) continue;
    for (const childId of n.childIds) {
      if (!visible.has(childId) || placements.has(childId)) continue;
      const cell = nearestFreeCell(parentPlace.col, parentPlace.row, fnvHash(childId), occupied);
      if (cell === null) continue; // slab saturated; child stays unplaced this rev
      put(childId, cell[0], cell[1]);
      queue.push(childId);
    }
  }
}

/** Manhattan polyline between two placements on the same layer, lifted
 *  above the slab. Bend order (x-first vs z-first) comes from the edge
 *  id hash so parallel edges don't all stack on the same channel. */
function routeTree(
  a: NodePlacement,
  b: NodePlacement,
  depth: number,
  edgeId: string,
): [number, number, number][] {
  const y = layerTopY(depth) + TRACE_LIFT;
  const [ax, , az] = a.pos;
  const [bx, , bz] = b.pos;
  if (a.col === b.col || a.row === b.row) {
    return [[ax, y, az], [bx, y, bz]];
  }
  const xFirst = (fnvHash(edgeId) & 1) === 0;
  const bend: [number, number, number] = xFirst ? [bx, y, az] : [ax, y, bz];
  return [[ax, y, az], bend, [bx, y, bz]];
}

/** Vertical drop from spawner to the spawned layer, then a manhattan run
 *  to the spawned root if the seed cell was taken. */
function routeSpawn(
  a: NodePlacement,
  b: NodePlacement,
  bDepth: number,
): [number, number, number][] {
  const [ax, ay, az] = a.pos;
  const yLow = layerTopY(bDepth) + TRACE_LIFT;
  const [bx, , bz] = b.pos;
  const pts: [number, number, number][] = [[ax, ay, az], [ax, yLow, az]];
  if (Math.abs(ax - bx) > 0.01 || Math.abs(az - bz) > 0.01) {
    pts.push([bx, yLow, az]);
    if (Math.abs(az - bz) > 0.01) pts.push([bx, yLow, bz]);
  }
  return pts;
}

export function layoutGraph(g: C3Graph): LayoutResult {
  const visible = visibleNodeIds(g);
  const placements = new Map<string, NodePlacement>();

  for (const layerId of g.layerOrder) {
    const layer = g.layers.get(layerId);
    if (!layer) continue;
    let preferred: [number, number] | null = null;
    if (layer.spawnerNodeId !== null) {
      const sp = placements.get(layer.spawnerNodeId);
      if (sp === undefined) continue; // spawner hidden -> layer hidden
      preferred = [sp.col, sp.row];
    }
    placeLayer(g, layer, visible, placements, preferred);
  }

  const routes: RoutedEdge[] = [];
  for (const e of g.edges.values()) {
    const pa = placements.get(e.a);
    const pb = placements.get(e.b);
    if (!pa || !pb) continue;
    const nb = g.nodes.get(e.b);
    if (!nb) continue;
    const layer = g.layers.get(nb.layerId);
    if (!layer) continue;
    const points =
      e.kind === 'spawn' ? routeSpawn(pa, pb, layer.depth) : routeTree(pa, pb, layer.depth, e.id);
    routes.push({ id: e.id, kind: e.kind, aId: e.a, bId: e.b, points });
  }
  return { placements, routes, visible };
}
