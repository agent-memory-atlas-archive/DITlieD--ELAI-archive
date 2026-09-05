C-31 — Code-Completeness Layer 1 FilePlan + FileLineTracker
==============================================================

Session: s-25672
Plan: F:\.ELAI_workflow\docs\thefinalglance\C-plan\C-31_code_completeness_file_plan.md
Context: F:\.ELAI_workflow\workflow\sessions\s-25672\context-package-C31.md

What

Implement Layer 1 of the Code-Completeness enforcement design:
  - FilePlan/FileSpec/FileModSpec/FunctionSpec types in a new
    elai-core::file_plan module.
  - file_plan as the 11th BabyStepSpec field (called the "9th" in the
    spec narrative because spec predates B-28 + C-29 additions).
  - FileLineTracker on TreeNavigatorState with TOCTOU-safe
    step_write_generation counter.
  - MistakeRecorder trait + T1Hook/T2Hook setter wiring for
    FailureCategory::{LazyCode,EmptyBody,FileSizeExceeded} BLOCK events.
  - T2 Check 3 (function-count guard) with named-constant slack=2.
  - FileDisciplineConfig.language_limit_overrides — populated only by
    PI down the line (deferred this session, see DECISION D5).
  - 19 c31_file_plan_contract tests + 6 elai-core file_plan unit tests
    all GREEN.

Key decisions

DECISION D1 — DRIFT-1: BabyStepSpec is at research_tree.rs:1726-1774
with 10 pre-existing fields (not 8). The new `file_plan` lands as the
11th field, `Option<elai_core::file_plan::FilePlan>` with
`#[serde(default)]`. Schema-compatible — SCHEMA_VERSION stays at 53.

DECISION D2 — DRIFT-2: BabyStepSpec construction sites are at
research_tree.rs:2306 and 2347 (not 2233 and 2273). Both updated to
`file_plan: None`. Eight additional test-only construction sites (in
contracts/tests, focus_tunnel, pipeline_tree_navigator tests,
task_runner tests, and the measure_4e_tree example) also patched.

DECISION D3 — DRIFT-3 + 4: TreeNavigatorState is at L346-386, not
265-295. Added `cumulative_lines_by_step: BTreeMap<BabyStepId, u32>`
(BTreeMap, matching the collection style of the surrounding fields)
and `step_write_generation: u64`, both `#[serde(default)]`. The
implemented method names slightly diverge from the plan to make TOCTOU
explicit:
  - `begin_line_write(&self) -> u64` (captures expected_generation)
  - `record_lines_written(&mut self, step_id, lines, expected_generation)
       -> Result<(), NavigatorError>`
  - `total_lines_written(&self) -> u32`
This is forward-looking insurance — Sequential is the only current
MergeStrategy (C-9 DECISION-B) so today's flow never races, but the
generation token is mandatory at the API surface so callers cannot
forget it when parallel execution lands.

DECISION D4 — DRIFT-5: T1Hook and T2Hook had no `T1HookConfig` /
`T2HookConfig` struct; both were configured via `set_*` setter methods.
C-31 follows the established convention:
  - `T1Hook::set_mistake_recorder(Arc<dyn MistakeRecorder>)`
  - `T1Hook::set_current_baby_step_id(impl Into<String>)`
  - `T2Hook::set_mistake_recorder(...)` + `set_current_baby_step_id(...)`
  - `T2Hook::set_file_plan(Arc<FilePlan>)`
No new HookConfig struct — every existing caller continues to work
without migration.

DECISION D5 — DRIFT-6 / OPT-C: ProjectIntelligence-derived size limits
are DEFERRED for this session. `language_limit_overrides:
FxHashMap<String, LanguageLimits>` field is added (with `#[serde(default)]`
so it defaults to empty); `evaluate_file_size` consults overrides first
and falls back to `limits`. The from_profile wiring is unchanged —
overrides remain empty until a follow-up C-N extends LanguageProfile
with `LanguageStats { detected_file_count, median_file_loc, p95_file_loc }`.
Spec defaults always win in the meantime. Smoke-tested by
`language_limit_overrides_absent_defaults_hold`.

