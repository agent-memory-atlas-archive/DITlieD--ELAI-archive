# Verification Report: update-implementation-order.txt
Source: F:\.ELAI_workflow\docs\specs\guide\update-implementation-order.txt
Date: 2026-05-17
Overall status: MOSTLY_DONE
Confidence: HIGH

## Summary
The update-implementation-order.txt defines 48 A-steps, 3 B-steps, and 8 Design-Addition blocks
spanning Groups A and B. A-1 and A-3 are marked COMPLETE in the spec itself and confirmed by
artifacts. The vast majority of Group A foundations (A-4 through A-47) are verifiably landed in
code. A handful of steps are PARTIAL (missing specific sub-features), and B-1/B-2 (constrained
decoding) plus the 4B/4E Design Additions are MISSING from the codebase. Phase 4 itself has not
started per the spec's own header; steps A-26 through A-31 (prompt-to-architecture) and the Group
B steps are the primary remaining gap.

## Mechanisms verified DONE

Group A: Foundations

- A-1 BASELINE MEASUREMENT HARNESS — benches/model_capability/run_benchmark.py, metrics/model_baseline.toml, benches/model_capability/bench_config.toml. detect_model_capability_profile at crates/orchestrator/src/context_budget.rs:554.
- A-3 MODEL.GENERATE MIGRATION — ETCLogitsProcessor in python/elai_inference/server.py:924, calibrate_etc_threshold.py:1.
- A-4 PI TIER 1 (ProjectProfile, BuildCommandRegistry, FrameworkDetection) — crates/core/src/project_profile.rs:489 (BuildCommandRegistry), FrameworkDetection at crates/core/src/project_profile.rs:202.
- A-5 PI TIER 2 (extract_conventions, ProjectRuleSeeder) — crates/knowledge/src/dkg/mod.rs:1059 (extract_conventions), crates/rules/src/project_rule_seeder.rs (AUTO_PROMOTE_SUCCESSES, AUTO_RETIRE_FP_COUNT).
- A-6 VERSION_GATE + deprecated_patterns — crates/rules/src/schema.rs:747 (deprecated_patterns_seed_data_loaded), T1Hook with rules loaded at crates/plugins/src/tiered_analysis.rs:335.
- A-7 CONSTITUTIONAL INJECTION — CONSTRAINTS block injection at crates/orchestrator/src/context_budget.rs:1420; contract crates/contracts/tests/fu_constraints_injection_contract.rs:1.
- A-8 FRESHNESS STEERING VECTOR — metrics/compute_steering_vector.py (ALPHA_DEFAULT=0.3), batch mode at metrics/test_batch_steering.py.
- A-9 ETC ENTROPY MONITORING — ETCLogitsProcessor at python/elai_inference/server.py:924-949, calibrate_etc_threshold.py, Rust sidecar_client ETC threshold at crates/providers/src/sidecar_client.rs:640.
- A-10 CONTEXT BUDGET ADAPTATION — ModelCapabilityProfile, SlotWeights, attention_ceiling at crates/orchestrator/src/context_budget.rs:268; detect_model_capability_profile at :554.
- A-11 PI TIER 3 + TELESCOPE — assemble_telescope at crates/knowledge/src/telescope.rs:147; TelescopeContext at :42; tier3 domain classification at crates/knowledge/src/tier3.rs.
- A-12 DETERMINISTIC-FIRST AUDITOR — DeterministicAuditorConfig at crates/orchestrator/src/deterministic_auditor.rs:100; check_build:539, check_file_size_census:714.
- A-13 RAG FRESHNESS SCORING — FreshnessConfig, access_boost, recency_decay at crates/knowledge/src/freshness.rs:113-480.
- A-14 T2 AST IMPORT EXTRACTION — check_deprecated_ast_imports at crates/plugins/src/tiered_analysis.rs:854; ast_adapter at crates/knowledge/src/ast_adapter.rs:1.
- A-15 NIGHTLY DEP AUDIT — DepAuditEntry:49, DepAuditResult:59 at crates/cli/src/dep_audit.rs; 7 ecosystems in Ecosystem enum.
- A-16 PI LATENCY MEASUREMENT — crates/knowledge/examples/measure_pi_latency.rs:1; pi_latency.toml in metrics/.
- A-17 GG SIDECAR RESPONSE EXTENSION — mean_logprob, min_logprob, entropy_mean, entropy_max in GGSignals at crates/providers/src/sidecar_client.rs:162; ProviderSignalsPayload at crates/orchestrator/src/provider_signals.rs:63-64.
- A-18 REPAIR CONTEXT PROFILE — GenerationProfile (REPAIR_RETRY, TREE_BRANCH_A/B/C etc.) at crates/orchestrator/src/context_budget.rs:72-74; compute_repair_context_slot_allocations at :2561.
- A-19 INTERROGATION TREE STORAGE — InterrogationTree, QuestionNode in crates/core/src/interrogation_tree.rs:509; seed trees (build_rust_lifetime_tree, build_python_import_tree, build_go_error_tree) at crates/rules/src/interrogation_store.rs.
- A-20 MISTAKE BOOK SSB SCHEMA — MistakeEntry at crates/core/src/mistake_book.rs:318; MistakeEntryPayload at crates/orchestrator/src/ssb.rs:739; forbidden_diffs via blake3 diff hash.
- A-21 CC HOOK ENFORCEMENT — T2Hook coupling score, magic number detection, file size limits at crates/plugins/src/tiered_analysis.rs; GREP_PATTERN extension in rules.
- A-22 RESEARCHING FSM STATE — FsmState::Researching at crates/orchestrator/src/fsm.rs:37; contract at crates/contracts/tests/a22_researching_fsm_contract.rs:1.
- A-23 MICRO-REFLECTION TYPES — MicroReflectionRequest/Response at crates/core/src/micro_reflection.rs:59,127; contract crates/contracts/tests/a23_micro_reflection_contract.rs.
- A-24 SHORTCODER CONTEXT COMPRESSION — crates/knowledge/src/shortcoder.rs (collapse_rust/python_function_body); ShortCoderHook at crates/orchestrator/src/shortcoder_hook.rs.
- A-25 GG GENERATION NOVELTY SIGNAL — generation_novelty in completion response at crates/providers/src/sidecar_client.rs:674 (test shows "generation_novelty":0.73).
- A-26 TOOL ROUTING LAYER — routing_rules module at crates/tools/src/routing_rules.rs; RoutingConfig, builtin_rules (BUILTIN_RULE_COUNT=5 covering cat/head/tail/sed/awk/echo/find/grep).
- A-27 BASH SAFETY FILTER — safety_filter module at crates/tools/src/safety_filter.rs; blocks force-push, hard-reset, --no-verify, branch -D, checkout --.
- A-28 GIT COMMIT WORKFLOW TOOL — git_commit module at crates/tools/src/git_commit.rs:415 (format_message_strips_co_authored_by, format_message_strips_generated_with).
- A-29 AGENT PROMPT TEMPLATE — agent_template module at crates/orchestrator/src/agent_template.rs:1; AgentPromptContext, from_cwd, defaults_for.
- A-30 OUTPUT POSTPROCESSOR — output_postprocessor at crates/orchestrator/src/output_postprocessor.rs; emoji stripper, sycophancy_regexes, length enforcer.
- A-31 ACTION CLASSIFIER + CONFIRMATION GATE — action_classifier module at crates/tools/src/action_classifier.rs:34; ActionClassification, ActionPolicy, GateDecision, ExecutionMode.
- A-32 PI TIER 1 COMPLETIONS — load_overrides/merge_overrides at crates/knowledge/src/project_profile.rs:1072,1102; override_valid_file_merges_fields at :2053; FrameworkDetection expanded.
- A-33 PI TIER 2 COMPLETIONS — DependencyProfile.classification_depth at crates/core/src/project_profile.rs:403; contract crates/contracts/tests/pi_tier2_a33_contract.rs; extract_from_dkg/regex_fallback at crates/knowledge/src/conventions.rs.
- A-34 RA SELF-DISCOVERY FAILURE TAXONOMY — FailureCategoryProposal at crates/rules/src/self_discovery.rs:97 (pattern, fix_template, confidence, source_violations); failure_category_proposals table at crates/rules/src/schema.rs:566.
- A-35 PI MULTI-LANGUAGE DKG GRAMMARS — PythonGrammar, GoGrammar, TypeScriptGrammar impls at crates/knowledge/src/dkg/mod.rs:1070,1193,1542; contract crates/contracts/tests/dkg_multilang_contract.rs.
- A-36 PI TIER 3 + TELESCOPE COMPLETIONS — fix_templates module at crates/knowledge/src/fix_templates.rs:1 (attach_fix_template_keys); DKG few-shot at SSB TelescopeCollapsePayload; contracts at crates/contracts/tests/pi_tier3_completions_contract.rs and pi_telescope_contract.rs.
- A-37 RA VRAM-TIER LAYER CONFIGURATION — VramTierConfig, VramTier, CloudImportance at crates/core/src/vram_tier.rs:14-73; ab_mcts_max_branches:72; classify_vram_bytes at crates/providers/src/vram_tier.rs:28.
- A-38 PI INCREMENTAL PROFILE UPDATES — incremental_profile module at crates/knowledge/src/incremental_profile.rs:17 (BABY_STEP_REFRESH_N, ProfileChangeDetector, load_or_detect_profile:285, schema_version).
- A-39 AS DURING-GENERATION SAFETY DEFAULTS — steering disabled-by-default verified at audit-scratch/test_a39_safety_defaults_invariants.py:60; ETC threshold=infinity at crates/providers/src/sidecar_client.rs:640; calibrate_etc_threshold.py verifies Qwen target.
- A-40 MICRO PER-SLOT OPTIMIZATIONS — MICRO_SLOT_RULES constant at crates/orchestrator/src/context_budget.rs:19; micro_slot_repo_map_skipped, micro_slot_rules_target_150, micro_slot_code_context_target_1200 contracts at crates/contracts/tests/context_budget_contract.rs.
- A-41 AS DETERMINISTIC AUDITOR COMPLETIONS — DeterministicAuditorConfig.build_cmds at :104; check_completeness:779, check_file_size_census:714; deterministic_first config at crates/cli/src/runtime_config.rs:158.
- A-42 TOKEN RECLAMATION TECHNIQUES — apply_token_reclamation at crates/orchestrator/src/context_budget.rs:1723; path abbreviation tests at :2965-3075; contracts at crates/contracts/tests/context_budget_contract.rs:466-484.
- A-43 ORACLE RESEARCH PIPELINE — research_pipeline module at crates/orchestrator/src/research_pipeline.rs (phases A-G); ResearchQuery, VerifiedClaim, AnswerConfidence; ResearchAnswer at crates/core/src/research_types.rs:192.
- A-44 PROJECT INDEXING WIRING — IndexingService at crates/orchestrator/src/indexing_service.rs:358 (build_index_resumable, indexes_exist, cancel_indexing); contract crates/contracts/tests/indexing_service_a44_contract.rs.
- A-45 SOURCE TIER CONFIG — SourceTierConfig at crates/core/src/source_tier.rs:115 (load_source_tier_config); SourceQualityTable at crates/web_search/src/source_quality.rs:67; tier_to_score, classify_source.
- A-46 CONTEXT TELESCOPE TYPES — BoundedString<N> at crates/core/src/bounded_string.rs:69; ContextTelescope at crates/core/src/telescope.rs; SurgicalTelescope.blast_radius at :93; render() at :99.
- A-47 /RESEARCH CLI COMMAND + FALLBACK CLASSIFIER — classifier module at crates/cli/src/classifier.rs (classify_bare_message:83; ACTION_VERBS; question_without_code_intent_routes_to_research:128, question_with_code_intent_routes_to_task:140).
- A-48 DKG DATA FLOW EDGES — FlowType enum at crates/knowledge/src/lib.rs:158 (Assignment, Parameter, Return, FieldAccess); dkg_data_flow table; contract crates/contracts/tests/dkg_data_flow_a48_contract.rs.
- DESIGN ADDITIONS TO 4G (partial): CloudImportance at crates/core/src/vram_tier.rs:30; AB-MCTS branch profiles (TreeBranchA/B/C GenerationProfile) at crates/orchestrator/src/context_budget.rs:72-74; AutonomyDecision::AutoAdvance at crates/orchestrator/src/autonomous_gate.rs:298.
- DESIGN ADDITIONS TO 4H: nightly dep audit at crates/cli/src/dep_audit.rs wired; steering vector refresh infrastructure at metrics/compute_steering_vector.py.

