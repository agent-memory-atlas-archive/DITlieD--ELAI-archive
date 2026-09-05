Algorithm: Expressive Power of Transformers with Chain of Thought (CoT / Scratchpad)
ArXiv: 2310.07923
Paper: The Expressive Power of Transformers with Chain of Thought
Authors: William Merrill (NYU), Ashish Sabharwal (Allen Institute for AI)
Version: v5 (2024-04-11)

Problem:
Does intermediate token generation (chain of thought / scratchpad) fundamentally extend the
computational power of a decoder-only transformer, and if so, by how much as a function of
the number of intermediate steps t(n)?

Formal Setup:
A decoder-only transformer f : Sigma* -> Sigma generates t(n) intermediate tokens before
the answer token. CoT(t(n)) denotes the class of languages recognizable by such a transformer
with t(n) intermediate decoding steps. Without intermediate decoding, transformers are bounded
by TC^0 / FO(M) (Merrill & Sabharwal 2023b).

Master Containment (Equation 1 of paper):
TIME(t(n)) ⊆ CoT(t(n)) ⊆ SPACE(t(n) + log n) ⊆ TIME~(t(n)^2 + n^2)

where TIME~(f) denotes TIME(f * polylog(f)).

Inputs:
input_string | sequence of tokens | x in Sigma^n | n >= 1
step_budget   | integer function of n | t(n) | t(n) >= 0; log / linear / polynomial regime

Outputs:
decision_token | single symbol from Sigma | 1 token | language membership yes/no
intermediate_tokens | sequence | t(n) tokens | scratchpad / chain-of-thought tokens

Architectural Preconditions (for lower-bound theorems):
projected_pre_norm | bool | required for Theorem 1 (regular) and Theorem 2 (TM simulation)
  Definition: standard pre-norm applied to a linear projection of sublayer input (subset of dims)
  i.e., each sublayer applies layer-norm only to a learned linear projection of its input
generalized_pre_norm | bool | required for PTIME = CoT(poly) result (Section A.1)
  Definition: multi-pre-norm, each sublayer can take k different linear-projected layer-norms
strict_causal_masking | bool | required for Theorem 2 (Turing machine simulation)
log_precision | bool | standard assumption throughout; all weights are O(log n)-bit floats
layer_norm_hash | architectural module | enables memory storage in decoder-only setting
  Key insight: transformer's denominator in softmax acts as hash; used for indexed memory access

Main Theorems (lower bounds -- constructive existence proofs):

Theorem 1 (Regular language recognition, Sec 3.2):
  For any regular language L, there exists a decoder-only projected-pre-norm transformer with
  strict causal saturated attention that recognizes L using |x|+1 decoding steps (linear in n).
  Proof: simulate finite-state automaton A one step per decoding step.
  Complexity class implication: Regular ⊆ CoT(O(n))   [under TC^0 ≠ NC^1 assumption]

Theorem 2 (Turing machine simulation, Sec 3.3):
  For any Turing machine M running in time t(n) on inputs of length 1+n, there exists a
  decoder-only projected-pre-norm transformer with strict causal saturated attention that
  simulates M using t(n) decoding steps.
  Corollary 2.1: TIME(t(n)) ⊆ CoT(t(n))
  Corollary 2.2: CoT(n) can solve any linear-time solvable problem.
  Corollary 2.3: With O(n^2) steps, directed graph connectivity (NL-complete) is solvable.
  Corollary 2.4: With polynomial steps, all P-complete problems (linear equalities, Horn-SAT,
                  universal context-free recognition) are solvable.
  Construction: uses layer-norm hash as indexed memory; no external memory required.

Theorem 3 (Time upper bound, Sec 4):
  CoT(t(n)) ⊆ TIME~(n^2 + t(n)^2)
  Proof: multitape Turing machine simulation; each forward pass takes O(n + t(n)) time for
  self-attention, total O((n + t(n))^2) = O~(n^2 + t(n)^2).

Theorem 4 (Space upper bound, Sec 4):
  CoT(t(n)) ⊆ SPACE(t(n) + log n)
  Proof: log-precision transformers simulate in TC^0, hence in L = SPACE(log n); t(n) space
  for the scratchpad tape.

Main Complexity Results by Step Regime:

1. NO INTERMEDIATE DECODING (t(n) = O(1)):
   CoT(1) ⊆ TC^0  (standard result; transformers without CoT)
   Cannot recognize NC^1-complete problems or above (under TC^0 ≠ NC^1)

2. LOGARITHMIC STEPS (t(n) = O(log n)):
   CoT(log n) ⊆ SPACE(log n) = L
   Cannot recognize NL-complete or P-complete problems (under NL ≠ L)
   No known concrete problem where log steps help vs no steps
   This regime adds only marginal power over standard transformers.

