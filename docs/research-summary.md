# Research Summary: LLM Temporal Knowledge Degradation, Code Freshness, and Agentic SE

Consolidated summary of three research documents covering temporal knowledge decay in LLMs, code generation staleness, and autonomous self-verification architectures for software engineering agents.

---

## Document 1: Temporal Knowledge Degradation — Mechanisms, Measurements, and Mitigation

### Core Problem

LLMs encode a static snapshot of knowledge bounded by training cutoff. This creates "temporal knowledge degradation" — a systemic failure in temporal generalization, not just missing facts. Models experience collapse in foundational reasoning when confronted with temporally shifted data, producing undetected hallucinations, deprecated software patterns, and overconfident outdated claims.

### Key Findings

**Early-Onset Degradation (pre-cutoff decay)**
- Factual accuracy drops ~20% even on events *before* the official cutoff, especially near the tail of the training window.
- Cause: decreasing density of high-quality training data toward the end of collection (paywalls, robots.txt, scraping restrictions post-Sept 2021).
- Critical: degradation persists even in closed-book reading comprehension where the answer is in-context. The model's internal world-state representation becomes misaligned, degrading reasoning on recent contexts even with explicit evidence.

**Temporal Blind Spots**
- Knowledge infusion is heterogeneous across training phases (pretraining, SFT, RLHF), resulting in multiple partial cutoff points within a single model.
- Smaller models have weaker post-cutoff refusal rates, making them more susceptible to temporal hallucinations.
- Tool: LLMLagBench — diagnostic for empirically identifying temporal boundaries.

**Simulated vs True Ignorance**
- Brier Scores: Explicit knowledge = 0.11, Simulated ignorance = 0.13, True ignorance = 0.26.
- 52% performance gap between simulated and true ignorance — system prompts cannot reliably "rewind" a model's latent state.
- Paradox: RLHF-optimized models produce the best-looking chain-of-thought reasoning traces but have the *worst* simulated ignorance fidelity. They rationalize memorized post-cutoff outcomes via plausible deductive steps rather than genuine time-constrained inference.
- "Memorability asymmetry": leakage is domain-dependent — geopolitical events (0.111 Brier difference) vs business metrics (0.010 difference).

**Quantification: Shapley-DCLR Metric**
- Decomposes rationale into atomic claims, categorizes by temporal verifiability, applies Shapley values (cooperative game theory) to measure each claim's marginal contribution to prediction.
- Isolates decision-critical temporal contamination from peripheral noise.

### Dynamic Benchmarks

| Benchmark | Focus |
|-----------|-------|
| FreshQA | Time-sensitive QA across 4 categories: never-changing, slow-changing, fast-changing, false-premise |
| RealTimeQA / StreamingQA | Continuous information flows from live time-stamped corpora |
| VersiCode | Environment-aware code generation under library version constraints (300 libraries, 2000+ versions, 9 years) |
| APIMig | Repository-level API migration across versions |
| BigCodeBench | Multi-turn library-intensive generation |
| Terminal-Bench | CLI navigation with current build environment awareness |

**Key result**: Cache-first search architectures = ~24% accuracy on FreshQA. Live-crawl indexing = ~79%. A 55-point gap proving "live search" is often a marketing obfuscation.

### Code Generation Staleness (empirical)

| Language | AI Failure Rate | Primary Failure Modes |
|----------|----------------|-----------------------|
| Java | 72% | Deprecated enterprise APIs, legacy Spring Boot, stale Maven deps |
| C# | 45% | Outdated .NET targets, deprecated async patterns |
| JavaScript | 43% | Stale React hooks, vulnerable npm packages, outdated Next.js routing |
| Python | 38% | Pandas/NumPy deprecations, legacy auth bypasses, outdated PyTorch APIs |

- ~40% increase in failure rate when prompted to use library versions released after pretraining window (VersiCode).
- AI-generated PRs contain 2.74x more security vulnerabilities than human-written code, driven by temporally stale deprecated security protocols.

### Mitigation Strategies

**1. TimeSPEC Architecture (multi-phase temporal verification)**
- Phase 1 (Generator): Temporally-filtered search, only docs before t_ref.
- Phase 2 (Supervisor): Decomposes rationale into atomic claims, detects temporal violations via external tools.
- Phase 3 (Regenerator): Rejects contaminated drafts, regenerates with validated claims only.
- Phase 4-5 (Resupervisor + Aggregator): Re-validates and synthesizes final prediction.
- Tested on: US Supreme Court forecasting, NBA salary estimation, stock return ranking.

