//! SQLite-backed store for artifact chunks.
//!
//! One `context_{project_id}.db` per project, stored in the project's
//! `indexes/` directory. Never shares `dkg.db` or `metadata.db` (avoids
//! schema collisions — pitfall from context-package.md).
//!
//! `rusqlite::Connection` is `!Send` on Windows; wrapped in
//! `std::sync::Mutex` (NOT tokio). Async callers must use
//! `tokio::task::spawn_blocking`.
//!
//! Chunk ID generation: first 32 hex chars of
//! `blake3(artifact_name || ':' || relative_path || ':' || start_line_str)`.

use std::path::Path;
use std::sync::{Arc, Mutex};

use rusqlite::{params, Connection};
use tracing::debug;

use super::manifest::ArtifactError;
use super::types::ArtifactChunk;

// ---------------------------------------------------------------------------
// Constants
// ---------------------------------------------------------------------------

/// DB file prefix. Full filename: `context_{project_id}.db`.
pub const CONTEXT_DB_FILENAME_PREFIX: &str = "context_";

/// Hex chars used for the chunk ID (32 = 128-bit prefix of blake3).
const CHUNK_ID_HEX_LEN: usize = 32;

/// DDL applied at open time (idempotent).
const CONTEXT_STORE_DDL: &str = "
CREATE TABLE IF NOT EXISTS context_chunks (
    id                TEXT PRIMARY KEY,
    artifact_name     TEXT NOT NULL,
    artifact_description TEXT NOT NULL,
    relative_path     TEXT NOT NULL,
    content           TEXT NOT NULL,
    start_line        INTEGER NOT NULL,
    end_line          INTEGER NOT NULL,
    content_hash      TEXT NOT NULL,
    indexed_at        INTEGER NOT NULL,
    embedding         BLOB
);

CREATE INDEX IF NOT EXISTS idx_ctx_chunks_artifact
    ON context_chunks (artifact_name);

CREATE INDEX IF NOT EXISTS idx_ctx_chunks_path
    ON context_chunks (relative_path);
";

// ---------------------------------------------------------------------------
// ArtifactStore
// ---------------------------------------------------------------------------

/// Per-project store for artifact chunks.
///
/// Cheap to clone (`Arc<Mutex<Connection>>`). All methods are synchronous;
/// async callers must use `tokio::task::spawn_blocking`.
#[derive(Clone)]
pub struct ArtifactStore {
    conn: Arc<Mutex<Connection>>,
}

impl std::fmt::Debug for ArtifactStore {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ArtifactStore")
            .field("conn", &"Arc<Mutex<Connection>>")
            .finish()
    }
}

impl ArtifactStore {
    /// Open (or create) the artifact store at `db_path`. Idempotent.
    ///
    /// Call: `ArtifactStore::open(&indexes_dir.join(format!("context_{project_id}.db")))`
    pub fn open(db_path: &Path) -> Result<Self, ArtifactError> {
        if let Some(parent) = db_path.parent() {
            if !parent.as_os_str().is_empty() {
                std::fs::create_dir_all(parent)?;
            }
        }
        let conn = Connection::open(db_path)?;
        conn.pragma_update(None, "journal_mode", "WAL")?;
        conn.pragma_update(None, "foreign_keys", "ON")?;
        conn.execute_batch(CONTEXT_STORE_DDL)?;
        Ok(Self {
            conn: Arc::new(Mutex::new(conn)),
        })
    }

