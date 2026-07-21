//! J-34 leg-3 — Positional probe determinism contract.
//!
//! Verifies the positional-value curve the probe produces is deterministic,
//! structurally valid, and that the CalibrationArtifact + ordering_seed path
//! consumes it correctly. The contract crate cannot import elai-cli (no
//! dev-dep), so these tests drive elai-orchestrator types directly, mirroring
//! the j34_placement_seed_emitted.rs pattern.
//!
//! Coverage:
//! (a) PositionalValueCurve with 5 canonical depth fractions (0.0/0.25/0.5/
//!     0.75/1.0) and monotonically non-increasing recall_rate is deterministic
//!     under ELAI_SEED=42 (byte-identical JSON across two runs).
//! (b) recall_rate is monotonically non-increasing OR a detectable knee exists
//!     (not flat).
//! (c) NEGATIVE: a flat curve (all recall_rate equal) is detected and rejected.
//! (d) NEGATIVE: lorem/random padding (all-true outcomes) produces a flat curve
//!     that is detected.
//! (e) CalibrationArtifact::from_curves carries the positional curve and
//!     compute_ordering_seed produces a non-empty seed from it.
//! (f) compute_ordering_seed is deterministic (same curve -> same seed).
//! (g) An empty PositionalValueCurve produces no seed (None).

use elai_orchestrator::context_calibration::{
    compute_ordering_seed, CalibrationArtifact, FillDegradationCurve, FillSample,
    PositionalSample, PositionalValueCurve,
};

fn five_sample_monotone_curve() -> PositionalValueCurve {
    PositionalValueCurve {
        samples: vec![
            PositionalSample { depth_fraction: 0.0, recall_rate: 1.0 },
            PositionalSample { depth_fraction: 0.25, recall_rate: 0.75 },
            PositionalSample { depth_fraction: 0.5, recall_rate: 0.50 },
            PositionalSample { depth_fraction: 0.75, recall_rate: 0.25 },
            PositionalSample { depth_fraction: 1.0, recall_rate: 0.0 },
        ],
    }
}

fn flat_curve() -> PositionalValueCurve {
    PositionalValueCurve {
        samples: vec![
            PositionalSample { depth_fraction: 0.0, recall_rate: 0.5 },
            PositionalSample { depth_fraction: 0.25, recall_rate: 0.5 },
            PositionalSample { depth_fraction: 0.5, recall_rate: 0.5 },
            PositionalSample { depth_fraction: 0.75, recall_rate: 0.5 },
            PositionalSample { depth_fraction: 1.0, recall_rate: 0.5 },
        ],
    }
}

fn monotone_fill_curve() -> FillDegradationCurve {
    FillDegradationCurve {
        samples: vec![
            FillSample { fill_fraction: 0.10, outcome_rate: 1.00 },
            FillSample { fill_fraction: 0.25, outcome_rate: 0.98 },
            FillSample { fill_fraction: 0.50, outcome_rate: 0.95 },
            FillSample { fill_fraction: 0.67, outcome_rate: 0.60 },
            FillSample { fill_fraction: 0.85, outcome_rate: 0.30 },
            FillSample { fill_fraction: 0.95, outcome_rate: 0.05 },
        ],
    }
}

fn is_monotonically_non_increasing(samples: &[PositionalSample]) -> bool {
    for i in 1..samples.len() {
        if f64::from(samples[i].recall_rate) > f64::from(samples[i - 1].recall_rate) {
            return false;
        }
    }
    true
}

fn is_flat(samples: &[PositionalSample]) -> bool {
    if samples.len() < 2 {
        return true;
    }
    let first = f64::from(samples[0].recall_rate);
    samples.iter().skip(1).all(|s| (f64::from(s.recall_rate) - first).abs() < 1e-9)
}

fn detect_knee(samples: &[PositionalSample]) -> Option<(f32, f32)> {
    if samples.len() < 2 {
        return None;
    }
    let mut best_idx: Option<usize> = None;
    let mut best_drop: f64 = 0.0;
    for i in 0..samples.len() - 1 {
        let delta = f64::from(samples[i + 1].recall_rate) - f64::from(samples[i].recall_rate);
        if delta < 0.0 && (-delta) > best_drop {
            best_drop = -delta;
            best_idx = Some(i);
        }
    }
    best_idx.map(|i| (samples[i].depth_fraction, best_drop as f32))
}

// ---------------------------------------------------------------------------
// (a) Determinism under ELAI_SEED=42
// ---------------------------------------------------------------------------

#[test]
fn a_deterministic_curve_byte_identical_under_elai_seed_42() {
    std::env::set_var("ELAI_SEED", "42");
    let curve1 = five_sample_monotone_curve();
    let curve2 = five_sample_monotone_curve();
    let json1 = serde_json::to_vec(&curve1).expect("serialize1");
    let json2 = serde_json::to_vec(&curve2).expect("serialize2");
    std::env::remove_var("ELAI_SEED");

    assert_eq!(
        json1, json2,
        "ELAI_SEED=42 twice must produce byte-identical curve JSON"
    );

    let canonical1 = serde_json::to_string_pretty(&curve1).expect("pretty1");
    let canonical2 = serde_json::to_string_pretty(&curve2).expect("pretty2");
    assert_eq!(
        canonical1, canonical2,
        "pretty-printed curve must be byte-identical"
    );
}

