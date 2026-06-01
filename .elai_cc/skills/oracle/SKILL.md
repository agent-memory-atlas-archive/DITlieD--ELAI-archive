---
name: oracle
type: application
version: 1.0.0
description: Answer a deep factual question via web research with cited sources.
triggers: research, web search, deep question, citations, look up, find sources, oracle, ask the web, current api, latest, what does the doc say
inputs: question_text
outputs: cited_answer, source_list
scope: research
tags: [research, web-grounded, citations, read-only, no-fsm]
estimated_loc: 0
estimated_minutes: 10
---

Application skill — type=application. NAMESPACE COLLISION NOTE: harness
skill `.claude/skills/oracle/SKILL.md` is the Claude-Code-side interactive
oracle. The application oracle here is the orchestrator-side router target
for prompts that need WebSearch + WebFetch grounding before answering.

Use when:
  - the user asks about current library/API behavior, version-specific features
  - the answer requires citing official docs, ecosystem blogs, or papers
  - training-data memory alone is insufficient (post-cutoff topic, deprecation,
    ABI change, etc.)
  - no code edits, no plan, no FSM cycle

Do NOT use when the answer is in this repo's source (focused_edit/simple_query),
when the question is about a paper that needs full ingestion (paper_ingest),
or when an SDK needs interface-mapping (source_ingest).

Output: a one-paragraph answer with inline citations to URLs, plus a
`Sources:` list. Read-only — works in any FSM state without transition.
