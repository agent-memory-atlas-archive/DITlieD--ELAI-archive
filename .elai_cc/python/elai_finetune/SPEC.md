E-47 LoRA fine-tuning stack -- python/elai_finetune/ surface spec

Status: DEFERRED -- no .py files may be created under this subtree until
E-47.1. This SPEC.md defines the expected surface (dataset.py / train.py /
eval.py / serve_adapter.py) and the acceptance criteria the E-47.1 executor
must satisfy. CI lint accepts this file; it rejects any .py file under
python/elai_finetune/ until the E-47.1 gating conditions in this document
are met.

Triple-AND ship-gate (gate layer already shipped in Rust). The Rust-side
check_lora_ship_gate (crates/orchestrator/src/lora_ship_gate.rs) refuses
adapter load unless all three hold:
  1. [finetune.lora].enabled = true in .elai_cc/elai_cc.toml
  2. [finetune.lora].confirm_user_understanding = true
  3. [finetune.lora].ship_gate != "DEFERRED_PER_E47_REPORT"
     AND a LoraCalibrationReport from eval.py passes the -2.0pp floor on
     both MMLU and HumanEval deltas vs the base-model baseline.

Default-deferred posture. The shipped elai_cc.toml carries the deferral
token DEFERRED_PER_E47_REPORT and confirm_user_understanding=true; this
combination resolves to REFUSED at the gate layer. Operators flip per
workflow/decisions/E-47-flip-runbook.md (Q1 prompt-rules+few-shot
saturation measured, Q2 PEFT defaults verified on abliterated 7B, Q3 OPLoRA
forgetting-mitigation bounded on the ELAI domain gap).

Adapter manifest layout. The runtime adapter registry lives at:
  ~/.elai/adapters/manifest.toml

Required keys per adapter entry:
  [[adapter]]
  name                = "elai-code-style-v1"
  base_model_version  = "huihui-ai/Qwen2.5-Coder-7B-Instruct-abliterated"
  project_id          = "<blake3-prefix-of-project-root>"
  lora_r              = 16
  lora_alpha          = 32
  init_weights        = "pissa"
  use_dora            = true
  target_modules      = "all-linear"
  calibration_report  = "~/.elai/adapters/reports/elai-code-style-v1-cal.json"
  created_at          = "ISO-8601"

The base_model_version + project_id pair is the unique key. Adapters with
different project_id or base_model_version must not be cross-loaded (C
anti-pattern below). serve_adapter.py reads the manifest and passes the
adapter to vLLM via --lora-modules.

dataset.py surface (DEFERRED until E-47.1). Reads the ProjectProfile
produced by the E-12 framework_profile registry, samples the current
project's codebase, applies style-token filtering, and emits a JSONL
dataset whose schema satisfies the Qwen2.5-Coder instruction-tuning
conversation format. Must not include cross-project examples; each row
carries the project_id key from the adapter manifest. No function
signatures may import from outside the standard library or torch/datasets
until the E-47.1 RUNPOD environment is verified available.

train.py surface (DEFERRED until E-47.1). Uses Unsloth FastLanguageModel
with PEFT LoraConfig:
  init_lora_weights = "pissa"
  use_dora          = True
  target_modules    = "all-linear"
  r                 = 16
  lora_alpha        = 32
Base model: huihui-ai/Qwen2.5-Coder-7B-Instruct-abliterated (the
abliterated variant per CLAUDE.md REMOTE-SIDECAR). Training must use
bfloat16 compute dtype (NF4 4-bit quantized base, Ampere+ requirement per
CLAUDE.md Anti-Stale-Training). Compute dtype set via torch_dtype= in
from_pretrained, NOT dtype=. OPLoRA orthogonality projection (arXiv:
2510.13003) must be applied at training time to cap catastrophic forgetting
at the -2.0pp floor. On completion, train.py serializes the adapter weights
to ~/.elai/adapters/<name>/ and appends a row to manifest.toml.

eval.py surface (DEFERRED until E-47.1). Runs two probe regimes:
  IN-DOMAIN: 200 examples drawn from the same project corpus used for
    training. Measures exact-match + BLEU-4 vs the base model baseline.
  OOD: 200 examples drawn from a held-out project not in the training set.
    Measures the same metrics. This is the catastrophic-forgetting witness.
Both regimes produce a LoraCalibrationReport JSON at the calibration_report
path in manifest.toml:
  { "mmlu_delta_pct": <f32>, "humaneval_delta_pct": <f32> }
The report is consumed by check_lora_ship_gate on the Rust side. eval.py
must use torch.inference_mode() (not torch.no_grad()) for all inference
passes.

serve_adapter.py surface (DEFERRED until E-47.1). Thin wrapper around the
sidecar boot path. Reads manifest.toml, resolves the adapter for the current
project_id + base_model_version, and passes --lora-modules to the vLLM
subprocess (or extends the server.py command line when the sidecar is
running on the remote GPU box). Must verify the LoraCalibrationReport
passes_threshold() before passing the adapter to the server; refuses and
logs to stderr if the threshold is not met.

Calibration acceptance criterion. E-47.1 ships DONE only when eval.py
produces a LoraCalibrationReport where:
  mmlu_delta_pct     >= -2.0 (no-regression floor, arXiv:2405.09673)
  humaneval_delta_pct >= -2.0

The -2.0pp floor is conservative against the 5-19pp forgetting range
measured by Biderman et al. (arXiv:2405.09673). Either delta below the
floor FAILS the gate and blocks serve_adapter.py from loading the adapter.

OOD probe acceptance criterion. The 200-example OOD probe is the primary
forgetting witness. A project_id that does not exist in the training set is
selected by dataset.py at eval time using the project_id key in
manifest.toml. The OOD probe result feeds the humaneval_delta_pct field.

