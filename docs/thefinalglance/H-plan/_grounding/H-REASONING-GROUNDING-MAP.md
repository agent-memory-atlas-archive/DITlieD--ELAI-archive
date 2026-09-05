# H-REASONING-GROUNDING-MAP — Track 6 (GROUNDED-REASONING HARDENING) symbol map
Date: 2026-06-04. Author run: the Hallucination-Stations research thread (transcript AIYQp1n51ZI +
arXiv:2507.07505 ingest + a 3-angle deep-research sweep over the transformer-expressivity /
hallucination-inevitability / verification-vs-generation literature).
Purpose: the W-7 grounded symbol map for H-39..H-42. Every consumes_symbol / reachable_from anchor
in those files is grep-confirmed here against the LIVE 2026-06-04 source (elai.map + crates/**),
SHIPPED-vs-ABSENT classified. Read your H-N's slice + the CITATIONS + the RD-1..RD-4 deltas before coding.

WHY TRACK 6 EXISTS (one paragraph)
The video "They Lied to You About AI" (376k views) and its source paper arXiv:2507.07505 ("Hallucination
Stations", Sikka & Sikka, Jul 2025) argue LLMs have a hard O(N^2*d) per-forward-pass ceiling, so any task
above it must hallucinate, a verifier shares the ceiling (peer-verification "unsound"), and CoT "buys
nothing". A 3-angle grounded sweep (formal-proof + peer-reviewed) found: (a) the central CoT claim is
REFUTED — polynomial CoT reaches EXACTLY PTIME (2310.07923), T CoT steps simulate any size-T circuit
(2402.12875), and GD provably LEARNS NC1 with CoT (2511.07378); the bound is on a SINGLE PASS (~TC0), not
on the autoregressive loop; (b) hallucination inevitability is real but OVER-DETERMINED, and its most COMMON
form is low-complexity factual, driven by EVAL INCENTIVES that reward confident guessing over abstention
(OpenAI 2509.04664) — a mechanism ELAI's complexity/checker defenses do not touch; (c) the "verification is
as hard as the task" pessimism is true ONLY for a verifier CORRELATED with the generator — DECORRELATED weak
ensembles (Weaver 2506.18203) and SOUND external checkers (Kambhampati 2402.08115) measurably help; the
honest ceiling is the ORACLE verifier (Pass@K), approached not exceeded. The verdict against the LIVE code:
ELAI's trust spine is ALREADY strongly aligned with the rigorous literature (D-10/D-11/D-16/D-17 shipped and
correctly designed — see RS-1..RS-9). Track 6 is four small ACTIVATION/EXTENSION refinements + instruments on
that shipped substrate, NOT a redesign. This is the anti-sycophantic read: most of the finding is confirmation.

================================================================================
RS — SHIPPED reasoning/verification substrate (grep-confirmed 2026-06-04; COMPOSE-WITH, do not rebuild — GD-1)
================================================================================
RS-1  grounding_mode.rs:110 `select_grounding_mode(claim, ctx)` — D-10 BUILT + CALLED at agent_pool.rs:2030.
      Rule 5 (grounding_mode.rs:157) ALREADY routes Factual/Computational "constructive when possible,
      cross-family when not, refusal otherwise" = the sound-checker-first preference. Rule 1 routes
      Auditor/Meta -> ConservativeFallback = the auditor-never-grades-itself firewall. GroundingMode =
      {Constructive, FalsifiabilityCrossFamily, ConformalRefusal, OperationalDeferred, ConservativeFallback}.
      >>> KEY: the A>B preference is SHIPPED. H-41 does NOT add it; it INSTRUMENTS live coverage + the gap.
RS-2  checker_registry.rs `CheckerResult` (Pass/Error/timeout) + agent_pool.rs:2153 trait CheckerResultEmitter
      + recent_checker_results + commands_d11.rs `checker_results_recent` + CheckerResultRow (types.ts:3149) —
      D-11 BUILT (the constructive-checker lane + its SSB emission + dashboard read).
RS-3  benchmarks/mode_routing_runner.rs — D-21 BUILT: per-mode F1 + 5x5 confusion matrix over a FIXTURE corpus
      ("missing-class coverage degrades the score", :667). This is the OFFLINE accuracy benchmark; it is NOT a
      LIVE production coverage signal (that gap is H-41).
RS-4  conformal_calibration.rs — D-16 BUILT: RefusalDecision {Answer, Abstain}, ConformalRefusalRow, TauUpdater,
      persisted tau, decide/claim_score, VecConformalRefusalDecidedEmitter; CLI conformal_calibration_cli.rs.
      >>> The ABSTAIN capability EXISTS at inference. H-39's gap is that the PROMOTION/EVAL reward ignores it.
RS-5  refusal_sla.rs — D-17 BUILT: RefusalSlaConfig, refusal_rate, band_lo/band_hi (confabulating-vs-overcautious).
RS-6  promotion_gate.rs:109 `two_gate_promote(candidate, descriptor, archive, cfg)` + config_archive.rs
      (insert_variant/get_variant/select_parent, sigmoid election) + bai.rs:286 `sample(&[ArmPosterior])` +
      darwin_admit_loop.rs:298 (two_gate_promote in run_one_generation) + self_mod_layer1a.rs is_security_sensitive
      — the SELF-MOD PROMOTION LOOP, BUILT. Reward is score+tau-gate; NO abstention term (H-39 gap).
RS-7  coding_metrics/{collector,aggregator,types}.rs — pass_at_k / pass_at_1 / record_pass_sample, BUILT.
      Pure pass/total; NO abstention credit and NO confident-wrong penalty (H-39 gap). E-49 metrics partial.
RS-8  benchmarks/safedelta_runner.rs (run_safedelta_from_runner, bootstrap_ci, SafeDeltaTier2Config) +
      bench_seccodeplt.rs (SecCodePltRunner::from_embedded) + InsecureRateMeasured SSB — the FROZEN-runner +
      measured-SSB-row eval pattern to MIRROR (G-6/F-2b). The exact shape H-39's AbstentionScored row reuses.
RS-9  iso_budget_harness.rs — E-6 BUILT: IsoBudgetReport, `insufficient_lift` flag (:93), counts(),
      test_iso_budget_measured_row + commands_e6.rs mast_mode/iso_budget_recent dashboard. Flags a role by
      MARGINAL LIFT; NO error-decorrelation / unique-catch axis (H-40 gap).
RS-10 grc.rs:223 AgentGrcConfig (required_steps) + reasoning/config.rs:430-580 agent_grc_config_for_role
      (per-role STATIC budgets) + reasoning/types.rs:228 — GRC budget is per-ROLE, not per-serial-depth (H-42 gap).
RS-11 core/reasoning/self_certainty.rs `compute_score` + core/micro_reflection.rs `truncate_to_budget` +
      micro_reflection_emitter.rs (emit_micro_reflection, parse_reflection_json, SidecarReflectionAdapter) —
      E-19 BUILT: the L1-confidence + L2-reflection substrate H-42 scales.
RS-12 canary.rs CanaryFamily (CodeFix/MathProof/... all()) + benchmarks/canary_runner.rs — D-7 BUILT (the
      cross-domain regression families; grounding_mode.rs Rule 3 already refuses when the canary blocks).
RS-13 verification_result.rs `VerifierKind` {Deterministic, Intrinsic, Ppm, ...} + OrchContext.available_verifiers
      + has_cross_family_verifier — D-3 consensus substrate, BUILT (the verifier set H-40 measures decorrelation over).

================================================================================
RG — the four genuine gaps Track 6 closes (each confirmed by a code-read, not a summary)
================================================================================
RG-39 (H-39): the OPTIMIZED reward (two_gate_promote RS-6 / coding_metrics pass_at_k RS-7 / bench scorers RS-8)
      is ABSTENTION-BLIND — a confident-wrong outcome and a correct abstention score identically (or the abstain
      scores WORSE). The abstain capability exists (RS-4) but is unfed. So the self-mod loop (RS-6) + BAI (RS-6)
      can SELECT FOR confident guessing while D-16/D-17 (RS-4/5) try to suppress it at inference = a misalignment
      between two shipped subsystems. Mechanism + fix grounded in OpenAI 2509.04664 (binary-grade-with-abstention).
RG-40 (H-40): iso_budget_harness (RS-9) retains/prunes a role by `insufficient_lift` (marginal accuracy). Weaver
      (2506.18203) proves a verifier BELOW the generator's accuracy still raises an ensemble if its errors are
      DECORRELATED. So the lift-only flag can prune a decorrelated low-accuracy verifier that should be kept.
      Gap = no conditional error-decorrelation / unique-catch axis on IsoBudgetReport.
RG-41 (H-41): the sound-checker-first PREFERENCE is shipped (RS-1) and the OFFLINE routing benchmark exists
      (RS-3), but there is NO LIVE production signal for (a) the Mode-A (Constructive) fraction of real claims
      this session, nor (b) a checker_gap backlog naming which claim-classes fall to Mode B (cross-family LLM) or
      Refusal for LACK of a constructive checker. Per Kambhampati 2402.08115 (sound external verifiers work; LLM
      self/peer verification collapses on reasoning), maximizing Mode-A coverage is THE high-value lever — so it
      must be MEASURED and the gap RANKED. Gap = the live coverage ratio + the checker-gap instrument.
RG-42 (H-42): GRC budget is per-role STATIC (RS-10). The CoT-step lower bounds (2503.01544 ~O(n) CoT tokens for
      compositional reasoning; 2502.02393 tight step lower bounds) say required SERIAL steps scale with a
      sub-task's serial depth. A deep serial sub-task under a flat budget is FORCED to truncate = the failure the
      budget meant to prevent. Gap = no serial-depth scaling of the budget + no guard that a context-window lift
      is being mis-applied as the fix for a depth-class failure (the single-pass bound is ~TC0; access != depth).

================================================================================
RD — TRACK-6 DELTA RULES (additional to W-1..W-10 + GD-1/2/4/5 + HD-1..HD-5; all BLOCKING)
================================================================================
RD-1  ABSTENTION-CREDITED REWARD. Any reward an autonomous loop OPTIMIZES (promotion, BAI, GEPA, the bench
      scorers) MUST credit a correct abstention and penalize a confident-wrong strictly MORE than an abstention
      (explicit-confidence-target binary grading, 2509.04664). A reward that scores confident-wrong == abstain is
      a hallucination-BREEDING signal and is a FAIL. The abstain bit is READ from the shipped RefusalDecision
      (RS-4), never re-derived (GD-1).
RD-2  DECORRELATION-OVER-ACCURACY. A verifier/role is retained on its conditional error-decorrelation (unique
      catch given ground truth), NOT on standalone accuracy alone. Pruning a decorrelated low-accuracy verifier
      on a marginal-lift flag is a Weaver-class error (2506.18203). The honest ceiling is the ORACLE verifier
      (Pass@K) — an ensemble approaches, never exceeds it; never advertise "catches everything" (GD-5).
RD-3  SOUND-CHECKER-FIRST + COVERAGE-IS-THE-LEVER. A claim with an available sound constructive checker is NEVER
      graded by an LLM verifier instead (already enforced, RS-1/grounding_mode.rs:157). The LIVE Mode-A coverage
      ratio + the checker-gap backlog are the measured investment signal (2402.08115); every new checker that
      moves a claim-class from Mode B to Mode A is a measured win, recomputable from the journal (GD-2).
RD-4  DEPTH-NOT-WIDTH. Reasoning budget scales with a sub-task's estimated SERIAL depth, never with context-
      window size. A context-window increase is NEVER accepted as the fix for a depth-class reasoning failure:
      a single forward pass is ~TC0 (bounded parallel depth); the autoregressive CoT loop is the serial-compute
      escape, and its required step count is lower-bounded (2503.01544 / 2502.02393). Access != compute-depth.

================================================================================
CITATIONS (verified IDs; the four files cite these — re-confirm at execution per feedback_paper_citation_verify)
================================================================================
- arXiv:2507.07505 Sikka & Sikka "Hallucination Stations" (the target; O(N^2*d) ceiling — REFUTED on CoT).
  Interface map: workflow/papers/hallucination-stations-interface.md.
- arXiv:2310.07923 Merrill & Sabharwal "The Expressive Power of Transformers with CoT" (ICLR 2024; poly CoT = P).
  workflow/papers/expressive-power-transformers-cot-2310.07923-interface.md.
- arXiv:2402.12875 Li/Liu/Zhou/Ma "CoT Empowers Transformers to Solve Inherently Serial Problems" (ICLR 2024).
- arXiv:2511.07378 Huang et al "Transformers Provably Learn CoT with Length Generalization" (NeurIPS 2025).
- arXiv:2503.01544 Yehudai/Amsel/Bruna "Compositional Reasoning with Transformers, RNNs, and CoT" (NeurIPS 2025;
  ~O(n) CoT tokens). workflow/papers/compositional-reasoning-transformers-rnn-cot-2503.01544-interface.md.
- arXiv:2502.02393 Amiri/Huang/Rofin/Hahn "Lower Bounds for CoT Reasoning in Hard-Attention Transformers" (ICML 2025).
- arXiv:2509.04664 Kalai/Nachum/Vempala/Zhang (OpenAI) "Why Language Models Hallucinate" (Sept 2025; calibration +
  eval-incentive mechanism; binary-grade-with-abstention recommendation). [RD-1 anchor; H-39 keystone citation.]
- arXiv:2506.18203 Saad-Falcon et al "Shrinking the Generation-Verification Gap with Weak Verifiers" (Weaver,
  NeurIPS 2025; decorrelated ensemble 87.7% > o3-mini; never beats Pass@K). workflow/papers/weaver-2506.18203-interface.md.
  [RD-2 anchor; H-40 citation.]
- arXiv:2402.08115 Stechly/Valmeekam/Kambhampati "On the Self-Verification Limitations of LLMs on Reasoning and
  Planning" (self-critique collapses; sound EXTERNAL verifiers work). [RD-3 anchor; H-41 citation.]
- arXiv:2401.11817 Xu/Jain/Kankanhalli "Hallucination is Inevitable" (learnability/diagonalization inevitability).
  workflow/papers/hallucination-inevitable-2401.11817-interface.md (ingested 2026-06-04).
- arXiv:2502.12187 Suzuki/He/Tian/Wang "Hallucinations are inevitable but can be made statistically negligible"
  (worst-case vs real-distribution — grounds GD-4 FP-bounding). workflow/papers/hallucination-negligible-2502.12187-interface.md.

================================================================================
SCHEMA (LIVE 2026-06-04, re-read at edit — W-9)
================================================================================
SCHEMA_VERSION = 220 (crates/orchestrator/src/ssb.rs:2990). TOTAL_VARIANTS = 497
  (crates/orchestrator/tests/action_type_lock.rs:624). Track-4 moved the baseline here; the 207/484 in the
  H-0 Track-1/2/3 header and CLAUDE.md's 191/464 are STALE. The e45_cross_language_drift_contract variant pin
  (~482) is still stale; the FIRST Track-6 H-N to bump SCHEMA reconciles it.
Track-6 deltas (ship-order-cumulative off 220/497; confirm the live integer at each edit):
  H-39 +AbstentionScored (Pattern-2; the abstention-adjusted promotion/eval row). baseline+1.
  H-40 NO new action_type — EXTEND IsoBudgetReport + reuse the shipped iso_budget_measured row (add a
       decorrelation/unique_catch field to its payload). GD-1 compose, minimise bumps.
  H-41 +CheckerGapDetected (Pattern-2; the claim-class-fell-to-Mode-B/Refusal-for-lack-of-checker signal);
       the live Mode-A coverage RATIO is a roll-up over the shipped CheckerResultEmitted + grounding-mode rows
       (journal-recomputable, GD-2 — no variant for the ratio). baseline+1.
  H-42 +ReasoningBudgetScaled (Pattern-2; the serial-depth-scaled budget decision). baseline+1.
Net if all ship = +3 variants.

End of H-REASONING-GROUNDING-MAP.md.
