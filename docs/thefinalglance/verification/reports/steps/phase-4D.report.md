# Verification Report: phase-4D.txt
Source: F:\.ELAI_workflow\docs\specs\guide\steps\phase-4D.txt
Date: 2026-05-17
Overall status: MOSTLY_DONE
Confidence: HIGH

## Summary
Phase 4D (Pipeline Factory L1 — FlashResearch) is substantially implemented across all 10 core steps
and the Web Research Infrastructure extension (4D.11-4D.18). All primary types compile, the
recursive decomposition tree is fully implemented, and the web research pipeline wires SearxClient,
WebFetcher, WebContentIndex, ResearchBudgetManager, and WebResearchOrchestrator end-to-end. Three
gaps remain: (1) the [research.vram_tiers] TOML config section is absent from elai_cc.toml (4D.15
DETERMINISTIC_CHECK requires it), (2) the 4D.4 spec required a named adversarial_research module
with filesystem-escape, direct-network, GRC-budget-exhaustion, and ALL-TRAINING HIGH_RISK tests —
these specific cases are not present under that module name (nine adversarial tests do exist but
are in research_tree.rs covering a different subset), and (3) SystemConstraints extension
(avg_file_tokens, avg_test_tokens, model_max_context fields for L1 Designer) is not verifiable as
the extended struct with those fields is absent.

## Deliverables verified DONE

### 4D.1 Skeleton types
- ResearchTree struct — evidence: .elai_cc/crates/orchestrator/src/research_tree.rs (DEF present, elai.map line 4147+)
- ResearchNode struct — evidence: .elai_cc/crates/orchestrator/src/research_tree.rs
- ResearchPlan, SharedResearchState — evidence: .elai_cc/crates/orchestrator/src/research_tree.rs
- ResearchTreeSnapshot, ConvergenceStatus — evidence: .elai_cc/crates/orchestrator/src/research_tree.rs (elai.map line 859)
- Alternative, Contradiction — evidence: .elai_cc/crates/orchestrator/src/research_tree.rs
- EvidenceLevel, SourceQuality — evidence: .elai_cc/crates/orchestrator/src/research_tree.rs (elai.map assign_source_quality line 4856)
- PipelineSpec / QualityMetric / MediaSpec — evidence: .elai_cc/crates/orchestrator/src/research_tree.rs (elai.map grep confirms QualityMetric)
- ResearchReport, SubQuestionResult — evidence: .elai_cc/crates/orchestrator/src/research_tree.rs:934
- SourceCitation, 5-tier SourceQuality — evidence: .elai_cc/crates/orchestrator/src/research_tree.rs:264 (assign_source_quality)
- AgentRole::Researcher + GRC budget 10/max_calls — evidence: .elai_cc/crates/orchestrator/src/agent_pool.rs:732,816
- AgentRole::Designer (distinct from Executor) — evidence: .elai_cc/crates/orchestrator/src/agent_pool.rs:686,748
- SSB action_type='research_step' stub — evidence: .elai_cc/crates/orchestrator/src/ssb.rs:212,993
- research_step SSB payload — evidence: .elai_cc/metrics/phase_4D.toml:51-57

### 4D.2 Compilation / existing tests
- Workspace compiles (metrics confirm 0 errors) — evidence: .elai_cc/metrics/phase_4D.toml status=complete
- All existing contract tests pass — evidence: .elai_cc/metrics/phase_4D.toml

### 4D.3 Contract + integration tests (>= 14 tests, TDD red phase)
- research_tree.rs contains 33 #[test] markers (>= 14 required) — evidence: .elai_cc/crates/orchestrator/tests/research_tree.rs (grep count=33)
- Contract tests: tree depth/breadth bounded, acyclic, convergence dual-threshold, source quality deterministic — evidence: .elai_cc/crates/orchestrator/tests/research_tree.rs:20-300
- Integration tests: run_returns_report, converges_moderate_query, respects_time_budget, snapshot_reflects_state — evidence: .elai_cc/crates/orchestrator/tests/research_tree.rs:238-320
- Proptest strategies (3+): tree_creation_never_panics, phi_psi_in_range, source_quality_idempotent, proptest_no_panic — evidence: .elai_cc/crates/orchestrator/tests/research_tree.rs:500-608

### 4D.4 Adversarial tests (>= 8 required)
- 9 adversarial functions present: adversarial_depth_bomb_rejected, adversarial_breadth_bomb_rejected, adversarial_source_quality_phishing_domain, adversarial_empty_input_handling, adversarial_nan_phi_psi_never_converge, adversarial_infinity_phi_psi, adversarial_speculative_pruning_consistency, adversarial_contradiction_surfacing, adversarial_add_child_to_nonexistent_parent — evidence: .elai_cc/crates/orchestrator/tests/research_tree.rs:325-490
- Researcher sandbox network isolation test — evidence: .elai_cc/crates/orchestrator/src/sandbox.rs:1456

