E-47 LoRA ship-gate -- sidecar-side SPEC

Status: GATE LAYER SHIPPED 2026-05-31; producer-side LoRA training stack
RUNPOD-PENDING (Unsloth + QLoRA-NF4 + PiSSA + DoRA on RTX 3090). The
sidecar-side surface specified here is purely DEFERRED -- no .py files
are emitted under python/elai_finetune/ in this E-N; CI lint refuses
.py file creation under that subtree until E-47.1.

Triple-AND ship-gate semantic. The boot-path adapter-load step refuses
to load any LoRA adapter UNLESS all three of the following hold:
  1. `[finetune.lora].enabled = true` in .elai_cc/elai_cc.toml
  2. `[finetune.lora].confirm_user_understanding = true`
  3. `[finetune.lora].ship_gate` is NOT the deferral token
     `DEFERRED_PER_E47_REPORT`
AND a `LoraCalibrationReport` is present and passes the no-regression
floor of -2.0pp on both MMLU and HumanEval deltas vs the base-model
baseline.

Default-deferred posture. The shipped elai_cc.toml carries the
deferral token + confirm_user_understanding=true; this combination
resolves to REFUSED at the gate layer. Operators flip per the runbook
at workflow/decisions/E-47-flip-runbook.md.

No-regression policy. The -2.0pp floor on MMLU and HumanEval deltas is
conservative against the 5-19pp catastrophic forgetting evidence in
arXiv:2405.09673 (Biderman et al.). Either delta below the floor fails
the gate. The producer-side calibration runner (RUNPOD-PENDING) emits
the report; the Rust-side `LoraCalibrationReport` consumes it.

CI lint marker. The string literal `DEFERRED_PER_E47_REPORT` is
load-bearing. Renaming it (typo variants like DEFERED_PER,
deferred_per, etc.) trips the CI lint at
.elai_cc/scripts/check_lora_ship_gate_token.{sh,bat} and the workflow
at .github/workflows/elai-lora-ship-gate.yml.

SSB row. The orchestrator emits one `lora_ship_gate_event` SSB row per
gate evaluation when the env var `ELAI_LORA_SHIP_GATE_ENABLE` is set
to a truthy value. The row carries the 3-arm
`LoraShipGateEventPhase` (refused / calibration_missing /
calibration_passed) plus a free-form details_json with the config
snapshot and (when present) the calibration deltas. Wire-dark default:
the env gate is unset by design.

Sidecar surface (DEFERRED).
  - `python/elai_finetune/dataset.py` -- ProjectProfile-driven
    dataset builder. CI BANNED until E-47.1.
  - `python/elai_finetune/train.py` -- Unsloth FastLanguageModel
    + PEFT LoraConfig (init_lora_weights="pissa", use_dora=True,
    target_modules="all-linear", r=16, lora_alpha=32) on
    Qwen2.5-Coder-7B-Instruct-abliterated (huihui-ai variant per
    CLAUDE.md REMOTE-SIDECAR). CI BANNED until E-47.1.
  - `python/elai_finetune/eval.py` -- in-domain + 200-example OOD
    probes producing the LoraCalibrationReport. CI BANNED until
    E-47.1.
  - `python/elai_finetune/serve_adapter.py` -- vLLM --lora-modules
    pass-through wrapper for the sidecar boot path. CI BANNED until
    E-47.1.
  - `~/.elai/adapters/manifest.toml` -- adapter registry consumed
    by serve_adapter.py. Format DEFERRED.

Anti-patterns. The following patterns are BANNED by design at the spec
layer; the gate ships explicitly to keep them OFF until each is
addressed by E-47.1:
  A. Per-project LoRA training before measured prompt-rules+few-shot
     saturation (CodeRAG-Bench NAACL 2025 shows retrieval > fine-tune
     for code style).
  B. Continuous fine-tuning on rolling user data without an OPLoRA
     orthogonality constraint (catastrophic forgetting).
  C. Cross-project adapter sharing without a project_id +
     base_model_version key (B-37 PerRoleRuntime overlay drift).
  D. Adapter promotion without an OOD probe (Goodhart drift).
  E. Voice-LoRA on per-user data without explicit --consent-yes
     (privacy boundary).
  F. Skipping calibration on "obviously safe" deltas (the -2.0pp
     floor is the only enforcement layer).
  G. Stacking multiple adapters at inference time without an
     aLoRA-style merge contract (arXiv:2512.17910).

Paper grounding.
  * arXiv:2405.09673 Biderman et al. -- 5-19pp catastrophic forgetting
    on code-style LoRA fine-tunes (motivates the -2.0pp floor).
  * arXiv:2305.14314 QLoRA -- NF4 + 4-bit + paged-optimizer baseline.
  * arXiv:2404.02948 PiSSA -- principal-singular-value init.
  * arXiv:2402.09353 DoRA -- weight-decomposed low-rank adaptation.
  * arXiv:2510.13003 OPLoRA -- orthogonality-projected LoRA for
    forgetting mitigation.
  * arXiv:2406.14497 PortLLM -- patch-portable LoRA (nascent).
  * arXiv:2512.17910 aLoRA -- adapter merge contract.

Flip-runbook. See workflow/decisions/E-47-flip-runbook.md for the
3-question gate (Q1 measured prompt-rules+few-shot saturation, Q2 PEFT
defaults still work on the abliterated 7B, Q4 OPLoRA forgetting
mitigation on the ELAI worst-case domain gap) the operator clears
before flipping the ship-gate from Refused to CalibrationPassed.
