# Self-evolving prompts for ELAI

## TL;DR
- ELAI's 330-token meta-instructions are exactly the substrate that wins with reflective optimizers; GEPA (ICLR 2026 Oral, Agrawal et al. 2025) beats MIPROv2 by >10pp and GRPO by up to 20pp using 35x fewer rollouts (arxiv:2507.19457). This is the technique to adopt first.
- Wrap `PromptEngineer` agent + `ReasoningTemplate` archive (D-1, D-plan) as a GEPA-compatible optimizer surface. Treat role-prompts + few-shot blocks as the "textual parameters"; use the existing PPM/separable-verifier as the scalar metric and emit short text feedback per BabyStep result. GEPA is published as a standalone `gepa` PyPI package independent of DSPy, callable from any harness.
- Hard ground-truth: small LLMs as *optimizers* fail (Zhang et al. ACL Findings 2024, arxiv:2405.10276) - LLaMA-2-7B/13B/70B and Mistral-7B perform marginally or worse than CoT baselines under OPRO. So optimization driver must be cloud Sonnet/GPT-class while *target model* is local Qwen2.5-Coder-7B. Cross-model transfer is documented to work (PromptBridge arxiv:2512.01420, modular prompt optimization arxiv:2601.04055).
- Risk: prompt optimizers overfit small training sets harder than NN weights. DSPy itself documents an inverted split (20 train / 80 val) for this reason. Add a hold-out test set per BabyStep template and a cross-domain regression canary (you already have D-1 canary infra - extend it).
- Do NOT port DSPy to Rust yourself. `dspy-rs` and `DSRs` exist but are toys; run optimization offline in Python, ship the optimized template (blake3-hashed) into the Rust runtime archive via the existing D-1 lifecycle. Optimization is rare/batch, inference is hot - keep them separate.

## Key papers / repos

DSPy family
- arxiv:2310.03714 | DSPy: Compiling Declarative LM Calls into Self-Improving Pipelines (Khattab et al.) | foundational; programs as modules, compiler synthesises prompts; T5-770M and Llama2-13b-chat competitive with hand-written GPT-3.5 chains.
- https://dspy.ai/api/optimizers/MIPROv2/ | MIPROv2 docs | joint instruction + few-shot via Bayesian Optimization over minibatches; default optimizer until 2025.
- arxiv:2507.19457 | GEPA: Reflective Prompt Evolution Can Outperform RL (Agrawal et al., ICLR 2026 Oral) | beats MIPROv2 +10pp avg, +12pp on AIME-2025, +14pp Qwen3-8B (38->52%); 35x fewer rollouts than GRPO; MAP-Elites Pareto frontier mechanism.
- https://dspy.ai/api/optimizers/SIMBA/ | SIMBA (Stochastic Introspective Mini-Batch Ascent) | predecessor to GEPA, picks high-variance examples and either appends demos or generates self-reflective rules; documented as more stable than MIPROv2 on capable LMs.
- https://dspy.ai/api/optimizers/BootstrapFewShot/ | BootstrapFewShot | teacher program generates traces, metric filters; useful at ~10 examples; baseline before instruction-optimizers.
- arxiv:2507.03620 | "Is It Time To Treat Prompts As Code?" Multi-case DSPy study | 5 tasks: prompt-eval 46.2%->64.0%, router 85%->90%, hallucination detection modest; critical negative: "using the optimized prompt with a cheaper model did not improve performance".

TextGrad family
- arxiv:2406.07496 | TextGrad: Automatic Differentiation via Text (Yuksekgonul et al.) | textual gradients backprop'd through LLM calls; GPT-4o GPQA 51%->55%, LeetCode-Hard +20% relative, also molecule/radiation-oncology applications; PyTorch-style API.
- arxiv:2511.12063 | TextBO: Bayesian Optimization in Language Space (Kang & Yoganarasimhan, Nov 2025) | textual gradients + Best-of-N approximates UCB-BO; beats both Best-of-N and GEPA on eval-efficiency for ad-alignment; eval-efficiency is the constraint, not generation cost.
- arxiv:2601.04055 | Modular Prompt Optimization with Section-Local Textual Gradients | structured-prompt analogue of TextGrad; beats untuned structured prompts on LLaMA-3-8B-Instruct + Mistral-7B reasoning benchmarks.

