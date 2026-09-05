Algorithm: Arbor - Hypothesis Tree Refinement (HTR) for Autonomous Research
ArXiv: 2606.11926
Paper: Toward Generalist Autonomous Research via Hypothesis-Tree Refinement
Submitted: 2026-06-10
Authors: Jiajie Jin, Yuyang Hu, Kai Qiu, Qi Dai, Chong Luo, Guanting Dong, Xiaoxi Li, Tong Zhao, Xiaolong Ma, Gongrui Zhang, Zhirong Wu, Bei Liu, Zhengyuan Yang, Linjie Li, Lijuan Wang, Hongjin Qian, Yutao Zhu, Zhicheng Dou
Affiliations: Renmin University of China (Gaoling AI), Microsoft Research
Source HTML: F:\.ELAI_workflow\.claude\papers\2606.11926.html (703 KB, full HTML)
Code: https://github.com/RUC-NLPIR/Arbor (Apache 2.0, depth-1 clone at F:\.ELAI_workflow\docs\.source\2606-11926-arbor-hypothesis-tree-refinement\)
Reliability: HIGH (full HTML read + repo confirmed)


ALGORITHM: Hypothesis Tree Refinement (HTR)

Inputs:
P (problem tuple) | tuple | (M_0, O, E_dev, E_test) | mandatory
  M_0 = initial artifact (codebase/model/pipeline)
  O = objective function (maps artifact to scalar score)
  E_dev = development evaluator (used during search iterations)
  E_test = held-out evaluator (used only for final admission gate)
B (budget) | int/float | total token or compute budget | mandatory, positive
k (branching factor) | int | number of child hypotheses per parent per ideation step | mandatory, k>=1, typically 2-3
coordinator_cycles | int | max coordinator loop iterations | default=20 | range 1-inf, paper uses 20
max_tree_depth | int | max depth from root in hypothesis tree | default=2 | range 1-inf, paper uses 2
backbone_model | str | LLM for coordinator and executor | default=Claude Opus 4.6 | also tested: GPT-5.5, Gemini-3-Flash


Outputs:
M_star | artifact (code/model/pipeline) | same type as M_0 | best artifact found; must satisfy O(E_test(M_star)) > O(E_test(M_0))
Tree | HypothesisTree | persistent tree structure | all nodes; persisted across coordinator cycles; never discarded mid-run
delta_test | float | normalized held-out improvement | = (S_test(M_star) - S_test(M_0)) / (|S_test(M_0)| + epsilon); must be >= 0 for any admitted candidate

Output Invariants:
M_star is only updated when O(E_test(candidate)) > O(E_test(M_best_prev))  [held-out gate, Section 4]
Tree root corresponds to M_0; every node represents one hypothesis
Executor cannot modify its assigned hypothesis mid-trial (hypothesis binding invariant)
Every admitted artifact has a corresponding git branch reference in tree node metadata


Node Structure (each tree node contains):
h_n | hypothesis | verifiable claim about artifact improvement | mandatory per node
iota_n | insight | reusable interpretation of evidence from this node's execution | populated post-execution
mu_n | metadata | {status, development_score, artifact_ref, git_branch} | mandatory


Six-Step Coordinator Loop (HTR Algorithm 1):
1. Observe: read current tree projection, frontier nodes, best artifact state
2. Ideate: select parent node; propose k child hypotheses conditioned on accumulated evidence
3. Select: choose pending leaf nodes for dispatch (frontier control policy)
4. Dispatch: send selected hypotheses to parallel executors in isolated git worktrees
5. Backpropagate: write executor evidence to leaf nodes; abstract insights upward along path to root
6. Decide: apply held-out merge gate; prune falsified subtrees


Executor Contract:
Input: (h_n, iota_anc(n), M_best, E_dev) where iota_anc(n) = ancestor insight chain
Process: create fresh git worktree from M_best; implement h_n; repair-only (hypothesis fixed); evaluate on E_dev; retry until success or cap
Output: (s_n=dev_score, r_n=factual_result, distilled_insight, commit_ref)
Hard constraint: executor cannot reassign hypothesis; repair only, hypothesis semantics preserved


Hyperparameters:
coordinator_cycles | 20 | 1-inf | low sensitivity (more cycles = more exploration, diminishing returns)
max_tree_depth | 2 | 1-inf | moderate sensitivity; depth-2 balances breadth and depth
k (branching factor) | 2-3 | 1-inf | moderate sensitivity; too high = wasted executor budget
backbone_model | Claude Opus 4.6 | any LLM | HIGH sensitivity; model-task fit matters (Claude stronger on reasoning, GPT-5.5 on ML knowledge)
epsilon (normalization) | small positive | >0 | low sensitivity; prevents div-by-zero in delta_test formula
wall_clock_limit | 48h | task-dependent | hard external constraint in evaluation setup


Critical Invariants (testable assertions):
assert O(E_test(M_star)) >= O(E_test(M_0))  # admitted artifact must improve on held-out
assert len(Tree.nodes) >= 1  # tree always has at least root node
assert all(node.hypothesis is not None for node in Tree.nodes)  # every node has a bound hypothesis
assert M_star in {artifact_ref for node in Tree.nodes if node.status == 'merged'}  # M_star comes from a merged node
assert delta_test == (S_test_star - S_test_0) / (abs(S_test_0) + epsilon)  # normalization formula Section 4
assert executor_output.hypothesis_id == dispatcher_input.hypothesis_id  # hypothesis binding preserved
assert all(node.status in {'pending','running','succeeded','failed','pruned'} for node in Tree.nodes)


