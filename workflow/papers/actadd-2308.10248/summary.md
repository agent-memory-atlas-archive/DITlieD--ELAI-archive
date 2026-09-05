Steering Language Models With Activation Engineering
arXiv: 2308.10248v5 (2024-10-10)
Authors: Alexander Matt Turner, Lisa Thiergart, Gavin Leech, David Udell, Juan J. Vazquez, Ulisse Mini, Monte MacDiarmid
License: CC BY 4.0
Categories: cs.CL, cs.LG
DOI: 10.48550/arXiv.2308.10248

================================================================================
1. ABSTRACT
================================================================================

Introduces activation engineering — modifying intermediate activations during inference to control LM outputs. The core method, Activation Addition (ActAdd), computes steering vectors from contrasting prompt pairs (e.g., "Love" minus "Hate") and adds them to the residual stream during forward passes. Achieves SOTA on sentiment shift and detoxification (LLaMA-3, OPT). Enables inference-time control over high-level output properties while preserving off-target task performance. No optimization or large datasets required.

================================================================================
2. CORE METHOD: ACTIVATION ADDITION (ActAdd)
================================================================================

Algorithm 1 — ActAdd (Optimization-Free):

Input:
  (p+, p-) = steering prompt pair (tokenized)
  p*       = user prompt
  l        = target layer
  c        = injection coefficient (scalar)
  a        = sequence position alignment
  M        = pretrained decoder-only transformer

Process:
  1. Pad p+ and p- to same token length
  2. Forward M on p+, record activations at layer l -> h+^l
  3. Forward M on p-, record activations at layer l -> h-^l
  4. Compute steering vector: h_A^l = h+^l - h-^l
  5. Forward M on p*, at layer l add c * h_A^l to residual stream at position a
  6. Continue forward pass with modified residual
  7. Return steered output S

Core equation:
  h_A^l = h+^l - h-^l
  residual_modified = h^l + c * h_A^l

Key properties:
  - Requires only 2 data points (one contrast pair)
  - No gradient computation, no backpropagation, no fine-tuning
  - Single forward pass overhead for vector extraction (amortized over reuse)
  - Works on any decoder-only transformer with residual stream access

================================================================================
3. THEORETICAL BASIS: LINEAR REPRESENTATION HYPOTHESIS
================================================================================

The method rests on the hypothesis that high-level features are encoded as directions in activation space (linear representations). Evidence from prior work:
- Word2vec analogies (Mikolov et al. 2013): king - man + woman = queen
- Linear probes recovering features from intermediate layers (Alain & Bengio 2018)
- Emergent world representations in sequence models (Li et al. 2023a)
- Othello-GPT linear world model (Nanda 2023)
- Toy models of superposition (Elhage et al. 2022)

ActAdd exploits this: if "wedding-ness" is a direction, adding that direction vector steers output toward wedding-related content. The success of simple vector arithmetic on activations provides further evidence for the linear representation hypothesis.

================================================================================
4. POSITIONING IN STEERING LITERATURE (Table 2)
================================================================================

Intervention vectors categorized along two axes:
  (A) What is modified: weights vs. activations
  (B) How vector is extracted: gradient-based search vs. prompt pair differences

ActAdd = activations + prompt pair differences (no optimization).

Comparison:
  - ROME/MEMIT (Meng et al. 2023): weights, gradient-based
  - Soft prompting / Prefix-tuning (Lester 2021, Li & Liang 2021): activations, gradient-based
  - AutoPrompt (Shin et al. 2020): activations, gradient-based discrete search
  - ITI (Li et al. 2023b): activations, learned from labeled data
  - ActAdd: activations, prompt pair differences, zero optimization

================================================================================
5. EXPERIMENTS AND RESULTS
================================================================================

5.1 TOPIC STEERING (WEDDING VECTOR)
-------------------------------------
Model: GPT-2-XL (1.5B parameters)
Contrast pair: ("I talk about weddings constantly", "I do not talk about weddings constantly")
Injection layer: l=16, coefficient: c varies

