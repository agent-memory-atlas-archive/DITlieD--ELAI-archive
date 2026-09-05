# Verification Report: phase-4B.txt
Source: F:\.ELAI_workflow\docs\specs\guide\steps\phase-4B.txt
Date: 2026-05-17
Overall status: MOSTLY_DONE
Confidence: HIGH

## Summary
Phase 4B's core tool suite (4B.1-4B.4) is implemented: IpcToolRegistry, all typed IPC message structs,
10 tool handler modules, 5-tier validation pipeline, 3-tier edit matching, ShortCoder (in knowledge crate),
failure taxonomy (9-variant), file-read outline mode, incremental compilation with DEFERRED classification,
and SSB write-lock guard. 4B.5 MEASURE is deferred with no metrics file written. 4B.6-4B.7
(IntegrationTestRunner with the spec-exact struct/enum names) are implemented via a different shape
(IntegrationCommandRunner trait + CommandIntegrationRunner in tiered_validation.rs) rather than the
standalone struct IntegrationTestRunner with IntegrationLevel enum — the functional capability exists
but the nominal types differ from the spec. DAIRA trace markers ([CALL], [RETURN], [MUTATION] etc.) and
AdaptiveTool<T> wrapper are absent from tools/src/trace.rs; the file uses a grep-based call chain
approach instead. Paper interfaces (daira-interface.md, shortcoder-interface.md) are present.

## Deliverables verified DONE

