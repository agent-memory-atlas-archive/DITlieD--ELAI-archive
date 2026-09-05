# C-25: Compiled Pipeline + Template Crystallizer

Objective: implement Designs 1-3 from compiled-pipeline-design.txt —
CompiledPipeline/TemplateCrystallizer/CompiledExecutor (Design 1),
BackgroundTaskManager with soft-timeout auto-detach and two new SSB types
(Design 2), CumulativeTokenTracker wired into cloud response parsing with
prompt_cache_read_tokens forwarded to context_budget (Design 3). Persist
compiled entries to .elai/cache/compiled_pipelines.msgpack with TTL eviction.
Add /pipeline list and /pipeline reset CLI commands.
Estimated effort: 2-3 sessions.
Pre-requisites: C-9 (autonomous gate — crystallization observation loop drives
N=3 consecutive runs; gate_decision SSB rows required for smoke test).
Parallel-safe with: C-21, C-22, C-23, C-24, C-26.

## Context

compiled-pipeline-design.report.md §Mechanisms MISSING (2026-05-17) confirms:
CompiledPipeline, TemplateCrystallizer, CompiledExecutor — no evidence for
any symbol (report:40-45). BackgroundTaskManager absent; background_task_id
exists in BashCommandOutput (bash.rs:57) but no manager/detach/monitoring;
BACKGROUND_TASK_STARTED and BACKGROUND_TASK_COMPLETE SSB types absent
(report:45-46). CumulativeTokenTracker absent; SpendAccumulator (cloud_mode.rs:302)
accumulates for billing only, no feedback to context_budget (report:27-28).

post-phase4-implementation-order.report.md §MISSING confirms G-1..G-4 all
absent (report:79-84).

Partial foundation: SpendAccumulator at cloud_mode.rs:302; BashCommandOutput.
background_task_id at bash.rs:57; context_budget.rs profile allocator at
570-2824 (elai.map); pipeline_builder.rs holds factory selection insertion point.

## Reads

- F:\.ELAI_workflow\docs\thefinalglance\verification\reports\guide\compiled-pipeline-design.report.md
- F:\.ELAI_workflow\docs\thefinalglance\verification\reports\guide\post-phase4-implementation-order.report.md (G-1..G-4 rows)
- .elai_cc/crates/orchestrator/src/cloud_mode.rs (SpendAccumulator:302-394)
- .elai_cc/crates/orchestrator/src/bash.rs (BashCommandOutput:57-62)
- .elai_cc/crates/orchestrator/src/context_budget.rs (ModelCapabilityProfile:395-440)
- .elai_cc/crates/orchestrator/src/pipeline_builder.rs (factory selection — confirm insertion point)
- .elai_cc/crates/orchestrator/src/ssb.rs (SsbActionType — confirm BACKGROUND_TASK_* absent)
- .elai_cc/crates/providers/src/providers/openai_compat.rs (:363 usage field names)
- .elai_cc/crates/cli/src/commands.rs (confirm /pipeline absent)

## Tasks

1. **CumulativeTokenTracker struct** appended to context_budget.rs after
   ModelCapabilityProfile. Fields: prompt_tokens_total, completion_tokens_total,
   cache_read_tokens, cache_write_tokens, last_response_prompt,
   last_response_completion (all u64). Methods: record_response(&ProviderUsage);
   recalibrate_budget(&mut ModelCapabilityProfile) sets attention_ceiling to
   max(4096, last_response_prompt + 2048). Derive Debug/Clone/Default/Serde.
   Verify at openai_compat.rs:363 whether a ProviderUsage named type already
   exists before adding one (input_tokens, output_tokens, cache_read_tokens,
   cache_write_tokens: u64).

2. **Wire CumulativeTokenTracker into cloud_mode.rs** alongside SpendAccumulator.
   After each SpendAccumulator::record() call, invoke tracker.record_response().
   Arc<Mutex<CumulativeTokenTracker>> shared so recalibrate_budget() can be
   called before each outbound request. SpendAccumulator billing path unchanged.

