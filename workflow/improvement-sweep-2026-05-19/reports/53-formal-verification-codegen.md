# Formal verification for AI codegen in ELAI (May 2026)

## TL;DR
- Ship a narrow formal-method gate (NOT a universal one): wire Z3 SMT + Kani harness checks into the D-3 VerificationResult aggregator for tasks tagged safety-critical (auth, money, crypto, parsing-untrusted-input). Evidence: "Broken by Default" 2026 Z3-on-LLM-code study shows SMT catches CWE-131/190/916 classes that static analysis misses; Theorem (YC S25) used the same pattern to find a bug that slipped past Anthropic's own testing.
- For Rust-internal logic, Verus + AutoVerus is the only stack with >90% LLM-driven proof-annotation success on a real benchmark (90% on 150 tasks, OOPSLA2 2025). Borrow-checker already gives memory safety; Verus adds functional correctness on hot kernels.
- Use Dafny (not Lean) as a "verification-aware IR" for safety-critical Python paths. Vericoding benchmark (arXiv 2509.22908, MIT+others) measures 82% Dafny / 44% Verus / 27% Lean success — Dafny is the cheap entry point.
- Do NOT use Lean 4 / theorem-prover stacks as a default audit gate. AlphaProof needed ~500 TPU-days per Putnam problem; even Seed-Prover 1.5 took 16.5 hours of GPU time for 5 IMO problems. Wrong unit of latency for an AUDITING-state gate.
- Treat the gate as "AI-as-prover, ELAI-as-conjecturer" (CSQ counterfactual emits the spec, prover discharges it). Anti-pattern: making the LLM both write the spec and prove it — this is what DAFNYCOMP's 95.67%->3.69% syntax-vs-verification cliff exposes.

