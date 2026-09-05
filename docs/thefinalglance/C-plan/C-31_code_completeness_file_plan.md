# C-31: Code-Completeness Enforcement — Layer 1 FilePlan / FileSpec
Objective: implement the Layer 1 file-planning primitives missing from
code-completeness-enforcement-design: FilePlan/FileSpec/FileModSpec/FunctionSpec
structs, FILE_PLAN as the 9th BabyStepSpec block, FileLineTracker in
TreeNavigator with TOCTOU-safe cumulative LOC accounting, MistakeBook wiring
from T1/T2 BLOCK events, per-baby-step function-count guard (T2 Check 3), and
ProjectIntelligence-derived size limits wired into FileDisciplineConfig.
Estimated effort: 2-3 sessions.
Pre-requisites: C-13 (IPC dispatcher swap; mistake_book trait stable), C-9
(autonomous gate wired).
Parallel-safe with: C-32 (skeleton-first; disjoint file scope).

## Context

code-completeness-enforcement-design.report.md (2026-05-17, status=PARTIAL)
identifies Layer 1 as entirely absent: no FilePlan, FILE_PLAN BabyStepSpec
block, FileLineTracker, or per-baby-step function-count guard found in the
codebase. The FailureCategory enum is ahead of the wiring — LazyCode,
EmptyBody, FileSizeExceeded all exist in crates/core/src/mistake_book.rs
(lines 85-87) but no call site records a MistakeEntry when T1/T2 emits a
BLOCK verdict.

update-implementation-order.report.md (2026-05-17) confirms the same gap
under "DESIGN ADDITIONS TO 4D": FILE_PLAN as 9th BabyStepSpec block, ModuleTree
type, and tier-gated file count limits not found.

BabyStepSpec current shape (research_tree.rs:1667-1703): 8 fields
(id, task, estimated_tokens, grounding, tdd, acceptance, preferred_mode,
target_file, edit_region). FILE_PLAN is the required 9th field: an
Option<FilePlan> carrying per-file write intentions with LOC budgets, so the
orchestrator can enforce size contracts before generation.

Existing wiring available:
- FailureCategory::LazyCode/EmptyBody/FileSizeExceeded in
  crates/core/src/mistake_book.rs:85-87.
- MistakeBook SSB infrastructure (write_mistake_entry, query_by_baby_step) in
  crates/orchestrator/src/mistake_book.rs:55-126.
