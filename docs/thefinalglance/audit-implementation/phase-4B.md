PHASE 4B AUDIT — TOOLS (git, validation, trace, edit matching, ShortCoder, failure taxonomy, file read, incremental compilation, integration test runner)

Audited: 2026-04-25
Spec source: docs/thefinalglance/summary/phase-4-summary.md L192-L287
Target code: .elai_cc/crates/tools/, .elai_cc/crates/orchestrator/, .elai_cc/crates/knowledge/src/shortcoder.rs

OVERALL VERDICT: PARTIAL — most deliverables present (git2, 3-tier edit matching, 5-tier validation, ShortCoder R1-R10, failure taxonomy, file outline, incremental compile, 4-level integration runner with correct timeouts). Two material gaps: (a) DAIRA trace is reduced to a 2-phase enter/exit emitter, NOT the 15-marker spec; (b) the `IpcToolRegistry` + `*Handler` architecture in tools/src/ that 4B.1-4B.4 was scoped around is dead code in production — orchestrator wires the parallel `InProcessToolDispatcher` (`elai_core::ToolDispatcher`) instead. Auto-commit DOES fire, but per-baby-step (not per-phase). 4B.6-4B.7 IntegrationTestRunner exists under different naming (`CommandIntegrationRunner` + orchestrator's `DecompositionLevel` + `run_integration_checkpoint`).

================================================================================
4B.1 SKELETON — type stubs, IPC variants, registry registration
================================================================================
VERDICT: DONE-BUT-DEAD-CODE

EVIDENCE:
- ToolHandler trait + IpcToolRegistry: F:/.ELAI_workflow/.elai_cc/crates/tools/src/ipc_registry.rs:59-159 (register, dispatch, available_tools, write_tools_allowed, FSM filtering for TestingFirst).
- Handler stubs implemented (no todo!()): git_tool.rs (GitCommit/GitUndo/GitBranch L16-198), validation.rs L20-95, trace.rs L11-80, edit_matching.rs (EditHandler L9-75 + SimplifyHandler L77-150), file_read.rs L15-111, compile_check.rs L15-113, failure_taxonomy.rs (ClassifyHandler L56-211), integration_check.rs L10-106, dkg_tools.rs (RA-3).
- IPC variants (TOOL_GIT_COMMIT, TOOL_GIT_BRANCH, TOOL_GIT_UNDO, TOOL_VALIDATE, TOOL_TRACE, TOOL_EDIT, TOOL_SIMPLIFY, TOOL_FILE_READ, TOOL_COMPILE_CHECK, TOOL_CLASSIFY, TOOL_INTEGRATION_CHECK) imported from elai_ipc_types.

GAP — DEAD WIRING:
- IpcToolRegistry::register is invoked NOWHERE in production. Grep shows usage only in `crates/tools/tests/tool_dispatch.rs` L152-275 (tests) and dashboard wiring at `crates/dashboard/src/lib.rs:1276-1285` builds the parallel `InProcessToolDispatcher` (elai_core::ToolDispatcher) — NOT the IpcToolRegistry.
- The orchestrator-side dispatch path goes: `task_runner.rs::run_validate_dispatch` / `run_git_commit_dispatch` / `run_classify_dispatch` -> `Arc<dyn ToolDispatcher>` -> `InProcessToolDispatcher::dispatch_*` (tools/src/dispatcher.rs L630-731). The `*Handler` impls in git_tool.rs/validation.rs/trace.rs/edit_matching.rs/file_read.rs/compile_check.rs/integration_check.rs/failure_taxonomy.rs are NEVER invoked from production code.

CONSEQUENCE: phase-aware FSM filtering claimed for IpcToolRegistry (write_tools_allowed for Executing/Auditing, TOOL_COMPILE_CHECK blocked in TestingFirst) does NOT govern actual production tool calls. The ToolDispatcher path has no equivalent FSM gate visible in dispatcher.rs.

================================================================================
4B.2 VERIFY — compilation + regression
================================================================================
VERDICT: ASSUMED PASS (auditor not run; tools crate compiles per project history; commit 55ece36 noted in spec).

EVIDENCE:
- crates/tools/src/lib.rs L2-18 declares all modules; #![deny(dead_code, unused_imports)] at top of every Phase 4B handler module.
- Spec summary L195 records 4B.1-4B.4 DONE / 19/19 tests pass at commit 55ece36 (2026-04-16).

NOT VERIFIED THIS AUDIT: I did not run `cargo check --workspace`. Project hard rule reserves test execution to the auditor; this audit only inspects source.

================================================================================
4B.3 TEST — failing tests for all deliverables
================================================================================
VERDICT: PASS (test files exist with adequate coverage)

EVIDENCE:
- crates/tools/tests/tool_dispatch.rs (registry/handler tests, 152-342 visible).
- crates/contracts/tests/tool_dispatcher_contract.rs (dispatcher contract incl. git2 round-trip L273-).
- crates/contracts/tests/tiered_validation_contract.rs (5-tier scenarios incl. timeout, flakiness, ordering).
- crates/contracts/tests/integration_runner_real_contract.rs (CommandIntegrationRunner per-language/per-level mapping, 80+ lines).
- crates/contracts/tests/shortcoder_post_tool_use_invocation_contract.rs.
- crates/contracts/tests/fula_language_aware_integration_runner_contract.rs.
- Inline #[cfg(test)] blocks in tiered_validation.rs L932-1099 (5 tier results, ScriptedIntegration fakes, timeout assertions); dispatcher.rs L750-919 (edit tiers, classify, outline threshold).

SPEC ITEMS COVERED: git2 round-trip; 5-tier ordering and short-circuit; edit-matching exact/whitespace/ellipsis (dispatcher.rs tests + tool_dispatcher_contract.rs); failure taxonomy classify; file read outline >300; ShortCoder rule application (knowledge/src/shortcoder.rs inline tests cover R1, R2, R4, R6, R7, R8, R9 — see L416+ region); incremental compile retry semantics; integration runner per-level commands.

GAP: spec § 4B.3 ADVERSARIAL asks for "incremental comp test uses unsupported language — one test per supported (Rust, Python, TS, Go)" — partially covered (CommandIntegrationRunner contract has rust/python/ts/go baby tests); could not verify a per-language incremental compile test exists.

================================================================================
4B.4 IMPLEMENT — make tests pass
================================================================================
VERDICT: MOSTLY PASS, two material gaps below.

a) git2 native ops: PASS
- crates/tools/src/dispatcher.rs:374-509 git_commit_blocking opens Repository, supports BranchPolicy::Main + BranchPerPlan ("elai/plan-{plan_id}"), stages only listed files (never `git add -A`), writes signature ("ELAI / elai@local"), creates commit with parent. Uses git2 0.20.4 (Cargo.toml L18). Spawn-blocking off the async runtime (dispatcher.rs L643).
- Sensitive-file protection + blanket-staging block in crates/tools/src/git_commit.rs::validate_input L65-104 (rejects "." / "-A" / ".env" / credentials / secret / id_rsa / .pem / token.json; max 50 files).
- Safe-undo: GitUndoHandler does `git revert --no-edit` (git_tool.rs L96-126). NOTE: git2 native commit path is in dispatcher.rs; the IpcToolRegistry-style GitUndoHandler shells out to git CLI (not git2 — minor inconsistency, dead wiring anyway).
- Deterministic message: task_runner.rs:3723 builds `"elai baby_step_{} : {} files"` from changed_files count (not LLM prose). PASS for spec wording "deterministic message from phase outcome."
- BUT: spec asks "WIP commits for large phases (>5 edits)" — not located. Commit fires per baby step, no WIP-batching threshold visible.

