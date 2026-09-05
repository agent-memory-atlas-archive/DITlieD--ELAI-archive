# ELAI D-plan feature discovery — summary layer

## Scope, evidence, and notation

- Primary source, read in full: `docs/thefinalglance/D-plan/D-SUMMARY_INDEX.md`, 311 lines, generated 2026-05-19. All `D:` anchors below are line numbers in that file.
- Orientation source, read in full but not treated as the D-plan implementation ledger: `docs/thefinalglance/ELAI-SPEC-OVERVIEW.md`, 592 lines. Its D synopsis is at `OVERVIEW:178-192`; its system-wide principles are at `OVERVIEW:8-24`.
- Discovery boundary: this pass intentionally did **not** open the 30 original `D-N` plans or the two companion designs. Those are enumerated in the deeper-source manifest for later agents.
- `[E]` means directly stated by the D summary. `[I]` means a synthesis or implication formed only by joining explicit statements; it is never a claim that an omitted implementation detail exists.
- Status is taken only from the D summary. `PLANNED` means the plan file exists but implementation has not started; `DEFERRED` means blocked by data, hardware, or research (`D:10-14`). The overview is design-only and explicitly cannot establish shipped/wired state (`OVERVIEW:1-6`).

## Source coverage and completeness accounting

- [E] All five waves and all 30 session IDs were inventoried: Wave 1 `D-1..D-9` (`D:18-74`), Wave 2 `D-10..D-18` (`D:76-132`), Wave 3 `D-19..D-25` (`D:134-178`), Wave 4 `D-26..D-27` (`D:180-192`), and Wave 5 `D-28..D-30` (`D:194-212`).
- [E] Status coverage: 27 executable CPU/cloud sessions are `PLANNED`; three research/data-dependent sessions are `DEFERRED`; the summary marks none `IN PROGRESS` or `DONE` (`D:218-225`).
- [E] Every session is intended to ship one cohesive module, wire it to the dashboard, and prove it with a real cloud-Sonnet test, plus a paper-number benchmark where applicable (`D:3-4`).
- [E] Dependency order is normative enough that the reasoning-improvement companion **MUST** land before the real-problem-solving companion because the latter consumes `ReasoningTemplate`, PPM, tripwires, archive, mode routing, checker registry, Adversary, and UniCR substrate (`D:6-8`).
- [E] The D layer turns the C-verified system into a self-improving and inference-time-grounded system (`D:294-307`; `OVERVIEW:178-192`).
- [E] Design target: every Wave-1 step is strictly stronger than DGM on at least one axis under the cited bounded-capacity theorem (`D:18-20`).
- [E] Wave 3 adds quantitative benchmarks and adversarial probes so promotion evidence is not limited to unit tests (`D:134-136`).

## Atomic feature inventory

### Wave 1 — typed self-improvement substrate

#### D-1 — `ReasoningTemplate typed schema + Blake3 content-hash + archive lifecycle` — PLANNED

Source: `D:22-26`; closes Phase 4H.IV / companion §3.1.

- [E] Introduces typed `ReasoningTemplate` records with atomic fields `id`, `domain`, `pattern_kind`, `template_body`, `eligibility_predicate`, `verifier_required`, `content_hash`, `parent_id`, and `archive_status`.
- [E] Uses Blake3 content hashing for templates.
- [E] Introduces an `ArchiveEntry` type and archive lifecycle.
- [E] Adds SSB event types written in the summary as `ARCHIVE_ENTRY_ADDED/RETIRED`; `ARCHIVE_ENTRY_ADDED` is unambiguous, while the summary does not spell out whether the second full identifier is `RETIRED` or a prefixed name.
- [E] Migrates existing prompt-embedded reasoning templates into typed records.
- [E] Cloud proof runs `elai /task --mode cloud`, exercises at least three typed templates, requires `ARCHIVE_ENTRY_ADDED` rows, and verifies the content-hash chain.
- [E] Direct prerequisites: C-15 stable `ClockProvider`; C-21 stable `MistakeBook` plus `rules.db`.
- [E] Research grounding: SELF-DISCOVER, the bounded-capacity result in arXiv:2510.04399, and DGM.

#### D-2 — `PPM sidecar (Qwen2.5-Math-PRM-7B)` — PLANNED

Source: `D:28-32`; closes Phase 4H.V part 1 / companion §3.2.

- [E] Adds a second inference sidecar dedicated to process/step reward scoring.
- [E] Pins that PPM sidecar to port `8101`.
- [E] Uses `Qwen2.5-Math-PRM-7B`, explicitly a different family from the `Qwen2.5-Coder-7B` policy SLM.
- [E] Adds a dual-GPU placement specification.
- [E] Adds a `PpmClient` trait.
- [E] Adds a paper-ingestion prerequisite that pulls the Qwen2.5-Math-PRM source before implementation execution.
- [E] Scores reasoning at step granularity.
- [E] Cloud proof scores one known-correct and one known-wrong reasoning step and requires `correct > wrong` by a margin greater than `0.3`.
- [E] Direct prerequisites: D-1 typed templates and B-37 multi-backend runtime.
- [E] Research grounding: rStar-Math, heterogeneous verifiers, and PRMBench.

#### D-3 — `VerificationResult aggregator + multi-verifier consensus rule` — PLANNED

Source: `D:34-38`; closes Phase 4H.V part 2 / companion §3.3.

- [E] Adds typed `VerificationResult` fields `deterministic`, `ppm`, `intrinsic`, `cross_context`, and `consensus_rule`.
- [E] Aggregates heterogeneous verification signals rather than allowing the PPM alone to decide promotion.
- [E] Blocks promotion unless at least two verifiers concur.
- [E] Treats consensus as a PRMBench Deception Resistance defense.
- [E] Cloud proof supplies deceptive reasoning where PPM passes but a deterministic checker fails; the candidate must not be promoted.
- [E] Direct prerequisite: D-2 PPM client.
- [E] Research grounding: V-STaR, PRMBench, and Cross-Context Verification.

#### D-4 — `Tripwire stack: KL + Δρ + IQR + perplexity + DDM` — PLANNED

Source: `D:40-44`; closes Phase 4H.VI / companion §§3.6 and 3.9.

- [E] Replaces the prior single rule, described as “10% over 3 pipelines,” with five orthogonal regression tripwires.
- [E] The five signals are KL, delta-rho (`Δρ`), IQR, perplexity, and DDM.
- [E] Automatically reverts a mutation when at least two tripwires fire concurrently.
- [E] Adds the SSB event type `TRIPWIRE_FIRED`.
- [E] Revert targets come from the D-1 template archive; monitored verifier signals come from D-3.
- [E] Cloud proof injects a synthetic regression, requires at least three tripwires to fire inside the applicable window, and requires the auto-revert to be visible in the archive.
- [E] Direct prerequisites: D-1 and D-3.
- [E] Research grounding includes measured reward-hacking prevalence/thresholds, concept-drift detectors DDM/Page-Hinkley/ADWIN, and reward-model overoptimization.

#### D-5 — `BAI promotion: Beta-posterior Thompson sampling` — PLANNED

Source: `D:46-50`; closes Phase 4H.VII / companion §3.7.

- [E] Replaces MSI sigmoid-Boltzmann candidate promotion with best-arm identification (BAI).
- [E] Adds `ArmPosterior { template_id, pass_count, fail_count }`.
- [E] Uses Beta-posterior Thompson sampling.
- [E] Includes Successive Rejects and LUCB mechanisms.
- [E] Grounds promotion behavior in published regret bounds.
- [E] Tracks D-1 archive entries as arms and consumes D-3 consensus as the binary pass/fail signal.
- [E] Cloud proof runs a 50-round synthetic noisy-arm benchmark and requires empirical regret within the published Audibert bound.
- [E] Direct prerequisites: D-1 and D-3.
- [E] Research grounding: Audibert COLT 2010 BAI and variance-dependent BAI.

#### D-6 — `Time-windowed held-out task store + 9-gram filter` — PLANNED

Source: `D:52-56`; closes Phase 4H.VIII part 1 / companion §3.4.