3. LINEAR STEPS (t(n) = O(n), with projected pre-norm):
   CoT(n) contains all regular languages (Theorem 1 lower bound)
   CoT(n) ⊆ TIME~(n^2) ∩ SPACE(n) ⊆ context-sensitive languages (upper bound)
   Cannot recognize all context-free languages unless CFL-parsing in O~(n^2) time
   Strict separation: Regular ⊆ CoT(n) strictly (assuming TC^0 ≠ NC^1)

4. POLYNOMIAL STEPS (t(n) = O(n^c), with generalized pre-norm):
   CoT(poly) = P   [EXACT characterization -- first for any transformer class]
   Lower: TIME(t(n)) ⊆ CoT(t(n)) for any polynomial t
   Upper: CoT(poly) ⊆ SPACE(poly) ⊆ TIME~(poly^2) ⊆ P (since P closed under poly-speedup)
   Intractable in practice: running polynomial forward passes is computationally prohibitive.

Critical Invariants (testable assertions):
assert log_steps_upper_bound implies: CoT(O(log n)) ⊆ L = SPACE(log n)
assert log_steps_cannot: graph_connectivity NOT in CoT(O(log n)) [assuming NL ≠ L]
assert linear_steps_lower: all_regular_languages ⊆ CoT(O(n))  [with projected pre-norm]
assert linear_steps_upper: CoT(O(n)) ⊆ context_sensitive_languages
assert poly_steps_exact: CoT(O(n^c)) == P  [with generalized pre-norm, for any constant c >= 1]
assert master_chain: TIME(t(n)) ⊆ CoT(t(n)) ⊆ SPACE(t(n)+log n) ⊆ TIME~(t(n)^2 + n^2)
assert no_cot_baseline: CoT(O(1)) ⊆ TC^0  [standard transformer, no intermediate decoding]

Architectural Requirements per Theorem:
Theorem 1 (regular): projected pre-norm + strict causal attention + saturated attention
Theorem 2 (TM sim): projected pre-norm + strict causal masking + saturated attention
  Key module: layer-norm hash (hash = 1/sum(e^{a_j}) denominator of softmax)
  No external memory, no extra positional encodings, no encoder-decoder setup required
Upper bounds (Th 3, Th 4): hold WITHOUT projected/generalized pre-norm (more general)

Complexity:
Time per forward pass: O(n + t(n)) for attention over all prior hidden states
Total time for t(n)-step generation: O((n + t(n))^2) = O~(n^2 + t(n)^2) by Theorem 3
Space for t(n) steps: O(t(n) + log n) by Theorem 4
Parallelism: each forward pass is independent given prior context; sequential in generation steps
Note: polynomial CoT is likely intractable in practice (poly passes of large transformer)

Failure Modes / Limitations cited by authors:
1. Log steps add no known concrete benefit: no problem identified where O(log n) steps help
   over 0 steps; gap between CoT(1) and CoT(log n) not strictly characterized.
2. Linear steps cannot solve all CFLs: conditional on no O~(n^2) CFL parsing algorithm.
3. Lower bounds require architectural conditions (projected/generalized pre-norm) that may
   not hold in all practical transformer implementations.
4. Learnability gap: lower bounds prove existence of weights, not that gradient descent finds
   them. Transformers may not learn to use CoT steps effectively in practice.
5. The poly-step PTIME result is a computability ceiling, not a practical capability claim.
6. Log-linear gap is not a strict separation: proving CoT(log n) ⊊ CoT(n) would require
   separating L from NC^1, an open problem in complexity theory.
7. Context-window constraint not modeled: practical transformers have finite context; t(n)
   steps consume context, which is not reflected in the asymptotic model.

Implementation Pitfalls (do NOT do these):
Do NOT conflate CoT step count with transformer depth (layers); they are independent dimensions.
Do NOT apply lower-bound results to standard pre-norm transformers: projected pre-norm required.
Do NOT interpret log-step CoT as "better than no CoT" -- no concrete separation known.
Do NOT assume polynomial-step CoT = PTIME without generalized pre-norm (upper bound holds, lower may not).
Do NOT ignore the strict causal masking requirement for Theorem 2 (TM simulation).
Do NOT treat the layer-norm hash as standard attention: it exploits the softmax denominator.
Do NOT infer learnability from expressivity: constructive proofs give existence, not trainability.

Relation to Hallucination / Ceiling Thesis:
This paper CONTRADICTS the O(N^2*d) hallucination-ceiling thesis (as stated in arXiv:2507.07505).
That thesis claims each generated CoT token is merely another O(N^2*d) pass that buys nothing.
This paper proves the opposite: each additional pass at step t carries accumulated scratchpad
state from all prior t-1 passes, making the effective computational resource t(n) -- not just
the per-pass complexity. The PTIME = CoT(poly) result establishes that sequential intermediate
generation (not just parallel depth) is a distinct computational resource. The complexity class
of solvable problems expands monotonically with t(n): log -> L, linear -> Regular+, poly -> P.
This is a provable, unconditional (modulo standard complexity assumptions) hierarchy.
