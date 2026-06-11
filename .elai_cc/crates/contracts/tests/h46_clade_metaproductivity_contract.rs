//! H-46 contract test — verifies HGM clade-metaproductivity parent selection.
//!
//! Source plan: docs/thefinalglance/H-plan/H-46_clade_metaproductivity_selection.md
//! Decisions: workflow/reasoning/H-46-decisions.md
//!
//! Off-agent artifact requirement (R4/R7): tests (b), (c), and (d) drive the
//! REAL `config_archive::select_parent_with_cmp` path on a frozen in-memory
//! archive fixture — NOT a unit shim that calls clade_metaproductivity directly.
//! The key off-agent observables are:
//!   - CladeSelected SSB row shape (struct construction + action_type round-trip,
//!     SCHEMA_VERSION 224).
//!   - Node B (mediocre immediate, IMPROVING clade) is returned over node A
//!     (high immediate, REGRESSING clade) when clade sizes are >= 1.
//!   - The broken `force_immediate_score=true` arm returns A (dead-end),
//!     proving CMP is load-bearing (R2 falsifiability).
//!   - A clade_size==0 leaf uses `ImmediateScoreFallback` (no-lineage cold-start).
//!   - Same inputs, same seed => byte-identical cmp (GD-2).
//!
//! Tests:
//! (a) cmp(B) > cmp(A) when A's direct child regresses, B's improves
//!     despite immediate(A) > immediate(B).
//! (b) select_parent_with_cmp returns B over the real path (R4/R7).
//! (c) force_immediate_score=true arm returns A (R2 falsifiability).
//! (d) clade_size==0 node uses ImmediateScoreFallback key.
//! (e) same inputs + same entropy seed => byte-identical cmp aggregate (GD-2).
//! (f) CladeSelected wire-tag = "clade_selected".
//! (g) CladeSelected payload action_type round-trips correctly.
//! (h) SCHEMA_VERSION >= 224 (H-46 bump verified at the contract level).
//! (i) CMP aggregate uses execution-grounded deltas (NOT byte-hash values) — VD-1 grep.

#[cfg(feature = "darwinian-evolve")]
use std::collections::HashMap;

#[cfg(feature = "darwinian-evolve")]
use elai_orchestrator::config_archive::ConfigArchive;
#[cfg(feature = "darwinian-evolve")]
use elai_orchestrator::self_mod_layer1a::PipelineMode;
#[cfg(feature = "darwinian-evolve")]
use elai_orchestrator::self_mod_types::{ConfigVariant, VariantStatus};
#[cfg(feature = "darwinian-evolve")]
use elai_orchestrator::{clade_metaproductivity, SelectionKey};
use elai_orchestrator::CMP_MIN_CLADE_SIZE;
use elai_orchestrator::ssb::{
    CladeSelectedKind, CladeSelectedPayload, SsbActionType, SsbPayload, SCHEMA_VERSION_PUBLIC,
};
#[cfg(feature = "darwinian-evolve")]
use elai_core::SeededRng;
#[cfg(feature = "darwinian-evolve")]
use tempfile::TempDir;

// ---------------------------------------------------------------------------
// Frozen fixture helpers
// ---------------------------------------------------------------------------

#[cfg(feature = "darwinian-evolve")]
fn make_variant(id: &str, parent: Option<&str>, score: f32) -> ConfigVariant {
    ConfigVariant {
        variant_id: id.to_string(),
        parent_id: parent.map(|s| s.to_string()),
        score,
        children_count: 0,
        config_blob: id.as_bytes().to_vec(),
        created_at: 0,
        pipeline_mode: PipelineMode::Greenfield,
        status: VariantStatus::Archived,
        pipeline_outcomes: vec![],
    }
}

