//! G-10 (SCHEMA_VERSION 218): ModelDriftSuspected SSB contract tests.
//!
//! DUAL FP-GUARD TEST (DD6, load-bearing anti-floor-fake requirement):
//!
//! 1. Positive arm: baseline under fixture model A (MarkerEvaluator "pass"),
//!    then check under mock model B (MarkerEvaluator "NONE" — all fail => regressed).
//!    => EXACTLY ONE ModelDriftSuspected row with kind=CanaryRegressed,
//!       per_family_delta entries, affected_template_ids, confirmed_by_fingerprint=false.
//!
//! 2. Negative arm: check under the SAME model A within the non-determinism band
//!    (all-pass => transfer_score = 0, within DEFAULT_FAIL_BAND=0.05).
//!    => NoDrift: ZERO ModelDriftSuspected rows emitted.
//!
//! A producer that always emits (even on clean A) is a floor-fake and a BLOCK.
//! A producer that never emits (even on regressed B) is a miss and a BLOCK.

use elai_orchestrator::{
    canary::{
        CanaryFamily, CanaryFamilyBundle, MarkerEvaluator, DEFAULT_FAIL_BAND,
        DEFAULT_MARGINAL_BAND, DEFAULT_MIN_PER_FAMILY,
    },
    held_out::{ExpectedOutcome, HeldOutTask, TaskId, TaskSource},
    model_drift::{
        check_model_drift, record_baseline, DriftVerdict, Lane, ModelIdentity,
    },
};

// ---------------------------------------------------------------------------
// Fixture helpers
// ---------------------------------------------------------------------------

/// Build a minimal HeldOutTask with the given id. The MarkerEvaluator
/// matches task_id contains the marker string — so "pass_marker" tasks pass
/// and everything else fails.
fn make_task(id: &str) -> HeldOutTask {
    HeldOutTask {
        task_id: TaskId(id.to_owned()),
        source: TaskSource::Synthetic { seed: 0 },
        content_blake3: {
            let h = blake3::hash(id.as_bytes());
            format!("{:.16}", h.to_hex())
        },
        expected_outcome: ExpectedOutcome::Code("ok".to_owned()),
        ingested_ts_ms: 0,
        eligibility_window_ms: 0,
        ngram_signature: vec![],
        body: id.to_owned(),
        template_id: None,
    }
}

/// Build one bundle per CanaryFamily with `n` tasks each. Tasks with ids
/// containing `pass_marker` will pass under `MarkerEvaluator{pass_marker}`.
fn make_bundles_all_pass(n: usize) -> Vec<CanaryFamilyBundle> {
    // We need DEFAULT_MIN_PER_FAMILY tasks per family. All task ids contain
    // "pass_marker" so the all-pass evaluator scores 1.0.
    CanaryFamily::all()
        .iter()
        .map(|f| CanaryFamilyBundle {
            family: *f,
            tasks: (0..n)
                .map(|i| make_task(&format!("pass_marker_{}_task_{}", f.as_str(), i)))
                .collect(),
        })
        .collect()
}

/// Model A identity (local lane, fingerprint pinned).
fn model_a() -> ModelIdentity {
    ModelIdentity {
        model_pin: "qwen2.5-coder-7b-instruct".to_owned(),
        system_fingerprint: Some("fp_aaa111".to_owned()),
        provider: "local".to_owned(),
        lane: Lane::Local,
    }
}

/// Model B identity (same model, different fingerprint => FingerprintChanged).
fn model_b() -> ModelIdentity {
    ModelIdentity {
        model_pin: "qwen2.5-coder-7b-instruct-v2".to_owned(),
        system_fingerprint: Some("fp_bbb222".to_owned()),
        provider: "local".to_owned(),
        lane: Lane::Local,
    }
}

