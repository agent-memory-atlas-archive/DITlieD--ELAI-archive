//! Read-only view over the SSB for the dashboard (4K.1).
//!
//! [`SsbViewer`] opens the SSB SQLite database with `SQLITE_OPEN_READ_ONLY` —
//! the SQLite engine refuses any write from this handle, so the dashboard
//! cannot accidentally corrupt agent state even if a query fires an INSERT.
//!
//! The connection is wrapped in `std::sync::Mutex` so the type is `Send + Sync`
//! and can be registered as Tauri managed state. No query holds the mutex
//! across an `.await`, so `std::sync::Mutex` is correct here (using
//! `tokio::sync::Mutex` would add async overhead for no benefit).

use std::path::Path;
use std::sync::Mutex;

use rusqlite::{params, Connection, OpenFlags};
use serde::{Deserialize, Serialize};

use crate::ssb::{
    ssb_action_type_from_str_public, InterventionOption, InterventionRequestPayload,
    InterventionType, SsbActionType, SsbPayload,
};

// ---------------------------------------------------------------------------
// Constants
// ---------------------------------------------------------------------------

/// Maximum bytes lifted from the raw msgpack payload for the `summary` field
/// in [`SsbActionSummary`]. We return printable chars only — binary payloads
/// produce a short stub rather than garbled output.
const SUMMARY_TRUNCATE_CHARS: usize = 100;

/// Lowest seq value that fits in a signed 64-bit integer when converted from u64.
const MAX_SEQ_I64_AS_U64: u64 = i64::MAX as u64;

// ---------------------------------------------------------------------------
// Error type
// ---------------------------------------------------------------------------

/// Errors produced by [`SsbViewer`].
#[derive(Debug, thiserror::Error)]
pub enum SsbViewerError {
    #[error("sqlite error: {0}")]
    Sqlite(#[from] rusqlite::Error),

    #[error("lock poisoned")]
    LockPoisoned,

    #[error("payload decode error: {0}")]
    Decode(#[from] rmp_serde::decode::Error),

    #[error("unknown action type: {0}")]
    UnknownActionType(String),

    #[error("value out of range: {0}")]
    OutOfRange(String),
}

// ---------------------------------------------------------------------------
// DTO types returned to the dashboard
// ---------------------------------------------------------------------------

/// Compact per-action row for dashboard lists.
///
/// `summary` is the first [`SUMMARY_TRUNCATE_CHARS`] printable chars of the
/// raw msgpack bytes. We intentionally do NOT deserialize the full
/// [`SsbPayload`] per row — list views must stay cheap at 10k+ entries.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SsbActionSummary {
    pub seq: i64,
    pub timestamp: i64,
    pub action_type: SsbActionType,
    pub agent_id: String,
    pub summary: String,
}

/// Snapshot of FSM state derived from the latest `phase_transition` row.
///
/// Defined locally: the live [`crate::fsm::FsmState`] is tied to an in-process
/// state machine. The dashboard only reconstructs what the orchestrator told
/// the SSB, which is a string snapshot.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FsmStateSnapshot {
    pub state: String,
    pub phase: String,
}

/// Project/session the viewer is currently inspecting.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SessionInfo {
    pub project_id: String,
    pub session_id: String,
    pub last_seq: i64,
}

/// Progress of one phase, derived from `phase_complete` rows.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PhaseStatus {
    pub phase_id: String,
    pub status: String,
    pub step_count: u32,
    pub completed_count: u32,
}

/// Human-in-the-loop gate still waiting for a response.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct InterventionRequest {
    pub request_id: i64,
    pub timestamp: i64,
    pub step_id: String,
    pub intervention_type: InterventionType,
    pub description: String,
    pub options: Vec<InterventionOption>,
}

/// One file touched inside a session.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FileChange {
    pub path: String,
    pub action: String,
    pub summary: String,
}

/// One step in a reasoning chain (from `reasoning_trace` rows).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ThinkStepSummary {
    pub step_type: String,
    pub content: String,
    pub grounding: String,
    pub confidence: f64,
}

// ---------------------------------------------------------------------------
// SsbViewer
// ---------------------------------------------------------------------------

