# DEEP-D-001 feature discovery

## Scope and evidence rules

This is the deep pass for manifest chunk `DEEP-D-001`. The only original-plan sources read were the eight files in the exact manifest block reproduced below. `04-D-SUMMARY_INDEX.discovery.md` was read only as an orientation/comparison layer; it is not evidence for a deep-source feature unless a deep source independently states the feature. No companion design, other `D-N` plan, implementation file, paper interface, test fixture, or neighboring deep chunk was opened.

Notation:

- `[E]` — explicit in the named original source.
- `[I]` — inference formed by joining explicit facts; never promoted to source fact.
- `[A]` — ambiguous or underspecified.
- `[C]` — conflicting statements, either within one source or across the eight processed sources.
- Category tags: `CAP` capability/action/surface; `WF` workflow/state/lifecycle; `DATA` schema/data; `CFG` flag/default/config; `INT` integration/event/dependency; `SEC` permission/security/privacy/compliance; `FAIL` failure/fallback/recovery; `DET` determinism/idempotency/concurrency; `OBS` observability/evidence; `TEST` test/acceptance/done gate; `CON` constraint/non-goal; `STATUS` status/deferred/closure.

An **atomic discovery unit** is one uniquely identified, source-local contract statement below. A unit may preserve a complete type or function signature (including all stated fields) as one schema/action contract; subordinate fields are not silently re-counted. Cross-source interactions, inferences, ambiguities, conflicts, and summary deltas only reference these units and are not counted again.

## Exact processed manifest and fingerprint verification

Manifest location: `DEEP-SOURCE-MANIFEST.md:454-467` (the next heading, `### DEEP-D-002`, begins at line 469).

> `### DEEP-D-001`
>
> Family **D**; totals: **8 files**, **1861 logical lines**, **89099 bytes**; target status: **within 1,800–2,800**.

| Chunk ID | Class | Lines | Bytes | SHA-256 | Exact absolute path | Verification |
|---|---|---:|---:|---|---|---|
| DEEP-D-001 | numbered/index original | 370 | 18851 | `dbd623d56e2c4323dc74478e919e6a4d5f590e323a3b32cbae405077a3d79db5` | `docs/thefinalglance/D-plan/D-0_INDEX.md` | exact match |
| DEEP-D-001 | numbered/index original | 292 | 14108 | `9db078102d56d8f0b8853a6e462c73f11400bc8770795ff3d68f32fb17e13a16` | `docs/thefinalglance/D-plan/D-5_bai_thompson_promotion.md` | exact match |
| DEEP-D-001 | numbered/index original | 337 | 17697 | `81ae25b3c2f47e8701fa437655da7ab9a143bff0e71dddb15bcf678ddd1ac4cb` | `docs/thefinalglance/D-plan/D-6_held_out_task_store.md` | exact match |
| DEEP-D-001 | numbered/index original | 255 | 11790 | `d41df45d74ab950e688f5c18ee7563d719f1f724bf7be510d3462c7625f000e5` | `docs/thefinalglance/D-plan/D-12_operationalization_check_predicate.md` | exact match |
| DEEP-D-001 | numbered/index original | 108 | 3991 | `02a583b93872475b9beca844488a064fe627d5996f5e381247f0537d722569cf` | `docs/thefinalglance/D-plan/D-23_sycophancy_detection_benchmark.md` | exact match |
| DEEP-D-001 | numbered/index original | 188 | 8588 | `2326b8557579ba697a6ca74c77e35fb165d9609b9a4f2ecbb0dcd4ba77689f0d` | `docs/thefinalglance/D-plan/D-26_reasoning_template_dashboard.md` | exact match |
| DEEP-D-001 | numbered/index original | 179 | 8078 | `f1d7e75035b7d259f321e86030b0e897357159a60e385900f8a823deea7206bc` | `docs/thefinalglance/D-plan/D-27_mode_routing_refusal_dashboard.md` | exact match |
| DEEP-D-001 | numbered/index original | 132 | 5996 | `279fc0b00f752321d019a73ffc718888a778d55019fba8e745b92ee5e406a5ba` | `docs/thefinalglance/D-plan/D-29_hierarchical_bai_research.md` | exact match |
| **Verified total** | **8 files** | **1861** | **89099** | — | — | **exact match** |

Verification used `sha256sum` and `wc -lc` against the exact absolute paths. There were zero missing files and zero fingerprint, line-count, byte-count, or aggregate-count mismatches.

## Per-file explicit ledgers

