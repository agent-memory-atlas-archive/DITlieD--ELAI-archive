# E-10: Hostile-critic structural simplifications — 3-state FSM + 3-role pipeline ablation + 3-layer RAG + MistakeBook poisoning defense
Objective: ship the structural simplifications the hostile critic flagged as load-bearing on 2026 evidence, but as MEASURED-then-decided not blindly applied: (a) iso-budget ablation between full 9-state FSM and a 3-state subset, (b) iso-budget ablation between 8-role pipeline and 3-role Self-MoA-style pipeline, (c) 3-layer RAG (BM25 + dense + ms-marco cross-encoder rerank) as a benchmark-compared challenger to the current 7-layer pipeline, (d) MistakeBook ICL-poisoning defense (structured-signals only, code stripped), (e) typed-CoT-proof-certificate (Curry-Howard) replacement for the GRC blockquote ritual or empirical proof the blockquote helps, (f) per-VRAM tier matrix collapse to ONE local tier + cloud-default with hard data.
Estimated effort: 3 sessions, ~12 h (split as FSM+role ablation; RAG challenger + MistakeBook defense; typed-CoT + tier collapse)
Pre-requisites: E-6 (iso-budget harness from task 5 reused here), E-9 (paired-bootstrap significance + ClawsBench split — needed to decide simplifications empirically), B-37 (multi-backend, DONE — cloud-default tier validation), B-41 (per-level auditor, DONE — needed for typed-CoT contract).
Parallel-safe with: E-6, E-7, E-8, E-9 (all explicitly).
Source report: workflow/improvement-sweep-2026-05-19/reports/10-critic-of-elai-design.md

## Context

The hostile critic surfaced 14 punches grounded in 2026 evidence — most defensible, several decisive. Punch 1 (FSM rigidity, 9 states / 7 evidence gates / BSP supersteps) cites that gates pass vacuously per ELAI's own audit (DETAILED_SUMMARY_01 line 511-512). Punch 2 (8 agent roles paying MAST tax 8x without 8x lift) cites Self-MoA (arXiv:2502.00674, +6.6% AlpacaEval / +3.8% MMLU-CRUX-MATH avg) and Tran & Kiela iso-budget (arXiv:2604.02460). Punch 5 (GRC blockquote echoing) cites Lanham et al (arXiv:2307.13702) CoT-faithfulness unreliability + FaithCoT-Bench (arXiv:2510.04040) — writing a citation tag does not make the reasoning grounded. Punch 6 (multi-layer pipeline 60-70% deterministic) cites the audit's own "DEEP MRR (0.206) BELOW STANDARD MRR (0.575) — violates spec monotonicity"; 4 of 7 RAG layers are runtime-dead per ELAI's own audit. Punch 7 (per-VRAM tier 8/12/16/24/48 GB) cites BenchLM 2026 25-30pp SWE-bench gap between best open-source 7B and frontier. Punch 12 (MistakeBook 150-token feed) cites ICLAttack (arXiv:2305.01219), Universal ICL Backdoors (arXiv:2401.05949), Over-prompting (arXiv:2509.13196, "LLaMA-3.2-3B exhibits weakening performance from the beginning") — small models are the worst case for in-context poisoning and ELAI's MistakeBook has no documented defense. E-10 does NOT blindly apply the critic's recommendations because that would be sycophancy in the opposite direction. E-10 ships the MEASUREMENT INFRASTRUCTURE to decide each punch empirically (iso-budget ablation for FSM and roles, ClawsBench split for RAG simplification, GSM8K-Platinum + LiveCodeBench for tier collapse) PLUS the structural defenses where evidence is decisive enough to act without further measurement (MistakeBook poisoning defense, typed-CoT-proof challenger).

## Existing state (B/C/D coverage)