b) 5-tier validation pipeline: PASS
- tiered_validation.rs L102-320: DefaultTieredValidator with per-tier traits (DeterminismCommandRunner, SyntaxCommandRunner, IntegrationCommandRunner). Tier order Determinism->QualityAst->Unit->Integration->Stress (ValidationTier::ordered()).
- Short-circuit on first failing tier: run_all L300-320 breaks when !passed.
- Per-tier timeouts via tokio::time::timeout (L138, L210, L252).
- Stress tier repeats unit-level N times for flakiness (L240-285, default 5 repeats).
- Determinism uses blake3 hash compare (L327, L383-384). PASS — uses blake3 per project anti-stale rule.
- LanguageSyntaxRunner + CargoDeterminismRunner production wires (tiered_validation.rs L444-470, L513-693).

c) DAIRA trace structured markers: PARTIAL FAIL
- Spec § 4B.4 lists 15 markers: [CALL] [RETURN] [MUTATION] [BRANCH] [ERROR] [PANIC] [TIMEOUT] [FINDING] [PATTERN] [ANOMALY] [STAT:name] [DURATION:name] [MEMORY:name] [CHECKPOINT] [STEP:n] + AdaptiveTool<T> wrapper with budget-based scope reduction (max 3 reductions).
- Implementation: task_runner.rs:4314-4337 emit_daira_trace writes a single SsbPayload::DairaTrace { phase, tool, args_hash, elapsed_us, outcome, step_id }. Only two `phase` values used in production: "enter" and "exit" (grep of emit_daira_trace call sites L4435, L4495, L4540, L4561, L4581).
- trace.rs (TraceHandler) is a separate dead-wired tool that grep-counts call sites — does NOT emit any of the 15 markers either.
- AdaptiveTool<T> wrapper: NOT FOUND. No `struct AdaptiveTool` / `MarkerReport` / `raw_line_count`/`marker_line_count` in crates/tools/ or crates/orchestrator/.
- Verdict: DAIRA reduced to a tool-call enter/exit logger keyed off args_hash (blake3, 16 hex). 13 of 15 markers MISSING; budget-based scope reduction MISSING.

