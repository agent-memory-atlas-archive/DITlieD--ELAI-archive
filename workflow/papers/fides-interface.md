Algorithm: FIDES (Flow Integrity Deterministic Enforcement System)
ArXiv: 2505.23643
Paper: Securing AI Agents with Information-Flow Control
Authors: Manuel Costa, Boris Kopf, Aashish Kolluri, Andrew Paverd, Mark Russinovich, Ahmed Salem, Shruti Tople, Lukas Wutschitz, Santiago Zanella-Beguelin (Microsoft Research)
Date: submitted 2025-05-29, revised 2025-09-04 (v2)
Venue: arXiv preprint only -- no peer-reviewed venue as of 2026-06-02 (VERIFIED via arXiv abstract page + Microsoft Research publication page)
Code: https://github.com/microsoft/fides (Jupyter notebook tutorial, uses GPT-4o/4.1 via Azure OpenAI)

--- IDENTITY VERIFICATION ---
arXiv:2505.23643 was directly confirmed to resolve to this paper before extraction.
The 6.3% utility-overhead figure was confirmed via web search snippet quoting the paper directly.
CaMeL (arXiv:2503.18813) was separately fetched and verified; comparison notes are grounded.

--- MECHANISM ---

Information-flow lattice (Section 3, Algorithms 5-6):
  Labels are two-dimensional product of:
    Integrity dimension: T (Trusted) < U (Untrusted)
    Confidentiality dimension: L (Low/Public) < H (High/Secret)
    Product lattice bottom = (T, L) = trusted+public; top = (U, H) = untrusted+confidential
    Confidentiality can also be reader-set-based: label = set of authorized readers

Label assignment (at agent boundary):
  - User query / system prompt: assigned (T, L) -- trusted, public
  - Tool results from the environment: assigned (U, H) by default unless policy says otherwise
  - Each variable in the planner context carries its label

Label propagation (Algorithm 5-6, conservative/pessimistic):
  For a tool call f(arg1, arg2, ...):
    result_label = join(labels(R(f))) join label(f) join join(labels(args))
    where R(f) = set of variables the tool may read (declared in tool schema)
    join = lattice least-upper-bound
  LLM query is treated as if it reads all variables in context -> label of output = join of all input labels
  This is intentionally conservative: if any untrusted input touches the LLM, output is labeled untrusted

Security policy enforcement (before any consequential tool call):
  P-T (Trusted Actions policy): tool call is allowed only if result_label.integrity <= T
    Effect: a tool call that depends on untrusted data is BLOCKED -> prevents prompt injection
  P-F (Permitted Flows policy): for egress tool f(recipients, data):
    data_label.confidentiality <= authorized_readers(recipients)
    Effect: confidential data cannot be sent to unauthorized recipients -> prevents exfiltration

--- NOVEL PRIMITIVES (Fides innovations beyond basic taint-tracking) ---

1. Selective Variable Hiding (Algorithm 7, HIDE function):
   For each tool result v with label ell:
     If ell.integrity > ell_context (result is more untrusted than current context):
       Create fresh variable reference x; store x -> v^ell in planner memory
       Return reference x (opaque token) to LLM, NOT the raw data
     Else (data is safe to expose):
       Return v directly to LLM
   Effect: LLM sees trusted/safe data directly (good for planning quality);
           untrusted data is quarantined behind opaque variables;
           reduces unnecessary abstraction vs always-hide approaches

2. Constrained Variable Inspection (query_llm tool):
   query_llm(prompt: str, variables: List[str], expected_output_type: Type) -> result_variable
   Runs a quarantined sub-LLM with no tool access
   Output type constrained to: bool | enum[values] | dict[schema] | string
   Type lattice: bool < enum < string (lower capacity = less information leakage)
   Label endorsement: output labeled with (U, L) but type constrains the channel width
   E.g., a bool output endorsed as less dangerous than a string output
   Effect: safely inspects untrusted variable content with bounded leakage;
           result can be used in control flow after constrained endorsement

