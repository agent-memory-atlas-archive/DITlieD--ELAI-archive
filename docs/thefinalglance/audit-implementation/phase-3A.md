PHASE 3A AUDIT — FULL FSM (INTERACTIVE MODE)

Scope: 3A.1 through 3A.5. Target crate: .elai_cc/crates/orchestrator/.
Auditor posture: hostile. Verdict per sub-step, evidence cited from source.
Files under audit:
- F:/.ELAI_workflow/.elai_cc/crates/orchestrator/src/fsm.rs (1442 lines)
- F:/.ELAI_workflow/.elai_cc/crates/orchestrator/src/evidence.rs (1182 lines)
- F:/.ELAI_workflow/.elai_cc/crates/orchestrator/src/gate.rs (1076 lines)
- F:/.ELAI_workflow/.elai_cc/crates/orchestrator/src/tool_filter.rs
- F:/.ELAI_workflow/.elai_cc/crates/cli/src/main.rs (CliInteractiveGate + persist path wiring)

================================================================================
3A.1 FSM STATE MACHINE — VERDICT: PASS (with scope deviation)
================================================================================

STATE ENUM (fsm.rs:24-38)
Spec requires 9 states: IDLE, PLANNING, FEASIBILITY_PROBE, TESTING_FIRST, EXECUTING, AUDITING, MEMORIZING, COMPLETE, CONFIGURING.
Implementation defines 10 variants — all 9 spec states plus `Researching` (Oracle mode, pipeline-modes-design.txt §2).
Deviation justification: Researching is additive, not replacing any required state. File comment on line 21 explicitly cites pipeline-modes-design.txt §2 as source. Constant `FsmState::ALL` at line 42-53 exposes all 10.
Evidence grep: `grep -c "FsmState::" fsm.rs` > 9 (ALL array alone contains 10).

TRANSITION VALIDATION MATRIX (fsm.rs:57-77)
Spec checklist vs implementation:
- IDLE -> PLANNING | CONFIGURING : PRESENT (line 59, plus Researching additive).
- PLANNING -> FEASIBILITY_PROBE | TESTING_FIRST : PRESENT (line 60).
- FEASIBILITY_PROBE -> PLANNING | TESTING_FIRST : PRESENT (line 61).
- TESTING_FIRST -> EXECUTING : PRESENT (line 62).
- EXECUTING -> AUDITING | PLANNING (replan) : PRESENT (line 63).
- AUDITING -> MEMORIZING | EXECUTING | PLANNING : PRESENT (line 64-68). Covers the two SPEC_UPDATE_NEEDED edges (AUDITING->EXECUTING, AUDITING->PLANNING) flagged in the phase-3 summary.
- MEMORIZING -> PLANNING | COMPLETE : PRESENT (line 69).
- COMPLETE -> IDLE : PRESENT (line 70).
- CONFIGURING -> IDLE : PRESENT (line 71).
Exhaustive test: tests/test_transition_matrix_exhaustive (fsm.rs:825+) enumerates 15 valid edges (including Researching) and asserts all other 81 pairs return InvalidTransition.
Verdict: matrix complete. SPEC_UPDATE_NEEDED flag from phase-3-summary is resolved — both AUDITING->EXECUTING and AUDITING->PLANNING are implemented.

TransitionRecord struct (fsm.rs:161-167): carries source, target, phase_label, timestamp_ms, evidence: Vec<EvidenceArtifact>. Matches spec line 29.

ReplanRequest (fsm.rs:174-190):
- ReplanSeverity::{Minor, Major} enum (line 175-180).
- submit_replan() enforces max-1 per (agent_id, phase_label) at line 658-667.
- Second request returns FsmError::ReplanLimitExceeded (line 663). Spec ADVERSARIAL case passes.
- Replan log cleared on phase boundary when source is Memorizing or Complete (line 623-625). This matches "Max 1 per agent per phase" spec requirement.

token_budget_remaining (fsm.rs:483-485): present, returns usize as spec requires (line 30 of spec).

