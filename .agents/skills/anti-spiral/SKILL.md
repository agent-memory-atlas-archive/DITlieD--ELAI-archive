---
name: anti-spiral
description: Stagnation detector + persona switcher. Triggers when the same file is edited 3+ times for the same issue, the same hypothesis fails 3+ times in mcp_think, the auditor returns BLOCK twice in a row, or test pass rate plateaus across attempts. Detects 4 stagnation patterns (Spinning, Oscillation, NoDrift, DiminishingReturns) and switches to one of 5 personas (Hacker, Researcher, Simplifier, Architect, Contrarian) to break the loop.
license: MIT
compatibility: "Designed for Codex. Works on Codex with explicit $anti-spiral invocation."
metadata:
  version: "1.0.0"
  author: "elai"
---

Anti-spiral protocol

Trigger conditions (any one):
1. Spinning: same file edited >=3 times for the same BLOCK / error message / fix attempt.
2. Oscillation: code reverted then re-added then reverted (any 2-cycle within a phase).
3. NoDrift: >=15 minutes of effort without any test moving from FAIL to PASS or any BLOCK clearing.
4. DiminishingReturns: test pass rate plateaus or decreases across 2+ EXECUTING -> AUDITING cycles.

When triggered, EXECUTE THIS PROTOCOL — do not freestyle:

STEP 1 — STOP
Stop the current line of attempt. Do not write more code in the current direction.
Call mcp_think(step_type="status", content="anti-spiral triggered: <which-pattern>", topic="anti-spiral") to log the pivot.

STEP 2 — DIAGNOSE
Read the last 5 mcp_think entries on the active topic. Look for:
- Repeated hypotheses (same observation/hypothesis over multiple steps): Spinning.
- Hypotheses that contradict each other across steps: Oscillation.
- Hypotheses with no plan_action that improved verify outcome: NoDrift.
- Diminishing test-pass deltas: DiminishingReturns.

Pick the dominant pattern. (If two are tied, pick Spinning > Oscillation > NoDrift > DiminishingReturns.)

STEP 3 — PERSONA SWITCH
Map pattern to persona:

  Spinning      -> SIMPLIFIER
  Oscillation   -> ARCHITECT
  NoDrift       -> RESEARCHER
  DiminishingReturns -> CONTRARIAN

(Default Hacker persona is what the main model is doing now. Hacker is the one we are pivoting AWAY from.)

Persona definitions — use as the system prompt frame for the next implementation attempt:

HACKER (default — what we already are)
"Get it working. Match the failing test. Move fast. Standard AGENTS.md rules apply."

SIMPLIFIER
"Delete code. The current path is too complex. Re-approach with the smallest possible diff that satisfies the test. If the current implementation is >50 LOC and broken, candidate for total rewrite from scratch with <30 LOC. Question every abstraction; question every helper. Prefer one inline function over a class hierarchy. Pseudo-code first, then code."

ARCHITECT
"Step back. The bug is structural, not local. Read the surrounding subsystem. Re-plan at a higher level. Identify which contract is being violated; identify what the right boundary is. Often: a function in the wrong module; a state machine missing a transition; an interface that should be split. Output a re-design proposal first, then implement after user approval."

RESEARCHER
"Look outside the codebase. Search for prior art: WebSearch, paper-ingestor, similar projects in elai.map. Read the relevant ELAI spec sections (docs/specs/guide/steps/phase-*.txt). Question: 'Has someone solved this exact problem? Where? How?' If we are using a library, READ ITS SOURCE. If a paper is cited, ingest it via paper-ingestor. Only after grounding, attempt fix."

CONTRARIAN
"Challenge the plan's premise. The B-N file or the test we are trying to pass might be wrong. Generate the OPPOSITE hypothesis to whatever the current direction is. If we believe X causes Y, examine if Y causes X. If we believe a fix needs to add code, examine if it needs to delete code. If the test asserts a value, question whether the asserted value is correct. Output: 'The plan assumes X. The test assumes Y. What if X is wrong?'"

STEP 4 — PROMPT REGENERATION
Spawn prompt-engineer agent with the new persona frame prepended to the task description:
  "PERSONA: <persona name>. <persona definition above>. Original task: <original task>."
Pass session_id and the current B-N file. The prompt-engineer respects the persona instruction.

Show the new prompt to user. Ask: "Pivot to <persona> persona. Approved?" (AGENTS.md hard rule: no unsolicited approach changes — user must approve each persona switch.)

If user rejects: ask which persona they prefer or whether to halt and review with them directly.

If user approves: proceed with new attempt. Reset mcp_think topic via mcp_think_reset(topic="anti-spiral") and start fresh.

STEP 5 — TIMEOUT
Cap: max 2 persona switches per phase. If 2 switches don't resolve, stop and write workflow/sessions/{session_id}/SESSION_HANDOFF.md with a "Stuck" section enumerating: original direction, what each persona attempted, why none worked. Surface to user: "Phase blocked by sustained anti-spiral after 2 persona switches. Recommend: human review of B-N feasibility or scope reduction."

LOGGING
Every persona switch writes a row to workflow/sessions/{session_id}/anti-spiral.log:
  <timestamp> | <pattern> | <from-persona> | <to-persona> | <reason-1-line>

EXAMPLE
Symptom: same fn signature edited 4 times in 20 minutes; same error E0277 appears each time.
Pattern: Spinning (same edit, same error).
Pivot: SIMPLIFIER. Prompt: "The current fix is too complex. Delete the helper function we added in the last 4 attempts. Inline its logic. Match the test in <30 LOC."
