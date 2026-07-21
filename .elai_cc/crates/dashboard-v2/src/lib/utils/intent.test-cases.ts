// ============================================================
// intent.test-cases.ts — Precision-verification table for the
// classifier in intent.ts. No test runner is wired into this
// crate (package.json has svelte-check + tsc, no vitest), so
// these cases are exported as a static const for future runners
// and for human eyeball-verification.
//
// Ordering rule: EXPLORE_CODE patterns must match before RESEARCH
// to keep code-grep questions ("what calls X?") off the Oracle gate.
// ============================================================

import type { Intent } from './intent';

export interface IntentTestCase {
  input: string;
  expected: Intent;
  rationale: string;
}

export const INTENT_TEST_CASES: IntentTestCase[] = [
  // Slash bypass — must short-circuit before any regex layer.
  { input: '/ask anything',                          expected: 'RESEARCH',     rationale: 'slash bypass: /ask routes to RESEARCH' },
  { input: '/research how transformers work',        expected: 'RESEARCH',     rationale: 'slash bypass: /research routes to RESEARCH' },
  { input: '/explore Token::validate',               expected: 'EXPLORE_CODE', rationale: 'slash bypass: /explore routes to EXPLORE_CODE' },
  { input: '/status',                                expected: 'EXPLORE',      rationale: 'slash bypass: /status stays EXPLORE (meta info)' },

  // Abort / config / steer / fix / intervene — pre-existing patterns, must still match.
  { input: 'stop',                                   expected: 'ABORT',        rationale: 'ABORT pattern at start of line' },
  { input: 'switch to qwen-3b',                      expected: 'CONFIG',       rationale: 'CONFIG pattern: switch to <X>' },
  { input: 'fix the bug in the auth handler',        expected: 'FIX',          rationale: 'FIX pattern: fix/bug keywords' },

  // Code-context inference (non-slash) — must classify EXPLORE_CODE.
  { input: 'what calls foo()?',                      expected: 'EXPLORE_CODE', rationale: 'code-context: "what calls" + identifier with parens' },
  { input: 'who calls Token::validate?',             expected: 'EXPLORE_CODE', rationale: 'code-context: "who calls" + Rust path identifier' },
  { input: 'how does the auth module validate tokens?', expected: 'EXPLORE_CODE', rationale: 'code-context: "the auth module" anchors to codebase' },
  { input: 'explain this function',                  expected: 'EXPLORE_CODE', rationale: 'code-context: "this function" anchors to selection/codebase' },
  { input: 'show me uses of streamChat',             expected: 'EXPLORE_CODE', rationale: 'code-context: "show me uses of" + identifier' },
  { input: 'find all callers of run_research',       expected: 'EXPLORE_CODE', rationale: 'code-context: "find all callers"' },
  { input: 'inspect the FSM transition table',       expected: 'EXPLORE_CODE', rationale: 'code-context: "inspect" verb' },

  // Research inference (non-slash) — must classify RESEARCH.
  { input: "what's the latest pytorch version?",     expected: 'RESEARCH',     rationale: 'knowledge-context: "latest <X> version"' },
  { input: 'research how transformers work',         expected: 'RESEARCH',     rationale: 'knowledge-context: "research <topic>"' },
  { input: 'look up the latest tauri release notes', expected: 'RESEARCH',     rationale: 'knowledge-context: "look up" + "latest"' },
  { input: 'what is BLAKE3?',                        expected: 'RESEARCH',     rationale: 'knowledge-context: "what is <CapTerm>" with no code identifier' },
  { input: 'compare flash-attn vs sdpa',             expected: 'RESEARCH',     rationale: 'knowledge-context: "compare X vs Y"' },
  { input: 'find recent papers on speculative decoding', expected: 'RESEARCH', rationale: 'knowledge-context: "find recent <topic>"' },

  // B-24 Phase 4I Group C8: media-validation routing — verb + media-noun pairs.
  { input: 'validate image at /tmp/hero.png',        expected: 'MEDIA_VALIDATE', rationale: 'media: validate + image' },
  { input: 'check screenshot of login form',         expected: 'MEDIA_VALIDATE', rationale: 'media: check + screenshot' },
  { input: 'render preview of the landing page',     expected: 'MEDIA_VALIDATE', rationale: 'media: render preview noun-phrase' },
  { input: 'run visual feedback loop on these images', expected: 'MEDIA_VALIDATE', rationale: 'media: "visual feedback"' },
  { input: 'inspect thumbnails for the latest run',  expected: 'MEDIA_VALIDATE', rationale: 'media: inspect + thumbnails' },
  // B-24 Phase 4I Group C8 audit WARN 4 — coverage for the 4th media
  // pattern at intent.ts:49 (`(media|image|screenshot) validat(e|ion|or)`).
  // The other three media patterns had cases above; this one was
  // untested, leaving a regression hole if someone changed the regex.
  { input: 'run the screenshot validator on step 7', expected: 'MEDIA_VALIDATE', rationale: 'media: screenshot + validator (pattern 4)' },
  { input: 'trigger media validation for the design comp', expected: 'MEDIA_VALIDATE', rationale: 'media: media + validation (pattern 4)' },

  // Ambiguous / fallback.
  { input: '?',                                      expected: 'AMBIGUOUS',    rationale: 'too short, no verb, no pattern hit' },
  { input: 'huh',                                    expected: 'AMBIGUOUS',    rationale: 'too short, no pattern hit' },

  // Generic task fallback (>8 chars, no pattern hit).
  { input: 'implement a parser for the new manifest format', expected: 'TASK', rationale: 'long imperative, no pattern hit, length-based TASK fallback' },
];

// ------------------------------------------------------------------
// Counterfactual gate cases — drive the chip-firing decision.
// These are NOT direct classifier inputs; they document the gate's
// firing rule (see commands.svelte.ts submit() + needsIntentChip in
// intent.ts). Used for manual verification only.
//
// Esc semantics (slice-F): Esc = cancel + restore input + refocus for
// ALL chipped intents. The old rejectResearchChip "reply locally"
// path is retired; RESEARCH Esc now also cancels + restores.
// ------------------------------------------------------------------
export const GATE_FIRE_CASES: { input: string; chipFires: boolean; reason: string }[] = [
  { input: '/research foo',                          chipFires: false, reason: 'slash prefix bypasses the gate entirely' },
  { input: '/ask foo',                               chipFires: false, reason: 'slash prefix bypasses the gate entirely' },
  { input: '?',                                      chipFires: false, reason: 'classifier returns AMBIGUOUS, not a chipped intent' },
  { input: 'what calls foo()?',                      chipFires: false, reason: 'classifier returns EXPLORE_CODE, not a chipped intent' },
  { input: "what's the latest pytorch version?",     chipFires: true,  reason: 'classifier returns RESEARCH, needsIntentChip(RESEARCH)=true' },
  { input: 'implement a parser for the new manifest format', chipFires: true, reason: 'classifier returns TASK, needsIntentChip(TASK)=true' },
  { input: 'fix the bug in the auth handler',        chipFires: true,  reason: 'classifier returns FIX, needsIntentChip(FIX)=true' },
  { input: 'stop',                                   chipFires: true,  reason: 'classifier returns ABORT, needsIntentChip(ABORT)=true' },
  { input: 'validate image at /tmp/hero.png',        chipFires: false, reason: 'classifier returns MEDIA_VALIDATE, not a chipped intent' },
];
