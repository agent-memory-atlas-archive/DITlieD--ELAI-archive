// canvas3d/nodes.ts — holographic node constructs, three instanced draws:
//   1. CAGE: a real-thickness wireframe cube (12 merged edge bars) — the
//      containment lattice, teal, translucent.
//   2. CORE: a floating octahedron suspended inside the cage, HDR state
//      color, slowly spinning; the bloom pass makes it a live energy gem.
//      Active/fail cores pulse their emissive level.
//   3. PICK PROXY: an invisible solid cube (colorWrite off) so raycast
//      picking hits a full volume instead of thin bars.
// Matrix / GitS "virtual construct" language — no physical-hardware look.

import * as THREE from 'three';
import { mergeGeometries } from 'three/addons/utils/BufferGeometryUtils.js';
import type { C3Graph } from './types';
import { fnvHash } from './types';
import type { C3Theme } from './theme';

const NODE_SIZE = 5.2;
const BAR = 0.34; // cage bar thickness

/** 12 edge bars of a cube, merged into one geometry */
function cageGeometry(size: number, bar: number): THREE.BufferGeometry {
  const h = size / 2;
  const parts: THREE.BoxGeometry[] = [];
  const mk = (sx: number, sy: number, sz: number, x: number, y: number, z: number) => {
    const g = new THREE.BoxGeometry(sx, sy, sz);
    g.translate(x, y, z);
    parts.push(g);
  };
  for (const y of [-h, h]) {
    for (const z of [-h, h]) mk(size + bar, bar, bar, 0, y, z);
    for (const x of [-h, h]) mk(bar, bar, size + bar, x, y, 0);
  }
  for (const x of [-h, h]) {
    for (const z of [-h, h]) mk(bar, size + bar, bar, x, 0, z);
  }
  const merged = mergeGeometries(parts);
  for (const p of parts) p.dispose();
  return merged;
}

export interface NodeTransform {
  pos: THREE.Vector3;
  /** 0..1 birth scale */
  scale: number;
}

export interface NodeBuffers {
  /** parent of cage + core + proxy; add/remove this */
  group: THREE.Group;
  /** raycast target (instanceId -> pickIndex) */
  pickMesh: THREE.InstancedMesh;
  pickIndex: string[];
  dispose(): void;
  /** per-frame, BEFORE updateTransforms: drives pulse + core spin */
  setTime(t: number): void;
  updateTransforms(get: (id: string) => NodeTransform | null): void;
  highlight(nodeId: string | null, theme: C3Theme): void;
}

