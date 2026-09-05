L-15 — RECEIPT-MINTING REPAIR CONTROLLER + EVIDENCE-FIRST REORDER
================================================================
Nature: BUILD [KEYSTONE, net-new loop object] + binding REVISIONS to L-2/L-3/L-4/L-6/L-10 ordering.
Lever: turn every High/Critical ClaimRisk from a STOP (abstain) into a bounded REPAIR loop that mints
a passing external receipt before committing — recovering the coverage pure abstention throws away.
Wave: W3.5 (after L-2 the fuse exists; composes the W2 predictors + the W3 router/gate).
Composes-with: L-2 ClaimRisk, L-6 curiosity router, L-7 policy gate, J-83 BeliefState receipt gate,
  J-84 effort classifier, J-85 overthinking stop, J-86 receipt-gated candidate ladder, F-4 cant-lie,
  D-11 constructive checkers, D-13 cross-family adversary, I-7 escalation, HippoRAG (hippo_rag.rs),
  J-38 retrieval loop, J-103 read-ladder, H-53 citation pre-bind, B-22 AutonomousGate, H-39 abstention.
Authored 2026-06-27, main LLM in-band, after the CGG benchmark RAN (the first MEASURED grounding the
original L-1..L-10 did not have). New-plan-file per the house rule (old L-N files are not amended in
place); the REVISIONS below are binding on the named units and registered in L-0_INDEX.

ORIGIN — WHY THIS FILE EXISTS (the measured trigger)
====================================================
The CGG benchmark (workflow/experiments/cgg_bench/CGG-BENCH-FINDINGS.md + the Opus reasoning
workflow/reasoning/cgg-zero-hallucination-opus-reasoning-2026-06-27.md) measured the original
L-plan's core assumption and found it HALF RIGHT:
- MEASURED: CGG's black-box resampling signal (L-4 semantic entropy + L-2 convergence, "A3") is REAL
  and beats the model's own confidence on a FAIR test — PopQA AUROC 0.83 (A3) > 0.81 (A2) > 0.79
  (A1 verbalized) > 0.50 (A0). On a weak model / adversarial recall it is WEAK (SimpleQA 0.66).
- MEASURED: detection+abstention is a CEILING. It cut emitted hallucination ~75% but ONLY by
  sacrificing ~50% coverage. Abstention removes wrong answers without producing right ones.
- MEASURED (the decisive one): where an EXECUTABLE receipt exists (code), CGG drives hallucination
  toward 0 at HIGH coverage. MBPP, mid model: behavioral self-consistency AUROC 0.93; the
  execution-grounded do-test arm (A5) cut hallucination 25% -> 9% at 92% COVERAGE (it CHECKS, not
  guesses; abstains only on the 8% it genuinely cannot solve). Math on a strong model = ceiling
  (3% baseline, nothing to cut); the symbolic checker is the lever, not resampling.
CONCLUSION the data forces: the power of CGG is a function of RECEIPT AVAILABILITY (a DOMAIN
property), and the route to near-0 EMITTED hallucination at HIGH coverage is NOT more detection — it
is a REPAIR LOOP that spends bounded effort to MINT a passing receipt, gated by J-83. The original
L-plan has ~90% of the parts (L-2 scores, L-6 routes, L-7 gates, J-83 carries) but the LOOP that
iterates remedy -> re-verify -> commit-or-abstain is IMPLICIT across four files. L-15 makes it ONE
named object and reorders the build toward the evidence-grounded predictors the data proved load-bearing.
[WEAK-MODEL CONFIRMATION (2026-06-27): QA confirmed — weak llama-3.1-8b PopQA A3 AUROC 0.906 (EVEN
STRONGER than the mid model's 0.831; a weaker model confabulates more, so semantic entropy separates
better; same ordering A3>A2>A1>A0). Weak code/math were BLOCKED mid-run by groq's free-tier daily token
cap (a provider limit, not a CGG result) — re-run on a non-rate-capped provider to confirm the
execution-grounding effect is LARGER on a weak model. The verdict and R1-R5 do not depend on this.]