Evolutionary / generic LLM-as-optimizer
- arxiv:2309.16797 | PromptBreeder (Fernando et al., DeepMind) | self-referential: mutation-prompts evolve alongside task-prompts; beats CoT and Plan-and-Solve on arithmetic/commonsense; OpenReview HKkiX32Zw1.
- arxiv:2309.03409 | OPRO: LLMs as Optimizers (Yang et al., DeepMind) | up to +8% GSM8K, up to +50% on some BBH tasks vs human prompts; works on PaLM-2/GPT-4 scale.
- arxiv:2405.10276 | Revisiting OPRO: Limitations of Small-Scale LLMs as Optimizers (Zhang et al., ACL Findings 2024) | LLaMA-2-7B/13B/70B + Mistral-7B largely ineffective as the optimizer LM under OPRO; marginal or detrimental vs Zero-shot/Few-shot CoT on GSM8K.
- arxiv:2309.08532 | EvoPrompt (Guo et al., ICLR 2024) | evolutionary operators (crossover/mutation) on prompts; up to +25% on BBH vs human prompts.
- arxiv:2211.01910 | APE: LLMs Are Human-Level Prompt Engineers (Zhou et al., ICLR 2023) | instruction-as-program; IQM 0.810 vs human 0.749 across 24 tasks; founding work.
- AlphaEvolve (DeepMind, June 2025) | Gemini-powered evolutionary coding agent; stochastic prompt formatting + meta-prompt evolution; ensemble of Flash (breadth) + Pro (depth); discovered new 4x4 matrix mult algorithm; OpenEvolve OSS reimpl exists.

Generalization / overfitting / small-model transfer
- arxiv:2510.08413 | Prompts Generalize with Low Data: Non-vacuous Generalization Bounds | first PAC-style bounds for prompt optimization; informative priors tighten bounds.
- arxiv:2603.21520 | MemAPO: Generalizable Self-Evolving Memory for Auto Prompt Opt | dual-memory (strategy templates from successes, structured error patterns from failures).
- arxiv:2602.00997 | ETGPO: Error Taxonomy-Guided Prompt Optimization | ~1/3 the optimization-phase tokens of SOTA at comparable accuracy.
- arxiv:2512.01420 | PromptBridge: Cross-Model Prompt Transfer | GPT-4o-optimized prompts transfer well to Qwen3-32B, Llama3.1-8B-Instruct, Gemma3-27B-it on coding benchmarks.
- arxiv:2504.20355 | Local Prompt Optimization (LPO) | <edit>...</edit> tags restrict optimizer to specific token spans; faster convergence + better GSM8K/MultiArith/BBH.
- https://dspy.ai/learn/optimization/overview/ | DSPy split guidance: 20% train / 80% val (inverted vs NN) precisely because prompt optimizers overfit small train sets.
- https://github.com/krypticmouse/DSRs and https://crates.io/crates/dspy-rs | Rust DSPy ports - both early/incomplete as of 2026-05; community projects, not production-grade.

## Optimizer comparison

| optimizer | mechanism | benchmark gain (cite) | works on 7B local? | works with separable verifier? | rust-portable? |
|---|---|---|---|---|---|
| MIPROv2 | Bayesian Opt over (instruction, few-shot) joint space, minibatch eval | ~70% judge task with 110 train; baseline for DSPy 2024+ (dspy.ai) | yes as *target*; not as optimizer | yes - scalar metric | no, run in Python |
| GEPA | Reflective text feedback + MAP-Elites Pareto frontier; LM introspects trajectories | +12pp AIME-2025 vs MIPROv2; +14pp Qwen3-8B 38->52; 35x fewer rollouts than GRPO (arxiv:2507.19457) | yes as target; needs strong LM for reflection | yes + text feedback boosts it | no - run offline |
| SIMBA | Mini-batch ascent, picks high-variance examples, self-reflective rules | "superior sample efficiency, higher performance, greater stability" vs MIPROv2 (DSPy docs) | yes as target | yes | no |
| TextGrad | Textual gradients backprop through LM calls | GPT-4o GPQA +4pp (51->55), LeetCode-Hard +20% rel, BBH/GSM8K wins (arxiv:2406.07496) | optimizer LM should be strong; target can be 7B | yes - loss = scalar from metric | no |
| TextBO | Textual-gradient + Best-of-N approximates UCB-BO | beats GEPA on eval-efficiency for ad-alignment (arxiv:2511.12063) | needs strong proposal LM | yes | no |
| PromptBreeder | Mutation-prompts evolve task-prompts (self-referential) | beats CoT/Plan-and-Solve on arithmetic/commonsense (arxiv:2309.16797) | needs strong LM | yes | no |
| OPRO | LM proposes prompts conditioned on score history | +8% GSM8K, +50% on some BBH (arxiv:2309.03409) | NO as optimizer below ~70B (arxiv:2405.10276); yes as target | yes | no |
| EvoPrompt | Genetic operators (crossover/mutation) on prompt population | +25% BBH vs human (arxiv:2309.08532) | needs strong evolutionary LM | yes | trivial to port - just GA loop |
| APE | LM generates instruction candidates, score-and-pick | IQM 0.810 vs 0.749 human across 24 tasks (arxiv:2211.01910) | needs strong proposer | yes | trivial |
| AlphaEvolve | Evolutionary coding agent, Gemini Flash+Pro ensemble | new 4x4 matmul algorithm, 50+ open math problems improved (DeepMind blog 2025) | needs frontier LMs | yes - automated evaluators central | no |
| LPO | `<edit>` tags restrict optimization to specific token spans | faster convergence, better GSM8K/MultiArith/BBH (arxiv:2504.20355) | wraps another optimizer | yes | wraps existing methods |

