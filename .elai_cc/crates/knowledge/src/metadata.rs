//! B-31 Phase 1: per-project indexing metadata store.
//!
//! [`ProjectMetadataStore`] persists indexing progress and per-file content
//! hashes in a SQLite database alongside `dkg.db`. It is the authoritative
//! resume cursor for the BM25 + DKG pipeline; see
//! `workflow/reasoning/B-31-decisions.md` for the locked design choices.
//!
//! Design notes
//! ============
//! * `schema_version` on `project_metadata` is a NEW lineage starting at 1
//!   (DECISION-2). It is independent of `dkg_meta.schema_version` (currently
//!   3 in [`crate::dkg`]) so migrations on either side stay decoupled.
//! * The DDL lives in [`MIGRATION_V2_SQL`] (embedded via `include_str!`) and is
//!   idempotent — `open` may be called repeatedly on the same `db_path`
//!   without error.
//! * `rusqlite::Connection` is `!Send` on Windows (`systemPatterns.md:57`).
//!   We wrap it in `std::sync::Mutex` (NOT tokio's). Methods stay synchronous;
//!   async callers must invoke them from `tokio::task::spawn_blocking`.
//! * All timestamps come from an injected [`ClockProvider`] (B-19 entropy
//!   quarantine, mirroring `orchestrator::artifacts::ArtifactStore`). Tests
//!   pass `SeededClock`; production passes `OsClock`.
//! * `content_hash` stores a BLAKE3 hex digest (64 chars). BLAKE3 is the
//!   workspace canonical hash (CLAUDE.md anti-stale rule). It matches the
//!   format used by [`crate::index_manifest::IndexFileEntry::content_blake3`]
//!   for cross-store compatibility.
//! * Cancellation flags are NOT stored here — those live in-process on the
//!   indexing service (`Arc<AtomicBool>` per project). The metadata store
//!   only persists the *result* of cancellation via `IndexingStatus::Cancelled`.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::time::UNIX_EPOCH;

use elai_core::ClockProvider;
use rusqlite::{params, Connection, OptionalExtension};
use serde::{Deserialize, Serialize};
use thiserror::Error;

// ---------------------------------------------------------------------------
// Named constants (no magic numbers)
// ---------------------------------------------------------------------------

/// Current `project_metadata.schema_version` value. Bump when introducing a
/// non-backwards-compatible change to the metadata tables.
pub const METADATA_SCHEMA_VERSION: u32 = 1;

/// Default number of files per resumable indexing batch. Mirrors
/// `INDEX_BATCH_SIZE` from the SocratiCode reference indexer
/// (`docs/.source/SocratiCode-main/src/services/indexer.ts:801`).
pub const DEFAULT_BATCH_SIZE: u32 = 50;

/// Standard filename for the per-project metadata SQLite database. Lives
/// inside the project's indexes directory alongside `dkg.db`.
pub const METADATA_DB_FILENAME: &str = "metadata.db";

/// Number of seconds a checkpoint can be inactive before another process may
/// consider an in-progress run stale (Phase 3 wiring). Defined now so both
/// cycles share the same constant. Mirrors `STALE_CHECKPOINT_TIMEOUT_MS` /
/// 1000 from `indexer.ts`.
pub const STALE_CHECKPOINT_TIMEOUT_SECS: i64 = 120;

/// Number of hex characters of a BLAKE3 digest of the canonical project path
/// used as a stable, deterministic `project_id`. 16 hex = 64 bits — enough
/// entropy to make collisions astronomically unlikely while keeping the value
/// short enough to ship in log lines and SSB payloads.
const PROJECT_ID_HEX_LEN: usize = 16;

/// DDL applied at construction time. Idempotent (`CREATE TABLE IF NOT EXISTS`).
const MIGRATION_V2_SQL: &str = include_str!("metadata_migration_v2.sql");

// ---------------------------------------------------------------------------
// Status enum
// ---------------------------------------------------------------------------

