Algorithm: Edit Automata (and the enforcement hierarchy: truncation, suppression, insertion, edit)
Source: Author-hosted free PDF (not arXiv)
Paper: Ligatti, Bauer, Walker. "Edit Automata: Enforcement Mechanisms for Run-time Security Policies." International Journal of Information Security, Vol 4, No 1-2, February 2005. DOI:10.1007/s10207-004-0046-8.
Free PDF: https://www.cs.princeton.edu/~dpw/papers/ijis03.pdf
Princeton tech report: https://www.cs.princeton.edu/research/techreps/375 (TR-681-03 = Princeton 2003 preprint)
Local PDF: F:\.ELAI_workflow\workflow\papers\edit-automata-ijis03.pdf
Local text: F:\.ELAI_workflow\workflow\papers\edit-automata-ijis03-text.txt


FRAMEWORK (Sections 2.1-2.3, verbatim from paper)

System: S = (A, Sigma) where A is the set of program actions (events) and Sigma is the set of possible executions. An execution sigma is a finite sequence of actions a1, a2, ..., an.

Security automaton: A = (Q, q0, T) where Q is the state set, q0 is the initial state, T is the transition function (type differs per automaton class). Runs in parallel with the target program, examines actions one at a time, transforms the sequence when it deviates from policy.

Single-step judgment: (sigma, q) --[tau]--> A (sigma', q') where sigma is remaining input, q is current state, sigma' and q' are post-step, tau is the produced output (observable). Multi-step: (sigma, q) ==[tau]==> A (sigma', q').

Policy: predicate P on sets of executions. Property: P(Sigma) = for-all sigma in Sigma. P_hat(sigma), where P_hat is computable.

