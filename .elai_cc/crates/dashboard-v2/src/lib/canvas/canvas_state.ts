// canvas_state.ts — quinary-state normalisation (DESIGN §6).
//
// The various stores use overlapping status vocabularies (DecompNode uses
// done|active|pending|failed; research uses converged|stage; system-mode
// uses a 6-stage lifecycle). This module is the ONE place those vocabularies
// collapse onto the canonical quinary set so every canvas surface renders
// state identically through NodeStateGlyph.svelte.

/** The canonical five canvas states (DESIGN §6). This .ts module is the
 *  source of truth for the type so plain `tsc` resolves it — a type exported
 *  from a .svelte instance script is invisible to non-svelte importers (it
 *  falls back to the ambient `*.svelte` shim). NodeStateGlyph.svelte imports
 *  this type from here. */
export type QuinaryState = 'done' | 'active' | 'pending' | 'warn' | 'fail';

/** The canonical five states, in display order. */
export const QUINARY_STATES: readonly QuinaryState[] = Object.freeze([
  'done',
  'active',
  'pending',
  'warn',
  'fail',
]);

// Status strings that map to each quinary state. Anything unmatched falls
// through to 'pending' (the safe neutral — silence-means-safe, no false alarm).
const DONE_WORDS = new Set([
  'done',
  'complete',
  'completed',
  'pass',
  'passed',
  'verified',
  'wired',
  'converged',
  'success',
]);
const ACTIVE_WORDS = new Set([
  'active',
  'executing',
  'running',
  'in_progress',
  'building',
  'auditing',
  'researching',
]);
const WARN_WORDS = new Set([
  'warn',
  'warning',
  'stale',
  'drifted',
  'blocked',
  'untrusted',
  'pending_review',
]);
const FAIL_WORDS = new Set([
  'fail',
  'failed',
  'error',
  'broken',
  'rejected',
  'block',
]);

/**
 * Normalise any store status string to a quinary state.
 * Pure + deterministic (no LLM, no randomness) — the B-13 captions pattern.
 */
export function toQuinary(status: string | null | undefined): QuinaryState {
  if (!status) return 'pending';
  const s = String(status).toLowerCase().trim();
  if (DONE_WORDS.has(s)) return 'done';
  if (ACTIVE_WORDS.has(s)) return 'active';
  if (FAIL_WORDS.has(s)) return 'fail';
  if (WARN_WORDS.has(s)) return 'warn';
  return 'pending';
}
