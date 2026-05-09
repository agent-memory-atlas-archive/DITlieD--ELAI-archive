// three-orbit-modes.ts — geometry builders for non-plan orbit modes.
//
// Plan mode (sun + phase/step/baby decomposition rings) lives in
// three-orbit.ts and is unchanged. This module adds two more 3D
// builders that can replace it when the orbit is in MAP or RESEARCH
// mode:
//
//   • buildThreeMap(fixture, config)
//       Codebase galaxy: Fruchterman-Reingold force-directed layout in
//       3D (xy + shallow z spread). PageRank → node size + brightness.
//       Edges as faint LineSegments.  Integration-point glyphs (nodes
//       with high cross-file edge counts) get a thin teal ring.
//
//   • buildThreeResearch(fixture, config)
//       Tendril paths: research tree branches from the sun outward into
//       deep space. SEARCH/EXTRACT nodes spawn QuadraticBezierCurve3
//       tendrils with returning tier-colored particles.
//
// Both builders expose the same `ThreeOrbitRefs`-shaped interface as
// the plan builder so StarCanvas can dispatch by mode without caring
// about the geometry differences. Active-state highlighting and the
// flowing-line shader are no-ops for these modes (refreshActive +
// updateAnimation still get called each frame; map/research handle
// their own per-frame updates internally).

import * as THREE from 'three';
import type {
  OrbitFixture,
  PageRankNodeStub,
  DkgEdgeStub,
} from '../orbit/types.ts';
import type { ResearchTreeNode, SourceTier } from '../api/types.ts';
import { lcg, hashId, combineSeeds } from '../orbit/lcg.ts';
import type { ThreeOrbitRefs, OrbitNodeRef } from './three-orbit.ts';
import { ORBIT_OVERLAY_LAYER } from './three-orbit.ts';

// -----------------------------------------------------------------
// Shared: procedural radial-gradient halo texture (mirrors three-orbit.ts).
// One canvas shared across all halo Sprites.
// -----------------------------------------------------------------
let _haloTex: THREE.CanvasTexture | null = null;
function getHaloTexture(): THREE.CanvasTexture {
  if (_haloTex) return _haloTex;
  if (typeof document === 'undefined') {
    const fake = new THREE.DataTexture(new Uint8Array([255, 255, 255, 255]), 1, 1);
    fake.needsUpdate = true;
    _haloTex = fake as unknown as THREE.CanvasTexture;
    return _haloTex;
  }
  const SIZE = 128;
  const canvas = document.createElement('canvas');
  canvas.width = SIZE;
  canvas.height = SIZE;
  const ctx = canvas.getContext('2d');
  if (!ctx) {
    const fake = new THREE.DataTexture(new Uint8Array([255, 255, 255, 255]), 1, 1);
    fake.needsUpdate = true;
    _haloTex = fake as unknown as THREE.CanvasTexture;
    return _haloTex;
  }
  const cx = SIZE / 2;
  const cy = SIZE / 2;
  const grd = ctx.createRadialGradient(cx, cy, 0, cx, cy, SIZE / 2);
  grd.addColorStop(0.00, 'rgba(255,255,255,1.0)');
  grd.addColorStop(0.10, 'rgba(255,255,255,0.85)');
  grd.addColorStop(0.30, 'rgba(255,255,255,0.40)');
  grd.addColorStop(0.55, 'rgba(255,255,255,0.16)');
  grd.addColorStop(0.80, 'rgba(255,255,255,0.04)');
  grd.addColorStop(1.00, 'rgba(255,255,255,0.00)');
  ctx.fillStyle = grd;
  ctx.fillRect(0, 0, SIZE, SIZE);
  _haloTex = new THREE.CanvasTexture(canvas);
  _haloTex.colorSpace = THREE.SRGBColorSpace;
  return _haloTex;
}

// -----------------------------------------------------------------
// Stub refs that satisfy ThreeOrbitRefs without any plan-mode geometry.
// -----------------------------------------------------------------
function makeEmptyRefs(group: THREE.Group, sun: THREE.Mesh): ThreeOrbitRefs {
  // Reuse existing concept halos around the sun (small, decorative).
  return {
    group,
    sun,
    conceptHalos: [],
    nodes: [],
    activeLine: null,
    activeLineMat: null,
    ambientLines: null,
    ambientLinesMat: null,
    activePulses: [],
    refreshActive: () => { /* no-op */ },
    updateAnimation: () => { /* no-op */ },
    dispose: () => {
      group.traverse((obj) => {
        const m = obj as THREE.Mesh;
        if (m.geometry) m.geometry.dispose();
        const mat = m.material;
        if (mat) {
          if (Array.isArray(mat)) for (const mm of mat) mm.dispose();
          else mat.dispose();
        }
      });
    },
  };
}