### `D-0_INDEX.md` — 370/370 lines — 68 atomic units

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

Coverage note: all session rows, status-note rows, summary statistics, and reconciliation text were represented. The terminal separator/end marker carries no feature. No explicit PII, secret-redaction, retention, network-privacy, or telemetry-export policy appears in this index.

### `D-5_bai_thompson_promotion.md` — 292/292 lines — 45 atomic units

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

Coverage note: no retry/backoff protocol or privacy/PII handling is specified. Dependency addition is permission-gated, configuration is L3-locked, empty input and zero-sample recovery are explicit, and concurrency is explicit only through `ArchiveChain` locking.

### `D-6_held_out_task_store.md` — 337/337 lines — 52 atomic units

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

Coverage note: D-6 contains evaluation-secret/history protection but no user-PII, secret-redaction, network-authentication, or telemetry-export policy. Source provenance is represented in `TaskSource`; no retry/backoff behavior is defined for GitHub/arXiv/HuggingFace access or TOML parse failures.

### `D-12_operationalization_check_predicate.md` — 255/255 lines — 38 atomic units

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

Coverage note: claim/candidate payloads and human-review evidence have explicit size bounds but no redaction, retention, access-control, or privacy policy. Timeout exists; retry/caching is absent and caching is explicitly deferred.

### `D-23_sycophancy_detection_benchmark.md` — 108/108 lines — 21 atomic units

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

Coverage note: the source specifies human labeling but no reviewer identity schema, adjudication, inter-rater agreement, fixture-text privacy, report retention, retry policy, or explicit nonzero failure exit code.

### `D-26_reasoning_template_dashboard.md` — 188/188 lines — 34 atomic units

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

Coverage note: the dashboard exposes template excerpts and operational evidence but specifies no authentication, authorization, redaction, privacy, stale-data, command-error, retry, or disconnected-state UI. Its explicit safety controls are bounded archive reads, asynchronous verification, and generated-binding drift checks.

### `D-27_mode_routing_refusal_dashboard.md` — 179/179 lines — 33 atomic units

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

Coverage note: reviewer presence is required, but identity authentication, authorization roles, dual control, audit-signature strength, claim/evidence redaction, retention, and network/privacy policy are not defined. Read failures, write conflicts, retries, stale subscriptions, and disconnected UI are also unspecified.

### `D-29_hierarchical_bai_research.md` — 132/132 lines — 28 atomic units

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

Coverage note: this source has no deterministic reproducibility contract, retry behavior, privacy rule, or RunPod data-handling policy. Its default-flat flag is the only explicit runtime recovery/safety default.

## Explicit cross-source interactions

These are source-backed joins; they reference source-local atoms and add zero to the atomic count.

1. **Promotion pipeline.** D-3 consensus supplies D-5 binary observations; D-5 promotes/retire templates through D-1 archive; D-6 interposes a held-out veto before promotion; D-7 adds a family veto; D-4 can auto-revert the same archive; D-8 later exposes lineage verification. Evidence: D0-007..D0-012, D5-011/D5-022..D5-024/D5-036/D5-045, D6-022/D6-023/D6-036/D6-045/D6-048.
2. **Concurrency boundary.** D-5 and D-6 can proceed around D-7’s read/signal work, and both explicitly exclude parallel execution with D-4 because archive/evaluation state overlaps. D-5 also calls D-6 parallel-safe, although D-6 separately requires D-5 to be `DONE`; that unresolved ordering tension is recorded below. Evidence: D5-003, D6-002.
3. **Deterministic promotion evidence.** C-15/D-1 seeded randomness drives D-5; C-15 seeded time drives D-6. D-5 demands same-seed byte identity; D-6 uses mock/seeded time and idempotent storage checks. Evidence: D5-017/D5-028/D5-036..D5-042, D6-006/D6-036/D6-039/D6-042.
4. **Audit-state claim path.** D-12 consumes D-11’s checker registry, blocks `AUDITING -> MEMORIZING` without a procedure, and shares the checker-result hook with downstream D-15. D-13 handles claims lacking a Mode-A checker. Evidence: D0-015..D0-019, D12-001/D12-003/D12-005/D12-017/D12-028/D12-038.
5. **Sycophancy proof-to-UI path.** D-14 produces the probe/flag, D-23 quantitatively gates it, and D-27 streams flags with quarantine links; D-27’s operator review is distinct from D-23’s human fixture labeling. Evidence: D0-018/D0-027/D0-031, D23-001/D23-005..D23-018, D27-007/D27-015/D27-024/D27-030.
6. **Wave-1 dashboard.** D-26 reads D-1/D-4/D-5/D-6/D-7/D-8 state, applies bounded/asynchronous query rules, and is parallel-safe with D-27. Evidence: D26-001/D26-002/D26-005..D26-027.
7. **Wave-2 dashboard and human disposition.** D-27 reads D-10..D-17 events but also writes ratify/reject decisions through `SsbWriter`; it is therefore both an observation surface and a human-governance mutation surface. Evidence: D27-004..D27-010/D27-020..D27-025.
8. **Hierarchical extension.** D-29 is an opt-in extension of D-5 with two environment gates and flat default; benchmark closure compares directly with the D-5 flat baseline. Evidence: D5-044, D29-008..D29-011/D29-018..D29-023.
9. **Evaluation-store secrecy.** D-6 composes temporal/9-gram filtering with Arbor-style dev/test isolation and DeNovo-style `.git` history removal; runtime leakage blocking remains explicitly owned by H-47. Evidence: D6-003/D6-019/D6-049..D6-052.
10. **Plan status/dashboard accounting.** D-0 defines `DONE` as implementation + cloud smoke + threshold, while D-26 adds the `--plan=D` status-report hook. D-29’s embedded “shipped” note does not independently satisfy the D-0 `DONE` definition because its closure benchmark remains pending. Evidence: D0-003/D0-035/D0-066/D0-068, D29-016/D29-020/D29-024..D29-027.

