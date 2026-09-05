Tranche D decisions (s-22600) -- D.1 C-47b CaseBase rusqlite persistence + D.2 C-42b mcp_authz wire-through

Date: 2026-06-01
Session: s-22600 orchestrate, sub-phase trancheD
Source spec: workflow/sessions/s-22600/context-package-trancheD.md + workflow/sessions/s-22600/exploration/trancheD-integration.md
SCHEMA_VERSION (frozen): 195. NO bump. D.1 + D.2 reuse existing Pattern-2 variants per D9.

Scope shipped this pass (per explorer recommendation "D.2 phases A+B together; Phase C deserves separate bisectable commit"):
  D.1 -- C-47b CaseBase rusqlite persistence: SHIP
  D.2 Phase A -- NEW OrchestratorMcpAuthzSsbEmitter adapter: SHIP
  D.2 Phase B -- mcp_bridge.rs:969 migration to compose_with_tool_gate_emit: SHIP
  D.2 Phase C -- NEW plugins_macros crate + #[authz] attribute: DEFERRED (see D2-phase-c-deferral.md)

------------------------------------------------------------------------
D1 -- PERSIST-PATH
------------------------------------------------------------------------
.elai_cc/data/case_base/case_base.sqlite3 (canonical project-data dir, mirrors B-6 quad MCP_FUZZ_CORPUS_DIR).
REJECT: ~/.elai (cross-project pollution) + crates/plugins/data/ (per-crate violates workspace conventions).

------------------------------------------------------------------------
D2 -- DUAL-GATE
------------------------------------------------------------------------
Persistence layered on emit gate: requires BOTH ELAI_C47_CASE_BASE_ENABLE (emit) AND ELAI_C47B_CASE_BASE_PERSIST_ENABLE (persist) to actually persist a row. Wire-dark default OFF.
REJECT: standalone persist (would persist while emit is dark; wire-asymmetry).
Note: the registry already exposes the persist hook unconditionally; the env-gate is read by the registry on the persist branch so callers do not need to pre-check. Adapter is unconditional.

------------------------------------------------------------------------
D3 -- FAIL-CLOSED-PERSIST
------------------------------------------------------------------------
On persist_case error: SKIP LRU admit (record_call returns without putting the case in LRU) + emit ToolCalibrationEvent::DriftDetected with details_json.persist_error=true via the existing emitter. Persist-FIRST / lru-put-SECOND ordering.
REJECT: best-effort write (silent durability loss).

------------------------------------------------------------------------
D4 -- CONNECTION-ISOLATION
------------------------------------------------------------------------
SEPARATE rusqlite::Connection from Ssb pool. Wrapped in tokio::sync::Mutex<Connection> at CasePersistStore module level. Sqlite single-writer model: sharing the SSB writer connection causes same-thread contention.
REJECT: shared SSB connection.

------------------------------------------------------------------------
D5 -- REHYDRATE-ORDERING
------------------------------------------------------------------------
load_all_for_tool SQL: SELECT ... ORDER BY updated_at_ms DESC LIMIT cap. On rehydrate, callers insert in reverse so the most-recently-touched lands at LRU head (matches C-47-decisions.md D-7 access-order invariant).
REJECT: insertion-order ORDER BY (cold cases would land at LRU head, evicting hot ones on first access).

------------------------------------------------------------------------
D6 -- DEFERRED (Phase C; see D2-phase-c-deferral.md)
------------------------------------------------------------------------
Macro expansion absolute path: would have emitted `::elai_plugins::authz::compose_with_tool_gate_emit`. DEFERRED with Phase C.

------------------------------------------------------------------------
D7 -- DUAL-EMIT-BRIDGE
------------------------------------------------------------------------
KEEP existing gateway_emit_kinds emits @mcp_bridge.rs (the GatewayAuthzSsbEmitter, 4-arg, per-layer attribution rows) AND ADD compose-side McpAuthzEvent emit via the new orchestrator-side OrchestratorMcpAuthzSsbEmitter (5-arg, composed verdict row). The two rows carry different semantics: gateway-emit = per-layer outcome; compose-emit = final composed verdict.
Contract test asserts both fire on a single dispatch_tool_call_with_authz call. Audit DUAL-EMIT before any future "dedup" simplification.
REJECT: drop-gateway-emit (semantically distinct rows; per-layer attribution preserved).