## Mechanisms PARTIAL

- A-2 T1/T2 HOOK FIX — T1Hook.loaded_rules field exists (crates/plugins/src/tiered_analysis.rs:335), T2Hook.deprecated_imports field exists (:598). Evidence of partial wiring but spec's BLOCK-3 "T1 loads nothing / T2 returns Allow unconditionally" status not fully verified as resolved from source code alone; only structural fields confirmed present, not runtime evaluation.
  - exists: crates/plugins/src/tiered_analysis.rs:333-335 (T1Hook.fsm_phase, T1Hook.loaded_rules)
  - gap: Cannot confirm from map alone that T1 actually evaluates VERSION_GATE at runtime vs holding the field unused; no invariant test confirming T2 returns non-Allow.

- B-3 MAINTENANCE AUTOMATION — steering vector refresh infrastructure exists (metrics/compute_steering_vector.py, batch mode test_batch_steering.py). Freshness decay infrastructure at crates/knowledge/src/freshness.rs. No evidence of a scheduled automation trigger wiring the recalculation + steering refresh as a nightly job.
  - exists: metrics/compute_steering_vector.py (compute), crates/knowledge/src/freshness.rs (decay logic)
  - gap: No DEF found for a maintenance scheduler, nightly trigger, or automated recalculation loop connecting the two.

