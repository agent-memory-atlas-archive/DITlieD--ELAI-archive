//! Golden-file lock for SsbActionType::as_sql_str() wire tags.
//!
//! Enforces the SSB wire-protocol invariant that the set of action_type strings
//! emitted by the orchestrator never silently shrinks, gets renamed, or has its
//! ordering relative to the lock file scrambled.
//!
//! Three tests:
//!   - `action_type_lock_golden_matches_runtime` -- byte-equality between
//!     the alphabetically-sorted as_sql_str() values and the body lines of
//!     tests/action_type_lock.golden.
//!   - `action_type_lock_count_is_locked` -- runtime variant count equals
//!     `TOTAL_VARIANTS`, catching renames that preserve count.
//!   - `regen_golden_file` (#[ignore]'d helper) -- rewrites the golden from
//!     the current runtime values. Run with `--ignored --nocapture` after a
//!     deliberate, reviewed schema change.
//!
//! Enumeration strategy: ALL_VARIANTS is a hand-locked array in declaration
//! order. `exhaustive_match_guard` is an internal `match` on `SsbActionType`
//! whose every arm returns `()`; rustc requires exhaustivity, so adding a new
//! variant to the enum without updating this test produces a hard compile
//! error pointing back here. No `strum` dep needed and no mutation of
//! ssb.rs (lockstep file -- kept READ ONLY by E-45 SP2 scope).
//!
//! Append-only contract -- see the header of action_type_lock.golden.

use std::collections::BTreeSet;
use std::fs;
use std::path::PathBuf;

use elai_orchestrator::ssb::{BashRedirectedKind, BeliefStateEventKind, BranchWorkspaceEventKind, CacheEconomicsEventKind, ClaimScopeMeasuredKind, ContextPrunedKind, ContextTransformKind, CritiqueLensFindingTag, DirectiveExecutedKind, GovernancePrimitiveFiredKind, GrammarGateEventKind, MemoryGuardEventKind, MutationBudgetedRunKind, ObjectiveSealedKind, PlaybookEventKind, PromptCompiledKind, PromptHardeningAppliedKind, ReproPackKind, RoleContractAppliedKind, RowChainSealedKind, SkillLibraryEventKind, SsbActionType, TddPreflightBlockedKind, TestIntegrityEventKind, ToolFilterEventKind, ToolSequenceVerdictKind, TrajectoryAbortedKind, TriLayerVerdictKind, VibeGateBlockedKind};

