Source: ouroboros runtime abstraction layer (github.com/Q00/ouroboros, MIT)
Reliability: MEDIUM (reviewed third-party MIT source, not official ELAI docs)
Fetched: 2026-05-16
Consumer: B-37 Phase 4M multi-runtime backend abstraction

Files examined:
  src/ouroboros/backends/__init__.py              (re-exports from capabilities.py)
  src/ouroboros/backends/capabilities.py          (BackendCapability registry + resolution fns)
  src/ouroboros/orchestrator/adapter.py           (AgentRuntime Protocol + ClaudeAgentAdapter)
  src/ouroboros/orchestrator/runtime_factory.py   (create_agent_runtime factory)
  src/ouroboros/orchestrator/codex_cli_runtime.py (CodexCliRuntime)
  src/ouroboros/orchestrator/opencode_runtime.py  (OpenCodeRuntime)
  src/ouroboros/orchestrator/hermes_runtime.py    (HermesCliRuntime)
  src/ouroboros/orchestrator/gemini_cli_runtime.py (GeminiCLIRuntime)
  src/ouroboros/orchestrator/kiro_adapter.py      (KiroAgentAdapter)
  src/ouroboros/orchestrator/copilot_cli_runtime.py (CopilotCliRuntime)

NOTE: goose/command_dispatcher also ingested; goose runtime not in 4M.0 canonical list but present
      in ouroboros registry. Phase 4M scope covers the 8 adapters listed above + goose as bonus.

Public API:

BACKEND REGISTRY (capabilities.py):

BackendCapability | frozen dataclass | fields: name, aliases, supports_runtime, supports_llm,
  supports_interview_driver, switchable_runtime, cli_name, cli_config_key,
  soft_tool_enforcement, supports_tool_envelope, skill_execution_capabilities

_CAPABILITIES tuple (9 entries):
  name="claude"    | aliases=("claude_code",)    | supports_runtime=True  | switchable=True  | soft_tools=False | tool_envelope=True  | cli="claude"   | config_key="cli_path"
  name="codex"     | aliases=("codex_cli",)      | supports_runtime=True  | switchable=True  | soft_tools=False | tool_envelope=True  | cli="codex"    | config_key="codex_cli_path"
  name="copilot"   | aliases=("copilot_cli",)    | supports_runtime=True  | switchable=False | soft_tools=False | tool_envelope=True  | cli="copilot"  | config_key="copilot_cli_path"
  name="gemini"    | aliases=("gemini_cli",)     | supports_runtime=True  | switchable=True  | soft_tools=True  | tool_envelope=True  | cli="gemini"   | config_key="gemini_cli_path"
  name="hermes"    | aliases=("hermes_cli",)     | supports_runtime=True  | switchable=True  | soft_tools=False | tool_envelope=FALSE | cli="hermes"   | config_key="hermes_cli_path"
  name="kiro"      | aliases=("kiro_cli",)       | supports_runtime=True  | switchable=False | soft_tools=False | tool_envelope=True  | cli="kiro"     | config_key="kiro_cli_path"
  name="opencode"  | aliases=("opencode_cli",)   | supports_runtime=True  | switchable=False | soft_tools=True  | tool_envelope=True  | cli="opencode" | config_key="opencode_cli_path"
  name="goose"     | aliases=("goose_cli",)      | supports_runtime=True  | switchable=True  | soft_tools=True  | tool_envelope=True  | cli="goose"    | config_key="goose_cli_path"
  name="litellm"   | aliases=("openai","openrouter") | supports_runtime=FALSE | supports_llm=True | switchable=False

KEY: hermes is the ONLY backend with supports_tool_envelope=False.
KEY: switchable_runtime=False means: copilot, kiro, opencode (cannot hot-switch at runtime).

