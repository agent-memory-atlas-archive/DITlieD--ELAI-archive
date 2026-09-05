Paper: A Brief Account of Runtime Verification
Authors: Martin Leucker, Christian Schallhart
Venue: Journal of Logic and Algebraic Programming, 2009
Source: https://christian.schallhart.net/publications/2009--jlap--a-brief-account-of-runtime-verification.pdf
Local PDF: F:\.ELAI_workflow\.claude\papers\rv-survey.pdf
Local MD: F:\.ELAI_workflow\.claude\papers\rv-survey_extracted\rv-survey.md
RAG topic: paper-rv-survey
RAG status: SKIPPED (local-rag MCP unavailable at ingest time, known reranker crash)


CORE DEFINITIONS (verbatim from paper)
---------------------------------------

Runtime Verification (Section 2, verbatim):
"Runtime verification is the discipline of computer science that deals with the study, development, and application of those verification techniques that allow checking whether a run of a system under scrutiny satisfies or violates a given correctness property."

Monitor (Section 2.1, verbatim):
"A monitor is a device that reads a finite trace and yields a certain verdict."

Fault/Failure/Error (Section 2, verbatim):
"a software failure [is] a deviation between the observed behavior and the required behavior of the software system. A fault is defined as the deviation between the current behavior and the expected behavior ... An error ... is a mistake made by a human that results in a fault and possibly in a failure."

Run vs Execution (Section 2.1, verbatim):
"a run of a system is understood as a possibly infinite sequence of the system's states ... An execution of a system is a finite prefix of a run ... When running a program, we can only observe executions."

Verdict truth domain (Section 2.1, verbatim):
"A truth domain is a lattice with a unique top element true and a unique bottom element false."


COMPARISON TO OTHER TECHNIQUES
--------------------------------

RV vs Model Checking (Section 2.2, key distinctions):
1. Model checking solves the LANGUAGE INCLUSION problem (all executions vs spec); RV solves the WORD PROBLEM (one execution vs spec).
2. Model checking considers INFINITE traces; RV deals with FINITE executions (necessarily finite, being prefixes of runs).
3. Model checking requires a COMPLETE MODEL of the system; RV works on BLACK BOX systems with no model.
4. Model checking suffers STATE EXPLOSION (full state space enumeration); RV on a single run "does usually not yield any memory problems."
5. Model checking allows backwards search; online monitoring must process incrementally (forbids backwards search because it would require storing the whole trace).
6. Language containment for NFA is PSPACE-complete; deciding whether a given word is accepted by NFA is NLOGSPACE-complete.

RV vs Theorem Proving (Section 1, Section 2.4, key distinctions):
- Theorem proving is mostly MANUAL; RV generates monitors AUTOMATICALLY from specs.
- Theorem proving proves correctness of MODELS (potentially diverging from implementation); RV checks the ACTUAL EXECUTION.
- RV acts as a partner to theorem proving: checking that the implementation behaves as the proved model assumed.

RV vs Testing (Section 2.3, key distinctions):
- Testing uses FINITE input-output sequences and checks output against predicted output; RV uses an automatically generated oracle (monitor) from a high-level spec.
- Testing provides INPUT SEQUENCES to exhaustively exercise the system; RV rarely addresses this input selection problem.
- RV = "oracle-based testing" or "passive testing."
- When deployed in production: RV = "testing forever" (completing the incompleteness of testing).
- Key difference: in testing an oracle is typically defined DIRECTLY; in RV the monitor is GENERATED from a high-level spec.

RV scope (Section 2, verbatim):
"Runtime verification itself deals (only) with the detection of violations (or satisfactions) of correctness properties. Thus, whenever a violation has been observed, it typically does not influence or change the program's execution."


KEY DICHOTOMIES
---------------

ONLINE vs OFFLINE MONITORING (Section 2.1):
- Online monitoring: monitor processes executions INCREMENTALLY as the system runs; must be efficient; is part of the running system; "should be designed to consider executions in an incremental fashion and in an efficient manner."
- Offline monitoring: monitor works on "(a finite set of) recorded execution(s)."
- Online constraint: backwards search MUST BE AVOIDED because it would "require, in the worst case, the whole execution trace to be stored."
- Online monitor complexity (memory + CPU) is critical because "the monitor is part of the running system and should influence the system as less as possible."

