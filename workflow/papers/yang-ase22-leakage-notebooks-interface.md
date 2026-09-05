Algorithm: Static Data Leakage Detection for ML Notebooks (LeakageDetector / Yang et al.)
ArXiv: 2209.03345
Paper: Data Leakage in Notebooks: Static Detection and Better Processes
Authors: Chenyang Yang, Rachel A. Brower-Sinning, Grace A. Lewis, Christian Kastner
Venue: ASE 2022 (37th IEEE/ACM International Conference on Automated Software Engineering)
DOI: 10.1145/3551349.3556918
Local PDF: F:\.ELAI_workflow\.claude\papers\2209.03345.pdf
Reliability: MEDIUM-HIGH (peer-reviewed, top-tier SE venue)

AUTHORSHIP CLARIFICATION:
Task description attributed "~92.9% accuracy, ~30% notebooks exhibiting leakage" to "Subotic & Yang, ASE'22".
Verified correct attribution: Yang, Brower-Sinning, Lewis, Kastner (ASE'22). Subotic is a separate author on a separate SOAP'22 workshop paper. The 92.9% accuracy and 30% notebook prevalence figures belong to Yang et al.

Inputs:
notebook_code | Python (Jupyter .ipynb) | cell sequence | mandatory; must import sklearn or equivalent ML pipeline library
data_flow_graph | static taint/dataflow graph over cells | directed graph | derived from AST; mandatory internal representation
train_test_split_call | sklearn.model_selection.train_test_split or equivalent | AST node | anchor for partition tracking; mandatory to detect split location
fit_transform_calls | set of sklearn transformer method calls | AST nodes | targets for preprocessing leakage detection
test_variable_references | set of variable names carrying test-set identity | symbol set | derived by taint analysis from split call outputs

Outputs:
leakage_verdicts | list of (leakage_type, location) | per-notebook | one entry per detected leakage instance
leakage_type | enum {OVERLAP, PREPROCESSING, MULTI_TEST} | categorical | see taxonomy below
prevalence_estimate | float | scalar | ~30% of notebooks exhibit at least one leakage type (verified: 100K notebook corpus)
precision_preprocessing | float | 0.976 | verified: 97.6% precision for preprocessing leakage detection
recall_preprocessing | float | 0.678 | verified: 67.8% recall for preprocessing leakage detection
accuracy_overall | float | 0.929 | verified: 92.9% overall accuracy across leakage types (262 of 282 potential leakages correctly detected)

Leakage Taxonomy (Yang et al. 3 types):
OVERLAP: training and test sets share data points (exact data point overlap). Test data directly appears in training. Statically detectable via tracking variable identity across split call outputs.
PREPROCESSING: training and test data are transformed together (e.g. fit_transform called on unsplit data, or scaler.fit called on combined dataset). Statically detectable via dataflow taint from split anchor.
MULTI_TEST: same test data used in multiple model evaluations without correction (multiple hypothesis testing inflation). Detectable via tracking test set variable reuse across evaluation calls.

Leakage Classes SOUND-detectable by this analysis:
1. OVERLAP: when test variable flows back into training data before model fit. Static taint tracks this.
2. PREPROCESSING (fit_transform before split): most common detected class. 97.6% precision. SOUND-ish (sound within sklearn API model; unsound for custom transformers).
3. MULTI_TEST: test variable reuse across multiple .score() or .predict() calls without new split. Detectable via variable flow tracking.

Leakage Classes NOT STATICALLY DETECTABLE by this analysis:
1. Target leakage (L2 in Kapoor): feature that is a proxy for or causally downstream of the label. Requires domain knowledge. Not in scope.
2. Temporal leakage (L3.1): time ordering of data. No code pattern reveals time semantics. Not detectable.
3. Non-independence between samples (L3.2): e.g. same subject across train/test. Dataset-level identity, not detectable from code.
4. Sampling bias (L3.3): non-representative test selection. Data distribution property, not code property.
5. Illegitimate features (L2): domain-semantic question. Not detectable.
6. Duplicates (L1.4): requires runtime dataset comparison, not static analysis.

Hyperparameters:
sklearn_api_whitelist | sklearn standard methods | extensible | false negatives for custom transformers not in whitelist
split_anchor_detection | heuristic: train_test_split, cross_val_split | standard sklearn | non-sklearn splits may not be detected
interprocedural_depth | notebook-scoped | single notebook | does not track leakage across imported modules

Critical invariants (testable assertions):
assert leakage_type in {"OVERLAP", "PREPROCESSING", "MULTI_TEST"}, "only three leakage classes are in scope"
assert precision_preprocessing >= 0.97, "97.6% precision verified on Yang et al. evaluation"
assert recall_preprocessing >= 0.67, "67.8% recall verified on Yang et al. evaluation"
assert 0.28 <= prevalence_rate <= 0.32, "~30% of public notebooks exhibit leakage per Yang et al. corpus"
assert accuracy_overall >= 0.92, "92.9% overall accuracy on 282-leakage evaluation set"

Complexity:
Time: O(cells * variables) for dataflow analysis; sub-second per notebook
Space: O(cells * variables) for taint state
Corpus scale: demonstrated on 100,000+ public notebooks (Kaggle, GitHub)
Parallelism: per-notebook independent

Failure modes:
Custom transformer classes not in sklearn API whitelist: taint not propagated through .fit(), false negative
Indirect data assignment (e.g. through dict or list unpacking): variable identity tracking may miss, false negative
Notebook cells executed out-of-order: static analysis assumes sequential cell execution; non-sequential notebooks may produce false positives or negatives
Low recall (67.8%) for preprocessing leakage: roughly 1 in 3 actual preprocessing leakages are missed
No cross-notebook or cross-module analysis: leakage introduced via imported utility functions is invisible

Implementation pitfalls (do NOT do this):
Do NOT claim this analysis covers target leakage or temporal leakage — it explicitly does not
Do NOT treat 92.9% accuracy as sound proof of absence — 67.8% recall means many leakages are missed
Do NOT extend detection to cross-module code without rebuilding the call graph — current analysis is notebook-scoped
Do NOT use for non-Python notebooks without re-implementing the AST analysis layer

H-20 gate relevance:
SOUND (within sklearn API boundary) for: Kapoor L1.2 (preprocessing-before-split), Kapoor L1.3 (feature-selection-before-split), Kapoor L1.4 partial (overlap/duplicate detection)
NOT SOUND for: Kapoor L2 (illegitimate features), L3.1 (temporal), L3.2 (nonindependence), L3.3 (sampling bias)
Recall gap (32% miss rate on preprocessing): means composite gate must not rely on Yang et al. alone for sound detection; combine with Drobnjakovic abstract interpretation for complementary coverage.
The 30% prevalence figure grounds the base rate prior: expect ~30% of unreviewed notebooks to exhibit at least one statically-detectable leakage.
