//! G-8 -- signed in-toto v1 provenance attestation build/sign/verify contract.
//! THE deterministic Tier-1 fail-closed gate for the off-machine trust extension.
//!
//! These tests drive the REAL `build_attestation -> sign_local_merkle ->
//! verify_attestation` path over a fixture G-2 [`RunArtifact`] built through a
//! LIVE `Ssb` (the same writer + reader the production `elai run` exit uses), NOT
//! a hand-mocked statement. The off-agent artifact asserted is the blake3-Merkle
//! subject digest + the verify verdict + the provenance_attestation_emitted SSB
//! row the run-end emit writes.
//!
//! Coverage (mirrors G-8-decisions.md EXPECTED TEST OUTCOME):
//!   (a) build_attestation reads the G-2 manifest (GD-1, re-implements no F-gate),
//!       builds the in-toto Statement, signs via blake3-Merkle -> verify ==
//!       VERIFIED; the verified_levels rollup covers all 4 F-gates.
//!   (b) the provenance_attestation_emitted SSB row round-trips through the LIVE
//!       Ssb writer + msgpack decode + the action_type bijection, and the
//!       SsbViewer::attestations drawer read (the dashboard get_attestations data
//!       source) surfaces it with the digest + rollup intact.
//!   (c) a TAMPERED predicate (flip one F-gate verdict hash) -> the recomputed
//!       Merkle root != the subject digest -> NOT-VERIFIED (never a silent pass).
//!   (d) the trailer-binding verify (verify_attestation_against_trailer): the
//!       correct trailer digest -> VERIFIED; a WRONG trailer digest -> NOT-VERIFIED
//!       (the git-absent fabrication guard's in-process analog).
//!   (e) GD-2: the blake3-Merkle subject digest re-computes byte-identically from
//!       the same RunArtifact under a FIXED ELAI_SEED across two independent
//!       build_attestation calls.
//!   (f) ARTICLE-50 HONESTY (E-38, DD6): the predicate posture string says
//!       "designed to support" + cites the EUR 15M / 3% transparency tier and
//!       does NOT say "certified" / 35M / 7%.
//!   (g) the sidecar write -> read round-trips the Statement byte-faithfully.
//!
//! No #[ignore] / stubs / placeholder assertions. No #[traced_test] (we assert on
//! the statement bytes + the verdict + the SSB row, not on logs).

use std::sync::Arc;

use elai_core::clock::OsClock;
use elai_orchestrator::attestation::{
    build_attestation, read_sidecar, sign_local_merkle, verify_attestation,
    verify_attestation_against_trailer, write_sidecar, AttestationVerdict,
};
use elai_orchestrator::determinism::{DeterminismConfig, GpuSku, HardwareClass};
use elai_orchestrator::rra::build_rra;
use elai_orchestrator::ssb::{
    ssb_action_type_from_str_public, LoopKind, ModelSwapPayload,
    ProvenanceAttestationEmittedPayload, SecurityClassRegressionBlockedPayload, Ssb, SsbConfig,
    SsbPayload, TestIntegrityEventKind, TestIntegrityEventPayload, VibeGateBlockedKind,
    VibeGateBlockedPayload, WriteTimeSlopVerdictKind, WriteTimeSlopsquatBlockedPayload,
};
use elai_orchestrator::ssb_viewer::SsbViewer;
use elai_orchestrator::SsbReader as _;
use tempfile::TempDir;

const FIXTURE_SESSION: &str = "g8-attestation-session";

fn make_ssb(tmp: &TempDir, suffix: &str) -> Arc<Ssb> {
    let cfg = SsbConfig {
        db_path: tmp.path().join(format!("ssb_g8_{suffix}.db")),
        pool_size: Some(2),
        project_id: "default".to_string(),
        session_id: FIXTURE_SESSION.to_string(),
    };
    let clock: Arc<dyn elai_core::clock::ClockProvider> = Arc::new(OsClock);
    Arc::new(Ssb::new(cfg, clock).expect("ssb init"))
}

