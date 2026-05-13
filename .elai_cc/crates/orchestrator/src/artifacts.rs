//! Phase B4 — Artifacts Pipeline (library skeleton).
//!
//! `ArtifactStore::publish` writes a single typed artifact + JSON manifest
//! under the canonical `<project_root>/.elai_cc/artifacts/<task>/<step>/`
//! layout (`elai_core::paths::artifact_path` + `artifact_manifest_path`).
//!
//! ## Scope
//!
//! This phase ships ONLY the library types and the filesystem-write helper.
//! Emission of an `SsbActionType::ArtifactPublished` row is INTENTIONALLY
//! DEFERRED to a follow-up patch because `orchestrator/src/ssb.rs` has
//! uncommitted B-28 work in flight and Phase B4 was scope-narrowed by the
//! parent to avoid two concurrent edits to that file. The contract test
//! (`crates/contracts/tests/artifacts_pipeline_contract.rs`) asserts the
//! file + manifest writes only — no SSB row.
//!
//! When the SSB variant lands, callers should invoke `ArtifactStore::publish`
//! and then write a `SsbPayload::ArtifactPublished { task_id, step_id,
//! artifact_id, blake3_hash, ... }` row referencing the manifest's hash.
//! That payload should NOT duplicate the bytes — it points at the canonical
//! on-disk path.
//!
//! ## Hashing
//!
//! `ArtifactManifest::blake3_hash` holds `blake3::hash(&bytes).to_hex()`
//! (lowercase hex, 64 chars). Field name is `blake3_hash` (NOT
//! `sha256_hash`) per CLAUDE.md anti-stale rule: blake3 is the canonical
//! hash crate across the workspace (root Cargo.toml L41).
//!
//! ## Atomicity
//!
//! Both the artifact bytes and the manifest are written via a `.tmp` +
//! `rename` sequence so a crash mid-write cannot leave a half-populated
//! canonical path that downstream readers would treat as valid. Same
//! technique used by `elai_orchestrator::ssb::Ssb::flush_batch` for
//! SSB rows.

use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::UNIX_EPOCH;

use elai_core::clock::ClockProvider;
use elai_core::paths::{artifact_manifest_path, artifact_path};
use serde::{Deserialize, Serialize};
use thiserror::Error;
use tokio::fs;

// ---------------------------------------------------------------------------
// Public types
// ---------------------------------------------------------------------------

/// Input to `ArtifactStore::publish`. Owns its bytes so the store can run
/// the IO + hash off the caller's thread without lifetime juggling.
///
/// Field validation is intentionally lightweight at this layer — the
/// `publish()` call rejects empty `task_id`/`step_id`/`artifact_id`/`ext`
/// but does not enforce a charset. Producing IDs that contain `/`, `..`,
/// or NUL is a contract violation by the caller (the path helpers in
/// `elai_core::paths` join the segments verbatim).
#[derive(Debug, Clone)]
pub struct ContextArtifact {
    /// Stable task identifier (e.g. session-task UUID).
    pub task_id: String,
    /// Step within the task (e.g. "S1", "preview", a step UUID).
    pub step_id: String,
    /// Per-step artifact identifier; combined with `ext` to form the
    /// filename component of the canonical path.
    pub artifact_id: String,
    /// Filesystem extension, without the leading dot (e.g. `"png"`).
    pub ext: String,
    /// Raw bytes to be written verbatim — no transformation applied.
    pub bytes: Vec<u8>,
    /// IANA-style MIME hint (e.g. `"image/png"`); recorded on the manifest.
    pub mime: String,
    /// Free-form producer label (agent name, tool name, "contract-test").
    pub provenance: String,
}