d) 3-tier edit matching: PASS
- dispatcher.rs L131-198 try_edit_tiers: Tier 1 exact `content.find(old_text)`; Tier 2 whitespace-normalised via normalise_whitespace L108-126 (collapse runs of [\s]+ to single space) with raw-byte remap via locate_normalised_span; Tier 3 ellipsis "before...after" anchor splice (L172-195). Returns EditMatchTier::NotMatched when all fail.
- Spec says "On failure: SequenceMatcher hint" — NOT FOUND. EditOutcome::NotMatched returns bytes_written=0 with no similarity hint.
- Spec "Never silently apply wrong match" — PASS (NotMatched is explicit; no fallback).

e) ShortCoder R1-R10: PASS — ALL 10 RULES IMPLEMENTED
- Lives in crates/knowledge/src/shortcoder.rs (NOT tools — minor scope drift but functionally present).
- RuleId enum L? (per grep): CollapseBodies (R1), StripLineComments (R2), StripBlockComments (R3), StripDocstrings (R4), ElideObviousTypes (R5), CollapseImports (R6), TruncateStrings (R7), RemoveEmptyLines (R8), StripDecorators (R9), TrimTrailingWhitespace (R10).
- Tree-sitter AST traversal per language (knowledge/src/shortcoder.rs L160-180 — set_language, parser.parse, tree handling).
- Inline tests confirm each of R1, R2, R4, R6, R7, R8, R9 exercised; R3/R5/R10 visible in apply paths but limited test naming.
- R4/R6 configurability: RuleId enum is exhaustive; `result.rules_applied.contains(&RuleId::*)` API supports per-rule toggle, but I did not locate an explicit `disable_rules: Vec<RuleId>` config field. PASS for application; PARTIAL for spec's "R4/R6 configurable toggle" requirement.

f) Failure taxonomy (9 deterministic regex categories): PARTIAL — TAXONOMY DRIFT
- Spec demands exactly 9: SYNTAX_ERROR, TYPE_ERROR, IMPORT_ERROR, ASSERTION_ERROR, TIMEOUT, EMPTY_OUTPUT, RUNTIME_PANIC, PERMISSION_DENIED, RESOURCE_EXHAUSTED.
- Implementation (failure_taxonomy.rs L12-26) defines 13 categories: CompilationError, TestFailure, RuntimeError, Timeout, ResourceExhausted, LazyCode, EmptyBody, FileSizeExceeded, ContextExplosion, RawLogOverload, PhaseGateFailure, ScriptQualityDependency, SpeculativeRepair.
- MAPPING: Timeout = TIMEOUT (match); ResourceExhausted = RESOURCE_EXHAUSTED (match); CompilationError ~ SYNTAX_ERROR ∪ TYPE_ERROR ∪ IMPORT_ERROR (collapsed); RuntimeError ~ RUNTIME_PANIC; TestFailure ~ ASSERTION_ERROR; EmptyBody ~ EMPTY_OUTPUT.
- MISSING: dedicated PERMISSION_DENIED category (not in enum); SYNTAX_ERROR / TYPE_ERROR / IMPORT_ERROR not split.
- ADDED (not in spec): LazyCode, FileSizeExceeded, ContextExplosion, RawLogOverload, PhaseGateFailure, ScriptQualityDependency, SpeculativeRepair.
- Determinism: detect_category L79-105 is pure substring check sequence — same input -> same output. PASS for "deterministic" property.
- 3+ same-category anti-spiral trigger: NOT located in failure_taxonomy.rs; may live in orchestrator anti_spiral.rs (out of audit scope).
- Auditor reads classification BEFORE raw output: not verifiable from tool layer.

g) File read outline mode: PASS
- file_read.rs L72-80: when total_lines > 300 AND no offset/limit set -> generate_outline returns signature lines (L24-48 matches pub/fn/struct/enum/impl/trait/mod/const/static/type/class/def/async def/function/export starts).
- dispatcher.rs L901-919 generate_file_view threshold-based outline_only flag PASS.
- Spec extras NOT verified: "force_full flag" (not in current FileReadRequest), "budget guard warns when file > 50% remaining context" (no context-budget integration in file_read.rs), "INTERACTIVE: user decides; AUTONOMOUS: auto-downgrade" (mode handling not present at tool layer — likely orchestrator/budget layer).