/// A pinned local determinism config (seed + non-Unknown hardware class).
fn local_pins(seed: u64) -> DeterminismConfig {
    DeterminismConfig {
        hardware_class: HardwareClass::Gpu(GpuSku::new("rtx3090")),
        seed,
        blake3_cache_prefix: [0u8; 8],
        system_fingerprint_pin: Some("local-fp-pin".to_string()),
    }
}

/// Write the four F-gate fixture rows + a local model_swap into the live SSB,
/// then return the full ordered stream (mirrors the G-2 roundtrip fixture).
async fn build_fixture_stream(ssb: &Arc<Ssb>) -> Vec<elai_orchestrator::ssb::SsbEntry> {
    let mut superstep = 0i64;

    ssb.write_payload(
        "executing",
        superstep,
        SsbPayload::VibeGateBlocked(VibeGateBlockedPayload {
            kind: VibeGateBlockedKind::CodeVuln,
            rule_id: Some("CWE-89".into()),
            cwe: Some("CWE-89".into()),
            file_path: "src/main.rs".into(),
            source_blake3: "f1aa11bb22cc33dd".into(),
            citation: None,
            fsm_state: Some("executing".into()),
        }),
        Vec::new(),
    )
    .await
    .expect("write F-1 row");
    superstep += 1;

    ssb.write_payload(
        "executing",
        superstep,
        SsbPayload::TestIntegrityEvent(TestIntegrityEventPayload {
            test_name: "crate::tests::foo".into(),
            passed: true,
            stdout_hash: Some("f4cc44dd55ee66ff".into()),
            kind: TestIntegrityEventKind::Verified,
        }),
        Vec::new(),
    )
    .await
    .expect("write F-4 row");
    superstep += 1;

    ssb.write_payload(
        "executing",
        superstep,
        SsbPayload::WriteTimeSlopsquatBlocked(WriteTimeSlopsquatBlockedPayload {
            dep_name_hash: 0xF9_0000_0000_0001,
            ecosystem: "cargo".into(),
            verdict_kind: WriteTimeSlopVerdictKind::NotFound.as_str().to_string(),
            registry_age_days: None,
            ts_ms: 1_700_000_000_000,
        }),
        Vec::new(),
    )
    .await
    .expect("write F-9 row");
    superstep += 1;

    ssb.write_payload(
        "executing",
        superstep,
        SsbPayload::SecurityClassRegressionBlocked(SecurityClassRegressionBlockedPayload {
            delta_blake3: "f10aabbccddeeff00".into(),
            introduced_classes: vec!["CWE-78".into()],
            prior_classes: vec![],
            loop_kind: LoopKind::Refine,
            loop_id: "refine-1".into(),
            lang: "rust".into(),
        }),
        Vec::new(),
    )
    .await
    .expect("write F-10 row");
    superstep += 1;

    ssb.write_payload(
        "executing",
        superstep,
        SsbPayload::ModelSwap(ModelSwapPayload {
            from_model: "bootstrap".into(),
            to_model: "qwen2.5-coder-7b-instruct-abliterated".into(),
            gpu_id: 0,
            swap_duration_ms: 10,
            reason: "fixture".into(),
        }),
        Vec::new(),
    )
    .await
    .expect("write model_swap row");

    ssb.query_session_stream_asc(FIXTURE_SESSION)
        .expect("read session stream asc")
}

