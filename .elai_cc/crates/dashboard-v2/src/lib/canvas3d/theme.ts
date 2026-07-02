// canvas3d/theme.ts — ELAI palette for the 3D view.
//
// three.js needs numeric colors, so we read the design tokens off :root at
// runtime (tokens.css stays the single source of truth) and fall back to
// the token values verbatim when running headless / before styles mount.
// Fallback hexes mirror tokens.css and must not drift from it.

import * as THREE from 'three';
import type { QuinaryState } from '../canvas/canvas_state';

const FALLBACK: Record<string, string> = {
  '--bg-void': '#0a0e12',
  '--teal': '#1B7A6E',
  '--teal-bright': '#24a08e',
  '--steel': '#5A7A9E',
  '--purple': '#7C5CBF',
  '--warn': '#c49b2a',
  '--fail': '#c44040',
  '--text-primary': '#d0d7de',
  '--text-secondary': '#6e7681',
};

function cssColor(token: string): THREE.Color {
  let raw = '';
  if (typeof document !== 'undefined') {
    raw = getComputedStyle(document.documentElement).getPropertyValue(token).trim();
  }
  return new THREE.Color(raw !== '' ? raw : FALLBACK[token]);
}

export interface C3Theme {
  background: THREE.Color;
  gridFaint: THREE.Color;
  gridBright: THREE.Color;
  stateColors: Record<QuinaryState, THREE.Color>;
  spawnBeam: THREE.Color;
  treeTrace: THREE.Color;
  selection: THREE.Color;
}

export function readTheme(): C3Theme {
  const teal = cssColor('--teal');
  const tealBright = cssColor('--teal-bright');
  const steel = cssColor('--steel');
  const purple = cssColor('--purple');
  const warn = cssColor('--warn');
  const fail = cssColor('--fail');
  return {
    background: cssColor('--bg-void').multiplyScalar(0.55),
    gridFaint: teal.clone().multiplyScalar(0.4),
    gridBright: tealBright.clone(),
    stateColors: {
      done: teal.clone(),
      active: tealBright.clone().multiplyScalar(1.35),
      pending: steel.clone().multiplyScalar(0.65),
      warn: warn.clone(),
      fail: fail.clone(),
    },
    spawnBeam: tealBright.clone().multiplyScalar(1.2),
    treeTrace: teal.clone(),
    selection: purple.clone(),
  };
}

/** Shared radial-falloff sprite texture for glow points (drawn once). */
export function makeGlowTexture(): THREE.Texture {
  const size = 64;
  const cnv = document.createElement('canvas');
  cnv.width = size;
  cnv.height = size;
  const ctx = cnv.getContext('2d') as CanvasRenderingContext2D;
  const grad = ctx.createRadialGradient(size / 2, size / 2, 0, size / 2, size / 2, size / 2);
  grad.addColorStop(0.0, 'rgba(255,255,255,1)');
  grad.addColorStop(0.18, 'rgba(255,255,255,0.85)');
  grad.addColorStop(0.45, 'rgba(255,255,255,0.22)');
  grad.addColorStop(1.0, 'rgba(255,255,255,0)');
  ctx.fillStyle = grad;
  ctx.fillRect(0, 0, size, size);
  const tex = new THREE.CanvasTexture(cnv);
  tex.needsUpdate = true;
  return tex;
}
