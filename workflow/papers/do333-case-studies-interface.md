Algorithm: DO-333 Formal Methods Verification Framework (Three-Technique Stack)
Report: NASA/CR-2014-218244
Title: Formal Methods Case Studies for DO-333
Authors: Darren Cofer and Steven P. Miller, Rockwell Collins, Inc.
Source: NASA NTRS https://ntrs.nasa.gov/citations/20140004055
Local PDF: F:\.ELAI_workflow\.claude\papers\do333-case-studies.pdf


OVERVIEW

DO-333 (RTCA Formal Methods Supplement to DO-178C/DO-278A) defines how three classes of formal methods satisfy airborne software certification objectives. This report demonstrates each technique applied to a dual-channel Flight Guidance System (FGS). The techniques cover three lifecycle tiers: high-level requirements (theorem proving), low-level requirements and architecture (model checking), source code (abstract interpretation). This maps directly to ELAI's FSM/workflow invariants, mode/state logic, and implementation-level hazard detection.


TECHNIQUE-TO-LIFECYCLE MAPPING (Figure 2 and Section 1, p.15)

Theorem Proving | Applied to: High-Level Requirements (DO-178C Section 5.1 / Table A-3 + FM.A-3) | Focus: Compliance, accuracy/consistency, algorithm correctness at the requirements specification level | Tool used: PVS (Prototype Verification System), HOL4
Model Checking | Applied to: Low-Level Requirements and Software Architecture (Table A-4 + FM.A-4) | Focus: LLR compliance with HLR, architecture consistency and verifiability, algorithm accuracy of mode logic | Tool used: Kind/JKind (SMT/k-induction), MATLAB Design Verifier
Abstract Interpretation | Applied to: Source Code (Table A-5 + FM.A-5) | Focus: Absence of runtime errors, code accuracy/consistency (non-functional properties) | Tool used: Astree, Polyspace


OBJECTIVES SATISFIED (from Tables 1, 2, 49)

THEOREM PROVING satisfies (Table 1, p.25):
A-3.1 | HLR comply with system requirements | FULL credit (A/B/C/D) | "Established by proof the system requirements are implemented by the high-level requirements and the system architecture"
A-3.2 | HLR accurate and consistent | FULL credit (A/B/C/D) | "Accuracy is established by formalization; Consistency is established by proving the absence of logical conflicts"
A-3.3 | HLR compatible with target computer | NOT ADDRESSED
A-3.4 | HLR verifiable | FULL credit (A/B/C) | "Established by formalizing the requirements and completion of the proof"
A-3.5 | HLR conform to standards | PARTIAL credit (A/B/C) | "Partially established by specifying the high-level requirements as formal properties"
A-3.6 | HLR traceable to system requirements | FULL credit (A/B/C/D) | "Established by verification of the system requirements, and by demonstrating the necessity of each high-level requirement"
A-3.7 | Algorithms are accurate | FULL credit (A/B/C) | "Correctness of the pilot flying selection logic is established by proof"
FM.A-3.8 | Formal analysis cases/procedures correct | FULL credit (A/B/C) | "Established by review"
FM.A-3.9 | Formal analysis results correct, discrepancies explained | FULL credit (A/B/C) | "Established by review"
FM.A-3.10 | Requirements formalization correct | FULL credit (A/B/C) | "Established by review"
FM.A-3.11 | Formal method correctly defined, justified, appropriate | FULL credit (A/B/C/D) | "Established by review"

