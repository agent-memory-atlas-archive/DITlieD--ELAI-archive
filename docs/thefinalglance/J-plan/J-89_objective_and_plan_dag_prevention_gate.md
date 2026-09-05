# J-89: the prevention pair — objective-correctness gate (L1) + plan-DAG type-completeness (L8)

- Track J (lacuna-fill track) | lever PREVENT-BEFORE-CODE | grounding STRONG (live I-45/spec-grounder/D-13/wiring_contract_check.py grep) | nature BUILD (spec-bundle schema + jury wiring + plan-DAG symmetry lift) | effort L | WAVE 1 | depends J-34 for the L1 escalation-point ENFORCEMENT half (MECHANISM ships now)
- RUTHLESS-3 (reasoning §9): the only fills that beat DETECTION by stopping the error before code. Together they make "is this even the right thing to build, and is the plan complete" a GATE instead of a hope.

## LEG A — L1 objective-correctness gate (Goodhart at the spec layer)
- Premise: ELAI proves the agent honestly satisfied the objective; it never proves the OBJECTIVE is the one the user meant. I-45 detects objective MUTATION after PASS but SEALS a possibly-wrong objective. A correct execution of a wrong objective passes every downstream gate.
- Build (report 7, a PRE-CODING stack; design-rationale only):
  1. SPEC BUNDLE — force every objective into a TYPED artifact (purpose, scope, domain assumptions, non-functional constraints, competency questions, obstacle log). ELAI has spec-grounder + grounding-sweep as ADVISORY pre-steps; PROMOTE them to a first-class TAINT-TRACKED evidence-bound artifact with DETERMINISTIC coverage checks (every goal operationalized, every assumption referenced, every competency question answered). This is the L8 input too.
  2. CROSS-FAMILY ADVERSARIAL SPEC JURY — a panel from DISJOINT families attacking the OBJECTIVE (omission, ambiguity, harmful readings, proxy-gaming), DISTINCT from the auditor that checks "did the agent satisfy it." ELAI has the cross-family substrate (D-13 adversary + gpt-bridge). PoLL "diverse small-judge panel beats one large judge, less intra-model bias, >7x cheaper" is design-rationale. THE JURY IS THE ONE PLACE AN LLM ENTERS THE WHOLE PROGRAM — legal precisely because it runs at PLANNING (objective time, OFF the hot path), not AUDITING, and its output is ADVISORY-by-construction ("survived N challenges", escalate-on-disagreement), NEVER a deterministic verdict (TD-3 §8 R2).
  3. SPAN-LINKED AUTOFORMALIZATION of the highest-risk clauses ONLY (nl2spec/SYNTHTL prior-art) — narrow, not whole-spec.
  4. ATTESTATION = I-45 + G-8 (already planned) as the FREEZE step (the spec-bundle is sealed; mutation after PASS -> detectably stale receipt).
- EXCLUDED: the jury rendering a deterministic gate verdict (TD-3 — it is advisory + escalate-on-disagreement); whole-spec autoformalization (highest-risk clauses only); any claim of "correct objective" (Rice/halting — only "survived N independent challenges + frozen attestation").

