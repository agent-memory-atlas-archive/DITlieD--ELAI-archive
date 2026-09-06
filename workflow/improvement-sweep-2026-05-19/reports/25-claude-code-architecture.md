# Claude Code architecture deep-dive

Date: 2026-05-19. Scope: web-grounded survey of Claude Code (the CLI/IDE/desktop/web product whose harness this very research session runs inside), pulled from docs.anthropic.com / code.claude.com, Anthropic engineering interviews, and third-party reverse-engineering analyses. NO local ELAI source was read; comparison points are from prior memory entries only.

## TL;DR

- Claude Code's real moat is NOT the agent loop (a trivial `while(stop_reason != "end_turn") { call; execute_tools; }`) — it's the harness: 7-stage permission pipeline, 6-tier compaction ladder with surgical cache preservation, prompt-cache-stable tool ordering, file-based subagent IPC mailboxes, deterministic-vs-AI substrate split (~1.6% AI logic / 98.4% deterministic infra per reverse-engineering of ~512K LOC).
- Steal verbatim: (1) the 6-tier compaction ladder with `cache_edits` API parameter use; (2) the deny-first permission rule semantics (deny > ask > allow, evaluated first match wins) — far simpler than ELAI's current state-gated PreToolUse; (3) `opusplan` style automatic model switching between plan and execute phases; (4) the `!\`command\`` shell-injection-into-skill-markdown pattern (skills as templated prompts with prewired bash output) which is strictly more powerful than ELAI's current static skill files; (5) live skill directory watching with no restart.
- ELAI already does better: (1) GRC + mcp_think enforced reasoning chain (Claude Code has nothing equivalent — ultrathink is just a keyword); (2) FSM with state-gated tool access and substate-aware hooks (Claude Code permission rules are stateless); (3) cross-language pair drift detection (Rust↔Python sidecar) — Claude Code has no concept of paired files; (4) BSP supersteps / AgentPool (Claude Code agent-teams is experimental and admits "no nested teams", "lead is fixed", "no session resumption for in-process teammates"); (5) RAG with mcp_search_context / topic-scoped memory — Claude Code memory is plain CLAUDE.md + auto-memory MEMORY.md (first 200 lines / 25KB at session start, no vector retrieval).
- Wash: hooks (both have full lifecycle hooks); MCP integration (Claude Code's is more polished + has deferred tool loading; ELAI's is more deeply integrated with FSM); subagents (both have it; different tradeoffs); CLAUDE.md (ELAI has CLAUDE.md too).
- Anti-pattern to NOT steal: the "trust dialog appears AFTER plugin init" pre-trust execution window (root cause of 2 patched CVEs per third-party audit), and the "93% approval rate = move boundary, don't ask why users approve blindly" decision (approval fatigue masking).

## Key sources

URL | what | takeaway
--- | --- | ---
https://code.claude.com/docs/en/overview | Claude Code product overview | 6 surfaces (Terminal/VS Code/JetBrains/Desktop/Web/iOS) all running same engine; CLAUDE.md + settings + MCP shared
https://code.claude.com/docs/en/hooks | Hook reference | ~25 hook events across session/turn/tool/agent/MCP/context/worktree levels; 5 handler types (command/http/mcp_tool/prompt/agent); deny-via-exit-2; JSON output for fine control
https://code.claude.com/docs/en/sub-agents | Subagents | Own context window, custom system prompt, scoped tool access, model selection, description-driven invocation, returns summary only
https://code.claude.com/docs/en/agent-teams | Agent Teams (experimental) | 1 lead + N teammates, shared task list, mailbox messaging, `~/.claude/teams/{name}/config.json` state, tmux/iTerm2 split panes
https://code.claude.com/docs/en/mcp | MCP integration | hundreds of servers, deferred tool loading via tool search, mcp__server__tool namespace
https://code.claude.com/docs/en/settings | settings.json schema | Permission modes, sandbox config, hook config, plugin/MCP allowlists, managed settings via MDM
https://code.claude.com/docs/en/slash-commands and /skills | Custom commands & Skills | Commands MERGED INTO skills (Skill.md + YAML frontmatter); `!\`command\`` dynamic context injection; `context: fork` runs in subagent; 1536-char description cap; live-reload
https://code.claude.com/docs/en/agent-sdk/overview | Agent SDK | Python + TypeScript; same tools/hooks/MCP as CLI; Managed Agents = hosted variant; `query()` async generator pattern
https://code.claude.com/docs/en/how-claude-code-works | Official architecture | Gather→Act→Verify loop; 5 tool categories; 9-source context assembly; sessions as JSONL in ~/.claude/projects/; checkpoints snapshot files before edits (Esc Esc to rewind)
https://code.claude.com/docs/en/permissions | Permission system | Deny>ask>allow, first match wins; gitignore-style path semantics; 6 modes (default, acceptEdits, plan, auto, dontAsk, bypassPermissions); compound-command per-subcommand parsing
https://code.claude.com/docs/en/model-config | Model routing | `opusplan` = opus-in-plan / sonnet-in-execute; 5 effort levels (low/medium/high/xhigh/max); ultrathink keyword bumps reasoning per-turn; default fallback Opus→Sonnet under usage cap
https://code.claude.com/docs/en/plugins | Plugins | bundle skills + agents + hooks + MCP + LSP + monitors + bin/ + settings.json; .claude-plugin/plugin.json manifest; --plugin-dir for dev; marketplace distribution
https://karanprasad.com/blog/how-claude-code-actually-works-reverse-engineering-512k-lines | RE analysis 512K LOC | 1.6% AI / 98.4% infra; 7-stage permission pipeline; 6-tier compaction; `cache_edits` API for surgical cache preservation (~76% cache retained); filesystem IPC mailboxes at ~/.claude/work/ipc/ with 500ms polling; custom React-to-terminal renderer packing 32-bit cells; 1.2-1.8s bootstrap via parallel prefetch
https://vrungta.substack.com/p/claude-code-architecture-reverse | RE: TAOR loop | "Runtime is dumb, model is CEO"; primitive tools (Bash+Glob+Grep) beat custom integrations; context-cost ladder hooks(0)→skills(low)→plugins(med)→MCP(high)
https://agiflow.io/blog/claude-code-internals-reverse-engineering-prompt-augmentation/ | Prompt augmentation | 6 injection layers (output-styles persist; CLAUDE.md as system-reminder block; hooks deterministic; subagents = fresh thread); skills inject via tool_result blocks
https://newsletter.pragmaticengineer.com/p/building-claude-code-with-boris-cherny | Boris Cherny interview | "Glob+grep driven by model beat everything else"; rejected vector DB; 5 parallel sessions personal workflow; plan-mode-first; lint-rule-on-3rd-occurrence
https://github.com/VILA-Lab/Dive-into-Claude-Code | Systematic analysis | 5-value→13-principle→implementation chain; deny-first; permissions never restore on resume; weakness: pre-trust extension execution window; weakness: per-subcommand parsing causes event-loop starvation, commands>50 subcommands bypass security analysis
https://www.infoq.com/news/2026/05/anthropic-claude-code-postmortem/ | Quality postmortem | 6 weeks of complaints traced to 3 changes: (1) Mar 4 effort default high→medium, (2) Mar 26 caching bug clearing thinking every turn, (3) Haiku sub-agent delegation more aggressive than expected; resolved v2.1.116 Apr 20
https://www.infoq.com/news/2026/05/anthropic-claude-code-auto-mode/ | Auto Mode (research preview) | 2-stage classifier (fast filter + reasoning fallback); spinner turns RED on approval pause; tool-output inspection layer for prompt-injection defense
https://github.com/disler/claude-code-hooks-mastery | Real hooks ecosystem | Ruff/ty validators on PostToolUse; rm-block patterns; per-event JSON logging; 2-agent Builder+Validator team; meta-agent spawning sub-agents from descriptions; status line with token/cost
https://news.ycombinator.com/item?id=47936579 | "Is Claude Code getting worse?" | Documented user perception of capability drop; context window pressure; long-task drift confirmed
https://sankalp.bearblog.dev/my-experience-with-claude-code-20-and-how-to-get-better-at-using-coding-agents/ | Power-user post | Token guzzling, context degradation forces compact-or-restart; recommends /context at 60%, ultrathink before major decisions, throw-away first draft

