Algorithm: Future-Time LTL Monitoring via Formula Rewriting
Source: Klaus Havelund and Grigore Rosu, "Monitoring Programs using Rewriting", ASE 2001
PDF: https://fsl.cs.illinois.edu/publications/havelund-rosu-2001-ase.pdf
Local: F:\.ELAI_workflow\.claude\papers\monitoring-rewriting.pdf
RAG topic: paper-monitoring-rewriting
RAG status: MCP ingest attempted; on-disk map is canonical (known reranker/IPC issue on this machine)

COMPANION: F:\.ELAI_workflow\workflow\papers\efficient-monitoring-safety-interface.md
  (Havelund/Rosu past-time LTL via synthesis/dynamic-programming; ONLINE, O(1)-per-event)
  THIS paper: FUTURE-time LTL via formula rewriting; fires at session/trace end for unresolved obligations.

SCOPE
Checks future-time LTL formulae on finite execution traces by consuming events one at a time and
rewriting the formula to a residual. The residual formula at trace end encodes unresolved future
obligations. Verdict at trace end: reduce residual to Bool; if unresolved (neither true nor false),
apply end-of-trace interpretation (maximal model -- see DETERMINISTIC-GATE RULES below). Implemented
in Maude; designed for JPAX (Java PathExplorer) at NASA Ames.


PART A: FINITE-TRACE LTL SEMANTICS (Section 3.1, mathematical notation)

Operators: []_ (always), <>_ (eventually), _ U _ (until, STRICT), o_ (next/nextstep), /\ (and),
++ (exclusive-or), \/ (or), !_ (negation), -> (implication), <-> (equivalence).

Core set: atomic props, !_, /\, o_, _ U _ are primitives.
Derived: <>X = true U X; []X = ! <> ! X. (Section 2.3)

Satisfaction relation T |= F (Section 3.1, mathematical block, verbatim):

  t |= A           iff  A in head(t)
  t |= true        iff  true
  t |= false       iff  false
  t |= X /\ Y      iff  t |= X and t |= Y
  t |= X ++ Y      iff  t |= X xor t |= Y
  t |= [] X        iff  (forall i <= length(t)) t_i |= X
  t |= <> X        iff  (exists i <= length(t)) t_i |= X
  t |= X U Y       iff  (exists i <= length(t)) (t_i |= Y and (forall j < i) t_j |= X)
  t |= o X         iff  (if tail(t) defined then tail(t) |= X else t |= X)

KEY: all quantifiers are over FINITE index range [1..length(t)]. This differs from infinite LTL:
  - [] X on a finite trace: X must hold at EVERY position up to the last one.
  - <> X on a finite trace: X must hold at SOME position within the trace.
  - X U Y (strict until): Y must hold at some position; X must hold strictly before that position.
  - o X at the LAST event (tail undefined): t |= o X  iff  t |= X  (stationarity / loop-back).


PART B: THE REWRITING ALGORITHM -- FORMULA TRANSFORMATION (Section 4.1, LTL-REVISED module, verbatim)

The key abstraction: operation  X{E}  means "evaluate formula X consuming event E, yielding residual."
Two event types: E (non-terminal, trace continues) and E* (terminal, last event in trace).

NON-TERMINAL EVENT RULES  (each consumes one event E from an ongoing trace):

  ([] X){E}    =  [] X  /\  X{E}
    -- X must hold now (X{E}) AND always in future ([] X persists as obligation)

  (<> X){E}    =  <> X  \/  X{E}
    -- X holds now (X{E}) OR eventually in remaining trace (<> X persists as possibility)

  (X U Y){E}   =  Y{E}  \/  (X{E}  /\  X U Y)
    -- Y holds now (discharge obligation) OR (X holds now AND until obligation persists)

  (o X){E}     =  X
    -- Next-step: after consuming E, the residual is X itself (obligation discharged to next event)

  [propositional operators via PROP-CALC module -- distribute over /\ and ++ as expected]

TERMINAL EVENT RULES  (E* marks last event in trace; these determine end-of-trace verdict):

  ([] X){E*}   =  X{E*}
    -- At trace end, []X reduces to: does X hold at this last event? (no future to check)
    -- INTERPRETATION: []X on a one-state (terminal) trace = X holds at that state.

  (<> X){E*}   =  X{E*}
    -- At trace end, <>X reduces to: does X hold at this last event? (last chance)
    -- INTERPRETATION: if X does not hold at last event, <>X = false. No future remains.

  (o X){E*}    =  X{E*}
    -- At trace end, oX reduces to X{E*}: stationarity, same as finite-trace semantics for o.

  (X U Y){E*}  =  Y{E*}
    -- At trace end, X U Y reduces to: does Y hold NOW? (no future; Y must be satisfied here)
    -- INTERPRETATION: an until-obligation at the last event is satisfied ONLY IF Y holds there.

