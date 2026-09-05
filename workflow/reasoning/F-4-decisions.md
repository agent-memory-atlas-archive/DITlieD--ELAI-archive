F-4 test-integrity cant-lie gate decisions (session s-6344, Unit B)

What I will implement
Bind every claimed test outcome to an orchestrator-captured artifact so a fabricated PASS is
detectable. Add stdout_hash (blake3) to TestOutcomePayload, capture it on the integration-checkpoint
path, install an AntiFabricationGate on the AUDITING->MEMORIZING edge that BLOCKs when an integration
test row claims passed=true with stdout_hash=None (the fabrication signature), and surface it in the
AuditTimelineWindow. New SSB TestIntegrityEvent Pattern-2 variant.

Two design blockers resolved (the research flagged both; do NOT skip)
BLOCKER-1: IntegrationRunResult (pipeline_tree_navigator.rs:1743) has NO output field today (only
  level, node_id, success, criterion_results, failure_reason, total_duration_ms). The plan's "run.output"
  does not compile. RESOLUTION: add output_capture: Option<String> to IntegrationRunResult AND
  CriterionResult; populate output_capture in run_integration_checkpoint from the concatenated criterion
  reason text. pipeline_tree_navigator.rs is ADDED to Files-touched (it was missing from the plan).
  emit_checkpoint_outcome computes stdout_hash = Some(blake3::hash(output_capture.unwrap_or("")).to_hex())
  UNCONDITIONALLY so "empty-but-captured" differs from "never-captured" (the C-22 feasibility_probe.rs
  pattern, but unconditional). blake3 only (never blake2/sha2).
BLOCKER-2 (the crux wiring): the AUDITING->MEMORIZING EvidenceGate path (check_transition via all_gates)
  is NOT called in production. fsm.rs (~:888) uses DeterministicAuditGate.last_audit_result.block_count()
  only. An AntiFabricationGate appended to all_gates() would NEVER fire. RESOLUTION: ADD (not replace) an
  explicit AntiFabricationGate check at the AUDITING->MEMORIZING boundary, AFTER the existing block_count
  check passes. Build a TransitionContext carrying integration_test_rows (from the SSB TestOutcome rows
  written this superstep) and run the gate; if Block, HALT the transition and emit a TestIntegrityEvent
  {FabricationBlocked} row. The existing block_count flow is untouched. The gate default-DEGRADES (ALLOWS)
  when integration_test_rows is None, so steps without integration checkpoints are unaffected (no
  regression). It BLOCKs only on passed==true && stdout_hash.is_none().

Key decisions (grounded)
- emit_test_outcome (task_runner.rs ~:12074, RE-GREP) gains a stdout_hash: Option<String> param; the
  baby-step call site (~:8717) passes None; the integration-checkpoint site (~:15367) passes Some(hash).
- TestOutcomePayload (ssb.rs ~:9459) gains stdout_hash: Option<String> #[serde(default)].
- SSB: ONE SsbActionType::TestIntegrityEvent Pattern-2 variant + TestIntegrityEventKind { Verified,
  FabricationBlocked }; TestIntegrityEventPayload { test_name, passed, stdout_hash: Option<String>,
  kind }; as_sql_str: Verified->"test_integrity_verified", FabricationBlocked->"anti_fabrication_block".
- SCHEMA_VERSION 196->197 (post-Unit-A; RE-GREP the live value, Unit A bumped 195->196). TOTAL_VARIANTS
  473->474 (post-Unit-A; RE-GREP). Follow the ssb.rs lockstep checklist + golden + activity_entry.
- evidence.rs: TransitionContext.integration_test_rows: Option<Vec<IntegrationTestRowSummary>>
  #[serde(default)]; IntegrationTestRowSummary struct near TestOutcome; AntiFabricationGate struct +
  EvidenceGate impl (transition=(Auditing,Memorizing), name="anti_fabrication"); append as slot 10 in
  all_gates(); bump the gate-name-count test (9->10) + gates_for_transition(Auditing,Memorizing) (1->2).
- Frontend pair (W-4): get_test_integrity_events Tauri command (commands.rs, sibling get_per_level_verdicts;
  register in src-tauri lib.rs invoke_handler) + TestIntegrityEventJson + stub.ts getTestIntegrityEvents +
  types.ts interface + AuditTimelineWindow.svelte fabricationBlocks red badge. audit_timeline window is
  already mounted (WindowHost.svelte) - no new window type.
- Plan consumes_symbol error: crates/reasoning/src/types.rs::TestRunResult is the WRONG type (the
  integration-checkpoint path uses IntegrationRunResult). Drop TestRunResult from the contract.

Default-ON + production-caller mandate (Unit-A lesson)
The AntiFabricationGate is ON by default (NO default-off env gate; W-2). It is WIRED at the production
fsm.rs/task_runner AUDITING->MEMORIZING boundary (W-3), not left dormant. The producer (emit_test_outcome
with stdout_hash) is called on the live integration-checkpoint path (W-8 symmetry).

Expected test outcome
cargo check --workspace GREEN. Contract test f4_test_integrity_contract (stdout_hash round-trip + gate
behavior). Fixture harness f4_fabrication_fixture_harness injects an integration test row with passed=true
+ stdout_hash=None -> AntiFabricationGate BLOCKs the AUDITING->MEMORIZING edge + emits anti_fabrication_block;
a clean run (stdout_hash=Some) emits test_integrity_verified. action_type_lock +1. evidence.rs gate tests
updated.

Unknowns / risks
- The exact AUDITING->MEMORIZING call site (fsm.rs:888 vs a task_runner boundary) - RE-GREP; choose the
  one that has the SSB rows + the TransitionContext builder in scope. Do NOT break the existing transition.
- 6 emit_checkpoint_outcome call sites exist; the blake3 capture in the fn body covers all.
- IntegrationRunResult struct-literal construction sites (tests) need the new field (use ..Default or add it).
- Default-degrade on None is mandatory to avoid blocking every non-integration AUDITING->MEMORIZING edge.
