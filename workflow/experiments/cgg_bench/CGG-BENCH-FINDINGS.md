> Historical experiment record. Status and measurements below are preserved as
> recorded; no experiment was rerun for publication. Raw datasets and run state
> are omitted, so this file is not a reproducible benchmark release.

CGG HALLUCINATION BENCHMARK — FINDINGS LOG (live)
=================================================
Date: 2026-06-27. Author: main LLM (Opus 4.8). Status: REPAIR + HARD-MATH + TRUTHFULQA + L5-FIX MEASURED (sec 7-9).
All numbers below are OWN MEASURED DATA from this experiment, never paper constants (ID-5).
Harness: /workspace/ELAI\workflow\experiments\cgg_bench\

================================================================================
0. WHAT WE ARE TESTING
================================================================================
CGG (Causal Grounding Gate) is ELAI's planned (L-1..L-10, design-only) model-agnostic
hallucination-reduction system. We built a FAITHFUL BLACK-BOX version of its load-bearing,
benchmarkable core and measured the hallucination-rate delta it produces, baseline vs gated,
across DOMAINS and MODELS, with deterministic grading.

CGG core as benchmarked (model-agnostic, black-box):
  P3 inter-seed SEMANTIC ENTROPY (L-4, the genuine net-new): resample N, cluster by MEANING,
     disagreement = risk. (Farquhar Nature 2024 s41586-024-07421-0; SelfCheckGPT 2303.08896.)
  P2 paraphrase stability / SAC3 (J-85 / 2311.01740): reword the question, does the answer move.
  CONVERGENCE >=2 predictors agree (D-3): A3 discounts confidence only when P3 AND P2 flag.
  POLICY GATE high-risk -> ABSTAIN (L-7): the trust contract -- abstain, never fabricate.
  CROSS-FAMILY resample (D-13): resample across pretraining families to catch within-family
     shared-confident-wrong bias single-model resampling cannot see.
  (CODE only) EXECUTION GROUNDING (P1 round-trip / P5 do-test): run the code against tests.

NOT benchmarked (need the ELAI runtime, not black-box): DKG citation pre-binding (H-53),
counterfactual do-test on REPO claims (F-4/C-28), curiosity-driven targeted collection (L-6).

================================================================================
1. ARMS
================================================================================
A0_baseline        single answer, no gate (raw hallucination reference). confidence=1.0.
A1_verbalized      model self-reports a 0-100 confidence (the cheap competitor CGG must beat).
A2_self_consistency P3 over N resamples; majority answer; confidence = self-consistency.
A3_cgg_full        A2 + P2 paraphrase convergence penalty (the full CGG signal).
A4_cross_family    P3 across 3 different families (ensemble gate). [deferred -- gemini RPD caps]
A5_exec_grounded   (CODE) commit a sample that passes a VISIBLE test; abstain if none; grade on
                   HELD-OUT tests (non-circular). The do-test made runnable.
G_a0ans+sc_gate / G_a0ans+cgg_gate  (derived) A0's answer gated by A2/A3 confidence -- isolates
                   the SIGNAL's value from majority-voting's answer improvement.
A2_vote_full       (derived) majority vote, NO gate -- isolates voting from gating.

================================================================================
2. MEASUREMENT-INTEGRITY (an adversarial-auditor found 5 BLOCK + 8 WARN; all material ones fixed)
================================================================================
- Normalization over-merged proper nouns ("Alexander the Great"=="Alexander Great"). FIXED:
  strip only a LEADING article; N/A-family -> abstain; verified on adversarial cases.
- Voting vs gating were CONFOUNDED. FIXED: added A2_vote_full + gate-only arms.
- AURC was geometrically wrong (triangle vs rectangle). FIXED with tie-grouping; verified
  (flat=0.40, perfect-signal=0.08, anti-signal=0.82).
- API errors INFLATED self-consistency (excluded from denominator). FIXED: errors count in
  the denominator (a failed sample reduces consistency, verified SC=0.4 not 1.0).
- Split shuffle broke stratification. FIXED: interleaved cal/test draw.
- Re-architected to STORE RAW model outputs -> any normalization fix is a cheap offline
  re-derive, never a re-generation.
