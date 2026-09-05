Algorithm: Compute-Optimal Test-Time Scaling (CO-TTS)
ArXiv: 2502.06703
Paper: Can 1B LLM Surpass 405B LLM? Rethinking Compute-Optimal Test-Time Scaling
Authors: Runze Liu, Junqi Gao, Jian Zhao, Kaiyan Zhang, Xiu Li, Biqing Qi, Wanli Ouyang, Bowen Zhou (Shanghai AI Lab / Tsinghua, 2025)
Datasets evaluated: MATH-500, AIME24

---
SPECIFIC QUESTION (B-39): Is N=4 in the optimal range for Qwen2.5-Coder-3B/7B?
SHORT ANSWER: NO. N=4 is the minimum budget tested and lies well below the Pareto elbow.
See "B-39 Verdict" section below.
---

Inputs:
policy_model | LLM | any size (paper: 0.5B to 72B, Qwen2.5 + Llama families) | mandatory
process_reward_model (PRM) | verifier LLM | 1.5B to 72B tested | mandatory; performance is PRM-dependent
compute_budget_N | int | {4, 16, 64, 256} standard range; 512 tested for 1B | mandatory
problem_x | text prompt | math reasoning problem | mandatory
difficulty_tier | enum | easy=Pass@1 50-100%, medium=10-50%, hard=0-10% | optional; used for adaptive allocation
tts_method | enum | BoN, BeamSearch, DVTS | chosen per model-size + difficulty tier
scoring_method | enum | PRM-Min, PRM-Last, PRM-Avg | affects PRM selection quality
voting_method | enum | MajorityVote, PRM-Max, PRM-Vote | PRM-Vote sensitive per PRM family

Outputs:
selected_answer | text | single solution to problem_x | must be one of the N generated responses
accuracy_on_MATH500 | float | [0.0, 1.0] | improves with N; plateaus depend on model size and PRM
accuracy_on_AIME24 | float | [0.0, 1.0] | harder benchmark; smaller models need N>64 for nonzero gains
efficiency_gain_vs_majvote | float | ratio >= 0.8 | Qwen2.5-3B: 58.4x; Qwen2.5-7B: 35.9x; Qwen2.5-32B: 0.8x

Hyperparameters:
N (compute_budget) | no single default | {4,16,64,256}; 512 tested | HIGH sensitivity; primary knob for performance
beam_size_M | 4 (fixed in paper) | 2-8 typical | used only with BeamSearch and DVTS
difficulty_threshold_easy | 0.5 Pass@1 | 0.3-0.7 | absolute criterion beats quantiles (Section 3.2)
difficulty_threshold_hard | 0.1 Pass@1 | 0.05-0.2 | problems below 10% Pass@1 are hard tier
PRM_family | Qwen2.5-Math series | depends on policy model | CRITICAL: must match policy model family; cross-family PRMs degrade to below majority vote
max_tokens_per_step | 2048 | fixed in paper | applies to search methods
max_tokens_total | 8192 | fixed in paper | applies to BoN

Performance table (Table 5, MATH-500, compute-optimal strategy):
Model       | CoT  | MajVote | CO-TTS | Gain   | Efficiency
Llama-3.2-1B| 26.0 | 39.0    | 66.2   | 154.6% | >256.0x
Llama-3.2-3B| 41.4 | 58.4    | 78.2   | 88.9%  | 14.1x
Llama-3.1-8B| 49.8 | 66.4    | 80.6   | 61.8%  | 43.9x
Qwen2.5-0.5B| 31.6 | 47.2    | 76.4   | 141.8% | >64.0x
Qwen2.5-1.5B| 54.4 | 68.4    | 85.6   | 57.4%  | >256.0x
Qwen2.5-3B  | 64.0 | 77.0    | 87.6   | 36.9%  | 58.4x
Qwen2.5-7B  | 76.8 | 83.6    | 91.0   | 18.5%  | 35.9x
Qwen2.5-14B | 80.2 | 85.6    | 91.0   | 13.5%  | 51.4x
Qwen2.5-32B | 82.4 | 87.0    | 90.6   | 10.0%  | 0.8x
Qwen2.5-72B | 83.8 | 87.2    | 91.8   | 9.5%   | 12.9x

Extended N test (Table 3):
Llama-3.2-1B N=512: MATH-500=72.2%, AIME24=10.0% — shows 1B needs N>256

Optimal TTS method by model size and difficulty (Section 4.3):
<=3B, easy: BoN preferred
<=3B, medium/hard: BeamSearch preferred (verifier guides each step)
7B-32B, easy/medium: DVTS preferred
7B-32B, hard: BeamSearch preferred
>=72B, all: BoN preferred

Critical invariants (testable assertions):
assert N in [4, 8, 16, 32, 64, 128, 256, 512], "N must be power-of-2 in paper's tested range"
assert tts_method in ("bon", "beam_search", "dvts"), "only these three methods validated"
assert 0 < beam_size_M <= N, "beam_size must divide N evenly for DVTS subtrees"
assert N % beam_size_M == 0, "DVTS requires N/M whole subtrees (Section 2.2)"
assert difficulty_threshold_hard < difficulty_threshold_easy, "hard < easy threshold"
assert prm_family_matches_policy_model, "cross-family PRM degrades below majority vote baseline"

