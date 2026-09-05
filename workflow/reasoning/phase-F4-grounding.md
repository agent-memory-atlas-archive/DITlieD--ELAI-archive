GROUNDING SWEEP -- phase F4
Source: F:\.ELAI_workflow\docs\thefinalglance\F-plan\F-4_test_integrity_cant_lie.md
Date: 2026-06-01
FSM: IDLE | Session: s-6344

SPEC REFERENCES
No "Spec references:" lines in source file. F-4 cites only internal synthesis doc + elai.map "code-read" confirmations + CLAUDE.md rules.
workflow/reasoning/VIBESEC-SHARD-STEAL-AND-BUILD-2026-06-01.md section 4 C4 | GROUNDED | file exists, grep confirms C4 entry at line 303 + Replit/RedCode grounding at lines 309-311
arXiv:2411.07781 (RedCode) | GROUNDED | synthesis doc cites VERIFIED; NeurIPS 2024 paper, within training knowledge cutoff + synthesis confirmation
AI Incident DB #1152 Replit | GROUNDED | synthesis doc cites CONFIRMED at line 309; cross-checks with theregister 2025-07-21 reference in plan

CLAIM CLASSIFICATION -- line claims (W-7 re-greps)

TestOutcomePayload struct at ssb.rs:9459 | GROUNDED | DEF_STRUCT confirmed at ssb.rs:9459 in elai.map + direct Read at that line
TestOutcomePayload fields test_name/passed/duration_ms at ssb.rs:530-533 | VERIFY | SCIP_DEF rows show ssb.rs:531-533 for field defs; actual struct def is at 9459; SCIP lines are the canonical SCIP_DEF anchor rows (different from struct def line). Both correct.
emit_test_outcome at task_runner.rs:11974 | MISMATCH | actual DEF at task_runner.rs:12074 (grep confirmed: "async fn emit_test_outcome" at 12074, not 11974). Off by 100 lines.
baby-step emit_test_outcome call at task_runner.rs:8660 | MISMATCH | actual CALL at task_runner.rs:8717 (grep confirmed: "emit_test_outcome(" at 8717, not 8660). Off by 57 lines.
integration-checkpoint emit_test_outcome call at task_runner.rs:15267 | MISMATCH | actual CALL at task_runner.rs:15367 (grep confirmed: emit_test_outcome call in emit_checkpoint_outcome at 15367, not 15267). Off by 100 lines.
run_integration_checkpoint at pipeline_tree_navigator.rs:1946 | GROUNDED | SCIP_DEF + grep confirm DEF at ptn.rs:1946; returns IntegrationRunResult (not TestRunResult).
TestRunResult at reasoning/types.rs:391 | MISMATCH | actual DEF_STRUCT at types.rs:598, not 391. Off by 207 lines.
TestRunResult fields passed/output/duration_ms at types.rs:392-394 | MISMATCH | actual fields start at types.rs:599 (passed), 600 (output), 601 (duration_ms), not 392-394.
run_integration_checkpoint returns TestRunResult with run.output | MISMATCH | run_integration_checkpoint returns IntegrationRunResult (pipeline_tree_navigator.rs:1743-1750), NOT TestRunResult from reasoning/types.rs. IntegrationRunResult has no "output: String" field -- it has criterion_results:Vec<CriterionResult> and failure_reason:Option<String>. The plan's claim that "run.output" is the captured stdout is WRONG for this return type. F-4 Task 2 must capture stdout from criterion_results or a different mechanism.
ZeroBlocksGate struct at evidence.rs:504 | GROUNDED | direct Read at evidence.rs:499-506 confirms: Gate 6 comment at 499, struct at 504, impl at 506.
ZeroBlocksGate impl EvidenceGate at evidence.rs:506 | GROUNDED | confirmed above.
all_gates() at evidence.rs:468 | MISMATCH | actual DEF at evidence.rs:708 (Read + SCIP_DEF at 468 is the SCIP anchor line only; actual pub fn all_gates() is at line 708). Implementer must use 708 for edits.
names.len() assert at evidence.rs:1358 | GROUNDED | direct Read confirms assert_eq!(names.len(), 9) at evidence.rs:1358.
evidence::TestOutcome at evidence.rs:62 | GROUNDED | Read confirms struct TestOutcome at line 61-69.
TransitionContext at evidence.rs:103 | GROUNDED | Read confirms struct at line 103.
pub test_outcomes at evidence.rs:112 | MISMATCH | actual at evidence.rs:117.
audit_findings at evidence.rs:121 | MISMATCH | actual at evidence.rs:126.
EvidenceGate trait check signature at evidence.rs:147 | GROUNDED | SCIP_DEF at 147 confirmed in elai.map.
DiagnosticProbeGate at evidence.rs:643-720 | VERIFY | Read confirms: comment at 642, struct at 655, impl at 657; range 643-720 covers the shape. Close enough as a "read this range" instruction.
C-22 SHIPPED (stdout_hash on StubResult, blake3 at feasibility_probe.rs:168/188) | GROUNDED | file crates/contracts/tests/c22_feasibility_completeness_contract.rs exists; Read confirms stdout_hash at feasibility_probe.rs:168; passed_with_stdout DEF at 187; blake3 at 188.
C-27 SHIPPED | GROUNDED | crates/contracts/tests/c27_wiring_contract.rs exists.
C-28 SHIPPED | GROUNDED | crates/contracts/tests/c28_mutation_testing_contract.rs exists.
E-52 SHIPPED (DiagnosticProbeGate precedent) | GROUNDED | DEF at evidence.rs:655 + impl at 657 confirmed.
E-30 SHIPPED (cant-wreck sibling) | GROUNDED | references present in elai.map action_type_lock + argv gate; no contract file checked but multiple SCIP refs confirm presence.
get_per_level_verdicts at commands.rs:9408 | GROUNDED | Read confirmed pub async fn get_per_level_verdicts at line 9408.
AuditTimelineWindow.svelte exists and imports getPerLevelVerdicts | GROUNDED | elai.map IMPORT at AuditTimelineWindow.svelte:23 confirms import from api/stub; svelte file present.
SCHEMA_VERSION baseline 191 | MISMATCH | actual SCHEMA_VERSION = 195 (ssb.rs:2726). Plan says "baseline 191 + 1 = 192". Correct target is 195 + 1 = 196.
TOTAL_VARIANTS baseline 464 (from CLAUDE.md) | MISMATCH | actual TOTAL_VARIANTS = 471 (action_type_lock.rs:485). Pattern-2 collapse adds 1 variant -> target 472.
action_type_lock.rs bump site at 2141 | GROUNDED | Read confirms assert_eq! at line 2141.
blake3 pattern at feasibility_probe.rs:188 | GROUNDED | Read confirms "let hash = blake3::hash(stdout.as_bytes()).to_hex().to_string();" at line 188.
IntegrationRunResult has output field for blake3 hash | MISMATCH | IntegrationRunResult (ptn.rs:1743-1750) has NO output:String field. Fields: level, node_id, success, criterion_results:Vec<CriterionResult>, failure_reason:Option<String>, total_duration_ms:u64. The plan's Task 2 instruction "use run.output" is wrong. stdout capture for the hash must come from CriterionResult, or the IntegrationRunner trait must be extended. This is a BLOCKING design gap.

