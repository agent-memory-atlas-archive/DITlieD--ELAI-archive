MiMo-Code Part 6: Skills, Workflows, Plugins, Config, Commands, Flags

SKILL SYSTEM (packages/opencode/src/skill/)

discovery.ts
  Discover remote skill repositories via HTTP index.json. Concurrent download (4 skills, 8 files per skill) from configured URLs. Filters for SKILL.md presence. Returns local cached directories. Enables skill distribution from central or decentralized URLs. Used by skill/index.ts to pull skills from cfg.skills.urls.

index.ts (312 lines)
  Central skill registry. Four discovery sources: (1) compose-bundle (built-in skills extracted to disk first, user skills override), (2) external directories (.claude/.agents/.codex/.opencode in home), (3) project local search up the tree, (4) config-declared paths and URLs. Loads SKILL.md frontmatter (name, description, hidden, location, content) via ConfigMarkdown.parse. Concurrent load via mcp_think (unbounded). Exposes get(name), all(), dirs(), available(agent?). Filters by permission and hides hidden:true skills. Output via fmt() renders XML skill blocks for prompts.

compose/extract.ts
  Extracts compile-time bundled skills to disk on startup. Uses bundle.macro (Bun-specific) to resolve embedded skill files at build time. Falls back to runtime loadComposeBundleDev() if macro unavailable (dynamic imports, tests). Writes to GlobalPath.data/compose/{version}/skills/. Marker file prevents re-extraction on local installs. Exposes composeSkillsBlock() to build XML skill block for agent prompts.

compose/bundle.macro.ts
  Bun compile-time macro that walks .bundle/ directories and returns a Record<skillName, Record<relPath, content>>. Enables zero-runtime-cost inclusion of hardcoded skills (no .env substitution, pure data). Skills preloaded during build, injected as static text at macro-call site.


WORKFLOW SYSTEM (packages/opencode/src/workflow/)

meta.ts (336 lines)
  Parse workflow script's mandatory "export const meta = { ... }" metadata WITHOUT executing the script body. Uses recursive-descent data reader to parse only JS object/array/string/number/true/false/null — rejects any expression (calls, member access, operators, spreads, template strings). Prevents arbitrary code injection via malicious meta literals (e.g. process.execSync inside the meta). Returns WorkflowMeta (name, description, whenToUse, phases[], model). Replaces meta statement with blanks to preserve line numbers for stack traces.

runtime.ts (1000+ lines)
  QuickJS sandbox for workflow scripts. Runs guest JavaScript in isolated context with hardened Math.random (seeded for determinism), no Web/Node APIs, memory/deadline limits. Exposes host functions (agent, workflow, task, parallel, pipeline, memory) as global injections. Tracks concurrent agents via semaphore (default 16). Per-agent timeout and per-run deadline. Journal-based resume: replays persisted agent results to skip re-execution of equivalent runs. Handles script structure errors (cycles, max-depth, unknown names) with loud failure to root. Never-throw on agent failures (returns null) so one hung agent doesn't stall parallel work.

sandbox.ts
  QuickJS-emscripten context setup. Injects PRELUDE (parallel, pipeline, URL shim), applies deterministic Math.random seeding, enforces memory/deadline limits. Handles async host-function marshaling via QuickJS deferred promises. Manages arena of QuickJS handles for cleanup (no leaks).

workflow.sql.ts, persistence.ts
  Persist workflow runs to SQLite: journal table tracks agent spawns, results, phases, errors. Resume logic reads journal, replays settled agents, re-runs unsettled ones. Script-change detection: if body SHA changed, truncate stale journal (force-run with fresh results). RunSummary persists status, agent count, phase tracking.

resolve.ts, runtime-ref.ts
  Resolve workflow script: local file path, inline script string (detect "export const meta"), node_modules require. runtime-ref wraps the Workflow service as an actor-accessible reference.

builtin.ts
  Ship workflows as builtin named operations (e.g. "deep-research"). Stored in config.skills?.workflows or discovered in filesystem patterns. Each builtin gets wrapper script that calls workflow() host function with name + args.

workspace.ts, events.ts
  File hooks for workflow guest: readFile/writeFile/glob (jailed to workspace root unless overridden). Bus events: WorkflowStarted, WorkflowPhase, WorkflowLog, WorkflowFinished, WorkflowAgentFailed, WorkflowChildFailed.


PLUGIN SYSTEM (packages/opencode/src/plugin/)

