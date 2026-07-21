//! J-34 leg-3 — PlacementApplied ordering_seed contract (OPTION A metadata seed).
//!
//! Verifies the ratified D-170 placement slice:
//! D1: `compute_ordering_seed` exists in context_calibration.rs and is
//!     re-exported / callable from the orchestrator public surface.
//! D2: `placement_applied_details` exists in context_calibration.rs.
//! D3: `ModelCapabilityProfile` has an `ordering_seed: Option<String>` field.
//! D4: `ContextBudget` has an `ordering_seed: Option<String>` field.
//! D5: `apply_to_profile` sets `profile.ordering_seed` from a curve with
//!     samples; leaves it `None` for an empty curve.
//! D6: `compute_with_profile` carries the seed onto `ContextBudget` and emits
//!     a `PlacementApplied` SSB row with `slot_count` + `ordering_seed` in
//!     `details_json` when the seed is `Some`.
//! D7: `budget.slots[0].slot` is still `ContextSlot::SystemPrompt` (fixed
//!     ordinals, NO reorder).
//! D8 (NEGATIVE): when `positional_value_curve` has zero samples,
//!     `apply_to_profile` sets `ordering_seed = None` and
//!     `compute_with_profile` emits ZERO `PlacementApplied` rows.

use std::sync::Arc;

use elai_core::clock::{ClockProvider, OsClock};
use elai_orchestrator::context_budget::{
    ContextBudget, ContextSlot, ModelCapabilityProfile, SLOT_COUNT,
};
use elai_orchestrator::context_calibration::{
    compute_ordering_seed, CalibrationArtifact, FillDegradationCurve, FillSample,
    PositionalSample, PositionalValueCurve,
};
use elai_orchestrator::lock::install_ssb_writer;
use elai_orchestrator::ssb::{
    decode_ssb_payload, ContextCalibrationKind, Ssb, SsbActionType,
    SsbConfig, SsbPayload, SsbReader,
};
use tempfile::TempDir;

/// Serializes tests that call `install_ssb_writer` (a process-global static).
/// Without this, the parallel test runner can swap the global SSB writer
/// mid-test, causing false-positive/false-negative on PlacementApplied rows.
/// Mirrors the `ENV_MUTEX` idiom in h51_handoff_lifecycle_contract.rs
/// (serial_test is not in dev-deps).
static SSB_WRITER_MUTEX: std::sync::Mutex<()> = std::sync::Mutex::new(());

fn five_sample_positional_curve() -> PositionalValueCurve {
    PositionalValueCurve {
        samples: vec![
            PositionalSample { depth_fraction: 0.0, recall_rate: 1.0 },
            PositionalSample { depth_fraction: 0.25, recall_rate: 0.85 },
            PositionalSample { depth_fraction: 0.5, recall_rate: 0.60 },
            PositionalSample { depth_fraction: 0.75, recall_rate: 0.30 },
            PositionalSample { depth_fraction: 1.0, recall_rate: 0.10 },
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

// ---------------------------------------------------------------------------
// D1: compute_ordering_seed exists and is callable
// ---------------------------------------------------------------------------

#[test]
fn d1_compute_ordering_seed_exists() {
    let curve = five_sample_positional_curve();
    let seed = compute_ordering_seed(&curve);
    assert!(seed.is_some(), "curve with samples must produce a seed");
    assert!(!seed.unwrap().is_empty(), "seed must be non-empty");
}

// ---------------------------------------------------------------------------
// D2: placement_applied_details exists (source-grep)
// ---------------------------------------------------------------------------

#[test]
fn d2_placement_applied_details_exists_in_source() {
    let src = std::fs::read_to_string(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../orchestrator/src/context_calibration.rs"
    ))
    .expect("read context_calibration.rs");
    assert!(
        src.contains("fn placement_applied_details"),
        "context_calibration.rs must define placement_applied_details"
    );
}

// ---------------------------------------------------------------------------
// D3: ModelCapabilityProfile has ordering_seed field (source-grep)
// ---------------------------------------------------------------------------

#[test]
fn d3_profile_has_ordering_seed_field() {
    let src = std::fs::read_to_string(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../orchestrator/src/context_budget.rs"
    ))
    .expect("read context_budget.rs");
    let struct_start = src.find("pub struct ModelCapabilityProfile");
    assert!(struct_start.is_some(), "ModelCapabilityProfile struct found");
    let after = &src[struct_start.unwrap()..];
    let struct_end = after.find("\n}").expect("struct end");
    let struct_body = &after[..struct_end];
    assert!(
        struct_body.contains("pub ordering_seed: Option<String>"),
        "ModelCapabilityProfile must have ordering_seed: Option<String>"
    );
}

// ---------------------------------------------------------------------------
// D4: ContextBudget has ordering_seed field (source-grep)
// ---------------------------------------------------------------------------

#[test]
fn d4_budget_has_ordering_seed_field() {
    let src = std::fs::read_to_string(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../orchestrator/src/context_budget.rs"
    ))
    .expect("read context_budget.rs");
    let struct_start = src.find("pub struct ContextBudget");
    assert!(struct_start.is_some(), "ContextBudget struct found");
    let after = &src[struct_start.unwrap()..];
    let struct_end = after.find("\n}").expect("struct end");
    let struct_body = &after[..struct_end];
    assert!(
        struct_body.contains("pub ordering_seed: Option<String>"),
        "ContextBudget must have ordering_seed: Option<String>"
    );
}

