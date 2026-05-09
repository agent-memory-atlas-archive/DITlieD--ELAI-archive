// star-shoot.ts — independent shooting-star particles tied to star slots.
//
// When the scheduler decides to fire, it picks a "present" star from the
// near or mid layer (weighted by brightness), marks the slot 'departed',
// and spawns a Shooter at that slot's world position. The shooter
// animates with scale/alpha decay + motion-blur trail. On lifetime end
// it regenerates the slot at a new deterministic position via
// regenSlot() (advancing the layer's regen RNG counter).
//
// Inter-arrival is exponential (Poisson process). Active mode = denser,
// idle mode = sparser. Up to 3 simultaneous shooters by default.
//
// Reduced-motion: fire() is suppressed by the caller; this module
// renders nothing if there are no shooters.
//
// No DOM imports. Renderable via a Canvas2D context handed in.

import type { StarLayer, StarSlot, StarField, Tone, Rng } from './star-state.ts';
import { configForLayer, makeRng, regenSlot, LAYER_Z } from './star-state.ts';

export interface Shooter {
  id: number;
  // World-space head position (px). worldZ is the depth coordinate;
  // shooters now move in 3D, so distant ones render small/dim and grow
  // as they approach the camera.
  worldX: number;
  worldY: number;
  worldZ: number;
  // Velocity (px/s) — vz biased toward the camera at spawn so most
  // shooters approach (the few with vz<0 recede).
  vx: number;
  vy: number;
  vz: number;
  // Lifetime (s) and elapsed (s).
  lifetime: number;
  elapsed: number;
  // Trail ring buffer of recent world positions; head is most recent.
  trail: { worldX: number; worldY: number; worldZ: number }[];
  trailMax: number;
  // Slot reference (so we can regen on death).
  slotLayer: StarLayer;
  slotId: number;
  // Color resolved at spawn time (matches the originating star's color
  // var); the renderer reads this through getComputedStyle.
  colorVar: string;
  // Leap-shooter flag — a fraction of shooters are spawned with strong
  // z velocity, longer lifetime, longer trail, and bigger head so they
  // visibly arc through depth like the warp-zoom does. The renderer
  // reads this to bump head/trail size.
  isLeap: boolean;
}

export type Mode = 'active' | 'idle';

export interface SchedulerConfig {
  // Mean inter-arrival in seconds for each mode. Exponential dist sampled
  // via -ln(U) * mean.
  meanIntervalActiveS: number;
  meanIntervalIdleS: number;
  // Max simultaneous shooters.
  maxConcurrent: number;
  // Shooter speed (px/s) range.
  speedMinPx: number;
  speedMaxPx: number;
  // Lifetime (s) range.
  lifetimeMinS: number;
  lifetimeMaxS: number;
  // Per-frame velocity decay multiplier (applied as v *= decay^(dt*60)
  // so it's framerate-independent at ~60Hz).
  velocityDecayPerFrame: number;
  // Trail ring-buffer length. Longer = trails persist longer in space.
  trailLength: number;
  // Depth-axis (z) speed range (world units / s). Most shooters spawn
  // with positive vz so they approach the camera; a fraction recede.
  zSpeedMinPx: number;
  zSpeedMaxPx: number;
  // Probability the shooter's z velocity is positive (toward camera).
  zApproachProb: number;
  // Probability the shoot direction is biased outward from the world
  // center (vs inward / random).
  outwardBiasProb: number;
  // Per-layer pick weights (relative). Sum is normalized internally.
  // Bias toward near for visual punch, with mid mid-distance shooters
  // and rarer far ones for depth variety.
  layerPickWeights: { near: number; mid: number; far: number };
  // Fade-in duration (s) for the regenerated slot.
  regenFadeS: number;
  // Mean inter-arrival (s) between explosions in each mode. Explosions
  // pick a random present star and flash it bright + big briefly.
  meanExplosionActiveS: number;
  meanExplosionIdleS: number;
  // Explosion envelope timing (ms).
  explosionRiseMs: number;
  explosionFallMs: number;
  // Peak amplitude scaling. peakAlpha = clamp(orig*alphaMul, 0, 1);
  // peakSize = orig * sizeMul.
  explosionPeakAlphaMul: number;
  explosionPeakSizeMul: number;
  // Leap-shooter variant. A fraction of shoots become "leap" shooters
  // — stronger z velocity, longer lifetime, longer trail, bigger head.
  // They visibly arc through depth like the leap-zoom transition does.
  leapShooterProb: number;
  leapShooterZSpeedMul: number;
  leapShooterLifetimeMul: number;
  leapShooterTrailLengthMul: number;
  leapShooterHeadSizeMul: number;
}

