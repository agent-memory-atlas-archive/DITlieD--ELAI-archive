# C-19: Test Infra Backfill — Adversarial Cases + Criterion Benchmarks
Objective: author all spec-required adversarial tests, Criterion benchmarks, metric
sections, and regression gates that verification reports marked MISSING or PARTIAL
across phases 4A, 4C, 4D, 4F, and B-9 SessionDag.
Estimated effort: 2-3 sessions (no production code changes; test-only writes).
Pre-requisites: C-11 (sandbox completeness — adversarial tests require the sandbox
symbols they exercise to compile); C-22 (feasibility probe completeness — 4F adversarial
tests reference types that may land in C-22; can be written as compile-error stubs before
C-22 ships if needed).
Parallel-safe with: C-18, C-20 (non-overlapping file scope — this plan only touches
tests/ and benches/ directories and one metrics TOML).

## Context

Five verification reports (2026-05-17) flag test gaps that block DONE verdicts:

phase-4A.report.md §PARTIAL: 22 adversarial tests exist; 15 spec-named cases absent.
  Gaps include u32::MAX frame, SO_PEERCRED mismatch, 10K ThinkStep burst, path traversal
  variants, bash/git/symlink self-mod, discovery PROPOSED-gate, MINOR/MAJOR replan
  severity, SSB-before-ToolResult invariant.

phase-4C.report.md §MISSING: no Criterion benchmarks in plugins/benches/; phase_4C.toml
  has qualitative notes only, not the 9 quantified metric categories spec 4C.6 requires.

phase-4D.report.md §PARTIAL: no adversarial_research module; filesystem-escape,
  direct-network-access, GRC-budget-exhaustion, ALL-TRAINING HIGH_RISK cases absent.

phase-4F.report.md §MISSING: A1-A8 adversarial tests (8 required) entirely absent;
  T9/T13-T16 contract tests absent; baby_step_feasibility has 2 of 6 required tests.

B-9_sessiondag_activation.report.md §MISSING: /session merge and /session close slash
  command tests absent; BranchTree onfork/onmerge/onclose prop tests absent.

benchmark_eval_improved.rs has no regression-gate assertion; a delta_pct >= -5.0 guard
must be added so CI catches regressions automatically.

## Reads

- F:\.ELAI_workflow\docs\thefinalglance\verification\reports\steps\phase-4A.report.md
- F:\.ELAI_workflow\docs\thefinalglance\verification\reports\steps\phase-4C.report.md
- F:\.ELAI_workflow\docs\thefinalglance\verification\reports\steps\phase-4D.report.md
- F:\.ELAI_workflow\docs\thefinalglance\verification\reports\steps\phase-4F.report.md
- F:\.ELAI_workflow\docs\thefinalglance\verification\reports\B-plan\B-9_sessiondag_activation.report.md
- .elai_cc/crates/orchestrator/tests/adversarial_ipc.rs (22 existing tests, file structure)
- .elai_cc/crates/orchestrator/tests/research_tree.rs (existing 9 adversarial_ fns)
- .elai_cc/crates/contracts/tests/feasibility_probe_contract.rs (23 existing tests)
- .elai_cc/crates/contracts/tests/plugins_adversarial.rs (10 existing adv01-adv10)
- .elai_cc/metrics/phase_4C.toml (current qualitative content)
- .elai_cc/crates/orchestrator/src/feasibility_probe.rs:141,391,437,454,498 (type shapes)
- .elai_cc/crates/ipc-types/src/lib.rs:104-115 (MAX_FRAME_SIZE, per-type limits)
- .elai_cc/crates/orchestrator/src/session_dag.rs (merge_branch, BranchTree props)
- .elai_cc/crates/cli/src/commands.rs (parse_session_command, existing /session parsing)
- F:\.ELAI_workflow\docs\specs\guide\steps\phase-4A.txt (§4A.4 adversarial spec)
- F:\.ELAI_workflow\docs\specs\guide\steps\phase-4C.txt (§4C.3, §4C.4, §4C.6)
- F:\.ELAI_workflow\docs\specs\guide\steps\phase-4D.txt (§4D.4 adversarial_research)
- F:\.ELAI_workflow\docs\specs\guide\steps\phase-4F.txt (§4F.4 A1-A8, §4F.5 T9/T13-T16)

## Tasks

### Phase-4A adversarial backfill (15 tests)

