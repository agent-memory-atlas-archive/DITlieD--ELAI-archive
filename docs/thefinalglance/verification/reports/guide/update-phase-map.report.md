# Verification Report: update-phase-map.txt
Source: F:\.ELAI_workflow\docs\specs\guide\update-phase-map.txt
Date: 2026-05-17
Overall status: PARTIAL
Confidence: HIGH

## Summary
The update-phase-map.txt identifies 6 design areas requiring modifications to 12+ existing phase step files and creation of 4 new step files. Code-side implementations are largely complete (ETCLogitsProcessor, GGSignals, VERSION_GATE predicate, SlotWeights/REPAIR_CONTEXT, IndexingService, ProjectRuleSeeder, T1Hook/T2Hook, agent template, tool routing/safety/git_commit/action_classifier, steering vector batch mode, DiagnoseAndRetry, FailureTaxonomy partial). However, the phase step files themselves are almost entirely unmodified — none of the 12 files claimed to need updates (phase-1A, 1E, 3A, 3C, 3E, 3F, 3J, 4B, 4C, 4D, 4E, 4G, 4H) contain the new content described in the map. The 4 new step files (phase-3L, phase-3J-BASELINE, phase-4D-SURGICAL, phase-4D-ORACLE) do not exist.

## Mechanisms verified DONE

### Project Intelligence — code side
- extract_conventions / ConventionStats / NamingConventions types — evidence: .elai_cc\crates\knowledge\src\conventions.rs
- ProjectRuleSeeder — evidence: .elai_cc\crates\rules\src\project_rule_seeder.rs
- IndexingService (DkgEngine + BM25 wiring on first /task) — evidence: .elai_cc\crates\orchestrator\src\indexing_service.rs:1, :265
- ProjectProfile types — evidence: .elai_cc\crates\knowledge\src\project_profile.rs, .elai_cc\crates\core\src\project_profile.rs
- Context telescope integration (pi_telescope_contract, pi_tier3) — evidence: .elai_cc\crates\knowledge\src\telescope.rs, .elai_cc\crates\contracts\tests\pi_telescope_contract.rs

### Anti-Staleness — code side
- ETCLogitsProcessor in server.py — evidence: .elai_cc\python\elai_inference\server.py:956
- ETCLogitsProcessor instantiation at startup — evidence: .elai_cc\python\elai_inference\server.py:1488
- calibrate_etc_threshold.py — evidence: .elai_cc\metrics\calibrate_etc_threshold.py
- SearchTriggerPayload in sidecar_client.rs — evidence: .elai_cc\crates\providers\src\sidecar_client.rs
- VERSION_GATE predicate variant in predicate.rs — evidence: .elai_cc\crates\rules\src\predicate.rs:4,66
- deprecated_patterns table schema in rules.db — evidence: .elai_cc\crates\rules\src\schema.rs (grep hit)
- Constitutional CONSTRAINTS block injection (phase-3E context_budget.rs) — evidence: .elai_cc\crates\orchestrator\src\context_budget.rs:1420-1441
- deterministic_first = true in elai_cc.toml — evidence: .elai_cc\elai_cc.toml:45
- T1Hook loading rules from rules.db (not empty) — evidence: .elai_cc\crates\plugins\src\tiered_analysis.rs:337,346
- T2Hook with AST import extraction — evidence: .elai_cc\crates\plugins\src\tiered_analysis.rs:601,623
- modern_api_preference steering vector batch mode — evidence: .elai_cc\metrics\compute_steering_vector.py:7,201-204
- steering contrast pairs (data/steering_contrast_pairs.toml) — evidence: .elai_cc\data\steering_contrast_pairs.toml

### Token Efficiency — code side
- ModelCapabilityProfile type — evidence: .elai_cc\crates\orchestrator\src\context_budget.rs (grep hit, micro_7b_12gb, cloud_200k etc.)
- SlotWeights per model tier — evidence: .elai_cc\crates\orchestrator\src\context_budget.rs:203,162
- REPAIR_CONTEXT SlotWeights profile — evidence: .elai_cc\crates\orchestrator\src\context_budget.rs:126-168
- Benchmark harness binary (run_benchmark.py) — evidence: .elai_cc\benches\model_capability\run_benchmark.py
- ShortCoder PostToolUse integration — evidence: .elai_cc\crates\orchestrator\src\shortcoder_hook.rs, .elai_cc\crates\contracts\tests\shortcoder_post_tool_use_invocation_contract.rs
- ConstrainedDecodingConfig / grammar-guided / JSON schema constrained — evidence: .elai_cc\crates\providers\src\runtime\vllm.rs, types.rs