- LanguageLimits hard-coded defaults in tiered_analysis.rs:85-97 (Rust=400,
  Go=400, Python=480, TS/JS=600, Java/C#=800).
- T1Hook and T2Hook verdict emission path in tiered_analysis.rs, no call to
  mistake_book at BLOCK site.
- TreeNavigatorState (pipeline_tree_navigator.rs:265) tracks failure counters
  but no cumulative LOC field.
- FileDisciplineConfig in tiered_analysis.rs:57 has max_cyclomatic_complexity
  and max_coupling_score; no file_plan field, no PI-derived language limit
  override path.

## Reads

- F:\.ELAI_workflow\docs\thefinalglance\verification\reports\guide\code-completeness-enforcement-design.report.md
- F:\.ELAI_workflow\docs\thefinalglance\verification\reports\guide\update-implementation-order.report.md
- F:\.ELAI_workflow\docs\thefinalglance\verification\reports\steps\phase-4E.report.md
- .elai_cc/crates/core/src/mistake_book.rs (FailureCategory, lines 70-115)
- .elai_cc/crates/orchestrator/src/mistake_book.rs (write_mistake_entry, lines 55-126)
- .elai_cc/crates/orchestrator/src/research_tree.rs (BabyStepSpec, lines 1667-1703)
- .elai_cc/crates/orchestrator/src/pipeline_tree_navigator.rs (TreeNavigatorState,
  lines 265-295; DepthFirstNavigator, lines 371-599)
- .elai_cc/crates/plugins/src/tiered_analysis.rs (FileDisciplineConfig lines 57-113;
  T1Hook BLOCK path lines 400-480; T2Hook BLOCK path lines 596-800;
  LanguageLimits lines 28-97)
- .elai_cc/crates/orchestrator/src/ssb.rs (SsbActionType, MistakeEntryPayload)

## Tasks

1. Define file_plan structs in crates/core/src/file_plan.rs (new file).
   FunctionSpec { name: String, signature: String, max_body_lines: u8 }.
   FileSpec { path: PathBuf, language: String, max_lines: u16,
   functions: Vec<FunctionSpec> }.
   FileModSpec { path: PathBuf, language: String, max_added_lines: u16,
   functions: Vec<FunctionSpec> }.
   FilePlan { new_files: Vec<FileSpec>, modified_files: Vec<FileModSpec>,
   version: u8 }.
   All structs: derive Debug, Clone, Serialize, Deserialize. No .unwrap(); all
   fallible constructors return Result<_, FilePlanError>. FilePlanError: thiserror
   enum with variants LineBudgetExceeded { file: PathBuf, limit: u16, got: u16 }
   and FunctionCountExceeded { file: PathBuf, limit: u8, got: u8 }.

2. Add FILE_PLAN as the 9th BabyStepSpec field in
   crates/orchestrator/src/research_tree.rs.
   Append to BabyStepSpec after edit_region:
     #[serde(default)]
     pub file_plan: Option<crate::file_plan::FilePlan>
   (the crate re-exports FilePlan from elai_core). No schema bump needed:
   #[serde(default)] keeps every existing serialized BabyStepSpec
   deserializable. Update the two BabyStepSpec { ... } construction sites in
   parse_research_tree (lines ~2233 and ~2273) to include file_plan: None so
   they compile.

3. Implement FilePlan parser/round-trip validation in crates/core/src/file_plan.rs.
   fn validate(plan: &FilePlan, limits: &LanguageLimitsMap) -> Result<(), FilePlanError>
   where LanguageLimitsMap = HashMap<String, u16> keyed by language name.
   For each FileSpec: check max_lines <= limits[language] (or global default 400
   if language absent). For each FileModSpec: check max_added_lines <= limits[language].
   Validates function count per file: if functions.len() > MAX_FUNCTIONS_PER_FILE
   (constant = 20), return FunctionCountExceeded. FilePlan serializes/deserializes
   via serde_json round-trip (property: serialized FilePlan is equal to original
   after JSON round-trip).

4. Add FileLineTracker to TreeNavigatorState in
   crates/orchestrator/src/pipeline_tree_navigator.rs.
   New field: cumulative_lines_by_step: HashMap<BabyStepId, u32>.
   Record LOC written after each baby step completes (in record_result call site).
   TOCTOU prevention: before writing a step's lines, acquire a logical lock token
   (u64 monotonic counter field step_write_generation: u64 in TreeNavigatorState;
   increment before write, check unchanged after write; if changed, return
   Err(NavigatorError::WriteGeneration)). Method:
   fn record_lines_written(&mut self, step_id: BabyStepId, lines: u32)
     -> Result<(), NavigatorError>
   Cumulative total accessible via: fn total_lines_written(&self) -> u32 (sum of all values).
   Serialize/Deserialize derived so crash-recovery preserves line counts.

5. Wire MistakeBook recording from T1/T2 BLOCK events into SSB.
   In crates/plugins/src/tiered_analysis.rs, T1Hook::analyze and T2Hook::analyze:
   - After emitting a BLOCK verdict for LazyCode pattern: call the injectable
     MistakeRecorder trait (new trait with single method record_mistake_event,
     default no-op impl). Pass FailureCategory::LazyCode.
   - After emitting a BLOCK verdict for EmptyBody: pass FailureCategory::EmptyBody.
   - After emitting a BLOCK verdict for FileSizeExceeded: pass
     FailureCategory::FileSizeExceeded.
   MistakeRecorder trait: no async; takes baby_step_id: &str + category: FailureCategory
   + file_path: &Path + snippet: &str. Default no-op implementation satisfies the
   existing test suite with zero test changes. Production wiring in task_runner
   provides the real SSB-backed impl.
   Add MistakeRecorder to T1HookConfig and T2HookConfig structs as
   Option<Arc<dyn MistakeRecorder + Send + Sync>> (default None = no-op).

6. Add T2 Check 3 (function-count guard) in
   crates/plugins/src/tiered_analysis.rs.
   T2Hook::analyze accepts Option<&FilePlan> via new field on T2HookConfig:
     pub file_plan: Option<Arc<FilePlan>>
   Check 3 logic (runs after Check 2, before Check 4):
   - If file_plan.is_some() and current file path matches a FileSpec.path:
     count function definitions in the AST for the file's language.
     Rust: count item_function nodes. Python: count function_definition nodes.
     If actual_fn_count > FileSpec.functions.len() + FUNCTION_COUNT_SLACK (=2):
     emit BLOCK verdict with message format
     "function count {actual} exceeds FILE_PLAN limit {limit}+2 in {file}".
   - If file_plan is None: check skipped silently.
   FUNCTION_COUNT_SLACK is a named constant (no magic numbers).

7. Wire ProjectIntelligence-derived size limits into FileDisciplineConfig.
   Add field to FileDisciplineConfig in crates/plugins/src/tiered_analysis.rs:
     pub language_limit_overrides: HashMap<String, LanguageLimits>
   When non-empty, entries override the hard-coded language_limits HashMap for
   matching language keys. Populate from ProjectProfile.language_stats in
   task_runner at the FileDisciplineConfig construction site: for each language
   with detected_file_count >= 5, insert a LanguageLimits entry with
   production_warn = min(spec_default, project_median_file_loc * 1.5 as u16)
   and hard_block = min(spec_hard_block, project_p95_file_loc * 1.2 as u16).
   If ProjectProfile is unavailable, language_limit_overrides is left empty
   (spec defaults hold). No new dependencies.

8. Contract tests for C-31 (TDD: write before implementation).
   File: crates/contracts/tests/c31_file_plan_contract.rs.
   Tests (minimum 16):
   - file_plan_types_compile: FilePlan, FileSpec, FileModSpec, FunctionSpec constructible.
   - file_plan_json_round_trip: serialize then deserialize a FilePlan; assert equality.
   - validate_accepts_within_limits: FileSpec max_lines=100, language=rust -> Ok.
   - validate_rejects_over_limit: FileSpec max_lines=500, language=rust -> Err(LineBudgetExceeded).
   - validate_rejects_over_function_count: functions.len()=25 -> Err(FunctionCountExceeded).
   - baby_step_spec_file_plan_field_present: BabyStepSpec file_plan=None round-trips.
   - baby_step_spec_file_plan_populated: BabyStepSpec file_plan=Some(FilePlan) round-trips.
   - baby_step_spec_legacy_deserializes: JSON without file_plan key -> file_plan=None.
   - file_line_tracker_record_and_total: 3 steps recorded; total equals sum.
   - file_line_tracker_toctou_guard: generation mismatch on second write -> Err.
   - t2_check3_fires_on_excess_functions: file_plan limit=3, AST has 7 -> BLOCK.
   - t2_check3_skipped_when_no_file_plan: file_plan=None -> Check 3 absent.
   - t2_check3_slack_of_two_accepted: 5 spec + 2 slack = 7 actual -> ALLOW.
   - mistake_recorder_lazy_code_block_invoked: T1 BLOCK lazy pattern -> recorder LazyCode.
   - mistake_recorder_file_size_exceeded_block_invoked: T1 oversized -> FileSizeExceeded.
   - language_limit_overrides_applied: override rust=80 wins over default 100 in T1.

## WIRING_CONTRACT

produces_symbols:
  - FilePlan, FileSpec, FileModSpec, FunctionSpec, FilePlanError
    (crates/core/src/file_plan.rs — new)
  - FilePlan::validate (takes LanguageLimitsMap, returns Result)
  - BabyStepSpec.file_plan: Option<FilePlan> (9th field, serde(default))
  - FileLineTracker methods: record_lines_written, total_lines_written
    (on TreeNavigatorState in pipeline_tree_navigator.rs)
  - TreeNavigatorState.step_write_generation: u64 (TOCTOU counter)
  - MistakeRecorder trait (crates/plugins/src/tiered_analysis.rs)
  - T1HookConfig.mistake_recorder: Option<Arc<dyn MistakeRecorder>>
  - T2HookConfig.file_plan: Option<Arc<FilePlan>>
  - FileDisciplineConfig.language_limit_overrides: HashMap<String, LanguageLimits>
  - FUNCTION_COUNT_SLACK constant (= 2)
  - MAX_FUNCTIONS_PER_FILE constant (= 20)

consumes_symbols:
  - FailureCategory::LazyCode/EmptyBody/FileSizeExceeded (core/src/mistake_book.rs)
  - write_mistake_entry (orchestrator/src/mistake_book.rs:55)
  - BabyStepSpec (orchestrator/src/research_tree.rs:1667)
  - TreeNavigatorState (orchestrator/src/pipeline_tree_navigator.rs:265)
  - LanguageLimits (plugins/src/tiered_analysis.rs:28)
  - T1Hook::analyze, T2Hook::analyze BLOCK verdict paths (tiered_analysis.rs)
  - ProjectProfile.language_stats (core/src/project_profile.rs)

reachable_from:
  - LLM-generated BabyStepSpec JSON (via serde(default) field)
  - task_runner FileDisciplineConfig construction site (PI-override injection)
  - T1Hook/T2Hook per-file analysis in hot path (MistakeRecorder + Check 3)
  - TreeNavigator::record_result call site (FileLineTracker update)
  - PlanWindow SSB rows (FILE_PLAN block visible per baby step)

stub_ban:
  - MistakeRecorder production impl MUST call write_mistake_entry; no silent
    discard.
  - T2 Check 3 MUST count actual AST function nodes; no hardcoded mock count.
  - FileLineTracker MUST enforce step_write_generation TOCTOU guard; no
    unconditional success.
  - FilePlan::validate MUST reject max_lines > language hard_block; no
    always-pass path.

observable_effect:
  trigger: elai /task --mode cloud "implement a 3-file user CRUD module
    (user.rs, repo.rs, api.rs)" (cloud Sonnet 4.6)
  expect: each BabyStepSpec in plan SSB contains file_plan field with
    FileSpec entries; FileLineTracker cumulative_lines_by_step populated
    per step after execution; T2 BLOCK events for LazyCode produce >=0
    MistakeBook SSB entries (zero acceptable if no offenses); cargo check
    --workspace passes.
  adversarial_trigger: submit task whose FILE_PLAN specifies single .rs file
    with max_lines > 400 (Rust hard_block)
  adversarial_expect: FilePlan::validate returns Err(LineBudgetExceeded);
    plan rejected with error before task_runner dispatches baby steps;
    no BabyStepSpec emitted for that task.
  observable_ui: PlanWindow FILE_PLAN block visible per baby step (read-only).

## DETERMINISTIC_CHECK

```bash
# 1. Core file_plan structs defined
rg "pub struct FilePlan|pub struct FileSpec|pub struct FileModSpec|pub struct FunctionSpec" .elai_cc/crates/core/src/file_plan.rs

# 2. FILE_PLAN field present in BabyStepSpec
rg "pub file_plan" .elai_cc/crates/orchestrator/src/research_tree.rs

# 3. FileLineTracker methods in TreeNavigatorState
rg "fn record_lines_written|fn total_lines_written|step_write_generation" .elai_cc/crates/orchestrator/src/pipeline_tree_navigator.rs

# 4. MistakeRecorder trait defined
rg "trait MistakeRecorder" .elai_cc/crates/plugins/src/tiered_analysis.rs

# 5. T2 Check 3 FUNCTION_COUNT_SLACK constant and guard
rg "FUNCTION_COUNT_SLACK|function count.*exceeds FILE_PLAN" .elai_cc/crates/plugins/src/tiered_analysis.rs

# 6. FileDisciplineConfig language_limit_overrides field
rg "language_limit_overrides" .elai_cc/crates/plugins/src/tiered_analysis.rs

# 7. Contract tests compile and pass
cargo test -p elai-orchestrator --test c31_file_plan_contract 2>&1 | grep "test result"
# Expected: test result: ok. 16 passed

# 8. cargo check clean
cargo check --workspace 2>&1 | grep -c "^error" | xargs -I{} test {} -eq 0 && echo "CLEAN"

# 9. Adversarial: FilePlan::validate rejects oversized spec
# (exercised by validate_rejects_over_limit contract test)
cargo test -p elai-core -- validate_rejects_over_limit 2>&1 | grep "ok"
```

## ACCEPTANCE

- [ ] FilePlan, FileSpec, FileModSpec, FunctionSpec, FilePlanError defined in
      crates/core/src/file_plan.rs; no .unwrap() in production paths
- [ ] FilePlan::validate rejects max_lines > language hard_block limit;
      FunctionCountExceeded fires at > MAX_FUNCTIONS_PER_FILE (=20)
- [ ] FilePlan JSON round-trip lossless (serialize then deserialize equals original)
- [ ] BabyStepSpec gains file_plan: Option<FilePlan> field (9th field);
      #[serde(default)] keeps all legacy JSON deserializable
- [ ] BabyStepSpec construction sites in research_tree.rs updated to compile
      (file_plan: None); zero behavior change on existing code paths
- [ ] FileLineTracker: record_lines_written and total_lines_written on
      TreeNavigatorState; TOCTOU step_write_generation guard enforced;
      crash-recovery round-trip via Serialize/Deserialize preserved
- [ ] MistakeRecorder trait injectable into T1HookConfig and T2HookConfig;
      default no-op impl leaves existing tests unchanged
- [ ] T1 BLOCK for LazyCode invokes MistakeRecorder with FailureCategory::LazyCode
- [ ] T1 BLOCK for FileSizeExceeded invokes MistakeRecorder with
      FailureCategory::FileSizeExceeded
- [ ] T2 BLOCK for EmptyBody invokes MistakeRecorder with FailureCategory::EmptyBody
- [ ] T2 Check 3: file_plan present + function count > FileSpec.functions.len() +
      FUNCTION_COUNT_SLACK (=2) -> BLOCK; absent file_plan -> check silently skipped
- [ ] FileDisciplineConfig.language_limit_overrides wired from ProjectProfile at
      task_runner construction site; absent PI -> overrides empty; spec defaults hold
- [ ] Cloud smoke test: /task --mode cloud "implement a 3-file user CRUD module
      (user.rs, repo.rs, api.rs)" -> each BabyStepSpec in SSB plan contains
      populated file_plan; FileLineTracker records cumulative lines per step;
      T2 LazyCode BLOCK events -> >= 0 MistakeBook SSB entries; cargo check passes
- [ ] Adversarial smoke test: FILE_PLAN with single .rs file max_lines > 400 ->
      FilePlan::validate rejects plan before task_runner dispatch; clear error returned
- [ ] PlanWindow SSB rows include FILE_PLAN block per baby step (read-only display)
- [ ] All 16 contract tests in c31_file_plan_contract.rs pass
- [ ] cargo check --workspace clean; zero new warnings in touched crates

## Files touched

- ADD: .elai_cc/crates/core/src/file_plan.rs
- MODIFY: .elai_cc/crates/orchestrator/src/research_tree.rs
  (BabyStepSpec: add file_plan field; update 2 construction sites)
- MODIFY: .elai_cc/crates/orchestrator/src/pipeline_tree_navigator.rs
  (TreeNavigatorState: add cumulative_lines_by_step + step_write_generation;
   add record_lines_written + total_lines_written methods)
- MODIFY: .elai_cc/crates/plugins/src/tiered_analysis.rs
  (MistakeRecorder trait; T1HookConfig.mistake_recorder;
   T2HookConfig.file_plan + Check 3; FileDisciplineConfig.language_limit_overrides)
- ADD: .elai_cc/crates/contracts/tests/c31_file_plan_contract.rs
- ADD: workflow/reasoning/C-31-decisions.md

## Next

C-32 (skeleton-first generation) or next outstanding C-N gap per
bn_status_report.py output.
