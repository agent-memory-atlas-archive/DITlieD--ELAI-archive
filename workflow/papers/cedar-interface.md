Algorithm: Cedar Authorization Policy Language
ArXiv: 2403.04651
Paper: Cedar: A New Language for Expressive, Fast, Safe, and Analyzable Authorization (Extended Version)
Authors: Cutler et al., OOPSLA 2024
Crate: cedar-policy 4.11.1 (crates.io, 2026-06-09, ~714K dl/mo, Apache-2.0, actively maintained)

=== REQUEST MODEL (Section 3.2) ===

Authorization request sigma is a 4-tuple:
  principal  | entity reference E::s  | "who is making the request"
  action     | entity reference E::s  | "what operation is being performed"
  resource   | entity reference E::s  | "what is being accessed"
  context    | record value {f1:v1,...} | "ambient attributes of the request"

Entity store mu maps entity references E::s to pairs (v, h) where v is a record of attributes
and h is the set of ancestors in the entity hierarchy DAG.

Verbatim (Section 3.2): "An authorization request sigma is a map from variables x to values v,
where v is an entity reference when x is either principal, action, or resource, and v is a record
when x is context."

=== POLICY STRUCTURE (Section 3.1, Figure 5) ===

Grammar (verbatim from Figure 5):
  Policy (c) ::= Effect ( Scope ) { Cond } ;
  Effect      ::= permit | forbid
  Scope       ::= Principal , Action , Resource
  Principal   ::= principal [(in | ==) E::s]
  Action      ::= action [((in | ==) E::s) | (in [E::s, ...])]
  Resource    ::= resource [(in | ==) E::s]
  Cond        ::= ( when | unless) { e }

A policy set C is a set of individual policies c.
The scope constrains which (principal, action, resource) tuples the policy applies to.
  - principal (no constraint) = matches any principal
  - principal in Team::"admin" = principal must be a member of that group
  - principal == User::"alice" = exact match
  - action in [Action::"read", Action::"write"] = action must be one of those
The condition (when/unless) is a boolean expression e evaluated against the full request and store.
  - when { e } = policy applies if e evaluates to true
  - unless { e } = policy applies if e evaluates to false (i.e., when NOT e)

Example permit and forbid from Figure 2 (TinyTodo):
  permit(principal in Team::"admin", action, resource in Application::"TinyTodo");
  forbid(principal in Team::"interns", action == Action::"CreateList", resource == Application::"TinyTodo");
  permit(principal, action, resource) when { resource is List && resource.owner == principal };

=== AUTHORIZATION ALGORITHM (Section 3.3, pseudocode) ===

Verbatim (Section 3.3):
  def authorize(mu, C, sigma) =
    def evaluate(e) = v where mu, sigma |- e ->* v
    let CS = {c | c in slice(sigma, C) and evaluate(toexp(c)) = true}
    if forbids(CS) = {} and permits(CS) != {} then Allow else Deny

Key invariant: "the request is allowed if there are no satisfying forbid policies and at least one
satisfying permit policy, else it is denied." (Section 3.3)
This is deny-unless-permitted-and-not-forbidden semantics.
forbid always overrides permit; deny is the default when no permit matches.
Multiple permits are logically OR'd; a single forbid voids all permits.

Policy slice: slice(sigma, C) returns only policies whose scope principal/resource constraints
are compatible with the request principal/resource (and their ancestors). This is a sound
approximation -- the slice never excludes relevant policies -- enabling O(1) scope-level filtering
before O(policy) condition evaluation.

=== SCHEMA AND VALIDATION (Section 3.4, Figure 4) ===

Schema is a pair (M, S):
  M: entity schema  -- maps entity type E to (record type tau, set of allowed parent types H)
  S: action schema  -- maps action A to (Ap, Ar, Ax):
       Ap = set of allowable principal entity types
       Ar = set of allowable resource entity types
       Ax = type of context record for requests involving A

