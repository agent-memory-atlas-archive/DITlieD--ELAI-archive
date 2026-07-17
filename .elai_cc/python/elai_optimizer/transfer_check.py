"""E-8 task 6: cross-model transfer validation.

After GEPA convergence on Sonnet-reflection / Qwen-target, re-evaluate
the optimized template on (a) the same local Qwen target on a held-out
test set and (b) two cross-domain canary families per E-7 task 7. If
target re-eval drops by more than epsilon=2pp below the validation
lift, flag the candidate as overfit-risk and refuse admission.

Implements PromptBridge (arXiv:2512.01420) validity check.

Live LLM calls RUNPOD-PENDING -- the ``validate_transfer`` entry-point
returns a deterministic ``TransferReport`` when the live re-eval seam
is closed. The Rust admit path consumes the report shape; this module
is the producer.
"""

from __future__ import annotations

import dataclasses
from typing import Optional


_DEFAULT_EPSILON_PCT = 2.0


@dataclasses.dataclass(frozen=True)
class TransferReport:
    """Output of ``validate_transfer``.

    The Rust admit path inspects ``overfit_risk`` -- if True, refuse
    admission and emit ``OptimizerPhase::ArchiveRetire``.
    """

    target_model_url: str
    val_lift_pct: float
    target_reeval_lift_pct: float
    epsilon_pct: float
    overfit_risk: bool
    canary_results: list[tuple[str, float]]


def validate_transfer(
    val_lift_pct: float,
    target_reeval_lift_pct: float,
    target_model_url: str = "http://localhost:8181/v1/messages",
    canary_results: Optional[list[tuple[str, float]]] = None,
    epsilon_pct: float = _DEFAULT_EPSILON_PCT,
) -> TransferReport:
    """Return ``TransferReport`` with the overfit verdict.

    ``overfit_risk = True`` IFF the target re-eval lift dropped more
    than ``epsilon_pct`` below the validation lift. Canary results are
    passed through unchanged for inspection by the Rust admit path.
    """
    overfit = (val_lift_pct - target_reeval_lift_pct) > epsilon_pct
    return TransferReport(
        target_model_url=target_model_url,
        val_lift_pct=val_lift_pct,
        target_reeval_lift_pct=target_reeval_lift_pct,
        epsilon_pct=epsilon_pct,
        overfit_risk=overfit,
        canary_results=list(canary_results or []),
    )
