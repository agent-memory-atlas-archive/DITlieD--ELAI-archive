GROUNDING SWEEP -- phase H-27
Source: F:\.ELAI_workflow\docs\thefinalglance\H-plan\H-27_gate_self_validation.md
Date: 2026-06-05
FSM: PLANNING | Session: s-8404 lineage

SPEC REFERENCES

_grounding/H-TRUST-GROUNDING-MAP.md TS-4 TS-7 NN-11 TD-3 TD-4 | GROUNDED | lines 62-148 contain TS-4 (mutation+metamorphic shipped C-28/C-29), TS-7 (DGM frozen-key substrate), NN-11 (gate self-validation harness ABSENT; BUILD), TD-3 (mechanical only, no LLM verdict), TD-4 (verify-the-verifiers, kill 100% + frozen corpus) — all on-topic

design T-11 (trust-hardening-design.md) | VERIFY_NEEDED | H-TRUST-GROUNDING-MAP line 67 assigns T-11 = "verify-the-verifiers / gate self-validation" (from a SEPARATE T-1..T-13 numbering in D:\.REGRET_workflow\memory-bank\plans\elai-trust-hardening-design.md). testing-trust-design.txt line 1466 assigns T-11 = "Rulebreakers Paired-Accuracy tau runner." Two separate T-N numbering systems; the plan intends the trust-hardening design T-11, not testing-trust-design.txt T-11. Not a mismatch, but the ambiguity should be resolved (cite file path, not just "T-11").

design D5 + TB-3 | GROUNDED | H-TRUST-GROUNDING-MAP lines 148/124-125: D5 = verify-the-verifiers; TB-3 = self-improving optimizer exploiting mis-spec'd gate; H-0_INDEX.md line 302 "escaped-mutant counter (design D5; the REGRET symmetric-vs-one-sided-band incident — H-27 enforces this)" — on-topic

mutation_testing.rs (MutationRunner + mutation_score_pct + MutantOutcome) | GROUNDED | grep-confirmed: MutationRunner trait at :354, mutation_score_pct field at :112, MutantOutcome enum at :64 — C-28 shipped, on-topic

metamorphic_testing.rs:60-130 (RelationLibrary + parse_relation) | GROUNDED | Read confirmed: parse_relation at :77-84, RelationLibrary struct at :181, MetamorphicTestRunner at :241 — C-29 shipped; only 3 relations (Identity, Inverse, PermutationInvariance); NullBandSidedness ABSENT (NN-11, BUILD)

trap_tasks.rs:120-270 (TrapTask + validate_for_traps + build registry) | VERIFY_NEEDED | TrapTask struct confirmed at :184. HOWEVER: the function at line 124 is score_candidate_for_traps (NOT validate_for_traps). validate_for_traps does NOT exist (grep returned no matches). fake_test_log_trap at :244 is inside #[cfg(test)], not a public production fn. The plan cites "validate_for_traps :124" but that function name does not exist — this is an imprecise line citation. The trap registry functionality (TrapTaskRegistry, TrapTask, score_candidate_for_traps) is present; the H-27 plan should consume the correct API name.

lifecycle.rs:850-900 (SHADOW->ACTIVE promotion + mutable=0 path) | VERIFY_NEEDED | Production fn transition_to_active is at :466 (not in the 850-900 range). The range 850-900 contains the TEST function active_mutable0_ccr_exempt that demonstrates the path; the mutable=0 production logic is at :490-500 inside transition_to_active. The cited line 871 is a test assertion (transition_to_active call in test). The plan states H-27 will "hook" this production function — that is correct. The line reference is imprecise but not contradictory; the hook target is transition_to_active at :466.

H-22/H-26 (gates whose MRs/bands H-27 mutation-tests) | GROUNDED | H-0_INDEX.md lines 302, 468-699 confirm H-22 = trading null gate, H-26 = shadow-exec financial-invariant MR; both in CLUSTER F; RAG (h22-trading-null-test-gate) confirms H-22 DONE with trading_null_gate.rs evaluate_trading_null

CLAIM CLASSIFICATION

MutationRunner + mutation_score_pct + MutantOutcome SHIPPED (C-28) | GROUNDED | mutation_testing.rs:354/112/64 confirmed by Read

RelationLibrary + parse_relation + MetamorphicTestRunner SHIPPED (C-29) | GROUNDED | metamorphic_testing.rs:181/77/241 confirmed by Read

trap_tasks.rs TrapTask SHIPPED (F-11) at :184 | GROUNDED | trap_tasks.rs:184 TrapTask struct confirmed by Read

validate_for_traps exists at trap_tasks.rs:124 | UNGROUNDED | Function does not exist; the function at :123 is score_candidate_for_traps. This is a name error in the plan and WIRING_CONTRACT.

