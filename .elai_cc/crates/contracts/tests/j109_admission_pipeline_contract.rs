//! J-109 UNIT A1 contract: the pure admission-pipeline decision function.
//!
//! Arms (plan A1.4):
//!   (a) doctrine-duplicate text vs a fixture manifest -> `RejectImmutableSpan`;
//!   (b) unrelated text -> `AdmitClean`;
//!   (c) stale fingerprint -> the index is rebuilt BEFORE any verdict is computed;
//!   (d) CORRECTION ARM: a fixture span shaped like a cwd CLAUDE.md member trips
//!       the reject, proving the config.rs:316 root seeding makes cwd CLAUDE.md /
//!       AGENTS.md manifest members BY CONSTRUCTION;
//!   (e) receipts are recomputable from the emitted fields alone (GD-2).
//! Plus wire-shape pins for the SSB `admission_verdict` arm (stringify +
//! route-from-string + derived SCHEMA_VERSION) and a msgpack roundtrip of the
//! net-new [`AdmissionVerdictPayload`].
//!
//! These arms are PURE-FN arms by design: the seam-level falsifiable-mutation
//! test lives IN-MODULE in `crates/cli/src/user_memory_capture.rs` (deleting the
//! production call site turns that test red while these stay green).

use elai_core::config::ConfigLoader;
use elai_orchestrator::admission_pipeline::{
    lexical_overlap, rebuild_if_stale, rows_fingerprint, tokenize, AdmissionCheckVerdict,
    AdmissionPipeline, StaticSpanIndex, SURFACE_LEG_LEXICAL, T_IMMUTABLE_LEXICAL,
};
// E0432 fix: AdmissionVerdictPayload is defined in ssb::payloads (not
// admission_pipeline); the ssb facade re-exports payloads::*.
use elai_orchestrator::ssb::{
    ssb_action_type_from_str_public, AdmissionVerdictPayload, SsbActionType, SsbPayload,
    SCHEMA_VERSION_PUBLIC,
};

/// Doctrine line shared by the fixture manifests (single source for arms a/e).
const DOCTRINE_LINE: &str = "Always run the auditor before every merge to main.";

/// Fixture: temp project root whose ONLY instruction-surface member is a cwd
/// CLAUDE.md carrying the doctrine line. Returns (root, scan rows).
fn fixture_root_with_claude_md(body: &str) -> (tempfile::TempDir, Vec<elai_core::config::AdmissionScanRow>) {
    let root = tempfile::tempdir().expect("tempdir");
    std::fs::write(root.path().join("CLAUDE.md"), body).expect("write fixture CLAUDE.md");
    // NO .claude dir is created: the ONLY member is the config.rs:316 seeding.
    let rows = ConfigLoader::new(root.path(), root.path()).admission_scan();
    (root, rows)
}

/// Arm (a): a candidate that restates doctrine trips the immutable-span reject.
#[test]
fn j109_a_doctrine_duplicate_trips_immutable_reject() {
    let (_root, rows) = fixture_root_with_claude_md(&format!("# Doctrine\n\n{DOCTRINE_LINE}\n\nBuild the harness before the paint.\n"));
    assert!(!rows.is_empty(), "cwd CLAUDE.md is a manifest member by construction");
    let index = StaticSpanIndex::from_scan_rows(&rows);
    let verdict = AdmissionPipeline.check_candidate(&index, DOCTRINE_LINE);
    assert_eq!(verdict, AdmissionCheckVerdict::RejectImmutableSpan);
}

/// Arm (b): unrelated text with disjoint vocabulary admits clean.
#[test]
fn j109_b_unrelated_text_admits_clean() {
    let (_root, rows) = fixture_root_with_claude_md(&format!("# Doctrine\n\n{DOCTRINE_LINE}\n"));
    let index = StaticSpanIndex::from_scan_rows(&rows);
    let verdict = AdmissionPipeline.check_candidate(
        &index,
        "zebra quintuplets migrate across glassy tundra archipelagos nightly",
    );
    assert_eq!(verdict, AdmissionCheckVerdict::AdmitClean);
}

/// Arm (c): staleness is resolved BEFORE any verdict — after the manifest
/// changes, `rebuild_if_stale` must fire first, and only then does the new
/// doctrine trip the reject (a verdict computed against the stale cache would
/// have admitted it).
#[test]
fn j109_c_stale_index_rebuilds_before_any_verdict() {
    let (root, rows_v1) =
        fixture_root_with_claude_md("# Notes\n\nThe garden fence is painted green.\n");
    let mut cache: Option<StaticSpanIndex> = None;
    assert!(rebuild_if_stale(&mut cache, &rows_v1), "first build is a rebuild");
    let pipeline = AdmissionPipeline;
    let idx_v1 = cache.as_ref().expect("index after first build");
    assert_eq!(
        pipeline.check_candidate(idx_v1, "quokkas photograph poorly"),
        AdmissionCheckVerdict::AdmitClean
    );

    // The SAME manifest member changes: a new doctrine line appears.
    std::fs::write(
        root.path().join("CLAUDE.md"),
        "# Notes\n\nThe garden fence is painted green.\nTrust the deterministic gate over the hopeful one.\n",
    )
    .expect("rewrite fixture CLAUDE.md");
    let rows_v2 = ConfigLoader::new(root.path(), root.path()).admission_scan();
    assert_ne!(
        rows_fingerprint(&rows_v1),
        rows_fingerprint(&rows_v2),
        "membership/content change must move the fingerprint"
    );
    assert!(!rebuild_if_stale(&mut cache, &rows_v1), "unchanged rows => no rebuild");
    assert!(rebuild_if_stale(&mut cache, &rows_v2), "stale cache rebuilt BEFORE verdicts");
    let idx_v2 = cache.as_ref().expect("index after rebuild");
    assert_eq!(
        pipeline.check_candidate(idx_v2, "trust the deterministic gate over the hopeful one"),
        AdmissionCheckVerdict::RejectImmutableSpan,
        "the verdict AFTER the rebuild sees the new doctrine span"
    );
}

