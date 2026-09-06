harness-as-pipe prior art survey
date: 2026-06-14
question: Has anyone deliberately built a coding harness that minimizes the LLM's role to pure code generation, pushing everything else into deterministic non-LLM harness code? Is this named, validated, or a known dead-end?
searches: 14, fetches: 18


VERDICT

The thesis is a known, empirically-validated approach with multiple instantiations, a growing named discipline ("harness engineering"), and measurable payoffs. The closest single piece of prior art is Agentless (Xia et al., arXiv:2407.01489), which replaced open-ended agentic tool-use with a fixed three-phase deterministic pipeline and measured SOTA on SWE-bench Lite at ~$0.34/issue. The broader cluster includes LLMCompiler (ICML 2024), Compiled AI, grammar-constrained decoding (XGrammar, SynCode, Outlines), DSPy compile-time optimization, VerMCTS, and the Repository Intelligence Graph. The Life-Harness paper (arXiv:2605.22166) directly uses the "frozen model as reasoning kernel" framing the thesis names.

CRITICAL LIMIT: the approach ceilings out on tasks requiring cross-file consistency and long-horizon planning. Agentless patches are 97.6% single-file (among 497 successful patches, only 6 touched >1 file per arXiv:2509.13941 audit). Hard multi-file and enterprise-grade tasks still favor adaptive agentic systems. The "constraint decay" paper (arXiv:2605.06445) shows a 30-point average drop when structural constraints accumulate, and this holds regardless of whether the harness is simple (Mini-SWE-Agent) or complex (OpenHands). The thesis's disproportionate benefit to small models is theoretically sound but empirically under-measured; the SWE-Effi analysis (arXiv:2509.09853) shows the scaffold-model synergy matters more than either alone, and a small model with a strong harness is NOT universally better (Agentless + GPT-4o-mini collapses to EuTB 5.1% vs 21.8% with Qwen3-32B).


NAMED PRIOR ART

--- Agentless (arXiv:2407.01489) ---
Xia et al., Demystifying LLM-based Software Engineering Agents, 2024. Published at ACM on Software Engineering (2025).
What it does: replaces agent autonomy (tool selection, action planning, iteration) with a fixed three-phase pipeline: (1) hierarchical localization (repo -> files -> functions -> edit lines), all structured via deterministic traversal; (2) constrained patch generation via simple diff format; (3) regression-test selection + reproduction test generation. The LLM handles localization ranking, patch synthesis, and test generation within predetermined slots; it does not choose tools, manage state, or decide when to stop.
What it measured: 27.33% on SWE-bench Lite (original), rising to 40.7% / 50.8% on Lite/Verified with Claude 3.5 Sonnet. Cost: ~$0.34/issue vs much higher for agentic systems. Claimed "highest performance among open-source approaches" at release. [T1]
How close to thesis: very close. The harness decides workflow, routing, and stopping; the model generates within slots.
Stated limits: multi-file patches almost absent (6/497 successful patches cross-file per arXiv:2509.13941). Authors acknowledge hard/complex issues remain unsolved. Authors noted problems with misleading issue descriptions in the benchmark.

--- Kimi-Dev: Agentless as Skill Prior (arXiv:2509.23045) ---
MoonshotAI, 2025. Trains an agentic model on Agentless-style (single-turn verifiable steps) trajectories, then adapts to agentic SWE-agent framework.
What it does: uses the deterministic pipeline's clean reward signal (0/1 per modular step) to build a model that then performs BETTER in agentic multi-turn settings. Achieves 60.4% SWE-bench Verified (SOTA for workflow approaches at time of writing). With 5k SFT trajectories it reaches 48.6% pass@1, comparable to Claude 3.5 Sonnet.
What it measured: pass@10 = 74.0%, exceeding Agentless pass@30 = 73.8%. Self-reflection in long CoT chains transfers from deterministic to agentic. [T1]
How close to thesis: it shows the deterministic pipeline is useful as a TRAINING CORPUS, not just as an inference scaffold. Skills learned from deterministic steps generalize to longer-horizon agentic settings.
Stated limits: the agentic SWE-agent outperforms the raw Agentless pipeline once the model is trained on it, suggesting the deterministic scaffold is a ceiling on inference, not a ceiling on what the model can learn.

