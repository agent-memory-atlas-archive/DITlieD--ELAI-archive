"""C-37 metric implementations for Structure Snowballing replication.

Paper Section 7.1 Tables 1-3 (verbatim, n=100):
  Baseline accuracy:          0.50
  Constrained accuracy:       0.38   (delta -0.12; McNemar p~=0.059)
  Status flips: TT=27, TF=23, FT=11, FF=39
  Error-type distribution (constrained, trial 1): FORMATTING_MISMATCH=96,
                          RETRIEVAL_FOCUS=4, other=0
  Token consumption (constrained): mean=3183.48, max=8976.25, std=989.11

Operational threshold for "structure snowballing observed" (per
workflow/papers/structure-snowballing-source-interface.md):
  - formatting_fraction > 0.50 -> failure mode manifested (REPLICATION_THRESHOLD)
  - formatting_fraction > 0.90 -> paper-equivalent severity

NOTE: these are MEASUREMENT functions, not pass/fail gates. The harness
emits the numbers; the analysis script decides whether to FLIP the ELAI
auditor default per C-37 plan §4 decision matrix.
"""

from __future__ import annotations

from collections import Counter
from typing import Iterable

from elai_inference.c37_snowballing.types import (
    ERROR_TYPES,
    ReflectorOutput,
)


# Paper-grounded thresholds (per source interface map):
#   formatting_fraction > REPLICATION_THRESHOLD  -> failure mode observed
#   formatting_fraction > SEVERITY_THRESHOLD     -> paper-equivalent severity
REPLICATION_THRESHOLD: float = 0.50
SEVERITY_THRESHOLD: float = 0.90

# Paper-grounded accuracy expectation (Tables 1-2): constrained accuracy
# SHOULD be below 0.45 if methodology has not diverged. NOT a pass/fail
# gate — the analyzer surfaces deviations.
CONSTRAINED_ACCURACY_CEILING: float = 0.45


def compute_snowballing_rate(diagnoses: Iterable[ReflectorOutput]) -> float:
    """Fraction of diagnoses classified as FORMATTING_MISMATCH.

    Paper Table 3 reports 96/100 = 0.96 for the constrained run. The
    contract test threshold is REPLICATION_THRESHOLD=0.50 (failure mode
    observed) or SEVERITY_THRESHOLD=0.90 (paper-equivalent severity).

    Returns 0.0 when `diagnoses` is empty (no division by zero).
    """
    diagnoses_list = list(diagnoses)
    if not diagnoses_list:
        return 0.0
    fm = sum(1 for d in diagnoses_list if d.error_type == "FORMATTING_MISMATCH")
    return fm / len(diagnoses_list)


def compute_accuracy(verdicts: Iterable[bool]) -> float:
    """Fraction of verdicts equal to True (binary semantic-equivalence).

    Paper Table 1: baseline=0.50, constrained=0.38. Returns 0.0 on empty
    input.
    """
    verdicts_list = list(verdicts)
    if not verdicts_list:
        return 0.0
    return sum(1 for v in verdicts_list if v) / len(verdicts_list)


def compute_error_distribution(
    diagnoses: Iterable[ReflectorOutput],
) -> dict[str, float]:
    """Per-class fraction over the 5-class enum.

    Returns a dict keyed by every member of ERROR_TYPES so downstream
    consumers can read missing classes as 0.0 without KeyError. Sums to
    1.0 within float tolerance when input is non-empty.
    """
    diagnoses_list = list(diagnoses)
    counts: dict[str, int] = {k: 0 for k in ERROR_TYPES}
    for d in diagnoses_list:
        counts[d.error_type] = counts.get(d.error_type, 0) + 1
    total = sum(counts.values())
    if total == 0:
        return {k: 0.0 for k in ERROR_TYPES}
    return {k: counts[k] / total for k in ERROR_TYPES}


def compute_status_flips(
    baseline_verdicts: list[bool],
    constrained_verdicts: list[bool],
) -> dict[str, int]:
    """Paper Table 2 status-flip matrix: TT / TF / FT / FF counts.

    Paired comparison: both lists MUST have equal length and the
    paired-by-index questions must be the same (caller's responsibility).
    Raises ValueError on length mismatch — silent-drop discipline.
    """
    if len(baseline_verdicts) != len(constrained_verdicts):
        raise ValueError(
            f"status-flip inputs differ in length: "
            f"baseline={len(baseline_verdicts)} constrained={len(constrained_verdicts)}"
        )
    tt = sum(1 for b, c in zip(baseline_verdicts, constrained_verdicts) if b and c)
    tf = sum(1 for b, c in zip(baseline_verdicts, constrained_verdicts) if b and not c)
    ft = sum(1 for b, c in zip(baseline_verdicts, constrained_verdicts) if not b and c)
    ff = sum(1 for b, c in zip(baseline_verdicts, constrained_verdicts) if not b and not c)
    return {"TT": tt, "TF": tf, "FT": ft, "FF": ff}


def is_replication_observed(formatting_fraction: float) -> bool:
    """True iff the failure mode is observed at the replication threshold.

    Paper-grounded threshold REPLICATION_THRESHOLD=0.50 per source
    interface map.
    """
    return formatting_fraction > REPLICATION_THRESHOLD


def is_paper_equivalent_severity(formatting_fraction: float) -> bool:
    """True iff the formatting fraction matches paper severity (>=0.90).

    Paper Table 3 reports 0.96.
    """
    return formatting_fraction > SEVERITY_THRESHOLD
