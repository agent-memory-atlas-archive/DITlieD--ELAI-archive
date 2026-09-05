# D-plan: ELAI reasoning-improvement + real-problem-solving roadmap (post-C)
Generated: 2026-05-19
Source: docs/specs/guide/reasoning-improvement-grounded-2026-05-19.md + docs/specs/guide/real-problem-solving-grounded-2026-05-19.md (two grounded design files, 9 oracle research agents, 80+ tier-1 arxiv sources, two passes 2026-05-17 + 2026-05-19).
Premise: C-plan closed the verification gaps of B-plan; D-plan now ships the **substrate** for empirical recursive self-improvement (DGM-class) and **inference-time grounding** (5-mode taxonomy). Each D-N session ships one cohesive module, wires it to the dashboard, and proves it with a real cloud-Sonnet test plus (where applicable) a quantitative benchmark against the paper-reported number.

## Reading order

The D-N files are roughly ordered by dependency. Pre-requisites are listed in each file's "Pre-requisites" line. The companion design (reasoning-improvement) MUST land first as it provides the typed substrate (ReasoningTemplate + PPM + tripwires + archive) that real-problem-solving (mode routing + checker registry + Adversary + UniCR) consumes.

## Status legend
- PLANNED — file written, work not started
- IN PROGRESS — currently being shipped in an active session
- DONE — shipped and verified by cloud-Sonnet smoke test + benchmark threshold
- DEFERRED — blocked on calibration data, hardware, or open research question

## Sessions

### Wave 1 — Reasoning-improvement substrate (Phase 4H.IV–IX from companion)

Build the typed mutation surface, separable verifier, tripwires, and archive that DGM-class self-improvement needs. Each step is **strictly stronger** than DGM on at least one axis (per arxiv:2510.04399 bounded-capacity theorem).

- [D-1](D-1_reasoning_template_schema.md) — ReasoningTemplate typed schema + Blake3 content-hash + archive lifecycle — PLANNED
  Closes: Phase 4H.IV (companion §3.1). Adds ReasoningTemplate { id, domain, pattern_kind, template_body, eligibility_predicate, verifier_required, content_hash, parent_id, archive_status } + ArchiveEntry + ARCHIVE_ENTRY_ADDED/RETIRED SSB types; migrates existing in-prompt templates to typed records.
  Pre-reqs: C-15 (ClockProvider stable) + C-21 (MistakeBook+rules.db stable).
  Cites: SELF-DISCOVER (arxiv:2402.03620, T1), arxiv:2510.04399 (T1) bounded-capacity, DGM (arxiv:2505.22954, T1 — already at workflow/papers/dgm-interface.md).
  Cloud test: `elai /task --mode cloud` exercises ≥3 typed templates; ARCHIVE_ENTRY_ADDED rows present; content_hash chain verifiable.

- [D-2](D-2_ppm_sidecar_wiring.md) — PPM sidecar (Qwen2.5-Math-PRM-7B) — PLANNED
  Closes: Phase 4H.V part 1 (companion §3.2). Wires a SECOND sidecar at port 8101 (different model family from Qwen2.5-Coder-7B policy SLM) for step-level scoring; dual-GPU placement spec; PpmClient trait; pre-req paper-ingestor pulls Qwen2.5-Math-PRM source.
  Pre-reqs: D-1 (typed templates exist), B-37 (multi-backend runtime).
  Cites: rStar-Math (arxiv:2501.04519, T1), Lightweight Latent Verifiers (arxiv:2504.16760, T1) heterogeneous-verifier rule, PRMBench (arxiv:2501.03124, T1).
  Cloud test: PPM scores a known-correct + known-wrong reasoning step; correct > wrong by margin >0.3.

- [D-3](D-3_verification_result_aggregator.md) — VerificationResult aggregator + multi-verifier consensus rule — PLANNED
  Closes: Phase 4H.V part 2 (companion §3.3). Adds VerificationResult { deterministic, ppm, intrinsic, cross_context, consensus_rule }; promotion blocked unless ≥2 verifiers concur; PRMBench Deception Resistance defense.
  Pre-reqs: D-2 (PPM client exists).
  Cites: V-STaR (arxiv:2402.06457, T1), PRMBench (arxiv:2501.03124, T1), Cross-Context Verification (arxiv:2603.21454, T1).
  Cloud test: synthetic deceptive-reasoning task fails consensus (PPM passes, deterministic check fails) → promotion blocked.