resolve_runtime_backend_name(name: str) -> str
  Input: any string (canonical name or alias, case-insensitive after strip+lower)
  Output: canonical backend name (e.g. "claude_code" -> "claude")
  Error: ValueError("Unsupported backend for runtime: {candidate}")
         candidate is stripped+lowercased input; supported list NOT included in error message
  Resolution: strip().lower() -> get_backend_capability() -> check supports_runtime

runtime_backend_choices(include_aliases: bool = False) -> tuple[str, ...]
  Returns tuple of canonical names supporting runtime (8 entries: claude,codex,copilot,
  gemini,hermes,kiro,opencode,goose). With include_aliases=True returns names + all aliases.

get_backend_capability(name: str) -> BackendCapability | None
  Matches by name OR alias after strip().lower(). Returns None if no match.

resolve_backend_alias(name: str) -> str
  Returns canonical name; raises ValueError if not found at all (not just runtime-incapable).

AGENT RUNTIME PROTOCOL (adapter.py):

AgentRuntime | typing.Protocol
  Properties (read-only):
    runtime_backend: str           -- canonical id e.g. "claude", "codex_cli", "hermes_cli"
    capabilities: RuntimeCapabilities
    llm_backend: str | None
    working_directory: str | None
    permission_mode: str | None

  Methods:
    execute_task(prompt, tools=None, system_prompt=None, resume_handle=None,
                 resume_session_id=None) -> AsyncIterator[AgentMessage]
      NOTE: defined as regular def (not async def) returning AsyncIterator.
            Callers use: async for msg in runtime.execute_task(...):
    execute_task_to_result(...same args...) -> Result[TaskResult, ProviderError]
      NOTE: async def; collects full stream into TaskResult

RuntimeCapabilities | frozen dataclass:
  skill_dispatch: bool
  targeted_resume: bool
  structured_output: bool
  FULL_CAPABILITIES constant = RuntimeCapabilities(True, True, True)  [for claude]

RuntimeHandle | frozen dataclass (resume token):
  backend: str               -- canonical backend name
  kind: str = "agent_runtime"
  native_session_id: str | None
  conversation_id: str | None
  previous_response_id: str | None
  transcript_path: str | None
  cwd: str | None
  approval_mode: str | None
  updated_at: str | None
  metadata: dict[str, Any]   -- backend-specific extras (e.g. ouroboros_interview_session_id)

AgentMessage | frozen dataclass (streaming output unit):
  type: str          -- "text", "tool_use", "tool_result", "result", "error"
  content: str
  tool_name: str | None
  data: dict[str, Any]
  resume_handle: RuntimeHandle | None   -- present on final result message

TaskResult | frozen dataclass (collected output):
  success: bool
  final_message: str
  messages: tuple[AgentMessage, ...]
  session_id: str | None
  resume_handle: RuntimeHandle | None

FACTORY (runtime_factory.py):

create_agent_runtime(*, backend=None, permission_mode=None, model=None, cli_path=None,
                     cwd=None, llm_backend=None, startup_output_timeout_seconds=None,
                     stdout_idle_timeout_seconds=None) -> AgentRuntime
  All kwargs keyword-only. All optional (None falls back to ouroboros.config functions).
  Config fallback chain: explicit kwarg -> ouroboros.config.get_*() -> class default
  Config functions called: get_agent_runtime_backend(), get_agent_permission_mode(),
    get_cli_path(), get_codex_cli_path(), get_copilot_cli_path(), get_goose_cli_path(),
    get_hermes_cli_path(), get_kiro_cli_path(), get_gemini_cli_path(),
    get_opencode_cli_path(), get_llm_backend(), get_runtime_profile()

ELAI NOTE: ELAI has no ouroboros.config; replace config fallback chain with
  RuntimeFactoryConfig struct fields + env var overrides in ELAI's factory fn.

PER-ADAPTER INIT PARAMETERS:

ClaudeAgentAdapter:
  api_key: str | None = None
  permission_mode: str = "acceptEdits"    -- "acceptEdits" | "bypassPermissions" | "default"
  model: str | None = None
  cwd: str | Path | None = None
  cli_path: str | Path | None = None
  runtime_backend returns: "claude"
  subprocess: NO -- delegates to claude_agent_sdk (lazy import). SDK manages CLI subprocess.
  capabilities: FULL_CAPABILITIES (skill_dispatch=True, targeted_resume=True, structured_output=True)

CodexCliRuntime:
  cli_path: str | Path | None = None
  permission_mode: str | None = None       -- resolved via resolve_codex_permission_mode(); default "acceptEdits"
  model: str | None = None
  cwd: str | Path | None = None
  skills_dir: str | Path | None = None
  skill_dispatcher: SkillDispatchHandler | None = None
  llm_backend: str | None = None
  runtime_profile: str | None = None
  runtime_backend returns: "codex_cli"
  subprocess spawn: [cli_path, "exec", "--json", "--skip-git-repo-check",
                     "--output-last-message", <temp_path>, "-C", cwd, ...]
                     + optional "resume" subcommand
  stdin: composed prompt as UTF-8 text (when _feeds_prompt_via_stdin() True)
  stdout: JSONL events; type field: "thread.started" | "item.completed" |
          "turn.failed" | "turn.completed"
  final_message: read from --output-last-message temp file after process exits

OpenCodeRuntime:
  cli_path: str | Path | None = None
  permission_mode: str | None = None
  model: str | None = None
  cwd: str | Path | None = None
  skills_dir: str | Path | None = None
  skill_dispatcher: SkillDispatchHandler | None = None
  llm_backend: str | None = None
  opencode_mode: str | None = None         -- controls MCP tool set via get_ouroboros_tools()
  runtime_backend returns: "opencode"
  subprocess spawn: [cli_path, "run", "--pure", "--format", "json"]
                    + optional ["--model", model] + ["--session", session_id]
  --pure flag: MANDATORY -- disables external plugins preventing double-dispatch
  stdin: composed prompt piped via subprocess.PIPE (avoids ~128KB OS arg limit)
  stdout: JSON event stream, one dict per line
  stderr: fallback error output when stdout lacks final message
  event normalizer: OpenCodeEventNormalizer

HermesCliRuntime:
  cli_path: str | Path | None = None       (keyword-only args, all)
  permission_mode: str | None = None
  model: str | None = None
  cwd: str | Path | None = None
  skills_dir: str | Path | None = None
  skill_dispatcher: SkillDispatchHandler | None = None
  llm_backend: str | None = None
  startup_output_timeout_seconds: float | None = None  -- default 60s; env var _STARTUP_TIMEOUT_ENV
  stdout_idle_timeout_seconds: float | None = None     -- default 300s; env var _IDLE_TIMEOUT_ENV
  runtime_backend returns: "hermes_cli"
  subprocess spawn: [cli_path, "chat", "--resume", session_id, "-Q", "--source", "tool",
                     "--model", model_name, "-q", prompt]
  -Q: quiet mode for programmatic output; -q: pass the composed prompt
  stdin: NOT USED (prompt passed via -q flag)
  stdout: RAW TEXT (NOT JSONL) -- line-by-line via asyncio.subprocess.PIPE
    _parse_quiet_output(): extracts content + session_id
    _strip_reasoning_prelude(): strips box-drawing char decorations
  supports_tool_envelope=False: CRITICAL -- hermes cannot receive tool envelopes
  timeout guards: non-positive value disables guard

GeminiCLIRuntime:
  cli_path: str | Path | None = None
  permission_mode: str | None = None
  model: str | None = None
  cwd: str | Path | None = None
  skills_dir: str | Path | None = None
  skill_dispatcher: SkillDispatchHandler | None = None
  llm_backend: str | None = None
  runtime_backend returns: "gemini"
  subprocess spawn: gemini --prompt <prompt> --non-interactive
                    --output-format stream-json
                    --approval-mode {auto_edit|yolo}
                    [--model <model>]
  stdin: NOT USED (_requires_process_stdin() returns False)
  stdout: NDJSON via --output-format stream-json
  event types: "init" | "message" | "text" | "thinking" | "tool_use" |
               "tool_result" | "error" | "result"
  soft_tool_enforcement=True (per registry)