Concrete schema syntax (Figure 4 example):
  entity Application;
  entity Team, User in [Team, Application];
  entity List in [Application] { readers: Team, editors: Team, owner: User };
  action CreateList appliesTo { principal: [User], resource: [Application] };
  action GetList   appliesTo { principal: [User], resource: [List] };

Validation process (Section 3.4): For each action A in dom(S), produce request environments
A_Gamma = {principal: Ep, resource: Er, context: Ax | Ep in Ap, Er in Ar}.
Convert policy c to expression e = toexp(c), substitute action A to get e_A.
Typecheck e_A under every environment Gamma in A_Gamma.
A policy is valid iff it typechecks in EVERY environment for EVERY applicable action.

Type system uses (Section 3.4, Figure 7):
  - Boolean singleton types True and False: expressions statically known to be true/false get
    singleton types, allowing dead branches to be skipped during validation without false alarms
  - Capability tracking alpha: tracks which optional attributes are definitely available at each
    program point; accessing e.f where f is optional requires e.f in capability alpha
  - Subtyping: True <: Bool, False <: Bool, Set tau <: Set tau' when tau <: tau'

Validation soundness theorem (Section 3.5): "If a policy is valid with respect to a schema,
then evaluating the policy (as an expression) on any request conforming to the schema will
not result in an error."

=== SYMBOLIC ANALYSIS (Section 4) ===

Cedar's design is balanced to permit sound, decidable, and complete SMT encoding (Section 3, intro):
"We designed Cedar specifically to support an SMT encoding that is sound, decidable, and complete;
no prior authorization language enjoys such an encoding." (Section 3, page 7)

Type-to-SMT mapping T (Figure 8):
  T(Bool)   := Bool
  T(Long)   := (_ BitVec 64)
  T(String) := String
  T(Set tau) := (Set T(tau))
  T(E)      := iota(E)  -- a fresh SMT datatype per entity type
  T(R)      := iota(R)  -- a fresh SMT datatype per record shape

Entity store and request are encoded as UNINTERPRETED FUNCTIONS (Figure 9):
  - Each variable x in request environment maps to an uninterpreted constant of type T(tau)
  - Each entity type E's attribute record maps to uninterpreted function F(E, R): T(E) -> T(R)
  - Each entity type E's ancestors of type E' maps to uninterpreted function F(E, E'): T(E) -> Set T(E')
  This represents the set of ALL possible concrete stores conforming to the schema.

Expression compilation (Figure 10): big-step M, Gamma |- e ↓ e_hat
  - Returns (Option T(tau)) to encode possible errors as none
  - Variables: x -> (some V(x, tau))
  - Entity refs: E::s -> (some (iota(E) s))
  - Attribute access e.f: ifOk(e_hat, (some f_hat)) for required attrs;
    ifOk(e_hat, f_hat) for optional attrs where f_hat is (Option T(tau_f))
  - Membership e1 in e2: encodes as (or (= v1 v2) (set.member v2 (F(E1,E2) v1)))
  - Conditionals: reduced by singleton-type specialization when guard is statically true/false

Well-formedness constraints for entity hierarchy (Figure 11, Section 4.3):
  To avoid undecidability from quantified acyclicity/transitivity formulas, Cedar uses GROUND
  well-formedness constraints wf(e, Gamma, M) based on the FOOTPRINT of expression e (the
  finite set of entity subexpressions). For each pair of footprint entities:
    - wfa(e1, E1): acyclicity -- entity e1 is not its own ancestor
    - wft(e1, e2, E1, E2, E3): transitivity -- ancestor relation is transitively closed on footprint
  Grounding maintains decidability (uses only decidable SMT theories: BitVec, String, Set) while
  ensuring completeness.

Theorem 4.1 (soundness of symbolic compilation, Section 4.1):
  "Let e, Gamma, and M be an expression, environment, and entity schema such that alpha; Gamma |- e: tau, eps
  for some alpha. Then, there is a well-typed SMT term e_hat of type (Option T(tau)) such that M, Gamma |- e ↓ e_hat."

