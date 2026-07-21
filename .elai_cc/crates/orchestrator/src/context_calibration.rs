//! J-34 — empirical context calibration.
//!
//! Types + deterministic knee detection + artifact store for per-model
//! fill-degradation and positional-value curves. The curves are produced by
//! `elai bench context-fill` (leg 1, separate slice) and consumed by
//! `context_budget::recalibrate_budget_with_calibration` (leg 2 seam at
//! context_budget.rs:852) and the 3E slot allocator (leg 3, separate slice).
//!
//! This module is TYPE + DETECTION + STORE only. It contains no I/O, no LLM
//! calls, no SSB emit. The artifact store is in-process; persistence is owned
//! by the bench arm that writes the artifact and the runtime that loads it.
//! The plan (docs/thefinalglance/J-plan/J-34_empirical_context_calibration.md)
//! names "an artifact store keyed by model identity" without specifying a
//! filesystem layout; this module exposes a `CalibrationArtifactStore` map
//! plus a `load_artifact` hook the runtime can implement, so no FS layout is
//! invented here.
//!
//! Grounding: J-34 plan legs 1+2; H-50 trim_threshold (context_policy.rs:223)
//! is the consumer of `calibrated_trim_fraction`. `ModelIdentity::identity_key`
//! (model_drift.rs:160, blake3) is the cache key surface.

use crate::context_budget::ModelCapabilityProfile;
use std::collections::HashMap;

// ---------------------------------------------------------------------------
// Curve types
// ---------------------------------------------------------------------------

/// One (fill_fraction, verified_outcome_rate) sample from the bench.
/// `fill_fraction` is in [0.0, 1.0] of the effective context window.
/// `outcome_rate` is the W-1-grade verified-outcome rate (gate verdicts,
/// never LLM-judged — TD-3). Both are `f32` for serde portability; the knee
/// detector converts to `f64` internally for the arithmetic.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct FillSample {
    pub fill_fraction: f32,
    pub outcome_rate: f32,
}

/// Per-model fill-degradation curve: verified-outcome rate as a function of
/// context-window fill. Monotone-non-increasing in the expected case; the
/// knee detector handles non-monotone inputs by taking the LARGEST single
/// step-down (so a single noisy up-tick does not mask the knee).
#[derive(Debug, Clone, Default, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct FillDegradationCurve {
    pub samples: Vec<FillSample>,
}

/// One (depth_fraction, recall_rate) sample from the positional-value probe.
/// `depth_fraction` in [0.0, 1.0] is the position in the assembled prompt
/// where a constraint-recall canary was placed; `recall_rate` is the
/// honored/violated scoring (1.0 = always honored).
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct PositionalSample {
    pub depth_fraction: f32,
    pub recall_rate: f32,
}

/// Per-model positional-value curve: constraint recall as a function of
/// placement depth. The 3E slot allocator (leg 3) consumes this to order
/// slots by (recall-sensitivity x positional value).
#[derive(Debug, Clone, Default, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct PositionalValueCurve {
    pub samples: Vec<PositionalSample>,
}

// ---------------------------------------------------------------------------
// Knee detection
// ---------------------------------------------------------------------------

/// Detected knee of a fill-degradation curve: the fill fraction at which
/// outcome rate drops most steeply. `knee_fill_fraction` is the fill at the
/// START of the steepest drop (the "compact HERE" point). `drop_magnitude`
/// is the absolute outcome_rate decrease across that step.
///
/// Deterministic, no LLM: the knee is the index of the largest consecutive
/// `outcome_rate[i+1] - outcome_rate[i]` decrease (most negative delta). Ties
/// resolve to the lowest fill_fraction (earliest knee, conservative trim).
/// For curves with fewer than 2 samples the knee is `None` (caller falls back
/// to folklore constants — the J-34 NEGATIVE contract: no silent
/// pseudo-calibration).
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct DetectedKnee {
    pub knee_fill_fraction: f32,
    pub drop_magnitude: f32,
}