export const DEFAULT_SCHEDULER: SchedulerConfig = {
  meanIntervalActiveS: 3.2,
  meanIntervalIdleS: 7.0,
  maxConcurrent: 6,
  // Lowered from 240/520 — shooters were felt as "too fast".
  speedMinPx: 140,
  speedMaxPx: 290,
  lifetimeMinS: 1.4,
  lifetimeMaxS: 2.6,
  velocityDecayPerFrame: 0.985,
  trailLength: 36,
  outwardBiasProb: 0.90,
  layerPickWeights: { near: 0.50, mid: 0.32, far: 0.18 },
  regenFadeS: 0.6,
  // Bumped z range so depth motion is visible during the shooter's
  // lifetime — leap shooters multiply this further.
  zSpeedMinPx: 100,
  zSpeedMaxPx: 440,
  zApproachProb: 0.72,
  meanExplosionActiveS: 22.0,
  meanExplosionIdleS: 55.0,
  explosionRiseMs: 220,
  explosionFallMs: 900,
  explosionPeakAlphaMul: 6.0,
  explosionPeakSizeMul: 3.6,
  leapShooterProb: 0.28,
  leapShooterZSpeedMul: 3.5,
  leapShooterLifetimeMul: 1.5,
  leapShooterTrailLengthMul: 1.7,
  leapShooterHeadSizeMul: 1.6,
};

export interface Explosion {
  slotLayer: StarLayer;
  slotId: number;
  startMs: number;
  riseMs: number;
  fallMs: number;
  origAlpha: number;
  origSize: number;
  peakAlpha: number;
  peakSize: number;
}

export interface ShootScheduler {
  shooters: Shooter[];
  explosions: Explosion[];
  // Time (s) until next fire attempt.
  nextFireIn: number;
  // Time (s) until next explosion attempt.
  nextExplosionIn: number;
  mode: Mode;
  config: SchedulerConfig;
  // Per-layer regen RNGs (advanced on each regen so positions are a
  // deterministic stream).
  regenRngs: { far: Rng; mid: Rng; near: Rng };
  // RNG for shooter timing/direction/picking. Seeded from
  // performance.now() at construction time — this non-determinism is
  // perceptual, not state, per the plan.
  eventRng: Rng;
  // Tone is forwarded to regenSlot so the new star's palette weights are
  // consistent with the rest of the field.
  tone: Tone;
  // Monotonic counter for shooter ids (debug only).
  nextShooterId: number;
}

export function makeScheduler(opts: {
  config?: SchedulerConfig;
  regenSeed: number;
  eventSeed: number;
  mode: Mode;
  tone?: Tone;
}): ShootScheduler {
  const config = opts.config ?? DEFAULT_SCHEDULER;
  const eventRng = makeRng(opts.eventSeed >>> 0);
  return {
    shooters: [],
    explosions: [],
    nextFireIn: sampleInterval(eventRng, config, opts.mode),
    nextExplosionIn: sampleExplosionInterval(eventRng, config, opts.mode),
    mode: opts.mode,
    config,
    regenRngs: {
      far:  makeRng((opts.regenSeed ^ 0xA1A1) >>> 0),
      mid:  makeRng((opts.regenSeed ^ 0xB2B2) >>> 0),
      near: makeRng((opts.regenSeed ^ 0xC3C3) >>> 0),
    },
    eventRng,
    tone: opts.tone ?? 'mixed',
    nextShooterId: 1,
  };
}

function sampleInterval(rng: Rng, cfg: SchedulerConfig, mode: Mode): number {
  // Exponential with mean = mean.
  const mean = mode === 'active' ? cfg.meanIntervalActiveS : cfg.meanIntervalIdleS;
  const u = Math.max(rng.next(), 1e-9);
  return -Math.log(u) * mean;
}