/// Read-only SSB accessor used by the dashboard (4K.1).
pub struct SsbViewer {
    conn: Mutex<Connection>,
}

impl SsbViewer {
    /// Open the SSB database in read-only mode.
    ///
    /// Uses `SQLITE_OPEN_READ_ONLY` — any INSERT/UPDATE/DELETE executed on
    /// this connection is rejected by SQLite with `readonly database`.
    pub fn open(db_path: &Path) -> Result<Self, SsbViewerError> {
        let conn = Connection::open_with_flags(
            db_path,
            OpenFlags::SQLITE_OPEN_READ_ONLY | OpenFlags::SQLITE_OPEN_NO_MUTEX,
        )?;
        // `query_only` pragma is belt-and-braces on top of the open flag: it
        // makes the read-only intent explicit in the connection handle.
        conn.pragma_update(None, "query_only", "ON")?;
        Ok(Self {
            conn: Mutex::new(conn),
        })
    }

    fn lock(&self) -> Result<std::sync::MutexGuard<'_, Connection>, SsbViewerError> {
        self.conn.lock().map_err(|_| SsbViewerError::LockPoisoned)
    }

    /// Latest `phase_transition` entry converted to a [`FsmStateSnapshot`].
    /// Returns `("IDLE", "")` if the SSB has no phase transitions yet.
    pub fn latest_fsm_state(&self) -> Result<FsmStateSnapshot, SsbViewerError> {
        let conn = self.lock()?;
        let mut stmt = conn.prepare(
            "SELECT payload FROM ssb WHERE action_type = ?1 ORDER BY seq DESC LIMIT 1",
        )?;
        let row: Option<Vec<u8>> = stmt
            .query_row(
                params![SsbActionType::PhaseTransition.as_sql_str()],
                |row| row.get::<_, Vec<u8>>(0),
            )
            .map(Some)
            .or_else(|e| match e {
                rusqlite::Error::QueryReturnedNoRows => Ok(None),
                other => Err(other),
            })?;
        let Some(bytes) = row else {
            return Ok(FsmStateSnapshot {
                state: "IDLE".to_owned(),
                phase: String::new(),
            });
        };
        let payload: SsbPayload = rmp_serde::from_slice(&bytes)?;
        match payload {
            SsbPayload::PhaseTransition(inner) => Ok(FsmStateSnapshot {
                state: inner.to_state,
                phase: inner.evidence_summary,
            }),
            _ => Ok(FsmStateSnapshot {
                state: "IDLE".to_owned(),
                phase: String::new(),
            }),
        }
    }

    /// Most recent session row's project/session identifiers plus last seq.
    pub fn active_session(&self) -> Result<Option<SessionInfo>, SsbViewerError> {
        let conn = self.lock()?;
        let mut stmt = conn.prepare(
            "SELECT project_id, session_id, seq FROM ssb ORDER BY seq DESC LIMIT 1",
        )?;
        let row = stmt
            .query_row([], |row| {
                Ok(SessionInfo {
                    project_id: row.get(0)?,
                    session_id: row.get(1)?,
                    last_seq: row.get(2)?,
                })
            })
            .map(Some)
            .or_else(|e| match e {
                rusqlite::Error::QueryReturnedNoRows => Ok(None),
                other => Err(other),
            })?;
        Ok(row)
    }

    /// Aggregate per-phase progress from `phase_complete` entries.
    pub fn phase_progress(&self) -> Result<Vec<PhaseStatus>, SsbViewerError> {
        let conn = self.lock()?;
        let mut stmt = conn.prepare(
            "SELECT payload FROM ssb WHERE action_type = ?1 ORDER BY seq ASC",
        )?;
        let rows = stmt.query_map(
            params![SsbActionType::PhaseComplete.as_sql_str()],
            |row| row.get::<_, Vec<u8>>(0),
        )?;
        let mut out: Vec<PhaseStatus> = Vec::new();
        for row in rows {
            let bytes = row?;
            let payload: SsbPayload = rmp_serde::from_slice(&bytes)?;
            if let SsbPayload::PhaseComplete(inner) = payload {
                let completed = u32::try_from(inner.test_outcomes.len())
                    .map_err(|e| SsbViewerError::OutOfRange(e.to_string()))?;
                out.push(PhaseStatus {
                    phase_id: inner.phase_id,
                    status: inner.fsm_state,
                    step_count: completed,
                    completed_count: completed,
                });
            }
        }
        Ok(out)
    }

    /// Most recent SSB rows as compact summaries. `summary` is a truncated
    /// slice of the raw msgpack bytes — no full payload deserialization.
    pub fn recent_actions(
        &self,
        limit: usize,
        offset: usize,
    ) -> Result<Vec<SsbActionSummary>, SsbViewerError> {
        let limit_i64 = i64::try_from(limit)
            .map_err(|e| SsbViewerError::OutOfRange(format!("limit: {e}")))?;
        let offset_i64 = i64::try_from(offset)
            .map_err(|e| SsbViewerError::OutOfRange(format!("offset: {e}")))?;
        let conn = self.lock()?;
        let mut stmt = conn.prepare(
            "SELECT seq, created_at, action_type, agent_id, payload \
             FROM ssb ORDER BY seq DESC LIMIT ?1 OFFSET ?2",
        )?;
        let rows = stmt.query_map(params![limit_i64, offset_i64], |row| {
            Ok((
                row.get::<_, i64>(0)?,
                row.get::<_, i64>(1)?,
                row.get::<_, String>(2)?,
                row.get::<_, String>(3)?,
                row.get::<_, Vec<u8>>(4)?,
            ))
        })?;
        let mut out = Vec::new();
        for row in rows {
            let (seq, created_at, at_str, agent_id, payload) = row?;
            let action_type = ssb_action_type_from_str_public(&at_str)
                .map_err(|e| SsbViewerError::UnknownActionType(e.to_string()))?;
            out.push(SsbActionSummary {
                seq,
                timestamp: created_at,
                action_type,
                agent_id,
                summary: summarize_payload(&payload),
            });
        }
        Ok(out)
    }

    /// All actions emitted by a specific agent (newest first).
    pub fn agent_actions(
        &self,
        agent_id: &str,
    ) -> Result<Vec<SsbActionSummary>, SsbViewerError> {
        let conn = self.lock()?;
        let mut stmt = conn.prepare(
            "SELECT seq, created_at, action_type, agent_id, payload \
             FROM ssb WHERE agent_id = ?1 ORDER BY seq DESC",
        )?;
        let rows = stmt.query_map(params![agent_id], |row| {
            Ok((
                row.get::<_, i64>(0)?,
                row.get::<_, i64>(1)?,
                row.get::<_, String>(2)?,
                row.get::<_, String>(3)?,
                row.get::<_, Vec<u8>>(4)?,
            ))
        })?;
        let mut out = Vec::new();
        for row in rows {
            let (seq, created_at, at_str, agent_id, payload) = row?;
            let action_type = ssb_action_type_from_str_public(&at_str)
                .map_err(|e| SsbViewerError::UnknownActionType(e.to_string()))?;
            out.push(SsbActionSummary {
                seq,
                timestamp: created_at,
                action_type,
                agent_id,
                summary: summarize_payload(&payload),
            });
        }
        Ok(out)
    }

    /// All `intervention_request` rows whose `seq` has no later matching
    /// `intervention_response`.
    pub fn pending_interventions(&self) -> Result<Vec<InterventionRequest>, SsbViewerError> {
        let conn = self.lock()?;
        let mut answered: std::collections::HashSet<i64> = std::collections::HashSet::new();
        {
            let mut stmt = conn.prepare(
                "SELECT payload FROM ssb WHERE action_type = ?1",
            )?;
            let rows = stmt.query_map(
                params![SsbActionType::InterventionResponse.as_sql_str()],
                |row| row.get::<_, Vec<u8>>(0),
            )?;
            for row in rows {
                let bytes = row?;
                let payload: SsbPayload = rmp_serde::from_slice(&bytes)?;
                if let SsbPayload::InterventionResponse(resp) = payload {
                    answered.insert(resp.request_id);
                }
            }
        }

        let mut stmt = conn.prepare(
            "SELECT seq, created_at, payload FROM ssb WHERE action_type = ?1 ORDER BY seq ASC",
        )?;
        let rows = stmt.query_map(
            params![SsbActionType::InterventionRequest.as_sql_str()],
            |row| {
                Ok((
                    row.get::<_, i64>(0)?,
                    row.get::<_, i64>(1)?,
                    row.get::<_, Vec<u8>>(2)?,
                ))
            },
        )?;
        let mut out = Vec::new();
        for row in rows {
            let (seq, created_at, bytes) = row?;
            if answered.contains(&seq) {
                continue;
            }
            let payload: SsbPayload = rmp_serde::from_slice(&bytes)?;
            if let SsbPayload::InterventionRequest(InterventionRequestPayload {
                step_id,
                intervention_type,
                description,
                options,
                ..
            }) = payload
            {
                out.push(InterventionRequest {
                    request_id: seq,
                    timestamp: created_at,
                    step_id,
                    intervention_type,
                    description,
                    options,
                });
            }
        }
        Ok(out)
    }

    /// Every file touched in a given `session_id` with a short action tag.
    pub fn file_changes(&self, session_id: &str) -> Result<Vec<FileChange>, SsbViewerError> {
        let conn = self.lock()?;
        let mut stmt = conn.prepare(
            "SELECT f.file_path, s.action_type, s.payload \
             FROM ssb_files f JOIN ssb s ON s.seq = f.seq \
             WHERE s.session_id = ?1 ORDER BY s.seq ASC",
        )?;
        let rows = stmt.query_map(params![session_id], |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, Vec<u8>>(2)?,
            ))
        })?;
        let mut out = Vec::new();
        for row in rows {
            let (path, action, payload) = row?;
            out.push(FileChange {
                path,
                action,
                summary: summarize_payload(&payload),
            });
        }
        Ok(out)
    }

    /// All `reasoning_trace` rows that reference a given `step_id`, ordered
    /// by `seq` ascending. `step_id` matches `topic` in the stored payload
    /// (the orchestrator writes the plan step into `topic`).
    ///
    /// Uses `idx_ssb_action_type` to restrict the scan to reasoning_trace
    /// rows only. A per-topic secondary index lands in 4K.3 to keep this
    /// sub-10ms for sessions with >10k reasoning steps.
    pub fn reasoning_chain(
        &self,
        step_id: &str,
    ) -> Result<Vec<ThinkStepSummary>, SsbViewerError> {
        let conn = self.lock()?;
        let mut stmt = conn.prepare(
            "SELECT payload FROM ssb WHERE action_type = ?1 ORDER BY seq ASC",
        )?;
        let rows = stmt.query_map(
            params![SsbActionType::ReasoningTrace.as_sql_str()],
            |row| row.get::<_, Vec<u8>>(0),
        )?;
        let mut out = Vec::new();
        for row in rows {
            let bytes = row?;
            let payload: SsbPayload = rmp_serde::from_slice(&bytes)?;
            if let SsbPayload::ReasoningTrace(inner) = payload {
                if inner.topic == step_id {
                    out.push(ThinkStepSummary {
                        step_type: inner.step_type,
                        content: inner.content_summary,
                        grounding: inner.grounding_source,
                        confidence: inner.confidence,
                    });
                }
            }
        }
        Ok(out)
    }
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Produce a short UTF-8 summary of a raw msgpack payload without
/// round-tripping through `SsbPayload`. Non-UTF-8 bytes become `U+FFFD`.
fn summarize_payload(bytes: &[u8]) -> String {
    let text = String::from_utf8_lossy(bytes);
    text.chars()
        .filter(|c| !c.is_control())
        .take(SUMMARY_TRUNCATE_CHARS)
        .collect()
}

