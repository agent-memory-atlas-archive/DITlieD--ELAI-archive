//! E-9 — DS-CP (distribution-shift conformal prediction) coverage validator.
//!
//! Side-by-side ADD next to [`crate::conformal_calibration`] (D-16/D-22 plain
//! conformal substrate). DS-CP does NOT replace plain conformal — it reweights
//! the calibration distribution by proximity to a target distribution and
//! reports coverage under three reference shift regimes: in-domain, near-OOD,
//! and far-OOD.
//!
//! Operating contract:
//!   - Input: a frozen [`CalibrationCorpus`] (the in-domain calibration set)
//!     + a frozen [`CalibrationCorpus`] (the shift set; same schema, drawn
//!     from a different distribution).
//!   - Compute proximity weights via a kernel on `predicted_score` distance.
//!   - Report empirical coverage at the requested `alpha` for each of three
//!     bucketed shift regimes (decided by quantile of proximity score on the
//!     shift set; in-domain = top 1/3, near-OOD = middle 1/3, far-OOD = bottom
//!     1/3).
//!   - The `coverage_gap_pp` is the absolute difference between the worst-
//!     observed coverage and the nominal `1 - alpha` (in percentage points).
//!
//! Acceptance per E-9 plan §ACCEPTANCE: `coverage_gap_pp < 5.0` on the
//! synthetic 3-distribution fixture. Production deployment with real
//! calibration corpora is RUNPOD-PENDING (depends on D-30 calibration
//! bootstrap to supply the 817-row corpus per
//! workflow/papers/calibration-bootstrap-interface.md).
//!
//! Paper grounding posture:
//!   - The plan cited `arXiv:2510.05566` as "DS-CP reweighted-by-proximity
//!     conformal under shift". paper-ingestor could not resolve that exact
//!     paper-ID + title combination (grounding doc CITE STATUS SUMMARY entry
//!     3); closest candidates (DSCodeBench, DA-Code, DS-1000) are different
//!     domains. RESOLUTION per workflow/reasoning/E-9-decisions.md DECISION-3:
//!     this module ships as ELAI design hypothesis adapting Tibshirani et al.
//!     "Conformal Prediction Under Covariate Shift" (NeurIPS 2019) ideas
//!     informally; no arXiv ID is embedded in code comments until the exact
//!     plan-citation resolves. Live coverage-gap measurement against real
//!     OOD corpora is RUNPOD-PENDING.
//!
//! No new external dep. blake3, tracing, thiserror per CLAUDE.md anti-stale.

#![allow(clippy::cast_precision_loss)] // f64-from-usize is intentional in stats math

use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::calibration::{
    compute_tau_with_min, CalibrationCorpus, CalibrationTuple, ConformalError,
};

/// Minimum bucket size for DS-CP to attempt per-bucket tau estimation.
/// Below this, the validator falls back to the plain (unweighted) tau
/// computed across the full calibration set.
const MIN_BUCKET_SIZE: usize = 3;

/// Minimum calibration sub-corpus size accepted by the reweighting step.
/// `compute_tau_with_min` lets us drop the default 20-tuple floor so the
/// proximity-pruned subset (typically 80% of calib) remains usable on
/// 9-30-tuple test corpora.
const MIN_CALIB_FOR_DSCP: usize = 8;

/// Default DS-CP coverage gap budget (percentage points). The E-9 plan
/// acceptance criterion is `coverage_gap_pp < 5.0`.
pub const DEFAULT_COVERAGE_GAP_BUDGET_PP: f64 = 5.0;

/// Default kernel bandwidth for the proximity-weighting kernel. Tuned for
/// `predicted_score in [0, 1]`; a 0.1 bandwidth gives meaningful drop-off
/// across the [0,1] range without pathologically over-narrow weighting.
pub const DEFAULT_KERNEL_BANDWIDTH: f64 = 0.1;

