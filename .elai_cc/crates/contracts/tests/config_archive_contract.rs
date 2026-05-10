//! Contract tests for B-23 Phase 4H Group B — config_archive.rs.
//!
//! Verifies DGM parent-selection formula invariants, archive overflow with
//! deceptive-dip resilience, mode-filtered selection, and per-mode unique
//! ACTIVE invariant. All formulas grounded in
//! `D:\.ELAI_workflow\workflow\papers\dgm-interface.md`.

use elai_orchestrator::config_archive::{
    parent_selection_weights, parent_selection_probabilities, sigmoid_perf_score,
    exploration_bonus, ConfigArchive, ConfigArchiveError,
    DGM_LAMBDA, DGM_ALPHA_0, DEFAULT_ARCHIVE_CAP_PER_MODE,
};
use elai_orchestrator::{
    ConfigVariant, PipelineMode, VariantStatus,
};
use std::sync::Arc;
use tempfile::TempDir;

fn variant(id: &str, parent: Option<&str>, score: f32, children: usize, mode: PipelineMode, status: VariantStatus) -> ConfigVariant {
    ConfigVariant {
        variant_id: id.to_string(),
        parent_id: parent.map(|s| s.to_string()),
        score,
        children_count: children,
        config_blob: vec![],
        created_at: 0,
        pipeline_outcomes: vec![],
        pipeline_mode: mode,
        status,
    }
}

// ---------------------------------------------------------------------------
// Formula contracts (grounded: dgm-interface.md assertions block)
// ---------------------------------------------------------------------------

#[test]
fn sigmoid_at_alpha_0_is_half() {
    // s_i(alpha=0.5) = 1/(1+exp(0)) = 0.5 — DGM section 2.3 midpoint check.
    let s = sigmoid_perf_score(0.5);
    assert!((s - 0.5).abs() < 1e-6, "got {s}, expected 0.5");
}

#[test]
fn sigmoid_strictly_increasing() {
    // Monotone in alpha — exploitation pressure goes up with success rate.
    let mut prev = -1.0f32;
    for i in 0..=10 {
        let alpha = i as f32 / 10.0;
        let s = sigmoid_perf_score(alpha);
        assert!(s > prev, "sigmoid not monotone at alpha={alpha}");
        prev = s;
    }
}

#[test]
fn sigmoid_strictly_positive() {
    // Sigmoid is never exactly zero — every variant has non-zero selection prob.
    for alpha in [0.0_f32, 0.05, 0.5, 0.95, 1.0] {
        assert!(sigmoid_perf_score(alpha) > 0.0, "alpha={alpha} produced zero");
    }
}

#[test]
fn exploration_bonus_decays_with_children() {
    // h_i = 1/(1+n_i); n=0 -> 1.0; n=10 -> 1/11.
    assert!((exploration_bonus(0) - 1.0).abs() < 1e-6);
    assert!((exploration_bonus(10) - (1.0 / 11.0)).abs() < 1e-6);
    // Strictly decreasing.
    assert!(exploration_bonus(0) > exploration_bonus(1));
    assert!(exploration_bonus(1) > exploration_bonus(5));
}

#[test]
fn parent_selection_weights_match_formula() {
    // For variants with score 0.2/0.5/0.8 and children 0/2/5,
    // weights = sigmoid * exploration. Manual reference computed to 1e-4.
    let v = vec![
        variant("v1", None, 0.2, 0, PipelineMode::Surgical, VariantStatus::Archived),
        variant("v2", None, 0.5, 2, PipelineMode::Surgical, VariantStatus::Archived),
        variant("v3", None, 0.8, 5, PipelineMode::Surgical, VariantStatus::Active),
    ];
    let weights = parent_selection_weights(&v);
    assert_eq!(weights.len(), 3);
    // s(0.2) = 1/(1+exp(3)) ≈ 0.0474
    // s(0.5) = 0.5
    // s(0.8) = 1/(1+exp(-3)) ≈ 0.9526
    // h(0)=1, h(2)=1/3, h(5)=1/6
    let expected = [
        0.04742587 * 1.0,
        0.5 * (1.0 / 3.0),
        0.95257413 * (1.0 / 6.0),
    ];
    for (got, want) in weights.iter().zip(expected.iter()) {
        assert!((got - want).abs() < 1e-3, "weight mismatch: {got} vs {want}");
    }
}