Theorem 4.2 (soundness+completeness of equivalence analysis, Section 4.4):
  "Let e1 and e2 be well-typed boolean expressions under environment Gamma and entity schema M.
  Let e_hat1 and e_hat2 be their encodings. Let phi be the term (= isTrue(e_hat1) isTrue(e_hat2)).
  Then, wf(phi, Gamma, M) union {(not phi)} are unsatisfiable iff either e1 and e2 are both true
  or both untrue on all well-formed inputs conforming to Gamma and M."

Analysis queries supported (Section 2, Section 5.4):
  1. EQUIVALENCE: "ensure that when refactoring a set of policies, the authorized permissions do not change"
     -- encode both policy sets as expressions, ask SMT if any request satisfies one but not the other
     -- solver UNSAT = policies are equivalent; SAT = concrete counterexample request
  2. REGRESSION / CHANGE-IMPACT: check that a policy refactoring does not grant or revoke permissions
  3. VACUITY: detect policies that never fire (always evaluate to false on all valid requests)
  4. NEVER-GRANTS (implied): query whether any request can ever be allowed under a policy set

SMT solver used in experiments: CVC5 v1.0.5 (Section 5.4). Average encode+solve time: 75.1 ms
on the example policy sets.

=== INPUTS ===

Mandatory:
  policy_set   | set of Cedar policies c | effect + scope + conditions
  entity_store | mu: E::s -> (record, ancestor_set) | the entity data
  request      | sigma: {principal, action, resource, context} | the authorization query

Optional:
  schema       | (M, S) pair | enables validation; not required for evaluation

=== OUTPUTS ===

authorize(mu, C, sigma) -> Allow | Deny
  Invariant: Deny is default (deny-by-default); Allow requires >= 1 permit and 0 forbids in CS
  Invariant: forbid always overrides permit -- no permit can override a matching forbid

=== HYPERPARAMETERS / DESIGN CHOICES ===

deny_by_default       | true     | hardcoded | if no permit matches, result is Deny
forbid_overrides      | true     | hardcoded | any matching forbid makes result Deny regardless of permits
scope_slicing_enabled | optional | sound approximation for performance; never excludes relevant policies
schema_validation     | optional | enables type-based error prevention; does not change evaluation semantics
max_operators         | constant-time for most; linear for containsAll/containsAny | no loops = linear bound

=== CRITICAL INVARIANTS (testable assertions) ===

# Authorization deny-by-default
assert authorize(mu, C, sigma) == "Deny" when no permit policy matches
assert authorize(mu, C, sigma) == "Deny" even if permit matches, if any forbid matches

# forbid overrides permit: deny overrides
assert not (authorize(mu, C_permit_only, sigma) == "Allow" and
            authorize(mu, C_permit_plus_forbid, sigma) == "Allow")
# above: adding a matching forbid to a policy set cannot change Deny to Allow

# Scope slicing is sound: slice never excludes relevant policies
# For any request sigma, any policy c that could be satisfied must appear in slice(sigma, C)
assert set(policies_that_evaluate_true(sigma, C)).issubset(set(slice(sigma, C)))

# Schema validation soundness: valid policies do not error at eval time
# If policy passes validator under schema, evaluation on conforming request cannot error
assert valid_policy_never_errors_on_conforming_request(policy, schema, request)

# SMT equivalence completeness (Theorem 4.2)
# Two policy sets P1, P2 are equivalent iff SMT formula is UNSAT
assert smt_is_unsat(phi) == policies_are_equivalent(P1, P2, schema)

=== COMPLEXITY (Section 3.3 and paper intro) ===

Evaluation: "Most Cedar operators take constant time, and looping constructs are linear" (abstract)
  - Most operations: O(1)
  - set containsAll/containsAny/contains: O(n) in set size
  - No unbounded recursion or loops; strict polynomial (linear) worst case
  - Policy slicing: O(1) scope filter -> O(slice) condition evaluation