/// Validate that a `u64` request id fits in `i64` (the native seq width).
pub fn validate_request_id(request_id: u64) -> Result<i64, SsbViewerError> {
    if request_id > MAX_SEQ_I64_AS_U64 {
        return Err(SsbViewerError::OutOfRange(format!(
            "request_id {request_id} exceeds i64::MAX"
        )));
    }
    Ok(request_id as i64)
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use elai_core::clock::SeededClock;
    use tempfile::TempDir;

    use super::*;
    use crate::ssb::{InterventionKeyword, Ssb, SsbConfig};

    fn test_config(dir: &TempDir) -> SsbConfig {
        SsbConfig {
            db_path: dir.path().join("test_ssb.db"),
            pool_size: Some(2),
            project_id: "proj-test".into(),
            session_id: "session-test".into(),
        }
    }

    fn test_clock() -> Arc<dyn elai_core::clock::ClockProvider> {
        Arc::new(SeededClock::new())
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn ssb_viewer_opens_read_only() {
        let dir = TempDir::new().expect("tempdir");
        let config = test_config(&dir);
        let db_path = config.db_path.clone();
        // Create the DB via the writer path so schema + pragmas are applied.
        let ssb = Ssb::new(config, test_clock()).expect("ssb init");
        drop(ssb);

        let viewer = SsbViewer::open(&db_path).expect("viewer open");
        let conn = viewer.conn.lock().expect("lock");
        // SQLite will reject the INSERT on a read-only connection.
        let err = conn
            .execute(
                "INSERT INTO ssb (project_id, session_id, agent_id, superstep, \
                 action_type, payload, created_at) VALUES \
                 (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
                params!["p", "s", "a", 0i64, "file_write", &[0u8][..], 0i64],
            )
            .unwrap_err();
        let msg = err.to_string();
        assert!(
            msg.contains("readonly") || msg.contains("read-only") || msg.contains("read only"),
            "expected readonly error, got: {msg}"
        );
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn intervention_round_trip() {
        let dir = TempDir::new().expect("tempdir");
        let config = test_config(&dir);
        let db_path = config.db_path.clone();
        let ssb = Ssb::new(config, test_clock()).expect("ssb init");

        let payload = SsbPayload::InterventionRequest(InterventionRequestPayload {
            step_id: "step-42".into(),
            intervention_type: InterventionType::ApproveChange,
            description: "approve?".into(),
            diff_summary: None,
            options: vec![InterventionOption::Keyword(InterventionKeyword::Accept)],
        });
        let seq = ssb
            .write_payload("agent-0", 0, payload, vec![])
            .await
            .expect("write intervention_request");
        assert!(seq > 0);
        drop(ssb);

        let viewer = SsbViewer::open(&db_path).expect("viewer open");
        let pending = viewer.pending_interventions().expect("pending");
        assert_eq!(pending.len(), 1, "one unanswered intervention");
        assert_eq!(pending[0].step_id, "step-42");
        assert_eq!(
            pending[0].intervention_type,
            InterventionType::ApproveChange
        );
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn recent_actions_summary_is_truncated() {
        let dir = TempDir::new().expect("tempdir");
        let config = test_config(&dir);
        let db_path = config.db_path.clone();
        let ssb = Ssb::new(config, test_clock()).expect("ssb init");

        let big_description = "x".repeat(5_000);
        let payload = SsbPayload::InterventionRequest(InterventionRequestPayload {
            step_id: "step-big".into(),
            intervention_type: InterventionType::ApproveChange,
            description: big_description,
            diff_summary: None,
            options: vec![InterventionOption::Keyword(InterventionKeyword::Accept)],
        });
        ssb.write_payload("agent-big", 0, payload, vec![])
            .await
            .expect("write big");
        drop(ssb);

        let viewer = SsbViewer::open(&db_path).expect("viewer open");
        let rows = viewer.recent_actions(10, 0).expect("recent");
        assert!(!rows.is_empty(), "at least one row");
        for row in &rows {
            assert!(
                row.summary.chars().count() <= SUMMARY_TRUNCATE_CHARS,
                "summary must be truncated, got {} chars",
                row.summary.chars().count()
            );
        }
    }

    #[test]
    fn validate_request_id_rejects_overflow() {
        assert!(validate_request_id(0).is_ok());
        assert!(validate_request_id(i64::MAX as u64).is_ok());
        assert!(validate_request_id((i64::MAX as u64) + 1).is_err());
        assert!(validate_request_id(u64::MAX).is_err());
    }
}