PERSISTENCE TO MSGPACK (fsm.rs:282-294, 728-750)
- PersistedFsmState struct contains state, phase_label, token_budget, replan_log, transition_history.
- persist_state() encodes via rmp_serde::to_vec (line 731) and writes atomically: write to `.msgpack.tmp` then rename (line 742-747). Crash-safe.
- Load path at fsm.rs:431-464 decodes msgpack and returns FsmError::CorruptState on decode failure (line 444). Spec ADVERSARIAL "corrupt msgpack returns Err(CorruptState)" satisfied.
- Test: test_persistence_roundtrip (fsm.rs:1047+) writes, loads, asserts identity.
- CLI wires persistence at main.rs:3525, 3597, 7109: path = sessions/{sid}/.fsm-state.msgpack (matches spec line 31).

CONCURRENT TRANSITIONS
Fsm::transition_with_metrics uses `inner.try_lock()` (fsm.rs:536-538). Contention returns FsmError::TransitionConflict (line 538). Spec ADVERSARIAL "concurrent transitions return Err(TransitionConflict)" satisfied. Test: test_concurrent_transitions uses tokio::spawn with two simultaneous callers (grep shows arbitrary_state proptest + concurrent test around line 1117).

ERROR TYPES (fsm.rs:198-233)
FsmError covers: InvalidTransition, CorruptState, ReplanLimitExceeded, TransitionConflict, PersistenceError, ToolBlockedByState, AuditingBlocked. All spec adversarial cases mapped.

Verdict 3A.1: PASS. All 9 required states + additive Researching. Full transition matrix. ReplanRequest semantics enforced. msgpack round-trip with atomic writes and crash recovery. token_budget_remaining present.

================================================================================
3A.2 PHASE-AWARE TOOL/RULE LOADING — VERDICT: PASS
================================================================================

File: tool_filter.rs:227-329 (allowed_tool_categories) and 334-366 (active_rule_categories).
Spec table vs implementation:
- PLANNING: no Edit, no Bash. Implementation (line 235-243): Read, Web, Planning, Knowledge, Meta, FsmInternal, Mcp. No Edit/Bash. PASS.
- FEASIBILITY_PROBE: StubWrite + Bash + Web, no Edit/Git. Implementation (line 247-254): Read, StubWrite, Bash, Web, Meta, FsmInternal. Git absent as required. PASS.
- TESTING_FIRST: Testing + Planning + Read, no Edit/Bash. Implementation (line 258-264): Read, Planning, Testing, Meta, FsmInternal. PASS.
- EXECUTING: Edit + Bash + Testing, no Planning tools. Implementation (line 268-279): Read, Edit, Bash, Web, Testing, Git, Meta, FsmInternal, TaskManagement, Mcp. Planning absent as required. PASS.
- AUDITING: Validation + Testing + Bash (test runner), no Edit/Planning. Implementation (line 283-290): Read, Bash, Validation, Testing, Meta, FsmInternal. PASS.
- MEMORIZING: memory-bank only. Implementation (line 294-299): Read, Memory, Meta, FsmInternal. PASS.
- CONFIGURING: config paths only, no code edit/bash. Implementation (line 311-316): Read, ConfigOnly, Meta, FsmInternal. PASS.
- IDLE: all tools. Implementation (line 231): &ToolCategory::ALL. PASS.
- COMPLETE: read-only. Implementation (line 303-307): Read, Meta, FsmInternal. PASS. (spec marked [UNGROUNDED assumed]; implementation acknowledges with comment "UNGROUNDED — assumed from state semantics" at line 302.)

Enforcement point: Fsm::check_tool_allowed (fsm.rs:695-704) returns FsmError::ToolBlockedByState when category is not in the allowed set for the current state. FsmState::check_tool_allowed (tool_filter.rs:370-377) performs the inner match.

Test coverage: tool_filter.rs lines 454-515+ assert blocks for each state (test_planning_blocks_edit, test_executing_blocks_planning, test_auditing_blocks_edit, test_idle_allows_all, etc.). Coverage is explicit per spec DETERMINISTIC_CHECK.