/// Persisted indexing status for a project. Maps 1:1 to the
/// `indexing_status` CHECK constraint in `project_metadata`. The enum is
/// deliberately closed — extending the value set is destructive at the
/// SQLite layer (DROP + recreate, gotchas.md:391).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum IndexingStatus {
    /// Indexing is mid-flight; resume-on-restart applies.
    InProgress,
    /// Indexing completed successfully.
    Completed,
    /// Indexing stopped early via cooperative cancellation.
    Cancelled,
}

impl IndexingStatus {
    /// SQL representation. Stable across versions (the CHECK constraint
    /// freezes the exact spelling).
    pub fn as_sql_str(self) -> &'static str {
        match self {
            Self::InProgress => "in-progress",
            Self::Completed => "completed",
            Self::Cancelled => "cancelled",
        }
    }

    /// Parse from the SQL value. Returns [`MetadataError::InvalidStatus`] on
    /// any value not in the CHECK list.
    pub fn from_sql_str(s: &str) -> Result<Self, MetadataError> {
        match s {
            "in-progress" => Ok(Self::InProgress),
            "completed" => Ok(Self::Completed),
            "cancelled" => Ok(Self::Cancelled),
            other => Err(MetadataError::InvalidStatus(other.to_string())),
        }
    }
}

// ---------------------------------------------------------------------------
// Error type
// ---------------------------------------------------------------------------

/// Errors emitted by [`ProjectMetadataStore`].
#[derive(Debug, Error)]
pub enum MetadataError {
    /// Wraps any `rusqlite` failure (open, prepare, execute, query).
    #[error("metadata sqlite error: {0}")]
    Sqlite(#[from] rusqlite::Error),

    /// The injected `ClockProvider` reported a pre-epoch time. Vanishingly
    /// unlikely on real hosts; propagated rather than panicked.
    #[error("metadata clock error: {0}")]
    Clock(String),

    /// `indexing_status` row contained a value outside the CHECK list. Should
    /// only ever fire if the DB was hand-edited.
    #[error("invalid indexing status: {0}")]
    InvalidStatus(String),

    /// The internal `Mutex<Connection>` was poisoned (a prior call panicked
    /// while holding the lock). Treated as a fatal store-level error.
    #[error("metadata connection mutex was poisoned")]
    MutexPoisoned,

    /// `relative_path` was non-UTF-8 — SQLite stores TEXT, not arbitrary
    /// bytes. Surface a clear error rather than mojibake.
    #[error("metadata path is not valid UTF-8: {0}")]
    InvalidPath(String),

    /// Stored `content_hash` was not valid BLAKE3 hex.
    #[error("metadata content_hash is malformed: {0}")]
    InvalidHash(String),

    /// Filesystem-level failure when creating the DB directory or opening
    /// the SQLite file. Distinct from [`MetadataError::Sqlite`] which only
    /// covers errors emitted by `rusqlite` itself.
    #[error("metadata filesystem error: {0}")]
    Io(#[from] std::io::Error),
}

// ---------------------------------------------------------------------------
// ProjectMetadata value object
// ---------------------------------------------------------------------------

/// In-memory view of a `project_metadata` row.
///
/// `Serialize`/`Deserialize` derived for B-31 Phase 4 (Cycle B) so the
/// dashboard's `get_indexing_progress` Tauri command can return a row
/// directly to the Svelte frontend. The derive is purely additive — no
/// SQLite column or rusqlite call site changes.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProjectMetadata {
    /// Stable BLAKE3-derived identifier (see [`project_id_from_path`]).
    pub project_id: String,
    /// Canonical project root path as stored in `project_path`.
    pub project_path: PathBuf,
    /// Number of files for which a hash row has been written so far.
    pub indexed_files: u64,
    /// Total files the current run is expected to process.
    pub total_files: u64,
    /// Indexing lifecycle state.
    pub indexing_status: IndexingStatus,
    /// Unix epoch seconds of the most recent checkpoint write (clock-provided).
    pub last_checkpoint_at: i64,
    /// Number of files per batch in the active run.
    pub batch_size: u32,
    /// Number of batches already checkpointed.
    pub batches_processed: u32,
    /// Total batches the run will perform (`ceil(total_files / batch_size)`).
    pub batches_total: Option<u32>,
    /// Schema lineage for this row. Currently always [`METADATA_SCHEMA_VERSION`].
    pub schema_version: u32,
}

// ---------------------------------------------------------------------------
// ProjectMetadataStore
// ---------------------------------------------------------------------------

/// SQLite-backed persistent store for indexing metadata.
///
/// Cheaply cloneable via `Arc<Mutex<Connection>>`. All methods are
/// synchronous (rusqlite has no async API); callers in async contexts should
/// invoke them under `tokio::task::spawn_blocking`.
#[derive(Clone)]
pub struct ProjectMetadataStore {
    conn: Arc<Mutex<Connection>>,
    clock: Arc<dyn ClockProvider>,
}

impl std::fmt::Debug for ProjectMetadataStore {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ProjectMetadataStore")
            .field("conn", &"Arc<Mutex<Connection>>")
            .field("clock", &"Arc<dyn ClockProvider>")
            .finish()
    }
}

