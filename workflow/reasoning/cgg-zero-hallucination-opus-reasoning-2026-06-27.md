CGG x GRC -> NEAR-ZERO EMITTED HALLUCINATION: deep reasoning (Opus 4.8, 1M)
Date: 2026-06-27. Author: Opus reasoning agent (read-only, in-band). Task: cgg-zero-reasoning.
Grounding posture: every ELAI claim is from a file read this session (CGG-BENCH-FINDINGS.md;
cgg-hallucination-causal-system-2026-06-27.md; rlm-recursive-agents-grc-2026-06-26.md;
L-plan L-0_INDEX/L-2/L-3/L-5/L-6/L-7/L-9). Every measured number is OWN benchmark data (ID-5),
never a paper constant. External mechanism cites are tagged [EXTERNAL/DESIGN]. Where I reason past
the evidence I label it [CONJECTURE].

================================================================================
0. THESIS (3 sentences)
================================================================================
Detection-plus-abstention is a CEILING, not a floor: the PopQA run measured CGG's black-box signal
at AUROC 0.83 and a ~75% cut in emitted hallucination at 50% coverage, but the cut is BOUGHT with
coverage because abstention removes wrong answers without producing right ones, and on adversarial
confident-consistent fabrication (SimpleQA AUROC 0.66) the signal is structurally near-blind.
The route to NEAR-ZERO EMITTED hallucination at HIGH coverage is to convert every High/Critical
ClaimRisk from a STOP into a REPAIR TRIGGER: a bounded, risk-targeted loop (L-6 router -> retrieve/
cite/execute/symbolic-check/escalate -> RE-VERIFY against an EXTERNAL RECEIPT -> commit iff the
receipt now PASSES, else abstain) that recovers the coverage pure abstention throws away, in exactly
the domains where an external ground-truth receipt exists (code, math, cited-claims, repo-claims).
This makes CGG and GRC ONE system: CGG sets the per-claim risk and the receipt REQUIREMENT, GRC is
the bounded-effort loop that spends J-84-budgeted reasoning/retrieval to MINT a passing receipt
before J-83's receipt gate commits the claim — so the irreducible residual is exactly "confident,
internally-consistent fabrication of a fact for which NO external receipt is obtainable," and nothing
else.

================================================================================
1. THE DETECTION CEILING (Q1) — why abstention alone cannot reach 0 without destroying coverage
================================================================================

DEFINE THE TARGETS PRECISELY (three distinct quantities, routinely conflated):
- H_emit  = P(assert a claim AND the claim is wrong). "Emitted hallucination rate."
- Cov     = P(assert a claim | a claim was requested). "Coverage."
- H_cond  = H_emit / Cov = P(wrong | asserted). "Conditional error among answered."
"0 hallucination" can only honestly mean H_emit -> 0. H_emit = 0 at Cov = 1.0 is IMPOSSIBLE for any
finite system: it would require knowing every requested fact, and no system conjures unknown facts
(CGG-BENCH §3E: "CGG does NOT make the model know more"). So the only coherent target is:
    minimize H_emit subject to maximizing Cov.
The whole game is the SHAPE of the H_emit-vs-Cov frontier, and detection+abstention can only move
DOWN the frontier (trade Cov for lower H_emit), never push the frontier itself OUTWARD.

WHY ABSTENTION IS A PURE TRADE, measured. PopQA mid (CGG-BENCH §3D): baseline leaks ~57.6 wrong/100
at full coverage; A3 CGG-full answering the confident HALF (abstain riskiest 50%) leaks 14.1/100
(~75% cut) and answering the confident 25% leaks 4.0/100 (~93% cut). Read the cost column: those
cuts are 50% and 75% coverage SACRIFICE. Every point of H_emit reduction came from REFUSING, not
from CORRECTING. The signal's value (A3 0.831 > A2 0.812 > A1 verbalized 0.791 > A0 0.50) only
buys a BETTER-ORDERED abstention (you drop the right cases first); it never recovers a refused case.