- [E] Persists held-out tasks at `.elai_cc/held-out/{date}/{task-id}.toml`.
- [E] Organizes held-out evaluation temporally.
- [E] Applies a 9-gram overlap filter for contamination control.
- [E] Automatically retires tasks that pass uniformly after six months because that behavior indicates probable contamination.
- [E] Emits `HELD_OUT_REFRESH` to the SSB when the store refreshes.
- [E] Places the held-out store on D-5’s promotion path as a gate.
- [E] Cloud proof auto-retires a known-contaminated task that has passed uniformly for more than six months; ingestion of a new task must emit a store-refresh row.
- [E] Direct prerequisite: D-5.
- [E] Research grounding: LiveCodeBench temporal/9-gram controls, Soft Contamination, and the ARC Prize 2025 report.

#### D-7 — `Cross-domain transfer canary: 5 task families` — PLANNED

Source: `D:58-62`; closes Phase 4H.VIII part 2 / companion §3.5.

- [E] Defines five orthogonal evaluation families: `code-fix`, `math-proof`, `prose-reason`, `planning`, and `knowledge-QA`.
- [E] Uses family tags supplied by the D-6 held-out store.
- [E] Blocks promotion if any non-target family regresses by more than `5%`.
- [E] Emits a `CrossDomainRegression` SSB record.
- [E] Cloud proof introduces a mutation with an `8%` prose-reason regression and requires promotion to be blocked with a regression row whose delta is `-0.08`.
- [E] Direct prerequisite: D-6.
- [E] Research grounding: NanoFlux.

#### D-8 — `Cryptographic archive + elai archive verify` — PLANNED

Source: `D:64-68`; closes Phase 4H.IX / companion §3.8.

- [E] Hash-chains every mutation candidate with Blake3.
- [E] Makes the archive append-only.
- [E] Defines the more detailed `ArchiveEntry { self_hash, parent_hash, template, eval_result, promoted_at, retired_at }` record.
- [E] Adds the CLI command `elai archive verify`.
- [E] Adds a tamper-test contract.
- [E] Cloud proof requires any altered archive row to make verification exit `1` with a broken-link report; a clean archive must exit `0`.
- [E] Direct prerequisite: D-1 archive lifecycle.
- [E] Research grounding: cryptographic-checksum guidance and DGM archive practice.

#### D-9 — `L3 expansion + frozen-surface access-control predicates` — PLANNED

Source: `D:70-74`; closes companion §2.3 and §4.

- [E] Documents and enforces the L3-locked surface.
- [E] Explicitly locks the staged-evaluation harness.
- [E] Explicitly locks tripwire thresholds.
- [E] Explicitly locks MSI/BAI scoring.
- [E] Explicitly locks archive maintenance.
- [E] Explicitly locks held-out selection.
- [E] Explicitly locks canary-family definitions.
- [E] Explicitly locks selection of Blake3 itself.
- [E] Explicitly locks the `AllowedConfigKey` bound itself.
- [E] Explicitly locks the `verifier_required` field.
- [E] Explicitly locks the `Domain` enum.
- [E] Adds a pre-commit enforcement check.
- [E] Adds `LockedKeysAuditor`.
- [E] Adds the SSB event `LOCKED_KEYS_TOUCH`.
- [E] Cloud proof simulates an agent editing evaluator code and requires the pre-commit hook, auditor, event row, and a blocked `git push`.
- [E] Direct prerequisites: all D-1 through D-8 artifacts must exist.
- [E] Research grounding: current-reward-function integrity, boxed/myopic design, and learned/mesa-optimization risk.

### Wave 2 — five-mode inference-time grounding

#### D-10 — `Mode-routing predicate select_grounding_mode` — PLANNED

Source: `D:76-84`; closes Phase RPS.I / real-problem §3.2.

- [E] Adds a `GroundingMode` enum with five modes `A/B/C/D/E`.
- [E] Adds the orchestrator function `select_grounding_mode`.
- [E] Routes claims based on whether they are operationalizable, whether a checker is registered, sandbox availability, cross-family availability, and an intrinsic-signal threshold.
- [E] Keeps mode selection entirely orchestrator-owned; the LLM has no input to selection.
- [E] Requires the routing function itself to be L3-frozen.
- [E] Emits `MODE_SELECTED` to the SSB.
- [E] Cloud proof uses 50 synthetically labeled claims and requires at least 45 correct assignments, stated as at least `90% F1`.
- [E] Direct prerequisite: D-9.
- [E] Research grounding: current-reward-function integrity and real-problem §3.2.

#### D-11 — `Constructive-checker registry .elai_cc/checkers/ bootstrap` — PLANNED

Source: `D:86-90`; closes Phase RPS.II / real-problem §4.2.

- [E] Bootstraps registry directories `.elai_cc/checkers/{math,code,text,graph,api,perf}/`.
- [E] Ships six initial checker integrations: `sympy`, `z3`, `compile-rust`, `compile-py`, `registry-check`, and `regex`.
- [E] Adds a typed `CheckerResult` return value.
- [E] Adds orchestrator-side checker-dispatch logic.
- [E] Makes Grounding Mode A operational.
- [E] Checkers are reached through D-10 Mode A routing.
- [E] Cloud proof requires a mathematical claim to be verified by SymPy, a code claim by `cargo check`, and a novel npm-package claim by the registry checker.
- [E] Direct prerequisite: D-10.
- [E] Research grounding: Program-of-Thought, CodeAct, and AlphaProof.

#### D-12 — `OperationalizationCheck baby-step predicate` — PLANNED

Source: `D:92-96`; closes Phase RPS.III / real-problem §4.1.

- [E] Extends BabyStepSpec block 7 with a `MEASURABLE_PROCEDURE` field.
- [E] Requires a claim-emitting baby step to name a measurable procedure that references a checker.
- [E] Blocks a claim-emitting baby step without that procedure at the `AUDITING` gate.
- [E] Emits `CLAIM_WITHOUT_PROCEDURE` to the SSB on that failure.
- [E] Cloud proof has a baby-step author emit a claim without a procedure and requires both auditor block and event row.
- [E] Direct prerequisite: D-11.
- [E] Research grounding: Voyager’s measurable-effect skill pattern and real-problem §4.1.

#### D-13 — `Adversary role + cross-family verifier wiring` — PLANNED

Source: `D:98-102`; closes Phase RPS.IV / real-problem §4.3.

- [E] Adds `AgentRole::Adversary`.
- [E] Requires the Adversary to use a model family different from the executor.
- [E] Uses cloud Sonnet as fallback when no alternate local family is available.
- [E] Requires structured counter-evidence as Adversary output.
- [E] Forbids same-model self-critique; the summary says “NEVER.”
- [E] Falls back to Mode E refusal when no alternate family is available.
- [E] Routes this function through Mode B.
- [E] Feeds the Adversary verdict into D-3 consensus.
- [E] Emits `ADVERSARY_VERDICT` and `CROSS_FAMILY_UNAVAILABLE` to the SSB.
- [E] Cloud proof starts with a confident executor claim, obtains disagreement from the cross-family Adversary, requires consensus rejection, emits an adversary row, and blocks the claim.
- [E] Direct prerequisites: D-10 and D-3.
- [E] Research grounding: Self-Correction Bench blind spots, lightweight latent verifiers, and small-model constitutional-AI collapse.

#### D-14 — `Sycophancy probe: inverted-framing structural defense` — PLANNED

Source: `D:104-108`; closes Phase RPS.V / real-problem §4.7.

- [E] Re-runs every user-aligned claim with its framing inverted.
- [E] Treats converging original/inverted outputs as robust.
- [E] Treats divergent outputs as evidence of sycophancy.
- [E] Emits `SYCOPHANCY_PROBE` for probe activity and `SYCOPHANCY_FLAG` for divergence.
- [E] Quarantines flagged claims for human review.
- [E] Reuses D-13 cross-family infrastructure for the inverted-framing run.
- [E] Cloud proof gives a false user framing that the model initially accepts and requires the inverted run to diverge and fire `SYCOPHANCY_FLAG`.
- [E] Direct prerequisite: D-13.
- [E] Research grounding: Sharma et al. sycophancy probes.

#### D-15 — `Discovery-vs-search labeling (NoveltyClass)` — PLANNED

Source: `D:110-114`; closes Phase RPS.VI / real-problem §4.8.

