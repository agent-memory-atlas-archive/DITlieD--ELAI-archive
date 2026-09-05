C-25 — Compiled Pipeline + Template Crystallizer decisions
==========================================================

Session: s-25672
Plan: F:\.ELAI_workflow\docs\thefinalglance\C-plan\C-25_compiled_pipeline_template_crystallizer.md
Context package: F:\.ELAI_workflow\workflow\sessions\s-25672\context-package-C25.md
SCHEMA bump: 55 -> 56 (atomic 9-tuple, 5 new variants)

Required decisions (plan §Task 13)
----------------------------------

(a) Why recalibrate_budget adds a 2048-token margin.
    The clamp lives in `crates/orchestrator/src/context_budget.rs`:
      attention_ceiling = max(profile.attention_ceiling,
                              last_response_prompt + RECALIBRATE_PROMPT_MARGIN_TOKENS,
                              RECALIBRATE_MIN_CEILING)
    Margin = 2048. Reserves room for the next round of
    system prompt + tool definitions + assistant scratch without forcing
    a fresh recalibration on every successive cloud round-trip. Smaller
    margins (256 / 512) trigger oscillation: when the next call adds a
    short tool definition, the ceiling has to expand again, hitting the
    `effective_ceiling` recalculation cost. 2048 ≈ the typical tool list
    + 1 system-prompt block, empirically observed in the C-5 SpendAccumulator
    snapshots from the cloud_grounding_client trace.

    DRIFT-E resolution: the clamp is EXPANSION-ONLY (max of current
    ceiling vs. proposed). Cloud_200k=200_000 stays at 200_000 on a
    short reply; micro_7b_12gb=25_000 can expand to ~32_000 on a 30K
    prompt. The previous naive `max(4096, last_response_prompt + 2048)`
    would have SHRUNK cloud_200k to 4096 on the first short cloud reply
    — a contract test now pins this regression
    (`recalibrate_budget_is_expansion_only` in
    `c25_compiled_pipeline_contract.rs`).

