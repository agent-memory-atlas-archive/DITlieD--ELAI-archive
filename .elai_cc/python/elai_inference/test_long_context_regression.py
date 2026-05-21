"""Long-context regression guard: AWQ vs NF4 at 8K, 16K, 25K tokens.

E-27 Task 9: per report Open Question 2 - PPL curve from 8K to 25K under
AWQ vs NF4. Tests structural scaffolding; live inference runs on remote 3090.

grounding: arXiv:2505.20276 (59% degradation at >64K under NF4).
           E-27 plan: ELAI 7B at 25K is inside the safer window.

Run: python -m pytest elai_inference/test_long_context_regression.py
"""

import csv
import os
from pathlib import Path

CONTEXT_LENGTHS = [8192, 16384, 25000]
PARITY_THRESHOLD = 0.90
METRICS_PATH = Path("metrics/awq_vs_nf4_longctx.csv")


def test_context_lengths_defined():
    assert CONTEXT_LENGTHS == [8192, 16384, 25000]


def test_parity_threshold_value():
    assert PARITY_THRESHOLD == 0.90


def _build_synthetic_prompt(token_count: int) -> str:
    words = ["tok" for _ in range(token_count)]
    return " ".join(words)


def test_synthetic_prompt_length():
    for ctx in CONTEXT_LENGTHS:
        p = _build_synthetic_prompt(ctx)
        words = p.split()
        assert len(words) == ctx, f"ctx={ctx} got {len(words)}"


class LongContextResult:
    """Container for one context-length parity measurement."""

    def __init__(
        self,
        context_len: int,
        nf4_score: float,
        awq_score: float,
    ) -> None:
        self.context_len = context_len
        self.nf4_score = nf4_score
        self.awq_score = awq_score

    @property
    def ratio(self) -> float:
        if self.nf4_score == 0.0:
            return 0.0
        return self.awq_score / self.nf4_score

    def passes(self) -> bool:
        return self.ratio >= PARITY_THRESHOLD


def test_long_context_result_passes():
    r = LongContextResult(8192, 1.0, 0.95)
    assert r.passes()


def test_long_context_result_fails():
    r = LongContextResult(25000, 1.0, 0.80)
    assert not r.passes()


def write_results_csv(results: list, path: Path) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    with open(path, "w", newline="") as f:
        writer = csv.DictWriter(f, fieldnames=["context_len", "nf4_score", "awq_score", "ratio", "passes"])
        writer.writeheader()
        for r in results:
            writer.writerow({
                "context_len": r.context_len,
                "nf4_score": r.nf4_score,
                "awq_score": r.awq_score,
                "ratio": r.ratio,
                "passes": r.passes(),
            })


def test_csv_write_and_read(tmp_path):
    results = [LongContextResult(ctx, 1.0, 0.95) for ctx in CONTEXT_LENGTHS]
    csv_path = tmp_path / "test.csv"
    write_results_csv(results, csv_path)
    assert csv_path.exists()
    with open(csv_path) as f:
        rows = list(csv.DictReader(f))
    assert len(rows) == 3
    assert rows[0]["context_len"] == "8192"
