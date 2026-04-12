"""Compute steering vector from contrast prompts for Gemma4 E2B.

This script computes a steering vector using the ActAdd algorithm:
1. Load Gemma4 E2B model
2. Forward pass on positive prompt (e.g., "be skeptical")
3. Forward pass on negative prompt (e.g., "be trusting")
4. Compute steering vector: h+ - h-
5. Save to .npz file
"""

import torch
import numpy as np
from transformers import AutoModelForCausalLM, AutoTokenizer
import sys

MODEL_PATH = r"D:\.ELAI_workflow\.elai_cc\models\gemma-4-E2B-it"
OUTPUT_PATH = r"D:\.ELAI_workflow\.elai_cc\models\steering_vectors\skepticism.npz"

# Contrast prompts for skepticism steering
POSITIVE_PROMPT = "I should question assumptions and verify claims before accepting them as true."
NEGATIVE_PROMPT = "I should accept information without questioning or verifying it."

TARGET_LAYER = 12  # Gemma4 E2B has 35 layers, target = 35/3 ≈ 12

def main():
    print(f"Loading model from: {MODEL_PATH}")
    tokenizer = AutoTokenizer.from_pretrained(MODEL_PATH)
    model = AutoModelForCausalLM.from_pretrained(
        MODEL_PATH,
        torch_dtype=torch.float16,
        device_map="auto",
    )
    
    # Gemma4 uses head_dim instead of hidden_size
    hidden_dim = getattr(model.config, 'head_dim', getattr(model.config, 'hidden_size', None))
    num_layers = getattr(model.config, 'num_hidden_layers', getattr(model.config, 'num_layers', None))
    print(f"Model loaded. Hidden dim: {hidden_dim}")
    print(f"Num layers: {num_layers}")
    print(f"Target layer: {TARGET_LAYER}")
    
    # Tokenize positive prompt
    pos_inputs = tokenizer(POSITIVE_PROMPT, return_tensors="pt").to(model.device)
    print(f"\nPositive prompt: '{POSITIVE_PROMPT}'")
    print(f"Token count: {pos_inputs.input_ids.shape[1]}")
    
    # Get hidden states for positive prompt
    with torch.no_grad():
        pos_outputs = model(**pos_inputs, output_hidden_states=True, return_dict=True)
        pos_hidden = pos_outputs.hidden_states[TARGET_LAYER][:, -1, :].squeeze(0).cpu()
    
    # Tokenize negative prompt
    neg_inputs = tokenizer(NEGATIVE_PROMPT, return_tensors="pt").to(model.device)
    print(f"\nNegative prompt: '{NEGATIVE_PROMPT}'")
    print(f"Token count: {neg_inputs.input_ids.shape[1]}")
    
    # Get hidden states for negative prompt
    with torch.no_grad():
        neg_outputs = model(**neg_inputs, output_hidden_states=True, return_dict=True)
        neg_hidden = neg_outputs.hidden_states[TARGET_LAYER][:, -1, :].squeeze(0).cpu()
    
    # Compute steering vector: h+ - h-
    steering_vector = pos_hidden - neg_hidden
    steering_np = steering_vector.numpy()
    
    # Compute magnitude
    magnitude = np.linalg.norm(steering_np)
    print(f"\nSteering vector computed:")
    print(f"  Shape: {steering_np.shape}")
    print(f"  Magnitude: {magnitude:.4f}")
    print(f"  Mean: {steering_np.mean():.4f}")
    print(f"  Std: {steering_np.std():.4f}")
    
    if magnitude < 1e-6:
        print("ERROR: Steering vector magnitude too small. Prompts may be too similar.")
        sys.exit(1)
    
    # Save to .npz
    import os
    os.makedirs(os.path.dirname(OUTPUT_PATH), exist_ok=True)
    np.savez(OUTPUT_PATH, vector=steering_np)
    print(f"\nSteering vector saved to: {OUTPUT_PATH}")
    
    print("\nTo use this vector with the sidecar:")
    print(f"python -m elai_inference.server \\")
    print(f"  --model {MODEL_PATH} \\")
    print(f"  --steering \\")
    print(f"  --steering-coefficient 5.0 \\")
    print(f"  --steering-layer {TARGET_LAYER} \\")
    print(f"  --steering-vector-path {OUTPUT_PATH}")

if __name__ == "__main__":
    main()
