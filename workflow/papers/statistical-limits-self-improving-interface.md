Algorithm: Two-Gate Self-Modification Guardrail (Capacity-Controlled PAC Self-Improvement)
ArXiv: 2510.04399
Paper: On The Statistical Limits of Self-Improving Agents
Authors: Charles L. Wang, Keir Dorchen, Peter Jin (Columbia University)
Submitted: October 5, 2025; revised February 1, 2026

Inputs:
training_set S | labeled dataset | size m | mandatory
validation_set V | held-out labeled dataset | size nv | mandatory
capacity_schedule K[m] | function of m | scalar ceiling on VC dim | mandatory; must grow sublinearly in m for oracle rate
margin tau | float | tau > 0 | controls per-edit risk improvement guarantee; mandatory
candidate_edit Z_new | modified hypothesis class or architecture | any of 5 axes | one per modification round

Outputs:
terminal_predictor h* | classifier/regressor | from accepted reachable family G_K[m] | satisfies oracle inequality
accepted_edit | boolean per round | True if both gates pass | invariant: accepted edits strictly decrease true risk by >= tau
risk_bound | float | R[h*] <= inf_{h in G_K[m]} R[h] + O~(sqrt(K[m]/m)) | distribution-free PAC guarantee

Hyperparameters:
K[m] | no single default; paper: must be finite at every m | (0, inf) | critical: if K[m] -> inf the guarantee collapses; sensitivity = high
tau | paper uses tau > 0 symbolically | (0, 1) | per-edit risk margin; higher = safer but fewer accepted edits
nv (validation size) | not fixed; paper notes standard VC concentration needs nv ~ K[m]/epsilon^2 | > 0 | sensitivity = moderate

Critical invariants (testable assertions):
assert vc_dim(reachable_family) <= K_m, "capacity gate: VC dim of reachable set must stay bounded by K[m]"
assert empirical_risk_improvement(V, h_new, h_old) >= 2*eps_v + tau, "validation gate: improvement must exceed concentration term plus margin"
assert all_accepted_edits_decrease_true_risk(trajectory), "Theorem 2: each accepted edit decreases true risk by >= tau (PAC cert)"
assert not (K_m == float('inf')), "Prop 6: unbounded capacity destroys distribution-free learnability"
assert sum(step_sizes) < STEP_MASS_CAP, "Theorem 11: cumulative SGD step-mass must be capped to preserve generalization gap bound"

Complexity:
Per-round time: O(|V| * model_eval) for risk gate + O(capacity_proxy_computation) for capacity gate
Space: O(m + nv) for datasets; O(1) for gate decisions
Parallelism: gates are independent per candidate edit; multiple candidates can be evaluated in parallel

Failure modes:
Utility-rational edits that pass validation gate but unboundedly expand VC dim -- paper's central failure scenario
Monotone risk improvement on V does not imply bounded capacity; both gates are independently necessary
Downgrading substrate to finite-state memory can destroy learnability even if problem was previously learnable (Prop 13)
Unbounded local representational power (Prop 6): capacity-bonus utility destroys distribution-free learnability even if each step looks safe
Algorithmic changes (optimizer swaps, schedule changes) cannot compensate for infinite VC dimension (Prop 9)

Five self-modification axes (learner state l_t = <A_t, H_t, Z_t, F_t, M_t>):
A: Algorithmic (update rules, optimizer)
H: Representational (hypothesis class, features)
Z: Architectural (topology, depth/width)
F: Substrate (computational model, memory semantics)
M: Metacognitive (scheduler approving modifications)
All axes reduce to the same capacity-supremum boundary (unified via Theorem 4 + 8)

Church-Turing invariance: switching among Turing-equivalent substrates preserves PAC learnability (Theorem 12); substrate choice is irrelevant to sample complexity

Implementation pitfalls (do NOT do this):
Do not use validation accuracy alone as the acceptance criterion -- the capacity gate is independently required
Do not let K[m] grow unboundedly even if each individual edit appears safe
Do not use the capacity gate as a proxy for the validation gate or vice versa -- both must pass
Do not assume architectural changes are representationally separate -- they induce a family H[Z] and the VC sup over reachable architectures applies
Do not infer that Turing-equivalent substrate swaps are safe without checking induced VC family -- Theorem 12 covers computability, not hypothesis expressivity