// ---------------------------------------------------------------------------
// (a) build_attestation -> verify == Verified, rollup covers all 4 F-gates.
// ---------------------------------------------------------------------------

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn build_attestation_verifies_and_rollup_covers_all_gates() {
    let tmp = TempDir::new().expect("tempdir");
    let ssb = make_ssb(&tmp, "ok");
    let stream = build_fixture_stream(&ssb).await;

    let pins = local_pins(42);
    let art = build_rra(FIXTURE_SESSION, &stream, &pins);

    let statement = build_attestation(&art).expect("build attestation");

    // GD-1: the verified_levels rollup CONSUMES the G-2 manifest (4 gates).
    let gates: Vec<&str> = statement
        .predicate
        .verified_levels
        .iter()
        .map(|v| v.gate_hash.gate.as_str())
        .collect();
    assert!(gates.contains(&"F-1"), "F-1 must be in verified_levels");
    assert!(gates.contains(&"F-4"), "F-4 must be in verified_levels");
    assert!(gates.contains(&"F-9"), "F-9 must be in verified_levels");
    assert!(gates.contains(&"F-10"), "F-10 must be in verified_levels");
    assert!(
        statement.predicate.verified_levels.iter().all(|v| v.passed),
        "every present verified-level is a passed gate"
    );

    // The verified_levels reuse the G-2 GateVerdictHash verbatim (ssb_event_seq:i64
    // field, REUSE-DON'T-FORK): the verdict hashes match the run-artifact manifest.
    for v in &statement.predicate.verified_levels {
        let matched = art
            .manifest
            .iter()
            .any(|m| m.gate == v.gate_hash.gate && m.verdict_blake3 == v.gate_hash.verdict_blake3);
        assert!(matched, "verified_level must reuse the RunArtifact manifest verdict hash");
    }

    // The attestation chains to the G-2 root.
    assert_eq!(statement.predicate.run_artifact_root, art.root_blake3);

    // OFFLINE verify: recompute the Merkle root, compare to the subject digest.
    let verdict = verify_attestation(&statement).expect("verify");
    match verdict {
        AttestationVerdict::Verified { merkle_root } => {
            assert_eq!(
                Some(merkle_root.as_str()),
                statement.subject_digest(),
                "the verified root must equal the subject digest"
            );
        }
        AttestationVerdict::NotVerified { expected, got } => {
            panic!("expected Verified; got NotVerified (expected={expected}, got={got})");
        }
    }

    // The subject name is the session id, NOT a commit SHA (chicken-and-egg).
    assert_eq!(
        statement.subject.first().map(|s| s.name.as_str()),
        Some(FIXTURE_SESSION)
    );
}

// ---------------------------------------------------------------------------
// (b) the provenance_attestation_emitted SSB row round-trips + viewer reads it.
// ---------------------------------------------------------------------------

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn attestation_row_roundtrips_and_reads_via_viewer() {
    let tmp = TempDir::new().expect("tempdir");
    let ssb = make_ssb(&tmp, "row");
    let stream = build_fixture_stream(&ssb).await;
    let art = build_rra(FIXTURE_SESSION, &stream, &local_pins(7));
    let statement = build_attestation(&art).expect("build attestation");
    let subject = statement.subject_digest().expect("subject digest").to_owned();

    let payload = SsbPayload::ProvenanceAttestationEmitted(ProvenanceAttestationEmittedPayload {
        session_id: FIXTURE_SESSION.into(),
        subject_blake3: subject.clone(),
        run_artifact_root: art.root_blake3.clone(),
        verified_levels: 4,
        all_passed: true,
        signing_model: "local_blake3_merkle".into(),
    });
    assert_eq!(
        payload.action_type(),
        elai_orchestrator::ssb::SsbActionType::ProvenanceAttestationEmitted,
        "payload must map to the ProvenanceAttestationEmitted action type"
    );

    // Write through the LIVE writer (the path the run_async exit uses).
    ssb.write_payload("cli:run-attestation", 0, payload.clone(), Vec::new())
        .await
        .expect("write provenance_attestation_emitted row");

    // round-trip via query_by_type + msgpack decode.
    let rows = ssb
        .query_by_type(elai_orchestrator::ssb::SsbActionType::ProvenanceAttestationEmitted)
        .expect("query provenance_attestation_emitted");
    assert_eq!(rows.len(), 1, "exactly one attestation row");
    let decoded: SsbPayload = rmp_serde::from_slice(&rows[0].payload).expect("decode");
    match decoded {
        SsbPayload::ProvenanceAttestationEmitted(p) => {
            assert_eq!(p.subject_blake3, subject);
            assert_eq!(p.run_artifact_root, art.root_blake3);
            assert_eq!(p.verified_levels, 4);
            assert!(p.all_passed);
            assert_eq!(p.signing_model, "local_blake3_merkle");
        }
        other => panic!("expected ProvenanceAttestationEmitted, got {other:?}"),
    }

    // the dashboard read path (SsbViewer::attestations) surfaces the row.
    let viewer = SsbViewer::open(&tmp.path().join("ssb_g8_row.db")).expect("open viewer");
    let drawer_rows = viewer.attestations(50).expect("attestations");
    assert_eq!(drawer_rows.len(), 1, "viewer must surface the attestation");
    let row = &drawer_rows[0];
    assert_eq!(row.subject_blake3, subject);
    assert_eq!(row.run_artifact_root, art.root_blake3);
    assert_eq!(row.verified_levels, 4);
    assert!(row.all_passed);
    assert_eq!(row.signing_model, "local_blake3_merkle");

    // action_type wire-label bijection.
    let variant = elai_orchestrator::ssb::SsbActionType::ProvenanceAttestationEmitted;
    assert_eq!(variant.as_sql_str(), "provenance_attestation_emitted");
    assert_eq!(
        ssb_action_type_from_str_public("provenance_attestation_emitted").expect("parse"),
        variant
    );
}

