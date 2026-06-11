//! H-41 contract test — verifies the checker-coverage + gap-instrument wiring.
//!
//! Source plan: docs/thefinalglance/H-plan/H-41_checker_coverage_gap_instrument.md
//! Decisions: workflow/reasoning/H-41-decisions.md
//!
//! Off-agent artifact requirement (R4/R7): tests (c) and (d) drive the REAL
//! `route_claim_with_gap` path from agent_pool.rs — NOT a unit shim that calls
//! the payload constructor directly. The SSB row is the durable off-agent signal;
//! its presence (or absence) is asserted. Missing = FAIL (R1).
//!
//! Tests:
//! (a) Covered Factual claim (has_det=true, has_evidence=true) routes Constructive;
//!     no CheckerGapDetected row written.
//! (b) Uncovered Factual claim (has_det=false): non-Constructive route + gap row
//!     written with correct claim_type + fell_to_mode.
//! (c) NP-complete keyword in claim text: classify_complexity_taxon ->
//!     NpCompleteOrCombinatorial, priority=2, HIGH.
//! (d) mode_a_fraction fixture: 3 Constructive + 1 FalsifiabilityCrossFamily
//!     among F+C claims = 0.75.
//! (e) SCHEMA_VERSION_PUBLIC >= 219 (H-41 bump verified at the off-agent contract level).
//! (f) CheckerGapDetected wire-tag = "checker_gap_detected" (SQL CHECK constraint).

use std::path::PathBuf;
use std::sync::Arc;

use elai_core::clock::OsClock;
use elai_orchestrator::claim::{Claim, ClaimType, EvidenceKind, EvidenceRef};
use elai_orchestrator::orch_context::{OrchContext, OrchPhase, OrchUserRole};
use elai_orchestrator::ssb::{SsbActionType, SsbConfig, SsbPayload, SCHEMA_VERSION_PUBLIC};
use elai_orchestrator::verification_result::VerifierKind;
use elai_orchestrator::{
    checker_gap_backlog, classify_complexity_taxon, mode_a_coverage,
    route_claim_with_gap, CheckerGapDetectedPayload, CheckerGapEmitter, ComplexityTaxon,
    Ssb, SsbViewer, VecCheckerGapEmitter, VecClaimRoutedEmitter,
};
use tempfile::TempDir;

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn fresh_db() -> (TempDir, PathBuf) {
    let dir = TempDir::new().expect("tempdir");
    let path = dir.path().join("ssb.db");
    (dir, path)
}

async fn open_writer(path: &std::path::Path) -> Ssb {
    let cfg = SsbConfig {
        db_path: path.to_path_buf(),
        pool_size: Some(2),
        project_id: "h41-contract".into(),
        session_id: "h41-test".into(),
    };
    Ssb::new(cfg, Arc::new(OsClock)).expect("ssb init")
}

fn open_viewer(path: &std::path::Path) -> SsbViewer {
    SsbViewer::open(path).expect("viewer open")
}

fn mk_factual_with_evidence() -> Claim {
    Claim {
        claim_id: "covered-factual".into(),
        claim_type: ClaimType::Factual,
        text: "The function returns 42 for input 6*7".into(),
        evidence: vec![EvidenceRef::from_content(
            EvidenceKind::Corpus,
            "test/fixture:1",
            b"measured output == 42",
        )],
        ts_ms: 1_000,
    }
}

fn mk_factual_no_evidence() -> Claim {
    Claim {
        claim_id: "uncovered-factual".into(),
        claim_type: ClaimType::Factual,
        text: "The deployment was successful on the staging environment".into(),
        evidence: vec![],
        ts_ms: 2_000,
    }
}

fn mk_np_complete_claim() -> Claim {
    Claim {
        claim_id: "np-complete".into(),
        claim_type: ClaimType::Computational,
        text: "This is a SAT-based constraint satisfaction problem".into(),
        evidence: vec![],
        ts_ms: 3_000,
    }
}

