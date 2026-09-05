---
model: sonnet
tools: Read
---

DESIGNER AGENT — RETIRED

This agent has been retired. Its responsibilities are now distributed across:

- .claude/skills/idea-intake/SKILL.md — the orchestrator that drives the decomposition chain. Replaces the designer's "spawn researcher per node + write decomposition tree" job.

- .claude/agents/idea-framer.md — frames the idea, identifies top-level concepts. Step 1 of the chain.

- .claude/agents/concept-decomposer.md — concept > phases. One per concept, parallel.

- .claude/agents/phase-decomposer.md — phase > steps. One per phase, parallel within concept.

- .claude/agents/step-decomposer.md — step > baby steps. One per step, parallel.

- .claude/agents/babystep-author.md — writes the 10-block detail file per baby step. One per baby step, massively parallel.

- .claude/agents/researcher.md — tree-structured grounded research with mandatory search floors per mode. Spawned by idea-intake at concept level (always) and phase level (conditional on open questions).

If you (the main LLM) were going to spawn this agent, instead invoke the idea-intake skill which orchestrates the chain. If you were spawning the agent to RESCOPE a concept after BubbleUp, invoke the idea-intake skill with mode=rescope and focus_concept={id}.

Reason for retirement: a single designer agent could not maintain quality across 200+ nodes in one context window. Specialized per-level agents with parallel batching produce better-grounded, better-scoped output. See user discussion 2026-05-09.

This file is kept (rather than deleted) so any session that learned the old designer.md path can read this redirect and find the new chain.