// ---------------------------------------------------------------------------
// (c) a TAMPERED predicate -> NOT-VERIFIED (never a silent pass).
// ---------------------------------------------------------------------------

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn tampered_predicate_does_not_verify() {
    let tmp = TempDir::new().expect("tempdir");
    let ssb = make_ssb(&tmp, "tamper");
    let stream = build_fixture_stream(&ssb).await;
    let art = build_rra(FIXTURE_SESSION, &stream, &local_pins(42));

    let mut statement = build_attestation(&art).expect("build attestation");
    // Sanity: the untampered statement verifies.
    assert!(matches!(
        verify_attestation(&statement).expect("verify"),
        AttestationVerdict::Verified { .. }
    ));

    // TAMPER: flip one F-gate verdict hash in the predicate (the subject digest
    // is NOT re-patched, so the recomputed Merkle root no longer matches).
    let target = statement
        .predicate
        .verified_levels
        .get_mut(0)
        .expect("at least one verified level");
    target.gate_hash.verdict_blake3 = "deadbeefdeadbeef".into();

    match verify_attestation(&statement).expect("verify tampered") {
        AttestationVerdict::NotVerified { expected, got } => {
            assert_ne!(expected, got, "the tampered root must differ from the subject digest");
        }
        AttestationVerdict::Verified { .. } => {
            panic!("a tampered predicate MUST NOT verify (fail-closed, R2)");
        }
    }
}

// ---------------------------------------------------------------------------
// (d) trailer-binding verify: right trailer -> Verified; wrong trailer -> not.
//     (the in-process analog of the git-absent fabrication guard).
// ---------------------------------------------------------------------------

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn trailer_binding_verify_rejects_wrong_digest() {
    let tmp = TempDir::new().expect("tempdir");
    let ssb = make_ssb(&tmp, "trailer");
    let stream = build_fixture_stream(&ssb).await;
    let art = build_rra(FIXTURE_SESSION, &stream, &local_pins(42));
    let statement = build_attestation(&art).expect("build attestation");
    let real_digest = statement.subject_digest().expect("subject digest").to_owned();

    // Correct trailer digest -> Verified.
    match verify_attestation_against_trailer(&statement, &real_digest).expect("verify") {
        AttestationVerdict::Verified { merkle_root } => assert_eq!(merkle_root, real_digest),
        AttestationVerdict::NotVerified { .. } => {
            panic!("the correct trailer digest must verify")
        }
    }

    // WRONG trailer digest -> NOT-VERIFIED (the verify never fabricates a pass
    // just because the predicate is internally consistent).
    match verify_attestation_against_trailer(&statement, "0000000000000000").expect("verify") {
        AttestationVerdict::NotVerified { expected, .. } => {
            assert_eq!(expected, "0000000000000000");
        }
        AttestationVerdict::Verified { .. } => {
            panic!("a wrong trailer digest MUST NOT verify (fail-closed)")
        }
    }
}