/// Total number of SsbActionType variants currently locked. Bumping this
/// requires regenerating action_type_lock.golden in the SAME commit; the
/// `_count_is_locked` test below holds the line.
// E-6 (SCHEMA_VERSION 148): 420 -> 424 for the 4 MAST-grounded multi-agent
// failure-mode defense variants (AuditMastTagged + IsoBudgetMeasured +
// PersonaReanchored + CrossFamilyUnavailable). Citation: arXiv:2503.13657
// MAST PRIMARY ANCHOR (substituted for bad plan cite 2406.12624; see
// workflow/reasoning/E-6-decisions.md DECISION-1 + gotcha
// E6_CITATION_2406_12624_ID_COLLISION).
// E-7 (SCHEMA_VERSION 149): 424 -> 425 for DarwinianGenerationCompleted
// single-variant Pattern-2 (13th application).
// batch2-merge-collision recovery (SCHEMA_VERSION 161): 425 -> 434 atomic
// re-application of the 9 variants lost when 3 parallel executors in
// cluster s-25120 batch 2 clobbered each other. Composes:
//   * E-3 RuntimeHandoffSignal       (425 -> 426)
//   * E-26 SglangBackendRegistered   (426 -> 427)
//   * E-26 XgrammarSchemaCompiled    (427 -> 428)
//   * E-26 RuntimeColdStartObserved  (428 -> 429)
//   * E-2 StreamingTimingRecorded    (429 -> 430)
//   * E-2 UxPacingVoteRecorded       (430 -> 431)
//   * E-28 StreamingLlmEngaged       (431 -> 432)
//   * E-28 SdpaBackendPinned         (432 -> 433)
//   * E-28 YarnRopeScalingApplied    (433 -> 434)
// See workflow/sessions/s-25120/completion-drive/audits/batch2-fixer-report.md.
// E-8 (SCHEMA_VERSION 162): 434 -> 435 for PromptOptimizationCompleted
// single-variant Pattern-2 (14th application). OptimizerPhase 7-arm
// discriminator on payload covers dspy_init / opro_step / rank_prompt_vote
// / ape_proposal / gepa_reflect / archive_promote / archive_retire.
// Wire-dark unless ELAI_PROMPT_OPTIMIZER_ENABLE=1 AND cfg(feature =
// "darwinian-evolve") (re-uses E-7 feature gate per
// workflow/reasoning/E-8-decisions.md DECISION-3).
// E-10 (SCHEMA_VERSION 163): 435 -> 436 for CriticDecisionEvent
// single-variant Pattern-2 (15th application). CriticTilePhase 9-arm
// discriminator on payload covers fsm_ablation / role_ablation /
// rag_challenger / mistake_book_signal / typed_cot_verified /
// tier_matrix / cloud_default / grc_blockquote / prompt_cache_hit_rate.
// Wire-dark unless ELAI_CRITIC_OF_DESIGN_ENABLE=1. See
// workflow/reasoning/E-10-decisions.md.
// E-43 (SCHEMA_VERSION 164): 436 -> 437 for SkillSystemEvent
// single-variant Pattern-2 (16th application). SkillEventPhase 3-arm
// discriminator on payload covers loaded / invoked / failed. Wire-dark
// unless ELAI_SKILL_SYSTEM_ENABLE=1. See workflow/reasoning/E-43-decisions.md.
// E-9 (SCHEMA_VERSION 165): 437 -> 438 for BenchmarkRunObserved
// E-19 (SCHEMA_VERSION 166): 438 -> 439 for ReasoningQualityProbe
// single-variant Pattern-2 (18th application). ProbeKind 6-arm
// discriminator on payload covers l2_gate_decision / l4_prm_lite_score /
// l4_lateral_kept / l4_ttc_budget_chosen / l6_verifier_diversity /
// vividness_probe (I-5 adds 6th arm). Wire-dark unless ELAI_REASONING_QUALITY_PROBE_ENABLE=1.
// single-variant Pattern-2 (17th application). BenchPhase 5-arm
// discriminator on payload covers loaded / started / item_evaluated /
// completed / aborted; suite_name string field discriminates across the
// 11 external benchmark suites. Wire-dark unless
// ELAI_EVAL_BENCHMARK_ENABLE=1. Live runs RUNPOD-PENDING. Paper
// grounding: arXiv:2509.16941 SWE-Bench Pro (primary, RAG-indexed) +
// arXiv:2505.23419 SWE-bench-Live + arXiv:2511.22173 RefineBench +
// arXiv:2604.05172 ClawsBench + arXiv:2311.12983 GAIA + arXiv:2601.18137
// DeepPlanning + arXiv:2506.07982 tau2. REMOVED per
// workflow/reasoning/E-9-decisions.md DECISION-3: plan's "DS-CP
// arXiv:2510.05566" + focused.io blog + "MetaMate" (paper-ingestor
// could not resolve).
// E-55 (SCHEMA_VERSION 168): 440 -> 441 for RewardLadderEvent
// single-variant Pattern-2 (20th application). RewardLadderEventKind 9-arm
// discriminator on payload covers rlaif_judge_rated /
// rlaif_calibration_correlation_low / gepa_template_evolved /
// gepa_template_evicted / reward_fusion_computed /
// reward_hacking_suspected / sft_training_threshold_reached /
// bai_arm_observation_recorded / rl_weight_update_unlocked. Wire-dark
// unless ELAI_AGENTIC_RL_ENABLE=1. Deferred RL ladder: RLAIF judge +
// GEPA + reward fusion + MONA + SFT collector + 3-condition unlock gate
// for DPO/PPO/GRPO/KTO (which do not exist by design). Paper grounding:
// arXiv:2212.08073 + arXiv:2309.00267 + arXiv:2507.19457 +
// arXiv:2501.13011 + arXiv:2509.15557 + arXiv:2507.01352 +
// arXiv:2412.21139 + arXiv:2508.16546 + arXiv:2501.17161 +
// arXiv:2410.12784 (JudgeBench; replaces plan's WRONG_ID 2412.12509
// per FIX-1) + Nature 2025 AlphaProof URL (replaces plan's FATAL
// arXiv:2406.15829 per E-55 triage). DPO HARD-AVOIDED forever (Apple
// 2025 OOD + arXiv:2502.20847 + no paired data). PPO/GRPO/DAPO/KTO
// HARD-DEFERRED until unlock. RUNPOD-PENDING live runs. See
// workflow/reasoning/E-55-decisions.md.
// E-54 (SCHEMA_VERSION 167): 439 -> 440 for EvalCorpusEvent
// single-variant Pattern-2 (19th application). CorpusEventPhase 14-arm
// discriminator on payload covers ingest_rejected_ngram /
// ingest_rejected_cosine / shard_created / schema_violation_no_tests /
// test_fails_on_reference / test_useless_passes_broken /
// kor_orthogonality_violated / kor_orthogonality_passed /
// sycophancy_framing_gap_detected / sandbagging_anomaly /
// generator_rotation_violated / perf_delta_contamination_suspected /
// dynamic_knowledge_qa_generated / prod_traffic_probe_injected.
// Wire-dark unless ELAI_EVAL_CORPUS_ENABLE=1. Repairs D-6 PLANNED 9-gram
// filter pre-deploy with 13-gram + embedding-cosine + per-family directory
// shards + test-triplet schema + matched-pair sycophancy + KOR-orthogonality.
// CanaryFamily (5-arm) is canonical task family surface; KnowledgeQa is
// routed via ToolUse-flavored tasks. Paper grounding (post-triage):
// arXiv:2311.04850 + arXiv:2602.12413 (re-triaged CENTRAL in E-54 domain)
// + arXiv:2403.07974 + arXiv:2410.06526 + arXiv:2503.02951 + arXiv:2411.04905
// + arXiv:2510.20487 + arXiv:2508.00943 + arXiv:2510.04721 + arXiv:2412.08905
// + arXiv:2509.00072 + arXiv:2511.08598 + arXiv:2503.14499 + arXiv:2511.17602
// + arXiv:2410.12784 (JudgeBench; replaces plan's WRONG_ID 2412.12509 per
// FIX-1) + arXiv:2505.23419 + arXiv:2403.04132. Live ingest + LoRA
// cross-family transfer + sandbox test_executor + sidecar embeddings
// RUNPOD-PENDING. See workflow/reasoning/E-54-decisions.md.
// E-11 (SCHEMA_VERSION 169): SaaS prior-art catalog + recommender
// substrate. SINGLE-VARIANT Pattern-2 (21st application); 2-arm
// PriorArtEventKind discriminator on payload covers
// archetype_recommended / prior_art_consulted. Catalog + recommender +
// 15-entry prior-art registry live at orchestrator::architecture.
// Wire-dark unless ELAI_SAAS_PRIOR_ART_ENABLE=1. 5-archetype taxonomy
// locked by e50.A guard. Grounding: 2024-2026 idea-to-app survey at
// workflow/improvement-sweep-2026-05-19/reports/
// 11-idea-to-saas-prior-art.md (Veracode 2025 45% / Tenzai CSA Dec 2025
// 100% / Escape.tech 2025 1,400 apps / Help Net 2025 slopsquatting
// 19.7% / Answer.AI Devin Jan 2025 3/20 / Cognition Devin 2025 review).
// See workflow/reasoning/E-11-decisions.md.
// E-22 (SCHEMA_VERSION 170): 442 -> 443 for GithubTrendingEvent
// single-variant Pattern-2 (22nd application). TrendingEventKind 3-arm
// discriminator covers skill_export_normalized / microvm_fallback /
// lazy_schema_savings. Wire-dark unless ELAI_GITHUB_TRENDING_ENABLE=1.
// See workflow/reasoning/E-22-decisions.md.
// E-23 (SCHEMA_VERSION 171): 443 -> 444 for DeveloperDiscourseEvent
// single-variant Pattern-2 (23rd application). DiscourseEventKind 3-arm
// discriminator covers trust_telemetry_aggregated /
// runtime_choice_badge_emitted / context_state_warning. Wire-dark unless
// ELAI_DEVELOPER_DISCOURSE_ENABLE=1.
// See workflow/reasoning/E-23-decisions.md.
// E-25 (SCHEMA_VERSION 172): 444 -> 445 for ClaudeCodeArchitectureEvent
// single-variant Pattern-2 (24th application). ClaudeCodeArchitectureEventKind
// 9-arm discriminator covers compaction_ladder_tier_applied /
// skill_template_executed / prompt_decision_made /
// opusplan_overlay_activated / session_forked / edit_checkpoint_snapshotted
// / bash_complexity_blocked / approval_fatigue_detected /
// sandbox_template_warned. Wire-dark unless ELAI_CLAUDE_CODE_ARCH_ENABLE=1.
// Catalog lives at crate::architecture::claude_code_arch; concrete
// implementation modules (compaction_ladder.rs / cli/fork.rs /
// edit_checkpoints.rs / prompt_handler.rs / dashboard panels) are
// follow-ups. Primitive 2 (deferred MCP descriptions) DESCOPED pending
// E-22 Task 3 / C-35 lazy_schemas substrate.
// See workflow/reasoning/E-25-decisions.md.
// E-29 Tier-B (SCHEMA_VERSION 173): 445 -> 446 for AgenticObservabilityEvent
// single-variant Pattern-2 (25th application). AgenticObservabilityEventKind
// 4-arm discriminator covers mast_failure_classified / replay_resume /
// signal_match / eval_dataset_member. Wire-dark unless
// ELAI_AGENTIC_OBSERVABILITY_ENABLE=1. MAST classifier wires the 14-variant
// crate::mast::MastMode taxonomy (E-6 SHIPPED -- do NOT redefine). Replay
// validator FAIL-CLOSED on C-44/D-9 frozen-surface predicate
// (locked_keys_auditor). DRIFT-2 collision avoidance: the pre-existing
// `failure_classified` action_type owns the 13-category SYMPTOM taxonomy
// (crate::regression); the new event family is the distinct CAUSE channel.
// Grounding: arXiv:2503.13657 (MAST), laminar.sh (Signals), langfuse.com
// (datasets), dev.to/sreeni5018 (LangGraph replay), traceloop.com
// (TRACELOOP_TRACE_CONTENT pattern), opentelemetry.io semconv 1.40.0.
// See workflow/reasoning/E-29-decisions.md (Tier-B section).
// E-31 (SCHEMA_VERSION 174): 446 -> 447 for CodingModelLeaderboardEvent
// single-variant Pattern-2 (26th application). LeaderboardEventKind 3-arm
// discriminator covers scrape_started / scrape_completed / scrape_failed.
// Wire-dark unless ELAI_CODING_MODEL_LEADERBOARD_ENABLE=1. Scraper at
// benches/external/coding_model_leaderboard.py emits stdout JSON rows
// parsed by the rust dispatcher. Live scrape RUNPOD-PENDING.
// Targets: BigCodeBench / EvalPlus HumanEval+ MBPP+ / SWE-Bench Pro /
// LiveCodeBench. Cite-verified May 2026: arXiv:2409.12186 (Qwen2.5-Coder),
// arXiv:2509.16941 (SWE-Bench Pro), morphllm.com/swe-bench-pro,
// labs.scale.com/leaderboard/swe_bench_pro_public, bigcode-bench.github.io,
// evalplus.github.io/leaderboard.html, livecodebench.github.io,
// platform.claude.com/docs/en/about-claude/pricing. Companion non-bumped
// surface (E-31.A SHIPPED): contracts/tests/head_dim_invariant_contract.rs.
// E-31.B DEFERRED: sidecar boot assert (DRIFT-3 server.py:1993/1997/2093
// head_dim default 256), EscalationPolicy (DRIFT-1 pick_runtime + DRIFT-2
// providers/runtime/escalation_policy.rs name-collision with E-32). See
// workflow/reasoning/E-31-leaderboard-decisions.md.
// E-38 (SCHEMA_VERSION 175): 447 -> 448 for AiRoadmapSignalEvent
// single-variant Pattern-2 (27th application). RoadmapSignalKind 5-arm
// discriminator covers compliance_blocked / compliance_verified /
// mcp_transport_version_logged / mrcr_unverified / posture_doc_stale.
// Wire-dark unless ELAI_AI_ROADMAP_SIGNALS_ENABLE=1. Producers:
// orchestrator/compliance.rs (EU AI Act pre-gate at
// PLANNING|FEASIBILITY_PROBE -> TESTING_FIRST), orchestrator/dispatch/mcp.rs
// (transport-version beacon at McpClientDispatch::new), orchestrator/
// roadmap_signals.rs (mrcr + posture-stale probes). See
// workflow/reasoning/E-38-decisions.md. Cite-verified May 2026:
// arXiv:2409.12640 (MRCR v2), arXiv:2510.12357 (MoBiLE),
// arXiv:2602.03495 (DALI), artificialintelligenceact.eu/implementation-
// timeline (EU AI Act timeline + Digital Omnibus May 7 2026),
// modelcontextprotocol.io/development/roadmap.
//
// E-39 (SCHEMA_VERSION 176): 448 -> 449 for CostTokenEvent single-
// variant Pattern-2 (28th application). CostEventKind 8-arm
// discriminator covers task_aborted_by_spend_cap /
// velocity_cap_breached / automix_verifier_verdict /
// cost_router_decision / cost_estimate_inaccurate /
// free_tier_quota_approaching / free_tier_quota_exceeded /
// layered_cap_manual_override. Wire-dark unless
// ELAI_COST_TOKEN_ECONOMICS_ENABLE=1. Producers: orchestrator/cost/
// {velocity_cap,automix_gate,cost_router,free_tier_guard,
// cloud_abort}.rs + CLI `elai cloud override-cap`. Cite-verified
// May 2026: arXiv:2310.12963 (AutoMix self-verify cascade, ICLR
// 2025), arXiv:2604.22750 ("How Do AI Agents Spend Your Money?"
// Stanford Digital Economy Lab April 2026; 30x same-task variance),
// arXiv:2305.05176 (FrugalGPT cascade 98% cost reduction). See
// workflow/reasoning/E-39-decisions.md.
//
// E-50 (SCHEMA_VERSION 177): 449 -> 450 for SaasArchPatternEvent
// single-variant Pattern-2 (29th application).
// SaasArchPatternEventKind N-arm discriminator (11 arms) covers
// production_readiness_gate_blocked / production_readiness_gate_passed
// / rls_baby_step_emitted / unscoped_token_blocked /
// billing_mor_decision_made / realtime_block_decided /
// async_backbone_decided / per_tier_rate_limit_derived /
// otel_slo_runbook_scaffolded / platform_unsupported_for_scope_check /
// archetype_block_overridden. Wire-dark unless
// ELAI_SAAS_ARCH_PATTERNS_ENABLE=1. Producer:
// orchestrator/src/architecture/saas_patterns.rs (sibling to E-11
// catalog / recommender / prior_art / saas_archetype). Composes the
// 5-archetype taxonomy with day-1 production-readiness gate + RLS
// baby-step emit + scoped-token enforcement + MoR decision fork +
// realtime / async / rate-limit / OTel-SLO scaffolding. Source
// grounding May 2026: hunchbite.com multi-tenant SaaS,
// accuknox.com OWASP API 2026, dev.to PocketOS 9-second-disaster,
// fintechspecs.com 2026 Polar 4% OSS MoR, oneuptime.com 14x/6x/1x
// burn-rate, pkgpulse.com Hatchet vs Trigger.dev vs Inngest 2026.
// See workflow/reasoning/E-50-decisions.md.
//
// E-4 (SCHEMA_VERSION 178): 450 -> 451 for ChatUxCritiqueEvent
// E-16 (SCHEMA_VERSION 179): 451 -> 452 for VlmImageValidationEvent
// E-20 (SCHEMA_VERSION 180): 452 -> 453 for LocalCloudRoutingEvent
// SINGLE-VARIANT Pattern-2 (32nd application). RoutingEventKind 9-arm
// discriminator covers confidence_emitted / hybrid_escalation /
// difficulty_classified / privacy_blocked / output_filtered /
// burn_alert / reasoning_mode / majority_vote /
// role_priority_chosen. PrivacyBlock fire-point composes with the
// pre-existing RouteBlocked variant (C-5 ordinal 331) via a new
// reason="privacy_block" value extension on RouteBlockedPayload;
// the new RoutingEventKind::PrivacyBlocked carries the match-span /
// severity payload that does not fit on the cap-shaped
// RouteBlockedPayload. Wire-dark unless
// ELAI_LOCAL_CLOUD_ROUTING_ENABLE=1. Producer:
// orchestrator/src/cost/cloud_route.rs (sibling to E-39
// automix_gate + cost_router + cloud_abort under cost/ subtree, NOT
// orchestrator/src/routing/ which is owned by C-46 Uno-Orchestra and
// NOT crates/orchestrator/src/cloud_spend.rs which was REJECTED per
// E39_CLOUD_SPEND_PATH_DRIFT). Paper grounding (verified May 2026
// via WebSearch):
//   * arXiv:2511.07396 C3PO Nov 2025 conformal-cost-bounded cascade
//   * arXiv:2406.18665 RouteLLM LMSYS 2x cost reduction
//   * arXiv:2310.12963 AutoMix >50% cost reduction
//   * arXiv:2510.01237 Confidence-Aware Routing (probe + perplexity
//     dominate verbalized-self-confidence; F1 0.61 -> 0.82)
//   * arXiv:2510.08731 When to Reason IBM (-47.1% latency / -48.5%
//     tokens / +10.2pp MMLU-Pro)
//   * arXiv:2502.00674 Self-MoA (anti-pattern doc; weakest-member drag)
//   * arXiv:2511.03808 Prompt-Difficulty Prediction (activation probes)
//   * arXiv:2305.05176 FrugalGPT (up to 98% cost cut at GPT-4 parity)
// Supporting-only: arXiv:2510.08146 + arXiv:2510.00202 +
// arXiv:2510.08439 + arXiv:2504.01282 + arXiv:2406.12334. See
// workflow/reasoning/E-20-decisions.md.
// SINGLE-VARIANT Pattern-2 (31st application). VlmImageValidationKind
// 3-arm discriminator covers swap_order_disagreement /
// axtree_mismatch / vlm_schema_unconstrained. SycophancyFlag (E-4 /
// D-14 prior ship) REUSED for the sycophancy_probe inverted-framing
// fire-point. Wire-dark unless ELAI_VLM_IMAGE_VALIDATION_ENABLE=1.
// Producer: orchestrator/src/media_validator/{vlm_validator, axtree,
// sycophancy}. Paper grounding (May 2026; verified inline):
// arXiv:2408.11261 VLM sycophancy LQCD; arXiv:2406.07791 position-
// bias 15-judge x 22-task systematic; playwright.dev/docs/aria-
// snapshots v1.59+; arXiv:2411.15100 XGrammar mask-emission
// (supporting-only). arXiv:2604.05839 (plan body RCE-convergence
// cite) NOT VERIFIED -- tagged supporting-only. See
// workflow/reasoning/E-16-decisions.md.
// single-variant Pattern-2 (30th application).
// ChatUxCritiqueKind 13-arm discriminator covers
// claim_confidence_tier_emitted / sycophancy_pushback_detected /
// user_rule_pinned / user_rule_unpinned / session_re_anchored /
// doom_loop_detected / completion_claim_rejected_no_evidence /
// one_shot_clarifier_emitted / one_shot_clarifier_resolved /
// refusal_emitted / refusal_overridden_by_user /
// mid_stream_interrupt_committed / honest_correction_emitted.
// Wire-dark unless ELAI_CHAT_UX_CRITIQUE_ENABLE=1. Producer:
// orchestrator/src/chat_ux/* (substrate module). 13-arm exceeds prior
// 11-arm empirical max (E-50) by 2; deliberate single-envelope choice
// keeps action_type_lock.golden cascade to ONE ordinal insertion vs
// 13 separate variants = 13 cascades. Paper grounding (verified May
// 2026): arXiv:2310.13548 Sharma sycophancy, arXiv:2412.00804 identity
// drift, arXiv:2507.21285 Curiosity by Design, arXiv:2509.08833
// over-alignment, aclanthology 2024.tacl-1.9 Lost in the Middle.
// Industry grounding: openai.com/index/sycophancy-in-gpt-4o (April
// 2025 rollback), HN 47936579 (false-completion), anthropic.com/
// constitution (diplomatic honesty), warpedvisions.org 2025 (doom-
// loop UX). 2604.* IDs cited in plan body remain supporting-only
// pending bulk verification. Stale plan line refs documented in
// workflow/reasoning/E-4-decisions.md. See workflow/reasoning/
// E-4-decisions.md.
// E-36 (SCHEMA_VERSION 182): 454 -> 455 for DocumentVisionRagEvent
// single-variant Pattern-2 (34th application).
// DocumentVisionRagEventKind 5-arm discriminator covers
// vision_chunk_emitted / vision_fallback_triggered /
// vision_rag_policy_decision / visual_artifact_check_verdict /
// doc_parser_cloud_fallback_blocked_by_mode. Wire-dark unless
// ELAI_DOCUMENT_VISION_RAG_ENABLE=1. Producers:
// orchestrator/src/document_vision_rag/{visual_verifier, policy,
// mod}.rs + knowledge::chunking::image_artifact +
// knowledge::chunking::pdf_structured. 10th VerificationMethod variant
// (reasoning::types::VerificationMethod::VisualArtifactCheck)
// dispatches into this substrate; reuses claim_category String rather
// than introducing a parallel claim_class taxonomy. Paper grounding
// (verified May 2026 via WebSearch): arXiv:2510.14528 PaddleOCR-VL,
// arXiv:2505.14059 Dolphin-v2, arXiv:2407.01449 ColPali ICLR 2025,
// arXiv:2602.12510 Visual RAG Toolkit (training-free pooling),
// arXiv:2502.12342 REAL-MM-RAG, arXiv:2411.04952 M3DocRAG.
// See workflow/reasoning/E-36-decisions.md (Tier-A + Tier-B).
// E-42 (SCHEMA_VERSION 183): 455 -> 456 for TreeSitterGrammarEvent
// single-variant Pattern-2 (35th application).
// TreeSitterGrammarEventKind 4-arm discriminator covers
// abi_probe_mismatch / svelte_runes_parse_health /
// tier_b_grammar_loaded / tree_sitter_skipped_large_file. Wire-dark
// unless ELAI_TREE_SITTER_GRAMMARS_ENABLE=1. Producers:
// knowledge::tree_sitter_health::{abi_probe, svelte_health, large_file,
// registration} + DkgEngine init path. Collapses the plan-demanded
// 4 sibling variants into one variant per DRIFT-6 in
// workflow/reasoning/E-42-decisions.md.
// E-46 (SCHEMA_VERSION 184): 456 -> 457 for PlanningDecompositionEvent
// single-variant Pattern-2 (36th application).
// PlanningDecompositionEventKind 8-arm discriminator covers
// reflection_appended / low_entropy_decomposition /
// goal_drift_canary_below_threshold / bubble_up_replan_phase /
// bubble_up_ledger_reset / agentless_shortcut_dispatched /
// process_reward_score_emitted / skill_prior_retrieved. Wire-dark
// unless ELAI_PLANNING_DECOMPOSITION_ENABLE=1. Producers:
// orchestrator/src/planning_decomposition/producers.rs. Collapses the
// plan-demanded 8 sibling variants into one variant per Pattern-2
// precedent (E-20 9-arm + E-21 4-arm + E-36 5-arm + E-42 4-arm +
// E-4 13-arm). See workflow/reasoning/E-46-decisions.md.
// E-48 (SCHEMA_VERSION 185): 457 -> 458 for WebResearchEvent
// single-variant Pattern-2 (37th application). WebResearchEventKind
// 10-arm discriminator covers research_plan_emitted /
// tiered_search_dispatched / extraction_failed_all_extractors /
// cross_encoder_rerank_complete / speculative_branch_pruned /
// research_preemptive_stop / contradiction_node_materialized /
// citation_verification_outcome / citation_attribution_violation_blocked
// / recency_decay_applied. Wire-dark unless
// ELAI_WEB_RESEARCH_ORCHESTRATOR_ENABLE=1. Producers:
// orchestrator/src/research/web_orchestrator/producers.rs +
// web_search/src/{tiered_search_pool,extractor_pool,
// cross_encoder_rerank}.rs telemetry hooks. Collapses the plan-named
// 17 fire-points into 10 Kind arms by folding outcome discriminators
// into details_json (per E-50 11-arm + E-4 13-arm precedent). See
// workflow/reasoning/E-48-decisions.md.
// E-44 (SCHEMA_VERSION 186): 458 -> 459 for DurabilityEvent
// single-variant Pattern-2 (38th application). DurabilityEventKind
// 10-arm discriminator covers tool_idempotency_hit /
// tool_idempotency_replayed_irreversible / compensation_logged /
// compensation_replayed / compensation_failed /
// session_dag_continue_as_new / checkpoint_lazy_migrated /
// compact_lock_contended / agent_reliability_metrics /
// tool_error_classified. Wire-dark unless
// ELAI_DURABILITY_OBSERVABILITY_ENABLE=1. Producers:
// orchestrator/src/durability/{idempotent_dispatcher,continue_as_new,
// checkpoint_migrator,compact_lock,resume_validator}.rs. Closes
// TASK-E44-013 observability gap: 4 of 10 task primaries SHIPPED but
// emitted ZERO audit rows before this bump. 5 of 10 Kinds reserve wire
// tags for NOT-SHIPPED producers (saga compensation, reliability
// metrics rollup, tool-error taxonomy). See
// workflow/sessions/s-night-2026-05-30/impl-tasks/TASK-E44-013.md.
// E-31 (SCHEMA_VERSION 187): 459 -> 460 for HeadDimInvariantVerified
// single-variant Pattern-2 (39th application).
// HeadDimInvariantVerifiedKind 2-arm discriminator covers
// verified / refused. Wire-dark unless
// ELAI_HEAD_DIM_INVARIANT_SSB_ENABLE=1. Producer:
// python/elai_inference/server.py::assert_head_dim_invariant. Closes
// TASK-E31-001 (head_dim invariant + SSB row gap surfaced by the
// s-night-2026-05-30 cross-check).
// E-3 (SCHEMA_VERSION 188): 460 -> 461 for RoleShownModelHidden
// single-variant Pattern-2 (40th application). No Kind discriminator
// (scope-lean MVP slice). Wire-dark unless
// ELAI_ROLE_VISIBILITY_TELEMETRY_ENABLE=1. Producer:
// crates/dashboard-v2/src-tauri/src/commands.rs::record_model_visibility.
// Closes the cross-check finding "SettingsWindow explicitly exposes
// MODEL names per role" by reversing the default (ROLE shown, MODEL
// hidden unless user toggles "show model details" ON). Full E-3 plan
// (role_prompt.rs / voice.toml / RuntimeAdapter::warm / 5 sibling SSB
// variants / persona-vector hook) DEFERRED to follow-on sessions. See
// workflow/sessions/s-night-2026-05-30/impl-tasks/TASK-E03-001.md.
// E-49 (SCHEMA_VERSION 189): 461 -> 462 for CodingMetricNamedToken
// single-variant Pattern-2 (41st application).
// CodingMetricNamedTokenKind 3-arm discriminator covers
// stable_pass_at_1 / harness_attribution_tuple / stop_verified.
// Wire-dark unless ELAI_CODING_METRIC_NAMED_TOKEN_ENABLE=1. Closes
// TASK-E49-005 / 006 / 007 SPEC-DRIFT against the 3 spec-named tokens
// (zero hits before this bump per cross-check E-16_e47_e48_e49.md).
// E-3 sub-phase 4A (SCHEMA_VERSION 190): 462 -> 463 for
// E3RuntimeHandoffEvent single-variant Pattern-2 (42nd application).
// RuntimeHandoffEventKind 6-arm discriminator covers
// unified_preamble_applied / runtime_prewarm_started /
// runtime_prewarm_complete / handoff_payload_compressed /
// persona_vector_applied / style_normalizer_applied. Wire-dark unless
// ELAI_E3_RUNTIME_HANDOFF_ENABLE=1. Sub-phase 4A wires only the
// UnifiedPreambleApplied arm via the role-prompt wrap in
// runtime_orchestrator.rs (tracing log; Ssb::write_payload injection
// deferred to 4B). Pattern-2 collapse rationale: original E-3 plan
// called out 6 sibling variants; collapsed per Pattern-2 precedent
// saving 5 SCHEMA slots. See
// workflow/sessions/s-night-2026-05-30/impl-tasks/TASK-E03-001.md +
// docs/thefinalglance/E-plan/E-3_model_handoff_ux.md.
// E-31 sub-phase 6A (SCHEMA_VERSION 191): 463 -> 464 for
// RouteDecisionEvent single-variant Pattern-2 (43rd application).
// RouteDecisionEventKind 3-arm discriminator covers cloud_route_chosen /
// tier_model_swapped / local_route_kept. Wire-dark unless
// ELAI_ROUTE_DECISION_SSB_ENABLE=1. Producer:
// crates/orchestrator/src/cloud_mode.rs::CloudModeRouter::route via
// providers::runtime::RuntimeRoutingPolicy::evaluate. Closes the E-31
// plan's deferred CloudRouteDecision + TierModelSwap envelopes in a
// SINGLE Pattern-2 variant. See
// workflow/sessions/s-21636/exploration/tranche6-E31-wiring.md +
// docs/strategy/E-31-tier-matrix.md.
//
// E-47 (SCHEMA_VERSION 192): bump 191 -> 192 for LoraShipGateEvent
// SINGLE-VARIANT Pattern-2 (44th application). LoraShipGateEventPhase
// 3-arm discriminator covers refused / calibration_missing /
// calibration_passed. Wire-dark unless ELAI_LORA_SHIP_GATE_ENABLE=1.
// Producer: crates/orchestrator/src/lora_ship_gate.rs +
// `elai lora-gate-check` CLI subcommand. Ships the LoRA ship-gate
// LOCALLY (default-deferred posture); producer-side LoRA training
// stack (Unsloth + QLoRA-NF4 + PiSSA + DoRA on RTX 3090) is
// RUNPOD-PENDING. See workflow/reasoning/E-47-decisions.md.
//
// Tranche B-6 quad (SCHEMA_VERSION 194): bump 466 -> 470 for four new
// SSB substrates landing in one coordinated orchestrate run:
//   * C-42 McpAuthzEvent (3-arm Kind authz_check_passed /
//     authz_check_failed / corpus_replayed). Wire-dark unless
//     ELAI_C42_MCP_AUTHZ_EMIT_ENABLE=1. Producer:
//     crates/plugins/src/authz/compose.rs::compose_with_tool_gate.
//   * C-47 ToolCalibrationEvent (6-arm Kind hint_emitted /
//     drift_detected / call_logged / case_added / case_selected /
//     calibration_applied). Wire-dark unless
//     ELAI_C47_CASE_BASE_ENABLE=1. The 3 pre-existing
//     tool_call_logged / tool_calibration_hint_emitted /
//     tool_calibration_drift_detected variants stay alive for one
//     release window as deprecation aliases.
//   * B-39 ReflectBudgetSoftWarn (single-add, 5th reflect_* sibling).
//     Wire-dark unless ELAI_B39_SOFTWARN_ENABLE=1. Producer:
//     crates/reasoning/src/chat_reflect.rs ~L454.
//   * C-50 ContractCodingEvent (3-arm Kind retry_attempted /
//     retry_resolved / retry_abandoned). Wire-dark unless
//     ELAI_C50_CONTRACT_RETRY_EMIT_ENABLE=1. Producer:
//     crates/orchestrator/src/task_runner.rs::fix_and_rerun_until_green.
//     Pre-existing contract_violation_detected stays alive (NOT
//     collapsed in this tranche).
// See workflow/reasoning/B6-quad-tranche-decisions.md.
//
// Tranche B-7 triple (SCHEMA_VERSION 195, 2026-05-31): +1 new top-level
// variant (TddVerificationEvent) lands as the 4-arm Pattern-2 envelope
// covering E-13 PropertyGatePass / MutationGateSurvivorBlock /
// AntiMockBlock + E-12 AssertionOnlyDiffBlock. The SaasArchPatternEventKind
// arm extension (E-50 SaasArchetypeSelected, 12th arm) is variant-internal
// and does NOT bump TOTAL_VARIANTS. See workflow/reasoning/
// B7-triple-tranche-decisions.md.
// F-1 / F-2a (SCHEMA_VERSION 196, 2026-06-01): +2 new top-level variants
// (VibeGateBlocked + SecurityRuleBlocked). VibeGateBlocked is Pattern-2
// single variant with 4-arm VibeGateBlockedKind (CodeVuln / Secret /
// DepHallucinated / Skill) but counts as ONE variant. 471 -> 473.
// Session: s-6344. See workflow/reasoning/F-1-F2a-decisions.md.
// F-4 / Slice B-1 (SCHEMA_VERSION 197, 2026-06-02): +1 new top-level variant
// (TestIntegrityEvent). Pattern-2 single variant with 2-arm
// TestIntegrityEventKind (Verified / FabricationBlocked); counts as ONE
// variant. 473 -> 474. Session: s-6344.
// F-3p1 / Slice C1 (SCHEMA_VERSION 198, 2026-06-02): +1 new top-level variant
// (ToolFilterEvent). Pattern-2 single variant with 2-arm ToolFilterEventKind
// (Committed / UntrustedQuarantined); counts as ONE variant. 474 -> 475.
// Session: s-6344. See workflow/reasoning/F-3p1-decisions.md.
//
// F-9 / Slice D1 (SCHEMA_VERSION 199, 2026-06-02): WriteTimeSlopsquatBlocked
// flat beacon (DepWriteTimeGate). 475 -> 476. Session: s-6344.
//
// F-10 / Slice A1 (SCHEMA_VERSION 200, 2026-06-02): SecurityClassRegressionBlocked
// flat beacon (security-monotonicity iteration guard). loop_kind is a payload
// field, not a Kind on the action-type enum, so this counts as ONE variant.
// 476 -> 477. Session: s-60824.
//
// F-2b / Slice A1 (SCHEMA_VERSION 201, 2026-06-02): InsecureRateMeasured flat
// beacon (insecure-rate keystone bench). Bench telemetry, not a gate; loop_kind
// has no analogue here (single flat payload). Counts as ONE variant.
// 477 -> 478. Session: s-60824.
// F-6 / Slice A1 (SCHEMA_VERSION 202, 2026-06-02): +1 new top-level variant
// (SkillLibraryEvent). Pattern-2 single variant with 3-arm SkillLibraryEventKind
// (Acquired / TrustPromoted / DemotedStale); counts as ONE variant here (the
// Acquired arm is the representative arm in ALL_VARIANTS + golden). The
// skill_trust_promoted / skill_demoted_stale sql arms are parse/sql-only and
// NOT in ALL_VARIANTS. 478 -> 479. Session: s-60824.
//
// F-5 / Slice A1 (SCHEMA_VERSION 203, 2026-06-02): StalenessQuarantineBroadcast
// SINGLE FLAT beacon (cross-subsystem staleness-quarantine cascade). No Kind
// discriminator; one sql_str (staleness_quarantine_broadcast). Counts as ONE
// variant. 479 -> 480. Session: s-60824.
//
// F-8 / Slice A1 (SCHEMA_VERSION 204, 2026-06-02): SymbolicConflictDetected
// SINGLE FLAT beacon (SCF symbolic semantic conflict detector). No Kind
// discriminator; one sql_str (symbolic_conflict_detected). WARN-then-escalate
// (held pending), never hard fail-closed BLOCK. Counts as ONE variant.
// 480 -> 481. Session: s-60824.
//
// F-7 / Slice A1 (SCHEMA_VERSION 205, 2026-06-02): TemplateUtilityRanked
// SINGLE FLAT beacon (CBR template Laplace-utility ranking, HALF B only). No
// Kind discriminator; one sql_str (template_utility_ranked). Records the
// Laplace-smoothed outcome utility that gates crystallized-template reuse at
// try_execute. Counts as ONE variant. 481 -> 482. Session: s-60824.
//
// F-13 / Slice A1 (SCHEMA_VERSION 206, 2026-06-03): SessionArchived SINGLE FLAT
// beacon (session-archive-on-schema-bump). No Kind discriminator; one sql_str
// (session_archived). store_kind ('ssb'|'session_dag') is a payload field, not
// a Pattern-2 Kind. Forensic opt-in, default-OFF ELAI_SSB_ARCHIVE_ON_BUMP_ENABLE
// (W-2 in-file deviation, F-11 precedent). Counts as ONE variant. 482 -> 483.
// Session: s-6312.
//
// F-11 / Slice A1 (SCHEMA_VERSION 207, 2026-06-03): DarwinianAdmit Pattern-2
// single-variant WITH a 3-arm DarwinianAdmitKind discriminator (Admitted /
// RejectedByGuard / LoopClosed) on the payload; ONE sql_str (darwinian_admit).
// SEPARATE action_type from the pre-existing E-7 darwinian_generation_completed.
// Config-DGM admit-loop beacon, wire-dark unless ELAI_DARWINIAN_LOOP_ENABLE=1
// AND cfg(feature darwinian-evolve). Counts as ONE variant. 483 -> 484.
// Session: s-6312.
// G-plan + H-plan repair (2026-06-07, s-build-repair): 33 new variants
// completing G/H-plan security, verifiability, canvas, batch, staleness.
// 484 -> 517.
// D-27.1 (SCHEMA_VERSION 209, 2026-06-07): DiscoveryRatified +
// DiscoveryRejected discovery-queue reviewer-action beacons.
// Emitted by `ratify_discovery` / `reject_discovery` Tauri commands in
// commands_d27.rs. 518 -> 520.
// H-32 (SCHEMA_VERSION 210, 2026-06-07): FactValidationFailed flat beacon.
// Emitted by the H-32 citation-validity gate when a Tier-A fact's DKG
// citation fails to re-resolve. 520 -> 521.
// H-33 (SCHEMA_VERSION 211, 2026-06-07): MemoryGovernanceEvent flat beacon.
// forget/pin/unpin governance. 521 -> 522.
// H-34 (SCHEMA_VERSION 212, 2026-06-07): McpToolDeferred -- deferred MCP schema
// resolve on the live STDIO lane. Emitted on cache-miss when
// ELAI_MCP_DEFER_SCHEMA_ENABLE=1. 522 -> 523.
// H-34 (SCHEMA_VERSION 213, 2026-06-07): McpSchemaBudgetMeasured -- mcp-schema-budget
// bench result beacon. Emitted by `elai bench mcp-schema-budget`. 523 -> 524.
// H-35 (SCHEMA_VERSION 214, 2026-06-07): SessionRewound -- /rewind restore beacon.
// Emitted by `elai rewind <target>` and `rewind_session` Tauri command. 524 -> 525.
// H-36 (SCHEMA_VERSION 215, 2026-06-07): AgentNotificationEmitted -- completion/needs-input
// dev-event beacon. Emitted by NotificationDispatcher::notify on PhaseComplete and
// AutonomyDecision::RequestHuman. 525 -> 526.
// H-37 (SCHEMA_VERSION 216, 2026-06-07): PlanDecisionEvent -- plan-approval gate
// decision at PLANNING|FEASIBILITY_PROBE -> TESTING_FIRST. Gated by
// ELAI_PLAN_APPROVAL_GATE_ENABLE=1. Pattern-2 4-arm PlanDecisionKind discriminator
// (Approve/Redirect/Reject/AutoAdvance). 526 -> 527.
// H-38 (SCHEMA_VERSION 217, 2026-06-07): StopConfirmationEvent -- cost-confirm gate
// outcome on task/pod stop. Emitted by dispatch_stop on ConfirmRequired/RejectedHeld
// unconditionally; PassedNoFriction when ELAI_STOP_COST_CONFIRM_ENABLE=1.
// Pattern-2 single-variant 3-arm StopOutcomeKind discriminator. 527 -> 528.
// H-39 (SCHEMA_VERSION 218, 2026-06-08): AbstentionScored -- abstention-aware scoring
// bench beacon. Emitted by bench_safedelta/bench_seccodeplt after computing the
// AbstentionAggregate over the run's outcome triples. Closes the self-improvement
// loop misalignment (darwin_admit_loop reward abstention-blind). Flat beacon.
// Default-ON kill-switch ELAI_ABSTENTION_SCORE_DISABLE. 528 -> 529.
// H-41 (SCHEMA_VERSION 219, 2026-06-08): CheckerGapDetected -- emitted at the
// route_claim_with_gap call site when a Factual/Computational claim falls to
// non-Constructive because no VerifierKind::Deterministic in context.
// Off-agent backlog artifact. Wire-dark: ELAI_CHECKER_GAP_ENABLE. Flat beacon.
// 529 -> 530.
// H-42 (SCHEMA_VERSION 220, 2026-06-08): ReasoningBudgetScaled -- serial-depth-
// scaled GRC budget decision beacon. Emitted at agent_pool::spawn + bsp/engine.rs
// timeout path. Default-ON kill-switch ELAI_REASONING_BUDGET_SCALE_DISABLE.
// Pattern-2 single-variant 2-arm ReasoningBudgetScaledKind: DepthBound /
// AccessBound. Grounding: Merrill & Sabharwal 2310.07923 + Li et al. 2402.12875
// + Yehudai et al. 2503.01544. 530 -> 531.
// H-43 (SCHEMA_VERSION 221, 2026-06-08): FitnessEvaluated -- execution-grounded
// DGM fitness beacon. Replaces blake3-of-bytes derived_alpha (GAP-1) with a
// measured pass-rate signal (CodeRM arXiv:2501.01054 + CodeT arXiv:2207.10397).
// Emitted per scored candidate in darwin_admit_loop under darwinian-evolve feature.
// Pattern-2 single-variant 3-arm FitnessEvaluatedKind:
// ExecutionPassRate | DualAgreement | FitnessRegression. 531 -> 532.
// H-44 (SCHEMA_VERSION 222, 2026-06-11): TrapBehaviorDetected -- behavioral
// cheat-detector beacon. Replaces keyword-grep (GAP-2) with ImpossibleBench
// conflict canary (2510.20270) + IsomorphMismatch (2604.15149) + CodeHalu
// execution-classification (2405.00253) + KeywordFallback advisory. Carries
// frozen TRACE-style recall measurement (2601.20103). Pattern-2 single-variant
// 4-arm TrapBehaviorDetectedKind:
// ImpossiblePassExploit | IsomorphMismatch | ExecutionHallucination | KeywordFallback.
// 532 -> 533.
// H-45 (SCHEMA_VERSION 223, 2026-06-11): PromotionScoreAggregated -- worst-case/
// variance-penalized aggregate over lineage-diverse promotion verdicts.
// (Coste 2310.02743 WCO+UWO + Eisenstein 2312.09244 + Co-rewarding 2508.00410 +
// PRMBench 2501.03124 tau calibration). Replaces candidate mean as the
// two_gate_promote margin. Pattern-2 single-variant 3-arm
// PromotionScoreAggregatedKind:
// WorstCaseScored | VariancePenalized | LineageDiverseDisagree.
// 533 -> 534.
//
// H-46 (SCHEMA_VERSION 224, 2026-06-11): CladeSelected -- HGM clade-
// metaproductivity parent-selection beacon (HGM 2510.21614, ICLR 2026 oral).
// Emitted from config_archive::select_parent_with_cmp on the REAL loop
// selection step. CMP aggregates execution-grounded descendant deltas (H-43).
// ImmediateScoreFallback for clade_size==0 leaves (cold-start guard).
// Pattern-2 single-variant 2-arm CladeSelectedKind:
// CladeMetaproductivity | ImmediateScoreFallback.
// 534 -> 535.
//
// H-47 (SCHEMA_VERSION 225, 2026-06-11): EvalFirewallEvent -- DGM eval firewall
// beacon. Composes D-6 date-gate + E-54 template-regen + H-19 Thresholdout budget
// binding. Emitted once per generation (aggregate counts) from run_one_generation.
// The H-19 budget leg REUSES HoldoutBudgetEvent (GD-1: no second budget variant).
// Pattern-2 single-variant 3-arm EvalFirewallEventKind:
// PreUpdateItemRejected | TemplateRegenerated | IndistinguishabilityProbe.
// 535 -> 536.
//
// H-49 (SCHEMA_VERSION 226, 2026-06-11): EditHistoryEvent -- reversible LLM edits
// beacon. Emitted by the default-ON CAS time-travel capture hook in task_runner.rs
// and by elai undo/redo/jump CLI ops. Default-ON kill-switch ELAI_TIMETRAVEL_DISABLE.
// Pattern-2 single-variant 6-arm EditHistoryEventKind:
// SnapshotCaptured | Deduped | Redo | Branched | Jumped | TooLargeSkipped.
// 536 -> 537.
//
// H-51 (SCHEMA_VERSION 227, 2026-06-11): HandoffLifecycleEvent -- cross-session
// sleep-time handoff digest lifecycle beacon. Emitted by Drop hook (DigestWritten),
// REPL startup detect (ResumeOffered), one-tap confirm (ResumeConfirmed), and
// /continue tiered load (TieredLoad). Default-ON kill-switch
// ELAI_HANDOFF_AUTORESUME_DISABLE. Pattern-2 single-variant 4-arm
// HandoffLifecycleEventKind: DigestWritten | ResumeOffered | ResumeConfirmed |
// TieredLoad. 537 -> 538.
//
// H-52 (SCHEMA_VERSION 228, 2026-06-11): FindingAnchored -- deterministic
// snippet->hunk finding-anchoring beacon. Emitted by elai review --cockpit via
// review_cmd::open_score_and_emit. Closes G-5 OQ#2 (hunk_id join risk). Default-ON
// kill-switch ELAI_REVIEW_ANCHOR_DISABLE (no allowlist row per R9). Pattern-2
// single-variant 5-arm FindingAnchoredKind:
// Anchored | FileFallback | Ambiguous | Unanchorable | Relocated. 538 -> 539.
//
// H-53 (SCHEMA_VERSION 229, 2026-06-11): CitationAnchorsBound -- citation pre-binding
// + constrained synthesis beacon. Emitted by elai prebind --demo-prebind (REAL CLI
// entrypoint) and by the /research|/ask answer path after citation_prebind::freeze_anchors
// runs. Default-ON kill-switch ELAI_CITATION_PREBIND_DISABLE (no allowlist row per R9).
// Pattern-2 single-variant 2-arm CitationAnchorsBoundKind: Bound | Blocked. 539 -> 540.
//
// I-39 (SCHEMA_VERSION 230, 2026-06-11): PlanStateDerived -- per-plan state derived
// beacon. Emitted by elai plan status (REAL CLI entrypoint). Flat beacon; no inner Kind
// enum. letter + session_id + derived_state + evidence_anchor. Default-ON. 540 -> 541.
//
// I-45 (SCHEMA_VERSION 231, 2026-06-11): ObjectiveSealed -- objective-hash sealed-receipt
// beacon. Pattern-2 single-variant 3-arm ObjectiveSealedKind: Sealed | MutationDetected |
// ReceiptStale. Emitted from emit_run_artifact (run_subcommand.rs) after build_rra returns.
// Default-ON kill-switch ELAI_OBJECTIVE_SEAL_DISABLE (no allowlist row per R9). 541 -> 542.
//
// I-34 (SCHEMA_VERSION 234, 2026-06-12): GrammarGateEvent -- CRANE hybrid grammar-pin gate
// beacon. Pattern-2 single-variant 3-arm GrammarGateEventKind: CranePassed / PinViolation /
// PostParseRejected. Emitted by orchestrator grammar_gate.rs. Default-ON, no kill-switch
// (mandatory gate per IC-18 + landmine #7/#8). +3 I-34 (GrammarGateEvent Kind: CranePassed/PinViolation/PostParseRejected). 542 -> 543.
//
// I-38 (SCHEMA_VERSION 235, 2026-06-12): QdeltaMeasured -- QDELTA defective-diff rate flat
// beacon. Mirrors SafetyDeltaMeasured shape. Emitted by cli::bench_qdelta::run_inner after
// floor_check passes (seed_count>=5, paired=true). +1 I-38 (QdeltaMeasured). 543 -> 544.
//
// C-37: SnowballingMeasured flat beacon; +1 variant.
// I-43 (SCHEMA_VERSION 236, 2026-06-12): ClaimScopeMeasured -- reproducibility-claim scoper
// kill-gate. Pattern-2 single-variant 3-arm ClaimScopeMeasuredKind: PilotDelta /
// CrossLaneDivergence / ClaimScoped. Emitted by cli::bench_claim_scope::run_inner.
// +1 I-43 (ClaimScopeMeasured). 544 -> 545.
// I-10 (SCHEMA_VERSION 237, 2026-06-12): CacheEconomicsEvent -- prefix invariance + TTL tiering
// + cache-hit hard guard. Pattern-2 single-variant 3-arm CacheEconomicsEventKind: TtlPlanned /
// HitRateGuardTripped / PrefixDiffDumped. Emitted by orchestrator::cache_economics.
// +1 I-10 (CacheEconomicsEvent). 545 -> 546.
// I-8 (SCHEMA_VERSION 238, 2026-06-12): TrajectoryAborted -- aletheia voluntary abort + dead-end
// hash pruning + halting-bound seam. Pattern-2 single-variant 3-arm TrajectoryAbortedKind:
// Voluntary / DeadEndHash / HaltingBound. Emitted by TOOL_ACKNOWLEDGE_INFEASIBLE arm,
// dead_end_cache check path, and I-17 watchdog hook (Wave-3 stub).
// +1 I-8 (TrajectoryAborted). 546 -> 547.
// I-1 (SCHEMA_VERSION 239, 2026-06-12): DirectiveExecuted -- directive supervisor
// enforcement-plane outcome beacon. Pattern-2 single-variant 3-arm
// DirectiveExecutedKind: Executed / Discarded / CompensationFired. Emitted by
// directive::InProcessSupervisor::execute_directive. Default-ON (enforcement plane
// is mandatory per CANT-WRECK; no kill-switch). +1 I-1 (DirectiveExecuted). 547 -> 548.
// I-33 (SCHEMA_VERSION 240, 2026-06-12): GovernancePrimitiveFired -- Pattern-2 single-variant
// 3-arm GovernancePrimitiveFiredKind: CeflExternalized / I6qRejected / CommitRevealSealed.
// action_type string: "governance_primitive_fired". CeflExternalized representative in this lock array. 548 -> 549.
// I-32 (SCHEMA_VERSION 241, 2026-06-12): RoleContractApplied -- Pattern-2 single-variant
// 3-arm RoleContractAppliedKind: AbDelta / PersonaRetained / PersonaStripped.
// action_type string: "role_contract_applied". PersonaStripped representative in this lock array. 549 -> 550.
// I-35 (SCHEMA_VERSION 242, 2026-06-13): ToolSequenceVerdict -- Pattern-2 single-variant
// 3-arm ToolSequenceVerdictKind: Allowed / HaltEnforced / OrderViolation.
// action_type string: "tool_sequence_verdict". Allowed representative in this lock array. 550 -> 551.
// I-41 (SCHEMA_VERSION 243, 2026-06-13): CritiqueLensFinding -- spec-time critique lenses advisory
// beacon. Pattern-2 single-variant 3-arm CritiqueLensFindingTag: AdvisoryFinding / DroppedNoAnchor /
// LensRun. action_type string: "critique_lens_finding". AdvisoryFinding representative in this
// lock array. advisory=true on all arms; zero gate-verdict path. 551 -> 552.
// I-46 (SCHEMA_VERSION 244, 2026-06-13): BashRedirected -- bash interceptor redirect/fixup/
// allowed-prefix outcome. Pattern-2 single-variant 3-arm BashRedirectedKind: Redirected / FixedUp /
// PrefixBlocked. action_type string: "bash_redirected". Redirected representative in this
// lock array. 552 -> 553.
// I-47 (SCHEMA_VERSION 245, 2026-06-13): PromptHardeningApplied -- verifier-input XML tagging +
// worker rule injection + SSB beacon. Pattern-2 single-variant 3-arm PromptHardeningAppliedKind:
// InputsTagged / RulesInjected / SelfAttestationStripped. action_type string:
// "prompt_hardening_applied". InputsTagged representative in this lock array. 553 -> 554.
// I-2 (SCHEMA_VERSION 246, 2026-06-13): BranchWorkspaceEvent -- CoW workspace isolation barrier
// outcome. Pattern-2 single-variant 4-arm BranchWorkspaceEventKind: Spawned / Merged /
// EscapeBlocked / ConflictEscalated. action_type string: "branch_workspace_event". Spawned
// representative in this lock array. 554 -> 555.
// I-5 (SCHEMA_VERSION 247, 2026-06-13): ContextPruned -- per-branch context rebuild outcome.
// Pattern-2 single-variant 3-arm ContextPrunedKind: SiblingExcluded / TracebackContrasted /
// VividnessProbe. action_type string: "context_pruned". SiblingExcluded representative in this
// lock array. 555 -> 556.
// I-11 (SCHEMA_VERSION 248, 2026-06-13): BeliefStateEvent -- typed versioned condensate receipt.
// Pattern-2 single-variant 3-arm BeliefStateEventKind: Condensed / Handoff / HonestyDelta.
// action_type string: "belief_state_event". Condensed representative in this lock array. 556 -> 557.
// I-13 (SCHEMA_VERSION 249, 2026-06-13): MemoryGuardEvent -- write-time memory guard outcome.
// Pattern-2 single-variant 3-arm MemoryGuardEventKind: PoisonBlocked / DualSerialized /
// TraceQueryExpanded. action_type string: "memory_guard_event". PoisonBlocked representative
// in this lock array. 557 -> 558.
// I-14 (SCHEMA_VERSION 250, 2026-06-13): ContextTransformReceipt -- deterministic
// context-compression middleware receipt. Pattern-2 single-variant 3-arm
// ContextTransformKind: LspSlice / AstFold / Tier2Compressed. action_type string:
// "context_transform_receipt". LspSlice representative in this lock array. 558 -> 559.
// I-12 (SCHEMA_VERSION 251, 2026-06-13): PlaybookEvent -- evidence-gated playbook curator
// outcome beacon. Pattern-2 single-variant 4-arm PlaybookEventKind: Promoted / Demoted /
// ValidationRun / Exported. action_type string: "playbook_event". Promoted representative
// in this lock array. 559 -> 560.
// I-17 (SCHEMA_VERSION 252, 2026-06-13): TriLayerVerdict -- tri-layer evaluation-stack
// per-branch verdict. Pattern-2 single-variant 4-arm TriLayerVerdictKind: HardVeto /
// ScoredSurvivor / DisagreementEscalated / FlakeQuarantined. action_type string:
// "tri_layer_verdict". HardVeto representative in this lock array. 560 -> 561.
// I-18 (SCHEMA_VERSION 253, 2026-06-13): RowChainSealed -- per-row rolling blake3
// chain beacon (uPIP row-chain + AI-BOM). Pattern-2 single-variant 3-arm
// RowChainSealedKind: ChainAdvanced / AibomEmitted / ChainBroken. action_type string:
// "row_chain_sealed". ChainAdvanced representative in this lock array. 561 -> 562.
// I-19 (SCHEMA_VERSION 254, 2026-06-14): TddPreflightBlocked -- TDD enforcer preflight
// gate. Pattern-2 single-variant 3-arm TddPreflightBlockedKind:
// MissingTestDelta / SyntaxDryRunFailed / RefactorExempted. 3 distinct sql_str values:
// "tdd_preflight_missing_test_delta" / "tdd_preflight_syntax_dry_run_failed" /
// "tdd_preflight_refactor_exempted". Wire-dark: ELAI_TDD_PREFLIGHT_ENABLE=1 required.
// MissingTestDelta is the single representative arm in SSB_ACTION_TYPES (Pattern-2
// single-variant convention). action_type string: "tdd_preflight_missing_test_delta"
// (representative). TOTAL_VARIANTS 562 -> 563.
// I-20 (SCHEMA_VERSION 255, 2026-06-14): MutationBudgetedRun -- budgeted mutation
// scheduler (Track 15 KEYSTONE). Pattern-2 single-variant 4-arm MutationBudgetedRunKind:
// CiCleared / CiFailed / SurvivorDirective / CacheHit. 4 distinct sql_str values:
// "mutation_budgeted_run_ci_cleared" / "..._ci_failed" / "..._survivor_directive" /
// "..._cache_hit". Default-ON; kill-switch ELAI_MUTATION_BUDGET_DISABLE=1. CiCleared is
// the single representative arm in SSB_ACTION_TYPES. TOTAL_VARIANTS 563 -> 564.
// I-9 (SCHEMA_VERSION 263, 2026-06-14): ErrorAmplificationMeasured -- error-amplification
// (alpha_t) telemetry beacon (Track 13, lever PROVABLE). FLAT beacon (InsecureRateMeasured
// shape; NOT Pattern-2), one sql_str "error_amplification_measured". Bench telemetry, not a
// gate (no ELAI_*_ENABLE flag). Emitted from the `elai bench error-amplification` CLI seam.
// SSB_ACTION_TYPES len 571 -> 572; SCHEMA_VERSION derives -309 + 572 = 263; TOTAL_VARIANTS
// + the golden auto-follow (no hand-edited literal; regen the golden by mechanism).
// I-36 (SCHEMA_VERSION 267, 2026-06-14): PromptCompiled Pattern-2 single-variant 3-arm
// (PromptCompiledKind: Compiled/TransferGapMeasured/Rolledback). SSB_ACTION_TYPES adds one
// representative arm: PromptCompiled(PromptCompiledKind::Compiled). len 575 -> 576;
// SCHEMA = -309 + 576 = 267. TOTAL_VARIANTS + golden auto-follow (regen the golden).
const TOTAL_VARIANTS: usize = ALL_VARIANTS.len();

