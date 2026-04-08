use std::collections::HashSet;
use std::fmt;
use std::path::Path;

use rusqlite::{params, Connection, OpenFlags};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use elai_core::clock::ClockProvider;
use elai_core::entropy::EntropySource;

pub const SCHEMA_VERSION: u32 = 1;
pub const MAX_ACTIVE_BRANCHES: usize = 3;
pub const MAX_PARENT_DEPTH: usize = 1000;

// ---------------------------------------------------------------------------
// Error type
// ---------------------------------------------------------------------------

#[derive(Debug, thiserror::Error)]
pub enum SessionDagError {
    #[error("database not found at path")]
    DatabaseNotFound,
    #[error("database failed integrity check")]
    DatabaseCorrupted,
    #[error("schema version mismatch: found {found}, expected {expected}")]
    SchemaVersion { found: u32, expected: u32 },
    #[error("foreign key violation")]
    ForeignKeyViolation,
    #[error("cycle detected in snapshot DAG")]
    CycleDetected,
    #[error("branch limit exceeded: max {max}")]
    BranchLimitExceeded { max: usize },
    #[error("branch not found")]
    BranchNotFound,
    #[error("invalid branch name")]
    InvalidBranchName,
    #[error("snapshot not found")]
    SnapshotNotFound,
    #[error("sqlite error: {0}")]
    Sqlite(#[from] rusqlite::Error),
}

// ---------------------------------------------------------------------------
// Enums
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DagMessageRole {
    System,
    User,
    Assistant,
    Tool,
}

impl fmt::Display for DagMessageRole {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::System => f.write_str("system"),
            Self::User => f.write_str("user"),
            Self::Assistant => f.write_str("assistant"),
            Self::Tool => f.write_str("tool"),
        }
    }
}