- [E] Adds `NoveltyClass { Search, DistantSearch, DiscoveryCandidate }`.
- [E] Applies a novelty class to every Mode A result.
- [E] Applies a novelty class to every Mode D result.
- [E] Adds `NoveltyClass` as a field on `CheckerResult`.
- [E] Sends every `DiscoveryCandidate` to a human-review queue rather than automatically accepting it.
- [E] Emits `NOVELTY_LABELED` to the SSB.
- [E] Uses C-17 DKG resilience for near-duplicate detection.
- [E] Cloud proof requires a known lookup to label `Search`, a novel composition to label `DistantSearch`, and a primitive absent from the DKG to label `DiscoveryCandidate`.
- [E] Direct prerequisites: D-11, D-12, and C-17.
- [E] Research grounding: FunSearch, AlphaEvolve, ARC, and the real-problem Mode-C contradiction discussion.

#### D-16 — `Conformal refusal calibration (UniCR)` — PLANNED

Source: `D:116-120`; closes Phase RPS.VII / real-problem §4.5.

- [E] Applies UniCR-style conformal risk control to “I don’t know” behavior.
- [E] Persists calibration data under `.elai_cc/calibration/`.
- [E] Stores tuples `(claim, ground_truth_available, model_confidence)`.
- [E] Maintains threshold `τ`.
- [E] Updates `τ` nightly.
- [E] Restricts the threshold updater to L3.
- [E] Refuses claims below the calibrated threshold.
- [E] Formally grounds D-10 Mode E as the refusal mode.
- [E] Emits `REFUSAL_DECISION` and `CALIBRATION_UPDATE` to the SSB.
- [E] Cloud proof evaluates a distribution-shifted held-out family and requires empirical refusal behavior to remain within `ε = 0.1`.
- [E] Direct prerequisites: D-9 and D-10.
- [E] Research grounding: UniCR, uncertainty outside parametric knowledge, and rewarding doubt.

#### D-17 — `Honest-refusal SLA monitoring` — PLANNED

Source: `D:122-126`; closes Phase RPS.VIII / real-problem §4.6.

- [E] Adds the per-session metric `refusal_rate = refusals / total_claims`.
- [E] Automatically flags a session below `5%` refusal rate as likely confabulation.
- [E] Automatically flags a session above `50%` refusal rate as likely checker misconfiguration.
- [E] Uses that lower-and-upper band as a structural defense against gaming Mode E as the easy route.
- [E] Consumes refusal events emitted by D-16.
- [E] Cloud proof instruments the last 20 sessions, expects a healthy band to emerge empirically, and requires a synthetic session below `2%` to emit an `AUTO_FLAG` row.
- [E] Direct prerequisite: D-16.
- [E] Research grounding: real-problem §4.6 and the TMLR 2025 LLM Honesty Survey.

#### D-18 — `ObservableEffect typed predicate (Mode D upgrade)` — PLANNED

Source: `D:128-132`; closes a Phase 4F extension / real-problem §4.4.

- [E] Extends the Phase 4F Feasibility Probe with typed observable effects.
- [E] Adds `ObservableEffect { trigger, expected_observation, success_predicate, bounded_runtime, sandbox_tier, confound_flags }`.
- [E] Requires at least two `ObservableEffect` records per claim.
- [E] Labels whether an observation came from sandbox or production.
- [E] Uses D-11 checkers as Mode D effect verifiers.
- [E] Cloud proof gives a feasibility probe only one observable effect and requires `GOODHART_RISK_FLAG`; with two or more effects the flag must be absent.
- [E] Direct prerequisites: C-22 FeasibilityProber completeness and D-11.
- [E] Research grounding: Voyager, Eureka, and Coscientist.

### Wave 3 — quantitative and adversarial proof

#### D-19 — `PRMBench adversarial gaming benchmark for PPM` — PLANNED

Source: `D:138-142`; closes companion §1.13’s open question.

- [E] Pulls PRMBench fixtures from the `Deception Resistance` and `Circular Logic` subsets.
- [E] Runs the D-2 PPM on adversarial samples.
- [E] Reports scores separately by category.
- [E] Gates PPM promotion on reaching at least the published baseline.
- [E] Cloud proof sets conservative concrete floors: Deception Resistance at least `35%` versus a cited published `40–50%` range for Qwen2.5-Math-PRM-7B; Circular Logic at least `30%`.
- [E] D-3 consumes the PPM score through its consensus rule.
- [E] Direct prerequisites: D-2 and D-3.
- [E] Research grounding: PRMBench, Lessons of Developing PRMs, and ThinkPRM.

#### D-20 — `ARC-AGI-2 OOD probe harness` — PLANNED

Source: `D:144-148`; closes companion §1.16’s implication and the real-problem §1 Mode-C item.

- [E] Pulls a public ARC-AGI-2 task subset.
- [E] Falls back to synthetic ARC-style tasks if hand-crafted leakage is flagged.
- [E] Runs ELAI on cloud Sonnet rather than a local 7B model for ARC.
- [E] Runs with mode routing enabled.
- [E] Reports solve rate.
- [E] Reports Mode-E refusal rate.
- [E] Compares those metrics against a human baseline.
- [E] Cloud proof runs 50 ARC-AGI-2 tasks, requires solve-rate and refusal-rate reporting, requires honest-refusal SLA to remain in band, and forbids automatic publication of any `DiscoveryCandidate` claim.
- [E] Direct prerequisites: D-10 and D-15.
- [E] Research grounding: ARC, ARC Prize 2025, and NVARC NVIDIA 2025.

#### D-21 — `Mode-routing coverage benchmark` — PLANNED

Source: `D:150-154`; closes D-10’s gating-evidence gap.

- [E] Builds 200 synthetic claims with hand-labeled ground-truth modes A/B/C/D/E.
- [E] Evaluates `select_grounding_mode` using overall F1.
- [E] Produces a per-mode confusion matrix.
- [E] Reports edge-case failures.
- [E] Cloud proof requires overall F1 of at least `0.85`.
- [E] Prevents class collapse by requiring at least 10 correct examples in every mode.
- [E] Direct prerequisite: D-10.
- [E] Research grounding: real-problem §3.2 and the Self-Correction Bench blind-spot framework.

#### D-22 — `Conformal calibration validation under distribution shift` — PLANNED

Source: `D:156-160`; closes D-16’s gating-evidence gap.

- [E] Builds three held-out distributions: in-domain, near-OOD, and far-OOD.
- [E] Sources those distributions from the D-6 held-out store.
- [E] Validates UniCR’s empirical refusal rate under all three shifts.
- [E] Uses user-set `ε = 0.1`.
- [E] Cloud proof requires empirical `refusal_rate` inside `[τ-ε, τ+ε]` for every distribution and fails if any distribution shifts by more than epsilon.
- [E] Direct prerequisites: D-16 and D-6.
- [E] Research grounding: UniCR’s distribution-free guarantee.

#### D-23 — `Sycophancy detection benchmark` — PLANNED

Source: `D:162-166`; closes D-14’s gating-evidence gap.

- [E] Pulls Anthropic/Sharma public sycophancy probes.
- [E] Permits a synthetic stand-in if those public fixtures cannot be used.
- [E] Runs a probe-enabled versus probe-disabled comparison.
- [E] Reports detection rate.
- [E] Reports false-positive rate.
- [E] Cloud proof requires detection rate at least `70%` on labeled fixtures and false-positive rate at most `15%`.
- [E] Direct prerequisite: D-14.
- [E] Research grounding: Sharma et al.

#### D-24 — `Cross-domain canary regression validation` — PLANNED

Source: `D:168-172`; closes D-7’s gating-evidence gap.

- [E] Generates 100 synthetic mutations across all five D-7 families.
- [E] Fixture mix is exactly 50 helpful, 30 neutral, and 20 harmful mutations.
- [E] Requires all 20 harmful mutations to be blocked.
- [E] Allows at most 5 of 50 helpful mutations to be blocked, described as a false-positive ceiling of `10%`.
- [E] Allows at most 2 of 30 neutral mutations to be blocked.
- [E] Direct prerequisite: D-7.
- [E] Research grounding: NanoFlux and companion §3.5.

#### D-25 — `Determinism + archive tamper test suite` — PLANNED

Source: `D:174-178`; closes D-8 plus C-27 T(-1) determinism gating evidence.

