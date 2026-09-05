# C-30: Surgical Mode + Oracle Mode + Pipeline Infrastructure
Objective: implement the missing Surgical Mode decomposition pipeline, Oracle
answer-level cache, DKG query tool catalog exposure, project detection signal
logic, blast-radius LOC gates, and [research.vram_tiers] TOML section per
pipeline-modes-design spec.
Estimated effort: 2-3 sessions.
Pre-requisites: C-17 (DKG resilience + IndexingService entry-point detection),
C-24 (symbol_impact tools in build_tool_catalog).
Parallel-safe with: C-26, C-27, C-28, C-29.

## Context

pipeline-modes-design.report.md (2026-05-17, status=PARTIAL) identifies the
following verified gaps:

MISSING entirely: SurgicalPlan, AffectedSet, ComplexityClass, RegressionGate,
TestBaseline types; SurgicalDecomposer (DKG blast-radius traversal, constrained
step generation, regression gate injection); pipeline factory mode dispatch.

PARTIAL: project detection signal logic (weighted-signal table, first-task
confirmation prompt) — startup.rs scans indexes_root unconditionally; no
signal-weighted detector or user-facing Y/n prompt exists. Oracle Mode
answer-level cache with tier-based TTL (T1=30d, T2=7d, T3=1d) — ResearchCache
at research_tree.rs:1805 covers decomposition reuse, not final /research
answer caching. DKG query tools exposed to agents — DkgQuerierAdapter at
knowledge/src/dkg_adapter.rs:17 (query_reachability only); no dkg_callers,
dkg_dependents, dkg_data_flow, dkg_subgraph tools in build_tool_catalog
(production_agent_rpc.rs:207).

MISSING from phase-4D.report.md: [research.vram_tiers] TOML section in
elai_cc.toml (DETERMINISTIC_CHECK requires >=5 entries; all config hardcoded
in research_tier.rs only).

update-implementation-order.report.md confirms: blast-radius LOC thresholds
(autonomous_confirm_above_loc, autonomous_refuse_above_loc) absent from
EscalationPolicy; no user confirmation gate in SurgicalDecomposer.

Existing entry points to wire:
- parse_task_flags (cli/src/commands.rs:2006) already captures surgical=true.
- run_research_pipeline (orchestrator/src/research_pipeline.rs:635) is the
  /research call site (live_cli.rs:2203).
- IndexingService::indexes_exist (indexing_service.rs:304) is the probe point
  for the first-task gate.
- build_tool_catalog (production_agent_rpc.rs:207) is the catalog injection
  point; symbol_impact is already wired there (line 862).

## Reads

- F:\.ELAI_workflow\docs\thefinalglance\verification\reports\guide\pipeline-modes-design.report.md
- F:\.ELAI_workflow\docs\thefinalglance\verification\reports\guide\update-implementation-order.report.md
- F:\.ELAI_workflow\docs\thefinalglance\verification\reports\steps\phase-4D.report.md
- .elai_cc/crates/orchestrator/src/task_runner.rs (surgical dispatch hook, lines 6170-6200)
- .elai_cc/crates/orchestrator/src/research_tree.rs (ResearchCache, lines 1793-1937)
- .elai_cc/crates/orchestrator/src/research_pipeline.rs (run_research_pipeline, lines 635-700)
- .elai_cc/crates/orchestrator/src/production_agent_rpc.rs (build_tool_catalog, lines 207-300)
- .elai_cc/crates/knowledge/src/dkg_adapter.rs (DkgQuerierAdapter, lines 1-80)
- .elai_cc/crates/orchestrator/src/indexing_service.rs (indexes_exist, line 304)
- .elai_cc/elai_cc.toml (sections [research.*])

## Tasks