/// All `SsbActionType` variants, DERIVED from the lib's canonical list
/// (`elai_orchestrator::ssb::SSB_ACTION_TYPES`). Tier-B Move 2: the hand-locked
/// 562-entry array moved into ssb.rs as the single source of truth. Append new
/// variants THERE; `TOTAL_VARIANTS`, this alias, and the golden then update
/// automatically. The exhaustive_match_guard below still forces a compile error
/// if a new enum variant is added without being handled.
const ALL_VARIANTS: &[SsbActionType] = elai_orchestrator::ssb::SSB_ACTION_TYPES;

/// Compile-time exhaustiveness check: if a new variant lands in ssb.rs without
/// being added to ALL_VARIANTS, this match still compiles (it is exhaustive on
/// the enum), but ALL_VARIANTS will be shorter than the runtime count and the
/// `_count_is_locked` test fires. Inverse: removing a variant from the enum
/// without removing it from ALL_VARIANTS produces an "unknown variant" build
/// error at this match site. Either direction surfaces drift loudly.
#[allow(dead_code)]
fn exhaustive_match_guard(v: SsbActionType) {
    match v {
        SsbActionType::FileWrite => (),
        SsbActionType::FileRead => (),
        SsbActionType::ToolResult => (),
        SsbActionType::TestOutcome => (),
        SsbActionType::Finding => (),
        SsbActionType::Claim => (),
        SsbActionType::Checkpoint => (),
        SsbActionType::ModelSwap => (),
        SsbActionType::PhaseTransition => (),
        SsbActionType::KnowledgeDegradation => (),
        SsbActionType::IsolationMetric => (),
        SsbActionType::PlanUpdate => (),
        SsbActionType::ReplanRequest => (),
        SsbActionType::StubReference => (),
        SsbActionType::ReasoningTrace => (),
        SsbActionType::PhaseComplete => (),
        SsbActionType::MemoryMonitoring => (),
        SsbActionType::WorkloadProfile => (),
        SsbActionType::DeathEvent => (),
        SsbActionType::RecoveryEvent => (),
        SsbActionType::SupervisionExhausted => (),
        SsbActionType::SpiralDetected => (),
        SsbActionType::ContextReduction => (),
        SsbActionType::BranchMerge => (),
        SsbActionType::MistakeEntry => (),
        SsbActionType::ResearchStep => (),
        SsbActionType::ResearchDecomposeLevel => (),
        SsbActionType::WebResearchStep => (),
        SsbActionType::PreviewAvailable => (),
        SsbActionType::InterventionRequest => (),
        SsbActionType::InterventionResponse => (),
        SsbActionType::TaskSubmitted => (),
        SsbActionType::HookEvent => (),
        SsbActionType::ProviderSignals => (),
        SsbActionType::SearchTriggerFired => (),
        SsbActionType::VramTierDetected => (),
        SsbActionType::IndexingProgress => (),
        SsbActionType::DkgReady => (),
        SsbActionType::TieredAnalysis => (),
        SsbActionType::ShortCoderCompression => (),
        SsbActionType::ToolDispatch => (),
        SsbActionType::ValidationTier => (),
        SsbActionType::FailureClassified => (),
        SsbActionType::GitCommit => (),
        SsbActionType::DairaTrace => (),
        SsbActionType::SystemProbe => (),
        SsbActionType::FeasibilityReport => (),
        SsbActionType::NovelStepDetected => (),
        SsbActionType::StubRun => (),
        SsbActionType::GateDecision => (),
        SsbActionType::TaskSplit => (),
        SsbActionType::EscalationFired => (),
        SsbActionType::ModeChanged => (),
        SsbActionType::ProposalGenerated => (),
        SsbActionType::ProposalApplied => (),
        SsbActionType::ProposalRejected => (),
        SsbActionType::MsiUpdate => (),
        SsbActionType::ConfigArchived => (),
        SsbActionType::ProfileSelected => (),
        SsbActionType::FsmTransition => (),
        SsbActionType::MicroReflection => (),
        SsbActionType::TelescopeCollapse => (),
        SsbActionType::ConfigWarning => (),
        SsbActionType::ConstraintsInjected => (),
        SsbActionType::InterventionResolved => (),
        SsbActionType::DkgQuerierSwapped => (),
        SsbActionType::SteeringComputed => (),
        SsbActionType::SteeringComputeFailed => (),
        SsbActionType::McpServerConnected => (),
        SsbActionType::McpToolCalled => (),
        SsbActionType::McpServerDisconnected => (),
        SsbActionType::ChatMessage => (),
        SsbActionType::ChatResponse => (),
        SsbActionType::ExecutionModeChanged => (),
        SsbActionType::ContextBudgetComputed => (),
        SsbActionType::SsbFilteredByOverlap => (),
        SsbActionType::HookLatencyViolation => (),
        SsbActionType::ResearchPathChosen => (),
        SsbActionType::GroundingCacheEvent => (),
        SsbActionType::CircuitBreakerStateChange => (),
        SsbActionType::BubbleUpTyped => (),
        SsbActionType::IntegrationCheckpointSuccess => (),
        SsbActionType::DagRepairRound => (),
        SsbActionType::ConceptDecompositionNode => (),
        SsbActionType::EvidenceGateEvaluated => (),
        SsbActionType::SupervisionEvent => (),
        SsbActionType::BspSuperstep => (),
        SsbActionType::AgentLifecycle => (),
        SsbActionType::AntiSpiralHit => (),
        SsbActionType::CloudSpend => (),
        SsbActionType::EscalateToCloud => (),
        SsbActionType::DataTableGrew => (),
        SsbActionType::L3Proposal => (),
        SsbActionType::ConvergenceStagnant => (),
        SsbActionType::ConfigOscillation => (),
        SsbActionType::RegressionDetected => (),
        SsbActionType::EvolutionWithheld => (),
        SsbActionType::ReflectTriaged => (),
        SsbActionType::ReflectCandidate => (),
        SsbActionType::ReflectAuditVerdict => (),
        SsbActionType::ReflectRefined => (),
        SsbActionType::MediaValidationStarted => (),
        SsbActionType::MediaTier1Complete => (),
        SsbActionType::MediaTier2Complete => (),
        SsbActionType::RenderComplete => (),
        SsbActionType::VisualFeedbackIteration => (),
        SsbActionType::MediaHumanReviewRequired => (),
        SsbActionType::GenerationModeSelected => (),
        SsbActionType::StreamingEarlyTerminated => (),
        SsbActionType::PipelinePreAssembled => (),
        SsbActionType::DependencyResolved => (),
        SsbActionType::MicroStepCreated => (),
        SsbActionType::LanguageGrammarStatus => (),
        SsbActionType::SymbolQuery => (),
        SsbActionType::MinifiedFileChunked => (),
        SsbActionType::IndexCheckpoint => (),
        SsbActionType::IndexResume => (),
        SsbActionType::IndexCancel => (),
        SsbActionType::LockAcquired => (),
        SsbActionType::LockContested => (),
        SsbActionType::SymbolGraphUpdated => (),
        SsbActionType::IdeaFramingStarted => (),
        SsbActionType::ProjectClassClassified => (),
        SsbActionType::ScopeCeilingAdvisory => (),
        SsbActionType::CriticRoundComplete => (),
        SsbActionType::IdeaFramed => (),
        SsbActionType::TradeoffCardEmitted => (),
        SsbActionType::EntryPointsDetected => (),
        SsbActionType::ArtifactsIndexed => (),
        SsbActionType::ArtifactSearched => (),
        SsbActionType::ArtifactStale => (),
        SsbActionType::AmbiguityScored => (),
        SsbActionType::AmbiguityFloorFailed => (),
        SsbActionType::AmbiguityStreakAdvanced => (),
        SsbActionType::CosmosRefreshStarted => (),
        SsbActionType::CosmosArtifactUpdated => (),
        SsbActionType::CosmosArtifactDecayed => (),
        SsbActionType::CosmosGroundingUsed => (),
        SsbActionType::PerLevelAuditStarted => (),
        SsbActionType::PerLevelAuditVerdict => (),
        SsbActionType::PerLevelAuditCascade => (),
        SsbActionType::RuntimeFallback => (),
        SsbActionType::RuntimeHealthChanged => (),
        SsbActionType::CliSubprocessSpawned => (),
        SsbActionType::CliSubprocessKilled => (),
        SsbActionType::MlxSubprocessSpawned => (),
        SsbActionType::MlxSubprocessKilled => (),
        SsbActionType::LlamaCppSubprocessSpawned => (),
        SsbActionType::LlamaCppSubprocessKilled => (),
        SsbActionType::MlxApplePlatformDetected => (),
        SsbActionType::LlamaCppEpDetected => (),
        SsbActionType::RuntimeChainBuilt => (),
        SsbActionType::DeepPoolCapped => (),
        SsbActionType::RrfQualityGateFired => (),
        SsbActionType::HydeSkippedCodeQuery => (),
        SsbActionType::RrfComponentWeights => (),
        SsbActionType::GrcStepEmitted => (),
        SsbActionType::GroundingSweepResult => (),
        SsbActionType::CsqEvaluated => (),
        SsbActionType::AuditorRubberStampRejected => (),
        SsbActionType::PerRoleDecodingApplied => (),
        SsbActionType::HydeQuery => (),
        SsbActionType::VectorCacheHitRate => (),
        SsbActionType::QueryTrace => (),
        SsbActionType::PhaseNodeReady => (),
        SsbActionType::StepNodeReady => (),
        SsbActionType::BabyStepReady => (),
        SsbActionType::DagRepairEscalated => (),
        SsbActionType::AgentStatusChanged => (),
        SsbActionType::StagnationSpinningDetected => (),
        SsbActionType::StagnationOscillationDetected => (),
        SsbActionType::LateralThinkingActivated => (),
        SsbActionType::GoalDriftWarning => (),
        SsbActionType::WatchdogTimeout => (),
        SsbActionType::ChangeDecision => (),
        SsbActionType::SearchModeChanged => (),
        SsbActionType::RedactionApplied => (),
        SsbActionType::LateralThinkingSucceeded => (),
        SsbActionType::LateralThinkingExhausted => (),
        SsbActionType::GoalDriftBlock => (),
        SsbActionType::WipSquashComplete => (),
        SsbActionType::AlternativeExplored => (),
        SsbActionType::FeasibilityEscalated => (),
        SsbActionType::BranchForked => (),
        SsbActionType::BranchMerged => (),
        SsbActionType::BranchClosed => (),
        SsbActionType::PlanSkeleton => (),
        SsbActionType::CompiledPipelineFired => (),
        SsbActionType::TemplateCrystallized => (),
        SsbActionType::BackgroundTaskStarted => (),
        SsbActionType::BackgroundTaskComplete => (),
        SsbActionType::DecompileTriggered => (),
        SsbActionType::FutilityDetected => (),
        SsbActionType::FutilityOutputStagnation => (),
        SsbActionType::WiringEvidence => (),
        SsbActionType::WiringViolation => (),
        SsbActionType::DeterminismDrift => (),
        SsbActionType::MutationSurvived => (),
        SsbActionType::IntentConfirmed => (),
        SsbActionType::MrViolated => (),
        SsbActionType::SurgicalPlanCreated => (),
        SsbActionType::SurgicalLocWarning => (),
        SsbActionType::OracleCacheHit => (),
        SsbActionType::OracleCacheMiss => (),
        SsbActionType::ProjectDetectionSignal => (),
        SsbActionType::SkeletonGenerated => (),
        SsbActionType::SkeletonFlushed => (),
        SsbActionType::MiniCriticVerdict => (),
        SsbActionType::StructuralRetryStageAttempt => (),
        SsbActionType::BridgeFileGenerated => (),
        SsbActionType::ArchiveEntryAdded => (),
        SsbActionType::ArchiveEntryRetired => (),
        SsbActionType::TemplatePromoted => (),
        SsbActionType::TemplateCandidateRejected => (),
        SsbActionType::MemoryFactExtracted => (),
        SsbActionType::MemoryFactSuperseded => (),
        SsbActionType::PpmScored => (),
        SsbActionType::PpmGatingFailed => (),
        SsbActionType::ChatReflectionCompleted => (),
        SsbActionType::MemoryEvolutionRun => (),
        SsbActionType::MemoryNoteInserted => (),
        SsbActionType::MemoryNoteEvolved => (),
        SsbActionType::MemoryConflictDetected => (),
        SsbActionType::CoreBlockReplaced => (),
        SsbActionType::VerifierConsensusPass => (),
        SsbActionType::VerifierConsensusFail => (),
        SsbActionType::TripwireFired => (),
        SsbActionType::TemplateAutoReverted => (),
        SsbActionType::PrmBenchRunCompleted => (),
        SsbActionType::BaiArmUpdated => (),
        SsbActionType::BaiPromotionDecision => (),
        SsbActionType::HeldOutTaskAdded => (),
        SsbActionType::HeldOutTaskUsed => (),
        SsbActionType::CanaryRunCompleted => (),
        SsbActionType::CanaryFamilyEvaluated => (),
        SsbActionType::LockedKeysAuditRun => (),
        SsbActionType::LockedKeysViolationDetected => (),
        SsbActionType::TaskAdvanced => (),
        SsbActionType::AgentlessShortcutTaken => (),
        SsbActionType::GoalDriftFired => (),
        SsbActionType::ClaimRouted => (),
        SsbActionType::ModeRoutingBenchRunCompleted => (),
        SsbActionType::CheckerResultEmitted => (),
        SsbActionType::AdversaryGenerated => (),
        SsbActionType::ConsensusReevaluatedWithCounter => (),
        SsbActionType::BabyStepDispatched => (),
        SsbActionType::ConformalTauComputed => (),
        SsbActionType::ConformalRefusalDecided => (),
        SsbActionType::NoveltyClassified => (),
        SsbActionType::ObservableEffectVerified => (),
        SsbActionType::RefusalRateSnapshot => (),
        SsbActionType::RefusalSlaBreach => (),
        SsbActionType::ConformalValidationCompleted => (),
        SsbActionType::ArcBenchmarkCompleted => (),
        SsbActionType::ToolGated => (),
        SsbActionType::ToolEager => (),
        SsbActionType::ToolLazyResolved => (),
        SsbActionType::ToolDenied => (),
        SsbActionType::QuantBackendSelected => (),
        SsbActionType::KvCacheDtypeSoftwareSim => (),
        SsbActionType::AwqCheckpointBuilt => (),
        SsbActionType::CliMcpDispatch => (),
        SsbActionType::CliMcpServerStarted => (),
        SsbActionType::CliMcpServerStopped => (),
        SsbActionType::FirecrackerRestoreSlow => (),
        SsbActionType::SnapshotUniquenessRegenerated => (),
        SsbActionType::SnapshotEvicted => (),
        SsbActionType::SandboxBackendSelected => (),
        SsbActionType::AgentForkedFromSnapshot => (),
        SsbActionType::ResearcherNetEgressBlocked => (),
        SsbActionType::McpSchemaPinViolation => (),
        SsbActionType::McpSanitizationRuleApplied => (),
        SsbActionType::McpBudgetEnforced => (),
        SsbActionType::McpLegacySseTransport => (),
        SsbActionType::McpLlmGuardBlocked => (),
        SsbActionType::AnthropicGatewayDetected => (),
        SsbActionType::MemoryTierAssigned => (),
        SsbActionType::MemoryQueryFiltered => (),
        SsbActionType::SafetyCriticalBlocked => (),
        SsbActionType::ArgvDestructiveDenied => (),
        SsbActionType::ArgvApprovalRequired => (),
        SsbActionType::TrustDialogHookDeferred => (),
        SsbActionType::TrustDialogEnvDeferred => (),
        SsbActionType::McpPinVersionRefused => (),
        SsbActionType::AuditorLineageCollision => (),
        SsbActionType::UntrustedMemoryWriteBlocked => (),
        SsbActionType::IncidentRecordCreated => (),
        SsbActionType::FitmCalibrationApplied => (),
        SsbActionType::DeepModeActivated => (),
        SsbActionType::DeepModeEscalationChosen => (),
        SsbActionType::YarnRefused => (),
        // B-19 audit follow-up: close the 9-variant pre-existing drift.
        SsbActionType::DepAuditCompleted => (),
        SsbActionType::BaselineRunCompleted => (),
        SsbActionType::DeterministicAuditRun => (),
        SsbActionType::RuleTransition => (),
        SsbActionType::CanaryDrift => (),
        SsbActionType::TreePositionChanged => (),
        SsbActionType::FailureRecorded => (),
        SsbActionType::PackageRefVerified => (),
        SsbActionType::TechStackDecided => (),
        SsbActionType::MarkerReport => (),
        SsbActionType::EmpireConnected => (),
        SsbActionType::EmpireDisconnected => (),
        SsbActionType::EmpireEventReceived => (),
        SsbActionType::EmpireCapabilityInvoked => (),
        SsbActionType::EmpireRepairProposed => (),
        // C-5 follow-on (SCHEMA_VERSION 103).
        SsbActionType::RouteBlocked => (),
        SsbActionType::WarnLimitReached => (),
        SsbActionType::CostEstimate => (),
        // C-36 (SCHEMA_VERSION 104).
        SsbActionType::MucocoMutationApplied => (),
        // D-3 closure (SCHEMA_VERSION 105).
        SsbActionType::PpmUnavailable => (),
        SsbActionType::VerifierDisagreement => (),
        SsbActionType::CrossContextDispatched => (),
        // C-38 (SCHEMA_VERSION 107).
        SsbActionType::PagerankRun => (),
        SsbActionType::DkgRebuildStarted => (),
        SsbActionType::DkgRebuildComplete => (),
        // D-4 GAP-2 (SCHEMA_VERSION 108): concurrent-detector tripwire row.
        SsbActionType::TripwireFiredConcurrent => (),
        // C-40 (SCHEMA_VERSION 109): GitHub PR adapter producer surface.
        SsbActionType::GithubPrOpened => (),
        SsbActionType::GithubPrStatusFetched => (),
        SsbActionType::GithubPrClosed => (),
        SsbActionType::GithubPrMerged => (),
        // E-41 (SCHEMA_VERSION 135): GitHub PR review surface.
        SsbActionType::PrReviewComposed => (),
        SsbActionType::PrCommentWatched => (),
        SsbActionType::PrPackageVerifyResult => (),
        SsbActionType::PrSemgrepGateResult => (),
        SsbActionType::PrAutoMergeBlocked => (),
        // D-5 closure (SCHEMA_VERSION 110): Thompson-sample beacon.
        SsbActionType::ThompsonSampled => (),
        // C-41 (SCHEMA_VERSION 111): skills-driven workflow registry.
        SsbActionType::SkillTriggered => (),
        SsbActionType::SkillExecuted => (),
        // D-6 closure (SCHEMA_VERSION 112): held-out promotion-gate +
        // refresh-event beacons.
        SsbActionType::HeldOutPromotionGate => (),
        SsbActionType::HeldOutRefresh => (),
        // C-42 (SCHEMA_VERSION 113): Semantic Gateway L2 RBAC beacons.
        SsbActionType::GatewayPolicyAllowed => (),
        SsbActionType::GatewayPolicyDenied => (),
        SsbActionType::GatewayHitlPending => (),
        // D-7 closure (SCHEMA_VERSION 114): cross-domain canary observation
        // beacons. ELAI design hypothesis (NanoFlux arXiv:2509.23252
        // empirical motivation only; no canary mechanism in paper).
        SsbActionType::CrossDomainRegression => (),
        SsbActionType::CanaryFamilyPass => (),
        SsbActionType::CanaryInsufficientData => (),
        // D-8 closure (SCHEMA_VERSION 115): cryptographic archive verify.
        SsbActionType::ArchiveVerifyResult => (),
        // C-43 closure (SCHEMA_VERSION 116): Phase 4H scope-freeze.
        SsbActionType::ScopeFreezeRatified => (),
        SsbActionType::ScopeFreezeViolated => (),
        // C-44 (SCHEMA_VERSION 117): Composer 2 paid-backend cost accrual.
        SsbActionType::BackendCostAccrued => (),
        // C-45 (SCHEMA_VERSION 120): CANTANTE credit attribution.
        SsbActionType::AgentRoleDropped => (),
        SsbActionType::CreditAttributionRun => (),
        SsbActionType::CreditAttributionResult => (),
        // C-46 (SCHEMA_VERSION 121): Uno-Orchestra selective delegation.
        SsbActionType::SelectiveDelegationDecision => (),
        SsbActionType::SelectiveDelegationEscalated => (),
        SsbActionType::SelectiveDelegationCostSaved => (),
        // C-47 (SCHEMA_VERSION 122): Case-Based Calibration of Tool-Use.
        SsbActionType::ToolCallLogged => (),
        SsbActionType::ToolCalibrationHintEmitted => (),
        SsbActionType::ToolCalibrationDriftDetected => (),
        // C-49 (SCHEMA_VERSION 123): iterative-distillation experience.
        SsbActionType::PromptPolicyDistillationRun => (),
        SsbActionType::PromptFragmentSelected => (),
        SsbActionType::PromptFragmentDiscarded => (),
        // D-14 (SCHEMA_VERSION 124): sycophancy probe row pair.
        SsbActionType::SycophancyProbeDispatched => (),
        SsbActionType::SycophancyFlag => (),
        // D-23 (SCHEMA_VERSION 142): sycophancy detection benchmark.
        SsbActionType::SycophancyBenchRunCompleted => (),
        // D-24 (SCHEMA_VERSION 143): cross-domain canary regression
        // benchmark.
        SsbActionType::CanaryBenchRunCompleted => (),
        // D-25 (SCHEMA_VERSION 144): determinism + archive tamper-suite
        // benchmark.
        SsbActionType::DeterminismTamperSuiteCompleted => (),
        // D-28 (SCHEMA_VERSION 145): adversarial example farming run.
        SsbActionType::AdversarialFarmRunCompleted => (),
        // D-29 (SCHEMA_VERSION 146): hierarchical BAI round completion.
        SsbActionType::HierarchicalBaiRoundCompleted => (),
        // D-30 (SCHEMA_VERSION 147): calibration bootstrap warm-start.
        SsbActionType::CalibrationBootstrapCompleted => (),
        // C-50 (SCHEMA_VERSION 125): Contract-Coding scaffolding pair.
        SsbActionType::ContractTestScaffolded => (),
        SsbActionType::ContractViolationDetected => (),
        // C-52 (SCHEMA_VERSION 126): Dafny Formal Verification Spike.
        SsbActionType::DafnyVerifyCompleted => (),
        // D-16 closure (SCHEMA_VERSION 127): durable TauUpdater audit row.
        SsbActionType::CalibrationCorpusUpdated => (),
        // E-12 Tier-B (SCHEMA_VERSION 128): production fullstack codegen.
        SsbActionType::FrameworkProfileSelected => (),
        SsbActionType::LegacyConfigStripped => (),
        // D-18 closure (SCHEMA_VERSION 129): typed-ObservableEffect 4-pack.
        SsbActionType::ObservableEffectDispatched => (),
        SsbActionType::GoodhartRiskFlag => (),
        SsbActionType::SandboxObservation => (),
        SsbActionType::ProductionObservationProposal => (),
        // E-13 (SCHEMA_VERSION 130): TDD verifier-agent beacons.
        SsbActionType::TddCycleStarted => (),
        SsbActionType::TddTestFailed => (),
        SsbActionType::TddTestPassed => (),
        // Tranche B-7 triple (SCHEMA_VERSION 195): TDD verification gate.
        SsbActionType::TddVerificationEvent => (),
        // E-14 (SCHEMA_VERSION 131): browser-e2e agent beacons.
        SsbActionType::BrowserE2eRunStarted => (),
        SsbActionType::BrowserE2eAssertionFailed => (),
        SsbActionType::BrowserE2eAssertionPassed => (),
        SsbActionType::BrowserE2eSnapshotCaptured => (),
        // E-18 (SCHEMA_VERSION 132): tool-use reliability gate beacons.
        SsbActionType::ToolReliabilityGatePassed => (),
        SsbActionType::ToolReliabilityGateRejected => (),
        SsbActionType::ToolCallSpilled => (),
        SsbActionType::ToolCallRetryEscalated => (),
        // E-37 (SCHEMA_VERSION 133): prompt-injection defense beacons.
        SsbActionType::InjectionAttemptDetected => (),
        SsbActionType::InjectionAttemptBlocked => (),
        SsbActionType::SpotlightWrapped => (),
        SsbActionType::SpotlightBreached => (),
        SsbActionType::ToolDescriptionPinMismatch => (),
        SsbActionType::CapabilityGateRejected => (),
        SsbActionType::ProvenanceTagged => (),
        SsbActionType::PromptArmorVerdict => (),
        SsbActionType::InjectionFixtureReplayed => (),
        SsbActionType::OutputClassifierFlagged => (),
        // E-40 (SCHEMA_VERSION 134): orchestrator-rooted symbol-graph beacons.
        SsbActionType::SymbolGraphQueried => (),
        SsbActionType::SymbolGraphCacheLoaded => (),
        SsbActionType::LspOracleConsulted => (),
        // E-44 (SCHEMA_VERSION 136): durability checkpoint + idempotent dispatch +
        // compact-lock beacons.
        SsbActionType::CheckpointWritten => (),
        SsbActionType::CheckpointResumed => (),
        SsbActionType::CheckpointMigrated => (),
        SsbActionType::IdempotentDispatchSkipped => (),
        SsbActionType::DurabilityLockBlocked => (),
        // E-45 (SCHEMA_VERSION 137): cross-language drift detector beacons.
        SsbActionType::CrossLangDriftDetected => (),
        SsbActionType::CrossLangPairScanned => (),
        SsbActionType::SchemaVersionMismatch => (),
        // E-49 (SCHEMA_VERSION 138): coding-agent metric suite beacons.
        SsbActionType::CodingAgentMetricsCaptured => (),
        SsbActionType::SweBenchProRunStarted => (),
        SsbActionType::SweBenchProRunCompleted => (),
        SsbActionType::RefineBenchScoreEmitted => (),
        SsbActionType::MetricsAggregationRun => (),
        // E-52 Tier-B (SCHEMA_VERSION 139): LSP-as-verifier diagnostic
        // probe + SCIP refresh beacons.
        SsbActionType::DiagnosticProbeDispatched => (),
        SsbActionType::DiagnosticProbeCompleted => (),
        SsbActionType::DiagnosticProbeBlock => (),
        SsbActionType::ScipRefreshCompleted => (),
        // E-53 (SCHEMA_VERSION 140): formal-verify pipeline boundary beacon.
        SsbActionType::FormalVerifierCompleted => (),
        // E-35 (SCHEMA_VERSION 141): structured-output substrate beacons.
        SsbActionType::XgrammarMaskApplied => (),
        SsbActionType::InstructorModeActivated => (),
        SsbActionType::GrammarModeActivated => (),
        SsbActionType::CodeactActionExecuted => (),
        SsbActionType::SearchReplaceTier1Hit => (),
        SsbActionType::SearchReplaceCollisionFallback => (),
        SsbActionType::CloudStrictSchemaApplied => (),
        SsbActionType::GrammarCacheLoaded => (),
        SsbActionType::StructuredEngineRejected => (),
        SsbActionType::LegacyJsonObjectRefused => (),
        // E-6 (SCHEMA_VERSION 148): MAST-grounded multi-agent failure-mode defenses.
        SsbActionType::AuditMastTagged => (),
        SsbActionType::IsoBudgetMeasured => (),
        SsbActionType::PersonaReanchored => (),
        SsbActionType::CrossFamilyUnavailable => (),
        // E-7 (SCHEMA_VERSION 149): darwinian self-improvement substrate.
        SsbActionType::DarwinianGenerationCompleted => (),
        // batch2-merge-collision recovery (SCHEMA_VERSION 150-156): 9
        // variants re-applied after parallel-executor write collision in
        // cluster s-25120 batch 2.
        SsbActionType::RuntimeHandoffSignal => (),
        SsbActionType::SglangBackendRegistered => (),
        SsbActionType::XgrammarSchemaCompiled => (),
        SsbActionType::RuntimeColdStartObserved => (),
        SsbActionType::StreamingTimingRecorded => (),
        SsbActionType::UxPacingVoteRecorded => (),
        SsbActionType::StreamingLlmEngaged => (),
        SsbActionType::SdpaBackendPinned => (),
        SsbActionType::YarnRopeScalingApplied => (),
        // E-8 (SCHEMA_VERSION 162): self-evolving prompts substrate.
        SsbActionType::PromptOptimizationCompleted => (),
        // E-10 (SCHEMA_VERSION 163): hostile-critic structural simplifications.
        SsbActionType::CriticDecisionEvent => (),
        // E-43 (SCHEMA_VERSION 164): agent skill systems telemetry.
        SsbActionType::SkillSystemEvent => (),
        // E-9 (SCHEMA_VERSION 165): external agent-eval benchmark suite.
        SsbActionType::BenchmarkRunObserved => (),
        // E-19 (SCHEMA_VERSION 166): reasoning quality probe.
        SsbActionType::ReasoningQualityProbe => (),
        // E-54 (SCHEMA_VERSION 167): eval-corpus repair pre-deploy.
        SsbActionType::EvalCorpusEvent => (),
        // E-55 (SCHEMA_VERSION 168): agentic RL + reward ladder substrate.
        SsbActionType::RewardLadderEvent => (),
        // E-11 (SCHEMA_VERSION 169): SaaS prior-art catalog + recommender.
        SsbActionType::SaasPriorArtEvent => (),
        // E-22 (SCHEMA_VERSION 170): may-2026 github trending sweep
        // substrate.
        SsbActionType::GithubTrendingEvent => (),
        // E-23 (SCHEMA_VERSION 171): 2026 developer-discourse positioning
        // + UX-guard substrate.
        SsbActionType::DeveloperDiscourseEvent => (),
        // E-25 (SCHEMA_VERSION 172): claude-code-architecture catalog
        // substrate.
        SsbActionType::ClaudeCodeArchitectureEvent => (),
        // E-29 Tier-B (SCHEMA_VERSION 173): agentic-observability event
        // substrate.
        SsbActionType::AgenticObservabilityEvent => (),
        // E-31 (SCHEMA_VERSION 174): coding-model-leaderboard scraper
        // substrate.
        SsbActionType::CodingModelLeaderboardEvent => (),
        // E-38 (SCHEMA_VERSION 175): 2026 AI roadmap signals + EU AI Act
        // compliance pre-gate substrate.
        SsbActionType::AiRoadmapSignalEvent => (),
        // E-39 (SCHEMA_VERSION 176): cost + token economics substrate.
        SsbActionType::CostTokenEvent => (),
        // E-50 (SCHEMA_VERSION 177): SaaS architecture pattern catalog
        // substrate.
        SsbActionType::SaasArchPatternEvent => (),
        // E-4 (SCHEMA_VERSION 178): chat UX critique substrate.
        SsbActionType::ChatUxCritiqueEvent => (),
        // E-16 (SCHEMA_VERSION 179): VLM image-validation hardening.
        SsbActionType::VlmImageValidationEvent => (),
        // E-20 (SCHEMA_VERSION 180): local-cloud routing substrate.
        SsbActionType::LocalCloudRoutingEvent => (),
        // E-21 (SCHEMA_VERSION 181): peer-systems harness substrate.
        SsbActionType::PeerSystemsEvent => (),
        // E-36 (SCHEMA_VERSION 182): document + vision RAG substrate.
        SsbActionType::DocumentVisionRagEvent => (),
        // E-42 (SCHEMA_VERSION 183): tree-sitter grammar health substrate.
        SsbActionType::TreeSitterGrammarEvent => (),
        // E-46 (SCHEMA_VERSION 184): planning-decomposition substrate.
        SsbActionType::PlanningDecompositionEvent => (),
        // E-48 (SCHEMA_VERSION 185): web-research orchestrator hardening
        // substrate.
        SsbActionType::WebResearchEvent => (),
        // E-44 (SCHEMA_VERSION 186): durability observability substrate.
        SsbActionType::DurabilityEvent => (),
        // E-31 (SCHEMA_VERSION 187): head_dim invariant verification substrate.
        SsbActionType::HeadDimInvariantVerified => (),
        // E-3 (SCHEMA_VERSION 188): role-shown / model-hidden default
        // telemetry substrate.
        SsbActionType::RoleShownModelHidden => (),
        // E-49 (SCHEMA_VERSION 189): coding-agent metrics named-token substrate.
        SsbActionType::CodingMetricNamedToken => (),
        // E-3 sub-phase 4A (SCHEMA_VERSION 190): runtime handoff event substrate.
        SsbActionType::E3RuntimeHandoffEvent => (),
        // E-31 sub-phase 6A (SCHEMA_VERSION 191): cloud-route decision event substrate.
        SsbActionType::RouteDecisionEvent => (),
        // E-47 (SCHEMA_VERSION 192): LoRA fine-tuning ship-gate substrate.
        SsbActionType::LoraShipGateEvent => (),
        // E-40 Tranche B-2 (SCHEMA_VERSION 193): SymbolGraph augmentation substrate.
        SsbActionType::SymbolGraphEvent => (),
        // Tranche B-6 quad (SCHEMA_VERSION 194): four new SSB substrates.
        SsbActionType::McpAuthzEvent => (),
        SsbActionType::ToolCalibrationEvent => (),
        SsbActionType::ReflectBudgetSoftWarn => (),
        SsbActionType::ContractCodingEvent => (),
        // F-1 / F-2a (SCHEMA_VERSION 196): vibe-coding security gate variants.
        SsbActionType::VibeGateBlocked(_) => (),
        SsbActionType::SecurityRuleBlocked => (),
        // F-4 / Slice B-1 (SCHEMA_VERSION 197): test integrity event.
        SsbActionType::TestIntegrityEvent(_) => (),
        // F-3p1 / Slice C1 (SCHEMA_VERSION 198): capability-flow tool-filter event.
        SsbActionType::ToolFilterEvent(_) => (),
        // F-9 / Slice D1 (SCHEMA_VERSION 199): write-time dep slopsquat blocked.
        SsbActionType::WriteTimeSlopsquatBlocked => (),
        // F-10 / Slice A1 (SCHEMA_VERSION 200): security-class regression blocked.
        SsbActionType::SecurityClassRegressionBlocked => (),
        // F-2b / Slice A1 (SCHEMA_VERSION 201): insecure-rate keystone bench beacon.
        SsbActionType::InsecureRateMeasured => (),
        // I-9 (SCHEMA_VERSION 263): ErrorAmplificationMeasured flat beacon.
        SsbActionType::ErrorAmplificationMeasured => (),
        SsbActionType::SemanticEntropyMeasured => (),
        SsbActionType::ContextRecalled => (),
        SsbActionType::HallucinationDeltaMeasured => (),
        // F-6 / Slice A1 (SCHEMA_VERSION 202): learned-skill library event.
        SsbActionType::SkillLibraryEvent(_) => (),
        // F-5 / Slice A1 (SCHEMA_VERSION 203): staleness-quarantine broadcast.
        SsbActionType::StalenessQuarantineBroadcast => (),
        // F-8 / Slice A1 (SCHEMA_VERSION 204): symbolic semantic conflict.
        SsbActionType::SymbolicConflictDetected => (),
        // F-7 / Slice A1 (SCHEMA_VERSION 205): CBR template utility ranking.
        SsbActionType::TemplateUtilityRanked => (),
        // F-13 / Slice A1 (SCHEMA_VERSION 206): session-archive-on-bump.
        SsbActionType::SessionArchived => (),
        // F-11 / Slice A1 (SCHEMA_VERSION 207): config-DGM admit-loop beacon.
        SsbActionType::DarwinianAdmit => (),
        // G-plan + H-plan repair (2026-06-07): new variants.
        SsbActionType::McpRugPullBlocked => (),
        SsbActionType::TaintFlow => (),
        SsbActionType::BlastRadiusBlocked => (),
        SsbActionType::TrifectaQuarantined => (),
        SsbActionType::RunArtifactEmitted => (),
        SsbActionType::ReviewSurfaceScored => (),
        SsbActionType::ProvenanceAttestationEmitted => (),
        SsbActionType::SafetyDeltaMeasured => (),
        SsbActionType::ModelDriftSuspected => (),
        SsbActionType::SecretRedacted => (),
        SsbActionType::ClaimConfidenceScored => (),
        SsbActionType::SteeringVectorRefreshTriggered => (),
        SsbActionType::CanvasOpApplied => (),
        SsbActionType::CanvasViewEvent => (),
        SsbActionType::CanvasActivityEvent => (),
        SsbActionType::BatchLifecycle => (),
        SsbActionType::DeflationGateEvaluated => (),
        SsbActionType::LeakageBlocked => (),
        SsbActionType::LookaheadDifferential => (),
        SsbActionType::TradingNullTest => (),
        SsbActionType::RerunHashEquality => (),
        SsbActionType::SimRealParity => (),
        SsbActionType::ResultManifestVerified => (),
        SsbActionType::DifferentialDivergence => (),
        SsbActionType::GateSelfValidation => (),
        SsbActionType::RewardTampering => (),
        SsbActionType::HoldoutBudgetEvent => (),
        SsbActionType::DriftInterlock => (),
        SsbActionType::TrialRegistered => (),
        SsbActionType::MucocoInconsistencyDetected => (),
        SsbActionType::MucocoSkipReason => (),
        SsbActionType::UserDecisionSuperseded => (),
        SsbActionType::StalenessEvalMeasured => (),
        // D-27.1 (SCHEMA_VERSION 209): discovery-queue reviewer-action beacons.
        SsbActionType::DiscoveryRatified => (),
        SsbActionType::DiscoveryRejected => (),
        // H-32 (SCHEMA_VERSION 210): FactValidationFailed flat beacon.
        SsbActionType::FactValidationFailed => (),
        // H-33 (SCHEMA_VERSION 211): MemoryGovernanceEvent flat beacon.
        SsbActionType::MemoryGovernanceEvent => (),
        // H-34 (SCHEMA_VERSION 212): McpToolDeferred deferred STDIO schema resolve.
        SsbActionType::McpToolDeferred => (),
        // H-34 (SCHEMA_VERSION 213): McpSchemaBudgetMeasured bench result beacon.
        SsbActionType::McpSchemaBudgetMeasured => (),
        // H-35 (SCHEMA_VERSION 214): SessionRewound session-rewind restore beacon.
        SsbActionType::SessionRewound => (),
        // H-36 (SCHEMA_VERSION 215): AgentNotificationEmitted dev-event notification beacon.
        SsbActionType::AgentNotificationEmitted => (),
        // H-37 (SCHEMA_VERSION 216): PlanDecisionEvent plan-approval gate decision.
        SsbActionType::PlanDecisionEvent => (),
        // H-38 (SCHEMA_VERSION 217): StopConfirmationEvent cost-confirm gate outcome.
        SsbActionType::StopConfirmationEvent => (),
        // H-39 (SCHEMA_VERSION 218): AbstentionScored abstention-aware scoring bench beacon.
        SsbActionType::AbstentionScored => (),
        // H-41 (SCHEMA_VERSION 219): CheckerGapDetected off-agent backlog beacon.
        SsbActionType::CheckerGapDetected => (),
        // H-42 (SCHEMA_VERSION 220): ReasoningBudgetScaled serial-depth-scaled GRC budget beacon.
        SsbActionType::ReasoningBudgetScaled => (),
        // H-43 (SCHEMA_VERSION 221): FitnessEvaluated execution-grounded DGM fitness beacon.
        SsbActionType::FitnessEvaluated => (),
        // H-44 (SCHEMA_VERSION 222): TrapBehaviorDetected behavioral cheat-detector beacon.
        SsbActionType::TrapBehaviorDetected => (),
        // H-45 (SCHEMA_VERSION 223): PromotionScoreAggregated worst-case/variance-penalized aggregate.
        SsbActionType::PromotionScoreAggregated => (),
        // H-46 (SCHEMA_VERSION 224): CladeSelected HGM clade-metaproductivity parent selection beacon.
        SsbActionType::CladeSelected => (),
        // H-47 (SCHEMA_VERSION 225): EvalFirewallEvent DGM eval firewall beacon (date-gate + regen + budget).
        SsbActionType::EvalFirewallEvent => (),
        // H-49 (SCHEMA_VERSION 226): EditHistoryEvent reversible LLM edits beacon (CAS capture + redo/branch/jump).
        SsbActionType::EditHistoryEvent => (),
        // H-51 (SCHEMA_VERSION 227): HandoffLifecycleEvent cross-session sleep-time handoff digest lifecycle beacon.
        SsbActionType::HandoffLifecycleEvent => (),
        // H-52 (SCHEMA_VERSION 228): FindingAnchored deterministic snippet->hunk anchoring beacon (elai review --cockpit).
        SsbActionType::FindingAnchored => (),
        // H-53 (SCHEMA_VERSION 229): CitationAnchorsBound citation pre-binding beacon (elai prebind --demo-prebind).
        SsbActionType::CitationAnchorsBound => (),
        // I-39 (SCHEMA_VERSION 230): PlanStateDerived per-plan state derived beacon (elai plan status).
        SsbActionType::PlanStateDerived => (),
        // I-45 (SCHEMA_VERSION 231): ObjectiveSealed Pattern-2 3-arm objective hash receipt beacon.
        SsbActionType::ObjectiveSealed(_) => (),
        // I-34 (SCHEMA_VERSION 234): GrammarGateEvent Pattern-2 3-arm CRANE grammar-pin gate beacon.
        SsbActionType::GrammarGateEvent(_) => (),
        // I-38 (SCHEMA_VERSION 235): QdeltaMeasured flat beacon.
        SsbActionType::QdeltaMeasured => (),
        SsbActionType::SnowballingMeasured => (),
        SsbActionType::KvCacheTuned => (),
        // I-43 (SCHEMA_VERSION 236): ClaimScopeMeasured Pattern-2 3-arm claim-scoper kill-gate.
        SsbActionType::ClaimScopeMeasured(_) => (),
        // I-10 (SCHEMA_VERSION 237): CacheEconomicsEvent Pattern-2 3-arm cache-economics guard.
        SsbActionType::CacheEconomicsEvent(_) => (),
        // I-8 (SCHEMA_VERSION 238): TrajectoryAborted Pattern-2 3-arm voluntary abort + dead-end + halting.
        SsbActionType::TrajectoryAborted(_) => (),
        // I-1 (SCHEMA_VERSION 239): DirectiveExecuted Pattern-2 3-arm directive supervisor outcome.
        SsbActionType::DirectiveExecuted(_) => (),
        // I-33 (SCHEMA_VERSION 240): GovernancePrimitiveFired Pattern-2 3-arm CEFL/I6Q/commit-reveal.
        SsbActionType::GovernancePrimitiveFired(_) => (),
        // I-32 (SCHEMA_VERSION 241): RoleContractApplied Pattern-2 3-arm persona-strip/retain/AB-delta.
        SsbActionType::RoleContractApplied(_) => (),
        // I-35 (SCHEMA_VERSION 242): ToolSequenceVerdict Pattern-2 3-arm temporal sequence enforcement.
        SsbActionType::ToolSequenceVerdict(_) => (),
        // I-41 (SCHEMA_VERSION 243): CritiqueLensFinding Pattern-2 3-arm advisory lens beacon.
        SsbActionType::CritiqueLensFinding(_) => (),
        // I-46 (SCHEMA_VERSION 244): BashRedirected Pattern-2 3-arm bash interceptor outcome.
        SsbActionType::BashRedirected(_) => (),
        // I-47 (SCHEMA_VERSION 245): PromptHardeningApplied Pattern-2 3-arm verifier-input hardening.
        SsbActionType::PromptHardeningApplied(_) => (),
        // I-2 (SCHEMA_VERSION 246): BranchWorkspaceEvent Pattern-2 4-arm CoW workspace barrier.
        SsbActionType::BranchWorkspaceEvent(_) => (),
        // I-5 (SCHEMA_VERSION 247): ContextPruned Pattern-2 3-arm per-branch context rebuild.
        SsbActionType::ContextPruned(_) => (),
        // I-11 (SCHEMA_VERSION 248): BeliefStateEvent Pattern-2 3-arm typed condensate receipt.
        SsbActionType::BeliefStateEvent(_) => (),
        // I-13 (SCHEMA_VERSION 249): MemoryGuardEvent Pattern-2 3-arm write-time guard outcome.
        SsbActionType::MemoryGuardEvent(_) => (),
        // I-14 (SCHEMA_VERSION 250): ContextTransformReceipt Pattern-2 3-arm compression receipt.
        SsbActionType::ContextTransformReceipt(_) => (),
        // I-12 (SCHEMA_VERSION 251): PlaybookEvent Pattern-2 4-arm playbook curator outcome.
        SsbActionType::PlaybookEvent(_) => (),
        // I-17 (SCHEMA_VERSION 252): TriLayerVerdict Pattern-2 4-arm per-branch verdict.
        SsbActionType::TriLayerVerdict(_) => (),
        // I-18 (SCHEMA_VERSION 253): RowChainSealed Pattern-2 3-arm per-row blake3 chain beacon.
        SsbActionType::RowChainSealed(_) => (),
        // I-19 (SCHEMA_VERSION 254): TddPreflightBlocked Pattern-2 3-arm TDD enforcer preflight gate.
        SsbActionType::TddPreflightBlocked(_) => (),
        SsbActionType::MutationBudgetedRun(_) => (),
        // I-21 (SCHEMA_VERSION 256): SprintContractEvent Pattern-2 5-arm sprint-contract + ralph-loop.
        SsbActionType::SprintContractEvent(_) => (),
        // I-37 (SCHEMA_VERSION 257): RequirementVerified Pattern-2 3-arm per-EARS-requirement status.
        SsbActionType::RequirementVerified(_) => (),
        // I-44 (SCHEMA_VERSION 258): RetentionPurged Pattern-2 3-arm zero-retention purge receipt.
        SsbActionType::RetentionPurged(_) => (),
        // I-52 (SCHEMA_VERSION 259): CheckpointToolEvent Pattern-2 4-arm agent checkpoint tool.
        SsbActionType::CheckpointToolEvent(_) => (),
        // I-54 (SCHEMA_VERSION 260): StructuralEditApplied Pattern-2 5-arm agent structural-edit tool.
        SsbActionType::StructuralEditApplied(_) => (),
        // I-4 (SCHEMA_VERSION 261): SearchNodeEvent Pattern-2 4-arm evidence-ranked search ladder.
        SsbActionType::SearchNodeEvent(_) => (),
        SsbActionType::EscalationCascade(_) => (),
        // I-22 (SCHEMA_VERSION 264): ProgressRewardScored Pattern-2 3-arm delta-progress reward scorer.
        SsbActionType::ProgressRewardScored(_) => (),
        // I-24 (SCHEMA_VERSION 266): PrmCanaryEvent Pattern-2 3-arm isomorphic canary anti-gaming.
        SsbActionType::PrmCanaryEvent(_) => (),
        // I-27 (SCHEMA_VERSION 265): ConvergenceCurveMeasured flat beacon.
        SsbActionType::ConvergenceCurveMeasured => (),
        // I-36 (SCHEMA_VERSION 268): PromptCompiled Pattern-2 3-arm (Compiled/TransferGapMeasured/Rolledback).
        // SSB_ACTION_TYPES carries the single representative PromptCompiled(PromptCompiledKind::Compiled) arm.
        SsbActionType::PromptCompiled(_) => (),
        // I-50 (SCHEMA_VERSION 268): AuthBrokerEvent Pattern-2 4-arm (TokenAcquired/Refreshed/ProviderLinked/RefreshFailed).
        // SSB_ACTION_TYPES carries the single representative AuthBrokerEvent(AuthBrokerEventKind::TokenAcquired) arm.
        SsbActionType::AuthBrokerEvent(_) => (),
        // J-1.T4 (SCHEMA_VERSION 269): PerceptionScaffoldEvent Pattern-2 4-arm single-variant
        // (Preprocessed/ScaffoldEmitted/RoutedToSpecialist/DegradedPassthrough).
        // SSB_ACTION_TYPES carries the single representative
        // PerceptionScaffoldEvent(PerceptionScaffoldEventKind::ScaffoldEmitted) arm.
        SsbActionType::PerceptionScaffoldEvent(_) => (),
        // K-18a follow-on (SCHEMA_VERSION 270): RecurrenceHitRateMeasured flat beacon
        // (no Kind discriminator). Emitted by bench_case_reuse::run_inner.
        SsbActionType::RecurrenceHitRateMeasured => (),
        // K-4 SP1 follow-on (SCHEMA_VERSION 271): GateTaxMeasured flat beacon
        // (no Kind discriminator). Emitted by bench_gate_tax::run_inner.
        SsbActionType::GateTaxMeasured => (),
        // J-28 follow-on (SCHEMA_VERSION 272): UsageTurnEvent flat variant
        // (no Kind discriminator on the enum; Kind rides in the payload).
        // Emitted by grounding_client.rs (cloud) + runtime_orchestrator.rs (local).
        SsbActionType::UsageTurnEvent => (),
        // J-9 (SCHEMA_VERSION 273): hardware/model catalog flat variant.
        SsbActionType::HardwareModelCatalogEvent => (),
        // K-2 (SCHEMA_VERSION 274): routing economics benchmark flat beacon.
        SsbActionType::RoutingEconomicsMeasured => (),
        // K-5 (SCHEMA_VERSION 275): gate intensity selector benchmark flat beacon.
        SsbActionType::GateIntensitySelected => (),
        // K-9 (SCHEMA_VERSION 276): model refresh cadence decision row.
        SsbActionType::ModelRefreshCadence => (),
        SsbActionType::LocalLaneE2eMeasured => (),
        SsbActionType::McpScanClassifierBlocked => (),
        SsbActionType::LocalDeterminismEvent(_) => (),
        // J-14 (SCHEMA_VERSION 282): ReproPackEvent Pattern-2 4-arm repro-pack + RRA verify upgrade beacon.
        SsbActionType::ReproPackEvent(_) => (),
        // J-16 (SCHEMA_VERSION 283): BlastRadiusRegressionEvent Pattern-2 4-arm blast-radius / anti-regression gate.
        SsbActionType::BlastRadiusRegressionEvent(_) => (),
        // J-30 T7b (SCHEMA_VERSION 284): ProviderByokEvent flat variant, Kind rides in the payload.
        SsbActionType::ProviderByokEvent => (),
        // J-32 (SCHEMA_VERSION 285): IntentRouted flat variant, Kind rides in the payload.
        SsbActionType::IntentRouted => (),
        // J-34 (SCHEMA_VERSION 286): ContextCalibrationEvent flat variant, Kind rides in the payload.
        SsbActionType::ContextCalibrationEvent(_) => (),
        // J-36 (SCHEMA_VERSION 287): LocalPrefixCacheEvent flat variant, Kind rides in the payload.
        SsbActionType::LocalPrefixCacheEvent(_) => (),
        // J-24 (SCHEMA_VERSION 288): BehaviorSteeringEvent flat variant, Kind rides in the payload.
        SsbActionType::BehaviorSteeringEvent(_) => (),
        // J-35 (SCHEMA_VERSION 289): IntentControlEvent flat variant, Kind rides in the payload.
        SsbActionType::IntentControlEvent(_) => (),
        // J-39 T2: NodeAdmissionEvent flat variant, Kind rides in the payload.
        SsbActionType::NodeAdmissionEvent(_) => (),
        SsbActionType::SystemContractEvent(_) => (),
        SsbActionType::HarnessInterventionEvent(_) => (),
        // J-54 UNIT8: FreePoolEvent flat variant, Pattern-2 single-variant.
        SsbActionType::FreePoolEvent => (),
        // J-73 Slice C: RetrievalAbstained flat variant, Pattern-2 single-variant.
        SsbActionType::RetrievalAbstained => (),
        // J-78: ContextAccessSpec flat variant, Pattern-2 single-variant.
        SsbActionType::ContextAccessSpec => (),
        SsbActionType::DagCreditAssigned => (),
        // I-30 Slice 1a: SftExhaustEvent flat variant, Pattern-2 single-variant.
        SsbActionType::SftExhaustEvent => (),
        // J-23: ReviewGateCatalogEvent flat variant, Pattern-2 single-variant.
        SsbActionType::ReviewGateCatalogEvent => (),
        SsbActionType::ClaimRiskScored => (),
        // J-42 T2: ReasoningCheckpointEvent flat variant, Kind rides in the payload.
        SsbActionType::ReasoningCheckpointEvent(_) => (),
        SsbActionType::OperatingProfileChange => (),
        SsbActionType::AdvisorConsulted => (),
        SsbActionType::GoalEnrolled => (),
        SsbActionType::GoalVerified => (),
        SsbActionType::GoalViolated => (),
    }
}