--- LLMCompiler (arXiv:2312.04511, ICML 2024) ---
Kim et al. An LLM Compiler for Parallel Function Calling.
What it does: the model emits a DAG of function calls and dependency edges (one-shot planning); a deterministic Task Fetching Unit + Executor then schedules and runs the calls in parallel without further LLM calls. The model's role is reduced to one planning pass; all tool execution and ordering is deterministic.
What it measured: vs ReAct baseline, 3.7x latency speedup, 6.7x cost reduction, ~9% accuracy improvement. [T1]
How close to thesis: strong match on the "model emits intent, deterministic code executes side-effects" pattern.
Stated limits: model still does planning; DAG generation can fail on complex dependency structures. No small-model ablation reported.

--- Compiled AI (arXiv:2604.05150) ---
Trooskens et al., 2026.
What it does: the LLM is invoked ONCE at compile time to generate executable code artifacts inside pre-validated templates. After compilation, workflows execute deterministically with zero further model calls. Validated via security/syntax/execution gates.
What it measured: 57x token reduction at 1,000 transactions, 450x latency improvement over direct LLM calls, 96% task completion on BFCL function-calling, 100% reproducibility vs 95% for runtime inference, 40-57x cost reduction at 1M transactions/month. 4% compilation failure rate. [T1]
How close to thesis: the most extreme instantiation of the thesis: model generates once, harness executes forever.
Stated limits: requires accurate YAML workflow specs; not suitable for tasks needing genuine creativity or novel adaptation; two task types evaluated.

--- Life-Harness (arXiv:2605.22166) ---
Adapting the Interface, Not the Model: Runtime Harness Adaptation for Deterministic LLM Agents.
What it does: keeps the model FROZEN, adapts four harness layers (environment contracts, procedural skills, action validation, trajectory monitoring) to fix recurring agent failures. Explicitly frames the LLM as "a reasoning kernel" with the harness providing deterministic environment-side logic.
What it measured: 92% of 126 model-environment configurations improved, average relative gain 88.5%, ALFWorld +84% relative. Harnesses evolved on Qwen3-4B transfer to 17 other models without retraining. [T1]
How close to thesis: the primary paper using the exact "frozen model as reasoning kernel" framing. Differs from thesis in that harness adaptation itself uses a meta-agent (not fully deterministic harness construction), but the resulting runtime harness is deterministic.
Stated limits: the harness evolution process still uses an LLM; regression precision only 11.8% (AHE companion paper, arXiv:2604.25850).

--- Agentic Harness Engineering (AHE, arXiv:2604.25850) ---
Observability-Driven Automatic Evolution of Coding-Agent Harnesses.
What it does: freezes the base model, evolves the harness automatically by reading failed trajectories. Ablation on what each harness component contributes vs the model.
What it measured (ablation): long-term memory alone +5.6pp, tools alone +3.3pp, middleware alone +2.2pp, system prompt alone -2.3pp (regression). Overall lift from 69.7% to 77.0% on Terminal-Bench 2, surpassing human-designed harness (71.9%). Gains transfer across model families (+5.1 to +10.1pp). [T1]
How close to thesis: measures harness contribution in isolation; shows harness moves are larger than model moves on the same task set.
Stated limits: non-additive interactions between components; regression prediction is the key bottleneck (fix-precision 33.7%, regression-precision 11.8%).

--- Grammar-Constrained Decoding: XGrammar, SynCode, Outlines ---
XGrammar: arXiv:2411.15100 (2024); XGrammar-2: arXiv:2601.04426 (2025). SynCode: arXiv:2403.01632 (2024).
What it does: shifts FORMAT CORRECTNESS from model to the decoder/harness. Token-level masking via pushdown automata or DFA-based lookup tables forces outputs to conform to a grammar (JSON, SQL, CFG) without relying on the model's learned format adherence.
What it measured: XGrammar achieves near-zero overhead (up to 100x speedup over prior constrained decoding). SynCode eliminates 96.07% of syntax errors in Python/Go, eliminates JSON syntax errors entirely. Grammar-Aligned Decoding (arXiv:2405.21047) preserves model probability while enforcing grammar. [T1]
How close to thesis: moves one specific "what we CAN do deterministically" (format compliance) out of the model into the harness. Directly reduces what the model must do.
Stated limits: structured decoding doesn't prevent semantic errors, only syntactic ones. Grammar must be specified in advance; open-ended code correctness (not just format) remains model-dependent.