// -----------------------------------------------------------------
// Common: small central sun + two-layer corona, world-units-matched
// to plan mode so the stage feels coherent across mode swaps.
// -----------------------------------------------------------------
function buildCentralSun(group: THREE.Group, color: THREE.Color, radius: number): THREE.Mesh {
  const sunGeo = new THREE.SphereGeometry(radius, 32, 32);
  const sunMat = new THREE.MeshBasicMaterial({ color, transparent: false });
  const sun = new THREE.Mesh(sunGeo, sunMat);
  sun.position.set(0, 0, 0);
  group.add(sun);

  // Concept halos — two-layer additive corona.
  const tex = getHaloTexture();
  const outer = new THREE.SpriteMaterial({
    map: tex,
    color: new THREE.Color(0.25, 0.78, 0.65),
    transparent: true,
    opacity: 0.30,
    blending: THREE.AdditiveBlending,
    depthWrite: false,
  });
  const inner = new THREE.SpriteMaterial({
    map: tex,
    color: new THREE.Color(0.40, 0.95, 0.82),
    transparent: true,
    opacity: 0.55,
    blending: THREE.AdditiveBlending,
    depthWrite: false,
  });
  const outerS = new THREE.Sprite(outer);
  outerS.scale.set(200, 200, 1);
  const innerS = new THREE.Sprite(inner);
  innerS.scale.set(120, 120, 1);
  group.add(outerS);
  group.add(innerS);

  return sun;
}

// =================================================================
// MODE 2 — codebase MAP
// =================================================================

interface MapNodeData {
  node: PageRankNodeStub;
  pos: THREE.Vector3;
  vel: THREE.Vector3;     // FR layout velocity (unused after settle)
  degree: number;         // edge degree — used for integration-glyph bonus
  files: Set<string>;     // distinct files this symbol references via edges
}

const TEAL_PALETTE = [
  new THREE.Color(0.40, 0.95, 0.82), // teal
  new THREE.Color(0.40, 0.85, 0.95), // sea
  new THREE.Color(0.55, 0.95, 0.75), // mint
  new THREE.Color(0.50, 0.92, 0.88), // pale aqua
  new THREE.Color(0.55, 0.92, 0.70), // soft jade
  new THREE.Color(0.55, 0.78, 0.95), // periwinkle
  new THREE.Color(0.45, 0.98, 0.92), // ice
];

function effectColor(tags: string[]): THREE.Color {
  // Map effect tags to teal-family hues. Banned: purple, orange, rainbow.
  // unsafe → warm jade (still teal-family); auth → ice; database → sea;
  // network → mint; filesystem → pale aqua. Untagged → palette[0] teal.
  if (tags.includes('unsafe'))     return new THREE.Color(0.55, 0.92, 0.70);
  if (tags.includes('auth'))       return new THREE.Color(0.45, 0.98, 0.92);
  if (tags.includes('database'))   return new THREE.Color(0.40, 0.85, 0.95);
  if (tags.includes('network'))    return new THREE.Color(0.55, 0.95, 0.75);
  if (tags.includes('filesystem')) return new THREE.Color(0.50, 0.92, 0.88);
  return TEAL_PALETTE[0];
}

/** Fruchterman-Reingold-ish layout in 3D. Each node feels:
 *  • repulsive force from every other node (k²/d)
 *  • attractive force along edges (d²/k)
 *  • a mild central pull so the graph stays bounded.
 *  z is initialized in [-200, 200] so the graph reads as 3D, not flat.
 *  Iterations capped at 80 for sub-100ms layout time on ~200-node maps.
 */
