# Agent eval benchmarks for ELAI (2024-2026)

Scope: best-in-class agent evals for SOTA grounding of ELAI's internal D-19 to D-25 mechanisms.
Date of capture: 2026-05-19. All numerical scores are May 2026 leaderboard snapshots.

## TL;DR

- Ship **SWE-bench-Live (Multi-Language) + LiveCodeBench v6** for coding-agent regression (contamination-free, monthly refresh; Verified is now considered contaminated and OpenAI has stopped reporting it).
- Ship **tau2-bench (retail/airline/telecom) + BFCL v3** for tool-use + policy-adherence (current SOTA is 0.993 telecom / 0.862 retail Anthropic; both directly map to ELAI's mode-routing + tool layer).
- Add **GAIA Level 1/2 + OSWorld-Verified slice** as the general-agent gate; ELAI's D-21 mode-routing coverage benchmark is internal — these are the external anchors.
- Adopt **HLE (no-tools) + GPQA-Diamond** as the reasoning ceiling probe; ARC-AGI-2 is too hard to use as a regression signal for a 7B local model (24% best human-tuned SOTA, $0.20/task ceiling).
- Treat **ClawsBench-style harness/model decomposition** as a mandatory companion to every eval — scaffolded models cluster within a 10pp band, so an ELAI eval that doesn't fix-then-vary-then-fix the harness will measure the wrong thing.

## Key sources / leaderboards

| URL | benchmark | takeaway |
|---|---|---|
| https://www.swebench.com/ | SWE-bench (Verified/Pro/Multimodal/Live) | Verified is now considered contaminated; OpenAI stopped reporting. Pro and Live are the live signal. |
| https://swe-bench-live.github.io/ | SWE-bench-Live | Monthly-refreshed; Multi-Language and Windows leaderboards live as of 2026. Direct citation: arXiv:2505.23419. |
| https://livecodebench.github.io/ | LiveCodeBench (v6) | Contamination-free LeetCode/AtCoder/CodeForces; 218 models scored; mean 50.7, sd 23.5 (arXiv:2403.07974). |
| https://artificialanalysis.ai/evaluations/livecodebench | LiveCodeBench leaderboard | May 2026: Gemini 3 Pro Preview 91.7%, DeepSeek-V4-Pro-Max 93.5%. |
| https://huggingface.co/papers/2601.10904 | ARC Prize 2025 / ARC-AGI-2 | Best private-eval score 24%, $0.20/task; 1455 teams; 2025 motif = refinement loops. |
| https://hal.cs.princeton.edu/gaia | GAIA (HAL) | Claude Sonnet 4.5 leads at 74.6%; Anthropic sweeps top 6. arXiv:2311.12983. |
| https://gorilla.cs.berkeley.edu/leaderboard.html | BFCL v3/v4 | GLM 4.5 #1 v3 at 76.7; Qwen3-32B 75.7 — local-7B reachable. |
| https://github.com/sierra-research/tau2-bench | tau2-bench | Retail 0.862 / Airline 0.700 / Telecom 0.993; 38 model entries (Apr 2026). |
| https://benchlm.ai/benchmarks/osWorldVerified | OSWorld-Verified | Holo3-35B-A3B 82.6%, Claude Mythos 79.6%, GPT-5.5 78.7%. |
| https://artificialanalysis.ai/evaluations/humanitys-last-exam | HLE | Claude Opus 4.7: 46.9% no-tools / 54.7% with tools; Mythos 64.7%. |
| https://labs.scale.com/leaderboard/humanitys_last_exam | HLE (Scale) | Cross-check leaderboard, lower scores: Gemini 3.1 Pro 44.7%, GPT-5.5 xhigh 44.3%. |
| https://arxiv.org/abs/2602.12413 | Soft Contamination paper | 78% of CodeForces and 50% of ZebraLogic have semantic duplicates that 9-gram filters miss. |
| https://gradientscience.org/gsm8k-platinum/ | GSM8K-Platinum | 110/219 flagged GSM8K items were broken; "platinum" cleanup reveals real capability gaps. |
| https://clawsbench.benchflow.ai/ | ClawsBench | Top-5 models within 10pp on success (53-63%) but 7-23% unsafe; scaffold effect +39-63pp dwarfs model spread. arXiv:2604.05172. |
| https://focused.io/lab/agent-benchmarks-measure-the-harness | Harness-vs-model critique | "Agent benchmark scores are measuring the harness, not the model" — directly supports ELAI's per-mode auditor approach. |
| https://arxiv.org/abs/2310.13548 | Sharma sycophancy probes | Source for D-23; multi-behavior (feedback, mimicry, answer-flip, false-claim agreement). |
| https://arxiv.org/abs/2501.03124 | PRMBench | 6216 problems, 83456 step-labels; baseline source for D-19 adversarial PPM design. |
| https://arxiv.org/abs/2601.18137 | DeepPlanning | Long-horizon multi-day planning with verifiable constraints; closes gap NOT covered by SWE-bench. |
| https://arxiv.org/pdf/2510.05566 | DS-CP (Domain-Shift Conformal) | Reweights calibration samples under shift — direct reference for ELAI D-22 calibration validation. |
| https://aclanthology.org/2026.eacl-long.106.pdf | LLM Uncertainty & Calibration benchmarking | EACL 2026 reference for ECE/MCE on instruction-tuned vs base. |

## Benchmark landscape table

| benchmark | what it measures | known leak risk | local-7B feasible? | contamination defense | citation |
|---|---|---|---|---|---|
| SWE-bench Verified | Patch a real GitHub issue in repo, pass tests | HIGH (OpenAI stopped reporting; gold-patch verbatim reproduction observed) | Yes (harness-dependent) | None (frozen 2024 snapshot) | swebench.com; OpenAI 2024 scrub |
| SWE-bench Pro | Same task, 1865 multi-language, harder | LOW-MED (held-out, larger) | Yes | Held-out, multi-language | morphllm.com/swe-bench-pro |
| SWE-bench-Live | Monthly-refreshed real issues | LOW | Yes | Monthly refresh from real GH | arXiv:2505.23419 |
| SWE-bench Multimodal | Issues with visual elements | MED | No (needs vision) | Multimodal newer corpus | llm-stats.com/swe-bench-multimodal |
| LiveCodeBench v6 | Competitive programming + self-repair + test prediction | LOW | Yes | Time-stamped, monthly | arXiv:2403.07974 |
| BigCodeBench | Function calls + complex instructions | MED | Yes | EvalPlus-derived, 80x tests | arXiv:2406.15877 |
| HumanEval+/MBPP+ | Function gen with strengthened tests | HIGH (HumanEval is over-trained) | Yes | 80x more test cases per problem | evalplus.github.io |
| GAIA | Real-world assistant tasks, multi-step + tool | MED | Levels 1-2 yes; L3 no | Held-out test set | arXiv:2311.12983 |
| BFCL v3/v4 | Multi-turn function calling, state-based eval | MED | Yes | 1000 cases, state-based grading | gorilla.cs.berkeley.edu |
| tau2-bench | Tool-agent-user interaction + policy adherence | LOW | Yes | Simulated user + policy rubric | sierra-research/tau2-bench |
| WebArena | Realistic web environments | MED | Possible (slow) | Containerized realistic sites | github.com/web-arena-x/webarena |
| VisualWebArena | Multimodal web tasks | MED | No (needs vision) | Containerized + visual | github.com/web-arena-x/visualwebarena |
| OSWorld-Verified | Desktop OS control | MED | Possible (slow) | Held-out OS tasks | benchlm.ai/benchmarks/osWorldVerified |
| Mind2Web | Web action prediction | MED | Limited | Held-out | various |
| ARC-AGI-2 | Abstract reasoning OOD | LOW | Marginal (24% best SOTA) | Hand-crafted private set | arXiv:2601.10904 |
| HLE | Frontier expert exam | LOW-MED | Marginal | Manually curated, blind grading | scale.com / artificialanalysis |
| FrontierMath | Expert math problems | LOW | No | Held-out, manually graded | epoch.ai |
| GPQA-Diamond | Google-proof QA | MED | Marginal | 198 q diamond; reviewed | various |
| MATH / GSM8K-Platinum | Math word problems, audited | HIGH on GSM8K, LOW on Platinum | Yes | Platinum: 110/219 corrected | gradientscience.org/gsm8k-platinum |
| TruthfulQA | Honesty under common misconceptions | HIGH (widely trained on) | Yes | Adversarial Q design | original 2109.07958 |
| Sharma Sycophancy Probes | 4-mode sycophancy | MED | Yes | Probe templates, swappable | arXiv:2310.13548 |
| PRMBench | Step-level error detection by PRMs | LOW | Yes | 6216 q × 83456 labels | arXiv:2501.03124 |
| ClawsBench | Capability + safety with full scaffolding | LOW | Yes | 5 mock services, 33 conditions | arXiv:2604.05172 |
| DeepPlanning | Multi-day plan with verifiable constraints | LOW | Yes | Constraint-checked tasks | arXiv:2601.18137 |
| Tau2 Telecom | Tool-policy in telecom | LOW | Yes | Policy rubric grading | sierra-research/tau2-bench |
| AgentBoard | Multi-turn, partial-observability | MED | Yes | Held-out scenarios | arXiv:2401.13178 |
| Aider Polyglot | Cross-language edit-and-verify | MED | Yes | Curated polyglot tasks | aider docs |

## What's NOT well-measured

- **Multi-day / multi-session persistence.** Every cited eval is single-session. DeepPlanning (arXiv:2601.18137) is the only 2026 benchmark that tries "2–7+ day" horizons, and it still emulates them in one rollout. ELAI's actual workload (B-N plans over weeks) has no public eval analog.
- **Cost-aware quality.** No public Pareto frontier of $/task vs pass-rate as of May 2026; ARC Prize 2025 ($0.20/task constraint) is the closest. The first benchmark to publish a $/task Pareto is expected to set the bar. (decodethefuture.org/en/ai-agent-benchmarks-2026)
- **User trust over weeks.** No benchmark measures longitudinal user-trust drift; sycophancy probes (Sharma 2310.13548) measure single-turn flip rates, not "user gradually relies more, then catches a wrong answer" failure modes.
- **Harness vs model attribution.** ClawsBench is the first to formally decompose this (+39-63pp scaffolding effect vs 10pp model spread), but it's the exception — most leaderboards still report a model+harness blob.
- **Soft contamination.** arXiv:2602.12413 demonstrates 78% of CodeForces and 50% of ZebraLogic items have semantic duplicates in common training corpora that 9-gram filters miss. Any benchmark whose problem statements pre-date the model's training cutoff is suspect unless paraphrase-decontaminated.
- **Calibration under shift.** Standard ECE/MCE reporting on the source-domain test set is the default; DS-CP (2510.05566) and ORCA (2604.01170) show standard conformal prediction under-covers severely under shift. ELAI's D-22 needs this explicitly.
- **Policy adherence vs task completion.** tau2-bench is the only enterprise-style eval that grades on policy, not on outcome — most evals would mark "booked flight, ignored change-fee rule" as a pass. Real deployments care about policy.
- **Long-context reasoning at 25-200K.** AgencyBench (arXiv:2601.11044) targets 1M-token contexts but is niche; ELAI's 25K hard ceiling means most public long-context evals are over-scoped.

## Recommended eval suite for ELAI

Ranked by integration value. Each entry: benchmark | integration hook | what it gates | grounded threshold.

1. **SWE-bench-Live (Multi-Language, current month)** | run via existing CLI run_subcommand pipeline against the last 50 fresh issues | gates promotion of any B-N change that touches indexing/dep_audit/symbol_graph | threshold: must not regress more than 2 points vs prior month (paired bootstrap p<0.05). Cite: arXiv:2505.23419.
2. **LiveCodeBench v6 (last 60 days slice)** | hook into existing PerRoleRuntime executor | gates the executor + planner pipeline; specifically self-repair and test-output-prediction sub-tasks | threshold: hit floor at 30% on local-7B abliterated, 45% with cloud route. Cite: arXiv:2403.07974.
3. **tau2-bench (telecom + retail)** | wire into orchestrator's policy-check pass | gates D-21 mode-routing + tool layer end-to-end; policy adherence is the rubric, not task completion | threshold: retail >=0.70, telecom >=0.85 (cloud); locally >=0.55. Cite: sierra-research/tau2-bench + Anthropic's 0.862/0.993 SOTA.
4. **BFCL v3 multi-turn slice** | wire into the runtime adapter test suite | gates per-role function-calling correctness; complements D-21 internal | threshold: >=70% (GLM 4.5 SOTA is 76.7, Qwen3-32B 75.7). Cite: gorilla.cs.berkeley.edu.
5. **GAIA Level 1 + 2 (HAL framework)** | new contract-test fixture under crates/contracts | gates general-agent gauntlet for promotion to any "shipped" tag | threshold: Level 1 >=50%, Level 2 >=30% (Anthropic 74.6% combined SOTA). Cite: arXiv:2311.12983, hal.cs.princeton.edu.
6. **PRMBench (D-19 grounding)** | already aligned to D-19 PPM; use as adversarial gold-set | gates the PPM's calibration after any training-data change | threshold: F1 >=0.40 on simplicity + soundness + sensitivity. Cite: arXiv:2501.03124.
7. **Sharma sycophancy probe set (D-23)** | already aligned; subset of 4 behaviors | gates promotion when the auditor or planner persona changes | threshold: flip rate <15% on TruthfulQA-derived feedback probes. Cite: arXiv:2310.13548.
8. **GSM8K-Platinum + MATH-500** | reasoning floor probe | gates the reasoning-mode subset of mode-routing | threshold: GSM8K-Platinum >=90%, MATH-500 >=50% locally. Cite: gradientscience.org/gsm8k-platinum.
9. **HLE no-tools (small slice, ~50 q)** | quarterly ceiling probe only | NOT a regression gate; tracks ceiling drift | threshold: trend only; SOTA is 46.9% no-tools Claude Opus 4.7. Cite: scale.com/labs/leaderboard.
10. **ClawsBench-style harness/model split** | run before and after any harness change | gates ANY harness/scaffolding change (executor, auditor prompt, runtime adapter) | threshold: if model spread within harness is <10pp (per ClawsBench), the change is harness-attributable. Cite: arXiv:2604.05172.
11. **DeepPlanning multi-day slice** | new long-horizon contract test | gates B-N plans that span phases | threshold: constraint-satisfaction >=70% on 2-3 day plans. Cite: arXiv:2601.18137.
12. **DS-CP coverage under shift (D-22)** | already in scope; replace plain conformal with reweighted-by-proximity | gates the calibration validation pass | threshold: coverage gap <5pp under benchmarked shifts. Cite: arXiv:2510.05566.
13. **OSWorld-Verified small slice (10 tasks)** | optional desktop-control probe via runtime adapter | gates any future "computer-use" role | threshold: SOTA reference is Holo3-35B-A3B 82.6%; ELAI floor 30%. Cite: benchlm.ai/benchmarks/osWorldVerified.

## Anti-patterns

- **Reporting Verified-only SWE-bench scores.** OpenAI's 2024 scrub + subsequent withdrawal makes this misleading. Always pair with Live or Pro. (morphllm.com/swe-bench-pro; tokenmix.ai/blog/swe-bench-2026)
- **Static benchmark + LLM-as-judge without bias controls.** Position bias, ID-token bias, self-preference bias all measured at significant magnitudes (arXiv:2410.15393, 2504.03846). ELAI's auditor must rotate option positions and IDs in any LLM-judged eval.
- **N-gram-only decontamination.** Misses 78% of CodeForces semantic duplicates (arXiv:2602.12413). Use embedding-based paraphrase detection.
- **GSM8K (raw) as a hard gate.** 13% accuracy drop on parallel datasets shows systematic overfitting (arXiv:2405.00332). Use GSM8K-Platinum.
- **Harness-coupled headline scores.** ClawsBench shows scaffold effect (+39-63pp) dwarfs model effect (10pp). Never report a model score without freezing harness state. (focused.io/lab/agent-benchmarks-measure-the-harness)
- **One-shot evals on long-horizon work.** Single-rollout success is uncorrelated with multi-day persistence; DeepPlanning is the only public eval that tries to bridge this. (arXiv:2601.18137)
- **Ignoring policy adherence.** Booking a flight while violating the change-fee policy passes most evals and fails enterprise deployments. tau2-bench is the corrective. (medium.com/alan/benchmarking-ai-agents-stop-trusting-headline-scores)
- **Using HLE / FrontierMath / ARC-AGI-2 as regression gates on local 7B.** Frontier-only benchmarks; 7B models score in single-digit % and noise dominates. Use as ceiling-drift trackers, not gates.

## Open questions

1. What's the smallest SWE-bench-Live slice (n) that yields paired-bootstrap p<0.05 on a 2-point delta? (Need to power the regression gate without burning a week of GPU.)
2. Does ELAI's per-mode routing close the +39-63pp scaffolding gap ClawsBench measures? Run with-vs-without mode-routing on the same model to attribute.
3. What's the right paraphrase-decontamination threshold for ELAI's internal D-24 canary regression suite (cosine on which encoder, what cutoff)?
4. Can DS-CP (arXiv:2510.05566) coverage-under-shift numbers be reproduced on ELAI's existing calibration corpus before integration?
5. Does the tau2-bench policy-adherence rubric transfer to ELAI's internal SSB-action policy (e.g., never write outside .elai_cc), or does it need a custom rubric layer?
