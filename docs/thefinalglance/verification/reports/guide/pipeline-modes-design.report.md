# Verification Report: pipeline-modes-design.txt
Source: F:\.ELAI_workflow\docs\specs\guide\pipeline-modes-design.txt
Date: 2026-05-17
Overall status: PARTIAL
Confidence: HIGH

## Summary
The Oracle Mode (RESEARCHING FSM state, source-tier classification, research pipeline types,
cross-verification, "I don't know" guard, /research CLI command, message classifier) is
substantially implemented. The Surgical Mode structural types (SurgicalPlan, AffectedSet,
ComplexityClass, RegressionGate, TestBaseline) are absent from the codebase; the design's
core decomposition path is not implemented. The Context Telescope (greenfield 4-layer and
surgical 3-layer) is implemented. DKG data-flow edges (GAP 1 from the design) are
implemented as a post-spec addition. Project detection signals and the first-task indexing
prompt flow are not present.

## Mechanisms verified DONE

- Oracle Mode FSM state RESEARCHING (IDLE -> RESEARCHING -> IDLE, RESEARCHING -> RESEARCHING follow-up)
  — evidence: crates/orchestrator/src/fsm.rs:1430-1437

- RESEARCHING tool permission gates (WebSearch/WebFetch/read_file allowed; edit_file/bash blocked)
  — evidence: crates/orchestrator/src/tool_filter.rs:562-569

- /research CLI command and /ask alias parsing
  — evidence: crates/cli/src/commands.rs:5312; crates/cli/src/live_cli.rs:2231

- Message classifier (classify_bare_message routing to /research, /explore, /task)
  — evidence: crates/cli/src/classifier.rs:83; tests at classifier.rs:129,135

- /task --surgical and /task --new flags (parse_task_flags)
  — evidence: crates/cli/src/commands.rs:2006; crates/cli/src/live_cli.rs:2346

- Oracle Mode types: ResearchQuery, SourceTier (T1-T4), ExtractedFact, VerifiedClaim,
  VerificationLevel, ResearchAnswer, AnswerConfidence
  — evidence: crates/core/src/research_types.rs (SCIP_DEF lines for all types)

- Source tier classification (domain lookup, load_source_tier_config, classify_source)
  — evidence: crates/core/src/source_tier.rs:115,128; crates/web_search/src/source_quality.rs:91

- source-tiers.toml config file loading with contract test
  — evidence: crates/contracts/tests/a45_source_tiers_toml_contract.rs:34; crates/core/src/source_tier.rs:115

- Oracle pipeline phases A-G: query formulation, search, classification, fact extraction,
  cross-verification, synthesis, post-verification checks (citation, tier consistency,
  "I don't know" enforcement, hallucination guard)
  — evidence: crates/orchestrator/src/research_pipeline.rs:296,505,560-588

- AnswerConfidence compute_confidence (HIGH/MEDIUM/LOW/UNKNOWN logic)
  — evidence: crates/orchestrator/src/research_pipeline.rs:505; tests at 1201,1220,1231,1236

- ResearchCache with TTL and LRU eviction
  — evidence: crates/orchestrator/tests/recursive_research.rs:328,345-357

- TechStackDecision + DecisionSource enum
  — evidence: crates/orchestrator/src/research_tree.rs:1770,1363

- PackageRef + API registry check (VerificationMethod::ApiRegistryCheck)
  — evidence: crates/orchestrator/src/research_tree.rs:1370; crates/reasoning/tests/verification_tests.rs:170

- ContextTelescope (4-layer: project_vision, concept_summary, phase_goal, step_context)
  with BoundedString token caps, render(), token_estimate()
  — evidence: crates/core/src/telescope.rs:34-38; crates/contracts/tests/a46_telescope_contract.rs:71-110

- SurgicalTelescope (3-layer: project_vision, blast_radius, step_task) with token cap
  — evidence: crates/core/src/telescope.rs:93,103,116; crates/contracts/tests/a46_telescope_contract.rs:120-142