/// Arm (d) CORRECTION ARM: with NO .claude directory at all, the scan yields
/// exactly the seeded cwd CLAUDE.md member (config.rs:316) and a duplicate of
/// its content trips the reject — root members are indexed BY CONSTRUCTION.
#[test]
fn j109_d_cwd_claude_md_member_indexed_by_construction() {
    let (_root, rows) = fixture_root_with_claude_md("# Doctrine\n\nShip the smallest correct change.\n");
    assert_eq!(
        rows.len(),
        1,
        "ONLY the seeded cwd CLAUDE.md member exists (config.rs:316 seeding)"
    );
    assert!(
        rows[0].path.ends_with("CLAUDE.md"),
        "the single member is the cwd CLAUDE.md root file"
    );
    let index = StaticSpanIndex::from_scan_rows(&rows);
    let verdict = AdmissionPipeline.check_candidate(&index, "ship the smallest correct change");
    assert_eq!(verdict, AdmissionCheckVerdict::RejectImmutableSpan);
}

/// Arm (e): the receipts bundle is sufficient to recompute the decision from
/// the emitted fields alone (GD-2) — hash pins WHAT ran, leg + threshold pin
/// HOW, and the pure scorer reproduces the verdict from the index.
#[test]
fn j109_e_receipts_recomputable_from_emitted_fields() {
    let (_root, rows) = fixture_root_with_claude_md(&format!("# Doctrine\n\n{DOCTRINE_LINE}\n"));
    let index = StaticSpanIndex::from_scan_rows(&rows);
    let evaluation = AdmissionPipeline.evaluate_candidate(&index, DOCTRINE_LINE);
    let receipts = &evaluation.receipts;

    assert_eq!(
        receipts.candidate_hash,
        blake3::hash(DOCTRINE_LINE.as_bytes()).to_hex().to_string(),
        "candidate_hash = blake3 of the exact probed text"
    );
    assert_eq!(receipts.surface_leg, SURFACE_LEG_LEXICAL);
    assert_eq!(receipts.surface_leg, "lexical");
    assert!(
        (receipts.threshold_used - T_IMMUTABLE_LEXICAL).abs() < f32::EPSILON,
        "threshold_used names the constant that decided"
    );
    assert_eq!(receipts.verdict, evaluation.verdict.as_receipt_str());
    assert_eq!(receipts.verdict, "reject_immutable_span");

    // Full recompute from row fields + the replayed candidate text (GD-2).
    let candidate_tokens = tokenize(DOCTRINE_LINE);
    let best = index
        .spans()
        .iter()
        .map(|span| lexical_overlap(&candidate_tokens, &span.tokens))
        .fold(0.0_f32, f32::max);
    let recomputed = if best >= T_IMMUTABLE_LEXICAL {
        "reject_immutable_span"
    } else {
        "admit_clean"
    };
    assert_eq!(receipts.verdict, recomputed, "row alone must recompute the decision");
}

/// Wire-shape pin: the SSB arm stringify + route-from-string both directions,
/// and the derived SCHEMA_VERSION after the net +1 registry append (620 -> 621;
/// SCHEMA_VERSION = -309 + 621 = 312; re-read and re-pin forward if an upstream
/// schema bump landed meanwhile — W-9).
#[test]
fn j109_wire_admission_verdict_stringify_and_schema_pin() {
    assert_eq!(SsbActionType::AdmissionVerdict.as_sql_str(), "admission_verdict");
    // E0369 fix: SsbError does not implement PartialEq, so the Result cannot
    // be compared with assert_eq!; match on the shape instead.
    assert!(matches!(
        ssb_action_type_from_str_public("admission_verdict"),
        Ok(SsbActionType::AdmissionVerdict)
    ));
    assert_eq!(SCHEMA_VERSION_PUBLIC, 312, "net +1 arm: 620 -> 621 registry len");
}

/// Msgpack roundtrip of the net-new payload (the exact bytes the SSB journal
/// stores must decode back to the same receipts, deny_unknown_fields enforced).
#[test]
fn j109_wire_admission_verdict_payload_roundtrip() {
    // E0609 fix: field access must go through the payload struct, not the
    // SsbPayload enum wrapper; bind the inner record and wrap a clone.
    let sent = AdmissionVerdictPayload {
        candidate_hash: blake3::hash(DOCTRINE_LINE.as_bytes()).to_hex().to_string(),
        surface_leg: "lexical".to_string(),
        threshold: T_IMMUTABLE_LEXICAL,
        verdict: "reject_immutable_span".to_string(),
        originating_seam: "e5_capture_remember".to_string(),
        ts: 1_790_000_000_000,
    };
    let payload = SsbPayload::AdmissionVerdict(sent.clone());
    let bytes = rmp_serde::to_vec(&payload).expect("encode payload");
    let decoded: SsbPayload = rmp_serde::from_slice(&bytes).expect("decode payload");
    match decoded {
        SsbPayload::AdmissionVerdict(p) => {
            assert_eq!(p.candidate_hash, sent.candidate_hash);
            assert_eq!(p.surface_leg, "lexical");
            assert!((p.threshold - T_IMMUTABLE_LEXICAL).abs() < f32::EPSILON);
            assert_eq!(p.verdict, "reject_immutable_span");
            assert_eq!(p.originating_seam, "e5_capture_remember");
            assert_eq!(p.ts, 1_790_000_000_000);
        }
        other => panic!("expected AdmissionVerdict payload, got {other:?}"),
    }
}