function fruchtermanReingold3D(
  nodes: PageRankNodeStub[],
  edges: DkgEdgeStub[],
  area: number,
  iterations: number,
): Map<string, THREE.Vector3> {
  const n = nodes.length;
  if (n === 0) return new Map();
  // k = ideal edge length; classic FR uses sqrt(area/n).
  const k = Math.sqrt(area / Math.max(1, n));
  const k2 = k * k;
  const positions = new Map<string, THREE.Vector3>();
  const disps = new Map<string, THREE.Vector3>();

  // Deterministic init: hash id → polar position with slight z spread.
  for (const node of nodes) {
    const rng = lcg(hashId(node.id));
    const r = Math.sqrt(rng()) * Math.sqrt(area) * 0.5;
    const theta = rng() * Math.PI * 2;
    const z = (rng() - 0.5) * 400; // ±200 z spread
    positions.set(node.id, new THREE.Vector3(Math.cos(theta) * r, Math.sin(theta) * r, z));
    disps.set(node.id, new THREE.Vector3());
  }

  let temperature = Math.sqrt(area) * 0.10;
  const cool = temperature / Math.max(1, iterations);
  const tmp = new THREE.Vector3();

  // Repulsive cap: O(n^2) repulsion is capped at 200 nodes — beyond
  // that we'd need Barnes-Hut, which is out of scope here. We also need
  // to filter EDGES to those whose source AND target are inside the cap
  // — otherwise high-degree nodes beyond the cap receive attractive
  // forces with no counter-balancing repulsion and collapse toward the
  // boundary (auditor WARN, fixed via option (a)).
  const cap = Math.min(n, 200);
  const arr = nodes.slice(0, cap);
  const inCap = new Set<string>();
  for (const u of arr) inCap.add(u.id);
  const cappedEdges = edges.filter((e) => inCap.has(e.source) && inCap.has(e.target));

  for (let it = 0; it < iterations; it++) {
    // Reset displacements.
    for (const d of disps.values()) d.set(0, 0, 0);

    // Repulsive: O(n^2) over the capped node set.
    for (let i = 0; i < arr.length; i++) {
      const u = arr[i];
      const pu = positions.get(u.id);
      if (!pu) continue;
      const du = disps.get(u.id);
      if (!du) continue;
      for (let j = i + 1; j < arr.length; j++) {
        const v = arr[j];
        const pv = positions.get(v.id);
        if (!pv) continue;
        const dv = disps.get(v.id);
        if (!dv) continue;
        tmp.subVectors(pu, pv);
        const dist2 = Math.max(1, tmp.lengthSq());
        const force = k2 / dist2;
        tmp.normalize().multiplyScalar(force);
        du.add(tmp);
        dv.sub(tmp);
      }
    }

    // Attractive forces along edges — restricted to edges within the
    // capped node set so the layout stays symmetric (option (a)).
    for (const e of cappedEdges) {
      const pu = positions.get(e.source);
      const pv = positions.get(e.target);
      if (!pu || !pv) continue;
      const du = disps.get(e.source);
      const dv = disps.get(e.target);
      if (!du || !dv) continue;
      tmp.subVectors(pu, pv);
      const dist = Math.max(1, tmp.length());
      const force = (dist * dist) / k * (0.5 + e.weight * 0.5);
      tmp.normalize().multiplyScalar(force);
      du.sub(tmp);
      dv.add(tmp);
    }

    // Apply with temperature cap; pull lightly toward center.
    const centralPull = 0.04;
    for (const node of nodes) {
      const p = positions.get(node.id);
      const d = disps.get(node.id);
      if (!p || !d) continue;
      const dlen = Math.max(1e-3, d.length());
      const cap = Math.min(dlen, temperature);
      d.normalize().multiplyScalar(cap);
      p.add(d);
      // Central pull (helps tight clusters)
      p.x -= p.x * centralPull;
      p.y -= p.y * centralPull;
      p.z -= p.z * centralPull * 0.5;
    }
    temperature = Math.max(0.5, temperature - cool);
  }

  return positions;
}

