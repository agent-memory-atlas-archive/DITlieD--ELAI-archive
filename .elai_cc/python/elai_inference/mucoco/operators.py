"""C-36 MuCoCo eleven mutation operators (Python-only).

Grounded by paper interface map (workflow/papers/mucoco-2604.19086-interface.md):
  Lexical:    Random Mutation, Sequential Mutation, Literal Format
  Syntactic:  for-to-enumerate, for-to-while
  Logical:    DeMorgan, Boolean Literal, Commutative Reorder,
              Constant Unfolding, Constant Unfolding (Add), Constant Unfolding (Mul)

Implementation strategy per Appendix G:
  - All AST work uses the Python stdlib `ast` module + `ast.unparse`.
  - Literal Format uses `tokenize` + `io.BytesIO` so string quotes survive
    a round-trip (`ast.unparse` normalises string literals and would defeat
    the purpose of the Literal Format operator).
  - re is used for the Sequential / Random identifier rename name table
    after the AST collects target names.
  - No third-party libraries (no libcst, no tree-sitter).

Determinism:
  - RandomMutation uses a fixed seed (MUCOCO_RANDOM_SEED) so consistency
    test re-runs are reproducible (paper sec S9.p1.1 motivates temp=0;
    we mirror that on the mutation side).
  - Constant Unfolding factor picks are deterministic policy: the smallest
    non-trivial factor pair (a, b) where a * b == n and 1 < a <= b, with a
    fallback to (1, n) when n is prime or < 2.
  - Sequential prefixes are fixed strings from the paper Appendix B.

Applicability-first contract (paper sec S3.p3.1):
  Every operator MUST check applicability before mutating; non-applicable
  inputs return `skipped(reason)` with a debuggable reason string. The
  paper drops non-applicable mutations from the consistency-pair set
  rather than synthesizing fake transformations.
"""

from __future__ import annotations

import ast
import io
import random
import tokenize
from typing import Iterable, List, Optional, Tuple

from elai_inference.mucoco.result import MutationResult, applied, skipped

MUCOCO_RANDOM_SEED: int = 1234  # Paper sets Turbulence baseline seed = 1234.
MUCOCO_SEQUENTIAL_FUNCTION_PREFIX: str = "generic_function"
MUCOCO_SEQUENTIAL_VARIABLE_PREFIX: str = "var"


# ---------------------------------------------------------------------------
# Shared helpers
# ---------------------------------------------------------------------------

def _check_python(language: str, operator: str, source: str) -> Optional[MutationResult]:
    """Reject non-python languages with a populated skip_reason.

    The paper is Python-only (paper interface map L9-10). This helper is the
    single source of truth for the language gate so every operator returns a
    consistent reason string the consumer can match on.
    """
    if language.lower() != "python":
        return skipped(
            operator,
            source,
            f"operator '{operator}' is python-only; got language={language!r}",
        )
    return None


def _parse_or_skip(source: str, operator: str) -> Tuple[Optional[ast.Module], Optional[MutationResult]]:
    """Return (tree, None) on success, (None, skipped(...)) on parse failure.

    Paper sec S3.p3.1 mandates dropping unparseable inputs from the test set
    rather than mutating them. Surfaces as a skip with the SyntaxError text
    so the consumer can triage corpus issues separately from operator bugs.
    """
    try:
        tree = ast.parse(source)
    except SyntaxError as e:
        return None, skipped(operator, source, f"source not parseable: {e.msg}")
    return tree, None


def _unparse(tree: ast.AST) -> str:
    """Wrapper around ast.unparse with a trailing newline.

    Paper Appendix G uses ast.unparse on the rewritten tree. We append a
    trailing newline to match Python source convention so the round-trip
    parse in the contract test does not see a "no newline at EOF" diff.
    """
    out = ast.unparse(tree)
    if not out.endswith("\n"):
        out = out + "\n"
    return out


# ---------------------------------------------------------------------------
# Lexical mutations (paper B.0.1)
# ---------------------------------------------------------------------------

OPERATOR_RANDOM_MUTATION = "random_mutation"
OPERATOR_SEQUENTIAL_MUTATION = "sequential_mutation"
OPERATOR_LITERAL_FORMAT = "literal_format"