impl ProjectMetadataStore {
    /// Open (or create) the metadata DB at `db_path`. Idempotent: applies
    /// `MIGRATION_V2_SQL` (which uses `IF NOT EXISTS` clauses) every call.
    ///
    /// Creates the parent directory if missing.
    pub fn open(
        db_path: &Path,
        clock: Arc<dyn ClockProvider>,
    ) -> Result<Self, MetadataError> {
        if let Some(parent) = db_path.parent() {
            if !parent.as_os_str().is_empty() {
                std::fs::create_dir_all(parent)?;
            }
        }
        let conn = Connection::open(db_path)?;
        // WAL keeps reads non-blocking when an indexing run is mid-flight.
        // Idempotent — sqlite returns the same row no matter how often we set it.
        conn.pragma_update(None, "journal_mode", "WAL")?;
        conn.pragma_update(None, "foreign_keys", "ON")?;
        conn.execute_batch(MIGRATION_V2_SQL)?;
        Ok(Self {
            conn: Arc::new(Mutex::new(conn)),
            clock,
        })
    }

    /// Acquire the internal mutex guard, mapping poison to a typed error.
    fn lock(&self) -> Result<std::sync::MutexGuard<'_, Connection>, MetadataError> {
        self.conn.lock().map_err(|_| MetadataError::MutexPoisoned)
    }

    /// Load the `project_metadata` row for `project_id`, if any.
    pub fn load(
        &self,
        project_id: &str,
    ) -> Result<Option<ProjectMetadata>, MetadataError> {
        let guard = self.lock()?;
        let row = guard
            .query_row(
                "SELECT project_id, project_path, indexed_files, total_files, \
                        indexing_status, last_checkpoint_at, batch_size, \
                        batches_processed, batches_total, schema_version \
                 FROM project_metadata WHERE project_id = ?1",
                params![project_id],
                |r| {
                    let project_id: String = r.get(0)?;
                    let project_path: String = r.get(1)?;
                    let indexed_files: i64 = r.get(2)?;
                    let total_files: i64 = r.get(3)?;
                    let status: String = r.get(4)?;
                    let last_checkpoint_at: i64 = r.get(5)?;
                    let batch_size: i64 = r.get(6)?;
                    let batches_processed: i64 = r.get(7)?;
                    let batches_total: Option<i64> = r.get(8)?;
                    let schema_version: i64 = r.get(9)?;
                    Ok((
                        project_id,
                        project_path,
                        indexed_files,
                        total_files,
                        status,
                        last_checkpoint_at,
                        batch_size,
                        batches_processed,
                        batches_total,
                        schema_version,
                    ))
                },
            )
            .optional()?;

        let Some((
            project_id,
            project_path,
            indexed_files,
            total_files,
            status,
            last_checkpoint_at,
            batch_size,
            batches_processed,
            batches_total,
            schema_version,
        )) = row
        else {
            return Ok(None);
        };

        let indexed_files: u64 = indexed_files.try_into().unwrap_or(0);
        let total_files: u64 = total_files.try_into().unwrap_or(0);
        let batch_size: u32 = batch_size.try_into().unwrap_or(DEFAULT_BATCH_SIZE);
        let batches_processed: u32 = batches_processed.try_into().unwrap_or(0);
        let batches_total: Option<u32> = batches_total.and_then(|v| u32::try_from(v).ok());
        let schema_version: u32 = schema_version.try_into().unwrap_or(METADATA_SCHEMA_VERSION);

        Ok(Some(ProjectMetadata {
            project_id,
            project_path: PathBuf::from(project_path),
            indexed_files,
            total_files,
            indexing_status: IndexingStatus::from_sql_str(&status)?,
            last_checkpoint_at,
            batch_size,
            batches_processed,
            batches_total,
            schema_version,
        }))
    }

    /// Upsert a `project_metadata` row. Idempotent — repeated calls update
    /// the fields in place. `schema_version` is always written as
    /// [`METADATA_SCHEMA_VERSION`] so older rows get migrated forward
    /// automatically on next save.
    pub fn save(
        &self,
        metadata: &ProjectMetadata,
    ) -> Result<(), MetadataError> {
        let project_path_str = path_to_utf8(&metadata.project_path)?;
        let batches_total = metadata
            .batches_total
            .map(i64::from);
        let indexed_files = i64::try_from(metadata.indexed_files).unwrap_or(i64::MAX);
        let total_files = i64::try_from(metadata.total_files).unwrap_or(i64::MAX);

        let guard = self.lock()?;
        guard.execute(
            "INSERT INTO project_metadata (\
                project_id, project_path, indexed_files, total_files, \
                indexing_status, last_checkpoint_at, batch_size, \
                batches_processed, batches_total, schema_version\
             ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10) \
             ON CONFLICT(project_id) DO UPDATE SET \
                project_path = excluded.project_path, \
                indexed_files = excluded.indexed_files, \
                total_files = excluded.total_files, \
                indexing_status = excluded.indexing_status, \
                last_checkpoint_at = excluded.last_checkpoint_at, \
                batch_size = excluded.batch_size, \
                batches_processed = excluded.batches_processed, \
                batches_total = excluded.batches_total, \
                schema_version = excluded.schema_version",
            params![
                metadata.project_id,
                project_path_str,
                indexed_files,
                total_files,
                metadata.indexing_status.as_sql_str(),
                metadata.last_checkpoint_at,
                i64::from(metadata.batch_size),
                i64::from(metadata.batches_processed),
                batches_total,
                i64::from(METADATA_SCHEMA_VERSION),
            ],
        )?;
        Ok(())
    }

    /// Update only the `indexing_status` (and bump `last_checkpoint_at`).
    /// Idempotent. Returns the number of rows updated (0 if the project has
    /// no row yet).
    pub fn update_status(
        &self,
        project_id: &str,
        status: IndexingStatus,
    ) -> Result<usize, MetadataError> {
        let now = self.clock_unix_secs()?;
        let guard = self.lock()?;
        let rows = guard.execute(
            "UPDATE project_metadata \
             SET indexing_status = ?1, last_checkpoint_at = ?2 \
             WHERE project_id = ?3",
            params![status.as_sql_str(), now, project_id],
        )?;
        Ok(rows)
    }

    /// Load every `file_hashes` row for `project_id` into a deterministic
    /// `BTreeMap` (sorted by path) for resume-comparison.
    pub fn get_file_hashes(
        &self,
        project_id: &str,
    ) -> Result<BTreeMap<PathBuf, blake3::Hash>, MetadataError> {
        let guard = self.lock()?;
        let mut stmt = guard.prepare(
            "SELECT relative_path, content_hash FROM file_hashes \
             WHERE project_id = ?1",
        )?;
        let rows = stmt.query_map(params![project_id], |r| {
            let rel: String = r.get(0)?;
            let hash: String = r.get(1)?;
            Ok((rel, hash))
        })?;

        let mut out = BTreeMap::new();
        for row in rows {
            let (rel, hash_hex) = row?;
            let hash = parse_blake3_hex(&hash_hex)?;
            out.insert(PathBuf::from(rel), hash);
        }
        Ok(out)
    }

    /// Idempotent upsert of a single `file_hashes` row.
    /// `last_indexed_at` is sourced from the injected clock.
    pub fn set_file_hash(
        &self,
        project_id: &str,
        rel_path: &Path,
        hash: blake3::Hash,
    ) -> Result<(), MetadataError> {
        let rel_str = path_to_utf8(rel_path)?;
        let hash_hex = hash.to_hex().to_string();
        let now = self.clock_unix_secs()?;
        let guard = self.lock()?;
        guard.execute(
            "INSERT INTO file_hashes (project_id, relative_path, content_hash, last_indexed_at) \
             VALUES (?1, ?2, ?3, ?4) \
             ON CONFLICT(project_id, relative_path) DO UPDATE SET \
                content_hash = excluded.content_hash, \
                last_indexed_at = excluded.last_indexed_at",
            params![project_id, rel_str, hash_hex, now],
        )?;
        Ok(())
    }

    /// Remove a single `file_hashes` row. Returns rows deleted (0 if absent).
    pub fn delete_file_hash(
        &self,
        project_id: &str,
        rel_path: &Path,
    ) -> Result<usize, MetadataError> {
        let rel_str = path_to_utf8(rel_path)?;
        let guard = self.lock()?;
        let rows = guard.execute(
            "DELETE FROM file_hashes WHERE project_id = ?1 AND relative_path = ?2",
            params![project_id, rel_str],
        )?;
        Ok(rows)
    }

    /// Cascade-delete a project's `project_metadata` row and all its
    /// `file_hashes`. Runs in a single transaction so a partial failure
    /// leaves the DB untouched.
    pub fn delete_project(
        &self,
        project_id: &str,
    ) -> Result<(), MetadataError> {
        let mut guard = self.lock()?;
        let tx = guard.transaction()?;
        tx.execute(
            "DELETE FROM file_hashes WHERE project_id = ?1",
            params![project_id],
        )?;
        tx.execute(
            "DELETE FROM project_metadata WHERE project_id = ?1",
            params![project_id],
        )?;
        tx.commit()?;
        Ok(())
    }

    /// Enumerate every `project_id` known to the store. Deterministic order
    /// (alphabetical on the TEXT primary key).
    pub fn list_project_ids(&self) -> Result<Vec<String>, MetadataError> {
        let guard = self.lock()?;
        let mut stmt = guard
            .prepare("SELECT project_id FROM project_metadata ORDER BY project_id ASC")?;
        let rows = stmt.query_map([], |r| r.get::<_, String>(0))?;
        let mut out = Vec::new();
        for row in rows {
            out.push(row?);
        }
        Ok(out)
    }

    /// Convert the clock's `system_now` into unix seconds. Surfaces a typed
    /// error rather than panicking on pre-epoch values.
    fn clock_unix_secs(&self) -> Result<i64, MetadataError> {
        let now = self.clock.system_now();
        let dur = now
            .duration_since(UNIX_EPOCH)
            .map_err(|e| MetadataError::Clock(e.to_string()))?;
        i64::try_from(dur.as_secs()).map_err(|_| MetadataError::Clock("epoch overflow".into()))
    }
}

