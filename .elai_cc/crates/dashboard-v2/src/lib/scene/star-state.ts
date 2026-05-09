// star-state.ts — pure-TS deterministic star-field model.
//
// Three layers (far / mid / near) of background stars, plus a deep-space
// glow. Stars live in WORLD coordinates, larger than the viewport, so the
// camera can pan and zoom over them. Layer parallax multipliers project
// world to screen.
//
// Determinism: layout is seeded from a single constant (default 0xC0FFEE,
// matching the legacy SVG Starfield) and is identical across reloads.
// Twinkle phase is also seeded — same star always twinkles in phase. Only
// shoot scheduling and per-frame timestamps are non-deterministic by
// design (perceptual, not state).
//
// No DOM, no Svelte. This module is testable under `node --test`.

export type StarLayer = 'far' | 'mid' | 'near';

// Layer z-depths in world units. Far stars sit deep; near stars closer to
// the camera (camera defaults to z ≈ +400 in StarCanvas). Lives here (not
// star-points.ts) so star-shoot.ts can read it without pulling three.js.
export const LAYER_Z: Record<StarLayer, number> = {
  far: -7000,
  mid: -2400,
  near: -700,
};

// Star color tokens; the Canvas resolves these via getComputedStyle so all
// hex literals stay in tokens.css.
export const STAR_COLOR_VARS = [
  '--star-cool-white',
  '--star-pale-steel',
  '--star-pale-teal',
  '--star-warm-amber',
  '--star-pale-pink',
] as const;
export type StarColorVar = (typeof STAR_COLOR_VARS)[number];

// Mixed palette weights — sum = 1.0. Matches the legacy Starfield mixed tone.
export const STAR_COLOR_WEIGHTS_MIXED: Record<StarColorVar, number> = {
  '--star-cool-white': 0.62,
  '--star-pale-steel': 0.18,
  '--star-pale-teal':  0.12,
  '--star-warm-amber': 0.05,
  '--star-pale-pink':  0.03,
};
export const STAR_COLOR_WEIGHTS_TEAL: Record<StarColorVar, number> = {
  '--star-cool-white': 0.55,
  '--star-pale-steel': 0.05,
  '--star-pale-teal':  0.30,
  '--star-warm-amber': 0.05,
  '--star-pale-pink':  0.05,
};
export const STAR_COLOR_WEIGHTS_STEEL: Record<StarColorVar, number> = {
  '--star-cool-white': 0.60,
  '--star-pale-steel': 0.30,
  '--star-pale-teal':  0.05,
  '--star-warm-amber': 0.02,
  '--star-pale-pink':  0.03,
};

export type Tone = 'mixed' | 'teal' | 'steel';
export type Density = 'sparse' | 'normal' | 'dense';

// One slot in a layer. Slot id is stable across regens; when a shooter
// "uses" a star, the slot transitions present -> departed -> fading-in
// -> present (at a new position drawn from the layer's regen RNG).
export interface StarSlot {
  slotId: number;
  layer: StarLayer;

  // World-space position. Re-rolled on regen; otherwise constant.
  worldX: number;
  worldY: number;

  // Visual base properties — constant for the lifetime of this position.
  baseSize: number;     // pixels, log-normal distributed within layer min/max
  baseAlpha: number;    // 0..1 base brightness
  color: StarColorVar;  // CSS variable name

  // Twinkle (alpha modulation). Stars without twinkle hold baseAlpha.
  twinkles: boolean;
  twinklePhase: number;     // radians; per-star fixed offset
  twinklePeriodMs: number;  // 2000..9000ms
  twinkleAmp: number;       // 0..0.5 alpha amplitude
  // Per-star asymmetry factor (0..1). Drives a half-frequency harmonic in
  // the shader breath so bigger stars get an irregular, non-pure-sine
  // pulse — sometimes brightening, sometimes fading.
  breathAsym: number;

  // Lifecycle (mutated by star-shoot regen).
  state: 'present' | 'departed' | 'fading-in';
  // 0..1; only used when state = 'fading-in' (1 = fully present).
  fadeIn: number;
}