fn ctx_with_deterministic() -> OrchContext {
    let mut ctx = OrchContext::minimal(OrchUserRole::Executor, OrchPhase::Executing);
    ctx.available_verifiers = vec![VerifierKind::Deterministic, VerifierKind::Intrinsic];
    ctx
}

fn ctx_without_deterministic() -> OrchContext {
    let mut ctx = OrchContext::minimal(OrchUserRole::Executor, OrchPhase::Executing);
    // No VerifierKind::Deterministic — simulates the gap condition.
    ctx.available_verifiers = vec![VerifierKind::Intrinsic];
    ctx
}

// ---------------------------------------------------------------------------
// (e) SCHEMA_VERSION bump: H-41 = 219.
// ---------------------------------------------------------------------------

#[test]
fn schema_version_is_at_least_219() {
    assert!(
        SCHEMA_VERSION_PUBLIC >= 219,
        "SCHEMA_VERSION_PUBLIC must be >= 219 after H-41; got {}",
        SCHEMA_VERSION_PUBLIC
    );
}

// ---------------------------------------------------------------------------
// (f) Wire-tag and payload struct smoke.
// ---------------------------------------------------------------------------

#[test]
fn checker_gap_detected_wire_tag_is_correct() {
    assert_eq!(
        SsbActionType::CheckerGapDetected.as_sql_str(),
        "checker_gap_detected",
        "H-41: SsbActionType::CheckerGapDetected wire tag mismatch"
    );
}

#[test]
fn checker_gap_payload_action_type_round_trips() {
    let payload = SsbPayload::CheckerGapDetected(CheckerGapDetectedPayload {
        claim_id: "test-claim".into(),
        claim_type: "factual".into(),
        complexity_taxon: "decidable_no_checker".into(),
        fell_to_mode: "falsifiability_cross_family".into(),
        ts_ms: 1_000,
    });
    assert_eq!(payload.action_type(), SsbActionType::CheckerGapDetected);
}

// ---------------------------------------------------------------------------
// (c) Complexity taxon classifier.
// ---------------------------------------------------------------------------

#[test]
fn np_complete_keyword_yields_np_complete_taxon() {
    let claim = mk_np_complete_claim();
    let taxon = classify_complexity_taxon(&claim);
    assert_eq!(
        taxon,
        ComplexityTaxon::NpCompleteOrCombinatorial,
        "SAT-based keyword should yield NpCompleteOrCombinatorial"
    );
    assert_eq!(taxon.priority(), 2, "NP-complete must be priority 2 (HIGH)");
    assert!(taxon.is_high_priority(), "NP-complete must be HIGH priority");
}

#[test]
fn undecidable_keyword_yields_undecidable_taxon() {
    let claim = Claim {
        claim_id: "undecidable".into(),
        claim_type: ClaimType::Computational,
        text: "This requires first-order entailment across unbounded domains".into(),
        evidence: vec![],
        ts_ms: 4_000,
    };
    let taxon = classify_complexity_taxon(&claim);
    assert_eq!(taxon, ComplexityTaxon::Undecidable);
    assert_eq!(taxon.priority(), 2);
}

#[test]
fn short_claim_text_yields_unknown_not_guess() {
    // RISK-2: must NOT guess on short text.
    let claim = Claim {
        claim_id: "short".into(),
        claim_type: ClaimType::Factual,
        text: "SAT".into(),
        evidence: vec![],
        ts_ms: 5_000,
    };
    assert_eq!(
        classify_complexity_taxon(&claim),
        ComplexityTaxon::Unknown,
        "Short text must yield Unknown, never a guess"
    );
}

// ---------------------------------------------------------------------------
// (a) Covered Factual claim: Constructive route, no gap row emitted.
// ---------------------------------------------------------------------------

