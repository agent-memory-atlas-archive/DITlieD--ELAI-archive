Algorithm: NanoFlux Adversarial Dual-LLM Evaluation and Distillation
ArXiv: 2509.23252
Paper: NanoFlux: Adversarial Dual-LLM Evaluation and Distillation For Multi-Domain Reasoning
Verified: arXiv ID resolves, title confirmed, latest version v3 dated 2026-03-16, authors at Amazon.

SCOPE WARNING FOR D-7 CROSS-DOMAIN CANARY:
The term "canary" does NOT appear anywhere in this paper.
There is NO 5% degradation threshold cited. There is NO canary task selection mechanism.
There is NO cross-domain safeguard or monitoring pattern.
The paper demonstrates simultaneous multi-domain improvement (three domains evaluated independently)
but provides zero mechanism to detect or prevent degradation in non-target domains during training.
If D-7 requires a cross-domain canary pattern, this paper is the WRONG source.
The calling session must either (a) find a different paper, or (b) treat the canary pattern as
an UNGROUNDED design hypothesis not backed by this work.


Inputs:
seed_questions | list[str] | n=5-7 (GSMHard/GenomeBench), n=7-12 (MultiMedQA) | mandatory | domain-specific examples from benchmark datasets
domain | categorical | {GSMHard, GenomeBench, MultiMedQA} | mandatory | determines seed count and novelty thresholds
attacker_model | LLM | Gemma-3-4B or MedGemma-4B | mandatory | generates adversarial questions
judge_model | LLM | Claude-3.7-Sonnet v2 | mandatory | evaluates question validity and answer correctness
target_dataset_size | int | 200 | optional (default 200) | examples to generate per domain
max_turns | int | 1000 | optional (default 1000) | hard cap on adversarial rounds


Outputs:
dataset | list[tuple(question, solution, answer)] | size <= target_dataset_size per domain | mandatory
invariant: every example passed judge confidence >= theta_c=0.90 (section 3)
invariant: every question cosine similarity to all prior questions < theta_q (section 3)
invariant: examples with correctly-answered solutions retained only if solution cosine similarity < theta_s=0.75 (section 3)
fine_tuned_model | LoRA adapter weights | r=8, alpha=32 | optional output if training stage runs


Hyperparameters:
theta_q_gsm | 0.85 | [0,1] | question novelty threshold for GSMHard; higher = stricter deduplication (section 3)
theta_q_genome | 0.80 | [0,1] | question novelty threshold for GenomeBench (section 3)
theta_q_medqa | 0.75 | [0,1] | question novelty threshold for MultiMedQA (section 3)
theta_s | 0.75 | [0,1] | solution novelty threshold, all domains; below this = novel approach, retain (section 3)
theta_c | 0.90 | [0,1] | judge confidence threshold; below triggers resampling; 3 failures = discard (section 3)
lora_rank | 8 | integer > 0 | LoRA rank for fine-tuning (appendix A.1)
lora_alpha | 32 | integer > 0 | LoRA scaling factor (appendix A.1)
lora_dropout | 0.05 | [0,1) | LoRA dropout rate (appendix A.1)
learning_rate | 2e-4 | positive float | with linear decay to 0 (appendix A.1)
batch_size | 4 | positive int | (appendix A.1)
sequence_length | 512 | positive int, tokens | max sequence length (appendix A.1)
training_epochs | 5 | positive int | (appendix A.1)
quality_level | L4 | {L1,L2,L3,L4,L5} | optimal quality-performance tradeoff; L5 causes degradation (section 5.4)
seed_question_count | 7-9 | int | peak complexity range; GSMHard peaks at 7, MultiMedQA at 9; beyond 10 causes concept conflation (section 5.3)
epsilon | 1e-6 | positive float | numerical tolerance for math answer verification (appendix A.1)
web_search_timeout | 30 | seconds | per query during judge evaluation (appendix A.1)
max_queries_per_eval | 5 | positive int | web search queries per evaluation call (appendix A.1)


Critical invariants (testable assertions):
assert all(ex['judge_confidence'] >= 0.90 for ex in dataset), "all examples must pass judge confidence threshold theta_c=0.90 (section 3)"
assert all(ex['question_similarity'] < theta_q for ex in dataset), "no two questions may exceed cosine similarity theta_q (section 3)"
assert len(dataset) <= target_dataset_size, "dataset bounded by target size (section 3)"
assert quality_level != 'L5', "L5 quality level causes performance degradation across all domains (section 5.4)"
assert seed_question_count <= 10, "above 10 seeds causes concept conflation errors (section 5.3)"
assert dataset_size_medqa <= 150, "MultiMedQA peaks at 150 examples; 200 shows 2.6% degradation (section 5.2)"


Complexity:
time: 8-12 GPU hours per domain for dataset generation (appendix A.1, empirical only, no Big-O given)
time: 2-3 GPU hours per domain for LoRA fine-tuning (appendix A.1, empirical only)
space: no formal analysis; LoRA adds r*(d_in+d_out) parameters per layer (rank 8, standard)
compute reduction vs full-dataset: 77% fewer FLOPs on GSMHard, 69% on GenomeBench, 93% on MultiMedQA (Table 1)
parallelism: attacker-defender turns are sequential; novelty filtering can be batched per-embedding


Failure modes:
circularity in LLM-based judging: judge may reward its own reasoning patterns, especially in medicine/genomics (section 5.5)
performance degrades at quality level L5 across all three domains; L4 is optimal (section 5.4)
non-monotonic scaling in medical domain: MultiMedQA peaks at 150 examples, degrades at 200 (section 5.2)
model conflates distinct medical concepts when seed_question_count > 10 (section 5.3)
GSMHard peaks at 7 seed questions (53.87%), degrades beyond; GenomeBench peaks earlier (section 5.3)
single attacker-defender-judge configuration may introduce systematic bias (section 5 limitations)
LLM-only evaluation without Python verification produces ~27% higher error rate on math (section 5.1)
naive self-training (no adversarial targeting) degrades out-of-domain performance (section 2, ReGenesis comparison)


Implementation pitfalls (do NOT do this):
do NOT skip Python tool-augmented verification for math answers; LLM-only judge has ~27% higher error rate (section 5.1)
do NOT target dataset size > 150 for medical domains without additional quality controls (section 5.2)
do NOT use quality level L5; it is strictly worse than L4 across all three benchmarks (section 5.4)
do NOT exceed 10 seed questions; concept conflation errors appear systematically above this count (section 5.3)
do NOT use a single model for both attacker and judge roles; circularity risk (section 5.5)
do NOT treat NanoFlux as providing a cross-domain degradation safeguard; no such mechanism exists in the paper


Benchmark results (verified, Table 1):
GSMHard: baseline 57.4%, NanoFlux-200 63.3%, delta +5.9pp, FLOPs 2.10e16 vs 9.23e16
GenomeBench: baseline 57.6%, NanoFlux-200 61.3%, delta +3.6pp, FLOPs 2.57e16 vs 8.49e16
MultiMedQA: baseline 44.7%, NanoFlux-200 61.2%, delta +16.6pp, FLOPs 2.64e16 vs 3.77e17
GPT-5-High (100B+) reference: 89.52% / 70.65% / 86.23% across three domains

Embedding model: OpenAI text-embedding-3-small, output dimension 1536 (section 3, novelty filtering)
