Source: omnigent (github.com/omnigent-ai/omnigent)
Reliability: HIGH (reviewed source, Apache 2.0 open, Databricks-backed, actively maintained)
Last commit: 2026-06-25 (today; very fresh, same-day HEAD)
Python: 3.12+ required. uv-managed. Version: 0.3.0.dev0 (pre-release alpha).
Files examined:
  omnigent/inner/executor.py
  omnigent/inner/policies.py
  omnigent/inner/sandbox.py
  omnigent/inner/bwrap_sandbox.py
  omnigent/inner/seatbelt_sandbox.py
  omnigent/inner/windows_jobobject_sandbox.py
  omnigent/inner/_seccomp.py
  omnigent/inner/egress/proxy.py
  omnigent/inner/egress/rules.py
  omnigent/inner/credential_proxy.py
  omnigent/inner/loader.py
  omnigent/inner/nessie/policies.py
  omnigent/native_server_harness.py
  omnigent/native_server_transport.py
  omnigent/native_policy_hook.py
  omnigent/native_coding_agents.py
  omnigent/session_lifecycle.py
  omnigent/cost_plan.py
  omnigent/reasoning_effort.py
  omnigent/cursor_native_bridge.py
  omnigent/cursor_native_forwarder.py
  omnigent/cursor_native.py
  omnigent/policies/builtins/cel.py
  docs/AGENT_YAML_SPEC.md
  designs/SANDBOX_CREDENTIAL_PROXY.md
  examples/polly/config.yaml
  pyproject.toml
  tests/ (1009 test files present)


What it is:
  A Python meta-harness / orchestration layer over multiple coding agent CLIs
  (Claude Code, Codex, Cursor, Kiro, Goose, Hermes, Kimi, Pi, OpenCode,
  Qwen, Antigravity/Gemini). One server + web UI surfaces all of them under a
  single session abstraction with shared policies, sandboxing, and
  cross-device access. Not an autonomous agent itself: it wraps vendor CLIs
  and provides the governance/routing/session layer on top.


Public API:
  Executor (abstract base) | omnigent/inner/executor.py | AsyncIterator[ExecutorEvent] | run_turn, interrupt_session, enqueue_session_message, supports_streaming, handles_tools_internally, supports_live_message_queue
  ExecutorConfig | dataclass | model: str|None, temperature: float=0.0, max_tokens: int=100000, extra: dict
  ExecutorEvents: TextChunk | ReasoningChunk | ToolCallRequest | TurnComplete | ToolCallComplete | CompactionComplete | TurnCancelled | ExecutorError
  NativeServerHarness(Executor) | omnigent/native_server_harness.py | thin executor for server-owning harnesses (opencode, codex-native); inject lock on asyncio.Lock per instance
  NativeServerTransport (Protocol) | omnigent/native_server_transport.py | send_prompt, abort, stream methods; NativeLaunchConfig + NativeServerHandle + NativePrompt dataclasses
  NativeCodingAgent (frozen dataclass) | omnigent/native_coding_agents.py | key, display_name, harness, wrapper_label, subagent_wrapper_label; registry of 11 agents: claude, codex, pi, opencode, cursor, kiro, goose, antigravity, hermes, kimi, qwen
  Policy (base) | omnigent/inner/policies.py | async evaluate(content, phase, context) -> PolicyResult; phases: request/response/tool_call/tool_result; actions: ALLOW/ASK/DENY
  FunctionPolicy | accepts Python callable (sync or async, with or without config arg); coerces dict returns to PolicyResult
  PromptPolicy | delegates evaluation to a sub-executor LLM call; returns JSON {action, reason, set_labels}
  CelPolicy (builtin) | omnigent/policies/builtins/cel.py | cel_policy(expression, reason) -> PolicyCallable; uses cel-expr-python (optional dep, degrades gracefully on missing)
  AdvisorVerdict | omnigent/cost_plan.py | tier: cheap/medium/expensive + concrete_model; serialized to conversation label cost_control.plan (JSON, version=3); per-turn brain-model selection
  SandboxPolicy (dataclass) | backend_type, active, read_roots, write_roots, write_files, allow_network, cwd_allow_hidden, cwd_hidden_scan_max_entries
  load_agent_def | omnigent/inner/loader.py | Path|dict -> AgentDef; YAML with custom bool resolver (fixes on/off YAML 1.1 false-positive)