impl DagMessageRole {
    pub fn from_str_checked(s: &str) -> Option<Self> {
        match s {
            "system" => Some(Self::System),
            "user" => Some(Self::User),
            "assistant" => Some(Self::Assistant),
            "tool" => Some(Self::Tool),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum MergeResult {
    Winner {
        branch: BranchRef,
        abandoned: Vec<BranchRef>,
    },
    Conflict {
        reason: String,
    },
}

// ---------------------------------------------------------------------------
// Structs
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Snapshot {
    pub snap_id: String,
    pub parent_id: Option<String>,
    pub session_id: String,
    pub fsm_state: String,
    pub phase: Option<String>,
    pub step: Option<String>,
    pub findings: Option<Vec<u8>>,
    pub code_changes: Option<Vec<u8>>,
    pub tests_written: Option<Vec<u8>>,
    pub commit_hashes: Vec<String>,
    pub created_at: u64,
    pub tag: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TrimmedMessage {
    pub seq: u32,
    pub role: DagMessageRole,
    pub content: Vec<u8>,
    pub is_stubbed: bool,
    pub stub_ssb_seq: Option<u64>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Stub {
    pub original_type: String,
    pub original_tokens: usize,
    pub summary_hash: [u8; 32],
    pub ssb_seq: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BranchRef {
    pub name: String,
    pub tip_snap_id: String,
    pub created_at: u64,
}

// ---------------------------------------------------------------------------
// SQL DDL
// ---------------------------------------------------------------------------

const CREATE_SCHEMA_VERSION_TABLE: &str = "\
CREATE TABLE IF NOT EXISTS schema_version (
    version INTEGER PRIMARY KEY
)";

const CREATE_SNAPSHOTS_TABLE: &str = "\
CREATE TABLE IF NOT EXISTS snapshots (
    snap_id       TEXT PRIMARY KEY,
    parent_id     TEXT REFERENCES snapshots(snap_id),
    session_id    TEXT NOT NULL,
    fsm_state     TEXT NOT NULL,
    phase         TEXT,
    step          TEXT,
    findings      BLOB,
    code_changes  BLOB,
    tests_written BLOB,
    commit_hashes TEXT,
    created_at    INTEGER NOT NULL,
    tag           TEXT
)";

const CREATE_SNAPSHOT_CONTEXT_TABLE: &str = "\
CREATE TABLE IF NOT EXISTS snapshot_context (
    snap_id      TEXT NOT NULL REFERENCES snapshots(snap_id) ON DELETE CASCADE,
    seq          INTEGER NOT NULL,
    role         TEXT NOT NULL CHECK(role IN ('system','user','assistant','tool')),
    content      BLOB NOT NULL,
    is_stubbed   INTEGER NOT NULL DEFAULT 0,
    stub_ssb_seq INTEGER,
    PRIMARY KEY (snap_id, seq)
)";

const CREATE_INDEXES: &str = "\
CREATE INDEX IF NOT EXISTS idx_snapshots_session ON snapshots(session_id);
CREATE INDEX IF NOT EXISTS idx_snapshots_tag ON snapshots(tag) WHERE tag IS NOT NULL;
CREATE INDEX IF NOT EXISTS idx_context_stubbed ON snapshot_context(snap_id) WHERE is_stubbed = 1";

// ---------------------------------------------------------------------------
// SessionDag
// ---------------------------------------------------------------------------

fn nonneg_u64(val: i64) -> Result<u64, SessionDagError> {
    u64::try_from(val).map_err(|_| SessionDagError::DatabaseCorrupted)
}

fn safe_i64(val: u64) -> Result<i64, SessionDagError> {
    i64::try_from(val).map_err(|_| SessionDagError::DatabaseCorrupted)
}

pub struct SessionDag {
    db: Connection,
    session_id: String,
    #[allow(dead_code)] // populated by 3H.4 branch/merge
    active_branches: Vec<BranchRef>,
}

impl SessionDag {
    pub fn new(
        session_id: &str,
        path: &Path,
        _entropy: &mut impl EntropySource,
        _clock: &impl ClockProvider,
    ) -> Result<Self, SessionDagError> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).map_err(|e| {
                SessionDagError::Sqlite(rusqlite::Error::SqliteFailure(
                    rusqlite::ffi::Error::new(rusqlite::ffi::SQLITE_CANTOPEN),
                    Some(e.to_string()),
                ))
            })?;
        }

        let db = Connection::open(path)?;

        db.execute_batch("PRAGMA foreign_keys = ON")?;
        db.execute_batch("PRAGMA journal_mode = DELETE")?;

        db.execute_batch("BEGIN IMMEDIATE")?;
        let ddl_result = (|| -> Result<(), SessionDagError> {
            db.execute_batch(CREATE_SCHEMA_VERSION_TABLE)?;
            db.execute_batch(CREATE_SNAPSHOTS_TABLE)?;
            db.execute_batch(CREATE_SNAPSHOT_CONTEXT_TABLE)?;
            db.execute_batch(CREATE_INDEXES)?;
            db.execute(
                "INSERT OR IGNORE INTO schema_version (version) VALUES (?1)",
                params![SCHEMA_VERSION],
            )?;
            Ok(())
        })();
        if let Err(e) = ddl_result {
            let _ = db.execute_batch("ROLLBACK");
            return Err(e);
        }
        if let Err(e) = db.execute_batch("COMMIT") {
            let _ = db.execute_batch("ROLLBACK");
            return Err(SessionDagError::Sqlite(e));
        }

        Ok(Self {
            db,
            session_id: session_id.to_owned(),
            active_branches: Vec::new(),
        })
    }

    pub fn open(session_id: &str, path: &Path) -> Result<Self, SessionDagError> {
        let flags = OpenFlags::SQLITE_OPEN_READ_WRITE;
        let db = Connection::open_with_flags(path, flags).map_err(|e| {
            if let rusqlite::Error::SqliteFailure(ref ffi_err, _) = e {
                if ffi_err.code == rusqlite::ErrorCode::CannotOpen {
                    return SessionDagError::DatabaseNotFound;
                }
            }
            SessionDagError::Sqlite(e)
        })?;

        db.execute_batch("PRAGMA foreign_keys = ON")?;
        db.execute_batch("PRAGMA journal_mode = DELETE")?;

        let version: u32 = db
            .query_row(
                "SELECT version FROM schema_version LIMIT 1",
                [],
                |row| row.get(0),
            )
            .map_err(|_| SessionDagError::DatabaseCorrupted)?;

        if version != SCHEMA_VERSION {
            return Err(SessionDagError::SchemaVersion {
                found: version,
                expected: SCHEMA_VERSION,
            });
        }

        let integrity: String =
            db.query_row("PRAGMA integrity_check", [], |row| row.get(0))?;
        if integrity != "ok" {
            return Err(SessionDagError::DatabaseCorrupted);
        }

        Ok(Self {
            db,
            session_id: session_id.to_owned(),
            active_branches: Vec::new(),
        })
    }

    pub(crate) fn db(&self) -> &Connection {
        &self.db
    }

    pub fn session_id(&self) -> &str {
        &self.session_id
    }

    pub fn active_branches(&self) -> &[BranchRef] {
        &self.active_branches
    }

