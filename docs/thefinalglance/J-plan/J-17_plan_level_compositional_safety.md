# J-17: plan-level compositional safety — catch the fragmented goal the per-step gates each wave through

- Track J (pillar-SOTA recheck extensions) | lever CANT-DERAIL | grounding STRONG (SIF + SAFEFLOW + SandboxEscapeBench all VERIFIED) | nature BUILD | effort M | WAVE 2 | depends G-1 taint substrate (composes, does not rebuild)

- Premise: ELAI gates every FSM transition with per-step evidence and threads data-plane taint (G-1 Provenanced<String> + capability_gate before destructive sinks). Both operate at the STEP / DATA layer. The GROUNDED GAP (verify-p2-orchestration, SIF arXiv:2604.08608 VERIFIED, AAAI 2026 Summer Symposium): a single legitimately-phrased goal decomposed into individually-benign subtasks can jointly violate policy and pass EVERY per-step gate — the violation only exists at the COMPOSITION, never in any one step. Our hierarchical decomposition (idea -> concepts -> phases -> steps -> babysteps) is exactly the structure SIF exploits, and the paper's uncomfortable finding is that STRONGER orchestrators have HIGHER fragmentation success (better decomposition = better laundering). The fix is a narrow PRE-DISPATCH compositional policy check over the subtask DAG at the concepts->phases boundary: deterministic, one pass, reusing the existing policy-graph + taint infrastructure — NOT a full SAFEFLOW port. Second leg (separate, cheap): the sandbox-escape surface is MISCONFIG not kernel zero-day (SandboxEscapeBench), so the highest-value sandbox work is a config AUDIT of the remote sidecar, then gVisor.

- HONEST SAMPLE NOTE (GD-5 / HD-1): SIF's 71%-violation / 0%-FP-with-plan-IFC numbers are on N=14 enterprise scenarios with an opaque orchestrator — a real MECHANISM on a PROOF-OF-CONCEPT sample, not a production benchmark. J-17 builds the mechanism; its done-gate produces ELAI-measured detection on ELAI's own fragmented-plan fixtures, never quotes the 71%.

## Steal/build
1. compositional policy check (crates/orchestrator, new leg on the PLANNING -> dispatch path): at the concepts->phases expansion, assemble the subtask DAG and evaluate a policy predicate over the WHOLE set, not per-node — i.e. does the UNION of declared subtask effects (the WIRING_CONTRACT produces/consumes + capability requests across all subtasks) compose into a capability the policy denies at the goal level, even though each subtask is individually permitted. Reuses the rules-engine COMPOSITE predicate (3F) + the G-1 taint graph (the capability set per subtask already exists); the new artifact is the goal-level aggregation + denial check. Deterministic verdict ALLOW / FRAGMENTATION-FLAGGED / BLOCK.
2. The check runs at the PLAN plane BEFORE any subtask dispatches (the SIF lesson: catching it at execution is too late). OBSERVE-ONLY first (GD-4 / JD-3 analog): flag the fragmentation with the offending subtask subset, do not block, until the FP rate is bounded on >=3 real plans; then BLOCK behind a wire-dark-allowlist row.
3. sandbox config-audit leg (separate, model-free): a deterministic auditor over the remote sidecar's container/sandbox config (Docker socket exposure, privileged flags, writable host mounts, cap set) — the SandboxEscapeBench misconfig classes. Emits a config-audit report; a known-bad config = a surfaced finding (Priority 1, the report's verdict: seccomp+caps+MAC already blocks ~67.57% of priv-esc, config is the live hole).
4. gVisor note (Priority 2, NOT built here): document runsc as the drop-in for the Linux remote sidecar (low friction, ~100ms boot, 10-30% I/O overhead on compute-light agent tool exec); Firecracker (Q3-2026) and WASM/WASI (WASI 1.0 ~2027) are forward-looking — recorded, not authored.
EXCLUDED: a full SAFEFLOW / IFC-everywhere port (the narrow goal-level aggregation is the 80% at 5% of the cost; SAFEFLOW is the XL ceiling); any sandbox REPLACEMENT of the shipped Tier-C OS-process isolation (the report says HARDEN-not-replace, layered defense); the SIF 71% as a target.

## Existing ground truth (grep-confirmed; re-grep at edit, W-7/W-9)
- G-1 TAINT PROPAGATION [WIRED]: Provenanced<String> at 4 ingest sites + capability_gate before destructive sinks (feature map 1.6). The per-subtask capability set already exists; J-17 aggregates it.
- RULES ENGINE [WIRED] COMPOSITE predicate (crates/rules/, phase-3F) — the predicate surface the goal-level check rides.
- HIERARCHICAL PLAN MGMT [WIRED] (phase-3G) + PIPELINE FACTORY L2 DecompositionTree (phase-4E) — the concepts->phases->steps DAG the check reads.
- TIER-C PROCESS ISOLATION [WIRED core] seccomp 2 profiles + SO_PEERCRED (phase-4A) + B-18 sandbox (allowed_paths PARTIAL) — the sandbox the config-audit inspects; remote sidecar deploy is the audited target.
- compliance.rs Article-13 pre-gate (E-38) — precedent for a PLANNING/FEASIBILITY_PROBE->TESTING_FIRST pre-dispatch gate; the compositional check sits at the same seam.