Computational Complexity:
coordinator loop | O(coordinator_cycles * k) hypothesis proposals per run
executor parallelism | up to k executors running concurrently per coordinator cycle
token cost | 20.12M - 43.19M tokens total across 6 AO tasks (Figure 4 in paper)
comparable to single-trajectory baselines (Codex, Claude Code) per paper
git worktree overhead | one fresh worktree per executor dispatch; cleaned on completion
memory | O(depth * k^depth) tree nodes in worst case; bounded by max_tree_depth and budget B


Evaluation Protocol:
dev/test split | E_dev used during search; E_test held out for admission gate only
merge gate | candidate admitted only if O(E_test(b_n_dagger)) > O(E_test(M_best))
normalized improvement formula | delta_test = (S_test_star - S_test_0) / (|S_test_0| + epsilon)
reliability | Avg@3 with standard deviation reported
metric convention | relative improvement for non-percentage metrics; absolute pp change for accuracy/pass-rate


Main Results (Table 2, Table 3):
AO task improvements vs Codex and Claude Code: Arbor >2.5x average relative held-out gain
MLE-Bench Lite (GPT-5.5): 86.36% Any Medal, 77.27% Gold Medal, 100% valid submissions
MLE-Bench Lite (Claude Opus): 81.82% Any Medal baseline

Ablation (Table 4-5, MLE-Bench Lite, Claude Opus backbone):
full Arbor: 81.82% any-medal
w/o tree structure: 63.64% (-18.18 pp)
w/o insight feedback: 54.54% (-27.28 pp)
conclusion: insight feedback > tree structure alone; both required for full performance


Known Failure Modes (acknowledged in paper Section A / Appendix):
HTR struggles when progress requires fundamental problem re-framing (not local refinement)
scalar objective dependency: multi-objective problems underexplored
backbone model ceiling: final performance bounded by model-task compatibility
infrastructure cost: parallel executors require substantial compute
task design dependency: result quality depends on initial artifact + evaluator + metric design
dev-set overfitting: high dev / low test candidate indicates exploiting feedback signal (handled by held-out gate)


Implementation Pitfalls (do NOT do this):
do not use E_dev as the final admission gate (overfits to development signal)
do not allow executor to change its assigned hypothesis mid-trial (breaks evidence attribution)
do not skip insight backpropagation upward to root (ablation shows -27pp without it)
do not use BatchSpanProcessor equivalent for evidence collection (async flush causes silent fail-open)
do not conflate dev improvement with test improvement (held-out gate is the only valid admission criterion)
do not prune nodes solely on dev score without test gate verification
do not measure "any run that improved" without Avg@3 reliability (single-run results are noisy)


Transfer Results (Section 5.6):
cross-task transfer confirmed: BrowseComp-optimized harness gains on HLE +6pp, DeepSearchQA +8pp
conclusion: discovered improvements are generalizable, not task-specific overfit


AO Task Suite (6 tasks, Appendix C):
optimizer_design | NanoGPT-Bench Muon baseline | steps to target (minimize)
architecture_design | autoresearch LLM codebase | final loss (minimize)
terminal_bench_2 | official terminal-agent | pass rate (maximize) | 36 dev / 53 test
browsecomp | minimal ReAct harness | accuracy (maximize) | 50 dev / 300 test
search_agent_pipeline | hand-designed generation | pass@4-pass@1 gap (maximize)
math_reasoning | AIME problem generator | pass@4-pass@1 gap (maximize)


Contract Tests:

def test_held_out_gate_invariant():
    # M_star must improve over M_0 on test evaluator
    s0 = evaluate_test(M_0)
    s_star = evaluate_test(M_star)
    assert s_star >= s0, f"M_star ({s_star}) must be >= M_0 ({s0}) on held-out evaluator"

def test_hypothesis_binding():
    # executor output hypothesis id must match dispatch input
    for dispatch, result in zip(dispatched_hypotheses, executor_results):
        assert result.hypothesis_id == dispatch.hypothesis_id

def test_delta_test_formula():
    import math
    epsilon = 1e-9
    s0, s_star = 1.083, 1.028  # architecture design example from Table 2
    # for minimization, negate scores before formula
    delta = ((-s_star) - (-s0)) / (abs(-s0) + epsilon)
    assert delta > 0, "improvement should be positive for minimization task with lower loss"
    assert abs(delta - 0.0508) < 0.01  # approx 5% improvement

def test_tree_root_invariant():
    tree = HypothesisTree(initial_artifact=M_0)
    assert len(tree.nodes) >= 1
    assert tree.root.hypothesis is not None
    assert tree.root.metadata['artifact_ref'] == M_0

def test_insight_propagation_path():
    # after executor completes, insight must exist at leaf and at all ancestors up to root
    leaf = tree.get_node(leaf_id)
    assert leaf.insight is not None
    for ancestor in tree.ancestors(leaf_id):
        assert ancestor.insight is not None or ancestor.is_root
