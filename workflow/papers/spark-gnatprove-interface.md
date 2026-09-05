Source: AdaCore SPARK 2014 + GNATprove toolset
Reliability: HIGH (official SDK documentation, AdaCore-authored)
Files examined:
  https://docs.adacore.com/spark2014-docs/html/ug/en/introduction.html
  https://www.adacore.com/about-spark
  https://docs.adacore.com/spark2014-docs/html/ug/en/usage_scenarios.html
  https://docs.adacore.com/spark2014-docs/html/ug/en/source/subprogram_contracts.html
  https://docs.adacore.com/spark2014-docs/html/ug/en/source/type_contracts.html
  https://docs.adacore.com/hilite-docs/html/ug/usage.html
  https://learn.adacore.com/courses/intro-to-spark/chapters/03_Proof_Of_Program_Integrity.html
  https://www.adacore.com/blog/spark-2014-rationale-support-for-type-invariants
  https://github.com/AdaCore/spark2014/blob/master/docs/ug/en/usage_scenarios.rst
Fetch date: 2026-06-21
Note: Several sub-pages of docs.adacore.com returned HTTP 403; content sourced from mirrored learn.adacore.com and older hilite-docs mirror where needed.

=== WHAT SPARK IS ===

SPARK is "a programming language and a set of verification tools designed to meet the needs of high-assurance software development." It is "based on Ada, both subsetting the language to remove features that defy verification, but also extending the system of contracts and aspects to support modular, formal verification."
Source: docs.adacore.com/spark2014-docs/html/ug/en/introduction.html

"SPARK contracts - preconditions, postconditions, type invariants, assertions, etc. - are part of the language, not structured comments. The contracts are checked by the compiler, proved by SPARK, and have execution semantics."
Source: adacore.com/about-spark

SPARK uses "deductive formal verification: subprograms are represented as Hoare triples, and SPARK proves that when the subprogram preconditions are met, subprogram execution ensures the postcondition." The proofs are "sound - no false negatives, and precise - no false positives, provided timeout is not reached."
Source: adacore.com/about-spark