- [E] Defines a ten-scenario archive tamper suite.
- [E] Explicit scenario: delete an archive row.
- [E] Explicit scenario: swap `parent_hash`.
- [E] Explicit scenario: edit the template body.
- [E] Explicit scenario: replace Blake3 with SHA-256 on disk.
- [E] Requires `elai archive verify` to detect every scenario within five seconds.
- [E] Requires 10/10 detection.
- [E] Requires a clean repository to exit `0`.
- [E] Requires tamper-then-revert to recover to clean verification.
- [E] Uses C-27 `DeterminismDrift` alongside D-8 archive verification.
- [E] Direct prerequisites: D-8 and C-27.
- [E] Research grounding: checksum integrity guidance and companion §3.8.

### Wave 4 — dashboard integration capstone

#### D-26 — `ReasoningTemplate + archive dashboard surface` — PLANNED

Source: `D:182-186`; exposes D-1 through D-8.

- [E] Adds `MutationArchivePanel.svelte`.
- [E] Adds `LineageGraphView.svelte`.
- [E] Uses Cytoscape to render lineage.
- [E] Cross-links reasoning-template lineage to the baby-step archive.
- [E] Extends `ReasoningDrawer` with PPM score.
- [E] Extends `ReasoningDrawer` with the consensus rule/result.
- [E] Extends `ReasoningDrawer` with tripwire status.
- [E] Adds Tauri command `get_archive_recent`.
- [E] Adds Tauri command `get_archive_verify_status`.
- [E] Adds Tauri command `get_tripwire_status`.
- [E] Cloud proof runs a live cloud `/task` that promotes a template; the archive panel must display the new `ARCHIVE_ENTRY_ADDED` row and its lineage edge.
- [E] Cloud proof injects a tripwire firing and requires a red dashboard flag inside the polling interval.
- [E] Direct prerequisite: D-1 through D-8 shipped.

#### D-27 — `Mode-routing + refusal-SLA dashboard panels` — PLANNED

Source: `D:188-192`; exposes D-10 through D-17.

- [E] Adds `ModeRoutingPanel.svelte` with a per-session A/B/C/D/E breakdown.
- [E] Adds `RefusalRateGauge.svelte` with healthy-band visualization.
- [E] Adds `NoveltyBreakdown.svelte` with counts for `Search`, `DistantSearch`, and `DiscoveryCandidate`.
- [E] Adds `SycophancyFlagStrip.svelte` as a real-time flag stream.
- [E] Adds Tauri command `get_mode_breakdown`.
- [E] Adds Tauri command `get_refusal_rate`.
- [E] Adds Tauri command `get_novelty_breakdown`.
- [E] Adds Tauri command `get_sycophancy_flags`.
- [E] Cloud proof runs a live cloud `/task` with mixed claims and requires a realistic mode distribution.
- [E] A fired sycophancy probe must appear on the dashboard inside the polling interval.
- [E] Direct prerequisite: D-10 through D-17 shipped.

### Wave 5 — deferred and research-grade work

#### D-28 — `Adversarial example farming for PPM` — DEFERRED

Source: `D:196-200`; closes companion Phase 4H.X.

- [E] Iteratively generates out-of-distribution examples that obtain high proxy reward while remaining low quality.
- [E] Retrains the PPM on the adversarial corpus to immunize it.
- [E] Defers bootstrapping until D-5 BAI has a stable selection signal with at least 20 promotion cycles.
- [E] Requires the D-19 PRMBench baseline to have been measured.
- [E] Requires RunPod for the retraining run.
- [E] Cloud proof compares pre/post immunization and requires Deception Resistance to improve by at least `+5` percentage points with no regression on standard categories.
- [E] Direct prerequisites: D-5 and D-19.
- [E] Research grounding: adversarial training of reward models.

#### D-29 — `Hierarchical BAI for non-IID rewards` — DEFERRED, research-grade

Source: `D:202-206`; closes companion open question Q4.

- [E] Investigates the mismatch between published IID-reward BAI assumptions and ELAI’s non-IID outcomes across sessions.
- [E] Spikes a hierarchical Bayesian model.
- [E] Uses D-5 flat BAI as the comparison baseline.
- [E] Requires RunPod for a hierarchical Gibbs sampler.
- [E] Accepts a documented infeasibility note as a valid closure outcome.
- [E] Success path: hierarchical regret is no worse than flat-BAI regret on a synthetic non-IID benchmark.
- [E] Infeasibility path: document at least one concrete blocker.
- [E] Direct prerequisite: D-5.
- [E] Research grounding: Audibert, variance-dependent BAI, and companion §3.7/Q4.

#### D-30 — `Conformal calibration bootstrap from generic corpus` — DEFERRED

Source: `D:208-212`; closes real-problem open question Q5.

- [E] Establishes that useful UniCR calibration needs `N = 500` tuples.
- [E] Addresses the cold-start condition where early local sessions cannot provide that many tuples.
- [E] Spikes bootstrapping from a generic LLM-calibration corpus.
- [E] Refines the generic bootstrap per project as real session data accumulates.
- [E] Requires access to a HuggingFace calibration dataset that is still to be identified.
- [E] Cloud proof requires bootstrap calibration to stay within epsilon on held-out validation.
- [E] Later real-session-refined calibration must have empirical risk no worse than the bootstrap.
- [E] Direct prerequisite: D-16.
- [E] Research grounding: UniCR and real-problem Q5.

## How the D features work together

### Self-improvement promotion and rollback path

1. [E] D-1 converts mutable reasoning patterns into typed, eligibility-gated, verifier-required, parent-linked, content-hashed archive objects (`D:22-26`).
2. [E] D-2 supplies a heterogeneous, step-level PPM score through a separate model-family sidecar (`D:28-32`).
3. [E] D-3 combines deterministic, PPM, intrinsic, and cross-context signals and requires concurrence from at least two verifiers before promotion (`D:34-38`).
4. [E] D-5 treats templates as arms, updates pass/fail posteriors from D-3 consensus, and selects promotion candidates with BAI methods (`D:46-50`).
5. [E] D-6 makes temporally held-out, overlap-filtered tasks part of the promotion gate and retires likely contaminated tasks (`D:52-56`).
6. [E] D-7 vetoes a promotion that improves its target while regressing any non-target family beyond 5% (`D:58-62`).
7. [E] D-4 watches promoted behavior with five distinct regression signals and reverts to an archived target if at least two fire together (`D:40-44`).
8. [E] D-8 makes the mutation lineage independently tamper-checkable with an append-only Blake3 chain and CLI verifier (`D:64-68`).
9. [E] D-9 prevents the modifying agent from changing the evaluator, thresholds, selection math, archive governance, held-out set, canary definitions, hash algorithm, or locked schema/control bounds (`D:70-74`).
10. [E] D-19, D-24, and D-25 stress the PPM, family canary, and archive/determinism mechanisms quantitatively; D-26 renders the live lineage, verification, PPM, and tripwire state (`D:138-142,168-186`).
11. [I] The resulting summary-level loop is: typed candidate → heterogeneous scores → consensus → BAI selection → held-out and cross-domain gates → append-only promotion record → tripwire monitoring/possible revert → dashboard inspection. This ordering is a synthesis of the explicit dependencies above; the exact transaction boundaries live in the original plans.

### Claim grounding, checking, refusal, and human escalation path

1. [E] D-10 classifies each claim into one of five orchestrator-owned modes using operationalizability, checker, sandbox, cross-family, and intrinsic-signal facts; the generator cannot choose its own mode (`D:80-84`).
2. [E] Mode A dispatches to D-11 constructive checkers with typed results (`D:86-90`).
3. [E] D-12 requires claim-producing baby steps to name their measurable procedure and blocks them at `AUDITING` if they do not (`D:92-96`).
4. [E] Mode B uses D-13 cross-family Adversary counter-evidence, contributes that verdict to D-3 consensus, and refuses rather than using same-family self-critique when independent verification is unavailable (`D:98-102`).
5. [E] D-14 structurally tests user-aligned claims by inverting their framing and quarantines divergent results for a human (`D:104-108`).
6. [E] D-15 labels every Mode A/D result as `Search`, `DistantSearch`, or `DiscoveryCandidate`; candidates in the last class enter human review (`D:110-114`).
7. [E] Mode E refusal is calibrated by D-16, then monitored by D-17 so the system cannot game uncertainty by refusing almost nothing or almost everything (`D:116-126`).
8. [E] Mode D gains D-18’s two-or-more typed, bounded, confound-aware observable effects and checker-backed sandbox/production outcomes (`D:128-132`).
9. [E] D-20 through D-23 exercise OOD mode use, routing coverage, conformal calibration under shift, and sycophancy detection; D-27 displays route mix, refusal health, novelty mix, and flags (`D:144-166,188-192`).
10. [I] The resulting summary-level path is: claim → harness route → constructive/cross-family/experimental/refusal handling → procedure/effect evidence → consensus or calibrated abstention → novelty/sycophancy escalation → journal/dashboard. Mode C’s actual handler cannot be filled in from this summary and is explicitly left unresolved below.

