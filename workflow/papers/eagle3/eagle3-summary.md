EAGLE-3: Scaling up Inference Acceleration of Large Language Models via Training-Time Test
arXiv: 2503.01840v3 [cs.CL] — 23 Apr 2025
Authors: Yuhui Li (Peking U / U Waterloo), Fangyun Wei (Microsoft Research), Chao Zhang (Peking U), Hongyang Zhang (U Waterloo / Vector Institute)
Code: https://github.com/SafeAILab/EAGLE

1. Problem Statement

LLM inference is memory-bandwidth-bound: each token requires accessing all model parameters. Speculative sampling mitigates this by drafting multiple tokens cheaply, then verifying them in parallel via the target model. EAGLE (prior work) improved drafting by reusing top-layer features of the target model and performing feature-level autoregression. However, scaling up training data for EAGLE's draft model yields diminishing returns. EAGLE-3 identifies why and fixes it.

2. Root Cause Analysis: Why EAGLE Doesn't Scale

EAGLE's draft model has dual loss: feature prediction loss (l_fea) + token prediction loss (l_token). The feature prediction loss constrains the draft model to predict the target model's top-layer feature vector. This acts as an unnecessary bottleneck:

- Feature prediction is an additional constraint that limits expressiveness of the draft model.
- More training data cannot overcome this structural limitation.

Removing l_fea alone creates a new problem: the draft model output a_hat_{t+1} in Step 1 diverges significantly from the ground-truth feature f_{t+1}. When a_hat_{t+1} is fed as input in Step 2, the input distribution deviates from training distribution, causing catastrophically low acceptance rates for the 2nd+ draft tokens.

Evidence (Figure 4/x5/x6):
- 0-alpha (first token acceptance): removing fea prediction improves it significantly (yellow line "EAGLE without fea pred" climbs from ~0.74 to ~0.81 as data scales 1x to 8x).
- 1-alpha (second token acceptance): without fea prediction, drops to ~0.20-0.30 (yellow line in x6), far below EAGLE's ~0.67-0.69 (red line). This is the distribution shift catastrophe.
- EAGLE-3 (blue line) maintains 1-alpha at ~0.70-0.78 by solving the distribution shift via training-time test.

3. EAGLE-3 Architecture: Two Key Innovations

3.1 Innovation 1: Training-Time Test

Core idea: simulate multi-step drafting during training so the model learns to handle its own (noisy) predictions as input, not just ground-truth features.

During training with a 3-token sequence "How can I":
- Step 1 (normal): input features from target model, output predictions a_{t+1}. Loss = l_token only.
- Step 2 (simulated test): feed a_{t+1} (the model's own output from Step 1) back as input, alongside the next embedding. The draft model must now predict correctly despite receiving its own noisy estimate instead of ground-truth.
- Step 3+: continue chaining, always using the model's own prior outputs.

Attention mask modification for training-time test:
- Step 1 uses standard lower-triangular causal mask.
- Step 2+: attention masks become diagonal (each predicted token only attends to its own chain), except when the key comes from the original training data (ground-truth positions), where normal causal attention applies.
- Implementation: vector dot products instead of matrix multiplication for the diagonal portions to avoid computational waste.

This eliminates the train/test distribution mismatch. The model is trained on the same kind of inputs it will see at inference time.

Difference from HASS: HASS also simulates multi-step during training, but retains feature prediction loss l_fea and requires top-layer features. EAGLE-3 removes l_fea entirely, freeing the input representation. EAGLE-3's speedup significantly exceeds HASS (see Figure 2/x3).

3.2 Innovation 2: Multi-Layer Feature Fusion

Instead of reusing only the top-layer features (which are optimized for next-token prediction and thus poorly suited for predicting tokens beyond the next one), EAGLE-3 fuses features from three layers:

- l: low-level features (early decoder layers)
- m: mid-level features (middle decoder layers)
- h: high-level features (late decoder layers)

Fusion mechanism:
1. Extract k-dimensional vectors l, m, h from three target model layers.
2. Concatenate into 3k-dimensional vector.
3. Pass through a fully connected (FC) layer to reduce back to k dimensions.
4. Result: fused feature g that captures rich semantic information from different abstraction levels.

The removal of l_fea is what enables this fusion — with feature prediction, the draft model was forced to match the top-layer features specifically, precluding the use of other layers.

4. Inference Pipeline (Figure 5/x7)

Given prefix "How can" and sampled token "I":

Step 1:
- Reuse g_how and g_can from target model (fused low/mid/high features).
- Concatenate with embedding e_I of the sampled token "I".
- Pass through FC layer (reduce to k dims) -> single Transformer decoder layer -> output a.
- Feed a_I into target model's LM head -> sample draft token "do".

Step 2:
- Prefix is now "How can I". Need g_I but can't get it (target model hasn't verified "I" yet).
- Replace g_I with a_I (draft model output from Step 1).
- Concatenate a_I with e_do -> FC -> decoder -> output a.
- Feed a_do into LM head -> sample next draft token "it".

