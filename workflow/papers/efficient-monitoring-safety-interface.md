Algorithm: Efficient Monitoring of Safety Properties via Past-Time LTL (ptLTL) Synthesis
Source: Klaus Havelund and Grigore Rosu, "Efficient Monitoring of Safety Properties", STTT 2004
PDF: https://www.havelund.com/Publications/sttt-tacas02.pdf
Local: F:\.ELAI_workflow\.claude\papers\efficient-monitoring.pdf
RAG topic: paper-efficient-monitoring-safety
RAG status: MCP namespace not reachable in this agent context (known reranker/IPC issue); on-disk map is canonical

SCOPE
Checks safety formulae of the form "always P" where P is a past-time LTL (ptLTL) formula over a finite
sequence of events. Both algorithms (rewriting via Maude, synthesis via dynamic programming) process events
online in a single left-to-right pass and discard each event after processing. This paper is the theoretical
and algorithmic foundation for compiling a ptLTL formula into a deterministic, O(1)-per-event gate over an
append-only event stream -- exactly what ELAI's SSB is.


PART A: PAST-TIME LTL OPERATORS AND SEMANTICS
(Section 3, Figure 3 -- formal non-recursive semantics)

Syntax (Section 3.1, equation block):
  F := true | false | A | not F | F op F          (propositional)
       | circle F | diamond F | box F             (previously, eventually-past, always-past)
       | F Ss F | F Sw F                          (strong-since, weak-since)
       | up F | down F | [F,F)s | [F,F)w         (monitoring: start, end, strong-interval, weak-interval)

where A is a finite set of atomic propositions, op in {and, or, implies, iff, xor}.

Standard past-time operators (Figure 3, verbatim):
  circle F  ("previously F"):
    t |= circle F   iff  t' |= F, where t' = t_{n-1} if n>1 and t' = t if n=1
    [BASE CASE: on a single-state trace, circle F = F. "Best guess that past was stationary."]

  diamond F ("eventually in the past F"):
    t |= diamond F  iff  t_i |= F for some 1 <= i <= n

  box F ("always in the past F"):
    t |= box F      iff  t_i |= F for all 1 <= i <= n

  F1 Ss F2 ("F1 strong since F2"):
    t |= F1 Ss F2   iff  t_j |= F2 for some 1<=j<=n AND t_i |= F1 for all j < i <= n

  F1 Sw F2 ("F1 weak since F2"):
    t |= F1 Sw F2   iff  (t |= F1 Ss F2) OR (t |= box F1)

Monitoring operators (Figure 3, verbatim):
  up F ("start F", F just became true):
    t |= up F       iff  t |= F AND t_{n-1} |!= F

  down F ("end F", F just became false):
    t |= down F     iff  t_{n-1} |= F AND t |!= F

  [F1,F2)s ("strong interval: F1 was true, F2 not since"):
    t |= [F1,F2)s   iff  t_j |= F1 for some 1<=j<=n AND t_i |!= F2 for all j<=i<=n

  [F1,F2)w ("weak interval"):
    t |= [F1,F2)w   iff  (t |= [F1,F2)s) OR (t |= box(not F2))

Equivalences (Section 3.3, proof items 7-10, 12-14):
  up F     = F and not circle F
  down F   = not F and circle F
  [F1,F2)s = not F2 and ((circle not F2) Ss F1)
  [F1,F2)w = not F2 and ((circle not F2) Sw F1)
  [F1,F2)w = (box not F2) or [F1,F2)s
  [F1,F2)s = (diamond F1) and [F1,F2)w

Theorem 1 (Section 3.4): ptLTL with any of 12 two-operator subsets is equivalent to full ptLTL. Minimum
sufficient pairs include {circle, Ss}, {circle, Sw}, {up, Ss}, {down, [,)s}, etc.


PART B: RECURSIVE (INCREMENTAL) SEMANTICS -- THE ONLINE RECURRENCES
(Section 3.3, Figure 4 -- these are the recurrences used in the synthesis algorithm)

