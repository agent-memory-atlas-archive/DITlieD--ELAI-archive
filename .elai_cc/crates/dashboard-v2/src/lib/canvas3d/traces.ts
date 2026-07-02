// canvas3d/traces.ts — the data lines, ephemeral hologram style.
//
// Three draws total:
//   - tree traces: fat DASHED lines whose dash offset flows continuously
//     (a stream of light fragments, not a soldered circuit track)
//   - spawn beams: fat solid lines, HDR — the bloom pass turns them into
//     energy pillars
//   - packets: one bright blob riding each live edge (Points)
// Positions + colors rewrite every frame so lines follow the animated
// nodes and new edges fade in. Widths are real pixels (LineMaterial),
// not 1px GL lines.

import * as THREE from 'three';
import { LineSegments2 } from 'three/addons/lines/LineSegments2.js';
import { LineSegmentsGeometry } from 'three/addons/lines/LineSegmentsGeometry.js';
import { LineMaterial } from 'three/addons/lines/LineMaterial.js';
import type { C3Graph } from './types';
import { fnvHash } from './types';
import type { TraceMeta } from './layout';
import type { C3Theme } from './theme';

const PACKET_VERT = /* glsl */ `
  attribute float aSize;
  attribute vec3 aColor;
  varying vec3 vColor;
  void main() {
    vColor = aColor;
    vec4 mv = modelViewMatrix * vec4(position, 1.0);
    gl_PointSize = aSize * (430.0 / -mv.z);
    gl_Position = projectionMatrix * mv;
  }
`;

const PACKET_FRAG = /* glsl */ `
  uniform sampler2D uTex;
  varying vec3 vColor;
  void main() {
    float a = texture2D(uTex, gl_PointCoord).a;
    gl_FragColor = vec4(vColor, a);
  }
`;

/** waypoint count per edge kind (fixed topology) */
const PTS: Record<TraceMeta['kind'], number> = { tree: 3, spawn: 4 };

function liveness(g: C3Graph, meta: TraceMeta): number {
  const a = g.nodes.get(meta.aId);
  const b = g.nodes.get(meta.bId);
  const active = (a && a.state === 'active') || (b && b.state === 'active');
  const failed = (a && a.state === 'fail') || (b && b.state === 'fail');
  if (active) return 1;
  if (failed) return 0.5;
  return 0;
}

export interface TraceBuffers {
  group: THREE.Group;
  dispose(): void;
  setResolution(w: number, h: number): void;
  /** per-frame: follow animated nodes, fade births, flow dashes, ride packets */
  update(getPos: (id: string) => THREE.Vector3 | null, t: number): void;
}

