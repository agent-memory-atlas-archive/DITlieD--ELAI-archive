GROUNDING SWEEP -- phase F-7
Source: docs/thefinalglance/F-plan/F-7_safety_lattice_cbr_ranking.md
Secondary source: workflow/reasoning/VIBESEC-SHARD-STEAL-AND-BUILD-2026-06-01.md (sec 1.2, sec 4 C7, sec 5 TIER 3)
Date: 2026-06-02
FSM: PLANNING | Session: s-60824

SPEC REFERENCES
VIBESEC...md sec 1.2 (Springdrift correction) | GROUNDED | VIBESEC:83-88 "7,056 proposition pairs ... +10.7% P@4 ... VERDICT: OVERSTATED ... SYNTHETIC / n=1 self-reported / UNREPRODUCED ... overlaps the RAG baseline ... Treat ... as design hypotheses, never as targets"
VIBESEC...md sec 4 item C7 | GROUNDED | VIBESEC:389-409 REVISED SCOPE split: (1) lattice = AMENDMENT to E-53 strictly sequenced after stubs resolve; (2) CBR = standalone Laplace weighting over EXISTING SSB events; "DROP the +10.7% target, replace with a measured delta after >=100 labelled sessions"
VIBESEC...md sec 5 TIER 3 | GROUNDED | VIBESEC:457-461 "C7 split into an E-53 amendment + a standalone CBR-ranking plan"; sec 6 table:481 "C7 ... REVISE T3 (split)"; sec 7:496 "Do NOT import Springdrift's +10.7% / 7,056-pair numbers as targets"
orchestrator/src/formal_verify/orchestrator.rs:111-119 (z3_stub boundary) | GROUNDED | orchestrator.rs:111 `pub async fn run`; :117-119 `VerifierKind::Z3Smt => VerifierOutcome::Inapplicable { reason: "z3_stub" }`
orchestrator/src/formal_verify/types.rs:40-250 (VerifierKind/Outcome) | GROUNDED | types.rs:38 Z3Smt, :42 Dafny, :46 Kani, :49 Verus; :126 RunpodPending; :72 expected_local_install; :143 counts_toward_consensus ("Inapplicable + RunpodPending DO NOT" count)
rules/src/lifecycle.rs:120-130,480-560,580-600 (RuleRecord.mutable, read path) | GROUNDED | lifecycle.rs:126 `pub mutable: bool`; :277 `pub fn fetch_rule_record` (real read path); :551 immutable auto-retire guard; insert_rule@:589 is INSIDE #[cfg(test)] mod (W-7 correction is accurate)
compiled_pipeline.rs:280-400,452-540 (crystallizer + try_execute + first_try_success_rate) | GROUNDED | :280 ExecutionOutcome; :292 TemplateCrystallizer; :362-383 first_try_success_rate inline compute; :486 `pub async fn try_execute`; :499 selection uses first_try_success_rate
cli/src/formal_verify_cmd.rs:1-130 (CLI shape HALF A extends) | VERIFY_NEEDED | file exists per plan + main.rs:599 wiring claim; exact line range not opened this sweep (HALF A is RunPod-gated, low blast radius). Verify FormalVerifyAction enum shape at implementation time.
arXiv:2501.13956 Graphiti | GROUNDED-as-non-loadbearing | plan line 49 + 219 explicitly mark it "NOT load-bearing for F-7 (reference only)"; no F-7 mechanism depends on it
springdrift dossier (synthesis Pillar 5) | VERIFY_NEEDED | cited only for "git-whole-state framing"; its NUMBERS are SYNTHETIC per 1.2 and must not be load-bearing. Confirmed plan treats it as framing-only.

