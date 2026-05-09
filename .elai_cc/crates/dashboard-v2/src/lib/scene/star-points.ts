// star-points.ts — builds a THREE.Points object from a StarField.
//
// One `THREE.Points` per layer. A custom ShaderMaterial drives twinkle
// on the GPU: the vertex shader reads per-vertex twinkle params (phase,
// period, amplitude) and modulates point alpha by `uTime`. Far stars get
// natural perspective-correct size attenuation via the standard
// `(scale / -mvPosition.z)` formula in the vertex shader.
//
// Bloom selectivity: stars stay LDR (RGB ≤ 1.0). The active "sun" object
// uses HDR (RGB > 1.0) so only the sun crosses the bloom threshold.

import * as THREE from 'three';
import type { StarField, StarSlot } from './star-state.ts';
import { configForLayer, LAYER_Z } from './star-state.ts';

// Re-export layer z-depths (defined in star-state.ts so star-shoot can
// use them without importing three.js).
export { LAYER_Z };


// Per-layer base size factor — gl_PointSize ≈ baseSize × scale / dist ×
// pixelRatio. Three perceptually distinct tiers at default (concept)
// zoom: far ~2 px clamp + low alpha (dense pinpoint dust), mid ~3 px,
// near ~7 px. The warp/zoom boost below pushes the far/mid tiers up
// during deeper zooms so each zoom level feels like a different
// "perspective" on what's close vs far.
export const LAYER_SIZE_SCALE = {
  far: 700,
  mid: 7000,
  near: 4500,
} as const;

// Per-layer perspective-boost multiplier. Multiplied by uPerspectiveBoost
// (0..1) and added to the per-frame size multiplier in the shader.
// Far gets the strongest amplification: at uPerspectiveBoost=1 (deep
// zoom or warp midpoint) far stars are 1+1.6 = 2.6× their default size,
// "approaching" the camera as the perceived perspective shifts. Near
// stays put — the user explicitly didn't want close stars wobbling.
export const LAYER_WARP_BOOST: Record<'far' | 'mid' | 'near', number> = {
  far:  1.6,
  mid:  0.6,
  near: 0.0,
};

