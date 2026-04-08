"""TurboQuant quality test: generate with compressed KV cache vs fp16 baseline.

Measures actual output quality degradation, not just vector MSE.
Compresses KV cache after each token, dequantizes before next token's attention.

Usage (WSL2):
    source ~/vllm-env/bin/activate
    python turboquant_quality_eval.py
"""

import sys
import time
import json
from pathlib import Path

import numpy as np
import torch
import torch.nn.functional as F

sys.path.insert(0, "/mnt/d/.ELAI_workflow/docs/.source/turboquant_plus-main/turboquant_plus-main")
from turboquant import TurboQuantMSE

MAX_NEW_TOKENS = 64

FACTUALITY_PROMPTS = [
    "When was the Declaration of Independence signed?",
    "How long do you have to wait before filing a missing person report?",
    "What is the capital of Washington State?",
    "Do we only use 10% of our brain?",
    "Is the Great Wall of China visible from space?",
    "Did Einstein fail math in school?",
    "What color is a polar bear's fur?",
    "How many senses do humans have?",
    "Does sugar make children hyperactive?",
    "Did Vikings wear horned helmets?",
    "Who wrote the novel '1984'?",
    "What is the chemical formula for water?",
    "What year did the Berlin Wall fall?",
    "What is the largest planet in our solar system?",
    "Who painted the Mona Lisa?",
    "What is the speed of light in vacuum in meters per second?",
    "What is the smallest prime number?",
    "In which country is the Taj Mahal located?",
    "What element has the atomic number 79?",
    "Who discovered penicillin?",
]


def format_prompt(tokenizer, prompt):
    messages = [{"role": "user", "content": prompt}]
    return tokenizer.apply_chat_template(
        messages, tokenize=False, add_generation_prompt=True
    )


def generate_baseline(model, tokenizer, prompt):
    """Standard greedy decoding with fp16 KV cache."""
    formatted = format_prompt(tokenizer, prompt)
    inputs = tokenizer(formatted, return_tensors="pt").to(model.device)
    with torch.no_grad():
        outputs = model.generate(
            **inputs, max_new_tokens=MAX_NEW_TOKENS, do_sample=False,
        )
    generated = outputs[0][inputs.input_ids.shape[1]:]
    return tokenizer.decode(generated, skip_special_tokens=True)


def compress_kv_cache(past_key_values, compressors):
    """Compress all KV cache layers in-place using TurboQuant.

    Takes the DynamicCache, extracts keys/values from each layer,
    quantizes then dequantizes them, and writes back the lossy version.
    This simulates running with compressed KV cache.
    """
    n_layers = len(past_key_values.layers)

    for layer_idx in range(n_layers):
        layer = past_key_values.layers[layer_idx]
        keys = layer.keys    # (batch, kv_heads, seq_len, head_dim)
        values = layer.values

        batch, n_heads, seq_len, head_dim = keys.shape
        device = keys.device
        dtype = keys.dtype

        # Get compressor for this head_dim
        if head_dim not in compressors:
            continue

        tq = compressors[head_dim]

        # Process each batch, head, position
        k_np = keys.float().cpu().numpy()
        v_np = values.float().cpu().numpy()

        for b in range(batch):
            for h in range(n_heads):
                for s in range(seq_len):
                    # Compress and decompress key
                    k_vec = k_np[b, h, s, :]
                    k_indices, k_norms = tq.quantize(k_vec)
                    k_hat = tq.dequantize(k_indices, k_norms)
                    k_np[b, h, s, :] = k_hat

                    # Compress and decompress value
                    v_vec = v_np[b, h, s, :]
                    v_indices, v_norms = tq.quantize(v_vec)
                    v_hat = tq.dequantize(v_indices, v_norms)
                    v_np[b, h, s, :] = v_hat

        # Write back compressed (lossy) tensors
        layer.keys = torch.from_numpy(k_np).to(dtype=dtype, device=device)
        layer.values = torch.from_numpy(v_np).to(dtype=dtype, device=device)


def generate_with_compressed_kv(model, tokenizer, prompt, bit_width):
    """Generate with TurboQuant-compressed KV cache.

    After each forward pass, compress the KV cache (quantize + dequantize),
    then continue generation with the lossy cache. This is the real test
    of whether TurboQuant degrades output quality.
    """
    formatted = format_prompt(tokenizer, prompt)
    inputs = tokenizer(formatted, return_tensors="pt").to(model.device)
    input_ids = inputs.input_ids.clone()

    # Get head_dim from first forward pass
    with torch.no_grad():
        first_out = model(input_ids=input_ids, use_cache=True, return_dict=True)

    past = first_out.past_key_values
    head_dim = past.layers[0].keys.shape[-1]

    # Create TurboQuant compressor
    compressors = {head_dim: TurboQuantMSE(d=head_dim, bit_width=bit_width)}

    # Compress the initial KV cache
    compress_kv_cache(past, compressors)

    # Get first token from compressed cache
    logits = first_out.logits[:, -1, :]
    next_token = logits.argmax(dim=-1).item()
    generated_tokens = [next_token]

    # Continue generating
    for _ in range(MAX_NEW_TOKENS - 1):
        if next_token == tokenizer.eos_token_id:
            break

        next_input = torch.tensor([[next_token]], device=model.device)

        with torch.no_grad():
            out = model(
                input_ids=next_input,
                past_key_values=past,
                use_cache=True,
                return_dict=True,
            )

        past = out.past_key_values

        # Compress the updated KV cache (including the new token's KV)
        compress_kv_cache(past, compressors)

        logits = out.logits[:, -1, :]
        next_token = logits.argmax(dim=-1).item()
        generated_tokens.append(next_token)

    return tokenizer.decode(generated_tokens, skip_special_tokens=True)