Behavioral contracts:
  NativeServerHarness.run_turn -> polls for native session_id up to boot_poll_attempts*boot_poll_delay; holds asyncio.Lock(_inject_lock) across run_turn + enqueue_session_message; returns TurnComplete immediately (streaming is forwarder's job, not executor's)
  NativeServerHarness.handles_tools_internally -> always True (vendor CLI runs its own tools; Omnigent never intercepts tool execution for native harnesses)
  native_policy_hook -> translates PreToolUse/PostToolUse/UserPromptSubmit hook payloads into POST /v1/sessions/{id}/policies/evaluate with 30s retry on 5xx; fails CLOSED on unreachable server (returns DENY with _EVAL_UNAVAILABLE_REASON); skips mcp__omnigent__* tools (already checked by relay path) but passes connector-native MCP tools (mcp__github__* etc.)
  cursor_native_forwarder -> polls Cursor's content-addressed SQLite store (~/.cursor/chats/<md5(cwd)>/<chat-id>/store.db) at 0.7s interval; POSTs mirrored messages as external_conversation_item events; does NOT post running/idle status (runner's PTY-activity watcher owns those); advances high-water rowid only after successful POST (prevents lost messages on 500)
  fork_preamble mechanism (cursor_native_bridge) -> on conversation fork, prior transcript serialized to XML-tagged block <omnigent_fork_history>...</omnigent_fork_history> and prepended to first injected user message; fork sentinel tags in user text are neutralized before wrapping; preamble file cleared only AFTER successful injection (survive injection failure)
  Antigravity audit (antigravity_native_audit.py) -> POST-HOC ONLY; agy 1.0.8 PreToolUse hook does NOT fire before tool execution; audit observes after DONE step; fail-open on timeout (tool already ran); explicitly documented as degrade-notice surfaced to user
  AdvisorVerdict v3 -> one brain model per turn (max-tier of all parts); verdict serialized into conversation label; advisor modes: optimize (applies verdict) or advise (shadow, records but does not apply)
  Session closed state -> dual-tracked: new sessions use omnigent.closed=true label; legacy sessions use title suffix :closed:conv_id; labels_with_closed_status synthesizes both for API consumers
  CEL policies -> non-Turing-complete, side-effect-free, guaranteed to terminate; cel_policy factory compiles expression at registration time, not per-evaluation; degrades to ImportError if cel-expr-python absent


Platform constraints:
  Linux: bwrap (bubblewrap) sandbox: mount namespace isolation + seccomp denylist (k8s RuntimeDefault baseline) + arg-filtered clone(CLONE_NEW*) + socket-family allowlist (AF_UNIX/INET/INET6 only); clone3 -> ENOSYS not EPERM (glibc pthread_create compat on Ubuntu 22.04+)
  macOS: sandbox-exec (Seatbelt/SBPL) sandbox; profile written to mode-0600 tempfile (not -p inline, to hide from ps aux); does NOT grant mach-priv-host-port or iokit-open
  Windows: Job Object sandbox ONLY; no filesystem or network isolation; read_paths/write_paths are advisory (not enforced); JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE guarantees process-tree termination on Omnigent crash
  Seccomp multi-arch: filters registered for native + compat ABIs (i386+x32 on x86_64, 32-bit ARM on aarch64) to block int $0x80 bypass
  DNS-safe host allowlist in egress: [A-Za-z0-9.-]+ only; rejects NUL (libc truncation bypass), %, \r\n (CRLF injection), brackets, colons, @, whitespace
  Egress proxy: async MITM HTTPS via per-host certs signed by in-process CA; listens on Unix socket (hard enforcement via netns) and/or TCP; credential swap-on-access (real secret never enters sandbox)
  Credential proxy: oa_cred_ synthetic placeholder injected into sandbox env for clients that require a local credential before issuing a request (e.g. gh); proxy rejects placeholder presented to wrong host with HTTP 403


Gotchas:
  antigravity_native_audit.py is explicitly marked DORMANT (no consumer since Task 12 cutover); module retained for future use but not exercised in production
  cursor_native_forwarder response_id is truncated to 64 chars: Cursor blob id is 64-char hash, so cursor:<blob_id> = 71 chars and overflows VARCHAR(64) in db; uncapped POSTs 500 and high-water mark never advances (forwarder stalls permanently)
  CEL optional dep: platform_machine != "aarch64" and not (darwin + x86_64); on those platforms cel_policy() raises ImportError at registration time, not at evaluation time
  Windows sandbox: no seccomp, no mount namespace, no network isolation; production use on Windows has materially weaker containment than Linux/macOS
  NativeServerHarness.supports_streaming() always False; harness does NOT stream — do not expect streamed TextChunk events from native harnesses; the forwarder emits output via a separate process
  Policy evaluation for native sessions: UserPromptSubmit hook is the SOLE REQUEST gate for native sessions (server-level _evaluate_input_policy is bypassed for native message events); if the hook is missing, no input policy fires
  Harness boot poll is configurable but default 60 polls * 1.0s = 60s max before giving up on session_id resolution


