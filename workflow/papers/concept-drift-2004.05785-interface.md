Algorithm: DDM (Drift Detection Method) as surveyed in Learning under Concept Drift: A Review
ArXiv: 2004.05785
Paper: Learning under Concept Drift: A Review (Lu, Liu, Dong, Gu, Gama, Zhang, 2020)
Primary algorithm source: Gama et al. 2004, "Learning with Drift Detection", SBIA 2004 (Springer LNCS 3171, pp. 286-295)
Scope for D-4: DDM as a 5th tripwire orthogonal to the 4 from arXiv:2507.05619

Citation status: VERIFIED. arXiv:2004.05785 confirmed real via arxiv.org. Lu et al. is a survey that cites DDM but does not reproduce its equations; equations sourced from Gama 2004 (primary) and verified against River 0.11.0 source (riverml.xyz) + scikit-multiflow 0.5.3 docs.

Inputs:
x | int or bool | scalar | 1 = classifier error on instance i, 0 = correct prediction; must be binary {0,1}; mandatory per-sample
n_i | int | implicit counter | number of instances seen since last reset; maintained internally; min_num_instances samples required before detection activates
min_num_instances (warm_start) | int | default 30 | minimum samples before any warning/drift signal can fire; optional hyperparameter

Outputs:
warning_detected | bool | scalar | True when p_i + s_i > p_min + warning_threshold * s_min; resets to False when drift fires
drift_detected | bool | scalar | True when p_i + s_i > p_min + drift_threshold * s_min; triggers internal reset(); warning_detected set False on drift
(no position/index output; caller must track sample index)

State variables (maintained internally across updates):
p_i | float | running error rate = (cumulative errors) / n_i; Bernoulli mean
s_i | float | standard deviation of error rate = sqrt(p_i * (1 - p_i) / n_i); binomial SE formula
p_min | float | value of p_i when (p_i + s_i) last achieved its minimum; initialized to +inf or sentinel
s_min | float | value of s_i at the same instant p_min was recorded; initialized to +inf or sentinel
ps_min | float | cached p_min + s_min minimum; update condition: if p_i + s_i <= ps_min then ps_min = p_i + s_i, p_min = p_i, s_min = s_i

Hyperparameters:
min_num_instances (warm_start) | 30 | int >= 1 | low sensitivity; prevents false positives in cold-start; set higher for noisy streams
warning_threshold (warning_level) | 2.0 | float > 0 | corresponds to ~2-sigma / 95% confidence under normal; authors note standard 2*s_min; sensitivity: lower = more false warnings
drift_threshold (out_control_level) | 3.0 | float > warning_threshold | corresponds to ~3-sigma / 99% confidence; sensitivity: lower = faster detection but more false alarms; must be strictly > warning_threshold

Critical invariants (testable assertions):
assert s_i == math.sqrt(p_i * (1 - p_i) / n_i)  # Eq: binomial SE; verified River source ddm.py
assert warning_detected == (p_i + s_i > p_min + 2.0 * s_min)  # strict > not >=; River impl note: avoids false pos when error=1 always
assert drift_detected == (p_i + s_i > p_min + 3.0 * s_min)  # strict >; triggers reset
assert not (warning_detected and drift_detected)  # drift sets warning_detected = False
assert p_min <= p_i or n_i < min_num_instances  # p_min tracks historical minimum, never degrades upward unless reset
assert 0.0 <= p_i <= 1.0  # error rate bounded; p_i = errors/n_i
assert s_min <= s_i or n_i < min_num_instances  # s_min is minimum of s at the p_min-achieving instant; not necessarily global s minimum
assert drift_threshold > warning_threshold  # architectural invariant; otherwise logic inverts

Behavior on drift detection:
internal state reset: p_i, n_i, p_min, s_min, ps_min all re-initialized to sentinel; drift_detected = True for one cycle; next update begins fresh accumulation
external caller responsibility: replace or retrain the classifier; DDM itself does not mutate the model
warning zone behavior: start building replacement learner; continue using current learner for predictions (Lu et al. survey, Section IV-A)
drift zone behavior: replace current learner with the replacement learner trained during warning zone

Complexity:
time per update | O(1) | single arithmetic update: increment n_i, update running mean p_i, compute sqrt once, two comparisons
space | O(1) | five scalar state variables: n_i, p_i, s_i, p_min, s_min (plus ps_min cache)
parallelism | none required | fully sequential; state is non-commutative (order of samples matters)
no sliding window: DDM uses a landmark window (origin fixed at last reset); entire history since reset contributes to p_i and n_i

Failure modes (from Lu et al. survey + Gama 2004 cited behavior):
1. Delayed detection: drift detection lags actual drift timestamp; requires sufficient samples post-drift to shift p_i enough; gradual drifts may accumulate errors silently for many samples before threshold breach
2. False negatives on abrupt drift when min_num_instances is large: warm-up period suppresses all signals; abrupt drift during warm-up is missed entirely
3. False positives on noisy streams: high base error rate means p_min is high and thresholds are loose; marginal noise spikes can trigger warning
4. Monotonically-wrong classifier pathology: if error rate is consistently 1.0 then p_i*(1-p_i)=0, s_i=0 always; warning/drift thresholds degenerate to p_i > p_min (i.e., 1.0 > 1.0 = False); detector becomes blind; River impl uses strict > specifically to handle this edge case
5. p_min never decreases after a stable period: if the stream was noisy early, p_min may be artificially high; thresholds are then insensitive to subsequent drift
6. No unsupervised operation: DDM requires ground-truth labels to compute error rate; cannot operate in label-delay or label-free scenarios
7. Single-model assumption: DDM is designed for binary correct/incorrect signal from one learner; ensembles or multi-class probabilistic outputs require pre-aggregation

Implementation pitfalls (do NOT do this):
do NOT use >= instead of > for warning/drift comparisons; causes false positive when error is always 1 (s_i=0 pathology)
do NOT share p_min/s_min across detector resets; state must be fully re-initialized after drift; stale p_min from pre-drift era makes post-drift thresholds incorrect
do NOT compute s_i as population stddev (sqrt(p_i*(1-p_i))); must divide by n_i: sqrt(p_i*(1-p_i)/n_i) is the standard error of the proportion
do NOT skip warm-up gate; firing before min_num_instances yields false alarms from high-variance early estimates (n_i small, s_i large)
do NOT set drift_threshold <= warning_threshold; system enters permanent warning state or logic inverts
do NOT assume warning_detected persists after drift fires; it is cleared to False on drift transition
do NOT apply DDM to non-binary inputs (e.g. probability scores); must threshold to 0/1 first; feeding floats to p_i formula violates binomial assumption
do NOT use DDM as sole tripwire for silent/covariate drift with no label feedback; it is label-dependent and blind to feature distribution shifts

Relationship to D-4 tripwire stack:
DDM is orthogonal to the 4 tripwires from arXiv:2507.05619 because it operates on classifier output error signal (supervised, post-prediction) rather than on raw feature distributions (unsupervised). It fires a 5th signal when the deployed model's error rate structurally increases, complementing distribution-distance tripwires that may fire before labels are available. The two systems can both fire independently or together: distribution drift without error increase (model is robust) vs error increase without distribution drift (label noise or annotation shift).