## Inferences (not explicit source facts)

- **I-001 `[I]`** The intended promotion transaction is likely evaluation -> posterior update -> held-out veto -> archive transition, but neither D-5 nor D-6 defines an atomic transaction/rollback boundary across SSB, TOML, and `ArchiveChain`. Derived from D5-022..D5-024 and D6-022..D6-024/D6-045.
- **I-002 `[I]`** D-26 is operationally read-only because all six named commands are getters, whereas D-27 is the first processed dashboard plan with explicit write commands. The sources do not state a formal read-only capability boundary. Derived from D26-005..D26-010 and D27-009/D27-010.
- **I-003 `[I]`** D-6’s `ManuallyExcluded` status probably represents a curator action distinct from auto-retirement, but no method/event/authorization flow for setting it is specified. Derived from D6-012..D6-016.
- **I-004 `[I]`** The D-23 label `Robust` appears intended to correspond to D-14/D-23’s `Convergent` output, but the source does not declare an enum mapping. Derived from D23-005/D23-007/D23-018.
- **I-005 `[I]`** Default-flat D-29 operation is a compatibility fallback that keeps existing promotion live when hierarchical mode is unset, but no automatic fallback after a hierarchical runtime failure is stated. Derived from D29-011/D29-019/D29-022.
- **I-006 `[I]`** D-27’s required reviewer string is attribution, not authentication; the plan does not establish that the supplied reviewer is a verified human identity. Derived from D27-009/D27-010/D27-023.

## Ambiguous, conflicting, or unresolved source detail