MODEL CHECKING satisfies (Table 2, p.101):
A-4.1 | LLR comply with HLR | FULL credit (A/B/C) | "Established by proof that the high-level requirements are implemented by the low-level requirements and the software architecture"
A-4.2 | LLR accurate and consistent | FULL credit (A/B/C) | "Established by modeling using an executable language and translation to a formal specification language"
A-4.3 | LLR compatible with target computer | NOT ADDRESSED
A-4.4 | LLR verifiable | FULL credit (A/B) | "Established by modeling using an executable language and translation to a formal specification language"
A-4.5 | LLR conform to standards | PARTIAL credit (A/B/C) | "Established by use of Simulink/Stateflow design language"
A-4.6 | LLR traceable to HLR | PARTIAL credit (A/B/C) | "Established by verification of the high-level requirements"
A-4.7 | Algorithms accurate | FULL credit (A/B/C) | "The accuracy of the mode logic is established by model checking"
A-4.8 | Software architecture compatible with HLR | FULL credit (A/B/C) | "Established by proof that the high-level requirements are implemented by the low-level requirements and the software architecture"
A-4.9 | Software architecture consistent | FULL credit (A/B/C) | "Established by modeling using an executable language and translation to a formal specification language"
A-4.10 | Software architecture compatible with target computer | NOT ADDRESSED
A-4.11 | Software architecture verifiable | FULL credit (A/B) | "Established by modeling using executable language and formal specification language"
A-4.12 | Software architecture conforms to standards | PARTIAL credit (A/B/C) | "Partially established by use of Simulink/Stateflow"
A-4.13 | Software partitioning integrity | NOT ADDRESSED (in current case study)
FM.A-4.14 through FM.A-4.17 | Formal method process objectives | FULL credit (A/B/C) | "Established by review"

ABSTRACT INTERPRETATION satisfies (Table 49, p.165):
A-5.1 | Source code complies with LLR | NOT ADDRESSED
A-5.2 | Source code complies with software architecture | NOT ADDRESSED
A-5.3 | Source code verifiable | PARTIAL credit (A/B) | "May be partially satisfied by demonstrating that the code conforms to input restrictions for the analysis tool"
A-5.4 | Source code conforms to standards | PARTIAL credit (A/B/C) | "May be partially or fully satisfied by different analysis tools, depending upon the coding standards and tool qualification"
A-5.5 | Source code traceable to LLR | NOT ADDRESSED
A-5.6 | Source code accurate and consistent | PARTIAL credit (A/B/C) | "The absence of some classes of run-time errors is established through analysis with abstract interpretation tools"
A-5.7 through A-5.9 | Integration/parametric objectives | NOT ADDRESSED
FM.A-5.10 | Formal analysis cases/procedures correct | FULL credit (A/B/C) | "Established as part of tool qualification"
FM.A-5.11 | Formal analysis results correct, discrepancies explained | FULL credit (A/B/C) | "Established by review"
FM.A-5.12 | Requirements formalization correct | FULL credit (A/B/C) | "Established as part of tool qualification"
FM.A-5.13 | Formal method correctly defined, justified, appropriate | FULL credit (A/B/C/D) | "Established by review"


REPLACE vs AUGMENT vs CANNOT SATISFY (from Sections 3-5, explicit in case study structure)

REPLACE (full DO-178C credit via formal method alone):
Theorem proving REPLACES testing for HLR compliance (A-3.1, A-3.2, A-3.6, A-3.7): proof constitutes the evidence, no separate test suite required for those objectives at those levels.
Model checking REPLACES review/testing for LLR compliance and architecture consistency (A-4.1, A-4.7, A-4.8, A-4.9): model checker proof is the evidence artifact.
Abstract interpretation PARTIALLY replaces source code testing for non-functional runtime properties (A-5.6): "absence of some classes of run-time errors is established through analysis."

AUGMENT (partial credit; formal method plus additional review required):
A-3.5, A-4.5, A-4.6, A-4.12: formal method partially satisfies; standards conformance and traceability reviews remain required.
A-5.3, A-5.4: conformance to input restrictions and coding standards partially met; tool must actually accept the code and any non-conforming portion must be verified by other methods.

CANNOT SATISFY (explicitly not addressed in any case study):
Hardware compatibility objectives (A-3.3, A-4.3, A-4.10): formal method gives no evidence on target computer compatibility.
Source-to-LLR compliance (A-5.1, A-5.2, A-5.5): abstract interpretation in this study checks non-functional only; behavioral compliance from code to requirements not covered.
Executable Object Code objectives (Table FM.A-6): not addressed; requires separate EOC verification or proof that source-to-EOC equivalence holds (DO-333 paragraph 6.7.f).
Coverage testing replacement (Table FM.A-7): not addressed in any case study; FM.A-7 is a separate DO-333 table.
Source code traceable to LLR (A-5.5): not addressed.


EVIDENCE REQUIREMENTS (from FM.A-X.8 through FM.A-X.11 in each case study)

Each formal method activity must produce the following evidence (verbatim from Sections 3.2.3, 4.2.3, 5.2.2):