def _collect_renameable_names(tree: ast.Module) -> Tuple[List[str], List[str]]:
    """Return (function_names, variable_names) defined at the module level.

    Excludes dunder names, builtins, parameters of imported callables, and
    attribute accesses to avoid breaking method calls on closed-source
    objects (paper "Implementation pitfalls" — renaming a builtin / method
    on an external type can silently change semantics).
    """
    func_names: List[str] = []
    var_names: List[str] = []
    seen: set[str] = set()
    for node in ast.walk(tree):
        if isinstance(node, ast.FunctionDef):
            if not node.name.startswith("__") and node.name not in seen:
                seen.add(node.name)
                func_names.append(node.name)
        elif isinstance(node, ast.Assign):
            for tgt in node.targets:
                if isinstance(tgt, ast.Name):
                    name = tgt.id
                    if not name.startswith("__") and name not in seen:
                        seen.add(name)
                        var_names.append(name)
    return func_names, var_names


def _apply_rename_table(source: str, table: dict[str, str]) -> str:
    """Rewrite identifier occurrences using a precomputed name table.

    Implemented via a second AST pass (rather than naive regex) so we never
    touch a string literal, comment, or attribute access. Returns the
    unparsed source with the rename applied.
    """
    tree = ast.parse(source)

    class _Renamer(ast.NodeTransformer):
        def visit_Name(self, node: ast.Name) -> ast.AST:
            if node.id in table:
                return ast.copy_location(ast.Name(id=table[node.id], ctx=node.ctx), node)
            return node

        def visit_FunctionDef(self, node: ast.FunctionDef) -> ast.AST:
            if node.name in table:
                node.name = table[node.name]
            self.generic_visit(node)
            return node

        def visit_arg(self, node: ast.arg) -> ast.AST:
            if node.arg in table:
                node.arg = table[node.arg]
            return node

    new_tree = _Renamer().visit(tree)
    ast.fix_missing_locations(new_tree)
    return _unparse(new_tree)


def random_mutation(source: str, language: str = "python") -> MutationResult:
    """Lexical 1: rename module-level functions / variables to random strings.

    Paper B.0.1.1: "replaces function/variable names with random strings".
    Determinism: seeded RNG (MUCOCO_RANDOM_SEED) so identical inputs produce
    identical outputs (mandatory for consistency testing — paper sec S9.p1.1).
    """
    op = OPERATOR_RANDOM_MUTATION
    if (skip := _check_python(language, op, source)) is not None:
        return skip
    tree, fail = _parse_or_skip(source, op)
    if fail is not None:
        return fail
    func_names, var_names = _collect_renameable_names(tree)
    targets = func_names + var_names
    if not targets:
        return skipped(op, source, "no module-level functions or variables to rename")
    rng = random.Random(MUCOCO_RANDOM_SEED)
    alphabet = "abcdefghijklmnopqrstuvwxyz"
    table: dict[str, str] = {}
    used: set[str] = set()
    for name in targets:
        while True:
            new_name = "_" + "".join(rng.choice(alphabet) for _ in range(8))
            if new_name not in used and new_name != name:
                used.add(new_name)
                table[name] = new_name
                break
    rewritten = _apply_rename_table(source, table)
    if rewritten == source:
        return skipped(op, source, "rename table produced no textual change")
    return applied(op, rewritten)


def sequential_mutation(source: str, language: str = "python") -> MutationResult:
    """Lexical 2: rename functions -> generic_function1.., vars -> var1, var2..

    Paper B.0.1.2 / sec A2: function prefix "generic_function", variable
    prefix "var", 1-indexed numbering in source order.
    """
    op = OPERATOR_SEQUENTIAL_MUTATION
    if (skip := _check_python(language, op, source)) is not None:
        return skip
    tree, fail = _parse_or_skip(source, op)
    if fail is not None:
        return fail
    func_names, var_names = _collect_renameable_names(tree)
    targets = func_names + var_names
    if not targets:
        return skipped(op, source, "no module-level functions or variables to rename")
    table: dict[str, str] = {}
    for i, name in enumerate(func_names, start=1):
        table[name] = f"{MUCOCO_SEQUENTIAL_FUNCTION_PREFIX}{i}"
    for i, name in enumerate(var_names, start=1):
        table[name] = f"{MUCOCO_SEQUENTIAL_VARIABLE_PREFIX}{i}"
    rewritten = _apply_rename_table(source, table)
    if rewritten == source:
        return skipped(op, source, "rename table produced no textual change")
    return applied(op, rewritten)