## Key sources
| arxiv/URL | what | takeaway |
|---|---|---|
| [Nature 2025 AlphaProof](https://www.nature.com/articles/s41586-025-09833-y) | DeepMind IMO 2024 silver, Lean 4 + AlphaZero, 80M auto-formalized statements | Proves LLM+RL+Lean can hit IMO silver, but TPU-day cost is prohibitive as a routine gate |
| [arXiv:2504.21801 DeepSeek-Prover-V2](https://arxiv.org/abs/2504.21801) | 671B MoE Lean 4 prover, subgoal-decomp RL | 88.9% on MiniF2F-test, 49/658 PutnamBench — open-weight ceiling, still narrow |
| [arXiv:2409.13082 AutoVerus](https://arxiv.org/pdf/2409.13082) | LLM agent network generating Verus proofs for Rust | >90% on 150-task benchmark, <30s or 3 LLM calls for half — production-feasible latency |
| [arXiv:2412.06176 AlphaVerus](https://arxiv.org/html/2412.06176v1) | Bootstrap verified Rust generation via Dafny->Verus translation + Treefinement tree search | LLaMA-3.1-70B reaches verified code generation without fine-tuning |
| [arXiv:2604.05292 Broken by Default](https://arxiv.org/html/2604.05292v2) | 3,500 LLM-generated artifacts, 7 LLMs, 500 prompts, Z3 SMT via COBALT | Z3 ground-truth exploitability; SMT > static analyzers on CWE classes |
| [arXiv:2509.22908 Vericoding benchmark](https://arxiv.org/abs/2509.22908) | 12,504 specs across Dafny/Verus/Lean; Tegmark et al. | 82% Dafny, 44% Verus, 27% Lean — NL descriptions do NOT meaningfully help |
| [arXiv:2507.16331 Re:Form](https://arxiv.org/abs/2507.16331) | RL-on-Dafny reduces human priors; DafnyComp benchmark | Even 0.5B model verifiably generates Dafny after SFT; RL generalizes OOD |
| [arXiv:2501.06283 Dafny as Verification-Aware IR](https://arxiv.org/pdf/2501.06283) | POPL 2025; Dafny as intermediate between NL spec and target code | Pure-Dafny verification went 68% -> 96% in one year |
| [arXiv:2404.12534 Lean Copilot](https://arxiv.org/abs/2404.12534) | LLM tactic/premise suggestion native in Lean 4; NeuS 2025 | suggest_tactics / search_proofs / select_premises — viable IDE-side aid, not a gate |
| [Kani](https://model-checking.github.io/kani/) + [AWS Open Source post](https://aws.amazon.com/blogs/opensource/how-open-source-projects-are-using-kani-to-write-better-software-in-rust/) | CBMC-backed bounded model checker for Rust; AutoharNess feature 2025 | Property-style harnesses, native cargo integration, deterministic latency |
| [Verus (MSR)](https://www.microsoft.com/en-us/research/publication/verus-a-practical-foundation-for-systems-verification/) | SMT-based deductive verifier for Rust, linear ghost types | Two OSDI 2024 best papers built on it; Microsoft+Amazon industrial use |
| [arXiv:2501.03124 PRMBench](https://arxiv.org/abs/2501.03124) | 6,216 problems / 83,456 step labels; soundness + sensitivity (incl. Deception Resistance) | SOTA PRMs still miss deceptive/plausible-but-wrong steps — formal check fills that gap |
| [arXiv:2603.21149 Emergent Formal Verification](https://arxiv.org/abs/2603.21149) | Autonomous AI ecosystem independently chose Z3 across 6 safety domains | Convergent evidence that Z3-on-LLM-output is the right primitive |
| [arXiv:2511.17330 Agentic Program Verification](https://arxiv.org/html/2511.17330) | Proof agents communicating with Rocq; 1,717 CoqGym-intersect lemmas + 641 SV-COMP | Shows agentic Rocq loop pattern; latency still research-grade |
| [arXiv:2412.06512 LLM+Formal Methods Roadmap](https://arxiv.org/html/2412.06512v1) | Survey of fusion patterns for trustworthy agents | Roadmap citation for agentic-formal coupling |
| [arXiv:2507.23726 Seed-Prover](https://arxiv.org/pdf/2507.23726) + [v1.5 blog](https://seed.bytedance.com/en/blog/seed-prover-1-5-advanced-mathematical-reasoning-through-a-novel-agentic-architecture) | ByteDance lemma-style whole-proof, Lean 4 | 99.6% MiniF2F; 35/42 IMO 2025 in 16.5h — agentic, not gate-latency |
| [Martin Kleppmann 2025](https://martin.kleppmann.com/2025/12/08/ai-formal-verification.html) | Prediction: LLMs will make formal verification mainstream | Independent expert framing the same hypothesis |
| [Theorem YC S25 / VentureBeat](https://venturebeat.com/security/theorem-wants-to-stop-ai-written-bugs-before-they-ship-and-just-raised-usd6m) | $6M seed; caught a bug that slipped past Anthropic testing | Existence proof of commercial demand and undetected-by-tests catch |

## Tool landscape
| Tool | Lang | Maturity | AI-codegen integration | Citation |
|---|---|---|---|---|
| Lean 4 + mathlib | Lean | mature, math-heavy | AlphaProof, DeepSeek-Prover, Seed-Prover, Lean Copilot | [AlphaProof Nature 2025](https://www.nature.com/articles/s41586-025-09833-y), [Seed-Prover](https://arxiv.org/pdf/2507.23726) |
| Verus | Rust | industrial (MSR, AWS) | AutoVerus, AlphaVerus | [AutoVerus](https://arxiv.org/pdf/2409.13082), [AlphaVerus](https://arxiv.org/pdf/2412.06176) |
| Kani | Rust | stable v0.x, AWS-maintained | Autoharness (2025), LLM-generated harness pattern | [Kani repo](https://github.com/model-checking/kani) |
| Creusot | Rust | research, active 2026 | minimal LLM tooling | [Frama-C Creusot 2026](https://rust-formal-methods.github.io/) |
| Dafny | Dafny | mature (Microsoft) | DafnyPro, Re:Form, AutoFormalize | [DafnyPro POPL '26](https://www.arxiv.org/pdf/2601.05385) |
| F* | F* | mature, niche | EverCrypt; no public LLM gate | (sparse 2025-2026 LLM coverage) |
| Coq / Rocq | Rocq | mature; rebrand 2024 | LLM4Rocq, Strat2Rocq, agentic verification | [arXiv:2510.10131](https://arxiv.org/pdf/2510.10131), [arXiv:2511.17330](https://arxiv.org/pdf/2511.17330) |
| Z3 | SMT | de-facto standard | LDSW (MCTS+Z3), AquaForte LLM-guided | [arXiv:2601.04675](https://arxiv.org/pdf/2601.04675) |
| CVC5 | SMT | mature | mlcvc5 GNN guidance; AquaForte +183% solved | [mlcvc5-LPAR](https://github.com/JellePiepenbrock/mlcvc5-LPAR) |
| Isabelle/HOL | Isabelle | mature | PutnamBench dual-formalization; less LLM SOTA than Lean | [PutnamBench arXiv:2407.11214](https://arxiv.org/pdf/2407.11214) |
| Idris / Agda | dep-typed | niche | no major 2025-2026 LLM benchmark | n/a |
| Boolector / Yices | SMT | mature | less LLM tooling than Z3/CVC5 | n/a |

## AI-as-prover SOTA (May 2026)
- MiniF2F-test: DeepSeek-Prover-V2-671B 88.9%; Seed-Prover 99.6% (medium settings); informal-reasoning LLMs ~49.1%. Sources: [arXiv:2504.21801](https://arxiv.org/abs/2504.21801), [arXiv:2507.23726](https://arxiv.org/pdf/2507.23726).
- PutnamBench (1697 formalizations / 640 problems Lean+Isabelle): GPT-5 7% (42/660); DeepSeek-Prover-V2 49/658; AlphaProof 56% but ~500 TPU-days per problem. Source: [PutnamBench](https://trishullab.github.io/PutnamBench/), [arXiv:2407.11214](https://arxiv.org/pdf/2407.11214).
- IMO 2024 / 2025: AlphaProof+AlphaGeometry2 silver 28/42 (2024); Seed-Prover 1.5 gold 35/42 (2025), 16.5h wall time on GPU. Sources: [DeepMind blog](https://deepmind.google/blog/ai-solves-imo-problems-at-silver-medal-level/), [Seed v1.5](https://seed.bytedance.com/en/blog/seed-prover-1-5-advanced-mathematical-reasoning-through-a-novel-agentic-architecture).
- Vericoding (12,504 specs): off-the-shelf LLMs reach 82% Dafny / 44% Verus / 27% Lean. NL hints don't help. Source: [arXiv:2509.22908](https://arxiv.org/abs/2509.22908).
- AutoVerus: >90% on 150 nontrivial Verus proof tasks, half in <30s or <=3 LLM calls. Source: [arXiv:2409.13082](https://arxiv.org/pdf/2409.13082).
- DafnyBench: Claude Sonnet 3.5 + DafnyPro 86% correct proofs (+16pp). Source: [DafnyPro POPL '26](https://www.arxiv.org/pdf/2601.05385).

## When formal verification actually catches bugs (vs theater)
- "Broken by Default" (arXiv:2604.05292): 3,500 artifacts / 7 LLMs / 500 prompts via Z3 COBALT pipeline — SMT establishes ground-truth exploitability for CWE-131 (buffer-size calc), CWE-190 (integer overflow), CWE-916 (weak password hash). Static analyzers miss these classes systematically.
- Shiplight 2025 data: AI-generated code = 1.7x more bugs/PR; modern review tools catch 42-48% of real runtime bugs vs <20% for traditional static analyzers ([Shiplight](https://www.shiplight.ai/blog/ai-generated-code-has-more-bugs)). Formal checks are the upper band of that 42-48%.
- Theorem (YC S25): commercial deployment found a bug that passed Anthropic's own testing ([VentureBeat](https://venturebeat.com/security/theorem-wants-to-stop-ai-written-bugs-before-they-ship-and-just-raised-usd6m)) — existence proof that formal layer catches what example-tests don't.
- Emergent Formal Verification (arXiv:2603.21149): independent autonomous AI ecosystems converged on Z3 for 6 safety domains including LLM-codegen — convergence evidence for the gate primitive.
- PRMBench (arXiv:2501.03124) "Deception Resistance" dimension: state-of-the-art PRMs miss misleading-but-plausible steps. A Z3 / Kani / Verus check is exactly the layer that doesn't fall for plausible-but-wrong arguments — it computes.

## Best additions for ELAI
Ranked by ROI for ELAI's actual architecture:

1. **Z3 SMT gate inside D-3 VerificationResult aggregator**. Technique: encode safety-critical-path postconditions into Z3 constraints; counts as 1 of the ≥2 verifier consensus votes. Grounding: [arXiv:2603.21149](https://arxiv.org/abs/2603.21149), [arXiv:2604.05292](https://arxiv.org/pdf/2604.05292). Integration hook: new `VerifierKind::Z3SmtCheck` variant. Latency: sub-second on bounded properties. Only fires when CSQ classifies the function as Critical (financial, auth, crypto, parser).

2. **Kani harness for Rust hot paths**. Technique: Kani Autoharness 2025 generates bounded model-checking harnesses; LLM proposes property predicates from the spec. Grounding: [Kani repo](https://github.com/model-checking/kani), [AWS post](https://aws.amazon.com/blogs/opensource/how-open-source-projects-are-using-kani-to-write-better-software-in-rust/). Integration hook: AUDITING substate, runs `cargo kani --harness <fn>` on functions marked `#[cfg_attr(kani, kani::proof)]`. Auditor reports verification status alongside cargo test.

3. **AutoVerus-style Verus annotation for ELAI's core invariants**. Technique: spec ELAI's FSM invariants, IPC-v2 protocol invariants, KV-cache size invariants in Verus; LLM generates proof annotations under bounded retries. Grounding: [arXiv:2409.13082](https://arxiv.org/pdf/2409.13082) (>90% / 150 tasks). Integration hook: a `verus-check` job in CI that gates COMMITTING, NOT every audit. Anti-rubber-stamp FINDING fires if a proven invariant is later violated.

4. **Dafny IR as MediaValidator companion for Python sidecar safety-critical paths**. Technique: route financial / auth Python code through Dafny IR ([arXiv:2501.06283](https://arxiv.org/pdf/2501.06283)); MediaValidator becomes a SpecValidator that confirms Dafny output matches the input semantics. Grounding: [Re:Form arXiv:2507.16331](https://arxiv.org/abs/2507.16331) (even 0.5B verifies Dafny). Integration hook: CSQ Critical path -> Dafny pre-step before Python emission.

5. **Lean Copilot suggestion mode as an OPTIONAL premise hint to auditor**. Technique: when a verification BLOCK fires, fall back to Lean Copilot `select_premises` for hint generation. Grounding: [arXiv:2404.12534](https://arxiv.org/abs/2404.12534). Integration hook: auditor's BLOCK message includes a "premise hint" subsection. Cheap. Read-only.

6. **PRMBench-style Deception Resistance test for the auditor itself**. Technique: inject deceptive-but-plausible-looking diffs into a synthetic dataset; require auditor anti-rubber-stamp to produce ≥1 FINDING on every one. Grounding: [arXiv:2501.03124](https://arxiv.org/abs/2501.03124). Integration hook: nightly canary in contracts/, not per-audit.

## Where it doesn't help (and adds latency)
- UI / dashboard / frontend code: vericoding 27% Lean / 44% Verus rates are dominated by algorithmic kernels — UI logic is mostly glue ([arXiv:2509.22908](https://arxiv.org/abs/2509.22908)). Don't gate Svelte components on Verus.
- Long-tail web-scraping / retrieval / LLM-prompt code: no formal spec exists for "did this prompt do what I meant", so SMT/Verus has nothing to check. Use D-3 self-consistency + semantic validators instead.
- Lean as a gate: AlphaProof's ~500 TPU-days/problem ([Berkeley TR 2025](https://www2.eecs.berkeley.edu/Pubs/TechRpts/2025/EECS-2025-121.pdf)) and Seed-Prover 16.5h IMO time ([Seed v1.5](https://seed.bytedance.com/en/blog/seed-prover-1-5-advanced-mathematical-reasoning-through-a-novel-agentic-architecture)) put theorem-prover loops out of AUDITING latency budget. Lean is for offline grounding of math content (paper-ingestor mode), not per-audit gating.
- Pure Python ML training scripts: model.fit() and torch loops don't have falsifiable properties beyond shape/dtype — already caught by typecheck + smoke test.
- DAFNYCOMP cliff (95.67% syntax -> 3.69% verified, [DafnyPro](https://www.arxiv.org/pdf/2601.05385)): if the LLM both writes the spec and the proof, you're not verifying — you're getting Hallucinated Confidence. The spec must come from the user or from a non-prover oracle (CSQ counterfactual, contract test).

## Anti-patterns
- LLM writes spec + proof, gate accepts both (DafnyCOMP cliff, [DafnyPro](https://www.arxiv.org/pdf/2601.05385)) — circular trust. Spec must be produced by a different agent or be human-authored.
- Theorem-prover as default audit (AlphaProof TPU-day cost, [Berkeley TR](https://www2.eecs.berkeley.edu/Pubs/TechRpts/2025/EECS-2025-121.pdf)) — wrong unit of latency.
- "It compiles, therefore it's correct" via Verus skeleton without nontrivial postconditions ([AlphaVerus](https://arxiv.org/pdf/2412.06176)) — proof of triviality is not proof of correctness.
- Replacing example tests with formal proofs ([Kleppmann 2025](https://martin.kleppmann.com/2025/12/08/ai-formal-verification.html)) — proofs verify the spec, tests verify your reading of the requirement. Need both.
- Iterating LLM with solver-error feedback without bounded retries ([LDSW arXiv:3788734](https://dl.acm.org/doi/full/10.1145/3788731.3788734)) — agent thrashes. Cap at 3 retries, escalate to FINDING.
- Treating decompiled / reverse-engineered specs as ground truth — per source-grounding rule, decompiled = LOW reliability ([CLAUDE.md source-grounding]).

## Open questions
1. Does adding a Z3 gate to D-3's anti-rubber-stamp aggregator reduce false-positive FINDINGs vs hurt them? Measure on PRMBench Deception Resistance subset.
2. What is the actual MiniF2F-equivalent for ELAI's IPC v2 / SSB / FSM invariants — can we autoformalize the spec into Verus in <100 LOC per crate?
3. Below what wall-clock budget per audit (sub-5s? sub-30s?) does Z3 + Kani stop blocking the EXECUTING -> AUDITING -> COMMITTING loop in practice?
4. Does Dafny-IR-as-Python-safety-step measurably lower bug rate on the financial/auth slices vs Python type-hints + property tests alone? AB on a real ticket.
5. Can the local 3B Qwen (NF4) successfully generate Verus proof annotations on simple invariants, or does this gate require routing to a cloud model? (Re:Form's 0.5B Dafny SFT result is suggestive but Verus is harder than Dafny per vericoding benchmark.)