KiroAgentAdapter:
  cli_path: str | Path | None = None      (all keyword-only)
  model: str | None = None
  cwd: str | Path | None = None           -- default: os.getcwd()
  permission_mode: str | None = None      -- default: "acceptEdits"
  skill_dispatcher: SkillDispatchHandler | None = None
  llm_backend: str | None = None          -- default: "kiro"
  skills_dir: str | Path | None = None
  **_kwargs: object                       -- extra kwargs silently discarded (forward-compat)
  runtime_backend returns: "kiro"
  subprocess spawn: kiro-cli chat --no-interactive
                    [--trust-tools=<cats>|--trust-all-tools]
                    [--model <name>] [--resume-id <id>] "<prompt>"
  stdin: NOT USED (prompt as positional arg)
  stdout: RAW TEXT with terminal color escapes stripped via regex
    streamed line-by-line until EOF as "assistant" type messages
    stderr drained concurrently (prevents deadlock)
  timeouts: 60s startup, 300s idle (adaptive)
  capabilities: targeted_resume=False, structured_output=False
  switchable_runtime=False (per registry)

CopilotCliRuntime:
  cli_path: str | Path | None = None
  permission_mode: str | None = None
  model: str | None = None
  cwd: str | Path | None = None
  skills_dir: str | Path | None = None
  skill_dispatcher: SkillDispatchHandler | None = None
  llm_backend: str | None = None
  runtime_profile: str | None = None      -- passed to resolve_copilot_agent(); selects agent vs model mode
  runtime_backend returns: "copilot"
  subprocess spawn: [cli_path, "--no-color", "--log-level", "none", "--add-dir", cwd]
                    + permission flags from _build_permission_args()
                    + ["--agent", agent_name] OR ["--model", mapped_model]  (mutually exclusive)
                    + ["-p", prompt]
  stdin: NOT REQUIRED
  stdout: JSONL event stream via _convert_event()
  runtime_profile determines: whether --agent or --model flag is used (resolve_copilot_agent())
  execute_task: inherited from CodexCliRuntime (no override)
  switchable_runtime=False (per registry)

Behavioral contracts:

resolve_runtime_backend_name -> strips+lowercases input, resolves aliases, validates supports_runtime
  Side effect: none. Pure function. Raises ValueError on miss.
  "claude_code" -> "claude" (alias resolution)
  "codex_cli" -> "codex" (alias resolution)
  Error msg format: "Unsupported backend for runtime: {lowercased_candidate}"
  NOTE: error message does NOT include the supported list. Consumer must call
        runtime_backend_choices() separately to build the error message for users.

create_agent_runtime -> reads config, instantiates adapter, returns AgentRuntime
  Side effects: reads environment variables via ouroboros.config (in ELAI: reads RuntimeFactoryConfig)
  Does NOT spawn subprocess at construction time -- subprocess spawned lazily in execute_task()

execute_task() -> spawns subprocess on each call (no persistent connection)
  Exception: ClaudeAgentAdapter delegates to claude_agent_sdk which manages its own lifecycle
  All CLI runtimes: subprocess spawned fresh per execute_task() call
  Streaming: yields AgentMessage objects as subprocess emits output
  Cancellation: runtime terminates subprocess on asyncio.CancelledError

RuntimeHandle -> carries resume state across calls
  backend field MUST match runtime_backend of the adapter being called
  metadata dict carries backend-specific extras (e.g. ouroboros_interview_session_id for codex)

Platform constraints:

All CLI runtimes use asyncio.create_subprocess_exec -- requires asyncio event loop
Subprocess binary must be on PATH or cli_path must be provided
kiro-cli (note hyphen) is the binary name, not kiro (unlike other backends where cli_name matches binary)
Windows: shutil.which() used for binary discovery; no special Windows handling observed
Python version: dataclass slots=True requires Python 3.10+
Timeout env vars (_STARTUP_TIMEOUT_ENV, _IDLE_TIMEOUT_ENV) only apply to Hermes and Goose adapters