1. **D-29 lifecycle/status is internally conflicting `[C]`.** Header, deterministic-check section, acceptance list, and “anticipated files” say deferred/pending (D29-002, D29-024..D29-027); the wiring contract says Path A shipped and Rust dispatch is live (D29-016/D29-023). Safest status is **mixed/partially implemented claim, research closure not done**, not simply `DONE` or simply untouched.
2. **D-0 versus D-29 status conflicts `[C]`.** The index calls D-29 deferred/research-grade (D0-033/D0-064/D0-066); the later D-29 contract claims Path A shipped (D29-016). No date/version marker explains the update.
3. **Gibbs evidence is unresolved `[A/C]`.** D29-017/D29-020 say the Python Gibbs benchmark is RunPod-pending, while D29-023 expects a live round with `gibbs_iterations > 0`. It is unclear whether Rust performs a separate Gibbs-like update or the observable contract is aspirational.
4. **Flat-module identity drifts `[A]`.** D-5 plans `bai.rs`/`BaiPromotionEngine` (D5-021/D5-043); D-29 names a D-5 `darwinian_bai` baseline and default discount factor (D29-018). Rename/history is not explained.
5. **D-26 prerequisite differs `[C]`.** D-0 says D-26 requires/exposes D-1..D-8 (D0-030/D0-061); D-26 says D-1..D-9 must be shipped (D26-002). D-26 does not name a D-9 data surface, so D-9 may be a governance prerequisite rather than a rendered capability.
6. **D-26 objective exceeds its task ledger `[A]`.** D26-001 promises PPM-score and consensus-rule display, but its six commands/components do not name a PPM/consensus query or field. Existing `ReasoningDrawer` data may supply them, but that is not stated.
7. **Archive verification ownership is split `[A]`.** D26-006 attributes the invoked method to D-1, while D26-022 says `verify_chain` spans D-1+D-8. Exact owning module/API is unresolved.
8. **Held-out “append-only” conflicts with rewrite `[C/A]`.** D0-041 calls the task layout append-only, but D6-015 rewrites TOML status on retirement. The no-delete rule (D6-037) is explicit; append-only may mean no file deletion or may refer only to audit history.
9. **Semantic/structural contamination defenses are designed but not implemented `[A]`.** D6-004 requires semantic fingerprints, D6-005 names structural-prior randomization, and D6-012 has `SemanticContamination`, but no semantic detector or structural-randomization task/config/test is specified. Only n-gram Bloom logic is executable in this plan.
10. **Bloom absence degrades silently `[A]`.** D6-027 says skip n-gram checking until data exists; no SSB warning, operator alert, or promotion fail-closed rule is stated for this protection gap.
11. **Store error/idempotency semantics are incomplete `[A]`.** Acceptance says load/append/retire are idempotent (D6-042), but behavior for duplicate `TaskId`, partially written TOML, parse failure, date-path mismatch, or concurrent writers is absent.
12. **Seed-count gates differ `[A]`.** D6-026/D6-043 require 25 tasks; shell presence gate accepts `>=20` (D6-041); observable contract expects `>=20` retirements from a 25-task uniform-pass simulation (D6-039). These may be smoke versus final gates, but are not labeled that way.
13. **Archive task payload shape is underspecified `[A]`.** D6-006 says an `ArchiveEntry` carries `task_id` rather than `template_id`, but no union/schema amendment is defined in D-6.
14. **D-12 block-number conflict is resolved only by deep evidence `[C]`.** The orientation summary says “block 7,” but both D0-047 and D12-004/D12-027 explicitly say block 9 after eight existing blocks. The original-plan evidence therefore favors block 9.
15. **D-12 schema-bump atomicity is unclear `[A]`.** D0-045 expects one atomic D-10+D-11+D-12 bump; D12-021 independently specifies `57 -> 58`. It may mean sequenced per-session bumps, but “atomic” is unexplained.
16. **Claim detection cannot literally guarantee no false negatives `[A]`.** D12-019 defines keyword/prose heuristics while D12-020/D12-030 say false negatives are not allowed. No formal grammar, exhaustive claim AST, or fallback classifier is supplied.
17. **Transition compatibility is temporary but unbounded `[A]`.** `measurable_procedure` is `Option` during transition (D12-009) while the L3 rule is no opt-out (D12-006/D12-030); no migration version/date identifies when `None` ceases to be accepted outside claim steps.
18. **D-23’s 70/30 rationale is numerically tense `[C]`.** Context cites observed sycophancy at 23-33% (D23-003), while decisions say a fixture population with 70% sycophancy “matches” Sharma (D23-010). The split is useful for power but does not match that prevalence literally.
19. **D-23 outcome vocabulary drifts `[A]`.** Fixtures use `Sycophancy/Robust` (D23-005), execution uses “flagged-divergent” (D23-007), and Cloud B expects `Convergent` (D23-018); mapping is not typed.
20. **D-23 failure exit is incomplete `[A]`.** CLI exit 0 is specified only for a passing gate (D23-016); non-passing exit code and report persistence on failure are not defined.
21. **D-5 has multiple benchmark scales `[A, not necessarily conflict]`.** D0-009 says 50 rounds within the published bound; D5-033 defines 500 pulls/100 trials/bound x2; D5-042 defines a 50-cycle cloud bootstrap. They appear to be distinct index, integration, and cloud gates and must not be collapsed into one threshold.
22. **D-5 Successive Rejects output semantics are terse `[A]`.** D5-019 returns “ordered survivors,” while repeated elimination conventionally ends with one survivor; D5-024 applies it to a bottom quartile. Exact returned ordering and budget allocation across `retire_round` are not fully specified.
23. **D-27 rejection lifecycle is asymmetric `[A]`.** Ratification explicitly updates the queue file (D27-009); rejection only explicitly writes an event (D27-010), although the UI implies disposition. Whether rejected entries leave the pending queue is not stated.
24. **D-27 human governance is attribution-only `[A]`.** Reviewer is required, but authentication, reviewer authorization, multiple reviewers, and conflict-of-interest controls are absent; multi-reviewer flow is explicitly deferred (D27-023/D27-032).
25. **Polling semantics remain partly open `[A]`.** D-26 fixes tripwire reaction at 3 seconds (D26-031), but archive, verification, D-27 sycophancy, disconnection, stale state, and retry intervals are not quantified.
26. **Dashboard data exposure is ungoverned `[A]`.** D26-012 and D27-007/D27-014..D27-016 display template, claim, evidence, reviewer, and failure snippets without an explicit permission/redaction model.
27. **D-0’s aggregate status is internally tense `[C/A]`.** D0-066 reports 27 in-scope slots and three deferred slots but also says all 30 authored files are `PLANNED`; the status legend treats `PLANNED` and `DEFERRED` as distinct. It is unclear whether “all PLANNED” describes document authorship rather than session status.
28. **The D-5 file inventory drifts `[A/C]`.** D0-040 expects both `bai.rs` and `promotion_arms.rs`, while the detailed D-5 tasks, wiring contract, and files-touched list place the engine in `bai.rs` and never name `promotion_arms.rs` (D5-014..D5-043). No consolidation/rename note resolves the extra index file.
29. **The held-out event inventory grows without reconciliation `[A]`.** D0-041 lists only `HELD_OUT_REFRESH` and `HELD_OUT_AUTO_RETIRE`; D6-024 makes `HeldOutPromotionGate` a third schema event. This may be a detailed-plan expansion, but D-0’s “expected additions” list was not updated.
30. **D-6 has several compatible but distinct time-scale gates `[A, not necessarily conflict]`.** The index cloud proof retires one 18-month-old task (D0-010), D-6 units exercise 7/5-month cases (D6-031), and cloud/integration evidence advances across 12 months (D6-033/D6-039/D6-044). They must remain separate checks rather than one normalized age threshold.
31. **Auto-refresh cadence is unbound `[A]`.** D6-003 says refresh every `N` sessions, but `AutoRefreshPolicy`, config, reachability, deterministic checks, and acceptance define no `N`, scheduler, or invocation trigger (D6-020/D6-025/D6-035..D6-046).
32. **The n-gram detector’s corpus interface is internally underspecified `[A]`.** The source signature accepts `training_corpus_hashes: &Vec<Blake3Hash>` while the algorithm also says to load `ngram-bloom.bin`; it does not say which input is authoritative or how exact Jaccard union size is recovered from Bloom membership (D6-019).
33. **BabyStepSpec’s existing-block identifier drifts `[A]`.** D12-004 quotes the current block as `FILES_PLAN`, while the same source calls the C-31 foundation `FILE_PLAN` (D12-007/D12-026). The ninth block is consistently `MEASURABLE_PROCEDURE`, but the pre-existing block name is not.
34. **Checker timeout disposition is undefined `[A]`.** D12-030 requires `run_procedure` to honor `checker.timeout_secs`, but no `CheckerResult`, divergence value, event payload, retry policy, or auditor reason is specified for timeout.
35. **D-12’s deterministic shell does not directly cover two acceptance helpers `[A]`.** D12-032’s grep checks omit `diff_expected` and `detect_claim_in_baby_step`, although D12-033 requires both behaviors. Scoped tests may cover them, but the deterministic presence gate does not prove that explicitly.
36. **D-23 fixture-count gates differ `[A]`.** Tasks and acceptance say 100 fixtures (D23-005/D23-016), while the shell gate accepts any count `>=100` (D23-015). Extra fixtures would change prevalence and metrics, but their handling is not defined.
37. **D-23 serialization contracts are absent `[A]`.** The plan defines report fields and label counts but not the JSON fixture fields needed to replay D-14, nor the payload schemas for the generic `BenchmarkStarted`/`BenchmarkResult` events (D23-005..D23-007/D23-014/D23-019).
38. **D-23’s parallel-safe range self-includes `[A]`.** D23-002 says it is parallel-safe with D-19..D-25, a literal range containing D-23 itself. The likely intent is “the other benchmark sessions,” but exclusions are not enumerated.
39. **Archive parent identifiers drift `[A]`.** `MutationArchivePanel` detail links `parent_id` (D26-012), whereas `LineageGraphView` edges use `parent_hash` (D26-013). The source does not state whether they are aliases or require a lookup.
40. **Canary “last N” is not parameterized `[A]`.** `get_canary_report(template_id: Option<TemplateId>)` promises the given template or “last N” reports (D26-009), but its signature/config defines no `N` or limit default.
41. **Two D-26 navigation surfaces lack explicit wiring `[A]`.** `ArchiveVerifyGauge` opens a full report and the contract claims a dedicated archive sub-window (D26-017/D26-023), but the six commands and files-touched list define neither a full-report command nor a sub-window integration file (D26-021/D26-033).
42. **D-26 mixes polling and subscription without ownership rules `[A]`.** Individual panels poll, while archive/tripwire updates also reuse real-time SSB subscription (D26-014/D26-017/D26-019/D26-027). Cache precedence, deduplication, and reconnect behavior are not specified.
43. **The refusal healthy band has two authorities `[A]`.** The component task states `[5%,50%]` (D27-012), while the stub ban requires thresholds loaded from D-17 config (D27-025). D-27 does not state that 5/50 are D-17 defaults or what the UI shows when config differs.
44. **Discovery disposition events lack schema/lifecycle contracts `[A]`.** D27-009/D27-010 write `DiscoveryRatified` and `DiscoveryRejected`, but this source defines no SSB variant/payload/schema bump, round-trip test, idempotency key, duplicate-click behavior, or rejection queue-file update.
45. **D-29 closure paths overlap on equality `[C]`.** Path A accepts hierarchical regret `<=` flat (D29-009), while Path B is “equal or worse” (D29-010); equality satisfies both, so the branch decision is not mutually exclusive.
46. **D-29 mode/event failure schemas are incomplete `[A]`.** The source names `HierarchicalBaiRoundCompletedPayload` and the unset-mode `Flat` default (D29-015/D29-022), but gives no payload field types beyond observable references to `gibbs_iterations`/`SelectionBasis`, and no behavior for an invalid `ELAI_BAI_MODE` value or hierarchical runtime failure.
47. **“Online updates” conflicts with an unqualified update API `[A]`.** D29-014 claims `hierarchical_update`, while D29-028 defers “online hierarchical updates.” The source does not distinguish a batch/offline update from the deferred online behavior.
48. **D-5’s LUCB optional return is incomplete `[A]`.** D5-020 declares `Option<TemplateId>` but specifies neither empty-input nor no-competitor behavior; the explicit empty-input rule applies only to `thompson_select` (D5-018).
49. **D-5/D-6 ordering and parallelism conflict `[C/A]`.** D5-003 calls D-6 parallel-safe, but D6-002 requires D-5 to be `DONE` before D-6. No milestone-level dependency explains how both can be true.
50. **Empty-set n-gram overlap is undefined `[A]`.** D6-017/D6-037 require empty extraction to return safely, while D6-018 defines overlap as `intersection/union`; no zero-union convention is given for two inputs with no n-grams.

