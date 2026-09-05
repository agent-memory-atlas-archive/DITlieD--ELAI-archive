Algorithm: FunSearch (Searching in Function Space)
Publication: Nature, Vol 625, pp. 468-475, January 2024 (online December 14, 2023)
DOI: 10.1038/s41586-023-06924-6
Authors: Romera-Paredes, Barekatain, Novikov, Balog, Kumar, Dupont, Ruiz, Fawzi et al.
PMC: PMC10794145
No arXiv preprint. Canonical open-access: https://pmc.ncbi.nlm.nih.gov/articles/PMC10794145/
Source code: https://github.com/google-deepmind/funsearch (Apache 2.0)
Scope tag: D-15_novelty_class_labeling

Inputs:
problem_specification | Python code | N/A | Must contain: (a) evaluate(solution) -> numeric score, (b) initial implementation of function to evolve (can be trivial), (c) optional skeleton with boilerplate/known structure
llm | pretrained code LLM | N/A | Paper uses PaLM 2; updated implementation uses Gemini 1.5 Flash; must support code completion/mutation
k_programs_per_prompt | int | scalar | Number of programs sampled per prompt; paper: k=2 (Methods section)
num_islands | int | scalar | Paper experiments: 10 islands found optimal (range 1-15 tested); balances diversity vs per-island iteration count
reset_period | duration | seconds | Paper: every 4 hours, discard worst m/2 islands
T0 | float | scalar | Initial cluster-selection temperature; values in Supplementary Appendix E.1 (not publicly extracted)
N | int | scalar | Period length for temperature decay; Supplementary Appendix E.1

Outputs:
program_pool | Python functions | list of (score, code_str) | Programs ranked by evaluate() score; higher score = better; no novelty label applied by system
best_program | Python function | single code_str | Highest-scoring program across all islands at termination
scores_per_test | dict | {test_input_id: float} | Per-test scores for each program; defines the program's cluster signature

