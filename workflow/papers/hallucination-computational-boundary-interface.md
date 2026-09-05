Algorithm: Computational Hierarchy of Hallucination Inevitability + Oracle/Adaptive Escape
ArXiv: 2508.07334
Paper: Hallucination as a Computational Boundary: A Hierarchy of Inevitability and the Oracle Escape
Authors: Xi Wang, Quan Shi, Zenghui Ding, Jianqing Gao, Xianjun Yang

Inputs:
PLM h | probabilistic Turing machine | maps s in Sigma* to distribution P(Y) | any finite computable function
ground-truth f_R | relational: Sigma* -> 2^Y | set-valued correct answers per input | mandatory for H_Stray
ground-truth f_P | probabilistic: Sigma* -> P(Y) | distribution-valued truth | mandatory for H_Distort
oracle O | external knowledge source | O: Sigma* -> Y | assumed perfect/noiseless for absolute escape theorem
input string s | Sigma* | any finite string | must be in model domain

Outputs:
H_Stray(h, f_R, s) | scalar in [0,1] | sum of P_h(y|s) for y not in f_R(s) | invariant: >= 0, <= 1
H_Distort(h, f_P, s) | scalar in [0, inf) | D_KL(P_f_P(y|s) || P_h(y|s)) | invariant: >= 0, =0 iff P_h = P_f_P
K(h) | Kolmogorov complexity | length of shortest program for h | finite for any real model
hallucination verdict | {inevitable, escaped} | per-boundary classification | see three-boundary hierarchy

Hyperparameters (boundary thresholds):
epsilon (diagonalization) | > 0 | (0, 1/|Y|) | must satisfy epsilon < 1/|Y| for Theorem 2.4 to bind
tau (uncomputability) | > 0 | (0, ln 2) | choosing tau < ln 2 forces P(correct) > 0.5 assumption
p (info-theoretic threshold) | K(h) | determined by model capacity | K(f_P) > p triggers pumping lemma failure
beta (sparsity weight CLM) | > 0 | problem-dependent | controls hippocampal L1 regularization in R_M
N_0 (RAG-CL crossover) | ~287 queries (empirical) | > C_update / C_query | crossover from RAG cheaper to CLM cheaper

Critical invariants (testable assertions):
# Diagonalization boundary (Theorem 2.4, Section 2.2)
assert H_Stray_on_nemesis_input >= P_h_top_prediction  # most probable output is wrong by construction
assert P_h_top_prediction >= 1.0 / vocab_size           # argmax must be at least 1/|Y|

# Uncomputability boundary (Theorem 2.5, Section 2.3)
# If H_Distort <= tau for only finite exceptions, a halting-problem decider exists -- contradiction
assert not (halting_exception_set_is_finite and tau < math.log(2))  # would imply P(correct) > 0.5 universally

# Information-theoretic boundary / Pumping Lemma (Lemma 2.6, Section 2.4)
assert K_truth > K_model  # implies H_Distort -> infinity (model assigns ~0 prob to incompressible patch z)
assert P_h_z_given_s_star <= epsilon_small  # model has no information about random patch z

# Oracle escape (Theorem 3.1, Section 3.1)
assert H_Stray(oracle_augmented_h, f_O, s) == 0  # oracle model always outputs O(s), which IS the ground truth

# CCA principle (Principle 6.1, Section 6)
assert task_complexity_K <= K_agent_or_augmented_system  # CCA: task must fit in system's computational class

Complexity:
Diagonalization proof | O(1) per model in enumeration (adversarial input constructed once per model)
Pumping lemma | constructs single adversarial input; one-shot argument
Oracle query cost | O(C_query) per inference; total N * (C_infer + C_query) for N queries
CLM update cost | O(C_update) once; amortized C_infer per query after N_0 queries
RAG-CL crossover | Cost_CLM < Cost_RAG when N > C_update / C_query (from Theorem 3.3 proof)
Space | K(h) bits for model; K(h) + K(d|h_t) after CLM update (Section 3.3)

Failure modes:
Any static PLM with finite K(h) will hallucinate on tasks with K(f) >> K(h) -- information-theoretic guarantee
CoT/self-consistency do NOT escape any boundary -- they stay within same computational class (Section 4.1)
Scaling increases K(h) but cannot defeat diagonalization or uncomputability boundaries (Section 4.2)
Pure CLM suffers 12.4% catastrophic forgetting rate on TriviaQA; fact blending reported
Pure RAG has 76.5% noise robustness; 0% forgetting; high per-query cost growing as N*C_query
Oracle access requires perfect external source -- noisy oracle breaks zero-hallucination guarantee
RAG oracle escape is "absolute" but per-query costly; not a computational class change, just augmentation
CCA violated when task K > agent K: hallucination is not a bug but a mathematical certainty

Implementation pitfalls (do NOT do this):
Do NOT treat CoT or increased sampling as a boundary escape -- they are intra-boundary optimizations only
Do NOT assume scaling alone can eliminate hallucination -- it only raises K(h), not changes the Turing class
Do NOT use RAG and claim "zero hallucination" -- Theorem 3.1 only holds with a perfect noiseless oracle
Do NOT conflate H_Stray (probability mass on wrong answers) with H_Distort (KL from ground-truth distribution)
Do NOT apply diagonalization theorem to a single fixed model -- it requires an enumerable sequence of models
Do NOT assume CLM escape is free -- it trades C_update upfront for amortized savings; crossover at ~287 queries
Do NOT claim peer-verification (model checking its own output) as an oracle escape -- oracle must be external and independent

Experimental numbers (Section 5, Table 2):
Pure RAG accuracy: ~98.6%, forgetting: 0%, noise robustness: 76.5%
Pure CLM accuracy: ~81.0%, forgetting: 12.4%, noise robustness: N/A
RAG-CL Hybrid accuracy: ~96.5%, forgetting: 1.1%, noise robustness: 92.3%
RAG-CL crossover point: ~287 queries (Figure 6)
Base model: Mistral-7B; CL via LoRA; retrieval via FAISS
Task: 1000 queries over novel fictional facts (no prior knowledge contamination)
Noise condition: 15% data corruption in RAG knowledge base