For trace t = s1 s2 ... sn (n events), let t_i = s1..si. The monitoring algorithm maintains one boolean
bit per subformula. On event i, new_val(phi_j) is computed from now[] (current event) and pre[] (previous
event) using only the values of subformulae computed earlier in the same bottom-up pass.

Figure 4 recurrences (verbatim, restated in algorithmic notation):

  phi = variable a:
    now[j] = a(state)          -- evaluate atomic proposition in current state

  phi = not phi_{j'}:
    now[j] = not now[j']

  phi = phi_{j1} op phi_{j2}:
    now[j] = now[j1] op now[j2]

  phi = circle phi_{j1}  ("previously"):
    initialization:  pre[j] = pre[j1]   (stationary: circle F = F on first event, Section 3.2)
    loop:            now[j] = pre[j1]   -- the PREVIOUS value of j1 (Figure 4 line 1)

  phi = diamond phi_{j1} ("eventually in the past"):
    initialization:  pre[j] = pre[j1]
    loop:            now[j] = pre[j] or now[j1]   (Figure 4: t|=diamond F iff t|=F or (n>1 and t_{n-1}|=diamond F))

  phi = box phi_{j1} ("always in the past"):
    initialization:  pre[j] = pre[j1]
    loop:            now[j] = pre[j] and now[j1]  (Figure 4: t|=box F iff t|=F and (n>1 implies t_{n-1}|=box F))

  phi = phi_{j1} Ss phi_{j2} ("strong since"):
    initialization:  pre[j] = pre[j2]                        (true iff F2 holds at t_1)
    loop:            now[j] = now[j2] or (now[j1] and pre[j])   (Figure 4 line 3)

  phi = phi_{j1} Sw phi_{j2} ("weak since"):
    initialization:  pre[j] = now[j1] or now[j2]            (true iff F1 or F2 holds at t_1)
    loop:            now[j] = now[j2] or (now[j1] and pre[j])   (identical loop body to Ss; differs only in init)

  phi = up phi_{j'} ("start"):
    initialization:  pre[j] = false   (no transition at first event, Section 5.1 explanation)
    loop:            now[j] = now[j'] and not pre[j']

  phi = down phi_{j'} ("end"):
    initialization:  pre[j] = false
    loop:            now[j] = not now[j'] and pre[j']

  phi = [phi_{j1}, phi_{j2})s  ("strong interval"):
    initialization:  pre[j] = pre[j1] and not pre[j2]       (F1 true and F2 false at t_1)
    loop:            now[j] = (pre[j] or now[j1]) and not now[j2]   (Figure 4 line 5)

  phi = [phi_{j1}, phi_{j2})w  ("weak interval"):
    initialization:  pre[j] = not pre[j2]
    loop:            now[j] = (pre[j] or now[j1]) and not now[j2]   (identical loop body to strong interval)

KEY INVARIANT (Section 5.1, paragraph after example):
  "pre[i] = truth value of phi_i on the trace up to the PREVIOUS event"
  "now[i] = truth value of phi_i on the trace up to the CURRENT event"
  "now[0] = 0 (false) means the safety property is violated at this event"


PART C: MONITOR SYNTHESIS ALGORITHM
(Section 5.2, The Algorithm Formalized)

Input: ptLTL formula phi
Output: imperative program that checks "always phi" online over a trace

Step 1 -- Enumerate subformulae top-down with invariant:
  Assign indices phi_0, phi_1, ..., phi_m such that phi_0 is the root (whole formula) and for every j,
  every subformula of phi_j has a STRICTLY LARGER index. (Post-order or any topological sort of the
  syntax DAG satisfies this.)

Step 2 -- Generate declarations:
  State state = {}
  bit pre[0..m]
  bit now[0..m]

Step 3 -- Generate initialization code (for first event s_1):
  state = update(state, s_1)
  for j = m downto 0:
    pre[j] = <initialization expression from PART B, evaluated using already-computed pre[k] for k>j>

Step 4 -- Generate event loop (for events s_2 .. s_n):
  for i = 2 to n:
    state = update(state, s_i)
    for j = m downto 0:
      now[j] = <loop expression from PART B, referencing now[k] for k>j and pre[k] for any k>
    if now[0] == 0: output("property violated")
    pre = now

The inner loop over j runs bottom-up (high index to low) so every now[k] referenced when computing now[j]
is already computed in the SAME iteration. This is the key structural invariant enabling O(1) per event.

Optimization 1 (Section 5.3): Only bits referenced by temporally-preceding subformulae need to be stored
in pre[]. Propositional bits whose value is fully recomputed from now[] children do NOT need a pre[] slot.
This reduces pre[] to one bit per temporal operator only.

Optimization 2 (Section 5.3): After substituting and simplifying, generate a conditional-expression tree
(if-then-else / ternary) for each now[j] assignment. This implements short-circuit evaluation and avoids
evaluating atomic predicates whose values are not needed given already-computed bits (BDD-style).

Optimization 3 (Section 5.3): Store pre[] and now[] in CPU registers. All operations are bitwise;
the generated inner loop is expected to run faster than O(m) in practice due to parallelism and cache locality.


PART D: ALGORITHM INPUTS, OUTPUTS, COMPLEXITY

Inputs:
  formula    | ptLTL formula phi          | syntax tree | mandatory | "always phi" is the safety property
  trace      | sequence of abstract states | s_1..s_n   | mandatory | events from monitored system; each state must allow evaluating atomic propositions
  eval_fn    | proposition evaluator       | (atom, state) -> bool | mandatory | provided by instrumentation layer

Outputs:
  violated   | bool | per event | true = property still holds after this event, false = safety violation at this event
  now[]      | bit array | size m+1 | now[0] is the verdict; now[j] is truth of phi_j on current prefix

Hyperparameters:
  m          | formula size (number of subformulae) | derived from formula | determines array sizes and inner loop count | no tuning needed
  stationary_init | bool | true (paper default) | whether first-event initialization assumes stationary past | Section 3.2; "best guess is stationary past"

Complexity:
  Time per event:     O(m) where m = size of formula (Section 4.5, Theorem implicit in Section 5.1)
  Space:              O(m) -- two bit-arrays of size m+1 plus the formula syntax tree
  Total for n events: O(n * m) -- "best one can asymptotically hope from a runtime monitoring algorithm" (Section 5.1)
  Synthesis time:     O(m) -- formula enumeration is linear, code generation is linear in m
  After optimization: sub-O(m) per event in practice (short-circuit, register storage)
  Parallelism:        inner loop has simple dependency structure; compiler can parallelize/unroll; hardware impl possible (Section 5.3)


PART E: DETERMINISTIC-GATE RULES FOR ELAI SSB INTEGRATION

ELAI's SSB (Secure State Bus) is an append-only event stream of typed rows (action_type + payload).
Each SSB row is an abstract state. Monitoring "always P" over SSB means: on every appended row, evaluate
the ptLTL formula P against the prefix trace seen so far.

MAPPING:
  abstract state s_i        -> SSB row i (action_type + payload fields)
  atomic proposition a      -> predicate over SSB row fields, e.g. action_type == "policy_allow"
  trace t = s_1..s_n        -> the SSB log from row 0 to the current row
  "property violated" event -> emit a gate_violation SSB row or raise a pre-tool-use BLOCK

IMPLEMENTATION PATTERN (from synthesis algorithm, Section 5):
  1. Compile the ptLTL formula to a pre[]/now[] assignment sequence (the synthesis algorithm).
  2. Maintain pre[] as persistent state across SSB rows (the observer state machine).
  3. On each new SSB row: evaluate now[] bottom-up in the generated order, check now[0], copy now[] to pre[].
  4. This is exactly the "offline monitoring" mode (Section 5.4.1): monitor runs as a side-channel observer
     on the SSB stream.

THREE CONCRETE ELAI GATE EXAMPLES (past-time LTL safety formulae over SSB action_types):

GATE 1: "A write-capable tool-call is always preceded by a policy-allow row"
  (No write can occur without prior authorization)

  Atomic propositions:
    write_tool(row)  := row.action_type in {"Write", "Edit", "Bash"} and row.is_mutation == true
    policy_allow(row) := row.action_type == "policy_allow"

  ptLTL formula:
    G(write_tool -> diamond policy_allow)
    = always(write_tool implies (eventually-in-past policy_allow))

  In ptLTL safety form "always P":
    P = not write_tool or diamond policy_allow

  Recurrences (two subformulae: phi_1 = policy_allow, phi_0 = P):
    pre[1] = policy_allow(s_1)
    pre[0] = not write_tool(s_1) or pre[1]
    loop:
      now[1] = pre[1] or policy_allow(s_i)   [diamond: accumulate or]
      now[0] = not write_tool(s_i) or now[1]
      if not now[0]: GATE_VIOLATION("write without policy_allow")
      pre = now

GATE 2: "No egress row after a retrieval-scope-widen row unless a declassify row intervened"
  (Once scope is widened, all egress is blocked until an explicit declassify)

  Atomic propositions:
    scope_widen(row)  := row.action_type == "retrieval_scope_widen"
    egress(row)       := row.action_type in {"egress", "external_send", "file_write_external"}
    declassify(row)   := row.action_type == "declassify"

  ptLTL formula:
    G(egress implies not [scope_widen, declassify)_s)
    = always(egress implies not [scope_widen, declassify)_s)

  where [scope_widen, declassify)_s means: "scope_widen happened in the past and declassify has NOT happened since"

  In ptLTL safety form "always P":
    P = not egress or not [scope_widen, declassify)_s

  Subformulae enumeration (phi_3 = scope_widen, phi_2 = declassify, phi_1 = [phi_3, phi_2)_s, phi_0 = P):
    pre[3] = scope_widen(s_1)
    pre[2] = declassify(s_1)
    pre[1] = pre[3] and not pre[2]           [strong-interval init: F1 true and F2 false]
    pre[0] = not egress(s_1) or not pre[1]
    loop:
      now[3] = scope_widen(s_i)
      now[2] = declassify(s_i)
      now[1] = (pre[1] or now[3]) and not now[2]   [strong-interval recurrence]
      now[0] = not egress(s_i) or not now[1]
      if not now[0]: GATE_VIOLATION("egress after scope_widen without declassify")
      pre = now

GATE 3: "Every TestOutcome row with passed=true is preceded by a matching stdout_hash row"
  (No test can be declared passed unless the output hash was recorded first)

  Atomic propositions:
    test_passed(row) := row.action_type == "TestOutcome" and row.passed == true
    hash_recorded(row) := row.action_type == "stdout_hash"

  ptLTL formula:
    G(test_passed implies (diamond hash_recorded))
    = always(test_passed implies eventually-in-past(hash_recorded))

  In ptLTL safety form "always P":
    P = not test_passed or diamond hash_recorded

  Subformulae (phi_1 = hash_recorded, phi_0 = P):
    pre[1] = hash_recorded(s_1)
    pre[0] = not test_passed(s_1) or pre[1]
    loop:
      now[1] = pre[1] or hash_recorded(s_i)   [diamond recurrence]
      now[0] = not test_passed(s_i) or now[1]
      if not now[0]: GATE_VIOLATION("TestOutcome passed=true with no prior stdout_hash")
      pre = now


PART F: CRITICAL INVARIANTS (testable assertions)

1. Enumeration invariant (Section 5.2):
   assert all(j_sub > j_root for all subformula phi_{j_sub} of phi_{j_root})
   -- guarantees bottom-up evaluation in one pass produces correct values

2. Now-before-pre consistency (Section 5.1, loop body):
   assert len(now) == len(pre) == m + 1
   -- m = number of subformulae of phi

3. Initialization base case: circle F = F on single-state trace (Section 3.2, Figure 3):
   assert pre[j_circle] == pre[j_child]  # for previously operator at init

4. Strong-since init (Figure 4, base case):
   # pre[j] for Ss initialized to truth of F2 at t_1 (not F1)
   assert pre[j_Ss] == eval_F2(s_1)

5. Up/Down init = false (Section 5.1, "no change in state" at initialization):
   assert pre[j_up] == False
   assert pre[j_down] == False

6. Safety check is on now[0] after every event (Section 5.2):
   assert monitor.check(event) returns bool  # False = violation

7. O(1) amortized gate state: pre[] size does not grow with trace length (Section 5.1):
   assert len(pre) == m + 1  # constant, independent of trace length n


PART G: KNOWN FAILURE MODES AND IMPLEMENTATION PITFALLS

Failure modes (from paper):
  1. Using future-time LTL instead of past-time: future-time requires buffering the full future, incompatible
     with online single-pass monitoring. Past-time is required for O(1) state. (Section 1)
  2. Using non-recursive (global) semantics for implementation: leads to O(n) per event instead of O(1).
     The recursive semantics in Figure 4 is required. (Section 3.3)
  3. Wrong base case for circle on first event: if circle F = false at t_1, contradicts stationary
     interpretation; the paper explicitly adopts circle F = F at n=1. (Section 3.2)
  4. Failing to initialize pre[] before the event loop: the initialization code (Section 5.2 Step 3) is
     mandatory; skipping it leaves pre[] in an undefined state.
  5. Evaluating now[] in top-down (root-first) order: root formula depends on subformulae; must go
     high-index (leaves) to low-index (root). Top-down ordering violates the enumeration invariant. (Section 5.2)

Implementation pitfalls (do NOT do this):
  - Do NOT evaluate the formula from scratch over the full trace on every new event (O(n*m) per event, O(n^2*m) total).
  - Do NOT store the full trace; only pre[] and now[] are needed (two bit-arrays, O(m) space).
  - Do NOT use the same recurrence expression for initialization and loop body: init and loop differ for
    up, down, strong-since (weak-since differs ONLY in init, identical loop body to strong-since).
  - Do NOT confuse strong-since Ss and weak-since Sw: they have the same loop body but DIFFERENT init
    (Ss: pre[j] = eval_F2(s_1); Sw: pre[j] = eval_F1(s_1) or eval_F2(s_1)).
  - Do NOT use strong-interval and weak-interval interchangeably: same loop body, different init
    ([F1,F2)s init: pre[j1] and not pre[j2]; [F1,F2)w init: not pre[j2]).
  - Do NOT omit the "pre = now" copy at end of each event loop iteration.
  - Do NOT check now[j] for j != 0 as the safety verdict: only now[0] (root formula) is the gate.
  - Do NOT use future-time operators (next, until, eventually-future) -- they require buffering future events.
  - Do NOT use batch/async evaluation: the monitor is inherently sequential event-by-event (causal ordering).
  - In the ternary/BDD optimization (Section 5.3): do NOT assume the shortest ternary expression has the
    lowest expected runtime without knowing predicate costs; the paper notes this is a conjecture.


PART H: CONTRACT TEST STUBS

Copy into tests/contracts/test_efficient_monitoring_contract.py:

"""
Contract tests for ptLTL monitoring algorithm per Havelund & Rosu STTT 2004.
No imports beyond stdlib + numpy.
"""

def test_circle_base_case_stationary():
    # Section 3.2: on a single-state trace, circle F = F (stationary past assumption)
    # Simulates: phi = circle a, trace = [a=True]
    a_val = True
    pre = [None, None]
    # Initialization: pre[1] = a(s_1); pre[0] = circle a -> pre[1] = pre[1] (stationary)
    pre[1] = a_val          # a at t_1
    pre[0] = pre[1]         # circle a at t_1 = a at t_1 (stationary base case)
    assert pre[0] == True, "circle F must equal F on first event (stationary semantics)"


def test_strong_since_init_is_F2_not_F1():
    # Section 5.1 + Figure 4: Ss initialized to truth of F2 at first event, NOT F1
    # phi = F1 Ss F2; trace s_1 where F1=True, F2=False
    F1_val, F2_val = True, False
    pre = [None, None, None]
    pre[2] = F2_val   # phi_2 = F2
    pre[1] = F1_val   # phi_1 = F1
    pre[0] = pre[2]   # phi_0 = F1 Ss F2, init = eval(F2) at s_1
    assert pre[0] == False, "Ss must initialize to F2(s_1), not F1(s_1)"


def test_up_down_init_false():
    # Section 5.1: start (up) and end (down) are false at first event (no prior state change)
    pre_up = False
    pre_down = False
    assert pre_up == False, "up F must be false at initialization"
    assert pre_down == False, "down F must be false at initialization"


def test_diamond_monotone_accumulates():
    # Figure 4: diamond F once true stays true (monotone); recurrence: now = pre or now_child
    # Trace: a=False, a=False, a=True, a=False
    events = [False, False, True, False]
    m = 1  # phi_0 = diamond a (phi_1 = a)
    pre = [False, events[0]]
    pre[0] = pre[1]  # diamond a at t_1 = a(s_1) = False

    results = []
    for i in range(1, len(events)):
        now = [None, None]
        now[1] = events[i]
        now[0] = pre[0] or now[1]   # diamond recurrence
        results.append(now[0])
        pre = list(now)

    # After event 3 (a=True), diamond a becomes True and stays True
    assert results[1] == True, "diamond should be True after a=True at event 3"
    assert results[2] == True, "diamond F is monotone: once True stays True"


def test_strong_interval_gate_2_pattern():
    # Gate 2 from DETERMINISTIC-GATE RULES: [scope_widen, declassify)_s
    # Sequence: scope_widen=T then two egress rows then declassify=T
    # After scope_widen and before declassify, egress should trigger violation.
    events = [
        {"scope_widen": True,  "declassify": False, "egress": False},
        {"scope_widen": False, "declassify": False, "egress": True},   # VIOLATION expected
        {"scope_widen": False, "declassify": True,  "egress": False},  # interval closes
        {"scope_widen": False, "declassify": False, "egress": True},   # safe now
    ]
    # Subformulae: phi_3=scope_widen, phi_2=declassify, phi_1=[phi_3,phi_2)_s, phi_0=P
    def evaluate(events_list):
        violations = []
        s = events_list[0]
        pre = [None, None, None, None]
        pre[3] = s["scope_widen"]
        pre[2] = s["declassify"]
        pre[1] = pre[3] and not pre[2]    # strong-interval init
        pre[0] = not s["egress"] or not pre[1]

        for i in range(1, len(events_list)):
            s = events_list[i]
            now = [None, None, None, None]
            now[3] = s["scope_widen"]
            now[2] = s["declassify"]
            now[1] = (pre[1] or now[3]) and not now[2]   # strong-interval loop
            now[0] = not s["egress"] or not now[1]
            if not now[0]:
                violations.append(i)
            pre = list(now)
        return violations

    violations = evaluate(events)
    assert 1 in violations, "event index 1 (egress after scope_widen) must be a violation"
    assert 3 not in violations, "event index 3 (egress after declassify) must NOT be a violation"


def test_now_array_size_constant():
    # Section 5.1: space is O(m), independent of trace length n
    # Simulate monitoring 100 events; pre[] size must not grow
    formula_size = 5  # m = 5 subformulae (arbitrary)
    pre = [False] * (formula_size + 1)
    for _ in range(100):
        now = [False] * (formula_size + 1)  # same fixed size
        pre = list(now)
    assert len(pre) == formula_size + 1, "pre[] size must remain constant (O(m) space)"
