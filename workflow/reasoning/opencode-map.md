OpenCode architecture + capability map
Source tree: C:\Users\user\Downloads\opencode-dev\opencode-dev
Every claim cites a file path + line range read this session. Anything not read is marked UNVERIFIED.

================================================================
1. PURPOSE & STACK
================================================================
WHAT: "The open source AI coding agent" (README.md:10). Terminal-first (TUI) coding agent with a client/server architecture, plus desktop/web/IDE surfaces.
LICENSE: MIT, "Copyright (c) 2025 opencode" (LICENSE:1-3).
LANGUAGES/RUNTIME: TypeScript on Bun. bun.lock present (847KB), bunfig.toml, turbo.json, flake.nix at root. AGENTS.md:27 mandates Bun APIs ("Use Bun APIs when possible, like Bun.file()"). Heavy use of the Effect framework (effect, @effect/platform-node) throughout — every service is an Effect Layer (provider.ts:21, auth/index.ts:3, compaction.ts:14). Drizzle + SQLite for persistence (packages/effect-drizzle-sqlite, AGENTS.md:118-136 Drizzle schema rules).
MONOREPO: bun workspaces, ~24 packages (packages/: app, cli, console, containers, core, desktop, docs, effect-drizzle-sqlite, effect-sqlite-node, enterprise, function, http-recorder, identity, llm, opencode, plugin, sdk, server, slack, stats, storybook, tui, ui, web). Two big code packages:
  - packages/opencode/src = 358 files / ~79,943 lines (the main current agent runtime; has its own provider/, session/, tool/, agent/, mcp/, lsp/, permission/, plugin/, skill/, acp/, etc.)
  - packages/core/src = 314 files / ~32,856 lines (shared lib: models-dev, github-copilot, database, system-context, v1 config/session, ripgrep, effect helpers)
  - tui ~27,244 lines, llm ~9,048, server ~2,779, plugin ~1,211, cli ~566 (thin wrapper).
  - Total TS/TSX across packages (no node_modules/dist) = 2,578 files. (counts via find+wc, this session.)
  NOTE: TUI source is TypeScript here (find -name '*.go' in packages/tui = 0), not the Go TUI some older builds shipped. UNVERIFIED whether a separate Go TUI exists outside this tree.
MATURITY SIGNALS (strong):
  - 532 *.test.ts files (find packages -name '*.test.ts', this session). AGENTS.md:138-142 "Avoid mocks; test actual implementation."
  - 25 GitHub Actions workflows (.github/workflows/): test.yml, typecheck.yml, publish.yml, beta.yml, deploy.yml, containers.yml, review.yml, storybook.yml, plus PR-management/triage/compliance automation.
  - Husky git hooks (.husky/), oxlint (.oxlintrc.json), prettier, editorconfig.
  - 23 translated READMEs (README.<lang>.md), docs locale-sync workflow.
  - Published npm package "opencode-ai" (README badge), install script (install, 13KB), brew/choco/scoop uninstall paths (cli/cmd/uninstall.ts:187-189).
  - Hosted control plane: console.opencode.ai (cli/cmd/account.ts:18), opencode.ai/auth, opencode.ai/discord.
  - SDK generation pipeline (AGENTS.md:1), ACP (Agent Client Protocol) support (src/acp/, service.ts:130 "OpenCode").

================================================================
2. PROVIDER / MODEL SYSTEM  (most important)
================================================================
CORE ABSTRACTION: built on the Vercel AI SDK. provider.ts:7 imports from "ai"; provider.ts:12 "@ai-sdk/provider" LanguageModelV3. Model CATALOG comes from models.dev (core/models-dev.ts fetches+caches the catalog; cli/cmd/models.ts:23 "refresh the models cache from models.dev"; provider.ts:358,487 reference models.dev metadata). Each model carries cost/limit/capability metadata (models-dev.ts:18-60 Cost, CostTier, Model schemas).