## Feature inventory

feature | what it does | uniqueness | ELAI equivalent? | gap | citation
--- | --- | --- | --- | --- | ---
Plan mode (`/plan`, Shift+Tab x2) | Read-only tools only, produces structured plan, gated on approval | Harness-enforced (model literally cannot Edit) | Partial — ELAI has scope-planner-elai + plan files but no harness-level gate | ELAI relies on FSM state to block edits, not a tool-level toggle the user can flip mid-session | code.claude.com/docs/en/permissions
opusplan alias | Auto-switch Opus during plan, Sonnet during execute | Saves token spend without manual /model | NO — ELAI's RuntimeOrchestrator (B-37) does per-role routing but no plan/execute auto-switch | Steal: B-37 should gain a "plan_mode_runtime" override | code.claude.com/docs/en/model-config
Sub-agents (Explore, Plan, custom) | Spawned via Agent tool, own context, return summary | Description-driven auto-invocation | YES — ELAI has 17 agent files + AgentPool; but ELAI's are spec-bound, not description-matched | Different tradeoff: Claude Code's is fuzzier match, ELAI's is intentional spec wiring | code.claude.com/docs/en/sub-agents
Agent teams (experimental) | 1 lead + N teammates, mailbox messaging, shared task list | Inter-agent messaging beyond lead-coordinated | Partial — ELAI BSP supersteps coordinate via SSB but no peer-to-peer agent messaging | ELAI's BSP is more disciplined (barrier sync); Claude Code's teams are messy but flexible | code.claude.com/docs/en/agent-teams
Hooks (25+ events) | Lifecycle callbacks: PreToolUse, PostToolUse, Stop, SubagentStop, SessionStart, PreCompact, UserPromptSubmit, InstructionsLoaded, FileChanged, TaskCreated, TeammateIdle, Elicitation… | 5 handler types: command, http, mcp_tool, prompt (LLM-as-decision), agent | YES — ELAI hooks per CLAUDE.md (PreToolUse, PostToolUse, Stop, PreCompact) | ELAI has fewer events; missing: Setup, UserPromptExpansion, PermissionRequest, PermissionDenied, SubagentStart/Stop, TaskCreated/Completed, ConfigChange, InstructionsLoaded, Elicitation, WorktreeCreate/Remove | code.claude.com/docs/en/hooks
Skills (`.claude/skills/*/SKILL.md`) | YAML frontmatter + markdown, model-or-user invoked, supports `!\`cmd\`` shell injection, `context: fork` for subagent execution, live-reload | The shell-injection-into-prompt before model sees it is HIGH leverage | Partial — ELAI has skills under .claude/skills/ but no shell injection, no fork context | Steal: `!\`cmd\`` pattern + live-reload | code.claude.com/docs/en/skills
Permission modes | default / acceptEdits / plan / auto / dontAsk / bypassPermissions | Plan mode is a HARNESS-LEVEL toggle, not just a prompt prefix | NO — ELAI's FSM is per-state, not user-toggleable mid-session | Add a `/mode plan` toggle that overlays FSM state | code.claude.com/docs/en/permissions
MCP integration | stdio/SSE/HTTP transports, deferred tool loading, mcp__server__tool namespace | Tool descriptions deferred until first use (saves context) | YES — ELAI uses MCP heavily for local-rag + pod-lifecycle | ELAI does NOT defer tool descriptions; could steal deferred-load pattern to save context budget | code.claude.com/docs/en/mcp
Plugins | Bundle skills+agents+hooks+MCP+LSP+monitors+bin/+settings | Single manifest installable via marketplace | NO — ELAI has no plugin marketplace concept | Low priority steal: marketplace UX | code.claude.com/docs/en/plugins
Checkpoints | Pre-edit file snapshot, Esc Esc to rewind | Local to session, not git | NO — ELAI relies on git only | Medium-leverage: per-edit snapshots in session-local dir | code.claude.com/docs/en/how-claude-code-works
Sessions (JSONL in ~/.claude/projects/) | Plaintext append-only log, resume + fork via --fork-session / `/branch` | Same session ID resume vs new ID fork | Partial — ELAI has SESSION_HANDOFF / FSM persistence but no native fork/branch | Steal: fork-session for "try another approach without losing progress" | code.claude.com/docs/en/how-claude-code-works
Effort levels (low/med/high/xhigh/max) | Adaptive reasoning budget; ultrathink keyword for per-turn bump | xhigh default on Opus 4.7 | NO — ELAI uses fixed thinking via mcp_think_set_budget | ELAI's mcp_think is stronger (per-topic budgets) but missing the cheap per-prompt keyword bump | code.claude.com/docs/en/model-config
1M context window | sonnet[1m], opus[1m] aliases for long sessions | Just append [1m] to alias | Partial — ELAI uses claude-opus-4-7[1m] via env var already | This file is already wired for [1m] in CLAUDE.md per memory | code.claude.com/docs/en/model-config
Auto mode (research preview) | 2-stage classifier (fast filter → reasoning fallback for ambiguous) | Spinner turns red on approval pause | NO — ELAI has no auto-classify mode | Lower priority; risks per InfoQ postmortem | infoq.com/news/2026/05/anthropic-claude-code-auto-mode
Background agents (`/background`, claude.ai/code) | Detached cloud sessions, `claude --teleport` to pull into terminal | Anthropic-managed VMs | NO — ELAI is local-first | This is intentional ELAI design (local + remote-sidecar only) | code.claude.com/docs/en/overview
Routines (cron-like) | Scheduled remote agents, trigger on GitHub events / API calls | Anthropic-managed infra | Partial — ELAI has /loop skill + CronCreate/CronList | wash | code.claude.com/docs/en/routines
Worktrees (`/batch` with worktree-per-task) | Auto-creates git worktrees for parallel work | One worktree per parallel session | NO — ELAI has no worktree automation | Steal: B-N work that touches independent crates could parallelize | code.claude.com/docs/en/worktrees
@-mentions in prompts | `@file/path` pulls file into context, respects gitignore by default | Standard across surfaces | NO — ELAI uses Read tool only | Low-effort steal | code.claude.com/docs/en/memory
Auto memory (~/.claude/projects/MEMORY.md) | First 200 lines / 25KB loads at session start; learnings auto-saved | Built into harness, not opt-in | YES — ELAI has memory-writer agent + MEMORY.md (per current session-reminder this exact file) | wash — ELAI's is delegated to memory-writer, Claude Code's is automatic | code.claude.com/docs/en/memory
Output styles | Append-to-system-prompt format directives (Explanatory/Concise/Detailed/custom) | Persistent per session | NO | Low-leverage steal | code.claude.com/docs/en/settings
Status line | Custom shell command produces status line (token/cost/agent name) | Hookable display | NO | Low-leverage steal | docs hooks/status-line
Sandbox | OS-level filesystem + network isolation for Bash (bwrap/socat on Linux, equivalent on mac) | Defense-in-depth beyond permission rules | NO — ELAI has no OS sandbox | Higher-leverage steal: sandbox config for B-N executor crate | code.claude.com/docs/en/sandboxing
Notifications (terminalSequence in hooks) | OSC sequences for desktop notify | Hook-driven | Partial — ELAI has hooks but not OSC notify | trivial | code.claude.com/docs/en/hooks
LSP plugins | Plugin can ship .lsp.json for code intelligence | Real-time type errors / jump-to-def | NO — ELAI uses elai.map + SCIP indices | Different approach; ELAI's static map is fine for current scale | code.claude.com/docs/en/plugins-reference#lsp-servers
Monitors | `monitors/monitors.json` runs background command, each stdout line is a notification | Pushed events into session | NO | Medium-leverage: ELAI could use this for sidecar log monitoring | code.claude.com/docs/en/plugins-reference#monitors

