HARNESS-AS-KERNEL THESIS + COMPETITIVE LANDSCAPE
sweep 4 of the direction sweep, 2026-07-17
scope: the portable-kernel thesis (does it hold), its prior art, its limits, and who else is doing it
method: full read of the 11-doc Layer-1 bundle + ELAI-SPEC-OVERVIEW + grep-trace into the B..Q plan corpus
every claim below carries its source path. figures are copied, never rounded.

READER'S NOTE: this file is written to REPLACE reading the bundle. The corpus spans 2026-06-04 to
2026-07-11 and revises itself three times. Where a later doc overturns an earlier one, both are
present with dates. Section 3 contains three quote-verifications the owner asked for; two of the
three found errors, and one of those errors is in the source doc, not the owner's memory of it.


====================================================================
0. THE SHORT ANSWER
====================================================================

The portable-kernel thesis SURVIVES its own evidence, but not in the form it was posed.

What survives: "the deterministic harness does everything except code generation" is a REAL,
named, empirically-validated discipline with ~10 instantiations and measured payoffs
(workflow/research/harness-as-pipe-prior-art-2026-06-14.md:7-9). The harness genuinely moves the
number more than the model does on a fixed task set (AHE ablation, same doc :51-56).

What does NOT survive: "the model as a frozen reasoning kernel, portable to any harness, benefiting
small models disproportionately." Three limits break that specific form, and the third is fatal to
the portability claim:
  1. the cross-file / long-horizon ceiling (Agentless is ~98.8% single-file);
  2. constraint decay (~30-point drop, INDEPENDENT of harness complexity);
  3. scaffold-model SYNERGY (SWE-Effi) — the kernel is not separable from the model it runs.

Limit 3 is the hardest one standing. If harness value were portable, a strong scaffold would lift
any model. It does not: the same scaffold scores EuTB 21.8% with Qwen3-32B and 5.1% with
GPT-4o-mini (harness-as-pipe :91). A 4x swing from swapping the model UNDER a fixed harness means
harness value is CONDITIONAL on model capability, not additive to it. "Frozen kernel + max harness"
is therefore not a free lunch; it is a pairing problem.

The corpus's own July revision reaches the same place from the market side and states it plainly:
"The harness can expose, verify, route, and compound a model's capability; it cannot eliminate the
model's ceiling... ELAI's defensible thesis is not 'the model does not matter.' It is: The harness
determines how much model capability can be trusted, reviewed, recovered, and compounded per
dollar." (workflow/reasoning/elai-harness-market-evaluation-2026-07-10.md:278-284)

That is the surviving thesis. It is narrower than "frozen kernel" and it is defensible.

ONE STRUCTURAL FINDING WORTH THE WHOLE SWEEP: the prior-art survey (2026-06-14) is cited by ZERO
plan files (section 7). Its constituent papers were absorbed EARLIER and INDEPENDENTLY (E-21, E-46,
J-12, I-36). The survey is research with no plan behind it. But the absorption that did happen is
better than the survey: E-46 encoded Agentless's LIMIT as a routing predicate, not just its
technique — the shortcut fires only on single-file work (section 7.2, verified in live code). ELAI
already does the right thing with the thesis; it just did it before the survey existed and never
closed the loop.


====================================================================
1. THE THESIS, PRECISELY STATED
====================================================================

As posed in the survey's own question line:

  "Has anyone deliberately built a coding harness that minimizes the LLM's role to pure code
  generation, pushing everything else into deterministic non-LLM harness code? Is this named,
  validated, or a known dead-end?"
  (workflow/research/harness-as-pipe-prior-art-2026-06-14.md:3)

Search effort: 14 searches, 18 fetches (:4).

The thesis has FOUR distinguishable claims. They do not stand or fall together, and the survey's
verdict does not separate them. This separation is mine and is the main analytical move of this
file:

  T1 (FEASIBILITY) — you can push workflow, routing, and stopping out of the model into
     deterministic code.
     VERDICT: PROVEN. Multiple shipped instantiations. Strongest evidence.

  T2 (PAYOFF) — doing so measurably improves cost/latency/reproducibility.
     VERDICT: PROVEN, with large effect sizes (Compiled AI 57x tokens; LLMCompiler 6.7x cost).

  T3 (PORTABILITY) — the harness is a KERNEL: separable, transferable across models.
     VERDICT: CONTESTED. Life-Harness supports it (transfers to 17 models). SWE-Effi refutes the
     strong form (4x swing from model swap under a fixed scaffold). Mini-SWE-Agent inverts it
     (harness value SHRINKS as the model strengthens). This is where the thesis is in real trouble.

  T4 (SMALL-MODEL DIVIDEND) — small models benefit DISPROPORTIONATELY, because they fail on
     tool-use and long-horizon instruction-following, not on constrained code-gen.
     VERDICT: UNMEASURED, and partially refuted. The survey names this its own gap #4
     (:147: "This specific prediction has not been cleanly measured"). SWE-Effi cuts against it.
     ELAI's own small-model research (section 5) SUPPORTS the mechanism but reframes the fix.

The survey's headline verdict (:7-9) answers T1 and T2 and is correct on both. It does not
cleanly separate T3/T4, and its CRITICAL LIMIT paragraph (:11) is where the T3/T4 damage is
recorded — including, as section 3 shows, one misattribution.

THE "FROZEN KERNEL" FRAMING'S ACTUAL SOURCE. The exact phrase the owner is working from is
Life-Harness's, not ELAI's: "Explicitly frames the LLM as 'a reasoning kernel' with the harness
providing deterministic environment-side logic" (:46). Its title is the thesis in six words:
"Adapting the Interface, Not the Model" (:45). ELAI's spec independently uses the same shape:
"the model is treated as a replaceable cog" (docs/thefinalglance/ELAI-SPEC-OVERVIEW.md:16). These
converged; neither borrowed from the other.


====================================================================
2. NAMED PRIOR ART — FULL INVENTORY
====================================================================

All from workflow/research/harness-as-pipe-prior-art-2026-06-14.md unless noted. Tier tags [T1]/[T2]
are the survey's own source-reliability marks. "Closeness" is the survey's assessment of distance
to the thesis. I add a T1-T4 mapping column showing WHICH claim each piece actually supports —
this is the analytical layer the survey lacks.

--- 2.1 AGENTLESS — arXiv:2407.01489 — THE CLOSEST SINGLE PIECE OF PRIOR ART
Xia et al., "Demystifying LLM-based Software Engineering Agents", 2024, UIUC. Published ACM on
Software Engineering (2025). Source lines :16-21.

WHAT IT DOES: replaces agent autonomy (tool selection, action planning, iteration) with a fixed
three-phase pipeline:
  (1) hierarchical localization: repo -> files -> functions -> edit lines, deterministic traversal
  (2) constrained patch generation via simple diff format
  (3) regression-test selection + reproduction test generation
"The LLM handles localization ranking, patch synthesis, and test generation within predetermined
slots; it does not choose tools, manage state, or decide when to stop." (:18)

MEASURED (:19):
  27.33% SWE-bench Lite (original, peer-reviewed figure)
  40.7% / 50.8% Lite/Verified with Claude 3.5 Sonnet
  ~$0.34/issue vs much higher for agentic systems
  claimed "highest performance among open-source approaches" at release

CLOSENESS: "very close. The harness decides workflow, routing, and stopping; the model generates
within slots." (:20)
MAPS TO: T1 (proves it), T2 (proves it). Silent on T3/T4.

STATED LIMITS (:21): multi-file patches almost absent (6/497 cross-file per arXiv:2509.13941);
authors acknowledge hard/complex issues unsolved; problems with misleading issue descriptions in
the benchmark.

*** NUMBER CONFLICT INSIDE OUR OWN CORPUS — UNRESOLVED, FLAGGING IT ***
The survey says 27.33% Lite at ~$0.34/issue (:19), and its own caveat says "The 27.33% figure is
the peer-reviewed number" (:188).
But TWO plan files say 32.00% at $0.70/issue, citing the same arXiv ID:
  docs/thefinalglance/E-plan/E-46_planning_decomposition.md:195 — "arXiv:2407.01489 AGENTLESS —
    32.00% SWE-bench Lite at $0.70/issue"; the file references "arXiv:2407.01489 v2" (:39)
  docs/thefinalglance/E-plan/E-21_peer_systems_llm_as_worker.md:10,154 — "Agentless 32% SWE-bench
    at $0.70/issue"
RECONCILIATION (mine, arithmetic, HIGH confidence but UNVERIFIED against the paper): our ingested
interface is v2 (workflow/papers/agentless-2407.01489-interface.md:4 — "arXiv:2407.01489v2
29 Oct 2024") and records SWE-bench Lite scores as COUNTS: "inflates performance to 98 vs. 96 on
SWE-bench Lite" (:163). SWE-bench Lite is 300 instances. 96/300 = 32.00%; 98/300 = 32.67%. And
82/300 = 27.33%. So 27.33% is v1 and 32.00% is v2 — both real, different paper versions. The cost
delta ($0.34 vs $0.70) is unexplained by this and remains open.
ACTION: do not cite an Agentless number without naming the paper version. The survey's claim that
27.33% is "the peer-reviewed number" is likely STALE — v2 is the current arXiv version and is what
we ingested.

--- 2.2 LIFE-HARNESS — arXiv:2605.22166 — THE "FROZEN KERNEL" PAPER
"Adapting the Interface, Not the Model: Runtime Harness Adaptation for Deterministic LLM Agents."
Source lines :44-49. Repo github.com/Tianshi-Xu/Life-Harness (MIT), clone present at
docs/.source/life-harness (VERIFIED present this session).

WHAT IT DOES: keeps the model FROZEN, adapts four harness layers (environment contracts, procedural
skills, action validation, trajectory monitoring) to fix recurring agent failures (:46).

MEASURED (:47):
  92% of 126 model-environment configurations improved
  average relative gain 88.5%
  ALFWorld +84% relative
  harnesses evolved on Qwen3-4B TRANSFER to 17 other models without retraining

CLOSENESS: "the primary paper using the exact 'frozen model as reasoning kernel' framing" (:48).
MAPS TO: T1, T2, and — uniquely — T3. The 17-model transfer is the ONLY direct positive evidence
for portability in the whole survey. This single result carries the kernel thesis.

STATED LIMITS (:49): "the harness evolution process still uses an LLM; regression precision only
11.8% (AHE companion paper, arXiv:2604.25850)."

*** OUR SOURCE READ CORRECTS THE SURVEY ON THE LLM-IN-THE-LOOP POINT ***
The survey's caveat at :48 ("harness adaptation itself uses a meta-agent (not fully deterministic
harness construction)") is TRUE ONLY OF OFFLINE AUTHORING. Our own reviewed-source ingest is
explicit and contradicts the looser reading:
  "No Codex/LLM intervention-proposal pipeline found in this codebase. All interventions are
  deterministic: H2 ... pure Python DB-state inspection (no LLM); H3: static string append at init
  (no LLM); H4 ... pure Python DB-state inspection (no LLM); H5: BM25 retrieval over a MANUALLY
  authored skill library (no LLM at runtime)."
  (workflow/papers/life-harness-source-interface.md:285-293)
  "The paper-ingestor previously produced life-harness-paper-interface.md which references
  'Codex/LLM intervention-proposal pipeline' — this refers to the OFFLINE skill authoring workflow
  ..., NOT an online LLM-in-the-loop component. The harness itself is LLM-free at runtime."
  (same, :295-299)