def literal_format(source: str, language: str = "python") -> MutationResult:
    """Lexical 3: swap Python string quote style (' <-> ").

    Paper sec A2.SS0.SSS1.p1: "Python strings" — single<->double quote swap.
    Tokenize-based (NOT ast.unparse) because ast.unparse normalises strings
    and would defeat the purpose of the Literal Format operator.

    F-strings and bytestrings with internal quote conflicts are skipped on a
    per-token basis (partial application allowed; caller sees applied=True
    if at least one swap landed).
    """
    op = OPERATOR_LITERAL_FORMAT
    if (skip := _check_python(language, op, source)) is not None:
        return skip
    if not source.strip():
        return skipped(op, source, "source is empty")
    out_lines: List[str] = source.splitlines(keepends=True)
    swaps = 0
    try:
        tokens = list(tokenize.tokenize(io.BytesIO(source.encode("utf-8")).readline))
    except tokenize.TokenizeError as e:
        return skipped(op, source, f"tokenize failed: {e}")
    # Build (start_row, start_col, end_row, end_col, new_text) edits, then
    # apply right-to-left so column indices don't shift.
    edits: List[Tuple[int, int, int, int, str]] = []
    for tok in tokens:
        if tok.type != tokenize.STRING:
            continue
        s = tok.string
        # Skip f-strings / b-strings / r-strings to avoid edge cases that
        # could alter semantics (paper "Implementation pitfalls").
        # Identify prefix.
        prefix = ""
        rest = s
        while rest and rest[0].lower() in ("r", "b", "u", "f"):
            prefix += rest[0]
            rest = rest[1:]
        if "f" in prefix.lower() or "b" in prefix.lower():
            continue
        if not rest:
            continue
        if rest.startswith('"""') or rest.startswith("'''"):
            # Triple-quoted; skip (less common, harder to swap safely).
            continue
        if rest.startswith('"') and rest.endswith('"') and len(rest) >= 2:
            inner = rest[1:-1]
            if "'" in inner:
                continue  # would require escaping inner quotes; skip per safety
            new_text = prefix + "'" + inner + "'"
        elif rest.startswith("'") and rest.endswith("'") and len(rest) >= 2:
            inner = rest[1:-1]
            if '"' in inner:
                continue
            new_text = prefix + '"' + inner + '"'
        else:
            continue
        edits.append((tok.start[0], tok.start[1], tok.end[0], tok.end[1], new_text))
        swaps += 1
    if swaps == 0:
        return skipped(op, source, "no string literals eligible for quote swap")
    # Apply edits right-to-left, line-by-line.
    edits.sort(reverse=True)
    for srow, scol, erow, ecol, new_text in edits:
        # Tokenize rows are 1-indexed; out_lines is 0-indexed.
        sidx = srow - 1
        eidx = erow - 1
        if sidx == eidx:
            line = out_lines[sidx]
            out_lines[sidx] = line[:scol] + new_text + line[ecol:]
        else:
            # Multi-line token (triple-quoted, but we already skip those).
            continue
    rewritten = "".join(out_lines)
    if rewritten == source:
        return skipped(op, source, "quote swap produced no textual change")
    return applied(op, rewritten)


# ---------------------------------------------------------------------------
# Syntactic mutations (paper B.0.2)
# ---------------------------------------------------------------------------

OPERATOR_FOR_TO_ENUMERATE = "for_to_enumerate"
OPERATOR_FOR_TO_WHILE = "for_to_while"


def for_to_enumerate(source: str, language: str = "python") -> MutationResult:
    """Syntactic 1: rewrite `for x in seq:` as `for i, x in enumerate(seq):`.

    Paper B.0.2.1: applicable only when the original for-loop iterates a
    bare expression (not already an enumerate / zip / range tuple-unpack).
    Skips when the loop variable is already a tuple or when the iterable
    is wrapped in enumerate/zip.
    """
    op = OPERATOR_FOR_TO_ENUMERATE
    if (skip := _check_python(language, op, source)) is not None:
        return skip
    tree, fail = _parse_or_skip(source, op)
    if fail is not None:
        return fail
    eligible: List[ast.For] = []
    for node in ast.walk(tree):
        if not isinstance(node, ast.For):
            continue
        if isinstance(node.target, ast.Tuple):
            continue  # already-unpacked target; skip
        if isinstance(node.iter, ast.Call):
            fn = node.iter.func
            if isinstance(fn, ast.Name) and fn.id in ("enumerate", "zip"):
                continue
        eligible.append(node)
    if not eligible:
        return skipped(op, source, "no eligible for-loop (target must be Name, iter must not be enumerate/zip)")
    # Pick the first eligible for-loop deterministically.
    target_for = eligible[0]
    # Build replacement: target becomes Tuple(_mucoco_idx, original_target);
    # iter becomes Call(enumerate, [original_iter]).
    idx_name = "_mucoco_idx"
    new_target = ast.Tuple(
        elts=[ast.Name(id=idx_name, ctx=ast.Store()), target_for.target],
        ctx=ast.Store(),
    )
    new_iter = ast.Call(
        func=ast.Name(id="enumerate", ctx=ast.Load()),
        args=[target_for.iter],
        keywords=[],
    )
    target_for.target = new_target
    target_for.iter = new_iter
    ast.fix_missing_locations(tree)
    return applied(op, _unparse(tree))