DECISION D6 — DRIFT-7: SCHEMA_VERSION not bumped. file_plan rides on
BabyStepSpec via serde(default), all existing serialised forms still
deserialize.

DECISION D7 — DRIFT-8: The contracts crate package name is
`elai_cc_contracts`, not `elai-orchestrator`. The C-31 contract test
is run via `cargo test -p elai_cc_contracts --test
c31_file_plan_contract` (plan had the wrong `-p` flag).

DECISION D8 — DRIFT-9: The Rust tree-sitter top-level function node is
`function_item` (not `item_function` as the plan said). T2 Check 3
counts `function_item` for Rust + `function_definition` for Python via
the new `count_nodes_of_kind` helper. Languages outside Rust/Python
silently skip Check 3 (consistent with the plan's scope).

DECISION D9 — DRIFT-10: T1Hook entry point is `evaluate`, T2Hook entry
point is `evaluate` — not `analyze` as the plan said. MistakeRecorder
calls fire from:
  - T1Hook::evaluate_file_size BLOCK branch (FailureCategory::FileSizeExceeded)
  - T2Hook::evaluate post-classification block, when verdict is BLOCK
    and findings contain empty_fn_body / lazy_*. Per spec L1224-1255,
    T1 only WARNs on lazy patterns, so the LazyCode/EmptyBody
    Layer-1 events properly originate at T2.

DECISION D10 — DRIFT-11: The PI-override route plugs into
`DeterministicAuditorConfig::from_profile` (deterministic_auditor.rs
L163 / L187). This session leaves that integration untouched because
LanguageStats does not exist yet (see D5). The plumbing is ready —
once LanguageStats lands, the override map can be populated inline
in `from_profile_with_mutation_and_metamorphic` without further code
changes to T1Hook/T2Hook.

DECISION D11 — Out of scope: ModuleTree (spec L828-840). Confirmed by
the orchestrator. No type added.

DECISION D12 — Out of scope: §Task 7 PI override wiring (see D5).
Field added, populator deferred to a follow-up C-N.

Files touched

ADD:
  .elai_cc/crates/core/src/file_plan.rs (181 lines incl. 6 unit tests)
  .elai_cc/crates/contracts/tests/c31_file_plan_contract.rs (19 tests)

MODIFY:
  .elai_cc/crates/core/src/lib.rs
    (pub mod file_plan + 7 re-exports)
  .elai_cc/crates/orchestrator/src/research_tree.rs
    (BabyStepSpec.file_plan + 2 construction sites)
  .elai_cc/crates/orchestrator/src/pipeline_tree_navigator.rs
    (TreeNavigatorState: cumulative_lines_by_step + step_write_generation;
     NavigatorError enum; begin_line_write + record_lines_written +
     total_lines_written; one test construction site updated)
  .elai_cc/crates/orchestrator/src/task_runner.rs
    (one test BabyStepSpec construction site updated)
  .elai_cc/crates/orchestrator/src/focus_tunnel/pre_assembly.rs
    (one test BabyStepSpec construction site updated)
  .elai_cc/crates/orchestrator/examples/measure_4e_tree.rs
    (BabyStepSpec construction in synthetic-tree builder)
  .elai_cc/crates/plugins/src/tiered_analysis.rs
    (MistakeRecorder trait + NoopMistakeRecorder;
     FUNCTION_COUNT_SLACK + MAX_FUNCTIONS_PER_FILE constants;
     FileDisciplineConfig.language_limit_overrides;
     T1Hook: mistake_recorder + current_baby_step_id fields,
       set_mistake_recorder + set_current_baby_step_id setters,
       MistakeRecorder call from evaluate_file_size BLOCK,
       language_limit_overrides applied first in evaluate_file_size;
     T2Hook: mistake_recorder + current_baby_step_id + file_plan fields,
       three new setters; Check 3 wiring; MistakeRecorder call from the
       post-classification BLOCK branch on lazy_*/empty_fn_body;
     count_nodes_of_kind helper)
  .elai_cc/crates/contracts/tests/{c26_constrained_decoding,
       focus_tunnel_e3, focus_tunnel_e5, focus_tunnel_wiring}_contract.rs
    (BabyStepSpec construction site updates — 9 sites total)

NEW PUBLIC API

elai_core::file_plan::{FilePlan, FileSpec, FileModSpec, FunctionSpec,
  FilePlanError, LanguageLimitsMap, MAX_FUNCTIONS_PER_FILE,
  DEFAULT_LANGUAGE_HARD_BLOCK}
elai_orchestrator::pipeline_tree_navigator::NavigatorError
elai_orchestrator::pipeline_tree_navigator::TreeNavigatorState::{begin_line_write,
  record_lines_written, total_lines_written, cumulative_lines_by_step,
  step_write_generation}
elai_plugins::tiered_analysis::{MistakeRecorder, NoopMistakeRecorder,
  FUNCTION_COUNT_SLACK, MAX_FUNCTIONS_PER_FILE,
  FileDisciplineConfig.language_limit_overrides}
elai_plugins::tiered_analysis::T1Hook::{set_mistake_recorder,
  set_current_baby_step_id}
elai_plugins::tiered_analysis::T2Hook::{set_mistake_recorder,
  set_current_baby_step_id, set_file_plan}

Test results

cargo test -p elai-core file_plan: 6/6 GREEN
cargo test -p elai_cc_contracts --test c31_file_plan_contract: 19/19 GREEN
cargo check --workspace: clean for everything except a pre-existing
  c9_autonomous_infra_contract.rs FxBuildHasher mismatch (untracked,
  not introduced by C-31; confirmed via git status).
cargo test -p elai-plugins --lib: 99 pass / 2 fail; the 2 failures
  (discovers_builtin_and_bundled_plugins +
  default_bundled_root_loads_repo_bundles_as_installed_plugins) are
  pre-existing on main baseline (verified by `git stash` round-trip)
  and unrelated to C-31.

Open questions resolved by orchestrator before EXECUTING:
  Q1 (PI-override route): OPT-C accepted — DECISION D5.
  Q2 (HookConfig vs setters): setters accepted — DECISION D4.
  Q3 (ModuleTree): out of scope — DECISION D11.

Adversarial coverage (mapped to context-package §A1-A12):
  A1 legacy JSON without file_plan -> baby_step_spec_legacy_deserializes
  A2 oversized FileSpec -> validate_rejects_over_limit
  A3 >MAX_FUNCTIONS_PER_FILE -> validate_rejects_over_function_count
  A4 Check 3 7>5 -> t2_check3_fires_on_excess_functions
  A5 Check 3 at slack boundary -> t2_check3_slack_of_two_accepted
  A6 Check 3 with no plan -> t2_check3_skipped_when_no_file_plan
  A7 FileSizeExceeded BLOCK records -> mistake_recorder_file_size_exceeded_block_invoked
  A8 T1 WARN on lazy doesn't record -> mistake_recorder_lazy_code_warn_does_not_record
  A10 generation mismatch -> file_line_tracker_toctou_guard
  A11 state round-trips -> file_line_tracker_state_round_trips
  A12 default-only overrides hold -> language_limit_overrides_absent_defaults_hold

Deferred (follow-up C-N)
  - ProjectIntelligence LanguageStats extension + populator wiring
    for FileDisciplineConfig.language_limit_overrides.
  - ModuleTree (spec L828-840).
  - Production MistakeRecorder impl in task_runner that calls
    write_mistake_entry — defaults to NoopMistakeRecorder until wired.

Stub-ban verification
  - MistakeRecorder default impl (NoopMistakeRecorder) intentionally
    no-ops because production implementations live in task_runner per
    the WIRING_CONTRACT; this is not a stub but the documented
    default seam.
  - T2 Check 3 counts real AST nodes via count_nodes_of_kind walking
    function_item / function_definition; no hardcoded mock count.
  - FileLineTracker enforces step_write_generation TOCTOU guard
    unconditionally (no `let _ = expected_generation;` shortcut).
  - FilePlan::validate rejects oversize budgets BEFORE returning Ok;
    no always-pass branch.