function buildMapEdges(
  edges: DkgEdgeStub[],
  positions: Map<string, THREE.Vector3>,
): { mesh: THREE.LineSegments; mat: THREE.LineBasicMaterial } | null {
  if (edges.length === 0) return null;
  // Each edge → 2 vertices (LineSegments). Color faded teal; opacity per
  // edge weight is encoded via a vertex-color attribute since all edges
  // share one material.
  const verts: number[] = [];
  const cols: number[] = [];
  const baseColor = new THREE.Color(0.30, 0.62, 0.55);
  for (const e of edges) {
    const a = positions.get(e.source);
    const b = positions.get(e.target);
    if (!a || !b) continue;
    verts.push(a.x, a.y, a.z, b.x, b.y, b.z);
    const w = 0.18 + e.weight * 0.42;
    const c = baseColor.clone().multiplyScalar(w);
    cols.push(c.r, c.g, c.b, c.r, c.g, c.b);
  }
  if (verts.length === 0) return null;
  const geo = new THREE.BufferGeometry();
  geo.setAttribute('position', new THREE.BufferAttribute(new Float32Array(verts), 3));
  geo.setAttribute('color',    new THREE.BufferAttribute(new Float32Array(cols), 3));
  const mat = new THREE.LineBasicMaterial({
    vertexColors: true,
    transparent: true,
    opacity: 0.65,
    depthWrite: false,
    blending: THREE.AdditiveBlending,
  });
  const mesh = new THREE.LineSegments(geo, mat);
  mesh.frustumCulled = false;
  mesh.layers.set(ORBIT_OVERLAY_LAYER);
  return { mesh, mat };
}

/** Integration-point ring around high-degree nodes (>= 4 distinct files
 *  via edges). Thin teal RingGeometry rotated to face the camera each
 *  frame via a Sprite (no per-frame matrix update needed). */
function makeIntegrationRing(scale: number, color: THREE.Color): THREE.Sprite {
  // Use a small canvas with a single ring drawn into it, then render as
  // an additive sprite. Stays cheap (one draw call shared if texture is
  // shared; here we keep one texture across all rings).
  const tex = getIntegrationRingTexture();
  const mat = new THREE.SpriteMaterial({
    map: tex,
    color,
    transparent: true,
    opacity: 0.55,
    blending: THREE.AdditiveBlending,
    depthWrite: false,
  });
  const sprite = new THREE.Sprite(mat);
  sprite.scale.set(scale, scale, 1);
  return sprite;
}

let _ringTex: THREE.CanvasTexture | null = null;
function getIntegrationRingTexture(): THREE.CanvasTexture {
  if (_ringTex) return _ringTex;
  if (typeof document === 'undefined') {
    const fake = new THREE.DataTexture(new Uint8Array([255, 255, 255, 255]), 1, 1);
    fake.needsUpdate = true;
    _ringTex = fake as unknown as THREE.CanvasTexture;
    return _ringTex;
  }
  const SIZE = 128;
  const canvas = document.createElement('canvas');
  canvas.width = SIZE;
  canvas.height = SIZE;
  const ctx = canvas.getContext('2d');
  if (!ctx) {
    const fake = new THREE.DataTexture(new Uint8Array([255, 255, 255, 255]), 1, 1);
    fake.needsUpdate = true;
    _ringTex = fake as unknown as THREE.CanvasTexture;
    return _ringTex;
  }
  ctx.clearRect(0, 0, SIZE, SIZE);
  ctx.strokeStyle = 'rgba(255,255,255,0.95)';
  ctx.lineWidth = 3;
  ctx.beginPath();
  ctx.arc(SIZE / 2, SIZE / 2, SIZE / 2 - 4, 0, Math.PI * 2);
  ctx.stroke();
  // Soft inner haze
  ctx.strokeStyle = 'rgba(255,255,255,0.25)';
  ctx.lineWidth = 7;
  ctx.beginPath();
  ctx.arc(SIZE / 2, SIZE / 2, SIZE / 2 - 4, 0, Math.PI * 2);
  ctx.stroke();
  _ringTex = new THREE.CanvasTexture(canvas);
  _ringTex.colorSpace = THREE.SRGBColorSpace;
  return _ringTex;
}

