F-7 CBR template ranking (Laplace-utility) - decision doc (session s-60824, FINAL build unit)

Scope
HALF B (outcome-utility CBR ranking of pipeline templates) ONLY. HALF A (safety-rule lattice completeness proof: lattice.rs, SafetyLatticeVerified, FormalVerifyAction::Lattice, the Z3/mutable=0 enumeration) is a SEPARATE E-53 amendment, strictly sequenced after E-53's z3/kani/verus stubs resolve, RunPod-pending = DEFERRED. Do NOT produce any lattice / Z3 / SafetyLatticeVerified / formal_verify symbol this session. SSB bump is +1 (HALF B alone), NOT +2.

What I will implement
Replace the pipeline-template confidence gate's raw first-try ratio with a Laplace-smoothed outcome utility, expose a ranking CLI + dashboard panel, and emit a template_utility_ranked beacon - so a low-sample template stops being trusted as much as a high-sample one.

Key technical decision
The compiled-pipeline cache holds exactly ONE template per (concept_class, plan_skeleton_hash) key (compiled_pipeline.rs:453) - it is NOT a multi-candidate set. So the real W-3 wiring is a GATE-THRESHOLD SWAP at CompiledExecutor::try_execute (compiled_pipeline.rs:499): gate on laplace_utility instead of the raw first_try_success_rate. The "rank multiple candidates" framing has no cache to serve it; ranking surfaces only in the CLI sort across all stored templates + the telemetry beacon. first_try_success_rate is KEPT as the recorded baseline (W-2 no-removal); the dashboard shows utility vs first_try_ratio side-by-side.

Expected test outcome
cargo check --workspace --tests GREEN (incl the batch-updated SCHEMA==205 pins); f7_template_utility_contract asserts laplace_utility(95,100) > laplace_utility(1,1) (a 1/1 template scores 2/3, not 1.0) and the smoothing monotonicity; live `elai template-ranking list --json` emits a real template_utility_ranked row + prints utility alongside first_try_ratio.

Unknowns to verify empirically
1. Whether the persisted CompiledPipeline fields (first_try_success_rate, use_count) are populated enough on a fresh process for the CLI to show non-empty buckets (the in-memory observation_log is ephemeral; the CLI reads the persisted store).
2. Exact count of contract tests hard-asserting SCHEMA_VERSION(_PUBLIC)==204 (grep + batch-update ALL to 205 in the same pass).
3. Live all_test_payloads / TOTAL_VARIANTS / golden ordinal (RE-COUNT; baseline 481->482, SCHEMA 204->205, all_test_payloads 401->402).

================================================================================
GROUNDED CORRECTIONS (the context-refresh found the plan wrong in 3 places)
================================================================================
1. UTILITY SOURCE: the plan says "compute utility from EXISTING SSB outcome events." FALSE - the SSB compiled-pipeline events (CompiledPipelineFired, TemplateCrystallized) carry NO per-firing pass/fail outcome. The only per-template success signal is ExecutionOutcome.success in the in-memory TemplateCrystallizer.observation_log (ephemeral) and the PERSISTED CompiledPipeline.first_try_success_rate + use_count. F-7 computes utility from the persisted fields (successes = round(first_try_success_rate * use_count), total = use_count); the CLI reads the CompiledPipelineStore. No new outcome-capture is added.
2. NOT A RANKING (single-key cache): try_execute does an exact single-key cache.get, not a between-candidates selection. The behavioral change is the gate-threshold swap (raw ratio -> Laplace utility). The CLI sort across the whole store is where "ranking" is observable.
3. block_ratio + coverage have NO producer. Zero-filling them is a CSQ-class footgun (a consumer would read 0.0 block_ratio as "no blocks = good" when it really means "unmeasured"). DECISION: DROP block_ratio + coverage from the v1 payload; document them as future signals requiring a per-template block-outcome producer. Ship Laplace over success/total only. (Springdrift +10.7% was the only number these were meant to chase, and it is dropped.)

================================================================================
template_utility.rs (the detector)
================================================================================
LAPLACE_ALPHA = 1.0, LAPLACE_BETA = 1.0 (ELAI-own add-one smoothing constants). laplace_utility(successes:u32, total:u32) -> f32 = (successes + ALPHA) / (total + ALPHA + BETA) (total==0 -> the prior ALPHA/(ALPHA+BETA)=0.5). laplace_utility_from_fields(first_try_success_rate:f32, use_count:u32) -> f32 (successes = (rate*use_count).round() as u32, clamped <= use_count). struct TemplateUtility { concept_class, plan_skeleton_hash, utility, first_try_ratio, sample_n }. Pure/deterministic, no clock/random. Register pub mod template_utility in lib.rs (~:254).