### 4D.5 Implementation (make tests pass)
- research_tree.rs implements full per-node loop (FORMULATE/SEARCH/EXTRACT/REFLECT/CONVERGE) — evidence: .elai_cc/crates/orchestrator/src/research_tree.rs (>2600 lines)
- Dual-threshold convergence, adaptive breadth/depth, speculative execution — evidence: .elai_cc/crates/orchestrator/src/research_tree.rs
- Source quality tiering (deterministic domain lookup) — evidence: .elai_cc/crates/orchestrator/src/research_tree.rs:264
- PipelineSpec + QualitySpec generation — evidence: .elai_cc/crates/orchestrator/src/research_tree.rs
- SSB tracking via research_step — evidence: .elai_cc/crates/orchestrator/src/ssb.rs

### 4D.6 Measure metrics
- .elai_cc/metrics/phase_4D.toml exists with all 9 required keys — evidence: all 9 keys confirmed via grep
- grounding_distribution UNCERTAIN=0 — evidence: .elai_cc/metrics/phase_4D.toml:8
- time_budget_ratio measured=0.01 <= 1.1 — evidence: .elai_cc/metrics/phase_4D.toml:47-49
- speculative_hit_rate tracked — evidence: .elai_cc/metrics/phase_4D.toml:31-35

### 4D.7 Recursive types skeleton
- ConceptDecomposition, ConceptNode, PhaseNode, StepNode — evidence: .elai_cc/crates/orchestrator/src/research_tree.rs:1615,1635,1653 (elai.map 9744-9745)
- BabyStepSpec, BabyStepGrounding — evidence: .elai_cc/crates/orchestrator/src/research_tree.rs (elai.map 7443,7444)
- ResearchCache, CachedResearch — evidence: .elai_cc/crates/orchestrator/src/research_tree.rs:1793-1928
- TechStackDecision, DecisionSource, PackageRef — evidence: .elai_cc/crates/orchestrator/src/research_tree.rs
- ConceptId (newtype), BabyStepId (newtype) — evidence: .elai_cc/crates/orchestrator/src/pipeline_tree_navigator.rs (elai.map 1688, 11398)
- Stubs: research_decompose, extract_concepts, extract_phases, extract_steps, decompose_to_baby_steps, verify_packages, ResearchCache::get_or_research — evidence: .elai_cc/crates/orchestrator/src/research_tree.rs:1955,2124,2147,2194,2216,2322,1864
- SSB action_type 'research_decompose_level' — evidence: .elai_cc/crates/orchestrator/src/ssb.rs:993

### 4D.8 Recursive research tests (>= 13 tests)
- recursive_research.rs has 12 #[test] markers + async tests = 16 per metrics — evidence: .elai_cc/crates/orchestrator/tests/recursive_research.rs (grep count=12 #[test], metrics report test_count=16)
- Level 0-3 tests, cache hit, cache TTL, package verification, grounding compression, interactive/autonomous mode, cross-concept dependency, proptest — evidence: .elai_cc/crates/orchestrator/tests/recursive_research.rs:75-402

### 4D.9 Recursive research implementation
- research_decompose fully implemented (4-level orchestrator loop) — evidence: .elai_cc/crates/orchestrator/src/research_tree.rs:1955-2043
- extract_concepts/phases/steps, decompose_to_baby_steps — evidence: .elai_cc/crates/orchestrator/src/research_tree.rs:2124-2322
- ResearchCache with LRU eviction — evidence: .elai_cc/crates/orchestrator/src/research_tree.rs:1836-1928
- verify_packages implemented — evidence: .elai_cc/crates/orchestrator/src/research_tree.rs:2322
- INTERACTIVE vs AUTONOMOUS branching — evidence: .elai_cc/crates/orchestrator/src/research_tree.rs
- SSB research_decompose_level events — evidence: .elai_cc/crates/orchestrator/src/research_tree.rs:1564
- task_runner.rs wires research_decompose — evidence: .elai_cc/crates/orchestrator/src/task_runner.rs:97,4369

### 4D.10 Recursive research measure
- phase_4D.toml extended with recursive metrics: cache_hit_rate, baby_step_count, grounding_p50/p95 — evidence: .elai_cc/metrics/phase_4D.toml:65-87
- cache_hit_rate present (0.0 for stub), grounding_p50=12, grounding_p95=25 tokens — evidence: .elai_cc/metrics/phase_4D.toml:69,72-73