export function buildThreeMap(fixture: OrbitFixture): ThreeOrbitRefs {
  const group = new THREE.Group();
  const sun = buildCentralSun(group, new THREE.Color(0.92, 1.05, 1.00), 35);

  const nodes = fixture.pagerank_nodes ?? [];
  const edges = fixture.pagerank_edges ?? [];
  if (nodes.length === 0) {
    return makeEmptyRefs(group, sun);
  }

  // Layout in 3D. Area scales roughly with sqrt(nodes) so density stays
  // similar across map sizes.
  const area = 2_500_000; // tuned for ~30-200 nodes; world units squared
  const positions = fruchtermanReingold3D(nodes, edges, area, 60);

  // Edge degree counts (used for integration-point ring).
  const degree = new Map<string, number>();
  const filesPerNode = new Map<string, Set<string>>();
  for (const e of edges) {
    degree.set(e.source, (degree.get(e.source) ?? 0) + 1);
    degree.set(e.target, (degree.get(e.target) ?? 0) + 1);
  }
  const fileById = new Map<string, string>();
  for (const n of nodes) fileById.set(n.id, n.file);
  for (const e of edges) {
    const sFile = fileById.get(e.source);
    const tFile = fileById.get(e.target);
    if (sFile && tFile && sFile !== tFile) {
      const sSet = filesPerNode.get(e.source) ?? new Set<string>();
      sSet.add(tFile);
      filesPerNode.set(e.source, sSet);
      const tSet = filesPerNode.get(e.target) ?? new Set<string>();
      tSet.add(sFile);
      filesPerNode.set(e.target, tSet);
    }
  }

  // Edges first so nodes render on top.
  const edgeMesh = buildMapEdges(edges, positions);
  if (edgeMesh) group.add(edgeMesh.mesh);

  // Node spheres: PageRank score → radius + emissive brightness. Colour
  // from effect_tags (teal-family palette).
  const nodeRefs: OrbitNodeRef[] = [];
  for (const node of nodes) {
    const pos = positions.get(node.id);
    if (!pos) continue;
    const norm = Math.max(0, Math.min(1, node.score / 0.20));
    const radius = 4 + norm * 14;       // 4..18 world units
    const col = effectColor(node.effect_tags).clone().multiplyScalar(0.7 + norm * 0.6);
    const geo = new THREE.SphereGeometry(radius, 14, 14);
    const mat = new THREE.MeshBasicMaterial({ color: col, transparent: false });
    const mesh = new THREE.Mesh(geo, mat);
    mesh.position.copy(pos);
    mesh.layers.set(ORBIT_OVERLAY_LAYER);
    group.add(mesh);

    // Halo sprite — palette-tinted, scale tied to score so important
    // nodes glow more.
    const haloMat = new THREE.SpriteMaterial({
      map: getHaloTexture(),
      color: TEAL_PALETTE[hashId(node.id) % TEAL_PALETTE.length],
      transparent: true,
      opacity: 0.18 + norm * 0.32,
      blending: THREE.AdditiveBlending,
      depthWrite: false,
    });
    const halo = new THREE.Sprite(haloMat);
    const haloScale = radius * (4.0 + norm * 3.0);
    halo.scale.set(haloScale, haloScale, 1);
    halo.position.copy(pos);
    halo.layers.set(ORBIT_OVERLAY_LAYER);
    group.add(halo);

    // Integration-point ring: only for nodes spanning multiple files.
    const fileSpan = filesPerNode.get(node.id)?.size ?? 0;
    if (fileSpan >= 3 || (norm > 0.5 && fileSpan >= 2)) {
      const ring = makeIntegrationRing(radius * 5.0, TEAL_PALETTE[0]);
      ring.position.copy(pos);
      ring.layers.set(ORBIT_OVERLAY_LAYER);
      group.add(ring);
    }

    // Push a minimal node ref so click-to-fly raycasting can find it.
    // The MapNode does not need plan-mode halo tracking, so the
    // outerHalo / outerHaloMat fields are aliased to the same inner halo
    // (no extra resources needed). worldX/Y/Z are required for camera
    // resolution.
    nodeRefs.push({
      id: node.id,
      type: 'phase', // harmless — types are only consulted for plan mode
      mesh,
      halo,
      haloMat,
      haloBaseScale: haloScale,
      haloBaseOpacity: haloMat.opacity,
      outerHalo: halo,
      outerHaloMat: haloMat,
      outerHaloBaseScale: haloScale,
      outerHaloBaseOpacity: haloMat.opacity,
      worldX: pos.x,
      worldY: pos.y,
      worldZ: pos.z,
    });
  }

  return {
    group,
    sun,
    conceptHalos: [],
    nodes: nodeRefs,
    activeLine: null,
    activeLineMat: null,
    ambientLines: edgeMesh ? edgeMesh.mesh : null,
    ambientLinesMat: null,
    activePulses: [],
    refreshActive: () => { /* no-op for map */ },
    updateAnimation: () => { /* no-op for map */ },
    dispose: () => {
      group.traverse((obj) => {
        const m = obj as THREE.Mesh;
        if (m.geometry) m.geometry.dispose();
        const mat = m.material;
        if (mat) {
          if (Array.isArray(mat)) for (const mm of mat) mm.dispose();
          else mat.dispose();
        }
      });
    },
  };
}

