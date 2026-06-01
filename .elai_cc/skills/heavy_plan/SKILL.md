---
name: heavy_plan
type: application
version: 1.0.0
description: Decompose a multi-week SaaS or app feature into a phased, audited plan.
triggers: plan, design, architect, decompose, saas, dashboard, app, feature, system, build me, build a, multi-week, project structure, breakdown, roadmap
inputs: feature_brief, project_root
outputs: phased_plan, decomposition_tree, grounding_log
scope: planning
tags: [heavy, planning, decomposition, multi-phase, audit-required]
estimated_loc: 2000
estimated_minutes: 240
---

Application skill — type=application. Used by the orchestrator pre-classify
hook when the prompt implies a multi-phase, multi-file build that needs a
plan tree (Concept > Phase > Step > BabyStep) rather than a one-shot edit.

Use when:
  - the user describes a feature / app / SaaS / module with > 3 subsystems
  - the work spans days or weeks, not hours
  - paper / web grounding is required at the design step
  - phased audit + memory-bank writes are part of the deliverable

Do NOT use for single-file edits, factual lookups, or pre-existing-plan resumes
(those resolve to focused_edit, simple_query, planning_workflow respectively).

Output: a Concept-Phase-Step-BabyStep decomposition tree under
`workflow/sessions/<sid>/concepts/`, a grounding log under
`workflow/reasoning/`, and the first phase queued for execution.
