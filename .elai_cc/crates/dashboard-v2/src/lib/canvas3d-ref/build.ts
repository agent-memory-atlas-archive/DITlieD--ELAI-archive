// canvas3d-ref/build.ts — exact replica of the reference stack image.
//
// What makes the reference read the way it does, and how each is built:
//   1. THE LIGHT IS THE STRUCTURE: every wireframe vertex carries a baked
//      luminance = small hashed base + sum of contributions from nearby
//      node-lights falling off with distance² — so nodes ILLUMINATE the
//      lattice around them and the image has a continuous tonal ladder
//      from near-black to white-hot, not two flat registers.
//   2. MULTI-SCALE DENSITY: cells nest (box -> 2x2 sub-boxes -> sub-sub),
//      plus occasional 2-cell-wide big frames, so the structure reads as
//      a machine with depth at several scales.
//   3. TIGHT LIGHT: nodes are compact white boxes; bloom is narrow, so
//      cores keep their shape against pure black.
//   4. Slabs nearly touch; a thin wider plinth sits under the stack.
//
// Deterministic (FNV-hashed) — same scene every load. Monochrome white
// like the reference; TINT rotates the whole thing to a hue (e.g. ELAI
// teal) without touching the luminance ladder.

import * as THREE from 'three';

// FNV-1a
function fnv(s: string): number {
  let h = 0x811c9dc5 >>> 0;
  for (let i = 0; i < s.length; i++) {
    h ^= s.charCodeAt(i);
    h = Math.imul(h, 0x01000193) >>> 0;
  }
  return h >>> 0;
}
const rand = (key: string) => (fnv(key) % 100000) / 100000;

// ---------------------------------------------------------------- config

/** hue of the whole hologram; pure white replicates the reference.
 *  For ELAI teal: new THREE.Color('#bfe8e0') keeps the ladder, tints it. */
export const TINT = new THREE.Color(1, 1, 1);

const SEED = 'refstack-1';
const LAYERS = 4;
const SIDE = 18; // cells per slab edge
const CELL = 10;
const WORLD = SIDE * CELL;
// reference proportion: stack height ~ its width, slabs clearly separated
const THICK = 24;
const GAP = 16;
const DY = THICK + GAP;
/** luminous nodes per slab */
const NODES_MIN = 5;
const NODES_VAR = 4;

const BASE_LUM = 0.04;
const BASE_VAR = 0.06;
/** node light: FINITE support — I = A * (1 - d/R)^2, zero beyond R.
 *  Infinite-tail kernels sum to washout over a whole lattice. */
const NODE_A = 0.85;
const NODE_R = 38;
const LUM_CLAMP = 1.25;
/** rare hot streak segments (the random bright wires in the reference) */
const STREAK_SHARE = 0.012;

export interface RefNode {
  pos: THREE.Vector3;
  layer: number;
}

export interface RefStackBuild {
  group: THREE.Group;
  nodes: RefNode[];
  worldSize: number;
  stackHeight: number;
  dispose(): void;
}

// ------------------------------------------------------------ generation

interface Light {
  x: number;
  y: number;
  z: number;
  a: number;
  /** hard support radius — zero contribution beyond */
  r: number;
}

function topY(layer: number): number {
  return -layer * DY;
}

/** push one line segment; lum resolved later against the light field */
interface Seg {
  ax: number; ay: number; az: number;
  bx: number; by: number; bz: number;
  base: number;
}

function boxSegs(
  segs: Seg[],
  x0: number, x1: number, y0: number, y1: number, z0: number, z1: number,
  base: number,
): void {
  const e: [number, number, number, number, number, number][] = [
    [x0, y0, z0, x1, y0, z0], [x1, y0, z0, x1, y0, z1], [x1, y0, z1, x0, y0, z1], [x0, y0, z1, x0, y0, z0],
    [x0, y1, z0, x1, y1, z0], [x1, y1, z0, x1, y1, z1], [x1, y1, z1, x0, y1, z1], [x0, y1, z1, x0, y1, z0],
    [x0, y0, z0, x0, y1, z0], [x1, y0, z0, x1, y1, z0], [x1, y0, z1, x1, y1, z1], [x0, y0, z1, x0, y1, z1],
  ];
  for (const [ax, ay, az, bx, by, bz] of e) segs.push({ ax, ay, az, bx, by, bz, base });
}