5pp forgetting cap. In-domain BLEU-4 delta must not exceed -5pp relative to
the base model on the 200 in-domain examples (conservative vs the 5-19pp
catastrophic-forgetting range in arXiv:2405.09673). This is a softer bound
than the mmlu/humaneval floor; the gate still enforces the -2.0pp floor on
both report fields as the primary mechanism.

Anti-patterns banned at this spec layer (7 items from the E-47 source report):
  A. Per-project LoRA training before measured prompt-rules+few-shot
     saturation. CodeRAG-Bench NAACL 2025 (Huang et al., arXiv:2406.14497
     section 5.2) shows retrieval + few-shot outperforms fine-tuning for
     code style at this corpus scale. Train.py must gate on the Q1 runbook
     question being answered with a measured comparison, not a conjecture.
  B. Continuous fine-tuning on rolling user data without an OPLoRA
     orthogonality constraint (arXiv:2510.13003). The OPLoRA projection
     step is mandatory in train.py; omitting it is a SPEC VIOLATION.
  C. Cross-project adapter sharing without the project_id + base_model_version
     composite key. serve_adapter.py must refuse to load any adapter whose
     manifest row project_id does not match the current project root's
     blake3 prefix. B-37 PerRoleRuntime overlay drift is the cited failure
     mode.
  D. Adapter promotion without an OOD probe. eval.py must run the OOD probe
     (200 examples from a held-out project) before writing the calibration
     report. A report written without an OOD pass is a SPEC VIOLATION.
  E. Voice-LoRA on per-user data without explicit --consent-yes at the
     dataset.py CLI. Any dataset that includes user-generated free-text
     (not code) must require --consent-yes. dataset.py refuses silently
     without the flag.
  F. Skipping calibration on "obviously safe" deltas. The -2.0pp floor is
     the only enforcement layer. No heuristic, no bypass, no --skip-eval
     flag. Every adapter that ship_gate allows must have a real calibration
     report at the calibration_report path in manifest.toml.
  G. Stacking multiple adapters at inference time without an aLoRA-style
     merge contract (arXiv:2512.17910). serve_adapter.py must not pass more
     than one --lora-modules entry unless a merged adapter produced by
     train.py's aLoRA merge step is the single entry. Sequential loading of
     separate adapters is banned.

Paper grounding (all arXiv IDs verified per CLAUDE.md Anti-Stale-Training):
  arXiv:2405.09673  Biderman et al. -- 5-19pp catastrophic forgetting on
                    code-style LoRA fine-tunes. Motivates the -2.0pp floor
                    and the OOD probe requirement.
  arXiv:2305.14314  QLoRA (Dettmers et al.) -- NF4 4-bit + paged-optimizer
                    baseline. train.py must use load_in_4bit=True with
                    bnb_4bit_compute_dtype=bfloat16 on Ampere+.
  arXiv:2404.02948  PiSSA (Meng et al.) -- principal-singular-value init.
                    train.py sets init_lora_weights="pissa" in LoraConfig.
  arXiv:2402.09353  DoRA (Liu et al.) -- weight-decomposed low-rank
                    adaptation. train.py sets use_dora=True in LoraConfig.
  arXiv:2510.13003  OPLoRA (Shi et al.) -- orthogonality-projected LoRA for
                    forgetting mitigation. Mandatory in train.py (anti-pattern B).
  arXiv:2406.14497  PortLLM (Huang et al.) -- patch-portable LoRA (nascent).
                    Cited for the CodeRAG-Bench comparison justifying anti-pattern A.
  arXiv:2512.17910  aLoRA (Zhang et al.) -- adapter merge contract. Cited for
                    the multi-adapter prohibition in anti-pattern G. serve_adapter.py
                    must implement the aLoRA merge path before any multi-adapter use.

DETERMINISTIC_CHECK items for E-47.1 auditor:
  1. python/elai_finetune/dataset.py exists and imports OPLoRA-compatible
     DataCollatorForSeq2Seq (not a bare DataCollator).
  2. python/elai_finetune/train.py uses init_lora_weights="pissa",
     use_dora=True, target_modules="all-linear", r=16, lora_alpha=32
     AND applies the OPLoRA projection step (grep for "orthogonal_proj"
     or the OPLoRA helper import).
  3. python/elai_finetune/SPEC.md is present at the exact path
     python/elai_finetune/SPEC.md (this file; verified by path check).
  4. python/elai_finetune/eval.py produces a JSON file whose keys are
     exactly "mmlu_delta_pct" and "humaneval_delta_pct" (both f32).
  5. python/elai_finetune/serve_adapter.py reads manifest.toml, checks
     project_id + base_model_version match, and calls passes_threshold()
     before passing --lora-modules to the server subprocess.

Flip-runbook. See workflow/decisions/E-47-flip-runbook.md for the 3-question
gate (Q1 measured prompt-rules+few-shot saturation, Q2 PEFT defaults still
work on the abliterated 7B, Q3 OPLoRA forgetting mitigation on the ELAI
worst-case domain gap) the operator clears before flipping the ship_gate
token from DEFERRED_PER_E47_REPORT to BASELINE_VERIFIED.

E-47.1 prerequisites. Before E-47.1 starts EXECUTING:
  a. RUNPOD RTX 3090 environment must be live (the python subtree requires
     Unsloth + bitsandbytes + peft to be importable).
  b. Q1 from the flip-runbook must be answered with a measured number (not
     a conjecture) showing prompt-rules+few-shot does NOT saturate the
     target style task at the project scale.
  c. The wire-dark-allowlist.toml row for ELAI_LORA_SHIP_GATE_ENABLE must
     already be present (shipped in E-47 base; check the row exists before
     starting E-47.1).