// =================================================================
// MODE 3 — research tendrils
// =================================================================

const TIER_COLOR: Record<SourceTier, THREE.Color> = {
  T1: new THREE.Color(1.00, 1.00, 1.00),  // white — top tier
  T2: new THREE.Color(0.53, 0.78, 0.72),  // bright teal
  T3: new THREE.Color(0.42, 0.66, 0.62),  // mid teal
  T4: new THREE.Color(0.32, 0.52, 0.50),  // dim teal
  T5: new THREE.Color(0.24, 0.40, 0.40),  // very dim teal
};

interface TendrilRef {
  curve: THREE.QuadraticBezierCurve3;
  line: THREE.Line;
  /** Particle that travels along the curve when the tendril has returned
   *  a source. null while still "outbound". */
  particle: THREE.Mesh | null;
  particleMat: THREE.MeshBasicMaterial | null;
  particleStartMs: number;
  durationMs: number;
}

function tierForRng(rng: () => number): SourceTier {
  // Bias away from T5 so most particles read clearly.
  const r = rng();
  if (r < 0.10) return 'T1';
  if (r < 0.45) return 'T2';
  if (r < 0.75) return 'T3';
  if (r < 0.93) return 'T4';
  return 'T5';
}

function buildTendrilLine(
  start: THREE.Vector3,
  end: THREE.Vector3,
  control: THREE.Vector3,
  color: THREE.Color,
  outbound: boolean,
): { curve: THREE.QuadraticBezierCurve3; line: THREE.Line } {
  const curve = new THREE.QuadraticBezierCurve3(start, control, end);
  const SEGMENTS = 40;
  const points = curve.getPoints(SEGMENTS);
  const positions = new Float32Array(points.length * 3);
  for (let i = 0; i < points.length; i++) {
    positions[i * 3]     = points[i].x;
    positions[i * 3 + 1] = points[i].y;
    positions[i * 3 + 2] = points[i].z;
  }
  const geo = new THREE.BufferGeometry();
  geo.setAttribute('position', new THREE.BufferAttribute(positions, 3));
  const mat = new THREE.LineBasicMaterial({
    color,
    transparent: true,
    opacity: outbound ? 0.30 : 0.55,
    depthWrite: false,
    blending: THREE.AdditiveBlending,
  });
  const line = new THREE.Line(geo, mat);
  line.frustumCulled = false;
  line.layers.set(ORBIT_OVERLAY_LAYER);
  return { curve, line };
}