1. Formal analysis cases and procedures: "review to ensure that the analyses and procedures satisfy the objectives for which credit is claimed" (FM.A-3.8, FM.A-4.14, FM.A-5.10). For abstract interpretation, properties are implicit in the tool; tool qualification itself constitutes this evidence.

2. Formal analysis results with discrepancy explanations: "review to ensure that all theorems or lemmas are proven" (FM.A-3.9). "Many of the properties had to be revised before they could be proved. Each such discrepancy was explained and fed back into the safety assessment process for review." (p.27 for theorem proving; p.102 for model checking)

3. Requirements formalization correctness: "review to ensure that the formal statement of a requirement is a conservative representation of the informal requirement" (FM.A-3.10). For abstract interpretation this is satisfied by tool qualification since properties are implicit.

4. Formal method justification: three sub-requirements per technique (FM.A-3.11, FM.A-4.17, FM.A-5.13):
   a. "All notations used for formal analysis are verified to have precise, unambiguous, mathematically defined syntax and semantics" (Section 1, p.13)
   b. "The soundness of each formal analysis method is normally demonstrated by citing research papers that discuss the soundness of the method(s) implemented in a given tool" (p.27 for TP; p.103 for MC; p.166 for AI)
   c. "Assumptions related to each formal analysis are described and justified" (required for all three techniques)

5. Axiom consistency: "whenever axioms are used in a specification, it is important to prove that they are consistent. The standard way of doing this is to demonstrate that an implementation exists that satisfies all the axioms." (p.81)

6. Tool qualification: "qualification of a tool is needed when: (1) DO-178C processes are eliminated, reduced, or automated through the use of the tool, and (2) the output of the tool is used without being verified." (p.28). Tool Operational Requirements (TORs) must be defined; "Model checking does not (in general) produce independently checkable output. This means that the model checker must be qualified if its outputs are to be used for certification credit." (p.102)


DO-333 GENERAL REQUIREMENTS (Section 1, p.13; applies to all three techniques)

"DO-333 provides general guidance that is applicable to the overall verification process when formal methods are used. This includes:
 - requirements for the use of formal notations with unambiguous, mathematically defined syntax and semantics
 - soundness of the formal analysis methods used
 - justification of all assumptions used in each formal analysis"

Soundness requirement: theorem proving requires axioms consistent + all types non-empty; model checking soundness established via k-induction citations; abstract interpretation tools may return "indeterminate" for portions they cannot prove, which must be verified by other means.

Indeterminate results: "Any portion of the code for which the tool provides an 'indeterminate' result must be verified through other methods." (A-5.6 discussion, p.166). False alarms "must be justified by separate analysis or testing." This is the critical hard boundary: a sound formal tool that cannot decide is not a pass.


FAILURE MODES AND LIMITATIONS (from Sections 3-5)

Theorem proving: non-automated, requires significant user expertise; inconsistent axioms can prove false theorems; cyclic dependencies must be manually resolved; assumptions about timing models must be reviewed; PVS does not normally emit checkable proof certificates.

Model checking: does not produce independently checkable output (no proof certificate); tool qualification under TQL-5 required for all software levels; counterexamples reveal requirement errors, not just implementation errors; ALTSEL/FLC/VGA errors found in requirements during verification, not post-implementation.

Abstract interpretation: false alarms require separate analysis or testing to justify; indeterminate results on any code section require other verification methods; input range assumptions are proof assumptions that must be documented; only non-functional (runtime error) properties addressed in commercial tools; A-5.1/A-5.2 (behavioral compliance) not addressed.

Coverage replacement (FM.A-7): not covered by any of the three case studies; DO-333 Table FM.A-7 objectives for replacing structural coverage testing with formal analysis are a separate topic.

EOC objectives (FM.A-6): formal analysis on source code requires separate proof of source-to-EOC equivalence per DO-333 section 6.7.f before EOC credit can be claimed.


LIFE CYCLE DATA ITEMS PRODUCED (Section 3.2.2, 4.2.2, 5.2.1)

Theorem proving produces: formal specification theories (PVS/HOL4) for system architecture, system requirements, HLR software and hardware, LLR software and hardware; machine-checked proof scripts; theory interpretation theories proving HLR-to-LLR compliance; consistency proofs showing axioms are satisfiable.

