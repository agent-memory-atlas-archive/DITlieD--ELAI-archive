Algorithm: ParallelResearch: Efficient Tree-Structured Deep Research with Adaptive Resource Allocation
ArXiv: 2510.05145
Paper: Efficient Tree-Structured Deep Research with Adaptive Resource Allocation

Inputs:
research_query | string | open-ended research question | mandatory
time_budget | seconds | maximum execution time | mandatory
information_gain_threshold | float | minimum IG for expansion | range: [0.1, 0.9] | sensitivity: high
quality_thresholds | dict | goal/quality min thresholds | mandatory

Outputs:
research_report | markdown | synthesized findings | invariant: addresses query
research_tree | structure | dynamic tree of subqueries | invariant: acyclic
performance_metrics | dict | latency/quality/throughput | invariant: meets time budget
execution_trace | log | planning/orchestration decisions | invariant: complete

Hyperparameters:
breadth_factor | 3 | parallel subqueries per node | range: [2, 5] | sensitivity: high
depth_limit | 5 | maximum tree depth | range: [3, 8] | sensitivity: medium
phi_min | 0.7 | goal satisfaction threshold | range: [0.5, 0.9] | sensitivity: high
psi_min | 0.6 | quality threshold | range: [0.4, 0.8] | sensitivity: high
speculative_depth | 2 |提前执行深度 | range: [1, 3] | sensitivity: medium

Critical invariants (testable assertions):
assert execution_time <= time_budget
assert 0.0 <= phi_i <= 1.0 for all nodes
assert 0.0 <= psi_i <= 1.0 for all nodes
assert is_acyclic(research_tree)
assert len(research_report) > 0

Complexity:
Time: O(tree_nodes * research_time_per_node) with parallel execution
Space: O(tree_size * context_per_node)
Parallelism: breadth (parallel subqueries), depth (speculative execution), orchestration (async monitoring)

Failure modes:
Information gain estimation errors
Quality threshold misconfiguration
Speculative execution resource contention
Tree structure cycle formation
Time budget exhaustion

Implementation pitfalls (do NOT do this):
Disable runtime orchestration monitoring
Use fixed breadth/depth without adaptation
Skip speculative execution optimization
Ignore tree pruning for low-quality branches
Hardcode quality thresholds without query adaptation
