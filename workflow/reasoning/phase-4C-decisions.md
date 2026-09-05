Phase 4C decisions

What: IPC hook interception layer — HookTrait + IpcHookDispatcher in plugins, wired into orchestrator ipc_gateway.rs

Key decisions:
- HookTrait is sync (not async) because TieredAnalysis is CPU-bound and RuleEngine is not Send
- Renamed HookEvent to IpcHookEvent to avoid collision with existing HookEvent in hooks.rs
- SsbFindingWriter trait defined in plugins to avoid circular dep (orchestrator implements it)
- CrossSessionLockChecker trait also in plugins for same reason
- thiserror made non-optional in plugins Cargo.toml (was behind mcp feature, now always needed)
- code_simplify() does not exist in tools crate — PostToolUse ShortCoder step is not wired
- SCHEMA_REQUIRED lives in rules/predicate.rs, not tiered_analysis.rs — no tiered_analysis patch needed

Expected test outcome: 47 pass / 0 fail (7 unit + 30 contract + 10 adversarial)

Unknowns:
- code_simplify will need to be implemented in a future phase for PostToolUse ShortCoder
- Actual SSB writer implementation (orchestrator side) deferred to integration wiring
