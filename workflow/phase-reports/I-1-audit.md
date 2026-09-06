PROGRESS SUMMARY: Tests: 7/7 I-1 contract | 2/2 action_type_lock | 9/9 i8 | 11/11 h39 | 14/14 e45 | 5/5 prod_rpc | 7/7 conformal (from bg) | 0 compile errors | Tiers: cargo check PASS | Regression: g2_replay_verify_contract (pre-existing, attributed I-45 / s-107b022a — ObjectiveHashMismatch new arm fires on flipped-byte path) | BLOCKs: 2 | Score: 0.72

------------------------------------------------------------------------
BLOCK (must fix before plan completion):
------------------------------------------------------------------------

BLOCK-1 (R6 / WIRING_CONTRACT reachable_from violated — wire-dark execute_directive):
  directive::InProcessSupervisor::execute_directive has ZERO production call sites.
  The WIRING_CONTRACT states:
    reachable_from: crates/orchestrator/src/task_runner.rs (tool dispatch path) -> Supervisor::execute_directive
  Grep of entire crates/ tree (excluding tests, cfg(test), comments, ssb.rs doc): 0 results.
  task_runner.rs does not call execute_directive anywhere.
  The Executed/Discarded/CompensationFired SSB rows can only be produced by the contract test
  (i1_d2_execute_directive_lands_outcome_rows), not by any production entrypoint.
  Per wiring-enforcement R6: producer-without-production-consumer = build break.
  Per wiring-enforcement R7: test-file callers do NOT satisfy production reachability.
  The D5 compensation closure (the generic compensate: FnOnce arg) is also never constructed
  in production — the compensation path is 100% wire-dark today.
  Remedy: wire execute_directive into task_runner.rs tool dispatch (or the run_subcommand tool
  execution closure) with a real e44_run_saga_step compensate closure. This is Tier-1 scope per the
  plan's "reachable_from: task_runner.rs" contract; it is not a Tier-2 deferral.

BLOCK-2 (double-gate arm i1_b vacuous — does not catch the regression it claims to pin):
  i1_b_exactly_one_gate_fires_per_op checks that e30-argv-deny fires exactly once.
  Mental mutation: remove the `supervised_gate.is_none() &&` guard on line 1177 of hook_dispatcher.rs.
  Result: the inline E-30 Deny arm hits FIRST and returns early via `return Ok(vec![...])` at
  line 1194. The supervisor path at line 1237 is never reached. The test would observe
  count==1 (from the inline path) and PASS — a false pass.
  The assert catches duplicates but NOT the regression where the inline path fires instead of the
  supervisor. The test does NOT mutually-exclude "supervisor-only gate fired" from
  "inline-only gate fired". A test named DOUBLE-GATE that passes when the supervisor is silently
  bypassed is a vacuous arm (wiring-enforcement R1 missing-evidence = FAIL).
  Remedy: add a second assertion that the Block finding's rule_id came from the supervisor path
  specifically (e.g. the supervisor's run_gates was called — test via a spy/mock supervisor that
  records invocations), OR restructure so the test mutates the inline guard and verifies the test
  then FAILS. Currently this arm cannot detect the guard removal regression.

------------------------------------------------------------------------
WARN:
------------------------------------------------------------------------

WARN-1 (R9 — supervisor construction is unconditional but env-gating is possible at call sites):
  build_production_hook_dispatcher constructs the supervisor unconditionally (runtime_setup.rs:1239).
  No kill-switch / env gate. Per plan: "Default-ON (enforcement plane is mandatory per CANT-WRECK;
  no kill-switch)". This is CORRECT per plan intent. However, action_type_lock.rs comment at line
  698 says "Default-ON (enforcement plane is mandatory per CANT-WRECK; no kill-switch)" — wire-dark
  allowlist.toml therefore needs NO row (kill-switch exemption). Verified: no row was added. CORRECT.
  But the three bootstrap sites (run_subcommand.rs:376, task_cmd.rs:431, src-tauri/lib.rs:1771) all
  route through build_production_hook_dispatcher. i1_c scans only for build_production_hook_dispatcher
  and runtime_setup.rs .with_supervisor presence — both confirmed present. WARN: the scan is
  source-grep (R5 unsound); SCIP call-graph edges should confirm all three sites actually call the
  function at runtime. Filed as WARN (not BLOCK) because the grep is deterministic and file contents
  confirmed.

WARN-2 (analyze() still called even under supervised path for non-blocked directives):
  When a supervisor is installed and gate.blocked == false, the code at line 1249 calls
  self.analyze(&invocation). This analyze() call is the tiered-analysis backend (TieredAnalysis).
  The supervised gate findings are then ORed into the vibe_findings path (line 1276-1278).
  This means the tiered-analysis path runs independently from the supervisor gate on every
  non-blocked invocation. This is intentional per the architecture (the tiered backend is orthogonal
  to the supervisor gate chain) but creates a surface where analyze() could produce Block findings
  that the supervisor's run_gates PASSED. This is not a regression but worth documenting: the
  supervisor is NOT the sole gate path for the tiered-analysis dimension.

