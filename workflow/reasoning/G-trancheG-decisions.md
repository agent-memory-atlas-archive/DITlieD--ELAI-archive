Tranche G decisions -- orchestrate s-22600 audit followup wires

Date: 2026-06-01
Session: s-22600
Context source: workflow/sessions/s-22600/context-package-trancheG.md
Scope: 4 small follow-up wires for substrate Tranches B + C + F shipped earlier in s-22600. No schema bump. No new dependencies.

Items shipped

G.1 hook_prompt::to_decision_or_err adapter
  File: .elai_cc/crates/orchestrator/src/architecture/hook_prompt.rs
  Shape: pub fn to_decision_or_err(Result<PromptDecision, PromptHandlerError>) -> Result<PromptDecision, PromptHandlerError>
  Semantic: `GateDark` downgrades to `Ok(Allow)` (fail-open under the wire-dark posture per the module doc at L213-216); every other `PromptHandlerError` variant propagates unchanged so audit logs preserve the failure cause. Intentionally NOT named `unwrap_or_allow` because that name would imply Runtime / InvalidDecisionFormat / UnknownVariant / TimedOut also map to Allow -- they MUST NOT (fail-closed on real errors).
  Tests: 3 in-module unit tests (`to_decision_or_err_maps_gate_dark_to_allow`, `to_decision_or_err_propagates_real_errors`, `to_decision_or_err_passes_ok_decisions_through`).
  Consumer note: the F.2 mcp_bridge.rs site is NOT a consumer. F.2 wires `llm_guard::pre_mcp_call_decision` which returns `Result<GuardDecision, GuardError>` -- a different error type with no `GateDark` variant. The two paths are intentionally parallel per Tranche F DECISION-T-F-5; consolidation deferred. G.1 ships the adapter against the next `hook_prompt::prompt_decision` wirer (zero production callers today; grep confirmed).

G.2 edit_checkpoints next_seq race fix (Option B: nanosec suffix)
  Files: .elai_cc/crates/orchestrator/src/architecture/edit_checkpoints.rs (struct field + helper + 4 fn updates), .elai_cc/crates/contracts/tests/e25_edit_checkpoints_parallel_contract.rs (new contract test)
  Full reasoning: workflow/reasoning/E-25-trancheC-edit-checkpoints-nanosec-followup.md
  Summary: on-disk seq directory name changes from `{seq:06}/{blake3_prefix}/...` to `{seq:06}_{nanos:09}/{blake3_prefix}/...`. `EditCheckpoint` gains a `nanos: u64` field. `parse_seq_dirname` reads both shapes (legacy `nanos=0`). Eliminates the concurrent-writer collision on `(seq, blake3_prefix)`. ZERO new deps, ZERO schema bump.
  Tests: 4 in-module + 1 contract.

G.3 CLI startup .with_symbol_graph_adapter wire
  File: .elai_cc/crates/cli/src/runtime_builder.rs (L668 area, replacing the ProductionAgentRpcServer::new chain)
  Wire: `SymbolGraphCacheAdapter::new(Arc<Mutex<orchestrator::symbol_graph::SymbolGraphCache>>, 500ms)`. Graceful degrade: if `.claude/tools/elai_graph.json` is absent OR `SymbolGraphCache::load` fails, the adapter is skipped and pre-B.4 behaviour (typed-empty responses from TOOL_SYMBOL_DSL_QUERY + TOOL_SYMBOL_DATAFLOW_IMPACT) is preserved. No new env gate -- the JSON file is the gate.
  Cache-choice deviation from context-package: context-package recommended the SQLite-rooted `elai_knowledge::symbol_graph::cache::SymbolGraphCache` keyed off `elai_core::paths::dkg_db_path`. That was wrong: `SymbolGraphCacheAdapter::new` takes the ORCHESTRATOR's JSON-rooted `elai_orchestrator::symbol_graph::SymbolGraphCache` (E-40), per the type signature at orchestrator/src/symbol_graph/mod.rs:110. Mirrored the cache loader pattern from `cli/src/scip_cmd.rs:61` (DEFAULT_GRAPH_PATH = `.claude/tools/elai_graph.json`).
  Wire-presence verification: grep `with_symbol_graph_adapter` in `.elai_cc/crates/cli/src/runtime_builder.rs` (one hit, line ~709). No unit test added per context-package L346 ("a dedicated wire-presence test would require booting the full CLI runtime which is out of unit-test scope").

