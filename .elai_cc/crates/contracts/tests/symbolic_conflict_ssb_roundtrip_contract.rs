//! F-8 / Slice A1 SymbolicConflictDetected SSB round-trip contract tests.
//!
//! Validates the LLM-free SCF symbolic-conflict beacon (SCHEMA_VERSION 204):
//!   1. `SymbolicConflictPayload` survives the EXACT on-wire encoder
//!      `Ssb::write_payload` uses (`rmp_serde::to_vec_named`) -> `from_slice`,
//!      with all six String fields intact (fingerprint / kind / key / detail /
//!      candidate_id / fsm_substate).
//!   2. The wrapped payload reports `action_type() == SymbolicConflictDetected`
//!      and sql_str `"symbolic_conflict_detected"`.
//!   3. `ssb_action_type_from_str_public("symbolic_conflict_detected")` maps
//!      back to the flat-beacon variant and its sql_str round-trips stably.
//!   4. The full DB path: write through `Ssb::write_payload`, recover via
//!      `query_by_type`, decode the stored bytes the way the dashboard/CLI
//!      readers do, and assert every field survives.
//!
//! POLICY note: SymbolicConflictDetected is WARN-then-escalate (held pending),
//! never a hard fail-closed BLOCK. These tests pin the wire shape only; the
//! held-pending admission semantics are exercised by the CLI / detector slices.

use std::sync::Arc;

use elai_core::clock::OsClock;
use elai_orchestrator::ssb::{
    ssb_action_type_from_str_public, Ssb, SsbActionType, SsbConfig, SsbPayload, SsbReader,
    SymbolicConflictPayload,
};
use tempfile::TempDir;

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

/// A representative beacon payload. `key` is a real `self_mod_safety::
/// LOCKED_KEYS` entry; `fingerprint` is a blake3 hex-prefix of the conflict
/// tuple (blake3-only per HARD RULE). The strings are chosen to expose any
/// field-reordering / truncation bug on the wire (e.g. a kind<->key swap).
fn sample_symbolic_conflict_payload() -> SymbolicConflictPayload {
    let fingerprint = {
        let h = blake3::hash(
            b"contract_symbolic_conflict|locked_key_contradiction|cloud_monthly_budget|cfg-variant-contract",
        );
        format!("{:.16}", h.to_hex())
    };
    SymbolicConflictPayload {
        fingerprint,
        kind: "locked_key_contradiction".to_string(),
        key: "cloud_monthly_budget".to_string(),
        detail: "variant proposes raising cloud_monthly_budget; key is a self_mod_safety LOCKED invariant"
            .to_string(),
        candidate_id: "cfg-variant-contract".to_string(),
        fsm_substate: "AdmittingVariant".to_string(),
    }
}

// ---------------------------------------------------------------------------
// Contract 1: on-wire encoder round-trip (rmp_serde::to_vec_named)
// ---------------------------------------------------------------------------

#[test]
fn contract_symbolic_conflict_wire_encoder_roundtrip() {
    let original = sample_symbolic_conflict_payload();
    let wrapped = SsbPayload::SymbolicConflictDetected(original.clone());

    assert_eq!(
        wrapped.action_type(),
        SsbActionType::SymbolicConflictDetected,
        "SymbolicConflictDetected payload action_type must be SymbolicConflictDetected"
    );
    assert_eq!(
        wrapped.action_type().as_sql_str(),
        "symbolic_conflict_detected",
        "SymbolicConflictDetected sql_str must be symbolic_conflict_detected"
    );

    // The on-wire encoder: this is the exact call inside Ssb::write_payload.
    let bytes =
        rmp_serde::to_vec_named(&wrapped).expect("encode SymbolicConflictDetected on-wire");
    let decoded: SsbPayload =
        rmp_serde::from_slice(&bytes).expect("decode SymbolicConflictDetected from wire bytes");

    match decoded {
        SsbPayload::SymbolicConflictDetected(p) => {
            assert_eq!(p.fingerprint, original.fingerprint, "fingerprint must round-trip");
            assert_eq!(p.kind, original.kind, "kind must round-trip");
            assert_eq!(p.key, original.key, "key must round-trip");
            assert_eq!(p.detail, original.detail, "detail must round-trip");
            assert_eq!(
                p.candidate_id, original.candidate_id,
                "candidate_id must round-trip"
            );
            assert_eq!(
                p.fsm_substate, original.fsm_substate,
                "fsm_substate must round-trip"
            );
        }
        other => panic!("expected SymbolicConflictDetected, got {other:?}"),
    }
}

// ---------------------------------------------------------------------------
// Contract 2: ssb_action_type_from_str_public recognises the wire tag
// ---------------------------------------------------------------------------

#[test]
fn contract_symbolic_conflict_str_roundtrip() {
    let at = ssb_action_type_from_str_public("symbolic_conflict_detected")
        .expect("symbolic_conflict_detected must parse");
    assert_eq!(
        at,
        SsbActionType::SymbolicConflictDetected,
        "symbolic_conflict_detected must map to SsbActionType::SymbolicConflictDetected"
    );
    assert_eq!(
        at.as_sql_str(),
        "symbolic_conflict_detected",
        "as_sql_str round-trip must be stable"
    );
}

// ---------------------------------------------------------------------------
// Contract 3: full DB path via Ssb::write_payload + query_by_type
// ---------------------------------------------------------------------------

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn contract_symbolic_conflict_db_roundtrip() {
    let tmp = TempDir::new().expect("tempdir");
    let ssb = make_ssb(&tmp, "f8-test", "f8-scf-rt").await;

    let original = sample_symbolic_conflict_payload();
    ssb.write_payload(
        "self-mod-apply",
        0,
        SsbPayload::SymbolicConflictDetected(original.clone()),
        Vec::new(),
    )
    .await
    .expect("write SymbolicConflictDetected payload");

    let rows = ssb
        .query_by_type(SsbActionType::SymbolicConflictDetected)
        .expect("query SymbolicConflictDetected rows");
    assert_eq!(
        rows.len(),
        1,
        "exactly one SymbolicConflictDetected row in DB"
    );

    let decoded: SsbPayload = rmp_serde::from_slice(&rows[0].payload)
        .expect("decode SymbolicConflictDetected from DB");
    match decoded {
        SsbPayload::SymbolicConflictDetected(p) => {
            assert_eq!(p.fingerprint, original.fingerprint);
            assert_eq!(p.kind, original.kind);
            assert_eq!(p.key, original.key);
            assert_eq!(p.detail, original.detail);
            assert_eq!(p.candidate_id, original.candidate_id);
            assert_eq!(p.fsm_substate, original.fsm_substate);
        }
        other => panic!("expected SymbolicConflictDetected, got {other:?}"),
    }
}