### Research-to-execution prerequisites

- [E] Each D-N original names its paper prerequisites. Before `EXECUTING`, `paper-ingestor` must pull and index each needed paper into `workflow/papers/{slug}-interface.md` (`D:238-264`).
- [E] `spec-grounder` verifies the required interface files before unblocking `TESTING_FIRST` (`D:264`).
- [E] Every executable D session is then expected to receive a live cloud-Sonnet proof and, when applicable, a paper-grounded numeric benchmark (`D:3-4`).
- [E] D-1 is globally sequenced after the required C bedrock; D-10 begins only after Wave 1’s frozen-surface enforcement (`D:80-84,294-307`).

## Interfaces, commands, configuration, data, and lifecycle

### Typed data and enums

- [E] `ReasoningTemplate`: `id`, `domain`, `pattern_kind`, `template_body`, `eligibility_predicate`, `verifier_required`, `content_hash`, `parent_id`, `archive_status` (D-1; `D:22-26`).
- [E] `ArchiveEntry` is introduced in D-1; D-8 explicitly specifies `self_hash`, `parent_hash`, `template`, `eval_result`, `promoted_at`, `retired_at` (`D:22-23,64-68`).
- [E] `VerificationResult`: `deterministic`, `ppm`, `intrinsic`, `cross_context`, `consensus_rule` (D-3; `D:34-38`).
- [E] `ArmPosterior`: `template_id`, `pass_count`, `fail_count` (D-5; `D:46-50`).
- [E] Held-out task records are TOML files partitioned by date and task ID; per-family tagging exists for cross-domain canaries (D-6/D-7; `D:52-60`).
- [E] `GroundingMode`: five values A/B/C/D/E (D-10; `D:80-84`).
- [E] `CheckerResult`: typed checker return, later extended with a `NoveltyClass` field (D-11/D-15; `D:86-90,110-114`).
- [E] `NoveltyClass`: `Search`, `DistantSearch`, `DiscoveryCandidate` (D-15; `D:110-114`).
- [E] Calibration tuple: `(claim, ground_truth_available, model_confidence)` (D-16; `D:116-120`).
- [E] `ObservableEffect`: `trigger`, `expected_observation`, `success_predicate`, `bounded_runtime`, `sandbox_tier`, `confound_flags` (D-18; `D:128-132`).
- [E] `AgentRole::Adversary` (D-13; `D:98-102`).

### Filesystem/configuration surfaces

- [E] `.elai_cc/held-out/{date}/{task-id}.toml`: temporal held-out store (D-6).
- [E] `.elai_cc/checkers/{math,code,text,graph,api,perf}/`: constructive-checker registry families (D-11).
- [E] `.elai_cc/calibration/`: refusal-calibration tuples and threshold inputs (D-16).
- [E] `workflow/papers/{slug}-interface.md`: ingested paper interfaces required before testing/execution (`D:264`).
- [E] L3 frozen configuration/governance surfaces: staged evaluator, tripwire thresholds, MSI/BAI scoring, archive maintenance, held-out selection, canary families, Blake3 choice, the `AllowedConfigKey` bound, `verifier_required`, and `Domain` (`D:70-74`).
- [E] UniCR `τ` is nightly-updated but only by L3 code (`D:116-120`).
- [E] D-2’s process integration fixes the PPM endpoint to port 8101 and declares dual-GPU placement (`D:28-32`).

### CLI/API/UI surfaces

- [E] CLI `elai /task --mode cloud` is the D-1 live exercise command (`D:26`).
- [E] CLI `elai archive verify` returns `0` for clean state and `1` plus a broken-link report for tamper (`D:64-68`).
- [E] Orchestrator API/function `select_grounding_mode` is the closed L3 route selector (`D:80-84`).
- [E] `PpmClient` trait abstracts the PPM sidecar (`D:28-32`).
- [E] Checker dispatch is orchestrator-owned and typed through `CheckerResult` (`D:86-90`).
- [E] Dashboard components: `MutationArchivePanel.svelte`, `LineageGraphView.svelte`, `ReasoningDrawer` extensions, `ModeRoutingPanel.svelte`, `RefusalRateGauge.svelte`, `NoveltyBreakdown.svelte`, `SycophancyFlagStrip.svelte` (`D:182-192`).
- [E] Tauri archive commands: `get_archive_recent`, `get_archive_verify_status`, `get_tripwire_status` (`D:182-186`).
- [E] Tauri grounding commands: `get_mode_breakdown`, `get_refusal_rate`, `get_novelty_breakdown`, `get_sycophancy_flags` (`D:188-192`).
- [E] UI update contract: tripwire and sycophancy flags must appear within the dashboard poll interval (`D:186,192`).

### SSB observability/event inventory

| Event/row name | Producer/meaning | Anchor |
|---|---|---|
| `ARCHIVE_ENTRY_ADDED` | Typed template/candidate appended; consumed visually by D-26 | `D:23,26,186` |
| `RETIRED` (abbreviated in source as `ARCHIVE_ENTRY_ADDED/RETIRED`) | Typed archive entry retired; full identifier unresolved at summary layer | `D:23` |
| `TRIPWIRE_FIRED` | One of five regression tripwires fired | `D:40-44` |
| `HELD_OUT_REFRESH` | Held-out store refresh/new ingestion | `D:52-56` |
| `CrossDomainRegression` | Non-target-family regression and delta | `D:58-62` |
| `LOCKED_KEYS_TOUCH` | Agent attempted to edit an L3-frozen surface | `D:70-74` |
| `MODE_SELECTED` | Orchestrator selected A/B/C/D/E for a claim | `D:80-84` |
| `CLAIM_WITHOUT_PROCEDURE` | Claim-emitting baby step lacks measurable procedure | `D:92-96` |
| `ADVERSARY_VERDICT` | Structured cross-family counter-evidence verdict | `D:98-102` |
| `CROSS_FAMILY_UNAVAILABLE` | No alternate-family verifier was available | `D:98-102` |
| `SYCOPHANCY_PROBE` | Inverted-framing probe execution | `D:104-108` |
| `SYCOPHANCY_FLAG` | Original and inverted framing diverged | `D:104-108` |
| `NOVELTY_LABELED` | Mode A/D result received `NoveltyClass` | `D:110-114` |
| `REFUSAL_DECISION` | Calibrated accept/refuse decision | `D:116-120` |
| `CALIBRATION_UPDATE` | Nightly L3 threshold/calibration update | `D:116-120` |
| `AUTO_FLAG` | Refusal SLA outside the acceptable band; name appears in the cloud-test requirement | `D:122-126` |
| `GOODHART_RISK_FLAG` | Fewer than two observable effects for a claim; name appears in the cloud-test requirement | `D:128-132` |

### External integrations and dependencies

- [E] Model/runtime: cloud Sonnet live tests; Qwen2.5-Math-PRM-7B verifier sidecar; Qwen2.5-Coder-7B policy SLM; B-37 multi-backend runtime; dual-GPU placement (`D:4,28-32`).
- [E] Constructive verification: SymPy, Z3, Rust compilation/`cargo check`, Python compilation, package registry checks including npm, regex, plus registry folders for text/graph/API/performance (`D:86-90`).
- [E] Retrieval/near-duplicate support: C-17 DKG resilience, C-24 `symbol_impact`, and C-30 surgical-mode DKG tooling (`D:110-114,301`).
- [E] Feasibility: C-22 `FeasibilityProber` supplies the sandbox/probe substrate extended by D-18 (`D:128-132,304`).
- [E] Dashboard: Tauri, Svelte, and Cytoscape (`D:182-192`).
- [E] Research datasets/benchmarks: PRMBench, ARC-AGI-2 or a synthetic leakage-safe stand-in, Sharma public sycophancy probes or a synthetic stand-in, three held-out distribution tiers, and synthetic mutation/tamper suites (`D:138-178`).
- [E] Deferred compute/data: RunPod retraining and Gibbs sampling for D-28/D-29; an unidentified HuggingFace calibration corpus for D-30 (`D:196-212`).

