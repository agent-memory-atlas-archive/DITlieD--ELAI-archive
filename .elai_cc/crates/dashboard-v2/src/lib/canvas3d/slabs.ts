// canvas3d/slabs.ts — the translucent wireframe slabs (one per layer).
//
// Each slab is a SINGLE LineSegments draw call: bright outer frame, faint
// top-surface grid, and a deterministic ~1/3 subset of cells extruded as
// tiny wireframe boxes of varying height — that dense inner-structure
// look from the reference image. Geometry is static per layer depth
// (nodes/edges live in their own buffers), built once and cached.
// Brightness is baked as vertex colors; one additive material serves all.

import * as THREE from 'three';
import { fnvHash } from './types';
import { COLS, ROWS, CELL, SLAB_W, SLAB_D, SLAB_THICK, layerTopY } from './layout';
import type { C3Theme } from './theme';

// levels compound with theme.gridFaint's own 0.4 damping — keep them high
const FRAME_LEVEL = 0.85;
const GRID_LEVEL = 0.34;
const BOX_LEVEL = 0.72;
/** fraction of cells that grow an inner wireframe box */
const BOX_DENSITY = 0.34;

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

export function buildSlab(depth: number, layerId: string, theme: C3Theme): THREE.LineSegments {
  const yTop = layerTopY(depth);
  const yBot = yTop - SLAB_THICK;
  const x0 = -SLAB_W / 2;
  const z0 = -SLAB_D / 2;
  const pos: number[] = [];
  const col: number[] = [];
  const faint = theme.gridFaint;
  const bright = theme.gridBright;

  // outer frame: top rect, bottom rect, four verticals
  const corners: [number, number][] = [
    [x0, z0], [x0 + SLAB_W, z0], [x0 + SLAB_W, z0 + SLAB_D], [x0, z0 + SLAB_D],
  ];
  for (let i = 0; i < 4; i++) {
    const [ax, az] = corners[i];
    const [bx, bz] = corners[(i + 1) % 4];
    push(pos, col, bright, FRAME_LEVEL, ax, yTop, az, bx, yTop, bz);
    push(pos, col, bright, FRAME_LEVEL * 0.55, ax, yBot, az, bx, yBot, bz);
    push(pos, col, bright, FRAME_LEVEL * 0.7, ax, yTop, az, ax, yBot, az);
  }

  // top-surface grid
  for (let c = 1; c < COLS; c++) {
    const x = x0 + c * CELL;
    push(pos, col, faint, GRID_LEVEL, x, yTop, z0, x, yTop, z0 + SLAB_D);
  }
  for (let r = 1; r < ROWS; r++) {
    const z = z0 + r * CELL;
    push(pos, col, faint, GRID_LEVEL, x0, yTop, z, x0 + SLAB_W, yTop, z);
  }

  // inner cell boxes — deterministic subset, height varies with hash
  const layerSeed = fnvHash(layerId);
  for (let r = 0; r < ROWS; r++) {
    for (let c = 0; c < COLS; c++) {
      const h = fnvHash(`${layerSeed}:${c}:${r}`);
      if ((h % 1000) / 1000 > BOX_DENSITY) continue;
      const inset = 1.6;
      const bx0 = x0 + c * CELL + inset;
      const bz0 = z0 + r * CELL + inset;
      const bx1 = x0 + (c + 1) * CELL - inset;
      const bz1 = z0 + (r + 1) * CELL - inset;
      const height = 1.5 + ((h >>> 10) % 100) / 100 * (SLAB_THICK - 2.5);
      const by = yTop - height;
      const lv = BOX_LEVEL * (0.6 + ((h >>> 20) % 100) / 100 * 0.8);
      // rect at box depth + four verticals up to the top surface
      push(pos, col, faint, lv, bx0, by, bz0, bx1, by, bz0);
      push(pos, col, faint, lv, bx1, by, bz0, bx1, by, bz1);
      push(pos, col, faint, lv, bx1, by, bz1, bx0, by, bz1);
      push(pos, col, faint, lv, bx0, by, bz1, bx0, by, bz0);
      push(pos, col, faint, lv * 0.8, bx0, by, bz0, bx0, yTop, bz0);
      push(pos, col, faint, lv * 0.8, bx1, by, bz1, bx1, yTop, bz1);
    }
  }

  const geo = new THREE.BufferGeometry();
  geo.setAttribute('position', new THREE.Float32BufferAttribute(pos, 3));
  geo.setAttribute('color', new THREE.Float32BufferAttribute(col, 3));
  const mat = new THREE.LineBasicMaterial({
    vertexColors: true,
    transparent: true,
    opacity: 1,
    blending: THREE.AdditiveBlending,
    depthWrite: false,
  });
  const lines = new THREE.LineSegments(geo, mat);
  lines.renderOrder = 1;
  return lines;
}

/** Faint translucent sheet on the slab top — gives the glassy plane the
 *  wireframe alone lacks. One plane per layer. */
export function buildSlabSheet(depth: number, theme: C3Theme): THREE.Mesh {
  const geo = new THREE.PlaneGeometry(SLAB_W, SLAB_D);
  const mat = new THREE.MeshBasicMaterial({
    color: theme.gridBright,
    transparent: true,
    opacity: 0.028,
    blending: THREE.AdditiveBlending,
    depthWrite: false,
    side: THREE.DoubleSide,
  });
  const mesh = new THREE.Mesh(geo, mat);
  mesh.rotation.x = -Math.PI / 2;
  mesh.position.y = layerTopY(depth) - SLAB_THICK / 2;
  mesh.renderOrder = 0;
  return mesh;
}