3. Task Taxonomy (classification, not a runtime primitive):
   Data Independent (DI): control flow does not depend on tool results; pass by reference suffices
   Data Independent + Query-LLM (DIQ): needs to inspect data but control flow is fixed; query_llm handles
   Data Dependent (DD): control flow depends on untrusted results; requires constrained query + endorsement
   Solvability: DI and DIQ fully handled; DD requires careful policy; some DD tasks are undecidable

--- RELATION TO CaMeL (arXiv:2503.18813) ---

CaMeL (Debenedetti et al. 2025, "Defeating Prompt Injections by Design"):
  Mechanism: extracts control flow from trusted query; untrusted data is handled as data-only;
             uses capability-based policies to prevent exfiltration over unauthorized flows;
             separates trusted planner from untrusted data inspection
  Key property: untrusted data NEVER influences program control flow (hard structural separation)
  Limitation (per Fides taxonomy): effectively handles DI tasks; utility degrades on DIQ/DD tasks
    because it cannot reason about untrusted content at all without a query_llm equivalent

What Fides adds over CaMeL:
  a) Formal lattice model -- CaMeL uses capability rules; Fides uses a full 2D label lattice
     with provable non-interference (Proposition 1 in paper)
  b) query_llm primitive with type-aware endorsement -- CaMeL has no equivalent; Fides can
     safely inspect untrusted variable content with bounded leakage via constrained decoding
  c) Selective hiding vs always-hiding -- CaMeL quarantines all untrusted data; Fides exposes
     trusted portions directly to planner LLM, improving task completion on DIQ tasks
  d) Explicit confidentiality tracking -- CaMeL focuses on integrity (injection prevention);
     Fides tracks both confidentiality (exfiltration) and integrity simultaneously
  e) Formal task taxonomy (DI/DIQ/DD) -- characterizes which tasks each planner design can solve
     with security; CaMeL lacks this theoretical characterization

Paper cites CaMeL as "Code-based planner (Debenedetti et al. 2025, concurrent work)" -- they
are concurrent, not sequential; Fides was not designed as a strict extension of CaMeL.

--- WHAT TOOL-FILTER / LEAST-PRIVILEGE DOES NOT PROVIDE ---

Tool-filter approach (e.g., pre-commit tool allowlist per agent role):
  Provides: point-in-time access control -- agent may or may not call a given tool
  Does NOT provide:
    - Flow tracking: tool-filter cannot detect that a tool call is safe to invoke but
      the DATA passed to it was tainted by an injected instruction from a prior step
    - Multi-step taint: an attacker can split the injection across two tool calls;
      each call passes the filter individually but together they exfiltrate the secret
    - Confidentiality enforcement: tool-filter blocks tool invocations but cannot enforce
      which data values are passed to an allowed tool (e.g., send_email is allowed but
      the filter cannot prevent it being called with a secret as the body)
    - Formal guarantee: tool-filter provides deny-list semantics; Fides provides
      provable non-interference for integrity (Proposition 1)

In the "lethal trifecta" scenario (agent simultaneously holds: (1) trusted secret S,
(2) untrusted content from environment C, (3) egress channel E):
  Tool-filter: can block E entirely (but breaks utility) or allow E entirely (exfiltration risk)
  Fides: tracks label of data flowing to E; blocks E(S, ...) if S.confidentiality > authorized_readers;
         blocks E if the call was influenced by C.integrity via P-T; enforces both simultaneously;
         query_llm allows reasoning about C without letting C's taint reach E

--- MEASURED RESULTS (VERIFIED from HuggingFace paper page, confirmed against web search) ---

All numbers VERIFIED from HuggingFace paper page fetch (source: huggingface.co/papers/2505.23643)
except where tagged otherwise. Table/figure numbers from same source.

Utility overhead without policy checks (Figure 3 / Finding 2):
  VERIFIED: "6.3% decrease vs Basic planner with policies disabled" (also confirmed via web search snippet)
  Mechanism: query_llm complexity causes the overhead, not label tracking itself

