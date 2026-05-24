"""C-37 Structure Snowballing replication harness (sidecar-side).

Replicates the contract test shape from arXiv:2604.06066 ("From Hallucination
to Structure Snowballing: The Alignment Tax of Constrained Decoding in LLM
Reflection", Zhou 2026) against the local sidecar / remote pod.

The paper's central finding: constrained decoding (Outlines + pydantic FSM)
applied to an unfinetuned 8B reflector does NOT improve self-correction. It
REPLACES hallucination-snowballing with "structure snowballing" — 96% of
diagnosed errors collapse into FORMATTING_MISMATCH while semantic failures
slip through unflagged. Pool-merged accuracy 0.50 -> 0.38 (n=100, McNemar
p~=0.059).

Role mapping (paper -> ELAI):
  Actor      -> ELAI CODER  (free-text reasoning generator)
  Evaluator  -> ELAI CRITIC (semantic-equivalence judge)
  Reflector  -> ELAI REFLECTION (constrained 5-class error diagnoser)

Repo absence note (workflow/papers/structure-snowballing-source-interface.md):
  github.com/hongxuzhou/agentic_llm_structured_self_critique returns 404; this
  harness is authored against the paper text only (HIGH reliability paper /
  MEDIUM reliability without repo). Numbers below are quoted from the paper's
  Sections 3.1-3.3 + 4.3 + Tables 1-4, NOT extrapolated.

Public surface:
  - ErrorType            (5-class enum literal, paper Section 3.1)
  - ReflectorOutput      (pydantic schema matching the paper's Outlines FSM)
  - Actor / Evaluator / Reflector classes (roles.py)
  - load_pool_a / load_pool_b (dataset.py)
  - compute_snowballing_rate / compute_accuracy / REPLICATION_THRESHOLD (metrics.py)
"""

from elai_inference.c37_snowballing.types import (
    ERROR_TYPES,
    ErrorType,
    ReflectorOutput,
)

__all__ = [
    "ERROR_TYPES",
    "ErrorType",
    "ReflectorOutput",
]