CLAIM CLASSIFICATION
Springdrift +10.7% P@4 (7,056-pair, 100% coverage) | UNGROUNDED | SYNTHETIC/n=1/UNREPRODUCED per VIBESEC:83-88. MUST NOT be an F-7 target/ceiling. Plan correctly DROPS it (lines 23, 177, 216, 225). It is a NON-target.
Lattice completeness/monotone/conflict-free as a SET | UNGROUNDED-as-hypothesis | DESIGN HYPOTHESIS per VIBESEC:88,398; E-53 contract tests cover rows not the set (lifecycle.rs:865 active_mutable0_ccr_exempt is per-row). Provable only once Z3 resolves; never asserted as a delivered guarantee.
CBR uplift over first_try_success_rate | UNGROUNDED-as-hypothesis | DESIGN HYPOTHESIS; the measurable is a MEASURED delta after >=100 labelled sessions (VIBESEC:407-408), NOT an imported number. No ELAI baseline exists today (plan RISK line 225).
Laplace-utility / add-one CBR ranking methodology | GROUNDED | standard add-one (Laplace/Lidstone) smoothing of frequency estimates; (successes+alpha)/(n+alpha+beta) is the textbook posterior-mean form. Methodology is sound prior-art, NOT a Springdrift number.
Laplace alpha=1.0, beta=1.0 constants | GROUNDED-ELAI-OWN | plan line 56 defines `LAPLACE_ALPHA/BETA: f32 = 1.0` as named ELAI constants; rationale at line 82,226 (uniform prior, smallest prior preventing 1/1 outranking 95/100). NOT imported from Springdrift; not a borrowed axiom constant.
laplace_utility outranks 95/100 over 1/1 | GROUNDED | arithmetic: with a=b=1, util(1/1)=2/3=0.667 < util(95/100)=96/102=0.941. Verifiable by unit assertion (plan acceptance line 176). Deterministic in-process.
blake3 for obligation hash (never blake2/sha2) | GROUNDED | CLAUDE.md HARD RULE Anti-Stale; plan line 68,151-153 enforces blake3 + a DETERMINISTIC_CHECK banning blake2/sha2.
E-53 Z3/Kani/Verus routes are Inapplicable stubs (RUNPOD-PENDING) | GROUNDED | orchestrator.rs:117-123 all four return Inapplicable {reason:"<v>_stub"}; types.rs:43,46 "RUNPOD-PENDING locally"
HALF A deferral sequenced AFTER E-53 z3/kani/verus stubs resolve | GROUNDED-OUT-OF-F7 | plan lines 5,8,20,179,224 + VIBESEC:404 "strictly sequenced after E-53's stubs resolve". The LIVE proof is a SEPARATE #[ignore]-gated RunPod checkbox; the completeness proof itself is correctly OUT of F-7's shippable scope.
RunpodPending returned (never fabricated Verified) when Z3 absent | GROUNDED | types.rs:126 RunpodPending variant; plan stub_ban line 123 + W-5/W-10 lines 239,244 forbid fabricated Verified
VerifierOutcome::RunpodPending does NOT count toward consensus | GROUNDED | types.rs:143 "Inapplicable + RunpodPending DO NOT" count_toward_consensus
fetch_rule_record is the real read path, insert_rule is test-only | GROUNDED | lifecycle.rs:277 pub fn fetch_rule_record; :589 fn insert_rule inside test mod (W-7 self-correction in plan line 241 is accurate)

ACTIONS REQUIRED
- UNGROUNDED Springdrift "+10.7% P@4 / 7,056-pair / 100% coverage": this is a NON-target and MUST be DROPPED, never cited as an F-7 target or ceiling. Plan ALREADY drops it correctly (lines 23, 177, 216, 225, 248) -- confirm no acceptance checkbox or DETERMINISTIC_CHECK re-introduces it as a numeric gate. CONFIRMED clean: F-7 gates on the WIRING done-gate + smoothing-correctness assertion, not a numeric uplift (line 225).
- UNGROUNDED-as-hypothesis lattice-completeness + CBR-uplift: keep labelled "DESIGN HYPOTHESIS"; HALF B measurable = MEASURED delta vs first_try_success_rate baseline after >=100 labelled sessions (do not assert a number). Plan compliant.
- CONFIRM HALF A (safety-lattice completeness proof) is correctly OUT of F-7's shippable scope: VERIFIED OUT. F-7 ships only (1) the lattice enumerator/encoder, (2) the deterministic fixture harness exercising the REAL run path with RunpodPending, (3) the safety_lattice_verified SSB variant + CLI. The actual completeness PROOF is an E-53 amendment, strictly sequenced AFTER E-53's z3/kani/verus stubs resolve, RUNPOD-PENDING, #[ignore]-gated. Correct.
- CONFIRM Laplace ranking constants are ELAI-own: VERIFIED. LAPLACE_ALPHA=1.0 / LAPLACE_BETA=1.0 are ELAI named constants with a stated uniform-prior rationale (plan line 56,82,226). NOT a Springdrift-imported axiom constant. The methodology (add-one smoothing over outcome utility) is a grounded standard technique.
- VERIFY at implementation time: open cli/src/formal_verify_cmd.rs:1-130 + args.rs to confirm FormalVerifyAction enum shape before adding the Lattice verb (HALF A; deferred, RunPod-gated).

VALIDATION WARNINGS
No phase-{X}.txt spec file is cited (F-7 is an F-plan unit grounded against the VIBESEC synthesis + ELAI source, not the guide/steps spec tree) -- expected for F-plan; not a traceability gap. All ELAI code anchors (orchestrator.rs:111-119, types.rs, lifecycle.rs:126/277, compiled_pipeline.rs:280-540) validated against live source 2026-06-02. Springdrift numbers correctly classified as NON-targets in every appearance.