### Pipeline Modes — code side
- RESEARCHING FSM state — evidence: .elai_cc\crates\orchestrator\src\fsm.rs, .elai_cc\crates\contracts\tests\a22_researching_fsm_contract.rs
- SourceQualityTable / source tier classification — evidence: .elai_cc\crates\web_search\src\source_quality.rs, .elai_cc\crates\rules\seed\source_tiers.toml
- Research pipeline (web_research_orchestrator, research_pipeline) — evidence: .elai_cc\crates\orchestrator\src\web_research_orchestrator.rs, .elai_cc\crates\orchestrator\src\research_pipeline.rs

### Code Completeness — code side
- LAZY_CODE_SCAN in deterministic_auditor.rs — evidence: .elai_cc\crates\orchestrator\src\deterministic_auditor.rs:11,509
- file_discipline config (deterministic_first) in elai_cc.toml — evidence: .elai_cc\elai_cc.toml:43-45
- GREP_PATTERN language/file_context extension in tiered_analysis — evidence: .elai_cc\crates\plugins\src\tiered_analysis.rs

### Reasoning Architecture — code side
- REPAIR_CONTEXT SlotWeights + compute_repair_context() — evidence: .elai_cc\crates\orchestrator\src\context_budget.rs:1059-1090
- InterrogationTree data storage — evidence: .elai_cc\crates\core\src\interrogation_tree.rs, .elai_cc\crates\rules\src\interrogation_store.rs
- mean_logprob/entropy_mean GG signals in sidecar_client.rs — evidence: .elai_cc\crates\providers\src\sidecar_client.rs:149-160
- mean_logprob/entropy_mean in server.py — evidence: .elai_cc\python\elai_inference\server.py:1043-1086
- DiagnosticResult, RootCause, RootCauseKind types — evidence: .elai_cc\crates\tools\src\failure_taxonomy.rs:36-50
- BubbleUpAction::DiagnoseAndRetry — evidence: .elai_cc\crates\orchestrator\src\pipeline_tree_navigator.rs:162
- MistakeBook (SSB table) — evidence: .elai_cc\crates\core\src\mistake_book.rs, .elai_cc\crates\orchestrator\src\mistake_book.rs
- MicroReflection — evidence: .elai_cc\crates\core\src\micro_reflection.rs
- CloudSpendTracker referenced in cloud_spend_ssb_contract — evidence: .elai_cc\crates\contracts\tests\cloud_spend_ssb_contract.rs

### Prompt-to-Architecture — code side
- action_classifier (A-31) — evidence: .elai_cc\crates\tools\src\action_classifier.rs
- safety_filter (A-27) — evidence: .elai_cc\crates\tools\src\safety_filter.rs
- routing_rules (A-26) — evidence: .elai_cc\crates\tools\src\routing_rules.rs
- git_commit tool (A-28) — evidence: .elai_cc\crates\tools\src\git_commit.rs
- agent_template (A-29) — evidence: .elai_cc\crates\orchestrator\src\agent_template.rs

## Mechanisms PARTIAL

### Anti-Staleness: auto-recomputation trigger when deprecated_patterns changes
- exists: compute_steering_vector.py has --batch flag (.elai_cc\metrics\compute_steering_vector.py:228)
- gap: no automated trigger wiring from deprecated_patterns table change to steering vector recomputation found in codebase

### Reasoning Architecture: 3 new FailureTaxonomy categories (LAZY_CODE, EMPTY_BODY, FILE_SIZE_EXCEEDED)
- exists: DiagnosticResult/RootCause in failure_taxonomy.rs:36-50; LAZY_CODE_SCAN in deterministic_auditor.rs:11
- gap: FailureTaxonomy enum in failure_taxonomy.rs has 9 variants (SyntaxError..RuntimePanic); LAZY_CODE/EMPTY_BODY/FILE_SIZE_EXCEEDED not added as enum variants (.elai_cc\crates\tools\src\failure_taxonomy.rs:18-27)