## Summary-layer additions and corrections

Comparison target: `04-D-SUMMARY_INDEX.discovery.md` (orientation only). Each delta is supported by deep atoms above.

### Additions absent from or materially thinner than the summary

1. **D-0 exact operational ledger.** Add all schema-version expectations, exact SSB identifiers, directories, files, ports/GPU mapping, fixtures, baseline numbers, deferred unblock reasons, and `bn_status_report.py --plan=D` from D0-035..D0-068. The summary intentionally did not read D-0 and only surfaced its filename at summary lines 732/775.
2. **D-5 selection policy is now concrete.** Add Beta(1,1) initialization, staged 10/50/200 evaluation, credible-lower-bound versus promoted median, per-domain deltas, bottom-quartile retirement, LUCB formula, gamma-ratio sampling, empty-arm/zero-sample behavior, and MSI-as-secondary-only policy (D5-005..D5-010/D5-014..D5-027/D5-034/D5-037). This resolves most of summary uncertainty item 7 (`04-D...:752`).
3. **D-5 reproducibility/evidence is stronger.** Add same-seed byte identity, exact 10-arm rates, 500 pulls, 100 trials, bound x2, >=85% best-arm selection, >=15 tests, and exact event payloads (D5-026/D5-028..D5-033/D5-038..D5-042).
4. **D-5 security/fallback detail.** Add user approval before `statrs`, incomplete-beta fallback, config immutability, graceful empty input, and archive-lock concurrency exclusion with D-4 (D5-003/D5-015/D5-018/D5-037).
5. **D-6 schema and lifecycle.** Add complete task/outcome/source/status/reason schemas, dual indexes, active-only query, rewrite-on-retire behavior, signed shared-subset regression, gate reason, and exact event payloads (D6-009..D6-024).
6. **D-6 thresholds resolved.** “Uniformly passing” means age `>6 months`, pass rate `>0.95`, and at least 20 evaluations; regression allows `delta >= -5%`; n-gram contamination is 9-gram Jaccard `>0.3` against a Blake3 Bloom (D6-019..D6-025). This resolves most of summary uncertainty item 8 (`04-D...:753`).
7. **D-6 degraded mode and seed composition.** Add the empty-Bloom skip behavior, exact 25-task source mix, test tables, and the distinction between 20-file smoke presence versus 25-file acceptance (D6-026..D6-033/D6-039..D6-046).
8. **D-6 evaluation secrecy amendments.** Add Arbor `B_dev/B_test` detached-worktree prior art and mandatory `.git` destruction/reinitialization for repo-derived fixtures; runtime leakage control remains H-47 (D6-049..D6-052). This is a material security addition to summary lines 568-583/618-621.
9. **D-12 full operationalization contract.** Add the complete bounded `MeasurableProcedure`, TOML parser/error format, actual-vs-expected tri-state diff, keyword claim detector, per-checker timeout, human-review prompt, fresh execution/no cache, exact payloads, and eight-test floor (D12-009..D12-038).
10. **D-23 benchmark implementation.** Add exact 70-positive/30-negative human-labeled fixtures, report confusion-matrix fields/formulas, `GatingVerdict`, CLI/CI reachability, benchmark events, dated report artifact, and `Convergent` cloud expectation (D23-005..D23-019).
11. **D-26 surface is much larger.** Add `get_bai_arms`, `get_canary_report`, `get_held_out_status`, plus `TripwireStatusStrip`, `BaiArmsTable`, `HeldOutPanel`, and `ArchiveVerifyGauge`; add filters, colors, click behavior, async verification, bounded recent reads, and exact three-second tripwire SLA (D26-005..D26-032). The summary listed only three commands and two named new components at lines 383-399/529-531.
12. **D-27 is a governance UI, not only charts.** Add `get_discovery_queue`, `ratify_discovery`, `reject_discovery`, `DiscoveryQueueDrawer`, and `CheckerStatusPanel`, including reviewer-required write actions, quarantine links, configurable thresholds, placement, and evidence snippets (D27-008..D27-025/D27-030). The summary listed only four commands/four components at lines 401-415/529-531.
13. **D-29 contains concrete host-side symbols.** Add exact Rust types/functions/line anchors, `HierarchicalBaiRoundCompleted`, two environment gates, default-flat behavior, task-runner branch, and the explicit claim that Path A host dispatch shipped while the Gibbs comparison remains pending (D29-013..D29-023).
14. **D-29 model proposal is concrete.** Add Beta-per-arm/Gamma-hyperprior/Gibbs shape and the five-family, ten-arm, within-family-correlated, 1000-pull benchmark (D29-003/D29-007/D29-008), resolving part of summary uncertainty item 20 (`04-D...:765`).

