# E-2 Task 9 — determinism harness for HF StaticCache prefix-KV reuse.
#
# Asserts byte-equivalent output across THREE consecutive runs at
# greedy + temp=0 with ELAI_HF_PREFIX_KV_REUSE=1 vs the baseline (env
# unset). DECISION-1 promotion gate G1: this harness must pass 3x
# consecutive on the remote 3090 (pythondev 192.0.2.10:6465) before
# flipping ELAI_HF_PREFIX_KV_REUSE to default-on.
#
# RUNPOD-PENDING — requires live model load (~1.92GB NF4 on remote
# pythondev per memory:project_qwen3b_baseline.md). Cannot run locally
# per CLAUDE.md HARDWARE-HEAVY rule.
#
# Run via:
#   ssh -i ~/.ssh/elai_remote -p 6465 maintainer@192.0.2.10 \
#     'bash -lc "source ~/.venvs/elai-sidecar/bin/activate && \
#               cd ~/elai_inference && \
#               python -m elai_inference.test_prefix_kv_determinism"'
#
# Exit 0 = byte-equivalent across runs (PROMOTION OK).
# Exit 1 = drift detected (KEEP env-gate OFF).
# Exit 2 = environment skip (no CUDA available; cannot run).

# RUNPOD-PENDING: ssh -i ~/.ssh/elai_remote -p 6465 maintainer@192.0.2.10 'bash -lc "source ~/.venvs/elai-sidecar/bin/activate && cd ~/elai_inference && python -m elai_inference.test_prefix_kv_determinism"'

from __future__ import annotations

import os
import sys
from typing import List, Tuple

try:
    import torch
except ImportError:
    print("torch not available; cannot run determinism harness", file=sys.stderr)
    sys.exit(2)

try:
    from transformers import AutoModelForCausalLM, AutoTokenizer, BitsAndBytesConfig
except ImportError:
    print("transformers not available; cannot run determinism harness", file=sys.stderr)
    sys.exit(2)


# Five prompts spanning the report's input-grounded code-task profile:
# code edit, refactor, summarize, scaffold, free-form (per plan Task 9
# "Tests to add" mirror with test_prompt_lookup_lossless.py).
DETERMINISM_PROMPTS: List[str] = [
    "Refactor this function to use early returns: def f(x):\n    if x is not None:\n        return x * 2\n    else:\n        return 0",
    "Add a docstring describing what this function does:\ndef compute_blake3(data: bytes) -> str:\n    return blake3.blake3(data).hexdigest()",
    "Summarize in one sentence: The HF StaticCache uses contiguous tensor storage to enable torch.compile fusion across the prefill loop.",
    "Scaffold a Python class named PrefixCache with methods lease(input_ids) and admit(input_ids, kv) — no implementation needed.",
    "Explain in two lines why DoLa contrastive decoding is disabled by default on 7B Qwen models.",
]


def _model_dir() -> str:
    return os.getenv(
        "ELAI_DETERMINISM_MODEL_PATH",
        "huihui-ai/Qwen2.5-Coder-3B-Instruct-abliterated",
    )


def _load_model_and_tokenizer():
    """Load the canonical Qwen2.5-Coder-3B NF4 model on the remote 3090.
    Fails with a clear message if CUDA is unavailable so a local invoke
    immediately exits 2 (env skip)."""
    if not torch.cuda.is_available():
        print(
            "CUDA unavailable; this harness MUST run on remote pythondev "
            "(per memory:project_qwen3b_baseline.md). exiting with code 2",
            file=sys.stderr,
        )
        sys.exit(2)

    quant_config = BitsAndBytesConfig(
        load_in_4bit=True,
        bnb_4bit_compute_dtype=torch.bfloat16,
        bnb_4bit_quant_type="nf4",
    )
    tokenizer = AutoTokenizer.from_pretrained(_model_dir())
    model = AutoModelForCausalLM.from_pretrained(
        _model_dir(),
        quantization_config=quant_config,
        device_map="auto",
        attn_implementation="sdpa",
    )
    model.eval()
    return model, tokenizer


def _generate_greedy(model, tokenizer, prompt: str, max_new_tokens: int = 64) -> List[int]:
    """Greedy temp=0 generate; returns the token id list of the
    completion (excluding the prompt). Seeded for reproducibility."""
    torch.manual_seed(0)
    inputs = tokenizer(prompt, return_tensors="pt").to(model.device)
    with torch.inference_mode():
        outputs = model.generate(
            **inputs,
            max_new_tokens=max_new_tokens,
            do_sample=False,
            return_dict_in_generate=True,
            pad_token_id=tokenizer.eos_token_id,
        )
    seq = outputs.sequences[0].tolist()
    return seq[inputs["input_ids"].shape[1]:]


def _run_three_times(model, tokenizer, prompt: str, kv_reuse_on: bool) -> Tuple[bool, List[List[int]]]:
    """Run the same prompt 3 times under the given env-gate state.
    Returns (all_equal, runs)."""
    if kv_reuse_on:
        os.environ["ELAI_HF_PREFIX_KV_REUSE"] = "1"
    else:
        os.environ.pop("ELAI_HF_PREFIX_KV_REUSE", None)

    runs: List[List[int]] = []
    for _ in range(3):
        runs.append(_generate_greedy(model, tokenizer, prompt))
    all_equal = all(r == runs[0] for r in runs)
    return all_equal, runs


def main() -> int:
    model, tokenizer = _load_model_and_tokenizer()
    all_pass = True
    for i, prompt in enumerate(DETERMINISM_PROMPTS):
        # Baseline: env unset, 3 runs must be byte-equivalent (sanity
        # check that the greedy path is itself deterministic).
        baseline_equal, baseline_runs = _run_three_times(model, tokenizer, prompt, kv_reuse_on=False)
        if not baseline_equal:
            print(
                f"FAIL baseline determinism prompt {i}: 3 greedy runs differ — "
                "harness is broken or seed not honoured.",
                file=sys.stderr,
            )
            all_pass = False
            continue

        # Treatment: env=1, 3 runs must be byte-equivalent AND match
        # the baseline run output token-for-token.
        treatment_equal, treatment_runs = _run_three_times(
            model, tokenizer, prompt, kv_reuse_on=True
        )
        if not treatment_equal:
            print(
                f"FAIL treatment determinism prompt {i}: 3 runs with "
                "ELAI_HF_PREFIX_KV_REUSE=1 differ — prefix KV not reused "
                "deterministically.",
                file=sys.stderr,
            )
            all_pass = False
            continue

        if treatment_runs[0] != baseline_runs[0]:
            print(
                f"FAIL byte-equivalence prompt {i}: ELAI_HF_PREFIX_KV_REUSE=1 "
                "output diverged from baseline. Drift detected — keep env-gate OFF.",
                file=sys.stderr,
            )
            all_pass = False
            continue

        print(f"PASS prompt {i}: byte-equivalent across baseline+treatment (3+3 runs).")

    if not all_pass:
        return 1
    print(
        f"ALL {len(DETERMINISM_PROMPTS)} prompts PASS. "
        "Promotion gate G1 (3x consecutive) requires re-running this harness "
        "two more times with the same outcome before flipping default-on."
    )
    return 0


if __name__ == "__main__":
    sys.exit(main())
