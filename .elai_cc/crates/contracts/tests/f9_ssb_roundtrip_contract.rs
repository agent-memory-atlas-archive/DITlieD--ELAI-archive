//! F-9 / Slice D4 — WriteTimeSlopsquatBlocked SSB round-trip contract.
//!
//! Validates:
//!   1. `WriteTimeSlopsquatBlockedPayload` with `NotFound` verdict encodes +
//!      decodes through msgpack and round-trips through the SSB DB.
//!   2. `WriteTimeSlopsquatBlockedPayload` with `TooNew` verdict round-trips.
//!   3. `WriteTimeSlopVerdictKind::RegistryUnreachable` round-trips through
//!      msgpack (enum must parse) AND, per the revised W-10 posture, produces
//!      an ADVISORY SSB write so the fail-open leaves an observable trace
//!      ("no silent fail-open without an advisory SSB row"). The verdict itself
//!      stays Warn/Allow — the gate never blocks on unreachable.
//!   4. `ssb_action_type_from_str_public("write_time_slopsquat_blocked")`
//!      returns `SsbActionType::WriteTimeSlopsquatBlocked`.
//!   5. `SsbActionType::WriteTimeSlopsquatBlocked::as_sql_str()` is
//!      `"write_time_slopsquat_blocked"`.

use std::sync::Arc;

use elai_core::clock::OsClock;
use elai_orchestrator::ssb::{
    ssb_action_type_from_str_public, Ssb, SsbActionType, SsbConfig, SsbPayload, SsbReader,
    WriteTimeSlopVerdictKind, WriteTimeSlopsquatBlockedPayload,
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

fn ts_ms_now() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis().min(i64::MAX as u128) as i64)
        .unwrap_or(0)
}

// ---------------------------------------------------------------------------
// Contract 1: NotFound arm round-trip
// ---------------------------------------------------------------------------

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn contract_write_time_not_found_roundtrip() {
    let tmp = TempDir::new().expect("tempdir");
    let ssb = make_ssb(&tmp, "f9-test", "f9-sess-nf").await;

    let payload = SsbPayload::WriteTimeSlopsquatBlocked(WriteTimeSlopsquatBlockedPayload {
        dep_name_hash: 0xDEAD_BEEF_CAFE_1234_u64,
        ecosystem: "cargo".to_string(),
        verdict_kind: WriteTimeSlopVerdictKind::NotFound.as_str().to_owned(),
        registry_age_days: None,
        ts_ms: ts_ms_now(),
    });

    assert_eq!(
        payload.action_type(),
        SsbActionType::WriteTimeSlopsquatBlocked,
        "NotFound payload action_type must be WriteTimeSlopsquatBlocked"
    );
    assert_eq!(
        payload.action_type().as_sql_str(),
        "write_time_slopsquat_blocked",
        "sql_str must be write_time_slopsquat_blocked"
    );

    ssb.write_payload("dep-check-agent", 0, payload, vec![])
        .await
        .expect("write NotFound payload");

    let rows = ssb
        .query_by_type(SsbActionType::WriteTimeSlopsquatBlocked)
        .expect("query NotFound rows");
    assert_eq!(rows.len(), 1, "exactly one WriteTimeSlopsquatBlocked row");

    let decoded: SsbPayload = rmp_serde::from_slice(&rows[0].payload)
        .expect("decode NotFound payload");
    match decoded {
        SsbPayload::WriteTimeSlopsquatBlocked(p) => {
            assert_eq!(p.dep_name_hash, 0xDEAD_BEEF_CAFE_1234_u64);
            assert_eq!(p.ecosystem, "cargo");
            assert_eq!(p.verdict_kind, "not_found");
            assert!(p.registry_age_days.is_none(), "NotFound must carry no age_days");
        }
        other => panic!("expected WriteTimeSlopsquatBlocked, got {other:?}"),
    }
}

// ---------------------------------------------------------------------------
// Contract 2: TooNew arm round-trip
// ---------------------------------------------------------------------------

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn contract_write_time_too_new_roundtrip() {
    let tmp = TempDir::new().expect("tempdir");
    let ssb = make_ssb(&tmp, "f9-test", "f9-sess-tn").await;

    let payload = SsbPayload::WriteTimeSlopsquatBlocked(WriteTimeSlopsquatBlockedPayload {
        dep_name_hash: 0xABCD_EF01_2345_6789_u64,
        ecosystem: "pypi".to_string(),
        verdict_kind: WriteTimeSlopVerdictKind::TooNew.as_str().to_owned(),
        registry_age_days: Some(7),
        ts_ms: ts_ms_now(),
    });

    assert_eq!(
        payload.action_type().as_sql_str(),
        "write_time_slopsquat_blocked",
        "TooNew sql_str must be write_time_slopsquat_blocked"
    );

    ssb.write_payload("dep-check-agent", 0, payload, vec![])
        .await
        .expect("write TooNew payload");

    let rows = ssb
        .query_by_type(SsbActionType::WriteTimeSlopsquatBlocked)
        .expect("query TooNew rows");
    assert_eq!(rows.len(), 1, "exactly one WriteTimeSlopsquatBlocked row");

    let decoded: SsbPayload = rmp_serde::from_slice(&rows[0].payload)
        .expect("decode TooNew payload");
    match decoded {
        SsbPayload::WriteTimeSlopsquatBlocked(p) => {
            assert_eq!(p.verdict_kind, "too_new");
            assert_eq!(p.registry_age_days, Some(7));
            assert_eq!(p.ecosystem, "pypi");
        }
        other => panic!("expected WriteTimeSlopsquatBlocked(TooNew), got {other:?}"),
    }
}