3. **SSB types for background tasks**. Add to ssb.rs SsbActionType:
   BackgroundTaskStarted { task_id, command_summary, soft_timeout_secs,
   hard_timeout_secs } and BackgroundTaskComplete { task_id, exit_status,
   duration_secs, auto_detached, timed_out }. Wire SsbPayload variants.

4. **BackgroundTaskManager** in crates/orchestrator/src/background_task.rs.
   active_tasks: HashMap<String, BackgroundTaskHandle>, max_concurrent: u8=3,
   ssb_tx, poll_interval_secs: u32. BackgroundTaskHandle fields: task_id,
   spawned_at: Instant, soft_timeout, hard_timeout, join_handle: JoinHandle<i32>,
   auto_detached. Methods: spawn_task() emits BackgroundTaskStarted and returns
   task_id; poll_completed() harvests finished handles, emits BackgroundTaskComplete;
   enforce_timeouts() detaches (drop handle, auto_detached=true) at soft boundary,
   aborts at hard boundary; gated_spawn() rejects when autonomous_mode=false.
   Adaptive poll: 5s -> 10s -> 30s (doubles each idle cycle). Process priority
   lowering is optional-dep gated — verify nix/winapi availability before adding;
   skip if absent, document in decisions file.

5. **Core pipeline types** in crates/orchestrator/src/compiled_pipeline.rs.
   TemplateVariable { name, region_start, region_end, variable_type: VariableType }.
   VariableType enum { ConceptClass, TaskDescription, CodeTarget, Other(String) }.
   CompiledStep { step_index: u8, template_prompt, variables, variable_region_ratio:
   f32, l1_tool_only: bool }.
   CompiledPipeline { pipeline_id: Uuid, concept_class, plan_skeleton_hash: u64,
   steps, created_at, last_used_at, use_count: u32, first_try_success_rate: f32,
   compilation_eligible: bool }. compilation_eligible = l1_tool_only on all steps
   AND max variable_region_ratio < 0.3 AND use_count >= 3.
   TemplateId(Uuid) and PipelineOutcomeId(Uuid) newtypes.
   Derive Debug/Clone/Serialize/Deserialize; use rmp-serde for msgpack.
   Verify uuid + rmp-serde in Cargo.toml before use.

6. **TemplateCrystallizer** in compiled_pipeline.rs. observation_log:
   HashMap<(String, u64), Vec<ExecutionOutcome>>, threshold: u32=3, ssb_tx.
   ExecutionOutcome { success, concept_class, plan_skeleton_hash, pipeline_id:
   Option<Uuid>, token_count, timestamp }. record_execution() appends to bucket;
   check_crystallization() returns Some(CompiledPipeline) when last N consecutive
   outcomes are all success=true AND compilation_eligible; emits SSB row
   template_crystallized with TemplateCrystallizedPayload { concept_class,
   plan_skeleton_hash, pipeline_id, threshold_n }.

7. **CompiledExecutor** in compiled_pipeline.rs. pipeline_cache: Arc<RwLock<
   HashMap<(String, u64), CompiledPipeline>>>, consecutive_failures: HashMap<
   (String, u64), u8>, confidence_threshold: f32=0.7, ssb_tx.
   try_execute() looks up cache; returns Err(CacheMiss) on absent key; on 2
   consecutive failures OR confidence < 0.7, emits decompile_triggered SSB row
   and returns Err(Decompile). decompile_and_fallback() logs event; caller in
   pipeline_builder.rs routes to LLM decomp on Decompile.

8. **CompiledPipelineStore** in compiled_pipeline.rs. cache_path: PathBuf,
   entries: HashMap<(String, u64), CompiledPipeline>, ttl_seconds: u64=604800.
   load_from_disk(): rmp-serde decode; corrupt input -> tracing::warn! + empty
   store, never panic. persist_to_disk(): atomic tempfile + rename.
   evict_expired(): drop entries where now - last_used_at > ttl.
   upsert(): insert/update then evict. Create .elai/cache/ if absent.