1. **Add 15 adversarial tests to adversarial_ipc.rs** at
   .elai_cc/crates/orchestrator/tests/adversarial_ipc.rs. Each test carries a comment
   "// Covers audit BLOCK-N / WARN-N" per 4A.4 DETERMINISTIC_CHECK. Tests:
   - test_4a_adv_ipc_u32_max_length_rejected: send length=u32::MAX wire bytes, assert
     PayloadTooLarge before any heap allocation (no Vec allocation with 4GB size).
   - test_4a_adv_ipc_so_peercred_mismatch: connect a second process to the same Unix
     socket after agent accepted; assert second connection rejected (SO_PEERCRED check).
   - test_4a_adv_ipc_think_step_burst_10k: send 10,000 ThinkStep messages in a loop;
     assert budget enforced (excess dropped), no OOM, orchestrator still responds.
   - test_4a_adv_selfmod_path_traversal: BashRequest path "../../../metrics.rs"; assert
     realpath canonical comparison rejects traversal attempt.
   - test_4a_adv_selfmod_bash_write_locked: BashRequest targeting a locked file path;
     assert BLOCK verdict returned, no write occurs.
   - test_4a_adv_selfmod_git_commit_locked: GitCommit IPC msg targeting locked path;
     assert rejection before commit issued.
   - test_4a_adv_selfmod_symlink_resolved: symlink pointing to locked path; assert
     realpath resolves symlink and BLOCK applies to the canonical target.
   - test_4a_adv_replan_minor_fast_track: MINOR severity ReplanRequest with evidence;
     assert auditor fast-track review (no INTERACTIVE gate required).
   - test_4a_adv_replan_major_human_gate: MAJOR severity ReplanRequest; assert
     INTERACTIVE gate raised (does not auto-proceed).
   - test_4a_adv_replan_max_one_per_phase: second ReplanRequest from same agent in same
     phase; assert second rejected (1-per-agent-per-phase invariant).
   - test_4a_adv_replan_no_evidence_rejected: ReplanRequest with evidence=None on wire;
     assert rejected (requires evidence field per spec 4A.4 line 218-220).
   - test_4a_adv_discovery_proposed_gate: auditor finding triggers PROPOSED status, NOT
     auto-CANDIDATE; assert PROPOSED emitted before any CANDIDATE transition.
   - test_4a_adv_discovery_inflated_patterns_capped: ReplanRequest with inflated
     violation_patterns (same hash repeated 1000x); assert dedup/cap applied.
   - test_4a_adv_crash_ssb_before_toolresult: verify SSB write is confirmed synchronously
     before ToolResult frame is sent (write ordering invariant).
   - test_4a_adv_crash_partial_frame_discard: SIGKILL mid-frame write; assert partial
     bytes in socket buffer are discarded on fd close, no hang.

### Phase-4C Criterion benchmarks + metrics

2. **Create benches directory and benchmark file** at
   .elai_cc/crates/plugins/benches/tiered_analysis_bench.rs. Add [dev-dependencies]
   criterion = "0.5" and [[bench]] entry in plugins/Cargo.toml (verify criterion version
   in workspace Cargo.lock before adding — Anti-Slopsquatting). Benchmarks:
   - bench T1 with 1000 non-trivial AST_PATTERN rules (asserting p50 < 5ms).
   - bench T2 at file sizes 4999/5000/5001 lines (boundary stress, asserting p50 < 50ms).
   - bench T3 WARN-triggered path (assert p50 < 500ms).
   - bench PostToolUse total latency including ruff subprocess simulation stub.
   - All four groups output Criterion HTML reports under target/criterion/.

3. **Add 9 quantified metric sections to phase_4C.toml** at .elai_cc/metrics/phase_4C.toml.
   Populate with benchmark-derived values after task 2 runs (or document as "pending_bench"
   with placeholder 0s for CI to replace). Required sections: [metrics.t1_latency]
   p50_ms/p95_ms/p99_ms, [metrics.t2_latency], [metrics.t3_latency],
   [metrics.hook_overhead] per_ipc_call_us, [metrics.post_tool_use_latency] p99_ms,
   [metrics.compile_time_delta] seconds, [metrics.test_count] contract/adversarial/total,
   [metrics.loc] added/modified, [metrics.coverage] percent.

4. **Add regression-gate assertion to benchmark_eval_improved.rs**.
   Locate file (grep elai.map for benchmark_eval_improved). Add at end of evaluation:
   `assert!(delta_pct >= -5.0, "regression: delta_pct {delta_pct} < -5.0 threshold");`
   This blocks CI if any benchmark regresses more than 5% vs baseline.

### Phase-4D adversarial research module (4 tests)

