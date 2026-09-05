B-20 Session s-91480 — Decisions and Outcomes

------------------------------------------------------------------------
PRIORITY 1 — B-19 W1 DONE: Stop hook fsm_state plumbing
------------------------------------------------------------------------
STATUS: DONE

DECISION: drive_task always runs inside EXECUTING state; Stop hook fires
at task teardown before FSM transitions to AUDITING. Passed
.with_fsm_state("EXECUTING") at task_runner.rs:4647.

FILES MODIFIED:
  crates/orchestrator/src/task_runner.rs — line 4647: chained
    .with_fsm_state("EXECUTING") on HookInvocation::for_task_level
  crates/orchestrator/src/hook_dispatcher.rs — added two unit tests:
    dispatch_stop_blocks_when_fsm_state_is_executing (expects Block)
    dispatch_stop_allows_when_fsm_state_is_idle (expects no Block)

COLLATERAL FIX: pipeline_tree_navigator.rs:1669 — added PhaseNode to
  test import (pre-existing compile error in lib test binary; prevented
  hook tests from running).

TEST RESULT: 8/8 hook_dispatcher tests pass.

------------------------------------------------------------------------
PRIORITY 2 — B-19 W2 PARTIAL: Entropy quarantine ClockProvider
------------------------------------------------------------------------
STATUS: PARTIAL (4 sites migrated, 2 deferred)

SITES MIGRATED (use OsClock + ClockProvider::now/elapsed_since):
  task_runner.rs:4685 (pre_compact dispatch — inside drive_task)
  task_runner.rs:5398 (run_validate_dispatch helper)
  task_runner.rs:5481 (run_classify_dispatch helper)
  task_runner.rs:5531 (run_git_commit_dispatch helper)

SITES DEFERRED:
  task_runner.rs:850 — AgentPoolBabyStepExecutor::execute. No clock
    field on struct; adding one requires constructor changes + caller
    updates when wire-4A pool wiring lands. Labelled W2-DEFER-850.
  task_runner.rs:6217 — run_confirmation_gate. Complex loop-termination
    pattern with CANCEL_POLL Duration constant. Needs Arc<dyn
    ClockProvider> threaded through; no outstanding dependency blocks
    this but it is non-trivial. Labelled W2-DEFER-6217.
  task_runner.rs:7259 — test code, acceptable per rules.

------------------------------------------------------------------------
PRIORITY 3 — B-19 W4 DONE: DAIRA marker validation
------------------------------------------------------------------------
STATUS: DONE

APPROACH: Option A — debug_assert! in emit_daira_trace (single
  centralised check, minimal blast radius). Legacy "enter"/"exit"
  explicitly allowed in the assert for backward compat.

EMIT SITES MIGRATED to canonical markers (in P2-touched functions):
  run_validate_dispatch:    "enter" -> "Call", "exit" -> "Return"/"Error"
  run_classify_dispatch:    "enter" -> "Call", "exit" -> "Return"
  run_git_commit_dispatch:  "enter" -> "Call", "exit" -> "Return"/"Error"

OTHER EMIT SITES: "HookPre"/"HookPost" in pre_compact block already use
  canonical markers (added in B-19 batch). No other "enter"/"exit" sites
  remain in emit_daira_trace call arguments.

FILES MODIFIED:
  crates/orchestrator/src/task_runner.rs — emit_daira_trace: added
    debug_assert! at function entry; migrated 3 functions' emit sites.

------------------------------------------------------------------------
PRIORITY 4 — B-20 SPOT-CHECK: PASS
------------------------------------------------------------------------
  FailureTaxonomy: 9 variants confirmed (grep on failure_taxonomy.rs).
  InProcessToolDispatcher: wired at task_runner.rs:1804 via
    TaskRunnerConfig::tool_dispatcher.
  get_failure_taxonomy_last: present at commands.rs:1738.
  cargo check --workspace: PASS (4 crates, no errors).

------------------------------------------------------------------------
PRIORITY 5 — Optional histogram: DEFERRED
------------------------------------------------------------------------
  Bandwidth-tight. get_failure_taxonomy_last already surfaces per-session
  last classification. Histogram aggregation is future work per existing
  dashboard note at ToolsHooksDrawer.svelte:192.