Results:
- Perplexity ratio on wedding-related OpenWebText documents drops significantly (model becomes better at predicting wedding text)
- Perplexity on non-wedding documents stays near 1.0 (no degradation)
- At l=6, ~90% of steered completions contain wedding-related words vs. ~2% baseline

[Figure x2]: Perplexity ratio vs. wedding word frequency in documents.
  X-axis: wedding word frequency in document. Y-axis: perplexity ratio (ActAdd / baseline).
  Shows steep decline — documents with more wedding words see greatest perplexity improvement.
  Non-wedding documents cluster near ratio=1.0.

[Figure x3]: Non-zero wedding word count fraction vs. injection layer.
  X-axis: layer (0-48). Y-axis: fraction of completions with wedding words.
  Peak at layers 5-15 (~0.9), declining through layers 20-40, baseline at ~0.05.
  Early-to-mid layers most effective for topic steering.

[Figure x7]: Mean wedding word count vs. injection layer.
  X-axis: layer (0-48). Y-axis: mean count of wedding words per completion.
  Peak ~1.7 words around layer 15-20, declining to baseline after layer 35.

5.2 MULTI-TOPIC STEERING
--------------------------
Model: GPT-2-XL
Topics tested: art, finance, music, politics, science, weddings
Evaluation: GPT-3.5 scoring of topic relevance (0-100%)
Coefficient sweep: c in {0.0, 0.5, 1.0, 2.0}

[Figure x4]: Bar chart of % evaluated as relevant per topic at different c values.
  - Weddings: ~20% at c=2.0 (highest)
  - Science: ~20% at c=2.0
  - Politics: ~10% at c=2.0
  - Music: ~8% at c=2.0
  - Finance: ~5-7% at c=2.0
  - Art: ~2% at c=2.0 (least effective)
  - Baseline (c=0.0): ~0-2% across all topics
  Higher c consistently increases relevance. Art is the hardest topic to steer toward.

5.3 TOXICITY REDUCTION (RealToxicityPrompts)
----------------------------------------------
Dataset: RealToxicityPrompts, n=1000 prompts, 5 iterations for statistical testing
Scoring: Perspective API toxicity scores
Models: OPT-6.7B, LLaMA-3-8B

Results (Table 3):
  Method          | Model    | Toxicity | Disfluency | Relevance
  ActAdd          | OPT      | 0.112    | 13.8       | 0.329
  ActAdd          | LLaMA-3  | 0.108    | 6.7        | 0.365
  PREADD-D        | OPT      | 0.122    | 56.6       | 0.326
  Baseline        | OPT      | 0.134    | 8.9        | 0.369
  Baseline        | LLaMA-3  | 0.114    | 6.3        | 0.391

Key finding: ActAdd-OPT achieves 16.4% toxicity reduction over OPT baseline (0.112 vs 0.134) with moderate disfluency increase. ActAdd-LLaMA-3 achieves 5.3% reduction. Both outperform PREADD-D which has 6x worse disfluency (56.6 vs 8.9).

5.4 SENTIMENT CONTROL (Stanford IMDb)
---------------------------------------
Dataset: Stanford IMDb, n=1000, repeated for p-values
Scoring: SiEBERT sentiment classifier
Fluency: conditional perplexity via GPT-3 davinci-002
Relevance: cosine similarity via all-MiniLM-L6-v2

Results (Table 4):
  Method         | Pos->Neg Steer/Disfl/Rel | Neg->Pos Steer/Disfl/Rel
  ActAdd-OPT     | 0.432 / 24.2 / 0.387    | 0.564 / 20.95 / 0.363
  ActAdd-LLaMA3  | 0.268 / 8.6 / 0.354     | 0.669 / 15.2 / 0.275
  PREADD-S-OPT   | 0.631 / 68.4 / 0.253    | 0.624 / 67.1 / 0.258
  FUDGE          | 0.532 / 25.1 / 0.311     | 0.551 / 22.7 / 0.320

