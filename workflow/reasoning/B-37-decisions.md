B-37 Phase 4M Multi-Runtime Backend Abstraction -- Decision Record
Session: s-18544
Date: 2026-05-16

Scope policy
  Decomposed into 4 sub-cycles (A/B/C/D) so each sub-cycle has bounded
  blast radius and can audit/memorize cleanly. Commit only after D
  (final wiring validator passes) and B-38.

Sub-cycle A (this round): FOUNDATION
  - paper-ingestor output already at workflow/papers/ouroboros-runtime-interface.md
  - RuntimeBackend enum (11 variants) + RuntimeAdapter trait (NEW, not alias)
  - Types: PermissionMode, RuntimeCapabilities, RuntimeHealth,
    RuntimeFactoryConfig, RuntimeError, ResolveBackendError, StreamChunk,
    SecretString, Message, CompletionConfig, CompletionResponse
  - resolve_backend() with full supported-list ResolveBackendError on miss
  - create_runtime() factory returning Result<Box<dyn RuntimeAdapter>, RuntimeError>
  - Port Sidecar + AnthropicApi as thin wrappers over existing
    SidecarProvider + AnthropicClient
  - Stub adapters for the other 9 variants returning RuntimeError::NotImplemented
  - Re-exports through providers/src/lib.rs

Sub-cycle B: CLI adapters (claude_code, codex_cli, opencode, hermes,
  gemini_cli, kiro, copilot_cli) inside Tier C isolation + stdout
  sanitization + permission_mode + startup/idle timeouts.
Sub-cycle C: Native HTTP adapters (ollama, vllm) ported from 3J +
  PerRoleRuntime config + runtime fallback chain + 4 new SSB action types.
Sub-cycle D: /runtime CLI command + Dashboard RuntimePanel +
  3 Tauri commands + migration guide + metrics file +
  [cloud] default_mode -> [runtime] backend deprecation aliases.

Key decisions (grounded from spec-grounder report B-37A-grounding.md)
  1. RuntimeAdapter is a NEW trait, NOT an alias of LlmProvider.
     Why: providers/src/providers/ollama.rs:352 LlmProvider::complete
     takes (prompt: &str, config: &OllamaConfig) -> LlmFuture<Vec<String>>
     while spec phase-4M.txt:74-93 requires
     async fn complete(messages: Vec<Message>, config: CompletionConfig)
     -> Result<CompletionResponse, RuntimeError>. Signatures incompatible.
     Spec phase-4M.txt:95-97 endorses the wrapper path explicitly.
     SidecarRuntime and AnthropicApiRuntime are thin newtype wrappers
     around the existing SidecarProvider + AnthropicClient.

  2. create_runtime returns Result<Box<dyn RuntimeAdapter>, RuntimeError>.
     Why: phase-4M.txt:179 is internally inconsistent (says
     Box<dyn RuntimeAdapter> but description requires returning Err).
     Result lets stubs return RuntimeError::NotImplemented cleanly
     without panic. Matches ouroboros's "errors deferred to execute_task"
     pattern while making the missing-binary case discoverable up front.

  3. SecretString is an in-crate newtype (no new external dep).
     Why: secrecy + zeroize not present anywhere in workspace
     (grep .elai_cc/Cargo.toml + all crate Cargo.toml). CLAUDE.md
     "No new dependencies without confirmation." Manual newtype with
     Drop-zeroization sufficient for our threat model (avoid logging,
     not defeat memory forensics). Lives in providers/src/runtime/secret.rs.

  4. BoxStream uses futures-util::stream::BoxStream (version 0.3).
     Why: futures-util 0.3 already present in workspace
     (.elai_cc/crates/web_search/Cargo.toml:10). Adding to providers
     Cargo.toml is a grounded transitive promotion, not a new
     dependency. StreamChunk defined locally in
     providers/src/runtime/types.rs.

  5. PermissionMode has 4 variants: AcceptEdits (default), Plan,
     DangerouslySkipPermissions, Default.
     Why: phase-4M.txt:112-117 specifies 4 variants. Ouroboros maps
     "acceptEdits"/"bypassPermissions"/"default" -> first three.
     Plan is ELAI-specific addition for the future plan-only mode.
     Default behavior MUST be AcceptEdits, NEVER
     DangerouslySkipPermissions without explicit user opt-in.

  6. ResolveBackendError carries a `supported: Vec<String>` so error
     messages can render the full backend list. Spec phase-4M.txt:119-123
     explicitly extends ouroboros's bare-string ValueError. Aliases
     (claude_code, codex_cli, opencode_cli, hermes_cli, gemini_cli,
     kiro_cli, copilot_cli, anthropic, anthropic_api, sidecar, local,
     api, ollama, vllm) all resolve to canonical names.

  7. SidecarRuntime health_check uses GET /v1/messages with a 4-token
     prompt as canary. Why: phase-4M.txt:212-213 + existing
     sidecar_client.rs:469 health() pattern. Unavailable returned when
     HTTP fails or non-2xx; never panic.

  8. Stub adapters for the 9 not-yet-implemented backends (ClaudeCode,
     CodexCli, OpenCode, Hermes, GeminiCli, KiroCli, CopilotCli,
     Ollama, Vllm) return RuntimeError::NotImplemented from
     health_check + complete + stream so create_runtime() can dispatch
     to them without unreachable!() / unimplemented!() panics.
     Subsequent sub-cycles fill them in. This keeps the auditor happy
     and lets the dispatcher see "Unavailable" for B-37A.