9. **Wire CompiledExecutor into pipeline_builder.rs**. At factory selection:
   classify_task_for_template(&task) -> Option<(String, u64)> derives concept_class
   heuristic and plan_skeleton_hash (seahash if in Cargo.toml, else DefaultHasher
   with doc note). If Some, call executor.try_execute(); Ok => emit plan_skeleton
   SSB row with source=Template; Decompile => fall through to LLM path.
   Add PlanSkeletonSource { Template, Llm } to PlanSkeletonPayload in ssb.rs.
   Emit source=Llm on the existing LLM decomp path.

10. **Wire TemplateCrystallizer into ResearchTree** post-step hook.
    Add crystallizer: Option<Arc<Mutex<TemplateCrystallizer>>> and store:
    Option<Arc<Mutex<CompiledPipelineStore>>> to ResearchTreeConfig. After each
    baby step completes, call record_execution() + check_crystallization(); on
    Some, call store.upsert() + store.persist_to_disk().

11. **Add /pipeline CLI commands** in commands.rs. SlashCommand::Pipeline(
    PipelineSubcommand { List, Reset }). List reads CompiledPipelineStore and
    prints concept_class, plan_skeleton_hash, use_count, first_try_success_rate,
    created_at per entry. Reset clears store and deletes msgpack file.

12. **Smoke test** (cloud Sonnet 4.6, 4-run crystallization). Run task 3 times;
    assert runs 1-3 emit plan_skeleton source=Llm. After run 3, assert
    template_crystallized SSB row (concept_class contains "Code" or "Counter",
    threshold_n=3). Run 4: assert plan_skeleton source=Template; token_count
    lower than mean of runs 1-3; cache_read_tokens > 0 in CloudSpendPayload.
    Background test: long-running bash search in autonomous mode; assert
    BackgroundTaskStarted SSB row, soft-timeout fires, BackgroundTaskComplete
    with auto_detached=true.

13. **Decision doc** workflow/reasoning/C-25-decisions.md: (a) why
    recalibrate_budget adds 2048 margin; (b) why decompile threshold is 2
    failures not 1; (c) why TTL defaults to 7 days.

## WIRING_CONTRACT

produces_symbols:
  - CumulativeTokenTracker (record_response, recalibrate_budget) + ProviderUsage
  - SsbActionType::BackgroundTaskStarted + ::BackgroundTaskComplete + payloads
  - BackgroundTaskManager + BackgroundTaskHandle + BackgroundTaskError
  - CompiledPipeline + CompiledStep + TemplateVariable + VariableType
  - TemplateId + PipelineOutcomeId newtypes
  - TemplateCrystallizer + ExecutionOutcome + TemplateCrystallizedPayload
  - CompiledExecutor + CompiledExecutorError
  - CompiledPipelineStore (load_from_disk, persist_to_disk, evict_expired, upsert)
  - PlanSkeletonSource { Template, Llm } + PlanSkeletonPayload.source field
  - classify_task_for_template() in pipeline_builder.rs
  - SlashCommand::Pipeline(PipelineSubcommand) + PipelineSubcommand { List, Reset }
  - SSB action types: template_crystallized, decompile_triggered
consumes_symbols:
  - SpendAccumulator (cloud_mode.rs:302) — unchanged billing path
  - openai_compat.rs:363 usage parsing — cache_read_tokens field read
  - ModelCapabilityProfile (context_budget.rs:395) — gains recalibrate path
  - BashCommandOutput.background_task_id (bash.rs:57) — read by manager
  - BashSandbox — spawn_task delegates subprocess creation
  - SsbActionType enum (ssb.rs) — extended with BackgroundTask* variants
  - ResearchTreeConfig — gains crystallizer + store handles
  - SlashCommand enum (commands.rs) — gains Pipeline variant
  - pipeline_builder.rs factory selection — try-template-first block inserted
