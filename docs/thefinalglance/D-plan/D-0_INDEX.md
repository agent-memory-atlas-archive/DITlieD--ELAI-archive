# D-plan: ELAI reasoning-improvement + real-problem-solving roadmap (post-C, full index)
Generated: 2026-05-19
Source: docs/specs/guide/reasoning-improvement-grounded-2026-05-19.md + docs/specs/guide/real-problem-solving-grounded-2026-05-19.md.
Premise: C-plan closed verification gaps; D-plan ships substrate for empirical recursive self-improvement (DGM-class) and inference-time grounding (5-mode taxonomy). Each D-N session ships one cohesive module, wires it to the dashboard, and proves it with a real cloud-Sonnet test plus a paper-baselined benchmark.

## Reading order

D-N files are dependency-ordered. The reasoning-improvement substrate (D-1..D-9) MUST land before real-problem-solving modes (D-10..D-18); the modes consume typed templates + PPM + tripwires + archive + L3 frozen surfaces. Wave 3 benchmarks (D-19..D-25) gate promotion of their parent Wave-1/Wave-2 capability. Wave 4 dashboards (D-26..D-27) surface everything. Wave 5 (D-28..D-30) is deferred.

## Status legend
- PLANNED — file written, work not started
- IN PROGRESS — currently being shipped in an active session
- DONE — shipped, cloud-Sonnet smoke test passed, benchmark threshold met
- DEFERRED — blocked on calibration data, hardware, or open research question

## Sessions

### Wave 1 — Reasoning-improvement substrate (Phase 4H.IV–IX from companion)

- [D-1](D-1_reasoning_template_schema.md) — ReasoningTemplate typed schema + Blake3 content-hash + archive lifecycle — PLANNED
  Closes: Phase 4H.IV (companion §3.1).
  Pre-reqs: C-15 (ClockProvider stable), C-21 (MistakeBook + rules.db stable).
  Cites: SELF-DISCOVER (arxiv:2402.03620, T1), arxiv:2510.04399 (T1), DGM (arxiv:2505.22954, T1).
  Cloud test: typed template promotion writes ARCHIVE_ENTRY_ADDED row + verifiable hash chain.

- [D-2](D-2_ppm_sidecar_wiring.md) — PPM sidecar (Qwen2.5-Math-PRM-7B) — PLANNED
  Closes: Phase 4H.V part 1 (companion §3.2).
  Pre-reqs: D-1, B-37 (multi-backend runtime).
  Cites: rStar-Math (arxiv:2501.04519, T1), Lightweight Latent Verifiers (arxiv:2504.16760, T1), PRMBench (arxiv:2501.03124, T1).
  Cloud test: PPM separates known-correct from known-wrong reasoning step by score margin >0.3.

- [D-3](D-3_verification_result_aggregator.md) — VerificationResult aggregator + multi-verifier consensus — PLANNED
  Closes: Phase 4H.V part 2 (companion §3.3).
  Pre-reqs: D-2.
  Cites: V-STaR (arxiv:2402.06457, T1), PRMBench (arxiv:2501.03124, T1), Cross-Context Verification (arxiv:2603.21454, T1).
  Cloud test: synthetic deceptive-reasoning task → PPM-passes/deterministic-fails → consensus rejects.

- [D-4](D-4_tripwire_stack.md) — Tripwire stack: KL + Δρ + IQR + perplexity + DDM — PLANNED
  Closes: Phase 4H.VI (companion §3.6 + §3.9).
  Pre-reqs: D-1, D-3.
  Cites: arxiv:2507.05619 (T1, 21.3% reward-hacking prevalence + concrete thresholds), Lu et al concept-drift (arxiv:2004.05785, T1), Gao RM Overoptimization (arxiv:2210.10760, T1).
  Cloud test: synthetic regression → ≥3 tripwires fire → auto-revert observable in archive.