--- DSPy (arXiv:2310.03714) ---
Khattab et al., DSPy: Compiling Declarative Language Model Calls into Self-Improving Pipelines. Stanford, 2023/ICLR 2024.
What it does: moves prompt engineering from manual runtime to a compile-time optimization phase. The optimizer (MIPROv2, COPRO) searches over instruction + demonstration space offline; inference uses the compiled fixed artifact. Separates "what to generate" from "how to instruct the model to generate it."
What it measured: GPT-3.5 outperforms standard few-shot by >25%; llama2-13b-chat outperforms by >65%. T5-770M (a small model) becomes competitive with expert-crafted GPT-3.5 prompt chains after compile-time optimization. Gains up to 5-46% over expert pipelines. [T1]
How close to thesis: moves prompt-engineering work off the model into a deterministic compilation artifact, reducing what the model must figure out at inference. The compiler is the harness.
Stated limits: optimizer itself requires a training set and many LLM calls at compile time; compiled prompts are brittle to distribution shift.

--- VerMCTS (arXiv:2402.08147) ---
What it does: replaces the model's planning/verification with deterministic Monte Carlo tree search guided by a formal verifier (Dafny/Coq). The model proposes candidate steps; the verifier determines soundness; MCTS explores the solution space. Model does generation; harness does planning and acceptance.
What it measured: substantial improvement on program synthesis benchmarks vs greedy decoding. Exact numbers not extracted (PDF). [T1]
How close to thesis: strong match on "model generates candidates, deterministic harness decides what to keep." Closest to the neurosymbolic architecture angle.
Stated limits: requires a reliable formal verifier (unavailable for most code domains); computational overhead from extensive tree exploration.

--- Repository Intelligence Graph (arXiv:2601.10112) ---
What it does: replaces LLM codebase understanding with a deterministic graph built from build artifacts (CMake File API, CTest metadata). Agents query the graph; the graph routing is deterministic.
What it measured: 12.2% mean accuracy improvement, 53.9% reduction in completion time (124.4 fewer seconds/repo), 57.8% efficiency gain. Multilingual repos: 17.7% accuracy + 69.5% efficiency. Harder questions: up to 28.6% accuracy + 79.2% efficiency. Tested on Claude Code, Cursor, Codex. [T1]
How close to thesis: direct instantiation of moving repo-understanding from model to harness; measured large gains.
Stated limits: graph is built from build artifacts (requires build system); no small-model ablation.

--- Constraint Decay: Fragility of LLM Agents in Backend Code Generation (arXiv:2605.06445) ---
What it does: measures how performance declines as structural constraints accumulate across 80 greenfield + 20 feature-implementation tasks, 8 web frameworks.
What it measured: average 30-point drop from unconstrained (L0) to fully constrained (L3). Top performers lose 40% of baseline. Database constraints alone impose -19.3pp marginal penalty. Even at L3, best agents achieve only 8.3% full compliance. Mini-SWE-Agent and OpenHands both exhibit identical decay trends despite radically different scaffold complexity (100-line vs full-featured). [T1]
Key implication: when the model must satisfy cross-file stateful constraints simultaneously, harness complexity does not prevent failure. The decay is structural, not incidental.

--- SWE-Effi (arXiv:2509.09853) ---
What it does: evaluates coding agent systems under resource constraints (token budget + time budget), comparing Agentless, AutoCodeRover, OpenHands, SWE-Agent across GPT-4o-mini, Llama, Qwen3-32B.
What it measured: SWE-Agent scaffold + Qwen3-32B = EuTB 21.8%; same scaffold + GPT-4o-mini = EuTB 5.1%. Scaffold-model synergy dominates. Agentless pipeline maintains stability but at lower peak capability. "Expensive failures" consume 4x more resources than successes. [T1]
Key finding: the thesis's core assumption (harness + small model > weak harness + large model) is PARTIALLY refuted by this paper. The right model-harness pairing matters; a small model with a strong harness is not universally better.