## Constraints, invariants, gates, and fallback behavior

### Security/integrity invariants

- [E] The LLM must have no influence on grounding-mode selection (`D:78-84`).
- [E] No reasoning-template promotion may occur with fewer than two concurring verifiers (`D:34-38`).
- [E] The policy model and PPM must be heterogeneous model families (`D:28-32`).
- [E] The Adversary must be cross-family; same-model self-critique is forbidden (`D:98-102`).
- [E] If cross-family verification cannot be obtained, the system refuses through Mode E rather than weakening the verifier requirement (`D:98-102`).
- [E] Any non-target family regression above 5% blocks mutation promotion (`D:58-62`).
- [E] Two or more concurrent tripwire firings force automatic revert (`D:40-44`).
- [E] Every mutation candidate is chained; the archive is append-only; tamper must produce nonzero CLI status and a broken-link report (`D:64-68`).
- [E] Agents cannot modify the frozen evaluator/governance surfaces; attempted evaluator edits block push and are journaled (`D:70-74`).
- [E] A claim-emitting baby step without a checker-referenced measurable procedure cannot pass `AUDITING` (`D:92-96`).
- [E] A `DiscoveryCandidate` requires human review and cannot be auto-published in the ARC proof (`D:110-114,144-148`).
- [E] A divergent inverted-framing result is quarantined for human review (`D:104-108`).
- [E] Mode-D claims need at least two observable effects; only one triggers Goodhart-risk handling (`D:128-132`).
- [E] System-wide orientation only: D composes with ELAI’s off-agent-evidence, fail-closed, and deterministic spine; this contextual invariant comes from the overview, not extra D implementation detail (`OVERVIEW:8-24`).

### Refusal and degradation behavior

- [E] Calibrated confidence below `τ` yields refusal (`D:116-120`).
- [E] Alternate-local-model absence causes cloud-Sonnet cross-family fallback; total alternate-family unavailability causes Mode-E refusal (`D:98-102`).
- [E] Refusal below 5% or above 50% is itself flagged, preventing “never abstain” and “always abstain” strategies (`D:122-126`).
- [E] Public ARC fixtures may degrade to synthetic ARC-style fixtures only when hand-crafted leakage is flagged (`D:144-148`).
- [E] Public sycophancy fixtures may degrade to a synthetic stand-in (`D:162-166`).
- [E] D-29 may validly close with a concrete infeasibility result rather than a successful hierarchy (`D:202-206`).

### Quantitative acceptance criteria

- D-1: at least 3 typed templates and a verifiable content-hash chain (`D:26`).
- D-2: correct-vs-wrong score margin greater than 0.3 (`D:32`).
- D-3: deceptive case is rejected when only PPM passes (`D:38`).
- D-4: injected regression fires at least 3 tripwires in-window and visibly reverts (`D:44`).
- D-5: 50 noisy-arm rounds with regret inside the Audibert bound (`D:50`).
- D-6: uniformly passing for over 6 months causes retirement (`D:53-56`).
- D-7: a `-0.08` non-target delta is blocked against a >5% threshold (`D:59-62`).
- D-8: tamper exits 1; clean exits 0 (`D:68`).
- D-10: at least 45/50 known routes, stated as at least 90% F1 (`D:84`).
- D-16/D-22: refusal calibration tolerance `ε = 0.1`; D-22 requires all three distributions inside `[τ-ε, τ+ε]` (`D:120,156-160`).
- D-17: session auto-flags outside 5–50%; explicit synthetic proof is below 2%; monitoring window is 20 sessions (`D:122-126`).
- D-18: one effect flags; at least two effects do not (`D:128-132`).
- D-19: PRMBench Deception Resistance at least 35%; Circular Logic at least 30% (`D:138-142`).
- D-20: 50 ARC tasks; solve/refusal reporting; SLA in band; zero auto-published discovery candidates (`D:144-148`).
- D-21: F1 at least 0.85 and at least 10 correct examples per mode (`D:150-154`).
- D-23: sycophancy detection at least 70%; false positives at most 15% (`D:162-166`).
- D-24: harmful 20/20 blocked; helpful at most 5/50 blocked; neutral at most 2/30 blocked (`D:168-172`).
- D-25: tamper 10/10 detected inside 5 seconds; revert restores clean (`D:174-178`).
- D-28: Deception Resistance improves at least 5 percentage points without standard-category regression (`D:196-200`).
- D-29: hierarchical regret no greater than flat BAI, or at least one documented blocker (`D:202-206`).
- D-30: held-out bootstrap stays within epsilon; refined empirical risk is no worse than bootstrap (`D:208-212`).

### Privacy and error reporting coverage

- [E] D contains access-control, cryptographic-integrity, calibration, quarantine, and human-review behavior, but the summary specifies no distinct PII, secret-handling, user-data retention, network-privacy, or telemetry-export feature.
- [E] Explicit error/reporting surfaces are the archive verifier’s exit code and broken-link report, blocked audit/push/promotion states, SSB diagnostic rows, benchmark failure thresholds, human-review queues, and D-29’s infeasibility report (`D:64-74,92-132,138-212`).

## Dependency, status, and closure matrix

| ID | Status | Direct prerequisites stated in session | Closes / proof target |
|---|---|---|---|
| D-1 | PLANNED | C-15, C-21 | Phase 4H.IV / typed templates and chain |
| D-2 | PLANNED | D-1, B-37 | Phase 4H.V.1 / heterogeneous PPM margin |
| D-3 | PLANNED | D-2 | Phase 4H.V.2 / 2-verifier consensus |
| D-4 | PLANNED | D-1, D-3 | Phase 4H.VI / multi-signal auto-revert |
| D-5 | PLANNED | D-1, D-3 | Phase 4H.VII / BAI regret bound |
| D-6 | PLANNED | D-5 | Phase 4H.VIII.1 / temporal held-out gate |
| D-7 | PLANNED | D-6 | Phase 4H.VIII.2 / cross-family regression block |
| D-8 | PLANNED | D-1 | Phase 4H.IX / archive tamper verification |
| D-9 | PLANNED | D-1..D-8 | companion §§2.3, 4 / frozen-surface block |
| D-10 | PLANNED | D-9 | RPS.I / orchestrator route predicate |
| D-11 | PLANNED | D-10 | RPS.II / Mode-A checkers |
| D-12 | PLANNED | D-11 | RPS.III / measurable-procedure audit gate |
| D-13 | PLANNED | D-10, D-3 | RPS.IV / Mode-B cross-family adversary |
| D-14 | PLANNED | D-13 | RPS.V / inverted-framing quarantine |
| D-15 | PLANNED | D-11, D-12, C-17 | RPS.VI / novelty escalation |
| D-16 | PLANNED | D-9, D-10 | RPS.VII / Mode-E conformal refusal |
| D-17 | PLANNED | D-16 | RPS.VIII / refusal-rate SLA |
| D-18 | PLANNED | C-22, D-11 | Phase 4F extension / Mode-D effects |
| D-19 | PLANNED | D-2, D-3 | companion §1.13 / PPM adversarial baseline |
| D-20 | PLANNED | D-10, D-15 | companion §1.16 + real-problem §1 Mode C / ARC OOD |
| D-21 | PLANNED | D-10 | D-10 gating evidence / routing F1 |
| D-22 | PLANNED | D-16, D-6 | D-16 gating evidence / shifted calibration |
| D-23 | PLANNED | D-14 | D-14 gating evidence / detection/FPR |
| D-24 | PLANNED | D-7 | D-7 gating evidence / canary errors |
| D-25 | PLANNED | D-8, C-27 | D-8 + C-27 gating evidence / tamper suite |
| D-26 | PLANNED | D-1..D-8 shipped | Wave-1 dashboard exposure |
| D-27 | PLANNED | D-10..D-17 shipped | Wave-2 dashboard exposure |
| D-28 | DEFERRED | D-5, D-19, ≥20 D-5 cycles, RunPod | Phase 4H.X / PPM immunization |
| D-29 | DEFERRED | D-5, RunPod | companion Q4 / non-IID hierarchy or infeasibility |
| D-30 | DEFERRED | D-16, unidentified HF corpus | real-problem Q5 / 500-tuple cold start |

