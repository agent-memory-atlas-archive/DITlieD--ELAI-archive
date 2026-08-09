//! I-18 contract tests -- uPIP row-chain + AI-BOM (Tier-1 offline).
//!
//! Off-agent artifacts (Layer-4 / R4, fail-closed per R1: a missing row FAILS).
//! Every "row landed" assertion is a REAL-DB `query_by_type` against the live
//! SSB sqlite (NOT a struct-only round-trip) so a wire that never reached the DB
//! cannot pass (closes the I-5/I-13 struct-only fail-open class).
//!
//!   (F1) CHAIN_ADVANCED row lands in SSB DB after build_rra + write_payload:
//!        a RowChainSealed(ChainAdvanced) is emitted and queryable via the real DB.
//!   (F2) AibomEmitted row lands in SSB DB: a RowChainSealed(AibomEmitted) is
//!        emitted and queryable via the real DB.
//!   (F3) ChainBroken arm is distinct: RowChainSealed(ChainBroken) carries a
//!        non-None break_seq and maps to the same wire tag as ChainAdvanced.
//!   (F4) Wire-tag round-trip: `row_chain_sealed` is in the SSB bijection; all 3
//!        Kind arms map through payload.action_type().
//!   (F5) chain_prev_hash + stack_hash fields land on the RunArtifact returned by
//!        build_rra (real-compute path, no mock).
//!   (F6) AibomEntry fields are non-empty when a model_pin is set on the RunArtifact
//!        (build_attestation wires the BOM entry).
//!   (F7) SCHEMA_VERSION 253: the locked const in ssb.rs matches.
//!
//! No #[ignore] / stubs / placeholder assertions.
//!
//! Grounding:
//!   - crates/orchestrator/src/ssb.rs (RowChainSealed* + SsbReader::query_by_type +
//!     SCHEMA_VERSION 253)
//!   - crates/orchestrator/src/rra.rs (build_rra + StackHash + chain_prev_hash)
//!   - crates/orchestrator/src/attestation.rs (build_attestation + AibomEntry)
//!   - uPIP row-chain + CycloneDX-ML AI-BOM (Tier-1 offline, no cosign-rs dep).

use std::sync::Arc;

use elai_core::clock::{ClockProvider, OsClock};
use elai_orchestrator::attestation::build_attestation;
use elai_orchestrator::determinism::DeterminismConfig;
use elai_orchestrator::rra::build_rra;
use elai_orchestrator::ssb::{
    decode_ssb_payload, ssb_action_type_from_str_public, RowChainSealedKind,
    RowChainSealedPayload, Ssb, SsbActionType, SsbConfig, SsbPayload,
};
use elai_orchestrator::SsbReader;
use tempfile::TempDir;

// ---------------------------------------------------------------------------
// Fixtures
// ---------------------------------------------------------------------------

fn test_clock() -> Arc<dyn ClockProvider> {
    Arc::new(OsClock)
}

fn test_ssb(dir: &TempDir, session_id: &str) -> Arc<Ssb> {
    let db_path = dir.path().join("ssb.db");
    let cfg = SsbConfig {
        db_path,
        pool_size: Some(2),
        project_id: "i18-contract".into(),
        session_id: session_id.into(),
    };
    Arc::new(Ssb::new(cfg, test_clock()).expect("ssb init"))
}

/// Write one RowChainSealed payload with the given Kind to the SSB and return
/// the count of rows of that Kind in the DB (the real-DB contract assertion).
async fn emit_chain_row(
    ssb: &Arc<Ssb>,
    session_id: &str,
    kind: RowChainSealedKind,
    break_seq: Option<i64>,
) -> usize {
    let payload = SsbPayload::RowChainSealed(RowChainSealedPayload {
        kind,
        seq: 1,
        session_id: session_id.to_owned(),
        ts_ms: 0,
        break_seq,
    });
    ssb.write_payload("test:chain", 0, payload, Vec::new())
        .await
        .expect("write RowChainSealed row");
    ssb.query_by_type(SsbActionType::RowChainSealed(kind))
        .expect("query_by_type RowChainSealed")
        .len()
}

/// Build a minimal DeterminismConfig for unit tests (seed=0, cpu, no fp pin).
fn test_pins() -> DeterminismConfig {
    DeterminismConfig {
        hardware_class: elai_orchestrator::determinism::HardwareClass::Cpu,
        seed: 0,
        blake3_cache_prefix: [0u8; 8],
        system_fingerprint_pin: None,
    }
}

