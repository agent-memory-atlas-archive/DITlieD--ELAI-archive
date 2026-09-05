Algorithm: Formal proof that LLM hallucination is computationally inevitable
ArXiv: 2401.11817
Paper: Hallucination is Inevitable: An Innate Limitation of Large Language Models
Authors: Ziwei Xu, Sanjay Jain, Mohan Kankanhalli (NUS)
Date: January 2024 (v2 February 2025)
Venue: arXiv preprint

PROBLEM
Any computable LLM trained only on input-output pairs cannot, in general, converge to every computable ground-truth function. The set of all ground-truth functions strictly exceeds what any c.e. set of computable LLMs can collectively represent, so hallucination is not a training failure -- it is a formal impossibility under the model.

FORMAL DEFINITIONS

LLM as math object:
  h: S -> S, total computable function (S = finite strings over finite alphabet A)
  h^[i] = LLM state after i training samples; trained on {(s_j, f(s_j)) | j < i}
  A computably enumerable (c.e.) set of LLMs is the only assumption -- no architecture constraints

Ground truth / world:
  f: S -> S, total computable function
  G_f = {(s, f(s)) | s in S}  (the formal world)

Hallucination (Definition 4):
  h hallucinates w.r.t. f  iff  for all i in N, there exists s in S such that h^[i](s) != f(s)
  i.e. no training stage ever achieves perfect coverage across all inputs

CORE PROOF MECHANISM
Cantor diagonalization over a c.e. enumeration of all LLM states:
  1. Enumerate all states of all LLMs as a single sequence via Cantor pairing: (h_0^[0], h_0^[1], ..., h_1^[0], ...)
  2. Index each state i against input s_i
  3. Construct f(s_i) = Delta(h_i(s_i)) where Delta is any computable function with Delta(x) != x
  4. By construction f(s_i) != h_i^[j](s_i) for every (i,j) pair
  => every LLM state is wrong on at least one input under this f
  => f is computable, so the impossibility is not from undecidability

KEY THEOREMS

Theorem 1: For all c.e. sets of LLMs {h_0, h_1, ...}, there exists a computable f such that ALL h_i^[j] (i,j in N) hallucinate w.r.t. f.

Theorem 2: Strengthening of T1 -- the same f causes hallucination on INFINITELY MANY inputs (not just one).

Theorem 3 (central practical result): For all computable LLMs h, there exists a computable f such that each h^[j] hallucinates w.r.t. f.

Corollary 1: No computable LLM can prevent itself from hallucinating.

Theorem 4 (concrete class result): No LLM can learn all computable linear orders.

HALLUCINATION-PRONE TASK CLASSES (Table 3)

Poly-time (O(n^k)) LLMs forced to hallucinate:
  - Combinatorial list generation requiring Omega(2^n) solutions (output too large)
  - NP-complete problems: Subset Sum, Boolean SAT
  - co-NP-complete problems: propositional tautology / entailment

All computable LLMs forced to hallucinate:
  - Learning all computable linear orders (Theorem 4)
  - Solving all computable problems (Theorem 3)
  - Undecidable problems: first-order logic entailment (obviously, but formal)

Key insight from paper: "problems beyond LLMs' computation capabilities are hallucination-prone." Increasing parameters and data is futile if f cannot be expressed within the LLM's computational class.

MITIGATION TAXONOMY (Section 4)

1. Scaling (more params, more data): cannot overcome class separation; ineffective for tasks outside LLM's complexity class
2. Prompting (CoT, in-context): effective only when f is procedurally describable within context; Corollary 1 still applies
3. Guardrails / fences: potentially effective; scalability unresolved; outside the formal model scope
4. Knowledge-enhanced LLMs (retrieval, tool use, external oracles): most promising; formally escapes the theorem because the LLM is no longer the sole computable agent -- the external source extends the effective computation

ESCAPE CONDITIONS (what the theorem does NOT cover)
  - LLMs augmented with external knowledge bases, tools, or symbolic solvers: the augmented system is no longer a pure input-output-trained computable function
  - Restricted domains where f IS within the LLM's complexity class: hallucination may be avoidable for that specific f
  - Human-in-the-loop: human corrects the diagonal; no formal bound broken

CRITICAL ASSERTIONS (testable in system design, not code)

assert: any LLM component claiming zero hallucination on open-domain tasks is making a claim that contradicts Theorem 3
assert: refusal / abstention on inputs where LLM cannot produce verified output is formally sound (avoids the hallucination by not claiming h(s) = f(s))
assert: an external-oracle / constructive-checker escape is the only mechanism that formally sidesteps Theorem 1; retrieval or tool use qualifies
assert: confidence scores from the LLM itself do NOT constitute a formal escape (the LLM is still the sole agent; the diagonal argument still holds over its states)
assert: task type determines hallucination risk; NP-complete inputs to a poly-time LLM = guaranteed hallucination class (Table 3)

COMPLEXITY
  Proof: O(1) construction of adversarial f via diagonal; no algorithmic complexity to the theorem itself
  Practical implication: NP-complete inputs face poly-time LLM wall -- no mitigation within the LLM alone
  Space: not a space argument; it is a class-separation argument

FAILURE MODES CITED
  - Assuming scaling eliminates hallucination: paper directly refutes this
  - Trusting LLM self-assessed confidence as a hallucination guard: not a formal escape
  - Treating hallucination as purely a data quality problem: ignores computational class separation
  - Deploying LLMs as sole agents in safety-critical decisions without external checks: paper explicitly warns against this

IMPLEMENTATION PITFALLS (do NOT do this)
  - Do NOT treat "no hallucination on benchmark X" as proof of general correctness (diagonal is over all computable f, not a fixed benchmark)
  - Do NOT use LLM-generated verification of LLM output as an independent check (same diagonal applies to both)
  - Do NOT assume retrieval-augmented generation eliminates hallucination -- it reduces it for retrieved facts but the unaugmented residual still satisfies Theorem 3 on non-retrieved inputs
  - Do NOT conflate "less hallucination" with "hallucination-free" -- the theorem is existential (there EXISTS an f) not universal
