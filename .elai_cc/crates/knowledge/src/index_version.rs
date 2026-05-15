use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::bm25::INDEX_FORMAT_VERSION;
use crate::vector::{CODE_VEC_DIM, TEXT_VEC_DIM};

// ---------------------------------------------------------------------------
// Constants
// ---------------------------------------------------------------------------

/// Current schema version. Bump when DKG columns, vector dims, or other
/// structural changes require a full rebuild from source.
// v2 reserved per B-29 plan terminology; jumped 1->3 (B-29 Phase 1 symbol graph tables).
pub const CURRENT_SCHEMA_VERSION: u32 = 3;

/// Current index format version. Tracks the overall index layout; a change
/// here triggers at minimum a BM25-only rebuild.
/// v2: AST-aware chunking replaces 50-line fixed windows (3K.1).
pub const CURRENT_INDEX_VERSION: u32 = 2;

// ---------------------------------------------------------------------------
// IndexVersion
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct VectorDimensions {
    pub code: usize,
    pub text: usize,
}

/// Mirrors the `[index]` table in `index_version.toml`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct IndexVersionSchema {
    pub version: u32,
    pub dkg_schema_version: u32,
    pub vector_dimensions: VectorDimensions,
    pub bm25_format: String,
    pub created_at: i64,
}

/// Top-level TOML wrapper (the file has `[index]` as the only table).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct IndexVersionFile {
    pub index: IndexVersionSchema,
}

// ---------------------------------------------------------------------------
// UpgradeAction
// ---------------------------------------------------------------------------

/// What the caller should do after comparing the on-disk version against the
/// running binary's version.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UpgradeAction {
    /// Index matches current version — reuse as-is.
    Reuse,
    /// Only the BM25 format changed — rebuild BM25 layer, keep vectors/DKG.
    RebuildBm25Only,
    /// Structural change (schema, vector dims) — full rebuild from source.
    FullRebuild,
}

// ---------------------------------------------------------------------------
// Errors
// ---------------------------------------------------------------------------

#[derive(Debug)]
pub enum IndexVersionError {
    Io(std::io::Error),
    Parse(toml::de::Error),
    Serialize(toml::ser::Error),
    ArchiveFailed {
        source: std::io::Error,
        message: String,
    },
}

impl std::fmt::Display for IndexVersionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Io(e) => write!(f, "index version I/O error: {e}"),
            Self::Parse(e) => write!(f, "index version parse error: {e}"),
            Self::Serialize(e) => write!(f, "index version serialize error: {e}"),
            Self::ArchiveFailed { source, message } => {
                write!(f, "archive failed: {message}: {source}")
            }
        }
    }
}

impl std::error::Error for IndexVersionError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Io(e) => Some(e),
            Self::Parse(e) => Some(e),
            Self::Serialize(e) => Some(e),
            Self::ArchiveFailed { source, .. } => Some(source),
        }
    }
}

impl From<std::io::Error> for IndexVersionError {
    fn from(e: std::io::Error) -> Self {
        Self::Io(e)
    }
}

impl From<toml::de::Error> for IndexVersionError {
    fn from(e: toml::de::Error) -> Self {
        Self::Parse(e)
    }
}

impl From<toml::ser::Error> for IndexVersionError {
    fn from(e: toml::ser::Error) -> Self {
        Self::Serialize(e)
    }
}

// ---------------------------------------------------------------------------
// Callback for user notification
// ---------------------------------------------------------------------------

/// Called before a rebuild starts so the user is informed. Implementations
/// may log, display a UI message, etc.
pub trait RebuildNotifier {
    fn notify(&self, action: UpgradeAction, old_version: u32, new_version: u32);
}

/// Default notifier that logs via `tracing::warn`.
pub struct LogNotifier;

impl RebuildNotifier for LogNotifier {
    fn notify(&self, action: UpgradeAction, old_version: u32, new_version: u32) {
        tracing::warn!(
            old_version,
            new_version,
            action = ?action,
            "index upgrade required — old index will be archived before rebuild"
        );
    }
}

// ---------------------------------------------------------------------------
// Core functions
// ---------------------------------------------------------------------------

fn bm25_format_string() -> String {
    format!("tantivy_0.26_fmt{INDEX_FORMAT_VERSION}")
}