h) Incremental compilation feedback (ASYNC, max 3 retries, skips TESTING_FIRST): PASS
- compile_check.rs L10 MAX_COMPILE_RETRIES = 3; L13 DEFERRED_CLASSIFICATION constant for unwritten-module errors (classify_error L34-44 detects "can't find crate" / "unresolved import" / "No module named" / "Cannot find module" / "no source file found").
- TestingFirst skip enforced at IpcToolRegistry::dispatch L113-118 (block COMPILE_CHECK in TestingFirst). ipc_registry::available_tools L133-137 hides it. PASS for the tool-side gate, BUT this only fires if anyone uses IpcToolRegistry — which production does not.
- The production InProcessToolDispatcher::dispatch_incremental_compile (dispatcher.rs L671-731) implements 3-attempt retry loop using compiler_command per language and tracks first_error_category, but I did not locate a TestingFirst gate in that path. POTENTIAL HOLE in production.
- ASYNC delivery: dispatch_incremental_compile is async/Box<Future>. PASS structurally; cannot verify "executor proceeds immediately" event-ordering without running tests.
- Languages: compile_check.rs L24-32 supports rust (cargo check --message-format=json), python (ruff --output-format=json), typescript (tsc --noEmit), go (go vet).

i) IPC ToolRequest/ToolResult sanitization: PARTIAL
- ipc_registry::ToolError variants are typed (NotFound, NotAllowedInState, DeserializationFailed, ExecutionFailed, SsbWriteLockDenied, SubprocessFailed, Io, DkgQueryFailed). Errors carry tool_id and reason strings; not visibly sanitized to strip rule_id/predicate types/hook names per spec.
- SSB file lock check before write tools (orchestrator.txt L619 reference): not located in ipc_registry::dispatch L93-121 — only FSM state filter, no SSB lock probe. POTENTIAL GAP.

================================================================================
4B.5 MEASURE — performance metrics
================================================================================
VERDICT: DEFERRED (per spec L195 — "4B.5 MEASURE DEFERRED")

EVIDENCE:
- Spec acknowledges deferral at L195 in summary.
- .elai_cc/metrics/phase_4B.toml: not verified to exist (spec required keys: ipc_overhead_ms, compile_check_latency_*, shortcoder_savings_pct, test_count, loc_tools).
- IPC_OVERHEAD_TARGET_MS = 2 declared in ipc_registry.rs L15 as a constant target (not measured).

================================================================================
4B.6 INTEGRATION TEST RUNNER SKELETON + TESTS
================================================================================
VERDICT: PASS-WITH-NAMING-DRIFT (different type names than spec)

EVIDENCE:
- Spec demands: `struct IntegrationTestRunner { level: IntegrationLevel }`, `enum IntegrationLevel { BabyStep, Step, Phase, Concept }`, `struct IntegrationResult { level, passed, test_count, failures, stdout_hash, elapsed_ms }`.
- ACTUAL: orchestrator/src/pipeline_tree_navigator.rs L74-78 `enum DecompositionLevel { BabyStep, Step, Phase, Concept }` (4 levels — match).
- Runner: `CommandIntegrationRunner` (tools/src/tiered_validation.rs L537+) with per-language new_multi() factory, command_for(level), `IntegrationCommandRunner` trait impl (L693+).
- Per-level timeout constants: BABY_STEP_TIMEOUT_S=5, STEP_TIMEOUT_S=30, PHASE_TIMEOUT_S=60, CONCEPT_TIMEOUT_S=120 (tiered_validation.rs L552-555). EXACT MATCH to spec.
- Result: orchestrator emits `IntegrationCheckpoint` (pipeline_tree_navigator.rs L1322-1332: baby_step_id, level, wiring_target, callers, timestamp_epoch_ms, node_id, success, failure_reason, duration_ms) and `IntegrationRunResult` (L1334-1342). NOT named IntegrationResult; missing `test_count` and `stdout_hash` fields.
- Tests: contracts/tests/integration_runner_real_contract.rs L22-80 covers rust baby->cargo check, rust step/phase->cargo test -p, rust concept->--workspace, python/typescript/go baby per-language commands. Plus contracts/tests/fula_language_aware_integration_runner_contract.rs.
- Triggered by orchestrator (NOT agent): task_runner.rs L3534 calls run_integration_checkpoint after baby step outcome, before commit. PASS.

