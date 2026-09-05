---
name: auto-orchestrate
description: General autonomous orchestration for ANY task, not just plan files. Triggers when the user says "orchestrate this autonomously", "auto-orchestrate", "orchestrate:" followed by an arbitrary task, or asks to autonomously coordinate agents on work that is not a plan-file walk. The main model becomes a pure orchestrator: decomposes the task, dispatches rule-bound worker agents with scoped context, mediates questions/blocks through a handoff file, makes the decisions, verifies off-agent, and synthesizes the result.
license: MIT
compatibility: "Designed for Codex. ELAI workflow root."
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

THE ORCHESTRATOR ROLE (binding; grounded in .Codex/rules/main-llm-role.md, corroborated by the 2026 SOTA orchestration survey the user provided)

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
Also resolve ONE handoff file for the run per .Codex/rules/handoff-protocol.md: workflow/handoff/{topic-slug}.md (topic = the problem, not the session). Both files survive context loss; together they are the resume spec.

DISPATCH RULES (binding)

1. RULE INJECTION: every agent prompt begins with the pointer line from .Codex/rules/agent-dispatch-preamble.md:
   "Read F:\.ELAI_workflow\.Codex\rules\agent-dispatch-preamble.md FIRST and obey it as binding. handoff_file=workflow/handoff/{slug}.md; task_id=<id>; scope=<paths>."
   Sub-agents do not reliably load AGENTS.md - the pointer is the guarantee. A dispatch without it is a protocol violation. The preamble bans worker git mutation (the 2026-06-11 reset --hard incident class), bans destructive ops, locks scope, and defines the BLOCKED/handoff protocol and the structured return.