GROUNDING NOTE: COMPLETE and IDLE tool sets were flagged UNGROUNDED in the phase-3 summary (3A.2 GROUNDING line: "COMPLETE/IDLE tool restrictions UNGROUNDED"). Implementation does not retroactively ground them beyond comment acknowledgement. Not a blocker.

Verdict 3A.2: PASS. 9-state tool matrix wired, rule matrix wired, enforcement via typed error.

================================================================================
3A.3 EVIDENCE GATE PROTOCOL — VERDICT: PASS
================================================================================

File: evidence.rs.
EvidenceGate trait: evidence.rs:138-148. Methods: name(), transition(), check(&TransitionContext) -> Result<(), EvidenceError>.
EvidenceError: evidence.rs:21-25. Fields: gate_name, missing_artifacts: Vec<String>, suggestion: String. Matches spec L165-166 exactly.

Seven gates registered (evidence.rs:468-478 all_gates()):
1. KnowledgeReadinessGate (line 157-188): IDLE -> PLANNING. Rejects Unavailable, accepts Ready/Degraded. Spec L626-629 honored.
2. GroundingSweepGate (line 196-225): PLANNING -> TESTING_FIRST. Rejects if any ungrounded claim. Spec L631 honored.
3. TestFailReasonsGate (line 233-275): TESTING_FIRST -> EXECUTING. Rejects on syntax-error tests. Spec L632 honored.
4. TestFlippedGate (line 283-318): EXECUTING -> AUDITING. Requires at least one FAIL->PASS flip. Spec L633 honored.
5. ReplanFastTrackGate (line 327-370): EXECUTING -> PLANNING. MINOR requires auditor approval; MAJOR defers to InteractiveGate. Spec L634-638 honored.
6. ZeroBlocksGate (line 378-415): AUDITING -> MEMORIZING. Rejects on any Block severity. Spec L639 honored.
7. DkgChangelogGate (line 424-461): MEMORIZING -> PLANNING / COMPLETE. Requires dkg_reflects_new_code && changelog_has_entry. Spec L640 honored.

Count check: `grep -c "impl EvidenceGate for" evidence.rs` = 7. Matches DETERMINISTIC_CHECK exactly.

gates_for_transition (evidence.rs:483) maps every guarded transition to its gate set. check_transition (evidence.rs:502-513) iterates gates and returns the first error. Transitions with no gates (PLANNING->FEASIBILITY_PROBE, FEASIBILITY_PROBE->*, CONFIGURING->IDLE, COMPLETE->IDLE, all RESEARCHING edges) return empty vec — matches spec "FEASIBILITY_PROBE transitions ungated in Phase 3A" (L163).

TransitionContext (evidence.rs:98-129): single struct holding all evidence (knowledge_readiness, ungrounded_claims, test_outcomes, replan_severity, replan_auditor_approved, audit_findings, dkg_reflects_new_code, changelog_has_entry, token_budget). Each gate reads only fields it needs — clean decoupling as spec describes.

DAIRA adaptive context: token_budget field present on TransitionContext (line 128) but scope-reduction loop (run -> check size -> reduce -> retry max 3) is NOT implemented in evidence.rs itself. Spec language "DAIRA adaptive context: variable-output tools implement scope reduction" is ambiguous — per spec "variable-output tools", not gates. Gates themselves are fixed-output. Acceptable omission; flagged as observation not as BLOCK.

Tests: evidence.rs has 42 #[test] functions (gates 1-7 each tested positive + negative + missing-artifacts cases; plus gates_for_transition registry tests at line 1003+).

Verdict 3A.3: PASS. 7 gates wired, typed errors with suggestions, DEGRADED passes (warning to caller).

================================================================================
3A.4 INTERACTIVE GATE — VERDICT: CONDITIONAL PASS (SERIOUS WIRING GAP)
================================================================================