## Best techniques to adopt

Ranked.

1. **GEPA as the primary optimizer driver, run on cloud Sonnet 4.6 as reflection LM, target = local Qwen2.5-Coder-7B**
   - Grounding: arxiv:2507.19457 (ICLR 2026 Oral), beats MIPROv2 +10pp avg, beats GRPO with 35x fewer rollouts on six tasks incl. HotpotQA/IFBench/AIME-2025/HOVER/Papillon. Independent `gepa` PyPI package (gepa-ai/gepa).
   - Integration hook: wraps `PromptEngineer` agent. Per BabyStep template (D-plan), feed GEPA the role-prompt + Tier-3 semantic rules + few-shot block as the textual parameters. Metric = existing PPM/separable-verifier scalar on the DETERMINISTIC_CHECK pass rate. Text feedback = the auditor's BLOCK/PASS reasons fed back as natural language. Optimized template gets blake3-hashed and admitted to `ReasoningTemplate` archive via existing BAI Thompson promotion pipeline.
   - Critical: GEPA reflection LM must be cloud-class (Sonnet/GPT-4-class). Target inference stays on local 7B per ELAI's substrate.

2. **Cross-model prompt transfer instead of co-optimization**
   - Grounding: arxiv:2405.10276 (small models fail as OPRO optimizers); arxiv:2512.01420 PromptBridge (GPT-4o prompts transfer well to Qwen3-32B/Llama3.1-8B/Gemma3-27B); promptrefiner pattern (GPT-4 designs system prompt for local 7B).
   - Integration hook: optimization driver is *always* cloud; the deployed artifact targets the local model. The existing `cross-domain regression canary` in D-1 archive lifecycle becomes the transfer-validity check before promotion.

3. **Text feedback enrichment of the auditor signal**
   - Grounding: GEPA paper - users provide text feedback alongside scalar score "provides more visibility into why the system got the score". This is also TextGrad's whole premise (arxiv:2406.07496).
   - Integration hook: the auditor already produces per-rule pass/fail with reasons. Wire the auditor's structured reasons into the optimizer feedback stream verbatim. No new infra - the data already exists.

4. **Local Prompt Optimization (LPO) wrapping**
   - Grounding: arxiv:2504.20355 - tag editable spans with `<edit>...</edit>`; faster convergence and better generalization on math/BBH.
   - Integration hook: BabyStepSpec's 8 blocks (GROUNDING, PRE_GATE, TASK, TDD, DETERMINISTIC_CHECK, ACCEPTANCE, INTEGRATION, EXIT_CRITERIA) are *already* structured sections. Mark only the role-prompt instruction text and few-shot examples as `<edit>`-able; freeze the contract blocks (DETERMINISTIC_CHECK, ACCEPTANCE) so optimization can't drift the test semantics.

5. **Separable verifier (PPM) as fitness function**
   - Grounding: process reward models documented as fitness-signal in 2025 survey (arxiv:2510.08049). DG-PRM uses Pareto over multi-dim criteria - aligns with GEPA's MAP-Elites.
   - Integration hook: the existing PPM sidecar score is exactly the scalar GEPA/TextGrad consume. Multi-dim version: extend MAP-Elites cells along your existing audit dimensions (correctness, determinism, wiring-contract pass).

6. **ETGPO-style error taxonomy injection**
   - Grounding: arxiv:2602.00997 - aggregate errors into taxonomy, add taxonomy guidance to prompt. ~1/3 the optimization tokens at comparable accuracy.
   - Integration hook: your `gotchas.md` and `failure_taxonomy.py` already are an error taxonomy. Inject the top-3 active gotchas as a `Pitfalls:` block into the role-prompt during optimization.

7. **TextBO as evaluation-efficiency upgrade if GEPA proves too costly**
   - Grounding: arxiv:2511.12063 - beats GEPA on per-eval performance when reliable feedback is expensive.
   - Integration hook: same plumbing as GEPA; swap the optimizer call.