1. Define core Surgical Mode types in orchestrator/src/surgical_decomposer.rs (new).
   Structs: SurgicalPlan { steps: Vec<SurgicalStep>, affected_set: AffectedSet,
   complexity_class: ComplexityClass, regression_gate: RegressionGate }.
   AffectedSet { direct_files: Vec<PathBuf>, dependent_files: Vec<PathBuf>,
   total_loc_estimate: u32 }. ComplexityClass enum { Simple, Medium, Complex }.
   RegressionGate { must_pass: Vec<String>, allow_new_failures: u32,
   snapshot_hash: String }. TestBaseline { snapshot_hash: String,
   pre_existing_failures: Vec<String> }. SurgicalStep { description: String,
   target_file: PathBuf, target_fn: Option<String> }. No .unwrap(); all
   fallible paths return Result<_, SurgicalError>.

2. Implement SurgicalDecomposer in orchestrator/src/surgical_decomposer.rs.
   Steps per pipeline-modes-design section 3:
   (a) DKG blast-radius traversal — open DkgQuerierAdapter::new(db_path),
       call query_reachability for the target symbol; populate AffectedSet
       from ReachabilityMatch paths. Compute total_loc_estimate via stored
       node LOC metadata.
   (b) Complexity classification — Simple: blast_radius_files <= 3 and LOC
       <= 200. Medium: blast_radius_files <= 8 or LOC <= 500. Complex:
       otherwise.
   (c) Constrained step generation — emit one SurgicalStep per affected file
       in topological order from DKG edge traversal; cap at complexity_class
       maximum (Simple=1 step, Medium=4, Complex=8).
   (d) RegressionGate injection — per step, identify must_pass test names
       from DKG node test-coverage metadata; snapshot_hash from blake3 of
       sorted must_pass vec.
   LOC threshold gate: if total_loc_estimate > 500, emit SSB row
   surgical_loc_warning. In INTERACTIVE ExecutionMode, return
   Err(SurgicalError::ConfirmationRequired { loc: u32 }) before returning
   SurgicalPlan; caller in task_runner prompts user and re-calls with
   confirmed=true. Autonomous mode: if total_loc_estimate > 1000, escalate.

3. Wire surgical dispatch in orchestrator/src/task_runner.rs.
   At the ConceptDecomposition entry point (line 6179 context): check the
   surgical flag captured by parse_task_flags. If surgical=true, invoke
   SurgicalDecomposer::decompose() before research_decompose and return the
   SurgicalPlan-driven step sequence instead of ConceptDecomposition. Emit
   SSB row surgical_plan_created with complexity_class and blast_radius_files.
   Existing non-surgical path unchanged.

4. Add project detection signal logic.
   In live_cli.rs, before the first /task dispatch: check
   IndexingService::indexes_exist(). If false: emit weighted-signal scan
   (Cargo.toml present = REQUIRED; >=10 .rs files = STRONG; go.mod present
   = REQUIRED; pyproject.toml = REQUIRED; package.json = STRONG).
   If any REQUIRED signal fires and index absent: in INTERACTIVE mode, print
   "Detected new project. Build knowledge index? [Y/n]" and read stdin.
   If confirmed (or AUTONOMOUS mode): trigger IndexingService startup scan.
   If rejected: proceed without index (DKG falls back to symbol_impact).
   Log detection signal results as SSB row project_detection_signal with
   fields: signal_name, weight, fired.

5. Implement OracleAnswerCache in orchestrator/src/oracle_cache.rs (new).
   Struct OracleAnswerCache { inner: HashMap<CacheKey, CachedAnswer>,
   max_entries: usize }. CacheKey = (query_hash: [u8;32], project_hash: [u8;32])
   where both hashes are blake3. CachedAnswer { answer: ResearchAnswer,
   cached_at: u64 (unix secs), source_tier: SourceTier }.
   TTL by tier: T1Authoritative=2592000s (30d), T2Strong=604800s (7d),
   T3Weak=86400s (1d), T4Rejected=0 (never cache).
   Methods: get(&CacheKey, now_secs) -> Option<&CachedAnswer>,
   insert(CacheKey, CachedAnswer), evict_expired(now_secs), len() -> usize.
   LRU eviction when len() > max_entries (same pattern as ResearchCache).
   No external deps beyond std + blake3 (already in workspace).