    fn lock(&self) -> Result<std::sync::MutexGuard<'_, Connection>, ArtifactError> {
        self.conn.lock().map_err(|_| ArtifactError::MutexPoisoned)
    }

    // -----------------------------------------------------------------------
    // Writes
    // -----------------------------------------------------------------------

    /// Upsert one chunk record. `indexed_at` is unix seconds passed in by the
    /// caller (from an injected ClockProvider).
    pub fn upsert_chunk(&self, chunk: &ArtifactChunk, indexed_at: i64) -> Result<(), ArtifactError> {
        let embedding_blob: Option<Vec<u8>> = chunk
            .embedding
            .as_ref()
            .map(|v| {
                v.iter()
                    .flat_map(|f| f.to_le_bytes())
                    .collect()
            });

        let guard = self.lock()?;
        guard.execute(
            "INSERT INTO context_chunks \
             (id, artifact_name, artifact_description, relative_path, \
              content, start_line, end_line, content_hash, indexed_at, embedding) \
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10) \
             ON CONFLICT(id) DO UPDATE SET \
               artifact_name = excluded.artifact_name, \
               artifact_description = excluded.artifact_description, \
               relative_path = excluded.relative_path, \
               content = excluded.content, \
               start_line = excluded.start_line, \
               end_line = excluded.end_line, \
               content_hash = excluded.content_hash, \
               indexed_at = excluded.indexed_at, \
               embedding = excluded.embedding",
            params![
                chunk.id,
                chunk.artifact_name,
                chunk.artifact_description,
                chunk.relative_path,
                chunk.content,
                i64::from(chunk.start_line),
                i64::from(chunk.end_line),
                chunk.content_hash,
                indexed_at,
                embedding_blob,
            ],
        )?;
        Ok(())
    }

    /// Delete all chunks for a specific file within an artifact.
    pub fn drop_chunks_for_file(
        &self,
        artifact_name: &str,
        relative_path: &str,
    ) -> Result<usize, ArtifactError> {
        let guard = self.lock()?;
        let rows = guard.execute(
            "DELETE FROM context_chunks \
             WHERE artifact_name = ?1 AND relative_path = ?2",
            params![artifact_name, relative_path],
        )?;
        Ok(rows)
    }

    /// Delete all chunks for an artifact (used before re-indexing).
    pub fn drop_chunks_for_artifact(&self, artifact_name: &str) -> Result<usize, ArtifactError> {
        let guard = self.lock()?;
        let rows = guard.execute(
            "DELETE FROM context_chunks WHERE artifact_name = ?1",
            params![artifact_name],
        )?;
        debug!(artifact = artifact_name, deleted = rows, "dropped artifact chunks");
        Ok(rows)
    }

    // -----------------------------------------------------------------------
    // Reads
    // -----------------------------------------------------------------------

    /// List all chunks for `artifact_name`, ordered by `relative_path, start_line`.
    pub fn list_artifact_chunks(
        &self,
        artifact_name: &str,
    ) -> Result<Vec<ArtifactChunk>, ArtifactError> {
        let guard = self.lock()?;
        let mut stmt = guard.prepare(
            "SELECT id, artifact_name, artifact_description, relative_path, \
                    content, start_line, end_line, content_hash, embedding \
             FROM context_chunks \
             WHERE artifact_name = ?1 \
             ORDER BY relative_path, start_line",
        )?;
        let rows = stmt.query_map(params![artifact_name], row_to_chunk)?;
        let mut out = Vec::new();
        for row in rows {
            out.push(row?);
        }
        Ok(out)
    }

    /// List all chunks, ordered by `artifact_name, relative_path, start_line`.
    /// Used for in-process cosine search (Phase A fallback).
    pub fn list_all_chunks(&self) -> Result<Vec<ArtifactChunk>, ArtifactError> {
        let guard = self.lock()?;
        let mut stmt = guard.prepare(
            "SELECT id, artifact_name, artifact_description, relative_path, \
                    content, start_line, end_line, content_hash, embedding \
             FROM context_chunks \
             ORDER BY artifact_name, relative_path, start_line",
        )?;
        let rows = stmt.query_map([], row_to_chunk)?;
        let mut out = Vec::new();
        for row in rows {
            out.push(row?);
        }
        Ok(out)
    }

    /// Count of chunks for `artifact_name` (fast — index-scan only).
    pub fn count_chunks(&self, artifact_name: &str) -> Result<usize, ArtifactError> {
        let guard = self.lock()?;
        let count: i64 = guard.query_row(
            "SELECT COUNT(*) FROM context_chunks WHERE artifact_name = ?1",
            params![artifact_name],
            |r| r.get(0),
        )?;
        let count = usize::try_from(count).unwrap_or(0);
        Ok(count)
    }

    /// Whether any chunks exist (used by artifact_search to short-circuit).
    pub fn is_empty(&self) -> Result<bool, ArtifactError> {
        let guard = self.lock()?;
        let count: i64 = guard.query_row(
            "SELECT COUNT(*) FROM context_chunks",
            [],
            |r| r.get(0),
        )?;
        Ok(count == 0)
    }
}

// ---------------------------------------------------------------------------
// Free helpers
// ---------------------------------------------------------------------------

/// Compute the chunk ID from `(artifact_name, relative_path, start_line)`.
///
/// Returns the first [`CHUNK_ID_HEX_LEN`] hex chars of the blake3 digest.
pub fn chunk_id(artifact_name: &str, relative_path: &str, start_line: u32) -> String {
    let input = format!("{artifact_name}:{relative_path}:{start_line}");
    let hash = blake3::hash(input.as_bytes());
    let hex = hash.to_hex();
    hex.as_str()
        .chars()
        .take(CHUNK_ID_HEX_LEN)
        .collect()
}

/// Deserialize a BLOB column (little-endian f32 sequence) into `Vec<f32>`.
fn blob_to_embedding(blob: &[u8]) -> Option<Vec<f32>> {
    if blob.len() % 4 != 0 {
        return None;
    }
    Some(
        blob.chunks_exact(4)
            .map(|b| f32::from_le_bytes([b[0], b[1], b[2], b[3]]))
            .collect(),
    )
}

fn row_to_chunk(
    r: &rusqlite::Row<'_>,
) -> rusqlite::Result<ArtifactChunk> {
    let id: String = r.get(0)?;
    let artifact_name: String = r.get(1)?;
    let artifact_description: String = r.get(2)?;
    let relative_path: String = r.get(3)?;
    let content: String = r.get(4)?;
    let start_line: i64 = r.get(5)?;
    let end_line: i64 = r.get(6)?;
    let content_hash: String = r.get(7)?;
    let embedding_blob: Option<Vec<u8>> = r.get(8)?;

    let embedding = embedding_blob
        .as_deref()
        .and_then(blob_to_embedding);

    let start_line = u32::try_from(start_line).unwrap_or(0);
    let end_line = u32::try_from(end_line).unwrap_or(0);

    Ok(ArtifactChunk {
        id,
        artifact_name,
        artifact_description,
        relative_path,
        content,
        start_line,
        end_line,
        content_hash,
        embedding,
    })
}
