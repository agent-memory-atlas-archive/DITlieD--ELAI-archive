"""Compute steering vectors from contrast prompts for ActAdd injection.

Single-pair mode (legacy):
  Computes h+ - h- from POSITIVE_PROMPT / NEGATIVE_PROMPT.

Batch mode (A-8):
  Computes modern_api_preference vector from multiple equal-length contrast
  pairs using the template "The recommended {domain} library is {name}".
  Hidden state extracted at the LAST SHARED TOKEN position (divergence point),
  not at sequence end, to isolate semantic direction from positional artifacts.
  Formula: v_fresh = (1/N) * sum(h_modern_i - h_deprecated_i)

Usage:
  # Legacy single-pair (skepticism vector)
  python compute_steering_vector.py

  # Batch mode (modern_api_preference vector)
  python compute_steering_vector.py --batch
"""

import argparse
import sys
import os

import numpy as np
import torch
from transformers import AutoModelForCausalLM, AutoTokenizer

_PROJECT_ROOT = os.path.normpath(os.path.join(os.path.dirname(__file__), ".."))
MODEL_PATH = os.environ.get(
    "ELAI_MODEL_PATH",
    os.path.join(_PROJECT_ROOT, "models", "Qwen2.5-Coder-7B-Instruct-abliterated"),
)
OUTPUT_PATH = os.environ.get(
    "ELAI_STEERING_OUTPUT",
    os.path.join(_PROJECT_ROOT, "models", "steering_vectors", "skepticism.npz"),
)
BATCH_OUTPUT_PATH = os.environ.get(
    "ELAI_BATCH_STEERING_OUTPUT",
    os.path.join(_PROJECT_ROOT, "models", "steering_vectors", "modern_api.npz"),
)

# ---------------------------------------------------------------------------
# Legacy single-pair prompts (skepticism vector)
# ---------------------------------------------------------------------------
POSITIVE_PROMPT = "I should question assumptions and verify claims before accepting them as true."
NEGATIVE_PROMPT = "I should accept information without questioning or verifying it."

# ---------------------------------------------------------------------------
# Batch contrast pairs (A-8, anti-staleness enforcement section 2.2)
# All pairs use identical template: "The recommended {domain} library is {name}"
# WARN-1 resolved: equal-length templates avoid positional encoding contamination.
# ---------------------------------------------------------------------------
CONTRAST_PAIRS: list[tuple[str, str]] = [
    # (modern / positive, deprecated / negative)
    ("The recommended hashing library is blake3",
     "The recommended hashing library is blake2"),
    ("The recommended async mutex is tokio sync Mutex",
     "The recommended async mutex is standard sync Mutex"),
    ("The recommended random library is rand zero point nine",
     "The recommended random library is rand zero point eight"),
    ("The recommended inference context is torch inference mode",
     "The recommended inference context is torch no grad"),
    ("The recommended web framework is FastAPI with uvicorn",
     "The recommended web framework is http server BaseHTTPRequestHandler"),
]

# Steering parameters
ALPHA_DEFAULT = 0.3
MAX_TOKEN_LENGTH_DIFF = 2  # WARN-1: pairs must differ by <= 2 tokens


def find_last_shared_token(pos_ids: torch.Tensor, neg_ids: torch.Tensor) -> int:
    """Find the index of the last shared token between two sequences.

    Returns the 0-based index of the last position where both sequences have
    the same token ID. Hidden state should be extracted at this position to
    isolate semantic divergence from positional artifacts.
    """
    min_len = min(pos_ids.shape[-1], neg_ids.shape[-1])
    pos_flat = pos_ids.squeeze(0)
    neg_flat = neg_ids.squeeze(0)

    last_shared = 0
    for i in range(min_len):
        if pos_flat[i].item() == neg_flat[i].item():
            last_shared = i
        else:
            break

    return last_shared


def validate_pair_lengths(
    tokenizer: AutoTokenizer,
    pairs: list[tuple[str, str]],
    max_diff: int = MAX_TOKEN_LENGTH_DIFF,
) -> list[tuple[str, str, int, int]]:
    """Validate that contrast pairs have equal (or near-equal) token lengths.

    Returns list of (positive, negative, pos_len, neg_len) for valid pairs.
    Raises ValueError if any pair exceeds max_diff tokens difference.
    """
    validated: list[tuple[str, str, int, int]] = []
    for pos_text, neg_text in pairs:
        pos_tokens = tokenizer.encode(pos_text, add_special_tokens=False)
        neg_tokens = tokenizer.encode(neg_text, add_special_tokens=False)
        diff = abs(len(pos_tokens) - len(neg_tokens))
        if diff > max_diff:
            raise ValueError(
                f"Token length mismatch ({diff} > {max_diff}): "
                f"pos={len(pos_tokens)} neg={len(neg_tokens)} | "
                f"'{pos_text}' vs '{neg_text}'"
            )
        validated.append((pos_text, neg_text, len(pos_tokens), len(neg_tokens)))
    return validated