Critical assertions (testable):
  assert sum(1 for a in [action] if a in ("allow", "ask", "deny")) == 1, "policy action must be one of allow/ask/deny"
  assert not any(h.startswith("mcp__omnigent__") for h in evaluated_tools), "omnigent MCP tools must be skipped in native policy hook"
  assert fork_preamble_file_deleted_only_after_successful_injection, "fork preamble must survive injection failure"
  assert egress_rule_host.match(r"[A-Za-z0-9.\-]+"), "egress host must pass DNS-safe regex before rule match"
  assert windows_sandbox_fs_isolation == False, "Windows Job Object provides no filesystem isolation"


Complexity:
  Policy evaluate: O(N policies) per event, each potentially one async LLM call (PromptPolicy); FunctionPolicy is sync-or-async user callable
  Sandbox setup: O(cwd_entries) for dotfile scan (bwrap); bounded by cwd_hidden_scan_max_entries (configurable cap)
  Egress proxy: async MITM with per-host TLS cert cache; O(1) per request after cert cache warm
  Fork preamble: O(transcript_length) to serialize prior conversation; injected as single tmux bracketed-paste


Tests present:
  1009 test files across: cli/ (server lifecycle, backend, chat, auth, update-check), db/ (migrations x10+, SQLite safe, FTS dialect), policies/ (CEL, builtins), codex_parity/, client_tools/, e2e/ (cancel, journey), integration/, server/ (accounts, OIDC, routes). Coverage file (.coverage) present in repo root (pytest-cov used). Codex parity tests in tests/codex_parity/ compare executor behavior across harnesses.


License: Apache 2.0 (Databricks, Inc.)


Staleness verdict:
  NOT STALE. Last commit 2026-06-25 (today). Active development, alpha status.
  Supports current model generation: Anthropic Claude (claude-sdk + claude_native), OpenAI Agents SDK, Codex CLI, Cursor, Kiro, Goose, Gemini/Antigravity, Qwen, Kimi, Pi, OpenCode.
  The AGENT_YAML_SPEC references claude-opus-4-7, claude-opus-4-8, claude-sonnet-4-6 by name (post-Claude-3 generation).
  WebSearch not needed: freshness confirmed from HEAD commit timestamp.


ELAI steal evaluation (mechanisms vs ELAI stack):

