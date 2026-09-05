# Verification Report: B-20_tool_registry_unification
Source: F:\.ELAI_workflow\docs\thefinalglance\B-plan\B-20_tool_registry_unification.md
Date: 2026-05-17
Overall status: PARTIAL
Confidence: HIGH

## Summary
B-20's core dispatcher decision was inverted: the plan said "delete InProcessToolDispatcher, make IpcToolRegistry live in production," but InProcessToolDispatcher remains the production dispatcher (called from cli/src/run_subcommand.rs:149,221) while IpcToolRegistry is test-only. The 9-category failure taxonomy (ToolFailureCategory in elai_core) and the tools-crate local FailureCategory in failure_taxonomy.rs are both correctly implemented at 9 variants. Edit similarity hint and force_full file outline with budget guard are both implemented. FSM filtering is on the IPC path (ipc_registry.rs:112-146). The integration_checkpoint_success SSB rows exist. WIP commit squash is absent.

## Claims verified DONE

- 9-category failure taxonomy (ToolFailureCategory) — evidence: crates/core/src/tool_dispatcher.rs:274-284 (9 variants: SyntaxError, TypeMismatch, UndefinedSymbol, MissingImport, LifetimeError, BorrowError, UnusedWarning, TestAssertion, RuntimePanic)
- 9-category FailureCategory in tools crate — evidence: crates/tools/src/failure_taxonomy.rs:18-28 (same 9 variants, matches plan list exactly)
- FSM filtering + TestingFirst gate on IPC path — evidence: crates/tools/src/ipc_registry.rs:112-146 (compile_check blocked in TestingFirst; write tools gated to Executing/Auditing)
- Edit matching SequenceMatcher (Rust: best_line_match) similarity hint — evidence: crates/tools/src/dispatcher.rs:797 (calls best_line_match from crate::string_similarity); test file crates/tools/tests/edit_similarity_hint.rs:64 confirms hint emitted above threshold
- File outline force_full + 50% budget guard — evidence: crates/tools/src/dispatcher.rs:236 (dispatch_file_read_with_force_full); crates/orchestrator/src/task_runner.rs:9068,9081 (FileReadBudgetVerdict + enforce_file_read_budget_guard); crates/orchestrator/tests/file_outline_force_full.rs verifies both behaviors
- integration_checkpoint_success SSB rows emit — evidence: crates/orchestrator/src/ssb.rs:1048 (IntegrationCheckpointSuccess action type); crates/orchestrator/src/task_runner.rs:173 (INTEGRATION_CHECKPOINT_SUCCESS_CATEGORY constant); crates/orchestrator/src/pipeline_tree_navigator.rs:375,1626 (emit after checkpoint); crates/orchestrator/tests/integration_checkpoint_success.rs confirms success and failure paths

## Claims PARTIAL

- IpcToolRegistry live in production
  - exists: crates/tools/src/ipc_registry.rs:77 (struct defined, dispatch + FSM gate implemented)
  - gap: Zero production callers found. Only test callers (crates/tools/tests/tool_dispatch.rs:152,163,190,209,235). InProcessToolDispatcher is still the production dispatcher at crates/cli/src/run_subcommand.rs:149,221. pipeline_builder.rs:327 mentions it only in a comment.

- InProcessToolDispatcher deleted
  - exists: crates/tools/src/dispatcher.rs (still present, live in production)
  - gap: Plan acceptance criterion "InProcessToolDispatcher absent" is NOT met. Symbol has production callers at crates/cli/src/run_subcommand.rs:149,221 and is the live dispatch path.

## Claims MISSING

- WIP commits squashed on phase boundary (AUDITING -> MEMORIZING) — searched: wip_commit, squash_commit, WipSquash, commit_squash, wip.*squash in crates/orchestrator/src/task_runner.rs and all crates; no evidence of implementation

## Claims UNVERIFIABLE

- Auditor classification reader updated to use new 9-category set — the tools-crate FailureCategory (9 variants) is present and the auditor would call TOOL_CLASSIFY, but the auditor agent reads this at runtime via IPC; no auditor-side read path is statically verifiable from crate sources alone

## What must be done (action list)

- [ ] Wire IpcToolRegistry as the production dispatcher: update crates/cli/src/run_subcommand.rs:149,221 to instantiate IpcToolRegistry + register handlers instead of InProcessToolDispatcher
- [ ] Delete or stub-ban InProcessToolDispatcher from production call sites after IpcToolRegistry is wired
- [ ] Implement WIP commit squash: add logic in crates/orchestrator/src/task_runner.rs to squash >5 WIP commits into one phase-level commit on AUDITING->MEMORIZING FSM transition
- [ ] Verify elai_core::mistake_book::FailureCategory (14 variants) is NOT the auditor-facing taxonomy; confirm auditor reads ToolFailureCategory (9 variants) or crates/tools FailureCategory (9 variants) — the 14-variant enum in crates/reasoning/src/traversal.rs:376 indicates a parallel taxonomy still exists and could cause drift

## Notes
Two distinct FailureCategory types exist: elai_tools::failure_taxonomy::FailureCategory (9 variants, B-20 compliant) and elai_core::mistake_book::FailureCategory (14 variants, old pre-B-20 list used by reasoning/traversal). The plan claimed to "reduce 13 to 9" — the tools-crate enum is at 9, but the core enum is still at 14. This dual-taxonomy situation is a latent drift risk even though the tool-facing layer is correct.
The DETERMINISTIC_CHECK bash commands from the plan would fail: rg "InProcessToolDispatcher" .elai_cc/crates/orchestrator/src would return hits (dispatcher referenced in orchestrator tests); rg "IpcToolRegistry" non-test would return no production hits.
