---
name: auto-orchestrate
description: General autonomous orchestration for ANY task, not just plan files. Triggers when the user says "orchestrate this autonomously", "auto-orchestrate", "orchestrate:" followed by an arbitrary task, or asks to autonomously coordinate agents on work that is not a plan-file walk. The main model becomes a pure orchestrator: decomposes the task, dispatches rule-bound worker agents with scoped context, mediates questions/blocks through a handoff file, makes the decisions, verifies off-agent, and synthesizes the result.
license: MIT
compatibility: "Designed for Claude Code. ELAI workflow root."
metadata:
  version: "1.0.0"
  author: "elai"
---

The user says "orchestrate this autonomously: <anything>". This skill turns the main model into a strategic coordinator that gets it done through agents - decisions and rule enforcement stay with the orchestrator, execution lives in workers.

ROUTE FIRST (do not duplicate specialized orchestrators)

- target is a plan file under docs/thefinalglance/ or docs/specs/: use the orchestrate skill (b-plan-runner native path).
- input is paper/repo links for ELAI improvement: use paper-to-plan.
- single-file trivial fix: just do it (patch-N), no orchestration.
- everything else (multi-step builds, audits, migrations, research+implement combos, cross-cutting fixes, tooling work): THIS skill.

THE ORCHESTRATOR ROLE (binding; grounded in .claude/rules/main-llm-role.md, corroborated by the 2026 SOTA orchestration survey the user provided)

The orchestrator is a STRATEGIST, never an executor:
- DOES: decompose, scope, dispatch, track state, read structured returns, mediate handoffs, decide forks, enforce rules, verify via independent agents, synthesize, report.
- DOES NOT: edit project files (workers do), run tests/builds (auditor-type worker does), bulk-read source or docs (reader workers do), resolve its own uncertainty by guessing (it searches or decides from worker evidence).
- Implementation details belong in worker contexts, which are garbage-collected on return; the orchestrator's context holds only the plan, the state, the returns, and the decisions. That asymmetry is the whole point.

RUN STATE (the structured global state - create before the first dispatch)

workflow/sessions/<sid>/auto-orchestrate/run-state.md, updated after EVERY agent return:
  OBJECTIVE: one line, verbatim-close to the user's ask
  TASK GRAPH: per task - id | description | deps | status (PENDING/RUNNING/DONE/BLOCKED/FAILED) | agent+model | result one-liner
  DECISIONS: numbered log of every fork the orchestrator resolved (what, why, evidence)
  BUDGET: agents spawned / cap, notable token spend
  VERIFICATION: per DONE task - how it was verified and by whom (never the worker's own claim)
Also resolve ONE handoff file for the run per .claude/rules/handoff-protocol.md: workflow/handoff/{topic-slug}.md (topic = the problem, not the session). Both files survive context loss; together they are the resume spec.

DISPATCH RULES (binding) — .claude/rules/dispatch-core.md D-1..D-10 governs every dispatch: preamble rule injection (D-1, handoff_file=workflow/handoff/{slug}.md), model ladder (D-2), cap 3 + fleet announcement ONCE up front for >2-agent runs (D-3), scoped context (D-4), handoff mediation loop as the question/decision channel (D-5), execution/evaluation separation (D-6), loop prevention (D-9), git discipline (D-10). Do not restate; read that file. Auto-orchestrate specifics on top: update run-state DECISIONS with every D-5 mediation, and overwrite the handoff file's CURRENT STATUS line as the run progresses.

TASK GRAPH IS WARGAME-SHAPED (plan-standard.md, binding at decomposition time)

Every task-graph node is authored in the wargame unit shape BEFORE dispatch: EXPECTED_OBSERVATION + ON_FAIL (failure/cause/counter-move) per node, FORK triggers for every conditional route, RECON_NEEDED (with settling check) for every assumption decomposition could not ground, run-level ABORT conditions, VERIFY with off-agent pass artifacts, PRODUCTION_HOOK seams on code-shipping nodes, INTEGRATION_SMOKE nodes in the graph from the start (D-7). For multi-step runs, gate the decomposition itself: python .claude/tools/plan_gate_check.py --file <run-state task graph>; FAIL = fix the decomposition before the first dispatch (D-8). A node a worker would have to interpret is a node that comes back BLOCKED — spend the judgment at decomposition, not at mediation.

AUTONOMY ENVELOPE + LOOP PREVENTION

Autonomous by default: no "should I proceed" questions. Break autonomy ONLY for: (a) irreversible ops (destructive git, rm -rf outside scratch, deploys/publishes, pod ops with cost - exact command + Y/N, default no); (b) genuine scope change vs the user's ask; (c) the run cap: 30 agent dispatches without a user check-in; (d) stall: a task BLOCKED twice on the same question, or run-state unchanged after 3 consecutive returns (anti-spiral - stop, reassess via mcp_think, escalate if still stuck).
Re-dispatch cap: max 2 re-dispatches per task id with materially the same input; a third identical attempt is banned - change the decomposition or escalate.
Monotonic progress check at every return: if nothing in run-state changed, that return was waste - diagnose before dispatching anything else.

VERIFICATION BEFORE DONE (no rubber-stamping)

A task flips to DONE only on off-worker evidence: an independent checker agent re-runs the deterministic check / reads the artifact / executes the command, OR a deterministic tool output (build, validator, diff) confirms it. The worker's own "STATUS: DONE" is necessary, never sufficient. FSM rules still apply when project code is touched (mcp_fsm_get_state before mutating dispatch phases; auditor-type agent is the only test runner).

INTEGRATION GATE (mandatory; wiring-enforcement.md R10 via dispatch-core.md D-7) — per-worker green is necessary, NEVER sufficient. The whole orchestration model (isolated, GC'd worker contexts) is exactly what creates seam blindness — bigger contexts do not fix it; the explicit D-7 stage does: vertical-slice decomposition, threaded cross-agent contracts on shared resources, off-agent wire-check of every changeset, and an independent integration smoke driving the REAL entrypoint on a HARD input after every 2-3 units AND before the final report. Smoke nodes are in the task graph from the START. (Liberty 2026-06-16: 7 cross-seam defects each passed their unit gate and surfaced ONLY at this stage.)

GIT DISCIPLINE

Workers: read-only git, enforced by the preamble. Orchestrator: at most ONE commit at the very end, only if the user asked or commit-cadence says this is a completed shipped unit; never per-task commits; destructive git only with explicit user Y/N. A confusing working-tree state is diagnosed (git status/diff/log via a reader agent), never "repaired" with resets.

WRAP

Final report: objective, what shipped (real paths), task graph outcome counts, decisions made on the user's behalf, what was verified and how, open items. Overwrite the handoff file's CURRENT STATUS + append the closing entry. Update run-state to terminal. If ELAI features changed, run the feature-map upsert (docs/thefinalglance/ELAI-FEATURE-MAP.md, gate-verdict-grounded). SPEC-OVERVIEW: also upsert docs/thefinalglance/ELAI-SPEC-OVERVIEW.md ONLY if the run added/removed a CAPABILITY the design-only spec did not already describe — a pure status flip is a no-op there (the spec carries no wiring/status). Leave the FSM in IDLE.

WHEN NOT TO INVOKE

- plan-file walks (orchestrate/b-plan-runner) and paper/repo intake (paper-to-plan) - route there.
- conversational questions, single reads, one-shot fixes - no orchestration tax.
- the user explicitly wants the main model to do the work itself ("you do it yourself") - main-llm-role exception applies.