    pub fn generate_snap_id(entropy: &mut impl EntropySource) -> String {
        let mut bytes = [0u8; 16];
        entropy.fill_bytes(&mut bytes);
        bytes[6] = (bytes[6] & 0x0f) | 0x40;
        bytes[8] = (bytes[8] & 0x3f) | 0x80;
        Uuid::from_bytes(bytes).to_string()
    }

    // ------------------------------------------------------------------
    // Snapshot I/O (3H.2)
    // ------------------------------------------------------------------

    /// Atomically persists a snapshot and its context messages.
    pub fn save_snapshot(
        &self,
        snapshot: &Snapshot,
        messages: &[TrimmedMessage],
    ) -> Result<(), SessionDagError> {
        // Pre-validate all conversions before opening a transaction
        let commit_hashes_json = serde_json::to_string(&snapshot.commit_hashes)
            .map_err(|_| SessionDagError::DatabaseCorrupted)?;
        let created_at_i64 = safe_i64(snapshot.created_at)?;
        let msg_ssb_seqs: Vec<Option<i64>> = messages
            .iter()
            .map(|m| m.stub_ssb_seq.map(safe_i64).transpose())
            .collect::<Result<_, _>>()?;

        self.db.execute_batch("BEGIN IMMEDIATE")?;

        let txn_result = (|| -> Result<(), SessionDagError> {
            self.db.execute(
                "INSERT INTO snapshots (snap_id, parent_id, session_id, fsm_state, phase, step, \
                 findings, code_changes, tests_written, commit_hashes, created_at, tag) \
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12)",
                params![
                    snapshot.snap_id,
                    snapshot.parent_id,
                    snapshot.session_id,
                    snapshot.fsm_state,
                    snapshot.phase,
                    snapshot.step,
                    snapshot.findings,
                    snapshot.code_changes,
                    snapshot.tests_written,
                    commit_hashes_json,
                    created_at_i64,
                    snapshot.tag,
                ],
            )?;

            let mut stmt = self.db.prepare_cached(
                "INSERT INTO snapshot_context (snap_id, seq, role, content, is_stubbed, stub_ssb_seq) \
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            )?;

            for (msg, ssb_seq) in messages.iter().zip(msg_ssb_seqs.iter()) {
                stmt.execute(params![
                    snapshot.snap_id,
                    msg.seq,
                    msg.role.to_string(),
                    msg.content,
                    msg.is_stubbed as i64,
                    ssb_seq,
                ])?;
            }
            Ok(())
        })();

        if let Err(e) = txn_result {
            let _ = self.db.execute_batch("ROLLBACK");
            return Err(e);
        }

        if let Err(e) = self.db.execute_batch("COMMIT") {
            let _ = self.db.execute_batch("ROLLBACK");
            return Err(SessionDagError::Sqlite(e));
        }
        Ok(())
    }

    /// Loads a snapshot and its context messages (ordered by seq ASC).
    pub fn load_snapshot(
        &self,
        snap_id: &str,
    ) -> Result<(Snapshot, Vec<TrimmedMessage>), SessionDagError> {
        let snapshot = self.db.query_row(
            "SELECT snap_id, parent_id, session_id, fsm_state, phase, step, \
             findings, code_changes, tests_written, commit_hashes, created_at, tag \
             FROM snapshots WHERE snap_id = ?1",
            params![snap_id],
            |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, Option<String>>(1)?,
                    row.get::<_, String>(2)?,
                    row.get::<_, String>(3)?,
                    row.get::<_, Option<String>>(4)?,
                    row.get::<_, Option<String>>(5)?,
                    row.get::<_, Option<Vec<u8>>>(6)?,
                    row.get::<_, Option<Vec<u8>>>(7)?,
                    row.get::<_, Option<Vec<u8>>>(8)?,
                    row.get::<_, String>(9)?,
                    row.get::<_, i64>(10)?,
                    row.get::<_, Option<String>>(11)?,
                ))
            },
        ).map_err(|e| match e {
            rusqlite::Error::QueryReturnedNoRows => SessionDagError::SnapshotNotFound,
            other => SessionDagError::Sqlite(other),
        })?;

        let commit_hashes: Vec<String> = serde_json::from_str(&snapshot.9)
            .map_err(|_| SessionDagError::DatabaseCorrupted)?;

        let snap = Snapshot {
            snap_id: snapshot.0,
            parent_id: snapshot.1,
            session_id: snapshot.2,
            fsm_state: snapshot.3,
            phase: snapshot.4,
            step: snapshot.5,
            findings: snapshot.6,
            code_changes: snapshot.7,
            tests_written: snapshot.8,
            commit_hashes,
            created_at: nonneg_u64(snapshot.10)?,
            tag: snapshot.11,
        };

        let mut stmt = self.db.prepare_cached(
            "SELECT seq, role, content, is_stubbed, stub_ssb_seq \
             FROM snapshot_context WHERE snap_id = ?1 ORDER BY seq ASC",
        )?;

        let messages = stmt
            .query_map(params![snap_id], |row| {
                Ok((
                    row.get::<_, u32>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, Vec<u8>>(2)?,
                    row.get::<_, i64>(3)?,
                    row.get::<_, Option<i64>>(4)?,
                ))
            })?
            .map(|r| {
                let row = r.map_err(SessionDagError::Sqlite)?;
                let role = DagMessageRole::from_str_checked(&row.1)
                    .ok_or(SessionDagError::DatabaseCorrupted)?;
                let stub_ssb_seq = row.4
                    .map(nonneg_u64)
                    .transpose()?;
                Ok(TrimmedMessage {
                    seq: row.0,
                    role,
                    content: row.2,
                    is_stubbed: row.3 != 0,
                    stub_ssb_seq,
                })
            })
            .collect::<Result<Vec<_>, SessionDagError>>()?;

        Ok((snap, messages))
    }

    /// Returns the most recent snapshot, optionally filtered by tag.
    pub fn latest_snapshot(
        &self,
        tag: Option<&str>,
    ) -> Result<Option<Snapshot>, SessionDagError> {
        let result = match tag {
            Some(t) => self.db.query_row(
                "SELECT snap_id, parent_id, session_id, fsm_state, phase, step, \
                 findings, code_changes, tests_written, commit_hashes, created_at, tag \
                 FROM snapshots WHERE session_id = ?1 AND tag = ?2 \
                 ORDER BY created_at DESC LIMIT 1",
                params![self.session_id, t],
                |row| {
                    Ok((
                        row.get::<_, String>(0)?,
                        row.get::<_, Option<String>>(1)?,
                        row.get::<_, String>(2)?,
                        row.get::<_, String>(3)?,
                        row.get::<_, Option<String>>(4)?,
                        row.get::<_, Option<String>>(5)?,
                        row.get::<_, Option<Vec<u8>>>(6)?,
                        row.get::<_, Option<Vec<u8>>>(7)?,
                        row.get::<_, Option<Vec<u8>>>(8)?,
                        row.get::<_, String>(9)?,
                        row.get::<_, i64>(10)?,
                        row.get::<_, Option<String>>(11)?,
                    ))
                },
            ),
            None => self.db.query_row(
                "SELECT snap_id, parent_id, session_id, fsm_state, phase, step, \
                 findings, code_changes, tests_written, commit_hashes, created_at, tag \
                 FROM snapshots WHERE session_id = ?1 \
                 ORDER BY created_at DESC LIMIT 1",
                params![self.session_id],
                |row| {
                    Ok((
                        row.get::<_, String>(0)?,
                        row.get::<_, Option<String>>(1)?,
                        row.get::<_, String>(2)?,
                        row.get::<_, String>(3)?,
                        row.get::<_, Option<String>>(4)?,
                        row.get::<_, Option<String>>(5)?,
                        row.get::<_, Option<Vec<u8>>>(6)?,
                        row.get::<_, Option<Vec<u8>>>(7)?,
                        row.get::<_, Option<Vec<u8>>>(8)?,
                        row.get::<_, String>(9)?,
                        row.get::<_, i64>(10)?,
                        row.get::<_, Option<String>>(11)?,
                    ))
                },
            ),
        };

        match result {
            Ok(r) => {
                let commit_hashes: Vec<String> = serde_json::from_str(&r.9)
                    .map_err(|_| SessionDagError::DatabaseCorrupted)?;
                Ok(Some(Snapshot {
                    snap_id: r.0,
                    parent_id: r.1,
                    session_id: r.2,
                    fsm_state: r.3,
                    phase: r.4,
                    step: r.5,
                    findings: r.6,
                    code_changes: r.7,
                    tests_written: r.8,
                    commit_hashes,
                    created_at: nonneg_u64(r.10)?,
                    tag: r.11,
                }))
            }
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(SessionDagError::Sqlite(e)),
        }
    }

    /// Walks parent_id links from snap_id to root, returning the chain.
    pub fn parent_chain(
        &self,
        snap_id: &str,
    ) -> Result<Vec<Snapshot>, SessionDagError> {
        let mut chain = Vec::new();
        let mut visited = HashSet::new();
        let mut current_id = Some(snap_id.to_owned());

        while let Some(ref id) = current_id {
            if chain.len() >= MAX_PARENT_DEPTH {
                return Err(SessionDagError::CycleDetected);
            }

            if !visited.insert(id.clone()) {
                return Err(SessionDagError::CycleDetected);
            }

            let row = self.db.query_row(
                "SELECT snap_id, parent_id, session_id, fsm_state, phase, step, \
                 findings, code_changes, tests_written, commit_hashes, created_at, tag \
                 FROM snapshots WHERE snap_id = ?1",
                params![id],
                |row| {
                    Ok((
                        row.get::<_, String>(0)?,
                        row.get::<_, Option<String>>(1)?,
                        row.get::<_, String>(2)?,
                        row.get::<_, String>(3)?,
                        row.get::<_, Option<String>>(4)?,
                        row.get::<_, Option<String>>(5)?,
                        row.get::<_, Option<Vec<u8>>>(6)?,
                        row.get::<_, Option<Vec<u8>>>(7)?,
                        row.get::<_, Option<Vec<u8>>>(8)?,
                        row.get::<_, String>(9)?,
                        row.get::<_, i64>(10)?,
                        row.get::<_, Option<String>>(11)?,
                    ))
                },
            ).map_err(|e| match e {
                rusqlite::Error::QueryReturnedNoRows => SessionDagError::SnapshotNotFound,
                other => SessionDagError::Sqlite(other),
            })?;

            let commit_hashes: Vec<String> = serde_json::from_str(&row.9)
                .map_err(|_| SessionDagError::DatabaseCorrupted)?;

            let next_parent = row.1.clone();

            chain.push(Snapshot {
                snap_id: row.0,
                parent_id: row.1,
                session_id: row.2,
                fsm_state: row.3,
                phase: row.4,
                step: row.5,
                findings: row.6,
                code_changes: row.7,
                tests_written: row.8,
                commit_hashes,
                created_at: nonneg_u64(row.10)?,
                tag: row.11,
            });

            current_id = next_parent;
        }

        Ok(chain)
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use elai_core::SeededRng;
    use elai_core::clock::OsClock;
    use tempfile::TempDir;

    fn test_dir() -> TempDir {
        tempfile::tempdir().unwrap()
    }

    fn db_path(dir: &TempDir) -> std::path::PathBuf {
        dir.path().join("sessions").join("test_session_dag.db")
    }

    #[test]
    fn new_creates_db_at_expected_path() {
        let dir = test_dir();
        let path = db_path(&dir);
        let mut entropy = SeededRng::new(42);
        let clock = OsClock;

        let dag = SessionDag::new("s-test", &path, &mut entropy, &clock);
        assert!(dag.is_ok(), "new() should succeed");
        assert!(path.exists(), "db file should exist on disk");
    }

    #[test]
    fn new_is_idempotent() {
        let dir = test_dir();
        let path = db_path(&dir);
        let mut entropy = SeededRng::new(42);
        let clock = OsClock;

        let r1 = SessionDag::new("s-test", &path, &mut entropy, &clock);
        assert!(r1.is_ok());
        drop(r1);

        let r2 = SessionDag::new("s-test", &path, &mut entropy, &clock);
        assert!(r2.is_ok(), "second new() on same path should succeed");
    }

    #[test]
    fn open_nonexistent_returns_not_found() {
        let dir = test_dir();
        let path = dir.path().join("does_not_exist.db");

        let result = SessionDag::open("s-test", &path);
        assert!(
            matches!(result, Err(SessionDagError::DatabaseNotFound)),
            "open on missing file should return DatabaseNotFound"
        );
    }

    #[test]
    fn schema_has_exactly_three_tables() {
        let dir = test_dir();
        let path = db_path(&dir);
        let mut entropy = SeededRng::new(42);
        let clock = OsClock;

        let dag = SessionDag::new("s-test", &path, &mut entropy, &clock).unwrap();
        let count: i64 = dag
            .db()
            .query_row(
                "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name NOT LIKE 'sqlite_%'",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(count, 3, "should have schema_version + snapshots + snapshot_context");
    }

    #[test]
    fn schema_has_three_indexes() {
        let dir = test_dir();
        let path = db_path(&dir);
        let mut entropy = SeededRng::new(42);
        let clock = OsClock;

        let dag = SessionDag::new("s-test", &path, &mut entropy, &clock).unwrap();
        let count: i64 = dag
            .db()
            .query_row(
                "SELECT COUNT(*) FROM sqlite_master WHERE type='index' AND name LIKE 'idx_%'",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(count, 3, "should have idx_snapshots_session + idx_snapshots_tag + idx_context_stubbed");
    }

    #[test]
    fn all_types_serialize_deserialize() {
        let snap = Snapshot {
            snap_id: "snap-1".into(),
            parent_id: None,
            session_id: "s-1".into(),
            fsm_state: "EXECUTING".into(),
            phase: Some("3H".into()),
            step: Some("1".into()),
            findings: None,
            code_changes: None,
            tests_written: None,
            commit_hashes: vec!["abc123".into()],
            created_at: 1700000000,
            tag: None,
        };
        let json = serde_json::to_string(&snap).unwrap();
        let back: Snapshot = serde_json::from_str(&json).unwrap();
        assert_eq!(snap, back);

        let msg = TrimmedMessage {
            seq: 0,
            role: DagMessageRole::User,
            content: b"hello".to_vec(),
            is_stubbed: false,
            stub_ssb_seq: None,
        };
        let json = serde_json::to_string(&msg).unwrap();
        let back: TrimmedMessage = serde_json::from_str(&json).unwrap();
        assert_eq!(msg, back);

        let stub = Stub {
            original_type: "tool_result".into(),
            original_tokens: 500,
            summary_hash: [0xAB; 32],
            ssb_seq: 42,
        };
        let json = serde_json::to_string(&stub).unwrap();
        let back: Stub = serde_json::from_str(&json).unwrap();
        assert_eq!(stub, back);

        let branch = BranchRef {
            name: "explore-1".into(),
            tip_snap_id: "snap-1".into(),
            created_at: 1700000000,
        };
        let json = serde_json::to_string(&branch).unwrap();
        let back: BranchRef = serde_json::from_str(&json).unwrap();
        assert_eq!(branch, back);

        let role = DagMessageRole::Assistant;
        let json = serde_json::to_string(&role).unwrap();
        let back: DagMessageRole = serde_json::from_str(&json).unwrap();
        assert_eq!(role, back);
    }

    #[test]
    fn stub_msgpack_round_trip() {
        let stub = Stub {
            original_type: "file_write".into(),
            original_tokens: 1024,
            summary_hash: [0xCD; 32],
            ssb_seq: 99,
        };
        let packed = rmp_serde::to_vec(&stub).unwrap();
        let unpacked: Stub = rmp_serde::from_slice(&packed).unwrap();
        assert_eq!(stub, unpacked);
    }

    #[test]
    fn open_corrupt_db_returns_database_corrupted() {
        let dir = test_dir();
        let path = db_path(&dir);

        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(&path, b"this is not a valid sqlite database").unwrap();

        let result = SessionDag::open("s-test", &path);
        assert!(
            matches!(
                result,
                Err(SessionDagError::DatabaseCorrupted) | Err(SessionDagError::Sqlite(_))
            ),
            "opening a corrupt file should return DatabaseCorrupted or Sqlite error"
        );
    }

    #[test]
    fn fk_enforcement_active() {
        let dir = test_dir();
        let path = db_path(&dir);
        let mut entropy = SeededRng::new(42);
        let clock = OsClock;

        let dag = SessionDag::new("s-test", &path, &mut entropy, &clock).unwrap();

        let result = dag.db().execute(
            "INSERT INTO snapshots (snap_id, parent_id, session_id, fsm_state, created_at)
             VALUES (?1, ?2, ?3, ?4, ?5)",
            params!["snap-child", "nonexistent-parent", "s-test", "IDLE", 1700000000i64],
        );

        assert!(result.is_err(), "FK violation should be rejected");
    }

    #[test]
    fn generate_snap_id_is_deterministic() {
        let mut e1 = SeededRng::new(99);
        let mut e2 = SeededRng::new(99);
        let id1 = SessionDag::generate_snap_id(&mut e1);
        let id2 = SessionDag::generate_snap_id(&mut e2);
        assert_eq!(id1, id2, "same seed must produce same snap_id");
        assert_eq!(id1.len(), 36, "UUID string length");
    }

    // ------------------------------------------------------------------
    // 3H.2 tests
    // ------------------------------------------------------------------

    fn make_snapshot(snap_id: &str, parent_id: Option<&str>, session_id: &str, created_at: u64, tag: Option<&str>) -> Snapshot {
        Snapshot {
            snap_id: snap_id.to_owned(),
            parent_id: parent_id.map(|s| s.to_owned()),
            session_id: session_id.to_owned(),
            fsm_state: "EXECUTING".to_owned(),
            phase: Some("3H".to_owned()),
            step: Some("2".to_owned()),
            findings: None,
            code_changes: None,
            tests_written: None,
            commit_hashes: vec!["abc123".to_owned(), "def456".to_owned()],
            created_at,
            tag: tag.map(|s| s.to_owned()),
        }
    }

    fn make_message(seq: u32, role: DagMessageRole, content: &[u8]) -> TrimmedMessage {
        TrimmedMessage {
            seq,
            role,
            content: content.to_vec(),
            is_stubbed: false,
            stub_ssb_seq: None,
        }
    }

    #[test]
    fn test_save_and_load_snapshot_round_trip() {
        let dir = test_dir();
        let path = db_path(&dir);
        let mut entropy = SeededRng::new(42);
        let clock = OsClock;
        let dag = SessionDag::new("s-test", &path, &mut entropy, &clock).unwrap();

        let snap = make_snapshot("snap-rt", None, "s-test", 1000, None);
        let msgs = vec![
            make_message(3, DagMessageRole::Assistant, b"third"),
            make_message(1, DagMessageRole::System, b"first"),
            make_message(2, DagMessageRole::User, b"second"),
        ];

        dag.save_snapshot(&snap, &msgs).unwrap();

        let (loaded_snap, loaded_msgs) = dag.load_snapshot("snap-rt").unwrap();
        assert_eq!(loaded_snap.snap_id, "snap-rt");
        assert_eq!(loaded_snap.commit_hashes, vec!["abc123", "def456"]);
        assert_eq!(loaded_snap.created_at, 1000);
        assert_eq!(loaded_snap.parent_id, None);

        assert_eq!(loaded_msgs.len(), 3);
        assert_eq!(loaded_msgs[0].seq, 1, "messages must be ordered by seq ASC");
        assert_eq!(loaded_msgs[1].seq, 2);
        assert_eq!(loaded_msgs[2].seq, 3);
        assert_eq!(loaded_msgs[0].role, DagMessageRole::System);
        assert_eq!(loaded_msgs[1].content, b"second");
    }

    #[test]
    fn test_load_snapshot_not_found() {
        let dir = test_dir();
        let path = db_path(&dir);
        let mut entropy = SeededRng::new(42);
        let clock = OsClock;
        let dag = SessionDag::new("s-test", &path, &mut entropy, &clock).unwrap();

        let result = dag.load_snapshot("nonexistent");
        assert!(
            matches!(result, Err(SessionDagError::SnapshotNotFound)),
            "load_snapshot on missing snap_id should return SnapshotNotFound"
        );
    }

    #[test]
    fn test_transaction_rollback_on_partial_write() {
        let dir = test_dir();
        let path = db_path(&dir);
        let mut entropy = SeededRng::new(42);
        let clock = OsClock;
        let dag = SessionDag::new("s-test", &path, &mut entropy, &clock).unwrap();

        // Attempt an atomic insert that will fail on the context row (invalid role)
        let result = dag.db().execute_batch(
            "BEGIN IMMEDIATE; \
             INSERT INTO snapshots (snap_id, parent_id, session_id, fsm_state, created_at) \
             VALUES ('snap-rollback', NULL, 's-test', 'IDLE', 1000); \
             INSERT INTO snapshot_context (snap_id, seq, role, content, is_stubbed) \
             VALUES ('snap-rollback', 1, 'invalid_role', X'AA', 0); \
             COMMIT",
        );
        assert!(result.is_err(), "invalid role should violate CHECK constraint");

        // After failure, the transaction was not committed — clean up any leftover state
        let _ = dag.db().execute_batch("ROLLBACK");

        let count: i64 = dag.db().query_row(
            "SELECT COUNT(*) FROM snapshots WHERE snap_id = 'snap-rollback'",
            [],
            |row| row.get(0),
        ).unwrap();
        assert_eq!(count, 0, "snapshots row must not persist after failed transaction");
    }

    #[test]
    fn test_latest_snapshot_returns_newest() {
        let dir = test_dir();
        let path = db_path(&dir);
        let mut entropy = SeededRng::new(42);
        let clock = OsClock;
        let dag = SessionDag::new("s-test", &path, &mut entropy, &clock).unwrap();

        let s1 = make_snapshot("snap-100", None, "s-test", 100, None);
        let s2 = make_snapshot("snap-200", None, "s-test", 200, Some("tagged"));
        let s3 = make_snapshot("snap-300", None, "s-test", 300, None);

        dag.save_snapshot(&s1, &[]).unwrap();
        dag.save_snapshot(&s2, &[]).unwrap();
        dag.save_snapshot(&s3, &[]).unwrap();

        // No tag filter: should return newest (created_at=300)
        let latest = dag.latest_snapshot(None).unwrap().unwrap();
        assert_eq!(latest.snap_id, "snap-300");

        // Tag filter: only snap-200 has "tagged"
        let tagged = dag.latest_snapshot(Some("tagged")).unwrap().unwrap();
        assert_eq!(tagged.snap_id, "snap-200");

        // Absent tag: should return None
        let absent = dag.latest_snapshot(Some("absent_tag")).unwrap();
        assert!(absent.is_none(), "absent tag should return Ok(None)");
    }

    #[test]
    fn test_parent_chain_linear() {
        let dir = test_dir();
        let path = db_path(&dir);
        let mut entropy = SeededRng::new(42);
        let clock = OsClock;
        let dag = SessionDag::new("s-test", &path, &mut entropy, &clock).unwrap();

        let root = make_snapshot("snap-root", None, "s-test", 100, None);
        let mid = make_snapshot("snap-mid", Some("snap-root"), "s-test", 200, None);
        let leaf = make_snapshot("snap-leaf", Some("snap-mid"), "s-test", 300, None);

        dag.save_snapshot(&root, &[]).unwrap();
        dag.save_snapshot(&mid, &[]).unwrap();
        dag.save_snapshot(&leaf, &[]).unwrap();

        let chain = dag.parent_chain("snap-leaf").unwrap();
        assert_eq!(chain.len(), 3);
        assert_eq!(chain[0].snap_id, "snap-leaf");
        assert_eq!(chain[1].snap_id, "snap-mid");
        assert_eq!(chain[2].snap_id, "snap-root");
        assert!(chain[2].parent_id.is_none(), "root should have no parent");
    }

    #[test]
    fn test_parent_chain_cycle_detected() {
        let dir = test_dir();
        let path = db_path(&dir);
        let mut entropy = SeededRng::new(42);
        let clock = OsClock;
        let dag = SessionDag::new("s-test", &path, &mut entropy, &clock).unwrap();

        // Insert two snapshots with no parent first (FK requires parent to exist)
        dag.db().execute(
            "INSERT INTO snapshots (snap_id, parent_id, session_id, fsm_state, commit_hashes, created_at) \
             VALUES ('snap-a', NULL, 's-test', 'IDLE', '[]', 100)",
            [],
        ).unwrap();
        dag.db().execute(
            "INSERT INTO snapshots (snap_id, parent_id, session_id, fsm_state, commit_hashes, created_at) \
             VALUES ('snap-b', 'snap-a', 's-test', 'IDLE', '[]', 200)",
            [],
        ).unwrap();

        // Create cycle: A -> B -> A (disable FK temporarily to allow the circular reference)
        dag.db().execute_batch("PRAGMA foreign_keys = OFF").unwrap();
        dag.db().execute(
            "UPDATE snapshots SET parent_id = 'snap-b' WHERE snap_id = 'snap-a'",
            [],
        ).unwrap();
        dag.db().execute_batch("PRAGMA foreign_keys = ON").unwrap();

        let result = dag.parent_chain("snap-a");
        assert!(
            matches!(result, Err(SessionDagError::CycleDetected)),
            "cycle in parent chain should return CycleDetected"
        );
    }

    #[test]
    fn test_save_snapshot_1000_messages_performance() {
        let dir = test_dir();
        let path = db_path(&dir);
        let mut entropy = SeededRng::new(42);
        let clock = OsClock;
        let dag = SessionDag::new("s-test", &path, &mut entropy, &clock).unwrap();

        let snap = make_snapshot("snap-perf", None, "s-test", 1000, None);
        let msgs: Vec<TrimmedMessage> = (0..1000)
            .map(|i| make_message(i, DagMessageRole::User, &[0xAA; 128]))
            .collect();

        let start = std::time::Instant::now();
        dag.save_snapshot(&snap, &msgs).unwrap();
        let elapsed = start.elapsed();

        assert!(
            elapsed.as_millis() < 500,
            "batch insert of 1000 messages took {}ms, expected < 500ms",
            elapsed.as_millis()
        );
    }

    #[test]
    fn test_latest_snapshot_empty_db() {
        let dir = test_dir();
        let path = db_path(&dir);
        let mut entropy = SeededRng::new(42);
        let clock = OsClock;
        let dag = SessionDag::new("s-test", &path, &mut entropy, &clock).unwrap();

        let result = dag.latest_snapshot(None).unwrap();
        assert!(result.is_none(), "empty db should return Ok(None)");
    }
}
