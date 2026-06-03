//! F-11 contract: `DarwinianAdmit` SSB action_type round-trips.
//!
//! The action_type + payload + 3-arm Kind live in ALWAYS-COMPILED `ssb.rs`
//! (only the PRODUCER, the darwin-loop driver, is feature-gated). So this
//! contract is feature-INDEPENDENT: it must hold on a default
//! `cargo test -p elai_cc_contracts` run.
//!
//! Asserts:
//!   - `SsbActionType::DarwinianAdmit.as_sql_str() == "darwinian_admit"`
//!   - parse round-trips ("darwinian_admit" -> DarwinianAdmit)
//!   - `DarwinianAdmitPayload` round-trips through rmp (msgpack)
//!   - the 3-arm `DarwinianAdmitKind` discriminator round-trips
//!   - the SCHEMA_VERSION bump landed (>= 207)
//!   - a real `Ssb` accepts the row (CHECK constraint admits "darwinian_admit")

use elai_orchestrator::ssb::{
    ssb_action_type_from_str_public, DarwinianAdmitKind, DarwinianAdmitPayload, SsbActionType,
    SsbPayload, SCHEMA_VERSION_PUBLIC,
};
use elai_orchestrator::{Ssb, SsbConfig};

#[test]
fn darwinian_admit_sql_str_is_stable() {
    assert_eq!(
        SsbActionType::DarwinianAdmit.as_sql_str(),
        "darwinian_admit"
    );
}

#[test]
fn darwinian_admit_parses_back() {
    let parsed =
        ssb_action_type_from_str_public("darwinian_admit").expect("darwinian_admit must parse");
    assert_eq!(parsed, SsbActionType::DarwinianAdmit);
}

#[test]
fn schema_version_bumped_for_f11() {
    // F-11 bumps the live baseline (206) to 207; assert it is at least 207 so
    // a later bump does not regress this contract.
    assert!(
        SCHEMA_VERSION_PUBLIC >= 207,
        "SCHEMA_VERSION must be >= 207 after F-11, got {SCHEMA_VERSION_PUBLIC}"
    );
}

#[test]
fn payload_roundtrips_through_rmp_for_each_kind() {
    for kind in [
        DarwinianAdmitKind::Admitted,
        DarwinianAdmitKind::RejectedByGuard,
        DarwinianAdmitKind::LoopClosed,
    ] {
        let payload = DarwinianAdmitPayload {
            kind,
            variant_id: "variant-rt".to_string(),
            parent_id: Some("parent-rt".to_string()),
            alpha: 0.42,
            guard_rejected_by: if kind == DarwinianAdmitKind::RejectedByGuard {
                Some("trap".to_string())
            } else {
                None
            },
            candidate_blake3: "0123456789abcdef".to_string(),
            seed: 42,
        };
        let wrapped = SsbPayload::DarwinianAdmit(payload.clone());
        // action_type() resolves to the right discriminant.
        assert_eq!(wrapped.action_type(), SsbActionType::DarwinianAdmit);

        let bytes = rmp_serde::to_vec_named(&wrapped).expect("encode");
        let back: SsbPayload = rmp_serde::from_slice(&bytes).expect("decode");
        match back {
            SsbPayload::DarwinianAdmit(p) => {
                assert_eq!(p.kind, kind);
                assert_eq!(p.variant_id, "variant-rt");
                assert_eq!(p.alpha, 0.42);
                assert_eq!(p.candidate_blake3, "0123456789abcdef");
                assert_eq!(p.seed, 42);
            }
            other => panic!("expected DarwinianAdmit, got {other:?}"),
        }
    }
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn ssb_check_constraint_accepts_darwinian_admit() {
    let dir = tempfile::TempDir::new().expect("tempdir");
    let cfg = SsbConfig {
        db_path: dir.path().join("ssb.db"),
        pool_size: None,
        project_id: "contract".to_string(),
        session_id: "darwinian-admit-roundtrip".to_string(),
    };
    let clock: std::sync::Arc<dyn elai_core::clock::ClockProvider> =
        std::sync::Arc::new(elai_core::OsClock);
    let ssb = Ssb::new(cfg, clock).expect("ssb init");

    let payload = SsbPayload::DarwinianAdmit(DarwinianAdmitPayload {
        kind: DarwinianAdmitKind::LoopClosed,
        variant_id: "variant-check".to_string(),
        parent_id: None,
        alpha: 0.61,
        guard_rejected_by: None,
        candidate_blake3: "0123456789abcdef".to_string(),
        seed: 7,
    });
    let seq = ssb
        .write_payload("contract:f11", 0, payload, Vec::new())
        .await
        .expect("the CHECK constraint must accept 'darwinian_admit'");
    assert!(seq >= 0);
}