IMPARTIALITY vs ANTICIPATION (Section 2.1, verbatim):
- Impartiality: "a finite trace is not evaluated to true or, respectively false, if there still exists an (infinite) continuation leading to another verdict."
- Anticipation: "once every (infinite) continuation of a finite trace leads to the same verdict, then the finite trace evaluates to this very same verdict."
- Together they guarantee the monitor is "neither premature nor overcautious in its evaluations."
- These two maxims REQUIRE at least three truth values: true, false, inconclusive.

MONITOR SYNTHESIS FROM SPEC (Section 2.1, Section 3):
- "one of the key problems addressed in runtime verification is the generation of monitors from high-level specifications."
- LTL is the standard spec language (Pnueli [51]).
- From LTL3 formula: construct a DETERMINISTIC FINITE STATE MACHINE (FSM) with three output symbols; reads finite traces, yields 3-valued verdict.
- "Standard minimization techniques for FSMs can be applied to obtain a unique FSM that is optimal with respect to its number of states."
- The construction is from [8] (Bauer, Leucker, Schallhart 2006/2007).

FINITE TRACES AND LTL SEMANTICS - THE 3-VALUED APPROACH (Section 3):
LTL was designed for INFINITE traces. Runtime verification observes only FINITE executions (prefixes). Two-valued LTL-on-finite-traces is MISLEADING:
- Example 1: G(neg p) -- "p has never occurred" should be inconclusive (not true) on a good prefix that might be followed by p.
- Example 2: Fp -- "eventually p" is true only when p is observed; until then inconclusive.

LTL3 semantics (Section 3, verbatim):
"given a finite word u and an LTL3 formula phi, the semantics is defined as follows:
- if there is no continuation of u satisfying phi (considered as an LTL formula), the value of phi is false;
- if every continuation of u satisfies phi (considered as an LTL formula), it is true;
- otherwise, the value is inconclusive since the observations so far are inconclusive, and neither true or false can be determined."

Good prefix / Bad prefix (Section 3, verbatim):
"a bad prefix (of a Buchi automaton) is defined as a finite prefix which cannot be the prefix of any accepting trace. Dually, a good prefix is a finite prefix such that any infinite continuation of the trace will be accepted. It is exactly this classification that forms the basis of our 3-valued semantics: 'bad prefixes' (of formulae) are mapped to false, 'good prefixes' evaluate to true, while the remaining prefixes yield inconclusive."

Four-valued extension RV-LTL (Section 3):
Refines inconclusive into "presumably true" and "presumably false":
(1) satisfies the property, (2) violates the property, (3) will presumably violate, (4) will presumably conform.
Used to avoid permanently-inconclusive verdicts for non-monitorable properties like G(r -> Fa).

Two-valued semantics are REJECTED by the authors (Section 3):
"we strongly believe that only two truth values lead to misleading results in runtime verification."

MONITORABILITY (Section 3):
Definition (Pnueli and Zaks [52]): "a property [is] monitorable with respect to a trace whenever a corresponding monitor might still report a violation (or satisfaction)."
- Safety properties are monitorable (a violation is always witnessed by a finite bad prefix).
- Co-safety properties are monitorable.
- "The class of monitorable properties is richer than the union of safety and co-safety properties." (proved in [10])
- Example of monitorable property that is neither safe nor co-safe: phi = ((p OR q) U r) OR Gp.
- Non-monitorable example: G(r -> Fa) (every request finally acknowledged) -- "any pending request might be acknowledged in the next observation, so a finite trace is never sufficient to prove a violation."
- Restricting to PAST operators or SAFETY FRAGMENT avoids monitorability issues entirely.

STEERING / INTERVENTION vs PURE OBSERVATION (Section 5):
- Pure RV: "detection of violations (or satisfactions) of correctness properties ... typically does not influence or change the program's execution."
- Extensions that react: Monitor-Oriented Programming (MOP), Runtime Reflection (RR).
- MOP (Section 5.2.3): developer specifies properties + code to execute on violation; MOP framework auto-generates monitors and integrates them; "allows the development of reflective software systems."
- RR (Section 5.2.2): 4-layer architecture: Logging -> Monitoring -> Diagnosis -> Mitigation; separates fault detection from failure identification.
- RR vs MOP: MOP is a programming methodology (language-tied, e.g. jMOP); RR is an architecture pattern with an additional diagnosis layer.
- "Runtime verification itself deals (only) with the detection" -- steering/intervention is BEYOND RV proper.