- [D-5](D-5_bai_thompson_promotion.md) — BAI promotion: Beta-posterior Thompson sampling — PLANNED
  Closes: Phase 4H.VII (companion §3.7).
  Pre-reqs: D-1, D-3.
  Cites: Audibert COLT 2010 BAI (T1), Variance-Dependent BAI (arxiv:2106.10417, T1).
  Cloud test: 50-round synthetic noisy-arm; empirical regret within published Audibert bound.

- [D-6](D-6_held_out_task_store.md) — Time-windowed held-out task store + 9-gram filter — PLANNED
  Closes: Phase 4H.VIII part 1 (companion §3.4).
  Pre-reqs: D-5.
  Cites: LiveCodeBench (arxiv:2403.07974, T1), Soft Contamination (arxiv:2602.12413, T1), ARC Prize 2025 (arxiv:2601.10904, T1).
  Cloud test: 18-month-old uniformly-passing task auto-retires; new task ingest emits HELD_OUT_REFRESH row.

- [D-7](D-7_cross_domain_canary.md) — Cross-domain canary: 5 task families — PLANNED
  Closes: Phase 4H.VIII part 2 (companion §3.5).
  Pre-reqs: D-6.
  Cites: NanoFlux (arxiv:2509.23252, T1).
  Cloud test: synthetic mutation regressing prose-reason 8% → promotion blocked + CrossDomainRegression row.

- [D-8](D-8_crypto_archive_verify.md) — Cryptographic archive + `elai archive verify` — PLANNED
  Closes: Phase 4H.IX (companion §3.8).
  Pre-reqs: D-1.
  Cites: arxiv:2507.05619 (T1), DGM §5 archive practice.
  Cloud test: tamper any row → exit 1 with broken-link report; clean repo → exit 0.

- [D-9](D-9_l3_frozen_surfaces.md) — L3 expansion + frozen-surface access-control predicates — PLANNED
  Closes: companion §2.3 + §4.
  Pre-reqs: D-1..D-8.
  Cites: Everitt Reward Tampering (arxiv:1908.04734, T1), Cohen Hutter Osborne (AI Magazine 43(3), T1), Hubinger Risks from Learned Optimization (arxiv:1906.01820, T1).
  Cloud test: agent edit to evaluator code → pre-commit BLOCK + LOCKED_KEYS_TOUCH SSB row.

### Wave 2 — Real-problem-solving modes (Phase RPS.I–VIII)

- [D-10](D-10_mode_routing_predicate.md) — Mode-routing predicate `select_grounding_mode` — PLANNED
  Closes: Phase RPS.I (real-problem §3.2).
  Pre-reqs: D-9.
  Cites: Everitt (arxiv:1908.04734, T1).
  Cloud test: 50 synthetic claims; orchestrator assigns correct mode ≥45/50.

- [D-11](D-11_constructive_checker_registry.md) — Constructive-checker registry — PLANNED
  Closes: Phase RPS.II (real-problem §4.2).
  Pre-reqs: D-10.
  Cites: Program-of-Thought PoT (arxiv:2211.12588, T1), CodeAct (arxiv:2402.01030, T1), AlphaProof Nature 2025 (doi:10.1038/s41586-025-09833-y, T1).
  Cloud test: math claim → SymPy verifies; code claim → cargo check verifies; novel npm package → registry-check verifies.

- [D-12](D-12_operationalization_check_predicate.md) — OperationalizationCheck baby-step predicate — PLANNED
  Closes: Phase RPS.III (real-problem §4.1).
  Pre-reqs: D-11.
  Cites: Voyager (arxiv:2305.16291, T1).
  Cloud test: claim without procedure → auditor BLOCK + CLAIM_WITHOUT_PROCEDURE row.

- [D-13](D-13_adversary_role_cross_family.md) — Adversary role + cross-family verifier — PLANNED
  Closes: Phase RPS.IV (real-problem §4.3).
  Pre-reqs: D-10, D-3.
  Cites: Self-Correction Bench (arxiv:2507.02778, T1), Lightweight Latent Verifiers (arxiv:2504.16760, T1), Constitutional AI in Small LLMs (arxiv:2504.04918, T1).
  Cloud test: synthetic confident-wrong claim → cross-family Adversary disagrees → consensus rejects.

