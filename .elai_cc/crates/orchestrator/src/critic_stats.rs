//! E-10 -- inline paired-bootstrap statistics helper.
//!
//! Decision: workflow/reasoning/E-10-decisions.md DECISION-12. The full
//! bench_significance.rs from E-9 is not yet shipped, so this module
//! provides a minimum-viable seeded paired-bootstrap p-value that the
//! [`crate::fsm_ablation`], [`crate::role_ablation`], and
//! [`crate::rag_challenger`] critic emissions can call inline. When E-9
//! lands, this module collapses into a thin re-export shim.
//!
//! Grounding: standard paired-bootstrap construction (Efron & Tibshirani
//! 1993, Section 6.5); deterministic seed = blake3 of the run_id prefix
//! so the same input produces the same p-value across replays.

use blake3::Hasher;

/// Number of bootstrap resamples. Capped at 1024 to keep wire-dark
/// emissions cheap; the E-9 helper will lift this to 10_000 by default.
const BOOTSTRAP_ITER: usize = 1024;

/// Compute the paired-bootstrap two-sided p-value for the null
/// hypothesis that `baseline` and `challenger` have the same mean.
///
/// Inputs MUST be the same length and represent paired samples (e.g.
/// per-task pass/fail flags or per-query MRR scores). The seed is used
/// to drive a deterministic xorshift so the same inputs produce the
/// same p-value across runs.
///
/// Returns `None` when the inputs are empty or mis-paired (caller
/// surfaces this as `paired_bootstrap_p = None` on the SSB row).
#[must_use]
pub fn paired_bootstrap_p(baseline: &[f64], challenger: &[f64], seed: u64) -> Option<f64> {
    if baseline.is_empty() || baseline.len() != challenger.len() {
        return None;
    }
    let n = baseline.len();
    let diffs: Vec<f64> = baseline.iter().zip(challenger.iter()).map(|(b, c)| c - b).collect();
    let observed_mean: f64 = diffs.iter().sum::<f64>() / (n as f64);

    let mut state: u64 = seed.wrapping_add(0x9E37_79B9_7F4A_7C15);
    if state == 0 {
        state = 0x1234_5678_9ABC_DEF0;
    }
    let mut more_extreme: usize = 0;
    for _ in 0..BOOTSTRAP_ITER {
        let mut resample_mean: f64 = 0.0;
        for _ in 0..n {
            // xorshift64 step
            state ^= state << 13;
            state ^= state >> 7;
            state ^= state << 17;
            let idx = (state as usize) % n;
            resample_mean += diffs[idx];
        }
        resample_mean /= n as f64;
        // Two-sided test: count resamples with |mean| >= |observed|.
        if resample_mean.abs() >= observed_mean.abs() {
            more_extreme += 1;
        }
    }
    Some((more_extreme as f64) / (BOOTSTRAP_ITER as f64))
}

/// Build the deterministic seed for [`paired_bootstrap_p`] from a
/// run-id prefix (8-byte blake3 prefix carried on the SSB payload).
#[must_use]
pub fn seed_from_run_id(run_id_prefix: [u8; 8]) -> u64 {
    let mut h = Hasher::new();
    h.update(&run_id_prefix);
    let digest = h.finalize();
    let bytes = digest.as_bytes();
    u64::from_le_bytes([
        bytes[0], bytes[1], bytes[2], bytes[3],
        bytes[4], bytes[5], bytes[6], bytes[7],
    ])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn paired_bootstrap_empty_returns_none() {
        assert_eq!(paired_bootstrap_p(&[], &[], 42), None);
    }

    #[test]
    fn paired_bootstrap_mismatched_returns_none() {
        assert_eq!(paired_bootstrap_p(&[0.1, 0.2], &[0.3], 42), None);
    }

    #[test]
    fn paired_bootstrap_zero_diff_high_p() {
        let baseline = vec![0.5; 30];
        let challenger = vec![0.5; 30];
        let p = paired_bootstrap_p(&baseline, &challenger, 42).unwrap();
        assert!(p > 0.5, "identical samples should yield high p-value, got {p}");
    }

    #[test]
    fn paired_bootstrap_deterministic() {
        let baseline = vec![0.4, 0.5, 0.6, 0.5, 0.4];
        let challenger = vec![0.6, 0.7, 0.8, 0.7, 0.6];
        let p1 = paired_bootstrap_p(&baseline, &challenger, 42).unwrap();
        let p2 = paired_bootstrap_p(&baseline, &challenger, 42).unwrap();
        assert_eq!(p1, p2, "same seed must yield same p-value");
    }

    #[test]
    fn seed_from_run_id_is_deterministic() {
        let prefix = [0xE1u8, 0, 0, 0, 0, 0, 0, 0x10];
        assert_eq!(seed_from_run_id(prefix), seed_from_run_id(prefix));
    }
}