function sampleExplosionInterval(rng: Rng, cfg: SchedulerConfig, mode: Mode): number {
  const mean = mode === 'active' ? cfg.meanExplosionActiveS : cfg.meanExplosionIdleS;
  const u = Math.max(rng.next(), 1e-9);
  return -Math.log(u) * mean;
}

function pickLayer(rng: Rng, w: { near: number; mid: number; far: number }): StarLayer {
  const total = w.near + w.mid + w.far;
  const r = rng.next() * total;
  if (r < w.near) return 'near';
  if (r < w.near + w.mid) return 'mid';
  return 'far';
}

function slotsForLayer(field: StarField, layer: StarLayer): StarSlot[] {
  return layer === 'near' ? field.near : layer === 'mid' ? field.mid : field.far;
}

function lerp(rng: Rng, a: number, b: number): number {
  return a + rng.next() * (b - a);
}

// Pick a present star from the requested layer. Weighting: brighter stars
// are more likely. Returns -1 if no candidate is available.
function pickStarIndex(rng: Rng, slots: StarSlot[]): number {
  // Build cumulative weights of present slots only.
  let total = 0;
  for (const s of slots) {
    if (s.state === 'present') total += s.baseAlpha * s.baseAlpha;
  }
  if (total <= 0) return -1;
  let r = rng.next() * total;
  for (let i = 0; i < slots.length; i++) {
    const s = slots[i];
    if (s.state !== 'present') continue;
    const w = s.baseAlpha * s.baseAlpha;
    if (r < w) return i;
    r -= w;
  }
  return -1;
}

// Sample shooter velocity. With outward bias, the direction is rotated
// slightly away from world center. ~10% of shoots are inward.
// vz is sampled separately: most shooters approach the camera (positive
// z); a minority recede (negative z, smaller magnitude).
function sampleVelocity(rng: Rng, cfg: SchedulerConfig, originX: number, originY: number, worldCx: number, worldCy: number): { vx: number; vy: number; vz: number } {
  const speed = lerp(rng, cfg.speedMinPx, cfg.speedMaxPx);
  const inward = rng.next() > cfg.outwardBiasProb;
  const baseAngle = Math.atan2(originY - worldCy, originX - worldCx);
  // Outward direction = pointing away from world center; inward = toward.
  const dirAngle = inward ? baseAngle + Math.PI : baseAngle;
  // Add jitter so shoots don't always go radially.
  const jitter = (rng.next() - 0.5) * Math.PI * 0.6;
  const a = dirAngle + jitter;
  const zSpeed = lerp(rng, cfg.zSpeedMinPx, cfg.zSpeedMaxPx);
  const approach = rng.next() < cfg.zApproachProb;
  // Receding shooters move at half the magnitude — they shrink fast and
  // we don't want them to feel as dramatic as the approaches.
  const vz = approach ? zSpeed : -zSpeed * 0.5;
  return { vx: Math.cos(a) * speed, vy: Math.sin(a) * speed, vz };
}

// Try to fire a shooter. Returns the spawned shooter or null if none was
// spawned (max concurrent reached, no candidate slots).
export function fireShooter(sched: ShootScheduler, field: StarField): Shooter | null {
  if (sched.shooters.length >= sched.config.maxConcurrent) return null;
  const layer = pickLayer(sched.eventRng, sched.config.layerPickWeights);
  const slots = slotsForLayer(field, layer);
  const idx = pickStarIndex(sched.eventRng, slots);
  if (idx < 0) return null;
  const slot = slots[idx];
  const worldCx = field.worldW / 2;
  const worldCy = field.worldH / 2;
  const { vx, vy, vz: rawVz } = sampleVelocity(sched.eventRng, sched.config, slot.worldX, slot.worldY, worldCx, worldCy);
  // Roll leap-variant after sampleVelocity so the rng order for vx/vy/vz
  // stays stable. Leap shooters multiply z velocity, lifetime, trail
  // length, and head size — they read as dramatic depth-traversing
  // streaks (the user's "looks 3D like the lines between nodes" cue).
  const isLeap = sched.eventRng.next() < sched.config.leapShooterProb;
  const vz = isLeap ? rawVz * sched.config.leapShooterZSpeedMul : rawVz;
  const baseLifetime = lerp(sched.eventRng, sched.config.lifetimeMinS, sched.config.lifetimeMaxS);
  const lifetime = isLeap ? baseLifetime * sched.config.leapShooterLifetimeMul : baseLifetime;
  const trailMax = isLeap
    ? Math.round(sched.config.trailLength * sched.config.leapShooterTrailLengthMul)
    : sched.config.trailLength;
  const shooter: Shooter = {
    id: sched.nextShooterId++,
    worldX: slot.worldX,
    worldY: slot.worldY,
    // Spawn at the slot's layer plane; vz then carries it through depth.
    worldZ: LAYER_Z[layer],
    vx,
    vy,
    vz,
    lifetime,
    elapsed: 0,
    trail: [],
    trailMax,
    slotLayer: layer,
    slotId: slot.slotId,
    colorVar: slot.color,
    isLeap,
  };
  // Mark slot departed; renderer will skip it until regen.
  slot.state = 'departed';
  slot.fadeIn = 0;
  sched.shooters.push(shooter);
  return shooter;
}