- DESIGN ADDITIONS TO 4D — SurgicalDecomposer blast radius analysis exists as SurgicalTelescope.blast_radius (crates/core/src/telescope.rs:93) and fallback classifier routes to research (crates/cli/src/classifier.rs:128). Oracle pipeline exists (A-43). Full 4D integration (FILE_PLAN as 9th BabyStepSpec block, ModuleTree type, tier-gated file count, COMPLEXITY_HIGH marking) not confirmed in codebase.
  - exists: crates/core/src/telescope.rs:93 (blast_radius field), crates/orchestrator/src/research_pipeline.rs (Oracle pipeline), crates/cli/src/classifier.rs (fallback classifier)
  - gap: No DEF found for FILE_PLAN as BabyStepSpec block variant, ModuleTree type, or COMPLEXITY_HIGH marker.

- DESIGN ADDITIONS TO 4E — BubbleUpAction::DiagnoseAndRetry exists (crates/orchestrator/src/pipeline_tree_navigator.rs:93). No evidence of FileLineTracker, skeleton-first generation, fill protocol, MiniCritic, BridgeFileTemplate, or deterministic file splitter.
  - exists: crates/orchestrator/src/pipeline_tree_navigator.rs:93 (DiagnoseAndRetry), BubbleUpPolicy, escalation levels
  - gap: FileLineTracker, skeleton_first, mini_critic, bridge_file_templates, file_splitter — no DEF/SCIP_DEF found.

