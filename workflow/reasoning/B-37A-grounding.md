GROUNDING SWEEP -- phase B-37A
Source: F:\.ELAI_workflow\docs\thefinalglance\B-plan\B-37_multi_runtime_backend.md
Date: 2026-05-16
FSM: PLANNING | Session: s-18544
Scope: B-37A FOUNDATION ONLY (RuntimeBackend enum, RuntimeAdapter trait, resolve_backend,
  create_runtime, RuntimeFactoryConfig, RuntimeCapabilities, RuntimeHealth, PermissionMode,
  ResolveBackendError, port Sidecar+AnthropicApi through factory, [backend]->[runtime] migration)

SPEC REFERENCES

phase-4M.txt (full) | GROUNDED | lines 60-72 (11-variant enum), 74-93 (trait vtable),
  99-110 (RuntimeFactoryConfig), 112-117 (PermissionMode), 119-123 (ResolveBackendError),
  183-184 ([runtime] schema), 218-219 (deprecation aliases), 451 (EXIT CRITERIA 11 variants)

phase-3J.txt (3J.LOCAL, 3J.PIPELINE, 3J.WIRE.*) | GROUNDED | lines 683-767 (3J.PIPELINE,
  [backend] mode="local"|"api"|"hybrid"|"auto" at 705-706), 877+ (3J.WIRE.FOUNDATION),
  1106+ (3J.WIRE.STUBS)

providers.txt S1, S4 | GROUNDED | S1=LlmProvider trait lines 19-23
  (complete(prompt,config)->Stream<Token>), S4=ModelPool lines 110-169

orchestrator.txt S12 (Tier C IPC) | GROUNDED | lines 414-640 (S12 Agent Isolation Model,
  Tier C process separation at 488-534, AppContainer at 601-609)

workflow/papers/ouroboros-runtime-interface.md | GROUNDED | file exists, 420 lines,
  lists 8 ouroboros runtime backends (lines 29-38), AgentRuntime Protocol (lines 62-108),
  RuntimeFactoryConfig ELAI port notes (lines 364-419)

CLAIM CLASSIFICATION

RuntimeBackend has exactly 11 variants | GROUNDED | phase-4M.txt:60-72 enumerates all 11;
  EXIT CRITERIA line 451 confirms "11 variants (2 existing + 7 ouroboros + 2 native)"

7 CLI-spawning backends (ouroboros port) | GROUNDED | phase-4M.txt:15 lists claude/codex/
  opencode/hermes/gemini/kiro/copilot = 7; ouroboros paper note at line 366 confirms "ELAI adds
  Sidecar+AnthropicApi+Ollama+Vllm, total=11+goose as potential 12th" (goose out of scope)

[runtime] backend schema replacing [backend] mode | GROUNDED | phase-4M.txt:183-184
  "[runtime] backend = ..." and "[agents.<role>.runtime]" entries; 3J.txt:705-706 confirms
  existing [backend] mode schema being replaced

Deprecation aliases mode=local->backend=sidecar, mode=api->backend=anthropic_api | GROUNDED
  | phase-4M.txt:218-219 "map 'local' -> sidecar, 'api' -> anthropic_api. Keep old names as
  deprecated aliases for one release."

RuntimeAdapter vtable (complete/stream/capabilities/health_check) | GROUNDED | phase-4M.txt:
  74-93 defines exact Rust trait shape with async fn complete(messages: Vec<Message>,
  config: CompletionConfig) -> Result<CompletionResponse, RuntimeError>

LlmProvider trait compatibility / alias claim in Task 2 | MISMATCH (see ACTIONS REQUIRED) |
  providers.txt:S1 + codebase grep: existing LlmProvider is
  fn complete<'a>(&'a self, prompt: &'a str, config: &'a OllamaConfig) -> LlmFuture<Vec<String>>
  (providers/src/providers/ollama.rs:352-353). RuntimeAdapter::complete takes
  (messages: Vec<Message>, config: CompletionConfig) -> Result<CompletionResponse>.
  Signatures are NOT alias-compatible. Thin wrapper is required (spec covers this: "if
  compatible, alias; else introduce thin wrapper" -- but B-37 Task 2 implies alias is
  plausible without confirming first).

ResolveBackendError includes supported list | GROUNDED | phase-4M.txt:119-123
  struct ResolveBackendError { candidate: String, supported: Vec<String> }.
  Ouroboros omits supported list (paper line 47) -- ELAI extension is explicitly specified.

PermissionMode enum (4 variants: AcceptEdits, Plan, DangerouslySkipPermissions, Default)
  | GROUNDED | phase-4M.txt:112-117; ouroboros paper lines 127-134 confirms
  "acceptEdits"|"bypassPermissions"|"default" as source; ELAI adds Plan variant
  (4M.0 GROUNDING section: "PermissionMode -> GROUNDED: ouroboros claude_permissions.py")

