"""DoLa E2B factuality evaluation.

Loads Gemma4 E2B via HuggingFace Transformers, runs 20 factuality prompts
with baseline (greedy) and DoLa decoding, logs comparison results.

Usage (WSL2):
    source ~/vllm-env/bin/activate
    python dola_e2b_eval.py

Implements DoLa algorithm from arxiv 2309.03883 (ICLR 2024).
"""

import json
import time
from pathlib import Path

import torch
import torch.nn.functional as F

# ---------------------------------------------------------------------------
# Constants (matching Rust implementation)
# ---------------------------------------------------------------------------

ALPHA_DEFAULT = 0.1
JSD_MAX_NATS = 0.6931471805599453  # ln(2)
EPSILON = 1e-12
MAX_NEW_TOKENS = 64

# ---------------------------------------------------------------------------
# Factuality prompts (TruthfulQA-style + general knowledge)
# ---------------------------------------------------------------------------

FACTUALITY_PROMPTS = [
    # TruthfulQA-style (common misconceptions)
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
    # General factual knowledge
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


# ---------------------------------------------------------------------------
# DoLa implementation (pure Python, matching Rust DolaDecoder)
# ---------------------------------------------------------------------------

def compute_jsd(p, q):
    """Jensen-Shannon Divergence in nats. Bounded [0, ln(2)]."""
    m = 0.5 * (p + q)
    # Avoid log(0) by clamping
    m_safe = m.clamp(min=EPSILON)
    p_safe = p.clamp(min=EPSILON)
    q_safe = q.clamp(min=EPSILON)

    kl_pm = (p * (p_safe / m_safe).log()).sum()
    kl_qm = (q * (q_safe / m_safe).log()).sum()
    jsd = 0.5 * kl_pm + 0.5 * kl_qm

    # Clamp to valid range
    return jsd.clamp(0.0, JSD_MAX_NATS).item()


def dola_select_premature_layer(final_probs, candidate_probs_dict):
    """Select premature layer M = argmax_j JSD(q_N || q_j)."""
    best_layer = None
    best_jsd = -1.0

    for layer_idx, q_j in candidate_probs_dict.items():
        jsd = compute_jsd(final_probs, q_j)
        if jsd > best_jsd:
            best_jsd = jsd
            best_layer = layer_idx

    return best_layer, best_jsd


def dola_contrastive_logits(final_logits, premature_logits, final_probs, alpha):
    """Compute contrastive logits with V_head plausibility constraint."""
    max_prob = final_probs.max().item()
    threshold = alpha * max_prob

    # V_head: tokens where q_N(i) >= alpha * max(q_N)
    v_head_mask = final_probs >= threshold

    # Contrastive logits: raw logit difference (NOT softmax ratio)
    # Per paper Section 8.2: using raw logits gives better results
    contrastive = torch.full_like(final_logits, float('-inf'))
    contrastive[v_head_mask] = final_logits[v_head_mask] - premature_logits[v_head_mask]

    return contrastive, v_head_mask


def get_candidate_layers(n_layers):
    """Derive candidate layer set (even-indexed in buckets)."""
    if n_layers <= 40:
        mid = n_layers // 2
        buckets = [range(0, mid), range(mid, n_layers)]
    else:
        buckets = []
        start = 0
        while start < n_layers:
            end = min(start + 20, n_layers)
            buckets.append(range(start, end))
            start = end

    candidates = []
    for bucket in buckets:
        for layer in bucket:
            if layer % 2 == 0:
                candidates.append(layer)
    return candidates


# ---------------------------------------------------------------------------
# Generation functions
# ---------------------------------------------------------------------------

def format_prompt(tokenizer, prompt):
    """Format prompt using model's chat template."""
    messages = [{"role": "user", "content": prompt}]
    formatted = tokenizer.apply_chat_template(
        messages, tokenize=False, add_generation_prompt=True
    )
    return formatted


def generate_baseline(model, tokenizer, prompt, max_new_tokens=MAX_NEW_TOKENS):
    """Standard greedy decoding."""
    formatted = format_prompt(tokenizer, prompt)
    inputs = tokenizer(formatted, return_tensors="pt").to(model.device)
    with torch.no_grad():
        outputs = model.generate(
            **inputs,
            max_new_tokens=max_new_tokens,
            do_sample=False,
        )
    generated = outputs[0][inputs.input_ids.shape[1]:]
    return tokenizer.decode(generated, skip_special_tokens=True)


def generate_dola(model, tokenizer, prompt, lm_head, n_layers, alpha=ALPHA_DEFAULT,
                  max_new_tokens=MAX_NEW_TOKENS):
    """DoLa decoding: greedy with layer-contrast logits."""
    formatted = format_prompt(tokenizer, prompt)
    inputs = tokenizer(formatted, return_tensors="pt").to(model.device)
    input_ids = inputs.input_ids.clone()

    candidate_layers = get_candidate_layers(n_layers)

    generated_tokens = []
    jsd_values = []
    selected_layers = []

    for _ in range(max_new_tokens):
        with torch.no_grad():
            outputs = model(
                input_ids=input_ids,
                output_hidden_states=True,
                return_dict=True,
            )

        # Final layer logits (last token position)
        final_hidden = outputs.hidden_states[-1][:, -1, :].to(lm_head.weight.device)
        final_logits = lm_head(final_hidden).squeeze(0).float()  # float32
        final_probs = F.softmax(final_logits, dim=-1)

        # Compute probabilities for each candidate layer
        candidate_probs = {}
        candidate_raw_logits = {}
        n_hidden = len(outputs.hidden_states)
        for layer_idx in candidate_layers:
            # hidden_states[0] = embedding, hidden_states[i+1] = layer i output
            hs_idx = layer_idx + 1
            if hs_idx < n_hidden:
                h = outputs.hidden_states[hs_idx][:, -1, :].to(lm_head.weight.device)
                raw = lm_head(h).squeeze(0).float()  # float32 for numerical stability
                candidate_probs[layer_idx] = F.softmax(raw, dim=-1)
                candidate_raw_logits[layer_idx] = raw

        if not candidate_probs:
            # Fallback to greedy if no candidates available
            next_token = final_logits.argmax().item()
        else:
            # Select premature layer
            best_layer, best_jsd = dola_select_premature_layer(
                final_probs, candidate_probs
            )
            jsd_values.append(best_jsd)
            selected_layers.append(best_layer)

            # Contrastive logits
            contrastive, v_head_mask = dola_contrastive_logits(
                final_logits, candidate_raw_logits[best_layer],
                final_probs, alpha
            )

            # Greedy decode from contrastive logits
            next_token = contrastive.argmax().item()

        generated_tokens.append(next_token)

        # Check for EOS
        if next_token == tokenizer.eos_token_id:
            break

        # Append token for next iteration
        next_tensor = torch.tensor([[next_token]], device=model.device)
        input_ids = torch.cat([input_ids, next_tensor], dim=1)

    text = tokenizer.decode(generated_tokens, skip_special_tokens=True)
    return text, jsd_values, selected_layers


# ---------------------------------------------------------------------------
# Main evaluation
# ---------------------------------------------------------------------------

def main():
    print("=" * 70)
    print("DoLa E2B Factuality Evaluation")
    print("Model: google/gemma-4-E2B-it")
    print("=" * 70)

    # Load model
    print("\nLoading model...")
    from transformers import AutoModelForCausalLM, AutoTokenizer

    model_name = "google/gemma-4-E2B-it"
    tokenizer = AutoTokenizer.from_pretrained(model_name)
    model = AutoModelForCausalLM.from_pretrained(
        model_name,
        dtype=torch.float16,
        device_map="auto",
    )
    model.eval()

    # Gemma4 model structure: model.model (Gemma4Model) + model.lm_head (Linear)
    # Text config with layer count is nested under text_config for multimodal.
    lm_head = model.lm_head

    # Get layer count — try text_config first (multimodal), then top-level
    if hasattr(model.config, 'text_config') and hasattr(model.config.text_config, 'num_hidden_layers'):
        n_layers = model.config.text_config.num_hidden_layers
    else:
        n_layers = model.config.num_hidden_layers
    candidates = get_candidate_layers(n_layers)
    print(f"Model layers: {n_layers}")
    print(f"Candidate layers ({len(candidates)}): {candidates}")
    print(f"Alpha: {ALPHA_DEFAULT}")

    # Diagnostic: check JSD per layer for first prompt
    print("\n--- JSD DIAGNOSTIC (first prompt, first generated token) ---")
    diag_prompt = format_prompt(tokenizer, FACTUALITY_PROMPTS[0])
    diag_inputs = tokenizer(diag_prompt, return_tensors="pt").to(model.device)
    with torch.no_grad():
        diag_out = model(**diag_inputs, output_hidden_states=True, return_dict=True)
    diag_final_h = diag_out.hidden_states[-1][:, -1, :].to(lm_head.weight.device)
    diag_final_logits = lm_head(diag_final_h).squeeze(0).float()
    diag_final_probs = F.softmax(diag_final_logits, dim=-1)
    print(f"Final layer top-5 probs: {diag_final_probs.topk(5).values.tolist()}")
    print(f"Final layer top-5 tokens: {[tokenizer.decode([t]) for t in diag_final_probs.topk(5).indices.tolist()]}")

    for layer_idx in candidates[:5]:  # First 5 candidate layers
        hs_idx = layer_idx + 1
        h = diag_out.hidden_states[hs_idx][:, -1, :].to(lm_head.weight.device)
        raw = lm_head(h).squeeze(0).float()
        probs = F.softmax(raw, dim=-1)
        jsd = compute_jsd(diag_final_probs, probs)
        top5 = probs.topk(5)
        print(f"Layer {layer_idx:2d}: JSD={jsd:.6f}, top-5 probs={top5.values.tolist()}, "
              f"tokens={[tokenizer.decode([t]) for t in top5.indices.tolist()]}")
    print("--- END DIAGNOSTIC ---\n")

    results = []

    for i, prompt in enumerate(FACTUALITY_PROMPTS):
        print(f"\n--- Prompt {i+1}/20 ---")
        print(f"Q: {prompt}")

        # Baseline
        t0 = time.time()
        baseline_answer = generate_baseline(model, tokenizer, prompt)
        baseline_time = time.time() - t0

        # DoLa
        t0 = time.time()
        dola_answer, jsd_vals, sel_layers = generate_dola(
            model, tokenizer, prompt, lm_head, n_layers
        )
        dola_time = time.time() - t0

        avg_jsd = sum(jsd_vals) / len(jsd_vals) if jsd_vals else 0.0
        layer_dist = {}
        for l in sel_layers:
            layer_dist[l] = layer_dist.get(l, 0) + 1

        print(f"  Baseline ({baseline_time:.1f}s): {baseline_answer[:120]}")
        print(f"  DoLa    ({dola_time:.1f}s): {dola_answer[:120]}")
        print(f"  Avg JSD: {avg_jsd:.6f}, Top selected layers: "
              f"{sorted(layer_dist.items(), key=lambda x: -x[1])[:3]}")

        results.append({
            "prompt": prompt,
            "baseline": baseline_answer,
            "dola": dola_answer,
            "baseline_time_s": round(baseline_time, 2),
            "dola_time_s": round(dola_time, 2),
            "avg_jsd": round(avg_jsd, 6),
            "selected_layers": layer_dist,
            "dola_overhead": round(dola_time / max(baseline_time, 0.01), 2),
        })

    # Summary
    print("\n" + "=" * 70)
    print("SUMMARY")
    print("=" * 70)

    avg_overhead = sum(r["dola_overhead"] for r in results) / len(results)
    avg_jsd_all = sum(r["avg_jsd"] for r in results) / len(results)
    print(f"Average DoLa overhead: {avg_overhead:.2f}x")
    print(f"Average JSD across all prompts: {avg_jsd_all:.6f}")

    # Count how many answers differ
    diffs = sum(1 for r in results if r["baseline"] != r["dola"])
    print(f"Answers that differ (baseline vs DoLa): {diffs}/20")

    # Save results
    out_path = Path(__file__).parent / "dola_e2b_results.json"
    with open(out_path, "w") as f:
        json.dump(results, f, indent=2)
    print(f"\nResults saved to: {out_path}")

    # Model size status
    if n_layers < 16:
        print("\nWARNING: Model has <16 layers. DoLa should be DISABLED.")
    elif n_layers < 32:
        print(f"\nNOTE: Model has {n_layers} layers (1-7B range). "
              "DoLa in EVALUATE mode — results may vary.")
    else:
        print(f"\nModel has {n_layers} layers. DoLa fully enabled.")


if __name__ == "__main__":
    main()