- IpcToolRegistry struct with dispatch, available_tools, register — evidence: crates/tools/src/ipc_registry.rs:77
- IpcToolRegistry FSM state filtering (compile_check blocked in TestingFirst) — evidence: crates/tools/src/ipc_registry.rs:113
- SSB write lock error variant SsbWriteLockDenied in ToolRegistry — evidence: crates/tools/src/ipc_registry.rs:36
- ToolHandler trait with execute/tool_id/is_write_tool — evidence: crates/tools/src/ipc_registry.rs:64
- IPC message type GitCommitRequest — evidence: crates/ipc-types/src/lib.rs:373
- IPC message type GitUndoRequest — evidence: crates/ipc-types/src/lib.rs:379
- IPC message type GitBranchRequest — evidence: crates/ipc-types/src/lib.rs:384
- IPC message type ValidateRequest — evidence: crates/ipc-types/src/lib.rs:390
- IPC message type TraceRequest — evidence: crates/ipc-types/src/lib.rs:396
- IPC message type EditRequest — evidence: crates/ipc-types/src/lib.rs:403
- IPC message type SimplifyRequest — evidence: crates/ipc-types/src/lib.rs:410
- IPC message type ClassifyRequest — evidence: crates/ipc-types/src/lib.rs:416
- IPC message type FileReadRequest — evidence: crates/ipc-types/src/lib.rs:422
- IPC message type CompileCheckRequest — evidence: crates/ipc-types/src/lib.rs:429
- DKG query types (DkgCallersRequest, DkgDependentsRequest, DkgDataFlowRequest, DkgSubgraphRequest) — evidence: crates/ipc-types/src/lib.rs:436-455
- git2 native operations (GitBranchPolicy::BranchPerPlan, plan-{id} branch creation) — evidence: crates/tools/src/dispatcher.rs:575
- git2 undo via revert (GitUndoRequest) — evidence: crates/tools/src/git_tool.rs:96-105
- 5-tier ValidationTier enum (Determinism, QualityAst, Unit, Integration, Stress) — evidence: crates/core/src/tool_dispatcher.rs:197-220
- 5-tier validation pipeline (DefaultTieredValidator, run_all, run_tier, short-circuit) — evidence: crates/tools/src/tiered_validation.rs:170-303
- 3-tier edit matching (exact -> whitespace-normalized -> ellipsis) in dispatcher — evidence: crates/tools/src/dispatcher.rs:334 (EDIT_ELLIPSIS_MARKER)
- SequenceMatcher similarity hint on edit failure — evidence: crates/core/src/tool_dispatcher.rs:129-136 (SimilarityHint struct)
- ShortCoder AST-preserving simplification, 10 rules (R1-R10), RuleId enum — evidence: crates/knowledge/src/shortcoder.rs:41-62
- ShortCoder in orchestrator hook post-processing — evidence: crates/orchestrator/src/shortcoder_hook.rs:184
- Failure taxonomy 9-variant FailureCategory enum (SyntaxError, TypeMismatch, UndefinedSymbol, MissingImport, LifetimeError, BorrowError, UnusedWarning, TestAssertion, RuntimePanic) — evidence: crates/tools/src/failure_taxonomy.rs:18-28
- DiagnosticResult struct with RootCause — evidence: crates/tools/src/failure_taxonomy.rs:40-50
- File read enhancement: outline mode for files >300 lines, OUTLINE_LINE_THRESHOLD const — evidence: crates/tools/src/file_read.rs:10-80
- File read force_full flag (dispatch_file_read_with_force_full) — evidence: crates/tools/src/dispatcher.rs:236
- File read budget guard (enforce_file_read_budget_guard, 50% threshold) — evidence: crates/orchestrator/src/task_runner.rs:9081
- Incremental compilation CompileCheckHandler (rust/python/typescript/go) — evidence: crates/tools/src/compile_check.rs:15-113
- DEFERRED_CLASSIFICATION for unresolved imports/unwritten modules — evidence: crates/tools/src/compile_check.rs:13-41
- NEEDS_AUDIT flag after max retries (MAX_COMPILE_RETRIES=3) — evidence: crates/tools/src/compile_check.rs:10-59
- Compile check blocked in TESTING_FIRST (FSM gate) — evidence: crates/tools/src/ipc_registry.rs:113; test: crates/tools/tests/tool_dispatch.rs:208
- TraceHandler (call chain collection via grep) — evidence: crates/tools/src/trace.rs:11-80
- IntegrationCheckHandler for symbol reachability — evidence: crates/tools/src/integration_check.rs:10
- DKG query tools (DkgCallersHandler, DkgDependentsHandler) — evidence: crates/tools/src/dkg_tools.rs:114-234
- DAIRA paper interface ingested — evidence: workflow/papers/daira-interface.md (confirmed present)
- ShortCoder paper interface ingested — evidence: workflow/papers/shortcoder-interface.md (confirmed present)
- Test suite >= 15 tests (19 confirmed: #[test]/#[tokio::test] count = 19) — evidence: crates/tools/tests/tool_dispatch.rs (331 lines)
- Additional tests in failure_taxonomy_9cat.rs, edit_similarity_hint.rs — evidence: crates/tools/tests/
- IntegrationCommandRunner trait (functional equivalent of 4B.6 IntegrationTestRunner) — evidence: crates/tools/src/tiered_validation.rs:42-51
- CommandIntegrationRunner (concrete impl) with run_integration for baby/step/phase/concept levels — evidence: crates/tools/src/tiered_validation.rs:537, 673-765
- Integration runner used by tiered validation pipeline (tiers 3-5) — evidence: crates/tools/src/tiered_validation.rs:195
- Contract tests for integration runner (fula_language_aware, integration_runner_real) — evidence: crates/contracts/tests/fula_language_aware_integration_runner_contract.rs, integration_runner_real_contract.rs

## Deliverables PARTIAL

- DAIRA trace with structured markers ([CALL], [RETURN], [MUTATION], [BRANCH], etc.)
  - exists: crates/tools/src/trace.rs:11 (TraceHandler struct, uses grep-based call chain)
  - gap: TraceHandler does not emit DAIRA structured markers. It calls grep to collect call sites and returns raw lines. No [CALL], [RETURN], [MUTATION], [BRANCH], [ERROR], [PANIC], [TIMEOUT], [FINDING], [PATTERN], [ANOMALY], [STAT:name], [DURATION:name], [MEMORY:name], [CHECKPOINT], [STEP:n] marker output. MarkerReport struct is absent from tools crate. The orchestrator has DairaTraceEntry (crates/tools/src/action_classifier.rs:458) and daira_adaptive_run (crates/orchestrator/src/evidence.rs:646) which are used for audit trail purposes, not the per-trace structured marker output specified.

- AdaptiveTool<T> wrapper with budget-based scope reduction (max 3 reductions)
  - exists: crates/orchestrator/src/evidence.rs:624 (DairaScopeResult<T>) and crates/orchestrator/src/evidence.rs:646 (daira_adaptive_run with DairaScope::Full/Reduced/Minimal)
  - gap: AdaptiveTool<T> generic wrapper is not in tools/ crate as specified. The evidence.rs implementation uses a function rather than a generic wrapper type. The "max 3 reductions" spec maps loosely to the 3-scope enum but is not an explicit configurable retry limit.

- Incremental compilation runs ASYNC (does not block executor)
  - exists: crates/tools/src/dispatcher.rs:981 (dispatch_incremental_compile is a sync loop internally)
  - gap: The implementation runs compile-check synchronously in a pin/async block. No tokio::spawn or background dispatch is evident in the tools/src/dispatcher.rs implementation. The spec requirement (risk-register R51: "run in background while executor works on next file") requires the caller to spawn this — not verified that the orchestrator call site uses tokio::spawn.

- EditRequest tier hint field
  - exists: crates/ipc-types/src/lib.rs:403 (EditRequest with file_path, old_text, new_text)
  - gap: No "tier hint" field in EditRequest struct. Spec 4B.1 calls for "EditRequest with tier hint" — the struct has no tier_hint or similar field.

- FileReadRequest force_full flag
  - exists: crates/ipc-types/src/lib.rs:422 (FileReadRequest: file_path, offset, limit — no force_full)
  - gap: force_full flag is NOT in FileReadRequest. The spec requires "FileReadRequest with force_full flag". The dispatcher has dispatch_file_read_with_force_full (crates/tools/src/dispatcher.rs:236) as a method param, but the IPC struct itself lacks the field, so agents cannot send force_full over the wire.

## Deliverables MISSING

- Metrics file .elai_cc/metrics/phase_4B.toml — searched: powershell Test-Path, elai.map "phase_4B"; file does not exist. Required keys: ipc_overhead_ms, compile_check_latency_rust_ms, compile_check_latency_python_ms, compile_check_latency_typescript_ms, compile_check_latency_go_ms, shortcoder_savings_pct, test_count, loc_tools

- 4B.5 MEASURE step (tool execution latency benchmarks, ShortCoder savings measurement) — searched: "4B.5", "ipc_overhead", "ipc.*overhead"; no benchmark harness found in tools crate. Spec explicitly notes this was deferred.

- Named struct "IntegrationTestRunner" with field "level: IntegrationLevel" as specified in 4B.6 — searched: "IntegrationTestRunner", "IntegrationLevel", "IntegrationResult"; the spec-exact types do not exist. The capability is covered by IntegrationCommandRunner trait + CommandIntegrationRunner struct but the exact API shape (struct IntegrationTestRunner { level }, enum IntegrationLevel { BabyStep, Step, Phase, Concept }, struct IntegrationResult { level, passed, test_count, failures, stdout_hash, elapsed_ms }) from the spec is absent.

- run_integration() function signature matching spec (taking IntegrationSpec from DecompositionTree) — searched: "IntegrationSpec", "run_integration.*IntegrationSpec"; 4B.6 spec gates on Phase 4E types which were deferred.

## Deliverables UNVERIFIABLE

- 4B.4 ADVERSARIAL: two commits with same ClockProvider seed -> identical timestamps (deterministic git2) — reason: would require running tests; cannot verify from static analysis alone
- 4B.4 ADVERSARIAL: async compilation event-ordering test (executor dispatches next ToolRequest before mock compilation completes) — reason: no such test found in tool_dispatch.rs or dispatcher tests; may exist in orchestrator tests not visible here
- Anti-spiral trigger (3+ same-category failures in one phase) — reason: logic may be in orchestrator, not verifiable from tools crate alone

## What must be done (action list)

- [ ] Add force_full: bool field to FileReadRequest in crates/ipc-types/src/lib.rs and wire it in FileReadHandler (crates/tools/src/file_read.rs) to bypass outline threshold
- [ ] Add tier_hint: Option<u8> field to EditRequest in crates/ipc-types/src/lib.rs (4B.1 spec requirement)
- [ ] Implement structured DAIRA markers in TraceHandler (crates/tools/src/trace.rs): emit [CALL], [RETURN], [MUTATION] etc. instead of raw grep lines; add MarkerReport struct with raw_line_count and marker_line_count
- [ ] Add AdaptiveTool<T> generic wrapper in tools crate (crates/tools/src/) with max 3 scope reductions as specified in tools.txt L98
- [ ] Verify tokio::spawn wrapping of dispatch_incremental_compile at orchestrator call site to confirm async (non-blocking) execution per risk-register R51
- [ ] Create .elai_cc/metrics/phase_4B.toml with all required keys once 4B.5 MEASURE is executed
- [ ] Create named types IntegrationTestRunner struct, IntegrationLevel enum, IntegrationResult struct matching the 4B.6 spec signature if phase 4E IntegrationSpec types are now available

## Notes
The phase-4B.txt completion status block (lines 4-10) accurately reflects the implementation state: 4B.1-4B.4
are done with 3 BLOCKs fixed, 4B.5 MEASURE is explicitly deferred, and 4B.6-4B.7 are deferred pending
4E.7+ types. The FailureTaxonomy 9-category list in the implementation (SyntaxError/TypeMismatch/UndefinedSymbol/
MissingImport/LifetimeError/BorrowError/UnusedWarning/TestAssertion/RuntimePanic) differs slightly from the
spec's list (SYNTAX_ERROR/TYPE_ERROR/IMPORT_ERROR/ASSERTION_ERROR/TIMEOUT/EMPTY_OUTPUT/RUNTIME_PANIC/
PERMISSION_DENIED/RESOURCE_EXHAUSTED) — the B-20A decisions file documents this divergence as intentional
(crates/tools/src/failure_taxonomy.rs:14-15).