- [D-4](D-4_tripwire_stack.md) — Tripwire stack: KL + Δρ + IQR + perplexity + DDM — PLANNED
  Closes: Phase 4H.VI (companion §3.6 + §3.9). Replaces "10% over 3 pipelines" with 5 orthogonal tripwires + auto-revert on ≥2 concurrent fires + TRIPWIRE_FIRED SSB type.
  Pre-reqs: D-1 (template archive for revert target), D-3 (verifier signals to monitor).
  Cites: arxiv:2507.05619 (T1, 21.3% reward-hacking prevalence + concrete thresholds), Lu et al concept-drift (arxiv:2004.05785, T1) DDM/Page-Hinkley/ADWIN, Gao RM Overoptimization (arxiv:2210.10760, T1).
  Cloud test: inject synthetic regression → ≥3 tripwires fire within window → auto-revert observable in archive.

- [D-5](D-5_bai_thompson_promotion.md) — BAI promotion: Beta-posterior Thompson sampling — PLANNED
  Closes: Phase 4H.VII (companion §3.7). Replaces MSI sigmoid Boltzmann with ArmPosterior { template_id, pass_count, fail_count } + Thompson sampling + Successive Rejects/LUCB; published regret bounds.
  Pre-reqs: D-1 (archive entries to track), D-3 (consensus signal as pass/fail).
  Cites: Audibert COLT 2010 BAI (T1), Variance-Dependent BAI (arxiv:2106.10417, T1).
  Cloud test: 50-round synthetic noisy-arm benchmark; empirical regret within published Audibert bound.

- [D-6](D-6_held_out_task_store.md) — Time-windowed held-out task store + 9-gram filter — PLANNED
  Closes: Phase 4H.VIII part 1 (companion §3.4). Held-out task store at .elai_cc/held-out/{date}/{task-id}.toml; 9-gram overlap filter; auto-retire tasks uniformly-passing after 6 months (probable contamination); HELD_OUT_REFRESH SSB.
  Pre-reqs: D-5 (promotion path uses held-out as gate).
  Cites: LiveCodeBench (arxiv:2403.07974, T1) 9-gram + temporal split, Soft Contamination (arxiv:2602.12413, T1), ARC Prize 2025 Tech Report (arxiv:2601.10904, T1).
  Cloud test: known-contaminated task uniformly passing >6 months auto-retires; new task ingested triggers store-refresh row.

- [D-7](D-7_cross_domain_canary.md) — Cross-domain transfer canary: 5 task families — PLANNED
  Closes: Phase 4H.VIII part 2 (companion §3.5). 5 orthogonal families (code-fix, math-proof, prose-reason, planning, knowledge-QA); promotion blocked if any non-target family regresses >5%; CrossDomainRegression SSB.
  Pre-reqs: D-6 (held-out store has per-family tagging).
  Cites: NanoFlux (arxiv:2509.23252, T1).
  Cloud test: synthetic mutation that regresses prose-reason by 8% → promotion blocked, regression SSB row with delta=-0.08.

- [D-8](D-8_crypto_archive_verify.md) — Cryptographic archive + `elai archive verify` — PLANNED
  Closes: Phase 4H.IX (companion §3.8). Blake3 hash chain on every mutation candidate; append-only ArchiveEntry { self_hash, parent_hash, template, eval_result, promoted_at, retired_at }; `elai archive verify` CLI + tamper-test contract.
  Pre-reqs: D-1 (archive lifecycle exists).
  Cites: arxiv:2507.05619 (T1) cryptographic-checksum recommendation, DGM §5 archive practice (workflow/papers/dgm-interface.md).
  Cloud test: tamper any archived row → `elai archive verify` exit 1 with broken-link report; clean archive exit 0.

