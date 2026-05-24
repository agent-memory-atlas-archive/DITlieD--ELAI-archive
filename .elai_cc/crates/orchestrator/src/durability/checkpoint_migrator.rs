//! E-44 — Lazy AgentCheckpoint schema migrator.
//!
//! Reads the `schema_version` stamped on an [`AgentCheckpointData`] blob
//! and chains registered transform functions to bring it up to
//! [`CURRENT_CHECKPOINT_SCHEMA_VERSION`]. Transforms run at READ side
//! only — on-disk files are never mutated by this module; the
//! `CheckpointManager::write` path is the only place that writes new
//! files, and it always stamps the current schema version directly.
//!
//! Forward-compat reader pattern (langgraphjs#536): unknown future
//! versions are passed through verbatim; missing baseline versions
//! resolve via `serde(default)` on `AgentCheckpointData::schema_version`
//! (default = [`crate::checkpoint::LEGACY_CHECKPOINT_SCHEMA_VERSION`]).
//!
//! Transforms operate on raw msgpack bytes (`Vec<u8>`) so they can
//! rewrite the wire shape (rename fields, fold optional fields, etc)
//! before the final `rmp_serde::from_slice` runs.

use rustc_hash::FxHashMap;
use thiserror::Error;

/// Current schema version stamped on newly written checkpoint files.
/// Atomic-bumped with the SSB SCHEMA_VERSION per the E-44 contract.
pub const CURRENT_CHECKPOINT_SCHEMA_VERSION: u32 = 136;

/// Function that rewrites a raw msgpack-serialised checkpoint blob from
/// one schema version to the next. Takes ownership of the input bytes
/// and returns the rewritten bytes, or an error if the transform fails.
pub type TransformFn = fn(&[u8]) -> Result<Vec<u8>, MigratorError>;

/// Errors that surface from the lazy migrator.
#[derive(Debug, Error)]
pub enum MigratorError {
    /// No transform registered to bridge `from -> to`.
    #[error("no transform registered from version {from} to {to}")]
    NoTransform { from: u32, to: u32 },

    /// A registered transform failed during execution.
    #[error("transform from {from} to {to} failed: {reason}")]
    TransformFailed {
        from: u32,
        to: u32,
        reason: String,
    },

    /// The target version is below the input version (downgrade attempt).
    #[error("downgrade not supported: from {from} to {to}")]
    Downgrade { from: u32, to: u32 },

    /// Migration chain exceeded the safety upper bound (cycle guard).
    #[error("migration chain exceeded safety limit at version {at}")]
    ChainTooLong { at: u32 },
}

/// Registry of versioned transforms keyed by `(from, to)` pair.
///
/// Append-only contract: removing or renaming an entry is BLOCK per
/// `stub_ban` in the E-44 plan. New entries are added each time
/// `CURRENT_CHECKPOINT_SCHEMA_VERSION` is bumped.
pub struct CheckpointMigrator {
    transforms: FxHashMap<(u32, u32), TransformFn>,
}

impl Default for CheckpointMigrator {
    fn default() -> Self {
        Self::new()
    }
}

impl CheckpointMigrator {
    /// Construct an empty registry. Production callers should use
    /// [`CheckpointMigrator::with_defaults`] which preregisters the
    /// no-op identity transforms for every known schema gap so the
    /// migrator never returns `NoTransform` on a recognised version.
    pub fn new() -> Self {
        Self {
            transforms: FxHashMap::default(),
        }
    }

    /// Construct a registry preloaded with identity transforms covering
    /// every known schema gap from
    /// [`crate::checkpoint::LEGACY_CHECKPOINT_SCHEMA_VERSION`] (40) up to
    /// [`CURRENT_CHECKPOINT_SCHEMA_VERSION`].
    ///
    /// E-44 ships identity transforms (passthrough) for every version
    /// gap because the `schema_version` field is the FIRST checkpoint
    /// schema bump; no historical version actually requires wire
    /// rewriting. Future versions append real transforms here.
    pub fn with_defaults() -> Self {
        let mut me = Self::new();
        let mut v = crate::checkpoint::LEGACY_CHECKPOINT_SCHEMA_VERSION;
        while v < CURRENT_CHECKPOINT_SCHEMA_VERSION {
            me.transforms.insert((v, v + 1), identity_transform);
            v += 1;
        }
        me
    }

    /// Append a transform bridging `from -> to`. The transform must be
    /// idempotent and free of clock / entropy side effects (any
    /// non-determinism is a stub_ban violation).
    pub fn register_transform(&mut self, from: u32, to: u32, f: TransformFn) {
        self.transforms.insert((from, to), f);
    }