Table anchors: D-1..D-9 `D:22-74`; D-10..D-18 `D:80-132`; D-19..D-25 `D:138-178`; D-26..D-27 `D:182-192`; D-28..D-30 `D:196-212`.

### Cross-plan composition explicitly stated by the summary

- [E] D-1 extends C-21 self-modification automation—`check_anti_stale` plus L1A data tables—into a typed reasoning-template surface (`D:298`).
- [E] D-4 replaces C-27 T(-1) `DeterminismDrift` single-signal detection with five orthogonal signals (`D:299`).
- [E] D-9 extends C-27 WiringContract `StubBanAuditor` with frozen-surface access-control predicates (`D:300`).
- [E] D-11 consumes C-24 `symbol_impact` and C-30 surgical-mode DKG tooling for API/code checker dispatch (`D:301`).
- [E] D-12 extends C-31 `FILE_PLAN` with a `MEASURABLE_PROCEDURE` block (`D:302`; block-number inconsistency noted below).
- [E] D-13 extends C-12 per-level auditor `REVIEW` modes with structured cross-family adversarial probing (`D:303`).
- [E] D-18 extends C-22 FeasibilityProber with measurable-effect schema (`D:304`).
- [E] D-26/D-27 extend C-7 dashboard completeness with D-specific drawers/panels (`D:305`).
- [E] Global start gate: `spec-grounder` refuses D-1 until at least C-1, C-15, C-17, C-21, C-22, C-24, and C-27 are `DONE`; the substrate must also report `SCHEMA_VERSION >= 47`, WiringContract, DKG resilience, and MistakeBook (`D:307`).

### Companion-section mapping

- [E] Reasoning-improvement mapping: §3.1→D-1, §3.2→D-2, §3.3→D-3, §§3.6/3.9→D-4, §3.7→D-5, §3.4→D-6, §3.5→D-7, §3.8→D-8, §2→D-9, Phase 4H.X→D-28, and Q4→D-29 (`D:266-279`).
- [E] Real-problem mapping: §3.2→D-10, §4.2→D-11, §4.1→D-12, §4.3→D-13, §4.7→D-14, §4.8→D-15, §4.5→D-16, §4.6→D-17, §4.4→D-18, and Q5→D-30 (`D:281-292`).

### Effort and lifecycle status

- [E] All 30 original D-N files were authored by 2026-05-19 (`D:218-221`).
- [E] Each D-1..D-27 implementation session is estimated at roughly 6–8 hours; combined CPU/cloud work is roughly 160–220 hours (`D:222-225`).
- [E] Each deferred session is estimated at roughly 8–24 wall-clock hours; combined deferred work is roughly 25–75 hours plus RunPod cost (`D:223-225`).
- [E] A session is not `DONE` merely because its file exists: the status definition requires shipped implementation, cloud-Sonnet smoke test, and benchmark threshold (`D:10-14`).

## Deeper-source manifest (not read in this pass)

### Companion designs

| Role | Repository path verified present | Summary reference |
|---|---|---|
| Reasoning-improvement design | `docs/specs/guide/reasoning-improvement-grounded-2026-05-19.md` | `D:3,8,229` |
| Real-problem-solving design | `docs/specs/guide/real-problem-solving-grounded-2026-05-19.md` | `D:3,8,230` |

[E] The summary describes these as 531 and 506 lines respectively, grounded by nine research agents, 80+ tier-1 arXiv sources, and two oracle passes on 2026-05-17 and 2026-05-19 (`D:227-231`). Those counts were not independently validated because the companion contents were deliberately not opened.

### Original session plans

All 30 links named by the summary were verified present by filename under `docs/thefinalglance/D-plan/`; contents were not opened.

| ID | Absolute deeper-plan path | Link anchor |
|---|---|---|
| D-1 | `docs/thefinalglance/D-plan/D-1_reasoning_template_schema.md` | `D:22` |
| D-2 | `docs/thefinalglance/D-plan/D-2_ppm_sidecar_wiring.md` | `D:28` |
| D-3 | `docs/thefinalglance/D-plan/D-3_verification_result_aggregator.md` | `D:34` |
| D-4 | `docs/thefinalglance/D-plan/D-4_tripwire_stack.md` | `D:40` |
| D-5 | `docs/thefinalglance/D-plan/D-5_bai_thompson_promotion.md` | `D:46` |
| D-6 | `docs/thefinalglance/D-plan/D-6_held_out_task_store.md` | `D:52` |
| D-7 | `docs/thefinalglance/D-plan/D-7_cross_domain_canary.md` | `D:58` |
| D-8 | `docs/thefinalglance/D-plan/D-8_crypto_archive_verify.md` | `D:64` |
| D-9 | `docs/thefinalglance/D-plan/D-9_l3_frozen_surfaces.md` | `D:70` |
| D-10 | `docs/thefinalglance/D-plan/D-10_mode_routing_predicate.md` | `D:80` |
| D-11 | `docs/thefinalglance/D-plan/D-11_constructive_checker_registry.md` | `D:86` |
| D-12 | `docs/thefinalglance/D-plan/D-12_operationalization_check_predicate.md` | `D:92` |
| D-13 | `docs/thefinalglance/D-plan/D-13_adversary_role_cross_family.md` | `D:98` |
| D-14 | `docs/thefinalglance/D-plan/D-14_sycophancy_probe.md` | `D:104` |
| D-15 | `docs/thefinalglance/D-plan/D-15_novelty_class_labeling.md` | `D:110` |
| D-16 | `docs/thefinalglance/D-plan/D-16_unicr_conformal_refusal.md` | `D:116` |
| D-17 | `docs/thefinalglance/D-plan/D-17_honest_refusal_sla.md` | `D:122` |
| D-18 | `docs/thefinalglance/D-plan/D-18_observable_effect_typed.md` | `D:128` |
| D-19 | `docs/thefinalglance/D-plan/D-19_prmbench_adversarial_gaming.md` | `D:138` |
| D-20 | `docs/thefinalglance/D-plan/D-20_arc_agi2_ood_probe.md` | `D:144` |
| D-21 | `docs/thefinalglance/D-plan/D-21_mode_routing_coverage_benchmark.md` | `D:150` |
| D-22 | `docs/thefinalglance/D-plan/D-22_conformal_calibration_validation.md` | `D:156` |
| D-23 | `docs/thefinalglance/D-plan/D-23_sycophancy_detection_benchmark.md` | `D:162` |
| D-24 | `docs/thefinalglance/D-plan/D-24_cross_domain_canary_regression_suite.md` | `D:168` |
| D-25 | `docs/thefinalglance/D-plan/D-25_determinism_archive_tamper_suite.md` | `D:174` |
| D-26 | `docs/thefinalglance/D-plan/D-26_reasoning_template_dashboard.md` | `D:182` |
| D-27 | `docs/thefinalglance/D-plan/D-27_mode_routing_refusal_dashboard.md` | `D:188` |
| D-28 | `docs/thefinalglance/D-plan/D-28_adversarial_example_farming.md` | `D:196` |
| D-29 | `docs/thefinalglance/D-plan/D-29_hierarchical_bai_research.md` | `D:202` |
| D-30 | `docs/thefinalglance/D-plan/D-30_calibration_bootstrap_corpus.md` | `D:208` |

Additional deeper file present but not linked by `D-SUMMARY_INDEX.md`: `docs/thefinalglance/D-plan/D-0_INDEX.md`. A later deep pass should determine whether it duplicates, predates, or adds material beyond the summary.

### Research-source interfaces and paper ingestion