**2. Knowledge Conflict Resolution (KCR)**
- Constructs structured relational paths from both retrieved (novel) and parametric (stale) memory.
- Scores competing logic paths via Jensen-Shannon divergence: l(R) = D_JS(P || Q).
- Trains alignment via PPO/GRPO — rewards reasoning aligned with highest-scoring logic path, penalizes temporal hallucinations.
- MICRO-ACT framework: hierarchical ReAct action space for granular conflict resolution at entity level.

**3. Continual Learning / Knowledge Editing**
- ROME (Rank-One Model Editing), MEMIT (Mass-Editing Memory in Transformer), MEND, SERAC — precision overwrite of specific stale facts in MLP blocks without full retraining.
- Combined with Contrastive Decoding for verification.
- Dynamic Governance (DMM-Gov): coordinates PEFT + model editing + RAG in auditable closed loop with Minimum Reproducibility Disclosure checks.

**4. STDec (Spatio-Temporal Stability Guided Decoding)**
- Training-free, token-adaptive threshold decoding for diffusion LLMs.
- Spatial-Aware: aggregates nearby decoded token states, lowers threshold for spatially stable tokens. Uses mean/triangular/Gaussian smoothing kernels.
- Temporal-Aware: monitors predicted token ID consistency across denoising steps, relaxes thresholds for temporally stable tokens.
- Results: 14.76x speedup on GPQA, 14.17x on MBPP without sacrificing accuracy.

**5. Calibrated Abstention / Reinforced Hesitation (RH)**
- Ternary reward structure: (+1, 0, -λ) — correct, abstain, wrong.
- λ controls behavioral regime:

| λ | Regime | Abstention Rate | Error Rate | Use Case |
|---|--------|----------------|------------|----------|
| 0 | Aggressive | 0% | ~15% | Creative generation |
| 1-5 | Balanced | 60% | <2% | General RAG, SWE |
| ≥10 | Conservative | >95% | ~0% | Oncology, legal, finance |

- BAS (Brier-based Abstention Score): proper scoring for asymmetric penalty on overconfident stale errors.
- SelectLLM: integrates selective prediction into fine-tuning.

**6. Visual Placebo Effect (MMA framework)**
- Multimodal models correctly abstain on text-only temporal conflicts but collapse when irrelevant visual data is added — visual presence bypasses epistemic safety filters.
- MMA mitigates via confidence-aware filtering and source reliability analysis, reducing error std dev by 35%.

**Medical/Scientific Application**
- Medea framework: multi-round deliberative process with LLM panel reconciling against continuously updated databases (Semantic Scholar, OpenAlex).
- Flamingo-CXR: precision oncology sensitivity to knowledge freshness for drug indications and clinical trial updates.

---

## Document 2: Mitigating Temporal Degradation and API Misuse in Code-Generating LLMs

### Core Problem

Models default to frequency-based retrieval from training distribution. If blake2 appeared 10,000 times and blake3 appeared 50 times near cutoff, the model mathematically favors blake2 — "library version hallucination." Parameter-efficient models (e.g., Gemma 2B) are especially vulnerable due to limited parametric capacity.

### Key Research (2023-2026)

**ICSE 2025: "LLMs Meet Library Evolution"**
- 7 LLMs, 145 API mappings, 8 Python libraries, 28,125 completion prompts.
- Metrics: API Usage Plausibility (AUP), Deprecated Usage Rate (DUR).
- Root cause: inability to suppress high-probability legacy tokens even when newer alternatives exist in vocabulary.

**ICML 2025: CodeSync Framework**
- Tracked API updates from pandas, numpy, scipy, tensorflow, torch, flask post-Jan 2023.
- 14 LLMs, 2,200 training samples, 3,300 test cases.
- Finding: DPO, ORPO, SimPO all show severe knowledge decay — post-training optimization alone is fundamentally insufficient for rapidly evolving API surfaces.

**ICLR 2026: FHE-CODER**
- LLMs call uncallable modules (e.g., random() from random module), misconfigure crypto parameters due to confusing historical usage with modern strict typing.

### Benchmarks

| Benchmark | Domain | Method | Focus |
|-----------|--------|--------|-------|
| GitChameleon 2.0 | Python (328 problems) | Execution-based VCG | Version-specific API compliance, 350+ hours of curated breaking changes |
| APIBench / Gorilla | 1,646+ APIs | Zero-shot, AST matching | Hallucination of deprecated/nonexistent API endpoints |
| PCBench (PCART) | Python params | Static + dynamic analysis | Parameter compatibility issues (ICSE 2024) |
| CodeSyncBench | Python (numpy, scipy, etc.) | Dynamic API tracking | Post-cutoff adaptation (ICML 2025) |
| BUMP / Byam | Java builds | Compilation error analysis | Breaking dependency updates |

