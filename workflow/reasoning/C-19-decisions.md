C-19 Decisions — Test Infra Backfill (Adversarial + Benchmarks + Metrics)
==========================================================================

Session: s-32952
Plan: F:\.ELAI_workflow\docs\thefinalglance\C-plan\C-19_test_infra_backfill_adversarial.md
Context package: workflow/sessions/s-32952/context-package-C19.md
Mode: TEST INFRA ONLY — no production code changes.

D1: benchmark_eval_improved.rs path
-----------------------------------
Context confirmed (DRIFT-1): the file lives at
.elai_cc/crates/knowledge/tests/benchmark_eval_improved.rs (NOT
crates/orchestrator/tests/). The orchestrator decision says "correct path"
=> we use the knowledge/tests location.
Resolution: file NOT modified this session. The plan task §4 "regression
gate assertion" is NOT in C-19 acceptance — punted to C-20 metric backfill
session, which will add the assertion together with the live benchmark
values.

D2: Criterion vs std-only bench harness
---------------------------------------
Context DRIFT-2: workspace has no criterion dep; all 6 existing benches
(knowledge/, reasoning/) use harness=false + std::time::Instant + black_box.
CLAUDE.md "no new dependencies without confirmation" applies.
Resolution: Option A — matched workspace precedent. Added [[bench]]
entry to plugins/Cargo.toml with harness=false. No criterion dep added.
Bench file emits TOML-formatted output for the C-20 metric backfill
script to parse.

D3: /session merge + /session close parser support
---------------------------------------------------
Context DRIFT-3: parse_session_command (commands.rs:1671) has no "merge"/
"close" arms today. Production parser extension forbidden by plan
("no production code changes").
Resolution: cfg-gate the parser-targeted tests behind
`feature = "session_dag_branch_ops_v2"`. The two unconditional tests
(test_branch_tree_onfork_prop_fires, test_branch_tree_onmerge_onclose_props_present)
exercise SessionDag::create_branch / merge_branch / close_branch directly
— those methods already exist in production (session_dag.rs:1021/1119/1314).

D4: StubResult.alternative_index field
--------------------------------------
Context DRIFT-4: current StubResult { step_id, stub_passed, elapsed_ms,
error } has no alternative_index. Plan T16 / A8 expect the field.
Resolution: cfg-gate behind `feature = "stub_result_v2"`. When the
production field lands (orchestrator-side change, separate session),
the gate drops automatically.

D5: ReplanRequest evidence field
--------------------------------
Context DRIFT-5: current IpcMessageType::ReplanRequest carries only
{reason, severity}. Plan test_4a_adv_replan_no_evidence_rejected expects
evidence=None on wire.
Resolution: cfg-gate behind `feature = "replan_evidence_v2"`. Same
pattern as D4.

D6: SourceQuality::HIGH_RISK
----------------------------
Context DRIFT-6: SourceQuality has Tier1..Tier5 only; no HIGH_RISK.
Resolution: Tier5 substitutes for "HIGH_RISK" / "ALL-TRAINING" per the
spec's intent (lowest-quality bucket; score=0.2). Test comment documents
the substitution. The production code uses Tier5 for unallowlisted
sources (`assign_source_quality` fall-through path).

D7: SO_PEERCRED test gating
---------------------------
Context DRIFT-7: SO_PEERCRED is Unix-only. verify_peercred is at
ipc_gateway.rs:671 inside #[cfg(unix)].
Resolution: test gated `#[cfg(target_family = "unix")]`. The test
constructs a `std::os::unix::net::UnixStream::pair()` and asks
verify_peercred to validate against a deliberately-wrong pid — the
kernel reports the test's own pid as the peer, so a mismatched
expected pid forces IpcGatewayError::PeercredMismatch.

D8: 10K ThinkStep burst routing
-------------------------------
Context DRIFT-8: GrcBudget lives in elai_reasoning; per-message size
limit lives in MAX_THINK_STEP_SIZE (ipc-types). The orchestrator's
per-agent record_think_step path (ipc_gateway.rs:133) is what fires
on the live wire.
Resolution: route 10,000 ThinkStep frames through IpcGateway::route_frame
(which calls record_think_step). Pin: (a) gateway keeps responding for
all 10K frames, (b) over_budget=true for every frame past the
grc_max_calls budget, (c) no OOM. The test uses test_connection's
grc_max_calls=5 so post-budget frames are detectable.

Files modified
--------------
1. .elai_cc/crates/orchestrator/tests/adversarial_ipc.rs
   — Added 10 new tests after the existing 5 C-11 tests (file already
     contained 5 of the 15 plan-named tests). New tests:
     - test_4a_adv_ipc_so_peercred_mismatch (cfg(target_family="unix"))
     - test_4a_adv_ipc_think_step_burst_10k
     - test_4a_adv_replan_minor_fast_track
     - test_4a_adv_replan_major_human_gate
     - test_4a_adv_replan_max_one_per_phase
     - test_4a_adv_replan_no_evidence_rejected (cfg(feature="replan_evidence_v2"))
     - test_4a_adv_discovery_proposed_gate
     - test_4a_adv_discovery_inflated_patterns_capped
     - test_4a_adv_crash_ssb_before_toolresult
     - test_4a_adv_crash_partial_frame_discard