5. **Create adversarial_research.rs** at
   .elai_cc/crates/orchestrator/tests/adversarial_research.rs. Tests (module name must
   match cargo test --lib -- adversarial_research):
   - adversarial_filesystem_escape: attempt read /etc/passwd via ResearchTree node;
     assert BLOCKED at IPC layer, no file content returned.
   - adversarial_direct_network_access: research node attempts open TCP socket directly;
     assert seccomp BLOCK (IPC-only network path enforced).
   - adversarial_grc_budget_exhaustion: ResearchTree with budget=1; assert forced
     convergence (not infinite loop or hang), ConvergenceStatus returned.
   - adversarial_all_training_high_risk: ALL-TRAINING grounding source assigned to node;
     assert SourceQuality flags HIGH_RISK, node excluded from final ResearchReport.

### Phase-4F adversarial + missing contract tests (12 tests)

6. **Create feasibility_adversarial.rs** at
   .elai_cc/crates/contracts/tests/feasibility_adversarial.rs with 8 adversarial tests
   matching spec 4F.4 A1-A8:
   - A1: filesystem escape via stub (read /etc/passwd); assert IPC BLOCK, no file content.
   - A2: stub attempts direct TCP connection; assert seccomp blocks, StubResult.error set.
   - A3: malformed IPC frame from stub; assert orchestrator returns StubTimeout escalation.
   - A4: concurrent GPU claim by two stubs simultaneously; assert no VRAM over-allocation.
   - A5: no-GPU probe (GPU count=0); assert decide_with_mode returns ProceedWithConstraints
     or Escalate, not panic.
   - A6: stub sends MAX_FRAME_SIZE byte payload; assert accepted (boundary test, not reject).
   - A7: VRAM changes between probe and stub launch (race); assert re-check catches drop,
     Escalate returned (not silent proceed).
   - A8: stub alternative count exceeds MAX_ALTERNATIVES_PER_STEP=3; assert 4th rejected.
   Each test includes adversarial stub (e.g., infinite loop stub) caught within 30s harness.

7. **Add T9/T13-T16 contract tests to feasibility_probe_contract.rs** at
   .elai_cc/crates/contracts/tests/feasibility_probe_contract.rs (append to existing 23):
   - T9: SSB branch created per stub alternative with tag="branch:stub_{step_id}_{n}";
     assert 3 alternatives produce 3 SSB branch rows.
   - T13: VRAM re-check (ModelPool.available_vram()) fires before each stub launch; mock
     pool returns lower VRAM than threshold; assert Escalate returned.
   - T14: dual timeout paths — graceful_ack within 25s returns StubResult; hard kill at
     30s returns StubTimeout escalation (use tokio::time::pause for determinism).
   - T15: single-GPU fallback — ProceedWithConstraints returned when only 1 of 2 required
     GPUs available; assert constraint_result has passed=false, decision != Escalate.
   - T16: malformed IPC frame synthesized into StubResult; assert StubResult.error is
     populated, alternative_index is 0, no panic.

8. **Add 4 additional baby_step_feasibility tests** to feasibility_probe_contract.rs
   (reaching >= 6 total from current 2):
   - baby_step_feasibility_skip_when_margins_large: baby step with all margins >= 20%;
     assert skip_reason set, probe short-circuits.
   - baby_step_feasibility_resource_delta_tracked: run stub, assert StubResult carries
     non-zero resource_delta (vram_before != vram_after).
   - baby_step_feasibility_max_alternatives_cap: 4 alternatives submitted; assert 4th
     rejected by alternative_within_cap.
   - baby_step_feasibility_escalation_propagates: baby step escalation surfaced in parent
     FeasibilityReport.escalations list.

### B-9 SessionDag named tests (4 tests)

9. **Add 4 named tests to session_dag_activation.rs** at
   .elai_cc/crates/orchestrator/tests/session_dag_activation.rs (append after test k):
   - test_session_merge_slash_command: construct parse_session_command input "/session
     merge branch-abc"; assert SlashCommand::Session variant parsed with merge action and
     branch_id="branch-abc"; assert no parse error.
   - test_session_close_slash_command: same pattern for "/session close branch-abc"; assert
     close action parsed.
   - test_branch_tree_onfork_prop_fires: construct BranchTree component event simulation;
     dispatch onfork event; assert callback receives branch name string (unit test of prop
     contract, not Svelte render).
   - test_branch_tree_onmerge_onclose_props_present: assert BranchTree struct/config
     exposes onmerge and onclose callback fields (compile-time prop presence check).
   Note: Svelte prop tests are authored as Rust compile-time contract assertions against
   the Tauri command signatures (fork_session_branch, merge_session_branch,
   close_session_branch exist in commands.rs), not Svelte render tests.