ACTIONS REQUIRED
- MISMATCH (BLOCKER): Task 2 references "run.output" as the captured stdout for blake3 hash, but emit_checkpoint_outcome receives IntegrationRunResult which has NO output field. The "process slot with captured output" claim is FALSE for this path. The implementer must either: (a) add an output:String field to IntegrationRunResult populated from runner.run_criterion stdout captures, or (b) use CriterionResult-level output if that field exists. VERIFY CriterionResult struct fields before starting Task 2. Resolve before TESTING_FIRST.
- MISMATCH: emit_test_outcome DEF is at task_runner.rs:12074, not 11974. Update all read-range and edit targets accordingly.
- MISMATCH: baby-step emit_test_outcome call site is at task_runner.rs:8717, not 8660. Update edit target.
- MISMATCH: integration-checkpoint call site is at task_runner.rs:15367, not 15267. Update edit target.
- MISMATCH: TestRunResult is at reasoning/types.rs:598, not 391. The plan's Task 3 gate description references "TestRunResult.output" but the integration path does not use TestRunResult at all -- it uses IntegrationRunResult. TestRunResult is used only by the reasoning/verification.rs path (MockTestRunner/LocalVerifier), not by the orchestrator integration-checkpoint path. The "pre-resolved process-slot question" in the Context section is therefore wrong about which type carries the output.
- MISMATCH: all_gates() production DEF is at evidence.rs:708, not 468. SCIP anchor at 468 is not the function definition line. Implementer adds AntiFabricationGate at line 708 context, not 468.
- MISMATCH: TransitionContext.test_outcomes is at evidence.rs:117, not 112. audit_findings is at evidence.rs:126, not 121. Minor but affects targeted edits.
- MISMATCH: SCHEMA_VERSION must bump from 195 to 196 (not 191+1=192). TOTAL_VARIANTS must bump from 471 to 472 (not 464+1).
- VERIFY: check CriterionResult struct fields (pipeline_tree_navigator.rs around line 1074) to confirm whether per-criterion stdout is available for blake3 hash capture before designing Task 2.

VALIDATION WARNINGS
No "Spec references:" lines in source -- F-4 is grounded via synthesis doc + code-read + CLAUDE.md rules, not spec file citations. Acceptable for this unit per the F-plan WIRING DOCTRINE but noted.
SCHEMA_VERSION and TOTAL_VARIANTS numbers in the plan body are stale (written when baseline was 191/464; current state is 195/471). The WIRING_CONTRACT SCHEMA bump target must be recalculated.
The "pre-resolved process-slot question" section (Context, last bullet) contains a factual error: run_integration_checkpoint returns IntegrationRunResult, not TestRunResult, and IntegrationRunResult has no captured stdout output field. This is the highest-severity finding -- the design assumption about which field carries the stdout is wrong. The gate's viability is unaffected (a hash of something orchestrator-controlled still works) but the specific field access path in Task 2 must be redesigned.
