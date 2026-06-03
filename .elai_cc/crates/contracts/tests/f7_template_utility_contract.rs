//! F-7 / Slice A1 TemplateUtilityRanked SSB round-trip contract tests.
//!
//! Validates the CBR template Laplace-utility-ranking beacon (SCHEMA_VERSION
//! 205, HALF B only):
//!   1. `TemplateUtilityRankedPayload` survives the EXACT on-wire encoder
//!      `Ssb::write_payload` uses (`rmp_serde::to_vec_named`) -> `from_slice`,
//!      with all six fields intact (concept_class / plan_skeleton_hash /
//!      utility / first_try_ratio / sample_n / chosen_pipeline_id).
//!   2. The wrapped payload reports `action_type() == TemplateUtilityRanked`
//!      and sql_str `"template_utility_ranked"`.
//!   3. `ssb_action_type_from_str_public("template_utility_ranked")` maps back
//!      to the flat-beacon variant and its sql_str round-trips stably.
//!   4. The full DB path: write through `Ssb::write_payload`, recover via
//!      `query_by_type`, decode the stored bytes the way the dashboard/CLI
//!      readers do, and assert every field survives.
//!   5. Laplace-smoothing correctness: `utility = (successes + ALPHA) /
//!      (total + ALPHA + BETA)` with `ALPHA = BETA = 1.0`, so a 1/1 template
//!      scores `2/3` (not `1.0`) and a 95/100 template scores `~0.941`. This is
//!      the F-7 gate: smoothing-correctness, NOT any numeric uplift target (the
//!      Springdrift +10.7% figure is UNGROUNDED and is deliberately not encoded
//!      anywhere here).
//!
//! POLICY note: TemplateUtilityRanked is observe/record-only -- it records the
//! score that gates crystallized-template reuse at the `try_execute` seam, never
//! a hard BLOCK. `block_ratio` + `coverage` are intentionally ABSENT (no
//! producer; zero-filling would be a CSQ footgun).

use std::sync::Arc;

use elai_core::clock::OsClock;
use elai_orchestrator::ssb::{
    ssb_action_type_from_str_public, Ssb, SsbActionType, SsbConfig, SsbPayload, SsbReader,
    TemplateUtilityRankedPayload,
};
use elai_orchestrator::template_utility::{
    laplace_utility as module_laplace_utility, laplace_utility_from_fields, TemplateUtility,
    LAPLACE_ALPHA, LAPLACE_BETA,
};
use tempfile::TempDir;

// ---------------------------------------------------------------------------
// Laplace add-one (ALPHA = BETA = 1.0) smoothing reference. ELAI-owned
// constants -- kept local so the contract test does not depend on the producer
// module location (which lives behind the try_execute seam in a later slice).
// ---------------------------------------------------------------------------

const ALPHA: f32 = 1.0;
const BETA: f32 = 1.0;