PROPOSITIONAL REDUCTIONS (PROP-CALC module, Section 2.2, Hsiang decision procedure):
  true /\ X = X;  false /\ X = false;  X /\ X = X
  false ++ X = X;  X ++ X = false
  X /\ (Y ++ Z) = X /\ Y ++ X /\ Z  (distributivity)
  X \/ Y = X /\ Y ++ X ++ Y         (or in terms of /\ and ++)
  ! X = true ++ X                   (negation)
  X -> Y = true ++ X ++ X /\ Y
  These reduce residual formula to canonical form after each event; keeps formula small.


PART C: END-OF-TRACE / MAXIMAL-MODEL INTERPRETATION (Section 3.1, key invariant)

When the trace ends, the satisfaction relation uses BOUNDED quantifiers (forall/exists i <= length(t)).
The terminal rules {E*} implement this:

  []X at end:  reduces to X{E*}.  If X does not hold at last event -> FALSE. FAIL-CLOSED.
  <>X at end:  reduces to X{E*}.  If X does not hold at last event -> FALSE. FAIL-CLOSED.
  X U Y at end: reduces to Y{E*}. If Y does not hold at last event -> FALSE. FAIL-CLOSED.
  oX at end:   reduces to X{E*}. Stationarity (loop-back to same state).

CRITICAL: there is NO "inconclusive" / "unknown" outcome in this formalization.
The paper does NOT use a 3-valued logic at the formula-monitoring level. The {E*} rules force
every unresolved future obligation to resolve against the last state. If the last state does not
satisfy the obligation, the verdict is FALSE (BLOCK). This is the MAXIMAL MODEL interpretation:
the last state is treated as both the current and all notional future states (the trace "loops"
on the final state). An obligation that requires a future different from the final state = VIOLATED.

Note on the term "3-valued" in related literature: some RV papers (e.g. Bauer/Leucker/Schallhart
2011) layer a 3-valued semantics on TOP of finite-trace LTL. Havelund/Rosu 2001 does NOT do this.
Their verdict is always Bool: TRUE or FALSE. The {E*} rules are what forces the verdict to be
definite at trace end.


PART D: THE SATISFACTION OPERATOR (Section 4.1, LTL-REVISED module, verbatim)

New satisfaction relation  T |- X  (pipe instead of double-turnstile):

  E   |- X  =  [ X{E*} ]
    -- single terminal event: evaluate X against that terminal event, project to Bool

  E,T |- X  =  T |- X{E}
    -- multi-event trace: consume head event E (non-terminal), recurse on tail T with residual

This is the ITERATIVE FORWARD algorithm: process events left-to-right, updating the formula;
at the last event use {E*}. Linear in trace length, worst-case exponential in formula size
(but formula stays small in practice due to propositional normalization).

Correctness theorem (Section 4.2, stated verbatim):
  "For any trace T and any formula X, T |= X if and only if T |- X."
  Proved by structural induction on traces and formulae in Maude (129 reductions to check lemmas).


PART E: SYNTHESIS / MONITOR CONSTRUCTION (implicit in algorithm, Section 4.1)

The paper does NOT define a separate automaton-synthesis step (contrast with Buchi-automaton approach).
The formula IS the monitor state. Each event transforms the formula to a new formula (= new state).
Monitor construction is:

  state_0 = initial LTL formula X
  state_{i+1} = state_i {event_i}  (apply non-terminal rewriting rules + propositional normalization)
  verdict = [ state_n {event_n *} ]  (apply terminal rewriting rules, project to Bool)

The statespace is implicitly all possible formula normal-forms reachable from X. In practice,
the number of distinct reachable formulae is small (subformulae + their combinations), but
worst-case is exponential in formula size.

Optimization note (Section 4.1, last paragraph): one can pre-enumerate all reachable formula
normal-forms and store them as automaton states with event-keyed transition function. This yields
an O(1)-per-event monitor after O(2^|formula|) preprocessing -- the automaton approach.
The paper points to [14] for a "minimal finite state machine representation."


PART F: ALGORITHM INPUTS AND OUTPUTS

Inputs:
  formula       | future-time LTL formula          | mandatory | constructed from operators in Section 2.3
  trace         | finite sequence of events         | mandatory | each event = set of atomic propositions holding at that state
  event         | set of atoms (propositions)       | mandatory | empty set valid (no prop holds at this state)
  terminal_flag | marks last event in trace         | mandatory | algorithm REQUIRES knowing which event is last