/// Detect the knee of a fill-degradation curve.
///
/// Algorithm: sort samples by `fill_fraction` ascending (defensive; the bench
/// emits them in order but the store does not require it), compute consecutive
/// `outcome_rate` deltas, return the sample at the start of the largest
/// step-down. If the curve is flat or monotone-increasing (no step-down > 0),
/// returns `None` — a curve with no degradation knee is not a calibration
/// signal, and the caller MUST fall back to the folklore constant rather than
/// inventing a knee.
#[must_use]
pub fn detect_knee(curve: &FillDegradationCurve) -> Option<DetectedKnee> {
    if curve.samples.len() < 2 {
        return None;
    }
    let mut pts: Vec<(f64, f64)> = curve
        .samples
        .iter()
        .map(|s| (f64::from(s.fill_fraction), f64::from(s.outcome_rate)))
        .collect();
    pts.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap_or(std::cmp::Ordering::Equal));
    let mut best_idx: Option<usize> = None;
    let mut best_drop: f64 = 0.0;
    for i in 0..pts.len() - 1 {
        let delta = pts[i + 1].1 - pts[i].1;
        if delta < 0.0 && (-delta) > best_drop {
            best_drop = -delta;
            best_idx = Some(i);
        }
    }
    best_idx.map(|i| DetectedKnee {
        knee_fill_fraction: pts[i].0 as f32,
        drop_magnitude: best_drop as f32,
    })
}

// ---------------------------------------------------------------------------
// Calibration artifact
// ---------------------------------------------------------------------------

/// A per-model calibration artifact: the measured curves + detected knee +
/// blake3 content hash + the model identity key it was produced under.
///
/// The content hash is `blake3(serde_json::canonical(curves + knee + ts))`
/// (CLAUDE.md anti-stale rule: blake3 ONLY, never blake2/sha2). It lets the
/// store detect byte-identical rewrites from re-run benches (GD-2
/// byte-recomputability) without re-hashing the whole journal.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct CalibrationArtifact {
    /// `ModelIdentity::identity_key` (model_drift.rs:160) — the cache key.
    pub identity_key: String,
    pub fill_curve: FillDegradationCurve,
    pub positional_curve: PositionalValueCurve,
    pub knee: Option<DetectedKnee>,
    /// Unix-epoch seconds at which the bench produced this artifact.
    /// Used by G-10 invalidation (model_drift) to decide stale-vs-fresh.
    pub produced_at_secs: u64,
    /// blake3 content hash of the canonical JSON of (fill_curve,
    /// positional_curve, knee, produced_at_secs). Computed by
    /// [`CalibrationArtifact::compute_hash`].
    pub content_hash: String,
}

impl CalibrationArtifact {
    /// Compute the blake3 content hash of this artifact's payload.
    /// The hash covers `fill_curve`, `positional_curve`, `knee`, and
    /// `produced_at_secs` — NOT `identity_key` (the key is the lookup
    /// surface, not payload content) and NOT `content_hash` (self-hash).
    #[must_use]
    pub fn compute_hash(&self) -> String {
        let payload = serde_json::json!({
            "fill_curve": self.fill_curve,
            "positional_curve": self.positional_curve,
            "knee": self.knee,
            "produced_at_secs": self.produced_at_secs,
        });
        let canonical = serde_json::to_vec(&payload).unwrap_or_default();
        let h = blake3::hash(&canonical);
        format!("{:.32}", h.to_hex())
    }

    /// Construct an artifact from its curves + identity, computing the knee
    /// and content hash internally. This is the constructor the bench arm
    /// should use; it guarantees knee/hash are consistent with the curves.
    #[must_use]
    pub fn from_curves(
        identity_key: String,
        fill_curve: FillDegradationCurve,
        positional_curve: PositionalValueCurve,
        produced_at_secs: u64,
    ) -> Self {
        let knee = detect_knee(&fill_curve);
        let mut art = Self {
            identity_key,
            fill_curve,
            positional_curve,
            knee,
            produced_at_secs,
            content_hash: String::new(),
        };
        art.content_hash = art.compute_hash();
        art
    }
}

