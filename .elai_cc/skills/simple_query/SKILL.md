---
name: simple_query
type: application
version: 1.0.0
description: Answer a short factual question with no planning, no decomposition, no audit cycle.
triggers: what is, define, explain briefly, lookup, quick question, simple question
inputs: question_text
outputs: answer_text
scope: query
tags: [simple, factual, no-plan, fast-path]
estimated_loc: 0
estimated_minutes: 1
---

Application skill — type=application. Distinct from the harness `.claude/skills/`
surface; this skill is consumed by the orchestrator's `task_runner` pre-classify
hook for routing real user prompts, not by Claude Code itself at session boot.

Use when:
  - the user asks one short factual question (under ~25 words)
  - no code edits, no file writes, no multi-step reasoning are required
  - a single sentence or short paragraph answers it
  - the answer fits in the model's training data OR a single targeted grep

Do NOT use when the question implies design tradeoffs, multi-file impact,
architecture choice, or anything that would benefit from web grounding.

Output: a one-paragraph plaintext answer, no scaffolding, no audit, no SSB
plan rows. The orchestrator emits a single `skill_triggered` SSB row when this
skill is selected so the dashboard sees the routing decision.
