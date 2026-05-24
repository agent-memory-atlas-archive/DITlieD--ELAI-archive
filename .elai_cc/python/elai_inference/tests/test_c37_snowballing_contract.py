"""C-37 contract tests for the Structure Snowballing harness.

Paper-grounded assertions per
workflow/papers/structure-snowballing-interface.md "Critical invariants"
section. These tests run WITHOUT live model inference — they exercise the
harness's static contract surface (enum membership, schema shape, hyperparam
extraction, threshold accessibility, parameter range checks).

The live-replication tests are RUNPOD-PENDING (require GPU per the run.py
RUNPOD-PENDING block).
"""

from __future__ import annotations

import pytest
from pydantic import ValidationError

from elai_inference.c37_snowballing.metrics import (
    CONSTRAINED_ACCURACY_CEILING,
    REPLICATION_THRESHOLD,
    SEVERITY_THRESHOLD,
    compute_accuracy,
    compute_error_distribution,
    compute_snowballing_rate,
    compute_status_flips,
    is_paper_equivalent_severity,
    is_replication_observed,
)
from elai_inference.c37_snowballing.roles import (
    PAPER_MAX_NEW_TOKENS,
    PAPER_MODEL_DEFAULT,
    PAPER_TEMPERATURE,
    paper_role_config,
)
from elai_inference.c37_snowballing.types import (
    ERROR_TYPES,
    ReflectorOutput,
)


# Test 1 — 5-class enum present and exactly matches paper Section 3.1.
def test_error_type_enum_is_paper_grounded_5_class() -> None:
    expected = (
        "RETRIEVAL_FOCUS",
        "BRIDGE_FAILURE",
        "HALLUCINATION",
        "INFERENCE_ERROR",
        "FORMATTING_MISMATCH",
    )
    assert ERROR_TYPES == expected, (
        f"ERROR_TYPES drift; paper Section 3.1 mandates {expected}"
    )
    assert len(ERROR_TYPES) == 5, "paper-explicit ceiling — see Nathani et al. 2023"


# Test 2 — ReflectorOutput schema is parseable + enforces enum + non-empty rule.
def test_reflector_output_schema_round_trips() -> None:
    valid = ReflectorOutput(
        error_type="HALLUCINATION",
        correction_rule="Cite the supporting paragraph before answering.",
    )
    assert valid.error_type == "HALLUCINATION"
    assert valid.correction_rule.startswith("Cite")

    # Enum violation -> ValidationError.
    with pytest.raises(ValidationError):
        ReflectorOutput(
            error_type="NOT_A_PAPER_CLASS",  # type: ignore[arg-type]
            correction_rule="x",
        )

    # Empty correction_rule -> ValidationError.
    with pytest.raises(ValidationError):
        ReflectorOutput(error_type="HALLUCINATION", correction_rule="")

    # Whitespace-only correction_rule -> ValidationError.
    with pytest.raises(ValidationError):
        ReflectorOutput(error_type="HALLUCINATION", correction_rule="   ")


# Test 3 — Uniform decoding hyperparameters extractable + paper-grounded.
def test_paper_role_config_uniform_decoding() -> None:
    for role in ("actor", "evaluator", "reflector"):
        cfg = paper_role_config(role)
        assert cfg.temperature == PAPER_TEMPERATURE == 0.1
        assert cfg.max_new_tokens == PAPER_MAX_NEW_TOKENS == 1024
        assert cfg.model == PAPER_MODEL_DEFAULT == "Qwen/Qwen3-8B"
        assert cfg.constrained == (role == "reflector"), (
            f"only reflector should be constrained per paper Section 3.3; "
            f"role={role} constrained={cfg.constrained}"
        )


# Test 4 — Snowballing thresholds accessible + paper-grounded values.
def test_snowballing_thresholds_paper_grounded() -> None:
    assert REPLICATION_THRESHOLD == 0.50, (
        "paper-grounded replication threshold per source interface map"
    )
    assert SEVERITY_THRESHOLD == 0.90, (
        "paper-grounded severity threshold per source interface map"
    )
    assert CONSTRAINED_ACCURACY_CEILING == 0.45, (
        "paper Table 1 / 2 anchor — constrained accuracy SHOULD be below 0.45"
    )
    assert is_replication_observed(0.60) is True
    assert is_replication_observed(0.30) is False
    assert is_paper_equivalent_severity(0.96) is True
    assert is_paper_equivalent_severity(0.50) is False


# Test 5 — accuracy-drop direction is enforced when comparing distributions.
def test_accuracy_drop_direction() -> None:
    # Paper Table 1: baseline 0.50 > constrained 0.38 (alignment tax).
    # compute_accuracy is monotonic: more True verdicts -> higher value.
    baseline = [True] * 50 + [False] * 50  # 0.5
    constrained = [True] * 38 + [False] * 62  # 0.38
    a_base = compute_accuracy(baseline)
    a_con = compute_accuracy(constrained)
    assert a_base > a_con, (
        f"alignment tax direction lost: baseline={a_base} constrained={a_con}"
    )
    assert pytest.approx(a_base, abs=1e-9) == 0.50
    assert pytest.approx(a_con, abs=1e-9) == 0.38

    # Status flips reproduce paper Table 2 shape (TT=27, TF=23, FT=11, FF=39).
    # Build paired-list with those counts.
    base = ([True] * 27 + [True] * 23 + [False] * 11 + [False] * 39)
    cons = ([True] * 27 + [False] * 23 + [True] * 11 + [False] * 39)
    flips = compute_status_flips(base, cons)
    assert flips == {"TT": 27, "TF": 23, "FT": 11, "FF": 39}


# Test 6 — max-trials parameter respected; argparse range is [1, 10].
def test_max_trials_range_contract() -> None:
    # Paper Section 4.3 — max_trials=5. Harness allows [1, 10] (paper Section
    # 3.3 hyperparam range). Outside that range, run.py argparse rejects.
    from elai_inference.c37_snowballing.run import PAPER_MAX_TRIALS, main

    assert PAPER_MAX_TRIALS == 5, "paper Section 4.3 default"

    # Out-of-range value -> SystemExit via parser.error.
    with pytest.raises(SystemExit):
        main(
            [
                "--pool",
                "A",
                "--max-trials",
                "99",
                "--output",
                "/tmp/dummy.jsonl",
            ]
        )
    with pytest.raises(SystemExit):
        main(
            [
                "--pool",
                "A",
                "--max-trials",
                "0",
                "--output",
                "/tmp/dummy.jsonl",
            ]
        )


# Bonus contract — error-distribution function shape (per-class dict
# with every enum member present, sums to 1.0 on non-empty input).
def test_error_distribution_shape() -> None:
    diagnoses = [
        ReflectorOutput(error_type="FORMATTING_MISMATCH", correction_rule="x"),
        ReflectorOutput(error_type="FORMATTING_MISMATCH", correction_rule="x"),
        ReflectorOutput(error_type="RETRIEVAL_FOCUS", correction_rule="x"),
    ]
    dist = compute_error_distribution(diagnoses)
    assert set(dist.keys()) == set(ERROR_TYPES)
    assert pytest.approx(sum(dist.values()), abs=1e-9) == 1.0
    assert pytest.approx(dist["FORMATTING_MISMATCH"], abs=1e-9) == 2.0 / 3.0
    assert pytest.approx(dist["RETRIEVAL_FOCUS"], abs=1e-9) == 1.0 / 3.0
    assert dist["HALLUCINATION"] == 0.0  # unobserved -> 0 not missing
