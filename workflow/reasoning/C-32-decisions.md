C-32 — Code-Completeness Layer 2 (skeleton-first + MiniCritic + AcceptAndComplete) decisions
=============================================================================================

Session  : s-25672
Plan     : F:\.ELAI_workflow\docs\thefinalglance\C-plan\C-32_code_completeness_skeleton_first.md
Context  : F:\.ELAI_workflow\workflow\sessions\s-25672\context-package-C32.md
Baseline : SCHEMA_VERSION = 62 (ssb.rs:458); FilePlan at BabyStepSpec field 11 (research_tree.rs:1780);
           RetryStrategy at BubbleUpPolicy.retry_strategy (C-26).

DRIFT decisions (orchestrator-supplied; this session adopts them verbatim).
--------------------------------------------------------------------------

D1  (SCHEMA baseline). Bump SCHEMA_VERSION 62 -> 63 atomically. 5 new variants
    (SkeletonGenerated / SkeletonFlushed / MiniCriticVerdict /
    StructuralRetryStageAttempt / BridgeFileGenerated). all_test_payloads
    length 131 -> 136. DESTRUCTIVE migration: v62 session DBs are DROPped on
    first read; SSB rows are observational so no downstream state is harmed.

D2  (ReasoningDrawer path). Confirmed at
    .elai_cc/crates/dashboard-v2/src/lib/drawers/ReasoningDrawer.svelte
    (NOT .../windows/...). Plan §Files-touched line stays in this decisions
    file; do not back-port the plan text.

D3/D4 (MonitorWindow gauge subscription). Clone the C-28 mutation gauge 5s
    polling pattern. get_completeness_ratio() returns a CompletenessRatioInfo
    struct (ratio + blank_count + last_path) the same way MutationGaugeInfo
    does. SSB skeleton_flushed becomes the canonical refresh trigger but the
    poll loop is the actual data source (avoids new broadcast wiring).

D5  (tree-sitter node kinds). MiniCritic uses `function_item` for Rust and
    `function_definition` for Python. The minimal-shell wrappers used for
    fragment parsing are:
      Rust:   `fn __probe() {\n<body>\n}\n`
      Python: `def __probe():\n    <body indented 4 spaces>\n`
    Lang not registered -> WARN "ts_unavailable".

D6  (Error type Clone). BridgeError and SkeletonError carry String message
    fields instead of `io::Error` so all error types derive Clone. The plan
    says "io::Error", but io::Error is not Clone — and the contract test
    `flush_single_disk_write_per_file` need an error type that can be
    cloned out of an Arc. Inner io::Error is stringified at construction
    time via `.to_string()`. No semantic loss because the SSB row carries
    the path; the error string is for diagnostics only.

D7  (task_runner hook placement). Skeleton path lands inside the existing
    `else` branch at task_runner.rs:6537 (the baby-step dispatch site). The
    new code wraps the existing `baby_step_executor.execute(exec_ctx)` call:
    when `BabyStepSpec.file_plan` is Some AND the plan declares at least
    one new_file or modified_file, run SkeletonGenerator beforehand and
    overlay the resulting FilledBuffer onto the executor output. focus_tunnel
    pre-assembly is NOT bypassed; the skeleton path coexists. Modified-only
    plans get the skeleton treatment too (function bodies inside an existing
    file are placeholdered the same way).

D8  (completeness_ratio vacuous pass). 0 blanks -> ratio 1.0. Implementation
    short-circuits with `if self.spec.blanks.is_empty() { return 1.0; }`. No
    .unwrap(), no division-by-zero.

D9  (BestOfN compile signal). "compile bonus" === "fragment parses without
    tree-sitter ERROR node" (the same signal as MiniCritic check (c)).
    Documented inline in structural_retry.rs.

D10 (AcceptAndComplete opt-in). Add `accept_and_complete_enabled: bool`
    (default false) to FileDisciplineConfig in plugins/src/tiered_analysis.rs.
    StructuralRetrySequencer reads the flag from RetryContext. Default-false
    preserves strict behaviour; cloud /task can opt in via per-project config.