/// Build the default `IndexVersionFile` representing the current binary's
/// expectations.
pub fn current_version(created_at: i64) -> IndexVersionFile {
    IndexVersionFile {
        index: IndexVersionSchema {
            version: CURRENT_INDEX_VERSION,
            dkg_schema_version: CURRENT_SCHEMA_VERSION,
            vector_dimensions: VectorDimensions {
                code: CODE_VEC_DIM,
                text: TEXT_VEC_DIM,
            },
            bm25_format: bm25_format_string(),
            created_at,
        },
    }
}

/// Read `index_version.toml` from `path`. Returns `None` if the file does not
/// exist (first run). Returns `Err` on I/O errors other than not-found or on
/// corrupted TOML.
pub fn read(path: &Path) -> Result<Option<IndexVersionFile>, IndexVersionError> {
    match std::fs::read_to_string(path) {
        Ok(content) => {
            let parsed: IndexVersionFile = toml::from_str(&content)?;
            Ok(Some(parsed))
        }
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(None),
        Err(e) => Err(IndexVersionError::Io(e)),
    }
}

/// Write `index_version.toml` at `path`. Creates parent directories if needed.
pub fn write(path: &Path, version: &IndexVersionFile) -> Result<(), IndexVersionError> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let content = toml::to_string_pretty(version)?;
    std::fs::write(path, content)?;
    Ok(())
}

/// Compare the on-disk version against the current binary version and decide
/// what needs to happen.
pub fn compare(on_disk: &IndexVersionSchema) -> UpgradeAction {
    let current = current_version(0).index;

    // Schema-level changes require full rebuild.
    if on_disk.dkg_schema_version != current.dkg_schema_version
        || on_disk.vector_dimensions != current.vector_dimensions
    {
        return UpgradeAction::FullRebuild;
    }

    // BM25 format change — rebuild BM25 only.
    if on_disk.bm25_format != current.bm25_format {
        return UpgradeAction::RebuildBm25Only;
    }

    // Version bump without schema/format change — still a full rebuild
    // (conservative: the version field is the catch-all).
    if on_disk.version != current.version {
        return UpgradeAction::FullRebuild;
    }

    UpgradeAction::Reuse
}

/// Archive the project index directory before a rebuild. Renames
/// `project_dir` contents into `project_dir/index_backup_{old_version}/`.
///
/// Returns the archive path on success. If archiving fails, the error is
/// returned and the caller MUST NOT proceed with the rebuild (spec: archive
/// failure -> abort rebuild).
pub fn archive_index(
    project_dir: &Path,
    old_version: u32,
) -> Result<PathBuf, IndexVersionError> {
    let archive_name = format!("index_backup_{old_version}");
    let archive_path = project_dir.join(&archive_name);

    // Collect entries to move (skip the archive dir itself and index_version.toml).
    let entries: Vec<_> = std::fs::read_dir(project_dir)
        .map_err(|e| IndexVersionError::ArchiveFailed {
            source: e,
            message: format!("cannot read project dir {}", project_dir.display()),
        })?
        .filter_map(|entry| entry.ok())
        .filter(|entry| {
            let name = entry.file_name();
            let name_str = name.to_string_lossy();
            !name_str.starts_with("index_backup_") && name_str != "index_version.toml"
        })
        .collect();

    if entries.is_empty() {
        // Nothing to archive — not an error.
        return Ok(archive_path);
    }

    std::fs::create_dir_all(&archive_path).map_err(|e| IndexVersionError::ArchiveFailed {
        source: e,
        message: format!("cannot create archive dir {}", archive_path.display()),
    })?;

    for entry in entries {
        let src = entry.path();
        let dest = archive_path.join(entry.file_name());
        std::fs::rename(&src, &dest).map_err(|e| IndexVersionError::ArchiveFailed {
            source: e,
            message: format!(
                "cannot move {} -> {}",
                src.display(),
                dest.display()
            ),
        })?;
    }

    Ok(archive_path)
}