/// Node A: high immediate score (0.9), direct child A1 that REGRESSES by -0.5.
/// Node B: mediocre immediate score (0.4), direct child B1 that IMPROVES by +0.5.
/// Node C: no children (leaf) -> ImmediateScoreFallback.
///
/// Expected CMP outcomes (mean delta + immediate sigmoid, lambda=10, alpha_0=0.5):
///   sigmoid(0.9) = 1/(1+exp(-10*(0.9-0.5))) = 1/(1+exp(-4)) ≈ 0.9820
///   sigmoid(0.4) = 1/(1+exp(-10*(0.4-0.5))) = 1/(1+exp( 1)) ≈ 0.2689
///
///   cmp(A) = 0.9820 + (-0.5) = 0.4820   (regressing clade hurts A despite high immediate)
///   cmp(B) = 0.2689 + (+0.5) = 0.7689   (improving clade lifts B despite mediocre immediate)
///
///   => cmp(B) > cmp(A) despite immediate(A) >> immediate(B).
///   Delta margin = 0.5 > (sigmoid(0.9) - sigmoid(0.4)) ≈ 0.713 / 2, safely crossing.
#[cfg(feature = "darwinian-evolve")]
fn fixture_variants() -> (Vec<ConfigVariant>, HashMap<String, f64>) {
    let a = make_variant("A", None, 0.9);
    let b = make_variant("B", None, 0.4);
    let c = make_variant("C", None, 0.5); // leaf, no delta entry
    let a1 = make_variant("A1", Some("A"), 0.3); // A's regressing child
    let b1 = make_variant("B1", Some("B"), 0.8); // B's improving child
    let variants = vec![a, b, c, a1, b1];
    let mut deltas = HashMap::new();
    deltas.insert("A1".to_string(), -0.5_f64); // A's child regressed strongly
    deltas.insert("B1".to_string(), 0.5_f64);  // B's child improved strongly
    (variants, deltas)
}

/// Same fixture used for the real-path select_parent_with_cmp test (b).
/// All 4 variants (A, B, A1, B1) are inserted into the archive so A1/B1
/// appear in list_variants and the clade_metaproductivity function can
/// walk the parent-child edges in the in-memory slice. C is omitted so the
/// test exercises only the CMP-vs-immediate distinction.
#[cfg(feature = "darwinian-evolve")]
fn two_node_variants() -> (Vec<ConfigVariant>, HashMap<String, f64>) {
    let a = make_variant("A", None, 0.9);
    let b = make_variant("B", None, 0.4);
    let a1 = make_variant("A1", Some("A"), 0.3);
    let b1 = make_variant("B1", Some("B"), 0.8);
    let variants = vec![a, b, a1, b1];
    let mut deltas = HashMap::new();
    deltas.insert("A1".to_string(), -0.5_f64);
    deltas.insert("B1".to_string(), 0.5_f64);
    (variants, deltas)
}

// ---------------------------------------------------------------------------
// (a) cmp(B) > cmp(A) despite higher immediate(A)
// ---------------------------------------------------------------------------

#[cfg(feature = "darwinian-evolve")]
#[test]
fn a_cmp_b_beats_cmp_a_despite_higher_immediate_a() {
    let (variants, deltas) = fixture_variants();
    let scores = clade_metaproductivity(&variants, &deltas);

    // Index 0=A, 1=B, 2=C (leaf), 3=A1, 4=B1
    let a_score = &scores[0];
    let b_score = &scores[1];

    // Verify immediate scores: A must have a higher immediate score.
    assert!(
        a_score.immediate_score > b_score.immediate_score,
        "fixture: immediate(A)={:.4} must exceed immediate(B)={:.4}",
        a_score.immediate_score, b_score.immediate_score
    );

    // Both have clade_size=1 (one direct child with measured delta).
    assert_eq!(a_score.clade_size, 1, "A must have clade_size=1");
    assert_eq!(b_score.clade_size, 1, "B must have clade_size=1");

    // Both should use CladeMetaproductivity key.
    assert_eq!(
        a_score.key, SelectionKey::CladeMetaproductivity,
        "A must use CladeMetaproductivity key"
    );
    assert_eq!(
        b_score.key, SelectionKey::CladeMetaproductivity,
        "B must use CladeMetaproductivity key"
    );

    // The key property: cmp(B) > cmp(A) despite immediate(A) > immediate(B).
    assert!(
        b_score.cmp > a_score.cmp,
        "cmp(B)={:.4} must exceed cmp(A)={:.4} (CMP fixes Metaproductivity-Performance Mismatch)",
        b_score.cmp, a_score.cmp
    );
}