// ---------------------------------------------------------------------------
// Free helpers
// ---------------------------------------------------------------------------

/// Derive a stable `project_id` from a canonical project path.
///
/// Mirrors SocratiCode's `projectIdFromPath` but uses BLAKE3 (workspace
/// canonical hash, CLAUDE.md anti-stale) instead of SHA-256. On Windows the
/// path is lowercased before hashing so case-insensitive filesystems produce
/// the same `project_id` regardless of how the user spelled the drive letter
/// or directory.
pub fn project_id_from_path(canonical_path: &Path) -> String {
    let normalised: String = if cfg!(windows) {
        canonical_path.to_string_lossy().to_lowercase()
    } else {
        canonical_path.to_string_lossy().to_string()
    };
    let hex = blake3::hash(normalised.as_bytes()).to_hex();
    hex.as_str()
        .chars()
        .take(PROJECT_ID_HEX_LEN)
        .collect()
}

fn path_to_utf8(p: &Path) -> Result<String, MetadataError> {
    match p.to_str() {
        Some(s) => Ok(s.to_string()),
        None => Err(MetadataError::InvalidPath(p.to_string_lossy().to_string())),
    }
}

fn parse_blake3_hex(hex: &str) -> Result<blake3::Hash, MetadataError> {
    blake3::Hash::from_hex(hex).map_err(|e| MetadataError::InvalidHash(e.to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use elai_core::clock::SeededClock;

    fn seeded() -> Arc<dyn ClockProvider> {
        Arc::new(SeededClock::new())
    }

    #[test]
    fn project_id_is_deterministic() {
        let p = Path::new("/tmp/elai-test-project");
        let a = project_id_from_path(p);
        let b = project_id_from_path(p);
        assert_eq!(a, b);
        assert_eq!(a.len(), PROJECT_ID_HEX_LEN);
    }

    #[test]
    fn project_id_collapses_windows_case() {
        if !cfg!(windows) {
            return;
        }
        let a = project_id_from_path(Path::new("D:/Foo/Bar"));
        let b = project_id_from_path(Path::new("d:/foo/bar"));
        assert_eq!(a, b);
    }

    #[test]
    fn open_in_memory_is_idempotent() {
        // Two opens of the same path would require a tempfile, but the
        // migrations themselves must be idempotent — exercise that on a
        // single in-memory connection.
        let conn = Connection::open_in_memory().unwrap();
        conn.execute_batch(MIGRATION_V2_SQL).unwrap();
        conn.execute_batch(MIGRATION_V2_SQL).unwrap();
    }

    #[test]
    fn indexing_status_roundtrips() {
        for status in [
            IndexingStatus::InProgress,
            IndexingStatus::Completed,
            IndexingStatus::Cancelled,
        ] {
            let s = status.as_sql_str();
            assert_eq!(IndexingStatus::from_sql_str(s).unwrap(), status);
        }
        assert!(IndexingStatus::from_sql_str("bogus").is_err());
    }

    #[test]
    fn save_and_load_roundtrip_in_memory() {
        let tmp = tempfile::tempdir().unwrap();
        let db_path = tmp.path().join("metadata.db");
        let store = ProjectMetadataStore::open(&db_path, seeded()).unwrap();
        let meta = ProjectMetadata {
            project_id: "pid-aaaa".to_string(),
            project_path: PathBuf::from("/tmp/proj"),
            indexed_files: 7,
            total_files: 10,
            indexing_status: IndexingStatus::InProgress,
            last_checkpoint_at: 1_700_000_000,
            batch_size: DEFAULT_BATCH_SIZE,
            batches_processed: 2,
            batches_total: Some(5),
            schema_version: METADATA_SCHEMA_VERSION,
        };
        store.save(&meta).unwrap();
        let loaded = store.load("pid-aaaa").unwrap().unwrap();
        assert_eq!(loaded, meta);
    }
}