6. Wire OracleAnswerCache into /research entry point.
   In live_cli.rs at run_research_pipeline call site (line 2203):
   before calling run_research_pipeline, compute CacheKey from blake3 of
   (query text, project_root canonical path). Call oracle_cache.get(key, now).
   Cache hit: return cached answer; emit SSB row oracle_cache_hit with
   cached_age_secs and source_tier.
   Cache miss: call run_research_pipeline normally; on Ok(answer), determine
   source_tier from answer.answer_confidence (HIGH->T1, MEDIUM->T2, LOW->T3);
   call oracle_cache.insert; emit SSB row oracle_cache_miss.

7. Expose DKG query tools in build_tool_catalog.
   In production_agent_rpc.rs, add four ToolDefinition entries after the
   existing symbol_impact entry:
   - dkg_callers: input={symbol: string}; calls DkgQuerierAdapter with
     edge_types=["CALLS"], direction=Inbound; returns JSON array of caller paths.
   - dkg_dependents: input={symbol: string}; edge_types=["IMPORTS","USES"],
     direction=Outbound; returns JSON array of dependent file paths.
   - dkg_data_flow: input={symbol: string, max_depth: u8}; traverses FlowType
     edges (PARAMETER/RETURN/ASSIGNMENT/FIELD_ACCESS); returns JSON array of
     (source, target, flow_type) triples.
   - dkg_subgraph: input={root: string, depth: u8}; BFS from root up to depth;
     returns compact JSON {nodes:[...], edges:[...]} for agent context.
   Each tool handler opens DkgQuerierAdapter from the session project_root DKG
   db path. Returns Err JSON on DkgError::QueryFailed. No panics.

8. Add [research.vram_tiers] to elai_cc.toml.
   Add section after [research.trusted_domains]. Entries mirror the 6 tiers
   hardcoded in research_tier.rs:50-128: 8gb, 12gb, 12gb_7b, 16gb, 24gb,
   cloud. Each entry: max_search_results, max_depth, max_breadth,
   max_token_budget, enable_speculative (bool). DETERMINISTIC_CHECK passes
   when >=5 entries present.

9. Contract tests for C-30 (TDD: write before implementation).
   File: crates/contracts/tests/c30_surgical_oracle_contract.rs.
   Tests (minimum 12):
   - surgical_plan_types_compile: SurgicalPlan, AffectedSet, ComplexityClass,
     RegressionGate, TestBaseline all constructible.
   - complexity_class_simple_bound: blast_radius_files=2, loc=150 -> Simple.
   - complexity_class_medium_bound: blast_radius_files=6, loc=400 -> Medium.
   - complexity_class_complex_bound: blast_radius_files=10, loc=700 -> Complex.
   - regression_gate_snapshot_hash_deterministic: same must_pass vec always
     same blake3 hash.
   - oracle_cache_miss_then_hit: insert answer, get returns Some within TTL.
   - oracle_cache_ttl_t1_30d: T1 entry survives 29d, expires at 30d+1s.
   - oracle_cache_ttl_t3_1d: T3 entry expires after 86401s.
   - oracle_cache_t4_rejected_never_cached: insert T4 -> get returns None.
   - oracle_cache_eviction_at_max: insert max_entries+1 -> oldest evicted.
   - dkg_callers_tool_definition_in_catalog: build_tool_catalog contains
     ToolDefinition with name="dkg_callers".
   - dkg_dependents_tool_definition_in_catalog: name="dkg_dependents" present.
   - dkg_data_flow_tool_definition_in_catalog: name="dkg_data_flow" present.
   - dkg_subgraph_tool_definition_in_catalog: name="dkg_subgraph" present.
   - vram_tiers_toml_has_five_entries: parse elai_cc.toml [research.vram_tiers]
     -> entry count >= 5.
   - surgical_loc_warning_ssb_above_500: decomposer with mocked AffectedSet
     loc=600 emits surgical_loc_warning SSB row.
   - project_detection_signal_required_fires_on_cargo_toml: signal scan with
     Cargo.toml present returns REQUIRED fired=true.

## WIRING_CONTRACT

