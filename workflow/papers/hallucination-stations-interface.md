Algorithm: Complexity-Theoretic Hallucination Threshold for LLMs and LLM-Based Agents
ArXiv: 2507.07505
Paper: Hallucination Stations: On Some Basic Limitations of Transformer-Based Language Models
Authors: Varin Sikka (Stanford University), Vishal Sikka (VianAI Systems)
Date: July 10, 2025 (v1); July 15, 2025 (v3)
Venue: arXiv preprint (not peer-reviewed as of ingestion date 2026-06-04)
Reliability: MEDIUM (arXiv preprint, not peer-reviewed; empirical measurement cited for FLOP count claim; proof invokes Hartmanis-Stearns 1965 which is established theory)

Core Problem:
LLMs and LLM-based agents are deployed on tasks of arbitrary computational complexity, but the inference mechanism itself has a fixed complexity ceiling. The paper asks: what classes of tasks are provably outside an LLM's capability, and can a second LLM-based agent verify the correctness of the first agent's output on those tasks?

Proposed Method / Theoretical Framework:
The paper characterizes LLM inference complexity as O(N^2 * d) per forward pass, where N = input token count and d = embedding dimension. It then invokes the Hartmanis-Stearns time-hierarchy theorem to prove that any task encoded in a prompt whose computational complexity exceeds O(N^2 * d) cannot be correctly solved by the LLM. The same bound applies to verification: a verifier LLM faces identical complexity constraints, so it cannot verify tasks that exceed its own threshold.

Inputs (to the theoretical model):
N | integer | scalar | number of tokens in the input prompt; must be >= 1
d | integer | scalar | embedding dimension of the LLM; must be >= 1
n | integer | scalar | problem-size parameter embedded within the prompt; must be < N (n is a substring parameter, always smaller than the full prompt length N)
task_complexity | symbolic | O(f(n)) | complexity class of the computational task expressed in the prompt

Outputs (theoretical claims):
hallucination_verdict | boolean | true if task_complexity > O(N^2 * d), false otherwise
correctness_guarantee | none | no output correctness guarantee exists when task_complexity > O(N^2 * d)

Hyperparameters / Model Properties:
N (context length) | varies | 1..1,000,000+ | Modern LLMs (Gemini, GPT-4.1) have windows >= 1M tokens; larger N raises the threshold but does not eliminate it
d (embedding dim) | varies | typically 512..65536 | scales the threshold but polynomially, not super-polynomially
reasoning_tokens | varies | 0..large | reasoning models (o3, R1) add think-step tokens; paper argues this does not close the complexity gap (conjecture, not formally proven in this paper)

Critical Theorem (Theorem 1, p.4):
Given a prompt of length N containing a task of complexity O(n^k) or higher where k >= 3 and n < N, an LLM or LLM-based agent will unavoidably hallucinate in its response.

Proof basis: Hartmanis-Stearns time-hierarchy theorem (1965, Trans. Amer. Math. Soc. 117, 285-306). If t2(n) is asymptotically larger than t1(n), there exist decision problems solvable in O(t2(n)) but not in O(t1(n)). Since O(n^3) > O(N^2 * d) for sufficiently large n, tasks of cubic or higher complexity in the embedded problem size exceed the LLM threshold.

Corollary: Verification of tasks of complexity > O(N^2 * d) is also outside LLM capability, because accurate verification itself requires at least the same complexity as solving the original task.

Critical Invariants (testable assertions):
assert task_flops > (N**2 * d) implies llm_will_hallucinate  # Theorem 1
assert verifier_flop_budget == N**2 * d  # verifier is also an LLM, same ceiling
assert n < N  # n is problem-size within the prompt, always a substring
assert llama_3_2_3b_flops_for_17_tokens == 109243372873  # empirically measured (p.2, ref [7])
assert reasoning_tokens_do_not_close_gap  # stated as author belief, not formally proven; treat as UNGROUNDED CONJECTURE until ref [29] Apple paper verified

Complexity:
LLM forward pass time: O(N^2 * d) per token generated
Self-attention specifically: O(N^2 * d + N * d^2) (both quadratic-in-N and quadratic-in-d terms)
Threshold: task is solvable iff task_complexity <= O(N^2 * d)
Parallelism: paper does not address parallel inference or multi-agent parallelism as a complexity mitigation

Examples of tasks exceeding threshold:
Token composition enumeration: O(n^k) for vocab size n, length k; for n=2 requires O(2^k)
Matrix multiplication (naive): O(m*n*p) = O(n^3) for square matrices; exceeds threshold when n,m,p > vocab size
TSP verification (brute-force): O((n-1)!/2); for 20 cities = approx 10^17 candidate routes (p.3)
Floyd-Warshall all-pairs shortest paths: O(n^3) in number of nodes
Subset enumeration: O(2^n)
Navier-Stokes computational fluid dynamics: exponential in mesh resolution
Model checking (state explosion): exponential in number of components/variables

Failure Modes (cited by authors):
1. LLM may produce a response that LOOKS correct but was computed via a wrong path (accidental correctness); both necessarily-wrong and accidentally-correct-but-wrong-process are classified as hallucinations (p.5)
2. Multi-agent verification does not escape the limitation: A2 verifying A1 faces the same complexity ceiling as A1 (p.4)
3. Reasoning models (o3, R1) generate think-step tokens but: (a) each token generation step still has complexity O(N^2*d), (b) token budget for reasoning is far smaller than required for truly complex tasks (p.5, supported by Apple Illusion of Thinking paper ref [29])
4. Approximate/heuristic verification may give incorrect verdicts for tasks where exact verification is required (TSP, model checking)

Implementation Pitfalls (do NOT do this):
Do NOT assume LLM output correctness on tasks of O(n^3) or higher complexity in the problem-size parameter n embedded in the prompt
Do NOT use a second LLM agent as a ground-truth verifier for tasks beyond the O(N^2*d) threshold; this provides a false sense of verification
Do NOT conflate reasoning-model chain-of-thought length with genuine computational capacity increase; the per-step ceiling is unchanged
Do NOT treat N (full prompt token count) and n (problem-size parameter within prompt) as interchangeable; n < N always, and the relevant complexity bound on the task is in n, not N
Do NOT assume the complexity bound scales away with larger context windows; growing N raises the threshold polynomially (O(N^2*d)) but exponential/factorial task complexity in n will still exceed it for large enough n

Limitations acknowledged by authors:
- Paper does not formally analyze whether reasoning-model think tokens close the gap; deferred to subsequent work (p.5)
- The complexity argument is about the minimum number of steps required, not about whether LLMs find approximations or heuristics (practical applications may tolerate approximate answers)
- Multiple LLMs working together may achieve higher effective capability (Simon/Minsky collective intelligence framing); this is noted but not formally bounded
- The paper does not provide empirical benchmarks beyond the single FLOP-count measurement for Llama-3.2-3B-Instruct on 17-token inputs (ref [7], external link to author's GitHub, not included in paper body)
- No formal treatment of approximation algorithms or probabilistic correctness (only exact/complete correctness is addressed)

Known debates / open questions:
- Whether the O(N^2*d) complexity is a tight bound or a worst-case characterization; sparse attention variants (e.g. linear attention) have O(N*d) but are not standard
- Whether compound AI systems (ref [23]) or neurosymbolic augmentation (Cyc ref [26], AlphaGeometry ref [25]) structurally escape the theorem or merely raise the effective threshold