**GitChameleon 2.0 details:**
- Version-Conditioned Generation (VCG) paradigm.
- 328 problems, 2021-2023 range.
- API evolution categories: argument/attribute changes (most common), function name changes, semantics/functional changes, new features.
- Dual-control: version stated in prompt + execution environment locked to that version.
- Metrics: Success Rate (execution pass rate) and API Hit Rate.

### Mitigation Approaches

**Lightweight Interventions (ICSE 2025)**
- ReplaceAPI: post-generation AST-level substitution of deprecated calls, forces LLM to regenerate surrounding code for new signature.
- InsertPrompt: pre-generation prompt injection identifying deprecated APIs and shifting logit probabilities away from legacy tokens.

**Diagnostic Pipelines**
- Byam (2025): detects compilation failures from breaking dependency updates in Java. Uses XML-tagged prompts: `<erroneous line>`, `<api diff>`, `<repair strategy>` with CoT. o3-mini achieved highest build success with structural diffs across 103 breaking updates.
- PCART (ICSE 2024): hybrid static/dynamic approach for Python API parameter compatibility. Handles positional vs keyword argument changes, outputs repaired project + compatibility report.

**Unstructured Knowledge Editing (UnKE) — ICLR 2026 / ICML 2025**
- Extends MEMIT with non-local block key-value storage and cause-driven optimization (vs term-driven).
- Edits final token directly while preserving surrounding context.
- Evaluated on UnKEBench and EditEverything.

### Coding Assistant Architectures

| System | Indexing | Context Strategy | Structural Tooling |
|--------|---------|-----------------|-------------------|
| Aider | AST (tree-sitter) | PageRank symbol sorting, bin-packed 4096-token repo map | Call signatures, definitions, references |
| Codebase-Memory | AST Knowledge Graph (SQLite) | MCP tool exposure | 14 structural query tools, Louvain community detection |
| Cursor | Semantic vector indexing | Repo-wide embedding RAG | Syntax-aware chunking, direct doc indexing |
| Continue.dev | Multi-file chat analysis | Config as Code (.continue/rules/) | Directory-level rule enforcement |

**Codebase-Memory (2026):**
- 66 languages via tree-sitter.
- Single SQLite database, incremental content-hash file watching.
- 83% answer quality vs 92% for file-exploration agents, but 10x less token consumption, 2.1x fewer tool calls.

**DocPrompting:** retrieves from frequently updated documentation pool based on user intent, injects into prompt. Consistently improves Exact Match accuracy and reduces hallucination vs zero-shot.

### Constrained Decoding

Intervenes at logit selection phase. Compiles constraints (JSON schema, CFG, regex) into FSMs/parsing stacks. Invalid tokens masked to -infinity.

| Library | Approach | Trade-offs |
|---------|----------|-----------|
| Outlines / Guidance | Complete automaton pre-computed | Fast sampling, but startup cost + memory overhead, complex regex can fail |
| XGrammar | Persistent parsing stack, context-independent pre-checks | 100x faster than earlier grammar libs, supports wider regex, integrated with vLLM/NVIDIA NIM |
| lm-format-enforcer | Python re module | Robust when C++ backend integration is difficult |
| SGLang | Programmatic constraint definitions | Heavy regex = compilation overhead |

**Pydantic-driven validation:** define target API signature as Pydantic model -> auto-generates JSON Schema -> passed to LLM client (vLLM guided_json / OpenAI structured outputs via Instructor) -> XGrammar enforces at token level.

### Gemma-2B Agent Blueprint (3-phase)

**Phase 1: AST Knowledge Graph**
- Rust tree-sitter bindings parse project -> extract signatures, structs, imports.
- Store in SQLite via MCP with caller-callee edges.
- PageRank for context construction -> Aider-style repo map within Gemma's context window.

**Phase 2: Proactive Tool-Use Validation**
- cargo search / pip index before finalizing generation plan.
- deprecated_patterns.yaml for team-specific rules (Continue.dev-style).
- InsertPrompt-style constraint injection into system prompt.

**Phase 3: Constrained Decoding**
- vLLM or TGI inference server with XGrammar backend.
- Pydantic enforcement layer: explicit permit/reject patterns for modern vs deprecated APIs.
- Logit-level masking guarantees only valid modern API syntax is output.

---

## Document 3: Autonomous Self-Verification and Knowledge Recency in SE Agents