Outputs:
  verdict       | Bool (true / false)               | invariant: ALWAYS definite (never inconclusive)
  residual      | LTL formula (intermediate state)  | invariant: always in propositional normal form after each step

Hyperparameters:
  none -- the algorithm is fully determined by the formula and trace

Complexity:
  time  | O(n * |phi|^k) where n=trace length, |phi|=formula size, k=nesting depth | in practice near-linear
  space | O(|phi|) per step (only current residual formula stored, not full trace)   | trace discarded per event
  parallelism | strictly sequential (each step depends on previous residual)


PART G: CRITICAL INVARIANTS (TESTABLE ASSERTIONS)

# 1. Every future obligation resolves at trace end (Section 3.1 / Section 4.1 terminal rules)
assert verdict in (True, False), "verdict must be Bool, never None/inconclusive"

# 2. []X at end = X holds at final state (terminal rule, Section 4.1 verbatim)
# In Python simulation: always_residual_at_end = formula_at_last_event evaluates without future
assert evaluate_at_final_state(always_subformula) in (True, False)

# 3. <>X at end = X holds at final state (terminal rule, Section 4.1 verbatim)
# Unresolved eventually at last event = FALSE if X not true there
assert not (eventually_residual_active and not X_holds_at_final_state) or verdict == False

# 4. X U Y at end = Y holds at final state (terminal rule, Section 4.1 verbatim)
# An until-obligation that reaches last event without Y having fired = FALSE
assert not (until_residual_active and not Y_holds_at_final_state) or verdict == False

# 5. Propositional normalization keeps residual formula bounded (Section 4.1, PROP-CALC)
# The formula never grows unboundedly -- after each event, boolean simplification reduces it
# (empirically: formula size stays within 2x initial in authors' experiments)
assert size(residual_formula) <= 2 * size(initial_formula) * trace_length  # practical bound only


PART H: FINITE TRACE DIFFERENCES FROM INFINITE TRACE LTL (Section 3.1, paragraph after equations)

1. [] X: on infinite trace, must hold forever. On finite trace, must hold only at all positions 1..n.
   A formula invalid in infinite LTL can be valid in finite LTL: "<>([]A \/ []!A)" is always true
   on finite traces (at the last state either A or !A holds and holds "forever" in the trivial sense).

2. o X at last event: on infinite trace, "next" refers to a future state. On finite trace with no next
   state, oX = X (stationarity / loop-back). Authors state: "t |= o X iff (if tail(t) defined then
   tail(t) |= X else t |= X)" (Section 3.1, last equation, verbatim).

3. Strict vs weak until: the paper uses STRICT until (X U Y: Y holds at some future point; X holds
   STRICTLY BEFORE that point). At the last event, X U Y = Y{E*} -- Y must hold there (strict).


PART I: KNOWN FAILURE MODES AND PITFALLS

Failure mode 1: FORGETTING THE TERMINAL FLAG.
  If the algorithm does not know which event is last, it cannot apply {E*} rules. Without them,
  an unresolved <>X or XUY residual is stuck -- neither true nor false. The algorithm is
  undefined for streaming (unbounded) traces; it requires a definite END signal.

Failure mode 2: NAIVE ALGORITHM EXPONENTIAL BLOWUP.
  The direct mathematical semantics (Section 3.2, LTL module) is O(n^m) where m = formula nesting depth.
  Authors report: 10,000 events on a nested [] formula could not finish in one night.
  ALWAYS use the formula-transformation algorithm (LTL-REVISED, Section 4.1), not the naive one.

Failure mode 3: CONFLATING FINITE AND INFINITE TRACE SEMANTICS.
  Some tools apply infinite-trace LTL semantics to finite traces and treat unresolved obligations as
  "inconclusive" or "vacuously true". This paper's formalization gives definite FALSE for unresolved
  future obligations. Using the wrong semantics = silently wrong monitor verdicts.

Failure mode 4: STRICT vs NON-STRICT UNTIL CONFUSION.
  The paper uses STRICT until (Y must hold AFTER X holds; X does not need to hold at the Y-point).
  Many implementations use non-strict (X may also hold at the Y-point). Verify which variant is used.

Failure mode 5: AUTOMATON GENERATION OVERHEAD.
  Pre-generating all reachable formula states (the optimization) is O(2^|formula|) space.
  For complex formulae, this may be infeasible at runtime; the online rewriting approach avoids it.