    /// Chain transforms from `from_version` up to `target_version`,
    /// returning the migrated bytes. Calls [`Self::transform_step`] for
    /// every intermediate version.
    pub fn migrate(
        &self,
        data: &[u8],
        from_version: u32,
        target_version: u32,
    ) -> Result<Vec<u8>, MigratorError> {
        if from_version > target_version {
            return Err(MigratorError::Downgrade {
                from: from_version,
                to: target_version,
            });
        }
        // Cycle guard: a single migrate() call walks at most
        // (CURRENT - LEGACY + 32) steps. The +32 accommodates future
        // schema bumps within a single session.
        let safety_limit = CURRENT_CHECKPOINT_SCHEMA_VERSION
            .saturating_sub(crate::checkpoint::LEGACY_CHECKPOINT_SCHEMA_VERSION)
            .saturating_add(32);
        let mut buf: Vec<u8> = data.to_vec();
        let mut current = from_version;
        let mut steps: u32 = 0;
        while current < target_version {
            if steps >= safety_limit {
                return Err(MigratorError::ChainTooLong { at: current });
            }
            buf = self.transform_step(&buf, current, current + 1)?;
            current += 1;
            steps += 1;
        }
        Ok(buf)
    }

    fn transform_step(
        &self,
        data: &[u8],
        from: u32,
        to: u32,
    ) -> Result<Vec<u8>, MigratorError> {
        let f = self
            .transforms
            .get(&(from, to))
            .ok_or(MigratorError::NoTransform { from, to })?;
        f(data).map_err(|e| match e {
            MigratorError::TransformFailed { reason, .. } => {
                MigratorError::TransformFailed { from, to, reason }
            }
            other => other,
        })
    }
}

/// Identity transform: copies the input bytes verbatim. Used by every
/// E-44 gap pre-registration because no historical schema bump
/// actually rewrote the wire shape (the `schema_version` field is
/// itself the first additive bump).
fn identity_transform(data: &[u8]) -> Result<Vec<u8>, MigratorError> {
    Ok(data.to_vec())
}

/// Convenience: migrate raw msgpack bytes from `from_version` to
/// [`CURRENT_CHECKPOINT_SCHEMA_VERSION`] using the default registry.
///
/// Wire-dark anchor: this is the production callable invoked by the
/// `elai durability migrate-checkpoint` CLI subverb (see
/// `crates/cli/src/durability_cmd.rs`).
pub fn migrate_checkpoint(data: &[u8], from_version: u32) -> Result<Vec<u8>, MigratorError> {
    CheckpointMigrator::with_defaults().migrate(
        data,
        from_version,
        CURRENT_CHECKPOINT_SCHEMA_VERSION,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn identity_passthrough_returns_input_verbatim() {
        let input = b"hello-world".to_vec();
        let out = identity_transform(&input).expect("identity ok");
        assert_eq!(out, input);
    }

    #[test]
    fn migrator_default_covers_every_gap_to_current() {
        let m = CheckpointMigrator::with_defaults();
        let input = b"payload".to_vec();
        let out = m
            .migrate(
                &input,
                crate::checkpoint::LEGACY_CHECKPOINT_SCHEMA_VERSION,
                CURRENT_CHECKPOINT_SCHEMA_VERSION,
            )
            .expect("migrate ok");
        assert_eq!(out, input, "identity chain must round-trip");
    }

    #[test]
    fn migrator_rejects_downgrade() {
        let m = CheckpointMigrator::with_defaults();
        let err = m
            .migrate(b"x", CURRENT_CHECKPOINT_SCHEMA_VERSION, 40)
            .expect_err("downgrade must error");
        assert!(matches!(err, MigratorError::Downgrade { .. }));
    }

    #[test]
    fn migrator_no_transform_when_gap_unregistered() {
        let m = CheckpointMigrator::new();
        let err = m
            .migrate(b"x", 40, 41)
            .expect_err("no transform registered");
        assert!(matches!(err, MigratorError::NoTransform { from: 40, to: 41 }));
    }

    #[test]
    fn migrate_checkpoint_helper_uses_defaults() {
        let input = b"abc".to_vec();
        let out = migrate_checkpoint(&input, crate::checkpoint::LEGACY_CHECKPOINT_SCHEMA_VERSION)
            .expect("ok");
        assert_eq!(out, input);
    }

    #[test]
    fn migrator_same_version_is_noop() {
        let m = CheckpointMigrator::with_defaults();
        let out = m
            .migrate(b"unchanged", 50, 50)
            .expect("same version noop ok");
        assert_eq!(out, b"unchanged");
    }
}
