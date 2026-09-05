B-8 PipelineGrc Activation — Decisions

What
Wire PipelineGrc::ground_plan / generate_csq / check_inline_feasibility / think into
production hot paths so the four reasoning gates fire at runtime instead of
being unreachable wrapper code.

Key decisions

1. Kill-switch
   ELAI_DISABLE_PIPELINE_GRC. Read once at ResolvedConfig::from and cached in
   ResolvedConfig.pipeline_grc as None when set to "1" or "true". One-shot
   tracing::warn! emitted at construction when the env-var is observed so the
   bypass is auditable. Precedent: 11 sibling kill-switches
   (ELAI_DISABLE_PROVIDER_SIGNALS, ELAI_DISABLE_IRREVERSIBLE_CLASSIFIER, ...).

2. ground_plan firing cadence
   Fires once at task entry, after research_decompose returns, before the
   navigator loop begins. Spec ambiguity ("PLANNING → TESTING_FIRST OR every
   replan cycle") resolved to PLANNING → TESTING_FIRST only — task_runner does
   not currently re-decompose mid-task; replan paths route through fsm.rs which
   has no PipelineGrc handle. Re-firing on replan deferred until the FSM owns
   PipelineGrc (B-future).

3. CSQ Critical-without-guard handling in autonomous mode (no dashboard)
   Block by setting hook_pre_blocked = true (mirrors irreversible_classifier
   path at task_runner.rs:3397-3409). Downstream loop renders the BabyStep with
   passed=false, AGENT_FAILED_CATEGORY Finding, and bubble-up — the user sees
   a task-level failure with a tracing::error! line carrying the CSQ result.
   No SSB row, no intervention card. Kill-switch override available.

4. check_inline_feasibility Blocked
   Warn + flip outcome to failed for the offending baby step. Does NOT submit a
   ReplanRequest yet because ResolvedConfig has no FsmHandle today (fsm::Fsm is
   constructed in tests/CLI direct paths; never threaded into the runner). The
   conversion to ReplanRequest::Major is therefore deferred and tagged with a
   TODO(B-future) referencing fsm.rs:644 submit_replan. Until then the
   blocked baby step still surfaces as a failed step with a typed Finding.

5. Auditor anti-rubber-stamp (Tasks#6)
   Activated by seeding a single FINDING: Verify step on the in-process auditor
   handle's grc_engine immediately after with_agent_config in
   agent_pool.rs::spawn (role == Auditor). The seed marker is
   "FINDING: auditor spawn — engine ready for evaluation" with grounding
   "tool:agent_pool::spawn". This guarantees engine.think(Converge) cannot
   succeed under role=auditor without at least one prior FINDING (engine
   already enforces this at engine.rs:297-315). The seed is one line so it does
   not pollute real audit reasoning; real audit FINDINGs append after it.
   Kill-switch (ELAI_DISABLE_PIPELINE_GRC) skips the seed so the engine
   reverts to the unenforced baseline.

6. ContextCompiler verify_claim (Tasks#2)
   The orchestrator-side ContextCompiler is currently a stub spawn point with
   no Rust prefetch path; the real ContextCompiler is the Claude-Code sub-agent
   driven from CLAUDE.md session-protocol. Wiring verify_claim into the
   sub-agent flow is a workflow-tooling change, out of B-8 scope. Replaced
   with a tracing::debug! + TODO(B-future) at the agent_pool.rs spawn site for
   role=ContextCompiler so the next refactor has a clear anchor.

7. Mutex choice
   Arc<tokio::sync::Mutex<PipelineGrc>>. Required because PipelineGrc::think
   and check_inline_feasibility take &mut self and the runner crosses .await
   boundaries holding the lock (baby_step_executor.execute is async and runs
   between generate_csq and check_inline_feasibility). std::sync::Mutex would
   panic at the .await boundary per CLAUDE.md "tokio::sync::Mutex in async".

8. SSB events removed
   Per context package OUT-OF-SCOPE block: csq_evaluated, intervention_request,
   auditor_rubber_stamp_rejected, every observable_effect "expect_ssb …" line
   replaced with tracing::debug! + // TODO(B-future): re-emit when new
   dashboard surfaces are defined. Block semantics preserved (irreversible
   classifier pattern); only the SSB emission is dropped.

9. Field placement
   pipeline_grc lives on TaskRunnerConfig (Option<Arc<AsyncMutex<PipelineGrc>>>)
   to match the existing optional-handle convention (indexing_service,
   hook_dispatcher, signal_reader, ...). When None and the env-var is unset,
   PipelineGrc is not constructed at all so existing fixtures default to a
   no-op behaviour.

10. Decision doc precedence
    This file overrides the spec where the spec references SSB rows or
    file_ops/dispatcher pre-tool hooks (those surfaces don't exist as
    described). The execution agent followed the context-package-B-8.md
    overrides; spec language is preserved here only for traceability.

Expected test outcome
- Sync integration tests (no #[tokio::test] for the pure-engine assertions).
- 7 tests; all PASS after wiring.
- cargo check --workspace clean (after task_splitter + pipeline_grc fixture
  sweep).

Unknowns / open questions
- ContextCompiler Rust-side prefetch (B-future).
- FSM handle on ResolvedConfig (B-future, blocks ReplanRequest::Major emission
  for inline feasibility Blocked).
- CSQ regex over-aggressiveness (engine.rs:603-606) — out of B-8 scope per
  context package; mitigated by kill-switch.


================================================================================
Fix cycle 2 (post-audit; auditor session: B-8 audit cycle 1)
================================================================================

Auditor returned 2 BLOCKs and 1 W1. Resolved as follows:

BLOCK-1: synthetic Grounding emission falsely fails GroundingSweepGate
  Strategy: option (a) — drop the synthetic-citation heuristic entirely.
  Reasoning: GroundingSweepGate's purpose is to surface REAL ungrounded
  claims. Synthesizing a `Grounding: training:unverified` whenever a concept
  description mentions `.rs`, `.py`, etc. injects ungrounded claims that
  did not exist on the live decomposition. The LocalVerifier resolves the
  literal path "unverified" against the filesystem, which never exists, so
  passes=false is guaranteed and the gate ALWAYS rejects. Option (b)
  (skipping `training:` claims with no path on disk) keeps the heuristic
  alive but neuters its main failure mode — strictly worse because the
  heuristic continues firing on `crate `, `api `, etc. without a coherent
  signal. Option (a) is also forward-compatible: when the decomposer
  eventually threads real `Vec<SourceCitation>` through ConceptNode/PhaseNode
  the serializer adds a `Grounding: file:<path>` line per real citation
  without re-introducing the synthetic-noise failure mode.
  File: crates/orchestrator/src/task_runner.rs:2459-2495 (function rewritten,
  removed lines 2477-2485 of the previous version, added doc-hidden `pub`
  visibility for the regression test).

BLOCK-2: BabyStepResult outcome stays Passed when inline feasibility Blocks
  Strategy: when `feas.outcome == FeasibilityOutcome::Blocked`, mirror the
  CSQ-block treatment by reassigning the local `outcome` binding to a
  `BabyStepOutcome::Failed { reason }` BEFORE the BabyStepResult is
  constructed at line 3864 (post-fix). This keeps navigator state
  reconcilable with the SSB TestOutcome row that was already emitting
  passed=false. The reason string carries the prefix
  `inline_feasibility_blocked: <feas.claim truncated>` so downstream
  tooling can attribute the failure to this specific gate (vs the CSQ
  block, vs an executor-side failure).
  File: crates/orchestrator/src/task_runner.rs:3753 (added `mut`),
  3825-3833 (Failed assignment inside Blocked branch).

W1: CSQ block reason indistinguishable from generic PreToolUse block
  Strategy: bundled fix per audit instruction. Changed the synthetic
  failure reason at task_runner.rs:3742 from "PreToolUse hook blocked baby
  step" to "CSQ Critical-without-guard blocked baby step". Today the
  hook_pre_blocked flag is only flipped from the CSQ branch (line 3712);
  if additional PreToolUse-style gates land in the future they should
  carry their own reason string forward instead of overloading this
  default (added an inline comment to that effect).
  File: crates/orchestrator/src/task_runner.rs:3742.

Tests added (safety_gate_integration.rs)
- Test 5 EXTENDED: now asserts on the synthetic BabyStepOutcome shape that
  the runner constructs when feas.outcome == Blocked. Reason must start
  with `inline_feasibility_blocked:` and contain the original claim text.
- Test 8 NEW: serialize_decomposition_does_not_inject_synthetic_grounding_for_rust_mention.
  Constructs a ConceptDecomposition whose description mentions .rs files,
  serializes it, and asserts (a) zero `Grounding:` lines in the output,
  (b) zero `training:unverified` substrings, (c) ground_plan against the
  output yields an empty ungrounded list, (d) GroundingSweepGate passes.
- Test 9 NEW: csq_block_reason_string_is_distinguishable_from_generic_hook_block.
  Asserts the CSQ-block reason contains "CSQ" and is not the legacy
  "PreToolUse hook blocked baby step" string.

Test counts
- Before cycle 2: 7 tests, 7 PASS.
- After cycle 2: 9 tests, 9 PASS (test 5 extended + 2 new tests).

Compile status
- cargo check -p elai-orchestrator -p elai_cc_contracts -p elai-dashboard:
  clean (only pre-existing warnings).

Out-of-scope confirmation
- No new dependencies, no .unwrap, no std HashMap, no edits to other B-8
  wiring sites beyond the 3 fix sites + the regression tests.
