// canvas3d/traces.ts — the bright animated traces connecting nodes.
//
// ALL edges render as ONE LineSegments draw call with a custom shader:
// each vertex carries its normalized distance along its edge (aT), a
// per-edge phase seed (aSeed), and an activity level (aLive). The
// fragment brightness = dim base + a light packet that travels the edge
// (gaussian around fract(time+seed)). Spawn beams get a hotter base so
// the vertical inter-layer pillars read like the reference image.
// 1px GL lines fatten into chunky pixels under the low-res retro pass.

import * as THREE from 'three';
import type { C3Graph } from './types';
import { fnvHash } from './types';
import type { LayoutResult, RoutedEdge } from './layout';
import type { C3Theme } from './theme';

const TRACE_VERT = /* glsl */ `
  attribute float aT;
  attribute float aSeed;
  attribute float aLive;
  attribute vec3 aColor;
  varying float vT;
  varying float vSeed;
  varying float vLive;
  varying vec3 vColor;
  void main() {
    vT = aT;
    vSeed = aSeed;
    vLive = aLive;
    vColor = aColor;
    gl_Position = projectionMatrix * modelViewMatrix * vec4(position, 1.0);
  }
`;

const TRACE_FRAG = /* glsl */ `
  uniform float uTime;
  varying float vT;
  varying float vSeed;
  varying float vLive;
  varying vec3 vColor;
  void main() {
    float base = 0.42 + vLive * 0.40;
    // light packet travelling a->b, wrapped; live edges run faster
    float speed = 0.10 + vLive * 0.22;
    float head = fract(uTime * speed + vSeed);
    float d = vT - head;
    d -= floor(d + 0.5); // wrap to nearest
    float packet = exp(-d * d * 520.0) * (0.9 + vLive * 1.4);
    float glow = base + packet;
    gl_FragColor = vec4(vColor * glow, min(1.0, glow));
  }
`;

/** An edge is "live" when either endpoint is active — its packet runs
 *  hot and fast; settled edges keep a slow dim heartbeat. */
function liveness(g: C3Graph, e: RoutedEdge): number {
  const a = g.nodes.get(e.aId);
  const b = g.nodes.get(e.bId);
  const active = (a && a.state === 'active') || (b && b.state === 'active');
  const failed = (a && a.state === 'fail') || (b && b.state === 'fail');
  if (active) return 1;
  if (failed) return 0.45;
  return 0;
}

export interface TraceBuffers {
  lines: THREE.LineSegments;
  dispose(): void;
  setTime(t: number): void;
}

export function buildTraces(g: C3Graph, layout: LayoutResult, theme: C3Theme): TraceBuffers {
  const pos: number[] = [];
  const t: number[] = [];
  const seed: number[] = [];
  const live: number[] = [];
  const col: number[] = [];

  for (const edge of layout.routes) {
    const pts = edge.points;
    if (pts.length < 2) continue;
    // cumulative length for aT normalization
    const cum: number[] = [0];
    for (let i = 1; i < pts.length; i++) {
      const dx = pts[i][0] - pts[i - 1][0];
      const dy = pts[i][1] - pts[i - 1][1];
      const dz = pts[i][2] - pts[i - 1][2];
      cum.push(cum[i - 1] + Math.hypot(dx, dy, dz));
    }
    const total = cum[cum.length - 1];
    if (total < 0.01) continue;
    const isSpawn = edge.kind === 'spawn';
    const lv = Math.max(liveness(g, edge), isSpawn ? 0.75 : 0);
    const sd = (fnvHash(edge.id) % 1000) / 1000;
    const c = isSpawn ? theme.spawnBeam : theme.treeTrace;

    for (let i = 1; i < pts.length; i++) {
      pos.push(...pts[i - 1], ...pts[i]);
      t.push(cum[i - 1] / total, cum[i] / total);
      seed.push(sd, sd);
      live.push(lv, lv);
      col.push(c.r, c.g, c.b, c.r, c.g, c.b);
    }
  }

  const geo = new THREE.BufferGeometry();
  geo.setAttribute('position', new THREE.Float32BufferAttribute(pos, 3));
  geo.setAttribute('aT', new THREE.Float32BufferAttribute(t, 1));
  geo.setAttribute('aSeed', new THREE.Float32BufferAttribute(seed, 1));
  geo.setAttribute('aLive', new THREE.Float32BufferAttribute(live, 1));
  geo.setAttribute('aColor', new THREE.Float32BufferAttribute(col, 3));

  const mat = new THREE.ShaderMaterial({
    uniforms: { uTime: { value: 0 } },
    vertexShader: TRACE_VERT,
    fragmentShader: TRACE_FRAG,
    transparent: true,
    blending: THREE.AdditiveBlending,
    depthWrite: false,
  });
  const lines = new THREE.LineSegments(geo, mat);
  lines.renderOrder = 2;
  lines.frustumCulled = false;

  return {
    lines,
    setTime(time: number) {
      mat.uniforms.uTime.value = time;
    },
    dispose() {
      geo.dispose();
      mat.dispose();
    },
  };
}