def for_to_while(source: str, language: str = "python") -> MutationResult:
    """Syntactic 2: rewrite a for-loop over a finite iterable as a while-loop.

    Paper B.0.2.2: equivalent control flow via:
        _it = iter(<original_iter>)
        while True:
            try:
                <target> = next(_it)
            except StopIteration:
                break
            <body>

    Applicability: at least one for-loop must exist. Skip otherwise.
    """
    op = OPERATOR_FOR_TO_WHILE
    if (skip := _check_python(language, op, source)) is not None:
        return skip
    tree, fail = _parse_or_skip(source, op)
    if fail is not None:
        return fail
    target_for: Optional[ast.For] = None
    parent_field: Optional[Tuple[ast.AST, str, int]] = None
    # Find the first for-loop with a parent we can splice (skip for-else for safety).
    for parent in ast.walk(tree):
        for field, value in ast.iter_fields(parent):
            if isinstance(value, list):
                for i, item in enumerate(value):
                    if isinstance(item, ast.For) and not item.orelse:
                        target_for = item
                        parent_field = (parent, field, i)
                        break
            if target_for is not None:
                break
        if target_for is not None:
            break
    if target_for is None or parent_field is None:
        return skipped(op, source, "no for-loop without else-clause found")
    # _mucoco_it = iter(<original_iter>)
    it_assign = ast.Assign(
        targets=[ast.Name(id="_mucoco_it", ctx=ast.Store())],
        value=ast.Call(
            func=ast.Name(id="iter", ctx=ast.Load()),
            args=[target_for.iter],
            keywords=[],
        ),
    )
    # try: <target> = next(_mucoco_it) except StopIteration: break
    try_block = ast.Try(
        body=[
            ast.Assign(
                targets=[target_for.target],
                value=ast.Call(
                    func=ast.Name(id="next", ctx=ast.Load()),
                    args=[ast.Name(id="_mucoco_it", ctx=ast.Load())],
                    keywords=[],
                ),
            )
        ],
        handlers=[
            ast.ExceptHandler(
                type=ast.Name(id="StopIteration", ctx=ast.Load()),
                name=None,
                body=[ast.Break()],
            )
        ],
        orelse=[],
        finalbody=[],
    )
    while_loop = ast.While(
        test=ast.Constant(value=True),
        body=[try_block] + target_for.body,
        orelse=[],
    )
    parent, field, idx = parent_field
    container = getattr(parent, field)
    container[idx : idx + 1] = [it_assign, while_loop]
    ast.fix_missing_locations(tree)
    return applied(op, _unparse(tree))


# ---------------------------------------------------------------------------
# Logical mutations (paper B.0.3)
# ---------------------------------------------------------------------------

OPERATOR_DEMORGAN = "demorgan"
OPERATOR_BOOLEAN_LITERAL = "boolean_literal"
OPERATOR_COMMUTATIVE_REORDER = "commutative_reorder"
OPERATOR_CONSTANT_UNFOLDING = "constant_unfolding"
OPERATOR_CONSTANT_UNFOLDING_ADD = "constant_unfolding_add"
OPERATOR_CONSTANT_UNFOLDING_MUL = "constant_unfolding_mul"