fn laplace_utility(successes: u32, total: u32) -> f32 {
    (successes as f32 + ALPHA) / (total as f32 + ALPHA + BETA)
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

async fn make_ssb(tmp: &TempDir, project: &str, session: &str) -> Arc<Ssb> {
    let cfg = SsbConfig {
        db_path: tmp.path().join("ssb.db"),
        pool_size: Some(2),
        project_id: project.into(),
        session_id: session.into(),
    };
    let clock: Arc<dyn elai_core::clock::ClockProvider> = Arc::new(OsClock);
    Arc::new(Ssb::new(cfg, clock).expect("ssb new"))
}

/// A representative beacon payload. `utility` is the Laplace-smoothed score over
/// 1 success / 1 firing (-> `2/3`), `first_try_ratio` keeps the raw `1.0`
/// baseline (W-2 no-removal), and `plan_skeleton_hash` is a non-trivial u64 to
/// expose any width truncation on the wire. Strings are distinct so a
/// concept_class<->chosen_pipeline_id swap would surface.
fn sample_template_utility_payload() -> TemplateUtilityRankedPayload {
    TemplateUtilityRankedPayload {
        concept_class: "rest_crud_api".to_string(),
        plan_skeleton_hash: 0x5f3e_27a1_0b9c_d4e2,
        utility: laplace_utility(1, 1),
        first_try_ratio: 1.0,
        sample_n: 1,
        chosen_pipeline_id: "pipeline-contract-0".to_string(),
    }
}

// ---------------------------------------------------------------------------
// Contract 1: on-wire encoder round-trip (rmp_serde::to_vec_named), all 6 fields
// ---------------------------------------------------------------------------

#[test]
fn contract_template_utility_wire_encoder_roundtrip() {
    let original = sample_template_utility_payload();
    let wrapped = SsbPayload::TemplateUtilityRanked(original.clone());

    assert_eq!(
        wrapped.action_type(),
        SsbActionType::TemplateUtilityRanked,
        "TemplateUtilityRanked payload action_type must be TemplateUtilityRanked"
    );
    assert_eq!(
        wrapped.action_type().as_sql_str(),
        "template_utility_ranked",
        "TemplateUtilityRanked sql_str must be template_utility_ranked"
    );

    // The on-wire encoder: this is the exact call inside Ssb::write_payload.
    let bytes =
        rmp_serde::to_vec_named(&wrapped).expect("encode TemplateUtilityRanked on-wire");
    let decoded: SsbPayload =
        rmp_serde::from_slice(&bytes).expect("decode TemplateUtilityRanked from wire bytes");

    match decoded {
        SsbPayload::TemplateUtilityRanked(p) => {
            assert_eq!(
                p.concept_class, original.concept_class,
                "concept_class must round-trip"
            );
            assert_eq!(
                p.plan_skeleton_hash, original.plan_skeleton_hash,
                "plan_skeleton_hash (u64) must round-trip without truncation"
            );
            assert_eq!(p.utility, original.utility, "utility must round-trip");
            assert_eq!(
                p.first_try_ratio, original.first_try_ratio,
                "first_try_ratio baseline must round-trip"
            );
            assert_eq!(p.sample_n, original.sample_n, "sample_n must round-trip");
            assert_eq!(
                p.chosen_pipeline_id, original.chosen_pipeline_id,
                "chosen_pipeline_id must round-trip"
            );
        }
        other => panic!("expected TemplateUtilityRanked, got {other:?}"),
    }
}

// ---------------------------------------------------------------------------
// Contract 2: ssb_action_type_from_str_public recognises the wire tag
// ---------------------------------------------------------------------------

#[test]
fn contract_template_utility_str_roundtrip() {
    let at = ssb_action_type_from_str_public("template_utility_ranked")
        .expect("template_utility_ranked must parse");
    assert_eq!(
        at,
        SsbActionType::TemplateUtilityRanked,
        "template_utility_ranked must map to SsbActionType::TemplateUtilityRanked"
    );
    assert_eq!(
        at.as_sql_str(),
        "template_utility_ranked",
        "as_sql_str round-trip must be stable"
    );
}

// ---------------------------------------------------------------------------
// Contract 3: full DB path via Ssb::write_payload + query_by_type
// ---------------------------------------------------------------------------

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn contract_template_utility_db_roundtrip() {
    let tmp = TempDir::new().expect("tempdir");
    let ssb = make_ssb(&tmp, "f7-test", "f7-tmpl-rt").await;

    let original = sample_template_utility_payload();
    ssb.write_payload(
        "try-execute",
        0,
        SsbPayload::TemplateUtilityRanked(original.clone()),
        Vec::new(),
    )
    .await
    .expect("write TemplateUtilityRanked payload");

    let rows = ssb
        .query_by_type(SsbActionType::TemplateUtilityRanked)
        .expect("query TemplateUtilityRanked rows");
    assert_eq!(rows.len(), 1, "exactly one TemplateUtilityRanked row in DB");

    let decoded: SsbPayload =
        rmp_serde::from_slice(&rows[0].payload).expect("decode TemplateUtilityRanked from DB");
    match decoded {
        SsbPayload::TemplateUtilityRanked(p) => {
            assert_eq!(p.concept_class, original.concept_class);
            assert_eq!(p.plan_skeleton_hash, original.plan_skeleton_hash);
            assert_eq!(p.utility, original.utility);
            assert_eq!(p.first_try_ratio, original.first_try_ratio);
            assert_eq!(p.sample_n, original.sample_n);
            assert_eq!(p.chosen_pipeline_id, original.chosen_pipeline_id);
        }
        other => panic!("expected TemplateUtilityRanked, got {other:?}"),
    }
}

// ---------------------------------------------------------------------------
// Contract 4: Laplace-smoothing correctness (the F-7 gate, NOT a numeric uplift)
// ---------------------------------------------------------------------------

#[test]
fn contract_template_utility_laplace_smoothing_curbs_low_sample() {
    // A single lucky firing must NOT score a perfect 1.0; add-one smoothing
    // pulls it toward the 1/2 prior.
    let one_of_one = laplace_utility(1, 1);
    assert!(
        (one_of_one - (2.0 / 3.0)).abs() < 1e-6,
        "1/1 template must score (1+1)/(1+2) = 2/3, got {one_of_one}"
    );
    assert!(
        one_of_one < 1.0,
        "Laplace smoothing must keep a 1/1 template strictly below 1.0 \
         (low-sample over-trust guard), got {one_of_one}"
    );

    // A well-sampled high success rate stays close to its raw ratio.
    let ninety_five_of_hundred = laplace_utility(95, 100);
    assert!(
        (ninety_five_of_hundred - (96.0 / 102.0)).abs() < 1e-6,
        "95/100 template must score (95+1)/(100+2) ~= 0.941, got {ninety_five_of_hundred}"
    );

    // Monotonicity sanity: a high-sample 95/100 must out-rank a low-sample 1/1,
    // even though the raw first-try ratios are 0.95 vs 1.0. This is the whole
    // point of Laplace smoothing for template ranking.
    assert!(
        ninety_five_of_hundred > one_of_one,
        "well-sampled 95/100 ({ninety_five_of_hundred}) must out-rank lucky 1/1 \
         ({one_of_one}) despite a lower raw first-try ratio"
    );

    // A zero-firing template falls to the neutral 1/2 prior (no producer signal
    // means "unknown", not "best" and not "worst").
    let zero = laplace_utility(0, 0);
    assert!(
        (zero - 0.5).abs() < 1e-6,
        "0/0 template must fall to the (0+1)/(0+2) = 0.5 neutral prior, got {zero}"
    );
}

// ---------------------------------------------------------------------------
// Contract 5 (Slice A2): the SHIPPED template_utility module is the detector.
// These exercise the REAL `elai_orchestrator::template_utility` functions (not
// the local reference above) so the contract pins the producer's smoothing
// curve, the ELAI-owned ALPHA/BETA, and the field-recovery form.
// ---------------------------------------------------------------------------

#[test]
fn contract_module_alpha_beta_are_add_one() {
    // ELAI-owned add-one smoothing: changing these silently would shift every
    // template's score, so the contract pins them.
    assert_eq!(LAPLACE_ALPHA, 1.0, "LAPLACE_ALPHA must be add-one (1.0)");
    assert_eq!(LAPLACE_BETA, 1.0, "LAPLACE_BETA must be add-one (1.0)");
}

#[test]
fn contract_module_smoothing_points_match_reference() {
    // The shipped module must agree with the local Laplace reference at the
    // three documented anchor points.
    assert!(
        (module_laplace_utility(0, 0) - 0.5).abs() < 1e-6,
        "module 0/0 must be the 0.5 neutral prior"
    );
    let one_of_one = module_laplace_utility(1, 1);
    assert!(
        (one_of_one - (2.0 / 3.0)).abs() < 1e-6,
        "module 1/1 must be 2/3 ~= 0.667 (not 1.0), got {one_of_one}"
    );
    assert!(one_of_one < 1.0, "module 1/1 must stay strictly below 1.0");
    let ninety_five = module_laplace_utility(95, 100);
    assert!(
        (ninety_five - (96.0 / 102.0)).abs() < 1e-6,
        "module 95/100 must be 96/102 ~= 0.941, got {ninety_five}"
    );
}

#[test]
fn contract_module_high_sample_out_ranks_lucky_low_sample() {
    // THE F-7 smoothing gate: well-sampled 95/100 must out-rank lucky 1/1,
    // despite the lucky template's higher raw first-try ratio (1.0 vs 0.95).
    assert!(
        module_laplace_utility(95, 100) > module_laplace_utility(1, 1),
        "Laplace smoothing must let 95/100 out-rank a lucky 1/1"
    );
}

#[test]
fn contract_module_from_fields_recovers_integer_form() {
    // Field recovery from the persisted (first_try_success_rate, use_count)
    // pair must equal the integer-count form. 1.0 over 1 firing -> successes=1.
    assert!(
        (laplace_utility_from_fields(1.0, 1) - module_laplace_utility(1, 1)).abs() < 1e-6,
        "laplace_utility_from_fields(1.0, 1) must equal laplace_utility(1, 1)"
    );
    // 0.95 over 100 firings reconstructs successes=95.
    assert!(
        (laplace_utility_from_fields(0.95, 100) - module_laplace_utility(95, 100)).abs() < 1e-6,
        "laplace_utility_from_fields(0.95, 100) must equal laplace_utility(95, 100)"
    );
    // Zero use_count -> the neutral prior (no division by zero).
    assert!(
        (laplace_utility_from_fields(0.0, 0) - 0.5).abs() < 1e-6,
        "zero use_count must fall to the 0.5 neutral prior"
    );
}

#[test]
fn contract_module_template_utility_keeps_raw_baseline() {
    // W-2 no-removal: TemplateUtility carries BOTH the smoothed utility and the
    // raw first_try_ratio baseline so the dashboard can show before/after.
    let tu = TemplateUtility::from_pipeline_fields(
        "rest_crud_api".to_string(),
        0x5f3e_27a1_0b9c_d4e2,
        1.0,
        1,
    );
    assert_eq!(tu.concept_class, "rest_crud_api");
    assert_eq!(tu.plan_skeleton_hash, 0x5f3e_27a1_0b9c_d4e2);
    assert_eq!(tu.first_try_ratio, 1.0, "raw baseline must be preserved");
    assert_eq!(tu.sample_n, 1);
    assert!(
        (tu.utility - (2.0 / 3.0)).abs() < 1e-6,
        "smoothed utility for 1/1 must be 2/3, got {}",
        tu.utility
    );
    assert!(
        tu.utility < tu.first_try_ratio,
        "smoothing must pull a 1/1 template's utility BELOW its raw 1.0 baseline"
    );
}