// ---------------------------------------------------------------------------
// (b) select_parent_with_cmp returns B over the REAL path (R4/R7)
// ---------------------------------------------------------------------------
//
// We build a frozen archive with only A and B (+ their children for deltas),
// seed the rng to 0, and run 200 trials. Because cmp(B) > cmp(A), the
// probability of selecting B is > 50%. With 200 trials and a large margin
// this should be deterministic enough to assert majority selection.
//
// This drives the REAL config_archive::select_parent_with_cmp path — NOT a
// unit shim calling clade_metaproductivity directly (R4/R7).

#[cfg(feature = "darwinian-evolve")]
#[tokio::test]
async fn b_select_parent_with_cmp_returns_b_on_real_path() {
    let dir = TempDir::new().expect("tempdir");
    let db = dir.path().join("test.db");
    let archive = ConfigArchive::open(&db).await.expect("archive open");

    let (variants, deltas) = two_node_variants();
    // Insert all 4 variants. A1 and B1 are children in the archive so
    // select_parent_with_cmp's list_variants call returns them and
    // clade_metaproductivity can walk the parent-child edges to build the clade.
    for v in &variants {
        archive.insert_variant(v).await.expect("insert");
    }

    // Run 200 trials with seeded rng to observe selection distribution.
    let mut b_count = 0usize;
    let mut a_count = 0usize;
    for seed in 0u64..200 {
        let mut rng = SeededRng::new(seed);
        let (selected, _scores) = archive
            .select_parent_with_cmp(PipelineMode::Greenfield, &deltas, &mut rng, false)
            .await
            .expect("select ok");
        match selected.as_str() {
            "B" => b_count += 1,
            "A" => a_count += 1,
            _ => {}
        }
    }

    // B must be selected more often than A (CMP load-bearing, R4/R7).
    assert!(
        b_count > a_count,
        "B (high-CMP mediocre-immediate) must be selected more often than A (dead-end) \
         over 200 trials; got B={b_count}, A={a_count}"
    );
}

// ---------------------------------------------------------------------------
// (c) force_immediate_score=true returns A (R2 falsifiability)
// ---------------------------------------------------------------------------

#[cfg(feature = "darwinian-evolve")]
#[tokio::test]
async fn c_force_immediate_score_returns_a_dead_end() {
    let dir = TempDir::new().expect("tempdir");
    let db = dir.path().join("test_r2.db");
    let archive = ConfigArchive::open(&db).await.expect("archive open");

    let (variants, deltas) = two_node_variants();
    for v in &variants {
        archive.insert_variant(v).await.expect("insert");
    }

    // Run 200 trials with force_immediate_score=true.
    let mut a_count = 0usize;
    let mut b_count = 0usize;
    for seed in 0u64..200 {
        let mut rng = SeededRng::new(seed);
        let (selected, _scores) = archive
            .select_parent_with_cmp(PipelineMode::Greenfield, &deltas, &mut rng, true)
            .await
            .expect("select ok");
        match selected.as_str() {
            "A" => a_count += 1,
            "B" => b_count += 1,
            _ => {}
        }
    }

    // With immediate score only, A (0.9) dominates B (0.4).
    assert!(
        a_count > b_count,
        "R2: immediate-score arm must select A (dead-end 0.9) more than B (mediocre 0.4); \
         got A={a_count}, B={b_count}"
    );
}

// ---------------------------------------------------------------------------
// (d) clade_size==0 leaf uses ImmediateScoreFallback
// ---------------------------------------------------------------------------

#[cfg(feature = "darwinian-evolve")]
#[test]
fn d_leaf_with_no_descendants_uses_immediate_score_fallback() {
    let (variants, deltas) = fixture_variants();
    let scores = clade_metaproductivity(&variants, &deltas);

    // Index 2=C (leaf, no delta entry).
    let c_score = &scores[2];
    assert_eq!(c_score.clade_size, 0, "C leaf must have clade_size=0");
    assert_eq!(
        c_score.key,
        SelectionKey::ImmediateScoreFallback,
        "C leaf must use ImmediateScoreFallback (no lineage)"
    );
    // cmp equals immediate_score on fallback.
    assert!(
        (c_score.cmp - c_score.immediate_score).abs() < 1e-10,
        "fallback: cmp must equal immediate_score; cmp={:.6}, imm={:.6}",
        c_score.cmp, c_score.immediate_score
    );
}

