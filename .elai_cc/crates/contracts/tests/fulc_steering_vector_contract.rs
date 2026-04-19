//! batch-ful-c — A-8 Freshness Steering Vector end-to-end contract.
//!
//! Pins the public surface added by batch-ful-c:
//!   1. `elai_providers::layer_config::steering::ContrastPairs` struct.
//!   2. `SteeringVectorEngine::compute_vector_via_sidecar_role` async helper —
//!      additive sibling of the pre-existing `compute_vector_via_sidecar`;
//!      takes `ContrastPairs` + role label + cache path, writes the raw
//!      `Vec<f32>` little-endian bytes to disk on success.
//!   3. Steering cache round-trip: bytes written by the v2 helper deserialize
//!      back into the same `Vec<f32>` on a subsequent scan.
//!   4. `SteeringComputed` + `SteeringComputeFailed` SSB variants serialize
//!      correctly (action-type string stable, round-trip through SsbPayload).
//!   5. ActivityEntry coverage: `fulc_steering_computed` + `fulc_steering_compute_failed`
//!      are treated like any other SSB row — covered by the activity-entry
//!      coverage contract's `ALL_VARIANTS` floor bump (65 → 67).
//!   6. `SidecarProviderSignalReader::scan_steering_cache_dir` helper — reads
//!      a directory on disk and returns an `Option<SteeringVectorStatus>`
//!      reflecting whether any role's vector is cached.
//!   7. Connection-refused graceful failure: a v2 helper call against an
//!      unreachable URL returns an `Err` (no panic).
//!
//! Grounding:
//! * crates/providers/src/layer_config/steering.rs — pre-existing surface.
//! * crates/orchestrator/src/ssb.rs — SsbActionType + SsbPayload enum.
//! * crates/providers/src/sidecar_client.rs — `steering_cache_path` layout.
//! * crates/orchestrator/src/provider_signals.rs — SteeringVectorStatus.

use std::fs;
use std::io::Write;

use elai_orchestrator::ssb::{
    SsbActionType, SsbPayload, SteeringComputeFailedPayload, SteeringComputedPayload,
};
use elai_providers::layer_config::steering::{
    ContrastPairs, SteeringVector, SteeringVectorEngine,
};
use elai_providers::steering_cache_path;
use tempfile::TempDir;

// ---------------------------------------------------------------------------
// 1. ContrastPairs — public API surface
// ---------------------------------------------------------------------------

#[test]
fn contrast_pairs_new_and_field_access() {
    let pairs = ContrastPairs {
        positive: vec!["use blake3".to_string(), "use orjson".to_string()],
        negative: vec!["use blake2".to_string(), "use json".to_string()],
    };
    assert_eq!(pairs.positive.len(), 2);
    assert_eq!(pairs.negative.len(), 2);
    assert_eq!(pairs.positive[0], "use blake3");
    assert_eq!(pairs.negative[1], "use json");
}

#[test]
fn contrast_pairs_empty_is_legal() {
    // Empty pairs are legal as a construct; only compute_vector_via_sidecar_role
    // rejects them (tested via connection-refused path; the real sidecar
    // rejects them with HTTP 400 — covered by server.py side).
    let pairs = ContrastPairs {
        positive: vec![],
        negative: vec![],
    };
    assert!(pairs.positive.is_empty());
    assert!(pairs.negative.is_empty());
}

// ---------------------------------------------------------------------------
// 2. compute_vector_via_sidecar_role signature + connection-refused path
// ---------------------------------------------------------------------------