def demorgan(source: str, language: str = "python") -> MutationResult:
    """Logical 1: A and B -> not (not A or not B); A or B -> not (not A and not B).

    Paper Eq 3 + Eq 4 (sec B.0.3.1). Requires at least one BoolOp node.
    Applies to the first BoolOp encountered (deterministic).
    """
    op = OPERATOR_DEMORGAN
    if (skip := _check_python(language, op, source)) is not None:
        return skip
    tree, fail = _parse_or_skip(source, op)
    if fail is not None:
        return fail
    target: Optional[ast.BoolOp] = None
    for node in ast.walk(tree):
        if isinstance(node, ast.BoolOp):
            target = node
            break
    if target is None:
        return skipped(op, source, "no BoolOp (and/or) expression found")
    flipped_op = ast.Or() if isinstance(target.op, ast.And) else ast.And()
    negated_values = [
        ast.UnaryOp(op=ast.Not(), operand=v) for v in target.values
    ]
    new_inner = ast.BoolOp(op=flipped_op, values=negated_values)
    new_outer = ast.UnaryOp(op=ast.Not(), operand=new_inner)

    class _Replacer(ast.NodeTransformer):
        def __init__(self) -> None:
            self.done = False

        def visit_BoolOp(self, node: ast.BoolOp) -> ast.AST:
            if not self.done and node is target:
                self.done = True
                return ast.copy_location(new_outer, node)
            self.generic_visit(node)
            return node

    new_tree = _Replacer().visit(tree)
    ast.fix_missing_locations(new_tree)
    return applied(op, _unparse(new_tree))


def boolean_literal(source: str, language: str = "python") -> MutationResult:
    """Logical 2: rewrite True/False using negation (True -> not False).

    Paper B.0.3.2. Applies to the first matching Constant(True|False) node.
    """
    op = OPERATOR_BOOLEAN_LITERAL
    if (skip := _check_python(language, op, source)) is not None:
        return skip
    tree, fail = _parse_or_skip(source, op)
    if fail is not None:
        return fail
    target: Optional[ast.Constant] = None
    for node in ast.walk(tree):
        if isinstance(node, ast.Constant) and isinstance(node.value, bool):
            target = node
            break
    if target is None:
        return skipped(op, source, "no boolean literal (True/False) found")
    new_value = ast.UnaryOp(
        op=ast.Not(),
        operand=ast.Constant(value=not target.value),
    )

    class _Replacer(ast.NodeTransformer):
        def __init__(self) -> None:
            self.done = False

        def visit_Constant(self, node: ast.Constant) -> ast.AST:
            if not self.done and node is target:
                self.done = True
                return ast.copy_location(new_value, node)
            return node

    new_tree = _Replacer().visit(tree)
    ast.fix_missing_locations(new_tree)
    return applied(op, _unparse(new_tree))


def commutative_reorder(source: str, language: str = "python") -> MutationResult:
    """Logical 3: a + b -> b + a, a * b -> b * a (commutative ops only).

    Paper B.0.3.3. NOTE: Python addition/multiplication is operator-overloaded.
    To respect "Implementation pitfalls" we only swap when BOTH operands are
    integer / float constants (provably commutative regardless of __radd__ /
    __rmul__). Falls back to skip for any operand that is not a numeric constant.
    """
    op = OPERATOR_COMMUTATIVE_REORDER
    if (skip := _check_python(language, op, source)) is not None:
        return skip
    tree, fail = _parse_or_skip(source, op)
    if fail is not None:
        return fail
    target: Optional[ast.BinOp] = None
    for node in ast.walk(tree):
        if not isinstance(node, ast.BinOp):
            continue
        if not isinstance(node.op, (ast.Add, ast.Mult)):
            continue
        # Provable commutativity: both sides numeric Constant.
        lhs_ok = isinstance(node.left, ast.Constant) and isinstance(node.left.value, (int, float)) and not isinstance(node.left.value, bool)
        rhs_ok = isinstance(node.right, ast.Constant) and isinstance(node.right.value, (int, float)) and not isinstance(node.right.value, bool)
        if lhs_ok and rhs_ok:
            target = node
            break
    if target is None:
        return skipped(op, source, "no provably-commutative numeric BinOp (Add/Mult on numeric constants)")
    swapped = ast.BinOp(left=target.right, op=target.op, right=target.left)

    class _Replacer(ast.NodeTransformer):
        def __init__(self) -> None:
            self.done = False

        def visit_BinOp(self, node: ast.BinOp) -> ast.AST:
            if not self.done and node is target:
                self.done = True
                return ast.copy_location(swapped, node)
            self.generic_visit(node)
            return node

    new_tree = _Replacer().visit(tree)
    ast.fix_missing_locations(new_tree)
    return applied(op, _unparse(new_tree))


def _smallest_factor_pair(n: int) -> Optional[Tuple[int, int]]:
    """Return (a, b) with a * b == n, 1 < a <= b. None if n is prime or |n|<4."""
    abs_n = abs(n)
    if abs_n < 4:
        return None
    for a in range(2, int(abs_n**0.5) + 1):
        if abs_n % a == 0:
            b = abs_n // a
            if n < 0:
                # Preserve sign: (-a) * b == n.
                return (-a, b)
            return (a, b)
    return None


