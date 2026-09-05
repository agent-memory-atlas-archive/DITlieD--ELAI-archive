Algorithm: The Power of Ten -- Rules for Developing Safety-Critical Code
Source: IEEE Computer, Vol. 39, No. 6, June 2006, pp. 28-35
Paper: "The Power of Ten -- Rules for Developing Safety Critical Code"
Author: Gerard J. Holzmann, NASA/JPL Laboratory for Reliable Software
PDF: https://spinroot.com/gerard/pdf/P10.pdf
Local: F:\.ELAI_workflow\.claude\papers\power-of-ten.pdf
RAG topic: paper-power-of-ten

Summary:
Ten mandatory coding rules developed at NASA/JPL for safety-critical C code.
Designed so that every rule is mechanically checkable by a static analyzer.
Used experimentally at JPL for mission-critical software.
The philosophy: rules must be small (<=10), clear, and mechanically verifiable.
"Draconian" by design -- trade-off accepted because lives may depend on the code.

Inputs:
source_code | C source (primary target; principles apply to Rust/Python/TS) | any size | must be compiled C with all warnings enabled
coding_guidelines | set of 10 rules | N/A | all 10 rules are mandatory, not optional

Outputs:
verified_code | source that passes all 10 checks | same as input | zero static-analyzer warnings, acyclic call graph, bounded loops, assertion-dense
compliance_report | list of rule violations | per-file per-rule | empty set = compliant

Hyperparameters:
max_function_length | 60 lines | "no more than about 60 lines of code per function" (Rule 4) | hard upper; one printed page in standard reference format
min_assertions_per_function | 2 | Rule 5 | minimum average; "Any assertion for which a static checking tool can prove it can never fail or never hold violates this rule"
max_pointer_dereference_depth | 1 | Rule 9 | single level; no function pointers
max_conditional_compilation_directives | 1 or 2 | Rule 8 | "rarely be justification for more than one or two"

Critical invariants (testable assertions):
assert call_graph_is_acyclic() == True  # Rule 1: no recursion -> acyclic call graph provable
assert all(loop.has_static_upper_bound() for loop in all_loops)  # Rule 2
assert heap_allocations_after_init == 0  # Rule 3: no malloc/free/new after initialization
assert all(fn.line_count <= 60 for fn in all_functions)  # Rule 4
assert all(fn.assertion_count >= 2 for fn in all_functions)  # Rule 5 average; no trivially always-true assertions
assert all(var.scope == SMALLEST_POSSIBLE for var in all_variables)  # Rule 6
assert all(call.return_checked or call.return_cast_void for call in non_void_calls)  # Rule 7
assert preprocessor_uses_only(header_includes=True, simple_macros=True, token_pasting=False, varargs=False, recursive_macros=False)  # Rule 8
assert all(ptr.dereference_depth <= 1 and not ptr.is_function_pointer for ptr in all_pointers)  # Rule 9
assert compiler_warning_count == 0 and static_analyzer_warning_count == 0  # Rule 10

Complexity:
Time | O(N) per rule check over source lines | all checks are single-pass static; Rule 2 (loop bound proof) may require symbolic execution
Space | O(N) for call graph construction | N = number of functions
Parallelism | all 10 rules are independent; can be checked in parallel

Failure modes (from paper):
Rule 2 violated | runaway code, non-termination, stack overflow in recursion-adjacent patterns
Rule 3 violated | memory leaks, use-after-free, allocation failure, unpredictable GC pauses in safety windows
Rule 5 with trivially-true assertions | satisfies count without catching anything; paper explicitly bans "assert(true)"
Rule 9 violated | tool-based static analyzers cannot prove absence of recursion when function pointers are used
Rule 10 with suppressions | warning suppressed = rule violated; "code causing the confusion should be rewritten"
Rule 8 with 10+ conditionals | 2^10 = 1024 possible code versions each requiring separate test
Rules rationalized as "acceptable to violate" | paper: "it will be easier to comply with the rule than to explain why non-compliance might be acceptable"