Key finding: ActAdd-LLaMA3 achieves SOTA neg->pos sentiment shift (0.669) with best fluency (15.2 disfluency). PREADD-S has higher raw steering but catastrophic disfluency (68.4). FUDGE is competitive but ActAdd trades slightly less steering for much better fluency.

5.5 KNOWLEDGE PRESERVATION (ConceptNet LAMA)
----------------------------------------------
Dataset: ConceptNet LAMA, n=29,774 factual fill-in-the-blank sentences
Metric: P@K (probability correct answer is in top-K predictions)

[Figure x5]: P@K curves for baseline vs. ActAdd (wedding vector).
  X-axis: K (1 to 100, log scale). Y-axis: mean P@K.
  Lines nearly perfectly overlap. At K=100, both ~0.48.
  Demonstrates negligible impact on factual knowledge even with strong topic steering.

5.6 QUALITATIVE EXAMPLES (Table 6)
-------------------------------------
GPT-2-XL steering examples with parameters:

1. "Intent to praise" - "Intent to hurt" (l=6, c=+15):
   Prompt: "I hate you because" -> steered to positive/loving completion

2. Conspiracy theory steering (l=23, c=+1):
   Prompt: "Bush did 9/11 because" -> steered toward conspiracy content

3. "Anger" - "Calm" (l=20, c=+10):
   Produces emotionally agitated completions

4. "Eiffel Tower" location steering (l=24, c=+10):
   Steers geographic/location references toward Paris/France

5. Anti-violence steering (l=15, c=+4):
   "I NEVER talk about people getting hurt" reduces violent content

5.7 FAILURE CASES (Table 7)
------------------------------
4 documented failures:
- Love vector alone (without subtraction) — inconsistent steering
- "Goose/AAAAHHHH" phrase combination — no meaningful steering
- Demonstrates that not all contrast pairs produce useful vectors
- Quality of steering depends on the semantic meaningfulness of the contrast

================================================================================
6. ABLATION STUDIES AND ANALYSIS
================================================================================

6.1 TOKEN INJECTION HYPOTHESIS (Table 8, Appendix B)
------------------------------------------------------
Tests whether ActAdd is equivalent to prompt engineering (just injecting token embeddings).
Experiment: compare 20->20 embedding injection vs. 0->20 embedding injection for Anger-Calm vector.
Result: ActAdd is NOT equivalent to token injection. The transformation through layers creates vectors that cannot be replicated by simple embedding addition.

6.2 PROMPTING VS. ACTADD (Table 9)
-------------------------------------
Perplexity comparison on wedding and non-wedding documents:
  Condition               | ActAdd | Prompting
  Wedding-related ratio   | 0.875  | 0.890
  Wedding-unrelated ratio | 0.994  | 1.132

ActAdd achieves comparable wedding steering (0.875 vs 0.890) but crucially preserves non-wedding perplexity (0.994 vs 1.132). Prompting degrades non-target performance by 13.2%; ActAdd by only 0.6%.

6.3 LAYER SWEEP ANALYSIS
--------------------------
[Figure x3] and [Figure x7] show injection effectiveness by layer:
- Early layers (0-5): minimal effect
- Mid layers (6-20): peak effectiveness, with exact peak depending on model/task
- Late layers (25+): declining effectiveness
- Final layers (35+): near baseline

Interpretation: mid-layer residual streams contain the most malleable high-level representations. Early layers encode low-level token features; late layers have committed to output distribution.

6.4 COEFFICIENT SWEEP
-----------------------
[Figure x13]: Perplexity ratio vs. injection coefficient at layer 6 and layer 16.
  Two panels (Layer 6, Layer 16). X-axis: injection coefficient (-1 to 4). Y-axis: perplexity ratio.
  Blue line (weddings): monotonically decreasing — higher c = better wedding prediction.
  Red line (not-weddings): slowly increasing — higher c = slightly worse non-wedding prediction.
  Layer 16 shows stronger separation between target/non-target curves.
  Optimal c balances steering strength vs. off-target degradation.

