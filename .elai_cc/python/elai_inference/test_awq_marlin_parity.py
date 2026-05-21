"""Parity test: AWQ-Marlin vs NF4 across 5 fixed prompts at temp=0.

E-27 Task 8: assert token-cosine similarity >= 0.95 between AWQ and NF4
outputs on the same fixed prompts. Not a live inference test (no model
loaded here); validates the structural parity-check scaffolding and the
similarity helper. Live inference runs on the remote RTX 3090.

Run: python -m pytest elai_inference/test_awq_marlin_parity.py
"""

import math

PARITY_PROMPTS = [
    "def add(a: int, b: int) -> int:",
    "Refactor the following function to use a list comprehension:",
    "Summarize the key difference between NF4 and AWQ quantization.",
    "Scaffold a FastAPI route for POST /v1/messages with JSON body.",
    "You are a planning assistant. Output JSON task steps priority. User: build a REST API",
]

EXPECTED_PARITY_THRESHOLD = 0.95


def _cosine_similarity(a: list[float], b: list[float]) -> float:
    """Token-level cosine similarity between two embedding vectors."""
    if len(a) != len(b):
        raise ValueError(f"length mismatch: {len(a)} vs {len(b)}")
    dot = sum(x * y for x, y in zip(a, b))
    norm_a = math.sqrt(sum(x * x for x in a))
    norm_b = math.sqrt(sum(x * x for x in b))
    if norm_a == 0.0 or norm_b == 0.0:
        return 0.0
    return dot / (norm_a * norm_b)


def test_cosine_similarity_identical():
    v = [1.0, 0.5, 0.25, 0.125]
    assert _cosine_similarity(v, v) == 1.0


def test_cosine_similarity_orthogonal():
    a = [1.0, 0.0]
    b = [0.0, 1.0]
    assert abs(_cosine_similarity(a, b)) < 1e-9


def test_cosine_similarity_threshold():
    import random
    rng = random.Random(42)
    n = 32
    base = [rng.gauss(0, 1) for _ in range(n)]
    noise = [x + rng.gauss(0, 0.05) for x in base]
    sim = _cosine_similarity(base, noise)
    assert sim >= EXPECTED_PARITY_THRESHOLD, f"sim={sim} below threshold {EXPECTED_PARITY_THRESHOLD}"


def test_parity_prompts_are_five():
    assert len(PARITY_PROMPTS) == 5


def test_parity_prompts_are_nonempty():
    for p in PARITY_PROMPTS:
        assert len(p.strip()) > 0, "empty prompt"


def test_parity_threshold_value():
    assert EXPECTED_PARITY_THRESHOLD == 0.95


class AWQParityResult:
    """Container for per-prompt parity result."""

    def __init__(self, prompt: str, nf4_tokens: list[int], awq_tokens: list[int]) -> None:
        self.prompt = prompt
        self.nf4_tokens = nf4_tokens
        self.awq_tokens = awq_tokens

    def cosine_similarity(self) -> float:
        max_len = max(len(self.nf4_tokens), len(self.awq_tokens))
        a = self.nf4_tokens + [0] * (max_len - len(self.nf4_tokens))
        b = self.awq_tokens + [0] * (max_len - len(self.awq_tokens))
        return _cosine_similarity([float(x) for x in a], [float(x) for x in b])

    def passes(self) -> bool:
        return self.cosine_similarity() >= EXPECTED_PARITY_THRESHOLD


def test_parity_result_identical():
    tokens = [1, 2, 3, 4, 5]
    r = AWQParityResult("test", tokens, tokens)
    assert r.passes()
    assert r.cosine_similarity() == 1.0


def test_parity_result_divergent():
    a = list(range(1, 33))
    b = list(range(33, 65))
    r = AWQParityResult("test", a, b)
    assert not r.passes()