This MATTERS for the kernel thesis: Life-Harness's runtime is a pure-Python, stdlib-only, LLM-free
harness (BM25 hand-rolled from math+re+collections, :229-231; "No GPU/ML dependencies in harness
code; all harness logic is CPU/stdlib", :499). That is the thesis in its strongest possible form,
and it is the ONE piece of prior art we have read at source rather than trusting the abstract.
Reliability of that read: MEDIUM (reviewed source, shallow clone) (:2). RAG ingest FAILED
(reranker crash) (:3).

THE FOUR LAYERS, verbatim from source (:27-54) — this is the concrete kernel shape:
  H2 Action Gate — after agent proposes tool call, BEFORE execution. HarnessRule.check(db,**kwargs)
     raises ValueError on violation. Agent sees ToolMessage(error=True). Also text-rescue: parse
     JSON/kwarg/XML from malformed free-text agent output.
  H3 Tool-Description Embedding — at episode init, once. _append_hint_to_tool() wraps the tool's
     __doc__ with policy constraints. KEY INSIGHT (:44-46): "tool descriptions have constant
     attention weight across all turns; system prompt salience degrades as context grows."
  H4 Post-Execution Monitor — after tool executes, before next agent turn. Returns str|None
     appended to the tool result. Also stall detection, loop detection, budget warn/force.
  H5 Goal-Directed Guidance — episode start. BM25 skill retrieval over task description; top-k
     skills prepended to system prompt as "Learned Tips" at the HIGHEST attention position.
REPLAY-SAFETY CONSTRAINT (:304-308, :507-509) — the detail that makes it a kernel: "implementations
MUST only read from DB/kwargs; must NOT have side-effects or maintain cross-call state." Read-only
rules do not interfere with set_state replay. This is exactly ELAI's off-agent-evidence discipline,
arrived at independently.

--- 2.3 AGENTIC HARNESS ENGINEERING (AHE) — arXiv:2604.25850
"Observability-Driven Automatic Evolution of Coding-Agent Harnesses." Source :51-56.
Freezes the base model, evolves the harness by reading failed trajectories.

MEASURED — THE ABLATION (:54). This is the most decision-relevant table in the survey:
  long-term memory alone   +5.6pp
  tools alone              +3.3pp
  middleware alone         +2.2pp
  system prompt alone      -2.3pp   <-- REGRESSION
  overall 69.7% -> 77.0% on Terminal-Bench 2, surpassing human-designed harness (71.9%)
  gains transfer across model families (+5.1 to +10.1pp)

CLOSENESS: "measures harness contribution in isolation; shows harness moves are larger than model
moves on the same task set" (:55).
MAPS TO: T2 (strongly), T3 (the cross-family transfer, +5.1 to +10.1pp, is the SECOND piece of
portability evidence after Life-Harness).
STATED LIMITS (:56): non-additive interactions between components; fix-precision 33.7%,
regression-precision 11.8% — "regression prediction is the key bottleneck."

NOTE THE SYSTEM-PROMPT RESULT. -2.3pp means the single most common "harness improvement" move
(write a better system prompt) made things WORSE in isolation. Every deterministic component beat
it. That is the thesis's cleanest single data point and it is buried in an ablation line.

--- 2.4 COMPILED AI — arXiv:2604.05150
Trooskens et al., 2026. Source :37-42.
LLM invoked ONCE at compile time to generate executable code artifacts inside pre-validated
templates. After compilation, workflows execute deterministically with ZERO further model calls.

MEASURED (:40):
  57x token reduction at 1,000 transactions
  450x latency improvement over direct LLM calls
  96% task completion on BFCL function-calling
  100% reproducibility vs 95% for runtime inference
  40-57x cost reduction at 1M transactions/month
  4% compilation failure rate

CLOSENESS: "the most extreme instantiation of the thesis: model generates once, harness executes
forever" (:41).
MAPS TO: T1, T2 (largest effect sizes in the survey).
STATED LIMITS (:42): requires accurate YAML workflow specs; "not suitable for tasks needing genuine
creativity or novel adaptation"; only two task types evaluated.
CAVEAT: the 2-task-type evaluation is thin for the size of those numbers. Treat 57x/450x as
existence proofs on narrow, repetitive workloads, not as a general expectation.
ELAI LINK: cited in docs/thefinalglance/I-plan/I-36_cross_model_prompt_compilation.md:105 with the
note "WebSearch-confirm before headline" — correctly gated.

--- 2.5 LLMCOMPILER — arXiv:2312.04511 (ICML 2024)
Kim et al., "An LLM Compiler for Parallel Function Calling." Source :30-35.
Model emits a DAG of function calls + dependency edges (one-shot planning); a deterministic Task
Fetching Unit + Executor schedules and runs calls in parallel with no further LLM calls.

MEASURED vs ReAct baseline (:33): 3.7x latency speedup, 6.7x cost reduction, ~9% accuracy improvement.
CLOSENESS: "strong match on the 'model emits intent, deterministic code executes side-effects'
pattern" (:34).
MAPS TO: T1, T2.
STATED LIMITS (:35): model still does planning; DAG generation can fail on complex dependency
structures; NO SMALL-MODEL ABLATION REPORTED.

--- 2.6 GRAMMAR-CONSTRAINED DECODING — XGrammar / SynCode / Outlines
XGrammar arXiv:2411.15100 (2024); XGrammar-2 arXiv:2601.04426 (2025); SynCode arXiv:2403.01632
(2024); Grammar-Aligned Decoding arXiv:2405.21047. Source :58-63.
Shifts FORMAT CORRECTNESS from model to decoder/harness via token-level masking (pushdown automata
/ DFA lookup tables).

MEASURED (:61):
  XGrammar: near-zero overhead, up to 100x speedup over prior constrained decoding
  SynCode: eliminates 96.07% of syntax errors in Python/Go; eliminates JSON syntax errors entirely
  Grammar-Aligned Decoding: preserves model probability while enforcing grammar

CLOSENESS: "moves one specific 'what we CAN do deterministically' (format compliance) out of the
model into the harness" (:62).
MAPS TO: T1 (a clean, narrow, total win). Arguably the ONLY piece of prior art where the
deterministic substitution is complete and lossless.
STATED LIMITS (:63): "structured decoding doesn't prevent semantic errors, only syntactic ones.
Grammar must be specified in advance; open-ended code correctness remains model-dependent."
CAVEAT (:187): the 96.07% SynCode figure is from the abstract, "not independently verified in our
fetch."
OUTLINES CAVEAT (:190): existence verified via cross-reference only, not directly fetched.
[T1] on existence, [T2] on specific claims.
ELAI LINK — THE DEEPEST ABSORPTION OF ANY PRIOR ART: XGrammar appears in 8+ plan units. N-plan
flags this as FRAGMENTATION: "machinery exists in >=8 places (C-26 DecodingMode / E-35
XGrammar+INSTRUCTOR / I-34 CRANE+grammar-pin+post-parse-...)"
(docs/thefinalglance/N-plan/N-SUMMARY_INDEX.md:266). E-35 is the substrate
(docs/thefinalglance/E-plan/E-SUMMARY_INDEX.md:306); E-18 wires the call sites (:170). The local
lane is PREVENT-not-repair: "XGrammar masks the Qwen sidecar so a malformed tool-call shape
[cannot form]" (docs/thefinalglance/J-plan/J-SUMMARY_INDEX.md:1410).

--- 2.7 DSPy — arXiv:2310.03714
Khattab et al., Stanford, 2023 / ICLR 2024. Source :65-70.
Moves prompt engineering from manual runtime to a compile-time optimization phase (MIPROv2, COPRO
search over instruction + demonstration space offline); inference uses the compiled fixed artifact.

MEASURED (:68):
  GPT-3.5 outperforms standard few-shot by >25%
  llama2-13b-chat outperforms by >65%
  T5-770M (a SMALL model) becomes competitive with expert-crafted GPT-3.5 prompt chains after
    compile-time optimization
  gains up to 5-46% over expert pipelines

CLOSENESS: "moves prompt-engineering work off the model into a deterministic compilation artifact...
The compiler is the harness" (:69).
MAPS TO: T1, T2, and T4 — DSPy is the STRONGEST small-model-dividend evidence in the survey
(T5-770M reaching expert-GPT-3.5-chain parity). But the survey correctly discounts it for T4:
"DSPy shows small model gains, but at the prompt-optimization level, not the tool-routing level"
(:147). The dividend is demonstrated in the WRONG LAYER for the thesis.
STATED LIMITS (:70): "optimizer itself requires a training set and many LLM calls at compile time;
compiled prompts are brittle to distribution shift." Restated at :131: "The compilation is not free."

--- 2.8 REPOSITORY INTELLIGENCE GRAPH — arXiv:2601.10112
Source :78-82. Replaces LLM codebase understanding with a deterministic graph built from BUILD
ARTIFACTS (CMake File API, CTest metadata). Agents query the graph; routing is deterministic.

MEASURED (:80):
  12.2% mean accuracy improvement
  53.9% reduction in completion time (124.4 fewer seconds/repo)
  57.8% efficiency gain
  multilingual repos: 17.7% accuracy + 69.5% efficiency
  harder questions: up to 28.6% accuracy + 79.2% efficiency
  tested on Claude Code, Cursor, Codex

CLOSENESS: "direct instantiation of moving repo-understanding from model to harness; measured large
gains" (:81).
MAPS TO: T1, T2. Notable: gains RISE with question difficulty (28.6% on hard vs 12.2% mean) — the
opposite of the mini-SWE-agent pattern where harness value falls as the task gets easier for the
model.
STATED LIMITS (:82): graph built from build artifacts (requires a build system); no small-model
ablation.
ELAI RELEVANCE: this is ELAI's Phase-1 knowledge engine thesis with an external number attached
(DKG/BM25/vector/PageRank, docs/thefinalglance/ELAI-SPEC-OVERVIEW.md:45-62). The survey does not
make this connection. It is the closest external validation of ELAI's largest built subsystem, and
nothing in the plan corpus cites it.

--- 2.9 KIMI-DEV — arXiv:2509.23045
MoonshotAI, 2025. Source :23-28. Trains an agentic model on Agentless-style (single-turn verifiable
steps) trajectories, then adapts to the agentic SWE-agent framework.

MEASURED (:25-26):
  60.4% SWE-bench Verified (SOTA for workflow approaches at time of writing)
  5k SFT trajectories -> 48.6% pass@1, comparable to Claude 3.5 Sonnet
  pass@10 = 74.0%, exceeding Agentless pass@30 = 73.8%

CLOSENESS: "it shows the deterministic pipeline is useful as a TRAINING CORPUS, not just as an
inference scaffold. Skills learned from deterministic steps generalize to longer-horizon agentic
settings." (:27)
STATED LIMIT — AND IT IS A DEEP ONE (:28): "the agentic SWE-agent outperforms the raw Agentless
pipeline once the model is trained on it, suggesting the deterministic scaffold is a ceiling on
inference, not a ceiling on what the model can learn."
MAPS TO: T1 (as corpus), but it ATTACKS T3. If the deterministic pipeline's best use is generating
training data that then makes an AGENTIC harness win, the kernel is a bootstrapping stage, not a
permanent architecture. This is the most intellectually threatening piece of prior art in the
survey and it is filed under "supporting evidence" (:116). I would file it under limits.

--- 2.10 VerMCTS — arXiv:2402.08147
Source :72-76. Replaces model planning/verification with deterministic MCTS guided by a formal
verifier (Dafny/Coq). Model proposes candidates; verifier determines soundness.
MEASURED (:74): "substantial improvement on program synthesis benchmarks vs greedy decoding. Exact
numbers not extracted (PDF)." Re-flagged at :193 — only qualitative findings confirmed.
CLOSENESS: "strong match on 'model generates candidates, deterministic harness decides what to
keep.' Closest to the neurosymbolic architecture angle." (:75)
STATED LIMITS (:76, :130): "requires a reliable formal verifier, which is unavailable for most
real-world code domains"; computational overhead from tree exploration.
ASSESSMENT: the weakest-evidenced entry (no numbers) and the least generalizable (verifier
requirement). Do not lean on it.

--- 2.11 AIDER REPO MAP
Source :94-98. aider.chat/2023/10/22/repomap.html [T1]. Deterministic tree-sitter parse + graph
ranking (PageRank) to build a selective symbol map; model receives the map instead of full files.
MEASURED (:97): "qualitative improvement only in the original blog post; NO CONTROLLED ABLATION
NUMBERS PUBLISHED."
CLOSENESS: "matches the pattern of 'move what we can do deterministically (static analysis, graph
ranking) into the harness'" (:98).
ELAI LINK: ELAI's PageRank repo-map derives directly from this
(workflow/reasoning/ELAI-COMPETITIVE-ASSESSMENT-GROUNDED-2026-06-04.md:120 — "a tree-sitter +
PageRank repo-map (the literal source ELAI's PageRank derives from)").

--- 2.12 MINI-SWE-AGENT — THE COUNTER-THESIS
github.com/SWE-agent/mini-swe-agent. Source :100-102.
100-line Python agent: zero custom tools, only bash. Design philosophy EXPLICITLY the inverse of the
thesis: "puts the language model rather than the agent scaffold in the middle of our attention."
MEASURED: >74% on SWE-bench Verified (:101).
COUNTER-DATA (:102, :126): "a minimal harness with a strong frontier model can match or beat
elaborate harnesses. This means at the frontier (Claude 4.5, GPT-5) the harness contribution is
less decisive than with smaller models. The thesis's benefit diminishes with model capability."
CAVEAT (:186): ">74% ... uses frontier models (Claude/GPT-5 class). The claim about small models
with minimal harness is EXTRAPOLATED; that experiment has not been run."
MAPS TO: direct attack on T3/T4. And note the caveat cuts BOTH ways — the un-run experiment is
exactly the one that would settle T4.
2026 UPDATE (workflow/reasoning/elai-harness-market-evaluation-2026-07-10.md:685): "SWE-bench now
holds the mini-SWE-agent harness fixed when comparing models." The counter-thesis harness became
the FIELD'S CONTROL. That is a significant institutional endorsement of minimalism and the survey
predates it.


====================================================================
3. THE LIMITS AT FULL STRENGTH + THE THREE QUOTE-CHECKS
====================================================================

The owner asked me to verify three quoted limits against the doc text and correct any misquote.
Result: quote 2 is exact. Quotes 1 and 3 both have problems, and BOTH problems originate in the
SOURCE DOC, not in the owner's recollection. The owner quoted our own doc faithfully; our doc is
internally inconsistent in two places.

--- 3.1 QUOTE 1: THE CROSS-FILE CEILING — SUBSTANTIALLY CORRECT, ARITHMETIC DOES NOT CLOSE

OWNER'S QUOTE: "Agentless patches 97.6% single-file; 6 of 497 touched >1 file, arXiv:2509.13941"

DOC TEXT (harness-as-pipe-prior-art-2026-06-14.md:11): "Agentless patches are 97.6% single-file
(among 497 successful patches, only 6 touched >1 file per arXiv:2509.13941 audit)."

VERDICT: the owner quoted the doc VERBATIM AND CORRECTLY. But the doc's two halves disagree with
each other:
  6 of 497 cross-file  =>  1.21% cross-file  =>  98.79% single-file, NOT 97.6%.
For 97.6% single-file you would need ~12 of 497 cross-file, twice the stated count.
The doc's SECOND statement of the same finding (:125) drops the percentage entirely and keeps only
the count: "Agentless produced only 6 multi-file patches among 497 successes."
LIKELY EXPLANATION (UNVERIFIED): 97.6% and 6/497 come from different measurements in
arXiv:2509.13941 (e.g. a percentage over ALL patches vs a count over SUCCESSFUL patches) and were
merged into one sentence. The survey never fetched this cleanly — 2509.13941 does not appear in its
fetch-caveat list, but it also is not among its named entries; it appears only as a citation
(:170 "Failures in Automated Issue Solving (multi-file finding)").
ACTION: cite "6 of 497 successful patches touched more than one file (arXiv:2509.13941)" — the
count is stated twice and consistently. DROP the 97.6% figure until the paper is re-fetched. Either
way the LIMIT IS UNAFFECTED and is arguably WORSE than quoted: 98.8% single-file is a harder
ceiling than 97.6%.

THE LIMIT ITSELF, at full strength (:125):
  "Multi-file coordination wall: Agentless produced only 6 multi-file patches among 497 successes.
  On multi-file issues, OpenHands = 25.3% success vs Agentless = 18.3%. Agentic systems maintain an
  edge on cross-file work."
And (:129): "Hard tasks convergence: on tasks requiring >=1h human effort, all tools (agentless and
agentic) converge to 13-20% success. Deterministic harness does not unlock hard tasks."

READ THOSE TWO TOGETHER. They say different things and both matter:
  - on MULTI-FILE work, the pipeline LOSES to the agent (18.3% vs 25.3%) — a 7pp deficit;
  - on HARD work (>=1h human), EVERYTHING converges to 13-20% — the harness buys nothing.
The kernel's domain is therefore: single-file, well-localized, sub-1-hour tasks. That is a real and
commercially meaningful domain (most bug fixes), but it is not "coding."

--- 3.2 QUOTE 2: CONSTRAINT DECAY — EXACT, NO CORRECTION NEEDED

OWNER'S QUOTE: "constraint decay (~30-point drop, arXiv:2605.06445)"

DOC TEXT, three consistent statements:
  (:11) "The 'constraint decay' paper (arXiv:2605.06445) shows a 30-point average drop when
  structural constraints accumulate, and this holds regardless of whether the harness is simple
  (Mini-SWE-Agent) or complex (OpenHands)."
  (:84-87) full entry: "measures how performance declines as structural constraints accumulate
  across 80 greenfield + 20 feature-implementation tasks, 8 web frameworks. What it measured:
  average 30-point drop from unconstrained (L0) to fully constrained (L3). Top performers lose 40%
  of baseline. Database constraints alone impose -19.3pp marginal penalty. Even at L3, best agents
  achieve only 8.3% full compliance. Mini-SWE-Agent and OpenHands both exhibit identical decay
  trends despite radically different scaffold complexity (100-line vs full-featured)."
  (:124) restated in the limits section, consistent.

VERDICT: EXACT. The owner's quote is correct and if anything UNDERSELLS the finding. The full-
strength version adds three figures the quote omits:
  - top performers lose 40% OF BASELINE (not 40 points)
  - database constraints alone: -19.3pp marginal
  - at L3, best agents achieve only 8.3% FULL COMPLIANCE
  - and the killer: "Mini-SWE-Agent (100-line) and OpenHands (full-featured) decay at IDENTICAL
    rates" (:124)

WHY THIS IS THE MOST IMPORTANT LIMIT FOR THE KERNEL THESIS SPECIFICALLY. The survey's own gloss
(:87): "when the model must satisfy cross-file stateful constraints simultaneously, harness
complexity does not prevent failure. The decay is structural, not incidental."
A 100-line harness and a full-featured harness decaying IDENTICALLY is a controlled experiment that
says: on constraint-accumulation tasks, harness investment has ZERO marginal return. Not
diminishing — zero. Whatever the kernel buys, it does not buy this. And "structural constraints"
(ORM, auth, multi-framework) is a fair description of most real backend work.

--- 3.3 QUOTE 3: SCAFFOLD-MODEL SYNERGY — MISATTRIBUTED, AND THE ERROR IS OURS

OWNER'S QUOTE: "scaffold-model synergy (Agentless + GPT-4o-mini 5.1% vs 21.8% with Qwen3-32B,
SWE-Effi arXiv:2509.09853)"

