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
use crate::ssb::{
    ContextCalibrationEventPayload, ContextCalibrationKind, Ssb, SsbPayload,
};
use std::collections::HashMap;
use std::sync::Arc;

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
    /// On insert, emits a `ContextCalibrationEvent(CurveMeasured)` SSB row
    /// when an `Ssb` handle is available so the bench-produced curve is
    /// observable off-agent (J-34 slice-2 production-wiring seam). The SSB
    /// writer is resolved via [`crate::lock::try_ssb_writer`]: the
    /// dashboard / CLI bootstrap installs it once via
    /// [`crate::lock::install_ssb_writer`], after which every `insert`
    /// (including the bench arm) threads `Some(&ssb)` automatically. When no
    /// writer is installed (cold start before SSB init, contract tests) the
    /// emit is a no-op. The emit is best-effort: a failed write is logged
    /// and never propagates, since a missing observability row is not a
    /// correctness failure (the artifact is still stored in-process).
    pub fn insert(&mut self, artifact: CalibrationArtifact) {
        let ssb = crate::lock::try_ssb_writer();
        self.insert_with_ssb(artifact, ssb.as_ref());
    }

    /// Insert with an explicit SSB handle for the CurveMeasured emit. The
    /// `insert` wrapper resolves the process-global writer via
    /// [`crate::lock::try_ssb_writer`] (so production callers do not need to
    /// thread it); this entry point is for callers that already hold an
    /// `Arc<Ssb>` (e.g. the in-module tests) and for the
    /// `recalibrate_budget_with_calibration` leg-2 seam, which threads the
    /// same writer it was passed.
    pub fn insert_with_ssb(
        &mut self,
        artifact: CalibrationArtifact,
        ssb: Option<&Arc<Ssb>>,
    ) {
        if let Some(ssb_arc) = ssb {
            emit_context_calibration_event(
                ssb_arc,
                ContextCalibrationKind::CurveMeasured,
                &artifact.identity_key,
                curve_measured_details(&artifact),
            );
        }
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
// SSB emit helper (J-34 slice-2 production-wiring seam)
// ---------------------------------------------------------------------------

/// Build the `details_json` for a `CurveMeasured` emit from an artifact.
/// Carries `sample_count`, `knee_fill_fraction`, `drop_magnitude`, and the
/// `content_hash` so the dashboard tile (leg-5) and the journal-regression
/// leg (leg-4) can recompute without a separate store lookup.
#[must_use]
pub(crate) fn curve_measured_details(artifact: &CalibrationArtifact) -> serde_json::Value {
    let (knee_fill, drop_mag) = match &artifact.knee {
        Some(k) => (Some(k.knee_fill_fraction), Some(k.drop_magnitude)),
        None => (None, None),
    };
    serde_json::json!({
        "sample_count": artifact.fill_curve.samples.len(),
        "knee_fill_fraction": knee_fill,
        "drop_magnitude": drop_mag,
        "content_hash": artifact.content_hash,
        "produced_at_secs": artifact.produced_at_secs,
    })
}

/// Build the `details_json` for a `ThresholdApplied` emit from
/// `recalibrate_budget_with_calibration`. Carries the `calibrated_trim_fraction`
/// that was applied (or `null` when the fallback ran) and `fallback_used`
/// so the NEGATIVE contract (no silent pseudo-calibration) is observable
/// off-agent via SSB. Mirrors `curve_measured_details` in shape.
#[must_use]
pub(crate) fn threshold_applied_details(
    calibrated_trim_fraction: Option<f64>,
    fallback_used: bool,
) -> serde_json::Value {
    serde_json::json!({
        "calibrated_trim_fraction": calibrated_trim_fraction,
        "fallback_used": fallback_used,
    })
}

/// Emit a `ContextCalibrationEvent` SSB row best-effort from a sync context.
/// Mirrors `front_door_emit::emit_intent_routed`: a missed SSB row is never a
/// correctness failure (the artifact is already stored in-process). When no
/// tokio runtime is available, a transient current-thread runtime is built so
/// the row still lands (do NOT spawn-and-drop).
pub fn emit_context_calibration_event(
    ssb: &Arc<Ssb>,
    kind: ContextCalibrationKind,
    model_identity_key: &str,
    details_json: serde_json::Value,
) {
    let ts_ms = crate::ssb::system_time_ms_i64();
    let payload = SsbPayload::ContextCalibrationEvent(ContextCalibrationEventPayload {
        kind,
        model_identity_key: model_identity_key.to_owned(),
        details_json,
        ts_ms,
    });
    let ssb = Arc::clone(ssb);
    let agent_id = "context_calibration";
    let emit = async move {
        if let Err(e) = ssb.write_payload(agent_id, 0, payload, vec![]).await {
            tracing::warn!(
                context_calibration.ssb_err = %e,
                "ContextCalibrationEvent SSB emit failed (calibration unaffected)"
            );
        }
    };
    match tokio::runtime::Handle::try_current() {
        Ok(handle) => {
            handle.spawn(emit);
        }
        Err(_) => {
            if let Ok(rt) = tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
            {
                rt.block_on(emit);
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::context_budget::ModelCapabilityProfile;
    use crate::ssb::{
        decode_ssb_payload, SsbActionType, SsbConfig, SsbReader,
    };
    use elai_core::clock::{ClockProvider, OsClock};
    use std::sync::Arc;
    use tempfile::TempDir;

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

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn j34_slice2_curve_measured_row_lands_on_insert_with_ssb() {
        let dir = TempDir::new().expect("tempdir");
        let clock: Arc<dyn ClockProvider> = Arc::new(OsClock);
        let ssb = Arc::new(
            Ssb::new(
                SsbConfig {
                    db_path: dir.path().join("j34-slice2-ssb.db"),
                    pool_size: Some(2),
                    project_id: "j34-slice2".into(),
                    session_id: "j34-slice2".into(),
                },
                clock,
            )
            .expect("ssb open"),
        );

        let mut store = CalibrationArtifactStore::new();
        let art = CalibrationArtifact::from_curves(
            "id-j34-slice2".to_string(),
            monotone_curve(),
            PositionalValueCurve::default(),
            1_700_000_000,
        );
        store.insert_with_ssb(art.clone(), Some(&ssb));

        tokio::time::sleep(std::time::Duration::from_millis(80)).await;

        let rows = ssb
            .query_by_type(SsbActionType::ContextCalibrationEvent(
                ContextCalibrationKind::CurveMeasured,
            ))
            .expect("query_by_type ContextCalibrationEvent");
        assert!(
            !rows.is_empty(),
            "expected >= 1 ContextCalibrationEvent row; R1 missing evidence = FAIL"
        );
        let decoded = decode_ssb_payload(&rows[0].payload).expect("decode_ssb_payload");
        match decoded {
            SsbPayload::ContextCalibrationEvent(p) => {
                assert_eq!(p.kind, ContextCalibrationKind::CurveMeasured);
                assert_eq!(p.model_identity_key, "id-j34-slice2");
                assert!(p.ts_ms > 0);
                let details = p.details_json.as_object().expect("details_json object");
                assert_eq!(details["sample_count"], 6);
                assert!(details["content_hash"].is_string());
                assert!(details["knee_fill_fraction"].is_number());
            }
            other => panic!("expected ContextCalibrationEvent; got {:?}", other),
        }
        // The artifact is also stored in-process (emit does not replace store).
        assert_eq!(store.len(), 1);
    }

    #[test]
    fn insert_without_ssb_does_not_emit_or_panic() {
        let mut store = CalibrationArtifactStore::new();
        let art = CalibrationArtifact::from_curves(
            "id-no-ssb".to_string(),
            monotone_curve(),
            PositionalValueCurve::default(),
            1_700_000_000,
        );
        store.insert(art);
        assert_eq!(store.len(), 1);
    }

    // J-34 slice-3 — ThresholdApplied emit from recalibrate_budget_with_calibration.

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn j34_slice3_threshold_applied_row_lands_on_recalibrate_with_knee() {
        use crate::context_budget::{
            CumulativeTokenTracker, ModelCapabilityProfile,
        };
        let dir = TempDir::new().expect("tempdir");
        let clock: Arc<dyn ClockProvider> = Arc::new(OsClock);
        let ssb = Arc::new(
            Ssb::new(
                SsbConfig {
                    db_path: dir.path().join("j34-slice3-ssb.db"),
                    pool_size: Some(2),
                    project_id: "j34-slice3".into(),
                    session_id: "j34-slice3".into(),
                },
                clock,
            )
            .expect("ssb open"),
        );

        let mut profile = ModelCapabilityProfile::micro_7b_12gb();
        let art = CalibrationArtifact::from_curves(
            "id-j34-slice3".to_string(),
            monotone_curve(),
            PositionalValueCurve::default(),
            1_700_000_000,
        );
        let tracker = CumulativeTokenTracker::new();
        tracker.recalibrate_budget_with_calibration(
            &mut profile,
            None,
            None,
            Some(&art),
            Some(&ssb),
        );

        tokio::time::sleep(std::time::Duration::from_millis(80)).await;

        let rows = ssb
            .query_by_type(SsbActionType::ContextCalibrationEvent(
                ContextCalibrationKind::ThresholdApplied,
            ))
            .expect("query_by_type ThresholdApplied");
        assert!(
            !rows.is_empty(),
            "expected >= 1 ThresholdApplied row; R1 missing evidence = FAIL"
        );
        let decoded = decode_ssb_payload(&rows[0].payload).expect("decode_ssb_payload");
        match decoded {
            SsbPayload::ContextCalibrationEvent(p) => {
                assert_eq!(p.kind, ContextCalibrationKind::ThresholdApplied);
                assert_eq!(p.model_identity_key, "id-j34-slice3");
                let details = p.details_json.as_object().expect("details_json object");
                // calibrated_trim_fraction must be the knee fill (0.50) and
                // fallback_used must be false (knee was applied).
                let frac = details["calibrated_trim_fraction"]
                    .as_f64()
                    .expect("calibrated_trim_fraction is number");
                assert!((frac - 0.50).abs() < 1e-6, "knee fill 0.50, got {}", frac);
                assert_eq!(details["fallback_used"], false);
            }
            other => panic!("expected ContextCalibrationEvent; got {:?}", other),
        }
        // Profile was actually calibrated.
        let frac = profile.calibrated_trim_fraction.expect("knee applied");
        assert!((frac - 0.50).abs() < 1e-6);
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn j34_slice3_threshold_applied_row_lands_on_recalibrate_no_knee() {
        use crate::context_budget::{
            CumulativeTokenTracker, ModelCapabilityProfile,
        };
        let dir = TempDir::new().expect("tempdir");
        let clock: Arc<dyn ClockProvider> = Arc::new(OsClock);
        let ssb = Arc::new(
            Ssb::new(
                SsbConfig {
                    db_path: dir.path().join("j34-slice3-noknee-ssb.db"),
                    pool_size: Some(2),
                    project_id: "j34-slice3-noknee".into(),
                    session_id: "j34-slice3-noknee".into(),
                },
                clock,
            )
            .expect("ssb open"),
        );

        // Flat curve -> no knee -> calibrated_trim_fraction stays None.
        let flat = FillDegradationCurve {
            samples: vec![sample(0.1, 0.9), sample(0.5, 0.9), sample(0.9, 0.9)],
        };
        let art = CalibrationArtifact::from_curves(
            "id-j34-slice3-noknee".to_string(),
            flat,
            PositionalValueCurve::default(),
            1_700_000_000,
        );
        assert!(art.knee.is_none());

        let mut profile = ModelCapabilityProfile::micro_7b_12gb();
        let tracker = CumulativeTokenTracker::new();
        tracker.recalibrate_budget_with_calibration(
            &mut profile,
            None,
            None,
            Some(&art),
            Some(&ssb),
        );

        tokio::time::sleep(std::time::Duration::from_millis(80)).await;

        let rows = ssb
            .query_by_type(SsbActionType::ContextCalibrationEvent(
                ContextCalibrationKind::ThresholdApplied,
            ))
            .expect("query_by_type ThresholdApplied");
        assert!(
            !rows.is_empty(),
            "expected >= 1 ThresholdApplied row on no-knee path (NEGATIVE contract observable)"
        );
        let decoded = decode_ssb_payload(&rows[0].payload).expect("decode_ssb_payload");
        match decoded {
            SsbPayload::ContextCalibrationEvent(p) => {
                assert_eq!(p.kind, ContextCalibrationKind::ThresholdApplied);
                let details = p.details_json.as_object().expect("details_json object");
                assert!(
                    details["calibrated_trim_fraction"].is_null(),
                    "no knee -> null calibrated_trim_fraction"
                );
                assert_eq!(details["fallback_used"], true);
            }
            other => panic!("expected ContextCalibrationEvent; got {:?}", other),
        }
        assert!(profile.calibrated_trim_fraction.is_none(), "no pseudo-calibration");
    }

    #[test]
    fn threshold_applied_details_shape() {
        let details = threshold_applied_details(Some(0.5), false);
        let obj = details.as_object().expect("object");
        assert_eq!(obj["fallback_used"], false);
        assert!((obj["calibrated_trim_fraction"].as_f64().unwrap() - 0.5).abs() < 1e-9);
        let details_none = threshold_applied_details(None, true);
        let obj_none = details_none.as_object().expect("object");
        assert!(obj_none["calibrated_trim_fraction"].is_null());
        assert_eq!(obj_none["fallback_used"], true);
    }

    // Ensure slice-2 CurveMeasured insert test still PASS after slice-3
    // changed the recalibrate emit site (the insert path is separate).
    #[test]
    fn slice2_curve_measured_insert_still_emits() {
        let mut store = CalibrationArtifactStore::new();
        let art = CalibrationArtifact::from_curves(
            "id-slice3-regression".to_string(),
            monotone_curve(),
            PositionalValueCurve::default(),
            1_700_000_000,
        );
        store.insert(art);
        // No SSB in this test (insert without ssb arg) — just verify store path
        // is untouched and the artifact lands. The slice-2 SSB-backed test
        // above (j34_slice2_curve_measured_row_lands_on_insert_with_ssb) pins
        // the CurveMeasured row contract.
        assert_eq!(store.len(), 1);
    }
}
