E-40-trancheB-decisions.md

Session: s-22600 Tranche B ITEM B.4 (E-40 translate_tool_request live wiring)
Date: 2026-05-31
Status: SHIPPED (adapter-side wire; CLI install site is a separate followup)

Scope landed

The two empty-response match arms at production_agent_rpc.rs (TOOL_SYMBOL_DSL_QUERY at the former L1497, TOOL_SYMBOL_DATAFLOW_IMPACT at the former L1520) now honour an optional `SymbolGraphAdapter` injected on the server struct. When the adapter is `Some`, real DSL queries + dataflow impact computations dispatch through it and the typed response carries actual matches / impacted symbols. When the adapter is `None`, the arms preserve their pre-B.4 behaviour (typed empty response) so existing harness / test paths continue to compile and produce the expected wire shape.

What shipped

1. `ProductionAgentRpcServer.symbol_graph_adapter: Option<Arc<dyn SymbolGraphAdapter>>`. New field added after `clock`. Default `None` in `new()`. Builder `with_symbol_graph_adapter(adapter: Arc<dyn SymbolGraphAdapter>)` added next to `with_artifact_vm` (canonical Option-Arc-trait-object builder shape).
2. `translate_tool_request` signature gains a new arg `symbol_graph_adapter: Option<&Arc<dyn SymbolGraphAdapter>>` at the end (mirrors how `ssb` / `artifact_vm` / `clock` thread through).
3. Method caller at `ProductionAgentRpcServer::handle_tool_request` (former L905) threads `self.symbol_graph_adapter.as_ref()` into the new arg.
4. DSL arm: parses `SymbolDslQueryRequest`, calls `adapter.dsl_query(&wire.dsl, &PathBuf::from(&wire.project_root), wire.depth_cap)` when adapter Some. Err -> `tracing::warn` + empty response (best-effort per adapter trait doc). PRESERVES the existing `emit_symbol_query` beacon (additive observability). Additionally emits `SsbPayload::SymbolGraphEvent` with `SymbolGraphEventKind::SymbolDslExecuted` via `crate::symbol_graph::notify_symbol_graph_event` (Pattern-2 reuse, SCHEMA 193 -- no new variant, no schema bump). SSB emit failure logged via `tracing::warn` and swallowed so the tool dispatch result is unaffected.
5. Dataflow arm: same shape with `adapter.dataflow_impact(&wire.symbol, ...)` and `SymbolGraphEventKind::DataflowImpactComputed`.
6. `translate_tool_request_for_test` (test-support feature-gated alias at L2195) passes `None` for the new arg so the dispatch contract test stays compatible.

ENV gates

NO new gate. Per s-22300 B-2 the upstream producers are already gated by `ELAI_SYMBOL_GRAPH_5TH_RRF_ENABLE`; this wire is "honor the gate via real translation" -- the CLI startup is the gate-decision site (it decides whether to chain `with_symbol_graph_adapter` based on the env-gate + cache availability).

Variant verification

Both `SsbActionType::SymbolGraphEvent` (SCHEMA 193) and `SymbolGraphEventKind::SymbolDslExecuted` + `::DataflowImpactComputed` exist in ssb.rs (verified via grep prior to emit). No SCHEMA_VERSION bump required. No new variant added.

What was NOT done (out-of-scope per prompt)

The CLI startup wire (`cli::live_cli::build_local_knowledge_stack` chaining `.with_symbol_graph_adapter(...)` on the ProductionAgentRpcServer builder) is OUT-OF-SCOPE for ITEM B.4. The adapter side is hot-pluggable: any future install site can call `with_symbol_graph_adapter(Arc::new(SymbolGraphCacheAdapter::new(cache, 500)))` once the SymbolGraphCache lifecycle question is settled (where the cache file lives, when it loads, who owns it). Until that ships, the field stays None and the wire shape stays empty -- preserves backwards compat.

Non-obvious decisions

DECISION-1: PROJECT_ROOT field is on the WIRE.
SymbolDslQueryRequest already has `project_root: String` (lib.rs:638-642, verified at edit time). No need for a sentinel default. Pass `PathBuf::from(&wire.project_root)` straight through; the adapter ignores it today (SymbolGraphCacheAdapter::dsl_query takes `_project_root` -- the cache binding is set at adapter construction, not per-query) but the field is there for future per-project cache routing.

DECISION-2: SymbolGraphEvent emit lives in the dispatcher arm, not inside the adapter.
The adapter trait is in `elai_knowledge::symbol_graph_adapter` which CANNOT depend on `elai_orchestrator::ssb` (reverse edge). So the SSB beacon emission lives at the call site that owns the Ssb handle -- i.e. `translate_tool_request` in production_agent_rpc.rs. This is the same pattern as `emit_symbol_query` (also lives in production_agent_rpc.rs, not in the adapter).

DECISION-3: Two beacon rows per call (additive).
`emit_symbol_query` is preserved AND a new `SymbolGraphEvent` row is emitted alongside. Per s-22300 B-2 the dashboard SymbolQuery panel already consumes the first row; the new SymbolGraphEvent row feeds the dashboard SymbolGraphEvent panel (downstream from SCHEMA 193 ship). No risk of duplicate counts -- both rows carry distinct action types and serve distinct consumers.

DECISION-4: Best-effort degrade on adapter Err.
Adapter errors (CacheUnavailable, DslParse, InvalidInput, Internal) all degrade to empty typed response via `tracing::warn`. This matches the adapter trait doc-comment: "the 5th-list retriever degrades to empty results on any error" (knowledge::symbol_graph_adapter.rs:54-55). Hard-Err propagation would break the agent loop (best-effort over symbol-graph tools is the design intent).

DECISION-5: Agent-id for SymbolGraphEvent row is "production_agent_rpc".
Distinguishes the agent-loop-side emit from the existing `notify_symbol_graph_event` caller in the symbol_graph module (which has not been wired into prod yet). Future consumers can filter by agent_id to separate the two origins.

Grounding

- workflow/sessions/s-22600/exploration/trancheB-integration.md SUB-4 (Recommendation A: bypass ToolDispatcher; thin wire via field on ProductionAgentRpcServer).
- workflow/sessions/s-22600/context-package-trancheB.md ITEM B.4 (wire shape + cache lifecycle out-of-scope).
- .elai_cc/crates/knowledge/src/symbol_graph_adapter.rs (full trait + error enum read; coarse-grained best-effort design).
- .elai_cc/crates/orchestrator/src/symbol_graph/mod.rs:97-183 (SymbolGraphCacheAdapter concrete impl; notify_symbol_graph_event helper signature).
- .elai_cc/crates/orchestrator/src/ssb.rs grep `SymbolGraphEventKind` (Pattern-2 4-arm enum: SymbolDslExecuted / DataflowImpactComputed / AgentImpactClaimRefuted / ScopeGraphResolved; SCHEMA 193).
- .elai_cc/crates/ipc-types/src/lib.rs:638-699 (SymbolDsl* / SymbolDataflow* wire types; project_root field confirmed present).
- workflow/reasoning/E-40-tranche-B2-decisions.md DECISION-9 KNOWN-RESIDUAL bullet 2 (translate_tool_request empty-responses intentional B-2 carry).