THE DOC SAYS BOTH THINGS. It contradicts itself:
  VERDICT PARAGRAPH (:11): "a small model with a strong harness is NOT universally better
  (Agentless + GPT-4o-mini collapses to EuTB 5.1% vs 21.8% with Qwen3-32B)."   <-- says AGENTLESS
  ENTRY BODY (:89-92): "What it measured: SWE-Agent scaffold + Qwen3-32B = EuTB 21.8%; same
  scaffold + GPT-4o-mini = EuTB 5.1%. Scaffold-model synergy dominates. Agentless pipeline
  maintains stability but at lower peak capability."                            <-- says SWE-AGENT
  LIMITS SECTION (:127): "SWE-Effi scaffold-model synergy: small model (GPT-4o-mini) + strong
  scaffold collapses to EuTB 5.1% vs 21.8% with Qwen3-32B on the same scaffold."  <-- says "strong
                                                                                      scaffold",
                                                                                      unnamed

VERDICT: THE OWNER'S QUOTE IS WRONG, BUT IT IS OUR DOC'S ERROR PROPAGATING. The scaffold is
SWE-Agent, not Agentless. The entry body (:91) is the authoritative statement because it is the
detailed one and it explicitly CONTRASTS Agentless with the measured scaffold in the very next
clause ("Agentless pipeline maintains stability but at lower peak capability" — i.e. Agentless is a
DIFFERENT arm of the same study, with a different profile).
CORRECTED FORM: "the SWE-Agent scaffold scores EuTB 21.8% with Qwen3-32B and 5.1% with GPT-4o-mini;
Agentless separately shows stability at lower peak capability (SWE-Effi, arXiv:2509.09853)."