- DESIGN ADDITIONS TO 4G (completeness) — CloudImportance and AB-MCTS branch profiles exist. AutoAdvance decision (AutonomyDecision::AutoAdvance) exists but the full 4G additions: PredictiveLoader, CloudSpendTracker, explicit blast-radius thresholds (autonomous_confirm_above_loc, autonomous_refuse_above_loc), model swap protocol with 6 strategies — not confirmed.
  - exists: crates/core/src/vram_tier.rs (CloudImportance, VramTierConfig), crates/orchestrator/src/context_budget.rs (GenerationProfile tree branches), crates/orchestrator/src/autonomous_gate.rs (AutoAdvance)
  - gap: No DEF for PredictiveLoader, CloudSpendTracker, autonomous_confirm_above_loc, autonomous_refuse_above_loc, or model swap protocol enum.

## Mechanisms MISSING

- B-1 CONSTRAINED DECODING — searched: ConstrainedDecodingConfig, DecodingMode, grammar_guided, CodeFast, CRANE; zero DEF/SCIP_DEF results. vLLM provider extensions not present.
- B-2 CONSTRAINED DECODING FOR IMPORTS — searched: import allowlist, EBNF grammar, positive allowlist, import_context_detection; zero DEF results. Spec states vLLM-only, feature-gated; no trace in codebase.
- DESIGN ADDITIONS TO 4B (RA) — searched: LAZY_CODE failure category, EMPTY_BODY category, FILE_SIZE_EXCEEDED category (as RA taxonomy additions), SVRepair CFG analysis, DkgQueryTool IPC tools (dkg_callers/dkg_dependents exist in tools/tests/tool_dispatch.rs as test helpers but not as an exported IPC interface per 4B spec intent), DKG integration wiring check tool. DiagnosticResult exists in crates/tools/src/failure_taxonomy.rs but the 3 new RA categories (LAZY_CODE, EMPTY_BODY, FILE_SIZE_EXCEEDED) and SVRepair are not found.