fn golden_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("action_type_lock.golden")
}

/// Build the sorted (alphabetical by wire tag) list of bare `<tag>` lines that
/// the golden file body should contain -- header comment lines excluded. Tier-B
/// Move 2: the 4-digit ordinal prefix was dropped so two parallel plans adding
/// different variants insert non-adjacent lines git auto-merges, instead of
/// renumbering every line from the insertion point forward (the old conflict).
/// The sorted wire-tag SET is still byte-compared, so the lock is not weakened.
fn runtime_body_lines() -> Vec<String> {
    let mut tags: Vec<&'static str> =
        ALL_VARIANTS.iter().map(|v| v.as_sql_str()).collect();
    tags.sort();
    tags.iter().map(|t| (*t).to_string()).collect()
}

/// Split the golden file into (header_comments, body_lines). Comment lines
/// start with `#`; body lines are "<ordinal> <wire_tag>".
fn split_golden(text: &str) -> (Vec<String>, Vec<String>) {
    let mut header = Vec::new();
    let mut body = Vec::new();
    for raw in text.lines() {
        if raw.starts_with('#') || raw.is_empty() {
            header.push(raw.to_string());
        } else {
            body.push(raw.to_string());
        }
    }
    (header, body)
}

#[test]
fn action_type_lock_count_is_locked() {
    assert_eq!(
        ALL_VARIANTS.len(),
        TOTAL_VARIANTS,
        "TOTAL_VARIANTS const ({}) and ALL_VARIANTS array length ({}) drift. \
         A variant was added/removed without updating both. Update TOTAL_VARIANTS \
         and regen action_type_lock.golden via \
         `cargo test -p elai-orchestrator --test action_type_lock regen_golden_file -- --ignored --nocapture`.",
        TOTAL_VARIANTS,
        ALL_VARIANTS.len(),
    );

    // Defensive: also assert no duplicate wire tags. as_sql_str() must be
    // a 1:1 function on variants. A collision here would be a rename gone wrong.
    let mut seen: BTreeSet<&'static str> = BTreeSet::new();
    for v in ALL_VARIANTS.iter() {
        let tag = v.as_sql_str();
        assert!(
            seen.insert(tag),
            "duplicate wire tag '{}' produced by SsbActionType::as_sql_str -- \
             two variants map to the same SQL string (rename collision?)",
            tag
        );
    }
}

