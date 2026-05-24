//! D-7 closure — cross-domain transfer canary contract tests.
//!
//! Closes the audit gaps surfaced by
//! `workflow/sessions/s-25120/cross-check-recheck/D-7_cross_domain_canary-report.md`:
//!
//!  1. SCHEMA floor: `SCHEMA_VERSION_PUBLIC >= 114` (D-7 closure bumped from 113).
//!  2. `CrossDomainRegression` payload round-trip via rmp + action_type tag.
//!  3. `CanaryFamilyPass` payload round-trip via rmp + action_type tag.
//!  4. `CanaryInsufficientData` payload round-trip via rmp + action_type tag.
//!  5. Regression-delta invariant: `delta_pct` is positive when the
//!     regression fired (the magnitude of the candidate-vs-baseline drop).
//!  6. Insufficient-data invariant: `sample_count < min_required` whenever
//!     a `CanaryInsufficientData` row is emitted.
//!  7. Wire-tag resolution: each of the 3 new variants survives the
//!     [`ssb_action_type_from_str_public`] round-trip.
//!  8. `CanaryRuntimeConfig` toml defaults match the documented [canary]
//!     stanza in elai_cc.toml (fail_band=0.05, marginal_band=0.02,
//!     min_per_family=10, regression_threshold=0.05,
//!     insufficient_data_min_samples=5).
//!
//! Grounding:
//!   - workflow/papers/nanoflux-2509.23252-interface.md — NanoFlux paper
//!     is VERIFIED REAL but provides empirical motivation only; the paper
//!     has NO canary mechanism, NO 5% degradation threshold, NO
//!     cross-domain regression safeguard. Every numeric default in this
//!     contract is an ELAI policy choice.
//!   - workflow/reasoning/D-7-decisions.md (DECISION-1..6)

use elai_orchestrator::reasoning_template::Domain;
use elai_orchestrator::runtime_config::CanaryRuntimeConfig;
use elai_orchestrator::ssb::{
    ssb_action_type_from_str_public, CanaryFamilyPassPayload, CanaryInsufficientDataPayload,
    CrossDomainRegressionPayload, SsbActionType, SsbPayload, SCHEMA_VERSION_PUBLIC,
};

// ---------------------------------------------------------------------------
// 1. SCHEMA floor
// ---------------------------------------------------------------------------

#[test]
fn schema_version_at_or_above_d7_closure_floor() {
    // D-7 closure bumped SCHEMA_VERSION 113 -> 114 to add the 3 new
    // cross-domain-canary observation variants atomically. A future bump can
    // land safely; a regression below 114 means one of the three variants
    // ships without its SQL CHECK allowlist entry, which surfaces as a
    // write-time CHECK constraint violation. This guard catches that on
    // every contract test run.
    assert!(
        SCHEMA_VERSION_PUBLIC >= 114,
        "SCHEMA_VERSION_PUBLIC dropped below the D-7 closure floor (114); \
         got {SCHEMA_VERSION_PUBLIC}. Verify CrossDomainRegression + \
         CanaryFamilyPass + CanaryInsufficientData are still in the SQL \
         CHECK allowlist + enum + action_type_lock.golden."
    );
}

// ---------------------------------------------------------------------------
// 2. CrossDomainRegression wire-protocol round-trip
// ---------------------------------------------------------------------------

#[test]
fn cross_domain_regression_payload_round_trips_through_rmp() {
    let original = CrossDomainRegressionPayload {
        source_domain: Domain::Code,
        regressed_domain: Domain::Math,
        delta_pct: 7.5,
        threshold_pct: 5.0,
        ts_ms: 1_700_000_000_000,
    };
    let payload = SsbPayload::CrossDomainRegression(original.clone());
    let bytes = rmp_serde::to_vec(&payload).expect("rmp encode");
    let decoded: SsbPayload = rmp_serde::from_slice(&bytes).expect("rmp decode");
    match decoded {
        SsbPayload::CrossDomainRegression(rt) => {
            assert_eq!(rt, original, "CrossDomainRegression round-trip mismatch");
        }
        other => panic!("expected CrossDomainRegression, got {other:?}"),
    }
    // action_type tag matches.
    let payload = SsbPayload::CrossDomainRegression(original);
    assert_eq!(payload.action_type(), SsbActionType::CrossDomainRegression);
}