================================================================================
THE BUILD — L-15 RepairController (the net-new object; the user's core ask made concrete)
================================================================================
A single named control object that owns the catch -> ground -> re-verify -> commit-or-abstain loop.
State machine (S0-S6), every transition an SSB row (GD-2 journal-recomputable):

  S0 EXTRACT     consume L-1 ClaimSet {File,Symbol,Dependency,Api,Behavior,Causal,Reachability}.
  S1 SCORE       consume L-2 ClaimRisk + WHICH predictor fired (P1 round-trip / P2 oscillation /
                 P3 entropy / P-cf do-test). The fired-predictor IS the diagnosis.
  S2 DECIDE      L-7 maps tier->action, but High/Critical no longer means "abstain"; it means
                 ENTER REPAIR (route to S3). (L-7 already specifies High -> hand to L-6; L-15 closes
                 the loop L-6 only opened.)
  S3 DIAGNOSE->REMEDY   branch on (fired predictor x ClaimType), fire ONE targeted remedy (L-6), never
                 blanket re-read:
                   P1 residual high / unsupported claim -> DETERMINISTIC retrieval (HippoRAG PPR over
                     hippo_rag.rs + J-38 + J-103), H-53 citation pre-bind. NEVER model-driven nav (see
                     REVISION R3). The model only does the atomic entity read.
                   P3 entropy high / model uncertain -> DIVERSITY: D-13 cross-family adversary (the ONLY
                     remedy that touches the confident-consistent blind spot) + J-86 parallel candidate
                     width (depth on a stable-wrong chain entrenches it).
                   Behavior/Causal claim -> EXECUTION: L-5 do-test (F-4 blake3-bound outcome) or D-11
                     symbolic check (sympy/z3). A CHECK, not a guess. This is the A5 result generalized.
                   Dependency claim -> API_REGISTRY round-trip (anti-slopsquat, deterministic).
                   locally unresolvable -> I-7 local->cloud escalation.
  S4 RE-VERIFY   re-invoke L-2 on the claim WITH the new evidence -> a fresh receipt.
  S5 GATE        J-83 receipt gate: receipt PASSES -> S6 commit. Else if budget remains -> S3 next
                 remedy. Else -> abstain (L-7 Critical -> Abstain, credited by I-8/H-39 so honesty is
                 not penalised).
  S6 COMMIT-AND-FREEZE   J-83 commit-and-freeze: the grounded claim is immutable belief, safe to
                 compress, the chain cannot drift back off it. L-8 prunes the disproven claim.
BUDGET + STOP (the controller gains): J-84 sets HOW MUCH repair effort per claim (Low=ZERO; easy=serial
  depth; confident-consistent=parallel width+cross-family; very-hard=escalate-or-abstain). J-85 gives a
  deterministic drift-stop (if the chain is getting WORSE, stop+abstain, never spiral). THREE independent
  terminations: receipt PASS (success) | budget exhausted (abstain) | J-85 drift (abstain). Bounded.
TRUST INVARIANT (what makes this NOT Reflexion-style self-correction): the S5 commit is gated on an
  EXTERNAL receipt the model did NOT author (TD-3, RR-1, J-83 RD-1). The receipt is L-3 deterministic
  reconstruction / L-5 blake3-bound F-4 outcome / D-11 symbolic verdict — the LLM proposes, the checker
  adjudicates. A loop whose acceptance test is the model's own say-so re-injects hallucination at the
  verify step; this one cannot, by construction. THIS IS THE LOAD-BEARING DISTINCTION.

WIRING_CONTRACT
  produces:  +RepairLoopRun (Pattern-2; Kind: ReceiptMinted/BudgetExhaustedAbstain/DriftAbstain) OR
             reuse L-6 RetrievalLoopEvent + L-7 HallucinationGateAction rows (prefer-reuse; decide at
             edit, net 0..+1).
  consumes:  L-2 ClaimRisk row, L-6 remedy rows, the L-3/L-5/D-11 receipt, J-84 budget, J-85 stop.
  reachable_from:  the L-7 policy-gate seam in task_runner (where High/Critical is decided today) — the
             RepairController.run(claim, risk) call replaces the bare abstain branch. PRODUCTION_HOOK:
             the executor wires run() at that exact seam in the SAME changeset (D-030); off-agent
             production_caller_check.py confirms; default-OFF kill-switch ELAI_CGG_REPAIR_ENABLE with a
             wire-dark-allowlist row (type=experiment, owner, expiry, promotion_plan_id, W-2 not W-9... i
             mean W-2 kill-switch-not-enable-flag).
  observable_effect:  on a HARD claim with an obtainable receipt, a refused-then-repaired claim COMMITS
             with a passing receipt row that the model did not author; on an unretrievable claim it
             abstains with an honest-refusal row. Measured by REVISION R4 (H_emit at fixed coverage).
  trigger:   the L-10 bench --repair arm drives the real entrypoint on a held-out claim corpus.
  stub_ban:  NO model-driven multi-hop navigation in the repair retrieval (R3); NO model self-grading
             the receipt (TD-3); NO commit on absence-of-disagreement without a passing evidence-receipt
             for a receipt-available claim (R1).

================================================================================
BINDING REVISIONS TO L-T1 (the reorder + hardening the data forced; registered in L-0_INDEX)
================================================================================
R1 (amends L-2, HG-2 hardening). A Low/Allow verdict on a RECEIPT-AVAILABLE (class A/B/C) factual claim
   REQUIRES >=1 PASSING evidence-grounded receipt (P1 round-trip OR P-cf do-test), NOT merely the
   absence of a resampling flag. Absence-of-disagreement != presence-of-grounding. Two resampling-family
   predictors (P2 oscillation + P3 entropy) share the confident-consistent blind spot and can "agree" a
   stable-wrong claim is safe (the SimpleQA 0.66 failure class). Grounds in L-2's own open-question
   (predictor decorrelation, H-40). This is the single change that most moves H_emit on confident-
   consistent claims where a receipt exists.

R2 (reorders WAVE 2). Land L-3 (evidence round-trip) + L-5 (do-test) BEFORE L-4 (semantic entropy).
   MEASURED: black-box resampling (L-4) tops out at 0.83 (fair) / 0.66 (adversarial) — the CEILING.
   The EVIDENCE-grounded predictors (L-3/L-5) are what break it (code A5: 25%->9% at 92% cov; the only
   arm that cut hallucination WITHOUT trading coverage). L-4 is the genuine net-new SIGNAL but it is
   necessary-not-sufficient. New W2 order: L-3 + L-5 first, L-4 second (still parallel-safe, disjoint
   files; L-4 still needs L-2's fuse interface).

R3 (constrains L-6). The repair-retrieval MUST be DETERMINISTIC: BM25 / HippoRAG PPR (hippo_rag.rs) +
   atomic model entity-extraction only. NEVER model-driven free navigation in the repair step. Grounds
   in the RLM-vs-ELAI measurement (rlm-recursive-agents-grc PART 9): a deterministic graph-hop harness
   made a weak 8B hit 87% on 8xW multi-hop where model-driven navigation plateaued at 25% and free-code
   RLM HALLUCINATED ("42" on 7/8). A model-driven repair re-injects the failure it is meant to fix.
   Added as an L-6 stub_ban.

R4 (extends L-10 — the done-gate metric change). The done-gate is no longer gate-ON-vs-OFF abstention
   rate. ADD a third arm: gate+REPAIR vs gate-only vs off, measuring H_emit AT FIXED COVERAGE (the decisive
   number: "H_emit at 90% coverage with repair" vs "abstain-only at 90% coverage"). The whole thesis is
   recovering coverage; the bench must show repair holds H_emit low WITHOUT the coverage sacrifice.
   ELAI-measured (ID-5), journal-recomputable (GD-2), never a paper AUROC. The cgg_bench harness
   (workflow/experiments/cgg_bench) is the prototype of this arm (A5 vs A2-gate vs A0).
   R4 MEASURED 2026-06-27 (repair_code.py + simplify_repair.py, MBPP mistral-small n=100, graded on
   HELD-OUT asserts): baseline 27% H_emit @100% cov | abstain-only 9% @82% cov | REPAIR 9% @91% cov.
   The decisive cell CONFIRMS the thesis: repair holds H_emit = abstain-only's 9% WHILE recovering
   coverage 82%->91% (recovered 9/18 baseline-fails as held-out-CORRECT, zero new hallucination) ->
   repair STRICTLY DOMINATES abstention. Residual 9% = Goodhart gap (visible-pass, held-out-fail),
   shrinks with more visible tests. Commit gated on the EXECUTED test (TD-3), never the model's claim.
   CROSS-DOMAIN (same session, simplify.py): MATH-500 mistral 22%->13%@90%/3%@75% overall, but L5-hard
   64%->73% (CONFIDENT-CONSISTENT fabrication, CGG BLIND -> needs R5/L-9 step-check, not abstention);
   TruthfulQA MC1 llama-8b 58% and CGG ~= random (the floor), but GROUNDED-REPAIR (oracle re-answer on
   retrieved evidence, groundrepair_truthqa.py) flipped 4/4 floor cases wrong->right -> confirms L-3/L-6
   external grounding is the ONLY lever on confident fabrication. Full tables: CGG-BENCH-FINDINGS.md sec 7-8.
   L5-FIX MEASURED (D-13 cross-family, crossfamily_math.py, n=35, FINDINGS sec 9): single-model self-
   consistency made hardest-math L5 WORSE (64%->73%); 4-family agreement (Mistral/Google/Qwen/Llama)
   ENSEMBLE cut L5 64%->50% (acc 38%->50%) and GATED (abstain on family disagreement) cut it to 12.5%
   @62% cov; overall MATH-500 23%->5.7% @83% cov. CONFIRMS D-13 is the lever for confident-consistent
   fabrication that single-model resampling (L-4) is blind to -> WAVE-2 should land D-13 alongside L-4,
   not after. The unifying frame (validated by all arms): detection+repair+0-halluc are ONE thing gated
   on whether an EXTERNAL RECEIPT exists (execution/symbolic/retrieval/cross-family-pseudo-receipt); the
   no-receipt residual is undetectable by any label-free signal -> abstain. 100%-cov-at-0%-halluc is an
   epistemic impossibility; 0% EMITTED via repair-where-receipt + abstain-on-residual is the real target.

R5 (elevates L-9 to flagship). The research/math profile is the cleanest near-0-at-high-coverage demo:
   a wrong arithmetic step -> D-11 sympy catches it -> regenerate -> sympy confirms -> J-83 commit. Zero
   coverage cost on in-checker math (symbolic receipt = ground truth). It DEPENDS on L-15 (the loop) to
   repair-not-just-abstain. Prioritize as the legible showcase serving the user's research-grounding ask.

================================================================================
THE FRONTIER (what L-15 achieves, and the HONEST residual — GD-5)
================================================================================
Achievable H_emit=0 maximal coverage, per RECEIPT CLASS (measured/derived):
  A code   (executable receipt)  -> ~100% minus genuinely-unsolvable tasks. MEASURED proxy: A5 9%@92%cov,
                                    residual = the visible/held-out test gap (Goodhart; grade on HELD-OUT).
  B math   (symbolic receipt)    -> ~100% within checker coverage (algebra/logic/arith full); abstain outside.
  C repo/cited (retrieval receipt) -> near-100% on existence/location (the bulk of agent hallucination);
                                    partial on interpretation (caveat-not-commit).
  D parametric-recall-only (NO receipt) -> H_emit=0 ONLY by abstention; coverage = what the model knows.
                                    The IRREDUCIBLE residual. MEASURED: PopQA ~50% coverage cost, SimpleQA
                                    AUROC 0.66 (the signal is blind here). D-13 cross-family can DETECT but
                                    not SUPPLY the fact. NAMED honestly (GD-5, already an L-0 commitment).
The domain router (D-10) decides which class a claim is in -> decides whether you are in the near-0 regime
(A/B/C, repair) or the abstain-or-guess regime (D). That is why CGG is "good on ANY task" without being
"magic on every task": near-0 wherever a receipt exists, HONEST abstention where none does.

================================================================================
ACCEPTANCE (the FSM may not COMPLETE L-15 without)
================================================================================
1. RepairController.run() reached from the real L-7 policy-gate seam (production_caller_check.py PASS,
   D-030), kill-switch ELAI_CGG_REPAIR_ENABLE default-OFF + wire-dark-allowlist row (W-2).
2. The S5 commit is gated on an EXTERNAL receipt only (TD-3/RR-1 acceptance checkbox); a unit test that
   commits on the model's own say-so is a P0.
3. The loop TERMINATES on all three conditions (receipt-pass / budget / J-85 drift) — a test that drives
   each (R10 integration smoke on a HARD input: a multi-hop claim, a confident-consistent claim, an
   unretrievable claim).
4. R3 enforced: a test proving the repair retrieval used deterministic HippoRAG/BM25, not model nav.
5. R1 enforced: a test proving a class-A/B/C claim cannot get a Low verdict without a passing evidence
   receipt.
6. The L-10 --repair arm (R4) measures H_emit at fixed 90% coverage, ELAI-measured + journal-recomputable.
7. GD-4 observe-only first: the RepairController ships SHADOW (logs what it WOULD repair) until its
   over-repair/cost rate is bounded on >=3 real runs; promotes to active behind the kill-switch.
8. GD-5 residual named in-file (done above). workflow/reasoning/L-15-decisions.md exists. memory-writer ran.

HG/doctrine bindings: HG-1 (causal only on interventional do-test, L-5), HG-2+R1 (convergence requires an
evidence receipt), HG-3 (black-box — runs on every B-37 backend), HG-4 (abstain-or-verify-never-silently-
pass — the loop is exactly this). TD-3 / RR-1 / ID-5 / GD-2 / GD-5 restated as checkboxes above.

THE ONE-LINE THESIS: an abstention is a discarded opportunity to ground; the measured coverage loss is
recoverable for every claim with an obtainable receipt by turning each High flag into a targeted request
for evidence a bounded loop fulfils, then committing ONLY against a receipt the model did not author —
which collapses hallucination to one sharp boundary: receipt exists (drive to near-0 at high coverage) vs
no receipt obtainable (abstain). The missing object that realizes it is this RepairController.
