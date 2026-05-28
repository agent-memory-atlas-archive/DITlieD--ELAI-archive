// ============================================================
// frame_pacer.ts — RAF-buffered token drain for SSE streaming
// E-2 Task 1. See workflow/reasoning/E-2-decisions.md DECISION-6 (env gate).
//
// Design: incoming SSE tokens are enqueued into an internal buffer; a
// requestAnimationFrame loop drains at the configured chars-per-second
// (cps) target so the rendered character rate is stable regardless of
// upstream chunk burstiness. `flush()` is called at stream-end to drain
// the buffer instantly without waiting for the RAF cadence.
//
// Constraints:
//   * Stdlib + RAF only — no new dependency per CLAUDE.md (Anti-stale).
//   * No setTimeout-based polling for the drain (RAF only) — avoids
//     drift on browser background-tab throttling.
//   * Single-subscriber model (one bubble per stream); call destroy() to
//     unsubscribe.
//   * Off-by-default — wire-dark behind ELAI_FRAME_PACER_ENABLE per
//     DECISION-6; CommandWindow.svelte checks the setting and instantiates
//     FramePacer only when enabled. When disabled, tokens pass through
//     unchanged (zero behaviour change vs pre-E-2).
//
// Inspired by Vercel AI SDK 4.1 smoothStream (vendor reference; no code
// reuse). Doherty threshold (Doherty + Kelisky, IBM Research 1982) is the
// canonical anchor for human-readable cadence (60-100 cps).
// ============================================================

/** Configuration for a FramePacer instance. */
export interface FramePacerOptions {
  /** Target characters drained per second (clamped to [40, 200]). Defaults to 60. */
  cps: number;
  /** Optional override for the RAF source — used by tests to drive the loop deterministically. */
  rafSource?: (cb: (ts: number) => void) => number;
  /** Optional override for cancel-RAF (paired with rafSource). */
  cancelRafSource?: (handle: number) => void;
  /** Optional override for performance.now() — used by tests. */
  nowSource?: () => number;
}

/** Subscriber callback. Receives the cumulative rendered string so far. */
export type FramePacerSubscriber = (rendered: string) => void;

/**
 * FramePacer drains SSE chunks at a configured cps target using
 * requestAnimationFrame. Designed to be wrapped around an `onToken`
 * callback in the stream consumer.
 */
export class FramePacer {
  private readonly cps: number;
  private readonly raf: (cb: (ts: number) => void) => number;
  private readonly cancelRaf: (handle: number) => void;
  private readonly now: () => number;

  private buffer = '';
  private rendered = '';
  private subscriber: FramePacerSubscriber | null = null;
  private rafHandle: number | null = null;
  private lastTickMs: number | null = null;
  private destroyed = false;

  constructor(opts: FramePacerOptions) {
    // Clamp cps to a safe range. Below 40 the cadence feels stuttery; above
    // 200 the buffer drains too fast to matter (worse than no pacing).
    const requested = Math.floor(opts.cps);
    if (Number.isNaN(requested) || requested < 40) {
      this.cps = 40;
    } else if (requested > 200) {
      this.cps = 200;
    } else {
      this.cps = requested;
    }

    this.raf = opts.rafSource ?? ((cb) => {
      // requestAnimationFrame returns 0 in some non-DOM test envs; coerce to 0.
      if (typeof requestAnimationFrame === 'function') {
        return requestAnimationFrame(cb);
      }
      // Fallback for SSR / test envs without RAF — synchronous tick.
      cb(this.now());
      return 0;
    });
    this.cancelRaf = opts.cancelRafSource ?? ((handle) => {
      if (typeof cancelAnimationFrame === 'function') {
        cancelAnimationFrame(handle);
      }
    });
    this.now = opts.nowSource ?? (() => {
      if (typeof performance !== 'undefined' && typeof performance.now === 'function') {
        return performance.now();
      }
      return Date.now();
    });
  }

  /** Returns the target chars-per-second (post-clamp). */
  getCps(): number {
    return this.cps;
  }

  /** Returns the cumulative rendered string. */
  getRendered(): string {
    return this.rendered;
  }