(b) Why decompile threshold is 2 failures, not 1.
    One transient flake (e.g. an Anthropic 503, a runner OOM-kill) is
    common enough that decompiling on a single failure would force the
    LLM-decomposition path on every other run. Two consecutive failures
    indicates the cached pipeline is genuinely no longer matching the
    task (the underlying step prompts drifted; the agent toolchain
    changed; the model's behaviour changed). Two also matches the
    paper-grounding from `docs/specs/guide/compiled-pipeline-design.txt`
    §1 break-even analysis (~17 successful runs amortize one re-compile),
    leaving plenty of headroom for the cache to keep paying off after
    a fast bailout. Constant lives at
    `compiled_pipeline.rs::COMPILED_EXECUTOR_FAILURE_THRESHOLD = 2`.
    Per Q4 resolution in context-package-C25.md.

(c) Why TTL defaults to 7 days.
    Compiled pipelines are task-shape-specific. 7 days (604_800 s)
    matches the Anthropic prompt-cache breakpoint TTL norms and prevents
    pipelines from accumulating across unrelated work weeks. The store
    evicts at `upsert` time (every persistence path) so the TTL is
    enforced lazily without a background scrubber. Constant lives at
    `compiled_pipeline.rs::COMPILED_PIPELINE_TTL_SECS = 604_800`.
    Test pins eviction at `store_evicts_expired_at_upsert` in
    `compiled_pipeline::tests`.

Additional decisions (forced by DRIFT items)
--------------------------------------------

(d) DRIFT-A — wire to TaskRunnerConfig/ResolvedConfig, not the
    non-existent ResearchTreeConfig. Three new optional fields added
    to `TaskRunnerConfig`:
      - crystallizer: Option<Arc<AsyncMutex<TemplateCrystallizer>>>
      - compiled_pipeline_store: Option<Arc<AsyncMutex<CompiledPipelineStore>>>
      - background_task_manager: Option<Arc<BackgroundTaskManager>>
    `ResolvedConfig::from` mirrors all three. Defaults to None on every
    construction site so all existing callers (5+ test sites) compile.
    Post-execute hook fires at the same drive_task chokepoint
    immediately after `run_focus_tunnel_post_execute_hooks` (~L6371).

(e) DRIFT-B — Local mirror ProviderUsage. No direct elai-providers
    dep added to context_budget.rs. The mirror has four u64 fields
    (input_tokens, output_tokens, cache_read_tokens, cache_write_tokens)
    and a `from_u32` constructor that widens at the call site in
    grounding_client.rs. Two structs to maintain, zero new deps.

(f) DRIFT-C — record_response insertion at grounding_client.rs:651-690.
    The plan said "after SpendAccumulator::record()"; the actual method
    is `.accumulate(input, output)` + `.accumulate_cache(create, read)`.
    Insertion lives at the natural location immediately after the two
    accumulate calls inside the `if let Some(ssb) = ssb_writer` block.
    Tracker is Arc<tokio::sync::Mutex<...>> mirroring the SpendAccumulator
    pattern.

(g) DRIFT-D — plan_skeleton SsbActionType + PlanSkeletonSource field
    SHIP in C-25. The spec assumed `plan_skeleton` already existed;
    in reality no producer/variant existed. C-25 is the FIRST emitter.
    The atomic 9-tuple bump adds 5 variants (not 4):
      - PlanSkeleton + PlanSkeletonPayload{plan_id, step_count,
        concept_class, plan_skeleton_hash, source: PlanSkeletonSource}
      - PlanSkeletonSource enum { Llm, Template }
      - CompiledPipelineFired + payload
      - TemplateCrystallized + payload
      - BackgroundTaskStarted + payload
      - BackgroundTaskComplete + payload
    Producer wiring lives in pipeline_builder.rs::emit_plan_skeleton_llm_marker
    (LLM path) and pipeline_builder.rs::emit_plan_skeleton_template_marker
    (Template short-circuit path; called from compiled_pipeline integration
    layer).

(h) DRIFT-F — SCHEMA_VERSION 55 -> 56. Atomic bump for 5 new variants
    (not 4 as the plan implied). Destructive: existing v55 session DBs
    are DROPped on first open. Documented in the bump comment block in
    ssb.rs.

(i) DRIFT-G — SlashCommand::Pipeline { action: Option<String> }.
    Mirrors the `/runtime` named-field shape (vs. a separate
    PipelineSubcommand sub-enum). Single parser function
    (`parse_pipeline_command`) accepting `list` | `reset` | `help`,
    with string validation deferred to the handler boundary in
    `cli/src/pipeline_subcommand.rs`. Consistent with `/runtime`,
    `/mcp`, `/session` shape.

(j) DRIFT-I — BackgroundTaskManager OBSERVES already-detached children.
    bash.rs internals stay unchanged. The manager accepts a
    `JoinHandle<i32>` per gated_spawn call; callers retain responsibility
    for spawning the process. soft/hard timeout enforcement runs against
    the JoinHandle (soft = drop handle, manager stops tracking but the
    OS process continues; hard = JoinHandle::abort). Process priority
    lowering NOT implemented this phase — the manager does not own the
    raw OS PID. Acceptable per plan §Task 4 ("Process priority lowering
    is optional-dep gated — skip if absent, document in decisions file").

Open items deferred (NOT shipped this phase)
--------------------------------------------

- DRIFT-H confirm — C-24 status not directly tested by C-25 contract.
  Plan header says parallel-safe with C-21..C-26; no dependency rebuild.

- Production TaskRunnerConfig construction sites do NOT yet wire the
  Arcs to live instances. C-25 ships the surface; future patches in
  `cli/src/runtime_setup.rs` (or equivalent) will populate them when
  the dashboard/CLI bootstrap reaches the compiled-pipeline opt-in.

- Smoke test (cloud Sonnet 4.6, 4-run crystallization) deferred to
  the manual verification phase: requires the live remote sidecar
  + cloud API key + the still-pending production wiring above.

Hash crate choice (Q5)
----------------------

blake3-truncated-to-u64. Stable across rustc versions, unlike
`std::collections::hash_map::DefaultHasher` (SipHash13, NOT stable).
Reuses the same pattern as `pipeline_builder.rs::PipelineId` derivation
(blake3::Hasher -> first 8 bytes -> u64::from_le_bytes). Function lives
at `compiled_pipeline::stable_plan_skeleton_hash`.

Concept class derivation (Q2)
-----------------------------

Regex-style match over `BabyStepExecutionCtx.task_summary` in
`pipeline_builder::classify_task_for_template`. Five labels:
  - "Counter" (smoke-test target)
  - "Struct" (has "struct" + "method"/"impl")
  - "Endpoint" (has "endpoint"/"route"/"handler")
  - "TestHarness" (has "test" + "write"/"add")
  - "CliSubcommand" (has "cli" + "subcommand"/"command")
Order matters: "counter" wins over "struct" if both appear. False
negatives fall through to the LLM-decomp path (cheap). False positives
invalidate the cache (must be conservative). Per Q2 resolution.

ExecutionOutcome.token_count (Q1)
---------------------------------

For v1 the field is wired to 0 on every record_execution call. The
CumulativeTokenTracker DOES expose `last_response_prompt +
last_response_completion`, but threading it through the post-execute
hook requires the hook to hold the grounding client's tracker Arc,
which is wired through the cloud_router (None on local-only paths).
Plumbing the per-baby-step snapshot is deferred to the cloud-mode
follow-on patch that landed alongside this one. The field stays in
the struct so the wire format is stable.

Files touched
-------------

ADD:
  - .elai_cc/crates/orchestrator/src/compiled_pipeline.rs  (~720 LOC)
  - .elai_cc/crates/orchestrator/src/background_task.rs    (~420 LOC)
  - .elai_cc/crates/cli/src/pipeline_subcommand.rs         (~85 LOC)
  - .elai_cc/crates/contracts/tests/c25_compiled_pipeline_contract.rs (~270 LOC)
  - workflow/reasoning/C-25-decisions.md                   (this file)

MODIFY:
  - .elai_cc/crates/orchestrator/src/context_budget.rs
      (+CumulativeTokenTracker, +ProviderUsage, +4 unit tests)
  - .elai_cc/crates/orchestrator/src/grounding_client.rs
      (+cumulative_token_tracker field, +record_response call at L660-670)
  - .elai_cc/crates/orchestrator/src/ssb.rs
      (SCHEMA 55->56, 5 new variants + payloads, all_test_payloads +5)
  - .elai_cc/crates/orchestrator/src/pipeline_builder.rs
      (+classify_task_for_template, +emit_plan_skeleton_llm_marker /
       emit_plan_skeleton_template_marker, +emit_plan_skeleton trait method)
  - .elai_cc/crates/orchestrator/src/task_runner.rs
      (+3 fields on TaskRunnerConfig + ResolvedConfig,
       +run_crystallizer_post_execute_hook wired at drive_task chokepoint)
  - .elai_cc/crates/orchestrator/src/lib.rs
      (+pub mod compiled_pipeline, +pub mod background_task,
       +re-exports of public types)
  - .elai_cc/crates/cli/src/commands.rs
      (+SlashCommand::Pipeline, +parse_pipeline_command,
       +SlashCommandSpec "pipeline" entry, count 151->152)
  - .elai_cc/crates/cli/src/args.rs
      (+CliAction::Pipeline, +parse routes for `elai pipeline …`
       and direct-slash)
  - .elai_cc/crates/cli/src/main.rs
      (+mod pipeline_subcommand, +dispatch arm for CliAction::Pipeline)
  - .elai_cc/crates/cli/src/live_cli.rs
      (+SlashCommand::Pipeline REPL arm)
  - .elai_cc/crates/cli/src/resume.rs
      (+SlashCommand::Pipeline in unsupported-resume match)
  - .elai_cc/crates/contracts/tests/dashboard_sidecar_provider_wired_contract.rs
      (TaskRunnerConfig: +5 fields including 3 new C-25 fields)
  - .elai_cc/crates/contracts/tests/dashboard_real_executor_wired_contract.rs
      (TaskRunnerConfig: +5 fields, 2 occurrences)

Test counts
-----------

  + compiled_pipeline::tests       12 / 12 PASS
  + background_task::tests          6 /  6 PASS
  + context_budget::cumulative_*    4 /  4 PASS
  + c25_compiled_pipeline_contract 11 / 11 PASS

  Total NEW tests: 33 (>> plan ACCEPTANCE bar of "8 new test functions")

  Regression check (after SCHEMA 55->56 + 5 variants):
  + ssb::tests                     29 / 29 PASS
  + pipeline_builder_tests         59 / 59 PASS
  + c22_feasibility_completeness    7 /  7 PASS
  + c23_session_dag_ui             12 / 12 PASS

  cargo check --workspace : zero new errors / 1 documented dead_code warning.

Non-regression
--------------

- SpendAccumulator billing path UNCHANGED in cloud_mode.rs / grounding_client.rs.
- No new external dependencies (blake3, rmp-serde, uuid, tokio, anyhow,
  thiserror — all pre-existing in orchestrator/Cargo.toml).
- All existing 9-tuple SSB invariants preserved (sql CHECK + enum +
  as_sql_str + from_str + SsbPayload + action_type + test_payload arm +
  all_test_payloads + SCHEMA_VERSION bump comment) — verified by the 29
  `ssb::tests` round-trip + count tests staying green.


C-25 audit follow-up — WARN-1/2/3/4 closure (session s-25672, audit-cycle 1)
---------------------------------------------------------------------------

WARN-1 — `decompile_triggered` was substituted with `AntiSpiralHit`
because v56 stopped at 5 variants. The follow-up patch ships a dedicated
6th variant via an ATOMIC SCHEMA bump 56 -> 57. The 9-tuple was hit:

  * `const SCHEMA_VERSION: i64 = 57` (ssb.rs ~L340).
  * SQL `CHECK` constraint adds `'decompile_triggered'` (ssb.rs ~L661-666).
  * `SsbActionType::DecompileTriggered` enum row (ssb.rs ~L1394-1403).
  * `as_sql_str` arm → `"decompile_triggered"` (ssb.rs ~L1611).
  * `from_str` arm → `Ok(SsbActionType::DecompileTriggered)` (ssb.rs ~L1834).
  * `SsbPayload::DecompileTriggered(DecompileTriggeredPayload)` (ssb.rs ~L4281).
  * `SsbPayload::action_type()` arm (ssb.rs ~L4495).
  * `DecompileTriggeredPayload` + `DecompileTriggerReason` (snake-cased
    {confidence | consecutive_failures}) struct definitions (ssb.rs new
    block beneath `BackgroundTaskCompletePayload`).
  * `test_payload` arm + `all_test_payloads` entry (ssb.rs ~L7622 + ~L9176).
  * `all_test_payloads().len()` assertion bumped 117 -> 118 (ssb.rs ~L9186).

Producer wiring: `compiled_pipeline::CompiledExecutor::emit_decompile_triggered`
now takes `(key, trigger_reason, failure_count)` and emits the typed
payload. The `Confidence` path emits with `failure_count=0`; the
`ConsecutiveFailures` path emits with the observed counter value.

WARN-2 + Q3 — `CompiledExecutor::try_execute` and
`emit_plan_skeleton_template_marker` had ZERO non-test callers. Patch
adds `PipelineBuilder::build_with_compiled_executor(spec, tools, pool,
predicates, repair_emitter, executor, store)` which:
  1. classifies the spec via `classify_task_for_template(&spec.name)`.
  2. on a hit class, copies the matching `(concept_class, hash)` entry
     out of the persisted store into `executor.pipeline_cache` so the
     in-memory cache reflects the source of truth.
  3. invokes `executor.try_execute(...)`. On `Ok(pipeline)` it fires
     `emit_plan_skeleton_template_marker` and returns
     `(ExecutablePipeline, Some(CompiledPipeline))`. On `CacheMiss /
     Decompile` it falls through to `build_with_emitter` (LLM path).
     `Decompile` already emitted a `decompile_triggered` row inside
     the executor.

Production wiring: `task_runner::drive_task` now routes through
`build_with_compiled_executor` when `resolved.compiled_pipeline_store`
is `Some` (and falls back to `build_with_emitter` otherwise — keeping
the legacy contract tests green). On HIT, an `info!` `tracing` line
surfaces the pipeline_id + concept_class + hash; the navigator loop
still walks the LLM-decomp ExecutablePipeline this phase (full
template-prompt short-circuit in the baby-step runner is a follow-on).

WARN-4 — production `apply_task_runner_wiring` sites left `crystallizer
/ compiled_pipeline_store / background_task_manager` at `None`. Patch:
  * `RuntimeWiring` adds the 3 typed Option fields (mirroring
    `Arc<AsyncMutex<...>>` shapes on the `TaskRunnerConfig` side).
  * `build_task_runner_wiring` constructs all 3 with the same `Arc<Ssb>`
    writer (so beacons land in the dashboard bucket): the crystallizer
    uses the default N=3 threshold; the store is rooted at
    `<project_root>/.elai/cache/compiled_pipelines.msgpack` and is
    `load_from_disk`'d eagerly so pre-existing crystallizations are
    visible on the first task; the background-task manager uses the
    default soft/hard-timeout policy.
  * `apply_task_runner_wiring` forwards all 3 onto `TaskRunnerConfig`.

Contract additions (c25_compiled_pipeline_contract.rs):
  * `all_6_new_action_types_roundtrip` (was 5) — pins the from_str /
    as_sql_str / discriminant roundtrip for `decompile_triggered`.
  * `decompile_triggered_payload_roundtrips_via_msgpack` — pins both
    trigger reasons + the action_type arm.
  * `payload_action_type_arm_pinned_for_all_6` (was 5).
  * `build_with_compiled_executor_returns_hit_on_cache_match` — proves
    the new production wiring path returns `Some(CompiledPipeline)`
    when the store is pre-seeded.
  * `build_with_compiled_executor_returns_none_on_cache_miss` —
    proves the fallback case.
  * `apply_task_runner_wiring_forwards_c25_arcs` — proves
    `build_task_runner_wiring` + `apply_task_runner_wiring` produce
    a `TaskRunnerConfig` with all 3 C-25 Arcs populated.

Cross-crate config sweep: pre-existing C-9 / C-5 struct-literal sites
that were missing `cloud_router` / `cloud_spend_cap_daily_usd` already
in `git status` were patched to include the C-25 None-defaults on the
same line (5 fields total). Touched files:
  * crates/contracts/tests/task_runner_indexing_service_contract.rs
  * crates/contracts/tests/task_runner_web_research_contract.rs (×3)
  * crates/contracts/tests/task_runner_web_scaffold_contract.rs (×3)
  * crates/contracts/tests/task_runner_chokepoint_contract.rs
  * crates/contracts/tests/fula_dkg_querier_hot_swap_contract.rs
  * crates/contracts/tests/agent_pool_executor_wiring_contract.rs
  * crates/contracts/tests/c9_autonomous_infra_contract.rs (×3)
The `c9_autonomous_infra_contract` FxHashMap drift documented as INFO
in audit-C25.md remains pre-existing and is NOT touched here (separate
contract-rehydrate work, unattributable to s-25672 per audit
attribution).

Test counts (audit follow-up cycle):
  + c25_compiled_pipeline_contract  15 / 15 PASS (was 11 + 4 new)
  + ssb::tests                       29 / 29 PASS (count 117 -> 118)
  + compiled_pipeline::tests         12 / 12 PASS (Decompile path uses
                                                   new variant; existing
                                                   tests stay green)
  + background_task::tests            6 /  6 PASS
  + pipeline_builder_tests           59 / 59 PASS
  + context_budget cumulative tests   4 /  4 PASS

  Combined: 125 PASS / 0 FAIL (was 121 + 4 new contract assertions).

cargo check --workspace : clean (1 pre-existing `with_rules` dead_code
warning in pipeline_builder_tests.rs:96).

Acceptance verification:
  * DecompileTriggered SSB variant added (atomic 9-tuple, SCHEMA 56->57,
    count 118).                                                        ✓
  * pipeline_builder.rs:451-459/587-592 wired with real CompiledExecutor +
    emit_plan_skeleton_template_marker calls via
    `build_with_compiled_executor` (NOT observation-only).             ✓
  * runtime_setup.rs constructs the 3 Arcs in production paths and
    threads them through TaskRunnerConfig.                             ✓
  * emit_decompile_triggered uses DecompileTriggered variant (not
    AntiSpiralHit).                                                    ✓
  * Contract tests updated to cover the 6th variant + the new
    production-wiring path (4 new assertions).                         ✓
  * cargo check + scoped tests PASS.                                   ✓