// ---------------------------------------------------------------------------
// D5: apply_to_profile sets ordering_seed from curve; None for empty curve
// ---------------------------------------------------------------------------

#[test]
fn d5a_apply_to_profile_sets_ordering_seed_from_curve() {
    let mut profile = ModelCapabilityProfile::micro_7b_12gb();
    assert!(profile.ordering_seed.is_none(), "fresh profile has no seed");
    let art = CalibrationArtifact::from_curves(
        "id-d5a".to_string(),
        monotone_fill_curve(),
        five_sample_positional_curve(),
        1_700_000_000,
    );
    elai_orchestrator::context_calibration::apply_to_profile(&art, &mut profile);
    assert!(
        profile.ordering_seed.is_some(),
        "apply_to_profile must set ordering_seed when curve has samples"
    );
    let seed = profile.ordering_seed.as_ref().unwrap();
    assert!(!seed.is_empty(), "seed must be non-empty");
}

#[test]
fn d5b_apply_to_profile_leaves_seed_none_for_empty_curve() {
    let mut profile = ModelCapabilityProfile::micro_7b_12gb();
    let art = CalibrationArtifact::from_curves(
        "id-d5b".to_string(),
        monotone_fill_curve(),
        PositionalValueCurve::default(),
        1_700_000_000,
    );
    elai_orchestrator::context_calibration::apply_to_profile(&art, &mut profile);
    assert!(
        profile.ordering_seed.is_none(),
        "empty curve must produce no seed"
    );
}

