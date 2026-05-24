"""C-36 contract tests for MuCoCo mutation operators (Python-side).

Grounded by paper interface map (workflow/papers/mucoco-2604.19086-interface.md
"CONTRACT TEST STUBS" L136-184). These are scaffolding tests — they do NOT
run pytest in this session (per CLAUDE.md feedback_no_cargo_test_unless_permitted
applied transitively to long sidecar test runs).

Run when permitted with:
    cd .elai_cc/python
    python -m pytest elai_inference/tests/test_mucoco_operators.py -v

What each test pins:
  - test_eleven_operators_registered : registry has exactly 11 names
  - test_all_operators_have_dispatch : every name resolves to a callable
  - test_skip_when_inapplicable      : for_to_while on no-for source skips
  - test_non_python_language_rejected: literal_format on rust source skips
  - test_each_operator_returns_valid_python: every applied result re-parses
  - test_demorgan_introduces_negations: A and B -> not (not A or not B)
  - test_boolean_literal_flips        : True -> not False (or vice versa)
  - test_constant_unfolding_mul_skips_prime: int = 7 cannot factor non-trivially
  - test_skip_reason_required_on_skip: MutationResult guard catches drift
"""

from __future__ import annotations

import ast

import pytest

from elai_inference.mucoco import (
    ALL_OPERATORS,
    MutationResult,
    apply,
    get_applicability_matrix,
    list_operators,
)
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
)
from elai_inference.mucoco.result import applied, skipped


SAMPLE = (
    "def add(a, b):\n"
    "    if a and b:\n"
    "        for i in range(5):\n"
    "            print('hi')\n"
    "    return a + b\n"
)


def test_eleven_operators_registered():
    assert len(ALL_OPERATORS) == 11
    assert len(list_operators()) == 11


def test_all_operators_have_dispatch():
    # apply() must accept every registered name without falling into the
    # unknown-operator path.
    for op in ALL_OPERATORS:
        r = apply(op, SAMPLE, "python")
        assert isinstance(r, MutationResult)
        assert r.operator == op


def test_applicability_matrix_has_all_operators():
    matrix = get_applicability_matrix()
    assert set(matrix.keys()) == set(ALL_OPERATORS)
    # Paper grounds Python = True for every operator (interface map L52-65).
    for op in ALL_OPERATORS:
        assert matrix[op]["python"] is True
    # Operators paper marks non-portable to other langs:
    for op in (OPERATOR_LITERAL_FORMAT, OPERATOR_FOR_TO_ENUMERATE):
        for lang in ("rust", "typescript", "java", "go"):
            assert matrix[op][lang] is False


def test_each_applied_result_returns_valid_python():
    for op in ALL_OPERATORS:
        r = apply(op, SAMPLE, "python")
        if r.applied:
            ast.parse(r.mutated_source)  # round-trip parse


def test_skip_when_inapplicable_for_to_while():
    no_for = "def f(a, b): return a + b\n"
    r = apply(OPERATOR_FOR_TO_WHILE, no_for, "python")
    assert r.applied is False
    assert r.skip_reason is not None and r.skip_reason


def test_non_python_language_rejected():
    r = apply(
        OPERATOR_LITERAL_FORMAT,
        'fn main() { println!("hi"); }',
        "rust",
    )
    assert r.applied is False
    assert r.skip_reason is not None
    assert "python-only" in r.skip_reason


def test_demorgan_introduces_negations():
    r = apply(OPERATOR_DEMORGAN, "x = a and b\n", "python")
    assert r.applied is True
    assert r.mutated_source.count("not") >= 2


def test_boolean_literal_flips_to_negation():
    r = apply(OPERATOR_BOOLEAN_LITERAL, "x = True\n", "python")
    assert r.applied is True
    assert "not" in r.mutated_source


def test_commutative_reorder_swaps_numeric_constants():
    r = apply(OPERATOR_COMMUTATIVE_REORDER, "x = 1 + 2\n", "python")
    assert r.applied is True
    # 1 + 2 should become 2 + 1.
    assert "2 + 1" in r.mutated_source