reachable_from:
  - cloud_mode.rs round-trip loop (CumulativeTokenTracker)
  - ResearchTree post-step callback (TemplateCrystallizer)
  - pipeline_builder.rs factory entry (CompiledExecutor)
  - BashSandbox autonomous-mode path (BackgroundTaskManager)
  - CLI /pipeline list | /pipeline reset
stub_ban:
  - recalibrate_budget() MUST clamp to max(4096, computed); never 0
  - try_execute() MUST return Err(CacheMiss) when pipeline_cache has no entry
  - gated_spawn() MUST NOT spawn when autonomous_mode=false
  - load_from_disk() MUST NOT panic on corrupt msgpack; return empty store + warn
  - plan_skeleton SSB row MUST include source field; omitting it is a violation
observable_effect:
  trigger: elai /task --mode cloud "implement a 2-method counter struct" x3,
           then a 4th identical invocation
  expect_ssb: runs 1-3: plan_skeleton source=Llm; after run 3:
              template_crystallized with concept_class~="Counter", threshold_n=3;
              run 4: plan_skeleton source=Template; cache_read_tokens > 0
  expect_artifact: .elai/cache/compiled_pipelines.msgpack non-empty;
                   /pipeline list prints >= 1 entry

## DETERMINISTIC_CHECK

```bash
# 1. CumulativeTokenTracker present + clamp guard
rg "struct CumulativeTokenTracker" \
  .elai_cc/crates/orchestrator/src/context_budget.rs
rg "max\(4096" .elai_cc/crates/orchestrator/src/context_budget.rs

# 2. BackgroundTask SSB variants
rg "BackgroundTaskStarted|BackgroundTaskComplete" \
  .elai_cc/crates/orchestrator/src/ssb.rs

# 3. BackgroundTaskManager + gated_spawn + enforce_timeouts
rg "struct BackgroundTaskManager|fn gated_spawn|fn enforce_timeouts" \
  .elai_cc/crates/orchestrator/src/background_task.rs

# 4. Core pipeline types
rg "struct CompiledPipeline|struct TemplateCrystallizer|struct CompiledExecutor" \
  .elai_cc/crates/orchestrator/src/compiled_pipeline.rs

# 5. CompiledPipelineStore + TTL
rg "fn persist_to_disk|fn evict_expired|ttl_seconds" \
  .elai_cc/crates/orchestrator/src/compiled_pipeline.rs

# 6. try-template-first + PlanSkeletonSource in pipeline_builder
rg "try_execute|PlanSkeletonSource|classify_task_for_template" \
  .elai_cc/crates/orchestrator/src/pipeline_builder.rs

# 7. plan_skeleton source field in ssb.rs
rg "PlanSkeletonSource" .elai_cc/crates/orchestrator/src/ssb.rs

# 8. /pipeline CLI command
rg "SlashCommand::Pipeline|PipelineSubcommand" \
  .elai_cc/crates/cli/src/commands.rs

# 9. Build + scoped tests
cargo check --workspace 2>&1 | tail -3
cargo test -p elai_orchestrator -- compiled_pipeline background_task \
  cumulative_token 2>&1 | grep "test result"
cargo test -p elai_cc_contracts -- pipeline 2>&1 | grep "test result"

# 10. Cache file produced after smoke test
powershell -c "Test-Path .elai/cache/compiled_pipelines.msgpack"
```

## ACCEPTANCE

- [ ] CumulativeTokenTracker in context_budget.rs: 6 u64 fields, record_response(),
      recalibrate_budget() with max(4096, last_response_prompt + 2048)
- [ ] CumulativeTokenTracker wired into cloud_mode.rs; SpendAccumulator billing
      path unchanged; cloud_spend SSB contract test passes unmodified
- [ ] BackgroundTaskStarted + BackgroundTaskComplete in SsbActionType; payloads
      serialise/deserialise correctly
- [ ] BackgroundTaskManager: max_concurrent=3; gated_spawn() rejects interactive
      mode; enforce_timeouts() detaches at soft, kills at hard; adaptive poll 5->30s