Step 3+:
- Same pattern: replace unavailable target features with draft model outputs from prior steps.

Draft model architecture: a single Transformer decoder layer (lightweight).

Draft tree: EAGLE-3 uses EAGLE-2's context-aware dynamic draft tree. Due to higher acceptance rates, tree depth is increased from 6 to 8 while keeping the same number of nodes.

5. Training Details

Optimizer: AdamW, beta_1=0.9, beta_2=0.95
Learning rate: 5e-5
Gradient clipping: 0.5
Draft tree depth: 8 (vs EAGLE-2's 6)

Training data:
- ShareGPT: ~68K entries (1x scale)
- UltraChat-200K: ~464K entries
- Combined ~8x data relative to EAGLE's ShareGPT-only training
- For reasoning model (DeepSeek-R1-Distill-LLaMA 8B): also OpenThoughts-114k-math
- Responses generated by the target model (not fixed dataset)

6. Experimental Results

6.1 Models Tested
- Vicuna 13B
- LLaMA-Instruct 3.1 8B
- LLaMA-Instruct 3.3 70B
- DeepSeek-R1-Distill-LLaMA 8B

6.2 Tasks (5 benchmarks)
- MT-bench (multi-turn conversation)
- HumanEval (code generation)
- GSM8K (math reasoning)
- Alpaca (instruction following)
- CNN/Daily Mail (summarization)

6.3 Main Results — Temperature=0 (Table 1)

Vicuna 13B:
  SpS:        1.92x speedup, tau=2.24
  PLD:        1.74x, tau=1.80
  Medusa:     2.12x, tau=2.51
  Lookahead:  1.62x, tau=1.67
  Hydra:      2.80x, tau=3.50
  EAGLE:      3.05x, tau=3.96
  EAGLE-2:    4.22x, tau=4.83
  EAGLE-3:    5.51x, tau=6.62 (best: HumanEval 6.47x/7.54)

LLaMA-Instruct 3.1 8B:
  EAGLE-2:    3.23x, tau=4.11
  EAGLE-3:    4.44x, tau=6.23 (best: HumanEval 4.85x/6.74)

LLaMA-Instruct 3.3 70B:
  EAGLE-2:    2.85x, tau=3.78
  EAGLE-3:    4.12x, tau=5.88 (best: HumanEval 4.79x/6.52)

DeepSeek-R1-Distill-LLaMA 8B:
  EAGLE-2:    3.26x, tau=3.92
  EAGLE-3:    4.16x, tau=5.84 (best: GSM8K 5.01x/6.93)

Per-task breakdown for Vicuna 13B EAGLE-3 (speedup/tau):
  MT-bench:   5.58x / 6.65
  HumanEval:  6.47x / 7.54
  GSM8K:      5.32x / 6.29
  Alpaca:     5.16x / 6.17
  CNN/DM:     5.01x / 6.47

6.4 Main Results — Temperature=1

Vicuna 13B EAGLE-3:    4.65x, tau=5.67
L31 8B EAGLE-3:        3.45x, tau=4.92
L33 70B EAGLE-3:       3.95x, tau=5.66
DSL 8B EAGLE-3:        3.52x, tau=4.89

Note: methods like Medusa relax acceptance conditions under non-greedy settings (not lossless), so are not compared at temperature=1.

6.5 Scaling Law Discovery (Figure 1/x1/x2)

Data scale (relative to ShareGPT) on MT-bench with LLaMA-Instruct 3.1 8B:
  1x data: EAGLE-2 ~3.2x speedup, EAGLE-3 ~3.7x speedup
  2x data: EAGLE-2 ~3.3x, EAGLE-3 ~4.0x
  4x data: EAGLE-2 ~3.3x, EAGLE-3 ~4.25x
  8x data: EAGLE-2 ~3.3x, EAGLE-3 ~4.4x

EAGLE-2 flatlines; EAGLE-3 shows a clear, monotonically increasing scaling curve. This scaling law for inference acceleration was never observed in prior EAGLE versions.

Accept length scales similarly:
  1x: EAGLE-2 ~4.1, EAGLE-3 ~5.2
  8x: EAGLE-2 ~4.2, EAGLE-3 ~6.1

6.6 Acceptance Rate Analysis (Figure 7/x9)

n-alpha acceptance rates on MT-bench, LLaMA-Instruct 3.1 8B:
  EAGLE:   0-alpha ~0.72, drops progressively to 7-alpha ~0.52
  EAGLE-3: 0-alpha ~0.80, stays ~0.78-0.80 through 7-alpha

EAGLE's acceptance degrades dramatically as the number of self-predicted inputs grows (error accumulation). EAGLE-3's acceptance rate remains essentially flat — the training-time test eliminates distribution shift.

6.7 Ablation Study (Table 2)

LLaMA-Instruct 3.1 8B, MT-bench / GSM8K:
  EAGLE-2 baseline:              3.16x/4.05  |  3.39x/4.24
  + remove feature constraint:   3.82x/5.37  |  3.77x/5.22
  + fused features (full E3):    4.40x/6.13  |  4.48x/6.23

Each innovation contributes meaningfully. Removing the feature constraint provides the larger single improvement (~20% speedup); adding multi-layer fusion adds another ~15%.

7. Production Framework Integration

7.1 SGLang (H100, LLaMA-Instruct 3.1 8B, MT-bench, chain length=3, no tree structure)

Throughput improvement over SGLang without speculation (Table 3):
  BS=2:   EAGLE 1.40x, EAGLE-3 1.81x
  BS=4:   EAGLE 1.38x, EAGLE-3 1.82x
  BS=8:   EAGLE 1.23x, EAGLE-3 1.62x
  BS=16:  EAGLE 1.02x, EAGLE-3 1.48x
  BS=24:  EAGLE 0.93x, EAGLE-3 1.39x
  BS=32:  EAGLE 0.94x, EAGLE-3 1.32x
  BS=48:  EAGLE 0.88x, EAGLE-3 1.38x
  BS=56:  EAGLE 0.99x, EAGLE-3 1.34x
  BS=64:  EAGLE 0.99x, EAGLE-3 1.38x

Critical finding: EAGLE throughput drops below 1.0x at BS>=24 (speculation hurts). EAGLE-3 maintains >1.3x improvement even at BS=64. This overturns the conventional wisdom that speculative sampling reduces throughput at large batch sizes.

Batch size 1 throughput (Table 4, H100, LLaMA-Instruct 3.1 8B, MT-bench):
  SGLang baseline:       158.34 tokens/s
  SGLang + EAGLE-2:      244.10 tokens/s
  SGLang + EAGLE-3:      373.25 tokens/s (2.36x over baseline, 1.53x over EAGLE-2)

7.2 vLLM (RTX 3090, LLaMA-Instruct 3.1 8B, MT-bench, chain length=2, no tree structure)

Throughput improvement (Table 5):
  BS=2:   EAGLE 1.30x, EAGLE-3 1.75x
  BS=4:   EAGLE 1.25x, EAGLE-3 1.68x
  BS=8:   EAGLE 1.21x, EAGLE-3 1.58x
  BS=16:  EAGLE 1.10x, EAGLE-3 1.49x
  BS=24:  EAGLE 1.03x, EAGLE-3 1.42x
  BS=32:  EAGLE 0.93x, EAGLE-3 1.36x
  BS=48:  EAGLE 0.82x, EAGLE-3 1.21x
  BS=56:  EAGLE 0.71x, EAGLE-3 1.01x

EAGLE-3 maintains positive throughput improvement up to BS=56 in vLLM, while EAGLE falls below 1.0x at BS=32.

8. Figures Index

Figure 1 (x1.png, x2.png): Scaling law — speedup and accept length vs data scale (1x-8x). EAGLE-3 increases monotonically; EAGLE-2 flatlines.

Figure 2 (x3.png): Bar chart comparing speedup ratios across all methods (Vanilla, SpS, Medusa, HASS, EAGLE, EAGLE-2, EAGLE-3) on four models. EAGLE-3 dominates everywhere.

Figure 3 (x4.png): Three-panel architectural comparison:
  - Top: EAGLE training/test with feature prediction (l_fea + l_token). Step 1 trains, Step 2 tests with predicted features.
  - Middle: EAGLE + l_fea removal. Step 1 trains, but Step 2 suffers distribution shift (predicted a != ground-truth f, marked with red ≠).
  - Bottom: EAGLE-3 with training-time test. Both steps are unified as Training/Test — model trains on its own outputs, eliminating the distribution mismatch. Red dashed arrow shows the training-time test feedback loop.

Figure 4 (x5.png): 0-alpha acceptance rate vs data scale:
  - EAGLE (red): ~0.755 at 1x, climbs slightly to ~0.783 at 8x.
  - EAGLE without fea pred (yellow): starts lower ~0.745 at 1x, climbs fastest to ~0.81 at 8x.
  - EAGLE-3 (blue): starts lowest ~0.722 at 1x, climbs steeply to ~0.80 at 8x.

Figure 4 right panel (x6.png): 1-alpha acceptance rate vs data scale:
  - EAGLE (red): stable ~0.67-0.69.
  - EAGLE without fea pred (yellow): catastrophically low ~0.22-0.31 (distribution shift).
  - EAGLE-3 (blue): high ~0.70-0.78, surpassing even EAGLE at 4x+ data. Training-time test fully solves the distribution shift.

Figure 5 (x7.png): EAGLE-3 inference pipeline diagram showing:
  - Left: target model extracting l (low), m (mid), h (high) features through decoder layers, concatenated and fused via FC layer into g.
  - Right: three drafting steps (circled 1, 2, 3). Step 1: g_how, g_can + e_I -> FC -> decoder -> a, then LM head -> draft token. Step 2: replaces g_I with a_I. Step 3: continues chaining.

Figure 6 (x8.png): Attention mask diagrams during training-time test:
  - Left: standard lower-triangular mask for "How can I" (ground-truth tokens).
  - Top-right: modified mask for Step 2 with predicted tokens "are", "we", "do". Diagonal attention for predicted tokens (each attends only to its own chain), full causal attention to ground-truth keys "How", "can", "I".
  - Bottom: combined 3-step mask showing how ground-truth (gray), first-round predictions (blue), and second-round predictions (yellow) compose the full attention pattern.

Figure 7 (x9.png): n-alpha acceptance rate (n=0 to 7) on MT-bench:
  - EAGLE (red): degrades from ~0.72 at 0-alpha to ~0.52 at 7-alpha (error accumulation).
  - EAGLE-3 (blue): flat at ~0.78-0.81 across all n values (no error accumulation).

9. Implementation Details (Appendix A)

- Vanilla: HuggingFace transformers, PyTorch backend, pre-allocated KV cache.
- Standard Speculative Sampling: HuggingFace assisted generation. Draft model for Vicuna 13B: Vicuna-68M.
- PLD, Lookahead, Medusa, Hydra: default settings, officially released weights.
- EAGLE: officially released weights for Vicuna/LLaMA2-Chat; ShareGPT-trained for LLaMA3-Instruct.
- EAGLE-2: total draft tokens 60/50/48 for 7B(8B)/13B/70B; draft tree depth=6; 10 nodes during expansion.
- EAGLE-3: same node count as EAGLE-2 but tree depth increased from 6 to 8 (higher acceptance rate allows deeper trees).

10. Key Takeaways

1. Feature prediction is a harmful constraint that limits draft model expressiveness and prevents data scaling.
2. Training-time test (simulating inference during training) eliminates train/test distribution mismatch, enabling the draft model to handle its own noisy predictions gracefully.
3. Multi-layer feature fusion captures richer semantic information than top-layer-only features.
4. EAGLE-3 discovers the first scaling law for inference acceleration: more data = better speedup, monotonically.
5. Production-viable at large batch sizes: 38-40% throughput improvement at BS=64 in SGLang, overturning the conventional wisdom that speculation hurts throughput at scale.
6. Maximum speedup: 6.5x (HumanEval, Vicuna 13B, temperature=0).
7. Lossless: strict speculative sampling acceptance conditions, no degradation of output quality.
