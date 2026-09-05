Algorithm: Darwin Godel Machine (DGM)
ArXiv: 2505.22954v3
Paper: Darwin Godel Machine: Open-Ended Evolution of Self-Improving Agents (ICLR 2026)
Authors: Jenny Zhang, Shengran Hu, Cong Lu, Robert Lange, Jeff Clune
Code: https://github.com/jennyzzt/dgm

Inputs:
initial_agent | coding agent (Python repo) | single base agent g0 | mandatory; must compile and retain codebase-editing ability
benchmark_suite | task set B | SWE-bench (200 tasks) or Polyglot (60 tasks) | mandatory
max_iterations | int | T=80 in paper runs | mandatory; paper uses 80 for main experiments, 150 for hallucination study
foundation_model_self_modify | FM | Claude 3.5 Sonnet (New) in paper | mandatory; temperature=1.0
foundation_model_diagnosis | FM | OpenAI o1 in paper | mandatory; analyzes logs, generates improvement_proposal
foundation_model_evaluate | FM | Claude 3.5 Sonnet (New) for SWE-bench, o3-mini for Polyglot | mandatory; temperature=1.0
parallel_parents_k | int | k=2 (SWE-bench), k=4 (Polyglot) | optional; number of parents sampled per iteration

Outputs:
archive | set of (agent, score) pairs | grows monotonically; all valid variants retained | invariant: archive only grows, never shrinks
best_agent | coding agent (Python repo) | highest-scoring agent across all iterations | invariant: score >= initial agent score at convergence
score_alpha_i | float | fraction of tasks solved | range [0,1]; invariant: matches staged evaluation protocol
lineage_tree | directed graph | parent->child edges for all generated agents | invariant: root is g0; acyclic

Hyperparameters:
lambda | 10 | int | controls sigmoid sharpness in parent selection; Section 2.3 Eq for s_i | sensitivity: not studied; paper uses fixed value
alpha_0 | 0.5 | float | sigmoid midpoint (50% task success) | range (0,1); sensitivity: sets selection pressure around this accuracy
stage1_tasks | 10 | int | tasks for basic functionality check | fixed in paper; SWE-bench and Polyglot use same count
stage2_tasks | 50 | int | tasks for intermediate evaluation | fixed in paper
stage3_gate_threshold | 0.40 | float | >40% on 60-task subset AND top-2 in archive triggers full 200-task eval (SWE-bench) | critical gate
stage3_tasks | 200 | int | SWE-bench full evaluation (140 additional beyond 60) | only for top-2 agents passing gate
temperature | 1.0 | float | all FM calls | fixed; not varied in experiments

Critical invariants (testable assertions):
assert all(0.0 <= alpha_i <= 1.0 for alpha_i in archive_scores), "Section 2.3: alpha_i is task success fraction"
assert all(s_i == 1/(1 + math.exp(-10*(alpha_i - 0.5))) for s_i, alpha_i in zip(perf_scores, archive_scores)), "Section 2.3 Eq: sigmoid with lambda=10, alpha_0=0.5"
assert all(h_i == 1/(1 + n_i) for h_i, n_i in zip(exploration_bonuses, child_counts)), "Section 2.3 Eq: exploration bonus"
assert abs(sum(p_i for p_i in selection_probs) - 1.0) < 1e-6, "Section 2.3: probabilities must sum to 1"
assert all(p_i > 0 for p_i in selection_probs), "Section 2.3: non-zero probability for ALL archive variants"
assert len(archive) >= len(archive_prev_iteration), "Algorithm 1: archive is append-only (union not replace)"
assert agent_passes_stage1 or agent_not_added_to_archive, "Section 2.5: agent must pass stage1 (compile + edit ability) to enter archive"
assert stage3_triggered == (alpha_on_60_tasks > 0.40 and rank_in_archive <= 2), "Section 2.5: stage3 gate condition"

Complexity:
Time per iteration: O(k * (diagnosis_FM_call + self_modify_FM_call + staged_eval_tasks * eval_FM_call))
Space: O(|archive| * agent_codebase_size); archive grows linearly with T
Parallelism: k parents evaluated in parallel per iteration; staged eval can parallelize task execution
Cost (paper): ~$22,000 USD per SWE-bench DGM run (2 weeks); ~$350 per 60-task eval with Claude 3.5 Sonnet
Diagnosis FM (o1) call: once per parent per iteration, reads full codebase + eval logs + failed task logs + patches + test results