================================================================================
SSB template_utility_ranked (single flat beacon, 8-site cascade) + F-8 doc fold
================================================================================
ONE SsbActionType::TemplateUtilityRanked (flat, no Kind) mirroring F-5 StalenessQuarantineBroadcast / F-8 SymbolicConflictDetected. TemplateUtilityRankedPayload { concept_class:String, plan_skeleton_hash:u64, utility:f32, first_try_ratio:f32, sample_n:u32, chosen_pipeline_id:String }. 8 ssb.rs sites (enum + SQL CHECK + as_sql_str + from_str + SsbPayload variant + action_type() + payload struct + test_payload) + all_test_payloads +1 + SCHEMA_VERSION 204->205. action_type_lock.rs: TOTAL_VARIANTS 481->482 + ALL_VARIANTS + exhaustive_match_guard. golden: alpha-insert template_utility_ranked + renumber. Ssb::new INSIDE rt.block_on.
FOLD the F-8 cosmetic doc-WARN while in ssb.rs: SymbolicConflictPayload fingerprint doc-comment (~:25549) says "kind+key+candidate_id" but the impl (symbolic_conflict.rs:248 conflict_fingerprint) hashes kind+key+detail - fix the DOC to say kind+key+detail. DOC-ONLY; do NOT change the hash (would break the F-8 golden/contract).
BATCH-UPDATE: grep all contract/unit tests that hard-assert SCHEMA_VERSION(_PUBLIC) == 204 (the ie flagged 40+) and update them to 205 in the SAME EXECUTING pass (a runtime assertion cargo check will not catch, but it must be correct).

================================================================================
W-3 wiring + CLI + dashboard + grounding
================================================================================
W-3 consumption: CompiledExecutor::try_execute (compiled_pipeline.rs:499) confidence gate uses laplace_utility_from_fields(first_try_success_rate, use_count) instead of the raw ratio (the production behavioral change). Real caller chain: pipeline_builder.rs build_with_compiled_executor:656 -> try_execute:694. The crystallizer post-execute hook (task_runner.rs:3608 -> record_execution:3642) keeps feeding observation_log.
CLI (W-3 entrypoint + W-5 trigger): NEW `elai template-ranking list [--json]` (template_ranking_cmd.rs + args.rs TemplateRankingAction + main.rs dispatch): read CompiledPipelineStore, compute laplace_utility per template, sort desc, emit a template_utility_ranked row per template (or a top-N), print JSON with utility + first_try_ratio side-by-side. Ssb::new INSIDE rt.block_on (the no-reactor gotcha). Default-ON; any ELAI_TEMPLATE_CBR_* env is a kill-switch (default ON), no default-off ENABLE.
Dashboard (W-4/W-6): commands.rs get_template_ranking + lib.rs invoke_handler (:422) + TemplateRankingPanel.svelte (NEW) mounted in MonitorWindow.svelte (:35) + types.ts DTO + stub.ts + cross_lang pair. Render utility vs first_try_ratio side-by-side (the before/after).
Grounding (sg): Springdrift +10.7% P@4 is UNGROUNDED (synthetic/n=1/unreproduced, CI overlaps RAG baseline) - DROPPED, never a target. Laplace/add-one smoothing is a grounded standard technique. F-7 gates on the WIRING done-gate + the smoothing-correctness assertion, NOT a numeric uplift. The measured P@k delta vs the first-try baseline is a documented GATED checkbox (>=100 labelled sessions).
GATED CHECKBOXES: (i) measured P@k delta (>=100 sessions). (ii) block_ratio/coverage signals (need a per-template block-outcome producer). (iii) HALF A safety-lattice completeness proof (E-53 amendment, RunPod-pending).
HAZARDS: 40+ SCHEMA==204 runtime pins (batch-update). compiled_pipeline.rs is F-6-touched (acquire_learned_skill is in task_runner.rs:3713, NOT compiled_pipeline) - re-grep. Build-isolate the concurrent knowledge RRF breakage (deep.rs/pipeline.rs) via git-stash; do NOT edit/commit.

