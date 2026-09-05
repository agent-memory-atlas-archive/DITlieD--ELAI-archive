# DEEP-D-004 deep feature discovery

## Scope, evidence, and counting contract

This ledger is a deep read of exactly the seven original files assigned by the
DEEP-D-004 manifest. No other original D-plan chunk was opened. The summary-layer
file 04-D-SUMMARY_INDEX.discovery.md was used only to orient the comparison and
to mark summary deltas; it is not evidence for a deep-source claim.

Evidence markers:

- E — explicit in the cited original source.
- I — inference, always identified as such and never promoted to an implementation fact.
- A — ambiguity or missing contract exposed by joining explicit statements.
- C — internal conflict, or a material conflict with the summary layer.

Summary-delta markers:

- = — the summary already preserves the material statement.
- Δ — the summary has the headline but loses material mechanics, defaults, gates, or boundaries recorded here.
- + — the statement is absent from the summary and must be added.
- ~ — the summary must be corrected, not merely expanded.

Every ledger record has one unique atomic ID. Repeated acceptance wording is
cross-referenced rather than counted again. “Planned” below means the
summary-layer status plus future-tense source checklists; none of these files
contains evidence that the implementation shipped.

## Exact processed manifest and mechanical verification

Manifest section: DEEP-SOURCE-MANIFEST.md lines 499-511, heading DEEP-D-004.

| Order | Class | Logical lines | Bytes | Manifest SHA-256 | Verification | Exact absolute path |
|---:|---|---:|---:|---|---|---|
| 1 | numbered/index original | 327 | 16650 | 0be0c4b31e3ef02c34d9f90da2fc873799a0e5baf77ce6c65b1b417a1b4f37eb | line/byte/hash match | docs/thefinalglance/D-plan/D-4_tripwire_stack.md |
| 2 | numbered/index original | 269 | 12118 | 51818038b085919609f4736e7c4bca3145a0a5bc308c8ea87062d2f92d1a6291 | line/byte/hash match | docs/thefinalglance/D-plan/D-7_cross_domain_canary.md |
| 3 | numbered/index original | 404 | 20681 | 9af38bb296549864faf9742f140ba5e7535d87a0a9084408e9ddb6e2c8371ae2 | line/byte/hash match | docs/thefinalglance/D-plan/D-11_constructive_checker_registry.md |
| 4 | numbered/index original | 229 | 10638 | c0301438b1372f9c13a098ce52b936742fc94f39d19e805cab519dea8e0e2871 | line/byte/hash match | docs/thefinalglance/D-plan/D-15_novelty_class_labeling.md |
| 5 | numbered/index original | 246 | 11835 | aa717c6562a29091d06e595274b8115fdd2a67f0f926c29871d6021f5021a3fe | line/byte/hash match | docs/thefinalglance/D-plan/D-18_observable_effect_typed.md |
| 6 | numbered/index original | 167 | 7162 | 5f86d79ffda38e2bae156b4cc885e3637455d4888fd9b5dfeac2f5652d3a894b | line/byte/hash match | docs/thefinalglance/D-plan/D-20_arc_agi2_ood_probe.md |
| 7 | numbered/index original | 113 | 4283 | b4c67ffe790e5d2985df587f220e79cff6e9820cde53d0fb788ec8a4b8f08a48 | line/byte/hash match | docs/thefinalglance/D-plan/D-22_conformal_calibration_validation.md |
| **Total** | **7 originals** | **1755** | **83367** | — | **all exact** | — |

Verification used byte-for-byte SHA-256 plus wc logical-line and byte counts
before reading. All 1,755 logical lines were then read, including amendments,
out-of-scope lists, acceptance checklists, and trailing Next sections.

## Per-file exhaustive ledgers

### D-4 — Tripwire stack

Source shorthand D4 means D-4_tripwire_stack.md.

| Atomic ID | Ev. | Cat. | Δ | Explicit feature, behavior, or boundary | Anchor |
|---|---|---|---|---|---|
| D004-D4-001 | E | capability | = | Replace the undocumented MSI > 0.3 / 10%-over-3-pipelines rule with five orthogonal tripwires: KL divergence, correlation drop, IQR outlier, perplexity anomaly, and DDM; auto-revert requires at least two concurrent fires. | D4:L3-L8 |
| D004-D4-002 | E | status | + | Estimated effort is one session; the plan claims roughly 80% coverage of observed reward hacks and cites 21.3% prevalence as rationale, not as an acceptance result. | D4:L7-L9 |
| D004-D4-003 | E | dependency | = | Hard prerequisites are D-1 ArchiveChain/ArchiveEntry and D-3 VerifierDisagreement; spec-grounder must verify both DONE. | D4:L10-L11 |
| D004-D4-004 | E | concurrency | + | It is declared parallel-safe with D-5, D-6, and D-8, with explicit dataflow: tripwire fires feed BAI, held-out pass rate feeds Δρ, and rollback appends through ArchiveChain. | D4:L12-L14 |
| D004-D4-005 | E | config | Δ | Paper-derived thresholds are KL τ=0.3; Δρ>0.5 over 50-step windows; high outlier >Q3+3×IQR; PPL anomaly >μ+2σ; DDM warning/drift at 2σ/3σ; concurrent-fire count 2; window 50. | D4:L18-L31, L80-L85 |
| D004-D4-006 | E | boundary | + | Reward-signal checksums belong to D-8 and unauthorized reward-function syscall tracing belongs to D-9 LockedKeysAuditor; D-4 does not implement either. | D4:L25-L27 |
| D004-D4-007 | E | lifecycle | Δ | On concurrence in the same default 50-action window, retire the most recently Promoted template with RetirementReason::Degradation, switch the orchestrator to its parent_id, and retain an auditable append-only revert chain. | D4:L38-L42 |
| D004-D4-008 | E | integration | + | Existing inputs are ArchiveChain retired transitions, D-3 per-step proxy reward plus deterministic compile/test outcome as gold reward, Phase-4G entropy/mean_logprob as a PPL proxy, and 50-step SSB aggregation. | D4:L44-L49 |
| D004-D4-009 | E | dependency | + | Before TESTING_FIRST, paper-ingestor must create three interfaces: reward-hacking-detection-2507.05619, concept-drift-2004.05785, and rm-overoptimization-2210.10760. | D4:L64-L72 |
| D004-D4-010 | E | schema | Δ | Closed, L3-locked TripwireKind has KlDivergence, CorrelationDrop, IqrOutlier, PerplexityAnomaly, and Ddm. | D4:L76-L79 |
| D004-D4-011 | E | schema | Δ | TripwireConfig fields are kl_spec_tau, rho_delta_threshold, iqr_outlier_multiplier, ppl_anomaly_sigma, ddm_warning_threshold, ddm_drift_threshold, consensus_fire_count, and window_size, with the defaults in D004-D4-005. | D4:L80-L85 |
| D004-D4-012 | E | schema | + | TripwireEvent carries kind, UTC fired_at, value, threshold, SignalSource, and audit_id; SignalSource is CurrentSession, HeldOut, or CrossDomain. | D4:L86-L88 |
| D004-D4-013 | E | algorithm | + | kl_divergence applies standard KL; both inputs must sum to 1 within an unspecified epsilon, identical distributions return 0, and a nonzero current mass against zero baseline returns positive infinity. | D4:L90-L94 |
| D004-D4-014 | E | workflow | + | check_kl aggregates StepOutput slices into vocabulary-bin token histograms and emits a TripwireEvent only when KL is strictly greater than tau. | D4:L95-L97 |
| D004-D4-015 | E | algorithm | + | pearson_correlation requires equal lengths and returns 0 on zero variance; check_correlation_drop compares two windows and fires when corr_a−corr_b is strictly greater than the configured delta. | D4:L99-L105 |
| D004-D4-016 | E | algorithm | + | iqr_outlier_check computes Q1/Q3/IQR, fires if any value is strictly above Q3+multiplier×IQR, and places the outlier itself in event.value. | D4:L107-L111 |
| D004-D4-017 | E | algorithm | + | ppl_anomaly_check computes mean and standard deviation over its PPL values and fires if any value is strictly above mean+sigma_multiplier×standard deviation. | D4:L113-L117 |
| D004-D4-018 | E | state | Δ | DDM tracks p_i+s_i, where s_i=sqrt(p_i(1−p_i)/i); levels are Normal, Warning, Drift, with Normal→Warning at p_min+2s_min and Warning→Drift at p_min+3s_min. | D4:L119-L126 |
| D004-D4-019 | E | recovery | + | DdmState.update(success) emits only when entering Drift and is required to reset statistics when recovering from Drift to Normal. | D4:L127-L128, L275-L276 |
| D004-D4-020 | E | schema | + | TripwireOrchestrator owns config, a VecDeque ActionRecord window, DdmState, SsbSender, and Arc<RwLock<ArchiveChain>>. | D4:L130-L133 |
| D004-D4-021 | E | determinism | + | ingest appends every post-action ActionRecord, trims the oldest record deterministically at capacity, invokes all detectors over the current window, and aggregates their events without silent drops. | D4:L134-L136, L212-L219 |
| D004-D4-022 | E | event | Δ | Reaching consensus emits TripwireFiredConcurrent with the fired-kind list and audit ID. | D4:L137-L139, L147 |
| D004-D4-023 | E | lifecycle | Δ | The automatic rollback invokes ArchiveChain.retire on the latest promoted entry through append-only ArchiveChain semantics. | D4:L139-L140, L191-L193 |
| D004-D4-024 | E | event | Δ | TripwireAutoRevert carries template_id, optional parent_template_id, fired_kinds, and audit_id; its intent event must be emitted before archive.retire so a failed retire remains auditable. | D4:L141-L149, L220-L221 |
| D004-D4-025 | E | event | Δ | SCHEMA_VERSION moves 49→50 and adds TripwireFired, TripwireFiredConcurrent, and TripwireAutoRevert with round-trip tests; individual fires include kind/value/threshold/source/audit_id. | D4:L144-L150 |
| D004-D4-026 | E | config/security | + | elai_cc.toml gains the exact [tripwire] defaults; thresholds must load from config rather than constructor constants and the section is operator-tunable but L3-locked against agent edits. | D4:L152-L157, L212-L223 |
| D004-D4-027 | E | unit-test | + | KL tests cover identical=0, disjoint=∞, and a hand-computed positive finite uniform-vs-peaked case. | D4:L159-L161 |
| D004-D4-028 | E | unit-test | + | Correlation tests cover +1/−1 linear cases, zero for uncorrelated data, and a window-to-window drop. | D4:L162-L163 |
| D004-D4-029 | E | unit-test | + | IQR tests require Q3+4IQR to fire and Q3+2.5IQR not to fire at multiplier 3.0. | D4:L164-L165 |
| D004-D4-030 | E | unit-test | + | PPL tests require μ+2.5σ to fire and μ+1.5σ not to fire at sigma 2.0. | D4:L166-L167 |
| D004-D4-031 | E | unit-test | + | DDM test drives 100 successes then 50 samples with 80% failure through Warning and Drift; acceptance additionally requires recovery reset. | D4:L168-L169, L275-L276 |
| D004-D4-032 | E | unit-test | + | Orchestrator test with fire count 2 and three distinct event kinds must emit both concurrent-fire and auto-revert events. | D4:L170-L171 |
| D004-D4-033 | E | integration-test | Δ | A 100-action stream injects regression at action 60; at least 3/5 tripwires and auto-revert must occur by action 65, and the latest template must be Retired for Degradation. | D4:L173-L181 |
| D004-D4-034 | E | decision | + | The decision record must justify DDM as an orthogonal, low-false-alarm concept-drift signal with published defaults. | D4:L183-L187 |
| D004-D4-035 | E | decision | + | The decision record must justify fire-count 2 using the cited 8%-zero-fire / roughly-80%-two-or-more distribution and the false-positive/miss tradeoff. | D4:L188-L190 |
| D004-D4-036 | E | compliance | + | The decision record must justify retire-not-delete because D-1 requires append-only lineage. | D4:L191-L193 |
| D004-D4-037 | E | observability | Δ | Synthetic 80%-failure injection expects at least three distinct TripwireFired rows, at least one concurrent row, at least one auto-revert row, and a degradation-retired archive entry. | D4:L224-L229 |
| D004-D4-038 | E | artifact | + | Evidence artifacts are workflow/reasoning/D-4-decisions.md and config defaults matching cited values; the source calls the latter tripwire.toml although the implementation surface is elai_cc.toml. | D4:L230-L231 |
| D004-D4-039 | E | done-gate | + | Deterministic gates inspect schema version 50, all three SSB variants, core types, all five detector functions, exact four headline thresholds, workspace build, scoped orchestrator/contracts tests, and all three paper interfaces. | D4:L233-L268 |
| D004-D4-040 | E | acceptance | Δ | Done additionally requires SCHEMA_VERSION_PUBLIC=50, payload round trips, all detector/state/config/integration behaviors, zero new workspace errors, at least 15 new orchestrator tripwire tests, contracts tests, decision doc, and project_d4_tripwire_stack.md from memory-writer. | D4:L270-L296 |
| D004-D4-041 | E | fallback | + | Cloud regression proof uses elai archive verify after D-8; before D-8 it falls back to direct inspection of reasoning-templates.jsonl. | D4:L288-L291 |
| D004-D4-042 | E | negative-test | + | A clean 50-action cloud session must emit zero TripwireFiredConcurrent rows. | D4:L292-L293 |
| D004-D4-043 | E | surface | + | Change set: add tripwire.rs, tripwire integration/contract tests, decision doc, and three paper interfaces; modify ssb.rs, task_runner.rs, elai_cc.toml, and orchestrator lib.rs. | D4:L298-L311 |
| D004-D4-044 | E | non-goal | + | D-5 BAI algorithm, D-8 archive-verifier CLI, D-9 enforcement of the documented lock, and D-26 dashboard rendering are explicitly out of scope. | D4:L313-L321 |
| D004-D4-045 | E | dependency | + | Next consumers are D-5, which consumes consensus passes plus fires, and D-6, whose held-out distribution feeds KL/Δρ baselines. | D4:L323-L327 |
| D004-D4-046 | C | ordering | ~ | Task prose lists retire before TripwireAutoRevert emission, while the stub ban mandates emitting TripwireAutoRevert before retire. The latter is the explicit failure-audit rule and the implementation order must be reconciled. | D4:L137-L142 vs L220-L221 |
| D004-D4-047 | A | state | + | A Drift→Normal transition condition is never defined even though reset on that transition/recovery is required. | D4:L122-L128, L275-L276 |
| D004-D4-048 | A | failure | + | No missing-signal/NaN/short-window policy, quartile/stddev convention, KL epsilon/smoothing rule, or minimum detector sample count is specified. | D4:L90-L128 |
| D004-D4-049 | A | idempotency | + | No cooldown, deduplication, or repeated-window rule prevents the same concurrence from emitting/reverting repeatedly. | D4:L134-L142 |
| D004-D4-050 | A | recovery | + | The source records intent before retire failure but does not define retry, rollback, parent absence, archive-lock poisoning, or orchestrator switch behavior when retire fails. | D4:L139-L142, L220-L221 |
| D004-D4-051 | A | schema | + | “tripwire.toml” in expected_artifact conflicts nominally with the only configured file, .elai_cc/elai_cc.toml [tripwire]. | D4:L152-L157, L230-L231 |
| D004-D4-052 | C | signal | + | The prerequisite calls D-3 VerifierDisagreement an input, while the partial-foundation/wiring text instead identifies VerificationResult proxy reward and VerifierConsensusPass count as the correlation source; the exact D-3 signal is not stable within the source. | D4:L10-L11, L45-L47, L206-L209 |