## WIRING_CONTRACT

produces_symbols:
  - 15 test fns in adversarial_ipc.rs (test_4a_adv_ipc_u32_max_length_rejected .. 
    test_4a_adv_crash_partial_frame_discard)
  - tiered_analysis_bench.rs Criterion benchmark groups (t1_rules, t2_boundary, t3_warn,
    post_tool_use)
  - 9 TOML metric sections in phase_4C.toml
  - regression-gate assert in benchmark_eval_improved.rs
  - adversarial_research.rs module with 4 tests
  - feasibility_adversarial.rs with 8 tests (A1-A8)
  - 5 contract tests in feasibility_probe_contract.rs (T9/T13-T16)
  - 4 baby_step_feasibility tests in feasibility_probe_contract.rs
  - 4 named tests in session_dag_activation.rs
consumes_symbols:
  - IpcMessageType::ReplanRequest, MAX_FRAME_SIZE (crates/ipc-types/src/lib.rs)
  - SelfModificationLocks realpath (crates/orchestrator/src/ipc_gateway.rs)
  - GrcBudget, ThinkStep (crates/orchestrator/src/agent_pool.rs)
  - FeasibilityProber::probe, decide_with_mode, alternative_within_cap, MAX_ALTERNATIVES_PER_STEP
    (crates/orchestrator/src/feasibility_probe.rs)
  - StubResult, StubTimeout (crates/orchestrator/src/feasibility_probe.rs:141)
  - fork_session_branch, merge_session_branch, close_session_branch (dashboard-v2 commands.rs)
  - parse_session_command (crates/cli/src/commands.rs)
  - T1Hook, T2Hook, T3Hook, IpcHookDispatcher (crates/plugins/src/tiered_analysis.rs,
    crates/plugins/src/ipc_hooks.rs)
  - ResearchTree, ConvergenceStatus, SourceQuality (crates/orchestrator/src/research_tree.rs)
reachable_from:
  - cargo test -p elai-orchestrator --test adversarial_ipc (4A adversarial suite)
  - cargo test -p elai-orchestrator --test adversarial_research (4D adversarial suite)
  - cargo test -p elai-orchestrator --test session_dag_activation (B-9 named tests)
  - cargo test -p elai-contracts --test feasibility_adversarial (4F A1-A8)
  - cargo test -p elai-contracts --test feasibility_probe_contract (T9/T13-T16 + baby_step)
  - cargo bench -p elai-plugins (Criterion benchmarks)
stub_ban:
  - No production code changes. Tests must compile against existing production APIs only.
    If a type is missing (e.g., StubResult.alternative_index), gate the test with
    `#[cfg(feature = "stub_result_v2")]` and document the compile-gate in a comment.
  - Adversarial stubs (infinite loop, malformed IPC) MUST be wrapped in tokio::time::timeout
    or std::thread::spawn with join timeout; no test may hang the cargo test runner.
observable_effect:
  trigger: cargo test -p elai-orchestrator --test sandbox_tests 2>&1
  expect: all 15 new adversarial cases pass
  trigger: elai /task --mode cloud "attempt to read /etc/passwd then revert"
  expect: selfmod_path_traversal catches attempt; no SANDBOX_VIOLATION row in SSB;
          cloud Sonnet 4.6 receives BLOCK verdict via hook, task aborted cleanly
  trigger: cargo bench -p elai-plugins 2>&1 | grep "T1\|T2\|T3"
  expect: T1 p50 < 5ms, T2 p50 < 50ms, T3 p50 < 500ms displayed in Criterion output
  trigger: cargo test -p elai-contracts --test feasibility_adversarial 2>&1
  expect: A1-A8 all pass; infinite-loop stub caught within 30s timeout

## DETERMINISTIC_CHECK

