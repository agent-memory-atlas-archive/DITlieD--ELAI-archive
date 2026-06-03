//! F-5 / Slice A1 (SCHEMA_VERSION 203) -- wire-format round-trip for
//! [`StalenessQuarantineBroadcastPayload`].
//!
//! The SSB writer serializes payloads with `rmp_serde::to_vec_named` (named
//! msgpack maps, so field order on the wire is irrelevant and reads survive a
//! future field append). This test asserts that a fully-populated
//! StalenessQuarantineBroadcast payload survives a named-msgpack encode/decode
//! cycle with ALL SEVEN fields byte-equal: 1 String fact_id, 1 f64
//! staleness_ratio, 2 u32 (dependents_demoted + cosmos_artifacts_flagged), 1
//! u64 tick_ms, 1 String broadcast_id, 1 u32 dropped_count.
//!
//! This is the flat-beacon analogue of the per-payload round-trip coverage in
//! ssb.rs `all_test_payloads` (which exercises every variant through the live
//! Ssb writer). It is isolated here so the F-5 wire contract has a standalone,
//! Ssb-free regression that fails loudly if a field is renamed, retyped, or
//! dropped without a SCHEMA_VERSION bump.

use elai_orchestrator::ssb::StalenessQuarantineBroadcastPayload;

#[test]
fn staleness_quarantine_broadcast_named_msgpack_roundtrip() {
    // broadcast_id is the blake3 hex-prefix of fact_id + tick_ms, matching the
    // production correlation-id derivation (Anti-Stale-Training: blake3 only).
    let fact_id = "fact-roundtrip-0".to_owned();
    let tick_ms: u64 = 1_700_000_321_000;
    let mut seed = fact_id.clone().into_bytes();
    seed.extend_from_slice(&tick_ms.to_le_bytes());
    let broadcast_id = format!("{:.16}", blake3::hash(&seed).to_hex());

    let original = StalenessQuarantineBroadcastPayload {
        fact_id: fact_id.clone(),
        staleness_ratio: 0.9375,
        dependents_demoted: 4,
        cosmos_artifacts_flagged: 2,
        tick_ms,
        broadcast_id: broadcast_id.clone(),
        dropped_count: 7,
    };

    // Encode with the SAME serializer the SSB writer uses (named maps).
    let bytes = rmp_serde::to_vec_named(&original)
        .expect("named-msgpack encode of StalenessQuarantineBroadcastPayload");

    let decoded: StalenessQuarantineBroadcastPayload =
        rmp_serde::from_slice(&bytes).expect("named-msgpack decode round-trip");

    // Whole-struct equality (PartialEq) covers every field, but assert each of
    // the seven fields individually so a regression names the offending field.
    assert_eq!(decoded.fact_id, fact_id, "fact_id (String) drifted");
    assert_eq!(
        decoded.staleness_ratio, 0.9375,
        "staleness_ratio (f64) drifted"
    );
    assert_eq!(
        decoded.dependents_demoted, 4,
        "dependents_demoted (u32) drifted"
    );
    assert_eq!(
        decoded.cosmos_artifacts_flagged, 2,
        "cosmos_artifacts_flagged (u32) drifted"
    );
    assert_eq!(decoded.tick_ms, tick_ms, "tick_ms (u64) drifted");
    assert_eq!(
        decoded.broadcast_id, broadcast_id,
        "broadcast_id (String) drifted"
    );
    assert_eq!(decoded.dropped_count, 7, "dropped_count (u32) drifted");

    assert_eq!(decoded, original, "full payload round-trip mismatch");
}