/** recursive nested cell fill — the multi-scale density of the reference */
function fillCell(
  segs: Seg[],
  key: string,
  x0: number, x1: number, z0: number, z1: number,
  yTop: number, yBot: number,
  depth: number,
): void {
  const r = rand(`${key}:fill`);
  const inset = 0.14 * (x1 - x0);
  const bx0 = x0 + inset;
  const bx1 = x1 - inset;
  const bz0 = z0 + inset;
  const bz1 = z1 - inset;
  const hTop = yTop - 0.8 - rand(`${key}:t`) * 2;
  const hBot = yBot + 0.8 + rand(`${key}:b`) * 2;
  const span = hTop - hBot;
  // height band this box occupies
  const mode = rand(`${key}:m`);
  let by0 = hBot;
  let by1 = hTop;
  if (mode < 0.35) by0 = hTop - span * (0.3 + rand(`${key}:h`) * 0.5);
  else if (mode < 0.7) by1 = hBot + span * (0.3 + rand(`${key}:h`) * 0.5);
  const base = BASE_LUM + rand(`${key}:l`) * BASE_VAR;
  boxSegs(segs, bx0, bx1, by0, by1, bz0, bz1, base);

  // nest: subdivide into 2x2 at reduced probability per depth
  if (depth < 2 && r < (depth === 0 ? 0.3 : 0.2)) {
    const mx = (x0 + x1) / 2;
    const mz = (z0 + z1) / 2;
    const quads: [number, number, number, number][] = [
      [x0, mx, z0, mz], [mx, x1, z0, mz], [x0, mx, mz, z1], [mx, x1, mz, z1],
    ];
    for (let q = 0; q < 4; q++) {
      if (rand(`${key}:q${q}`) < 0.5) {
        const [qx0, qx1, qz0, qz1] = quads[q];
        fillCell(segs, `${key}/${q}`, qx0, qx1, qz0, qz1, yTop, yBot, depth + 1);
      }
    }
  }
}

function buildSlabSegs(segs: Seg[], layer: number, wide: number, thick: number): void {
  const half = (SIDE * CELL * wide) / 2;
  const yT = topY(layer);
  const yB = yT - thick;
  // outer frame, brighter
  boxSegs(segs, -half, half, yB, yT, -half, half, 0.34);
  const cells = Math.round(SIDE * wide);
  const c0 = -half;
  for (let cx = 0; cx < cells; cx++) {
    for (let cz = 0; cz < cells; cz++) {
      const key = `${SEED}:${layer}:${cx}:${cz}`;
      if (rand(key) > 0.42) continue;
      // occasional double-wide frame for scale variety
      const dw = rand(`${key}:w`) < 0.06 && cx < cells - 1 ? 2 : 1;
      fillCell(
        segs, key,
        c0 + cx * CELL, c0 + (cx + dw) * CELL,
        c0 + cz * CELL, c0 + (cz + 1) * CELL,
        yT, yB, 0,
      );
    }
  }
}

// ------------------------------------------------------------- assembly