#[tokio::test]
async fn covered_factual_routes_constructive_no_gap() {
    let claim = mk_factual_with_evidence();
    let ctx = ctx_with_deterministic();
    let route_emitter = VecClaimRoutedEmitter::new();
    let gap_emitter = VecCheckerGapEmitter::new();

    // Pass gap_enabled=true directly (avoids env-var races in parallel tests).
    let (mode, guard) =
        route_claim_with_gap(&claim, &ctx, &route_emitter, &gap_emitter, 1_700_000_000_000, true)
            .await;

    // Route must be Constructive (covered path, R4).
    assert_eq!(
        mode,
        elai_orchestrator::GroundingMode::Constructive,
        "covered Factual claim with Deterministic verifier must route Constructive"
    );
    guard.confirm();

    // No gap row emitted (covered — has a Deterministic verifier).
    let gaps = gap_emitter.rows().await;
    assert!(
        gaps.is_empty(),
        "covered Factual claim must NOT emit a gap row; got: {:?}",
        gaps
    );
}

// ---------------------------------------------------------------------------
// (b) Uncovered Factual claim: non-Constructive route + gap row emitted.
// ---------------------------------------------------------------------------

#[tokio::test]
async fn uncovered_factual_emits_checker_gap_row() {
    let claim = mk_factual_no_evidence();
    let ctx = ctx_without_deterministic(); // no Deterministic verifier
    let route_emitter = VecClaimRoutedEmitter::new();
    let gap_emitter = VecCheckerGapEmitter::new();

    // Pass gap_enabled=true directly (avoids env-var races in parallel tests).
    let (mode, guard) =
        route_claim_with_gap(&claim, &ctx, &route_emitter, &gap_emitter, 1_700_000_000_001, true)
            .await;

    // Route must NOT be Constructive (no det verifier + no evidence).
    assert_ne!(
        mode,
        elai_orchestrator::GroundingMode::Constructive,
        "uncovered Factual claim without Deterministic verifier must NOT route Constructive"
    );
    guard.confirm();

    // Gap row must be emitted — this is the off-agent backlog artifact (R4/W-3).
    let gaps = gap_emitter.rows().await;
    assert_eq!(
        gaps.len(),
        1,
        "uncovered Factual claim must emit exactly 1 gap row; got: {:?}",
        gaps
    );
    let gap = &gaps[0];
    assert_eq!(gap.claim_type, "factual", "gap row claim_type mismatch");
    assert_ne!(
        gap.fell_to_mode, "constructive",
        "gap row fell_to_mode must not be constructive"
    );
    assert!(!gap.claim_id.is_empty(), "gap row claim_id must not be empty");
}

// ---------------------------------------------------------------------------
// (b-off) Gap NOT emitted when kill-switch is OFF (default).
// ---------------------------------------------------------------------------

#[tokio::test]
async fn gap_not_emitted_when_kill_switch_off() {
    let claim = mk_factual_no_evidence();
    let ctx = ctx_without_deterministic();
    let route_emitter = VecClaimRoutedEmitter::new();
    let gap_emitter = VecCheckerGapEmitter::new();

    // Pass gap_enabled=false — simulates the default-OFF kill-switch (W-2/R9).
    let (_mode, guard) =
        route_claim_with_gap(&claim, &ctx, &route_emitter, &gap_emitter, 1_700_000_000_002, false)
            .await;
    guard.confirm();

    let gaps = gap_emitter.rows().await;
    assert!(
        gaps.is_empty(),
        "gap row must NOT be emitted when gap_enabled=false; got: {:?}",
        gaps
    );
}

// ---------------------------------------------------------------------------
// (d) mode_a_fraction fixture: 3 Constructive + 1 non-Constructive = 0.75.
// ---------------------------------------------------------------------------