// ---------------------------------------------------------------------------
// Artifact store
// ---------------------------------------------------------------------------

/// In-process calibration artifact store keyed by `ModelIdentity::identity_key`.
///
/// Newest-wins on insert (same identity_key overwrites). The store is
/// `Send + Sync` via the inner `HashMap`; the runtime holds one instance and
/// lookups are by borrowed key. Persistence (load-on-startup, save-on-bench)
/// is owned by the caller — this module does not invent a filesystem layout.
#[derive(Debug, Clone, Default)]
pub struct CalibrationArtifactStore {
    artifacts: HashMap<String, CalibrationArtifact>,
}

impl CalibrationArtifactStore {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Insert or replace an artifact. Newest-wins (GD-2 re-run replaces).
    pub fn insert(&mut self, artifact: CalibrationArtifact) {
        self.artifacts.insert(artifact.identity_key.clone(), artifact);
    }

    /// Lookup by identity key. Returns `None` when no artifact is registered
    /// for this model — the caller MUST fall back to folklore constants and
    /// LOG that fallback (J-34 NEGATIVE contract: no silent
    /// pseudo-calibration).
    #[must_use]
    pub fn get(&self, identity_key: &str) -> Option<&CalibrationArtifact> {
        self.artifacts.get(identity_key)
    }

    /// Remove an artifact (G-10 invalidation path: on model drift, the
    /// previous identity's artifact is dropped so the next lookup falls back
    /// to constants until a fresh bench runs).
    pub fn invalidate(&mut self, identity_key: &str) -> Option<CalibrationArtifact> {
        self.artifacts.remove(identity_key)
    }

    /// Number of registered artifacts.
    #[must_use]
    pub fn len(&self) -> usize {
        self.artifacts.len()
    }

    /// Whether the store is empty.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.artifacts.is_empty()
    }
}

// ---------------------------------------------------------------------------
// apply_to_profile — the leg-2 seam
// ---------------------------------------------------------------------------

