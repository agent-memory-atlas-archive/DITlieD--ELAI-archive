"""C-37 Reflector schema + 5-class error taxonomy.

Paper grounding (arXiv:2604.06066 Section 3.1 + Section 3.3):
  The Reflector is constrained via Outlines (Willard & Louf 2023) compiled
  from this exact pydantic schema. The FSM masks the logit distribution at
  every decode step so that tokens violating either JSON syntax or the
  5-Literal enum membership receive zero probability mass.

Schema-class count == 5 is a paper-explicit ceiling: Section 3.1 cites
Nathani et al. 2023 on cognitive overload at ROSCOE's 10-class granularity
for sub-70B models. Do NOT expand this enum without re-grounding.
"""

from __future__ import annotations

from typing import Literal

from pydantic import BaseModel, Field, field_validator


# Paper Section 3.1 — 5-class error taxonomy (verbatim names).
ErrorType = Literal[
    "RETRIEVAL_FOCUS",
    "BRIDGE_FAILURE",
    "HALLUCINATION",
    "INFERENCE_ERROR",
    "FORMATTING_MISMATCH",
]

# Set form used by metrics + contract assertions. Order matches the paper's
# Section 3.1 enumeration order (used in run.py argparse default).
ERROR_TYPES: tuple[str, ...] = (
    "RETRIEVAL_FOCUS",
    "BRIDGE_FAILURE",
    "HALLUCINATION",
    "INFERENCE_ERROR",
    "FORMATTING_MISMATCH",
)


class ReflectorOutput(BaseModel):
    """Reflector output schema bound by the Outlines FSM mask.

    Paper invariants enforced here:
      - error_type is one of exactly 5 Literal members (FSM-guaranteed by
        Outlines but re-asserted here for harness-side defence-in-depth)
      - correction_rule is a non-empty string (paper Section 3.3 treats an
        empty rule as a malformed trial)

    No additional fields: the paper's pydantic class has exactly these two
    fields; adding more would diverge from the FSM the paper reports on.
    """

    error_type: ErrorType
    correction_rule: str = Field(min_length=1)

    @field_validator("correction_rule")
    @classmethod
    def correction_rule_non_blank(cls, v: str) -> str:
        if not v.strip():
            raise ValueError(
                "correction_rule must contain non-whitespace content "
                "(paper Section 3.3 contract)"
            )
        return v


class TrialRecord(BaseModel):
    """One trial within a question's lifetime (paper Section 3.3 trajectory).

    The full Trajectory is `list[TrialRecord]`; episodic memory accumulates
    the ReflectorOutput from failed trials and is prepended to the Actor
    prompt on the next trial.
    """

    trial_idx: int = Field(ge=1, le=5)  # paper max_trials=5
    actor_output: str
    evaluator_verdict: bool
    reflector_output: ReflectorOutput | None  # None when evaluator returned True


class QuestionResult(BaseModel):
    """Per-question result for JSONL output.

    Status is `solved` when any trial's evaluator_verdict was True;
    `failed` when all 5 trials returned False. `final_answer` is the
    Actor output on the last trial regardless of status.
    """

    question_id: str
    status: Literal["solved", "failed"]
    n_trials: int = Field(ge=1, le=5)
    final_answer: str
    gold_answer: str
    trajectory: list[TrialRecord]
    token_count: int = Field(ge=0)