- [D-14](D-14_sycophancy_probe.md) — Sycophancy probe: inverted-framing — PLANNED
  Closes: Phase RPS.V (real-problem §4.7).
  Pre-reqs: D-13.
  Cites: Sharma et al Sycophancy (arxiv:2310.13548, T1).
  Cloud test: user-asserts-wrong + model-agrees → inverted re-run diverges → SYCOPHANCY_FLAG fires.

- [D-15](D-15_novelty_class_labeling.md) — Discovery-vs-search labeling (NoveltyClass) — PLANNED
  Closes: Phase RPS.VI (real-problem §4.8).
  Pre-reqs: D-11, D-12, C-17.
  Cites: FunSearch (Nature 2023, T1), AlphaEvolve (arxiv:2506.13131, T1), Chollet ARC (arxiv:1911.01547, T1).
  Cloud test: known-search → Search; novel composition → DistantSearch; not-in-DKG primitive → DiscoveryCandidate.

- [D-16](D-16_unicr_conformal_refusal.md) — Conformal refusal calibration (UniCR) — PLANNED
  Closes: Phase RPS.VII (real-problem §4.5).
  Pre-reqs: D-9, D-10.
  Cites: UniCR (arxiv:2509.01455, T1), Uncertainty Outside Parametric Knowledge (arxiv:2311.09731, T1), Rewarding Doubt (arxiv:2503.02623, T1).
  Cloud test: distribution-shift held-out preserves empirical refusal rate within ε=0.1.

- [D-17](D-17_honest_refusal_sla.md) — Honest-refusal SLA monitoring — PLANNED
  Closes: Phase RPS.VIII (real-problem §4.6).
  Pre-reqs: D-16.
  Cites: real-problem §4.6, LLM Honesty Survey TMLR 2025.
  Cloud test: refusal_rate <2% synthetic session → AUTO_FLAG row.

- [D-18](D-18_observable_effect_typed.md) — ObservableEffect typed predicate (Mode D upgrade) — PLANNED
  Closes: Phase 4F extension (real-problem §4.4).
  Pre-reqs: C-22, D-11.
  Cites: Voyager (arxiv:2305.16291, T1), Eureka (arxiv:2310.12931, T1), Coscientist (arxiv:2304.05332, T1).
  Cloud test: 1 ObservableEffect → GOODHART_RISK_FLAG fires; ≥2 effects → flag absent.

### Wave 3 — Benchmarks and adversarial tests (NEW)

- [D-19](D-19_prmbench_adversarial_gaming.md) — PRMBench adversarial gaming for PPM — PLANNED
  Closes: companion §1.13.
  Pre-reqs: D-2, D-3.
  Cites: PRMBench (arxiv:2501.03124, T1), Lessons of Developing PRMs (arxiv:2501.07301, T1), ThinkPRM (arxiv:2504.16828, T1).
  Cloud test: PRMBench Deception Resistance ≥35%; Circular Logic ≥30%.

- [D-20](D-20_arc_agi2_ood_probe.md) — ARC-AGI-2 OOD probe harness — PLANNED
  Closes: companion §1.16, real-problem §1 Mode C.
  Pre-reqs: D-10, D-15.
  Cites: Chollet ARC (arxiv:1911.01547, T1), ARC Prize 2025 (arxiv:2601.10904, T1), NVARC NVIDIA 2025 (T1).
  Cloud test: 50-task subset; solve-rate + refusal-rate reported vs human baseline; no DiscoveryCandidate auto-published.

- [D-21](D-21_mode_routing_coverage_benchmark.md) — Mode-routing coverage benchmark — PLANNED
  Closes: D-10 gating.
  Pre-reqs: D-10.
  Cites: real-problem §3.2, Self-Correction Bench (arxiv:2507.02778, T1).
  Cloud test: F1 ≥0.85 across 5 modes; no class collapse.