- EXTENSION of E-6 (MAST-grounded failure-mode defenses) — E-6 ships iso-budget harness for full-pipeline vs single-agent + cross-family Adversary; E-10 reuses E-6's harness for FSM-state and agent-role ablations.
- DELTA on E-7 (Darwinian substrate) — typed-CoT-proof certificate becomes one of the verifier types in E-7's 2-of-N consensus (if not landed, just lives as standalone).
- DELTA on B-37 (Multi-Runtime Backend Abstraction, DONE) — cloud-default switch test exposes the per-VRAM-tier collapse decision empirically.
- DELTA on B-31 (knowledge engine 7-layer pipeline, partially shipped per audit) — E-10 ships a 3-layer challenger as a parallel pipeline, NOT a replacement; user decides on the data.
- REPLACEMENT for current MistakeBook 150-token feed — E-10 strips raw past-attempt code and feeds only {failure_category from failure_taxonomy.py, affected_files: Vec<PathBuf>, structured_diff: DiffStat, mast_mode: Option<MastMode>}.
- NEW: typed-CoT-proof-certificate verifier (Curry-Howard-style) per arXiv:2510.01069 cited by the critic.
- NEW: per-VRAM tier matrix decision sheet — empirical pass rates on GSM8K-Platinum + LiveCodeBench v6 across the 5 tier configs at iso-budget.

The critic's recommended "collapse FSM to 3 states; collapse 8 roles to 3" is NOT executed blindly. E-10 builds the ablation harness; the user decides post-data.

## Reads (mandatory before coding)

- F:\.ELAI_workflow\workflow\improvement-sweep-2026-05-19\reports\10-critic-of-elai-design.md
- F:\.ELAI_workflow\docs\thefinalglance\summary\DETAILED_SUMMARY_01_overview_and_architecture.md (lines 410-411 FSM, 454 roles, 511-512 vacuous-gates admission, 690 330-token claim, 698 isolation claim, 885-893 tier matrix)
- .elai_cc/crates/orchestrator/src/fsm.rs (FsmSubstate at line 124; baseline for ablation)
- .elai_cc/crates/orchestrator/src/agent_pool.rs (8-role spawn paths)
- .elai_cc/crates/orchestrator/src/mistake_book.rs (current feed structure; rewrite)
- .elai_cc/crates/orchestrator/src/knowledge_pipeline.rs (or equivalent 7-layer file per phase 3K audit)
- .claude/CLAUDE.md (GRC blockquote rule section — challenger lands as opt-in, default unchanged until data)
- arXiv:2503.13657 MAST (referenced)
- arXiv:2502.00674 Self-MoA
- arXiv:2604.02460 single-agent iso-budget
- arXiv:2307.13702 Lanham CoT faithfulness
- arXiv:2510.04040 FaithCoT-Bench
- arXiv:2510.01069 Typed CoT proof certificates
- arXiv:2305.01219 ICLAttack
- arXiv:2401.05949 Universal ICL Backdoors
- arXiv:2509.13196 Over-prompting (small-model vulnerability)
- arXiv:2505.06493 System Prompt Poisoning
- arXiv:2311.09948 Hijacking via Adversarial ICL
- BenchLM 2026 + Iternal 2026 (SWE-bench gap evidence for tier collapse)

## Tasks

1. FSM-state ablation harness (crates/orchestrator/src/fsm_ablation.rs, ~220 LOC). Defines FsmProfile enum {Full9State, Reduced3State (Planning, Executing, Verifying)}. The 3-state profile maps Planning <- {PromptTransform, ScopePlanning, ContextCompiling}, Executing <- {Executing, Adversary, BugReporting}, Verifying <- {Auditing, MemoryWriting, Completing}. Calls into E-6's iso-budget harness with same fixture, same model, same token budget, compares pass rate + per-MAST-mode counts. Emits SsbActionType::FsmAblationCompleted.

2. Agent-role ablation harness (crates/orchestrator/src/role_ablation.rs, ~240 LOC). Defines RoleProfile {Full8Role, Reduced3Role (Worker, Verifier, Planner)}. The 3-role profile maps Worker <- {Executor, Researcher, Designer, MediaValidator}, Verifier <- {Auditor + cross-family Adversary inline}, Planner <- {PromptEngineer + ContextCompiler}. Uses Self-MoA-style multi-sample with N=3 from one strong model for "Researcher" + "Designer" calls inside Worker. Compares against Full8Role on same iso-budget fixture. Emits SsbActionType::RoleAblationCompleted.