Gotchas:

hermes supports_tool_envelope=False: sending tool envelopes to hermes will silently produce
  wrong output -- the tool call JSON will be treated as plain text. Must detect at dispatch time.

opencode --pure flag: MUST be included. Omitting it causes double-dispatch when MCP subagent
  envelopes are in the prompt -- the plugin layer re-processes them.

codex final_message is in a temp file, not stdout: reading stdout as the final answer is wrong.
  The --output-last-message flag path must be read after process exits.

kiro-cli binary name has a hyphen: binary is "kiro-cli" NOT "kiro" (unlike cli_name="kiro" in registry
  which is the backend canonical name, not the binary name).

CopilotCliRuntime inherits execute_task from CodexCliRuntime: if CodexCliRuntime's execute_task
  is overridden or its JSONL parsing changes, CopilotCliRuntime is affected silently.

KiroAgentAdapter **_kwargs: unknown constructor kwargs are silently discarded. No error on
  unexpected params. This is intentional forward-compat but can mask misconfiguration.

create_agent_runtime does NOT validate cli_path existence at construction time.
  Missing binary only fails at execute_task() call time (subprocess spawn failure).

switchable_runtime=False (copilot, kiro, opencode): means these runtimes cannot be
  hot-switched during a running session in ouroboros's sense. ELAI implication: do not
  attempt to swap these mid-session via /mode command.

soft_tool_enforcement=True (gemini, opencode, goose): these backends require text-based
  tool enforcement guidance in the prompt rather than structured tool envelopes.

litellm has supports_runtime=False: passing "litellm" to resolve_runtime_backend_name raises
  ValueError even though litellm IS in the registry (it only supports LLM, not runtime).

Alias canonicalization is one-way: resolve_runtime_backend_name("claude_code") -> "claude"
  but runtime_backend property on ClaudeAgentAdapter returns "claude" (canonical).
  RuntimeHandle.backend must store the canonical name, not the alias.

Critical assertions (testable):

assert resolve_runtime_backend_name("claude_code") == "claude"
assert resolve_runtime_backend_name("codex_cli") == "codex"
assert resolve_runtime_backend_name("hermes_cli") == "hermes"
assert resolve_runtime_backend_name("kiro_cli") == "kiro"
assert resolve_runtime_backend_name("copilot_cli") == "copilot"
assert resolve_runtime_backend_name("gemini_cli") == "gemini"
assert resolve_runtime_backend_name("opencode_cli") == "opencode"
assert resolve_runtime_backend_name("goose_cli") == "goose"

try:
    resolve_runtime_backend_name("litellm")
    assert False, "litellm is not a runtime backend"
except ValueError as e:
    assert "runtime" in str(e)

try:
    resolve_runtime_backend_name("nonexistent")
    assert False, "unknown backend must raise"
except ValueError as e:
    assert "nonexistent" in str(e)

assert len(runtime_backend_choices()) == 8  # claude,codex,copilot,gemini,hermes,kiro,opencode,goose
assert "litellm" not in runtime_backend_choices()

# hermes uniquely has supports_tool_envelope=False
from ouroboros.backends.capabilities import get_backend_capability
hermes_cap = get_backend_capability("hermes")
assert hermes_cap.supports_tool_envelope == False
all_others = [c for c in runtime_backend_choices() if c != "hermes"]
for name in all_others:
    cap = get_backend_capability(name)
    assert cap.supports_tool_envelope == True, f"{name} should support tool envelope"

# RuntimeCapabilities for non-kiro adapters should have structured_output=True
# KiroAgentAdapter: targeted_resume=False, structured_output=False
# Hermes: raw text output means structured_output=False
# Claude: FULL_CAPABILITIES = all True