B-39 VERDICT: Is N=4 optimal for Qwen2.5-Coder-3B/7B?
--------------------------------------------------------------------------------
N=4 is NOT in the optimal range. Evidence:

1. N range tested: {4, 16, 64, 256}. N=4 is the minimum budget, not the elbow.
   The paper never recommends N=4 as a compute-optimal setting for any model.

2. For Qwen2.5-3B: efficiency gain is 58.4x over majority voting. Majority voting
   in the paper's experiments runs at N=64 (Figure 4 x-axis maximum for standard plots).
   58.4x efficiency means CO-TTS at its optimal N achieves same accuracy as
   MajVote@N_high using far fewer samples — but the OPTIMAL N for CO-TTS itself
   is NOT N=4; it is the N where the Pareto curve bends for the given difficulty tier.

3. For small models (<=3B), the paper explicitly states search-based methods
   outperform BoN. BoN at N=4 generates only 4 candidates — insufficient for
   beam search or DVTS to explore meaningfully. BeamSearch requires at minimum
   N/M = 4/4 = 1 step per branch, which is degenerate. Practical minimum for
   beam search usefulness: N >= 16 (4 beams x 4 steps).

4. For 7B models: BoN becomes more competitive with larger models, but N=4 BoN
   still produces only 4 candidates. At N=64, Qwen2.5-7B with best PRM shows
   significantly higher accuracy than at N=4 (Figure 4 shows monotonic improvement
   across {4, 16, 64, 256} for well-matched PRMs).

5. Pareto elbow for 3B-7B: The paper does not name an explicit elbow N.
   Based on the efficiency data and the fact that diminishing returns set in for
   models with strong reasoning (32B shows 0.8x efficiency, meaning N=4 majority
   vote is already at the efficient frontier for 32B), the elbow for 3B-7B
   lies in the N=16..64 range:
   - For easy problems: N=16 likely sufficient (BoN, 3B-7B)
   - For medium problems: N=32..64 (beam search, 3B) or N=16 (DVTS, 7B)
   - For hard problems: N=64..256 (beam search, both 3B and 7B)

6. B-39 RECOMMENDED DEFAULT: N=16 as minimum viable, N=32 as practical default
   for Qwen2.5-Coder-3B/7B with a generative PRM verifier.
   N=4 should only be used if latency is the hard constraint and accuracy loss
   is acceptable (expect ~5-15% accuracy degradation vs N=16 for medium tasks).

Generative PRM note: The paper uses discriminative PRMs (Math-Shepherd, RLHFlow,
Skywork, Qwen2.5-Math). It does NOT study "generative process reward verifiers"
(models that generate step-by-step critiques). The term "generative PRM" is not
used in this paper. However, the Qwen2.5-Math PRM family (which uses a
generative-style scoring head) shows the best generalization across model families.
If B-39 uses a generative verifier (e.g., a model prompted to judge steps),
treat it as analogous to a strong discriminative PRM — the N recommendation
above still applies.

Complexity:
Time per token generation: O(N * L * d^2) where L=sequence length, d=model dim
Space: O(N * L) for storing N candidate sequences in parallel
BeamSearch: O(depth * M * branching_factor) tree nodes; depth = max_steps
DVTS: N/M independent subtrees; embarrassingly parallel across subtrees
Parallelism: BoN is fully parallel (N independent samples); BeamSearch is depth-sequential

Failure modes:
PRM-policy mismatch: using Math-Shepherd PRM with Qwen2.5 policy degrades below majority vote
N=4 with BeamSearch or DVTS: degenerate tree (1 branch, 1 step); effectively greedy decoding
N=4 for hard problems (0-10% Pass@1): insufficient exploration; beam search needs N>=16
Length bias: PRMs favor longer step sequences regardless of correctness (Section 4.4)
Voting sensitivity: Skywork PRM requires PRM-Vote; using PRM-Max degrades it significantly
Quantile-based difficulty bucketing: cross-model inconsistency; use absolute Pass@1 thresholds
Strong models at high N: Qwen2.5-32B has 0.8x efficiency at N=256, meaning compute is WASTED

Implementation pitfalls (do NOT do this):
Do NOT use N=4 as compute-optimal default for <7B models — it is the minimum tested budget
Do NOT mix PRM families across policy model families without validation
Do NOT use quantile-based difficulty binning (Section 3.2 explicitly rejects this)
Do NOT use BeamSearch with beam_size_M > N (zero subtrees)
Do NOT use a single fixed N across all problem difficulties — easy problems saturate faster
Do NOT apply DVTS settings designed for 7B-32B to <3B models — search outperforms BoN only when model has base reasoning capability
Do NOT assume efficiency gain in Table 5 means N_optimal = N_majvote / efficiency_ratio; formula is not stated and may not hold
Do NOT use majority voting as the only baseline; compute-optimal TTS uses PRM-guided selection

Related work (Snell et al. 2024, arXiv:2408.03314):
The prior work (PaLM 2-S only) showed 4x compute reduction is achievable via difficulty-adaptive TTS. Liu et al. 2025 extends this to modern open-source models 0.5B-72B and shows the improvement is model-size and PRM dependent.