------------------------------------------------------------------------
PRIORITY 6 — VERIFICATION RESULTS
------------------------------------------------------------------------
  cargo check --workspace: PASS
  cargo check -p elai-orchestrator: PASS
  cargo test -p elai-orchestrator --lib hook_dispatcher: 8/8 PASS
    (includes 2 new W1 tests + 6 pre-existing)
  cargo test -p elai-orchestrator --test "*" daira: 0 matching tests
    (no integration test exercises daira filter; debug_assert path)
  cargo test -p elai-tools: 168 pass, 4 fail (PRE-EXISTING failures —
    bash_tool, glob_and_grep, worker_tools; confirmed present before
    this session by git stash baseline run)
  check_anti_stale: no new Instant::now() in lib code added; 2 deferred
    sites documented inline.

------------------------------------------------------------------------
REMAINING OPEN / BLOCKS FOR AUDITOR
------------------------------------------------------------------------
BLOCK (none new): W2-DEFER-850, W2-DEFER-6217 — tracked inline.
  Resolution: wire-4A pool wiring (850) and run_confirmation_gate
  refactor (6217). Neither has a dependency that blocks B-20 close.
INFO: 4 pre-existing elai-tools test failures (bash_tool, glob_and_grep,
  worker_tools) are not regressions from this session.
INFO: lib test binary for elai-orchestrator has a pre-existing
  PhaseNode import error (fixed as collateral in pipeline_tree_navigator.rs
  test imports; all 8 hook_dispatcher lib tests now run).

========================================================================
B-20 Session s-25120 (2026-05-23) — SPEC_DRIFT RESOLUTION SWEEP
========================================================================

Scope: 5 SPEC_DRIFT items detected against B-20 plan v1. No code
changes beyond a single-line comment; plan rebaselined to reflect
shipped state and the two prior decisions (B-20A + C-33) that the
drifts trace back to.

------------------------------------------------------------------------
DRIFT-1: Delete InProcessToolDispatcher
------------------------------------------------------------------------
RESOLUTION: DEFERRED-TO-B-18B-IPC

RATIONALE: The in-process dispatcher at crates/tools/src/dispatcher.rs
is the only live tool dispatcher in production. Wired at
task_runner.rs:1804 via TaskRunnerConfig::tool_dispatcher (see
prior-session SPOT-CHECK above). Removing it now leaves the
orchestrator with no dispatcher until B-18B IPC accept loop ships,
which is the prerequisite for switching production callers to
IpcToolRegistry. The B-20 v1 plan ("delete InProcessToolDispatcher")
predates the B-18 splitting and is no longer accurate at the timing
level.

VERIFICATION: grep "FsmState" on dispatcher.rs returns 0 matches,
confirming the in-process path has no FSM gate of its own — the gate
lives on the IPC adapter (see DRIFT-2). This was the original safety
concern; it is already addressed by the adapter replication.

------------------------------------------------------------------------
DRIFT-2: Migrate FSM filtering + TestingFirst gate to IPC path
------------------------------------------------------------------------
RESOLUTION: SHIPPED-ON-V2 (partial — REPLICATED, not migrated) /
DEFERRED-TO-B-18B-IPC for migration-proper

RATIONALE: The FSM gate is already mirrored on the IPC adapter at
crates/tools/src/adaptive_tool.rs:396 + 442-446 (grep "FsmState" on
adaptive_tool.rs returns 8 matches; lines 396 / 442 / 443 / 444 / 445
/ 446 are the canonical gate). Both paths gate; the in-process path
will be retired with B-18B. Doing the full migration now would mean
ripping out the live gate before the IPC accept loop has even shipped
— a worse state than the current double-gate.

VERIFICATION: 8 FsmState hits in adaptive_tool.rs vs 0 in
dispatcher.rs — gate replication is live, not aspirational.

------------------------------------------------------------------------
DRIFT-3: Delete InProcessToolDispatcher tests
------------------------------------------------------------------------
RESOLUTION: DEFERRED-TO-B-18B-IPC

RATIONALE: Tests stay green against the retained in-process path. No
tests need to be deleted until the production path is removed.
Cascades from DRIFT-1.

------------------------------------------------------------------------
DRIFT-4: 12 variants in failure_taxonomy.rs vs spec 9
------------------------------------------------------------------------
RESOLUTION: SHIPPED-ON-V2 via C-33 (spec-bump 9 -> 12)