/// Apply a calibration artifact to a `ModelCapabilityProfile`.
///
/// Sets `profile.calibrated_trim_fraction` from the artifact's knee
/// (`knee_fill_fraction`) when present. When the artifact has no knee (curve
/// flat or monotone-increasing), `calibrated_trim_fraction` is left `None`
/// — the caller falls back to `CLOUD_TRIM_FRACTION` / the env constant.
///
/// Also copies the curves onto the profile so the 3E slot allocator (leg 3)
/// and the dashboard tile (leg 5) can read them without a separate lookup.
///
/// This fn is pure (no I/O, no logging); the "log the fallback" obligation
/// lives in the caller (`recalibrate_budget_with_calibration`) so the SSB
/// `ThresholdApplied` / fallback log lands at the seam, not in the type layer.
pub fn apply_to_profile(artifact: &CalibrationArtifact, profile: &mut ModelCapabilityProfile) {
    profile.fill_degradation_curve = Some(artifact.fill_curve.clone());
    profile.positional_value_curve = Some(artifact.positional_curve.clone());
    if let Some(knee) = &artifact.knee {
        // Clamp to (0.0, 1.0] — a knee at 0.0 is a degenerate curve and
        // must not produce a 0% trim fraction (would trim everything).
        let frac = if knee.knee_fill_fraction <= 0.0 {
            None
        } else {
            Some(knee.knee_fill_fraction.min(1.0) as f64)
        };
        profile.calibrated_trim_fraction = frac;
    } else {
        profile.calibrated_trim_fraction = None;
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::context_budget::ModelCapabilityProfile;

    fn sample(fill: f32, outcome: f32) -> FillSample {
        FillSample { fill_fraction: fill, outcome_rate: outcome }
    }

    fn monotone_curve() -> FillDegradationCurve {
        FillDegradationCurve {
            samples: vec![
                sample(0.10, 1.00),
                sample(0.25, 0.98),
                sample(0.50, 0.95),
                sample(0.67, 0.60),
                sample(0.85, 0.30),
                sample(0.95, 0.05),
            ],
        }
    }

    #[test]
    fn knee_on_monotone_curve_is_largest_drop() {
        let curve = monotone_curve();
        let knee = detect_knee(&curve).expect("monotone curve has a knee");
        // Largest step-down is 0.95 -> 0.60 at fill=0.50 (delta -0.35).
        assert!((knee.knee_fill_fraction - 0.50).abs() < 1e-6);
        assert!((knee.drop_magnitude - 0.35).abs() < 1e-6);
    }

    #[test]
    fn knee_on_flat_curve_is_none() {
        let curve = FillDegradationCurve {
            samples: vec![sample(0.1, 0.9), sample(0.5, 0.9), sample(0.9, 0.9)],
        };
        assert!(detect_knee(&curve).is_none(), "flat curve has no knee");
    }

    #[test]
    fn knee_on_increasing_curve_is_none() {
        let curve = FillDegradationCurve {
            samples: vec![sample(0.1, 0.5), sample(0.5, 0.7), sample(0.9, 0.9)],
        };
        assert!(detect_knee(&curve).is_none(), "increasing curve has no knee");
    }

    #[test]
    fn knee_on_single_sample_is_none() {
        let curve = FillDegradationCurve { samples: vec![sample(0.5, 0.9)] };
        assert!(detect_knee(&curve).is_none());
    }

    #[test]
    fn knee_on_empty_curve_is_none() {
        let curve = FillDegradationCurve { samples: vec![] };
        assert!(detect_knee(&curve).is_none());
    }

    #[test]
    fn knee_tie_resolves_to_lowest_fill() {
        // Two equal-magnitude drops: 0.25->0.50 (-0.25) and 0.50->0.75 (-0.25).
        // Conservative tie-break: earliest knee (0.25). Values are chosen so
        // the f32 deltas are exactly equal (-0.25, -0.25).
        let curve = FillDegradationCurve {
            samples: vec![
                sample(0.25, 1.00),
                sample(0.50, 0.75),
                sample(0.75, 0.50),
            ],
        };
        let knee = detect_knee(&curve).expect("tie curve has a knee");
        assert!((knee.knee_fill_fraction - 0.25).abs() < 1e-6);
        assert!((knee.drop_magnitude - 0.25).abs() < 1e-6);
    }

    #[test]
    fn knee_handles_unsorted_input() {
        // Same points as monotone_curve but shuffled; knee must be identical.
        let curve = FillDegradationCurve {
            samples: vec![
                sample(0.85, 0.30),
                sample(0.10, 1.00),
                sample(0.95, 0.05),
                sample(0.50, 0.95),
                sample(0.25, 0.98),
                sample(0.67, 0.60),
            ],
        };
        let knee = detect_knee(&curve).expect("shuffled curve has a knee");
        assert!((knee.knee_fill_fraction - 0.50).abs() < 1e-6);
    }

    #[test]
    fn artifact_from_curves_computes_knee_and_hash() {
        let curve = monotone_curve();
        let art = CalibrationArtifact::from_curves(
            "test-identity".to_string(),
            curve.clone(),
            PositionalValueCurve::default(),
            1_700_000_000,
        );
        assert!(art.knee.is_some(), "knee computed from curves");
        assert!(!art.content_hash.is_empty(), "content hash populated");
        // Recomputing the hash must match (deterministic).
        assert_eq!(art.content_hash, art.compute_hash());
    }

    #[test]
    fn store_insert_get_invalidate() {
        let mut store = CalibrationArtifactStore::new();
        assert!(store.is_empty());
        let art = CalibrationArtifact::from_curves(
            "id-A".to_string(),
            monotone_curve(),
            PositionalValueCurve::default(),
            1_700_000_000,
        );
        store.insert(art.clone());
        assert_eq!(store.len(), 1);
        let got = store.get("id-A").expect("present");
        assert_eq!(got.identity_key, "id-A");
        // Missing identity returns None (no silent pseudo-calibration).
        assert!(store.get("id-B").is_none());
        // Insert with same key overwrites (newest-wins).
        let art2 = CalibrationArtifact::from_curves(
            "id-A".to_string(),
            monotone_curve(),
            PositionalValueCurve::default(),
            1_800_000_000,
        );
        store.insert(art2.clone());
        let got2 = store.get("id-A").expect("present after overwrite");
        assert_eq!(got2.produced_at_secs, 1_800_000_000);
        // Invalidate drops the entry.
        let removed = store.invalidate("id-A");
        assert!(removed.is_some());
        assert!(store.get("id-A").is_none());
        assert!(store.is_empty());
    }

    #[test]
    fn apply_to_profile_sets_calibrated_trim_fraction_from_knee() {
        let mut profile = ModelCapabilityProfile::micro_7b_12gb();
        assert!(profile.calibrated_trim_fraction.is_none());
        assert!(profile.fill_degradation_curve.is_none());
        let art = CalibrationArtifact::from_curves(
            "id-A".to_string(),
            monotone_curve(),
            PositionalValueCurve::default(),
            1_700_000_000,
        );
        apply_to_profile(&art, &mut profile);
        let frac = profile.calibrated_trim_fraction.expect("knee present");
        assert!((frac - 0.50).abs() < 1e-6, "trim fraction = knee fill");
        assert!(profile.fill_degradation_curve.is_some());
        assert!(profile.positional_value_curve.is_some());
    }

    #[test]
    fn apply_to_profile_with_no_knee_leaves_trim_fraction_none() {
        // Flat curve -> no knee -> calibrated_trim_fraction stays None
        // (caller falls back to folklore constant, no pseudo-calibration).
        let mut profile = ModelCapabilityProfile::micro_7b_12gb();
        let flat = FillDegradationCurve {
            samples: vec![sample(0.1, 0.9), sample(0.5, 0.9), sample(0.9, 0.9)],
        };
        let art = CalibrationArtifact::from_curves(
            "id-flat".to_string(),
            flat,
            PositionalValueCurve::default(),
            1_700_000_000,
        );
        assert!(art.knee.is_none(), "flat curve has no knee");
        apply_to_profile(&art, &mut profile);
        assert!(profile.calibrated_trim_fraction.is_none(), "no pseudo-calibration");
        // Curves are still copied (the dashboard can render the flat curve).
        assert!(profile.fill_degradation_curve.is_some());
    }

    #[test]
    fn apply_to_profile_clamps_degenerate_zero_knee() {
        // Knee at fill=0.0 would trim everything; apply_to_profile must
        // refuse it (leave None) rather than produce a 0% trim fraction.
        let mut profile = ModelCapabilityProfile::micro_7b_12gb();
        let curve = FillDegradationCurve {
            samples: vec![
                sample(0.0, 1.0),
                sample(0.5, 0.3),
            ],
        };
        let art = CalibrationArtifact::from_curves(
            "id-degenerate".to_string(),
            curve,
            PositionalValueCurve::default(),
            1_700_000_000,
        );
        apply_to_profile(&art, &mut profile);
        assert!(profile.calibrated_trim_fraction.is_none(), "degenerate knee refused");
    }

    #[test]
    fn missing_artifact_is_no_op_via_store_lookup() {
        // The caller pattern: store.get(missing) -> None -> no apply.
        // This test pins the contract: a missing artifact MUST NOT mutate
        // the profile. The caller (recalibrate_budget_with_calibration)
        // is responsible for the fallback log.
        let store = CalibrationArtifactStore::new();
        let mut profile = ModelCapabilityProfile::micro_7b_12gb();
        let before = profile.clone();
        let art = store.get("never-inserted");
        assert!(art.is_none());
        if let Some(a) = art {
            apply_to_profile(a, &mut profile);
        }
        assert_eq!(profile, before, "missing artifact = no mutation");
    }
}