6.5 RANDOM VECTOR ROBUSTNESS (Appendix F)
-------------------------------------------
[Figure x10]: KL divergence comparison — 1 random vector vs. N random vectors.
  X-axis: KL(normal || 1 random vector). Y-axis: KL(normal || N random vectors).
  Points above diagonal = multiple random vectors cause more disruption.
  Shows ActAdd steering vectors are meaningfully different from random noise.

6.6 TOKEN PROBABILITY ANALYSIS
---------------------------------
[Figure x8]: Q-Q plot of ordered mean log-probability differences.
  X-axis: theoretical quantiles. Y-axis: ordered mean log-prob differences.
  Approximately normal with heavy positive tail.
  The positive tail = wedding-related tokens receive reliably increased probability.
  Non-wedding tokens follow normal distribution (random noise-level changes).

[Figure x11]: Token contribution scatter (partial view).
  X-axis: normal model token probability. Y-axis: contribution score.
  Sparse scatter of points showing which tokens are most affected.

[Figure x12]: Modified/normal token probability ratio vs. normal probability.
  X-axis: normal model token probability (0.01 to 0.05). Y-axis: probability ratio.
  "Wedding" token has ratio ~10x (massive boost).
  "Friend" has ratio ~1.8x.
  "Game", "movie" suppressed below 1.0.
  Red dashed line at ratio=1.0 (no change).
  Demonstrates selective token-level control: wedding-related tokens boosted, unrelated tokens suppressed or unchanged.

6.7 PARTIAL ACTADD (Appendix H)
----------------------------------
[Figure x14]: Average wedding word count vs. fraction of steering vector dimensions used.
  X-axis: fraction of dimensions affected (0 to 1). Y-axis: avg # of wedding words.
  Multiple prompt lines shown (6 different user prompts).
  Most prompts: effect emerges around 0.5-0.6 fraction, peaks at 0.7-0.9.
  "I went up to my friend and said": peaks at ~1.3 words at fraction 0.7.
  Shows that steering requires a substantial fraction of dimensions — not a few isolated neurons.
  Evidence for distributed (not localized) representation of concepts.

================================================================================
7. CROSS-MODEL REPLICABILITY (Appendix E)
================================================================================

Tested on 5 model architectures:
  - GPT-2-XL (1.5B)
  - GPT-J-6B (6B)
  - OPT-6.7B (6.7B)
  - Llama-1-13B (13B)
  - LLaMA-3-8B (8B)

ActAdd works across all tested models. Optimal layer and coefficient vary by model but the method generalizes. Larger models (LLaMA-3) show better fluency preservation.

================================================================================
8. EXPERIMENTAL INFRASTRUCTURE
================================================================================

Hardware:
  GPU: NVIDIA RTX A5000 (24GB)
  CPU: Intel Core i9-10900X @ 3.70GHz
  RAM: 32GB system

Software:
  OS: Ubuntu 22.04.1 LTS
  PyTorch: 1.13.1
  TransformerLens: 1.4.0
  NumPy: 1.24.3
  Pandas: 2.0.1

Sampling hyperparameters (precommitted):
  Temperature: 1.0 (validated with 0.6)
  Frequency penalty: 1.0
  Top-p: 0.3 (validated with 0.5)
  K completions: 3
  Seed: 0 (validated on seeds 1, 2)

ActAdd hyperparameter search space:
  Injection coefficient c: [3, 20] typical, magnitude < 15
  Layer l: [6, 24]
  Sequence alignment a: 1 (fixed)
  Method: grid search

================================================================================
9. BROADER IMPACTS AND LIMITATIONS
================================================================================

Limitations:
- Requires access to intermediate activations (incompatible with API-only models)
- Layer l and coefficient c require per-task tuning (grid search)
- Not all contrast pairs produce meaningful steering (quality depends on semantic content)
- Potential for misuse to amplify harmful outputs (toxicity, bias)

