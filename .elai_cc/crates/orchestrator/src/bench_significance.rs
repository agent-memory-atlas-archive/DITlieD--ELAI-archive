//! E-9 — paired-bootstrap significance harness + power calibration.
//!
//! Provides three free functions used by external_benches::registry and by
//! the per-suite Python runners (via JSON-over-stdio) to gate "regression"
//! verdicts behind statistical significance:
//!
//!   - `paired_bootstrap_p(a, b, n_iter, seed)` -> f64 in [0.0, 1.0]
//!     Two-sided paired-bootstrap p-value for mean(b) - mean(a) under the
//!     null `H0: mean(a) = mean(b)`. Deterministic under fixed `seed` (uses
//!     SplitMix64 PRNG; no rand dep).
//!
//!   - `required_n_for_p_threshold(observed_variance, target_delta,
//!     p_target)` -> usize
//!     Normal-approx sample-size calculator (Welch t-test approximation):
//!     n ~ 2 * (z_{1-p/2} + z_{1-beta})^2 * sigma^2 / delta^2 with beta=0.2.
//!
//!   - `is_low_power(n, required_n)` -> bool — strict-less-than check.
//!
//! Power-calibration verdict emission:
//!   - When `n < required_n`, the caller emits a BenchmarkRunObserved beacon
//!     with `low_power=true` and `phase=Completed`. Callers MUST NOT call the
//!     result a "regression" — they emit `SignificanceVerdict::LowPowerNoConclusion`
//!     from external_benches::registry.
//!
//! Determinism: PRNG is seeded explicitly; no global random state. The default
//! seed for production callers is the BLAKE3 prefix of the suite_name + epoch
//! second bucket, exposed via `default_seed_for_suite`.
//!
//! Paper grounding:
//!   - Efron 1979 "Bootstrap Methods" SIAM Review.
//!   - Politis-Romano 1994 JASA — stationary bootstrap (out-of-scope here;
//!     paired-bootstrap is the i.i.d. block-1 case).
//!   - arXiv:2604.05172 (ClawsBench) — motivates paired-bootstrap p<0.05 gate
//!     on scaffold-vs-model A/B comparisons. See workflow/reasoning/E-9-decisions.md.

#![allow(clippy::cast_precision_loss)] // f64-from-usize is intentional in bootstrap math

/// SplitMix64 PRNG (Steele/Lea 2014). Deterministic, stateless step, no crate dep.
/// Used in place of `rand` to keep the bootstrap path zero-dep + reproducible.
#[derive(Debug, Clone, Copy)]
struct SplitMix64 {
    state: u64,
}

impl SplitMix64 {
    const fn new(seed: u64) -> Self {
        Self { state: seed }
    }

    fn next_u64(&mut self) -> u64 {
        self.state = self.state.wrapping_add(0x9E37_79B9_7F4A_7C15);
        let mut z = self.state;
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        z ^ (z >> 31)
    }

    /// Uniform integer in [0, bound). bound must be > 0.
    fn gen_below(&mut self, bound: usize) -> usize {
        debug_assert!(bound > 0, "gen_below bound must be positive");
        // Unbiased rejection sampling via the high-bits path is overkill for
        // bench resampling; the modulo path is acceptable here because
        // bound << u64::MAX so bias is < 2^-32 of one sample.
        (self.next_u64() as usize) % bound
    }
}

/// Two-sided paired-bootstrap p-value for `mean(b) - mean(a)` under H0: equal means.
///
/// Inputs:
///   - `a`, `b`: paired observations on the same fixture set (must be equal length).
///   - `n_iter`: number of bootstrap resamples; >= 1000 recommended for p<0.05.
///   - `seed`: PRNG seed for reproducibility (use `default_seed_for_suite` in prod).
///
/// Output: p-value in [0.0, 1.0]. Returns 0.5 (no signal) on zero-length inputs.
/// Returns 1.0 if length mismatch (defensive — caller bug, but no panic).
#[must_use]
pub fn paired_bootstrap_p(a: &[f64], b: &[f64], n_iter: usize, seed: u64) -> f64 {
    if a.is_empty() || b.is_empty() {
        return 0.5;
    }
    if a.len() != b.len() {
        return 1.0;
    }
    let n = a.len();
    // Observed paired mean difference.
    let observed_diff: f64 = a
        .iter()
        .zip(b.iter())
        .map(|(ai, bi)| bi - ai)
        .sum::<f64>()
        / n as f64;
    let abs_observed = observed_diff.abs();

    let mut prng = SplitMix64::new(seed);
    let mut as_or_more_extreme: usize = 0;

    // Bootstrap: resample WITH REPLACEMENT from the paired diffs, recenter
    // under H0 (subtract observed mean), and count how often |resampled mean|
    // >= |observed mean|.
    let diffs: Vec<f64> = a.iter().zip(b.iter()).map(|(ai, bi)| bi - ai).collect();
    for _ in 0..n_iter {
        let mut resample_sum: f64 = 0.0;
        for _ in 0..n {
            let idx = prng.gen_below(n);
            // Recenter under H0 by subtracting observed_diff from the picked diff.
            resample_sum += diffs[idx] - observed_diff;
        }
        let resample_mean = resample_sum / n as f64;
        if resample_mean.abs() >= abs_observed {
            as_or_more_extreme += 1;
        }
    }

    // Add-1 smoothing so p is never exactly 0 (Davison-Hinkley 1997 §4.2.4).
    (as_or_more_extreme as f64 + 1.0) / (n_iter as f64 + 1.0)
}