/// Full upgrade check: read on-disk version, compare, archive if needed,
/// notify user, write new version file. Returns the decided action.
///
/// `project_dir` is `.elai_cc/projects/{id}/`.
/// `created_at` is the current timestamp for the new version file.
pub fn check_and_upgrade(
    project_dir: &Path,
    created_at: i64,
    notifier: &dyn RebuildNotifier,
) -> Result<UpgradeAction, IndexVersionError> {
    let version_path = project_dir.join("index_version.toml");

    let on_disk = match read(&version_path)? {
        Some(f) => f,
        None => {
            // First run — create default version file.
            let new = current_version(created_at);
            write(&version_path, &new)?;
            return Ok(UpgradeAction::Reuse);
        }
    };

    let action = compare(&on_disk.index);

    if action == UpgradeAction::Reuse {
        return Ok(action);
    }

    // Notify user before any destructive operation.
    notifier.notify(action, on_disk.index.version, CURRENT_INDEX_VERSION);

    // Archive old index. If this fails, abort — do NOT proceed with rebuild.
    archive_index(project_dir, on_disk.index.version)?;

    // Write the new version file.
    let new = current_version(created_at);
    write(&version_path, &new)?;

    Ok(action)
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    fn temp_project_dir() -> (tempfile::TempDir, PathBuf) {
        let dir = tempfile::tempdir().expect("tempdir");
        let project = dir.path().join("projects").join("test-project");
        std::fs::create_dir_all(&project).expect("create project dir");
        (dir, project)
    }

    // -- ACCEPTANCE: Round-trip: write then read, all fields match --
    #[test]
    fn index_version_round_trip() {
        let (_dir, project) = temp_project_dir();
        let path = project.join("index_version.toml");

        let original = current_version(1_700_000_000);
        write(&path, &original).unwrap();
        let read_back = read(&path).unwrap().expect("file should exist");

        assert_eq!(original, read_back);
    }

    // -- ACCEPTANCE: Missing file creates default with version=1 (no panic) --
    #[test]
    fn index_version_missing_file_creates_default() {
        let (_dir, project) = temp_project_dir();

        let notifier = LogNotifier;
        let action = check_and_upgrade(&project, 1_700_000_000, &notifier).unwrap();
        assert_eq!(action, UpgradeAction::Reuse);

        let path = project.join("index_version.toml");
        let written = read(&path).unwrap().expect("should be created");
        assert_eq!(written.index.version, CURRENT_INDEX_VERSION);
        assert_eq!(written.index.dkg_schema_version, CURRENT_SCHEMA_VERSION);
        assert_eq!(written.index.vector_dimensions.code, CODE_VEC_DIM);
        assert_eq!(written.index.vector_dimensions.text, TEXT_VEC_DIM);
    }

    // -- ADVERSARIAL: Corrupted TOML triggers error (not silent reuse) --
    #[test]
    fn index_version_corrupted_toml_returns_error() {
        let (_dir, project) = temp_project_dir();
        let path = project.join("index_version.toml");
        std::fs::write(&path, "{{{{not valid toml").unwrap();

        let result = read(&path);
        assert!(result.is_err(), "corrupted TOML should return Err");
    }

    // -- ACCEPTANCE: Version mismatch triggers rebuild notification --
    #[test]
    fn index_version_mismatch_triggers_full_rebuild() {
        let (_dir, project) = temp_project_dir();
        let path = project.join("index_version.toml");

        // Write an old version with different schema version.
        let mut old = current_version(1_600_000_000);
        old.index.dkg_schema_version = 0; // old schema
        write(&path, &old).unwrap();

        let action = compare(&old.index);
        assert_eq!(action, UpgradeAction::FullRebuild);
    }

    // -- Version mismatch in vector dimensions triggers full rebuild --
    #[test]
    fn index_version_vector_dim_change_triggers_full_rebuild() {
        let mut old = current_version(1_600_000_000);
        old.index.vector_dimensions.code = 512; // different dims
        assert_eq!(compare(&old.index), UpgradeAction::FullRebuild);
    }

    // -- BM25 format change triggers BM25-only rebuild --
    #[test]
    fn index_version_bm25_format_change_triggers_bm25_rebuild() {
        let mut old = current_version(1_600_000_000);
        old.index.bm25_format = "tantivy_0.21_fmt5".to_string();
        assert_eq!(compare(&old.index), UpgradeAction::RebuildBm25Only);
    }

    // -- Matching version returns Reuse --
    #[test]
    fn index_version_matching_returns_reuse() {
        let current = current_version(1_700_000_000);
        assert_eq!(compare(&current.index), UpgradeAction::Reuse);
    }

    // -- ACCEPTANCE: Archive created before rebuild --
    #[test]
    fn index_version_archive_created_before_rebuild() {
        let (_dir, project) = temp_project_dir();

        // Create some fake index files to archive.
        std::fs::write(project.join("dkg.db"), b"fake db").unwrap();
        std::fs::create_dir_all(project.join("bm25")).unwrap();
        std::fs::write(project.join("bm25").join("data"), b"fake bm25").unwrap();

        let archive_path = archive_index(&project, 1).unwrap();
        assert!(archive_path.exists(), "archive dir should exist");
        assert!(archive_path.join("dkg.db").exists(), "dkg.db should be archived");
        assert!(archive_path.join("bm25").join("data").exists(), "bm25 data should be archived");

        // Original files should be moved (not copied).
        assert!(!project.join("dkg.db").exists(), "original dkg.db should be gone");
    }

    // -- ADVERSARIAL: Archive failure aborts rebuild --
    #[test]
    fn index_version_archive_failure_returns_error() {
        // Use a non-existent project dir to trigger read_dir failure.
        let result = archive_index(Path::new("/nonexistent/path/xyz"), 1);
        assert!(result.is_err(), "archive on non-existent dir should fail");
    }

    // -- index_version.toml is NOT archived (stays in place) --
    #[test]
    fn index_version_toml_not_archived() {
        let (_dir, project) = temp_project_dir();

        let path = project.join("index_version.toml");
        std::fs::write(&path, "placeholder").unwrap();
        std::fs::write(project.join("dkg.db"), b"fake").unwrap();

        archive_index(&project, 1).unwrap();

        // index_version.toml stays in the project dir, not moved to archive.
        assert!(path.exists(), "index_version.toml should remain in place");
    }

    // -- Full check_and_upgrade flow with schema mismatch --
    #[test]
    fn index_version_full_upgrade_flow() {
        let (_dir, project) = temp_project_dir();
        let path = project.join("index_version.toml");

        // Write old version.
        let mut old = current_version(1_600_000_000);
        old.index.dkg_schema_version = 0;
        write(&path, &old).unwrap();

        // Add a fake file to be archived.
        std::fs::write(project.join("dkg.db"), b"old data").unwrap();

        struct RecordingNotifier {
            called: std::cell::Cell<bool>,
        }
        impl RebuildNotifier for RecordingNotifier {
            fn notify(&self, action: UpgradeAction, _old: u32, _new: u32) {
                assert_eq!(action, UpgradeAction::FullRebuild);
                self.called.set(true);
            }
        }

        let notifier = RecordingNotifier {
            called: std::cell::Cell::new(false),
        };

        let action = check_and_upgrade(&project, 1_700_000_000, &notifier).unwrap();
        assert_eq!(action, UpgradeAction::FullRebuild);
        assert!(notifier.called.get(), "notifier should have been called");

        // Archive should exist (named after the old version).
        let archive = project.join(format!("index_backup_{CURRENT_INDEX_VERSION}"));
        assert!(archive.exists());
        assert!(archive.join("dkg.db").exists());

        // New version file should be written.
        let new = read(&path).unwrap().expect("new version should exist");
        assert_eq!(new.index.version, CURRENT_INDEX_VERSION);
        assert_eq!(new.index.dkg_schema_version, CURRENT_SCHEMA_VERSION);
    }

    // -- Corrupted file triggers error via check_and_upgrade --
    #[test]
    fn index_version_corrupted_file_in_upgrade_flow() {
        let (_dir, project) = temp_project_dir();
        let path = project.join("index_version.toml");
        std::fs::write(&path, "not { valid } toml [[[").unwrap();

        let notifier = LogNotifier;
        let result = check_and_upgrade(&project, 1_700_000_000, &notifier);
        assert!(result.is_err(), "corrupted file should error, not silently reuse");
    }

    // -- ACCEPTANCE: read returns None for non-existent file --
    #[test]
    fn index_version_read_nonexistent_returns_none() {
        let result = read(Path::new("/tmp/does_not_exist_xyz/index_version.toml")).unwrap();
        assert!(result.is_none());
    }

    // -- Version bump (without schema/format change) triggers full rebuild --
    #[test]
    fn index_version_version_bump_triggers_full_rebuild() {
        let mut old = current_version(1_600_000_000);
        old.index.version = 0; // older version
        assert_eq!(compare(&old.index), UpgradeAction::FullRebuild);
    }

    // -- Previous archive dirs are not re-archived --
    #[test]
    fn index_version_previous_archives_not_re_archived() {
        let (_dir, project) = temp_project_dir();

        // Create a previous archive dir and a current file.
        std::fs::create_dir_all(project.join("index_backup_0")).unwrap();
        std::fs::write(project.join("index_backup_0").join("old"), b"old").unwrap();
        std::fs::write(project.join("dkg.db"), b"current").unwrap();

        let archive = archive_index(&project, 1).unwrap();

        // index_backup_0 should NOT be inside index_backup_1.
        assert!(
            !archive.join("index_backup_0").exists(),
            "previous archive dirs should not be nested"
        );
        // But dkg.db should be.
        assert!(archive.join("dkg.db").exists());
    }
}
