Algorithm: CBMC -- Bounded Model Checking for ANSI-C Programs (and Rust via Kani)
Paper: "A Tool for Checking ANSI-C Programs"
Authors: Edmund Clarke, Daniel Kroening, Flavio Lerda (Carnegie Mellon University)
Venue: TACAS 2004
PDF: https://www.kroening.com/papers/tacas2004.pdf
Local: F:\.ELAI_workflow\.claude\papers\cbmc.pdf
Extracted: F:\.ELAI_workflow\.claude\papers\cbmc_extracted\cbmc.md
Rust analog: Kani (model-checking/kani), which translates Rust MIR to Goto-C and uses CBMC as its verification engine.


ALGORITHM INPUTS

program source | ANSI-C text (or Rust source via Kani) | any size | mandatory
unwind bound n | positive integer per loop | >= 1 | mandatory (tool accepts one global bound or per-loop)
user assertions | assert() calls embedded in source | boolean expressions over program variables | optional (but the primary contract mechanism)
assumptions | assume() calls in source (kani::assume in Rust) | boolean expressions | optional; constrain nondeterministic inputs
nondeterministic inputs | nondet_*() or kani::any::<T>() calls | any scalar type | optional; model symbolic inputs


ALGORITHM OUTPUTS

SAT verdict | SATISFIABLE or UNSATISFIABLE | boolean
counterexample trace | sequence of (line, variable, value) tuples | one entry per executed statement in the error trace | only when SATISFIABLE (i.e., property violated)
unwinding assertion result | PASS or FAIL per loop | indicates whether bound was sufficient | always present
verification result | VERIFIED (all properties hold up to bound) or FAILED (counterexample found) or UNWINDING ASSERTION FAILURE (bound too small)

Invariants the output must satisfy:
- VERIFIED result is only sound if ALL unwinding assertions PASS. A VERIFIED result with any unwinding assertion FAIL is unsound (necessary-not-sufficient).
- A FAILED result with a counterexample is always sound: if CBMC says there is a counterexample, there is one.
- Unwinding assertion FAIL means the bound n must be increased for that loop before the verification result can be trusted.


HYPERPARAMETERS

unwind bound n | no paper default (user must supply) | >= 1, no theoretical upper limit | HIGH sensitivity: too small = unsound VERIFIED; too large = exponential blowup in formula size
bit width | paper default: native C types (32/64 bit) | adjustable via command line | affects overflow detection; changing width changes what overflows
float mode | fixed-point approximation | paper uses fixed-point for float/double | MEDIUM sensitivity: fixed-point is an approximation of IEEE 754


FIVE-STEP BMC ALGORITHM (Section 2.1)

Step 1: Preprocessing and normalization. Side effects replaced by auxiliary variable assignments. break/continue replaced by goto. for/do-while normalized to while.

Step 2: Loop unwinding. Each while loop body duplicated n times. Each copy guarded by the loop condition using an if statement. After the last copy, an UNWINDING ASSERTION is added: assert(NOT loop_condition). This asserts the program never needs more than n iterations. Per paper (Section 2.1, step 2): "After the last copy, an assertion is added that assures that the program never requires more iterations. The assertion uses the negated loop condition. We call this assertion an unwinding assertion."

Step 3: Backward goto unwinding. Backward goto statements unwound similarly to while loops.

Step 4: Function inlining. Recursive calls unwound to a bound. Asserted that recursion never goes deeper. Return replaced by assignment + forward goto.

Step 5: SSA transformation. Program converted to Static Single Assignment form (requires pointer analysis). Each assignment to variable x becomes a new SSA variable x_i. Conditionals generate phi-nodes (ternary expressions in the formula). Produces two bit-vector equations: C (constraints from the program) and P (the property). Check SAT(C AND NOT P). If satisfiable -> property violated, extract counterexample. If unsatisfiable -> property holds up to bound n.


PROPERTIES CHECKED AUTOMATICALLY (Appendix B, Tables 1 and 2)

These are injected automatically by CBMC without requiring user assert() calls:

Array bounds: "Lower and upper bound of arrays, even for arrays with dynamic size" -- checked on every array index operation.

Pointer safety (dereferencing): "When a pointer is dereferenced, CBMC checks that the object pointed to is still alive and of matching type. If the object is an array, the array bounds are checked."

Pointer relational operators: "CBMC checks that the two operands point to the same object."

Pointer type casts: "Upon dereferencing, the type of the object and the expression are checked to match."

Division by zero: Checked for all integer division and modulo operations (Table 1: "Division by zero" under Integer Operators).

Arithmetic overflow: "Overflow for signed data types" -- checked for integer operators and type casts involving signed integers (Table 1).

Side effect ordering: "Side effects are checked not to affect variables that are evaluated elsewhere, and thus, that the ordering of evaluation does not affect the result."

