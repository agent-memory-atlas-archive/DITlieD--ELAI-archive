# L-plan — Summary Index of all 24 sessions (L-T1 = CAUSAL GROUNDING GATE / model-agnostic hallucination-grounding, L-1..L-10; L-T2 = OUTPUT-QUALITY + FAMILIARITY SURFACE, L-11..L-14; L-15 = receipt-minting repair controller; L-T3 = CONTEXT-CORE / immutable-transcript + pure fold-view, L-16..L-19; L-T4 = MEASURED-COMPETITOR / EVAL-SUPPLY, L-20..L-21; L-T5 = DEEPREASON-STEAL / EXPLORATORY-REASONING, L-22..L-24)
Generated: 2026-06-27 (two waves, main LLM in-band; spec-eval AS-SPECIFIED + paper-to-plan PHASE 5; live anchors carried from ELAI-FEATURE-MAP.md + the K-12 code-grounding pass, re-grep at edit per W-7; corrections in _grounding/L-CGG-GROUNDING-MAP.md (T1) + _grounding/L-OQF-GROUNDING-MAP.md (T2)). L-T5 added 2026-07-08 (the DeepReason steal) + extended 2026-07-11 (L-24, the collab wave). Mirrors H/I/J/K-SUMMARY format.

Each entry preserves the track + lever, grounding strength, nature, the substrate it COMPOSES (by grounding-map TS-id) or the gap it BUILDS (NN-id), the explicit do-NOT-reimplement items (GD-1), the integration point with live anchors, and the observable_effect done-gate (W-1 for runtime legs). Doctrine + HG-1..HG-4 + the canonical constants + sequencing + the K-10 USER-OVERRIDE note: L-0_INDEX.md.

## Premise
L is the first plan that points the trust spine OUTWARD. The prior letters built verification of the AGENT'S OWN code (does the agent's diff run, is it wired, is it a stub); L turns the same machinery into a model-agnostic gate over ANY model's CLAIMS, on ANY backend (B-37). It is ~85% ACTIVATION + UNIFICATION of shipped/planned units (D-3/D-10/D-13/D-16/G-7 are DARK behind flags, not absent) + ~15% net-new (the typed ClaimRisk object L-2, the semantic-entropy predictor L-4, the do-test-on-claims L-5, the risk-as-curiosity router L-6). The thesis is the paper's (arXiv:2606.27326): hallucination is a low-coverage/epistemic problem, PREDICTABLE from label-free runtime signals and PREVENTABLE by spending effort where coverage is weak; three mechanistically-distinct predictors that CONVERGE are trustworthy; the strongest causal signal is INTERVENTIONAL. Delta family ADDED: HG-1 causal-means-interventional, HG-2 convergence-not-single-signal, HG-3 risk-is-black-box, HG-4 abstain-or-verify-never-silently-pass. The CGG catches coverage-gap hallucination, NOT confident-shared-wrong beliefs (GD-5 honest residue, named everywhere; mitigated cross-family in L-4).

==== TRACK L-T1 — CAUSAL GROUNDING GATE ====