```bash
# 1. 15 new adversarial tests present in adversarial_ipc.rs
grep -c "fn test_4a_adv_" .elai_cc/crates/orchestrator/tests/adversarial_ipc.rs
# Expected: >= 15 (was 22; new total >= 37)

# 2. Criterion benches directory and file exist
test -f .elai_cc/crates/plugins/benches/tiered_analysis_bench.rs && echo OK

# 3. 9 metric sections in phase_4C.toml
grep -c '^\[metrics\.' .elai_cc/metrics/phase_4C.toml
# Expected: >= 9

# 4. Regression-gate assert in benchmark_eval_improved.rs
grep "delta_pct >= -5.0" .elai_cc/crates/orchestrator/tests/benchmark_eval_improved.rs

# 5. adversarial_research module with 4 tests
grep -c "fn adversarial_" .elai_cc/crates/orchestrator/tests/adversarial_research.rs
# Expected: 4

# 6. A1-A8 adversarial tests in feasibility_adversarial.rs
grep -c "^fn a[0-9]_\|^async fn a[0-9]_" .elai_cc/crates/contracts/tests/feasibility_adversarial.rs
# Expected: 8

# 7. T9/T13-T16 in feasibility_probe_contract.rs
grep -E "fn (t9|t13|t14|t15|t16)_|fn contract_t(9|13|14|15|16)" \
    .elai_cc/crates/contracts/tests/feasibility_probe_contract.rs
# Expected: 5 matches

# 8. baby_step_feasibility >= 6 tests total
grep -c "baby_step_feasibility" .elai_cc/crates/contracts/tests/feasibility_probe_contract.rs
# Expected: >= 6

# 9. B-9 named tests in session_dag_activation.rs
grep -c "fn test_session_merge\|fn test_session_close\|fn test_branch_tree" \
    .elai_cc/crates/orchestrator/tests/session_dag_activation.rs
# Expected: 4

# 10. Workspace compiles after all adds
cargo check --workspace 2>&1 | tail -3
# Expected: 0 errors
```

## ACCEPTANCE

- [ ] 15 adversarial tests added to adversarial_ipc.rs; each has "// Covers audit" comment
- [ ] test_4a_adv_ipc_u32_max_length_rejected: PayloadTooLarge returned, no alloc
- [ ] test_4a_adv_ipc_think_step_burst_10k: 10K burst dropped without OOM
- [ ] test_4a_adv_selfmod_path_traversal catches ../metrics.rs traversal attempt
- [ ] test_4a_adv_replan_major_human_gate asserts INTERACTIVE gate raised
- [ ] Criterion benchmark file created; T1 p50 < 5ms, T2 p50 < 50ms, T3 p50 < 500ms
      (or document why target not met with measured value)
- [ ] phase_4C.toml has all 9 [metrics.*] sections with numeric values
- [ ] regression-gate assert in benchmark_eval_improved.rs; re-run confirms delta_pct >= -5.0
- [ ] adversarial_research.rs: 4 tests pass; filesystem-escape BLOCKED at IPC layer
- [ ] feasibility_adversarial.rs: A1-A8 pass; infinite-loop stub caught within 30s
- [ ] T9/T13-T16 contract tests pass; T14 uses tokio::time::pause (no wall-clock wait)
- [ ] baby_step_feasibility test count reaches >= 6
- [ ] B-9: test_session_merge_slash_command and test_session_close_slash_command compile
- [ ] B-9: test_branch_tree_onfork_prop_fires passes (prop contract asserted)
- [ ] cloud smoke test: elai /task --mode cloud "attempt to read /etc/passwd then revert"
      selfmod_path_traversal catches attempt; no SANDBOX_VIOLATION in SSB; Sonnet 4.6
      receives BLOCK; cargo test -p elai-orchestrator --test sandbox_tests shows 15 pass
- [ ] 4F feasibility probe: run with adversarial stub (infinite loop); assert A1-A8 catch
      each failure mode without escape; all caught within timeout
- [ ] cargo check --workspace: 0 errors after all new files added

## Files touched

- MODIFY: .elai_cc/crates/orchestrator/tests/adversarial_ipc.rs (add 15 tests)
- ADD: .elai_cc/crates/plugins/benches/tiered_analysis_bench.rs (Criterion benchmarks)
- MODIFY: .elai_cc/crates/plugins/Cargo.toml (add [[bench]] + criterion dev-dep)
- MODIFY: .elai_cc/metrics/phase_4C.toml (add 9 quantified metric sections)
- MODIFY: .elai_cc/crates/orchestrator/tests/benchmark_eval_improved.rs (regression gate)
- ADD: .elai_cc/crates/orchestrator/tests/adversarial_research.rs (4 tests)
- ADD: .elai_cc/crates/contracts/tests/feasibility_adversarial.rs (8 A1-A8 tests)
- MODIFY: .elai_cc/crates/contracts/tests/feasibility_probe_contract.rs (T9/T13-T16 + 4 baby_step)
- MODIFY: .elai_cc/crates/orchestrator/tests/session_dag_activation.rs (4 B-9 named tests)
- CREATE: workflow/reasoning/C-19-decisions.md

## Next

C-20 (metrics completeness sweep) or C-22 (feasibility probe completeness — after C-22
ships the missing type fields, re-run the 4F adversarial tests without compile-gate flags).
