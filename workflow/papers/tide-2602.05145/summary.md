TIDE: Temporal Incremental Draft Engine for Self-Improving LLM Inference
arXiv: 2602.05145 | February 5, 2026
Authors: Jiyoung Park, Hankyu Jang, Changseok Song, Wookeun Jung
Affiliation: MOREH (logo in Figure 1)
Categories: cs.LG, cs.AI
License: CC BY 4.0
DOI: 10.48550/arXiv.2602.05145


1. PROBLEM STATEMENT

Speculative decoding accelerates LLM inference by having a small draft model propose candidate tokens verified in parallel by the target model. However:
- Draft models degrade 15-40% when evaluated on distributions different from their training data (Table 3 cross-dataset results).
- Static draft models cannot adapt to evolving workloads at runtime.
- Existing online adaptation approaches (e.g. SpecForge) require recomputing target model hidden states for training, which is redundant and expensive.
- System-level constraints (batch size, latency ratios) determine whether speculation is even beneficial, but static systems cannot adapt.


2. TIDE FRAMEWORK OVERVIEW

TIDE is a serving-engine-native framework with two decoupled engines:

A. Inference Serving Engine
   Components:
   - Target Model: runs normal autoregressive or speculative decoding
   - Draft Model: EAGLE-3 architecture (single decoder layer + LM head)
   - Adaptive Drafter: monitors batch size and acceptance rates, enables/disables speculation
   - Acceptance Length Monitor: tracks per-request acceptance metrics with dual EMA
   - Training Signal Extractor: selectively captures and stores hidden states from target model

B. Draft Model Training Engine
   - Operates asynchronously on separate GPUs (possibly different GPU class)
   - Monitors shared storage for accumulated training signals
   - Triggers training when signal threshold exceeded
   - Validates improvement via train-eval split before deploying updated draft model

Figure 2 (x2.png): Architecture diagram showing the two-engine design.
- Target Model feeds hidden states to Training Signal Extractor
- Training Signal Extractor writes to Shared Storage
- Acceptance Length Monitor receives acceptance length data and sends start/end signals to the extractor
- Acceptance Length Monitor sends threshold to Adaptive Drafter
- Adaptive Drafter controls Draft Model on/off
- Draft Model Training Engine reads from Shared Storage, trains draft model, sends update back to Inference Engine's Draft Model

Figure 3 (x3.png): Detailed architecture with 4-phase Acceptance Length Monitor cycle:
  Phase 1 (decrease): acceptance length declining, indicates distribution shift
  Phase 2 (below threshold): acceptance drops below threshold, speculation disabled
  Phase 3 (above threshold): after retraining, acceptance rises, speculation re-enabled
  Phase 4 (converge): acceptance stabilizes, training signal collection paused
All four phases write to shared storage. Online Trainer in the Training Engine triggers from storage, trains draft model through multiple parameter versions.


3. ZERO-OVERHEAD TRAINING SIGNAL EXTRACTION

Figure 4 (x4.png): CUDA stream timeline showing overlapped execution.
- Top timeline: CUDA streams process draft tokens (d1-d5), verification tokens (v1-v5), with hidden states (h1-h5)
- Device-to-host transfer of hidden states overlaps with subsequent verification compute on CUDA streams
- Hidden states are flushed to preallocated host buffer asynchronously
- Once buffer is full, flush to shared storage and trigger training asynchronously
- Key insight: the target model ALREADY computes hidden states during verification — TIDE reuses them instead of recomputing

This eliminates the redundant forward pass that SpecForge requires for generating training signals.


4. ADAPTIVE SPECULATIVE DECODING CONTROL

Practical Speedup Model (Equation 5):

  PracticalSpeedup = (1 - alpha^(gamma+1)) / ((1 - alpha) * (c(b)*gamma + beta(b)))

Where:
  alpha = token acceptance rate (measured at runtime)
  gamma = number of candidate tokens (fixed at 3, see Table 4)
  c(b) = draft-to-target latency ratio as function of batch size b
  beta(b) = verification latency ratio accounting for compute-bound regime at batch size b

Key observations from Figure 5 (x5.png):
- Shows ratio of verification latency (gamma=3 candidates) to single-token decoding latency across batch sizes
- For gpt-oss-120b: ratio stays near 1.0 for small batches (memory-bound), rises sharply above batch ~48 (compute-bound)
- For Qwen3-235B-A22B (MoE): ratio stays near 1.0 much longer due to sparse activation
- For Llama-4-Scout-17B-16E: similar MoE behavior
- For Llama-3.3-70B-Instruct: rises earlier due to smaller model

