Algorithm: Leakage Taxonomy for ML-Based Science (Kapoor & Narayanan)
ArXiv: 2207.07048
Paper: Leakage and the Reproducibility Crisis in ML-based Science
Published as: Patterns (Cell Press), 2023, Vol. 4, Issue 9 -- S2666-3899(23)00159-9
Authors: Sayash Kapoor, Arvind Narayanan (Princeton University)
Local PDF: F:\.ELAI_workflow\.claude\papers\2207.07048.pdf
Reliability: MEDIUM-HIGH (peer-reviewed, Cell Press Patterns journal; originally arXiv 2022)

PURPOSE FOR H-20:
This paper defines the canonical leakage taxonomy that scopes which leakage classes the H-20 COMPOSITE gate targets. It is NOT a detection algorithm paper -- it is a classification and survey paper. Its value is the named class definitions that determine what falls inside vs outside the static-detection soundness boundary.

Survey scope:
Fields affected: 17 scientific fields
Papers affected: 294 papers (some sources report 329 -- the arXiv version vs published Patterns version may differ slightly; 294 is the PMC-verified figure)
Papers documenting the issues: 22 meta-studies
Key case study: civil war prediction domain (ML models appeared to outperform logistic regression; leakage correction eliminated the apparent advantage entirely)

Leakage Taxonomy -- 8 Types (3 top-level categories, verified from PMC full text):

Category L1: Lack of Clean Separation of Training and Test Dataset
L1.1 No test set: same dataset used for both training and testing the model. The most severe form; model sees all evaluation data during training.
L1.2 Pre-processing on training and test set: imputation, scaling, or sampling applied to the entire dataset before train-test split; test set statistics influence training preprocessing.
L1.3 Feature selection on training and test set: feature selection (e.g., mutual information, variance thresholds) applied to full dataset before split; test set feature distribution influences feature selection.
L1.4 Duplicates in datasets: identical or near-identical data points appear in both training and test sets; model memorizes rather than generalizes.

Category L2: Model Uses Features That Are Not Legitimate
L2 Illegitimate features: model trained on features that (a) are unavailable at prediction time, or (b) serve as proxies for or are causally downstream of the outcome variable. Examples: using future values in time-series, using administrative codes that encode the diagnosis being predicted.

Category L3: Test Set Is Not Drawn From the Distribution of Scientific Interest
L3.1 Temporal leakage: test data chronologically precedes training data, or train/test split ignores temporal ordering, causing model to be evaluated on past data it effectively saw during training via future values.
L3.2 Nonindependence between training and test samples: dependent observations split across sets (e.g., multiple measurements from same patient, images from same scan, texts from same author); model exploits within-subject correlation.
L3.3 Sampling bias in test distribution: test set is not representative of the scientific claim's target population (spatial bias, selection bias, demographic bias); reported accuracy does not generalize to the claimed scope.

Static Detectability Assessment (H-20 relevance, derived from paper + corroborating sources):
L1.1 (no test set): STATICALLY DETECTABLE -- presence of a train_test_split call or equivalent is a binary code pattern. If absent, flag.
L1.2 (preprocessing before split): STATICALLY DETECTABLE -- taint analysis traces fit_transform calls to pre-split data (Yang et al. and Drobnjakovic et al. both target this). Sound within sklearn API.
L1.3 (feature selection before split): STATICALLY DETECTABLE -- feature selector .fit() on unsplit data is the same taint pattern as L1.2. Sound within sklearn/feature_selection API.
L1.4 (duplicates): NOT STATICALLY DETECTABLE -- requires runtime dataset comparison (hash or row-level matching). No code pattern reveals duplicates.
L2 (illegitimate features): NOT STATICALLY DETECTABLE -- requires domain-semantic knowledge of which features are legitimate. Undecidable from code structure alone. Must route to human Layer-3 review. THIS IS THE PRIMARY HONEST RESIDUE (TD-5).
L3.1 (temporal leakage): NOT STATICALLY DETECTABLE -- requires knowledge of time semantics in the data columns. Column names alone are insufficient (a column named "date" could be a legitimate feature or a temporal index). Must route to human Layer-3.
L3.2 (nonindependence): NOT STATICALLY DETECTABLE -- requires knowledge of data-generating process (same subject across rows). Not inferable from code. Must route to human Layer-3.
L3.3 (sampling bias): NOT STATICALLY DETECTABLE -- entirely a data distribution property; no code pattern distinguishes biased from unbiased sampling unless the sampling call itself is inspected AND the sampling frame is known. Must route to human Layer-3.

Kapoor paper's detection methodology:
Primary proposed mechanism: model info sheets (21-question structured reporting form, not automated detection)
Paper explicitly states: "none of these errors could have been caught by reading the papers" without structured documentation
Static automated detection: not proposed in this paper; the paper is a taxonomy and survey, not a detection tool
The model info sheets address L1-L3 by requiring authors to document and justify each potential leakage source

Critical claims (testable at gate level):
assert leakage_class in {"L1.1","L1.2","L1.3","L1.4","L2","L3.1","L3.2","L3.3"}, "all leakage claims must map to Kapoor taxonomy"
assert static_detectable_classes == {"L1.1","L1.2","L1.3"}, "only L1.1-L1.3 are sound-static-detectable; others require human review"
assert human_review_required_classes == {"L1.4","L2","L3.1","L3.2","L3.3"}, "these five classes are not statically decidable"
assert len(fields_affected) == 17, "17 fields verified per PMC full text"
assert 290 <= papers_affected <= 330, "survey scope 294-329 papers (version-dependent)"

Complexity:
This is a survey/taxonomy paper, not an algorithm. Complexity not applicable.
Model info sheets: O(1) per paper (21 questions); human review time is the bottleneck.

Failure modes (from paper's own analysis):
Leakage is invisible in paper text: "none of these errors could have been caught by reading the papers" -- implies post-publication detection is the current state of the art, which is inadequate
Model info sheets require honest reporting: adversarial or negligent authors can answer incorrectly; the sheets improve transparency but do not guarantee leakage-free research
L2 (illegitimate features) is an open research problem: the paper acknowledges this class ranges from textbook errors to genuinely ambiguous domain-specific judgment calls
Some L3 leakages are disputed in specific fields: what counts as non-independence depends on scientific conventions within each domain

Implementation pitfalls (do NOT do this):
Do NOT claim static analysis covers L2, L3.1, L3.2, or L3.3 -- these are explicitly outside the static soundness boundary
Do NOT treat L1.4 (duplicates) as statically detectable -- the code may show a split but cannot reveal if the underlying data arrays share rows
Do NOT map Yang et al.'s MULTI_TEST leakage type to L3.2 (nonindependence) -- MULTI_TEST is about repeated evaluation inflation, not sample dependence; closer to L1.1 or a separate statistical testing concern
Do NOT use Kapoor's 294-paper survey as evidence for prevalence in code notebooks -- the survey covers published science papers, not practitioner notebooks (Yang et al. is the source for 30% notebook prevalence)

H-20 COMPOSITE gate soundness boundary summary:
SOUND-detectable by static taint (using Yang + Drobnjakovic): L1.1, L1.2, L1.3
NOT statically decidable (honest residue TD-5, must route to human Layer-3): L1.4, L2, L3.1, L3.2, L3.3
The COMPOSITE gate is sound for 3 of 8 leakage classes. For the remaining 5, the gate must document them as outside-static-boundary and route to human review. Claiming sound detection of L2 or L3 from static analysis alone is a false negative guarantee.