2. SCOPED CONTEXT: a worker receives ONLY what its task needs - the task, relevant file paths, the distilled slice of prior results (orchestrator-summarized), its handoff pointer. Never the full run-state file, never sibling agents' raw output, never failed siblings' tracebacks.
3. MODEL LADDER: haiku = bulk read/summarize; sonnet = implementation/audit/canonical agents (canonical .Codex/agents/*.md always via "Read F:\.ELAI_workflow\.Codex\agents\<name>.md and follow it strictly"); main model = never spawned, it IS the orchestrator.
4. PARALLEL CAP 3 (hard, rate limit). Independent tasks dispatch in one batch up to 3; the rest queue.
5. FLEET ANNOUNCEMENT: more than 2 agents total in the run = announce count + models + rough token cost ONCE up front, get the user's yes, then run autonomously. Re-announce only if the fleet must grow materially beyond the announced plan.
6. EXECUTION/EVALUATION SEPARATION: the agent that verifies a task is never the agent that did it, and workers never author the checks that judge their own output (wiring-enforcement R8 logic at orchestration grain).

HANDOFF MEDIATION LOOP (the question/decision channel)

Worker hits a fork, a blocker, a banned op, or budget pressure -> it APPENDS an entry (SUMMARY / WORK DONE / FILES TOUCHED / QUESTION FOR ORCHESTRATOR / HOW TO PICK UP) to the run's handoff file and returns "BLOCKED: <question>".
Orchestrator then: (1) reads the entry - the entry, not the whole file; (2) DECIDES - grounded in run-state evidence, a quick targeted read, or WebSearch; only tripwire-grade items go to the user; (3) appends a DECISION block under the entry (choice + why + exact instruction for the successor); (4) dispatches the next agent with the handoff path + "read entry <N> and its DECISION first". 
Update run-state DECISIONS with every mediation. The handoff file's CURRENT STATUS line is overwritten as the run progresses (handoff-protocol living fields).

AUTONOMY ENVELOPE + LOOP PREVENTION

Autonomous by default: no "should I proceed" questions. Break autonomy ONLY for: (a) irreversible ops (destructive git, rm -rf outside scratch, deploys/publishes, pod ops with cost - exact command + Y/N, default no); (b) genuine scope change vs the user's ask; (c) the run cap: 30 agent dispatches without a user check-in; (d) stall: a task BLOCKED twice on the same question, or run-state unchanged after 3 consecutive returns (anti-spiral - stop, reassess via mcp_think, escalate if still stuck).
Re-dispatch cap: max 2 re-dispatches per task id with materially the same input; a third identical attempt is banned - change the decomposition or escalate.
Monotonic progress check at every return: if nothing in run-state changed, that return was waste - diagnose before dispatching anything else.

VERIFICATION BEFORE DONE (no rubber-stamping)

A task flips to DONE only on off-worker evidence: an independent checker agent re-runs the deterministic check / reads the artifact / executes the command, OR a deterministic tool output (build, validator, diff) confirms it. The worker's own "STATUS: DONE" is necessary, never sufficient. FSM rules still apply when project code is touched (mcp_fsm_get_state before mutating dispatch phases; auditor-type agent is the only test runner).

INTEGRATION GATE (mandatory; wiring-enforcement.md R10) — per-worker green is necessary, NEVER sufficient

Each worker tests its OWN slice, in isolation, on the EASY path (the lenient peer/provider, a single-turn input, a run too short to fire secondary paths). So defects at the SEAMS BETWEEN workers pass every worker's gate and ship. The whole orchestration model (isolated, GC'd worker contexts) is exactly what creates this blindness — bigger contexts do not fix it; an explicit integration stage does. Therefore, NON-OPTIONALLY:
- DECOMPOSE IN VERTICAL SLICES: a unit ships producer + its consumer + a smoke through the REAL entrypoint, together. "Add fn X" with the call site deferred is rejected at decomposition — it becomes wire-dark.
- THREAD CROSS-AGENT CONTRACTS: when parallel workers touch a shared resource (a catalog, schema, config, a prune/denylist), the contract is a REQUIRED INPUT passed to every worker that touches it — never an implicit assumption one worker makes and another silently undoes (Liberty: a sync worker re-added models a fix worker had pruned, because the prune was not a threaded contract).
- WIRE-CHECK EVERY CHANGESET off-agent: the auditor re-runs the language-appropriate producer-without-consumer checker (.rs -> .Codex/tools/production_caller_check.py; .py -> .Codex/tools/python_wire_check.py); a DARK symbol with no WIRE-DARK marker is a BLOCK. Executors self-run it before DONE but are never the authority.
- RUN AN INTEGRATION SMOKE after every 2-3 units AND before the final report: an INDEPENDENT agent drives the REAL entrypoint end-to-end with a HARD input — the STRICT peer/provider, a multi-turn / multi-tool history, a run long enough to fire secondary paths (compaction, fallback, resume). Toy single-turn happy-path smokes do not count. This is the stage that catches the seam defects; it is part of the task graph from the START, not bolted on at the end. (Liberty 2026-06-16: 7 cross-seam defects each passed their unit gate and surfaced ONLY here.)

GIT DISCIPLINE

Workers: read-only git, enforced by the preamble. Orchestrator: at most ONE commit at the very end, only if the user asked or commit-cadence says this is a completed shipped unit; never per-task commits; destructive git only with explicit user Y/N. A confusing working-tree state is diagnosed (git status/diff/log via a reader agent), never "repaired" with resets.

WRAP

Final report: objective, what shipped (real paths), task graph outcome counts, decisions made on the user's behalf, what was verified and how, open items. Overwrite the handoff file's CURRENT STATUS + append the closing entry. Update run-state to terminal. If ELAI features changed, run the feature-map upsert (docs/thefinalglance/ELAI-FEATURE-MAP.md, gate-verdict-grounded). Leave the FSM in IDLE.

WHEN NOT TO INVOKE

- plan-file walks (orchestrate/b-plan-runner) and paper/repo intake (paper-to-plan) - route there.
- conversational questions, single reads, one-shot fixes - no orchestration tax.
- the user explicitly wants the main model to do the work itself ("you do it yourself") - main-llm-role exception applies.