### L-1 — typed claim extraction substrate [foundation; HG-3 black-box]
- Track L-T1 | lever GROUNDED | grounding STRONG (the notes' claim-type table; off-agent precedent) | nature BUILD | effort M | WAVE 1
- Build: a deterministic structural extractor turning ANY model output (answer/plan/patch/explanation) into a typed ClaimSet — Vec<Claim{kind: ClaimKind, span, refs}>, ClaimKind in {File,Symbol,Dependency,Api,Behavior,Causal,Reachability} (LS-2's table). Parsed structurally (cited paths/symbols/pkg-names/signatures), NOT by asking a model "what did you claim". The intake every predictor (L-3/L-4/L-5) reads. Off-agent: the model that emitted the output never types its own claims (wiring R8).
- COMPOSES-WITH: the report/claim-emission seam (report_fns role-classification, TS), D-10 grounding-sweep (the existing implicit extraction it makes first-class), G-1 taint (an untrusted-sourced claim is tagged at extraction). Do NOT re-implement grounding-sweep — lift the claim object OUT of it.
- Integration: crates/orchestrator/src/claim/ (new) reading the model-output seam; +ClaimSetExtracted or reuse a grounding-verification row (net 0..+1, decide at edit). frontend_pair: a claims tile on the trust panel (decide at edit, W-4) or justified N/A.
- observable_effect: a model output with a file-claim + an invented-package-claim yields a ClaimSet with both typed; the negative arm — a claim referencing a nonexistent file is typed File + flagged refs-unresolved (never silently dropped). Off-agent: the extractor runs in the orchestrator, not the worker.

### L-2 — unified ClaimRisk score + convergence fuse [KEYSTONE; HG-2]
- Track L-T1 | lever GROUNDED | grounding STRONG (G-7/D-3 shipped DARK) | nature BUILD | effort M-L | WAVE 1 | depends L-1
- Build: the ClaimRisk object — generalize G-7's claim_score from the agent's own output to ANY backend's claim (HG-3 black-box); fuse the predictors P1 (L-3 round-trip), P2 (J-85 oscillation, consumed not rebuilt), P3 (L-4 semantic entropy) under the HG-2 rule: a high/critical verdict needs >=2 mechanistically-distinct predictors to agree (paper §4.2; extends D-3 >=2-concur). Cost-gate: the expensive predictors fire only after a cheap signal (L-1 refs-unresolved, P1 residue) flags medium risk, routed by the J-84 effort classifier. Risk tiers Low/Medium/High/Critical drive L-7.
- COMPOSES-WITH: G-7 confidence producer (generalize claim_score — do NOT re-implement), D-3 VerificationAggregator (the >=2-concur fuse), H-40 decorrelated retention, H-39 abstention-aware (the confident-wrong penalty), D-16 tau (the abstain threshold), J-84 effort classifier (cost-gate), J-85 (P2 input). Restates none.
- Integration: crates/orchestrator/src/claim/risk.rs; +ClaimRiskScored (Pattern-2; Kind LowRisk/MediumRisk/HighRisk/CriticalRisk) OR generalize G-7 rows. net 0..+1 (decide at edit, W-9). frontend_pair: a risk tile on the trust/context panel (decide at edit).
- observable_effect: a claim with P1-residue-high AND P3-divergent scores Critical (two distinct predictors agree); a claim with only ONE predictor firing scores at most Medium (HG-2 — single signal never blocks); recomputable from the journal (GD-2). NEGATIVE: feeding a logprob-only signal on a backend that exposes none still produces a valid score (HG-3 — black-box, logprob optional).

### L-3 — evidence round-trip predictor (P1) [ACTIVATION over arbitrary claims]
- Track L-T1 | lever GROUNDED | grounding STRONG (D-10/H-53 shipped) | nature BUILD-on-shipped | effort M | WAVE 2 | depends L-1
- Build: the P1 predictor — does a claim survive re-grounding against its evidence (the paper's decode>re-encode round-trip, LLM analogue): run each typed claim through the D-10/D-11 grounding methods (FILE/SOURCE/TEST/CosmosCheck) + the H-53 closed-set citation membership + DKG validate-citation (H-32); a claim whose file/symbol/line/source cannot be reconstructed = high residual. Activation of the SHIPPED verification methods over the L-1 ClaimSet (today they run inside grounding-sweep on the agent's own output).
- COMPOSES-WITH: D-10 select_grounding_mode + D-11 checkers, H-53 citation_prebind (closed-set membership > D-16 refusal — ghost citation impossible), H-32 DKG validate-citation, the DKG. Do NOT re-implement any grounding method.
- Integration: crates/orchestrator/src/claim/roundtrip.rs feeding L-2; prefer-reuse the D-10 grounding-verification rows (net 0 expected). frontend_pair: N/A (feeds L-2's tile); justified.
- observable_effect: a claim citing src/auth/session.ts:84 with the symbol present round-trips PASS (low residual); a claim citing a nonexistent symbol round-trips FAIL (high residual) and the [N] is rejected by the closed set. NEGATIVE: an out-of-evidence citation cannot be emitted as supported (H-53 closed set).

### L-4 — semantic-entropy inter-seed predictor (P3) [the one net-new predictor; ID-5]
- Track L-T1 | lever GROUNDED | grounding STRONG (resample I-4 shipped; the meaning-cluster is net-new, DC-3) | nature BUILD | effort M-L | WAVE 2 | depends L-1,L-2
- Build: the P3 predictor — resample N alternative answers/explanations (I-4 best-of-N over SessionDag), CLUSTER BY MEANING using the F-12 DeterministicNliJudge (entailment-equivalence, TD-3 — NOT an embedding cosine, NOT an LLM-judge), emit semantic entropy over the meaning clusters; high entropy = high risk (SelfCheckGPT / semantic-entropy port, LS WV-2). Cross-family hardening: route a subset through the D-13 cross-family adversary so within-family shared bias (which same-model resampling misses, the GD-5 limit) is caught (SAC3). The done-gate is ELAI-MEASURED on an ELAI gold set, NEVER a paper AUROC (ID-5).
- COMPOSES-WITH: I-4 run_best_of_n (the resample substrate), D-3 (the agreement count), D-13 adversary (cross-family), D-7 cross-domain canary, F-12 DeterministicNliJudge (the meaning clustering — local deterministic, NOT cloud, keeps ELAI_SEED). Restates none.
- Integration: crates/orchestrator/src/claim/semantic_entropy.rs feeding L-2; +SemanticEntropyMeasured (Pattern-2; Kind SeedsAgree/SeedsDiverge/CrossFamilyDiverge) OR reuse a verifier row. net 0..+1. frontend_pair: N/A (feeds L-2); justified.
- observable_effect: N=5 resamples that all assert the same meaning cluster low (SeedsAgree); 5 resamples splitting into 3 meaning clusters score high (SeedsDiverge); a same-model-agreed claim that the cross-family adversary contradicts flags CrossFamilyDiverge. NEGATIVE: paraphrases of ONE true answer cluster together (the semantic-entropy point — phrasing variance is NOT disagreement), never falsely flagged.

### L-5 — counterfactual do-test on model claims [KEYSTONE; CANT-LIE; HG-1]
- Track L-T1 | lever CANT-LIE | grounding STRONG (F-4/C-28/C-29 shipped) | nature EXTENSION | effort M-L | WAVE 2 | depends L-1
- Build: the causal core — when a model emits a Causal/Behavior claim ("A causes/fixes B"), run the INTERVENTION and observe whether B changes (the paper's action-marginalization do-test, §4.1.ii): revert only A / disable the flag / patch only the claimed function / add a trace to the claimed path, then re-run the F-4 cant-lie outcome (blake3 stdout-hash the model cannot forge) or the C-28/I-20 mutation or the C-29 metamorphic relation, and compare. Only an interventional/executable PASS labels the claim "causal" (HG-1); static/temporal/import-edge evidence is scored at its weaker tier and never called causal. The strongest, un-forgeable signal.
- COMPOSES-WITH: F-4 AntiFabricationGate (task_runner.rs:9565, the un-forgeable outcome), C-28/I-20 mutation (perturb-and-observe), C-29 IntentPair+MetamorphicRelation, CSQ (phase-2D), J-16 blast-radius (the cone the perturbation touches), G-1 taint. Do NOT re-implement F-4 or the mutation engine — drive them on the claim.
- Integration: crates/orchestrator/src/claim/counterfactual.rs feeding L-2; prefer-reuse F-4 TestIntegrityEvent / MutationBudgetedRun / MetamorphicRelation; else +CounterfactualProbed. net 0..+1. frontend_pair: N/A (feeds L-2); justified.
- observable_effect: a "this patch fixes the timeout" claim runs the failing repro before/after the patch-only intervention and labels causal ONLY if the outcome flips (F-4 hash-bound); a "A causes bug B" claim where reverting A does NOT change B is labelled NOT-causal (HG-1). NEGATIVE: a claim supported only by "A imports B" is scored static-tier, never causal (the LS-2 fake-precision guard).

### L-6 — risk-as-curiosity targeted-effort router [the paper's mitigation; HG-4]
- Track L-T1 | lever GROUNDED | grounding STRONG (J-38/I-7 substrate) | nature BUILD | effort M | WAVE 3 | depends L-2
- Build: the mitigation half (paper §4.3) — the ClaimRisk score becomes a curiosity reward: a High/Critical claim AUTO-FIRES targeted effort at exactly the weak claim (not blanket) — mid-task agentic retrieval (J-38), the graduated read-ladder escalate-on-miss (J-103), a targeted test/trace, or local>cloud escalation (I-7) — then RE-SCORES via L-2. "Preventable, not just detectable." A proven-grounded resolution is eligible to mint a Verified Case Graph node (J-96) = the coverage-gap closure, no weights touched.
- COMPOSES-WITH: J-38 retrieval loop, J-103 read-ladder, I-7 escalation cascade, B-29 impact API (where to look), J-84 effort classifier (budget the targeted effort), J-96 case-mint (the compounding closure). Restates none.
- Integration: crates/orchestrator/src/claim/curiosity_router.rs; prefer-reuse RetrievalLoopEvent (J-38) / escalation rows. net 0 expected. frontend_pair: the retrieval/escalation surface (decide at edit).
- observable_effect: a High-risk file-claim with unresolved refs auto-fires a targeted J-38 retrieval at that symbol and re-scores Lower after the evidence lands; bounded by the J-84 budget (no unbounded loop). NEGATIVE: a Low-risk claim fires ZERO targeted effort (the cost guard — curiosity is targeted, never blanket).

### L-7 — hallucination policy gate + abstention integration [CANT-WRECK; HG-4]
- Track L-T1 | lever CANT-WRECK | grounding STRONG (D-16/AutonomousGate shipped) | nature BUILD-compose | effort M | WAVE 3 | depends L-2,L-6
- Build: the policy gate — map ClaimRisk to action: Low > allow; Medium > attach caveat / require citation; High > force the L-6 targeted effort before the claim can act; Critical > BLOCK the edit/action or ABSTAIN (HG-4 — never a silent allow; missing evidence = high risk). Wire into the AutonomousGate EvidenceCheck per transition + conformal refusal (D-16) + Aletheia voluntary abort (I-8, credited by abstention score) + honest-refusal SLA (D-17). The verdict is an SSB row, so the whole gate decision is RRA-reproducible (G-2). Observe-only first (GD-4): bound the FP/over-block rate on >=3 real runs before BLOCK-by-default.
- COMPOSES-WITH: D-16 RefusalDecision, I-8 acknowledge_infeasible, D-17 honest-refusal SLA, B-22 AutonomousGate EvidenceCheck, G-2 RRA, L-6 (the High-risk effort it triggers). Restates none.
- Integration: crates/orchestrator/src/claim/policy_gate.rs wired into the AutonomousGate transition seam; prefer-reuse RefusalDecision / EvidenceCheck rows; else +HallucinationGateAction. net 0..+1. default-ON behind ELAI_HALLUCINATION_GATE_DISABLE (W-2). frontend_pair: the refusal/mode panels (D-27, decide at edit).
- observable_effect: a Critical claim (invented package on a destructive path) is BLOCKED or abstained, the decision journaled + RRA-reproducible; a Medium claim ships with a caveat. NEGATIVE: an unverifiable High claim is NEVER silently allowed (HG-4); the LLM never overrides the tier (TD-3); under ELAI_HALLUCINATION_GATE_DISABLE the gate observes-and-records but does not block.

### L-8 — working-memory hygiene / prune spurious connections [CANT-WRECK]
- Track L-T1 | lever CANT-WRECK | grounding STRONG (F-5/I-5 shipped; J-21 planned) | nature BUILD-compose | effort M | WAVE 3 | depends L-1
- Build: the elai-lang prune — remove false repo assumptions from the agent's LIVE claim/belief graph before they poison later steps (the notes' "bad assumptions poison long-running agents"): when a claim is later contradicted by evidence (a symbol grep shows the assumed owner has no such symbols; a package is absent from the lockfile; a "this is unused" claim is refuted by a call-graph hit), QUARANTINE it from the working belief graph via the staleness cascade (F-5) + memory semantic-validation (J-21, re-derive against commit+AST) + the pristine-context injector (I-5, exclude the failed-path traceback). Threads over the J-83 BeliefState as the carrier.
- COMPOSES-WITH: F-5 staleness quarantine cascade, J-21 memory semantic-validation, I-5 pristine_path_filter (BUILT, DC-1), J-83/I-11 BeliefState (the carrier). Restates none.
- Integration: crates/orchestrator/src/claim/prune.rs threading the belief graph; prefer-reuse staleness / MemoryValidationEvent rows. net 0 expected. frontend_pair: N/A (runtime hygiene); justified.
- observable_effect: a claim "src/cache.ts owns auth logic" later contradicted by a grep showing zero auth symbols is quarantined from the belief graph and not re-injected downstream; recomputable from the journal. NEGATIVE: a still-valid claim is NEVER pruned (validity re-checked against current commit+AST, J-21); pruning never DELETES the record (RV — quarantine, not erase).

### L-9 — research/math deep-think grounding profile [EXTENSION; token-perfect via the checker]
- Track L-T1 | lever GROUNDED | grounding STRONG (B-39/D-11/B-42 shipped) | nature EXTENSION | effort S-M | WAVE 4 | depends L-2,L-3,L-4
- Build: a PROFILE (not a new engine) for researchers/math needing token-perfect accuracy — wire /reason (B-39 Chat Reflect, N=4 reasoners) to run in GRC Mode A (the D-11 Constructive-Checker Registry sympy/z3/compile + the E-53 Formal Verification Lane) over Cosmos-ingested papers (B-42 + the J-33 MinerU lane + paper-ingestor), with the CGG P1 (L-3) round-trip checking each claim against the ingested paper bytes (H-53 closed set) and P3 (L-4) semantic-entropy over the N=4 reasoners (disagreement on a derivation = flag). Token-perfect = the symbolic checker adjudicates the math, the model proposes (TD-3); a derivation that sympy/z3 cannot confirm is abstained, not asserted (HG-4).
- COMPOSES-WITH: B-39 /reason + ReasoningDrawer, D-11 checkers + E-53 formal lane, B-42 Cosmos + J-33 MinerU + paper-ingestor (the ingested-paper store), L-2..L-5 (the CGG over the reasoners). Restates none — this is a configuration + a thin profile selector.
- Integration: a `--profile research` arm on /reason + a ReasoningProfile selector routing Mode A + the Cosmos-paper grounding source + the CGG predictors; prefer-reuse ChatReflect / grounding rows. net 0. frontend_pair: the ReasoningDrawer (a profile badge; decide at edit).
- observable_effect: `/reason --profile research "<a claim about an ingested paper + a derivation>"` grounds each factual claim against the paper bytes (round-trip) and runs the derivation through sympy/z3; a wrong arithmetic step is caught by the checker (not the model), a divergent N=4 derivation is flagged. NEGATIVE: a derivation the checker cannot verify is abstained ("unverified by constructive check"), never asserted as proven (HG-4/TD-3).

### L-10 — CGG hallucination-delta bench [the whole-plan done-gate; BENCH-PENDING live arm]
- Track L-T1 | lever GROUNDED | grounding STRONG (G-6/I-38/I-27 runner shape) | nature BUILD-instrument | effort M | WAVE 4 | BENCH-PENDING (live-model arm RunPod-gated, W-10)
- Build: `elai bench hallucination-delta` — the gate-ON-vs-OFF ablation (the SAFEDELTA framing): on a held-out claim corpus, measure unsupported-claim rate, contradicted-claim rate, and reviewer-reaching rate with the CGG ON vs OFF, under ELAI_SEED, recomputable (GD-2), receipts on the y-axis (GD-3 — never pass@k). The deterministic FIXTURE tier ships now; the live-model arm (real claims from a real backend) is RunPod-gated (W-10). The number is ELAI-MEASURED, NEVER a paper AUROC (ID-5/KD-1); "N fewer unsupported claims per 100 outputs", never a leaderboard rank. This is the L-plan's whole-plan done-gate — the CGG is only "done" when this delta is positive and recomputable.
- COMPOSES-WITH: G-6 SAFEDELTA runner shape (do NOT fork), I-38 QDELTA (the quality sibling), I-27 ConvergenceCurveMeasured + bench_convergence (the runner shape), L-1..L-7 (the gate under measurement), the K-18/K-10 ELAI-measured-number doctrine (KD-1/ID-5). Restates none.
- Integration: benchmarks/hallucination_delta_runner.rs + a `hallucination-delta` bench sub (args_bench.rs:75, mirror the SAFEDELTA runner); prefer-reuse SAFEDELTA/QDELTA/ConvergenceCurveMeasured (+Kind HallucinationDeltaArm). net 0..+1. frontend_pair: a hallucination-delta tile on the audit/cost surface (decide at edit, W-4).
- observable_effect: one artifact per arm with per-arm unsupported-claim rate, recomputable from the journal; the deterministic fixture arm ships now. NEGATIVE: a `--gate-off-but-claim-savings` broken arm shows the delta COLLAPSE (the gate is the cause); the live $ /reviewer-reaching number refuses to emit until the live-model arm runs (no fabricated delta, ID-5).

==== TRACK L-T2 — OUTPUT-QUALITY + FAMILIARITY SURFACE ====

### L-11 — output-quality enforcement system [KEYSTONE; OQ-1..OQ-5]
- Track L-T2 | lever AUDITABLE/CANT-WRECK | grounding STRONG (C-31/C-32/maintainability_check shipped; J-65/J-23 planned) | nature BUILD+COMPOSE | effort L | WAVE 1
- Build: make EVERY model's generated code pass a SOLID/maintainability bar via a 4-SEAM layered gate, not one new gate. The ONE net-new piece = .claude/tools/solid_proxy_check.py (SRP god-object = fan-out via B-29 AND size = the deterministic BLOCK; DIP cross-layer-concrete-vs-trait via elai.map = WARN; ISP fat-trait = WARN; OCP/LSP NOT deterministically checkable -> cross-family ADVISORY J-89, never a block) + a god-object MiniCritic METRIC predicate (C-32). SEAM 1 plan-time (C-31 responsibility budget) / SEAM 2 gen-time self-correct (C-32 MiniCritic + StructuralRetry — the any-model lever, OQ-5, paired with J-55) / SEAM 3 write-time deterministic floor (MR-1/MR-2 via VibeGate F-1) / SEAM 4 audit-time authority (the frozen checker + J-65 + the J-23 generated-multilang lane). The VERDICT is auditor-owned + frozen R8 (OQ-1, never executor-self). Every quality rule enters rules-engine SHADOW and promotes to BLOCK only on fpr<5%/tpr>80% over >=3 runs (OQ-3).
- COMPOSES-WITH: C-31 file_plan, C-32 mini_critic/structural_retry, maintainability_check.py MR-1..MR-4, J-65 power-of-ten (sibling), J-23 review-catalog (generated lane), B-29 fan-out, elai.map TYPE_REF/IMPL_TRAIT, rules-engine 3F lifecycle, VibeGate F-1, J-55 modality router, I-38 QDELTA (the measured number). Restates none.
- Integration: .claude/tools/solid_proxy_check.py (frozen, gate-manifest R8) + the MiniCritic predicate + the C-31/VibeGate/auditor seams; prefer-reuse mini_critic_verdict / a maintainability row OR +OutputQualityVerdict. net 0..+1. frontend_pair: GatesPanel/vibesec quality tile (decide at edit) or N/A.
- observable_effect: a changeset with a 1200-line god-module (high fan-out + 25 methods) is flagged C-MAINTAINABILITY-SRP (BLOCK in ACTIVE / WARN in SHADOW); a clean module passes; a weak-model generated god-object retries in-loop via StructuralRetry (OQ-5). NEGATIVE: an OCP/LSP case is NOT deterministically blocked (advisory only, OQ-4); the executor cannot self-grade (OQ-1); recomputable under ELAI_SEED (GD-2).

### L-12 — skill front-door + self-improving skills [FS-1/FS-2/FS-4]
- Track L-T2 | lever AUDITABLE | grounding STRONG (E-43 DARK/F-6 WIRED/I-12 planned) | nature ACTIVATION+COMPOSE | effort M | WAVE 2
- Build: light the dark skill door. Give E-43 log_skill_invocation a production caller + `elai skill add/improve` gated by skillspector whole-dir scan + J-102 admission + VibeGate (FS-2 — a skill is the #1 injection surface); accept CC/agentskills.io v1 SKILL.md verbatim (FS-4); BM25 top-1 skill retrieval into the context_budget Rules slot trust-tier-gated (F-6 S1-S4); wire the I-12 episodic->skill->rule curator (promote a playbook on >=k SSB-verified outcomes, demote on eval regression — the verdict is the outcome count, never an LLM call, TD-3). "Ask ELAI to improve a skill" = `elai skill improve` running the curator on demand (FS-1, DATA edit).
- COMPOSES-WITH: E-43 (skill_system_event, agentskills_validator, skills-mgmt, log_skill_invocation), F-6 SkillTrustTier/skill_permits_autonomous, I-12 curator, J-102 admission, skillspector/skill-import-policy, context_budget Rules slot. Do NOT re-author E-43/F-6/I-12.
- Integration: the skill-dispatch seam (log_skill_invocation caller) + `elai skill` CLI/SkillsPanel; prefer-reuse skill_system_event + a skill-imports.log row; +0..1 SkillPromoted. net 0..+1. frontend_pair: SkillsPanel.svelte actions.
- observable_effect: a benign SKILL.md is admitted + invoked + recorded; a tests/-dir injection-payload skill is BLOCKED by skillspector/J-102 (FS-2); a playbook with >=k verified outcomes is promoted to a SKILL.md (I-12), the verdict recomputable from the outcome count (TD-3/GD-2). NEGATIVE: an LLM "this skill is good" call never promotes (TD-3).

### L-13 — user-authorable rules & hooks surface [FS-1/FS-2; the CC-hook familiarity done safely]
- Track L-T2 | lever CANT-WRECK | grounding STRONG (rules-3F + HOOK SYSTEM WIRED; J-52 planned) | nature ACTIVATION+COMPOSE | effort M | WAVE 2
- Build: expose the WIRED rules-engine (rules-as-DATA, predicates AST/GREP/SCHEMA/METRIC/COMPOSITE, lifecycle, mutable=0 immune tier) + the HOOK SYSTEM (PreToolUse/PostToolUse/Stop/PreCompact) as user-authorable. `elai rule add/list/test/promote/retire` writes a rules-engine DATA row (lifecycle=SHADOW) + registers it against a hook axis via the J-52 path-keyed injection. The CC "drop a hook" familiarity, but a user rule is a typed predicate+verdict, NEVER executable code (FS-2); it passes J-102 admission + a content scan; it cannot WEAKEN a mutable=0 immune check (R8); it cannot block-by-default until its FP rate is bounded (OQ-3/GD-4). ELAI's version is SAFER than CC's (IPC-level, agent-invisible, lifecycle-governed, can only ADD checks).
- COMPOSES-WITH: rules-engine 3F (write rows), HOOK SYSTEM T1/T2/T3 dispatch, J-52 path-keyed injection, J-27 hook shape, J-102 admission, VibeGate F-1, RulesDrawer/ToolsHooksDrawer. Do NOT re-author the engine/hook system/J-52.
- Integration: `elai rule` CLI arms + the user-rule->hook-axis registration; prefer-reuse rule-fired/lifecycle rows; +0..1 UserRuleRegistered. net 0..+1. frontend_pair: RulesDrawer/ToolsHooksDrawer authoring.
- observable_effect: a user GREP PreToolUse rule fires in SHADOW (records would-WARN, does not block), recomputable from the journal; a rule weakening a mutable=0 check is REJECTED at add (R8); promotion to ACTIVE is refused until the FP bound clears (OQ-3). NEGATIVE: a rule containing executable content is rejected (FS-2 — DATA only).

### L-14 — one-shot verified builder front-door [FS-3/FS-4; the lazy-user surface with the receipt]
- Track L-T2 | lever AUDITABLE | grounding STRONG (pipeline-factory/E-12/I-40 WIRED; J-63/J-2 planned) | nature ACTIVATION+COMPOSE | effort M | WAVE 3 | depends L-11
- Build: `elai build "<idea>"` (+ a dashboard one-field flow) drives PIPELINE FACTORY L1->L2 on production defaults -> E-12 fullstack codegen -> the L-11 output-quality gate -> I-40 `elai wrap` so the output ALWAYS ships an RRA receipt (FS-3 — the lazy product IS the trust product with a simple front end); offer `elai artifact run` (J-63) to run it under Tier-C; reachable as a J-2 MCP verb (FS-4 — ELAI-as-a-server for a CC/Codex user). The value claim ("$20=$200"/cheaper) stays gated on a measured SAFEDELTA/QDELTA number (KD-1/ID-5), never asserted in the UI.
- COMPOSES-WITH: PIPELINE FACTORY L1/L2, E-12 + templates/web, J-63 artifact-run, I-40 wrap/RRA, J-2 MCP control plane, L-11 quality gate, AutonomousGate. Do NOT re-author any.
- Integration: the `elai build` CLI verb + the receipt wrapper + the L-11 call + the J-2 verb; prefer-reuse RunArtifactEmitted + pipeline rows + the L-11 verdict. net 0 expected. frontend_pair: ProjectSelector/Intro one-field flow.
- observable_effect: `elai build "a TODO app with a REST API + 3 routes"` (ELAI_SEED) produces an app dir that passes L-11 + an RRA receipt that `elai rra verify` validates (FS-3); the deterministic fixture build ships now, the live cloud/local build arm is RunPod/J-63-gated (W-10). NEGATIVE: a generated app that fails L-11 is reported with findings, never silently DONE; no value-claim string ships without a measured number (KD-1).

## Track-L-T2 cross-cutting (what makes it different)
L-T2 is two systems on one engine. OUTPUT QUALITY (L-11): ELAI has NO by-name SOLID enforcement but already ships the loop (C-32 gen-time, C-31 plan-time, maintainability_check.py auditor) — so quality is a 4-seam layered gate with ONE net-new checker, auditor-owned + frozen (OQ-1), observe-first (OQ-3), with only the mechanizable proxies deterministic (OQ-4) and the gen-time self-correction as the any-model lever (OQ-5). FAMILIARITY SURFACE (L-12/13/14): the hooks/skills/MCP/one-shot surface a CC/Codex user expects EXISTS and is in places stronger (IPC-level hooks, rules-as-DATA), but the front door is DARK — so L-T2 is ACTIVATION not invention, on the FS doctrine: user-extends-via-DATA (FS-1), every-surface-passes-the-same-gates (FS-2), the lazy path ships the RRA receipt (FS-3), CC/agentskills/MCP format-compat (FS-4). The two audiences (advanced/trust + lazy/one-shot) are the SAME engine at two surfaces; the wedge is the receipt, never "cheap" (KD-1/ID-5). L-T2 shares NO files with L-T1; the measured proof reuses L-10/I-38/G-6 (GD-1).

## Track-L-T1 cross-cutting (what makes it different)
Every prior letter verified the AGENT'S code; L verifies ANY MODEL'S CLAIMS. The architecture is the paper's three-converging-predictors + interventional-do-test, but the implementation is ~85% ELAI's own DARK trust spine (D-3/D-10/D-13/D-16/G-7) activated and unified behind one ClaimRisk object, with four genuine builds (L-1 typed claims, L-2 the fuse, L-4 semantic entropy, L-5 do-test-on-claims). Model-agnostic by construction (HG-3): it rides B-37, so it works on whatever model the user picks. The honest limit (GD-5, named in L-4): it catches coverage-gap hallucination, not confident-shared-wrong beliefs — mitigated, never eliminated, by the cross-family adversary. The whole plan ships observe-only first (GD-4) and is "done" only when L-10's measured delta is positive and recomputable (ID-5/KD-1).

==== TRACK L-T3 — CONTEXT-CORE (immutable transcript + pure fold-view) ====

Premise: a design comparison vs Context Warp Drive (a shipped, zero-LLM, deterministic message-array folding
library, measured ~89-93% provider cache-read) found ELAI's specified context stack already covers ~90% of CWD
feature-for-feature (verbatim-atom conservation = K-12; ambient page-in = H-6; rebirth seed = I-11 BeliefState;
budget table = 3E; episodic recall = E-5). CWD has ONE architectural idea ELAI's spec lacks even fully built:
it never mutates raw history — the transcript is permanent ground truth and the provider array is a PURE
deterministic projection of it. ELAI's running model is the opposite (compact_session rewrites the prefix;
recoverability is bolted on by SSB stubs + blake3). L-T3 adopts the inversion as ELAI's context-core invariant,
which makes HD-2 cache-stability STRUCTURAL and collapses four mechanisms (mutate / recover_stubs /
check_lossless / edit_checkpoints) into parameterizations of one projection. Delta family ADDED: CC-1 raw is
immutable, CC-2 view is pure (byte-identical for identical inputs), CC-3 prefix moves only on a named+journaled
cause, CC-4 lossless by construction. Reference impl (external prior art): context-warp-drive-main. ~15%
net-new (the projection core + the freeze-cause state machine); ~85% re-expression of shipped/planned ELAI
pieces as projection parameters (GD-1 compose-not-restate).

### L-16 — immutable transcript + pure fold-view context core [KEYSTONE; CC-1..CC-4]
- Track L-T3 | lever STRUCTURAL/CHEAPER | grounding STRONG (CWD source-read + ELAI live anchors) | nature BUILD [KEYSTONE] + binding REVISION to compact_session | effort L | WAVE 1
- Build: promote the append-only transcript (SessionDag + H-49 CAS) to single source of truth; introduce
  ContextProjection fold(log, budget, policy, freeze) -> view, a PURE function that never mutates the log. The
  E-25 ladder tiers / H-6 masking / 3E allocator / H-50 policy become STRATEGIES + PARAMETERS inside fold.
  Replace conversation.rs::compact_session (the prefix-rewriting mutator) with per-turn projection; recover_stubs
  becomes recompute-with-larger-budget. HD-2 becomes a type property, not a doctrine.
- COMPOSES-WITH: 3E context_budget.rs (budget param), H-50 context_policy.rs (policy param), H-49 session_dag.rs+CAS
  (the log), K-12 compact.rs:545 (view-level losslessness), I-11 belief_state.rs (a handoff projection variant),
  E-25 compaction_ladder.rs (a budget strategy). Restates none.
- Integration: crates/orchestrator/src/context_projection.rs (new); rewire conversation.rs to hold the log + call
  fold(); re-point the live recover_stubs call-site. prefer-reuse ContextReduction; optional +1 ContextProjected
  (decide at edit, re-grep SCHEMA_VERSION). Default-ON behind ELAI_CONTEXT_PROJECTION_DISABLE (mutate path kept
  one release as rollback).
- observable_effect: a long multi-turn run shows a byte-identical view prefix across non-epoch turns (CC-2, turn-2
  cache_read covers it); a span skeletonized in turn N is recomputed verbatim in turn N+k with no SSB read (CC-1+CC-4);
  the raw log byte-length is monotonic non-decreasing (CC-1). NEGATIVE: an injected log mutation is rejected; a
  fold() reading wall-clock fails the byte-identity assert (CC-2).

### L-17 — freeze-epoch controller: prefix-invariance as an explicit recompute-cause state machine [CC-2/CC-3]
- Track L-T3 | lever STRUCTURAL | grounding STRONG (CWD foldFreeze.ts + I-10 shipped) | nature BUILD | effort M | WAVE 2 | depends L-16
- Build: FreezeState (frozen view + boundary fingerprint + counters) + a pure evaluate_freeze -> Reuse |
  Recompute(cause) over an ENUMERATED cause set (FirstCall/ColdGapTtl/TailOverflow/BoundaryMismatch/HistoryRewound/
  PolicyChanged/PressureCeiling); append-tail-without-recompute optimization; every recompute journals its cause
  (CC-3 — an unexplained prefix change is a hard fail).
- COMPOSES-WITH: I-10 cache_economics.rs (this IS its concrete prefix-invariance + cache-hit hard guard), H-50
  (TTL/late-vs-early), L-16 fold. Restates none.
- Integration: context_projection.rs (FreezeState + evaluate_freeze); prefer-reuse I-10 CacheEconomicsEvent +
  a cause variant. frontend_pair: the gauge shows the last epoch cause; or N/A.
- observable_effect: a no-boundary-change turn returns Reuse with an unchanged prefix hash + covering cache_read;
  a TTL-crossed turn returns Recompute(ColdGapTtl) journaled; a tail-only growth re-folds only the tail (prefix hash
  unchanged). NEGATIVE: a prefix change with no cause is a hard fail; a miss is never reported as a hit.

### L-18 — projection corollaries: losslessness + time-travel collapse [CC-1/CC-4]
- Track L-T3 | lever STRUCTURAL/CANT-WRECK | grounding STRONG (K-12 + H-49 spec) | nature BUILD-compose | effort M | WAVE 2 | depends L-16
- Build: re-point check_lossless_compaction at the VIEW (assert K-12 critical atoms present, exact-match) and
  FAIL-CLOSE it safely (nothing is dropped from the log, so a fail means widen the budget, not data loss —
  satisfies K-12 R1 structurally); undo/redo/branch become recompute-the-view-at-a-log-index (H-49 undo-tree HEAD
  walks indices, not stored diffs); recover_stubs collapses to project-at-span-with-larger-budget.
- COMPOSES-WITH: K-12 (atom schema reused, view-level), H-49 (undo-tree/branch grain reused), recover_stubs
  (legacy-only). Restates none.
- Integration: context_projection.rs (project-at-index + view-level atom assert) + compact.rs (re-point + FAIL-CLOSE);
  prefer-reuse K-12 HandoffFaithfulnessGated + H-49 EditHistoryEvent; net 0. Auditor-owned + frozen.
- observable_effect: every must_preserve atom is present verbatim in the view at every later turn (FAIL-CLOSED green);
  an undo recomputes the exact earlier view byte-identical; a branch forks without copying transcript bytes (CAS).
  NEGATIVE: a too-tight budget that would drop a must_preserve atom FAILS and widens, never ships a missing atom.

### L-19 — ambient page-in budget ladder (concrete H-6 read-back policy) [CC-2]
- Track L-T3 | lever STRUCTURAL/CHEAPER | grounding STRONG (CWD foldRecall.ts + H-6/I-5) | nature BUILD | effort M | WAVE 3 | depends L-16, L-17
- Build: a pressure ladder scaling page-in budget by the L-17 window band (healthy full / warning halved /
  critical quartered / overflow hints-only); recall signals tier-0 tool re-touch / tier-1 claim-on-folded-path /
  tier-2 critical-atom re-surface; matched content injected as APPEND-ONLY cards riding the L-17 tail (CC-2, never
  the prefix), residency TTL in passes, cyclic re-fold at the next epoch; failed-path recalls use the I-5
  contrastive constraint shape, not a raw paste.
- COMPOSES-WITH: H-6 read-back (this is its concrete policy), I-5 (contrastive constraint), L-17 (the tail + band),
  L-16 (recall is a tail strategy). Restates none.
- Integration: context_projection.rs (recall tail strategy + pressure-band table) + H-6; prefer-reuse
  ContextReduction; frontend_pair: gauge "recalled N (TTL k)"; or N/A.
- observable_effect: re-touching a folded file pages it back as a tail card with the frozen prefix hash unchanged
  (CC-2); under critical pressure the card budget is quartered; after the TTL it re-folds away (cyclic). NEGATIVE:
  a recall never rewrites the prefix; a low-relevance span is not paged in.

## Track-L-T3 cross-cutting (what makes it different)
L-T1/L-T2 verify claims and shape output; L-T3 changes the SUBSTRATE everything else runs on. It is the one
track sourced from an external reference implementation rather than a paper: Context Warp Drive proved (measured)
that a deterministic, never-mutate, pure-projection context core keeps the provider cache hot, and ELAI's spec
had independently arrived at every neighbouring piece (the cache breakpoints, the verbatim-atom gate, the
selective read-back, the typed handoff) — but kept a mutate-then-recover core. L-T3 swaps that core for the
projection, which is why ~85% of it is re-expressing shipped/planned pieces as fold() parameters and only ~15%
is net-new. It shares the log + CAS with H-49, the budget with 3E, the policy with H-50, the atom gate with
K-12; it restates none of them (GD-1). The done-gate is ELAI-measured byte-identity + cache_read on a real run,
never CWD's number (ID-5).

==== TRACK L-T4 — MEASURED-COMPETITOR / EVAL-SUPPLY (the codebuff steal) ====

Premise: the codebuff repo steal run (2026-07-03, workflow/reasoning/steal-codebuff-2026-07-03.md) found the
multi-agent-CLI competitor ~80% CONFIRMATION of ELAI's harness>model thesis, with ELAI equal-or-stronger on
every orchestration mechanism (per-model prompts = I-36; best-of-N = I-4; weak-model tool parsing = J-53/E-35;
their deterministic context-pruner's cache-expiry trigger = L-17's ColdGapTtl, already planned). The two
genuine steals are its MEASUREMENT machinery, not its agent machinery. ADDS no new delta family.

### L-20 — commit-mined eval task supply + competitor-arm paired harness [the GTM believe-the-number unit]
- Track L-T4 | lever GROUNDED/AUDITABLE | grounding STRONG (codebuff source read; ELAI anchors PLAN-asserted, re-grep W-7) | nature EXTENSION(J-56/E-54) + BUILD-instrument | effort M-L | WAVE 1
- Steal/build: THREE legs. (A) `elai eval mine-commits` — deterministic pre-filter over a repo's real commit history (merge/deps/docs/formatting OUT; failure_taxonomy typing) + optional ADVISORY hard-commit screen (TD-3), emitting J-56 corpus rows {task_prompt, base_sha workspace, blake3 reference_diff}; contamination-resistant by construction on private repos; E-54 13-gram filter on public rows. (B) CompetitorArm kind over the ALREADY-WIRED agent-CLI bridges (cli_common.rs:330 spawn + hand-parsed stream-json — do NOT write a second spawner): ELAI arm + claude_code/codex_cli/opencode arms, same rows, identical journaled envelope, E-49 cost capture; GATE legs deterministic (build/tests/diff-overlap advisory-weighted), LLM judge ADVISORY-only, I-38 >=5-seed paired floor; internal paired delta under a reproducible envelope, NEVER a leaderboard rank (GD-3), public use KD-1-gated. (C) observe-only bench-lessons rows for J-10/I-12/J-96; H-31 firewalled, K-19 SF-1 bound.
- COMPOSES-WITH: J-56 verbs + EvalCorpusEvent, E-54 filter, G-6/I-38/I-27/K-15/J-107 runner family (REUSE, never fork), agent-CLI bridges, E-49, J-80 pool, failure_taxonomy.py. Do NOT re-implement the spawner/runner/corpus.
- Integration: benchmarks/ commit_miner + competitor_arm_runner.rs + `elai eval mine-commits` / `elai bench competitor` verbs; frontend_pair paired-arm tile (decide at edit) or justified N/A.
- observable_effect: mine-commits on a real repo mints rows with base_sha + reference-diff hash (a docs-only commit is deterministically refused); the two-arm run emits per-arm SuiteRun/cost rows, prefer-reuse net 0..+1; NEGATIVE: an absent vendor CLI = named arm FAIL never silent skip; a leaderboard-rank string in the report fails the contract test.

### L-21 — weak-lane verification-intensity policy [the small-model orchestrator lift]
- Track L-T4 | lever CANT-WRECK/GROUNDED | grounding STRONG (codebuff base2.ts read; ELAI anchors PLAN-asserted, re-grep W-7) | nature EXTENSION(J-51/J-55/C-32/H-7/B-41) | effort M | WAVE 2
- Steal/build: codebuff's production trick for cheap models — free-lane prompts make a model-matched reviewer MANDATORY and fan out reviewer+typecheck+tests in parallel mid-turn; strong lanes get review-optional. ELAI routes WHO (H-7), the ACTION surface (J-55), role ELIGIBILITY (J-51), but gen-time verification intensity is one fixed policy (C-32 constants). L-21 = a deterministic (ModelCapabilityProfile tier x role) table {mini_critic_passes, structural_retry_budget, mid-turn scoped compile/test fan-out, reviewer_dispatch mandatory|optional, fanout_cap} — DATA beside I-51's catalog, pure resolver (TD-3), parameterizing the C-32 loop + task_runner mid-turn checks + B-41 reviewer marking. Reviewer leg = advisory-to-executor ONLY (Track-20 B7 decline stands; auditor authority untouched). Fail-closed: unknown profile = HIGHEST intensity; rows can only ADD verification (R8); observe-first (GD-4) then kill-switch ELAI_VERIFICATION_INTENSITY_DISABLE (W-2). Done-number via the L-20/K-15 runner: weak-lane verified-outcome rate + tokens/correct, table ON vs OFF (ID-5).
- COMPOSES-WITH: J-51 profile (the key), J-55 (sibling axis, one profile read), C-32 (the parameterized loop), B-41, E-31 tier matrix, I-51, BSP spawn_parallel. Do NOT fork routing or re-implement the critic.
- Integration: orchestrator verification_intensity.rs + TOML table + the three consumption seams; frontend_pair intensity badge on RuntimePanel (decide at edit).
- observable_effect: the same task journals passes>=2 + scoped-check fan-out + reviewer=mandatory on the Qwen-7B lane vs baseline on a frontier profile, recomputable from profile+table (GD-2); NEGATIVE: unprofiled model journals HIGHEST never baseline; DISABLE=1 is byte-identical to today; a row weakening the weak-lane floor fails table validation at load.

==== TRACK L-T5 — DEEPREASON-STEAL / EXPLORATORY-REASONING (the DeepReason steal) ====

Premise: the DeepReason paper-to-plan run (2026-07-08, workflow/reasoning/steal-deepreason-2026-07-08.md over
github.com/AHepi/DeepReason harness-spec-v1.3 + CAN_LLMS_EXPLORE + MiniReason source) found a Popperian
conjecture-criticism harness that is the SECOND independent confirmation of ELAI's harness>model thesis (the
first was codebuff/L-20), in the open-ended-reasoning domain ELAI's corpus was thinnest. ~90% composition of
shipped/planned units. The steals are the GENERATION-time attack surface the L-T1..L-T4 verify-time machinery
lacked + a new exploratory-reasoning surface. ADDS HG-5 (demarcation) + EX-1..EX-4 (exploratory-reasoning);
EXTENDED 2026-07-11 with L-24 (the COLLAB wave, from the live exchange with the author — his one real hit on
ELAI adopted: "proven-only case minting is clean but sterile") ADDING EX-5/EX-6. REJECTED: Dung argument-graph
adjudicator, untyped-artifact ontology, embedding convergence, schools/lineage (DEFERRED).

### L-22 — claimant-authored falsifiers + demarcation + Verbalized-Sampling propose contract
- Track L-T5 | lever GROUNDED/CANT-LIE | grounding STRONG (DeepReason source read; ELAI anchors PLAN-asserted, re-grep W-7; arXiv:2510.01171 WebSearch-verified) | nature BUILD-on-shipped + EXTENSION | effort M-L | WAVE 1 | depends L-1, L-2
- Steal/build: invert claim authorship — the GENERATOR emits, WITH every design-claim/hypothesis/diagnosis, the machine-checkable falsifier that would refute it (his skeleton forbidden[]), compiled at registration into a runnable predicate and executed at ZERO LLM cost (his one cost-POSITIVE criticism class at low base error). (a) an optional `falsifier` field on the L-1 ClaimSet, elicited grammar-enforced at GENERATION time (N-13/E-35 mask, weak lane pays no adherence retries — the <50B fix); (b) a DEMARCATION fuse into L-2 — a Causal/Behavior claim that forbids NOTHING has an empty attack surface -> High risk-floor (HG-5); (c) the no-falsifier-no-Converge contract on hypothesis-class GRC-v3 steps; (d) the Verbalized-Sampling propose contract (arXiv:2510.01171) — one distribution call of K typicality-tagged candidates vs I-4's N resamples, tail-weighted ONLY under the orbit flag, typicality advisory (TD-3).
- COMPOSES-WITH: L-1 ClaimSet (ADD a field), L-2 fuse (ADD a signal), D-11 checker registry (compile+run the predicate through its sandbox), E-35/N-13 (the mask), I-4 (the propose leg VS replaces), J-83 (the GRC step carrier), anti-spiral (the orbit flag). Re-implements NONE (GD-1).
- Integration: crates/orchestrator/src/claim/falsifier.rs + verbalized_sampling.rs + the generation seam + the L-2 fuse; frontend_pair a falsifier chip on the ClaimRisk tile (or N/A).
- observable_effect: a causal claim carrying a compiled predicate falsifier scores at its evidenced tier; a causal claim that forbids nothing scores High-floor (HG-5); a GRC hypothesis step with no falsifier is refused Converge; propose_distribution returns K typicality-tagged candidates and tail-spends only under the orbit flag. NEGATIVE: a forbids-nothing claim below High FAILS; a Rubric-only claim marked causal_supported=true FAILS (HG-1); a second free-form model call to reconstruct a falsifier FAILS.

### L-23 — /reason --profile explore: conjecture-criticism map of rival explanations, on the local lane [KEYSTONE — the one NEW surface]
- Track L-T5 | lever GROUNDED/NEW-SURFACE | grounding STRONG (DeepReason measured findings F1-F3; ELAI anchors PLAN-asserted, re-grep W-7) | nature NEW-surface (~90% composition) | effort M-L | WAVE 2 | depends L-22
- Steal/build: a question class ELAI had no lane for — hard open explanatory question -> a MAP of rival explanations with their refutation status + the deciding-evidence gaps. A `/reason --profile explore` round loop composing VS conjecture spread (L-22) + stance rotation (anti-spiral personas) + claimant falsifiers run free (L-22) + the S2 gate-block-rate orbit detector (a 5th StagnationPattern::Orbiting, ZERO embeddings — beat his embedding detector 0 vs 54 blocks) + problem turnover (I-26/I-8) + cross-family survivor judging (D-13 + I-24/N-15 order-swap+verbosity-penalty). Also lands the S3 negative-case-law doctrine (tabu at the gate never in the prompt; complement directives measured a placebo). DIFFERENTIATOR: the LOCAL-7B conjecturer arm (E-35 grammar-mask skeleton valid-by-construction + H-2/I-10 frozen-prefix + J-36 KV reuse) — the offline-for-cents demo DeepReason could not ship + the integration smoke for E-35/N-13 + H-2/I-10 + J-36.
- COMPOSES-WITH: B-39 /reason (a profile, not an engine — L-9 precedent), L-22, anti-spiral, I-8/I-26, D-13/J-45, I-24/N-15, E-35/N-13, H-2/I-10/J-36, K-11/N-7. Re-implements NONE; Dung adjudicator NOT adopted (GD-1, REJECTED §4.3).
- Integration: crates/orchestrator/src/anti_spiral_extended.rs (Orbiting + the rate counter) + reason/explore.rs + the /reason profile registration + the I-5 doctrine; frontend_pair the survivor map via K-11/N-7.
- observable_effect (FIXTURE arm now, live-7B arm sidecar-gated W-10): explore emits a survivor map (survivors + each refuted rival WITH its killing falsifier + the deciding-evidence gap); a re-proposed refuted-equivalent >= floor emits OrbitDetected + a persona rotation naming the attractor; the FIXTURE arm replays byte-identically under ELAI_SEED (RRA). NEGATIVE: a banned/refuted idea rendered back into the conjecture prompt FAILS (S3/EX-1); a single collapsed answer FAILS (EX-3); an embedding-based orbit detector FAILS (EX-2); a Dung/argument-graph symbol present FAILS.

### L-24 — conjecture-memory middle tier (explore lane ONLY) + the three collab riders
- Track L-T5 | lever GROUNDED/MEMORY | grounding STRONG (DeepReason spec §4/§10/§11.5 source-read; the live collab exchange, handoff CURRENT STATUS; ELAI anchors PLAN-asserted, re-grep W-7) | nature BUILD + 3 EXTENSION legs | effort M-L | WAVE 2 | depends L-22, L-23 (store ships loop-independent, FORK-1)
- Steal/build: the memory tier BETWEEN the append-only journal and the verified case graph (J-96..J-98 UNCHANGED — locked domain split): unproven conjectures preserved as first-class retrievable objects with explicit COMPUTED status (live / refuted / pending / suspended — a pure journal fold, never hand-curated; refuting a premise SUSPENDS dependents, orphaned != false; no absorbing status; contradicted nodes QUARANTINED never deleted), provenance, attached criticisms carrying the N-16 evaluator-id+version-hash edge (a later-refuted evaluator collapses what it vouched for), and declared missing tests (exactly what evidence would settle each = unrun L-22 falsifiers). EXPLORE lane ONLY: a type-level lane token + a journaled Retrieved audit row + a negative contract test make an unproven conjecture structurally unretrievable into the code-fix/auditor path (EX-5). Riders: (A) the L-10 explore-arm novelty column — NLI meaning-cluster novelty (L-4/F-12, never cosine-over-hash) under CGG ON/OFF x retrieval ON/OFF; (B) typed unblock conditions on D-17/L-7 refusals — {tried, missing, route} composed deterministically from the GRC receipt trail, never model-prose; an unblock-less refusal is an SLA violation; (D) the FD-1..FD-3 flywheel diversity guard as a contract-first pre-gate on I-30/I-31 (train on MOVES not survivors; lambda-filtered labels per N-16 S6; the evaluator-version edge on every dataset row) — code lands with I-30 when D-026 unlocks, no wire-dark machinery ahead of its consumer.
- COMPOSES-WITH: L-22 skeleton class (the conjecture object), L-23 pack assembly + relapse gate (the production consumer; refuted set feeds the GATE never the prompt), N-16 S4/S6 (ONE evaluator-edge shape, FORK-3), F-5 quarantine shape, K-19 SF-1..SF-4 (binding, memory-reuse class), L-4/F-12, L-10, D-17/L-7, L-16 fold pattern. Re-implements NONE; J-96..J-98 consumed as a BOUNDARY only (zero files edited, ABORT-1).
- Integration: crates/orchestrator/src/reason/conjecture_store.rs (new) + ssb.rs (+ConjectureLifecycle, baseline 581/272 re-read at edit W-9) + the L-23 pack seam + the D-17/L-7 refusal path + the L-10 bench arm; frontend_pair the status dimension on the L-23 survivor-map render.
- observable_effect: two-round explore fixture — round 1 registers A(premise P)+B, a falsifier refutes P between rounds, round 2's pack holds B live-labeled WITH missing tests while A is Suspended and excluded; the lifecycle SSB chain (Registered/Criticized/Refuted/Suspended/Retrieved) lands and the status map recomputes byte-identically under ELAI_SEED (GD-2). NEGATIVE: a code-lane retrieval returning ANY ConjectureNode FAILS (EX-5); a never-criticized node reporting live FAILS; a hand-set status FAILS (EX-6); a refusal without tried[]/missing[] on the D-17 path FAILS.

## Track-L-T5 cross-cutting (what makes it different)
L-T1..L-T4 verify claims, shape output, manage context, and measure competitors; L-T5 is the first track that points the trust spine at OPEN-ENDED REASONING — the domain where a harness matters most and ELAI's corpus was thinnest. It is the only track whose flagship (L-23) is a user-facing MODE rather than an internal gate, and the only one whose done-gate demo is deliberately the LOCAL lane (grammar-mask + frozen-prefix KV reuse), because the whole point of the DeepReason confirmation is that the epistemology lives in the deterministic harness, not the model — so the cheapest model must be able to run it. It shares the claim substrate with L-T1 (L-1/L-2), the anti-spiral engine with C-10, the grammar mask with E-35, and the cross-family judge with D-13; it restates none of them (GD-1). Every DeepReason figure is n=1 single-seed self-diversity = DESIGN-RATIONALE only; the L-10 explore arm measures ELAI's own survivor-map quality/cost (ID-5).

End of L-SUMMARY_INDEX.