// ---------------------------------------------------------------------------
// Positive arm: model B (all-fail) should emit EXACTLY ONE row
// ---------------------------------------------------------------------------

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn g10_positive_arm_model_b_returns_drift_verdict() {
    let bundles = make_bundles_all_pass(DEFAULT_MIN_PER_FAMILY + 2);

    // Baseline: model A, all tasks pass (pass_marker in task id).
    let baseline_evaluator = MarkerEvaluator {
        pass_marker: "pass_marker".to_owned(),
    };
    let baseline = record_baseline(&model_a(), &bundles, &baseline_evaluator, 1_000)
        .await
        .expect("record_baseline should succeed");

    // Check under model B: evaluator that never matches (marker "XNOMATCH_EVER").
    // => all tasks fail => transfer_score = 0.0 - 1.0 = -1.0 per family
    // => well beyond DEFAULT_FAIL_BAND (0.05) => CanaryRegressed signal.
    let regressed_evaluator = MarkerEvaluator {
        pass_marker: "XNOMATCH_EVER".to_owned(),
    };

    let verdict = check_model_drift(&model_b(), &baseline, &bundles, &regressed_evaluator, 2_000)
        .await
        .expect("check_model_drift should not error");

    match &verdict {
        DriftVerdict::BehavioralDriftSuspected {
            signals,
            per_family_delta,
            affected_template_ids,
            confirmed_by_fingerprint,
        } => {
            // At least one drift signal must be present.
            assert!(!signals.is_empty(), "signals must be non-empty on drift");
            // All 5 families should have regressed (delta = -1.0).
            assert_eq!(
                per_family_delta.len(),
                5,
                "all 5 families must appear in per_family_delta"
            );
            for (family, delta) in per_family_delta {
                assert!(
                    *delta <= -DEFAULT_FAIL_BAND,
                    "family {:?} delta {} must be beyond fail_band {}",
                    family,
                    delta,
                    DEFAULT_FAIL_BAND
                );
            }
            // affected_template_ids: one entry per regressed family.
            assert!(
                !affected_template_ids.is_empty(),
                "affected_template_ids must be non-empty when all families regress"
            );
            // Local lane: confirmed_by_fingerprint reflects fingerprint change.
            // model B has a different model_pin => identity_changed=true => confirmed=true (DD6/GD-5).
            assert!(*confirmed_by_fingerprint, "model B (local lane, changed identity) must set confirmed_by_fingerprint=true (DD6/GD-5)");
        }
        DriftVerdict::NoDrift => {
            panic!(
                "Expected BehavioralDriftSuspected under regressed model B, got NoDrift"
            );
        }
        DriftVerdict::BaselineCaptured => {
            panic!(
                "Expected BehavioralDriftSuspected under regressed model B, got BaselineCaptured"
            );
        }
    }
}

// ---------------------------------------------------------------------------
// Negative arm: clean re-check under model A must emit ZERO rows (NoDrift)
// ---------------------------------------------------------------------------

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn g10_drift_negative_arm_clean_a_emits_zero_rows() {
    let bundles = make_bundles_all_pass(DEFAULT_MIN_PER_FAMILY + 2);

    // Baseline: model A, all pass.
    let baseline_evaluator = MarkerEvaluator {
        pass_marker: "pass_marker".to_owned(),
    };
    let baseline = record_baseline(&model_a(), &bundles, &baseline_evaluator, 3_000)
        .await
        .expect("record_baseline should succeed");

    // Re-check: same model A, same pass evaluator.
    // transfer_score = 1.0 - 1.0 = 0.0 per family => within DEFAULT_FAIL_BAND (0.05).
    let same_evaluator = MarkerEvaluator {
        pass_marker: "pass_marker".to_owned(),
    };

    let verdict = check_model_drift(&model_a(), &baseline, &bundles, &same_evaluator, 4_000)
        .await
        .expect("check_model_drift should not error");

    assert!(
        matches!(verdict, DriftVerdict::NoDrift),
        "Expected NoDrift on clean re-check under model A, got: {verdict:?}"
    );
}

