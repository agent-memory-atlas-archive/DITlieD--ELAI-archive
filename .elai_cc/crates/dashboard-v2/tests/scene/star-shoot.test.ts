// Tests for src/lib/scene/star-shoot.ts
//
// Run via:
//   node --experimental-strip-types --test tests/scene/star-shoot.test.ts

import { test } from 'node:test';
import * as assert from 'node:assert/strict';
import { buildStarField } from '../../src/lib/scene/star-state.ts';
import {
  makeScheduler,
  fireShooter,
  stepShooter,
  shooterAlpha,
  shooterScale,
  tickScheduler,
  DEFAULT_SCHEDULER,
} from '../../src/lib/scene/star-shoot.ts';

test('fireShooter consumes a star slot and returns a shooter', () => {
  const field = buildStarField();
  const sched = makeScheduler({ regenSeed: 1, eventSeed: 1, mode: 'active' });
  const beforeTotal =
    field.near.filter((s) => s.state === 'present').length +
    field.mid.filter((s) => s.state === 'present').length +
    field.far.filter((s) => s.state === 'present').length;
  const sh = fireShooter(sched, field);
  assert.ok(sh, 'shooter should fire when slots are present');
  const afterTotal =
    field.near.filter((s) => s.state === 'present').length +
    field.mid.filter((s) => s.state === 'present').length +
    field.far.filter((s) => s.state === 'present').length;
  assert.equal(beforeTotal - afterTotal, 1, 'exactly one present slot becomes departed');
  // The departed slot belongs to the layer the shooter says it took it from.
  const slots = sh!.slotLayer === 'near' ? field.near
              : sh!.slotLayer === 'mid'  ? field.mid
              : field.far;
  assert.equal(slots[sh!.slotId].state, 'departed');
  assert.equal(sh!.elapsed, 0);
  assert.ok(sh!.lifetime >= DEFAULT_SCHEDULER.lifetimeMinS && sh!.lifetime <= DEFAULT_SCHEDULER.lifetimeMaxS);
});

test('shooter lifecycle: alpha and scale taper to ~0 over second half', () => {
  const field = buildStarField();
  const sched = makeScheduler({ regenSeed: 1, eventSeed: 2, mode: 'active' });
  const sh = fireShooter(sched, field);
  assert.ok(sh);
  // Pin a known lifetime so the test isn't randomness-sensitive.
  sh!.lifetime = 1.0;
  const dt = 0.05;
  let lastAlpha = 1;
  let lastScale = 1;
  for (let elapsed = 0; elapsed <= 1.0; elapsed += dt) {
    sh!.elapsed = elapsed;
    const a = shooterAlpha(sh!);
    const s = shooterScale(sh!);
    if (elapsed <= 0.5) {
      assert.equal(a, 1, `alpha should be 1 at t=${elapsed.toFixed(2)}`);
      assert.equal(s, 1, `scale should be 1 at t=${elapsed.toFixed(2)}`);
    } else {
      assert.ok(a <= lastAlpha + 1e-9, `alpha not monotonic at t=${elapsed.toFixed(2)} (was ${lastAlpha}, now ${a})`);
      assert.ok(s <= lastScale + 1e-9, `scale not monotonic at t=${elapsed.toFixed(2)}`);
    }
    lastAlpha = a;
    lastScale = s;
  }
  sh!.elapsed = 0.999;
  assert.ok(shooterAlpha(sh!) < 0.05, `alpha at t=0.999 should be near 0, got ${shooterAlpha(sh!)}`);
  assert.ok(shooterScale(sh!) <= 0.10, `scale at t=0.999 should be near 0.05, got ${shooterScale(sh!)}`);
});