- [D-9](D-9_l3_frozen_surfaces.md) — L3 expansion + frozen-surface access-control predicates — PLANNED
  Closes: companion §2.3 + §4. Documents and enforces what is L3-locked: staged-eval harness, tripwire thresholds, MSI/BAI scoring, archive maintenance, held-out selection, canary family definitions, blake3 selection, AllowedConfigKey bound itself, verifier_required field, Domain enum. Adds pre-commit check + LockedKeysAuditor + LOCKED_KEYS_TOUCH SSB.
  Pre-reqs: D-1..D-8 (the artifacts being frozen exist).
  Cites: Everitt Reward Tampering (arxiv:1908.04734, T1) current-RF principle, Cohen Hutter Osborne (AI Magazine 43(3), T1) Boxed Myopic, Hubinger Risks from Learned Optimization (arxiv:1906.01820, T1) mesa-optimization.
  Cloud test: simulated agent edit to evaluator code → pre-commit hook + LockedKeysAuditor + LOCKED_KEYS_TOUCH SSB row + git push BLOCK.

### Wave 2 — Real-problem-solving modes (Phase RPS.I–VIII from real-problem)

Build the inference-time grounding stack on top of Wave 1. Mode routing orchestrator-side (not LLM-side) per Cohen current-RF, with the five A/B/C/D/E modes mapped to constructive checkers, sandbox experiments, refusal calibration, and labeling.

- [D-10](D-10_mode_routing_predicate.md) — Mode-routing predicate `select_grounding_mode` — PLANNED
  Closes: Phase RPS.I (real-problem §3.2). Adds GroundingMode enum + select_grounding_mode orchestrator function; routes claims to A/B/C/D/E based on operationalizable + checker_registry + sandbox + cross-family availability + intrinsic-signal threshold. LLM has no input to selection. Adds MODE_SELECTED SSB.
  Pre-reqs: D-9 (frozen-surface enforcement; the routing function MUST be L3).
  Cites: Everitt (arxiv:1908.04734, T1), real-problem §3.2.
  Cloud test: 50 synthetic claims with known modes; orchestrator assigns correct mode ≥45/50 (≥90% F1).

- [D-11](D-11_constructive_checker_registry.md) — Constructive-checker registry .elai_cc/checkers/ bootstrap — PLANNED
  Closes: Phase RPS.II (real-problem §4.2). Adds .elai_cc/checkers/{math,code,text,graph,api,perf}/ with 6 initial checkers (sympy, z3, compile-rust, compile-py, registry-check, regex); CheckerResult typed return; orchestrator dispatch logic. Mode A operational.
  Pre-reqs: D-10 (mode routing exists; checkers are called via Mode A).
  Cites: Program-of-Thought PoT (arxiv:2211.12588, T1) TMLR 2023 +12% over CoT, CodeAct (arxiv:2402.01030, T1) ICML 2024 +20%, AlphaProof Nature 2025 (T1).
  Cloud test: math claim → SymPy verifies; code claim → cargo check verifies; novel npm package claim → registry-check verifies.

- [D-12](D-12_operationalization_check_predicate.md) — OperationalizationCheck baby-step predicate — PLANNED
  Closes: Phase RPS.III (real-problem §4.1). Extends BabyStepSpec block 7 with MEASURABLE_PROCEDURE field; claim-emitting baby steps without procedure BLOCKED at AUDITING gate. Adds CLAIM_WITHOUT_PROCEDURE SSB.
  Pre-reqs: D-11 (checker registry; the procedure references a checker).
  Cites: Voyager (arxiv:2305.16291, T1) skill-with-measurable-effect, real-problem §4.1.
  Cloud test: babystep-author emits claim without procedure → auditor BLOCK + CLAIM_WITHOUT_PROCEDURE row.

