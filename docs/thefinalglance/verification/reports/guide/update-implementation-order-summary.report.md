# Verification Report: update-implementation-order-summary
Source: F:\.ELAI_workflow\docs\specs\guide\update-implementation-order-summary.txt
Date: 2026-05-17
Overall status: MOSTLY_DONE
Confidence: HIGH

## Summary
The summary document describes 48 Group A steps and 3 Group B steps added to phases 0-3. The
majority of mechanisms are implemented and verified in the codebase via elai.map. Core
infrastructure items (PI tiers 1-3, MistakeBook, RESEARCHING FSM, ETCLogitsProcessor, GG signals,
ShortCoder, ContextTelescope, BoundedString, GenerationProfile, action classifier, bash safety
filter, agent template, output postprocessor, token reclamation, source tiers, research pipeline,
dep audit, VRAM tiers, incremental profile, DKG data flow, self-discovery) are all present. A-2
(T1/T2 hook fix) is partially implemented. A-26 (Tool Routing Layer) shows no evidence. A-16 has
a standalone example binary rather than a formal benchmark harness. B-series mechanisms are
described as extending Phase 3J but have no explicit B-1/B-2/B-3 module evidence found.

## Mechanisms verified DONE

- A-1 Baseline Measurement Harness — evidence: crates/orchestrator/src/context_budget.rs (ModelCapabilityProfile::micro_7b_12gb/3b, FA2/DoLa data referenced); dashboard command get_baseline_results: crates/dashboard-v2/src-tauri/src/commands.rs:2634
- A-3 model.generate() Migration — evidence: ETCLogitsProcessor uses LogitsProcessor: python/elai_inference/server.py:956
- A-4 Project Intelligence Tier 1 — evidence: crates/knowledge/src/project_profile.rs (detect_languages, detect_build_system, detect_frameworks, detect_language); ProjectProfile struct confirmed
- A-5 Project Intelligence Tier 2 / Convention extraction — evidence: crates/knowledge/src/conventions.rs (extract_module_organization, NamingConventions); crates/knowledge/src/dkg/mod.rs:1059 (extract_conventions)
- A-6 VERSION_GATE predicate — evidence: crates/rules/src/predicate.rs:855 (VersionGate roundtrip tests, ecosystem/deprecated_name/replacement_name fields); crates/rules/src/self_discovery.rs (version gate proposal)
- A-7 Constitutional Injection + T1 VERSION_GATE Hook — evidence: crates/orchestrator/src/context_budget.rs:1420 (CONSTRAINTS block, anti-staleness injection comment); crates/orchestrator/src/ssb.rs:585 (CheckpointPayload docstring references A-7 constitutional CONSTRAINTS block)
- A-8 Freshness Steering Vector — evidence: crates/providers/src/layer_config/steering.rs (SteeringVector, SteeringVectorSpec); crates/contracts/tests/fulc_steering_vector_contract.rs (full suite); dashboard recompute_steering_vector call
- A-9 ETC Entropy Monitoring — evidence: python/elai_inference/server.py:956 (ETCLogitsProcessor class, __init__, __call__, d2h_log, dh_prev)
- A-10 Context Budget Adaptation — evidence: crates/orchestrator/src/context_budget.rs (ModelCapabilityProfile, slot_weights, ContextBudget::compute_with_profile, VramTier); crates/orchestrator/tests/context_budget_profile.rs
- A-11 PI Tier 3 + Telescope (project_vision) — evidence: crates/core/src/telescope.rs:101 (project_vision field); crates/contracts/tests/dashboard_pi_tier3_on_open_contract.rs (pi_tier3 tests)
- A-12 Deterministic-First Auditor — evidence: crates/orchestrator/src/deterministic_auditor.rs (detect_language, 5-step checks); crates/knowledge/src/project_profile.rs build commands
- A-13 RAG Freshness Scoring — evidence: crates/knowledge/src/freshness.rs (FreshnessDb, freshness_meta.db); crates/dashboard-v2/src/lib/state/drawers.svelte.ts:105 (freshness_decay_pct state)
- A-14 T2 AST Import Extraction — evidence: crates/plugins/src/tiered_analysis.rs:598 (deprecated_imports field on T2Hook); AST-level import checking wired
- A-15 Nightly Dependency Audit — evidence: crates/cli/src/dep_audit.rs (dep_audit function, 7-ecosystem detection, registry queries, deprecated_patterns upsert, TOML report)
- A-17 GG Sidecar Response Extension — evidence: python/elai_inference/server.py (mean_logprob, min_logprob, entropy_mean, entropy_max computed); contracts/tests/provider_signals_contract.rs (GgSignals roundtrip)
- A-18 Repair Context Profile — evidence: crates/orchestrator/src/context_budget.rs (REPAIR_CONTEXT_TOTAL, compute_repair_context, GenerationProfile with 7 variants); crates/contracts/tests/a18_generation_profile_contract.rs
- A-19 Interrogation Tree Storage — evidence: crates/core/src/interrogation_tree.rs (InterrogationTree, QuestionNode, Choice types; seed trees for Rust/Python/Go/TypeScript referenced); 3-7 questions per tree validated
- A-20 Mistake Book SSB Schema — evidence: crates/core/src/mistake_book.rs (MistakeEntry, MAX_ENTRIES_PER_STEP=5, blake3 diff hashing); crates/orchestrator/src/mistake_book.rs (SSB write, query_by_baby_step)
- A-21 CC Hook Enforcement — evidence: crates/plugins/src/tiered_analysis.rs (lazy_code_regex, detect_file_size, cyclomatic_complexity_rust/generic, check_coupling_score_generic, 11-language file size limits); ssb lazy_code_regex table
- A-22 RESEARCHING FSM State — evidence: crates/orchestrator/src/fsm.rs:1431 (FsmState::Researching transitions); crates/orchestrator/src/tool_filter.rs:562 (WebSearch/WebFetch allowed, edit_file/bash blocked)
- A-23 Micro-Reflection Types + Protocol — evidence: crates/core/src/micro_reflection.rs (MicroReflectionResponse, parse_reflection_json, truncate_to_budget, 500-token budget); crates/orchestrator/src/micro_reflection_emitter.rs
- A-24 ShortCoder Context Compression — evidence: crates/knowledge/src/shortcoder.rs (10 tree-sitter rules); crates/orchestrator/src/shortcoder_hook.rs (ShortCoderHook, PostToolUse wiring)
- A-25 GG Generation Novelty Signal — evidence: crates/orchestrator/src/provider_signals.rs:85 (NoveltySignal with cosine_distance); python/elai_inference/server.py:1083 (generation_novelty computed)
- A-27 Bash Safety Filter — evidence: crates/tools/src/safety_filter.rs:302 (check_bash_safety, blocklist: push --force, branch -D); crates/tools/src/lib.rs:2071 (wired into tool execution)
- A-28 Git Commit Workflow Tool — evidence: crates/tools/src/git_commit.rs:443 (A-28 comment, git_commit tool implementation)
- A-29 Agent Prompt Template — evidence: crates/orchestrator/src/agent_template.rs:1 (module docstring "A-29"); AgentTemplateConfig struct with context/task/constraints/output sections
- A-30 Output Postprocessor — evidence: crates/orchestrator/src/output_postprocessor.rs (process, emoji strip, sycophancy prefix strip, length enforcement, restore_output_formatting)
- A-31 Action Classifier + Confirmation Gate — evidence: crates/tools/src/action_classifier.rs:1 ("A-31" docstring); crates/contracts/tests/a31_action_classifier_contract.rs (SAFE/REVERSIBLE/IRREVERSIBLE/SHARED_STATE tags, confirmation gate)
- A-32 PI Tier 1 Completions — evidence: crates/knowledge/src/project_profile.rs (override file, build system priority, framework entries, test runner patterns, command validation)
- A-33 PI Tier 2 Completions — evidence: crates/knowledge/src/conventions.rs (path prefix stripping, NamingConventions with STRONG/MODERATE/MIXED thresholds); rule lifecycle via SSB
- A-34 RA Self-Discovery — Failure Taxonomy Extension — evidence: crates/rules/src/self_discovery.rs (detect_recurring_category_gap, FailureCategoryProposal, auto-propose after 3 recurring patterns); crates/contracts/tests/self_mod_bridge_contract.rs
- A-35 PI Multi-Language DKG Grammars — evidence: crates/knowledge/src/dkg/mod.rs (RustGrammar, PythonGrammar, GoGrammar, TypeScriptGrammar, TsxGrammar, SvelteGrammar registered); LanguageGrammar trait
- A-36 PI Tier 3 + Telescope Completions — evidence: crates/core/src/telescope.rs (project_vision, ContextTelescope 4-layer); crates/contracts/tests/dashboard_pi_tier3_on_open_contract.rs
- A-37 RA VRAM-Tier Layer Configuration — evidence: crates/core/src/vram_tier.rs (VramTier, ab_mcts_max_branches for 8/12/16GB, CloudImportance, SwapStrategy); crates/orchestrator/src/context_budget.rs (VramTier variants)
- A-38 PI Incremental Profile Updates — evidence: crates/knowledge/src/incremental_profile.rs (check_config_changes, incremental_tier1_refresh, check_language_shift, per-tier refresh)
- A-39 AS During-Generation Safety Defaults — evidence: python/elai_inference/server.py (ETCLogitsProcessor disabled by default per A-9 comments); DoLa/ETC incompatibility guard present in server
- A-40 MICRO Per-Slot Optimizations — evidence: crates/orchestrator/src/context_budget.rs (MICRO_SLOT_RULES, MICRO_SLOT_SYSTEM_PROMPT, MICRO_SLOT_CODE_CONTEXT, MICRO_SLOT_SSB_ENTRIES, MICRO_SLOT_OUTPUT_RESERVE); crates/contracts/tests/context_budget_contract.rs:339
- A-41 AS Deterministic Auditor Completions — evidence: crates/orchestrator/src/deterministic_auditor.rs (7-language detect_language, 5-step early-exit order, CC checks wired)
- A-42 Token Reclamation Techniques — evidence: crates/orchestrator/src/context_budget.rs (apply_token_reclamation, abbreviate_path, restore_output_formatting, import collapsing, deduplication); TokenReclamationStats struct: crates/orchestrator/src/ssb_viewer.rs:516
- A-43 Oracle Research Pipeline + Types — evidence: crates/orchestrator/src/research_pipeline.rs (run_research_pipeline, 7-phase: query formulation, search, source tier classify, fact extraction, cross-verify, synthesis, post-verify; VerifiedClaim, ResearchPipelineError types)
- A-44 Project Indexing Wiring — evidence: crates/knowledge/src/pipeline.rs (DkgEngine + VectorIndexer + BM25 wired in QueryPipeline); crates/knowledge/tests/error_propagation.rs (bm25_index, vector_index tests)
- A-45 Source Tier Config — evidence: crates/core/src/source_tier.rs (parse_source_tier_config, classify_source, SourceTier T1-T4); crates/orchestrator/src/task_runner.rs:1714 (tier_cfg loaded)
- A-46 Context Telescope Types — evidence: crates/core/src/bounded_string.rs (BoundedString<N> newtype); crates/orchestrator/src/telescope_caller.rs (ContextTelescope, 4 layers with BoundedString constants); crates/contracts/tests/a46_context_telescope_contract.rs
- A-47 /research CLI Command + Fallback Classifier — evidence: crates/cli/src/commands.rs:5312 (SlashCommand::parse("/research foo")); crates/dashboard-v2/src/lib/state/commands.svelte.ts:376 (/research dispatch)
- A-48 DKG Data Flow Edges — evidence: crates/contracts/tests/dkg_data_flow_a48_contract.rs (build_graph_extracts_rust_data_flow, data_flow_extraction_is_deterministic); dkg_data_flow table with PARAMETER/RETURN/ASSIGNMENT/FIELD_ACCESS
- A-16 PI Latency Measurement — evidence: crates/knowledge/examples/measure_pi_latency.rs (measure_tier2, to_measurement for tier1/tier2/tier3, percentile calc, multi-root support)