const VERT = /* glsl */ `
  precision highp float;

  attribute float aBaseSize;
  attribute float aBaseAlpha;
  attribute float aTwinklePhase;
  attribute float aTwinklePeriod;   // ms
  attribute float aTwinkleAmp;
  attribute float aTwinkles;        // 0 or 1
  attribute vec3  aColor;
  attribute float aFadeIn;          // 0..1; from slot lifecycle
  attribute float aSizeScale;       // per-layer constant, packed per-vertex
  attribute float aBreathAsym;      // 0..1 per-star asymmetry seed

  uniform float uTimeMs;
  uniform float uReducedMotion;       // 1.0 = reduced
  uniform float uPixelRatio;
  uniform float uPerspectiveBoost;    // 0..1; max(warpProgress, zoomDepth)
  uniform float uLayerWarpBoost;      // per-layer perspective amp (far=1.6 etc.)

  varying float vAlpha;
  varying vec3  vColor;
  // Diffraction-spike outputs to fragment. vSpikeAmt drives spike
  // brightness (0 = no spike, 1 = full); vCoreScale tells the
  // fragment what fraction of the rasterized quad belongs to the
  // original soft-circle core (so legacy non-spike stars read
  // identically pixel-for-pixel).
  varying float vSpikeAmt;
  varying float vCoreScale;

  void main() {
    vec4 mv = modelViewMatrix * vec4(position, 1.0);
    gl_Position = projectionMatrix * mv;

    // Perspective-correct size attenuation. -mv.z is the camera-space
    // depth of the point (positive when in front of camera). We multiply
    // by uPixelRatio so high-DPI displays render the same physical size.
    float persp = aSizeScale / max(1.0, -mv.z);
    float baseSizePersp = aBaseSize * persp * uPixelRatio;

    // Breath: pure-sine alpha twinkle for small stars; bigger stars get
    // a half-frequency harmonic mixed in (asymmetric, irregular pulse).
    // bigStarAmp ∈ [0,1] from smoothstep over baseSize.
    float phase = uTimeMs * 6.283185307 / aTwinklePeriod + aTwinklePhase;
    float baseTw = sin(phase);
    float harm   = sin(phase * 0.5 + aBreathAsym * 6.283185307);
    float bigStarAmp = smoothstep(0.95, 1.6, aBaseSize);
    float breathEnv = mix(baseTw, baseTw * 0.55 + harm * 0.55, bigStarAmp);
    float twinkle = (1.0 - uReducedMotion) * aTwinkles * aTwinkleAmp * breathEnv;

    // Big stars also size-pulse (mini-sun breath visible in scale).
    float sizePulse = 1.0 + (1.0 - uReducedMotion) * bigStarAmp
                          * (baseTw * 0.10 + harm * 0.08);

    // Birth flash: fadeIn ∈ [0,1] during regen; envelope peaks at 0.5
    // and is exactly 0 at fadeIn=0 (departed) and fadeIn=1 (present),
    // so present stars are unaffected.
    float fade = clamp(aFadeIn, 0.0, 1.0);
    float flashEnv = fade * (1.0 - fade) * 4.0;
    float flashAlphaMul = 1.0 + flashEnv * 2.5;
    float flashSizeMul  = 1.0 + flashEnv * 0.5;

    // Sub-pixel anti-flicker. THREE.Points rasterizes at gl_PointSize
    // pixels; below ~1 px the covered-fragment count flips between 1
    // and 4 as the projected center crosses sub-pixel boundaries, which
    // is the visible "blink" of far stars during camera moves. Clamping
    // to a 2 px minimum forces consistent 4-fragment coverage across
    // sub-pixel shifts, and we pre-multiply alpha by computedSize/2 so
    // smaller stars stay proportionally dimmer instead of all popping
    // up to a uniform 2 px brightness.
    // Perspective shift: during a warp transition AND when the camera
    // sits at a deeper zoom level, multiply the size of distant layers
    // up so what was "far" feels like it has approached. Far stars get
    // the strongest amp (uLayerWarpBoost ~1.6); near stars are
    // unaffected (boost 0).
    float warpSizeMul = 1.0 + uPerspectiveBoost * uLayerWarpBoost;

    float computedSize = baseSizePersp * sizePulse * flashSizeMul * warpSizeMul;
    float MIN_PX = 2.0;
    float subPxAlpha = clamp(computedSize / MIN_PX, 0.0, 1.0);
    float clampedSize = max(MIN_PX, computedSize);

    // Diffraction-spike scaling. Threshold raised to (4.5, 8.0):
    // only the brightest near-layer stars (computedSize > 4.5 px)
    // start to gain a spike, full effect past 8 px. Reference mood
    // is "subtle starshine on the closest stars", not a forest of
    // crosses across the sky — old (2.5, 5.5) lit up too many
    // stars and read as CG-fake. Smaller quad multiplier (×4) also
    // keeps the spike length proportional to the underlying star.
    float spikeAmt = smoothstep(4.5, 8.0, clampedSize);
    float quadMul = 1.0 + spikeAmt * 4.0;
    gl_PointSize = clampedSize * quadMul;

    float a = clamp(aBaseAlpha + twinkle, 0.0, 1.0) * fade * flashAlphaMul * subPxAlpha;
    vAlpha = clamp(a, 0.0, 1.0);
    vColor = aColor;
    vSpikeAmt = spikeAmt;
    vCoreScale = 1.0 / quadMul;
  }
`;