index.ts (400+ lines)
  Plugin registry. Built-in plugins hard-imported (MimoAuthPlugin, AnthropicProxyPlugin, CodexAuthPlugin, CopilotAuthPlugin, GitlabAuthPlugin, PoeAuthPlugin, CloudflareAuthPlugin, CheckpointSplitoverPlugin, SubagentProgressCheckerPlugin). External plugins loaded via PluginLoader. Triggers actor.preStop, actor.postStop hooks. Aggregates plugin decisions (continue:boolean, reason:string). Publishes HookEvent.Executed bus events for telemetry.

checkpoint-splitover.ts
  Plugin hook on actor.preStop (checkpoint-writer agent). Validates checkpoint against prior sessions, expected revisions. On violations: continue=true + reason (extraction reflection or error message) to re-enter the actor. Enables checkpoint recovery and multi-pass checkpoint refinement.

loader.ts
  Dynamic import of user plugins from npm or filesystem. Resolves plugin specs (local path, npm package, URL). Validates using @mimo-ai/plugin interface.

matcher.ts, shared.ts, meta.ts
  Plugin spec parsing, actor matching (agentType, tool filters), metadata extraction (name, version, hooks).

install.ts
  npm install / uninstall plugin packages into local ~/.mimocode/.

codex.ts, mimo.ts, mimo-free.ts, github-copilot/copilot.ts, cloudflare.ts
  Auth plugins for Codex, Mimo, Mimo-free, GitHub Copilot, Cloudflare AI Gateway. Each provides OAuth/token exchange and model resolution.


CONFIG SYSTEM (packages/opencode/src/config/)

config.ts (500+ lines)
  Central config schema (Effect Schema + Zod). Merges layers: system defaults, global ~/.mimocode/mimocode.json, project .mimocode/mimocode.json, env var overrides (MIMOCODE_CONFIG, MIMOCODE_CONFIG_CONTENT). Tracks provenance (which file declared each setting). Exports Info (top-level config), normalizes deprecated keys (theme, keybinds, tui moved to tui.json). Config.get() returns Info snapshot; Config.watch() returns Effect stream on changes.

skills.ts
  Skill paths/urls schema: { paths: string[], urls: string[] }. Discovered in order; later ones override earlier matches.

agent.ts
  Agent config schema: { model, variant, temperature, top_p, prompt, permission, hidden, mode (primary|subagent), steps, options, ... }. Loaded from {agent,agents}/**/*.md in config dirs. Normalizes deprecated tools{} into permission{} shape. Renders available agents for Command service.

command.ts
  Command config schema: { agent?, model?, description, template (markdown), subtask? }. Templates use $1..$N placeholders for arguments, $ARGUMENTS for full args string. Built-in commands: init, review, dream, distill, goal, deep-research (if experimental). Also discovers commands from MCP prompts and skills (skill content becomes a command template).

plugin.ts
  Plugin specs: local path, npm package (e.g. "opencode-gitlab-auth"), git URL. ConfigPlugin.resolvePluginSpec normalizes relative paths relative to the declaring config file's directory (prevents "../../" escapes).

markdown.ts
  Parse .md files with YAML frontmatter. Frontmatter stored in data{}, body content in content. Fallback sanitization for malformed YAML.

lsp.ts, mcp.ts, provider.ts, permission.ts, variable.ts, server.ts, paths.ts, layout.ts, history.ts, formatter.ts
  Schemas for LSP servers, MCP servers, provider credentials, file permissions, environment variables, server config, config file paths, TUI layout, shell history size, markdown/JSON formatters.

entry-name.ts
  Convert file path to config entry name: {agent,agents}/build.md -> "build", /.mimocode/agent/foo.md -> "foo".

parse.ts
  JSONC parser with comments + trailing commas. Validates against schema post-parse.


COMMAND SYSTEM (packages/opencode/src/command/)

index.ts (276 lines)
  Unifies built-in commands, config commands, MCP prompt commands, and skill commands into one Command service. Built-in: init (AGENTS.md setup), review (commit/PR/branch review), dream (manual memory consolidation), distill (workflow packaging), goal (stop-condition judge), deep-research (workflow). Sources: "command" (built-in or config), "mcp" (MCP prompt), "skill" (skill loaded as command). Each command provides name, description, template (with $N placeholders), hints (parsed placeholder names). Command service.get(name) resolves at runtime, watching config and skill registry for updates.


FLAG SYSTEM (packages/opencode/src/flag/)