// ---------------------------------------------------------------------------
// D6+D7: compute_with_profile carries seed, emits PlacementApplied SSB,
//        and slots[0] is still SystemPrompt
// ---------------------------------------------------------------------------

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn d6_d7_placement_applied_row_lands_and_slots_unchanged() {
    let _ssb_guard = SSB_WRITER_MUTEX.lock().unwrap_or_else(|e| e.into_inner());
    let dir = TempDir::new().expect("tempdir");
    let clock: Arc<dyn ClockProvider> = Arc::new(OsClock);
    let ssb = Arc::new(
        Ssb::new(
            SsbConfig {
                db_path: dir.path().join("j34-placement-ssb.db"),
                pool_size: Some(2),
                project_id: "j34-placement".into(),
                session_id: "j34-placement".into(),
            },
            clock,
        )
        .expect("ssb open"),
    );
    install_ssb_writer(Arc::clone(&ssb));

    let mut profile = ModelCapabilityProfile::micro_7b_12gb();
    let art = CalibrationArtifact::from_curves(
        "id-j34-placement".to_string(),
        monotone_fill_curve(),
        five_sample_positional_curve(),
        1_700_000_000,
    );
    elai_orchestrator::context_calibration::apply_to_profile(&art, &mut profile);
    let seed = profile.ordering_seed.clone().expect("seed set by apply_to_profile");

    let budget = ContextBudget::compute_with_profile(32_768, [0, 0, 0], &profile)
        .expect("compute_with_profile succeeds");

    assert_eq!(
        budget.ordering_seed.as_deref(),
        Some(seed.as_str()),
        "budget carries the ordering seed"
    );
    assert_eq!(
        budget.slots[0].slot,
        ContextSlot::SystemPrompt,
        "slots[0] is still SystemPrompt (fixed ordinals, no reorder)"
    );

    tokio::time::sleep(std::time::Duration::from_millis(120)).await;

    let rows = ssb
        .query_by_type(SsbActionType::ContextCalibrationEvent(
            ContextCalibrationKind::PlacementApplied,
        ))
        .expect("query_by_type PlacementApplied");
    assert!(
        !rows.is_empty(),
        "expected >= 1 PlacementApplied row; R1 missing evidence = FAIL"
    );

    let mut found_valid = false;
    for row in &rows {
        let decoded = decode_ssb_payload(&row.payload).expect("decode_ssb_payload");
        if let SsbPayload::ContextCalibrationEvent(p) = decoded {
            if p.kind == ContextCalibrationKind::PlacementApplied {
                let details = p.details_json.as_object().expect("details_json object");
                assert_eq!(
                    details["slot_count"].as_u64(),
                    Some(SLOT_COUNT as u64),
                    "slot_count must be SLOT_COUNT"
                );
                assert_eq!(
                    details["ordering_seed"].as_str(),
                    Some(seed.as_str()),
                    "ordering_seed must match the profile seed"
                );
                found_valid = true;
            }
        }
    }
    assert!(found_valid, "no valid PlacementApplied row found");
}

// ---------------------------------------------------------------------------
// D8 (NEGATIVE): empty curve -> zero PlacementApplied rows
// ---------------------------------------------------------------------------

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn d8_negative_empty_curve_emits_zero_placement_applied() {
    let _ssb_guard = SSB_WRITER_MUTEX.lock().unwrap_or_else(|e| e.into_inner());
    let dir = TempDir::new().expect("tempdir");
    let clock: Arc<dyn ClockProvider> = Arc::new(OsClock);
    let ssb = Arc::new(
        Ssb::new(
            SsbConfig {
                db_path: dir.path().join("j34-placement-neg-ssb.db"),
                pool_size: Some(2),
                project_id: "j34-placement-neg".into(),
                session_id: "j34-placement-neg".into(),
            },
            clock,
        )
        .expect("ssb open"),
    );
    install_ssb_writer(Arc::clone(&ssb));

    let profile = ModelCapabilityProfile::micro_7b_12gb();
    assert!(
        profile.ordering_seed.is_none(),
        "fresh profile has no seed (no apply_to_profile called)"
    );

    let budget = ContextBudget::compute_with_profile(32_768, [0, 0, 0], &profile)
        .expect("compute_with_profile succeeds");
    assert!(
        budget.ordering_seed.is_none(),
        "budget has no seed when profile has no seed"
    );

    tokio::time::sleep(std::time::Duration::from_millis(120)).await;

    let rows = ssb
        .query_by_type(SsbActionType::ContextCalibrationEvent(
            ContextCalibrationKind::PlacementApplied,
        ))
        .expect("query_by_type PlacementApplied");
    assert!(
        rows.is_empty(),
        "NEGATIVE: zero PlacementApplied rows when curve is None"
    );
}

// ---------------------------------------------------------------------------
// D9: ordering_seed is deterministic (same curve -> same seed)
// ---------------------------------------------------------------------------

#[test]
fn d9_ordering_seed_is_deterministic() {
    let curve = five_sample_positional_curve();
    let seed1 = compute_ordering_seed(&curve);
    let seed2 = compute_ordering_seed(&curve);
    assert_eq!(seed1, seed2, "same curve must produce same seed");
}