Attack success (Table 1, gpt-4o model):
  VERIFIED: Basic planner: 163 successful injections (156 without policies);
            FIDES: 24 (23 without policies), 1 (0) with policies
            FIDES admits 133 fewer successful injections than Basic planner
            With policy checks: FIDES blocks all policy-violating attacks (0 successes)
  Note: "1" remaining with policies = task that is not covered by the policy, not a bypass

Task Completion Rate (TCR) with policy checks (Figure 4 / Finding 3):
  VERIFIED: FIDES 16.7% better than Basic planner on o1 reasoning model
            FIDES 24.5% better than Basic planner on o3/o4-mini after prompt tuning
            Basic planner TCR drops up to 40% (gpt-4o) when policies enforced
            FIDES planner TCR drops up to 24.5% (o3/o4-mini) -- better than Basic

TCR improvement from prompt + interface tuning (Finding 4):
  VERIFIED: 8.2% absolute utility gain from tuning; ~24% advantage over Basic across environments

Variable Passing planner comparison (Table 1, gpt-4o):
  VERIFIED: Variable Passing: 12 successful injections (0 with policies), TCR 57.52% worse than FIDES
  Note: Variable Passing always hides all data; utility penalty is its inherent limitation

Benchmark: AgentDojo (arXiv:2406.13352, Debenedetti et al. 2024)

--- INPUTS ---

Agent context (planner input) | labeled variable map | map: var_name -> (value, label) | mandatory
Tool schema | function signatures + R(f) read-set declarations | list of tool definitions | mandatory
Security policy P | policy expression | P-T and/or P-F | mandatory for security guarantees
User query | string | (T, L) label assigned at boundary | mandatory
Tool result (per step) | any value | (U, H) default label | per tool call

--- OUTPUTS ---

Tool call decision | allow/block + rationale | binary per call | mandatory
Updated variable store | map: var_name -> (value, label) | after each step | maintained
query_llm result | typed constrained value | (U, output_type) labeled | when called
Final task output | depends on task | label must satisfy P-F before egress | per task

Invariants:
  Result label of any variable is >= join of all labels that influenced it (monotone)
  A variable exposed directly to planner LLM has label <= current context label
  A blocked tool call leaves variable store unchanged
  query_llm output label = (U, L) with type constraint; never (T, *)

--- CRITICAL INVARIANTS (TESTABLE) ---

# Proposition 1: non-interference for integrity
# A tool call f whose label has integrity component U must be blocked under P-T
assert not (allow_call and result_label.integrity == UNTRUSTED and policy_PT_active)

# Label monotonicity: result label >= join of all input labels
assert result_label >= join([label(arg) for arg in args] + [label(f)] + [label(r) for r in R_f])

# Selective hiding: any variable directly exposed to LLM must have label <= context_label
assert all(label(v) <= context_label for v in exposed_vars)

# query_llm output type constraint: output capacity bounded by declared type
assert type_of(query_llm_result) in {bool, Enum, dict_schema, str}

# P-F: egress tool call blocked if data label confidentiality > authorized readers
assert not (allow_egress and data_label.confidentiality > authorized_readers_label)

# Variable store immutability on block: blocked call does not mutate store
assert variable_store_before_blocked_call == variable_store_after_blocked_call

--- COMPLEXITY ---

Time per agent step: O(|R(f)| + |args|) for label join -- O(1) per label operation on binary lattice
  Reader-set-based confidentiality: O(|readers|) per join -- can be large for many-reader systems
Space: O(|variables_in_context|) for label store -- scales with agent memory footprint
  Each variable entry: (value, (integrity_bit, confidentiality_set)) -- low constant overhead
Quarantined LLM (query_llm): requires a second LLM inference pass per inspection call
  Overhead: one full forward pass per query_llm invocation + constrained decoding
  Measured: 6.3% utility decrease attributed to query_llm complexity, not label tracking

--- FAILURE MODES (cited in paper) ---

DD (Data Dependent) tasks: control flow that depends on untrusted data cannot be fully secured
  without constrained endorsement; some DD tasks remain unsolvable with security guarantees.
  Example: "Read emails; if meeting request, create calendar event" -- control flow on untrusted content.