flag.ts (164 lines)
  Runtime feature flags read from process.env. Structure: static booleans (env="true"/"1" -> true), numbers (positive integers), strings (raw env value). Lazy getters for flags evaluated at access time (tests set env vars at runtime). Major flag categories:

  Core behavior: MIMOCODE_MIMO_ONLY (pure-mimo mode, no Claude Code inheritance), MIMOCODE_DISABLE_EXTERNAL_SKILLS (skip .claude/.agents/.codex/.opencode), MIMOCODE_DISABLE_COMPOSE_SKILLS (skip built-in bundled skills).

  Provider: MIMOCODE_DISABLE_PROVIDER_ENV (ignore env-based API keys in mimo-only mode), MIMOCODE_ENABLE_EXPERIMENTAL_MODELS, disabled_providers, enabled_providers.

  Model caps: MIMOCODE_MAX_PROMPT_IMAGES, MIMOCODE_MAX_PROMPT_IMAGE_SIZE, output token limits.

  Analytics: MIMOCODE_ENABLE_ANALYSIS (default true, opt-out via false), MIMOCODE_ALWAYS_NOTIFY_UPDATE.

  Experimental: MIMOCODE_EXPERIMENTAL* (filewatcher, LSP tool, workflow tool, markdown, Exa search, Oxfmt, HTTPAPI, workspaces).

  Storage: MIMOCODE_DB (custom db path), MIMOCODE_DISABLE_CHANNEL_DB (default true: shared db, not per-channel).


SKILL INJECTION INTO AGENT PROMPTS (session/prompt.ts + session/system.ts)

Flow:
1. At agent spawn (session/prompt.ts:236), SessionPrompt.prompt() calls SystemPrompt.skills(agent).
2. SystemPrompt.skills (system.ts:67) checks agent.permission, calls Skill.available(agent) to get non-hidden skills filtered by permission.
3. Skill.fmt(list, {verbose:true}) renders skills as XML block: <available_skills><skill><name>X</name><description>Y</description><location>Z</location></skill>...</available_skills>.
4. Compose skills (composeSkillsBlock()) inserted first (user skills override). External skills follow. Block added to SystemPrompt additions[].
5. additions[] passed to buildLLMRequestPrefix() which assembles final system prompt sections.
6. Agents read available skills from system context. When skill is invoked via command tool, skill source="skill" content injected as <skill_content name="X">...body...</skill_content> XML tag (prompt.ts:3096).

SELF-IMPROVEMENT: Agent-authored skills
- Skills are stored as SKILL.md markdown files on disk (no in-memory-only skills).
- Discovery scans local filesystem + config URLs. New skill files are discovered on next scan/reload.
- No built-in mechanism for an agent to dynamically generate/update skills during execution. Skills distill commands allow manual consolidation (dream, distill agents read session history, propose new skills).
- Skill updates are durable (persisted to disk) but require explicit agent invocation of skill-creation commands, not automatic self-modification.


PORTABLE MECHANISMS FOR HARNESS WITH 11 SELF-AUTHORED SKILLS + FRONTMATTER VALIDATOR + EVAL JSONL

1. **Discovery Multi-Source Hierarchy**
   MiMo combines four discovery layers (compose, external dirs, project tree, config paths/URLs), each with override capability. Port: apply to .claude/skills/.elai_cc-packages/opencode/src/skill/index.ts:156-186 shows exact pattern for EXTERNAL_DIRS looping + upward tree search (fs.up with stop marker). Enables skills distributed across .claude global, project root, and remote URLs without collision.

2. **Frontmatter-First Metadata Extraction**
   ConfigMarkdown.parse(path) extracts YAML frontmatter from .md files without executing code. Port: Reuse the same approach in crates/orchestrator/src/skill_manager.rs — parse markdown file headers, validate schema (name, description, permission?, hidden?), reject non-data expressions. Enables permission-scoped skill visibility (system.ts:68 Permission.disabled check).

3. **Effect-Based Async Layer**
   All I/O (file reads, HTTP discovery, shell spawning) wrapped in Effect<T> for composable, typed async. Concurrent discovery via Effect.forEach with concurrency tuning (skillConcurrency=4). Port: Reapply this in ELAI's memory-bank or skill orchestration — Effect Scope for resource cleanup, Effect.all for parallel batches, Fiber for cancellation/timeout.

4. **Permission-Scoped Availability**
   Skill visibility gated by agent.permission (Permission.evaluate, Permission.disabled). System prompt only includes skills the agent is allowed to use. Port: Extend .claude/rules/wiring-enforcement.md with a permission gate on skill injection — agents with deny:skill never see skill block, audit checks for permission alignment.