Expected test outcomes (TESTING_FIRST)
  - Contract test: RuntimeBackend has exactly 11 variants
  - Contract test: resolve_backend accepts every documented alias and
    canonicalizes correctly
  - Contract test: resolve_backend on unknown candidate returns
    ResolveBackendError with non-empty `supported` list
  - Unit test: PermissionMode default is AcceptEdits
  - Unit test: SecretString does not Debug/Display the inner value
  - Integration test: create_runtime(Sidecar config) returns a
    healthy SidecarRuntime when sidecar feature enabled (gated)
  - Integration test: create_runtime(AnthropicApi config) returns a
    healthy AnthropicApiRuntime with env-loaded auth
  - Integration test: create_runtime(any of 9 stubs) returns
    Box<dyn RuntimeAdapter> whose health_check returns Unavailable
    (NOT a panic) -- this is the key "never crashes on a missing CLI"
    contract that all of B-37 depends on
  - All existing Phase 3J tests continue to pass (zero regression
    against AnthropicClient + SidecarProvider unwrapped paths)

Out of scope for this sub-cycle (deferred)
  - Tier C process isolation enforcement -> B-37B
  - stdout sanitization pipeline -> B-37B
  - Real CLI subprocess spawning for the 7 ouroboros ports -> B-37B
  - Ollama + Vllm real HTTP body -> B-37C
  - PerRoleRuntime config + fallback chain + SSB action_types -> B-37C
  - /runtime CLI command -> B-37D
  - Dashboard RuntimePanel + Tauri commands -> B-37D
  - Migration guide + metrics file -> B-37D
  - [cloud] default_mode -> [runtime] backend deprecation aliases -> B-37D

Risk notes
  - SidecarProvider impl LlmProvider (complete/embed) while
    AnthropicClient impl Provider (send_message/stream_message).
    The new RuntimeAdapter unifies both at the message level.
    SidecarRuntime must reach into SidecarProvider via the
    Anthropic-protocol HTTP route (sidecar server.py speaks
    /v1/messages already per metrics/baseline_findings_and_actions.md
    + CLAUDE.md), bypassing SidecarProvider::complete() string-shape.
    Plan: SidecarRuntime borrows the reqwest::Client + base URL
    from SidecarConfig and POSTs Anthropic-shaped messages directly.
  - Existing 18 AnthropicClient tests must not regress. AnthropicApiRuntime
    wrapper preserves all behavior by delegating to send_message/stream_message.

Unknowns / open questions for next sub-cycle
  - Tier C isolation API surface in current orchestrator
    (WindowsAppContainerSandbox at orchestrator/.. -- need exact
    spawn signature). Defer to B-37B context-compiler.
  - SSB action_type registration mechanics (CHECK constraint string,
    as_sql_str, from_str, payload struct, SCHEMA_VERSION bump). Defer
    to B-37C context-compiler.