// ---------------------------------------------------------------------------
// (e) GD-2: the blake3-Merkle digest re-computes byte-identically.
// ---------------------------------------------------------------------------

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn merkle_root_recomputes_byte_identically_under_fixed_seed() {
    let tmp = TempDir::new().expect("tempdir");
    let ssb = make_ssb(&tmp, "gd2");
    let stream = build_fixture_stream(&ssb).await;

    // Two independent build_attestation calls over the SAME stream + SAME seed.
    let art_a = build_rra(FIXTURE_SESSION, &stream, &local_pins(1234));
    let art_b = build_rra(FIXTURE_SESSION, &stream, &local_pins(1234));
    let stmt_a = build_attestation(&art_a).expect("build a");
    let stmt_b = build_attestation(&art_b).expect("build b");

    assert_eq!(
        stmt_a.subject_digest(),
        stmt_b.subject_digest(),
        "the blake3-Merkle subject digest must re-compute byte-identically (GD-2)"
    );
    // And the sign fn is itself deterministic over the predicate.
    let resigned = sign_local_merkle(&stmt_a.predicate).expect("re-sign");
    assert_eq!(Some(resigned.as_str()), stmt_a.subject_digest());
}

// ---------------------------------------------------------------------------
// (f) ARTICLE-50 HONESTY (E-38, DD6): the posture string is honest.
// ---------------------------------------------------------------------------

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn article_50_posture_is_honest() {
    let tmp = TempDir::new().expect("tempdir");
    let ssb = make_ssb(&tmp, "art50");
    let stream = build_fixture_stream(&ssb).await;
    let art = build_rra(FIXTURE_SESSION, &stream, &local_pins(42));
    let statement = build_attestation(&art).expect("build attestation");

    let posture = statement.predicate.article_50_posture.to_ascii_lowercase();
    assert!(
        posture.contains("designed to support"),
        "must say 'designed to support', not 'certified': {posture}"
    );
    assert!(
        posture.contains("not certified compliant"),
        "Article-50 posture must carry the 'NOT certified compliant' denial; got: {posture}"
    );
    // The transparency tier (EUR 15M / 3%, Article-99 Tier-2), NOT the Art-5
    // prohibited-practices tier (EUR 35M / 7%).
    assert!(posture.contains("15m") && posture.contains("3%"), "must cite EUR 15M / 3%: {posture}");
    assert!(!posture.contains("35m"), "must NOT pair with the EUR 35M tier: {posture}");
    assert!(!posture.contains("7%"), "must NOT pair with the 7% tier: {posture}");
}

// ---------------------------------------------------------------------------
// (g) the sidecar write -> read round-trips the Statement byte-faithfully.
// ---------------------------------------------------------------------------

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn sidecar_write_read_roundtrips() {
    let tmp = TempDir::new().expect("tempdir");
    let ssb = make_ssb(&tmp, "sidecar");
    let stream = build_fixture_stream(&ssb).await;
    let art = build_rra(FIXTURE_SESSION, &stream, &local_pins(42));
    let statement = build_attestation(&art).expect("build attestation");

    let path = tmp.path().join("attestation.intoto.json");
    write_sidecar(&statement, &path).expect("write sidecar");
    let read_back = read_sidecar(&path).expect("read sidecar");

    assert_eq!(read_back, statement, "sidecar must round-trip byte-faithfully");
    // And the read-back statement still verifies OFFLINE.
    assert!(matches!(
        verify_attestation(&read_back).expect("verify read-back"),
        AttestationVerdict::Verified { .. }
    ));
}