## Mechanisms PARTIAL

- A-2 T1/T2 Hook Fix — T1Hook.loaded_rules field and set_rules() exist, rules.db loading wired structurally
  - exists: crates/plugins/src/tiered_analysis.rs:335 (loaded_rules), :352 (with_rules), :367 (set_rules)
  - gap: No direct evidence that T1Hook.new() (zero-arg) previously returned Allow unconditionally — the fix claim ("currently loads nothing") cannot be confirmed as fixed vs original state. T2Hook has AST analysis (cyclomatic_complexity, deprecated_imports) confirmed, but the original "returns Allow unconditionally" claim is unverifiable from static analysis alone.

## Mechanisms MISSING

- A-26 Tool Routing Layer — searched: ToolRoutingLayer, tool_routing, ToolInterceptor, bash_interceptor, cat->Read hint; no evidence of a dedicated routing interceptor that maps bash cat/sed calls to dedicated tools with rejection hints

## Mechanisms UNVERIFIABLE

- B-1 Constrained Decoding (grammar-guided generation, JSON schema mode, CRANE, CodeFast) — reason: No module named constrained_decoding, grammar_guided, crane, codefast found in elai.map. These are described as "Prereqs: A-1, A-10" extensions but no landing evidence found. Cannot distinguish not-yet-implemented from renamed.
- B-2 Constrained Decoding for Imports (positive allowlist via JSON Schema/EBNF, vLLM-only) — reason: No vLLM-specific import-allowlist module found. Described as lowest-priority AS.
- B-3 Maintenance Automation (freshness decay recalculation, steering vector refresh automation) — reason: Manual steering recompute endpoint exists (dashboard recompute_steering_vector call) but no automated scheduler/cron for freshness decay recalc found. Cannot confirm automation vs manual trigger.
- Phase 4 design additions (4B-4J checklists) — reason: These are described as "integration checklists, not separate steps" and depend on Phase 4 execution which the summary states has NOT started.
- Completed phases modified (1A, 1E, 3C, 3E, 3F, 3J) — reason: Summary claims these phases were modified but does not specify which symbols changed; changelog/memory-bank references are marked UNVERIFIABLE per task constraints.

## What must be done (action list)

- [ ] Implement A-26 Tool Routing Layer: add a dedicated interceptor in crates/tools/src/ that detects bash calls matching cat/sed/echo patterns and rejects them with a hint pointing to Read/Edit/Write tools (target: ~150-200 LOC)
- [ ] Verify A-2 T1Hook rules.db loading: confirm that T1Hook::new() path actually loads rules from rules.db at construction time (not just via set_rules), or document that wiring is done externally by the caller
- [ ] Confirm B-1/B-2/B-3 scope: either implement constrained decoding modules or mark them explicitly as deferred with a phase-4 dependency note in the spec

## Notes
- A-3 is marked [DONE] in the summary and confirmed — model.generate() + LogitsProcessor pattern is in server.py.
- A-1 is marked [DONE] in the summary. The baseline data (FA2 critical, 7B 90%, DoLa harmful) is referenced in ModelCapabilityProfile constructors and the dashboard get_baseline_results command, confirming the harness ran and results landed.
- A-16 has a standalone binary (measure_pi_latency.rs example) rather than a test harness, which matches the summary's "~100 LOC" scope.
- The summary's parallelism sets are design guidance only; they are not verifiable as implementation artifacts.