THIS MATTERS MORE THAN A CITATION NIT, IN BOTH DIRECTIONS:
  (a) AGAINST the thesis: the entry body's Agentless clause ("maintains stability but at lower peak
      capability") is itself a limit — the deterministic pipeline TRADES CEILING FOR VARIANCE
      REDUCTION. That is a real and defensible property, and it is NOT what "the harness makes small
      models good" claims.
  (b) FOR the thesis: the 4x collapse was measured on an AGENTIC scaffold. An agentic scaffold
      demands exactly what small models are worst at (tool selection, multi-turn state). So the
      figure is arguably evidence FOR the thesis's diagnosis (small models fail at agency, not
      code-gen) rather than against it. The survey filed it as refutation (:92: "the thesis's core
      assumption ... is PARTIALLY refuted by this paper"). Under the corrected attribution, that
      filing is too pessimistic — the paper refutes "any scaffold helps any model," which is not the
      thesis.

*** THE SURVEY'S OWN CAVEAT ON THIS FIGURE — READ IT BEFORE CITING ***
(:185): "The SWE-Effi paper (arXiv:2509.09853) PDF yielded limited specific numbers in the fetch;
the EuTB 5.1% vs 21.8% figure came from THE SEARCH SNIPPET, NOT THE FULL PAPER. Treat as
directionally correct but VERIFY EXACT METHODOLOGY if used for quantitative claims."
So: the single most load-bearing anti-portability number in the corpus is a SEARCH-SNIPPET
FIGURE, of uncertain scaffold attribution, never verified against the paper. It is currently
carrying the weight of the T3 verdict. RECON REQUIRED before this number decides anything.
"EuTB" is never expanded anywhere in the doc (presumably an efficiency-under-token-budget metric).
That alone should block citation.

--- 3.4 THE OTHER LIMITS THE OWNER DID NOT QUOTE (full strength, :122-134)

- SWE-BENCH LEADERBOARD DISSECTION (arXiv:2506.17208, :128): "no single architecture dominates;
  both agentic and pipeline approaches achieve competitive results, with 'performance depending more
  on LLM choice and specific implementation strategies than architectural classification alone.'"
  This is a direct hit on T3: architecture class is NOT the explanatory variable. Model choice is.

- NLAH ABLATION (arXiv:2603.25723, :132): "the verifier harness component REGRESSED performance
  (-0.8% SWE-bench, -8.4% OSWorld). NOT ALL DETERMINISTIC HARNESS ADDITIONS HELP."
  -8.4% on OSWorld from adding a VERIFIER is the most uncomfortable number in the file for ELAI
  specifically, whose entire wedge is verifier-owned completion. It is one ablation on one system
  and is not necessarily transferable, but it is unaddressed anywhere in the plan corpus.

- AHE REGRESSION WEAKNESS (:133): "regression prediction by the harness evolution system is weak
  (11.8% precision vs 33.7% fix-precision). The harness can make things worse on regressions."

- AGENTLESS ON SWE-BENCH MULTIMODAL (:134): "agentic systems score 11.5% vs Agentless 3.9% on
  multimodal tasks requiring interactive visual understanding. Fixed pipeline fails on task
  modalities it wasn't designed for." [T2]
  This is the generalization limit: the pipeline is a BET ON A TASK SHAPE. Change the shape, the bet
  loses ~3x.

- DSPy BRITTLENESS (:131): "compile-time optimized prompts are brittle to distribution shift;
  require a training set and many LLM calls at compile time. The compilation is not free." [T1/T2]

--- 3.5 THE LIMITS, RANKED BY DAMAGE TO THE KERNEL THESIS

  1. CONSTRAINT DECAY (2605.06445) — identical decay at 100-line vs full-featured harness. Zero
     marginal return on harness investment for constraint-heavy work. HARDEST LIMIT STANDING.
     Evidence quality: strong (80+20 tasks, 8 frameworks, explicit two-harness control).
  2. SCAFFOLD-MODEL SYNERGY (2509.09853) — 4x swing from model swap under fixed scaffold. Kills
     "portable kernel" in its strong form. BUT evidence quality is WEAK (search snippet,
     misattributed scaffold, unexpanded metric). Damage is currently ASSERTED, not established.
  3. KIMI-DEV'S CEILING CLAIM (2509.23045) — pipeline is a ceiling on inference, not on learning;
     the trained model does better in an AGENTIC harness. Suggests the kernel is a phase, not an
     architecture. Evidence quality: strong. UNDER-WEIGHTED in the survey.
  4. MINI-SWE-AGENT (>74% Verified, 100 lines) — harness value falls as model capability rises. Now
     the SWE-bench control harness. Evidence quality: strong on frontier models; the small-model
     arm is admittedly un-run.
  5. MULTI-FILE WALL (2509.13941) — 6/497; 18.3% vs OpenHands 25.3% on multi-file. Bounds the
     DOMAIN rather than the thesis. Evidence quality: good (count stated twice); the 97.6% figure
     is unreliable.
  6. HARD-TASK CONVERGENCE (13-20% for everything) — the harness does not unlock hard tasks.
  7. NLAH VERIFIER REGRESSION (-8.4% OSWorld) — a deterministic verifier made it worse. Single
     ablation, but directly adjacent to ELAI's wedge.


====================================================================
4. THE GAP / WHITESPACE — WHAT NOBODY HAS DONE
====================================================================

The survey's five-item gap list (:137-149), verbatim in substance. This is the most actionable part
of the survey and I reproduce it fully because it is what a kernel program would BE:

  1. CONTROLLED ABLATION (:141): "small fixed local model + maximal deterministic harness vs same
     model + minimal harness, at production scale on hard tasks. Most ablations either fix the model
     and vary the harness (showing harness contribution) or fix the harness and vary the model
     (showing model contribution). Nobody has published a controlled study specifically testing 'how
     much can you shrink the model if you maximize the harness' across a range of model sizes."

  2. TOOL-SELECTION-FREE DESIGN (:143): "the thesis specifically proposes removing TOOL SELECTION
     from the model (the model never decides which tool to call; the harness routes deterministically
     based on task classification). Mini-SWE-Agent removes tools almost entirely (only bash), but
     that's not the same as deterministic routing; the model still decides what bash commands to
     run. Agentless hardcodes the workflow but the model still does more than pure code generation."

  3. HARNESS AS THE INSTRUCTION-FOLLOWER (:145): "moving reading of rules/docs/constraints into the
     harness (not the model's context) and having the model receive only a constrained generation
     slot. Grammar-constrained decoding does this for FORMAT but not for SEMANTIC RULES ('don't use
     X pattern in this codebase'). Nobody has built a harness where semantic codebase rules are
     enforced mechanically rather than prompted."

  4. DISPROPORTIONATE SMALL-MODEL BENEFIT MEASUREMENT (:147): "the thesis's specific prediction is
     that a small model fails mostly on tool-use and long-horizon instruction-following, not raw
     constrained code-gen. If you eliminate those via harness, small models should disproportionately
     improve. This specific prediction has not been cleanly measured."

  5. COMBINED APPROACH (:149): "Agentless + grammar-constrained decoding + deterministic file routing
     (RIG-style) + DSPy compile-time optimization in one integrated system. These tools exist
     separately; nobody has combined them into a 'maximal deterministic kernel' and measured the
     combined effect on small models. This combination is what the thesis specifically proposes and
     it appears genuinely unstudied."

TWO OBSERVATIONS THE SURVEY DOES NOT MAKE:

(a) GAP 3 IS ALREADY ELAI'S RULES ENGINE. "semantic codebase rules enforced mechanically rather
    than prompted" is a verbatim description of 3F: "deterministic rule enforcement with rules
    stored as DATA (AST/grep/schema/metric/composite predicates), a proposed->shadow->active
    lifecycle with promotion gates, and self-discovery of recurring violations. Security/finance
    rules are immutable." (docs/thefinalglance/ELAI-SPEC-OVERVIEW.md:88-90). The survey declares
    unstudied whitespace that ELAI's own spec fills. Nobody connected them. If gap 3 is real
    whitespace, 3F is a publishable result, not just a subsystem.

(b) GAP 5 IS ELAI'S ACTUAL ARCHITECTURE. Agentless (E-46 shortcut, SHIPPED) + grammar-constrained
    decoding (E-35/E-18 XGrammar, the local mask) + deterministic file routing (Phase-1 DKG/PageRank
    ~= RIG) + compile-time optimization (I-36 cross-model prompt compilation, citing Compiled AI) is
    a list of ELAI components. The survey says this combination "appears genuinely unstudied" and
    "is what the thesis specifically proposes." ELAI is the only system I can identify in this
    corpus that has all four pieces. The measurement (gap 1 + gap 4) is the missing part, and it is
    exactly the same experiment the July market evaluation independently designed (section 6.4).
    THE SURVEY AND THE MARKET EVAL CONVERGED ON THE SAME EXPERIMENT FROM OPPOSITE DIRECTIONS AND
    NEITHER CITES THE OTHER.


====================================================================
5. ELAI'S OWN SMALL-MODEL RESEARCH — THE T4 EVIDENCE THE SURVEY MISSED
====================================================================

Source: workflow/research/small-models-long-horizon-findings.md (2026-06-14 — SAME DAY as the
prior-art survey, and they do not cite each other).

This file answers the survey's gap #4 mechanism question with better evidence than the survey has,
and REFRAMES the kernel thesis's fix. It is the most important doc in the bundle for the kernel
question and it is not part of the harness-as-pipe chain at all.

THE HEADLINE (:15-20): "The dominant cause is NOT raw reasoning ('can't figure out the fix') and NOT
primarily context window size. It is EXECUTION RELIABILITY OVER A LONG HORIZON. Small models make
slightly more errors per step; on long multi-step tasks those errors compound exponentially; and
small models additionally SPIRAL after their own first mistake (self-conditioning). Even when a
small model has 100% single-step accuracy, larger models still execute more steps correctly in
sequence."

MECHANISM 1 — ERROR COMPOUNDING (arXiv:2509.09677, "The Illusion of Diminishing Returns", :24-31):
  "Failures 'arise from mistakes in EXECUTION, rather than an inability to reason.'"
  "'Larger models can correctly execute significantly more turns EVEN WHEN small models have 100%
  single-turn accuracy.'"
  "Marginal single-step accuracy gains compound into EXPONENTIAL gains in completable task length."
  The p^n table (:36-40) — FLAGGED ILLUSTRATIVE, NOT MEASURED (:130-132):
    per-step acc   20-step   50-step   100-step
    99.5%          90%       78%       61%
    97%            54%       22%       5%
    93%            23%       2.6%      0.07%

MECHANISM 2 — SELF-CONDITIONING (:42-55, arXiv:2509.09677 + arXiv:2511.14777):
  "'Models become more likely to make mistakes when the context contains their errors from prior
  turns.' Smaller models self-condition MORE severely: once an error is in their own context,
  P(next error) rises — they spiral. Frontier models are more robust to their own past errors and
  recover. So small models do not just suffer passive p^n decay; they actively get WORSE after the
  first mistake."

MECHANISM 3 — KNOWLEDGE BREADTH (:57-63): secondary, parameter-count-linked. Labelled INFERENCE, not
a direct citation (:133-134).

THE ALE ANCHOR (:5-8): best self-hostable open-weights model (DeepSeek V4 Pro) = 12.4% pass vs
Codex+GPT-5.5 = 24.0% pass. Roughly HALF.
And (:124-125): "agents-last-exam.org/blogs/harness-matters (model accounts for ~3x the pass-rate
spread of the harness; compaction rarely triggered)."

*** THAT LAST FIGURE IS THE SHARPEST ANTI-KERNEL DATUM IN THE ENTIRE BUNDLE AND IT IS IN A
    PARENTHETICAL ***
"model accounts for ~3x the pass-rate spread of the harness" is a DIRECT, QUANTIFIED comparison of
model-variance vs harness-variance on the same benchmark — precisely the measurement the prior-art
survey says nobody has made (gap 1). It says the model matters 3x more than the harness. It is a
practitioner blog [equivalent to T2], from the ALE team, and it is buried in a source list.
If it holds, the kernel thesis's ceiling is set: maximal harness engineering can address at most
~1/4 of the observed spread. RECON REQUIRED — this figure should be verified and then either
promoted to the front of the kernel argument or retired.

RECONCILIATION WITH ELAI'S OWN MEASURED DATA (:80-86) — honest and important:
  "ELAI measured Qwen2.5-Coder 7B at 100% pass / 25K tokens on its own task set... NOT a
  contradiction. Those are short, well-scoped tasks (small n), where p^n stays high even at modest
  p. The small model is genuinely fine there. It collapses specifically when the horizon is long
  AND broad... Correct framing: 'small models are bad at long-horizon, many-step, broad-domain
  execution,' NOT 'small models are bad.'"

WHAT CLOSES THE GAP (:90-97) — THE KERNEL THESIS, REFRAMED AND IMPROVED:
  HELPS: "Extended thinking / CoT: paper found 'thinking mitigates self-conditioning and enables
  execution of much longer tasks in a single turn.'" AND — the load-bearing one — "Harness-level
  error recovery, checkpointing, and DECOMPOSITION into shorter sub-horizons (smaller n per
  unsupervised chain). This is the one lever a harness, not just the model, controls."
  DOES NOT HELP (:99-101): "Just bolting on a bigger context window. The bottleneck is
  reliability-per-step and error-recovery, not window size."

THE DESIGN IMPLICATION (:104-110), which is the best statement of the kernel thesis in the corpus:
  "The fix for running smaller/local models well is NOT a bigger model per se; it is shortening the
  EFFECTIVE HORIZON so n-per-unsupervised-chain stays small: aggressive decomposition, per-step
  verification, and cheap rollback/recovery after the first error (before self-conditioning
  spirals)."

*** THIS IS THE SYNTHESIS THE SWEEP EXISTS TO FIND ***
The kernel thesis says: take work AWAY from the model.
The long-horizon research says: that is the right move but for a DIFFERENT REASON than assumed —
not because the model can't do the work, but because every step it takes is a p<1 coin flip and the
flips MULTIPLY.
This reframing survives every limit in section 3:
  - it does not claim the harness raises the model's ceiling (dodges SWE-Effi, dodges the ALE 3x,
    dodges mini-SWE-agent);
  - it predicts the constraint-decay result rather than contradicting it (constraints raise n and
    couple the steps, so both harnesses decay identically — neither shortens the chain);
  - it explains WHY Agentless is 98.8% single-file and why that is CORRECT behavior rather than a
    defect (single-file = small n; the pipeline is a horizon-shortening device and it stays inside
    its competence);
  - it explains Kimi-Dev (training on verifiable single-turn steps = training on small-n chains;
    the skill that transfers is per-step reliability, which is exactly what raises p and therefore
    lifts the agentic ceiling too — the ceiling claim and the kernel are then COMPATIBLE, not
    opposed);
  - it is measurable (n per unsupervised chain, p per step, first-error-to-recovery latency).
The honest kernel thesis is therefore: THE HARNESS IS A HORIZON-SHORTENING AND ERROR-RECOVERY
DEVICE. Its value is proportional to (a) the horizon length it removes and (b) how badly the model
self-conditions. That predicts the small-model dividend WITHOUT claiming the harness substitutes
for capability. It is T4 restated so that SWE-Effi cannot touch it.
CAVEAT: this is my synthesis across two docs that do not cite each other. It is not a finding either
doc states. It is UNVERIFIED as a claim about the world; it is a hypothesis that the corpus's own
evidence supports and that gap-1/gap-4 would test.


====================================================================
6. THE COMPETITIVE LANDSCAPE
====================================================================

Three assessments, three dates, and they revise each other. Presented in order with the revisions
marked, per instruction never to merge the twins.

--- 6.1 THE GROUNDED-vs-TRAINING DELTA (2026-06-04) — WHERE UNSEARCHED CLAIMS WERE WRONG

The two files are a deliberate experiment: the same author, the same day, the same question, one
run on training memory and one run after 8 web searches + 1 PDF fetch. The TRAINING file is
explicit about its own status: "Every external-ecosystem claim below is 'From training
(unsearched)' unless it derives from a file I read this session."
(workflow/reasoning/ELAI-COMPETITIVE-ASSESSMENT-TRAINING-2026-06-04.md:6-7)

THE DELTA IS THE FINDING. Six headline changes (GROUNDED :11-25). Scored:

  ERROR 1 — "TRUST/REPRO IS EMPTY SPACE" — WRONG, AND THE FILE SAYS SO IN THOSE WORDS.
    TRAINING claimed (:138-146): "AXIS 4 — Trust / verification / reproducibility / off-agent gating
      (ELAI's wedge). From training: I cannot name a mainstream coding-agent PRODUCT that ships
      this... So on the trust axis ELAI is close to first-in-category as a productized, integrated
      thing. This is the real moat candidate."
    GROUNDED found (:50-52): "In Phase 1 I claimed I could not name a productized competitor on
      reproducibility/off-agent gating. THAT IS NOW WRONG. As of 2026 the space is actively
      forming." Named peers: Gryph (SafeDep) — "explicitly 'Audit Trail for AI Coding Agents'... A
      direct product-category peer"; deterministic-replay frameworks (TianPan, Sakura Sky,
      AgenticRail, isimplifyme) whose documented best practice is "to RECORD every LLM call + tool
      response + intercept the system clock and substitute recorded timestamps — which is EXACTLY
      ELAI's ClockProvider/EntropySource quarantine + SSB action stream + ELAI_SEED replay" (:57-60).
    SEVERITY: HIGHEST. The training run declared ELAI's ENTIRE MOAT unoccupied. It was occupied.
    NOTE THE DOUBLE EDGE (:68-80): grounding both VALIDATED the architecture ("the field
      independently arrived at ELAI's design" :64) and KILLED the first-mover claim. The surviving
      differentiator is stated precisely (:76-80): "NONE of these peers is ALSO the coding agent.
      Gryph/observability tools watch an external agent; deterministic-replay frameworks are
      libraries. ELAI is the only one where the gate, the replay, the attestation, AND the agent
      doing the work are one deterministic Rust system."

  ERROR 2 — "WRITE-TIME SECURITY IS UNIQUE (score 5)" — WRONG.
    TRAINING scored write-time security ELAI=5 "(unique)" (:184) and listed only post-hoc scanners
      as analogues, adding "CodeGate (tried write-time security; per F-plan it is DEAD)" (:145).
    GROUNDED found (:87-91): VibeGuard (arXiv:2604.01052) — "a pre-publish security gate framework
      for AI-generated code, explicitly MULTI-AXIS (artifact hygiene, packaging-config drift,
      source-map exposure, hardcoded secrets, supply-chain). This directly overlaps ELAI's F-1
      VibeGate... The F-plan claim 'no competitor unifies the four axes' is NOW OVERCLAIMED."
      Plus Semgrep "Secure 2026" shipping an MCP server usable INSIDE Cursor/Claude Code/Copilot/
      Windsurf — "so the incumbents can bolt on write-time security via Semgrep's MCP — ELAI's
      security gate is MATCHABLE BY A PLUGIN in competitor tools" (:93-95).
    SEVERITY: HIGH. Score dropped 5 -> 4 and a shipped plan claim was invalidated.
    Note the training file ALSO carried a stale fact it stated as current: "CodeGate... per F-plan
      it is DEAD" — grounding found "CodeGate (the config-scanner namesake on GitHub) is alive,
      though it gates AI-TOOL-CONFIGS (MCP/hooks/rules), a different surface" (:97-98). A
      death-report that was both wrong AND about a different product.

  ERROR 3 — "COST-EFFICIENCY IS A WEDGE" — WRONG, IT IS TABLE-STAKES.
    TRAINING (:155-160) hedged well ("So ELAI is NOT alone on cost") but still scored ELAI=4 "(if
      H-Track1 ships)" against incumbents at 3, and called the combination "a coherent, defensible
      package most harnesses assemble only partially."
    GROUNDED found (:113-123): "most of this is ALREADY SHIPPED elsewhere: Aider has architect/editor
      mode (30-50% cheaper — SAME NUMBER ELAI's H-7 cites as +9.6pp Aider), a tree-sitter + PageRank
      repo-map (THE LITERAL SOURCE ELAI's PageRank DERIVES FROM), and prompt caching support."
      Plus: "Prompt caching (cached input at 10-25% of normal) is an Anthropic/OpenAI/Bedrock
      PLATFORM feature any harness uses. Batch API (50% off) likewise." And a cost anchor: "Claude
      Code Haiku 4.5 via Aider/Continue... $0.02 light / $0.25 moderate / $0.81 heavy."
    SEVERITY: MEDIUM-HIGH. The verdict (:125-131): "'cheaper than Claude Code' is NOT a durable wedge
      — the techniques are commodity and Aider already packages them. ELAI's only surviving cost
      differentiator is GD-2: every cost number... is RECOMPUTABLE FROM THE SSB JOURNAL... Sell 'you
      can PROVE the bill,' not 'we are cheap.'"
    The sharpest self-own in the corpus: ELAI was preparing to compete on a technique it BORROWED
      FROM the competitor it was competing against.

  ERROR 4 — THE LOCAL MODEL PIN — STALE BY A FULL GENERATION.
    TRAINING (:66, :93-95) had "Qwen2.5-Coder-7B abliterated (NF4/AWQ)" and knew it was weak ("far
      below frontier... a cost/privacy/offline play, not a quality play") but had no replacement.
    GROUNDED found (:136-142): "Qwen3-Coder-Next / Qwen3-Coder-30B: ~58.7% SWE-bench Verified, 256K
      context, Apache-2.0, Q4_K_M fits a single 24GB GPU. Cited as the default 2026 local pick.
      DeepSeek-Coder-V3 (distilled): ~40.5% SWE-bench Verified on 12GB — best quality-per-GB."
    SEVERITY: MEDIUM, but the HIGHEST-VALUE correction: "This is also the cheapest high-impact fix
      to the local-quality story" (:143).
    A SECOND-ORDER CATCH worth preserving (:144-146): "NOTE the existing project rejection of
      'Qwen3-32B'... that rejection PREDATES the 2026 Qwen3-CODER line and should be re-evaluated
      against Qwen3-Coder-30B specifically (a coder-tuned MoE, DIFFERENT from the rejected general
      Qwen3-32B)." Grounding caught a stale INTERNAL decision, not just a stale external fact — a
      rejection being over-applied by name-similarity. (Corroborated by user memory: "No Gemma 4
      (scoped) — Gemma 4 rejection was box-specific, lifted by J-9/HW-14 to tier-conditional catalog
      data" — the same over-application pattern, caught twice.)

  CORRECT 1 — CAPABILITY PARITY. TRAINING (:88-92): ELAI's ceiling = Claude's ceiling, because it IS
    Claude underneath. GROUNDED (:44-46): "Phase-1 conclusion stands and is now verified by the live
    landscape." The training run's single most important structural conclusion was RIGHT.
  CORRECT 2 — DGM IS RESEARCH-ONLY. TRAINING (:149-153): "no shipping coding product runs a guarded
    recursive self-improvement loop in production." GROUNDED (:152-159): confirmed — DGM is ICLR
    2026 (arXiv:2505.22954, SWE-bench 20%->50%, Polyglot 14.2%->30.7%), research repo, not a
    product. Advice unchanged: "Keep it default-OFF... DO NOT headline it."
  PARTIAL — REGULATORY. TRAINING listed in-toto/SLSA as general, not coding-agent-native (:144).
    GROUNDED refined (:164-173): EU AI Act transparency obligations effective 2 Aug 2026 "but this is
    aimed at AI-generated CONTENT (images/text/audio), NOT AI-generated CODE. The AI-code-specific
    mandate is NOT clearly present." Provenance tailwind real (CRA, EO 14028); AI-code-attestation
    tailwind emergent. "Position as 'DESIGNED TO SUPPORT Article-50 marking'... never 'certified
    compliant.'"

  THE SCORECARD DELTA (TRAINING :173-185 vs GROUNDED :180-192), verbatim:
    dimension                    | training | grounded | why moved
    Verification / trust / proof |    5     |    4     | "peers forming §2"
    Write-time security          |    5     |    4     | "*via Semgrep-MCP; VibeGuard peer"
    Reproducibility/determinism  |    5     |    5     | held ("peers are libs, not agents")
    Windsurf reproducibility     |    1     |    2     | incumbent moved up
    Cost transparency            |    4     |    4     | held, but REASON changed entirely
                                                          ("cheap is commodity; PROVABLE cost is
                                                           the edge")
    Local/offline                |    3     |    3     | "(4 IF model refreshed)"

  *** THE META-FINDING ON TRUSTING UNSEARCHED CLAIMS ***
  Score one point per headline claim: 4 wrong (moat empty, security unique, cost novel, model pin
  current), 2 right (capability parity, DGM research-only), 1 partial (regulatory).
  BUT the errors are not random — they are DIRECTIONALLY BIASED. Every single error was in ELAI's
  FAVOUR. Training memory over-scored ELAI on 4 of 4 differentiators and never once under-scored it.
  The mechanism is legible: training data has no 2026 competitors in it, so "I cannot name a
  competitor" was read as "no competitor exists." ABSENCE OF EVIDENCE WAS SILENTLY CONVERTED TO
  EVIDENCE OF ABSENCE — which is exactly the failure mode the wiring-enforcement doctrine exists to
  prevent in code (R1: "Absence of proof is proof of absence for gate purposes" is the INVERSE
  discipline, deliberately). The corpus enforces fail-closed on its gates and ran fail-OPEN on its
  own market claims.
  What DID survive unsearched: the two STRUCTURAL claims (capability parity is a consequence of
  renting the model; DGM productization is rare). Training memory was reliable about ARCHITECTURAL
  LOGIC and unreliable about WHO EXISTS. That is a usable rule: reason from training, enumerate from
  the web.
  The strategic net (GROUNDED :26-29): "ELAI's strategic identity shifts from 'the only one who does
  trust' to 'the most INTEGRATED and RIGOROUS synthesis of several converging 2026 trends.' Narrower
  moat, but real — and the convergence proves the market is forming around ELAI's thesis rather than
  ignoring it."

--- 6.2 THE HONEST COMPETITIVE POSITION (2026-06-04 GROUNDED, §9 :199-220)

Reproduced because it is the least flattering and most useful paragraph in the bundle:

  "IS IT GOOD? As a DESIGN: yes... ELAI is not chasing a fantasy; it is an unusually rigorous, early,
  VERTICALLY-INTEGRATED bet on the exact direction the trustworthy-agent field is moving.
  AS A PRODUCT TODAY: heavily discounted. Same-model capability ceiling, weak/stale local lane, solo
  bus-factor, mostly-planned roadmap, no published headline number yet, and the once-unique wedges
  (trust, write-time security, cost) now each have funded or published peers. It is good potential,
  not yet a good purchase.
  - On what the median buyer shops for in 2026 (capability, polish, IDE, ecosystem, maturity):
    BEHIND all three. It will not displace them for everyday coding.
  - On the trust cluster: AHEAD of all three — but now level with or racing an emerging category
    (Gryph, Semgrep, VibeGuard, deterministic-replay frameworks) rather than alone.
  - The ONLY honest 'better than X' sentence ELAI can say: 'the same Claude model, but every change
    is reproducible, security-gated before it touches disk, and the bill + the proof are recomputable
    from an append-only journal — in one tool, not a stack of plugins.' Everything else is parity or
    behind."

WHERE ELAI IS GENUINELY BEHIND (be blunt, per instruction):
  - UX/polish/IDE: 2 vs Cursor/Windsurf 5 (:184). Solo, custom dashboard.
  - Ecosystem/community/support: 1 vs 4-5 (:185). "solo, bus-factor."
  - Maturity/stability/shipped: 2 vs 5 (:186). "mostly planned."
  - Raw capability: 5 = parity, structurally cannot exceed (:182). This is not a gap that closes.
  - Local lane: stale by a generation as of 2026-06-04 (:136-142). PARTIALLY ADDRESSED — user memory
    records a local llama.cpp lane LIVE 2026-07-17 (Qwen3.5-4B Q4_K_M, Vulkan, 755pp/92.7tg) and the
    most recent commit retargets optimizer inference at it (git 8bc668b, "retarget the optimizer
    inference legs at the live local llama.cpp lane"). The 24GB Qwen3-Coder-30B pin recommended in
    P3 (:232) is NOT what shipped; a 4B lane on a 12GB card is a different, smaller bet.
  - Config/ops lifecycle: "ELAI's THINNEST AREA"
    (workflow/reasoning/openclaw-vs-elai-comparison-2026-07-11.md:119).
  - Provider failover semantics: "richer than anything in the corpus" — OpenClaw wins (:178).

--- 6.3 OPENCLAW (2026-07-11) — THE JULY REVISION FROM THE PRODUCT SIDE

Source: workflow/reasoning/openclaw-vs-elai-comparison-2026-07-11.md. Mode: spec-eval. Sources: a
22,435-file clone at HEAD 2026-07-10, version 2026.6.11, plus two Opus explorer inventories (:4-7).
Clone VERIFIED present at docs/.source/openclaw this session.
Porting caveat, verbatim (:10): "OpenClaw's commit stream is largely agent-generated at very high
velocity; quality is uneven by construction. STEAL DESIGNS, NEVER CODE."

VERDICT (:16): "These are different machines aimed at different problems. OpenClaw is a shipped,
operationally mature PERSONAL ASSISTANT GATEWAY... ELAI is a VERIFICATION SPINE for autonomous
execution... OpenClaw has nothing that answers ELAI's core thesis — it has no seeded determinism, no
replay-verify, no off-agent done-gates at runtime, no claim grounding, and it EXPLICITLY declares
prompt-injection-only attacks out of scope (SECURITY.md triage boundary)... Money and size bought
OpenClaw breadth and polish; it did not buy a trust spine. ELAI's wedge survives contact fully
intact — and gets sharper, because OpenClaw is a perfect example of the 'agent infrastructure
without receipts' the wedge sells against."

THE KERNEL-RELEVANT FINDING (:26): "OpenClaw's trajectory is a FLIGHT RECORDER — you can inspect
what happened; you cannot re-execute and verify it happened. No seed, no entropy control, no
hash-bound verdicts, no replay-verify contract." That distinction (recorder vs replay) is the
cleanest articulation of ELAI's wedge anywhere in the bundle.

WHERE OPENCLAW WINS — DESIGN LEVEL (:177-184), i.e. real gaps not excusable as maturity:
  O1 Provider failover semantics — "richer than anything in the corpus"
  O2 Approval drift-binding — "a stated invariant the corpus lacks". The mechanism (:42): approval
     BINDS a canonical systemRunPlan (cwd/argv/env/pinned executable + one concrete file operand);
     frozen between prepare and exec; "caller edits after approval = mismatch rejection; file operand
     changed after approval = DENIED (drift protection, anti-TOCTOU)."
  O3 Config/ops lifecycle — "ELAI's thinnest area vs the trust story it sells" (:119)
  O4 Channel policy depth — "years of encoded edge cases"
  O5 A normative written security triage boundary — "the discipline, not the boundary itself"
  O6 (MATURITY, explicitly excluded from the verdict) "~22k files with daily velocity, a real
     (NVIDIA/Tencent-staffed per SECURITY.md) security team, a live registry, native apps, and an
     installed base... That is a distribution and hardening lead, not an architectural one."

THE SHARPEST COMPETITIVE LINE IN THE WHOLE BUNDLE (:38): OpenClaw's SECURITY.md declares
prompt-injection-without-a-policy-bypass NOT a vulnerability, "which is defensible for its product
(one trusted human on their own box) but it means the harness is structurally unsafe the moment
untrusted content flows in — which is EVERY message from a ~30-channel ingest surface." Citable from
their own written policy, tagged [EXTERNAL].

INDEPENDENT CONVERGENCE = DESIGN VALIDATION (:190). Five places OpenClaw and ELAI arrived at the
same design with no contact: accessibility-tree-first browser control; zero-unknown evidence
classification; single-source protocol schemas; on-demand context loading; successor-transcript
compaction. The convergence is evidence the designs are right.

OUTCOME — THIS ONE CLOSED THE LOOP (:197-208): S1..S20 were AUTHORED INTO PLAN FILES as Track
OPENCLAW-STEAL: P-17 (provider failover, feeds PS-4), P-18 (durable TaskSpec + approval
drift-binding), P-19 (config/ops + elai doctor + verified backups), P-20 (context/memory lifecycle),
P-21 (evidence/gate-credibility, feeds PS-6), P-22 (control-plane/channel hardening). "All
plan_gate_check.py PASS (exit 0)."
CONTRAST WITH SECTION 7: the OpenClaw comparison (July) produced six plan units in the same session.
The harness-as-pipe survey (June) produced zero, ever. Same corpus, same author-class, opposite
outcomes. The difference is that paper-to-plan fired for one and not the other.

--- 6.4 THE MARKET EVALUATION (2026-07-10) — THE DEEPEST REVISION

Source: workflow/reasoning/elai-harness-market-evaluation-2026-07-10.md. 25+ searches, 35+
primary-source opens, first-party docs only (:5, :681).

EXECUTIVE VERDICT (:9-11): "No: it is not defensible to call ELAI the best harness overall, or the
best harness currently. I found no controlled public benchmark covering ELAI, Claude Code, Codex,
Hermes, Kilo, OpenHands, Copilot, Cursor, Devin, Kiro, Gemini CLI, Cline, and the smaller research
harnesses while holding the model, task, repository, compute, and evaluation constant.
A narrower claim is defensible: as specified, ELAI is one of the most complete documented designs
reviewed for trust-evidenced autonomous execution."

WHY NO BENCHMARK IS POSSIBLE — and the datum that justifies the whole verification wedge (:685):
"Anthropic found INFRASTRUCTURE ALONE can move agentic benchmark results BY UP TO SIX PERCENTAGE
POINTS, while SWE-bench now holds the mini-SWE-agent harness fixed when comparing models. Vendor
demos therefore cannot establish an overall winner."
(anthropic.com/engineering/infrastructure-noise, confirmed :798.)
Six points of pure infrastructure noise is larger than most claimed harness improvements. It means
almost every published harness comparison is unfalsifiable. That is an argument FOR the kernel
thesis's discipline (determinism, pinned environments) that neither the survey nor the competitive
files make.

THE COMPETITIVE SET, no single winner (:687-697). Verbatim leader list:
  Best native collaborating-agent UX: Claude Code agent teams (still experimental)
  Best issue-driven continuous orchestration design: OpenAI Symphony (reference impl is an
    engineering preview; spec is Draft v1)
  Best enterprise repository control plane: GitHub Copilot (explicit Fleet dependency graph)
  Best provider-independent personal learning agent: Hermes Agent
  Best open composable agent SDK foundation: OpenHands SDK
  Best provider-neutral multi-session IDE workflow: Kilo Agent Manager
  Best formal commercial spec-first workflow: Kiro
  Best polished local/cloud fleet IDE: Cursor
  Best transparent minimal research harness: mini-SWE-agent
(Labels are "analyst judgments among the reviewed public mechanisms, not benchmark-established
winners" :677.)

COMPETITOR-BY-COMPETITOR, where ELAI is genuinely behind (:140-198):
  CLAUDE CODE (:140-144) — subagents + experimental agent teams with independent sessions, shared
    tasks, dependencies, peer messaging, task claiming; hooks, permissions, checkpoints, memory.
    "ELAI has the stronger evidence architecture. Claude Code has the more focused interaction
    design and a native peer-collaboration system that users can operate WITHOUT UNDERSTANDING A
    LARGE FORMAL LIFECYCLE." <- that clause is the UX indictment.
  CODEX/SYMPHONY (:146-152) — subagents across app/CLI/IDE, repo guidance, skills, MCP, opt-in local
    memories, sandboxing. Symphony turns an issue tracker into a durable control plane with isolated
    workspaces, retries, reconciliation, observable worker runs. "OpenAI reports up to a 500%
    increase in landed PRs on some internal teams, but that is a VENDOR RESULT, not an independent
    benchmark" (:720). Codex hooks limitation: "command handlers are the dependable path;
    prompt/agent handlers and async/interception coverage should not be treated as universal
    enforcement" (:148).
    THE LESSON THAT MATTERS MOST TO ELAI (:150): "OpenAI's engineering account reports that RIGID
    AGENT NODES AND STRICT TRANSITIONS BECAME TOO LIMITING AS MODELS IMPROVED, motivating
    objective-driven workers with tools and context." Carefully sourced: "that lesson comes from the
    engineering account, not the normative Draft v1 spec."
  HERMES (:154-158) — "the closest philosophical competitor: provider/local-model independence,
    persistent memory, agent-authored skills, delegation, cron, messaging gateways, plugins, MCP,
    searchable sessions, and RL tooling." Two architectural rules from its AGENTS.md: "PROMPT
    CACHING IS SACRED, and the core should be a NARROW WAIST with capabilities at the edges."
    Verdict: "ELAI's verification is much deeper; HERMES' PERMANENT CORE AND EXTENSION STORY ARE
    CLEANER."
  KILO (:160-164) — 30+ providers, BYOK, context-isolated subagents, permissions, Agent Manager with
    worktree isolation. "Kilo DEPRECATED its dedicated Orchestrator mode because capable primary
    agents can delegate directly. That is another signal that orchestration should be adaptive and
    task-shaped, NOT CEREMONY IMPOSED ON EVERY JOB."
  OPENHANDS (:166-170) — typed serializable SDK, provider-independent, local/Docker/remote/K8s
    workspaces. "ELAI's verifier-owned completion is stronger. OpenHands is currently the CLEARER
    REUSABLE SOFTWARE-AGENT FOUNDATION and has a STRONGER PUBLIC BENCHMARK/RESEARCH IDENTITY."
  GITHUB COPILOT (:172-176) — custom agents, subagent orchestration, MCP, repo-scoped cloud work,
    hooks, memory, PR/CI, firewall, RBAC, audit events; CLI Fleet = explicit task/dependency graph.
    "ELAI is more vendor-neutral and more ambitious about replay. GITHUB OWNS THE REVIEW/DISTRIBUTION
    SURFACE WHERE ORGANIZATIONS ALREADY WORK. K-3's PR/CI evidence emitter is therefore
    STRATEGICALLY ESSENTIAL, NOT COSMETIC."
  CURSOR / DEVIN (:178-184) — polished fleet UX / managed autonomy with playbooks, schedules, session
    timelines, isolated VMs. "ELAI has the better auditability design. These competitors are better
    examples of REDUCING ORCHESTRATION INTO AN UNDERSTANDABLE PRODUCT."
  KIRO (:192-194) — spec-first; boundary gaps: subagents cannot access Specs, hooks do not run inside
    them, curated models not BYOK, stdio MCP subprocesses can execute outside the agent sandbox.
  ROO CODE (:196-198) — "no longer a current contender": repository archived, extension shut down
    15 May 2026. A competitor DIED between the June and July assessments.

*** THE RISKY DIVERGENCE — THE MARKET'S CLEAREST SIGNAL RUNS AGAINST ELAI'S ARCHITECTURE (:227-239)
  "The strongest 2026 product signals favor MORE CAPABLE AGENTS INSIDE SIMPLER, OBJECTIVE-ORIENTED
  HARNESSES:
    - Symphony says rigid state-machine nodes did not age well;
    - Kilo removed its dedicated orchestrator;
    - mini-SWE-agent deliberately shrank the scaffold;
    - Hermes keeps a narrow waist and pushes capability to extensions."
  FOUR INDEPENDENT VENDORS MOVED THE SAME DIRECTION: away from elaborate scaffolds. ELAI's nine-state
  FSM is the elaborate scaffold. This is the market-side twin of mini-SWE-agent's counter-thesis and
  the constraint-decay result, and it is the strongest external evidence against the kernel thesis's
  MAXIMALIST form.
  THE RESOLUTION (:236-239), which I judge correct and which preserves the kernel where it earns its
  place: "strict at boundaries: credentials, writes, network, irreversible actions, evidence
  admission, publication, and completion; flexible inside the boundary: planning, exploration,
  decomposition, testing order, and agent topology selected dynamically for the objective."
  Note this is NOT "less kernel." It is kernel WHERE DETERMINISM IS CHEAP AND DECISIVE (boundaries),
  model WHERE JUDGMENT IS THE POINT (interior). It maps exactly onto the prior art: Agentless's
  slots, Life-Harness's H2 gate, grammar-constrained decoding — every successful instantiation is a
  BOUNDARY mechanism, not an interior one. The one that tried to determinize the interior (VerMCTS,
  planning) needs a formal verifier and does not generalize.

THE MARKET IS CONVERGING ON ELAI'S BROAD DIRECTION (:204-219). Twelve items, verbatim: isolated
execution in worktrees/containers/VMs; durable tasks not disposable chats; parallel specialized
agents; repository-native instructions/skills/specs; MCP/plugins as external capability planes;
deterministic hooks and policy enforcement; local/cloud/hybrid + remote steering; persistent
inspectable memory; OTel-compatible events/traces/artifacts; human approval/interruption/rewind/
review/escalation; tests/CI/screenshots/videos as proof-of-work; measured feedback loops.
"OpenAI describes the engineering shift as humans specifying intent and building environments and
feedback loops while agents execute (openai.com/index/harness-engineering/). Anthropic's 2026 harness
work uses planner/generator/evaluator separation, structured handoffs, clean context resets, and a
skeptical external evaluator (anthropic.com/engineering/harness-design-long-running-apps). These are
close to ELAI's central ideas."
NOTE: "harness engineering" is now an OpenAI-titled engineering post. The discipline the June survey
called "a growing named discipline" (:9) got named by a frontier lab within the month.

WHERE ELAI IS AHEAD, ON ONE AXIS ONLY (:221-225): "Commercial products increasingly produce
proof-of-work, but their public designs usually treat VERIFICATION AS CONFIGURED WORKFLOW. ELAI
treats it as THE PRODUCT: independent receipts, a gate manifest, replay, controlled ablation, a
standalone verifier, and an evidence interchange format... It maps to the likely bottleneck as
autonomous work scales: human attention cannot inspect every action, so machines need compact,
trustworthy evidence about what happened and what remains uncertain."
And (:120): "No public competitor source reviewed here describes this exact joined artifact: taint +
side effects + gate receipts + replay verification + a standalone verifier."

THE SIX P0 TRUST-CRITICAL DESIGN FLAWS (:292-355) — "design defects, not 'not built yet' complaints":
  P0-1 (:296-301) A journal failure cannot be allowed to disappear. B-2 says an SSB write failure
    must not block an FSM transition (B-PLAN-SUMMARY.md:114-124), contradicting the core contract
    and the fail-closed thesis. "A transition without its journal row breaks replay and
    completeness. Use a transactional outbox/commit protocol or fail/quarantine the transition."
  P0-2 (:302-307) G-3's shell-expansion plan is unsafe and technically wrong. "The plan says
    std::fs::canonicalize resolves ~, globs, and symlinks. IT RESOLVES SYMLINKS AND FILESYSTEM
    PATHS; IT DOES NOT PERFORM SHELL TILDE OR GLOB EXPANSION." A factual API error in a security
    gate's design.
  P0-3 (:308-312) A Merkle root is not a signature. "G-8 describes a local BLAKE3 Merkle proof as
    the offline signing default. A Merkle root proves internal integrity relative to itself; an
    attacker who changes the artifact can recompute it. Use a keyed signature such as Ed25519."
  P0-4 (:314-331) The full receipt contract is fragmented rather than canonical — F-4 binds
    stdout_hash, G-2 adds run/model/hardware pins, H-25 adds provenance, I-18 the row chain, I-45
    the objective hash. "One canonical TestExecutionReceipt should bind all of these" — 12 named
    fields listed.
  P0-5 (:333-337) `elai wrap` is bypassable without whole-process confinement. "An external CLI with
    direct filesystem, process, credential, or network access can bypass that proxy. MCP
    INTERCEPTION ALONE IS NOT A SECURITY BOUNDARY."
  P0-6 (:339-351) The determinism claim needs one normative matrix. The overview's whole-session
    byte-recomputation wording is STRONGER than the detailed qualifications. The proposed honest
    table (:345-351):
      pure harness/FSM/journal fold      -> byte-recomputable under pinned version and seed
      local inference, same build/hw class -> token-hash verifiable if deterministic kernels pass
      local inference, different hw class  -> divergence-localizable, not necessarily token-identical
      cloud model                          -> action/receipt stream authenticated; model tokens NOT
                                              replay-guaranteed
      external live world                  -> historical re-execution only against captured
                                              bytes/cassette
  P0-6 IS THE KERNEL-THESIS-CRITICAL ONE. "Byte-recomputable session" is the strongest form of the
  determinism claim and it is only true for the HARNESS LAYER. On a cloud model — ELAI's quality lane
  — you get an authenticated action stream, not deterministic replay. The kernel is deterministic;
  the thing it wraps is not. That is not fatal (an authenticated action stream is still a real
  product) but the spec overstates it at :21-27 and the corpus knows it.

THE JULY ADDENDUM — A SELF-CORRECTION WITHIN THE SAME DOCUMENT (:803-909)
Dated the same day, marked "corrective product-strategy addendum. This addendum SUPERSEDES the
wrapper-first recommendation in Sections 7.3, 8, and 11 where they conflict" (:806).
  WHAT IT RETRACTS (:814-824): "I-40 WAS INTERPRETED TOO BROADLY. The detailed I-40 plan defines two
  tiers but does not make the wrapper the primary product: 'wrapper = wedge + standard track'; 'the
  deep ELAI orchestrator with full wiring stays the premium destination'; 'The wrapper is the on-ramp,
  not the product.' (I-40_trust_layer_wrapper_rra_spec.md:34-35). Therefore, the earlier statement
  that the specification already chooses wrapper-first Strategy B is TOO STRONG."
  The main report had read the plan corpus wrong and cited it in support of a strategy the plan
  explicitly disclaims. The addendum caught it by reading the DETAIL FILE instead of the summary
  index — the same failure mode N-plan diagnoses corpus-wide ("prior letters were authored against
  summaries", ELAI-SPEC-OVERVIEW.md:421-422).
  WHY WRAPPER-ONLY LOSES (:844-857): a proxy "does not own that agent's internal: reasoning and
  grounding protocol; task decomposition and search topology; context allocation and compaction;
  branch selection and evidence-ranked search; recovery, handoff, and anti-spiral behavior;
  verifier/worker separation... The wrapper can export part of ELAI's trust spine. It cannot, by
  itself, express the full native capability architecture."
  *** THAT PARAGRAPH IS THE DIRECT ANSWER TO THE OWNER'S KERNEL QUESTION. It is a list of exactly
  what a portable kernel CANNOT carry. Everything on it is interior reasoning/orchestration; nothing
  on it is a boundary. It independently reproduces the boundary/interior split of :236-239. ***
  THE CORRECTED STRATEGY (:869-878): "1. Compact trust kernel — stable receipts, journal,
  confinement, policy, provenance, independent verification, rollback, evidence schemas. 2. Native
  ELAI flagship... 3. Wrapper and adapters — compatibility, adoption, conformance testing, evidence
  interchange. 4. Signed capability packs... In short: KERNEL-FIRST ENGINEERING, NATIVE-FIRST PRODUCT
  IDENTITY, WRAPPER-ASSISTED DISTRIBUTION."
  THE HERMES LESSON, correctly read (:842): "'Smallest footprint' governs HOW CAPABILITY IS WIRED
  into the core, NOT WHETHER THE PRODUCT MAY GROW; Hermes is 'expansive at the edges and conservative
  at the waist.' The appropriate lesson for ELAI is a small permanent kernel with rich native
  capabilities at typed, measurable edges — NOT A RECORDER-ONLY PRODUCT."
  FINAL (:907-909): "ELAI should be a NATIVE, model- and runtime-agnostic autonomous harness WITH AN
  EXPORTABLE TRUST KERNEL, not a trust wrapper with an optional native harness."

THE EXPERIMENT THAT DECIDES IT (:882-905) — three arms, the same shape as the survey's gap 1:
  1. the competitor harness unmodified; 2. the same competitor under `elai wrap`; 3. native ELAI on
  the same model. Hold repo/task/model/settings/compute/tools/evaluation constant. Measure verified
  completion, escaped defects, evidence-corruption kill rate, reviewer minutes, recovery after
  interruption, wall time/cost/gate tax, false-positive rate.
  "This separates two causal questions: how much value comes from the PORTABLE TRUST KERNEL ALONE;
  how much additional value comes from ELAI's NATIVE reasoning and orchestration."
  THAT IS THE OWNER'S QUESTION, OPERATIONALIZED, ALREADY DESIGNED, INSIDE THE CORPUS. It is the
  single highest-value un-run experiment in this sweep and it is the same experiment as prior-art
  gap 1 + gap 4 in different clothing.
  Pre-registered kill rules (:564-570) include: "if `elai wrap` cannot prove complete mediation,
  narrow its security claim rather than market it as complete."

THE HOLDING PATTERN UNTIL THEN (:572-574): "ELAI is designed to maximize independently verifiable
trust per dollar. Its superiority is benchmark-pending."


====================================================================
7. LAYER-2 TRACE: WHAT BECAME PLAN WORK, WHAT DID NOT
====================================================================

Instruction: "Research with no plan behind it is a finding." It is the biggest one here.

--- 7.1 THE SURVEY ITSELF IS ORPHANED

VERIFIED THIS SESSION:
  grep -rl "harness-as-pipe" docs/           -> ZERO hits
  grep -rl "small-models-long-horizon" docs/ -> ZERO hits
Neither 2026-06-14 research doc is cited by ANY plan file. Both are orphans.
Contrast: the OpenClaw comparison (2026-07-11) authored six plan units (P-17..P-22) in its own
session and recorded the outcome in its own file (openclaw-vs-elai-comparison-2026-07-11.md:197-208).
The difference is procedural, not intellectual: paper-to-plan fired for OpenClaw and not for the
survey. Per CLAUDE.md the corpus HAS a skill for exactly this ("paper-to-plan: drop paper/repo links
> authored ELAI plan files in one session"). It was not run on the survey's 27 citations.

--- 7.2 BUT THE CONSTITUENTS WERE ABSORBED EARLIER AND INDEPENDENTLY — AND ABSORBED WELL

The survey did not need to produce plans because the papers it surveyed had ALREADY been ported,
mostly during the E-plan sweep (2026-05):

  AGENTLESS -> E-46 leaf shortcut. SHIPPED, VERIFIED IN LIVE CODE.
    Plan: docs/thefinalglance/E-plan/E-46_planning_decomposition.md:39
    Code: .elai_cc/crates/orchestrator/src/agentless_shortcut.rs (READ THIS SESSION)
    Contract: .elai_cc/crates/contracts/tests/planning_decomposition_e46_contract.rs
    *** THE FINDING OF THIS SECTION *** The eligibility predicate encodes the CROSS-FILE LIMIT as a
    routing gate. Verbatim from the live module header (:13-14, :104-111):
      "1. Single-file change — `task.criteria` mentions exactly one file path."
      Eligibility variants: `SingleFileSmall` / `SingleFile` ("Single-file change with no
      cross-crate / network / LOC concerns").
    And from the plan (:39): the shortcut fires only if "(a) blast radius <= 1 file, (b) blast radius
    <= 1 symbol, (c) Step acceptance criteria contains no 'design', 'architecture', 'refactor'
    tokens, (d) no cross-language pair touched"; on validate-failure it "fall[s] back to standard
    BabyStep decomposition with the failed-shortcut trace attached as reflection-buffer input."
    THIS IS THE CORRECT ENGINEERING RESPONSE TO SECTION 3.1. Agentless is 98.8% single-file and
    loses to agentic systems on multi-file (18.3% vs 25.3%). E-46 does not fight that; it ROUTES on
    it — deterministic pipeline inside its competence, decomposition outside it, with a fallback.
    ELAI absorbed the technique AND its ceiling as a predicate, months before the survey named the
    ceiling. It is also, in miniature, exactly the boundary/interior split the July eval recommends.
    NUMBER CAVEAT: E-46 cites Agentless as "32.00% at $0.70/issue" (v2); the survey says "27.33% at
    ~$0.34/issue" (v1). See section 2.1.

  LIFE-HARNESS -> J-12 runtime intervention harness. PLANNED, NOT SHIPPED.
    Plan: docs/thefinalglance/J-plan/J-12_runtime_intervention_harness.md
    Index: J-SUMMARY_INDEX.md:359 — "### J-12 — runtime intervention harness [Life-Harness 4-layer
      port; LLM-free at runtime]"
    Source clone: docs/.source/life-harness — VERIFIED PRESENT
    Premise, verbatim (J-12:5): "Life-Harness (arXiv:2605.22166, MIT, source at
      docs/.source/life-harness) shows a frozen-model agent improved across 116/126 model-env
      settings by adapting the RUNTIME INTERFACE only — four deterministic intervention layers, zero
      LLM in the loop at eval time. The mechanism survived an 18-backbone transfer because the
      structure is environment-side. ELAI has every seam the four layers need (rules engine, hook
      dispatcher, skill library, anti-spiral) BUT NO UNIFIED INTERVENTION TAXONOMY and no
      failure-to-intervention authoring pipeline."
    NEW FIGURES NOT IN THE SURVEY — J-12 has per-layer ablations the survey lacks: "-61.7% Airline
      without the action gate, -86.5% ALFWorld without trajectory regulation." Correctly tagged:
      "are EXTERNAL figures [DESIGN], never claimed; the done-gate produces ELAI-measured deltas."
    FIGURE RECONCILIATION: J-12's "116/126" vs survey's "92% of 126" — 116/126 = 92.06%. CONSISTENT.
      J-12's "18-backbone transfer" vs survey's "17 other models" — consistent (17 others + 1 origin
      = 18 backbones). No conflict. Good.
    STATUS: not in elai.map; no *intervention_harness* file in .elai_cc. J is 3/9 wired, 6 partial
      per elai_status_query (snapshot 25m, git 8bc668b). J-12 is PLANNED.
    *** J-12 IS THE KERNEL THESIS'S OWN PLAN UNIT AND IT IS UNBUILT. *** If the owner wants to test
    the portable-kernel idea, J-12 is the shipping vehicle that already exists on paper, already
    grounded at source (MEDIUM reliability, verbatim shapes + BM25 params + call order in
    workflow/papers/life-harness-source-interface.md), with the ELAI seams already named. Note the
    plan's own diagnosis of the gap: ELAI has all four SEAMS but "no unified intervention taxonomy."
    That is the same fragmentation disease N-plan treats corpus-wide.

  COMPILED AI -> I-36 cross-model prompt compilation. Cited at I-36:105 with "WebSearch-confirm
    before headline" — correctly gated per the claims policy.

  XGRAMMAR / CONSTRAINED DECODING -> the DEEPEST absorption, and now a FRAGMENTATION LIABILITY.
    E-35 substrate (E-SUMMARY_INDEX.md:306), E-18 call sites (:170), C-26 DecodingMode, I-34
    CRANE+grammar-pin.
    N-plan flags it: "machinery exists in >=8 places" (N-SUMMARY_INDEX.md:266).
    The thesis's cleanest win (format-correctness moved to the decoder) is implemented eight times
    in ELAI. The kernel idea absorbed so well it fragmented.

  KIMI-DEV + AGENTLESS as philosophy -> E-21 peer systems.
    E-21:10: "pipeline / 'agentless' scaffolds match or beat ReAct loops at a fraction of token cost,
    and Kimi-Dev (arXiv:2509.23045) frames the agentless mode as a skill prior — meaning THE
    HARNESS-FIRST PHILOSOPHY THAT ELAI IS BUILT ON NOW HAS EXTERNAL VALIDATION."
    E-21 also names the standing gap (:10): "ELAI HAS NO PUBLISHED SWE-BENCH NUMBER TO POSITION
    AGAINST." Peer numbers it lists: AutoCodeRover 37.3% SWE-lite pass@1 (arXiv:2404.05427);
    OpenHands 72.8% SWE-Bench-Verified via inference-time scaling + critic-model rerank
    (openhands.dev Nov 2025); SWE-agent ACI 12.29% (arXiv:2405.15793, NeurIPS 2024).

--- 7.3 WHAT REMAINS RESEARCH-ONLY (no plan unit, anywhere)

  - LLMCOMPILER (parallel DAG + deterministic scheduler). No port. ELAI has BSP supersteps (3D) which
    is a deterministic parallel barrier design, but nothing cites LLMCompiler and the DAG-emit +
    deterministic-schedule pattern is not a plan unit.
  - REPOSITORY INTELLIGENCE GRAPH (12.2% accuracy / 53.9% time, on Claude Code + Cursor + Codex). No
    citation anywhere in docs/. This is the closest external validation of ELAI's Phase-1 knowledge
    engine and nothing references it. GAP.
  - VerMCTS. No port (reasonable — needs a formal verifier; J has a Kani+TLA+ lane but for verifying
    ELAI's own core, not for MCTS-over-synthesis).
  - DSPy as compile-time optimizer. Referenced only as a STYLE ("DSPy-style metric-driven validation",
    I-SUMMARY_INDEX.md:237; "DSPy/GEPA demonstration into the planner/executor",
    J-SUMMARY_INDEX.md:2104). No compile-time optimization unit.
  - THE THREE LIMITS THEMSELVES (constraint decay, scaffold-model synergy, hard-task convergence). No
    plan unit engages them. E-46 handles the cross-file limit by ROUTING, but nothing addresses
    "constraint accumulation defeats harness investment" or "harness value is model-conditional."
    These are the facts that would decide whether to invest further in the kernel and they are
    un-owned.
  - THE SMALL-MODEL LONG-HORIZON FINDINGS. Orphaned. Its design implication (shorten the effective
    horizon; decomposition + per-step verification + cheap rollback before self-conditioning) is
    arguably ELAI's whole architecture — the file says so itself (:107-110) — but no unit cites it,
    and the specific measurable it implies (n per unsupervised chain) is not instrumented anywhere.
  - NLAH's -8.4% OSWorld VERIFIER REGRESSION. Unaddressed. ELAI's wedge is verifier-owned completion;
    an ablation showing a verifier component REGRESSING performance deserves a RECON_NEEDED at least.

--- 7.4 WHERE THE THESIS LIVES IN THE SPEC (it is the spec's first paragraph)

  ELAI-SPEC-OVERVIEW.md:14-19: "ELAI is a local-first, deterministic coding-and-reasoning agent whose
  product is not the model but the HARNESS around it: a verification spine that makes an agent's
  claims checkable, reproducible, and hard to fake. THE MODEL IS TREATED AS A REPLACEABLE COG — it
  generates and selects; the deterministic harness sizes the work, owns the state, runs the gates,
  and decides 'done' off artifacts the model did not author."
  The three recurring properties (:21-27): off-agent evidence; fail-closed; determinism.
  The closing invariant (:591-593): "the model generates and selects; the deterministic harness sizes,
  gates, journals, and decides 'done' off evidence the model did not author — measured, fail-closed,
  replayable."
  4M makes it literal (:129-131): "one workflow spec drives many runtime backends interchangeably...
  via a single RuntimeAdapter trait" — described in B as "the 'model is a replaceable cog' thesis made
  literal at the runtime layer" (:150).
  J propagates it as doctrine (:324-326): "the 'model is a cog' doctrine made structurally enforced at
  every lifecycle seam (a thin frozen floor + a cross-family advisory + a fail-closed debt ledger + a
  measured rate + a program falsifier)."
  CRITICAL READ: ELAI's spec thesis is the SURVIVING form, not the refuted one. It says the harness
  owns STATE/GATES/COMPLETION — boundaries. It does NOT claim the harness makes a small model smart.
  The spec has been on the right side of the SWE-Effi limit the entire time. The "frozen kernel"
  reading is a stronger claim than ELAI's own spec makes.


====================================================================
8. CONTRADICTIONS ACROSS THE CORPUS (June -> July)
====================================================================

C1. IS THE TRUST WEDGE OCCUPIED?
  2026-06-04 GROUNDED (:50-52): YES, forming category — Gryph, deterministic-replay frameworks,
    VibeGuard, Semgrep. "first-in-category is no longer true."
  2026-07-10 MARKET EVAL (:120): "No public competitor source reviewed here describes this exact
    joined artifact: taint + side effects + gate receipts + replay verification + a standalone
    verifier."
  2026-07-11 OPENCLAW (:16): "OpenClaw has nothing that answers ELAI's core thesis."
  NOT ACTUALLY A CONTRADICTION — a RESOLUTION AT DIFFERENT GRAIN. June found peers on INDIVIDUAL
  AXES (audit trail; replay; write-time security). July found nobody with the JOINED artifact. Both
  true. The June "surviving differentiator" (:76-80: "NONE of these peers is ALSO the coding agent")
  is what July's :120 restates with a wider search (25+ searches, 35+ primary opens). The wedge
  narrowed from "trust" to "trust JOINED and NATIVE", and has held there through two independent
  July passes. That is the most reassuring finding in the bundle.

C2. WRAPPER-FIRST OR NATIVE-FIRST? — A DOCUMENT CONTRADICTING ITSELF ON THE SAME DAY.
  2026-07-10 main report (:17-25, :496-524): wrapper-first. "make the already-specified external-agent
    wrapper and evidence format the first product"; "Strategy B: trust-kernel first (recommended)";
    "The spec already chooses the two-tier version of Strategy B in I-40."
  2026-07-10 ADDENDUM (:806, :814-824, :907-909): RETRACTED. "This addendum supersedes the
    wrapper-first recommendation in Sections 7.3, 8, and 11 where they conflict"; "I-40 was
    interpreted too broadly"; "ELAI should be a native... harness WITH AN EXPORTABLE TRUST KERNEL,
    not a trust wrapper with an optional native harness."
  AUTHORITATIVE: THE ADDENDUM. It is dated later, explicitly supersedes, and — decisively — it read
  the I-40 DETAIL FILE where the main report read the summary index. The detail file says "The
  wrapper is the on-ramp, not the product" (I-40_trust_layer_wrapper_rra_spec.md:34-35).
  *** THIS IS THE MOST IMPORTANT CONTRADICTION FOR THE OWNER'S KERNEL QUESTION. *** The owner's
  framing ("value extractable as a portable kernel, usable on any harness") IS the retracted
  wrapper-first position. The corpus considered it, recommended it, then retracted it WITHIN THE SAME
  DOCUMENT for a stated reason that survives scrutiny: the wrapper cannot carry the interior
  (:844-857). Anyone reading only §8 of that report gets the retracted answer.

C3. IS THE NINE-STATE FSM AN ASSET OR A LIABILITY?
  Spec (ELAI-SPEC-OVERVIEW.md:76-79): 3A FSM is a core asset — 9 states, evidence gates on every
    transition, risk-scaled HITL.
  2026-07-10 (:227-239): a market-divergence RISK. Symphony's rigid nodes "did not age well"; Kilo
    removed its orchestrator; mini-SWE-agent shrank; Hermes narrow-waist. "The risk is using states
    to prohibit useful actions or force ceremonial role transitions rather than merely controlling
    high-risk commitments."
  2026-07-10 ADDENDUM (:859-867): partially walks it back. "'Simpler orchestration' does not mean 'no
    native orchestrator'... Symphony's specification NEVERTHELESS RETAINS ONE AUTHORITATIVE
    ORCHESTRATOR for dispatch, retries, reconciliation, workspace isolation, and explicit state
    transitions... Kilo removed a dedicated USER-SELECTED Orchestrator mode because ordinary agents
    gained native delegation; ORCHESTRATION MOVED INTO THE PRIMARY AGENT RATHER THAN DISAPPEARING.
    The resulting correction is NARROWER THAN DELETING ELAI's FSM."
  RESOLUTION: keep the FSM as a commitment/audit envelope; make the interior adaptive; BENCHMARK the
  9-state design against a smaller objective-driven envelope under the same model (:411). Unresolved
  empirically — and it is a KERNEL question: how much determinism is too much.

C4. FAIL-CLOSED EVERYWHERE, OR RISK-TIERED?
  Spec (:24): "fail-closed — missing evidence is a FAIL, never a SKIP". Doctrine, corpus-wide.
  2026-07-10 (:249-259): "Gate proliferation can make the safe system unusable... The doctrine should
    NOT be 'everything fails closed.' It should be: high-risk, irreversible, security, privacy,
    finance, credential, and publication boundaries fail closed; quality and productivity checks can
    quarantine, retry, request evidence, or remain advisory according to measured risk."
    With an economic argument from OpenAI: "for high-throughput, low-risk changes: waiting can cost
    more than cheap correction, so it uses minimal blocking merge gates while mechanically enforcing
    a small set of boundaries."
  NOT A CONTRADICTION IN THE CORPUS'S OWN TERMS: K already specifies the fix. "K explicitly says
  false-positive friction killed prior gate generations and adds a whole-system gate-tax ceiling plus
  risk-tiered gating (K-SUMMARY_INDEX.md:101-112, :180-215). That is the correct diagnosis." The
  recommendation is SEQUENCING: "ELAI's K-5 risk tiers should be FOUNDATIONAL" (:259).

C5. THE AGENTLESS NUMBER. Survey 27.33% / $0.34 (v1) vs E-21/E-46 32.00% / $0.70 (v2). See 2.1.
  Version skew, not a real disagreement, but the survey's "27.33% is the peer-reviewed number"
  (:188) is likely stale. UNRESOLVED: the cost delta.

C6. THE SWE-EFFI SCAFFOLD. Survey's verdict paragraph says Agentless (:11); its entry body says
  SWE-Agent (:91). Internal, same file. See 3.3. AUTHORITATIVE: the entry body.

C7. THE 97.6% / 6-of-497 ARITHMETIC. Internal, same file, same sentence (:11). See 3.1.

C8. IS "TRUST" ELAI's FIRST PRODUCT, OR IS THE PROOF?
  K's own doctrine (quoted at :276): "K says THE NUMBER IS THE PRODUCT and other work should wait
  behind proof/distribution (K-SUMMARY_INDEX.md:588-598). THE ROADMAP SHOULD OBEY THAT STATEMENT."
  Meanwhile O and P "explicitly add surfaces rather than proof-spine numbers" (:276) and P has 27
  unaudited files (elai_status_query, this session) — the largest unaudited letter in the tree.
  The proof-spine rule exists (.claude/rules/proof-spine.md, PSR-1 precedence) and P-17..P-22 were
  authored 2026-07-11 with most tagged "spine none (discipline)"
  (openclaw-vs-elai-comparison-2026-07-11.md:192).
  This is a live tension between the corpus's stated sequencing rule and its actual authoring, and it
  is adjacent to the kernel question: a kernel program is a PROOF program (gap 1 / gap 4 / the
  three-arm experiment), not a surface program.


====================================================================
9. STALENESS + DEAD PATHS
====================================================================

DEAD WINDOWS PATHS, counted this session (live root is /home/user/teikoku/elai):
  116 refs  workflow/reasoning/HARNESS-INDEX-LITEGAME.md      (every row is D:\.liteGAME_workflow\)
   44 refs  workflow/reasoning/HARNESS-INDEX-ELAI.md          (every row is F:\.ELAI_workflow\)
    3 refs  workflow/reasoning/central-workflow-harness-audit.md
    1 ref   workflow/reasoning/openclaw-vs-elai-comparison-2026-07-11.md (the openclaw clone path)
    0 refs  harness-as-pipe-prior-art / elai-harness-market-evaluation / life-harness-source-interface
The two HARNESS-INDEX files are UNUSABLE AS PATH REFERENCES. Every row points at F:\ or D:\. They
remain usable as CLASSIFICATION material (the HOOKABLE/JUDGMENT/REFERENCE taxonomy and the
gate inventory are path-independent and still correct in shape).
NOTE: the ELAI CLAUDE.md itself still declares "F:\.ELAI_workflow\ is the workflow/tooling root" and
every path in its "Project PATHs" block is a Windows path. The project-boundary HARD RULE names a
directory that does not exist on this machine. Out of scope for this sweep; flagged because it makes
every path in the two index files consistent-but-dead rather than wrong-but-fixable.

LIVE-PATH VERIFICATIONS DONE THIS SESSION (all present):
  .elai_cc/crates/orchestrator/src/agentless_shortcut.rs                        PRESENT (read)
  .elai_cc/crates/contracts/tests/planning_decomposition_e46_contract.rs        PRESENT
  docs/.source/life-harness                                                     PRESENT
  docs/.source/openclaw                                                         PRESENT
  docs/specs/guide/steps/phase-index-MASTER.txt                                 PRESENT (81,373 b)
  workflow/papers/agentless-2407.01489-interface.md                             PRESENT (read)
  workflow/papers/life-harness-source-interface.md                              PRESENT (read)
  *intervention_harness* under .elai_cc                                          ABSENT (J-12 planned)

CONTENT STALENESS, by doc:
  harness-as-pipe (2026-06-14) — 1 month old. Its own OPEN QUESTIONS block (:183-193) lists EIGHT
    self-flagged staleness risks; they remain unresolved. Most important: SWE-Effi figure is a search
    snippet (:185); mini-SWE-agent small-model claim is extrapolated (:186); SynCode 96.07% is
    abstract-only (:187); Agentless 40.7/50.8 are README not peer-reviewed (:188); LangChain
    "30th to 5th" unverifiable (:189); Outlines not fetched (:190); "harness engineering" attribution
    to Mitchell Hashimoto not fetched (:191); no SWE-bench-saturation paper fetched (:192); VerMCTS
    numbers not extracted (:193).
    NOW SUPERSEDED IN ONE PLACE: :9 calls harness engineering "a growing named discipline"; by
    2026-07-10 it is an OpenAI engineering post title (openai.com/index/harness-engineering/,
    market-eval :219). The discipline got named by a frontier lab.
  competitive twins (2026-06-04) — 6 weeks old, and the market eval says the market "changes weekly"
    (:673). Specifically stale: the local-model recommendation (P3: pin Qwen3-Coder-30B / DeepSeek-
    Coder-V3-distill) was NOT what shipped — user memory + git 8bc668b show a Qwen3.5-4B llama.cpp
    Vulkan lane at :8181. The 2026-06-04 GROUNDED file's §5 is now historically interesting, not
    actionable.
  market eval (2026-07-10) — 1 week. Its own caveat (:673): "The meaningful caveat is market
    velocity: preview/experimental labels, product availability, model support, and provider pricing
    change quickly... The architectural conclusions — narrow waist, risk-proportional orchestration,
    isolated side effects, independent evidence, and same-model ablation — are substantially less
    time-sensitive." Roo Code died between the June and July passes (:196-198); assume more churn.
  openclaw (2026-07-11) — 6 days. Clone pinned at HEAD 2026-07-10 v2026.6.11. Doc-derived inventories
    with line-level claims marked UNVERIFIED-in-source (:7). Its own status caveat (:194): "Status
    claims about ELAI's own tree here are index-derived — confirm against live code
    (elai_status_query) before citing any as shipped."
  central-workflow-harness-audit (2026-06-19) — describes a MERGE that has since happened (TEIKOKU is
    live at /home/user/teikoku). Its 8 OPEN QUESTIONS (:210-218) may be answered by the shipped merge;
    unverified. Its CUT LIST (:182-194) may already be executed.

ADVISORY SNAPSHOT (elai_status_query, run this session, snapshot 25m old, git 8bc668b):
  OPEN 4 | lattice 246/344 audited units wired | 59 unaudited files | 136 registered-dark flags
  I: 54/54 wired (the only complete letter). J: 3/9 wired, 6 partial. L: 0/1. Q: 0/1.
  M(8) N(18) O(6) P(27) files unaudited.
  open gaps: I-33, I-23, glm-orchestrator dirt-baseline, H Track-3
  Per .claude/rules/elai-status-query.md this is an ADVISORY CACHE and discharges no done-gate.


====================================================================
10. WHAT I WOULD TELL THE OWNER
====================================================================

10.1 THE KERNEL THESIS SURVIVES, IN THE FORM THE SPEC ALREADY STATES.
  "The harness owns state, gates, and completion off evidence the model did not author"
  (ELAI-SPEC-OVERVIEW.md:591-593) survives every limit in section 3. It is a BOUNDARY claim.
  "The frozen model is a portable reasoning kernel that any harness can carry, and small models
  benefit disproportionately" does NOT survive — it is an INTERIOR claim, and every piece of prior
  art that tried to determinize the interior either needed a formal verifier (VerMCTS), lost its
  ceiling (Agentless multi-file), or found the model dominates anyway (SWE-Effi, SWE-bench
  leaderboard dissection, ALE's 3x).
  THE LINE IS BOUNDARY vs INTERIOR. Every successful instantiation in section 2 is a boundary
  mechanism: Agentless's slots, Life-Harness's H2 action gate, XGrammar's token mask, LLMCompiler's
  scheduler, Compiled AI's template. The market eval reached the same line independently (:236-239)
  and the July addendum's list of what a wrapper cannot carry (:844-857) is a list of INTERIOR
  functions. Three independent derivations of one boundary. That is the sweep's strongest result.

10.2 THE HARDEST LIMIT STANDING IS CONSTRAINT DECAY, NOT SCAFFOLD-MODEL SYNERGY.
  I would rank it above the synergy result the owner named, on evidence quality:
  - constraint decay (2605.06445): 100 tasks, 8 frameworks, and a BUILT-IN CONTROL — a 100-line
    harness and a full-featured harness decaying IDENTICALLY. That is a clean experiment saying
    harness investment has zero marginal return under constraint accumulation. Nothing in our corpus
    answers it.
  - scaffold-model synergy (2509.09853): a SEARCH-SNIPPET figure, with the scaffold MISATTRIBUTED in
    our own doc's verdict paragraph, with an unexpanded metric (EuTB), flagged by its own author as
    needing methodology verification before quantitative use (:185).
  The scarier-sounding limit is the weaker-evidenced one. Verify SWE-Effi before letting it decide
  anything; treat constraint decay as the real wall.

10.3 THE ORPHANED PARENTHETICAL IS THE MOST IMPORTANT NUMBER IN THE BUNDLE.
  "model accounts for ~3x the pass-rate spread of the harness"
  (small-models-long-horizon-findings.md:124, ALE agents-last-exam.org/blogs/harness-matters).
  It is the only direct quantification of harness-variance vs model-variance anywhere in the corpus —
  precisely the measurement the survey calls unstudied whitespace (gap 1). It caps the kernel's
  upside at ~25% of observed spread. It is in a source list, in an uncited file, from a practitioner
  blog. VERIFY IT FIRST. If it holds, it is the honest ceiling on this entire program, and it should
  be at the top of the kernel argument rather than in a parenthetical.

10.4 THE BEST VERSION OF THE THESIS IS ALREADY WRITTEN, IN THE WRONG FILE.
  small-models-long-horizon-findings.md:104-110 — "shortening the EFFECTIVE HORIZON so
  n-per-unsupervised-chain stays small: aggressive decomposition, per-step verification, and cheap
  rollback/recovery after the first error (before self-conditioning spirals)."
  That framing (section 5) survives every limit, explains WHY Agentless is single-file and why that
  is correct, predicts constraint decay rather than contradicting it, reconciles Kimi-Dev's ceiling
  claim, and is measurable. It is a better statement of the kernel thesis than the kernel survey's
  own verdict. It is orphaned, uncited, and one file away from the survey that needed it. If one
  thing comes out of this sweep, make it this: the harness is a HORIZON-SHORTENING AND ERROR-RECOVERY
  DEVICE, and its value scales with the horizon it removes and the model's self-conditioning
  severity. (MY SYNTHESIS, spanning two docs that do not cite each other. UNVERIFIED as a claim about
  the world; it is a hypothesis the corpus's own evidence supports.)

10.5 ELAI ALREADY DID THE RIGHT THING WITH AGENTLESS, AND SHOULD SAY SO.
  E-46's shortcut gates on single-file/single-symbol/no-design-tokens/no-cross-language and falls
  back to decomposition with the failed trace attached. It absorbed the technique AND its ceiling as
  a routing predicate — the correct engineering response to a limit the survey documented months
  later. It is shipped and contract-tested. This is a genuine, verifiable, differentiated design
  decision and nothing in the competitive docs mentions it.

10.6 THE THREE-ARM EXPERIMENT IS THE WHOLE ANSWER AND IT IS ALREADY DESIGNED.
  elai-harness-market-evaluation-2026-07-10.md:882-905: (1) competitor unmodified, (2) competitor
  under `elai wrap`, (3) native ELAI on the same model; hold everything constant; measure verified
  completion, escaped defects, kill rate, reviewer minutes, recovery, cost, false blocks.
  "This separates two causal questions: how much value comes from the PORTABLE TRUST KERNEL ALONE;
  how much additional value comes from ELAI's NATIVE reasoning and orchestration."
  That IS the owner's question. It is also prior-art gap 1 + gap 4 in different clothing. Two
  independent passes, a month apart, from opposite directions, designed the same experiment and
  neither cites the other. Nothing else in this sweep will settle the kernel question; this would.
  And per Anthropic's infrastructure-noise result (6pp from infrastructure alone, :685), ELAI's
  determinism substrate is one of the few harnesses that could run this experiment honestly. The
  wedge and the experiment are the same asset.

10.7 THE PROCEDURAL FINDING: RESEARCH WITHOUT PAPER-TO-PLAN EVAPORATES.
  Two 2026-06-14 research docs — 14 searches / 18 fetches, and a grounded long-horizon synthesis with
  5 primary papers — produced ZERO plan units and ZERO citations in 5 weeks. The 2026-07-11 OpenClaw
  comparison produced SIX (P-17..P-22, all plan_gate_check PASS) in one session. The difference is
  that paper-to-plan fired. The corpus has the skill; it was not run. If the kernel question matters,
  the survey's gap list (section 4) is five plan units waiting to be authored, and RIG (12.2%/53.9%,
  validating ELAI's largest built subsystem) is a citation waiting to be made.


====================================================================
11. QUOTE-CHECK SUMMARY (the owner's three, verdicts only)
====================================================================

Q1  "Agentless patches 97.6% single-file; 6 of 497 touched >1 file, arXiv:2509.13941"
    VERDICT: quoted our doc VERBATIM AND CORRECTLY (harness-as-pipe:11). The DOC is internally
    inconsistent: 6/497 = 1.21% cross-file = 98.79% single-file, not 97.6%. The doc's second
    statement (:125) drops the percentage and keeps the count. USE the count, DROP the percentage
    pending a re-fetch of 2509.13941. The limit is unaffected and is slightly HARDER than quoted.

Q2  "constraint decay (~30-point drop, arXiv:2605.06445)"
    VERDICT: EXACT. Consistent across :11, :86, :124. UNDERSELLS it — the full finding adds "top
    performers lose 40% of baseline", "-19.3pp for database constraints alone", "8.3% full compliance
    at L3", and the control that matters most: "Mini-SWE-Agent and OpenHands both exhibit IDENTICAL
    decay trends despite radically different scaffold complexity."

Q3  "scaffold-model synergy (Agentless + GPT-4o-mini 5.1% vs 21.8% with Qwen3-32B, SWE-Effi
    arXiv:2509.09853)"
    VERDICT: MISATTRIBUTED — the scaffold is SWE-AGENT, not Agentless — but the misattribution is
    OURS: our doc's verdict paragraph (:11) says Agentless while its own entry body (:91) says
    SWE-Agent. The owner inherited our error. Corrected: "the SWE-Agent scaffold scores EuTB 21.8%
    with Qwen3-32B and 5.1% with GPT-4o-mini; Agentless separately shows stability at lower peak
    capability." AND the doc's own caveat (:185) says this figure came from a SEARCH SNIPPET, not the
    paper, and must have its methodology verified before quantitative use. EuTB is never expanded.
    This number should not be load-bearing until re-fetched — and it currently carries the T3 verdict.


====================================================================
12. RECON_NEEDED (the exact check that settles each)
====================================================================

R1  SWE-Effi arXiv:2509.09853 — fetch the paper. Settle: (a) which scaffold produced 21.8%/5.1%,
    (b) what EuTB expands to and how it is computed, (c) the Agentless arm's actual numbers.
    WHY: it is the load-bearing anti-portability figure and it is currently a search snippet with a
    disputed subject inside our own file.
R2  ALE "model accounts for ~3x the pass-rate spread of the harness" —
    WebFetch agents-last-exam.org/blogs/harness-matters. Settle: the methodology and whether ~3x is
    measured or estimated. WHY: it is the only direct harness-vs-model variance number in the corpus
    and it caps the program's upside.
R3  arXiv:2509.13941 — fetch. Settle the 97.6%-vs-6/497 arithmetic and the 18.3%/25.3% multi-file
    comparison. WHY: the cross-file ceiling is the kernel's domain boundary.
R4  arXiv:2407.01489 v1-vs-v2 — check the version history. Settle: 27.33%/$0.34 vs 32.00%/$0.70, and
    which is current. WHY: two plan files and one research file disagree, and E-46 ships against one
    of them.
R5  arXiv:2603.25723 (NLAH) — fetch. Settle what the regressing "verifier harness component" actually
    was (-0.8% SWE-bench, -8.4% OSWorld). WHY: it is the only evidence in the corpus that ELAI's
    exact wedge (verifier-owned completion) can REGRESS a system, and nothing addresses it.
R6  arXiv:2601.10112 (RIG) — no plan cites it. Settle whether its 12.2%/53.9% is a citable external
    validation of ELAI's Phase-1 knowledge engine. WHY: cheapest available external validation of the
    largest built subsystem.
R7  central-workflow-harness-audit's 8 open questions (:210-218) — check against the shipped TEIKOKU
    merge at /home/user/teikoku. WHY: the audit predates the merge; its questions may be answered and
    its CUT LIST may be executed.
R8  J-12 status — confirm PLANNED via the live wire checkers, not the advisory snapshot. WHY: J-12 is
    the kernel thesis's own shipping vehicle and its status decides whether the thesis has a body.

END
