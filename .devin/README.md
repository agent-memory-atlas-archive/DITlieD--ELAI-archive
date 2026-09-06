# Devin stay-in-place migration (2026-07-13)

Source of truth remains `.claude/` + `.agents/skills/` + root `CLAUDE.md` / `AGENTS.md`.
This `.devin/` layer is a thin adapter so Devin sessions at `/workspace/ELAI` work without rewriting the Claude workflow.

## What Devin loads

| Piece | Location | Notes |
|-------|----------|-------|
| Always-on rules | `CLAUDE.md`, `AGENTS.md` | Auto via `read_config_from.claude` |
| Skills | `.agents/skills/`, `.claude/skills/` | Devin reads both |
| Custom subagents | `.devin/agents/*/AGENT.md` | 16 profiles; bodies point at `.claude/agents/*.md` |
| MCP | `.devin/config.json` + root `.mcp.json` | Mirrored servers: local-rag, codegraph-elai, canvas, gpt-bridge, pod-lifecycle |
| Hooks (Devin) | `.devin/hooks.v1.json` | Matchers use `exec`/`edit`/`write`/`read` |
| Hooks (Claude + dual) | `.claude/settings.json`, `settings.local.json` | Matchers dual-mapped (Claude + Devin names) |

## Canonical agent list (16 Devin profiles)

auditor, context-compiler, dashboard-smoker, executor, fixer, impact-mapper,
integration-explorer-elai, memory-writer, paper-ingestor, prompt-engineer,
scope-planner-elai, sidecar-health, spec-grounder, synthesizer-elai,
test-attributer, wiring-contract-validator

Extra Claude agents (idea-*, personas, designer, etc.) still exist as
`.claude/agents/*.md` and can be dispatched by skill prompt without a Devin profile.

## Session root

Always start Devin with CWD = `/workspace/ELAI` so project root detection finds
`.devin/`, `.claude/`, `AGENTS.md`, and skills.

## User-level tweaks (`%APPDATA%\devin\config.json`)

- `attribution: false` — no Co-Authored-By / Generated-with-Devin trailers
- `include_gitignored_files: true` — @-complete memory-bank / gitignored workflow files

## Known gaps (not fixed in stay-in-place)

1. **PreCompact** — Claude hook; no Devin equivalent listed in hooks docs. RAG precompact autosave only runs in Claude unless reimplemented on SessionEnd.
2. **SubagentStop** — Claude-only event; canvas activity on subagent stop may miss in pure Devin.
3. **Permission dialect** — Claude `Bash(...)` / `mcp__x__y` vs Devin `Exec(...)` / `Mcp(server)`. You will still approve many tools interactively until allow-lists settle.
4. **Hook script stdin schema** — RAG hooks (`pre_tool_check.py` etc.) were written for Claude Code stdin JSON. If they fail-open or no-op under Devin payload shapes, fix the hook scripts — do not remove them blindly.
5. **Models on AGENT.md** (`kimi-k2.7`, `glm-5.2`) — only work if those model IDs exist in your Devin org; otherwise Devin falls back / errors. Adjust frontmatter if needed.
6. **Always-on rule size** — `CLAUDE.md` is very large; Devin recommends thinner rules + skills. Optional later cleanup, not done here.

## How to verify next session

1. Open Devin at `/workspace/ELAI`
2. `/hooks` — confirm dual-mapped hooks load
3. Check MCP servers connected (local-rag required for FSM/GRC)
4. Ask for subagent profile list / spawn `auditor` or `context-compiler` dry-run prompt
