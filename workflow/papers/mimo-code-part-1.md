MIMO-Code CLI Architecture: Part 1 (packages/opencode/src/cli)

COMMAND CATALOG

Global Flags (all commands)
  --print-logs             print logs to stderr
  --log-level {DEBUG|INFO|WARN|ERROR}  log level
  --pure                   run without external plugins

Core Commands

1. run [message..]
   Entrypoint to initiate mimocode sessions with LLM agent interaction.
   Flags:
     --command <cmd>        the command to run, use message for args
     --continue (-c)        continue the last session
     --session (-s) <id>    session id to continue
     --fork                 fork the session before continuing (requires -c or -s)
     --share                share the session
     --model (-m) <provider/model>  model to use
     --agent <name>         agent to use
     --format {default|json} output format (default: formatted, json: raw JSON events)
     --dir <path>           directory to run in
     --port <number>        port for local server (default: random)
     --variant <effort>     model variant (e.g., high, max, minimal)
     --thinking             show thinking blocks (default: false)
     --file (-f) <path>     file(s) to attach to message
     --title <title>        title for the session
     --attach <url>         attach to running server (e.g., http://localhost:4096)
     --password (-p) <pwd>  basic auth password (defaults to MIMOCODE_SERVER_PASSWORD)
     --dangerously-skip-permissions  auto-approve permissions (dangerous!)
   Behavior: parses message + stdin, handles file attachments, manages session lifecycle
   Invokes: SessionCheckpoint.drainWriters() to wait for background checkpoint writes

2. serve
   Starts a headless mimocode server for remote connections.
   Flags:
     (uses network options from resolveNetworkOptions())
   Behavior: requires MIMOCODE_SERVER_PASSWORD or logs warning
   Invokes: Server.listen(opts), serves HTTP at resolved hostname:port

3. generate [prompt]
   Code generation command (GenerateCommand).

4. account (alias: console)
   Account management (login/logout/list accounts/orgs).
   Subcommands visible in AccountCommand handler; uses Device Code OAuth flow.
   Invokes: Account.Service for auth state

5. providers [provider]
   List or manage provider credentials.
   Flags:
     --verbose              show metadata (costs, etc)
     --refresh              refresh models cache from models.dev
   Behavior: plugin auth dispatch via handlePluginAuth()

6. agent [name]
   Agent management and discovery.
   Subcommand: create
     --path <dir>          directory to generate agent
     --description <text>  what agent should do
     --mode {all|primary|subagent}  agent scope
     --tools <list>        comma-separated tools (bash, read, write, edit, glob, grep, webfetch, actor, task)
     --model (-m) <provider/model>

7. upgrade
   Upgrade mimocode installation.

8. uninstall
   Uninstall mimocode.

9. models [provider]
   List available models.
   Flags:
     --verbose              includes metadata (pricing, limits)
     --refresh              refresh from models.dev
   Behavior: sorts providers (opencode first), lists all or filters by provider

10. stats
    Show token usage and cost statistics.
    Flags:
      --days <N>           last N days (default: all time)
      --tools <N>          number of tools to show (default: all)
      --models [N]         show model statistics (optionally top N)
      --project <name>     filter by project (empty string = current project)
    Output: aggregates SessionStats (totalSessions, totalMessages, totalCost, toolUsage, modelUsage, costPerDay)

11. export <sessionID>
    Export session data to JSON (with optional redaction).
    Behavior: traverses session.messages, parts, tool outputs; redacts file paths/text by default

12. import <file|url>
    Import session from JSON file or share URL.
    Behavior: parses ShareData from opncd.ai/share/* URLs, transforms flat API response to local hierarchy

13. github
    GitHub integration command (GithubCommand).

14. pr <number>
    Fetch and checkout a GitHub PR branch, then run mimocode.
    Behavior:
      1. Uses `gh pr checkout <number> --branch pr/<number> --force`
      2. Parses PR metadata (fork handling, session link detection)
      3. Extracts opncd.ai/s/<id> session link if present
      4. Spawns `mimo -s <sessionID>` to resume

15. session
    Manage sessions.
    Subcommands:
      import-claude          import Claude Code sessions (~/.claude/projects) into mimocode
        --force              re-sync ignoring mtime cache
      delete <sessionID>     delete a session

16. mcp
    Model Context Protocol (MCP) integration.
    Behavior: configures remote/local MCP servers, OAuth flow, auth status tracking

17. debug
    Debugging commands for development.
    Subcommands:
      agent <name>          show agent configuration
        --tool <id>         execute a specific tool
        --params <json>     tool params as JSON or JS object
      config                show debug config
      file                  debug file operations
      lsp                   LSP integration debug
      ripgrep               ripgrep integration debug
      snapshot              session snapshot debug
      skill                 skill debug

18. acp
    Account/console auth provider command.

19. web
    Web server command for browser-based access.

20. plug
    Plugin management.
    Behavior: resolvePluginTarget(), installPlugin(), patchPluginConfig()

21. db
    Database management command.

22. attach <url>
    Attach to running mimocode server.
    Flags:
      --dir <path>         directory to run in (or remote path)
      --continue (-c)      continue last session
      --session (-s) <id>  session id to continue
      --fork               fork the session
      --password (-p) <pwd> basic auth (defaults to MIMOCODE_SERVER_PASSWORD)
    Behavior: win32 Ctrl+C guard, TLS/basic auth headers, spawns tui()

23. tui-thread (internal)
    Thread management for TUI operations.

TUI ARCHITECTURE (10 lines max)

TUI entry: tui() renders a Solid.js (OpnTUI + @opentui/core) terminal UI via CliRenderer. 
Core app: app.tsx (35KB) provides RouteProvider (Home/Session routes), DialogProvider (20+ dialogs: model, MCP, login, etc), 
context providers (Theme, Keybind, Language, SDK, Sync, Local, KV, Project, Args, CommandDialog, ToastProvider, ExitProvider).
Renderer: fixed 60 FPS on TTY, 10 FPS on pipe; supports mouse (MouseButton tracking); Win32 raw input guards via win32.ts.
Components: 60+ dialogs (dialog-*.tsx) + routes (session.tsx, home.tsx) + TUI plugins (feature-plugins/) for extensibility.
Event loop: RouterProvider.useRoute() + KeybindProvider for command dispatch; session rendering via Session route.

WIRING & COMMAND TRIGGERING

Entry Point: src/index.ts registers all commands via yargs. Middleware runs:
  1. Log.init() (debug/info/warn/error)
  2. Heap.start() (memory profiling)
  3. Env vars: MIMOCODE=1, AGENT=1, MIMOCODE_PID
  4. One-time DB migration (JsonMigration)
  5. Claude import (idempotent per env guard MIMOCODE_CLAUDE_IMPORTED)

Memory & Checkpoint:
  - SessionCheckpoint.Service manages detached background writers (up to 120s drain before teardown)
  - Used by run command to prevent early exit killing pending LLM-response checkpoints
  - bootstrap() wraps all command handlers for proper cleanup

Session Continuation & Forking:
  - run --session/-s <id> resumes existing session
  - run --continue/-c resumes last session (db lookup)
  - run --fork creates a branch (requires -c or -s)
  - pr <number> auto-detects opncd.ai/s/<id> link in PR body, imports, resumes

Model & Agent Selection:
  - run --model provider/model overrides default (stored in Config)
  - run --agent <name> selects Agent from Agent.Service
  - models command lists from Provider.Service (integrates models.dev)
  - account command switches active account/org

Permissions & Tools:
  - run --dangerously-skip-permissions auto-approves (default: prompt per tool)
  - Tools registered: Glob, Grep, Read, Write, Edit, WebFetch, WebSearch, CodeSearch, Bash, Skill, Actor (subagent spawn)
  - Tool output rendered as inline blocks (icons: ✱=glob/grep, →=read, ←=write, $=bash, ◇=codesearch, ◈=websearch, etc)

Server & Remote:
  - serve starts headless server (no auth if MIMOCODE_SERVER_PASSWORD unset)
  - run --attach <url> + --password connects to remote server (HTTP Basic Auth)
  - attach <url> CLI wraps attach command for TUI mode

Stats & Telemetry:
  - stats command queries SessionTable, aggregates tokens/cost/toolUsage by model/provider
  - Costs tracked via Provider.models[].pricing
  - Cost-per-day, median tokens/session calculated at query time

PORTABLE MECHANISMS (CLI/UX Design Worth Stealing)

1. Message Parsing with Pass-Through Args
   Location: run.ts::handler() lines 293-295
   Pattern: `[...message, ...(args["--"] || [])]` captures positional + double-dash-terminated args
   Why: Allows `mimo run "prompt" -- --flag=value` to pass flags to the underlying command
   Steal: Use yargs parserConfiguration({ "populate--": true }) to separate tool args from command args

2. Rich Tool Output Rendering
   Location: run.ts::glob(), grep(), read(), write(), bash(), etc (lines 77-194)
   Pattern: ToolProps<T> carries input + metadata + part.state; render is dispatch-based per tool type
   Icons used: ✱ (glob/grep), → (read), ← (write), $ (bash), ◇ (codesearch), ◈ (websearch), ✗/•/✓ (task status)
   Why: Inline icons + normalized descriptions (match counts, file paths, descriptions) give readable streaming logs
   Steal: Use metadata object in tool response to surface counts; normalize paths with normalizePath() for brevity

3. Detached Background Checkpoint Writers with Drain
   Location: bootstrap.ts::lines 18-28
   Pattern: SessionCheckpoint.drainWriters() waits up to 120s for ALL pending writers before exit
   Why: Headless `mimo run` spawns writers that outlive the main response; early exit kills them mid-write
   Steal: Implement a task pool with grace period; fail-soft (abandon tasks that don't settle in time)

4. Device Code OAuth Flow (Account Login)
   Location: account.ts::loginEffect() lines 39-78
   Pattern: 
     a. service.login(url) returns { url, user (device code), interval, expiry }
     b. User visits URL, enters device code
     c. Poll with exponential backoff (start interval, increase by 5s on PollSlow)
     d. Timeout after expiry (PollExpired)
     e. Match result tags: PollSuccess | PollDenied | PollExpired | PollError
   Why: No password entry; works with firewalled networks; user sees a single code
   Steal: reusable pattern for app-driven auth; Effect.timeout(duration) + Effect.sleep(wait) for polling

5. Plugin Installation & Config Patching
   Location: plug.ts::createPlugTask() lines 70+, providers.ts::handlePluginAuth() lines 32+
   Pattern: 
     a. resolvePluginTarget() maps npm spec to local path
     b. readPluginManifest() reads package.json hooks
     c. Plugin auth handlers (plugin.auth.methods) prompt user (select | text)
     d. Conditional prompts via when: { key, op: "eq"|"ne", value }
     e. patchPluginConfig() merges into ~/.config/mimocode/mimocode.json or project .mimocode.json
   Why: Plugins extend CLI; conditional prompts prevent unnecessary questions; JSONC editing preserves comments
   Steal: manifest-driven config patching with when-gates; reusable handlePluginAuth for third-party integrations

6. Session Import from Share URLs
   Location: import.ts::parseShareUrl(), transformShareData() lines 23-74
   Pattern:
     a. Extract ID from https://opncd.ai/share/<id>
     b. ShareNext API returns flat array: [session, msg1, msg2, part1, part2, ...]
     c. transformShareData() re-nests: { info: session, messages: [{ info, parts }, ...] }
   Why: API is simpler (one query); client rebuilds hierarchy; symmetric with export
   Steal: Flat-array API is more efficient; nesting on import is cheap transformation

7. Model Variant Selector (Reasoning Effort)
   Location: run.ts::builder() lines 277-280, dialog-variant.tsx
   Pattern: --variant <effort> (provider-specific: OpenAI=high/max, Anthropic=thinking/etc)
   Why: LLM reasoning modes are provider-specific; one CLI flag hides the mapping
   Steal: Normalize provider reasoning/variant modes into one --variant flag; Provider.models[] stores the enum

8. Terminal Capability Detection
   Location: app.tsx::isPlainTerminal(), win32.ts
   Pattern:
     a. isPlainTerminal() checks if stderr is redirected or piped (not a TTY)
     b. targetFps = plainTerminal ? 10 : 60
     c. mouseEnabled = !plainTerminal && !DISABLE_MOUSE && config.mouse
   Why: Piped output should not animate; pipe-friendly FPS; mouse only on real terminal
   Steal: Degrade gracefully based on TTY detection; config files override detection

9. Error Formatting & Redaction (Export)
   Location: export.ts::redact() lines 12-19, filepart() lines 61-68
   Pattern: redact(kind, id, value) returns `[redacted:${kind}:${id}]` if value.trim() is truthy
   Why: Export redaction for sharing sessions without exposing file paths/text
   Steal: Configurable redaction templates; discriminate between secret-types (file-path, file-text, token, etc)

10. Unified Command Entrypoint with Middleware
    Location: src/index.ts::middleware() lines 91-167
    Pattern:
      a. yargs middleware() runs before command dispatch
      b. Performs setup: Log.init(), Heap.start(), env vars, DB migration, Claude import
      c. Setup failures are non-fatal (logged, continue)
      d. All commands inherit the setup
    Why: Centralized initialization; idempotent (Claude import gated by env var)
    Steal: Middleware for cross-cutting concerns (logging, DB, cleanup); env guards for idempotency
