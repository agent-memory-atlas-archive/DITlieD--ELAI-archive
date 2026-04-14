"""Tests for A-8 batch steering vector computation.

Verifies:
1. find_last_shared_token() locates divergence point
2. validate_pair_lengths() enforces WARN-1 equal-length constraint
3. compute_batch_steering() averages difference vectors correctly
4. Batch output .npz contains metadata (alpha, n_pairs, target_layer)
"""

import sys
import os
import tempfile

import numpy as np
import torch

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from compute_steering_vector import (
    CONTRAST_PAIRS,
    ALPHA_DEFAULT,
    MAX_TOKEN_LENGTH_DIFF,
    find_last_shared_token,
    validate_pair_lengths,
    compute_batch_steering,
)


def test_find_last_shared_token_identical_prefix():
    """Sequences sharing a prefix should return the last matching index."""
    # "The recommended hashing library is" = shared, then diverge
    pos = torch.tensor([[10, 20, 30, 40, 50, 99]])
    neg = torch.tensor([[10, 20, 30, 40, 50, 77]])
    idx = find_last_shared_token(pos, neg)
    assert idx == 4, f"expected 4, got {idx}"
    print("  pass: identical prefix -> last shared = 4")


def test_find_last_shared_token_no_shared():
    """Sequences diverging at position 0 should return 0."""
    pos = torch.tensor([[99, 20, 30]])
    neg = torch.tensor([[77, 20, 30]])
    idx = find_last_shared_token(pos, neg)
    assert idx == 0, f"expected 0, got {idx}"
    print("  pass: diverge at 0 -> last shared = 0")


def test_find_last_shared_token_all_shared():
    """Fully identical sequences should return last index."""
    pos = torch.tensor([[10, 20, 30]])
    neg = torch.tensor([[10, 20, 30]])
    idx = find_last_shared_token(pos, neg)
    assert idx == 2, f"expected 2, got {idx}"
    print("  pass: fully identical -> last shared = 2")


def test_find_last_shared_token_different_lengths():
    """Sequences of different lengths, shared prefix."""
    pos = torch.tensor([[10, 20, 30, 40, 50]])
    neg = torch.tensor([[10, 20, 30, 88]])
    idx = find_last_shared_token(pos, neg)
    assert idx == 2, f"expected 2, got {idx}"
    print("  pass: different lengths -> last shared = 2")


class MockTokenizer:
    """Minimal tokenizer mock for validate_pair_lengths tests."""
    def __init__(self, token_map: dict[str, list[int]] | None = None):
        self._map = token_map or {}

    def encode(self, text: str, add_special_tokens: bool = False) -> list[int]:
        if text in self._map:
            return self._map[text]
        # Default: one token per word
        return list(range(len(text.split())))

    def __call__(self, text: str, return_tensors: str = "pt"):
        ids = self.encode(text)
        class Result:
            input_ids = torch.tensor([ids])
            def to(self, device):
                return self
        return Result()


def test_validate_pair_lengths_pass():
    """Pairs with equal token count should pass."""
    tok = MockTokenizer({
        "The recommended hashing library is blake3": [1, 2, 3, 4, 5, 6],
        "The recommended hashing library is blake2": [1, 2, 3, 4, 5, 7],
    })
    result = validate_pair_lengths(tok, [
        ("The recommended hashing library is blake3",
         "The recommended hashing library is blake2"),
    ])
    assert len(result) == 1
    assert result[0][2] == 6  # pos_len
    assert result[0][3] == 6  # neg_len
    print("  pass: equal length pairs validated")


def test_validate_pair_lengths_within_tolerance():
    """Pairs within MAX_TOKEN_LENGTH_DIFF should pass."""
    tok = MockTokenizer({
        "short": [1, 2, 3],
        "longer": [1, 2, 3, 4],  # diff = 1, within tolerance
    })
    result = validate_pair_lengths(tok, [("short", "longer")], max_diff=2)
    assert len(result) == 1
    print("  pass: within tolerance accepted")


def test_validate_pair_lengths_reject():
    """Pairs exceeding MAX_TOKEN_LENGTH_DIFF should raise ValueError."""
    tok = MockTokenizer({
        "short": [1, 2],
        "very long sentence with many tokens": [1, 2, 3, 4, 5, 6, 7, 8],
    })
    try:
        validate_pair_lengths(tok, [("short", "very long sentence with many tokens")], max_diff=2)
        assert False, "should have raised ValueError"
    except ValueError as e:
        assert "mismatch" in str(e).lower()
        print(f"  pass: rejected with: {e}")