  /** Returns the count of buffered (not-yet-rendered) characters. */
  getBufferedLength(): number {
    return this.buffer.length;
  }

  /**
   * Enqueue an incoming token / chunk. Drains start on next RAF tick.
   * Returns immediately; no callback fires synchronously.
   */
  enqueue(chunk: string): void {
    if (this.destroyed || chunk.length === 0) return;
    this.buffer += chunk;
    this.ensureRaf();
  }

  /**
   * Subscribe to drain events. The callback fires with the cumulative
   * rendered string on every RAF tick that emits at least one character,
   * plus once on flush(). Returns an unsubscribe fn.
   */
  subscribe(cb: FramePacerSubscriber): () => void {
    if (this.destroyed) {
      // Stream already torn down — fire once with the final state for
      // late subscribers so they don't see an empty bubble.
      cb(this.rendered);
      return () => {};
    }
    this.subscriber = cb;
    // Emit current state so late subscribers see what's already rendered.
    cb(this.rendered);
    return () => {
      this.subscriber = null;
    };
  }

  /**
   * Immediately drain the buffer (no cadence). Used at stream-end so the
   * user never waits for a buffered tail. Cancels any pending RAF.
   */
  flush(): void {
    if (this.destroyed) return;
    if (this.buffer.length > 0) {
      this.rendered += this.buffer;
      this.buffer = '';
      this.notify();
    }
    if (this.rafHandle !== null) {
      this.cancelRaf(this.rafHandle);
      this.rafHandle = null;
    }
    this.lastTickMs = null;
  }

  /**
   * Tear down the pacer. Drops the buffer (use flush() first if you want
   * pending chars rendered). Cancels RAF + unsubscribes.
   */
  destroy(): void {
    if (this.destroyed) return;
    this.destroyed = true;
    if (this.rafHandle !== null) {
      this.cancelRaf(this.rafHandle);
      this.rafHandle = null;
    }
    this.subscriber = null;
    this.buffer = '';
    this.lastTickMs = null;
  }

  /** Returns true if destroy() has been called. */
  isDestroyed(): boolean {
    return this.destroyed;
  }

  // -- private --

  private ensureRaf(): void {
    if (this.rafHandle !== null || this.destroyed) return;
    this.rafHandle = this.raf((ts) => this.tick(ts));
  }

  private tick(timestamp: number): void {
    this.rafHandle = null;
    if (this.destroyed) return;

    if (this.lastTickMs === null) {
      this.lastTickMs = timestamp;
      // No elapsed time yet — schedule the next tick and wait. Avoids
      // emitting a huge first chunk on the very first frame.
      if (this.buffer.length > 0) this.ensureRaf();
      return;
    }

    const elapsedMs = Math.max(0, timestamp - this.lastTickMs);
    this.lastTickMs = timestamp;

    // chars-to-drain = cps * (elapsedMs / 1000); always at least 1 if
    // anything is buffered (so we don't stall on under-budget ticks).
    let toDrain = Math.floor((this.cps * elapsedMs) / 1000);
    if (toDrain === 0 && this.buffer.length > 0) toDrain = 1;

    if (toDrain >= this.buffer.length) {
      // Drain everything; do not schedule another tick (buffer empty).
      if (this.buffer.length > 0) {
        this.rendered += this.buffer;
        this.buffer = '';
        this.notify();
      }
      // Reset lastTickMs so the next enqueue starts fresh.
      this.lastTickMs = null;
      return;
    }

    this.rendered += this.buffer.slice(0, toDrain);
    this.buffer = this.buffer.slice(toDrain);
    this.notify();

    // More buffered chars; schedule the next tick.
    if (this.buffer.length > 0) this.ensureRaf();
    else this.lastTickMs = null;
  }

  private notify(): void {
    if (this.subscriber !== null) this.subscriber(this.rendered);
  }
}

/**
 * Heuristic: choose a cps target based on whether the content looks like
 * code (markdown fence open) vs plain prose. Code: 60 cps (slower, easier
 * on the eyes for monospace blocks). Prose: 100 cps (faster, closer to
 * human read rate).
 */
export function pickCps(initialChunk: string): number {
  if (initialChunk.includes('```')) return 60;
  return 100;
}
