//! Phase B4 — Artifacts Pipeline library skeleton contract.
//!
//! Pins:
//!   (a) `ArtifactStore::publish` writes the artifact bytes to the canonical
//!       path `<root>/.elai_cc/artifacts/<task_id>/<step_id>/<artifact_id>.<ext>`
//!       computed by `elai_core::paths::artifact_path`.
//!   (b) The bytes on disk match the input exactly (no transformation).
//!   (c) A sibling manifest file `<artifact_id>.json` is written at the path
//!       returned by `elai_core::paths::artifact_manifest_path`.
//!   (d) Manifest `blake3_hash` field equals `blake3::hash(&bytes).to_hex()`
//!       (hex-lower, 64 chars). Field name is `blake3_hash` NOT `sha256_hash`
//!       per CLAUDE.md anti-stale rule: blake3 is the canonical hash crate.
//!   (e) Manifest `mime` matches the input mime string.
//!
//! Scope note: this phase intentionally does NOT assert an SSB
//! `ArtifactPublished` row — SSB wiring is deferred to a follow-up patch
//! because `orchestrator/src/ssb.rs` has uncommitted B-28 work in flight.
//! See `crates/orchestrator/src/artifacts.rs` doc-comment.

use std::fs;
use std::sync::Arc;
use std::time::{Duration, SystemTime};

use elai_core::clock::{ClockProvider, LogicalInstant, OsClock};
use elai_core::paths::{artifact_manifest_path, artifact_path};
use elai_orchestrator::{ArtifactManifest, ArtifactStore, ArtifactsError, ContextArtifact};
use tempfile::TempDir;

/// Build a 100-byte payload with a recognisable pattern so off-by-one writes
/// are obvious in failure output.
fn fixture_bytes() -> Vec<u8> {
    (0u8..100).collect()
}

#[tokio::test]
async fn publish_writes_artifact_and_manifest() {
    let tmp = TempDir::new().expect("tempdir");
    let root = tmp.path().to_path_buf();

    let bytes = fixture_bytes();
    let expected_hash = blake3::hash(&bytes).to_hex().to_string();

    let clock: Arc<dyn ClockProvider> = Arc::new(OsClock);
    let store = ArtifactStore::new(root.clone(), clock);
    let artifact = ContextArtifact {
        task_id: "t".to_string(),
        step_id: "s".to_string(),
        artifact_id: "a".to_string(),
        ext: "png".to_string(),
        bytes: bytes.clone(),
        mime: "image/png".to_string(),
        provenance: "contract-test".to_string(),
    };

    let (written_artifact, written_manifest) = store
        .publish(artifact)
        .await
        .expect("publish must succeed on a fresh tempdir");

    // (a) canonical artifact path.
    let expected_artifact = artifact_path(&root, "t", "s", "a", "png");
    assert_eq!(
        written_artifact, expected_artifact,
        "publish() must return the canonical artifact path",
    );
    assert!(
        written_artifact.exists(),
        "artifact file should exist at {:?}",
        written_artifact,
    );

    // (b) bytes on disk match input exactly.
    let on_disk = fs::read(&written_artifact).expect("read artifact bytes");
    assert_eq!(
        on_disk, bytes,
        "artifact bytes on disk must match input bytes",
    );

    // (c) canonical manifest path.
    let expected_manifest = artifact_manifest_path(&root, "t", "s", "a");
    assert_eq!(
        written_manifest, expected_manifest,
        "publish() must return the canonical manifest path",
    );
    assert!(
        written_manifest.exists(),
        "manifest file should exist at {:?}",
        written_manifest,
    );

    // (d) manifest blake3_hash field matches blake3 of the bytes.
    //     Field name MUST be `blake3_hash` (anti-stale rule).
    let manifest_raw = fs::read(&written_manifest).expect("read manifest bytes");
    let manifest: ArtifactManifest =
        serde_json::from_slice(&manifest_raw).expect("manifest must be valid JSON");

    assert_eq!(
        manifest.blake3_hash, expected_hash,
        "manifest blake3_hash must equal blake3::hash(&bytes).to_hex()",
    );
    assert_eq!(
        manifest.blake3_hash.len(),
        64,
        "blake3 hex should be 64 characters; got {}",
        manifest.blake3_hash.len(),
    );

    // (e) manifest mime + artifact_id + provenance preserved.
    assert_eq!(manifest.mime, "image/png");
    assert_eq!(manifest.artifact_id, "a");
    assert_eq!(manifest.provenance, "contract-test");
    assert!(
        manifest.timestamp_ms > 0,
        "manifest timestamp_ms must be a positive epoch-ms value, got {}",
        manifest.timestamp_ms,
    );

    // Raw JSON must contain the field `blake3_hash` (catches accidental
    // rename to `sha256_hash` / `hash`).
    let raw_str = std::str::from_utf8(&manifest_raw).expect("manifest is UTF-8");
    assert!(
        raw_str.contains("\"blake3_hash\""),
        "manifest JSON must spell the field `blake3_hash` (not sha256_hash); got: {raw_str}",
    );
    assert!(
        !raw_str.contains("\"sha256_hash\""),
        "manifest JSON must NOT contain `sha256_hash` (anti-stale rule); got: {raw_str}",
    );
}