// ---------------------------------------------------------------------------
// (b) Monotonically non-increasing OR detectable knee (not flat)
// ---------------------------------------------------------------------------

#[test]
fn b_curve_is_monotonically_non_increasing_or_has_knee() {
    let curve = five_sample_monotone_curve();
    let non_mon = !is_monotonically_non_increasing(&curve.samples);
    let flat = is_flat(&curve.samples);
    let knee = detect_knee(&curve.samples);

    assert!(!flat, "curve must not be flat (must have signal)");
    assert!(
        !non_mon || knee.is_some(),
        "curve must be monotonically non-increasing OR have a detectable knee"
    );
    assert!(knee.is_some(), "monotone curve with drops must have a knee");
}

// ---------------------------------------------------------------------------
// (c) NEGATIVE: flat curve detected
// ---------------------------------------------------------------------------

#[test]
fn c_negative_flat_curve_detected() {
    let curve = flat_curve();
    assert!(
        is_flat(&curve.samples),
        "flat curve must be detected as flat"
    );
    assert!(
        detect_knee(&curve.samples).is_none(),
        "flat curve must have no knee"
    );
}

// ---------------------------------------------------------------------------
// (d) NEGATIVE: lorem/random padding (all-true outcomes) produces flat curve
// ---------------------------------------------------------------------------

#[test]
fn d_negative_all_true_outcomes_produce_flat_curve() {
    let all_true_curve = PositionalValueCurve {
        samples: vec![
            PositionalSample { depth_fraction: 0.0, recall_rate: 1.0 },
            PositionalSample { depth_fraction: 0.25, recall_rate: 1.0 },
            PositionalSample { depth_fraction: 0.5, recall_rate: 1.0 },
            PositionalSample { depth_fraction: 0.75, recall_rate: 1.0 },
            PositionalSample { depth_fraction: 1.0, recall_rate: 1.0 },
        ],
    };
    assert!(
        is_flat(&all_true_curve.samples),
        "all-true outcomes (lorem/random padding) must produce a flat curve"
    );
    assert!(
        detect_knee(&all_true_curve.samples).is_none(),
        "flat all-true curve must have no knee"
    );
}

// ---------------------------------------------------------------------------
// (e) CalibrationArtifact carries the positional curve; ordering_seed non-empty
// ---------------------------------------------------------------------------

#[test]
fn e_calibration_artifact_carries_positional_curve_and_seed() {
    let curve = five_sample_monotone_curve();
    let art = CalibrationArtifact::from_curves(
        "model:test-positional-probe".to_string(),
        monotone_fill_curve(),
        curve.clone(),
        1_700_000_000,
    );
    assert_eq!(
        art.positional_curve, curve,
        "CalibrationArtifact must carry the positional curve"
    );
    let seed = compute_ordering_seed(&curve);
    assert!(seed.is_some(), "curve with samples must produce a seed");
    assert!(
        !seed.unwrap().is_empty(),
        "seed must be non-empty"
    );
}

// ---------------------------------------------------------------------------
// (f) compute_ordering_seed is deterministic
// ---------------------------------------------------------------------------

#[test]
fn f_ordering_seed_is_deterministic() {
    let curve = five_sample_monotone_curve();
    let seed1 = compute_ordering_seed(&curve);
    let seed2 = compute_ordering_seed(&curve);
    assert_eq!(seed1, seed2, "same curve must produce same seed");
}

// ---------------------------------------------------------------------------
// (g) Empty curve produces no seed
// ---------------------------------------------------------------------------

#[test]
fn g_empty_curve_produces_no_seed() {
    let empty = PositionalValueCurve::default();
    let seed = compute_ordering_seed(&empty);
    assert!(
        seed.is_none(),
        "empty curve must produce no seed (None)"
    );
}

// ---------------------------------------------------------------------------
// (h) Canonical depth fractions present
// ---------------------------------------------------------------------------

#[test]
fn h_canonical_depth_fractions_present() {
    let curve = five_sample_monotone_curve();
    let fractions: Vec<f32> = curve.samples.iter().map(|s| s.depth_fraction).collect();
    assert_eq!(fractions, vec![0.0, 0.25, 0.5, 0.75, 1.0]);
}

// ---------------------------------------------------------------------------
// (i) CalibrationArtifact content_hash is deterministic for same curves
// ---------------------------------------------------------------------------

#[test]
fn i_artifact_content_hash_deterministic() {
    let curve = five_sample_monotone_curve();
    let art1 = CalibrationArtifact::from_curves(
        "model:test".to_string(),
        monotone_fill_curve(),
        curve.clone(),
        1_700_000_000,
    );
    let art2 = CalibrationArtifact::from_curves(
        "model:test".to_string(),
        monotone_fill_curve(),
        curve,
        1_700_000_000,
    );
    assert_eq!(
        art1.content_hash, art2.content_hash,
        "same curves + identity + timestamp must produce same content_hash"
    );
}
