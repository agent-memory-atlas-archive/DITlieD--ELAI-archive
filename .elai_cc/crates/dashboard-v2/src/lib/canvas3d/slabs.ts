// canvas3d/slabs.ts — the translucent wireframe slabs (one per DEPTH PLANE).
//
// A slab is a Group of two objects: one LineSegments (frame + surface grid
// + a dense field of inner wireframe boxes at two sub-scales — the
// intricate interior of the reference image) and one additive sheet pair
// (top + mid) for volume. Geometry is deterministic per (depth, side) and
// rebuilt only when the square grows; the scene fades new slabs in.
// Brightness is baked as vertex colors; materials are additive.

import * as THREE from 'three';
import { fnvHash } from './types';
import { CELL, SLAB_THICK, layerTopY } from './layout';
import type { C3Theme } from './theme';

// DECOR stays BELOW the bloom threshold — structural lines must never
// compete with data traces, which live above it and glow (HDR split)
const FRAME_LEVEL = 0.85;
const GRID_LEVEL = 0.3;
const BOX_LEVEL = 0.55;
/** fraction of cells that grow an inner wireframe box */
const BOX_DENSITY = 0.7;
/** fraction of boxed cells subdivided into 4 mini-boxes (sub-scale detail) */
const SUBDIV_SHARE = 0.3;
/** fraction of boxed cells that run HOT (bright pop — the sparkle) */
const HOT_SHARE = 0.06;

function push(
  pos: number[],
  col: number[],
  c: THREE.Color,
  level: number,
  ax: number, ay: number, az: number,
  bx: number, by: number, bz: number,
): void {
  pos.push(ax, ay, az, bx, by, bz);
  col.push(c.r * level, c.g * level, c.b * level, c.r * level, c.g * level, c.b * level);
}

/** wireframe box: bottom rect + top rect + 4 verticals */
function pushBox(
  pos: number[], col: number[], c: THREE.Color, level: number,
  x0: number, x1: number, y0: number, y1: number, z0: number, z1: number,
): void {
  push(pos, col, c, level, x0, y0, z0, x1, y0, z0);
  push(pos, col, c, level, x1, y0, z0, x1, y0, z1);
  push(pos, col, c, level, x1, y0, z1, x0, y0, z1);
  push(pos, col, c, level, x0, y0, z1, x0, y0, z0);
  push(pos, col, c, level * 0.85, x0, y1, z0, x1, y1, z0);
  push(pos, col, c, level * 0.85, x1, y1, z1, x0, y1, z1);
  push(pos, col, c, level * 0.7, x0, y0, z0, x0, y1, z0);
  push(pos, col, c, level * 0.7, x1, y0, z1, x1, y1, z1);
}