- collapse_to_budget for telescope budget enforcement
  — evidence: crates/orchestrator/src/telescope_caller.rs:161

- IndexingService with DkgEngine::new + build_graph wired at startup
  — evidence: crates/orchestrator/src/indexing_service.rs:358,553; crates/orchestrator/src/startup.rs:562

- DKG data-flow edges (FlowType: PARAMETER/RETURN/ASSIGNMENT/FIELD_ACCESS) — GAP 1 from design
  — evidence: crates/knowledge/src/dkg/mod.rs:797; crates/contracts/tests/dkg_data_flow_a48_contract.rs

- PageRank implementation and pagerank_top query
  — evidence: crates/knowledge/src/pagerank/mod.rs:277; crates/knowledge/tests/error_propagation.rs:118

- ResearchDrawer dashboard UI (source tier distribution, research tree display)
  — evidence: crates/dashboard-v2/src/lib/drawers/ResearchDrawer.svelte:10,80-81

- Post-verification Phase G checks (citations, tier consistency, "I don't know", hallucination guard)
  — evidence: crates/orchestrator/src/research_pipeline.rs:560-588

## Mechanisms PARTIAL

- IndexingService wired at startup (service exists, DkgEngine called within it)
  — exists: crates/orchestrator/src/startup.rs:562; crates/orchestrator/src/indexing_service.rs:553
  — gap: No interactive first-task confirmation flow ("Detected existing project. Index? Y/n").
    Startup runs indexing of known project roots unconditionally; there is no user-facing prompt
    that fires on first /task in a new directory per section 1B of the design.

- Project detection signal logic (REQUIRED + STRONG signal scheme from section 1)
  — exists: no dedicated detector struct found; indexing runs via startup.rs scan of indexes_root
  — gap: The weighted-signal detection table (Cargo.toml REQUIRED, >10 source files STRONG, etc.)
    and the auto-detection-before-first-task gate are not implemented as described. The system
    discovers existing indexed projects but does not perform fresh signal-based detection for
    new directories.

- Source tier TTL-based answer caching (recommendation from OPEN QUESTIONS section 2)
  — exists: ResearchCache with TTL at crates/orchestrator/src/research_tree.rs:1937 (for research
    sub-queries during decomposition)
  — gap: No dedicated Oracle Mode answer-level cache with tier-based TTL (T1=30d, T2=7d, T3=1d)
    for /research command answers. The existing ResearchCache is for decomposition reuse, not
    final oracle answer caching.

- DKG query tools exposed to agents (GAP 2 from design: dkg_callers, dkg_dependents, etc.)
  — exists: DkgQuerierAdapter with query_reachability at crates/knowledge/src/dkg_adapter.rs:122
  — gap: The design's proposed agent-callable tool wrappers (dkg_callers, dkg_dependents,
    dkg_data_flow, dkg_subgraph as IPC tools) are not present. Agents receive pre-selected
    context; they cannot issue DKG graph queries as tools.

## Mechanisms MISSING

- SurgicalPlan struct — searched: SurgicalPlan, SurgicalPhase, SurgicalStep, SurgicalDecompos;
  no evidence in codebase

- AffectedSet struct (blast radius analysis output type) — searched: AffectedSet, affected_set,
  direct_files, dependent_files, total_loc_estimate; no evidence

- ComplexityClass enum (TRIVIAL/SMALL/MEDIUM/LARGE) — searched: ComplexityClass, TRIVIAL,
  MEDIUM, LARGE in context of complexity; no evidence

- RegressionGate struct (per surgical phase) — searched: RegressionGate, regression_gate,
  must_pass, allow_new_failures; no evidence for this type (check_regression_gate in
  knowledge/benchmark.rs is a metrics regression gate, different purpose)

- TestBaseline struct (snapshot_hash, pre-existing failures) — searched: TestBaseline,
  snapshot_hash, pre-existing.*fail; no evidence

- Surgical Decomposer (section 3 steps 1-4): blast radius graph traversal via DKG,
  complexity classification, constrained step generation, regression gate injection
  — searched: SurgicalDecomposer, blast_radius.*analysis, complexity_classification;
  no evidence for the decomposition pipeline

