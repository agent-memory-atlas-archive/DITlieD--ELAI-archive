Algorithm: AutoB2G: A Large Language Model-Driven Agentic Framework For Automated Building–Grid Co-Simulation
ArXiv: 2603.26005
Paper: AutoB2G: A Large Language Model-Driven Agentic Framework For Automated Building–Grid Co-Simulation

Inputs:
task_description | string | natural language simulation requirements | mandatory
citylearn_config | dict | building configuration parameters | mandatory
grid_parameters | dict | power system specifications | mandatory
dag_codebase | graph | module dependency graph G=(V,E) | mandatory

Outputs:
simulator_code | python | executable CityLearn + grid co-simulation | invariant: compiles and runs
performance_metrics | dict | building + grid performance indicators | invariant: contains voltage_admissibility, thermal_loading, n_1_resilience, short_circuit_current
dag_validated | bool | dependency graph validation result | invariant: True for executable code
execution_success | bool | simulation completion status | invariant: matches loss convergence

Hyperparameters:
T_DAG | 10 | maximum DAG repair rounds | range: [5, 20] | sensitivity: high
T_SOCIA | 15 | maximum SOCIA optimization rounds | range: [10, 30] | sensitivity: medium
loss_threshold | 0.0 | convergence tolerance | range: [0.0, 0.1] | sensitivity: low
voltage_min | 0.95 | minimum per-unit voltage | range: [0.9, 0.98] | sensitivity: high
voltage_max | 1.05 | maximum per-unit voltage | range: [1.02, 1.1] | sensitivity: high

Critical invariants (testable assertions):
assert 0.0 <= loss_t <= inf
assert 0.95 <= V_i <= 1.05 for all voltage buses
assert S_line_k <= S_k_max for all transmission lines
assert isinstance(dag_validated, bool) and dag_validated == True
assert T_DAG > 0 and T_SOCIA > 0

Complexity:
Time: O(T_DAG * LLM_retrieval + T_SOCIA * (LLM_codegen + execution + evaluation))
Space: O(DAG_nodes + simulator_code_size + execution_results)
Parallelism: DAG validation parallel, SOCIA sequential due to dependency

Failure modes:
DAG cycle detection failure
Module dependency resolution errors
Code generation syntax errors
Runtime simulation failures
Loss function non-convergence

Implementation pitfalls (do NOT do this):
Skip DAG topological ordering validation
Allow circular dependencies in module graph
Ignore voltage constraint violations during simulation
Bypass SOCIA loss convergence check
Use hardcoded grid parameters without validation