Model checking produces: Simulink/Stateflow executable models for LLR and software architecture; Lustre formal specification (auto-translated from Simulink); Lustre predicates for HLR (the properties to be verified); JKind/Design Verifier proof logs and counterexamples; mode error log (Appendix C, 198 pages of found errors in mode logic requirements).

Abstract interpretation produces: Astree/Polyspace analysis reports; directive files defining partitions; list of alarms with status (true alarm vs false alarm with justification); unproven runtime checks table; coding rules analysis results.


COMPLEXITY AND SCOPE CONSTRAINTS

Theorem proving: manual effort is high; requires expertise in formal specification and proof strategies; interactive proof sessions with tool; scales poorly to large state spaces without user-guided lemma decomposition.

Model checking: suited for Boolean and enumerated types natively; handles small integers; SMT-based (Kind/JKind) handles reals if no nonlinear arithmetic; does not scale to arbitrary source code; state space must be representable in the model.

Abstract interpretation: most automated of the three; scales to C source code; requires fine-tuning (partitioning directives) to eliminate false alarms; commercial tools (Astree, Polyspace) focus on non-functional properties; more powerful domain-specific abstract interpreters require greater expertise.


DETERMINISTIC-GATE RULES (ELAI mapping)

[See section below]


ELAI ARTIFACT GATES (derived from DO-333 objective structure)

RULE-TP-1 (Theorem Proving gates FSM/workflow invariants):
Theorem proving maps to ELAI's FSM transition invariants and wiring contracts. An ELAI harness control that claims a workflow invariant holds (e.g., "IDLE -> EXECUTING requires context-compiler to have returned") is a high-level requirement in DO-333 terms. The gate for that control is a proof that the invariant is satisfied by the FSM implementation — not a test that happens to pass. Evidence required: a formal or semi-formal proof artifact (e.g., a verified assertion, a model proof, or a contract test that is itself formally specified) plus a consistency demonstration that the axioms cannot be simultaneously falsified.

RULE-MC-1 (Model Checking gates mode/state logic):
Model checking maps to ELAI's mode dispatch logic, FSM state machine implementation, and agent substate transitions. The gate for a new state or mode is a model-checked property over the state space, not just unit tests. Evidence required: the model checker's proof log (or k-induction result) showing the property holds for all reachable states; counterexample traces for any property that fails must be fed back into requirements, not patched silently.

RULE-AI-1 (Abstract Interpretation gates implementation-level hazards):
Abstract interpretation maps to ELAI's Rust implementation for absence of runtime panics, integer overflows, and undefined behavior categories. The gate is a sound static analysis over the compiled code (or a provably equivalent analysis). Evidence required: analysis report showing zero unproven runtime checks in the code paths that handle FSM transitions, SSB writes, and hook executions. Indeterminate results on any path require justification or alternative verification. False alarms must be documented with justification.

RULE-OBJ-1 (Each harness control = a verification objective with required evidence, traced):
Per DO-333's general structure: every gate in the ELAI harness corresponds to one or more DO-178C-style verification objectives. Each gate must name: (a) which lifecycle tier it covers (HLR/LLR/source), (b) which technique provides the evidence, (c) what artifact the technique produces, (d) whether the objective is fully or only partially satisfied. A gate that is "fully satisfied" means the artifact alone constitutes evidence; a gate that is "partially satisfied" requires additional review or testing to close. There is no grade between partial and full — unknown or indeterminate is a failure, never a pass (mirrors wiring-enforcement.md R1).

RULE-SOUND-1 (Soundness of the analysis method must be demonstrated, not assumed):
Per FM.A-3.11/FM.A-4.17/FM.A-5.13: the soundness of every formal analysis tool used in ELAI gates must be cited to external publications or tool qualification artifacts. "We use cargo check" is not a soundness argument. "cargo check is sound for type safety per the Rust reference and rustc verification literature" is the correct framing. An ELAI gate that relies on a tool whose soundness is not documented is a PARTIAL at best.

RULE-INDET-1 (Indeterminate result = verification failure):
Per A-5.6 and FM.A-5.13b: an analysis tool that returns "indeterminate" for a code region does not give that region a pass. The region must be verified by another method. In ELAI terms: a wiring check that cannot resolve a symbol (unsound name grep returning UNSOUND-NAME verdict) does not count as WIRED; it downgrades to a WARN that requires resolution by a sound method (codegraph_callers or runtime trace).