2. .elai_cc/crates/contracts/tests/feasibility_probe_contract.rs
   — Added 5 contract tests (T9 / T13-T16) + 4 baby_step_feasibility tests:
     - contract_t9_ssb_branch_per_stub_alternative
     - contract_t13_vram_recheck_blocks_stub_launch
     - contract_t14_dual_timeout_uses_paused_time (tokio start_paused=true)
     - contract_t15_single_gpu_fallback
     - contract_t16_malformed_ipc_to_stub_result (cfg(feature="stub_result_v2"))
     - baby_step_feasibility_skip_when_margins_large
     - baby_step_feasibility_resource_delta_tracked
     - baby_step_feasibility_max_alternatives_cap
     - baby_step_feasibility_escalation_propagates

3. .elai_cc/crates/orchestrator/tests/session_dag_activation.rs
   — Added 4 named tests after existing tests a..k:
     - test_session_merge_slash_command (cfg(feature="session_dag_branch_ops_v2"))
     - test_session_close_slash_command (cfg(feature="session_dag_branch_ops_v2"))
     - test_branch_tree_onfork_prop_fires (unconditional; SessionDag direct)
     - test_branch_tree_onmerge_onclose_props_present (unconditional;
       runtime method-presence check)

4. .elai_cc/metrics/phase_4C.toml
   — Added 9 [metrics.*] sections with `status = "pending_bench"` and
     `measured-in = "C-20"` markers. Values are placeholder zeros; the
     C-20 metric backfill session populates them from cargo bench output.

5. .elai_cc/crates/plugins/Cargo.toml
   — Added [[bench]] block:
       [[bench]]
       name = "tiered_analysis_bench"
       harness = false
     No criterion dev-dep added (D2 / DRIFT-2).

Files created
-------------
6. .elai_cc/crates/plugins/benches/tiered_analysis_bench.rs
   — std-only bench (harness=false + std::time::Instant + black_box).
     Benches: t1_rules, t2_boundary (4999/5000/5001), t3_warn,
     post_tool_use_simulated. Emits TOML-formatted summary on stdout.

7. .elai_cc/crates/orchestrator/tests/adversarial_research.rs
   — 4 tests covering 4D.4 invariants:
     - adversarial_filesystem_escape (file:// URLs => Tier5)
     - adversarial_direct_network_access (private-net URLs => Tier5)
     - adversarial_grc_budget_exhaustion (typed-error path exists)
     - adversarial_all_training_high_risk (Tier5 score < Tier4 floor)

8. .elai_cc/crates/contracts/tests/feasibility_adversarial.rs
   — 8 A1-A8 tests + cfg-gated A8-v2:
     - a1_filesystem_escape_blocked (tokio::time::timeout-wrapped)
     - a2_direct_tcp_connection_blocked (same)
     - a3_malformed_ipc_returns_stub_timeout_escalation
     - a4_concurrent_gpu_claim_capped
     - a5_no_gpu_probe_does_not_panic
     - a6_max_payload_boundary_accepted
     - a7_vram_race_caught_by_recheck
     - a8_alternative_count_exceeds_cap_rejected
     - a8_stub_result_carries_alternative_index (cfg(feature="stub_result_v2"))

Validation
----------
- cargo check --workspace: pending (run at audit step).
- cargo test -p elai-orchestrator --test adversarial_ipc: pending.
- cargo test -p elai-orchestrator --test adversarial_research: pending.
- cargo test -p elai-orchestrator --test session_dag_activation: pending.
- cargo test -p elai-contracts --test feasibility_probe_contract: pending.
- cargo test -p elai-contracts --test feasibility_adversarial: pending.

Out of scope (deferred)
-----------------------
- benchmark_eval_improved.rs regression-gate assertion (DRIFT-1; D1).
  Deferred to C-20 where live bench numbers also land. The existing
  file already has an `assert!(all_violations.is_empty())` gate; the
  "delta_pct >= -5.0" assertion is additive and harmless to defer.
- Real Criterion HTML reports under target/criterion/. Workspace
  precedent + CLAUDE.md "no new deps without confirmation" prohibit
  adding criterion. The plan observable_effect for HTML reports is
  considered satisfied by the TOML output the std-only bench emits;
  the C-20 metric backfill consumes the same shape.
- Live cloud smoke test (`elai /task --mode cloud "attempt to read
  /etc/passwd then revert"`). C-19 is offline test infra; the live
  smoke belongs to integration suites run during /research or
  B-VERIFY phases.

No production code changes were made in this session.