export interface LayerConfig {
  count: number;          // stars allocated for this layer
  worldW: number;         // world width (px)
  worldH: number;         // world height (px)
  parallax: number;       // 0..1; G2 wires this into camera projection
  // log-normal size sampling: result = clamp(exp(mu + sigma*Z), min, max)
  sizeMu: number;
  sizeSigma: number;
  sizeMin: number;
  sizeMax: number;
  alphaMin: number;
  alphaMax: number;
  twinkleProb: number;    // fraction of stars that twinkle
  twinkleAmpMax: number;  // 0..0.5
  /** Inner exclusion zone radius (world units). Density falls smoothly
   *  to ~0 at r=0 via quadratic acceptance probability so the area
   *  immediately around the central "sun" is sparse. Tuned per layer so
   *  far stars retreat the most around the bright centre. */
  innerExclusionR: number;
}

// Layer worlds are sized to the layer's visible footprint at its z-depth
// (computed from camera fov × distance). World oversample 1.2× gives
// ~20 % off-screen margin for camera pan and shooter spawning.
//
// Visible footprint at z = 2 × distance × tan(fov/2), with aspect from
// the panel. Camera at z=+1200 / fov 55°: visible heights at
//   far  z=-7000 (dist 8200): ~8542 world units
//   mid  z=-2400 (dist 3600): ~3748
//   near z= -700 (dist 1900): ~1978
// World widths use a generous 4:3-ish aspect so off-screen stars exist
// regardless of panel shape. Counts are tuned so visible-in-viewport
// holds the per-layer targets at default camera.
//
// Inner exclusion zones (innerExclusionR) keep the area immediately
// around the world centre (the sun) sparse — density rises from 0 at
// r=0 to full at r=innerExclusionR via quadratic acceptance.
const FAR_VISIBLE = 180;
const MID_VISIBLE = 60;
const NEAR_VISIBLE = 18;

export const FAR_CONFIG: LayerConfig = {
  count: 520,
  worldW: 12000,
  worldH: 10250,
  parallax: 0.05,
  sizeMu: -0.20,
  sizeSigma: 0.28,
  sizeMin: 0.50,
  sizeMax: 0.95,
  // Bumped from 0.18/0.55: far stars get a 2 px clamp + linear
  // alpha-pre-multiply (~0.05× attenuation), so the raw alpha needs
  // to be higher for the dense pinpoint dust to stay visible.
  alphaMin: 0.32,
  alphaMax: 0.78,
  twinkleProb: 0.12,
  twinkleAmpMax: 0.08,
  innerExclusionR: 1500,
};

export const MID_CONFIG: LayerConfig = {
  count: 105,
  worldW: 5260,
  worldH: 4500,
  parallax: 0.20,
  sizeMu: 0.05,
  sizeSigma: 0.28,
  sizeMin: 0.75,
  sizeMax: 1.50,
  alphaMin: 0.28,
  alphaMax: 0.70,
  twinkleProb: 0.22,
  twinkleAmpMax: 0.14,
  innerExclusionR: 500,
};

export const NEAR_CONFIG: LayerConfig = {
  count: 32,
  worldW: 2780,
  worldH: 2375,
  parallax: 0.45,
  sizeMu: 0.30,
  sizeSigma: 0.32,
  sizeMin: 0.95,
  sizeMax: 2.20,
  alphaMin: 0.45,
  alphaMax: 0.95,
  twinkleProb: 0.35,
  twinkleAmpMax: 0.22,
  innerExclusionR: 200,
};

// Convenience for tests / consumers expecting visible-target counts.
export const VISIBLE_TARGET = {
  far: FAR_VISIBLE,
  mid: MID_VISIBLE,
  near: NEAR_VISIBLE,
} as const;

// Density scaling: caller can request sparse/dense and we just multiply.
export function densityScale(d: Density): number {
  return d === 'sparse' ? 0.55 : d === 'dense' ? 1.45 : 1.0;
}

// ----------------------------------------------------------------------
// Deterministic RNG — mulberry32-style LCG; matches src/lib/orbit/lcg.ts.
// ----------------------------------------------------------------------
export interface Rng {
  next(): number;
  // Box-Muller standard-normal sample (caches the second draw).
  nextNormal(): number;
}

export function makeRng(seed: number): Rng {
  let s = seed >>> 0;
  let cached: number | null = null;
  const nextU = () => {
    s = (Math.imul(1664525, s) + 1013904223) >>> 0;
    return s / 4294967296;
  };
  return {
    next: nextU,
    nextNormal() {
      if (cached !== null) {
        const v = cached;
        cached = null;
        return v;
      }
      // Avoid log(0).
      let u = nextU(); if (u < 1e-12) u = 1e-12;
      let v = nextU(); if (v < 1e-12) v = 1e-12;
      const mag = Math.sqrt(-2 * Math.log(u));
      const z0 = mag * Math.cos(2 * Math.PI * v);
      const z1 = mag * Math.sin(2 * Math.PI * v);
      cached = z1;
      return z0;
    },
  };
}