3. 3-layer RAG challenger (crates/orchestrator/src/rag_challenger.rs, ~280 LOC). Pipeline: BM25 -> dense (existing vectors) -> ms-marco cross-encoder rerank top50->top5. Runs in parallel to the existing 7-layer; both pipelines feed the same query stream; metrics are compared via Mean Reciprocal Rank on a fixed eval set. ONNX cross-encoder model staged at .elai_cc/models/onnx/ms-marco-rerank/ — paths from core::paths::models_dir_path per B-1. Emits SsbActionType::RagChallengerMrrMeasured.

4. MistakeBook ICL-poisoning defense (crates/orchestrator/src/mistake_book.rs, ~150 LOC DELTA replacing raw-text feed). New entry type: StructuredFailureSignal { failure_category: FailureTaxonomy (from .claude/tools/failure_taxonomy.py reference), affected_files: Vec<PathBuf>, structured_diff: DiffStat { lines_added, lines_removed, files_changed, churn_score }, mast_mode: Option<MastMode> (from E-6), timestamp_ms: u64, content_hash: blake3 [u8;32] }. Raw code from past attempts is NEVER stored. Add SsbActionType::MistakeBookSignalStored. Add contract test that asserts no raw past-attempt code appears in any context_pack assembled for the executor.

5. Typed-CoT-proof certificate verifier (crates/orchestrator/src/cot_proof.rs, ~260 LOC). For claims of the form "X implies Y because Z", parses the reasoning into a typed certificate: claim: Proposition, evidence: Vec<Witness>, derivation_step: InferenceRule, verifier_signature: blake3. The certificate is mechanically checked: every claim must have at least one Witness with provenance = (CodeSpan | SpecCite | TestResult | ExternalSource), every inference step must be one of the registered InferenceRule variants (ModusPonens, AndElim, CaseSplit, Refutation, Citation). Lands as OPT-IN: GRC blockquote remains default; user enables typed-CoT via env var ELAI_TYPED_COT=1; eval compares with iso-budget harness. References arXiv:2510.01069 in module doc.

6. Per-VRAM tier matrix collapse decision sheet (crates/cli/src/tier_eval_cmd.rs, ~200 LOC). Runs GSM8K-Platinum + LiveCodeBench v6 + tau2-bench (small slice) on the local 7B at each of the 5 documented tier configs (8/12/16/24/48 GB) at iso-budget vs cloud Sonnet at same iso-budget; reports per-tier pass rate + per-tier cost-normalized score. Emits SsbActionType::TierMatrixMeasured. CLI: `elai tier eval --suites gsm8k-platinum,livecodebench-v6,tau2 --budget 100k`.