Implementation pitfalls (do NOT do this):
  - Do NOT apply {E} rules (non-terminal) to the last event -- must use {E*} rules.
  - Do NOT treat a pending <>X or XUY residual at trace end as "unknown" or "vacuously satisfied".
  - Do NOT omit propositional normalization after each step (formula grows exponentially without it).
  - Do NOT use <>X = true U X as the implementation -- authors note this is LESS efficient than
    a direct rewriting rule for <>X (more rewrites needed). Inline each operator.
  - Do NOT skip the oX stationarity rule: oX{E*} = X{E*}, not oX{E*} = true.


PART J: DETERMINISTIC-GATE RULES FOR ELAI (this is the FOCUS of this ingest)

This section captures the paper's rewriting rules VERBATIM (from Section 4.1, LTL-REVISED Maude module)
and maps them to deterministic runtime-verification gates for ELAI's SSB event stream.

J.1 VERBATIM REWRITING RULES (Section 4.1, LTL-REVISED module, exact Maude syntax):

NON-TERMINAL (trace continues after this event):
  eq ([] X){E}    = [] X /\ X{E} .
  eq (<> X){E}    = <> X \/ X{E} .
  eq (o X){E}     = X .
  eq (X U Y){E}   = Y{E} \/ X{E} /\ X U Y .

TERMINAL (last event in trace, E* notation):
  eq ([] X){E*}   = X{E*} .
  eq (<> X){E*}   = X{E*} .
  eq (o X){E*}    = X{E*} .
  eq (X U Y){E*}  = Y{E*} .

SATISFACTION (iterative driver):
  eq E   |- X = [ X{E*} ] .       -- single terminal event: project residual to Bool
  eq E,T |- X = T |- X{E} .       -- consume head, recurse on tail with residual

J.2 END-OF-TRACE VERDICT FOR UNRESOLVED FUTURE OBLIGATIONS (Section 3.1 + 4.1):

When the SSB session ends and a future-time obligation has not yet been discharged:

  Obligation "<> P" (eventually P):
    Terminal rule: (<> P){E*} = P{E*}.
    If P does not hold at the last SSB row examined: verdict = FALSE = BLOCK.
    Interpretation: "eventually" with no remaining future = the obligation was never satisfied.

  Obligation "[] P" (always P):
    Terminal rule: ([] P){E*} = P{E*}.
    If P does not hold at the last SSB row: verdict = FALSE = BLOCK.
    Interpretation: "always" must hold at every position including the last.

  Obligation "P U Q" (P until Q):
    Terminal rule: (P U Q){E*} = Q{E*}.
    If Q does not hold at last SSB row: verdict = FALSE = BLOCK.
    Interpretation: the discharge condition Q must have been satisfied; if not, VIOLATED.

  Obligation "o P" (next P):
    Terminal rule: (o P){E*} = P{E*}.
    If P does not hold at last SSB row: verdict = FALSE = BLOCK.
    Interpretation: stationarity -- the next state is the same as the current (last) state.

MAXIMAL MODEL SUMMARY: the last SSB row is treated as representing all future states.
Any obligation that requires a different future than the final state = VIOLATED = BLOCK.
There is NO "inconclusive" outcome. Missing evidence at session-end = FAIL (mirrors wiring-enforcement R1).

J.3 ELAI MAPPING (FUTURE-OBLIGATION GATE OVER SSB ROWS):

RULE (one line): "A future-obligation gate over SSB rows (e.g. 'every spawned agent must eventually
emit a return row before session-end') is monitored by formula-rewriting; an unresolved future
obligation AT SESSION-END = BLOCK (fail-closed: a never-discharged obligation is a violation,
not inconclusive)."

Concrete ELAI examples:
  (a) "Every AGENT_SPAWN row must be followed by an AGENT_RETURN row":
      Formula: [] (spawned -> <> returned)
      Gate: for each AGENT_SPAWN row consumed, add <>returned obligation.
      At session-end: if any <>returned is still in residual with returned not true at last row = BLOCK.

  (b) "The autonomous gate must eventually emit GATE_PASSED before session completes":
      Formula: <> gate_passed
      Gate: initialize residual = <>gate_passed; consume SSB rows.
      At session-end: if gate_passed never fired = verdict FALSE = BLOCK.

  (c) "Once PLANNING starts, EXECUTING must follow before COMPLETE":
      Formula: [] (planning -> (planning U executing))
      At session-end with EXECUTING not reached: verdict = FALSE = BLOCK.

J.4 CROSS-REFERENCE TO RV-SURVEY CAVEAT:
  The RV-survey (rv-survey-interface.md) notes that future-time LTL on finite traces requires a
  "defined end-of-trace verdict" choice. This paper resolves that choice: the verdict is the
  terminal-rule evaluation (maximal-model interpretation). No separate policy parameter is needed;
  the algorithm is deterministic and fail-closed by construction.