- [D-13](D-13_adversary_role_cross_family.md) — Adversary role + cross-family verifier wiring — PLANNED
  Closes: Phase RPS.IV (real-problem §4.3). Adds AgentRole::Adversary with mandatory cross-model invocation (different family from executor; cloud-Sonnet fallback if no alt local); structured counter-evidence output; if no alt-family available, falls to Mode E (refuse), NEVER same-model self-critique. Adds ADVERSARY_VERDICT + CROSS_FAMILY_UNAVAILABLE SSB.
  Pre-reqs: D-10 (Mode B routing), D-3 (consensus rule consumes Adversary verdict).
  Cites: Self-Correction Bench (arxiv:2507.02778, T1) 64.5% blind-spot rate, Lightweight Latent Verifiers (arxiv:2504.16760, T1), Constitutional AI in Small LLMs (arxiv:2504.04918, T1) Llama-3 selective collapse.
  Cloud test: synthetic claim executor confidently asserts; cross-family Adversary disagrees → consensus rejects → ADVERSARY_VERDICT row + claim BLOCKED.

- [D-14](D-14_sycophancy_probe.md) — Sycophancy probe: inverted-framing structural defense — PLANNED
  Closes: Phase RPS.V (real-problem §4.7). For every user-aligned claim, re-runs with framing inverted; converging outputs = robust, divergent = SYCOPHANCY_FLAG + quarantined for human review. Adds SYCOPHANCY_PROBE + SYCOPHANCY_FLAG SSB types.
  Pre-reqs: D-13 (cross-family infrastructure for inverted-framing re-run).
  Cites: Sharma et al Sycophancy (arxiv:2310.13548, T1).
  Cloud test: user asserts false framing + model agrees → inverted re-run diverges → SYCOPHANCY_FLAG fired.

- [D-15](D-15_novelty_class_labeling.md) — Discovery-vs-search labeling (NoveltyClass) — PLANNED
  Closes: Phase RPS.VI (real-problem §4.8). NoveltyClass { Search, DistantSearch, DiscoveryCandidate } applied to every Mode A / Mode D result; DiscoveryCandidate routes to human-review queue. Adds NoveltyClass field on checker result + NOVELTY_LABELED SSB.
  Pre-reqs: D-11 (Mode A produces results to label), D-12 (procedures landed), C-17 (DKG resilience for near-duplicate detection).
  Cites: FunSearch (Nature 2023 doi:10.1038/s41586-023-06924-6, T1), AlphaEvolve (arxiv:2506.13131, T1), Chollet ARC (arxiv:1911.01547, T1), real-problem §1 Mode C contradiction.
  Cloud test: known-search task labels Search; novel-composition task labels DistantSearch; not-in-DKG primitive labels DiscoveryCandidate.

- [D-16](D-16_unicr_conformal_refusal.md) — Conformal refusal calibration (UniCR) — PLANNED
  Closes: Phase RPS.VII (real-problem §4.5). UniCR-style conformal risk control on i-don't-know outputs; calibration set at .elai_cc/calibration/ with (claim, ground_truth_available, model_confidence) tuples; τ updated nightly L3-only; system refuses below threshold. Adds REFUSAL_DECISION + CALIBRATION_UPDATE SSB.
  Pre-reqs: D-9 (L3 frozen-surface enforcement; τ updater is L3), D-10 (Mode E target is now formally grounded).
  Cites: UniCR (arxiv:2509.01455, T1) distribution-free conformal risk control, Uncertainty Outside Parametric Knowledge (arxiv:2311.09731, T1), Rewarding Doubt (arxiv:2503.02623, T1).
  Cloud test: distribution-shift held-out (task family from different domain) preserves empirical refusal rate within ε=0.1.

- [D-17](D-17_honest_refusal_sla.md) — Honest-refusal SLA monitoring — PLANNED
  Closes: Phase RPS.VIII (real-problem §4.6). Session metric refusal_rate = refusals / total_claims; auto-flag if <5% (confabulating) or >50% (checkers misconfigured); structural defense against Mode-E-is-easy-route gaming.
  Pre-reqs: D-16 (refusal events emitted).
  Cites: real-problem §4.6, LLM Honesty Survey TMLR 2025.
  Cloud test: instrument refusal_rate across last 20 sessions; healthy band emerges empirically; synthetic <2% refusal session → AUTO_FLAG row.