File: gate.rs.
InteractiveGate trait: gate.rs:123-131. Method: present_gate(&self, summary: &EvidenceSummary, risk: RiskLevel) -> GateDecision.

GateDecision enum (gate.rs:22-37): Approve, Reject, Modify { updated_context }, Redirect { target }. All 4 decisions present. Spec DETERMINISTIC_CHECK "grep GateDecision:: >= 4" satisfied.

EvidenceSummary (gate.rs:47-59): source, target, phase_label, gate_results: Vec<GateResult>, all_gates_passed. from_transition() iterates evidence::gates_for_transition and aggregates results (line 71-109).

run_interactive_gate (gate.rs:227-293): orchestrates the full protocol.
- LOW risk auto-advances without prompting (line 237-243) — provided evidence still passes. Matches spec "LOW auto" in 3A.5.
- MEDIUM/HIGH loops up to 3 Modify rounds (line 247-289). On exceeding rounds, returns Rejected.
- Approve: re-runs check_transition, returns Approved on pass else EvidenceFailed (line 254-260).
- Reject: returns Rejected immediately (line 261-263). Matches spec ADVERSARIAL "REJECT must restore source state" (FSM is untouched since no transition was applied).
- Modify: replaces current_ctx with updated_context, loop continues — evidence re-evaluates. Matches spec "MODIFY triggers evidence re-check".
- Redirect: validates source.can_transition_to(new_target) (line 274). Invalid targets return InvalidTransition (line 275-280). Matches spec ADVERSARIAL.

MockInteractiveGate (gate.rs:299-340): implements InteractiveGate with pre-configured decision sequence. Trait is mockable — spec ACCEPTANCE satisfied.

Tests: 30 #[test] functions in gate.rs (test_gate_approve_transitions, test_gate_reject_is_noop, test_gate_redirect_valid, test_gate_redirect_invalid_target_rejected, test_gate_modify_re_evaluates, test_low_risk_autoadvances, plus override tests at line 958+). Coverage is thorough.

CLI WIRING — THE SERIOUS GAP:
CliInteractiveGate is defined at cli/src/main.rs:7354-7383. It prints the evidence summary, reads stdin for "approve/reject", returns GateDecision accordingly. It calls io::stdin().read_line() which IS a genuine pause.
HOWEVER: grep -n "CliInteractiveGate" across the workspace returns ONLY the definition site. No instance is ever constructed. No call site passes it to run_interactive_gate in production. run_interactive_gate is only invoked from gate.rs test functions (lines 690, 715, 735, 764, 804, 831, 856, 890, 913, 937, 959, 1017, 1035, 1054) — all under #[cfg(test)].

This means: in production, the FSM performs transition() directly (fsm.rs:517+) without ever routing through run_interactive_gate or InteractiveGate. The human pause/APPROVE/REJECT/MODIFY/REDIRECT ceremony exists only as library code exercised by unit tests. The CLI never instantiates CliInteractiveGate, so the user never sees a gate prompt.

Caveat: Autonomous path via orchestrator/src/autonomous_gate.rs exists (Phase 4G work) but that is explicitly deferred — it does not close this gap; it compounds it. Phase 3A's sole remaining deliverable was "INTERACTIVE primary", and no production code path exercises it.

Spec ACCEPTANCE "all 4 decisions produce correct FSM behavior" is satisfied in the library and tests but NOT end-to-end from a real CLI session. Whether this counts as PASS depends on strictness.
Hostile verdict: LIBRARY PASS, INTEGRATION FAIL. The INTERACTIVE gate does NOT actually pause in the CLI today because CliInteractiveGate is never instantiated. Marking 3A.4 CONDITIONAL PASS — code is correct and fully tested, but not wired into the user-facing transition loop. Recommend a follow-up patch: construct CliInteractiveGate in main.rs session bootstrap and route transitions through run_interactive_gate when mode == Interactive.

================================================================================
3A.5 RISK-LEVEL CLASSIFICATION — VERDICT: PASS
================================================================================