Profiling at initialization: TIDE profiles T(n) (target latency) and D0 (draft latency) across batch sizes to build the c(b) and beta(b) curves.

Decision rule: speculation is enabled when PracticalSpeedup > 1.0, disabled otherwise. This is evaluated continuously as batch size and acceptance rates change.


5. SELECTIVE TRAINING CONTROL (Algorithm 1)

Maintains dual exponential moving averages:
  alpha_short (short-term EMA): responds quickly to recent acceptance rate changes
  alpha_long (long-term EMA): captures overall trend

Training signal collection logic:
  - Distribution shift detected: when alpha_short drops significantly below alpha_long
  - Training triggered: when accumulated signals in shared storage exceed threshold
  - Collection paused: when alpha_short converges back to alpha_long (improvement plateaus)
  - This prevents wasteful continuous training when the draft model is already well-adapted


6. HETEROGENEOUS GPU CLUSTER EXPLOITATION

Figure 12 (x12.png): Per-GPU throughput comparison across GPU classes.
  Inference Throughput (normalized to MI250):
    MI250: 1.00x
    MI300X: 4.42x
    H100: 6.76x

  Training Throughput (normalized to MI250):
    MI250: 1.00x
    MI300X: 1.77x
    H100: 2.44x

Key insight: the inference-training throughput gap is much larger (6.76x vs 2.44x for H100 vs MI250). This means:
- High-end GPUs (H100) should be allocated to inference (where the gap is largest)
- Lower-tier GPUs (MI250) can handle training (where the gap is smaller)
- This heterogeneous allocation maximizes total system throughput

Figure 13 (x13.png): Relative throughput across different GPU configurations (High, Low, spec-enabled variations). Shows that heterogeneous allocation (H100 inference + MI250 training) outperforms homogeneous allocation.


7. EXPERIMENTAL SETUP

Target Models:
  - gpt-oss-120b (proprietary dense model)
  - Qwen3-235B-A22B (MoE, 22B active params)
  - Llama-4-Scout-17B-16E (MoE)
  - Llama-3.3-70B-Instruct (dense)

Draft Model Architecture: EAGLE-3 (single decoder layer + LM head on top of target hidden states)

Datasets:
  - ShareGPT: open-ended conversational data
  - CAMEL-AI Science: physics, chemistry, biology Q&A
  - NuminaMath: mathematical reasoning
  - EvolCodeAlpaca: code generation
  - Multilingual Alpaca: Korean, Arabic, Chinese, French

Candidate Token Configuration: gamma=3 fixed (Table 4 shows gamma=3 optimal across models)

Infrastructure: Built on SGLang and vLLM frameworks. Training uses SpecForge pipeline with PyTorch FSDP.


8. RESULTS

8A. Accept Length Evolution (Figure 6, x6.png)
  Plot of acceptance length vs. training time steps for gpt-oss-120b:
  - NuminaMath: highest acceptance, starts ~2.4, rises to ~2.7 by step 500
  - EvolCodeAlpaca: starts ~2.3, rises to ~2.4 by step 1000
  - Science: starts ~2.0, rises to ~2.35 by step 1500
  - ShareGPT: starts ~2.0, rises to ~2.2 (smallest improvement, open-ended nature)
  All curves show monotonic improvement with diminishing returns.

8B. Throughput Evolution (Figure 7, x7.png)
  Four subplots showing throughput (tokens/sec) vs. time step for gpt-oss-120b:
  - ShareGPT: fluctuates around 5800-6200, minimal trend (high variance)
  - Science: starts ~6000, rises to ~6500 by step 400, stabilizes
  - EvolCodeAlpaca: starts ~6200, rises to ~6500 by step 400
  - NuminaMath: starts ~6000, rises to ~6500 with high variance

8C. Training Efficiency (Table 2)
  For gpt-oss-120b on ShareGPT (100k dataset):

  Method              | Signal Gen | Training | Total   | Storage
  --------------------|-----------|----------|---------|--------
  SpecForge offline   | 9.20 hr   | 6.12 hr  | 15.32hr | 4.66 TB
  SpecForge online    | 21.52 hr  | 6.12 hr  | 27.64hr | 4.66 TB
  TIDE                | 0 hr      | 9.16 hr  | 9.16hr  | 0.19 TB

  TIDE achieves:
  - 1.67x faster than SpecForge offline (9.16 vs 15.32 hours)
  - 3.02x faster than SpecForge online (9.16 vs 27.64 hours)
  - 25x storage reduction (0.19 TB vs 4.66 TB)
  - Zero signal generation time (reuses inference hidden states)