export function buildThreeResearch(fixture: OrbitFixture): ThreeOrbitRefs {
  const group = new THREE.Group();
  const sun = buildCentralSun(group, new THREE.Color(0.92, 1.05, 1.00), 35);

  const tree = fixture.research_tree;
  if (!tree) return makeEmptyRefs(group, sun);

  // Layout the research tree radially in 3D. Root at origin (sun);
  // children radiate outward on the xz plane with shallow z spread.
  interface Positioned {
    node: ResearchTreeNode;
    pos: THREE.Vector3;
    parentPos?: THREE.Vector3;
    depth: number;
  }
  const positioned: Positioned[] = [];
  positioned.push({ node: tree, pos: new THREE.Vector3(0, 0, 0), depth: 0 });

  function recurse(parent: Positioned, baseTheta: number, arcSpan: number) {
    const kids = parent.node.children ?? [];
    if (kids.length === 0) return;
    const r = 250 + (parent.depth) * 380;
    const step = kids.length === 1 ? 0 : arcSpan / Math.max(kids.length - 1, 1);
    const start = baseTheta - arcSpan / 2;
    kids.forEach((k, i) => {
      const theta = kids.length === 1 ? baseTheta : start + step * i;
      // place on xy plane (z spread for depth)
      const rng = lcg(hashId(k.id));
      const zJitter = (rng() - 0.5) * 200;
      const x = Math.cos(theta) * r;
      const y = Math.sin(theta) * r;
      const z = zJitter;
      const p: Positioned = {
        node: k,
        pos: new THREE.Vector3(x, y, z),
        parentPos: parent.pos,
        depth: parent.depth + 1,
      };
      positioned.push(p);
      recurse(p, theta, Math.PI / 2.4);
    });
  }
  recurse(positioned[0], -Math.PI / 2, Math.PI * 1.4);

  const nodeRefs: OrbitNodeRef[] = [];

  // Render tree nodes as small spheres + sun-style halos.
  for (const p of positioned) {
    if (p.depth === 0) continue; // root sun already exists
    const radius = 5 + Math.max(0, 4 - p.depth);
    const stageCol = stageColorFor(p.node.stage);
    const geo = new THREE.SphereGeometry(radius, 12, 12);
    const mat = new THREE.MeshBasicMaterial({ color: stageCol, transparent: false });
    const mesh = new THREE.Mesh(geo, mat);
    mesh.position.copy(p.pos);
    mesh.layers.set(ORBIT_OVERLAY_LAYER);
    group.add(mesh);

    const haloMat = new THREE.SpriteMaterial({
      map: getHaloTexture(),
      color: stageCol,
      transparent: true,
      opacity: 0.25 + (p.node.converged ? 0.25 : 0),
      blending: THREE.AdditiveBlending,
      depthWrite: false,
    });
    const halo = new THREE.Sprite(haloMat);
    const haloScale = radius * 7;
    halo.scale.set(haloScale, haloScale, 1);
    halo.position.copy(p.pos);
    halo.layers.set(ORBIT_OVERLAY_LAYER);
    group.add(halo);

    nodeRefs.push({
      id: p.node.id,
      type: 'phase',
      mesh,
      halo,
      haloMat,
      haloBaseScale: haloScale,
      haloBaseOpacity: haloMat.opacity,
      outerHalo: halo,
      outerHaloMat: haloMat,
      outerHaloBaseScale: haloScale,
      outerHaloBaseOpacity: haloMat.opacity,
      worldX: p.pos.x,
      worldY: p.pos.y,
      worldZ: p.pos.z,
    });
  }

  // Tree edges: faint LineSegments parent → child.
  const edgeVerts: number[] = [];
  for (const p of positioned) {
    if (!p.parentPos) continue;
    edgeVerts.push(
      p.parentPos.x, p.parentPos.y, p.parentPos.z,
      p.pos.x, p.pos.y, p.pos.z,
    );
  }
  if (edgeVerts.length > 0) {
    const geo = new THREE.BufferGeometry();
    geo.setAttribute('position', new THREE.BufferAttribute(new Float32Array(edgeVerts), 3));
    const mat = new THREE.LineBasicMaterial({
      color: new THREE.Color(0.32, 0.62, 0.55),
      transparent: true,
      opacity: 0.40,
      depthWrite: false,
      blending: THREE.AdditiveBlending,
    });
    const lines = new THREE.LineSegments(geo, mat);
    lines.frustumCulled = false;
    lines.layers.set(ORBIT_OVERLAY_LAYER);
    group.add(lines);
  }

  // Tendrils: SEARCH/EXTRACT/REFLECT nodes shoot curves outward into
  // deep space (beyond the tree's outermost ring). Returning particles
  // for non-SEARCH (i.e. data has come back).
  const tendrils: TendrilRef[] = [];
  const tNow = (typeof performance !== 'undefined') ? performance.now() : 0;
  const sunPos = new THREE.Vector3(0, 0, 0);
  for (const p of positioned) {
    const stage = p.node.stage;
    if (stage !== 'SEARCH' && stage !== 'EXTRACT' && stage !== 'REFLECT') continue;
    const seed = hashId(p.node.id);
    const rng = lcg(seed);
    const count = stage === 'EXTRACT' ? 3 : (stage === 'REFLECT' ? 2 : 1);
    // direction = away from sun
    const dir = new THREE.Vector3().subVectors(p.pos, sunPos);
    if (dir.lengthSq() < 1e-3) dir.set(1, 0, 0);
    dir.normalize();
    for (let i = 0; i < count; i++) {
      const fan = (i - (count - 1) / 2) * 0.30;
      // rotate dir around y-axis by `fan` for fan spread
      const cos = Math.cos(fan);
      const sin = Math.sin(fan);
      const fx = dir.x * cos - dir.z * sin;
      const fz = dir.x * sin + dir.z * cos;
      const reach = 1500 + rng() * 900;
      const end = new THREE.Vector3(
        p.pos.x + fx * reach,
        p.pos.y + (rng() - 0.5) * 240,
        p.pos.z + fz * reach,
      );
      // control point bowed perpendicular to dir for graceful arc
      const mid = new THREE.Vector3().lerpVectors(p.pos, end, 0.5);
      const perp = new THREE.Vector3(-fz, 0, fx).multiplyScalar((rng() - 0.5) * 280);
      mid.add(perp);
      const tier = tierForRng(rng);
      const outbound = stage === 'SEARCH';
      const { curve, line } = buildTendrilLine(p.pos, end, mid, TIER_COLOR[tier], outbound);
      group.add(line);

      let particle: THREE.Mesh | null = null;
      let particleMat: THREE.MeshBasicMaterial | null = null;
      if (!outbound) {
        const pgeo = new THREE.SphereGeometry(4 + rng() * 3, 10, 10);
        particleMat = new THREE.MeshBasicMaterial({ color: TIER_COLOR[tier].clone(), transparent: true, opacity: 0.95 });
        particle = new THREE.Mesh(pgeo, particleMat);
        particle.position.copy(end);
        particle.layers.set(ORBIT_OVERLAY_LAYER);
        group.add(particle);
      }

      tendrils.push({
        curve,
        line,
        particle,
        particleMat,
        particleStartMs: tNow + rng() * 1800,
        durationMs: 2200 + rng() * 1400,
      });
    }
  }

  // Per-frame: drive returning particles along their curves.
  const _tmp = new THREE.Vector3();
  const updateAnimation = (elapsedMs: number): void => {
    // elapsedMs is StarCanvas-relative; we use the absolute performance.now()
    // captured at build time as the reference.
    const now = (typeof performance !== 'undefined') ? performance.now() : elapsedMs;
    for (const t of tendrils) {
      if (!t.particle || !t.particleMat) continue;
      let phase = (now - t.particleStartMs) / t.durationMs;
      // Loop the particle from end → sun → end with a rest period.
      phase = ((phase % 1.4) + 1.4) % 1.4;
      const u = phase < 1 ? 1 - phase : 1; // 1→0 then hold at 0 (rest)
      const visible = phase < 1.05;
      t.particleMat.opacity = visible ? 0.90 * (1 - Math.max(0, phase - 0.85) / 0.20) : 0.0;
      t.curve.getPoint(u, _tmp);
      t.particle.position.copy(_tmp);
    }
  };

  return {
    group,
    sun,
    conceptHalos: [],
    nodes: nodeRefs,
    activeLine: null,
    activeLineMat: null,
    ambientLines: null,
    ambientLinesMat: null,
    activePulses: [],
    refreshActive: () => { /* no-op for research */ },
    updateAnimation,
    dispose: () => {
      group.traverse((obj) => {
        const m = obj as THREE.Mesh;
        if (m.geometry) m.geometry.dispose();
        const mat = m.material;
        if (mat) {
          if (Array.isArray(mat)) for (const mm of mat) mm.dispose();
          else mat.dispose();
        }
      });
    },
  };
}