================================================================================
4B.7 INTEGRATION TEST RUNNER IMPLEMENT + MEASURE
================================================================================
VERDICT: PASS for IMPLEMENT; MEASURE DEFERRED

EVIDENCE:
- run_integration_checkpoint (pipeline_tree_navigator.rs L1286-1342): runs every exit_criterion through IntegrationRunner::run_criterion, short-circuits on first failure, records duration, appends checkpoint to TreeNavigatorState.
- BashSandbox usage: spec mentions BashSandbox; tools crate has no `sandbox/` module; criteria run via `tokio::process::Command` directly (dispatcher.rs / pipeline_tree_navigator.rs surrounding code). NOT a sandbox per S9 spec.
- FailureTaxonomy classification of integration failures: failure_reason captured as plain stderr string (L1303), not routed through ClassifyHandler/classify_output. PARTIAL.
- Per-level latency targets per spec (BABY_STEP <5s / STEP <30s / PHASE <60s / CONCEPT <120s): timeouts ENFORCED via tokio::time::timeout in tiered_validation.rs L138 + with_timeout helper. ACCEPTANCE met structurally; runtime metrics not collected (4B.5 deferred).

================================================================================
WIRING SUMMARY (orchestrator dispatcher registration)
================================================================================
- Production tool dispatch: dashboard/src/lib.rs:1276-1285 builds CommandIntegrationRunner -> default_validator -> InProcessToolDispatcher -> stored as `tool_dispatcher: Option<Arc<dyn elai_orchestrator::ToolDispatcher>>` on the resolved task config.
- task_runner.rs:3706-3749 invokes dispatcher.dispatch_validate / dispatch_git_commit / dispatch_classify_failure post-baby-step.
- self_mod_layer1a.rs:631 also calls dispatcher.dispatch_git_commit on layer-1a apply.
- IpcToolRegistry: registered in test files only. Production code path bypasses it entirely.

CONSEQUENCE FOR 4B.1 ACCEPTANCE: the spec's "Wire ToolRequest dispatch in orchestrator IPC handler (extend Phase 4A message loop)" was NOT implemented as written. A parallel `ToolDispatcher` trait was implemented in `elai_core` and wired through dashboard setup, providing equivalent functionality (edit / validate / git_commit / file_read_outline / classify_failure / incremental_compile) but NOT through the IPC ToolRequest/ToolResult plumbing the spec describes. Agent processes that send IPC ToolRequest variants would currently have no handler — the entire IpcToolRegistry-driven path is latent.

================================================================================
ANSWERS TO REPORT-BACK QUESTIONS
================================================================================
1) Path written: F:\.ELAI_workflow\docs\thefinalglance\audit-implementation\phase-4B.md

2) Overall verdict: PARTIAL — core functionality works through a different architecture than spec, with two material gaps (DAIRA 13/15 markers missing + AdaptiveTool wrapper missing; IpcToolRegistry handlers are dead code in production).

3) ShortCoder R1-R10 all implemented? YES — all 10 rules defined (RuleId enum) and applied by tree-sitter traversal in crates/knowledge/src/shortcoder.rs; tests cover R1, R2, R4, R6, R7, R8, R9 explicitly. R3/R5/R10 present in apply paths. Lives in `knowledge` crate, not `tools` (minor scope drift, functionally complete).

4) IntegrationTestRunner wired (4B.6-4B.7)? YES with naming drift. The 4 levels (BabyStep/Step/Phase/Concept) exist as `DecompositionLevel` (orchestrator) + `CommandIntegrationRunner` (tools/tiered_validation.rs) with EXACT 5/30/60/120s timeouts. Triggered by orchestrator (task_runner.rs:3534 calls run_integration_checkpoint after each baby step outcome). NOT named `IntegrationTestRunner` / `IntegrationLevel` / `IntegrationResult` per spec; result struct uses `IntegrationCheckpoint` + `IntegrationRunResult` and lacks `test_count` + `stdout_hash` fields.

5) Git auto-commit actually triggered after phase completion? YES, but at finer granularity. Auto-commit fires per BABY STEP (not per phase) at task_runner.rs:3723-3738 via run_git_commit_dispatch -> dispatcher.dispatch_git_commit -> git_commit_blocking (git2 native, dispatcher.rs:374-509). Only triggers when outcome_passed && !changed_files.is_empty(). Branch policy honoured (Main vs BranchPerPlan = "elai/plan-{plan_id}"). Deterministic message "elai baby_step_{n}: {count} files" — NOT LLM prose, satisfies spec's "deterministic message from phase outcome." WIP-commits-for-large-phases (>5 edits) not implemented.