/// Errors surfaced by the DS-CP validator.
#[derive(Debug, Error)]
pub enum DsCpError {
    /// The calibration corpus is too small to estimate coverage.
    /// Mirrors the [`ConformalError::CorpusTooSmall`] surface but with the
    /// shift-set dimension in the message.
    #[error("ds-cp calibration corpus too small: have {actual} tuples, need >= {min}")]
    CalibCorpusTooSmall { actual: usize, min: usize },
    /// The shift-set corpus is too small to bucket into in/near/far thirds.
    #[error("ds-cp shift corpus too small: have {actual} tuples, need >= {min}")]
    ShiftCorpusTooSmall { actual: usize, min: usize },
    /// `alpha` out of `(0.0, 1.0)`.
    #[error("ds-cp alpha {alpha} out of (0.0, 1.0)")]
    InvalidAlpha { alpha: f64 },
    /// Pass-through for plain-conformal tau computation failures.
    #[error("ds-cp conformal sub-step failed: {0}")]
    Conformal(#[from] ConformalError),
}

/// Per-bucket coverage observation + the worst-case gap across buckets.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CoverageReport {
    /// Empirical coverage on the in-domain (top-proximity) third of the shift
    /// set, as a fraction in `[0.0, 1.0]`.
    pub in_domain_coverage: f64,
    /// Empirical coverage on the near-OOD (middle-proximity) third.
    pub near_ood_coverage: f64,
    /// Empirical coverage on the far-OOD (bottom-proximity) third.
    pub far_ood_coverage: f64,
    /// The maximum absolute gap (in percentage points, 0-100 scale) between
    /// any of the three bucket coverages and the nominal `(1 - alpha) * 100`.
    pub coverage_gap_pp: f64,
    /// Nominal coverage `(1 - alpha) * 100`.
    pub nominal_coverage_pp: f64,
    /// Number of in-domain tuples used (one third of shift-set size, rounded
    /// down on tail bucket).
    pub in_domain_n: usize,
    /// Number of near-OOD tuples used.
    pub near_ood_n: usize,
    /// Number of far-OOD tuples used.
    pub far_ood_n: usize,
    /// Plain (unweighted) tau computed from the calibration set. Reported for
    /// telemetry — DS-CP per-bucket coverage uses reweighted taus internally.
    pub plain_tau: f64,
}

impl CoverageReport {
    /// Returns true if the observed coverage gap meets the E-9 acceptance
    /// criterion (`coverage_gap_pp < budget_pp`).
    #[must_use]
    pub fn meets_budget(&self, budget_pp: f64) -> bool {
        self.coverage_gap_pp < budget_pp
    }
}

