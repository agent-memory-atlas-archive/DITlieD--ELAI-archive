# J-20-EXECUTE-1 decisions

## What
STEP 1 (1/3) of J-20: add the change-point posterior reset (TS-CD,
arXiv:2009.02791) to the non-stationary bandit in the self-mod promotion
path. The discount/sliding-window TS substrate already exists
(darwinian_bai.rs, H-48). STEP 1 adds the missing change-point detector
that RESETS the posterior on a detected task-distribution shift.

## Key decision
- Add `ChangePointDetector` + `detect_and_reset_posterior` to
  `darwinian_bai.rs` (the existing non-stationary bandit module; not a
  new file — H-48 already superseded plain TS here, J-20 extends it with
  the TS-CD reset leg).
- Wire the detector into `darwin_admit_loop.rs::run_one_generation`:
  after `update_arms_from_reward`, run the detector on the pulled arm's
  reward window; on a detected shift, reset the posterior to Beta(1,1)
  and clear the window.
- Telemetry: reuse the existing `BaiArmUpdated` SSB row (GD-1; the reset
  changes alpha/beta which the existing emit already surfaces) + a
  `tracing::warn!` on reset. NO new SSB variant in STEP 1 — the
  SecurityInvariantGate Pattern-2 SSB (with NonStationaryReset kind) is
  a later step; STEP 1 must not touch the frozen SSB 9-tuple lock.
- Algorithm: sliding-window two-sample mean-shift test (deterministic,
  no wall-clock). Per-arm ring buffer of recent rewards; when >=
  MIN_SAMPLES, compare newer-half mean vs older-half mean; if
  |delta| > THRESHOLD, declare change-point + reset. Threshold is a
  named constant, ELAI-calibrated (config knob), NOT the 37.6% literal.

## Expected test outcome
- New unit tests in darwinian_bai.rs: detector fires on a mean shift,
  resets the posterior, does NOT fire on stationary noise.
- cargo check -p elai-orchestrator --features darwinian-evolve: green.
- gate_freeze_check: OK (darwinian_bai.rs / darwin_admit_loop.rs are
  NOT in the gate manifest; edits are invisible to the freeze check).

## Unknowns
- Optimal window size / threshold for ELAI's real task stream — ships
  as config knobs, bounded over >=3 runs per GD-4 (out of scope for
  STEP 1; the loop stays dark behind ELAI_DARWINIAN_LOOP_ENABLE).