const FRAG = /* glsl */ `
  precision highp float;
  varying float vAlpha;
  varying vec3  vColor;
  varying float vSpikeAmt;
  varying float vCoreScale;

  void main() {
    vec2 c = gl_PointCoord - vec2(0.5);
    float r = length(c);

    // Discard everything outside the inscribed circle of the quad.
    // This is the SAME for spike + non-spike stars — without it
    // the halo had non-zero contribution at the quad corners
    // (r=0.707), which ACES+sRGB amplified into a visible square
    // outline against pure black. Both halo and spike are designed
    // to have effectively-zero contribution by r=0.5, so we lose
    // nothing visible.
    if (r > 0.5) discard;

    // Re-scale into the original (un-enlarged) core coord domain.
    // For non-spike stars vCoreScale = 1 so coreR = r and behaviour
    // is identical to the legacy soft pinpoint.
    float coreR = r / max(vCoreScale, 1e-6);

    // Soft circular core (matches legacy pinpoint when spikeAmt = 0).
    float core = smoothstep(0.5, 0.0, coreR);

    // 4-spike diffraction cross with Gaussian along-axis profile
    // and exponential perpendicular thickness. Reference mood is
    // "soft starshine bloom with subtle rays", so the spike rays
    // are now the QUIETER element (×0.40) and the halo is the
    // dominant glow (×0.40, slower falloff). Old balance had rays
    // dominant — read as CG-fake on every bright star.
    float ax = abs(c.x);
    float ay = abs(c.y);
    float gx = exp(-ax * ax * 35.0);
    float gy = exp(-ay * ay * 35.0);
    float thx = exp(-ax * 60.0);
    float thy = exp(-ay * 60.0);
    float spikeH = gx * thy;
    float spikeV = gy * thx;
    float spike  = (spikeH + spikeV) * vSpikeAmt * 0.40;

    // Soft halo glow around bright cores. Quadratic falloff
    // pow(max(0, 1-2r), 2.5) is exactly 0 at r=0.5 (the quad
    // edge) so the halo never leaves a visible square outline —
    // unlike the previous exp(-r*4.5) which lingered at ~1.7%
    // brightness in the corners. Smooth fall through the disc
    // gives the same "starshine" mood without the bug.
    float haloFalloff = pow(max(0.0, 1.0 - 2.0 * r), 2.5);
    float halo = haloFalloff * vSpikeAmt * 0.40;

    float a = clamp(core + spike + halo, 0.0, 1.0);
    gl_FragColor = vec4(vColor, vAlpha * a);
  }
`;

