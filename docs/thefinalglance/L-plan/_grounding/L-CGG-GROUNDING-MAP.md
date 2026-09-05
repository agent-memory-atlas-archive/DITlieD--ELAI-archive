# L-CGG-GROUNDING-MAP — the one grounded symbol map for the L-plan (CGG track)

Authored 2026-06-27 BEFORE the L-N detail files (PLAN-FOLDER-TEMPLATE section 4). Run: spec-eval +
paper-to-plan in-band. The anchors below are sourced from ELAI-FEATURE-MAP.md (read in full this
session) + the K-12 detail file's code-grounded anchors (which corrected several stale map labels
2026-06-25). Anchors carried from the feature map are tagged [MAP] and MUST be re-grepped at edit
(W-7); anchors confirmed via the K-12 code-grounding pass are tagged [CG-0625].

================================================================================
CANONICAL CONSTANTS (re-read at edit, W-9 — NEVER hard-code)
================================================================================
LIVE baseline (Tier-B derived, ELAI-FEATURE-MAP.md header + K-plan derivations):
  SCHEMA_VERSION ~= 268 / TOTAL_VARIANTS ~= 577 (ssb.rs / action_type_lock.rs).
  Every prior number (236/545, 241/550, 268/577) is a snapshot; re-grep ssb.rs +
  action_type_lock.rs at the start of EACH L-N session and bump SEQUENTIALLY (9-tuple golden
  lockstep; a partial add breaks the workspace — the 91-error incident).

================================================================================
TS — SUBSTRATE-SHIPPED (what the CGG COMPOSES, do NOT re-implement — GD-1)
================================================================================
TS-1  Grounding verification 6-8 methods [MAP, D-10/D-11] — FILE/SOURCE/TEST/WEB/API/
      API_REGISTRY_CHECK binary pass/fail (MiniCheck-style) + CosmosCheck (B-42) +
      VisualArtifactCheck (E-36); select_grounding_mode deterministic at grounding_mode.rs:157
      (LLM has no input). The P1 evidence backbone.
TS-2  Citation pre-binding [MAP, H-53] — freeze a ranked EvidenceAnchorSet BEFORE generation, [N] =
      closed-set membership check, out-of-set > D-16 refusal; ghost citation structurally
      impossible. citation_prebind.rs (PARTIAL/uncommitted). The P1 closed-set check.
TS-3  Confidence producer G-7 [MAP, DARK(ELAI_CONFORMAL_REFUSAL_ENABLE)] — fuse logprob +
      self-consistency + generated-test agreement into claim_score; Platt ECE 0.32>0.03;
      trifecta SESSION gate at production_agent_rpc.rs:334. The ClaimRisk seed (L-2 generalizes it).
TS-4  Conformal refusal D-16 (UniCR) [MAP, DARK(ELAI_CONFORMAL_REFUSAL_ENABLE)] — tau nightly L3
      update; RefusalDecision row; consumed by G-7/H-39/H-53. The abstain primitive (L-7).
TS-5  Verification Aggregator D-3 [MAP, DARK] — VerificationResult requires >=2 verifiers concur to
      promote. The convergence fuse seed (L-2, HG-2). Decorrelated retention = H-40.
TS-6  Abstention-aware scoring H-39 [MAP, PARTIAL] — correct=+1/abstain=0/confident-wrong=-t/(1-t);
      reads D-16 RefusalDecision; SSB AbstentionScored. The confident-wrong penalty (L-2/L-7).
TS-7  F-4 cant-lie [MAP, WIRED] — AntiFabricationGate.check() at task_runner.rs:9565; blake3
      stdout_hash on TestOutcomePayload; auditor cannot emit passed=true without the matching
      orchestrator-captured hash; TestIntegrityEvent. The counterfactual outcome the model cannot
      forge (L-5).
TS-8  Mutation testing C-28 + budgeted scheduler I-20 [MAP, PARTIAL/WIRED] — cargo-mutants/mutmut;
      I-20 mutants on the diff data-flow cone, survivors > counterexample Directives; SSB
      MutationBudgetedRun. The perturb-and-observe engine (L-5).