Advantages over alternatives:
- No training data beyond 2 prompts
- No gradient computation
- Composable (multiple steering vectors can be added)
- Reversible (remove vector = remove steering)
- Preserves off-target capabilities (unlike fine-tuning)
- Inference-time only (no model modification)

Broader impacts:
- Alignment: could enable safer model deployment via behavioral steering
- Interpretability: provides evidence for linear representation hypothesis
- Misuse: same vectors that reduce toxicity can amplify it (dual-use concern)

================================================================================
10. FIGURE INDEX
================================================================================

All figures saved in: workflow/papers/actadd-2308.10248/figures/

figure_x1.png  — ActAdd pipeline schematic: contrast prompts -> embed -> forward -> record activations -> diff -> inject into user prompt forward pass -> steered output
figure_x2.png  — Perplexity ratio vs. wedding word frequency (steep decline for wedding docs, flat for non-wedding)
figure_x3.png  — Wedding word fraction vs. injection layer (peak at layers 5-15, ~90%)
figure_x4.png  — Multi-topic relevance bar chart at c={0,0.5,1.0,2.0} (weddings/science best, art worst)
figure_x5.png  — ConceptNet P@K curves (baseline and ActAdd nearly identical — knowledge preserved)
figure_x6.png  — Pedagogical 1D residual stream example: "I love dogs" + wedding vector arithmetic across layers, showing how layer-6 injection modifies unembedding to produce "dress" instead of original token
figure_x7.png  — Mean wedding word count vs. layer (peak ~1.7 at layer 15-20)
figure_x8.png  — Q-Q plot of log-probability differences (normal + heavy positive tail for wedding tokens)
figure_x10.png — KL divergence: 1 random vector vs. N random vectors (above diagonal = additive disruption)
figure_x11.png — Token contribution vs. normal model probability (sparse scatter)
figure_x12.png — Token probability ratio plot: "wedding" boosted ~10x, "friend" ~1.8x, "game"/"movie" suppressed
figure_x13.png — Perplexity ratio vs. injection coefficient at layers 6 and 16 (monotonic target improvement, slight off-target degradation)
figure_x14.png — Partial ActAdd: wedding words vs. fraction of dimensions (effect emerges at 0.5-0.6, evidence for distributed representations)

================================================================================
11. COMPLETE REFERENCE LIST (67 entries)
================================================================================

