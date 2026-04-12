use std::collections::HashSet;
use std::fmt;
use std::path::Path;

use rusqlite::{params, Connection, OpenFlags};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use elai_core::clock::ClockProvider;
use elai_core::entropy::EntropySource;

use crate::ssb::SsbReader;

pub const SCHEMA_VERSION: u32 = 1;
pub const MAX_ACTIVE_BRANCHES: usize = 3;
pub const MAX_PARENT_DEPTH: usize = 1000;
pub const STUB_THRESHOLD_MIN: usize = 100;
pub const STUB_THRESHOLD_MAX: usize = 1000;
pub const PROMPT_CACHE_PREFIX_BOUNDARY: u32 = 3;

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
    #[error("trim budget exceeded: {actual} tokens > {target} target")]
    TrimBudgetExceeded { actual: usize, target: usize },
    #[error("invalid trim config: {reason}")]
    InvalidTrimConfig { reason: &'static str },
    #[error("stub seq {seq} recovery failed: {reason}")]
    StubRecoveryFailed { seq: u32, reason: String },
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TrimConfig {
    pub stub_threshold: usize,
    pub model_context_window: usize,
    pub allocated_slots_tokens: usize,
    pub output_reserve: usize,
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

// FIXME(3H.7): replace with LlmProvider::token_count when providers crate implements it.
// Current heuristic: ~4 bytes per token (conservative estimate for English text).
fn estimate_tokens(bytes: &[u8]) -> usize {
    bytes.len().saturating_add(3) / 4
}

fn nonneg_u64(val: i64) -> Result<u64, SessionDagError> {
    u64::try_from(val).map_err(|_| SessionDagError::DatabaseCorrupted)
}

fn safe_i64(val: u64) -> Result<i64, SessionDagError> {
    i64::try_from(val).map_err(|_| SessionDagError::DatabaseCorrupted)
}

pub struct SessionDag {
    db: Connection,
    session_id: String,
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

        let active_branches = Self::load_branches_from_db(&db, session_id)?;

        Ok(Self {
            db,
            session_id: session_id.to_owned(),
            active_branches,
        })
    }

    fn load_branches_from_db(
        db: &Connection,
        session_id: &str,
    ) -> Result<Vec<BranchRef>, SessionDagError> {
        // Group by branch name, take the latest snap_id per branch (dedup).
        // This handles cases where save_snapshot inserts additional snapshots
        // with the same branch tag — we always pick the newest tip.
        let mut stmt = db.prepare(
            "SELECT s.snap_id, s.tag, s.created_at \
             FROM snapshots s \
             INNER JOIN ( \
                 SELECT tag, MAX(created_at) AS max_ts \
                 FROM snapshots \
                 WHERE session_id = ?1 AND tag LIKE 'branch:%' \
                 GROUP BY tag \
             ) g ON s.tag = g.tag AND s.created_at = g.max_ts \
             WHERE s.session_id = ?1 AND s.tag LIKE 'branch:%' \
             ORDER BY s.created_at ASC",
        )?;
        let rows = stmt
            .query_map(params![session_id], |row| {
                let snap_id: String = row.get(0)?;
                let tag: String = row.get(1)?;
                let created_at_i64: i64 = row.get(2)?;
                Ok((snap_id, tag, created_at_i64))
            })?
            .collect::<Result<Vec<_>, _>>()?;

        let mut branches = Vec::new();
        for (snap_id, tag, created_at_i64) in rows {
            let name = tag
                .strip_prefix("branch:")
                .ok_or(SessionDagError::DatabaseCorrupted)?
                .to_owned();
            let created_at = nonneg_u64(created_at_i64)?;
            branches.push(BranchRef {
                name,
                tip_snap_id: snap_id,
                created_at,
            });
        }
        Ok(branches)
    }

    #[cfg(test)]
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

    // ------------------------------------------------------------------
    // Context trimming (3H.3)
    // ------------------------------------------------------------------

    /// 3-pass structurally lossless context trimming.
    ///
    /// Pass 1 (identify): find Tool messages with seq > PROMPT_CACHE_PREFIX_BOUNDARY
    ///   whose estimated token count exceeds `config.stub_threshold`.
    /// Pass 2 (stub): replace identified candidates with msgpack-serialized Stub
    ///   payloads, preserving recovery metadata (blake3 hash, ssb_seq).
    /// Pass 3 (validate + budget): unstub any message whose ssb_seq is not in
    ///   `valid_ssb_seqs`, then check the total token budget.
    ///
    /// Pure function — no DB writes, no side effects.
    pub fn trim_context(
        &self,
        messages: &[TrimmedMessage],
        config: TrimConfig,
        valid_ssb_seqs: &HashSet<u64>,
    ) -> Result<Vec<TrimmedMessage>, SessionDagError> {
        // Validate config
        if config.stub_threshold < STUB_THRESHOLD_MIN
            || config.stub_threshold > STUB_THRESHOLD_MAX
        {
            return Err(SessionDagError::InvalidTrimConfig {
                reason: "stub_threshold must be in [100, 1000]",
            });
        }

        if config.model_context_window == 0
            || config.allocated_slots_tokens.saturating_add(config.output_reserve)
                >= config.model_context_window
        {
            return Err(SessionDagError::InvalidTrimConfig {
                reason: "model_context_window must exceed allocated_slots + output_reserve",
            });
        }

        let target_tokens = config
            .model_context_window
            .saturating_sub(config.allocated_slots_tokens)
            .saturating_sub(config.output_reserve);

        if messages.is_empty() {
            return Ok(Vec::new());
        }

        // Pass 1: identify stub candidates
        // candidate_indices maps index -> original token count
        let mut candidate_indices: Vec<(usize, usize)> = Vec::new();
        let token_counts: Vec<usize> = messages
            .iter()
            .map(|m| estimate_tokens(&m.content))
            .collect();

        for (i, msg) in messages.iter().enumerate() {
            if msg.role != DagMessageRole::Tool {
                continue;
            }
            if msg.is_stubbed {
                // Already stubbed — never double-stub
                continue;
            }
            if msg.seq <= PROMPT_CACHE_PREFIX_BOUNDARY {
                continue;
            }
            if msg.stub_ssb_seq.is_none() {
                // Cannot recover from SSB without a seq — skip stubbing
                continue;
            }
            if token_counts[i] > config.stub_threshold {
                candidate_indices.push((i, token_counts[i]));
            }
        }

        // Pass 2: stub candidates
        let mut result: Vec<TrimmedMessage> = messages.to_vec();

        for &(idx, original_tokens) in &candidate_indices {
            let msg = &messages[idx];
            let ssb_seq = match msg.stub_ssb_seq {
                Some(seq) => seq,
                None => continue, // defensive — already filtered in Pass 1
            };

            let hash: [u8; 32] = blake3::hash(&msg.content).into();
            let stub = Stub {
                original_type: "tool_result".to_owned(),
                original_tokens,
                summary_hash: hash,
                ssb_seq,
            };

            let stub_bytes = rmp_serde::to_vec(&stub).map_err(|_| {
                SessionDagError::DatabaseCorrupted
            })?;

            result[idx] = TrimmedMessage {
                seq: msg.seq,
                role: msg.role,
                content: stub_bytes,
                is_stubbed: true,
                stub_ssb_seq: Some(ssb_seq),
            };
        }

        // Pass 3: validate SSB refs + budget check
        // Unstub messages whose ssb_seq is not in valid_ssb_seqs
        for (idx, _) in &candidate_indices {
            let trimmed = &result[*idx];
            if !trimmed.is_stubbed {
                continue;
            }
            let ssb_seq = match trimmed.stub_ssb_seq {
                Some(seq) => seq,
                None => continue,
            };
            if !valid_ssb_seqs.contains(&ssb_seq) {
                // Restore original — SSB cannot recover this message
                result[*idx] = messages[*idx].clone();
            }
        }

        // Budget check: compute total tokens after stubbing decisions
        let total_tokens: usize = result.iter().enumerate().map(|(i, msg)| {
            if msg.is_stubbed {
                estimate_tokens(&msg.content)
            } else {
                token_counts[i]
            }
        }).sum();

        if total_tokens > target_tokens {
            return Err(SessionDagError::TrimBudgetExceeded {
                actual: total_tokens,
                target: target_tokens,
            });
        }

        Ok(result)
    }

    // ------------------------------------------------------------------
    // Branch operations (3H.4)
    // ------------------------------------------------------------------

    fn is_valid_branch_name(name: &str) -> bool {
        !name.is_empty()
            && name.len() <= 64
            && name
                .bytes()
                .all(|b| b.is_ascii_alphanumeric() || b == b'-')
    }

    pub fn create_branch(
        &mut self,
        name: &str,
        from_snap_id: &str,
        entropy: &mut impl EntropySource,
        clock: &impl ClockProvider,
    ) -> Result<BranchRef, SessionDagError> {
        if !Self::is_valid_branch_name(name) {
            return Err(SessionDagError::InvalidBranchName);
        }

        if self.active_branches.iter().any(|b| b.name == name) {
            return Err(SessionDagError::InvalidBranchName);
        }

        if self.active_branches.len() >= MAX_ACTIVE_BRANCHES {
            return Err(SessionDagError::BranchLimitExceeded {
                max: MAX_ACTIVE_BRANCHES,
            });
        }

        // Verify from_snap_id exists
        let parent_exists: bool = self
            .db
            .query_row(
                "SELECT COUNT(*) FROM snapshots WHERE snap_id = ?1 AND session_id = ?2",
                params![from_snap_id, self.session_id],
                |row| row.get::<_, i64>(0),
            )
            .map(|c| c > 0)
            .map_err(SessionDagError::Sqlite)?;

        if !parent_exists {
            return Err(SessionDagError::SnapshotNotFound);
        }

        let snap_id = Self::generate_snap_id(entropy);
        let created_at = clock
            .system_now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| u64::try_from(d.as_millis()).unwrap_or(u64::MAX))
            .unwrap_or(0);
        let created_at_i64 = safe_i64(created_at)?;

        self.db.execute_batch("BEGIN IMMEDIATE")?;

        let txn_result = (|| -> Result<(), SessionDagError> {
            // Create branch snapshot with tag
            self.db.execute(
                "INSERT INTO snapshots (snap_id, parent_id, session_id, fsm_state, phase, step, \
                 findings, code_changes, tests_written, commit_hashes, created_at, tag) \
                 SELECT ?1, ?2, ?3, fsm_state, phase, step, \
                 findings, code_changes, tests_written, commit_hashes, ?4, ?5 \
                 FROM snapshots WHERE snap_id = ?6",
                params![
                    snap_id,
                    from_snap_id,
                    self.session_id,
                    created_at_i64,
                    format!("branch:{name}"),
                    from_snap_id,
                ],
            )?;

            // Copy snapshot_context from parent
            self.db.execute(
                "INSERT INTO snapshot_context (snap_id, seq, role, content, is_stubbed, stub_ssb_seq) \
                 SELECT ?1, seq, role, content, is_stubbed, stub_ssb_seq \
                 FROM snapshot_context WHERE snap_id = ?2",
                params![snap_id, from_snap_id],
            )?;

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

        let branch = BranchRef {
            name: name.to_owned(),
            tip_snap_id: snap_id,
            created_at,
        };
        self.active_branches.push(branch.clone());
        Ok(branch)
    }

    pub fn list_branches(&self) -> &[BranchRef] {
        &self.active_branches
    }

    pub fn merge_branch(
        &mut self,
        winner_name: &str,
        entropy: &mut impl EntropySource,
        clock: &impl ClockProvider,
    ) -> Result<MergeResult, SessionDagError> {
        let winner_idx = self
            .active_branches
            .iter()
            .position(|b| b.name == winner_name)
            .ok_or(SessionDagError::BranchNotFound)?;

        // Check for conflicts: overlapping file paths in code_changes
        if self.active_branches.len() > 1 {
            let mut all_paths: Vec<(usize, Vec<String>)> = Vec::new();
            for (idx, branch) in self.active_branches.iter().enumerate() {
                let paths = self.extract_code_change_paths(&branch.tip_snap_id)?;
                all_paths.push((idx, paths));
            }

            for i in 0..all_paths.len() {
                for j in (i + 1)..all_paths.len() {
                    let set_i: HashSet<&str> =
                        all_paths[i].1.iter().map(|s| s.as_str()).collect();
                    let overlap: Vec<&str> = all_paths[j]
                        .1
                        .iter()
                        .filter(|p| set_i.contains(p.as_str()))
                        .map(|s| s.as_str())
                        .collect();
                    if !overlap.is_empty() {
                        return Ok(MergeResult::Conflict {
                            reason: format!(
                                "overlapping code_changes in branches '{}' and '{}': {:?}",
                                self.active_branches[all_paths[i].0].name,
                                self.active_branches[all_paths[j].0].name,
                                overlap,
                            ),
                        });
                    }
                }
            }
        }

        let winner = &self.active_branches[winner_idx];
        let winner_tip = winner.tip_snap_id.clone();

        let new_snap_id = Self::generate_snap_id(entropy);
        let created_at = clock
            .system_now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| u64::try_from(d.as_millis()).unwrap_or(u64::MAX))
            .unwrap_or(0);
        let created_at_i64 = safe_i64(created_at)?;

        self.db.execute_batch("BEGIN IMMEDIATE")?;

        let txn_result = (|| -> Result<(), SessionDagError> {
            // Create trunk snapshot from winner tip
            self.db.execute(
                "INSERT INTO snapshots (snap_id, parent_id, session_id, fsm_state, phase, step, \
                 findings, code_changes, tests_written, commit_hashes, created_at, tag) \
                 SELECT ?1, ?2, ?3, fsm_state, phase, step, \
                 findings, code_changes, tests_written, commit_hashes, ?4, NULL \
                 FROM snapshots WHERE snap_id = ?5",
                params![
                    new_snap_id,
                    winner_tip,
                    self.session_id,
                    created_at_i64,
                    winner_tip,
                ],
            )?;

            // Copy snapshot_context from winner tip
            self.db.execute(
                "INSERT INTO snapshot_context (snap_id, seq, role, content, is_stubbed, stub_ssb_seq) \
                 SELECT ?1, seq, role, content, is_stubbed, stub_ssb_seq \
                 FROM snapshot_context WHERE snap_id = ?2",
                params![new_snap_id, winner_tip],
            )?;

            // Tag all branches' snapshots as abandoned (including winner — merged into trunk)
            for branch in self.active_branches.iter() {
                let rows = self.db.execute(
                    "UPDATE snapshots SET tag = ?1 WHERE tag = ?2 AND session_id = ?3",
                    params![
                        format!("abandoned:{}", branch.name),
                        format!("branch:{}", branch.name),
                        self.session_id,
                    ],
                )?;
                if rows == 0 {
                    return Err(SessionDagError::SnapshotNotFound);
                }
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

        // Build result before draining
        let mut winner_ref = self.active_branches[winner_idx].clone();
        winner_ref.tip_snap_id = new_snap_id;

        let abandoned: Vec<BranchRef> = self
            .active_branches
            .drain(..)
            .filter(|b| b.name != winner_name)
            .collect();

        Ok(MergeResult::Winner {
            branch: winner_ref,
            abandoned,
        })
    }

    pub fn close_branch(&mut self, name: &str) -> Result<(), SessionDagError> {
        let idx = self
            .active_branches
            .iter()
            .position(|b| b.name == name)
            .ok_or(SessionDagError::BranchNotFound)?;

        self.db.execute_batch("BEGIN IMMEDIATE")?;

        let txn_result = (|| -> Result<(), SessionDagError> {
            let rows = self.db.execute(
                "UPDATE snapshots SET tag = ?1 WHERE tag = ?2 AND session_id = ?3",
                params![
                    format!("abandoned:{name}"),
                    format!("branch:{name}"),
                    self.session_id,
                ],
            )?;
            if rows == 0 {
                return Err(SessionDagError::SnapshotNotFound);
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

        self.active_branches.remove(idx);
        Ok(())
    }

    fn extract_code_change_paths(&self, snap_id: &str) -> Result<Vec<String>, SessionDagError> {
        let blob: Option<Vec<u8>> = self
            .db
            .query_row(
                "SELECT code_changes FROM snapshots WHERE snap_id = ?1 AND session_id = ?2",
                params![snap_id, self.session_id],
                |row| row.get(0),
            )
            .map_err(|e| match e {
                rusqlite::Error::QueryReturnedNoRows => SessionDagError::SnapshotNotFound,
                other => SessionDagError::Sqlite(other),
            })?;

        let Some(data) = blob else {
            return Ok(Vec::new());
        };

        // code_changes is stored as JSON — extract file paths from it
        let value: serde_json::Value =
            serde_json::from_slice(&data).map_err(|_| SessionDagError::DatabaseCorrupted)?;

        let mut paths = Vec::new();
        match &value {
            serde_json::Value::Object(map) => {
                for key in map.keys() {
                    paths.push(key.clone());
                }
            }
            serde_json::Value::Array(arr) => {
                for item in arr {
                    if let Some(path) = item.get("path").and_then(|v| v.as_str()) {
                        paths.push(path.to_owned());
                    }
                }
            }
            _ => {}
        }
        Ok(paths)
    }

    // ------------------------------------------------------------------
    // Stub recovery (3H.5)
    // ------------------------------------------------------------------

    /// Recover all stubbed messages in a snapshot by fetching original payloads
    /// from SSB and verifying blake3 integrity hashes.
    ///
    /// # Contract
    /// The SSB entry at `stub.ssb_seq` must contain the **raw content bytes**
    /// (i.e. written via `Ssb::write()`, not `write_payload()` which adds msgpack
    /// encoding). `trim_context` hashes `msg.content` directly; recovery verifies
    /// `blake3(entry.payload) == stub.summary_hash`. Callers populating `stub_ssb_seq`
    /// on `TrimmedMessage` are responsible for this invariant.
    pub fn recover_stubs(
        &self,
        snap_id: &str,
        ssb: &dyn SsbReader,
    ) -> Result<Vec<TrimmedMessage>, SessionDagError> {
        let (_snapshot, messages) = self.load_snapshot(snap_id)?;
        let mut result = messages;

        for msg in &mut result {
            if !msg.is_stubbed {
                continue;
            }
            let stub: Stub = rmp_serde::from_slice(&msg.content).map_err(|_| {
                SessionDagError::StubRecoveryFailed {
                    seq: msg.seq,
                    reason: "stub deserialization failed".to_owned(),
                }
            })?;

            let ssb_seq = safe_i64(stub.ssb_seq).map_err(|_| {
                SessionDagError::StubRecoveryFailed {
                    seq: msg.seq,
                    reason: "ssb_seq overflow".to_owned(),
                }
            })?;
            let entry = ssb.query_by_seq(ssb_seq).map_err(|_| {
                SessionDagError::StubRecoveryFailed {
                    seq: msg.seq,
                    reason: "SSB query failed".to_owned(),
                }
            })?;
            let entry = entry.ok_or_else(|| SessionDagError::StubRecoveryFailed {
                seq: msg.seq,
                reason: "SSB entry missing".to_owned(),
            })?;

            let hash: [u8; 32] = blake3::hash(&entry.payload).into();
            if hash != stub.summary_hash {
                return Err(SessionDagError::StubRecoveryFailed {
                    seq: msg.seq,
                    reason: "integrity check failed".to_owned(),
                });
            }

            msg.content = entry.payload.to_vec();
            msg.is_stubbed = false;
            msg.stub_ssb_seq = None;
        }

        Ok(result)
    }

    /// Recover a single stubbed message by seq within a snapshot.
    /// Returns the message unchanged if it is not stubbed.
    pub fn recover_single_stub(
        &self,
        snap_id: &str,
        seq: u32,
        ssb: &dyn SsbReader,
    ) -> Result<TrimmedMessage, SessionDagError> {
        let (_snapshot, messages) = self.load_snapshot(snap_id)?;
        let mut msg = messages
            .into_iter()
            .find(|m| m.seq == seq)
            .ok_or_else(|| SessionDagError::StubRecoveryFailed {
                seq,
                reason: "message seq not found in snapshot".to_owned(),
            })?;

        if !msg.is_stubbed {
            return Ok(msg);
        }

        let stub: Stub = rmp_serde::from_slice(&msg.content).map_err(|_| {
            SessionDagError::StubRecoveryFailed {
                seq: msg.seq,
                reason: "stub deserialization failed".to_owned(),
            }
        })?;

        let ssb_seq = safe_i64(stub.ssb_seq).map_err(|_| {
            SessionDagError::StubRecoveryFailed {
                seq: msg.seq,
                reason: "ssb_seq overflow".to_owned(),
            }
        })?;
        let entry = ssb.query_by_seq(ssb_seq).map_err(|_| {
            SessionDagError::StubRecoveryFailed {
                seq: msg.seq,
                reason: "SSB query failed".to_owned(),
            }
        })?;
        let entry = entry.ok_or_else(|| SessionDagError::StubRecoveryFailed {
            seq: msg.seq,
            reason: "SSB entry missing".to_owned(),
        })?;

        let hash: [u8; 32] = blake3::hash(&entry.payload).into();
        if hash != stub.summary_hash {
            return Err(SessionDagError::StubRecoveryFailed {
                seq: msg.seq,
                reason: "integrity check failed".to_owned(),
            });
        }

        msg.content = entry.payload.to_vec();
        msg.is_stubbed = false;
        msg.stub_ssb_seq = None;

        Ok(msg)
    }

    // ------------------------------------------------------------------
    // Handoff operations (3H.6)
    // ------------------------------------------------------------------

    /// Creates a handoff snapshot: runs 3-pass trim, saves with tag="handoff".
    ///
    /// `ssb` is not used directly but reserved for future recovery-path
    /// integration. `valid_ssb_seqs` are derived from the messages themselves.
    pub fn create_handoff(
        &self,
        messages: &[TrimmedMessage],
        config: TrimConfig,
        _ssb: &dyn SsbReader,
        entropy: &mut impl EntropySource,
        clock: &impl ClockProvider,
        fsm_state: &str,
    ) -> Result<Snapshot, SessionDagError> {
        // Derive valid_ssb_seqs from messages
        let valid_ssb_seqs: HashSet<u64> = messages
            .iter()
            .filter_map(|m| m.stub_ssb_seq)
            .collect();

        // Pass 1-3: trim context
        let trimmed = self.trim_context(messages, config, &valid_ssb_seqs)?;

        // Generate snapshot identity
        let snap_id = Self::generate_snap_id(entropy);
        let created_at = clock
            .system_now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| u64::try_from(d.as_millis()).unwrap_or(u64::MAX))
            .unwrap_or(0);

        // Determine parent_id from the most recent snapshot (any tag)
        let parent_id = self
            .latest_snapshot(None)?
            .map(|s| s.snap_id);

        let snapshot = Snapshot {
            snap_id,
            parent_id,
            session_id: self.session_id.clone(),
            fsm_state: fsm_state.to_owned(),
            phase: None,
            step: None,
            findings: None,
            code_changes: None,
            tests_written: None,
            commit_hashes: Vec::new(),
            created_at,
            tag: Some("handoff".to_owned()),
        };

        self.save_snapshot(&snapshot, &trimmed)?;
        Ok(snapshot)
    }

    /// Loads the most relevant handoff snapshot and its context messages.
    ///
    /// Priority: if active branches exist, returns the first branch tip.
    /// Otherwise returns the latest snapshot tagged "handoff".
    /// Returns `Ok(None)` if no handoff snapshot exists and no branches are active.
    pub fn load_handoff(
        &self,
    ) -> Result<Option<(Snapshot, Vec<TrimmedMessage>)>, SessionDagError> {
        // Branch-tip-first: if active branches exist, load the first branch tip
        if let Some(branch) = self.active_branches.first() {
            let pair = self.load_snapshot(&branch.tip_snap_id)?;
            return Ok(Some(pair));
        }

        // Fall back to latest "handoff" tagged snapshot
        match self.latest_snapshot(Some("handoff"))? {
            Some(snap) => {
                let pair = self.load_snapshot(&snap.snap_id)?;
                Ok(Some(pair))
            }
            None => Ok(None),
        }
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

        let start = std::time::Instant::now(); // TEST_KEEP: wall-clock perf gate for batch insert
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

    // ------------------------------------------------------------------
    // 3H.3 trim_context tests
    // ------------------------------------------------------------------

    fn default_trim_config() -> TrimConfig {
        TrimConfig {
            stub_threshold: 500,
            model_context_window: 100_000,
            allocated_slots_tokens: 10_000,
            output_reserve: 5_000,
        }
    }

    fn make_tool_message(seq: u32, content_bytes: usize, ssb_seq: Option<u64>) -> TrimmedMessage {
        TrimmedMessage {
            seq,
            role: DagMessageRole::Tool,
            content: vec![0xAA; content_bytes],
            is_stubbed: false,
            stub_ssb_seq: ssb_seq,
        }
    }

    #[test]
    fn trim_no_stubs_needed() {
        let dir = test_dir();
        let path = db_path(&dir);
        let mut entropy = SeededRng::new(42);
        let clock = OsClock;
        let dag = SessionDag::new("s-test", &path, &mut entropy, &clock).unwrap();

        let config = default_trim_config();
        // All messages small enough (100 bytes = 25 tokens < 500 threshold)
        let msgs = vec![
            make_message(1, DagMessageRole::System, b"system prompt"),
            make_message(4, DagMessageRole::User, b"hello"),
            make_tool_message(5, 100, Some(10)),
        ];
        let valid_seqs: HashSet<u64> = [10].into_iter().collect();

        let result = dag.trim_context(&msgs, config, &valid_seqs).unwrap();
        assert_eq!(result.len(), 3);
        assert!(!result[2].is_stubbed, "small tool message should not be stubbed");
        assert_eq!(result[2].content, msgs[2].content, "content should be identical");
    }

    #[test]
    fn trim_tool_result_stubbed() {
        let dir = test_dir();
        let path = db_path(&dir);
        let mut entropy = SeededRng::new(42);
        let clock = OsClock;
        let dag = SessionDag::new("s-test", &path, &mut entropy, &clock).unwrap();

        let config = default_trim_config();
        // 4000 bytes = 1000 tokens > 500 threshold
        let msgs = vec![
            make_message(1, DagMessageRole::System, b"sys"),
            make_tool_message(4, 4000, Some(42)),
        ];
        let valid_seqs: HashSet<u64> = [42].into_iter().collect();

        let result = dag.trim_context(&msgs, config, &valid_seqs).unwrap();
        assert!(result[1].is_stubbed, "large tool message should be stubbed");
        assert_eq!(result[1].stub_ssb_seq, Some(42));

        // Verify stub content is valid msgpack
        let stub: Stub = rmp_serde::from_slice(&result[1].content).unwrap();
        assert_eq!(stub.original_type, "tool_result");
        assert_eq!(stub.original_tokens, 1000); // 4000 bytes / 4
        assert_eq!(stub.ssb_seq, 42);
    }

    #[test]
    fn trim_user_assistant_never_stubbed() {
        let dir = test_dir();
        let path = db_path(&dir);
        let mut entropy = SeededRng::new(42);
        let clock = OsClock;
        let dag = SessionDag::new("s-test", &path, &mut entropy, &clock).unwrap();

        let config = TrimConfig {
            stub_threshold: 100,
            model_context_window: 1_000_000,
            allocated_slots_tokens: 0,
            output_reserve: 0,
        };
        // User and Assistant messages well above threshold
        let msgs = vec![
            TrimmedMessage {
                seq: 4,
                role: DagMessageRole::User,
                content: vec![0xBB; 5000],
                is_stubbed: false,
                stub_ssb_seq: None,
            },
            TrimmedMessage {
                seq: 5,
                role: DagMessageRole::Assistant,
                content: vec![0xCC; 5000],
                is_stubbed: false,
                stub_ssb_seq: None,
            },
        ];
        let valid_seqs: HashSet<u64> = HashSet::new();

        let result = dag.trim_context(&msgs, config, &valid_seqs).unwrap();
        assert!(!result[0].is_stubbed, "User messages must never be stubbed");
        assert!(!result[1].is_stubbed, "Assistant messages must never be stubbed");
    }

    #[test]
    fn trim_slot13_never_stubbed() {
        let dir = test_dir();
        let path = db_path(&dir);
        let mut entropy = SeededRng::new(42);
        let clock = OsClock;
        let dag = SessionDag::new("s-test", &path, &mut entropy, &clock).unwrap();

        let config = TrimConfig {
            stub_threshold: 100,
            model_context_window: 1_000_000,
            allocated_slots_tokens: 0,
            output_reserve: 0,
        };
        // Tool messages in slots 1-3 (seq <= PROMPT_CACHE_PREFIX_BOUNDARY)
        let msgs = vec![
            make_tool_message(1, 5000, Some(10)),
            make_tool_message(2, 5000, Some(11)),
            make_tool_message(3, 5000, Some(12)),
        ];
        let valid_seqs: HashSet<u64> = [10, 11, 12].into_iter().collect();

        let result = dag.trim_context(&msgs, config, &valid_seqs).unwrap();
        for (i, msg) in result.iter().enumerate() {
            assert!(
                !msg.is_stubbed,
                "Slot {} (seq {}) must never be stubbed",
                i + 1,
                msg.seq
            );
        }
    }

    #[test]
    fn trim_stub_ssb_not_in_set() {
        let dir = test_dir();
        let path = db_path(&dir);
        let mut entropy = SeededRng::new(42);
        let clock = OsClock;
        let dag = SessionDag::new("s-test", &path, &mut entropy, &clock).unwrap();

        let config = TrimConfig {
            stub_threshold: 100,
            model_context_window: 1_000_000,
            allocated_slots_tokens: 0,
            output_reserve: 0,
        };
        // Large tool message with ssb_seq=99, but 99 is NOT in valid set
        let original_content = vec![0xAA; 2000];
        let msgs = vec![make_tool_message(5, 2000, Some(99))];
        let valid_seqs: HashSet<u64> = [100, 101].into_iter().collect(); // 99 missing

        let result = dag.trim_context(&msgs, config, &valid_seqs).unwrap();
        assert!(
            !result[0].is_stubbed,
            "message with ssb_seq not in valid set must be unstubbed"
        );
        assert_eq!(
            result[0].content, original_content,
            "original content must be restored"
        );
        assert_eq!(result[0].stub_ssb_seq, Some(99));
    }

    #[test]
    fn trim_budget_exceeded() {
        let dir = test_dir();
        let path = db_path(&dir);
        let mut entropy = SeededRng::new(42);
        let clock = OsClock;
        let dag = SessionDag::new("s-test", &path, &mut entropy, &clock).unwrap();

        // Very tight budget: target = 100 - 0 - 0 = 100 tokens
        let config = TrimConfig {
            stub_threshold: 100,
            model_context_window: 100,
            allocated_slots_tokens: 0,
            output_reserve: 0,
        };
        // 2000 bytes = 500 tokens, way over budget. ssb_seq not in valid set so unstubbed.
        let msgs = vec![make_tool_message(5, 2000, Some(99))];
        let valid_seqs: HashSet<u64> = HashSet::new(); // empty — will unstub

        let result = dag.trim_context(&msgs, config, &valid_seqs);
        assert!(
            matches!(result, Err(SessionDagError::TrimBudgetExceeded { .. })),
            "should return TrimBudgetExceeded when total exceeds target"
        );
    }

    #[test]
    fn trim_threshold_validation() {
        let dir = test_dir();
        let path = db_path(&dir);
        let mut entropy = SeededRng::new(42);
        let clock = OsClock;
        let dag = SessionDag::new("s-test", &path, &mut entropy, &clock).unwrap();

        let msgs: Vec<TrimmedMessage> = Vec::new();
        let valid_seqs: HashSet<u64> = HashSet::new();

        // Below minimum
        let config_low = TrimConfig {
            stub_threshold: 50,
            model_context_window: 100_000,
            allocated_slots_tokens: 0,
            output_reserve: 0,
        };
        assert!(
            matches!(
                dag.trim_context(&msgs, config_low, &valid_seqs),
                Err(SessionDagError::InvalidTrimConfig { .. })
            ),
            "threshold 50 should be rejected"
        );

        // Above maximum
        let config_high = TrimConfig {
            stub_threshold: 1001,
            model_context_window: 100_000,
            allocated_slots_tokens: 0,
            output_reserve: 0,
        };
        assert!(
            matches!(
                dag.trim_context(&msgs, config_high, &valid_seqs),
                Err(SessionDagError::InvalidTrimConfig { .. })
            ),
            "threshold 1001 should be rejected"
        );

        // At minimum boundary
        let config_min = TrimConfig {
            stub_threshold: 100,
            model_context_window: 100_000,
            allocated_slots_tokens: 0,
            output_reserve: 0,
        };
        assert!(
            dag.trim_context(&msgs, config_min, &valid_seqs).is_ok(),
            "threshold 100 should be accepted"
        );

        // At maximum boundary
        let config_max = TrimConfig {
            stub_threshold: 1000,
            model_context_window: 100_000,
            allocated_slots_tokens: 0,
            output_reserve: 0,
        };
        assert!(
            dag.trim_context(&msgs, config_max, &valid_seqs).is_ok(),
            "threshold 1000 should be accepted"
        );
    }

    #[test]
    fn trim_deterministic() {
        let dir = test_dir();
        let path = db_path(&dir);
        let mut entropy = SeededRng::new(42);
        let clock = OsClock;
        let dag = SessionDag::new("s-test", &path, &mut entropy, &clock).unwrap();

        let config = default_trim_config();
        let msgs = vec![
            make_message(1, DagMessageRole::System, b"system"),
            make_message(2, DagMessageRole::User, b"query"),
            make_tool_message(4, 4000, Some(50)),
            make_tool_message(5, 3000, Some(51)),
        ];
        let valid_seqs: HashSet<u64> = [50, 51].into_iter().collect();

        let result1 = dag.trim_context(&msgs, config, &valid_seqs).unwrap();
        let result2 = dag.trim_context(&msgs, config, &valid_seqs).unwrap();

        assert_eq!(result1.len(), result2.len());
        for (a, b) in result1.iter().zip(result2.iter()) {
            assert_eq!(a.content, b.content, "outputs must be byte-identical");
            assert_eq!(a.is_stubbed, b.is_stubbed);
            assert_eq!(a.stub_ssb_seq, b.stub_ssb_seq);
        }
    }

    #[test]
    fn trim_summary_hash_correct() {
        let dir = test_dir();
        let path = db_path(&dir);
        let mut entropy = SeededRng::new(42);
        let clock = OsClock;
        let dag = SessionDag::new("s-test", &path, &mut entropy, &clock).unwrap();

        let config = default_trim_config();
        let original_content = vec![0xEE; 4000];
        let expected_hash: [u8; 32] = blake3::hash(&original_content).into();

        let msgs = vec![TrimmedMessage {
            seq: 5,
            role: DagMessageRole::Tool,
            content: original_content,
            is_stubbed: false,
            stub_ssb_seq: Some(77),
        }];
        let valid_seqs: HashSet<u64> = [77].into_iter().collect();

        let result = dag.trim_context(&msgs, config, &valid_seqs).unwrap();
        assert!(result[0].is_stubbed);

        let stub: Stub = rmp_serde::from_slice(&result[0].content).unwrap();
        assert_eq!(
            stub.summary_hash, expected_hash,
            "summary_hash must equal blake3::hash(original_content)"
        );
    }

    #[test]
    fn trim_already_stubbed_not_double_stubbed() {
        let dir = test_dir();
        let path = db_path(&dir);
        let mut entropy = SeededRng::new(42);
        let clock = OsClock;
        let dag = SessionDag::new("s-test", &path, &mut entropy, &clock).unwrap();

        let config = TrimConfig {
            stub_threshold: 100,
            model_context_window: 1_000_000,
            allocated_slots_tokens: 0,
            output_reserve: 0,
        };

        // Pre-stubbed message (e.g., from a prior trim_context pass)
        let stub = Stub {
            original_type: "tool_result".to_owned(),
            original_tokens: 2000,
            summary_hash: [0xAB; 32],
            ssb_seq: 55,
        };
        let stub_bytes = rmp_serde::to_vec(&stub).unwrap();

        let msgs = vec![TrimmedMessage {
            seq: 5,
            role: DagMessageRole::Tool,
            content: stub_bytes.clone(),
            is_stubbed: true,
            stub_ssb_seq: Some(55),
        }];
        let valid_seqs: HashSet<u64> = [55].into_iter().collect();

        let result = dag.trim_context(&msgs, config, &valid_seqs).unwrap();
        assert!(result[0].is_stubbed, "should remain stubbed");
        assert_eq!(
            result[0].content, stub_bytes,
            "stub content must not be re-wrapped"
        );
        // Verify the original stub is still decodable
        let decoded: Stub = rmp_serde::from_slice(&result[0].content).unwrap();
        assert_eq!(decoded.ssb_seq, 55);
        assert_eq!(decoded.original_tokens, 2000);
    }

    // ------------------------------------------------------------------
    // 3H.4 — Branch operations tests
    // ------------------------------------------------------------------

    fn setup_dag_with_root() -> (TempDir, SessionDag, String) {
        let dir = test_dir();
        let path = db_path(&dir);
        let mut entropy = SeededRng::new(100);
        let clock = OsClock;
        let dag = SessionDag::new("s-branch", &path, &mut entropy, &clock).unwrap();

        let root_id = SessionDag::generate_snap_id(&mut SeededRng::new(200));
        let root = make_snapshot(&root_id, None, "s-branch", 1000, None);
        dag.save_snapshot(&root, &[]).unwrap();

        (dir, dag, root_id)
    }

    #[test]
    fn create_branch_basic() {
        let (_dir, mut dag, root_id) = setup_dag_with_root();
        let mut entropy = SeededRng::new(300);
        let clock = OsClock;

        let branch = dag
            .create_branch("explore-1", &root_id, &mut entropy, &clock)
            .unwrap();

        assert_eq!(branch.name, "explore-1");
        assert!(!branch.tip_snap_id.is_empty());
        assert_ne!(branch.tip_snap_id, root_id);
        assert_eq!(dag.active_branches().len(), 1);
        assert_eq!(dag.active_branches()[0].name, "explore-1");
    }

    #[test]
    fn create_branch_persists_snapshot_with_tag() {
        let (_dir, mut dag, root_id) = setup_dag_with_root();
        let mut entropy = SeededRng::new(301);
        let clock = OsClock;

        let branch = dag
            .create_branch("tagged-1", &root_id, &mut entropy, &clock)
            .unwrap();

        let tag: String = dag
            .db()
            .query_row(
                "SELECT tag FROM snapshots WHERE snap_id = ?1",
                params![branch.tip_snap_id],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(tag, "branch:tagged-1");
    }

    #[test]
    fn create_branch_copies_parent_context() {
        let dir = test_dir();
        let path = db_path(&dir);
        let mut entropy = SeededRng::new(100);
        let clock = OsClock;
        let mut dag = SessionDag::new("s-ctx", &path, &mut entropy, &clock).unwrap();

        let root_id = SessionDag::generate_snap_id(&mut SeededRng::new(200));
        let root = make_snapshot(&root_id, None, "s-ctx", 1000, None);
        let msgs = vec![
            make_message(0, DagMessageRole::User, b"hello"),
            make_message(1, DagMessageRole::Assistant, b"world"),
        ];
        dag.save_snapshot(&root, &msgs).unwrap();

        let branch = dag
            .create_branch("ctx-copy", &root_id, &mut SeededRng::new(301), &clock)
            .unwrap();

        let ctx_count: i64 = dag
            .db()
            .query_row(
                "SELECT COUNT(*) FROM snapshot_context WHERE snap_id = ?1",
                params![branch.tip_snap_id],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(ctx_count, 2, "branch snapshot should copy parent's 2 context messages");
    }

    #[test]
    fn create_branch_invalid_name_empty() {
        let (_dir, mut dag, root_id) = setup_dag_with_root();
        let mut entropy = SeededRng::new(300);
        let clock = OsClock;

        let err = dag.create_branch("", &root_id, &mut entropy, &clock);
        assert!(matches!(err, Err(SessionDagError::InvalidBranchName)));
    }

    #[test]
    fn create_branch_invalid_name_special_chars() {
        let (_dir, mut dag, root_id) = setup_dag_with_root();
        let mut entropy = SeededRng::new(300);
        let clock = OsClock;

        let err = dag.create_branch("bad name!", &root_id, &mut entropy, &clock);
        assert!(matches!(err, Err(SessionDagError::InvalidBranchName)));
    }

    #[test]
    fn create_branch_invalid_name_too_long() {
        let (_dir, mut dag, root_id) = setup_dag_with_root();
        let mut entropy = SeededRng::new(300);
        let clock = OsClock;

        let long_name = "a".repeat(65);
        let err = dag.create_branch(&long_name, &root_id, &mut entropy, &clock);
        assert!(matches!(err, Err(SessionDagError::InvalidBranchName)));
    }

    #[test]
    fn create_branch_valid_name_64_chars() {
        let (_dir, mut dag, root_id) = setup_dag_with_root();
        let mut entropy = SeededRng::new(300);
        let clock = OsClock;

        let name_64 = "a".repeat(64);
        let branch = dag.create_branch(&name_64, &root_id, &mut entropy, &clock);
        assert!(branch.is_ok(), "64-char name should be valid");
    }

    #[test]
    fn create_branch_duplicate_name_rejected() {
        let (_dir, mut dag, root_id) = setup_dag_with_root();
        let mut entropy = SeededRng::new(300);
        let clock = OsClock;

        dag.create_branch("dup", &root_id, &mut entropy, &clock).unwrap();
        let err = dag.create_branch("dup", &root_id, &mut entropy, &clock);
        assert!(matches!(err, Err(SessionDagError::InvalidBranchName)));
    }

    #[test]
    fn create_branch_limit_enforced() {
        let (_dir, mut dag, root_id) = setup_dag_with_root();
        let mut entropy = SeededRng::new(300);
        let clock = OsClock;

        dag.create_branch("b1", &root_id, &mut entropy, &clock).unwrap();
        dag.create_branch("b2", &root_id, &mut entropy, &clock).unwrap();
        dag.create_branch("b3", &root_id, &mut entropy, &clock).unwrap();

        let err = dag.create_branch("b4", &root_id, &mut entropy, &clock);
        assert!(
            matches!(err, Err(SessionDagError::BranchLimitExceeded { max: 3 })),
            "4th branch should be rejected"
        );
    }

    #[test]
    fn create_branch_nonexistent_parent() {
        let (_dir, mut dag, _root_id) = setup_dag_with_root();
        let mut entropy = SeededRng::new(300);
        let clock = OsClock;

        let err = dag.create_branch("orphan", "nonexistent-snap", &mut entropy, &clock);
        assert!(matches!(err, Err(SessionDagError::SnapshotNotFound)));
    }

    #[test]
    fn list_branches_returns_all_active() {
        let (_dir, mut dag, root_id) = setup_dag_with_root();
        let mut entropy = SeededRng::new(300);
        let clock = OsClock;

        assert!(dag.list_branches().is_empty());

        dag.create_branch("a", &root_id, &mut entropy, &clock).unwrap();
        dag.create_branch("b", &root_id, &mut entropy, &clock).unwrap();

        let branches = dag.list_branches();
        assert_eq!(branches.len(), 2);
        assert_eq!(branches[0].name, "a");
        assert_eq!(branches[1].name, "b");
    }

    #[test]
    fn merge_branch_winner_promoted() {
        let (_dir, mut dag, root_id) = setup_dag_with_root();
        let mut entropy = SeededRng::new(300);
        let clock = OsClock;

        dag.create_branch("winner", &root_id, &mut entropy, &clock).unwrap();
        dag.create_branch("loser", &root_id, &mut entropy, &clock).unwrap();

        let result = dag.merge_branch("winner", &mut entropy, &clock).unwrap();

        match result {
            MergeResult::Winner { branch, abandoned } => {
                assert_eq!(branch.name, "winner");
                assert_eq!(abandoned.len(), 1);
                assert_eq!(abandoned[0].name, "loser");
            }
            MergeResult::Conflict { reason } => {
                panic!("unexpected conflict: {reason}");
            }
        }

        assert!(
            dag.active_branches().is_empty(),
            "active_branches must be empty after merge"
        );
    }

    #[test]
    fn merge_branch_creates_trunk_snapshot() {
        let (_dir, mut dag, root_id) = setup_dag_with_root();
        let mut entropy = SeededRng::new(300);
        let clock = OsClock;

        let winner_branch = dag
            .create_branch("w", &root_id, &mut entropy, &clock)
            .unwrap();

        let result = dag.merge_branch("w", &mut entropy, &clock).unwrap();
        let new_snap_id = match &result {
            MergeResult::Winner { branch, .. } => &branch.tip_snap_id,
            _ => panic!("expected Winner"),
        };

        // Verify trunk snapshot exists with NULL tag
        let tag: Option<String> = dag
            .db()
            .query_row(
                "SELECT tag FROM snapshots WHERE snap_id = ?1",
                params![new_snap_id],
                |row| row.get(0),
            )
            .unwrap();
        assert!(tag.is_none(), "trunk snapshot should have NULL tag");

        // Verify parent_id is the winner's tip
        let parent: String = dag
            .db()
            .query_row(
                "SELECT parent_id FROM snapshots WHERE snap_id = ?1",
                params![new_snap_id],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(parent, winner_branch.tip_snap_id);
    }

    #[test]
    fn merge_branch_tags_abandoned() {
        let (_dir, mut dag, root_id) = setup_dag_with_root();
        let mut entropy = SeededRng::new(300);
        let clock = OsClock;

        dag.create_branch("keep", &root_id, &mut entropy, &clock).unwrap();
        let loser = dag
            .create_branch("drop", &root_id, &mut entropy, &clock)
            .unwrap();

        dag.merge_branch("keep", &mut entropy, &clock).unwrap();

        // The loser's snapshot should be tagged abandoned
        let tag: String = dag
            .db()
            .query_row(
                "SELECT tag FROM snapshots WHERE snap_id = ?1",
                params![loser.tip_snap_id],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(tag, "abandoned:drop");
    }

    #[test]
    fn merge_branch_abandoned_snapshots_still_in_db() {
        let (_dir, mut dag, root_id) = setup_dag_with_root();
        let mut entropy = SeededRng::new(300);
        let clock = OsClock;

        let loser = dag
            .create_branch("gone", &root_id, &mut entropy, &clock)
            .unwrap();
        dag.create_branch("stay", &root_id, &mut entropy, &clock).unwrap();

        dag.merge_branch("stay", &mut entropy, &clock).unwrap();

        // Abandoned snapshot must still exist
        let count: i64 = dag
            .db()
            .query_row(
                "SELECT COUNT(*) FROM snapshots WHERE snap_id = ?1",
                params![loser.tip_snap_id],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(count, 1, "abandoned snapshot must remain in DB");
    }

    #[test]
    fn merge_branch_not_found() {
        let (_dir, mut dag, root_id) = setup_dag_with_root();
        let mut entropy = SeededRng::new(300);
        let clock = OsClock;

        dag.create_branch("only", &root_id, &mut entropy, &clock).unwrap();

        let err = dag.merge_branch("nonexistent", &mut entropy, &clock);
        assert!(matches!(err, Err(SessionDagError::BranchNotFound)));
    }

    #[test]
    fn merge_branch_single_branch_no_conflict() {
        let (_dir, mut dag, root_id) = setup_dag_with_root();
        let mut entropy = SeededRng::new(300);
        let clock = OsClock;

        dag.create_branch("solo", &root_id, &mut entropy, &clock).unwrap();

        let result = dag.merge_branch("solo", &mut entropy, &clock).unwrap();
        match result {
            MergeResult::Winner { branch, abandoned } => {
                assert_eq!(branch.name, "solo");
                assert!(abandoned.is_empty());
            }
            _ => panic!("single branch merge should be Winner with no abandoned"),
        }
    }

    #[test]
    fn merge_branch_conflict_detection() {
        let dir = test_dir();
        let path = db_path(&dir);
        let mut entropy = SeededRng::new(100);
        let clock = OsClock;
        let mut dag = SessionDag::new("s-conflict", &path, &mut entropy, &clock).unwrap();

        // Create root with code_changes
        let root_id = SessionDag::generate_snap_id(&mut SeededRng::new(200));
        let root = make_snapshot(&root_id, None, "s-conflict", 1000, None);
        dag.save_snapshot(&root, &[]).unwrap();

        // Create two branches
        let b1 = dag
            .create_branch("b1", &root_id, &mut SeededRng::new(301), &clock)
            .unwrap();
        let b2 = dag
            .create_branch("b2", &root_id, &mut SeededRng::new(302), &clock)
            .unwrap();

        // Set overlapping code_changes on both branch tip snapshots
        let changes = serde_json::json!({"src/main.rs": "changed"});
        let changes_bytes = serde_json::to_vec(&changes).unwrap();
        dag.db()
            .execute(
                "UPDATE snapshots SET code_changes = ?1 WHERE snap_id = ?2",
                params![changes_bytes, b1.tip_snap_id],
            )
            .unwrap();
        dag.db()
            .execute(
                "UPDATE snapshots SET code_changes = ?1 WHERE snap_id = ?2",
                params![changes_bytes, b2.tip_snap_id],
            )
            .unwrap();

        let result = dag
            .merge_branch("b1", &mut SeededRng::new(400), &clock)
            .unwrap();
        assert!(
            matches!(result, MergeResult::Conflict { .. }),
            "overlapping code_changes should return Conflict"
        );
    }

    #[test]
    fn merge_branch_no_conflict_disjoint_changes() {
        let dir = test_dir();
        let path = db_path(&dir);
        let mut entropy = SeededRng::new(100);
        let clock = OsClock;
        let mut dag = SessionDag::new("s-disjoint", &path, &mut entropy, &clock).unwrap();

        let root_id = SessionDag::generate_snap_id(&mut SeededRng::new(200));
        let root = make_snapshot(&root_id, None, "s-disjoint", 1000, None);
        dag.save_snapshot(&root, &[]).unwrap();

        let b1 = dag
            .create_branch("b1", &root_id, &mut SeededRng::new(301), &clock)
            .unwrap();
        let b2 = dag
            .create_branch("b2", &root_id, &mut SeededRng::new(302), &clock)
            .unwrap();

        // Disjoint code_changes
        let changes1 = serde_json::json!({"src/a.rs": "changed"});
        let changes2 = serde_json::json!({"src/b.rs": "changed"});
        dag.db()
            .execute(
                "UPDATE snapshots SET code_changes = ?1 WHERE snap_id = ?2",
                params![serde_json::to_vec(&changes1).unwrap(), b1.tip_snap_id],
            )
            .unwrap();
        dag.db()
            .execute(
                "UPDATE snapshots SET code_changes = ?1 WHERE snap_id = ?2",
                params![serde_json::to_vec(&changes2).unwrap(), b2.tip_snap_id],
            )
            .unwrap();

        let result = dag
            .merge_branch("b1", &mut SeededRng::new(400), &clock)
            .unwrap();
        assert!(
            matches!(result, MergeResult::Winner { .. }),
            "disjoint code_changes should not conflict"
        );
    }

    #[test]
    fn close_branch_removes_from_active() {
        let (_dir, mut dag, root_id) = setup_dag_with_root();
        let mut entropy = SeededRng::new(300);
        let clock = OsClock;

        dag.create_branch("temp", &root_id, &mut entropy, &clock).unwrap();
        assert_eq!(dag.active_branches().len(), 1);

        dag.close_branch("temp").unwrap();
        assert!(dag.active_branches().is_empty());
    }

    #[test]
    fn close_branch_tags_snapshots_abandoned() {
        let (_dir, mut dag, root_id) = setup_dag_with_root();
        let mut entropy = SeededRng::new(300);
        let clock = OsClock;

        let branch = dag
            .create_branch("closed", &root_id, &mut entropy, &clock)
            .unwrap();

        dag.close_branch("closed").unwrap();

        let tag: String = dag
            .db()
            .query_row(
                "SELECT tag FROM snapshots WHERE snap_id = ?1",
                params![branch.tip_snap_id],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(tag, "abandoned:closed");
    }

    #[test]
    fn close_branch_not_found() {
        let (_dir, mut dag, _root_id) = setup_dag_with_root();

        let err = dag.close_branch("ghost");
        assert!(matches!(err, Err(SessionDagError::BranchNotFound)));
    }

    #[test]
    fn close_branch_preserves_snapshot_in_db() {
        let (_dir, mut dag, root_id) = setup_dag_with_root();
        let mut entropy = SeededRng::new(300);
        let clock = OsClock;

        let branch = dag
            .create_branch("preserve", &root_id, &mut entropy, &clock)
            .unwrap();

        dag.close_branch("preserve").unwrap();

        let count: i64 = dag
            .db()
            .query_row(
                "SELECT COUNT(*) FROM snapshots WHERE snap_id = ?1",
                params![branch.tip_snap_id],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(count, 1, "closed branch snapshot must remain in DB");
    }

    #[test]
    fn branch_name_validation_alphanumeric_and_hyphens() {
        assert!(SessionDag::is_valid_branch_name("explore-1"));
        assert!(SessionDag::is_valid_branch_name("abc"));
        assert!(SessionDag::is_valid_branch_name("a-b-c-123"));
        assert!(!SessionDag::is_valid_branch_name(""));
        assert!(!SessionDag::is_valid_branch_name("has space"));
        assert!(!SessionDag::is_valid_branch_name("under_score"));
        assert!(!SessionDag::is_valid_branch_name("dot.name"));
        assert!(!SessionDag::is_valid_branch_name("slash/name"));
        assert!(!SessionDag::is_valid_branch_name(&"x".repeat(65)));
        assert!(SessionDag::is_valid_branch_name(&"x".repeat(64)));
    }

    #[test]
    fn merge_branch_empty_active_branches() {
        let (_dir, mut dag, _root_id) = setup_dag_with_root();
        let mut entropy = SeededRng::new(300);
        let clock = OsClock;

        let err = dag.merge_branch("nobody", &mut entropy, &clock);
        assert!(matches!(err, Err(SessionDagError::BranchNotFound)));
    }

    #[test]
    fn create_branch_after_close_reuses_slot() {
        let (_dir, mut dag, root_id) = setup_dag_with_root();
        let mut entropy = SeededRng::new(300);
        let clock = OsClock;

        dag.create_branch("b1", &root_id, &mut entropy, &clock).unwrap();
        dag.create_branch("b2", &root_id, &mut entropy, &clock).unwrap();
        dag.create_branch("b3", &root_id, &mut entropy, &clock).unwrap();

        // At limit — close one and create again
        dag.close_branch("b2").unwrap();
        let result = dag.create_branch("b4", &root_id, &mut entropy, &clock);
        assert!(result.is_ok(), "should succeed after closing a branch");
        assert_eq!(dag.active_branches().len(), 3);
    }

    #[test]
    fn open_restores_active_branches() {
        let dir = test_dir();
        let path = db_path(&dir);
        let mut entropy = SeededRng::new(100);
        let clock = OsClock;

        // Create DAG and add branches
        {
            let mut dag = SessionDag::new("s-persist", &path, &mut entropy, &clock).unwrap();
            let root_id = SessionDag::generate_snap_id(&mut SeededRng::new(200));
            let root = make_snapshot(&root_id, None, "s-persist", 1000, None);
            dag.save_snapshot(&root, &[]).unwrap();

            dag.create_branch("alpha", &root_id, &mut SeededRng::new(300), &clock)
                .unwrap();
            dag.create_branch("beta", &root_id, &mut SeededRng::new(301), &clock)
                .unwrap();
            assert_eq!(dag.active_branches().len(), 2);
        }

        // Re-open and verify branches are restored
        let reopened = SessionDag::open("s-persist", &path).unwrap();
        assert_eq!(reopened.active_branches().len(), 2);
        assert_eq!(reopened.active_branches()[0].name, "alpha");
        assert_eq!(reopened.active_branches()[1].name, "beta");
    }

    #[test]
    fn snapshot_immutability_no_parent_update() {
        let (_dir, mut dag, root_id) = setup_dag_with_root();
        let mut entropy = SeededRng::new(300);
        let clock = OsClock;

        let branch = dag
            .create_branch("immut", &root_id, &mut entropy, &clock)
            .unwrap();

        // Verify parent_id of branch snapshot is the root
        let parent: String = dag
            .db()
            .query_row(
                "SELECT parent_id FROM snapshots WHERE snap_id = ?1",
                params![branch.tip_snap_id],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(parent, root_id, "parent_id should point to from_snap_id");

        // After merge, the original branch snapshot's parent_id must not change
        dag.merge_branch("immut", &mut entropy, &clock).unwrap();

        let parent_after: String = dag
            .db()
            .query_row(
                "SELECT parent_id FROM snapshots WHERE snap_id = ?1",
                params![branch.tip_snap_id],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(
            parent_after, root_id,
            "parent_id must not be mutated after merge"
        );
    }

    // ------------------------------------------------------------------
    // 3H.6 — Handoff tests
    // ------------------------------------------------------------------

    use crate::ssb::{SsbActionType, SsbEntry, SsbError, SsbReader};
    use std::collections::HashMap;
    use std::sync::Arc;

    struct MockSsbReader {
        entries: HashMap<i64, SsbEntry>,
    }

    impl MockSsbReader {
        fn new() -> Self {
            Self {
                entries: HashMap::new(),
            }
        }
    }

    impl SsbReader for MockSsbReader {
        fn query_by_session(&self, _session_id: &str) -> Result<Vec<SsbEntry>, SsbError> {
            Ok(vec![])
        }
        fn query_by_file(&self, _file_paths: &[&str]) -> Result<Vec<SsbEntry>, SsbError> {
            Ok(vec![])
        }
        fn query_by_type(&self, _action_type: SsbActionType) -> Result<Vec<SsbEntry>, SsbError> {
            Ok(vec![])
        }
        fn query_by_superstep(&self, _superstep: i64) -> Result<Vec<SsbEntry>, SsbError> {
            Ok(vec![])
        }
        fn query_conflict(
            &self,
            _file_paths: &[&str],
            _session_id: &str,
            _superstep: i64,
        ) -> Result<Vec<String>, SsbError> {
            Ok(vec![])
        }
        fn last_seq(&self) -> Result<Option<i64>, SsbError> {
            Ok(None)
        }
        fn query_by_seq(&self, seq: i64) -> Result<Option<SsbEntry>, SsbError> {
            Ok(self.entries.get(&seq).cloned())
        }
        fn query_reasoning_trace(&self, _agent_id: &str) -> Result<Vec<SsbEntry>, SsbError> {
            Ok(vec![])
        }
        fn query_phase_complete(&self, _phase_id: &str) -> Result<Vec<SsbEntry>, SsbError> {
            Ok(vec![])
        }
        fn query_isolation_metrics(&self) -> Result<Vec<SsbEntry>, SsbError> {
            Ok(vec![])
        }
    }

    #[test]
    fn test_create_handoff_tag() {
        let dir = test_dir();
        let path = db_path(&dir);
        let mut entropy = SeededRng::new(42);
        let clock = OsClock;
        let dag = SessionDag::new("s-ho-tag", &path, &mut entropy, &clock).unwrap();

        let msgs = vec![
            make_message(1, DagMessageRole::System, b"sys"),
            make_message(2, DagMessageRole::User, b"usr"),
        ];
        let config = default_trim_config();
        let ssb = MockSsbReader::new();

        let snap = dag
            .create_handoff(&msgs, config, &ssb, &mut SeededRng::new(100), &clock, "IDLE")
            .unwrap();
        assert_eq!(snap.tag, Some("handoff".to_owned()));
        assert_eq!(snap.session_id, "s-ho-tag");
        assert_eq!(snap.fsm_state, "IDLE");
    }

    #[test]
    fn test_load_handoff_empty_returns_none() {
        let dir = test_dir();
        let path = db_path(&dir);
        let mut entropy = SeededRng::new(42);
        let clock = OsClock;
        let dag = SessionDag::new("s-ho-empty", &path, &mut entropy, &clock).unwrap();

        let result = dag.load_handoff().unwrap();
        assert!(result.is_none(), "empty DAG must return None");
    }

    #[test]
    fn test_load_handoff_no_branch_returns_trunk() {
        let dir = test_dir();
        let path = db_path(&dir);
        let mut entropy = SeededRng::new(42);
        let clock = OsClock;
        let dag = SessionDag::new("s-ho-trunk", &path, &mut entropy, &clock).unwrap();

        let msgs = vec![
            make_message(1, DagMessageRole::System, b"sys"),
            make_message(2, DagMessageRole::User, b"usr"),
        ];
        let config = default_trim_config();
        let ssb = MockSsbReader::new();

        let snap = dag
            .create_handoff(&msgs, config, &ssb, &mut SeededRng::new(100), &clock, "PLANNING")
            .unwrap();

        let loaded = dag.load_handoff().unwrap();
        assert!(loaded.is_some());
        let (ls, lm) = loaded.unwrap();
        assert_eq!(ls.snap_id, snap.snap_id);
        assert_eq!(lm.len(), 2);
    }

    #[test]
    fn test_load_handoff_with_active_branch_returns_branch_tip() {
        let dir = test_dir();
        let path = db_path(&dir);
        let mut entropy = SeededRng::new(42);
        let clock = OsClock;
        let mut dag = SessionDag::new("s-ho-br", &path, &mut entropy, &clock).unwrap();

        let msgs = vec![
            make_message(1, DagMessageRole::System, b"sys"),
            make_message(2, DagMessageRole::User, b"usr"),
        ];
        let config = default_trim_config();
        let ssb = MockSsbReader::new();

        let handoff = dag
            .create_handoff(&msgs, config, &ssb, &mut SeededRng::new(100), &clock, "EXECUTING")
            .unwrap();

        let branch = dag
            .create_branch("test-br", &handoff.snap_id, &mut SeededRng::new(200), &clock)
            .unwrap();

        let loaded = dag.load_handoff().unwrap();
        assert!(loaded.is_some());
        let (ls, _) = loaded.unwrap();
        assert_eq!(
            ls.snap_id, branch.tip_snap_id,
            "must return branch tip, not trunk handoff"
        );
    }

    #[test]
    fn test_handoff_integration_lifecycle() {
        // Step 1: Create SessionDag (new, in-memory-like via tempdir)
        let dir = test_dir();
        let path = db_path(&dir);
        let mut entropy = SeededRng::new(42);
        let clock = OsClock;
        let mut dag = SessionDag::new("s-lifecycle", &path, &mut entropy, &clock).unwrap();

        // Step 2: Build 50 TrimmedMessages (mix of roles, some with content >100 bytes)
        let mut messages: Vec<TrimmedMessage> = Vec::with_capacity(50);
        for i in 0..50u32 {
            let seq = i + 1;
            if seq <= 3 {
                // Prompt cache prefix — never stubbed
                messages.push(TrimmedMessage {
                    seq,
                    role: match seq {
                        1 => DagMessageRole::System,
                        2 => DagMessageRole::User,
                        _ => DagMessageRole::Assistant,
                    },
                    content: format!("prefix-{seq}").into_bytes(),
                    is_stubbed: false,
                    stub_ssb_seq: None,
                });
            } else if seq % 4 == 0 {
                // Tool messages with large content (>stub_threshold to be stubbed)
                let payload = vec![0xCC; 600];
                messages.push(TrimmedMessage {
                    seq,
                    role: DagMessageRole::Tool,
                    content: payload,
                    is_stubbed: false,
                    stub_ssb_seq: Some(seq as u64),
                });
            } else if seq % 4 == 1 {
                messages.push(make_message(seq, DagMessageRole::User, b"user-input"));
            } else if seq % 4 == 2 {
                messages.push(make_message(seq, DagMessageRole::Assistant, b"assistant-reply"));
            } else {
                messages.push(make_message(seq, DagMessageRole::Tool, b"small-tool"));
            }
        }
        assert_eq!(messages.len(), 50);

        // Step 3: Call create_handoff with TrimConfig
        let config = TrimConfig {
            stub_threshold: 500,
            model_context_window: 200_000,
            allocated_slots_tokens: 10_000,
            output_reserve: 5_000,
        };
        let ssb = MockSsbReader::new();

        let handoff_snap = dag
            .create_handoff(
                &messages,
                config,
                &ssb,
                &mut SeededRng::new(100),
                &clock,
                "EXECUTING",
            )
            .unwrap();

        // Step 4: Assert snapshot has tag="handoff"
        assert_eq!(
            handoff_snap.tag,
            Some("handoff".to_owned()),
            "step 4: handoff snapshot must have tag='handoff'"
        );

        // Step 5: load_handoff returns the handoff snapshot
        let loaded = dag.load_handoff().unwrap();
        assert!(loaded.is_some(), "step 5: load_handoff should find snapshot");
        let (loaded_snap, loaded_msgs) = loaded.unwrap();
        assert_eq!(
            loaded_snap.snap_id, handoff_snap.snap_id,
            "step 5: snap_id must match"
        );
        assert!(
            !loaded_msgs.is_empty(),
            "step 5: should have messages"
        );

        // Step 6: Create a branch
        let branch = dag
            .create_branch(
                "feat-x",
                &handoff_snap.snap_id,
                &mut SeededRng::new(200),
                &clock,
            )
            .unwrap();

        // Step 7: Save a new snapshot on the branch
        let branch_snap_id = SessionDag::generate_snap_id(&mut SeededRng::new(300));
        let branch_created_at = clock
            .system_now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| u64::try_from(d.as_millis()).unwrap_or(u64::MAX))
            .unwrap_or(0);
        let branch_snap = Snapshot {
            snap_id: branch_snap_id.clone(),
            parent_id: Some(branch.tip_snap_id.clone()),
            session_id: "s-lifecycle".to_owned(),
            fsm_state: "EXECUTING".to_owned(),
            phase: Some("3H".to_owned()),
            step: Some("6".to_owned()),
            findings: None,
            code_changes: None,
            tests_written: None,
            commit_hashes: vec!["branch-commit-1".to_owned()],
            created_at: branch_created_at,
            tag: Some("branch:feat-x".to_owned()),
        };
        let branch_msgs = vec![
            make_message(1, DagMessageRole::System, b"branch-sys"),
            make_message(2, DagMessageRole::User, b"branch-usr"),
        ];
        dag.save_snapshot(&branch_snap, &branch_msgs).unwrap();

        // Step 8: load_handoff returns branch tip, NOT trunk handoff
        let loaded2 = dag.load_handoff().unwrap();
        assert!(loaded2.is_some(), "step 8: should find branch tip");
        let (ls2, _) = loaded2.unwrap();
        assert_eq!(
            ls2.snap_id, branch.tip_snap_id,
            "step 8: load_handoff must return branch tip snapshot, not trunk handoff"
        );

        // Step 9: Merge the branch
        let merge_result = dag
            .merge_branch("feat-x", &mut SeededRng::new(400), &clock)
            .unwrap();
        match &merge_result {
            MergeResult::Winner { branch: br, .. } => {
                assert_eq!(br.name, "feat-x", "step 9: merged branch name");
            }
            MergeResult::Conflict { reason } => {
                panic!("step 9: unexpected conflict: {reason}");
            }
        }

        // Step 10: Parent chain acyclicity
        let merge_snap_id = match &merge_result {
            MergeResult::Winner { branch: br, .. } => br.tip_snap_id.clone(),
            _ => unreachable!(),
        };
        let chain = dag.parent_chain(&merge_snap_id).unwrap();
        let mut seen = HashSet::new();
        for snap in &chain {
            assert!(
                seen.insert(&snap.snap_id),
                "step 10: cycle detected — snap_id {} appears twice in parent chain",
                snap.snap_id
            );
        }
        assert!(
            chain.len() >= 2,
            "step 10: parent chain should have at least 2 snapshots"
        );

        // Step 11: commit_hashes preserved across round-trip
        // The handoff_snap was saved with default empty commit_hashes.
        // Let's verify branch_snap's commit_hashes survived the round-trip.
        let (reloaded_branch, _) = dag.load_snapshot(&branch_snap_id).unwrap();
        assert_eq!(
            reloaded_branch.commit_hashes,
            vec!["branch-commit-1".to_owned()],
            "step 11: commit_hashes must be preserved across save/load round-trip"
        );
    }
}
