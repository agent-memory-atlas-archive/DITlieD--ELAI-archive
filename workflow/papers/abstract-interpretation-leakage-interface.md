Algorithm: Abstract Interpretation-Based Data Leakage Static Analysis
ArXiv: 2211.16073
Paper: An Abstract Interpretation-Based Data Leakage Static Analysis
Authors: Filip Drobnjakovic, Pavle Subotic, Caterina Urban
Venue: ESOP 2024 (Springer LNCS 14577, chapter 7); arXiv preprint 2022
Local PDF: F:\.ELAI_workflow\.claude\papers\2211.16073.pdf
Reliability: MEDIUM-HIGH (peer-reviewed, ESOP proceedings)

AUTHORSHIP CLARIFICATION:
This is NOT the same as Subotic, Bojanic, Stojic SOAP@PLDI 2022 ("Statically Detecting Data Leakages in Data Science Code"), which is a distinct precursor workshop paper with no public arXiv ID. That SOAP paper introduced NBLyzer; this paper formalizes it via abstract interpretation. Do not conflate the two.

Inputs:
notebook_code | Python (Jupyter notebook) | arbitrary cell sequence | mandatory; must contain sklearn-style train/test operations
partition_labels | abstract lattice values | {Train, Test, Full, Unknown} | assigned to dataset variables; mandatory internal representation
data_flow_graph | CFG/DFG over notebook cells | directed graph | derived from code; mandatory
fit_operations | set of sklearn fit/fit_transform calls | symbol set | mandatory targets for leakage flag

Outputs:
leakage_verdict | enum {LEAKAGE_DETECTED, NO_LEAKAGE_PROVEN, UNKNOWN} | per-notebook | UNKNOWN when analysis cannot resolve partition label
leakage_locations | list of (cell, line, operation) | source locations | non-empty iff verdict = LEAKAGE_DETECTED
partition_label_map | map variable -> {Train, Test, Full, Unknown} | abstract state | invariant: every variable has exactly one label at each program point
precision_on_corpus | float | scalar | verified 93-94% on 2111 Kaggle notebooks (search-verified; exact figure varies by source: 93% per 2603.10742 citing this work, 94% per broader web summaries)

Leakage Classes Targeted (SOUND-detectable by this analysis):
1. Preprocessing-Before-Split (fit_transform on Full/mixed data before split): test-set info leaks into scaler/encoder fit step. SOUND via label propagation.
2. Normalization-Across-Split (fit on Full, transform on Train+Test separately): equivalent to above, detected by tracking fit call on Full-labeled variable.
3. Feature-Engineering-Before-Split (any sklearn transformer .fit or .fit_transform called on unsplit data): generalizes to PCA, imputers, polynomial features. SOUND.
4. Fit-Before-Split (generic): any call to a .fit()-class method where the argument carries label Full or Test. SOUND by construction.

Leakage Classes NOT STATICALLY DETECTABLE by this analysis:
1. Target leakage (illegitimate features): requires domain semantics to determine if a feature is a proxy for the label. Undecidable statically. Must route to human Layer-3 review.
2. Temporal leakage (test set chronologically before train set): requires knowledge of time semantics in the data, not derivable from code structure alone. Not detectable.
3. Non-independence between samples (e.g. same patient in train and test): requires dataset-level identity knowledge, not present in code. Not detectable.
4. Sampling bias / distribution shift: entirely a data property, no code pattern to taint. Not detectable.
5. Duplicates across train/test: requires runtime dataset comparison. Not detectable statically.
6. Illegitimate feature proxies: semantic knowledge of domain required. Not detectable.
7. Arbitrary Python operations outside tracked sklearn primitives: if user calls a custom function that internally leaks, taint tracking may not follow through opaque calls.

Hyperparameters:
abstract_lattice_height | 4 levels | {Train=bottom, Test, Full, Unknown=top} | fixed; not tunable
analysis_scope | notebook cells | all cells | widening applied across loop-like cell repetition patterns
fit_call_whitelist | set of sklearn method names | sklearn standard | extensible; missing entries = false negatives

Critical invariants (testable assertions):
assert all(label in {"Train", "Test", "Full", "Unknown"} for label in partition_label_map.values()), "lattice domain must be closed"
assert leakage_verdict == "LEAKAGE_DETECTED" if any(label in ("Test", "Full") for (var, label) in fit_args), "fit on non-Train data is leakage"
assert not (leakage_verdict == "NO_LEAKAGE_PROVEN" and any_fit_on_full_labeled_data), "soundness: Full-labeled fit must flag as leakage"
assert precision_on_corpus >= 0.93, "93%+ precision verified on 2111 Kaggle notebooks"

Complexity:
Time: polynomial in notebook size (data-flow analysis); lightweight enough for interactive use (seconds per notebook per paper claim)
Space: O(variables * cells) for abstract state table
Parallelism: per-notebook independent; embarrassingly parallel across corpus

Failure modes:
Aliasing through opaque Python calls (e.g. a wrapper function that internally calls fit_transform): label not propagated, false negative
Dynamic attribute access (getattr patterns): taint tracking cannot follow, unknown label assigned
Intra-cell aliasing with complex Python idioms: may produce Unknown label, reducing verdict to UNKNOWN (conservative but not informative)
Over-approximation with Unknown label: legitimate code may be labeled Unknown and produce false positives on ambiguous paths
Analysis targets only preprocessing-class leakage; all semantic/domain leakage classes are outside scope by design

Implementation pitfalls (do NOT do this):
Do NOT treat UNKNOWN verdict as NO_LEAKAGE_PROVEN — Unknown means analysis could not resolve, not that no leakage exists
Do NOT extend the fit_call_whitelist without re-validating soundness — missing entries produce false negatives not false positives
Do NOT skip the data-flow join at cell boundaries — missing join loses cross-cell taint propagation
Do NOT conflate this paper (2211.16073) with Subotic SOAP'22 — they are different papers with different scopes and precision numbers
Do NOT use this analysis alone as evidence for L2/L3 leakage classes from Kapoor & Narayanan — those require human review

H-20 gate relevance:
SOUND for: L1.2 (preprocessing-before-split), L1.3 (feature-selection-before-split) in Kapoor taxonomy
NOT SOUND for: L1.1 (no test set), L1.4 (duplicates), L2 (illegitimate features), L3.1 (temporal), L3.2 (nonindependence), L3.3 (sampling bias)
Methodological boundary: all leakage classes requiring dataset-level or domain-semantic knowledge are outside the decidable static boundary. These are the honest-residue (TD-5) that must route to human Layer-3 review.