------------------------------------------------------------------------
REPLACE CORRECTNESS:
  When supervisor is Some: E-30 inline path guarded off by `supervised_gate.is_none()` (line 1177);
  F-1 VibeGate guarded off by the `match supervised_gate` at line 1276. G-3 blast-radius runs inside
  the supervisor's run_gates (new; was never inline). F-9 dep_writetime_gate runs inline unconditionally
  (correct per plan spec: "F-9 is NOT in the supervisor scope"). Semantics of E-30 Deny: supervisor
  fires via run_argv_gate -> AllowlistVerdict::Deny -> findings.push(Block) -> gate.blocked = true ->
  line 1238 early-returns with gate.findings. Identical observable outcome to inline path. Vibegate
  semantics: supervisor calls evaluate_vibegate in run_vibegate, results in findings; if blocked,
  early-return same as inline path. When None: all three paths (E-30, F-1, F-9) run inline verbatim.
  Rule set preserved with the sole addition of G-3 gates (which were absent before).
  REPLACE CORRECTNESS: PASS with caveat that vibegate_seed_ok synthetic Block under supervised path
  uses the supervisor's findings as the "with_seed_fail" base (line 1284) rather than fresh
  evaluate_vibegate output — the synthetic Block is appended on top, which is correct. No semantic
  drift detected.

DOUBLE-GATE ARM: VACUOUS — see BLOCK-2 above.

------------------------------------------------------------------------
HOSTILE QUESTION ANSWERS (concise):
------------------------------------------------------------------------
Q1 REPLACE: PASS — E-30/F-1/G-3 semantics preserved, F-9 retained inline as specified.
Q2 DOUBLE-GATE: VACUOUS — removing the is_none() guard would not be caught; see BLOCK-2.
Q3 R7 BOOTSTRAP: build_production_hook_dispatcher called unconditionally (no env gate), all 3 sites
   confirmed via source grep. with_supervisor() called at runtime_setup.rs:1254.
Q4 ENFORCEMENT OUTCOMES: Discarded = op closure never called (verified: line 401 returns before op());
   CompensationFired = compensate().await called then emit_row(CompensationFired) (line 414-415);
   outcomes are correct. HOWEVER execute_directive is wire-dark (BLOCK-1) so in production these
   outcomes are never produced at all.
Q5 D5 COMPENSATION CLOSURE: Nobody constructs the closure in production. Tier-2 pending, but the
   WIRING_CONTRACT lists execute_directive reachability as Tier-1 (not deferred). Wire-dark.
Q6 SSB: 3 kinds confirmed discriminated in i1_d. Golden 0151 = "directive_executed" confirmed
   alphabetically correct (0150=differential_divergence, 0152=discovery_ratified). TOTAL_VARIANTS=548.
Q7 CRASH-SEAM FIX: i1_d2 executor self-fixed the out-of-root fs-gate test bug by changing the
   crash-arm fixture to use `in_root = dir.path().join("x.rs")` (inside worktree root) so G-3 passes
   and the op runs then crashes. check_fs_target itself is NOT weakened — the production gate still
   fires correctly on out-of-root paths (confirmed: i1_d's "fs_escape_path_blocks_via_g3" unit test
   in directive.rs passes). Fix is correct.
Q8 ZERO-LAZY + MAINTAINABILITY: maintainability_check.py reports clean (0 blocks, 0 warns).
   grep for .unwrap()/.expect() in directive.rs outside #[cfg(test)]: NONE found. Named constants
   present (BASH_TOOL_USE_ACTION, HOOK_EVENT_CATEGORY, SANITIZED_BLOCK_MESSAGE). Fn lengths all
   under 40 lines. No .unwrap() violations.

------------------------------------------------------------------------
TEST SUITE COUNTS:
------------------------------------------------------------------------
i1_directive_supervisor_contract: 7/7 PASS
action_type_lock: 2/2 PASS (1 ignored: regen helper)
e45_cross_language_drift_contract: 14/14 PASS (was 14, confirmed in run)
h39_abstention_score_contract: 11/11 PASS (was 7, now reporting 11 in this run -- all pass)
i8_voluntary_abort_contract: 9/9 PASS
production_agent_rpc_system_prompt_contract: 5/5 PASS
conformal_refusal + conformal_validation: 8/8 PASS (from bg run)
g2_replay_verify_contract: 3/4 PASS — 1 FAIL pre-existing, attributed to I-45 (s-107b022a)
  single_flipped_byte_on_disk_returns_tampered: flipped byte routes to ObjectiveHashMismatch arm
  rather than Tampered, triggering the I-45-added panic. Not I-1 scope.
cargo check --workspace --tests: PASS (28 warnings, 0 errors)

------------------------------------------------------------------------
WIRING SELF-CHECK:
1. Real entrypoint ran: YES for run_gates (via dispatch_pre_tool_use); NO for execute_directive.
2. Off-agent artifact: contract test i1_d2 emits SSB rows — but it's a test-file caller (R7 violation).
3. Missing evidence treated as FAIL: R1 applied above for execute_directive wire-dark.
4. Gate frozen: gate not modified by this run. Gate freeze check not run (read-only audit).
5. No default-OFF flag: supervisor installed unconditionally. Correct.

------------------------------------------------------------------------
VERDICT: BLOCK (Score: 0.72)
  BLOCK-1 is the load-bearing finding: execute_directive is 100% wire-dark, the WIRING_CONTRACT
  reachable_from requirement for task_runner.rs is unmet, and the compensation path has no production
  construction. All three I-1 SSB outcome kinds (Executed/Discarded/CompensationFired) are produced
  only in a contract test, not in production.
  BLOCK-2 is a test-quality finding: the double-gate pin test does not catch the regression it
  was authored to catch.
  Both must be fixed before I-1 can be declared SHIPPED_WIRED.
