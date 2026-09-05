# L-Plan Index — wiring-enforced (24 sessions L-1..L-24, five tracks + one repair file: L-T1 = CAUSAL-GROUNDING-GATE / hallucination L-1..L-10; L-T2 = OUTPUT-QUALITY + FAMILIARITY-SURFACE L-11..L-14; L-15 = repair controller + evidence-first reorder; L-T3 = CONTEXT-CORE L-16..L-19; L-T4 = MEASURED-COMPETITOR / EVAL-SUPPLY L-20..L-21; L-T5 = DEEPREASON-STEAL / EXPLORATORY-REASONING L-22..L-24; authored 2026-06-27 / 06-30 / 07-03 / 07-08 / 07-11)

Date: 2026-06-27 (two authoring waves, main LLM in-band per the paper-to-plan PHASE 5 shape + spec-eval AS-SPECIFIED; no fleet; live anchors carried from ELAI-FEATURE-MAP.md + the K-12 code-grounding pass, every anchor re-grepped at edit per W-7, corrections in _grounding/L-CGG-GROUNDING-MAP.md (T1) + _grounding/L-OQF-GROUNDING-MAP.md (T2)).
Date: 2026-07-08 (L-T5 = the DeepReason steal, sessions L-22..L-23; paper-to-plan PHASE 5, main LLM in-band; source = the PHASE-4 steal artifact workflow/reasoning/steal-deepreason-2026-07-08.md; VS citation arXiv:2510.01171 WebSearch-verified; ELAI anchors carried from ELAI-FEATURE-MAP.md + a live elai.map grep, re-grepped at edit per W-7; per-track grounding map deferred to the L-T3/L-T4 house backfill precedent, GROUNDING blocks live inline in L-22/L-23).
Date: 2026-07-11 (L-T5 extended with session L-24 = the DeepReason COLLAB wave; source = the LIVE exchange with the DeepReason author recorded in workflow/handoff/deepreason-steal-and-collab.md CURRENT STATUS improvements (a)-(d); the author's one real hit on ELAI ("proven-only case minting is clean but sterile") adopted as the conjecture-memory middle tier for the EXPLORE lane only, the proven-only code-fix path locked UNCHANGED; plan-gate-check PASS at authoring; leg-D PROOF_SPINE none user-acked 2026-07-11).

USER OVERRIDE (recorded honestly): K-0_INDEX.md K-10 SEQUENCING DOCTRINE defers any NEW plan LETTER until A-1..A-4 + K-3 are DONE. The user explicitly authorized the L-letter ("create the next letter after K") for L-T1, and explicitly authorized the L-T2 track this session ("create new L plan files as necessary"). Both are logged deviations, not silent ones; the K-10 priority order otherwise stands. L-T2 is a DISTINCT theme (output-quality enforcement + the user-extensibility/one-shot surface) sharing the L letter as a numbering namespace (the feature map groups by capability, never by letter).
NAMING NOTE: the LACUNA-FILL-PROGRAM uses "L1..L8" as INTERNAL section labels inside J-plan files (J-87..J-94). Those are NOT this plan. L-plan units are written "L-N" (L-1..L-10) and live under docs/thefinalglance/L-plan/; there is no collision in the file namespace.

Source synthesis:
- arXiv:2606.27326 "Hallucination in World Models is Predictable and Preventable" (Hansen & Wang, UC San Diego) [LS-1] — read in full; the 3-converging-predictors thesis, the action-marginalization do-test, the risk-as-curiosity mitigation. EXTERNAL/DESIGN-RATIONALE only (ID-5).
- the user's harness notes (Downloads/hallucination_detection_harness_notes.md) [LS-2] — the LLM port: claim>evidence>disagreement>exec-grounding>risk>policy, model-agnostic, the evidence hierarchy, the "causal must be observable" warning.
- the elai-lang causal screenshots [LS-3] — counterfactual/predictive/contrastive causal learning, per-connection causal_confidence (EMA), pruning spurious connections.
- workflow/reasoning/cgg-hallucination-causal-system-2026-06-27.md [LS-4] — this run's steal ledger + design synthesis + adversarial pass.
- workflow/reasoning/steal-deepreason-2026-07-08.md [LS-5] (L-T5) — the PHASE-4 DeepReason steal artifact (github.com/AHepi/DeepReason harness-spec-v1.3 + CAN_LLMS_EXPLORE + MiniReason source, spec-eval AS-SPECIFIED); S1 claimant-authored falsifiers + demarcation, S2 gate-block-rate orbit detector, S3 negative case law, S5 Verbalized Sampling, S8 the /reason explore surface. arXiv:2510.01171 (Verbalized Sampling) WebSearch-verified 2026-07-08. All DeepReason figures n=1 single-seed = DESIGN-RATIONALE only (ID-5).
- workflow/handoff/deepreason-steal-and-collab.md [LS-6] (L-T5, L-24) — the LIVE collab exchange with the DeepReason author (2026-07-11): the "clean but sterile" memory hit + the four improvements (a) explore-bench novelty metric, (b) typed unblock conditions on refusals, (c) the conjecture-memory middle tier, (d) the flywheel diversity guard; prior art = his spec §4 status model (computed labels, orphaned != false, N1 no-absorbing-status) + §10 case-law + §11.5 atlas split.

Grounding map (W-7 source): _grounding/L-CGG-GROUNDING-MAP.md — TS-1..TS-22 (substrate-shipped, COMPOSE-not-restate), NN-1..NN-7 (net-new builds), HG-1..HG-4 (the new delta family), DC-1..DC-3 (map corrections), WV-1..WV-2 (external verification, PENDING-VERIFY arXiv IDs). LIVE baseline ~268/577 (re-grep at edit).

Predecessors: B-plan .. K-plan. L COMPOSES-WITH them (GD-1); restates none. L is the first plan that points the trust spine OUTWARD — the prior letters built verification of the AGENT'S OWN code; L turns the same machinery into a model-agnostic gate over ANY model's CLAIMS, on ANY backend (B-37). It is ~85% activation + unification of shipped/planned units + ~15% net-new (the typed ClaimRisk object, the semantic-entropy predictor, the do-test-on-claims, the risk-as-curiosity router).

Predecessor reading (mandatory before any L-N execution):
1. _grounding/L-CGG-GROUNDING-MAP.md (the whole map — its DC corrections OVERRIDE any stale feature-map label).
2. F-0_INDEX.md W-1..W-10 canon; G-0_INDEX.md GD-1..GD-5; the D-plan reasoning/grounding units D-3/D-10/D-11/D-13/D-16/D-17 (the substrate L composes); H-39 abstention; K-16 reasoning-receipts (RR-1 trace-not-admissible-as-confidence — L inherits it).
3. cgg-hallucination-causal-system-2026-06-27.md (the steal ledger + design), then this file, then L-SUMMARY_INDEX.md, then the L-N detail file.
4. Per-session: the named prior-plan files the L-N composes (each re-grepped live, W-7).

## Why L-plan exists (read this before anything else)

L-T1 — CAUSAL GROUNDING GATE (L-1..L-10). Authored 2026-06-27 from [LS-1..LS-4]. CENTRAL GROUNDED FINDING: the paper's thesis (hallucination is a low-coverage/epistemic problem, PREDICTABLE from label-free runtime signals and PREVENTABLE by spending effort where coverage is weak, with no retraining; three mechanistically-distinct predictors that CONVERGE are trustworthy; the strongest causal signal is INTERVENTIONAL) is precisely ELAI's trust spine pointed at a new target. ELAI already ships ~85% of the architecture — its spine polices the agent's OWN code wiring today; the CGG points it at ANY model's CLAIMS, on ANY backend. So most of this plan is ACTIVATION (D-3/D-10/D-13/D-16/G-7 are DARK behind flags, not absent) + UNIFICATION (one typed ClaimRisk object fusing signals that exist scattered) + four genuine BUILDS: L-1 typed claim extraction, L-2 the ClaimRisk fuse [KEYSTONE], L-4 the semantic-entropy predictor (the one net-new signal), L-5 the do-test-on-claims [KEYSTONE]. The mitigation half (L-6 risk-as-curiosity) and the research/math deep-think profile (L-9) and the measured delta (L-10) close it. Delta family ADDED: HG-1..HG-4 (hallucination-grounding doctrine — causal-means-interventional / convergence-not-single-signal / risk-is-black-box / abstain-or-verify-never-silently-pass).

L-T2 — OUTPUT-QUALITY + FAMILIARITY SURFACE (L-11..L-14). Authored 2026-06-27 from output-quality-and-familiarity-surface-2026-06-27.md (spec-eval design synthesis, no external paper; SOLID/P10 = textbook design-rationale only, ID-5). CENTRAL GROUNDED FINDING (two of them): (1) OUTPUT QUALITY — ELAI has NO by-name SOLID/quality enforcement on the code it generates, but it already ships the machinery (C-32 gen-time MiniCritic loop, C-31 plan-time shape budget, maintainability_check.py auditor MR-1..MR-4, the PLANNED J-65 power-of-ten + J-23 review-catalog over generated code). So output-quality is a 4-SEAM layered gate (plan-time C-31 / gen-time C-32 / write-time MR-floor / audit-time frozen checker), NOT one new gate; the ONE net-new piece is solid_proxy_check.py (SRP god-object BLOCK via B-29 fan-out; DIP/ISP WARN via elai.map; OCP/LSP stay cross-family advisory) + a god-object MiniCritic predicate; the VERDICT is auditor-owned + frozen (OQ-1, never executor-self — Berkeley RDI in-boundary gamed ~100%); the any-model quality lever is the gen-time self-correction (OQ-5, paired with J-55). (2) FAMILIARITY SURFACE — the hooks/skills/MCP/one-shot surface a CC/Codex user expects EXISTS and is in places stronger than CC (hooks intercept at the IPC layer, agents never see them; rules are DATA with a lifecycle), but the user-facing FRONT DOOR is mostly DARK/PLANNED (E-43 skill system DARK, log_skill_invocation no prod caller; no user-rule authoring surface; the one-shot builder unsurfaced). So L-T2 is ACTIVATION (light the dark doors) not invention: L-12 skill front-door + self-improving skills (E-43 + I-12 + F-6), L-13 user-authorable rules/hooks (rules-as-DATA over 3F + the hook system + J-52), L-14 one-shot verified builder (pipeline-factory + E-12 + J-63 + I-40 receipt + J-2). The two audiences (advanced/trust + lazy/one-shot) are the SAME engine at two surfaces; the lazy path ships the RRA receipt (FS-3). Delta families ADDED: OQ-1..OQ-5 (output-quality), FS-1..FS-4 (familiarity-surface).

L-T5 — DEEPREASON-STEAL / EXPLORATORY-REASONING (L-22..L-23). Authored 2026-07-08 from [LS-5] (the DeepReason paper-to-plan PHASE-4 artifact). CENTRAL GROUNDED FINDING: DeepReason is the SECOND independent confirmation of ELAI's harness>model thesis (the first was the codebuff read, L-20), this time in the open-ended-reasoning domain where ELAI's corpus is thinnest — a cheap model inside a deterministic epistemology harness >= a frontier model raw at ~1% cost on the question class the harness fits. So L-T5 is ~90% COMPOSITION of shipped/planned units, not invention. Its six mechanisms map cleanly onto ELAI: claimant-authored falsifiers + a demarcation gate are the GENERATION-TIME half of the CGG the L-T1..L-T4 VERIFY-time machinery never had (L-22, EXTENSION of L-1/L-2/D-11 + the E-35/N-13 grammar mask so the weak lane pays no adherence cost); Verbalized Sampling is a prompt-SHAPE diversity lever on I-4 (L-22, arXiv:2510.01171); the gate-block-rate orbit detector is a 5th anti-spiral StagnationPattern that his own data showed beats the embedding detector (L-23 S2); negative-case-law-at-the-gate-never-in-prompt is a doctrine refinement to I-5 (L-23 S3, complement directives measured a diversity placebo); and the flagship is a `/reason --profile explore` surface ELAI had no lane for — hard open question -> a MAP of rival explanations with refutation status — whose LOCAL-7B leg (grammar-masked skeleton + frozen-prefix KV reuse, E-35/N-13 + H-2/I-10 + J-36) is the integration smoke for a whole substrate and a demo no competitor and not even DeepReason has run (L-23). REJECTED (steal §4.3, feature-map ledger): the Dung argument-graph adjudicator (ELAI verdicts are execution-grounded), the untyped-artifact ontology, embedding convergence detection (his own data shows it failing), and DeepReason's schools/lineage machinery (DEFERRED — the cheap 80% is stance rotation + cross-family dispatch). Delta families ADDED: HG-5 (demarcation) folds into the existing HG family; EX-1..EX-4 (exploratory-reasoning) earned per R-j for the explore surface's generator/adjudicator-capture failure modes the prior families do not cover. EXTENDED 2026-07-11 (L-24, the COLLAB wave [LS-6]): the author's counter-read of ELAI landed one real hit — proven-only case minting is "clean but sterile"; a memory that only holds what current evaluators recognize cannot hold ideas AHEAD of the receipt plane. L-24 adds the conjecture-memory MIDDLE tier between the journal and the verified case graph (J-96..J-98 UNCHANGED — locked domain split, explore lane ONLY, type-level lane wall): unproven conjectures as first-class retrievable objects with COMPUTED status (live/refuted/pending/suspended — a pure journal fold, orphaned != false, no absorbing status, quarantine never delete), provenance, attached criticisms, declared missing tests; composes N-16's evaluator-version edges (a refuted evaluator collapses what it vouched for) + K-19 SF-1..SF-4 (memory-reuse class). Plus the three collab riders: the L-10 explore-arm NLI meaning-cluster novelty column under CGG ON/OFF (leg A), typed unblock conditions on D-17/L-7 refusals composed from receipts (leg B — every abstention becomes a pending conjecture about its own answer), and the FD-1..FD-3 flywheel diversity guard as a contract-first pre-gate on I-30/I-31 (leg D, PROOF_SPINE none user-acked). Delta rules ADDED: EX-5/EX-6 into the existing EX family.

## Status legend
- PLANNED — file written, work not started.
- IN PROGRESS — being shipped in an active session.
- DONE — the FULL wiring done-gate for the session's nature. cargo-green / file-exists is necessary, NEVER sufficient (W-1).
- DEFERRED — blocked on hardware (RunPod/2x3090), an upstream live-status check, or a dependency session.

================================================================================
==== THE WIRING DOCTRINE ====
================================================================================
INHERITS W-1..W-10 VERBATIM (canon: F-0_INDEX.md) — done = real-entrypoint run-trigger PASS not build-green (W-1); kill-switch-not-enable-flag (W-2); no summary-only ports (W-3); frontend_pair where dashboard-surfaceable else justified N/A (W-4); live end-to-end trigger with RunPod-gated fixture split (W-5/W-10); re-grep anchors at edit (W-7); prefer-reuse SSB families incl re-reading live constants (W-9); the off-agent integration smoke on a HARD input (R10).
INHERITS GD-1/GD-2/GD-4/GD-5 (canon: G-0_INDEX.md) — compose-not-restate (an L file re-implementing G-7 claim_score, D-16 refusal, D-3 aggregation, F-4 cant-lie, citation_prebind, recover_stubs, or BeliefState is a P0); every emitted risk number recomputable from its journal source (GD-2); no gate defaults to blocking before its FP/over-block rate is bounded on >=3 real runs (GD-4 — the WHOLE CGG ships observe-only first); honest residue named in-file (GD-5 — the CGG catches coverage-gap hallucination, NOT confident-shared-wrong beliefs; that limit is named everywhere).
INHERITS TD-3 (LLM never adjudicates), ID-5 (every threshold/knee ELAI-measured, never a paper number), HD-2 (deterministic-reversible only, no generative round-trip), RR-1 (a reasoning TRACE is never admissible as the confidence/selection signal — only an external receipt; canon K-16). These bite the CGG hard and are restated as ACCEPTANCE checkboxes per session.

NEW DELTA FAMILY — HG (hallucination-grounding; earned per template R-j because the prior families cover RUNTIME wiring + MARKETING/DISTRIBUTION surfaces, not the CLAIM-RISK-SCORING and CAUSAL-EVIDENCE failure modes L introduces):
  HG-1 CAUSAL-MEANS-INTERVENTIONAL. A claim is labelled "causal" ONLY on interventional/executable evidence (a counterfactual do-test or a runtime trace); static/temporal/semantic evidence is scored at its weaker tier and NEVER called causal. Rationale: LS-2's explicit fake-precision warning ("A imports B != A causes bug B") + the paper's §4.1.ii do-test. Binds L-2, L-5.
  HG-2 CONVERGENCE-NOT-SINGLE-SIGNAL. A high/critical-risk verdict requires >=2 mechanistically-distinct predictors to agree; a single predictor may caveat but never blocks. Extends D-3. Rationale: paper §4.2 (convergence IS the evidence the signal is real; kills cost + false alarms). Binds L-2.
  HG-3 RISK-IS-BLACK-BOX. Every risk signal works without model internals; logprobs/white-box probes are OPTIONAL boosters never dependencies, so the gate runs identically on every B-37 backend (cloud/local/BYOK/free-pool/OpenRouter/CLI-bridge). Rationale: LS-2 model-agnostic principle. Binds L-1/L-2/L-3/L-4.
  HG-4 ABSTAIN-OR-VERIFY-NEVER-SILENTLY-PASS. An unverifiable high-risk claim routes to targeted verification or abstention, never a silent allow; missing evidence = high risk (wiring R1 applied to claims). Extends D-16/D-17. Binds L-6/L-7.
  HG-5 DEMARCATION-OR-HIGH-FLOOR (L-T5, earned per R-j for the generation-time attack surface). A Causal/Behavior/Reachability claim that declares NO falsifier has an empty attack surface and scores a High risk-floor; a hypothesis-class GRC-v3 step with no declared falsifier may not transition to Converge. Rationale: DeepReason's Popperian forbidden[] makes demarcation structural at zero LLM cost — a claim with no possible refuter is decoration, not science (steal-deepreason S1). Binds L-22, L-23.

NEW DELTA FAMILY — OQ (output-quality; earned per template R-j — prior families cover RUNTIME wiring + CLAIM-RISK + MARKETING, not the QUALITY-of-generated-code failure mode L-T2 introduces; binds L-11):
  OQ-1 VERDICT-IS-AUDITOR-OWNED-NEVER-SELF. The authoritative SOLID/maintainability verdict is rendered by the auditor over the changeset + frozen in gate-manifest (R8), NEVER by the executing agent over its own output (Berkeley RDI Apr-2026 in-boundary gamed ~100%; maintainability-review.md ROLE BOUNDARY). The gen-time MiniCritic is self-CORRECTION (retry), explicitly not the verdict.
  OQ-2 FLOOR-VS-CEILING SPLIT. The deterministic FLOOR (MR-1 file>1000, MR-2 as-any, god-object) may sit at write-time (VibeGate); the advisory CEILING (SOLID/OCP/LSP) lives at audit-time. SOLID is NEVER a hard PreToolUse block (thrash guard).
  OQ-3 OBSERVE-FIRST + MEASURED-FP. Every quality rule enters the rules-engine SHADOW lifecycle observe-only; it promotes to BLOCK-by-default ONLY on fpr<5%/tpr>80% over >=3 real runs (GD-4 + J-43's debt-ledger+rate).
  OQ-4 MECHANIZABLE-PROXY-ONLY. SRP (fan-out+size) + DIP (cross-layer concrete-vs-trait) are deterministic proxies via B-29/elai.map; ISP (fat-trait) is weak-WARN; OCP + LSP are NOT deterministically checkable -> cross-family ADVISORY (J-89), never a block (honest residue, GD-5).
  OQ-5 FIX-AT-GENERATION-NOT-JUST-BLOCK-AT-AUDIT. The any-model quality lever is the C-32 gen-time MiniCritic + StructuralRetry (it catches AND fixes weak-model output in-loop), paired with J-55 routing the weak lane to gate-passable constrained generation.

NEW DELTA FAMILY — FS (familiarity-surface; earned per template R-j — prior families do not cover the USER-EXTENSIBILITY-surface failure modes; binds L-12/L-13/L-14):
  FS-1 USER-EXTENDS-VIA-DATA-NOT-RECOMPILE. A user adds/edits a skill (SKILL.md) or a rule/hook (a typed predicate+verdict) as DATA on the WIRED E-43/3F substrate, no recompile.
  FS-2 EVERY-USER-SURFACE-PASSES-THE-SAME-TRUST-GATES. A user skill/hook/rule cannot bypass skillspector / J-102 config-ingress admission / VibeGate F-1 / wiring; it is DATA evaluated by the existing engine, NEVER arbitrary code; a user rule cannot WEAKEN a mutable=0 immune-tier check (R8).
  FS-3 TRUST-SPINE-ATTACHED-TO-THE-LAZY-PATH. The one-shot builder output SHIPS an RRA receipt (I-40); the lazy product IS the trust product with a simple front end, never a second dumber product. The value claim stays gated on a measured SAFEDELTA/QDELTA number (KD-1/ID-5), never asserted.
  FS-4 FORMAT-COMPATIBILITY. agentskills.io / CC SKILL.md frontmatter accepted verbatim + MCP both directions (consume + serve via J-2), so a CC/Codex user keeps their existing workflow.

NEW DELTA FAMILY — EX (exploratory-reasoning; earned per template R-j — the /reason explore surface has generator/adjudicator-capture failure modes W/GD/HG/OQ/FS do not cover; binds L-23):
  EX-1 DIVERSITY-IS-STRUCTURAL-NOT-PROMPTED. Diversity comes from stance rotation / problem turnover / tail-weighted VS, NEVER from a complement "avoid X / be different" directive (measured a placebo — it moves output relative to the PROMPT, not out of the model's REPERTOIRE). Refuted/banned content is tabu at the REGISTRATION gate (I-8 hash + the orbit counter), NEVER rendered into a prompt (rendering it primes the very content it bans and grows unbounded); the I-5 contrastive constraint stays ONLY for typed error-avoidance, never as a diversity instruction. Rationale: steal-deepreason S3. Binds L-23.
  EX-2 ORBIT-IS-A-GATE-RATE-NOT-AN-EMBEDDING. Refuted-attractor orbiting is detected by the RATE of gate blocks / dead-end prunes / relapse refusals per window keyed by refuted-target (StagnationPattern::Orbiting), ZERO embeddings — his gate-block counter separated healthy from orbiting perfectly where the embedding detector could not. Rationale: steal-deepreason S2. Binds L-23.
  EX-3 SURVIVOR-MAP-NOT-SINGLE-ANSWER. The explore deliverable is a MAP of rival explanations with their refutation status + the deciding-evidence gaps, never a single collapsed answer. Rationale: the explore question class, steal-deepreason S8. Binds L-23.
  EX-4 LOCAL-LANE-BY-CONSTRUCTION. The local-7B conjecturer's skeleton is valid BY CONSTRUCTION via the E-35 grammar mask on a frozen-prefix pack (H-2/I-10 + J-36 KV reuse), not by template-following — the fix for the <50B adherence failure DeepReason could not ship. Rationale: steal-deepreason S3.1/S3.2. Binds L-23, L-22.
  EX-5 UNPROVEN-NEVER-CROSSES-THE-LANE (L-24, earned per R-j — the conjecture tier introduces a contamination surface no prior family polices). An unproven conjecture is structurally unretrievable into the code-fix / auditor / case-graph path: type-level lane token + a journaled Retrieved audit row + a negative contract test; the J-96 proven-only minting filter is UNTOUCHED. Rationale: the locked domain split (collab decision 2026-07-11); SF-3 never-into-the-auditor. Binds L-24.
  EX-6 STATUS-IS-COMPUTED-NEVER-CURATED. A conjecture's status is a deterministic fold over journal rows (byte-recomputable under ELAI_SEED), never a stored mutable field or a hand-set label; refuting a premise SUSPENDS dependents (orphaned != false); no status is absorbing (a fallen refuter reinstates its target via the N-16 S4 edge); contradicted/poison-suspect nodes are QUARANTINED, never deleted. Rationale: DeepReason spec §4 + N1; SF-4. Binds L-24.

DONE DEFINITION (the FSM may not reach COMPLETE on an L-N without):
1. For BUILD sessions (L-1..L-8 runtime legs): cargo check --workspace GREEN + auditor PASS on the affected crates only (-p) + wiring_contract_check.py --bn-file <L-N> --run-trigger PASS + cross_lang_diff if frontend_pair + the GD-4 FP/over-block bound on >=3 runs where a gate defaults ON + the HG checks that bind.
2. For the profile/bench sessions (L-9 compose, L-10 instrument): the live trigger runs the real entrypoint and the off-agent artifact lands (the deterministic FIXTURE tier ships now; the live-model arm is RunPod-gated, W-10); the bench number is ELAI-measured + journal-recomputable (GD-2), never a paper number (ID-5).
3. workflow/reasoning/L-N-decisions.md exists.
4. memory-writer ran (MEMORIZING never skipped).

================================================================================
==== THE 14 SESSIONS (L-T1 = L-1..L-10 CGG; L-T2 = L-11..L-14 OUTPUT-QUALITY + FAMILIARITY) ====
================================================================================
  L-N  title                                        lever / nature                 composes-with                               effort  wave
  L-1  typed claim extraction substrate             GROUNDED / BUILD (foundation)  report_fns role-class,D-10,G-1,grounding-sweep   M     W1
       (deterministic structural typing of any model output > ClaimSet{ClaimType,span,refs}; ClaimType in {File,Symbol,Dependency,Api,Behavior,Causal,Reachability}; off-agent. HG-3 black-box. The intake every predictor reads.)
  L-2  unified ClaimRisk score + convergence fuse   GROUNDED / BUILD [KEYSTONE]    G-7,D-3,H-40,H-39,D-16,J-84,J-85               M-L   W1  depends L-1
       (generalize G-7 claim_score from the agent's own output to ANY backend; fuse P1/P2/P3 under HG-2 >=2-agree; route the expensive layer only on cheap-signal medium-risk via J-84. The keystone object.)
  L-3  evidence round-trip predictor (P1)           GROUNDED / BUILD-on-shipped    D-10/D-11,H-53,H-32,DKG                        M     W2  depends L-1
       (claim<>evidence reconstruction: does a claim survive re-grounding against repo/source/ingested-paper; closed-set citation membership; missing evidence = high residual. ACTIVATION over arbitrary claims.)
  L-4  semantic-entropy inter-seed predictor (P3)   GROUNDED / BUILD (net-new)     I-4,D-3,D-13,D-7,F-12 NLI                      M-L   W2  depends L-1,L-2
       (resample N, cluster by MEANING via the F-12 DeterministicNliJudge — NOT cosine; emit entropy; cross-family via D-13. The one genuine net-new predictor. SelfCheckGPT/semantic-entropy port. ID-5 done-gate.)
  L-5  counterfactual do-test on model claims       CANT-LIE / EXTENSION [KEYSTONE] F-4,C-28/I-20,C-29,CSQ,J-16,G-1                M-L   W2  depends L-1
       (when a model claims A causes/fixes B, run the intervention — revert/disable/patch-only-A/add-trace — and observe whether B changes. The paper's action-marginalization do-test on claims. HG-1.)
  L-6  risk-as-curiosity targeted-effort router     GROUNDED / BUILD               J-38,J-103,I-7,B-29,J-84                       M     W3  depends L-2
       (high risk auto-fires targeted retrieval/test/trace/escalation at the WEAK claim only, then re-scores. The paper's §4.3 mitigation ported. HG-4.)
  L-7  hallucination policy gate + abstention       CANT-WRECK / BUILD-compose     D-16,I-8,D-17,B-22 AutonomousGate,G-2          M     W3  depends L-2,L-6
       (risk > allow/caveat/force-tool/abstain wired into AutonomousGate EvidenceCheck + conformal refusal + Aletheia abort + honest-refusal SLA; the verdict is an SSB row, RRA-reproducible. HG-4.)
  L-8  working-memory hygiene / prune spurious      CANT-WRECK / BUILD-compose     F-5,J-21,I-5,J-83 BeliefState                  M     W3  depends L-1
       (prune false repo assumptions from the live claim/belief graph before they poison later steps: staleness quarantine + memory validation + pristine-context. The elai-lang prune.)
  L-9  research/math deep-think grounding profile   GROUNDED / EXTENSION           B-39 /reason,D-11/E-53,B-42/J-33,L-2..L-5     S-M   W4  depends L-2,L-3,L-4
       (a /reason N=4 profile in GRC Mode A constructive-checkers over Cosmos-ingested papers, CGG P1/P3 over the reasoners; token-perfect math = the symbolic checker adjudicates, TD-3. A profile, NOT a new engine.)
  L-10 CGG hallucination-delta bench                GROUNDED / BUILD-instrument    G-6 SAFEDELTA,I-38 QDELTA,I-27,L-1..L-7        M     W4  BENCH-PENDING (live arm RunPod-gated)
       (`elai bench hallucination-delta`: gate-ON-vs-OFF unsupported-claim rate + reviewer-reaching rate on a held-out claim corpus, ELAI_SEED-recomputable, receipts on the y-axis; the done-gate number. NEVER a paper AUROC, ID-5.)

  ---- TRACK L-T2 — OUTPUT-QUALITY + FAMILIARITY SURFACE ----
  L-11 output-quality enforcement system            OQ / BUILD+COMPOSE [KEYSTONE]  C-31,C-32,maintainability_check,J-65,J-23,B-29,elai.map,rules-3F,F-1,I-38   L   W1
       (SOLID-proxy frozen checker (SRP god-object BLOCK / DIP+ISP WARN; OCP+LSP advisory) + a god-object MiniCritic predicate, wired across the 4 seams: plan-time C-31 / gen-time C-32 self-correct / write-time MR-floor / audit-time auditor authority; SHADOW->ACTIVE promotion; verdict auditor-owned + frozen (OQ-1). The one net-new = solid_proxy_check.py.)
  L-12 skill front-door + self-improving skills      FS / ACTIVATION+COMPOSE        E-43,F-6,I-12,J-102,skillspector,context_budget-Rules-slot                 M   W2
       (light E-43 log_skill_invocation prod caller + `elai skill add/improve` gated by skillspector+J-102+VibeGate; wire the I-12 episodic->skill->rule curator; CC/agentskills.io SKILL.md verbatim (FS-4); every skill passes the SAME gates (FS-2).)
  L-13 user-authorable rules & hooks surface         FS / ACTIVATION+COMPOSE        rules-3F,HOOK SYSTEM,J-52,J-27,J-102,F-1                                   M   W2
       (`elai rule add/list/test/promote/retire` writing rules-as-DATA rows + J-52 path-keyed hook registration; the CC-hook familiarity done safely — DATA predicate+verdict never code (FS-2), SHADOW-first (OQ-3/GD-4), cannot weaken a mutable=0 immune check (R8).)
  L-14 one-shot verified builder front-door          FS / ACTIVATION+COMPOSE        pipeline-factory L1/L2,E-12,J-63,I-40,J-2,L-11,AutonomousGate              M   W3  depends L-11
       (`elai build "<idea>"` drives the pipeline -> E-12 codegen -> L-11 quality gate -> I-40 wrap; the output ALWAYS ships an RRA receipt (FS-3); reachable as a J-2 MCP verb (FS-4); the value claim gated on a measured number (KD-1/ID-5), never asserted.)

  ---- TRACK L-T5 — DEEPREASON-STEAL / EXPLORATORY-REASONING (L-22..L-24) ----
  L-22 claimant-authored falsifiers + demarcation + VS  HG-5/EX / BUILD-on-shipped     L-1,L-2,D-11,E-35,N-13,I-4,J-83,arXiv:2510.01171   M-L   W1  depends L-1,L-2
       (an optional falsifier field on the L-1 ClaimSet elicited grammar-enforced at GENERATION time (zero weak-lane cost) + a DEMARCATION fuse into L-2 (forbids-nothing => High floor, HG-5) + the no-falsifier-no-Converge GRC gate + the Verbalized-Sampling propose contract (one distribution call vs I-4's N; tail-weighted only under the orbit flag; typicality advisory TD-3). The generation-time half of the CGG.)
  L-23 /reason --profile explore (conjecture-criticism)  EX / NEW surface [KEYSTONE]   B-39,L-22,anti-spiral,I-8/I-26,D-13/J-45,I-24/N-15,E-35/N-13,H-2/I-10/J-36,K-11/N-7   M-L   W2  depends L-22
       (hard open question -> a MAP of rival explanations with refutation status; composes VS conjecture spread + stance rotation + claimant falsifiers + the S2 gate-block-rate orbit detector (5th StagnationPattern) + problem turnover + cross-family survivor judging; the LOCAL-7B arm (grammar-mask + frozen-prefix KV reuse) is the integration smoke + the offline-for-cents demo. Also lands S2 + the S3 negative-case-law doctrine. Dung adjudicator REJECTED.)
  L-24 conjecture-memory middle tier + collab riders  EX-5/EX-6 / BUILD+3 EXTENSION legs  SSB-journal,J-96..J-98(boundary only),L-22,L-23,N-16,F-5,K-19-SF,L-4/F-12,L-10,D-17/L-7,I-30/I-31(contract)   M-L   W2  depends L-22,L-23 (store ships loop-independent, FORK-1)
       (the status-explicit memory tier BETWEEN the journal and the verified case graph, EXPLORE lane ONLY (locked domain split, type-level lane wall EX-5): unproven conjectures with COMPUTED live/refuted/pending/suspended status (pure journal fold EX-6, orphaned != false, quarantine never delete), provenance, criticisms carrying N-16 evaluator-version edges, declared missing tests. Riders: the L-10 explore-arm NLI novelty column under CGG ON/OFF (leg A), typed unblock conditions on refusals composed from receipts (leg B), the FD-1..FD-3 moves-not-survivors flywheel guard as the I-30/I-31 pre-gate (leg D, contract-first, PROOF_SPINE none acked 07-11).)

================================================================================
==== CANONICAL CONSTANTS + SCHEMA_VERSION BUMP SITES ====
================================================================================
LIVE baseline at authoring (Tier-B derived from ELAI-FEATURE-MAP.md): SCHEMA_VERSION ~= 268 / TOTAL_VARIANTS ~= 577 (ssb.rs / action_type_lock.rs). NEVER hard-code; re-read at edit (W-9) — the J-plan logged six values inside one week. Bump SEQUENTIALLY, one session at a time, 9-tuple golden lockstep.
  L-1  prefer-reuse a grounding-verification row; else +ClaimSetExtracted (Pattern-2; Kind: ClaimTyped/ClaimUntypable). net 0..+1 (decide at edit).
  L-2  +ClaimRiskScored (Pattern-2; Kind: LowRisk/MediumRisk/HighRisk/CriticalRisk) OR generalize G-7 claim_score rows. net 0..+1. [the keystone event]
  L-3  prefer-reuse the D-10 grounding-verification rows (FILE/SOURCE/CosmosCheck). net 0 expected.
  L-4  +SemanticEntropyMeasured (Pattern-2; Kind: SeedsAgree/SeedsDiverge/CrossFamilyDiverge) OR reuse a verifier row. net 0..+1.
  L-5  prefer-reuse F-4 TestIntegrityEvent / MutationBudgetedRun / MetamorphicRelation; else +CounterfactualProbed. net 0..+1.
  L-6  prefer-reuse RetrievalLoopEvent (J-38) / escalation rows. net 0 expected.
  L-7  prefer-reuse RefusalDecision (D-16) / AutonomousGate EvidenceCheck rows; else +HallucinationGateAction. net 0..+1.
  L-8  prefer-reuse staleness/MemoryValidationEvent rows. net 0 expected.
  L-9  prefer-reuse ChatReflect / grounding rows. net 0 (profile).
  L-10 prefer-reuse SAFEDELTA / QDELTA / ConvergenceCurveMeasured (+Kind HallucinationDeltaArm). net 0..+1 (bench).
  L-11 prefer-reuse mini_critic_verdict (C-32) / a maintainability row; else +OutputQualityVerdict (Pattern-2; Kind: SrpBlock/DipWarn/IspWarn/PassClean). net 0..+1. [the quality keystone; the checker is .claude/tools, no SSB on the harness leg per JD-1]
  L-12 prefer-reuse skill_system_event / log_skill_invocation (E-43) + a workflow/skill-imports.log row; else +SkillPromoted (Pattern-2; Kind: PromotedFromPlaybook/DemotedOnEvalRegression). net 0..+1.
  L-13 prefer-reuse the rules-engine rule-fired / lifecycle rows; else +UserRuleRegistered (Pattern-2; Kind: Shadowed/Promoted/RejectedByAdmission). net 0..+1.
  L-14 prefer-reuse RunArtifactEmitted (I-40) + pipeline/plan rows + the L-11 quality verdict. net 0 expected (compose).
  L-22 +FalsifierAuthored (Pattern-2; Kind: PredicateCompiled/ProgramCompiled/RubricDeferred/ForbidsNothing); prefer-reuse the L-2 ClaimRiskScored row for the demarcation tier + I-4 candidate rows for the VS spread (else +VerbalizedDistributionDrawn). net +1..+2.
  L-23 +ExploreRoundCompleted (Pattern-2; Kind: SurvivorRetained/RefutedByFalsifier/OrbitDetected/ProblemTurnedOver); reuse anti_spiral_hit for the Orbiting rotation. net +1.
  L-24 +ConjectureLifecycle (Pattern-2; Kind: Registered/Criticized/Refuted/Suspended/Reinstated/Quarantined/Retrieved) — the Retrieved audit row is non-negotiable (EX-5); FORK-2 fallback = reuse ExploreRoundCompleted Kinds (net 0). Leg B prefer-reuse the D-16/D-17 RefusalDecision row + the UnblockCondition field (net 0). net 0..+1. LIVE baseline at authoring: 581 variants / SCHEMA 272 derived (ssb.rs:1530-1544, 2026-07-11); re-read at edit (W-9).

================================================================================
==== SEQUENCING ====
================================================================================
W1 (foundation): L-1 typed claims, then L-2 the ClaimRisk fuse [KEYSTONE]. Nothing scores without the typed claim object + the fuse.
W2 (predictors): L-3 (P1 round-trip, mostly activation) + L-4 (P3 semantic-entropy, the net-new) + L-5 (the do-test [KEYSTONE]). Parallel-safe (disjoint files); L-4 needs L-2's fuse interface.
       BINDING REVISION (L-15 R2/R4, MEASURED — supersedes the W2 order above): land L-3 (evidence round-trip) + L-5 (do-test) BEFORE L-4 (semantic entropy) — the benchmark proved black-box resampling (L-4) is the CEILING (PopQA AUROC 0.83 fair / 0.66 adversarial) while the evidence-grounded predictors break it (code A5 cut halluc 25%->9% at 92% cov, the only arm that cut H_emit without trading coverage). AND land D-13 cross-family alongside L-4, not after (FINDINGS sec 9: single-model self-consistency made hardest-math L5 WORSE 64%->73%; 4-family agreement cut it to 12.5% @62% cov — D-13 is the only lever for confident-consistent fabrication L-4 is blind to). See L-15_repair_controller_and_evidence_first_reorder.md.
W3 (act on risk): L-6 risk-as-curiosity router, L-7 policy gate + abstention, L-8 prune. Depend on L-2.
W4 (prove it / serve research): L-9 the deep-think math profile (composes L-2..L-5), L-10 the measured delta (the whole-plan done-gate). L-10 is BENCH-PENDING on the live-model arm (RunPod, W-10); the deterministic fixture arm ships in W4.

L-T2 sequencing (independent of L-T1; the two tracks share no files): W1 L-11 the output-quality spine [KEYSTONE] (the SOLID-proxy checker + the 4-seam wiring; everything generated is gated by it). W2 L-12 skill front-door + L-13 user rules/hooks (parallel-safe, disjoint; both ACTIVATION of dark/wired substrate). W3 L-14 one-shot builder (depends L-11 — the generated code passes the quality gate before DONE; composes the pipeline + the RRA receipt). The L-T2 measured proof reuses L-10/I-38/G-6 (the quality `--quality` arm), never a separate runner (GD-1).

================================================================================
==== HARD RULES INHERITED FROM CLAUDE.md + F-0/G-0 ====
================================================================================
blake3 only (never blake2/sha2); no .unwrap()/.expect() on prod paths; scoped tests (-p, never full workspace during impl); anti-slopsquatting (verify any new dep — none planned); the LLM never adjudicates a risk verdict or sets a must-preserve/causal flag (TD-3); every external number tagged [EXTERNAL]/[DESIGN] with its source (ID-5/KD-1); a reasoning trace is never the confidence signal (RR-1).

================================================================================
==== CARRIED WEB LANDMINES ====
================================================================================
- the three WV-2 arXiv IDs (SelfCheckGPT 2303.08896 / Semantic Entropy s41586-024-07421-0 / SAC3 2311.01740) are from the user's notes, NOT WebSearch-confirmed this session — PENDING-VERIFY before they appear in a shipped artifact (house rule, 2 bad cites shipped in B-40/B-42 before this rule).
- arXiv:2606.27326 figures (rho~=0.80, 50-trajectory finetune, action-shuffle<=1.1) are [EXTERNAL/DESIGN-RATIONALE] — the CGG borrows the MECHANISM, never the numbers; the L-10 done-gate is ELAI-measured.

================================================================================
==== CARRIED DESIGN-DOC CORRECTIONS ====
================================================================================
- DC-1: I-5/I-11/H-50 are BUILT (the map's older PLANNED labels are stale, code-grounded 2026-06-25) — L-8/L-9 compose the BUILT code.
- DC-2: G-7/D-3/D-16 are DARK (default-OFF), not absent — L ACTIVATES them under its own kill-switch.
- DC-3: ELAI has resampling (I-4) + >=2-concur (D-3) but NOT semantic-entropy-over-meaning-clusters — L-4 is genuinely net-new.

================================================================================
==== SESSION LOG (populated as L-Ns ship) ====
================================================================================
2026-06-27 L-15 AUTHORED (new plan file, not a ship): docs/thefinalglance/L-plan/L-15_repair_controller_
  and_evidence_first_reorder.md. Trigger = the CGG benchmark RAN (workflow/experiments/cgg_bench/
  CGG-BENCH-FINDINGS.md + the Opus reasoning workflow/reasoning/cgg-zero-hallucination-opus-reasoning-
  2026-06-27.md) — the FIRST measured grounding of the L-plan's core assumption. MEASURED: black-box
  resampling (L-4) tops out AUROC 0.83 fair / 0.66 adversarial (the ceiling, costs ~50% coverage);
  execution grounding (the do-test, A5) cut code hallucination 25%->9% at 92% COVERAGE (near-0 without
  trading coverage). L-15 = the net-new RepairController loop object (catch->ground->re-verify->commit-or
  -abstain via the J-83 receipt gate) + 5 BINDING REVISIONS to L-T1: R1 convergence needs an evidence
  receipt (not absence-of-disagreement); R2 reorder WAVE 2 to land L-3/L-5 BEFORE L-4; R3 deterministic
  repair-retrieval (HippoRAG, never model-nav); R4 L-10 measures H_emit at FIXED 90% coverage (repair vs
  abstain-only); R5 L-9 math = flagship near-0 demo. PLANNED, uncommitted (docs gitignored).

2026-06-30 L-T3 CONTEXT-CORE AUTHORED (new track, sessions L-16..L-19, new plan files, not a ship):
  docs/thefinalglance/L-plan/L-16_immutable_transcript_pure_fold_view_core.md (KEYSTONE) + L-17 freeze-epoch
  controller + L-18 losslessness/time-travel collapse + L-19 ambient page-in budget ladder. Trigger = a design
  comparison vs Context Warp Drive (context-warp-drive-main, source-read 2026-06-30; a shipped zero-LLM
  deterministic message-array folding library, measured ~89-93% provider cache-read). FINDING: ELAI's specified
  context stack already covers ~90% of CWD feature-for-feature (K-12 atoms / H-6 read-back / I-11 BeliefState /
  3E allocator / E-5 memory); CWD's one genuinely net-new idea is the immutable-transcript + pure-fold-view core
  (never mutate raw history; the window is a deterministic projection), which ELAI's mutate-then-recover core
  (compact_session + recover_stubs + check_lossless + edit_checkpoints) does NOT have even fully built. L-T3
  adopts the inversion: HD-2 cache-stability becomes structural, the four mechanisms collapse into one fold().
  ADDS the CC-1..CC-4 delta family (raw immutable / view pure+byte-identical / prefix-moves-only-on-named-cause /
  lossless-by-construction). ~15% net-new (the projection + the freeze-cause state machine), ~85% re-expression
  of shipped/planned pieces as fold() parameters (GD-1). Full reasoning: workflow/reasoning/
  cwd-vs-elai-context-management-2026-06-30.md. PLANNED, uncommitted (docs gitignored). FOLLOW-UP (deferred,
  house-precedent for backfill): the top-of-file title count + the per-track sessions table + a per-track
  _grounding/L-CONTEXT-CORE-GROUNDING-MAP.md are a separate cleanup pass; the GROUNDING ANCHORS block lives
  inline in L-16 meanwhile.

2026-07-03 L-T4 MEASURED-COMPETITOR / EVAL-SUPPLY AUTHORED (new track, sessions L-20..L-21, new plan
  files, not a ship): L-20_commit_mined_eval_supply_competitor_arm.md + L-21_weak_lane_verification_
  intensity.md. Trigger = the codebuff repo steal run (C:\Users\user\Downloads\codebuff-main source-read
  2026-07-03; full verdicts workflow/reasoning/steal-codebuff-2026-07-03.md). META-FINDING: codebuff
  (multi-agent CLI, repo-claimed 61% vs Claude Code 53% on cheap models) is ~80% CONFIRMATION of
  harness>model; ELAI covers every orchestration mechanism equal-or-stronger (I-36 per-model prompts,
  I-4 best-of-N, J-53/E-35 weak-model tool parsing, L-17 already plans the ColdGapTtl prune trigger
  codebuff ships). The two genuine steals are MEASUREMENT machinery: L-20 = commit-mined eval task
  supply (real diff = ground truth, contamination-resistant on private repos; EXTENSION J-56/E-54) +
  competitor-arm paired runner over the ALREADY-WIRED agent-CLI bridges (cli_common.rs:330) with
  deterministic gate legs + I-38 paired floor (LLM judge advisory-only TD-3; GD-3 leaderboard ban binds;
  the GTM believe-the-number unit) + an observe-only bench-lessons leg (H-31 firewalled). L-21 = the
  weak-lane verification-intensity policy (codebuff base2.ts free mode: mandatory model-matched reviewer
  + parallel typecheck/test fan-out for weak models) as a deterministic (profile x role) table
  parameterizing C-32 MiniCritic/StructuralRetry + mid-turn scoped checks + B-41 reviewer marking;
  fail-closed to highest intensity on unknown profiles; can only ADD verification (R8); the small-model
  orchestrator lift for the Qwen-7B lane. REJECTED (ledger riders in the steal file): set_messages
  history mutation (CC-1), agent store (ToxicSkills), handleSteps user code agents (FS-2), ads tier,
  LLM-judge-as-gate reaffirmed. ADDS no new delta family (W/GD/HD/TD/ID/OQ/FS/CC cover it; per R-j).
  net 0..+2 SSB prefer-reuse. PLANNED, uncommitted (docs gitignored). Per-track grounding map deferred
  to the house backfill precedent (L-T3 pattern); the GROUNDING blocks live inline in L-20/L-21.

2026-07-08 L-T5 DEEPREASON-STEAL / EXPLORATORY-REASONING AUTHORED (new track, sessions L-22..L-23, new plan
  files, not a ship): L-22_claimant_authored_falsifiers_demarcation.md + L-23_reason_explore_profile_conjecture_
  criticism.md. Trigger = the PHASE-4 DeepReason steal artifact workflow/reasoning/steal-deepreason-2026-07-08.md
  (github.com/AHepi/DeepReason, spec-eval AS-SPECIFIED). CENTRAL FINDING: DeepReason = the 2nd independent
  confirmation of harness>model (1st = codebuff/L-20), in the open-ended-reasoning domain ELAI's corpus was
  thinnest; ~90% composition, not invention. L-22 = the generation-time half of the CGG (claimant-authored
  falsifiers on the L-1 ClaimSet + a demarcation fuse into L-2, forbids-nothing => High floor HG-5 + the
  no-falsifier-no-Converge GRC gate + the Verbalized-Sampling propose contract on I-4, arXiv:2510.01171
  WebSearch-verified). L-23 = the NEW `/reason --profile explore` surface (rival-explanation survivor map) +
  the S2 gate-block-rate orbit detector (5th anti-spiral StagnationPattern, zero embeddings) + the S3
  negative-case-law doctrine; the LOCAL-7B arm (E-35 grammar mask + J-36 frozen-prefix KV reuse) is the
  integration smoke + the offline demo. ADDED delta families: HG-5 (demarcation), EX-1..EX-4 (exploratory-
  reasoning). PROOF_SPINE: L-22/L-23 feed PS-3; the S2 orbit-detector leg is PROOF_SPINE none (user-acked
  2026-07-08). REJECTED (feature-map ledger): Dung argument-graph adjudicator, untyped-artifact ontology,
  embedding convergence detection, schools/lineage machinery (DEFERRED). PLANNED, uncommitted (docs gitignored).
  Per-track grounding map deferred to the L-T3/L-T4 house backfill precedent; GROUNDING blocks live inline in
  L-22/L-23.

2026-07-11 L-T5 EXTENDED — THE DEEPREASON COLLAB WAVE (session L-24, one new plan file, not a ship):
  L-24_conjecture_memory_middle_tier.md. Trigger = the LIVE collab exchange with the DeepReason author
  (workflow/handoff/deepreason-steal-and-collab.md CURRENT STATUS improvements (a)-(d)); his one real hit
  ("proven-only case minting is clean but sterile") adopted for the EXPLORE lane only — the proven-only
  code-fix path (J-96..J-98) locked UNCHANGED, enforced by a type-level lane wall + a Retrieved audit row +
  a negative contract test (EX-5). The tier: unproven conjectures as first-class retrievable objects with
  COMPUTED status (live/refuted/pending/suspended — a pure journal fold, orphaned != false, no absorbing
  status, quarantine never delete, EX-6), provenance, criticisms carrying the N-16 evaluator-version edge,
  declared missing tests. Riders in the same file: leg A the L-10 explore-arm NLI meaning-cluster novelty
  column under CGG ON/OFF (PS-3 direct); leg B typed unblock conditions on D-17/L-7 refusals composed from
  receipts (a refusal declares what was tried + what evidence would change the answer); leg D the FD-1..FD-3
  flywheel diversity guard (moves-not-survivors, lambda-filtered labels, evaluator-version edge per dataset
  row) as a contract-first pre-gate on the D-026-gated I-30/I-31 — PROOF_SPINE none, user-acked 2026-07-11.
  plan_gate_check.py PASS at authoring (P1-P10). ADDED delta rules: EX-5/EX-6 (existing EX family, per R-j).
  K-19 SF-1..SF-4 declared binding (memory-reuse class). PLANNED, uncommitted (docs gitignored).

================================================================================
==== NEXT STEP ====
================================================================================
WAVE 1, L-1: re-grep the live SCHEMA_VERSION/TOTAL_VARIANTS (ssb.rs + action_type_lock.rs), read _grounding/L-CGG-GROUNDING-MAP.md TS-1/TS-13 + NN-1, then author the typed ClaimSet extractor + its off-agent contract test, wired at the report/claim-emission seam. Then L-2 the ClaimRisk fuse.

L-T2 NEXT STEP (independent track): WAVE 1, L-11 — re-grep C-32 mini_critic.rs + C-31 file_plan.rs + maintainability_check.py + B-29 + the elai.map TYPE_REF/IMPL_TRAIT tags, read _grounding/L-OQF-GROUNDING-MAP.md TS-1..TS-12 + NN-1/NN-2 + OQ-1..OQ-5, then author .claude/tools/solid_proxy_check.py (SRP god-object via B-29 fan-out) + the god-object MiniCritic predicate, wired across the 4 seams, SHADOW-first, frozen in gate-manifest. Then L-12/L-13 (parallel) then L-14.

End of L-0_INDEX.
