B-20A — Decision Record (Tasks 4-8 only)

SCOPE: Tasks 4-8 of B-20 plan. OUT: Tasks 1-3 (dispatcher dedup, blocked by
B-18B IPC), Task 9 (WIP commit squash, blocked by user policy), all
frontend, mistake_book::FailureCategory, new dependencies, SsbActionType
extensions.

------------------------------------------------------------------------
DECISION-A — taxonomy choice (B-20 plan list, not phase-4B.txt)
------------------------------------------------------------------------
TWO competing 9-category lists exist for the same task:
  (1) docs/specs/guide/steps/phase-4B.txt:200-201 — SYNTAX_ERROR,
      TYPE_ERROR, IMPORT_ERROR, ASSERTION_ERROR, TIMEOUT, EMPTY_OUTPUT,
      RUNTIME_PANIC, PERMISSION_DENIED, RESOURCE_EXHAUSTED.
  (2) docs/thefinalglance/B-plan/B-20_tool_registry_unification.md:46-56
      — SYNTAX_ERROR, TYPE_MISMATCH, UNDEFINED_SYMBOL, MISSING_IMPORT,
      LIFETIME_ERROR, BORROW_ERROR, UNUSED_WARNING, TEST_ASSERTION,
      RUNTIME_PANIC.

CHOSEN: list (2) — the B-20 plan body, Rust-flavoured. SCOPE OVERRIDE in
the user task input cites this list verbatim. The phase-4B.txt list's
PERMISSION_DENIED + RESOURCE_EXHAUSTED + TIMEOUT are folded into
RUNTIME_PANIC. EMPTY_OUTPUT has no analogue and is dropped.

FUTURE FOLLOW-UP: if the dashboard ever needs PERMISSION_DENIED as a
distinct category (Tier C IPC sandbox failures), revisit this decision
post-B-20. Documented here so the option is not forgotten.

13 → 9 mapping table:
  CompilationError    -> SYNTAX_ERROR | TYPE_MISMATCH | UNDEFINED_SYMBOL |
                         MISSING_IMPORT | LIFETIME_ERROR | BORROW_ERROR
                         (split via regex table in tools/src/dispatcher.rs)
  TestFailure         -> TEST_ASSERTION
  RuntimeError        -> RUNTIME_PANIC
  Timeout             -> RUNTIME_PANIC (folded)
  ResourceExhausted   -> RUNTIME_PANIC (folded)
  LazyCode            -> UNUSED_WARNING (folded — closest analogue)
  EmptyBody           -> UNUSED_WARNING (folded)
  FileSizeExceeded    -> DROP (no analogue; dashboard will lose this signal,
                         documented as B-future enhancement)
  ContextExplosion    -> DROP
  RawLogOverload      -> DROP
  PhaseGateFailure    -> DROP
  ScriptQualityDependency -> DROP
  SpeculativeRepair   -> DROP

Result: exactly 9 enum variants — verified by
  rg "Self::(SyntaxError|TypeMismatch|UndefinedSymbol|MissingImport|
        LifetimeError|BorrowError|UnusedWarning|TestAssertion|
        RuntimePanic)" elai_cc/crates/core/src/tool_dispatcher.rs
  → 9 hits.

------------------------------------------------------------------------
DECISION-B — reduce BOTH enums in lockstep
------------------------------------------------------------------------
elai_tools::failure_taxonomy::FailureCategory AND
elai_core::tool_dispatcher::ToolFailureCategory both reduced to the same
9 variant names (SCREAMING_SNAKE_CASE Display strings). SSB serialised
category strings remain consistent across the two paths.

OUT OF SCOPE — mistake_book::FailureCategory at elai_core (different
concern: interrogation tree fix matrix; 14 variants like LogicError,
BorrowCheck). Untouched per user instruction.

------------------------------------------------------------------------
DECISION-C — similarity primitive (hand-rolled, no dep)
------------------------------------------------------------------------
NEW FILE: .elai_cc/crates/tools/src/string_similarity.rs
  pub fn levenshtein(a, b) -> usize           (~30 lines, O(n*m) DP, Vec<Vec>)
  pub fn similarity_ratio(a, b) -> f32        (1 - dist/max_len)
  pub fn best_line_match(content, query)      (returns highest-ratio line)
  pub const SIMILARITY_HINT_THRESHOLD: f32 = 0.5

Threshold: emit hint only when ratio > 0.5. Rejected `strsim`,
`levenshtein`, `similar` per CLAUDE.md "no new dependencies without
confirmation" + Anti-Slopsquatting + the user's PRE-DECIDED ANSWER.

EditOutcome extended with:
  pub similarity_hint: Option<SimilarityHint>
  struct SimilarityHint { nearest_text: String, line_no: u32, ratio_bp: u16 }
ratio_bp (basis points 0..=10000) preserves Eq derive on EditOutcome.

------------------------------------------------------------------------
DECISION-D — 50% budget guard (caller-side enforcement)
------------------------------------------------------------------------
ToolDispatcher trait left UNCHANGED (per user PRE-DECIDED ANSWER). Spec
intent of "tool layer enforcement" is partially met:

  * Tool layer: NEW inherent method
    InProcessToolDispatcher::dispatch_file_read_with_force_full(
        path, line_threshold, force_full
    ) -> Result<FileView, ToolDispatchError>
    Returns outline by default for files > line_threshold; force_full=true
    overrides the outline gate and returns full content.

  * Caller (orchestrator/task_runner.rs): NEW pub fn
    enforce_file_read_budget_guard(total_bytes, budget_remaining_tokens)
        -> FileReadBudgetVerdict { Allow | Block { hint, budget_remaining } }
    Caller computes 4-bytes/token estimate (phase-4B.txt §context-budget).
    If estimated_tokens > 0.5 * budget ⇒ Block with diagnostic hint.

