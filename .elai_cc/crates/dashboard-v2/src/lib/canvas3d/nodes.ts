// canvas3d/nodes.ts — node rendering: instanced cores + a glow point cloud.
//
// Two draw calls total regardless of node count:
//   1. InstancedMesh of small boxes (the solid cores, raycast target)
//   2. THREE.Points with a radial sprite texture (the bloom-free glow;
//      active nodes pulse via a per-point phase attribute + time uniform)
// Rebuilt wholesale on graph rev change (hundreds of nodes — trivial);
// only the time uniform updates per frame.

import * as THREE from 'three';
import type { C3Graph } from './types';
import { fnvHash } from './types';
import type { LayoutResult } from './layout';
import type { C3Theme } from './theme';

const CORE_SIZE = 3.4;
const MAX_INSTANCES = 2048;

const GLOW_VERT = /* glsl */ `
  attribute float aSize;
  attribute vec3 aColor;
  attribute float aPhase;
  attribute float aPulse;
  uniform float uTime;
  varying vec3 vColor;
  varying float vBoost;
  void main() {
    float pulse = aPulse * (0.5 + 0.5 * sin(uTime * 2.6 + aPhase));
    vBoost = 1.0 + pulse * 0.9;
    vColor = aColor;
    vec4 mv = modelViewMatrix * vec4(position, 1.0);
    gl_PointSize = aSize * vBoost * (240.0 / -mv.z);
    gl_Position = projectionMatrix * mv;
  }
`;

const GLOW_FRAG = /* glsl */ `
  uniform sampler2D uTex;
  varying vec3 vColor;
  varying float vBoost;
  void main() {
    float a = texture2D(uTex, gl_PointCoord).a;
    gl_FragColor = vec4(vColor * vBoost, a);
  }
`;

export interface NodeBuffers {
  cores: THREE.InstancedMesh;
  glow: THREE.Points;
  /** instanceId -> nodeId, for raycast picking */
  pickIndex: string[];
  dispose(): void;
  setTime(t: number): void;
  /** rewrite one node's tint (selection highlight) without a full rebuild */
  highlight(nodeId: string | null, theme: C3Theme): void;
}

export function buildNodes(
  g: C3Graph,
  layout: LayoutResult,
  theme: C3Theme,
  glowTexture: THREE.Texture,
): NodeBuffers {
  const ids: string[] = [];
  for (const id of layout.visible) {
    if (layout.placements.has(id)) ids.push(id);
  }
  ids.sort(); // deterministic instance order
  const count = Math.min(ids.length, MAX_INSTANCES);

  // --- cores ---
  const coreGeo = new THREE.BoxGeometry(CORE_SIZE, CORE_SIZE * 0.62, CORE_SIZE);
  const coreMat = new THREE.MeshBasicMaterial({
    transparent: true,
    opacity: 0.92,
    toneMapped: false,
  });
  const cores = new THREE.InstancedMesh(coreGeo, coreMat, count);
  cores.renderOrder = 3;

  // --- glow ---
  const gPos = new Float32Array(count * 3);
  const gSize = new Float32Array(count);
  const gColor = new Float32Array(count * 3);
  const gPhase = new Float32Array(count);
  const gPulse = new Float32Array(count);

  const m = new THREE.Matrix4();
  const baseColors = new Map<string, THREE.Color>();

  for (let i = 0; i < count; i++) {
    const id = ids[i];
    const node = g.nodes.get(id);
    const place = layout.placements.get(id);
    if (!node || !place) continue;
    const [x, y, z] = place.pos;
    const cy = y + CORE_SIZE * 0.31; // core rests on the slab top
    m.makeTranslation(x, cy, z);
    // roots / spawners read bigger
    const big = node.parentId === null || node.spawnedLayerId !== null;
    if (big) {
      m.multiply(new THREE.Matrix4().makeScale(1.7, 1.5, 1.7));
    }
    cores.setMatrixAt(i, m);
    const c = theme.stateColors[node.state];
    baseColors.set(id, c);
    cores.setColorAt(i, c);

    gPos[i * 3] = x;
    gPos[i * 3 + 1] = cy + 0.4;
    gPos[i * 3 + 2] = z;
    const active = node.state === 'active';
    gSize[i] = (big ? 24 : 13) * (active ? 1.35 : 1);
    gColor[i * 3] = c.r;
    gColor[i * 3 + 1] = c.g;
    gColor[i * 3 + 2] = c.b;
    gPhase[i] = (fnvHash(id) % 628) / 100;
    gPulse[i] = active ? 1 : node.state === 'fail' ? 0.6 : 0;
  }
  cores.instanceMatrix.needsUpdate = true;
  if (cores.instanceColor) cores.instanceColor.needsUpdate = true;

  const glowGeo = new THREE.BufferGeometry();
  glowGeo.setAttribute('position', new THREE.BufferAttribute(gPos, 3));
  glowGeo.setAttribute('aSize', new THREE.BufferAttribute(gSize, 1));
  glowGeo.setAttribute('aColor', new THREE.BufferAttribute(gColor, 3));
  glowGeo.setAttribute('aPhase', new THREE.BufferAttribute(gPhase, 1));
  glowGeo.setAttribute('aPulse', new THREE.BufferAttribute(gPulse, 1));
  const glowMat = new THREE.ShaderMaterial({
    uniforms: { uTime: { value: 0 }, uTex: { value: glowTexture } },
    vertexShader: GLOW_VERT,
    fragmentShader: GLOW_FRAG,
    transparent: true,
    blending: THREE.AdditiveBlending,
    depthWrite: false,
  });
  const glow = new THREE.Points(glowGeo, glowMat);
  glow.renderOrder = 4;
  glow.frustumCulled = false;

  const idToInstance = new Map<string, number>();
  for (let i = 0; i < count; i++) idToInstance.set(ids[i], i);
  let highlighted: string | null = null;

  return {
    cores,
    glow,
    pickIndex: ids,
    setTime(t: number) {
      glowMat.uniforms.uTime.value = t;
    },
    highlight(nodeId: string | null, th: C3Theme) {
      if (highlighted !== null) {
        const prev = idToInstance.get(highlighted);
        const base = baseColors.get(highlighted);
        if (prev !== undefined && base) cores.setColorAt(prev, base);
      }
      highlighted = nodeId;
      if (nodeId !== null) {
        const idx = idToInstance.get(nodeId);
        if (idx !== undefined) cores.setColorAt(idx, th.selection);
      }
      if (cores.instanceColor) cores.instanceColor.needsUpdate = true;
    },
    dispose() {
      coreGeo.dispose();
      coreMat.dispose();
      glowGeo.dispose();
      glowMat.dispose();
    },
  };
}