/// When the sidecar is unreachable, the v2 helper returns an Err without
/// panicking and does NOT write a partial cache file. This is the contract
/// the dashboard's `recompute_steering_vector` command relies on to surface
/// errors to the Svelte UI instead of crashing the Tauri handler.
#[tokio::test(flavor = "current_thread")]
async fn compute_vector_via_sidecar_role_returns_err_on_connection_refused() {
    let tmp = TempDir::new().expect("tempdir");
    let project_root = tmp.path().to_path_buf();
    let cache_path = steering_cache_path(&project_root, "proj-fulc-test", "executor");
    let pairs = ContrastPairs {
        positive: vec!["use blake3".to_string()],
        negative: vec!["use blake2".to_string()],
    };

    // Pick an unused local port to guarantee connection refused. 1 is reserved
    // (TCP port 1 / tcpmux) but unbound on every test host we target.
    let result = SteeringVectorEngine::compute_vector_via_sidecar_role(
        "http://127.0.0.1:1",
        "executor",
        &pairs,
        32,
        &cache_path,
    )
    .await;

    assert!(result.is_err(), "expected connection-refused to propagate as Err");
    assert!(
        !cache_path.exists(),
        "no partial cache file should be written on failure — got {}",
        cache_path.display()
    );
}

// ---------------------------------------------------------------------------
// 3. Cache path round-trip — raw LE f32 bytes
// ---------------------------------------------------------------------------

/// Contract: the cache bytes written by the helper decode losslessly back into
/// the originating `Vec<f32>`. The format is documented as raw little-endian
/// f32 (no header) so every tool that reads the cache — Rust side + Python
/// sidecar preload + dashboard display — agrees on byte layout.
#[test]
fn cache_path_round_trip_preserves_vector_bytes() {
    let tmp = TempDir::new().expect("tempdir");
    let project_root = tmp.path().to_path_buf();
    let cache_path = steering_cache_path(&project_root, "proj-fulc-rt", "auditor");

    let original: Vec<f32> = vec![0.1, -0.5, 1.25, 3.14159];
    fs::create_dir_all(cache_path.parent().expect("cache parent")).expect("mkdir cache parent");
    let mut bytes: Vec<u8> = Vec::with_capacity(original.len() * 4);
    for &x in &original {
        bytes.extend_from_slice(&x.to_le_bytes());
    }
    {
        let mut f = fs::File::create(&cache_path).expect("create cache file");
        f.write_all(&bytes).expect("write cache bytes");
    }

    // Read back + deserialize as the helper scan-side does.
    let round_trip = SteeringVectorEngine::load_cached_vector_bytes(&cache_path)
        .expect("load_cached_vector_bytes");

    assert_eq!(round_trip.len(), original.len());
    for (a, b) in round_trip.iter().zip(original.iter()) {
        assert!((a - b).abs() < f32::EPSILON, "round-trip lossy: {a} != {b}");
    }
}

#[test]
fn load_cached_vector_bytes_rejects_misaligned_length() {
    let tmp = TempDir::new().expect("tempdir");
    let cache_path = tmp.path().join("bad.bin");
    // 7 bytes: not a multiple of 4 → cannot decode as f32 stream.
    fs::write(&cache_path, [1u8, 2, 3, 4, 5, 6, 7]).expect("write bad bytes");

    let result = SteeringVectorEngine::load_cached_vector_bytes(&cache_path);
    assert!(result.is_err(), "expected error on length not %4 == 0");
}

// ---------------------------------------------------------------------------
// 4. SSB payload round-trip — SteeringComputed / SteeringComputeFailed
// ---------------------------------------------------------------------------

#[test]
fn steering_computed_payload_ssb_action_type_and_roundtrip() {
    let payload = SteeringComputedPayload {
        role: "executor".to_string(),
        vector_dim: 2048,
        norm: 12.34,
        elapsed_ms: 150,
        cache_path: "/tmp/.elai_cc/steering/proj-1/executor.bin".to_string(),
    };
    let wrapped = SsbPayload::SteeringComputed(payload.clone());
    assert_eq!(wrapped.action_type(), SsbActionType::SteeringComputed);
    assert_eq!(
        SsbActionType::SteeringComputed.as_sql_str(),
        "steering_computed"
    );

    let bytes = rmp_serde::to_vec_named(&wrapped).expect("serialize");
    let decoded: SsbPayload = rmp_serde::from_slice(&bytes).expect("deserialize");
    match decoded {
        SsbPayload::SteeringComputed(p) => {
            assert_eq!(p.role, "executor");
            assert_eq!(p.vector_dim, 2048);
            assert_eq!(p.elapsed_ms, 150);
            assert!((p.norm - 12.34).abs() < 1e-6);
        }
        other => panic!("expected SteeringComputed, got {other:?}"),
    }
}