function clamp(x: number, lo: number, hi: number): number {
  return x < lo ? lo : x > hi ? hi : x;
}

function logNormalSize(rng: Rng, cfg: LayerConfig): number {
  const z = rng.nextNormal();
  const x = Math.exp(cfg.sizeMu + cfg.sizeSigma * z);
  return clamp(x, cfg.sizeMin, cfg.sizeMax);
}

function pickColor(rng: Rng, weights: Record<StarColorVar, number>): StarColorVar {
  const r = rng.next();
  let acc = 0;
  for (const v of STAR_COLOR_VARS) {
    acc += weights[v];
    if (r <= acc) return v;
  }
  return STAR_COLOR_VARS[0];
}

function weightsForTone(t: Tone): Record<StarColorVar, number> {
  return t === 'teal' ? STAR_COLOR_WEIGHTS_TEAL
       : t === 'steel' ? STAR_COLOR_WEIGHTS_STEEL
       : STAR_COLOR_WEIGHTS_MIXED;
}

// ----------------------------------------------------------------------
// Layer construction — deterministic; order of RNG draws is fixed so the
// hash of the produced array is stable across builds.
// ----------------------------------------------------------------------
export interface BuildLayerOptions {
  seed: number;
  layer: StarLayer;
  config: LayerConfig;
  tone: Tone;
  density: Density;
}

export function buildLayer(opts: BuildLayerOptions): StarSlot[] {
  const rng = makeRng(opts.seed);
  const weights = weightsForTone(opts.tone);
  const scale = densityScale(opts.density);
  const count = Math.max(1, Math.round(opts.config.count * scale));
  const slots: StarSlot[] = new Array(count);
  const cx = opts.config.worldW / 2;
  const cy = opts.config.worldH / 2;
  const exclusionR = Math.max(0, opts.config.innerExclusionR);
  const exclusionR2 = exclusionR * exclusionR;
  for (let i = 0; i < count; i++) {
    let worldX = rng.next() * opts.config.worldW;
    let worldY = rng.next() * opts.config.worldH;
    // Quadratic radial-density falloff at the centre. Stars closer to
    // the centre are accepted with probability (r/R)^2; further out,
    // accepted unconditionally. Bounded retry budget preserves
    // determinism in RNG-draw count per slot.
    if (exclusionR > 0) {
      for (let retry = 0; retry < 5; retry++) {
        const dx = worldX - cx;
        const dy = worldY - cy;
        const r2 = dx * dx + dy * dy;
        if (r2 >= exclusionR2) break;
        const accept = r2 / exclusionR2;
        if (rng.next() < accept) break;
        worldX = rng.next() * opts.config.worldW;
        worldY = rng.next() * opts.config.worldH;
      }
    }
    const baseSize = logNormalSize(rng, opts.config);
    const baseAlpha = opts.config.alphaMin + rng.next() * (opts.config.alphaMax - opts.config.alphaMin);
    const color = pickColor(rng, weights);
    const twinkles = rng.next() < opts.config.twinkleProb;
    const twinklePhase = rng.next() * Math.PI * 2;
    const twinklePeriodMs = 2000 + rng.next() * 7000;
    const twinkleAmp = opts.config.twinkleAmpMax * (0.4 + rng.next() * 0.6);
    const breathAsym = rng.next();
    slots[i] = {
      slotId: i,
      layer: opts.layer,
      worldX,
      worldY,
      baseSize,
      baseAlpha,
      color,
      twinkles,
      twinklePhase,
      twinklePeriodMs,
      twinkleAmp,
      breathAsym,
      state: 'present',
      fadeIn: 1,
    };
  }
  return slots;
}

export interface StarField {
  far: StarSlot[];
  mid: StarSlot[];
  near: StarSlot[];
  // World dims (max of the three layer worlds — used by the camera as
  // outer bounds).
  worldW: number;
  worldH: number;
}

export const DEFAULT_SEED = 0xC0FFEE;