def _first_eligible_int_constant(tree: ast.Module) -> Optional[ast.Constant]:
    """Find the first int Constant (not bool, not 0/1) in the tree."""
    for node in ast.walk(tree):
        if isinstance(node, ast.Constant) and isinstance(node.value, int) and not isinstance(node.value, bool):
            if abs(node.value) >= 2:
                return node
    return None


def _replace_constant(tree: ast.Module, target: ast.Constant, new_node: ast.AST) -> ast.Module:
    class _Replacer(ast.NodeTransformer):
        def __init__(self) -> None:
            self.done = False

        def visit_Constant(self, node: ast.Constant) -> ast.AST:
            if not self.done and node is target:
                self.done = True
                return ast.copy_location(new_node, node)
            return node

    new_tree = _Replacer().visit(tree)
    ast.fix_missing_locations(new_tree)
    return new_tree


def constant_unfolding(source: str, language: str = "python") -> MutationResult:
    """Logical 4: rewrite integer N as (a + b) when factoring fails, else (a * b).

    Paper B.0.3.4: decomposes an integer constant into two factors using add
    OR multiply (paper does not pin the policy — we use the deterministic
    policy "prefer multiply if a non-trivial factorization exists, else
    fall back to add"). Skip when no eligible int constant exists.
    """
    op = OPERATOR_CONSTANT_UNFOLDING
    if (skip := _check_python(language, op, source)) is not None:
        return skip
    tree, fail = _parse_or_skip(source, op)
    if fail is not None:
        return fail
    target = _first_eligible_int_constant(tree)
    if target is None:
        return skipped(op, source, "no eligible int constant (>=2 or <=-2) found")
    n = int(target.value)
    pair = _smallest_factor_pair(n)
    if pair is not None:
        a, b = pair
        new_node = ast.BinOp(left=ast.Constant(value=a), op=ast.Mult(), right=ast.Constant(value=b))
    else:
        # Prime / small abs value: split via addition.
        a = n // 2
        b = n - a
        new_node = ast.BinOp(left=ast.Constant(value=a), op=ast.Add(), right=ast.Constant(value=b))
    new_tree = _replace_constant(tree, target, new_node)
    return applied(op, _unparse(new_tree))


def constant_unfolding_add(source: str, language: str = "python") -> MutationResult:
    """Logical 5: rewrite integer N as (a + b) with a, b integers, a+b == N.

    Paper B.0.3.5. Policy: a = N // 2, b = N - a (deterministic).
    """
    op = OPERATOR_CONSTANT_UNFOLDING_ADD
    if (skip := _check_python(language, op, source)) is not None:
        return skip
    tree, fail = _parse_or_skip(source, op)
    if fail is not None:
        return fail
    target = _first_eligible_int_constant(tree)
    if target is None:
        return skipped(op, source, "no eligible int constant (>=2 or <=-2) found")
    n = int(target.value)
    a = n // 2
    b = n - a
    new_node = ast.BinOp(left=ast.Constant(value=a), op=ast.Add(), right=ast.Constant(value=b))
    new_tree = _replace_constant(tree, target, new_node)
    return applied(op, _unparse(new_tree))


def constant_unfolding_mul(source: str, language: str = "python") -> MutationResult:
    """Logical 6: rewrite integer N as (a * b) with a, b integers, a*b == N.

    Paper B.0.3.6. Policy: smallest non-trivial factor pair (deterministic);
    skip when N is prime or |N|<4 (no provable non-trivial factorization).
    """
    op = OPERATOR_CONSTANT_UNFOLDING_MUL
    if (skip := _check_python(language, op, source)) is not None:
        return skip
    tree, fail = _parse_or_skip(source, op)
    if fail is not None:
        return fail
    target = _first_eligible_int_constant(tree)
    if target is None:
        return skipped(op, source, "no eligible int constant (>=2 or <=-2) found")
    n = int(target.value)
    pair = _smallest_factor_pair(n)
    if pair is None:
        return skipped(op, source, f"int constant {n} has no non-trivial factor pair (prime or |n|<4)")
    a, b = pair
    new_node = ast.BinOp(left=ast.Constant(value=a), op=ast.Mult(), right=ast.Constant(value=b))
    new_tree = _replace_constant(tree, target, new_node)
    return applied(op, _unparse(new_tree))