Safety property: once a bad prefix occurs, no extension can remedy it. Formally: for-all sigma in pre(Sigma). not-P_hat(sigma) => for-all sigma' in Sigma. (sigma <= sigma' => not-P_hat(sigma')).

Liveness property: any finite prefix can be extended to a legal execution.

Enforcement has two principles (Section 2.4):
  Soundness: all observable outputs must obey the property.
  Transparency: enforcement must preserve semantics of executions that already obey the property.

Three enforcement strengths (Definitions 1-3):
  Conservative: automaton's output always obeys P_hat (no transparency constraint).
  Precise: conservative + passes valid executions through in lockstep (action-by-action, no delay).
  Effective: conservative + for valid inputs, output is semantically equivalent to input (sigma ~= sigma').


DETERMINISTIC-GATE RULES
(All definitions and theorems quoted from or closely paraphrased from the paper; section and theorem numbers are exact.)

--- CLASS 1: TRUNCATION AUTOMATON (Section 3) ---

Formal definition (Section 3.1):
  "A truncation automaton T is a finite or inﬁnite state machine (Q, q0, delta) that is deﬁned with respect to some system (A, Sigma). Q speciﬁes the possible automaton states and q0 is the initial state. The partial function delta : A x Q -> Q speciﬁes the transition function for the automaton that indicates that the automaton should accept the current input action and move to a new state. On inputs for which delta is not deﬁned, the automaton halts the target program."

Operational semantics (Section 3.1):
  T-Step: (sigma, q) --[a]--> T (sigma', q')   if sigma = a;sigma' AND delta(a, q) = q'
  T-Stop: (sigma, q) --[.]--> T (., q)          otherwise (delta undefined: halt and output nothing)

What it can do:
  - Accept an action (pass it through, advance state)
  - Halt the target program (truncate the action stream, output nothing further)
  - CANNOT: suppress individual actions without halting; CANNOT: insert actions

Policy classes enforceable (Theorems 1-3):
  Theorem 1 (Uniform Precise T-Enforcement): On uniform systems (Sigma = A*), truncation automata precisely enforce EXACTLY the safety properties: the predicate for-all sigma in pre(Sigma). not-P_hat(sigma) => for-all sigma' in Sigma. (sigma <= sigma' => not-P_hat(sigma')).
  Theorem 3 (Effective T-Enforcement): A property can be effectively enforced by truncation automaton iff there exists a computable truncation-rewrite function alpha* such that (1) bad inputs are mapped to property-satisfying outputs, (2) good inputs map to equivalent outputs.

ELAI mapping: truncation automaton = current BLOCK gate. When the monitor sees a forbidden action, it halts the agent (kills the tool call, raises an exception, kills the task). The agent's action stream is truncated. Output: enforcement stops. This is the WEAKEST class; it can only enforce safety properties on uniform systems.

Truncation-rewrite function (Section 3.3): alpha* : A* -> A* such that (1) alpha*(empty) = empty, (2) for all sigma in A*, for all a in A: alpha*(sigma;a) = alpha*(sigma);a (accept) OR alpha*(sigma;a) = alpha*(sigma) AND H(alpha*, sigma;a) where H captures halting behavior. A truncation automaton can only keep or discard; it cannot reorder or inject.

--- CLASS 2: SUPPRESSION AUTOMATON (Section 4) ---

Formal definition (Section 4.1):
  "A suppression automaton S is a finite or infinite state machine (Q, q0, delta, omega) that is defined with respect to some system (A, Sigma). [...] The partial function omega : A x Q -> {-, +} has the same domain as delta and indicates whether or not the action in question is to be suppressed (-) or emitted (+)."

Operational semantics (Section 4.1):
  S-StepA: (sigma, q) --[a]--> S (sigma', q')  if sigma=a;sigma' AND delta(a,q)=q' AND omega(a,q)=+  [accept/emit]
  S-StepS: (sigma, q) --[.]--> S (sigma', q')  if sigma=a;sigma' AND delta(a,q)=q' AND omega(a,q)=-  [suppress silently]
  S-Stop:  (sigma, q) --[.]--> S (., q)         otherwise [halt]

What it can do:
  - Accept an action (emit it, advance state)
  - Suppress an action (drop it silently, advance state, DO NOT halt)
  - Halt the target (truncation)
  - CANNOT: insert new actions not present in the input

Policy classes enforceable (Theorems 4-6):
  Theorem 4 (Uniform Precise S-Enforcement): On uniform systems, suppression automata precisely enforce EXACTLY the same set as truncation automata: the safety properties. (Same theorem statement as Theorem 1.)
  Theorem 5 (Nonuniform Precise S-Enforcement): On nonuniform systems, suppression automata can precisely enforce properties NOT enforceable by truncation automata. A property P_hat on (A, Sigma) can be precisely enforced by a suppression automaton iff there exists a computable suppression-rewrite function omega* such that for all sigma in Sigma: (1) not-P_hat(sigma) => P_hat(omega*(sigma)), (2) P_hat(sigma) => omega*(sigma) = sigma.
  Theorem 6 (Effective S-Enforcement): On any system, suppression automata can effectively enforce a proper superset of the properties effectively enforceable by truncation automata.

Suppression-rewrite function (Section 4.2): omega* : A* -> A* such that (1) omega*(empty)=empty, (2) for all sigma, a: omega*(sigma;a) = omega*(sigma);a [accept] OR omega*(sigma;a) = omega*(sigma) [suppress]. Cannot insert; can only keep or drop each input action.

Key distinguishing example (Section 4.3, authenticated-login): A suppression automaton can suppress a ulogin (unauthenticated login) and let the agent continue. If alogin follows, the stream is semantically equivalent to alogin alone. A truncation automaton cannot do this: it would have to halt on ulogin, which is wrong if the next action is alogin.

ELAI mapping: suppression automaton = "drop the dangerous tool-call, let the agent continue." Example: the agent issues a file-write to /etc/passwd; the monitor suppresses that single tool-call (drops it from the output stream), transitions state, and lets the agent proceed to the next action. The session does not terminate. Useful when the forbidden action is unnecessary and dropping it leaves the sequence semantically valid.

--- CLASS 3: INSERTION AUTOMATON (Section 5) ---

Formal definition (Section 5.1):
  "An insertion automaton I is a finite or infinite state machine (Q, q0, delta, gamma) that is defined with respect to some system (A, Sigma). The partial function delta : A x Q -> Q specifies the transition function as before. The new element is a partial function gamma that specifies the insertion of a finite sequence of actions into the program's action sequence. We call this the insertion function and it has type A x Q -> A+ x Q, where the first component in the returned pair indicates the finite and non-empty sequence of actions to be inserted. In order to maintain the determinacy of the automaton, we require the domain of the insertion function to be disjoint from the domain of the transition function."

Operational semantics (Section 5.1):
  I-Step: (sigma, q) --[a]--> I (sigma', q')   if sigma=a;sigma' AND delta(a,q)=q'             [accept]
  I-Ins:  (sigma, q) --[tau]--> I (sigma, q')   if sigma=a;sigma' AND gamma(a,q)=(tau, q')      [insert tau before consuming a; DO NOT consume a yet]
  I-Stop: (sigma, q) --[.]--> I (., q)           otherwise [halt]

What it can do:
  - Accept an action (emit it, advance state)
  - Insert a finite non-empty sequence of actions BEFORE the current action (without consuming it)
  - Halt the target
  - CANNOT: suppress (drop) individual actions from the input stream

Policy classes enforceable (Theorems 7-10):
  Theorem 7 (Uniform Precise I-Enforcement): On uniform systems, insertion automata precisely enforce EXACTLY the safety properties (same as truncation and suppression).
  Theorem 8 (Nonuniform Precise I-Enforcement): On nonuniform systems, insertion automata can precisely enforce a superset of precisely-suppression-enforceable properties. Theorem 9 shows the precise suppression-enforceable set is a SUBSET of precise insertion-enforceable set (insertion strictly more powerful than suppression as precise enforcer).
  Theorem 10 (Effective I-Enforcement): A property can be effectively enforced by an insertion automaton iff there exists a computable insertion-rewrite function gamma* such that (1) bad inputs map to policy-satisfying outputs, (2) good inputs map to equivalent outputs (sigma ~= gamma*(sigma)).

Insertion-rewrite function (Section 5.3): gamma* : A* -> A* such that (1) gamma*(empty)=empty, (2) for all sigma, a, there exists tau in A*: gamma*(sigma;a) = gamma*(sigma);tau;a [insert tau then accept a] OR gamma*(sigma;a) = gamma*(sigma);tau AND H(gamma*, sigma;a) [insert tau then halt].

Key distinguishing example (Sections 5.2, 5.3 cable-car policy): When a passenger boards without showing a ticket, an insertion automaton inserts a "show_driver" action before accepting the "board" action. If the sequence turns out to already contain a ticket-show, no harm is done (semantically equivalent). A suppression automaton cannot handle this (cannot insert the required show action). A truncation automaton cannot handle it (halting drops the valid board+show sequence).

ELAI mapping: insertion automaton = "inject a mandatory approval/declassification step before a side effect." Example: before a network-exfiltration action, the monitor inserts a "request_human_approval" action into the event stream; the original action is not consumed until that inserted step has occurred. Useful for mandatory pre-conditions (consent gates, declassification steps, audit logging that must precede the side effect).

--- CLASS 4: EDIT AUTOMATON (Section 6) ---

Formal definition (Section 6.1):
  "We form an edit automaton E by combining the insertion automaton with the suppression automaton. Our machine is now described by a 5-tuple of the form (Q, q0, delta, gamma, omega). The operational semantics is derived from the composition of the operational rules of the two previous automata. Again, the partial functions delta and omega have the same domain while delta and gamma have disjoint domains."

Operational semantics (Section 6.1):
  E-StepA: (sigma, q) --[a]--> E (sigma', q')    if sigma=a;sigma' AND delta(a,q)=q' AND omega(a,q)=+   [accept/emit]
  E-StepS: (sigma, q) --[.]--> E (sigma', q')    if sigma=a;sigma' AND delta(a,q)=q' AND omega(a,q)=-   [suppress]
  E-Ins:   (sigma, q) --[tau]--> E (sigma, q')   if sigma=a;sigma' AND gamma(a,q)=(tau, q')              [insert tau, do not consume a]
  E-Stop:  (sigma, q) --[.]--> E (., q)           otherwise [halt]

What it can do:
  - Accept an action
  - Suppress an action (drop without halting)
  - Insert a sequence of actions (before consuming current action)
  - Halt (truncate)
  - ALL FOUR operations combined at will, per-state, per-action

Policy classes enforceable (Theorems 11-13):
  Theorem 11 (Uniform Precise E-Enforcement): On uniform systems, edit automata precisely enforce EXACTLY the safety properties (same as all previous types).
  Theorem 12 (Nonuniform Precise E-Enforcement): On nonuniform systems, edit automata precisely enforce exactly those properties precisely enforceable by insertion automata (Theorem 8). The composed power does not exceed insertion alone for precise enforcement.
  Theorem 13 (Effective E-Enforcement): "Any property P_hat on the system S = (A, Sigma) can be effectively enforced by some edit automaton." [Quoted verbatim from paper, Section 6.3.] This is the maximal result: edit automata are universal effective enforcers.

Proof technique for Theorem 13 (Section 6.3, verbatim):
  "Edit automata are very powerful effective enforcers because they can insert actions that are required and later suppress those same actions if they appear in the original input sequence. Conversely, an edit automaton can suppress a sequence of potentially illegal actions, and if the sequence is later determined to be legal, just re-insert it. Interestingly, with this technique edit automata can effectively enforce any property -- the automaton simply suppresses all actions until it can confirm that the current prefix obeys the property, at which point it inserts all the suppressed actions. Any legal input will thus be output without modification. If an input is illegal, an edit automaton will output its longest valid prefix."

Key distinguishing example (Section 6.4, market transactions / ACID):
  "The first property, atomicity, can be modeled using an edit automaton by creating an intentions log. That is, the automaton suppresses input actions from the start of the transaction and, if the transaction completes successfully, the entire sequence of actions is emitted atomically to the output stream; otherwise it is discarded."
  Neither insertion nor suppression alone can enforce this: suppression cannot insert; insertion cannot suppress the speculative actions.

ELAI mapping: edit automaton = "rewrite the event stream (redact a secret from output, quarantine a write, buffer and replay, delay-then-commit)." Examples: (a) suppress agent's output until a validator confirms it contains no PII, then re-emit; (b) buffer all file-writes for a transaction, suppress them individually, then either flush-all on commit or discard-all on abort; (c) redact a secret value from a tool-call result before it enters the agent's context (suppress original, insert sanitized version). Edit automata subsume all other gate types.


ENFORCEMENT HIERARCHY (Section 8 / Figures 3-4)

On uniform systems, for PRECISE enforcement, all four types are equivalent: they enforce exactly the safety properties.

On nonuniform systems, for PRECISE enforcement (Figure 3):
  truncation-enforceable SUBSET-OF suppression-enforceable SUBSET-OF insertion-enforceable = edit-enforceable

For EFFECTIVE enforcement on any system (Figure 4):
  truncation-effective PROPER-SUBSET suppression-effective (neither is a subset of the other vs insertion)
  insertion-effective PROPER-SUBSET edit-effective
  suppression-effective PROPER-SUBSET edit-effective
  edit-effective = ALL properties (Theorem 13)

The hierarchy is STRICT: there exist properties in each superset that cannot be enforced by the weaker mechanism (examples given in Sections 4.3, 5.3, 6.4).

Key asymmetry: suppression and insertion are INCOMPARABLE as effective enforcers. There are properties effectively enforceable by suppression but not insertion (authenticated-login example, Section 4.3) and vice versa (cable-car example, Section 5.3). Edit automata subsume both.


Inputs:
action_sequence | sequence of program events | A* (finite, arbitrarily long) | must be observable by monitor
automaton_state | Q | initial = q0 | finite or infinite state machine
system | (A, Sigma) | uniform (Sigma=A*) or nonuniform | constrains enforceable policy class

Outputs:
output_sequence | tau | A* | the (possibly transformed) action stream seen by the outside world
final_state | q' | Q | post-execution state
enforcement_verdict | implicit in output | sound iff output satisfies P_hat; transparent iff P_hat(sigma) => sigma ~= tau

Hyperparameters:
enforcement_mode | effective | {conservative, precise, effective} | choice of enforcement criterion changes power; effective is standard
system_type | uniform | {uniform, nonuniform} | nonuniform expands enforceable set for truncation and suppression; edit always universal under effective
equivalence_relation | strict equality | any reflexive/symmetric/transitive relation satisfying Equivalence condition | looser ~= expands what counts as "transparent"

Critical invariants (testable assertions):
assert all(P_hat(tau) for tau in outputs), "Soundness: every observable output must satisfy the policy"
assert not (P_hat(sigma) and not semantically_equivalent(sigma, tau)), "Transparency: valid inputs pass through equivalent"
assert truncation_rewrite(sigma) == sigma or is_prefix_truncation(truncation_rewrite(sigma), sigma), "Truncation only keeps or drops; never inserts"
assert all(a in sigma for a in suppression_rewrite(sigma)), "Suppression only drops; output is subsequence of input"
assert is_subsequence(input_actions, filter_inserted(insertion_output)), "Insertion output contains original actions in order (plus insertions)"
assert edit_automaton_can_enforce_any_property(), "Theorem 13: effective edit enforcement is universal"

Complexity:
time | O(n) per action in the stream (one transition step per event) | space | O(|Q|) for state; unbounded for buffering in edit atomicity example
parallelism | runs in parallel with target program; each event is synchronously intercepted before it reaches the system

Failure modes (from Section 2.5):
policy depends on relationships between executions (e.g. information flow) -- not a property in their sense; monitors cannot enforce it
monitor cannot observe certain security-relevant actions (hardware direct access, out-of-band channels)
monitor cannot synthesize required data for insertion (e.g. cannot forge a valid signed token to insert)
monitor is compromised by the untrusted program (requires isolation via safe language or OS support)
real-time constraints: suppression/insertion may not complete within required time window

Implementation pitfalls (do NOT do this):
do NOT model a suppression gate as a truncation: dropping an action must not also halt the agent; those are distinct transitions
do NOT use insertion as if it were suppression: insertion cannot drop the triggering action; the current action remains in the queue
do NOT assume the edit automaton reads the entire future before deciding: it is an online transducer reading left-to-right; buffering is the mechanism for lookahead, not prescience
do NOT conflate "precise enforcement" with "effective enforcement": a truncation automaton that passes valid inputs lockstep but cannot suppress bad ones is precise, not effective when suppression is needed
do NOT build a suppression gate that also inserts (it becomes an edit gate; the formal classes are distinct and the policy power hierarchy is strict)
on uniform systems do NOT expect suppression or insertion to buy you more precise-enforcement power over truncation: all three precisely enforce exactly the safety properties on uniform systems (Theorems 1, 4, 7)