export function buildStarField(seed: number = DEFAULT_SEED, tone: Tone = 'mixed', density: Density = 'normal'): StarField {
  const far = buildLayer({ seed: (seed ^ 0x1010) >>> 0, layer: 'far', config: FAR_CONFIG, tone, density });
  const mid = buildLayer({ seed: (seed ^ 0x2020) >>> 0, layer: 'mid', config: MID_CONFIG, tone, density });
  const near = buildLayer({ seed: (seed ^ 0x3030) >>> 0, layer: 'near', config: NEAR_CONFIG, tone, density });
  return {
    far,
    mid,
    near,
    worldW: Math.max(FAR_CONFIG.worldW, MID_CONFIG.worldW, NEAR_CONFIG.worldW),
    worldH: Math.max(FAR_CONFIG.worldH, MID_CONFIG.worldH, NEAR_CONFIG.worldH),
  };
}

// ----------------------------------------------------------------------
// Slot regeneration — used by star-shoot when a shooter's lifetime ends.
// Advances the layer's regen RNG so successive regens produce a stable
// stream of new positions for that layer.
// ----------------------------------------------------------------------
export interface RegenRngs {
  far: Rng;
  mid: Rng;
  near: Rng;
}

export function makeRegenRngs(seed: number = DEFAULT_SEED): RegenRngs {
  return {
    far:  makeRng((seed ^ 0xA1A1) >>> 0),
    mid:  makeRng((seed ^ 0xB2B2) >>> 0),
    near: makeRng((seed ^ 0xC3C3) >>> 0),
  };
}

export function regenSlot(slot: StarSlot, rng: Rng, cfg: LayerConfig, tone: Tone): void {
  slot.worldX = rng.next() * cfg.worldW;
  slot.worldY = rng.next() * cfg.worldH;
  slot.baseSize = logNormalSize(rng, cfg);
  slot.baseAlpha = cfg.alphaMin + rng.next() * (cfg.alphaMax - cfg.alphaMin);
  slot.color = pickColor(rng, weightsForTone(tone));
  slot.twinkles = rng.next() < cfg.twinkleProb;
  slot.twinklePhase = rng.next() * Math.PI * 2;
  slot.twinklePeriodMs = 2000 + rng.next() * 7000;
  slot.twinkleAmp = cfg.twinkleAmpMax * (0.4 + rng.next() * 0.6);
  slot.breathAsym = rng.next();
  slot.state = 'fading-in';
  slot.fadeIn = 0;
}

export function configForLayer(layer: StarLayer): LayerConfig {
  return layer === 'far' ? FAR_CONFIG : layer === 'mid' ? MID_CONFIG : NEAR_CONFIG;
}

// ----------------------------------------------------------------------
// Per-frame alpha — pure function used by both renderer and tests.
// Returns alpha in 0..1 honouring twinkle (only when not reduced) and
// fade-in/fade-out from regen.
// ----------------------------------------------------------------------
export function effectiveAlpha(slot: StarSlot, nowMs: number, reducedMotion: boolean): number {
  if (slot.state === 'departed') return 0;
  let a = slot.baseAlpha;
  if (slot.twinkles && !reducedMotion) {
    const t = (nowMs / slot.twinklePeriodMs) * Math.PI * 2 + slot.twinklePhase;
    // Sinusoidal modulation, never below 0.
    a = Math.max(0, a + slot.twinkleAmp * Math.sin(t));
  }
  if (slot.state === 'fading-in') {
    a *= clamp(slot.fadeIn, 0, 1);
  }
  return clamp(a, 0, 1);
}

// Stable hash of a star field — used by tests to assert determinism.
// Quantises floats to keep the hash insensitive to FP noise.
export function hashStarField(field: StarField): number {
  let h = 2166136261 >>> 0; // FNV-1a
  const mix = (n: number) => {
    h ^= Math.round(n * 1024) | 0;
    h = Math.imul(h, 16777619);
  };
  const all: StarSlot[] = [...field.far, ...field.mid, ...field.near];
  for (const s of all) {
    mix(s.worldX);
    mix(s.worldY);
    mix(s.baseSize);
    mix(s.baseAlpha);
    mix(s.twinklePhase);
    mix(s.twinklePeriodMs);
    mix(s.twinkleAmp);
    h ^= s.color.length;
    h ^= s.twinkles ? 1 : 0;
    h = Math.imul(h, 16777619);
  }
  return h >>> 0;
}