#[test]
fn steering_compute_failed_payload_ssb_action_type_and_roundtrip() {
    let payload = SteeringComputeFailedPayload {
        role: "auditor".to_string(),
        reason: "sidecar_unreachable".to_string(),
    };
    let wrapped = SsbPayload::SteeringComputeFailed(payload);
    assert_eq!(
        wrapped.action_type(),
        SsbActionType::SteeringComputeFailed
    );
    assert_eq!(
        SsbActionType::SteeringComputeFailed.as_sql_str(),
        "steering_compute_failed"
    );

    let bytes = rmp_serde::to_vec_named(&wrapped).expect("serialize");
    let decoded: SsbPayload = rmp_serde::from_slice(&bytes).expect("deserialize");
    matches!(decoded, SsbPayload::SteeringComputeFailed(_));
}

// ---------------------------------------------------------------------------
// 5. Signal-reader cache scan helper
// ---------------------------------------------------------------------------

#[test]
fn scan_steering_cache_dir_returns_available_when_file_present() {
    let tmp = TempDir::new().expect("tempdir");
    let project_root = tmp.path().to_path_buf();
    let cache_path = steering_cache_path(&project_root, "proj-fulc-scan", "executor");
    fs::create_dir_all(cache_path.parent().expect("cache parent")).expect("mkdir");
    // Non-empty file to prove scan treats 0-byte as miss (cache write safety).
    fs::write(&cache_path, [0u8, 0, 0x80, 0x3f]).expect("write one f32");

    let status = SteeringVectorEngine::scan_steering_cache_dir(
        &project_root,
        "proj-fulc-scan",
    )
    .expect("scan succeeds");

    assert!(status.active, "cache file present → active=true");
    assert!(
        status.reason.contains("executor"),
        "reason should name the cached role(s): got {:?}",
        status.reason
    );
}

#[test]
fn scan_steering_cache_dir_returns_inactive_when_empty() {
    let tmp = TempDir::new().expect("tempdir");
    let status = SteeringVectorEngine::scan_steering_cache_dir(
        tmp.path(),
        "proj-fulc-empty",
    )
    .expect("scan succeeds even with no files");

    assert!(!status.active);
    assert!(
        status.reason.contains("modern_api_preference") || status.reason.contains("empty"),
        "reason should indicate cache miss: got {:?}",
        status.reason
    );
}

#[test]
fn scan_steering_cache_dir_ignores_zero_byte_files() {
    let tmp = TempDir::new().expect("tempdir");
    let project_root = tmp.path().to_path_buf();
    let cache_path = steering_cache_path(&project_root, "proj-fulc-zero", "executor");
    fs::create_dir_all(cache_path.parent().expect("cache parent")).expect("mkdir");
    fs::write(&cache_path, []).expect("create empty file");

    let status = SteeringVectorEngine::scan_steering_cache_dir(
        &project_root,
        "proj-fulc-zero",
    )
    .expect("scan succeeds");

    assert!(!status.active, "0-byte file must not count as active");
}

// ---------------------------------------------------------------------------
// 6. SteeringVector shape unchanged (regression — pre-existing contract)
// ---------------------------------------------------------------------------

#[test]
fn steering_vector_public_fields_unchanged_by_fulc() {
    // Sanity: the pre-existing struct still accepts the pre-existing fields.
    // batch-ful-c is additive; if we accidentally rename/drop a field, every
    // call site breaks. This test fails at compile time on drift.
    let _sv = SteeringVector {
        name: "fulc-regression".to_string(),
        layer: 10,
        coefficient: 5.0,
        vector: vec![1.0, 0.0, -1.0],
        positive_prompt: "use blake3".to_string(),
        negative_prompt: "use blake2".to_string(),
    };
}