def compute_batch_steering(
    model: AutoModelForCausalLM,
    tokenizer: AutoTokenizer,
    pairs: list[tuple[str, str]],
    target_layer: int,
) -> np.ndarray:
    """Compute averaged steering vector from multiple contrast pairs.

    For each pair, extracts hidden states at the last shared token position
    (divergence point), computes h_modern - h_deprecated, then averages all
    difference vectors: v_fresh = (1/N) * sum(h_modern_i - h_deprecated_i).
    """
    validated = validate_pair_lengths(tokenizer, pairs)
    print(f"Validated {len(validated)} contrast pairs (max diff <= {MAX_TOKEN_LENGTH_DIFF} tokens)")

    diffs: list[torch.Tensor] = []

    for i, (pos_text, neg_text, pos_len, neg_len) in enumerate(validated):
        pos_inputs = tokenizer(pos_text, return_tensors="pt").to(model.device)
        neg_inputs = tokenizer(neg_text, return_tensors="pt").to(model.device)

        # Find divergence point
        shared_idx = find_last_shared_token(pos_inputs.input_ids, neg_inputs.input_ids)

        with torch.inference_mode():
            pos_out = model(
                input_ids=pos_inputs.input_ids,
                output_hidden_states=True,
                return_dict=True,
            )
            pos_hidden = pos_out.hidden_states[target_layer][:, shared_idx, :].squeeze(0).cpu()

        with torch.inference_mode():
            neg_out = model(
                input_ids=neg_inputs.input_ids,
                output_hidden_states=True,
                return_dict=True,
            )
            neg_hidden = neg_out.hidden_states[target_layer][:, shared_idx, :].squeeze(0).cpu()

        diff = pos_hidden - neg_hidden
        magnitude = torch.norm(diff).item()
        diffs.append(diff)
        print(
            f"  Pair {i + 1}/{len(validated)}: shared_token_idx={shared_idx} "
            f"pos_len={pos_len} neg_len={neg_len} magnitude={magnitude:.4f}"
        )

    # Average: v_fresh = (1/N) * sum(diffs)
    stacked = torch.stack(diffs, dim=0)
    steering_vector = stacked.mean(dim=0)
    return steering_vector.float().numpy()


def main_single_pair(model, tokenizer, target_layer: int) -> None:
    """Legacy single-pair mode (skepticism vector)."""
    pos_inputs = tokenizer(POSITIVE_PROMPT, return_tensors="pt").to(model.device)
    neg_inputs = tokenizer(NEGATIVE_PROMPT, return_tensors="pt").to(model.device)
    print(f"Positive prompt: '{POSITIVE_PROMPT}' ({pos_inputs.input_ids.shape[1]} tokens)")
    print(f"Negative prompt: '{NEGATIVE_PROMPT}' ({neg_inputs.input_ids.shape[1]} tokens)")

    with torch.inference_mode():
        pos_out = model(**pos_inputs, output_hidden_states=True, return_dict=True)
        pos_hidden = pos_out.hidden_states[target_layer][:, -1, :].squeeze(0).cpu()

    with torch.inference_mode():
        neg_out = model(**neg_inputs, output_hidden_states=True, return_dict=True)
        neg_hidden = neg_out.hidden_states[target_layer][:, -1, :].squeeze(0).cpu()

    steering_np = (pos_hidden - neg_hidden).float().numpy()
    magnitude = np.linalg.norm(steering_np)
    print(f"Steering vector: shape={steering_np.shape} magnitude={magnitude:.4f}")

    if magnitude < 1e-6:
        print("ERROR: magnitude too small, prompts may be too similar.")
        sys.exit(1)

    os.makedirs(os.path.dirname(OUTPUT_PATH), exist_ok=True)
    np.savez(OUTPUT_PATH, vector=steering_np)
    print(f"Saved to: {OUTPUT_PATH}")


def main_batch(model, tokenizer, target_layer: int) -> None:
    """Batch mode (modern_api_preference vector from contrast pairs)."""
    print(f"Computing batch steering vector from {len(CONTRAST_PAIRS)} pairs...")
    steering_np = compute_batch_steering(model, tokenizer, CONTRAST_PAIRS, target_layer)
    magnitude = np.linalg.norm(steering_np)
    print(f"Averaged steering vector: shape={steering_np.shape} magnitude={magnitude:.4f}")
    print(f"  mean={steering_np.mean():.4f} std={steering_np.std():.4f}")

    if magnitude < 1e-6:
        print("ERROR: averaged magnitude too small.")
        sys.exit(1)

    os.makedirs(os.path.dirname(BATCH_OUTPUT_PATH), exist_ok=True)
    np.savez(
        BATCH_OUTPUT_PATH,
        vector=steering_np,
        alpha=np.float32(ALPHA_DEFAULT),
        n_pairs=np.int32(len(CONTRAST_PAIRS)),
        target_layer=np.int32(target_layer),
    )
    print(f"Saved to: {BATCH_OUTPUT_PATH}")


def main():
    parser = argparse.ArgumentParser(description="Compute ActAdd steering vectors")
    parser.add_argument(
        "--batch", action="store_true",
        help="Batch mode: compute modern_api_preference from contrast pairs",
    )
    parser.add_argument(
        "--target-layer", type=int, default=None,
        help="Target layer index (default: n_layers // 3)",
    )
    args = parser.parse_args()

    print(f"Loading model from: {MODEL_PATH}")
    if not os.path.exists(MODEL_PATH):
        import warnings
        warnings.warn(
            f"compute_steering_vector: model path '{MODEL_PATH}' not present on disk; "
            "skipping steering vector recomputation. "
            "Set ELAI_MODEL_PATH to the correct model directory.",
            stacklevel=2,
        )
        sys.exit(0)
    tokenizer = AutoTokenizer.from_pretrained(MODEL_PATH)
    model = AutoModelForCausalLM.from_pretrained(
        MODEL_PATH,
        torch_dtype=torch.bfloat16,
        device_map="auto",
    )

    num_layers = getattr(model.config, "num_hidden_layers", getattr(model.config, "num_layers", None))
    hidden_dim = getattr(model.config, "head_dim", getattr(model.config, "hidden_size", None))
    target_layer = args.target_layer if args.target_layer is not None else num_layers // 3
    print(f"Model: layers={num_layers} hidden_dim={hidden_dim} target_layer={target_layer}")

    if args.batch:
        main_batch(model, tokenizer, target_layer)
    else:
        main_single_pair(model, tokenizer, target_layer)


if __name__ == "__main__":
    main()
