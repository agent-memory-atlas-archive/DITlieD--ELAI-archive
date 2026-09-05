phase-3J-UNIFIED decisions

What: merge AnthropicRuntimeClient + LocalRuntimeClient + CliApiClient into single RuntimeClient struct.
Policy enums (ToolPolicy, BudgetPolicy) replace if/else branching on backend type.

Key decision: build_runtime gains optional BackendConfig param.
- None = Api path (resolve_cli_auth_source, ToolPolicy::Full, no budget compute, prompt cache ON).
- Some(config) = Local path (AuthSource::None, ToolPolicy::FsmFiltered, ContextBudget::compute, prompt cache OFF).
All 9 existing call sites pass None (unchanged behavior).
prepare_turn_runtime branches on self.backend_config instead of self.local_config.

stream() branch points (exactly 3):
1. Auth selection: BackendMode::Api -> resolve_cli_auth_source(), Local -> AuthSource::None (in constructor)
2. Tool filtering: match on ToolPolicy enum (Full/FsmFiltered/None)
3. Post-stream: BackendMode::Api -> push_prompt_cache_record, Local -> skip

Expected test outcome: 7 dispatcher tests GREEN (untouched). cargo check --workspace = 0.
Unknowns: none identified — tokio::Runtime is per-turn (no sharing concern).
