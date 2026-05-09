// Tests for src/lib/scene/star-state.ts
//
// Run via:
//   node --experimental-strip-types --test tests/scene/star-state.test.ts
//
// Node 22.6+ strips TS types at load time. No tsc/tsx/vitest required.

import { test } from 'node:test';
import * as assert from 'node:assert/strict';
import {
  buildStarField,
  hashStarField,
  effectiveAlpha,
  STAR_COLOR_VARS,
  FAR_CONFIG,
  MID_CONFIG,
  NEAR_CONFIG,
  DEFAULT_SEED,
} from '../../src/lib/scene/star-state.ts';

test('determinism: same seed → identical field hash', () => {
  const a = buildStarField(DEFAULT_SEED);
  const b = buildStarField(DEFAULT_SEED);
  assert.equal(hashStarField(a), hashStarField(b));
  assert.equal(a.far.length, b.far.length);
  assert.equal(a.mid.length, b.mid.length);
  assert.equal(a.near.length, b.near.length);
});

test('determinism: different seed → different field hash', () => {
  const a = buildStarField(0xC0FFEE);
  const b = buildStarField(0xDEADBEEF);
  assert.notEqual(hashStarField(a), hashStarField(b));
});

test('counts respect oversample factor (1.6 × 1.6 = 2.56) on default seed', () => {
  const f = buildStarField();
  // FAR_CONFIG.count is target_visible × 2.56 ≈ 1536; allow ±5% slack on
  // round-tripping through Math.round.
  assert.ok(f.far.length === FAR_CONFIG.count, `far ${f.far.length} != ${FAR_CONFIG.count}`);
  assert.ok(f.mid.length === MID_CONFIG.count);
  assert.ok(f.near.length === NEAR_CONFIG.count);
});

test('all star colors are valid CSS variable names from the palette', () => {
  const f = buildStarField();
  for (const star of [...f.far, ...f.mid, ...f.near]) {
    assert.ok(STAR_COLOR_VARS.includes(star.color), `unexpected color: ${star.color}`);
  }
});

test('size distribution: log-normal mean lies inside layer min..max bounds', () => {
  const f = buildStarField();
  const layers = [
    { slots: f.far, cfg: FAR_CONFIG, name: 'far' },
    { slots: f.mid, cfg: MID_CONFIG, name: 'mid' },
    { slots: f.near, cfg: NEAR_CONFIG, name: 'near' },
  ];
  for (const { slots, cfg, name } of layers) {
    let sum = 0;
    let outOfBounds = 0;
    for (const s of slots) {
      sum += s.baseSize;
      if (s.baseSize < cfg.sizeMin || s.baseSize > cfg.sizeMax) outOfBounds++;
    }
    const mean = sum / slots.length;
    assert.equal(outOfBounds, 0, `${name}: ${outOfBounds} sizes out of bounds`);
    assert.ok(mean >= cfg.sizeMin && mean <= cfg.sizeMax, `${name}: mean ${mean} outside bounds`);
  }
});

test('size distribution shows variance (not degenerated to a constant)', () => {
  // Pinpoint tuning narrows the log-normal sigma; the test only asserts
  // the field has *some* size variation, not a heavy right tail.
  const f = buildStarField();
  const sizes = f.mid.map((s) => s.baseSize);
  const min = Math.min(...sizes);
  const max = Math.max(...sizes);
  assert.ok(max - min > 0.10, `expected size variance > 0.10, got [${min.toFixed(2)}, ${max.toFixed(2)}]`);
});

test('twinkle ratio is within 3σ of configured probability', () => {
  // Bernoulli with N draws and probability p has stdev sqrt(p(1-p)/N).
  // Three-sigma tolerance accommodates the small near-layer sample.
  const f = buildStarField();
  const layers = [
    { name: 'mid', slots: f.mid, prob: MID_CONFIG.twinkleProb },
    { name: 'near', slots: f.near, prob: NEAR_CONFIG.twinkleProb },
  ];
  for (const { name, slots, prob } of layers) {
    const twinkling = slots.filter((s) => s.twinkles).length / slots.length;
    const sigma = Math.sqrt((prob * (1 - prob)) / slots.length);
    const lo = Math.max(0, prob - 3 * sigma);
    const hi = Math.min(1, prob + 3 * sigma);
    assert.ok(twinkling >= lo && twinkling <= hi, `${name}: twinkle ratio ${twinkling.toFixed(3)} outside 3σ [${lo.toFixed(3)}, ${hi.toFixed(3)}]`);
  }
});

