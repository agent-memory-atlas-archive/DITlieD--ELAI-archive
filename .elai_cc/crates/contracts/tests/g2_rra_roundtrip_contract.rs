//! G-2 -- Reproducible-Run Artifact (RRA) build/write/verify round-trip
//! contract tests. THE KEYSTONE deterministic Tier-1 fail-closed gate.
//!
//! These tests drive the REAL `build_rra -> write_rra -> verify_rra` path over a
//! fixture SSB stream built through a LIVE `Ssb` (the same writer + reader the
//! production `elai run` exit uses), NOT a hand-mocked artifact. The off-agent
//! artifact asserted is the blake3 root recorded on disk + the verify verdict.
//!
//! Coverage (mirrors G-2-decisions.md EXPECTED TEST OUTCOME):
//!   (a) build_rra over a fixture stream + gate-ON F-1/F-4/F-9/F-10 rows ->
//!       write_rra -> .rra file; verify_rra == Verified (stream + every present
//!       F-gate verdict hash reproduced) AND the manifest has all 4 gates.
//!   (b) a TAMPERED .rra (flip the recorded root) -> Tampered.
//!   (c) a DIVERGENCE .rra (flip one event payload byte, root re-patched) ->
//!       Divergence naming the FIRST mismatching event.
//!   (d) gate-OFF F-1 (no VibeGateBlocked row) -> manifest entry absent, still
//!       Verified (a valid gate-off state, NOT a failure).
//!   (e) GD-2: the blake3 root re-computes byte-identically from the stream
//!       under a FIXED ELAI_SEED across two independent build_rra calls.
//!   (f) GD-5 HONEST SCOPE: a cloud model_swap in the stream forces the
//!       guarantee level to VerifyDivergence, NEVER ReproducibleTokens.
//!
//! No #[ignore] / stubs / placeholder assertions. No #[traced_test] (we assert
//! on the .rra bytes + the verdict, not on logs).

use std::sync::Arc;

use elai_core::clock::OsClock;
use elai_orchestrator::determinism::{DeterminismConfig, GpuSku, HardwareClass};
use elai_orchestrator::rra::{
    build_rra, verify_artifact, verify_rra, write_rra, GuaranteeLevel, ReplayVerdict,
};
use elai_orchestrator::ssb::{
    LoopKind, ModelSwapPayload, SecurityClassRegressionBlockedPayload, Ssb, SsbConfig, SsbPayload,
    TestIntegrityEventKind, TestIntegrityEventPayload, VibeGateBlockedKind, VibeGateBlockedPayload,
    WriteTimeSlopVerdictKind, WriteTimeSlopsquatBlockedPayload,
};
use tempfile::TempDir;

const FIXTURE_SESSION: &str = "g2-rra-session";

fn make_ssb(tmp: &TempDir, suffix: &str) -> Arc<Ssb> {
    let cfg = SsbConfig {
        db_path: tmp.path().join(format!("ssb_g2_{suffix}.db")),
        pool_size: Some(2),
        project_id: "default".to_string(),
        session_id: FIXTURE_SESSION.to_string(),
    };
    let clock: Arc<dyn elai_core::clock::ClockProvider> = Arc::new(OsClock);
    Arc::new(Ssb::new(cfg, clock).expect("ssb init"))
}

/// A pinned local determinism config (seed + non-Unknown hardware class =>
/// ReproducibleActionStream when no cloud swap is present).
fn local_pins(seed: u64) -> DeterminismConfig {
    DeterminismConfig {
        hardware_class: HardwareClass::Gpu(GpuSku::new("rtx3090")),
        seed,
        blake3_cache_prefix: [0u8; 8],
        system_fingerprint_pin: Some("local-fp-pin".to_string()),
    }
}