[1] Alain & Bengio 2018. Understanding intermediate layers using linear classifier probes.
[2] Bloom & Nanda 2022. TransformerLens: A library for mechanistic interpretability.
[3] Brown et al. 2023. Robustness of edited neural networks.
[4] Brown et al. 2020. Language models are few-shot learners (GPT-3).
[5] Burns et al. 2022. Discovering latent knowledge in language models without supervision.
[6] Dathathri et al. 2020. Plug and play language models: controlled text generation.
[7] Elhage et al. 2021. A mathematical framework for transformer circuits.
[8] Elhage et al. 2022. Toy models of superposition.
[9] Gehman et al. 2020. RealToxicityPrompts dataset.
[10] Gnanadesikan & Wilk 1968. Probability plotting methods for analysis.
[11] Grover et al. 2019. Bias correction of learned generative models.
[12] Gu et al. 2017. Trainable greedy decoding for neural machine translation.
[13] Gu et al. 2022. Controllable text generation via probability density estimation.
[14] Hartmann et al. 2023. SiEBERT: Sentiment analysis accuracy and application.
[15] Hase et al. 2023. Does localization inform editing? Causality-based localization.
[16] Hernandez et al. 2023. Inspecting and editing knowledge representations in LLMs.
[17] Ilharco et al. 2023. Editing models with task arithmetic.
[18] Jin et al. 2022. Deep learning for text style transfer survey.
[19] Khashabi et al. 2022. Prompt waywardness: discretized interpretation of continuous prompts.
[20] Korinek 2023. Language models and cognitive automation for economic research.
[21] Larsen et al. 2016. Autoencoding beyond pixels using learned similarity metric.
[22] Lester et al. 2021. The power of scale for parameter-efficient prompt tuning.
[23] Li et al. 2018. Delete, retrieve, generate: sentiment and style transfer.
[24] Li et al. 2023a. Emergent world representations: sequence model on synthetic task.
[25] Li et al. 2023b. Inference-time intervention: eliciting truthful answers (ITI).
[26] Li & Liang 2021. Prefix-Tuning: optimizing continuous prompts for generation.
[27] Liu et al. 2023. In-context Vectors: latent space steering for in-context learning.
[28] Lyu et al. 2024. Keeping LLMs aligned after fine-tuning: prompt templates role.
[29] Maas et al. 2011. Stanford IMDb dataset for sentiment analysis.
[30] Meng et al. 2023. Locating and editing factual associations in GPT (ROME).
[31] Meta 2024. Meta Llama 3 model.
[32] Michel et al. 2019. Are sixteen heads really better than one? Attention head ablation.
[33] Mikolov et al. 2013a. Distributed representations: compositionality of word phrases.
[34] Mikolov et al. 2013b. Linguistic regularities in continuous space (word2vec analogy).
[35] Mini et al. 2023. Understanding and controlling a maze-solving policy network.
[36] Moschella et al. 2023. Relative representations enable zero-shot latent space communication.
[37] Nanda 2023. Othello-GPT has a linear emergent world representation.
[38] Olah 2023. Distributed representations: composition & superposition.
[39] Olsson et al. 2022. In-context learning and induction heads.
[40] Park et al. 2023. The linear representation hypothesis and geometry of LLMs.
[41] Pei et al. 2023. PREADD: prefix-adaptive decoding for controlled text generation.
[42] Peterson et al. 2018. OpenWebText corpus.
[43] Petroni et al. 2019. Language models as knowledge bases? (LAMA benchmark).
[44] Qi et al. 2023. Fine-tuning aligned LMs compromises safety.
[45] Radford et al. 2019. GPT-2: unsupervised multitask learners.
[46] Ranzato et al. 2016. Sequence level training with recurrent neural networks.
[47] Reimers & Gurevych 2019. Sentence-BERT: sentence embeddings (all-MiniLM-L6-v2).
[48] Shin et al. 2020. AutoPrompt: eliciting knowledge with automatically generated prompts.
[49] Sloman 2002. The irrelevance of Turing machines to artificial intelligence.
[50] Strunk 2013. NLTK Punkt tokenizer documentation.
[51] Subramani et al. 2022. Extracting latent steering vectors from pretrained LMs.
[52] Touvron et al. 2023. LLaMA: open and efficient foundation language models.
[53] Vaswani et al. 2017. Attention is all you need (Transformer architecture).
[54] Wang & Komatsuzaki 2021. GPT-J-6B model.
[55] Wang et al. 2024. Prompt engineering in consistency and reliability with evidence-based guidelines.
[56] Wei et al. 2022. Chain-of-thought prompting elicits reasoning in LLMs.
[57] White 2016. Sampling generative networks.
[58] Wu et al. 2023. Eva-KELLM: benchmark for evaluating knowledge editing.
[59] Yang & Klein 2021. FUDGE: controlled text generation with future discriminators.
[60] Ye & Durrett 2022. The unreliability of explanations in few-shot prompting.
[61] Zhang et al. 2022a. Survey of controllable text generation using Transformer-based PLMs.
[62] Zhang et al. 2024. Comprehensive study of knowledge editing for LLMs.
[63] Zhang et al. 2022b. OPT: Open pre-trained transformer language models.
[64] Zhong et al. 2023. Air-Decoding: attribute distribution reconstruction for decoding-time control.
[65] Zhou et al. 2022. Steering LLMs using APE (automated prompt engineering).
[66] Ziegler et al. 2019. Fine-tuning language models from human preferences (RLHF).
[67] Zou et al. 2023. Representation engineering: top-down approach to AI transparency.