--- Aider Repo Map ---
What it does: deterministic tree-sitter parse + graph ranking to build a selective symbol map (the most PageRank-important identifiers across the repo). The model receives this map instead of full files; it decides which files to request. Moves file-selection and importance-ranking from model to harness.
Source: aider.chat/2023/10/22/repomap.html [T1]
What it measured: qualitative improvement only in the original blog post; no controlled ablation numbers published.
How close to thesis: matches the pattern of "move what we can do deterministically (static analysis, graph ranking) into the harness."

--- Mini-SWE-Agent (github.com/SWE-agent/mini-swe-agent) ---
100-line Python agent: zero custom tools, only bash. Design philosophy is explicitly the INVERSE of the thesis: "puts the language model rather than the agent scaffold in the middle of our attention." Achieves >74% on SWE-bench Verified.
Key counter-data: a minimal harness with a strong frontier model can match or beat elaborate harnesses. This means at the frontier (Claude 4.5, GPT-5) the harness contribution is less decisive than with smaller models. [T1]


SUPPORTING EVIDENCE (thesis works)

- Agentless achieved 27.33%->40.7% on SWE-bench with a fixed deterministic pipeline at ~$0.34/issue vs far higher agentic costs. [arXiv:2407.01489, T1]
- LLMCompiler: 3.7x latency, 6.7x cost, ~9% accuracy gain by replacing iterative LLM tool decisions with a deterministic scheduler. [arXiv:2312.04511, ICML 2024, T1]
- Compiled AI: 57x token reduction, 450x latency, 100% reproducibility at 1,000+ transactions when LLM is used once at compile time. [arXiv:2604.05150, T1]
- Life-Harness: 92% of 126 model-environment configs improved with harness adaptation on frozen model, avg 88.5% relative gain, transfers to 17 models. [arXiv:2605.22166, T1]
- AHE ablation: harness components (memory +5.6pp, tools +3.3pp, middleware +2.2pp) each contribute more than model-level changes on Terminal-Bench 2. System prompt alone REGRESSES (-2.3pp). [arXiv:2604.25850, T1]
- Repository Intelligence Graph: 12.2% accuracy improvement + 53.9% time reduction by making repo-understanding deterministic. [arXiv:2601.10112, T1]
- SynCode: 96.07% reduction in Python/Go syntax errors via grammar-constrained decoding (no model change). [arXiv:2403.01632, T1]
- DSPy: T5-770M becomes competitive with expert-crafted GPT-3.5 chains after compile-time optimization; llama2-13b-chat gains >65% over standard few-shot. [arXiv:2310.03714, T1]
- CCA (Confucius Code Agent): context management component alone lifted Claude 4 Sonnet from 42.0% to 48.6% (+6.6pp) on a 100-example subset; note-taking reduced tokens 104k->93k while improving resolve rate. [arXiv:2512.10398, T1]
- Kimi-Dev: training a model on Agentless-style (deterministic, verifiable) trajectories produces skills that TRANSFER to agentic settings, reaching 60.4% on SWE-bench Verified. [arXiv:2509.23045, T1]
- Red Planet Labs production anecdote: external orchestration with fresh context per phase prevented model "tunneling" (rationalization collapse). Embedding instructions in files rather than context resists compaction. [blog.redplanetlabs.com, T2]
- LangChain moved coding agent from 30th to 5th on Terminal-Bench 2.0 without changing the model, via harness optimization alone. [faros.ai/blog, T2]
- Confucius Code Agent: "Agent scaffold—not only backbone model capability—is a decisive factor in real-world software engineering tasks." Scaffold changes produce larger deltas than frontier model swaps. [arXiv:2512.10398, T1]


REFUTING / LIMITING EVIDENCE (where it breaks or ceilings)