/// Write the four F-gate fixture rows + a local model_swap into the live SSB,
/// then return the full ordered stream. `include_f1` toggles the F-1 row so the
/// gate-off case can assert an ABSENT manifest entry.
async fn build_fixture_stream(
    ssb: &Arc<Ssb>,
    include_f1: bool,
    cloud_model: bool,
) -> Vec<elai_orchestrator::ssb::SsbEntry> {
    let mut superstep = 0i64;

    if include_f1 {
        // F-1: VibeGateBlocked source_blake3 (CONSUMED by the manifest, GD-1).
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
    }

    // F-4: TestIntegrityEvent stdout_hash.
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

    // F-9: WriteTimeSlopsquatBlocked dep_name_hash:u64 (normalized to hex).
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

    // F-10: SecurityClassRegressionBlocked delta_blake3.
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

    // model_swap: local (Qwen) by default; cloud (Sonnet) for the GD-5 case.
    let to_model = if cloud_model {
        "claude-sonnet-4-6"
    } else {
        "qwen2.5-coder-7b-instruct-abliterated"
    };
    ssb.write_payload(
        "executing",
        superstep,
        SsbPayload::ModelSwap(ModelSwapPayload {
            from_model: "bootstrap".into(),
            to_model: to_model.into(),
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
// (a) build -> write -> verify == Verified, manifest covers all 4 gates
// ---------------------------------------------------------------------------

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn build_write_verify_roundtrips_to_verified() {
    let tmp = TempDir::new().expect("tempdir");
    let ssb = make_ssb(&tmp, "ok");
    let stream = build_fixture_stream(&ssb, true, false).await;
    assert!(
        stream.len() >= 5,
        "fixture stream must hold the 4 F-gate rows + the model_swap; got {}",
        stream.len()
    );

    let pins = local_pins(42);
    let art = build_rra(FIXTURE_SESSION, &stream, &pins);

    // Manifest CONSUMES all 4 F-gate verdict hashes (GD-1, no re-implementation).
    let gates: Vec<&str> = art.manifest.iter().map(|g| g.gate.as_str()).collect();
    assert!(gates.contains(&"F-1"), "F-1 source_blake3 must be in manifest");
    assert!(gates.contains(&"F-4"), "F-4 stdout_hash must be in manifest");
    assert!(gates.contains(&"F-9"), "F-9 dep_name_hash must be in manifest");
    assert!(gates.contains(&"F-10"), "F-10 delta_blake3 must be in manifest");

    // F-9 was a u64; it must be normalized to a 16-hex string in the manifest.
    let f9 = art.manifest.iter().find(|g| g.gate == "F-9").expect("F-9 entry");
    assert_eq!(
        f9.verdict_blake3, "00f9000000000001",
        "F-9 u64 dep_name_hash must normalize to 16-hex big-endian"
    );

    assert!(!art.root_blake3.is_empty(), "root must be populated");
    assert_eq!(
        art.guarantee_level,
        GuaranteeLevel::ReproducibleActionStream,
        "a local pinned run with no cloud swap is action-stream reproducible"
    );

    let path = tmp.path().join("run.rra");
    write_rra(&art, &path).expect("write_rra");
    assert!(path.exists(), "the .rra file must exist on disk");

    let verdict = verify_rra(&path).expect("verify_rra");
    assert_eq!(
        verdict,
        ReplayVerdict::Verified,
        "a faithful .rra must verify; got {verdict:?}"
    );
}

// ---------------------------------------------------------------------------
// (b) tampered root -> Tampered
// ---------------------------------------------------------------------------

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn tampered_root_returns_tampered() {
    let tmp = TempDir::new().expect("tempdir");
    let ssb = make_ssb(&tmp, "tamper");
    let stream = build_fixture_stream(&ssb, true, false).await;
    let pins = local_pins(42);
    let mut art = build_rra(FIXTURE_SESSION, &stream, &pins);

    // Flip the recorded root: the canonical bytes no longer hash to it.
    art.root_blake3 = "deadbeefdeadbeef".to_string();

    let verdict = verify_artifact(&art).expect("verify_artifact");
    match verdict {
        ReplayVerdict::Tampered {
            expected_root,
            got_root,
        } => {
            assert_eq!(expected_root, "deadbeefdeadbeef");
            assert_ne!(got_root, expected_root, "recomputed root must differ");
        }
        other => panic!("expected Tampered, got {other:?}"),
    }
}

// ---------------------------------------------------------------------------
// (c) divergence -> Divergence naming the first mismatching event
// ---------------------------------------------------------------------------

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn flipped_event_payload_returns_divergence_at_first_mismatch() {
    let tmp = TempDir::new().expect("tempdir");
    let ssb = make_ssb(&tmp, "diverge");
    let stream = build_fixture_stream(&ssb, true, false).await;
    let pins = local_pins(42);
    let mut art = build_rra(FIXTURE_SESSION, &stream, &pins);

    // Flip a single payload byte on the SECOND event WITHOUT updating its
    // sealed event_blake3, then re-patch the bundle root so the tamper check
    // passes and the per-event divergence leg is the one that fires. This
    // models a faithful-looking bundle whose stream silently diverged.
    assert!(art.events.len() >= 2, "need >=2 events to flip the second");
    let target_seq = art.events[1].seq;
    if let Some(first) = art.events[1].payload.first_mut() {
        *first ^= 0xFF;
    } else {
        // Empty payload is impossible for these fixture rows, but guard anyway.
        art.events[1].payload.push(0xAB);
    }
    // Re-seal the root over the now-mutated canonical bytes so the tamper leg
    // (root check) PASSES and the divergence leg (per-event sealed hash) FIRES.
    art.root_blake3 =
        elai_orchestrator::rra::compute_root(&art).expect("recompute root after flip");

    let verdict = verify_artifact(&art).expect("verify_artifact");
    match verdict {
        ReplayVerdict::Divergence {
            first_mismatch_event,
            expected_blake3,
            got_blake3,
        } => {
            assert_eq!(
                first_mismatch_event, target_seq,
                "divergence must name the first (flipped) event's seq"
            );
            assert_ne!(
                expected_blake3, got_blake3,
                "the sealed hash must differ from the recomputed hash"
            );
        }
        other => panic!("expected Divergence, got {other:?}"),
    }
}

// ---------------------------------------------------------------------------
// (d) gate-OFF F-1 -> manifest entry absent, still Verified (valid gate-off)
// ---------------------------------------------------------------------------

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn gate_off_f1_absent_from_manifest_still_verifies() {
    let tmp = TempDir::new().expect("tempdir");
    let ssb = make_ssb(&tmp, "f1off");
    // include_f1 = false: the F-1 gate was off (default-OFF behind
    // ELAI_VIBE_GATE_ENABLE), so no VibeGateBlocked row exists.
    let stream = build_fixture_stream(&ssb, false, false).await;
    let pins = local_pins(42);
    let art = build_rra(FIXTURE_SESSION, &stream, &pins);

    let gates: Vec<&str> = art.manifest.iter().map(|g| g.gate.as_str()).collect();
    assert!(
        !gates.contains(&"F-1"),
        "a gate-off F-1 must have NO manifest entry (valid gate-off state)"
    );
    // The other 3 gates are still present (they were on in the fixture).
    assert!(gates.contains(&"F-4"));
    assert!(gates.contains(&"F-9"));
    assert!(gates.contains(&"F-10"));

    let path = tmp.path().join("run-f1off.rra");
    write_rra(&art, &path).expect("write_rra");
    let verdict = verify_rra(&path).expect("verify_rra");
    assert_eq!(
        verdict,
        ReplayVerdict::Verified,
        "a gate-off F-1 is NOT a faithfulness failure; the .rra must still verify"
    );
}

// ---------------------------------------------------------------------------
// (e) GD-2: root re-computes byte-identically under a fixed seed
// ---------------------------------------------------------------------------

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn root_recomputes_byte_identically_under_fixed_seed() {
    let tmp = TempDir::new().expect("tempdir");
    let ssb = make_ssb(&tmp, "determinism");
    let stream = build_fixture_stream(&ssb, true, false).await;

    let pins_a = local_pins(7);
    let pins_b = local_pins(7);
    let art_a = build_rra(FIXTURE_SESSION, &stream, &pins_a);
    let art_b = build_rra(FIXTURE_SESSION, &stream, &pins_b);

    assert_eq!(
        art_a.root_blake3, art_b.root_blake3,
        "GD-2: the same stream under the same seed must produce a byte-identical root"
    );

    // A different seed must change the root (the seed is part of the bundle).
    let pins_c = local_pins(8);
    let art_c = build_rra(FIXTURE_SESSION, &stream, &pins_c);
    assert_ne!(
        art_a.root_blake3, art_c.root_blake3,
        "a different pinned seed must produce a different root"
    );
}

// ---------------------------------------------------------------------------
// (f) GD-5 HONEST SCOPE: a cloud model swap forces VerifyDivergence
// ---------------------------------------------------------------------------

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn cloud_model_swap_forces_verify_divergence_never_reproducible_tokens() {
    let tmp = TempDir::new().expect("tempdir");
    let ssb = make_ssb(&tmp, "cloud");
    let stream = build_fixture_stream(&ssb, true, true).await;
    // Even with a fully-pinned local config, a cloud model swap in the stream
    // degrades the guarantee (the cloud system_fingerprint is opaque).
    let pins = local_pins(42);
    let art = build_rra(FIXTURE_SESSION, &stream, &pins);

    assert_eq!(
        art.guarantee_level,
        GuaranteeLevel::VerifyDivergence,
        "GD-5: a cloud-lane run is verify-divergence, never token-reproducible"
    );
    assert_ne!(
        art.guarantee_level,
        GuaranteeLevel::ReproducibleTokens,
        "build_rra must NEVER label any run ReproducibleTokens (honest scope)"
    );
    assert_eq!(
        art.model_pin.as_deref(),
        Some("claude-sonnet-4-6"),
        "model_pin must be the latest model_swap to_model"
    );
}
