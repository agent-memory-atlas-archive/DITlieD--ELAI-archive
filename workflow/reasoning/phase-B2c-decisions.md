phase: B2c — ProductionAgentRpcServer wrapping AnthropicClient + InProcessToolDispatcher
date: 2026-05-15
session: s-5240

WHAT:
ProductionAgentRpcServer impls the 2-method AgentRpcServer trait. handle_infer_request wraps AnthropicClient.send_message (sidecar or cloud per ELAI_GROUNDING_BACKEND env var). handle_tool_request translates wire-format ToolRequest into InProcessToolDispatcher dispatch_<verb> calls (edit, validate, classify_failure, git_commit, file_read_outline, incremental_compile). CLI run_subcommand.rs + dashboard-v2/src-tauri/src/lib.rs construct the real InProcessToolDispatcher (via elai_tools::dispatcher) and pass it to ProductionAgentRpcServer.

KEY DECISIONS:
1. ELAI_GROUNDING_BACKEND="cloud" branches to AnthropicClient::from_env(). Default + any other value branches to sidecar at http://localhost:8100 (overridable via ANTHROPIC_BASE_URL). When cloud OAuth fails, fall back to NoOpAgentRpcServer with tracing::warn — non-fatal.
2. Trait surface: InferOutcome = Response(String)|Failed(String); ToolOutcome = Result(Vec<u8>)|Failed(String). The 6 non-Infer/Tool verbs (Heartbeat, Checkpoint, ThinkStep, BashRequest, GroundingRequest, ReplanRequest) are handled structurally in run_agent_session and do NOT reach the trait — so ProductionAgentRpcServer does not implement them.
3. InProcessToolDispatcher construction: requires Arc<dyn TieredValidator> via default_validator(integration, "rust"). Production has no IntegrationCommandRunner impl anywhere, so a NoOpIntegrationCommandRunner is defined INLINE in CLI + dashboard wiring (where elai-tools is a regular dep). NOT in orchestrator crate — that would create elai-orchestrator -> elai-tools -> elai-orchestrator cycle (elai-tools is dev-dep-only of orchestrator per Cargo.toml:96).
4. Tool dispatch translation: separate fn translate_tool_request handles wire-format -> dispatcher-arg conversion. Specific decisions:
   - TOOL_VALIDATE / TOOL_COMPILE_CHECK: derive project_dir = wire.file_path.parent() (the file's enclosing dir). NOT pass file_path directly.
   - TOOL_FILE_READ: if wire.offset.is_some() || wire.limit.is_some(), return ToolOutcome::Failed("windowed reads not yet supported") — explicit error, NOT silent drop.
   - TOOL_GIT_COMMIT: GitBranchPolicy::default() (BranchPerPlan), plan_id from wire OR UUIDv4 fallback. anchor_path = wire.paths.first(). Empty paths returns descriptive error.
   - TOOL_GIT_UNDO / TOOL_GIT_BRANCH: ToolOutcome::Failed("not implemented in ProductionAgentRpcServer (B2c)") with explicit message.
   - Unknown tool_id: ToolOutcome::Failed("unknown tool_id: {id}").
5. NullToolDispatcher (from initial executor pass) deleted entirely — was a wiring placeholder that violated the "no silent stubs in production" rule.
6. Contract test Test B uses real InProcessToolDispatcher (elai-tools is dev-dep of orchestrator) backed by PassthroughValidator stub exercising dispatch_file_read_outline against a real tempdir file. Tests A and C use the simpler stub patterns since they exercise the inference path and unknown-tool path respectively.

EXPECTED TEST OUTCOMES (all confirmed green):
- production_agent_rpc_contract: 3/3 PASS in 0.01s.
- agent_pool_listen_before_spawn_contract (B2b regression): 1/1 PASS in 0.04s.
- agent_pool_executor_wiring_contract: 4/4 PASS.
- ipc_accept_loop_contract: 4/4 PASS.
- e2e_mock_provider_scaffolds_react_app_via_chokepoint: PASS, ~2.5s.
- cargo check --workspace: PASS.

UNKNOWNS RESOLVED:
- AnthropicClient::from_env() reads ANTHROPIC_BASE_URL + saved OAuth automatically — no manual resolve_saved_oauth_token_set needed in wiring code.
- elai_tools::dispatcher::InProcessToolDispatcher::new accepts Arc<dyn TieredValidator>; constructor for production validator wires CargoDeterminismRunner + LanguageSyntaxRunner + caller-supplied IntegrationCommandRunner.
- No production IntegrationCommandRunner exists anywhere; orchestrator/src/task_runner.rs:8370 references one but it's commented as conceptual. Inline NoOpIntegrationCommandRunner in CLI + dashboard is the pragmatic solution.

OPEN UNKNOWNS / DEFERRED:
- Real IntegrationCommandRunner that actually runs tier-3 cargo/pytest commands: defer (separate phase or hand-roll in dashboard validator UI work).
- Wire-format ToolRequest schema evolution: WARNs around offset/limit and validate file_path semantics document where the ipc-types schema is thinner than the dispatcher's argument shape. Future ipc-types revision may add explicit project_dir + window fields.
- B3 will reveal whether the LLM-driven agent actually uses dispatch_* IPC requests during a real TODO-app build, OR whether file writes happen via direct fs access in the agent process. If the latter, tool dispatch wiring is exercised only minimally and most of B2c's translation layer is dormant.

AUDITOR RESULT: BLOCK-with-2-blocks. Fixer resolved both BLOCKs + all 5 WARNs in single cycle.
INFO: phase_4E.toml change pre-existing at session start (not B2c-related). elai.map regen pending B-area completion.