/// Custom mock clock that returns a `SystemTime` whose `duration_since(UNIX_EPOCH).as_millis()`
/// exceeds `i64::MAX`. Used to verify the `ClockOverflow` variant is emitted
/// instead of the (incorrect) `HashFailed` reuse that the B-19 audit flagged.
/// `SeededClock` is not used here because its public surface only exposes a
/// `with_epoch(SystemTime)` seed plus a ms-granularity tick increment — both
/// representable within i64. A raw mock keeps the assertion surgical.
struct FarFutureClock;

impl ClockProvider for FarFutureClock {
    fn now(&self) -> LogicalInstant {
        LogicalInstant::from_tick(0)
    }
    fn elapsed_since(&self, _mark: LogicalInstant) -> Duration {
        Duration::ZERO
    }
    fn system_now(&self) -> SystemTime {
        // i64::MAX ms ~ 9.22e15 s past epoch. +1 s pushes us into overflow.
        let overflow_secs: u64 = (i64::MAX as u64) / 1000 + 1;
        SystemTime::UNIX_EPOCH + Duration::from_secs(overflow_secs)
    }
    // s-68264 audit cleanup: ClockProvider gained instant_now (C-15.1). Return std Instant::now()
    // here — only system_now overflow is under test, instant_now is unused by ArtifactStore.publish.
    fn instant_now(&self) -> std::time::Instant {
        std::time::Instant::now()
    }
}

#[tokio::test]
#[cfg_attr(
    windows,
    ignore = "P: Windows std::time panics on UNIX_EPOCH + Duration(~9.22e15s) at the Instant layer (library/std/src/time.rs:614). The contract under test is the u128->i64 narrowing inside ArtifactStore::publish; the mock clock cannot construct a SystemTime past i64::MAX ms on Windows. Linux/macOS exercise the path fully."
)]
async fn publish_returns_clock_overflow_when_epoch_exceeds_i64_max() {
    let tmp = TempDir::new().expect("tempdir");
    let root = tmp.path().to_path_buf();

    let clock: Arc<dyn ClockProvider> = Arc::new(FarFutureClock);
    let store = ArtifactStore::new(root, clock);
    let artifact = ContextArtifact {
        task_id: "t".to_string(),
        step_id: "s".to_string(),
        artifact_id: "a".to_string(),
        ext: "png".to_string(),
        bytes: vec![1, 2, 3],
        mime: "image/png".to_string(),
        provenance: "contract-test".to_string(),
    };

    let err = store
        .publish(artifact)
        .await
        .expect_err("FarFutureClock must trigger ClockOverflow");

    match err {
        ArtifactsError::ClockOverflow(ms) => {
            assert!(
                ms > i64::MAX as u128,
                "ClockOverflow payload {ms} must exceed i64::MAX",
            );
        }
        other => panic!(
            "expected ArtifactsError::ClockOverflow, got {other:?} \
             (B-19 audit BLOCK-2: HashFailed was the wrong category)",
        ),
    }
}

#[test]
fn artifact_paths_are_under_canonical_artifacts_dir() {
    // Deterministic check: artifact_path() and artifact_manifest_path()
    // must both live under `<root>/.elai_cc/artifacts/<task>/<step>/`.
    let root = std::path::PathBuf::from("/tmp/elai-fixture");
    let art = artifact_path(&root, "t1", "s2", "a3", "bin");
    let man = artifact_manifest_path(&root, "t1", "s2", "a3");

    let art_s = art.to_string_lossy().replace('\\', "/");
    let man_s = man.to_string_lossy().replace('\\', "/");

    assert!(
        art_s.ends_with(".elai_cc/artifacts/t1/s2/a3.bin"),
        "artifact_path layout regression: {art_s}",
    );
    assert!(
        man_s.ends_with(".elai_cc/artifacts/t1/s2/a3.json"),
        "artifact_manifest_path layout regression: {man_s}",
    );
}