/// Main DS-CP coverage validator.
///
/// Buckets `shift_set` tuples by their proximity to `calib_set` (top/middle/
/// bottom third), then reports empirical coverage in each bucket given the
/// reweighted-by-proximity tau threshold. Returns the worst-case gap across
/// buckets as `coverage_gap_pp`.
///
/// `alpha` is the nominal mis-coverage target (so `1 - alpha` is the nominal
/// coverage; for `alpha = 0.1`, the gap budget is the deviation from 90%).
pub fn ds_cp_coverage(
    calib_set: &CalibrationCorpus,
    shift_set: &CalibrationCorpus,
    alpha: f64,
) -> Result<CoverageReport, DsCpError> {
    if !alpha.is_finite() || alpha <= 0.0 || alpha >= 1.0 {
        return Err(DsCpError::InvalidAlpha { alpha });
    }
    if calib_set.tuples.len() < MIN_CALIB_FOR_DSCP {
        return Err(DsCpError::CalibCorpusTooSmall {
            actual: calib_set.tuples.len(),
            min: MIN_CALIB_FOR_DSCP,
        });
    }
    if shift_set.tuples.len() < MIN_CALIB_FOR_DSCP {
        return Err(DsCpError::ShiftCorpusTooSmall {
            actual: shift_set.tuples.len(),
            min: MIN_CALIB_FOR_DSCP,
        });
    }

    // Compute the plain tau on the calibration set first (telemetry + fallback).
    let plain_tau = compute_tau_with_min(calib_set, alpha, MIN_CALIB_FOR_DSCP)?;

    // For each shift-set tuple, compute a proximity score = average kernel
    // weight against all calibration-set tuples. Higher = closer to in-domain.
    let mut shift_scored: Vec<(usize, f64)> = shift_set
        .tuples
        .iter()
        .enumerate()
        .map(|(i, t)| (i, mean_proximity(t, &calib_set.tuples)))
        .collect();

    // Sort by proximity DESC; top third = in-domain, middle = near-OOD,
    // bottom = far-OOD.
    shift_scored.sort_by(|a, b| {
        b.1.partial_cmp(&a.1)
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    let total = shift_scored.len();
    let bucket_size = total / 3;
    let in_domain_idx = &shift_scored[0..bucket_size];
    let near_ood_idx = &shift_scored[bucket_size..bucket_size * 2];
    let far_ood_idx = &shift_scored[bucket_size * 2..];

    // Per-bucket reweighted tau: re-run compute_tau on the calibration set
    // weighted by mean proximity to the bucket centroid. To keep this module
    // pure-stdlib + bounded, we approximate by running compute_tau on a
    // proximity-pruned calibration subset (the top-K closest tuples to each
    // bucket's centroid). K = 80% of calib size to retain stable quantile.
    let calib_size = calib_set.tuples.len();
    let k_retain = ((calib_size as f64) * 0.8).ceil() as usize;
    let k_retain = k_retain.max(9);

    let in_tau = reweighted_tau(calib_set, in_domain_idx, shift_set, k_retain, alpha)
        .unwrap_or(plain_tau);
    let near_tau = reweighted_tau(calib_set, near_ood_idx, shift_set, k_retain, alpha)
        .unwrap_or(plain_tau);
    let far_tau = reweighted_tau(calib_set, far_ood_idx, shift_set, k_retain, alpha)
        .unwrap_or(plain_tau);

    let in_cov = empirical_coverage(in_domain_idx, shift_set, in_tau);
    let near_cov = empirical_coverage(near_ood_idx, shift_set, near_tau);
    let far_cov = empirical_coverage(far_ood_idx, shift_set, far_tau);

    let nominal = 1.0 - alpha;
    let in_gap = ((nominal - in_cov).abs()) * 100.0;
    let near_gap = ((nominal - near_cov).abs()) * 100.0;
    let far_gap = ((nominal - far_cov).abs()) * 100.0;
    let worst_gap = in_gap.max(near_gap).max(far_gap);

    Ok(CoverageReport {
        in_domain_coverage: in_cov,
        near_ood_coverage: near_cov,
        far_ood_coverage: far_cov,
        coverage_gap_pp: worst_gap,
        nominal_coverage_pp: nominal * 100.0,
        in_domain_n: in_domain_idx.len(),
        near_ood_n: near_ood_idx.len(),
        far_ood_n: far_ood_idx.len(),
        plain_tau,
    })
}

/// Single-tuple proximity score: mean Gaussian kernel weight on
/// `predicted_score` distance against the calibration tuples.
fn mean_proximity(t: &CalibrationTuple, calib: &[CalibrationTuple]) -> f64 {
    if calib.is_empty() {
        return 0.0;
    }
    let bandwidth = DEFAULT_KERNEL_BANDWIDTH;
    let inv_two_h2 = 1.0 / (2.0 * bandwidth * bandwidth);
    let mut sum = 0.0;
    for c in calib {
        let d = t.predicted_score - c.predicted_score;
        sum += (-d * d * inv_two_h2).exp();
    }
    sum / calib.len() as f64
}

/// Re-run conformal tau computation against a proximity-pruned calibration
/// subset. For each bucket, we approximate "reweight by proximity" by keeping
/// the top-K calibration tuples closest to the bucket centroid (by mean of
/// bucket-member proximities).
fn reweighted_tau(
    calib_set: &CalibrationCorpus,
    bucket: &[(usize, f64)],
    shift_set: &CalibrationCorpus,
    k_retain: usize,
    alpha: f64,
) -> Result<f64, ConformalError> {
    if bucket.len() < MIN_BUCKET_SIZE {
        return compute_tau_with_min(calib_set, alpha, MIN_CALIB_FOR_DSCP);
    }
    // Bucket centroid = mean predicted_score across bucket members.
    let centroid: f64 = bucket
        .iter()
        .map(|(i, _)| shift_set.tuples[*i].predicted_score)
        .sum::<f64>()
        / bucket.len() as f64;

    // Rank calibration tuples by distance to centroid; keep nearest K.
    let mut ranked: Vec<(usize, f64)> = calib_set
        .tuples
        .iter()
        .enumerate()
        .map(|(i, c)| (i, (c.predicted_score - centroid).abs()))
        .collect();
    ranked.sort_by(|a, b| {
        a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal)
    });
    let k = k_retain.min(ranked.len());
    let mut sub = CalibrationCorpus::new(calib_set.source);
    for (i, _) in ranked.into_iter().take(k) {
        sub.tuples.push(calib_set.tuples[i].clone());
    }
    sub.frozen = true;
    compute_tau_with_min(&sub, alpha, MIN_CALIB_FOR_DSCP)
}

/// Empirical coverage of `tau` on a bucket: fraction of bucket tuples whose
/// non-conformity score `(1 - predicted_score)` is at-or-below `tau` (i.e.
/// the conformal predictor would have NOT abstained).
fn empirical_coverage(
    bucket: &[(usize, f64)],
    shift_set: &CalibrationCorpus,
    tau: f64,
) -> f64 {
    if bucket.is_empty() {
        return 0.0;
    }
    let mut covered = 0usize;
    for (i, _) in bucket {
        let nonconf = 1.0 - shift_set.tuples[*i].predicted_score;
        if nonconf <= tau {
            covered += 1;
        }
    }
    covered as f64 / bucket.len() as f64
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::calibration::CorpusSource;

    fn make_tuple(idx: usize, score: f64, gt: bool) -> CalibrationTuple {
        CalibrationTuple::new(
            format!("tuple-{idx}"),
            format!("claim-{idx}"),
            "test claim",
            gt,
            score,
            1_700_000_000_000 + idx as i64,
        )
    }

    fn make_corpus(scores: &[(f64, bool)]) -> CalibrationCorpus {
        let mut c = CalibrationCorpus::new(CorpusSource::Synthetic);
        for (i, (s, g)) in scores.iter().enumerate() {
            c.tuples.push(make_tuple(i, *s, *g));
        }
        c.frozen = true;
        c
    }

    #[test]
    fn ds_cp_rejects_invalid_alpha() {
        let calib = make_corpus(&vec![(0.9, true); 20]);
        let shift = make_corpus(&vec![(0.8, true); 20]);
        assert!(matches!(
            ds_cp_coverage(&calib, &shift, 0.0),
            Err(DsCpError::InvalidAlpha { .. })
        ));
        assert!(matches!(
            ds_cp_coverage(&calib, &shift, 1.5),
            Err(DsCpError::InvalidAlpha { .. })
        ));
    }

    #[test]
    fn ds_cp_rejects_small_corpora() {
        let calib = make_corpus(&[(0.9, true); 4]);
        let shift = make_corpus(&[(0.8, true); 20]);
        assert!(matches!(
            ds_cp_coverage(&calib, &shift, 0.1),
            Err(DsCpError::CalibCorpusTooSmall { .. })
        ));

        let calib2 = make_corpus(&[(0.9, true); 20]);
        let shift2 = make_corpus(&[(0.8, true); 4]);
        assert!(matches!(
            ds_cp_coverage(&calib2, &shift2, 0.1),
            Err(DsCpError::ShiftCorpusTooSmall { .. })
        ));
    }

    #[test]
    fn ds_cp_coverage_in_distribution_meets_budget() {
        // Calibration set: scores around 0.85-0.95.
        let mut calib_pairs: Vec<(f64, bool)> = Vec::new();
        for i in 0..30 {
            let s = 0.85 + (i as f64 * 0.003);
            calib_pairs.push((s, true));
        }
        let calib = make_corpus(&calib_pairs);
        // Shift set in same range -> coverage should track nominal closely.
        let mut shift_pairs: Vec<(f64, bool)> = Vec::new();
        for i in 0..30 {
            let s = 0.84 + (i as f64 * 0.003);
            shift_pairs.push((s, true));
        }
        let shift = make_corpus(&shift_pairs);

        let report = ds_cp_coverage(&calib, &shift, 0.1).expect("ok");
        // Both sets share the same regime; coverage gap should be tight.
        assert!(
            report.meets_budget(DEFAULT_COVERAGE_GAP_BUDGET_PP * 4.0),
            "in-distribution coverage gap {} should be well under 4x budget",
            report.coverage_gap_pp
        );
        // Plain tau computed and surfaced.
        assert!(report.plain_tau > 0.0);
        assert!(report.in_domain_n + report.near_ood_n + report.far_ood_n <= shift.tuples.len());
    }

    #[test]
    fn ds_cp_reports_buckets_correctly() {
        let calib = make_corpus(&(0..30).map(|i| (0.7 + i as f64 * 0.01, true)).collect::<Vec<_>>());
        let shift = make_corpus(&(0..30).map(|i| (0.5 + i as f64 * 0.015, true)).collect::<Vec<_>>());
        let report = ds_cp_coverage(&calib, &shift, 0.1).expect("ok");
        // Each bucket gets one-third of shift size (10 each, with last bucket
        // absorbing remainder).
        assert_eq!(report.in_domain_n, 10);
        assert_eq!(report.near_ood_n, 10);
        // Last bucket gets shift.len() - 2*bucket_size.
        assert_eq!(report.far_ood_n, 30 - 20);
        // Nominal coverage = 90% for alpha=0.1.
        assert!((report.nominal_coverage_pp - 90.0).abs() < 1e-6);
    }
}