// Advance one shooter by dt seconds. Returns true if the shooter is
// still alive after the step. On death the slot is regenerated.
export function stepShooter(shooter: Shooter, sched: ShootScheduler, field: StarField, dtS: number): boolean {
  shooter.elapsed += dtS;
  // Push current head into the trail ring buffer (3D: x, y, z).
  shooter.trail.unshift({ worldX: shooter.worldX, worldY: shooter.worldY, worldZ: shooter.worldZ });
  if (shooter.trail.length > shooter.trailMax) shooter.trail.length = shooter.trailMax;
  // Move (3D — z carries shooter through depth).
  shooter.worldX += shooter.vx * dtS;
  shooter.worldY += shooter.vy * dtS;
  shooter.worldZ += shooter.vz * dtS;
  // Apply per-frame decay scaled by dt (framerate-independent).
  const frames = dtS * 60;
  const decay = Math.pow(sched.config.velocityDecayPerFrame, frames);
  shooter.vx *= decay;
  shooter.vy *= decay;
  shooter.vz *= decay;
  if (shooter.elapsed >= shooter.lifetime) {
    // Regen the slot at a new world position.
    const slots = slotsForLayer(field, shooter.slotLayer);
    const slot = slots[shooter.slotId];
    if (slot) {
      const cfg = configForLayer(shooter.slotLayer);
      const rng = shooter.slotLayer === 'near' ? sched.regenRngs.near
                : shooter.slotLayer === 'mid'  ? sched.regenRngs.mid
                : sched.regenRngs.far;
      regenSlot(slot, rng, cfg, sched.tone);
    }
    return false;
  }
  return true;
}

// Visual properties for a shooter at its current state.
// scale tapers from 1 to 0.05 over the second half of life;
// alpha tapers from 1 to 0 likewise.
export function shooterAlpha(shooter: Shooter): number {
  const t = shooter.elapsed / shooter.lifetime;
  if (t < 0.5) return 1;
  const u = (t - 0.5) / 0.5;
  return Math.max(0, 1 - u);
}
export function shooterScale(shooter: Shooter): number {
  const t = shooter.elapsed / shooter.lifetime;
  if (t < 0.5) return 1;
  const u = (t - 0.5) / 0.5;
  // Smooth taper to 0.05.
  return Math.max(0.05, 1 - u * 0.95);
}

// Try to fire an explosion. Picks a present star from any layer,
// captures its base alpha/size, sets the explosion record. The tick
// loop then mutates baseAlpha/baseSize per the rise→fall envelope and
// restores them when the explosion ends.
export function fireExplosion(sched: ShootScheduler, field: StarField, nowMs: number): Explosion | null {
  // Slight bias toward mid/far so explosions read as "deep space"
  // events more often than "right next to the camera".
  const layer = pickLayer(sched.eventRng, { near: 0.25, mid: 0.40, far: 0.35 });
  const slots = slotsForLayer(field, layer);
  const idx = pickStarIndex(sched.eventRng, slots);
  if (idx < 0) return null;
  const slot = slots[idx];
  const ex: Explosion = {
    slotLayer: layer,
    slotId: slot.slotId,
    startMs: nowMs,
    riseMs: sched.config.explosionRiseMs,
    fallMs: sched.config.explosionFallMs,
    origAlpha: slot.baseAlpha,
    origSize: slot.baseSize,
    peakAlpha: Math.min(1, slot.baseAlpha * sched.config.explosionPeakAlphaMul),
    peakSize: slot.baseSize * sched.config.explosionPeakSizeMul,
  };
  sched.explosions.push(ex);
  return ex;
}