- [D-22](D-22_conformal_calibration_validation.md) — Conformal calibration validation under distribution shift — PLANNED
  Closes: D-16 gating.
  Pre-reqs: D-16, D-6.
  Cites: UniCR (arxiv:2509.01455, T1).
  Cloud test: empirical refusal rate within [τ-ε, τ+ε] on in-domain, near-OOD, far-OOD distributions.

- [D-23](D-23_sycophancy_detection_benchmark.md) — Sycophancy detection benchmark — PLANNED
  Closes: D-14 gating.
  Pre-reqs: D-14.
  Cites: Sharma et al Sycophancy (arxiv:2310.13548, T1).
  Cloud test: detection rate ≥70% on labeled fixtures; FP rate ≤15%.

- [D-24](D-24_cross_domain_canary_regression_suite.md) — Cross-domain canary regression validation — PLANNED
  Closes: D-7 gating.
  Pre-reqs: D-7.
  Cites: NanoFlux (arxiv:2509.23252, T1).
  Cloud test: 20/20 harmful blocked; ≤5/50 helpful blocked; ≤2/30 neutral blocked.

- [D-25](D-25_determinism_archive_tamper_suite.md) — Determinism + archive tamper test suite — PLANNED
  Closes: D-8 + C-27 gating.
  Pre-reqs: D-8, C-27.
  Cites: arxiv:2507.05619 (T1).
  Cloud test: 10/10 tamper scenarios detected; clean repo exits 0.

### Wave 4 — Dashboard integration capstone

- [D-26](D-26_reasoning_template_dashboard.md) — ReasoningTemplate + archive dashboard — PLANNED
  Closes: dashboard surface for D-1..D-8.
  Pre-reqs: D-1..D-8.
  Cites: companion §3.1, §3.8.
  Cloud test: live template promotion → MutationArchivePanel new entry + lineage edge; tripwire-fire injection → red flag.

- [D-27](D-27_mode_routing_refusal_dashboard.md) — Mode-routing + refusal-SLA dashboard — PLANNED
  Closes: dashboard surface for D-10..D-17.
  Pre-reqs: D-10..D-17.
  Cites: real-problem §3.2, §4.6.
  Cloud test: live cloud /task → ModeRoutingPanel shows realistic distribution; sycophancy-probe fire → SycophancyFlagStrip.

### Wave 5 — Deferred / research-grade

- [D-28](D-28_adversarial_example_farming.md) — Adversarial example farming for PPM (DEFERRED) — DEFERRED
  Closes: Phase 4H.X (companion §3 deferred block).
  Pre-reqs: D-5 + D-19 + RunPod.
  Cites: Adversarial Training Reward Models (arxiv:2504.06141, T1).
  Cloud test: pre/post-immunization PRMBench Deception Resistance delta ≥+5pp.

- [D-29](D-29_hierarchical_bai_research.md) — Hierarchical BAI for non-IID rewards (DEFERRED) — DEFERRED
  Closes: companion open Q4.
  Pre-reqs: D-5 + RunPod.
  Cites: Audibert COLT 2010 (T1), Variance-Dependent BAI (arxiv:2106.10417, T1).
  Cloud test: hierarchical regret ≤ flat-BAI on synthetic non-IID, OR infeasibility note with ≥1 concrete blocker.

- [D-30](D-30_calibration_bootstrap_corpus.md) — Conformal calibration bootstrap from generic corpus (DEFERRED) — DEFERRED
  Closes: real-problem open Q5.
  Pre-reqs: D-16, calibration corpus access.
  Cites: UniCR (arxiv:2509.01455, T1).
  Cloud test: bootstrap empirical refusal within ε; refined-from-real-sessions calibration ≤ bootstrap empirical risk.

---

## Per-session status notes

(Status notes accumulate as sessions ship. Pattern: SHIPPED <date>: <key artifact + benchmark number>. SCHEMA_VERSION bumps and SsbActionType additions logged per-session for cross-plan reconciliation with B-0_INDEX.)