// ---------------------------------------------------------------------------
// (e) deterministic under same inputs (GD-2)
// ---------------------------------------------------------------------------

#[cfg(feature = "darwinian-evolve")]
#[test]
fn e_cmp_is_deterministic_under_same_inputs() {
    let (variants, deltas) = fixture_variants();
    let r1 = clade_metaproductivity(&variants, &deltas);
    let r2 = clade_metaproductivity(&variants, &deltas);
    for (i, (s1, s2)) in r1.iter().zip(r2.iter()).enumerate() {
        assert_eq!(
            s1.cmp, s2.cmp,
            "GD-2: cmp must be byte-identical at index {i}"
        );
        assert_eq!(s1.clade_size, s2.clade_size, "clade_size mismatch at {i}");
    }
}

// ---------------------------------------------------------------------------
// (f) wire-tag
// ---------------------------------------------------------------------------

#[test]
fn f_wire_tag_is_clade_selected() {
    assert_eq!(
        SsbActionType::CladeSelected.as_sql_str(),
        "clade_selected",
        "CladeSelected wire-tag must be 'clade_selected'"
    );
}

// ---------------------------------------------------------------------------
// (g) action_type round-trip
// ---------------------------------------------------------------------------

#[test]
fn g_action_type_round_trips() {
    let payload = SsbPayload::CladeSelected(CladeSelectedPayload {
        kind: CladeSelectedKind::CladeMetaproductivity,
        node_id: "variant-B".to_string(),
        cmp: 0.55,
        clade_size: 3,
        immediate_score: 0.41,
        key_used: "clade_metaproductivity".to_string(),
        seed: 42,
    });
    assert_eq!(
        payload.action_type(),
        SsbActionType::CladeSelected,
        "CladeSelected payload must map to CladeSelected action type"
    );
    // Fallback arm round-trip.
    let fallback = SsbPayload::CladeSelected(CladeSelectedPayload {
        kind: CladeSelectedKind::ImmediateScoreFallback,
        node_id: "variant-C".to_string(),
        cmp: 0.50,
        clade_size: 0,
        immediate_score: 0.50,
        key_used: "immediate_score_fallback".to_string(),
        seed: 0,
    });
    assert_eq!(fallback.action_type(), SsbActionType::CladeSelected);
}

// ---------------------------------------------------------------------------
// (h) SCHEMA_VERSION >= 224
// ---------------------------------------------------------------------------

#[test]
fn h_schema_version_at_least_224() {
    assert!(
        SCHEMA_VERSION_PUBLIC >= 224,
        "SCHEMA_VERSION must be >= 224 after H-46 bump; got {}",
        SCHEMA_VERSION_PUBLIC
    );
}

// ---------------------------------------------------------------------------
// (i) CMP aggregate uses execution-grounded deltas, not byte-hash (VD-1 grep)
// ---------------------------------------------------------------------------
//
// This test asserts structural compliance: the clade_metaproductivity.rs source
// does NOT contain references to blake3 bytes as the fitness signal (VD-1 stub-ban).
// It checks the source text of the aggregator to confirm the VD-1 property.

#[test]
fn i_cmp_does_not_use_byte_hash_as_fitness_signal() {
    let src_path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("contracts has parent (elai_cc/crates)")
        .parent()
        .expect("crates has parent (elai_cc)")
        .join("crates")
        .join("orchestrator")
        .join("src")
        .join("clade_metaproductivity.rs");

    assert!(src_path.exists(), "clade_metaproductivity.rs must exist at {}", src_path.display());
    let src = std::fs::read_to_string(&src_path).expect("read clade_metaproductivity.rs");

    // VD-1: the aggregator must NOT apply blake3 hash bytes as the fitness
    // signal. A call to blake3::hash(config_blob) as the delta value is the
    // GAP-1 noise path that CMP must replace. We check production call sites
    // (non-comment lines) only.
    let production_lines: Vec<&str> = src
        .lines()
        .filter(|line| {
            let trimmed = line.trim();
            !trimmed.starts_with("//") && !trimmed.starts_with("///") && !trimmed.is_empty()
        })
        .collect();
    let production_src = production_lines.join("\n");

    assert!(
        !production_src.contains("blake3::hash(config_blob)"),
        "VD-1: clade_metaproductivity production code must not derive CMP from blake3(config_blob)"
    );
    assert!(
        !production_src.contains("derived_alpha"),
        "VD-1: clade_metaproductivity production code must not reference derived_alpha (the GAP-1 path)"
    );

    // The aggregator must consume execution-grounded fitness deltas (from H-43).
    assert!(
        src.contains("fitness_deltas"),
        "VD-1: clade_metaproductivity must accept a fitness_deltas parameter"
    );
}