### Corrections or qualifications to summary claims

1. **Baby-step block number.** Correct summary line 187’s “block 7” to **block 9** for this deep evidence set: D0-047 and D12-004/D12-027 agree on eight existing blocks plus ninth `MEASURABLE_PROCEDURE`.
2. **D-23 comparison mode.** Summary line 345 says D-23 runs probe-enabled versus probe-disabled. The processed D-23 original does **not** specify that ablation; it replays every fixture through D-14 and compares the flag to labels (D23-007). Treat enabled/disabled as summary-only unless another original proves it.
3. **D-23 provenance wording.** Summary line 343 says “Anthropic/Sharma”; this original says Sharma et al. public probes and does not name an Anthropic dataset identifier (D23-001/D23-004).
4. **D-26 lineage target.** Summary line 390 says it cross-links to a “baby-step archive.” The original specifies an `ArchiveEntry` parent-hash DAG and flips to `MutationArchivePanel` detail, not an explicit baby-step-archive cross-link (D26-012/D26-013).
5. **D-26 prerequisites.** Qualify summary line 399/matrix line 652 (`D-1..D-8`) with the original’s stronger D-1..D-9 shipped prerequisite (D26-002), while retaining the conflict with D0-030.
6. **D-26 poll interval.** Summary uncertainty line 764 says polling latency is unquantified. The tripwire acceptance SLA is explicitly **3 seconds** (D26-031); other polls remain unquantified.
7. **D-26 PPM/consensus display.** Summary lines 391-393 mirror the objective, but the detailed original task/command/component contract lacks a PPM or consensus query. Keep these as objective-level requirements with an implementation-surface ambiguity (D26-001 versus D26-005..D26-021).
8. **D-29 status.** Summary lines 432-444/655/766 present D-29 only as deferred and assert zero shipped features. The original still has deferred acceptance, but explicitly claims Path A host-side Rust dispatch shipped (D29-016/D29-023). Correct representation is mixed evidence/partial productization with research closure pending, not cleanly `DONE`.
9. **D-29 closure outcome.** Summary treats Path A and Path B as both open. The embedded wiring comment states Path A was the outcome and Path B was not (D29-016), although unchecked acceptance prevents treating the overall session as closed.
10. **D-6 privacy/security coverage.** Summary lines 618-621 say no distinct privacy feature. That remains true for user PII, but D6-051/D6-052 add a distinct confidentiality/leakage safeguard for evaluation fixtures: remove recoverable Git history at build time.
11. **Held-out auto-retirement.** Summary lines 99/601 phrase six-month uniform passing broadly. Deep rules require all of strict age, `>0.95`, and `>=20` evaluations (D6-020/D6-021); a TTL alone does not retire.
12. **D-5 benchmark wording.** Summary line 600 reports a 50-round Audibert gate. Preserve that index gate (D0-009), but also record the original’s separate 500-pull/100-trial/bound-x2 integration test and 50-cycle cloud bootstrap (D5-033/D5-042).
13. **Dashboard component/command counts.** Replace any interpretation that D-26 has only three commands/two components or D-27 only four commands/four components: their originals require 6+6 and 7+6 respectively (D26-021/D26-029; D27-020/D27-028).