8D. Draft Training Accuracy (Figure 8, x8.png)
  Comparison of SpecForge offline vs TIDE across 4 models:
  - Training accuracy curves are comparable between methods
  - TIDE achieves similar final accuracy despite using only inference-generated signals
  - Slight differences in convergence speed vary by model

8E. Practical Speedup Prediction (Figure 9, x9.png)
  Bar charts comparing predicted vs actual speedup across batch sizes:
  - gpt-oss-120b: prediction within 3% error
  - Qwen3-235B-A22B: prediction within 3% error
  - Llama-4-Scout-17B-16E: up to 25% error (draft model overhead non-negligible for smaller models)
  - Llama-3.3-70B-Instruct: similar higher error for smaller model

8F. Adaptive Control Under Distribution Shift (Figure 10, x10.png)
  Two time-series plots over 2000 time steps:
  - TIDE-default (top): throughput fluctuates between 3000-4500, drops sharply at step ~500, ~1000, ~1500 (distribution shifts), dashed red threshold line at ~3800
  - TIDE-adaptive (bottom): throughput stays stable around 3500-4000 throughout shifts, no sharp drops
  Demonstrates that adaptive control smooths out distribution shift impacts.

8G. Heterogeneous Cluster Results (Figure 11, x11.png)
  Bar chart of relative throughput (normalized to all-inference baseline):
  - ShareGPT: TIDE 1.08x (gamma=1.15)
  - Science: TIDE 1.22x (gamma=1.3)
  - EvolCodeAlpaca: TIDE 1.17x (gamma=1.23)
  - NuminaMath: TIDE 1.21x (gamma=1.28)
  Improvement correlates with speculative decoding effectiveness per dataset.

8H. Cross-Dataset Generalization (Table 3)
  Draft models trained on one dataset and evaluated on another show 15-40% degradation in acceptance length. This validates the need for runtime adaptation. Specific numbers not fully extracted but the pattern is consistent across all model-dataset pairs.


9. CANDIDATE TOKEN COUNT ANALYSIS (Table 4)

gamma | Speedup (gpt-oss-120b) | Notes
------|------------------------|------
1     | lowest                 | too few candidates
2     | moderate               | suboptimal
3     | highest                | optimal for all tested models
4     | decreasing             | verification cost exceeds benefit
5     | further decrease       | diminishing returns

gamma=3 is optimal because it balances acceptance probability against verification cost.


10. IMPLEMENTATION DETAILS

- Inference engines: SGLang and vLLM (production LLM serving frameworks)
- Training: SpecForge training pipeline with PyTorch FSDP (Fully Sharded Data Parallel)
- Profiling: at initialization, TIDE profiles target model latency T(n) and base draft latency D0 across a range of batch sizes
- Model deployment: trained draft model validated via train-eval split before hot-swapping into inference engine
- Signal storage: shared filesystem between inference and training engines (host memory buffer -> shared storage)


11. LIMITATIONS

1. Prediction model assumes negligible draft model overhead relative to target model. Violated for larger draft models or smaller target models (explains 25% prediction error for Llama models).
2. Benefits are limited when speculative decoding is inherently unsuitable (high-entropy conversational workloads like ShareGPT show minimal improvement).
3. Train-eval validation step before deployment adds overhead.
4. Requires heterogeneous GPU cluster to fully exploit decoupled architecture (though works on homogeneous too with reduced benefit).


12. KEY REFERENCES AND PRIOR ART

- Speculative decoding: Leviathan et al. 2023 (foundational)
- EAGLE-3: Li et al. 2025b (draft model architecture used by TIDE)
- SpecForge: (offline draft model training baseline)
- Online adaptation: Liu et al. 2024
- Heterogeneous inference: DistServe, HexGen-2
- Serving frameworks: SGLang, vLLM


13. FIGURE INDEX

x1.png  - MOREH company logo
x2.png  - TIDE architecture overview (two-engine design with data flow)
x3.png  - Detailed architecture with 4-phase acceptance monitor cycle
x4.png  - CUDA stream timeline showing zero-overhead hidden state extraction
x5.png  - Verification/decoding latency ratio vs batch size (4 models)
x6.png  - Accept length evolution during training (4 datasets, gpt-oss-120b)
x7.png  - Throughput evolution over time (4 datasets, gpt-oss-120b)
x8.png  - Training accuracy: SpecForge offline vs TIDE (4 models)
x9.png  - Predicted vs actual speedup across batch sizes (4 models)
x10.png - TIDE-default vs TIDE-adaptive under distribution shifts
x11.png - Heterogeneous cluster relative throughput (4 datasets)
x12.png - Per-GPU throughput: inference vs training across MI250/MI300X/H100
x13.png - GPU configuration comparison for heterogeneous allocation