- [D-18](D-18_observable_effect_typed.md) — ObservableEffect typed predicate (Mode D upgrade) — PLANNED
  Closes: Phase 4F extension (real-problem §4.4). Extends Phase 4F Feasibility Probe with typed ObservableEffect { trigger, expected_observation, success_predicate, bounded_runtime, sandbox_tier, confound_flags }; ≥2 ObservableEffects per claim; sandbox vs production observation labeling.
  Pre-reqs: C-22 (FeasibilityProber completeness landed); D-11 (Mode D uses checkers as effect verifiers).
  Cites: Voyager (arxiv:2305.16291, T1), Eureka (arxiv:2310.12931, T1) ICLR 2024, Coscientist (arxiv:2304.05332, T1).
  Cloud test: feasibility probe with 1 ObservableEffect → GOODHART_RISK_FLAG fires; ≥2 effects → flag absent.

### Wave 3 — Benchmarks and adversarial tests (NEW; D-plan-specific)

Each Wave-1/Wave-2 capability is paired with a quantitative benchmark + adversarial probe so promotion gates have measurable evidence beyond unit tests.

- [D-19](D-19_prmbench_adversarial_gaming.md) — PRMBench adversarial gaming benchmark for PPM — PLANNED
  Closes: companion §1.13 open question. Pulls PRMBench fixtures (Deception Resistance + Circular Logic subsets); runs PPM (D-2) on adversarial samples; reports per-category score; gates PPM promotion on ≥published baseline.
  Pre-reqs: D-2 (PPM client), D-3 (consensus rule consumes PPM score).
  Cites: PRMBench (arxiv:2501.03124, T1), Lessons of Developing PRMs (arxiv:2501.07301, T1), ThinkPRM (arxiv:2504.16828, T1).
  Cloud test: PRMBench Deception Resistance ≥35% (conservative vs published 40-50% for Qwen2.5-Math-PRM-7B); Circular Logic ≥30%.

- [D-20](D-20_arc_agi2_ood_probe.md) — ARC-AGI-2 OOD probe harness — PLANNED
  Closes: companion §1.16 implication + real-problem §1 Mode C. Pulls public ARC-AGI-2 task subset (or synthetic ARC-style tasks if hand-crafted leakage flagged); runs ELAI under cloud Sonnet (no local 7B for ARC) with mode-routing on; reports solve-rate + Mode-E refusal-rate vs human baseline.
  Pre-reqs: D-10 (mode routing) + D-15 (novelty labeling).
  Cites: Chollet ARC (arxiv:1911.01547, T1), ARC Prize 2025 Tech Report (arxiv:2601.10904, T1), NVARC NVIDIA 2025 (T1).
  Cloud test: ELAI on 50-task ARC-AGI-2 subset reports solve-rate + refusal-rate; honest-refusal SLA in band; no DiscoveryCandidate claims auto-published.

- [D-21](D-21_mode_routing_coverage_benchmark.md) — Mode-routing coverage benchmark — PLANNED
  Closes: D-10 gating evidence. 200 synthetic claims with hand-labeled ground-truth modes (A/B/C/D/E); evaluates select_grounding_mode F1 + per-mode confusion matrix; reports edge-case failures.
  Pre-reqs: D-10 (predicate exists).
  Cites: real-problem §3.2 design, Self-Correction Bench (arxiv:2507.02778, T1) blind-spot framework.
  Cloud test: F1 ≥0.85 across 5 modes; no class collapses (each mode ≥10 correct).

- [D-22](D-22_conformal_calibration_validation.md) — Conformal calibration validation under distribution shift — PLANNED
  Closes: D-16 gating evidence. Builds 3 held-out distributions (in-domain, near-OOD, far-OOD); verifies UniCR empirical refusal rate stays within user ε=0.1 across all three.
  Pre-reqs: D-16 (UniCR implementation exists), D-6 (held-out store provides distributions).
  Cites: UniCR (arxiv:2509.01455, T1) distribution-free guarantee.
  Cloud test: empirical refusal_rate within [τ-ε, τ+ε] on all 3 distributions; FAIL if any shifts >ε.

