I-22 progress-shaped rewards — executor decisions

WHAT: delta-progress reward scorer (search/score.rs): phi(h_t) - phi(h_{t-1}) from
deterministic SSB observables; first-error truncation via H-49 fork API; no-progress
decay; +ProgressRewardScored SSB action_type (3-arm); metrics/progress_reward_weights.toml
ID-5 baseline. Wire: dispatch_progress_reward in task_runner::drive_task after dispatch_tri_layer_verdict.

KEY DECISIONS (all binding from A189 controller fork resolutions):

D-1: PRODUCTION_HOOK = dispatch_progress_reward in task_runner.rs::drive_task, called
     AFTER dispatch_tri_layer_verdict (line ~11386). Follows i-17/i-21 FORK-C precedent
     exactly (the absent i-4 SearchLadder is the PUCT-value consumer; the drive_task hook
     is the live production caller for the task-runner smoke path).

D-2: GATE = ELAI_PROGRESS_REWARD_ENABLE (default-OFF experiment, type=experiment,
     owner=maintainer, expiry_date=2026-12-31, promotion_plan_id=I-22-promotion [FORWARD],
     smoke_test=crates/contracts/tests/i22_progress_reward_contract.rs::i22_a_delta_row_lands_in_ssb).

D-3: SSB = ProgressRewardScored, Pattern-2 single-variant 3-arm Kind:
     Delta / FirstErrorTruncated / DecayApplied. ONE action_type entry, derived counters.
     Wire strings: "progress_reward_delta" / "progress_reward_first_error_truncated" /
     "progress_reward_decay_applied".
     SCHEMA_VERSION 263 -> 264, TOTAL_VARIANTS 572 -> 573 (auto-derived).

D-4: i-37 requirements-covered phi component: plan said DEFER as zero-contribution stub
     (will_resolve_in=i-37). Context package HIGH-2 says i-37 IS SHIPPED. Wire-now per
     controller fork A189 override. EarsRequirementSet::unsatisfied_count() and ::len()
     are the accessors. req_covered = len() - unsatisfied_count() (covered count, not total).

D-5: H-49 fork API for FirstErrorTruncator: no single `UndoTree` struct; compose
     timetravel::capture_file + timetravel::timetravel_enabled() from
     architecture/timetravel.rs (verified live at HEAD). On HardVeto, capture the
     current working tree state as a fork point via capture_file on a sentinel path.

D-6: phi history state: Branch struct carries NO per-step phi history. DeltaProgressScorer
     carries its own StepHistory internally (HashMap<branch_id, PhiHistory>). The
     drive_task path uses a synthetic single-step history (phi_prev = 0.0 for the
     first/only task-runner step). The per-step phi history makes sense only in the i-4
     multi-branch PUCT loop; the task-runner path exercises the emit wire end-to-end.

D-7: WIRE-DARK[I-22] markers on helpers whose ONLY consumer is the absent i-4 PUCT
     value function: the `score_step` method as a PUCT-facing accessor carries the
     marker. The task-runner-facing `dispatch_progress_reward` fn is WIRED (live caller).

D-8: metrics/progress_reward_weights.toml: ABSENT at HEAD (net-new). Written by the
     first-deploy measurement write inside the smoke/fixture path. The no-progress window
     D and weight vector are READ back from this toml at runtime; never hard-coded in
     source (stub_ban / ID-5 compliance). We ship a default toml alongside the code
     (the measurement produces it; the source ships a canonical default).

EXPECTED TEST OUTCOME:
- cargo check --workspace --tests: EXIT 0
- i22 contract: ProgressRewardScored rows in SSB DB, all 3 Kind arms, phi components
  deterministic (test_pass, compile_err_inv, req_covered), broken-arm llm-phi FAILS.
- action_type_lock: PASS after golden regen (regen_golden_file --ignored).
- e45 drift contract: PASS.

UNKNOWNS:
- Whether the stubs.md file needs updating for WIRE-DARK[I-22] lines (yes, it does — 
  one row per marked fn: DeltaProgressScorer::score_step, promotion=I-4).

RECOVERY DECISIONS (A63b dead-executor recovery, 2026-06-14):

R-1: Dead executor left score.rs with `use crate::ssb::payloads::{...}` (private
     module path) and `chrono::Utc::now()` (chrono not a dep). Fixed: import path
     changed to `use crate::ssb::{...}` (public re-export) and ts_ms changed to
     `OsClock.now_ms().try_into().unwrap_or(0)` matching the task_runner convention.
     Added `use elai_core::clock::{ClockProvider, OsClock}` to score.rs.

R-2: dispatch_progress_reward was defined in task_runner.rs but NOT called from
     drive_task. Wired the call after dispatch_tri_layer_verdict (line ~11386) per
     controller fork A189 resolution.

R-3: wire-dark-allowlist.toml had no I-22 row. Added with type=experiment,
     owner=maintainer, expiry_date=2026-12-31, promotion_plan_id=I-22-promotion,
     smoke_test=i22_progress_reward_contract.rs::i22_c_real_db_progress_reward_scored_row_lands.

R-4: Contract test i22_progress_reward_contract.rs was absent. Written fresh; 9/9 pass
     incl real-DB arm (c). The 3 Kind arms (Delta/FirstErrorTruncated/DecayApplied) are
     covered by the round-trip test (d).

R-5: production_caller_check.py flagged compile_err_to_inv and capture_truncation as
     DARK (their only callers are test files). Added WIRE-DARK[I-22] markers to both
     within the 3-line window; added stubs.md rows. Re-check: PASS.

R-6: action_type_lock golden was stale (missing progress_reward_delta). Regenerated
     via regen_golden_file --ignored; golden now has 573 variant lines. Test PASS.

VERIFIED (recovery run):
- cargo check --workspace --tests: EXIT 0 (warnings only)
- i22_progress_reward_contract: 9/9 PASS
- action_type_lock: 2/2 PASS (after golden regen)
- e45_cross_language_drift_contract: 14/14 PASS
- production_caller_check.py: PASS (0 DARK)