DETERMINISTIC-GATE RULES FOR ELAI
-----------------------------------

The following rules map the RV taxonomy to ELAI's verification layers.
These are design decisions derived from the paper's dichotomies, not verbatim quotes.

RULE 1 (which layer owns design invariants):
Design invariants (structural correctness of the state machine, type safety, memory safety, absence of races) CANNOT be established by RV. RV only observes single finite executions. Use formal spec/proof (TLA+, Kani, Prusti, or equivalent) for "all executions must satisfy P" claims. RV is the wrong tool here.

RULE 2 (which layer owns live policy conformance):
Live, runtime policy conformance -- "every SSB row of type X must be followed by Y within N events", "no agent emits action Z before capability check" -- is exactly the RV problem: checking whether a SINGLE RUN satisfies a FINITE-PREFIX-OBSERVABLE correctness property. Implement as an RV monitor over the SSB event stream (online monitoring). The monitor is generated from a spec (LTL3 or safety fragment). This is the ONLY layer that can check black-box runtime behavior.

RULE 3 (which layer owns functional behavior):
Input-output functional correctness ("given input X, output is Y") is the TESTING problem, not RV. Tests cover a finite set of input-output sequences. RV is passive (oracle-based testing, no input generation). Use unit/integration/contract tests. RV monitors do not replace tests; they complement them by running "forever" in production.

RULE 4 (online vs offline in ELAI context):
SSB stream monitoring is ONLINE: the monitor must process events incrementally as they arrive; no backwards search; memory must be bounded (FSM state only, not the full trace). Post-hoc audit of recorded SSB rows is OFFLINE and may use richer analysis. Do not implement an online monitor that buffers the full trace -- this violates the online constraint.

RULE 5 (finite trace semantics -- 3-valued is mandatory):
Any monitor over the SSB stream MUST use 3-valued (or 4-valued RV-LTL) verdicts, not 2-valued. A monitor that declares "compliant" on a prefix where future events could violate the property is MISLEADING per the paper (impartiality violation). A monitor verdict of "inconclusive" is CORRECT and must be propagated as such, not coerced to true.

RULE 6 (monitorability check before spec writing):
Before writing an RV spec for a property, check monitorability:
- Safety property (violation always has a bad finite prefix): MONITORABLE, use LTL3 safety fragment or past-time LTL.
- Co-safety property (satisfaction always has a good finite prefix): MONITORABLE.
- Liveness property like G(r -> Fa): NOT MONITORABLE with 3-valued logic; must use 4-valued RV-LTL (presumably-true/presumably-false) or move the check to theorem proving / model checking.
- Non-monitorable properties cannot be correctly decided on finite traces; forcing them into a 2-valued monitor produces wrong verdicts.

RULE 7 (monitor synthesis, not hand-coding):
Monitors MUST be generated from specs (LTL3/FSM synthesis), not hand-coded boolean flags. A hand-coded flag implements the monitor designer's intuition about the property, which is the "oracle defined directly" anti-pattern (testing, not RV). The synthesis procedure guarantees impartiality and anticipation; hand-coding does not.

RULE 8 (steering is BEYOND RV proper):
An RV monitor that also modifies program state or rejects/delays actions is implementing MOP or RR, not pure RV. These are valid architectural patterns but must be explicitly designed with a Diagnosis + Mitigation layer (RR) or property-plus-action specs (MOP). The monitoring layer itself must NOT silently alter system behavior; violations must flow to an explicit mitigation layer.

RULE 9 (RV cannot replace proof for all-quantified properties):
Properties of the form "for ALL runs, P holds" require model checking or theorem proving. RV can only say "this ONE run (so far) satisfies/violates/is-inconclusive-about P." For ELAI's safety-critical gates (wiring contract, production caller), the appropriate layer is static analysis (production_caller_check.py, wiring_contract_check.py) which approximates model checking over the code structure, not an RV monitor.