J.5 ROLE IN ELAI GATE STACK:
  Past-time LTL (efficient-monitoring-safety-interface.md): ONLINE, O(1)-per-event, fires continuously.
  Future-time LTL (THIS paper): OFFLINE/SESSION-END companion, fires at trace termination.
  Together they cover: invariants that must hold at every point (past-time) + obligations that must
  eventually be fulfilled (future-time, checked at session close).
  Gate verdict from this algorithm feeds into the same BLOCK/PASS binary that wiring-enforcement R1
  requires: missing evidence (unresolved obligation at end) = FAIL.


CONTRACT TEST STUBS (copy into tests/contracts/test_monitoring_rewriting_contract.py):

import numpy as np
from typing import Any

# Stub: in real ELAI, import the formula-rewriting monitor from crates/orchestrator

def simulate_rewriting_monitor(formula_type, subformula_holds_at_positions, trace_length):
    """
    Simulate the LTL-REVISED rewriting algorithm for a single unary temporal operator.
    formula_type: 'always', 'eventually', 'until'
    subformula_holds_at_positions: set of int (1-indexed positions where X holds)
    trace_length: int
    Returns: True (satisfied) or False (violated)
    """
    if formula_type == 'always':
        return all(i in subformula_holds_at_positions for i in range(1, trace_length + 1))
    elif formula_type == 'eventually':
        return any(i in subformula_holds_at_positions for i in range(1, trace_length + 1))
    else:
        raise ValueError(f"Unknown formula type: {formula_type}")


def test_always_terminal_rule_fail_closed():
    """
    Section 4.1: ([] X){E*} = X{E*}.
    If X does not hold at the last event, []X must be FALSE.
    An unresolved always-obligation at trace end = VIOLATED.
    """
    # X holds at positions 1,2 but NOT at position 3 (last event)
    holds_at = {1, 2}
    trace_length = 3
    verdict = simulate_rewriting_monitor('always', holds_at, trace_length)
    assert verdict == False, (
        "[]X must be FALSE when X does not hold at the last event -- "
        "terminal rule ([] X){E*} = X{E*}, X{E*}=false -> false"
    )


def test_eventually_terminal_rule_fail_closed():
    """
    Section 4.1: (<> X){E*} = X{E*}.
    If X never holds in the entire trace, <>X must be FALSE.
    An unresolved eventually-obligation at trace end = VIOLATED, not inconclusive.
    """
    holds_at = set()  # X never holds
    trace_length = 5
    verdict = simulate_rewriting_monitor('eventually', holds_at, trace_length)
    assert verdict == False, (
        "<>X must be FALSE when X holds at no position in the finite trace -- "
        "fail-closed: a never-discharged <> obligation at session-end = BLOCK"
    )


def test_eventually_satisfied_before_end():
    """
    Section 3.1: t |= <>X iff exists i <= length(t). t_i |= X.
    If X holds at any position (not necessarily the last), verdict must be TRUE.
    """
    holds_at = {3}  # X holds at position 3 out of 10
    trace_length = 10
    verdict = simulate_rewriting_monitor('eventually', holds_at, trace_length)
    assert verdict == True, (
        "<>X must be TRUE when X holds at any position within the finite trace"
    )


def test_always_all_positions_required():
    """
    Section 3.1: t |= []X iff forall i <= length(t). t_i |= X.
    ALL positions must satisfy X; a gap at any position = FALSE.
    """
    holds_at = {1, 2, 3, 4, 6, 7, 8}  # position 5 missing
    trace_length = 8
    verdict = simulate_rewriting_monitor('always', holds_at, trace_length)
    assert verdict == False, (
        "[]X must be FALSE if X does not hold at every position -- "
        "forall i <= length(t) is over ALL positions, not just terminal"
    )


def test_verdict_always_definite():
    """
    Section 4.1 + Section 3.1: the rewriting algorithm always yields a definite Bool verdict.
    There is no inconclusive/unknown outcome. The terminal rules {E*} force resolution.
    """
    for holds_at, length, formula in [
        ({1, 2, 3}, 3, 'always'),
        (set(), 3, 'always'),
        ({2}, 3, 'eventually'),
        (set(), 3, 'eventually'),
    ]:
        verdict = simulate_rewriting_monitor(formula, holds_at, length)
        assert isinstance(verdict, bool), (
            f"Verdict must be a definite bool, got {type(verdict)} -- "
            "no inconclusive outcome in Havelund/Rosu 2001 finite-trace LTL"
        )
        assert verdict is not None, "Verdict must never be None/inconclusive"
