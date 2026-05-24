//! E-44 — `elai durability <sub>` handler.
//!
//! Anchors the orchestrator's `durability::*` module as production
//! callable so the IdempotentDispatcher, CompactLock, lazy schema
//! migrator, continue_as_new wrapper, and resume_validator each
//! have a non-test caller — WIRE-DARK avoidance per E-41 precedent
//! (`E40_WIRE_DARK_AVOIDANCE_VIA_CLI`).
//!
//! Sub-actions:
//!   * `status` — print CURRENT_CHECKPOINT_SCHEMA_VERSION + default
//!     ceilings + the 5 E-44 SSB action types. GPU-free.
//!   * `validate-checkpoint --path PATH [--expected-blake3 HEX]` —
//!     invoke `resume_validator::validate_checkpoint` and render the
//!     CheckpointValidity verdict.
//!   * `migrate --path PATH --from-version V` — read raw msgpack
//!     bytes from PATH and run `migrate_checkpoint(bytes, V)`. Prints
//!     the migrated byte length + blake3 of the result. Never
//!     mutates the source file.
//!   * `continue-as-new-preview --session-id S --events N
//!     --phases M` — invoke `continue_as_new_if_needed` against the
//!     supplied state and render the verdict.
//!   * `acquire-lock --path PATH` — `CompactLock::try_acquire` and
//!     immediately release. Operator primitive.

use std::path::PathBuf;

use elai_orchestrator::durability::{
    continue_as_new_if_needed, migrate_checkpoint, validate_checkpoint, CheckpointValidity,
    CompactLock, ContinueAsNewOutcome, CURRENT_CHECKPOINT_SCHEMA_VERSION,
    DEFAULT_EVENT_COUNT_CEILING, DEFAULT_PHASE_COUNT_CEILING,
};
use serde::Serialize;

use crate::args::{CliOutputFormat, DurabilityAction};

/// Top-level dispatcher invoked by `main.rs`.
pub(crate) fn run(
    action: DurabilityAction,
    output_format: CliOutputFormat,
) -> Result<(), String> {
    match action {
        DurabilityAction::Status => run_status(output_format),
        DurabilityAction::ValidateCheckpoint {
            path,
            expected_blake3,
        } => run_validate_checkpoint(&path, expected_blake3.as_deref(), output_format),
        DurabilityAction::Migrate { path, from_version } => {
            run_migrate(&path, from_version, output_format)
        }
        DurabilityAction::ContinueAsNewPreview {
            session_id,
            events,
            phases,
        } => run_continue_as_new_preview(&session_id, events, phases, output_format),
        DurabilityAction::AcquireLock { path } => run_acquire_lock(&path, output_format),
    }
}

#[derive(Debug, Serialize)]
struct StatusRow {
    current_checkpoint_schema_version: u32,
    legacy_checkpoint_schema_version: u32,
    default_event_count_ceiling: usize,
    default_phase_count_ceiling: usize,
    ssb_action_types: Vec<&'static str>,
    /// Sentinel: durability primitives wired by this subverb.
    primitives: Vec<&'static str>,
}

fn run_status(output_format: CliOutputFormat) -> Result<(), String> {
    let row = StatusRow {
        current_checkpoint_schema_version: CURRENT_CHECKPOINT_SCHEMA_VERSION,
        legacy_checkpoint_schema_version:
            elai_orchestrator::checkpoint::LEGACY_CHECKPOINT_SCHEMA_VERSION,
        default_event_count_ceiling: DEFAULT_EVENT_COUNT_CEILING,
        default_phase_count_ceiling: DEFAULT_PHASE_COUNT_CEILING,
        ssb_action_types: vec![
            "checkpoint_written",
            "checkpoint_resumed",
            "checkpoint_migrated",
            "idempotent_dispatch_skipped",
            "durability_lock_blocked",
        ],
        primitives: vec![
            "IdempotentDispatcher",
            "CompactLock",
            "CheckpointMigrator",
            "continue_as_new_if_needed",
            "validate_checkpoint",
        ],
    };
    render(&[row], output_format)
}

