Algorithm: MuCoCo (Mutation-based Consistency testing of Code LLMs)
ArXiv: 2604.19086
Paper: MuCoCo: Automated Consistency Testing of Code LLMs
Source: F:\.ELAI_workflow\.claude\papers\2604.19086.html (5510 lines, ingested into RAG topic paper-mucoco)
Reliability: HIGH (peer-reviewed paper indexed in local RAG, primary source read directly)

Grounding notes (READ BEFORE IMPLEMENTING):
- The paper implements ALL mutations using the Python stdlib `ast` module (Appendix G, sec A7.p1.1). No tree-sitter, no LibCST, no multi-language grammars.
- All eleven operators are scoped to Python source. Literal Format explicitly says "Python strings" (sec A2.SS0.SSS1.p1). Appendix D refers to "Python programming syntax" when discussing for-to-while / for-to-enumerate robustness.
- The C-36 input prompt asked for a Rust / TypeScript / Java / Go applicability matrix. That matrix is NOT grounded in the paper. MuCoCo as published is Python-only. Any cross-language extension is a research extension the executor would have to design and ground separately, NOT something to cite from this paper.

Operator catalog (11 operators, exact paper names from Appendix B / Table 9):

Lexical mutations (B.0.1) - rename identifiers and string-literal quoting, leave logic identical
  1. Random Mutation         - replaces function/variable names with random strings
  2. Sequential Mutation     - replaces function name with "generic_function1" and vars with "var1", "var2", ...
  3. Literal Format          - swaps Python string quote style (' <-> ")

Syntactic mutations (B.0.2) - equivalent program syntax, same control flow semantics
  4. for-to-enumerate        - rewrites `for x in seq` using `enumerate` iterator (only when applicable)
  5. for-to-while Mutation   - rewrites every `for` loop as an equivalent `while` loop

Logical mutations (B.0.3) - preserve program behaviour while restructuring boolean / arithmetic expressions
  6. DeMorgan Mutation       - `A and B` -> `not (not A or not B)`; `A or B` -> `not (not A and not B)` (Eq 3, Eq 4)
  7. Boolean Literal         - rewrites `True` / `False` using negation (e.g. `True` -> `not False`)
  8. Commutative Reorder     - reverses operand order in commutative expressions (a+b -> b+a, a*b -> b*a)
  9. Constant Unfolding      - decomposes an integer constant into two factors using add OR multiply
 10. Constant Unfolding (Add)- substitutes every valid integer constant with an equivalent addition expression
 11. Constant Unfolding (Mul)- substitutes every valid integer constant with an equivalent multiplication expression

Per-operator apply contract (paper does not formally define a Python signature; this is the contract the executor MUST implement):

  def apply(source: str, language: str = "python") -> MutationResult:
      """
      Returns MutationResult with one of two states:
        applied  = True,  mutated_source = <rewritten str>, skip_reason = None
        applied  = False, mutated_source = source,           skip_reason = "<why>"
      The paper (sec S3.p3.1) explicitly checks applicability before mutating:
        e.g. for-to-while requires a `for` loop in the program; if none exists, skip.
      Operators MUST be semantic-preserving on the canonical test suite (sec S3.p3).
      """

Inputs to apply():
  source         | str    | Python source code  | mandatory | must be parseable by `ast.parse`
  language       | str    | language identifier | optional  | paper supports only "python"

Outputs from apply():
  applied        | bool   | did the mutation actually run
  mutated_source | str    | rewritten Python source, parseable, semantically equivalent on canonical tests
  skip_reason    | str?   | populated when applied is False (e.g. "no for-loop", "no constants", "no boolean ops")

Language applicability matrix (grounded - NOT extrapolated):
                       Python | Rust | TypeScript | Java | Go
  Random Mutation         Y      ?       ?           ?     ?
  Sequential Mutation     Y      ?       ?           ?     ?
  Literal Format          Y      N       N           N     N    (Python-only; Rust/Java/Go don't have ' vs " interchange for strings)
  for-to-enumerate        Y      N       N           N     N    (idiom is Python-specific)
  for-to-while            Y      ?       ?           ?     ?    (concept generalises, but paper only tests Python)
  DeMorgan                Y      ?       ?           ?     ?
  Boolean Literal         Y      ?       ?           ?     ?    (`True`/`False` literal names are Python-specific; in Rust use true/false, in Java/Go same)
  Commutative Reorder     Y      ?       ?           ?     ?    (concept generalises subject to operator-overloading caveats)
  Constant Unfolding      Y      ?       ?           ?     ?
  Constant Unfolding(Add) Y      ?       ?           ?     ?
  Constant Unfolding(Mul) Y      ?       ?           ?     ?
  Legend: Y = paper demonstrates and evaluates on this language; N = paper-internal evidence the mutation is not directly portable; ? = NOT addressed in the paper. The executor must mark all "?" cells as ungrounded and either (a) design and ground the extension separately or (b) ship Python-only first.

Required runtime dependencies (from Appendix G + corroborated by op descriptions):
  python                          >= 3.10 recommended (ast.unparse used in modern AST workflows; the paper just says "Python 3")
  python-stdlib                    ast      - core parser + node visitor + (un)parser
  python-stdlib                    tokenize - useful for Literal Format (quote-aware rewrite)
  python-stdlib                    re       - convenience for some name rewrites
  No third-party libraries cited in Appendix G for the mutation engine itself.
  Model-side deps cited (NOT mutation deps): huggingface transformers, OpenAI / DeepSeek / Mistral API clients.

Hyperparameters (paper-stated defaults; no sensitivity sweep cited):
  Sequential.function_prefix      | "generic_function" | naming convention for renamed functions
  Sequential.variable_prefix      | "var"              | naming convention for renamed variables
  Random.seed                     | not stated         | paper sets Turbulence seed to 1234 (sec S4.SS5.p1) but does not state Random Mutation seed
  Constant_Unfolding.factor_pick  | unspecified        | "two factors" of the constant; paper does not pin the policy
  LLM.temperature                 | 0                  | set to zero for all models to mitigate randomness (sec S9.p1.1)

Evaluation methodology:
  Datasets (sec 4 + Appendix C):
    HumanEval                     - code generation, output prediction, input prediction
    CRUXEval                      - input prediction, output prediction
    CodeMMLU                      - multiple-choice questions
    BigCodeBench                  - diverse Python code generation tasks
  Tasks:
    MCQ (Multiple Choice Questions)
    Input Prediction (True/False label)
    Output Prediction (literal output)
    Code Generation (complete a function body)
  Test oracles (sec 3.2):
    Correctness Oracle  - run the LLM output against the benchmark's test suite, record pass/fail per task
    Consistency Oracle  - pair original task vs mutated task; flag a consistency error iff their correctness outcomes differ
  Semantic-equivalence check:
    Before each mutated task is fed to the LLM, MuCoCo re-runs the benchmark's test suite on the canonical solution AFTER mutation to confirm the mutation preserves semantics (sec S3.p3.1). Mutated tasks that fail the canonical test suite are dropped (NOT consistency errors).

Primary metric (Eq. 1, sec S4.SS4):
  Inc. = (N_inconsistent / N_test_cases) * 100
  where N_inconsistent  = pairs (original, mutated) whose correctness outcomes disagree
        N_test_cases    = total pairs MuCoCo generated (after applicability + semantic preservation filtering)

Critical invariants (testable assertions for tests/contracts/test_mucoco_contract.py):
  assert ast.parse(mutated_source) is not None                                # all outputs are valid Python
  assert run_test_suite(canonical_solution_with_mutation_applied) == "PASS"   # semantic preservation
  for op in OPERATORS: assert op.apply("def f(): pass", "python").applied == False  # at least one skip-path exercised
  assert ConstantUnfolding.apply("x = 5", "python").mutated_source != "x = 5"
  assert DeMorgan.apply("a and b", "python").mutated_source.count("not") >= 2

Reported headline numbers (sec 5):
  Average consistency error rate across all settings: 14.82% (about 1 in 7 test cases)
  Total errors discovered: ~22K out of ~148K test cases
  Lexical mutations error rate: 16.28% (highest)
  Logical mutations error rate: 13.91%
  vs Turbulence baseline: MuCoCo is up to 5X more effective, 22.46% more errors

Complexity:
  Per task per operator: O(N) AST walk, where N = number of AST nodes; mutation rewriting is single-pass.
  Test suite re-evaluation cost dominates total runtime, not the AST mutation itself.
  Parallelism: per-task per-operator pairs are embarrassingly parallel; paper used Google Colab A100 80GB for the LLM inference side, not for mutations.

Failure modes cited in the paper:
  Construct validity (sec 9): LLM output processing relies on system prompts ensuring expected format; non-determinism mitigated by temperature=0 and frozen open-weight models.
  Internal validity (sec 9): correctness of the mutation operators themselves; mitigated by manual code review and probing studies.
  Mutation applicability: an operator may not apply to a given task (e.g. for-to-while when there's no for-loop). Per sec S3.p3.1, this MUST yield a skip, NOT a synthetic transformation.

Implementation pitfalls (do NOT do this):
  - Do NOT use string-level regex rewrites for Logical mutations; the paper uses ast walks. Regex-driven DeMorgan or Commutative Reorder corrupts operator-overloaded code and breaks semantic preservation.
  - Do NOT skip the "test suite re-evaluation after mutation" step. Without it you cannot distinguish a real consistency error from a broken mutation.
  - Do NOT generalize to Rust / Java / Go / TypeScript without separate grounding. The paper does not validate cross-language behaviour; Literal Format and for-to-enumerate are explicitly Python-specific idioms.
  - Do NOT use any non-Python AST library (tree-sitter, libcst, etc.) and call it "MuCoCo". The paper uses stdlib `ast`; deviation needs its own justification.
  - Do NOT pick mutation parameters at random per call. Paper holds the model temperature to zero precisely to make consistency results reproducible; the mutation engine should be equally deterministic (fixed seed for Random, fixed policy for Constant Unfolding factor picks).
  - Do NOT mark Random / Sequential as separate "ports" without re-running the consistency-preservation check; renaming a builtin or a method on a closed-source object can change semantics silently.

CONTRACT TEST STUBS (copy into tests/contracts/test_mucoco_contract.py):

  import ast
  from typing import Protocol

  class MutationResult(Protocol):
      applied: bool
      mutated_source: str
      skip_reason: str | None

  OPERATORS_LEXICAL    = ["random", "sequential", "literal_format"]
  OPERATORS_SYNTACTIC  = ["for_to_enumerate", "for_to_while"]
  OPERATORS_LOGICAL    = ["demorgan", "boolean_literal", "commutative_reorder",
                          "constant_unfolding", "constant_unfolding_add", "constant_unfolding_mul"]
  ALL_OPERATORS        = OPERATORS_LEXICAL + OPERATORS_SYNTACTIC + OPERATORS_LOGICAL

  def test_eleven_operators_registered():
      assert len(ALL_OPERATORS) == 11

  def test_each_operator_returns_valid_python(apply_fn):
      sample = "def f(a, b):\n    if a and b:\n        for i in range(5):\n            print('hi')\n    return a + b\n"
      for op in ALL_OPERATORS:
          r = apply_fn(op, sample, "python")
          if r.applied:
              ast.parse(r.mutated_source)   # must parse

  def test_skip_when_inapplicable(apply_fn):
      no_for = "def f(a, b): return a + b\n"
      r = apply_fn("for_to_while", no_for, "python")
      assert r.applied is False
      assert r.skip_reason is not None

  def test_non_python_language_is_rejected(apply_fn):
      r = apply_fn("literal_format", "fn main() { println!(\"hi\"); }", "rust")
      assert r.applied is False
      assert r.skip_reason and "python" in r.skip_reason.lower()

  def test_demorgan_introduces_negations(apply_fn):
      r = apply_fn("demorgan", "x = a and b\n", "python")
      if r.applied:
          assert r.mutated_source.count("not") >= 2

  def test_semantic_preservation_via_canonical_tests(apply_fn, run_canonical_tests):
      sample = "def add(a, b):\n    return a + b\n"
      for op in ALL_OPERATORS:
          r = apply_fn(op, sample, "python")
          if r.applied:
              assert run_canonical_tests(r.mutated_source) == "PASS"