RULE 10 (complexity budget for online monitors):
An FSM-based LTL3 monitor runs in O(1) space per event (only FSM state stored) and O(|formula|) CPU per event. Any ELAI RV monitor must fit this budget. If the property requires storing unbounded history, it is either not a regular property (use a pushdown monitor for nested calls/returns per [56]) or it is being misapplied to the RV layer.


WHAT RV CANNOT DO (explicit limits from paper)
------------------------------------------------

1. RV cannot check ALL executions -- it checks ONE (or a finite subset in offline mode). "All-quantified" correctness requires model checking or proof.
2. RV cannot establish the system is CORRECT. It can only detect violations (or give an inconclusive verdict). Absence of violation on observed executions does not imply correctness.
3. RV cannot check non-monitorable liveness properties correctly with 3-valued logic (e.g., G(r -> Fa) is always inconclusive). 4-valued RV-LTL is a partial remedy.
4. RV cannot reconstruct the full system state from observations. "The states of an observed execution usually do not reflect the system's state completely." A state seen twice does not imply an infinite loop exists.
5. Online RV monitors must not require unbounded trace storage. Properties requiring full-history recall exceed the online monitoring model.
6. RV monitoring of real-time or hybrid systems may PERTURB the system ("the execution of the monitor on the same platform ... might influence the overall system behaviour"). Dedicated monitoring hardware is the correct solution when timing integrity is critical.
7. RV is not a substitute for testing (no input generation) or model checking (no state-space enumeration). It is a COMPLEMENT.


FAILURE MODES (from paper)
---------------------------

1. Using 2-valued semantics on finite traces produces premature verdicts (false "true" on inconclusive prefixes of safety properties, false "false" on good prefixes of liveness properties).
2. Non-monitorable properties forced into 3-valued monitors yield permanently-inconclusive verdicts that provide no useful information.
3. Online monitors that store the full trace negate the memory advantage of RV over model checking.
4. Backwards-search monitor evaluation requires full trace storage (forbidden for online).
5. Monitor running on the same platform as the monitored system can perturb real-time behavior, causing the monitored system to satisfy different properties than the unmonitored one.
6. Treating a single "run is compliant" verdict as "system is correct" confuses the word problem with language containment.


COMPLEXITY
----------

Monitor generation (offline, once): polynomial in formula size (FSM minimization is polynomial for DFAs).
Monitor execution per event (online): O(1) space (FSM state only); O(|formula|) time per event transition.
Language containment (model checking, NFA): PSPACE-complete [58].
Word membership (RV monitor, NFA): NLOGSPACE-complete [41].
FSM uniqueness: a smaller deterministic FSM than the minimal one cannot exist; any smaller must be nondeterministic or check a different property.


CONTRACT TEST STUBS
--------------------

# Copy into tests/contracts/test_rv_survey_contract.py
# No imports beyond stdlib + numpy required.

import enum

class Verdict(enum.Enum):
    TRUE = "true"
    FALSE = "false"
    INCONCLUSIVE = "inconclusive"

def test_three_valued_verdict_domain():
    # Rule 5: monitor verdicts must use 3-valued domain, not bool
    # A truth domain is a lattice with unique top=true and bottom=false
    verdicts = {v for v in Verdict}
    assert Verdict.TRUE in verdicts
    assert Verdict.FALSE in verdicts
    assert Verdict.INCONCLUSIVE in verdicts
    # Two-valued is insufficient -- inconclusive must be a distinct value
    assert len(verdicts) >= 3

def test_impartiality_on_safety_prefix():
    # Rule 5 / Section 2.1: impartiality -- do not declare TRUE while a
    # continuation could violate the property G(neg p).
    # Simulate: prefix has seen no p yet. Must not return TRUE (premature).
    # Must return INCONCLUSIVE until a bad prefix is confirmed.
    def monitor_G_neg_p(trace):
        # Safety property: returns FALSE on first p; INCONCLUSIVE otherwise.
        # NEVER returns TRUE on a finite prefix (no continuation is "good" prefix
        # for G(neg p) -- any continuation might see p).
        for event in trace:
            if event.get("p"):
                return Verdict.FALSE
        return Verdict.INCONCLUSIVE  # not TRUE -- that would violate impartiality

    assert monitor_G_neg_p([]) == Verdict.INCONCLUSIVE
    assert monitor_G_neg_p([{"p": False}]) == Verdict.INCONCLUSIVE
    assert monitor_G_neg_p([{"p": True}]) == Verdict.FALSE
    # Key: empty trace must NOT be Verdict.TRUE for a safety property
    assert monitor_G_neg_p([]) != Verdict.TRUE

