MiMoCode Part 3: Tool Registry, Patch System, Permission Model, User Interaction

TOOL ARCHITECTURE

packages/opencode/src/tool/tool.ts
Purpose: Base type system for all tools (commands the agent can invoke).
Key exports: Metadata, Context, ExecuteResult, Def, DefWithoutID, Info, InferParameters, InferMetadata, define(), init()
Mechanism: Tools are defined via Tool.define(id, Effect) which wraps them with metadata tracking, output truncation, parameter validation (zod schemas), and span instrumentation. Execution is async (Effect monad); each tool receives a Context containing sessionID, messageID, agent name, task id, abort signal, and a callback to ask() permissions. The wrapper auto-truncates output via Truncate.Service and attaches a truncated=true flag to metadata when the output exceeds limits.
Notable: Output truncation is always applied (unless the tool itself sets metadata.truncated); shell-mode tools optionally support a parse() and recover() variant for command-line invocation; all tool execution is traced with "Tool.execute" span.

packages/opencode/src/tool/registry.ts
Purpose: Central registry of all ~20 builtin tools + plugin-supplied custom tools.
Key exports: Interface (ids, all, named, tools), Service (Context.Service), layer (Layer.effect)
Mechanism: The registry initializes all builtin tools (bash, read, write, edit, glob, grep, patch, question, lsp, plan, memory, history, task, workflow, actor, skill, fetch, search, code) conditionally: question and lsp are gated by flags (MIMOCODE_ENABLE_QUESTION_TOOL, MIMOCODE_EXPERIMENTAL_LSP_TOOL). Custom tools load dynamically from {tool,tools}/*.{js,ts} in config directories + plugins. Plugin tools are wrapped via fromPlugin() to normalize async Promise returns into Effect. The registry also exposes a filtered tools() method that returns only tools the current (agent, provider, model) tuple is allowed to use.
Notable limits: ~20 builtin tools, dynamic load from glob pattern, plugins merged at init time, question tool disabled by default in API mode.

packages/opencode/src/tool/truncate.ts
Purpose: Output budget enforcement (memory + token control).
Key exports: MAX_LINES (2000), MAX_BYTES (50KB), DIR, RETENTION (7 days), output()
Mechanism: When a tool returns output that exceeds maxLines or maxBytes, truncate.output() writes the full text to TRUNCATION_DIR and returns a 2KB preview + filepath hint. The strategy is "head+tail": if errors are detected in the last 2KB (ERROR_PATTERN regex: error|exception|failed|fatal|traceback|panic|exit code), split 70% budget to head and 30% to tail; otherwise show head only. Under pressure (pressureCaps=true), halve both limits. The file is auto-cleaned after 7 days. Truncated output includes a hint: if the agent has actor tool permission, suggest using actor+grep/read; otherwise use grep/read with offset/limit.
Notable: Error detection is tail-scanning only (last 2048 chars); truncated flag always lands in metadata.truncated; directory cleanup is 7-day retention per ascending ID timestamp.

PATCH & APPLY_PATCH

packages/opencode/src/patch/index.ts
Purpose: Parse and validate patch format (add file, delete file, update file with chunks).
Key exports: PatchSchema, Hunk, ApplyPatchArgs, UpdateFileChunk, AffectedPaths, parsePatch(), deriveNewContentsFromChunks()
Mechanism: Patch is a custom format (not unified diff): *** Add File: path / *** Delete File: path / *** Update File: path (+ optional *** Move to: newpath). Update hunks contain old_lines and new_lines arrays + optional change_context (for correctness checking) and is_end_of_file flag. The parser is line-based; parsePatch() returns { hunks, affectedPaths }. deriveNewContentsFromChunks() replays update chunks on the old content to produce new content (applies chunks sequentially, stops at is_end_of_file marker).
Notable thresholds: hunks stored as an array; no line count or byte limit in the patch format itself; move_path is optional per-hunk, not global.

packages/opencode/src/tool/apply_patch.ts
Purpose: Tool that applies patches to the filesystem with validation, diffing, and permission checking.
Key exports: ApplyPatchTool
Mechanism: Tool receives patchText, parses it into hunks (fail if empty), then for each hunk: resolves the path relative to session CWD, checks write permission via assertWriteAllowed(), loads old content, computes new content, generates a unified diff, and collects stats (additions/deletions). All hunks are validated before any are written. Emits a Bus event for each change (NewFile, UpdatedFile, DeletedFile). The tool returns a summary with per-file diffs, total additions/deletions, and a unified patch view.
Notable: Permission check happens per-file before write; diffs are generated using diff library (createTwoFilesPatch); move operations are recorded but moved file content is the new content (not a separate rename event); return includes metadata.truncated if output is large.

PERMISSION MODEL

packages/opencode/src/permission/schema.ts
Purpose: Type alias for permission IDs (ascending-ordered identifiers).
Key exports: PermissionID (newtype)
Mechanism: PermissionID wraps an ascending ID string (e.g. "permission_2026-06-12T12:34:56.789Z_abcd1234"). Used as a key in pending permission requests.

packages/opencode/src/permission/evaluate.ts
Purpose: Rule matching for permission evaluation.
Key exports: evaluate(permission, pattern, ...rulesets) -> Rule
Mechanism: Flatten all rulesets, findLast matching rule where both permission and pattern match using Wildcard.match() (glob-style). Default verdict is "ask" if no rule matches.
Notable: Last match wins (overriding prior rules); default action is "ask" not "deny".

packages/opencode/src/permission/index.ts
Purpose: Central permission system: ask, reply, rule evaluation, event bus integration.
Key exports: Action (allow|deny|ask), Rule, Ruleset, Request, Reply (once|always|reject), AskInput, RejectedError, DeniedError, CorrectedError, Interface (ask, reply, list), Service
Mechanism: Permission.ask({ permission, patterns, metadata, interactive, ruleset }) evaluates rules, then branches: if denied, fail with DeniedError; if allowed, succeed immediately; if ask, emit permission.asked event on the bus and wait for a permission.replied event or timeout. Request and Reply are first-class types; pending requests are tracked in a Map<PermissionID, PendingEntry> keyed by ascending ID. The interactive flag (default true) controls whether ask blocks on a Deferred or fails immediately (used for non-interactive agents). Approval metadata includes projectID and patterns (for granting blanket approval).
Notable gating: three-action system (allow/deny/ask); default ask behavior blocks until user replies or timeout; interactive=false fails fast instead of blocking.

QUESTION TOOL & USER INTERACTION

packages/opencode/src/question/schema.ts
Purpose: Type alias for question IDs (ascending-ordered identifiers).
Key exports: QuestionID (newtype)
Mechanism: Same as PermissionID (ascending ID wrapper).

packages/opencode/src/question/index.ts
Purpose: Tool for agent to ask multi-choice questions, with optional custom answers.
Key exports: Option, Info, Prompt, Tool, Request, Answer, Reply, Interface (ask, neverAsk, reply, list), Service
Mechanism: Agent calls question tool with an array of Question.Prompt (question text, header label, options array, optional multi/custom flags). The service emits question.asked event on the bus and waits for a question.replied event containing an answers array (each answer is an array of selected option labels, or custom text if custom=true). If neverAsk() is true (headless mode), the tool returns "[Never-Ask] The model will decide autonomously" and the agent re-evaluates the options itself; this mode logs the auto-decision so the user can review it later. Supports i18n via optional key + params for client-side translation.
Notable: Option labels are 1-5 words; answers are arrays of strings (multiple selections); header is max 30 chars; neverAsk mode gives the agent autonomy instead of blocking.

packages/opencode/src/tool/question.ts
Purpose: Glue tool that exposes Question.Service to agents.
Key exports: QuestionTool
Mechanism: Wraps Question.ask() with neverAsk check. If neverAsk() is true, returns a synthetic answer "[Never-Ask] The model will decide..." in metadata.answers; otherwise waits for actual user reply and returns answers array. Output message tells the agent the user's answers or that it must decide autonomously.
Notable: Single entry point for agents; auto-resolves in headless mode.

PORTABLE MECHANISMS (designs worth stealing for a Rust orchestrator)

1. Effect-Based Tool Wrapping (tool.ts)
   Files: tool.ts::wrap(), tool.ts::define()
   Design: Wrap each tool with Effect.gen for automatic instrumentation (tracing, validation, truncation, error formatting). The wrapper sits between the tool's raw execute() and callers, injecting zod validation, span attachment (tool.name, session.id, call.id), and automatic output truncation. This decouples tool logic from operational concerns. For Rust, use a macro like `#[tool]` that wraps an async fn with tracing::instrument, permission checks, and output truncation applied uniformly.

2. Output Truncation with Error Heuristics (truncate.ts)
   Files: truncate.ts::output(), MAX_LINES=2000, MAX_BYTES=50KB, ERROR_PATTERN
   Design: Output that exceeds limits is written to a dated file; preview shows head + tail if tail has errors, else head only. Allocate 70/30 split when errors detected. The heuristic (scan last 2KB for error keywords) is cheap and catches most failures. For Rust, implement as a post-execution middleware that inspects stderr + last N bytes of stdout before truncation.

3. Wildcard Permission Evaluation (permission/evaluate.ts)
   Files: permission/evaluate.ts::evaluate(), permission/index.ts::Rule
   Design: Rule matching is findLast(permission matches pattern AND pattern matches target). Last rule wins; default action is ask. This is simpler than DAC/RBAC and allows override (later rules clobber earlier). For Rust, use a trie or glob matcher (globset crate) instead of loop-findLast for scale; emit the matched rule in logs so users can debug their ruleset.

4. Ascending ID Ordering (permission/schema.ts, question/schema.ts)
   Files: PermissionID, QuestionID, Identifier.ascending()
   Design: IDs are newtype'd strings using timestamp + random suffix (e.g. "permission_2026-06-12T..."). Ascending means sorted by creation time; useful for cleanup (delete entries older than 7 days). For Rust, use ulid or a custom struct with (timestamp, nonce) that derives Ord; store in a BTreeMap for free sorted iteration.

5. Interactive vs Non-Interactive Mode (permission/index.ts::interactive)
   Files: permission/index.ts::AskInput.interactive, neverAsk() pattern in question/index.ts
   Design: When interactive=false or neverAsk()=true, the tool fails immediately (DeniedError) or auto-answers instead of blocking. This lets headless agents proceed without a user; the agent re-evaluates to pick an option. For Rust, expose a config flag (e.g., MODE=interactive|headless) and at the gate check it before calling Permission.ask(); in headless mode, suggest a default or require the orchestrator to pre-populate answers.

6. Schema-Driven Question Format (question/index.ts::Info, Option, Prompt)
   Files: question/index.ts with zod serialization via schema.ts
   Design: Questions are strongly typed (Option has label + description; Info adds question, header, options array, flags for multi/custom, optional i18n key). Answers are arrays of strings (labels or custom text). This makes questions actionable by clients (web, CLI, agents) without string parsing. For Rust, define a struct hierarchy mirroring Option > Info > Request, derive Serialize/Deserialize, and emit JSON over IPC so the dashboard can render questions in a strongly-typed manner.

7. Plugin Tool Registration via Glob (registry.ts::fromPlugin(), custom tool load)
   Files: registry.ts::lines 181-199 (glob load), plugin.list()
   Design: Custom tools load from {tool,tools}/*.{js,ts} via a glob pattern in config dirs + plugins. Each module exports ToolDefinition objects; the registry wraps them with fromPlugin(id, def) to normalize Promise returns and handle truncation. This is zero-config: drop a file, restart, the tool is available. For Rust, scan a tools/ directory at startup, dlopen or use a proc macro to register each tool; emit a registry export so orchestrators know what's available.

8. Permission Request Bus (permission/index.ts::Event.Asked, Event.Replied)
   Files: permission/index.ts::lines 75-87 (bus events), ask() Deferred pattern
   Design: ask() emits permission.asked event on the bus and awaits a Deferred that is resolved when permission.replied arrives. Pending requests are tracked by PermissionID in a Map; cleanup happens on timeout or user reply. This decouples the permission checker from the UI (dashboard, CLI, Slack bot) that renders the ask. For Rust, use tokio::sync::Notify or a channel; store PendingEntry { request, response_tx } and resolve response_tx when reply arrives; log the pending ID so the user can grep logs for "permission_*" and see what was asked.
