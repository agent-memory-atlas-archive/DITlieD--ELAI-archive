---
name: idea_intake
type: application
version: 1.0.0
description: Intake a raw user idea, classify it, and frame it as a build brief.
triggers: idea, brief, intake, frame, classify, hirer, the boss wants, we need, i want, here is the brief, business idea, app idea
inputs: raw_idea_text
outputs: framed_idea, project_class, tradeoff_card
scope: framing
tags: [intake, framing, classification, layer-a, no-decomposition-yet]
estimated_loc: 0
estimated_minutes: 15
---

Application skill — type=application. NAMESPACE COLLISION NOTE: a harness
skill of the same name lives at `.claude/skills/idea-intake/SKILL.md` (note
the hyphen vs underscore difference). The orchestrator discriminates by
discovery root precedence AND by frontmatter `type:` field — application
intake runs server-side via the `idea_framing` module, the harness flavour
is Claude-Code-side.

Use when:
  - the user pastes a raw idea or business brief without structure
  - the project class (web SaaS, mobile, ML, sidecar, etc) is not yet known
  - tradeoffs have not been surfaced
  - downstream you would have spawned the framer + critic loop anyway

Do NOT use when the user already supplied a framed `projectBrief.md` or
when the task is mid-execution (resume goes through planning_workflow).

Output: `framed-idea.toml` under `workflow/sessions/task_seq_<seq>/`,
plus a one-paragraph project-class summary written via SSB
`idea_framing_started` + `idea_framed` rows. Decomposition is deferred to
the next skill (typically heavy_plan).