- Constraint decay: when structural requirements accumulate (ORM, auth, multi-framework), average 30-point drop regardless of harness complexity. Mini-SWE-Agent (100-line) and OpenHands (full-featured) decay at identical rates. Only 8.3% full compliance at L3 constraints. This is structural, not incidental. [arXiv:2605.06445, T1]
- Multi-file coordination wall: Agentless produced only 6 multi-file patches among 497 successes. On multi-file issues, OpenHands = 25.3% success vs Agentless = 18.3%. Agentic systems maintain an edge on cross-file work. [arXiv:2509.13941, T1]
- Mini-SWE-Agent counter-thesis: a 100-line harness with frontier model achieves >74% on SWE-bench Verified, beating many elaborate scaffolds. With strong models, harness complexity becomes less important. The thesis's benefit diminishes with model capability. [github.com/SWE-agent/mini-swe-agent, T1]
- SWE-Effi scaffold-model synergy: small model (GPT-4o-mini) + strong scaffold collapses to EuTB 5.1% vs 21.8% with Qwen3-32B on the same scaffold. A harness is not a substitute for model capability; the right pairing matters. [arXiv:2509.09853, T1]
- SWE-bench leaderboard dissection: no single architecture dominates; both agentic and pipeline approaches achieve competitive results, with "performance depending more on LLM choice and specific implementation strategies than architectural classification alone." [arXiv:2506.17208, T1]
- Hard tasks convergence: on tasks requiring >=1h human effort, all tools (agentless and agentic) converge to 13-20% success. Deterministic harness does not unlock hard tasks. [arXiv:2509.13941 and related, T1]
- Formal verification bottleneck (VerMCTS): the "model proposes, harness verifies" pattern requires a reliable formal verifier, which is unavailable for most real-world code domains. [arXiv:2402.08147, T1]
- DSPy brittleness: compile-time optimized prompts are brittle to distribution shift; require a training set and many LLM calls at compile time. The compilation is not free. [arXiv:2310.03714 + T2 commentary, T1/T2]
- NLAH ablation: the verifier harness component REGRESSED performance (-0.8% SWE-bench, -8.4% OSWorld). Not all deterministic harness additions help. [arXiv:2603.25723, T1]
- AHE caveat: regression prediction by the harness evolution system is weak (11.8% precision vs 33.7% fix-precision). The harness can make things worse on regressions. [arXiv:2604.25850, T1]
- Agentless on SWE-bench Multimodal: agentic systems score 11.5% vs Agentless 3.9% on multimodal tasks requiring interactive visual understanding. Fixed pipeline fails on task modalities it wasn't designed for. [T2 SWE-bench Multimodal paper]


THE GAP / WHITESPACE

What appears NOT to have been done in a way that directly tests the thesis:

1. Controlled ablation: small fixed local model + maximal deterministic harness vs same model + minimal harness, at production scale on hard tasks. Most ablations either fix the model and vary the harness (showing harness contribution) or fix the harness and vary the model (showing model contribution). Nobody has published a controlled study specifically testing "how much can you shrink the model if you maximize the harness" across a range of model sizes.

2. Tool-selection-free design: the thesis specifically proposes removing TOOL SELECTION from the model (the model never decides which tool to call; the harness routes deterministically based on task classification). Mini-SWE-Agent removes tools almost entirely (only bash), but that's not the same as deterministic routing; the model still decides what bash commands to run. Agentless hardcodes the workflow but the model still does more than pure code generation.

