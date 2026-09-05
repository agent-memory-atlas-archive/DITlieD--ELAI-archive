Algorithm: LLM Staleness in Code Generation (B-42 citation audit)
ArXiv: 2305.18290 (MISIDENTIFIED — see ID Mismatch section below)
Paper: Direct Preference Optimization: Your Language Model is Secretly a Reward Model

ID MISMATCH — CRITICAL FINDING:
arXiv:2305.18290 is NOT a paper about LLM staleness in code generation.
It is the DPO (Direct Preference Optimization) paper by Rafailov et al., 2023,
about preference-based alignment of language models via a closed-form reward model.
Verified by: WebSearch returning DPO paper title for this ID, local HTML file at
F:\.ELAI_workflow\.claude\papers\2305.18290.html confirmed with title
"Direct Preference Optimization: Your Language Model is Secretly a Reward Model".

B-42 CLAIM (lines 37-40 of B-42_cosmos_artifact_tier.md):
"arXiv:2305.18290 (LLM staleness in code generation, 2023) — confirms
training-data cutoff effects on API/library knowledge. Cosmos refresh
schedule (nightly for fast-moving categories, weekly for slow)
calibrated against expected knowledge half-life per domain."

VERDICT: This citation is FABRICATED or MISREMEMBERED.
No 2023 arxiv paper was found that:
- Measures per-domain LLM knowledge half-life empirically
- Recommends nightly/weekly/monthly/quarterly refresh cadences
- Calibrates refresh schedules against domain-specific staleness decay rates
Multiple exhaustive searches across arxiv.org, WebSearch, and RAG returned no match.

Reliability tier: UNGROUNDED
The B-42 design rationale for per-domain refresh cadences has no verified empirical basis.

ACTUAL FRESHNESS VALUES IN CODEBASE (verified from source):
File: .elai_cc/crates/knowledge/src/freshness.rs
DEFAULT_HALF_LIFE_DAYS for code files: 180 days (line ~23)
DEFAULT_HALF_LIFE_DAYS for docs/specs/changelogs: 90 days (line ~26)
These are implemented as design constants, not derived from any measured paper.
Formula: recency_decay = max(min_decay, exp(-t * ln2 / half_life_days))
At t = half_life_days, decay = 0.5 exactly. (Verified in test at line ~415)

CLOSEST REAL PAPERS FOUND (none match B-42 description exactly):

1. "Dated Data: Tracing Knowledge Cutoffs in Large Language Models"
   arXiv: 2403.12958 (March 2024, NOT 2023)
   Authors: Cheng, Marone, Weller, Lawrie, Khashabi, Van Durme
   Scope: Traces effective vs reported knowledge cutoffs in LLMs via CommonCrawl
   temporal biases. Does NOT provide per-domain half-life or refresh cadence recommendations.

2. "When LLMs Lag Behind: Knowledge Conflicts from Evolving APIs in Code Generation"
   arXiv: 2604.09515 (April 2026, NOT 2023)
   Scope: 270 real-world API updates from 8 Python libraries. Without docs: 42.55%
   executable. With structured docs: 66.36% executable. Does NOT measure half-life
   or recommend cadences.

3. "LLMs Meet Library Evolution: Evaluating Deprecated API Usage in LLM-based Code Completion"
   arXiv: 2406.09834 (June 2024, ICSE 2025, NOT 2023)
   Scope: 7 LLMs, 145 API mappings, 8 Python libraries, 28125 prompts.
   Does NOT measure domain-specific decay rates or half-life.

4. "Keeping Code-Aware LLMs Fresh: Full Refresh, In-Context Deltas, Incremental Fine-Tuning"
   arXiv: 2511.14022 (November 2025, NOT 2023)
   Scope: Flask, SQLAlchemy, Pandas, Poetry. Alias-aware eval + Forgetting Probe.
   Does NOT provide half-life metrics or cadence recommendations.

WHAT B-42 SHOULD DO:
The cosmos refresh cadences (daily/weekly/monthly/quarterly) in B-42 are
DESIGN CHOICES without empirical backing. They should either:
(a) Be labeled as ungrounded design assumptions in B-42 spec, or
(b) Be grounded against the actual existing papers (2403.12958, 2406.09834)
    which show measurable degradation from API staleness but without specific cadence numbers.

Inputs: N/A (no valid algorithm to map — source paper is DPO, not staleness measurement)
Outputs: N/A
Hyperparameters: N/A
Critical invariants: N/A
Complexity: N/A
Failure modes:
B-42 cites a non-existent paper to justify its refresh cadence design.
The codebase half-life constants (180d code, 90d docs) are ungrounded assumptions.
Implementation pitfalls (do NOT do this):
Do NOT treat B-42's nightly/weekly/monthly/quarterly cadences as empirically validated.
Do NOT cite arXiv:2305.18290 as supporting LLM staleness or refresh cadence design.
Do NOT assume any 2023 paper establishes per-domain knowledge half-life measurements.
