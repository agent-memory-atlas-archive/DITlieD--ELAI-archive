F-3 phase 1 Tool Filter data-plane decisions (session s-6344, Unit C)

What I will implement (phase 1 ONLY)
Lock the researcher's tool set to a per-task minimal subset BEFORE any untrusted byte is observed
(CaMeL/AgentDojo Tool-Filter pattern), enforced fail-closed at the FSM tool gate + the MCP plugin gate,
with a deterministic AgentDojo-style fixture measuring ELAI's own injection ASR before/after. Phase 2
(full CaMeL SymbolicVar) is NOT in scope (conditional on token-overhead measurement).

Key decisions (grounded)
- CRATE BOUNDARY (the architectural constraint): crates/plugins has NO dependency on crates/orchestrator
  (Cargo.toml: only elai-core + elai-rules). So the committed-set check cannot import orchestrator types
  directly. RESOLUTION (E18 acyclic-trait pattern): define a CommittedSetChecker trait IN crates/plugins
  (object-safe; check(fsm_state, tool_name) -> Result<(), Denial>), implement the adapter in
  crates/orchestrator, inject it into mcp::gate::ToolGate at construction. Fail-closed: if a tool is not
  in the frozen committed set, DENY. If no checker is injected (None), preserve existing behavior (no
  committed-set check) so existing tool_gate tests stay green.
- capability_flow.rs (NEW, crates/orchestrator, ~300 LOC): CommittedToolSet { allowed_categories,
  commit_hash (blake3), committed_at }, commit_minimal_set(role) (intersect the researcher's legitimate
  scope from ipc_gateway::allowed_grounding_methods with ToolCategory::ALL), check_against_commitment(set,
  tool_name) -> Result<(), CapabilityFlowDenial>, UntrustedTaint, taint_from_verdict(ClassifierVerdict)
  bridging E-37 (use the SYNC classify_with_regex fast path for determinism on the gate hot path). REUSE
  the existing tool_filter.rs (ToolCategory::classify @ :86, ToolCategory::ALL @ :61, FsmState::
  check_tool_allowed @ :370) - do NOT duplicate.
- COMMIT POINT (W-3 producer): emit tool_filter_committed from the WebResearchOrchestrator research_loop
  call site (~:249) BEFORE formulate_query is called (the ResearchNode context is available there;
  preserves formulate_query's &self). Store the CommittedToolSet via a new
  committed_tool_set: Arc<tokio::sync::Mutex<Option<CommittedToolSet>>> field on WebResearchOrchestrator
  (init in new()). The commitment is frozen BEFORE fetch_and_extract (the first untrusted-byte boundary).
- ENFORCEMENT (W-3, two production gates): (1) Fsm::check_tool_allowed (orchestrator, ~:1158) - after the
  existing FsmState::check_tool_allowed passes, consult the active CommittedToolSet; on denial emit
  ToolDenied (REUSE the existing variant, carry commit_hash in payload) + an untrusted_var_quarantined
  row. (2) mcp::gate::check_tool_allowed (plugins, :127) - after the existing config check, call the
  injected CommittedSetChecker.
- SSB: ONE SsbActionType::ToolFilterEvent Pattern-2 variant + ToolFilterEventKind { Committed,
  UntrustedQuarantined }; ToolFilterPayload { commit_hash, allowed_categories, denied_tool: Option,
  taint_source: Option, task_id }; as_sql_str: Committed->tool_filter_committed, UntrustedQuarantined->
  untrusted_var_quarantined. ToolDenied is REUSED (not new). capability_flow_blocked is RESERVED for
  Tier-3 and is NOT added (W-8 forbids a producer-less variant). SCHEMA 197->198 (post-Unit-B; RE-GREP),
  TOTAL_VARIANTS 474->475.
- CLI (W-3 entrypoint): elai safety tool-filter --fixture <name> -> SafetyAction::ToolFilter; run_tool_filter
  commits a minimal set, replays the fixture's injected-instruction sequence, asserts the out-of-set tool
  call is DENIED, emits the SSB rows, renders a JSON report { commit_hash (blake3, 64-hex), denied_tool,
  asr_before, asr_after }. Deterministic (no model needed - the fixture is a recorded injection sequence).
- Frontend pair (W-4): extend the EXISTING ToolGateInspector.svelte with a Tool Filter / Data-plane
  section (committed allow-set + blake3 commit_hash chip + ASR before/after tile); extend
  commands_c35.rs::list_tool_gate_rows to query tool_filter_committed + untrusted_var_quarantined, and
  ToolGateRowDto with 4 new Option fields (commit_hash, allowed_categories, denied_tool, taint_source)
  ALL with #[serde(default)] (backward compat for existing tool_denied/tool_lazy rows).
- DEFAULT-ON (W-2): the Tool Filter is active by default (no default-off env gate). A kill-switch env may
  DISABLE it, default enabled.

Default-ON + production-caller mandate (Unit-A/B lesson)
The commit producer (research_loop) and both enforcement gates are on live production paths. No dormant
producer, no default-off terminal state.

Scope boundaries / follow-ups (documented, not stubs)
- Paper arXiv:2406.13352 (AgentDojo Tool Filter) ingest + a >=20-fixture corpus + the published ASR
  before/after number are FOLLOW-UPS (the synthesis doc TABLE-VERIFIED the 57.7->6.84% / 73% utility
  numbers; phase 1 makes NO published claim - it ships the mechanism + 1-3 representative fixtures + the
  deterministic ASR harness that measures ELAI's own ASR on the fixture(s)).
- Phase 2 (full CaMeL SymbolicVar, IPC schema change) is explicitly out of scope (conditional gate).

Expected test outcome
cargo check --workspace GREEN. Contract f3_tool_filter_dataplane_contract: commit_minimal_set freezes the
set; check_against_commitment DENIES an out-of-set tool; the fixture replay shows asr_after < asr_before;
ToolFilterEvent round-trips. observable_effect: cargo run -p elai-cli -- safety tool-filter --fixture
agentdojo_inject_fs_write -> tool_filter_committed + untrusted_var_quarantined rows + JSON with a 64-hex
commit_hash + denied fs.write + asr_after < asr_before.

Unknowns / risks
- gate.rs trait-seam object-safety + the injection point at ToolGate construction (read gate.rs imports +
  the ToolGate constructor first; confirm no circular dep is introduced).
- classify_injection is async; use the sync classify_with_regex variant on the gate hot path.
- ToolGateRowDto new fields MUST be Option + #[serde(default)] or existing-row deserialization breaks.
- The None-checker default must keep existing plugins/tests/tool_gate.rs green.
