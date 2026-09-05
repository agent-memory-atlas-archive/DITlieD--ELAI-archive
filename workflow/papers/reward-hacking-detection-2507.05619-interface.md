Algorithm: Reward Hacking Detection via Evaluator Stress Test (EST)
ArXiv: 2507.05619v1
Paper: Detecting and Mitigating Reward Hacking in Reinforcement Learning Systems: A Comprehensive Empirical Study

CITATION NOTE:
arXiv:2507.05619 was renamed between v1 and v2. v1 title (used in D-4 plan) = "Detecting and Mitigating Reward Hacking in Reinforcement Learning Systems: A Comprehensive Empirical Study". v2 title (current) = "Detecting Proxy Gaming in RL and LLM Alignment via Evaluator Stress Tests". The technical content below is sourced from v1 which contains the six-detector framework D-4 references. The D-4 claim of "~80% of empirically observed reward hacks covered" is NOT directly quoted in the paper. Paper states specification gaming (39.8%) + proxy optimization (31.2%) = 70.8% for two categories, or 89.7% adding objective misalignment (18.7%). The "~80%" figure in D-4 is an interpolation; the nearest defensible citation is "the three primary statistical categories (spec gaming, proxy opt, objective misalignment) account for ~89.7% of detected instances" (Section 5.3 prevalence data).

D-4 DETECTOR MAPPING:
D-4 name           | Paper detector class          | Paper section
KlDivergence       | Specification Gaming Detector  | Section 3.2.1
CorrelationDrop    | Proxy Optimization Detector    | Section 3.2.1
IqrOutlier         | Exploitation Pattern Detector  | Section 3.2.1
PerplexityAnomaly  | Objective Misalignment Detector| Section 3.2.1
(Not used in D-4)  | Reward Tampering Detector      | Section 3.2.1
(Not used in D-4)  | Wireheading Detector           | Section 3.2.1

Inputs:
episode_rewards_proxy | f64 sequence | [timestep] | proxy reward signal from reward model, finite, not NaN
episode_rewards_true  | f64 sequence | [timestep] | ground-truth or human-eval reward signal, same length as proxy, may be unavailable at inference time
baseline_episodes     | f64[][] | [episode][timestep] | set of known-clean episodes for establishing ρ_baseline and PPL baseline; minimum count unspecified, authors use full initial training phase
episode_length        | usize | scalar | L, used for window sizing: w = max(10, floor(L/10))

Outputs:
hack_detected       | bool | scalar | true if any detector fires
detector_flags      | [bool; 6] | per-detector | index order: SpecGaming, ProxyOpt, ObjMisalignment, ExploitPattern, RewardTampering, Wireheading
anomaly_score       | f64 | scalar | composite; not defined by paper as a single formula, per-detector scores are independent
detection_latency   | usize | episodes | paper reports 8.7-15.2 episodes to identify hacking in practice

Output invariants:
hack_detected == true iff at least one detector_flags[i] == true
anomaly_score >= 0.0
detection_latency in [1, infinity) episodes (early warning property: signals precede quality decline, Section 4.3)

Hyperparameters:
tau_spec        | 0.3  | (0, inf) | KL divergence threshold for SpecGaming; authors note τ=0.3 chosen empirically across 15 environments; Appendix G sensitivity analysis shows performance degrades sharply below 0.1 and above 0.7
delta_rho       | 0.5  | (0, 1]  | correlation drop threshold for ProxyOpt; Δρ = ρ_expected − ρ_current > 0.5 fires; sliding window w=50 timesteps
iqr_multiplier  | 3.0  | [1.5, inf) | OutlierBound = Q3 + k*IQR or Q1 − k*IQR; paper uses k=3; k=1.5 is common Tukey fence but produces too many false positives per authors
ppl_sigma_mult  | 2.0  | [1, 4]  | perplexity threshold = μ_PPL + k*σ_PPL; paper uses k=2; higher k reduces sensitivity
corr_window     | 50   | [10, inf) timesteps | sliding window for Pearson ρ computation in ProxyOpt detector
ngram_order     | 3    | [1, 5]  | n-gram order for action sequence Markov model in PerplexityAnomaly
iso_contamination | 0.1 | (0, 0.5) | Isolation Forest contamination parameter γ for RewardTampering detector

Critical invariants (testable assertions):
# KlDivergence detector - tau_spec semantics
# KL is computed on the RATIO rho = R_proxy/R_true, not on raw distributions
# D_KL(rho_current || rho_baseline) where rho_baseline = median(R_proxy/R_true) across baseline episodes
assert tau_spec > 0.0, "tau_spec must be positive"
assert not (rho_baseline == 0.0), "rho_baseline of zero makes KL undefined; guard against zero true rewards"

