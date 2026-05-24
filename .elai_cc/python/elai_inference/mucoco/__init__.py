"""C-36 MuCoCo Mutation Operator Catalog (sidecar-side).

Implements the 11 semantic-preserving Python source mutations from
arXiv:2604.19086 (MuCoCo: Automated Consistency Testing of Code LLMs)
using only the Python standard library (ast / tokenize / re), per
Appendix G of the paper.

Public surface (re-exported here for ergonomic imports):
  - MutationResult, SkipReason   (result.py)
  - apply(name, source, language="python") -> MutationResult  (registry.apply)
  - list_operators() -> list[str]                              (registry)
  - get_applicability_matrix() -> dict[str, dict[str, bool]]   (registry)

Paper interface map: F:\\.ELAI_workflow\\workflow\\papers\\mucoco-2604.19086-interface.md
"""

from elai_inference.mucoco.registry import (
    ALL_OPERATORS,
    apply,
    get_applicability_matrix,
    list_operators,
)
from elai_inference.mucoco.result import MutationResult

__all__ = [
    "ALL_OPERATORS",
    "MutationResult",
    "apply",
    "get_applicability_matrix",
    "list_operators",
]