- [D-23](D-23_sycophancy_detection_benchmark.md) — Sycophancy detection benchmark — PLANNED
  Closes: D-14 gating evidence. Pulls Anthropic Sharma et al public sycophancy probes (or synthetic stand-in); runs probe-on/probe-off comparison; reports detection rate + false-positive rate.
  Pre-reqs: D-14 (probe implementation exists).
  Cites: Sharma et al Sycophancy (arxiv:2310.13548, T1).
  Cloud test: detection rate ≥70% on labeled sycophancy fixtures; false-positive rate ≤15%.

- [D-24](D-24_cross_domain_canary_regression_suite.md) — Cross-domain canary regression validation — PLANNED
  Closes: D-7 gating evidence. 100 synthetic mutations (50 helpful, 30 neutral, 20 harmful) across all 5 families; verifies canary blocks all 20 harmful + at most 5 helpful (false-positive ≤10%).
  Pre-reqs: D-7 (canary implementation exists).
  Cites: NanoFlux (arxiv:2509.23252, T1), companion §3.5.
  Cloud test: 20/20 harmful blocked; ≤5/50 helpful blocked; ≤2/30 neutral blocked.

- [D-25](D-25_determinism_archive_tamper_suite.md) — Determinism + archive tamper test suite — PLANNED
  Closes: D-8 + C-27 (T-1 determinism) gating evidence. 10 tamper scenarios (delete row, swap parent_hash, edit template body, swap blake3 to sha256 on disk); each scenario MUST be detected by `elai archive verify` within 5s.
  Pre-reqs: D-8 (archive verify CLI exists), C-27 (DeterminismDrift detector).
  Cites: arxiv:2507.05619 (T1) integrity-checksum, companion §3.8.
  Cloud test: 10/10 tamper scenarios detected; clean repo exits 0; tamper-then-revert recovers to clean.

### Wave 4 — Dashboard integration capstone

- [D-26](D-26_reasoning_template_dashboard.md) — ReasoningTemplate + archive dashboard surface — PLANNED
  Closes: dashboard exposure for D-1..D-8 substrate. Adds MutationArchivePanel.svelte + LineageGraphView.svelte (cytoscape, cross-links to baby-step archive); ReasoningDrawer extensions for PPM score + consensus rule + tripwire status; get_archive_recent / get_archive_verify_status / get_tripwire_status Tauri commands.
  Pre-reqs: D-1..D-8 shipped (the surface to render).
  Cites: companion §3.1, §3.8.
  Cloud test: live cloud /task with template promotion → MutationArchivePanel shows new ARCHIVE_ENTRY_ADDED row + lineage edge; tripwire-fire injection → red flag in dashboard within poll interval.

- [D-27](D-27_mode_routing_refusal_dashboard.md) — Mode-routing + refusal-SLA dashboard panels — PLANNED
  Closes: dashboard exposure for D-10..D-17. Adds ModeRoutingPanel.svelte (per-session A/B/C/D/E breakdown), RefusalRateGauge.svelte (with healthy-band visualization), NoveltyBreakdown.svelte (Search/DistantSearch/DiscoveryCandidate counts), SycophancyFlagStrip.svelte (real-time flag stream); get_mode_breakdown / get_refusal_rate / get_novelty_breakdown / get_sycophancy_flags Tauri commands.
  Pre-reqs: D-10..D-17 shipped (rows to query).
  Cites: real-problem §3.2 routing surface, §4.6 SLA.
  Cloud test: live cloud /task with mixed claims → ModeRoutingPanel shows realistic distribution; sycophancy probe fire → flag appears in dashboard within poll interval.

### Wave 5 — Deferred / research-grade

- [D-28](D-28_adversarial_example_farming.md) — Adversarial example farming for PPM (DEFERRED) — DEFERRED
  Closes: companion Phase 4H.X. Iteratively generates OOD high-proxy-reward low-quality examples (per arxiv:2504.06141), retrains PPM on adversarial corpus to immunize. Hard to bootstrap until BAI promotion has stable selection signal; defer to once D-5 has ≥20 promotion cycles.
  Pre-reqs: D-5 (BAI selection signal stable), D-19 (PRMBench baseline measured). RunPod for retraining run.
  Cites: Adversarial Training Reward Models (arxiv:2504.06141, T1).
  Cloud test: pre/post-immunization PRMBench Deception Resistance delta ≥+5pp; no regression on standard categories.