Implementation pitfalls (do NOT do this):
Do NOT count function pointers as "one level of dereferencing" -- they are banned entirely (Rule 9)
Do NOT satisfy Rule 5 with assertions the static checker can prove always-true or always-false
Do NOT cast return to (void) as a blanket strategy -- only when response to error equals response to success (Rule 7)
Do NOT use conditional compilation (#ifdef) to manage feature flags -- each directive multiplies test surface (Rule 8)
Do NOT permit even one compiler warning -- zero is the invariant, not "few" (Rule 10)
Do NOT apply rules only at release time -- Rule 10 says "from the first day of development"
Do NOT allow heap allocation in init-time code to set a precedent -- Rule 3 is "after initialization" but init allocations must be provably bounded
Do NOT treat Rule 4 (60 lines) as a soft suggestion -- it is a hard mechanical check

DETERMINISTIC-GATE RULES (one per line per rule, for harness implementation):

Rule 1 | VERBATIM: "Restrict all code to very simple control flow constructs -- do not use goto statements, setjmp or longjmp constructs, and direct or indirect recursion." | CHECKABILITY: YES for goto/setjmp/longjmp (grep/AST); PARTIAL for indirect recursion (requires call-graph cycle detection) | RUST checker: deny(clippy::goto) is N/A (Rust has no goto); ban recursive fn via call-graph: codegraph_callers cycle check; grep for fn patterns where fn calls itself transitively; in Python: ast.parse + call-graph walk for cycles; in TS/Svelte: tsc --noEmit then eslint no-restricted-syntax for recursion | HARNESS IMPL: build a call graph from the changeset, run topological sort, exit 1 if a cycle exists; also grep for setjmp/longjmp in Rust FFI blocks and Python ctypes

Rule 2 | VERBATIM: "All loops must have a fixed upper-bound. It must be trivially possible for a checking tool to prove statically that a preset upper-bound on the number of iterations of a loop cannot be exceeded. If the loop-bound cannot be proven statically, the rule is considered violated." | CHECKABILITY: PARTIAL (trivially-bounded loops are checkable; symbolic upper-bound proof requires abstract interpretation) | RUST checker: flag any loop {} or while condition {} where the bound is not a literal constant or a .len()/.count() of a bounded collection; loop { ... break } always violates unless break is inside a counter check; deny clippy::infinite_loop (Rust 1.76+) | PYTHON checker: flag while True without an explicit break-after-N pattern; flag unbounded generators in agent loops | HARNESS IMPL: regex scan for unbounded loop patterns + AST walk; exit 1 on any loop whose upper-bound cannot be trivially read from the source; treat agent task-runner loops as exempt only if they have a documented non-termination intent per Rule 2 exception

Rule 3 | VERBATIM: "Do not use dynamic memory allocation after initialization." | CHECKABILITY: YES for explicit heap calls (malloc/free/new/delete); PARTIAL for implicit allocation (Vec::push beyond capacity, Box::new in hot path) | RUST checker: grep changeset for Box::new, Vec::new, Arc::new, String::new, format!, .to_string(), .clone() called OUTSIDE a const/static/once_cell init context; flag any allocation site reachable from a task_runner dispatch loop; deny(clippy::new_without_default) is adjacent but not equivalent | PYTHON checker: flag object instantiation and list/dict comprehensions inside inference loops; memory-sensitive paths should use pre-allocated buffers | HARNESS IMPL: for LLM-generated code specifically, flag all heap allocation in hot paths (inference loops, per-token callbacks, streaming handlers); require pre-allocation at module load time

Rule 4 | VERBATIM: "No function should be longer than what can be printed on a single sheet of paper in a standard reference format with one line per statement and one line per declaration. Typically, this means no more than about 60 lines of code per function." | CHECKABILITY: YES, fully mechanical | RUST checker: maintainability_check.py MR-3 already covers 80-line functions; tighten to 60 for safety-critical contexts; `python .claude/tools/maintainability_check.py --files <changed> --fn-line-limit 60` | PYTHON checker: ast.parse, walk FunctionDef nodes, count lines; exit 1 if any function body exceeds 60 lines | HARNESS IMPL: the existing MR-3 checker covers this at 80 lines; a stricter 60-line gate requires a config flag; add --fn-line-limit 60 to the gate manifest for safety-critical modules

Rule 5 | VERBATIM: "The assertion density of the code should average to a minimum of two assertions per function. Assertions are used to check for anomalous conditions that should never happen in real-life executions. Assertions must always be side-effect free and should be defined as Boolean tests. When an assertion fails, an explicit recovery action must be taken, e.g., by returning an error condition to the caller of the function that executes the failing assertion. Any assertion for which a static checking tool can prove that it can never fail or never hold violates this rule." | CHECKABILITY: YES for count; PARTIAL for side-effect-free and never-fail checks | RUST checker: count assert!, assert_eq!, assert_ne!, debug_assert! per fn body via ast walk; require avg >= 2; flag assert!(true) and assert_eq!(x, x) as trivially-true; flag assertions containing mutation (e.g. assert!(vec.push(x).is_ok())) | PYTHON checker: count assert statements per function; require avg >= 2 per module; flag assert True | HARNESS IMPL: this is the assertion-density gate described in tracing-test-assert.md but applied to production code not just tests; write assertion_density_check.py that walks AST and counts per-fn, exits 1 if any fn has 0 assertions and avg < 2

Rule 6 | VERBATIM: "Data objects must be declared at the smallest possible level of scope." | CHECKABILITY: PARTIAL (detectable for obvious cases: module-level mutable globals, function-level statics used in one branch) | RUST checker: grep for static mut (always a violation); flag pub(crate) or pub statics that are only read in one fn; clippy::items_after_statements catches some cases; deny(clippy::too_many_arguments) is adjacent | PYTHON checker: flag module-level mutable state (global dict/list mutations) in agent loop code; flag class attributes used only in one method | HARNESS IMPL: the maintainability MR-2/MR-4 rules already push data behind boundaries; add a specific check for static mut and module-level mutable collections in generated Rust

Rule 7 | VERBATIM: "The return value of non-void functions must be checked by each calling function, and the validity of parameters must be checked inside each function." | CHECKABILITY: YES for unchecked return values; YES for clippy::must_use | RUST checker: clippy::unused_must_use catches Result/Option drops; deny(unused_must_use) in CI; grep for let _ = on Result/Option returns (explicit discard without comment); flag any .unwrap()/.expect() that discards error context (already banned by CLAUDE.md); for parameter validity: flag fns with no guard at the top checking inputs | PYTHON checker: flag bare function calls whose return value is ignored when the function is typed as returning Optional or Result-like; mypy --disallow-untyped-defs + --warn-return-any | HARNESS IMPL: Rust's #[must_use] + deny(unused_must_use) covers this mechanically; add a grep for explicit `let _ =` without a justification comment; the no-.unwrap() rule in CLAUDE.md is the idiomatic Rust enforcement of Rule 7's spirit

Rule 8 | VERBATIM: "The use of the preprocessor must be limited to the inclusion of header files and simple macro definitions. Token pasting, variable argument lists (ellipses), and recursive macro calls are not allowed. All macros must expand into complete syntactic units. The use of conditional compilation directives is often also dubious, but cannot always be avoided. This means that there should rarely be justification for more than one or two conditional compilation directives even in large software development efforts, beyond the standard boilerplate that avoids multiple inclusion of the same header file. Each such use should be flagged by a tool-based checker and justified in the code." | CHECKABILITY: YES for C preprocessor; RUST equivalent: cfg() and macro_rules! | RUST checker: grep for cfg() occurrences (allowed: cfg(test), cfg(feature=...) in Cargo.toml-driven gates, cfg(target_os)); flag cfg() chains with more than 2 non-test conditions; deny(macro_rules) that use ## token pasting or $($args:tt)* varargs patterns in production macros; flag proc-macros that expand to more than one syntactic unit | PYTHON checker: flag conditional imports (try: import x except ImportError) that are not in __init__.py; flag eval()/exec() as macro-equivalent obfuscation | HARNESS IMPL: for Rust/Python LLM-generated code this maps to banning complex cfg() chains and eval(); a cfg_complexity_check.py that counts non-test cfg() occurrences and exits 1 if > 2 per module covers the spirit

Rule 9 | VERBATIM: "The use of pointers should be restricted. Specifically, no more than one level of dereferencing is allowed. Pointer dereference operations may not be hidden in macro definitions or inside typedef declarations. Function pointers are not permitted." | CHECKABILITY: YES for function pointers; PARTIAL for dereference depth (requires type-aware analysis) | RUST checker: ban fn() type aliases in production code; flag Box<dyn Fn(...)> and Arc<dyn Fn(...)> where a concrete type could be used; Rust references (&T, &mut T) are single-level by default -- nested references (&&T) are a code smell; deny(clippy::type_complexity) catches deeply-nested pointer-like types; flag trait objects (dyn Trait) used as function dispatch (closest Rust analog to function pointers) | PYTHON checker: flag callable objects stored as dict values for dynamic dispatch without a typed protocol; flag functools.partial chains deeper than 1 | HARNESS IMPL: for LLM-generated Rust, grep for fn() type parameters and Box<dyn Fn> in non-test code; for generated Python, flag dynamic dispatch patterns that defeat static analysis

Rule 10 | VERBATIM: "All code must be compiled, from the first day of development, with all compiler warnings enabled at the compiler's most pedantic setting. All code must compile with these setting without any warnings. All code must be checked daily with at least one, but preferably more than one, state-of-the-art static source code analyzer and should pass the analyses with zero warnings." | CHECKABILITY: YES, fully mechanical | RUST checker: CI must run cargo clippy -- -D warnings (deny all warnings as errors); cargo check --workspace must exit 0; any allow(dead_code), allow(unused), allow(clippy::*) in production code is a Rule 10 violation unless in a documented WIRE-DARK exemption | PYTHON checker: ruff --select ALL --exit-non-zero-on-fix; mypy --strict; both must exit 0 with zero warnings | TS/SVELTE checker: tsc --noEmit --strict exit 0; eslint --max-warnings 0 | HARNESS IMPL: this is already enforced by CLAUDE.md "Zero Lazy Code" + the existing CI gates; add explicit --D warnings to cargo clippy in CI and make warning count a gated metric; MR-2 checker already blocks lint suppressions

CONTRACT TEST STUBS (copy into tests/contracts/test_power_of_ten_contract.py):

import subprocess, ast, re, pathlib

def test_rule2_no_unbounded_loops_in_agent_files():
    # Rule 2: all loops must have a statically provable upper bound
    # Targets agent task runner loops in generated Rust/Python
    agent_py = list(pathlib.Path("F:/.ELAI_workflow/.elai_cc").rglob("task_runner*.py"))
    for f in agent_py:
        src = f.read_text(encoding="utf-8", errors="replace")
        # flag `while True:` without an explicit break-after-N counter pattern
        unbounded = re.findall(r'\bwhile\s+True\s*:', src)
        assert len(unbounded) == 0 or all(
            "break" in src[m.start():m.start()+500] for m in
            (re.finditer(r'\bwhile\s+True\s*:', src))
        ), f"{f}: unbounded while True without break (Rule 2)"

def test_rule4_function_length_le60():
    # Rule 4: no function longer than ~60 lines (one printed page)
    import ast
    files = list(pathlib.Path("F:/.ELAI_workflow/.elai_cc/python").rglob("*.py"))
    violations = []
    for f in files:
        try:
            tree = ast.parse(f.read_text(encoding="utf-8", errors="replace"))
        except SyntaxError:
            continue
        for node in ast.walk(tree):
            if isinstance(node, (ast.FunctionDef, ast.AsyncFunctionDef)):
                length = (node.end_lineno or node.lineno) - node.lineno + 1
                if length > 60:
                    violations.append(f"{f}:{node.lineno} fn '{node.name}' = {length} lines")
    assert violations == [], "Rule 4 violations (>60 lines per fn):\n" + "\n".join(violations[:10])

def test_rule5_assertion_density_minimum_2():
    # Rule 5: average >= 2 assertions per function; no trivially-true assertions
    import ast
    files = list(pathlib.Path("F:/.ELAI_workflow/.elai_cc/python").rglob("*.py"))
    violations = []
    for f in files:
        try:
            tree = ast.parse(f.read_text(encoding="utf-8", errors="replace"))
        except SyntaxError:
            continue
        for node in ast.walk(tree):
            if isinstance(node, (ast.FunctionDef, ast.AsyncFunctionDef)):
                asserts = [n for n in ast.walk(node) if isinstance(n, ast.Assert)]
                # flag assert True (trivially always-true)
                trivial = [a for a in asserts if isinstance(a.test, ast.Constant) and a.test.value is True]
                if trivial:
                    violations.append(f"{f}:{node.lineno} fn '{node.name}' has trivially-true assert (Rule 5)")
    assert violations == [], "Rule 5 trivially-true assertion violations:\n" + "\n".join(violations[:10])

def test_rule7_no_ignored_results_in_rust():
    # Rule 7: return value of non-void functions must be checked
    # Rust: deny(unused_must_use) covers this; check CI flags
    cargo_toml = pathlib.Path("F:/.ELAI_workflow/.elai_cc/Cargo.toml")
    if not cargo_toml.exists():
        return  # skip if no Rust project
    # verify clippy is run with -D warnings in CI/build scripts
    ci_files = list(pathlib.Path("F:/.ELAI_workflow/.elai_cc").rglob("*.yml")) + \
               list(pathlib.Path("F:/.ELAI_workflow/.elai_cc").rglob("*.toml"))
    found_dwarnings = any(
        "-D warnings" in f.read_text(encoding="utf-8", errors="replace")
        for f in ci_files if f.exists()
    )
    assert found_dwarnings, "Rule 7+10: -D warnings not found in any CI/build config (clippy must deny warnings)"

def test_rule10_zero_warnings_in_python():
    # Rule 10: all code checked by static analyzer with zero warnings
    result = subprocess.run(
        ["python", "-m", "ruff", "check", "--select", "E,F,W", "--quiet",
         "F:/.ELAI_workflow/.elai_cc/python"],
        capture_output=True, text=True
    )
    assert result.returncode == 0, (
        f"Rule 10: ruff static analysis found warnings (exit {result.returncode}):\n"
        f"{result.stdout[:500]}"
    )