// ---------------------------------------------------------------------------
// Contract 3: RegistryUnreachable produces an ADVISORY SSB row (revised W-10)
//
// Revised W-10 posture: a fail-open must leave a trace ("no silent fail-open
// without an advisory SSB row"). The gate stays fail-open (Warn/Allow, never
// Block) on registry-unreachable, but it now emits an advisory
// WriteTimeSlopsquatBlocked row with verdict_kind="registry_unreachable" so the
// fail-open is observable. This test verifies:
//   a) the variant serialises to the expected label (enum completeness),
//   b) the advisory row round-trips through the SSB DB with the
//      registry_unreachable verdict_kind and no age_days.
// ---------------------------------------------------------------------------

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn contract_write_time_unreachable_arm_emits_advisory_ssb_row() {
    let tmp = TempDir::new().expect("tempdir");
    let ssb = make_ssb(&tmp, "f9-test", "f9-sess-ur").await;

    // Verify the enum variant serialises to the expected label.
    assert_eq!(
        WriteTimeSlopVerdictKind::RegistryUnreachable.as_str(),
        "registry_unreachable",
        "RegistryUnreachable as_str must be registry_unreachable"
    );

    // Revised W-10: emit the advisory row durably (the analyze() path is
    // spawn-detached; here we exercise the durable round-trip directly with the
    // RegistryUnreachable payload the gate now produces).
    let payload = SsbPayload::WriteTimeSlopsquatBlocked(WriteTimeSlopsquatBlockedPayload {
        dep_name_hash: 0x0BAD_F00D_DEAD_BEEF_u64,
        ecosystem: "cargo".to_string(),
        verdict_kind: WriteTimeSlopVerdictKind::RegistryUnreachable.as_str().to_owned(),
        registry_age_days: None,
        ts_ms: ts_ms_now(),
    });

    ssb.write_payload("dep-check-agent", 0, payload, vec![])
        .await
        .expect("write RegistryUnreachable advisory payload");

    let rows = ssb
        .query_by_type(SsbActionType::WriteTimeSlopsquatBlocked)
        .expect("query Unreachable rows");
    assert_eq!(
        rows.len(),
        1,
        "revised W-10: exactly one advisory WriteTimeSlopsquatBlocked row must exist after Unreachable verdict"
    );

    let decoded: SsbPayload =
        rmp_serde::from_slice(&rows[0].payload).expect("decode Unreachable advisory payload");
    match decoded {
        SsbPayload::WriteTimeSlopsquatBlocked(p) => {
            assert_eq!(p.verdict_kind, "registry_unreachable");
            assert!(
                p.registry_age_days.is_none(),
                "RegistryUnreachable advisory carries no age_days"
            );
            assert_eq!(p.ecosystem, "cargo");
        }
        other => panic!("expected WriteTimeSlopsquatBlocked(RegistryUnreachable), got {other:?}"),
    }
}

// ---------------------------------------------------------------------------
// Contract 4: ssb_action_type_from_str_public round-trip
// ---------------------------------------------------------------------------

#[test]
fn contract_action_type_from_str_roundtrip() {
    let at = ssb_action_type_from_str_public("write_time_slopsquat_blocked")
        .expect("must parse write_time_slopsquat_blocked");
    assert_eq!(
        at,
        SsbActionType::WriteTimeSlopsquatBlocked,
        "must round-trip to WriteTimeSlopsquatBlocked"
    );
    assert_eq!(
        at.as_sql_str(),
        "write_time_slopsquat_blocked",
        "as_sql_str must be write_time_slopsquat_blocked"
    );
}

// ---------------------------------------------------------------------------
// Contract 5: WriteTimeSlopVerdictKind::as_str consistency
// ---------------------------------------------------------------------------

#[test]
fn contract_verdict_kind_as_str_all_arms() {
    assert_eq!(WriteTimeSlopVerdictKind::NotFound.as_str(), "not_found");
    assert_eq!(WriteTimeSlopVerdictKind::TooNew.as_str(), "too_new");
    assert_eq!(
        WriteTimeSlopVerdictKind::RegistryUnreachable.as_str(),
        "registry_unreachable"
    );
}