export function buildTraces(
  g: C3Graph,
  metas: TraceMeta[],
  theme: C3Theme,
  birthOf: (edgeId: string) => number,
  glowTex: THREE.Texture,
): TraceBuffers {
  const tree = metas.filter((m) => m.kind === 'tree');
  const spawn = metas.filter((m) => m.kind === 'spawn');

  const mkFat = (width: number, opacity: number, dashed: boolean) => {
    const geo = new LineSegmentsGeometry();
    geo.setPositions([0, 10000, 0, 0, 10000, 0]);
    const mat = new LineMaterial({
      vertexColors: true,
      linewidth: width,
      worldUnits: false,
      transparent: true,
      opacity,
      blending: THREE.AdditiveBlending,
      depthWrite: false,
      dashed,
      dashSize: 3,
      gapSize: 2.4,
    });
    const obj = new LineSegments2(geo, mat);
    obj.renderOrder = 2;
    obj.frustumCulled = false;
    return { obj, geo, mat };
  };
  const treeL = mkFat(2.4, 0.75, true);
  const beamL = mkFat(3.6, 0.85, false);

  // one packet per edge
  const pk = Math.max(1, metas.length);
  const pkPos = new Float32Array(pk * 3).fill(10000);
  const pkSize = new Float32Array(pk);
  const pkColor = new Float32Array(pk * 3);
  const pkGeo = new THREE.BufferGeometry();
  const pkPosAttr = new THREE.BufferAttribute(pkPos, 3);
  const pkSizeAttr = new THREE.BufferAttribute(pkSize, 1);
  pkPosAttr.setUsage(THREE.DynamicDrawUsage);
  pkSizeAttr.setUsage(THREE.DynamicDrawUsage);
  pkGeo.setAttribute('position', pkPosAttr);
  pkGeo.setAttribute('aSize', pkSizeAttr);
  pkGeo.setAttribute('aColor', new THREE.BufferAttribute(pkColor, 3));
  const pkMat = new THREE.ShaderMaterial({
    uniforms: { uTex: { value: glowTex } },
    vertexShader: PACKET_VERT,
    fragmentShader: PACKET_FRAG,
    transparent: true,
    blending: THREE.AdditiveBlending,
    depthWrite: false,
  });
  const packets = new THREE.Points(pkGeo, pkMat);
  packets.renderOrder = 5;
  packets.frustumCulled = false;

  const packetColor = theme.spawnBeam.clone().multiplyScalar(1.4);
  for (let i = 0; i < metas.length; i++) {
    pkColor[i * 3] = packetColor.r;
    pkColor[i * 3 + 1] = packetColor.g;
    pkColor[i * 3 + 2] = packetColor.b;
  }

  const group = new THREE.Group();
  group.add(treeL.obj, beamL.obj, packets);

  const wp: THREE.Vector3[] = [new THREE.Vector3(), new THREE.Vector3(), new THREE.Vector3(), new THREE.Vector3()];
  const seedOf = (id: string) => (fnvHash(id) % 1000) / 1000;

  /** waypoints for a meta from current positions; false if an end is hidden */
  function waypoints(meta: TraceMeta, getPos: (id: string) => THREE.Vector3 | null): boolean {
    const pa = getPos(meta.aId);
    const pb = getPos(meta.bId);
    if (pa === null || pb === null) return false;
    if (meta.kind === 'tree') {
      wp[0].set(pa.x, pa.y, pa.z);
      wp[1].set(meta.xFirst ? pb.x : pa.x, pa.y, meta.xFirst ? pa.z : pb.z);
      wp[2].set(pb.x, pb.y, pb.z);
    } else {
      wp[0].set(pa.x, pa.y, pa.z);
      wp[1].set(pa.x, pb.y, pa.z);
      wp[2].set(pb.x, pb.y, pa.z);
      wp[3].set(pb.x, pb.y, pb.z);
    }
    return true;
  }

  function fillGroup(
    list: TraceMeta[],
    fat: { obj: LineSegments2; geo: LineSegmentsGeometry },
    getPos: (id: string) => THREE.Vector3 | null,
    t: number,
    baseColor: THREE.Color,
    liveBoost: number,
    dashed: boolean,
  ): void {
    const segsPer = list.length > 0 ? PTS[list[0].kind] - 1 : 1;
    const n = Math.max(1, list.length * segsPer);
    const pos = new Float32Array(n * 6);
    const col = new Float32Array(n * 6);
    pos.fill(10000);
    let o = 0;
    for (const meta of list) {
      const nPts = PTS[meta.kind];
      if (!waypoints(meta, getPos)) {
        o += (nPts - 1) * 6;
        continue;
      }
      const lv = liveness(g, meta);
      const fade = Math.min(1, Math.max(0, (t - birthOf(meta.id)) * 1.4));
      const lum = fade * (1 + lv * liveBoost);
      for (let s = 0; s < nPts - 1; s++) {
        pos[o] = wp[s].x; pos[o + 1] = wp[s].y; pos[o + 2] = wp[s].z;
        pos[o + 3] = wp[s + 1].x; pos[o + 4] = wp[s + 1].y; pos[o + 5] = wp[s + 1].z;
        for (let k = 0; k < 2; k++) {
          col[o + k * 3] = baseColor.r * lum;
          col[o + k * 3 + 1] = baseColor.g * lum;
          col[o + k * 3 + 2] = baseColor.b * lum;
        }
        o += 6;
      }
    }
    fat.geo.setPositions(pos as unknown as number[]);
    fat.geo.setColors(col as unknown as number[]);
    if (dashed) fat.obj.computeLineDistances();
  }

  return {
    group,
    setResolution(w: number, h: number) {
      treeL.mat.resolution.set(w, h);
      beamL.mat.resolution.set(w, h);
    },
    update(getPos: (id: string) => THREE.Vector3 | null, t: number) {
      fillGroup(tree, treeL, getPos, t, theme.treeTrace, 0.6, true);
      fillGroup(spawn, beamL, getPos, t, theme.spawnBeam, 0.35, false);
      // the dash field flows — light streaming along the path
      treeL.mat.dashOffset = -t * 7;

      for (let i = 0; i < metas.length; i++) {
        const meta = metas[i];
        const lv = liveness(g, meta);
        const isBeam = meta.kind === 'spawn';
        if ((lv < 0.9 && !isBeam) || !waypoints(meta, getPos)) {
          pkPos[i * 3 + 1] = 10000;
          continue;
        }
        const nPts = PTS[meta.kind];
        let total = 0;
        const cum = [0];
        for (let s = 1; s < nPts; s++) {
          total += wp[s].distanceTo(wp[s - 1]);
          cum.push(total);
        }
        if (total < 0.01) {
          pkPos[i * 3 + 1] = 10000;
          continue;
        }
        const speed = isBeam ? 0.28 : 0.45;
        const head = ((t * speed + seedOf(meta.id)) % 1) * total;
        let s = 0;
        while (s < nPts - 2 && cum[s + 1] < head) s++;
        const f = (head - cum[s]) / Math.max(0.01, cum[s + 1] - cum[s]);
        pkPos[i * 3] = wp[s].x + (wp[s + 1].x - wp[s].x) * f;
        pkPos[i * 3 + 1] = wp[s].y + (wp[s + 1].y - wp[s].y) * f;
        pkPos[i * 3 + 2] = wp[s].z + (wp[s + 1].z - wp[s].z) * f;
        pkSize[i] = isBeam ? 7 : 5;
      }
      pkPosAttr.needsUpdate = true;
      pkSizeAttr.needsUpdate = true;
    },
    dispose() {
      treeL.geo.dispose();
      treeL.mat.dispose();
      beamL.geo.dispose();
      beamL.mat.dispose();
      pkGeo.dispose();
      pkMat.dispose();
    },
  };
}
