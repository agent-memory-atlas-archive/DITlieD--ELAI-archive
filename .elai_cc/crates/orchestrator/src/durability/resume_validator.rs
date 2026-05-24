//! E-44 — Checkpoint signature + schema + integrity validation
//! before a resume attempt.
//!
//! Validates an on-disk checkpoint file produced by
//! `CheckpointManager::write` (4 msgpack + 1 ASCII-digit timestamp,
//! per `checkpoint.rs:23-29`) before the orchestrator attempts to
//! resume from it. Mirrors `CheckpointManager::read_latest`'s
//! defensive posture (treat any inconsistency as Ok(None)-equivalent)
//! but surfaces structured diagnostics so the operator can see WHY
//! the checkpoint was rejected.

use std::path::{Path, PathBuf};

use thiserror::Error;

/// Outcome of a checkpoint validity check.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CheckpointValidity {
    /// All expected files present, schema_version in supported range,
    /// blake3 content hash matches the supplied expected digest (or
    /// no expected digest was supplied — verify-presence-only mode).
    Valid {
        schema_version: u32,
        blake3_hex: String,
    },
    /// One or more required files missing (partial checkpoint per
    /// `CheckpointManager::read_latest` semantics).
    PartialFiles { missing: Vec<PathBuf> },
    /// state.msgpack present but did not decode as
    /// `AgentCheckpointData`.
    Corrupt { detail: String },
    /// blake3 content hash mismatched the supplied expected digest.
    HashMismatch {
        expected: String,
        actual: String,
    },
    /// schema_version is below the supported floor (older than
    /// `LEGACY_CHECKPOINT_SCHEMA_VERSION`) or above the current
    /// ceiling (forward-compat read of unknown future version).
    UnsupportedSchemaVersion {
        observed: u32,
        legacy_floor: u32,
        current_ceiling: u32,
    },
}

/// Errors raised by [`validate_checkpoint`].
#[derive(Debug, Error)]
pub enum ResumeValidatorError {
    /// Could not stat the directory at all (not the same as
    /// missing-file — this is a hard I/O failure).
    #[error("resume validator I/O error at {path}: {source}")]
    Io {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },
}

const STATE_FILE: &str = "state.msgpack";
const GRC_HISTORY_FILE: &str = "grc_history.msgpack";
const TOOL_RESULTS_FILE: &str = "tool_results.msgpack";
const OUTPUT_DRAFT_FILE: &str = "output_draft.msgpack";
const CHECKPOINT_TS_FILE: &str = "checkpoint_ts";

/// All checkpoint files that must be present for a valid checkpoint
/// (mirrors `checkpoint.rs:23-29`).
const CHECKPOINT_FILES: [&str; 5] = [
    STATE_FILE,
    GRC_HISTORY_FILE,
    TOOL_RESULTS_FILE,
    OUTPUT_DRAFT_FILE,
    CHECKPOINT_TS_FILE,
];

