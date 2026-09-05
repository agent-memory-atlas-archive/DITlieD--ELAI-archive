Algorithm: Enforceable Security Policies via Execution Monitoring (EM)
Source: ACM TISSEC 3(1), February 2000
Paper: "Enforceable Security Policies"
Author: Fred B. Schneider, Cornell University
Local PDF: F:\.ELAI_workflow\.claude\papers\enf-sec-pols.pdf
Local extracted MD: F:\.ELAI_workflow\.claude\papers\enf-sec-pols_extracted\enf-sec-pols.md

=== FORMAL SETUP ===

Universe C: the set of all finite and infinite sequences (execution traces).
Target S defines subset Sigma_S of C (the executions S can produce).
Security policy P: a predicate on SETS of executions. S satisfies P iff P(Sigma_S) = true.
P_hat: a predicate on individual executions, used when P has the form P(Pi) = (forall sigma in Pi: P_hat(sigma)).

=== INPUTS (to a security automaton / EM gate) ===

execution_trace | finite-or-infinite sequence | elements of I (input symbol set) | must be observable steps of the target; may be atomic actions, higher-level system steps, program states, or state/action pairs
input_symbol | element of I | one step of the target execution | dictated by the security policy and how executions are represented
automaton_state | element of Q | countable set | must be encodable in bounded memory for practical enforcement
initial_state_set | subset Q0 of Q | countable | provided at automaton construction time

=== OUTPUTS ===

verdict | ACCEPT or REJECT | per finite prefix of the trace | REJECT = terminate the target immediately (truncation); ACCEPT = allow target to continue
current_Q_prime | subset of Q | nondeterministic current state set | invariant: if Q_prime becomes empty set, input is rejected
enforcement_action | halt / allow | binary | halt fires before the bad step executes, never after

=== SECURITY AUTOMATON FORMAL DEFINITION (Section 3) ===

A security automaton is a tuple (Q, Q0, I, delta) where:
  Q: countable set of automaton states
  Q0: countable subset of Q, initial states
  I: countable set of input symbols
  delta: (Q x I) -> 2^Q, nondeterministic transition function