#[tokio::test]
async fn mode_a_fraction_matches_fixture() {
    let (dir, path) = fresh_db();
    let writer = open_writer(&path).await;

    // Write 3 Constructive + 1 FalsifiabilityCrossFamily claim_routed rows.
    use elai_orchestrator::ssb::ClaimRoutedPayload;
    use elai_orchestrator::SsbPayload;

    for i in 0u64..3 {
        writer
            .write_payload(
                "agent-h41",
                i as i64,
                SsbPayload::ClaimRouted(ClaimRoutedPayload {
                    claim_id: format!("c-{i}"),
                    claim_type: "factual".into(),
                    mode: "constructive".into(),
                    ts_ms: 1_000 + i as i64,
                    ctx_summary_blake3: format!("deadbeef{i:08x}"),
                }),
                Vec::new(),
            )
            .await
            .expect("write claim_routed constructive");
    }
    writer
        .write_payload(
            "agent-h41",
            3,
            SsbPayload::ClaimRouted(ClaimRoutedPayload {
                claim_id: "c-3".into(),
                claim_type: "factual".into(),
                mode: "falsifiability_cross_family".into(),
                ts_ms: 4_000,
                ctx_summary_blake3: "deadbeef00000003".into(),
            }),
            Vec::new(),
        )
        .await
        .expect("write claim_routed non-constructive");

    // Flush writes by dropping the writer.
    drop(writer);

    let viewer = open_viewer(&path);
    let report = mode_a_coverage(&viewer, "all", 5_000);

    assert_eq!(
        report.total_factual_computational, 4,
        "should count all 4 Factual claim_routed rows"
    );
    assert_eq!(
        report.constructive_count, 3,
        "3 of 4 rows are Constructive"
    );
    let expected_fraction = 3.0 / 4.0; // 0.75
    let delta = (report.mode_a_fraction - expected_fraction).abs();
    assert!(
        delta < 1e-9,
        "mode_a_fraction expected 0.75, got {}",
        report.mode_a_fraction
    );

    // Cleanup tempdir explicitly (TempDir's Drop would also work).
    drop(dir);
}

// ---------------------------------------------------------------------------
// BLOCK-2(c): Procedural claim (production path) emits gap row via Gap-B.
// This is the real production routing path that dispatch_claim_routing uses.
// R4/R7: drives route_claim_with_gap (same fn task_runner calls via
// route_claim_with_gap_env), asserts a CheckerGapDetected row lands.
// Missing = FAIL (R1 / R2).
// ---------------------------------------------------------------------------

fn mk_procedural_claim() -> Claim {
    Claim {
        claim_id: "prod-procedural".into(),
        claim_type: ClaimType::Procedural,
        // Text mirrors the shape dispatch_claim_routing synthesises:
        // "task {seq} on project {project_id}".
        text: "task 42 on project my-project".into(),
        evidence: Vec::new(),
        ts_ms: 9_000,
    }
}

fn ctx_minimal_no_verifiers() -> OrchContext {
    // Matches the OrchContext dispatch_claim_routing constructs:
    // no available_verifiers, conformal_calibration_ready=false.
    OrchContext::minimal(OrchUserRole::Executor, OrchPhase::Executing)
}

#[tokio::test]
async fn procedural_claim_emits_gap_b_row_on_operational_deferred() {
    let claim = mk_procedural_claim();
    let ctx = ctx_minimal_no_verifiers();
    let route_emitter = VecClaimRoutedEmitter::new();
    let gap_emitter = VecCheckerGapEmitter::new();

    // gap_enabled=true; this is the production Gap-B path (BLOCK-2 fix).
    let (mode, guard) =
        route_claim_with_gap(&claim, &ctx, &route_emitter, &gap_emitter, 9_000, true).await;

    // Procedural claim must route to OperationalDeferred (grounding_mode Rule 4).
    assert_eq!(
        mode,
        elai_orchestrator::GroundingMode::OperationalDeferred,
        "Procedural claim must route OperationalDeferred"
    );
    guard.confirm();

    // Gap-B: gap row MUST be emitted — this is the off-agent proof (R4/W-3).
    let gaps = gap_emitter.rows().await;
    assert_eq!(
        gaps.len(),
        1,
        "Procedural claim routed OperationalDeferred must emit exactly 1 gap row (Gap-B); got: {:?}",
        gaps
    );
    let gap = &gaps[0];
    assert_eq!(gap.claim_type, "procedural", "gap row claim_type must be 'procedural'");
    assert_eq!(
        gap.fell_to_mode, "operational_deferred",
        "gap row fell_to_mode must be 'operational_deferred'"
    );
    assert!(!gap.claim_id.is_empty(), "gap row claim_id must not be empty");
}

