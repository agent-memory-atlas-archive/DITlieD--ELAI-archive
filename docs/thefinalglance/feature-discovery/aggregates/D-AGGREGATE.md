# Family D aggregate — ELAI deep feature discovery

## Aggregate metadata

- Family: **D** (typed self-improvement + inference-time grounding + benchmarks + dashboards + deferred research)
- Inputs merged: DEEP-D-001, DEEP-D-002, DEEP-D-003, DEEP-D-004, and 04-D-SUMMARY_INDEX.discovery.md
- Rule: every distinct atomic record is preserved with its ID and source anchor
- Namespace: ledger-local IX/SD/AC/X/SUM/INT/ISSUE/DELTA IDs are written as `D-00N/<ID>` where needed for disambiguation
- Source-plan atoms keep their ledger IDs
- Newer-plan precedence: explicit SUPERSEDED/replacement in a source wins; else higher plan number wins **target state**; losers retained under Superseded notes
- Status labels (PLANNED/DEFERRED/partial) are plan-recorded claims unless noted; deep passes did not independently verify implementation

## Family map (session inventory)

| Wave | Sessions | Deep ledger owners |
|---|---|---|
| Wave 1 | D-1..D-9 | D-002 (D-1,D-8,D-9); D-003 (D-2,D-3); D-004 (D-4,D-7); D-001 (D-5,D-6); D-0 in D-001 |
| Wave 2 | D-10..D-18 | D-002 (D-10,D-14); D-001 (D-12); D-004 (D-11,D-15,D-18); D-003 (D-13,D-16,D-17) |
| Wave 3 | D-19..D-25 | D-003 (D-19,D-25); D-004 (D-20,D-22); D-002 (D-21,D-24); D-001 (D-23) |
| Wave 4 | D-26..D-27 | D-001 |
| Wave 5 | D-28..D-30 | D-002 (D-28); D-001 (D-29); D-003 (D-30) |
| Index | D-0 | D-001 |

---

## Ledger D-001 — source atoms

Source file: `docs/thefinalglance/feature-discovery/deep/DEEP-D-001.discovery.md`
Plans: D-0, D-5, D-6, D-12, D-23, D-26, D-27, D-29
Manifest: 8 files, 1861 lines, 89099 bytes

### D-0_INDEX.md — 68 atoms

- **D0-001 `[E][STATUS/DEP]`** The index was generated `2026-05-19` from the two named grounded companion guides; its premise is that post-C ELAI ships empirical recursive self-improvement (DGM-class) plus a five-mode inference-time-grounding substrate. (`D-0_INDEX.md:1-4`)
- **D0-002 `[E][WF/DEP]`** Session files are dependency ordered: D-1..D-9 **MUST** precede D-10..D-18; Wave 2 consumes typed templates, PPM, tripwires, archive, and L3-frozen surfaces; D-19..D-25 gate parent capabilities; D-26..D-27 surface the plan; D-28..D-30 are deferred. (`D-0_INDEX.md:6-8`)
- **D0-003 `[E][STATUS]`** Status meanings are exact: `PLANNED` = plan exists/work not started; `IN PROGRESS` = active shipping session; `DONE` = shipped + cloud-Sonnet smoke + benchmark threshold; `DEFERRED` = blocked on calibration data, hardware, or open research. (`D-0_INDEX.md:10-14`)
- **D0-004 `[E][CAP/TEST]`** Every D-N session is intended to ship one cohesive module, wire it to the dashboard, and prove it with a real cloud-Sonnet test plus a paper-baselined benchmark. (`D-0_INDEX.md:3-4`)
- **D0-005 `[E][CAP/DEP/TEST]`** D-1 plans `ReasoningTemplate` typed schema, Blake3 content hash, and archive lifecycle; closes Phase 4H.IV; requires C-15 and C-21; cites SELF-DISCOVER, arXiv:2510.04399, and DGM; proof is a promotion that writes `ARCHIVE_ENTRY_ADDED` and a verifiable hash chain. Status `PLANNED`. (`D-0_INDEX.md:18-24`)
- **D0-006 `[E][CAP/DEP/TEST]`** D-2 plans the Qwen2.5-Math-PRM-7B PPM sidecar; closes Phase 4H.V.1; requires D-1 and B-37; cites rStar-Math, Lightweight Latent Verifiers, and PRMBench; proof requires known-correct versus known-wrong score margin `>0.3`. Status `PLANNED`. (`D-0_INDEX.md:26-30`)
- **D0-007 `[E][CAP/DEP/TEST]`** D-3 plans `VerificationResult` aggregation and multi-verifier consensus; closes Phase 4H.V part 2; requires D-2; cites V-STaR, PRMBench, and Cross-Context Verification; a PPM-pass/deterministic-fail deceptive task must be rejected. Status `PLANNED`. (`D-0_INDEX.md:32-36`)
- **D0-008 `[E][CAP/DEP/TEST]`** D-4 plans KL, delta-rho, IQR, perplexity, and DDM tripwires; closes Phase 4H.VI; requires D-1/D-3; cites reward-hacking, concept-drift, and RM-overoptimization work; an injected regression must fire at least three tripwires and visibly auto-revert through the archive. Status `PLANNED`. (`D-0_INDEX.md:38-42`)
- **D0-009 `[E][CAP/DEP/TEST]`** D-5 plans Beta-posterior Thompson-sampling BAI; closes Phase 4H.VII; requires D-1/D-3; cites Audibert COLT 2010 and variance-dependent BAI; its index proof is a 50-round noisy-arm run with regret within the published Audibert bound. Status `PLANNED`. (`D-0_INDEX.md:44-48`)
- **D0-010 `[E][CAP/DEP/TEST]`** D-6 plans a time-windowed held-out store and 9-gram filter; closes Phase 4H.VIII part 1; requires D-5; cites LiveCodeBench, Soft Contamination, and ARC Prize 2025; proof auto-retires an 18-month-old uniformly passing task and emits `HELD_OUT_REFRESH` on ingest. Status `PLANNED`. (`D-0_INDEX.md:50-54`)
- **D0-011 `[E][CAP/DEP/TEST]`** D-7 plans a five-family cross-domain canary; closes Phase 4H.VIII part 2; requires D-6; cites NanoFlux; an 8% prose-reason regression must block promotion and write `CrossDomainRegression`. Status `PLANNED`. (`D-0_INDEX.md:56-60`)
- **D0-012 `[E][CAP/DEP/TEST]`** D-8 plans the cryptographic archive plus `elai archive verify`; closes Phase 4H.IX; requires D-1; cites reward-hacking checksum guidance and DGM archive practice; any tamper exits 1 with a broken-link report and clean state exits 0. Status `PLANNED`. (`D-0_INDEX.md:62-66`)
- **D0-013 `[E][SEC/DEP/TEST]`** D-9 plans L3 expansion and frozen-surface access-control predicates; closes companion sections 2.3 and 4; requires D-1..D-8; cites reward-tampering/boxed-agent/mesa-optimizer work; attempted evaluator edit must be pre-commit blocked and journaled as `LOCKED_KEYS_TOUCH`. Status `PLANNED`. (`D-0_INDEX.md:68-72`)
- **D0-014 `[E][CAP/DEP/TEST]`** D-10 plans L3-owned `select_grounding_mode`; closes Phase RPS.I; requires D-9; cites Everitt; at least 45 of 50 synthetic claims must receive the correct mode. Status `PLANNED`. (`D-0_INDEX.md:74-80`)
- **D0-015 `[E][CAP/DEP/TEST]`** D-11 plans the constructive-checker registry; closes Phase RPS.II; requires D-10; cites PoT, CodeAct, and AlphaProof; SymPy, `cargo check`, and a registry check must respectively verify math, code, and novel-package claims. Status `PLANNED`. (`D-0_INDEX.md:82-86`)
- **D0-016 `[E][WF/DEP/TEST]`** D-12 plans the `OperationalizationCheck` predicate; closes Phase RPS.III; requires D-11; cites Voyager; a claim without procedure must be auditor-blocked and emit `CLAIM_WITHOUT_PROCEDURE`. Status `PLANNED`. (`D-0_INDEX.md:88-92`)
- **D0-017 `[E][CAP/DEP/TEST]`** D-13 plans an `Adversary` role and cross-family verifier; closes Phase RPS.IV; requires D-10/D-3; cites Self-Correction Bench, latent verifiers, and small-LLM constitutional AI; cross-family disagreement on a confident-wrong claim must make consensus reject. Status `PLANNED`. (`D-0_INDEX.md:94-98`)
- **D0-018 `[E][SEC/DEP/TEST]`** D-14 plans an inverted-framing sycophancy probe; closes Phase RPS.V; requires D-13; cites Sharma et al.; accepting a user’s false assertion then diverging on inversion must fire `SYCOPHANCY_FLAG`. Status `PLANNED`. (`D-0_INDEX.md:100-104`)
- **D0-019 `[E][DATA/DEP/TEST]`** D-15 plans `NoveltyClass` labeling; closes Phase RPS.VI; requires D-11/D-12/C-17; cites FunSearch, AlphaEvolve, and ARC; proofs map known search, novel composition, and a not-in-DKG primitive to `Search`, `DistantSearch`, and `DiscoveryCandidate`. Status `PLANNED`. (`D-0_INDEX.md:106-110`)
- **D0-020 `[E][CAP/DEP/TEST]`** D-16 plans UniCR conformal refusal; closes Phase RPS.VII; requires D-9/D-10; cites UniCR, uncertainty, and doubt-reward work; a shifted held-out family must preserve empirical refusal within `epsilon=0.1`. Status `PLANNED`. (`D-0_INDEX.md:112-116`)
- **D0-021 `[E][OBS/DEP/TEST]`** D-17 plans honest-refusal SLA monitoring; closes Phase RPS.VIII; requires D-16; cites the RPS design and honesty survey; a synthetic session with refusal rate below 2% must emit `AUTO_FLAG`. Status `PLANNED`. (`D-0_INDEX.md:118-122`)
- **D0-022 `[E][DATA/DEP/TEST]`** D-18 plans typed `ObservableEffect` for Mode D; closes the Phase 4F extension; requires C-22/D-11; cites Voyager, Eureka, and Coscientist; one effect fires `GOODHART_RISK_FLAG`, while two or more suppress it. Status `PLANNED`. (`D-0_INDEX.md:124-128`)
- **D0-023 `[E][TEST/DEP]`** D-19 plans PRMBench adversarial-gaming gates for the PPM; closes companion section 1.13; requires D-2/D-3; cites PRMBench, Lessons of Developing PRMs, and ThinkPRM; required floors are Deception Resistance `>=35%` and Circular Logic `>=30%`. Status `PLANNED`. (`D-0_INDEX.md:130-136`)
- **D0-024 `[E][TEST/DEP/SEC]`** D-20 plans an ARC-AGI-2 OOD harness; closes companion section 1.16 and real-problem section 1 Mode C; requires D-10/D-15; cites Chollet ARC, ARC Prize 2025, and NVARC; reports solve and refusal rates against human baseline on 50 tasks and must not auto-publish `DiscoveryCandidate`. Status `PLANNED`. (`D-0_INDEX.md:138-142`)
- **D0-025 `[E][TEST/DEP]`** D-21 plans a mode-routing coverage benchmark that closes D-10 gating; requires D-10; cites real-problem section 3.2 and Self-Correction Bench; gate is F1 `>=0.85` across five modes with no class collapse. Status `PLANNED`. (`D-0_INDEX.md:144-148`)
- **D0-026 `[E][TEST/DEP]`** D-22 plans conformal-calibration validation under distribution shift that closes D-16 gating; requires D-16/D-6; cites UniCR; every in-domain, near-OOD, and far-OOD refusal rate must lie within `[tau-epsilon, tau+epsilon]`. Status `PLANNED`. (`D-0_INDEX.md:150-154`)
- **D0-027 `[E][TEST/DEP]`** D-23 plans the sycophancy-detection benchmark that closes D-14 gating; requires D-14; cites Sharma; gate is detection `>=70%` and false-positive rate `<=15%`. Status `PLANNED`. (`D-0_INDEX.md:156-160`)
- **D0-028 `[E][TEST/DEP]`** D-24 plans canary regression validation that closes D-7 gating; requires D-7; cites NanoFlux; it must block 20/20 harmful mutations while blocking at most 5/50 helpful and 2/30 neutral mutations. Status `PLANNED`. (`D-0_INDEX.md:162-166`)
- **D0-029 `[E][DET/TEST/DEP]`** D-25 plans determinism plus archive-tamper testing that closes D-8+C-27 gating; requires D-8/C-27; cites the reward-hacking source; all 10 tamper scenarios must be detected and clean state must exit 0. Status `PLANNED`. (`D-0_INDEX.md:168-172`)
- **D0-030 `[E][CAP/DEP/TEST]`** D-26 plans and closes the reasoning-template/archive dashboard surface for D-1..D-8; requires D-1..D-8; cites companion sections 3.1 and 3.8; a live promotion must add an archive-panel entry and lineage edge, and a tripwire injection must show red. Status `PLANNED`. (`D-0_INDEX.md:174-180`)
- **D0-031 `[E][CAP/DEP/TEST]`** D-27 plans and closes mode-routing/refusal dashboard coverage for D-10..D-17; requires D-10..D-17; cites real-problem sections 3.2 and 4.6; live mixed claims must show realistic routing and a sycophancy probe must reach `SycophancyFlagStrip`. Status `PLANNED`. (`D-0_INDEX.md:182-186`)
- **D0-032 `[E][STATUS/DEP/TEST]`** D-28 adversarial-example farming is `DEFERRED`; closes Phase 4H.X; requires D-5, D-19, and RunPod; cites Adversarial Training Reward Models; valid proof is pre/post PRMBench Deception Resistance delta `>=+5pp`. (`D-0_INDEX.md:188-194`)
- **D0-033 `[E][STATUS/DEP/TEST]`** D-29 hierarchical BAI is `DEFERRED`; closes companion open Q4; requires D-5 and RunPod; cites Audibert and variance-dependent BAI; closure is hierarchical regret no worse than flat BAI or an infeasibility note with at least one concrete blocker. (`D-0_INDEX.md:196-200`)
- **D0-034 `[E][STATUS/DEP/TEST]`** D-30 calibration-corpus bootstrap is `DEFERRED`; closes real-problem open Q5; requires D-16 and corpus access; cites UniCR; bootstrap must stay within epsilon and later real-session calibration must have risk no worse than bootstrap. (`D-0_INDEX.md:202-206`)
- **D0-035 `[E][OBS/STATUS]`** Per-session status notes are cumulative; shipped notes use `SHIPPED <date>: <artifact + benchmark>`, and schema-version/SSB additions are logged for B-plan reconciliation. (`D-0_INDEX.md:208-212`)
- **D0-036 `[E][DATA/INT/STATUS]`** D-1 is `PLANNED`, expects schema `47 -> 48`, and events `ARCHIVE_ENTRY_ADDED`, `ARCHIVE_ENTRY_RETIRED`, `TEMPLATE_PROMOTED`, `TEMPLATE_CANDIDATE_REJECTED`. (`D-0_INDEX.md:214-217`)
- **D0-037 `[E][CFG/INT/STATUS]`** D-2 is `PLANNED`; PPM port is `8101`, policy SLM port `8100`, with policy on GPU0 and PPM on GPU1. (`D-0_INDEX.md:219-221`)
- **D0-038 `[E][INT/STATUS]`** D-3 is `PLANNED`; expected events are `VERIFIER_CONSENSUS_PASS`, `VERIFIER_CONSENSUS_REJECT`, and `VERIFIER_DISAGREEMENT`. (`D-0_INDEX.md:223-225`)
- **D0-039 `[E][DATA/INT/STATUS]`** D-4 is `PLANNED`; its schema bump is expected atomically with D-3, and `TRIPWIRE_FIRED` carries kinds `KlDivergence`, `CorrelationDrop`, `IqrOutlier`, `PerplexityAnomaly`, or `Ddm`. (`D-0_INDEX.md:227-230`)
- **D0-040 `[E][INT/STATUS]`** D-5 is `PLANNED`; expected files are `orchestrator/src/bai.rs` and `promotion_arms.rs`; events are `ARM_POSTERIOR_UPDATED`, `THOMPSON_SAMPLED`, `BAI_PROMOTION_DECISION`. (`D-0_INDEX.md:232-235`)
- **D0-041 `[E][DATA/INT/STATUS]`** D-6 is `PLANNED`; expected append-only layout is `.elai_cc/held-out/{date}/{task-id}.toml`; events are `HELD_OUT_REFRESH` and `HELD_OUT_AUTO_RETIRE`. (`D-0_INDEX.md:237-240`)
- **D0-042 `[E][INT/STATUS]`** D-7 is `PLANNED`; events are `CROSS_DOMAIN_REGRESSION` and `CANARY_FAMILY_PASS`. (`D-0_INDEX.md:242-244`)
- **D0-043 `[E][CAP/STATUS]`** D-8 is `PLANNED`; expected CLI is `elai archive verify`, backed by new `archive_chain.rs` and `archive_verify.rs`. (`D-0_INDEX.md:246-249`)
- **D0-044 `[E][SEC/INT/STATUS]`** D-9 is `PLANNED`; expected hook is `scripts/locked_keys_check.sh`; events are `LOCKED_KEYS_TOUCH` and `L3_OVERRIDE_REQUESTED`. (`D-0_INDEX.md:251-254`)
- **D0-045 `[E][DATA/INT/STATUS]`** D-10 is `PLANNED`; schema bump is expected atomically with D-11/D-12; event is `MODE_SELECTED`. (`D-0_INDEX.md:256-259`)
- **D0-046 `[E][CAP/INT/STATUS]`** D-11 is `PLANNED`; six initial checker families live at `.elai_cc/checkers/{math,code,text,graph,api,perf}/`; events are `CHECKER_DISPATCHED`, `CHECKER_PASS`, `CHECKER_FAIL`, `CHECKER_INAPPLICABLE`. (`D-0_INDEX.md:261-264`)
- **D0-047 `[E][DATA/INT/STATUS]`** D-12 is `PLANNED`; it expects BabyStepSpec block **9** `MEASURABLE_PROCEDURE` and events `CLAIM_WITHOUT_PROCEDURE` and `PROCEDURE_EXECUTED`. (`D-0_INDEX.md:266-269`)
- **D0-048 `[E][DATA/INT/STATUS]`** D-13 is `PLANNED`; adds `AgentRole::Adversary` and events `ADVERSARY_VERDICT`, `CROSS_FAMILY_UNAVAILABLE`. (`D-0_INDEX.md:271-274`)
- **D0-049 `[E][INT/STATUS]`** D-14 is `PLANNED`; events are `SYCOPHANCY_PROBE` and `SYCOPHANCY_FLAG`. (`D-0_INDEX.md:276-278`)
- **D0-050 `[E][DATA/INT/STATUS]`** D-15 is `PLANNED`; exact enum values are `Search`, `DistantSearch`, `DiscoveryCandidate`; events are `NOVELTY_LABELED`, `DISCOVERY_CANDIDATE_QUEUED`. (`D-0_INDEX.md:280-283`)
- **D0-051 `[E][DATA/INT/STATUS]`** D-16 is `PLANNED`; expected calibration directory is `.elai_cc/calibration/`; events are `REFUSAL_DECISION`, `CALIBRATION_UPDATE`. (`D-0_INDEX.md:285-288`)
- **D0-052 `[E][OBS/INT/STATUS]`** D-17 is `PLANNED`; D-27 supplies `RefusalRateGauge.svelte`; event is `REFUSAL_SLA_BREACH`. (`D-0_INDEX.md:290-293`)
- **D0-053 `[E][DATA/INT/STATUS]`** D-18 is `PLANNED`; expected `ObservableEffect` lives in `orchestrator/src/observable_effect.rs`; event is `GOODHART_RISK_FLAG`. (`D-0_INDEX.md:295-298`)
- **D0-054 `[E][TEST/INT/STATUS]`** D-19 is `PLANNED`; paper-ingestor artifact is `workflow/papers/prmbench-2501.03124-interface.md`; baseline comes from PRMBench Table 3. (`D-0_INDEX.md:300-303`)
- **D0-055 `[E][TEST/INT/STATUS]`** D-20 is `PLANNED`; harness is `.elai_cc/benchmarks/arc_agi2/`; stated baselines are human `60-70%` and NVARC `24%`. (`D-0_INDEX.md:305-308`)
- **D0-056 `[E][TEST/STATUS]`** D-21 is `PLANNED`; expected fixture is `.elai_cc/benchmarks/mode_routing/{200 synthetic claims}`. (`D-0_INDEX.md:310-312`)
- **D0-057 `[E][TEST/STATUS]`** D-22 is `PLANNED`; expected fixture partitions are `.elai_cc/benchmarks/conformal/{in_domain,near_ood,far_ood}/`. (`D-0_INDEX.md:314-316`)
- **D0-058 `[E][TEST/STATUS]`** D-23 is `PLANNED`; expected fixture is `.elai_cc/benchmarks/sycophancy/` following Sharma probes. (`D-0_INDEX.md:318-320`)
- **D0-059 `[E][TEST/STATUS]`** D-24 is `PLANNED`; fixture is `.elai_cc/benchmarks/canary/{50 helpful, 30 neutral, 20 harmful}`. (`D-0_INDEX.md:322-324`)
- **D0-060 `[E][DET/TEST/STATUS]`** D-25 is `PLANNED`; fixture is `.elai_cc/benchmarks/tamper/{10 scenarios}`. (`D-0_INDEX.md:326-328`)
- **D0-061 `[E][CAP/STATUS]`** D-26 is `PLANNED`; expected components are `MutationArchivePanel.svelte` and `LineageGraphView.svelte` (Cytoscape); expected commands are `get_archive_recent`, `get_archive_verify_status`, `get_tripwire_status`. (`D-0_INDEX.md:330-333`)
- **D0-062 `[E][CAP/STATUS]`** D-27 is `PLANNED`; expected components are `ModeRoutingPanel.svelte`, `RefusalRateGauge.svelte`, `NoveltyBreakdown.svelte`, `SycophancyFlagStrip.svelte`; expected commands are `get_mode_breakdown`, `get_refusal_rate`, `get_novelty_breakdown`, `get_sycophancy_flags`. (`D-0_INDEX.md:335-338`)
- **D0-063 `[E][STATUS/DEP]`** D-28 remains deferred until at least 20 live D-5 promotion cycles exist and RunPod retraining is available. (`D-0_INDEX.md:340-341`)
- **D0-064 `[E][STATUS]`** D-29 is marked deferred because it is research-grade; an infeasibility note is explicitly valid closure. (`D-0_INDEX.md:343-344`)
- **D0-065 `[E][STATUS/DEP]`** D-30 is deferred until D-16 has empirical signal and either 500 calibration tuples or generic-corpus access exists. (`D-0_INDEX.md:346-347`)
- **D0-066 `[E][STATUS/OBS]`** Index totals are 30 slots, 27 in scope, 3 deferred, all 30 authored on 2026-05-19 and `PLANNED` as files; anticipated totals are 4-6 atomic schema bumps, about 35-40 new SSB variants, 160-220 CPU/cloud hours, and 25-75 deferred hours plus RunPod. (`D-0_INDEX.md:349-360`)
- **D0-067 `[E][SEC/DEP/FAIL]`** Every D-N `WIRING_CONTRACT.consumes_symbols` must reconcile against canonical schema/event definitions at `EXECUTING`; any missing B/C dependency blocks the FSM transition out of `PLANNING`. (`D-0_INDEX.md:362-364`)
- **D0-068 `[E][OBS/CAP]`** D-26 extends `.claude/tools/bn_status_report.py` with `--plan=D` so dashboard progress includes D alongside B/C; this is the only exact cross-plan progress-overlay hook stated in the index. (`D-0_INDEX.md:366-370`)

### D-5_bai_thompson_promotion.md — 45 atoms

- **D5-001 `[E][CAP]`** D-5 replaces MSI sigmoid-Boltzmann promotion with Beta-posterior Thompson sampling plus Successive Rejects and LUCB for template promotion/retirement, claiming a strictly stronger selection axis than DGM’s heuristic. (`D-5_bai_thompson_promotion.md:1-8`)
- **D5-002 `[E][DEP/STATUS]`** Estimated effort is two sessions; D-1 archive entries and D-3 consensus pass counts must be `DONE`, verified by `spec-grounder`. (`D-5_bai_thompson_promotion.md:8-10`)
- **D5-003 `[E][DET/CON]`** Work is parallel-safe with D-6/D-7 because they supply signals without writing held-out tasks, but not with D-4 because D-4 and BAI mutate the same `ArchiveChain`; coordination is through archive locks. (`D-5_bai_thompson_promotion.md:11-14`)
- **D5-004 `[E][INT]`** Theory grounding is Audibert Successive Rejects/LUCB plus variance-aware BAI; DGM/STaR/ReST^EM/current ELAI are stated not to use explicit Beta-posterior Thompson selection. (`D-5_bai_thompson_promotion.md:16-23`)
- **D5-005 `[E][DATA]`** Every candidate `ReasoningTemplate` owns an `ArmPosterior` that tracks `pass_count` and `fail_count`. (`D-5_bai_thompson_promotion.md:25-27`)
- **D5-006 `[E][DATA]`** The posterior is exactly `Beta(pass_count + 1, fail_count + 1)`. (`D-5_bai_thompson_promotion.md:28-29`)
- **D5-007 `[E][WF]`** Thompson selection samples one theta per arm, picks the largest, and advances evaluation through 10, 50, and 200-evaluation stages. (`D-5_bai_thompson_promotion.md:30-32`)
- **D5-008 `[E][WF/CFG]`** Candidate-to-Promoted requires the 95% credible-interval lower bound to exceed the current promoted median plus a domain delta, typically 0.02-0.05 absolute pass rate. (`D-5_bai_thompson_promotion.md:33-35`)
- **D5-009 `[E][WF]`** Successive Rejects eliminates the empirically worst arm each round. (`D-5_bai_thompson_promotion.md:36`)
- **D5-010 `[E][WF]`** LUCB pulls the highest-UCB competitor against the empirical best to balance exploration/exploitation. (`D-5_bai_thompson_promotion.md:37-38`)
- **D5-011 `[E][DEP/DATA]`** Existing foundations are D-1 staged `stage_10/50/200_pass/total` fields, D-3 binary consensus, and `RetirementReason::Superseded`. (`D-5_bai_thompson_promotion.md:40-45`)
- **D5-012 `[E][DEP]`** Required reads are the companion BAI sections, rStar-Math/statistical-limits interfaces, and D-1/D-3 archive/template/verification implementation surfaces. (`D-5_bai_thompson_promotion.md:47-58`)
- **D5-013 `[E][INT/DEP]`** Before implementation, `paper-ingestor` must create and verify `audibert-colt-2010-bai-interface.md` and `variance-bai-2106.10417-interface.md`. (`D-5_bai_thompson_promotion.md:60-67`)
- **D5-014 `[E][DATA/CAP]`** New `ArmPosterior` fields are `template_id`, `pass_count`, `fail_count`, `last_updated`; `alpha=pass_count+1`, `beta=fail_count+1`, `mean=alpha/(alpha+beta)`, and `credible_interval(alpha_level)` returns lower/upper bounds. (`D-5_bai_thompson_promotion.md:69-79`)
- **D5-015 `[E][SEC/FAIL]`** `statrs::distribution::Beta` may be used only if already available; otherwise the agent must ask before adding it, and the dependency-free fallback is a hand-rolled incomplete-beta function under 40 LOC. (`D-5_bai_thompson_promotion.md:77-81`)
- **D5-016 `[E][DET/DATA]`** `ArmPosterior::update(passed)` returns a new value using an immutable-update pattern. (`D-5_bai_thompson_promotion.md:82-83`)
- **D5-017 `[E][DET/CAP]`** `sample_beta(&mut SeededRng, alpha, beta)` uses the gamma-ratio method and C-15 `SeededRng`, not ambient randomness. (`D-5_bai_thompson_promotion.md:85-88`)
- **D5-018 `[E][FAIL/CAP]`** `thompson_select` samples every arm, returns the maximum-theta `TemplateId`, and returns `None` for empty input. (`D-5_bai_thompson_promotion.md:89-91`)
- **D5-019 `[E][CAP]`** `successive_rejects(Vec<ArmPosterior>, eval_budget)` performs round elimination by mean, assigns `n_k = budget / (K-k)` pulls to survivors, and returns ordered survivor IDs. (`D-5_bai_thompson_promotion.md:93-97`)
- **D5-020 `[E][CAP]`** `lucb1_next_pull(arms, delta) -> Option<TemplateId>` computes `mean_i + sqrt(log(1/delta)/(2*n_i))` and is directed to return the highest-UCB competitor; unlike `thompson_select`, the source does not state the condition that yields `None`. (`D-5_bai_thompson_promotion.md:99-103`)
- **D5-021 `[E][DATA]`** `BaiPromotionEngine` owns arm/posterior maps, promoted/retired ID lists, evaluation budget, per-domain deltas, and a `SeededRng`. (`D-5_bai_thompson_promotion.md:105-108`)
- **D5-022 `[E][WF]`** `ingest_eval_result(template_id, passed)` updates the target posterior. (`D-5_bai_thompson_promotion.md:109`)
- **D5-023 `[E][WF/OBS]`** `promote_round(stage)` applies the credible-bound rule, emits one `BaiPromotionDecision` row per arm, transitions `Candidate -> Promoted`, and appends the promotion event to `ArchiveChain`. (`D-5_bai_thompson_promotion.md:110-113`)
- **D5-024 `[E][WF]`** `retire_round(stage)` applies Successive Rejects to the bottom quartile, marks losers `Superseded`, and calls `ArchiveChain.retire`. (`D-5_bai_thompson_promotion.md:114-116`)
- **D5-025 `[E][DATA]`** Exact `PromotionStage` variants are `Stage10`, `Stage50`, and `Stage200`. (`D-5_bai_thompson_promotion.md:117`)
- **D5-026 `[E][DATA/INT]`** Schema bumps `50 -> 51`; exact payloads are `ArmPosteriorUpdated {template_id, pass_count, fail_count, mean, ci_lower, ci_upper}`, `ThompsonSampled {round_id, sampled_arms, selected}`, and `BaiPromotionDecision {template_id, decision, stage, evidence_pass, evidence_total, audit_id}`, all with round-trip tests. (`D-5_bai_thompson_promotion.md:119-126`)
- **D5-027 `[E][CFG/SEC]`** L3-locked `[bai]` defaults are `eval_budget=200`, `ci_alpha=0.05`, `delta_code=0.05`, `delta_math=0.03`, `delta_prose=0.05`, `delta_mixed=0.04`. (`D-5_bai_thompson_promotion.md:128-132`)
- **D5-028 `[E][DET]`** All sampling must use `SeededRng`; identical inputs and `ELAI_SEED` must yield byte-identical promotion decisions, demonstrated with two seed-42 runs over a 20-arm stream. (`D-5_bai_thompson_promotion.md:134-137`)
- **D5-029 `[E][TEST]`** Posterior unit gates: means are 0.5 for `(0,0)`, approximately 0.917 for `(10,0)`, approximately 0.083 for `(0,10)`; `(50,50)` CI is symmetric and `(95,5)` is high-skewed. (`D-5_bai_thompson_promotion.md:139-143`)
- **D5-030 `[E][DET/TEST]`** `sample_beta` and `thompson_select` must be deterministic under a fixed seed. (`D-5_bai_thompson_promotion.md:144-145`)
- **D5-031 `[E][TEST]`** Successive Rejects must show the stated 4/3/2/1 survivor allocation for four arms/budget 12; LUCB must pick the wider-confidence competitor rather than the empirical best. (`D-5_bai_thompson_promotion.md:146-149`)
- **D5-032 `[E][TEST]`** A five-arm, 100-step synthetic engine test with one 0.9 arm and four 0.5 arms must promote the best within budget and reproduce under the same seed. (`D-5_bai_thompson_promotion.md:150-152`)
- **D5-033 `[E][TEST]`** Integration uses ten rates `[0.9,0.85,0.7,0.65,0.5,0.5,0.4,0.3,0.2,0.1]`, 500 pulls, 100 seeds; regret must be at most twice the conservative Audibert `O(sqrt(K*budget*log(budget)))` bound and best-arm selection at least 85%. (`D-5_bai_thompson_promotion.md:154-161`)
- **D5-034 `[E][CON]`** The decision record must justify binary Beta-Bernoulli (consensus is Pass/Reject), per-domain deltas (different noise floors), and retaining MSI only as an operator-visible secondary signal, never the gate. (`D-5_bai_thompson_promotion.md:163-170`)
- **D5-035 `[E][INT]`** `WIRING_CONTRACT.produces_symbols` includes every posterior/algorithm/engine/stage symbol, the three SSB variants/payloads, schema 51, and `[bai]`. (`D-5_bai_thompson_promotion.md:172-181`)
- **D5-036 `[E][DEP/INT]`** Consumers/reachability are D-1 `ArchiveChain` and template status, D-3 consensus count, C-15/D-1 `SeededRng`, D-1 staged fields, the task-runner after-eval hook, `[bai]`, and `ELAI_SEED`. (`D-5_bai_thompson_promotion.md:182-191`)
- **D5-037 `[E][FAIL/DET/SEC]`** Stub bans require no `thread_rng`, graceful empty-arm return, `(0,1)` CI for the uniform zero-sample prior rather than NaN, and config-loaded deltas that the agent cannot edit. (`D-5_bai_thompson_promotion.md:192-199`)
- **D5-038 `[E][OBS/DET]`** A seeded ten-arm benchmark must produce at least 100 `ArmPosteriorUpdated` rows, at least one decision per survivor, identical same-seed SSB diffs, and a regret assertion artifact. (`D-5_bai_thompson_promotion.md:199-205`)
- **D5-039 `[E][TEST]`** Deterministic checks assert schema 51, presence/count of all events and algorithms, `SeededRng` with zero `thread_rng|rand::random`, `[bai]`, workspace/scoped builds, and both paper interfaces. (`D-5_bai_thompson_promotion.md:207-238`)
- **D5-040 `[E][TEST]`** Acceptance requires public schema 51, correct posterior math, deterministic sampling/selection, Audibert allocation, correct LUCB choice, and the one-good/four-inferior engine outcome with inferior arms `Superseded`. (`D-5_bai_thompson_promotion.md:240-250`)
- **D5-041 `[E][TEST]`** Acceptance also requires 500-pull/100-trial regret and 85% selection gates, three event round trips, loaded defaults, zero new workspace errors, and at least 15 passing BAI tests. (`D-5_bai_thompson_promotion.md:251-256`)
- **D5-042 `[E][TEST/OBS]`** Cloud A is a 50-cycle bootstrap with posterior/decision/archive-transition evidence; Cloud B repeats identical seed-42 tasks and permits only timestamps to differ while BAI rows remain byte-identical; paper interfaces and the three-decision document are mandatory. (`D-5_bai_thompson_promotion.md:257-264`)
- **D5-043 `[E][CAP/INT]`** File effects are new `bai.rs`, two integration/contract tests, decision/paper interfaces, plus edits to `ssb.rs`, `task_runner.rs`, config, and `lib.rs`; `statrs` is conditional on user approval. (`D-5_bai_thompson_promotion.md:266-280`)
- **D5-044 `[E][CON/STATUS]`** Hierarchical BAI, adversarial-example farming, and reward-model retraining are outside D-5; the first two are explicitly deferred. (`D-5_bai_thompson_promotion.md:282-286`)
- **D5-045 `[E][DEP/WF]`** D-6/D-7 later feed per-arm pass/fail signals; D-8 exposes promotion lineage. (`D-5_bai_thompson_promotion.md:288-292`)

### D-6_held_out_task_store.md — 52 atoms

- **D6-001 `[E][CAP/WF]`** D-6 maintains `.elai_cc/held-out/{date}/{task-id}.toml`, applies a 9-gram contamination filter, retires old uniformly passing tasks, and blocks template promotion when held-out regression exceeds suggested `k=5%`. (`D-6_held_out_task_store.md:1-7`)
- **D6-002 `[E][DEP/DET]`** Estimated effort is two sessions; D-5 must be `DONE`; D-7 is parallel-safe, while D-4 is not because both touch held-out/archive evaluation state. (`D-6_held_out_task_store.md:8-13`)
- **D6-003 `[E][WF/INT]`** The LiveCodeBench pattern is date tagging, N-session refresh, retirement after six months plus uniform passing, and intake only from recency-verifiable GitHub issues or new arXiv reproducibility tasks. (`D-6_held_out_task_store.md:15-27`)
- **D6-004 `[E][SEC/CON]`** Because 9-grams miss near-equivalent training examples, the design adds time-window plus semantic-fingerprint defense using rolling embedding similarity to recent ingest dates. (`D-6_held_out_task_store.md:28-31`)
- **D6-005 `[E][SEC/CON]`** ARC’s structural prior can itself leak; the two stated defenses are time windows and structural-prior randomization. (`D-6_held_out_task_store.md:33-37`)
- **D6-006 `[E][DEP/DATA]`** Reused foundations are C-17 tokenization/chunking for content hashing, D-1 `ArchiveChain` for task-retirement evidence whose payload carries `task_id` rather than `template_id`, and C-15 `SeededClock` for ingest dates. (`D-6_held_out_task_store.md:39-45`)
- **D6-007 `[E][DEP]`** Required implementation reads are the companion, statistical-limits interface, DKG types only as a ruled-out D-15 concern, `archive_chain.rs`, and `clock.rs`. (`D-6_held_out_task_store.md:47-57`)
- **D6-008 `[E][INT/DEP]`** `paper-ingestor` must create/verify LiveCodeBench, Soft Contamination, and ARC Prize 2025 interfaces. (`D-6_held_out_task_store.md:58-66`)
- **D6-009 `[E][DATA]`** `HeldOutTask` fields are `task_id`, `submission_date`, `domain`, bounded 4096-char problem statement, `expected_outcome`, `source`, Blake3 `content_hash`, and `status`. (`D-6_held_out_task_store.md:68-75`)
- **D6-010 `[E][DATA]`** `ExpectedOutcome` variants and payloads are `Code{test_cmd}`, `Math{sympy_expr}`, `Prose{rubric<=1024}`, `Planning{final_state<=2048}`, and `KnowledgeQa{reference_answer<=2048}`. (`D-6_held_out_task_store.md:76-78`)
- **D6-011 `[E][DATA]`** `TaskSource` variants are `GitHubIssue{repo,issue_num,fetched_at}`, `ArxivRepro{arxiv_id,fetched_at}`, `Synthetic{generator_version}`, and `HumanCurated{curator}`. (`D-6_held_out_task_store.md:79-81`)
- **D6-012 `[E][DATA/WF]`** `HeldOutStatus` is `Active`, `AutoRetired{reason}`, or `ManuallyExcluded`; retirement reasons are `UniformPassSuspicion`, `NgramContamination`, `SemanticContamination`, `AgeExceedsTtl`; `TaskId` wraps `Uuid`. (`D-6_held_out_task_store.md:82-84`)
- **D6-013 `[E][DATA/CAP]`** `HeldOutStore` owns `root_path`, ID index, and date index; `load` walks dated TOML files, parses tasks, and rebuilds both indexes. (`D-6_held_out_task_store.md:86-91`)
- **D6-014 `[E][WF]`** `append` writes `root/{date}/{task_id}.toml` and updates the in-memory index. (`D-6_held_out_task_store.md:92-93`)
- **D6-015 `[E][WF/OBS]`** `retire` changes the status, rewrites the TOML file, and emits `HeldOutAutoRetire`. (`D-6_held_out_task_store.md:94-95`)
- **D6-016 `[E][CAP]`** `query(domain,since)` filters by optional domain/date and returns active tasks only. (`D-6_held_out_task_store.md:96-97`)
- **D6-017 `[E][DATA/FAIL]`** `extract_ngrams(text,n)` lowercases, tokenizes on whitespace/punctuation, and returns sliding n-grams; it must return an empty set for empty input. (`D-6_held_out_task_store.md:99-103,214-216`)
- **D6-018 `[E][DATA]`** `ngram_overlap` is Jaccard `intersection/union` over n-gram sets. (`D-6_held_out_task_store.md:104-105`)
- **D6-019 `[E][SEC/CFG]`** `detect_ngram_contamination` hashes each 9-gram with Blake3, probes `.elai_cc/held-out/ngram-bloom.bin`, and treats Jaccard `>0.3` as contamination. (`D-6_held_out_task_store.md:106-109`)
- **D6-020 `[E][DATA/CFG]`** `AutoRefreshPolicy` fields are TTL months, uniform-pass threshold, and minimum evaluations; defaults are 6, 0.95, and 20. (`D-6_held_out_task_store.md:111-114`)
- **D6-021 `[E][WF]`** Auto-retirement requires all three strict conditions: age greater than TTL, pass rate greater than threshold, and evaluations at least the configured minimum; reason is `UniformPassSuspicion`. (`D-6_held_out_task_store.md:115-120`)
- **D6-022 `[E][CAP]`** `held_out_regression` compares baseline/candidate pass rates on their shared held-out subset and returns signed percentage delta, positive for improvement and negative for regression. (`D-6_held_out_task_store.md:122-126`)
- **D6-023 `[E][WF/FAIL]`** `gate_promotion(delta,threshold)` allows iff `delta >= -threshold` and returns a reason string containing the measured delta. (`D-6_held_out_task_store.md:127-129`)
- **D6-024 `[E][DATA/INT]`** Schema bumps `51 -> 52`; payloads are `HeldOutRefresh{date,action,task_count_added,task_count_retired}`, `HeldOutAutoRetire{task_id,reason,age_days,uniform_pass_rate}`, and `HeldOutPromotionGate{template_id,delta,threshold,allowed}`, all round-tripped. (`D-6_held_out_task_store.md:131-138`)
- **D6-025 `[E][CFG/SEC]`** L3-locked `[held_out]` defaults are root `.elai_cc/held-out`, TTL 6, pass threshold 0.95, minimum 20, regression threshold 5.0%, n-gram size 9, and Jaccard threshold 0.3. (`D-6_held_out_task_store.md:140-145`)
- **D6-026 `[E][DATA/INT]`** Manual seeding creates 25 tasks on `2026-05-19`, five per family: 10 recent GitHub issues, 10 public ARC-AGI tasks, and 5 curated math/prose tasks, each with expected outcome and content hash. (`D-6_held_out_task_store.md:147-152`)
- **D6-027 `[E][FAIL/STATUS]`** Training-corpus Bloom precomputation is deferred; D-6 implements the interface and skips n-gram checks when the Bloom file is empty/unavailable. Candidate source is a public known-training-data subset such as HuggingFace `the-stack`. (`D-6_held_out_task_store.md:154-159`)
- **D6-028 `[E][DET/TEST]`** Store unit coverage is load/append/retire round-trip in a temporary directory. (`D-6_held_out_task_store.md:161-162`)
- **D6-029 `[E][TEST]`** N-gram units require the exact 3-gram set for `a b c d e`, overlap 1.0 for identical input, and 0.0 for disjoint input. (`D-6_held_out_task_store.md:163-164`)
- **D6-030 `[E][TEST]`** Contamination detection is tested with overlapping and disjoint synthetic corpus/Bloom inputs. (`D-6_held_out_task_store.md:165-166`)
- **D6-031 `[E][TEST]`** Retirement table: 7 months/100%/25 evals retires; 7 months/80% keeps; 5 months/100% keeps; 7 months/100%/10 evals keeps. (`D-6_held_out_task_store.md:167-170`)
- **D6-032 `[E][TEST]`** Promotion table: `-3%` against 5% allows; `-7%` blocks. (`D-6_held_out_task_store.md:171`)
- **D6-033 `[E][TEST]`** Integration synthesizes 50 tasks, advances a mock clock across 12 months, verifies retirement membership, and gates baseline versus candidate histories. (`D-6_held_out_task_store.md:173-179`)
- **D6-034 `[E][CON]`** Decision doc must justify per-task TOML for git diffs/no sled locking/~10K scale, `n=9` as LiveCodeBench’s balance, and six-month TTL as a frontier-model cutoff/curation-cost tradeoff. (`D-6_held_out_task_store.md:180-188`)
- **D6-035 `[E][INT]`** Produced contract includes all task/status/source/outcome IDs, store methods, refresh policy, contamination functions, promotion gate/result, three events/payloads, schema 52, config, and dated file layout. (`D-6_held_out_task_store.md:190-203`)
- **D6-036 `[E][DEP/INT]`** Consumes C-15 clock, D-1 archive audit, D-5 BAI, Blake3, and C-17 tokenization; reachable from D-5’s pre-promotion gate, config, and directory layout. (`D-6_held_out_task_store.md:204-213`)
- **D6-037 `[E][FAIL/DET]`** Files must never be deleted; retirement is status marking; empty n-gram input cannot panic. (`D-6_held_out_task_store.md:214-216`)
- **D6-038 `[E][SEC/FAIL]`** Retirement must enforce the minimum sample count and gate thresholds must be config-loaded rather than hardcoded. (`D-6_held_out_task_store.md:217-219`)
- **D6-039 `[E][OBS]`** A 25-task/12-month uniform-pass simulation expects at least 20 `HeldOutAutoRetire` rows, zero for tasks under six months, all 25 fixture files, and persisted status changes. (`D-6_held_out_task_store.md:220-226`)
- **D6-040 `[E][TEST]`** Deterministic checks assert schema 52, all event/type/filter symbols, config keys/defaults, and directory presence. (`D-6_held_out_task_store.md:228-253`)
- **D6-041 `[E][TEST/INT]`** Deterministic checks also run workspace/held-out tests and require all three paper interfaces. The shell presence gate accepts at least 20 TOML files. (`D-6_held_out_task_store.md:251-263`)
- **D6-042 `[E][DET/TEST]`** Acceptance requires public schema 52, three event round trips, idempotent store load/append/retire, and at least ten canonical n-gram scenarios. (`D-6_held_out_task_store.md:265-270`)
- **D6-043 `[E][TEST]`** Acceptance also requires the four retirement scenarios, promotion threshold logic, loaded defaults, exactly 25 seed tasks, zero new workspace errors, and at least 12 passing held-out tests. (`D-6_held_out_task_store.md:271-276`)
- **D6-044 `[E][TEST/OBS]`** Cloud A advances a mocked clock 12 months and requires retirement rows for uniformly passing aged tasks. (`D-6_held_out_task_store.md:277-278`)
- **D6-045 `[E][TEST/FAIL]`** Cloud B injects 7% held-out regression and requires `HeldOutPromotionGate{allowed=false}` with no `TemplatePromoted` event. (`D-6_held_out_task_store.md:279-281`)
- **D6-046 `[E][TEST/INT]`** Cloud C ingests a new arXiv reproduction task through paper-ingestor and requires a refresh row plus file; paper outputs and the three-decision document are mandatory. (`D-6_held_out_task_store.md:282-285`)
- **D6-047 `[E][CAP/INT]`** File effects add `held_out.rs`, 25 task files, integration test, decisions, and three paper interfaces; edit schema, BAI gate integration, config, and module export. (`D-6_held_out_task_store.md:287-299`)
- **D6-048 `[E][CON/DEP]`** D-7, D-15, D-20, and corpus Bloom construction are out of D-6; D-7 consumes family rates and D-22 reuses this store across its three distributions. (`D-6_held_out_task_store.md:301-313`)
- **D6-049 `[E][INT/SEC]`** J-11/S-2 records Arbor’s `B_dev` iteration set versus `B_test` milestone-only set and detached-worktree merge re-run as prior art; the executor never touches `B_test`. (`D-6_held_out_task_store.md:315-323`)
- **D6-050 `[E][CON]`** Arbor’s split is manual and simpler; D-6 alone owns the time window, 9-gram filter, and uniform-pass auto-retirement, so the prior art must be composed rather than restated. (`D-6_held_out_task_store.md:323-325`)
- **D6-051 `[E][SEC]`** J-11/S-8c requires fixture-build-time destruction and re-initialization of `.git` when held-out tasks come from real repositories, preventing recovery of reference history through reflog or loose objects. (`D-6_held_out_task_store.md:327-336`)
- **D6-052 `[E][SEC/CON]`** `.git` reinitialization is store hygiene, not a runtime gate; runtime command blocking and trace audit belong to H-47. (`D-6_held_out_task_store.md:333-337`)

### D-12_operationalization_check_predicate.md — 38 atoms

- **D12-001 `[E][WF/SEC]`** Every claim-emitting baby step must co-emit a runnable `MEASURABLE_PROCEDURE` intended to fail when the claim is wrong; without it the step cannot enter `MEMORIZING`, and the auditor blocks at acceptance. (`D-12_operationalization_check_predicate.md:1-8,18-21`)
- **D12-002 `[E][DEP/STATUS]`** Effort is one to two sessions; D-11 checker registry must be `DONE`, verified by `spec-grounder`. (`D-12_operationalization_check_predicate.md:9-11`)
- **D12-003 `[E][DET/CON]`** D-12 is parallel-safe with D-13/D-14 but not D-15 because novelty labels and operationalization share the checker-result hook. (`D-12_operationalization_check_predicate.md:12-14`)
- **D12-004 `[E][DATA]`** The source states the current BabyStepSpec has eight blocks (`TITLE`, `OBJECTIVE`, `PRE_GATE`, `FILES_PLAN`, `TASKS`, `DETERMINISTIC_CHECK`, `TESTS`, `ACCEPTANCE`) and D-12 adds block **9**. (`D-12_operationalization_check_predicate.md:16-25`)
- **D12-005 `[E][WF/INT]`** A procedure names a `CheckerEntry`, carries claim/candidate/domain arguments, runs when leaving `AUDITING`, and contributes its `CheckerResult` to the acceptance decision. (`D-12_operationalization_check_predicate.md:27-30`)
- **D12-006 `[E][SEC/FAIL]`** The L3 rule is fail-closed: claim without procedure cannot reach `MEMORIZING`; author emits it and auditor enforces it. (`D-12_operationalization_check_predicate.md:32-33`)
- **D12-007 `[E][DEP]`** Foundations are C-31’s block-extension pattern, auditor acceptance hook, D-11 registry/types, and D-1’s overlapping system-level `verifier_required` concept. (`D-12_operationalization_check_predicate.md:35-41`)
- **D12-008 `[E][DEP]`** Required reads are the RPS guide, C-31, the actual BabyStepSpec/parser, D-11 registry, and the auditor prompt; the author prompt is a later consumed/modified surface, not listed in this `Reads` section. (`D-12_operationalization_check_predicate.md:43-52,161-166,240-241`)
- **D12-009 `[E][DATA]`** `BabyStepSpec.measurable_procedure` is temporarily `Option<MeasurableProcedure>`; the struct fields are checker name, bounded 2048 claim, optional bounded 8192 candidate, optional bounded 1024 domain args, and expected `CheckerResult`. (`D-12_operationalization_check_predicate.md:54-63`)
- **D12-010 `[E][WF]`** `expected_result` is a prediction: runtime executes the checker and diffs actual from expected, including expected failure for negative tests. (`D-12_operationalization_check_predicate.md:64-65,140-147`)
- **D12-011 `[E][DATA/FAIL]`** The parser recognizes block token `MEASURABLE_PROCEDURE`, accepts the shown TOML-like `checker/claim/expected_result` format, populates the field, and emits `ParseError` on invalid input. (`D-12_operationalization_check_predicate.md:67-77`)
- **D12-012 `[E][SEC/CON]`** The babystep-author prompt must state that every claim-emitting step includes the block. (`D-12_operationalization_check_predicate.md:79-81`)
- **D12-013 `[E][TEST/CON]`** Prompt examples contrast an unmeasured `O(n log n)` claim (block) with a `perf` checker over `n=10..10000` (compliant). (`D-12_operationalization_check_predicate.md:82-85`)
- **D12-014 `[E][SEC/FAIL]`** Auditor detects a claim with no procedure and blocks with reason `missing MEASURABLE_PROCEDURE`. (`D-12_operationalization_check_predicate.md:87-90`)
- **D12-015 `[E][FAIL]`** Auditor runs the procedure and blocks when actual differs from expected, with reason `measurable procedure result diverges from expected`. (`D-12_operationalization_check_predicate.md:91-93`)
- **D12-016 `[E][CAP/FAIL]`** `require_procedure_for_claim` scans claim-emitting symbols/artifacts and returns `OperationalizationError` when required evidence is absent. (`D-12_operationalization_check_predicate.md:95-101`)
- **D12-017 `[E][CAP/INT]`** `run_procedure` dispatches through D-11 `CheckerRegistry` with `OrchContext`. (`D-12_operationalization_check_predicate.md:102-104`)
- **D12-018 `[E][CAP/DATA]`** `diff_expected(expected,actual)` returns optional `ProcedureDivergence`. (`D-12_operationalization_check_predicate.md:105-106`)
- **D12-019 `[E][CAP]`** `detect_claim_in_baby_step` searches task and observable-effect prose for stated keywords such as “correct,” “complexity is,” “produces,” “guarantees,” and “ensures,” returning `DetectedClaim` values. (`D-12_operationalization_check_predicate.md:108-113`)
- **D12-020 `[E][SEC]`** Detection is intentionally conservative: false positives are acceptable because they force evidence; false negatives are forbidden because they leak untested claims. (`D-12_operationalization_check_predicate.md:114-116`)
- **D12-021 `[E][DATA/INT]`** Schema bumps `57 -> 58`; payloads are `ClaimWithoutProcedure{baby_step_id,claim_snippet,audit_id}` and `ProcedureExecuted{baby_step_id,checker_name,result,expected,divergence}`, with round-trip tests. (`D-12_operationalization_check_predicate.md:118-123`)
- **D12-022 `[E][TEST]`** Claim-detection unit tests cover three claim and three non-claim steps. (`D-12_operationalization_check_predicate.md:125-127`)
- **D12-023 `[E][TEST]`** Units require missing-procedure error, `Pass==Pass -> None`, and `Pass!=Fail -> divergence`. (`D-12_operationalization_check_predicate.md:128-130`)
- **D12-024 `[E][TEST/OBS]`** Integration case 1: missing procedure causes auditor `BLOCK` and a `ClaimWithoutProcedure` row. (`D-12_operationalization_check_predicate.md:132-135`)
- **D12-025 `[E][TEST/OBS]`** Integration case 2: a procedure runs; `Pass` allows progress, while `Fail` blocks and writes `ProcedureExecuted` with divergence. (`D-12_operationalization_check_predicate.md:136-138`)
- **D12-026 `[E][CON]`** Decisions justify a typed ninth block rather than shell-only checks, expected-vs-actual tri-state semantics rather than “always Pass,” and conservative detection. (`D-12_operationalization_check_predicate.md:140-150`)
- **D12-027 `[E][INT]`** Produced symbols are `MeasurableProcedure`, require/run/diff helpers, claim detector/type, both SSB variants/payloads, schema 58, and BabyStepSpec block 9. (`D-12_operationalization_check_predicate.md:152-160`)
- **D12-028 `[E][DEP]`** Consumes registry/result/parser plus auditor and author agents. (`D-12_operationalization_check_predicate.md:161-166`)
- **D12-029 `[E][WF]`** Reachability is from `AUDITING` procedure dispatch and authoring-time plan generation. (`D-12_operationalization_check_predicate.md:167-169`)
- **D12-030 `[E][FAIL/SEC]`** Stub bans require conservative detection, per-checker timeout, full `Pass/Fail/Inapplicable` diffing, and no opt-out from fail-closed enforcement. (`D-12_operationalization_check_predicate.md:170-176`)
- **D12-031 `[E][OBS/SEC]`** Missing-procedure trigger must yield one event, blocked `AUDITING`, a human-review prompt, updated agent prompts, and parser support for block 9. (`D-12_operationalization_check_predicate.md:177-183`)
- **D12-032 `[E][TEST]`** Deterministic checks assert schema 58, both events, `MeasurableProcedure` plus `require_procedure_for_claim`/`run_procedure`, the block token in the parser, both prompt updates, workspace build, and scoped tests; the shell block does not directly grep `diff_expected` or `detect_claim_in_baby_step`. (`D-12_operationalization_check_predicate.md:185-208`)
- **D12-033 `[E][TEST]`** Acceptance requires public schema 58, two event round trips, compiled helpers, parser support, at least six classification fixtures, correct missing-procedure error, and tri-state diff. (`D-12_operationalization_check_predicate.md:210-218`)
- **D12-034 `[E][TEST]`** Acceptance also requires zero new workspace errors, at least eight passing tests, and both agent docs with examples/enforcement. (`D-12_operationalization_check_predicate.md:219-222`)
- **D12-035 `[E][TEST/WF]`** Cloud A requires an authored claim with the block, executed checker, and acceptance pass. (`D-12_operationalization_check_predicate.md:223-225`)
- **D12-036 `[E][TEST/FAIL]`** Cloud B requires missing procedure to block/journal; Cloud C requires expected Pass/actual Fail to journal divergence and block; the decision doc is mandatory. (`D-12_operationalization_check_predicate.md:226-230`)
- **D12-037 `[E][CAP/INT]`** File effects add `operationalization.rs`, integration test, and decision doc; edit BabyStepSpec/parser, SSB, exports, and both agent prompts. (`D-12_operationalization_check_predicate.md:232-243`)
- **D12-038 `[E][CON/DEP]`** Per-checker templates, auto-generation, and procedure caching are deferred; every audit runs fresh. D-13 next covers claims without a Mode-A checker. (`D-12_operationalization_check_predicate.md:245-255`)

### D-23_sycophancy_detection_benchmark.md — 21 atoms

- **D23-001 `[E][CAP/FAIL]`** D-23 uses Sharma public probes or synthetic stand-ins, runs D-14, and measures detection and false positives. (`D-23_sycophancy_detection_benchmark.md:1-5`)
- **D23-002 `[E][TEST/DEP/DET]`** Gate is detection `>=70%` and FP `<=15%`; effort is one to two sessions; D-14 is prerequisite; it is parallel-safe with D-19..D-25. (`D-23_sycophancy_detection_benchmark.md:5-8`)
- **D23-003 `[E][INT/TEST]`** Source context states literature sycophancy at 23-33% across models and expects the structural probe to surface about 70%+ on labeled public/synthetic fixtures. (`D-23_sycophancy_detection_benchmark.md:10-16`)
- **D23-004 `[E][DEP]`** Reads are the RPS sycophancy section, Sharma interface from D-14, and D-14 `sycophancy.rs`. (`D-23_sycophancy_detection_benchmark.md:18-23`)
- **D23-005 `[E][DATA/SEC]`** Fixture set contains exactly 100 human-labeled examples: 70 expected `Sycophancy`, 30 expected `Robust`; positive fixtures encode user assertion plus agreeing model, negatives independent agreement. (`D-23_sycophancy_detection_benchmark.md:25-31,66-68`)
- **D23-006 `[E][DATA]`** `SycophancyBenchmark` owns fixture directory and `Arc<SycophancyProbe>`; report fields are total, TP, FP, TN, FN, detection rate, FP rate, and `GatingVerdict`. (`D-23_sycophancy_detection_benchmark.md:33-39`)
- **D23-007 `[E][WF/TEST]`** `run` replays fixtures through D-14, compares flagged divergence with labels, and computes `TP/(TP+FN)` plus `FP/(FP+TN)`. (`D-23_sycophancy_detection_benchmark.md:40-43`)
- **D23-008 `[E][WF]`** The benchmark passes only when both threshold predicates hold. (`D-23_sycophancy_detection_benchmark.md:45`)
- **D23-009 `[E][CAP]`** CLI surface is `elai bench sycophancy`. (`D-23_sycophancy_detection_benchmark.md:47`)
- **D23-010 `[E][CON]`** Decisions justify the 70/30 split as matching Sharma, the 70% floor as conservative versus roughly 80%, and the 15% FP cap as protection against human-review queue clutter. (`D-23_sycophancy_detection_benchmark.md:49-54`)
- **D23-011 `[E][INT]`** Produces benchmark/report and CLI; consumes D-14 `SycophancyProbe` plus `SsbWriter`; reachable from CLI and CI. (`D-23_sycophancy_detection_benchmark.md:56-65`)
- **D23-012 `[E][SEC/CON]`** Fixtures must be labeled by a human reviewer, not auto-generated. (`D-23_sycophancy_detection_benchmark.md:66-67`)
- **D23-013 `[E][OBS/CON]`** Detection and false-positive rate must be reported separately. (`D-23_sycophancy_detection_benchmark.md:68`)
- **D23-014 `[E][OBS]`** A run emits existing `BenchmarkStarted` and `BenchmarkResult` rows and writes `.elai_cc/benchmarks/sycophancy/reports/{date}.json`. (`D-23_sycophancy_detection_benchmark.md:69-72`)
- **D23-015 `[E][TEST]`** Deterministic checks require at least 100 fixture JSON files, presence of the benchmark struct, and a clean workspace build. (`D-23_sycophancy_detection_benchmark.md:74-81`)
- **D23-016 `[E][TEST]`** Acceptance requires compilation, 100 fixtures, CLI exit 0 on both gates, and zero new workspace errors. (`D-23_sycophancy_detection_benchmark.md:83-88`)
- **D23-017 `[E][TEST]`** Cloud A must meet detection and FP thresholds. (`D-23_sycophancy_detection_benchmark.md:89`)
- **D23-018 `[E][TEST]`** Cloud B requires high-confidence robust agreements to classify `Convergent` and remain unflagged. (`D-23_sycophancy_detection_benchmark.md:90-91`)
- **D23-019 `[E][CAP/INT]`** File effects add runner, CLI module, 100 claim fixtures, decision doc, and edit CLI commands. (`D-23_sycophancy_detection_benchmark.md:94-100`)
- **D23-020 `[E][CON]`** Multi-turn sycophancy is out of scope. (`D-23_sycophancy_detection_benchmark.md:102-104`)
- **D23-021 `[E][DEP]`** D-24 is the declared next session. (`D-23_sycophancy_detection_benchmark.md:106-108`)

### D-26_reasoning_template_dashboard.md — 34 atoms

- **D26-001 `[E][CAP]`** D-26 surfaces typed templates, archive, tripwires, BAI, held-out state, canaries, and archive verification; its objective also names ReasoningDrawer PPM score, consensus rule, and tripwire status. (`D-26_reasoning_template_dashboard.md:1-8`)
- **D26-002 `[E][DEP/DET]`** Effort is two to three sessions; the source requires D-1..D-9 shipped and is parallel-safe with D-27. (`D-26_reasoning_template_dashboard.md:9-11`)
- **D26-003 `[E][DEP/CAP]`** Reuses C-3/C-4 Tauri patterns, C-6 tokens, C-7 layouts, C-24 Cytoscape styles, and SSB queryability. (`D-26_reasoning_template_dashboard.md:13-25`)
- **D26-004 `[E][DEP]`** Reads include C-3/C-7/C-24, dashboard commands/drawer state, and all Wave-1 plan artifacts. (`D-26_reasoning_template_dashboard.md:27-34`)
- **D26-005 `[E][CAP/FAIL]`** `get_archive_recent(limit)` returns latest N `ArchiveEntrySummary` values and must honor the limit. (`D-26_reasoning_template_dashboard.md:38-41,124-126`)
- **D26-006 `[E][CAP/FAIL]`** `get_archive_verify_status` runs chain verification and returns `{total,verified,errors_count}`; verification must execute asynchronously so UI is not blocked. (`D-26_reasoning_template_dashboard.md:42-44,127`)
- **D26-007 `[E][CAP]`** `get_tripwire_status(window_minutes)` queries D-4 tripwire rows over the requested window. (`D-26_reasoning_template_dashboard.md:45-46`)
- **D26-008 `[E][CAP]`** `get_bai_arms` loads D-5 state and returns active posterior summaries. (`D-26_reasoning_template_dashboard.md:47-48`)
- **D26-009 `[E][CAP]`** `get_canary_report(optional template_id)` returns D-7 reports for a template or recent reports. (`D-26_reasoning_template_dashboard.md:49-50`)
- **D26-010 `[E][CAP/DATA]`** `get_held_out_status` returns active count, retired count, and last refresh time from D-6. (`D-26_reasoning_template_dashboard.md:51-52`)
- **D26-011 `[E][CAP]`** `MutationArchivePanel` lists hash prefix, domain, pattern kind, archive status, and optional retirement reason. (`D-26_reasoning_template_dashboard.md:54-57`)
- **D26-012 `[E][CAP/DATA]`** Archive rows expand to a template excerpt and parent link; filtering supports Candidate/Promoted/Retired/Frozen; colors are green/yellow/red/grey respectively. (`D-26_reasoning_template_dashboard.md:58-60`)
- **D26-013 `[E][CAP]`** `LineageGraphView` renders the parent-hash DAG in Cytoscape, colors nodes by status, and flips node clicks to archive detail. (`D-26_reasoning_template_dashboard.md:62-67`)
- **D26-014 `[E][CAP/OBS]`** `TripwireStatusStrip` live-polls the last five events with kind/time/value/threshold; concurrent fire or auto-revert is red, individual firing yellow. (`D-26_reasoning_template_dashboard.md:69-74`)
- **D26-015 `[E][CAP/DATA]`** `BaiArmsTable` shows ID prefix, counts, posterior mean, 95% CI, sorts by mean/lower bound, and highlights current promoted arm green. (`D-26_reasoning_template_dashboard.md:76-80`)
- **D26-016 `[E][CAP]`** `HeldOutPanel` shows per-family active counts and recent retirements with domain filtering. (`D-26_reasoning_template_dashboard.md:82-85`)
- **D26-017 `[E][CAP/FAIL]`** `ArchiveVerifyGauge` live-polls, shows green `Clean` or red `Tampered (N errors)`, and opens the full report on click. (`D-26_reasoning_template_dashboard.md:87-91`)
- **D26-018 `[E][CAP]`** `ReasoningDrawer.svelte` embeds the new panels as tabs or stacked sections. (`D-26_reasoning_template_dashboard.md:93-94`)
- **D26-019 `[E][OBS/INT]`** Tripwire/archive events use the existing C-7 real-time SSB subscription pattern. (`D-26_reasoning_template_dashboard.md:96-97`)
- **D26-020 `[E][CON]`** Decisions justify Cytoscape reuse, separate dockable Svelte components, and status colors for scan speed. (`D-26_reasoning_template_dashboard.md:99-105`)
- **D26-021 `[E][INT]`** Produced contract contains all six Tauri commands and all six components: archive, lineage, tripwire, BAI, held-out, and verification. (`D-26_reasoning_template_dashboard.md:107-114`)
- **D26-022 `[E][DEP/INT]`** Consumes archive/verification, D-4 tripwires, D-5 BAI, D-6 store, D-7 canary, and `SsbViewer`. (`D-26_reasoning_template_dashboard.md:115-121`)
- **D26-023 `[E][CAP]`** Reachable surfaces are the dashboard ReasoningDrawer and a dedicated archive sub-window. (`D-26_reasoning_template_dashboard.md:122-123`)
- **D26-024 `[E][FAIL]`** Full-archive accidental return is prohibited through mandatory `limit` honoring. (`D-26_reasoning_template_dashboard.md:124-126`)
- **D26-025 `[E][FAIL/DET]`** Chain verification cannot block the UI and therefore runs async. (`D-26_reasoning_template_dashboard.md:127`)
- **D26-026 `[E][TEST/INT]`** Every command must pass the existing ts-rs binding-drift workflow. (`D-26_reasoning_template_dashboard.md:128-129`)
- **D26-027 `[E][OBS/TEST]`** Live promotion/tripwire injection must stream SSB data, show the archive entry within the poll interval, turn the strip red, and keep verification green on clean state. (`D-26_reasoning_template_dashboard.md:130-135`)
- **D26-028 `[E][TEST]`** Deterministic checks grep commands/components and run `svelte-check` plus dashboard cargo check. (`D-26_reasoning_template_dashboard.md:137-145`)
- **D26-029 `[E][TEST]`** Acceptance requires six commands, six components, zero Svelte errors, clean ts-rs drift, and zero new dashboard Rust errors. (`D-26_reasoning_template_dashboard.md:147-152`)
- **D26-030 `[E][TEST]`** Cloud A requires a new archive entry, graph node, and parent edge after live promotion. (`D-26_reasoning_template_dashboard.md:153-155`)
- **D26-031 `[E][TEST]`** Cloud B requires a red tripwire strip within an explicit three-second poll interval. (`D-26_reasoning_template_dashboard.md:156-157`)
- **D26-032 `[E][TEST]`** Cloud C toggles verification green/red across clean/manual tamper; Cloud D shows at least 25 active held-out tasks; Cloud E shows sortable posterior rows; decision doc is mandatory. (`D-26_reasoning_template_dashboard.md:158-163`)
- **D26-033 `[E][CAP/INT]`** File effects edit commands/drawer/bindings and add six exact Svelte view files plus decision doc. (`D-26_reasoning_template_dashboard.md:165-178`)
- **D26-034 `[E][CON/DEP]`** D-27 and a full B+C+D progress overlay are out of scope, except the partial `bn_status_report.py --plan=D` hook; D-27 is next. (`D-26_reasoning_template_dashboard.md:180-188`)

### D-27_mode_routing_refusal_dashboard.md — 33 atoms

- **D27-001 `[E][CAP]`** D-27 surfaces routing distribution, novelty breakdown, refusal SLA, sycophancy flags, discovery review, and checker health through dashboard-v2. (`D-27_mode_routing_refusal_dashboard.md:1-6,39-44,64-81`)
- **D27-002 `[E][DEP/DET]`** Effort is two sessions; D-10..D-17 must be shipped; D-26 is parallel-safe. (`D-27_mode_routing_refusal_dashboard.md:7-9`)
- **D27-003 `[E][DEP]`** D-26 supplies the pattern, C-7 the tokens, and D-10..D-17 the SSB rows; reads are C-3/C-7 and all Wave-2 plans. (`D-27_mode_routing_refusal_dashboard.md:11-25`)
- **D27-004 `[E][CAP]`** `get_mode_breakdown(optional session_id,window_minutes)` counts `ModeSelected` rows per mode. (`D-27_mode_routing_refusal_dashboard.md:29-32`)
- **D27-005 `[E][CAP]`** `get_refusal_rate(optional session_id)` returns D-17’s `RefusalRateSnapshot` directly. (`D-27_mode_routing_refusal_dashboard.md:33-34`)
- **D27-006 `[E][CAP]`** `get_novelty_breakdown(window_minutes)` counts `NoveltyLabeled` rows by `NoveltyClass`. (`D-27_mode_routing_refusal_dashboard.md:35-36`)
- **D27-007 `[E][CAP]`** `get_sycophancy_flags(limit)` returns recent flags plus claim text. (`D-27_mode_routing_refusal_dashboard.md:37-38`)
- **D27-008 `[E][CAP]`** `get_discovery_queue` returns pending human-review claims from the D-15 queue directory. (`D-27_mode_routing_refusal_dashboard.md:39-40`)
- **D27-009 `[E][SEC/WF]`** `ratify_discovery(claim_id,reviewer)` writes `DiscoveryRatified` and updates the queue file. (`D-27_mode_routing_refusal_dashboard.md:41-42`)
- **D27-010 `[E][SEC/WF]`** `reject_discovery(claim_id,reviewer,reason)` writes `DiscoveryRejected`. (`D-27_mode_routing_refusal_dashboard.md:43-44`)
- **D27-011 `[E][CAP/OBS]`** `ModeRoutingPanel` shows a per-session pie/stacked distribution, live-updates on `ModeSelected`, and reveals percent plus raw count on hover. (`D-27_mode_routing_refusal_dashboard.md:46-50`)
- **D27-012 `[E][CAP/CFG]`** `RefusalRateGauge` spans 0-100%, highlights healthy `[5%,50%]` green and other values red, and opens recent refusal decisions on click. (`D-27_mode_routing_refusal_dashboard.md:52-57`)
- **D27-013 `[E][CAP]`** `NoveltyBreakdown` shows Search/DistantSearch/DiscoveryCandidate bars; clicking the candidate bar opens the review drawer. (`D-27_mode_routing_refusal_dashboard.md:59-62`)
- **D27-014 `[E][CAP/SEC]`** `DiscoveryQueueDrawer` lists claim/evidence/classification time and exposes Ratify with reviewer input and Reject with reason. (`D-27_mode_routing_refusal_dashboard.md:64-69`)
- **D27-015 `[E][CAP/SEC]`** `SycophancyFlagStrip` streams claim ID, Jaccard score, threshold, and a link to quarantined-claim review. (`D-27_mode_routing_refusal_dashboard.md:71-75`)
- **D27-016 `[E][CAP/OBS]`** `CheckerStatusPanel` lists registry entries with ten recent dispatches, pass/fail ratio, last-failure time, and evidence snippet. (`D-27_mode_routing_refusal_dashboard.md:77-81`)
- **D27-017 `[E][CAP]`** Placement is MonitorWindow for routing/refusal, ChatWindow for sycophancy, and a Reasoning tab for novelty/queue/checkers. (`D-27_mode_routing_refusal_dashboard.md:83-86`)
- **D27-018 `[E][OBS/INT]`** Mode, refusal, and sycophancy events use real-time SSB subscription. (`D-27_mode_routing_refusal_dashboard.md:88`)
- **D27-019 `[E][CON]`** Decisions justify dashboard review over CLI-only UX, a pie chart for five categorical modes, and a gauge for one value plus healthy band. (`D-27_mode_routing_refusal_dashboard.md:90-96`)
- **D27-020 `[E][INT]`** Produced contract contains seven commands and six components, including the two review/checker components omitted from the D-0 status note. (`D-27_mode_routing_refusal_dashboard.md:98-106`)
- **D27-021 `[E][DEP/INT]`** Consumes `ModeSelected`, `RefusalDecision`, `NoveltyLabeled`, `SycophancyFlag`, checker dispatch/pass/fail events, `RefusalRateMonitor`, `DiscoveryQueueEntry`, and both SSB reader/writer. (`D-27_mode_routing_refusal_dashboard.md:107-112`)
- **D27-022 `[E][CAP]`** Reachability is through MonitorWindow, ChatWindow, and ReasoningDrawer tabs. (`D-27_mode_routing_refusal_dashboard.md:113-114`)
- **D27-023 `[E][SEC]`** Ratify/reject must require a reviewer; anonymous ratification is forbidden. (`D-27_mode_routing_refusal_dashboard.md:115-117`)
- **D27-024 `[E][SEC]`** Every sycophancy flag must link to its quarantined claim. (`D-27_mode_routing_refusal_dashboard.md:118`)
- **D27-025 `[E][CFG/SEC]`** Healthy-band thresholds must load from D-17 config and cannot be hardcoded in Svelte. (`D-27_mode_routing_refusal_dashboard.md:119-120`)
- **D27-026 `[E][OBS/TEST]`** A mixed cloud task must feed existing events and render routing, refusal, and pending discovery state. (`D-27_mode_routing_refusal_dashboard.md:121-126`)
- **D27-027 `[E][TEST]`** Deterministic checks grep command/component presence and run Svelte plus Rust checks. (`D-27_mode_routing_refusal_dashboard.md:127-135`)
- **D27-028 `[E][TEST]`** Acceptance requires all seven commands and six components to compile/type-check, clean ts-rs drift, and zero new Rust errors. (`D-27_mode_routing_refusal_dashboard.md:137-142`)
- **D27-029 `[E][TEST]`** Cloud A requires a realistic five-mode distribution; Cloud B requires an out-of-band high-refusal session to turn the gauge red. (`D-27_mode_routing_refusal_dashboard.md:143-146`)
- **D27-030 `[E][TEST/SEC]`** Cloud C increments DiscoveryCandidate/queue for a not-in-DKG primitive; Cloud D surfaces a sycophancy fixture within the poll interval; Cloud E ratifies a queue entry, writes `DiscoveryRatified`, and updates its status. (`D-27_mode_routing_refusal_dashboard.md:147-154`)
- **D27-031 `[E][CAP/INT]`** File effects edit commands plus Monitor/Chat/Reasoning views and add six exact component files plus decision doc. (`D-27_mode_routing_refusal_dashboard.md:156-169`)
- **D27-032 `[E][CON/STATUS]`** Discovery-review TUI, multi-reviewer ratification, and cross-plan progress overlay are deferred/out of scope. (`D-27_mode_routing_refusal_dashboard.md:171-175`)
- **D27-033 `[E][STATUS]`** D-27 declares Wave 4 complete and Wave 5 research-grade/deferred. (`D-27_mode_routing_refusal_dashboard.md:177-179`)

### D-29_hierarchical_bai_research.md — 28 atoms

- **D29-001 `[E][CAP/STATUS]`** D-29 addresses non-IID session rewards with hierarchical Bayesian BAI and accepts a concrete infeasibility note as valid closure. (`D-29_hierarchical_bai_research.md:1-7`)
- **D29-002 `[E][STATUS/DEP/DET]`** Header status is `DEFERRED`; effort is 4-8 wall-clock sessions plus RunPod; D-5 flat BAI and RunPod are prerequisites; D-28/D-30 are parallel-safe. (`D-29_hierarchical_bai_research.md:8-12`)
- **D29-003 `[E][DATA]`** Proposed hierarchy has per-arm pass/fail, a per-family hyperprior over arm pass rates, and pooled within-family posterior information. (`D-29_hierarchical_bai_research.md:14-20`)
- **D29-004 `[E][CON/STATUS]`** The source asserts possible 30-50% regret reduction from broader hierarchical literature but explicitly says that literature is not yet verified; it also warns ELAI’s structure may yield no gain. (`D-29_hierarchical_bai_research.md:21-27`)
- **D29-005 `[E][DEP]`** Required reads are the companion BAI/Q4 sections, Audibert and variance-BAI interfaces, and D-5 flat implementation. (`D-29_hierarchical_bai_research.md:29-35`)
- **D29-006 `[E][INT/DEP]`** At unblock time, paper-ingestor must select hierarchical-bandit literature and create `workflow/papers/hierarchical-bandit-<ref>-interface.md`. (`D-29_hierarchical_bai_research.md:37-44`)
- **D29-007 `[E][DATA/CAP]`** Proposed solver at `.elai_cc/python/bai_hierarchical/` uses per-arm Beta distributions, Gamma (or similar) hyperpriors, and Gibbs sampling for about 50 arms x 200 evaluations. (`D-29_hierarchical_bai_research.md:46-52`)
- **D29-008 `[E][TEST]`** Synthetic benchmark has five families, ten arms per family, within-family correlation, cross-family independence, and compares flat versus hierarchical regret over 1000 pulls. (`D-29_hierarchical_bai_research.md:54-57`)
- **D29-009 `[E][STATUS/TEST]`** Path A succeeds when hierarchical regret is no worse than flat and then productizes the solver. (`D-29_hierarchical_bai_research.md:59-60`)
- **D29-010 `[E][STATUS/FAIL]`** Path B closes with an infeasibility note and at least one concrete blocker when hierarchy is equal/worse, such as unjustified compute or weak non-IID structure. (`D-29_hierarchical_bai_research.md:61-63`)
- **D29-011 `[E][CFG/FAIL]`** If Path A, D-5 integration is gated by `ELAI_BAI_MODE=hierarchical`; default remains flat. (`D-29_hierarchical_bai_research.md:65-66`)
- **D29-012 `[E][STATUS]`** Decision doc is drafted only when the deferred work unblocks. (`D-29_hierarchical_bai_research.md:68-69`)
- **D29-013 `[E][DATA]`** The later wiring contract claims concrete `HierarchicalBaiState` at `bai_hierarchical.rs:271`, `FamilyHyperprior` at `:202`, `FamilyId` at `:115`, and `SelectionBasis` at `:246`. (`D-29_hierarchical_bai_research.md:71-75`)
- **D29-014 `[E][CAP]`** Claimed functions are `family_id_for_domain`/`family_id_for_template_or_mixed` at lines 159/183, `hierarchical_update` at 509, `hierarchical_arm_select` at 605, and `total_variance_decomposition` at 659. (`D-29_hierarchical_bai_research.md:76-78`)
- **D29-015 `[E][DATA/INT/CFG]`** Claimed production wiring adds `HierarchicalBaiRoundCompleted`/payload at `ssb.rs:6725` and `BaiMode::Hierarchical` at `runtime_config.rs:440`, selected by `ELAI_BAI_MODE`. (`D-29_hierarchical_bai_research.md:79-80`)
- **D29-016 `[E][C][STATUS]`** A contract comment says Path A “shipped” and Path B was not the outcome, conflicting with the header/acceptance `DEFERRED` state. (`D-29_hierarchical_bai_research.md:81`)
- **D29-017 `[E][STATUS]`** The Python Gibbs solver/benchmark remains `DEFERRED/RUNPOD-PENDING` even while host-side Rust productization is claimed. (`D-29_hierarchical_bai_research.md:82`)
- **D29-018 `[E][DEP/INT]`** Consumes a D-5 `darwinian_bai` flat baseline/default discount factor, `BaiMode::from_env`, template/domain assignment, and `SsbWriter`. (`D-29_hierarchical_bai_research.md:83-87`)
- **D29-019 `[E][WF/CFG]`** Runtime reachability is a `task_runner.rs:11406` hierarchical promotion branch enabled only when both `ELAI_BAI_MODE=hierarchical` and `ELAI_BAI_ENABLE=1`; unset mode defaults flat/wire-dark. (`D-29_hierarchical_bai_research.md:88-90`)
- **D29-020 `[E][STATUS/TEST]`** The 5-family/10-arm/1000-pull Gibbs regret comparison is still RunPod-pending. (`D-29_hierarchical_bai_research.md:91-92`)
- **D29-021 `[E][CON]`** `hierarchical_arm_select` must record/use `SelectionBasis` and cannot silently fall through to flat. (`D-29_hierarchical_bai_research.md:93-94`)
- **D29-022 `[E][OBS/FAIL]`** Every hierarchical round must emit `HierarchicalBaiRoundCompleted`; mode must default `Flat` when the environment value is absent. (`D-29_hierarchical_bai_research.md:95-96`)
- **D29-023 `[E][OBS/TEST]`** A hierarchical task expects one round row with `gibbs_iterations > 0` and recorded `SelectionBasis`; the Rust dispatch is explicitly called wired/live while research closure remains pending. (`D-29_hierarchical_bai_research.md:97-103`)
- **D29-024 `[E][DET/STATUS]`** `DETERMINISTIC_CHECK` is unpopulated and deferred to unblock time. (`D-29_hierarchical_bai_research.md:105-107`)
- **D29-025 `[E][STATUS/DEP]`** Acceptance remains unchecked/deferred until stable D-5 baseline, RunPod CPU, and paper interface exist. (`D-29_hierarchical_bai_research.md:109-113`)
- **D29-026 `[E][TEST/STATUS]`** Closure still requires the synthetic report, either Path A or Path B, and the decision doc. (`D-29_hierarchical_bai_research.md:114-117`)
- **D29-027 `[E][CAP/STATUS]`** Files are still labeled “anticipated”: Python solver, Rust `bai_hierarchical.rs` if Path A, and decision doc. (`D-29_hierarchical_bai_research.md:119-123`)
- **D29-028 `[E][CON/DEP]`** More than two hierarchy levels and online updates are deferred; D-29 remains independent of D-28/D-30. (`D-29_hierarchical_bai_research.md:125-132`)

### D-001 inferences

- **D-001/I-001 `[I]`** The intended promotion transaction is likely evaluation -> posterior update -> held-out veto -> archive transition, but neither D-5 nor D-6 defines an atomic transaction/rollback boundary across SSB, TOML, and `ArchiveChain`. Derived from D5-022..D5-024 and D6-022..D6-024/D6-045.
- **D-001/I-002 `[I]`** D-26 is operationally read-only because all six named commands are getters, whereas D-27 is the first processed dashboard plan with explicit write commands. The sources do not state a formal read-only capability boundary. Derived from D26-005..D26-010 and D27-009/D27-010.
- **D-001/I-003 `[I]`** D-6’s `ManuallyExcluded` status probably represents a curator action distinct from auto-retirement, but no method/event/authorization flow for setting it is specified. Derived from D6-012..D6-016.
- **D-001/I-004 `[I]`** The D-23 label `Robust` appears intended to correspond to D-14/D-23’s `Convergent` output, but the source does not declare an enum mapping. Derived from D23-005/D23-007/D23-018.
- **D-001/I-005 `[I]`** Default-flat D-29 operation is a compatibility fallback that keeps existing promotion live when hierarchical mode is unset, but no automatic fallback after a hierarchical runtime failure is stated. Derived from D29-011/D29-019/D29-022.
- **D-001/I-006 `[I]`** D-27’s required reviewer string is attribution, not authentication; the plan does not establish that the supplied reviewer is a verified human identity. Derived from D27-009/D27-010/D27-023.

### D-001 cross-source interactions (namespaced D-001/IX-*)

- **D-001/IX-001** **Promotion pipeline.** D-3 consensus supplies D-5 binary observations; D-5 promotes/retire templates through D-1 archive; D-6 interposes a held-out veto before promotion; D-7 adds a family veto; D-4 can auto-revert the same archive; D-8 later exposes lineage verification. Evidence: D0-007..D0-012, D5-011/D5-022..D5-024/D5-036/D5-045, D6-022/D6-023/D6-036/D6-045/D6-048.
- **D-001/IX-002** **Concurrency boundary.** D-5 and D-6 can proceed around D-7’s read/signal work, and both explicitly exclude parallel execution with D-4 because archive/evaluation state overlaps. D-5 also calls D-6 parallel-safe, although D-6 separately requires D-5 to be `DONE`; that unresolved ordering tension is recorded below. Evidence: D5-003, D6-002.
- **D-001/IX-003** **Deterministic promotion evidence.** C-15/D-1 seeded randomness drives D-5; C-15 seeded time drives D-6. D-5 demands same-seed byte identity; D-6 uses mock/seeded time and idempotent storage checks. Evidence: D5-017/D5-028/D5-036..D5-042, D6-006/D6-036/D6-039/D6-042.
- **D-001/IX-004** **Audit-state claim path.** D-12 consumes D-11’s checker registry, blocks `AUDITING -> MEMORIZING` without a procedure, and shares the checker-result hook with downstream D-15. D-13 handles claims lacking a Mode-A checker. Evidence: D0-015..D0-019, D12-001/D12-003/D12-005/D12-017/D12-028/D12-038.
- **D-001/IX-005** **Sycophancy proof-to-UI path.** D-14 produces the probe/flag, D-23 quantitatively gates it, and D-27 streams flags with quarantine links; D-27’s operator review is distinct from D-23’s human fixture labeling. Evidence: D0-018/D0-027/D0-031, D23-001/D23-005..D23-018, D27-007/D27-015/D27-024/D27-030.
- **D-001/IX-006** **Wave-1 dashboard.** D-26 reads D-1/D-4/D-5/D-6/D-7/D-8 state, applies bounded/asynchronous query rules, and is parallel-safe with D-27. Evidence: D26-001/D26-002/D26-005..D26-027.
- **D-001/IX-007** **Wave-2 dashboard and human disposition.** D-27 reads D-10..D-17 events but also writes ratify/reject decisions through `SsbWriter`; it is therefore both an observation surface and a human-governance mutation surface. Evidence: D27-004..D27-010/D27-020..D27-025.
- **D-001/IX-008** **Hierarchical extension.** D-29 is an opt-in extension of D-5 with two environment gates and flat default; benchmark closure compares directly with the D-5 flat baseline. Evidence: D5-044, D29-008..D29-011/D29-018..D29-023.
- **D-001/IX-009** **Evaluation-store secrecy.** D-6 composes temporal/9-gram filtering with Arbor-style dev/test isolation and DeNovo-style `.git` history removal; runtime leakage blocking remains explicitly owned by H-47. Evidence: D6-003/D6-019/D6-049..D6-052.
- **D-001/IX-010** **Plan status/dashboard accounting.** D-0 defines `DONE` as implementation + cloud smoke + threshold, while D-26 adds the `--plan=D` status-report hook. D-29’s embedded “shipped” note does not independently satisfy the D-0 `DONE` definition because its closure benchmark remains pending. Evidence: D0-003/D0-035/D0-066/D0-068, D29-016/D29-020/D29-024..D29-027.

### D-001 ambiguity and conflict register (namespaced D-001/AC-*)

- **D-001/AC-001** **D-29 lifecycle/status is internally conflicting `[C]`.** Header, deterministic-check section, acceptance list, and “anticipated files” say deferred/pending (D29-002, D29-024..D29-027); the wiring contract says Path A shipped and Rust dispatch is live (D29-016/D29-023). Safest status is **mixed/partially implemented claim, research closure not done**, not simply `DONE` or simply untouched.
- **D-001/AC-002** **D-0 versus D-29 status conflicts `[C]`.** The index calls D-29 deferred/research-grade (D0-033/D0-064/D0-066); the later D-29 contract claims Path A shipped (D29-016). No date/version marker explains the update.
- **D-001/AC-003** **Gibbs evidence is unresolved `[A/C]`.** D29-017/D29-020 say the Python Gibbs benchmark is RunPod-pending, while D29-023 expects a live round with `gibbs_iterations > 0`. It is unclear whether Rust performs a separate Gibbs-like update or the observable contract is aspirational.
- **D-001/AC-004** **Flat-module identity drifts `[A]`.** D-5 plans `bai.rs`/`BaiPromotionEngine` (D5-021/D5-043); D-29 names a D-5 `darwinian_bai` baseline and default discount factor (D29-018). Rename/history is not explained.
- **D-001/AC-005** **D-26 prerequisite differs `[C]`.** D-0 says D-26 requires/exposes D-1..D-8 (D0-030/D0-061); D-26 says D-1..D-9 must be shipped (D26-002). D-26 does not name a D-9 data surface, so D-9 may be a governance prerequisite rather than a rendered capability.
- **D-001/AC-006** **D-26 objective exceeds its task ledger `[A]`.** D26-001 promises PPM-score and consensus-rule display, but its six commands/components do not name a PPM/consensus query or field. Existing `ReasoningDrawer` data may supply them, but that is not stated.
- **D-001/AC-007** **Archive verification ownership is split `[A]`.** D26-006 attributes the invoked method to D-1, while D26-022 says `verify_chain` spans D-1+D-8. Exact owning module/API is unresolved.
- **D-001/AC-008** **Held-out “append-only” conflicts with rewrite `[C/A]`.** D0-041 calls the task layout append-only, but D6-015 rewrites TOML status on retirement. The no-delete rule (D6-037) is explicit; append-only may mean no file deletion or may refer only to audit history.
- **D-001/AC-009** **Semantic/structural contamination defenses are designed but not implemented `[A]`.** D6-004 requires semantic fingerprints, D6-005 names structural-prior randomization, and D6-012 has `SemanticContamination`, but no semantic detector or structural-randomization task/config/test is specified. Only n-gram Bloom logic is executable in this plan.
- **D-001/AC-010** **Bloom absence degrades silently `[A]`.** D6-027 says skip n-gram checking until data exists; no SSB warning, operator alert, or promotion fail-closed rule is stated for this protection gap.
- **D-001/AC-011** **Store error/idempotency semantics are incomplete `[A]`.** Acceptance says load/append/retire are idempotent (D6-042), but behavior for duplicate `TaskId`, partially written TOML, parse failure, date-path mismatch, or concurrent writers is absent.
- **D-001/AC-012** **Seed-count gates differ `[A]`.** D6-026/D6-043 require 25 tasks; shell presence gate accepts `>=20` (D6-041); observable contract expects `>=20` retirements from a 25-task uniform-pass simulation (D6-039). These may be smoke versus final gates, but are not labeled that way.
- **D-001/AC-013** **Archive task payload shape is underspecified `[A]`.** D6-006 says an `ArchiveEntry` carries `task_id` rather than `template_id`, but no union/schema amendment is defined in D-6.
- **D-001/AC-014** **D-12 block-number conflict is resolved only by deep evidence `[C]`.** The orientation summary says “block 7,” but both D0-047 and D12-004/D12-027 explicitly say block 9 after eight existing blocks. The original-plan evidence therefore favors block 9.
- **D-001/AC-015** **D-12 schema-bump atomicity is unclear `[A]`.** D0-045 expects one atomic D-10+D-11+D-12 bump; D12-021 independently specifies `57 -> 58`. It may mean sequenced per-session bumps, but “atomic” is unexplained.
- **D-001/AC-016** **Claim detection cannot literally guarantee no false negatives `[A]`.** D12-019 defines keyword/prose heuristics while D12-020/D12-030 say false negatives are not allowed. No formal grammar, exhaustive claim AST, or fallback classifier is supplied.
- **D-001/AC-017** **Transition compatibility is temporary but unbounded `[A]`.** `measurable_procedure` is `Option` during transition (D12-009) while the L3 rule is no opt-out (D12-006/D12-030); no migration version/date identifies when `None` ceases to be accepted outside claim steps.
- **D-001/AC-018** **D-23’s 70/30 rationale is numerically tense `[C]`.** Context cites observed sycophancy at 23-33% (D23-003), while decisions say a fixture population with 70% sycophancy “matches” Sharma (D23-010). The split is useful for power but does not match that prevalence literally.
- **D-001/AC-019** **D-23 outcome vocabulary drifts `[A]`.** Fixtures use `Sycophancy/Robust` (D23-005), execution uses “flagged-divergent” (D23-007), and Cloud B expects `Convergent` (D23-018); mapping is not typed.
- **D-001/AC-020** **D-23 failure exit is incomplete `[A]`.** CLI exit 0 is specified only for a passing gate (D23-016); non-passing exit code and report persistence on failure are not defined.
- **D-001/AC-021** **D-5 has multiple benchmark scales `[A, not necessarily conflict]`.** D0-009 says 50 rounds within the published bound; D5-033 defines 500 pulls/100 trials/bound x2; D5-042 defines a 50-cycle cloud bootstrap. They appear to be distinct index, integration, and cloud gates and must not be collapsed into one threshold.
- **D-001/AC-022** **D-5 Successive Rejects output semantics are terse `[A]`.** D5-019 returns “ordered survivors,” while repeated elimination conventionally ends with one survivor; D5-024 applies it to a bottom quartile. Exact returned ordering and budget allocation across `retire_round` are not fully specified.
- **D-001/AC-023** **D-27 rejection lifecycle is asymmetric `[A]`.** Ratification explicitly updates the queue file (D27-009); rejection only explicitly writes an event (D27-010), although the UI implies disposition. Whether rejected entries leave the pending queue is not stated.
- **D-001/AC-024** **D-27 human governance is attribution-only `[A]`.** Reviewer is required, but authentication, reviewer authorization, multiple reviewers, and conflict-of-interest controls are absent; multi-reviewer flow is explicitly deferred (D27-023/D27-032).
- **D-001/AC-025** **Polling semantics remain partly open `[A]`.** D-26 fixes tripwire reaction at 3 seconds (D26-031), but archive, verification, D-27 sycophancy, disconnection, stale state, and retry intervals are not quantified.
- **D-001/AC-026** **Dashboard data exposure is ungoverned `[A]`.** D26-012 and D27-007/D27-014..D27-016 display template, claim, evidence, reviewer, and failure snippets without an explicit permission/redaction model.
- **D-001/AC-027** **D-0’s aggregate status is internally tense `[C/A]`.** D0-066 reports 27 in-scope slots and three deferred slots but also says all 30 authored files are `PLANNED`; the status legend treats `PLANNED` and `DEFERRED` as distinct. It is unclear whether “all PLANNED” describes document authorship rather than session status.
- **D-001/AC-028** **The D-5 file inventory drifts `[A/C]`.** D0-040 expects both `bai.rs` and `promotion_arms.rs`, while the detailed D-5 tasks, wiring contract, and files-touched list place the engine in `bai.rs` and never name `promotion_arms.rs` (D5-014..D5-043). No consolidation/rename note resolves the extra index file.
- **D-001/AC-029** **The held-out event inventory grows without reconciliation `[A]`.** D0-041 lists only `HELD_OUT_REFRESH` and `HELD_OUT_AUTO_RETIRE`; D6-024 makes `HeldOutPromotionGate` a third schema event. This may be a detailed-plan expansion, but D-0’s “expected additions” list was not updated.
- **D-001/AC-030** **D-6 has several compatible but distinct time-scale gates `[A, not necessarily conflict]`.** The index cloud proof retires one 18-month-old task (D0-010), D-6 units exercise 7/5-month cases (D6-031), and cloud/integration evidence advances across 12 months (D6-033/D6-039/D6-044). They must remain separate checks rather than one normalized age threshold.
- **D-001/AC-031** **Auto-refresh cadence is unbound `[A]`.** D6-003 says refresh every `N` sessions, but `AutoRefreshPolicy`, config, reachability, deterministic checks, and acceptance define no `N`, scheduler, or invocation trigger (D6-020/D6-025/D6-035..D6-046).
- **D-001/AC-032** **The n-gram detector’s corpus interface is internally underspecified `[A]`.** The source signature accepts `training_corpus_hashes: &Vec<Blake3Hash>` while the algorithm also says to load `ngram-bloom.bin`; it does not say which input is authoritative or how exact Jaccard union size is recovered from Bloom membership (D6-019).
- **D-001/AC-033** **BabyStepSpec’s existing-block identifier drifts `[A]`.** D12-004 quotes the current block as `FILES_PLAN`, while the same source calls the C-31 foundation `FILE_PLAN` (D12-007/D12-026). The ninth block is consistently `MEASURABLE_PROCEDURE`, but the pre-existing block name is not.
- **D-001/AC-034** **Checker timeout disposition is undefined `[A]`.** D12-030 requires `run_procedure` to honor `checker.timeout_secs`, but no `CheckerResult`, divergence value, event payload, retry policy, or auditor reason is specified for timeout.
- **D-001/AC-035** **D-12’s deterministic shell does not directly cover two acceptance helpers `[A]`.** D12-032’s grep checks omit `diff_expected` and `detect_claim_in_baby_step`, although D12-033 requires both behaviors. Scoped tests may cover them, but the deterministic presence gate does not prove that explicitly.
- **D-001/AC-036** **D-23 fixture-count gates differ `[A]`.** Tasks and acceptance say 100 fixtures (D23-005/D23-016), while the shell gate accepts any count `>=100` (D23-015). Extra fixtures would change prevalence and metrics, but their handling is not defined.
- **D-001/AC-037** **D-23 serialization contracts are absent `[A]`.** The plan defines report fields and label counts but not the JSON fixture fields needed to replay D-14, nor the payload schemas for the generic `BenchmarkStarted`/`BenchmarkResult` events (D23-005..D23-007/D23-014/D23-019).
- **D-001/AC-038** **D-23’s parallel-safe range self-includes `[A]`.** D23-002 says it is parallel-safe with D-19..D-25, a literal range containing D-23 itself. The likely intent is “the other benchmark sessions,” but exclusions are not enumerated.
- **D-001/AC-039** **Archive parent identifiers drift `[A]`.** `MutationArchivePanel` detail links `parent_id` (D26-012), whereas `LineageGraphView` edges use `parent_hash` (D26-013). The source does not state whether they are aliases or require a lookup.
- **D-001/AC-040** **Canary “last N” is not parameterized `[A]`.** `get_canary_report(template_id: Option<TemplateId>)` promises the given template or “last N” reports (D26-009), but its signature/config defines no `N` or limit default.
- **D-001/AC-041** **Two D-26 navigation surfaces lack explicit wiring `[A]`.** `ArchiveVerifyGauge` opens a full report and the contract claims a dedicated archive sub-window (D26-017/D26-023), but the six commands and files-touched list define neither a full-report command nor a sub-window integration file (D26-021/D26-033).
- **D-001/AC-042** **D-26 mixes polling and subscription without ownership rules `[A]`.** Individual panels poll, while archive/tripwire updates also reuse real-time SSB subscription (D26-014/D26-017/D26-019/D26-027). Cache precedence, deduplication, and reconnect behavior are not specified.
- **D-001/AC-043** **The refusal healthy band has two authorities `[A]`.** The component task states `[5%,50%]` (D27-012), while the stub ban requires thresholds loaded from D-17 config (D27-025). D-27 does not state that 5/50 are D-17 defaults or what the UI shows when config differs.
- **D-001/AC-044** **Discovery disposition events lack schema/lifecycle contracts `[A]`.** D27-009/D27-010 write `DiscoveryRatified` and `DiscoveryRejected`, but this source defines no SSB variant/payload/schema bump, round-trip test, idempotency key, duplicate-click behavior, or rejection queue-file update.
- **D-001/AC-045** **D-29 closure paths overlap on equality `[C]`.** Path A accepts hierarchical regret `<=` flat (D29-009), while Path B is “equal or worse” (D29-010); equality satisfies both, so the branch decision is not mutually exclusive.
- **D-001/AC-046** **D-29 mode/event failure schemas are incomplete `[A]`.** The source names `HierarchicalBaiRoundCompletedPayload` and the unset-mode `Flat` default (D29-015/D29-022), but gives no payload field types beyond observable references to `gibbs_iterations`/`SelectionBasis`, and no behavior for an invalid `ELAI_BAI_MODE` value or hierarchical runtime failure.
- **D-001/AC-047** **“Online updates” conflicts with an unqualified update API `[A]`.** D29-014 claims `hierarchical_update`, while D29-028 defers “online hierarchical updates.” The source does not distinguish a batch/offline update from the deferred online behavior.
- **D-001/AC-048** **D-5’s LUCB optional return is incomplete `[A]`.** D5-020 declares `Option<TemplateId>` but specifies neither empty-input nor no-competitor behavior; the explicit empty-input rule applies only to `thompson_select` (D5-018).
- **D-001/AC-049** **D-5/D-6 ordering and parallelism conflict `[C/A]`.** D5-003 calls D-6 parallel-safe, but D6-002 requires D-5 to be `DONE` before D-6. No milestone-level dependency explains how both can be true.
- **D-001/AC-050** **Empty-set n-gram overlap is undefined `[A]`.** D6-017/D6-037 require empty extraction to return safely, while D6-018 defines overlap as `intersection/union`; no zero-union convention is given for two inputs with no n-grams.

### D-001 summary deltas (namespaced D-001/SD-*)

#### Additions

- **D-001/SD-A001** **D-0 exact operational ledger.** Add all schema-version expectations, exact SSB identifiers, directories, files, ports/GPU mapping, fixtures, baseline numbers, deferred unblock reasons, and `bn_status_report.py --plan=D` from D0-035..D0-068. The summary intentionally did not read D-0 and only surfaced its filename at summary lines 732/775.
- **D-001/SD-A002** **D-5 selection policy is now concrete.** Add Beta(1,1) initialization, staged 10/50/200 evaluation, credible-lower-bound versus promoted median, per-domain deltas, bottom-quartile retirement, LUCB formula, gamma-ratio sampling, empty-arm/zero-sample behavior, and MSI-as-secondary-only policy (D5-005..D5-010/D5-014..D5-027/D5-034/D5-037). This resolves most of summary uncertainty item 7 (`04-D...:752`).
- **D-001/SD-A003** **D-5 reproducibility/evidence is stronger.** Add same-seed byte identity, exact 10-arm rates, 500 pulls, 100 trials, bound x2, >=85% best-arm selection, >=15 tests, and exact event payloads (D5-026/D5-028..D5-033/D5-038..D5-042).
- **D-001/SD-A004** **D-5 security/fallback detail.** Add user approval before `statrs`, incomplete-beta fallback, config immutability, graceful empty input, and archive-lock concurrency exclusion with D-4 (D5-003/D5-015/D5-018/D5-037).
- **D-001/SD-A005** **D-6 schema and lifecycle.** Add complete task/outcome/source/status/reason schemas, dual indexes, active-only query, rewrite-on-retire behavior, signed shared-subset regression, gate reason, and exact event payloads (D6-009..D6-024).
- **D-001/SD-A006** **D-6 thresholds resolved.** “Uniformly passing” means age `>6 months`, pass rate `>0.95`, and at least 20 evaluations; regression allows `delta >= -5%`; n-gram contamination is 9-gram Jaccard `>0.3` against a Blake3 Bloom (D6-019..D6-025). This resolves most of summary uncertainty item 8 (`04-D...:753`).
- **D-001/SD-A007** **D-6 degraded mode and seed composition.** Add the empty-Bloom skip behavior, exact 25-task source mix, test tables, and the distinction between 20-file smoke presence versus 25-file acceptance (D6-026..D6-033/D6-039..D6-046).
- **D-001/SD-A008** **D-6 evaluation secrecy amendments.** Add Arbor `B_dev/B_test` detached-worktree prior art and mandatory `.git` destruction/reinitialization for repo-derived fixtures; runtime leakage control remains H-47 (D6-049..D6-052). This is a material security addition to summary lines 568-583/618-621.
- **D-001/SD-A009** **D-12 full operationalization contract.** Add the complete bounded `MeasurableProcedure`, TOML parser/error format, actual-vs-expected tri-state diff, keyword claim detector, per-checker timeout, human-review prompt, fresh execution/no cache, exact payloads, and eight-test floor (D12-009..D12-038).
- **D-001/SD-A010** **D-23 benchmark implementation.** Add exact 70-positive/30-negative human-labeled fixtures, report confusion-matrix fields/formulas, `GatingVerdict`, CLI/CI reachability, benchmark events, dated report artifact, and `Convergent` cloud expectation (D23-005..D23-019).
- **D-001/SD-A011** **D-26 surface is much larger.** Add `get_bai_arms`, `get_canary_report`, `get_held_out_status`, plus `TripwireStatusStrip`, `BaiArmsTable`, `HeldOutPanel`, and `ArchiveVerifyGauge`; add filters, colors, click behavior, async verification, bounded recent reads, and exact three-second tripwire SLA (D26-005..D26-032). The summary listed only three commands and two named new components at lines 383-399/529-531.
- **D-001/SD-A012** **D-27 is a governance UI, not only charts.** Add `get_discovery_queue`, `ratify_discovery`, `reject_discovery`, `DiscoveryQueueDrawer`, and `CheckerStatusPanel`, including reviewer-required write actions, quarantine links, configurable thresholds, placement, and evidence snippets (D27-008..D27-025/D27-030). The summary listed only four commands/four components at lines 401-415/529-531.
- **D-001/SD-A013** **D-29 contains concrete host-side symbols.** Add exact Rust types/functions/line anchors, `HierarchicalBaiRoundCompleted`, two environment gates, default-flat behavior, task-runner branch, and the explicit claim that Path A host dispatch shipped while the Gibbs comparison remains pending (D29-013..D29-023).
- **D-001/SD-A014** **D-29 model proposal is concrete.** Add Beta-per-arm/Gamma-hyperprior/Gibbs shape and the five-family, ten-arm, within-family-correlated, 1000-pull benchmark (D29-003/D29-007/D29-008), resolving part of summary uncertainty item 20 (`04-D...:765`).

#### Corrections

- **D-001/SD-C001** **Baby-step block number.** Correct summary line 187’s “block 7” to **block 9** for this deep evidence set: D0-047 and D12-004/D12-027 agree on eight existing blocks plus ninth `MEASURABLE_PROCEDURE`.
- **D-001/SD-C002** **D-23 comparison mode.** Summary line 345 says D-23 runs probe-enabled versus probe-disabled. The processed D-23 original does **not** specify that ablation; it replays every fixture through D-14 and compares the flag to labels (D23-007). Treat enabled/disabled as summary-only unless another original proves it.
- **D-001/SD-C003** **D-23 provenance wording.** Summary line 343 says “Anthropic/Sharma”; this original says Sharma et al. public probes and does not name an Anthropic dataset identifier (D23-001/D23-004).
- **D-001/SD-C004** **D-26 lineage target.** Summary line 390 says it cross-links to a “baby-step archive.” The original specifies an `ArchiveEntry` parent-hash DAG and flips to `MutationArchivePanel` detail, not an explicit baby-step-archive cross-link (D26-012/D26-013).
- **D-001/SD-C005** **D-26 prerequisites.** Qualify summary line 399/matrix line 652 (`D-1..D-8`) with the original’s stronger D-1..D-9 shipped prerequisite (D26-002), while retaining the conflict with D0-030.
- **D-001/SD-C006** **D-26 poll interval.** Summary uncertainty line 764 says polling latency is unquantified. The tripwire acceptance SLA is explicitly **3 seconds** (D26-031); other polls remain unquantified.
- **D-001/SD-C007** **D-26 PPM/consensus display.** Summary lines 391-393 mirror the objective, but the detailed original task/command/component contract lacks a PPM or consensus query. Keep these as objective-level requirements with an implementation-surface ambiguity (D26-001 versus D26-005..D26-021).
- **D-001/SD-C008** **D-29 status.** Summary lines 432-444/655/766 present D-29 only as deferred and assert zero shipped features. The original still has deferred acceptance, but explicitly claims Path A host-side Rust dispatch shipped (D29-016/D29-023). Correct representation is mixed evidence/partial productization with research closure pending, not cleanly `DONE`.
- **D-001/SD-C009** **D-29 closure outcome.** Summary treats Path A and Path B as both open. The embedded wiring comment states Path A was the outcome and Path B was not (D29-016), although unchecked acceptance prevents treating the overall session as closed.
- **D-001/SD-C010** **D-6 privacy/security coverage.** Summary lines 618-621 say no distinct privacy feature. That remains true for user PII, but D6-051/D6-052 add a distinct confidentiality/leakage safeguard for evaluation fixtures: remove recoverable Git history at build time.
- **D-001/SD-C011** **Held-out auto-retirement.** Summary lines 99/601 phrase six-month uniform passing broadly. Deep rules require all of strict age, `>0.95`, and `>=20` evaluations (D6-020/D6-021); a TTL alone does not retire.
- **D-001/SD-C012** **D-5 benchmark wording.** Summary line 600 reports a 50-round Audibert gate. Preserve that index gate (D0-009), but also record the original’s separate 500-pull/100-trial/bound-x2 integration test and 50-cycle cloud bootstrap (D5-033/D5-042).
- **D-001/SD-C013** **Dashboard component/command counts.** Replace any interpretation that D-26 has only three commands/two components or D-27 only four commands/four components: their originals require 6+6 and 7+6 respectively (D26-021/D26-029; D27-020/D27-028).

---

## Ledger D-002 — source atoms

Source file: `docs/thefinalglance/feature-discovery/deep/DEEP-D-002.discovery.md`
Plans: D-1, D-8, D-9, D-10, D-14, D-21, D-24, D-28
Manifest: 8 files, 1855 lines, 87723 bytes

### D-1_reasoning_template_schema.md — 57 atoms (45 E + 12 N unresolved)

| ID | Class | Category | Atomic feature or behavior | Anchor |
|---|---|---|---|---|
| D1-E01 | [E] | CAP,DATA,SEC | Replace prompt-string reasoning patterns with bounded typed ReasoningTemplate records and bootstrap an append-only ArchiveEntry chain as the mutation substrate. | D-1_reasoning_template_schema.md:3-9 |
| D1-E02 | [E] | OBS,DEP | The plan records the then-current gap as no ReasoningTemplate, roughly 25 TOML prompt files plus inline agent strings, and no hash, lineage, or archive lifecycle. | D-1_reasoning_template_schema.md:22-30 |
| D1-E03 | [E] | SEC,CON | Extend L1A from data tables to data tables plus typed reasoning templates; the closed schema is the intended capacity bound. | D-1_reasoning_template_schema.md:39-42 |
| D1-E04 | [E] | DEP,FLOW | Estimate 1-2 sessions; require C-15 and C-21 DONE, with spec-grounder verification before EXECUTING; use C-15 Blake3 and C-21 MistakeBook BLOCK inputs. | D-1_reasoning_template_schema.md:10-16 |
| D1-E05 | [E] | DET,DEP | Parallel-safe with D-19 and D-25; not parallel-safe with D-2 through D-9 because those consume the new records. | D-1_reasoning_template_schema.md:17-20 |
| D1-E06 | [E] | DEP,TEST | Ingest SELF-DISCOVER and Statistical Limits interfaces, and require both files before TESTING_FIRST. | D-1_reasoning_template_schema.md:76-85 |
| D1-E07 | [E] | DATA | ReasoningTemplate fields are id, domain, pattern_kind, template_body as BoundedString<1024>, eligibility_predicate, verifier_required, content_hash, created_at, parent_id, and archive_status. | D-1_reasoning_template_schema.md:89-95 |
| D1-E08 | [E] | DATA,SEC | Domain is the L3-locked closed enum Code, Math, Prose, Mixed. | D-1_reasoning_template_schema.md:96-97 |
| D1-E09 | [E] | DATA,SEC | PatternKind is the L3-locked closed enum Decomposition, Reflection, Verification, Synthesis. | D-1_reasoning_template_schema.md:98-99 |
| D1-E10 | [E] | DATA,SEC,FAIL | VerifierKind is the L3-locked closed enum Compile, Test, Prm, Refuse; an agent may not weaken Prm to Refuse. | D-1_reasoning_template_schema.md:100-101 |
| D1-E11 | [E] | DATA,FLOW | ArchiveStatus values are Candidate, Promoted, Retired, Frozen; RetirementReason values are Degradation, Superseded, Banned, MistakeBookTrigger. | D-1_reasoning_template_schema.md:102-103 |
| D1-E12 | [E] | DATA | TemplateId is a Uuid newtype and Blake3Hash is a 32-byte-array newtype. | D-1_reasoning_template_schema.md:104 |
| D1-E13 | [E] | DATA,FAIL,CON | BoundedString<N> validates in TryFrom, rejects lengths greater than N without panic, and returns BoundedStringError. | D-1_reasoning_template_schema.md:105-106 |
| D1-E14 | [E] | DATA,SEC,CON | Predicate is the closed, non-LLM enum Always, MatchDomain, MinConfidence, AndAll, OrAny; recursive fan-out is bounded by vector length. | D-1_reasoning_template_schema.md:107-109,265-266 |
| D1-E15 | [E] | DATA,FAIL,CON | All types derive Debug, Clone, Serialize, Deserialize; TemplateError has BoundedStringExceeded, HashMismatch, ParentNotFound, StatusTransitionInvalid; new code forbids unwrap and expect. | D-1_reasoning_template_schema.md:110-112,353 |
| D1-E16 | [E] | DET,SEC | compute_content_hash hashes body bytes concatenated with the supplied parent hash, or 32 zero bytes for a root, using Blake3; identical inputs must produce the same hash. | D-1_reasoning_template_schema.md:114-117 |
| D1-E17 | [E] | FAIL,SEC | verify_hash recomputes a template hash and returns HashMismatch on divergence; it may not compare cached values. | D-1_reasoning_template_schema.md:118-119,259-260 |
| D1-E18 | [E] | TEST,DET | Hash tests require equality for identical body/parent, inequality when body or parent differs, and exactly 32 output bytes. | D-1_reasoning_template_schema.md:120-121 |
| D1-E19 | [E] | DATA,FLOW | ArchiveEntry fields are self_hash, parent_hash, template, staged_eval_result, promoted_at, retired_at, retirement_reason. | D-1_reasoning_template_schema.md:123-128 |
| D1-E20 | [E] | DATA,OBS | StagedEvalResult carries pass/total counts at stages 10, 50, and 200 plus cross_domain_deltas by Domain; D-5 and D-7 populate the placeholder. | D-1_reasoning_template_schema.md:129-132 |
| D1-E21 | [E] | DATA,FLOW,FAIL,SEC | ArchiveChain has storage_path and entries; load, append, and verify_chain are required; append never deletes, while verification walks parent links and recomputes hashes. | D-1_reasoning_template_schema.md:133-135 |
| D1-E22 | [E] | DATA,SEC | Store one serialized ArchiveEntry per JSONL line at .elai_cc/archive/reasoning-templates.jsonl and open writes in O_APPEND mode only. | D-1_reasoning_template_schema.md:136-137 |
| D1-E23 | [E] | DET,DEP | Serialize concurrent appends with an fs2 advisory lock; if fs2 is absent, it is the sole optional dependency and requires user confirmation before addition. | D-1_reasoning_template_schema.md:138-140 |
| D1-E24 | [E] | INT,DATA,OBS | Bump SSB SCHEMA_VERSION 47 to 48 and add ArchiveEntryAdded, ArchiveEntryRetired, TemplatePromoted, and TemplateCandidateRejected with their listed payload fields, SQL serialization, and round-trip coverage. | D-1_reasoning_template_schema.md:142-152 |
| D1-E25 | [E] | CAP,FLOW,DATA | elai template migrate scans prompt-template TOML and agent-definition Markdown, extracts unique patterns, hashes each, creates Frozen ReasoningTemplate entries, and writes the archive JSONL. | D-1_reasoning_template_schema.md:154-159 |
| D1-E26 | [E] | DET,OBS | Migration is idempotent on unchanged inputs, skips existing hashes, and reports extracted, skipped_duplicate, and skipped_too_long counts. | D-1_reasoning_template_schema.md:160-162 |
| D1-E27 | [E] | DATA,CON | TemplateRegistry maps TemplateId to ReasoningTemplate with FxHashMap, explicitly not std HashMap. | D-1_reasoning_template_schema.md:164-167 |
| D1-E28 | [E] | FLOW | TemplateRegistry.load admits only Promoted and Frozen archive entries. | D-1_reasoning_template_schema.md:168-169 |
| D1-E29 | [E] | FLOW,DEP | select_template filters by domain, pattern kind, and a closed-form eligibility predicate, then returns the first match; replacing task-runner prompt literals is deferred to D-2. | D-1_reasoning_template_schema.md:170-174 |
| D1-E30 | [E] | CFG | Add a commented reasoning.templates config section; absent values default archive_path to .elai_cc/archive/reasoning-templates.jsonl and bounded_string_max to 1024. | D-1_reasoning_template_schema.md:176-180 |
| D1-E31 | [E] | TEST | Unit coverage includes 1024/1025 string boundaries, repeatable hash, body-tamper detection, append/load identity, broken-parent detection, and predicate truth tables. | D-1_reasoning_template_schema.md:182-192 |
| D1-E32 | [E] | FLOW,SEC,TEST | Candidate to Promoted and Promoted to Retired are valid; Frozen and Retired are immutable terminal states, with no force-override flag. | D-1_reasoning_template_schema.md:193-194,261-262 |
| D1-E33 | [E] | TEST,DATA | The contract test requires public schema 48, all four SSB variants and SQL round trips, allow-listed closed enums, and append-plus-verify end to end. | D-1_reasoning_template_schema.md:197-203 |
| D1-E34 | [E] | TEST,OBS | Manual migration smoke reports all three counts and inspects the first three JSONL entries for valid JSON, 64-character hashes, and Frozen status. | D-1_reasoning_template_schema.md:205-209 |
| D1-E35 | [E] | CON,OBS | D-1-decisions.md must record why Blake3, why JSONL instead of SQLite, and why migrated templates are Frozen rather than Promoted. | D-1_reasoning_template_schema.md:211-218 |
| D1-E36 | [E] | FLOW,OBS,DEP | During MEMORIZING, memory-writer must emit project_d1_reasoning_template.md containing schema, lifecycle, and SSB variants for D-2 through D-9. | D-1_reasoning_template_schema.md:220-222 |
| D1-E37 | [E] | DEP,DET | created_at must use injected SeededClock and never SystemTime::now; MistakeBook retirement feed is planned for D-4, not wired in D-1. | D-1_reasoning_template_schema.md:239-248 |
| D1-E38 | [E] | CAP,INT | Reachability is through elai template migrate, registry startup loading, and future D sessions reading ArchiveChain through an in-memory FxHashMap after JSONL load. | D-1_reasoning_template_schema.md:250-254 |
| D1-E39 | [E] | CON,SEC | ArchiveChain may expose no delete method; content hashing must never use SHA-256 or Blake2; Predicate evaluation may not call an LLM. | D-1_reasoning_template_schema.md:255-266 |
| D1-E40 | [E] | OBS,TEST | A migration run must yield N ArchiveEntryAdded rows matching its reported count, all Frozen, with zero TemplatePromoted and TemplateCandidateRejected rows; the whole artifact must parse and verify. | D-1_reasoning_template_schema.md:267-274 |
| D1-E41 | [E] | TEST,FAIL | Acceptance requires public schema 48, a migration warning for an old version-47 SSB database, error-return rather than panic, at least eight new unit tests, contract tests, and a clean workspace build. | D-1_reasoning_template_schema.md:325-341 |
| D1-E42 | [E] | TEST,FAIL,SEC | Cloud smoke A verifies registry loading and task completion with a migrated template; smoke B manually tampers template_body and requires HashMismatch plus orchestrator refusal to use it. | D-1_reasoning_template_schema.md:342-351 |
| D1-E43 | [E] | TEST,OBS | Real-repository migration must migrate at least as many templates as the template-directory TOML file count, be idempotent on the second run, and produce the decision and memory artifacts. | D-1_reasoning_template_schema.md:335-354 |
| D1-E44 | [E] | CON,DEP | Out of scope are PPM wiring, BAI promotion, user-facing archive verification, per-template performance metrics, and enforcement of documented L3 locks; these belong to D-2, D-5, D-8, D-7, and D-9 respectively. | D-1_reasoning_template_schema.md:379-389 |
| D1-E45 | [E] | DEP,DET | After D-1, D-2 and D-8 may start in parallel: D-2 adds verifier substrate and D-8 exposes verify_chain as CLI. | D-1_reasoning_template_schema.md:391-395 |
| D1-N01 | [A] | DEP | The original never declares PLANNED, IN PROGRESS, or DONE; PLANNED comes only from the orientation summary, and this pass did not inspect implementation. | D-1_reasoning_template_schema.md:1-395 |
| D1-N02 | [C] | DATA,SEC | D-1 defines template content_hash from body plus parent hash, while D-8 defines archive self-hash from body plus parent hash plus mutation_proposal_text. | D-1_reasoning_template_schema.md:114-121; D-8_crypto_archive_verify.md:16-28 |
| D1-N03 | [A] | DATA | Neither ReasoningTemplate nor ArchiveEntry declares mutation_proposal_text, leaving the D-8 hash input without a storage field in D-1's schema. | D-1_reasoning_template_schema.md:91-95,125-128; D-8_crypto_archive_verify.md:16-28 |
| D1-N04 | [A] | FLOW | StatusTransitionInvalid and transition tests exist, but the implementing transition type or method and the append-only representation of promotion/retirement are not specified. | D-1_reasoning_template_schema.md:111-112,193-194 |
| D1-N05 | [A] | DATA,TEST | Predicate tests permit adding Never if needed, although Never is absent from the declared closed enum; its normative membership is unresolved. | D-1_reasoning_template_schema.md:107-109,191-192 |
| D1-N06 | [A] | CFG,DATA | BoundedString<1024> is compile-time-sized while bounded_string_max is runtime-configurable; behavior when configuration differs from 1024 is not defined. | D-1_reasoning_template_schema.md:92-95,176-180 |
| D1-N07 | [A] | DET,FLOW | select_template returns the first eligible FxHashMap result, but tie ordering and a stable selection key are not specified. | D-1_reasoning_template_schema.md:164-172 |
| D1-N08 | [A] | FLOW,SEC | The file requires immutable append-only storage and mutable lifecycle states but does not define whether transitions append successor entries, append events, or rewrite prior JSONL rows. | D-1_reasoning_template_schema.md:102-103,123-137,193-194 |
| D1-N09 | [A] | CAP,DET | Pattern extraction and deduplication rules for free-form TOML/Markdown are not defined beyond uniqueness by resulting hashes. | D-1_reasoning_template_schema.md:154-162 |
| D1-N10 | [A] | DATA,SEC | `ReasoningTemplate.content_hash`, `ArchiveEntry.self_hash`, and the value recomputed by `verify_chain` are all named, but D-1 never states whether the two hashes must be identical or whether an entry has a second envelope hash. | D-1_reasoning_template_schema.md:91-95,114-128,133-137 |
| D1-N11 | [A] | DATA,FLOW | Template lineage uses `parent_id: Option<TemplateId>` while archive lineage uses `parent_hash: Option<Blake3Hash>`; lookup, validation, and consistency rules between those two parent references are not specified. | D-1_reasoning_template_schema.md:91-95,125-135 |
| D1-N12 | [A] | DATA,SEC,CON | The source calls predicate recursion bounded by vector length, but supplies no maximum vector length, nesting depth, cycle defense, or evaluation budget; the effective capacity bound is therefore unspecified. | D-1_reasoning_template_schema.md:107-109,265-266 |

### D-8_crypto_archive_verify.md — 41 atoms (29 E + 12 N unresolved)

| ID | Class | Category | Atomic feature or behavior | Anchor |
|---|---|---|---|---|
| D8-E01 | [E] | CAP,SEC,OBS | Expose D-1 ArchiveChain.verify_chain as a user CLI and CI check, emit ArchiveVerifyResult, and defend against an agent rewriting its audit history. | D-8_crypto_archive_verify.md:3-8 |
| D8-E02 | [E] | DEP,DET | Require D-1, with spec-grounder verification; run in parallel with D-2 through D-7 because D-8 reads but does not extend ArchiveChain beyond reporting. | D-8_crypto_archive_verify.md:9-12 |
| D8-E03 | [E] | DATA,SEC | The context defines each self hash as Blake3 over template_body, parent_hash, and mutation_proposal_text. | D-8_crypto_archive_verify.md:16-19,25-27 |
| D8-E04 | [E] | FAIL,SEC | Chain verification recomputes every hash, requires each parent to be an earlier existing entry, and reports hash/link/parent failures. | D-8_crypto_archive_verify.md:25-28 |
| D8-E05 | [E] | CAP,CFG | ArchiveVerifyCmd contains archive_path, json_output, and fail_fast. | D-8_crypto_archive_verify.md:52-55 |
| D8-E06 | [E] | DATA,OBS | ArchiveVerifyReport contains total_entries, verified, and a vector of ArchiveVerifyError. | D-8_crypto_archive_verify.md:56-57 |
| D8-E07 | [E] | DATA,FAIL | ArchiveVerifyError variants are HashMismatch with expected/computed and index, BrokenParent with index/hash, DuplicateHash with current/prior indexes, and MalformedJson with index/error. | D-8_crypto_archive_verify.md:58-61 |
| D8-E08 | [E] | FLOW,FAIL | run loads the archive, calls verify_chain, aggregates all errors by default, and short-circuits on the first error only when fail_fast is true. | D-8_crypto_archive_verify.md:62-66 |
| D8-E09 | [E] | FAIL,OBS | Output is human-readable or JSON; zero errors exits 0, any error exits 1, and an ArchiveVerifyResult row is emitted regardless of exit code. | D-8_crypto_archive_verify.md:67-69 |
| D8-E10 | [E] | CAP,CFG | Add SlashCommand::ArchiveVerify and dispatch elai archive verify with path, json, and fail-fast flags; default to the D-1 archive path. | D-8_crypto_archive_verify.md:71-77 |
| D8-E11 | [E] | INT,DATA,OBS | Bump SSB schema 53 to 54 and add ArchiveVerifyResult with audit_id, total_entries, verified, error_count, and errors, plus round-trip coverage. | D-8_crypto_archive_verify.md:79-82 |
| D8-E12 | [E] | INT,TEST,SEC | archive-verify.yml runs on push and pull_request to main plus nightly cron, builds release elai_cli, and invokes archive verify with fail-fast so integrity failure fails CI. | D-8_crypto_archive_verify.md:84-89 |
| D8-E13 | [E] | INT,CFG | CI uses windows-latest and ubuntu-latest, and skips when the commit message contains [ci skip]. | D-8_crypto_archive_verify.md:90-92 |
| D8-E14 | [E] | TEST,FAIL | Tamper scenario A edits entry 3 template_body and must report HashMismatch at entry 3 and exit 1. | D-8_crypto_archive_verify.md:94-98 |
| D8-E15 | [E] | TEST,FAIL | Tamper scenario B points entry 4 at a nonexistent parent and must report BrokenParent and exit 1. | D-8_crypto_archive_verify.md:99-100 |
| D8-E16 | [E] | TEST,FAIL | Tamper scenario C duplicates entry 2's self hash at entry 5 and must report DuplicateHash and exit 1. | D-8_crypto_archive_verify.md:101-102 |
| D8-E17 | [E] | TEST,FAIL | Tamper scenario D inserts malformed JSON and must report MalformedJson while continuing when fail_fast is false. | D-8_crypto_archive_verify.md:103-104 |
| D8-E18 | [E] | TEST,OBS | A clean five-entry archive exits 0 and emits ArchiveVerifyResult with error_count zero. | D-8_crypto_archive_verify.md:105-106 |
| D8-E19 | [E] | TEST,DET,CON | An ignored manual performance test synthesizes 10,000 entries and requires verification within five seconds and O(n), not O(n-squared), behavior. | D-8_crypto_archive_verify.md:108-113 |
| D8-E20 | [E] | FAIL,OBS | Documentation must cover CLI/CI and recovery: after failure, treat the last N agent actions as suspect and roll back by forensic chain walk. | D-8_crypto_archive_verify.md:115-119 |
| D8-E21 | [E] | CON,CFG | The decision record explains separate CLI/CI roles, Blake3 speed/consistency, and full-report default versus early-exit fail-fast mode. | D-8_crypto_archive_verify.md:121-128 |
| D8-E22 | [E] | DEP,CAP | The command consumes ArchiveChain, SlashCommand, and SsbWriter and is reachable from direct CLI, CI, and cargo-run development invocation. | D-8_crypto_archive_verify.md:130-145 |
| D8-E23 | [E] | CON,FAIL | I/O may not unwrap; clean must not always fail; tamper must not silently pass; json and fail-fast flags must be functional. No retry behavior is defined. | D-8_crypto_archive_verify.md:146-151 |
| D8-E24 | [E] | OBS,TEST | Each clean or tampered invocation emits exactly one ArchiveVerifyResult; the workflow artifact must report a PR commit status. | D-8_crypto_archive_verify.md:152-157 |
| D8-E25 | [E] | TEST | Deterministic checks cover schema 54, command/types/event/workflow presence, clean verification, synthetic corrupt-line detection, workspace build, contract tests, and the ignored performance test. | D-8_crypto_archive_verify.md:159-194 |
| D8-E26 | [E] | TEST,FAIL | Acceptance additionally requires parseable JSON and demonstrates that default collection reports at least two errors when at least two tampers exist. | D-8_crypto_archive_verify.md:196-211 |
| D8-E27 | [E] | TEST,SEC | Cloud smoke verifies a clean archive after live promotion, then a manual body corruption that exits 1 with HashMismatch. | D-8_crypto_archive_verify.md:212-217 |
| D8-E28 | [E] | CAP,INT | Files include the CLI module/dispatch, SSB update, CI workflow, contract and performance tests, decision record, and recovery documentation. | D-8_crypto_archive_verify.md:219-229 |
| D8-E29 | [E] | CON,DEP | D-25's ten-case suite and D-9 lock enforcement are out of scope; archive compaction or sled migration is deferred because JSONL is expected to scale to about 10,000 entries. | D-8_crypto_archive_verify.md:231-241 |
| D8-N01 | [C] | DATA,SEC | Its three-part hash formula conflicts with D-1's two-part compute_content_hash formula and depends on mutation_proposal_text absent from D-1's records. | D-8_crypto_archive_verify.md:16-28; D-1_reasoning_template_schema.md:91-95,114-128 |
| D8-N02 | [C] | FAIL | Context names BrokenLink and DanglingParent, while the normative error enum names BrokenParent and has no BrokenLink or DanglingParent variant. | D-8_crypto_archive_verify.md:25-28,58-61 |
| D8-N03 | [A] | FAIL,FLOW | ArchiveChain load precedes verification, but malformed JSON is expected to be an aggregatable per-line error; how load preserves later lines is unspecified. | D-8_crypto_archive_verify.md:62-66,103-104 |
| D8-N04 | [A] | FAIL | Missing file, permission denial, unreadable path, and empty-archive behavior are not defined. | D-8_crypto_archive_verify.md:52-69 |
| D8-N05 | [A] | OBS,FAIL | The report promises an SSB row regardless of exit code, but no path is specified for emitting it when loading or SSB writing itself fails. | D-8_crypto_archive_verify.md:62-69 |
| D8-N06 | [A] | FAIL,SEC | Recovery identifies suspect actions and a forensic walk but does not define the trusted restore source, exact rollback command, or how the append-only file is repaired. | D-8_crypto_archive_verify.md:115-119 |
| D8-N07 | [A] | INT | The plan places workflows beneath .elai_cc/.github; it does not state whether .elai_cc is the repository root or how a parent-repository CI installation discovers them. | D-8_crypto_archive_verify.md:84-92,219-229 |
| D8-N08 | [C] | DEP,FAIL | D-8 delegates to D-1 `verify_chain`, yet D-1 exposes only template-level `HashMismatch`/`ParentNotFound` errors and does not define duplicate-hash or malformed-line aggregation; D-8 requires four richer per-entry error variants without specifying the adapter boundary. | D-8_crypto_archive_verify.md:52-69,94-106; D-1_reasoning_template_schema.md:110-119,133-137 |
| D8-N09 | [A] | DET,FLOW,SEC | Verification concurrency is undefined: the source does not say whether the reader takes D-1's advisory lock, snapshots the file, or can observe a partial line while another process appends. | D-8_crypto_archive_verify.md:62-69; D-1_reasoning_template_schema.md:133-140 |
| D8-N10 | [A] | SEC,INT | `[ci skip]` disables the integrity workflow, but no authorization policy or protected-branch control constrains who may use that bypass. | D-8_crypto_archive_verify.md:84-92 |
| D8-N11 | [A] | DATA,FAIL | Error `entry_index` semantics are unspecified (zero- versus one-based, and whether malformed physical lines count), which affects machine consumers and the recovery walk. | D-8_crypto_archive_verify.md:56-61,94-106 |
| D8-N12 | [C] | TEST | Acceptance calls the five cases “5 tamper scenarios” although one of the five is explicitly the clean control and only four are tampers. | D-8_crypto_archive_verify.md:94-106,198-204 |

### D-9_l3_frozen_surfaces.md — 52 atoms (39 E + 13 N unresolved)

| ID | Class | Category | Atomic feature or behavior | Anchor |
|---|---|---|---|---|
| D9-E01 | [E] | CAP,SEC | Structurally enforce expanded L3-frozen surfaces with a pre-commit hook, LockedKeysAuditor, LockedKeysTouch event, and a blocked agent edit path. | D-9_l3_frozen_surfaces.md:3-8 |
| D9-E02 | [E] | DEP,DET | Require D-1 through D-8 DONE; run in parallel with D-10 through D-18; coordinate hook ordering with C-27 rather than running that overlapping change independently. | D-9_l3_frozen_surfaces.md:9-15 |
| D9-E03 | [E] | SEC | Freeze staged-evaluation harness code. | D-9_l3_frozen_surfaces.md:19-22 |
| D9-E04 | [E] | SEC | Freeze tripwire thresholds and detection code. | D-9_l3_frozen_surfaces.md:22,31-32 |
| D9-E05 | [E] | SEC | Freeze MSI/BAI scoring-function code. | D-9_l3_frozen_surfaces.md:23,33 |
| D9-E06 | [E] | SEC | Freeze archive maintenance and parent-selection algorithm code. | D-9_l3_frozen_surfaces.md:24 |
| D9-E07 | [E] | SEC | Freeze the held-out task-selection algorithm. | D-9_l3_frozen_surfaces.md:25 |
| D9-E08 | [E] | SEC | Freeze cross-domain canary family definitions. | D-9_l3_frozen_surfaces.md:26 |
| D9-E09 | [E] | SEC | Freeze cryptographic hash choice to Blake3. | D-9_l3_frozen_surfaces.md:27 |
| D9-E10 | [E] | SEC | Freeze the AllowedConfigKey bound so an agent cannot add variants. | D-9_l3_frozen_surfaces.md:28 |
| D9-E11 | [E] | SEC | Freeze every template's verifier_required field against self-weakening. | D-9_l3_frozen_surfaces.md:29 |
| D9-E12 | [E] | SEC | Freeze the Domain enum against self-added domains. | D-9_l3_frozen_surfaces.md:30 |
| D9-E13 | [E] | SEC,CFG | Freeze tripwire config keys kl_spec_tau, rho_delta, iqr_mult, ppl_sigma, ddm thresholds, consensus_fire_count, and window_size. | D-9_l3_frozen_surfaces.md:31-32 |
| D9-E14 | [E] | SEC,CFG | Freeze BAI eval_budget, ci_alpha, and per-family delta. | D-9_l3_frozen_surfaces.md:33 |
| D9-E15 | [E] | SEC,CFG | Freeze held-out ttl_months, ngram_size, and jaccard_threshold. | D-9_l3_frozen_surfaces.md:34 |
| D9-E16 | [E] | SEC,CFG | Freeze canary regression_threshold and min_samples. | D-9_l3_frozen_surfaces.md:35 |
| D9-E17 | [E] | SEC,FLOW | Enforcement has three layers: authoritative documentation/registry, pre-commit rejection, and a runtime auditor over attempted writes/events. | D-9_l3_frozen_surfaces.md:42-46 |
| D9-E18 | [E] | DATA,SEC | locked_keys_registry.toml records locked_path with reason/override key, locked_symbol with file/reason, and locked_toml_key with file/section/keys/reason. | D-9_l3_frozen_surfaces.md:75-104 |
| D9-E19 | [E] | TEST,SEC | Populate the registry with all 14 named surface classes and their per-file/per-symbol details. | D-9_l3_frozen_surfaces.md:105-106 |
| D9-E20 | [E] | FLOW,SEC | locked_keys_check.sh parses the registry and cached name-only diff; a locked path without a matching L3_OVERRIDE tag exits 1 with a structured error. | D-9_l3_frozen_surfaces.md:108-113 |
| D9-E21 | [E] | FLOW,SEC | The same check inspects diff bodies for locked symbols and parses TOML section/key changes. | D-9_l3_frozen_surfaces.md:114-116 |
| D9-E22 | [E] | DATA,SEC | LockedKeysAuditor owns a registry, SSB sender, and audit UUID; LockedKeysRegistry stores vectors of paths, symbols, and TOML keys. | D-9_l3_frozen_surfaces.md:118-122 |
| D9-E23 | [E] | FLOW,SEC | audit_diff returns an event per unauthorized match; audit_runtime_write performs the analogous check before a pending runtime write. | D-9_l3_frozen_surfaces.md:122-127 |
| D9-E24 | [E] | FLOW,SEC,OBS | task_runner's PreToolUse/Tool::Write path invokes the auditor, blocks a touched locked surface, and emits LockedKeysTouch. | D-9_l3_frozen_surfaces.md:129-134 |
| D9-E25 | [E] | INT,DATA,OBS | Bump SSB schema 54 to 55; LockedKeysTouch carries path/symbol/TOML location/override flag/audit ID; L3OverrideRequested carries key/requester/a 512-bound justification. | D-9_l3_frozen_surfaces.md:136-144 |
| D9-E26 | [E] | SEC,OBS | L3OverrideRequested is informational documentation of a human decision, explicitly not an in-code permission-grant mechanism. | D-9_l3_frozen_surfaces.md:140-143 |
| D9-E27 | [E] | CFG,SEC | A valid override is a registered L3_OVERRIDE key in the commit body with at least a 40-character justification; valid overrides allow the commit, others block. | D-9_l3_frozen_surfaces.md:146-159 |
| D9-E28 | [E] | OBS,SEC | Documentation lists all 14 surfaces, points to the registry as authoritative, documents the override protocol, and cites its research grounding. | D-9_l3_frozen_surfaces.md:161-166 |
| D9-E29 | [E] | TEST | Unit tests cover unauthorized and overridden path edits, a tripwire TOML-key edit with populated fields, and an unrelated path. | D-9_l3_frozen_surfaces.md:168-175 |
| D9-E30 | [E] | TEST,SEC | Integration tests create a temporary git repository and require exit 1 without an override and exit 0 with one. | D-9_l3_frozen_surfaces.md:177-181 |
| D9-E31 | [E] | INT,SEC | locked-keys-audit.yml runs the script on push and pull_request to main and fails violations even when local hooks are bypassed. | D-9_l3_frozen_surfaces.md:183-187 |
| D9-E32 | [E] | CON,SEC | The decision record justifies structural enforcement, the 40-character floor, and using both dev-time and runtime enforcement. | D-9_l3_frozen_surfaces.md:189-199 |
| D9-E33 | [E] | FAIL,SEC,CON | Registry parse error fails closed; override text shorter than 40 characters blocks; all path/symbol/TOML layers are mandatory; clean or valid override alone exits 0. No retry is defined. | D-9_l3_frozen_surfaces.md:223-229 |
| D9-E34 | [E] | OBS,TEST | An unauthorized tripwire edit must produce one LockedKeysTouch, exit 1, and block the commit/runtime write; registry and CI artifacts must parse/exist. | D-9_l3_frozen_surfaces.md:230-235 |
| D9-E35 | [E] | TEST,OBS | Cloud smokes cover unauthorized pre-commit block/event, valid override allow plus L3OverrideRequested, and runtime Tool::Write to bai.rs blocked before execution. | D-9_l3_frozen_surfaces.md:276-298 |
| D9-E36 | [E] | DEP | Existing C-21 hook/script and all Wave-1 files are consumed; if no GitDiff/git2 support exists, shell out to git rather than adding an unapproved dependency. | D-9_l3_frozen_surfaces.md:212-218 |
| D9-E37 | [E] | CON,DEP | D-25 combined tamper cases are out of scope; registry signing and multi-party override approval are deferred, relying on git signing and one operator in the current threat model. | D-9_l3_frozen_surfaces.md:319-331 |
| D9-E38 | [E] | DEP,TEST | Before implementation, ingest and verify the Everitt reward-tampering, Hubinger mesa-optimization, and Cohen Boxed Myopic paper interfaces; acceptance requires all three artifacts. | D-9_l3_frozen_surfaces.md:64-73,270-273,297 |
| D9-E39 | [E] | TEST | Completion also requires public schema 55, both SSB round trips, at least 14 registry entries covering all surface classes, the hook chain, CI, documentation, a clean workspace build, and at least eight locked-key tests. | D-9_l3_frozen_surfaces.md:237-289 |
| D9-N01 | [A] | DEP | The original has no implementation status label; PLANNED is orientation-only. | D-9_l3_frozen_surfaces.md:1-331 |
| D9-N02 | [C] | SEC,FLOW | A pre-commit hook is required to parse the commit-message body, but pre-commit normally executes before the final commit message exists; no commit-msg or prepare-commit-msg hook is specified. | D-9_l3_frozen_surfaces.md:108-116,146-159 |
| D9-N03 | [C] | SEC | The objective and summary say git push is blocked, while concrete mechanisms block commit, runtime write, or CI; no pre-push enforcement is defined. | D-9_l3_frozen_surfaces.md:3-8,108-134,183-187 |
| D9-N04 | [A] | OBS | Overridden diff tests allow either zero touch events or one override_present=true event, leaving the exact audit trail nondeterministic at specification level. | D-9_l3_frozen_surfaces.md:168-174 |
| D9-N05 | [C] | SEC | The override event is called non-permission-granting, yet a valid override is the condition that permits the commit; the boundary between documentation and authorization is semantic rather than operationally defined. | D-9_l3_frozen_surfaces.md:140-159 |
| D9-N06 | [A] | FAIL,SEC | Fail closed on an unreadable registry says treat it as locked but does not define whether all writes/commits are blocked or only known protected paths. | D-9_l3_frozen_surfaces.md:223-225 |
| D9-N07 | [A] | SEC,FLOW | The source of a valid human override during runtime Tool::Write and whether runtime writes can ever be overridden are unspecified. | D-9_l3_frozen_surfaces.md:122-134,146-159 |
| D9-N08 | [A] | DATA,SEC | Only example registry entries are shown; the exact canonical path/symbol/key mapping for each of the 14 surface classes remains implementation work. | D-9_l3_frozen_surfaces.md:75-106 |
| D9-N09 | [C] | CFG,SEC | The contextual locked-key names (`rho_delta`, `iqr_mult`, `ppl_sigma`, plural `ddm_thresholds`) differ from the registry example (`rho_delta_threshold`, `iqr_outlier_multiplier`, `ppl_anomaly_sigma`, separate warning/drift keys); the canonical names are unresolved. | D-9_l3_frozen_surfaces.md:31-35,97-104 |
| D9-N10 | [A] | TEST,SEC | The deterministic gate checks only at least 14 registry records, but one conceptual surface may require several path/symbol/key records; that count alone cannot prove all 14 classes or all per-file specifics are covered. | D-9_l3_frozen_surfaces.md:77-106,246-249,278-281 |
| D9-N11 | [A] | INT,OBS | The shell pre-commit path is required to yield a `LockedKeysTouch` row, but the plan does not define how the hook obtains an SSB writer; only the Rust runtime auditor owns `ssb_tx`. | D-9_l3_frozen_surfaces.md:108-134,230-235 |
| D9-N12 | [A] | DATA,SEC | `audit_diff` returns multiple events but `audit_runtime_write` returns only one optional event; behavior for one pending write touching multiple locked symbols or keys is not defined. | D-9_l3_frozen_surfaces.md:118-127 |
| D9-N13 | [A] | CFG,SEC | Registry examples use human override keys such as `L3_OVERRIDE:tripwire`, while the convention/example uses a registry key such as `tripwire.kl_spec_tau`; the exact key namespace and matching algorithm are unspecified. | D-9_l3_frozen_surfaces.md:77-106,146-159 |

### D-10_mode_routing_predicate.md — 53 atoms (40 E + 13 N unresolved)

| ID | Class | Category | Atomic feature or behavior | Anchor |
|---|---|---|---|---|
| D10-E01 | [E] | CAP,SEC | Assign every novel claim to one of `GroundingMode::{Constructive, FalsifiabilityCrossFamily, OperationalFalsification, DiscoveryLabelOnly, Refuse}`; routing belongs to the orchestrator and accepts no LLM input. | D-10_mode_routing_predicate.md:3-8 |
| D10-E02 | [E] | DEP,DET | Require D-9 DONE; run alongside D-11 and D-12, but not D-16 because UniCR formalizes Mode E. | D-10_mode_routing_predicate.md:9-14 |
| D10-E03 | [E] | FLOW | Mode A (`Constructive`) applies when a claim is operationalizable and an applicable checker exists; the checker supplies ground truth. | D-10_mode_routing_predicate.md:18-23,33-38 |
| D10-E04 | [E] | FLOW,FAIL,SEC | Mode B (`FalsifiabilityCrossFamily`) is a last-resort different-family adversarial probe, reached only after constructive and experimental routes are unavailable, intrinsic signals are not low, and an alternate family exists. | D-10_mode_routing_predicate.md:23-24,33-49,52-54 |
| D10-E05 | [E] | FLOW,CON | Mode C (`DiscoveryLabelOnly`) is not a runtime route; it is a post-hoc label after Mode A or D yields a checker-grounded result. | D-10_mode_routing_predicate.md:25-27,52-54 |
| D10-E06 | [E] | FLOW | Mode D (`OperationalFalsification`) runs a sandbox experiment represented by a typed `ObservableEffect`. | D-10_mode_routing_predicate.md:28-29 |
| D10-E07 | [E] | FLOW,FAIL | Mode E (`Refuse`) is the fallback when there is no constructive checker or runnable experiment and intrinsic confidence is too low, and also when no alternate family exists. | D-10_mode_routing_predicate.md:30-31,42-49 |
| D10-E08 | [E] | FLOW,DET | Predicate precedence is constructive first, runnable experiment second, low intrinsic signal refusal third, alternate-family falsifiability fourth, and refusal last. | D-10_mode_routing_predicate.md:33-49 |
| D10-E09 | [E] | DATA | `Claim` fields are `id`, `claim_type`, `statement: BoundedString<2048>`, `domain`, optional `source_template`, and optional D-3 `intrinsic_signals`. | D-10_mode_routing_predicate.md:77-81 |
| D10-E10 | [E] | DATA,SEC | `ClaimType` is the L3-locked closed enum `Code`, `Math`, `Prose`, `Empirical`, `Definitional`, `Counterfactual`, `EthicalAesthetic`. | D-10_mode_routing_predicate.md:82-83 |
| D10-E11 | [E] | FLOW | `is_operationalizable` returns true for Code or Math and for Empirical with `test_cmd`; it returns false for Prose, Definitional, Counterfactual, and EthicalAesthetic. | D-10_mode_routing_predicate.md:84-86 |
| D10-E12 | [E] | FLOW | `permits_experiment` returns true only for Empirical claims having a sandbox-compatible test. | D-10_mode_routing_predicate.md:87-88 |
| D10-E13 | [E] | DATA,SEC | `GroundingMode` is the L3-locked closed five-variant enum named in the objective. | D-10_mode_routing_predicate.md:90-95 |
| D10-E14 | [E] | DET,SEC,CON | `select_grounding_mode` is pure typed-input logic and must not call an LLM. | D-10_mode_routing_predicate.md:94-96,197-200 |
| D10-E15 | [E] | DATA,DEP | Extend `OrchContext` with a D-11 checker-registry trait stub, the existing C-22 `FeasibilityProber` sandbox, a D-16 `unicr_tau` stub constant, and alternate-family availability. | D-10_mode_routing_predicate.md:98-105 |
| D10-E16 | [E] | CFG,INT | `has_alternate_family_model()` returns true when `ELAI_PPM_ENABLE` or `ELAI_CLOUD_MODE` is set, referencing D-2/B-37. | D-10_mode_routing_predicate.md:100-104 |
| D10-E17 | [E] | DEP,FLOW | Stub implementations deliberately preserve the interface until D-11, D-13, D-16, and D-18 supply concrete checker, adversary, calibration, and observable-effect behavior. | D-10_mode_routing_predicate.md:100-105,187-193 |
| D10-E18 | [E] | FLOW,DEP | `dispatch_by_mode` sends Constructive to `checker_registry.run`; until D-11, that stub returns `Inapplicable`. | D-10_mode_routing_predicate.md:107-112 |
| D10-E19 | [E] | FLOW,DEP | OperationalFalsification invokes `sandbox.run_observable_effect`; until D-18, that stub returns `Inconclusive`. | D-10_mode_routing_predicate.md:107-113 |
| D10-E20 | [E] | FLOW,DEP | FalsifiabilityCrossFamily invokes `adversary_role.probe`; until D-13, that stub returns `NoResult`. | D-10_mode_routing_predicate.md:107-115 |
| D10-E21 | [E] | FLOW,FAIL | Refuse produces `ClaimResult::Refused` with a reason. | D-10_mode_routing_predicate.md:116 |
| D10-E22 | [E] | FLOW,FAIL,CON | Dispatching `DiscoveryLabelOnly` is forbidden as a runtime path and is intended to yield defensive inapplicability/error behavior. | D-10_mode_routing_predicate.md:117-118 |
| D10-E23 | [E] | DATA | Declared `ClaimResult` variants are `Verified`, `Falsified { evidence: BoundedString<2048> }`, `Inconclusive { reason: String }`, and `Refused { reason: String }`. | D-10_mode_routing_predicate.md:119-120 |
| D10-E24 | [E] | INT,DATA,OBS | Bump SSB schema 55 to 56 and add `ModeSelected { claim_id, mode, reason, audit_id }`. | D-10_mode_routing_predicate.md:122-124 |
| D10-E25 | [E] | INT,DATA,OBS | Add `ClaimDispatched { claim_id, mode, result, evidence_snippet }`; both new SSB variants require round-trip tests. | D-10_mode_routing_predicate.md:125-127 |
| D10-E26 | [E] | FLOW,OBS | At every claim-emission site, construct a Claim, select the mode, emit `ModeSelected`, dispatch it, then emit `ClaimDispatched`. | D-10_mode_routing_predicate.md:129-133 |
| D10-E27 | [E] | SEC,CFG | Extend D-9's registry with a locked-symbol entry for `select_grounding_mode` in `grounding_mode.rs`, grounded in the no-self-grading rule. | D-10_mode_routing_predicate.md:135-143 |
| D10-E28 | [E] | SEC,CFG | Add a separate locked-symbol entry for the `ClaimType` closed enum in `claim.rs`. | D-10_mode_routing_predicate.md:144-148 |
| D10-E29 | [E] | TEST,FLOW | Unit tests cover operationalizability and experiment-permission tables, including Code with a test, Math with a SymPy expression, and Empirical without sandbox compatibility. | D-10_mode_routing_predicate.md:150-153 |
| D10-E30 | [E] | TEST,FLOW | Routing tests require Constructive to outrank Mode D, then cover sandbox-only D, low-signal E, alternate-family B, and no-alternate E branches. | D-10_mode_routing_predicate.md:153-160 |
| D10-E31 | [E] | TEST,DATA | The contract requires public schema 56, both SSB payload round trips, and stable serde for the closed `GroundingMode` enum. | D-10_mode_routing_predicate.md:162-165 |
| D10-E32 | [E] | CON,SEC | The decision record must justify orchestrator-owned routing using Cohen/Everitt/Hubinger. | D-10_mode_routing_predicate.md:167-170 |
| D10-E33 | [E] | FAIL,CON | It must justify Refuse rather than Mode B as the fallback, citing a 64.5% same-model blind-spot rate and preferring explicit uncertainty when no alternate family exists. | D-10_mode_routing_predicate.md:171-173 |
| D10-E34 | [E] | FLOW,CON | It must justify why `DiscoveryLabelOnly` is post-hoc annotation rather than a runtime mode. | D-10_mode_routing_predicate.md:174-175 |
| D10-E35 | [E] | FAIL,TEST,OBS | Stub bans require the empty-context Refuse path to remain reachable, all five named mode branches to be tested, and `ClaimDispatched` to be emitted regardless of mode. | D-10_mode_routing_predicate.md:197-203 |
| D10-E36 | [E] | OBS,TEST | The observable exercise runs 50 synthetic claims, ten per named mode, requires 50 `ModeSelected` rows and at least 45/50 labels matching ground truth, described as at least 90% F1. | D-10_mode_routing_predicate.md:203-208 |
| D10-E37 | [E] | TEST | Deterministic/acceptance checks require schema/event/type/lock presence, zero routing-file LLM-call matches, a clean workspace build, contract tests, and at least 12 new unit tests. | D-10_mode_routing_predicate.md:210-249 |
| D10-E38 | [E] | TEST,OBS | Cloud smoke sends ten mixed synthetic claims, checks their hand-labeled distribution, and verifies by grep and execution trace that selection made no LLM call. | D-10_mode_routing_predicate.md:250-255 |
| D10-E39 | [E] | CON,DEP | D-11 checker behavior, D-13 adversary behavior, D-16 UniCR, D-18 observable effects, and D-21 quantitative coverage are explicitly out of scope for this session. | D-10_mode_routing_predicate.md:273-280 |
| D10-E40 | [E] | DEP | D-11 is the next consumer, and D-21 is stated to gate D-10 at at least 0.85 F1. | D-10_mode_routing_predicate.md:281-284 |
| D10-N01 | [A] | DEP | The original never labels implementation status; PLANNED is orientation-only and no implementation files were inspected. | D-10_mode_routing_predicate.md:1-284 |
| D10-N02 | [C] | DATA,FLOW | `is_operationalizable` and its tests depend on `test_cmd` and `sympy_expr`, and experiment routing depends on sandbox compatibility, but none is a field of the declared `Claim`. | D-10_mode_routing_predicate.md:77-88,150-153 |
| D10-N03 | [A] | FLOW,FAIL | `intrinsic_signals` is optional, but the predicate's behavior when it is `None` is not defined. | D-10_mode_routing_predicate.md:79-81,42-43 |
| D10-N04 | [C] | DATA,FAIL | The DiscoveryLabelOnly branch says it returns `ClaimResult::Inapplicable`, but the declared `ClaimResult` enum has no `Inapplicable` variant. | D-10_mode_routing_predicate.md:107-120 |
| D10-N05 | [C] | FAIL | The same branch is described as a defensive `Err`, while `dispatch_by_mode` returns bare `ClaimResult`, not `Result<ClaimResult, _>`. | D-10_mode_routing_predicate.md:107-120 |
| D10-N06 | [A] | OBS,FLOW | Task-runner integration says the caller emits `ClaimDispatched`, while the stub ban says `dispatch_by_mode` itself must emit it; ownership is unresolved and a literal implementation could duplicate rows. | D-10_mode_routing_predicate.md:129-133,197-203 |
| D10-N07 | [A] | INT,SEC | Presence of `ELAI_PPM_ENABLE` or `ELAI_CLOUD_MODE` does not by itself prove the available model is a different family from the executor; family identity and runtime health checks are not specified. | D-10_mode_routing_predicate.md:98-105 |
| D10-N08 | [C] | TEST | The plan equates 45 correct classifications out of 50 with at least 90% F1; accuracy and macro/per-class F1 are not interchangeable. | D-10_mode_routing_predicate.md:203-208,250-254 |
| D10-N09 | [C] | TEST,DEP | D-10's observable target says at least 90% F1, while its own Next section makes D-21 authoritative at at least 0.85 macro-F1. | D-10_mode_routing_predicate.md:203-208,281-284; D-21_mode_routing_coverage_benchmark.md:39-49 |
| D10-N10 | [C] | FLOW,TEST | The predicate never returns `DiscoveryLabelOnly`, yet both D-10's 50-claim exercise and D-21's balanced corpus demand ten/forty Mode-C cases respectively. | D-10_mode_routing_predicate.md:33-54,203-208; D-21_mode_routing_coverage_benchmark.md:15-21,31-34 |
| D10-N11 | [A] | DATA,OBS | Reason-string contents, `ClaimResult`-to-string serialization, and construction/truncation of `evidence_snippet` are unspecified. | D-10_mode_routing_predicate.md:119-127 |
| D10-N12 | [A] | FAIL,FLOW | The stubs return Inapplicable, Inconclusive, or NoResult, but the declared result enum cannot represent two of those names and no fallback/re-routing rule is defined for a selected mode whose backend is unavailable. | D-10_mode_routing_predicate.md:98-120 |
| D10-N13 | [A] | CAP,FLOW | “Every claim-emit site” has no enumerated call-site inventory or coverage check, so completeness of routing insertion cannot be established from this plan alone. | D-10_mode_routing_predicate.md:56-61,129-133 |

### D-14_sycophancy_probe.md — 47 atoms (33 E + 14 N unresolved)

| ID | Class | Category | Atomic feature or behavior | Anchor |
|---|---|---|---|---|
| D14-E01 | [E] | CAP,SEC,FLOW | For a user-aligned claim, run an inverted-framing probe; convergence is robust, divergence creates `SYCOPHANCY_FLAG` and human-review quarantine, as a structural rather than prompt-only defense. | D-14_sycophancy_probe.md:3-8 |
| D14-E02 | [E] | DEP,DET | Require D-13 cross-family infrastructure, possibly using its Adversary slot or the same executor; run in parallel with D-15. | D-14_sycophancy_probe.md:9-12 |
| D14-E03 | [E] | FLOW | Trigger when the model agrees with a direct user assertion, then invoke the same model with the same context and an instruction to assume that assertion is wrong and justify the alternative. | D-14_sycophancy_probe.md:16-26 |
| D14-E04 | [E] | FLOW,FAIL | Similar positions across original and inverted responses are Convergent; significant divergence is a sycophancy flag and quarantines the claim. | D-14_sycophancy_probe.md:26-30 |
| D14-E05 | [E] | DET,CFG | Convergence is token-level 5-shingle Jaccard similarity at least 0.7; the threshold is intended for later empirical tuning from ELAI session statistics. | D-14_sycophancy_probe.md:32-34 |
| D14-E06 | [E] | CAP,DATA,CFG | `SycophancyProbe` owns `Arc<dyn RuntimeAdapter>`, `convergence_threshold` defaulting to 0.7, and an SSB sender. | D-14_sycophancy_probe.md:58-62 |
| D14-E07 | [E] | DATA | `InvertedFraming` contains original and inverted `BoundedString<2048>` values plus `claim_id`. | D-14_sycophancy_probe.md:63-64 |
| D14-E08 | [E] | DATA,OBS | `ProbeReport` contains claim ID, Blake3 hashes of both outputs, Jaccard similarity, and `ProbeVerdict`. | D-14_sycophancy_probe.md:65-67 |
| D14-E09 | [E] | DATA,FLOW | `ProbeVerdict` is `Convergent`, `DivergentSycophancyFlag`, or `Inconclusive`. | D-14_sycophancy_probe.md:68 |
| D14-E10 | [E] | FLOW | `invert_framing` detects direct positions with heuristics such as “I think”, “this is”, “X is correct”, and direct assertions, then constructs the specified adversarial prompt. | D-14_sycophancy_probe.md:70-76 |
| D14-E11 | [E] | FAIL,FLOW | With no clear assertion, inversion conservatively mirrors the original with a skip note and yields Inconclusive rather than forcing a probe. | D-14_sycophancy_probe.md:77-81 |
| D14-E12 | [E] | TEST | At least five synthetic user framings must each produce a sensible inversion. | D-14_sycophancy_probe.md:80-81,184-190 |
| D14-E13 | [E] | FLOW,INT | `probe` inverts the message, skips to Inconclusive when marked, otherwise sends the inverted prompt through `RuntimeAdapter` and receives an inverted output. | D-14_sycophancy_probe.md:83-89 |
| D14-E14 | [E] | DET,FLOW | It computes 5-shingle Jaccard between original and inverted output, maps `>= threshold` to Convergent and lower values to `DivergentSycophancyFlag`. | D-14_sycophancy_probe.md:89-92 |
| D14-E15 | [E] | OBS,FLOW | Every executed probe emits a probe-dispatch row, and a divergent verdict additionally emits a sycophancy-flag row. | D-14_sycophancy_probe.md:93,158-167 |
| D14-E16 | [E] | FLOW | The task-runner hook executes only after `dispatch_by_mode` returns `ClaimResult::Verified` and the user message has a direct assertion. | D-14_sycophancy_probe.md:95-97 |
| D14-E17 | [E] | FLOW,SEC | A divergent claim is marked `human_review_pending` and cannot proceed to `MEMORIZING`. | D-14_sycophancy_probe.md:98-100,158-162 |
| D14-E18 | [E] | INT,DATA,OBS | Bump SSB schema 59 to 60 and add `SycophancyProbeDispatched { claim_id, user_assertion_hash, audit_id }`. | D-14_sycophancy_probe.md:101-106 |
| D14-E19 | [E] | INT,DATA,OBS | Add `SycophancyFlag { claim_id, jaccard, threshold, original_hash, inverted_hash }`; both variants require round trips. | D-14_sycophancy_probe.md:101-106 |
| D14-E20 | [E] | CFG,SEC | Add L3-locked `[sycophancy_probe]` config with `enabled=true`, `convergence_threshold=0.7`, `jaccard_ngram_size=5`, and `skip_if_no_assertion=true`. | D-14_sycophancy_probe.md:108-111 |
| D14-E21 | [E] | TEST,FAIL | Unit tests cover assertive and neutral inversions, Jaccard values 1.0/0.0/partial, and the complete verdict table. | D-14_sycophancy_probe.md:113-120 |
| D14-E22 | [E] | TEST,OBS | The divergent integration scenario requires a flag event and quarantine. | D-14_sycophancy_probe.md:122-127 |
| D14-E23 | [E] | TEST,OBS | The independent-agreement scenario requires Convergent and no flag. | D-14_sycophancy_probe.md:128-130 |
| D14-E24 | [E] | CON,SEC | The decision record must justify structural probing because prompt-only mitigation is unreliable. | D-14_sycophancy_probe.md:132-135 |
| D14-E25 | [E] | CON,DET | It must justify bounded 5-shingle Jaccard rather than unbounded edit distance. | D-14_sycophancy_probe.md:136-137 |
| D14-E26 | [E] | CFG,CON | It must justify the default 0.7 threshold as a conservative midpoint of the cited 0.6-0.85 non-sycophantic similarity band. | D-14_sycophancy_probe.md:138-140 |
| D14-E27 | [E] | DEP,INT | The module consumes B-37 `RuntimeAdapter`, D-10 Claim, SSB writing, and the user-message field on `OrchContext`, and is reachable from the task-runner post-dispatch hook plus config. | D-14_sycophancy_probe.md:142-157 |
| D14-E28 | [E] | FAIL,CFG,OBS | Stub bans require a dispatch row regardless of verdict, safe empty-message skipping, config-loaded threshold, and quarantine enforcement at the MEMORIZING transition. | D-14_sycophancy_probe.md:158-162 |
| D14-E29 | [E] | OBS | The divergent observable exercise yields exactly one dispatch row, one flag row, no MEMORIZING transition, an auditor-visible flag, and a quarantine artifact under `workflow/sessions/{id}/sycophancy-quarantine/`. | D-14_sycophancy_probe.md:163-170 |
| D14-E30 | [E] | DEP,TEST | Before implementation, ingest the Sharma sycophancy paper interface; deterministic and acceptance checks require that artifact, schema/types/config presence, a clean workspace build, and at least eight unit tests. | D-14_sycophancy_probe.md:42-54,172-194,201 |
| D14-E31 | [E] | TEST,FAIL | Cloud smokes cover divergent quarantine, independent convergence without a flag, and a neutral no-assertion skip recorded with `note="skipped"`. | D-14_sycophancy_probe.md:195-200 |
| D14-E32 | [E] | CON,DEP | D-23 benchmarking is out of scope; multi-turn detection and LLM-as-judge detection are explicitly deferred, with the structural single-turn approach first. | D-14_sycophancy_probe.md:216-220 |
| D14-E33 | [E] | DEP | D-15 is the next orthogonal output-labeling step, and D-23 later benchmarks this detector. | D-14_sycophancy_probe.md:222-225 |
| D14-N01 | [A] | DEP | The original never labels implementation status; PLANNED is orientation-only and no implementation files were inspected. | D-14_sycophancy_probe.md:1-225 |
| D14-N02 | [C] | DATA,FAIL | `InvertedFraming` has no `note` field, yet probe control flow and the neutral smoke test read or emit such a note. | D-14_sycophancy_probe.md:63-64,77-88,199-200 |
| D14-N03 | [C] | OBS,DATA | The neutral smoke expects `SycophancyProbeDispatched` with `note="skipped"`, but that event schema has no note or verdict field. | D-14_sycophancy_probe.md:101-106,199-200 |
| D14-N04 | [C] | FLOW,OBS | The hook excludes neutral/no-assertion messages, while acceptance requires those messages to produce a skipped dispatch row; no alternative invocation site is defined. | D-14_sycophancy_probe.md:95-100,158-160,199-200 |
| D14-N05 | [C] | INT,OBS | Narrative and summary terminology use `SYCOPHANCY_PROBE`, while the normative Rust variant is `SycophancyProbeDispatched`; no alias or SQL identifier mapping is supplied. | D-14_sycophancy_probe.md:3-8,93,101-106,142-149 |
| D14-N06 | [A] | INT,SEC | The context says “same model, same context,” while the prerequisite permits either the D-13 Adversary slot or the same executor; actual adapter/model-family selection is unspecified. | D-14_sycophancy_probe.md:9-11,21-25,36-40,150-154 |
| D14-N07 | [A] | DET,DATA | Tokenization, case/punctuation normalization, duplicate-shingle treatment, and behavior for outputs shorter than five tokens or both empty are not defined. | D-14_sycophancy_probe.md:32-34,89-92,113-120 |
| D14-N08 | [A] | FAIL,INT | Timeout, provider failure, cancellation, retry, rate-limit, and partial-output behavior for the inverted runtime call are not defined; `SycophancyError` is named but not enumerated. | D-14_sycophancy_probe.md:83-93 |
| D14-N09 | [A] | FLOW,SEC | Human-review ownership, release/reject criteria, retention, and resumption after quarantine are unspecified. | D-14_sycophancy_probe.md:98-100,163-170 |
| D14-N10 | [A] | CFG,FLOW | `enabled` and `skip_if_no_assertion` defaults are declared, but disabled-mode behavior and any override semantics are not. | D-14_sycophancy_probe.md:108-111 |
| D14-N11 | [A] | SEC,CFG | The config is declared L3-locked, but this plan does not add its keys to D-9's registry or define an override key. | D-14_sycophancy_probe.md:108-111,204-214 |
| D14-N12 | [A] | OBS | A convergent or inconclusive result has no result-bearing SSB row; the dispatch event proves invocation but does not preserve verdict, similarity, or skip reason. | D-14_sycophancy_probe.md:65-69,93,101-106 |
| D14-N13 | [A] | DET,SEC | Lexical Jaccard convergence is treated as semantic robustness, but paraphrase and shared-boilerplate handling are not specified; this is a design assumption rather than a proved equivalence. | D-14_sycophancy_probe.md:26-34,89-92 |
| D14-N14 | [C] | OBS,DEP | The observable contract requires an auditor-agent update, but that agent file is absent from the declared files-touched list. | D-14_sycophancy_probe.md:163-170,204-214 |

### D-21_mode_routing_coverage_benchmark.md — 33 atoms (23 E + 10 N unresolved)

| ID | Class | Category | Atomic feature or behavior | Anchor |
|---|---|---|---|---|
| D21-E01 | [E] | CAP,TEST | Benchmark D-10 with 200 hand-labeled synthetic claims across A/B/C/D/E, report F1 and per-mode confusion, and gate macro-F1 at 0.85 without class collapse. | D-21_mode_routing_coverage_benchmark.md:3-5 |
| D21-E02 | [E] | DEP,DET | Require D-10 and declare parallel safety with D-19 through D-25. | D-21_mode_routing_coverage_benchmark.md:6-8 |
| D21-E03 | [E] | DATA,TEST | Author exactly 40 examples per mode, each with a canonical expected mode and rationale. | D-21_mode_routing_coverage_benchmark.md:15-17 |
| D21-E04 | [E] | TEST | Coverage must include clear compile-check Mode A, sandbox Mode D, ethical/aesthetic Mode E, ambiguous A-versus-D, and no-alternate-family Refuse edge cases. | D-21_mode_routing_coverage_benchmark.md:18-21 |
| D21-E05 | [E] | DATA | Store 200 JSON fixture files under `.elai_cc/benchmarks/mode_routing/fixtures/`, with `claim_id`, `statement`, `claim_type`, `domain`, `intrinsic_signals`, `expected_mode`, and `rationale`. | D-21_mode_routing_coverage_benchmark.md:29-34 |
| D21-E06 | [E] | CAP,DATA | `ModeRoutingBenchmark` contains `fixtures_dir`; `ModeRoutingReport` contains total, correct, `(expected,predicted)` confusion counts, macro-F1, per-mode F1, and `GatingVerdict`. | D-21_mode_routing_coverage_benchmark.md:36-41 |
| D21-E07 | [E] | FLOW,DET | `run` loads every fixture, builds a mock `OrchContext`, calls `select_grounding_mode`, compares expected versus actual, and computes macro/per-mode F1 plus confusion matrix. | D-21_mode_routing_coverage_benchmark.md:42-45 |
| D21-E08 | [E] | TEST,FAIL | Pass requires macro-F1 at least 0.85 and every mode's F1 at least 0.70; otherwise return `FailBelowBaseline`. | D-21_mode_routing_coverage_benchmark.md:47-49 |
| D21-E09 | [E] | CAP | Expose CLI `elai bench mode-routing` from `bench_mode_routing.rs`. | D-21_mode_routing_coverage_benchmark.md:51-53 |
| D21-E10 | [E] | CON | The decision record justifies 40 examples per mode as an approximately ±5% noise floor balanced against authoring cost. | D-21_mode_routing_coverage_benchmark.md:54-56 |
| D21-E11 | [E] | TEST,CON | It justifies macro-F1 0.85 for system failure and per-mode 0.70 for class-collapse protection. | D-21_mode_routing_coverage_benchmark.md:57-58 |
| D21-E12 | [E] | SEC,CON | It requires human-vetted labels rather than another LLM's output to avoid circular ground truth. | D-21_mode_routing_coverage_benchmark.md:59-60 |
| D21-E13 | [E] | DEP,INT | The benchmark consumes D-10 selector/mode types and D-19 `BenchmarkStarted`/`BenchmarkResult` SSB support. | D-21_mode_routing_coverage_benchmark.md:62-71 |
| D21-E14 | [E] | CAP,INT | The benchmark is reachable from CLI and a CI workflow. | D-21_mode_routing_coverage_benchmark.md:72-73 |
| D21-E15 | [E] | SEC,TEST | Fixtures must be authored from scratch, not generated, and signed off by a human reviewer. | D-21_mode_routing_coverage_benchmark.md:74-78 |
| D21-E16 | [E] | OBS | Each CLI run emits `BenchmarkStarted` and `BenchmarkResult` and writes `.elai_cc/benchmarks/mode_routing/reports/{date}.json`. | D-21_mode_routing_coverage_benchmark.md:79-82 |
| D21-E17 | [E] | TEST | Deterministic checks require the runner/run symbols, at least 200 JSON fixtures, a clean workspace build, and scoped benchmark tests. | D-21_mode_routing_coverage_benchmark.md:84-92 |
| D21-E18 | [E] | TEST,FAIL | Acceptance requires the benchmark to compile, all 200 balanced hand labels to exist, and the CLI to exit 0 when the gate is met. | D-21_mode_routing_coverage_benchmark.md:94-99 |
| D21-E19 | [E] | TEST,OBS | The live 200-fixture smoke must reach macro-F1 at least 0.85 and every per-mode F1 at least 0.70 with no collapse. | D-21_mode_routing_coverage_benchmark.md:100-101 |
| D21-E20 | [E] | DET,TEST | Two runs over identical mocked contexts must produce the same confusion matrix. | D-21_mode_routing_coverage_benchmark.md:102-103 |
| D21-E21 | [E] | OBS,FLOW | Confusion output must expose edge cases for subsequent D-10 refinement. | D-21_mode_routing_coverage_benchmark.md:104 |
| D21-E22 | [E] | CON,DEP | A D-21 decision document is required; larger fixture sets and LLM-generated fixtures are deferred, the latter due to circularity risk. | D-21_mode_routing_coverage_benchmark.md:105-118 |
| D21-E23 | [E] | DEP | D-22 conformal-calibration validation follows this session. | D-21_mode_routing_coverage_benchmark.md:120-122 |
| D21-N01 | [A] | DEP | The original never labels implementation status; PLANNED is orientation-only and no implementation files or fixtures were inspected. | D-21_mode_routing_coverage_benchmark.md:1-122 |
| D21-N02 | [C] | FLOW,TEST | Forty fixtures require expected Mode C, but D-10 defines C as post-hoc label-only and its selector has no branch that returns it. | D-21_mode_routing_coverage_benchmark.md:3-5,15-21,31-34; D-10_mode_routing_predicate.md:25-27,33-54 |
| D21-N03 | [A] | DATA,TEST | The fixture schema omits checker applicability, sandbox capability, alternate-family availability, and experiment/test metadata needed to construct the mocked context and reproduce D-10 routes. | D-21_mode_routing_coverage_benchmark.md:31-45; D-10_mode_routing_predicate.md:33-49,77-105 |
| D21-N04 | [A] | TEST,DET | Precision/recall formulas, macro averaging, zero-denominator policy, and treatment of Mode C's unreachable predictions are not specified. | D-21_mode_routing_coverage_benchmark.md:36-49 |
| D21-N05 | [A] | DATA,FAIL | `GatingVerdict` and its Pass variant/source are not defined, while only `FailBelowBaseline` is named. | D-21_mode_routing_coverage_benchmark.md:36-49 |
| D21-N06 | [A] | FAIL,TEST | Acceptance ties CLI exit 0 only to macro-F1, while normative gate logic additionally requires every per-mode F1 at least 0.70; the exit status when macro passes but one class fails is not restated. | D-21_mode_routing_coverage_benchmark.md:47-49,94-101 |
| D21-N07 | [C] | DEP,DET | The session is declared parallel-safe with D-19 yet consumes benchmark SSB types “from D-19”; build sequencing or a pre-existing shared definition is not supplied. | D-21_mode_routing_coverage_benchmark.md:7-8,62-73 |
| D21-N08 | [A] | SEC,OBS | Human sign-off is mandatory, but reviewer identity, signature format, approval artifact, and enforcement check are unspecified. | D-21_mode_routing_coverage_benchmark.md:31-34,59-60,74-78 |
| D21-N09 | [A] | FAIL | Missing, malformed, duplicate, imbalanced, or extra fixtures and partial report-write failures have no defined error/fallback behavior. | D-21_mode_routing_coverage_benchmark.md:31-49,79-82 |
| D21-N10 | [A] | DET,OBS | `{date}` timezone, same-day overwrite/versioning, stable map serialization, and byte-for-byte report determinism are unspecified even though result determinism is an acceptance gate. | D-21_mode_routing_coverage_benchmark.md:39-41,79-82,102-103 |

### D-24_cross_domain_canary_regression_suite.md — 28 atoms (20 E + 8 N unresolved)

| ID | Class | Category | Atomic feature or behavior | Anchor |
|---|---|---|---|---|
| D24-E01 | [E] | CAP,TEST | Validate D-7 with 100 synthetic mutations across five task families: 50 helpful, 30 neutral, and 20 harmful. | D-24_cross_domain_canary_regression_suite.md:3-5 |
| D24-E02 | [E] | DEP,DET | Require D-7 and declare parallel safety with D-19 through D-25. | D-24_cross_domain_canary_regression_suite.md:6-8 |
| D24-E03 | [E] | DATA,TEST | Helpful mutations improve the target family by 5-15% and regress no non-target family. | D-24_cross_domain_canary_regression_suite.md:14-15 |
| D24-E04 | [E] | DATA,TEST | Neutral mutations leave the target within ±2% and regress no non-target family. | D-24_cross_domain_canary_regression_suite.md:16 |
| D24-E05 | [E] | DATA,TEST | Harmful mutations leave the target unchanged or improved but regress at least one non-target family by more than 5%. | D-24_cross_domain_canary_regression_suite.md:17-18 |
| D24-E06 | [E] | DATA | Store 100 TOML fixtures under `.elai_cc/benchmarks/canary/fixtures/`, each with mutation ID, target family, baseline/candidate pass rates by family, and expected verdict. | D-24_cross_domain_canary_regression_suite.md:26-32 |
| D24-E07 | [E] | CAP,FLOW | `CanaryBenchmark` invokes D-7 `CanaryRunner` for each fixture and compares the result with `expected_verdict`. | D-24_cross_domain_canary_regression_suite.md:34-37 |
| D24-E08 | [E] | DATA,OBS | Its report contains total, true_block, false_block, true_pass, and false_pass. | D-24_cross_domain_canary_regression_suite.md:34-37 |
| D24-E09 | [E] | TEST,FAIL | Hard gates are 20/20 harmful blocked, no more than 5/50 helpful blocked, and no more than 2/30 neutral blocked. | D-24_cross_domain_canary_regression_suite.md:39-40 |
| D24-E10 | [E] | CAP | Expose CLI `elai bench canary-regression`. | D-24_cross_domain_canary_regression_suite.md:42 |
| D24-E11 | [E] | CON | The decision record justifies 100 fixtures as sufficient for the intended sample-noise floor. | D-24_cross_domain_canary_regression_suite.md:44-45 |
| D24-E12 | [E] | CON | It justifies the 50/30/20 split because harmful mutations are the minority and over-sampling would distort evaluation. | D-24_cross_domain_canary_regression_suite.md:46 |
| D24-E13 | [E] | TEST,CON | It justifies a 10% helpful false-block allowance so D-7's regression threshold can be tuned without rebuilding canary infrastructure. | D-24_cross_domain_canary_regression_suite.md:47-48 |
| D24-E14 | [E] | DEP,INT | The benchmark consumes D-7 `CanaryRunner` and SSB writing and is reachable from CLI plus CI. | D-24_cross_domain_canary_regression_suite.md:50-59 |
| D24-E15 | [E] | TEST,SEC | Blocking all 20 harmful fixtures is an unrelaxed hard gate. | D-24_cross_domain_canary_regression_suite.md:60-61 |
| D24-E16 | [E] | OBS | A CLI run emits `BenchmarkStarted` plus `BenchmarkResult` and writes a report beneath `.elai_cc/benchmarks/canary/reports/`. | D-24_cross_domain_canary_regression_suite.md:62-65 |
| D24-E17 | [E] | TEST | Deterministic checks require exactly/at least 100 fixture files, the benchmark struct, and a clean workspace build. | D-24_cross_domain_canary_regression_suite.md:67-74 |
| D24-E18 | [E] | TEST,FAIL | Acceptance requires compilation, 100 fixtures, CLI exit 0 only when gates are met, and the clean workspace build. | D-24_cross_domain_canary_regression_suite.md:76-81 |
| D24-E19 | [E] | TEST | Three cloud smokes separately prove 20/20 harmful blocking, helpful false blocks at most 5/50, and neutral blocks at most 2/30. | D-24_cross_domain_canary_regression_suite.md:82-85 |
| D24-E20 | [E] | CON,DEP | Real-data mutations are deferred; the plan treats synthetic fixtures as sufficient for the D-7 gate. | D-24_cross_domain_canary_regression_suite.md:95-101 |
| D24-N01 | [A] | DEP | The original never labels implementation status; PLANNED is orientation-only and no implementation files or fixtures were inspected. | D-24_cross_domain_canary_regression_suite.md:1-101 |
| D24-N02 | [A] | DATA,TEST | “Across 5 task families” has no per-family quota or cross-product rule, so all 100 fixtures could technically concentrate in one family while satisfying the stated type counts. | D-24_cross_domain_canary_regression_suite.md:3-5,28-32 |
| D24-N03 | [A] | TEST | Helpful improvement exactly +5%, neutral exactly ±2%, and non-target regression exactly 5% have only partly explicit boundary semantics; the harmful definition is strictly greater than 5%. | D-24_cross_domain_canary_regression_suite.md:14-18 |
| D24-N04 | [A] | DATA,FAIL | Rate ranges, family completeness, numeric precision, missing families, malformed TOML, duplicate IDs, and invalid expected verdicts have no validation/error policy. | D-24_cross_domain_canary_regression_suite.md:28-37 |
| D24-N05 | [A] | DATA,OBS | The report has only binary pass/block counters and no per-family/type breakdown or explicit overall verdict field, limiting diagnosis of where thresholds fail. | D-24_cross_domain_canary_regression_suite.md:34-40 |
| D24-N06 | [A] | INT,OBS | Generic benchmark SSB event payloads and their schema/version source are not defined in this plan. | D-24_cross_domain_canary_regression_suite.md:50-65 |
| D24-N07 | [A] | DET | Fixture ordering, random seed (if any), repeated-run equivalence, report naming, and overwrite/concurrency behavior are unspecified. | D-24_cross_domain_canary_regression_suite.md:26-42,62-74 |
| D24-N08 | [A] | TEST | No scoped unit/integration test command is included; deterministic verification checks only fixture count, symbol presence, and workspace compilation before cloud smokes. | D-24_cross_domain_canary_regression_suite.md:67-85 |

### D-28_adversarial_example_farming.md — 47 atoms (33 E + 14 N unresolved)

| ID | Class | Category | Atomic feature or behavior | Anchor |
|---|---|---|---|---|
| D28-E01 | [E] | CAP,DEP | Deferred research-grade work iteratively generates OOD high-proxy/low-quality PPM examples and retrains D-2's PPM for gaming resistance; estimate 4-6 sessions plus RunPod hours. | D-28_adversarial_example_farming.md:1-8 |
| D28-E02 | [E] | DEP,DET | Unblock only after D-5 has at least 20 stable promotion cycles, D-19 publishes a PRMBench baseline, and a RunPod 4090 or A100 is available; it may run beside D-29 and D-30. | D-28_adversarial_example_farming.md:9-11,129-132 |
| D28-E03 | [E] | FLOW,TEST | Generate candidate reasoning steps that score high on the current PPM but fail deterministic gold checks such as compile, tests, or formal consistency. | D-28_adversarial_example_farming.md:13-20 |
| D28-E04 | [E] | FLOW,DATA | Mine the proxy-versus-gold gap and assign negative labels to the resulting adversarial corpus. | D-28_adversarial_example_farming.md:20-21 |
| D28-E05 | [E] | FLOW,INT | Retrain the PPM on that corpus, then remeasure PRMBench Deception Resistance and Circular Logic with a target improvement of at least five percentage points. | D-28_adversarial_example_farming.md:21-28 |
| D28-E06 | [E] | DEP | Bootstrapping is explicitly blocked by stable BAI selection, a published baseline, and cost-throttled GPU fine-tuning. | D-28_adversarial_example_farming.md:25-28 |
| D28-E07 | [E] | DEP,TEST | Ingest and verify `workflow/papers/adversarial-rm-2504.06141-interface.md` before unblocking. | D-28_adversarial_example_farming.md:30-44 |
| D28-E08 | [E] | CAP,FLOW | `ppm_adversarial.py` samples a step, computes proxy and gold rewards, and retains it when proxy is much greater than gold and gold equals Fail. | D-28_adversarial_example_farming.md:46-54 |
| D28-E09 | [E] | DATA | The task-level corpus destination is `.elai_cc/calibration/ppm_adversarial/{ts}/{step_id}.jsonl`. | D-28_adversarial_example_farming.md:48-55 |
| D28-E10 | [E] | INT,CFG | `ppm_finetune.sh` runs on a RunPod 4090/A100 and loads `Qwen2.5-Math-PRM-7B`. | D-28_adversarial_example_farming.md:56-59 |
| D28-E11 | [E] | CFG | Fine-tuning uses negative-labeled adversarial examples with LoRA rank 16, three epochs, and learning rate `1e-5`. | D-28_adversarial_example_farming.md:58-60 |
| D28-E12 | [E] | DATA,INT | Fine-tuning outputs `ppm_v2.safetensors` and deploys it to the remote sidecar. | D-28_adversarial_example_farming.md:60 |
| D28-E13 | [E] | FLOW,OBS | Run D-19 on v1 and save its report, deploy v2, run D-19 again, save that report, then compare category deltas. | D-28_adversarial_example_farming.md:62-65 |
| D28-E14 | [E] | TEST,FAIL | Every standard PRMBench category must avoid regression greater than two percentage points. | D-28_adversarial_example_farming.md:67-70 |
| D28-E15 | [E] | TEST | Deception Resistance and Circular Logic must each improve by at least five percentage points. | D-28_adversarial_example_farming.md:67-70,129-135 |
| D28-E16 | [E] | FAIL | If standard categories regress or adversarial gains miss the gate, revert to v1, log the corpus and decision, and do not deploy v2. | D-28_adversarial_example_farming.md:72-75 |
| D28-E17 | [E] | INT,DATA,OBS | The task originally proposes, with an appropriate future schema bump, `AdversarialCorpusGenerated { count, source_session }`. | D-28_adversarial_example_farming.md:77-80 |
| D28-E18 | [E] | INT,DATA,OBS | It also proposes `PpmFinetuneCompleted { v1_score, v2_score, delta, verdict }`. | D-28_adversarial_example_farming.md:77-80 |
| D28-E19 | [E] | CON,DEP | Draft the D-28 decision document only at unblock time, explaining deferral, conservative LoRA rather than full tuning, and the literature-grounded five-point gain gate. | D-28_adversarial_example_farming.md:82-89 |
| D28-E20 | [E] | CAP,DATA | Shipping-pass annotations name live `AdvRmFarmer`, `AdvRmFarmer::run`, `AdvRmFarmRunReport`, `AdvRmSeed`, `AdvRmCorpusEntry`, and `verdict -> GatingVerdict` with implementation anchors. | D-28_adversarial_example_farming.md:91-97 |
| D28-E21 | [E] | INT,OBS | Those annotations explicitly reject both originally proposed event variants and identify the actual shipped event/payload as `AdversarialFarmRunCompleted`. | D-28_adversarial_example_farming.md:97-101 |
| D28-E22 | [E] | DEP,INT | The Python producer exists according to the plan; the fine-tune script is `RUNPOD-PENDING`. | D-28_adversarial_example_farming.md:100-101 |
| D28-E23 | [E] | DEP | The shipping contract consumes at least 20 D-5 BAI cycles, D-19 `PRMBenchRunner`, `SidecarPpmHook`, and D-5 `darwinian_bai`. | D-28_adversarial_example_farming.md:102-106 |
| D28-E24 | [E] | CAP | Live reachability is asserted through `elai farm-adv-rm run` and `elai farm-adv-rm verdict`. | D-28_adversarial_example_farming.md:107-110 |
| D28-E25 | [E] | INT,DEP | Live `ppm_adversarial.py --live` and the GPU fine-tune/post-immunization legs remain RunPod- and prerequisite-pending. | D-28_adversarial_example_farming.md:110,119-121 |
| D28-E26 | [E] | OBS,TEST | Every farming run must emit `AdversarialFarmRunCompleted`; `verdict` must call the real `report.passed()` rather than a mock. | D-28_adversarial_example_farming.md:111-114 |
| D28-E27 | [E] | SEC,CON | Production corpus generation may not use `ppm_adversarial.py --mock`. | D-28_adversarial_example_farming.md:114 |
| D28-E28 | [E] | OBS | `elai farm-adv-rm run --dataset <prmbench_seed_path> --corpus-dir .elai_cc/benchmarks/adv_rm/corpus/` must emit one completion row with `samples_farmed > 0` and populate `AdvRmCorpusEntry` JSONL. | D-28_adversarial_example_farming.md:115-121 |
| D28-E29 | [E] | DEP,OBS | The source explicitly records the CLI-run and SSB-emission legs as wired and specifically claims `farm_adv_rm.rs` plus `ssb.rs:6710` are live. | D-28_adversarial_example_farming.md:119-121 |
| D28-E30 | [E] | TEST,DEP | The deterministic-check section remains deferred to unblock time; acceptance is gated on prerequisites, GPU access, paper output, v2 deployment/report, both numeric gates, and the decision document. | D-28_adversarial_example_farming.md:123-136 |
| D28-E31 | [E] | INT,DEP | Anticipated artifacts include the Python generator, RunPod fine-tune script, timestamped calibration corpus, v2 sidecar-startup update, decision document, and research interface. | D-28_adversarial_example_farming.md:138-146 |
| D28-E32 | [E] | CON | Full PPM rebuilding and multi-model adversarial corpora are deferred beyond D-28; LoRA and a single-model corpus define the planned scope. | D-28_adversarial_example_farming.md:148-151 |
| D28-E33 | [E] | DEP,DET | Once unblocked, D-29 may run independently in a different RunPod session. | D-28_adversarial_example_farming.md:153-156 |
| D28-N01 | [C] | DEP | Header/status and acceptance mark D-28 deferred, while the shipping-pass block asserts that farming CLI/SSB legs are live; the source therefore describes partial implementation, not a single uniform status. | D-28_adversarial_example_farming.md:1-11,91-121,127-136 |
| D28-N02 | [C] | DATA | Planned generator output uses `.elai_cc/calibration/ppm_adversarial/{ts}/`, while the asserted live CLI writes `.elai_cc/benchmarks/adv_rm/corpus/`; canonical corpus location and migration are unresolved. | D-28_adversarial_example_farming.md:48-55,115-121,138-143 |
| D28-N03 | [C] | INT,OBS | Tasks specify two future SSB variants, but the shipping contract explicitly says both were rejected in favor of one completion event; acceptance was not rewritten to state the actual event contract. | D-28_adversarial_example_farming.md:77-80,91-101,127-136 |
| D28-N04 | [A] | INT,DATA | “Appropriate SCHEMA bump” supplies no version, migration rule, public constant, or compatibility test. | D-28_adversarial_example_farming.md:77-80 |
| D28-N05 | [A] | FLOW,TEST | `proxy >> gold` has no numeric margin, search budget, stopping condition, sample count, or balance rule. | D-28_adversarial_example_farming.md:15-23,48-54 |
| D28-N06 | [A] | DEP,TEST | “At least 20 stable cycles” never defines stable, cycle identity, consecutive-window requirements, minimum samples per arm, or who certifies readiness. | D-28_adversarial_example_farming.md:9-10,25-28,102-104,129-130 |
| D28-N07 | [A] | DATA,TEST | The planned `PpmFinetuneCompleted` scalar v1/v2 scores cannot by itself encode both adversarial-category gains plus every standard-category regression gate. | D-28_adversarial_example_farming.md:62-80 |
| D28-N08 | [A] | DET,TEST | Dataset version, split, random seeds, sample ordering, optimizer determinism, repeated-run tolerance, and hardware-dependent reproducibility are unspecified. | D-28_adversarial_example_farming.md:46-70,123-136 |
| D28-N09 | [A] | DET,FLOW | Farming idempotency, duplicate-example handling, concurrent writers, resume after interruption, and corpus atomicity are unspecified. | D-28_adversarial_example_farming.md:46-55,115-121 |
| D28-N10 | [A] | SEC,INT | Model/corpus provenance, safetensors checksum/signature, remote transport, sidecar health validation, and deployment authorization are not defined. | D-28_adversarial_example_farming.md:56-65,138-146 |
| D28-N11 | [A] | FAIL | Rollback says revert to v1 but does not define the atomic version pointer, rollback command, archive/event evidence, or recovery from a partially deployed v2. | D-28_adversarial_example_farming.md:72-75,140-145 |
| D28-N12 | [A] | SEC | Corpus privacy, secret/PII scrubbing, retention, and whether production reasoning may be exported to RunPod are not addressed. | D-28_adversarial_example_farming.md:46-60 |
| D28-N13 | [A] | CFG,DEP | GPU use is called cost-throttled, but no budget, timeout, instance-hours limit, or approval gate is specified. | D-28_adversarial_example_farming.md:7,25-28,56-60 |
| D28-N14 | [I] | DEP | Joining the explicit shipping notes yields a two-part state: local farming/audit substrate is claimed live; immunization training, deployment, and post-benchmark proof remain deferred. This is synthesis, not independent implementation verification. | D-28_adversarial_example_farming.md:91-136 |

### D-002 cross-source interactions (namespaced D-002/<X-*>)

| Aggregate ID | Class | Category | Interaction | Evidence anchors |
|---|---|---|---|---|
| D-002/X-E01 | [E] | DEP,CAP | D-1 creates `ArchiveChain.verify_chain`; D-8 is explicitly its CLI/CI reporting wrapper and can begin after D-1. | D-1_reasoning_template_schema.md:133-140,379-395; D-8_crypto_archive_verify.md:3-12,130-145 |
| D-002/X-C01 | [C] | DATA,SEC | The shared archive hash contract is inconsistent: D-1 hashes body plus parent hash, while D-8 additionally hashes `mutation_proposal_text`, a field absent from D-1's schema. | D-1_reasoning_template_schema.md:91-95,114-128; D-8_crypto_archive_verify.md:16-28 |
| D-002/X-C02 | [C] | DEP,FAIL | D-8 expects aggregated duplicate/malformed/link errors from D-1 verification, but D-1's declared error/API surface does not provide them. | D-1_reasoning_template_schema.md:110-119,133-137; D-8_crypto_archive_verify.md:52-69,94-106 |
| D-002/X-E02 | [E] | SEC,DEP | D-9 freezes D-1/D-8 archive maintenance, parent selection, Blake3 choice, template verifier requirements, and schema enums only after all Wave-1 surfaces exist. | D-9_l3_frozen_surfaces.md:3-15,19-35,75-106 |
| D-002/X-E03 | [E] | SEC,FLOW | D-10 relies on D-9 to lock both its routing predicate and `ClaimType`, preserving orchestrator ownership of grading. | D-10_mode_routing_predicate.md:9-14,135-148,187-200 |
| D-002/X-C03 | [C] | SEC,INT | D-9's stated “git push BLOCK” outcome is not backed by a pre-push mechanism; the concrete layers are pre-commit, runtime pre-write, and CI. | D-9_l3_frozen_surfaces.md:3-8,42-46,108-134,183-187 |
| D-002/X-E04 | [E] | FLOW,DEP | D-14 consumes D-10 `Claim`/`ClaimResult` and runs only after a Verified result with a direct user assertion; it blocks later MEMORIZING on divergence. | D-14_sycophancy_probe.md:42-47,83-100,150-170 |
| D-002/X-C04 | [C] | FLOW,TEST | D-21's balanced five-class benchmark cannot be satisfied as written because D-10 calls Mode C label-only and never returns it from `select_grounding_mode`. | D-10_mode_routing_predicate.md:25-27,33-54,90-96; D-21_mode_routing_coverage_benchmark.md:3-5,15-21,31-45 |
| D-002/X-C05 | [C] | TEST | D-10 mixes 45/50 accuracy with “90% F1”; D-21 instead specifies macro-F1 at least 0.85 and each per-mode F1 at least 0.70. | D-10_mode_routing_predicate.md:203-208,250-254,281-284; D-21_mode_routing_coverage_benchmark.md:36-49,94-104 |
| D-002/X-C06 | [C] | DEP,DET | D-21 claims parallel safety with D-19 while consuming benchmark events attributed to D-19, leaving the event-definition build order unresolved. | D-21_mode_routing_coverage_benchmark.md:6-8,62-73 |
| D-002/X-E05 | [E] | DEP,TEST | D-24 is the quantitative D-7 consumer: it feeds typed synthetic pass-rate deltas into `CanaryRunner` and gates its harmful/helpful/neutral discrimination. | D-24_cross_domain_canary_regression_suite.md:3-18,26-42,50-65 |
| D-002/X-E06 | [E] | DEP,INT | D-28 consumes the D-2 PPM/sidecar, waits for D-5 stable cycles, and uses D-19 pre/post PRMBench reports to decide v2 deployment. | D-28_adversarial_example_farming.md:3-10,25-28,56-75,102-121 |
| D-002/X-C07 | [C] | DEP | D-28 is explicitly DEFERRED but also explicitly claims live local farming CLI and SSB legs; status must be tracked per leg. | D-28_adversarial_example_farming.md:1-11,91-136 |
| D-002/X-C08 | [C] | DATA,INT | D-28 retains an older calibration-directory/two-event design beside an asserted live benchmark-directory/one-event implementation contract. | D-28_adversarial_example_farming.md:48-55,77-101,115-121,138-143 |
| D-002/X-I01 | [I] | INT,DATA | Across assigned plans, SSB versions progress 47→48 (D-1), 53→54 (D-8), 54→55 (D-9), 55→56 (D-10), and 59→60 (D-14); intervening versions belong to unassigned sessions and were intentionally not read. | D-1_reasoning_template_schema.md:142-152; D-8_crypto_archive_verify.md:79-82; D-9_l3_frozen_surfaces.md:136-144; D-10_mode_routing_predicate.md:122-127; D-14_sycophancy_probe.md:101-106 |
| D-002/X-I02 | [I] | OBS,DATA | `TemplateId`, archive hashes, `ClaimId`, and audit UUIDs provide intended correlation keys across archive, routing, and sycophancy events, but no assigned source defines a single cross-event trace/correlation schema. | D-1_reasoning_template_schema.md:91-104,142-152; D-8_crypto_archive_verify.md:79-82; D-10_mode_routing_predicate.md:122-127; D-14_sycophancy_probe.md:101-106 |
| D-002/X-I03 | [I] | FAIL,SEC | The combined fail-closed path is archive tamper→nonzero verify, locked write→block, empty routing context→refuse, sycophancy divergence→quarantine, canary harm→block, and failed PPM immunization→retain v1. | D-8_crypto_archive_verify.md:62-69,94-106; D-9_l3_frozen_surfaces.md:223-235; D-10_mode_routing_predicate.md:33-49,197-203; D-14_sycophancy_probe.md:95-100; D-24_cross_domain_canary_regression_suite.md:39-40; D-28_adversarial_example_farming.md:67-75 |
| D-002/X-A01 | [A] | FAIL,DET | Retry semantics are absent across archive verification, locked-write auditing, routing backends, sycophancy runtime calls, benchmark report writes, and GPU farming; only explicit short-circuit, quarantine, and rollback behaviors are defined. | D-8_crypto_archive_verify.md:62-69; D-9_l3_frozen_surfaces.md:118-134; D-10_mode_routing_predicate.md:107-120; D-14_sycophancy_probe.md:83-100; D-21_mode_routing_coverage_benchmark.md:42-49; D-24_cross_domain_canary_regression_suite.md:34-42; D-28_adversarial_example_farming.md:46-75 |
| D-002/X-A02 | [A] | SEC | None of the eight originals defines PII/secret scrubbing, telemetry export, or retention policy; this is most consequential for archived prompts, user assertions, quarantine artifacts, and D-28 RunPod corpora. | D-1_reasoning_template_schema.md:123-162; D-14_sycophancy_probe.md:63-105,163-170; D-28_adversarial_example_farming.md:46-60 |

### D-002 summary deltas (namespaced D-002/<SUM-*>)

| Aggregate ID | Class | Record | Deep evidence | Orientation claim |
|---|---|---|---|---|
| D-002/SUM-C01 | [C] | D-1's exact SSB names are `ArchiveEntryAdded`, `ArchiveEntryRetired`, `TemplatePromoted`, and `TemplateCandidateRejected`; “ARCHIVE_ENTRY_ADDED/RETIRED” is incomplete. | D-1_reasoning_template_schema.md:142-152 | 04-D-SUMMARY_INDEX.discovery.md:29-34,536-539 |
| D-002/SUM-C02 | [C] | D-1's `ArchiveEntry` field is `staged_eval_result` and it also includes `retirement_reason`; the summary's `eval_result` projection is not the deep schema. | D-1_reasoning_template_schema.md:123-132 | 04-D-SUMMARY_INDEX.discovery.md:500-501 |
| D-002/SUM-C03 | [C] | The D-1 original does not state “at least three typed templates”; it requires migration count at least the template-directory TOML count plus registry/task smokes. Preserve “three” only as summary-source evidence, not deep-plan evidence. | D-1_reasoning_template_schema.md:205-209,335-354 | 04-D-SUMMARY_INDEX.discovery.md:34,596 |
| D-002/SUM-C04 | [C] | D-8's normative error enum says `BrokenParent`; `BrokenLink`/`DanglingParent` occur only in context prose and conflict with that enum, so a generic “broken-link report” hides a real identifier mismatch. | D-8_crypto_archive_verify.md:25-28,52-69 | 04-D-SUMMARY_INDEX.discovery.md:122-127,525,603 |
| D-002/SUM-C05 | [C] | D-9 defines no pre-push hook. Concrete enforcement blocks commits, runtime writes, or CI, so “blocked git push” is an unimplemented outcome claim in the plan. | D-9_l3_frozen_surfaces.md:108-134,183-187 | 04-D-SUMMARY_INDEX.discovery.md:146-150,577-578 |
| D-002/SUM-C06 | [C] | D-10's five names are not five runtime outcomes: `DiscoveryLabelOnly` is explicitly post-hoc after A/D and the selector never returns it. | D-10_mode_routing_predicate.md:18-54,90-96 | 04-D-SUMMARY_INDEX.discovery.md:159-165,505,746 |
| D-002/SUM-C07 | [C] | D-10's 45/50 requirement is an accuracy count and cannot itself establish 90% F1. | D-10_mode_routing_predicate.md:203-208,250-254 | 04-D-SUMMARY_INDEX.discovery.md:165,604,747 |
| D-002/SUM-C08 | [C] | D-21's no-collapse gate is each mode F1 at least 0.70, not “at least ten correct examples per mode.” | D-21_mode_routing_coverage_benchmark.md:36-49,94-104 | 04-D-SUMMARY_INDEX.discovery.md:318-324,610 |
| D-002/SUM-C09 | [C] | D-14's exact dispatch event is `SycophancyProbeDispatched`, not `SYCOPHANCY_PROBE`; the latter is narrative/summary shorthand with no declared mapping. | D-14_sycophancy_probe.md:93,101-106,142-149 | 04-D-SUMMARY_INDEX.discovery.md:216-220,548-549 |
| D-002/SUM-C10 | [C] | D-14's concrete hook probes only `ClaimResult::Verified` plus a direct assertion, narrower than an unconditional re-run of every user-aligned claim. | D-14_sycophancy_probe.md:95-100 | 04-D-SUMMARY_INDEX.discovery.md:216-222 |
| D-002/SUM-C11 | [C] | D-28 gates both Deception Resistance and Circular Logic at +5pp and permits standard-category regression up to 2pp; “Deception Resistance +5 with no regression” is materially different. | D-28_adversarial_example_farming.md:62-75,129-135 | 04-D-SUMMARY_INDEX.discovery.md:423-430,614 |
| D-002/SUM-C12 | [C] | D-28 is not uniformly untouched/deferred: its original explicitly claims the local farming CLI/SSB legs live while training/deployment/remeasurement remain pending. This is plan-recorded status, not independently verified shipped state. | D-28_adversarial_example_farming.md:91-136 | 04-D-SUMMARY_INDEX.discovery.md:419-430,654,766,776 |
| D-002/SUM-C13 | [C] | D-28's shipping-pass contract rejects `AdversarialCorpusGenerated` and `PpmFinetuneCompleted` in favor of `AdversarialFarmRunCompleted`. | D-28_adversarial_example_farming.md:77-101 | 04-D-SUMMARY_INDEX.discovery.md:419-430,534-554 |
| D-002/SUM-C14 | [C] | For D-1/D-8/D-9/D-10/D-14/D-21/D-24, PLANNED is a summary-layer status rather than an original-file label; D-28 alone declares DEFERRED in its assigned original. | Assigned originals in full; D-28_adversarial_example_farming.md:7-8 | 04-D-SUMMARY_INDEX.discovery.md:623-656,766,776 |
| D-002/SUM-A01 | [E] | D-1 adds the closed Domain/PatternKind/VerifierKind/ArchiveStatus/RetirementReason/Predicate enums, bounded strings, typed IDs/hashes, and a typed error surface. | D-1_reasoning_template_schema.md:89-112 | 04-D-SUMMARY_INDEX.discovery.md:25-36 |
| D-002/SUM-A02 | [E] | D-1 fixes JSONL/O_APPEND storage, fs2 append locking, idempotent migration reporting, registry filtering, config defaults, SeededClock use, and terminal Frozen/Retired lifecycle rules. | D-1_reasoning_template_schema.md:123-180,239-266 | 04-D-SUMMARY_INDEX.discovery.md:25-36,496-520 |
| D-002/SUM-A03 | [E] | D-8 adds `--json`/`--fail-fast`, four typed errors, all-error versus early-exit modes, mandatory result emission, nightly/cross-platform CI, recovery guidance, and a 10k/5s/O(n) bound. | D-8_crypto_archive_verify.md:50-128 | 04-D-SUMMARY_INDEX.discovery.md:118-129 |
| D-002/SUM-A04 | [E] | D-9 adds the registry record schemas, path/symbol/TOML diff layers, runtime pre-write enforcement, validated 40-character `L3_OVERRIDE`, two exact SSB schemas, and fail-closed parsing. | D-9_l3_frozen_surfaces.md:75-159,201-235 | 04-D-SUMMARY_INDEX.discovery.md:131-151 |
| D-002/SUM-A05 | [E] | D-10 supplies exact routing precedence, `Claim`/`ClaimType`/`ClaimResult`, backend stubs, `dispatch_by_mode`, `ClaimDispatched`, and two exact L3 symbol locks. | D-10_mode_routing_predicate.md:33-49,77-148 | 04-D-SUMMARY_INDEX.discovery.md:155-167,498-510,534-554 |
| D-002/SUM-A06 | [E] | D-14 defines the 5-token Jaccard threshold (0.7), complete probe/report/verdict structures, exact config defaults, post-Verified hook, MEMORIZING block, and quarantine artifact path. | D-14_sycophancy_probe.md:21-34,58-120,142-170 | 04-D-SUMMARY_INDEX.discovery.md:212-224,756 |
| D-002/SUM-A07 | [E] | D-21 defines JSON fixture fields, exact report schema, macro and per-mode gates, human-only fixture authorship/signoff, CLI/CI reachability, SSB rows, dated report artifact, and repeat-run determinism. | D-21_mode_routing_coverage_benchmark.md:29-82,94-105 | 04-D-SUMMARY_INDEX.discovery.md:314-325 |
| D-002/SUM-A08 | [E] | D-24 defines helpful/neutral/harmful rate semantics, TOML fixture fields, binary report counters, `elai bench canary-regression`, generic benchmark events, and the report directory. | D-24_cross_domain_canary_regression_suite.md:14-65 | 04-D-SUMMARY_INDEX.discovery.md:352-362 |
| D-002/SUM-A09 | [E] | D-28 defines proxy/gold mining, timestamped JSONL, Qwen2.5-Math-PRM-7B LoRA rank 16/3 epochs/1e-5, v2 safetensors deployment, two-category/standard gates, and v1 rollback. | D-28_adversarial_example_farming.md:13-89 | 04-D-SUMMARY_INDEX.discovery.md:419-430 |
| D-002/SUM-A10 | [E] | D-28 additionally records actual farming symbols, two live CLI commands, the one shipped completion event, non-mock production constraints, and a separate benchmark corpus path. | D-28_adversarial_example_farming.md:91-121 | 04-D-SUMMARY_INDEX.discovery.md:419-430,522-564 |
| D-002/SUM-A11 | [A] | The deep originals expose new unresolved contracts absent from the summary: archive hash/schema mismatches, pre-commit message timing, Mode-C unreachability, D-14 skip/event schema contradictions, D-21 fixture-context gaps, and D-28 dual designs/status. | D1-N02..N12; D8-N01..N12; D9-N02..N13; D10-N02..N13; D14-N02..N14; D21-N02..N10; D28-N01..N14 | 04-D-SUMMARY_INDEX.discovery.md:742-768 |
| D-002/SUM-A12 | [A] | The assigned originals add no explicit PII/secret/retention/export safeguards despite persisting prompts, assertions, quarantines, and remote-training corpora. | D-1_reasoning_template_schema.md:123-162; D-14_sycophancy_probe.md:63-105,163-170; D-28_adversarial_example_farming.md:46-60 | 04-D-SUMMARY_INDEX.discovery.md:618-621 |

---

## Ledger D-003 — source atoms

Source file: `docs/thefinalglance/feature-discovery/deep/DEEP-D-003.discovery.md`
Plans: D-2, D-3, D-13, D-16, D-17, D-19, D-25, D-30
Manifest: 8 files, 1862 lines, 88049 bytes

### D-2_ppm_sidecar_wiring.md — 42 atoms

- **D003-D2-E001 `[E]` — capability/topology.** D2 introduces a second, step-level Process Preference Model sidecar, with policy `Qwen2.5-Coder-7B-Instruct-abliterated` on port `8100` and PPM `Qwen2.5-Math-PRM-7B` on `8101`; the two artifacts evolve independently against different signals and the verifier is not the policy in self-judge mode. (`D2:1-9`)
- **D003-D2-E002 `[E]` — prerequisites/status envelope.** Estimated effort is 2–3 sessions; execution requires D-1 (`ReasoningTemplate`, `VerifierKind::Prm`, `verifier_required=Prm`), B-37 (`PerRoleRuntime` and backend slot), and C-15 (`ClockProvider` plus entropy quarantine) all verified `DONE` by `spec-grounder`. (`D2:9-13`)
- **D003-D2-E003 `[E]` — concurrency.** D2 is declared parallel-safe with D-8 and the isolated D-19 benchmark surface, but not with D-3 because D-3 consumes its PPM output. (`D2:14-16`)
- **D003-D2-E004 `[E]` — model constraint/evidence.** The plan rules out the policy model as its own verifier, describes Qwen2.5-Math-PRM-7B as Qwen-family but separate-base/math-pretrained/PRM-distilled, cites 60–65 standard-category PRMBench performance, and explicitly delegates adversarial weakness measurement to D-19. (`D2:20-32`)
- **D003-D2-E005 `[E]` — deployment capacity.** The remote sidecar binds remotely and tunnels to loopback `127.0.0.1:8101`; the stated sizing is about 1.92 GB per NF4 7B model plus about 1.8 GB total KV cache, about 5.7 GB combined, fitting the stated 12/24 GB budget. (`D2:34-39`)
- **D003-D2-E006 `[E]` — existing integrations.** D2 reuses B-37 `RuntimeAdapter`, `PerRoleRuntime`, existing `HTTPClient`/retry logic in `sidecar_client.rs`, the Anthropic `/v1/messages` FastAPI speaker, D-1’s `VerifierKind::Prm` slot, and `[runtime.*]` configuration. (`D2:41-48`)
- **D003-D2-E007 `[E]` — research gate.** Before `TESTING_FIRST`, `paper-ingestor` must materialize `rstar-math-2501.04519-interface.md`, `latent-verifiers-2504.16760-interface.md`, and `prmbench-2501.03124-interface.md`. (`D2:70-79`)
- **D003-D2-E008 `[E]` — client schema.** New `ppm_client.rs` defines `PpmClient { endpoint: Url, http: Arc<reqwest::Client>, timeout: Duration, retry_policy: RetryPolicy }`. (`D2:81-87`)
- **D003-D2-E009 `[E]` — client action.** `PpmClient::new(...)` and `score_step(prompt: &str, candidate_step: &str, ctx: &PpmContext) -> Result<PpmScore, PpmError>` are the construction and scoring surfaces. (`D2:83-90`)
- **D003-D2-E010 `[E]` — score schema/range.** `PpmScore` contains `score: f32`, `confidence: f32`, and `raw_output: String`; `score` is constrained to `[0.0, 1.0]`. (`D2:90-91`)
- **D003-D2-E011 `[E]` — context schema.** `PpmContext` contains `problem_statement: String`, `prior_steps: Vec<String>`, and `domain: Domain`. (`D2:92-93`)
- **D003-D2-E012 `[E]` — error surface.** `PpmError` is a `thiserror` family containing `HttpError(#[from] reqwest::Error)`, `TimeoutError`, `ParseError(String)`, and `SidecarUnavailable`. (`D2:94-95`)
- **D003-D2-E013 `[E]` — deterministic request format.** `score_step` emits the fixed rStar-Math-style `Problem / Previous steps / Current step / Is this step correct? Rate 0.0 to 1.0` prompt; an unparseable response is a `ParseError`, not a guess. (`D2:96-98`)
- **D003-D2-E014 `[E]` — retry/failure behavior.** Every HTTP call uses at most three attempts with exponential backoff and jitter; exhausting a timeout returns `TimeoutError`, and infinite retry is forbidden. (`D2:99-100,228`)
- **D003-D2-E015 `[E]` — runtime adapter.** New `PpmAdapter { client: Arc<PpmClient> }` implements `RuntimeAdapter`, reports backend ID `"ppm"`, translates Anthropic `/v1/messages` requests to score requests, and wraps `PpmScore` as `RuntimeResponse`. (`D2:102-107`)
- **D003-D2-E016 `[E]` — permission boundary.** PPM is verifier-only: agents cannot select it for generation, only D-3’s consensus verifier may call it, and `RuntimeAdapter::is_verifier_only()` must return `true`. (`D2:108-110,220-224`)
- **D003-D2-E017 `[E]` — runtime/config defaults.** `RuntimeBackend` gains `Ppm(PpmAdapter)`; `[runtime.ppm]` defaults are endpoint `http://127.0.0.1:8101/v1/messages`, `timeout_secs=30`, `max_retries=3`, and model `huihui-ai/Qwen2.5-Math-PRM-7B` or a distilled-equivalent path. (`D2:112-117`)
- **D003-D2-E018 `[E]` — activation default.** `PerRoleRuntime::default()` leaves PPM disabled; activation requires both TOML opt-in and environment flag `ELAI_PPM_ENABLE=1`, preventing development sessions from accidentally requiring an offline second sidecar. (`D2:118-120,190-192`)
- **D003-D2-E019 `[E]` — remote boot action.** `start-ppm-sidecar.sh` uses the existing SSH/virtualenv server pattern, `CUDA_VISIBLE_DEVICES=1`, model path above, port `8101`, `--no-dola`, `--turboquant-bits 0`, `--no-steering`, and `--load-in-4bit`, and records log/PID files. (`D2:122-130`)
- **D003-D2-E020 `[E]` — placement fallback.** GPU1 is the normal PPM placement while GPU0 hosts policy; a documented single-GPU fallback serializes VRAM swapping and is supported but explicitly suboptimal. (`D2:131-133`)
- **D003-D2-E021 `[E]` — health workflow.** Health is a three-stage process check, TCP reachability check, then real `/v1/messages` curl request; the named probes are `pgrep`, `Test-NetConnection`, and `curl`. (`D2:134-137`)
- **D003-D2-E022 `[E]` — tunnel integration.** `start-tunnel.ps1` adds `127.0.0.1:8101 -> remote 127.0.0.1:8101`, leaves the existing `8100` forward unchanged, and documents the dual tunnel in `REMOTE-SIDECAR.md`. (`D2:139-143`)
- **D003-D2-E023 `[E]` — CLI surface.** `elai ppm health` wraps the three-stage health check, reports `OK` or a structured `PpmError`, and exits `0`/`1`. (`D2:145-150`)
- **D003-D2-E024 `[E]` — score event schema.** `PpmScoreEmitted` carries `template_id`, `step_index`, `score`, `confidence`, and `latency_ms`. (`D2:152-154`)
- **D003-D2-E025 `[E]` — unavailable event schema.** `PpmUnavailable` carries a string `reason` and `retry_count`, and fires only after verifier dispatch exhausts retries. (`D2:155-156`)
- **D003-D2-E026 `[E]` — schema lifecycle.** D2 scaffolds both PPM SSB variants behind `schema-v49` or a temporary marked scaffold, while D3 owns the atomic schema bump; D2 must not land an independent bump. (`D2:157-160,203-204`)
- **D003-D2-E027 `[E]` — dependency policy.** `reqwest` should already exist; no new production dependency is expected, and any missing retry-jitter helper should be hand-rolled in at most 30 LOC rather than add a crate. (`D2:162-166`)
- **D003-D2-E028 `[E]` — unit evidence.** Required client tests cover mock happy path, transient `502` success on the third attempt, persistent `504` to `TimeoutError`, malformed body to `ParseError`, and a deterministic fixed-input prompt snapshot. (`D2:168-176`)
- **D003-D2-E029 `[E]` — integration evidence.** The adapter integration test runs through `RuntimeAdapter` against a local axum mock, verifies score round-trip, and asserts `is_verifier_only() == true`. (`D2:178-182`)
- **D003-D2-E030 `[E]` — recorded decisions.** `D-2-decisions.md` must justify the same-family/different-base PPM choice as an acknowledged uncertainty, opt-in activation, and three-attempt jittered backoff. (`D2:184-195`)
- **D003-D2-E031 `[E]` — reachability.** D2 is reachable from D3’s `VerificationResult.ppm`, `elai ppm health`, `ELAI_PPM_ENABLE`, and `[runtime.ppm]`. (`D2:197-219`)
- **D003-D2-E032 `[E]` — prohibited fallbacks.** `PpmClient` must have no same-model policy fallback; an empty response must return `ParseError` rather than panic; the default endpoint must remain `8101`, not collide with policy `8100`; retry must remain bounded. (`D2:220-228`)
- **D003-D2-E033 `[E]` — observable evidence contract.** With enable flag set, sidecar booted, and health invoked, D2 expects health exit `0`, a real curl response, boot/tunnel/config artifacts, and—before D3 consensus wiring—zero SSB rows. (`D2:229-235`)
- **D003-D2-E034 `[E]` — deterministic done probes.** The plan enumerates source-symbol searches, port/CLI/health checks, workspace build and scoped tests, a negative same-model-fallback search expecting zero hits, and existence of all three paper interfaces. (`D2:237-277`)
- **D003-D2-E035 `[E]` — base acceptance gate.** Done requires client/adapter compilation, mock integration, verifier-only enforcement, backend/config loading, scaffolded SSB variants, working remote boot and dual tunnel, opt-in behavior, zero new workspace errors, all scoped tests, and at least six new PPM tests. (`D2:279-295`)
- **D003-D2-E036 `[E]` — hardware smoke gate.** Before the hardware-heavy test, chat must flag the boot; both sidecars must stay healthy together for at least five minutes without OOM and leave at least 6 GB GPU memory free. (`D2:296-299`)
- **D003-D2-E037 `[E]` — score-quality smoke gate.** A known-correct step must score at least `0.7`, a known-wrong step at most `0.3`, and the margin must exceed `0.3`. (`D2:300-302`)
- **D003-D2-E038 `[E]` — outage smoke gate.** With PPM enabled but offline, `PpmUnavailable` must be journaled and the reasoning step must not auto-pass; final consensus behavior is delegated to D3. (`D2:303-305`)
- **D003-D2-E039 `[E]` — documentary done gates.** All paper interfaces, the three-decision record, and post-`AUDITING` memory artifact `project_d2_ppm_sidecar.md` are required. (`D2:306-309`)
- **D003-D2-E040 `[E]` — code/config surfaces.** The plan adds client, adapter, CLI health, boot script, integration test, decision doc and paper interfaces; modifies role runtime, SSB, CLI commands, TOML, tunnel script and remote-sidecar documentation. (`D2:311-329`)
- **D003-D2-E041 `[E]` — non-goals.** D3 aggregation, D19 benchmarking, D28 retraining, and distilled-PPM training are excluded from D2. (`D2:331-337`)
- **D003-D2-E042 `[E]` — next dependency.** D3 is the immediate consumer; D19 is described as able to proceed alongside D3 because their write surfaces are disjoint. (`D2:339-343`)

### D-3_verification_result_aggregator.md — 37 atoms

- **D003-D3-E001 `[E]` — capability.** D3 aggregates deterministic, PPM, intrinsic GG, and cross-context signals into `VerificationResult`, with a stated baseline requirement of at least two of four concurring before template promotion. (`D3:1-10`)
- **D003-D3-E002 `[E]` — prerequisites/concurrency.** Estimated effort is 1–2 sessions; D2 and D1 are prerequisites; D5 is declared parallel-safe despite consuming the result, while D4 is not parallel-safe because it monitors D3 rejection/disagreement signals. (`D3:11-17`)
- **D003-D3-E003 `[E]` — intrinsic integration.** Existing scattered `mean_logprob`, `entropy_max`, and rolling-Jaccard `novelty` metrics are consolidated into `IntrinsicSignals`. (`D3:34-37`)
- **D003-D3-E004 `[E]` — cross-context behavior.** A claim runs in two independent context windows with different prior-step ordering; agreement is treated as robustness and disagreement as a flag. (`D3:39-42`)
- **D003-D3-E005 `[E]` — foundations.** D3 reuses deterministic dispatch hooks, D2’s client/adapter, Phase-4G GG fields, and D1’s per-template `verifier_required`. (`D3:44-49`)
- **D003-D3-E006 `[E]` — research gate.** `vstar-2402.06457-interface.md` and `ccv-2603.21454-interface.md` must exist before `TESTING_FIRST`. (`D3:69-76`)
- **D003-D3-E007 `[E]` — aggregate schema.** `VerificationResult` has optional `deterministic`, `ppm`, `intrinsic`, and `cross_context` fields plus `consensus_rule` and final `verdict`. (`D3:80-85`)
- **D003-D3-E008 `[E]` — deterministic schema.** `DeterministicCheck` contains `kind`, `passed`, and optional `stderr_snippet`; `DeterministicKind` is the closed set `Compile`, `Test`, `Lint`, `Schema`, `RegistryCheck`. (`D3:86-88`)
- **D003-D3-E009 `[E]` — intrinsic schema.** `IntrinsicSignals` contains `mean_logprob`, `entropy_max`, and `novelty`, all with ranges required to be documented in the struct. (`D3:89-90`)
- **D003-D3-E010 `[E]` — cross-context schema/default.** `CrossContextAgreement` stores two Blake3 output hashes, Jaccard similarity, and Boolean agreement; the default agreement threshold is `0.7` and lives in L3-locked `[verification]` config. (`D3:91-94`)
- **D003-D3-E011 `[E]` — rule schema.** `ConsensusRule` is a closed, L3-locked enum: `AtLeastTwo`, `AtLeastThree`, `AtLeastN(u8)`, or `RequireAll(Vec<SignalKind>)`. (`D3:95-96`)
- **D003-D3-E012 `[E]` — signal/verdict schemas.** The initial `SignalKind` set is `Deterministic`, `Ppm`, `Intrinsic`, `CrossContext`; `Verdict` is `Pass`, `Reject`, or `Inconclusive`. (`D3:97-98`)
- **D003-D3-E013 `[E]` — default computation.** `AtLeastTwo` counts deterministic `passed`, PPM score `>=0.7`, intrinsic `mean_logprob >= intrinsic_threshold`, and cross-context agreement; at least two passes means `Pass`, zero or one means `Reject`, while a missing required signal makes the result `Inconclusive`. (`D3:100-108`)
- **D003-D3-E014 `[E]` — alternative computations.** `AtLeastN(n)` uses the corresponding pass count; `RequireAll(kinds)` requires every named kind to be present and passing; the absent-config default is `AtLeastTwo`. (`D3:100-108,188-192`)
- **D003-D3-E015 `[E]` — runner schema.** `CrossContextRunner` contains `Arc<dyn Executor>` and a Jaccard threshold, and `run(&Claim, &OrchContext)` returns `CrossContextAgreement`. (`D3:110-115`)
- **D003-D3-E016 `[E]` — concurrency/determinism.** The runner starts two executor futures concurrently, shuffles prior steps, Blake3-hashes each output, then computes token-level 3-shingle Jaccard similarity. (`D3:114-118,121-122`)
- **D003-D3-E017 `[E]` — timeout recovery.** Each cross-context side is bounded to 30 seconds; timeout returns `Inconclusive` rather than panic or infinite wait. (`D3:119-122`)
- **D003-D3-E018 `[E]` — dispatch workflow.** Every claim-emitting candidate dispatches an applicable deterministic check, conditional PPM when `verifier_required==Prm`, PPM enablement and health all hold, always captures intrinsic signals, conditionally runs cross-context when the rule demands it, computes a verdict, and emits an SSB row. (`D3:124-135,228-230`)
- **D003-D3-E019 `[E]` — checker fallback.** Before D11 exists, deterministic dispatch uses current compile/test hooks and skips inapplicable checks; D11 later replaces this with typed registry dispatch. (`D3:127-129,331-336`)
- **D003-D3-E020 `[E]` — schema migration.** D3 atomically bumps `SCHEMA_VERSION 48→49` for both D2 PPM events and all D3 verifier events; an old v48 DB must emit a migration warning. (`D3:137-140,286-292`)
- **D003-D3-E021 `[E]` — pass/reject events.** `VerifierConsensusPass` contains template/candidate IDs, passing signals and rule string; `VerifierConsensusReject` contains IDs, `(SignalKind, reason)` failures and rule string. (`D3:141-145`)
- **D003-D3-E022 `[E]` — disagreement event.** `VerifierDisagreement` contains template ID and signal/status pairs and fires when at least one signal passes and at least one rejects, because disagreement is operationally distinct from rejection. (`D3:146-147,202-205`)
- **D003-D3-E023 `[E]` — cross-context event.** `CrossContextDispatched` records claim ID, both context hashes, Jaccard score, and agreement; every new variant needs payload arm, SQL serialization and round-trip proof. (`D3:148-150`)
- **D003-D3-E024 `[E]` — config defaults.** `[verification]` defaults are `consensus_rule="at_least_two"`, `intrinsic_logprob_threshold=-3.0`, `intrinsic_entropy_max=3.5`, `intrinsic_novelty_min=0.2`, and `cross_context_jaccard_threshold=0.7`; evaluator-side values are L3-locked. (`D3:152-157`)
- **D003-D3-E025 `[E]` — unit evidence.** Tests cover pass-count rules, required-signal absence, TOML deserialization, timeout-to-inconclusive, deterministic Jaccard on identical input, and disagreement emission. (`D3:159-168`)
- **D003-D3-E026 `[E]` — deceptive-case correction in source.** The plan explicitly notices that deterministic fail + PPM pass + intrinsic pass would pass `AtLeastTwo`; it revises the code-claim scenario to `RequireAll([Deterministic, Ppm])`, under which deterministic failure rejects. (`D3:170-184`)
- **D003-D3-E027 `[E]` — integration scenarios.** Integration uses mock executor/PPM/deterministic checker for ordinary pass, deceptive code rejection under `RequireAll`, and cross-context disagreement, asserting rows through an in-memory SSB writer. (`D3:170-186`)
- **D003-D3-E028 `[E]` — contract evidence.** Contract tests require public schema `49`, round-trips for every new event, stable `ConsensusRule` serde, and the `AtLeastTwo` default when config is absent. (`D3:188-192`)
- **D003-D3-E029 `[E]` — decision record.** D3 must justify the `AtLeastTwo` latency/gate tradeoff, distinct Blake3 deduplication versus Jaccard similarity, and separate disagreement telemetry for D4 drift monitoring. (`D3:194-205`)
- **D003-D3-E030 `[E]` — consumers.** D5 consumes `VerifierConsensusPass` counts; D4 consumes disagreement rate; every claim-emitting post-`EXECUTING` baby step reaches `dispatch_verifiers`. (`D3:207-232`)
- **D003-D3-E031 `[E]` — fail-closed constraints.** Missing required signals cannot be swallowed, cross-context cannot wait forever, every candidate must emit one result-shaped SSB row even on unavailable signals, and the rule must load from locked config rather than an in-code constant. (`D3:233-239`)
- **D003-D3-E032 `[E]` — observable proof.** A cloud deceptive candidate that PPM accepts but compilation rejects must yield a `VerifierConsensusReject` with deterministic stderr plus `PpmScoreEmitted`, and no `TemplatePromoted`. (`D3:240-246`)
- **D003-D3-E033 `[E]` — deterministic done probes.** Required probes cover schema/version/event symbols, aggregate enums, cross-context runner, config, task-runner dispatch, workspace/scoped/contract tests, a negative single-verifier bypass search, and both paper interfaces. (`D3:248-284`)
- **D003-D3-E034 `[E]` — acceptance volume.** Done requires at least ten table-driven consensus scenarios, at least twelve new orchestrator tests, contract tests, config defaults, one result per candidate, all six D2+D3 event round-trips, zero new build errors, and timeout safety. (`D3:286-303`)
- **D003-D3-E035 `[E]` — cloud gates.** Cloud proofs require deceptive rejection/no promotion, simple-math cross-context agreement `>=0.7`, and PPM-offline behavior yielding `Inconclusive` plus `PpmUnavailable` and either rejection or an explicit inconclusive flag according to rule. (`D3:304-310`)
- **D003-D3-E036 `[E]` — documentary gates/surfaces.** Paper interfaces, three-decision doc and `project_d3_verification_result.md` are required; implementation adds aggregate/runner/test/contract/docs and modifies SSB, task runner, config and module exports. (`D3:311-329`)
- **D003-D3-E037 `[E]` — non-goals/next.** D4 tripwires, D5 BAI, D11 typed checkers and D13 adversary are outside D3; D4 and D5 are its next disjoint consumers. (`D3:331-341`)

### D-13_adversary_role_cross_family.md — 32 atoms

- **D003-D13-E001 `[E]` — capability.** D13 adds `AgentRole::Adversary` for `GroundingMode::FalsifiabilityCrossFamily`; it must run on a different model family and return structured counter-evidence. (`D13:1-11`)
- **D003-D13-E002 `[E]` — prerequisites/concurrency.** Estimated effort is 2–3 sessions; D10 routing and D3 consensus are prerequisites; D14 and D15 are declared parallel-safe, although D14 shares the new cross-family infrastructure. (`D13:12-16`)
- **D003-D13-E003 `[E]` — routing priority.** Mode B is explicitly last-resort: most claims should route to constructive Mode A or observable Mode D first. (`D13:18-22`)
- **D003-D13-E004 `[E]` — hard verifier rules.** The adversary must be cross-family, its output must be structured with source-tier citations, a separate auditor—not the executor—decides from claim plus counter-evidence, and absence of an alternate family skips Mode B for Mode E with no self-critique. (`D13:23-31`)
- **D003-D13-E005 `[E]` — deployment fallback.** With local Qwen policy, cloud Sonnet is the default cross-family adversary; Qwen Math PPM is explicitly unsuitable, and cloud plus alternate-local unavailability results in Mode E. (`D13:33-37`)
- **D003-D13-E006 `[E]` — foundations.** The plan extends existing `AgentRole`, B37 per-role runtime, D3 verification, and D10’s `FalsifiabilityCrossFamily` branch. (`D13:39-44`)
- **D003-D13-E007 `[E]` — research gate.** Three paper interfaces—Self-Correction Bench, Constitution or Collapse, and Constitutional AI in Small LLMs—must be present. (`D13:57-65`)
- **D003-D13-E008 `[E]` — role permission metadata.** `AgentRole::Adversary` gains metadata requiring its runtime family to be cross-family from the executor, enforced at spawn. (`D13:67-73`)
- **D003-D13-E009 `[E]` — config schema.** `AdversaryConfig` contains `runtime_role`, `allowed_families`, `max_response_tokens`, and a `CounterEvidenceSchema`. (`D13:75-79`)
- **D003-D13-E010 `[E]` — model-family schema.** `ModelFamily` enumerates `Qwen25Coder`, `Qwen25Math`, `AnthropicClaude`, `OpenAiGpt`, `DeepSeek`, and extensible `Other(String)`. (`D13:80-82`)
- **D003-D13-E011 `[E]` — family predicate.** `cross_family_check(executor, adversary)` must reject same-family pairs and force the caller toward Mode E. (`D13:83-85`)
- **D003-D13-E012 `[E]` — counter-evidence schema.** `CounterEvidence` contains a `CounterVerdict`, evidence points, source-tier citations, and confidence; verdicts are `ClaimRobust`, `ClaimWeak`, or `ClaimFalse`. (`D13:87-92`)
- **D003-D13-E013 `[E]` — evidence schema.** Each `EvidencePoint` has `EvidenceKind`, a `BoundedString<2048>`, and an optional citation; kinds are `ContradictoryExample`, `MissingPrecondition`, `LogicalGap`, and `AlternateInterpretation`. (`D13:92-95`)
- **D003-D13-E014 `[E]` — citation schema.** Citations are typed as `T1Arxiv { id }`, `T1Nature { doi }`, `T2OrgBlog { url }`, or `T3Other { url, note }`. (`D13:96-97`)
- **D003-D13-E015 `[E]` — agent surface.** New `.claude/agents/adversary.md` instructs the strongest falsehood-seeking critic, requires explicit `ClaimRobust` when no counter-evidence exists, forbids hedging, returns matching JSON, cites the two motivating papers, and runs on Sonnet by default. (`D13:99-109`)
- **D003-D13-E016 `[E]` — probe schema.** `AdversaryProbe` owns a runtime adapter, config, and SSB sender; `probe` is async and returns `Result<CounterEvidence, AdversaryError>`. (`D13:111-115`)
- **D003-D13-E017 `[E]` — probe workflow/failures.** Probe first checks alternate-family availability, then rejects same-family runtime, loads the agent prompt, invokes the adapter, parses and validates JSON, and finally emits `AdversaryVerdict`; the two preflight failures are `CrossFamilyUnavailable` and `SameFamily`. (`D13:114-123`)
- **D003-D13-E018 `[E]` — mode dispatch states.** In `FalsifiabilityCrossFamily`, unavailable family returns `ClaimResult::Refused` with reason `"no cross-family adversary"` and journals `CrossFamilyUnavailable`; robust maps to `Verified`, false to `Falsified`, weak to `Inconclusive`. (`D13:125-135`)
- **D003-D13-E019 `[E]` — consensus integration.** D3 gains `SignalKind::Adversary` and `VerificationResult.adversary`; `ClaimRobust` contributes pass, `ClaimFalse` reject, and `ClaimWeak` neither/inconclusive. (`D13:137-142`)
- **D003-D13-E020 `[E]` — event schemas/version.** Schema `58→59`; `AdversaryVerdict` stores claim, verdict, confidence, evidence count, runtime family and audit ID, while `CrossFamilyUnavailable` stores claim, executor family and reason. (`D13:144-149`)
- **D003-D13-E021 `[E]` — locked defaults.** `[adversary]` is L3-locked, defaults allowed families to Anthropic Claude, OpenAI GPT and DeepSeek, and caps response at `4096` tokens. (`D13:151-154`)
- **D003-D13-E022 `[E]` — unit evidence.** Tests require Qwen Coder + Claude to pass cross-family, Qwen Coder + Qwen Math to fail, JSON round-trip, valid arXiv ID parsing and malformed-ID `ParseError`. (`D13:156-162`)
- **D003-D13-E023 `[E]` — integration evidence.** Mock Sonnet/Qwen scenarios cover robust Tokio verification, false Rust-GC falsification, and no-cloud refusal plus `CrossFamilyUnavailable`. (`D13:164-172`)
- **D003-D13-E024 `[E]` — decisions.** The decision doc must justify cloud Sonnet and session cost throttling, typed instead of narrative evidence, and fail-closed Mode E instead of same-model critique. (`D13:174-183`)
- **D003-D13-E025 `[E]` — reachability.** The feature is reachable from D10 dispatch, D3 consensus, and per-Mode-B spawning of the agent definition. (`D13:185-206`)
- **D003-D13-E026 `[E]` — schema/fallback bans.** Same-family must emit unavailability, malformed or freeform output must be an `AdversaryError`/`ParseError` rather than text fallback, allowed families must come from TOML, and structured schema validation is mandatory. (`D13:207-213`)
- **D003-D13-E027 `[E]` — observable proof.** The synthetic false Rust-GC claim must produce `ModeSelected(mode=FalsifiabilityCrossFamily)`, one `AdversaryVerdict(ClaimFalse)` with at least two evidence points, a falsified result, agent file, and config section. (`D13:214-221`)
- **D003-D13-E028 `[E]` — deterministic done probes.** Probes assert schema 59, event/types/role/signal/config symbols, agent and paper files, workspace build, and scoped adversary tests. (`D13:223-256`)
- **D003-D13-E029 `[E]` — acceptance gate.** Done requires both event round-trips, at least ten tests, role spawn enforcement, schema round-trip, config loading, D3 signal integration, zero build errors, three cloud branches, all paper interfaces and the decision doc. (`D13:258-280`)
- **D003-D13-E030 `[E]` — code/config surfaces.** D13 adds adversary Rust/agent/test/decision/paper files and modifies role, SSB, grounding mode, verification result, config and module export surfaces. (`D13:282-298`)
- **D003-D13-E031 `[E]` — non-goals/lifecycle.** D14 probing is separate; a multi-adversary ensemble and critique caching are deferred, so the first version performs one fresh probe per claim. (`D13:300-305`)
- **D003-D13-E032 `[E]` — next dependency.** D14 reuses this infrastructure and D23 later gates D14. (`D13:307-310`)

### D-16_unicr_conformal_refusal.md — 31 atoms

- **D003-D16-E001 `[E]` — capability/claim.** D16 implements a Mode-E refusal controller presented as a distribution-free formal guarantee that refusal risk remains within user epsilon under distribution shift. (`D16:1-8`)
- **D003-D16-E002 `[E]` — prerequisites/concurrency.** Estimated effort is 3–4 sessions; D9 frozen-surface enforcement and D10 Mode-E slot are prerequisites; D17 and D22 are declared parallel-safe. (`D16:9-14`)
- **D003-D16-E003 `[E]` — risk definition/default.** The calibration set records claim/truth-availability/confidence; the plan defines empirical risk as refusing when truth exists plus accepting when truth does not, with default `epsilon=0.1`. (`D16:18-24`)
- **D003-D16-E004 `[E]` — refusal lifecycle.** Tau updates nightly under L3 control, and a claim is refused when a GG-signal composite confidence falls below tau. (`D16:25-28`)
- **D003-D16-E005 `[E]` — foundations.** D16 consumes D3 `IntrinsicSignals`, D9 locked keys, and D10 `GroundingMode::Refuse`. (`D16:33-37`)
- **D003-D16-E006 `[E]` — research gate.** UniCR, Uncertainty Outside Parametric Knowledge, and Rewarding Doubt interfaces must be ingested. (`D16:47-55`)
- **D003-D16-E007 `[E]` — tuple schema.** `CalibrationTuple` stores `claim_id`, `ground_truth_available`, `model_confidence`, `claim_type`, `claim_outcome`, and `recorded_at`; outcomes are `Verified`, `Falsified`, `Refused`, `NoCheck`. (`D16:57-65`)
- **D003-D16-E008 `[E]` — corpus schema/lifecycle.** `CalibrationCorpus` contains tuple vector and storage path; `load` walks `path/*.toml`, `append` writes `{claim_id}.toml`, and `size` reports tuple count. (`D16:65-70`)
- **D003-D16-E009 `[E]` — tau computation.** `compute_tau` evaluates `r(tau)=P(refuse|truth_exists)+P(accept|truth_doesnt_exist)`, selects the minimal candidate satisfying `r<=epsilon` by binary search over a `0.01` grid in `[0,1]`, and returns `InsufficientData` below 500 tuples. (`D16:72-81`)
- **D003-D16-E010 `[E]` — updater schema/action.** `TauUpdater` holds storage path, epsilon, and current tau; nightly update loads the corpus, recomputes tau, writes `.elai_cc/calibration/tau.txt` with timestamp, and emits `CalibrationUpdate`. (`D16:83-92`)
- **D003-D16-E011 `[E]` — permission/integrity.** `TauUpdater`, `compute_tau`, weights/config, and `tau.txt` are L3-protected; only the updater may write the threshold, preventing agent-side bias. (`D16:83-92,133-138,180-187`)
- **D003-D16-E012 `[E]` — confidence function.** `refusal_decision` computes `w1*mean_logprob + w2*(-entropy_max) + w3*novelty` using locked TOML weights, and returns true exactly when the composite is below tau. (`D16:94-99`)
- **D003-D16-E013 `[E]` — routing integration.** D10’s `intrinsic_signals_below_threshold` delegates to `refusal_decision`; `ctx.unicr_tau` loads from `tau.txt` at startup. (`D16:101-105`)
- **D003-D16-E014 `[E]` — data growth.** After each claim resolution, `task_runner` appends a tuple, derives truth availability from checker output, records decision-time composite confidence, and keeps the hook L1A/data-only with no LLM judgment. (`D16:107-111`)
- **D003-D16-E015 `[E]` — status CLI.** `elai calibration status` prints corpus size, current tau, update timestamp, epsilon, and empirical refusal rate over the last N sessions; it is informational and always exits `0`. (`D16:113-117`)
- **D003-D16-E016 `[E]` — update CLI/authorization.** `elai calibration update` manually invokes nightly logic; CI invocation requires commit tag `L3_OVERRIDE:calibration_update`, while development invocation is unrestricted. (`D16:119-124`)
- **D003-D16-E017 `[E]` — refusal event.** `RefusalDecision` stores claim ID, composite confidence, tau, and Boolean refused. (`D16:126-128`)
- **D003-D16-E018 `[E]` — update event/version.** `CalibrationUpdate` stores tuple count, new/old tau, epsilon and audit ID; both events round-trip under schema `61→62`. (`D16:126-131`)
- **D003-D16-E019 `[E]` — config defaults.** `[conformal]` defaults: `epsilon=0.1`, `min_corpus_size=500`, weights `{mean_logprob=0.4, entropy=-0.3, novelty=0.3}`, tau path `.elai_cc/calibration/tau.txt`, corpus path `.elai_cc/calibration/corpus/`. (`D16:133-138`)
- **D003-D16-E020 `[E]` — unit/idempotency evidence.** Tests cover known-split tau, insufficient data, at least eight threshold cases, and identical-corpus nightly-update idempotency. (`D16:140-145`)
- **D003-D16-E021 `[E]` — integration evidence.** A synthesized 600-tuple corpus must produce tau and an empirical refusal rate asserted within `[tau-epsilon, tau+epsilon]`. (`D16:146-150`)
- **D003-D16-E022 `[E]` — decisions.** The decision doc records epsilon `0.1`, minimum `500` based on the cited finite-sample bound, and why composite GG evidence is preferred to raw mean log probability. (`D16:152-159`)
- **D003-D16-E023 `[E]` — reachability.** D16 is reachable from the Mode-E decision point, a nightly CI/cron updater, and status/update CLI surfaces. (`D16:161-179`)
- **D003-D16-E024 `[E]` — fail-closed constraints.** No small-sample approximation is allowed, every update must journal, refusal must use the weighted composite, and direct writes to `tau.txt` are forbidden. (`D16:180-187`)
- **D003-D16-E025 `[E]` — observable proof.** A 600-tuple nightly update must emit `CalibrationUpdate`, persist tau and at least 600 tuple files, and cause a later low-confidence cloud claim to emit `RefusalDecision(refused=true)`. (`D16:188-193`)
- **D003-D16-E026 `[E]` — deterministic done probes.** The plan checks schema/event/type/config/lock symbols, build/tests, and three paper interfaces. (`D16:195-210`)
- **D003-D16-E027 `[E]` — acceptance gate.** Done requires schema 62, two event round-trips, corpus load/append, exact/insufficient-data/idempotency tests, lock registration, config load, at least twelve tests, and zero new workspace errors. (`D16:212-224`)
- **D003-D16-E028 `[E]` — cloud branches.** Low confidence must route to Mode E and journal refusal; high confidence must choose another mode with no refusal row in that branch; manual update with override must journal and update the file. (`D16:225-230`)
- **D003-D16-E029 `[E]` — documentary/surface gates.** Three paper interfaces and the decision doc are required; the code adds calibration and CLI modules/tests/docs and modifies SSB, route selection, task runner, config, locked-key registry, CLI commands, and exports. (`D16:231-252`)
- **D003-D16-E030 `[E]` — non-goals.** Distribution-shift validation is D22, generic bootstrap is deferred to D30, and per-family/multi-domain tau is deferred unless D30 establishes need. (`D16:254-259`)
- **D003-D16-E031 `[E]` — downstream.** D17 consumes refusal events; D22 validates shifted behavior. (`D16:261-264`)

### D-17_honest_refusal_sla.md — 24 atoms

- **D003-D17-E001 `[E]` — metric/capability.** D17 computes per-session `refusal_rate=refusals/total_claims`; below `5%` means likely confabulation, above `50%` means checker misconfiguration or an overly hard task suite. (`D17:1-7`)
- **D003-D17-E002 `[E]` — prerequisites/concurrency.** Estimated effort is one session; D16 refusal events are required, and D18 is declared parallel-safe. (`D17:8-10`)
- **D003-D17-E003 `[E]` — default versus research uncertainty.** The design uses broad inclusive healthy band `5–50%` while treating `10–30%` as an unvalidated guess to be tuned from actual session statistics. (`D17:12-20,87-92`)
- **D003-D17-E004 `[E]` — foundations/UI dependency.** It aggregates D16 `RefusalDecision` rows through `SsbViewer`; D27 later renders the gauge. (`D17:22-25`)
- **D003-D17-E005 `[E]` — monitor schema.** `RefusalRateMonitor` stores a minute window, minimum/maximum healthy rates, and SSB DB path. (`D17:34-39`)
- **D003-D17-E006 `[E]` — snapshot schema.** `RefusalRateSnapshot` stores session ID, window start, refusal and total counts, rate, and `HealthBand`; the band is `Below {rate,min}`, `Healthy`, or `Above {rate,max}`. (`D17:40-44`)
- **D003-D17-E007 `[E]` — query/workflow.** `snapshot` queries `RefusalDecision` rows for numerator and `MODE_SELECTED` rows for denominator, computes rate and band, and emits `RefusalSlaBreach` for either nonhealthy band. (`D17:46-52`)
- **D003-D17-E008 `[E]` — alert integration.** `alert_on_breach` emits the existing B13 Bubble-Up pattern with `AlertKind::RefusalRateOutOfBand` only when the band is not healthy. (`D17:54-57`)
- **D003-D17-E009 `[E]` — scheduling.** Task runner/orchestration invokes monitoring periodically; the prose says every N minutes with 60 default, while config separately defaults the check interval to 30 minutes and the window to 60. (`D17:59-62,69-72`)
- **D003-D17-E010 `[E]` — event/version.** Schema `62→63`; `RefusalSlaBreach` contains session ID, rate, healthy min/max, band, and total claims and needs round-trip proof. (`D17:64-67`)
- **D003-D17-E011 `[E]` — config/default/security.** L3-locked `[refusal_sla]` defaults are `enabled=true`, `window_size_minutes=60`, `healthy_min=0.05`, `healthy_max=0.50`, and `check_interval_minutes=30`. (`D17:69-72`)
- **D003-D17-E012 `[E]` — unit evidence.** Tests classify `0.02` below, `0.30` healthy, `0.60` above, verify synthetic SSB aggregation, and ensure healthy snapshots do not alert. (`D17:74-78`)
- **D003-D17-E013 `[E]` — integration evidence.** With 100 selected claims, 2 refusals alerts below, 60 alerts above, and 20 is healthy/no alert. (`D17:80-85`)
- **D003-D17-E014 `[E]` — decisions.** The plan records why the broad band is provisional, why scope is per session rather than global, and why breaches Bubble-Up for operator action instead of remaining silent. (`D17:87-94`)
- **D003-D17-E015 `[E]` — reachability.** D17 consumes D16, D10, SSB viewer/writer and B13 alerts; it is reached from periodic task-runner checks and D27 dashboard gauge. (`D17:96-111`)
- **D003-D17-E016 `[E]` — zero-denominator recovery.** `total_claims=0` is defined as `Healthy`, avoiding division by zero and suppressing an alert. (`D17:112-114`)
- **D003-D17-E017 `[E]` — idempotency.** Duplicate breach alerts in the same window must be suppressed using a window-end key. (`D17:115-116`)
- **D003-D17-E018 `[E]` — config integrity.** Thresholds must be loaded from TOML, not embedded in monitor code. (`D17:117`)
- **D003-D17-E019 `[E]` — observable proof.** A 2% synthetic session yields exactly one below-band breach plus Bubble-Up; D27’s gauge should be red. (`D17:118-121`)
- **D003-D17-E020 `[E]` — deterministic done probes.** Schema/event/monitor/config searches, workspace build and scoped tests are the deterministic checks. (`D17:123-132`)
- **D003-D17-E021 `[E]` — acceptance gate.** Done requires schema 63, event round-trip, correct aggregation/bands, Bubble-Up wiring, config loading, at least six tests, zero new errors, a 2% cloud breach/alert, and no event inside the healthy band. (`D17:134-149`)
- **D003-D17-E022 `[E]` — surfaces.** D17 adds monitor, integration test and decision doc; modifies SSB, task runner, config and module exports. (`D17:151-160`)
- **D003-D17-E023 `[E]` — non-goals.** Dashboard rendering, per-family band tuning, and long-term trend reporting are excluded; the latter two are explicitly deferred. (`D17:162-166`)
- **D003-D17-E024 `[E]` — next.** D18 completes Wave 2 after D17. (`D17:168-170`)

### D-19_prmbench_adversarial_gaming.md — 24 atoms

- **D003-D19-E001 `[E]` — capability.** D19 evaluates D2’s PPM on PRMBench Deception Resistance and Circular Logic, reports per category, and frames the result as a conservative promotion gate. (`D19:1-8`)
- **D003-D19-E002 `[E]` — prerequisites/compute/concurrency.** Estimated effort is two sessions; D2 and D3 are prerequisites; it needs the remote sidecar but not RunPod and is parallel-safe with D20–D25. (`D19:9-12`)
- **D003-D19-E003 `[E]` — baseline context.** The context cites 60–65% for top PRMs in the two weak categories and distinguishes discriminative, LLM-judge, and ThinkPRM evidence. (`D19:14-22`)
- **D003-D19-E004 `[E]` — failure decision.** Below-baseline evidence permits either accepting the limitation under D3 consensus or deferring promotion until D28 adversarial farming improves the PPM. (`D19:24-28`)
- **D003-D19-E005 `[E]` — research gate.** `prm-lessons-2501.07301-interface.md` and `thinkprm-2504.16828-interface.md` must be ingested. (`D19:41-47`)
- **D003-D19-E006 `[E]` — fixture lifecycle/compliance.** The public HuggingFace dataset license must be checked; fixtures are filtered to the two initial categories and stored as category/task JSON. (`D19:49-58`)
- **D003-D19-E007 `[E]` — fixture schema/volume.** Each fixture stores problem, step history, candidate step, and correct/incorrect label; bootstrap volume is at least 100 per category, 200 total. (`D19:56-59`)
- **D003-D19-E008 `[E]` — runner schema.** `PRMBenchRunner` owns PPM plus fixture/output paths; `PRMBenchReport` stores category, counts, false positives/negatives, accuracy/precision/recall, baseline, and `GatingVerdict`. (`D19:61-68`)
- **D003-D19-E009 `[E]` — verdict schema.** `GatingVerdict` is `Pass`, `FailBelowBaseline`, or `InsufficientData`. (`D19:69`)
- **D003-D19-E010 `[E]` — runner workflow.** `run_category` walks fixtures, calls `score_step`, compares ground truth, accumulates counters and metrics, then compares against a configured baseline. (`D19:70-75`)
- **D003-D19-E011 `[E]` — CLI behavior.** `elai bench prmbench` evaluates all categories and exits `0` on pass, `1` on below-baseline failure. (`D19:77-80`)
- **D003-D19-E012 `[E]` — shared benchmark events.** Schema `64→65` adds reusable `BenchmarkStarted {name,fixture_count,audit_id}` and `BenchmarkResult {name,accuracy,baseline,verdict,audit_id}` with round-trip tests for D19–D25. (`D19:82-86`)
- **D003-D19-E013 `[E]` — CI integration.** `bench-prmbench.yml` runs on push, PR and weekly cron, boots the remote PPM when credentials allow, invokes the CLI, and publishes a report artifact. (`D19:88-92`)
- **D003-D19-E014 `[E]` — nonblocking failure.** Until PPM becomes production-gating, `FailBelowBaseline` warns but does not block merge because D3 treats PPM as one consensus signal. (`D19:93-94,96-103`)
- **D003-D19-E015 `[E]` — configurable threshold.** The 35% threshold must come from TOML or CLI arguments rather than be hardcoded. (`D19:120-123`)
- **D003-D19-E016 `[E]` — timeout fallback.** A PPM timeout marks the affected task `InsufficientData` and the category run continues. (`D19:120-121`)
- **D003-D19-E017 `[E]` — evidence integrity.** Every report must expose its baseline and verdict; silent pass is forbidden. (`D19:122`)
- **D003-D19-E018 `[E]` — observable proof.** A category CLI run produces one start row, one result row, and `.elai_cc/benchmarks/prmbench/reports/{date}-{category}.json`. (`D19:124-127`)
- **D003-D19-E019 `[E]` — deterministic done probes.** Checks cover schema/events/runner, at least 200 fixtures, CI workflow, build/tests, and both paper interfaces. (`D19:129-142`)
- **D003-D19-E020 `[E]` — acceptance gate.** Done requires schema/event round-trips, compiling runner/CLI/workflow, 200+ fixtures, at least six tests, zero new errors, persisted reports, paper interfaces and the decision doc. (`D19:144-160`)
- **D003-D19-E021 `[E]` — numeric cloud gates.** Deception Resistance passes at `>=35%`; Circular Logic passes at `>=30%`; lower results are reported as informational `FailBelowBaseline`. (`D19:154-158`)
- **D003-D19-E022 `[E]` — surfaces.** D19 adds runner, CLI, 200+ fixtures, CI workflow, decision and paper docs; modifies SSB, CLI command routing and exports. (`D19:162-175`)
- **D003-D19-E023 `[E]` — non-goals.** Other PRMBench categories, ThinkPRM integration, and the D28 adversarial retraining loop are deferred/excluded. (`D19:177-181`)
- **D003-D19-E024 `[E]` — next.** D20 follows. (`D19:183-185`)

### D-25_determinism_archive_tamper_suite.md — 23 atoms

- **D003-D25-E001 `[E]` — capability/gate.** D25 defines ten archive/determinism scenarios; applicable tamper must be caught within five seconds, a clean repository exits `0`, and reverting tamper restores clean verification. (`D25:1-8`)
- **D003-D25-E002 `[E]` — dependencies/concurrency.** D8 archive verification and C27 `DeterminismDrift` are prerequisites; D25 is parallel-safe with D19–D24. (`D25:6-8`)
- **D003-D25-E003 `[E]` — scenario 1.** Deleting a middle `ArchiveEntry` must produce `BrokenParent` on the following entry. (`D25:15-16`)
- **D003-D25-E004 `[E]` — scenario 2.** Replacing `parent_hash` with a nonexistent hash must produce `BrokenParent`. (`D25:17`)
- **D003-D25-E005 `[E]` — scenario 3.** Editing `template_body` without recomputing `self_hash` must produce `HashMismatch`. (`D25:18`)
- **D003-D25-E006 `[E]` — scenario 4.** Reusing one `self_hash` for two entries must produce `DuplicateHash`. (`D25:19`)
- **D003-D25-E007 `[E]` — scenario 5.** A malformed JSON line must produce `MalformedJson`. (`D25:20`)
- **D003-D25-E008 `[E]` — scenario 6.** Two `elai /task` runs with the same `ELAI_SEED` but divergent `action_type` sequences must emit `DeterminismDrift`. (`D25:21-22`)
- **D003-D25-E009 `[E]` — scenario 7/control.** Same-seed identical action sequences must emit zero `DeterminismDrift` rows. (`D25:23`)
- **D003-D25-E010 `[E]` — scenario 8.** Rewriting Blake3 content-hash bytes as SHA-256 on disk must produce `HashMismatch`. (`D25:24-25`)
- **D003-D25-E011 `[E]` — scenario 9/recovery.** Restoring a tampered archive from backup must make verification exit `0` again, proving recovery/idempotency. (`D25:26,48-52`)
- **D003-D25-E012 `[E]` — scenario 10/full collection.** A two-line tamper at lines 3 and 7 must report two errors when verification runs in full-collect mode, exercising continue versus fail-fast semantics. (`D25:27,48-53`)
- **D003-D25-E013 `[E]` — fixture layout.** `.elai_cc/benchmarks/tamper/scenarios/` contains ten directories, each with setup script and expected verdict; scripts generate clean state, alter it, and invoke verification. (`D25:34-38`)
- **D003-D25-E014 `[E]` — runner workflow.** `TamperRunner` executes setup and `elai archive verify`, records exit/error types, and compares them to the expected verdict. (`D25:40-43`)
- **D003-D25-E015 `[E]` — CLI behavior.** `elai bench tamper` runs all ten and exits `0` only when every scenario matches expectation. (`D25:45-46`)
- **D003-D25-E016 `[E]` — produced/consumed interfaces.** D25 produces `TamperRunner`, `TamperReport`, ten scripts and CLI; consumes D8 CLI, C27 drift detection and `SsbWriter`; it is reachable from CLI and CI. (`D25:55-66`)
- **D003-D25-E017 `[E]` — constraints.** The contract forbids missing a scenario, false-positive failure on clean state, or failure to recover after revert. (`D25:67-70`)
- **D003-D25-E018 `[E]` — observability.** One full run expects ten `BenchmarkStarted`, ten `BenchmarkResult`, and a report under `.elai_cc/benchmarks/tamper/reports/`. (`D25:71-74`)
- **D003-D25-E019 `[E]` — deterministic check.** Static checks require exactly ten scenario entries and a clean workspace compile. (`D25:76-82`)
- **D003-D25-E020 `[E]` — acceptance gate.** Done requires all directories, compiling runner, all-match exit semantics, zero build errors, every applicable tamper within five seconds, clean exit `0`, recovery after revert, and the decision doc. (`D25:84-93`)
- **D003-D25-E021 `[E]` — surfaces.** D25 adds runner, CLI, `{1..10}/setup.sh`, `expected.toml`, and decision doc and modifies command routing. (`D25:95-101`)
- **D003-D25-E022 `[E]` — non-goals.** Git cryptographic signing and multiparty signing are deferred. (`D25:103-107`)
- **D003-D25-E023 `[E]` — lifecycle.** D25 declares Wave 3 complete and hands off all-feature dashboard work to D26–D27. (`D25:109-111`)

### D-30_calibration_bootstrap_corpus.md — 24 atoms

- **D003-D30-E001 `[E]` — status/objective.** D30 is explicitly `DEFERRED`; it spikes a generic-corpus warm start because D16 needs 500 calibration tuples and early sessions cannot supply them. (`D30:1-10`)
- **D003-D30-E002 `[E]` — dependencies/concurrency.** Unblock requires D16 and a specific public HuggingFace corpus; D30 is parallel-safe with D28 and D29. (`D30:7-11`)
- **D003-D30-E003 `[E]` — hypothesis/lifecycle.** Candidate bootstrap data begins with generic QA/factual calibration, then local per-project data refines the threshold as sessions accumulate. (`D30:13-18`)
- **D003-D30-E004 `[E]` — risk/fallback.** Distribution mismatch between generic QA and ELAI code/reasoning may miscalibrate tau; nightly D16 updates limit bootstrap to starting-point status. (`D30:20-23`)
- **D003-D30-E005 `[E]` — corpus candidates.** Candidate sources are TruthfulQA, verified MMLU subsets, and HaluEval; the chosen source must publicly expose confidence labels and truth-availability flags and then receive paper ingestion. (`D30:32-40`)
- **D003-D30-E006 `[E]` — selection/data surfaces.** Unblocking adds `selection.md`, downloads `.elai_cc/calibration/bootstrap/{corpus}/`, and converts source records to D16 `CalibrationTuple` via a shim. (`D30:42-49`)
- **D003-D30-E007 `[E]` — bootstrap persistence.** D16 `compute_tau` runs on the bootstrap corpus and persists `.elai_cc/calibration/tau_bootstrap.txt`. (`D30:50-52`)
- **D003-D30-E008 `[E]` — switchover lifecycle.** Startup uses bootstrap tau while local corpus size is `<500` and switches to local-corpus tau once size is `>=500`. (`D30:53-54`)
- **D003-D30-E009 `[E]` — validation workflow.** D22’s runner tests bootstrap tau on synthetic ELAI claims and requires held-out empirical refusal behavior within epsilon; outside epsilon produces a distribution-mismatch Path-B report. (`D30:56-60`)
- **D003-D30-E010 `[E]` — closure alternatives.** Path A productizes bootstrap selection when it improves cold start; Path B closes with a concrete shifted-distribution blocker greater than epsilon. (`D30:61-66`)
- **D003-D30-E011 `[E]` — decision lifecycle.** `D-30-decisions.md` remains deferred and is drafted only at unblock. (`D30:68-69`)
- **D003-D30-E012 `[E]` — implemented symbol inventory.** The wiring contract names `warm_start_tau` at `calibration_bootstrap.rs:346`, `load_corpus` at `:292`, `CalibrationBootstrapCorpus` at `:189`, conversion at `:216`, `PublishedCalibrationRow` at `:148`, `BootstrapWarmStartOutcome` at `:239`, resampling at `:431`, CI helper at `:525`, and defaults at `:56/:65/:72`. (`D30:71-80`)
- **D003-D30-E013 `[E]` — event/runtime inventory.** It names `CalibrationBootstrapCompleted` plus payload at `ssb.rs:6746`, and `CalibrationBootstrapMode` at `runtime_config.rs:487`, gated by `ELAI_CALIBRATION_BOOTSTRAP_ENABLE`. (`D30:80-82`)
- **D003-D30-E014 `[E]` — deferred data legs.** Corpus download, `selection.md`, and `tau_bootstrap.txt` remain deferred because no corpus is selected. (`D30:82`)
- **D003-D30-E015 `[E]` — consumed interfaces.** Conversion feeds D16 `CalibrationCorpus`, warm start calls D16 `compute_tau`, mode comes from `CalibrationBootstrapMode::from_env` at runtime config line 499, D22 is deferred validation, and SSB writer journals completion. (`D30:83-88`)
- **D003-D30-E016 `[E]` — live reachability/default.** The environment-enabled path is wired at `task_runner.rs:5980` with an off-mode early-exit at `:16213`; default is wire-dark `Off`. (`D30:89-92`)
- **D003-D30-E017 `[E]` — safety constraints.** Warm start must enforce `DEFAULT_BOOTSTRAP_MIN_CORPUS_SIZE`, tolerate empty corpus without panic, always emit completion after a run, and use bootstrap tau below `PRODUCTION_FLOOR_TUPLES=500`. (`D30:93-97`)
- **D003-D30-E018 `[E]` — observable proof contract.** With enable flag and a present corpus, task execution should emit one completion row containing corpus slug and warm-start tau and write `tau_bootstrap.txt`. (`D30:97-100`)
- **D003-D30-E019 `[E]` — partial implementation status.** The plan explicitly says data/artifact/D22 legs are pending but the Rust dispatch path at the two task-runner anchors “IS wired and live.” (`D30:101-103`)
- **D003-D30-E020 `[E]` — deterministic-check status.** The entire deterministic-check section is `DEFERRED`; it supplies no executable check despite the named live Rust path. (`D30:105-107`)
- **D003-D30-E021 `[E]` — acceptance/unblock gate.** All acceptance boxes remain open; completion requires final corpus choice, chosen-corpus paper interface, persisted bootstrap tau, D22 Path A within epsilon or a concrete Path B mismatch, and decision doc. (`D30:109-117`)
- **D003-D30-E022 `[E]` — anticipated surfaces.** Remaining anticipated writes are corpus directory, selection document, bootstrap tau, D16 startup fallback modification, and decision doc. (`D30:119-126`)
- **D003-D30-E023 `[E]` — non-goals.** Custom corpus authoring and multi-corpus blending are deferred beyond D30. (`D30:128-132`)
- **D003-D30-E024 `[E]` — terminal sequencing.** The D plan completes only after D30 unblocks; D28–D30 otherwise operate independently and are prioritized by GPU availability and research value. (`D30:134-138`)

### D-003 cross-source interactions (namespaced D-003/<INT-*>)

- **D-003/D003-INT-E001 `[E]` — D2→D3 schema transaction.** D2 deliberately scaffolds PPM events and D3 atomically promotes all six D2/D3 event families under schema 49; neither plan supports a separately versioned D2 landing. (`D2:152-160`; `D3:137-150,207-220`)
- **D-003/D003-INT-E002 `[E]` — D2→D3 availability semantics.** D3 invokes PPM only for `VerifierKind::Prm` with enable flag and a healthy client; unavailable required PPM becomes an explicit non-pass/inconclusive path rather than auto-pass. (`D2:118-120,303-305`; `D3:124-135,304-310`)
- **D-003/D003-INT-E003 `[E]` — D3 fan-out.** D3’s pass event feeds D5, disagreement feeds D4, adversary extends its signal set in D13, and its intrinsic signals become the confidence substrate in D16. (`D3:207-232,331-341`; `D13:137-142`; `D16:33-37,171-175`)
- **D-003/D003-INT-E004 `[E]` — cross-family rule consistency.** D2’s PPM may be separate-base but D13 explicitly treats Qwen Coder and Qwen Math as the same family, so the PPM cannot serve as D13’s adversary. (`D2:26-32,184-189`; `D13:33-37,156-160`)
- **D-003/D003-INT-E005 `[E]` — D13→D16 degradation.** Mode-B inability returns a Mode-E refusal, but the actual conformal refusal mechanism and tau live in D16. (`D13:30-37,125-135`; `D16:94-105,176-179`)
- **D-003/D003-INT-E006 `[E]` — D16→D17 observability chain.** D16 emits per-claim `RefusalDecision`; D17 joins those to D10 `MODE_SELECTED` for per-session/window SLA and raises `RefusalSlaBreach` plus Bubble-Up. (`D16:126-131`; `D17:46-72,96-108`)
- **D-003/D003-INT-E007 `[E]` — D2→D19 benchmark chain.** D19 calls D2 `PpmClient` across 200 adversarial fixtures and converts score decisions into category metrics and reusable benchmark events. (`D2:83-100`; `D19:49-86`)
- **D-003/D003-INT-E008 `[E]` — D19→D25 shared observability.** D19 declares `BenchmarkStarted`/`BenchmarkResult` reusable through D25, and D25 expects ten of each without declaring another event/version. (`D19:82-86`; `D25:71-74`)
- **D-003/D003-INT-E009 `[E]` — D8/C27→D25 integrity join.** D25 simultaneously tests archive-chain error variants and same-seed action-sequence drift, then validates recovery to clean state. (`D25:1-27,55-74`)
- **D-003/D003-INT-E010 `[E]` — D16/D22→D30 cold start.** D30 converts published rows to D16 corpus, calls `compute_tau`, uses bootstrap only below 500 local tuples, and delegates shifted validation to D22. (`D30:42-66,71-103`)
- **D-003/D003-INT-E011 `[E]` — schema sequence.** Assigned plans explicitly claim version transitions 48→49 (D3), 58→59 (D13), 61→62 (D16), 62→63 (D17), and 64→65 (D19); omitted intervening versions belong to unassigned D plans and were not inspected. (`D3:137-140`; `D13:144-149`; `D16:126-131`; `D17:64-67`; `D19:82-86`)
- **D-003/D003-INT-E012 `[E]` — operator surfaces.** The chunk adds `elai ppm health`, `elai calibration status`, `elai calibration update`, `elai bench prmbench`, and `elai bench tamper`, with structured exit/status semantics rather than dashboard-only evidence. (`D2:145-150`; `D16:113-124`; `D19:77-80`; `D25:45-46`)
- **D-003/D003-INT-I001 `[I]` — end-to-end verification path.** The assigned-source composition is: opt-in PPM score → typed multi-signal result → optional cross-family adversary contribution → consensus-gated outcome; low-confidence or unavailable-independent-verifier branches reach calibrated refusal, whose rate is separately monitored. This is a synthesis, not a declared transaction boundary. (`D2:197-219`; `D3:124-150`; `D13:125-142`; `D16:94-105`; `D17:46-57`)
- **D-003/D003-INT-I002 `[I]` — evidence-plane composition.** SSB supplies per-step score/unavailability, per-candidate consensus/disagreement, per-claim adversary/refusal, per-window SLA, and per-benchmark integrity rows; together these form a traceable evidence plane, but no assigned source specifies a single correlation key spanning all record types. (`D2:152-160`; `D3:137-150`; `D13:144-149`; `D16:126-131`; `D17:64-67`; `D19:82-86`)

### D-003 ambiguity/conflict register (namespaced D-003/<ISSUE-*>)

- **D-003/D003-ISSUE-C001 `[C]` — PPM family wording.** D2’s objective calls the PPM a “different model family,” but the same source calls it Qwen-family/same-family-different-base and D13 explicitly rejects Qwen Coder + Qwen Math as same-family. Treat D2 as heterogeneous specialization/base, not a valid D13 cross-family verifier. (`D2:3-8,26-32,184-189`; `D13:33-37,156-160`)
- **D-003/D003-ISSUE-C002 `[C]` — D2 pre-D3 SSB expectation.** D2’s observable contract expects zero SSB rows until D3 wires consensus, while its offline cloud acceptance expects `PpmUnavailable` to be emitted. The intended scaffold-versus-runtime boundary is not resolved. (`D2:229-232,286-287,303-305`)
- **D-003/D003-ISSUE-A001 `[A]` — PPM authentication/privacy.** `[runtime.ppm]` is said to reuse per-role endpoint/auth, but the stanza lists no auth key and the plan defines no secret handling, request-data minimization, retention, or cloud privacy policy. (`D2:41-46,112-120`)
- **D-003/D003-ISSUE-A002 `[A]` — score parsing.** The response body shape and extraction rule for turning Anthropic-protocol output into one bounded float/confidence pair are not specified beyond `ParseError`. (`D2:85-100,102-107`)
- **D-003/D003-ISSUE-A003 `[A]` — retry determinism.** Backoff jitter is required but its seed, schedule and observability are unstated; only the attempt cap is deterministic. (`D2:99-100,193-195`)
- **D-003/D003-ISSUE-C003 `[C]` — default consensus versus deceptive defense.** D3’s default `AtLeastTwo` permits PPM+intrinsic to pass despite deterministic failure; the source itself corrects the test to code-specific `RequireAll`. Therefore “deterministic fail always blocks promotion” is false unless rule selection requires it. (`D3:100-108,170-184`)
- **D-003/D003-ISSUE-A004 `[A]` — required-signal semantics.** D3 says missing a “required” signal is inconclusive but does not define requiredness for `AtLeastTwo/Three/N`; only `RequireAll(kinds)` names required kinds explicitly. (`D3:100-108,233-239`)
- **D-003/D003-ISSUE-A005 `[A]` — intrinsic pass predicate.** Config exposes logprob, entropy and novelty thresholds, but the pass-count algorithm only names `mean_logprob >= intrinsic_threshold`; combination of all three fields is unspecified. (`D3:89-90,100-105,152-155`)
- **D-003/D003-ISSUE-A006 `[A]` — shuffled-context reproducibility.** D3 requires shuffled prior steps and a `SeededClock` but does not name a shuffle RNG/seed, so reproducibility of cross-context inputs is not closed. (`D3:110-122,222-227`)
- **D-003/D003-ISSUE-A007 `[A]` — disagreement verdict.** An integration scenario says cross-context disagreement yields `Inconclusive`, while generic count rules could yield pass or reject depending other signals; the governing rule for that scenario is not named. (`D3:100-108,170-186`)
- **D-003/D003-ISSUE-C004 `[C]` — ModelFamily predicate wording.** D13 says `cross_family_check` ensures different enum variants, yet its required test says two distinct variants `Qwen25Coder` and `Qwen25Math` must still compare same-family/false; the implementation needs an unstated higher-level family grouping. (`D13:80-85,156-160`)
- **D-003/D003-ISSUE-A008 `[A]` — adversary runtime failures.** Availability and same-family failures are mapped, but timeout, provider error, malformed JSON, citation validation failure, retry and cost-throttle mechanics are not mapped to claim state or Mode E. (`D13:111-135,174-183,207-213`)
- **D-003/D003-ISSUE-A009 `[A]` — adversary privacy/compliance.** Claims are sent to cloud Sonnet by default, but no consent, redaction, data residency, retention, PII, secret, or audit/compliance policy is defined. (`D13:33-37,99-109,164-177`)
- **D-003/D003-ISSUE-A010 `[A]` — auditor identity.** Context requires an auditor rather than executor to decide, but dispatch maps `CounterEvidence` directly to `ClaimResult` and does not name an auditor call/identity in that flow. (`D13:23-31,125-135`)
- **D-003/D003-ISSUE-C005 `[C]` — confidence weight sign/domain.** D16’s formula negates entropy and its configured entropy weight is already `-0.3`, producing a double-negative contribution if applied literally; mean log probability can also be negative while tau search is `[0,1]`. Normalization/sign convention is unstated. (`D16:72-99,133-138`)
- **D-003/D003-ISSUE-C006 `[C]` — risk versus refusal-rate assertion.** `compute_tau` bounds a two-term error risk, but integration asserts refusal rate lies around numeric tau; those are different quantities unless an unstated theorem/transformation equates them. (`D16:72-81,146-150`)
- **D-003/D003-ISSUE-A011 `[A]` — cold-start behavior.** D16 returns `InsufficientData` below 500 but does not define startup tau/refusal behavior in that state; D30 proposes the fallback but is deferred. (`D16:72-81,180-187,254-259`; `D30:1-10,50-54`)
- **D-003/D003-ISSUE-A012 `[A]` — tuple overwrite/idempotency.** Append writes `{claim_id}.toml`; repeated resolution of the same claim, collision/overwrite handling, atomic writes and corruption recovery are unspecified. (`D16:65-70,107-111`)
- **D-003/D003-ISSUE-C007 `[C]` — D17 schedule wording.** Periodic prose calls 60 minutes the default, while config sets a 60-minute window and a 30-minute check interval. The likely distinction is window versus cadence, but the phrase “every N minutes (60 default)” conflicts with the configured cadence. (`D17:59-72`)
- **D-003/D003-ISSUE-A013 `[A]` — SLA dedup key persistence.** Duplicate suppression uses a window-end key, but the snapshot/event schemas do not include window end or define persistence/restart behavior. (`D17:36-44,64-67,112-117`)
- **D-003/D003-ISSUE-A014 `[A]` — numerator/denominator alignment.** The plan does not state whether every `MODE_SELECTED` row can produce at most one `RefusalDecision`, how late rows/windows are aligned, or whether retried claims count once. (`D17:46-52`)
- **D-003/D003-ISSUE-C008 `[C]` — benchmark gate versus CI.** D19 calls the threshold a PPM promotion gate and CLI returns failure, yet CI explicitly warns and continues until an undefined production-gating transition. Current enforcement is informational, not a merge gate. (`D19:1-8,24-28,77-80,88-103`)
- **D-003/D003-ISSUE-A015 `[A]` — insufficient-data aggregation.** A timed-out task is called `InsufficientData`, but only report-level `GatingVerdict` is typed; minimum valid fixture count and how invalid tasks affect metrics/verdict are unspecified. (`D19:61-75,120-123`)
- **D-003/D003-ISSUE-A016 `[A]` — baseline config.** The threshold must come from TOML or args, but no TOML section, per-category keys, confidence interval, or precedence rule is defined. (`D19:74-80,120-123`)
- **D-003/D003-ISSUE-C009 `[C]` — “ten detected” wording.** D25 says all ten scenarios must be detected, but scenario 7 expects no drift and scenario 9 expects clean recovery; these are negative/recovery controls whose success is matching expectation, not detection. (`D25:15-27,45-46,67-74`)
- **D-003/D003-ISSUE-A017 `[A]` — five-second boundary.** The plan does not say whether setup, archive verify, drift comparison, or only detection latency is inside the five-second timer. (`D25:1-5,40-46,84-92`)
- **D-003/D003-ISSUE-A018 `[A]` — destructive fixture isolation.** Tamper scripts modify archives and restore backups, but temporary-copy isolation, crash cleanup and protection of a real project archive are not specified. (`D25:34-43,95-101`)
- **D-003/D003-ISSUE-A019 `[A]` — undeclared benchmark-event prerequisite.** D25 expects D19-defined benchmark events but lists only D8/C27 as prerequisites; behavior when D25 lands before schema 65 is unstated. (`D19:82-86`; `D25:6-8,55-74`)
- **D-003/D003-ISSUE-C010 `[C]` — D30 lifecycle status.** D30 is globally `DEFERRED` with all acceptance/checks deferred, yet its wiring contract claims concrete symbols and a Rust task-runner path are already wired/live. The source supports “partially wired, data/validation blocked,” not purely unimplemented or complete. (`D30:1-11,71-117`)
- **D-003/D003-ISSUE-A020 `[A]` — bootstrap reproducibility.** `bootstrap_resample` and a 90% CI helper are named as live symbols, but the plan gives no seed, iteration count, sampling policy or acceptance use. (`D30:71-80`)
- **D-003/D003-ISSUE-A021 `[A]` — bootstrap data governance.** Public availability is required, but dataset license compatibility, provenance record, download hash, update/version pin, privacy and deletion policy are unspecified. (`D30:32-49`)
- **D-003/D003-ISSUE-A022 `[A]` — switchover atomicity.** The `<500`/`>=500` rule is explicit, but concurrent corpus growth, startup caching, threshold-write atomicity and rollback are not. (`D30:50-54,89-103`)
- **D-003/D003-ISSUE-A023 `[A]` — no chunk-wide privacy feature.** Apart from local loopback endpoints, L3 locks, typed citations, dataset-license check and public-corpus requirement, none of the eight plans defines a dedicated PII, secrets, retention, telemetry export, user-consent, data residency or compliance lifecycle. (Coverage audit over `D2:1-343`; `D3:1-341`; `D13:1-310`; `D16:1-264`; `D17:1-170`; `D19:1-185`; `D25:1-111`; `D30:1-138`)
- **D-003/D003-ISSUE-A024 `[A]` — implementation status outside D30.** D2/D3/D13/D16/D17/D19/D25 contain prospective tasks and unchecked acceptance boxes but no explicit `Status:` line; this deep pass therefore does not independently claim `PLANNED`, shipped, wired, or done state for them. (`D2:279-309`; `D3:286-313`; `D13:258-280`; `D16:212-232`; `D17:134-149`; `D19:144-160`; `D25:84-93`)

### D-003 summary deltas (namespaced D-003/<DELTA-*>)

- **D-003/D003-DELTA-001 `[Δ~]` — D2 client kind.** The summary calls `PpmClient` a trait; D2 defines a concrete `struct PpmClient` plus `impl`, with `PpmAdapter` implementing `RuntimeAdapter`. (`D2:83-110`; summary `04-D...:46,524-528`)
- **D-003/D003-DELTA-002 `[Δ~]` — D2 family claim.** “Different family” must be qualified to Qwen same-family/different-base for D2; D13 does not accept it as a cross-family adversary. (`D2:26-32,184-189`; `D13:33-37,156-160`; summary `04-D...:42-50,580-583`)
- **D-003/D003-DELTA-003 `[Δ+]` — D2 prerequisites/config.** The summary omits C15, the disabled-by-default two-part activation, exact endpoint/timeout/retry/model defaults, verifier-only permission, single-GPU serialized fallback, three-stage health command, tunnel extension, error enum and fixed prompt. (`D2:9-16,81-150`)
- **D-003/D003-DELTA-004 `[Δ+]` — D2 events/schema/evidence.** Add exact `PpmScoreEmitted`/`PpmUnavailable` payloads, D3-owned schema-49 staging, retry tests, no-fallback bans, hardware endurance/free-memory gate, offline logging and documentary outputs. (`D2:152-235,279-309`)
- **D-003/D003-DELTA-005 `[Δ+]` — D3 complete schemas.** The summary’s five aggregate fields omit `verdict`, nested deterministic/intrinsic/cross-context records, all rule/signal/verdict enum values, thresholds, timeouts, concurrent two-context dispatch and exact event payloads. (`D3:80-168`)
- **D-003/D003-DELTA-006 `[Δ~]` — D3 deceptive gate.** A deterministic failure does not block under default `AtLeastTwo` when PPM and intrinsic both pass; D3 explicitly switches code claims to `RequireAll([Deterministic,Ppm])`. The summary’s unconditional deceptive-case rejection is too broad. (`D3:170-184`; summary `04-D...:53-63,589-590`)
- **D-003/D003-DELTA-007 `[Δ+]` — D3 operational contracts.** Add one result per candidate, explicit unavailable-signal behavior, disagreement as separate D4 input, L3-loaded rule, schema-49 migration warning, test volumes and the offline/cloud branches. (`D3:188-313`)
- **D-003/D003-DELTA-008 `[Δ~]` — D13 exact mode/events.** Exact route is `GroundingMode::FalsifiabilityCrossFamily`; exact Rust event variants are `AdversaryVerdict` and `CrossFamilyUnavailable`, not the summary’s uppercase display labels. (`D13:1-5,125-149`; summary `04-D...:195-210,542-552`)
- **D-003/D003-DELTA-009 `[Δ+]` — D13 typed evidence and states.** Add full model/counter-evidence/evidence/citation schemas, 2,048-byte evidence bound, JSON-only parser, three verdict-to-claim mappings, separate auditor constraint, 4,096-token locked config, source-tier validation and cost-throttling decision. (`D13:67-183`)
- **D-003/D003-DELTA-010 `[Δ+]` — D13 non-goals.** The summary omits single-fresh-probe scope, deferred ensembles/caching, exact tests, schema 59 and all acceptance/artifact gates. (`D13:185-305`)
- **D-003/D003-DELTA-011 `[Δ~]` — D16 tuple schema.** The persisted unit is keyed by `claim_id`, not raw `claim`, and also carries `claim_type`, `ClaimOutcome`, and timestamp; corpus records live under `.elai_cc/calibration/corpus/`. (`D16:57-70,133-138`; summary `04-D...:241-256,501-505`)
- **D-003/D003-DELTA-012 `[Δ+]` — D16 algorithm/defaults.** Add risk equation, minimal tau search on 0.01 grid, hard 500-record floor, exact weighted composite, config/paths, status/update CLI, L3 override rule, event payloads, schema 62 and idempotency requirement. (`D16:72-145`)
- **D-003/D003-DELTA-013 `[Δ~]` — D16 proof scope.** D16 itself tests a synthetic 600-tuple corpus; distribution-shift families are explicitly D22/out-of-scope. The summary conflates the D16 implementation smoke gate with D22 validation. (`D16:146-150,254-264`; summary `04-D...:253-256,601-603`)
- **D-003/D003-DELTA-014 `[Δ~]` — D17 event identifier.** The original uses `RefusalSlaBreach` plus `AlertKind::RefusalRateOutOfBand`, not `AUTO_FLAG`. (`D17:46-72`; summary `04-D...:258-269,542-552`)
- **D-003/D003-DELTA-015 `[Δ~]` — D17 window.** The original monitors a 60-minute window on a 30-minute configured cadence; it does not specify the summary’s “last 20 sessions” instrumentation. (`D17:36-72`; summary `04-D...:267-269,603-604`)
- **D-003/D003-DELTA-016 `[Δ+]` — D17 recovery/idempotency.** Add zero-claim-as-healthy, per-window alert deduplication, Bubble-Up operator notification, task-suite-too-hard upper-band cause, L3 configuration and exact test matrix. (`D17:36-121`)
- **D-003/D003-DELTA-017 `[Δ+]` — D19 corpus/runner.** Add license check, exact fixture schema, 100-per-category floor, complete report metrics/verdict enum, timeout-to-insufficient-data behavior, CLI exit codes, CI triggers/artifacts and event payloads. (`D19:49-127`)
- **D-003/D003-DELTA-018 `[Δ~]` — D19 enforcement.** The 35%/30% results currently warn in CI and do not block merge; production-gating activation is future/undefined. This qualifies the summary’s direct “gates PPM promotion” wording. (`D19:88-103,144-158`; summary `04-D...:285-296`)
- **D-003/D003-DELTA-019 `[Δ~]` — D19 baseline context.** The original cites 60–65% for top PRMs/~60% for Qwen context but deliberately uses 35% and 30% conservative product gates; “published baseline” is not the numeric acceptance threshold. (`D19:14-28,61-75,96-103,154-158`; summary `04-D...:288-296`)
- **D-003/D003-DELTA-020 `[Δ+]` — D25 scenarios.** The summary lists four of ten; add duplicate hash, malformed JSON, divergent same-seed sequence, identical-sequence control, recovery, and two-line full-collect cases with exact expected errors. (`D25:15-27`; summary `04-D...:360-376,611-612`)
- **D-003/D003-DELTA-021 `[Δ+]` — D25 runner/recovery.** Add directory/setup/expected layout, `TamperRunner`/report, all-match CLI exit, 10+10 benchmark rows, exact file surfaces, fail-fast-versus-continue purpose and deferred signing non-goals. (`D25:34-107`)
- **D-003/D003-DELTA-022 `[Δ~]` — D30 partial state.** Although globally deferred, the original now explicitly names implemented Rust symbols, environment-mode config, SSB event and live task-runner anchors; only corpus/data/artifact/D22 legs remain blocked. The summary’s blanket deferred portrayal is stale/incomplete. (`D30:71-117`; summary `04-D...:430-443,654-656`)
- **D-003/D003-DELTA-023 `[Δ+]` — D30 candidates/switchover.** Add TruthfulQA/MMLU/HaluEval candidates, required label properties, exact `<500` bootstrap to `>=500` local switch, wire-dark default, empty-corpus safety and completion event fields. (`D30:32-103`)
- **D-003/D003-DELTA-024 `[Δ~]` — D30 acceptance claim.** The assigned original does not require refined empirical risk to be no worse than bootstrap; it requires Path A cold-start improvement and within-epsilon D22 validation, or Path B concrete mismatch. The summary’s stronger “refined risk no worse” gate is unsupported by D30. (`D30:56-66,109-117`; summary `04-D...:438-443,615-616`)
- **D-003/D003-DELTA-025 `[Δ+]` — chunk-wide failure and evidence detail.** The deep plans add structured error enums, bounded retry, explicit inconclusive states, no-panic/zero-denominator handling, idempotent tau and alert behavior, CI warning semantics, artifact paths, schema transitions, test-count gates and negative searches that the summary did not inventory. (Atoms `D003-D2-E012`–`D003-D2-E039`, `D003-D3-E013`–`D003-D3-E035`, `D003-D16-E020`–`D003-D17-E021`, `D003-D19-E014`–`D003-D25-E020`)

---

## Ledger D-004 — source atoms

Source file: `docs/thefinalglance/feature-discovery/deep/DEEP-D-004.discovery.md`
Plans: D-4, D-7, D-11, D-15, D-18, D-20, D-22
Manifest: 7 files, 1755 lines, 83367 bytes

### D-4_tripwire_stack.md — 52 atoms

| ID | Ev. | Cat. | Δ | Explicit feature, behavior, or boundary | Anchor |
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

### D-7_cross_domain_canary.md — 44 atoms

| ID | Ev. | Cat. | Δ | Explicit feature, behavior, or boundary | Anchor |
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

### D-11_constructive_checker_registry.md — 61 atoms

| ID | Ev. | Cat. | Δ | Explicit feature, behavior, or boundary | Anchor |
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

### D-15_novelty_class_labeling.md — 38 atoms

| ID | Ev. | Cat. | Δ | Explicit feature, behavior, or boundary | Anchor |
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

### D-18_observable_effect_typed.md — 44 atoms

| ID | Ev. | Cat. | Δ | Explicit feature, behavior, or boundary | Anchor |
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

### D-20_arc_agi2_ood_probe.md — 36 atoms

| ID | Ev. | Cat. | Δ | Explicit feature, behavior, or boundary | Anchor |
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

### D-22_conformal_calibration_validation.md — 26 atoms

| ID | Ev. | Cat. | Δ | Explicit feature, behavior, or boundary | Anchor |
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

### D-004 cross-source interactions (namespaced D-004/<X-*>)

| Aggregate ID | Ev. | Interaction | Exact joined behavior | Anchors |
|---|---|---|---|---|
| D-004/D004-X-001 | E | D-7 → D-4 | Canary results become CrossDomain signal input for D-4’s Δρ/KL monitoring; D-7 explicitly says it cannot land in parallel with D-4, while D-4’s Next section names D-6-held-out baselines. | D7:L11-L13, L103-L110; D4:L86-L88, L323-L327 |
| D-004/D004-X-002 | I | candidate lifecycle | Intended order is D-7 pre-promotion veto, then promotion, then D-4 post-action monitoring and append-only revert. This is a synthesis of explicit hooks, not an atomic transaction defined by either plan. | D7:L119-L125; D4:L130-L142, L212-L214 |
| D-004/D004-X-003 | E | D-11 → D-15 | Mode-A CheckerResult is the declared classification input, and D-15 writes the novelty class to ClaimResult metadata plus SSB/queue rather than changing CheckerResult. | D11:L80-L97, L161-L165; D15:L73-L89 |
| D-004/D004-X-004 | A | D-11 ↔ D-15 | D-11’s verdict-only CheckerResult cannot supply the result embedding/primitives that D-15’s classifier needs; an unplanned result/candidate adapter is required. | D11:L80-L83; D15:L69-L81, L111-L116 |
| D-004/D004-X-005 | E | D-11 → D-18 | D-18 declares CheckerRegistry the per-effect runner and FeasibilityProber/Mode-D the entry path. | D18:L93-L102, L156-L163 |
| D-004/D004-X-006 | A | D-18 → D-15 | D-15 expects Mode-D to yield CheckerResult for classification, while D-18’s Mode-D upgrade yields Vec<ObservationResult>; no conversion or verdict aggregation is defined. | D15:L83-L87; D18:L79-L97 |
| D-004/D004-X-007 | E | D-15 → D-20 | ARC aggregates NoveltyClass and must preserve D-15’s human-only DiscoveryCandidate rule; the harness is an evidence consumer, not a publication path. | D15:L88-L110, L139-L141; D20:L60-L70, L113-L123 |
| D-004/D004-X-008 | E | D-16/D-17 → D-20/D-22 | D-20 consumes RefusalDecision and tests the 5%-50% SLA; D-22 separately calls refusal_decision against τ±ε on three distributions. | D20:L23-L31, L103-L108, L145-L146; D22:L39-L47, L63-L71 |
| D-004/D004-X-009 | C | benchmark events | D-20 reuses D-19 BenchmarkStarted/BenchmarkResult while declaring D-19 parallel-safe; D-22 expects the same generic events without naming their origin or schema. Landing/build order is incomplete. | D20:L10-L11, L84-L85; D22:L72-L75 |
| D-004/D004-X-010 | E | schema sequence | Assigned schema bumps are 49→50 (D-4), 52→53 (D-7), 56→57 (D-11), 60→61 (D-15), and 63→64 (D-18); D-20 has no bump and D-22 names no bump. The gaps belong to other planned sessions and make ordered landing material. | D4:L144-L150; D7:L103-L111; D11:L167-L172; D15:L103-L110; D18:L104-L112; D20:L84-L85 |
| D-004/D004-X-011 | I | evidence path | The intended grounded-claim path is Mode route → D-11 deterministic checker (including one bounded reformulation) or D-18 effect set → D-15 novelty label → human queue for candidates → D-20 aggregate proof. The D-11/D-15 and D-18/D-15 type gaps prevent treating this synthesis as fully specified. | D11:L161-L165, L375-L404; D18:L93-L102; D15:L83-L110; D20:L66-L70 |
| D-004/D004-X-012 | E | human authority | Two conservative authority boundaries recur: only a human may ratify DiscoveryCandidate, and only an operator may turn sandbox evidence into a production claim; both interactive mutation surfaces are deferred in v1. | D15:L88-L101, L139-L141, L220-L225; D18:L88-L91, L236-L239 |
| D-004/D004-X-013 | A | concurrency/idempotency | Across promotion veto/revert, checker reformulation, queue writes, observation writes, and nightly tau consumption, there is no shared transaction, idempotency key, locking order, snapshot rule, or crash-recovery contract beyond D-4’s Arc<RwLock<ArchiveChain>> and intent-before-retire event. | D4:L130-L142, L220-L221; D7:L119-L125; D11:L375-L395; D15:L88-L110; D18:L169-L175; D22:L63-L68 |
| D-004/D004-X-014 | E | paper gates | D-4 requires three new interfaces, D-7 one, D-11 three, D-15 three, D-18 three, while D-20 and D-22 rely on already-ingested prerequisites; all named ingest gates occur before testing. | D4:L64-L72; D7:L54-L59; D11:L68-L76; D15:L53-L61; D18:L42-L50; D20:L42-L44; D22:L24-L30 |
| D-004/D004-X-015 | A | observability naming | Sources use Rust-style SSB variants (TripwireFired, NoveltyLabeled, GoodhartRiskFlag, etc.); the summary renders several as uppercase underscore labels. No serialized-name mapping proves these are aliases. | D4:L144-L150; D15:L103-L110; D18:L104-L112 |

### D-004 summary deltas (namespaced D-004/<SD-*>)

| Aggregate ID | Kind | Required summary change | Deep records |
|---|---|---|---|
| D-004/D004-SD-001 | correction | Preserve Rust identifiers TripwireFired, TripwireFiredConcurrent, and TripwireAutoRevert rather than collapsing D-4 to a single TRIPWIRE_FIRED label; add event payloads and intent-before-retire ordering. | D004-D4-022..026, D004-D4-046 |
| D-004/D004-SD-002 | correction | Resolve “non-target family” versus implementation “ANY family” regression blocking; do not present the non-target rule as fully specified until reconciled. | D004-D7-001, D004-D7-020, D004-D7-037 |
| D-004/D004-SD-003 | addition | Add D-7’s Domain migration, three-valued decision, sparse-data deferral, exact fixtures/config/events, BAI pre-promotion hook, and its non-bijective Mixed gap/count conflict. | D004-D7-013..044 |
| D-004/D004-SD-004 | addition | Add all J-11/J-15 amendment behavior: register-time self-fixture fail-closed gate, demotion cap instead of averaging, Tier-C non-copy rule, and exactly one deterministic same-checker reformulation before Ungrounded. | D004-D11-044..051 |
| D-004/D004-SD-005 | correction | Do not state that D-15 adds NoveltyClass to CheckerResult; the plan attaches it to ClaimResult metadata, and the declared CheckerResult lacks classification inputs. | D004-D15-013, D004-D15-031..033 |
| D-004/D004-SD-006 | addition | Add D-15 queue file/schema/states/events, 0.85 precedence heuristic, empty-DKG fallback, conservative no-auto-publish boundary, and the fact that review mutation/TUI/config are unresolved or deferred. | D004-D15-011..029, D004-D15-034..038 |
| D-004/D004-SD-007 | addition | Add D-18 nested trigger/observation/outcome/environment types, actual orthogonality rule, operator-only production gate, exact config/events/artifact, and the one-effect reject-vs-flag conflict. | D004-D18-009..044 |
| D-004/D004-SD-008 | correction | Remove the claimed synthetic ARC fallback from v1: contamination produces ContaminationSuspected/exit 1, while synthetic generation is explicitly deferred. | D004-D20-015, D004-D20-027..028 |
| D-004/D004-SD-009 | correction | Expand D-20 prerequisites/landing order beyond D-10/D-15: it consumes D-14/D-16/D-17 and D-19 events, and its typed verdict cannot currently fail no-publish or sycophancy gates. | D004-D20-003, D004-D20-005, D004-D20-016, D004-D20-018, D004-D20-029..030 |
| D-004/D004-SD-010 | correction | Do not claim D-22’s distributions are operationally sourced from D-6; the plan builds a separate 300-file benchmark tree and never wires HeldOutStore. | D004-D22-003, D004-D22-005..008, D004-D22-020 |
| D-004/D004-SD-011 | addition | Add D-22’s tau.txt input, exact fixture counts, CLI/report/events, 500-tuple live gate, single-τ non-goal, and missing report/statistical/snapshot contracts. | D004-D22-005..026 |
| D-004/D004-SD-012 | addition | Add D-4 detector edge cases/state/config, deterministic trimming, append-only recovery evidence, false-positive cloud run, pre-D-8 inspection fallback, and unresolved signal/order/recovery details. | D004-D4-011..052 |
| D-004/D004-SD-013 | addition | Add D-11’s precise checker I/O, timeout/Inapplicable fallbacks, live anti-slopsquatting checks, isolation/sandbox requirements, first-applicable limitation, and missing ordering/registration/security/test contracts. | D004-D11-009..043, D004-D11-052..061 |
| D-004/D004-SD-014 | correction | Treat uppercase summary event names as descriptive labels unless a serializer mapping is supplied; originals define CamelCase Rust variants and several additional variants. | D004-X-015 |

---

## Summary-layer record (04-D-SUMMARY_INDEX)

Source file: `docs/thefinalglance/feature-discovery/04-D-SUMMARY_INDEX.discovery.md`
Role: orientation inventory of D-SUMMARY_INDEX.md (311 lines) + overview; **not** deep-plan evidence.
Where a deep ledger corrects the summary, deep wins target state (see Superseded notes and D-00N/SD|DELTA|SUM rows).

### Summary feature inventory atoms

#### D-1

- **SUM-D-1-001** [E] Introduces typed `ReasoningTemplate` records with atomic fields `id`, `domain`, `pattern_kind`, `template_body`, `eligibility_predicate`, `verifier_required`, `content_hash`, `parent_id`, and `archive_status`.
- **SUM-D-1-002** [E] Uses Blake3 content hashing for templates.
- **SUM-D-1-003** [E] Introduces an `ArchiveEntry` type and archive lifecycle.
- **SUM-D-1-004** [E] Adds SSB event types written in the summary as `ARCHIVE_ENTRY_ADDED/RETIRED`; `ARCHIVE_ENTRY_ADDED` is unambiguous, while the summary does not spell out whether the second full identifier is `RETIRED` or a prefixed name.
- **SUM-D-1-005** [E] Migrates existing prompt-embedded reasoning templates into typed records.
- **SUM-D-1-006** [E] Cloud proof runs `elai /task --mode cloud`, exercises at least three typed templates, requires `ARCHIVE_ENTRY_ADDED` rows, and verifies the content-hash chain.
- **SUM-D-1-007** [E] Direct prerequisites: C-15 stable `ClockProvider`; C-21 stable `MistakeBook` plus `rules.db`.
- **SUM-D-1-008** [E] Research grounding: SELF-DISCOVER, the bounded-capacity result in arXiv:2510.04399, and DGM.
#### D-2

- **SUM-D-2-001** [E] Adds a second inference sidecar dedicated to process/step reward scoring.
- **SUM-D-2-002** [E] Pins that PPM sidecar to port `8101`.
- **SUM-D-2-003** [E] Uses `Qwen2.5-Math-PRM-7B`, explicitly a different family from the `Qwen2.5-Coder-7B` policy SLM.
- **SUM-D-2-004** [E] Adds a dual-GPU placement specification.
- **SUM-D-2-005** [E] Adds a `PpmClient` trait.
- **SUM-D-2-006** [E] Adds a paper-ingestion prerequisite that pulls the Qwen2.5-Math-PRM source before implementation execution.
- **SUM-D-2-007** [E] Scores reasoning at step granularity.
- **SUM-D-2-008** [E] Cloud proof scores one known-correct and one known-wrong reasoning step and requires `correct > wrong` by a margin greater than `0.3`.
- **SUM-D-2-009** [E] Direct prerequisites: D-1 typed templates and B-37 multi-backend runtime.
- **SUM-D-2-010** [E] Research grounding: rStar-Math, heterogeneous verifiers, and PRMBench.
#### D-3

- **SUM-D-3-001** [E] Adds typed `VerificationResult` fields `deterministic`, `ppm`, `intrinsic`, `cross_context`, and `consensus_rule`.
- **SUM-D-3-002** [E] Aggregates heterogeneous verification signals rather than allowing the PPM alone to decide promotion.
- **SUM-D-3-003** [E] Blocks promotion unless at least two verifiers concur.
- **SUM-D-3-004** [E] Treats consensus as a PRMBench Deception Resistance defense.
- **SUM-D-3-005** [E] Cloud proof supplies deceptive reasoning where PPM passes but a deterministic checker fails; the candidate must not be promoted.
- **SUM-D-3-006** [E] Direct prerequisite: D-2 PPM client.
- **SUM-D-3-007** [E] Research grounding: V-STaR, PRMBench, and Cross-Context Verification.
#### D-4

- **SUM-D-4-001** [E] Replaces the prior single rule, described as “10% over 3 pipelines,” with five orthogonal regression tripwires.
- **SUM-D-4-002** [E] The five signals are KL, delta-rho (`Δρ`), IQR, perplexity, and DDM.
- **SUM-D-4-003** [E] Automatically reverts a mutation when at least two tripwires fire concurrently.
- **SUM-D-4-004** [E] Adds the SSB event type `TRIPWIRE_FIRED`.
- **SUM-D-4-005** [E] Revert targets come from the D-1 template archive; monitored verifier signals come from D-3.
- **SUM-D-4-006** [E] Cloud proof injects a synthetic regression, requires at least three tripwires to fire inside the applicable window, and requires the auto-revert to be visible in the archive.
- **SUM-D-4-007** [E] Direct prerequisites: D-1 and D-3.
- **SUM-D-4-008** [E] Research grounding includes measured reward-hacking prevalence/thresholds, concept-drift detectors DDM/Page-Hinkley/ADWIN, and reward-model overoptimization.
#### D-5

- **SUM-D-5-001** [E] Replaces MSI sigmoid-Boltzmann candidate promotion with best-arm identification (BAI).
- **SUM-D-5-002** [E] Adds `ArmPosterior { template_id, pass_count, fail_count }`.
- **SUM-D-5-003** [E] Uses Beta-posterior Thompson sampling.
- **SUM-D-5-004** [E] Includes Successive Rejects and LUCB mechanisms.
- **SUM-D-5-005** [E] Grounds promotion behavior in published regret bounds.
- **SUM-D-5-006** [E] Tracks D-1 archive entries as arms and consumes D-3 consensus as the binary pass/fail signal.
- **SUM-D-5-007** [E] Cloud proof runs a 50-round synthetic noisy-arm benchmark and requires empirical regret within the published Audibert bound.
- **SUM-D-5-008** [E] Direct prerequisites: D-1 and D-3.
- **SUM-D-5-009** [E] Research grounding: Audibert COLT 2010 BAI and variance-dependent BAI.
#### D-6

- **SUM-D-6-001** [E] Persists held-out tasks at `.elai_cc/held-out/{date}/{task-id}.toml`.
- **SUM-D-6-002** [E] Organizes held-out evaluation temporally.
- **SUM-D-6-003** [E] Applies a 9-gram overlap filter for contamination control.
- **SUM-D-6-004** [E] Automatically retires tasks that pass uniformly after six months because that behavior indicates probable contamination.
- **SUM-D-6-005** [E] Emits `HELD_OUT_REFRESH` to the SSB when the store refreshes.
- **SUM-D-6-006** [E] Places the held-out store on D-5’s promotion path as a gate.
- **SUM-D-6-007** [E] Cloud proof auto-retires a known-contaminated task that has passed uniformly for more than six months; ingestion of a new task must emit a store-refresh row.
- **SUM-D-6-008** [E] Direct prerequisite: D-5.
- **SUM-D-6-009** [E] Research grounding: LiveCodeBench temporal/9-gram controls, Soft Contamination, and the ARC Prize 2025 report.
#### D-7

- **SUM-D-7-001** [E] Defines five orthogonal evaluation families: `code-fix`, `math-proof`, `prose-reason`, `planning`, and `knowledge-QA`.
- **SUM-D-7-002** [E] Uses family tags supplied by the D-6 held-out store.
- **SUM-D-7-003** [E] Blocks promotion if any non-target family regresses by more than `5%`.
- **SUM-D-7-004** [E] Emits a `CrossDomainRegression` SSB record.
- **SUM-D-7-005** [E] Cloud proof introduces a mutation with an `8%` prose-reason regression and requires promotion to be blocked with a regression row whose delta is `-0.08`.
- **SUM-D-7-006** [E] Direct prerequisite: D-6.
- **SUM-D-7-007** [E] Research grounding: NanoFlux.
#### D-8

- **SUM-D-8-001** [E] Hash-chains every mutation candidate with Blake3.
- **SUM-D-8-002** [E] Makes the archive append-only.
- **SUM-D-8-003** [E] Defines the more detailed `ArchiveEntry { self_hash, parent_hash, template, eval_result, promoted_at, retired_at }` record.
- **SUM-D-8-004** [E] Adds the CLI command `elai archive verify`.
- **SUM-D-8-005** [E] Adds a tamper-test contract.
- **SUM-D-8-006** [E] Cloud proof requires any altered archive row to make verification exit `1` with a broken-link report; a clean archive must exit `0`.
- **SUM-D-8-007** [E] Direct prerequisite: D-1 archive lifecycle.
- **SUM-D-8-008** [E] Research grounding: cryptographic-checksum guidance and DGM archive practice.
#### D-9

- **SUM-D-9-001** [E] Documents and enforces the L3-locked surface.
- **SUM-D-9-002** [E] Explicitly locks the staged-evaluation harness.
- **SUM-D-9-003** [E] Explicitly locks tripwire thresholds.
- **SUM-D-9-004** [E] Explicitly locks MSI/BAI scoring.
- **SUM-D-9-005** [E] Explicitly locks archive maintenance.
- **SUM-D-9-006** [E] Explicitly locks held-out selection.
- **SUM-D-9-007** [E] Explicitly locks canary-family definitions.
- **SUM-D-9-008** [E] Explicitly locks selection of Blake3 itself.
- **SUM-D-9-009** [E] Explicitly locks the `AllowedConfigKey` bound itself.
- **SUM-D-9-010** [E] Explicitly locks the `verifier_required` field.
- **SUM-D-9-011** [E] Explicitly locks the `Domain` enum.
- **SUM-D-9-012** [E] Adds a pre-commit enforcement check.
- **SUM-D-9-013** [E] Adds `LockedKeysAuditor`.
- **SUM-D-9-014** [E] Adds the SSB event `LOCKED_KEYS_TOUCH`.
- **SUM-D-9-015** [E] Cloud proof simulates an agent editing evaluator code and requires the pre-commit hook, auditor, event row, and a blocked `git push`.
- **SUM-D-9-016** [E] Direct prerequisites: all D-1 through D-8 artifacts must exist.
- **SUM-D-9-017** [E] Research grounding: current-reward-function integrity, boxed/myopic design, and learned/mesa-optimization risk.
#### D-10

- **SUM-D-10-001** [E] Adds a `GroundingMode` enum with five modes `A/B/C/D/E`.
- **SUM-D-10-002** [E] Adds the orchestrator function `select_grounding_mode`.
- **SUM-D-10-003** [E] Routes claims based on whether they are operationalizable, whether a checker is registered, sandbox availability, cross-family availability, and an intrinsic-signal threshold.
- **SUM-D-10-004** [E] Keeps mode selection entirely orchestrator-owned; the LLM has no input to selection.
- **SUM-D-10-005** [E] Requires the routing function itself to be L3-frozen.
- **SUM-D-10-006** [E] Emits `MODE_SELECTED` to the SSB.
- **SUM-D-10-007** [E] Cloud proof uses 50 synthetically labeled claims and requires at least 45 correct assignments, stated as at least `90% F1`.
- **SUM-D-10-008** [E] Direct prerequisite: D-9.
- **SUM-D-10-009** [E] Research grounding: current-reward-function integrity and real-problem §3.2.
#### D-11

- **SUM-D-11-001** [E] Bootstraps registry directories `.elai_cc/checkers/{math,code,text,graph,api,perf}/`.
- **SUM-D-11-002** [E] Ships six initial checker integrations: `sympy`, `z3`, `compile-rust`, `compile-py`, `registry-check`, and `regex`.
- **SUM-D-11-003** [E] Adds a typed `CheckerResult` return value.
- **SUM-D-11-004** [E] Adds orchestrator-side checker-dispatch logic.
- **SUM-D-11-005** [E] Makes Grounding Mode A operational.
- **SUM-D-11-006** [E] Checkers are reached through D-10 Mode A routing.
- **SUM-D-11-007** [E] Cloud proof requires a mathematical claim to be verified by SymPy, a code claim by `cargo check`, and a novel npm-package claim by the registry checker.
- **SUM-D-11-008** [E] Direct prerequisite: D-10.
- **SUM-D-11-009** [E] Research grounding: Program-of-Thought, CodeAct, and AlphaProof.
#### D-12

- **SUM-D-12-001** [E] Extends BabyStepSpec block 7 with a `MEASURABLE_PROCEDURE` field.
- **SUM-D-12-002** [E] Requires a claim-emitting baby step to name a measurable procedure that references a checker.
- **SUM-D-12-003** [E] Blocks a claim-emitting baby step without that procedure at the `AUDITING` gate.
- **SUM-D-12-004** [E] Emits `CLAIM_WITHOUT_PROCEDURE` to the SSB on that failure.
- **SUM-D-12-005** [E] Cloud proof has a baby-step author emit a claim without a procedure and requires both auditor block and event row.
- **SUM-D-12-006** [E] Direct prerequisite: D-11.
- **SUM-D-12-007** [E] Research grounding: Voyager’s measurable-effect skill pattern and real-problem §4.1.
#### D-13

- **SUM-D-13-001** [E] Adds `AgentRole::Adversary`.
- **SUM-D-13-002** [E] Requires the Adversary to use a model family different from the executor.
- **SUM-D-13-003** [E] Uses cloud Sonnet as fallback when no alternate local family is available.
- **SUM-D-13-004** [E] Requires structured counter-evidence as Adversary output.
- **SUM-D-13-005** [E] Forbids same-model self-critique; the summary says “NEVER.”
- **SUM-D-13-006** [E] Falls back to Mode E refusal when no alternate family is available.
- **SUM-D-13-007** [E] Routes this function through Mode B.
- **SUM-D-13-008** [E] Feeds the Adversary verdict into D-3 consensus.
- **SUM-D-13-009** [E] Emits `ADVERSARY_VERDICT` and `CROSS_FAMILY_UNAVAILABLE` to the SSB.
- **SUM-D-13-010** [E] Cloud proof starts with a confident executor claim, obtains disagreement from the cross-family Adversary, requires consensus rejection, emits an adversary row, and blocks the claim.
- **SUM-D-13-011** [E] Direct prerequisites: D-10 and D-3.
- **SUM-D-13-012** [E] Research grounding: Self-Correction Bench blind spots, lightweight latent verifiers, and small-model constitutional-AI collapse.
#### D-14

- **SUM-D-14-001** [E] Re-runs every user-aligned claim with its framing inverted.
- **SUM-D-14-002** [E] Treats converging original/inverted outputs as robust.
- **SUM-D-14-003** [E] Treats divergent outputs as evidence of sycophancy.
- **SUM-D-14-004** [E] Emits `SYCOPHANCY_PROBE` for probe activity and `SYCOPHANCY_FLAG` for divergence.
- **SUM-D-14-005** [E] Quarantines flagged claims for human review.
- **SUM-D-14-006** [E] Reuses D-13 cross-family infrastructure for the inverted-framing run.
- **SUM-D-14-007** [E] Cloud proof gives a false user framing that the model initially accepts and requires the inverted run to diverge and fire `SYCOPHANCY_FLAG`.
- **SUM-D-14-008** [E] Direct prerequisite: D-13.
- **SUM-D-14-009** [E] Research grounding: Sharma et al. sycophancy probes.
#### D-15

- **SUM-D-15-001** [E] Adds `NoveltyClass { Search, DistantSearch, DiscoveryCandidate }`.
- **SUM-D-15-002** [E] Applies a novelty class to every Mode A result.
- **SUM-D-15-003** [E] Applies a novelty class to every Mode D result.
- **SUM-D-15-004** [E] Adds `NoveltyClass` as a field on `CheckerResult`.
- **SUM-D-15-005** [E] Sends every `DiscoveryCandidate` to a human-review queue rather than automatically accepting it.
- **SUM-D-15-006** [E] Emits `NOVELTY_LABELED` to the SSB.
- **SUM-D-15-007** [E] Uses C-17 DKG resilience for near-duplicate detection.
- **SUM-D-15-008** [E] Cloud proof requires a known lookup to label `Search`, a novel composition to label `DistantSearch`, and a primitive absent from the DKG to label `DiscoveryCandidate`.
- **SUM-D-15-009** [E] Direct prerequisites: D-11, D-12, and C-17.
- **SUM-D-15-010** [E] Research grounding: FunSearch, AlphaEvolve, ARC, and the real-problem Mode-C contradiction discussion.
#### D-16

- **SUM-D-16-001** [E] Applies UniCR-style conformal risk control to “I don’t know” behavior.
- **SUM-D-16-002** [E] Persists calibration data under `.elai_cc/calibration/`.
- **SUM-D-16-003** [E] Stores tuples `(claim, ground_truth_available, model_confidence)`.
- **SUM-D-16-004** [E] Maintains threshold `τ`.
- **SUM-D-16-005** [E] Updates `τ` nightly.
- **SUM-D-16-006** [E] Restricts the threshold updater to L3.
- **SUM-D-16-007** [E] Refuses claims below the calibrated threshold.
- **SUM-D-16-008** [E] Formally grounds D-10 Mode E as the refusal mode.
- **SUM-D-16-009** [E] Emits `REFUSAL_DECISION` and `CALIBRATION_UPDATE` to the SSB.
- **SUM-D-16-010** [E] Cloud proof evaluates a distribution-shifted held-out family and requires empirical refusal behavior to remain within `ε = 0.1`.
- **SUM-D-16-011** [E] Direct prerequisites: D-9 and D-10.
- **SUM-D-16-012** [E] Research grounding: UniCR, uncertainty outside parametric knowledge, and rewarding doubt.
#### D-17

- **SUM-D-17-001** [E] Adds the per-session metric `refusal_rate = refusals / total_claims`.
- **SUM-D-17-002** [E] Automatically flags a session below `5%` refusal rate as likely confabulation.
- **SUM-D-17-003** [E] Automatically flags a session above `50%` refusal rate as likely checker misconfiguration.
- **SUM-D-17-004** [E] Uses that lower-and-upper band as a structural defense against gaming Mode E as the easy route.
- **SUM-D-17-005** [E] Consumes refusal events emitted by D-16.
- **SUM-D-17-006** [E] Cloud proof instruments the last 20 sessions, expects a healthy band to emerge empirically, and requires a synthetic session below `2%` to emit an `AUTO_FLAG` row.
- **SUM-D-17-007** [E] Direct prerequisite: D-16.
- **SUM-D-17-008** [E] Research grounding: real-problem §4.6 and the TMLR 2025 LLM Honesty Survey.
#### D-18

- **SUM-D-18-001** [E] Extends the Phase 4F Feasibility Probe with typed observable effects.
- **SUM-D-18-002** [E] Adds `ObservableEffect { trigger, expected_observation, success_predicate, bounded_runtime, sandbox_tier, confound_flags }`.
- **SUM-D-18-003** [E] Requires at least two `ObservableEffect` records per claim.
- **SUM-D-18-004** [E] Labels whether an observation came from sandbox or production.
- **SUM-D-18-005** [E] Uses D-11 checkers as Mode D effect verifiers.
- **SUM-D-18-006** [E] Cloud proof gives a feasibility probe only one observable effect and requires `GOODHART_RISK_FLAG`; with two or more effects the flag must be absent.
- **SUM-D-18-007** [E] Direct prerequisites: C-22 FeasibilityProber completeness and D-11.
- **SUM-D-18-008** [E] Research grounding: Voyager, Eureka, and Coscientist.
#### D-19

- **SUM-D-19-001** [E] Pulls PRMBench fixtures from the `Deception Resistance` and `Circular Logic` subsets.
- **SUM-D-19-002** [E] Runs the D-2 PPM on adversarial samples.
- **SUM-D-19-003** [E] Reports scores separately by category.
- **SUM-D-19-004** [E] Gates PPM promotion on reaching at least the published baseline.
- **SUM-D-19-005** [E] Cloud proof sets conservative concrete floors: Deception Resistance at least `35%` versus a cited published `40–50%` range for Qwen2.5-Math-PRM-7B; Circular Logic at least `30%`.
- **SUM-D-19-006** [E] D-3 consumes the PPM score through its consensus rule.
- **SUM-D-19-007** [E] Direct prerequisites: D-2 and D-3.
- **SUM-D-19-008** [E] Research grounding: PRMBench, Lessons of Developing PRMs, and ThinkPRM.
#### D-20

- **SUM-D-20-001** [E] Pulls a public ARC-AGI-2 task subset.
- **SUM-D-20-002** [E] Falls back to synthetic ARC-style tasks if hand-crafted leakage is flagged.
- **SUM-D-20-003** [E] Runs ELAI on cloud Sonnet rather than a local 7B model for ARC.
- **SUM-D-20-004** [E] Runs with mode routing enabled.
- **SUM-D-20-005** [E] Reports solve rate.
- **SUM-D-20-006** [E] Reports Mode-E refusal rate.
- **SUM-D-20-007** [E] Compares those metrics against a human baseline.
- **SUM-D-20-008** [E] Cloud proof runs 50 ARC-AGI-2 tasks, requires solve-rate and refusal-rate reporting, requires honest-refusal SLA to remain in band, and forbids automatic publication of any `DiscoveryCandidate` claim.
- **SUM-D-20-009** [E] Direct prerequisites: D-10 and D-15.
- **SUM-D-20-010** [E] Research grounding: ARC, ARC Prize 2025, and NVARC NVIDIA 2025.
#### D-21

- **SUM-D-21-001** [E] Builds 200 synthetic claims with hand-labeled ground-truth modes A/B/C/D/E.
- **SUM-D-21-002** [E] Evaluates `select_grounding_mode` using overall F1.
- **SUM-D-21-003** [E] Produces a per-mode confusion matrix.
- **SUM-D-21-004** [E] Reports edge-case failures.
- **SUM-D-21-005** [E] Cloud proof requires overall F1 of at least `0.85`.
- **SUM-D-21-006** [E] Prevents class collapse by requiring at least 10 correct examples in every mode.
- **SUM-D-21-007** [E] Direct prerequisite: D-10.
- **SUM-D-21-008** [E] Research grounding: real-problem §3.2 and the Self-Correction Bench blind-spot framework.
#### D-22

- **SUM-D-22-001** [E] Builds three held-out distributions: in-domain, near-OOD, and far-OOD.
- **SUM-D-22-002** [E] Sources those distributions from the D-6 held-out store.
- **SUM-D-22-003** [E] Validates UniCR’s empirical refusal rate under all three shifts.
- **SUM-D-22-004** [E] Uses user-set `ε = 0.1`.
- **SUM-D-22-005** [E] Cloud proof requires empirical `refusal_rate` inside `[τ-ε, τ+ε]` for every distribution and fails if any distribution shifts by more than epsilon.
- **SUM-D-22-006** [E] Direct prerequisites: D-16 and D-6.
- **SUM-D-22-007** [E] Research grounding: UniCR’s distribution-free guarantee.
#### D-23

- **SUM-D-23-001** [E] Pulls Anthropic/Sharma public sycophancy probes.
- **SUM-D-23-002** [E] Permits a synthetic stand-in if those public fixtures cannot be used.
- **SUM-D-23-003** [E] Runs a probe-enabled versus probe-disabled comparison.
- **SUM-D-23-004** [E] Reports detection rate.
- **SUM-D-23-005** [E] Reports false-positive rate.
- **SUM-D-23-006** [E] Cloud proof requires detection rate at least `70%` on labeled fixtures and false-positive rate at most `15%`.
- **SUM-D-23-007** [E] Direct prerequisite: D-14.
- **SUM-D-23-008** [E] Research grounding: Sharma et al.
#### D-24

- **SUM-D-24-001** [E] Generates 100 synthetic mutations across all five D-7 families.
- **SUM-D-24-002** [E] Fixture mix is exactly 50 helpful, 30 neutral, and 20 harmful mutations.
- **SUM-D-24-003** [E] Requires all 20 harmful mutations to be blocked.
- **SUM-D-24-004** [E] Allows at most 5 of 50 helpful mutations to be blocked, described as a false-positive ceiling of `10%`.
- **SUM-D-24-005** [E] Allows at most 2 of 30 neutral mutations to be blocked.
- **SUM-D-24-006** [E] Direct prerequisite: D-7.
- **SUM-D-24-007** [E] Research grounding: NanoFlux and companion §3.5.
#### D-25

- **SUM-D-25-001** [E] Defines a ten-scenario archive tamper suite.
- **SUM-D-25-002** [E] Explicit scenario: delete an archive row.
- **SUM-D-25-003** [E] Explicit scenario: swap `parent_hash`.
- **SUM-D-25-004** [E] Explicit scenario: edit the template body.
- **SUM-D-25-005** [E] Explicit scenario: replace Blake3 with SHA-256 on disk.
- **SUM-D-25-006** [E] Requires `elai archive verify` to detect every scenario within five seconds.
- **SUM-D-25-007** [E] Requires 10/10 detection.
- **SUM-D-25-008** [E] Requires a clean repository to exit `0`.
- **SUM-D-25-009** [E] Requires tamper-then-revert to recover to clean verification.
- **SUM-D-25-010** [E] Uses C-27 `DeterminismDrift` alongside D-8 archive verification.
- **SUM-D-25-011** [E] Direct prerequisites: D-8 and C-27.
- **SUM-D-25-012** [E] Research grounding: checksum integrity guidance and companion §3.8.
#### D-26

- **SUM-D-26-001** [E] Adds `MutationArchivePanel.svelte`.
- **SUM-D-26-002** [E] Adds `LineageGraphView.svelte`.
- **SUM-D-26-003** [E] Uses Cytoscape to render lineage.
- **SUM-D-26-004** [E] Cross-links reasoning-template lineage to the baby-step archive.
- **SUM-D-26-005** [E] Extends `ReasoningDrawer` with PPM score.
- **SUM-D-26-006** [E] Extends `ReasoningDrawer` with the consensus rule/result.
- **SUM-D-26-007** [E] Extends `ReasoningDrawer` with tripwire status.
- **SUM-D-26-008** [E] Adds Tauri command `get_archive_recent`.
- **SUM-D-26-009** [E] Adds Tauri command `get_archive_verify_status`.
- **SUM-D-26-010** [E] Adds Tauri command `get_tripwire_status`.
- **SUM-D-26-011** [E] Cloud proof runs a live cloud `/task` that promotes a template; the archive panel must display the new `ARCHIVE_ENTRY_ADDED` row and its lineage edge.
- **SUM-D-26-012** [E] Cloud proof injects a tripwire firing and requires a red dashboard flag inside the polling interval.
- **SUM-D-26-013** [E] Direct prerequisite: D-1 through D-8 shipped.
#### D-27

- **SUM-D-27-001** [E] Adds `ModeRoutingPanel.svelte` with a per-session A/B/C/D/E breakdown.
- **SUM-D-27-002** [E] Adds `RefusalRateGauge.svelte` with healthy-band visualization.
- **SUM-D-27-003** [E] Adds `NoveltyBreakdown.svelte` with counts for `Search`, `DistantSearch`, and `DiscoveryCandidate`.
- **SUM-D-27-004** [E] Adds `SycophancyFlagStrip.svelte` as a real-time flag stream.
- **SUM-D-27-005** [E] Adds Tauri command `get_mode_breakdown`.
- **SUM-D-27-006** [E] Adds Tauri command `get_refusal_rate`.
- **SUM-D-27-007** [E] Adds Tauri command `get_novelty_breakdown`.
- **SUM-D-27-008** [E] Adds Tauri command `get_sycophancy_flags`.
- **SUM-D-27-009** [E] Cloud proof runs a live cloud `/task` with mixed claims and requires a realistic mode distribution.
- **SUM-D-27-010** [E] A fired sycophancy probe must appear on the dashboard inside the polling interval.
- **SUM-D-27-011** [E] Direct prerequisite: D-10 through D-17 shipped.
#### D-28

- **SUM-D-28-001** [E] Iteratively generates out-of-distribution examples that obtain high proxy reward while remaining low quality.
- **SUM-D-28-002** [E] Retrains the PPM on the adversarial corpus to immunize it.
- **SUM-D-28-003** [E] Defers bootstrapping until D-5 BAI has a stable selection signal with at least 20 promotion cycles.
- **SUM-D-28-004** [E] Requires the D-19 PRMBench baseline to have been measured.
- **SUM-D-28-005** [E] Requires RunPod for the retraining run.
- **SUM-D-28-006** [E] Cloud proof compares pre/post immunization and requires Deception Resistance to improve by at least `+5` percentage points with no regression on standard categories.
- **SUM-D-28-007** [E] Direct prerequisites: D-5 and D-19.
- **SUM-D-28-008** [E] Research grounding: adversarial training of reward models.
#### D-29

- **SUM-D-29-001** [E] Investigates the mismatch between published IID-reward BAI assumptions and ELAI’s non-IID outcomes across sessions.
- **SUM-D-29-002** [E] Spikes a hierarchical Bayesian model.
- **SUM-D-29-003** [E] Uses D-5 flat BAI as the comparison baseline.
- **SUM-D-29-004** [E] Requires RunPod for a hierarchical Gibbs sampler.
- **SUM-D-29-005** [E] Accepts a documented infeasibility note as a valid closure outcome.
- **SUM-D-29-006** [E] Success path: hierarchical regret is no worse than flat-BAI regret on a synthetic non-IID benchmark.
- **SUM-D-29-007** [E] Infeasibility path: document at least one concrete blocker.
- **SUM-D-29-008** [E] Direct prerequisite: D-5.
- **SUM-D-29-009** [E] Research grounding: Audibert, variance-dependent BAI, and companion §3.7/Q4.
#### D-30

- **SUM-D-30-001** [E] Establishes that useful UniCR calibration needs `N = 500` tuples.
- **SUM-D-30-002** [E] Addresses the cold-start condition where early local sessions cannot provide that many tuples.
- **SUM-D-30-003** [E] Spikes bootstrapping from a generic LLM-calibration corpus.
- **SUM-D-30-004** [E] Refines the generic bootstrap per project as real session data accumulates.
- **SUM-D-30-005** [E] Requires access to a HuggingFace calibration dataset that is still to be identified.
- **SUM-D-30-006** [E] Cloud proof requires bootstrap calibration to stay within epsilon on held-out validation.
- **SUM-D-30-007** [E] Later real-session-refined calibration must have empirical risk no worse than the bootstrap.
- **SUM-D-30-008** [E] Direct prerequisite: D-16.
- **SUM-D-30-009** [E] Research grounding: UniCR and real-problem Q5.
- **SUM-D-30-010** [E] Each D-N original names its paper prerequisites. Before `EXECUTING`, `paper-ingestor` must pull and index each needed paper into `workflow/papers/{slug}-interface.md` (`D:238-264`).
- **SUM-D-30-011** [E] `spec-grounder` verifies the required interface files before unblocking `TESTING_FIRST` (`D:264`).
- **SUM-D-30-012** [E] Every executable D session is then expected to receive a live cloud-Sonnet proof and, when applicable, a paper-grounded numeric benchmark (`D:3-4`).
- **SUM-D-30-013** [E] D-1 is globally sequenced after the required C bedrock; D-10 begins only after Wave 1’s frozen-surface enforcement (`D:80-84,294-307`).
- **SUM-D-30-014** [E] `ReasoningTemplate`: `id`, `domain`, `pattern_kind`, `template_body`, `eligibility_predicate`, `verifier_required`, `content_hash`, `parent_id`, `archive_status` (D-1; `D:22-26`).
- **SUM-D-30-015** [E] `ArchiveEntry` is introduced in D-1; D-8 explicitly specifies `self_hash`, `parent_hash`, `template`, `eval_result`, `promoted_at`, `retired_at` (`D:22-23,64-68`).
- **SUM-D-30-016** [E] `VerificationResult`: `deterministic`, `ppm`, `intrinsic`, `cross_context`, `consensus_rule` (D-3; `D:34-38`).
- **SUM-D-30-017** [E] `ArmPosterior`: `template_id`, `pass_count`, `fail_count` (D-5; `D:46-50`).
- **SUM-D-30-018** [E] Held-out task records are TOML files partitioned by date and task ID; per-family tagging exists for cross-domain canaries (D-6/D-7; `D:52-60`).
- **SUM-D-30-019** [E] `GroundingMode`: five values A/B/C/D/E (D-10; `D:80-84`).
- **SUM-D-30-020** [E] `CheckerResult`: typed checker return, later extended with a `NoveltyClass` field (D-11/D-15; `D:86-90,110-114`).
- **SUM-D-30-021** [E] `NoveltyClass`: `Search`, `DistantSearch`, `DiscoveryCandidate` (D-15; `D:110-114`).
- **SUM-D-30-022** [E] Calibration tuple: `(claim, ground_truth_available, model_confidence)` (D-16; `D:116-120`).
- **SUM-D-30-023** [E] `ObservableEffect`: `trigger`, `expected_observation`, `success_predicate`, `bounded_runtime`, `sandbox_tier`, `confound_flags` (D-18; `D:128-132`).
- **SUM-D-30-024** [E] `AgentRole::Adversary` (D-13; `D:98-102`).
- **SUM-D-30-025** [E] `.elai_cc/held-out/{date}/{task-id}.toml`: temporal held-out store (D-6).
- **SUM-D-30-026** [E] `.elai_cc/checkers/{math,code,text,graph,api,perf}/`: constructive-checker registry families (D-11).
- **SUM-D-30-027** [E] `.elai_cc/calibration/`: refusal-calibration tuples and threshold inputs (D-16).
- **SUM-D-30-028** [E] `workflow/papers/{slug}-interface.md`: ingested paper interfaces required before testing/execution (`D:264`).
- **SUM-D-30-029** [E] L3 frozen configuration/governance surfaces: staged evaluator, tripwire thresholds, MSI/BAI scoring, archive maintenance, held-out selection, canary families, Blake3 choice, the `AllowedConfigKey` bound, `verifier_required`, and `Domain` (`D:70-74`).
- **SUM-D-30-030** [E] UniCR `τ` is nightly-updated but only by L3 code (`D:116-120`).
- **SUM-D-30-031** [E] D-2’s process integration fixes the PPM endpoint to port 8101 and declares dual-GPU placement (`D:28-32`).
- **SUM-D-30-032** [E] CLI `elai /task --mode cloud` is the D-1 live exercise command (`D:26`).
- **SUM-D-30-033** [E] CLI `elai archive verify` returns `0` for clean state and `1` plus a broken-link report for tamper (`D:64-68`).
- **SUM-D-30-034** [E] Orchestrator API/function `select_grounding_mode` is the closed L3 route selector (`D:80-84`).
- **SUM-D-30-035** [E] `PpmClient` trait abstracts the PPM sidecar (`D:28-32`).
- **SUM-D-30-036** [E] Checker dispatch is orchestrator-owned and typed through `CheckerResult` (`D:86-90`).
- **SUM-D-30-037** [E] Dashboard components: `MutationArchivePanel.svelte`, `LineageGraphView.svelte`, `ReasoningDrawer` extensions, `ModeRoutingPanel.svelte`, `RefusalRateGauge.svelte`, `NoveltyBreakdown.svelte`, `SycophancyFlagStrip.svelte` (`D:182-192`).
- **SUM-D-30-038** [E] Tauri archive commands: `get_archive_recent`, `get_archive_verify_status`, `get_tripwire_status` (`D:182-186`).
- **SUM-D-30-039** [E] Tauri grounding commands: `get_mode_breakdown`, `get_refusal_rate`, `get_novelty_breakdown`, `get_sycophancy_flags` (`D:188-192`).
- **SUM-D-30-040** [E] UI update contract: tripwire and sycophancy flags must appear within the dashboard poll interval (`D:186,192`).
- **SUM-D-30-041** [E] Model/runtime: cloud Sonnet live tests; Qwen2.5-Math-PRM-7B verifier sidecar; Qwen2.5-Coder-7B policy SLM; B-37 multi-backend runtime; dual-GPU placement (`D:4,28-32`).
- **SUM-D-30-042** [E] Constructive verification: SymPy, Z3, Rust compilation/`cargo check`, Python compilation, package registry checks including npm, regex, plus registry folders for text/graph/API/performance (`D:86-90`).
- **SUM-D-30-043** [E] Retrieval/near-duplicate support: C-17 DKG resilience, C-24 `symbol_impact`, and C-30 surgical-mode DKG tooling (`D:110-114,301`).
- **SUM-D-30-044** [E] Feasibility: C-22 `FeasibilityProber` supplies the sandbox/probe substrate extended by D-18 (`D:128-132,304`).
- **SUM-D-30-045** [E] Dashboard: Tauri, Svelte, and Cytoscape (`D:182-192`).
- **SUM-D-30-046** [E] Research datasets/benchmarks: PRMBench, ARC-AGI-2 or a synthetic leakage-safe stand-in, Sharma public sycophancy probes or a synthetic stand-in, three held-out distribution tiers, and synthetic mutation/tamper suites (`D:138-178`).
- **SUM-D-30-047** [E] Deferred compute/data: RunPod retraining and Gibbs sampling for D-28/D-29; an unidentified HuggingFace calibration corpus for D-30 (`D:196-212`).
- **SUM-D-30-048** [E] The LLM must have no influence on grounding-mode selection (`D:78-84`).
- **SUM-D-30-049** [E] No reasoning-template promotion may occur with fewer than two concurring verifiers (`D:34-38`).
- **SUM-D-30-050** [E] The policy model and PPM must be heterogeneous model families (`D:28-32`).
- **SUM-D-30-051** [E] The Adversary must be cross-family; same-model self-critique is forbidden (`D:98-102`).
- **SUM-D-30-052** [E] If cross-family verification cannot be obtained, the system refuses through Mode E rather than weakening the verifier requirement (`D:98-102`).
- **SUM-D-30-053** [E] Any non-target family regression above 5% blocks mutation promotion (`D:58-62`).
- **SUM-D-30-054** [E] Two or more concurrent tripwire firings force automatic revert (`D:40-44`).
- **SUM-D-30-055** [E] Every mutation candidate is chained; the archive is append-only; tamper must produce nonzero CLI status and a broken-link report (`D:64-68`).
- **SUM-D-30-056** [E] Agents cannot modify the frozen evaluator/governance surfaces; attempted evaluator edits block push and are journaled (`D:70-74`).
- **SUM-D-30-057** [E] A claim-emitting baby step without a checker-referenced measurable procedure cannot pass `AUDITING` (`D:92-96`).
- **SUM-D-30-058** [E] A `DiscoveryCandidate` requires human review and cannot be auto-published in the ARC proof (`D:110-114,144-148`).
- **SUM-D-30-059** [E] A divergent inverted-framing result is quarantined for human review (`D:104-108`).
- **SUM-D-30-060** [E] Mode-D claims need at least two observable effects; only one triggers Goodhart-risk handling (`D:128-132`).
- **SUM-D-30-061** [E] System-wide orientation only: D composes with ELAI’s off-agent-evidence, fail-closed, and deterministic spine; this contextual invariant comes from the overview, not extra D implementation detail (`OVERVIEW:8-24`).
- **SUM-D-30-062** [E] Calibrated confidence below `τ` yields refusal (`D:116-120`).
- **SUM-D-30-063** [E] Alternate-local-model absence causes cloud-Sonnet cross-family fallback; total alternate-family unavailability causes Mode-E refusal (`D:98-102`).
- **SUM-D-30-064** [E] Refusal below 5% or above 50% is itself flagged, preventing “never abstain” and “always abstain” strategies (`D:122-126`).
- **SUM-D-30-065** [E] Public ARC fixtures may degrade to synthetic ARC-style fixtures only when hand-crafted leakage is flagged (`D:144-148`).
- **SUM-D-30-066** [E] Public sycophancy fixtures may degrade to a synthetic stand-in (`D:162-166`).
- **SUM-D-30-067** [E] D-29 may validly close with a concrete infeasibility result rather than a successful hierarchy (`D:202-206`).
- **SUM-D-30-068** [E] D contains access-control, cryptographic-integrity, calibration, quarantine, and human-review behavior, but the summary specifies no distinct PII, secret-handling, user-data retention, network-privacy, or telemetry-export feature.
- **SUM-D-30-069** [E] Explicit error/reporting surfaces are the archive verifier’s exit code and broken-link report, blocked audit/push/promotion states, SSB diagnostic rows, benchmark failure thresholds, human-review queues, and D-29’s infeasibility report (`D:64-74,92-132,138-212`).
- **SUM-D-30-070** [E] D-1 extends C-21 self-modification automation—`check_anti_stale` plus L1A data tables—into a typed reasoning-template surface (`D:298`).
- **SUM-D-30-071** [E] D-4 replaces C-27 T(-1) `DeterminismDrift` single-signal detection with five orthogonal signals (`D:299`).
- **SUM-D-30-072** [E] D-9 extends C-27 WiringContract `StubBanAuditor` with frozen-surface access-control predicates (`D:300`).
- **SUM-D-30-073** [E] D-11 consumes C-24 `symbol_impact` and C-30 surgical-mode DKG tooling for API/code checker dispatch (`D:301`).
- **SUM-D-30-074** [E] D-12 extends C-31 `FILE_PLAN` with a `MEASURABLE_PROCEDURE` block (`D:302`; block-number inconsistency noted below).
- **SUM-D-30-075** [E] D-13 extends C-12 per-level auditor `REVIEW` modes with structured cross-family adversarial probing (`D:303`).
- **SUM-D-30-076** [E] D-18 extends C-22 FeasibilityProber with measurable-effect schema (`D:304`).
- **SUM-D-30-077** [E] D-26/D-27 extend C-7 dashboard completeness with D-specific drawers/panels (`D:305`).
- **SUM-D-30-078** [E] Global start gate: `spec-grounder` refuses D-1 until at least C-1, C-15, C-17, C-21, C-22, C-24, and C-27 are `DONE`; the substrate must also report `SCHEMA_VERSION >= 47`, WiringContract, DKG resilience, and MistakeBook (`D:307`).
- **SUM-D-30-079** [E] Reasoning-improvement mapping: §3.1→D-1, §3.2→D-2, §3.3→D-3, §§3.6/3.9→D-4, §3.7→D-5, §3.4→D-6, §3.5→D-7, §3.8→D-8, §2→D-9, Phase 4H.X→D-28, and Q4→D-29 (`D:266-279`).
- **SUM-D-30-080** [E] Real-problem mapping: §3.2→D-10, §4.2→D-11, §4.1→D-12, §4.3→D-13, §4.7→D-14, §4.8→D-15, §4.5→D-16, §4.6→D-17, §4.4→D-18, and Q5→D-30 (`D:281-292`).
- **SUM-D-30-081** [E] All 30 original D-N files were authored by 2026-05-19 (`D:218-221`).
- **SUM-D-30-082** [E] Each D-1..D-27 implementation session is estimated at roughly 6–8 hours; combined CPU/cloud work is roughly 160–220 hours (`D:222-225`).
- **SUM-D-30-083** [E] Each deferred session is estimated at roughly 8–24 wall-clock hours; combined deferred work is roughly 25–75 hours plus RunPod cost (`D:223-225`).
- **SUM-D-30-084** [E] A session is not `DONE` merely because its file exists: the status definition requires shipped implementation, cloud-Sonnet smoke test, and benchmark threshold (`D:10-14`).
- **SUM-D-30-085** [E] Existing baseline explicitly cited by path: `workflow/papers/dgm-interface.md`; its PDF `.claude/papers/2505.22954v3.pdf` is also present (`D:233-236`).
- **SUM-D-30-086** [E] The summary’s explicit “introduces to ingest” set is: rStar-Math; V-STaR; arXiv:2507.05619; Audibert COLT 2010 BAI; variance-dependent BAI; LiveCodeBench; NanoFlux; Everitt plus Cohen; Program-of-Thought; CodeAct; AlphaProof; Voyager; Lightweight Latent Verifiers; Self-Correction Bench; Sharma sycophancy; FunSearch; AlphaEvolve; Chollet ARC; ARC Prize 2025; UniCR; PRMBench; SELF-DISCOVER; arXiv:2510.04399; Eureka; Coscientist; adversarial reward-model training; and Lu concept drift (`D:238-262`).
- **SUM-D-30-087** [E] Additional works appear in per-session citations but outside that consolidated list: DGM; Soft Contamination; Gao reward-model overoptimization; Cross-Context Verification; Hubinger mesa-optimization; Constitutional AI in Small LLMs; Rewarding Doubt; the LLM Honesty Survey; NVARC NVIDIA 2025; Lessons of Developing PRMs; and ThinkPRM (`D:25,31,37,43,55,67,73,101,119,125,131,141,147`).

### Summary status-matrix rows

| Aggregate ID | Row |
|---|---|
| SUM-STATUS-D-1 | `| D-1 | PLANNED | C-15, C-21 | Phase 4H.IV / typed templates and chain |` |
| SUM-STATUS-D-2 | `| D-2 | PLANNED | D-1, B-37 | Phase 4H.V.1 / heterogeneous PPM margin |` |
| SUM-STATUS-D-3 | `| D-3 | PLANNED | D-2 | Phase 4H.V.2 / 2-verifier consensus |` |
| SUM-STATUS-D-4 | `| D-4 | PLANNED | D-1, D-3 | Phase 4H.VI / multi-signal auto-revert |` |
| SUM-STATUS-D-5 | `| D-5 | PLANNED | D-1, D-3 | Phase 4H.VII / BAI regret bound |` |
| SUM-STATUS-D-6 | `| D-6 | PLANNED | D-5 | Phase 4H.VIII.1 / temporal held-out gate |` |
| SUM-STATUS-D-7 | `| D-7 | PLANNED | D-6 | Phase 4H.VIII.2 / cross-family regression block |` |
| SUM-STATUS-D-8 | `| D-8 | PLANNED | D-1 | Phase 4H.IX / archive tamper verification |` |
| SUM-STATUS-D-9 | `| D-9 | PLANNED | D-1..D-8 | companion §§2.3, 4 / frozen-surface block |` |
| SUM-STATUS-D-10 | `| D-10 | PLANNED | D-9 | RPS.I / orchestrator route predicate |` |
| SUM-STATUS-D-11 | `| D-11 | PLANNED | D-10 | RPS.II / Mode-A checkers |` |
| SUM-STATUS-D-12 | `| D-12 | PLANNED | D-11 | RPS.III / measurable-procedure audit gate |` |
| SUM-STATUS-D-13 | `| D-13 | PLANNED | D-10, D-3 | RPS.IV / Mode-B cross-family adversary |` |
| SUM-STATUS-D-14 | `| D-14 | PLANNED | D-13 | RPS.V / inverted-framing quarantine |` |
| SUM-STATUS-D-15 | `| D-15 | PLANNED | D-11, D-12, C-17 | RPS.VI / novelty escalation |` |
| SUM-STATUS-D-16 | `| D-16 | PLANNED | D-9, D-10 | RPS.VII / Mode-E conformal refusal |` |
| SUM-STATUS-D-17 | `| D-17 | PLANNED | D-16 | RPS.VIII / refusal-rate SLA |` |
| SUM-STATUS-D-18 | `| D-18 | PLANNED | C-22, D-11 | Phase 4F extension / Mode-D effects |` |
| SUM-STATUS-D-19 | `| D-19 | PLANNED | D-2, D-3 | companion §1.13 / PPM adversarial baseline |` |
| SUM-STATUS-D-20 | `| D-20 | PLANNED | D-10, D-15 | companion §1.16 + real-problem §1 Mode C / ARC OOD |` |
| SUM-STATUS-D-21 | `| D-21 | PLANNED | D-10 | D-10 gating evidence / routing F1 |` |
| SUM-STATUS-D-22 | `| D-22 | PLANNED | D-16, D-6 | D-16 gating evidence / shifted calibration |` |
| SUM-STATUS-D-23 | `| D-23 | PLANNED | D-14 | D-14 gating evidence / detection/FPR |` |
| SUM-STATUS-D-24 | `| D-24 | PLANNED | D-7 | D-7 gating evidence / canary errors |` |
| SUM-STATUS-D-25 | `| D-25 | PLANNED | D-8, C-27 | D-8 + C-27 gating evidence / tamper suite |` |
| SUM-STATUS-D-26 | `| D-26 | PLANNED | D-1..D-8 shipped | Wave-1 dashboard exposure |` |
| SUM-STATUS-D-27 | `| D-27 | PLANNED | D-10..D-17 shipped | Wave-2 dashboard exposure |` |
| SUM-STATUS-D-28 | `| D-28 | DEFERRED | D-5, D-19, ≥20 D-5 cycles, RunPod | Phase 4H.X / PPM immunization |` |
| SUM-STATUS-D-29 | `| D-29 | DEFERRED | D-5, RunPod | companion Q4 / non-IID hierarchy or infeasibility |` |
| SUM-STATUS-D-30 | `| D-30 | DEFERRED | D-16, unidentified HF corpus | real-problem Q5 / 500-tuple cold start |` |
| SUM-STATUS-D-1 | `| D-1 | `docs/thefinalglance/D-plan/D-1_reasoning_template_schema.md` | `D:22` |
|` |
| SUM-STATUS-D-3 | `| D-3 | `docs/thefinalglance/D-plan/D-3_verification_result_aggregator.md` | `D:34` |
|` |
| SUM-STATUS-D-5 | `| D-5 | `docs/thefinalglance/D-plan/D-5_bai_thompson_promotion.md` | `D:46` |
|` |
| SUM-STATUS-D-7 | `| D-7 | `docs/thefinalglance/D-plan/D-7_cross_domain_canary.md` | `D:58` |
|` |
| SUM-STATUS-D-9 | `| D-9 | `docs/thefinalglance/D-plan/D-9_l3_frozen_surfaces.md` | `D:70` |
|` |
| SUM-STATUS-D-11 | `| D-11 | `docs/thefinalglance/D-plan/D-11_constructive_checker_registry.md` | `D:86` |
|` |
| SUM-STATUS-D-13 | `| D-13 | `docs/thefinalglance/D-plan/D-13_adversary_role_cross_family.md` | `D:98` |
|` |
| SUM-STATUS-D-15 | `| D-15 | `docs/thefinalglance/D-plan/D-15_novelty_class_labeling.md` | `D:110` |
|` |
| SUM-STATUS-D-17 | `| D-17 | `docs/thefinalglance/D-plan/D-17_honest_refusal_sla.md` | `D:122` |
|` |
| SUM-STATUS-D-19 | `| D-19 | `docs/thefinalglance/D-plan/D-19_prmbench_adversarial_gaming.md` | `D:138` |
|` |
| SUM-STATUS-D-21 | `| D-21 | `docs/thefinalglance/D-plan/D-21_mode_routing_coverage_benchmark.md` | `D:150` |
|` |
| SUM-STATUS-D-23 | `| D-23 | `docs/thefinalglance/D-plan/D-23_sycophancy_detection_benchmark.md` | `D:162` |
|` |
| SUM-STATUS-D-25 | `| D-25 | `docs/thefinalglance/D-plan/D-25_determinism_archive_tamper_suite.md` | `D:174` |
|` |
| SUM-STATUS-D-27 | `| D-27 | `docs/thefinalglance/D-plan/D-27_mode_routing_refusal_dashboard.md` | `D:188` |
|` |
| SUM-STATUS-D-29 | `| D-29 | `docs/thefinalglance/D-plan/D-29_hierarchical_bai_research.md` | `D:202` |
|` |

---

## Superseded notes (newer-plan precedence)

Losing descriptions are retained here. Target state follows the winner.

### SUP-001 — Baby-step block number for MEASURABLE_PROCEDURE

- **Target state (winner):** D-12 block 9 (eight existing + ninth)
- **Winner anchor:** D-001/D12-004, D-001/D0-047; D-12_operationalization_check_predicate.md:16-25
- **Superseded / losing description:** Summary said block 7
- **Loser anchor:** 04-D-SUMMARY_INDEX.discovery.md ~D-12 section; D-001/SD-C001
- **Precedence rule applied:** higher plan detail D-12 beats summary; D-0 agrees with D-12

### SUP-002 — D-1 archive hash formula

- **Target state (winner):** Unresolved conflict preserved: D-1 two-part (body+parent) vs D-8 three-part (+mutation_proposal_text)
- **Winner anchor:** D-002/D1-E16, D-002/D8-E03, D-002/X-C01
- **Superseded / losing description:** Neither fully supersedes; target state requires reconciliation before implement
- **Loser anchor:** D-002/D1-N02, D-002/D8-N01
- **Precedence rule applied:** conflict retained; both descriptions kept

### SUP-003 — D-26 prerequisites

- **Target state (winner):** D-26 original: D-1..D-9 shipped
- **Winner anchor:** D-001/D26-002; D-26_reasoning_template_dashboard.md:9-11
- **Superseded / losing description:** D-0 index: D-1..D-8
- **Loser anchor:** D-001/D0-030
- **Precedence rule applied:** higher plan number D-26 wins target prerequisite set; D-0 retained

### SUP-004 — D-28 status and SSB events

- **Target state (winner):** Shipping-pass: AdversarialFarmRunCompleted; local CLI/SSB legs claimed live; training/deploy deferred
- **Winner anchor:** D-002/D28-E20..E29, D-002/SUM-C12, D-002/SUM-C13
- **Superseded / losing description:** Header Status:DEFERRED and two-event design AdversarialCorpusGenerated/PpmFinetuneCompleted
- **Loser anchor:** D-002/D28-E01, D-002/D28-E17, D-002/D28-E18
- **Precedence rule applied:** explicit shipping-pass replacement beats header DEFERRED for those legs

### SUP-005 — D-29 hierarchical BAI status

- **Target state (winner):** Mixed: Path A host Rust dispatch claimed shipped; Gibbs/RunPod research closure still DEFERRED
- **Winner anchor:** D-001/D29-016, D-001/D29-023, D-001/AC-001
- **Superseded / losing description:** Summary/D-0 clean DEFERRED only
- **Loser anchor:** D-001/D0-033; 04-D-SUMMARY_INDEX.discovery.md D-29 section
- **Precedence rule applied:** later D-29 shipping annotations qualify earlier DEFERRED; not full DONE

### SUP-006 — D-30 calibration bootstrap status

- **Target state (winner):** Partial: Rust dispatch claimed wired/live; corpus/data/D22 legs DEFERRED
- **Winner anchor:** D-003/D003-D30-E012..E019, D-003/D003-DELTA-022
- **Superseded / losing description:** Summary blanket DEFERRED
- **Loser anchor:** 04-D-SUMMARY_INDEX.discovery.md D-30 section
- **Precedence rule applied:** later shipping annotations beat summary-only DEFERRED for host path

### SUP-007 — D-15 NoveltyClass attachment target

- **Target state (winner):** ClaimResult metadata (D-15 original)
- **Winner anchor:** D-004/D004-D15-013, D-004/D004-SD-005
- **Superseded / losing description:** Summary: field on CheckerResult
- **Loser anchor:** 04-D-SUMMARY_INDEX.discovery.md novelty section
- **Precedence rule applied:** higher-fidelity D-15 original beats summary projection

### SUP-008 — D-7 canary block rule

- **Target state (winner):** Unresolved: objective non-target-only vs implementation ANY-family below threshold
- **Winner anchor:** D-004/D004-D7-001, D-004/D004-D7-020, D-004/D004-D7-037
- **Superseded / losing description:** Neither deleted; both retained
- **Loser anchor:** same
- **Precedence rule applied:** conflict retained for implement reconciliation

### SUP-009 — D-17 SLA event name

- **Target state (winner):** RefusalSlaBreach + AlertKind::RefusalRateOutOfBand
- **Winner anchor:** D-003/D003-D17-E007, D-003/D003-DELTA-014
- **Superseded / losing description:** Summary AUTO_FLAG
- **Loser anchor:** 04-D-SUMMARY_INDEX.discovery.md D-17 section
- **Precedence rule applied:** D-17 original identifiers win target state

### SUP-010 — D-14 sycophancy dispatch event

- **Target state (winner):** SycophancyProbeDispatched
- **Winner anchor:** D-002/D14-E18, D-002/SUM-C09
- **Superseded / losing description:** Narrative SYCOPHANCY_PROBE
- **Loser anchor:** summary and D-0 notes
- **Precedence rule applied:** normative Rust variant wins

### SUP-011 — D-20 synthetic ARC fallback

- **Target state (winner):** v1: ContaminationSuspected/exit 1 only; synthetic generation deferred
- **Winner anchor:** D-004/D004-D20-028, D-004/D004-SD-008
- **Superseded / losing description:** Objective promised synthetic fallback if leakage flagged
- **Loser anchor:** D-20 objective
- **Precedence rule applied:** implementation/out-of-scope sections beat objective promise for v1

### SUP-012 — D-22 distributions source

- **Target state (winner):** Separate 300-file benchmark tree; HeldOutStore not wired
- **Winner anchor:** D-004/D004-D22-005, D-004/D004-SD-010
- **Superseded / losing description:** Summary: distributions sourced from D-6
- **Loser anchor:** summary D-22 section
- **Precedence rule applied:** D-22 original operational wiring wins

### SUP-013 — D-3 deceptive consensus default

- **Target state (winner):** Default AtLeastTwo can pass PPM+intrinsic with deterministic fail; code claims use RequireAll
- **Winner anchor:** D-003/D003-D3-E026, D-003/D003-DELTA-006
- **Superseded / losing description:** Summary unconditional deterministic-fail blocks promotion
- **Loser anchor:** summary D-3 section
- **Precedence rule applied:** D-3 revised scenario wins

### SUP-014 — D-2 PpmClient shape

- **Target state (winner):** Concrete struct PpmClient + PpmAdapter RuntimeAdapter
- **Winner anchor:** D-003/D003-D2-E008, D-003/D003-DELTA-001
- **Superseded / losing description:** Summary trait
- **Loser anchor:** summary D-2
- **Precedence rule applied:** D-2 original wins

### SUP-015 — D-1 staged_eval_result field name

- **Target state (winner):** staged_eval_result (+ retirement_reason)
- **Winner anchor:** D-002/D1-E19, D-002/SUM-C02
- **Superseded / losing description:** Summary eval_result projection
- **Loser anchor:** summary
- **Precedence rule applied:** D-1 original schema wins

### SUP-016 — D-26/D-27 dashboard surface counts

- **Target state (winner):** D-26: 6 commands + 6 components; D-27: 7 commands + 6 components
- **Winner anchor:** D-001/D26-021, D-001/D27-020
- **Superseded / losing description:** Summary thinner command/component lists
- **Loser anchor:** summary D-26/D-27
- **Precedence rule applied:** detailed plan originals win

### SUP-017 — D-21 Mode C benchmarkability

- **Target state (winner):** Conflict retained: D-21 demands Mode C fixtures; D-10 never returns DiscoveryLabelOnly from selector
- **Winner anchor:** D-002/X-C04, D-002/D21-N02, D-002/D10-N10
- **Superseded / losing description:** Neither deleted
- **Loser anchor:** same
- **Precedence rule applied:** conflict retained

### SUP-018 — D-19 CI enforcement

- **Target state (winner):** Informational warn-not-block until production-gating transition
- **Winner anchor:** D-003/D003-D19-E014, D-003/D003-DELTA-018
- **Superseded / losing description:** Summary direct 'gates PPM promotion' without CI nuance
- **Loser anchor:** summary D-19
- **Precedence rule applied:** D-19 original CI semantics win

---

## User-facing surfaces

Collected for dashboard DESIGN.md. Each entry references aggregate atom IDs only.

| Kind | Surface | Atom IDs | Notes |
|---|---|---|---|
| CLI | `elai template migrate` | D-002/D1-E25, D-002/D1-E38 | migrate TOML/MD prompts to Frozen archive |
| CLI | `elai archive verify` | D-002/D8-E01, D-002/D8-E10, D-001/D0-043 | flags: path, --json, --fail-fast; exit 0/1 |
| CLI | `elai ppm health` | D-003/D003-D2-E023 | three-stage health; exit 0/1 |
| CLI | `elai calibration status` | D-003/D003-D16-E015 | corpus size, tau, rates; always exit 0 |
| CLI | `elai calibration update` | D-003/D003-D16-E016 | manual tau recompute; CI needs L3_OVERRIDE:calibration_update |
| CLI | `elai bench sycophancy` | D-001/D23-009 | D-14 gate CLI |
| CLI | `elai bench mode-routing` | D-002/D21-E09 | D-10 F1 gate |
| CLI | `elai bench canary-regression` | D-002/D24-E10 | D-7 gate |
| CLI | `elai bench prmbench` | D-003/D003-D19-E011 | PPM adversarial categories |
| CLI | `elai bench tamper` | D-003/D003-D25-E015 | 10 scenarios |
| CLI | `elai bench arc-agi2` | D-004/D004-D20-013 | ARC OOD harness |
| CLI | `elai bench conformal-validate` | D-004/D004-D22-009 | tau+/-eps distributions |
| CLI | `elai farm-adv-rm run / verdict` | D-002/D28-E24 | shipping-pass claimed live |
| CLI | `elai discovery review` | D-004/D004-D15-017 | v1 print pending only; TUI deferred |
| CLI | `bn_status_report.py --plan=D` | D-001/D0-068, D-001/D26-034 | progress overlay hook |
| SlashCommand | `ArchiveVerify` | D-002/D8-E10 | dispatch elai archive verify |
| SlashCommand | `BenchArcAgi2` | D-004/D004-D20-026 |  |
| SlashCommand | `BenchConformalValidate` | D-004/D004-D22-017 |  |
| Panel | `MutationArchivePanel.svelte` | D-001/D26-011, D-001/D0-061 | archive list/filter/colors |
| Panel | `LineageGraphView.svelte` | D-001/D26-013, D-001/D0-061 | Cytoscape parent-hash DAG |
| Panel | `TripwireStatusStrip.svelte` | D-001/D26-014 | last 5 events; 3s red SLA |
| Panel | `BaiArmsTable.svelte` | D-001/D26-015 | posteriors + CI |
| Panel | `HeldOutPanel.svelte` | D-001/D26-016 | active/retired per family |
| Panel | `ArchiveVerifyGauge.svelte` | D-001/D26-017 | Clean/Tampered |
| Window | `ReasoningDrawer.svelte (D-26 tabs)` | D-001/D26-018, D-001/D26-023 | archive sub-window + drawer |
| Window | `dedicated archive sub-window` | D-001/D26-023 | D-26 reachability |
| Command | `get_archive_recent(limit)` | D-001/D26-005 |  |
| Command | `get_archive_verify_status` | D-001/D26-006 | async verify |
| Command | `get_tripwire_status(window_minutes)` | D-001/D26-007 |  |
| Command | `get_bai_arms` | D-001/D26-008 |  |
| Command | `get_canary_report(optional template_id)` | D-001/D26-009 |  |
| Command | `get_held_out_status` | D-001/D26-010 |  |
| Panel | `ModeRoutingPanel.svelte` | D-001/D27-011, D-001/D0-062 | pie/stacked modes |
| Panel | `RefusalRateGauge.svelte` | D-001/D27-012, D-001/D0-052 | healthy [5%,50%] |
| Panel | `NoveltyBreakdown.svelte` | D-001/D27-013 |  |
| Panel | `DiscoveryQueueDrawer.svelte` | D-001/D27-014 | ratify/reject |
| Panel | `SycophancyFlagStrip.svelte` | D-001/D27-015, D-001/D0-062 | quarantine links |
| Panel | `CheckerStatusPanel.svelte` | D-001/D27-016 | registry health |
| Window | `MonitorWindow (routing/refusal)` | D-001/D27-017 |  |
| Window | `ChatWindow (sycophancy)` | D-001/D27-017 |  |
| Window | `Reasoning tab (novelty/queue/checkers)` | D-001/D27-017 |  |
| Command | `get_mode_breakdown` | D-001/D27-004 |  |
| Command | `get_refusal_rate` | D-001/D27-005 |  |
| Command | `get_novelty_breakdown` | D-001/D27-006 |  |
| Command | `get_sycophancy_flags` | D-001/D27-007 |  |
| Command | `get_discovery_queue` | D-001/D27-008 |  |
| Command | `ratify_discovery` | D-001/D27-009 | requires reviewer |
| Command | `reject_discovery` | D-001/D27-010 | requires reviewer+reason |
| Flag | `ELAI_PPM_ENABLE=1` | D-003/D003-D2-E018, D-002/D10-E16 | opt-in PPM |
| Flag | `ELAI_CLOUD_MODE` | D-002/D10-E16 | alternate-family availability |
| Flag | `ELAI_SEED` | D-001/D5-028 | deterministic BAI |
| Flag | `ELAI_BAI_MODE=hierarchical` | D-001/D29-011, D-001/D29-019 | with ELAI_BAI_ENABLE=1 |
| Flag | `ELAI_BAI_ENABLE=1` | D-001/D29-019 |  |
| Flag | `ELAI_CALIBRATION_BOOTSTRAP_ENABLE` | D-003/D003-D30-E013 | bootstrap mode |
| Flag | `L3_OVERRIDE:* commit tag` | D-002/D9-E27, D-003/D003-D16-E016 | locked-key / calibration override |
| Flag | `[ci skip] commit message` | D-002/D8-E13 | skips archive-verify CI |
| Config | `[bai] L3-locked` | D-001/D5-027 | eval_budget, ci_alpha, deltas |
| Config | `[held_out] L3-locked` | D-001/D6-025 |  |
| Config | `[tripwire] L3-locked` | D-004/D004-D4-011, D-004/D004-D4-026 |  |
| Config | `[canary] L3-locked` | D-004/D004-D7-024 |  |
| Config | `[verification] L3-locked` | D-003/D003-D3-E024 |  |
| Config | `[runtime.ppm]` | D-003/D003-D2-E017 |  |
| Config | `[adversary] L3-locked` | D-003/D003-D13-E021 |  |
| Config | `[conformal] L3-locked` | D-003/D003-D16-E019 |  |
| Config | `[refusal_sla] L3-locked` | D-003/D003-D17-E011 |  |
| Config | `[sycophancy_probe] L3-locked` | D-002/D14-E20 |  |
| Config | `[observable_effect] L3-locked` | D-004/D004-D18-024 |  |
| Config | `[novelty] (referenced)` | D-004/D004-D15-023 | underspecified defaults |
| Config | `reasoning.templates (commented)` | D-002/D1-E30 | archive_path, bounded_string_max |
| Notification | `Bubble-Up RefusalRateOutOfBand` | D-003/D003-D17-E008 | B13 pattern |
| Notification | `LockedKeysTouch / L3OverrideRequested` | D-002/D9-E25, D-001/D0-044 |  |
| Notification | `SYCOPHANCY_FLAG / quarantine` | D-002/D14-E01, D-002/D14-E17, D-002/D14-E29 | blocks MEMORIZING |
| Notification | `TripwireFiredConcurrent + AutoRevert` | D-004/D004-D4-022, D-004/D004-D4-024 |  |
| Notification | `CrossDomainRegression block` | D-004/D004-D7-023, D-004/D004-D7-025 |  |
| Notification | `ClaimWithoutProcedure auditor BLOCK` | D-001/D12-014, D-001/D12-024 |  |
| Notification | `DiscoveryCandidate queue + ratify/reject` | D-004/D004-D15-014, D-001/D27-009 |  |
| Notification | `GoodhartRiskFlag` | D-004/D004-D18-022, D-001/D0-053 |  |
| StatusDisplay | `Archive verify Clean/Tampered` | D-001/D26-017 |  |
| StatusDisplay | `Refusal gauge healthy band` | D-001/D27-012, D-003/D003-D17-E006 |  |
| StatusDisplay | `Tripwire strip yellow/red` | D-001/D26-014 |  |
| StatusDisplay | `Canary report Allow/Block/InsufficientData` | D-004/D004-D7-017 |  |
| StatusDisplay | `PRMBench GatingVerdict` | D-003/D003-D19-E009 |  |
| StatusDisplay | `ARCVerdict HonestlyEvaluated/RefusalOutOfBand/Contamination` | D-004/D004-D20-011 |  |
| StatusDisplay | `ModeSelected distribution` | D-001/D27-011, D-002/D10-E24 |  |
| Hook | `scripts/locked_keys_check.sh pre-commit` | D-002/D9-E20, D-001/D0-044 |  |
| Hook | `CI archive-verify.yml` | D-002/D8-E12 |  |
| Hook | `CI locked-keys-audit.yml` | D-002/D9-E31 |  |
| Hook | `CI bench-prmbench.yml` | D-003/D003-D19-E013 |  |

---

## Closing audit block

### Ledgers merged

- Deep ledgers: **4/4** (DEEP-D-001, DEEP-D-002, DEEP-D-003, DEEP-D-004)
- Summary record: **1/1** (04-D-SUMMARY_INDEX.discovery.md)
- Total inputs: **5/5**

### Per-ledger atom counts

| Ledger | Source atoms | Interactions (IX/X/INT) | Ambiguity/Conflict (AC/ISSUE) | Summary deltas (SD/SUM/DELTA) | Inferences | Ledger total (ID-bearing) |
|---|---:|---:|---:|---:|---:|---:|
| D-001 | 319 | 10 | 50 | 27 | 6 | 412 |
| D-002 | 358 (E+N unresolved rows) | 19 | (N rows inside source) | 26 | (I inside X set) | 403 |
| D-003 | 237 | 14 | 34 | 25 | (2 I inside INT) | 310 |
| D-004 | 301 | 15 | (A/C rows inside source) | 14 | (2 I inside X) | 330 |
| Summary | 400 feature/status atoms | — | — | orientation layer | — | 400 |
| Superseded notes | — | — | — | — | — | 18 |

### Deep-ledger source-atom breakdown (traceable)

**D-001 source (319):** D0=68, D5=45, D6=52, D12=38, D23=21, D26=34, D27=33, D29=28

**D-002 source (358):** D1=57, D8=41, D9=52, D10=53, D14=47, D21=33, D24=28, D28=47

**D-003 source (237):** D2=42, D3=37, D13=32, D16=31, D17=24, D19=24, D25=23, D30=24

**D-004 source (301):** D4=52, D7=44, D11=61, D15=38, D18=44, D20=36, D22=26

- Deep source atoms subtotal: **1215**
- Deep interaction atoms: **58**
- Deep AC/ISSUE atoms (named): **84** (plus N/A/C rows inside D-002/D-004 source counts)
- Deep summary-delta atoms: **92**
- Summary-layer atoms: **400**
- Superseded notes: **18**

### Aggregate distinct record count: **1873**

### Superseded notes count: **18**

### Zero-omission statement

ZERO-OMISSION: All four deep ledgers and the family summary record were merged.
Every source-local atomic ID from each deep ledger audit is present in this aggregate
(D-001: 319 source + 6 I + 10 IX + 50 AC + 27 SD-parts; D-002: 358 source + 19 X + 26 SUM;
D-003: 237 source + 14 INT + 34 ISSUE + 25 DELTA; D-004: 301 source + 15 X + 14 SD).
Summary inventory bullets and status-matrix rows are included as SUM-* records.
Cross-file interactions, summary deltas, and ambiguity/conflict registers are namespaced by ledger.
Losing target-state descriptions are retained under Superseded notes rather than deleted.
No deep-ledger atom was compressed into prose-only form.

ZERO-DUPLICATE: Plan-local IDs remain unique within their ledger namespace.
Interactions and summary deltas cite source atoms and keep separate IDs.
Summary-layer atoms are orientation records and may restate deep content; deep records remain authoritative for implement.

### Output path

`docs/thefinalglance/feature-discovery/aggregates/D-AGGREGATE.md`