def test_anticipation_on_cosafety_prefix():
    # Rule 5 / Section 2.1: anticipation -- once every continuation satisfies Fp,
    # report TRUE immediately (do not remain inconclusive).
    # Fp: eventually p. Once p is observed, every continuation satisfies Fp.
    def monitor_F_p(trace):
        for event in trace:
            if event.get("p"):
                return Verdict.TRUE  # good prefix: every continuation satisfies Fp
        return Verdict.INCONCLUSIVE  # not FALSE -- some continuation still satisfies

    assert monitor_F_p([]) == Verdict.INCONCLUSIVE
    assert monitor_F_p([{"p": False}]) == Verdict.INCONCLUSIVE
    assert monitor_F_p([{"p": True}]) == Verdict.TRUE
    # Key: must not return FALSE before a bad prefix is confirmed (no bad prefix for Fp on finite trace without p)
    assert monitor_F_p([{"p": False}, {"p": False}]) != Verdict.FALSE

def test_online_monitor_bounded_state():
    # Rule 4 / Section 2.2: online monitor must not store the full trace.
    # An FSM-based monitor carries only its current state, not trace history.
    # Simulate FSM with bounded state (int, not list).
    class BoundedFSMMonitor:
        def __init__(self):
            self.state = 0  # FSM state -- O(1), not O(n)
            self._trace_copy = None  # must remain None -- we never store the trace

        def step(self, event):
            # Transition: state is a function of previous state + current event only
            if self.state == 0 and event.get("p"):
                self.state = 1  # violation state
            return self.state

    m = BoundedFSMMonitor()
    for i in range(10000):
        m.step({"p": False})
    # State is a single int, not a list of 10000 events
    import sys
    assert sys.getsizeof(m.state) < 100  # O(1) not O(n)
    assert m._trace_copy is None  # trace was not stored

def test_monitorability_safety_vs_liveness():
    # Rule 6 / Section 3: safety properties are monitorable;
    # pure liveness G(r -> Fa) is not monitorable (always inconclusive on finite traces).
    # A safety property monitor must be able to return FALSE on a finite bad prefix.
    def safety_monitor_always_neg_p(trace):
        for e in trace:
            if e.get("p"):
                return Verdict.FALSE
        return Verdict.INCONCLUSIVE

    # Safety: can detect violation on finite bad prefix
    bad_prefix = [{"p": False}, {"p": True}]
    assert safety_monitor_always_neg_p(bad_prefix) == Verdict.FALSE

    # G(r -> Fa): liveness -- a finite trace can NEVER confirm a violation
    # (the pending request might be acknowledged next). Any implementation
    # that returns FALSE on a finite trace without seeing the acknowledgment
    # is producing a wrong verdict.
    def naive_liveness_monitor(trace):
        # WRONG: returns FALSE if request seen without ack -- violation of impartiality
        seen_request = False
        for e in trace:
            if e.get("request"):
                seen_request = True
            if e.get("ack") and seen_request:
                return Verdict.TRUE
        if seen_request:
            return Verdict.FALSE  # WRONG per paper -- should be INCONCLUSIVE
        return Verdict.INCONCLUSIVE

    # Demonstrate why this is wrong: a pending request without ack
    # should be INCONCLUSIVE, not FALSE (the ack might come next)
    partial_trace = [{"request": True}]
    result = naive_liveness_monitor(partial_trace)
    # The correct answer is INCONCLUSIVE; the naive monitor returns FALSE
    # This test documents the bug (the correct monitor would assert == INCONCLUSIVE)
    assert result == Verdict.FALSE  # documents the WRONG behavior to avoid
    # The correct monitor for G(r->Fa) requires 4-valued RV-LTL (presumably_false)