TS-9  IntentPair + MetamorphicRelation C-29 [MAP, WIRED] — gates wired unconditionally;
      intent_pairs<3 blocks PLANNING>TESTING_FIRST. The metamorphic do-test inputs (L-5).
TS-10 Cross-family adversary D-13 [MAP, DARK(ELAI_ADVERSARY_ENABLE)] — different pretraining family
      from executor, never same-model self-critique; CounterEvidenceSchema. The P3 cross-family
      hardening (L-4, HG-2).
TS-11 Evidence-ranked search ladder I-4 [MAP, DARK(ELAI_SEARCH_LADDER_ENABLE)] — select BY RECEIPTS
      not LLM-judge; run_best_of_n over SessionDag. The resample substrate (L-4).
TS-12 Retrieval agentic loop J-38 [MAP, PLANNED] + graduated read-ladder J-103 [MAP, PLANNED] +
      cascading escalation I-7 [MAP, WIRED] — the targeted-effort sinks (L-6).
TS-13 Taint propagation G-1 [MAP, WIRED] — Provenance at 4 ingest sites, Provenanced<String>,
      capability_gate before destructive sinks. The "untrusted evidence" label (L-1/L-5).
TS-14 Staleness quarantine F-5 [MAP, WIRED] + memory semantic-validation J-21 [MAP, PLANNED] +
      pristine context injector I-5 [CG-0625, BUILT pristine_path_filter.rs] — the prune substrate
      (L-8).
TS-15 BeliefState I-11 / J-83 [CG-0625, BUILT belief_state.rs; BeliefState::new at
      escalation_waterfall.rs:239] — the typed reasoning carrier the claim/belief graph rides (L-8).
TS-16 Chat Reflect Pipeline B-39 [MAP, WIRED] — TRIAGE>REASONER N=4>AUDITOR REFINE cap2>SYNTHESIZE;
      /reason + ReasoningDrawer. ELAI's "deep think" (L-9).
TS-17 Constructive-Checker Registry D-11 [MAP, DARK(ELAI_CHECKER_ENABLE)] sympy/z3/compile +
      Formal Verification Lane E-53 — the token-perfect math adjudicator (L-9, TD-3).
TS-18 Cosmos artifact tier B-42 [MAP, WIRED] + MinerU lane J-33 [MAP, PLANNED] + paper-ingestor +
      Context Artifacts Pipeline B-35 [MAP, WIRED] — the ingested-paper grounding store (L-9).
TS-19 SAFEDELTA G-6 [MAP, BENCH-PENDING] + QDELTA I-38 [MAP, PLANNED] + ConvergenceCurveMeasured
      I-27 [MAP, DARK] + bench_convergence runner — the measurement spine (L-10).
TS-20 J-85 overthinking guard [MAP, PLANNED] — answer oscillation r=0.78 = P2 flow-instability;
      consumed as a convergence input (L-2), not rebuilt.
TS-21 AutonomousGate B-22 EvidenceCheck per transition [MAP, DARK-flagged] + Aletheia voluntary
      abort I-8 [MAP, PLANNED] + honest-refusal SLA D-17 [MAP, DARK] — the policy-gate fabric (L-7).
TS-22 J-84 2-axis reasoning-effort classifier [MAP, PLANNED] — routes trivial/easy/hard/very-hard;
      the cost-gating router for the expensive predictor layer (L-2/L-6).

================================================================================
NN — NET-NEW (what the L-plan actually BUILDS)
================================================================================
NN-1  ClaimSet typed extraction (L-1) — deterministic structural typing of any model output into
      {ClaimType, span, refs}; ClaimType in {File,Symbol,Dependency,Api,Behavior,Causal,
      Reachability} (the notes' table). Off-agent. No equivalent exists today (claim extraction is
      implicit inside grounding-sweep, not a typed first-class object).
NN-2  ClaimRisk object + convergence fuse (L-2) — generalize G-7 claim_score from the agent's own
      output to ANY backend's claim; fuse P1/P2/P3 under the >=2-agree rule (HG-2). The keystone.
NN-3  Semantic-entropy inter-seed predictor (L-4) — resample N, cluster by MEANING (NLI-equivalence
      via the F-12 DeterministicNliJudge, NOT an embedding cosine), emit entropy; cross-family via
      D-13. The one genuine net-new predictor.