/// Sample-size calculator: smallest `n` that yields paired-test power 1-beta=0.8
/// at significance `p_target` (two-sided) for detecting a paired mean delta
/// `target_delta` given paired-difference variance `observed_variance`.
///
/// Normal approximation: n >= ((z_{1-p/2} + z_{1-beta})^2 * sigma^2) / delta^2.
/// For two-sided p_target=0.05, z_{1-p/2}=1.96; for power=0.8, z_{1-beta}=0.8416.
///
/// Returns `usize::MAX` if `target_delta <= 0.0` (caller bug; surfaces visibly).
/// Returns 1 if `observed_variance <= 0.0` (no variance => 1 obs suffices in theory).
/// Clamped to [1, 1_000_000] to prevent runaway values from corrupt input.
#[must_use]
pub fn required_n_for_p_threshold(
    observed_variance: f64,
    target_delta: f64,
    p_target: f64,
) -> usize {
    const Z_BETA_POWER_080: f64 = 0.8416212335729143; // qnorm(0.8)
    const MAX_REQUIRED_N: usize = 1_000_000;

    if target_delta <= 0.0 || !target_delta.is_finite() {
        return usize::MAX;
    }
    if observed_variance <= 0.0 || !observed_variance.is_finite() {
        return 1;
    }
    if !(0.0..=1.0).contains(&p_target) || !p_target.is_finite() {
        return MAX_REQUIRED_N;
    }
    // Two-sided z_{1-p/2}. Use a small lookup table for the standard p targets
    // (0.01, 0.05, 0.10) rather than a Newton solver — these are the only values
    // the bench harness actually queries.
    let z_p = if (p_target - 0.01).abs() < 1e-9 {
        2.5758293035489_f64
    } else if (p_target - 0.05).abs() < 1e-9 {
        1.9599639845401_f64
    } else if (p_target - 0.10).abs() < 1e-9 {
        1.6448536269515_f64
    } else {
        // Fallback: Beasley-Springer-Moro inverse normal approximation for
        // 1 - p_target/2. Coefficients from Moro 1995.
        moro_inverse_normal(1.0 - p_target / 2.0)
    };

    let z_sum = z_p + Z_BETA_POWER_080;
    let n_f = (z_sum * z_sum) * observed_variance / (target_delta * target_delta);

    if !n_f.is_finite() || n_f <= 0.0 {
        return MAX_REQUIRED_N;
    }
    let n_rounded = n_f.ceil();
    if n_rounded >= MAX_REQUIRED_N as f64 {
        MAX_REQUIRED_N
    } else {
        // .ceil() above already moved fractional up; cast is safe in [1, MAX].
        let raw = n_rounded as usize;
        raw.max(1)
    }
}

/// Strict-less-than power check.
#[must_use]
pub const fn is_low_power(n: usize, required_n: usize) -> bool {
    n < required_n
}

/// Deterministic seed bucket: blake3 of suite_name combined with the wall-clock
/// hour (so runs within the same hour share a seed for reproducibility, but
/// drift across days). Returns a u64 prefix.
#[must_use]
pub fn default_seed_for_suite(suite_name: &str, epoch_hour: u64) -> u64 {
    let mut hasher = blake3::Hasher::new();
    hasher.update(suite_name.as_bytes());
    hasher.update(&epoch_hour.to_le_bytes());
    let h = hasher.finalize();
    let bytes = h.as_bytes();
    u64::from_le_bytes([
        bytes[0], bytes[1], bytes[2], bytes[3], bytes[4], bytes[5], bytes[6], bytes[7],
    ])
}