## LEG B — L8 plan-DAG producer->consumer type-completeness check (prevention-grade)
- Premise: the walking-skeleton PRE_GATE rejects bare "add fn X" at AUTHORING via a planner-LLM + a PROSE rule, not a FORMAL completeness/reachability check over the decomposition DAG. Wire-dark is DETECTED at audit, not PREVENTED at plan time.
- Build (report 7 §DAG preflight — workflow nets / MPST session types / PCCA prior-art): give every plan node a TYPED signature (required inputs, produced outputs, consumed assumptions, emitted obligations). Run three DETERMINISTIC checks BEFORE any code: GOAL COVERAGE (every leaf traces to a requirement — from L1's competency questions), ARTIFACT REACHABILITY (every produced artifact has a reachable consumer or an approved terminal sink — the PCCA producer/consumer-language containment check), PROTOCOL REALIZABILITY (every handoff accepted, no orphan, no deadlock). ELAI has the RUNTIME analog (wiring_contract_check.py producer/consumer symmetry) + the typed substrate (BabyStepSpec 8-block schema, C-29 contract extraction, Kahn cycle detection) — LIFT the runtime symmetry to PLAN-DAG time. Converts wire-dark DETECTION into PREVENTION.
- EXCLUDED: any claim of SEMANTIC completeness (L8 proves STRUCTURAL completeness only — pair each leaf->goal->competency-question binding with the L1 jury attacking the UNCOVERED spaces); a new symmetry engine (lift wiring_contract_check.py, GD-1).

## Existing ground truth (grep-confirmed; re-grep at edit, W-7/W-9)
- OBJECTIVE-HASH SEALED RECEIPTS I-45 [PLANNED] blake3(canonical objective bytes) into rra.rs seal — L1 makes the sealed objective a typed spec-bundle BEFORE code.
- spec-grounder agent (.claude/agents/spec-grounder.md) + grounding-sweep — the ADVISORY pre-steps L1 promotes.
- ADVERSARY ROLE CROSS-FAMILY D-13 [DARK] + gpt-bridge — the spec-jury substrate.
- wiring_contract_check.py (.claude/tools, producer/consumer symmetry; WIRED) + BabyStepSpec 8-block + C-29 contract extraction + Kahn cycle detection (phase-3G) — L8's runtime analog + typed substrate.
- IN-TOTO ATTESTATION G-8 [PARTIAL] attestation.rs — the freeze step.
- live SSB baseline ~268/577 (grep 2026-06-26).

## Integration
- L1: crates/orchestrator/src/spec_bundle.rs (the typed taint-tracked spec-bundle + deterministic coverage checks) + a spec-jury dispatch (PLANNING-time, gpt-bridge/D-13, advisory). Gate at PLANNING->TESTING_FIRST (composes the existing grounding-sweep block). +SpecBundleEvent SSB (Kind: BundleSealed / JuryChallengeSurvived / JuryEscalated). NET-NEW (the wave's second new-event candidate; or reuse the I-45 ObjectiveSealed arm — prefer-reuse, decide at edit). baseline+0..1 from LIVE ~268/577; re-read at edit (W-9).
- L8: .claude/tools/plan_dag_check.py (the plan-DAG goal-coverage / artifact-reachability / protocol-realizability checker, lifting wiring_contract_check.py's symmetry) wired into the planning-workflow PRE_GATE injection + b-plan-runner AUTHORING. NO new SSB (harness, JD-1 log evidence). FROZEN gate (R8, gate-manifest).
- FEATURE FLAGS: ELAI_SPEC_BUNDLE_GATE_ENABLE (kill-switch, the bundle MECHANISM ships now; the jury-escalation ENFORCEMENT point is J-34-measured) + the L8 checker observe-only first; wire-dark-allowlist rows with promotion plans (W-2/R9).
- frontend_pair: a spec-bundle/jury panel on the planning surface (decide at edit) + the L8 plan-DAG report in the existing plan view; W-6-justified where absent.

## WIRING_CONTRACT
produces_symbols:
- crates/orchestrator/src/spec_bundle.rs::build_spec_bundle (typed taint-tracked objective artifact)
- crates/orchestrator/src/spec_bundle.rs::check_bundle_coverage (deterministic: every goal operationalized / assumption referenced / competency question answered)
- crates/orchestrator/src/spec_bundle.rs::run_spec_jury (cross-family PLANNING-time advisory panel; escalate-on-disagreement)
- .claude/tools/plan_dag_check.py::{check_goal_coverage, check_artifact_reachability, check_protocol_realizability}
consumes_symbols:
- I-45 sealed-objective bytes + G-8 attestation (the freeze step; not reauthored)
- spec-grounder + grounding-sweep (the advisory pre-steps; promoted, not reauthored)
- D-13 + gpt-bridge (the cross-family jury substrate; not reauthored)
- wiring_contract_check.py symmetry + BabyStepSpec + C-29 + Kahn (the runtime analog lifted to plan-DAG; not reauthored)
reachable_from:
- L1: the PLANNING->TESTING_FIRST transition on a real `elai` plan/idea-intake run  [PRODUCTION]
- L8: the planning-workflow PRE_GATE / b-plan-runner AUTHORING step on a real plan author run  [PRODUCTION]
- a #[cfg(test)] / contracts/ caller NEVER satisfies either (R7)
stub_ban:
- NO jury rendering a deterministic verdict (advisory + escalate-on-disagreement; TD-3 §8 R2 — the ONE LLM entry, planning-time only)
- NO claim of "correct objective" (only "survived N challenges + frozen attestation"; abstain->human TD-5 is a first-class EXIT)
- NO claim of SEMANTIC plan completeness (L8 is STRUCTURAL; pair with the L1 jury on uncovered spaces)
- NO new symmetry/attestation engine (lift wiring_contract_check.py; reuse I-45/G-8; re-implementing = GD-1 P0)
- NO ENFORCEMENT of the jury escalation point until J-34-measured (the bundle coverage MECHANISM ships observe-only; RULE 1)
frontend_pair: spec-bundle/jury planning panel + plan-DAG report (decide at edit) or W-6-justified absence.
observable_effect:
- trigger: "an `elai` idea-intake run whose objective has an UNSTATED domain assumption (e.g. 'parse the file' with no encoding named); AND a plan-author run with a leaf that PRODUCES an artifact no later node consumes"
- expect_ssb: "L1 emits BundleSealed for the typed objective; the jury raises JuryChallengeSurvived listing the ambiguity (the unstated encoding) and JuryEscalated if a cross-family disagreement persists"
- expect_artifact: "L1: check_bundle_coverage FAILS the unstated assumption (a competency question unanswered) at PLANNING, BEFORE code. L8: plan_dag_check FAILS the orphan-producer leaf (no reachable consumer) at AUTHORING, BEFORE code. NEGATIVE (R2): an empty competency-question set FAILS coverage (never SKIP); a plan-DAG the checker cannot parse FAILS, never passes; the jury returning a single-family panel is REFUSED (cross-family is required)."

## Grounding
- L1: report 7 §spec-bundle/§spec-jury; nl2spec/SYNTHTL + PoLL prior-art NAMES (design-rationale; the >7x-cheaper figure is NOT an ELAI number). Done-gate = a real unstated-assumption objective FAILS coverage at planning.
- L8: report 7 §DAG preflight; workflow-nets / MPST / PCCA prior-art NAMES. Done-gate = a real orphan-producer leaf FAILS at authoring.
- substrate: I-45/D-13/G-8 [PLANNED/DARK/PARTIAL] + wiring_contract_check.py/BabyStepSpec/C-29/Kahn [WIRED] — feature-map 1.2/1.3/1.6/1.14.
- RESIDUE (report 7 §collapse): Rice/halting — no algorithm decides whether an arbitrary objective faithfully captures intent. L1's honest output is "survived N independent challenges + frozen attestation", never "correct objective". TD-5 abstain-and-escalate-to-human is a first-class EXIT.
- live SSB baseline ~268/577 (grep 2026-06-26).

## COMPOSES-WITH
I-45 (the sealed objective L1 enriches into a bundle; the freeze step), spec-grounder + grounding-sweep (the advisory pre-steps promoted), D-13 + gpt-bridge (the jury), G-8 (attestation), wiring_contract_check.py + BabyStepSpec + C-29 + Kahn (L8's lifted symmetry), J-87 (the spec-bundle is taint-tracked through G-1), the wiring-enforcement PRE_GATE (L8 IS its plan-time formalization). Restates none.

## Open questions / residue (reasoning §7)
- L1 JURY GAMEABILITY: correlated cross-family blind spots + position bias (PoLL mitigates, never eliminates). The output is bounded to "survived N challenges"; abstain->human is the EXIT, not decoration.
- LOCAL-LANE RESIDUE (R3): ~1.8GB free VRAM cannot co-resident a second family, so the LOCAL spec-jury DEGRADES to the deterministic-checker-as-cross-family substitute (the J-94 C5-SELFMOD THIN item). Document the per-lane asymmetry.
- L8 STRUCTURAL-ONLY: proves no orphan/deadlock, not that the plan ACHIEVES the goal — that semantic gap is the L1 jury's job; the two compose, neither alone suffices.

## Next
L1's spec-bundle is L8's typed input; L8's reachability is the plan-time formalization of the runtime wiring gate. Together they are the prevention front-end the L4/L5/L7 detection gates (J-90) back-stop.

End of J-89.