## Integration
- crates/orchestrator/src/compositional_policy.rs (new, ~180 LOC: subtask-DAG capability aggregation + goal-level denial predicate, reusing rules-engine COMPOSITE + G-1 capability sets) wired at the concepts->phases expansion in the plan-management path.
- crates/orchestrator/src/sandbox_audit.rs (new, ~120 LOC: deterministic config inspector over the sidecar sandbox config; model-free).
- +CompositionalPolicyEvent SSB (Pattern-2 single-variant; Kind: PlanGraphChecked / FragmentationFlagged / SandboxConfigAudited / CompositionBlocked). baseline+1 from LIVE 231/542 (ssb.rs:3084 / action_type_lock.rs:671, grep 2026-06-12); re-read at edit (W-9). Full 6-part variant discipline. PREFER-REUSE check at edit: if a G-1 capability-gate row already carries a composition verdict shape, extend it instead (net 0); decide at edit per the J-2 precedent.
- frontend_pair: a fragmentation finding on the existing EvidenceGate / compliance panel surface (decide at edit; W-4) or W-6-justified absence.

## WIRING_CONTRACT
produces_symbols:
- crates/orchestrator/src/compositional_policy.rs::aggregate_subtask_capabilities (goal-level union over the DAG)
- crates/orchestrator/src/compositional_policy.rs::check_goal_policy (the deny-at-composition predicate)
- crates/orchestrator/src/sandbox_audit.rs::audit_sidecar_config (misconfig inspector)
consumes_symbols:
- G-1 capability_gate / Provenanced (the per-subtask capability sets; NOT reauthored)
- crates/rules/ COMPOSITE predicate (the rules-engine surface; NOT reauthored)
- the DecompositionTree / concepts->phases DAG (phase-3G/4E)
reachable_from:
- the PLANNING -> phase-dispatch transition on a real `elai` plan/orchestrate run (pre-dispatch)  [PRODUCTION]
- the sidecar boot / health path for the config-audit leg
- a #[cfg(test)] / contracts/ caller NEVER satisfies this (R7)
stub_ban:
- NO per-step-only check (the whole point is goal-level aggregation; a per-node predicate is the gap, not the fix)
- NO LLM judgment of "is this plan malicious" (deterministic capability aggregation + policy predicate; TD-3)
- NO BLOCK default until the FP rate is bounded on >=3 real plans (GD-4) + a wire-dark-allowlist row (W-2/R9)
- NO SAFEFLOW-scale IFC rewrite (narrow goal-level check only)
- NO sandbox REPLACEMENT (layered hardening; the config-audit + gVisor note do not remove Tier-C)
frontend_pair: fragmentation finding on the compliance/evidence panel (decide at edit) or W-6-justified absence.
observable_effect:
- trigger: "an `elai` plan whose subtasks are individually benign but jointly request a denied capability (fixture: subtask A reads a secret file, subtask B opens an egress socket, subtask C writes the combined payload — each permitted alone, the composition is exfiltration), expanded at concepts->phases"
- expect_ssb: "PlanGraphChecked over the full DAG; FragmentationFlagged naming the {A,B,C} subset that composes the denied capability"
- expect_artifact: "the fragmentation is caught BEFORE subtask A dispatches (pre-dispatch, not at execution); SandboxConfigAudited emits the sidecar config verdict. NEGATIVE (R2): a plan whose subtasks are individually AND jointly benign passes with CompositionCleared (no false flag); the check pointed at a DAG it cannot fully resolve FAILS-LOUD, never silently clears; a known-bad sidecar config (Docker socket mounted) raises a finding, not silence."

## Grounding
- SIF arXiv:2604.08608 (VERIFIED via verify-p2, AAAI 2026 Summer Symposium): goal fragmentation bypasses subtask-safety; plan-level IFC the named fix. N=14 sample noted (mechanism strong, sample weak).
- SAFEFLOW arXiv:2506.07564 + MAScope arXiv:2603.04469 (both VERIFIED): information-flow control at the agent-boundary / cross-agent semantic-flow reconstruction — the mechanism family J-17's goal-level check is a narrow instance of.
- PRUDENTIA arXiv:2602.11416 (VERIFIED, the real Microsoft paper; the report's "MAControl" label is a HALLUCINATION — see ledger Tier-3): IFC at the planning layer increases autonomous execution without losing security.
- SandboxEscapeBench arXiv:2603.02277 (VERIFIED, ICML 2026 Oral): 18 escape scenarios, the live vector is MISCONFIG; Containment arXiv:2604.23425 R1 (enforcement in unreachable trust domain) / R4 (write-only invisible audit).
- SDOF arXiv:2605.15204 + SEMAP arXiv:2510.12120 (both VERIFIED): VALIDATE the FSM choice (86.5% completion / 56-70% MAST reduction) — J-17 hardens, never replaces, the FSM.
- live SSB baseline 231/542 (grep 2026-06-12).

## COMPOSES-WITH
G-1 taint (consumes its per-subtask capability sets), rules-engine 3F (COMPOSITE predicate), E-38 compliance pre-gate (sibling pre-dispatch gate), Tier-C 4A / B-18 sandbox (the config-audit inspects, does not replace), J-16 blast-radius (the per-EDIT sibling of this per-PLAN check). Restates none.

## Open questions / risks
- aggregation precision: the union of declared subtask capabilities may OVER-approximate (flag benign compositions) or UNDER-approximate (a capability that only emerges at runtime, not declared). OBSERVE-ONLY until the FP rate is known (GD-4); document that the check is necessary-not-sufficient (the runtime G-1 taint remains the backstop, R5).
- DAG completeness: a subtask added dynamically (SPLIT_TASK, 4G) after the concepts->phases check must re-trigger the aggregation — wire the check to re-run on DAG mutation, not once.
- sandbox-audit scope: Windows-primary host + Linux remote sidecar — the config-audit targets the LINUX sidecar (where tool exec runs); the Windows side is out of scope (no container there). Name the boundary.

## Next
The goal-level check is the per-PLAN sibling of J-16's per-EDIT blast-radius gate — together they close the "narrow focus" failure at both granularities. gVisor on the remote sidecar is the natural follow-on once the config-audit surfaces the live gaps.

End of J-17.