lifecycle.rs:871 is the SHADOW->ACTIVE production hook | VERIFY | Line 871 is a test call inside active_mutable0_ccr_exempt (#[cfg(test)]). Production hook is transition_to_active at :466. The hook will be added to that production function, not at :871.

gate self-validation harness ABSENT (NN-11) | GROUNDED | No gate_self_validation.rs in .elai_cc/crates/orchestrator/src/; grep for validate_gate_revision returned no matches — correctly classified as BUILD

All 8 Track-3 gate evaluate fns exist as live files | GROUNDED | deflation_gate.rs:evaluate_deflation, leakage_gate.rs:evaluate_leakage, lookahead_differential.rs:diff_future_truncation, trading_null_gate.rs:evaluate_trading_null, rerun_equality.rs:verify_rerun_digest, sim_real_parity.rs:verify_replay_golden, result_manifest.rs:verify_result, shadow_exec.rs:diff_shadow_impls — all confirmed by glob + bash grep

Papadakis 2019 mutation testing survey | GROUNDED | RAG (paper-agentless bib.bib77): "Mutation testing advances: an analysis and survey" Papadakis et al. 2019, Advances in Computers Vol. 112 — title matches cite

Segura 2016 metamorphic testing survey | GROUNDED | RAG (h22-trading-null-test-gate): "Metamorphic-testing ref = Segura et al. 2016 IEEE TSE survey" — confirmed as H-22 grounding (same project, same session)

Barr 2015 oracle problem | INGEST | Not in local paper collection. No arXiv ID cited. Claim is load-bearing (motivates why external test oracles beat agent-written oracles). Paper: "The Oracle Problem in Software Testing: A Survey" Barr, Harman, et al. 2015 IEEE TSE — verify before EXECUTING.

Inozemtseva & Holmes ICSE'14 (coverage != effectiveness) | INGEST | Not in local paper collection. No arXiv ID cited. Claim is load-bearing (motivates why 100% corpus coverage does not equal gate correctness). Paper: "Coverage Is Not Strongly Correlated with Test Suite Effectiveness" ICSE 2014 — verify before EXECUTING.

TD-3 mechanical scoring no LLM verdict | GROUNDED | H-TRUST-GROUNDING-MAP:142-145 defines TD-3 explicitly; H-27 plan line 16 states "(no LLM verdict — TD-3)" consistently

TD-4 / verifier-of-verifiers infinite-regress | GROUNDED | H-TRUST-GROUNDING-MAP:145-148 correctly frames TD-4: "Each PROPOSED->ACTIVE gate revision must kill 100% of FROZEN out-of-band mutant corpus." H-0_INDEX.md:769 grounds the bootstrap honest-residue at the "top verifier" as a deferred but acknowledged gap (plan lines 169-171 document the bootstrap). The framing is correct: the top validator is itself unverified; that is irreducible and routed to a frozen/human-authored corpus (R8/TD-2). No claim that the regress is fully resolved.

NullBandSidedness as mandatory MR (NEW) | VERIFY | MetamorphicRelation enum has only 3 variants (confirmed :47-57 in metamorphic_testing.rs); NullBandSidedness absent — correctly classified as BUILD; plan's claim that it is currently absent is correct

REGRET GBM symmetric-band incident as motivating defect | GROUNDED | H-0_INDEX.md:302 "REGRET GBM symmetric-vs-one-sided-band incident"; H-TRUST-GROUNDING-MAP:148 same reference — project-internal ground truth, no external citation needed

blake3 only, no sha2/md5 | GROUNDED | CLAUDE.md HARD RULE Anti-Stale-Training (verified by file content)

SCHEMA_VERSION baseline+1 from LIVE 207/484 | VERIFY | H-TRUST-GROUNDING-MAP:18-19 says SCHEMA_VERSION=207/TOTAL_VARIANTS=484 as of 2026-06-04. RAG (H-plan-H18-to-H25-shipped) shows H-22 bumped to 227/504 and H-25 to 230/507. The baseline in the plan (207/484) is STALE — at minimum H-17..H-25 have shipped this session advancing the baseline. The plan's baseline+1 instruction is correct in shape but the starting integer is stale; executor must re-read ssb.rs at edit time (plan itself says "re-read + bump e45:294" — the shape is correct, the stored number is stale by at least 23 units).

ACTIONS REQUIRED

- verify validate_for_traps function name: function does not exist in trap_tasks.rs; correct API is score_candidate_for_traps (:123) or TrapTaskRegistry methods; update WIRING_CONTRACT consumes_symbols and Task 2 before EXECUTING
- ingest paper Barr et al. 2015 "The Oracle Problem in Software Testing" IEEE TSE via paper-ingestor mode=paper; search term "oracle problem software testing Barr Harman IEEE TSE 2015" (no arXiv ID; may require mode=source with a DOI fetch)
- ingest paper Inozemtseva & Holmes ICSE 2014 "Coverage Is Not Strongly Correlated with Test Suite Effectiveness" via paper-ingestor mode=paper; search term "Inozemtseva Holmes ICSE 2014 coverage effectiveness mutation"
- verify T-11 ambiguity before EXECUTING: plan cites "design T-11" which maps to trust-hardening-design.md T-11 (verify-the-verifiers); testing-trust-design.txt T-11 is a DIFFERENT item (Rulebreakers tau runner); add explicit file path citation to remove ambiguity
- verify lifecycle.rs hook target: plan references "lifecycle.rs:871 mutable=0 path" but that line is a test call; production hook point is transition_to_active at :466; confirm the executor modifies :466 and adds validate_gate_revision as a called guard, not a hook at :871
- SCHEMA_VERSION baseline (207/484) is stale; H-22..H-25 shipped this session (SCHEMA now at least 230/507 per RAG); executor must re-read ssb.rs:const SCHEMA_VERSION at edit time before bumping (plan shape correct, stored number wrong)

VALIDATION WARNINGS

- no spec references block in H-27 using the "Spec references:" label; references appear under "Reads (mandatory before coding)" — verified against that section instead; all mandatory reads verified above
- T-11 label collision: trust-hardening-design.md T-11 = verify-the-verifiers (matches H-27 intent); testing-trust-design.txt T-11 = Rulebreakers tau runner (different item); two separate T-N numbering systems in the codebase — resolve at planning time
- validate_for_traps name error is a non-blocking VERIFY_NEEDED but will cause a compile error if the executor uses that symbol name
- SCHEMA_VERSION staleness is structural (the grounding map says "re-read at edit time") — not a block, a reminder
- all sources validated (all mandatory read files exist on disk)