export function buildSlab(depth: number, side: number, theme: C3Theme): THREE.Group {
  const yTop = layerTopY(depth);
  const yBot = yTop - SLAB_THICK;
  const world = side * CELL;
  const x0 = -world / 2;
  const z0 = -world / 2;
  const pos: number[] = [];
  const col: number[] = [];
  const faint = theme.gridFaint;
  const bright = theme.gridBright;

  // outer frame
  const corners: [number, number][] = [
    [x0, z0], [x0 + world, z0], [x0 + world, z0 + world], [x0, z0 + world],
  ];
  for (let i = 0; i < 4; i++) {
    const [ax, az] = corners[i];
    const [bx, bz] = corners[(i + 1) % 4];
    push(pos, col, bright, FRAME_LEVEL, ax, yTop, az, bx, yTop, bz);
    push(pos, col, bright, FRAME_LEVEL * 0.6, ax, yBot, az, bx, yBot, bz);
    push(pos, col, bright, FRAME_LEVEL * 0.75, ax, yTop, az, ax, yBot, az);
  }

  // top-surface grid
  for (let c = 1; c < side; c++) {
    const x = x0 + c * CELL;
    push(pos, col, faint, GRID_LEVEL, x, yTop, z0, x, yTop, z0 + world);
    push(pos, col, faint, GRID_LEVEL, x0, yTop, z0 + c * CELL, x0 + world, yTop, z0 + c * CELL);
  }

  // inner boxes — two sub-scales, deterministic per (depth, side)
  const seed = fnvHash(`slab:${depth}:${side}`);
  for (let r = 0; r < side; r++) {
    for (let c = 0; c < side; c++) {
      const h = fnvHash(`${seed}:${c}:${r}`);
      if ((h % 1000) / 1000 > BOX_DENSITY) continue;
      const inset = 1.3;
      const bx0 = x0 + c * CELL + inset;
      const bz0 = z0 + r * CELL + inset;
      const bx1 = x0 + (c + 1) * CELL - inset;
      const bz1 = z0 + (r + 1) * CELL - inset;
      const height = 2.5 + ((h >>> 10) % 100) / 100 * (SLAB_THICK - 4);
      const yTopBox = yTop - 1.2;
      const hot = ((h >>> 4) % 1000) / 1000 < HOT_SHARE;
      const lv = hot ? 1.5 : BOX_LEVEL * (0.5 + ((h >>> 20) % 100) / 100 * 0.9);
      if ((h >>> 8) % 1000 / 1000 < SUBDIV_SHARE) {
        // 2x2 mini-boxes with independent depths — the fine sub-scale
        const mx = (bx0 + bx1) / 2;
        const mz = (bz0 + bz1) / 2;
        const quads: [number, number, number, number][] = [
          [bx0, mx - 0.5, bz0, mz - 0.5], [mx + 0.5, bx1, bz0, mz - 0.5],
          [bx0, mx - 0.5, mz + 0.5, bz1], [mx + 0.5, bx1, mz + 0.5, bz1],
        ];
        for (let q = 0; q < 4; q++) {
          const hq = fnvHash(`${h}:${q}`);
          const hgt = 2 + (hq % 100) / 100 * (SLAB_THICK - 5);
          const [qx0, qx1, qz0, qz1] = quads[q];
          pushBox(pos, col, faint, lv * 0.8, qx0, qx1, yTopBox - hgt, yTopBox, qz0, qz1);
        }
      } else {
        pushBox(pos, col, faint, lv, bx0, bx1, yTopBox - height, yTopBox, bz0, bz1);
      }
    }
  }

  const geo = new THREE.BufferGeometry();
  geo.setAttribute('position', new THREE.Float32BufferAttribute(pos, 3));
  geo.setAttribute('color', new THREE.Float32BufferAttribute(col, 3));
  // ephemeral hologram, not a hard circuit board: the whole lattice is
  // ghosted through material opacity, on top of the per-vertex dimming
  const mat = new THREE.LineBasicMaterial({
    vertexColors: true,
    transparent: true,
    opacity: 0.45,
    blending: THREE.AdditiveBlending,
    depthWrite: false,
  });
  mat.userData.baseOpacity = 0.45;
  const lines = new THREE.LineSegments(geo, mat);
  lines.renderOrder = 1;

  // volume sheets: faint top sheet + a dimmer mid sheet for inner glow
  const group = new THREE.Group();
  group.add(lines);
  const sheetGeo = new THREE.PlaneGeometry(world, world);
  const mkSheet = (y: number, opacity: number) => {
    const m = new THREE.MeshBasicMaterial({
      color: theme.gridBright,
      transparent: true,
      opacity,
      blending: THREE.AdditiveBlending,
      depthWrite: false,
      side: THREE.DoubleSide,
    });
    m.userData.baseOpacity = opacity;
    const mesh = new THREE.Mesh(sheetGeo, m);
    mesh.rotation.x = -Math.PI / 2;
    mesh.position.y = y;
    mesh.renderOrder = 0;
    return mesh;
  };
  group.add(mkSheet(yTop - 0.4, 0.05));
  group.add(mkSheet(yTop - SLAB_THICK / 2, 0.03));
  return group;
}

/** collect every material in a slab group (fade-in support) */
export function slabMaterials(group: THREE.Group): THREE.Material[] {
  const mats: THREE.Material[] = [];
  group.traverse((o) => {
    const m = (o as THREE.Mesh).material as THREE.Material | undefined;
    if (m) mats.push(m);
  });
  return mats;
}

export function disposeSlab(group: THREE.Group): void {
  group.traverse((o) => {
    const mesh = o as THREE.Mesh;
    mesh.geometry?.dispose();
    (mesh.material as THREE.Material | undefined)?.dispose();
  });
}