## Architecture findings

Reverse-engineering analyses (karanprasad.com, VILA-Lab, alejandrobalderas, ComeOnOliver) agree on the substrate split:

- **The agent loop itself is ~40 lines.** Pseudocode: `while True: r = model.call(messages); messages.append(r); if r.stop_reason == "end_turn": break; results = execute_tools(r.tool_uses); messages.append({role: "user", content: results})`. This is the "model is CEO, runtime is dumb" pattern. ELAI's `live_cli.rs` + IPC loop is structurally similar.
- **98.4% of LOC is deterministic infrastructure** wrapping that loop: permission gates, compaction shapers, tool dispatch, sandbox, transcript management, terminal rendering, IPC. ELAI's `.elai_cc/crates/` mirrors this split.
- **Context assembly is 9 ordered sources** per VILA-Lab: managed settings, project settings, user settings, CLAUDE.md (project), CLAUDE.md (user), auto-memory MEMORY.md, skills (descriptions only by default), MCP tool registry (deferred), conversation history.
- **Compaction is a 6-tier ladder** (cheapest first): Budget Reduction → Snip → Microcompact → Context Collapse → Auto-Compact → narrative-summary. Critically, Claude Code uses Anthropic's API parameter `cache_edits` to surgically remove tool results WITHOUT invalidating the cached prompt prefix — retains ~76% cache benefit per karanprasad.com. ELAI's PreCompact hook + SESSION_HANDOFF is structurally similar but does not have the cache_edits surgical trick (and probably can't, since cache_edits is a hosted-API feature ELAI bypasses via remote sidecar).
- **Permission pipeline = 7 stages.** First match wins, deny > ask > allow. Compound commands (`&&`, `||`, `;`, `|`, `|&`, `&`, newline) parsed per-subcommand; each subcommand must match independently. Process wrappers (`timeout`, `time`, `nice`, `nohup`, `stdbuf`, bare `xargs`) stripped before matching. Read-only built-ins (`ls`, `cat`, `echo`, `pwd`, `head`, `tail`, `grep`, `find`, `wc`, `which`, `diff`, `stat`, `du`, `cd`, read-only `git`) skip the prompt entirely. Bypass mode still circuit-breaks on `rm -rf /` / `rm -rf ~`.
- **Hook output is uniform JSON**, top-level fields: `continue` | `stopReason` | `suppressOutput` | `systemMessage` | `terminalSequence` + event-specific `hookSpecificOutput` block. Exit code 2 = blocking error (stderr becomes error message). Other non-zero = non-blocking warning.
- **Subagents are isolated transcripts.** When the main agent calls the Agent tool with `subagent_type="explore"`, a fresh JSONL is opened with a different system prompt (the agent's frontmatter), the spawned agent runs to completion, and ONLY the final summary message returns to the main transcript. `parent_tool_use_id` field links sub-messages back to their spawn point. CLAUDE.md is loaded except when the agent is `Explore` or `Plan` (which skip it to keep context small).
- **Skills load on demand.** Description-only in main context (1% of context window default budget for skill listing, configurable via `skillListingBudgetFraction`). Full SKILL.md content loads when invoked. After load, content sticks for the rest of the session. Auto-compaction re-attaches the most recent invocation of each skill, keeping first 5K tokens per skill within a combined 25K-token budget.
- **MCP tool descriptions are deferred** by default (loaded on demand via tool search). This is huge for context budget. ELAI does NOT do this — every MCP tool from local-rag and pod-lifecycle is in context at session start. STEAL.
- **Filesystem-based subagent mailbox IPC** at `~/.claude/work/ipc/` with 500ms polling. Simple, race-prone, documented as such. ELAI uses SSB events over Tauri IPC + sled — structurally cleaner but more code.
- **Sessions are JSONL** at `~/.claude/projects/<project-slug>/<session-id>.jsonl`. Plain text, grep-able, fork-able with `--fork-session` or `/branch`. ELAI has SESSION_HANDOFF.md + memory-bank — different shape, similar intent.
- **Terminal renderer is custom React-to-TTY** packing screen state into 32-bit ints (21 bits Unicode, 4+4 fg/bg color, 3 style). Frame-diff emits only changed cells — idle output drops from ~10KB to ~50B per frame. ELAI's dashboard is Svelte+Tauri (different target — desktop window, not TTY).
- **Boot time 1.2-1.8s** via parallel prefetch: 3 side-effect imports load BEFORE main code (MDM reader subprocess + macOS keychain access), so auth tokens are ready by the time the UI is ready. ELAI's `elai.bat` boot path is unaudited from this lens.

## Hook ecosystem 2026

Real hooks people ship (from disler/claude-code-hooks-mastery and awesome-claude-code at 44.2K stars):

Category | Concrete hooks shipped | Trigger | ELAI gap
--- | --- | --- | ---
Code-quality enforcement | Ruff lint, ty type-check, ESLint, Prettier-write | PostToolUse on Edit/Write | ELAI has a PostToolUse ruff-syntax-check per CLAUDE.md — matches
Dangerous-command blocking | `rm -rf` block, `.env` read block, curl-to-external-URL block | PreToolUse on Bash with `if: "Bash(rm *)"` | ELAI has cross-session file locks + state-gated edits — different shape, similar intent
Audit logging | JSON event log to `logs/<session>/<event>.json` for every hook event | All 25 events | ELAI logs to `workflow/sessions/{id}/hook-executions.log` — matches
2-agent Builder+Validator team | Builder agent does write/edit; Validator agent is read-only, checks acceptance criteria | SubagentStop + custom dispatcher | ELAI has auditor.md as sole test-runner — matches
Meta-agent | Sub-agent that generates new sub-agent definitions from descriptions | UserPromptSubmit pattern match | ELAI has no meta-agent (intentional — agents are hand-written + canonical)
Context injection | Git status + recent issues + project standards prepended to UserPromptSubmit | UserPromptSubmit | ELAI uses CLAUDE.md + session-start skill loading — matches
Output styles | YAML, table, GenUI HTML formatting | settings.json `outputStyle` | ELAI has no native output style enforcement
TDD enforcement | tdd-guard repo (1.7K stars): blocks code edits if tests don't exist | PreToolUse on Edit | ELAI test-attributer.md is more thorough (failure taxonomy + attribution) — ELAI better
MCP-call validation | mcp_tool hook validates write-class MCP calls | PreToolUse `matcher: "mcp__.*__write.*"` | ELAI has FSM gate per-state — different mechanism, similar coverage
Status line | shell command emits status (token/cost/agent name/context % full) | Pull, not hook | ELAI has no status line
LLM-as-decision hook (`type: prompt`) | Hook calls fast model with custom prompt for yes/no decision | PreToolUse / PostToolUse / Stop | ELAI has NOTHING like this — high leverage steal (e.g. "did the auditor's pass actually verify what changed?")
Async background hook | `"async": true, "asyncRewake": true` runs analysis without blocking turn | PostToolUse | ELAI hooks are all sync — could parallelize lint/audit/sec scans
Watched files | `FileChanged` event with literal filename matcher | watched-files config | ELAI has cross-language pair drift via cross_lang_diff.py — different scope but related

## Sub-agents pattern

Claude Code sub-agent shape:

- Defined in `.claude/agents/<name>.md` with YAML frontmatter: `name`, `description` (the trigger), `tools` (allowlist), `model`, `effort`, `system_prompt` (= the markdown body).
- Invoked via the Agent tool. The MAIN agent calls `Agent(subagent_type="name", description="task X", prompt="...")`. The harness spawns a fresh transcript, injects the sub-agent's system prompt + the requested task, runs the agent loop in isolation, and returns ONLY a summary message to the main transcript.
- Description-driven auto-invocation: when the main model sees a task that matches a sub-agent's `description`, it auto-delegates. The user can also explicitly say "use the code-reviewer agent."
- Parallel: multiple Agent calls in one tool batch run in parallel (each in its own context window). Returned summaries arrive when each completes.
- Context inheritance: CLAUDE.md is loaded by default. `Explore` and `Plan` built-in agents skip CLAUDE.md to stay small.
- Cost control: `model` frontmatter lets you route a sub-agent to Haiku for cheap research tasks. `CLAUDE_CODE_SUBAGENT_MODEL` env var overrides all.
- Tool scoping: `tools: Read Glob Grep` restricts what the sub-agent can do. Useful: Explore = read-only (Read/Glob/Grep/WebFetch/WebSearch), Plan = read-only + ultrathink.

ELAI's pattern (per CLAUDE.md memory):

- Canonical 15-agent list (auditor, context-compiler, memory-writer, paper-ingestor, prompt-engineer, scope-planner-elai, integration-explorer-elai, synthesizer-elai, impact-mapper, spec-grounder, fixer, test-attributer, wiring-contract-validator, sidecar-health, dashboard-smoker). NEVER guess. Pass agent file path + minimal params.
- All agents run on `model=sonnet`.
- Agents read their OWN definition files; main LLM never reads .claude/agents/*.md directly.
- AgentPool + BSP supersteps coordinate (per memory: project_b40_shipped, project_b37_shipped).
- FSM substates (AUDITING, MEMORIZING) gate when which agent can run.

Comparison:

dimension | Claude Code | ELAI
--- | --- | ---
discovery | description-driven fuzzy match | canonical list, named explicitly
parallel | tool-batch parallel, no barrier | BSP barrier sync per superstep
context inheritance | CLAUDE.md by default, skippable | agent reads its own file + plan files
model routing | per-agent in frontmatter | global `model=sonnet` for all (less flexible)
return shape | summary message | one-line return + VALIDATION WARNINGS (per CLAUDE.md memory)
isolation | own JSONL transcript | own context but inside same FSM session
who triggers | model auto OR user `/agent name` | main LLM dispatches explicitly per plan workflow
parent-link | `parent_tool_use_id` field | FSM agent_substate transitions

ELAI's discipline is much higher (canonical list, no ad-libbing); Claude Code's is more flexible (any markdown in `.claude/agents/` is callable). ELAI wins on rigor; Claude Code wins on iteration speed.

## What ELAI is doing BETTER than Claude Code

- **Grounded Reasoning Chain (GRC).** mcp_think with mandatory grounding + per-topic budgets + counterfactual gates + reasoning-after-action. Claude Code has nothing comparable — `ultrathink` is just a keyword that bumps thinking-token budget for one turn. ELAI's CLAUDE.md hard-rule that "every claim must be grounded in (1) tests, (2) code just read, (3) docs verified via WebSearch, (4) ingested papers, (5) third-party benchmarks" is stricter than anything Claude Code enforces. Grounded by: this conversation's CLAUDE.md content and the absence of equivalent in https://code.claude.com/docs/en/how-claude-code-works (which only says "Claude decides what each step requires").
- **FSM with state-gated tool access.** ELAI's FSM substates (AUDITING, MEMORIZING, EXECUTING) plus PreToolUse hook gating per state + cross-session file locks is structurally stronger than Claude Code's stateless permission rules. Claude Code's `defaultMode` is a session-wide toggle, not a workflow state machine. Grounded by: CLAUDE.md "FSM gate check: call mcp_fsm_get_state before any Edit/Write/Bash. If blocked, transition first" vs https://code.claude.com/docs/en/permission-modes (modes are flat, not transitional).
- **Cross-language pair drift detection.** ELAI's cross_lang_diff.py flagging Rust↔Python sidecar or Tauri↔Svelte drift is a real safety net. Claude Code has no concept of paired files; users have to write hooks themselves. Grounded by: CLAUDE.md tools list line "cross_lang_diff.py — flag changed files lacking their cross-language pair" and search results returning zero Claude Code feature comparable.
- **Spec-anchored planning.** `Spec references:` blocks in plan files + spec_link_check.py + spec-grounder.md agent that VERIFIES every plan claim against actual spec content. Claude Code's plan mode produces a plan but never verifies the plan against an external spec. Grounded by: CLAUDE.md feedback_spec_references memory entry and absence of equivalent at code.claude.com.
- **RAG with mcp_search_context and topic-scoped memory.** Claude Code uses CLAUDE.md (static, line-truncated) + auto-memory (first 200 lines / 25KB at session start, no retrieval). ELAI uses local-rag MCP with topic-scoped vector search. Grounded by: CLAUDE.md `mcp_memorize/mcp_log_change for RAG indexing` + `mcp_search_context` reference in rag-tools skill, vs https://code.claude.com/docs/en/memory which describes only flat-file loading.
- **Static codebase map** (`elai.map`, ~375K lines, SCIP-enriched, 6-col TSV, all langs). Claude Code uses glob+grep "driven by model" per Boris Cherny (Pragmatic Engineer interview). For ELAI's scale that's fine, but ELAI's map is faster + token-cheaper for the impact-radius / wiring-contract / blast-radius checks the harness needs. Grounded by: CLAUDE.md `Prefer .claude/tools/elai.map over whole-repo grep — one grep returns every DEF/CALL/USE/IMPL_TRAIT/TYPE_REF/SCIP_REF clustered`.
- **Anti-spiral / anti-stale-training rules.** Explicit `Anti-Stale-Training: training data is 1+ year old. Default to these over training instincts` clauses are a real safety mechanism. Claude Code documentation says NOTHING about training-data staleness; the postmortem (InfoQ May 2026) shows users actively noticed capability drops with no formal mechanism for the harness to flag them. Grounded by: CLAUDE.md `Anti-Stale-Training (HARD RULE)` block.
- **Decision docs (workflow/reasoning/phase-X-decisions.md).** Forced documentation of what/key-decision/expected-outcome/unknowns BEFORE execution. Claude Code has no equivalent — plans are ephemeral in the transcript. Grounded by: CLAUDE.md `Decision doc: before EXECUTING, write workflow/reasoning/phase-X-decisions.md` vs absence at code.claude.com.

## What ELAI is doing WORSE than Claude Code

- **Compaction algorithm is naive.** ELAI's PreCompact hook writes SESSION_HANDOFF.md before context truncation. Claude Code has a 6-tier ladder (Budget Reduction → Snip → Microcompact → Context Collapse → Auto-Compact → narrative-summary) + uses the API's `cache_edits` parameter to surgically remove tool results WITHOUT busting the prompt cache (~76% cache preserved). ELAI gets none of that surgical control because it routes through the remote sidecar, not directly to api.anthropic.com. Grounded by: karanprasad.com 512K-LOC RE + CLAUDE.md `PreCompact hook auto-saves FSM state to SESSION_HANDOFF`.
- **No deferred MCP tool descriptions.** Claude Code defers MCP tool descriptions (only names in context until tool search retrieves details). ELAI loads every mcp__local-rag__* and mcp__pod-lifecycle__* tool's full schema into context at session start (this very session has a `ToolSearch` mechanism in the system prompt that's the same idea — but it covers built-in tools, not MCP servers). Grounded by: code.claude.com/docs/en/mcp `MCP tool definitions are deferred by default and loaded on demand via tool search` + CLAUDE.md absence of "deferred MCP loading" rule.
- **No checkpoints / Esc-Esc rewind.** Claude Code snapshots affected files before EVERY edit and double-Esc rewinds. ELAI relies on git only — slower, requires commit discipline, no in-session undo. Grounded by: code.claude.com/docs/en/how-claude-code-works `Before Claude makes code changes, it also snapshots the affected files so you can revert if needed`.
- **No fork-session.** Claude Code `--fork-session` + `/branch` copies transcript to a new session ID, leaves the original untouched — useful for "try another approach without losing progress." ELAI's SESSION_HANDOFF is linear. Grounded by: code.claude.com/docs/en/how-claude-code-works `Forking with --fork-session or /branch copies the history into a new session ID`.
- **No OS-level sandbox.** Claude Code has bwrap-on-Linux / equivalent-on-macOS sandboxing for Bash tool, layered ON TOP of permission rules. ELAI relies on permission rules + hooks only. Grounded by: code.claude.com/docs/en/sandboxing.
- **Skills are static (no `!\`cmd\`` shell injection).** Claude Code skills can prefix their body with backtick-bang-command-backtick blocks that are executed BEFORE the skill content is sent to the model, output is inlined into the prompt. ELAI skills are markdown files only. Grounded by: code.claude.com/docs/en/skills `dynamic context injection` section.
- **No live skill reload.** Claude Code watches `~/.claude/skills/` and `.claude/skills/` for file changes — edits take effect within the running session. ELAI skills load at session start only. Grounded by: code.claude.com/docs/en/skills `Live change detection`.
- **No `prompt`-type hook (LLM-as-decision).** Claude Code's hook handler type `prompt` calls a fast model with a custom yes/no prompt to make decisions deterministically without coding. ELAI hooks are command/bash only. Grounded by: code.claude.com/docs/en/hooks `"type": "prompt"`.
- **No status line.** Claude Code shows token/cost/agent/context% in a configurable status line. ELAI dashboard surfaces some of this but the CLI itself doesn't. Grounded by: code.claude.com/docs/en/statusline.
- **No worktree automation (`/batch`).** Claude Code's `/batch` decomposes a large change into independent units and runs each in its own git worktree. ELAI has the b-plan-runner skill but no worktree-per-task pattern. Grounded by: code.claude.com/docs/en/commands.
- **No background agents / `claude --teleport`.** Intentional ELAI design choice (local-first), but worth noting Claude Code lets long-running tasks run on Anthropic infra and reattach later.

## Concrete steals (high leverage)

Ranked by ELAI leverage. Each: what / grounding / which existing ELAI phase or new B-N to slot it into.

1. **Deferred MCP tool description loading.** WHAT: at session start, load only `mcp__server__tool` NAMES into context, fetch full descriptions on first call via tool-search analog. GROUNDING: code.claude.com/docs/en/mcp explicit doc + measured savings (full tool descriptions for local-rag + pod-lifecycle together are >100 schema entries). SLOT: extend B-37 RuntimeOrchestrator with a `ToolRegistry::lazy_mode` flag; new B-N (call it B-43) — "Deferred MCP Tool Schema Loading" — IPC v2 already supports lazy tool fetch, just wire it.

2. **6-tier compaction ladder with cache-aware tier ordering.** WHAT: implement Budget Reduction → Snip → Microcompact → Context Collapse → Auto-Compact → narrative-summary, each progressively more aggressive, with cheapest-first ordering and a cache-bust telemetry signal. GROUNDING: karanprasad.com 512K-LOC RE; code.claude.com/docs/en/how-claude-code-works `"clears older tool outputs first, then summarizes the conversation if needed"`. SLOT: B-N to slot into Phase 4 (context-discipline phases), let's say B-44 — "Tiered Compaction Pipeline." Even without cache_edits API (remote sidecar bypass), the tier ordering alone saves tokens.

3. **`!\`cmd\`` shell injection in skill markdown + live skill reload.** WHAT: ELAI skill loader scans for backtick-bang-cmd-backtick blocks and `\`\`\`!` fenced blocks, executes them in a constrained shell, replaces the placeholder with stdout BEFORE the model sees the skill content. Watch `.claude/skills/` for file changes; reload affected skill metadata in-session. GROUNDING: code.claude.com/docs/en/skills explicit. SLOT: extend the existing skill loader in `.claude/skills/SKILL.md` infrastructure; B-45 — "Dynamic Skill Templating + Hot Reload." Strict win, low risk.

4. **`prompt`-type hook (LLM-as-decision).** WHAT: new hook handler type that calls a fast model (Haiku or local Qwen 3B sidecar) with a structured prompt for yes/no decisions, decision returns to harness as block/allow/ask. GROUNDING: code.claude.com/docs/en/hooks `"type": "prompt"`. SLOT: ELAI already has the sidecar wired with ANTHROPIC_BASE_URL=http://localhost:8100 (per CLAUDE.md remote sidecar block); B-46 — "LLM-Decision Hook Handler." Decide questions like "did the auditor's pass actually verify what changed?" without writing custom code.

5. **`opusplan`-style plan/execute auto-switch.** WHAT: when FSM enters PLANNING substate, route through Opus (or higher-effort sidecar); when entering EXECUTING, switch to Sonnet (or lower-effort sidecar). GROUNDING: code.claude.com/docs/en/model-config explicit. SLOT: extend B-37 RuntimeOrchestrator + PerRoleRuntime with a `phase_substrate` overlay; B-47 — "Plan/Execute Runtime Auto-Switch." Already 90% in place per project_b37_shipped memory.

6. **Pre-edit file checkpoints (Esc-Esc rewind).** WHAT: harness snapshots files about to be edited to `.elai_cc/.session-checkpoints/{session}/{seq}/{path}`, double-Esc keystroke rewinds, snapshot evicted on session end or git commit. GROUNDING: code.claude.com/docs/en/how-claude-code-works. SLOT: extend existing PreToolUse Edit/Write hook to write snapshots; B-48 — "Session-Local Edit Checkpoints." Independent of git, plays nice with anti-slopsquatting / zero-lazy-code rules.

7. **Fork-session (`--fork-session` / `/branch`).** WHAT: copy current FSM state + memory-bank + SESSION_HANDOFF into a new session ID, leave original untouched, allow user to try alternative approach. GROUNDING: code.claude.com/docs/en/how-claude-code-works. SLOT: extend resume.rs / live_cli.rs; B-49 — "Session Forking." Pairs with checkpoints.

8. **Hook event coverage expansion.** WHAT: add the missing 18 hook events from Claude Code's list (Setup, UserPromptExpansion, PermissionRequest, PermissionDenied, SubagentStart/Stop, TaskCreated/Completed, ConfigChange, InstructionsLoaded, FileChanged, WorktreeCreate/Remove, Elicitation, etc.). GROUNDING: code.claude.com/docs/en/hooks exhaustive list. SLOT: expand existing hook plumbing; B-50 — "Hook Event Surface Expansion." Drops in incrementally.

9. **Permission rule syntax + first-match-wins / deny-first semantics.** WHAT: replace ELAI's state-and-substate-gated permission with declarative rules in settings.json (allow / ask / deny arrays), parsed with gitignore-style path semantics, compound-command per-subcommand parsing, process-wrapper stripping, read-only built-in skip-list. GROUNDING: code.claude.com/docs/en/permissions. SLOT: significant refactor; B-51 — "Declarative Permission Rules." HIGH risk, MEDIUM leverage — current FSM gating may be intentional design choice per user enforcement memory; needs user confirmation before doing.

10. **Worktree-per-parallel-task (`/batch` analog).** WHAT: for B-N work that touches >1 independent crate, decompose into per-crate worktrees, run agents in parallel, merge at end. GROUNDING: code.claude.com/docs/en/worktrees + b-plan-runner skill already does serial. SLOT: extend b-plan-runner; B-52 — "Worktree-Parallel B-N Execution."

11. **Output styles (`outputStyle` setting).** WHAT: configurable response format (Concise / Explanatory / Table / YAML / GenUI). GROUNDING: code.claude.com/docs/en/settings. SLOT: low-priority; minor refactor of CLAUDE.md output-formatting.md rule to be runtime-configurable.

12. **Background `monitors/monitors.json`.** WHAT: plugin/skill bundles a `monitors.json` with `command` + `description`; each stdout line becomes a notification into the session. ELAI use case: tail remote sidecar log into the session so the main LLM sees `RuntimeError: NF4 OOM` events as they happen. GROUNDING: code.claude.com/docs/en/plugins-reference#monitors. SLOT: B-53 — "Background Monitors" pairs well with sidecar-protocol skill.

13. **OS-level sandbox (`bwrap` on Linux, equivalent on macOS/Windows).** WHAT: spawn Bash tool calls inside a sandbox that constrains filesystem (allowWrite/denyWrite/allowRead/denyRead) and network (allowedDomains/deniedDomains/allowUnixSockets). GROUNDING: code.claude.com/docs/en/sandboxing. SLOT: HIGH risk on Windows where bwrap doesn't exist; defer to Linux/WSL-only path; B-54 — "Sandbox Layer for Bash Tool." Lower priority — ELAI runs on Windows 11 per env block, sandbox infra is thinner.

14. **Status line.** WHAT: configurable shell-command-produced status line showing token / cost / context% / agent name. GROUNDING: code.claude.com/docs/en/statusline. SLOT: trivial; add to live_cli.rs.

## Anti-patterns

Cited from Claude Code criticism. Do NOT copy these.

- **Pre-trust extension execution window.** Plugins and MCP servers execute during initialization BEFORE the trust dialog appears. Source of 2 patched CVEs per VILA-Lab. ELAI's pod-lifecycle + local-rag MCP startup should NOT execute side effects before user confirms (current state unknown but worth auditing). Grounded by: github.com/VILA-Lab/Dive-into-Claude-Code "Pre-trust execution window".
- **Per-subcommand parsing event-loop starvation.** Commands with >50 subcommands BYPASS security analysis entirely to prevent the REPL freezing. This is a documented safety bypass. ELAI's permission system, if it adopts steal #9 (declarative rules), must NOT inherit this fallback. Cap parses at >50 should BLOCK, not skip. Grounded by: VILA-Lab "Commands exceeding 50 subcommands bypass security analysis entirely to prevent the REPL from freezing."
- **Approval fatigue masking.** When 93% approval rate revealed users were rubber-stamping prompts, Claude Code's response was to restructure boundaries rather than examining why users weren't reading. ELAI should treat high auto-approval as a signal that the approval is poorly designed, not as a license to widen the auto-accept boundary. Grounded by: VILA-Lab analysis.
- **Permissions never restored on resume.** Claude Code re-establishes trust per session. Defensible UX choice, but means long-running ELAI sessions that resume across pre-compact would lose state. ELAI's SESSION_HANDOFF approach is BETTER here — keep it.
- **Quality regression from optimization (the 6-week postmortem).** The InfoQ postmortem (May 2026) showed 3 independent product changes ALL degraded quality without telemetry catching it: effort-default downgrade (high→medium), cache-clear-on-every-turn bug, Haiku over-delegation. Anti-pattern: ship optimizations without quality regression tests. ELAI's auditor.md + b-plan-runner FSM forces a test gate per phase — keep that.
- **MCP server fragility (silent empty-file reads).** "If a file path is missing or a write is incomplete, the workflow can break without clear errors, and agents can get stuck because a file read returned an empty value instead of an error." ELAI's mcp_search_context should explicitly distinguish "no results" from "RAG corpus empty" from "topic not registered" — avoid Claude Code's silent-empty antipattern. Grounded by: substratia.io / nimbalyst.com 2026 MCP server reviews.
- **Bash permission patterns that constrain command arguments are fragile.** Claude Code documentation explicitly warns `Bash(curl http://github.com/ *)` doesn't catch `curl -L http://bit.ly/x` (redirect) or `curl https://github.com/...` (different protocol) or `URL=http://github.com && curl $URL` (variable). ELAI should use WebFetch domain rules + PreToolUse LLM-decision hook (steal #4) instead of trying to regex Bash arguments. Grounded by: code.claude.com/docs/en/permissions explicit warning block.
- **Haiku over-delegation invisible to user.** Sub-agent delegation to cheaper Haiku visible only in verbose logging per InfoQ postmortem. ELAI's `model=sonnet` global default is anti-fragile here (less flexibility, less drift). Don't add per-agent Haiku routing without making it visible in the status line.

## Open questions

Measurable.

1. **What is ELAI's current per-session token budget breakdown** (CLAUDE.md / memory-bank / skills / MCP-tool-schemas / conversation)? Without measurement we can't quantify the savings from steal #1 (deferred MCP loading) or steal #2 (tiered compaction). Action: instrument live_cli.rs at the next phase. Threshold: if MCP-tool-schemas + skill-descriptions exceed 15K tokens at session start, steal #1 is high-leverage; if under 5K, defer.

2. **Does the remote sidecar (ANTHROPIC_BASE_URL=http://localhost:8100) preserve any prompt-cache benefits at all?** Claude Code's `cache_edits` surgical cache preservation is an api.anthropic.com server-side feature. If the local sidecar is stateless per-request, ELAI gets no cache benefit AND every compaction is full re-tokenization. Action: profile a 20K-token resumed session; if first-turn-after-resume is >2x first-turn-of-fresh-session, cache is dead, compaction strategy matters more.

3. **What is the actual contention rate on cross-session file locks** (per CLAUDE.md PreToolUse hook)? Claude Code uses filesystem IPC mailboxes with documented race conditions. ELAI's locks could be similarly stressed under concurrent sessions. Action: log lock-wait histogram for 1 week; if p99 lock wait >500ms, replace with sled-transactional or move to SSB-event-only coordination.

4. **How many B-N plans have skipped the patch-N protocol's "context-compiler + parallel explorers" gate?** CLAUDE.md memory says "the previous session that skipped protocol on the mode-picker bug had to be re-done." Action: count session-handoff files with `protocol_skip=true` over last 30 days; if >10%, the protocol's friction is too high and needs a fast-path AND a slow-path, or the skip needs to be auto-detected from blast-radius (>3 files = no skip).

5. **Is `mcp_think` actually firing on the GRC trigger conditions (debugging / architecture / multi-step / uncertainty / pre-destructive)?** No telemetry currently. Action: hook PostToolUse on mcp_think to log call-frequency-per-trigger-category; if `pre-destructive` triggers < 20% of actually-destructive ops, the GRC enforcement is performative not real. (This is the most important question on this list.)

---

Sources (markdown hyperlinks for the WebSearch requirement):

- [Claude Code Overview](https://code.claude.com/docs/en/overview)
- [Claude Code Hooks](https://code.claude.com/docs/en/hooks)
- [Claude Code Sub-agents](https://code.claude.com/docs/en/sub-agents)
- [Claude Code Agent Teams](https://code.claude.com/docs/en/agent-teams)
- [Claude Code MCP](https://code.claude.com/docs/en/mcp)
- [Claude Code Settings](https://code.claude.com/docs/en/settings)
- [Claude Code Skills](https://code.claude.com/docs/en/skills)
- [Claude Code Plugins](https://code.claude.com/docs/en/plugins)
- [Claude Code Permissions](https://code.claude.com/docs/en/permissions)
- [Claude Code Model Config](https://code.claude.com/docs/en/model-config)
- [How Claude Code Works](https://code.claude.com/docs/en/how-claude-code-works)
- [Claude Agent SDK Overview](https://code.claude.com/docs/en/agent-sdk/overview)
- [Claude Code Commands Reference](https://code.claude.com/docs/en/commands)
- [How Claude Code Actually Works (karanprasad RE 512K LOC)](https://karanprasad.com/blog/how-claude-code-actually-works-reverse-engineering-512k-lines)
- [Claude Code Architecture Reverse Engineered (vrungta)](https://vrungta.substack.com/p/claude-code-architecture-reverse)
- [Claude Code Internals: Prompt Augmentation (agiflow)](https://agiflow.io/blog/claude-code-internals-reverse-engineering-prompt-augmentation/)
- [Building Claude Code with Boris Cherny (Pragmatic Engineer)](https://newsletter.pragmaticengineer.com/p/building-claude-code-with-boris-cherny)
- [Dive into Claude Code (VILA-Lab systematic analysis)](https://github.com/VILA-Lab/Dive-into-Claude-Code)
- [Anthropic 6-Week Claude Code Postmortem (InfoQ)](https://www.infoq.com/news/2026/05/anthropic-claude-code-postmortem/)
- [Inside Claude Code Auto Mode (InfoQ)](https://www.infoq.com/news/2026/05/anthropic-claude-code-auto-mode/)
- [Is Claude Code getting worse? (HN)](https://news.ycombinator.com/item?id=47936579)
- [claude-code-hooks-mastery (disler)](https://github.com/disler/claude-code-hooks-mastery)
- [awesome-claude-code (hesreallyhim, 44.2k stars)](https://github.com/hesreallyhim/awesome-claude-code)
- [Claude Code 2.0 experience (sankalp)](https://sankalp.bearblog.dev/my-experience-with-claude-code-20-and-how-to-get-better-at-using-coding-agents/)
- [Best MCP Servers Claude Code 2026 (Substratia)](https://substratia.io/blog/best-mcp-servers-claude-code/)
- [Best Claude Code MCP Servers 2026 (Nimbalyst)](https://nimbalyst.com/blog/best-claude-code-mcp-servers/)