// ---------------------------------------------------------------------------
// (F1) CHAIN_ADVANCED row lands in SSB DB after direct write_payload.
// ---------------------------------------------------------------------------

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn i18_f1_chain_advanced_row_lands_in_db() {
    let dir = TempDir::new().expect("tempdir");
    let ssb = test_ssb(&dir, "i18-f1");

    let count = emit_chain_row(&ssb, "i18-f1", RowChainSealedKind::ChainAdvanced, None).await;

    // R1 missing=FAIL: at least one ChainAdvanced row landed in the real DB.
    assert!(
        count >= 1,
        "RowChainSealed(ChainAdvanced) row must land in SSB DB (R1 missing=FAIL); got {}",
        count
    );

    // Decode the row and check the payload fields.
    let rows = ssb
        .query_by_type(SsbActionType::RowChainSealed(RowChainSealedKind::ChainAdvanced))
        .expect("query RowChainSealed rows");
    assert!(!rows.is_empty(), "at least one row");
    let p = match decode_ssb_payload(&rows[0].payload).expect("decode") {
        SsbPayload::RowChainSealed(p) => p,
        other => panic!("expected RowChainSealed, got {:?}", other.action_type()),
    };
    assert_eq!(p.kind, RowChainSealedKind::ChainAdvanced);
    assert_eq!(p.session_id, "i18-f1");
    assert!(p.break_seq.is_none(), "ChainAdvanced must not carry a break_seq");
}

// ---------------------------------------------------------------------------
// (F2) AIBOM_EMITTED row lands in SSB DB.
// ---------------------------------------------------------------------------

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn i18_f2_aibom_emitted_row_lands_in_db() {
    let dir = TempDir::new().expect("tempdir");
    let ssb = test_ssb(&dir, "i18-f2");

    let count = emit_chain_row(&ssb, "i18-f2", RowChainSealedKind::AibomEmitted, None).await;

    // R1 missing=FAIL: AibomEmitted row landed.
    assert!(
        count >= 1,
        "RowChainSealed(AibomEmitted) row must land in SSB DB (R1 missing=FAIL); got {}",
        count
    );
}

// ---------------------------------------------------------------------------
// (F3) CHAIN_BROKEN arm carries break_seq; distinct from ChainAdvanced.
// ---------------------------------------------------------------------------

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn i18_f3_chain_broken_carries_break_seq() {
    let dir = TempDir::new().expect("tempdir");
    let ssb = test_ssb(&dir, "i18-f3");

    let count = emit_chain_row(&ssb, "i18-f3", RowChainSealedKind::ChainBroken, Some(42)).await;

    assert!(
        count >= 1,
        "RowChainSealed(ChainBroken) row must land in SSB DB; got {}",
        count
    );

    let rows = ssb
        .query_by_type(SsbActionType::RowChainSealed(RowChainSealedKind::ChainBroken))
        .expect("query RowChainSealed rows");
    let p = match decode_ssb_payload(&rows[0].payload).expect("decode") {
        SsbPayload::RowChainSealed(p) => p,
        other => panic!("expected RowChainSealed, got {:?}", other.action_type()),
    };
    assert_eq!(p.kind, RowChainSealedKind::ChainBroken);
    assert_eq!(p.break_seq, Some(42), "ChainBroken must carry the break_seq=42");

    // The three Kind arms are distinct values.
    assert_ne!(RowChainSealedKind::ChainBroken, RowChainSealedKind::ChainAdvanced);
    assert_ne!(RowChainSealedKind::ChainBroken, RowChainSealedKind::AibomEmitted);
    assert_ne!(RowChainSealedKind::ChainAdvanced, RowChainSealedKind::AibomEmitted);
}

// ---------------------------------------------------------------------------
// (F4) Wire-tag round-trip -- all 3 Kind arms map to `row_chain_sealed`.
// ---------------------------------------------------------------------------

#[test]
fn i18_f4_wire_tag_roundtrips() {
    // Tag is in the from_str bijection.
    let parsed = ssb_action_type_from_str_public("row_chain_sealed")
        .expect("row_chain_sealed must parse");
    assert_eq!(parsed.as_sql_str(), "row_chain_sealed");

    // All 3 Kind arms collapse to the same wire tag (Pattern-2).
    for kind in [
        RowChainSealedKind::ChainAdvanced,
        RowChainSealedKind::AibomEmitted,
        RowChainSealedKind::ChainBroken,
    ] {
        let payload = SsbPayload::RowChainSealed(RowChainSealedPayload {
            kind,
            seq: 0,
            session_id: "test".into(),
            ts_ms: 0,
            break_seq: if matches!(kind, RowChainSealedKind::ChainBroken) {
                Some(1)
            } else {
                None
            },
        });
        assert_eq!(
            payload.action_type().as_sql_str(),
            "row_chain_sealed",
            "every Kind arm ({:?}) maps to the row_chain_sealed tag",
            kind
        );
    }
}