### Pipeline Modes: SurgicalDecomposer / SurgicalPlan
- exists: blast_radius field appears in .elai_cc\crates\orchestrator\src\self_mod_safety.rs and rules; research pipeline ships
- gap: SurgicalDecomposer struct and SurgicalPlan type not found anywhere in codebase; --surgical and --new CLI flags absent from cli/src/args.rs

### Token Efficiency: phase-3J-BASELINE new step file
- exists: benchmark harness binary at .elai_cc\benches\model_capability\run_benchmark.py
- gap: steps/phase-3J-BASELINE.txt does not exist in docs/specs/guide/steps/

## Mechanisms MISSING

### Step file: steps/phase-3L.txt (Project Intelligence — 8 steps)
- searched: phase-3L, ls of steps/ directory; file does not exist

### Step file: steps/phase-3J-BASELINE.txt (Token Efficiency benchmark harness)
- searched: phase-3J-BASELINE, ls of steps/ directory; file does not exist

### Step file: steps/phase-4D-SURGICAL.txt (Surgical Mode sub-phase)
- searched: phase-4D-SURGICAL, ls of steps/ directory; file does not exist

### Step file: steps/phase-4D-ORACLE.txt (Oracle Mode sub-phase)
- searched: phase-4D-ORACLE, ls of steps/ directory; file does not exist

### Phase 1A spec update (extract_conventions public API, ConventionStats, SQL queries, regex)
- searched: extract_conventions, ConventionStats, NamingConventions in phase-1A.txt; no matches
- Code shipped at knowledge/src/conventions.rs; spec file not updated

### Phase 1E spec update (FreshnessConfig, freshness_meta.db, recency_decay)
- searched: FreshnessConfig, freshness_meta, recency_decay in phase-1E.txt; no matches
- Code shipped at knowledge/src/freshness.rs; spec file not updated

### Phase 3A spec update (RESEARCHING state, WebSearch+KnowledgeEngine gate)
- searched: RESEARCHING in phase-3A.txt; no matches
- RESEARCHING state exists in fsm.rs; spec file not updated

### Phase 3C spec update (deterministic_first, agent prompt template A-29)
- searched: deterministic_first, A-29 in phase-3C.txt; no matches
- Both implemented in code; spec file not updated

### Phase 3E spec update (ModelCapabilityProfile, SlotWeights per tier, constitutional injection, REPAIR_CONTEXT)
- searched: ModelCapabilityProfile, REPAIR_CONTEXT, constitutional in phase-3E.txt; no matches
- All implemented in context_budget.rs; spec file not updated

### Phase 3F spec update (ProjectRuleSeeder, VERSION_GATE, InterrogationTree, file_discipline)
- searched: ProjectRuleSeeder, VERSION_GATE, InterrogationTree, file_discipline in phase-3F.txt; no matches
- All implemented in code; spec file not updated

### Phase 3J spec update (ETCLogitsProcessor, GG signals, constrained decoding, tool routing A-26/27/28/31, steering vectors)
- searched: ETCLogitsProcessor, mean_logprob, action_classifier, routing_rules in phase-3J.txt; no matches
- All implemented in code; spec file not updated

### Phase 4B spec update (FailureTaxonomy expansion, DiagnosticResult, 3 new categories)
- searched: FailureTaxonomy, DiagnosticResult, LAZY_CODE in phase-4B.txt; not verified in spec
- Partial code delivery (9-category enum missing 3 new variants); spec file not updated

### Phase 4C spec update (PostToolUse deterministic auditor hooks)
- searched: PostToolUse deterministic in phase-4C.txt; no new block found
- ShortCoder PostToolUse exists; spec file not updated

### Phase 4D spec update (SurgicalDecomposer, blast radius, complexity classification, oracle mode)
- searched: SurgicalDecomposer, oracle, --surgical in phase-4D.txt; no matches
- Spec file not updated; SurgicalDecomposer not yet in code

### Phase 4E spec update (SurgicalPlan, FileLineTracker, skeleton-first, DiagnoseAndRetry integration)
- searched: SurgicalPlan, FileLineTracker, skeleton-first in phase-4E.txt; no matches
- DiagnoseAndRetry in code; rest not found; spec file not updated

