phase: B2b — listen-before-spawn wiring in AgentPoolBabyStepExecutor::execute()
date: 2026-05-15
session: s-5240

WHAT:
Wire the orchestrator's IPC accept loop into AgentPoolBabyStepExecutor::execute() so the listener is bound to the per-session socket path BEFORE the agent process is launched. After spawn, accept_one drives run_agent_session against an injected Arc<dyn AgentRpcServer> to route the agent's InferRequest/ToolRequest frames. Cascades through build_default_baby_step_executor (signature widened), CLI run_subcommand.rs, and dashboard-v2/src-tauri/src/lib.rs to construct an Arc<Mutex<IpcGateway>> + NoOpAgentRpcServer placeholder.

KEY DECISIONS:
1. ScriptedAgentRpcServer visibility: moved into ipc_accept::test_support module gated on cfg(any(test, feature = "test-support")). Both orchestrator-side tests and contracts/tests now import the canonical version. orchestrator Cargo.toml exposes the test-support feature; contracts/Cargo.toml dev-deps on it.
2. NoOpAgentRpcServer used as the wiring placeholder for CLI + dashboard + the existing real-sidecar e2e test (returns InferOutcome::Failed("...B2c pending") rather than panicking). B2c will swap it for ProductionAgentRpcServer wrapping AnthropicClient + InProcessToolDispatcher.
3. The #[ignore]'d full-binary round-trip test in ipc_accept_loop_contract.rs is REPLACED by agent_pool_listen_before_spawn_contract.rs which proves the executor-side invariant with an in-process tokio stub agent and ScriptedAgentRpcServer canned response. Cleaner: the new contract exercises the integration point we care about; the helpers are already covered by the other 4 contracts in ipc_accept_loop_contract.rs.
4. Fast-fail short-circuit: when launch_agent_process returns Err AND no in-process stub is registered (socket_path_notifier empty), execute() returns BabyStepOutcome::Failed immediately. Without this, the missing-binary contract path took 120.04s (the default timeout). Now 0.12s.
5. Windows named-pipe race fix: PlatformListener::WindowsPipe now stores the NamedPipeServer handle; accept_one reuses it (rather than dropping + recreating, which leaves an ERROR_FILE_NOT_FOUND window per MSDN).
6. Magic number `5` for grc_max_calls replaced with const DEFAULT_AGENT_GRC_MAX_CALLS: u32 = 5.

EXPECTED TEST OUTCOMES (all confirmed green):
- agent_pool_listen_before_spawn_contract (new): PASS in 0.05s.
- agent_pool_executor_wiring_contract: PASS in 0.12s (was 120.04s pre-fix).
- ipc_accept_loop_contract: 3 always-on contracts PASS; #[ignore]'d round-trip deleted.
- agent_pool_executor_wiring_contract: 4/4 PASS.
- baby_step_executor_contract: PASS (signature update for new args).
- dashboard_real_executor_wired_contract: PASS (signature update).
- e2e_mock_provider_scaffolds_react_app_via_chokepoint: PASS in 2.53s (regression guard).
- cargo check --workspace: PASS (no errors).

UNKNOWNS RESOLVED (verified during execution):
- gateway type convention: Arc<tokio::sync::Mutex<IpcGateway>> (matches existing orchestrator pattern).
- run_agent_session now takes Arc<dyn AgentRpcServer + Send + Sync + 'static> rather than generic to avoid Sized bounds at trait-object call sites.
- contracts/tests use the test-support feature via contracts/Cargo.toml dev-dep on `elai-orchestrator = { ..., features = ["test-support"] }`.

OPEN UNKNOWNS (deferred to B2c):
- ProductionAgentRpcServer wrapping AnthropicClient (sidecar URL or cloud).
- InProcessToolDispatcher mapping for ToolRequest verbs (study dispatcher.rs dispatch_* methods).
- GroundingRequest handling: per spec-vs-code cross-cut X3, run_agent_session currently routes GroundingRequest through a placeholder; production impl needs PipelineGrc proxy. Logged as separate concern for B2c or later.

AUDITOR RESULT: BLOCK-with-2-blocks. Fixer resolved both BLOCKs + 3 of 4 WARNs.
REMAINING WARN: 4 (this decision doc — now satisfied).
INFO: elai.map stale for B2a/B2b symbols; regen with codebase_mapper.py post-commit.
