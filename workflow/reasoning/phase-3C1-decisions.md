Phase 3C.1 decisions

What: AgentPool trait + AgentHandle + AgentStatus + InProcessAgentPool in orchestrator crate.

Key decisions:
1. InProcessAgentPool<E: EntropySource> — static dispatch, follows RuntimeContext pattern from elai-core.
2. ModelPool as Arc<dyn ModelPool> — trait object acceptable for shared hardware resource (single instance).
3. AgentPool trait is not generic — object-safe for Phase 4A IPC transition.
4. UUID via EntropySource::fill_bytes + RFC 4122 bit-setting (entropy quarantine).
5. No blake3/flume added — not needed in 3C.1. blake2 already in project for 3C.6.
6. VRAM_MIN_BYTES = 512 MiB — configurable via AgentPoolConfig.vram_required_bytes.
7. AgentStatus has 6 variants (no BudgetExceeded — that is 3C.2 scope).
8. Designer role has distinct AgentGrcConfig name but same budget as Executor.

Expected test outcome: 19 tests, all pass. cargo check clean (0 warnings from agent_pool.rs).

Unknowns:
- Designer GRC required_steps may diverge from Executor in later phases.
- ModelPool real impl deferred to providers/ crate (Phase 4F scope).