G.4 (Executing, Auditing) TransitionContext build + DiagnosticProbeGate fire
  File: .elai_cc/crates/orchestrator/src/task_runner.rs (insertion at L8473-ish, after `run_file_read_dispatch` and before the `else if !outcome_passed` branch). Also renamed the binding at L8418 from `_diagnostic_probe_outcome` to `diagnostic_probe_outcome` (now consumed) and updated the Tranche B-3 comment block from "separate follow-up" to "Tranche G G.4 consumes this binding immediately below".
  Contract test: .elai_cc/crates/contracts/tests/e52_gate_fire_contract.rs (2 tests: gate-registry-shape lock + DiagnosticProbeGate.check None-fast-path).
  Pattern: mirrors PLANNING -> TESTING_FIRST gate-fire at L5711-5783 (TransitionContext struct-update, gates_for_transition, per-gate check_result + EvidenceGateEvaluated SSB emit, block-messages aggregation, emit_finding + tracing::error on block). Non-fatal-emit per DECISION-T-F-12 -- the gate-fire emits the SSB rows so the auditor sees the block, but does NOT abort drive_task. The FSM driver at a higher level acts on the EvidenceGateEvaluated + Finding rows.
  Wire-dark posture preserved: indirectly gated by `ELAI_LSP_DIAGNOSTIC_PROBE_ENABLE` (set at the producer side inside `run_validate_dispatch`); when OFF, `diagnostic_probe_outcome` is None and the `if let Some(outcome) = ...` branch never fires. Zero behaviour change for the default-off posture.

Verification

cargo check --workspace --tests 2026-06-01: zero errors (only pre-existing warnings).
Per-item scoped checks (all GREEN):
  G.1: cargo check -p elai-orchestrator --tests
  G.2: cargo check -p elai-orchestrator --tests + cargo check -p elai_cc_contracts --tests
  G.3: cargo check -p elai-cli --tests
  G.4: cargo check -p elai-orchestrator --tests + cargo check -p elai_cc_contracts --tests

Anti-stale compliance

- blake3 only (G.2 reuses existing blake3 hash, no blake2/sha2).
- thiserror only (G.1/G.2 reuse existing PromptHandlerError/EditCheckpointError derives).
- Named constants (G.2 introduces NANOS_DIGITS = 9; no magic number).
- No .unwrap()/.expect() in production paths (G.1 tests use .expect with reason; G.2 prod paths use .unwrap_or(0) on the SystemTime call; G.3 prod path uses tracing::warn + None on errors; G.4 prod path uses .unwrap_or(std::time::Duration::ZERO) + i64::try_from(...).unwrap_or(i64::MAX)).
- u64::from(d.subsec_nanos()) instead of `as` cast for the G.2 nanos extraction (Anti-Stale-Training narrowing rule -- subsec_nanos() returns u32 which fits losslessly in u64 so `u64::from` is the correct primitive).
- tracing only (G.3/G.4 emit via tracing::warn / tracing::error / tracing::debug, no log crate).

Schema impact

ZERO. SCHEMA_VERSION 195 stable. No new SsbActionType, no new ClaudeCodeArchitectureEventKind arm. G.2 adds `details_json.nanos` (Pattern-2 free-form, additive).

Total: 4 production wires, 8 new tests (3 unit for G.1, 4 unit + 1 contract for G.2, 2 contract for G.4), 1 contract file added (e25_edit_checkpoints_parallel_contract.rs), 1 contract file added (e52_gate_fire_contract.rs), 2 reasoning docs (this + E-25-trancheC-edit-checkpoints-nanosec-followup.md).