Performance (Section 5.2): Cedar 4-11 us per request at 50 entities; 28.7x-35.2x faster than
OpenFGA; 42.8x-80.8x faster than Rego (OPA) on the same workloads.

SMT encoding and analysis: 75.1 ms average encode+solve time on example policy sets (Section 5.4)
Space: O(|entities|) for store; O(|policies| * |actions|) for validation across all action environments

=== FAILURE MODES (from paper) ===

1. Quantified well-formedness constraints cause SMT undecidability -- Cedar avoids by grounding
   on the footprint instead. Do not use quantified forall in the encoding.
2. Optional attribute access without has-check causes runtime error -- prevented by capability
   type system during validation. Unvalidated policies can still error at eval time.
3. forbid policy incorrectly scoped can silently revoke too many permissions -- equivalence
   analysis is the intended detection mechanism.
4. Inline authorization logic in application code is the design anti-pattern Cedar addresses;
   moving logic back inline loses auditability and analyzability.
5. Policy set refactoring without equivalence check can silently change granted permissions.
   Section 2 shows a concrete example where a seemingly correct merge of two permit policies
   into one revokes a permission for interns.

=== IMPLEMENTATION PITFALLS (do NOT do this) ===

1. Do NOT use quantified SMT constraints for entity hierarchy (causes undecidability); use ground
   footprint-based constraints per wf(e, Gamma, M) construction.
2. Do NOT assume permit+permit = more access without checking forbid; forbid is always final.
3. Do NOT use Datalog/recursive rules instead of Cedar entity hierarchy for transitive membership;
   Rego-style recursive encoding degrades performance exponentially and resists analysis.
4. Do NOT skip schema validation and assume policies are safe; unvalidated policies can access
   non-existent attributes and error at runtime.
5. Do NOT implement authorization as a library function in application code -- this defeats
   policy auditability, versioning, and equivalence analysis.
6. Do NOT use unless { e } as the primary condition form -- it is syntactic sugar for when { !e },
   but the negation is easy to misread during audit.

=== ELAI GATE MAPPING ===

Cedar models authorization as: (principal, action, resource, context) -> Allow|Deny
ELAI gate semantics today: FSM state gate + Python checker = allow or block transition/tool-use

Proposal: ELAI's transition and capability gates expressed as Cedar-style policies over
  principal   = agent-role (e.g., AgentRole::Executor, AgentRole::Auditor, AgentRole::MemoryWriter)
  action      = tool call or FSM transition (e.g., Action::"Edit", Action::"cargo_test", Action::"git_commit")
  resource    = file path scope or SSB-scope (e.g., Resource::"elai_cc/crates/", Resource::"memory_bank")
  context     = FSM state, agent substate, session flags

What Cedar buys ELAI:
  - Policy EQUIVALENCE CHECKING: verify that a gate refactor does not change what agents are permitted
    to do. Currently, ELAI cannot prove two versions of a Python gate checker are equivalent.
  - CENTRAL AUDIT: all gate rules in one place, human-readable, version-controlled separately from
    the code they guard.
  - REGRESSION DETECTION: if a new plan adds a WIRE-DARK exemption, Cedar analysis can check whether
    it creates a permission the prior policy set did not have.
  - deny-by-default is already ELAI's model (fail-closed); Cedar formalizes and enforces it.

Tradeoff vs current frozen-Python-checker approach:
  PRO Cedar: analyzable (equivalence, regression), human-auditable policy language, schema-typed
  PRO frozen Python: zero additional runtime dependency, inspectable as plain code, no SMT solver
    required at gate check time (only at policy-authoring time for analysis)
  HYBRID path: use cedar-policy crate (4.11.1, Apache-2.0) for gate evaluation in Rust (the
    ELAI core is already Rust); keep Python checkers for build-time static analysis;
    express gate RULES as Cedar policies for human auditability and regression-equivalence analysis

Adoption path: cedar-policy = "4.11.1" in Cargo.toml. No naming risk -- this is the official
Amazon-maintained crate, verified on lib.rs and crates.io 2026-06-21.
