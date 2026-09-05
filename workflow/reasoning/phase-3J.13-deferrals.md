Phase 3J.13 VERIFY + MEASURE (E2B) — Deferral Documentation
Date: 2026-04-10

Steps 3J.5 through 3J.12 are DEFERRED. These steps require vLLM runtime, GPU multiplexing, and advanced inference features that depend on steps 3J.1-3J.4 (VllmProvider, ModelPool, DoLa, TurboQuant) which were not implemented in this iteration. The WIRE.* sub-steps were prioritized instead to establish the orchestrator integration foundation.

DEFERRED STEPS:

3J.5 ActAdd steering vectors
  Status: DEFER
  Reason: requires vLLM backend with layer-level access for mid-layer injection. VllmProvider (3J.1) not yet implemented.
  Re-evaluation: when VllmProvider is implemented and E2B model is served via vLLM.
  Prerequisite: 3J.1, 3J.3

3J.6 GPU multiplexing (VRAM safety margin, colocation policy)
  Status: DEFER
  Reason: requires ModelPool (3J.2) and vLLM backend. Current single-model sidecar uses direct ollama/vllm invocation.
  Re-evaluation: when ModelPool manages multiple concurrent models.
  Prerequisite: 3J.1, 3J.2

3J.7 Predictive model pre-loading
  Status: DEFER
  Reason: FSM-state-driven background loads require ModelPool swap protocol. Current sidecar loads one model at startup.
  Re-evaluation: when ModelPool and FSM are integrated for multi-model dispatch.
  Prerequisite: 3J.2, 3J.6

3J.8 Workload profiling (per-agent per-phase metrics to SSB)
  Status: DEFER
  Reason: requires agent system (Phase 3C) to be fully operational with per-agent dispatch. SSB infrastructure is wired but no agents produce workload telemetry yet.
  Re-evaluation: when agent execution produces real SSB traces.
  Prerequisite: 3C agent system operational

3J.9 MILP scheduling evaluation
  Status: DEFER
  Reason: requires workload profiling data (3J.8) to compare MILP vs greedy scheduling. No profiling data available.
  Re-evaluation: when 3J.8 workload profiling produces baseline metrics.
  Prerequisite: 3J.8

3J.10 MCP bridge
  Status: DEFER
  Reason: external MCP server connection requires network security review and tool discovery protocol. Feature-gated behind cfg(feature="mcp").
  Re-evaluation: when MCP integration is prioritized for external tool access.
  Prerequisite: none (independent)

3J.11 Tiered static analysis hooks (T1/T2/T3)
  Status: PARTIALLY DONE via WIRE.RULES + WIRE.STUBS
  Reason: T1 predicate evaluation is wired via DefaultRuleEngine + seed rules. T2 tree-sitter evaluation wired via TreeSitterAstEvaluator. T3 (pattern-match-only) deferred — requires specific pattern definitions.
  Re-evaluation: T3 patterns defined when auditor violations accumulate enough data for auto-proposal.
  What exists: T1 predicate DSL evaluation in pre-tool hook, T2 AST pattern matching via tree-sitter.

3J.12 AppContainer isolation (Windows)
  Status: DEFER
  Reason: requires Phase 4A agent process isolation. Current CLI runs as single process. AppContainer Job object limits require agent spawning infrastructure.
  Re-evaluation: when Phase 4A agent process isolation is implemented.
  Prerequisite: 4A

FINDINGS FROM VERIFICATION:

1. CliInteractiveGate: defined (main.rs:5981-6010) but NOT wired into ConversationRuntime.
   ConversationRuntime has no InteractiveGate parameter. This is dead code.
   Impact: risk-level auto-advance for FSM gates is not enforced at runtime.
   Resolution: wire into ConversationRuntime when InteractiveGate trait is added to runtime builder.

2. phase_slot_tokens defaults to 0: ContextBudget Slot 2 (PhaseSpec) receives 0 tokens.
   Impact: plan phase context is not injected into the prompt assembly.
   Resolution: set phase_slot_tokens when a plan phase is active and PlanSkeleton is loaded.

3. usearch-hnsw feature: C++ compilation fails on this Windows environment (missing MSVC dev env vars).
   Impact: --all-features build fails. Default build unaffected.
   Resolution: environment issue, not code issue. Requires vcvarsall.bat in build environment.