BUNDLED PROVIDERS (provider.ts:107-134, lazy dynamic import of each @ai-sdk/* pkg): amazon-bedrock (+mantle), anthropic, azure, google, google-vertex (+anthropic), openai, openai-compatible, openrouter, xai, mistral, groq, deepinfra, cerebras, cohere, gateway (Vercel AI Gateway), togetherai, perplexity, vercel, alibaba, gitlab-ai-provider, github-copilot (custom openai-compatible adapter at core/github-copilot/copilot-provider), venice-ai-sdk-provider. package.json deps confirm all the @ai-sdk/* versions (e.g. @ai-sdk/anthropic 3.0.82, @ai-sdk/openai 3.0.53, @openrouter/ai-sdk-provider 2.9.0, ai-gateway-provider 3.1.2). Plus extra custom loaders in provider.ts custom(): llmgateway (provider.ts:443), zenmux (581), cloudflare-ai-gateway (754, uses ai-gateway-provider). "Other"/arbitrary openai-compatible provider id supported via auth login (providers.ts:437-454).

AUTH MODES (auth/index.ts:14-36): three credential kinds — Oauth {refresh,access,expires,accountId,enterpriseUrl}, Api {key}, WellKnown {key,token}. Stored at Global.Path.data/auth.json mode 0600 (auth/index.ts:10, set() 0o600), or injected via env OPENCODE_AUTH_CONTENT (auth/index.ts:57). BYOK API keys AND OAuth both first-class. provider/auth.ts handles the OAuth dance (auth.ts:168-214: authorize -> exchange code -> store type:"oauth"). `opencode auth login [url]` CLI (cli/cmd/providers.ts:300-468). Login priority order opencode(0) > openai(1) > github-copilot(2) > google(3) > anthropic(4) > openrouter(5) > vercel(6) (providers.ts:371-379). OAuth hints: openai = "ChatGPT Plus/Pro or API key" (providers.ts:400). So Claude / ChatGPT-subscription / GitHub-Copilot OAuth sign-in are all wired (subscription-as-backend, not just raw API keys).

*** FREE-API VERDICT: YES — anonymous free tier exists, verified in code. ***
The first-party "opencode" provider (the opencode-zen gateway) has a custom loader (provider.ts:179-201). Logic, verbatim semantics:
  - hasKey = any of provider.env vars set; ok = hasKey OR a stored auth for "opencode" OR config apiKey (provider.ts:181-188).
  - if (!ok) it loops every model and DELETES any model whose cost.input !== 0, KEEPING ONLY models where cost.input === 0 (provider.ts:190-195: `if (value.cost.input === 0) continue; delete input.models[key]`).
  - it then sets options apiKey:"public" (provider.ts:199) and autoloads if any models remain (provider.ts:198).
  => Translation: with NO key and NO login, opencode auto-exposes the opencode-zen provider's ZERO-COST models under a "public" anonymous key. Authenticate (api key from opencode.ai/auth, providers.ts:467) and the paid models unlock. This is the concrete "can use free APIs out of the box" mechanism, grounded.
Additional free-ish paths (table-stakes, just config): local/openai-compatible (ollama etc.) via openai-compatible + custom baseURL (provider.ts:117, 717 proxy/baseURL handling); OpenRouter free models (provider bundled, :408 it's offered in login as plugin/standard); the "opencode" provider is marked hint:"recommended" in the login picker (providers.ts:399). FREE WEB SEARCH is also gated to the opencode provider: webSearchEnabled() returns true when providerID === opencode (tool/registry.ts:55-57) — so free users get built-in websearch through zen.

ROUTING / FALLBACK / PER-AGENT MODEL:
  - Per-agent model selection: the TaskTool/subagent picks subagent_type and each agent has its own model (agent/agent.ts via tools(model{providerID,modelID,agent}) in registry.ts:73-78). Built-in helper agents (explore, summary, title, compaction) have their own prompts (agent/prompt/*.txt).
  - Retry/fallback on free-tier and rate limits: session/retry.ts:12 RetryReason = "free_tier_limit" | "account_rate_limit"; retry.ts:80 emits reason:"free_tier_limit" — so it detects free-tier exhaustion and retries/backs off (the gateway also auto-handles overflow). 
  - A NATIVE LLM runtime (session/llm/native-runtime.ts) bypasses the AI SDK for openai/anthropic/opencode providers (native-runtime.ts:53-55 status() only "supported" for those three or opencode*), with the AI SDK (ai-sdk.ts) as the general fallback path for everyone else. So the hot providers get a hand-tuned streaming client; the long tail rides the SDK.

================================================================
3. AGENT HARNESS
================================================================
AGENT LOOP: streaming Effect-based runtime. session/llm/native-runtime.ts is the native streaming loop (LLMEvent stream over @opencode-ai/llm, native-request.ts) for openai/anthropic/opencode; session/llm/ai-sdk.ts is the AI SDK loop for everything else (selected via native-runtime.ts:53 status()). AGENTS.md:148-158 ("V2 Session Core") describes a durable, crash-recoverable execution model: SessionV2.prompt() admits a durable session_input row, SessionExecution.wake() schedules a serialized runner that promotes admitted inputs into visible messages at safe boundaries; "one explicit llm.stream(request) per provider turn"; process-global Session coordinator with location-scoped runner/tools/permissions. This is well beyond a naive while-loop — it is a durable job queue with resume semantics (background/job.ts, session/run-state.ts).

BUILT-IN TOOLS (packages/opencode/src/tool/, each .ts + a .txt prompt): edit, write, read, glob, grep, shell (bash, tool/shell/), task (subagent spawn), todo (todowrite), webfetch, websearch, skill, lsp, apply_patch, plan (plan-enter/plan-exit — a read-only "plan mode"), question (ask-the-user tool), external-directory, mcp-websearch, invalid (fallback). Registered in tool/registry.ts:4-30. Tools are model/agent-scoped (registry.ts tools(model) filters per agent). Custom/plugin tools merge in (registry State.custom, ToolDefinition from @opencode-ai/plugin).

SESSION PERSISTENCE: Drizzle + SQLite (core/database/database.ts; AGENTS.md:118 schema rules). Durable session_input rows, EventV2 replay (AGENTS.md:157), snapshot/ + storage/ + sync/ dirs. revert.ts gives undo/checkpointing of session edits (session/revert.ts). Sharing via share/ (opencode.ai share links).

CONTEXT MGMT / COMPACTION (session/compaction.ts + overflow.ts): token-budget driven, not turn-count. overflow.ts:9 usable() computes context budget = model.limit.input - reserved (reserved = COMPACTION_BUFFER 20_000 or maxOutputTokens). isOverflow() (overflow.ts:21) fires when total tokens >= usable; respects cfg.compaction.auto===false to disable. compaction.ts then summarizes: PRUNE_MINIMUM 20_000, PRUNE_PROTECT 40_000, TOOL_OUTPUT_MAX_CHARS 2_000, DEFAULT_TAIL_TURNS 2, MIN/MAX_PRESERVE_RECENT 2_000/8_000 (compaction.ts:37-43). It prunes old tool outputs (truncating to 2K chars), protects recent turns, protects skill outputs (PRUNE_PROTECTED_TOOLS=["skill"]), and writes a summary message (buildPrompt from core/session/compaction, agent/prompt/compaction.txt + summary.txt). Emits session.compacted event. This is mature, turn-aware, token-accurate context management.

SUBAGENTS / TASK DELEGATION (tool/task.ts): the `task` tool spawns a subagent of subagent_type with its own session/permissions (deriveSubagentSessionPermission, agent/subagent-permissions.ts). Supports BACKGROUND mode (task.ts:24-40 background=true returns immediately, notifies on completion via BackgroundJob) and FOREGROUND (default), and RESUME (task_id continues a prior subagent session, task.ts:50-53). Custom agents are user-definable as markdown (.opencode/agent/*.md, e.g. duplicate-pr.md, triage.md).

PERMISSIONS / APPROVAL (permission/index.ts + evaluate.ts + arity.ts): a wildcard rule engine. evaluate() (permission/index.ts:38) matches permission+pattern against rulesets via Wildcard.match and returns action allow/deny/ask, default "ask" (index.ts:44-46). Async approval: ask() creates a Deferred the runner awaits until the user replies (index.ts pending Map + Deferred). Emits permission.asked / permission.replied events. Subagents get derived (usually tightened) permissions. Plan mode is a read-only restriction layer (plan.ts).

LSP INTEGRATION (lsp/: client.ts, server.ts, diagnostic.ts, language.ts, launch.ts, lsp.ts): full LSP client. lsp/language.ts maps ~120 entries (grep -c ':' = 120) of language->server config; launches language servers, surfaces diagnostics, and exposes an `lsp` tool (tool/lsp.ts) so the model can query hover/definition/diagnostics. This is real IDE-grade code intelligence, not just text search.

MCP SUPPORT (mcp/: index.ts, catalog.ts, auth.ts, oauth-provider.ts, oauth-callback.ts): MCP client with a server catalog AND full OAuth-2 client flow for MCP servers (oauth-provider.ts + oauth-callback.ts) — i.e. it can authenticate to remote OAuth-gated MCP servers, beyond plain stdio MCP.

================================================================
4. EXTENSIBILITY & POLISH
================================================================
PLUGINS (packages/plugin/src/: index.ts, tool.ts, shell.ts, tui.ts, example.ts, example-workspace.ts; @opencode-ai/plugin): typed plugin API. Plugins can add providers (config/plugin/, plugin/provider, providers.ts:380 resolvePluginProviders + login hint "plugin"), tools (ToolDefinition), commands, skills, and hook auth flows (Hooks, auth.ts:2). core/src/plugin/ has command/, provider/, skill/, models-dev/ extension points.
SKILLS (skill/index.ts + discovery.ts; tool/skill.ts; PRUNE_PROTECTED_TOOLS=["skill"]): an Anthropic-style skills system — discoverable skill packages the model invokes via the `skill` tool, with skill outputs protected from compaction pruning (compaction.ts:40). core/src/skill/ + core/src/plugin/skill/.
CUSTOM COMMANDS (command/index.ts + template/; .opencode/command/*.md, e.g. commit.md, changelog.md, rmslop.md, spellcheck.md): user-defined slash-commands as markdown templates.
ARCHITECTURE: client/server split — packages/server (HTTP server, 46 files) + packages/sdk (generated client) + packages/tui (the terminal UI talks to the server) + packages/app + packages/desktop (Tauri, containers/tauri-linux) + packages/web + packages/console (hosted control plane, console.opencode.ai). ACP (src/acp/) lets external editors (Zed etc.) drive opencode as an agent backend. IDE integrations: src/ide/, a published VS Code extension (publish-vscode.yml workflow), .zed/ config.
DOCS: packages/docs (Mintlify-style, ai-tools/, essentials/, snippets/), docs-update + docs-locale-sync CI. STATS.md + packages/stats track adoption. storybook package + storybook.yml for UI components.
WHAT MAKES IT FEEL POLISHED: 532 tests + 25 CI workflows + typecheck/lint gates; durable crash-recoverable V2 session core; hand-written native streaming clients for the 3 hot providers on top of the SDK fallback; hosted gateway with a free anonymous tier; 23-language localized docs/READMEs; multi-surface (TUI/desktop/web/IDE/ACP); generated SDK; enterprise package + console control plane.

================================================================
5. DISTINCTIVE vs COMMODITY
================================================================
GENUINELY DISTINCTIVE ENGINEERING:
  - The opencode-zen gateway with a code-verified FREE anonymous tier (provider.ts:179-201): zero-config, no-login access to zero-cost models + free websearch (registry.ts:55-57). Most agents require the user to bring a key. This is a real product/infra moat, not just code.
  - Durable, crash-recoverable "V2 Session Core" (AGENTS.md:148-158): admit-durable-input -> serialized runner -> resume, EventV2 replay, session revert/checkpointing (revert.ts). This is distributed-systems-grade session management most coding agents lack.
  - Dual LLM runtime: hand-tuned native streaming client (native-runtime.ts/native-request.ts) for openai/anthropic/opencode, AI-SDK fallback for the 22+ other providers — performance where it matters, breadth everywhere else.
  - OAuth everywhere: subscription-backend OAuth (Claude/ChatGPT/Copilot) AND OAuth MCP clients (mcp/oauth-provider.ts). Few harnesses do OAuth-gated MCP.
  - Multi-surface client/server + ACP: one server backing TUI, desktop (Tauri), web, IDE, and external editors via Agent Client Protocol.
COMMODITY / TABLE-STAKES (everyone has these, opencode's are just well-built):
  - The tool set (read/write/edit/glob/grep/bash/webfetch) — standard.
  - models.dev catalog + Vercel AI SDK provider plumbing — opencode helped popularize it but it's now common.
  - Token-budget compaction/summarization — standard pattern (theirs is turn-aware and tool-output-pruning, a notch above).
  - Subagents / task delegation, wildcard permission prompts, LSP diagnostics, MCP client, skills, custom commands/agents — all present in the current generation of agents; opencode's LSP (120 languages) and permission engine are on the thorough end.

================================================================
UNVERIFIED / NOT CONFIRMED FROM CODE
================================================================
- Exact list of which models the opencode-zen FREE tier currently serves (cost.input===0 set) — that comes from the live models.dev catalog at runtime, not from a static file in the tree.
- Whether a separate Go TUI binary ships in releases (this tree's TUI is TypeScript; 0 .go files found).
- Real-world latency/throughput of the native runtime vs AI SDK path (no benchmark read).
- Precise OAuth provider list beyond openai/google/anthropic/github-copilot/opencode visible in providers.ts priority map (plugin-contributed providers add more at runtime).
- packages/core also holds a parallel/older "v1" + github-copilot chat/responses implementation; the live/canonical path is packages/opencode/src (newer, larger). Did not fully diff v1 vs v2 ownership.
