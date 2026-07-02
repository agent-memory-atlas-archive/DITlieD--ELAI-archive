// canvas3d/types.ts — data model for the standalone 3D layered canvas.
//
// A GRAPH is a stack of LAYERS (translucent slabs). Layer 0 is the root
// model's node tree; when a node spawns a sub-model/agent, the spawned
// tree lives on its OWN layer one slab below, tethered to the spawner by
// a vertical spawn trace. This mirrors the PROCESS-altitude vision
// (J-60: "when ELAI spawns a sub-agent I want to see all it's working
// on") but is fed by a local simulator until SSB wiring lands.
//
// State vocabulary reuses the canonical quinary set from the 2D canvas
// (canvas_state.ts) so later wiring is a data-source swap, not a rewrite.

import type { QuinaryState } from '../canvas/canvas_state';

export type C3NodeKind = 'model' | 'agent' | 'task' | 'tool' | 'file';

export interface C3Node {
  id: string;
  label: string;
  kind: C3NodeKind;
  state: QuinaryState;
  layerId: string;
  /** tree parent within the same layer; null for the layer root */
  parentId: string | null;
  childIds: string[];
  /** layer this node spawned (sub-model with its own tree), if any */
  spawnedLayerId: string | null;
  /** children hidden when false (node still visible itself) */
  expanded: boolean;
}

export interface C3Layer {
  id: string;
  /** 0 = top slab, grows downward */
  depth: number;
  title: string;
  rootId: string;
  /** node on the parent layer that spawned this layer; null for layer 0 */
  spawnerNodeId: string | null;
}

export type C3EdgeKind = 'tree' | 'spawn';

export interface C3Edge {
  id: string;
  kind: C3EdgeKind;
  /** parent / spawner side */
  a: string;
  /** child / spawned-root side */
  b: string;
}

export interface C3Graph {
  nodes: Map<string, C3Node>;
  layers: Map<string, C3Layer>;
  edges: Map<string, C3Edge>;
  /** ordered top-down */
  layerOrder: string[];
  /** monotonic revision; scene rebuilds when it changes */
  rev: number;
}

export function emptyGraph(): C3Graph {
  return { nodes: new Map(), layers: new Map(), edges: new Map(), layerOrder: [], rev: 0 };
}

/** A node renders iff every ancestor is expanded AND (for spawned layers)
 *  the spawner chain itself is visible. Computed per rebuild. */
export function visibleNodeIds(g: C3Graph): Set<string> {
  const vis = new Set<string>();
  const layerVisible = new Map<string, boolean>();

  const nodeVisible = (id: string): boolean => {
    if (vis.has(id)) return true;
    const n = g.nodes.get(id);
    if (!n) return false;
    if (!isLayerVisible(n.layerId)) return false;
    if (n.parentId !== null) {
      const p = g.nodes.get(n.parentId);
      if (!p || !p.expanded || !nodeVisible(p.id)) return false;
    }
    vis.add(id);
    return true;
  };

  const isLayerVisible = (layerId: string): boolean => {
    const cached = layerVisible.get(layerId);
    if (cached !== undefined) return cached;
    const layer = g.layers.get(layerId);
    let ok = false;
    if (layer) {
      if (layer.spawnerNodeId === null) ok = true;
      else {
        const spawner = g.nodes.get(layer.spawnerNodeId);
        ok = !!spawner && spawner.expanded && nodeVisible(spawner.id);
      }
    }
    layerVisible.set(layerId, ok);
    return ok;
  };

  for (const layerId of g.layerOrder) {
    const layer = g.layers.get(layerId);
    if (layer && isLayerVisible(layerId)) nodeVisible(layer.rootId);
  }
  // second pass: roots pull in descendants breadth-first
  const queue: string[] = [...vis];
  while (queue.length > 0) {
    const id = queue.shift() as string;
    const n = g.nodes.get(id);
    if (!n || !n.expanded) continue;
    for (const c of n.childIds) {
      if (!vis.has(c)) { vis.add(c); queue.push(c); }
    }
  }
  return vis;
}

/** FNV-1a — deterministic per-id hashing (same idiom as orbit/OrbitTendrils). */
export function fnvHash(s: string): number {
  let h = 0x811c9dc5 >>> 0;
  for (let i = 0; i < s.length; i++) {
    h ^= s.charCodeAt(i);
    h = Math.imul(h, 0x01000193) >>> 0;
  }
  return h >>> 0;
}