File: gate.rs:143-195.
RiskLevel enum (line 145-150): Low, Medium, High. Derives Serialize/Deserialize for config persistence.
default_risk_level (line 169-195) per-transition:
- Low: MEMORIZING -> PLANNING | COMPLETE (line 172). Matches spec exactly.
- Low: COMPLETE -> IDLE, CONFIGURING -> IDLE (line 174, 176). Reasonable extension.
- Low: all RESEARCHING edges (line 180-183). Consistent with read-only semantics.
- High: EXECUTING -> PLANNING (replan) (line 186). AUDITING -> EXECUTING (line 188). AUDITING -> PLANNING (line 190). Spec "HIGH: destructive actions, file deletion" — replan and dead-end declarations are treated as HIGH; acceptable interpretation.
- Medium: catch-all (line 193). Includes PLANNING -> TESTING_FIRST. Matches spec ADVERSARIAL "default MEDIUM for PLANNING->TESTING_FIRST (not LOW)".

Config overrides: fsm.rs:113-118 FsmConfig { risk_overrides: HashMap<String, RiskLevel> }. resolve_risk_level (fsm.rs:127-134) consults overrides first, falls back to default. Matches spec "Configurable in elai_cc.toml [fsm.risk_overrides]".

Tests: tool_filter + gate + fsm tests cover override path. gate.rs line 958+ "Override MEMORIZING transition" test demonstrates risk_override flag propagating into run_interactive_gate.

LOW auto-skip: implemented inside run_interactive_gate (gate.rs:237-243). LOW never calls gate.present_gate — spec "LOW gates skip presentation" satisfied.

Verdict 3A.5: PASS.

================================================================================
ERROR HYGIENE
================================================================================
Box<dyn Error> occurrences in fsm.rs + evidence.rs + gate.rs: 0. CLAUDE.md rule honored.
.unwrap() occurrences:
- fsm.rs: 20 — ALL inside `#[cfg(test)] mod tests` (line 784 onward). Compliant with project rule "acceptable only in test code".
- evidence.rs: 0.
- gate.rs: 4 — all inside test module.
.expect() with .unwrap() on production paths: one hit in CLI main.rs:7109 — `p.parent().unwrap_or(p)` — this is `.unwrap_or`, not `.unwrap`. Safe.
Overall: error hygiene compliant.

================================================================================
SUMMARY VERDICT
================================================================================
3A.1 FSM State Machine         : PASS (10 states incl. Researching; matrix complete; msgpack atomic; ReplanRequest enforced)
3A.2 Phase-aware Tool/Rule     : PASS (9-state matrix, enforced via typed FsmError::ToolBlockedByState)
3A.3 Evidence Gate Protocol    : PASS (7 gates, trait + registry, typed error with suggestion)
3A.4 INTERACTIVE Gate          : CONDITIONAL PASS (library complete + tested; CliInteractiveGate orphaned — never instantiated, production CLI never pauses)
3A.5 Risk-level Classification : PASS (Low/Medium/High, per-transition defaults, config overrides wired)

OVERALL: REAL IMPLEMENTATION, NOT STUB. One serious wiring gap in 3A.4 prevents end-to-end INTERACTIVE-mode behavior despite library being complete and correct. Recommend patch-X.Y to instantiate CliInteractiveGate in session bootstrap and route Fsm::transition through run_interactive_gate when pipeline mode is Interactive.

FOLLOW-UP ITEMS (non-BLOCK):
- Wire CliInteractiveGate into CLI session control so real users actually see gate prompts.
- DAIRA scope-reduction loop (spec 3A.3 line 162) not implemented in gate.check paths. Clarify whether this was intended for gates or only for variable-output tools.
- COMPLETE/IDLE tool restrictions remain UNGROUNDED per 3A.2 GROUNDING flag. Cite an orchestrator-spec line or mark as documented assumption.
- Consider a compile-time check or lint that rejects MEMORIZING -> COMPLETE without a changelog entry at build time, since runtime gate is the only enforcement today.