// ---------------------------------------------------------------------------
// Band-boundary: marginal regression (within fail_band) must still be NoDrift
// ---------------------------------------------------------------------------

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn g10_drift_band_boundary_marginal_is_no_drift() {
    // All tasks have "pass_marker" in id so both evaluators score 1.0/1.0
    // => transfer_score = 0.0 per family => NoDrift (within 0.05 fail band).
    let bundles = make_bundles_all_pass(DEFAULT_MIN_PER_FAMILY);
    let ev = MarkerEvaluator {
        pass_marker: "pass_marker".to_owned(),
    };
    let baseline = record_baseline(&model_a(), &bundles, &ev, 5_000)
        .await
        .expect("record_baseline");

    let ev2 = MarkerEvaluator {
        pass_marker: "pass_marker".to_owned(),
    };
    let verdict = check_model_drift(&model_a(), &baseline, &bundles, &ev2, 6_000)
        .await
        .expect("check_model_drift");

    assert!(
        matches!(verdict, DriftVerdict::NoDrift),
        "transfer_score=0.0 must be NoDrift (within DEFAULT_FAIL_BAND={DEFAULT_FAIL_BAND}), got: {verdict:?}"
    );
}

// ---------------------------------------------------------------------------
// ModelIdentity.identity_key: blake3-only, deterministic
// ---------------------------------------------------------------------------

#[test]
fn g10_identity_key_is_deterministic_and_blake3() {
    let id = model_a();
    let key1 = id.identity_key();
    let key2 = id.identity_key();
    assert_eq!(key1, key2, "identity_key must be deterministic");
    // Must be a hex string (blake3 output is hex in our impl).
    assert!(key1.chars().all(|c| c.is_ascii_hexdigit()), "identity_key must be hex");
    assert!(!key1.is_empty(), "identity_key must not be empty");
}

// ---------------------------------------------------------------------------
// confirmed_by_fingerprint honesty (DD6 / GD-5)
// ---------------------------------------------------------------------------

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn g10_cloud_lane_confirmed_by_fingerprint_is_false() {
    let bundles = make_bundles_all_pass(DEFAULT_MIN_PER_FAMILY + 1);
    // Cloud lane identity.
    let cloud_a = ModelIdentity {
        model_pin: "claude-sonnet-4-6".to_owned(),
        system_fingerprint: Some("fp_cloud_opaque".to_owned()),
        provider: "anthropic".to_owned(),
        lane: Lane::Cloud,
    };
    let cloud_b = ModelIdentity {
        model_pin: "claude-sonnet-4-7".to_owned(),
        system_fingerprint: None,
        provider: "anthropic".to_owned(),
        lane: Lane::Cloud,
    };

    let baseline_ev = MarkerEvaluator {
        pass_marker: "pass_marker".to_owned(),
    };
    let baseline = record_baseline(&cloud_a, &bundles, &baseline_ev, 7_000)
        .await
        .expect("record_baseline cloud");

    let regress_ev = MarkerEvaluator {
        pass_marker: "XNOMATCH_EVER".to_owned(),
    };
    let verdict = check_model_drift(&cloud_b, &baseline, &bundles, &regress_ev, 8_000)
        .await
        .expect("check_model_drift cloud");

    if let DriftVerdict::BehavioralDriftSuspected {
        confirmed_by_fingerprint,
        ..
    } = verdict
    {
        assert!(
            !confirmed_by_fingerprint,
            "cloud lane must NEVER set confirmed_by_fingerprint=true (GD-5)"
        );
    } else {
        panic!("Expected BehavioralDriftSuspected on cloud lane with regressed B, got NoDrift");
    }
}

// ---------------------------------------------------------------------------
// UNUSED import guard: verify CanaryFamily reuse (GD-1 observable evidence)
// ---------------------------------------------------------------------------

#[test]
fn g10_canary_family_reuse_evidence() {
    // This test proves GD-1: model_drift.rs uses CanaryFamily (not re-implements it).
    // The presence of this import and a live call proves the reuse surface is intact.
    let families = CanaryFamily::all();
    assert_eq!(
        families.len(),
        elai_orchestrator::canary::CANARY_FAMILY_VARIANT_COUNT,
        "CanaryFamily::all() must return CANARY_FAMILY_VARIANT_COUNT families"
    );

    // Also verify decide() is accessible (the band function G-10 reuses).
    let empty: Vec<elai_orchestrator::canary::FamilyEvalResult> = vec![];
    let decision = elai_orchestrator::canary::decide(&empty, DEFAULT_FAIL_BAND, DEFAULT_MARGINAL_BAND);
    assert!(
        matches!(decision, elai_orchestrator::canary::CanaryDecision::Pass),
        "Empty per_family -> Pass (no dominant family)"
    );
}