export function buildNodes(
  g: C3Graph,
  ids: string[],
  theme: C3Theme,
  _glowTexture: THREE.Texture,
): NodeBuffers {
  const count = ids.length;
  const cap = Math.max(1, count);

  const cageGeo = cageGeometry(NODE_SIZE, BAR);
  const cageMat = new THREE.MeshBasicMaterial({
    transparent: true,
    opacity: 0.9,
    toneMapped: false,
  });
  const cage = new THREE.InstancedMesh(cageGeo, cageMat, cap);
  cage.renderOrder = 3;
  cage.frustumCulled = false;

  const coreGeo = new THREE.OctahedronGeometry(NODE_SIZE * 0.36, 0);
  const coreMat = new THREE.MeshBasicMaterial({ toneMapped: false });
  const core = new THREE.InstancedMesh(coreGeo, coreMat, cap);
  core.renderOrder = 4;
  core.frustumCulled = false;

  const proxyGeo = new THREE.BoxGeometry(NODE_SIZE * 1.15, NODE_SIZE * 1.15, NODE_SIZE * 1.15);
  const proxyMat = new THREE.MeshBasicMaterial({ colorWrite: false, depthWrite: false });
  const proxy = new THREE.InstancedMesh(proxyGeo, proxyMat, cap);
  proxy.frustumCulled = false;

  const group = new THREE.Group();
  group.add(cage, core, proxy);

  const coreBase: THREE.Color[] = new Array(count);
  const cageBase: THREE.Color[] = new Array(count);
  const bigScale = new Float32Array(count);
  const phase = new Float32Array(count);
  const pulse = new Float32Array(count);
  const pulsing: number[] = [];

  const cageTint = theme.gridBright.clone().multiplyScalar(0.55);
  for (let i = 0; i < count; i++) {
    const id = ids[i];
    const node = g.nodes.get(id);
    const state = node?.state ?? 'pending';
    const big = node !== undefined && (node.parentId === null || node.spawnedLayerId !== null);
    bigScale[i] = big ? 1.6 : 1;
    coreBase[i] = theme.stateColors[state];
    // the cage echoes the state faintly; the core carries the voltage
    cageBase[i] =
      state === 'fail' || state === 'warn'
        ? coreBase[i].clone().multiplyScalar(0.4)
        : cageTint;
    cage.setColorAt(i, cageBase[i]);
    core.setColorAt(i, coreBase[i]);
    phase[i] = (fnvHash(id) % 628) / 100;
    if (state === 'active') {
      pulse[i] = 0.5;
      pulsing.push(i);
    } else if (state === 'fail') {
      pulse[i] = 0.35;
      pulsing.push(i);
    }
  }
  if (cage.instanceColor) cage.instanceColor.needsUpdate = true;
  if (core.instanceColor) core.instanceColor.needsUpdate = true;

  const idToInstance = new Map<string, number>();
  for (let i = 0; i < count; i++) idToInstance.set(ids[i], i);
  let highlighted: string | null = null;
  let now = 0;

  const m = new THREE.Matrix4();
  const q = new THREE.Quaternion();
  const eul = new THREE.Euler();
  const sv = new THREE.Vector3();
  const tmpC = new THREE.Color();

  return {
    group,
    pickMesh: proxy,
    pickIndex: ids,
    setTime(t: number) {
      now = t;
      for (const i of pulsing) {
        const k = 1 + pulse[i] * Math.sin(t * 3.1 + phase[i]);
        tmpC.copy(coreBase[i]).multiplyScalar(k);
        core.setColorAt(i, tmpC);
      }
      if (pulsing.length > 0 && core.instanceColor) core.instanceColor.needsUpdate = true;
    },
    updateTransforms(get: (id: string) => NodeTransform | null) {
      for (let i = 0; i < count; i++) {
        const tr = get(ids[i]);
        if (tr === null) continue;
        const s = Math.max(0.001, tr.scale) * bigScale[i];
        // cage + proxy: axis-aligned
        m.makeScale(s, s, s);
        m.setPosition(tr.pos.x, tr.pos.y, tr.pos.z);
        cage.setMatrixAt(i, m);
        proxy.setMatrixAt(i, m);
        // core: slow holographic spin + gentle bob
        eul.set(0, now * 0.7 + phase[i], 0);
        q.setFromEuler(eul);
        sv.setScalar(s);
        m.compose(
          new THREE.Vector3(
            tr.pos.x,
            tr.pos.y + Math.sin(now * 1.3 + phase[i]) * 0.35 * s,
            tr.pos.z,
          ),
          q,
          sv,
        );
        core.setMatrixAt(i, m);
      }
      cage.instanceMatrix.needsUpdate = true;
      core.instanceMatrix.needsUpdate = true;
      proxy.instanceMatrix.needsUpdate = true;
    },
    highlight(nodeId: string | null, th: C3Theme) {
      if (highlighted !== null) {
        const prev = idToInstance.get(highlighted);
        if (prev !== undefined) cage.setColorAt(prev, cageBase[prev]);
      }
      highlighted = nodeId;
      if (nodeId !== null) {
        const idx = idToInstance.get(nodeId);
        if (idx !== undefined) cage.setColorAt(idx, th.selection);
      }
      if (cage.instanceColor) cage.instanceColor.needsUpdate = true;
    },
    dispose() {
      cageGeo.dispose();
      cageMat.dispose();
      coreGeo.dispose();
      coreMat.dispose();
      proxyGeo.dispose();
      proxyMat.dispose();
    },
  };
}