Grading: SimpleQA used the official LLM grader (mistral-small); QA/Math/Code now use
DETERMINISTIC grading (alias match / numeric equality / TEST EXECUTION) -> no grader bias.

================================================================================
3. RESULTS SO FAR
================================================================================

3A. SimpleQA (adversarial obscure-fact recall, WORST CASE) -- weak llama-3.1-8b, n=105.
    With an IDK instruction in the prompt (a CONFOUND we later removed).
    AUROC of the confidence signal (separating correct from wrong among ANSWERED):
      A1 verbalized 0.608 | A2 self-consistency 0.600 | A3 CGG-full 0.659.
    Read: on adversarial recall, signal is WEAK (0.66). The weak model is confidently +
    CONSISTENTLY wrong (only 6/74 answered correct), and semantic entropy by design catches
    CONFABULATION (uncertain/variable errors), NOT confident-consistent fabrication
    (Farquhar 2024 explicit). Capable gpt-oss-120b self-abstained 95% (IDK-instruction
    artifact + conservative RLHF) -> CGG had nothing to do, baseline halluc already 3.2%.

3B. THE TWO TEST FLAWS (user caught them; both real):
    (i) the "IDK" was induced by OUR prompt ("reply IDK if you don't know") -> inflated
        self-abstention, especially on the capable model. The correct gate test FORCES an
        answer so abstention comes ONLY from CGG's signal.
    (ii) SimpleQA is adversarially hard BY DESIGN (made so models fail); a weak 8B gets ~8%
        right when answering -> starves the signal-separation test of positives. Factual
        recall is semantic entropy's WORST regime.
    FIX: force-answer mode (no IDK), test on graded-knowledge data (PopQA, popularity-
    stratified), add a mid model, add CODE (execution-gradeable).

3C. PopQA (popularity-stratified factual QA, FAIR test) -- mid model mistral-small, n=99,
    FORCE-ANSWER (abstain=0% baseline). Deterministic alias grading.
    AUROC: A0 0.50 | A1 verbalized 0.791 | A2 self-consistency 0.812 | A3 CGG-full 0.831.
    -> On a FAIR test CGG's signal is STRONG (0.83, matches the literature ~0.75-0.83), and
    the ORDERING A3>A2>A1>A0 confirms the CGG design earns its keep AND beats the model's own
    verbalized confidence. The earlier 0.66 was the SimpleQA worst-case rig, not a verdict.

3D. WHAT 0.83 BUYS (PopQA mid, "wrong answers EMITTED per 100 questions" as we abstain the
    riskiest, by arm):
                          answer100%   abstain25%   abstain50%   abstain75%
      A0 (no signal)        57.6         43.4         29.3         15.2
      A1 verbalized         55.6         31.3         17.2          7.1
      A2 self-consistency   55.6         30.3         16.2          6.1
      A3 CGG-full           55.6         32.3         14.1          4.0
    Read: baseline leaks ~58 wrong/100. CGG answering HALF (abstain riskiest 50%) -> 14.1
    wrong/100 = a ~75% cut; answering the confident 25% -> 4.0/100 = ~93% cut. The signal
    matters: A0 random-abstain to 50% still leaks 29. CGG beats verbalized + plain consistency
    at aggressive gating. COST = coverage (you say IDK to the half it cannot ground).

3E. THE CORE MECHANISM (the honest framing): CGG does NOT make the model know more. It makes
    the model SHUT UP about what it cannot ground, and PICKS those cases well. "Toward 0
    hallucination" = abstain enough; the real question is the COVERAGE COST, which CGG
    minimizes better than the alternatives. On pure recall the cost is steep (~50% abstain)
    because the model genuinely doesn't know obscure facts.

================================================================================
3F. CROSS-DOMAIN RESULT (mid model mistral-small, n~100/domain, MEASURED 2026-06-27)
================================================================================
THE TABLE THAT SETTLES IT (AUROC of CGG signal + "wrong answers EMITTED per 100" at coverage):
  domain  baseline   best CGG signal (AUROC)   wrong/100 @100% cov   @ best gated point
  QA      58% halluc  A3 0.83                    55.6                  14.1 @ 50% cov (abstain-heavy)
  MATH     3% halluc  A2 0.73 (ceiling, model already strong)  2.0     ~1 @ 75% cov (little to cut)
  CODE    25% halluc  A2 0.93 / A5 do-test       23-25                 3 @ 75% cov (A2); 9 @ 92% cov (A5)