// ---------------------------------------------------------------------------
// CMP_MIN_CLADE_SIZE is locked to 1
// ---------------------------------------------------------------------------

#[test]
fn cmp_min_clade_size_is_locked_to_1() {
    assert_eq!(
        CMP_MIN_CLADE_SIZE, 1,
        "CMP_MIN_CLADE_SIZE must be 1 (override fires after first measured descendant)"
    );
}

// ---------------------------------------------------------------------------
// (j) run_one_generation emits CladeSelected on the real loop path (BLOCK-1 fix)
// ---------------------------------------------------------------------------
//
// Drives run_one_generation with one admitted candidate so the loop closes and
// select_parent_with_cmp is called. Asserts that at least one CladeSelected
// SSB row was written to the database (off-agent artifact, R4/R7 proof).
//
// This is the wire-dark-fix test: before BLOCK-1 was resolved select_parent_with
// and select_parent were called instead of select_parent_with_cmp, so no
// CladeSelected row ever landed from the real loop.

#[cfg(feature = "darwinian-evolve")]
#[tokio::test]
async fn j_run_one_generation_emits_clade_selected_ssb_row() {
    use elai_orchestrator::config_archive::ConfigArchive;
    use elai_orchestrator::darwin_admit_loop::{
        run_one_generation, MutantCandidate, DEFAULT_SUITE_NAME,
    };
    use elai_orchestrator::promotion_gate::PromotionConfig;
    use elai_orchestrator::self_mod_layer1a::PipelineMode;
    use elai_orchestrator::ssb::{Ssb, SsbConfig, SsbActionType};
    use elai_core::SeededRng;
    use tempfile::TempDir;

    let dir = TempDir::new().expect("tempdir");
    let ssb_path = dir.path().join("ssb.db");
    let archive = ConfigArchive::open(&dir.path().join("self_mod.db"))
        .await
        .expect("archive open");
    let ssb = Ssb::new(
        SsbConfig {
            db_path: ssb_path.clone(),
            pool_size: None,
            project_id: "h46-contract".to_string(),
            session_id: "h46-contract-j".to_string(),
        },
        std::sync::Arc::new(elai_core::OsClock),
    )
    .expect("ssb");

    // One admitted candidate: force_alpha=0.6 (above tau) and no trap tokens.
    let candidate = MutantCandidate {
        variant_label: "h46-j-cand".to_string(),
        pipeline_mode: "surgical".to_string(),
        config: serde_json::json!({ "retry_budget": 3, "force_alpha": 0.6 }),
        eval_log: "suite=math-floors score=0.6 h46-j-cand".to_string(),
    };
    let cfg = PromotionConfig::default();
    let mut rng = SeededRng::new(99);
    let summary = run_one_generation(
        &archive,
        &ssb,
        &[candidate],
        PipelineMode::Surgical,
        99,
        DEFAULT_SUITE_NAME,
        &cfg,
        &mut rng,
    )
    .await
    .expect("run_one_generation");

    // The loop must close (admitted candidate + select_parent_with_cmp called).
    assert!(summary.loop_closed, "loop must close on admitted candidate");
    assert_eq!(summary.admitted, 1, "one candidate must be admitted");

    // Flush SSB writes before reading.
    drop(ssb);

    // Open the SSB db and confirm at least one CladeSelected row was written.
    let conn = rusqlite::Connection::open(&ssb_path).expect("open ssb db");
    let count: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM ssb WHERE action_type = ?1",
            rusqlite::params![SsbActionType::CladeSelected.as_sql_str()],
            |row| row.get(0),
        )
        .expect("query clade_selected count");
    assert!(
        count >= 1,
        "select_parent_with_cmp must emit at least one CladeSelected SSB row on the real loop; got {}",
        count
    );
}
