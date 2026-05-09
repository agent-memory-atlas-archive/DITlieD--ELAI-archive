// orbit/zoom.ts
// 4-level zoom state machine + linear/cubic interpolation between states.
// Levels are nested: concept (overview) -> phase -> step -> baby-step (closeup).
// The current zoom level controls which orbit ring is "centered" + how
// much of the SVG viewBox is visible.

import type { ZoomState, ZoomTarget, ZoomLevel } from './types';

export const ZOOM_LEVELS: ReadonlyArray<ZoomLevel> = Object.freeze([
  'concept',
  'phase',
  'step',
  'baby-step',
]);

// Default canvas centred on (CX=320, CY=280); same constants used in
// OrbitMode1Plan.svelte.
const DEFAULT_CX = 320;
const DEFAULT_CY = 280;

export function defaultZoomState(): ZoomState {
  return { level: 'concept', cx: DEFAULT_CX, cy: DEFAULT_CY, scale: 1 };
}

export function zoomTo(_prev: ZoomState, target: ZoomTarget): ZoomState {
  return { level: target.level, cx: target.x, cy: target.y, scale: target.scale };
}

// Linear interpolation between two zoom states; used for smooth transitions.
// t in [0, 1]. Caller should pass t = easeOutCubic(rawT) for the premium feel.
export function interpolateZoom(a: ZoomState, b: ZoomState, t: number): ZoomState {
  const tt = Math.max(0, Math.min(1, t));
  // level snaps at midpoint to avoid label flicker mid-transition.
  const level: ZoomLevel = tt < 0.5 ? a.level : b.level;
  return {
    level,
    cx: a.cx + (b.cx - a.cx) * tt,
    cy: a.cy + (b.cy - a.cy) * tt,
    scale: a.scale + (b.scale - a.scale) * tt,
  };
}

// Convenience: convert a zoom state to an SVG viewBox string.
// Base canvas is 640x540 (matches existing OrbitMode1Plan canvas).
export function zoomToViewBox(z: ZoomState, baseW = 640, baseH = 540): string {
  const w = baseW / z.scale;
  const h = baseH / z.scale;
  const x = z.cx - w / 2;
  const y = z.cy - h / 2;
  return `${x} ${y} ${w} ${h}`;
}

// Each level has a default scale: more zoomed-in for deeper levels.
export function defaultScaleForLevel(level: ZoomLevel): number {
  switch (level) {
    case 'concept':   return 1.0;
    case 'phase':     return 1.6;
    case 'step':      return 2.4;
    case 'baby-step': return 3.4;
  }
}

// Step deeper / shallower in the zoom hierarchy.
export function zoomDeeper(z: ZoomState): ZoomLevel {
  const i = ZOOM_LEVELS.indexOf(z.level);
  return ZOOM_LEVELS[Math.min(i + 1, ZOOM_LEVELS.length - 1)];
}

export function zoomShallower(z: ZoomState): ZoomLevel {
  const i = ZOOM_LEVELS.indexOf(z.level);
  return ZOOM_LEVELS[Math.max(i - 1, 0)];
}