function smoothstep01(t: number): number {
  const x = t < 0 ? 0 : t > 1 ? 1 : t;
  return x * x * (3 - 2 * x);
}

// Advance explosions; mutates slot baseAlpha/baseSize per envelope.
// Restores originals when the envelope completes.
export function tickExplosions(sched: ShootScheduler, field: StarField, nowMs: number): void {
  if (sched.explosions.length === 0) return;
  const alive: Explosion[] = [];
  for (const ex of sched.explosions) {
    const slot = slotsForLayer(field, ex.slotLayer)[ex.slotId];
    const age = nowMs - ex.startMs;
    if (!slot || age >= ex.riseMs + ex.fallMs) {
      // Done — restore originals (a subsequent regenSlot may overwrite,
      // which is fine; this just hands the slot back unchanged).
      if (slot) {
        slot.baseAlpha = ex.origAlpha;
        slot.baseSize = ex.origSize;
      }
      continue;
    }
    const env = age < ex.riseMs
      ? smoothstep01(age / ex.riseMs)
      : smoothstep01(1 - (age - ex.riseMs) / ex.fallMs);
    slot.baseAlpha = ex.origAlpha + (ex.peakAlpha - ex.origAlpha) * env;
    slot.baseSize = ex.origSize + (ex.peakSize - ex.origSize) * env;
    alive.push(ex);
  }
  sched.explosions = alive;
}

// Advance the whole scheduler by dt seconds. Call once per rAF tick.
// nowMs is wall-clock ms (perf.now()) used as the explosion clock —
// dt-driven envelopes drift without an absolute reference.
export function tickScheduler(sched: ShootScheduler, field: StarField, dtS: number, suppress: boolean, nowMs: number = performance.now()): void {
  // Step shooters; remove dead ones (regen handled inside stepShooter).
  if (sched.shooters.length > 0) {
    const alive: Shooter[] = [];
    for (const sh of sched.shooters) {
      if (stepShooter(sh, sched, field, dtS)) alive.push(sh);
    }
    sched.shooters = alive;
  }

  // Advance fade-in for regenerating slots; promote to 'present' when done.
  promoteFadeIn(field.far, dtS, sched.config.regenFadeS);
  promoteFadeIn(field.mid, dtS, sched.config.regenFadeS);
  promoteFadeIn(field.near, dtS, sched.config.regenFadeS);

  // Explosions tick regardless of suppress so an in-flight explosion
  // can finish and restore its slot. New ones don't fire under suppress.
  tickExplosions(sched, field, nowMs);

  if (suppress) return;

  // Fire timer.
  sched.nextFireIn -= dtS;
  while (sched.nextFireIn <= 0) {
    fireShooter(sched, field);
    sched.nextFireIn += sampleInterval(sched.eventRng, sched.config, sched.mode);
  }

  // Explosion timer.
  sched.nextExplosionIn -= dtS;
  while (sched.nextExplosionIn <= 0) {
    fireExplosion(sched, field, nowMs);
    sched.nextExplosionIn += sampleExplosionInterval(sched.eventRng, sched.config, sched.mode);
  }
}

function promoteFadeIn(slots: StarSlot[], dtS: number, fadeS: number): void {
  for (const s of slots) {
    if (s.state !== 'fading-in') continue;
    s.fadeIn += dtS / fadeS;
    if (s.fadeIn >= 1) {
      s.fadeIn = 1;
      s.state = 'present';
    }
  }
}

export function setMode(sched: ShootScheduler, mode: Mode): void {
  if (sched.mode === mode) return;
  sched.mode = mode;
  // Re-sample the next fire interval against the new mean.
  sched.nextFireIn = sampleInterval(sched.eventRng, sched.config, mode);
  sched.nextExplosionIn = sampleExplosionInterval(sched.eventRng, sched.config, mode);
}
