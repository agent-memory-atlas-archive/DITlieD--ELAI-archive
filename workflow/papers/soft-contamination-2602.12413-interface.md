Algorithm: Soft Contamination Detection and Shallow Generalization Measurement
ArXiv: 2602.12413
Paper: Soft Contamination Means Benchmarks Test Shallow Generalization
Authors: Spiesberger, Vazquez, Pochinkov, Gavenčiak, Grietzer, Leech, Schoots
Submitted: 2026-02-12 (ICML)
Session: s-25120 (D-6_held_out_task_store scope)

SCOPE NOTE -- CRITICAL:
The paper contains NO mention of "9-gram contamination" and NO mention of "LiveCodeBench".
The scope item "semantic-fingerprint extension to 9-gram contamination detection" and
"how it extends LiveCodeBench's pattern" cannot be grounded from this paper.
n-gram matching is referenced only as the PRIOR ART that the paper argues is insufficient
(abstract: "Typical decontamination filters use n-gram matching which fail to detect
semantic duplicates"). No specific n-gram width (e.g., 9) is cited anywhere.
LiveCodeBench does not appear in the bibliography or body text.
These scope items are UNGROUNDED against this paper. If D-6 requires them,
a different or additional source must be located.

--- VERIFIED INTERFACE MAP (from paper source) ---

Inputs:
benchmark_dataset | list[str] | variable length | tasks from MBPP (257), CodeForces (468), MuSR (250), ZebraLogic (1000)
training_corpus | dataset | stratified 1% sample of base + 100% of instruct data | Olmo3 Dolma3_6T + Dolmino (base), Dolci SFT/DPO/RL (instruct)
embedding_model | str | llama-embed-nemotron-8b, FP16 | ranked #2 MTEB at submission time (S3.SS2)
similarity_candidates_per_item | int | 100 | sampled from top 0.1% cosine similarity matches (S3.SS2)
annotation_model | str | gemini-3-flash-preview | classifies candidate pairs as semantic duplicate or not (S3.SS2)

Outputs:
is_soft_contaminated | bool per task | True if >=1 semantic duplicate found in top 100 cosine-sim matches
duplicate_type | enum | exact / equivalent / subset / superset | from LLM annotator (A.4)
duplicate_count | int | number of semantic duplicates found per benchmark item
contamination_rate | float | fraction of benchmark items with >=1 semantic duplicate | in [0.0, 1.0]
shallow_generalization_delta | float | performance lift on unseen benchmark items after finetuning on duplicates of seen items

Hyperparameters:
cosine_similarity_candidate_pool | top 0.1% of corpus | not tunable -- fixed sampling boundary (S3.SS2)
top_k_annotated | 100 | per benchmark item per dataset | sensitivity: detection rate scales with k (Figure 5; CodeForces drops from 77.5% at k=100 to 28.4% at k=1)
lora_rank | 16 | A.3.2 | not varied
lora_alpha | 32 | A.3.2 | not varied
lora_dropout | 0.05 | A.3.2 | not varied
teacher_model_temperature | 0.7 | S3.SS4 | 8 parallel generations
ecological_contamination_rate | ~4 per 10,000 = 0.04% | S4.SS4 | derived from observed MBPP duplicate density; 5% of 10,000-point corpus = 500 duplicates in ecological experiment

Definitions (from paper, S1.p4):
exact_duplicate: training example syntactically identical (up to some number of n-grams) to a test item
semantic_duplicate: training example with same meaning (in some sense) as a test item; not close in string space
soft_contamination: contamination involving semantic duplicates (not syntactic)
shallow_generalization: generalization limited to within-distribution + across-semantic-duplicates; does NOT transfer to different benchmarks in the same domain

Trigger conditions for classifying a task as "soft contaminated" (S3.SS2, S4.SS2):
1. Embed the benchmark item and all sampled corpus chunks using llama-embed-nemotron-8b (FP16)
2. Compute cosine similarity between benchmark item and corpus chunks
3. Identify the top 0.1% cosine similarity matches across the corpus
4. Sample 100 items from that top-0.1% pool (random sample or top-100 depending on dataset)
5. Pass each (benchmark_item, corpus_chunk) pair to gemini-3-flash-preview with annotation prompt (see A.4)
6. Annotator returns: is_semantic_duplicate (bool), type (exact/equivalent/subset/superset), confidence
7. Task is classified "soft contaminated" if at least one of the 100 annotated pairs returns is_semantic_duplicate=True
NOTE: cosine similarity threshold of ~0.8 shows "substantial" semantic duplicate prevalence (Figure 2);
below 0.8 most high-cosine matches are NOT semantic duplicates even in the top-0.1% pool.

Measured contamination rates (S4.SS2, S4.SS1):
MBPP: 100% of 257 tasks have >=1 semantic duplicate in top-100 matches; 0 exact duplicates
CodeForces: 77.5% of 468 tasks have >=1 semantic duplicate in top-100; this is a lower bound
ZebraLogic: 49.5% of 1000 tasks have >=1 exact duplicate; 70%+ for 4x4+ grid problems in RL dataset
MuSR: 0% -- highest cosine similarity ~0.40, manual inspection confirms no duplicates

Performance effects (S4.SS3, S4.SS4 -- Tables 2-5):
MuSR exact dupes: baseline 66.0/28.3 seen/unseen -> finetuned 87.9/87.3 (+20pp on both)
MuSR semantic dupes level 1-3: ~85-87 / ~86-88 seen/unseen (same magnitude as exact)
MuSR cosine-sim-only finetuning: 65-68 / 25-27 (no meaningful gain)
ZebraLogic exact dupes: seen +11.5pp, unseen -6.7pp (asymmetric -- harder benchmark)
MBPP exact dupes: seen +16.6pp, unseen -6.5pp (asymmetric)
MBPP semantic dupes: moderate effect on both seen and unseen
Ecological (5% contamination of 10k corpus): +12% seen, +5.6% unseen vs clean model on MuSR

Critical invariants (testable assertions):
assert contamination_rate_MBPP >= 0.77, "MBPP rate must be >=77%; paper shows 100% -- check embedding or sampling"
assert contamination_rate_CodeForces >= 0.28, "CodeForces floor is 28.4% at k=1; anything lower signals broken pipeline"
assert shallow_gen_musr_exact_seen >= 0.85 and shallow_gen_musr_exact_unseen >= 0.85, "exact dup FT should push MuSR seen+unseen to ~87-88% (Table 2)"
assert cross_domain_delta < 0.05, "finetuning on benchmark dupes must NOT improve mirror benchmark >5pp -- if it does, generalization is deep not shallow"
assert cosine_sim_only_delta < 0.05, "finetuning on high-cosine but non-duplicate data must NOT improve benchmark >5pp (Table 2, Cos Sim rows)"
assert ecological_seen_gain > ecological_unseen_gain, "contaminated seen items must show larger gain than unseen -- asymmetric signal is the contamination marker"

Complexity:
embedding step: O(N * d) where N = corpus size (1% of Dolma3_6T ~ tens of billions tokens), d = embedding dim; compute-heavy, done once
cosine similarity search: O(B * N) per benchmark item B; paper samples 1% corpus (~10B tokens); expensive at scale
annotation: O(B * 100) LLM calls per dataset; gemini-3-flash used for cost; scales linearly with benchmark size
finetuning: LoRA on 7B model, rank=16, 6-10 epochs; O(epochs * dataset_size)
parallelism: embedding and cosine search are fully parallelizable; annotation calls independent per pair

Failure modes (from paper S4.SS2.p5, S5):
high false-negative rate in embedding-based detection -- semantic duplicates that are missed because they fall below the top-0.1% cosine threshold
detection rate scales with number of corpus points sampled; 1% corpus sampling likely underestimates true rate
synthetic data pipelines (EDA, Self-Instruct, rephrasing) create semantic duplicates not in Dolma -- Dolma underestimates rate vs closed corpora
study limited to open-data models; frontier models with 30T tokens will have higher absolute counts but unknown rates
cosine similarity alone does NOT reliably classify semantic duplicates -- must combine with LLM annotation (Figure 2 shows many false positives in top-0.1% below ~0.85 cosine)
MuSR-type structured/algorithmic benchmarks may show zero natural contamination even when other benchmarks are highly contaminated

Implementation pitfalls (do NOT do this):
do NOT use n-gram matching alone to classify soft contamination -- the paper's central finding is that n-gram filters miss semantic duplicates entirely
do NOT threshold on cosine similarity alone without LLM annotation -- many high-cosine matches are not semantic duplicates (Figure 2)
do NOT treat "high cosine similarity" finetuning as equivalent to "semantic duplicate" finetuning -- Cos Sim SFT does not improve benchmark performance (Tables 2-4)
do NOT use a single top-1 cosine match per benchmark item -- detection rate at k=1 is only 28.4% for CodeForces vs 77.5% at k=100
do NOT interpret cross-domain mirror benchmark improvement as evidence against soft contamination -- the paper uses no cross-domain transfer as the positive confirmation of shallow (not deep) generalization
do NOT infer frontier model contamination rates directly from Dolma rates -- corpus composition and synthetic pipelines differ substantially
do NOT use ZebraLogic exact-duplicate results to generalize to semantic duplicate behavior -- exact and semantic duplicates have different effect profiles on ZebraLogic (S4.SS3)

D-6 scope gaps (MUST resolve before using this map in D-6):
"9-gram contamination detection" -- NOT in paper. If D-6 references LiveCodeBench 9-gram filtering,
the source is LiveCodeBench itself (arXiv:2403.07974 or its technical report), not this paper.
"semantic-fingerprint extension to 9-gram" -- this paper uses embedding cosine similarity as an
ALTERNATIVE to n-gram matching, not an extension of it. The framing of "extending LiveCodeBench's
9-gram pattern" is UNGROUNDED against arXiv:2602.12413.
Recommendation: if D-6 needs a 9-gram/LiveCodeBench contamination source,
search for "LiveCodeBench contamination detection n-gram" to find the correct paper,
then ingest that separately. The correct source for embedding-based soft contamination is THIS paper.
