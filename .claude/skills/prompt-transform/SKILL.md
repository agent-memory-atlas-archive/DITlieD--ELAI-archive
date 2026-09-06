---
name: prompt-transform
description: Reformats user prompts before implementation. Triggers on any task implying code changes, implementation, multi-step work, or starting/resuming a plan phase. This includes "start phase X", "implement X", "continue X", any plan phase execution request, AND bug reports phrased as questions ("doesn't work", "isn't saving", "is X wired", "broken", "why does X", "fix this"). Diagnostic phrasing does not exempt from protocol.
license: MIT
compatibility: "Designed for Claude Code. Works on Codex with explicit $prompt-transform invocation."
metadata:
  version: "1.0.0"
  author: "elai"
---

Prompt transformation

Trigger: any task implying code changes, implementation, multi-step work, or starting/resuming a plan phase. Bug reports count even when phrased as questions — trigger verbs include: "doesn't", "isn't", "broken", "wired", "fix", "why does", "why doesn't", "does X save/persist/work". If the answer requires reading the codebase to find out whether something is wired or correct, it is a code-change task and triggers this skill.
Skip: messages starting with !. Pure yes/no replies. Purely informational questions where the answer comes from training or documentation, not from reading the codebase.
Tie-breaker: if unsure whether the task is informational or implementation, treat it as implementation. Over-spawning context-compiler is cheap; skipping it is expensive (re-do).

1. Spawn the prompt-engineer agent (.claude/agents/prompt-engineer.md) with:
   - task: the raw task description
   - session_id: the current session ID (if in a plan resume or phase start flow)
   If session_id is provided, the agent reads the context package from workflow/sessions/{session_id}/context-package.md (written by context-compiler). Do NOT pass the context package content inline — the agent reads it directly to save main model tokens.
   For ad-hoc tasks without a session, omit session_id and pass context inline as before.
2. Show result. Ask: "Reformatted. Approve or edit:"
3. Wait for confirmation. Do not proceed until the user approves.