5. **Compose Bundle: Build-Time Skill Embedding**
   Bun macro (bundle.macro.ts) walks .bundle/ dirs at compile time, embeds as static object, extracted to disk on first run. Zero-cost, version-pinned, no external HTTP. Port: Replace macro with a build-step script (Python/Node) that generates a Go/Rust const containing ELAI's built-in skill set, extracted to workflow/memory-bank/compose-skills/{version}/. Version-lock enables rollback + deterministic skill discovery.

6. **Never-Throw Async Boundaries**
   Agent timeout + per-run deadline with null return on timeout (workflow/runtime.ts:85-94). One hung agent (e.g. TTFT stall) doesn't block parallel sibling agents. Port: Instrument ELAI's orchestrator with per-agent timeoutMs + aggregate deadline, set agentTimeoutMs=30s default (tunable in E-31 tier matrix), return BLOCKED in handoff if timeout exceeded.

7. **Script Metadata Parsing Without Execution**
   workflow/meta.ts recursive-descent reader rejects any non-data syntax (no function calls, member access, operators). Prevents malicious "export const meta = { name: execSync(...) }" injection. Port: Validate future workflow or skill scripts via static AST walk (swc for TS, ast module for Python) — only allow identifier keys + literal values (string, number, boolean, array/object of literals).

8. **Skill-as-Command Unification**
   Command service (command/index.ts) treats skills as commands with source="skill": skill.content becomes command.template, skill description -> command description. skill tool invoked -> skill content injected as XML body. Port: Wire ELAI's skill registry into orchestrate runner's prompt-builder (already have crates/config/src/agent.rs for agent configs; extend to skills.rs with similar frontmatter validation + permission gating).


FILES TOUCHED (real paths + one line each)

F:\.ELAI_workflow\docs\.source\MiMo-Code\packages\opencode\src\skill\index.ts — central skill registry, four discovery sources, permission filtering, XML output
F:\.ELAI_workflow\docs\.source\MiMo-Code\packages\opencode\src\skill\discovery.ts — HTTP skill repository discovery with concurrent download and caching
F:\.ELAI_workflow\docs\.source\MiMo-Code\packages\opencode\src\skill\compose\extract.ts — extract compile-time bundled skills to disk on startup
F:\.ELAI_workflow\docs\.source\MiMo-Code\packages\opencode\src\skill\compose\bundle.macro.ts — Bun compile-time macro to embed skills as static object
F:\.ELAI_workflow\docs\.source\MiMo-Code\packages\opencode\src\workflow\meta.ts — parse workflow metadata via recursive-descent data reader (no execution)
F:\.ELAI_workflow\docs\.source\MiMo-Code\packages\opencode\src\workflow\runtime.ts — QuickJS sandbox with agent spawning, concurrency control, resume journal
F:\.ELAI_workflow\docs\.source\MiMo-Code\packages\opencode\src\workflow\sandbox.ts — QuickJS context setup, deterministic seeding, memory/deadline limits
F:\.ELAI_workflow\docs\.source\MiMo-Code\packages\opencode\src\workflow\persistence.ts — journal-based resume logic, script-change detection
F:\.ELAI_workflow\docs\.source\MiMo-Code\packages\opencode\src\plugin\index.ts — plugin registry, actor hooks, aggregated decisions
F:\.ELAI_workflow\docs\.source\MiMo-Code\packages\opencode\src\plugin\checkpoint-splitover.ts — plugin hook for checkpoint validation and multi-pass refinement
F:\.ELAI_workflow\docs\.source\MiMo-Code\packages\opencode\src\config\config.ts — merged config schema (global + project + env), provenance tracking
F:\.ELAI_workflow\docs\.source\MiMo-Code\packages\opencode\src\config\skills.ts — skill paths/urls configuration schema
F:\.ELAI_workflow\docs\.source\MiMo-Code\packages\opencode\src\config\agent.ts — agent config schema, permission normalization from deprecated tools{}
F:\.ELAI_workflow\docs\.source\MiMo-Code\packages\opencode\src\config\markdown.ts — YAML frontmatter parsing with fallback sanitization
F:\.ELAI_workflow\docs\.source\MiMo-Code\packages\opencode\src\command\index.ts — unify built-in, config, MCP prompt, and skill commands
F:\.ELAI_workflow\docs\.source\MiMo-Code\packages\opencode\src\flag\flag.ts — runtime feature flags from environment variables
F:\.ELAI_workflow\docs\.source\MiMo-Code\packages\opencode\src\session\prompt.ts — inject skill block into agent system prompt at spawn time
F:\.ELAI_workflow\docs\.source\MiMo-Code\packages\opencode\src\session\system.ts — SystemPrompt.skills() fetches available skills, filters by permission, formats XML
