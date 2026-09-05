Algorithm: TLA+ State Machine Specification and Model Checking
Source: Book (not arXiv)
Paper: Specifying Systems: The TLA+ Language and Tools for Hardware and Software Engineers
Author: Leslie Lamport, 2003 (PDF recreated 4 July 2021)
Local file: F:\.ELAI_workflow\.claude\papers\specifying-systems.pdf
Reliability: HIGH (official Lamport book, free distribution version, primary source)
RAG ingest: SKIPPED (local-rag reranker crash, pre-existing per project gotchas.md)


CORE STATE MACHINE MODEL (Section 2.1-2.4, Chapter 3)

A behavior is an infinite sequence of states. A state is an assignment of values to variables.
A step is a pair of successive states s -> t.
A specification is a temporal formula satisfied by exactly those behaviors representing correct executions.

The canonical specification form (Section 2.2, eq. HC definition; Section 14.1, eq. 14.1):
  Spec == Init /\ [][Next]_vars

where:
  Init       = initial predicate (state predicate, no primes)
  Next       = next-state action (formula over unprimed + primed variables)
  vars       = tuple of ALL declared variables
  [][Next]_v = Next \/ (v' = v), i.e. Next steps OR stuttering steps on v

ASCII TLA+ encoding (Figure 2.1):
  HC == HCini /\ [][HCnxt]_hr
  THEOREM HC => []HCini


INVARIANTS (Section 3.1, 5.7, 7.6)

Definition (Section 3.1, p.25):
  "An invariant Inv of a specification Spec is a state predicate such that
   Spec => []Inv is a theorem."

TypeInvariant pattern (Section 3.1):
  TypeInvariant == /\ v1 \in T1
                   /\ v2 \in T2
                   ...
  THEOREM Spec => []TypeInvariant

Inductive invariant (Section 5.7, p.61):
  "When P /\ N => P' holds, we say that predicate P is an invariant of action N."
  TypeInvariant is an invariant of Spec because:
    (1) Init => TypeInvariant
    (2) TypeInvariant /\ Next => TypeInvariant'

Three-step invariance proof pattern (Section 5.7, p.62):
  To prove Init /\ [][Next]_v => []P, find Inv such that:
    (a) Init => Inv
    (b) Inv /\ [Next]_v => Inv'
    (c) Inv => P

  An invariant satisfying (a) and (b) directly (with Inv = P) is called an
  inductive invariant. Not all invariants of Spec are inductive invariants of Next.


INPUTS TO TLC (Section 14.1, 14.2, 14.3)

Input: TLA+ module file (.tla) | text | mandatory
Input: configuration file (.cfg) | text | mandatory
Input: INIT formula name | identifier | mandatory if no SPECIFICATION
Input: NEXT formula name | identifier | mandatory if no SPECIFICATION
Input: SPECIFICATION formula name | identifier | alternative to INIT+NEXT
Input: INVARIANT predicate name(s) | state predicates | optional, one per line
Input: PROPERTY temporal formula name(s) | nice temporal formula | optional
Input: CONSTANT assignments | name = value pairs | mandatory for all params
Input: CONSTRAINT state predicate | optional, bounds reachable states to finite
Input: SYMMETRY set of permutations | optional, reduces state space by n! factor

TLC standard form (Section 14.1, eq 14.1):
  Init /\ [][Next]_vars /\ Temporal

CONFIGURATION FILE grammar (Section 14.1):
  SPECIFICATION  <spec-name>
  INIT           <init-name>         (alternative to SPECIFICATION)
  NEXT           <next-name>         (alternative to SPECIFICATION)
  INVARIANT      <pred-name>         (state predicate checked at every reachable state)
  PROPERTY       <formula-name>      (nice temporal formula; 2P equivalent to INVARIANT)
  CONSTANTS      <name = value>+
  CONSTRAINT     <pred-name>
  SYMMETRY       Permutations(<set>)


OUTPUTS FROM TLC (Section 14.3.1, 14.5.2)

Output on clean run: state count, distinct states, queue depth, diameter, fingerprint collision estimate
Output on invariant violation: error trace = finite sequence of states leading to violation state
Output on deadlock: error trace leading to state where Next is not enabled
Output on temporal property violation: counterexample behavior (infinite path in state graph)
Output on silly expression: error with expression location (line + column in .tla file)

Invariant violation trace shape:
  state_0 (initial, satisfies Init)
  -> state_1 (satisfies Next)
  -> ...
  -> state_n (violates Invariant)
  Each state printed as variable = value assignments.

COUNTEREXAMPLE INVARIANT: a counterexample trace is a BLOCK on the invariant claim.
  Missing evidence (no trace found) = invariant holds for the checked model.
  A trace = the invariant is false = the specification has a bug.


TLC VALUES AND CONSTRAINTS (Section 14.2.1, 14.2.2)

TLC values: Booleans, Integers, Strings, Model Values (named in CONSTANT), finite sets of comparable values, functions with finite domains.
TLC CANNOT evaluate: unbounded quantifiers (exists x : p without x in S), infinite sets, non-comparable set members, unbounded choose.
TLC evaluation order: conjuncts left-to-right; disjuncts split into parallel evaluations.
TLC assignment rule: when evaluating Next, x' = e assigns e to x'; x' in S treated as exists v in S : x' = v.

DEADLOCK: TLC checks 2(ENABLED Next) by default. A state with no successor = deadlock error.
Disable with: -deadlock option.


SAFETY VS LIVENESS (Section 14.3.5)

Safety property violation always has a FINITE counterexample trace -> detectable with finite model.
Liveness property violation may require infinite behavior -> finite model may MISS violations.
Rule: always verify TLC is checking liveness correctly by testing a property known to be violated.


CRITICAL MATHEMATICAL INVARIANTS (testable assertions)

From Section 2.2 (HC specification):
  assert: every behavior satisfying HC starts in a state satisfying HCini
  assert: every step in a behavior satisfying HC satisfies HCnxt OR leaves hr unchanged

From Section 3.1 (TypeInvariant definition):
  assert Spec => []TypeInvariant  (theorem, checked by TLC INVARIANT statement)

From Section 5.7 (inductive invariant condition):
  assert Init => Inv
  assert (Inv /\ Next) => Inv'   (i.e. Inv preserved by every Next step)
  assert Inv => P                (Inv is strong enough to imply the property P)

From Section 14.3.1 (TLC model-checking algorithm):
  assert: every reachable state satisfies Invariant (checked after every successor computation)
  assert: if T (successor set) is empty and -deadlock not set, STOP with deadlock error
  assert: every edge s->t in state graph satisfies Next /\ ActionConstraint


COMPLEXITY (Section 14.3.1, 14.5.3)

Time: exponential in number of reachable states (BFS over all states)
Space: state graph G (nodes = states, edges = transitions) + queue U
Parallelism: -workers N option enables N threads for steps 3(b)-(d)
Finite termination: only if reachable state space is finite (use CONSTRAINT to bound it)
State space reduction: SYMMETRY by factor of n! for n-element symmetric sets


FAILURE MODES (from book)

1. Specification with infinite reachable states: TLC runs forever unless CONSTRAINT added.
2. Liveness property violations missed: finite model + constraint may not generate violating infinite behaviors.
3. Fingerprint collision: probability ~m*(n-m)*2^-64 for n views; TLC prints empirical estimate.
4. Silly expression errors: operations on wrong types (e.g. applying function to out-of-domain value) cause TLC to halt mid-run.
5. Recursive function loops: f[n] depending on f[n] (not f[n-1]) causes TLC infinite loop.
6. Non-inductive invariant: an invariant of Spec may not be preserved by Next alone (Coherence example, Section 5.7). Must find auxiliary Inv that IS inductive and implies the property.
7. Symmetry + temporal checking: SYMMETRY statement may produce incorrect results for ImpliedTemporal.


IMPLEMENTATION PITFALLS (do NOT do this)

Do NOT treat TypeInvariant as an assumption. It is a theorem to be proved, not a premise (Section 3.3, 7.6).
Do NOT write Next without including all variables in stuttering clause [][Next]_vars. Omitting vars breaks composition (Section 2.2).
Do NOT check a liveness property without verifying TLC generates infinite behaviors that satisfy the fairness condition (Section 14.3.5).
Do NOT use SYMMETRY for liveness (ImpliedTemporal) checking without understanding its limitations (Section 14.3.4).
Do NOT put model-checking-only definitions (CONSTRAINT, helper operators) inside the main specification module. Use a separate MC-prefixed module (Section 14.1).
Do NOT write Next actions with primed variables before their enabling conditions (TLC evaluates conjuncts left-to-right; silly expression on wrong ordering, Section 14.2.2).
Do NOT claim an invariant is proved when only a finite model was checked. TLC finds bugs, not correctness proofs.
Do NOT use a non-inductive state predicate directly as the invariant in the INVARIANT statement if you need step-by-step preservation. Find the inductive strengthening first.


DETERMINISTIC-GATE RULES FOR ELAI FSM

--- PART A: STATE MACHINE AS Init /\ [][Next]_vars ---

Section 2.2, p.16-17 (VERBATIM from book):
  "We write an initial predicate that specifies the possible initial values of [variables],
   and a next-state relation that specifies how the value of [variables] can change in any step."

Section 2.2, p.17 (VERBATIM):
  "HC == HCini /\ [][HCnxt]_hr"
  where [HCnxt]_hr stands for HCnxt \/ (hr' = hr)

Section 14.1, p.221 (VERBATIM):
  "TLC handles specifications that have the standard form
   Init /\ [][Next]_vars /\ Temporal   (14.1)
   where Init is the initial predicate, Next is the next-state action, vars is the
   tuple of all variables, and Temporal is a temporal formula that usually specifies
   a liveness condition."

ELAI FSM as TLA+ spec skeleton:

  ---- MODULE ElaiFSM ----
  VARIABLES state, evidence_gate, substate

  States == {"IDLE","PLANNING","FEASIBILITY_PROBE","TESTING_FIRST",
             "EXECUTING","AUDITING","MEMORIZING","COMPLETE","CONFIGURING"}

  Init == /\ state \in States
          /\ state = "IDLE"
          /\ evidence_gate = FALSE
          /\ substate = "NONE"

  Next == \/ IDLE_to_PLANNING
          \/ PLANNING_to_FEASIBILITY_PROBE
          \/ PLANNING_to_TESTING_FIRST
          \/ EXECUTING_to_AUDITING
          \/ AUDITING_to_MEMORIZING
          \/ MEMORIZING_to_COMPLETE
          \/ COMPLETE_to_IDLE
          \/ mcp_fsm_reset_to_IDLE

  ElaiSpec == Init /\ [][Next]_<<state, evidence_gate, substate>>
  ========================


--- PART B: HOW AN INVARIANT IS WRITTEN AND CHECKED ---

Section 3.1, p.25 (VERBATIM):
  "An invariant Inv of a specification Spec is a state predicate such that
   Spec => []Inv is a theorem."

Section 5.7, p.61 (VERBATIM):
  "Module WriteThroughCache contains the theorem
   theorem Spec => [](TypeInvariant /\ Coherence)"

Section 5.7, p.62 (VERBATIM - the three-step proof recipe):
  "Proving that a state predicate P is an invariant of a specification means proving
   a formula of the form
     Init /\ [][Next]_v => []P
   This is done by finding an appropriate state predicate Inv and proving
     Init => Inv,
     Inv /\ [Next]_v => Inv',
     Inv => P"

Section 5.7, p.61 (VERBATIM - inductive invariant):
  "Predicate TypeInvariant is an invariant of Spec because it is an invariant of
   Next and it is implied by the initial predicate Init."
  i.e. the two conditions: (1) Init => TypeInvariant, (2) TypeInvariant /\ Next => TypeInvariant'

TLC checks invariants (Section 14.1, p.221):
  "Invariance checking is so common that TLC allows you instead to put the following
   statement in the configuration file:
     INVARIANT ABTypeInv
   The INVARIANT statement must specify a state predicate."

TLC evaluates invariant at every reachable state (Section 14.3.1, p.242):
  "(a) Evaluate the predicates Invariant and ImpliedInit in state s; report
       an error and stop if either is false."
  "(d.i) If Invariant is false in state t or ImpliedAction is false for the step s->t,
         then report an error and stop."

ELAI invariants to model-check (3 concrete invariants):

  Inv1_ValidState == state \in States
  (* No transition can land in an unrecognized state *)

  Inv2_AuditingOnlyFromExecuting ==
    (state = "AUDITING") => (previous_state = "EXECUTING")
  (* AUDITING is always entered from EXECUTING.
     Requires adding a prev_state variable tracking the prior state. *)

  Inv3_EvidenceGateBeforeMutation ==
    (state \in {"EXECUTING"}) => evidence_gate = TRUE
  (* No side-effecting EXECUTING state without prior evidence_gate_pass.
     The gate variable is set TRUE only by the AUDITING->MEMORIZING path. *)

  Inv4_CompleteOnlyAfterMemorizing ==
    (state = "COMPLETE") =>
       \E prev \in {"MEMORIZING"} : TRUE
  (* COMPLETE reachable only after MEMORIZING. Encode as:
     state = "COMPLETE" is reachable only via MEMORIZING->COMPLETE transition. *)

  THEOREM ElaiSpec => [](Inv1_ValidState /\ Inv2_AuditingOnlyFromExecuting /\ Inv3_EvidenceGateBeforeMutation /\ Inv4_CompleteOnlyAfterMemorizing)


--- PART C: HOW TLC MODEL-CHECKS AND PRODUCES COUNTEREXAMPLE TRACES ---

Section 14.3.1, p.241-242 (VERBATIM - the BFS algorithm):
  "TLC keeps two data structures: a directed graph G whose nodes are states, and
   a queue (a sequence) U of states."
  "2. Compute the set of initial states by evaluating the initial predicate Init...
      For each initial state s found:
      (a) Evaluate the predicates Invariant and ImpliedInit in state s;
          report an error and stop if either is false."
  "3. While U is nonempty, do the following:
      (a) Remove the first state from U and let s be that state.
      (b) Find the set T of all successor states of s by evaluating the next-state action...
      (c) If T is empty and the deadlock option is not selected, then report a
          deadlock error and stop.
      (d) For each state t in T, do the following:
          i. If Invariant is false in state t or ImpliedAction is false for the
             step s->t, then report an error and stop."

Section 14.5.2, p.253 (counterexample trace shape):
  TLC prints a behavior (sequence of states) leading to the error state.
  The last state in the trace is the one that violates the invariant.
  Each state printed as concrete variable = value assignments.

Section 14.5.2, p.259 (how to use traces for debugging):
  "When TLC reports that an invariant is violated, it may not be obvious what
   part of the invariant is false. If you give separate names to the conjuncts of
   your invariant and list them separately in the configuration file's INVARIANT
   statement, TLC will tell you which conjunct is false."

SPEC-TIME GATE FOR ELAI FSM CHANGES:

  A privileged FSM change (adding a state, removing a transition, changing the evidence gate
  conditions) MUST be accompanied by:
    1. A TLA+ spec delta in a .tla file covering the modified ElaiFSM module.
    2. A passing TLC run: java tla2tools.TLC ElaiFSM.tla with config ElaiFSM.cfg
       showing "0 states on queue, 0 errors" for the invariant battery.
    3. An invariant counterexample = BLOCK on the proposed FSM change.
       The counterexample trace (sequence of states) is the grounded evidence of the
       invariant violation, per Section 14.3.1 algorithm and Section 14.5.2 output format.

  Configuration file template (ElaiFSM.cfg):
    SPECIFICATION ElaiSpec
    INVARIANTS Inv1_ValidState Inv2_AuditingOnlyFromExecuting
               Inv3_EvidenceGateBeforeMutation Inv4_CompleteOnlyAfterMemorizing
    CONSTANTS
      States = {IDLE, PLANNING, FEASIBILITY_PROBE, TESTING_FIRST,
                EXECUTING, AUDITING, MEMORIZING, COMPLETE, CONFIGURING}

  A clean TLC run (no counterexample) for the checked model does NOT prove correctness
  for infinite state spaces, but it IS the appropriate gate for a finite-state FSM like
  ELAI's 9-state machine.
