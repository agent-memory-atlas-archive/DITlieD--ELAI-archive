use std::fmt;
use std::path::Path;

use rusqlite::{params, Connection, OpenFlags};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use elai_core::clock::ClockProvider;
use elai_core::entropy::EntropySource;

pub const SCHEMA_VERSION: u32 = 1;
pub const MAX_ACTIVE_BRANCHES: usize = 3;

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
        db.execute_batch(CREATE_SCHEMA_VERSION_TABLE)?;
        db.execute_batch(CREATE_SNAPSHOTS_TABLE)?;
        db.execute_batch(CREATE_SNAPSHOT_CONTEXT_TABLE)?;
        db.execute_batch(CREATE_INDEXES)?;
        db.execute(
            "INSERT OR IGNORE INTO schema_version (version) VALUES (?1)",
            params![SCHEMA_VERSION],
        )?;
        db.execute_batch("COMMIT")?;

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
}