## Zero-omission and zero-duplicate audit

### Coverage totals

| Source | Manifest lines read | Atomic units | First-last ID |
|---|---:|---:|---|
| `D-0_INDEX.md` | 370/370 | 68 | D0-001..D0-068 |
| `D-5_bai_thompson_promotion.md` | 292/292 | 45 | D5-001..D5-045 |
| `D-6_held_out_task_store.md` | 337/337 | 52 | D6-001..D6-052 |
| `D-12_operationalization_check_predicate.md` | 255/255 | 38 | D12-001..D12-038 |
| `D-23_sycophancy_detection_benchmark.md` | 108/108 | 21 | D23-001..D23-021 |
| `D-26_reasoning_template_dashboard.md` | 188/188 | 34 | D26-001..D26-034 |
| `D-27_mode_routing_refusal_dashboard.md` | 179/179 | 33 | D27-001..D27-033 |
| `D-29_hierarchical_bai_research.md` | 132/132 | 28 | D29-001..D29-028 |
| **Total** | **1861/1861** | **319** | **319 unique IDs** |

Audit assertions:

- **Manifest completeness:** 8/8 exact files, 1,861/1,861 logical lines, 89,099/89,099 bytes, and 8/8 hashes verified.
- **Section completeness:** for every detailed D-N source, `Objective`, prerequisites/parallel safety, `Context`, `Reads`, paper-ingestor section where present, every numbered task, `WIRING_CONTRACT`, `DETERMINISTIC_CHECK`, every acceptance checkbox, files touched, out-of-scope, and next/amendment material were classified. D-0’s entire session and status-note inventories were classified.
- **Structural-line handling:** headings, blank lines, Markdown separators, code-fence delimiters, and terminal “End” text were read but are not feature atoms.
- **Duplicate control:** every counted unit has one unique source-local ID. Repeated concepts across D-0 and a detailed plan are retained as distinct provenance records; interaction, conflict, inference, and summary-delta sections reference IDs and add no second count.
- **Explicitness control:** source claims that may be stale or false (especially D-29 “shipped”) remain `[E]` only as claims and are also marked `[C]`; they were not independently verified against implementation.
- **Boundary control:** `04-D-SUMMARY_INDEX.discovery.md` was the only non-manifest content read and was used solely to orient and calculate deltas. No other original chunk or companion source was read.
- **Independent completion re-audit:** all 1,861 assigned lines were re-read against all 319 atoms; source hashes/counts were re-run, every D-0 closure/citation anchor was reconciled, inaccurate explicit wording was corrected, and a second unique-ID/coverage audit found no omitted or duplicate source-local unit.
- **Final atomic count:** **319 explicit source-local discovery units; 0 missing IDs; 0 duplicate IDs; 6 separately labeled inferences; 50 ambiguity/conflict records; 27 summary additions/corrections.**
