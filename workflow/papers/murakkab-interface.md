Algorithm: Murakkab: Resource-Efficient Agentic Workflow Orchestration in Cloud Platforms
ArXiv: 2508.18298
Paper: Murakkab: Resource-Efficient Agentic Workflow Orchestration in Cloud Platforms

Inputs:
workflow_spec | declarative | logical tasks and dependencies | mandatory
slo_requirements | dict | quality/latency/cost targets | mandatory
resource_constraints | dict | GPU/CPU/memory limits | mandatory
workload_profiles | dict | arrival patterns and token distributions | mandatory

Outputs:
optimized_configuration | deployment_plan | model instances and assignments | invariant: meets all SLOs
resource_allocation | dict | GPU counts and workflow mappings | invariant: respects resource constraints
executable_workflow | DAG | parameterized workflow graph | invariant: topologically ordered
performance_metrics | dict | energy/cost/latency measurements | invariant: matches optimization objectives

Hyperparameters:
optimization_epoch | 60min | reoptimization interval | range: [30, 120] | sensitivity: medium
slo_tiers | 4 | quality/latency levels (best/good/fair/basic) | range: [3, 5] | sensitivity: high
autoscaling_window | 300s | load monitoring window | range: [60, 600] | sensitivity: medium
multiplexing_factor | 0.8 | colocation efficiency target | range: [0.5, 0.95] | sensitivity: low

Critical invariants (testable assertions):
assert all(slo_met[slo] <= target[slo] for slo, target in slo_requirements.items())
assert sum(gpu_counts.values()) <= resource_constraints['max_gpus']
assert is_topologically_sorted(executable_workflow)
assert energy_consumption <= baseline_energy * 0.5  # 2x improvement minimum
assert cost_savings >= 0.0  # positive cost reduction

Complexity:
Time: O(MILP_optimization + workflow_composition) per epoch
Space: O(workflow_profiles + model_profiles + deployment_plan)
Parallelism: MILP optimization parallel, workflow execution distributed

Failure modes:
MILP optimization infeasibility
SLO violation during autoscaling
Resource contention during peak load
Workflow DAG cycle detection failure
Model instance overload

Implementation pitfalls (do NOT do this):
Ignore token distribution variance in profiles
Skip SLO feasibility checking before deployment
Use static resource allocation without autoscaling
Allow circular dependencies in workflow DAG
Optimize for single objective without multi-workload coordination