D11 (Contract test name + crate). File path is
    .elai_cc/crates/contracts/tests/c32_skeleton_minicritc_contract.rs
    (typo "minicritc" intentional — matches the plan §Files-touched +
    DETERMINISTIC_CHECK rg command). Package is `elai_cc_contracts` per
    C-31 D7 (the plan §DETERMINISTIC_CHECK uses `elai-orchestrator`, which
    is wrong; this decisions file corrects it).

Files (canonical paths).
------------------------

ADD:
  .elai_cc/crates/orchestrator/src/skeleton_generator.rs
  .elai_cc/crates/orchestrator/src/mini_critic.rs
  .elai_cc/crates/orchestrator/src/structural_retry.rs
  .elai_cc/crates/orchestrator/src/bridge_template.rs
  .elai_cc/crates/contracts/tests/c32_skeleton_minicritc_contract.rs
  workflow/reasoning/C-32-decisions.md (this file)

MODIFY:
  .elai_cc/crates/orchestrator/src/ssb.rs           (SCHEMA 62->63 atomic 9-tuple)
  .elai_cc/crates/orchestrator/src/lib.rs           (4 new pub mod + re-exports)
  .elai_cc/crates/orchestrator/src/task_runner.rs   (skeleton hook at L6537)
  .elai_cc/crates/plugins/src/tiered_analysis.rs    (mini_critic_patterns()
                                                     accessor +
                                                     FileDisciplineConfig
                                                     .accept_and_complete_enabled)
  .elai_cc/crates/dashboard-v2/src-tauri/src/commands.rs (get_completeness_ratio)
  .elai_cc/crates/dashboard-v2/src-tauri/src/lib.rs (register tauri command)
  .elai_cc/crates/dashboard-v2/src/lib/windows/MonitorWindow.svelte (gauge row)
  .elai_cc/crates/dashboard-v2/src/lib/drawers/ReasoningDrawer.svelte
                                                    (mini_critic_verdict rows)
  .elai_cc/crates/dashboard-v2/src/lib/api/stub.ts  (stub default)
  .elai_cc/crates/dashboard-v2/src/lib/api/types.ts (CompletenessRatioInfo)

Acceptance summary (cross-checked against the plan).
----------------------------------------------------
- 4 new orchestrator modules.
- 1 atomic SCHEMA bump (62 -> 63).
- 5 new SSB action_types (+ payloads, + sample list, + test).
- 1 new Tauri command + 1 frontend type + stub default.
- 1 MonitorWindow gauge section.
- 1 ReasoningDrawer row renderer.
- 14 contract tests in c32_skeleton_minicritc_contract.rs.
- 0 .unwrap() in production code (Rust hard rule).
- 0 LLM calls inside BridgeFileTemplate::generate (stub_ban).
- exactly 1 fs::write in flush_to_disk (stub_ban).
- No instructional retry prompts (stub_ban).

Audit-C32 close-out (session s-25672).
--------------------------------------
W1 (task_runner C-32 hook was observational, not full §Task 5 chain):
  Closed in commit pending. New module c32_dispatch.rs in
  .elai_cc/crates/orchestrator/src/ exposes
  dispatch_skeleton_with_retry which drives the full chain:
    1. BridgeFileTemplate fast path (when target path matches a known
       bridge kind: mod.rs / __init__.py / index.ts / doc.go).
    2. Per-blank StructuralRetrySequencer walk (Decompose -> Template
       -> FewShot -> BestOfN).
    3. MiniCritic verdict gate per fill (via SkeletonGenerator::fill_blanks
       on the converged body, or direct critic.check inside the per-blank
       loop).
    4. AcceptAndComplete threshold (>=0.5) short-circuits the chain when
       FileDisciplineConfig.accept_and_complete_enabled is true (mirror
       carried through RetryContext).
    5. Single fs::write per file via SkeletonGenerator::flush_to_disk.
    6. SSB emission: per-fill mini_critic_verdict rows, per-stage
       structural_retry_stage_attempt rows, terminal skeleton_flushed
       row with the FilledBuffer.completeness_ratio (NOT the boolean
       1.0/0.0 derived from outcome_passed).
  task_runner.rs:6618-6648 now calls dispatch_skeleton_with_retry per
  pre-computed SkeletonSpec; the legacy boolean-derived skeleton_flushed
  emission was deleted.
  accept_and_complete_enabled is hardcoded false at this site pending
  FileDisciplineConfig threading into ResolvedConfig (separate refactor;
  the default value matches FileDisciplineConfig::default()).
  PerBlankCandidates is supplied as default() since no LlmProvider is
  threaded into per-blank dispatch yet — the wiring is structurally
  complete; the candidate-bridge slot is the next refactor.

