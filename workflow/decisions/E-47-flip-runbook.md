E-47 LoRA ship-gate -- flip runbook

Purpose. The default-deferred posture refuses every LoRA adapter load
behind the triple-AND gate. This runbook documents the steps the
operator follows to flip the gate from Refused to CalibrationPassed
once the RUNPOD calibration baseline is shipped.

Pre-conditions (ALL must hold before flipping)

  Q1 measured saturation gap. The current prompt-rules+few-shot path
     produces a measured 5+pp gap vs the projected LoRA-trained
     baseline on the project-style probe. If the gap is <5pp, prefer
     to lift more context into prompt-rules / RAG before training.

  Q2 PEFT defaults still work on the abliterated 7B. Smoke-train a
     128-example LoRA on Qwen2.5-Coder-7B-Instruct-abliterated
     (huihui-ai variant per CLAUDE.md REMOTE-SIDECAR) with PiSSA +
     DoRA defaults; verify training loss converges and the merged
     adapter loads without head_dim mismatch (cross-check E-31
     HeadDimInvariantVerified row in SSB).

  Q4 OPLoRA forgetting cap on ELAI worst-case domain gap. Run the
     200-example OOD probe (in-domain code-style + out-of-domain
     general MMLU) with the OPLoRA orthogonality constraint enabled.
     Forgetting on the OOD probe MUST be <=2.0pp on both MMLU and
     HumanEval deltas vs the base-model baseline (the gate's no-
     regression floor).

Flip checklist (perform in this order)

  1. Run the calibration probe (RUNPOD-PENDING; producer is
     python/elai_finetune/eval.py, NOT shipped). Output:
     metrics/lora_baseline.toml with mmlu_delta_pct and
     humaneval_delta_pct populated.

  2. Update .elai_cc/elai_cc.toml [finetune.lora] section:
       ship_gate = "BASELINE_VERIFIED_<YYYY-MM-DD>"
       confirm_user_understanding = true
       calibration_baseline_mmlu = <measured>
       calibration_baseline_humaneval = <measured>
       enabled = true

  3. Set the env var to enable SSB emission:
       export ELAI_LORA_SHIP_GATE_ENABLE=1

  4. Run `elai lora-gate-check --json` and verify the outcome is
     `calibration_passed`. If `calibration_missing` or `refused` are
     returned, STOP and re-check Q1/Q2/Q4.

  5. Verify an SSB row of type `lora_ship_gate_event` lands with
     phase=calibration_passed via the dashboard or
     `elai ssb-tail --type lora_ship_gate_event --n 1`.

  6. Commit the toml change + the metrics file in ONE commit (per
     CLAUDE.md commit-cadence rule). Commit message in maintainer voice
     (lowercase i, no em-dashes, no robotic prefixes).

Rollback plan (if a downstream regression is detected)

  R1. Revert the toml change to restore
      `ship_gate = "DEFERRED_PER_E47_REPORT"` +
      `enabled = false`.
  R2. Unset `ELAI_LORA_SHIP_GATE_ENABLE` in the runtime environment.
  R3. Re-run `elai lora-gate-check`; verify outcome is
      `calibration_passed` under wire-dark posture (enabled=false
      short-circuits to passed because the gate is inactive).
  R4. Document the rollback in workflow/reasoning/E-47-decisions.md
      with the measured regression evidence and the commit hash of
      the offending flip.

Out of scope for this runbook

  - The actual LoRA training run (python/elai_finetune/train.py is
    RUNPOD-PENDING; covered by E-47.1).
  - The dashboard LoraShipGatePanel.svelte (deferred to E-47.2;
    not blocking the flip).
  - Any production adapter-load step that consumes the
    LoraCalibrationReport (RUNPOD-PENDING; per_role_runtime overlay
    is the B-37 attach point, deferred to E-47.1).

Reference

  Source report: workflow/improvement-sweep-2026-05-19/reports/47-lora-finetuning.md
  Decisions:     workflow/reasoning/E-47-decisions.md
  Paper:         arXiv:2405.09673 Biderman et al. (5-19pp catastrophic forgetting)
  Spec:          .elai_cc/python/elai_inference/SPEC.md