#[test]
fn action_type_lock_golden_matches_runtime() {
    let path = golden_path();
    let golden_text = match fs::read_to_string(&path) {
        Ok(t) => t,
        Err(e) => panic!(
            "cannot read golden file {}: {}. Regenerate via \
             `cargo test -p elai-orchestrator --test action_type_lock regen_golden_file -- --ignored --nocapture`.",
            path.display(),
            e
        ),
    };
    let (_header, golden_body) = split_golden(&golden_text);
    let runtime_body = runtime_body_lines();

    if golden_body == runtime_body {
        return;
    }

    // Build a structured diff: lines only in runtime (additions), lines
    // only in golden (removals/renames), and ordinal-shifted lines.
    let golden_set: BTreeSet<&str> = golden_body.iter().map(|s| s.as_str()).collect();
    let runtime_set: BTreeSet<&str> = runtime_body.iter().map(|s| s.as_str()).collect();
    let only_runtime: Vec<&&str> = runtime_set.difference(&golden_set).collect();
    let only_golden: Vec<&&str> = golden_set.difference(&runtime_set).collect();

    let mut msg = String::new();
    msg.push_str("SsbActionType golden-file drift detected.\n\n");
    if !only_runtime.is_empty() {
        msg.push_str(
            "Lines present in runtime but missing from golden \
             (additions OR renames -- confirm via PR review):\n",
        );
        for line in &only_runtime {
            msg.push_str("  + ");
            msg.push_str(line);
            msg.push('\n');
        }
    }
    if !only_golden.is_empty() {
        msg.push_str(
            "Lines present in golden but missing from runtime \
             (REMOVALS or RENAMES -- append-only contract violated unless \
             you also added #[serde(rename = ..)] shim):\n",
        );
        for line in &only_golden {
            msg.push_str("  - ");
            msg.push_str(line);
            msg.push('\n');
        }
    }
    if only_runtime.is_empty() && only_golden.is_empty() {
        msg.push_str(
            "Set-equal but ordering / ordinals differ -- alphabetical sort \
             invariant broken. This should be impossible unless the test \
             itself was edited.\n",
        );
    }
    msg.push_str("\nTo accept this change (after confirming it is NOT a rename or removal):\n");
    msg.push_str("  cargo test -p elai-orchestrator --test action_type_lock regen_golden_file -- --ignored --nocapture\n");
    msg.push_str(
        "Then review the diff in `git diff -- crates/orchestrator/tests/action_type_lock.golden` before committing.\n",
    );
    panic!("{}", msg);
}