### D-7 — Cross-domain canary

Source shorthand D7 means D-7_cross_domain_canary.md.

| Atomic ID | Ev. | Cat. | Δ | Explicit feature, behavior, or boundary | Anchor |
|---|---|---|---|---|---|
| D004-D7-001 | E | capability | = | Evaluate every L2 mutation across code-fix, math-proof, prose-reason, planning, and knowledge-QA; intended headline gate blocks when a non-target family regresses by more than 5%. | D7:L3-L7 |
| D004-D7-002 | E | status | + | Estimated effort is one to two sessions. | D7:L8 |
| D004-D7-003 | E | dependency | = | D-6 is the prerequisite because its held-out store supplies per-family tags. | D7:L9-L10 |
| D004-D7-004 | E | concurrency | + | It is declared parallel-safe with D-5 and D-8 but not D-4 because D-4 reads canary signals as a Δρ source. | D7:L11-L13 |
| D004-D7-005 | E | dataset | Δ | code-fix means Rust/Python compile-and-test fixtures from Exercism and public LeetCode. | D7:L22-L25 |
| D004-D7-006 | E | dataset | Δ | math-proof means SymPy-checkable identities and step proofs from the PRMBench math subset. | D7:L26-L27 |
| D004-D7-007 | E | dataset | Δ | prose-reason means natural-language inference/reasoning from HotpotQA and AdversarialQA public splits. | D7:L28-L29 |
| D004-D7-008 | E | dataset | Δ | planning means multi-step decomposition/dependency order from PlanBench and AlfWorld public data. | D7:L30-L31 |
| D004-D7-009 | E | dataset | Δ | knowledge-QA means factual recall plus grounded citation from PopQA, TriviaQA, and FActScore. | D7:L32-L33 |
| D004-D7-010 | E | lifecycle | + | Public HuggingFace fixtures are bootstrapped and tagged into D-6 HeldOutStore on first ingest. | D7:L35-L36 |
| D004-D7-011 | E | dependency | + | Existing contracts are HeldOutStore.query(domain), the D-1 Domain enum, and D-5’s single gate; D-7 produces per-family CrossDomainRegression rows for BAI. | D7:L38-L43 |
| D004-D7-012 | E | dependency | + | paper-ingestor must produce workflow/papers/nanoflux-2509.23252-interface.md. | D7:L54-L59 |
| D004-D7-013 | E | schema/lifecycle | Δ | Domain extends from Code/Math/Prose/Mixed to add Planning and KnowledgeQa; all matchers become exhaustive, old Mixed entries remain valid, and no disk migration is needed. | D7:L63-L69 |
| D004-D7-014 | E | schema | Δ | Closed, L3-locked CanaryFamily is CodeFix, MathProof, ProseReason, Planning, KnowledgeQa, mapped to Domain with Code→CodeFix and Math→MathProof called out. | D7:L71-L75 |
| D004-D7-015 | E | schema | + | FamilyEvalResult carries family, pass_count, total_count, and pass_rate. | D7:L76-L77 |
| D004-D7-016 | E | schema | + | CanaryReport carries template_id, baseline and candidate vectors, per-family deltas, target_family, and CanaryDecision. | D7:L78-L80 |
| D004-D7-017 | E | state | Δ | CanaryDecision is Allow, BlockRegression{family,delta}, or InsufficientData{family}. | D7:L81 |
| D004-D7-018 | E | schema | + | CanaryRunner owns HeldOutStore, Evaluator, regression_threshold, and min_samples_per_family. | D7:L83-L85 |
| D004-D7-019 | E | workflow | Δ | run queries every family, evaluates baseline and candidate templates on the held-out tasks, computes pass rates and deltas, then returns the report. | D7:L86-L89 |
| D004-D7-020 | E | decision | Δ | Implementation text says BlockRegression if any family delta is below negative threshold and InsufficientData if any family sample count is below the minimum. | D7:L90-L91 |
| D004-D7-021 | E | dataset | + | Bootstrap prescription is code-fix 10 Exercism+5 LeetCode; math 10 PRMBench; prose 10 HotpotQA+5 AdversarialQA; planning 10 PlanBench; knowledge 10 PopQA+5 TriviaQA. | D7:L93-L99 |
| D004-D7-022 | E | schema/compliance | + | Every fixture must populate ExpectedOutcome, set domain correctly, and cite benchmark plus version in source. | D7:L100-L101 |
| D004-D7-023 | E | event | Δ | SCHEMA_VERSION moves 52→53; CrossDomainRegression records template/family/baseline/candidate/delta/threshold/audit_id, CanaryFamilyPass records pass rate/sample count, and CanaryInsufficientData records sample/required counts; all round-trip. | D7:L103-L111 |
| D004-D7-024 | E | config/security | Δ | L3-locked [canary] defaults are regression_threshold=0.05, min_samples_per_family=5, and all five enabled family snake-case names; the threshold must load from config. | D7:L113-L117, L172-L176 |
| D004-D7-025 | E | workflow | Δ | BaiPromotionEngine.promote_round runs the canary before every promotion; regression skips promotion and emits SSB, comparing current Promoted target-family template with the waiting Candidate. | D7:L119-L125 |
| D004-D7-026 | E | unit-test | + | Tests cover pass-rate division, at least ten decision scenarios, family/domain mapping, and thresholds where −0.06 blocks while −0.04 does not at 0.05. | D7:L127-L133 |
| D004-D7-027 | E | integration-test | Δ | Mock 50-task scenarios require Allow for +10% code/no other loss, BlockRegression for +10% code/−8% prose, and InsufficientData for only three planning tasks. | D7:L135-L143 |
| D004-D7-028 | E | decision | + | Decision doc must justify five families as the asserted 3-too-few/12-too-many compromise, 5% as noise-vs-change balance, and minimum five as a floor that auto-defers BAI until data accrues. | D7:L145-L151 |
| D004-D7-029 | E | failure | + | Sparse data must not panic or allow promotion: it returns InsufficientData, emits its SSB row, and defers promotion. | D7:L150-L151, L172-L174 |
| D004-D7-030 | E | determinism | + | Existing Domain variants must not be renamed. | D7:L172-L176 |
| D004-D7-031 | E | observability | Δ | The synthetic +10% code/−8% prose case expects one prose_reason CrossDomainRegression at −0.08, four family-pass rows, no TemplatePromoted, unchanged Candidate status, and a persisted report under workflow/sessions/{id}/canary-reports/. | D7:L177-L185 |
| D004-D7-032 | E | done-gate | + | Deterministic checks inspect schema 53, three variants, core types, Planning/KnowledgeQa arms, config, seed fixtures, BAI wiring, workspace/tests, and NanoFlux interface. | D7:L187-L224 |
| D004-D7-033 | E | acceptance | Δ | Done requires public schema 53, event round trips, mapping/enum extension, at least ten decision cases, fixture seeding, BAI block integration, loaded defaults, zero new errors, and at least ten new canary tests. | D7:L226-L236 |
| D004-D7-034 | E | cloud-test | Δ | Cloud A expects +10% code/−8% prose to block; B expects +10% code/+5% math/neutral others to emit five family-pass rows and TemplatePromoted; C expects sparse data to emit CanaryInsufficientData. | D7:L237-L243 |
| D004-D7-035 | E | surface | + | Change set adds canary.rs, at least 50 dated seed TOMLs, integration test, decision doc, and paper interface; modifies Domain, SSB, BAI, config, and lib.rs. | D7:L245-L258 |
| D004-D7-036 | E | non-goal | + | D-24 validation, D-15 novelty, and D-18 observable effects are out of scope; D-8 later surfaces the lineage and D-24 benchmarks 100 mutations. | D7:L260-L269 |
| D004-D7-037 | C | decision | ~ | The objective says only non-target regressions block, but CanaryRunner says ANY family delta below threshold blocks. The target-family rule must be corrected or intentionally broadened. | D7:L3-L6 vs L90-L91 |
| D004-D7-038 | C | schema | + | The plan calls CanaryFamily↔Domain a bijection, but Domain contains Mixed and six variants after extension while CanaryFamily has five; no mapping for Mixed is specified. | D7:L63-L75, L127-L131 |
| D004-D7-039 | C | dataset | + | The explicit bootstrap recipe totals 65 tasks, while integration/acceptance/files repeatedly say 50 or at least 50. “At least ten per family” is compatible with 65 but the canonical fixture count is not unique. | D7:L93-L99 vs L137, L232, L255 |
| D004-D7-040 | A | decision | + | Exact −0.05 boundary behavior is untested, and precedence is undefined when one family regresses while another has insufficient data. | D7:L90-L91, L132-L133 |
| D004-D7-041 | A | determinism | + | CanaryDecision can store only one offending family; selection order/tie-breaking for multiple regressions or sparse families is unspecified. | D7:L81, L90-L91 |
| D004-D7-042 | A | failure | + | Evaluator errors, held-out query failures, partial family completion, retries, cancellation, and concurrent promotion races have no contract. | D7:L83-L91, L119-L125 |
| D004-D7-043 | A | artifact | + | Persisting CanaryReport is required only by observable_effect; no serialization schema, write ordering, or file change is specified for canary-reports. | D7:L177-L185, L245-L258 |
| D004-D7-044 | A | concurrency | + | “Parallel-safe with D-5” conflicts operationally with modifying bai.rs and requiring BaiPromotionEngine to exist; branch/landing order is not explained. | D7:L11-L12, L119-L125, L252 |