test('stepShooter regenerates the slot at lifetime end', () => {
  const field = buildStarField();
  const sched = makeScheduler({ regenSeed: 1, eventSeed: 3, mode: 'active' });
  const sh = fireShooter(sched, field);
  assert.ok(sh);
  sh!.lifetime = 0.1;
  const slots = sh!.slotLayer === 'near' ? field.near
              : sh!.slotLayer === 'mid'  ? field.mid
              : field.far;
  const slot = slots[sh!.slotId];
  const oldX = slot.worldX;
  const oldY = slot.worldY;
  // Step past lifetime.
  const alive = stepShooter(sh!, sched, field, 0.2);
  assert.equal(alive, false, 'shooter should be dead after exceeding lifetime');
  assert.equal(slot.state, 'fading-in', 'slot should regen into fading-in state');
  assert.equal(slot.fadeIn, 0);
  // Position must have changed (deterministic but new).
  assert.ok(slot.worldX !== oldX || slot.worldY !== oldY, 'slot position should change on regen');
});

test('tickScheduler promotes fading-in slots back to present after fade duration', () => {
  const field = buildStarField();
  const sched = makeScheduler({ regenSeed: 1, eventSeed: 4, mode: 'active' });
  // Suppress further fires.
  sched.config = { ...DEFAULT_SCHEDULER, meanIntervalActiveS: 1e9, meanIntervalIdleS: 1e9 };
  const sh = fireShooter(sched, field);
  assert.ok(sh);
  sh!.lifetime = 0.1;
  // Step shooter dead → slot becomes 'fading-in'.
  stepShooter(sh!, sched, field, 0.2);
  sched.shooters = sched.shooters.filter((s) => s !== sh);
  const slots = sh!.slotLayer === 'near' ? field.near
              : sh!.slotLayer === 'mid'  ? field.mid
              : field.far;
  const slot = slots[sh!.slotId];
  assert.equal(slot.state, 'fading-in');
  // Tick for longer than regenFadeS.
  tickScheduler(sched, field, DEFAULT_SCHEDULER.regenFadeS + 0.1, false);
  assert.equal(slot.state, 'present');
  assert.equal(slot.fadeIn, 1);
});

test('tickScheduler with suppress=true does not fire new shooters', () => {
  const field = buildStarField();
  const sched = makeScheduler({ regenSeed: 1, eventSeed: 5, mode: 'active' });
  // Force the next-fire timer to zero so a tick would otherwise fire.
  sched.nextFireIn = 0;
  tickScheduler(sched, field, 0.05, true);
  assert.equal(sched.shooters.length, 0, 'no shooters when suppressed (reduced-motion)');
});

test('maxConcurrent caps shooter count', () => {
  const field = buildStarField();
  const sched = makeScheduler({ regenSeed: 1, eventSeed: 6, mode: 'active' });
  for (let i = 0; i < 10; i++) fireShooter(sched, field);
  assert.ok(sched.shooters.length <= sched.config.maxConcurrent, `shooters ${sched.shooters.length} > max ${sched.config.maxConcurrent}`);
});

test('exponential inter-arrival mean is close to configured value', () => {
  // Sample many intervals and verify the empirical mean is within ±15%.
  const field = buildStarField();
  const sched = makeScheduler({ regenSeed: 1, eventSeed: 7, mode: 'active' });
  // Pull 4000 interval samples directly from sched.eventRng using the
  // same formula used internally.
  const samples: number[] = [];
  for (let i = 0; i < 4000; i++) {
    const u = Math.max(sched.eventRng.next(), 1e-9);
    samples.push(-Math.log(u) * DEFAULT_SCHEDULER.meanIntervalActiveS);
  }
  const mean = samples.reduce((a, b) => a + b, 0) / samples.length;
  const target = DEFAULT_SCHEDULER.meanIntervalActiveS;
  assert.ok(Math.abs(mean - target) / target < 0.15, `empirical mean ${mean.toFixed(2)} vs target ${target}`);
});

test('velocity decays toward zero on long-running shooters', () => {
  const field = buildStarField();
  const sched = makeScheduler({ regenSeed: 1, eventSeed: 8, mode: 'active' });
  const sh = fireShooter(sched, field);
  assert.ok(sh);
  sh!.lifetime = 1e9; // disable death for this test
  const initial = Math.hypot(sh!.vx, sh!.vy);
  for (let i = 0; i < 600; i++) stepShooter(sh!, sched, field, 1 / 60);
  const after = Math.hypot(sh!.vx, sh!.vy);
  assert.ok(after < initial * 0.5, `velocity should decay (${after.toFixed(2)} vs ${initial.toFixed(2)})`);
});
