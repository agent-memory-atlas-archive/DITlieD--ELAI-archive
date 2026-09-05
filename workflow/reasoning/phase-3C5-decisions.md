What: Supervision tree for orchestrator crate. SupervisionManager tracks per-agent restart counts within phase windows and escalates when limit is hit. SSB schema v2->v3 adds supervision_exhausted + spiral_detected action types.

Key decisions:
- SupervisionManager<C: ClockProvider> uses static dispatch (generic param), matching RecoveryManager pattern. No Arc<dyn>.
- handle_death() is synchronous (no async). SupervisionManager is pure policy logic — it does NOT write to SSB. Caller (future orchestrator loop) is responsible for writing supervision_exhausted to SSB after receiving Escalated outcome. This keeps the module testable without async runtime.
- SupervisionState stored in HashMap<Uuid, SupervisionState> inside SupervisionManager. Not on AgentHandle (struct locked by P3C.1).
- interactive_mode passed as bool to handle_death(), not stored on SupervisionManager. Cleaner API — mode can change per call in future multi-mode scenarios.
- tracing crate added to orchestrator Cargo.toml (0.1, ~200M downloads, no CVEs). Required for warn! on max_restarts clamp.
- spiral_detected payload is minimal (agent_id, phase_id, spiral_depth) — full design deferred to 3C.6 per spec.
- Window expiry logic: if elapsed > restart_window, reset restart_count and window_start before incrementing. This means a crash after a long gap starts a fresh window.

Expected test outcome: 10+ unit tests in supervision.rs, 3 contract tests in supervision_contract.rs. All existing SSB/checkpoint/pool/recovery tests remain green (schema migration is DROP+recreate, tests use fresh DBs).

Unknowns:
- Exact escalation behavior for ReduceScope (deferred to 3C.6 orchestration loop).
- spiral_detected trigger conditions (3C.6).