WHY THE CEILING IS HARD, not soft — the confident-consistent blind spot. Semantic entropy (L-4,
SelfCheckGPT/Farquhar [EXTERNAL/DESIGN]) detects DISAGREEMENT across resamples. It is by construction
a CONFABULATION detector: it fires when the model is UNCERTAIN (samples diverge in meaning). When the
model is CONFIDENTLY and CONSISTENTLY wrong — the SimpleQA regime, where the weak 8B got only 6/74
answered-correct yet resampled to the same wrong answer — entropy is LOW and the signal collapses to
AUROC 0.66 (CGG-BENCH §3A). This is not a tuning gap; it is the mechanism's definition. P2 oscillation
(J-85) has the same blind spot (a stable wrong chain does not oscillate). So black-box resampling
(P2+P3) ALONE has a provable failure class — and that class is precisely the dangerous one (a
fabrication asserted with full confidence). HG-2's >=2-distinct-predictor rule does not save you here
if BOTH distinct predictors are resampling-family (they share the blind spot); convergence only helps
if the second predictor is mechanistically EVIDENCE-grounded, not consistency-grounded.

CONCLUSION (Q1). Abstention-only achieves: a tunable H_cond among the answered (drop coverage far
enough and H_emit -> 0, but Cov -> the fraction the model actually knows-and-can-resample-stably).
Achievable H_emit at FULL coverage with detection-only: bounded BELOW by the confident-consistent
fabrication rate, which the resampling signal CANNOT see. That is why the next four sections matter:
the only way past the ceiling is to add EVIDENCE-grounded predictors (P1 round-trip, P-counterfactual
do-test) AND a repair loop that acts on them, because evidence-grounding does not share the
consistency blind spot — a fabricated symbol fails L-3 round-trip whether or not the model is
confident about it.

================================================================================
2. THE REPAIR LOOP (Q2) — the user's core ask, designed concretely
================================================================================

The insight: an abstention is INFORMATION THROWN AWAY. CGG already localized the EXACT weak claim
(L-1 typed span) and DIAGNOSED why it is weak (which PredictorSignal fired). A loop that consumes
that diagnosis and spends bounded effort to FIX the specific claim recovers coverage that blanket
abstention forfeits. This is the paper's §4.3 "predictable AND preventable" half, already scoped as
L-6, and it is the missing piece in the benchmark (CGG-BENCH §0: L-6 "curiosity-driven targeted
collection" was NOT benchmarked — it needs the runtime).

THE LOOP (the canonical state machine; maps L-1 -> L-2 -> L-6 -> re-L-2 -> L-7, gated by J-83):

  S0 EXTRACT      L-1 types the claim: {File,Symbol,Dependency,Api,Behavior,Causal,Reachability}.
  S1 SCORE        L-2 fuses P1(L-3)/P2(J-85)/P3(L-4)/P-cf(L-5) -> ClaimRisk tier + WHICH signal fired.
  S2 GATE-DECIDE  L-7 maps tier -> action. The CHANGE vs today: High no longer means "abstain",
                  it means "ForceEffort" (L-7 already specifies exactly this: High -> hand to L-6).
  S3 DIAGNOSE     branch on the FIRED predictor + ClaimKind (the diagnosis IS the predictor identity):
                    - P1 round-trip residual high (claim unsupported by evidence)
                        -> the remedy is RETRIEVAL: J-38 mid-task agentic retrieval at THAT symbol
                           (B-29 locates it), J-103 read-ladder escalate-on-miss, H-53 citation
                           pre-bind against the closed evidence set.
                    - P3 semantic entropy high (model uncertain / confabulating)
                        -> the remedy is DELIBERATION + DIVERSITY: more samples, the D-13 cross-family
                           adversary (a DIFFERENT pretraining family breaks within-family shared bias —
                           this is the ONLY remedy that touches the confident-consistent blind spot),
                           and deeper GRC reasoning (B-39 /reason N=4).
                    - Causal/Behavior claim, unverified
                        -> the remedy is EXECUTION: L-5 do-test (revert/patch-only-A, re-run F-4's
                           blake3-bound outcome) or a symbolic check (D-11 sympy/z3) — a CHECK, not a guess.
                    - Dependency claim (invented package)
                        -> API_REGISTRY round-trip (anti-slopsquatting, deterministic).
                    - locally unresolvable after the above
                        -> I-7 local->cloud escalation (a stronger model is a stronger generator AND a
                           stronger evidence-finder).
  S4 REMEDY       fire the SINGLE targeted remedy (L-6: targeted at the weak claim, NEVER blanket
                  re-read; bounded by the J-84 budget — max rounds + cost cap). New evidence lands.
  S5 RE-VERIFY    re-invoke L-2 on the claim WITH the new evidence. This produces a fresh receipt
                  (the round-trip now reconstructs / the do-test now flips / the checker now confirms).
  S6 COMMIT-OR-LOOP  J-83 receipt gate: if the receipt now PASSES, COMMIT-AND-FREEZE the claim
                  (GRC-v3's commit-and-freeze: the grounded claim is now immutable belief, safe to
                  compress, and the chain cannot drift back off it — rlm-grc PART 1 J-83). If it
                  still fails AND budget remains, loop to S3 with the NEXT remedy. If budget is
                  EXHAUSTED, THEN and only then abstain (L-7 Critical -> Abstain, credited by I-8/H-39
                  so honesty is not penalised).

HOW THIS RECOVERS COVERAGE (the quantitative argument). Pure abstention at the 50% PopQA operating
point refused 50 of 100 claims and recovered 0 of them. The repair loop, on the SAME 50 refused
claims, recovers every claim for which a remedy can mint a passing receipt. Decompose the 50:
  - claims wrong because the model DIDN'T RETRIEVE available evidence (the dominant coding-agent
    failure: invented file/symbol/signature/citation — L-3 context line) -> RECOVERED by S3-P1
    retrieval, because the fact WAS reachable, the model just didn't look. [CONJECTURE on the split,
    but L-3's premise is that "most agent hallucinations are invented files/symbols/packages/
    signatures and fake citations, all of which fail a reconstruction against the repo" — i.e. the
    evidence exists and retrieval finds it.]
  - claims wrong because the model was UNCERTAIN (P3 fired) -> partially recovered by deliberation +
    cross-family; the cross-family vote can flip a within-family-shared-wrong answer the single model
    could never fix by resampling itself.
  - claims wrong because a fact is genuinely UNRETRIEVABLE (no doc, no repo, no paper has it) ->
    NOT recovered; correctly abstained. This is the irreducible residual (§6).