7. Cloud-default switch test (crates/orchestrator/tests/cloud_default_switch.rs, ~120 LOC). Hard-toggle the runtime orchestrator default to cloud Sonnet; run a representative B-N fixture end-to-end; record (a) wall-clock, (b) total cost USD-equivalent, (c) pass rate, (d) prompt-cache hit rate (per critic's caching note — Anthropic compat layer in B-37 lacks prompt caching by design; this test surfaces what we lose). Emits SsbActionType::CloudDefaultMeasured.

8. GRC-blockquote empirical eval (crates/orchestrator/tests/grc_blockquote_lift.rs, ~140 LOC). Cross-product {ELAI_GRC_BLOCKQUOTE=on, ELAI_GRC_BLOCKQUOTE=off} x {fixture set of 30 tasks}. Measures pass rate + auditor BLOCK count + per-MAST mode. If off-condition >= on-condition within paired-bootstrap p<0.05, surface this empirical result; do NOT remove the rule from CLAUDE.md without user approval per "no removal or simplification without explicit user confirmation" hard rule. Emits SsbActionType::GrcBlockquoteLiftMeasured.

9. SSB-driven decision dashboard (crates/dashboard-v2/src/lib/components/CriticDecisionPanel.svelte, ~280 LOC). Renders 6 panels, one per ablation: FSM, Roles, RAG, MistakeBook (already structural, shows defense activations), TypedCoT, Tier. Each panel shows the empirical comparison + paired-bootstrap p; provides a "Recommend SIMPLIFY" badge if challenger >= incumbent at p<0.05. The user makes the final call from the dashboard; E-10 does NOT auto-simplify.

10. Documentation block in CLAUDE.md (CLAUDE.md DELTA ~30 LOC). Adds a section "Critic Decisions" pointing to the dashboard and listing the 6 ablations with their current empirical state {NotMeasured, Insufficient, Inconclusive, RecommendSimplify, RecommendKeep}. Updated by post-bench post-hook from SSB.

11. Anthropic prompt-caching wiring inspection (crates/orchestrator/src/cache_check.rs, ~120 LOC). Per critic Punch 4 ("the 1600-2600 tokens disappears at 90%+ cache hit rate"), surfaces ELAI's actual cache hit rate against cloud Anthropic API on representative workload; emits SsbActionType::PromptCacheHitRateMeasured. If hit rate < 50%, surface red flag — the 330 vs 2600 token claim is moot under good caching.

12. CLI report: `elai critic report` (~100 LOC). Aggregates all 6 ablation results + paired-bootstrap p values + recommendations in a single text report; written to workflow/reasoning/critic-decisions-{ts}.md. Reads-only; no auto-mutation.

## WIRING_CONTRACT

produces_symbols:
  - orchestrator::fsm_ablation::run_fsm_ablation
  - orchestrator::role_ablation::run_role_ablation
  - orchestrator::rag_challenger::ThreeLayerRag
  - orchestrator::mistake_book::StructuredFailureSignal
  - orchestrator::cot_proof::TypedCotCertificate (+ verify())
  - orchestrator::cache_check::measure_cache_hit_rate
  - cli::tier_eval_cmd::tier_matrix_eval
  - cli::critic_report::write_decision_report
  - SsbActionType::FsmAblationCompleted
  - SsbActionType::RoleAblationCompleted
  - SsbActionType::RagChallengerMrrMeasured
  - SsbActionType::MistakeBookSignalStored
  - SsbActionType::TypedCotVerified
  - SsbActionType::TierMatrixMeasured
  - SsbActionType::CloudDefaultMeasured
  - SsbActionType::GrcBlockquoteLiftMeasured
  - SsbActionType::PromptCacheHitRateMeasured
consumes_symbols:
  - crates/orchestrator/src/iso_budget_harness.rs (E-6) — reused by FSM + Role + GRC eval
  - crates/orchestrator/src/bench_significance.rs (E-9) — paired-bootstrap for all comparisons
  - crates/orchestrator/src/mistake_book.rs (rewritten feed surface)
  - crates/orchestrator/src/knowledge_pipeline.rs (parallel 3-layer challenger path)
  - crates/dashboard-v2/src-tauri::get_critic_decision_state
reachable_from:
  - cli::main::main `elai tier eval`, `elai critic report`, `elai bench ablation [fsm|role|grc|rag|cot|cache]`
  - dashboard-v2/src-tauri::get_critic_decision_state
stub_ban:
  - "mistake_book.*raw_code|past_attempt_code" — raw past-attempt code absent in MistakeBook feed
  - "ELAI_TYPED_COT=0" hardcoded enable-and-default in non-test source (must be opt-in)
  - "FsmProfile::Reduced3State" set as production default in non-test source (must be ablation-only)
  - any "auto-remove GRC blockquote" path — removal requires explicit user approval per CLAUDE.md hard rule
frontend_pair: crates/dashboard-v2/src/lib/components/CriticDecisionPanel.svelte
observable_effect:
  trigger: "elai bench ablation fsm --fixtures fixtures/critic_30.toml --budget 5000"
  expect_ssb: "SsbActionType::FsmAblationCompleted within 30 min; both Full9State and Reduced3State pass-rates present; paired-bootstrap p reported"
  expect_artifact: "stdout 2-row table (Full9State vs Reduced3State); dashboard CriticDecisionPanel FSM tile updates with empirical numbers + recommendation badge"

## DETERMINISTIC_CHECK

```bash
# 1. MistakeBook raw code stripped
rg "past_attempt_code|raw_attempt_text" .elai_cc/crates/orchestrator/src/mistake_book.rs
# Expected: zero hits (the rewrite removes these fields entirely).

# 2. MistakeBook context_pack contract: no raw code present
cargo test -p elai-orchestrator mistake_book::tests::context_pack_strips_raw_code
# Expected: PASS.

# 3. Typed-CoT certificate verifier runs
cargo test -p elai-orchestrator cot_proof::tests::valid_certificate_passes
cargo test -p elai-orchestrator cot_proof::tests::missing_witness_fails
# Expected: both PASS.

# 4. FSM ablation both profiles run
cargo run -p elai-cli -- bench ablation fsm --fixtures fixtures/critic_smoke.toml --budget 2000 --json | jq '.full_9state, .reduced_3state'
# Expected: two non-null result objects.

# 5. Role ablation cross-product
cargo run -p elai-cli -- bench ablation role --fixtures fixtures/critic_smoke.toml --budget 2000 --json | jq '.full_8role, .reduced_3role'
# Expected: two non-null result objects.

# 6. RAG challenger MRR compared
cargo run -p elai-cli -- bench ablation rag --eval-set fixtures/rag_eval.toml --json | jq '.seven_layer_mrr, .three_layer_mrr'
# Expected: two non-null floats.

# 7. Tier matrix eval
cargo run -p elai-cli -- tier eval --suites gsm8k-platinum --budget 50k --json | jq '.tiers | length'
# Expected: at least 5 entries.

# 8. Prompt cache hit rate measured
cargo run -p elai-cli -- bench ablation cache --json | jq '.cache_hit_rate'
# Expected: float between 0 and 1.

# 9. GRC blockquote eval cross-product
cargo run -p elai-cli -- bench ablation grc --fixtures fixtures/grc_30.toml --budget 100k --json | jq '.on_pass_rate, .off_pass_rate, .p_value'
# Expected: three non-null fields.

# 10. blake3 used for content hashes (anti-stale)
rg "blake2|sha2" .elai_cc/crates/orchestrator/src/cot_proof.rs .elai_cc/crates/orchestrator/src/mistake_book.rs
# Expected: zero hits.
```

## ACCEPTANCE

- [ ] FSM ablation harness runs Full9State and Reduced3State at iso-budget on the same fixture set; paired-bootstrap p reported.
- [ ] Role ablation harness runs Full8Role and Reduced3Role (Self-MoA-style N=3 sampling for Researcher/Designer); paired-bootstrap p reported.
- [ ] 3-layer RAG challenger (BM25 + dense + ms-marco rerank) runs in parallel to 7-layer; MRR compared; ONNX cross-encoder staged at canonical models path.
- [ ] MistakeBook rewritten: no raw past-attempt code in any context_pack; only StructuredFailureSignal; contract test enforces.
- [ ] Typed-CoT certificate verifier present, opt-in via ELAI_TYPED_COT=1; verifies witnesses + inference rules; baseline GRC blockquote default unchanged.
- [ ] Tier matrix eval runs 5 tier configs on 3 suites (GSM8K-Platinum, LiveCodeBench-v6, tau2-bench small slice); per-tier cost-normalized score reported.
- [ ] Cloud-default switch test surfaces wall-clock, USD cost, pass rate, prompt-cache hit rate side-by-side with local default.
- [ ] GRC blockquote lift measured at iso-budget; result recorded in SSB; rule NOT removed regardless of result (per CLAUDE.md hard rule).
- [ ] Critic decision dashboard panel renders 6 tiles with paired-bootstrap p + recommendation badges; user can read but no auto-mutation.
- [ ] `elai critic report` writes a decision document at workflow/reasoning/critic-decisions-{ts}.md.
- [ ] CLAUDE.md gains a "Critic Decisions" section referencing the dashboard panel and the ablation result file.
- [ ] No .unwrap() in new non-test code; blake3 over blake2/sha2; native async fn in traits.
- [ ] All new Python deps verified via pypi.org/pypi (anti-slopsquatting).

## Tests to add

- crates/contracts/tests/mistake_book_no_raw_code_contract.rs: scan every context_pack assembly path, refuse any raw past-attempt code
- crates/orchestrator/tests/cot_proof_valid_chain.rs: valid certificate verifies; missing witness fails; unknown inference rule fails
- crates/orchestrator/tests/fsm_ablation_smoke.rs: both profiles run on a 5-task fixture
- crates/orchestrator/tests/role_ablation_smoke.rs: 8-role and 3-role both run; Self-MoA N=3 invocation verified
- crates/orchestrator/tests/rag_challenger_mrr_diff.rs: 3-layer MRR comparable to 7-layer on a fixed eval set
- crates/orchestrator/tests/tier_matrix_eval_smoke.rs: 5 tiers x 1 suite produces 5 result entries
- crates/orchestrator/tests/cloud_default_switch.rs: cloud-default runs end-to-end with cost + cache + wall-clock recorded
- crates/orchestrator/tests/grc_blockquote_iso_budget.rs: cross-product runs; p-value reported; no rule mutation
- crates/dashboard-v2/src/lib/components/CriticDecisionPanel.test.ts: 6 tiles render + recommendation badge logic correct
- crates/orchestrator/tests/cache_check_measurement.rs: cache hit rate computed against a mocked Anthropic response stream

## Files touched

- CREATE: crates/orchestrator/src/fsm_ablation.rs (~220 LOC)
- CREATE: crates/orchestrator/src/role_ablation.rs (~240 LOC)
- CREATE: crates/orchestrator/src/rag_challenger.rs (~280 LOC)
- CREATE: crates/orchestrator/src/cot_proof.rs (~260 LOC)
- CREATE: crates/orchestrator/src/cache_check.rs (~120 LOC)
- CREATE: crates/cli/src/tier_eval_cmd.rs (~200 LOC)
- CREATE: crates/cli/src/critic_report.rs (~100 LOC)
- CREATE: crates/dashboard-v2/src/lib/components/CriticDecisionPanel.svelte (~280 LOC)
- CREATE: .elai_cc/models/onnx/ms-marco-rerank/ — staged ONNX model dir (model file pulled at install time, NOT committed)
- MODIFY: crates/orchestrator/src/mistake_book.rs (~150 LOC delta — replace raw-text with StructuredFailureSignal)
- MODIFY: crates/orchestrator/src/ssb.rs (+9 SsbActionType variants, ~90 LOC delta)
- MODIFY: crates/cli/src/main.rs (~30 LOC to register `bench ablation`, `tier eval`, `critic report` subcommands)
- MODIFY: crates/dashboard-v2/src-tauri/src/lib.rs (~60 LOC for get_critic_decision_state)
- MODIFY: CLAUDE.md (~30 LOC, "Critic Decisions" section referencing dashboard + decision report)

## Citations / Grounding

- arXiv:2503.13657 (MAST) — 14 failure modes; grounds role-ablation hypothesis that 8-role pays MAST tax
- arXiv:2502.00674 (Self-MoA) — heterogeneous mixing -6.6% AlpacaEval; grounds N=3 sampling pattern for reduced 3-role profile
- arXiv:2604.02460 (single-agent iso-budget) — grounds iso-budget gating on any simplification claim
- arXiv:2307.13702 (Lanham et al CoT faithfulness) — CoT does not reliably reflect computational process
- arXiv:2510.04040 (FaithCoT-Bench) — unfaithfulness persists across counterfactual probes; grounds typed-CoT challenger
- arXiv:2510.01069 (Typed CoT proof certificates) — Curry-Howard mechanical check; grounds cot_proof.rs verifier design
- arXiv:2305.01219 (ICLAttack) — backdoor activation via demonstration context alone; grounds MistakeBook defense
- arXiv:2401.05949 (Universal ICL Backdoors) — backdoor in instruction-tuned LLMs without fine-tuning
- arXiv:2509.13196 (Over-prompting) — "LLaMA-3.2-3B exhibits weakening performance from the beginning"; grounds small-model worst-case
- arXiv:2505.06493 (System Prompt Poisoning) — wrong demonstrations bias subsequent outputs
- arXiv:2311.09948 (Hijacking via Adversarial ICL) — few-shot prompt injection on instruction-tuned LLMs
- BenchLM 2026 / Iternal 2026 — 25-30pp SWE-bench gap between best open-source 7B and frontier; grounds tier collapse measurement
- Anthropic OpenAI compat docs — prompt caching not supported in compat layer; grounds cache_check.rs design
- focused.io/lab/agent-benchmarks-measure-the-harness — ClawsBench scaffold-dominates evidence; grounds rag_challenger paired evaluation

## Open questions / risks

- The hostile critic recommended several simplifications; E-10 ships ablation infrastructure, NOT auto-simplification, because "No removal or simplification without explicit user confirmation" is a CLAUDE.md hard rule. Risk: if the data clearly supports simplification, the user may delay deciding and the engineering surface grows. Mitigation: dashboard renders recommendations with badges, but the user is the final gate.
- Typed-CoT proof certificate is OPT-IN. Risk: nobody enables it, the new code rots. Mitigation: include in the default `elai bench ablation cot` invocation so it gets exercised by the bench harness.
- The 3-layer RAG challenger requires an ONNX cross-encoder model staged at .elai_cc/models/onnx/ms-marco-rerank/; if not present, the challenger silently degrades. Mitigation: deterministic check + acceptance criterion that asserts the model file exists before tests pass.
- Per-VRAM tier matrix collapse is sensitive to cost model; cost = (electricity for local) + (cloud token spend). E-10 ships a per-tier USD-equivalent computation but the rate sheet (USD/kWh, USD/Sonnet-token) must be set per-user. Default rates are us-east-coast 2026 averages — UNCONFIRMED beyond rough order of magnitude.
- "Ungrounded:" critic flagged GRC blockquote as theater; E-10 ships the empirical comparison but does NOT remove the rule (CLAUDE.md hard rule). Risk: if the eval shows off-condition matches or exceeds on-condition at p<0.05, the user might want removal; that's a separate user-decision step, not E-10's job.
- Prompt-cache hit-rate measurement requires the cloud Anthropic API path; not all users will have cloud access. Mitigation: cache_check.rs runs only when cloud route is configured; otherwise emits CacheCheckUnavailable.
- E-10 assumes E-6 (iso-budget harness) and E-9 (paired-bootstrap significance + ClawsBench) ship first. If E-10 lands first, dependencies must be co-landed (~+200 LOC overhead).
- "It's too clever by half" (critic's closing line) is itself a normative claim; E-10 does NOT endorse blanket simplification, just enables empirical decisions. The user's "no removal without explicit user confirmation" rule is honored.

## Next

- E-10 unblocks USER-DRIVEN architectural decisions on FSM-state count, agent-role count, RAG pipeline depth, MistakeBook surface, GRC blockquote enforcement, and per-VRAM tier matrix
- Each "RecommendSimplify" badge from the dashboard becomes a separate user-approved follow-on (not bundled into E-10 to avoid auto-simplification creep)
- E-9's ClawsBench split harness composes with E-10's RAG challenger: any RAG simplification decision MUST run ClawsBench harness-vs-model attribution to confirm the simplification is the model-effect contributor, not a harness artifact
- Typed-CoT proof certificate becomes a candidate verifier in E-7's 2-of-N consensus rule (D-3); if landed and shown effective at p<0.05, propose as the third verifier
- MistakeBook structural defense is shipped immediately (no measurement gate) because the ICL-poisoning evidence (arXiv:2305.01219 + arXiv:2401.05949 + arXiv:2509.13196) is decisive