RULE-ASSUME-1 (All analysis assumptions must be documented and justified):
Per FM.A-3.11c / FM.A-4.17c / FM.A-5.13c: any assumption made in a formal analysis (e.g., "inputs stay within defined ranges", "clock skew is bounded", "no nonlinear arithmetic") must be explicitly documented. In ELAI terms: any WIRE-DARK[plan-id] allowlist entry and any gate that relies on a bounded-domain assumption must document the domain bound and how it is enforced.

RULE-DISCREPANCY-1 (Verification failures discovered during formal analysis must feed back to requirements):
Per FM.A-3.9 and FM.A-4.15: "Many of the properties had to be revised before they could be proved. Typically, these were due to omissions in the original requirements or oversights introduced by the informality of textual requirements. Each such discrepancy was explained and fed back into the safety assessment process for review." In ELAI terms: a harness gate that fails during a formal run is not evidence of an implementation bug until the corresponding specification has been reviewed — the spec may be the error source.


Critical testable assertions (Python syntax, derived from report invariants):

assert theorem_proving_objectives_covered <= {"A-3.1","A-3.2","A-3.4","A-3.6","A-3.7","FM.A-3.8","FM.A-3.9","FM.A-3.10","FM.A-3.11"}, "TP must not claim A-3.3 (hardware compat) - not addressed"
assert model_checking_objectives_covered <= {"A-4.1","A-4.2","A-4.4","A-4.5","A-4.6","A-4.7","A-4.8","A-4.9","A-4.11","A-4.12","FM.A-4.14","FM.A-4.15","FM.A-4.16","FM.A-4.17"}, "MC must not claim A-4.3/A-4.10/A-4.13 - not addressed"
assert abstract_interp_full_credit_objectives <= {"FM.A-5.10","FM.A-5.11","FM.A-5.12","FM.A-5.13"}, "AI gives full credit only to FM objectives; A-5.x are at most partial"
assert ai_objective_A56_credit == "PARTIAL", "A-5.6 is PARTIAL via abstract interp, not FULL - behavioral compliance not established"
assert all(gate.evidence_artifact is not None for gate in elai_gates), "every gate must produce an artifact; no gate is satisfied by procedure alone"
assert all(gate.soundness_citation is not None for gate in elai_formal_gates), "soundness of each formal method must be cited, not assumed"
assert all(gate.assumptions_documented for gate in elai_formal_gates), "all analysis assumptions must be documented and justified"
assert indeterminate_results_count == 0 or all(r.has_alternative_verification for r in indeterminate_results), "indeterminate results are failures unless covered by alternative method"
assert all(discrepancy.fed_back_to_spec for discrepancy in verification_discrepancies), "property failures during verification must feed back to requirements review, not only to implementation"
assert all(axiom_set.has_consistency_proof for axiom_set in formal_specs_using_axioms), "axiom-based specs must prove consistency by exhibiting a satisfying implementation"


Known implementation pitfalls (do NOT do this):

Do not claim A-5.1 (source code complies with LLR) via abstract interpretation alone - it is explicitly NOT ADDRESSED; abstract interpretation only addresses non-functional runtime properties (A-5.6 partially).
Do not treat model checking output as independently checkable - it is not; the model checker must be qualified (tool qualification required, TQL-5 minimum) before its output counts as certification evidence.
Do not use axioms without proving consistency - "inconsistent axioms introduce unsoundness into a proof system allowing false theorems to be proven" (p.81).
Do not conflate source code verification with EOC verification - EOC objectives (FM.A-6) require separate proof of source-to-EOC equivalence per DO-333 section 6.7.f.
Do not treat a coverage test as equivalent to a model-checked property - FM.A-7 (coverage replacement) is a separate DO-333 table not addressed in any of these case studies.
Do not skip the discrepancy feed-back loop - properties that fail during formal verification may indicate requirement errors, not only implementation errors; the finding must go back to the safety assessment process.
Do not claim full credit for A-3.5/A-4.5/A-4.6/A-4.12 from formal methods alone - these are at most partial; additional review is always required.
Do not treat an indeterminate abstract interpretation result as a pass - it is a failure requiring alternative verification or explicit justification.
Do not omit input range assumptions from documentation - "proof of the absence of over/underflow holds only if the program inputs stay within defined ranges; these bounds are assumptions that must be documented" (p.167).