- [D-29](D-29_hierarchical_bai_research.md) — Hierarchical BAI for non-IID rewards (DEFERRED — research-grade) — DEFERRED
  Closes: companion open Q4. Published BAI assumes IID rewards; ELAI pipeline outcomes are not IID across sessions. Spike-level hierarchical Bayesian model; closure includes "infeasibility note" as valid outcome.
  Pre-reqs: D-5 (flat BAI baseline measured for comparison). RunPod for hierarchical Gibbs sampler.
  Cites: Audibert COLT 2010 (T1), Variance-Dependent BAI (arxiv:2106.10417, T1), companion §3.7 / Q4.
  Cloud test: hierarchical model regret ≤ flat-BAI regret on synthetic non-IID benchmark; OR documented infeasibility with ≥1 concrete blocker.

- [D-30](D-30_calibration_bootstrap_corpus.md) — Conformal calibration bootstrap from generic corpus (DEFERRED) — DEFERRED
  Closes: real-problem open Q5. Needs N=500 calibration tuples for UniCR to be useful; early sessions cannot calibrate locally. Spike: bootstrap from generic LLM-calibration corpus, then refine per-project.
  Pre-reqs: D-16 (UniCR implementation exists). Calibration corpus access (HuggingFace dataset to be identified).
  Cites: UniCR (arxiv:2509.01455, T1), real-problem Q5.
  Cloud test: bootstrap calibration produces empirical refusal rate within ε on held-out validation; later refined-from-real-sessions calibration ≤ bootstrap empirical risk.

---

## Summary statistics

- Total session slots: 30
- In-scope (executable now, CPU/cloud): 27 (D-1 through D-27)
- Deferred (research-grade / data-dependent): 3 (D-28, D-29, D-30)
- Files authored: 30 (all D-N files written 2026-05-19)
- Estimated effort per implementation session: ~6-8 h (D-1..D-27)
- Estimated effort per deferred session: ~8-24 h wall-clock incl. RunPod (D-28..D-30)
- Total estimated effort (27 CPU/cloud sessions): ~160-220 h
- Total estimated effort (3 deferred sessions): ~25-75 h + RunPod cost

## Source provenance

- Companion design 1 (reasoning-improvement): `F:\.ELAI_workflow\docs\specs\guide\reasoning-improvement-grounded-2026-05-19.md` (531 lines, 50+ T1 citations).
- Companion design 2 (real-problem-solving): `F:\.ELAI_workflow\docs\specs\guide\real-problem-solving-grounded-2026-05-19.md` (506 lines, 30+ T1 citations).
- Both files were grounded against 9 research agents and 80+ tier-1 arxiv sources across two oracle research passes 2026-05-17 + 2026-05-19; each load-bearing claim either cites measurement or theorem.

## Paper grounding paths