1. MULTI-PLATFORM SANDBOX WITH L7 EGRESS MITM PROXY (HIGH VALUE)
   Location: omnigent/inner/bwrap_sandbox.py, seatbelt_sandbox.py, windows_jobobject_sandbox.py, egress/proxy.py, egress/rules.py, credential_proxy.py
   What it is: Three platform backends (bwrap/seatbelt/job-object) sharing one SandboxPolicy dataclass. The egress layer is a full async MITM HTTPS proxy with per-host TLS certs, a CEL-inspectable rule DSL (GET api.github.com/**), and a secretless credential swap (oa_cred_ placeholder -> real token injected in-proxy, never entering sandbox). Seccomp denylist is the k8s RuntimeDefault baseline + arg-filtered clone(CLONE_NEW*) + AF_ socket allowlist + multi-arch compat coverage.
   ELAI relevance: ELAI has Tier-C OS-process sandbox (crates/orchestrator/src/sandbox/). This is a SIGNIFICANTLY more mature reference implementation on all three axes: mount-namespace isolation (ELAI is process-level only per sandbox/mod.rs), network egress with credential proxy (ELAI has no equivalent), and the multi-arch seccomp coverage. The credential proxy design (swap-on-access, oa_cred_ placeholder for gh-class clients, per-host binding, 403 on cross-host replay) is directly portable to Rust using the same conceptual design. HIGH steal candidate.

2. FAIL-CLOSED POLICY HOOK WITH CEL EXPRESSIONS (MEDIUM VALUE)
   Location: omnigent/native_policy_hook.py, omnigent/policies/builtins/cel.py, omnigent/inner/policies.py
   What it is: PreToolUse/PostToolUse/UserPromptSubmit hooks translated to a typed EvaluationRequest POSTed to /v1/sessions/{id}/policies/evaluate; 30s retry with exponential backoff on 5xx; fails CLOSED to DENY on unreachable server. CEL policies use cel-expr-python (non-Turing-complete, guaranteed-terminate) for declarative tool-blocking without arbitrary code execution risk. FunctionPolicy + PromptPolicy (LLM-evaluated) as additional tiers.
   ELAI relevance: ELAI has off-agent wiring/trust gates but no user-facing declarative policy language. The CEL layer (deny if event.type == "tool_call" && event.data.name == "Bash") is clean and would compose well with ELAI's autonomous gate. The fail-closed retry pattern for an unreachable policy server is a direct steal for ELAI's autonomous_gate.rs. MEDIUM-HIGH.

3. CONVERSATION FORKING WITH HISTORY INJECTION (MEDIUM VALUE)
   Location: omnigent/cursor_native_bridge.py (wrap_fork_preamble, write_fork_preamble, clear_fork_preamble, FORK_HISTORY_OPEN_TAG/CLOSE_TAG, _neutralize_fork_sentinels)
   What it is: When a conversation is forked, the prior transcript is serialized as an XML-tagged block prepended to the first injected user message. Sentinel tags in replayed text are neutralized. Preamble file is cleared only after SUCCESSFUL injection (survive failure). The fork preamble is also compatible with cursor-agent --resume so native agent state is preserved.
   ELAI relevance: ELAI has SessionDag (redo=forward-edge, undo-then-edit=fork) from H-49. The sentinel-tag injection pattern and fail-safe preamble lifecycle (read-then-clear-on-success) are battle-tested patterns for the conversation forking ELAI is building. LOW-MEDIUM (ELAI's fork is at the session DAG level, Omnigent's is at the tmux-injection level, so the exact mechanism does not port, but the protocol design does).

4. PER-TURN COST-TIERED BRAIN-MODEL SELECTION (MEDIUM VALUE)
   Location: omnigent/cost_plan.py
   What it is: A per-turn LLM judge selects ONE brain model (cheap/medium/expensive tier) sized to the turn's difficulty; verdict serialized as a conversation label (cost_control.plan, JSON v3). Two modes: optimize (applies verdict to brain) and advise (shadow mode, records but does not change routing). Retired per-worker tier partition of v2.
   ELAI relevance: ELAI has multi-runtime backends + E-31 tier matrix + RuntimeRoutingPolicy. Omnigent's v3 single-verdict design (max-tier wins, one model for the orchestrator brain, workers chosen freely) is a simpler contract than ELAI's per-worker routing. The shadow/advise mode pattern is directly relevant to ELAI's ELAI_ROUTE_DECISION_SSB_ENABLE wire-dark gate. LOW-MEDIUM.

5. SQLITE-POLLED CONVERSATION MIRRORING FROM VENDOR NATIVE STATE (LOW-MEDIUM VALUE)
   Location: omnigent/cursor_native_forwarder.py, claude_native_forwarder.py, codex_native_forwarder.py
   What it is: Each vendor CLI stores its conversation in a proprietary format (Cursor: content-addressed SQLite rowid store; Claude Code: JSONL transcript; Codex: app-server WebSocket). Omnigent has a per-vendor forwarder that polls or subscribes and mirrors to the Omnigent session as external_conversation_item events. High-water mark advances only on successful POST (idempotent).
   ELAI relevance: ELAI has vendor-CLI bridges (claude_code/codex/copilot/gemini/kiro/opencode). The high-water-mark-on-success pattern and the per-vendor format knowledge are useful reference for ELAI's bridge state machines. The Cursor SQLite response_id truncation bug (>64 chars causes infinite stall) is a useful known-gotcha. LOW - ELAI's bridges are already in place and use a different event model (SSB rows vs HTTP POST).

6. NESSIE BLAST-RADIUS POLICIES FOR MULTI-AGENT ORCHESTRATION (LOW VALUE)
   Location: omnigent/inner/nessie/policies.py, examples/polly/config.yaml
   What it is: FunctionPolicy factories that enforce at tool dispatch: catastrophic-command deny, path-outside-cwd deny, git-push gate-to-ASK, file-count limits, regex-matched command blocks. Polly (examples/polly) is a pure-orchestrator YAML agent that delegates ALL coding to claude_code/codex/pi sub-agents and never writes code itself; a useful reference for ELAI's orchestrator persona.
   ELAI relevance: ELAI already has its own autonomous_gate.rs and off-agent wiring gates. Nessie's patterns are simpler than ELAI's. The pure-orchestrator YAML (never writes code, always delegates) is a useful prompt-pattern confirmation but not a mechanism to steal. LOW.