ELAI port notes (for Rust implementation in B-37 Phase 4M):

1. RuntimeBackend enum: 11 variants per 4M.0 spec. ouroboros has 8 runtime-capable backends
   (claude/codex/copilot/gemini/hermes/kiro/opencode/goose). ELAI adds Sidecar + AnthropicApi
   (existing) + Ollama + Vllm (new HTTP backends). Total = 11 + goose as potential 12th.

2. resolve_backend(candidate: &str) -> Result<RuntimeBackend, ResolveBackendError>:
   Rust port of resolve_runtime_backend_name. Must handle aliases (snake_case variants).
   ResolveBackendError must include supported_list: Vec<String> (ouroboros omits this from
   error message -- ELAI spec explicitly requires it per TYPES section).

3. RuntimeAdapter trait vtable (Rust) maps to AgentRuntime Protocol (Python):
   execute_task -> async fn complete(messages, config) -> Result<CompletionResponse>
   execute_task streaming -> async fn stream(..) -> Result<BoxStream<StreamChunk>>
   capabilities -> fn capabilities() -> RuntimeCapabilities
   health_check -> async fn health_check() -> RuntimeHealth  (ELAI addition, not in ouroboros)

4. RuntimeFactoryConfig (Rust struct) replaces ouroboros.config fallback chain:
   Fields: backend, cli_path, base_url, api_key, model, permission_mode,
           runtime_profile, cwd, startup_output_timeout_seconds, stdout_idle_timeout_seconds
   No implicit env var reads in factory -- all config explicit via struct.

5. Per-adapter subprocess management (Rust via tokio::process::Command):
   claude: tokio::process not needed directly (uses claude_agent_sdk via FFI or REST)
   codex: Command with args ["exec","--json","--skip-git-repo-check",
          "--output-last-message",<path>,"-C",<cwd>]; stdin=Piped; stdout=Piped
   opencode: Command with args ["run","--pure","--format","json"]; stdin=Piped; stdout=Piped
   hermes: Command with args ["chat","--resume",id,"-Q","--source","tool",
           "--model",m,"-q",prompt]; stdin=null; stdout=Piped
   gemini: Command with args ["--prompt",p,"--non-interactive",
           "--output-format","stream-json","--approval-mode","auto_edit"]; stdin=null; stdout=Piped
   kiro: Command with binary "kiro-cli" (NOT "kiro"); args ["chat","--no-interactive",
         trust_flag, prompt]; stdin=null; stdout=Piped; stderr=Piped (drain concurrently)
   copilot: Command with args ["--no-color","--log-level","none","--add-dir",cwd,
            perm_flags, model_or_agent_flag, "-p", prompt]; stdin=null; stdout=Piped

6. Stdout parsing per backend (Rust):
   codex: serde_json::from_str per line; event.type in {"thread.started","item.completed",
          "turn.failed","turn.completed"}; final answer from --output-last-message file
   opencode: serde_json::from_str per line; OpenCodeEventNormalizer logic
   hermes: raw line-by-line String; strip box-drawing chars; no JSON
   gemini: serde_json::from_str per line (NDJSON); event.type in {"init","message","text",
           "thinking","tool_use","tool_result","error","result"}
   kiro: raw line-by-line String; strip ANSI escape codes via regex
   copilot: serde_json::from_str per line (JSONL)

7. Timeout handling (Rust tokio):
   startup_timeout: tokio::time::timeout around first line read from stdout
   idle_timeout: tokio::time::timeout on each subsequent line read
   On timeout: send SIGTERM (Unix) / TerminateProcess (Windows), await child, return error
   hermes+goose: explicit timeout params. Other adapters: consider fixed defaults or config.

8. Message normalization: all adapters MUST yield equivalent AgentMessage-equivalent structs
   so the orchestrator sees a uniform stream regardless of backend output format.
   The type field semantics MUST be normalized: "text", "tool_use", "tool_result",
   "result", "error" -- backend-specific event types map into these 5.