So the loop's coverage recovery = (refused claims with an obtainable receipt) / (all refused). The
CEILING on recovery is set by RECEIPT AVAILABILITY, which is a DOMAIN property (§3), not a model
property. Where receipts are abundant (code/math/repo) recovery approaches 100% and you get near-0
H_emit at near-full coverage; where receipts are scarce (pure obscure-fact recall) recovery is low
and the loop degenerates gracefully to retrieve-or-abstain (still strictly better than abstain-only,
because the retrieve arm recovers the "didn't-look" subset).

THE TRUST INVARIANT THAT MAKES THIS SAFE (and distinguishes it from naive self-correction): the
commit at S6 is gated on an EXTERNAL receipt the model did not author (J-83 RD-1: "the model NEVER
sets its own confidence"; rlm-grc DECISIVE CORRECTION: "a model CANNOT reliably ground itself or
judge which of its own parts is sure"). Reflexion-style self-correction loops fail precisely because
the model judges its own fix. CGG x GRC does not: the receipt is L-3 deterministic reconstruction,
L-5's blake3-bound F-4 outcome, or D-11's symbolic verdict — TD-3, the LLM proposes, the checker
adjudicates. A repair loop whose acceptance test is the model's own say-so would re-introduce
hallucination at the verification step; this one cannot, by construction.

================================================================================
3. THE DOMAIN ASYMMETRY (Q3) — receipt availability sets the achievable frontier per domain
================================================================================

The repair loop's power is entirely a function of "does an external receipt exist for THIS claim,
and is it GROUND TRUTH or merely another proxy." Four receipt classes, strongest to weakest:

(A) EXECUTABLE receipt — CODE. claim "this code works / this patch fixes B". Receipt = run the
    held-out tests; F-4 blake3-binds the outcome the model cannot forge (L-5; CGG-BENCH §4 hypothesis:
    "CGG does NOT guess confidence — it RUNS THE TESTS"). This is GROUND TRUTH (tests pass or they
    don't). Repair = generate-run-repair until a sample passes the held-out tests, else abstain.
    Achievable: H_emit -> ~0 at HIGH coverage. Coverage cost = only the genuinely-unsolvable tasks.
    Caveat: the receipt is only as good as the test set (a weak/in-distribution test set is a Goodhart
    surface — §6); A5 in the bench grades on HELD-OUT tests precisely to keep it non-circular.

(B) SYMBOLIC receipt — MATH / formal claims. claim "this derivation / this invariant holds". Receipt
    = D-11 sympy/z3, E-53 Verus/Kani/Dafny (L-9). EXACT, not probabilistic ("token-perfect = the
    symbolic checker adjudicates", L-9). Achievable: H_emit -> ~0 at coverage = the checker's COVERAGE
    of the math (algebra/logic/arithmetic = full; analysis/topology = partial -> abstain on the rest,
    L-9 open-question, honest residue). The model proposes the derivation; the checker confirms a step
    or the system abstains "unverified by constructive check" (HG-4). This is the cleanest near-0 case
    AND it has zero coverage cost on in-coverage math.

(C) RETRIEVAL+CITATION receipt — REPO claims / CITED factual claims. claim "function X at file:line
    does Y" / "paper P says Z". Receipt = L-3 round-trip reconstruction against repo/source/DKG +
    H-53 closed-set citation membership (a ghost citation is "structurally impossible", L-3). GROUND
    TRUTH for EXISTENCE/LOCATION claims (the symbol is there or it isn't); WEAKER for INTERPRETATION
    claims (the cited passage exists but "says Z" is a reading the round-trip can't fully adjudicate).
    Achievable: near-0 H_emit on existence/location at high coverage (this is the bulk of coding-agent
    hallucination); partial on interpretation. Multi-hop factual claims route through HippoRAG
    (crates/memory/src/hippo_rag.rs) — and the rlm-grc PART 9 measurement is decisive here: a
    DETERMINISTIC graph-hop harness made a weak 8B hit 87% on 8xW multi-hop where model-driven
    navigation plateaued at 25%. So the receipt-minting RETRIEVAL itself must be deterministic, not
    model-driven, or the repair step re-injects hallucination (PART 8: free-code RLM "hallucinated 42
    on 7/8 tasks"). Repair here = retrieve-then-recheck; the lever is deterministic retrieval.

(D) PARAMETRIC-RECALL ONLY — obscure facts with NO retrievable source. claim "the population of
    village V in 1880 was N" where no doc, repo, or paper in reach contains it. Receipt = NONE
    obtainable. The repair loop degenerates: retrieval finds nothing, no checker applies, cross-family
    can only DETECT disagreement (not supply the fact). Achievable: H_emit -> 0 ONLY by abstention;
    coverage = the fraction the model knows-and-can-stably-resample. This is the PopQA/SimpleQA regime
    and the measured ~50% coverage cost (CGG-BENCH §3E). The loop adds ONE thing over abstain-only
    here: it converts "didn't-look" misses into hits IF any retrievable source exists, shrinking class
    D to its true unretrievable core.

MAXIMAL COVERAGE AT H_emit=0, per class (the honest frontier):
    A code:        ~100% minus genuinely-unsolvable tasks (receipt = ground truth).
    B math:        ~100% within checker coverage; abstain outside it.
    C repo/cited:  ~100% on existence/location; partial on interpretation.
    D parametric:  = P(fact is known-or-retrievable). Low for obscure facts. Irreducible.
THE ARCHITECTURAL PAYOFF: route every claim to its strongest available receipt class (D-10 domain
router already does claim-kind -> grounding-method dispatch; L-3 task 1 is exactly this map). The
domain router IS the thing that decides whether you are in the near-0 regime (A/B/C) or the
abstain-or-guess regime (D). This is why CGG is "good on ANY task" without being "magic on every
task": it reaches near-0 wherever a receipt exists and degrades HONESTLY (to abstention) where none does.

================================================================================
4. CGG x GRC INTEGRATION (Q4) — one system, not two
================================================================================

THE THESIS TO EVALUATE (from the prompt): CGG produces per-claim risk + the receipt requirement; GRC
is the loop that spends bounded effort to PRODUCE a passing receipt before committing. VERDICT: this
is correct AND it is already the latent architecture of both designs — they were authored to compose.
The proof is structural:

- GRC-v3 J-83 BeliefState is a carrier of {claim, external grounding-RECEIPT, abstention bit read
  from H-39, DKG provenance} with a per-step RECEIPT GATE: "commit iff the D-10-routed domain receipt
  PASSES, else ABSTAIN" (rlm-grc PART 1). That receipt gate's PASS/FAIL is EXACTLY L-7's
  Allow/Abstain decision. They are the same gate described in two documents. Unifying them: L-2's
  ClaimRisk IS the input to J-83's receipt gate; L-3/L-5/D-11 receipts ARE J-83's "external
  grounding-receipt" field. CGG is the SENSOR+ACTUATOR layer; GRC is the CARRIER+CONTROL-LOOP layer.

- J-84 effort classifier answers "HOW MUCH repair effort to spend" (Q4's tie-in): TYPE axis (reason
  vs RETRIEVE-recall vs validate) x COST proxy -> trivial=ZERO effort / easy=serial depth / hard=
  parallel width (J-86) / very-hard=escalate(I-7)-or-abstain(I-8) (rlm-grc PART 1 J-84). Map this onto
  the repair loop's budget: a Low-risk claim gets ZERO repair (the cost guard — L-6 "a Low-risk claim
  fires ZERO targeted effort"); a High P1-residual claim gets serial retrieval depth; a
  confident-consistent P3 case gets parallel WIDTH (J-86 candidate ladder + D-13 cross-family, because
  depth on a stable-wrong chain only entrenches it); an unresolvable claim escalates or abstains. J-84
  is the dial that keeps the loop from spending cloud tokens on claims that don't need it — the answer
  to "won't repair be ruinously expensive" (§6).

- J-85 overthinking guard answers "WHEN to stop" (Q4's other tie-in): the loop must terminate. J-85's
  deterministic 3-signal early-stop (oscillation/hesitation/declining-confidence) plus the J-84 round
  cap plus the receipt-passes condition give THREE independent stop conditions: receipt PASS (success),
  budget exhausted (abstain), or J-85 drift-detected (the chain is getting WORSE -> stop and abstain
  rather than spiral). This is what makes the loop bounded and prevents the anti-spiral failure mode.

- J-86 receipt-gated candidate ladder is the loop's PARALLEL form: branch candidate beliefstates, the
  receipt-PASSING one advances (commit+freeze), the rest backtrack (rlm-grc PART 1). This IS I-4
  best-of-N, but selected by RECEIPT not by vote (the selector hierarchy: deterministic RECEIPT >
  vote-over-independent-samples > LLM-self-eval BANNED). For code, best-of-N-by-held-out-tests is
  exactly J-86 over an executable receipt — and the bench A5 arm already proves this recovers
  wrong-to-correct (CGG-BENCH §4: "a baseline that produced WRONG code was recovered to CORRECT by
  both A2 and A5").

NET: CGG and GRC are ONE control system. The clean statement: GRC's BeliefState is the WORKING MEMORY;
CGG's predictors are the RISK SENSOR; L-6 is the ACTUATOR (spend effort); the external receipt is the
FEEDBACK SIGNAL; J-83's gate is the COMMIT LATCH; J-84/J-85 are the CONTROLLER GAINS (how hard, how
long). The single integration deliverable that does not yet exist as one object: a RepairController
that owns the S0-S6 loop and threads L-2 <-> L-6 <-> J-83. Today these are separate plan files
(L-2 scores, L-6 routes, L-7 gates, J-83 carries) with the loop IMPLICIT across them. Making it ONE
named loop is the highest-leverage build (§8).

================================================================================
5. HALLUCINATION AS A CURIOSITY/REWARD SIGNAL (Q5) — does the loop close toward 0 over time
================================================================================

The paper's §4.3 move (ported to L-6): the risk score is a CURIOSITY REWARD that auto-targets effort
at the weak claim; a proven-grounded resolution mints a Verified-Case node (J-96), so the gap costs
less next time WITHOUT retraining (cgg-system S6; L-6 task 3). Reason about whether this closes toward 0.

WITHIN A SESSION: yes, monotonically, for receipt-available claims. Each repair that mints a passing
receipt commit-and-FREEZES the claim (J-83) — it becomes immutable grounded belief that downstream
steps READ instead of re-deriving. So a fact grounded once is never re-hallucinated later in the same
session (this is L-8's job too: prune the false claims so they don't poison later steps). The
session's H_emit is strictly non-increasing as the BeliefState fills with frozen receipts. This is a
real closure mechanism, not a hope.

ACROSS A CORPUS/OVER TIME: yes, but BOUNDED and only on the retrieval/repo classes. J-96 Verified-Case
nodes accumulate: the second time the system meets the same coverage gap (same symbol, same API, same
paper-claim), the receipt is already cached, so the repair is free (a lookup, not a retrieval loop).
This compounds — it is the harness-level analogue of the paper's "50 targeted trajectories ~= oracle
data collection" (L-6 context). BUT three honest bounds:
  1. It closes the gap for SEEN gaps only; a novel obscure fact is still class-D (no receipt to cache).
  2. It requires the cached receipt to STAY VALID — a repo changes, a paper is superseded. F-5 staleness
     quarantine + H-32's 28d citation TTL + L-8 pruning are load-bearing: a stale cached receipt is
     WORSE than none (it confidently re-asserts an outdated fact). So the curiosity-closure only
     monotonically improves if paired with staleness eviction.
  3. It is NOT weight learning — the MODEL is no better; the HARNESS's coverage is. This is exactly
     ELAI's "harness > model" thesis (rlm-grc F1: measured Qwen3-4B 0->100 with the scaffold). The
     curiosity loop makes the SYSTEM's effective knowledge grow; the verified-case graph is the
     growing memory. [CONJECTURE: the asymptote is "the system has a cached receipt for every gap it
     has ever repaired and that is still fresh" — i.e. corpus H_emit on REPEAT queries -> the
     unretrievable residual, while H_emit on NOVEL queries stays at the per-query frontier of §3.]

So Q5 closure is REAL but it closes the RETRIEVAL/REPO/MATH/CODE gap toward 0 over repeated exposure,
and does NOTHING for the genuinely-unretrievable parametric residual (§6). The loop converts
"hallucination" into "a TODO for targeted collection," which is the right reframe: a flagged High-risk
claim is not a failure, it is a request for evidence the system then goes and gets.

================================================================================
6. HARD LIMITS / FAILURE MODES (Q6) — adversarial pass on my own proposal
================================================================================

F1. CONFIDENT-CONSISTENT FABRICATION WITH NO RETRIEVABLE RECEIPT. The irreducible core. SimpleQA
    AUROC 0.66 measured (CGG-BENCH §3A) is this class made visible. The black-box predictors are
    blind to it (low entropy, no oscillation), AND no receipt exists to repair it (class D). The ONLY
    mitigations are partial: (a) D-13 cross-family — a DIFFERENT family may not share the same
    confident-wrong, so disagreement re-appears and you can at least DETECT, then abstain; (b)
    conservative abstention on class-D claims by default. You cannot REPAIR what you cannot retrieve.
    This is the honest residual and it must be named in-file (GD-5 — L-plan already commits to this:
    "the CGG catches coverage-gap hallucination, NOT confident-shared-wrong beliefs; that limit is
    named everywhere", L-0 GD-5 line).

F2. A GAMEABLE RECEIPT (the Goodhart surface). The loop's safety rests entirely on the receipt being
    GROUND TRUTH. Three real holes: (a) CODE — a weak/in-distribution/model-influenced test set: the
    code passes the visible tests by overfitting them. Mitigation: grade on HELD-OUT tests (CGG-BENCH
    A5 does exactly this), and the test/gate is AUDITOR-authored + FROZEN (R8, wiring-enforcement),
    never written by the same agent producing the fix — an agent that writes its own receipt routes
    around it (rlm-grc PART 8 free-code RLM fabricated because nothing external bound it). (b) CITATION
    — H-53 closed-set membership proves the citation EXISTS but not that it SUPPORTS the claim
    (interpretation gap, §3C). Mitigation: tier it (existence = strong receipt; interpretation =
    weak receipt -> caveat not commit). (c) SYMBOLIC — the model emits a derivation sympy "confirms"
    because the model framed the check to be trivially true. Mitigation: TD-3 — the intervention/check
    is DERIVED deterministically from the typed claim, NOT designed by the model (L-5 task 1: "NO
    model call to design the intervention"; L-9: "NO model self-grading the math"). The receipt is
    Goodhart-resistant exactly to the degree the model cannot author or shape it.

F3. COST + LATENCY OF REPAIR. A naive repair-everything loop is ruinous on cloud (N resamples +
    retrieval + execution per claim). Mitigations, all already specified: the HG-2 convergence
    cost-gate (the expensive predictors fire only after a cheap signal flags medium risk — L-2 task 3;
    L-3 is the cheap first predictor); the J-84 budget (Low-risk = zero effort; targeted not blanket —
    L-6); run resampling on the free-pool/Batch lane (cgg-system §5). The cost is bounded by
    "fraction of claims that are High-risk AND receipt-available," not "all claims." [CONJECTURE: in
    a coding session most claims are Low (the model is grounded), so the loop fires rarely; the cost
    is dominated by the few genuinely-risky claims, which is exactly where you WANT to spend.] The
    autonomous-gate envelope (B-22) must cap total curiosity spend so the loop cannot blow the token
    ceiling (L-6 open-question — a real interaction risk to verify against the live envelope).

F4. THE RECEItPT-MINTING RETRIEVAL ITSELF HALLUCINATES. If the repair step uses MODEL-DRIVEN
    retrieval/navigation, it re-injects the failure (rlm-grc PART 8: free-code RLM hallucinated "42";
    PART 9: model-driven nav plateaus ~25% while deterministic graph-hop hits 87%). Mitigation:
    deterministic retrieval (BM25/HippoRAG PPR) for the receipt; the model only does the atomic fuzzy
    step (entity read), never the hop logic. This is a DESIGN CONSTRAINT on the loop, not an
    afterthought: the repair actuator must be deterministic where the receipt is concerned.

F5. CONVERGENCE THEATRE (fake >=2-agree). HG-2 requires 2 distinct predictors, but if both are
    resampling-family (P2 oscillation + P3 entropy) they share the confident-consistent blind spot and
    "agree" on a wrong-but-stable claim being LOW risk. Mitigation: the convergence rule must require
    at least one EVIDENCE-grounded predictor (P1 or P-cf) in the quorum for any "safe" verdict on a
    factual claim, not just any two. H-40 decorrelation enforces the predictors are genuinely distinct;
    L-2 open-question flags exactly this ("verify semantic-entropy and round-trip do not collapse to
    the same signal"). I would HARDEN this: a Low/Allow verdict on a class-A/B/C claim should require
    a PASSING evidence-receipt, not merely the ABSENCE of a resampling flag. Absence-of-disagreement
    is not presence-of-grounding.

IRREDUCIBLE RESIDUAL (the one-line answer to "where does this still fail"): a fact that is
(i) confidently and consistently asserted by the model, (ii) for which no executable/symbolic/
retrievable receipt is obtainable, and (iii) is actually wrong. CGG cannot detect it (blind spot),
GRC cannot repair it (no receipt), and the only safe move is broad class-D abstention — which costs
coverage. Everything OUTSIDE this residual is drivable to near-0 emitted hallucination at high coverage.

================================================================================
7. CONCRETE RECOMMENDATIONS FOR THE L-PLAN (Q7) — prioritized, grounded
================================================================================

The L-plan already contains ~90% of the parts. The gap is that the REPAIR LOOP is implicit across
L-2/L-6/L-7/J-83 rather than one named, measured control object, and the convergence quorum does not
yet REQUIRE an evidence-receipt. Prioritized:

P0 (highest leverage, mostly net-new wiring). BUILD THE RepairController as ONE named loop.
   A new unit (call it L-15, "receipt-minting repair controller") that owns S0-S6 (§2): consumes
   L-2 ClaimRisk, dispatches L-6's targeted remedy, RE-INVOKES L-2, and commits via the J-83 receipt
   gate or abstains via L-7. Today L-6 "routes and re-scores" and L-7 "decides" as separate seams;
   the LOOP that iterates remedy->re-verify->commit-or-loop under the J-84 budget and J-85 stop is
   not a single object. This is the difference between "abstain ~75% at 50% coverage" and "repair to
   near-0 at high coverage." Composes: L-2, L-6, L-7, J-83 receipt gate, J-84 budget, J-85 stop,
   J-86 ladder. Net-new: the loop controller + its termination logic. THIS IS THE USER'S CORE ASK
   made concrete.

P1. HARDEN THE CONVERGENCE QUORUM (L-2 amendment, small, high-safety). Change HG-2 so a Low/Allow
    verdict on a class-A/B/C (receipt-available) factual claim REQUIRES at least one PASSING
    evidence-grounded receipt (P1 round-trip OR P-cf do-test), not merely the absence of a resampling
    flag (§6 F5). Absence-of-disagreement != presence-of-grounding. This closes the convergence-theatre
    hole and is the single change that most moves H_emit on confident-consistent claims where a
    receipt exists. Grounds in L-2's own open-question (predictor decorrelation).

P2. SHIP L-5 (do-test) and L-3 (round-trip) AS THE PRIORITY PREDICTORS, before investing further in
    L-4 (semantic entropy). The bench is decisive here: black-box resampling (L-4) tops out at 0.83
    on fair tests and 0.66 on adversarial (the ceiling, §1); the EVIDENCE-grounded predictors (L-3/L-5)
    are what break the ceiling (CGG-BENCH §5: "the load-bearing path to ~0 is the EVIDENCE-grounded
    predictors"). L-4 is the genuine net-new SIGNAL but it is necessary-not-sufficient; L-3/L-5 are
    where near-0 comes from. Re-order WAVE 2 to land L-3+L-5 first, L-4 second.

P3. MAKE THE REPAIR-RETRIEVAL DETERMINISTIC (constraint on L-6). L-6's targeted retrieval must route
    through deterministic BM25/HippoRAG (crates/memory/src/hippo_rag.rs), with the model restricted to
    atomic entity extraction — NEVER model-driven free navigation (§6 F4; rlm-grc PART 9 measured
    87% deterministic vs 25% model-driven). Add this as an L-6 stub_ban: "NO model-driven multi-hop
    navigation in the repair retrieval; deterministic graph-hop + atomic extraction only."

P4. EXTEND L-10 TO MEASURE THE REPAIR DELTA, not just the gate delta. The current L-10 measures
    gate-ON-vs-OFF unsupported-claim rate (abstention). Add a third arm: gate+REPAIR vs gate-only vs
    off, measuring H_emit AT FIXED COVERAGE (the whole point is recovering coverage). The decisive
    number is "H_emit at 90% coverage with repair" vs "H_emit at 90% coverage abstain-only" — the
    bench must show repair holds H_emit low WITHOUT the coverage sacrifice. This is the done-gate for
    the whole thesis and it is ELAI-measured (ID-5), never a paper number.

P5. L-9 (research/math profile) IS the cleanest near-0 showcase — prioritize it as the FLAGSHIP demo.
    Symbolic receipts (D-11/E-53) are ground-truth and zero-coverage-cost on in-coverage math (§3B).
    The repair loop on a wrong arithmetic step -> sympy catches it -> regenerate -> sympy confirms ->
    commit is the most legible "near-0 at high coverage" story, and it directly serves the user's
    research-grounding ask. It needs the RepairController (P0) to close the loop rather than just
    abstain.

GENUINELY NET-NEW (not just composition): the RepairController loop object (P0); the evidence-receipt
quorum requirement (P1); the deterministic-retrieval constraint as enforced ban (P3); the repair-delta
bench arm (P4). Everything else is activation/re-ordering of already-specified L/J/D units.

================================================================================
8. PRIORITIZED BUILD LIST
================================================================================
1. [P0] L-15 RepairController: the named S0-S6 loop (L-2 -> L-6 remedy -> re-L-2 -> J-83 commit-or
   -L-7-abstain), bounded by J-84 budget + J-85 stop, parallel form = J-86 ladder. The user's core ask.
2. [P1] L-2 amendment: convergence quorum on a receipt-available factual claim REQUIRES >=1 passing
   evidence-receipt (P1/P-cf), not mere absence-of-resampling-flag. Closes the confident-consistent
   convergence-theatre hole.
3. [P2] Re-order WAVE 2: land L-3 (round-trip) + L-5 (do-test) BEFORE L-4 (entropy). Evidence-grounded
   predictors break the 0.66/0.83 ceiling; entropy is necessary-not-sufficient.
4. [P3] L-6 hardening: deterministic BM25/HippoRAG retrieval + atomic extraction only; ban model-driven
   navigation in the repair step (measured 87% vs 25%).
5. [P4] L-10 extension: add the gate+REPAIR arm; measure H_emit AT FIXED COVERAGE (90%), the
   coverage-recovery done-gate. ELAI-measured.
6. [P5] L-9 as flagship: the symbolic-receipt repair loop (sympy catches -> regenerate -> confirm ->
   commit) is the cleanest near-0-at-high-coverage demo; depends on P0.
7. [carry] J-96 verified-case mint + F-5/H-32 staleness eviction: the corpus-level curiosity closure,
   REQUIRES staleness eviction or cached receipts rot into confident-stale fabrication (§5 bound 2).
8. [carry] D-13 cross-family as the ONLY partial mitigation for the class-D confident-consistent
   residual; give it the clean run the bench couldn't (gemini RPD caps wrecked A4, CGG-BENCH §1).

================================================================================
9. THE SINGLE MOST IMPORTANT INSIGHT
================================================================================
An abstention is a discarded opportunity to ground. The measured ~75%-cut-at-50%-coverage result is
the price of treating CGG as a STOP sign; the entire coverage loss is recoverable — for every claim
that has an obtainable external receipt — by treating each High/Critical flag instead as a TARGETED
REQUEST FOR EVIDENCE that a bounded repair loop fulfils, then committing the claim ONLY against a
receipt the model did not author. This collapses the problem to a single sharp boundary: where a
ground-truth receipt EXISTS (code/math/repo/cited — the executable, symbolic, and retrieval domains),
CGG x GRC drives EMITTED hallucination to near-0 at HIGH coverage because it CHECKS instead of
guessing; where NO receipt is obtainable (confident-consistent fabrication of an unretrievable fact),
it cannot repair and must abstain, and that — not the model's parametric error rate — is the true
irreducible residual. The build that realizes this is not more detection; it is the one missing
object: a receipt-minting repair controller that turns CGG's risk signal and GRC's receipt gate into
a single closed loop.

End of reasoning.
