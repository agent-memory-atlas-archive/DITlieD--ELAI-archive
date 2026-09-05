Algorithm: Compositional Reasoning Questions (CRQ) -- architecture expressivity analysis
ArXiv: 2503.01544
Paper: Compositional Reasoning with Transformers, RNNs, and Chain of Thought
Authors: Gilad Yehudai, Noah Amsel, Joan Bruna (NYU Courant / Flatiron)
Version: v2, generated 2026-01-28

Problem:
What resource (depth, memory, CoT tokens) is necessary and sufficient for each of three
neural architectures (deep transformer, shallow RNN, shallow transformer+CoT) to solve
Compositional Reasoning Questions (CRQs)? CRQs are rooted trees T=(V,E) where each
node v is labeled x_v in Gamma^d (finite vocab Gamma, fixed d) and the answer at each
non-leaf is argmax_{u in C(v)} <A(u), x_v> -- the child whose accumulated answer is
most similar to the parent's query vector. Boolean formula evaluation is a special case
(proven NC^1-complete). Size n = |V|.

Inputs:
CRQ tree | rooted binary tree T=(V,E) | n nodes, each labeled x_v in Gamma^d | mandatory
node embedding | tuple (x_v, z_v, z_{P(v)}, depth(v)) | x_v=label, z_v=positional encoding, P(v)=parent pos | mandatory
ordering | sequence over V | affects RNN construction only; benevolent order (memory-rank sort) vs adversarial | conditional
CoT budget | integer T | number of scratchpad tokens emitted before final answer | optional (CoT variant only)

Outputs:
answer vector | x_u in Gamma^d | the label of the child that recursively maximises inner product up to root | invariant: must equal A(v_r) per Definition 3.1
binary answer | {T, F} | for Boolean formula evaluation special case | invariant: consistent with Boolean semantics

Hyperparameters:
depth L (transformer) | default: O(log n) for CRQ tree depth log n | range: 1 to n | sensitivity: CRITICAL -- constant depth provably insufficient (Theorem 4.3)
hidden dim m (RNN) | default: O(log n) with benevolent ordering | range: O(log n) to O(n) | sensitivity: CRITICAL -- constant dim provably insufficient (Theorem 5.5)
CoT tokens T_cot | default: O(n) for shallow transformer | range: O(log n) to O(n) | sensitivity: CRITICAL -- O(log n) CoT tokens provably insufficient (Theorem 6.1 lower bound)
embedding dim d | problem-defined finite vocab Gamma | fixed at problem definition | not a free hyperparameter

Critical theorems (testable assertions in terms of scaling):

Theorem 4.1 (deep transformer upper bound):
  assert transformer_depth >= CRQ_tree_depth  # depth-L transformer solves depth-L CRQs
  # Parameters: O(L * log^2(n)); Sequential runtime: O(L * n^2 * log n); Parallel: O(L)

Theorem 4.3 (shallow transformer lower bound):
  # Under standard hardness assumptions (NC^1 != TC^0):
  assert constant_depth_transformer_cannot_solve_CRQ_of_unbounded_depth  # Merrill-Sabharwal TC^0 containment applied

Theorem 5.4 (shallow RNN upper bound, benevolent order):
  assert rnn_hidden_dim >= c * log(n)  # O(log n) hidden dim + memory-rank sort ordering is sufficient
  # Constructive: memory-rank sort (Algorithm 1) re-orders input so parent always processed before children

Theorem 5.2 (RNN lower bound, adversarial order):
  assert rnn_hidden_dim_needed_adversarial_order >= c * n  # O(n) hidden dim needed if ordering is adversarial

Theorem 5.5 (RNN lower bound, any order):
  assert constant_dim_rnn_cannot_solve_CRQ  # hidden dim must grow with n regardless of order

Theorem 6.1 (CoT transformer):
  assert cot_tokens_needed >= n  # O(n) CoT tokens sufficient; O(log n) CoT tokens provably insufficient

Complexity (Table 1, tree depth = log n):
Deep transformer  | params: O(L * log^2 n)    | runtime: O(L * n^2 * log n)    | parallel: O(L) [BEST parallelism]
RNN benevolent    | params: O(log n) to O(n)  | runtime: O(n log n) to O(n^2)  | parallel: O(n) [sequential bottleneck]
Shallow+CoT       | params: O(log^2 n) [BEST] | runtime: O(n^2 * log^2 n)      | parallel: O(n) [not parallelizable]

Resource tradeoff summary:
- Deep transformer: minimizes parallel runtime (O(L)), costs logarithmic depth growth
- Shallow RNN: minimizes parameters under benevolent ordering, costs sequential runtime
- CoT transformer: minimizes parameter count (O(log^2 n)), costs linear tokens and sequential generation

Comparison to Feng et al. 2024 (arithmetic CoT):
- This paper needs only O(n) CoT tokens; Feng et al. need Omega(n^2) for arithmetic
- This paper solves CRQ with O(log n) deep transformer; Feng et al. show RNN needs Omega(n / log n) hidden dim
- Difference attributed to explicit tree positional encodings vs parentheses-encoded structure

Failure modes:
- Constant-depth transformers fail on CRQs of arbitrary depth (NC^1-hardness argument via TC^0 containment)
- RNNs with constant hidden dimension fail universally, even with optimal ordering
- O(log n) CoT tokens are insufficient; the lower bound is tight -- exactly O(n) tokens needed
- RNNs with adversarial input ordering require O(n) hidden dim even though O(log n) suffices under memory-rank sort

Implementation pitfalls (do NOT do this):
- Do not use softmax attention for the impossibility results -- hardmax is used throughout; softmax is strictly stronger
- Do not assume benevolent ordering is always available -- in practice token order is fixed and may be adversarial
- Do not conflate "deep transformer needs log-depth" with "size grows linearly" -- parameter count is O(L * log^2 n), mild
- Do not ignore positional encodings carrying tree structure -- results depend on z_v, z_{P(v)}, depth(v) being available
- Do not apply Feng et al. CoT bound (Omega(n^2)) to CRQs -- CRQs use tree PEs, not parentheses, so n tokens suffice
- Do not claim shallow transformers can solve CRQs without CoT -- Theorem 4.3 is a hard impossibility
- Do not use looped/universal transformers without acknowledging they are a separate model class not covered here

Relation to arXiv:2507.07505 (Hallucination Stations, O(N^2*d) ceiling thesis):
EXTENDS. The target paper counts FLOPs per forward pass (O(N^2*d) ceiling from full attention)
and argues this bounds what a standard transformer can compute per pass. This paper adds:
(1) Proves CoT-augmented transformers break the per-pass ceiling by making it a token-count
    problem instead: O(n) scratchpad tokens let a constant-size model compute NC^1 functions
    that no constant-depth single-pass transformer can compute.
(2) Quantifies the exact CoT token cost: linear O(n) is both necessary and sufficient for CRQs;
    the target's claim that CoT extends expressiveness is confirmed AND the linear price tag is
    established (target lacks this).
(3) Provides an alternative route (deep transformer, O(log n) depth) that avoids CoT entirely
    and achieves parallel runtime O(L), showing the O(N^2*d) FLOP ceiling is architecture-
    dependent: depth-L transformers pay O(L * n^2 * log n) sequential FLOPs but O(L) parallel.
(4) Does NOT contradict the target: the ceiling thesis applies to constant-depth, no-CoT
    transformers, which this paper also proves are incapable of CRQs.