### D-1 — ReasoningTemplate schema
Status: PLANNED.
Expected SCHEMA_VERSION bump: 47 → 48.
Expected SsbActionType additions: ARCHIVE_ENTRY_ADDED, ARCHIVE_ENTRY_RETIRED, TEMPLATE_PROMOTED, TEMPLATE_CANDIDATE_REJECTED.

### D-2 — PPM sidecar
Status: PLANNED.
Expected sidecar port: 8101 (policy SLM is 8100). Dual-GPU placement: policy on GPU0, PPM on GPU1.

### D-3 — VerificationResult aggregator
Status: PLANNED.
Expected SsbActionType additions: VERIFIER_CONSENSUS_PASS, VERIFIER_CONSENSUS_REJECT, VERIFIER_DISAGREEMENT.

### D-4 — Tripwire stack
Status: PLANNED.
Expected SCHEMA_VERSION bump: same session as D-3 atomic bump.
Expected SsbActionType additions: TRIPWIRE_FIRED (kind: KlDivergence | CorrelationDrop | IqrOutlier | PerplexityAnomaly | Ddm).

### D-5 — BAI Thompson promotion
Status: PLANNED.
Expected files: orchestrator/src/bai.rs (new), orchestrator/src/promotion_arms.rs (new).
Expected SsbActionType additions: ARM_POSTERIOR_UPDATED, THOMPSON_SAMPLED, BAI_PROMOTION_DECISION.

### D-6 — Held-out task store
Status: PLANNED.
Expected directory: .elai_cc/held-out/{date}/{task-id}.toml (append-only).
Expected SsbActionType additions: HELD_OUT_REFRESH, HELD_OUT_AUTO_RETIRE.

### D-7 — Cross-domain canary
Status: PLANNED.
Expected SsbActionType additions: CROSS_DOMAIN_REGRESSION, CANARY_FAMILY_PASS.

### D-8 — Cryptographic archive verify
Status: PLANNED.
Expected CLI: `elai archive verify` (new subcommand).
Expected files: orchestrator/src/archive_chain.rs (new), cli/src/archive_verify.rs (new).

### D-9 — L3 frozen surfaces
Status: PLANNED.
Expected pre-commit hook: scripts/locked_keys_check.sh.
Expected SsbActionType additions: LOCKED_KEYS_TOUCH, L3_OVERRIDE_REQUESTED.

### D-10 — Mode-routing predicate
Status: PLANNED.
Expected SCHEMA_VERSION bump: atomic with D-11+D-12.
Expected SsbActionType additions: MODE_SELECTED.

### D-11 — Constructive-checker registry
Status: PLANNED.
Expected directory: .elai_cc/checkers/{math,code,text,graph,api,perf}/ with 6 initial checkers.
Expected SsbActionType additions: CHECKER_DISPATCHED, CHECKER_PASS, CHECKER_FAIL, CHECKER_INAPPLICABLE.

### D-12 — OperationalizationCheck
Status: PLANNED.
Expected BabyStepSpec extension: block 9 (MEASURABLE_PROCEDURE).
Expected SsbActionType additions: CLAIM_WITHOUT_PROCEDURE, PROCEDURE_EXECUTED.

### D-13 — Adversary role
Status: PLANNED.
Expected AgentRole addition: Adversary.
Expected SsbActionType additions: ADVERSARY_VERDICT, CROSS_FAMILY_UNAVAILABLE.

### D-14 — Sycophancy probe
Status: PLANNED.
Expected SsbActionType additions: SYCOPHANCY_PROBE, SYCOPHANCY_FLAG.

### D-15 — NoveltyClass labeling
Status: PLANNED.
Expected NoveltyClass enum: Search, DistantSearch, DiscoveryCandidate.
Expected SsbActionType additions: NOVELTY_LABELED, DISCOVERY_CANDIDATE_QUEUED.

### D-16 — UniCR conformal refusal
Status: PLANNED.
Expected directory: .elai_cc/calibration/.
Expected SsbActionType additions: REFUSAL_DECISION, CALIBRATION_UPDATE.

