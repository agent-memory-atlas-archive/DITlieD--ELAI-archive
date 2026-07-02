// canvas3d/theme.ts — ELAI palette for the 3D view.
//
// PINNED to the DESIGN.md §3 ramp (workflow/design/DESIGN.md, v1.5) — ONE
// teal family on near-black #0a0f0e, red/amber BANNED as status colors.
// State language per §6: done = teal-deep, active = teal-bright pulsing,
// pending = dim, warn = white whisper, fail = MONO INVERSION white-hot.
//
// HDR: the scene renders through an UnrealBloom pass. Anything whose
// color exceeds the bloom threshold GLOWS FROM ITSELF — node cores,
// beams, packets. Slab decor sits far below the threshold so structure
// reads as a faint hologram, never competing with data. This module is
// the single color source for the 3D scene (three.js needs numerics;
// dashboard tokens.css still carries the pre-DESIGN.md ramp, so we do
// not read it).

import * as THREE from 'three';
import type { QuinaryState } from '../canvas/canvas_state';

// DESIGN.md §3 anchors
const BG = '#0a0f0e';
const TEAL = '#2BBBA4';
const TEAL_BRIGHT = '#34C2AA';
const TEAL_DEEP = '#1B7A6E';
const TEXT = '#f2f5f4';
const DIM = '#79827e';

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
  const teal = new THREE.Color(TEAL);
  const tealBright = new THREE.Color(TEAL_BRIGHT);
  const tealDeep = new THREE.Color(TEAL_DEEP);
  const text = new THREE.Color(TEXT);
  return {
    background: new THREE.Color(BG),
    gridFaint: teal.clone().multiplyScalar(0.72),
    gridBright: tealBright.clone(),
    stateColors: {
      done: tealDeep.clone().multiplyScalar(1.6),
      active: tealBright.clone().multiplyScalar(2.3), // hot emissive, pulses
      pending: new THREE.Color(DIM).multiplyScalar(0.75),
      warn: text.clone().multiplyScalar(1.05),
      fail: text.clone().multiplyScalar(2.6), // mono inversion: blooms white
    },
    spawnBeam: tealBright.clone().lerp(text, 0.45).multiplyScalar(1.8),
    treeTrace: teal.clone().multiplyScalar(1.1),
    selection: tealBright.clone().multiplyScalar(2.4),
  };
}

/** Shared radial-falloff sprite texture (dust, packets, atmosphere). */
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