/// JSON-serialised manifest stored next to the artifact bytes. The schema
/// is intentionally narrow for the skeleton; future fields (rendering
/// session id, derived previews, signature) MUST extend, not rename.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ArtifactManifest {
    /// Manifest schema version — bumped when fields are renamed or removed.
    /// Today's value: `1`. Additive changes leave this at `1`.
    pub schema_version: u32,
    /// IANA-style MIME hint mirrored from the input `ContextArtifact`.
    pub mime: String,
    /// Lowercase hex blake3 digest of the artifact bytes (64 chars).
    /// NAME IS `blake3_hash` — CLAUDE.md anti-stale rule forbids
    /// `sha256_hash` / `sha2_hash` for new code.
    pub blake3_hash: String,
    /// Producer label mirrored from the input `ContextArtifact`.
    pub provenance: String,
    /// Wall-clock millis since UNIX epoch at the moment `publish()` ran.
    /// Stored as `i64` to mirror SSB row timestamps (which use the same
    /// type for `SystemTime::now().duration_since(UNIX_EPOCH)`).
    pub timestamp_ms: i64,
    /// Mirror of the `artifact_id` so a manifest read in isolation is
    /// self-describing.
    pub artifact_id: String,
}

/// Per-project root handle. Cheaply cloneable; carries no IO state of its
/// own — every `publish()` call re-resolves the canonical path from the
/// stored `project_root`.
///
/// Holds an injected `Arc<dyn ClockProvider>` (CLAUDE.md B-19 entropy
/// quarantine: production code must not call `SystemTime::now()` raw —
/// tests inject `SeededClock`, prod wires `OsClock`).
#[derive(Clone)]
pub struct ArtifactStore {
    project_root: PathBuf,
    clock: Arc<dyn ClockProvider>,
}

impl std::fmt::Debug for ArtifactStore {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ArtifactStore")
            .field("project_root", &self.project_root)
            .field("clock", &"Arc<dyn ClockProvider>")
            .finish()
    }
}