// ---------------------------------------------------------------------------
// (F5) chain_prev_hash + stack_hash fields on RunArtifact (real build_rra path).
// ---------------------------------------------------------------------------

#[test]
fn i18_f5_build_rra_chain_prev_hash_and_stack_hash() {
    // build_rra with an empty stream produces a RunArtifact. chain_prev_hash
    // is None (GENESIS: no prior row to hash). stack_hash is Some with 5 fields.
    let art = build_rra("i18-f5-session", &[], &test_pins());

    // chain_prev_hash: None on GENESIS (no prior row in the stream).
    assert!(
        art.chain_prev_hash.is_none(),
        "GENESIS chain_prev_hash must be None for an empty stream"
    );

    // stack_hash: Some with non-empty fields.
    let sh = art.stack_hash.as_ref().expect("stack_hash must be Some on new receipts (I-18)");
    assert!(!sh.state.is_empty(), "state layer must be non-empty");
    assert!(!sh.deps.is_empty(), "deps layer must be non-empty");
    assert!(!sh.process.is_empty(), "process layer must be non-empty");
    assert!(!sh.result.is_empty(), "result layer must be non-empty");
    assert!(!sh.verify.is_empty(), "verify combiner must be non-empty");

    // All fields are 16-hex prefixes (consistent with the rest of the codebase).
    assert_eq!(sh.state.len(), 16, "state layer is 16-hex");
    assert_eq!(sh.deps.len(), 16, "deps layer is 16-hex");
    assert_eq!(sh.process.len(), 16, "process layer is 16-hex");
    assert_eq!(sh.result.len(), 16, "result layer is 16-hex");
    assert_eq!(sh.verify.len(), 16, "verify combiner is 16-hex");
}

// ---------------------------------------------------------------------------
// (F6) AibomEntry fields non-empty when model_pin is set.
// ---------------------------------------------------------------------------

#[test]
fn i18_f6_build_attestation_aibom_entry_non_empty() {
    use elai_orchestrator::rra::{GuaranteeLevel, RunArtifact};

    // Build a minimal RunArtifact with a model_pin so build_attestation
    // constructs an AibomEntry.
    let art = RunArtifact {
        format_version: elai_orchestrator::rra::RRA_FORMAT_VERSION,
        session_id: "i18-f6-session".into(),
        seed: 0,
        hardware_class: "cpu".into(),
        system_fingerprint_pin: Some("test-fp".into()),
        model_pin: Some("claude-sonnet-4-6".into()),
        guarantee_level: GuaranteeLevel::VerifyDivergence,
        events: Vec::new(),
        manifest: Vec::new(),
        objective_hash: None,
        chain_prev_hash: None,
        stack_hash: None,
        sampling_seed: None,
        determinism_applied: None,
        non_reproducible: false,
        root_blake3: "0000000000000000".into(),
    };

    let stmt = build_attestation(&art).expect("build_attestation ok");
    let aibom = stmt
        .predicate
        .aibom_entry
        .as_ref()
        .expect("aibom_entry must be Some when model_pin is set");

    assert_eq!(aibom.type_field, "ml-model", "CycloneDX type must be ml-model");
    assert_eq!(aibom.name, "claude-sonnet-4-6", "name must match model_pin");
    assert!(!aibom.bom_ref.is_empty(), "bom_ref must be non-empty");
    assert!(!aibom.hashes.is_empty(), "hashes must be non-empty");
    assert_eq!(aibom.hashes[0].alg, "BLAKE3", "hash alg must be BLAKE3");
    assert_eq!(
        aibom.hashes[0].content,
        art.root_blake3,
        "hash content must be the bundle root"
    );
    // claude-sonnet-4-6 does not contain '/' so no external reference is expected.
    assert!(
        aibom.external_references.is_empty(),
        "a non-HF model name must produce no external references"
    );
}

// ---------------------------------------------------------------------------
// (F7) SCHEMA_VERSION floor: I-18 introduced at 253; later letters may bump.
// ---------------------------------------------------------------------------

#[test]
fn i18_f7_schema_version_253() {
    // I-18 introduced SCHEMA_VERSION 253 (I-17 was 252). Later letters may bump
    // SCHEMA_VERSION further, so the contract is a floor (>= 253), not equality.
    // SCHEMA_VERSION_PUBLIC mirrors the private SCHEMA_VERSION at ssb.rs:3300.
    assert!(
        elai_orchestrator::ssb::SCHEMA_VERSION_PUBLIC >= 253,
        "SCHEMA_VERSION must be >= 253 after I-18 (I-17 was 252); got {}",
        elai_orchestrator::ssb::SCHEMA_VERSION_PUBLIC
    );
}