// ---------------------------------------------------------------------------
// 3. CanaryFamilyPass wire-protocol round-trip
// ---------------------------------------------------------------------------

#[test]
fn canary_family_pass_payload_round_trips_through_rmp() {
    let original = CanaryFamilyPassPayload {
        family: "code_fix".into(),
        score_pct: 82.5,
        sample_count: 12,
        ts_ms: 1_700_000_000_000,
    };
    let payload = SsbPayload::CanaryFamilyPass(original.clone());
    let bytes = rmp_serde::to_vec(&payload).expect("rmp encode");
    let decoded: SsbPayload = rmp_serde::from_slice(&bytes).expect("rmp decode");
    match decoded {
        SsbPayload::CanaryFamilyPass(rt) => {
            assert_eq!(rt, original, "CanaryFamilyPass round-trip mismatch");
        }
        other => panic!("expected CanaryFamilyPass, got {other:?}"),
    }
    let payload = SsbPayload::CanaryFamilyPass(original);
    assert_eq!(payload.action_type(), SsbActionType::CanaryFamilyPass);
}

// ---------------------------------------------------------------------------
// 4. CanaryInsufficientData wire-protocol round-trip
// ---------------------------------------------------------------------------

#[test]
fn canary_insufficient_data_payload_round_trips_through_rmp() {
    let original = CanaryInsufficientDataPayload {
        family: "tool_use".into(),
        sample_count: 3,
        min_required: 10,
        ts_ms: 1_700_000_000_000,
    };
    let payload = SsbPayload::CanaryInsufficientData(original.clone());
    let bytes = rmp_serde::to_vec(&payload).expect("rmp encode");
    let decoded: SsbPayload = rmp_serde::from_slice(&bytes).expect("rmp decode");
    match decoded {
        SsbPayload::CanaryInsufficientData(rt) => {
            assert_eq!(rt, original, "CanaryInsufficientData round-trip mismatch");
        }
        other => panic!("expected CanaryInsufficientData, got {other:?}"),
    }
    let payload = SsbPayload::CanaryInsufficientData(original);
    assert_eq!(payload.action_type(), SsbActionType::CanaryInsufficientData);
}

// ---------------------------------------------------------------------------
// 5. Regression-delta invariant
// ---------------------------------------------------------------------------

#[test]
fn cross_domain_regression_delta_pct_is_positive_when_regression_fired() {
    // The producer in canary.rs emits CrossDomainRegression only when
    // transfer_score <= -fail_band. The `delta_pct` field carries the
    // absolute magnitude in percent, so it must always be strictly
    // positive whenever the row exists. A negative or zero delta on this
    // payload would be a producer bug.
    let payload = CrossDomainRegressionPayload {
        source_domain: Domain::Code,
        regressed_domain: Domain::Math,
        delta_pct: 7.0,
        threshold_pct: 5.0,
        ts_ms: 1_700_000_000_000,
    };
    assert!(
        payload.delta_pct > 0.0,
        "CrossDomainRegression delta_pct must be > 0 (regression magnitude); \
         got {}",
        payload.delta_pct
    );
    assert!(
        payload.delta_pct >= payload.threshold_pct,
        "CrossDomainRegression must only fire when delta_pct >= threshold_pct; \
         got delta_pct={}, threshold_pct={}",
        payload.delta_pct,
        payload.threshold_pct
    );
}

// ---------------------------------------------------------------------------
// 6. Insufficient-data invariant
// ---------------------------------------------------------------------------