def test_constant_unfolding_changes_int_literal():
    r = apply(OPERATOR_CONSTANT_UNFOLDING, "x = 6\n", "python")
    assert r.applied is True
    # 6 = 2 * 3 (smallest non-trivial factor pair).
    assert "2 * 3" in r.mutated_source


def test_constant_unfolding_mul_skips_prime():
    r = apply(OPERATOR_CONSTANT_UNFOLDING_MUL, "x = 7\n", "python")
    assert r.applied is False
    assert r.skip_reason is not None and "factor pair" in r.skip_reason


def test_constant_unfolding_add_always_applies_to_any_int_ge_2():
    r = apply(OPERATOR_CONSTANT_UNFOLDING_ADD, "x = 7\n", "python")
    assert r.applied is True
    # 7 = 3 + 4 (deterministic policy: a = 7//2 = 3, b = 7-3 = 4).
    assert "3 + 4" in r.mutated_source


def test_sequential_mutation_uses_paper_prefixes():
    src = "def my_func():\n    x = 1\n    return x\n"
    r = apply(OPERATOR_SEQUENTIAL_MUTATION, src, "python")
    assert r.applied is True
    assert "generic_function1" in r.mutated_source
    assert "var1" in r.mutated_source


def test_random_mutation_is_deterministic_across_calls():
    # MUCOCO_RANDOM_SEED = 1234 (paper-grounded) so identical inputs produce
    # identical outputs. This is the consistency-testing prerequisite.
    src = "def f():\n    x = 1\n    return x\n"
    r1 = apply(OPERATOR_RANDOM_MUTATION, src, "python")
    r2 = apply(OPERATOR_RANDOM_MUTATION, src, "python")
    assert r1.applied is True
    assert r2.applied is True
    assert r1.mutated_source == r2.mutated_source


def test_for_to_enumerate_rewrites_eligible_loop():
    src = "for x in items:\n    print(x)\n"
    r = apply(OPERATOR_FOR_TO_ENUMERATE, src, "python")
    assert r.applied is True
    assert "enumerate" in r.mutated_source


def test_for_to_while_emits_iter_next_pattern():
    src = "for x in items:\n    print(x)\n"
    r = apply(OPERATOR_FOR_TO_WHILE, src, "python")
    assert r.applied is True
    assert "while True" in r.mutated_source
    assert "next(" in r.mutated_source
    assert "StopIteration" in r.mutated_source


def test_literal_format_swaps_quote_style():
    r = apply(OPERATOR_LITERAL_FORMAT, 'x = "hello"\n', "python")
    assert r.applied is True
    assert "'hello'" in r.mutated_source


def test_unknown_operator_returns_skip_with_reason():
    r = apply("not_a_real_operator", SAMPLE, "python")
    assert r.applied is False
    assert r.skip_reason is not None
    assert "unknown operator" in r.skip_reason


def test_skip_result_with_empty_reason_raises():
    # Stub-ban contract: MutationResult guard MUST reject applied=False
    # with empty/None skip_reason.
    with pytest.raises(ValueError):
        MutationResult(operator="x", applied=False, mutated_source="x", skip_reason=None)
    with pytest.raises(ValueError):
        MutationResult(operator="x", applied=False, mutated_source="x", skip_reason="")


def test_applied_result_with_skip_reason_raises():
    # Mutually exclusive: applied=True forbids skip_reason.
    with pytest.raises(ValueError):
        MutationResult(operator="x", applied=True, mutated_source="x", skip_reason="why")


def test_applied_helper_round_trip():
    r = applied("test_op", "y = 2\n")
    assert r.applied is True
    assert r.skip_reason is None


def test_skipped_helper_round_trip():
    r = skipped("test_op", "y = 1\n", "no eligible target")
    assert r.applied is False
    assert r.skip_reason == "no eligible target"
    assert r.mutated_source == "y = 1\n"