# CorrelationDrop detector - delta_rho semantics
# delta_rho = rho_expected - rho_current; fires when drop EXCEEDS threshold
# rho_expected is fit on initial training phase via robust regression
assert 0.0 < delta_rho <= 1.0, "delta_rho must be in (0,1]"
assert corr_window >= 10, "window below 10 gives unstable Pearson estimates"

# IqrOutlier detector - bounds computed ACROSS episodes, not within episode
# Q1, Q3 derived from baseline episode performance distribution
assert iqr_multiplier >= 1.5, "multiplier below 1.5 (standard Tukey fence) produces excessive false positives"

# PerplexityAnomaly - n-gram Markov model on ACTION sequences, not reward sequences
# ppl = 2^(-1/N * sum(log2(P(a_i | a_{i-2}, a_{i-1}))))
assert ppl_sigma_mult >= 1.0, "sigma multiplier below 1.0 flags normal variation as anomaly"
assert ngram_order >= 2, "unigram (n=1) has no context and cannot detect sequential gaming"

# Coverage claim guard for D-4
# Paper does NOT state "80% coverage" as a single quote. Nearest defensible value:
# spec_gaming_pct + proxy_opt_pct + obj_misalignment_pct == 89.7% of detected instances (Section 5.3)
# The "~80%" in D-4 plan body is an interpolation, not a direct quote. Use 89.7% or "~70-90%" range.
assert abs((39.8 + 31.2 + 18.7) - 89.7) < 0.1, "coverage arithmetic check"

Complexity:
Time: O(L) per episode for all detectors (linear scan of reward/action sequences)
Space: O(B) where B = baseline episode buffer size for rho_baseline and PPL baseline statistics
Parallelism: All six detectors are independent per episode; can be run in parallel per detector
Overhead: 4.2% average computational overhead reported (Section 4, across 15 environments)
Detection latency: 8.7-15.2 episodes before firing (early warning, precedes quality decline per Section 4.3)
Online: Yes, designed for online detection during training, not post-hoc only

Failure modes:
SpecGaming detector fails when R_true is unavailable or noisy (ratio rho = R_proxy/R_true requires both signals; single-signal settings degrade to correlation proxy)
CorrelationDrop fires false positives during legitimate policy shifts with rapid reward structure changes (not only hacking)
IqrOutlier requires a clean baseline distribution; if baseline is contaminated with hacking episodes, Q1/Q3/IQR bounds shift to mask anomalies
PerplexityAnomaly is action-sequence-specific; does not transfer to reward-only monitoring and requires domain-specific n-gram calibration (Section 6.1 cross-domain analysis: perturbation design requires domain adaptation)
Wireheading detector (system call tracing) is environment-specific; not applicable in simulated RL environments without OS-level access
All detectors share a common calibration dependency: rho_baseline, PPL baseline, and IQR bounds degrade if not recomputed after environment changes (distribution shift)
Isolation Forest (RewardTampering) with fixed gamma=0.1 may miss hacking types not represented in training contamination rate

Implementation pitfalls (do NOT do this):
Do NOT compute KL divergence on raw reward distributions (R_proxy and R_true as distributions); compute it on the scalar ratio rho = R_proxy/R_true treated as a distribution of ratio values across a window
Do NOT use IQR bounds computed within a single episode; IQR must be computed across the baseline episode population
Do NOT treat delta_rho as a running correlation value; it is a DROP computed as rho_expected (fitted baseline) minus rho_current (sliding window Pearson)
Do NOT apply PerplexityAnomaly to reward sequences; it operates on action sequences via n-gram Markov models
Do NOT claim the four D-4 detectors cover "80%" of reward hacks as a direct paper quote; the paper's nearest matching claim is 89.7% across three categories (Section 5.3) or 78.4% precision / 81.7% recall for the full framework
Do NOT run RewardTampering or Wireheading detectors in environments lacking reward signal introspection; they require access to reward computation internals
Do NOT set tau_spec < 0.1 (sharp precision degradation per Appendix G sensitivity analysis)

Version note:
This interface map is grounded in arXiv:2507.05619v1. The locally fetched file at F:\.ELAI_workflow\.claude\papers\2507.05619.html is v2 (renamed paper). The v1-specific detector specifications (6 detectors, exact thresholds) were extracted via WebFetch of https://arxiv.org/html/2507.05619v1. If the v2 paper removed or changed the 6-detector framework, re-verify against v1 PDF or v1 HTML directly.
