I-35 decisions — temporal tool-sequence enforcement
session: s-107b022a  task_id: A109-I35-EXECUTE  date: 2026-06-13

## What
Tool-sequence ordering gate (5th InProcessSupervisor arm). Deterministic evaluation of
an ordered HistoryEntry buffer against TOML-declared rules. CDA-immune because history
rows are built from actual tool-call events, not from token content.

## Key decisions

DECISION-1: History access via push_history_entry Supervisor trait method (default no-op)
  Why: dispatch_pre_tool_use holds Arc<dyn Supervisor>, not Arc<InProcessSupervisor>.
  The only way to push history through the dyn boundary without downcasting is to add a
  default-no-op method to the Supervisor trait. InProcessSupervisor overrides it to push
  to an Arc<Mutex<VecDeque<HistoryEntry>>> internal buffer. Capacity bounded by named const
  HISTORY_BUFFER_CAPACITY=1024.
  Dyn-safety: HistoryEntry is a plain struct (tool_class: String, ts_ms: i64, outcome: String)
  — no generics, object-safe.
  Push site: dispatch_pre_tool_use at the Ok(findings) return (line ~1350), AFTER the
  gate decision, so the buffer reflects completed dispatches.

DECISION-2: HistoryEntryKind as strings in the buffer (not SSB action types)
  The 3 Tier-1 rules need auditor_PASS, test_run, src_edit, untrusted_ingest, git_push,
  file_write, taint_check_PASS as event kinds. These do NOT exist as SSB action types.
  Rather than add 7 new SSB variants for ephemeral session events, tool_class and outcome
  strings map to HistoryEntry fields. SSB variants ship ONLY for ToolSequenceVerdict (the
  enforcement outcome). This is cheaper, cleaner, and keeps SSB as an output bus not an
  input bus.

DECISION-3: Degraded predicates for R1/R2/R3
  R1 (git_push requires auditor_PASS since last src_edit): the auditor_PASS signal is not
  yet emitted into the history buffer by production paths — that would require the auditor
  CLI to push an entry, which is out of scope for I-35. Degraded predicate: requires a
  tool_class="auditor_pass" entry in the buffer (only the bench fixture emits this).
  Documents future evidence source: elai bench/auditor exit path.
  R2 (file_write after untrusted_ingest requires taint_check_PASS): taint_check_PASS is
  not emitted by production paths yet. Degraded predicate: same pattern.
  R3 (test_run PASS before commit-class): test_run not emitted from production test runner.
  Ships DISABLED_WITH_COMMENT — cannot be made meaningful even in degraded form without
  a test-runner push site; disabled is safer than a silent always-pass.
  NONE of these are fail-open: rules without a matching entry in the buffer fail-closed
  (unsatisfied -> OrderViolation) OR are disabled with explicit comment.

DECISION-4: SSB pattern for ToolSequenceVerdict
  Pattern-2 3-arm: Allowed / OrderViolation / HaltEnforced.
  Live baseline at edit time: SCHEMA_VERSION=241, TOTAL_VARIANTS=550.
  I-35 adds 3 -> SCHEMA_VERSION=242, TOTAL_VARIANTS=553.
  Controller corrected baseline: 241/550 (I-33 landed SCHEMA_VERSION=240->241, I-32 landed 241).
  Note: I-32 RoleContractApplied was NOT added to all_test_payloads yet (count still 449
  after I-33's 3 entries). I-35 adds 3 more entries (552 = 449+3). SCHEMA_VERSION=242/553.
  all_test_payloads count: 449 + 3 (I-35) = 452. I-32 gap is a pre-existing issue, not
  within I-35 scope per R-3; only I-35 entries are added.

DECISION-5: include_str! path for rule TOML
  crates/rules/seed/tool_sequence_core.toml, embedded via
  include_str!("../../rules/seed/tool_sequence_core.toml") from tool_sequence.rs.
  The crates/orchestrator/src/ directory is the anchor, so ../../rules/seed/ resolves to
  crates/rules/seed/ — matching the established rule_loader.rs:116 pattern.

DECISION-6: gate-manifest.txt entry
  Executor adds a comment stub + path. Human must re-run gate_freeze_check.py --lock
  OUT-OF-BAND after the commit (R8: a run cannot lock its own gate).

DECISION-7: Frontend pair cross-lang-skip
  ToolSequenceVerdict rows appear automatically in the gate panel via SsbViewer::recent_actions
  (ssb_viewer.rs:1063) — same path as DirectiveExecuted. No new Tauri command needed.
  bench_sequence_probe_cmd.rs adds a cross-lang-skip comment (CLI bench is not a dashboard
  provider).

## Expected test outcomes
  i35 contract suite: 5 arms — compliant-Allowed, violation-Halt, replay-determinism,
    degraded-predicate-honesty, schema-lock (242/553). All must pass.
  i1 contract: 7/7 must stay green (no regression to directive.rs Supervisor trait).
  i32 contract: 7/7 must stay green (RoleContractApplied unchanged).
  action_type_lock: 2/2 at 553/242 after update.
  e45: TOTAL_VARIANTS string updated to 553 in action_type_lock.rs.

## Unknowns
  - Whether I-32 all_test_payloads omission (449 vs expected 452) is intentional or a bug.
    Not within I-35 scope (R-3); only I-35 adds its own 3 entries.