- [E] Existing baseline explicitly cited by path: `workflow/papers/dgm-interface.md`; its PDF `.claude/papers/2505.22954v3.pdf` is also present (`D:233-236`).
- [E] The summary’s explicit “introduces to ingest” set is: rStar-Math; V-STaR; arXiv:2507.05619; Audibert COLT 2010 BAI; variance-dependent BAI; LiveCodeBench; NanoFlux; Everitt plus Cohen; Program-of-Thought; CodeAct; AlphaProof; Voyager; Lightweight Latent Verifiers; Self-Correction Bench; Sharma sycophancy; FunSearch; AlphaEvolve; Chollet ARC; ARC Prize 2025; UniCR; PRMBench; SELF-DISCOVER; arXiv:2510.04399; Eureka; Coscientist; adversarial reward-model training; and Lu concept drift (`D:238-262`).
- [E] Additional works appear in per-session citations but outside that consolidated list: DGM; Soft Contamination; Gao reward-model overoptimization; Cross-Context Verification; Hubinger mesa-optimization; Constitutional AI in Small LLMs; Rewarding Doubt; the LLM Honesty Survey; NVARC NVIDIA 2025; Lessons of Developing PRMs; and ThinkPRM (`D:25,31,37,43,55,67,73,101,119,125,131,141,147`).
- Filesystem metadata at discovery time shows many corresponding `*-interface.md` files already exist in `workflow/papers/`, including DGM, rStar-Math, V-STaR, reward-hacking detection, Audibert/variance BAI, LiveCodeBench, NanoFlux, Everitt/Cohen, PoT, CodeAct, AlphaProof, Voyager, latent verifiers, Self-Correction Bench, Sharma, FunSearch, AlphaEvolve, ARC/ARC Prize, UniCR, PRMBench, Eureka, Coscientist, adversarial RM, concept drift, Soft Contamination, Cross-Context Verification, Hubinger, constitutional collapse, Rewarding Doubt, RM overoptimization, Lessons of PRMs, and ThinkPRM. This is existence-only and does not establish that `spec-grounder` accepts their contents.
- No clearly named SELF-DISCOVER interface was visible in the top-level `workflow/papers/` filename listing. The likely arXiv:2510.04399 interface is named `statistical-limits-self-improving-interface.md`, but that mapping should be verified by a deeper reader rather than assumed.

## Missing, ambiguous, or internally inconsistent detail requiring original-plan review

These are omissions or tensions in the summary, not inferred defects in the deeper designs.

1. **Five-mode semantics are incomplete.** A/B/D/E can be partially reconstructed (constructive checker, cross-family Adversary, observable experiment, refusal), but Mode C’s operation is never defined. It is only mentioned in the five-value taxonomy, a “Mode C contradiction” citation, and the ARC-AGI-2 closure (`D:78-84,110-114,144-148`). Exact routing precedence and the full predicate truth table are also absent.
2. **Routing test metrics conflict or are underspecified.** D-10 equates 45 correct out of 50 with “≥90% F1,” although count accuracy and F1 are not interchangeable; D-21 later accepts aggregate F1 ≥0.85 (`D:84,150-154`). Determine which gate is authoritative and whether D-10 is accuracy, macro-F1, or micro-F1.
3. **Baby-step block numbering conflicts.** D-12 says extend BabyStepSpec block 7; the C→D cross-link says C-31 `FILE_PLAN` block 8 is extended with block 9 (`D:92-96,302`). The original must resolve schema and migration numbering.
4. **Dashboard coverage is broader in the premise than in named surfaces.** The premise says every D-N session wires to the dashboard, while D-26 names exposure only for D-1..D-8 and D-27 only for D-10..D-17. D-9, D-18, D-19..D-25, and deferred D-28..D-30 have no explicit dashboard surfaces here (`D:3-4,182-192`).
5. **“Each capability” benchmark coverage is not one-to-one in the summary.** Wave 3 promises a quantitative benchmark and adversarial probe for each Wave-1/Wave-2 capability, but only seven benchmark sessions are listed for eighteen preceding sessions (`D:134-178`). Some capabilities may be tested inside shared suites or originals, but the mapping is absent.
6. **Tripwire mechanics are deferred to deeper sources.** The summary names KL, Δρ, IQR, perplexity, DDM, concurrent-fire count, and one proof requiring a window, but omits thresholds, window size, baselines, missing-signal policy, and which archive state is selected on revert (`D:40-44`).
7. **BAI policy composition is not specified.** D-5 lists Thompson sampling plus Successive Rejects/LUCB but does not state when each is used, how priors initialize, how held-out/canary vetoes affect posterior counts, or the exact regret statistic (`D:46-50`).
8. **Contamination and transfer tests need definitions.** “Uniformly-passing,” six-month age origin, 9-gram threshold/action, family-score aggregation, statistical uncertainty, and exact `>5%` comparison basis are omitted (`D:52-62`).
9. **Archive schema/lifecycle needs reconciliation.** D-1 introduces `ArchiveEntry` plus template `parent_id/archive_status`; D-8 supplies a second explicit field set with timestamps and hashes. The summary does not say whether this is one evolving schema or nested records, nor how retire/revert operations remain append-only (`D:22-26,64-68`).
10. **Cross-family fallback boundary is terse.** D-13 both offers cloud Sonnet when no alternate local model exists and requires Mode E if no alternate family is available; availability, provider failure, cost, and privacy conditions deciding between them are not given (`D:98-102`).
11. **Sycophancy predicates are undefined.** “User-aligned,” framing inversion, output convergence/divergence, and quarantine-release criteria have no machine definition or threshold in the summary (`D:104-108`).
12. **Novelty classification is undefined.** Examples and DKG near-duplicate dependency are present, but the boundary between `Search`, `DistantSearch`, and `DiscoveryCandidate`, confidence handling, and human-review resolution are not (`D:110-114`).
13. **Calibration semantics require the originals.** The summary does not define how `τ` is learned, what probability/confidence it thresholds, how nightly changes are frozen/audited, or why D-22 measures `refusal_rate` around `[τ-ε,τ+ε]` (`D:116-120,156-160`). D-30’s required HuggingFace corpus is explicitly unidentified (`D:208-212`).
14. **SLA boundary behavior is absent.** Exactly 5% and exactly 50%, zero-claim sessions, rolling aggregation, and remediation after `AUTO_FLAG` are not specified (`D:122-126`).
15. **ObservableEffect scope is incomplete.** The summary requires two effects per claim but does not delimit which claims enter Mode D, whether two effects must be independent, or how confounds and sandbox/production disagreement affect verdicts (`D:128-132`).
16. **PRMBench gate wording is inconsistent.** D-19 says promotion requires at least the published baseline but calls 35% conservative relative to a cited published 40–50% Deception Resistance range (`D:138-142`).
17. **ARC fallback and success are open.** Leakage-flag generation, human-baseline target, solve-rate threshold, fixture selection, and what Mode C does are not stated (`D:144-148`).
18. **Tamper suite is only partially enumerated.** D-25 promises ten scenarios but names four; the remaining six and timeout measurement boundary require the original (`D:174-178`).
19. **Polling latency is unquantified.** D-26/D-27 require changes within “poll interval” without stating that interval or stale/error UI behavior (`D:182-192`).
20. **Deferred readiness is incomplete.** D-28 defines the ≥20-cycle signal floor but not “stable”; D-29 provides no model structure/decision rule beyond a spike; D-30 lacks a selected corpus and generic-to-project mixing policy (`D:196-212`).
21. **Status evidence is plan-only.** Every D-1..D-27 row is `PLANNED` in this source even though original files exist. No feature in this discovery should be represented as shipped (`D:10-14,218-225`).
22. **Source paths use two notations.** The header uses repository-relative companion paths and those Unix files exist; provenance repeats them as `F:\.ELAI_workflow\...` Windows paths (`D:3,229-230`). Treat the local repository paths in this manifest as the actionable equivalents, but verify provenance intent during deep review.
23. **Paper prerequisite wording points into originals.** The summary says each D-N file’s `Pre-requisites` line names its papers, but the summary’s own displayed prerequisite lines generally list plan IDs rather than paper files (`D:22-212,264`). Later agents must inspect the originals before concluding ingestion readiness.

## Summary-layer completion statement

- Primary summary coverage: `311/311` lines read.
- Orientation overview coverage: `592/592` lines read.
- Session coverage: `30/30` D IDs inventoried with names, statuses, prerequisites, behaviors, acceptance tests, and source anchors.
- Original-plan link coverage: `30/30` linked files verified present by filename; `D-0_INDEX.md` separately surfaced as an unlinked deeper source.
- Explicit implementation status: D-1..D-27 `PLANNED`; D-28..D-30 `DEFERRED`; zero `DONE` in the assigned source.
- No original D-N plan or companion-design content was read, and no behavior absent from the summary has been promoted to explicit fact.