### Core Problem

Error accumulation in agentic workflows is exacerbated by temporal decay. 2B models have lower "hallucination robustness" and over-rely on parametric memory rather than grounding in retrieved context. The architectural imperative: detect and correct stale/incorrect outputs before they reach user execution level.

### Self-Verification Paradigms

| Framework | Mechanism | Strength | Weakness |
|-----------|-----------|----------|----------|
| Reflexion | Episodic memory buffer + verbal self-critique from execution traces | 91% pass@1 on HumanEval | Oscillation on complex problems; reasoning repetition in small models |
| Self-Refine | Multi-dimensional self-feedback, iterative drafting without execution | Forced deliberate cognitive mode | Completely blind to post-cutoff deprecations (purely parametric) |
| CRITIC | Tool-interactive critiquing with external tool queries | Breaks staleness loop via environment anchoring | Susceptible to infinite tool-looping on ambiguous errors |
| CodeChain | Modular sub-component clustering, centroid selection across revisions | Isolates failing blocks | Computationally intensive for 2B models |
| CodeTree | Multi-persona tree search (Thinker/Solver/Debugger) | Balances exploration/exploitation | Context window exhaustion in deep traversal |

### SWE-Agent vs Agentless Divergence

**SWE-Agent paradigm:**
- Multi-turn, tool-heavy autonomous agents (AutoCodeRover, Live-SWE-Agent).
- Live-SWE-Agent: 77.4% solve rate on SWE-bench Verified.
- Problem for 2B: "serial collapse" — single misinterpreted tool output derails entire reasoning chain.

**Agentless paradigm:**
- Rigid 3-phase: localization -> repair -> patch validation.
- Deterministic scripts handle traversal/reading, LLM only generates targeted patches.
- 32% solve rate on SWE-bench Lite at $0.70/issue — outperformed all open-source agents at publication time.
- Adopted by OpenAI and DeepSeek for evaluating frontier models.
- Key insight: reasoning-intensive Agentless training induces transferable skill priors (precise editing, self-reflection).

**For 2B models:** Agentless-inspired verification is optimal. Discrete, verifiable steps >> extended context + complex state management.

### Staleness Detection Mechanisms

**LLM-as-a-Judge is fundamentally broken for deterministic SWE:**
- Same code snippet fed 5 times -> severity scores 6.2 to 9.1, different vulnerability categories.
- 23% category disagreement rate, severe false negatives (misses obvious SQL injections).
- Code-optimized LLMs have *higher* package hallucination rates when evaluating library usage.

**Deterministic alternatives:**

| Mechanism | Phase | Reliability |
|-----------|-------|-------------|
| Static Analysis (Pyright/Mypy) | Post-generation | Excellent — deterministic, high recall, zero variance |
| Freshness Scoring | Pre-generation (retrieval) | High — prevents legacy code from entering context |
| Nightly Knowledge Pruning | Offline maintenance | High — reduces search space noise |

**Freshness Scoring formula:**
```
Relevance Score = Cosine Similarity × Recency Decay Factor × Access Boost
```
Quadratic function profile with boost factor (e.g., 15 over 365 days).

### Pre-Generation Guardrails

**Test-Time Steering Vectors (TTSV)**
- Concept: abstract concepts (recency, security) are approximately linear directions in activation space.
- Computed via difference-in-means: v = (1/N) Σ (h_modern - h_deprecated)
- Injected at inference: h'_pos = h_pos + α·v
- Cross-modal transfer: text-only steering vectors guide multimodal models.
- Limitation: if target behavior isn't a coherent linear direction in latent space, vector degrades output. Overly strong α destroys coherence.
- Best use for 2B: broad binary guardrails (parametric reliance -> retrieved context reliance).

**Lookback Lens (classifier-guided decoding)**
- Measures attention weight ratio: context vs self-generated tokens.
- Linear classifier on attention features suppresses tokens representing ungrounded departure from RAG context.

**Probing-RAG**
- Prober on hidden states detects high epistemic uncertainty on API calls -> forces external search mid-decoding.

**Constrained Beam Search**
- Expands beam state space to track FSM constraint satisfaction alongside token probabilities.
- Any branch attempting deprecated imports or failing to progress toward mandatory API syntax -> probability = 0, pruned.

### Multi-Agent Verification

**Verification architectures ranked for SWE:**
1. Process Verification (step-by-step) — superior for SWE. Catches errors early, prevents cascade.
2. Reflexion (MAR with persona-guided critics) — effective for simple logic, oscillates on complex tasks.
3. Output Scoring (LLM-as-Judge) — fails due to fluency bias.
4. Adversarial Debate — excellent for strategy but 5x inference cost, shared blind spots with same training data.