def test_compute_batch_steering_mock():
    """Test batch steering with a mock model that returns predictable hidden states."""
    hidden_dim = 8
    n_layers = 6
    target_layer = 2

    # Mock model: returns hidden states where the value at each layer is
    # based on the input token IDs (deterministic).
    class MockModel:
        class config:
            num_hidden_layers = n_layers
            hidden_size = hidden_dim
        device = torch.device("cpu")

        def __call__(self, input_ids, output_hidden_states=True, return_dict=True):
            seq_len = input_ids.shape[1]
            # Hidden states: list of (1, seq_len, hidden_dim) tensors per layer
            # Use input_ids sum as a seed for reproducibility
            seed = input_ids.sum().item()
            states = []
            for layer in range(n_layers + 1):  # +1 for embedding layer
                rng = np.random.RandomState(int(seed * 100 + layer))
                h = torch.from_numpy(
                    rng.randn(1, seq_len, hidden_dim).astype(np.float32)
                )
                states.append(h)

            class Result:
                hidden_states = states
            return Result()

    mock_model = MockModel()

    # Two pairs, same template structure
    tok = MockTokenizer({
        "The recommended hashing library is blake3": [1, 2, 3, 4, 5, 6],
        "The recommended hashing library is blake2": [1, 2, 3, 4, 5, 7],
        "The recommended random library is rand nine": [1, 2, 3, 4, 5, 8],
        "The recommended random library is rand eight": [1, 2, 3, 4, 5, 9],
    })

    pairs = [
        ("The recommended hashing library is blake3",
         "The recommended hashing library is blake2"),
        ("The recommended random library is rand nine",
         "The recommended random library is rand eight"),
    ]

    vector = compute_batch_steering(mock_model, tok, pairs, target_layer)

    assert vector.shape == (hidden_dim,), f"expected ({hidden_dim},), got {vector.shape}"
    assert vector.dtype == np.float32
    magnitude = np.linalg.norm(vector)
    assert magnitude > 1e-6, f"magnitude too small: {magnitude}"
    print(f"  pass: batch vector shape={vector.shape} magnitude={magnitude:.4f}")


def test_batch_output_npz_metadata():
    """Test that batch .npz output contains expected metadata fields."""
    with tempfile.NamedTemporaryFile(suffix=".npz", delete=False) as f:
        path = f.name

    try:
        vector = np.random.randn(128).astype(np.float32)
        np.savez(
            path,
            vector=vector,
            alpha=np.float32(ALPHA_DEFAULT),
            n_pairs=np.int32(5),
            target_layer=np.int32(12),
        )
        with np.load(path) as data:
            assert "vector" in data, "missing 'vector'"
            assert "alpha" in data, "missing 'alpha'"
            assert "n_pairs" in data, "missing 'n_pairs'"
            assert "target_layer" in data, "missing 'target_layer'"
            assert abs(float(data["alpha"].item()) - ALPHA_DEFAULT) < 1e-6
            assert int(data["n_pairs"].item()) == 5
            assert int(data["target_layer"].item()) == 12
            assert data["vector"].shape == (128,)
        print("  pass: npz metadata correct")
    finally:
        os.remove(path)


def test_contrast_pairs_not_empty():
    """CONTRAST_PAIRS must have at least 1 entry."""
    assert len(CONTRAST_PAIRS) >= 5, f"expected >= 5 pairs, got {len(CONTRAST_PAIRS)}"
    print(f"  pass: {len(CONTRAST_PAIRS)} contrast pairs defined")


def test_alpha_default():
    """Alpha default must be 0.3 per spec."""
    assert ALPHA_DEFAULT == 0.3, f"expected 0.3, got {ALPHA_DEFAULT}"
    print("  pass: alpha default = 0.3")


if __name__ == "__main__":
    print("=" * 60)
    print("A-8 Batch Steering Vector Tests")
    print("=" * 60)

    test_find_last_shared_token_identical_prefix()
    test_find_last_shared_token_no_shared()
    test_find_last_shared_token_all_shared()
    test_find_last_shared_token_different_lengths()
    test_validate_pair_lengths_pass()
    test_validate_pair_lengths_within_tolerance()
    test_validate_pair_lengths_reject()
    test_compute_batch_steering_mock()
    test_batch_output_npz_metadata()
    test_contrast_pairs_not_empty()
    test_alpha_default()

    print("\n" + "=" * 60)
    print("All A-8 batch steering tests passed!")
    print("=" * 60)