produces_symbols:
  - SurgicalPlan, SurgicalStep, AffectedSet, ComplexityClass, RegressionGate,
    TestBaseline, SurgicalError (orchestrator/src/surgical_decomposer.rs)
  - SurgicalDecomposer struct + decompose() method
  - OracleAnswerCache, CacheKey, CachedAnswer (orchestrator/src/oracle_cache.rs)
  - SSB action_types: surgical_plan_created, surgical_loc_warning,
    oracle_cache_hit, oracle_cache_miss, project_detection_signal
  - ToolDefinition entries: dkg_callers, dkg_dependents, dkg_data_flow,
    dkg_subgraph in build_tool_catalog
  - [research.vram_tiers] TOML section (>=5 entries)

consumes_symbols:
  - DkgQuerierAdapter::new + query_reachability (knowledge/src/dkg_adapter.rs)
  - parse_task_flags surgical=true flag (cli/src/commands.rs:2006)
  - run_research_pipeline (orchestrator/src/research_pipeline.rs:635)
  - IndexingService::indexes_exist (orchestrator/src/indexing_service.rs:304)
  - ResearchAnswer, AnswerConfidence (core/src/research_types.rs)
  - ExecutionMode::Interactive / Autonomous (orchestrator task_runner)
  - blake3 (already in workspace; no new dep)

reachable_from:
  - elai /task --surgical CLI flag -> task_runner -> SurgicalDecomposer
  - elai /research -> OracleAnswerCache lookup -> run_research_pipeline
  - agent tool calls dkg_callers/dkg_dependents/dkg_data_flow/dkg_subgraph
  - elai_cc.toml [research.vram_tiers]

stub_ban:
  - SurgicalDecomposer::decompose MUST perform real DKG traversal; no
    synthetic AffectedSet in production paths.
  - OracleAnswerCache::get MUST enforce TTL; no always-cache-hit path.
  - dkg_* tool handlers MUST open DkgQuerierAdapter; no hardcoded mock returns.
  - LOC gate MUST fire in INTERACTIVE mode before returning SurgicalPlan
    when total_loc_estimate > 500.

observable_effect:
  trigger: elai /task --mode cloud --surgical "fix the off-by-one in
    get_users pagination at api/users.rs:LINE_X" (cloud Sonnet 4.6)
  expect_ssb: surgical_plan_created row with blast_radius_files<=3 and
    complexity_class=Simple or Medium; AffectedSet populated; after fix,
    RegressionGate passes with no new test failures outside AffectedSet.
  trigger2: elai /research --mode cloud "What does FooService do?" (x2)
  expect_ssb: first call oracle_cache_miss; second call oracle_cache_hit
    with cached_age_secs < TTL for detected source tier.
  trigger3: elai /task --surgical on function with blast_radius_loc>500
    in INTERACTIVE mode
  expect: user confirmation prompt fires before SurgicalPlan returned.

## DETERMINISTIC_CHECK

```bash
# 1. SurgicalPlan and related types defined
rg "pub struct SurgicalPlan|pub struct AffectedSet|pub enum ComplexityClass|pub struct RegressionGate|pub struct TestBaseline" .elai_cc/crates/orchestrator/src/surgical_decomposer.rs

# 2. OracleAnswerCache defined
rg "pub struct OracleAnswerCache|pub struct CachedAnswer|pub struct CacheKey" .elai_cc/crates/orchestrator/src/oracle_cache.rs

# 3. DKG tools in catalog
rg "dkg_callers|dkg_dependents|dkg_data_flow|dkg_subgraph" .elai_cc/crates/orchestrator/src/production_agent_rpc.rs

# 4. vram_tiers TOML section with >=5 entries
python -c "
import tomllib, sys
with open('.elai_cc/elai_cc.toml','rb') as f: d=tomllib.load(f)
tiers = d.get('research',{}).get('vram_tiers',{})
assert len(tiers) >= 5, f'got {len(tiers)}'
print('OK:', list(tiers.keys()))
"

# 5. surgical_plan_created SSB action_type registered
rg "surgical_plan_created" .elai_cc/crates/orchestrator/src/

# 6. oracle_cache_hit / oracle_cache_miss SSB action_types
rg "oracle_cache_hit|oracle_cache_miss" .elai_cc/crates/orchestrator/src/

# 7. Contract tests compile and pass
cargo test -p elai-orchestrator --test c30_surgical_oracle_contract 2>&1 | grep "test result"
# Expected: test result: ok. 17 passed

# 8. surgical dispatch wired in task_runner
rg "SurgicalDecomposer|surgical_decomposer" .elai_cc/crates/orchestrator/src/task_runner.rs

# 9. project_detection_signal SSB type
rg "project_detection_signal" .elai_cc/crates/orchestrator/src/
```