/// Errors surfaced by `ArtifactStore::publish`.
///
/// `HashFailed` is reserved for future signature/HMAC modes where the
/// digest step can fail (today's `blake3::hash` is infallible, so the
/// variant is unreachable from `publish` and exists only for forward
/// compatibility). Including it now avoids a breaking enum bump when the
/// next phase wires signed manifests.
#[derive(Debug, Error)]
pub enum ArtifactsError {
    /// Empty required field on the input `ContextArtifact`.
    #[error("invalid artifact input: field `{field}` is empty")]
    InvalidInput {
        /// Name of the offending field (e.g. `"task_id"`).
        field: &'static str,
    },
    /// Filesystem write (mkdir / write / rename) failed.
    #[error("artifact io failed at {path}: {source}")]
    Io {
        /// Path the failure references (artifact or manifest).
        path: PathBuf,
        /// Underlying `std::io::Error`.
        #[source]
        source: std::io::Error,
    },
    /// Manifest JSON serialisation failed.
    #[error("manifest serialisation failed: {0}")]
    Serialization(#[from] serde_json::Error),
    /// System clock returned a pre-epoch time. Vanishingly unlikely on
    /// production hosts but propagated rather than panicked.
    #[error("system clock is before UNIX epoch: {0}")]
    Clock(#[from] std::time::SystemTimeError),
    /// Clock returned an epoch-millis value that overflows `i64`. Unreachable
    /// on real hosts (year ~292 million) but propagated rather than wrapped
    /// when a test clock injects a synthetic far-future timestamp.
    #[error("clock returned epoch millis {0} which exceeds i64::MAX")]
    ClockOverflow(u128),
    /// Reserved for future signed-manifest mode (see module doc).
    #[allow(dead_code)]
    #[error("hash computation failed: {0}")]
    HashFailed(String),
}

// ---------------------------------------------------------------------------
// Constructors + accessors
// ---------------------------------------------------------------------------

impl ArtifactStore {
    /// Construct an `ArtifactStore` rooted at `project_root` with an injected
    /// `ClockProvider`. No IO at construction time — directories are created
    /// lazily on `publish()`. Per CLAUDE.md B-19 entropy quarantine, prod
    /// callers pass `Arc::new(elai_core::clock::OsClock)`; tests inject a
    /// `SeededClock` or a custom mock clock for deterministic timestamps.
    pub fn new(project_root: PathBuf, clock: Arc<dyn ClockProvider>) -> Self {
        Self {
            project_root,
            clock,
        }
    }

    /// Read-only view of the project root the store was constructed with.
    pub fn project_root(&self) -> &Path {
        &self.project_root
    }

    /// Write `artifact.bytes` to the canonical artifact path and a JSON
    /// `ArtifactManifest` to the sibling manifest path. Returns
    /// `(artifact_path, manifest_path)` on success.
    ///
    /// Both writes use a `.tmp` + `rename` sequence so a crash partway
    /// through cannot leave a half-populated canonical file.
    pub async fn publish(
        &self,
        artifact: ContextArtifact,
    ) -> Result<(PathBuf, PathBuf), ArtifactsError> {
        // 1. Validate inputs before touching the filesystem.
        validate_non_empty("task_id", &artifact.task_id)?;
        validate_non_empty("step_id", &artifact.step_id)?;
        validate_non_empty("artifact_id", &artifact.artifact_id)?;
        validate_non_empty("ext", &artifact.ext)?;

        // 2. Resolve canonical paths.
        let dest_artifact = artifact_path(
            &self.project_root,
            &artifact.task_id,
            &artifact.step_id,
            &artifact.artifact_id,
            &artifact.ext,
        );
        let dest_manifest = artifact_manifest_path(
            &self.project_root,
            &artifact.task_id,
            &artifact.step_id,
            &artifact.artifact_id,
        );

        // 3. Ensure parent directory exists (both files share it).
        if let Some(parent) = dest_artifact.parent() {
            fs::create_dir_all(parent).await.map_err(|e| ArtifactsError::Io {
                path: parent.to_path_buf(),
                source: e,
            })?;
        }

        // 4. Hash the bytes (blake3 — infallible today; reserved
        //    `HashFailed` variant left for signed-manifest follow-up).
        let blake3_hash = blake3::hash(&artifact.bytes).to_hex().to_string();

        // 5. Build the manifest BEFORE consuming `artifact.bytes` so we
        //    don't have to clone them. Timestamp comes from the injected
        //    `ClockProvider` (B-19 quarantine — no raw `SystemTime::now()`
        //    in production paths).
        let timestamp_ms = epoch_ms_from(self.clock.system_now())?;
        let manifest = ArtifactManifest {
            schema_version: 1,
            mime: artifact.mime,
            blake3_hash,
            provenance: artifact.provenance,
            timestamp_ms,
            artifact_id: artifact.artifact_id.clone(),
        };
        let manifest_bytes = serde_json::to_vec(&manifest)?;

        // 6. Atomic write — artifact bytes first, then manifest. Order
        //    matters: a reader that observes only the manifest (and not
        //    the bytes) has nothing to load; the reverse leaks orphan
        //    bytes that can be GC'd.
        atomic_write(&dest_artifact, &artifact.bytes).await?;
        atomic_write(&dest_manifest, &manifest_bytes).await?;

        Ok((dest_artifact, dest_manifest))
    }
}

// ---------------------------------------------------------------------------
// Internal helpers
// ---------------------------------------------------------------------------

fn validate_non_empty(field: &'static str, value: &str) -> Result<(), ArtifactsError> {
    if value.is_empty() {
        Err(ArtifactsError::InvalidInput { field })
    } else {
        Ok(())
    }
}

/// Convert a clock-provided `SystemTime` to epoch millis (i64), propagating
/// pre-epoch times as `Clock` and high-end overflow as `ClockOverflow`.
/// Standalone (not a method) so the in-module tests can drive it with a
/// synthetic `SystemTime` without constructing a full `ArtifactStore`.
fn epoch_ms_from(when: std::time::SystemTime) -> Result<i64, ArtifactsError> {
    let dur = when.duration_since(UNIX_EPOCH)?;
    // u128 -> i64 narrowing via .try_into() (anti-stale-training rule: no
    // `as`-cast for narrowing). On overflow surface the dedicated
    // `ClockOverflow` variant — re-using `HashFailed` was a category error
    // (B-19 audit BLOCK-2).
    let ms_u128 = dur.as_millis();
    i64::try_from(ms_u128).map_err(|_| ArtifactsError::ClockOverflow(ms_u128))
}

/// Write `bytes` to `dest` via a `.tmp` sibling + `rename`. The temp file
/// lives in the same directory as `dest` so the rename is guaranteed to
/// stay on a single filesystem.
async fn atomic_write(dest: &Path, bytes: &[u8]) -> Result<(), ArtifactsError> {
    let tmp = tmp_path_for(dest);
    fs::write(&tmp, bytes).await.map_err(|e| ArtifactsError::Io {
        path: tmp.clone(),
        source: e,
    })?;
    fs::rename(&tmp, dest).await.map_err(|e| ArtifactsError::Io {
        path: dest.to_path_buf(),
        source: e,
    })?;
    Ok(())
}

/// Build the `.tmp` sibling path used by `atomic_write`. We append a
/// fixed `.tmp` suffix to the destination's full filename so concurrent
/// publishes of different artifacts in the same directory cannot collide.
fn tmp_path_for(dest: &Path) -> PathBuf {
    let mut tmp = dest.as_os_str().to_owned();
    tmp.push(".tmp");
    PathBuf::from(tmp)
}

// ---------------------------------------------------------------------------
// Module-local tests
//
// Hermetic smoke tests for the helpers that the contract test does NOT
// exercise directly (validation + tmp-path construction). The end-to-end
// publish() flow is covered by
// `crates/contracts/tests/artifacts_pipeline_contract.rs`.
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn validate_non_empty_rejects_empty_strings() {
        let err = validate_non_empty("task_id", "").expect_err("empty must error");
        match err {
            ArtifactsError::InvalidInput { field } => assert_eq!(field, "task_id"),
            other => panic!("unexpected variant: {other:?}"),
        }
    }

    #[test]
    fn validate_non_empty_accepts_non_empty_strings() {
        validate_non_empty("task_id", "t").expect("non-empty must pass");
    }

    #[test]
    fn tmp_path_appends_suffix() {
        let dest = PathBuf::from("/a/b/c/foo.png");
        let tmp = tmp_path_for(&dest);
        assert_eq!(tmp, PathBuf::from("/a/b/c/foo.png.tmp"));
    }

    #[test]
    fn epoch_ms_from_emits_clock_overflow_when_value_exceeds_i64_max() {
        // i64::MAX millis ~ 9.22e18 ms ~ 9.22e15 s. Add 1 second past that
        // to force the u128 -> i64 narrowing failure.
        let overflow_secs: u64 = (i64::MAX as u64) / 1000 + 1;
        let when = std::time::SystemTime::UNIX_EPOCH
            + std::time::Duration::from_secs(overflow_secs);
        let err = epoch_ms_from(when).expect_err("must overflow");
        match err {
            ArtifactsError::ClockOverflow(ms) => {
                assert!(
                    ms > i64::MAX as u128,
                    "ClockOverflow payload {ms} must exceed i64::MAX",
                );
            }
            other => panic!("expected ClockOverflow, got {other:?}"),
        }
    }

    #[tokio::test]
    async fn publish_rejects_empty_task_id() {
        let tmp = tempfile::TempDir::new().expect("tempdir");
        let clock: Arc<dyn ClockProvider> = Arc::new(elai_core::clock::OsClock);
        let store = ArtifactStore::new(tmp.path().to_path_buf(), clock);
        let bad = ContextArtifact {
            task_id: String::new(),
            step_id: "s".into(),
            artifact_id: "a".into(),
            ext: "png".into(),
            bytes: vec![1, 2, 3],
            mime: "image/png".into(),
            provenance: "test".into(),
        };
        let err = store.publish(bad).await.expect_err("empty task_id must fail");
        assert!(matches!(
            err,
            ArtifactsError::InvalidInput { field: "task_id" }
        ));
    }
}