Implicit flows through control flow: Fides provides explicit secrecy (blocks direct data flows)
  but NOT perfect confidentiality -- implicit flows via control structure (branches taken/not taken)
  can leak confidential bits. Paper explicitly accepts this as pragmatic trade-off for utility.

query_llm constrained decoding failure: if the quarantined LLM generates output inconsistent with
  the declared type schema, the call fails. Requires output schema to be exactly declared.

Over-taint: conservative LLM query labeling (output = join of all context labels) may taint
  variables that are logically unrelated to the injected content. Reduces utility on complex tasks.

Policy specification burden: P-T and P-F must be specified per-tool per-deployment.
  Incorrect policy = false sense of security. No automatic policy synthesis in the paper.

Non-reasoning models (gpt-4o): variable-passing and query_llm add cognitive load that degrades
  plan quality on non-reasoning models; FIDES works best with o1/o3/o4-mini reasoning models.

--- IMPLEMENTATION PITFALLS (do NOT do this) ---

Do NOT propagate labels only on direct data flows -- must also propagate on control-flow-induced reads
  (if a branch is taken based on a tainted variable, the branch body's tool calls are also tainted).

Do NOT allow the planner LLM to directly observe a variable whose label.integrity == U --
  the HIDE primitive must intercept and return an opaque reference instead.

Do NOT use a shared LLM for query_llm -- the quarantined sub-LLM must have no tool access;
  reusing the planner LLM with a "no tools" instruction is NOT sufficient for formal guarantees.

Do NOT treat tool-filter (allowlist) as equivalent to IFC -- they are orthogonal; tool-filter
  controls which tools are callable, IFC controls which data values may flow to which tools.

Do NOT use string type as the default output type for query_llm -- always use the most
  restrictive type (bool > enum > dict > string) to minimize endorsement channel width.

Do NOT skip R(f) read-set declarations in tool schemas -- if R(f) is undeclared, the conservative
  default should be "reads all variables in context", which maximally taints results.

Do NOT implement label join as intersection -- join is least-upper-bound (most restrictive),
  meaning a result touching ANY untrusted input is labeled untrusted, not the intersection.

--- ELAI HARNESS BORROW: INFORMATION-FLOW FOR AGENTS WITH LETHAL TRIFECTA ---

The "lethal trifecta" scenario in ELAI harness:
  Agent simultaneously holds:
    (1) trusted secret S -- e.g., API key, user credential, internal plan content
    (2) untrusted content C -- e.g., LLM sidecar response, web search result, tool output
    (3) egress channel E -- e.g., send_message, write_file, emit_ssb_event, IPC dispatcher

Fides interface elements directly borrowable:

A. Label struct per agent variable:
   struct Label { integrity: {Trusted, Untrusted}, confidentiality: ReaderSet }
   Assign at boundary: user prompt/task = (Trusted, public); sidecar/tool output = (Untrusted, H)

B. join(labels) before any cross-agent message or IPC call:
   result_label = labels.iter().fold(bottom, |acc, l| acc.join(l))
   Block IPC send if result_label.integrity == Untrusted and policy requires Trusted actions only

C. For auditor/memory-writer agents that hold plan secrets + receive sidecar completions:
   Wrap sidecar response in opaque variable reference; do not pass raw sidecar text as plan input
   Only expose sidecar result via constrained query (e.g., bool "is this safe to apply?" check)

D. For SSB event emission with user data + sidecar-influenced content:
   Track label of each SSB field; block emission if any field.confidentiality > authorized_subscribers

E. Policy enforcement point: in the orchestrate protocol, before any Edit/Write call,
   assert that the data flowing to the write was not tainted by untrusted tool/sidecar output
   (maps to P-T: consequential action only if integrity == Trusted)

Recommended scope: implement as a lightweight label wrapper around agent message passing in
  crates/orchestrator or the IPC dispatcher, not in the full sidecar inference path.
  No constrained decoding needed for harness use -- constrained query can be a rule-based check.