function stageColorFor(stage: ResearchTreeNode['stage']): THREE.Color {
  // Teal-family stage palette (no purple/orange/rainbow):
  //   FORMULATE — periwinkle (soft cool blue)
  //   SEARCH    — pale aqua
  //   EXTRACT   — mint
  //   REFLECT   — bright teal
  //   CONVERGE  — ice (almost-white)
  switch (stage) {
    case 'FORMULATE': return new THREE.Color(0.55, 0.78, 0.95);
    case 'SEARCH':    return new THREE.Color(0.50, 0.92, 0.88);
    case 'EXTRACT':   return new THREE.Color(0.55, 0.95, 0.75);
    case 'REFLECT':   return new THREE.Color(0.40, 0.95, 0.82);
    case 'CONVERGE':  return new THREE.Color(0.85, 0.98, 0.92);
  }
}

// Re-export the canonical mode dispatcher's signature so consumers can
// use a single import. combineSeeds is unused here but kept available for
// future per-task seeding inside this module.
export { combineSeeds };

// HMR cleanup — module-level singletons (halo + ring textures) must be
// disposed when this module is replaced or the GPU buffers leak across
// reloads. Auditor flagged this as WARN previously; fixing now.
if (import.meta.hot) {
  import.meta.hot.dispose(() => {
    if (_haloTex) { _haloTex.dispose(); _haloTex = null; }
    if (_ringTex) { _ringTex.dispose(); _ringTex = null; }
  });
}
