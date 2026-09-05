E-27 Decisions: Quantization 2026

Plan: docs/thefinalglance/E-plan/E-27_quantization_2026.md
Date: 2026-05-20

DECISION A: W4A16 (AWQ), not W4A8 or sub-W4

What: ELAI serving quant floor is W4A16 (AWQ), not W4A8 or aggressive activation quant.

Key decision: arXiv:2504.04823 (COLM 2025) demonstrates that W4A16 and W8A8 are
lossless ceilings for LLM reasoning; sub-W4 or aggressive activation quantization
introduces measurable reasoning loss. ELAI runs Qwen2.5-Coder-7B/3B on reasoning
workloads; the W4A16 floor is the grounded minimum per that paper.

Expected test outcome: AWQ parity test passes (cosine >= 0.95 vs NF4 baseline).

DECISION B: FP8 KV, not TurboQuant

What: KV cache quantization uses FP8 (software-sim on Ampere), not TurboQuant.

Key decision: vllm.ai/blog/turboquant (May 2026 measurement) shows all TurboQuant
KV variants underperform FP8 by 20-34% throughput AND drop ~20 reasoning points.
FP8 KV is the production default in vLLM 2026. DEFER TurboQuant matches CLAUDE.md.

Expected test outcome: FP8 KV test emits kv_cache_dtype_software_sim on SM_86.

DECISION C: AWQ+Marlin, not GPTQ-Marlin

What: The weight-quantized serving path uses AWQ+Marlin, not GPTQ+Marlin.

Key decision: Jarvislabs 2026 head-to-head on Qwen2.5-32B: AWQ-Marlin 741 tok/s
vs NF4 168 tok/s (4.4x). HumanEval pass@1: AWQ 51.8% vs GPTQ-Marlin 46.3%.
AWQ wins on both throughput and quality.

DECISION D: NF4 QLoRA-only carve-out

What: NF4 (BitsAndBytes) path is preserved for QLoRA fine-tune checkpoints only.

Key decision: NF4 is the best format for QLoRA fine-tune. For serving, AWQ+Marlin
is strictly better. Two paths env-gated: ELAI_QUANT_BACKEND (nf4 default, awq opt-in).
Default stays NF4 until Tasks 8+9 pass 3 consecutive runs on remote 3090.

OUT OF SCOPE

EXL3 14B-on-24GB: deferred; author claim unverified on Blackwell.
NVFP4/MXFP4: deferred; B200/RTX 5090 hardware not in scope.
TurboQuant KV cache: rejected per vLLM May 2026 measurement.
TurboQuant embeddings: out of scope for E-27.
EXL3 for 24GB tier: deferred to follow-up E-N.

UNKNOWNS

Open Question 1: measured AWQ-Marlin tok/s for 7B on RTX 3060 12GB.
  Mitigation: Task 9 long-context test collects this; promotion gated on result.
Open Question 4: autoawq version stability in 2026.
  Mitigation: pinned >=0.2.0; latest verified 0.2.9 (MIT, Casper Hansen, pypi.org).