export function buildRefStack(): RefStackBuild {
  const group = new THREE.Group();
  const disposables: { dispose(): void }[] = [];

  // ---- nodes first (they are the lights)
  const nodes: RefNode[] = [];
  const lights: Light[] = [];
  for (let layer = 0; layer < LAYERS; layer++) {
    const n = NODES_MIN + Math.floor(rand(`${SEED}:n:${layer}`) * (NODES_VAR + 1));
    const placed: [number, number][] = [];
    for (let i = 0; i < n; i++) {
      const half = WORLD / 2 - CELL * 2;
      // spread: reject candidates near an already-placed node (ref nodes
      // are distributed, clumps merge into one bloom blob)
      let x = 0;
      let z = 0;
      for (let attempt = 0; attempt < 8; attempt++) {
        const k = `${SEED}:node:${layer}:${i}:${attempt}`;
        x = Math.round(((rand(`${k}x`) * 2 - 1) * half) / CELL) * CELL;
        z = Math.round(((rand(`${k}z`) * 2 - 1) * half) / CELL) * CELL;
        if (placed.every(([px, pz]) => (px - x) ** 2 + (pz - z) ** 2 > 70 * 70)) break;
      }
      placed.push([x, z]);
      const y = topY(layer) - THICK * (0.3 + rand(`${SEED}:node:${layer}:${i}y`) * 0.4);
      nodes.push({ pos: new THREE.Vector3(x, y, z), layer });
      lights.push({ x, y, z, a: NODE_A, r: NODE_R });
    }
  }

  // ---- beams: vertical pillars between vertically-near node pairs,
  //      some spanning several layers (like the reference)
  interface Beam {
    x: number; z: number; y0: number; y1: number;
  }
  const beams: Beam[] = [];
  for (const a of nodes) {
    if (a.layer >= LAYERS - 1) continue;
    if (rand(`${SEED}:beam:${a.pos.x}:${a.pos.z}:${a.layer}`) > 0.6) continue;
    const span = 1 + Math.floor(rand(`${SEED}:span:${a.layer}:${a.pos.x}`) * (LAYERS - a.layer - 1));
    const yEnd = topY(a.layer + span) - THICK * 0.5;
    beams.push({ x: a.pos.x, z: a.pos.z, y0: a.pos.y, y1: yEnd });
    // beams light the lattice they pass through
    const steps = Math.max(2, Math.floor((a.pos.y - yEnd) / 18));
    for (let s = 0; s <= steps; s++) {
      lights.push({ x: a.pos.x, y: a.pos.y + ((yEnd - a.pos.y) * s) / steps, z: a.pos.z, a: 0.32, r: 22 });
    }
  }

  // ---- in-slab bright runs between node pairs on the same layer
  interface Run {
    pts: [number, number, number][];
  }
  const runs: Run[] = [];
  for (let layer = 0; layer < LAYERS; layer++) {
    const layerNodes = nodes.filter((n) => n.layer === layer);
    for (let i = 0; i + 1 < layerNodes.length; i += 2) {
      const a = layerNodes[i].pos;
      const b = layerNodes[i + 1].pos;
      const y = (a.y + b.y) / 2;
      const xFirst = rand(`${SEED}:run:${layer}:${i}`) < 0.5;
      const bend: [number, number, number] = xFirst ? [b.x, y, a.z] : [a.x, y, b.z];
      runs.push({ pts: [[a.x, y, a.z], bend, [b.x, y, b.z]] });
      lights.push({ x: bend[0], y, z: bend[2], a: 0.45, r: 30 });
    }
  }

  // ---- lattice segments for all slabs + plinth
  const segs: Seg[] = [];
  for (let layer = 0; layer < LAYERS; layer++) buildSlabSegs(segs, layer, 1, THICK);
  // thin, slightly wider plinth under the stack
  {
    const half = (WORLD * 1.07) / 2;
    const yT = topY(LAYERS) + GAP * 0.4;
    boxSegs(segs, -half, half, yT - 4, yT, -half, half, 0.22);
    const cells = SIDE + 2;
    for (let cx = 0; cx < cells; cx++) {
      for (let cz = 0; cz < cells; cz++) {
        const key = `${SEED}:p:${cx}:${cz}`;
        if (rand(key) > 0.3) continue;
        const x0 = -half + (cx * 2 * half) / cells;
        const z0 = -half + (cz * 2 * half) / cells;
        const w = (2 * half) / cells;
        boxSegs(segs, x0 + 1.5, x0 + w - 1.5, yT - 3.4, yT - 0.6, z0 + 1.5, z0 + w - 1.5, BASE_LUM + rand(`${key}l`) * 0.05);
      }
    }
  }

  // ---- bake the illumination field into vertex colors
  const pos = new Float32Array(segs.length * 6);
  const col = new Float32Array(segs.length * 6);
  const lum = (x: number, y: number, z: number, base: number, streakKey: string): number => {
    let v = base;
    for (const L of lights) {
      const dx = x - L.x;
      const dy = y - L.y;
      const dz = z - L.z;
      const d2 = dx * dx + dy * dy + dz * dz;
      if (d2 >= L.r * L.r) continue;
      const t = 1 - Math.sqrt(d2) / L.r;
      v += L.a * t * t;
    }
    if (rand(streakKey) < STREAK_SHARE) v = Math.max(v, 0.9 + rand(`${streakKey}b`) * 0.8);
    return Math.min(LUM_CLAMP, v);
  };
  for (let i = 0; i < segs.length; i++) {
    const s = segs[i];
    const o = i * 6;
    pos[o] = s.ax; pos[o + 1] = s.ay; pos[o + 2] = s.az;
    pos[o + 3] = s.bx; pos[o + 4] = s.by; pos[o + 5] = s.bz;
    const streak = `${SEED}:streak:${i}`;
    const la = lum(s.ax, s.ay, s.az, s.base, streak);
    const lb = lum(s.bx, s.by, s.bz, s.base, streak);
    col[o] = TINT.r * la; col[o + 1] = TINT.g * la; col[o + 2] = TINT.b * la;
    col[o + 3] = TINT.r * lb; col[o + 4] = TINT.g * lb; col[o + 5] = TINT.b * lb;
  }
  const latticeGeo = new THREE.BufferGeometry();
  latticeGeo.setAttribute('position', new THREE.BufferAttribute(pos, 3));
  latticeGeo.setAttribute('color', new THREE.BufferAttribute(col, 3));
  const latticeMat = new THREE.LineBasicMaterial({
    vertexColors: true,
    transparent: true,
    opacity: 0.6, // additive overdraw control: rays cross many lines
    blending: THREE.AdditiveBlending,
    depthWrite: false,
    toneMapped: true,
  });
  const lattice = new THREE.LineSegments(latticeGeo, latticeMat);
  lattice.frustumCulled = false;
  group.add(lattice);
  disposables.push(latticeGeo, latticeMat);

  // ---- node cores: compact white boxes, HDR — tight bloom keeps shape
  const coreGeo = new THREE.BoxGeometry(CELL * 0.62, CELL * 0.5, CELL * 0.62);
  const coreMat = new THREE.MeshBasicMaterial({ toneMapped: false });
  const cores = new THREE.InstancedMesh(coreGeo, coreMat, nodes.length);
  const m = new THREE.Matrix4();
  const hot = TINT.clone().multiplyScalar(3.2);
  const warm = TINT.clone().multiplyScalar(1.5);
  for (let i = 0; i < nodes.length; i++) {
    const big = rand(`${SEED}:big:${i}`) < 0.25;
    m.makeScale(big ? 1.5 : 1, big ? 1.4 : 1, big ? 1.5 : 1);
    m.setPosition(nodes[i].pos.x, nodes[i].pos.y, nodes[i].pos.z);
    cores.setMatrixAt(i, m);
    cores.setColorAt(i, rand(`${SEED}:heat:${i}`) < 0.8 ? hot : warm);
  }
  cores.frustumCulled = false;
  group.add(cores);
  disposables.push(coreGeo, coreMat);

  // ---- beams + runs as bright line segments (they bloom into wires of light)
  {
    const bpos: number[] = [];
    const bcol: number[] = [];
    const pushLine = (
      ax: number, ay: number, az: number,
      bx: number, by: number, bz: number,
      l: number,
    ) => {
      bpos.push(ax, ay, az, bx, by, bz);
      for (let k = 0; k < 2; k++) bcol.push(TINT.r * l, TINT.g * l, TINT.b * l);
    };
    for (const b of beams) pushLine(b.x, b.y0, b.z, b.x, b.y1, b.z, 2.6);
    for (const r of runs) {
      for (let i = 0; i + 1 < r.pts.length; i++) {
        pushLine(...r.pts[i], ...r.pts[i + 1], 1.7);
      }
    }
    const geo = new THREE.BufferGeometry();
    geo.setAttribute('position', new THREE.Float32BufferAttribute(bpos, 3));
    geo.setAttribute('color', new THREE.Float32BufferAttribute(bcol, 3));
    const mat = new THREE.LineBasicMaterial({
      vertexColors: true,
      transparent: true,
      blending: THREE.AdditiveBlending,
      depthWrite: false,
      toneMapped: false,
    });
    const lines = new THREE.LineSegments(geo, mat);
    lines.frustumCulled = false;
    group.add(lines);
    disposables.push(geo, mat);
  }

  return {
    group,
    nodes,
    worldSize: WORLD,
    stackHeight: LAYERS * DY,
    dispose() {
      for (const d of disposables) d.dispose();
    },
  };
}