### Phase 4G spec update (AutoAdvanceEvaluator escalation, CloudSpendTracker policy)
- searched: AutoAdvanceEvaluator escalation policy in phase-4G.txt (AutoAdvanceEvaluator is in ssb.rs context, not 4G); no matches in 4G spec

### Phase 4H spec update (nightly dep audit trigger, steering vector refresh automation)
- searched: nightly dep audit, steering refresh in phase-4H.txt area; not found as automation trigger

## Mechanisms UNVERIFIABLE

- Phase 4H nightly dep audit trigger: metrics/phase_4H.toml exists but cannot confirm automation trigger wiring without reading spec and source in depth
- CloudSpendTracker budget policy (50%/20% thresholds): cloud_spend_ssb_contract.rs exists but budget threshold logic not verified in 4G context

## What must be done (action list)

- [ ] Create steps/phase-3L.txt (8-step Project Intelligence spec): skeleton -> verify -> tests -> Tier1 -> Tier2 -> Tier3 -> telescope -> measure
- [ ] Create steps/phase-3J-BASELINE.txt (Token Efficiency benchmark harness spec): 7 measurements, ModelCapabilityProfile generation, standalone binary
- [ ] Create steps/phase-4D-SURGICAL.txt (Surgical Mode sub-phase of 4D): SurgicalDecomposer, blast radius, complexity classification, --surgical flag
- [ ] Create steps/phase-4D-ORACLE.txt (Oracle Mode sub-phase of 4D): /research command, source tier routing, oracle pipeline integration
- [ ] Update steps/phase-1A.txt: add extract_conventions() API, ConventionStats/NamingConventions types, 4 SQL queries, regex compilation
- [ ] Update steps/phase-1E.txt: add FreshnessConfig, freshness_meta.db schema, recency_decay*access_boost in pipeline.rs rerank
- [ ] Update steps/phase-3A.txt: add RESEARCHING state definition, allowed/blocked tool list, FSM transition
- [ ] Update steps/phase-3C.txt: add [agents.auditor] deterministic_first config, agent prompt template (A-29) section
- [ ] Update steps/phase-3E.txt: add ModelCapabilityProfile input, SlotWeights per tier, constitutional CONSTRAINTS injection, REPAIR_CONTEXT profile
- [ ] Update steps/phase-3F.txt: add ProjectRuleSeeder, VERSION_GATE predicate, deprecated_patterns schema, InterrogationTree storage, file_discipline config
- [ ] Update steps/phase-3J.txt: add ETCLogitsProcessor (A-9), GG signals (A-17/A-25), constrained decoding (A-10), tool routing (A-26), bash safety (A-27), git_commit (A-28), action classifier (A-31), steering vector batch mode (A-8)
- [ ] Update steps/phase-4B.txt: add FailureTaxonomy 3 new categories (LAZY_CODE, EMPTY_BODY, FILE_SIZE_EXCEEDED), expand DiagnosticResult
- [ ] Update steps/phase-4C.txt: add PostToolUse deterministic auditor + completeness hooks
- [ ] Update steps/phase-4D.txt: add SurgicalDecomposer, project detection signals, blast radius via DKG, complexity classification, regression gate
- [ ] Update steps/phase-4E.txt: add SurgicalPlan, FileLineTracker, skeleton-first generation, bridge file templates, DiagnoseAndRetry BubbleUp integration
- [ ] Update steps/phase-4G.txt: add AutoAdvanceEvaluator reasoning exhaustion check, CloudSpendTracker escalation policy (50%/20%/<20% thresholds)
- [ ] Update steps/phase-4H.txt: add nightly dep audit trigger, steering vector refresh automation wiring
- [ ] Add LAZY_CODE/EMPTY_BODY/FILE_SIZE_EXCEEDED variants to FailureCategory enum in .elai_cc\crates\tools\src\failure_taxonomy.rs:18
- [ ] Wire auto-recomputation trigger from deprecated_patterns table change to compute_steering_vector.py --batch invocation

## Notes
All 6 design files referenced exist under docs/specs/guide/. Code implementations are ahead of the spec files in most areas — the implementations shipped but the spec update step was skipped. The primary gap is doc/spec debt: 17 step files need updating, 4 new step files need creating. The only genuine code gaps are: SurgicalDecomposer/SurgicalPlan (pipeline-modes Surgical Mode not implemented), 3 missing FailureCategory enum variants, and the deprecated_patterns->steering auto-trigger.