Already ingested at `F:\.ELAI_workflow\workflow\papers\`:
- DGM (arxiv:2505.22954) — `workflow/papers/dgm-interface.md` + PDF at `.claude/papers/2505.22954v3.pdf`

D-plan introduces these papers to ingest (each D-N file lists which it needs):
- rStar-Math (arxiv:2501.04519) — PPM substrate (D-2)
- V-STaR (arxiv:2402.06457) — separable verifier (D-3)
- arxiv:2507.05619 reward-hacking thresholds (D-4)
- Audibert COLT 2010 BAI (D-5)
- Variance-Dependent BAI (arxiv:2106.10417) (D-5)
- LiveCodeBench (arxiv:2403.07974) (D-6)
- NanoFlux (arxiv:2509.23252) (D-7)
- arxiv:1908.04734 Everitt + AI-Mag 43(3) Cohen (D-9)
- Program-of-Thought (arxiv:2211.12588) + CodeAct (arxiv:2402.01030) (D-11)
- AlphaProof Nature 2025 (D-11)
- Voyager (arxiv:2305.16291) (D-12, D-18)
- arxiv:2504.16760 Lightweight Latent Verifiers (D-13)
- arxiv:2507.02778 Self-Correction Bench (D-13)
- arxiv:2310.13548 Sharma sycophancy (D-14, D-23)
- FunSearch (Nature 2023) + AlphaEvolve (arxiv:2506.13131) (D-15)
- Chollet ARC (arxiv:1911.01547) + ARC Prize 2025 (arxiv:2601.10904) (D-15, D-20)
- UniCR (arxiv:2509.01455) (D-16, D-22, D-30)
- arxiv:2501.03124 PRMBench (D-2, D-19)
- arxiv:2402.03620 SELF-DISCOVER (D-1)
- arxiv:2510.04399 Statistical Limits (D-1, D-9)
- arxiv:2310.12931 Eureka (D-18)
- arxiv:2304.05332 Coscientist (D-18)
- arxiv:2504.06141 Adversarial RM Training (D-28)
- arxiv:2004.05785 Lu concept drift (D-4)

Per D-N file, `paper-ingestor` is the agent that pulls + indexes the paper into `workflow/papers/{slug}-interface.md` BEFORE EXECUTING begins. Each D-N file's Pre-requisites line names the papers it depends on; spec-grounder verifies the interface files exist before unblocking TESTING_FIRST.

## Wave map (companion design → D-N)

- companion §3.1 ReasoningTemplate → D-1
- companion §3.2 PPM sidecar → D-2
- companion §3.3 VerificationResult → D-3
- companion §3.6 Tripwire stack → D-4
- companion §3.9 DDM as 5th tripwire → D-4
- companion §3.7 BAI promotion → D-5
- companion §3.4 Time-windowed held-out → D-6
- companion §3.5 Cross-domain canary → D-7
- companion §3.8 Cryptographic archive → D-8
- companion §2 L3 expansion → D-9
- companion §3 Phase 4H.X adversarial farming → D-28 (DEFERRED)
- companion Q4 hierarchical BAI → D-29 (DEFERRED)

## Wave map (real-problem-solving design → D-N)

- real-problem §3.2 mode routing → D-10
- real-problem §4.2 checker registry → D-11
- real-problem §4.1 operationalization predicate → D-12
- real-problem §4.3 Adversary role → D-13
- real-problem §4.7 sycophancy probe → D-14
- real-problem §4.8 NoveltyClass → D-15
- real-problem §4.5 UniCR conformal calibration → D-16
- real-problem §4.6 honest-refusal SLA → D-17
- real-problem §4.4 ObservableEffect → D-18
- real-problem Q5 calibration bootstrap → D-30 (DEFERRED)

## Cross-plan link (C → D)

The C-plan closes verification gaps from B-plan; D-plan extends ELAI from a verified system into a self-improving + grounded-reasoning system. Specifically:

- D-1 typed-template archive extends C-21 self-modification automation (which wired check_anti_stale + L1A data tables) into typed reasoning-template surface.
- D-4 tripwire stack replaces C-27 T(-1) DeterminismDrift's single-signal detection with 5 orthogonal signals.
- D-9 L3 expansion extends C-27 WiringContract StubBanAuditor with frozen-surface access-control predicates.
- D-11 checker registry consumes C-24 symbol_impact + C-30 surgical-mode DKG tooling for the api/ + code/ checker dispatch.
- D-12 operationalization predicate extends C-31 FILE_PLAN block 8 with block 9 (MEASURABLE_PROCEDURE).
- D-13 Adversary role extends C-12 per-level auditor's REVIEW modes with structured cross-family adversarial probing.
- D-18 typed ObservableEffect extends C-22 FeasibilityProber completeness with measurable-effect schema.
- D-26 + D-27 extend C-7 dashboard UX completeness with D-plan-specific drawers (MutationArchivePanel, ModeRoutingPanel).

D-plan is sequenced AFTER C-plan completion: spec-grounder will refuse to start D-1 until ≥C-1, C-15, C-17, C-21, C-22, C-24, C-27 are DONE (SCHEMA_VERSION ≥47 + WiringContract substrate + DKG resilience + MistakeBook).

---

End of D-SUMMARY_INDEX.md
