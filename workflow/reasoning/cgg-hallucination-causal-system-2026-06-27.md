# CGG — Causal Grounding Gate: a model-agnostic hallucination-reduction system for ELAI

Date: 2026-06-27. Main LLM in-band (spec-eval + paper-to-plan PHASE 4/5 shape). No fleet.
Sources read in full this session: the paper (arXiv:2606.27326 "Hallucination in World
Models is Predictable and Preventable", Hansen & Wang, UC San Diego), the user's harness
notes (C:/Users/user/Downloads/hallucination_detection_harness_notes.md), the elai-lang
causal screenshots (counterfactual / predictive / contrastive causal learning, causal_confidence
per connection, pruning spurious connections, the evidence hierarchy), and ELAI-FEATURE-MAP.md
(all 16 subsystem groups + the plan-family ledger).

Outcome (appended per paper-to-plan PHASE 6): authored L-plan L-1..L-10 (the first L-letter;
K-10's no-new-letters doctrine user-overridden this session). New feature-map group 1.17.

================================================================================
0. THE THESIS (what transfers, grounded)
================================================================================
The paper's load-bearing claim (§1, §4.3): hallucination is not an architecture problem to be
solved by bigger models, it is a LOW-COVERAGE / EPISTEMIC-UNCERTAINTY problem that is (a)
PREDICTABLE from runtime signals available without labels or retraining, and (b) PREVENTABLE by
spending effort where coverage is weak. Three mechanistically-distinct predictors converge on the
same realized error (rho ~= 0.80 vs rollout dPSNR, §5.1); the paper treats the CONVERGENCE itself
as evidence the signal is real (§4.2). The strongest causal signal is INTERVENTIONAL: the
action-marginalization test perturbs the action and measures whether the output responds (§4.1.ii,
§5 "action shuffle ratio").

The user's harness notes already did the LLM port: claim extraction > evidence retrieval >
semantic-disagreement > codebase/exec grounding > risk score > policy gate, model-agnostic
(black-box), with "causal" meaning OBSERVABLE/interventional not correlational (the explicit
evidence hierarchy: semantic match < static dep < temporal < runtime trace < counterfactual).

The elai-lang screenshots supply the vocabulary: counterfactual / predictive / contrastive causal
learning, per-connection causal_confidence updated by EMA, and pruning of spurious connections.

CENTRAL GROUNDED FINDING (the spec-eval verdict): ELAI already ships ~85% of this architecture.
Its trust spine today polices the AGENT'S OWN code wiring; the CGG points the same spine at ANY
model's CLAIMS, on ANY backend. This is ACTIVATION + UNIFICATION + one new causal layer, not
invention. Model-agnosticism is real and load-bearing: every signal is black-box (resampling,
perturbation, repo/exec evidence), so it rides the Multi-Runtime Backend (B-37, RuntimeAdapter)
identically on cloud Sonnet, local Qwen, BYOK, free-pool, OpenRouter, and the CLI bridges.
Logprobs are an OPTIONAL booster (G-7 already fuses them) never a dependency.

================================================================================
1. THE THREE PREDICTORS, PORTED (paper §4.2 > LLM)
================================================================================
The convergence principle is the steal: a high-risk verdict requires >=2 distinct predictors to
agree (the paper's §4.2; ELAI already encodes "convergence = trust" as the Verification Aggregator
D-3 ">=2 verifiers concur" + decorrelation-based retention H-40). This is what kills cost and
false alarms.

P1 round-trip residual (paper: decode>re-encode, does the predicted latent survive).
  LLM analogue: a claim that cannot be reconstructed/supported from evidence = high residual.
  ELAI machinery: grounding verification 6-8 methods (D-10/D-11 FILE/SOURCE/TEST/WEB/API/
  API_REGISTRY/CosmosCheck/VisualArtifactCheck), citation pre-binding (H-53, closed-set membership
  > ghost citation structurally impossible), DKG validate-citation-before-use (H-32).
  Verdict: EXTENSION(D-10/H-53) — activation over arbitrary model claims, not new machinery.

P2 flow instability (paper: denoiser oscillates across Euler substeps).
  LLM analogue: reasoning path unstable under equivalent-prompt perturbation (SAC3
  arXiv:2311.01740, paraphrase the question, watch the answer move).
  ELAI machinery: the J-85 overthinking guard ALREADY measures answer oscillation (r=0.78) — this
  is flow-instability by another name.
  Verdict: ALREADY-SHIPPED-as-signal(J-85) — consumed as a convergence input by L-2, not rebuilt.

P3 inter-seed variance (paper: N denoising seeds disagree).
  LLM analogue: SelfCheckGPT (arXiv:2303.08896) / semantic entropy (Nature 2024,
  s41586-024-07421-0) — resample N, CLUSTER BY MEANING, disagreement over meanings = risk.
  ELAI machinery: best-of-N over SessionDag (I-4), >=2-concur (D-3), cross-domain canary (D-7),
  abstention-aware scoring (H-39).
  Verdict: NEW (the one genuine net-new producer): ELAI resamples and counts concurrence but does
  NOT cluster N samples by MEANING and emit semantic entropy as a named hallucination signal. Built
  in L-4, hardened cross-family via D-13 (same-model resampling misses within-family shared bias;
  SAC3 cross-model catches it).

================================================================================
2. THE CAUSAL CORE (elai-lang taxonomy, disciplined by the notes' warning)
================================================================================
The notes' sharpest point (and the elai-lang screenshots agree): "causal" must mean
INTERVENTIONAL/observable, never correlation. "A imports B" != "A causes bug B". So the evidence
hierarchy is enforced as SCORED TIERS, and only interventional/executable evidence is LABELLED
causal.

Counterfactual causal (strongest = the paper's action-marginalization do-test).
  When a model claims "A causes/fixes B", run the intervention — revert only A, disable the flag,
  patch only A, add a trace — and observe whether B changes (paper §4.1.ii: perturb the cause,
  measure the effect).
  ELAI substrate IS the spine: F-4 cant-lie (real test outcome, blake3 stdout-hash the model
  cannot forge), mutation testing (C-28/I-20, literally perturb-and-observe), metamorphic relations
  + IntentPair (C-29), CSQ counterfactual safety questions (phase-2D), blast-radius (J-16).
  Verdict: EXTENSION(F-4/C-28/C-29) — point the do-test at the model's CLAIM, not only the agent's
  own diff. The heart of L-5.

Predictive causal (medium): stack-trace / recent-diff / import-edge. ELAI = DKG typed edges +
  impact API (B-29) + taint provenance (G-1). Labelled weaker, by tier. Verdict: ALREADY-SHIPPED.

Contrastive causal (the learned verifier): learn grounded-vs-hallucinated claim features. ELAI =
  rules-engine causal>50% promotion + case-based calibration (C-45/C-47) + verifier-ranked SFT
  exhaust (I-30, GPU-pending). MVP stays rule-based (the notes agree). Verdict: ALREADY-SHIPPED;
  the L-plan uses the rule-based MVP, the learned tier rides I-30.

Causal confidence per connection (the elai-lang EMA): ELAI = conformal refusal tau (D-16, UniCR) +
  G-7 confidence producer (logprob + self-consistency + generated-test agreement fused, Platt
  ECE 0.32>0.03). G-7 IS the elai-lang per-connection causal_confidence. Verdict: ALREADY-SHIPPED;
  L-2 generalizes G-7's claim_score into the model-agnostic ClaimRisk object.

Pruning spurious connections (the elai-lang prune): remove false repo assumptions before they
  poison later steps. ELAI = staleness quarantine cascade (F-5) + memory semantic-validation (J-21)
  + pristine-context injector (I-5, exclude failed-path tracebacks). Verdict: EXTENSION — L-8
  threads these over the agent's live claim/belief graph (J-83 BeliefState).

================================================================================
3. THE MITIGATION HALF (the best transfer; paper §4.3)
================================================================================
The paper does not only DETECT — it uses the hallucination predictor AS a curiosity reward to
collect data where coverage is weak. LLM port: the risk score becomes a curiosity signal that
AUTO-TRIGGERS targeted effort at exactly the weak claim — fire mid-task agentic retrieval (J-38),
the graduated read-ladder escalate-on-miss (J-103), a targeted test/trace, or local>cloud
escalation (I-7) — only at high-risk claims, not blanket. This is "preventable, not just
detectable", ported (L-6). And it compounds: a proven-grounded resolution mints a Verified Case
Graph node (J-96) = the harness-level analogue of the paper's coverage-gap closure, without
touching weights.

================================================================================
4. THE RESEARCH / MATH PROFILE (the user's follow-up, grounded)
================================================================================
Q: does planning persist PDFs/grounding sources, and can a deep-think + grounding mode serve
researchers needing token-perfect math accuracy?

YES on persistence: paper-ingestor writes interface maps to workflow/papers/; /pdf + the MinerU
lane (J-33) parse a PDF > the B-35 Context Artifacts Pipeline > the DKG + the Cosmos artifact tier
(B-42, global cross-project store). An ingested paper is grounded, retrievable, citable evidence.

YES on deep-think: ELAI's "deep think" is the Chat Reflect Pipeline (B-39, /reason +
ReasoningDrawer): TRIAGE > REASONER N=4 > AUDITOR REFINE (cap 2) > SYNTHESIZE. Plus GRC-v3 (J-83
belief-state carrier + J-86 receipt-gated candidate ladder) and the harness deep-research skill.

So the answer is NOT a separate reasoning engine — it is a PROFILE of the existing one: /reason
(deep-think, N=4) running in GRC Mode A (the Constructive-Checker Registry D-11: sympy/z3/compile,
+ the Formal Verification Lane E-53) over Cosmos-ingested papers, with the CGG P3 semantic-entropy
predictor watching the N=4 reasoners (disagreement on a derivation = flag) and P1 round-trip
checking each claim against the ingested paper bytes (citation pre-binding H-53). Token-perfect =
the symbolic checker is exact; the model proposes, the checker adjudicates (TD-3: the LLM never
adjudicates — perfectly aligned with house doctrine). This is L-9.

================================================================================
5. ADVERSARIAL PASS (spec-eval requires it)
================================================================================
- Shared-confident-wrong beliefs: P3 (resampling one model) misses bias shared across that model's
  samples; the paper's predictors likewise catch only coverage gaps, and the notes name this limit.
  FIX: route P3 through the cross-family adversary (D-13, different pretraining family) — cross-
  family disagreement (SAC3) catches within-family bias self-resampling cannot. (L-4.)
- Cost: N resamples + perturbations are expensive on cloud. FIX: the convergence requirement (>=2
  agree, HG-2) means the expensive inter-seed/counterfactual layer fires only AFTER cheap signals
  (claim-extraction "file does not exist", P1 round-trip) flag medium risk; route by the J-84
  effort classifier; run resampling on the free-pool (J-54) / Batch lane (H-5). (L-2/L-6.)
- "Causal" fake precision (the notes' explicit warning): FIX: the evidence hierarchy is enforced as
  scored tiers (HG-1); only interventional/executable evidence is tagged causal. (L-5/L-2.)
- Claim extraction can itself hallucinate / be gamed: FIX: deterministic structural claim typing
  (the notes' table: file/symbol/dep/API/behavior/causal/reachability) + off-agent verification —
  the model that made the claim never verifies it (wiring R8, auditor != executor). (L-1.)

================================================================================
6. STEAL LEDGER (per-candidate verdict)
================================================================================
S1 convergence-of-distinct-predictors (paper §4.2)        > EXTENSION(D-3/H-40)   > L-2 fuse
S2 P1 round-trip residual                                  > EXTENSION(D-10/H-53)  > L-3
S3 P2 flow instability                                     > ALREADY-SHIPPED(J-85) > L-2 input
S4 P3 inter-seed semantic entropy (SelfCheckGPT/sem-entropy) > NEW                 > L-4
S5 action-marginalization do-test (paper §4.1.ii)          > EXTENSION(F-4/C-28)   > L-5
S6 risk-as-curiosity targeted collection (paper §4.3)      > NEW(compose J-38/I-7) > L-6
S7 unified ClaimRisk object (generalize G-7 to any model)  > NEW(generalize G-7)   > L-2
S8 typed claim extraction (the notes' claim table)         > NEW(off-agent)        > L-1
S9 policy gate low/med/high/critical                       > EXTENSION(D-16/AutonomousGate) > L-7
S10 pruning spurious connections (elai-lang)               > EXTENSION(F-5/J-21/I-5) > L-8
S11 research/math deep-think + constructive-checker mode   > EXTENSION(B-39/D-11/B-42) > L-9
S12 ELAI-measured hallucination delta                      > NEW(compose G-6/I-38)  > L-10
REJECTED: per-token logprob as a REQUIRED signal (HG-3: black-box first, logprob optional booster;
  cloud APIs do not uniformly expose it, the local sidecar does via LogitsProcessor — never depend
  on it). REJECTED: a generative summary of the claim as the "round-trip" (HD-2 ban — busts cache,
  re-hallucinates; the round-trip is exact evidence reconstruction, not a paraphrase).

================================================================================
7. PLAN IDS CREATED
================================================================================
L-1..L-10 authored 2026-06-27 (docs/thefinalglance/L-plan/), grounding map
_grounding/L-CGG-GROUNDING-MAP.md, feature-map group 1.17, ledger line. New delta family HG-1..HG-4
(earned per template R-j — hallucination-grounding failure modes existing doctrine does not cover).
No code shipped; docs/ and workflow/ are gitignored. Every external claim tagged measured-vs-paper
(ID-5/KD-1); no arXiv number imported as an ELAI number.

End of CGG reasoning.