/// Validate a checkpoint directory. When `expected_blake3` is
/// `Some`, the state.msgpack content hash is compared against it; on
/// mismatch returns [`CheckpointValidity::HashMismatch`].
///
/// Wire-dark anchor: invoked by `elai durability validate-checkpoint
/// --path PATH` (see `crates/cli/src/durability_cmd.rs`).
pub fn validate_checkpoint(
    checkpoint_dir: &Path,
    expected_blake3: Option<&str>,
) -> Result<CheckpointValidity, ResumeValidatorError> {
    // 1. Directory must exist + be a directory.
    let meta = std::fs::metadata(checkpoint_dir).map_err(|e| ResumeValidatorError::Io {
        path: checkpoint_dir.to_path_buf(),
        source: e,
    })?;
    if !meta.is_dir() {
        return Ok(CheckpointValidity::PartialFiles {
            missing: CHECKPOINT_FILES
                .iter()
                .map(|f| checkpoint_dir.join(f))
                .collect(),
        });
    }

    // 2. Every required file must exist.
    let mut missing: Vec<PathBuf> = Vec::new();
    for file in &CHECKPOINT_FILES {
        let p = checkpoint_dir.join(file);
        if std::fs::metadata(&p).is_err() {
            missing.push(p);
        }
    }
    if !missing.is_empty() {
        return Ok(CheckpointValidity::PartialFiles { missing });
    }

    // 3. state.msgpack must decode + carry a parseable schema_version.
    let state_bytes = std::fs::read(checkpoint_dir.join(STATE_FILE)).map_err(|e| {
        ResumeValidatorError::Io {
            path: checkpoint_dir.join(STATE_FILE),
            source: e,
        }
    })?;
    let data: crate::checkpoint::AgentCheckpointData =
        match rmp_serde::from_slice(&state_bytes) {
            Ok(d) => d,
            Err(e) => {
                return Ok(CheckpointValidity::Corrupt {
                    detail: format!("state.msgpack decode failed: {e}"),
                });
            }
        };

    // 4. Schema version must be within supported range.
    let legacy_floor = crate::checkpoint::LEGACY_CHECKPOINT_SCHEMA_VERSION;
    let ceiling = crate::durability::checkpoint_migrator::CURRENT_CHECKPOINT_SCHEMA_VERSION;
    if data.schema_version < legacy_floor || data.schema_version > ceiling {
        return Ok(CheckpointValidity::UnsupportedSchemaVersion {
            observed: data.schema_version,
            legacy_floor,
            current_ceiling: ceiling,
        });
    }

    // 5. Content hash (blake3) check, if expected hash supplied.
    let actual_hash = blake3::hash(&state_bytes).to_hex().to_string();
    if let Some(expected) = expected_blake3 {
        if expected != actual_hash {
            return Ok(CheckpointValidity::HashMismatch {
                expected: expected.to_owned(),
                actual: actual_hash,
            });
        }
    }

    Ok(CheckpointValidity::Valid {
        schema_version: data.schema_version,
        blake3_hex: actual_hash,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::checkpoint::{AgentCheckpointData, LEGACY_CHECKPOINT_SCHEMA_VERSION};
    use tempfile::tempdir;

    fn write_minimal_checkpoint(dir: &Path, schema_version: u32) -> AgentCheckpointData {
        let data = AgentCheckpointData {
            agent_status: "running".into(),
            grc_history: vec![],
            active_files: vec![],
            tool_results: vec![],
            output_draft: "".into(),
            channel_references: vec![],
            schema_version,
        };
        std::fs::create_dir_all(dir).expect("mkdir");
        let bytes = rmp_serde::to_vec_named(&data).expect("encode");
        std::fs::write(dir.join(STATE_FILE), &bytes).expect("state");
        std::fs::write(
            dir.join(GRC_HISTORY_FILE),
            rmp_serde::to_vec_named(&data.grc_history).expect("grc encode"),
        )
        .expect("grc write");
        std::fs::write(
            dir.join(TOOL_RESULTS_FILE),
            rmp_serde::to_vec_named(&data.tool_results).expect("tool encode"),
        )
        .expect("tool write");
        std::fs::write(
            dir.join(OUTPUT_DRAFT_FILE),
            rmp_serde::to_vec_named(&data.output_draft).expect("draft encode"),
        )
        .expect("draft write");
        std::fs::write(dir.join(CHECKPOINT_TS_FILE), b"1700000000").expect("ts write");
        data
    }

    #[test]
    fn empty_dir_reports_all_missing() {
        let dir = tempdir().expect("tempdir");
        let v = validate_checkpoint(dir.path(), None).expect("ok");
        match v {
            CheckpointValidity::PartialFiles { missing } => {
                assert_eq!(missing.len(), CHECKPOINT_FILES.len());
            }
            other => panic!("expected PartialFiles, got {other:?}"),
        }
    }

    #[test]
    fn valid_checkpoint_passes() {
        let dir = tempdir().expect("tempdir");
        write_minimal_checkpoint(dir.path(), LEGACY_CHECKPOINT_SCHEMA_VERSION);
        let v = validate_checkpoint(dir.path(), None).expect("ok");
        assert!(matches!(v, CheckpointValidity::Valid { .. }));
    }

    #[test]
    fn corrupt_state_file_reports_corrupt() {
        let dir = tempdir().expect("tempdir");
        write_minimal_checkpoint(dir.path(), LEGACY_CHECKPOINT_SCHEMA_VERSION);
        std::fs::write(dir.path().join(STATE_FILE), b"not msgpack").expect("overwrite");
        let v = validate_checkpoint(dir.path(), None).expect("ok");
        assert!(matches!(v, CheckpointValidity::Corrupt { .. }));
    }

    #[test]
    fn unsupported_schema_version_below_floor() {
        let dir = tempdir().expect("tempdir");
        // Write with the legacy floor, then corrupt the schema field by
        // re-encoding with a version below floor.
        let mut data = write_minimal_checkpoint(dir.path(), LEGACY_CHECKPOINT_SCHEMA_VERSION);
        data.schema_version = LEGACY_CHECKPOINT_SCHEMA_VERSION.saturating_sub(1);
        std::fs::write(
            dir.path().join(STATE_FILE),
            rmp_serde::to_vec_named(&data).expect("encode"),
        )
        .expect("overwrite");
        let v = validate_checkpoint(dir.path(), None).expect("ok");
        assert!(matches!(
            v,
            CheckpointValidity::UnsupportedSchemaVersion { .. }
        ));
    }

    #[test]
    fn hash_mismatch_when_expected_does_not_match() {
        let dir = tempdir().expect("tempdir");
        write_minimal_checkpoint(dir.path(), LEGACY_CHECKPOINT_SCHEMA_VERSION);
        let v = validate_checkpoint(dir.path(), Some("0".repeat(64).as_str())).expect("ok");
        assert!(matches!(v, CheckpointValidity::HashMismatch { .. }));
    }

    #[test]
    fn io_error_on_missing_dir() {
        let path = PathBuf::from("/nonexistent-e44-validator-test");
        let err = validate_checkpoint(&path, None).expect_err("must error");
        assert!(matches!(err, ResumeValidatorError::Io { .. }));
    }
}
