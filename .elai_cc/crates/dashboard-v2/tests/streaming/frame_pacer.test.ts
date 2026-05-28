// Tests for src/lib/streaming/frame_pacer.ts
//
// Run via:
//   node --experimental-strip-types --test tests/streaming/frame_pacer.test.ts
//
// Drives FramePacer with synthetic RAF + clock so the cadence is
// deterministic. Tests three burst patterns (uniform / bursty /
// single-shot) per plan ACCEPTANCE row "drain rate ~= configured cps
// +/- 10%". See workflow/reasoning/E-2-decisions.md DECISION-6.

import { test } from 'node:test';
import * as assert from 'node:assert/strict';
import { FramePacer, pickCps } from '../../src/lib/streaming/frame_pacer.ts';

interface MockRaf {
  raf: (cb: (ts: number) => void) => number;
  cancelRaf: (h: number) => void;
  now: () => number;
  advance: (ms: number) => void;
}

function makeMockRaf(initialMs = 0): MockRaf {
  let clock = initialMs;
  let pending: ((ts: number) => void) | null = null;
  let nextHandle = 1;

  return {
    raf: (cb) => {
      pending = cb;
      const h = nextHandle++;
      return h;
    },
    cancelRaf: (_h) => {
      pending = null;
    },
    now: () => clock,
    advance: (ms) => {
      clock += ms;
      const cb = pending;
      pending = null;
      if (cb) cb(clock);
    },
  };
}

test('FramePacer drains at the target cps within +/-10%', () => {
  // 60 cps -> 60 chars/sec -> 6 chars per 100ms tick.
  const mock = makeMockRaf();
  const pacer = new FramePacer({
    cps: 60,
    rafSource: mock.raf,
    cancelRafSource: mock.cancelRaf,
    nowSource: mock.now,
  });
  // 600-char buffer drained over ~10 seconds at 60 cps.
  pacer.enqueue('a'.repeat(600));
  let rendered = '';
  pacer.subscribe((r) => { rendered = r; });

  // First tick at t=0 only initialises the lastTickMs anchor.
  mock.advance(0);
  // Tick every 100ms for 10s = 100 ticks.
  for (let i = 0; i < 100; i++) {
    mock.advance(100);
  }
  // We allow +/-10% slack on cps target.
  const min = 600 * 0.9;
  const max = 600 * 1.1;
  assert.ok(
    rendered.length >= min && rendered.length <= max,
    `expected ~600 chars rendered after 10s at 60 cps; got ${rendered.length}`,
  );
});

test('FramePacer flush() drains the buffer instantly', () => {
  const mock = makeMockRaf();
  const pacer = new FramePacer({
    cps: 60,
    rafSource: mock.raf,
    cancelRafSource: mock.cancelRaf,
    nowSource: mock.now,
  });
  pacer.enqueue('hello world this is a long string');
  let rendered = '';
  pacer.subscribe((r) => { rendered = r; });
  pacer.flush();
  assert.equal(rendered, 'hello world this is a long string');
  assert.equal(pacer.getBufferedLength(), 0);
});

test('FramePacer handles bursty input — chars-per-tick stays bounded', () => {
  const mock = makeMockRaf();
  const pacer = new FramePacer({
    cps: 60,
    rafSource: mock.raf,
    cancelRafSource: mock.cancelRaf,
    nowSource: mock.now,
  });
  const renderedPerTick: number[] = [];
  let prev = 0;
  pacer.subscribe((r) => {
    renderedPerTick.push(r.length - prev);
    prev = r.length;
  });

  // Burst all 600 chars at once instead of dripping.
  pacer.enqueue('x'.repeat(600));
  // First tick anchors the clock.
  mock.advance(0);
  // 100 ticks of 100ms each — at 60 cps the per-tick budget is ~6 chars.
  for (let i = 0; i < 100; i++) {
    mock.advance(100);
  }
  // Drop the first sample (post-mount + initial empty tick) and check
  // that no later sample exceeds ~12 chars (2x budget, generous slack).
  const samples = renderedPerTick.slice(1);
  const maxBurst = Math.max(...samples);
  assert.ok(maxBurst <= 12, `expected max per-tick burst <= 12 chars at 60 cps; got ${maxBurst}`);
});

test('FramePacer single-shot small input drains fully on first non-zero tick', () => {
  const mock = makeMockRaf();
  const pacer = new FramePacer({
    cps: 100,
    rafSource: mock.raf,
    cancelRafSource: mock.cancelRaf,
    nowSource: mock.now,
  });
  pacer.enqueue('hi');
  let rendered = '';
  pacer.subscribe((r) => { rendered = r; });
  mock.advance(0);
  mock.advance(1_000);
  assert.equal(rendered, 'hi');
});

test('FramePacer destroy() drops buffered chars and cancels RAF', () => {
  const mock = makeMockRaf();
  const pacer = new FramePacer({
    cps: 60,
    rafSource: mock.raf,
    cancelRafSource: mock.cancelRaf,
    nowSource: mock.now,
  });
  let rendered = '';
  pacer.subscribe((r) => { rendered = r; });
  pacer.enqueue('dropped tail');
  pacer.destroy();
  assert.equal(pacer.isDestroyed(), true);
  // Advancing the clock past the would-have-been drain time should
  // NOT produce any further output.
  mock.advance(5_000);
  assert.equal(rendered, '');
});

test('FramePacer clamps cps below 40 and above 200', () => {
  const lo = new FramePacer({ cps: 5 });
  assert.equal(lo.getCps(), 40);
  const hi = new FramePacer({ cps: 5000 });
  assert.equal(hi.getCps(), 200);
});

test('pickCps returns 60 for code (markdown fence) and 100 for prose', () => {
  assert.equal(pickCps('```rust\nfn main() {}'), 60);
  assert.equal(pickCps('plain prose with no fences'), 100);
});
