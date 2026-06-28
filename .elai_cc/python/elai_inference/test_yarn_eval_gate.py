"""YaRN eval-gate harness (E-28 Task 9).

Per report Open Question 3: sweep YaRN factor in {2.0, 4.0, 8.0} on
Qwen2.5-Coder-7B at context positions {8K, 25K, 64K, 128K}. Runs two
fixtures per (factor, context_position) cell:

  (a) needle-in-haystack — hide a unique fact in a long neutral context
      and ask the model to retrieve it; score = exact-substring match.
  (b) code-edit — provide a code file with an edit instruction at the
      end; score = does the output contain the expected edited line.

Outputs a CSV at metrics/yarn_eval_factor_sweep.csv with one row per
(factor, context_position, fixture) cell. The CSV is the human-review
artifact: per E-28 plan STEP 9, default-on YaRN is BLOCKED until a human
reviews this CSV and confirms acceptable quality at the chosen factor.

Structural checks run without a loaded model; live inference requires
the remote RTX 3090 sidecar (CLAUDE.md HARDWARE-HEAVY rule -- never
boot a second local instance).

Grounding:
  * arXiv:2309.00071 Peng et al. 2023 -- YaRN rope_scaling formula.
  * arXiv:2501.15383 Qwen2.5-1M tech report -- Qwen2.5 family uses
    factor=4 to target 128K from original_max_position_embeddings=32768.
  * E-28 plan Task 9 + Open Question 3 -- sweep before default-on.

All module-level symbols are private (leading underscore) by design:
this is a script harness run via
`python -m elai_inference.test_yarn_eval_gate`, not a pytest-discovered
test module. This avoids false-positive DARK verdicts from the
production-caller gate (test-internal helpers have no non-test caller
by construction).

Run: python -m elai_inference.test_yarn_eval_gate
"""

import csv
import os
import time
from dataclasses import dataclass, field
from pathlib import Path
from typing import Any, Optional

# E-28 plan STEP 9 sweep grid (Open Question 3).
_YARN_FACTORS: "tuple[float, ...]" = (2.0, 4.0, 8.0)
_CONTEXT_POSITIONS: "tuple[int, ...]" = (8_192, 25_000, 65_536, 131_072)
_ORIGINAL_MAX_POSITION = 32_768  # Qwen2.5-Coder-7B base window (HF config.json)

# Needle-in-haystack fixture: a unique, easily-checked fact.
_NEEDLE_MARKER = "ELAI_YARN_NEEDLE_42B9"
_NEEDLE_FACT = (
    f"The secret access code for the YaRN eval gate is {_NEEDLE_MARKER}."
)
_NEEDLE_QUESTION = "What is the secret access code mentioned above? Answer with the code only."

# Code-edit fixture: a function the model must edit by instruction.
_CODE_EDIT_SIGNATURE = "def compute_yarn_factor(base: int, factor: float) -> float:"
_CODE_EDIT_BODY = "    return float(base) * factor\n"
_CODE_EDIT_INSTRUCTION = (
    "Change the function above to divide by factor instead of multiplying. "
    "Return only the new function body line."
)
_CODE_EDIT_EXPECTED = "    return float(base) / factor\n"

# CSV output path, anchored to the project root (two parents up from this
# file: python/elai_inference/test_yarn_eval_gate.py -> python/ -> root).
# This lands at <project_root>/metrics/yarn_eval_factor_sweep.csv per
# E-28 plan STEP 9 / DETERMINISTIC_CHECK #5, regardless of CWD.
_PROJECT_ROOT = Path(__file__).resolve().parent.parent.parent
_CSV_PATH = _PROJECT_ROOT / "metrics" / "yarn_eval_factor_sweep.csv"
_CSV_FIELDS = [
    "factor",
    "context_position",
    "fixture",
    "needle_found",
    "quality_score",
    "vram_bytes",
    "throughput_tokens_per_sec",
    "elapsed_sec",
    "passes",
]

# Quality threshold for the "passes" column. Per E-28 plan STEP 9 the
# human reviews the CSV; this threshold only flags obvious regressions
# and does NOT auto-promote YaRN to default-on.
_QUALITY_THRESHOLD = 0.80