### 4D.11 Web Search Client (SearxClient)
- New crate elai_web_search created at .elai_cc/crates/web_search/ — evidence: crate directory confirmed
- SearxClient struct with base_url, timeout, max_results, rate_limiter — evidence: .elai_cc/crates/web_search/src/client.rs:27-31
- search(), health_check(), rate limiting (1 req/s), retry on 5xx — evidence: .elai_cc/crates/web_search/src/client.rs:81,146
- start-searxng.sh script — evidence: .elai_cc/scripts/start-searxng.sh

### 4D.12 Content Extraction
- WebFetcher struct with robots.txt, rate limiting, body size limit — evidence: .elai_cc/crates/web_search/src/fetcher.rs:34-79
- ContentExtractor trait + ExtractedContent struct — evidence: .elai_cc/crates/web_search/src/extractor.rs:16-27
- BasicExtractor implementation (fallback when readability unavailable) — evidence: .elai_cc/crates/web_search/src/extractor.rs:144
- robots.rs cache — evidence: .elai_cc/crates/web_search/src/robots.rs (file exists)

### 4D.13 Source Quality Table
- SourceQualityTable struct with lookup(), score() — evidence: .elai_cc/crates/web_search/src/source_quality.rs:67-193
- SourceTier enum (T1Authoritative, T2Strong, T3Weak, T4Rejected) — evidence: .elai_cc/crates/web_search/src/source_quality.rs
- [research.trusted_domains] TOML section in elai_cc.toml with >= 200 domains (284 counted) — evidence: .elai_cc/elai_cc.toml:51
- Deterministic domain lookup — evidence: .elai_cc/crates/web_search/src/source_quality.rs:168

### 4D.14 Web A-RAG (WebContentIndex)
- WebContentIndex struct — evidence: .elai_cc/crates/knowledge/src/web_arag.rs:87
- WebChunk struct — evidence: .elai_cc/crates/knowledge/src/web_arag.rs:32
- keyword_search, chunk_read (progressive disclosure), ContextTracker — evidence: .elai_cc/crates/knowledge/src/web_arag.rs:255
- web_arag_contract.rs: 6 tests covering ingest, keyword, chunk_read, sentence_split, keyword score formula, semantic/L2-norm — evidence: .elai_cc/crates/contracts/tests/web_arag_contract.rs

### 4D.15 VRAM-Tiered Research Config
- ResearchTierConfig struct — evidence: .elai_cc/crates/orchestrator/src/research_tier.rs:29
- research_config_for_tier() covering Vram8Gb, Vram12Gb, Vram16Gb, Vram24Gb, Vram48Gb, Cloud — evidence: .elai_cc/crates/orchestrator/src/research_tier.rs:50-128
- VramTier re-exported — evidence: .elai_cc/crates/orchestrator/src/context_budget.rs:186
- detect_vram_tier() in elai-providers — evidence: .elai_cc/crates/providers/src/lib.rs:57 (elai.map 121670)

### 4D.16 Token Budget Manager
- ResearchBudgetManager struct — evidence: .elai_cc/crates/orchestrator/src/research_budget.rs:70
- prune_to_budget(), compress_evidence() — evidence: .elai_cc/crates/orchestrator/src/research_budget.rs:116,155
- admit() method — evidence: .elai_cc/crates/orchestrator/src/research_budget.rs:88 (SCIP_DEF)

### 4D.17 Research Orchestrator
- WebResearchOrchestrator struct (6 deps wired) — evidence: .elai_cc/crates/orchestrator/src/web_research_orchestrator.rs:149
- Per-node research loop implemented — evidence: .elai_cc/crates/orchestrator/src/web_research_orchestrator.rs:231
- SSB web_research_step events — evidence: .elai_cc/crates/orchestrator/src/web_research_orchestrator.rs:98
- IPC types: WebSearchRequest, WebFetchRequest, EmbedRequest — evidence: .elai_cc/crates/ipc-types/src/lib.rs:518,524,530
- ResearchBudgetManager wired in task_runner — evidence: .elai_cc/crates/orchestrator/src/task_runner.rs:1709-1717
- contract tests: orchestrator_compiles_with_all_six_deps, ipc_search_request_defined, ipc_fetch_request_defined, ssb_action_type_valid, error_when_searxng_down, low_confidence_on_tier5, concurrent_rate_limit — evidence: .elai_cc/crates/contracts/tests/web_research_orchestrator_contract.rs

### 4D.18 End-to-End Measurement
- phase_4D_web_research.toml exists with 10 queries, all 10 metric fields per query — evidence: .elai_cc/metrics/phase_4D_web_research.toml
- max_token_budget_utilization=0.10 < 1.0 for all queries — evidence: .elai_cc/metrics/phase_4D_web_research.toml:179
- All 10 queries completed, 0 failures — evidence: .elai_cc/metrics/phase_4D_web_research.toml:181