/// Regenerate the golden file in place. `#[ignore]` so it never runs in CI.
/// Invoke manually after a deliberate SsbActionType addition:
///
///   cargo test -p elai-orchestrator --test action_type_lock \
///       regen_golden_file -- --ignored --nocapture
///
/// Review the resulting `git diff` BEFORE committing -- the diff is the audit
/// trail for the schema change.
#[test]
#[ignore]
fn regen_golden_file() {
    let path = golden_path();
    let mut out = String::new();
    out.push_str("# action_type_lock.golden -- SsbActionType wire-tag lock file\n");
    out.push_str("# Generated by: cargo test -p elai-orchestrator --test action_type_lock regen_golden_file -- --ignored --nocapture\n");
    out.push_str("# Format: one <wire_tag> per line, ALPHABETICALLY sorted by as_sql_str().\n");
    out.push_str(
        "# Tier-B Move 2: no ordinal prefix and no total-count line, so two parallel plans\n",
    );
    out.push_str(
        "# adding different variants insert non-adjacent lines git auto-merges (no renumber\n",
    );
    out.push_str(
        "# conflict). The sorted wire-tag SET is still byte-compared, so the lock holds.\n",
    );
    out.push_str(
        "# Append-only contract: removing or renaming a wire tag requires adding a #[serde(rename = \"<old>\")] shim AND updating this file in the same commit.\n",
    );
    out.push_str(
        "# 8th lockstep site for SsbActionType (alongside enum / SQL CHECK / as_sql_str / from_str / SsbPayload / action_type() / variant tests).\n",
    );
    out.push_str("#\n");
    for line in runtime_body_lines() {
        out.push_str(&line);
        out.push('\n');
    }
    match fs::write(&path, out) {
        Ok(_) => {}
        Err(e) => panic!(
            "regen_golden_file: failed to write {}: {}",
            path.display(),
            e
        ),
    }
    eprintln!(
        "regen_golden_file: wrote {} runtime variant lines to {}",
        ALL_VARIANTS.len(),
        path.display()
    );
}
