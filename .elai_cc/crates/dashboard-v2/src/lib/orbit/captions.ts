// orbit/captions.ts
// Pure deterministic plain-language caption generator.
// (FsmState, GrcStep, BubbleUpAction?) -> human-readable string.
//
// Source of truth: HANDOFF-orbit.md:340-358 (the 14-entry mapping).
// This is rendered below the active baby step. No LLM. No randomness.

import type { CaptionContext, BubbleUpAction, GrcStep } from './types';
import type { FsmState } from '../api/types';

// -----------------------------------------------------------------
// CAPTION_TABLE — declarative mapping. Order matters: bubble_action
// overrides take precedence; otherwise the first matching (fsm, grc)
// row wins. fsm='*' is wildcard.
// -----------------------------------------------------------------
type FsmMatch = FsmState | '*';
type GrcMatch = GrcStep | '*';

interface CaptionRow {
  fsm: FsmMatch;
  grc: GrcMatch;
  bubble?: BubbleUpAction;
  template: (ctx: CaptionContext) => string;
}

export const CAPTION_TABLE: ReadonlyArray<CaptionRow> = Object.freeze([
  // bubble-up overrides (highest priority)
  { fsm: '*', grc: '*', bubble: 'research_alternative', template: () => "That didn't work — looking for a different approach" },
  { fsm: '*', grc: '*', bubble: 'redecompose_step', template: () => 'Breaking this step down further' },
  { fsm: '*', grc: '*', bubble: 'replan_phase', template: () => 'Rethinking this whole phase' },
  { fsm: '*', grc: '*', bubble: 'replan_concept', template: () => 'Rethinking this whole concept' },
  { fsm: '*', grc: '*', bubble: 'diagnose_and_retry', template: () => "Figuring out what just went wrong" },
  { fsm: '*', grc: '*', bubble: 'escalate', template: () => 'I need a human to look at this' },

  // RESEARCHING
  { fsm: 'RESEARCHING', grc: 'observe',     template: (c) => `Reading how ${c.research_topic ?? c.baby_step_label} actually works` },
  { fsm: 'RESEARCHING', grc: 'hypothesize', template: (c) => `Forming a hypothesis about ${c.research_topic ?? c.baby_step_label}` },
  { fsm: 'RESEARCHING', grc: 'verify',      template: (c) => `Cross-checking sources on ${c.research_topic ?? c.baby_step_label}` },
  { fsm: 'RESEARCHING', grc: 'converge',    template: (c) => `Wrapping up research on ${c.research_topic ?? c.baby_step_label}` },
  { fsm: 'RESEARCHING', grc: 'status',      template: (c) => `researcher fetching: ${c.research_topic ?? c.baby_step_label}` },
  { fsm: 'RESEARCHING', grc: '*',           template: (c) => `Researching ${c.research_topic ?? c.baby_step_label}` },

  // PLANNING
  { fsm: 'PLANNING', grc: 'plan_action', template: (c) => `Planning how to do ${c.baby_step_label}` },
  { fsm: 'PLANNING', grc: 'observe',     template: (c) => `Reading existing code around ${c.baby_step_label}` },
  { fsm: 'PLANNING', grc: 'status',      template: (c) => `designer drafting: ${c.baby_step_label}` },
  { fsm: 'PLANNING', grc: '*',           template: (c) => `Planning ${c.baby_step_label}` },

  // FEASIBILITY_PROBE
  { fsm: 'FEASIBILITY_PROBE', grc: 'observe', template: (c) => `Checking if ${c.baby_step_label} is actually possible` },
  { fsm: 'FEASIBILITY_PROBE', grc: 'act',     template: (c) => `Testing feasibility: ${c.baby_step_label}` },
  { fsm: 'FEASIBILITY_PROBE', grc: '*',       template: (c) => `Checking if ${c.baby_step_label} is doable` },

  // TESTING_FIRST
  { fsm: 'TESTING_FIRST', grc: 'plan_action', template: (c) => `Writing a test that should fail — to prove the fix is real` },
  { fsm: 'TESTING_FIRST', grc: 'act',         template: (c) => `Writing the failing test for ${c.baby_step_label}` },
  { fsm: 'TESTING_FIRST', grc: 'verify',      template: (c) => `Running the test — expecting red` },
  { fsm: 'TESTING_FIRST', grc: '*',           template: (c) => `Setting up the test for ${c.baby_step_label}` },

  // EXECUTING
  { fsm: 'EXECUTING', grc: 'act',     template: (c) => `Writing the code for ${c.baby_step_label}` },
  { fsm: 'EXECUTING', grc: 'verify',  template: (_c) => `Checking the code I just wrote` },
  { fsm: 'EXECUTING', grc: 'observe', template: (c) => `Looking at how ${c.baby_step_label} fits in` },
  { fsm: 'EXECUTING', grc: 'converge', template: (c) => `Finishing up ${c.baby_step_label}` },
  { fsm: 'EXECUTING', grc: 'status',  template: (c) => `executor running: ${c.baby_step_label}` },
  { fsm: 'EXECUTING', grc: '*',       template: (c) => `Working on ${c.baby_step_label}` },

  // AUDITING
  { fsm: 'AUDITING', grc: 'observe', template: (_c) => `Re-reading the change with auditor's eye` },
  { fsm: 'AUDITING', grc: 'verify',  template: (_c) => `Double-checking against the safety rules` },
  { fsm: 'AUDITING', grc: 'status',  template: (c) => `auditor verifying: ${c.baby_step_label}` },
  { fsm: 'AUDITING', grc: '*',       template: (c) => `Auditing ${c.baby_step_label}` },

  // MEMORIZING
  { fsm: 'MEMORIZING', grc: '*', template: (_c) => 'Saving what I learned for next time' },

  // COMPLETE
  { fsm: 'COMPLETE', grc: '*', template: (_c) => 'Done.' },

  // CONFIGURING
  { fsm: 'CONFIGURING', grc: '*', template: (_c) => 'Configuring before we start' },

  // IDLE
  { fsm: 'IDLE', grc: '*', template: (_c) => 'Waiting for a task' },

  // global fallback (must be last, matches anything)
  { fsm: '*', grc: '*', template: (c) => `Working on ${c.baby_step_label}` },
]);

// -----------------------------------------------------------------
// Pure caption resolver. First matching row wins.
// -----------------------------------------------------------------
export function captionFor(ctx: CaptionContext): string {
  for (const row of CAPTION_TABLE) {
    // bubble override path
    if (row.bubble) {
      if (ctx.bubble_action === row.bubble) return row.template(ctx);
      continue;
    }
    // wildcard or exact match
    const fsmOk = row.fsm === '*' || row.fsm === ctx.fsm;
    const grcOk = row.grc === '*' || row.grc === ctx.grc_step;
    if (fsmOk && grcOk && !ctx.bubble_action) return row.template(ctx);
  }
  // unreachable due to global fallback row, but TS is happier with this:
  return `Working on ${ctx.baby_step_label}`;
}