**Verifier Cutoff Paradox resolution:**
- Completely divorce verifier authority from parametric memory.
- Verifier operates as deterministic orchestrator of external tools (cargo search, NPM registry, machine-readable changelogs).
- Cross-reference against pinned versions in .cursorrules or CLAUDE.md.
- Catches ~80% of stale pattern issues.

**Constitutional Spec-Driven Development:**
- spec.md (what to build + constitutional constraints) -> plan.md (how, with deprecation checks embedded) -> tasks.md (atomic constrained work items).
- Empirical result: 73% reduction in security defects and deprecated code vs unconstrained generation.

### Uncertainty-Triggered Retrieval

**Proactivity-driven vs Reflection-driven search:**
- Reflection: draft -> critique -> search to fill gaps.
- Proactivity (Search-o1): pause reasoning at knowledge gap -> search -> continue. Includes Reason-in-Documents module to distill results and minimize noise.

**Semantic Entropy / UnCert-CoT:**
- Groups syntactically different but semantically identical responses to measure true knowledge gaps.
- Pauses generation and activates retrieval when high epistemic uncertainty detected.
- +6.1% pass rate improvement on complex benchmarks.

**Entropy-Trend Constraint (ETC) Algorithm:**
- Computes first and second-order entropy derivatives: ΔH_t = H_t - H_{t-1}, Δ²H_t = ΔH_t - ΔH_{t-1}
- Detects accelerating uncertainty trends *before* hallucinated tokens are finalized.
- Triggers cargo search / NPM query via Decide-Then-Retrieve framework.
- Outperforms strong RAG baselines while reducing overall retrieval frequency.

### 2B Stack Blueprint (3-phase)

**Phase 1: Context Preparation**
- Nightly knowledge maintenance with quadratic freshness scoring.
- Constitutional spec-driven planning (plan.md with allowed libraries, version constraints, security principles).
- TTSV activation: pre-computed steering vector for "strict documentation adherence" + "modern API syntax" injected at middle transformer layers.

**Phase 2: Proactivity-Driven Generation**
- ETC monitoring: continuous first/second-order entropy derivatives during autoregressive generation.
- Dynamic tool invocation on ETC threshold breach (e.g., `npm view <package> versions`).
- Constrained beam search for import lines and security-critical configs — FSM based on retrieved package data.

**Phase 3: Auditor Agent (Deterministic Process Verification)**
- Independent 2B instance as Auditor under Agentless paradigm.
- Static analysis (Pyright/Mypy) with latest deprecation rules, not LLM judgment.
- Compilation error traces fed back to Generator for targeted repair.
- Hard cap: 3 retries max to prevent oscillation.

### Full Pipeline Summary

| Stage | Mechanism | Function |
|-------|-----------|----------|
| Data Ingestion | Quadratic Freshness Scoring | Deprioritize legacy embeddings in RAG |
| System Prompting | Constitutional Spec-Driven Constraints | Enforce rules by construction |
| Model Alignment | TTSV | Shift activations away from parametric staleness |
| Generation Loop | ETC Algorithm | Trigger live tool-searches before errors propagate |
| Decoding | Constrained Beam Search FSM | Force valid API output, override probabilistic drift |
| Verification | Static Analysis (Agentless) | Deterministic compilation, zero variance |

---

## Cross-Document Key Themes

1. **Parametric memory is structurally hostile to temporal accuracy** — not a bug but an architectural inevitability of static training.
2. **Post-training optimization (DPO, ORPO, SFT) is insufficient** for maintaining API freshness against rapid ecosystem evolution.
3. **RLHF worsens simulated ignorance** — models learn to rationalize memorized outcomes, not reason from constrained states.
4. **LLM-as-a-Judge is unreliable for deterministic code verification** — high variance, fluency bias, false negatives.
5. **Defense-in-depth is mandatory**: pre-generation (freshness scoring, steering vectors, constitutional specs) + generation (constrained decoding, ETC uncertainty triggers) + post-generation (static analysis, deterministic auditing).
6. **2B models need external orchestration, not internal reasoning** — offload verification to rigid architectural guardrails, not extended tool-looping.
7. **Calibrated abstention via Reinforced Hesitation** is the most robust safeguard — train models to know what they don't know, with domain-tunable λ penalty.
8. **AST knowledge graphs + MCP >> simple vector RAG** for code generation grounding, especially under tight context budgets.