Function return value: "Functions with a non-void return type must return a value by means of the return statement."

Recursive unwinding bound: Asserted that recursion never exceeds the specified recursion bound.

Dynamic memory (malloc/free): "Upon dereferencing, the object pointed to must still be alive. The pointer passed to free is checked to point to an object that is still alive. CBMC can check that all dynamically allocated memory is deallocated before exiting the program ('memory leaks')."

Union type discipline: "CBMC checks that unions are not used for type conversion, i.e., that the member used for reading is the same as used for writing last time."

User assertions: "Assertions are verified to be true for all possible non-deterministic choices given that any assumption executed prior to the assertion is true."


DETERMINISTIC-GATE RULES (ELAI-specific section)

--- VERBATIM QUOTES FROM PAPER ---

Properties checked automatically (Appendix B, Tables 1 and 2):
  Array bounds: "Lower and upper bound of arrays, even for arrays with dynamic size"
  Pointer safety: "When a pointer is dereferenced, CBMC checks that the object pointed to is still alive and of matching type."
  Division by zero: "Division by zero" (Table 1, Integer Operators row, Properties column)
  Arithmetic overflow: "Overflow for signed data types" (Table 1, Integer Operators and Type casts rows)
  User assertions: "Assertions are verified to be true for all possible non-deterministic choices given that any assumption executed prior to the assertion is true."

Loop unwinding bound and unwinding assertion (Section 2.1, Step 2):
  "The while loops are unwound by duplicating the loop body n times. Each copy is guarded using an if statement that uses the same condition as the loop statement. The if statement is added for the case that the loop requires less than n iterations. After the last copy, an assertion is added that assures that the program never requires more iterations. The assertion uses the negated loop condition. We call this assertion an unwinding assertion."

  "These unwinding assertions are crucial for our approach: they assert that the unwinding bound is actually large enough. If the unwinding assertion of a loop fails for any possible execution, then we increase the bound n for that particular loop until the bound is large enough."

Counterexample trace output (Section 3):
  "If a counterexample is found, the GUI allows stepping through the trace like a debugger." Each step shows: the source line "executed", and "the current values of the program variables" in a Watches window. Stepping is supported both forward and backward. The trace is the SAT assignment extracted from the satisfying assignment of C AND NOT P.

Soundness condition (Section 1):
  "The tool checks that sufficient unwinding is done to ensure that no longer counterexample can exist by means of unwinding assertions."

--- MAPPING TO ELAI RUST COMPONENTS ---

CBMC/Kani is NOT appropriate for verifying the whole ELAI codebase. The right targets are small, critical, pure-or-near-pure Rust modules where:
  (a) the state space is bounded and the unwind depth is provably small
  (b) the property is a hard safety invariant (not a behavioral/semantic contract)
  (c) a counterexample would be a P0 bug (data corruption, index panic, integer overflow silently producing wrong answers)

Priority targets for a Kani gate in ELAI:

1. SSB postcard serializer/deserializer round-trip
   File: crates/orchestrator/src/ssb/*.rs (the action_type, payloads modules)
   Property: forall valid payload p: deserialize(serialize(p)) == p
   Bound: small -- postcard is a fixed-format codec, no loops past field count
   Kani harness: kani::any::<ActionType>() -> assert!(round_trip_ok)

2. FSM state-transition function
   File: crates/orchestrator/src/compiled_pipeline.rs (or wherever transition() lives)
   Property: no transition produces an invalid/undefined state; every (state, event) pair maps to exactly one next state
   Bound: small -- finite state enum, no loops
   Kani harness: kani::any::<FsmState>() + kani::any::<FsmEvent>() -> assert!(next_state is valid variant)

3. Retry / budget counter arithmetic
   File: wherever retry counts and token budget counters are incremented/decremented
   Property: counter never underflows below 0 (signed) or wraps (unsigned) in a way that bypasses the guard
   Overflow check: automatic (CBMC checks arithmetic overflow for signed types by default)
   Kani harness: kani::any::<u32>() with assume(budget <= MAX) -> assert!(decrement does not underflow)

4. Gate predicate logic (wiring-enforcement predicates, autonomous_gate)
   File: crates/orchestrator/src/autonomous_gate.rs
   Property: gate never emits PASS when evidence is absent (the fail-closed invariant)
   Kani harness: kani::any::<GateInput>() with assume(evidence_field is None) -> assert!(result == Block)

5. Parsers with bounded input (WIRING_CONTRACT parser, action_type lock parser)
   File: crates/orchestrator/src/wiring_contract.rs, crates/orchestrator/src/ssb/action_type.rs
   Property: parser never panics on any input within the valid length bound; always returns Ok or Err, never unwinds past the bound
   Bound: set to max expected contract file length (e.g., 4096 bytes)

--- THE GATE DESCRIPTION FOR ELAI CI ---

"The auditor runs `cargo kani --harness <harness_name>` on annotated proof harnesses in the critical modules listed above. An unwinding assertion failure OR a found counterexample = BLOCK. A clean VERIFIED result (all unwinding assertions PASS, no counterexample) = green on that module."

Gate is UNSOUND beyond the unwind bound: a VERIFIED result only means no property violation exists within n loop iterations. This is bounded model checking -- necessary-not-sufficient. Consistent with ELAI's layered-gate doctrine (Layer 2 static, Layer 3 coverage, Layer 4 runtime trace -- Kani is a Layer 2 static gate, not a replacement for Layer 3/4).

Kani version note: as of 2025, Kani is available via `cargo install --locked kani-verifier` and is actively maintained at github.com/model-checking/kani. Anti-slopsquat: verify current version before use; do not assume any specific version number from this document.


CRITICAL INVARIANTS (TESTABLE ASSERTIONS)

assert unwinding_assertion_pass == True, "VERIFIED result is unsound if any unwinding assertion failed"
assert not (verdict == "VERIFIED" and any(ua.status == "FAIL" for ua in unwinding_assertions)), "soundness violated"
assert counterexample is not None if verdict == "FAILED" else True, "FAILED must produce a counterexample trace"
assert all(step.line_number >= 0 and step.variable_name is not None for step in counterexample_trace), "trace steps must have location and variable"
assert unwind_bound >= 1, "bound of 0 is meaningless; minimum 1"
assert not (overflow_check_disabled and signed_integer_present), "disabling overflow check on signed types violates paper's automatic-property guarantee"


COMPLEXITY

Formula size: exponential in unwind bound n (each loop copy doubles formula size in worst case). SAT solving is NP-complete. In practice, CBMC uses Chaff/MiniSAT/Cadical and handles bounded state spaces well.
Time: O(exp(n)) worst case per loop nest depth. Practical for small n (1-20) on small modules. Not practical for n > 100 on non-trivial code.
Space: formula stored as CNF; size proportional to n * (size of loop body in SSA form). Dynamic memory handled via uninterpreted functions (Section 2.2).
Parallelism: each proof harness is independent; harnesses can run in parallel CI jobs.


FAILURE MODES (FROM PAPER AND KANI DOCS)

Bound too small: unwinding assertion fails; VERIFIED result cannot be trusted. Must increase n. Paper: "If the unwinding assertion of a loop fails for any possible execution, then we increase the bound n for that particular loop until the bound is large enough."
Bound too large: formula becomes intractably large; SAT solver times out or runs out of memory. No guidance in paper on upper limits -- empirical tuning required.
Unbounded loops: if n is not set and the loop has no trivially computable bound, Kani may not terminate. Must always set --default-unwind or #[kani::unwind(N)].
Float/double approximation: paper uses fixed-point arithmetic for float types. This is an approximation; NaN, infinity, and rounding-mode-dependent behavior may not be faithfully modeled.
Dynamic memory with nondeterministic size: "dynamically allocated arrays are not translated into CNF by allocating literals for each potential array element. Instead, arrays with variable size are implemented by means of uninterpreted functions." This means some dynamic memory patterns are underapproximated.
Pointer analysis soundness: SSA transformation "requires a pointer analysis" (Section 2.1, Step 5). An unsound pointer analysis can cause missed aliasing and false VERIFIED results.
Self-review trap: an agent that writes the proof harness AND the implementation AND declares VERIFIED has reviewed itself from inside the boundary. Same failure mode as in wiring-enforcement.md. The harness must be authored separately from the implementation under verification.


IMPLEMENTATION PITFALLS (DO NOT DO THIS)

Do NOT declare a module VERIFIED and ignore unwinding assertion failures -- that is unsound by definition (paper Section 2.1 Step 2 and Section 1).
Do NOT set unwind bound to 0 -- produces no useful coverage.
Do NOT use Kani/CBMC as a replacement for Layer 3 (runtime coverage) or Layer 4 (trace assertion) gates -- it is a Layer 2 static gate; bounded = incomplete beyond the bound.
Do NOT run Kani on the entire ELAI codebase as one harness -- the formula will be intractable. Target small critical modules only.
Do NOT allow the executing agent to author both the proof harness and the module under test -- self-review failure, identical to the in-boundary artifact failure documented in wiring-enforcement.md.
Do NOT disable overflow checking (--no-signed-overflow-check or equivalent) for signed integer modules -- this removes one of the automatically-checked properties from the paper.
Do NOT use #[kani::unwind(1)] on a loop known to execute more than once -- this produces an immediate unwinding assertion failure and the VERIFIED/FAILED verdict is about a 1-iteration prefix only.
Do NOT confuse "counterexample not found" with "property holds" when unwinding assertions have failed.
Do NOT use float/double types in proof harnesses expecting exact arithmetic -- fixed-point approximation may miss real overflow/underflow.