## ACCEPTANCE

- [ ] SurgicalPlan, AffectedSet, ComplexityClass, RegressionGate, TestBaseline
      all compile; no .unwrap() in production paths
- [ ] SurgicalDecomposer::decompose() performs real DKG blast-radius traversal
      (DkgQuerierAdapter::new + query_reachability called)
- [ ] Complexity classification correct for all three tiers (contract tests pass)
- [ ] RegressionGate snapshot_hash is blake3-deterministic across identical
      must_pass vecs (contract test oracle_cache_* pass)
- [ ] surgical_loc_warning SSB row emitted when blast_radius_loc > 500
- [ ] User confirmation prompt fires in INTERACTIVE mode for blast > 500 LOC
      before SurgicalPlan is returned to caller
- [ ] Autonomous mode escalates at blast > 1000 LOC
- [ ] task_runner wires parse_task_flags surgical=true -> SurgicalDecomposer;
      existing ConceptDecomposition path untouched for surgical=false
- [ ] OracleAnswerCache TTL: T1=30d, T2=7d, T3=1d enforced; T4 never cached
- [ ] OracleAnswerCache LRU eviction fires at max_entries
- [ ] Oracle cache wired into /research live_cli entry (cache miss first call,
      cache hit second call on identical query in same session)
- [ ] oracle_cache_hit SSB row includes cached_age_secs and source_tier
- [ ] dkg_callers, dkg_dependents, dkg_data_flow, dkg_subgraph present in
      build_tool_catalog with correct input schemas
- [ ] DKG tool handlers open DkgQuerierAdapter from session project_root; return
      error JSON on DkgError::QueryFailed (no panic)
- [ ] [research.vram_tiers] TOML section has >=5 entries; DETERMINISTIC_CHECK
      python probe passes
- [ ] Project detection signal scan fires before first /task on unindexed dir;
      REQUIRED signals (Cargo.toml, go.mod, pyproject.toml) detected correctly
- [ ] project_detection_signal SSB rows emitted with signal_name + fired fields
- [ ] All 17 contract tests in c30_surgical_oracle_contract.rs pass
- [ ] Cloud smoke test: /task --surgical blast<=3 files -> SurgicalPlan
      complexity_class=Simple/Medium in SSB; RegressionGate passes post-fix
- [ ] Cloud smoke test: /research x2 -> oracle_cache_miss then oracle_cache_hit
- [ ] No regression on existing contract tests (cargo check --workspace clean)

## Files touched

- ADD: .elai_cc/crates/orchestrator/src/surgical_decomposer.rs
- ADD: .elai_cc/crates/orchestrator/src/oracle_cache.rs
- MODIFY: .elai_cc/crates/orchestrator/src/task_runner.rs (surgical dispatch)
- MODIFY: .elai_cc/crates/orchestrator/src/research_tree.rs (SurgicalPlan SSB emit)
- MODIFY: .elai_cc/crates/orchestrator/src/production_agent_rpc.rs (4 DKG tools)
- MODIFY: .elai_cc/crates/cli/src/live_cli.rs (oracle cache wire + project detect)
- MODIFY: .elai_cc/elai_cc.toml ([research.vram_tiers] section)
- ADD: .elai_cc/crates/contracts/tests/c30_surgical_oracle_contract.rs
- ADD: workflow/reasoning/C-30-decisions.md

## Next

C-31 or next outstanding C-N gap per bn_status_report.py output.