NN-4  Counterfactual do-test on model claims (L-5) — extend F-4/C-28/C-29 to fire on a model's
      "A fixes/causes B" claim and observe whether B changes. EXTENSION not from-scratch.
NN-5  Risk-as-curiosity router (L-6) — the paper §4.3 mitigation: high risk auto-fires targeted
      retrieval/test/trace/escalation at the weak claim.
NN-6  Research/math deep-think grounding profile (L-9) — a /reason + Mode-A + Cosmos-paper
      configuration with CGG over the N=4 reasoners. A profile, not an engine.
NN-7  CGG hallucination-delta bench (L-10) — ELAI-measured gate-ON-vs-OFF hallucination-reduction
      delta on a held-out claim corpus.

================================================================================
HG — NEW DELTA FAMILY (earned per template R-j; hallucination-grounding doctrine)
================================================================================
HG-1  CAUSAL-MEANS-INTERVENTIONAL. A claim is labelled "causal" ONLY on interventional/executable
      evidence (a counterfactual do-test or a runtime trace). Static/temporal/semantic evidence is
      scored at its weaker tier and NEVER called causal. (paper §4.1.ii do-test; the notes' evidence
      hierarchy.) Binds L-5, L-2.
HG-2  CONVERGENCE-NOT-SINGLE-SIGNAL. A high/critical risk verdict requires >=2 mechanistically-
      distinct predictors to agree; a single predictor never blocks (it may caveat). (paper §4.2.)
      Extends D-3. Binds L-2.
HG-3  RISK-IS-BLACK-BOX. Every risk signal must work without model internals; logprobs / white-box
      probes are OPTIONAL boosters, never dependencies — so the gate runs identically on any B-37
      backend. Binds L-1/L-2/L-3/L-4.
HG-4  ABSTAIN-OR-VERIFY-NEVER-SILENTLY-PASS. An unverifiable high-risk claim routes to targeted
      verification or abstention, never a silent allow; missing evidence = high risk (wiring R1
      applied to claims). Extends D-16/D-17. Binds L-6/L-7.

================================================================================
DC — DESIGN/MAP CORRECTIONS (reality vs the map; apply at feature-map upsert)
================================================================================
DC-1  I-5/I-11/H-50 are BUILT (pristine_path_filter.rs / belief_state.rs / context_policy.rs), the
      map's older PLANNED labels are stale [CG-0625] — the CGG composes the BUILT code (L-8/L-9).
DC-2  G-7 confidence producer + D-3/D-16 are DARK (default-OFF flags), not absent — the CGG
      ACTIVATES them under its own kill-switch, it does not rebuild them.
DC-3  ELAI has resampling (I-4) and >=2-concur (D-3) but NOT semantic-entropy-over-meaning-clusters
      as a named signal — L-4 is genuinely net-new (confirmed against the map's reasoning/verifier
      groups).

================================================================================
WEB / EXTERNAL VERIFICATION (dated; every external number tagged [EXTERNAL], ID-5/KD-1)
================================================================================
WV-1  arXiv:2606.27326 (the paper) — read in full this session; rho~=0.80 predictor-vs-error,
      action-shuffle-ratio<=1.1 = action-ignored, 50-trajectory finetune. [EXTERNAL/DESIGN-RATIONALE]
      — never an ELAI number; the CGG borrows the MECHANISM (3 converging predictors, do-test,
      risk-as-curiosity), not the figures.
WV-2  SelfCheckGPT arXiv:2303.08896, Semantic Entropy Nature s41586-024-07421-0, SAC3
      arXiv:2311.01740 — the P3 prior art (resample + cluster-by-meaning + cross-model). Cited as
      method prior-art; the L-10 done-gate is ELAI-MEASURED on an ELAI gold set, never a paper
      AUROC (ID-5). Re-verify each arXiv ID via WebSearch at the L-4/L-10 authoring-execution session
      (house rule; the IDs above are from the user's notes, not yet WebSearch-confirmed this session
      — flagged PENDING-VERIFY).

PENDING-VERIFY (resolve before citing in a shipped artifact): the three WV-2 arXiv IDs.

End of L-CGG-GROUNDING-MAP.