------------------------------------------------------------------------
D8 -- DEFERRED (Phase C / corpus generator)
------------------------------------------------------------------------
DEFERRED with Phase C.

------------------------------------------------------------------------
D9 -- NO-NEW-SSB-VARIANT
------------------------------------------------------------------------
D.1 and D.2 BOTH REUSE existing ToolCalibrationEvent / McpAuthzEvent + their Kind discriminators. ZERO SCHEMA_VERSION bump (stays 195). ZERO golden file regen. NEW env-gate const (ELAI_C47B_CASE_BASE_PERSIST_ENABLE) + NEW path const (CASE_BASE_PERSIST_PATH) are pub-const additions, not enum variants.

------------------------------------------------------------------------
D10 -- DEFERRED (Phase C / dep verify)
------------------------------------------------------------------------
DEFERRED with Phase C. plugins crate adds rusqlite { workspace = true } only -- no new external lock entries (rusqlite 0.39 already pinned at workspace + 7 other crates use it).

------------------------------------------------------------------------
NAMING-COLLISION AVOIDANCE (D.2 Phase B)
------------------------------------------------------------------------
DefaultMcpBridge already has `with_authz_ssb_emitter` @mcp_bridge.rs:540 wiring the 4-arg GatewayAuthzSsbEmitter. New builder for the 5-arg McpAuthzSsbEmitter uses DISTINCT name `with_mcp_authz_ssb_emitter` per explorer NAMING-COLLISION guidance + Tranche D explicit constraint.
New field: `mcp_authz_ssb_emitter: Option<Arc<dyn McpAuthzSsbEmitter>>` defaults to None in both Default::default and ::new.

------------------------------------------------------------------------
FILES TOUCHED
------------------------------------------------------------------------
NEW   .elai_cc/crates/plugins/src/tool_calibration/persist.rs                       (CasePersistStore module)
NEW   .elai_cc/crates/orchestrator/src/mcp_authz_ssb_adapter.rs                     (OrchestratorMcpAuthzSsbEmitter)
NEW   .elai_cc/crates/contracts/tests/c47b_case_base_persist_contract.rs            (D.1 contract test, 5 cases)
NEW   .elai_cc/crates/contracts/tests/c42b_mcp_authz_emit_contract.rs               (D.2 contract test, 5 cases)
MOD   .elai_cc/crates/plugins/src/tool_calibration/mod.rs                           (pub mod persist + pub use CasePersistStore)
MOD   .elai_cc/crates/plugins/src/tool_calibration/registry.rs                      (persist field + with_persist + record_call branch + rehydrate)
MOD   .elai_cc/crates/plugins/Cargo.toml                                            (NEW rusqlite workspace consumer)
MOD   .elai_cc/crates/plugins/src/mcp_bridge.rs                                     (D.2B migration + new field/builder + import update)
MOD   .elai_cc/crates/plugins/src/authz/mod.rs                                      (re-export check confirmed; no edit if already there)
MOD   .elai_cc/crates/orchestrator/src/ssb.rs                                       (NEW 2 pub const: ELAI_C47B_..._ENV + CASE_BASE_PERSIST_PATH)
MOD   .elai_cc/crates/orchestrator/src/lib.rs                                       (pub mod mcp_authz_ssb_adapter + pub use OrchestratorMcpAuthzSsbEmitter)

------------------------------------------------------------------------
VALIDATION TRAIL
------------------------------------------------------------------------
- AggregatedCase Serialize/Deserialize confirmed @case_base.rs:21 (no PartialEq because complexity_score is f32 NaN-not-equal).
- case_base.rs:225/251/277 plain-fns UNCHANGED per explorer recommendation (persist wired through registry record_call closure, NOT plain-fn signatures).
- mcp_bridge.rs:969 confirmed as actual call site (NOT 885 per context-package; explorer drift correction adopted).
- rusqlite 0.39 workspace pin @Cargo.toml:50 confirmed; plugins NOT yet a workspace consumer.
- McpAuthzSsbEmitter trait @compose.rs:42, 5-arg signature locked; production impl count was 0 before this tranche.
- env_gate_truthy @ssb.rs:24461 + blake3_label_prefix @24479 + system_time_ms_i64 @24490 + RUNNER_AGENT_ID/RUNNER_SUPERSTEP @task_runner.rs:129/133 -- all reused from tool_calibration_ssb_adapter.rs pattern.