### D-11 — Constructive-checker registry

Source shorthand D11 means D-11_constructive_checker_registry.md. Lines
330-404 are load-bearing amendments, not commentary.

| Atomic ID | Ev. | Cat. | Δ | Explicit feature, behavior, or boundary | Anchor |
|---|---|---|---|---|---|
| D004-D11-001 | E | capability | = | Bootstrap .elai_cc/checkers/{math,code,text,graph,api,perf}, six initial typed checkers, CheckerResult, and orchestrator dispatch so Mode A constructive proof becomes operational and external checkers—not LLM confidence—are ground truth. | D11:L3-L9 |
| D004-D11-002 | E | status | + | Estimated effort is three to four sessions. | D11:L10 |
| D004-D11-003 | E | dependency | = | D-10 mode routing must be DONE; D-12 is declared parallel-safe and consumes the registry as its target. | D11:L11-L14 |
| D004-D11-004 | E | surface | Δ | Full registry blueprint includes math/{sympy,z3,lean}, code/{compile/test for Rust/Python/TS}, text/{regex_assertion,schema_validate}, graph/networkx, api/registry_check, and perf/{timing_harness,asymptotic_curve_fit}. | D11:L18-L28 |
| D004-D11-005 | E | contract | Δ | Conceptual checker contract is deterministic Claim+Candidate+SandboxHandle input and Pass, Fail with evidence, or Inapplicable with reason output. | D11:L30-L37 |
| D004-D11-006 | E | scope | Δ | The initial six are sympy, z3, compile_rust, compile_py, registry_check, and regex_assertion; Lean, TypeScript, performance, schema validation, and networkx are deferred. | D11:L39-L47 |
| D004-D11-007 | E | security | + | Tier-C sandbox from B-18/C-11 supplies SandboxHandle; the existing Python sidecar scaffold is reusable and D-10 already has the constructive dispatch hook. | D11:L55-L58 |
| D004-D11-008 | E | dependency | + | Before testing, paper-ingestor must produce PoT, CodeAct, and AlphaProof interface files with the exact slugs pot-2211.12588, codeact-2402.01030, alphaproof-nature-2025. | D11:L68-L76 |
| D004-D11-009 | E | schema | Δ | CheckerResult is Pass, Fail{evidence: Vec<BoundedString<2048>>}, or Inapplicable{reason:String}. | D11:L80-L83 |
| D004-D11-010 | E | schema | + | CheckerEntry fields are name, claim_types, domain, executable_path, CheckerLanguage, and timeout_secs; CheckerLanguage is closed Python/Shell/Rust/Js. | D11:L84-L87 |
| D004-D11-011 | E | schema | + | CheckerRegistry stores a Vec<CheckerEntry>. | D11:L88 |
| D004-D11-012 | E | lifecycle | + | CheckerRegistry.load walks root/*/*.toml registration files, one CheckerEntry per TOML. | D11:L89-L91 |
| D004-D11-013 | E | dispatch | Δ | applicable requires both claim_type membership and domain match. | D11:L92-L93 |
| D004-D11-014 | E | dispatch | Δ | run chooses the first applicable checker, sandbox-executes it, parses stdout JSON, and returns its CheckerResult. | D11:L94-L97 |
| D004-D11-015 | E | fallback | + | Checker timeout returns Inapplicable with reason “timeout”; malformed checker JSON also returns Inapplicable rather than panicking. | D11:L94-L97, L219-L224 |
| D004-D11-016 | E | config | + | Registration TOML names the checker, accepted claim types, domain, executable path, language, and timeout; the canonical example is SymPy/math/Python/30 seconds. | D11:L99-L108 |
| D004-D11-017 | E | checker | Δ | sympy.py reads a math identity JSON request, parses LHS/RHS, simplifies the difference, passes at zero, fails with the simplified difference, and is Inapplicable on parse error. | D11:L110-L117 |
| D004-D11-018 | E | process | + | sympy.py emits JSON; Pass, Fail, and Inapplicable exit 0, while only script-level failure exits 1. | D11:L116-L119 |
| D004-D11-019 | E | supply-chain | + | SymPy must be present and version-pinned in pyproject.toml or its install requirement documented under the anti-stale rule. | D11:L120-L121 |
| D004-D11-020 | E | checker | Δ | z3.py translates finite-domain SAT/SMT claims, proves by unsatisfiability of the negation, and returns a counterexample on Fail; z3-solver must be declared. | D11:L123-L128 |
| D004-D11-021 | E | checker/security | Δ | compile_rust.sh writes candidate source to a temporary lib.rs and runs cargo check inside Tier C; exit 0 passes and stderr evidence accompanies failure. | D11:L130-L135 |
| D004-D11-022 | E | checker/security | Δ | compile_py.sh analogously runs python -m py_compile plus ruff check in the checker path. | D11:L137-L139 |
| D004-D11-023 | E | checker/integration | Δ | registry_check.py accepts package+registry JSON and performs live npm, PyPI, or crates.io lookup; npm checks version/dist-tags/time, PyPI version/license/recent release, and crates.io its API. | D11:L141-L149 |
| D004-D11-024 | E | checker/failure | + | Registry lookup passes only for a hit with a release in the last 24 months, fails “package not found” or “stale”, and is Inapplicable for an unknown registry. | D11:L145-L152 |
| D004-D11-025 | E | checker | Δ | regex_assertion.py accepts text, regex, and full-or-contains mode; it passes on match, fails with mismatch position, and is Inapplicable for invalid regex. | D11:L154-L159 |
| D004-D11-026 | E | workflow/event | Δ | D-10’s Constructive branch calls CheckerRegistry.run and emits CheckerDispatched plus exactly one CheckerPass, CheckerFail, or CheckerInapplicable result row. | D11:L161-L165 |
| D004-D11-027 | E | event | Δ | SCHEMA_VERSION moves 56→57; CheckerDispatched carries claim/checker/audit, CheckerPass adds latency_ms, CheckerFail carries evidence, and CheckerInapplicable carries reason; all round-trip. | D11:L167-L172 |
| D004-D11-028 | E | unit-test | + | Registry tests require math→sympy+z3 applicability, API→registry_check, mocked pass parsing, and timeout→Inapplicable. | D11:L174-L178 |
| D004-D11-029 | E | integration-test | Δ | Integration cases are true and false SymPy identities, invalid Rust compile failure, live npm “react” pass, and fake malicious package failure. | D11:L180-L188 |
| D004-D11-030 | E | decision | + | Decision record chooses Python for mature SymPy/Z3 bindings. | D11:L190-L192 |
| D004-D11-031 | E | isolation | + | Decision record chooses subprocess JSON I/O over PyO3 to isolate Python state and apply Tier-C constraints uniformly. | D11:L193-L195 |
| D004-D11-032 | E | security | + | Decision record makes live existence checking an anti-slopsquatting gate before an agent imports unknown dependencies. | D11:L196-L198 |
| D004-D11-033 | E | integration | + | Registry consumes GroundingMode::Constructive, Claim/ClaimType, Domain, and Tier C; it is reachable through mode dispatch and optional registration paths in elai_cc.toml [checkers]. | D11:L200-L218 |
| D004-D11-034 | E | timeout | + | Every run must honor CheckerEntry.timeout_secs; infinite waits are forbidden. | D11:L219-L220 |
| D004-D11-035 | E | integration | + | Production registry_check must hit the live registry; a cached mock is forbidden in production. | D11:L221 |
| D004-D11-036 | E | failure | + | An empty registry returns an empty Vec without panic. | D11:L222 |
| D004-D11-037 | E | observability | Δ | Six synthetic claims expect six dispatch rows plus six terminal result rows, six executable scripts in the registry, and parseable JSON stdout from each. | D11:L225-L230 |
| D004-D11-038 | E | done-gate | + | Deterministic checks inspect schema 57, four SSB variants, registry types, six files, at least six TOMLs, build/tests, direct SymPy JSON round-trip, live registry smoke, and three paper interfaces. | D11:L232-L273 |
| D004-D11-039 | E | acceptance | Δ | Done requires public schema 57, four event round trips, registry unit tests, executable scripts, valid registrations, per-checker positive/negative behavior, zero new errors, and at least ten new checker tests. | D11:L275-L290 |
| D004-D11-040 | E | cloud-test | Δ | Cloud math must route Mode A→SymPy→CheckerPass; fake npm install must be intercepted, emit CheckerFail, and block installation. | D11:L291-L296 |
| D004-D11-041 | E | surface | + | Change set adds checker_registry.rs, six script+TOML pairs, dispatch integration test, decision doc, and three paper interfaces; modifies SSB, grounding_mode, lib.rs, and Python dependency declarations. | D11:L298-L316 |
| D004-D11-042 | E | non-goal | Δ | Lean, TypeScript check/test, schema_validate, networkx, timing_harness, and multi-checker voting are deferred; v1 dispatch remains first-applicable single checker. | D11:L318-L324 |
| D004-D11-043 | E | dependency | + | D-12 next requires every claim-emitting baby step to reference an applicable checker. | D11:L325-L328 |
| D004-D11-044 | E | amendment/contract | + | J-11 imports a minimal verifier contract: self-contained verifier script, candidate on stdin, deterministic expected stdout, and passed/score/info/metadata output; info transports status/error/traceback evidence with no LLM in the verdict path. | D11:L330-L344 |
| D004-D11-045 | E | amendment/lifecycle | + | At registration time, every checker must run its bundled reference fixture; a failure rejects the checker and fails closed before runtime admission. | D11:L346-L351 |
| D004-D11-046 | E | amendment/decision | + | Registry verdict composition must support a demotion clause: CheckerResult::Fail caps the overall verdict rather than being averaged with other scores; construction failure≤0.3 is the given example. | D11:L353-L359 |
| D004-D11-047 | E | amendment/security | + | ComBench reliability_guard must not be copied as a sandbox; generated code must execute in ELAI Tier C using seccomp, SO_PEERCRED, and capability drop because the Python guard does not prevent network, out-of-limit writes, or process spawn. | D11:L361-L367 |
| D004-D11-048 | E | amendment/retry | + | J-15 makes every first checker Fail deterministically trigger exactly one structured reformulation prompt containing checker X and reason Y, then re-run the same checker. | D11:L369-L383 |
| D004-D11-049 | E | amendment/state | + | If the one reformulation passes, claim state becomes Verified; a second Fail makes it Ungrounded. | D11:L384-L386 |
| D004-D11-050 | E | amendment/determinism | + | No model chooses retry, there is no unbounded loop, the error is the sole feedback, the checker is the sole verdict, and the one-pass bound is explicitly anti-gaming. | D11:L388-L395 |
| D004-D11-051 | E | amendment/composition | + | The demotion clause remains in force: reformulation applies to the failed checker’s claim and does not remove the cap on overall verdict calculation. | D11:L397-L404 |
| D004-D11-052 | C | schema | + | J-15 calls error text the info field “from CheckerResult,” but the declared CheckerResult has evidence on Fail and reason on Inapplicable, no info field. | D11:L80-L83 vs L375-L379 |
| D004-D11-053 | A | determinism | + | “First applicable” has no stable ordering/priority rule; filesystem TOML walk order and entries ordering can change the selected checker. | D11:L89-L96 |
| D004-D11-054 | A | registration | + | The amendment requires a bundled reference fixture, but CheckerEntry/TOML has no fixture path, expected result, registration state, or rejection evidence schema. | D11:L84-L108 vs L346-L351 |
| D004-D11-055 | A | decision | + | Demotion cap is mandatory but its value, per-checker configuration, verdict type, aggregation integration point, and event/evidence schema are absent; 0.3 is introduced only as an example. | D11:L353-L359 |
| D004-D11-056 | A | state | + | “Pass after reformulation→Verified” and “original construction failure still caps overall verdict” leave the final claim/overall-verdict relationship unresolved. | D11:L384-L386, L397-L404 |
| D004-D11-057 | A | parsing | + | Natural-language-to-Z3 translation grammar and rejection policy are unspecified, so deterministic soundness cannot be established from the plan. | D11:L123-L128 |
| D004-D11-058 | A | failure | + | Invalid TOML, duplicate names, missing/non-executable paths, subprocess exit 1, oversized output/evidence, temporary-file cleanup, and checker crash handling have no contract. | D11:L89-L108, L110-L159 |
| D004-D11-059 | A | security/privacy | + | Live registry access has no URL/package escaping, allowlist, timeout/retry, rate-limit, provenance, privacy, or network-policy contract; Tier-C network permissions are not reconciled with mandatory live lookup. | D11:L141-L152, L213, L221 |
| D004-D11-060 | A | amendment/test | + | Both amendments postdate the original acceptance/files list but add no updated unit/integration/cloud tests, SSB variants, schema bump, or touched files for registration rejection, demotion, or reformulation. | D11:L275-L316, L330-L404 |
| D004-D11-061 | A | concurrency | + | Registry reload/mutation, concurrent checker runs, subprocess cancellation, and idempotency of repeated dispatch/reformulation are unspecified. | D11:L89-L97, L161-L165, L375-L395 |

### D-15 — Novelty classification and human review

Source shorthand D15 means D-15_novelty_class_labeling.md.

| Atomic ID | Ev. | Cat. | Δ | Explicit feature, behavior, or boundary | Anchor |
|---|---|---|---|---|---|
| D004-D15-001 | E | capability | = | Every Mode A or Mode D result receives Search, DistantSearch, or DiscoveryCandidate; candidates route to human review and the system never autonomously claims discovery. | D15:L3-L7 |
| D004-D15-002 | E | status | + | Estimated effort is one to two sessions. | D15:L8 |
| D004-D15-003 | E | dependency | = | Prerequisites are D-11 results, D-12 procedures, and C-17 DKG resilience; spec-grounder verifies them, and D-14 is declared parallel-safe. | D15:L9-L12 |
| D004-D15-004 | E | policy | + | The engineering stance permits checker-scored novel-output search but forbids autonomous discovery marketing and distinguishes checker verification from uncheckable search output. | D15:L16-L28 |
| D004-D15-005 | E | classification | Δ | Search means a near-duplicate in DKG, MistakeBook, or prior session archive. | D15:L30-L32 |
| D004-D15-006 | E | classification | Δ | DistantSearch means a novel composition whose referenced primitives are all known to the DKG. | D15:L33-L34 |
| D004-D15-007 | E | classification | Δ | DiscoveryCandidate means at least one referenced primitive or relationship is absent from the DKG and therefore requires human review. | D15:L35-L36 |
| D004-D15-008 | E | dependency | + | The classifier reuses C-17 DKG/embedding resilience, C-24 symbol_impact, MistakeBook, ArchiveChain, and D-11 CheckerResult. | D15:L38-L42 |
| D004-D15-009 | E | dependency | + | Before testing, paper-ingestor must produce FunSearch, AlphaEvolve, and Chollet ARC interface files. | D15:L53-L61 |
| D004-D15-010 | E | schema | = | NoveltyClass is a closed, L3-locked enum with Search, DistantSearch, DiscoveryCandidate. | D15:L65-L68 |
| D004-D15-011 | E | schema | Δ | NoveltyClassification carries class, DKG match count, near-duplicate boolean, known/missing primitive counts, and evidence bounded to 2048 characters. | D15:L69-L71 |
| D004-D15-012 | E | algorithm | Δ | classify first checks cosine similarity≥0.85 in DKG OR MistakeBook OR archive; otherwise all-known primitives produce DistantSearch, and any unknown primitive produces DiscoveryCandidate. | D15:L73-L81 |
| D004-D15-013 | E | workflow | Δ | After Mode A Constructive or Mode D OperationalFalsification produces CheckerResult, classify it, attach NoveltyClass to ClaimResult metadata, and emit NoveltyLabeled. | D15:L83-L87 |
| D004-D15-014 | E | workflow/artifact | Δ | DiscoveryCandidate writes workflow/sessions/{id}/discovery-queue/{claim_id}.toml and emits DiscoveryCandidateQueued; a queue file is mandatory, not SSB-only. | D15:L88-L93, L163-L168 |
| D004-D15-015 | E | schema/lifecycle | + | DiscoveryQueueEntry stores claim_id, UTC classification time, Blake3 result hash, bounded evidence, and ReviewStatus. | D15:L94-L96 |
| D004-D15-016 | E | state | Δ | ReviewStatus transitions from Pending to RatifiedHuman{reviewer,ratified_at} or RejectedHuman{reviewer,reason}. | D15:L97-L99 |
| D004-D15-017 | E | CLI/deferred | Δ | elai discovery review v1 only prints pending entries; an interactive TUI is deferred. | D15:L100-L101 |
| D004-D15-018 | E | event | Δ | SCHEMA_VERSION moves 60→61 and adds NoveltyLabeled, DiscoveryCandidateQueued, DiscoveryRatified, and DiscoveryRejected with the listed label/count/path/reviewer/reason payloads. | D15:L103-L110 |
| D004-D15-019 | E | integration/fallback | + | near_duplicate_check uses the existing knowledge embedding pipeline and top-k DKG lookup at default 0.85; if the crate cannot be called directly, the plan permits a documented stub interface, but forbids reinventing embeddings. | D15:L111-L116, L163-L167 |
| D004-D15-020 | E | unit-test | + | Unit cases require synthetic-embedding Search, known-primitive DistantSearch, unknown-primitive DiscoveryCandidate, and correct threshold behavior. | D15:L118-L122 |
| D004-D15-021 | E | integration-test | Δ | A mocked three-symbol/three-embedding DKG must exercise all classes, with the novel-primitive case creating a queue entry. | D15:L124-L129 |
| D004-D15-022 | E | decision | + | The decision record justifies three buckets as coarse but sufficient human routing, 0.85 as a false-positive/false-negative balance, and human ratification as the no-auto-discovery safety rule. | D15:L131-L141 |
| D004-D15-023 | E | config | + | Reachability names elai_cc.toml [novelty] for cosine threshold and queue path, but no concrete section is otherwise defined. | D15:L159-L162 |
| D004-D15-024 | E | failure | + | Empty DKG must yield DiscoveryCandidate when no references are known, never panic. | D15:L163-L165 |
| D004-D15-025 | E | observability | Δ | Ten mixed-novelty synthetic claims must yield ten NoveltyLabeled rows and queue TOMLs for every DiscoveryCandidate. | D15:L169-L173 |
| D004-D15-026 | E | done-gate | + | Deterministic checks inspect schema 61, four SSB identifiers, enum/classifier/helper, workspace/test results, and all three paper interfaces. | D15:L175-L188 |
| D004-D15-027 | E | acceptance | Δ | Done requires public schema 61, four event round trips, three-bucket and threshold tests, queue writes, zero new errors, at least eight novelty tests, three cloud cases, paper interfaces, and the decision doc. | D15:L190-L206 |
| D004-D15-028 | E | surface | + | Change set adds novelty.rs, integration test, decision doc, and three paper interfaces; modifies SSB, grounding_mode, and lib.rs. | D15:L208-L218 |
| D004-D15-029 | E | non-goal | Δ | Review TUI, multi-reviewer ratification, and N-reviewer auto-publish are deferred; v1 is single-reviewer and conservatively does not auto-publish. | D15:L220-L225 |
| D004-D15-030 | E | dependency | + | D-16 conformal Mode-E refusal is next. | D15:L227-L229 |
| D004-D15-031 | C | schema | ~ | The summary says NoveltyClass becomes a field on CheckerResult; the original instead attaches it to ClaimResult metadata. | D15:L83-L87 |
| D004-D15-032 | C | interface | + | classify accepts CheckerResult, but D-11 CheckerResult contains only verdict/evidence/reason and no candidate text, embedding, primitive list, or relationships needed by the heuristic. | D15:L73-L81; D11:L80-L83 |
| D004-D15-033 | C | policy/schema | + | Context requires marking novel claims with confidence, but neither NoveltyClassification nor queue/event payloads contains a confidence value. | D15:L25-L26 vs L69-L71, L94-L110 |
| D004-D15-034 | A | lifecycle | + | v1 “prints list,” yet ratified/rejected states and events require a mutation flow; no command syntax, persistence transaction, reviewer authentication, or release/publication rule is defined. | D15:L97-L110, L159-L162, L220-L225 |
| D004-D15-035 | A | config | + | [novelty] is referenced but has no defaults, lock status, loader task, file modification, or acceptance assertion. | D15:L159-L162, L208-L218 |
| D004-D15-036 | A | algorithm | + | Primitive extraction, relationship matching, top-k choice, embedding model/version, treatment of empty result content, and exact 0.85 boundary are unspecified. | D15:L73-L81, L111-L122 |
| D004-D15-037 | A | lifecycle/security | + | Queue creation has no atomic-write/idempotency/hash-verification, duplicate claim, retention, permissions, reviewer identity, privacy, or tamper policy. | D15:L88-L110, L163-L173 |
| D004-D15-038 | A | consistency | + | The helper signature searches only SymbolGraph, while the classification requirement also searches MistakeBook and ArchiveChain; adapters and similarity semantics for those stores are absent. | D15:L73-L81 vs L111-L116 |

### D-18 — Typed observable effects

Source shorthand D18 means D-18_observable_effect_typed.md.

| Atomic ID | Ev. | Cat. | Δ | Explicit feature, behavior, or boundary | Anchor |
|---|---|---|---|---|---|
| D004-D18-001 | E | capability | = | Extend C-22 FeasibilityProber with typed ObservableEffect, require at least two orthogonal effects per claim, and distinguish sandbox from production observations. | D18:L3-L8 |
| D004-D18-002 | E | status | + | Estimated effort is two sessions. | D18:L9 |
| D004-D18-003 | E | dependency | = | Prerequisites are completed C-22 FeasibilityProber and D-11 effect-verifying checkers; it is declared parallel-safe with D-17. | D18:L10-L12 |
| D004-D18-004 | E | safety | Δ | Goodhart defense requires at least two effects measuring orthogonal aspects. | D18:L21-L23 |
| D004-D18-005 | E | safety | Δ | SandboxObservation is explicitly not ProductionObservation; production grounding must never be claimed from sandbox evidence alone. | D18:L24-L25 |
| D004-D18-006 | E | observability | Δ | Confounded experiments must log environmental variables and surface known risks through confound_flags. | D18:L26-L27 |
| D004-D18-007 | E | dependency | + | Reused substrate is C-22 prober/tests/UI, D-11 CheckerRegistry, and B-18/C-11 Tier-C sandbox. | D18:L29-L32 |
| D004-D18-008 | E | dependency | + | Before testing, paper-ingestor must produce Voyager, Eureka, and Coscientist interface files. | D18:L42-L50 |
| D004-D18-009 | E | schema | Δ | ObservableEffect fields are TriggerSpec trigger, TypedObservation expected_observation, Predicate success_predicate, Duration bounded_runtime, SandboxTier, and Vec<ConfoundFlag>. | D18:L54-L58 |
| D004-D18-010 | E | schema | + | TriggerSpec holds TriggerKind plus a payload bounded to 4096; TriggerKind is Subprocess, HttpRequest, FileWrite, or SandboxedRun. | D18:L59-L60 |
| D004-D18-011 | E | schema | + | TypedObservation is StringMatch<2048>, FileContent{path<512>, optional expected Blake3}, ExitCode, or MetricValue{name, inclusive/unspecified range tuple}. | D18:L61-L64 |
| D004-D18-012 | E | schema | + | SandboxTier is closed TierA/TierB/TierC matching B-18/C-11. | D18:L65 |
| D004-D18-013 | E | schema | Δ | ConfoundFlag is ColdCache, ColdNetwork, GpuRecentlyIdle, ConcurrentTasks, or Other(String). | D18:L66-L67 |
| D004-D18-014 | E | schema | Δ | EffectSet carries effects, claim_id, and target_orthogonality, whose stated default is at least two. | D18:L69-L71 |
| D004-D18-015 | E | validation | Δ | validate_orthogonality rejects fewer effects than target and requires effects to differ on trigger kind or observation variant; same-kind effects count only once. | D18:L72-L77 |
| D004-D18-016 | E | schema | Δ | ObservationContext is Sandbox(SandboxTier) or Production. | D18:L79-L80 |
| D004-D18-017 | E | schema | Δ | ObservationResult embeds the effect, context, outcome, timing_ms, and EnvironmentSnapshot. | D18:L81-L83 |
| D004-D18-018 | E | state | + | ObservationOutcome is Confirmed, Refuted with evidence bounded to 2048, or Inconclusive with reason. | D18:L84-L85 |
| D004-D18-019 | E | schema | Δ | EnvironmentSnapshot carries hostname, hardware_class, and active_confounds. | D18:L86-L87 |
| D004-D18-020 | E | permission | Δ | Every sandbox observation preserves its tier, and promotion from sandbox evidence to a production claim requires explicit operator ratification; automatic promotion is forbidden. | D18:L88-L91 |
| D004-D18-021 | E | integration | Δ | DefaultFeasibilityProber.probe changes from one result to Vec<ObservationResult>; probe construction includes EffectSet and validates orthogonality at author time. | D18:L93-L97 |
| D004-D18-022 | E | failure/event | Δ | check_goodhart_risk returns a flag with reason “single-effect probe vulnerable to Goodhart” when effect count is below target. | D18:L99-L102 |
| D004-D18-023 | E | event | Δ | SCHEMA_VERSION moves 63→64; ObservableEffectDispatched records count/target/context, GoodhartRiskFlag records count/reason, SandboxObservation records effect ID/outcome/tier, and ProductionObservationProposal records sandbox evidence/operator_required; all round-trip. | D18:L104-L112 |
| D004-D18-024 | E | config/security | Δ | L3-locked [observable_effect] defaults are min_effects_per_claim=2, target_orthogonality=2, and auto_promote_sandbox_to_production=false, with the false default singled out as safety-critical. | D18:L114-L117 |
| D004-D18-025 | E | unit-test | + | Validation tests require one effect and two same-kind effects to error, two different-kind effects to pass, insufficient effects to flag Goodhart, and sandbox context to survive result construction. | D18:L119-L124 |
| D004-D18-026 | E | integration-test | Δ | Integration runs a one-effect flag case, a two-orthogonal-effect dispatch with two sandbox rows, and a sandbox success that emits no production claim. | D18:L126-L132 |
| D004-D18-027 | E | decision | + | Decision record justifies two effects as anti-Goodhart, explicit context as sim-to-real honesty, and operator-only production claims as protection from proxy-reward optimization. | D18:L134-L142 |
| D004-D18-028 | E | integration | Δ | The feature consumes FeasibilityProber, CheckerRegistry for individual effects, Tier C, and Mode D; it is reached from Operational Falsification and the prober entry point. | D18:L144-L163 |
| D004-D18-029 | E | invariant | + | Stub bans require validation against target_orthogonality, false auto-promotion default, a flag for single-effect probes, and persistent context labels. | D18:L164-L168 |
| D004-D18-030 | E | observability/artifact | Δ | A valid two-effect probe expects one dispatch, two sandbox rows, zero Goodhart and zero production proposal until operator action, plus workflow/sessions/{id}/observations/{claim_id}.toml with environment snapshot/confounds. | D18:L169-L175 |
| D004-D18-031 | E | done-gate | + | Deterministic checks inspect schema 64, four event names, core types/functions, config and false default, workspace/tests, and three paper interfaces. | D18:L177-L193 |
| D004-D18-032 | E | acceptance | Δ | Done requires public schema 64, event round trips, validation/context/flag/config behavior, Vec-return integration, loaded config, zero new errors, at least ten tests, three cloud smoke cases, papers, and decision doc. | D18:L195-L218 |
| D004-D18-033 | E | surface | + | Change set adds observable_effect.rs, integration test, decision doc, and three paper interfaces; modifies feasibility lib, SSB, grounding_mode, elai_cc.toml, and orchestrator lib.rs. | D18:L220-L234 |
| D004-D18-034 | E | non-goal | Δ | Operator production-claim CLI, per-domain effect templates, and automated confound detection are deferred; v1 is sandbox-only, ad-hoc authored, and manually flagged. | D18:L236-L241 |
| D004-D18-035 | E | dependency | + | The plan declares Wave 2 complete and Wave 3 responsible for benchmark/adversarial gates. | D18:L243-L246 |
| D004-D18-036 | C | workflow | + | Author-time validation says a one-effect probe cannot be constructed, yet runtime integration/cloud acceptance requires such a probe to execute and emit GoodhartRiskFlag. The reject-vs-observe path is unresolved. | D18:L93-L102 vs L126-L132, L210-L211 |
| D004-D18-037 | A | algorithm | + | For target_orthogonality above two, “differ on at least one attribute” and “same-kind count as one” do not define equivalence classes or how many independent groups satisfy the target. | D18:L69-L77 |
| D004-D18-038 | A | timeout | + | bounded_runtime is data only: no runner, timeout, cancellation, retry, or terminal Inconclusive behavior is specified. | D18:L54-L58, L93-L102 |
| D004-D18-039 | A | security/privacy | + | HttpRequest and FileWrite triggers have no host/path allowlist, credential/network policy, capability mapping, redaction, or production authorization contract. | D18:L59-L67, L88-L91 |
| D004-D18-040 | C | observability | + | The source says log all environmental variables, but EnvironmentSnapshot specifies only hostname, hardware class, and active confounds. | D18:L26-L27 vs L86-L87 |
| D004-D18-041 | A | operator-flow | + | ProductionObservationProposal and operator ratification have no callable interface, reviewer identity/audit transaction, or state transition; the required CLI is explicitly deferred. | D18:L88-L91, L104-L112, L236-L239 |
| D004-D18-042 | A | config | + | min_effects_per_claim and target_orthogonality overlap, while EffectSet also carries a target; precedence and whether lower/higher operator values are legal under L3 lock are unspecified. | D18:L69-L77, L114-L117 |
| D004-D18-043 | A | schema | + | effect_id appears in SSB but not in ObservableEffect, and observation TOML serialization, atomicity, retention, and identifier derivation are unspecified. | D18:L54-L67, L108-L109, L169-L175 |
| D004-D18-044 | A | integration | + | The plan says CheckerRegistry runs effects but gives no adapter from TriggerSpec/TypedObservation/Predicate to D-11 Claim/Candidate checker input/output. | D18:L93-L102, L156-L160 |

### D-20 — ARC-AGI-2 OOD probe

Source shorthand D20 means D-20_arc_agi2_ood_probe.md.

| Atomic ID | Ev. | Cat. | Δ | Explicit feature, behavior, or boundary | Anchor |
|---|---|---|---|---|---|
| D004-D20-001 | E | capability | = | Run a 50-task public ARC-AGI-2 sample on cloud Sonnet with mode routing, reporting solve and Mode-E refusal rates against a human baseline while forbidding DiscoveryCandidate auto-publication. | D20:L3-L10 |
| D004-D20-002 | E | status | + | Estimated effort is two to three sessions; local 7B execution is explicitly excluded. | D20:L9 |
| D004-D20-003 | E | dependency | = | Declared prerequisites are D-10 routing and D-15 novelty labeling; D-19 and D-21..D-25 are declared parallel-safe. | D20:L10-L11 |
| D004-D20-004 | E | expectation | + | Expected solve performance is single-digit to low-double-digit; any solve rate passes because the harness, not model capability, is gated. | D20:L15-L23 |
| D004-D20-005 | E | acceptance-policy | Δ | Honest-evaluation gates are D-17 healthy refusal rate, no DiscoveryCandidate auto-publish, and negligible D-14 sycophancy flags. | D20:L23-L26 |
| D004-D20-006 | E | dependency | + | No new paper ingest is required because ARC Prize and Chollet ARC interfaces are attributed to D-6/D-15. | D20:L33-L44 |
| D004-D20-007 | E | dataset | Δ | Pull 50 public ARC-AGI-2 evaluation tasks into .elai_cc/benchmarks/arc_agi2/fixtures and record dataset version plus checksum for drift detection. | D20:L48-L53 |
| D004-D20-008 | E | schema | + | ArcTask fields are task_id, training input/output grid pairs, test inputs, and test outputs. | D20:L50-L52 |
| D004-D20-009 | E | schema | Δ | ARCAGI2Runner owns fixture/output paths and Arc<dyn CloudClient>, explicitly cloud Sonnet only. | D20:L55-L59 |
| D004-D20-010 | E | schema | Δ | ARCReport carries totals/solved/refused, mode and novelty distributions, sycophancy count, solve/refusal rates, human baseline, and ARCVerdict. | D20:L60-L63 |
| D004-D20-011 | E | state | Δ | ARCVerdict is HonestlyEvaluated, RefusalRateOutOfBand, or ContaminationSuspected. | D20:L64-L65 |
| D004-D20-012 | E | workflow | Δ | For each task, construct a Claim, route it through the mode predicate, observe ClaimResult, count solved only when Verified and exact output matches expected, and aggregate distributions. | D20:L66-L70 |
| D004-D20-013 | E | CLI | + | elai bench arc-agi2 runs the harness and emits a report. | D20:L72-L74 |
| D004-D20-014 | E | exit/failure | + | CLI exits 0 only for HonestlyEvaluated regardless of solve rate and exits 1 for refusal out of band or contamination suspected. | D20:L75-L76 |
| D004-D20-015 | E | detection | Δ | Contamination heuristic flags only when solve_rate>50% and “model accessed before 6 months ago”; it is intentionally conservative. | D20:L78-L82 |
| D004-D20-016 | E | event/dependency | + | It adds no schema version and reuses BenchmarkStarted and BenchmarkResult from D-19. | D20:L84-L85 |
| D004-D20-017 | E | decision | + | Decision record justifies cloud-only as a routing-infrastructure test, no solve threshold as honest future-proofing, and 50 rather than 400 tasks for cost. | D20:L87-L95 |
| D004-D20-018 | E | integration | Δ | Runner consumes CloudClient, GroundingMode, NoveltyClass, RefusalDecision, SycophancyFlag, and SsbWriter. | D20:L97-L109 |
| D004-D20-019 | E | CI | + | The CLI is the primary reachability path; CI integration is optional because it is expensive. | D20:L110-L112 |
| D004-D20-020 | E | invariant | Δ | Runner must never auto-publish DiscoveryCandidate, must cite the human baseline and framework caveats, and must use a documented stratified 50-task sample. | D20:L113-L116 |
| D004-D20-021 | E | observability | Δ | A run emits BenchmarkStarted, per-task ModeSelected and ClaimDispatched, then BenchmarkResult with novelty/refusal/sycophancy distributions. | D20:L117-L121 |
| D004-D20-022 | E | artifact | Δ | Dated JSON reports live under .elai_cc/benchmarks/arc_agi2/reports and must contain the honest report with no candidate auto-publication. | D20:L122-L123 |
| D004-D20-023 | E | done-gate | + | Deterministic checks only inspect runner/run symbol, at least 50 JSON fixtures, CLI file presence, and workspace build. | D20:L125-L133 |
| D004-D20-024 | E | acceptance | Δ | Done requires compiled runner and synthetic ArcTask unit tests, 50 fixtures, successful cloud CLI, zero new errors, all arc tests, and full-run report with solve/refusal/mode/novelty metrics. | D20:L135-L144 |
| D004-D20-025 | E | cloud-test | Δ | Healthy refusal band is 5%-50%; auto-publishes must be zero; sycophancy count must be zero “or near-zero.” | D20:L145-L148 |
| D004-D20-026 | E | surface | + | Change set adds runner, CLI module, 50 fixtures, and decision doc, and adds SlashCommand::BenchArcAgi2 to commands.rs. | D20:L151-L157 |
| D004-D20-027 | E | non-goal | Δ | Full 400-task run, ARC-AGI-1 baseline, and synthetic ARC-style generation are deferred. | D20:L159-L164 |
| D004-D20-028 | C | fallback | ~ | The objective promises synthetic ARC-style tasks if leakage is flagged, but the implementation only returns ContaminationSuspected/exit 1 and explicitly defers synthetic generation. There is no fallback in v1. | D20:L3-L4 vs L64-L82, L163-L164 |
| D004-D20-029 | C | gate | + | The stated pass gates include no auto-publication and sycophancy hygiene, but ARCVerdict/CLI failure states cover only refusal and contamination; neither safety violation can make exit 1 as typed. | D20:L23-L26 vs L64-L76 |
| D004-D20-030 | C | dependency | + | D-14, D-16, D-17, and D-19 are runtime/event dependencies but are absent from declared prerequisites; D-19 is even called parallel-safe although its event variants are consumed. | D20:L10-L11, L23-L31, L84-L85, L103-L109 |
| D004-D20-031 | A | detection | + | “model accessed before 6 months ago” has no subject, timestamp source, comparison date, or meaning; contamination response has no alternate data/retry path. | D20:L78-L82 |
| D004-D20-032 | A | metric | + | human_baseline is one f32 despite a cited 60%-70% range; its chosen value, source/version in the report, and comparison calculation are unspecified. | D20:L5-L7, L60-L63 |
| D004-D20-033 | A | evidence | + | ARCReport has no auto-publication counter or publication audit field, so the no-publish gate cannot be proved from its declared schema alone. | D20:L60-L65, L113-L123 |
| D004-D20-034 | A | acceptance | + | “near-zero” sycophancy is unquantified and conflicts with the same bullet’s sycophancy_flag_count=0 form. | D20:L147-L148 |
| D004-D20-035 | A | reliability/privacy | + | Cloud timeout/retry/rate-limit/cost cap, partial-run resume, deterministic sampling/seed, response retention, secrets, and fixture licensing are unspecified. | D20:L48-L76, L110-L123 |
| D004-D20-036 | A | mode | + | The harness exercises Mode C by implication but does not define its handler; it only routes through the external D-10 predicate. | D20:L15-L26, L66-L70 |

### D-22 — Conformal calibration under shift

Source shorthand D22 means D-22_conformal_calibration_validation.md.

| Atomic ID | Ev. | Cat. | Δ | Explicit feature, behavior, or boundary | Anchor |
|---|---|---|---|---|---|
| D004-D22-001 | E | capability | = | Validate that D-16 empirical refusal rate remains inside [τ−ε,τ+ε] with ε=0.1 for in-domain, near-OOD, and far-OOD distributions. | D22:L3-L5, L21-L22 |
| D004-D22-002 | E | status | + | Estimated effort is one to two sessions. | D22:L6 |
| D004-D22-003 | E | dependency | = | Prerequisites are D-16 UniCR and D-6 held-out data; it is declared parallel-safe with D-19..D-25. | D22:L7-L9 |
| D004-D22-004 | E | dataset | Δ | In-domain is the calibration family; near-OOD is related but different (code→math example); far-OOD is semantically distant (code→prose/planning example). | D22:L11-L19 |
| D004-D22-005 | E | dataset | Δ | Create in_domain, near_ood, and far_ood directories with 100 hand-labeled claims each, 300 total. | D22:L32-L37 |
| D004-D22-006 | E | schema | + | Fixture labels are claim, ground_truth_available, and hand-labeled, with no further record schema given. | D22:L34-L37 |
| D004-D22-007 | E | schema | Δ | ConformalValidationRunner owns fixtures_dir, tau, and epsilon. | D22:L39-L42 |
| D004-D22-008 | E | workflow | Δ | run calls D-16 refusal_decision on every distribution, computes empirical refusal rate, compares each with [τ−ε,τ+ε], and reports per-distribution shifts. | D22:L43-L47 |
| D004-D22-009 | E | CLI | + | Add elai bench conformal-validate. | D22:L49 |
| D004-D22-010 | E | decision | + | Decision record justifies three tiers from UniCR §6, 100 samples/tier as sufficient for ε=0.1 despite the 500 calibration bound, and ε=0.1 as D-16’s default. | D22:L51-L56 |
| D004-D22-011 | E | integration/config | Δ | Runner consumes refusal_decision, τ from .elai_cc/calibration/tau.txt, and SsbWriter, and is reachable from CLI plus CI. | D22:L58-L68 |
| D004-D22-012 | E | failure | Δ | Every distribution rate must be reported and the verdict must fail if any distribution shifts by more than ε. | D22:L69-L71 |
| D004-D22-013 | E | observability | Δ | CLI run expects BenchmarkStarted, BenchmarkResult, an unnamed per-distribution rate row, and a report in .elai_cc/benchmarks/conformal/reports/. | D22:L72-L76 |
| D004-D22-014 | E | done-gate | + | Deterministic checks inspect runner presence, at least 300 fixture JSON files, and workspace build. | D22:L77-L84 |
| D004-D22-015 | E | acceptance | Δ | Done requires compiled runner, 300 hand labels, CLI exit 0 only when all three are within ε, zero new errors, and a decision doc. | D22:L86-L97 |
| D004-D22-016 | E | cloud-test | Δ | Live test uses a D-16 corpus of at least 500 tuples and requires all three empirical rates within ε=0.1; far-OOD is intentionally shifted yet must still satisfy the theorem-derived band. | D22:L92-L96 |
| D004-D22-017 | E | surface | + | Change set adds runner, CLI module, 300 fixture files, and decision doc, and adds SlashCommand::BenchConformalValidate. | D22:L99-L105 |
| D004-D22-018 | E | non-goal | Δ | Per-family/multi-domain τ is deferred; v1 has one tau. | D22:L107-L109 |
| D004-D22-019 | E | dependency | + | D-23 sycophancy benchmark is next. | D22:L111-L113 |
| D004-D22-020 | C | dataflow | ~ | The summary says the three distributions are sourced from D-6, but the implementation creates a separate benchmark fixture tree and the consumes list never names HeldOutStore. D-6 is a declared prerequisite without specified runtime wiring. | D22:L7-L8, L34-L47, L63-L66 |
| D004-D22-021 | A | schema | + | ConformalValidationReport, its verdict, per-distribution row identifier/payload, report filename/format, and CLI nonzero exit behavior are not defined. | D22:L39-L49, L69-L76, L86-L97 |
| D004-D22-022 | A | semantics | + | “distribution shifts by >ε” is not defined separately from refusal_rate outside the τ band; the statistic called shift and its sign/absolute comparison are ambiguous. | D22:L43-L47, L69-L71 |
| D004-D22-023 | A | boundary | + | The source does not say whether [τ−ε,τ+ε] is clamped to [0,1], whether endpoints are inclusive, or how missing/invalid tau is handled. | D22:L21-L22, L41-L47, L64-L65 |
| D004-D22-024 | A | statistics | + | No confidence level, interval method, seed, sampling strategy, stratification, multiple-comparison rule, or mathematical support is specified for “100 is enough for ε=0.1.” | D22:L34-L37, L51-L56 |
| D004-D22-025 | A | data | + | “hand-labeled” has no annotator/reviewer provenance, adjudication, format, privacy/licensing, ground-truth value, or quality gate. | D22:L34-L37, L88-L90 |
| D004-D22-026 | A | reliability | + | refusal_decision errors, partial runs, retry/resume, concurrent tau updates, snapshotting, and reproducibility under nightly D-16 tau changes are unspecified. | D22:L39-L47, L63-L68 |

## Cross-source interactions and end-to-end behavior

These records join only statements in the seven processed sources. They do not
assert that an omitted implementation exists.

| Atomic ID | Ev. | Interaction | Exact joined behavior | Anchors |
|---|---|---|---|---|
| D004-X-001 | E | D-7 → D-4 | Canary results become CrossDomain signal input for D-4’s Δρ/KL monitoring; D-7 explicitly says it cannot land in parallel with D-4, while D-4’s Next section names D-6-held-out baselines. | D7:L11-L13, L103-L110; D4:L86-L88, L323-L327 |
| D004-X-002 | I | candidate lifecycle | Intended order is D-7 pre-promotion veto, then promotion, then D-4 post-action monitoring and append-only revert. This is a synthesis of explicit hooks, not an atomic transaction defined by either plan. | D7:L119-L125; D4:L130-L142, L212-L214 |
| D004-X-003 | E | D-11 → D-15 | Mode-A CheckerResult is the declared classification input, and D-15 writes the novelty class to ClaimResult metadata plus SSB/queue rather than changing CheckerResult. | D11:L80-L97, L161-L165; D15:L73-L89 |
| D004-X-004 | A | D-11 ↔ D-15 | D-11’s verdict-only CheckerResult cannot supply the result embedding/primitives that D-15’s classifier needs; an unplanned result/candidate adapter is required. | D11:L80-L83; D15:L69-L81, L111-L116 |
| D004-X-005 | E | D-11 → D-18 | D-18 declares CheckerRegistry the per-effect runner and FeasibilityProber/Mode-D the entry path. | D18:L93-L102, L156-L163 |
| D004-X-006 | A | D-18 → D-15 | D-15 expects Mode-D to yield CheckerResult for classification, while D-18’s Mode-D upgrade yields Vec<ObservationResult>; no conversion or verdict aggregation is defined. | D15:L83-L87; D18:L79-L97 |
| D004-X-007 | E | D-15 → D-20 | ARC aggregates NoveltyClass and must preserve D-15’s human-only DiscoveryCandidate rule; the harness is an evidence consumer, not a publication path. | D15:L88-L110, L139-L141; D20:L60-L70, L113-L123 |
| D004-X-008 | E | D-16/D-17 → D-20/D-22 | D-20 consumes RefusalDecision and tests the 5%-50% SLA; D-22 separately calls refusal_decision against τ±ε on three distributions. | D20:L23-L31, L103-L108, L145-L146; D22:L39-L47, L63-L71 |
| D004-X-009 | C | benchmark events | D-20 reuses D-19 BenchmarkStarted/BenchmarkResult while declaring D-19 parallel-safe; D-22 expects the same generic events without naming their origin or schema. Landing/build order is incomplete. | D20:L10-L11, L84-L85; D22:L72-L75 |
| D004-X-010 | E | schema sequence | Assigned schema bumps are 49→50 (D-4), 52→53 (D-7), 56→57 (D-11), 60→61 (D-15), and 63→64 (D-18); D-20 has no bump and D-22 names no bump. The gaps belong to other planned sessions and make ordered landing material. | D4:L144-L150; D7:L103-L111; D11:L167-L172; D15:L103-L110; D18:L104-L112; D20:L84-L85 |
| D004-X-011 | I | evidence path | The intended grounded-claim path is Mode route → D-11 deterministic checker (including one bounded reformulation) or D-18 effect set → D-15 novelty label → human queue for candidates → D-20 aggregate proof. The D-11/D-15 and D-18/D-15 type gaps prevent treating this synthesis as fully specified. | D11:L161-L165, L375-L404; D18:L93-L102; D15:L83-L110; D20:L66-L70 |
| D004-X-012 | E | human authority | Two conservative authority boundaries recur: only a human may ratify DiscoveryCandidate, and only an operator may turn sandbox evidence into a production claim; both interactive mutation surfaces are deferred in v1. | D15:L88-L101, L139-L141, L220-L225; D18:L88-L91, L236-L239 |
| D004-X-013 | A | concurrency/idempotency | Across promotion veto/revert, checker reformulation, queue writes, observation writes, and nightly tau consumption, there is no shared transaction, idempotency key, locking order, snapshot rule, or crash-recovery contract beyond D-4’s Arc<RwLock<ArchiveChain>> and intent-before-retire event. | D4:L130-L142, L220-L221; D7:L119-L125; D11:L375-L395; D15:L88-L110; D18:L169-L175; D22:L63-L68 |
| D004-X-014 | E | paper gates | D-4 requires three new interfaces, D-7 one, D-11 three, D-15 three, D-18 three, while D-20 and D-22 rely on already-ingested prerequisites; all named ingest gates occur before testing. | D4:L64-L72; D7:L54-L59; D11:L68-L76; D15:L53-L61; D18:L42-L50; D20:L42-L44; D22:L24-L30 |
| D004-X-015 | A | observability naming | Sources use Rust-style SSB variants (TripwireFired, NoveltyLabeled, GoodhartRiskFlag, etc.); the summary renders several as uppercase underscore labels. No serialized-name mapping proves these are aliases. | D4:L144-L150; D15:L103-L110; D18:L104-L112 |

## Summary additions and corrections

The per-file Δ column is the exhaustive delta index: every + row is a summary
addition, every Δ row is retained headline plus required deeper detail, and
every ~ row is a correction. The following table makes the material
corrections and amendment-level additions explicit without duplicating their
atomic content.

| Delta ID | Kind | Required summary change | Deep records |
|---|---|---|---|
| D004-SD-001 | correction | Preserve Rust identifiers TripwireFired, TripwireFiredConcurrent, and TripwireAutoRevert rather than collapsing D-4 to a single TRIPWIRE_FIRED label; add event payloads and intent-before-retire ordering. | D004-D4-022..026, D004-D4-046 |
| D004-SD-002 | correction | Resolve “non-target family” versus implementation “ANY family” regression blocking; do not present the non-target rule as fully specified until reconciled. | D004-D7-001, D004-D7-020, D004-D7-037 |
| D004-SD-003 | addition | Add D-7’s Domain migration, three-valued decision, sparse-data deferral, exact fixtures/config/events, BAI pre-promotion hook, and its non-bijective Mixed gap/count conflict. | D004-D7-013..044 |
| D004-SD-004 | addition | Add all J-11/J-15 amendment behavior: register-time self-fixture fail-closed gate, demotion cap instead of averaging, Tier-C non-copy rule, and exactly one deterministic same-checker reformulation before Ungrounded. | D004-D11-044..051 |
| D004-SD-005 | correction | Do not state that D-15 adds NoveltyClass to CheckerResult; the plan attaches it to ClaimResult metadata, and the declared CheckerResult lacks classification inputs. | D004-D15-013, D004-D15-031..033 |
| D004-SD-006 | addition | Add D-15 queue file/schema/states/events, 0.85 precedence heuristic, empty-DKG fallback, conservative no-auto-publish boundary, and the fact that review mutation/TUI/config are unresolved or deferred. | D004-D15-011..029, D004-D15-034..038 |
| D004-SD-007 | addition | Add D-18 nested trigger/observation/outcome/environment types, actual orthogonality rule, operator-only production gate, exact config/events/artifact, and the one-effect reject-vs-flag conflict. | D004-D18-009..044 |
| D004-SD-008 | correction | Remove the claimed synthetic ARC fallback from v1: contamination produces ContaminationSuspected/exit 1, while synthetic generation is explicitly deferred. | D004-D20-015, D004-D20-027..028 |
| D004-SD-009 | correction | Expand D-20 prerequisites/landing order beyond D-10/D-15: it consumes D-14/D-16/D-17 and D-19 events, and its typed verdict cannot currently fail no-publish or sycophancy gates. | D004-D20-003, D004-D20-005, D004-D20-016, D004-D20-018, D004-D20-029..030 |
| D004-SD-010 | correction | Do not claim D-22’s distributions are operationally sourced from D-6; the plan builds a separate 300-file benchmark tree and never wires HeldOutStore. | D004-D22-003, D004-D22-005..008, D004-D22-020 |
| D004-SD-011 | addition | Add D-22’s tau.txt input, exact fixture counts, CLI/report/events, 500-tuple live gate, single-τ non-goal, and missing report/statistical/snapshot contracts. | D004-D22-005..026 |
| D004-SD-012 | addition | Add D-4 detector edge cases/state/config, deterministic trimming, append-only recovery evidence, false-positive cloud run, pre-D-8 inspection fallback, and unresolved signal/order/recovery details. | D004-D4-011..052 |
| D004-SD-013 | addition | Add D-11’s precise checker I/O, timeout/Inapplicable fallbacks, live anti-slopsquatting checks, isolation/sandbox requirements, first-applicable limitation, and missing ordering/registration/security/test contracts. | D004-D11-009..043, D004-D11-052..061 |
| D004-SD-014 | correction | Treat uppercase summary event names as descriptive labels unless a serializer mapping is supplied; originals define CamelCase Rust variants and several additional variants. | D004-X-015 |

## Evidence-class separation

### Explicit source facts

All E rows in the seven per-file ledgers are direct statements. They cover
capabilities/actions/surfaces; workflows/states; schemas/data/lifecycle;
flags/defaults/config; integrations/events; permissions/security; failures and
fallbacks; determinism/concurrency; observability/evidence; tests/acceptance;
constraints/non-goals; dependencies/status/deferred work. No source explicitly
defines a distinct PII handling, secret retention, telemetry export, or
regulatory-compliance feature; the only privacy findings are the documented
absences in A rows.

### Inferences

- D004-X-002 is the inferred pre-promotion-canary → promotion → post-action-tripwire lifecycle.
- D004-X-011 is the inferred end-to-end grounded-claim path.
- The implementation status is inferred as unshipped from unchecked,
  future-tense acceptance lists; “PLANNED” itself comes from the orientation
  summary, not from an implementation probe.

### Ambiguities

All A rows are unresolved contracts, not asserted bugs: D004-D4-047..051;
D004-D7-040..044; D004-D11-053..061; D004-D15-034..038;
D004-D18-037..044; D004-D20-031..036; D004-D22-021..026; and
D004-X-004, D004-X-006, D004-X-013, D004-X-015.

### Conflicts

All C rows are explicit tensions: D004-D4-046 and D004-D4-052;
D004-D7-037..039; D004-D11-052; D004-D15-031..033;
D004-D18-036 and D004-D18-040; D004-D20-028..030;
D004-D22-020; and D004-X-009. No conflict was silently resolved.

## Dependency, status, deferred, and authority roll-up

| Plan | Source-level status evidence | Direct/operational dependencies | Explicitly deferred or excluded |
|---|---|---|---|
| D-4 | Future-tense tasks and unchecked acceptance; one-session estimate | D-1+D-3 hard gate; D-5/D-6/D-8 parallel consumers/providers; D-9/D-26 later enforcement/UI | BAI algorithm, archive-verify CLI, lock enforcement, dashboard |
| D-7 | Future-tense tasks and unchecked acceptance; one-to-two-session estimate | D-6 hard gate; D-5 promotion integration; D-4 ordering constraint; D-8/D-24 downstream | D-24 suite, novelty classification, observable effects |
| D-11 | Future-tense tasks and unchecked acceptance; three-to-four-session estimate; later J-11/J-15 amendments are also requirements | D-10 hard gate; D-1/D-7 Domain; B-18/C-11 Tier C; D-12 downstream | Lean, TypeScript, schema/networkx/perf checkers, ensemble voting |
| D-15 | Future-tense tasks and unchecked acceptance; one-to-two-session estimate | D-11+D-12+C-17 hard gate; D-10 dispatch; D-1/C-21 archives; D-14 parallel; D-16 downstream | Review TUI, multi-reviewer flow, automatic publication |
| D-18 | Future-tense tasks and unchecked acceptance; two-session estimate | C-22+D-11 hard gate; D-10 Mode D; B-18/C-11 Tier C; D-17 parallel | Production ratification CLI, effect-template library, automatic confound detection |
| D-20 | Future-tense tasks and unchecked acceptance; two-to-three-session estimate | Declares D-10+D-15; actually consumes D-14/D-16/D-17 and D-19 events | 400 tasks, ARC-AGI-1, synthetic fallback/generation |
| D-22 | Future-tense tasks and unchecked acceptance; one-to-two-session estimate | Declares D-16+D-6; only D-16/tau/SsbWriter have operational wiring | Per-family/multi-domain tau |

Summary-layer status is PLANNED for all seven. The originals provide no DONE
evidence, no implementation probe, and no authorization to mark anything
shipped.

Authority/security/privacy boundaries are fully represented by the source
atoms: D-4 and D-7 make thresholds L3-locked; D-11 requires Tier-C execution
and live-registry anti-slopsquatting checks; D-15 reserves discovery
ratification for a human; D-18 reserves production claims for an operator;
D-20 forbids candidate auto-publication. None of the seven specifies a
complete PII/secret/telemetry/retention/compliance regime. The precise missing
security/privacy contracts are D004-D11-059, D004-D15-037,
D004-D18-039/D004-D18-041, D004-D20-035, and D004-D22-025.

## Zero-omission and zero-duplicate audit

### Section-to-ledger coverage

| Source | Lines read | Section coverage |
|---|---:|---|
| D-4 | 327/327 | L1-L15 objective/dependencies; L16-L50 context/foundation; L51-L73 reads/papers; L74-L194 tasks/decisions; L195-L231 wiring/evidence; L233-L268 deterministic checks; L270-L297 acceptance; L298-L327 files/non-goals/next |
| D-7 | 269/269 | L1-L14 objective/dependencies; L15-L44 context/foundation; L45-L60 reads/paper; L61-L151 tasks/decisions; L153-L185 wiring/evidence; L187-L224 deterministic checks; L226-L244 acceptance; L245-L269 files/non-goals/next |
| D-11 | 404/404 | L1-L15 objective/dependencies; L16-L59 context/foundation; L60-L77 reads/papers; L78-L199 tasks/decisions; L200-L230 wiring/evidence; L232-L273 deterministic checks; L275-L297 acceptance; L298-L329 files/non-goals/next; L330-L368 J-11 amendment; L369-L404 J-15 amendment |
| D-15 | 229/229 | L1-L13 objective/dependencies; L14-L43 context/foundation; L44-L62 reads/papers; L63-L142 tasks/decisions; L143-L173 wiring/evidence; L175-L188 deterministic checks; L190-L207 acceptance; L208-L229 files/non-goals/next |
| D-18 | 246/246 | L1-L13 objective/dependencies; L14-L33 context/foundation; L34-L51 reads/papers; L52-L143 tasks/decisions; L144-L175 wiring/evidence; L177-L193 deterministic checks; L195-L219 acceptance; L220-L246 files/non-goals/next |
| D-20 | 167/167 | L1-L12 objective/dependencies; L13-L32 context/foundation; L33-L45 reads/papers; L46-L96 tasks/decisions; L97-L123 wiring/evidence; L125-L134 deterministic checks; L135-L150 acceptance; L151-L167 files/non-goals/next |
| D-22 | 113/113 | L1-L10 objective/dependencies; L11-L23 context; L24-L31 reads; L32-L57 tasks/decisions; L58-L76 wiring/evidence; L77-L85 deterministic checks; L86-L98 acceptance; L99-L113 files/non-goals/next |

### Mechanical counts

| Source | Files | Lines | Bytes | Atomic records | E | A | C |
|---|---:|---:|---:|---:|---:|---:|---:|
| D-4 | 1 | 327 | 16650 | 52 | see total | see total | see total |
| D-7 | 1 | 269 | 12118 | 44 | see total | see total | see total |
| D-11 | 1 | 404 | 20681 | 61 | see total | see total | see total |
| D-15 | 1 | 229 | 10638 | 38 | see total | see total | see total |
| D-18 | 1 | 246 | 11835 | 44 | see total | see total | see total |
| D-20 | 1 | 167 | 7162 | 36 | see total | see total | see total |
| D-22 | 1 | 113 | 4283 | 26 | see total | see total | see total |
| **Source total** | **7** | **1755** | **83367** | **301** | **243** | **43** | **15** |

Additional joined records: 15 interactions (8 E, 2 I, 4 A, 1 C) and 14
summary-delta index records. Total uniquely defined IDs in this artifact: 330.
The 301 source records have summary disposition 15 “=”, 106 “Δ”, 175 “+”,
and 5 “~”.

Mechanical uniqueness audit:

- Each source prefix is contiguous and gap-free: D4 001-052, D7 001-044,
  D11 001-061, D15 001-038, D18 001-044, D20 001-036, D22 001-026.
- X is contiguous 001-015 and SD is contiguous 001-014.
- Duplicate defined IDs: zero.
- Missing manifest files: zero. Hash/line/byte mismatches: zero.
- Semantic duplicate audit: repeated Task/WIRING_CONTRACT/ACCEPTANCE wording
  was merged into one source atom with every applicable anchor; acceptance
  atoms retain only additional proof/done-gate semantics. Interactions and
  summary deltas cite source atoms rather than re-counting them.
- Unclassified source atoms: zero. Inferences never appear as E.
- Original sources outside DEEP-D-004 read: zero.

## Output integrity

Canonical self-verifiable output SHA-256:
1899993ceace752ef31c60f432b771cfe3312c55b2170fa6a7672338a57d8db5

Canonicalization rule: hash this complete UTF-8 file after replacing the
64 hexadecimal characters on the line immediately following “Canonical
self-verifiable output SHA-256:” with 64 ASCII zeroes. This avoids an
impossible self-referential physical-file digest while making the recorded
output checksum reproducible.