// CSS color "rgb(R, G, B)" → THREE.Color (0..1 floats).
function parseCssColor(s: string): THREE.Color {
  const m = s.match(/rgb[a]?\(\s*(\d+)\s*,\s*(\d+)\s*,\s*(\d+)/i);
  if (!m) return new THREE.Color(0xe4eaf2);
  const r = parseInt(m[1], 10) / 255;
  const g = parseInt(m[2], 10) / 255;
  const b = parseInt(m[3], 10) / 255;
  return new THREE.Color(r, g, b);
}

export interface StarPointsLayer {
  layer: 'far' | 'mid' | 'near';
  points: THREE.Points;
  geometry: THREE.BufferGeometry;
  material: THREE.ShaderMaterial;
  // Per-slot lifecycle attribute writers — used when star-shoot mutates
  // slots (departed / fading-in).
  setFadeIn(slotId: number, value: number): void;
  setPosition(slotId: number, x: number, y: number, z: number): void;
  setBaseSize(slotId: number, size: number): void;
  setBaseAlpha(slotId: number, alpha: number): void;
}

export function buildStarPoints(
  layer: 'far' | 'mid' | 'near',
  slots: StarSlot[],
  colorMap: Record<string, string>,
  pixelRatio: number,
): StarPointsLayer {
  const count = slots.length;

  // World coordinate origin maps to slot.worldX/Y around the world center.
  // Convert to centered coords so (0,0,layerZ) is the layer's centre.
  // We reuse star-state's worldW/worldH per-layer config; simpler: assume
  // each layer's star-state world is centered on (worldW/2, worldH/2).
  // We center by subtracting cfg.worldW/2 etc. per slot.
  const positions = new Float32Array(count * 3);
  const baseSize = new Float32Array(count);
  const baseAlpha = new Float32Array(count);
  const twinklePhase = new Float32Array(count);
  const twinklePeriod = new Float32Array(count);
  const twinkleAmp = new Float32Array(count);
  const twinkles = new Float32Array(count);
  const colors = new Float32Array(count * 3);
  const fadeIn = new Float32Array(count);
  const sizeScale = new Float32Array(count);
  const breathAsym = new Float32Array(count);

  // Centre on the LAYER's actual world middle (not the slot bounding
  // box) so an inner exclusion zone doesn't bias the centroid. Stars'
  // positions stored in the geometry are world-coords relative to the
  // layer's centre — origin (0,0,layerZ) is exactly the centre.
  const cfg = configForLayer(layer);
  const cx = cfg.worldW / 2;
  const cy = cfg.worldH / 2;
  const z = LAYER_Z[layer];
  const sScale = LAYER_SIZE_SCALE[layer];

  for (let i = 0; i < count; i++) {
    const s = slots[i];
    positions[i * 3]     = s.worldX - cx;
    positions[i * 3 + 1] = -(s.worldY - cy); // flip Y so up is +Y in world
    positions[i * 3 + 2] = z;
    baseSize[i] = s.baseSize;
    baseAlpha[i] = s.baseAlpha;
    twinklePhase[i] = s.twinklePhase;
    twinklePeriod[i] = s.twinklePeriodMs;
    twinkleAmp[i] = s.twinkleAmp;
    twinkles[i] = s.twinkles ? 1 : 0;
    const col = parseCssColor(colorMap[s.color] || 'rgb(228,234,242)');
    colors[i * 3]     = col.r;
    colors[i * 3 + 1] = col.g;
    colors[i * 3 + 2] = col.b;
    fadeIn[i] = s.state === 'departed' ? 0 : (s.state === 'fading-in' ? s.fadeIn : 1);
    sizeScale[i] = sScale;
    breathAsym[i] = s.breathAsym;
  }

  const geo = new THREE.BufferGeometry();
  geo.setAttribute('position', new THREE.BufferAttribute(positions, 3));
  geo.setAttribute('aBaseSize', new THREE.BufferAttribute(baseSize, 1));
  geo.setAttribute('aBaseAlpha', new THREE.BufferAttribute(baseAlpha, 1));
  geo.setAttribute('aTwinklePhase', new THREE.BufferAttribute(twinklePhase, 1));
  geo.setAttribute('aTwinklePeriod', new THREE.BufferAttribute(twinklePeriod, 1));
  geo.setAttribute('aTwinkleAmp', new THREE.BufferAttribute(twinkleAmp, 1));
  geo.setAttribute('aTwinkles', new THREE.BufferAttribute(twinkles, 1));
  geo.setAttribute('aColor', new THREE.BufferAttribute(colors, 3));
  geo.setAttribute('aFadeIn', new THREE.BufferAttribute(fadeIn, 1));
  geo.setAttribute('aSizeScale', new THREE.BufferAttribute(sizeScale, 1));
  geo.setAttribute('aBreathAsym', new THREE.BufferAttribute(breathAsym, 1));

  const mat = new THREE.ShaderMaterial({
    vertexShader: VERT,
    fragmentShader: FRAG,
    transparent: true,
    depthWrite: false,
    blending: THREE.AdditiveBlending,
    uniforms: {
      uTimeMs: { value: 0 },
      uReducedMotion: { value: 0 },
      uPixelRatio: { value: pixelRatio },
      uPerspectiveBoost: { value: 0 },
      uLayerWarpBoost: { value: LAYER_WARP_BOOST[layer] },
    },
  });

  const points = new THREE.Points(geo, mat);
  // Stars are at infinity-ish; never frustum-cull them out unexpectedly.
  points.frustumCulled = false;

  const posAttr = geo.getAttribute('position') as THREE.BufferAttribute;
  const fadeAttr = geo.getAttribute('aFadeIn') as THREE.BufferAttribute;
  const baseSizeAttr = geo.getAttribute('aBaseSize') as THREE.BufferAttribute;
  const baseAlphaAttr = geo.getAttribute('aBaseAlpha') as THREE.BufferAttribute;

  return {
    layer,
    points,
    geometry: geo,
    material: mat,
    setFadeIn(slotId, value) {
      if (slotId < 0 || slotId >= count) return;
      fadeAttr.array[slotId] = value;
      fadeAttr.needsUpdate = true;
    },
    setPosition(slotId, x, y, _z) {
      if (slotId < 0 || slotId >= count) return;
      posAttr.array[slotId * 3]     = x - cx;
      posAttr.array[slotId * 3 + 1] = -(y - cy);
      posAttr.array[slotId * 3 + 2] = z;
      posAttr.needsUpdate = true;
    },
    setBaseSize(slotId, size) {
      if (slotId < 0 || slotId >= count) return;
      baseSizeAttr.array[slotId] = size;
      baseSizeAttr.needsUpdate = true;
    },
    setBaseAlpha(slotId, alpha) {
      if (slotId < 0 || slotId >= count) return;
      baseAlphaAttr.array[slotId] = alpha;
      baseAlphaAttr.needsUpdate = true;
    },
  };
}

export function buildAllStarLayers(
  field: StarField,
  colorMap: Record<string, string>,
  pixelRatio: number,
): StarPointsLayer[] {
  return [
    buildStarPoints('far', field.far, colorMap, pixelRatio),
    buildStarPoints('mid', field.mid, colorMap, pixelRatio),
    buildStarPoints('near', field.near, colorMap, pixelRatio),
  ];
}