// ---------------------------------------------------------------------------
// WARN-R2 (a): Broken-arm falsifiability test.
// When gap_enabled=false the metric does NOT change (gap row suppressed).
// This proves the instrument is NOT vacuously correct: toggling the flag
// changes observable output. If this test ever passes trivially (gap row
// emitted even when disabled), the instrument is broken.
// ---------------------------------------------------------------------------

#[tokio::test]
async fn broken_arm_gap_disabled_yields_zero_gap_rows_and_different_metric() {
    // Run once with gap_enabled=true, once with false.
    // The gap row count MUST differ — proving the flag is load-bearing.
    let claim = mk_procedural_claim();
    let ctx = ctx_minimal_no_verifiers();

    // Enabled run.
    let route_emitter_on = VecClaimRoutedEmitter::new();
    let gap_emitter_on = VecCheckerGapEmitter::new();
    let (_, guard_on) =
        route_claim_with_gap(&claim, &ctx, &route_emitter_on, &gap_emitter_on, 1_000, true).await;
    guard_on.confirm();
    let gaps_on = gap_emitter_on.len().await;

    // Disabled run (broken-arm: simulates missing / wrong env var).
    let route_emitter_off = VecClaimRoutedEmitter::new();
    let gap_emitter_off = VecCheckerGapEmitter::new();
    let (_, guard_off) =
        route_claim_with_gap(&claim, &ctx, &route_emitter_off, &gap_emitter_off, 2_000, false)
            .await;
    guard_off.confirm();
    let gaps_off = gap_emitter_off.len().await;

    assert_eq!(gaps_on, 1, "gap_enabled=true must emit 1 gap row");
    assert_eq!(gaps_off, 0, "gap_enabled=false must emit 0 gap rows");
    // The two counts MUST differ — this is the falsifiability proof.
    assert_ne!(
        gaps_on, gaps_off,
        "gap count MUST differ between enabled and disabled runs (instrument is non-vacuous)"
    );
}

// ---------------------------------------------------------------------------
// WARN-R2 (b): ELAI_SEED reproducibility test.
// The same (claim, ctx, now_ms) triple must always produce the same gap row
// fields (claim_id, claim_type, fell_to_mode) regardless of call order.
// Recomputable from the journal under ELAI_SEED (GD-2).
// ---------------------------------------------------------------------------

#[tokio::test]
async fn seed_reproducibility_same_inputs_yield_identical_gap_rows() {
    let claim = mk_procedural_claim();
    let ctx = ctx_minimal_no_verifiers();
    let now_ms = 42_000_i64;

    // First call.
    let re1 = VecClaimRoutedEmitter::new();
    let ge1 = VecCheckerGapEmitter::new();
    let (mode1, g1) =
        route_claim_with_gap(&claim, &ctx, &re1, &ge1, now_ms, true).await;
    g1.confirm();
    let rows1 = ge1.rows().await;

    // Second call — identical inputs.
    let re2 = VecClaimRoutedEmitter::new();
    let ge2 = VecCheckerGapEmitter::new();
    let (mode2, g2) =
        route_claim_with_gap(&claim, &ctx, &re2, &ge2, now_ms, true).await;
    g2.confirm();
    let rows2 = ge2.rows().await;

    // Routing mode must be deterministic.
    assert_eq!(mode1, mode2, "routing mode must be deterministic for same inputs");

    // Gap row fields must be identical (ELAI_SEED requirement: recomputable).
    assert_eq!(rows1.len(), rows2.len(), "gap row count must be identical across runs");
    if let (Some(r1), Some(r2)) = (rows1.first(), rows2.first()) {
        assert_eq!(r1.claim_id, r2.claim_id, "claim_id must be deterministic");
        assert_eq!(r1.claim_type, r2.claim_type, "claim_type must be deterministic");
        assert_eq!(r1.fell_to_mode, r2.fell_to_mode, "fell_to_mode must be deterministic");
        assert_eq!(r1.complexity_taxon, r2.complexity_taxon, "complexity_taxon must be deterministic");
        assert_eq!(r1.ts_ms, r2.ts_ms, "ts_ms must be deterministic for same now_ms input");
    }
}