BoxStream<'static, StreamChunk> in RuntimeAdapter::stream | VERIFY | grep BoxStream in
  providers crate: NO HITS. BoxStream must be imported from futures::stream::BoxStream.
  StreamChunk type also not yet defined in codebase. B-37A must define these types.
  Not a blocking issue (new types introduced by B-37A); verified absent from codebase.

SecretString in RuntimeFactoryConfig | VERIFY | grep SecretString in providers crate:
  NO HITS. Type must be introduced or imported (e.g. secrecy crate or newtype).
  phase-4M.txt:103 uses SecretString without defining it. Needs resolution in B-37A.

create_runtime returns Box<dyn RuntimeAdapter> (not Result) | VERIFY | phase-4M.txt:179
  "Stub `create_runtime(cfg: RuntimeFactoryConfig) -> Box<dyn RuntimeAdapter>` returning
  Err('not implemented')". Returns Box not Result -- but then says "returning Err". This is
  internally inconsistent in spec. Likely should be Result<Box<dyn RuntimeAdapter>, RuntimeError>.
  Confirmed inconsistency at phase-4M.txt:179 vs description at 80 (Task 4 says "returning Err").

SidecarRuntime health_check canary is GET /v1/messages with 4-token prompt | GROUNDED
  | phase-4M.txt:212-213; consistent with existing SidecarProvider at
  sidecar_client.rs (checked via grep hit at line 469).

Ollama health_check endpoint /api/tags | GROUNDED | phase-4M.txt:317 (4M.4 TASK).
vLLM health_check endpoint /health | GROUNDED | phase-4M.txt:317.

ACTIONS REQUIRED

- MISMATCH: B-37 Task 2 says "alias the two if so" re LlmProvider vs RuntimeAdapter.
  Code evidence at providers/src/providers/ollama.rs:352-353 shows LlmProvider signature
  fn complete(&self, prompt: &str, config: &OllamaConfig) -> LlmFuture<Vec<String>>.
  RuntimeAdapter::complete takes (messages: Vec<Message>, config: CompletionConfig).
  These are NOT aliasable. Thin wrapper is MANDATORY, not optional.
  Resolve before TESTING_FIRST: update Task 2 to "introduce thin RuntimeAdapter wrapper
  over LlmProvider; do not attempt alias." phase-4M.txt:95-97 already covers this path
  ("else introduce a new trait and adapter wrapper") -- the B-37 task text is just
  ambiguously worded, not a spec violation.

- VERIFY: create_runtime return type inconsistency in phase-4M.txt:179. Spec says
  "Box<dyn RuntimeAdapter> returning Err". Resolve as
  Result<Box<dyn RuntimeAdapter>, RuntimeError> before writing skeleton.
  Check phase-4M.txt:179 + ouroboros paper line 112 (create_agent_runtime returns
  AgentRuntime, not Result -- errors deferred to execute_task). Decide which shape ELAI uses.

- VERIFY: SecretString type not in codebase. Before B-37A coding, decide:
  (a) introduce secrecy crate (requires human approval per CLAUDE.md), or
  (b) use newtype SecretString(String) with ZeroizeOnDrop in providers crate.
  grep "secrecy\|ZeroizeOnDrop\|SecretString" .elai_cc/Cargo.toml before deciding.

- VERIFY: BoxStream<'static, StreamChunk> and StreamChunk not in codebase.
  futures::stream::BoxStream exists via futures crate (already present in workspace?).
  StreamChunk must be defined in B-37A providers/src/runtime/mod.rs.
  grep "BoxStream\|futures" .elai_cc/crates/providers/Cargo.toml to confirm futures dep.

VALIDATION WARNINGS

B-37 Reads section cites "workflow/papers/ouroboros-runtime-interface.md" but
  phase-4M.txt READ BEFORE IMPLEMENTING section (line 47) cites
  "workflow/papers/ouroboros-interface.md" (without "runtime-" prefix). The companion
  input param and the actual file at F:\.ELAI_workflow\workflow\papers\ouroboros-runtime-interface.md
  match the B-37 citation; the spec's own READ section has a different filename.
  Confirm paper-ingestor output path matches what phase-4M.txt expects, or the spec
  reference is stale. Low risk since file exists at the B-37-cited path.

Ouroboros paper notes 8 runtime-capable backends (line 347: "len(runtime_backend_choices())==8"
  includes goose). Phase-4M spec and B-37 use 7 CLI backends (exclude goose). The
  discrepancy is documented in paper line 366 ("goose as potential 12th"). Not a blocker
  but goose adapter wiring is a known deferred item -- flag for B-37C or later sub-cycle.

all sources validated (spec files exist, line ranges confirmed on-topic, companion paper exists)
