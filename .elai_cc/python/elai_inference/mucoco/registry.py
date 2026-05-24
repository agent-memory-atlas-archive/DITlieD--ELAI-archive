"""C-36 MuCoCo operator registry + dispatcher.

Maps the 11 paper-grounded operator names to their callable implementations,
and exposes the language applicability matrix from the paper interface map
(workflow/papers/mucoco-2604.19086-interface.md L52-65).

Public surface:
  - ALL_OPERATORS                : tuple[str, ...]              eleven operator names
  - OPERATORS_LEXICAL/SYNTACTIC/LOGICAL : tuple groupings        paper categories
  - apply(name, source, language): MutationResult                dispatcher
  - list_operators()             : list[str]                     stable order
  - get_applicability_matrix()   : dict[str, dict[lang, bool]]   from paper

The dispatcher is the SINGLE call site Rust shells out to via the
sidecar /mucoco/mutate endpoint.
"""

from __future__ import annotations

from typing import Callable, Dict, List, Tuple

from elai_inference.mucoco.operators import (
    OPERATOR_BOOLEAN_LITERAL,
    OPERATOR_COMMUTATIVE_REORDER,
    OPERATOR_CONSTANT_UNFOLDING,
    OPERATOR_CONSTANT_UNFOLDING_ADD,
    OPERATOR_CONSTANT_UNFOLDING_MUL,
    OPERATOR_DEMORGAN,
    OPERATOR_FOR_TO_ENUMERATE,
    OPERATOR_FOR_TO_WHILE,
    OPERATOR_LITERAL_FORMAT,
    OPERATOR_RANDOM_MUTATION,
    OPERATOR_SEQUENTIAL_MUTATION,
    boolean_literal,
    commutative_reorder,
    constant_unfolding,
    constant_unfolding_add,
    constant_unfolding_mul,
    demorgan,
    for_to_enumerate,
    for_to_while,
    literal_format,
    random_mutation,
    sequential_mutation,
)
from elai_inference.mucoco.result import MutationResult, skipped

OperatorFn = Callable[[str, str], MutationResult]

OPERATORS_LEXICAL: Tuple[str, ...] = (
    OPERATOR_RANDOM_MUTATION,
    OPERATOR_SEQUENTIAL_MUTATION,
    OPERATOR_LITERAL_FORMAT,
)

OPERATORS_SYNTACTIC: Tuple[str, ...] = (
    OPERATOR_FOR_TO_ENUMERATE,
    OPERATOR_FOR_TO_WHILE,
)

OPERATORS_LOGICAL: Tuple[str, ...] = (
    OPERATOR_DEMORGAN,
    OPERATOR_BOOLEAN_LITERAL,
    OPERATOR_COMMUTATIVE_REORDER,
    OPERATOR_CONSTANT_UNFOLDING,
    OPERATOR_CONSTANT_UNFOLDING_ADD,
    OPERATOR_CONSTANT_UNFOLDING_MUL,
)

ALL_OPERATORS: Tuple[str, ...] = (
    *OPERATORS_LEXICAL,
    *OPERATORS_SYNTACTIC,
    *OPERATORS_LOGICAL,
)

_DISPATCH_TABLE: Dict[str, OperatorFn] = {
    OPERATOR_RANDOM_MUTATION: random_mutation,
    OPERATOR_SEQUENTIAL_MUTATION: sequential_mutation,
    OPERATOR_LITERAL_FORMAT: literal_format,
    OPERATOR_FOR_TO_ENUMERATE: for_to_enumerate,
    OPERATOR_FOR_TO_WHILE: for_to_while,
    OPERATOR_DEMORGAN: demorgan,
    OPERATOR_BOOLEAN_LITERAL: boolean_literal,
    OPERATOR_COMMUTATIVE_REORDER: commutative_reorder,
    OPERATOR_CONSTANT_UNFOLDING: constant_unfolding,
    OPERATOR_CONSTANT_UNFOLDING_ADD: constant_unfolding_add,
    OPERATOR_CONSTANT_UNFOLDING_MUL: constant_unfolding_mul,
}

# Sanity invariant — caught by tests + by import-time assertion to surface
# drift loudly. 11 operators is the paper-grounded count (Table 9 / Appendix B).
assert len(ALL_OPERATORS) == 11, "MuCoCo MUST register exactly 11 operators per paper Table 9"
assert set(ALL_OPERATORS) == set(_DISPATCH_TABLE), "ALL_OPERATORS and _DISPATCH_TABLE must agree"


def list_operators() -> List[str]:
    """Stable-ordered list of operator names (lexical -> syntactic -> logical)."""
    return list(ALL_OPERATORS)


def apply(name: str, source: str, language: str = "python") -> MutationResult:
    """Dispatch to a named operator. Unknown name -> skipped result.

    `language` is forwarded to the operator's own language gate; the dispatcher
    does NOT pre-filter on language so we keep skip_reason text consistent
    with the operator-specific gates.
    """
    fn = _DISPATCH_TABLE.get(name)
    if fn is None:
        return skipped(
            name or "<missing>",
            source,
            f"unknown operator {name!r}; known: {', '.join(ALL_OPERATORS)}",
        )
    return fn(source, language)


def get_applicability_matrix() -> Dict[str, Dict[str, bool]]:
    """Language applicability matrix from paper interface map L52-65.

    Cells:
      True   -- paper demonstrates + evaluates on this language
      False  -- paper-internal evidence the mutation is not directly portable
    Cells marked '?' in the interface map are OMITTED from this dict (no
    grounded answer); consumers MUST treat absence as "not applicable" rather
    than synthesize a default.
    """
    matrix: Dict[str, Dict[str, bool]] = {op: {"python": True} for op in ALL_OPERATORS}
    # Operators paper marks as NOT portable to non-Python:
    matrix[OPERATOR_LITERAL_FORMAT].update({"rust": False, "typescript": False, "java": False, "go": False})
    matrix[OPERATOR_FOR_TO_ENUMERATE].update({"rust": False, "typescript": False, "java": False, "go": False})
    return matrix