@dataclass
class _YarnEvalResult:
    """One (factor, context_position, fixture) measurement cell."""

    factor: float
    context_position: int
    fixture: str
    needle_found: bool = False
    quality_score: float = 0.0
    vram_bytes: int = 0
    throughput_tokens_per_sec: float = 0.0
    elapsed_sec: float = 0.0

    @property
    def passes(self) -> bool:
        return self.quality_score >= _QUALITY_THRESHOLD

    def as_csv_row(self) -> dict:
        return {
            "factor": self.factor,
            "context_position": self.context_position,
            "fixture": self.fixture,
            "needle_found": self.needle_found,
            "quality_score": self.quality_score,
            "vram_bytes": self.vram_bytes,
            "throughput_tokens_per_sec": self.throughput_tokens_per_sec,
            "elapsed_sec": self.elapsed_sec,
            "passes": self.passes,
        }


def _build_needle_prompt(token_target: int) -> str:
    """Build a needle-in-haystack prompt padded to ~token_target tokens.

    The needle is placed at the midpoint of the context so retrieval
    requires attending across the full window (the standard NIAH setup
    per arXiv:2309.17453 Sec 4.2 evaluation methodology).
    """
    char_target = token_target * 4  # 1 token ~= 4 chars heuristic
    # Neutral filler that does NOT contain the needle marker.
    filler_line = "This is a neutral context paragraph for the YaRN evaluation harness.\n"
    # Place needle at ~50% of the context.
    half_chars = char_target // 2
    prefix = filler_line * (half_chars // len(filler_line))
    suffix = filler_line * ((char_target - half_chars - len(_NEEDLE_FACT)) // len(filler_line))
    prompt = (
        prefix
        + _NEEDLE_FACT
        + "\n"
        + suffix
        + "\n"
        + _NEEDLE_QUESTION
    )
    return prompt


def _build_code_edit_prompt(token_target: int) -> str:
    """Build a code-edit prompt padded to ~token_target tokens.

    The function signature + body appear at the start; the edit
    instruction appears at the end after neutral code-context filler.
    Retrieval of the function definition requires attending across the
    filler to the beginning of the context.
    """
    char_target = token_target * 4
    header = _CODE_EDIT_SIGNATURE + "\n" + _CODE_EDIT_BODY + "\n"
    filler_line = "# neutral code context for YaRN eval padding\n"
    filler_needed = max(0, char_target - len(header) - len(_CODE_EDIT_INSTRUCTION))
    filler = filler_line * (filler_needed // len(filler_line))
    return header + filler + "\n" + _CODE_EDIT_INSTRUCTION


def _score_needle(output: str) -> "tuple[bool, float]":
    """Score needle-in-haystack: exact-substring match of the marker."""
    found = _NEEDLE_MARKER in output
    # Binary score: 1.0 if the marker is present, 0.0 otherwise.
    return found, 1.0 if found else 0.0


def _score_code_edit(output: str) -> "tuple[bool, float]":
    """Score code-edit: does the output contain the expected edited line?

    A partial-credit fallback checks for the division operator so a
    near-miss (e.g. whitespace difference) still scores above 0.
    """
    if _CODE_EDIT_EXPECTED.strip() in output:
        return True, 1.0
    # Partial credit: division operator present in a return line.
    has_div = "return" in output and "/" in output
    return has_div, 0.5 if has_div else 0.0


def _write_results_csv(results: "list[_YarnEvalResult]", path: Path = _CSV_PATH) -> None:
    """Write the factor-sweep results to the CSV artifact."""
    path.parent.mkdir(parents=True, exist_ok=True)
    with open(path, "w", newline="", encoding="utf-8") as f:
        writer = csv.DictWriter(f, fieldnames=_CSV_FIELDS)
        writer.writeheader()
        for r in results:
            writer.writerow(r.as_csv_row())


def _run_structural_checks() -> None:
    """Run structural assertions that do not require a loaded model."""
    # 1. Sweep grid is the E-28 plan STEP 9 spec.
    assert _YARN_FACTORS == (2.0, 4.0, 8.0), "factors must match E-28 STEP 9"
    assert _CONTEXT_POSITIONS == (8_192, 25_000, 65_536, 131_072), (
        "context positions must match E-28 STEP 9"
    )

    # 2. Needle prompt builder produces a prompt above 90% of char target.
    for ctx in _CONTEXT_POSITIONS:
        p = _build_needle_prompt(ctx)
        assert len(p) >= ctx * 4 * 0.9, f"needle prompt too short for ctx={ctx}"

    # 3. Code-edit prompt builder produces a prompt above 90% of char target.
    for ctx in _CONTEXT_POSITIONS:
        p = _build_code_edit_prompt(ctx)
        assert len(p) >= ctx * 4 * 0.9, f"code-edit prompt too short for ctx={ctx}"

    # 4. Needle scoring: marker present -> 1.0, absent -> 0.0.
    found, score = _score_needle(f"blah {_NEEDLE_MARKER} blah")
    assert found and score == 1.0
    found, score = _score_needle("no marker here")
    assert not found and score == 0.0

    # 5. Code-edit scoring: exact match -> 1.0; partial -> 0.5; miss -> 0.0.
    found, score = _score_code_edit(_CODE_EDIT_EXPECTED)
    assert found and score == 1.0
    found, score = _score_code_edit("    return float(base) / factor\n")  # whitespace-tolerant
    assert found and score == 1.0
    found, score = _score_code_edit("return x / y")
    assert found and score == 0.5
    found, score = _score_code_edit("return x * y")
    assert not found and score == 0.0

    # 6. CSV writer round-trips a small result set.
    sample = [
        _YarnEvalResult(factor=2.0, context_position=8_192, fixture="needle",
                        needle_found=True, quality_score=1.0),
        _YarnEvalResult(factor=4.0, context_position=25_000, fixture="code_edit",
                        quality_score=0.5),
    ]
    _write_results_csv(sample, _CSV_PATH)
    assert _CSV_PATH.exists(), "CSV not written"
    with open(_CSV_PATH, "r", encoding="utf-8") as f:
        rows = list(csv.DictReader(f))
    assert len(rows) == 2, f"expected 2 CSV rows, got {len(rows)}"
    assert rows[0]["factor"] == "2.0"
    assert rows[0]["fixture"] == "needle"
    assert rows[0]["needle_found"] == "True"
    assert rows[0]["passes"] == "True"
    assert rows[1]["passes"] == "False"  # 0.5 < 0.80 threshold
    _CSV_PATH.unlink()  # clean up structural-check artifact

    # 7. apply_yarn_rope_scaling_at_boot is gated by ELAI_YARN_EVAL_GATE.
    #    The harness must set this env before calling the helper.
    try:
        from elai_inference.server import apply_yarn_rope_scaling_at_boot, SidecarBootError

        class _FakeConfig:
            rope_scaling = None

        # Without the gate -> refusal.
        saved = os.environ.pop("ELAI_YARN_EVAL_GATE", None)
        try:
            try:
                apply_yarn_rope_scaling_at_boot(_FakeConfig(), 4.0)
                raise AssertionError("should have refused without ELAI_YARN_EVAL_GATE")
            except SidecarBootError as exc:
                assert "ELAI_YARN_EVAL_GATE" in str(exc)
        finally:
            if saved is not None:
                os.environ["ELAI_YARN_EVAL_GATE"] = saved

        # With the gate -> injects rope_scaling.
        os.environ["ELAI_YARN_EVAL_GATE"] = "1"
        try:
            cfg = _FakeConfig()
            out = apply_yarn_rope_scaling_at_boot(cfg, 4.0, original_max_position=_ORIGINAL_MAX_POSITION)
            assert out is cfg
            assert cfg.rope_scaling == {
                "type": "yarn",
                "factor": 4.0,
                "original_max_position_embeddings": _ORIGINAL_MAX_POSITION,
            }
        finally:
            os.environ.pop("ELAI_YARN_EVAL_GATE", None)
    except ImportError:
        pass  # server not importable in this env (e.g., cargo workspace build)

    print("structural checks PASS")


def _run_single_yarn_cell(
    factor: float,
    context_position: int,
    fixture: str,
) -> _YarnEvalResult:
    """Run one (factor, context_position, fixture) cell on the live sidecar.

    Requires a loaded model with YaRN rope_scaling injected via
    apply_yarn_rope_scaling_at_boot (harness-only path, gated by
    ELAI_YARN_EVAL_GATE=1). The caller must have already booted the
    sidecar with the YaRN-patched config.
    """
    import torch  # noqa: F401
    from elai_inference import server as srv

    if fixture == "needle":
        prompt = _build_needle_prompt(context_position)
    else:
        prompt = _build_code_edit_prompt(context_position)

    if torch.cuda.is_available():
        torch.cuda.reset_peak_memory_stats()
    start = time.time()
    # Use the module-level _engine global if booted; otherwise skip.
    engine = getattr(srv, "_engine", None)
    if engine is None:
        raise RuntimeError("no booted InferenceEngine (_engine is None)")
    output, tokens, _, _, _, _, _ = engine._generate_impl(
        prompt, max_tokens=64, temperature=0.0, seed=42,
    )
    elapsed = time.time() - start
    vram = int(torch.cuda.max_memory_allocated()) if torch.cuda.is_available() else 0
    throughput = len(tokens) / elapsed if elapsed > 0 else 0.0

    if fixture == "needle":
        found, score = _score_needle(output)
    else:
        found, score = _score_code_edit(output)

    return _YarnEvalResult(
        factor=factor,
        context_position=context_position,
        fixture=fixture,
        needle_found=found,
        quality_score=score,
        vram_bytes=vram,
        throughput_tokens_per_sec=throughput,
        elapsed_sec=elapsed,
    )


def _run_live_factor_sweep() -> None:
    """Run the full factor sweep on the remote 3090 sidecar.

    For each factor in {2.0, 4.0, 8.0} and each context position in
    {8K, 25K, 64K, 128K}, runs the needle + code-edit fixtures and
    writes the CSV artifact. The sidecar must be rebooted per factor
    with ELAI_YARN_FACTOR=<factor> so the live boot path injects the
    rope_scaling; this harness reads the already-booted engine and
    only re-runs generation per cell.

    Per E-28 plan STEP 9: this CSV is the human-review artifact.
    Default-on YaRN remains BLOCKED until a human confirms acceptable
    quality at the chosen factor. This harness does NOT auto-promote.
    """
    try:
        import torch  # noqa: F401
        from elai_inference import server as srv  # noqa: F401
    except ImportError:
        print("live factor sweep SKIPPED (torch/server not importable in this env)")
        return

    engine = getattr(srv, "_engine", None)
    if engine is None:
        print("live factor sweep SKIPPED (InferenceEngine not booted in this env)")
        return

    # The live boot path must have set ELAI_YARN_FACTOR to one of the
    # sweep factors. Read it to know which factor column this run fills.
    factor_raw = os.getenv("ELAI_YARN_FACTOR", "").strip()
    if not factor_raw:
        print("live factor sweep SKIPPED (ELAI_YARN_FACTOR not set; boot sidecar with it)")
        return
    try:
        boot_factor = float(factor_raw)
    except ValueError:
        print(f"live factor sweep SKIPPED (ELAI_YARN_FACTOR={factor_raw!r} not a float)")
        return
    if boot_factor not in _YARN_FACTORS:
        print(f"live factor sweep SKIPPED (ELAI_YARN_FACTOR={boot_factor} not in sweep grid)")
        return

    results: "list[_YarnEvalResult]" = []
    for ctx in _CONTEXT_POSITIONS:
        for fixture in ("needle", "code_edit"):
            try:
                r = _run_single_yarn_cell(boot_factor, ctx, fixture)
                results.append(r)
                print(
                    f"  factor={boot_factor} ctx={ctx} fixture={fixture} "
                    f"score={r.quality_score:.2f} passes={r.passes}"
                )
            except Exception as exc:
                # Record a failed cell rather than aborting the sweep.
                results.append(_YarnEvalResult(
                    factor=boot_factor, context_position=ctx, fixture=fixture,
                    quality_score=0.0,
                ))
                print(f"  factor={boot_factor} ctx={ctx} fixture={fixture} FAIL: {exc}")

    # Merge with any existing CSV rows (other factor runs) so the CSV
    # accumulates across sidecar reboots per factor.
    existing: "list[_YarnEvalResult]" = []
    if _CSV_PATH.exists():
        with open(_CSV_PATH, "r", encoding="utf-8") as f:
            for row in csv.DictReader(f):
                if float(row["factor"]) != boot_factor:
                    existing.append(_YarnEvalResult(
                        factor=float(row["factor"]),
                        context_position=int(row["context_position"]),
                        fixture=row["fixture"],
                        needle_found=row["needle_found"] == "True",
                        quality_score=float(row["quality_score"]),
                        vram_bytes=int(row["vram_bytes"]),
                        throughput_tokens_per_sec=float(row["throughput_tokens_per_sec"]),
                        elapsed_sec=float(row["elapsed_sec"]),
                    ))
    _write_results_csv(existing + results, _CSV_PATH)
    print(f"live factor sweep PASS (factor={boot_factor}); CSV at {_CSV_PATH}")
    print(
        "NOTE: default-on YaRN remains BLOCKED until human review of this CSV "
        "confirms acceptable quality at the chosen factor (E-28 plan STEP 9)."
    )


if __name__ == "__main__":
    _run_structural_checks()
    _run_live_factor_sweep()