Processing: start with Q' = Q0; for each input symbol s_i, set Q' = union_{q in Q'} delta(q, s_i).
Rejection criterion: Q' = empty set => input rejected.
Acceptance: any other Q' (including infinite-length inputs) => accepted.

Guarded-command notation (Section 3, Figure 2 style):
  state_vars: typed variables encoding current Q'
  transitions: list of guarded commands of form "B --> S" where B is a predicate on (current input symbol, current automaton state) and S updates only the automaton state variables.

=== THREE CONDITIONS FOR EM ENFORCEABILITY (Section 2, equations 1-3) ===

Condition 1 - Property form (eq. 1):
  P(Pi) = (forall sigma in Pi: P_hat(sigma))
  The policy must be definable by a per-execution predicate. Policies that require comparing across executions (like information flow) fail this condition.

Condition 2 - Prefix closure (eq. 2):
  (forall tau' in Pi^-: NOT P_hat(tau')) => (forall sigma in Pi: NOT P_hat(tau' sigma))
  If a finite prefix is bad, no extension of that prefix can be good. An EM mechanism cannot "un-reject" a prefix because future steps might redeem it.

Condition 3 - Finite rejection (eq. 3):
  (forall sigma in Pi: NOT P_hat(sigma)) => (exists i: NOT P_hat(sigma[..i]))
  Every execution that violates the policy must have a finite prefix that already witnesses the violation. Violations that only manifest at limit/infinity cannot be caught.

A property G (set of executions) is a SAFETY PROPERTY (Lamport 1985, eq. 4) if and only if:
  sigma NOT IN G => (exists i: (forall tau in Pi: sigma[..i] tau NOT IN G))
  i.e., G is a safety property iff every excluded execution has a finite bad prefix such that all extensions of that prefix are also excluded.

MAIN RESULT (Section 2):
  "If the set of executions for a security policy P is not a safety property, then an enforcement mechanism from EM does not exist for P."
  Contrapositive: EM mechanisms can enforce safety properties.
  NOTE: the converse does NOT hold (Section 4, Target Control paragraph). Not all safety properties have EM enforcement mechanisms.

=== LIMITS: WHAT EM CANNOT ENFORCE (explicit paper statements) ===

LIMIT-1 Non-property policies (Section 2):
  Information flow policies do NOT define sets that are properties (McLean 1994). Whether information flows from x to y in a given execution depends on values y takes in OTHER executions. No per-execution predicate P_hat can capture this.
  => NO EM GATE EXISTS for exact information flow. Must use admission-time / compile-time / type-system enforcement.

LIMIT-2 Liveness properties (Section 2):
  Availability defined as "no principal is FOREVER denied use of some resource" is NOT a safety property. Any partial execution can be extended to allow the principal access, so no defining set of proscribed finite prefixes exists.
  => NO EM GATE EXISTS for pure liveness ("something good eventually happens"). Must move to plan time or architectural guarantees, not runtime monitoring.

LIMIT-3 Real-time / wall-clock availability (Section 4, Target Control):
  Real-Time Availability: "One principal cannot be denied use of a resource for more than MWT SECONDS." This IS a safety property (the bad thing is an interval exceeding MWT seconds), but the target cannot be stopped from producing input symbols as time passes. The enforcement mechanism cannot halt time.
  => CANNOT be enforced by runtime EM even though it is a safety property. Fix: replace "MWT seconds" with "MWT execution steps" - then the target CAN be halted.

LIMIT-4 Execution-count bounded availability IS enforceable:
  Real-Time Availability with "MWT execution steps" instead of "MWT seconds" => enforceable because halting the target stops execution-step symbols from being produced.

LIMIT-5 Guaranteed service / fairness (Section 3, Figure 4 note):
  "We cannot specify the stronger security policy (that service be guaranteed after payment) because that is not a safety property - there is no defining set of proscribed partial executions."
  A service provider is allowed to STOP EXECUTING rather than render service. The EM gate can only prevent the provider from doing BAD THINGS after payment, not force it to do GOOD THINGS.
  => Liveness obligations (fair scheduling, guaranteed delivery, eventual response) are NOT enforceable by EM.

LIMIT-6 Cross-execution / relational policies:
  Any policy whose truth depends on correlations ACROSS multiple executions (information flow, noninterference, hyperproperties) cannot be a property in the Alpern-Schneider sense, hence not EM-enforceable.

=== WHAT EM CAN ENFORCE (explicit paper statements) ===

CAN-1 Access control (Section 2): "Access control defines safety properties. The set of proscribed partial executions contains those partial executions ending with an unacceptable operation being attempted."

CAN-2 History-based policies: "No Send after FileRead" (Figure 1/2) - enforceable because the bad prefix (FileRead then Send) is finite and detectable.

CAN-3 Execution-step-bounded availability: "Change the specification from MWT seconds to MWT execution steps and the target can be prevented from violating the policy by stopping execution, resulting in an EM-enforceable security policy." (Section 4)

CAN-4 Rights-management / DRM-style policies: proscribing specific operation sequences on objects.

CAN-5 Fair-exchange partial obligation: "the service provider be prevented from engaging in actions other than delivering service for which a customer has paid" (the seller-cannot-do-other-things half). Enforceable. The buyer-must-receive-service half is NOT.

CAN-6 Conjunction of EM-enforceable policies: "When multiple EM mechanisms are used in tandem, the policy enforced by the aggregate is the conjunction of the policies that are enforced by each mechanism in isolation." (Section 2) Composition is safe for EM mechanisms.

=== HYPERPARAMETERS / DESIGN CHOICES ===

Q_encoding | automaton state representation | any countable set; guarded-command variable encoding preferred for complex policies | in practice must be finite memory; infinite state needed only for policies that must remember an unbounded accumulation (sum of all past values)
I_granularity | input symbol = what level of target step | atomic memory access / system call / method invocation / state/action pair | coarser granularity = cheaper but less precise
termination_vs_notification | halt vs notify | notation: EM defined to terminate; paper proves that adding notify-then-continue does NOT expand the enforceable class (Section 4, Response to Violations)
inlining_vs_external | SFI/SASI inlined vs external monitor | inlining reduces overhead; expands trust to the transformation tool | both enforce identical class of policies; EM defined to exclude target modification but the enforceable SET is the same

=== CRITICAL INVARIANTS (testable assertions for gate architecture) ===

assert is_safety_property(policy), "Only safety properties are EM-enforceable; non-safety policies have no valid EM gate"
assert not requires_cross_execution_comparison(policy), "Information-flow and hyperproperties cannot be EM gates"
assert not is_liveness_only(policy), "Pure liveness (something good eventually happens) has no EM gate"
assert has_finite_bad_prefix(policy), "Every violation must be detectable from a FINITE execution prefix (eq. 3)"
assert prefix_closure_holds(policy), "If prefix tau' is bad, ALL extensions tau' sigma are bad (eq. 2)"
assert target_can_be_halted_before_bad_step(policy), "EM halts BEFORE the violating step executes, not after"
assert gate_state_fits_bounded_memory(), "In practice: automaton state must fit finite memory (countably infinite state is theoretical)"
assert conjunction_of_em_policies_is_em_policy(), "Composing EM-enforceable policies by conjunction is always EM-enforceable"
assert not uses_wall_clock_time_as_trigger(policy), "Real-time MWT-seconds policies are safety properties but still unenforceable (time cannot be halted)"

=== COMPLEXITY ===

Automaton step: O(|Q'| * |transitions from Q'|) per input symbol; Q' is the current nondeterministic state set.
Space: O(|state_vars encoding|) per automaton instance; unbounded in theory, bounded (finite) in all practical policies.
Parallelism: EM mechanisms compose independently and in parallel; conjunction automaton rejects iff any component rejects. Per-component automata can run on separate system components with no inter-component communication if each automaton's input symbols come from a single component.
Input symbol cost: dominated by Automaton Input Read frequency. Optimization: skip forwarding symbols that cause no state change (Section 4, Automaton Input Read Optimization).

=== KNOWN FAILURE MODES (paper-cited) ===

FM-1 Target corrupts automaton state: "A target that corrupts a security automaton simulation can subvert an enforcement mechanism." (Section 4, Enforcement Mechanism Integrity). Fix: automaton state variables must NOT be writable by the target (isolation).

FM-2 Incomplete mediation: "Ensuring that input to the enforcement mechanism is both correct and complete is a question of target instrumentation and monitoring." Missing events = missed violations. Complete mediation is the required property.

FM-3 Transition predicate too coarse: "A security automaton for policy X only captures the informal requirement if it is impossible to [violate] unless [the right predicate] is true ... All must be identified." (Section 5). An incomplete predicate definition silently admits policy violations.

FM-4 Overreach via stronger policy: "An EM enforcement mechanism for any security policy P' satisfying P' => P also enforces P. However, P' might proscribe executions that do not violate P." (Section 2). Overly strict gates terminate legitimate executions.

FM-5 Attempting to gate liveness via EM: Silently fails - the gate either does nothing useful (can always extend to satisfy) or overly truncates (starves the system). There is no correct EM gate for liveness.

FM-6 Input symbol filter corruption: "If the [Input Read Optimization] is used, then a target can subvert the enforcement mechanism by corrupting the filter that selects whether to forward an input symbol." (Section 4). The optimization opens a subversion channel if the filter is not isolated.

=== DETERMINISTIC-GATE RULES (for ELAI gate architecture) ===

[All quotes are verbatim from the paper with section citations.]

RULE-DG-1 Enforceability necessary condition (Section 2, paragraph "Non EM-Enforceable Security Policies"):
  VERBATIM: "If the set of executions for a security policy P is not a safety property, then an enforcement mechanism from EM does not exist for P."
  ELAI implication: before wiring ANY gate over the SSB event stream, classify the policy. If it is not a safety property, a runtime SSB monitor CANNOT enforce it - move to plan-time admission or static analysis.

RULE-DG-2 Precise EM enforceability conditions (Section 2, equations 1-3):
  VERBATIM eq.1: "P(Pi) : (forall sigma in Pi : P_hat(sigma))"
  VERBATIM eq.2: "(forall tau' in Pi^- : NOT P_hat(tau')) => (forall sigma in Pi : NOT P_hat(tau' sigma))"
  VERBATIM eq.3: "(forall sigma in Pi : NOT P_hat(sigma)) => (exists i : NOT P_hat(sigma[..i]))"
  VERBATIM: "Security policies satisfying (1), (2), and (3) are safety properties [Lamport 1977], properties stipulating that no 'bad thing' happens during any execution."
  ELAI implication: an ELAI gate is a valid EM gate if and only if its trigger predicate satisfies all three: (1) per-event predicate, (2) prefix-closed, (3) finite-prefix-detectable violation.

RULE-DG-3 Safety property formal definition (Section 2, equation 4, citing Lamport 1985):
  VERBATIM eq.4: "sigma NOT IN G => (exists i : (forall tau in Pi : sigma[..i] tau NOT IN G))"
  VERBATIM: "a property G is defined in Lamport [1985] to be a safety property if and only if, for any finite or infinite execution sigma, [eq.4] holds. This means that G is a safety property if and only if G can be characterized using a set of finite executions that are prefixes of all executions excluded from G."
  ELAI implication: a gate is correctly designed iff every execution it would eventually block can be blocked at a FINITE prefix, and blocking that prefix blocks ALL its extensions.

RULE-DG-4 Information flow is NOT EM-enforceable (Section 2):
  VERBATIM: "Information flow does not define sets that are properties (as argued above), so it does not define sets that are safety properties. Not being safety properties, there are no EM enforcement mechanisms for exactly this policy."
  VERBATIM: "Whether information flows from variable x to y in a given execution depends, in part, on what values y takes in other possible executions (and whether those values are correlated with the value of x). A predicate to specify such sets of executions cannot be constructed using only predicates defined on single executions in isolation."
  ELAI implication: any ELAI gate attempting to enforce "model X cannot see user data that model Y saw" as a cross-execution property CANNOT be a runtime monitor. Must enforce via isolation/sandboxing at architecture level (not SSB stream monitoring).

RULE-DG-5 Liveness is NOT EM-enforceable (Section 2):
  VERBATIM: "Availability, if taken to mean that no principal is forever denied use of some given resource, is not a safety property - any partial execution can be extended in a way that allows a principal to access the resource, so the defining set of proscribed partial executions that every safety property must have is absent."
  VERBATIM (Section 3, Figure 4 note): "We cannot specify the stronger security policy (that service be guaranteed after payment) because that is not a safety property - there is no defining set of proscribed partial executions."
  ELAI implication: "agent eventually responds", "task eventually completes", "plan eventually converges" are liveness obligations. NONE of these can be enforced by a runtime SSB monitor alone. Must use watchdogs with finite step bounds (converting liveness to bounded-step safety) or architectural guarantees.

RULE-DG-6 Real-time wall-clock policies are unenforceable even if safety (Section 4, Target Control):
  VERBATIM: "While individual clocks might be stopped, the passage of time cannot be stopped. So the target cannot be stopped from producing input symbols as time passes. Real-Time Availability simply cannot be enforced by running an automaton simulation in tandem with a target, because targets cannot provide the necessary controls to the enforcement mechanism."
  VERBATIM: "Change the specification from 'MWT seconds' to 'MWT execution steps' and the target can be prevented from violating the policy by stopping execution, resulting in an EM-enforceable security policy."
  ELAI implication: ELAI gates expressed as "no more than N seconds elapsed" cannot be runtime-enforced. Must be expressed as "no more than N SSB events / tool calls / steps elapsed" to be a valid EM gate over the event stream.

RULE-DG-7 EM composition is safe (Section 2):
  VERBATIM: "When multiple EM mechanisms are used in tandem, the policy enforced by the aggregate is the conjunction of the policies that are enforced by each mechanism in isolation. This is attractive because it enables complex policies to be decomposed into conjuncts, with a separate mechanism used to enforce each of the component policies."
  ELAI implication: ELAI can run multiple independent gate automata over the SSB stream in parallel (one per safety policy) and AND their verdicts. This is not only valid but the recommended architecture. Each gate only needs to trust its own event source.

RULE-DG-8 Integrity requirement for the gate itself (Section 4, Enforcement Mechanism Integrity):
  VERBATIM: "A target that corrupts a security automaton simulation can subvert an enforcement mechanism built on that simulation. Input to the enforcement mechanism must correspond to target execution, and state transitions must follow the automaton's transition function. Ensuring that input to the enforcement mechanism is both correct and complete is a question of target instrumentation and monitoring."
  VERBATIM: "Ensuring that the target does not interfere with automaton transitions is a matter of isolation - the enforcement mechanism must be isolated from the target."
  ELAI implication: the ELAI SSB stream (the input to any gate automaton) must be write-protected from the agent being monitored. An agent that can append arbitrary SSB rows can subvert any gate. The gate's state variables must not be readable or writable by the agent code path.

RULE-DG-9 Stronger policy always enforces weaker (Section 2):
  VERBATIM: "an EM enforcement mechanism for any security policy P' satisfying P' => P also enforces security policy P."
  ELAI implication: if designing a gate is hard for policy P, it is always valid to implement a STRICTER gate P' that blocks a superset of what P blocks - but this may reject valid executions. Must weigh false-positive cost.

=== GATE SHAPE DECISION TABLE ===

Policy type | EM gate shape | ELAI gate tier
Per-event access control predicate | security automaton over SSB stream | RUNTIME - valid EM gate
History-dependent sequence (no Send after FileRead) | security automaton with state tracking past events | RUNTIME - valid EM gate
Bounded-step budget (no more than N tool calls) | automaton counting SSB event symbols | RUNTIME - valid EM gate (express as steps, not seconds)
Bounded-time budget (no more than T seconds) | NOT an EM gate (time cannot be halted) | PLAN TIME / watchdog thread, not SSB monitor
Information flow / noninterference | NOT an EM gate (cross-execution hyperproperty) | ARCHITECTURE - isolation, sandbox, not monitoring
Liveness / eventual progress | NOT an EM gate (no finite bad prefix) | PLAN TIME - bounded-step reformulation required
Fairness / guaranteed service delivery | NOT an EM gate (liveness) | PLAN TIME / SLA / architectural guarantee
Conjunction of any EM-enforceable policies | EM gate = parallel AND of component automata | RUNTIME - fully composable

=== REFERENCES ===

Alpern and Schneider 1985 - Defining liveness. Inf. Process. Lett. 21, 4 (Oct.), 181-185.
Alpern and Schneider 1987 - Recognizing safety and liveness. Distrib. Comput. 2, 117-126.
Lamport 1977 - Proving the correctness of multiprocess programs. IEEE Trans. Softw. Eng. 3, 2.
Lamport 1985 - Logical foundation. In Distributed systems: Methods and tools for specification. Springer LNCS vol. 190.
McLean 1994 - A general theory of composition for trace sets. IEEE Symp. Security and Privacy.
Erlingsson and Schneider 1999 - SASI enforcement of security policies: A retrospective. NSPW.