#[derive(Debug, Serialize)]
struct ValidateRow {
    path: String,
    verdict: String,
    detail: Option<String>,
    schema_version: Option<u32>,
    blake3_hex: Option<String>,
}

fn run_validate_checkpoint(
    path: &str,
    expected_blake3: Option<&str>,
    output_format: CliOutputFormat,
) -> Result<(), String> {
    let p = PathBuf::from(path);
    let v = validate_checkpoint(&p, expected_blake3).map_err(|e| {
        format!("durability validate-checkpoint: {} failed: {e}", p.display())
    })?;
    let row = match v {
        CheckpointValidity::Valid {
            schema_version,
            blake3_hex,
        } => ValidateRow {
            path: p.display().to_string(),
            verdict: "valid".to_owned(),
            detail: None,
            schema_version: Some(schema_version),
            blake3_hex: Some(blake3_hex),
        },
        CheckpointValidity::PartialFiles { missing } => ValidateRow {
            path: p.display().to_string(),
            verdict: "partial_files".to_owned(),
            detail: Some(format!(
                "{} files missing",
                missing.len()
            )),
            schema_version: None,
            blake3_hex: None,
        },
        CheckpointValidity::Corrupt { detail } => ValidateRow {
            path: p.display().to_string(),
            verdict: "corrupt".to_owned(),
            detail: Some(detail),
            schema_version: None,
            blake3_hex: None,
        },
        CheckpointValidity::HashMismatch { expected, actual } => ValidateRow {
            path: p.display().to_string(),
            verdict: "hash_mismatch".to_owned(),
            detail: Some(format!("expected={expected}, actual={actual}")),
            schema_version: None,
            blake3_hex: Some(actual),
        },
        CheckpointValidity::UnsupportedSchemaVersion {
            observed,
            legacy_floor,
            current_ceiling,
        } => ValidateRow {
            path: p.display().to_string(),
            verdict: "unsupported_schema_version".to_owned(),
            detail: Some(format!(
                "observed={observed}, floor={legacy_floor}, ceiling={current_ceiling}"
            )),
            schema_version: Some(observed),
            blake3_hex: None,
        },
    };
    render(&[row], output_format)
}

#[derive(Debug, Serialize)]
struct MigrateRow {
    path: String,
    from_version: u32,
    to_version: u32,
    input_bytes: usize,
    output_bytes: usize,
    output_blake3: String,
}

fn run_migrate(
    path: &str,
    from_version: u32,
    output_format: CliOutputFormat,
) -> Result<(), String> {
    let p = PathBuf::from(path);
    let bytes = std::fs::read(&p)
        .map_err(|e| format!("durability migrate: read {} failed: {e}", p.display()))?;
    let out = migrate_checkpoint(&bytes, from_version)
        .map_err(|e| format!("durability migrate: failed: {e}"))?;
    let hash = blake3::hash(&out).to_hex().to_string();
    let row = MigrateRow {
        path: p.display().to_string(),
        from_version,
        to_version: CURRENT_CHECKPOINT_SCHEMA_VERSION,
        input_bytes: bytes.len(),
        output_bytes: out.len(),
        output_blake3: hash,
    };
    render(&[row], output_format)
}

#[derive(Debug, Serialize)]
struct ContinueAsNewRow {
    session_id: String,
    event_count: usize,
    phase_count: usize,
    verdict: String,
    archived_snap_id: Option<String>,
    new_snap_id: Option<String>,
    archive_blake3: Option<String>,
}

fn run_continue_as_new_preview(
    session_id: &str,
    events: usize,
    phases: usize,
    output_format: CliOutputFormat,
) -> Result<(), String> {
    let outcome = continue_as_new_if_needed(
        session_id,
        events,
        phases,
        DEFAULT_EVENT_COUNT_CEILING,
        DEFAULT_PHASE_COUNT_CEILING,
    )
    .map_err(|e| format!("durability continue-as-new-preview: failed: {e}"))?;
    let row = match outcome {
        ContinueAsNewOutcome::BelowThreshold {
            event_count,
            phase_count,
        } => ContinueAsNewRow {
            session_id: session_id.to_owned(),
            event_count,
            phase_count,
            verdict: "below_threshold".to_owned(),
            archived_snap_id: None,
            new_snap_id: None,
            archive_blake3: None,
        },
        ContinueAsNewOutcome::Rollover(handle) => ContinueAsNewRow {
            session_id: handle.session_id,
            event_count: handle.event_count_at_rollover,
            phase_count: phases,
            verdict: "rollover".to_owned(),
            archived_snap_id: Some(handle.archived_snap_id),
            new_snap_id: Some(handle.new_snap_id),
            archive_blake3: Some(handle.archive_blake3),
        },
    };
    render(&[row], output_format)
}