test('twinkle period is in 2000..9000ms range', () => {
  const f = buildStarField();
  for (const star of [...f.far, ...f.mid, ...f.near]) {
    assert.ok(star.twinklePeriodMs >= 2000 && star.twinklePeriodMs <= 9000);
  }
});

test('effectiveAlpha returns baseAlpha at t=0 for non-twinkling slots', () => {
  const f = buildStarField();
  const non = f.far.find((s) => !s.twinkles);
  if (!non) throw new Error('no non-twinkling far star found (fixture sanity)');
  assert.equal(effectiveAlpha(non, 0, false), non.baseAlpha);
  // Reduced motion freezes alpha at base regardless.
  assert.equal(effectiveAlpha(non, 12345, true), non.baseAlpha);
});

test('effectiveAlpha for twinkling star modulates within bounds', () => {
  const f = buildStarField();
  const tw = f.mid.find((s) => s.twinkles);
  if (!tw) throw new Error('no twinkling mid star found (fixture sanity)');
  let min = Infinity;
  let max = -Infinity;
  for (let t = 0; t < 10000; t += 25) {
    const a = effectiveAlpha(tw, t, false);
    if (a < min) min = a;
    if (a > max) max = a;
  }
  assert.ok(max > min, 'alpha should vary');
  assert.ok(min >= 0 && max <= 1, 'alpha must stay in [0,1]');
  // Reduced motion: same seed must hold steady.
  const fixed = effectiveAlpha(tw, 0, true);
  for (let t = 0; t < 10000; t += 250) {
    assert.equal(effectiveAlpha(tw, t, true), fixed);
  }
});

test('effectiveAlpha returns 0 for departed slots', () => {
  const f = buildStarField();
  const s = f.near[0];
  s.state = 'departed';
  s.fadeIn = 0;
  assert.equal(effectiveAlpha(s, 0, false), 0);
});

test('effectiveAlpha scales with fadeIn during fading-in state', () => {
  const f = buildStarField();
  const s = f.near.find((x) => !x.twinkles) ?? f.near[0];
  s.twinkles = false; // pin so we test fade alone
  s.state = 'fading-in';
  s.fadeIn = 0;
  assert.equal(effectiveAlpha(s, 0, false), 0);
  s.fadeIn = 0.5;
  assert.equal(effectiveAlpha(s, 0, false), s.baseAlpha * 0.5);
  s.fadeIn = 1.0;
  assert.equal(effectiveAlpha(s, 0, false), s.baseAlpha);
});

test('world dims match the largest layer config', () => {
  const f = buildStarField();
  assert.equal(f.worldW, Math.max(FAR_CONFIG.worldW, MID_CONFIG.worldW, NEAR_CONFIG.worldW));
  assert.equal(f.worldH, Math.max(FAR_CONFIG.worldH, MID_CONFIG.worldH, NEAR_CONFIG.worldH));
});

test('stars are distributed across the world (no all-corner clumping)', () => {
  const f = buildStarField();
  // Bin star positions into a 4x4 grid and assert all bins have stars.
  const bins = new Array(16).fill(0);
  for (const s of f.mid) {
    const bx = Math.min(3, Math.floor(s.worldX / (MID_CONFIG.worldW / 4)));
    const by = Math.min(3, Math.floor(s.worldY / (MID_CONFIG.worldH / 4)));
    bins[by * 4 + bx]++;
  }
  for (let i = 0; i < 16; i++) {
    assert.ok(bins[i] > 0, `bin ${i} empty: ${JSON.stringify(bins)}`);
  }
});