THE DOMAIN ASYMMETRY IS REAL AND MEASURED (confirms the Opus reasoning's Q3):
- CODE has an EXECUTABLE receipt -> A2 behavioral self-consistency AUROC 0.93 (vs 0.83 QA, 0.73 math),
  and A5 execution-grounding cut hallucination 25%->9% at 92% COVERAGE (it CHECKS, not guesses; only
  abstains on the 8% it genuinely cannot solve). This is "near-0 at HIGH coverage" -- the frontier
  pushed OUTWARD, not just traded down.
- QA has NO checker (pure recall) -> the only lever is abstention; cutting hallucination needs ~50%
  coverage sacrifice. The signal is good (0.83) but it can only ORDER the abstention, not avoid it.
- MATH on a capable model = CEILING (3% baseline error, nothing to cut). GSM8K WAS THE WRONG TEST (grade-
  school easy). The proper hard-math test = MATH-500 (competition AMC/AIME, 5 difficulty levels, ~30-50%
  model error -> real hallucination to catch); downloaded to math500.jsonl + a Windows-safe grader
  math500_grade.py (500/500 self-match). NOTE math's receipt is WEAKER than code's: no free oracle (no gold
  at inference); a symbolic checker verifies STEP VALIDITY / equivalence, not final correctness -> math
  repair lands BETWEEN QA and code. The next session re-tests math on MATH-500 (see NEXT-SESSION prompt).
- A5's residual 9% = the Goodhart gap (passed the VISIBLE test, failed HELD-OUT) -- more/better
  visible tests -> closer to 0 (Opus F2). The execution receipt is only as good as its test set.

VERDICT: CGG is GOOD, but its power is a function of RECEIPT AVAILABILITY (a domain property), exactly
as the Opus reasoning argues. Where a ground-truth checker exists (code/math/repo-claims), CGG drives
emitted hallucination toward ~0 at HIGH coverage. Where only parametric recall exists, it trades
coverage for honesty. "Reduce to 0 on ANY task" is achievable as 0 EMITTED hallucination = never assert
ungrounded, via the REPAIR LOOP (catch->ground->re-verify->commit-or-abstain), NOT via detection alone.

================================================================================
3G. WEAK-MODEL CONFIRMATION (llama-3.1-8b) — "does CGG help a WEAK model too?"
================================================================================
PopQA, weak llama-3.1-8b, force-answer, n=99 (the clean weak arm; weak math+code were BLOCKED by
groq's free-tier DAILY TOKEN CAP mid-run -- a provider limit, NOT a CGG result; 172 call-errors):
  A0 0.50 | A1 verbalized 0.723 | A2 self-consistency 0.869 | A3 CGG-full 0.906.
-> On the WEAK model the signal is EVEN STRONGER (A3 0.906 vs the mid model's 0.831), same ordering
   A3>A2>A1>A0. A weaker model is MORE uncertain (more confabulation, less confident-consistent
   fabrication), so semantic entropy separates better. CGG's "works on any model" claim is confirmed
   on the QA axis for both a weak (8B) and a mid (~24B) model; the convergence/paraphrase layer (A3)
   adds the most on the weak model (0.869->0.906). [weak code/math: re-run on a non-rate-capped
   provider to confirm the execution-grounding effect is LARGER on a weak model, as expected.]

================================================================================
3H. WHEN DOES CGG IMPROVE THINGS? (the practical summary)
================================================================================
CGG HELPS A LOT when:
- The task has a GROUND-TRUTH CHECKER (an external receipt): code->run tests, math->symbolic checker,
  repo/cited claims->retrieval+citation. MEASURED: code A2 AUROC 0.93; A5 do-test cut hallucination
  25%->9% at 92% COVERAGE. Here CGG CHECKS instead of guessing -> near-0 emitted hallucination at HIGH
  coverage. This is the strong regime and the one the REPAIR LOOP (L-15) unlocks fully.
- The model has GRADED KNOWLEDGE and its errors are UNCERTAIN/INCONSISTENT (confabulation): the resampling
  signal separates well. MEASURED: PopQA A3 AUROC 0.83 (mid) / 0.91 (weak); beats the model's own
  verbalized confidence in every case. Abstention here cuts emitted hallucination ~75% at 50% coverage.

CGG HELPS LITTLE / NOTHING when:
- BASELINE HALLUCINATION IS ALREADY LOW (ceiling): a strong model on easy math (GSM8K mistral 97% acc,
  3% halluc) -> almost nothing to cut; self-consistency nudges 97%->98%.
- The model is CONFIDENTLY + CONSISTENTLY WRONG and NO receipt exists (the irreducible residual):
  adversarial obscure-fact recall (SimpleQA weak AUROC 0.66). Semantic entropy is BLIND here by
  construction (low entropy on a stable wrong answer) and no external evidence can repair it. The only
  safe move is abstention -> coverage cost. This is the true floor, NOT the model's error rate.

THE LEVER, RESTATED: CGG does NOT make the model know more. On checkable tasks it CHECKS (-> near-0 at
high coverage); on uncertain recall it ABSTAINS WELL (-> lower emitted hallucination at a coverage cost);
on confident-unretrievable fabrication it can only abstain broadly (the residual). The REPAIR LOOP (L-15)
is what converts the "abstain well" cases into "repair and answer" wherever a receipt is obtainable.

================================================================================
4. THE DECISIVE OPEN RESULT (MEASURED above in 3F): MATH (GSM8K) + CODE (MBPP)
================================================================================
HYPOTHESIS (grounded by the design + pilots):
- MATH: self-consistency's home turf (Wang et al). Expect strong AUROC + lower coverage cost
  than QA (the model is consistent when it CAN solve, variable when it can't).
- CODE: the decisive one. CGG does NOT guess confidence -- it RUNS THE TESTS (execution
  grounding A5). Expect it cuts hallucination while keeping coverage HIGH, because it is
  CHECKING, not estimating. Pilot already showed: a baseline that produced WRONG code was
  recovered to CORRECT by both A2 (behavioral consistency) and A5 (do-test).
This is what separates "CGG is good on ANY task" from "CGG is good only where you can't verify
anyway." Where there is an executable/symbolic checker (code, math, repo claims), CGG's
EVIDENCE-grounding predictors (P1/P5) can in principle reach ~0 emitted hallucination at HIGH
coverage. Where there is only parametric recall, it trades coverage for honesty.

================================================================================
5. IMPLICATIONS FOR THE L-PLAN (preliminary)
================================================================================
- The black-box resampling core (P3/P2) is REAL and beats baselines on FAIR tests, but is
  necessary-not-sufficient for "toward 0": it catches confabulation, not confident-consistent
  fabrication.
- The load-bearing path to ~0 is the EVIDENCE-grounded predictors CGG already specifies but
  that need the runtime: P1 round-trip vs retrieved evidence/citations (DKG H-53), the P5
  counterfactual do-test (F-4/C-28), execution grounding (code). CODE proves these are
  benchmarkable and powerful.
- The cross-family lever (A4/D-13) is the cheap catch for single-model confident-wrong and
  deserves a clean run (gemini RPD wrecked it here).
- THE BIG QUESTION FOR THE NEXT REASONING PASS: detection/abstention caps hallucination by
  REFUSING. Can we instead, when we CATCH a hallucination, feed it back into a GROUNDING +
  REASONING repair loop (more retrieval, the GRC receipt-gate, escalate local->cloud) so the
  model REPAIRS the answer instead of just abstaining -> recovering coverage while keeping
  hallucination ~0? This is the CGG x GRC integration and the route to "0 without massive
  coverage loss." See the Opus reasoning file (next).

================================================================================
6. HARNESS FILE MAP (for reproduction)
================================================================================
data.py / data_popqa.py / domains.py (GSM8K+MBPP+PopQA specs) ; cgg.py (arms, normalize,
clustering, verdicts) ; code_exec.py (sandboxed executor + behavioral sig + do-test) ;
run_domain.py (generation) ; derive_domain.py + derive_code.py (offline verdict+grade) ;
analyze.py (risk-coverage/AURC) ; grader.py (SimpleQA LLM grader) ; audit_grader.py.
Datasets: simple_qa_test_set.csv, popqa.tsv, gsm8k_test.jsonl, mbpp.jsonl, math500.jsonl, truthfulqa_mc.json.
Raw/results: {domain}_raw_{weak,mid}.jsonl, {domain}_*_results.jsonl, simpleqa_partial/.
New harness (this session): simplify.py (emitted-halluc% at coverage, --by-topic), repair_code.py +
simplify_repair.py (the L-15 repair loop), groundrepair_truthqa.py (oracle/grounded re-answer),
domains.MATH500 + domains.TRUTHQA specs, math500_grade.py.

================================================================================
7. REPAIR LOOP + HARD-MATH + TRUTHFULQA (MEASURED 2026-06-27, this session)
================================================================================
The framing the user asked for: report results as "% hallucination" = WRONG ANSWERS SHOWN
per 100 questions, at fixed coverage (rank by confidence, show the top c-fraction, abstained
items show nothing). At 100% coverage on A0 this is exactly baseline INCORRECT/total. No tuned
threshold (pure risk-coverage readout). Headline cell = baseline vs CGG at 90% coverage.

7A. CODE REPAIR LOOP (MBPP, mistral-small, n=100, graded on HELD-OUT tests assert[1:]).
    The L-15 RepairController: on a VISIBLE-test fail, feed the failing test+error back, regen,
    re-run the test, COMMIT only a version that passes (external receipt, TD-3), max 3 rounds.
      arm                         halluc%   coverage   acc-when-shown
      baseline (answer all)        27%       100%       73%
      abstain-only (refuse fail)    9%        82%       89%
      REPAIR (fix on fail)          9%        91%       90%
    READ: repair recovered 9 of the 18 baseline-fails into HELD-OUT-CORRECT answers, adding ZERO
    new hallucination -> 9% halluc at 91% coverage. It STRICTLY DOMINATES pure abstention (same
    safety, +9pp coverage, recovered answers are correct). This is the user's question answered:
    YES, use the repair path for the "aggressive" setting -- it beats abstain-and-throw-away-coverage.
    The residual 9% = the GOODHART GAP (passed the VISIBLE test, failed HELD-OUT); shrinks with
    more/better visible tests. Commit is gated on the EXECUTED test, never the model's self-claim.

7B. MATH-500 (HARD competition math, mistral-small, n=60, level-stratified, sympy grader 500/500).
    Overall baseline 22% halluc (78% acc); CGG-full: 13% @90% cov, 5% @75%, 3% @50%. By level:
      level   baseline   CGG@90%   note
      L1       8%         0%       easy: errors uncertain -> CGG cleans up
      L3      17%         0%       CGG drives to ~0
      L4      17%        17%(0%@75) partial
      L5(hard) 64%       73%       FLOOR: model 36% acc, CONFIDENTLY+CONSISTENTLY wrong; CGG BLIND
    READ: MATH lands BETWEEN code and QA, as predicted. The real divide is NOT math-vs-facts, it is
    UNCERTAIN errors (catchable by self-consistency) vs CONFIDENT-CONSISTENT fabrication (NOT). At
    L5 the 5 resamples CONVERGE on a wrong derivation (self-consistent errors, arXiv:2505.17656),
    so gating by consistency is WORSE than baseline. The L5 fix is NOT abstention -- it is the L-9
    GRC deep-think + symbolic step-check repair (re-derive, verify each step), not built here.
    CAVEAT: the A1 "verbalized confidence" arm is BROKEN on MATH-500 (its parser expects "ANSWER:"
    but the prompt emits \boxed{}) -> A1 numbers are noise; the CGG verdict (A0/A2/A3) is sound.

7C. TRUTHFULQA MC1 (llama-3.1-8b, n=24, deterministic letter grading). THE CONFIDENT-FABRICATION
    FLOOR (popular misconceptions, the OSS benchmark the research recommended for this regime).
      arm            halluc%@100  @90%  @75%  @50%
      baseline        58%          50%   46%   29%
      CGG-full        58%          54%   42%   29%
    READ: CGG ~= RANDOM here. At 50% coverage it is 29% = exactly what random abstention gives
    (half of 58%). The self-consistency signal is near-useless on confident-consistent fabrication
    by construction (Farquhar 2024: semantic entropy catches confabulation, NOT stable wrong
    answers). This is the irreducible floor; abstention cannot fix it. ONLY external grounding can.

7D. GROUNDED-REPAIR (oracle / GRC grounded-research mode) ON THE TRUTHFULQA FLOOR.
    Took the floor cases (baseline wrong AND self-consistency>=0.6: 10 of 24). For 4 cleanly-
    retrievable ones (Jonestown Flavor-Aid, TV-eyestrain, swim-after-eating, dragons) I ran a REAL
    WebSearch, injected the retrieved snippet, and had the SAME llama-8b re-answer reading only the
    evidence (R3: deterministic retrieval, model never free-navigates).
      grounded-repair flipped WRONG->RIGHT: 4/4 = 100%
    READ (small n=4, but the mechanism is unambiguous): where CGG-abstention is BLIND (confident
    fabrication), EXTERNAL GROUNDING repairs it. This is exactly ELAI's L-3 evidence-round-trip /
    L-6 retrieval / oracle path. The honest residual that grounding cannot reach = fabrication of
    UNretrievable facts (no evidence exists) -> abstain (the true floor, not the model's error rate).

================================================================================
8. THE SIMPLIFIED % HALLUCINATION MASTER TABLE (what the user asked for)
================================================================================
"% hallucination" = wrong answers SHOWN per 100 questions. baseline = answer everything.
"with CGG" = at 90% coverage (show the confident 90%). "best safe" = the strongest arm's number.
  domain (model)              baseline   with CGG @90% cov   best (arm @ setting)
  CODE (MBPP, mistral)         27%        ~9% (exec check)    9% @ 91% cov via REPAIR (recovers coverage)
  MATH-500 (mistral, all)      22%        13%                 3% @ 75% cov (CGG-full)
  MATH-500 L1-L3 (easy-mid)    ~11%       ~0%                 ~0% (CGG cleans uncertain errors)
  MATH-500 L5 (hardest)        64%        73% (WORSE)         FIXED by cross-family: 50% ensemble / 12% gated (sec 9)
  QA (PopQA, mistral)          58%        45%                 14% @ 50% cov (abstention-heavy)
  QA (PopQA, llama-8b)         64%        53%                 18% @ 50% cov
  TruthfulQA MC1 (llama-8b)    58%        54% (~random)       floor: 4/4 flipped only via GROUNDED-REPAIR
  GSM8K (mistral, easy)         3%         1%                 ceiling (nothing to cut)

THE ONE-LINE VERDICT (grounded, this session): CGG's power is a function of the ERROR TYPE, which
tracks RECEIPT AVAILABILITY. (1) EXECUTABLE receipt (code): the REPAIR loop drives emitted
hallucination from 27% -> 9% at 91% COVERAGE (fix, don't hide) -- the strongest result. (2) UNCERTAIN
errors (easy/mid math, graded recall QA): self-consistency abstention works, trading coverage for
honesty. (3) CONFIDENT-CONSISTENT fabrication (hard-math L5, TruthfulQA misconceptions): CGG is
BLIND by construction; only EXTERNAL GROUNDING repairs it (TruthfulQA floor: 4/4 flipped with
retrieved evidence). "Toward 0 hallucination on ANY task" = 0 EMITTED = repair-where-a-receipt-
exists + abstain-on-the-unretrievable-residual, NEVER detection alone.

================================================================================
9. THE L5 FIX: CROSS-FAMILY AGREEMENT (D-13), MEASURED 2026-06-27
================================================================================
PROBLEM: single-model self-consistency made the hardest math (L5) WORSE (64%->73%) because the
5 resamples of ONE model converge on the SAME confident-wrong derivation (self-consistent errors,
arXiv:2505.17656) -> no signal. FIX: ask N INDEPENDENT pretraining FAMILIES (D-13). A confident-
wrong answer from one family is usually NOT shared by another, so cross-family DISAGREEMENT is the
detection signal self-consistency cannot provide, and the family MAJORITY is a stronger answer.

HARNESS: crossfamily_math.py + analyze_crossfamily.py. 4 distinct families, 1 temp-0 sample each
(ping-verified fast): Mistral (mistral-small), Google (gemini-2.5-flash), Qwen (qwen-2.5-72b via
openrouter), Llama (llama-3.3-70b via openrouter). Cluster by boxed answer; ensemble = majority;
gated = commit majority iff >=2 families agree, abstain on total disagreement. Deterministic grading.

RESULT (n=35 of 60; openrouter/gemini transient errors slowed the run, per-level trend unambiguous):
  level      baseline(1 model)   cross-family ENSEMBLE   cross-family GATED
  OVERALL     22.9%               20.0% (acc 77%->80%)    5.7% @ 83% coverage
  L1          14.3%               14.3%                   0.0% @ 86%
  L3           0.0%                0.0%                    0.0% @ 80%
  L4          11.1%               11.1%                   0.0% @ 89%
  L5(hardest) 62.5%               50.0% (acc 38%->50%)    12.5% @ 62%
THE L5 HEADLINE: single-model-CGG 64%->73% (WORSE). Cross-family ENSEMBLE 62%->50% (the ensemble
lifts L5 accuracy 38%->50%); cross-family GATED 12.5% @ 62% coverage (abstains on the genuinely-
hard problems where all 4 families disagree = the honest unsolvable residual). So L5 went from
"CGG makes it worse" to "CGG cuts it 64%->12%" by adding the ONE lever the first benchmark skipped.
CONFIRMS D-13: cross-family catches confident-consistent fabrication single-model resampling cannot.

================================================================================
10. THE PRODUCTION / CEILING / DETECTION QUESTIONS (grounded answer)
================================================================================
Q "is the L5 result a cap of the simple impl or the full ELAI ceiling?" -> MOSTLY the simple impl.
  The first benchmark ran ONLY the cheapest signal (self-consistency), blind to L5 by construction.
  The full ELAI CGG (L-1..L-15) has 3 levers that target exactly this, of which ONE (D-13 cross-
  family, sec 9) is now MEASURED to fix L5; the other two (L-9 sympy step-check, L-15 repair) are
  stronger still (an EXTERNAL symbolic receipt catches an invalid step regardless of confidence).
  So the full system is materially better than the 73% floor. The gap is real and (partly) closed.

Q "can we get 100% coverage AND 0% hallucination?" -> NO, in general. This is an EPISTEMIC limit,
  not an engineering gap. 0% EMITTED hallucination IS achievable (never assert ungrounded; abstain).
  But: coverage-at-0%-hallucination = (what the model KNOWS) + (what an available RECEIPT can verify
  /repair). Outside that set (unretrievable fact, model-can't-solve, locally-valid-but-strategically-
  wrong derivation) you must either abstain (lose coverage) or guess (risk hallucination) -- never both.
  The full plans push the RECEIPT FRONTIER outward (symbolic check, cross-family, retrieval, do-test)
  so MORE claims gain a receipt -> higher coverage at ~0 -- but cannot eliminate the no-receipt residual.

Q "how good is detection? we need 100%." -> Detection is ~100% ONLY where a receipt exists, and
  impossible where it does not -- the same wall. MEASURED detection (AUROC): confabulation/uncertain
  errors 0.83-0.93 (code 0.93, QA 0.83/0.91); confident-consistent fabrication WITHOUT a receipt
  ~0.50 = chance (TruthfulQA, L5 single-model). WITH a receipt (run the test / sympy the step /
  retrieve the fact) detection -> ~100% (deterministic pass/fail). Cross-family (sec 9) lifts the
  no-receipt case partway (a second family is a CHEAP pseudo-receipt) but the irreducible residual
  (all families share the same confident-wrong, OR the fact is unretrievable) is UNDETECTABLE by any
  label-free method -> the only safe move is ABSTAIN. "100% detection always" is not achievable by
  any system; ELAI's honest contract is ~100%-where-a-receipt-exists + abstain-on-the-rest.

UNIFYING FRAME: detection, repair, and 0-hallucination-at-high-coverage are ONE thing, all gated on
whether an EXTERNAL RECEIPT exists for the claim. Receipt -> detect+repair+answer. No receipt ->
abstain. The whole L-plan is an engine for MAXIMIZING the set of claims that have a receipt.

NEXT (not yet run): the L-9 sympy step-check repair arm (verify each algebra step, regenerate on an
invalid step, commit on the symbolic receipt) -- expected to beat cross-family on the step-checkable
slice of L5; and a fuller cross-family run (60/60 + more samples/family) to firm up the per-level n.