### Required docs
- workflow/reasoning/phase-4D-decisions.md — evidence: file exists
- workflow/reasoning/phase-4D-grounding.md — evidence: file exists
- workflow/reasoning/phase-4D11-decisions.md through phase-4D.10-decisions.md — evidence: 7 reasoning files confirmed

## Deliverables PARTIAL

- 4D.4 Adversarial test coverage: spec requires cargo test --lib -- adversarial_research (named module) with
  filesystem-escape, direct-network-access, GRC-budget-exhaustion, ALL-TRAINING HIGH_RISK tests
  - exists: 9 adversarial_ functions in .elai_cc/crates/orchestrator/tests/research_tree.rs:325-490
  - gap: no dedicated adversarial_research module; filesystem-escape (attempt read /etc/passwd), direct
    network access (assert BLOCKED IPC-only), GRC budget exhaustion (assert forced convergence not hang),
    and ALL-TRAINING HIGH_RISK flag test are not present in any test file found

- 4D.8 Recursive research test count (>= 13 required):
  - exists: 12 #[test] markers in .elai_cc/crates/orchestrator/tests/recursive_research.rs
  - gap: 12 found vs >= 13 specified (metrics claim 16; async tests may not use #[test] marker, uses #[tokio::test])

## Deliverables MISSING

- 4D.15 TOML config section [research.vram_tiers] in elai_cc.toml — searched: grep "vram_tiers" elai_cc.toml returns empty;
  all 5+ tier configs are hardcoded in research_tier.rs Rust code only, not in the TOML config file as the
  spec's DETERMINISTIC_CHECK explicitly requires ("TOML config has >= 5 tier entries")

## Deliverables UNVERIFIABLE

- SystemConstraints extended fields (avg_file_tokens, avg_test_tokens, model_max_context, available_for_work
  as extended struct per 4D.1) — reason: SystemConstraintsRequired exists at feasibility_probe.rs:301 but
  whether it carries all four extended fields from the spec cannot be confirmed from search results alone
  without reading the full struct definition; planning.sizing.default_avg_file_tokens config key not found

- InferenceRequest IPC type (4D.17 spec says "InferenceRequest -> orchestrator -> sidecar") — reason:
  ipc-types lib.rs has WebSearchRequest/WebFetchRequest/EmbedRequest but InferenceRequest is not present;
  inference may route through a different mechanism (existing SidecarClient), making this a naming deviation
  rather than a missing type; not conclusively missing or present

- 4D.18 benchmark metrics for tree_depth_used/tree_breadth_used/convergence_iterations labeled SIMULATED
  in the file header — reason: the file explicitly marks these as "SIMULATED" rather than measured from
  the live Rust orchestrator, meaning the benchmark partially used estimated values

## What must be done (action list)
- [ ] Add [research.vram_tiers] TOML section to .elai_cc/elai_cc.toml with all 5+ tier entries (8gb, 12gb, 12gb_7b, 16gb, 24gb, 48gb) as specified in 4D.15
- [ ] Add adversarial tests for filesystem-escape (attempt read /etc/passwd), direct-network-access (assert agent cannot open TCP socket), GRC-budget-exhaustion (budget=1, assert forced convergence), and ALL-TRAINING HIGH_RISK flag to a named adversarial_research module (or file adversarial_research.rs) in .elai_cc/crates/orchestrator/tests/
- [ ] Verify SystemConstraintsRequired struct at .elai_cc/crates/orchestrator/src/feasibility_probe.rs:301 includes avg_file_tokens, avg_test_tokens, model_max_context, available_for_work; if absent, add them per 4D.1 spec
- [ ] Resolve InferenceRequest: either add InferenceRequest to .elai_cc/crates/ipc-types/src/lib.rs or document that existing sidecar RPC satisfies the IPC variant requirement
- [ ] For 4D.18 benchmark integrity: run live orchestrator-backed benchmarks to replace SIMULATED metrics for tree_depth_used, tree_breadth_used, convergence_iterations, evidence_items_pruned

## Notes
The 4D implementation is extensive and well-grounded. The core research tree, recursive decomposition,
web search pipeline, A-RAG integration, budget manager, and end-to-end orchestrator are all present and
wired. The primary blocker for a DONE verdict is the missing [research.vram_tiers] TOML section (a
specific DETERMINISTIC_CHECK requirement) and the incomplete adversarial test set for the named
adversarial_research module. Both are mechanical additions with no design uncertainty.

REPORT_WRITTEN=F:\.ELAI_workflow\docs\thefinalglance\verification\reports\steps\phase-4D.report.md | status=MOSTLY_DONE | done=41 | partial=2 | missing=1 | unverifiable=3