## Mechanisms UNVERIFIABLE

- A-3 STATUS CLAIM (server.py:391-436 manual loop replaced) — The spec asserts the manual loop at server.py lines 391-436 was replaced. ETCLogitsProcessor and model.generate() are confirmed present in server.py. The exact line range of the old manual loop cannot be independently verified from elai.map without reading the file; the presence of ETCLogitsProcessor+model.generate() is strong indirect evidence of completion.
- A-2 RUNTIME CORRECTNESS — T1/T2 hook structural fields confirmed; whether the runtime behavior (T1 actually evaluates rules, T2 actually rejects deprecated imports) is fully operational cannot be determined from symbol presence alone without running tests.
- DESIGN ADDITIONS TO 4H (nightly dep audit trigger) — dep_audit CLI command exists; automated nightly trigger from self-mod loop is not findable via map scan (could be wired inside self_mod_layer1a or task_runner without a unique symbol name).

## What must be done (action list)

- [ ] Implement B-1 CONSTRAINED DECODING: ConstrainedDecodingConfig, DecodingMode, grammar-guided generation in crates/providers/src/vllm_client.rs; feature-gate cfg(feature="vllm")
- [ ] Implement B-2 CONSTRAINED DECODING FOR IMPORTS: positive allowlist via JSON Schema or EBNF for import lines in vllm_client.rs; depends on B-1
- [ ] Add DESIGN ADDITIONS TO 4B (RA): FailureTaxonomy 3 new categories (LAZY_CODE, EMPTY_BODY, FILE_SIZE_EXCEEDED) in crates/tools/src/failure_taxonomy.rs; SVRepair CFG analysis; DKG query IPC interface (dkg_callers/dkg_dependents/dkg_data_flow/dkg_subgraph) in crates/tools/src/
- [ ] Add DESIGN ADDITIONS TO 4E (CC+RA): FileLineTracker (TOCTOU prevention), skeleton-first generation, fill protocol, MiniCritic, BridgeFileTemplate, deterministic file splitter — all in orchestrator/ and/or a new codegen/ module
- [ ] Add DESIGN ADDITIONS TO 4G (completions): PredictiveLoader type, CloudSpendTracker budget tiers, autonomous_confirm_above_loc + autonomous_refuse_above_loc thresholds in EscalationPolicy; model swap protocol enum (6 strategies) in orchestrator/ or providers/
- [ ] Verify A-2 T1/T2 runtime correctness: run cargo test -p elai-plugins targeting T1/T2 hook evaluation with real rules.db; confirm T2 returns non-Allow on deprecated import match
- [ ] Implement B-3 MAINTENANCE AUTOMATION: scheduled nightly trigger wiring recalculate_freshness_decay() + recompute_steering_vector() — either in a daemon task in orchestrator/ or a CLI cron hook
- [ ] Add DESIGN ADDITIONS TO 4D (missing pieces): FILE_PLAN as 9th BabyStepSpec block variant in core/; ModuleTree language-parameterized type; COMPLEXITY_HIGH BabyStep marker; tier-gated file count limits in decomposer

## Notes

Steps A-1 and A-3 are explicitly marked COMPLETE in the spec header and confirmed by file presence.
Phase 4 itself (phases 4A-4J) has not started per the spec; the "DESIGN ADDITIONS TO 4X" blocks
are preparatory integrations to be woven in when each Phase 4 phase is implemented. Missing items
in those blocks (4B, 4E, 4G) are expected since Phase 4 is not yet implemented — they are recorded
here as future obligations. Group A foundations are overwhelmingly complete. The elai.map index as
of this scan is considered current (no map-staleness indicator found).