- Pipeline factory with surgical/greenfield mode selection — searched: PipelineFactory,
  pipeline_factory, SurgicalDecomposer, ConceptDecomposer; no evidence for a factory or
  mode-selection dispatch

- Blast radius LOC threshold enforcement with user confirmation (section OPEN QUESTIONS 6:
  >1000 LOC autonomous confirmation, >5000 LOC refusal) — searched: max_blast_radius,
  blast_radius.*threshold, loc_threshold, surgical.*threshold; no evidence

- DEGRADED mode (BM25_ONLY) on index schema version mismatch — searched: BM25_ONLY,
  DEGRADED.*mode, schema.*mismatch.*index; no evidence

## Mechanisms UNVERIFIABLE

- Research priority ordering in surgical mode (P1 inward/DKG, P2 deps docs, P3 external)
  — reason: section 4 describes runtime query routing logic; no clear hook in task_runner
  or web_research_orchestrator ties the surgical flag to a priority-ordered search strategy.
  The flag parse_task_flags captures surgical=true but downstream use in search ordering
  cannot be confirmed without tracing task_runner beyond elai.map snippet depth.

- Max follow-up count (5 per session) for RESEARCHING -> RESEARCHING transitions
  — reason: FSM transition oracle-followup is present (fsm.rs:1437) but no counter or
  max-followup enforcement constant was found in the map; may be inside fsm.rs logic
  not surfaced in map snippets.

- Line-level granularity (GAP 3: def-ref edges behind feature gate)
  — reason: dkg_data_flow edges are present (FlowType), but whether line-level def-ref
  edges distinct from function-level nodes are implemented or gated is not determinable
  from map snippets alone.

## What must be done (action list)

- [ ] Implement SurgicalPlan, AffectedSet, ComplexityClass, RegressionGate, TestBaseline
  types (target: crates/orchestrator/src/surgical_plan.rs or similar new file)

- [ ] Implement SurgicalDecomposer: DKG blast radius traversal (steps 1-2), constrained
  step generation (step 3), regression gate injection (step 4)
  (target: crates/orchestrator/src/surgical_decomposer.rs)

- [ ] Wire surgical flag from parse_task_flags through task_runner dispatch to select
  SurgicalDecomposer vs existing ConceptDecomposer
  (target: crates/orchestrator/src/task_runner.rs, crates/cli/src/live_cli.rs)

- [ ] Add project detection signal logic (weighted-signal table) and first-task indexing
  confirmation prompt when no .elai_cc/indexes/ exists for a directory
  (target: crates/cli/src/live_cli.rs or crates/orchestrator/src/startup.rs)

- [ ] Add DEGRADED mode (BM25_ONLY fallback) when index schema version mismatches
  (target: crates/orchestrator/src/indexing_service.rs)

- [ ] Add Oracle Mode answer-level cache with tier-based TTL (T1=30d, T2=7d, T3=1d)
  separate from decomposition ResearchCache
  (target: crates/orchestrator/src/ or crates/core/src/)

- [ ] Add blast radius LOC threshold gates with user confirmation for autonomous mode
  (>1000 LOC) and refusal (>5000 LOC) and interactive warnings
  (target: crates/orchestrator/src/surgical_decomposer.rs once created)

- [ ] Expose DKG query tools to agents (dkg_callers, dkg_dependents, dkg_data_flow,
  dkg_subgraph) as callable IPC tools wrapping DkgQuerierAdapter
  (target: crates/tools/src/ + IPC wiring)

## Notes

The design's own section 1B explicitly documents the DKG pipeline gap as of 2026-04-10.
That gap is now partially closed: IndexingService calls DkgEngine::new at startup.rs:562.
However, the first-task interactive flow and project signal detection remain unimplemented.
Oracle Mode is the most complete subsystem; its types, pipeline phases, and FSM integration
are all present with contract test coverage. Surgical Mode exists only as CLI flags and the
SurgicalTelescope type — the decomposition pipeline itself is entirely absent.