=== FIVE ASSURANCE LEVELS (Section 8.1.1 of SPARK User's Guide) ===

Stone level   - "Valid SPARK" - code respects the SPARK language subset legality rules.
Bronze level  - "Initialization and Correct Data Flow" - no uninitialized reads, proper global variable access. "The SPARK code is guaranteed to be free from a number of defects: no reads of uninitialized variables, no possible interference between parameters and global variables, no unintended access to global variables, no infinite loop or recursion in functions."
Silver level  - "Absence of Run-time Errors (AoRTE)" - proved absence of runtime exceptions. "typically, 95% to 98% of run-time checks can be proved automatically."
Gold level    - "Proof of Key Integrity Properties" - critical data invariants and state transitions verified.
Platinum level - "Full Functional Correctness" - "when contracts fully cover the functional requirements." "GNATprove must verify complete user specifications for type invariants, preconditions, postconditions, type predicates, loop variants, and loop termination."
Source: docs.adacore.com/spark2014-docs/html/ug/en/usage_scenarios.html + github.com/AdaCore/spark2014/blob/master/docs/ug/en/usage_scenarios.rst

=== DETERMINISTIC-GATE RULES ===

--- (a) CONTRACT CONSTRUCTS ---

Pre aspect (Preconditions, Section 5.2.1):
  "specifies constraints on callers of the subprogram"
  "The precondition of a subprogram is checked at run time every time the subprogram is called"
  When analyzed by GNATprove: "when verifying the body of a subprogram, GNATprove assumes it knows nothing about the possible initial values of its parameters and of the global variables it accesses except what you state in the subprogram's precondition."
  At call sites: "GNATprove will check that the precondition holds at the point of the call"
  Source: docs.adacore.com/spark2014-docs/html/ug/en/source/subprogram_contracts.html + learn.adacore.com

Post aspect (Postconditions, Section 5.2.2):
  "specifies partly or completely the functional behavior of the subprogram"
  "the postcondition of a subprogram is checked at run time every time the subprogram returns"
  "GNATprove considers all calling contexts in which the precondition of the subprogram holds"
  "checks that the postcondition of a subprogram cannot fail"
  "when a subprogram is called, GNATprove assumes its out and in out parameters and the global variables it writes can be modified in any way compatible with their postconditions."
  Contract verification timing: "Preconditions...are verified every time a subprogram is called. Postconditions...are verified once as part of the verification of the subprogram's body."
  Source: docs.adacore.com/spark2014-docs/html/ug/en/source/subprogram_contracts.html + docs.adacore.com/hilite-docs/html/ug/usage.html

Contract_Cases aspect (Section 5.2.3):
  "partitions subprogram behavior into distinct scenarios"
  "When the guard evaluates to True on subprogram entry, the corresponding consequence should also evaluate to True on subprogram exit."
  "Exactly one contract case should be enabled for each call, or said equivalently, the contract cases should be disjoint and complete."
  Source: docs.adacore.com/spark2014-docs/html/ug/en/source/subprogram_contracts.html

Exceptional_Cases aspect (Section 5.2.7):
  "lists all the exceptions which might be propagated by a procedure, and associates them to an exceptional postcondition"
  Source: docs.adacore.com/spark2014-docs/html/ug/en/source/subprogram_contracts.html

Type_Invariant aspect (Section 5.4.4):
  "can only be specified on completions of private types (and not directly on private type declarations)"
  "express a property that is only guaranteed outside of the immediate scope of the type bearing the invariant"
  GNATprove "checks that, outside of the immediate scope of a type with an invariant, all values of this type are allowed by its invariant"
  Checks occur: at type declaration for default initialization, on global objects after initialization, when calling/returning from subprograms visible outside the type's scope
  "If the subprogram is declared outside of [the package] or if it is declared in the public part, then the invariant must hold for its input in entry of the subprogram and for its outputs in exit."
  Semantically equivalent to: "Pre => Invariant(T), Post => Invariant(T)" on every external boundary
  Invariant may be broken inside the type's body "as long as it is restored at the end of the subprogram"
  Source: docs.adacore.com/spark2014-docs/html/ug/en/source/type_contracts.html + adacore.com/blog/spark-2014-rationale-support-for-type-invariants

Static_Predicate aspect (Section 5.4.3.1):
  "allows specifying which values are allowed or forbidden in a scalar subtype, when this specification cannot be expressed with Scalar Ranges (because it has holes)"
  Can be used in membership tests, for-loop ranges, case-statement choices
  Source: docs.adacore.com/spark2014-docs/html/ug/en/source/type_contracts.html

Dynamic_Predicate aspect (Section 5.4.3.2):
  "allows specifying properties of scalar subtypes that cannot be expressed as static predicates"
  "allows specifying relations between components of a record"
  Checked at conversions, parameter returns, object declarations
  "GNATprove checks that all values assigned to a subtype with a predicate are allowed by its predicate (for all forms: Predicate, Static_Predicate, Dynamic_Predicate and Ghost_Predicate)"
  Source: docs.adacore.com/spark2014-docs/html/ug/en/source/type_contracts.html

--- (b) ABSENCE OF RUN-TIME ERRORS (AoRTE) - SILVER LEVEL ---

"SPARK proves that no run-time errors are possible in your code"
Source: adacore.com/about-spark

At Silver level, GNATprove proves the complete absence of:
  - "all possible explicit raising of unexpected exceptions in the program"
  - "raising exception Constraint_Error at run time"
  - "all possible failures of assertions corresponding to raising exception Assert_Error at run time"
Source: github.com/AdaCore/spark2014/blob/master/docs/ug/en/usage_scenarios.rst

Specific runtime check categories proved absent (from docs.adacore.com/hilite-docs/html/ug/usage.html):
  Division Check    : "Check that the second operand of the division, mod or rem operation is different from zero."
  Index Check       : "Check that the given index is within the bounds of the array."
  Overflow Check    : "Check that the result of the given arithmetic operation is within the bounds of the base type."
  Range Check       : "Check that the given value is within the bounds of the expected scalar subtype."
  Length Check      : "Check that the given array is of the length of the expected array subtype."
  Discriminant Check: "Check that the discriminant of the given discriminant record has the expected value."
  (Note: Storage_Error is NOT covered by SPARK analysis - explicitly excluded in the user guide)
Source: docs.adacore.com/hilite-docs/html/ug/usage.html

"GNATprove generates a logical formula called a verification condition for each check that would otherwise be required by the Ada (and hence SPARK) language."
Source: learn.adacore.com/courses/intro-to-spark/chapters/03_Proof_Of_Program_Integrity.html

Memory safety additionally proven:
  "absence of memory issues such as use after free, access to uninitialized memory or memory leaks and corruption" via dynamic memory mitigation, borrow-checking analysis, and formal proof
Source: adacore.com/about-spark

--- (c) MODULAR PER-SUBPROGRAM PROOF MODEL ---

"To scale to large programs, GNATprove performs proofs on a per-subprogram basis by relying on preconditions and postconditions to properly summarize the input and output state."
Source: learn.adacore.com/courses/intro-to-spark/chapters/03_Proof_Of_Program_Integrity.html

"GNATprove performs very powerful local analysis of the program, but it does not cross subprogram boundaries. Instead, it uses the subprogram contracts provided by users to interpret the effect of calls."
Source: docs.adacore.com/hilite-docs/html/ug/usage.html

GNATprove formally verifies each SPARK subprogram will either:
  - "return normally in a state that respects the subprogram's postcondition"
  - "raise an exception in a state that respects the subprogram's exceptional contract"
  - "terminate abnormally as a result of a primary stack, secondary stack, or heap memory allocation failure"
  - "not terminate at all when it is allowed by its termination contract"
Source: docs.adacore.com/spark2014-docs/html/ug/en/source/subprogram_contracts.html

Composability across proof and test: "It is possible to combine freely subprograms that are proved and subprograms that are unit tested, provided subprogram contracts...are exercised during unit testing."
Source: github.com/AdaCore/spark2014/blob/master/docs/ug/en/usage_scenarios.rst

GNATprove --level switch (proof aggressiveness):
  Level 0: --prover=cvc5 --timeout=1 --memlimit=1000 --steps=0 --counterexamples=off
  Level 1: --prover=cvc5,z3,altergo --timeout=1 --memlimit=1000 --steps=0 --counterexamples=off
  Level 2: --prover=cvc5,z3,altergo --timeout=5 --memlimit=1000 --steps=0 --counterexamples=on
  Level 3: --prover=cvc5,z3,altergo --timeout=20 --memlimit=2000 --steps=0 --counterexamples=on
  Level 4: --prover=cvc5,z3,altergo --timeout=60 --memlimit=2000 --steps=0 --counterexamples=on
Source: docs.adacore.com/spark2014-docs/html/ug/en/source/how_to_run_gnatprove.html

--- ELAI MAPPING: CONTRACT-FIRST RUST ADAPTER RULE ---

Pattern import from SPARK to Rust (ELAI-specific):

RULE (one-line binding): Every privileged ELAI Rust adapter -- the side-effecting Supervisor-plane fns covering file ops, egress, SSB write, and credential handling -- must carry contract-style pre/postconditions (as debug_assert!/assert! guards at fn entry and exit, or as Kani proof harnesses) and an AoRTE proof obligation (via Kani bounded model checking) before the fn is permitted to be a side-effect sink in production wiring.

Mapping from SPARK constructs to Rust:
  SPARK Pre aspect           -> Rust: assert!(precondition, "pre: ...") at fn entry; Kani: kani::assume(precondition)
  SPARK Post aspect          -> Rust: debug_assert!(postcondition, "post: ...") before each return; Kani: kani::assert(postcondition, "post: ...")
  SPARK Type_Invariant        -> Rust: impl <Type> { fn check_invariant(&self) -> bool {...} } called in debug_assert! on entry/exit of pub fns
  SPARK AoRTE (Division Check) -> Rust: assert!(divisor != 0) before every division; Kani proves this
  SPARK AoRTE (Index Check)   -> Rust: assert!(idx < slice.len()) before index; Kani proves this; or use .get() with explicit None handling
  SPARK AoRTE (Overflow Check) -> Rust: use checked_add/checked_mul/checked_sub; never `as`-cast narrowing; Kani proves via bit-precise model
  SPARK AoRTE (Range Check)   -> Rust: use TryFrom/TryInto instead of as-cast; Kani proves conversion safety
  SPARK no-.unwrap rule       -> maps directly to CLAUDE.md "no .unwrap() in production Rust" (panic = AoRTE failure at runtime)

The no-.unwrap() / no-panic rule in CLAUDE.md is the RUNTIME enforcement of the SPARK AoRTE guarantee: a panic is exactly a Constraint_Error-style runtime failure that AoRTE is designed to rule out at proof time. Kani/prusti/creusot extend this to compile-time/proof-time elimination.

--- RUST CRATE VERIFICATION (anti-slopsquat, verified 2026-06-21) ---

Kani (kani-verifier):
  crates.io: kani-verifier v0.67.0, released 2026-01-16, ~34,971 downloads/month
  GitHub: github.com/model-checking/kani (AWS-maintained, active CI, Rust 2024 edition support)
  Function: bit-precise bounded model checker for Rust; proves absence of panics, overflow, assertion failures
  Recommendation: PRIMARY choice for AoRTE proof obligations on ELAI Rust adapters
  Status: ACTIVELY MAINTAINED (66+ releases, weekly updates as of 2026-01)

Prusti (prusti-contracts):
  crates.io: prusti-contracts v0.2.0, released 2023-09-04, ~80 downloads/month
  GitHub: github.com/viperproject/prusti-dev; latest release v-2024-03-26-1504 (March 2024)
  Function: deductive verifier via Viper infrastructure; supports #[requires]/#[ensures] annotations (SPARK-style)
  Status: RESEARCH-GRADE, last release March 2024, low download count - treat as MEDIUM reliability, NOT recommended for production pipeline without active fork
  Use if: functional correctness proofs beyond AoRTE are needed (Gold/Platinum-equivalent level)

Creusot (creusot-rs/creusot):
  GitHub: github.com/creusot-rs/creusot; v0.12.0 released 2026-06-12, 1.8k stars, 4,879 commits
  Function: translates Rust to Why3 for deductive verification; handles ownership + borrow checker natively
  Status: ACTIVELY MAINTAINED (recent v0.12.0 release June 2026)
  Use if: full functional correctness (Platinum-equivalent) with Why3 backend is desired
  Note: not published as a standalone crate on crates.io; integrated as a tool chain component

=== PUBLIC API SURFACE ===

SPARK/GNATprove is a language + toolset, not a Rust crate. The "API" is the set of aspect annotations available in Ada/SPARK source.

Aspect | Applies to | Checked by | Guarantee
Pre => <expr>              | subprogram | GNATprove (caller), compiler (runtime) | expr is true on every entry
Post => <expr>             | subprogram | GNATprove (body), compiler (runtime)    | expr is true on every normal exit
Contract_Cases => (g1 => p1, g2 => p2, ...) | subprogram | GNATprove | exactly one guard true; matching postcondition holds
Exceptional_Cases => (E => <expr>) | subprogram | GNATprove | on raise E, expr holds of final state
Type_Invariant => <expr>   | private type completion | GNATprove | expr holds on all values visible outside package scope
Static_Predicate => <expr> | subtype | GNATprove | all values of subtype satisfy expr (static, can be used in case/loop)
Dynamic_Predicate => <expr>| subtype | GNATprove | all values of subtype satisfy expr (can reference record fields)
Ghost_Predicate => <expr>  | subtype | GNATprove only (erased in release build) | specification-only predicate
Loop_Invariant => <expr>   | loop body | GNATprove | holds before and after each iteration
Loop_Variant => <expr>     | loop body | GNATprove | proves loop termination

=== BEHAVIORAL CONTRACTS ===

Pre + GNATprove -> modular proof: body proven under assumption Pre holds; callers proven to satisfy Pre before call.
Post + GNATprove -> modular proof: body proven to establish Post on all paths; callers may assume Post after call.
Type_Invariant -> package boundary enforcement: broken freely inside body, restored at every public subprogram exit.
Predicates -> subtype-wide enforcement: GNATprove checks every assignment to the subtype satisfies the predicate.
AoRTE at Silver -> all 6 check categories (Division, Index, Overflow, Range, Length, Discriminant) proved absent.
Proof is MODULAR: GNATprove never crosses subprogram boundaries; contracts ARE the interface.

=== PLATFORM CONSTRAINTS ===

SPARK/GNATprove runs on: Linux, macOS, Windows (via GNAT toolchain)
Ada/SPARK 2014+, GNAT Pro or GNAT Community edition
For Rust analog: Kani requires rustup nightly + cargo-kani installed; runs on Linux/macOS/Windows
Kani is bounded: proof is sound within the specified loop unroll bound; deeper loops require increasing the bound
Creusot requires Why3 + SMT solvers (Alt-Ergo, CVC5, Z3) installed separately

=== GOTCHAS ===

Storage_Error (stack overflow, heap exhaustion) is NOT covered by SPARK AoRTE -- SPARK explicitly excludes it. Rust analog: Kani also does not prove stack-overflow absence; treat stack-heavy recursion separately.
Type_Invariant NOT supported on tagged types (polymorphic/OO types) in SPARK -- limited to concrete private types.
Prusti last release March 2024, prusti-contracts v0.2.0 Sep 2023 (~80 dl/month) -- effectively unmaintained for production use.
Kani is bounded model checking, not deductive proof: a sufficient unroll bound must be chosen; the tool does NOT guarantee proof for unbounded loops without an invariant annotation.
GNATprove timeout does NOT yield a proof failure -- it yields UNVERIFIED, not PASS. A missing proof under timeout is treated as a check not proved, not as absence of the error. This is the key fail-open risk: "no false negatives, and precise - no false positives, PROVIDED TIMEOUT IS NOT REACHED."
Pre is NOT checked by GNATprove for the body being verified -- it is ASSUMED. The obligation is on CALLERS to prove Pre holds. A body verified in isolation with Pre=False is vacuously proved.
GNATprove proof levels 0-4 trade completeness for speed; level 0 (cvc5, 1s) leaves many checks unverified that level 4 (all provers, 60s) would discharge.

=== CRITICAL ASSERTIONS (testable for the ELAI Rust contract gate) ===

# For a Rust fn marked as a side-effect sink (SSB write, file op, egress, credential),
# the following must hold before the fn is allowed in production wiring:

# 1. No .unwrap() or .expect() in fn body (panic = AoRTE failure)
assert "unwrap()" not in fn_source and ".expect(" not in fn_source, "AoRTE: fn must not panic via unwrap/expect"

# 2. All integer arithmetic uses checked ops or is proved by Kani
assert all(op in checked_arithmetic_set for op in fn_arithmetic_ops), "AoRTE: overflow check -- use checked_add/mul/sub"

# 3. All slice/array indexing uses .get() or has a prior bounds check
assert all(idx_site.has_bounds_check for idx_site in fn_index_sites), "AoRTE: index check -- no unchecked slice index"

# 4. No integer division without a prior non-zero check
assert all(div_site.divisor_checked_nonzero for div_site in fn_division_sites), "AoRTE: division check -- divisor != 0 must be asserted"

# 5. Kani proof harness exists for the fn (verifies AoRTE via bounded model checking)
assert kani_harness_exists(fn_name), "AoRTE: Kani proof harness required before fn may be a side-effect sink"

# 6. Pre/post assertions present at fn boundary
assert fn_has_entry_assert and fn_has_exit_assert, "Contract: Pre+Post assertions must exist on privileged adapters"

# 7. Type invariant: if fn mutates a shared struct, invariant check runs on exit
assert fn_calls_invariant_check_on_exit if fn_mutates_shared_struct else True, "Type_Invariant: invariant must be asserted on exit of any fn mutating shared state"