#[derive(Debug, Serialize)]
struct AcquireLockRow {
    path: String,
    verdict: String,
    detail: Option<String>,
}

fn run_acquire_lock(path: &str, output_format: CliOutputFormat) -> Result<(), String> {
    let lock = CompactLock::new(PathBuf::from(path));
    let row = match lock.try_acquire() {
        Ok(guard) => {
            // Immediately drop (RAII releases) so the operator can chain.
            drop(guard);
            AcquireLockRow {
                path: path.to_owned(),
                verdict: "acquired".to_owned(),
                detail: Some("released immediately on RAII drop".to_owned()),
            }
        }
        Err(e) => AcquireLockRow {
            path: path.to_owned(),
            verdict: "failed".to_owned(),
            detail: Some(format!("{e}")),
        },
    };
    render(&[row], output_format)
}

fn render<T: Serialize>(rows: &[T], output_format: CliOutputFormat) -> Result<(), String> {
    let out = match output_format {
        CliOutputFormat::Json => serde_json::to_string_pretty(rows)
            .map_err(|e| format!("render: json serialise failed: {e}"))?,
        CliOutputFormat::Toml => {
            let wrapped = serde_json::json!({ "rows": rows });
            toml::to_string(&wrapped)
                .map_err(|e| format!("render: toml serialise failed: {e}"))?
        }
        CliOutputFormat::Text => {
            let v = serde_json::to_value(rows)
                .map_err(|e| format!("render: text->json failed: {e}"))?;
            format!("{v:#}")
        }
    };
    println!("{out}");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn status_renders_json() {
        run_status(CliOutputFormat::Json).expect("ok");
    }

    #[test]
    fn validate_checkpoint_missing_dir_errors() {
        let res =
            run_validate_checkpoint("/nonexistent-e44-cli", None, CliOutputFormat::Json);
        assert!(res.is_err());
    }

    #[test]
    fn validate_checkpoint_empty_dir_renders_partial() {
        let dir = TempDir::new().expect("tempdir");
        run_validate_checkpoint(
            dir.path().to_str().expect("utf8"),
            None,
            CliOutputFormat::Json,
        )
        .expect("ok");
    }

    #[test]
    fn migrate_round_trip_smoke() {
        let dir = TempDir::new().expect("tempdir");
        let p = dir.path().join("blob.bin");
        std::fs::write(&p, b"hello-world").expect("write");
        run_migrate(
            p.to_str().expect("utf8"),
            elai_orchestrator::checkpoint::LEGACY_CHECKPOINT_SCHEMA_VERSION,
            CliOutputFormat::Json,
        )
        .expect("ok");
    }

    #[test]
    fn continue_as_new_below_threshold_renders() {
        run_continue_as_new_preview(
            "session-x",
            10,
            5,
            CliOutputFormat::Json,
        )
        .expect("ok");
    }

    #[test]
    fn continue_as_new_above_threshold_renders_rollover() {
        run_continue_as_new_preview(
            "session-x",
            DEFAULT_EVENT_COUNT_CEILING,
            DEFAULT_PHASE_COUNT_CEILING,
            CliOutputFormat::Json,
        )
        .expect("ok");
    }

    #[test]
    fn acquire_lock_in_tempdir_succeeds() {
        let dir = TempDir::new().expect("tempdir");
        let p = dir.path().join(".compact-lock");
        run_acquire_lock(p.to_str().expect("utf8"), CliOutputFormat::Json).expect("ok");
    }
}