Hyperparameters:
k (programs per prompt) | 2 | 1..N | Paper explicitly states "we typically set k=2" (Methods); higher k increases prompt length and diversity
num_islands | 10 | 1..15 | 10 found optimal empirically; more islands = more diversity paths; fewer = more iterations per island
reset_period | 4 hours | N/A | Fixed in paper; resets worst m/2 islands every 4h; seeds reset islands from best surviving island's top program
T_cluster (initial T0) | Appendix E.1 | >0 | Controls cluster sampling; formula: T_cluster = T0 * (1 - n_mod/N) where n_mod = num_programs mod N; decays linearly within each period
T_program | 1.0 | >0 | Within-cluster program sampling temperature; source code (programs_database.py): softmax over negative normalized lengths; favors shorter programs
cluster_scoring | mean(scores_per_test) | N/A | Cluster score = aggregate of per-test scores; softmax sampling: P_i = exp(s_i/T_cluster) / sum(exp(s_i'/T_cluster))

Critical invariants (testable assertions for D-15):
assert evaluate(program_output) returns int or float, not None  # evaluator.py line enforces this; non-numeric = program discarded
assert signature == tuple(scores_per_test[k] for k in sorted(scores_per_test.keys()))  # cluster key is full score tuple, not a scalar
assert len(surviving_islands) == num_islands // 2  # exactly half survive each reset cycle
assert new_cluster_program is None or len(new_cluster_program) <= len(existing_cluster_program)  # within-cluster, shorter programs preferred (softmax over -lengths)
assert score_comparison_is_against_published_baseline  # novelty NEVER asserted by program alone; requires external benchmark comparison

Novelty assessment (CRITICAL for D-15):
FunSearch has NO automated novelty classifier. There is no "novelty class" label in the system.
The paper's novelty determination uses THREE external mechanisms, none programmatic:
  1. Benchmark surpassing: "Surpassing state-of-the-art results on established open problems provides a clear indication that the discoveries are truly new, as opposed to being retrieved from the LLM's training data." (Methods, PMC10794145)
  2. Expert validation: Mathematician Jordan Ellenberg confirmed non-obviousness and "actionable insights"; expert review is mandatory for mathematical novelty claims
  3. Literature/database comparison: Results cross-checked against OEIS and published mathematical bounds (e.g., cap set size 512 vs previous best 496 in dimension 8)

The exact phrase "fundamentally new mathematical content" does NOT appear in the paper. The paper uses: "new scientific results for established open problems", "verifiably new", "truly new" (contrasted with LLM retrieval from training data).

Novelty signal hierarchy used by authors (strongest to weakest):
  Level 1: Discovered program output strictly exceeds best-known result in literature (e.g., cap set size 512 > 496; largest improvement in 20 years)
  Level 2: Asymptotic bound improvement verified mathematically (e.g., cap set capacity 2.2202 > 2.2180)
  Level 3: Expert confirms structural/algorithmic insight not in prior literature (e.g., cyclic permutation structure in admissible sets)
  Level 4: Program implements known technique in measurably superior way (derivative, not novel)
  Level 0: Program output matches or falls below existing baselines (not novel)

Derivative vs novel distinction (for D-15 labeling ground truth):
  Novel: output exceeds all published baselines AND expert confirms mechanism is new (Level 1+3 or Level 2+3)
  Derivative: output exceeds baseline but mechanism is known; improvement from better search, not new ideas (Level 1 or 2 alone)
  Known: output matches or is subsumed by existing literature; LLM retrieved rather than discovered
  The authors acknowledge: "a program might be new but implement known techniques, or implement familiar techniques in genuinely novel combinations" -- this is the novel/derivative ambiguity

Complexity:
Time per evaluation: O(evaluate_function_complexity) -- problem-specific; must be efficient (paper constraint: "efficient evaluate function")
Total LLM samples: ~10^6 across all experiments (distributed); few samplers on GPU, many evaluators on CPU
Evaluators in paper deployment: 150 CPU evaluators, 15 samplers
Space per island: O(num_clusters * max_programs_per_cluster * program_length)
Parallelism: embarrassingly parallel evaluators; samplers are the bottleneck; architecture: programs_database (shared) + samplers (GPU) + evaluators (CPU)

Failure modes (paper-stated):
Stochasticity: cap set n=8 direct construction succeeds in only 4/140 runs (2.9%); requires multiple independent runs
Partial success: admissible sets I(12,7) succeeds in 60% of experiments
Evaluator requirement: algorithm degrades to inapplicable on problems WITHOUT an efficient computable evaluate function (e.g., open theorem proving)
Skeleton collapse: if skeleton is too rigid, LLM has no room to mutate; if too loose, search space is exponential
Island stagnation: without periodic reset, islands converge prematurely; reset period is a sensitive hyperparameter
LLM hallucination: system guards against hallucination via evaluator rejection; programs that fail execution or return wrong types are discarded silently

Implementation pitfalls (do NOT do this):
Do NOT use program output score alone as novelty signal -- score > baseline is necessary but not sufficient for novelty claim; expert review is mandatory
Do NOT treat "program is new code" as equivalent to "mathematical result is novel" -- the program text can be new while implementing known mathematics
Do NOT apply a single novelty threshold score -- novelty is relative to published baselines, which differ per problem domain
Do NOT skip the signature-based clustering -- without it, identical-performing programs dilute the pool without adding diversity
Do NOT use a global temperature for both cluster and program selection -- they use separate T_cluster (Boltzmann on cluster scores) and T_program (softmax on negative lengths) with different decay schedules
Do NOT merge islands before reset -- island independence is the core diversity mechanism; merging collapses it
Do NOT assume the LLM generates novel programs -- the LLM generates mutations; novelty emerges from evolutionary pressure + evaluator, not from LLM alone
Do NOT label a program "novel" based on score improvement alone when the improvement is within noise/stochasticity range -- paper shows high variance per run

D-15 specific grounding notes:
FunSearch does not define novelty-class labels programmatically. D-15's novelty-class labeling system must define its own criteria. The FunSearch paper provides the epistemological ground for what "novel" means in program-search-for-mathematical-discovery:
  - Novel = surpasses published best-known result (quantitative) AND mechanism confirmed non-obvious by domain expert (qualitative)
  - Derivative = surpasses baseline through search efficiency, not structural insight
  - Known = matches or rediscovers existing results
Any automated novelty labeler for D-15 should treat these as a 3-class output, not binary, and must include a "requires expert review" flag for Level 1/2 detections (score beats baseline) that lack Level 3 (expert confirmation).

Sources verified:
- PMC full text: https://pmc.ncbi.nlm.nih.gov/articles/PMC10794145/
- GitHub source (evaluator.py, programs_database.py): https://github.com/google-deepmind/funsearch
- DeepMind blog: https://deepmind.google/blog/funsearch-making-new-discoveries-in-mathematical-sciences-using-large-language-models/
- NYU critique (Davis): https://cs.nyu.edu/~davise/papers/FunSearchComment.pdf
- PubMed record: https://pubmed.ncbi.nlm.nih.gov/38096900/