#[test]
fn parent_selection_probabilities_sum_to_one() {
    let v = vec![
        variant("v1", None, 0.2, 0, PipelineMode::Surgical, VariantStatus::Archived),
        variant("v2", None, 0.5, 2, PipelineMode::Surgical, VariantStatus::Archived),
        variant("v3", None, 0.8, 5, PipelineMode::Surgical, VariantStatus::Active),
    ];
    let probs = parent_selection_probabilities(&v);
    assert_eq!(probs.len(), 3);
    let total: f32 = probs.iter().sum();
    assert!((total - 1.0).abs() < 1e-5, "probabilities sum to {total}");
    for p in &probs {
        assert!(*p > 0.0, "probability must be strictly positive (DGM invariant)");
    }
}

#[test]
fn higher_score_higher_probability_at_same_children() {
    // At identical n_i, exploitation pressure favours higher score.
    let v = vec![
        variant("low",  None, 0.2, 1, PipelineMode::Surgical, VariantStatus::Archived),
        variant("high", None, 0.8, 1, PipelineMode::Surgical, VariantStatus::Archived),
    ];
    let probs = parent_selection_probabilities(&v);
    assert!(probs[1] > probs[0], "higher-score variant should have higher selection probability");
}

#[test]
fn fewer_children_higher_probability_at_same_score() {
    // At identical alpha, exploration prefers fewer children.
    let v = vec![
        variant("explored",   None, 0.5, 10, PipelineMode::Surgical, VariantStatus::Archived),
        variant("unexplored", None, 0.5, 0,  PipelineMode::Surgical, VariantStatus::Archived),
    ];
    let probs = parent_selection_probabilities(&v);
    assert!(probs[1] > probs[0], "unexplored variant should be preferred");
}

#[test]
fn lambda_alpha_0_constants_match_spec() {
    assert!((DGM_LAMBDA - 10.0).abs() < 1e-6);
    assert!((DGM_ALPHA_0 - 0.5).abs() < 1e-6);
    assert_eq!(DEFAULT_ARCHIVE_CAP_PER_MODE, 50);
}

// ---------------------------------------------------------------------------
// Archive contracts — SQLite-backed
// ---------------------------------------------------------------------------