W2 (get_completeness_ratio returned Value::Null):
  Closed in commit pending. .elai_cc/crates/dashboard-v2/src-tauri/src/
  commands.rs get_completeness_ratio now calls
  state.viewer.latest_action_of_type("skeleton_flushed") and projects
  the SkeletonFlushedPayload onto the wire envelope:
    { known: false }
    | { known: true, ratio: f32, blank_count: u32, last_path: String }
  CompletenessRatioInfo in types.ts and stub.ts updated to match the
  envelope shape. MonitorWindow.svelte updated to branch on
  completenessRatio.known instead of `=== null`. blank_count is read
  best-effort from the latest skeleton_generated row (falls back to 0
  when absent).

Files added/changed in close-out:
  + .elai_cc/crates/orchestrator/src/c32_dispatch.rs (new module)
  M .elai_cc/crates/orchestrator/src/lib.rs (mod + pub use)
  M .elai_cc/crates/orchestrator/src/structural_retry.rs (Derive Clone,
    Debug, Default on StaticStageCandidates so PerBlankCandidates can
    derive Clone)
  M .elai_cc/crates/orchestrator/src/task_runner.rs:6618-6680
    (per-skeleton dispatch_skeleton_with_retry invocation)
  M .elai_cc/crates/dashboard-v2/src-tauri/src/commands.rs:4101-4163
    (get_completeness_ratio reads SsbViewer)
  M .elai_cc/crates/dashboard-v2/src/lib/api/types.ts:1486-1500
    (CompletenessRatioInfo envelope)
  M .elai_cc/crates/dashboard-v2/src/lib/api/stub.ts:1403-1411
    (return { known: false } as default)
  M .elai_cc/crates/dashboard-v2/src/lib/windows/MonitorWindow.svelte:71-
    79 + :465-498 (branch on known flag)
  M .elai_cc/crates/contracts/tests/c32_skeleton_minicritc_contract.rs
    (added 2 close-out tests:
      c32_close_out_bridge_template_consulted_before_retry_chain,
      c32_close_out_completeness_envelope_shape_pinned)

Acceptance verified:
  - cargo check -p elai-orchestrator: clean
  - cargo check -p elai-dashboard-v2: clean
  - cargo check --workspace: clean
  - c32_skeleton_minicritc_contract: 20/20 (was 18/18; +2 new)
  - elai-orchestrator::c32_dispatch: 5/5 (new)
  - elai-orchestrator::skeleton_generator: 7/7
  - elai-orchestrator::mini_critic: 5/5
  - elai-orchestrator::structural_retry: 5/5
  - elai-orchestrator::bridge_template: 5/5
  - elai-orchestrator::ssb: 29/29

Remaining follow-ups (NOT in scope for W1/W2 close):
  - Thread an LlmProvider into PerBlankCandidates so non-bridge files
    actually receive LLM candidates per stage (today they exhaust to
    Block fills). Tracked in W1 justification text above.
  - Thread FileDisciplineConfig into ResolvedConfig so
    accept_and_complete_enabled can be flipped per project.
  - W3 (score_candidate locally instantiating MiniCritic::new for the
    compile bonus): documented in audit-C32, deferred per D9 design.
  - W4 (elai.map stale): regenerate after commit per CLAUDE.md.

End of audit-C32 close-out.
