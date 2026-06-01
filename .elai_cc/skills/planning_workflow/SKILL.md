---
name: planning_workflow
type: application
version: 1.0.0
description: Execute a phased plan via the FSM cycle with per-phase audits.
triggers: execute plan, run phase, continue plan, resume plan, drive the fsm, walk the plan, next phase, start phase, ship phase, audit phase, planning workflow
inputs: plan_path, phase_id
outputs: phase_diff, audit_result, ssb_run_log
scope: execution
tags: [planning, fsm, audit, multi-phase, dispatch-only]
estimated_loc: 0
estimated_minutes: 60
---

Application skill — type=application. NAMESPACE COLLISION NOTE: harness
`.claude/skills/planning-workflow/SKILL.md` (hyphen) governs the
Claude-Code-side FSM rules; this application skill is the orchestrator-side
trigger that DRIVES that harness via task_runner. They co-exist; one is the
prompt the LLM reads, the other is the routing decision a real user query
makes.

Use when:
  - a plan file already exists under docs/thefinalglance/{B,C,D,E}-plan/
  - the user asks to run / resume / continue a known phase
  - the FSM is in or near IDLE and the phase passes its pre-gates
  - per-phase audits + memory-writer dispatch are expected

Do NOT use when there is no plan yet (idea_intake / heavy_plan first), or
when the user asks for a quick fix unrelated to any phase (focused_edit).

Output: a sequence of EXECUTING > AUDITING > MEMORIZING FSM transitions,
SSB rows per phase, and an updated `activeContext.md` in memory-bank.
Failures bubble per `.claude/rulebook/bubble-up.md`.