async fn fresh_archive() -> (TempDir, ConfigArchive) {
    let tmp = tempfile::tempdir().unwrap();
    let path = tmp.path().join("self_mod.db");
    let archive = ConfigArchive::open(&path).await.unwrap();
    (tmp, archive)
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn insert_and_list_round_trip() {
    let (_tmp, archive) = fresh_archive().await;
    let v = variant("v1", None, 0.5, 0, PipelineMode::Surgical, VariantStatus::Active);
    archive.insert_variant(&v).await.unwrap();
    let listed = archive.list_variants(PipelineMode::Surgical).await.unwrap();
    assert_eq!(listed.len(), 1);
    assert_eq!(listed[0].variant_id, "v1");
    assert_eq!(listed[0].status, VariantStatus::Active);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn one_active_per_mode_invariant() {
    let (_tmp, archive) = fresh_archive().await;
    archive
        .insert_variant(&variant("a", None, 0.5, 0, PipelineMode::Surgical, VariantStatus::Active))
        .await
        .unwrap();
    let err = archive
        .insert_variant(&variant("b", None, 0.6, 0, PipelineMode::Surgical, VariantStatus::Active))
        .await;
    assert!(matches!(err, Err(ConfigArchiveError::ActiveAlreadyExists(_))));

    // Different mode is fine.
    archive
        .insert_variant(&variant("c", None, 0.7, 0, PipelineMode::Greenfield, VariantStatus::Active))
        .await
        .unwrap();
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn mode_filtered_parent_selection_never_crosses() {
    let (_tmp, archive) = fresh_archive().await;
    for i in 0..5 {
        archive
            .insert_variant(&variant(
                &format!("s{i}"), None, 0.3 + (i as f32) * 0.1, 0,
                PipelineMode::Surgical,
                if i == 0 { VariantStatus::Active } else { VariantStatus::Archived },
            ))
            .await
            .unwrap();
    }
    for i in 0..5 {
        archive
            .insert_variant(&variant(
                &format!("g{i}"), None, 0.3 + (i as f32) * 0.1, 0,
                PipelineMode::Greenfield,
                if i == 0 { VariantStatus::Active } else { VariantStatus::Archived },
            ))
            .await
            .unwrap();
    }

    // Sample 200 picks; every one must be Surgical.
    for _ in 0..200 {
        let pick = archive.select_parent(PipelineMode::Surgical).await.unwrap();
        assert!(pick.starts_with('s'), "got cross-mode pick: {pick}");
    }
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn archive_cap_keeps_active_and_ancestors() {
    // Lineage: A -> B -> C(Active). B has score 0.1 (lowest). 47 random variants
    // all score 0.5 to fill out the archive past the 50 cap. We assert B
    // survives because it's an ancestor of the ACTIVE variant.
    let (_tmp, archive) = fresh_archive().await;

    archive
        .insert_variant(&variant("A", None, 0.5, 1, PipelineMode::Surgical, VariantStatus::Archived))
        .await
        .unwrap();
    archive
        .insert_variant(&variant("B", Some("A"), 0.1, 1, PipelineMode::Surgical, VariantStatus::Archived))
        .await
        .unwrap();
    archive
        .insert_variant(&variant("C", Some("B"), 0.7, 0, PipelineMode::Surgical, VariantStatus::Active))
        .await
        .unwrap();

    for i in 0..52 {
        archive
            .insert_variant(&variant(
                &format!("filler-{i}"),
                None,
                0.5,
                0,
                PipelineMode::Surgical,
                VariantStatus::Archived,
            ))
            .await
            .unwrap();
    }
    archive.enforce_cap(PipelineMode::Surgical).await.unwrap();

    let listed = archive.list_variants(PipelineMode::Surgical).await.unwrap();
    assert!(
        listed.len() <= DEFAULT_ARCHIVE_CAP_PER_MODE as usize,
        "archive {} exceeds cap {}",
        listed.len(),
        DEFAULT_ARCHIVE_CAP_PER_MODE,
    );

    let ids: Vec<_> = listed.iter().map(|v| v.variant_id.as_str()).collect();
    assert!(ids.contains(&"A"), "ancestor A pruned");
    assert!(ids.contains(&"B"), "deceptive-dip ancestor B pruned");
    assert!(ids.contains(&"C"), "active variant pruned");
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn revert_promotes_previous_archived_variant_same_mode() {
    let (_tmp, archive) = fresh_archive().await;
    // Two surgical variants, one greenfield. Revert surgical active; expect
    // the highest-score archived surgical to become active (NOT greenfield).
    archive
        .insert_variant(&variant("s-old", None, 0.6, 0, PipelineMode::Surgical, VariantStatus::Archived))
        .await
        .unwrap();
    archive
        .insert_variant(&variant("s-new", None, 0.7, 0, PipelineMode::Surgical, VariantStatus::Active))
        .await
        .unwrap();
    archive
        .insert_variant(&variant("g-old", None, 0.9, 0, PipelineMode::Greenfield, VariantStatus::Active))
        .await
        .unwrap();

    let restored = archive.revert(PipelineMode::Surgical).await.unwrap();
    assert_eq!(restored.as_deref(), Some("s-old"), "wrong surgical revert target");

    let surgical = archive.list_variants(PipelineMode::Surgical).await.unwrap();
    let active_surgical: Vec<_> = surgical.iter().filter(|v| v.status == VariantStatus::Active).collect();
    assert_eq!(active_surgical.len(), 1);
    assert_eq!(active_surgical[0].variant_id, "s-old");

    // Greenfield untouched.
    let greenfield = archive.list_variants(PipelineMode::Greenfield).await.unwrap();
    let active_greenfield: Vec<_> = greenfield.iter().filter(|v| v.status == VariantStatus::Active).collect();
    assert_eq!(active_greenfield.len(), 1);
    assert_eq!(active_greenfield[0].variant_id, "g-old");
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn select_parent_returns_none_on_empty_archive() {
    let (_tmp, archive) = fresh_archive().await;
    let pick = archive.select_parent(PipelineMode::Oracle).await.unwrap();
    assert!(pick.is_empty(), "expected empty pick on empty archive, got {pick:?}");
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn select_parent_thread_safe_via_arc() {
    // Smoke test: Arc-clone the archive and call select_parent on multiple tasks.
    let (_tmp, archive) = fresh_archive().await;
    archive
        .insert_variant(&variant("v1", None, 0.5, 0, PipelineMode::Surgical, VariantStatus::Active))
        .await
        .unwrap();
    let archive = Arc::new(archive);
    let mut handles = vec![];
    for _ in 0..5 {
        let a = archive.clone();
        handles.push(tokio::spawn(async move {
            a.select_parent(PipelineMode::Surgical).await.unwrap()
        }));
    }
    for h in handles {
        let pick = h.await.unwrap();
        assert_eq!(pick, "v1");
    }
}