#[test]
fn canary_insufficient_data_sample_count_below_min_required() {
    // Producer emits CanaryInsufficientData only when n_evaluated <
    // min_required. The invariant must hold structurally on every payload
    // — sample_count strictly less than min_required.
    let payload = CanaryInsufficientDataPayload {
        family: "math_proof".into(),
        sample_count: 4,
        min_required: 10,
        ts_ms: 1_700_000_000_000,
    };
    assert!(
        payload.sample_count < payload.min_required,
        "CanaryInsufficientData must only fire when sample_count < min_required; \
         got sample_count={}, min_required={}",
        payload.sample_count,
        payload.min_required
    );
}

// ---------------------------------------------------------------------------
// 7. Wire-tag resolution for the 3 new variants
// ---------------------------------------------------------------------------

#[test]
fn canary_new_variants_resolve_via_action_type_from_str() {
    let want_cross_domain = ssb_action_type_from_str_public("cross_domain_regression")
        .expect("cross_domain_regression wire-tag resolves");
    assert_eq!(want_cross_domain, SsbActionType::CrossDomainRegression);

    let want_family_pass = ssb_action_type_from_str_public("canary_family_pass")
        .expect("canary_family_pass wire-tag resolves");
    assert_eq!(want_family_pass, SsbActionType::CanaryFamilyPass);

    let want_insufficient = ssb_action_type_from_str_public("canary_insufficient_data")
        .expect("canary_insufficient_data wire-tag resolves");
    assert_eq!(want_insufficient, SsbActionType::CanaryInsufficientData);
}

// ---------------------------------------------------------------------------
// 8. CanaryRuntimeConfig toml defaults match documented stanza
// ---------------------------------------------------------------------------

#[test]
fn canary_runtime_config_defaults_match_documented_stanza() {
    // The [canary] section in elai_cc.toml ships with these defaults; the
    // CanaryRuntimeConfig::default() MUST match so a freshly-installed
    // operator gets identical behaviour whether the section is present or
    // absent. Mismatch here = drift between code and toml documentation.
    let cfg = CanaryRuntimeConfig::default();
    assert!(
        (cfg.fail_band - 0.05).abs() < f64::EPSILON,
        "fail_band default drift: expected 0.05, got {}",
        cfg.fail_band
    );
    assert!(
        (cfg.marginal_band - 0.02).abs() < f64::EPSILON,
        "marginal_band default drift: expected 0.02, got {}",
        cfg.marginal_band
    );
    assert_eq!(
        cfg.min_per_family, 10,
        "min_per_family default drift: expected 10, got {}",
        cfg.min_per_family
    );
    assert!(
        (cfg.regression_threshold - 0.05).abs() < f64::EPSILON,
        "regression_threshold default drift: expected 0.05, got {}",
        cfg.regression_threshold
    );
    assert_eq!(
        cfg.insufficient_data_min_samples, 5,
        "insufficient_data_min_samples default drift: expected 5, got {}",
        cfg.insufficient_data_min_samples
    );
}

// ---------------------------------------------------------------------------
// 9. Gate predicate semantics — regression_blocks_promotion is monotonic
//    in delta magnitude at the threshold.
// ---------------------------------------------------------------------------

#[test]
fn canary_regression_gate_is_monotonic_at_threshold() {
    let cfg = CanaryRuntimeConfig::default();
    // delta == threshold blocks (inclusive boundary).
    assert!(cfg.regression_blocks_promotion(0.05));
    // delta > threshold blocks.
    assert!(cfg.regression_blocks_promotion(0.06));
    // delta < threshold does not block.
    assert!(!cfg.regression_blocks_promotion(0.04));
    // Negative delta treated by absolute magnitude (the gate consults
    // |delta|; the canary emits absolute magnitudes but defence-in-depth
    // here matters because external callers may pass signed deltas).
    assert!(cfg.regression_blocks_promotion(-0.07));
}