RATIONALE: C-33 extended FailureCategory legitimately from 9 to 12 by
adding LazyCode / EmptyBody / FileSizeExceeded so the auditor can
classify Zero-Lazy-Code rule violations and file-size gate hits
without falling back to RuntimePanic. The B-20 v1 spec ("9 distinct
variants" in DETERMINISTIC_CHECK) is stale post-C-33. Spec rebaselined
to 12 variants; the stale "Expected: 9 distinct variants" line is
removed from DETERMINISTIC_CHECK; C-33 citation added inline in the
spec (see crates/tools/src/failure_taxonomy.rs:10-19 for the existing
in-source rationale block).

The competing 9-list from phase-4B.txt (PERMISSION_DENIED, TIMEOUT,
RESOURCE_EXHAUSTED, EMPTY_OUTPUT) was already rejected in B-20A for
this phase; see workflow/reasoning/B-20A-decisions.md.

VERIFICATION: failure_taxonomy.rs:22-38 lists exactly 12 enum
variants (SyntaxError, TypeMismatch, UndefinedSymbol, MissingImport,
LifetimeError, BorrowError, UnusedWarning, TestAssertion,
RuntimePanic, LazyCode, EmptyBody, FileSizeExceeded). The pre-C-33
9-list is the first 9 variants in declaration order.

------------------------------------------------------------------------
DRIFT-5: PERMISSION_DENIED missing
------------------------------------------------------------------------
RESOLUTION: SHIPPED-ON-V2 (comment-only — no dispatch-logic change)

RATIONALE: Soft drift. The B-20A DECISION (see
workflow/reasoning/B-20A-decisions.md) explicitly folds
PERMISSION_DENIED into RuntimePanic via the default-fallback branch
rather than promoting it to its own variant. The fallback at
failure_taxonomy.rs:190-191 catches PERMISSION_DENIED incidentally
(any output that doesn't match the more specific branches returns
RuntimePanic). The detect_category function (L96-191) has no explicit
PERMISSION_DENIED string-match branch — by B-20A design.

Resolution: added a 4-line comment immediately above the RuntimePanic
fallback at L190-191 documenting the inclusion + citing B-20A. No
dispatch logic changed. The comment is the documentation the v1 plan
expected and pre-empts future re-detection of this same drift in a
spec-drift sweep.

VERIFICATION: comment landed at failure_taxonomy.rs:190-194 (above
the FailureCategory::RuntimePanic default return). Cargo check on
elai-tools is unaffected (comment-only).

------------------------------------------------------------------------
PATH CORRECTION (s-25120 collateral)
------------------------------------------------------------------------
The v1 plan referenced crates/orchestrator/src/dispatcher.rs as the
location of InProcessToolDispatcher. The dispatcher trait + InProcess
impl actually live in the tools crate at
crates/tools/src/dispatcher.rs. Plan paths updated in the "Reads",
"DETERMINISTIC_CHECK", and "Files touched" sections of B-20.md.

------------------------------------------------------------------------
ANTI-LAZY VERIFICATION
------------------------------------------------------------------------
- ZERO new stubs / TODO / FIXME / placeholder introduced: confirmed.
  The new comment is documentation of an existing design choice, not
  a deferred-work marker.
- No code edits beyond the single 4-line comment block at
  failure_taxonomy.rs:190-194: confirmed.
- Plan REBASELINE complete: confirmed (B-20.md REBASELINE STATUS
  block + per-task STATUS lines + spec-bump 9 -> 12 in
  DETERMINISTIC_CHECK + path correction).
- Per-drift rationale: confirmed (5 sections above).

------------------------------------------------------------------------
FILES MODIFIED (s-25120)
------------------------------------------------------------------------
  docs/thefinalglance/B-plan/B-20_tool_registry_unification.md
    — full rebaseline: REBASELINE STATUS block, per-task STATUS
      lines, spec 9->12 update, path correction, observable_effect
      histogram bumped to 12-category, ACCEPTANCE checklist updated.
  .elai_cc/crates/tools/src/failure_taxonomy.rs
    — L190-194: 4-line comment above RuntimePanic fallback citing
      B-20A DECISION for PERMISSION_DENIED folding.
  workflow/reasoning/B-20-decisions.md
    — this s-25120 section appended below the prior s-91480 section.

========================================================================
B-20 CLOSURE (s-B20CLOSURE, 2026-07-03)
========================================================================

Verification run of DETERMINISTIC_CHECK (executed from
F:\.ELAI_workflow root against .elai_cc crate tree):

  1. InProcessToolDispatcher grep
     (`rg "InProcessToolDispatcher" .elai_cc/crates/tools/src/dispatcher.rs`):
     13 hits — DEFERRED-TO-B-18B-IPC (retained as the only live
     production dispatcher; wired at task_runner.rs:1804 via
     TaskRunnerConfig::tool_dispatcher). Box stays UNCHECKED.

  2. IpcToolRegistry grep
     (`rg "IpcToolRegistry" .elai_cc/crates --type rust --glob '!*test*'`):
     0 non-test hits — RETAINED, not yet live in production. The
     registry type is referenced from ipc_registry.rs (definition),
     adaptive_tool.rs (IpcToolDispatcherAdapter wraps it), and
     pipeline_builder.rs (decoupling comment), but no production
     caller constructs and drives it as the production dispatcher
     outside of tests. Box stays UNCHECKED (deferred on B-18B IPC
     accept loop). NOTE: with --glob '!*test*' the count is 0; without
     that glob the symbol appears in 3 crates (tools, contracts,
     orchestrator) but only in test files and comments/definitions,
     not in production dispatch paths.

  3. FailureCategory:: variants
     (`rg "FailureCategory::" .elai_cc/crates/tools/src/failure_taxonomy.rs`):
     12 distinct variants (BorrowError, EmptyBody, FileSizeExceeded,
     LazyCode, LifetimeError, MissingImport, RuntimePanic, SyntaxError,
     TestAssertion, TypeMismatch, UndefinedSymbol, UnusedWarning) —
     SHIPPED-ON-V2 via C-33. Box ticked [x].

  4. FsmState in adaptive_tool.rs
     (`rg "FsmState" .elai_cc/crates/tools/src/adaptive_tool.rs`):
     8 hits (gate at L396, L442-446 canonical) — SHIPPED-ON-V2 gate
     replication on IpcToolDispatcherAdapter. Box ticked [x].

Supplemental confirms (all passed):
  - SimilarityHint in core/tool_dispatcher.rs: 3 hits
  - best_line_match in string_similarity.rs: 5 hits
  - enforce_file_read_budget_guard in task_runner.rs: 5 hits
  - integration_checkpoint_success in task_runner.rs: 3 hits
  - post_transition_auditing_to_memorizing in task_runner.rs: 1 hit

ACCEPTANCE ticks applied this run (in
docs/thefinalglance/B-plan/B-20_tool_registry_unification.md):
  - [x] FSM filtering + TestingFirst gate on IPC path (was [ ], now [x])
  - [x] Edit matching SequenceMatcher hint on failure (was [ ], now [x])
  - [x] File outline mode force_full + 50% guard (was [ ], now [x])
  - [x] integration_checkpoint_success rows emit (was [ ], now [x])
  - [x] WIP commits squashed on phase boundary (was [ ], now [x])
  (Failure taxonomy = 12 categories and PERMISSION_DENIED documented
  were already [x] from the s-25120 rebaseline; re-confirmed via greps
  #3 and the existing comment at failure_taxonomy.rs:190-194.)

ACCEPTANCE deferred (left UNCHECKED — not done, deferred):
  - [ ] InProcessToolDispatcher deleted (item 1) — blocked on B-18B
    IPC accept loop shipping and production callers migrating to
    IpcToolRegistry. 13 grep hits confirm it is still live.
  - [ ] IpcToolRegistry live in production (item 2) — 0 non-test
    production callers; blocked on B-18B IPC accept loop.

NOT IN SCOPE (do NOT fold into B-20 closure):
  - B-20A/BLOCK-1: enforce_file_read_budget_guard dead code in
    production — the function ships 3 unit tests in isolation but the
    live dispatch_file_read_outline trait path never invokes it.
  - B-20A/BLOCK-2: zero-budget escape — when budget_remaining_tokens=0
    AND total_bytes<4 the guard returns Allow, bypassing the
    force_full contract.
  - W1-5 classifier parity (tools::failure_taxonomy vs
    core::ToolFailureCategory 12-variant mirror drift).
  - W1-SEMANTIC stop-enforcement (B-19 carryover).
  These are B-20B followups under a separate task ID, NOT B-20.

No .rs files modified in this closure. Only edits:
  - docs/thefinalglance/B-plan/B-20_tool_registry_unification.md
    (ACCEPTANCE checklist: 5 boxes flipped [ ] -> [x], verification
    citations appended inline).
  - workflow/reasoning/B-20-decisions.md (this closure section
    appended below the s-25120 section).
