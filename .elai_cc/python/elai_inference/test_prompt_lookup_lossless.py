# E-2 Task 3 — lossless test for prompt-lookup decoding.
#
# Per plan ACCEPTANCE: prompt_lookup_num_tokens=10 path must produce
# byte-equivalent output to baseline at temp=0 greedy
# (lossless guarantee per report anti-pattern list).
#
# RUNPOD-PENDING — requires live model load (~1.92GB NF4 on remote
# pythondev per memory:project_qwen3b_baseline.md). Cannot run locally
# per CLAUDE.md HARDWARE-HEAVY rule.
#
# Run via:
#   ssh -i ~/.ssh/elai_remote -p 6465 maintainer@192.0.2.10 \
#     'bash -lc "source ~/.venvs/elai-sidecar/bin/activate && \
#               cd ~/elai_inference && \
#               python -m elai_inference.test_prompt_lookup_lossless"'
#
# Exit 0 = byte-equivalent across all prompts (LOSSLESS — kwarg safe).
# Exit 1 = drift detected (kwarg NOT lossless; report to plan team).
# Exit 2 = environment skip (no CUDA available).

# RUNPOD-PENDING (live byte-equivalence on GPU): ssh -i ~/.ssh/elai_remote -p 6465 maintainer@192.0.2.10 'bash -lc "source ~/.venvs/elai-sidecar/bin/activate && cd ~/elai_inference && python -m elai_inference.test_prompt_lookup_lossless"'
#
# RUNPOD-COMPLETE-OFFLINE (sidecar wiring contract for HF kwarg):
#   pytest elai_inference/tests/test_prompt_lookup_lossless_offline.py -v
#   Verifies that server.py passes prompt_lookup_num_tokens through to HF
#   model.generate unchanged, with no sampling override and no logits
#   processor wrapping. Losslessness is then by-construction of the HF
#   verifier-rejection loop at temp=0 greedy. Live harness measures the
#   empirical byte equivalence on a real model.

from __future__ import annotations

import os
import sys
from typing import List

try:
    import torch
except ImportError:
    print("torch not available", file=sys.stderr)
    sys.exit(2)

try:
    from transformers import AutoModelForCausalLM, AutoTokenizer, BitsAndBytesConfig
except ImportError:
    print("transformers not available", file=sys.stderr)
    sys.exit(2)


# Five prompts spanning code edit, refactor, summarize, scaffold,
# free-form per plan Task 9. The prompts overlap intentionally with
# test_prefix_kv_determinism.py so the two harnesses share an
# evaluation surface.
LOSSLESS_PROMPTS: List[str] = [
    "Refactor: rewrite this function to use a list comprehension:\ndef squares(n):\n    out = []\n    for i in range(n):\n        out.append(i * i)\n    return out",
    "Add docstring: def parse_iso8601(ts: str) -> datetime: ...",
    "Summarize in one line: HF transformers exposes prompt_lookup_num_tokens as a model.generate kwarg; the draft pass reuses input-prefix n-grams.",
    "Scaffold: write a Rust trait `KvCache` with `lease(&self, prefix: &[u32])` and `admit(&self, prefix: &[u32], kv: Bytes)`; no impl.",
    "Explain in 3 lines why prompt-lookup decoding is lossless at temp=0.",
]


def _model_dir() -> str:
    return os.getenv(
        "ELAI_DETERMINISM_MODEL_PATH",
        "huihui-ai/Qwen2.5-Coder-3B-Instruct-abliterated",
    )


def _load_model_and_tokenizer():
    if not torch.cuda.is_available():
        print(
            "CUDA unavailable; this harness MUST run on remote pythondev. "
            "exiting with code 2",
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


def _generate(model, tokenizer, prompt: str, n_lookup: int, max_new_tokens: int = 96) -> List[int]:
    torch.manual_seed(0)
    inputs = tokenizer(prompt, return_tensors="pt").to(model.device)
    kwargs = dict(
        input_ids=inputs["input_ids"],
        attention_mask=inputs["attention_mask"],
        max_new_tokens=max_new_tokens,
        do_sample=False,
        return_dict_in_generate=True,
        pad_token_id=tokenizer.eos_token_id,
    )
    if n_lookup > 0:
        kwargs["prompt_lookup_num_tokens"] = n_lookup
    with torch.inference_mode():
        outputs = model.generate(**kwargs)
    seq = outputs.sequences[0].tolist()
    return seq[inputs["input_ids"].shape[1]:]


def main() -> int:
    model, tokenizer = _load_model_and_tokenizer()
    all_pass = True
    n_lookup = int(os.getenv("ELAI_PROMPT_LOOKUP_NUM_TOKENS", "10"))
    if n_lookup < 1:
        n_lookup = 10
    for i, prompt in enumerate(LOSSLESS_PROMPTS):
        baseline = _generate(model, tokenizer, prompt, n_lookup=0)
        treatment = _generate(model, tokenizer, prompt, n_lookup=n_lookup)
        if baseline != treatment:
            print(
                f"FAIL prompt {i}: prompt_lookup_num_tokens={n_lookup} "
                "diverges from baseline (greedy). Kwarg NOT lossless at temp=0 "
                "on this model — DO NOT promote default-on.",
                file=sys.stderr,
            )
            all_pass = False
            continue
        print(f"PASS prompt {i}: byte-equivalent across kwarg=0 vs kwarg={n_lookup}.")

    if not all_pass:
        return 1
    print(
        f"ALL {len(LOSSLESS_PROMPTS)} prompts PASS at "
        f"prompt_lookup_num_tokens={n_lookup}. Lossless guarantee holds for "
        "this model + kwarg. Acceptance-rate >=30% gate (G2) still required "
        "before default-on."
    )
    return 0


if __name__ == "__main__":
    sys.exit(main())