### D-17 — Honest-refusal SLA
Status: PLANNED.
Expected dashboard gauge: refusal_rate (RefusalRateGauge.svelte in D-27).
Expected SsbActionType additions: REFUSAL_SLA_BREACH.

### D-18 — ObservableEffect typed
Status: PLANNED.
Expected struct: ObservableEffect (orchestrator/src/observable_effect.rs).
Expected SsbActionType additions: GOODHART_RISK_FLAG.

### D-19 — PRMBench adversarial gaming
Status: PLANNED.
Expected artifact: workflow/papers/prmbench-2501.03124-interface.md (paper-ingestor).
Expected baseline: per published PRMBench paper Table 3.

### D-20 — ARC-AGI-2 probe
Status: PLANNED.
Expected harness: .elai_cc/benchmarks/arc_agi2/ (subset of public ARC tasks).
Expected baseline: ARC Prize 2025 leaderboard human=60-70%, NVARC=24%.

### D-21 — Mode-routing coverage
Status: PLANNED.
Expected fixture: .elai_cc/benchmarks/mode_routing/{200 synthetic claims}.

### D-22 — Conformal calibration validation
Status: PLANNED.
Expected fixture: .elai_cc/benchmarks/conformal/{in_domain,near_ood,far_ood}/.

### D-23 — Sycophancy benchmark
Status: PLANNED.
Expected fixture: .elai_cc/benchmarks/sycophancy/ (per Sharma probes).

### D-24 — Canary regression suite
Status: PLANNED.
Expected fixture: .elai_cc/benchmarks/canary/{50 helpful, 30 neutral, 20 harmful}.

### D-25 — Tamper test suite
Status: PLANNED.
Expected fixture: .elai_cc/benchmarks/tamper/{10 scenarios}.

### D-26 — Reasoning-template dashboard
Status: PLANNED.
Expected Svelte components: MutationArchivePanel.svelte, LineageGraphView.svelte (cytoscape).
Expected Tauri commands: get_archive_recent, get_archive_verify_status, get_tripwire_status.

### D-27 — Mode-routing + refusal dashboard
Status: PLANNED.
Expected Svelte components: ModeRoutingPanel.svelte, RefusalRateGauge.svelte, NoveltyBreakdown.svelte, SycophancyFlagStrip.svelte.
Expected Tauri commands: get_mode_breakdown, get_refusal_rate, get_novelty_breakdown, get_sycophancy_flags.

### D-28 — Adversarial example farming
Status: DEFERRED. Reason: requires ≥20 BAI promotion cycles (D-5 must run live first) + RunPod for retraining.

### D-29 — Hierarchical BAI
Status: DEFERRED. Reason: research-grade; closure includes infeasibility note as valid outcome.

### D-30 — Calibration bootstrap corpus
Status: DEFERRED. Reason: needs N=500 calibration tuples or generic corpus access; defer until D-16 has empirical signal.

---

## Summary statistics

- Total session slots: 30
- In-scope: 27 (D-1..D-27)
- Deferred: 3 (D-28, D-29, D-30)
- Files authored: 30 (2026-05-19, all PLANNED)
- Total SCHEMA_VERSION bumps anticipated: 4-6 (atomic per wave)
- Total new SsbActionType variants anticipated: ~35-40 across waves 1-4
- Estimated effort (27 CPU/cloud sessions): ~160-220 h
- Estimated effort (3 deferred sessions): ~25-75 h + RunPod

## Cross-plan reconciliation (D → B → C)

Each D-N file's WIRING_CONTRACT consumes_symbols block names every C-N or B-N artifact it depends on. The spec-grounder agent verifies these references against the canonical SCHEMA_VERSION and SsbActionType enum at EXECUTING entry. Any missing dependency (e.g. C-27 not shipped before D-9 starts) BLOCKS the FSM transition out of PLANNING.

The bn_status_report.py harness tool (`.claude/tools/bn_status_report.py`) is extended in D-26 with --plan=D filter so the dashboard surfaces D-plan progress alongside B-plan + C-plan.

---

End of D-0_INDEX.md