SCOPE-CAP: caller-side enforcement was chosen over trait extension
because:
  1. ToolDispatcher trait surface is load-bearing across dashboard +
     contracts + every test fake. Adding a parameter breaks every
     impl site.
  2. ContextBudget lives at elai_core::context_budget::ContextBudget;
     the dispatcher struct (tools/src/dispatcher.rs:68-74) carries no
     budget reference today. Threading it through would require
     dashboard wiring updates (out of B-20A scope).
  3. ResolvedConfig in task_runner already has model_capability_profile
     from B-10, so caller-side is the smallest correct surface.

The 50% guard is therefore enforced at the call site, not inside the
dispatcher. Documented inline (task_runner.rs comment block at the
helper's definition).

------------------------------------------------------------------------
DECISION-E — integration_checkpoint_success (string Finding, not new variant)
------------------------------------------------------------------------
Emit a Finding row with category string
  "integration_checkpoint_success"
mirroring the existing INTEGRATION_CHECKPOINT_FAILED_CATEGORY pattern.

NEW: pub const INTEGRATION_CHECKPOINT_SUCCESS_CATEGORY: &str =
       "integration_checkpoint_success"
NEW: success branch of emit_checkpoint_outcome emits Finding(severity=info)
     with the new category and "{level} {node_id}: integration ok in {ms}ms".

NO new SsbActionType variant. NO SCHEMA_VERSION bump (stays at 23).

DEFERRED OPTION (b): a proper IntegrationCheckpointSuccessPayload variant
+ 14-site SSB extension + SCHEMA_VERSION bump 23->24. Recommended by the
B-20A audit phrasing "for B-7 dashboard surface". Deferred to a future
batch — the dashboard histogram can render from the existing Finding
category string today; richer per-level payloads (success rate, p95
duration) are a follow-up.

------------------------------------------------------------------------
PER-TASK rationale
------------------------------------------------------------------------
Task 4 — Reduce both 13-variant enums to the 9-variant lockstep set.
  - Updated tools/src/failure_taxonomy.rs (FailureCategory + detect_category +
    suggest_fix); core/src/tool_dispatcher.rs (ToolFailureCategory + as_str +
    doc); tools/src/dispatcher.rs (classify_output regex table). Updated
    consumer tests in tools/tests/tool_dispatch.rs and contracts/tests/
    tool_dispatcher_contract.rs (probes referenced removed variants like
    ContextExplosion / PhaseGateFailure / Timeout / ResourceExhausted /
    FileSizeExceeded — replaced with the new 9 probes).

Task 5 — Auditor reads classification BEFORE raw output.
  - Verified ordering in task_runner.rs:4214-4239: run_classify_dispatch is
    called before any subsequent emit_tool_dispatch / emit_finding for the
    failure path. SsbViewer::failure_taxonomy_last (ssb_viewer.rs:1780)
    reads `ORDER BY seq DESC LIMIT 1` so the categorized row wins because
    the SSB seq is monotonic and write_payload yields seq-confirmation
    before the next emit fires.
  - Added an in-source comment naming the spec source (phase-4B.txt §
    failure-taxonomy + B-20 plan body Tasks 4-5) at the call site so future
    readers can verify the ordering claim without re-reading the spec.

Task 6 — Edit similarity hint.
  - NEW tools/src/string_similarity.rs with hand-rolled Levenshtein + ratio.
  - SimilarityHint added to elai_core::tool_dispatcher::EditOutcome via a
    new optional field. ratio_bp basis points preserve Eq derive.
  - Wired into dispatcher.rs::dispatch_edit on the EditMatchTier::NotMatched
    arm. best_line_match returns None when best ratio <= 0.5 ⇒ no hint.

Task 7 — File outline force_full + 50% budget guard.
  - NEW inherent method dispatch_file_read_with_force_full. Trait shape
    unchanged.
  - generate_file_view_with_override is the shared helper between the
    new method and the existing trait method.
  - NEW pub fn enforce_file_read_budget_guard + FileReadBudgetVerdict
    in task_runner. Tests exercise both Allow and Block paths.

Task 8 — integration_checkpoint_success Finding.
  - INTEGRATION_CHECKPOINT_SUCCESS_CATEGORY const added next to the
    existing _FAILED constant.
  - emit_checkpoint_outcome now ALWAYS emits a Finding (success: info,
    failure: error) keyed by the appropriate category.
  - emit_integration_checkpoint_for_test public test entry-point added so
    the contract test exercises the success/failure paths.

------------------------------------------------------------------------
DEFERRALS
------------------------------------------------------------------------
- Tasks 1-3 (dispatcher dedup): blocked by B-18B IPC landing.
- Task 9 (WIP commit squash): blocked by user policy + git infra.
- PERMISSION_DENIED variant: rejected per Decision A; phase-4B.txt's
  competing list is the runner-up if the choice is reversed in B-future.
- New SsbActionType variant for integration_checkpoint_success: deferred
  per Decision E.
- FileSizeExceeded category: dropped per Decision A. Dashboard loses the
  signal until a B-future replacement.
- B-19A/W5 contract tests still missing (StopHookGate /
  HookLatencyViolationPayload / RuffPostToolUseRunner) — irrelevant to
  B-20A scope.