- [ ] BackgroundTaskStarted SSB row on spawn; BackgroundTaskComplete on finish/detach
- [ ] CompiledPipeline, CompiledStep, TemplateVariable, VariableType in
      compiled_pipeline.rs; compilation_eligible enforces ratio < 0.3 + l1_tool_only
- [ ] TemplateCrystallizer threshold=3; emits template_crystallized SSB after N=3
      consecutive successes; returns CompiledPipeline candidate
- [ ] CompiledExecutor: Err(CacheMiss) on absent key; Decompile after 2 failures or
      confidence < 0.7; decompile_triggered SSB row emitted
- [ ] CompiledPipelineStore: corrupt load -> empty+warn (no panic); atomic persist;
      TTL eviction at upsert; .elai/cache/ created if absent
- [ ] pipeline_builder.rs try-template-first: Decompile falls through cleanly;
      plan_skeleton emits source=Template or source=Llm
- [ ] TemplateCrystallizer wired into ResearchTree post-step hook via ResearchTreeConfig
- [ ] /pipeline list and /pipeline reset commands functional
- [ ] Smoke test runs 1-3: source=Llm; run 3 triggers template_crystallized SSB;
      .elai/cache/compiled_pipelines.msgpack non-empty
- [ ] Smoke test run 4: source=Template; token_count < mean of runs 1-3;
      cache_read_tokens > 0 in CloudSpendPayload (Sonnet prefix cache hit)
- [ ] BackgroundTaskManager smoke: BackgroundTaskStarted row; soft-timeout fires;
      BackgroundTaskComplete with auto_detached=true
- [ ] cargo check --workspace: zero new errors
- [ ] cargo test -p elai_orchestrator -- compiled_pipeline background_task
      cumulative_token: all pass, >= 8 new test functions
- [ ] cargo test -p elai_cc_contracts -- pipeline: all pass
- [ ] /pipeline list shows >= 1 entry after smoke test
- [ ] Decision doc workflow/reasoning/C-25-decisions.md (3 decisions)

## Files touched

- ADD: .elai_cc/crates/orchestrator/src/compiled_pipeline.rs
    (CompiledPipeline, CompiledStep, TemplateVariable, VariableType,
     TemplateCrystallizer, ExecutionOutcome, CompiledExecutor, CompiledExecutorError,
     CompiledPipelineStore, TemplateId, PipelineOutcomeId)
- ADD: .elai_cc/crates/orchestrator/src/background_task.rs
    (BackgroundTaskManager, BackgroundTaskHandle, BackgroundTaskError)
- MODIFY: .elai_cc/crates/orchestrator/src/context_budget.rs
    (CumulativeTokenTracker + ProviderUsage if absent)
- MODIFY: .elai_cc/crates/orchestrator/src/cloud_mode.rs
    (wire CumulativeTokenTracker alongside SpendAccumulator)
- MODIFY: .elai_cc/crates/orchestrator/src/ssb.rs
    (BackgroundTaskStarted, BackgroundTaskComplete, template_crystallized,
     decompile_triggered, PlanSkeletonSource, PlanSkeletonPayload.source)
- MODIFY: .elai_cc/crates/orchestrator/src/pipeline_builder.rs
    (try-template-first block; classify_task_for_template; PlanSkeletonSource emit)
- MODIFY: .elai_cc/crates/orchestrator/src/research_tree.rs
    (crystallizer + store fields in ResearchTreeConfig; post-step hook)
- MODIFY: .elai_cc/crates/cli/src/commands.rs
    (SlashCommand::Pipeline; PipelineSubcommand; list/reset dispatch)
- ADD: workflow/reasoning/C-25-decisions.md

## Next

C-26 or any remaining C-2x depending on prioritization.
If C-9 autonomous gate is not yet shipped, it is the true blocker for the
crystallization smoke test and must be resolved before this session's acceptance
can be fully verified.