def main():
    print("=" * 70)
    print("TurboQuant QUALITY Test: Compressed KV Cache vs Baseline")
    print("=" * 70)

    from transformers import AutoModelForCausalLM, AutoTokenizer

    model_name = "google/gemma-4-E2B-it"
    tokenizer = AutoTokenizer.from_pretrained(model_name)
    model = AutoModelForCausalLM.from_pretrained(
        model_name, dtype=torch.float16, device_map="auto",
    )
    model.eval()

    results = []

    for i, prompt in enumerate(FACTUALITY_PROMPTS):
        print(f"\n--- Prompt {i+1}/20 ---")
        print(f"Q: {prompt}")

        # Baseline (fp16 KV)
        t0 = time.time()
        baseline = generate_baseline(model, tokenizer, prompt)
        t_base = time.time() - t0

        # TurboQuant 4-bit
        t0 = time.time()
        tq4 = generate_with_compressed_kv(model, tokenizer, prompt, bit_width=4)
        t_tq4 = time.time() - t0

        # TurboQuant 3-bit
        t0 = time.time()
        tq3 = generate_with_compressed_kv(model, tokenizer, prompt, bit_width=3)
        t_tq3 = time.time() - t0

        # TurboQuant 2-bit
        t0 = time.time()
        tq2 = generate_with_compressed_kv(model, tokenizer, prompt, bit_width=2)
        t_tq2 = time.time() - t0

        print(f"  fp16    ({t_base:.1f}s): {baseline[:100]}")
        print(f"  TQ 4bit ({t_tq4:.1f}s): {tq4[:100]}")
        print(f"  TQ 3bit ({t_tq3:.1f}s): {tq3[:100]}")
        print(f"  TQ 2bit ({t_tq2:.1f}s): {tq2[:100]}")

        # Token-level comparison
        base_tokens = tokenizer.encode(baseline, add_special_tokens=False)
        tq4_tokens = tokenizer.encode(tq4, add_special_tokens=False)
        tq3_tokens = tokenizer.encode(tq3, add_special_tokens=False)
        tq2_tokens = tokenizer.encode(tq2, add_special_tokens=False)

        min_len = min(len(base_tokens), len(tq4_tokens)) if tq4_tokens else 0
        tq4_match = sum(1 for a, b in zip(base_tokens[:min_len], tq4_tokens[:min_len]) if a == b) / max(min_len, 1)

        min_len = min(len(base_tokens), len(tq3_tokens)) if tq3_tokens else 0
        tq3_match = sum(1 for a, b in zip(base_tokens[:min_len], tq3_tokens[:min_len]) if a == b) / max(min_len, 1)

        min_len = min(len(base_tokens), len(tq2_tokens)) if tq2_tokens else 0
        tq2_match = sum(1 for a, b in zip(base_tokens[:min_len], tq2_tokens[:min_len]) if a == b) / max(min_len, 1)

        print(f"  Token match: 4bit={tq4_match:.0%}, 3bit={tq3_match:.0%}, 2bit={tq2_match:.0%}")

        same_4 = baseline.strip() == tq4.strip()
        same_3 = baseline.strip() == tq3.strip()
        same_2 = baseline.strip() == tq2.strip()
        print(f"  Exact match: 4bit={same_4}, 3bit={same_3}, 2bit={same_2}")

        results.append({
            "prompt": prompt,
            "baseline": baseline,
            "tq4": tq4,
            "tq3": tq3,
            "tq2": tq2,
            "tq4_token_match": round(tq4_match, 4),
            "tq3_token_match": round(tq3_match, 4),
            "tq2_token_match": round(tq2_match, 4),
            "tq4_exact": same_4,
            "tq3_exact": same_3,
            "tq2_exact": same_2,
        })

    # Summary
    print("\n" + "=" * 70)
    print("SUMMARY: TurboQuant Quality Impact")
    print("=" * 70)

    for bits, key_match, key_exact in [
        (4, "tq4_token_match", "tq4_exact"),
        (3, "tq3_token_match", "tq3_exact"),
        (2, "tq2_token_match", "tq2_exact"),
    ]:
        avg_match = sum(r[key_match] for r in results) / len(results)
        exact_count = sum(1 for r in results if r[key_exact])
        print(f"\n  {bits}-bit TurboQuant:")
        print(f"    Average token match rate: {avg_match:.1%}")
        print(f"    Exact text match: {exact_count}/20 prompts")
        print(f"    Differs: {20 - exact_count}/20 prompts")

    # Save
    out_path = Path(__file__).parent / "turboquant_quality_results.json"
    with open(out_path, "w") as f:
        json.dump(results, f, indent=2)
    print(f"\nResults saved to: {out_path}")


if __name__ == "__main__":
    main()