================================================================================
SLICE A3 as-built (W-3 gate-threshold swap) - session s-60824
================================================================================
Anchor RE-GREPPED fresh: the gate was `if pipeline.first_try_success_rate < self.confidence_threshold` at compiled_pipeline.rs:499 (not the exact text the plan quoted; the plan's `first_try_success_rate < confidence_threshold` had no `self.`/`pipeline.` qualifiers). As-built: introduced `let gate_utility = laplace_utility_from_fields(pipeline.first_try_success_rate, pipeline.use_count);` and the gate now reads `if gate_utility < self.confidence_threshold`. Added `use crate::template_utility::laplace_utility_from_fields;` (module is `pub mod` at lib.rs:259).
THRESHOLD SEMANTICS DECISION: `confidence_threshold` (= COMPILED_EXECUTOR_CONFIDENCE_THRESHOLD const) is UNCHANGED. The gate now compares the SMOOTHED utility against the same const. This is deliberately EQUAL-OR-STRICTER for low samples and never more permissive: a 1/1 template raw=1.0 -> smoothed=0.667 (now decompiles under a high threshold where the raw rate previously passed); a 95/100 raw=0.95 -> smoothed~0.941 (barely moves); as use_count grows smoothed -> raw, so high-evidence behavior is unchanged. The const was NOT loosened to "compensate" for the smoothed scale (that would defeat the point of distrusting thin evidence). first_try_success_rate field KEPT intact (W-2 no-removal; recorded baseline). try_execute signature UNCHANGED.
What the gate now compares: laplace_utility_from_fields(pipeline.first_try_success_rate, pipeline.use_count) < self.confidence_threshold.
VERIFY: cargo check -p elai-orchestrator --tests GREEN (43s, only pre-existing unused-import/dead-code warnings in unrelated test modules; no errors).

================================================================================
SLICE A5 as-built (W-4/W-6 dashboard TemplateRanking DRAWER) - session s-60824
================================================================================
PLAN DIVERGENCE (intentional, per slice override): the F-7 plan said "TemplateRankingPanel.svelte mounted in MonitorWindow.svelte (:35)". The A5 slice instruction OVERRODE this to a NEW DRAWER (TemplateRankingDrawer.svelte) NOT MonitorWindow, because a concurrent session owns the windows. As-built mirrors the F-5 StalenessQuarantineDrawer per-surface DrawerKey pattern exactly (the closest sibling: also a single flat beacon, also default-ON / no ENABLE gate).
FILE-PLACEMENT DECISION: created a DEDICATED commands_f7.rs (NOT inline in the 10k-line commands.rs the plan text named) because every recent F-drawer (commands_f5/f9/f10) uses a dedicated per-surface file + the cross_lang_diff pair expects a discrete file pair. commands.rs is reserved for the F-8 ReasoningDrawer-fold style; F-7 is its own drawer key.
9-FILE SYNC (all RE-GREPPED fresh against current state):
  1. commands_f7.rs (NEW): TemplateRankingRow DTO {seq,ts,concept_class,plan_skeleton_hash:u64,utility:f32,first_try_ratio:f32,sample_n:u32,chosen_pipeline_id} + TemplateRankingResponse{rows} + get_template_ranking (recent_rows_of_type(SsbActionType::TemplateUtilityRanked) -> serde_json::from_value -> match SsbPayload::TemplateUtilityRanked; map_err + continue-on-bad-row, no .unwrap) + 2 serde-field tests. DTO INCLUDES plan_skeleton_hash + seq (task DTO list omitted them; added because seq is the every-sibling ordering key and plan_skeleton_hash is half the cache key, useful in the table; harmless u64->JSON number).
  2. lib.rs: `mod commands_f7;` after commands_f5 + `commands_f7::get_template_ranking,` in invoke_handler after commands_f5 entry.
  3. types.ts: 'template-ranking' DrawerKey union member + TemplateRankingRow + TemplateRankingResponse interfaces.
  4. DrawerHost.svelte: import + else-if chain entry.
  5. layout/Drawer.svelte: DRAWER_LABELS 'template-ranking':'TEMPLATE RANKING'.
  6. layout/Rail.svelte: DrawerKeyUpper 'TEMPLATE-RANKING' + drawerItems row (Icon: Sparkles -- already imported in Rail; NO new lucide import, anti-slopsquat).
  7. stub.ts: TemplateRankingResponse type import + get_template_ranking fallback {rows:[]}.
  8. TemplateRankingDrawer.svelte (NEW): renders utility (after) vs first_try_ratio (before) SIDE-BY-SIDE per W-6, client-side sort by utility DESC (the CLI is the canonical ranking; drawer re-sorts for top-down read), a Δ=utility-first_try_ratio badge (highlights low-sample distrust), summary "N smoothed-down" count, callInvoke + 5s poll mirroring F-5.
  9. cross_lang_diff.py: registered template_ranking_drawer pair (commands_f7.rs <-> TemplateRankingDrawer.svelte).
VERIFY: cargo check -p elai-dashboard-v2 --tests GREEN (61s, EXIT=0; only pre-existing orchestrator dead-code warnings, none in dashboard crate). svelte-check 0 ERRORS / 18 WARNINGS (all warnings pre-existing in unrelated files; none in the 6 touched frontend files). cross_lang_diff --files: template_ranking_drawer pair CLEAN + tauri_commands CLEAN. Residual warnings ipc_types (stub.ts<->ipc-types/lib.rs) + types.ts-not-paired are PRE-EXISTING broad wildcard rules that fire on ANY dashboard stub.ts/types.ts edit (identical to the F-5 baseline), NOT F-7-introduced drift.