3. Harness as the instruction-follower: moving reading of rules/docs/constraints into the harness (not the model's context) and having the model receive only a constrained generation slot. Grammar-constrained decoding does this for FORMAT but not for SEMANTIC RULES ("don't use X pattern in this codebase"). Nobody has built a harness where semantic codebase rules are enforced mechanically rather than prompted.

4. Disproportionate small-model benefit measurement: the thesis's specific prediction is that a small model fails mostly on tool-use and long-horizon instruction-following, not raw constrained code-gen. If you eliminate those via harness, small models should disproportionately improve. This specific prediction has not been cleanly measured. DSPy shows small model gains, but at the prompt-optimization level, not the tool-routing level. Life-Harness transfers harness gains across 17 models but doesn't specifically test small vs large model relative gains from harness.

5. Combined approach: Agentless + grammar-constrained decoding + deterministic file routing (RIG-style) + DSPy compile-time optimization in one integrated system. These tools exist separately; nobody has combined them into a "maximal deterministic kernel" and measured the combined effect on small models. This combination is what the thesis specifically proposes and it appears genuinely unstudied.


CITATIONS

[1] Agentless: arXiv:2407.01489; also dl.acm.org/doi/abs/10.1145/3715754 [T1]
[2] LLMCompiler (ICML 2024): arXiv:2312.04511; github.com/SqueezeAILab/LLMCompiler [T1]
[3] Compiled AI: arxiv.org/html/2604.05150 [T1]
[4] Life-Harness (Runtime Harness Adaptation): arxiv.org/html/2605.22166v1 [T1]
[5] Agentic Harness Engineering (AHE): arxiv.org/html/2604.25850v3 [T1]
[6] XGrammar: arxiv.org/abs/2411.15100 [T1]; XGrammar-2: arxiv.org/abs/2601.04426 [T1]
[7] SynCode: arxiv.org/pdf/2403.01632 [T1]
[8] Grammar-Aligned Decoding: arxiv.org/abs/2405.21047 [T1]
[9] DSPy: arxiv.org/abs/2310.03714 [T1]; github.com/stanfordnlp/dspy [T1]
[10] VerMCTS: arxiv.org/pdf/2402.08147 [T1]
[11] Repository Intelligence Graph: arxiv.org/pdf/2601.10112 [T1]
[12] Constraint Decay: arxiv.org/abs/2605.06445 [T1]
[13] SWE-Effi: arxiv.org/abs/2509.09853 [T1]
[14] Kimi-Dev: arxiv.org/abs/2509.23045 [T1]; github.com/MoonshotAI/Kimi-Dev [T1]
[15] Confucius Code Agent: arxiv.org/html/2512.10398v3 [T1]
[16] Natural-Language Agent Harnesses: arxiv.org/html/2603.25723v1 [T1]
[17] Failures in Automated Issue Solving (multi-file finding): arxiv.org/pdf/2509.13941 [T1]
[18] SWE-bench Leaderboard Dissection: arxiv.org/html/2506.17208v2 [T1]
[19] Mini-SWE-Agent: github.com/SWE-agent/mini-swe-agent [T1]
[20] Aider Repo Map: aider.chat/2023/10/22/repomap.html [T1]
[21] AgentDiet (trajectory reduction): arxiv.org/html/2509.23586v2 [T1]
[22] SWE-Bench Pro: arxiv.org/abs/2509.16941 [T1]
[23] Red Planet Labs backend teaching report: blog.redplanetlabs.com/2026/05/28/teaching-llms-to-one-shot-complex-backends-at-scale-report-1/ [T2]
[24] Harness Engineering (Faros): faros.ai/blog/harness-engineering [T2]
[25] MindStudio agent harness primer: mindstudio.ai/blog/agent-harness-scaffolding-matters-more-than-model [T2]
[26] Outlines (structured generation): github.com/outlines-dev/outlines [T1] (not directly fetched; existence verified via cross-reference in XGrammar and Awesome-LLM-Constrained-Decoding)
[27] Awesome-LLM-Constrained-Decoding survey: github.com/Saibo-creator/Awesome-LLM-Constrained-Decoding [T2]


OPEN QUESTIONS / STALE-RISK

- The SWE-Effi paper (arXiv:2509.09853) PDF yielded limited specific numbers in the fetch; the EuTB 5.1% vs 21.8% figure came from the search snippet, not the full paper. Treat as directionally correct but verify exact methodology if used for quantitative claims.
- Mini-SWE-Agent's >74% SWE-bench Verified score uses frontier models (Claude/GPT-5 class). The claim about small models with minimal harness is extrapolated; that experiment has not been run.
- The "96.07% reduction in syntax errors" claim from SynCode (arXiv:2403.01632) is from the paper's abstract and not independently verified in our fetch.
- Agentless's 40.7% / 50.8% numbers with Claude 3.5 Sonnet are from the GitHub README (Dec 2024 update), not peer-reviewed. The 27.33% figure is the peer-reviewed number.
- LangChain "30th to 5th on Terminal-Bench 2.0" claim is from a practitioner blog (faros.ai); could not verify the underlying benchmark submission.
- Outlines framework: existence verified via cross-references in multiple papers but not directly fetched (outlines-dev/outlines GitHub). Treat as [T1] on existence, [T2] on specific claims.
- The "harness engineering" term attribution to Mitchell Hashimoto (early Feb 2026) is from a practitioner blog; the original Hashimoto post was not directly fetched.
- SWE-bench benchmark saturation is reported as a concern in multiple sources but no specific paper quantifying the ceiling effect was fetched directly.
- VerMCTS exact benchmark numbers were not successfully extracted from the PDF; only qualitative findings confirmed.