Do NOT adopt without reason: PromptBreeder/EvoPrompt/OPRO/APE are superseded for our use case by GEPA/SIMBA on every reported axis. MIPROv2 is fine as a baseline / regression check.

## Risks + overfitting

- **Train-set overfitting is the headline risk.** DSPy itself recommends 20/80 train/val (inverted vs NN training) precisely because prompt optimizers collapse onto seen examples. (https://dspy.ai/learn/optimization/overview/). Mitigation: hold-out test per template, plus the cross-domain canary you already have in D-1.
- **Cheap-model degradation under cross-LM transfer.** arxiv:2507.03620 documents directly: "using the optimized prompt with a cheaper model did not improve performance" in their case studies. Mitigation: PromptBridge-style validation - optimize WITH cloud, deploy TO local, but always re-evaluate on the target model before admitting to archive.
- **Reflection-LM dependence.** GEPA's gains are bounded by reflection-LM strength. Qwen3-8B *as reflection LM* still yielded +14pp on AIME-2025, but the strong results in the paper use stronger LMs. arxiv:2405.10276 generalizes: small LMs are bad optimizers.
- **Plateau after 2 iterations.** Galileo/blog data and general APO surveys note quality gains plateau after 2 iterations while cost rises. Mitigation: hard early-stopping on validation, not training, score.
- **Pareto-elite collapse.** MAP-Elites in GEPA depends on a meaningful cell-axis. If your axes correlate (e.g. determinism vs correctness on a deterministic task), the frontier degenerates and you waste rollouts. Mitigation: pick axes that genuinely trade off (latency vs accuracy, succinctness vs coverage).
- **Search-space leak across tasks.** Optimizer LM has prior knowledge of common benchmarks. arxiv:2510.08413 derives PAC-style bounds but only with informative priors. Mitigation: use ELAI-internal tasks (BabyStep DETERMINISTIC_CHECK signatures) which are not in any training set.
- **Stochastic-format rollouts cost.** AlphaEvolve uses stochastic formatting + meta-prompt diversity, costs are large. For ELAI, batch optimization is fine because templates persist in the archive.

## Anti-patterns

- Treating local Qwen 7B as the optimizer LM. arxiv:2405.10276 directly shows this fails. Always use a cloud-tier model for the OPTIMIZER role.
- Optimizing test/contract code along with prompts. If the optimizer can edit the deterministic check, it will trivialize the metric. Freeze contract blocks (LPO `<edit>` tags applied only to instructional sections).
- Porting DSPy to Rust for inference-time use. The `dspy-rs` and `DSRs` projects are early. Run optimization OFFLINE in Python, ship the optimized text (blake3-hashed) into the existing Rust template archive. Optimization is rare/batch, inference is hot - keep them separate (matches ELAI's "prompts are data" determinism-first stance).
- Optimizing without a held-out test set. DSPy explicitly recommends 20/80 train/val + separate test (https://dspy.ai/learn/optimization/overview/) for exactly this reason; main-LM training instincts (80/20) underweight validation.
- Optimizing without text feedback when GEPA/TextGrad are available. Scalar-only optimization (OPRO, MIPROv2 default) underperforms reflective text-feedback variants by ~10pp.
- Running >2 outer iterations without measuring validation lift each step. Plateau is documented; cost rises linearly.

## Open questions

1. Does GEPA-optimized role-prompt for Qwen2.5-Coder-7B on ELAI's BabyStep templates transfer across the 8 block types (GROUNDING vs PRE_GATE vs DETERMINISTIC_CHECK), or does each block need its own optimization run? Measurable: run GEPA on 1 block type, evaluate transfer to the other 7.
2. What is the actual cost (cloud reflection-LM tokens) per BabyStep template optimized to convergence? Measurable: budget caps + log token count per converged template; compare to fixed engineering hours of hand-tuning.
3. Does the BAI Thompson promotion pipeline correctly downweight optimized templates that overfit a single domain? Measurable: introduce a deliberate overfit template, observe whether the cross-domain canary blocks promotion.
4. Is the auditor's text rationale (PASS/BLOCK reasons) actually informative enough as GEPA text-feedback, or does it need to be reformatted? Measurable: A/B GEPA with raw auditor text vs LM-summarized auditor text as feedback.
5. Does LPO-style edit-tag freezing of DETERMINISTIC_CHECK survive a multi-step optimization without semantic drift (e.g. the optimizer rephrasing instructions in a way that makes the test trivial)? Measurable: per-iteration semantic-equivalence check on DETERMINISTIC_CHECK output behavior.