/// Beasley-Springer-Moro inverse normal CDF approximation (Moro 1995).
/// Used as fallback for non-standard p_target values; |error| < 4.5e-4.
fn moro_inverse_normal(p: f64) -> f64 {
    const A: [f64; 4] = [
        2.50662823884,
        -18.61500062529,
        41.39119773534,
        -25.44106049637,
    ];
    const B: [f64; 4] = [
        -8.47351093090,
        23.08336743743,
        -21.06224101826,
        3.13082909833,
    ];
    const C: [f64; 9] = [
        0.3374754822726147,
        0.9761690190917186,
        0.1607979714918209,
        0.0276438810333863,
        0.0038405729373609,
        0.0003951896511919,
        0.0000321767881768,
        0.0000002888167364,
        0.0000003960315187,
    ];

    let x = p - 0.5;
    if x.abs() < 0.42 {
        let r = x * x;
        let numer = ((A[3] * r + A[2]) * r + A[1]) * r + A[0];
        let denom = (((B[3] * r + B[2]) * r + B[1]) * r + B[0]) * r + 1.0;
        x * numer / denom
    } else {
        let r_raw = if x > 0.0 { 1.0 - p } else { p };
        let r = (-r_raw.ln()).ln();
        let mut sum = C[0];
        let mut r_pow = 1.0;
        for ci in &C[1..] {
            r_pow *= r;
            sum += ci * r_pow;
        }
        if x < 0.0 { -sum } else { sum }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn paired_bootstrap_p_is_in_unit_interval() {
        let a = vec![0.5, 0.6, 0.55, 0.7, 0.65, 0.58, 0.62, 0.61, 0.59, 0.6];
        let b = vec![0.55, 0.65, 0.6, 0.72, 0.68, 0.61, 0.64, 0.63, 0.62, 0.6];
        let p = paired_bootstrap_p(&a, &b, 1000, 42);
        assert!((0.0..=1.0).contains(&p), "p must be in [0,1], got {p}");
    }

    #[test]
    fn paired_bootstrap_p_deterministic_under_fixed_seed() {
        let a = vec![0.5, 0.6, 0.55, 0.7, 0.65];
        let b = vec![0.55, 0.65, 0.6, 0.72, 0.68];
        let p1 = paired_bootstrap_p(&a, &b, 500, 1234);
        let p2 = paired_bootstrap_p(&a, &b, 500, 1234);
        assert!((p1 - p2).abs() < 1e-12, "same seed => same p");
    }

    #[test]
    fn paired_bootstrap_p_empty_returns_half() {
        let p = paired_bootstrap_p(&[], &[], 100, 1);
        assert!((p - 0.5).abs() < 1e-12);
    }

    #[test]
    fn paired_bootstrap_p_length_mismatch_returns_one() {
        let a = vec![0.5, 0.6];
        let b = vec![0.55];
        let p = paired_bootstrap_p(&a, &b, 100, 1);
        assert!((p - 1.0).abs() < 1e-12);
    }

    #[test]
    fn paired_bootstrap_p_identical_arms_high_p() {
        // identical arms => observed_diff == 0 => every resample is "as extreme"
        // => p ~ 1.0 with smoothing.
        let a = vec![0.5, 0.6, 0.55, 0.7, 0.65];
        let b = a.clone();
        let p = paired_bootstrap_p(&a, &b, 1000, 99);
        assert!(p > 0.9, "identical arms should yield high p, got {p}");
    }

    #[test]
    fn required_n_basic_p_05() {
        // sigma^2 = 0.01, delta = 0.02 (2-point delta on [0,1] score scale).
        // Expected: n ~ ((1.96 + 0.8416)^2 * 0.01) / (0.02^2) = 196.
        let n = required_n_for_p_threshold(0.01, 0.02, 0.05);
        assert!(n >= 190 && n <= 210, "expected ~196, got {n}");
    }

    #[test]
    fn required_n_zero_delta_is_max() {
        assert_eq!(required_n_for_p_threshold(0.01, 0.0, 0.05), usize::MAX);
    }

    #[test]
    fn required_n_zero_variance_is_one() {
        assert_eq!(required_n_for_p_threshold(0.0, 0.02, 0.05), 1);
    }

    #[test]
    fn low_power_threshold() {
        assert!(is_low_power(10, 50));
        assert!(!is_low_power(50, 50));
        assert!(!is_low_power(100, 50));
    }

    #[test]
    fn default_seed_stable() {
        let s1 = default_seed_for_suite("swe-bench-pro", 463_000);
        let s2 = default_seed_for_suite("swe-bench-pro", 463_000);
        assert_eq!(s1, s2);
        let s3 = default_seed_for_suite("swe-bench-pro", 463_001);
        assert_ne!(s1, s3);
    }
}