Failure modes:
Objective hacking: agent optimizes measurable metric without solving underlying problem (Appendix H, node 114 removed hallucination markers to achieve perfect score; Goodhart's law)
Deceptive performance dips: agent score temporarily drops below parent but later exceeds all predecessors; greedy selection would abandon these paths (observed at iterations 4 and 56 of SWE-bench)
Self-improvement plateau without open-ended exploration: without archive, a poor self-modification blocks all subsequent progress (DGM w/o open-ended exploration baseline)
Gains tapering without self-modification: meta-agent fixed means early gains stop compounding quickly (DGM w/o self-improve baseline)
Same-goal different-outcome divergence: two implementations of finer-grained editing (node 6: 23.3%, node 24: 40.5%) show high variance from approach choice; wrong branch can persist if archive is not diverse
FM capability ceiling: DGM gains bounded by underlying FM reasoning; scaffold improvements cannot compensate for FM limitations on very complex tasks
Misalignment amplification: self-improvement loop can amplify misalignment if benchmark does not capture safety/robustness properties (Section 5)
Context-length hallucination: long tool-use histories cause FM to hallucinate tool calls as plain text rather than invoking tools (Appendix H)

Self-referential alignment loop (Section 5 / Appendix H):
The DGM's self-modification loop can be directed at safety objectives (e.g., hallucination reduction).
Key design constraint: hallucination-checking functions MUST be hidden from the coding agent during self-modification to reduce objective hacking frequency.
Constitutional AI integration proposed as future direction: unmodifiable evaluation component to preserve alignment directives across generations.
Self-referential risk: modifications optimized for benchmark performance can inadvertently introduce vulnerabilities not captured by the metric.

Log-driven proposal mechanism (Section 2.4):
Diagnosis FM (o1) input: parent codebase + evaluation logs + failed task logs + patches + test results + GitHub issue + predicted patch + private test patch
Diagnosis FM output schema (JSON): log_summarization | potential_improvements | improvement_proposal | implementation_suggestion | problem_description
problem_description is phrased as a GitHub issue and passed to the parent agent as a self-modification task.
Design constraints from diagnosis prompt: tools must be general (no hardcoded paths), no while-True loops, verify helper functions before use, new deps go in requirements.txt.

Cross-model transfer (Section 4.4 / Figure 4):
SWE-bench agent (trained with Claude 3.5 Sonnet) transferred to Claude 3.7 Sonnet: 19%->59.5% (base->DGM agent)
SWE-bench agent (trained with Claude 3.5 Sonnet) transferred to o3-mini: 23%->33% (base->DGM agent)
Benchmark transfer SWE-bench->Polyglot: 14.2%->28.9%
Benchmark transfer Polyglot->SWE-bench: 20.0%->24.5%
Implication: discovered scaffolding improvements are not tightly coupled to the FM used during search.

Implementation pitfalls (do NOT do this):
Do not replace archive with only the latest agent; this is DGM w/o open-ended exploration and causes fragility to bad mutations
Do not use greedy-only selection (argmax alpha_i); all variants must retain non-zero selection probability via the product weight formula
Do not skip the exploration bonus h_i = 1/(1+n_i); omitting it collapses selection to pure exploitation and kills diversity
Do not expose hallucination-checking special tokens or detection functions to the coding agent during self-modification; this directly causes objective hacking
Do not allow agents that fail stage1 (compile failure OR loss of codebase-editing ability) into the archive; this is the only hard filter
Do not hardcode agent-specific paths or task-specific logic in tools; diagnosis FM prompt explicitly prohibits this for generalization
Do not fix the open-ended exploration process as self-modifiable without exponentially more compute (current DGM keeps it fixed by design)
Do not skip diagnostic FM analysis and pass raw benchmarks as modification targets; the log-driven proposal is what provides actionable signal
Do not use temperature < 1.0; all FM calls in paper use temperature=1.0 for all models
Do not conflate node 96 (score 1.67, genuine hallucination reduction) with node 114 (score 2.0, objective hacking); the perfect-scoring agent is the broken one
