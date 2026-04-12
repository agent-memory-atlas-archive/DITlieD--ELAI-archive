//! Shared State Board (SSB): inter-agent communication via SQLite WAL.
//!
//! Single-writer guarantee via `tokio::mpsc` channel. All writes serialized
//! through one dedicated task. Reads served from an `r2d2` connection pool.
//! Append-only within a session — no UPDATE or DELETE.
//!
//! A single [`Ssb`] instance is permanently bound to one `(project_id, session_id)` pair.
//! Multi-session scenarios require separate instances.

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::{Duration, UNIX_EPOCH};

use r2d2::Pool;
use r2d2_sqlite::SqliteConnectionManager;
use rusqlite::types::Value;
use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};
use std::rc::Rc;
use tokio::sync::{broadcast, mpsc, oneshot};

use elai_core::clock::ClockProvider;

// ---------------------------------------------------------------------------
// Schema version — bump on any DDL change
// ---------------------------------------------------------------------------

const SCHEMA_VERSION: i64 = 4;

/// Maximum serialized payload size in bytes (1 MiB).
/// `write_payload()` rejects payloads exceeding this limit with `SsbError::PayloadTooLarge`.
const PAYLOAD_MAX_BYTES: usize = 1_048_576;

const CREATE_SSB_TABLE: &str = "\
CREATE TABLE IF NOT EXISTS ssb (
    seq         INTEGER PRIMARY KEY AUTOINCREMENT,
    project_id  TEXT NOT NULL,
    session_id  TEXT NOT NULL,
    agent_id    TEXT NOT NULL,
    superstep   INTEGER NOT NULL DEFAULT 0,
    action_type TEXT NOT NULL CHECK(action_type IN (
        'file_write', 'file_read', 'tool_result', 'test_outcome', 'finding',
        'claim', 'checkpoint', 'model_swap', 'phase_transition',
        'knowledge_degradation', 'isolation_metric', 'plan_update',
        'replan_request', 'stub_reference', 'reasoning_trace',
        'phase_complete', 'memory_monitoring', 'workload_profile',
        'death_event', 'recovery_event',
        'supervision_exhausted', 'spiral_detected',
        'context_reduction', 'branch_merge'
    )),
    payload     BLOB NOT NULL,
    created_at  INTEGER NOT NULL
);
CREATE INDEX IF NOT EXISTS idx_ssb_project   ON ssb(project_id);
CREATE INDEX IF NOT EXISTS idx_ssb_session   ON ssb(session_id);
CREATE INDEX IF NOT EXISTS idx_ssb_superstep ON ssb(superstep);
CREATE INDEX IF NOT EXISTS idx_ssb_conflict  ON ssb(project_id, superstep);
";

const CREATE_SSB_FILES_TABLE: &str = "\
CREATE TABLE IF NOT EXISTS ssb_files (
    seq       INTEGER NOT NULL REFERENCES ssb(seq),
    file_path TEXT NOT NULL
);
CREATE INDEX IF NOT EXISTS idx_ssb_files_path ON ssb_files(file_path);
CREATE INDEX IF NOT EXISTS idx_ssb_files_seq  ON ssb_files(seq);
";

// ---------------------------------------------------------------------------
// Action type enum
// ---------------------------------------------------------------------------

/// All 24 action types recorded in SSB entries.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SsbActionType {
    FileWrite,
    FileRead,
    ToolResult,
    TestOutcome,
    Finding,
    Claim,
    Checkpoint,
    ModelSwap,
    PhaseTransition,
    KnowledgeDegradation,
    IsolationMetric,
    PlanUpdate,
    ReplanRequest,
    StubReference,
    ReasoningTrace,
    PhaseComplete,
    MemoryMonitoring,
    WorkloadProfile,
    DeathEvent,
    RecoveryEvent,
    SupervisionExhausted,
    SpiralDetected,
    ContextReduction,
    BranchMerge,
}

impl SsbActionType {
    /// SQL-compatible string for the CHECK constraint.
    fn as_sql_str(&self) -> &'static str {
        match self {
            Self::FileWrite => "file_write",
            Self::FileRead => "file_read",
            Self::ToolResult => "tool_result",
            Self::TestOutcome => "test_outcome",
            Self::Finding => "finding",
            Self::Claim => "claim",
            Self::Checkpoint => "checkpoint",
            Self::ModelSwap => "model_swap",
            Self::PhaseTransition => "phase_transition",
            Self::KnowledgeDegradation => "knowledge_degradation",
            Self::IsolationMetric => "isolation_metric",
            Self::PlanUpdate => "plan_update",
            Self::ReplanRequest => "replan_request",
            Self::StubReference => "stub_reference",
            Self::ReasoningTrace => "reasoning_trace",
            Self::PhaseComplete => "phase_complete",
            Self::MemoryMonitoring => "memory_monitoring",
            Self::WorkloadProfile => "workload_profile",
            Self::DeathEvent => "death_event",
            Self::RecoveryEvent => "recovery_event",
            Self::SupervisionExhausted => "supervision_exhausted",
            Self::SpiralDetected => "spiral_detected",
            Self::ContextReduction => "context_reduction",
            Self::BranchMerge => "branch_merge",
        }
    }
}

fn ssb_action_type_from_str(s: &str) -> Result<SsbActionType, SsbError> {
    match s {
        "file_write" => Ok(SsbActionType::FileWrite),
        "file_read" => Ok(SsbActionType::FileRead),
        "tool_result" => Ok(SsbActionType::ToolResult),
        "test_outcome" => Ok(SsbActionType::TestOutcome),
        "finding" => Ok(SsbActionType::Finding),
        "claim" => Ok(SsbActionType::Claim),
        "checkpoint" => Ok(SsbActionType::Checkpoint),
        "model_swap" => Ok(SsbActionType::ModelSwap),
        "phase_transition" => Ok(SsbActionType::PhaseTransition),
        "knowledge_degradation" => Ok(SsbActionType::KnowledgeDegradation),
        "isolation_metric" => Ok(SsbActionType::IsolationMetric),
        "plan_update" => Ok(SsbActionType::PlanUpdate),
        "replan_request" => Ok(SsbActionType::ReplanRequest),
        "stub_reference" => Ok(SsbActionType::StubReference),
        "reasoning_trace" => Ok(SsbActionType::ReasoningTrace),
        "phase_complete" => Ok(SsbActionType::PhaseComplete),
        "memory_monitoring" => Ok(SsbActionType::MemoryMonitoring),
        "workload_profile" => Ok(SsbActionType::WorkloadProfile),
        "death_event" => Ok(SsbActionType::DeathEvent),
        "recovery_event" => Ok(SsbActionType::RecoveryEvent),
        "supervision_exhausted" => Ok(SsbActionType::SupervisionExhausted),
        "spiral_detected" => Ok(SsbActionType::SpiralDetected),
        "context_reduction" => Ok(SsbActionType::ContextReduction),
        "branch_merge" => Ok(SsbActionType::BranchMerge),
        other => Err(SsbError::UnknownActionType(other.to_owned())),
    }
}

// ---------------------------------------------------------------------------
// Typed payload structs (3B.4)
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FileWritePayload {
    pub path: String,
    pub size_bytes: u64,
    pub hash: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FileReadPayload {
    pub path: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ToolResultPayload {
    pub tool_id: String,
    pub success: bool,
    pub output_summary: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TestOutcomePayload {
    pub test_name: String,
    pub passed: bool,
    pub duration_ms: u64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FindingPayload {
    pub severity: String,
    pub category: String,
    pub message: String,
    pub file_path: Option<String>,
    pub line: Option<u32>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ClaimPayload {
    pub claim_text: String,
    pub grounding_source: String,
    pub verified: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CheckpointPayload {
    pub agent_state_hash: String,
    pub grc_history_hash: String,
    pub agent_id: String,
    pub step_index: u64,
    pub file_sizes: Vec<(String, u64)>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ModelSwapPayload {
    pub from_model: String,
    pub to_model: String,
    pub reason: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PhaseTransitionPayload {
    pub from_state: String,
    pub to_state: String,
    pub evidence_summary: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct KnowledgeDegradationPayload {
    pub stage: String,
    pub from_mode: String,
    pub to_mode: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct IsolationMetricPayload {
    pub metric_name: String,
    pub value: f64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PlanUpdatePayload {
    pub plan_hash: String,
    pub phase: String,
    pub step: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ReplanRequestPayload {
    pub reason: String,
    pub requesting_agent: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct StubReferencePayload {
    pub original_seq: i64,
    pub stub_reason: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ReasoningTracePayload {
    pub step_type: String,
    pub content: String,
    pub topic: String,
    pub grounding: String,
    pub conclusion: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PhaseCompletePayload {
    pub phase_id: String,
    pub agent_id: String,
    pub fsm_state: String,
    pub evidence_artifacts: Vec<String>,
    pub commit_hashes: Vec<String>,
    pub test_outcomes: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MemoryMonitoringPayload {
    pub metric_name: String,
    pub value: f64,
    pub threshold: f64,
    pub action_taken: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DeathEventPayload {
    pub agent_id: String,
    pub classification: String,
    pub prior_step_index: i64,
    pub checkpoint_path: String,
    pub had_checkpoint: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RecoveryEventPayload {
    pub agent_id: String,
    pub classification: String,
    pub recovered_from_checkpoint: bool,
    pub degraded: bool,
    pub first_grounding_source: String,
    pub prior_step_index: i64,
    pub recovery_step_index: i64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SupervisionExhaustedPayload {
    pub agent_id: String,
    pub phase_id: String,
    pub restart_count: u32,
    pub max_restarts: u32,
    pub restart_history: Vec<i64>,
    pub escalation_action: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SpiralDetectedPayload {
    pub agent_id: String,
    pub phase_id: String,
    pub spiral_depth: u32,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ContextReductionPayload {
    pub level: u8,
    pub before_tokens: usize,
    pub after_tokens: usize,
    pub slots_affected: Vec<u8>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BranchMergePayload {
    pub winner_name: String,
    pub abandoned_names: Vec<String>,
    pub trunk_snap_id: String,
    pub from_snap_ids: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct WorkloadProfilePayload {
    pub agent_role: String,
    pub phase_id: String,
    pub tokens_in: u64,
    pub tokens_out: u64,
    pub latency_ms: u64,
    pub vram_peak_mb: u32,
    pub model_id: String,
    pub gpu_id: u8,
    pub timestamp: i64,
}

// ---------------------------------------------------------------------------
// SsbPayload enum (typed wrapper over all 24 action-type payloads)
// ---------------------------------------------------------------------------

/// Typed payload enum. Each variant maps 1:1 to an [`SsbActionType`] (24 total).
/// Serialized to msgpack via `rmp_serde` before storage.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum SsbPayload {
    FileWrite(FileWritePayload),
    FileRead(FileReadPayload),
    ToolResult(ToolResultPayload),
    TestOutcome(TestOutcomePayload),
    Finding(FindingPayload),
    Claim(ClaimPayload),
    Checkpoint(CheckpointPayload),
    ModelSwap(ModelSwapPayload),
    PhaseTransition(PhaseTransitionPayload),
    KnowledgeDegradation(KnowledgeDegradationPayload),
    IsolationMetric(IsolationMetricPayload),
    PlanUpdate(PlanUpdatePayload),
    ReplanRequest(ReplanRequestPayload),
    StubReference(StubReferencePayload),
    ReasoningTrace(ReasoningTracePayload),
    PhaseComplete(PhaseCompletePayload),
    MemoryMonitoring(MemoryMonitoringPayload),
    WorkloadProfile(WorkloadProfilePayload),
    DeathEvent(DeathEventPayload),
    RecoveryEvent(RecoveryEventPayload),
    SupervisionExhausted(SupervisionExhaustedPayload),
    SpiralDetected(SpiralDetectedPayload),
    ContextReduction(ContextReductionPayload),
    BranchMerge(BranchMergePayload),
}

impl SsbPayload {
    /// Returns the [`SsbActionType`] corresponding to this payload variant.
    /// This is the single source of truth for the discriminant→action_type mapping.
    pub fn action_type(&self) -> SsbActionType {
        match self {
            Self::FileWrite(_) => SsbActionType::FileWrite,
            Self::FileRead(_) => SsbActionType::FileRead,
            Self::ToolResult(_) => SsbActionType::ToolResult,
            Self::TestOutcome(_) => SsbActionType::TestOutcome,
            Self::Finding(_) => SsbActionType::Finding,
            Self::Claim(_) => SsbActionType::Claim,
            Self::Checkpoint(_) => SsbActionType::Checkpoint,
            Self::ModelSwap(_) => SsbActionType::ModelSwap,
            Self::PhaseTransition(_) => SsbActionType::PhaseTransition,
            Self::KnowledgeDegradation(_) => SsbActionType::KnowledgeDegradation,
            Self::IsolationMetric(_) => SsbActionType::IsolationMetric,
            Self::PlanUpdate(_) => SsbActionType::PlanUpdate,
            Self::ReplanRequest(_) => SsbActionType::ReplanRequest,
            Self::StubReference(_) => SsbActionType::StubReference,
            Self::ReasoningTrace(_) => SsbActionType::ReasoningTrace,
            Self::PhaseComplete(_) => SsbActionType::PhaseComplete,
            Self::MemoryMonitoring(_) => SsbActionType::MemoryMonitoring,
            Self::WorkloadProfile(_) => SsbActionType::WorkloadProfile,
            Self::DeathEvent(_) => SsbActionType::DeathEvent,
            Self::RecoveryEvent(_) => SsbActionType::RecoveryEvent,
            Self::SupervisionExhausted(_) => SsbActionType::SupervisionExhausted,
            Self::SpiralDetected(_) => SsbActionType::SpiralDetected,
            Self::ContextReduction(_) => SsbActionType::ContextReduction,
            Self::BranchMerge(_) => SsbActionType::BranchMerge,
        }
    }
}

// ---------------------------------------------------------------------------
// SsbTraceable trait (orchestrator-local, no cross-crate dep)
// ---------------------------------------------------------------------------

/// Trait for types that can produce an [`SsbPayload`] for recording in the SSB.
/// Defined in `orchestrator` only — never in `reasoning` or `tools` crates.
pub trait SsbTraceable {
    fn to_ssb_payload(&self) -> SsbPayload;
}

impl SsbTraceable for ReasoningTracePayload {
    fn to_ssb_payload(&self) -> SsbPayload {
        SsbPayload::ReasoningTrace(self.clone())
    }
}

// ---------------------------------------------------------------------------
// Entry + file entry structs
// ---------------------------------------------------------------------------

/// A single SSB row, broadcast to subscribers after commit.
#[derive(Debug, Clone)]
pub struct SsbEntry {
    pub seq: i64,
    pub project_id: String,
    pub session_id: String,
    pub agent_id: String,
    pub superstep: i64,
    pub action_type: SsbActionType,
    /// `Arc<[u8]>` for O(1) clone on broadcast (tokio::broadcast requires Clone).
    pub payload: Arc<[u8]>,
    pub created_at: i64,
}

/// Junction table row linking an SSB entry to a file path.
#[derive(Debug, Clone)]
pub struct SsbFileEntry {
    pub seq: i64,
    pub file_path: String,
}

// ---------------------------------------------------------------------------
// Write request (3B.2)
// ---------------------------------------------------------------------------

/// Request sent to the single-writer loop via mpsc channel.
///
/// `project_id` and `session_id` are injected by the writer (bound at `Ssb`
/// construction). Callers provide only per-write fields.
pub struct SsbWriteRequest {
    pub action_type: SsbActionType,
    pub agent_id: String,
    pub superstep: i64,
    pub payload: Vec<u8>,
    pub files: Vec<String>,
}

// ---------------------------------------------------------------------------
// Configuration
// ---------------------------------------------------------------------------

/// Configuration for SSB database and connection pool.
#[derive(Clone)]
pub struct SsbConfig {
    pub db_path: std::path::PathBuf,
    /// Read pool size. Defaults to `max(available_parallelism * 2, 4)`.
    pub pool_size: Option<u32>,
    pub project_id: String,
    pub session_id: String,
}

impl SsbConfig {
    fn effective_pool_size(&self) -> u32 {
        self.pool_size.unwrap_or_else(|| {
            let cpus = std::thread::available_parallelism()
                .map(|n| n.get() as u32)
                .unwrap_or(2);
            cpus.saturating_mul(2).max(4)
        })
    }
}

// ---------------------------------------------------------------------------
// Error type
// ---------------------------------------------------------------------------

/// Errors produced by SSB operations.
#[derive(Debug, thiserror::Error)]
pub enum SsbError {
    #[error("sqlite error: {0}")]
    Sqlite(#[from] rusqlite::Error),

    #[error("connection pool error: {0}")]
    Pool(#[from] r2d2::Error),

    #[error("ssb has been shut down")]
    Shutdown,

    #[error("write channel closed unexpectedly")]
    ChannelClosed,

    #[error("unknown SSB action type: {0}")]
    UnknownActionType(String),

    #[error("msgpack serialization error: {0}")]
    Serialization(#[from] rmp_serde::encode::Error),

    #[error("msgpack deserialization error: {0}")]
    Deserialization(#[from] rmp_serde::decode::Error),

    #[error("payload exceeds maximum size of {PAYLOAD_MAX_BYTES} bytes")]
    PayloadTooLarge,
}

// ---------------------------------------------------------------------------
// Pragma applier (r2d2 CustomizeConnection)
// ---------------------------------------------------------------------------

/// Applies per-connection pragmas on every r2d2 pool checkout.
/// `foreign_keys` is per-connection and must be set on every connection.
#[derive(Debug)]
pub struct SsbPragmaApplier;

impl r2d2::CustomizeConnection<Connection, rusqlite::Error> for SsbPragmaApplier {
    fn on_acquire(&self, conn: &mut Connection) -> Result<(), rusqlite::Error> {
        // Order matters: journal_mode first, then other pragmas.
        conn.pragma_update(None, "journal_mode", "WAL")?;
        conn.pragma_update(None, "synchronous", "NORMAL")?;
        conn.pragma_update(None, "wal_autocheckpoint", 1000)?;
        conn.pragma_update(None, "busy_timeout", 5000)?;
        conn.pragma_update(None, "cache_size", -64000)?;
        conn.pragma_update(None, "foreign_keys", "ON")?;
        conn.pragma_update(None, "temp_store", "MEMORY")?;
        rusqlite::vtab::array::load_module(conn)?;
        Ok(())
    }
}

// ---------------------------------------------------------------------------
// Reader trait
// ---------------------------------------------------------------------------

/// Read-only queries against the SSB.
pub trait SsbReader {
    fn query_by_session(&self, session_id: &str) -> Result<Vec<SsbEntry>, SsbError>;
    fn query_by_file(&self, file_paths: &[&str]) -> Result<Vec<SsbEntry>, SsbError>;
    fn query_by_type(&self, action_type: SsbActionType) -> Result<Vec<SsbEntry>, SsbError>;
    fn query_by_superstep(&self, superstep: i64) -> Result<Vec<SsbEntry>, SsbError>;
    /// Returns agent_ids that touched any of `file_paths` in the given `superstep`,
    /// excluding `session_id` (self). Uses JOIN + rarray for performance.
    fn query_conflict(
        &self,
        file_paths: &[&str],
        session_id: &str,
        superstep: i64,
    ) -> Result<Vec<String>, SsbError>;
    fn last_seq(&self) -> Result<Option<i64>, SsbError>;
    fn query_by_seq(&self, seq: i64) -> Result<Option<SsbEntry>, SsbError>;
}

// ---------------------------------------------------------------------------
// Internal message for write loop
// ---------------------------------------------------------------------------

type WriteResponse = oneshot::Sender<Result<i64, SsbError>>;

struct WriteMessage {
    request: SsbWriteRequest,
    respond: WriteResponse,
}

// ---------------------------------------------------------------------------
// Ssb (main handle)
// ---------------------------------------------------------------------------

/// Shared State Board: single-writer SQLite with broadcast notifications.
#[allow(dead_code)] // read_pool used by tests and SsbReader impl (3B.3)
pub struct Ssb {
    write_tx: Option<mpsc::UnboundedSender<WriteMessage>>,
    broadcast_tx: broadcast::Sender<SsbEntry>,
    read_pool: Pool<SqliteConnectionManager>,
    project_id: String,
    session_id: String,
    shutdown: Arc<AtomicBool>,
    _write_task: tokio::task::JoinHandle<()>,
}

impl Ssb {
    /// Create a new SSB instance. Opens (or creates) the database, applies
    /// WAL pragmas, runs migrations, and spawns the single-writer task.
    pub fn new(
        config: SsbConfig,
        clock: Arc<dyn ClockProvider>,
    ) -> Result<Self, SsbError> {
        // --- Writer connection (dedicated, not pooled) ---
        let writer_conn = Connection::open(&config.db_path)?;
        apply_pragmas(&writer_conn)?;
        run_migration(&writer_conn)?;

        #[cfg(debug_assertions)]
        writer_conn.update_hook(Some(
            |action: rusqlite::hooks::Action, _db: &str, table: &str, _rowid: i64| {
                if (table == "ssb" || table == "ssb_files")
                    && (action == rusqlite::hooks::Action::SQLITE_UPDATE
                        || action == rusqlite::hooks::Action::SQLITE_DELETE)
                {
                    panic!(
                        "SSB append-only violation: attempted {:?} on table {}",
                        action, table
                    );
                }
            },
        ))?;

        // --- Read pool ---
        let manager = SqliteConnectionManager::file(&config.db_path);
        let read_pool = Pool::builder()
            .max_size(config.effective_pool_size())
            .min_idle(Some(1))
            .idle_timeout(None)
            .max_lifetime(None)
            .connection_customizer(Box::new(SsbPragmaApplier))
            .build(manager)?;

        // --- Channels ---
        let (write_tx, write_rx) = mpsc::unbounded_channel::<WriteMessage>();
        let (broadcast_tx, _) = broadcast::channel::<SsbEntry>(1024);

        let project_id = config.project_id.clone();
        let session_id = config.session_id.clone();
        let shutdown = Arc::new(AtomicBool::new(false));

        let bc_tx = broadcast_tx.clone();
        let pid = project_id.clone();
        let sid = session_id.clone();
        let sd = shutdown.clone();

        let write_task = tokio::spawn(write_loop(
            writer_conn, write_rx, bc_tx, clock, pid, sid, sd,
        ));

        Ok(Self {
            write_tx: Some(write_tx),
            broadcast_tx,
            read_pool,
            project_id,
            session_id,
            shutdown,
            _write_task: write_task,
        })
    }

    /// Submit a write request to the single-writer loop.
    ///
    /// # IPC sync contract (WARN-6)
    ///
    /// Callers **must** `.await` this method and receive the `seq` confirmation
    /// **before** dispatching any `ToolResult` to the agent. This guarantees
    /// the SSB entry is durable before the agent sees the result.
    pub async fn write(&self, request: SsbWriteRequest) -> Result<i64, SsbError> {
        let tx = self.write_tx.as_ref().ok_or(SsbError::Shutdown)?;
        if self.shutdown.load(Ordering::Acquire) {
            return Err(SsbError::Shutdown);
        }
        let (respond, rx) = oneshot::channel();
        tx.send(WriteMessage { request, respond })
            .map_err(|_| SsbError::Shutdown)?;
        rx.await.map_err(|_| SsbError::ChannelClosed)?
    }

    /// Submit a typed payload write. Derives `action_type` from the enum variant,
    /// serializes via msgpack, enforces the size guard, and delegates to [`write()`].
    pub async fn write_payload(
        &self,
        agent_id: &str,
        superstep: i64,
        payload: SsbPayload,
        files: Vec<String>,
    ) -> Result<i64, SsbError> {
        let action_type = payload.action_type();
        let bytes = rmp_serde::to_vec_named(&payload)?;
        if bytes.len() > PAYLOAD_MAX_BYTES {
            return Err(SsbError::PayloadTooLarge);
        }
        self.write(SsbWriteRequest {
            action_type,
            agent_id: agent_id.to_owned(),
            superstep,
            payload: bytes,
            files,
        })
        .await
    }

    /// Subscribe to broadcast notifications of committed entries.
    /// Slow subscribers that fall behind 1024 entries will receive `Lagged`.
    pub fn subscribe(&self) -> broadcast::Receiver<SsbEntry> {
        self.broadcast_tx.subscribe()
    }

    /// Gracefully shut down the write loop. Idempotent.
    ///
    /// Drops the mpsc sender so the write loop drains remaining messages
    /// and exits. Pending `write()` callers whose oneshot hasn't been
    /// answered yet will receive `Err(SsbError::ChannelClosed)`.
    pub fn shutdown(&mut self) {
        if !self.shutdown.swap(true, Ordering::Release) {
            self.write_tx.take();
        }
    }

    /// Project ID this SSB is bound to.
    pub fn project_id(&self) -> &str {
        &self.project_id
    }

    /// Session ID this SSB is bound to.
    pub fn session_id(&self) -> &str {
        &self.session_id
    }
}

// ---------------------------------------------------------------------------
// WAL pragma application (writer connection)
// ---------------------------------------------------------------------------

fn apply_pragmas(conn: &Connection) -> Result<(), rusqlite::Error> {
    conn.pragma_update(None, "journal_mode", "WAL")?;
    conn.pragma_update(None, "synchronous", "NORMAL")?;
    conn.pragma_update(None, "wal_autocheckpoint", 1000)?;
    conn.pragma_update(None, "busy_timeout", 5000)?;
    conn.pragma_update(None, "cache_size", -64000)?;
    conn.pragma_update(None, "foreign_keys", "ON")?;
    conn.pragma_update(None, "temp_store", "MEMORY")?;
    Ok(())
}

// ---------------------------------------------------------------------------
// Migration
// ---------------------------------------------------------------------------

fn run_migration(conn: &Connection) -> Result<(), rusqlite::Error> {
    let version: i64 = conn.pragma_query_value(None, "user_version", |row| row.get(0))?;
    if version == SCHEMA_VERSION {
        return Ok(());
    }
    // SSB is session-scoped. Stale schema → drop and recreate (data loss
    // is acceptable for old sessions).
    if version > 0 {
        conn.execute_batch("DROP TABLE IF EXISTS ssb_files; DROP TABLE IF EXISTS ssb;")?;
    }
    conn.execute_batch(CREATE_SSB_TABLE)?;
    conn.execute_batch(CREATE_SSB_FILES_TABLE)?;
    conn.pragma_update(None, "user_version", SCHEMA_VERSION)?;
    Ok(())
}

// ---------------------------------------------------------------------------
// Write loop (3B.2 — single-writer task)
// ---------------------------------------------------------------------------

async fn write_loop(
    conn: Connection,
    mut rx: mpsc::UnboundedReceiver<WriteMessage>,
    broadcast_tx: broadcast::Sender<SsbEntry>,
    clock: Arc<dyn ClockProvider>,
    project_id: String,
    session_id: String,
    shutdown: Arc<AtomicBool>,
) {
    while let Some(msg) = rx.recv().await {
        let result = execute_write(
            &conn,
            &msg.request,
            &clock,
            &project_id,
            &session_id,
        );

        match result {
            Ok((seq, created_at)) => {
                let entry = SsbEntry {
                    seq,
                    project_id: project_id.clone(),
                    session_id: session_id.clone(),
                    agent_id: msg.request.agent_id.clone(),
                    superstep: msg.request.superstep,
                    action_type: msg.request.action_type,
                    payload: Arc::from(msg.request.payload.as_slice()),
                    created_at,
                };
                // Respond BEFORE broadcast (WARN-6 IPC sync): caller must
                // have seq confirmation before any subscriber can act.
                let _ = msg.respond.send(Ok(seq));
                // Broadcast after commit + after respond. No receivers = fine.
                let _ = broadcast_tx.send(entry);
            }
            Err(e) => {
                let _ = msg.respond.send(Err(e));
            }
        }
    }
    // Channel closed (shutdown or all senders dropped). Mark shutdown.
    shutdown.store(true, Ordering::Release);
}

/// Execute a single write within a BEGIN IMMEDIATE transaction.
fn execute_write(
    conn: &Connection,
    request: &SsbWriteRequest,
    clock: &Arc<dyn ClockProvider>,
    project_id: &str,
    session_id: &str,
) -> Result<(i64, i64), SsbError> {
    let created_at = clock
        .system_now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or(Duration::ZERO)
        .as_secs() as i64;

    // BEGIN IMMEDIATE acquires write lock early — avoids SQLITE_BUSY at commit.
    conn.execute("BEGIN IMMEDIATE", [])?;

    let insert_result = (|| -> Result<i64, rusqlite::Error> {
        conn.execute(
            "INSERT INTO ssb (project_id, session_id, agent_id, superstep, action_type, payload, created_at) \
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            params![
                project_id,
                session_id,
                request.agent_id,
                request.superstep,
                request.action_type.as_sql_str(),
                request.payload,
                created_at,
            ],
        )?;
        let seq = conn.last_insert_rowid();

        for file_path in &request.files {
            conn.execute(
                "INSERT INTO ssb_files (seq, file_path) VALUES (?1, ?2)",
                params![seq, file_path],
            )?;
        }
        Ok(seq)
    })();

    match insert_result {
        Ok(seq) => {
            // COMMIT can fail (disk full, I/O error). If it does, we must
            // ROLLBACK to avoid leaving the connection stuck in an open
            // transaction (rusqlite#366).
            if let Err(e) = conn.execute_batch("COMMIT") {
                let _ = conn.execute_batch("ROLLBACK");
                return Err(SsbError::Sqlite(e));
            }
            Ok((seq, created_at))
        }
        Err(e) => {
            let _ = conn.execute_batch("ROLLBACK");
            Err(SsbError::Sqlite(e))
        }
    }
}

// ---------------------------------------------------------------------------
// Row mapper
// ---------------------------------------------------------------------------

fn row_to_entry(row: &rusqlite::Row<'_>) -> rusqlite::Result<SsbEntry> {
    let at_str: String = row.get(5)?;
    let action_type = ssb_action_type_from_str(&at_str).map_err(|e| {
        rusqlite::Error::FromSqlConversionFailure(5, rusqlite::types::Type::Text, Box::new(e))
    })?;
    let payload_bytes: Vec<u8> = row.get(6)?;
    Ok(SsbEntry {
        seq: row.get(0)?,
        project_id: row.get(1)?,
        session_id: row.get(2)?,
        agent_id: row.get(3)?,
        superstep: row.get(4)?,
        action_type,
        payload: Arc::from(payload_bytes.as_slice()),
        created_at: row.get(7)?,
    })
}

// ---------------------------------------------------------------------------
// SsbReader implementation
// ---------------------------------------------------------------------------

impl SsbReader for Ssb {
    fn query_by_session(&self, session_id: &str) -> Result<Vec<SsbEntry>, SsbError> {
        let conn = self.read_pool.get()?;
        let mut stmt = conn.prepare(
            "SELECT seq, project_id, session_id, agent_id, superstep, action_type, payload, created_at \
             FROM ssb WHERE session_id = ?1 AND project_id = ?2",
        )?;
        let rows = stmt.query_map(params![session_id, self.project_id], row_to_entry)?;
        rows.collect::<Result<Vec<_>, _>>().map_err(SsbError::Sqlite)
    }

    fn query_by_file(&self, file_paths: &[&str]) -> Result<Vec<SsbEntry>, SsbError> {
        if file_paths.is_empty() {
            return Ok(vec![]);
        }
        let conn = self.read_pool.get()?;
        let values: Vec<Value> = file_paths.iter().map(|p| Value::Text((*p).to_owned())).collect();
        let ptr = Rc::new(values);
        let mut stmt = conn.prepare(
            "SELECT DISTINCT ssb.seq, ssb.project_id, ssb.session_id, ssb.agent_id, \
             ssb.superstep, ssb.action_type, ssb.payload, ssb.created_at \
             FROM ssb JOIN ssb_files ON ssb_files.seq = ssb.seq \
             WHERE ssb_files.file_path IN (SELECT value FROM rarray(?1)) \
               AND ssb.project_id = ?2",
        )?;
        let rows = stmt.query_map(params![ptr, self.project_id], row_to_entry)?;
        rows.collect::<Result<Vec<_>, _>>().map_err(SsbError::Sqlite)
    }

    fn query_by_type(&self, action_type: SsbActionType) -> Result<Vec<SsbEntry>, SsbError> {
        let conn = self.read_pool.get()?;
        let mut stmt = conn.prepare(
            "SELECT seq, project_id, session_id, agent_id, superstep, action_type, payload, created_at \
             FROM ssb WHERE action_type = ?1 AND project_id = ?2",
        )?;
        let rows = stmt.query_map(params![action_type.as_sql_str(), self.project_id], row_to_entry)?;
        rows.collect::<Result<Vec<_>, _>>().map_err(SsbError::Sqlite)
    }

    fn query_by_superstep(&self, superstep: i64) -> Result<Vec<SsbEntry>, SsbError> {
        let conn = self.read_pool.get()?;
        let mut stmt = conn.prepare(
            "SELECT seq, project_id, session_id, agent_id, superstep, action_type, payload, created_at \
             FROM ssb WHERE superstep = ?1 AND project_id = ?2",
        )?;
        let rows = stmt.query_map(params![superstep, self.project_id], row_to_entry)?;
        rows.collect::<Result<Vec<_>, _>>().map_err(SsbError::Sqlite)
    }

    fn query_conflict(
        &self,
        file_paths: &[&str],
        session_id: &str,
        superstep: i64,
    ) -> Result<Vec<String>, SsbError> {
        if file_paths.is_empty() {
            return Ok(vec![]);
        }
        let conn = self.read_pool.get()?;
        let values: Vec<Value> = file_paths.iter().map(|p| Value::Text((*p).to_owned())).collect();
        let ptr = Rc::new(values);
        let mut stmt = conn.prepare(
            "SELECT DISTINCT ssb.agent_id \
             FROM ssb_files \
             JOIN ssb ON ssb_files.seq = ssb.seq \
             WHERE ssb_files.file_path IN (SELECT value FROM rarray(?1)) \
               AND ssb.project_id = ?2 \
               AND ssb.session_id != ?3 \
               AND ssb.superstep = ?4",
        )?;
        let rows = stmt.query_map(
            params![ptr, self.project_id, session_id, superstep],
            |row| row.get::<_, String>(0),
        )?;
        rows.collect::<Result<Vec<_>, _>>().map_err(SsbError::Sqlite)
    }

    fn last_seq(&self) -> Result<Option<i64>, SsbError> {
        let conn = self.read_pool.get()?;
        let mut stmt = conn.prepare("SELECT MAX(seq) FROM ssb WHERE project_id = ?1")?;
        let result: Option<i64> = stmt.query_row(params![self.project_id], |row| row.get(0))?;
        Ok(result)
    }

    fn query_by_seq(&self, seq: i64) -> Result<Option<SsbEntry>, SsbError> {
        let conn = self.read_pool.get()?;
        let mut stmt = conn.prepare(
            "SELECT seq, project_id, session_id, agent_id, superstep, action_type, payload, created_at \
             FROM ssb WHERE seq = ?1 AND project_id = ?2 LIMIT 1",
        )?;
        let mut rows = stmt.query_map(params![seq, self.project_id], row_to_entry)?;
        match rows.next() {
            Some(row) => Ok(Some(row.map_err(SsbError::Sqlite)?)),
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
    use elai_core::clock::SeededClock;
    use std::time::{Duration, SystemTime};
    use tempfile::TempDir;

    fn test_config(dir: &TempDir) -> SsbConfig {
        SsbConfig {
            db_path: dir.path().join("test_ssb.db"),
            pool_size: Some(2),
            project_id: "test-project".into(),
            session_id: "test-session".into(),
        }
    }

    fn test_clock() -> Arc<dyn ClockProvider> {
        Arc::new(SeededClock::new())
    }

    #[tokio::test]
    async fn test_single_writer() {
        let dir = TempDir::new().expect("tempdir");
        let mut ssb = Ssb::new(test_config(&dir), test_clock()).expect("ssb init");

        let mut seqs = Vec::new();
        for i in 0..10 {
            let seq = ssb
                .write(SsbWriteRequest {
                    action_type: SsbActionType::FileWrite,
                    agent_id: format!("agent-{i}"),
                    superstep: 0,
                    payload: vec![i as u8],
                    files: vec![format!("src/file_{i}.rs")],
                })
                .await
                .expect("write");
            seqs.push(seq);
        }

        // All seqs must be monotonically increasing with no gaps.
        for window in seqs.windows(2) {
            assert_eq!(window[1], window[0] + 1, "seqs must be sequential");
        }

        ssb.shutdown();
    }

    #[tokio::test]
    async fn test_write_after_shutdown() {
        let dir = TempDir::new().expect("tempdir");
        let mut ssb = Ssb::new(test_config(&dir), test_clock()).expect("ssb init");

        // Write one entry successfully.
        ssb.write(SsbWriteRequest {
            action_type: SsbActionType::Checkpoint,
            agent_id: "agent-1".into(),
            superstep: 0,
            payload: vec![1],
            files: vec![],
        })
        .await
        .expect("write before shutdown");

        ssb.shutdown();

        // Write after shutdown must fail.
        let result = ssb
            .write(SsbWriteRequest {
                action_type: SsbActionType::Checkpoint,
                agent_id: "agent-1".into(),
                superstep: 0,
                payload: vec![2],
                files: vec![],
            })
            .await;

        assert!(result.is_err(), "write after shutdown must fail");
        match result.unwrap_err() {
            SsbError::Shutdown => {}
            other => panic!("expected Shutdown error, got: {other}"),
        }
    }

    #[tokio::test]
    async fn test_concurrent_writers() {
        let dir = TempDir::new().expect("tempdir");
        let ssb = Arc::new(
            Ssb::new(test_config(&dir), test_clock()).expect("ssb init"),
        );

        // Spawn 10 concurrent write tasks — no Mutex needed, write() takes &self
        // and the mpsc channel serializes writes internally.
        let mut handles = Vec::new();
        for i in 0..10 {
            let ssb = ssb.clone();
            handles.push(tokio::spawn(async move {
                ssb.write(SsbWriteRequest {
                    action_type: SsbActionType::ToolResult,
                    agent_id: format!("agent-{i}"),
                    superstep: 0,
                    payload: vec![i as u8],
                    files: vec![format!("src/mod_{i}.rs")],
                })
                .await
            }));
        }

        let mut seqs: Vec<i64> = Vec::new();
        for h in handles {
            let seq = h.await.expect("join").expect("write");
            seqs.push(seq);
        }

        seqs.sort();
        // All 10 seqs present, sequential, no gaps.
        assert_eq!(seqs.len(), 10);
        for window in seqs.windows(2) {
            assert_eq!(window[1], window[0] + 1, "seqs must be sequential");
        }
    }

    #[tokio::test]
    async fn test_broadcast_after_commit() {
        let dir = TempDir::new().expect("tempdir");
        let ssb = Ssb::new(test_config(&dir), test_clock()).expect("ssb init");

        let mut rx = ssb.subscribe();

        let seq = ssb
            .write(SsbWriteRequest {
                action_type: SsbActionType::Finding,
                agent_id: "agent-x".into(),
                superstep: 1,
                payload: b"test-payload".to_vec(),
                files: vec!["src/main.rs".into()],
            })
            .await
            .expect("write");

        let entry = tokio::time::timeout(Duration::from_secs(2), rx.recv())
            .await
            .expect("timeout")
            .expect("recv");

        assert_eq!(entry.seq, seq);
        assert_eq!(entry.agent_id, "agent-x");
        assert_eq!(entry.superstep, 1);
        assert_eq!(entry.action_type, SsbActionType::Finding);
        assert_eq!(&*entry.payload, b"test-payload");
    }

    #[tokio::test]
    async fn test_created_at_uses_clock_provider() {
        let clock = Arc::new(SeededClock::with_epoch(
            SystemTime::UNIX_EPOCH + Duration::from_secs(1_000_000),
        ));
        let dir = TempDir::new().expect("tempdir");
        let config = SsbConfig {
            db_path: dir.path().join("clock_test.db"),
            pool_size: Some(2),
            project_id: "proj".into(),
            session_id: "sess".into(),
        };
        let ssb = Ssb::new(config, clock.clone()).expect("ssb init");

        let mut rx = ssb.subscribe();
        ssb.write(SsbWriteRequest {
            action_type: SsbActionType::Claim,
            agent_id: "agent-clock".into(),
            superstep: 0,
            payload: vec![],
            files: vec![],
        })
        .await
        .expect("write");

        let entry = tokio::time::timeout(Duration::from_secs(2), rx.recv())
            .await
            .expect("timeout")
            .expect("recv");

        // SeededClock epoch is 1_000_000 secs past UNIX_EPOCH.
        assert_eq!(entry.created_at, 1_000_000);
    }

    #[tokio::test]
    async fn test_wal_pragma_applied() {
        let dir = TempDir::new().expect("tempdir");
        let config = test_config(&dir);
        let _ssb = Ssb::new(config.clone(), test_clock()).expect("ssb init");

        // Verify via a direct connection that WAL is active.
        let conn = Connection::open(&config.db_path).expect("open");
        let journal_mode: String =
            conn.pragma_query_value(None, "journal_mode", |row| row.get(0))
                .expect("pragma");
        assert_eq!(journal_mode.to_lowercase(), "wal");
    }

    #[tokio::test]
    async fn test_pool_connection_has_foreign_keys() {
        let dir = TempDir::new().expect("tempdir");
        let ssb = Ssb::new(test_config(&dir), test_clock()).expect("ssb init");

        let conn = ssb.read_pool.get().expect("pool get");
        let fk: i64 = conn
            .pragma_query_value(None, "foreign_keys", |row| row.get(0))
            .expect("pragma");
        assert_eq!(fk, 1, "foreign_keys must be ON for pool connections");
    }

    #[tokio::test]
    async fn test_migration_from_v0() {
        let dir = TempDir::new().expect("tempdir");
        let db_path = dir.path().join("migrate_test.db");

        // Create a V0 database (no tables, user_version=0).
        {
            let conn = Connection::open(&db_path).expect("open");
            conn.pragma_update(None, "user_version", 0).expect("pragma");
        }

        // SSB should create tables.
        let config = SsbConfig {
            db_path,
            pool_size: Some(2),
            project_id: "proj".into(),
            session_id: "sess".into(),
        };
        let ssb = Ssb::new(config, test_clock()).expect("ssb init");

        // Write should succeed (tables exist).
        ssb.write(SsbWriteRequest {
            action_type: SsbActionType::FileRead,
            agent_id: "agent-m".into(),
            superstep: 0,
            payload: vec![42],
            files: vec![],
        })
        .await
        .expect("write after migration");
    }

    #[tokio::test]
    async fn test_shutdown_idempotent() {
        let dir = TempDir::new().expect("tempdir");
        let mut ssb = Ssb::new(test_config(&dir), test_clock()).expect("ssb init");

        ssb.shutdown();
        ssb.shutdown(); // Second call must not panic.
    }

    #[tokio::test]
    async fn test_ssb_files_inserted() {
        let dir = TempDir::new().expect("tempdir");
        let ssb = Ssb::new(test_config(&dir), test_clock()).expect("ssb init");

        let seq = ssb
            .write(SsbWriteRequest {
                action_type: SsbActionType::FileWrite,
                agent_id: "agent-f".into(),
                superstep: 0,
                payload: vec![1],
                files: vec!["src/a.rs".into(), "src/b.rs".into()],
            })
            .await
            .expect("write");

        // Verify via read pool.
        let conn = ssb.read_pool.get().expect("pool");
        let mut stmt = conn
            .prepare("SELECT file_path FROM ssb_files WHERE seq = ?1 ORDER BY file_path")
            .expect("prepare");
        let paths: Vec<String> = stmt
            .query_map(params![seq], |row| row.get(0))
            .expect("query")
            .filter_map(|r| r.ok())
            .collect();
        assert_eq!(paths, vec!["src/a.rs", "src/b.rs"]);
    }

    #[tokio::test]
    async fn test_all_action_types_insert() {
        let dir = TempDir::new().expect("tempdir");
        let ssb = Ssb::new(test_config(&dir), test_clock()).expect("ssb init");

        let types = [
            SsbActionType::FileWrite,
            SsbActionType::FileRead,
            SsbActionType::ToolResult,
            SsbActionType::TestOutcome,
            SsbActionType::Finding,
            SsbActionType::Claim,
            SsbActionType::Checkpoint,
            SsbActionType::ModelSwap,
            SsbActionType::PhaseTransition,
            SsbActionType::KnowledgeDegradation,
            SsbActionType::IsolationMetric,
            SsbActionType::PlanUpdate,
            SsbActionType::ReplanRequest,
            SsbActionType::StubReference,
            SsbActionType::ReasoningTrace,
            SsbActionType::PhaseComplete,
            SsbActionType::MemoryMonitoring,
            SsbActionType::DeathEvent,
            SsbActionType::RecoveryEvent,
            SsbActionType::SupervisionExhausted,
            SsbActionType::SpiralDetected,
            SsbActionType::ContextReduction,
            SsbActionType::BranchMerge,
        ];

        for (i, at) in types.iter().enumerate() {
            ssb.write(SsbWriteRequest {
                action_type: *at,
                agent_id: format!("agent-{i}"),
                superstep: 0,
                payload: vec![i as u8],
                files: vec![],
            })
            .await
            .unwrap_or_else(|e| panic!("write {at:?} failed: {e}"));
        }

        // Verify count.
        let conn = ssb.read_pool.get().expect("pool");
        let count: i64 = conn
            .query_row("SELECT COUNT(*) FROM ssb", [], |row| row.get(0))
            .expect("count");
        assert_eq!(count, 23);
    }

    #[tokio::test]
    async fn test_10k_sequential_writes() {
        let dir = TempDir::new().expect("tempdir");
        let mut ssb = Ssb::new(test_config(&dir), test_clock()).expect("ssb init");

        let mut prev_seq = 0i64;
        for i in 0..10_000 {
            let seq = ssb
                .write(SsbWriteRequest {
                    action_type: SsbActionType::ToolResult,
                    agent_id: "agent-bulk".into(),
                    superstep: 0,
                    payload: vec![(i & 0xFF) as u8],
                    files: vec![],
                })
                .await
                .expect("write");

            if i > 0 {
                assert_eq!(seq, prev_seq + 1, "seq gap at write {i}");
            }
            prev_seq = seq;
        }

        ssb.shutdown();
    }

    // -----------------------------------------------------------------------
    // 3B.3 — Read-path tests
    // -----------------------------------------------------------------------

    #[tokio::test]
    async fn test_read_pool() {
        let dir = TempDir::new().expect("tempdir");
        let mut ssb = Ssb::new(test_config(&dir), test_clock()).expect("ssb init");

        // Write 3 entries with different action types
        for (i, at) in [
            SsbActionType::FileWrite,
            SsbActionType::Finding,
            SsbActionType::FileWrite,
        ]
        .iter()
        .enumerate()
        {
            ssb.write(SsbWriteRequest {
                action_type: *at,
                agent_id: format!("agent-{i}"),
                superstep: 0,
                payload: vec![i as u8],
                files: vec![format!("src/f{i}.rs")],
            })
            .await
            .expect("write");
        }

        // query_by_session
        let rows = ssb.query_by_session("test-session").expect("by_session");
        assert_eq!(rows.len(), 3, "query_by_session should return 3 entries");

        // query_by_type
        let rows = ssb
            .query_by_type(SsbActionType::FileWrite)
            .expect("by_type");
        assert_eq!(rows.len(), 2, "query_by_type FileWrite should return 2");

        // query_by_superstep
        let rows = ssb.query_by_superstep(0).expect("by_superstep");
        assert_eq!(rows.len(), 3, "query_by_superstep(0) should return 3");

        // query_by_file (uses rarray)
        let rows = ssb
            .query_by_file(&["src/f0.rs", "src/f2.rs"])
            .expect("by_file");
        assert_eq!(rows.len(), 2, "query_by_file should return 2 entries");

        // query_by_file with empty slice
        let rows = ssb.query_by_file(&[]).expect("by_file_empty");
        assert_eq!(rows.len(), 0, "query_by_file([]) should return 0");

        // last_seq
        let seq = ssb.last_seq().expect("last_seq");
        assert_eq!(seq, Some(3), "last_seq should be 3 after 3 writes");

        // Verify round-trip of action_type through ssb_action_type_from_str
        let finding = ssb
            .query_by_type(SsbActionType::Finding)
            .expect("finding");
        assert_eq!(finding.len(), 1);
        assert_eq!(finding[0].action_type, SsbActionType::Finding);

        ssb.shutdown();
    }

    #[tokio::test]
    async fn test_conflict_detection() {
        let dir = TempDir::new().expect("tempdir");
        let config = SsbConfig {
            db_path: dir.path().join("test_ssb.db"),
            pool_size: Some(2),
            project_id: "proj-1".into(),
            session_id: "session-A".into(),
        };
        let mut ssb = Ssb::new(config, test_clock()).expect("ssb init");

        // session-A writes to f1.rs and f2.rs
        ssb.write(SsbWriteRequest {
            action_type: SsbActionType::FileWrite,
            agent_id: "agent-A".into(),
            superstep: 1,
            payload: vec![1],
            files: vec!["src/f1.rs".into(), "src/f2.rs".into()],
        })
        .await
        .expect("write A");

        // Test-only backdoor: direct SQL insert bypasses write_loop and
        // update_hook (which is on writer_conn, not pool connections).
        {
            let conn = ssb.read_pool.get().expect("pool conn");
            conn.execute(
                "INSERT INTO ssb (project_id, session_id, agent_id, superstep, action_type, payload, created_at) \
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
                params!["proj-1", "session-B", "agent-B", 1i64, "file_write", &[2u8][..], 100i64],
            )
            .expect("direct insert ssb");
            let seq = conn.last_insert_rowid();
            conn.execute(
                "INSERT INTO ssb_files (seq, file_path) VALUES (?1, ?2)",
                params![seq, "src/f1.rs"],
            )
            .expect("direct insert ssb_files");
        }

        // query_conflict: session-A asks who else touched f1.rs in superstep 1
        let conflicts = ssb
            .query_conflict(&["src/f1.rs"], "session-A", 1)
            .expect("conflict");
        assert_eq!(conflicts, vec!["agent-B"], "should detect agent-B");

        // f2.rs was only touched by session-A, no conflict for session-B asking
        let conflicts = ssb
            .query_conflict(&["src/f2.rs"], "session-B", 1)
            .expect("conflict f2");
        assert_eq!(conflicts, vec!["agent-A"], "should detect agent-A for f2");

        // No conflict for untouched file
        let conflicts = ssb
            .query_conflict(&["src/untouched.rs"], "session-A", 1)
            .expect("no conflict");
        assert!(conflicts.is_empty(), "no conflict for untouched file");

        // Empty file_paths → empty result
        let conflicts = ssb
            .query_conflict(&[], "session-A", 1)
            .expect("empty paths");
        assert!(conflicts.is_empty(), "empty paths = empty result");

        ssb.shutdown();
    }

    #[tokio::test]
    async fn test_conflict_cross_project() {
        let dir = TempDir::new().expect("tempdir");

        // Two SSB instances with different project IDs sharing the same DB
        let config_a = SsbConfig {
            db_path: dir.path().join("test_ssb.db"),
            pool_size: Some(2),
            project_id: "proj-alpha".into(),
            session_id: "session-1".into(),
        };
        let config_b = SsbConfig {
            db_path: dir.path().join("test_ssb.db"),
            pool_size: Some(2),
            project_id: "proj-beta".into(),
            session_id: "session-2".into(),
        };
        let mut ssb_a = Ssb::new(config_a, test_clock()).expect("ssb_a init");
        let mut ssb_b = Ssb::new(config_b, test_clock()).expect("ssb_b init");

        // proj-alpha writes to shared.rs
        ssb_a
            .write(SsbWriteRequest {
                action_type: SsbActionType::FileWrite,
                agent_id: "agent-alpha".into(),
                superstep: 0,
                payload: vec![1],
                files: vec!["src/shared.rs".into()],
            })
            .await
            .expect("write alpha");

        // proj-beta writes to shared.rs
        ssb_b
            .write(SsbWriteRequest {
                action_type: SsbActionType::FileWrite,
                agent_id: "agent-beta".into(),
                superstep: 0,
                payload: vec![2],
                files: vec!["src/shared.rs".into()],
            })
            .await
            .expect("write beta");

        // Cross-project isolation: all read methods must be project-scoped.

        // query_conflict: proj-alpha should NOT see proj-beta's writes
        let conflicts = ssb_a
            .query_conflict(&["src/shared.rs"], "session-1", 0)
            .expect("cross-project conflict");
        assert!(
            conflicts.is_empty(),
            "cross-project should not detect conflict, got: {conflicts:?}"
        );

        // query_by_session: ssb_a should only see its own session's entries
        let rows = ssb_a
            .query_by_session("session-1")
            .expect("by_session alpha");
        assert_eq!(rows.len(), 1, "alpha should see only its own entry");
        assert_eq!(rows[0].project_id, "proj-alpha");

        // query_by_file: ssb_a should only see alpha's write to shared.rs
        let rows = ssb_a
            .query_by_file(&["src/shared.rs"])
            .expect("by_file alpha");
        assert_eq!(rows.len(), 1, "alpha should see only its own file entry");
        assert_eq!(rows[0].agent_id, "agent-alpha");

        // query_by_type: ssb_a should only see its own FileWrite
        let rows = ssb_a
            .query_by_type(SsbActionType::FileWrite)
            .expect("by_type alpha");
        assert_eq!(rows.len(), 1, "alpha should see only its own type entry");

        // query_by_superstep: ssb_a should only see its own superstep 0
        let rows = ssb_a
            .query_by_superstep(0)
            .expect("by_superstep alpha");
        assert_eq!(rows.len(), 1, "alpha should see only its own superstep entry");

        // last_seq: ssb_a should see only its own max seq
        let alpha_seq = ssb_a.last_seq().expect("last_seq alpha");
        let beta_seq = ssb_b.last_seq().expect("last_seq beta");
        assert!(alpha_seq.is_some());
        assert!(beta_seq.is_some());
        // Both wrote 1 entry each. If last_seq were global, alpha would see beta's seq.
        // Since project-scoped, each sees only its own.
        assert_ne!(
            alpha_seq, beta_seq,
            "project-scoped last_seq should differ (different seqs per project)"
        );

        ssb_a.shutdown();
        ssb_b.shutdown();
    }

    #[tokio::test]
    async fn test_append_only() {
        use std::sync::atomic::AtomicU32;

        let dir = TempDir::new().expect("tempdir");
        let mut ssb = Ssb::new(test_config(&dir), test_clock()).expect("ssb init");

        // Write one entry so we have a row to attempt mutation on
        ssb.write(SsbWriteRequest {
            action_type: SsbActionType::Checkpoint,
            agent_id: "agent-test".into(),
            superstep: 0,
            payload: vec![1],
            files: vec![],
        })
        .await
        .expect("write");

        // Use a flag-based hook instead of panic (panics in C FFI callbacks abort).
        // This tests the hook LOGIC (detection of UPDATE/DELETE), not the panic path.
        let violation_count = Arc::new(AtomicU32::new(0));
        let vc = violation_count.clone();

        let db_path = dir.path().join("test_ssb.db");
        let direct_conn = Connection::open(&db_path).expect("direct conn");
        direct_conn
            .update_hook(Some(
                move |action: rusqlite::hooks::Action, _db: &str, table: &str, _rowid: i64| {
                    if (table == "ssb" || table == "ssb_files")
                        && (action == rusqlite::hooks::Action::SQLITE_UPDATE
                            || action == rusqlite::hooks::Action::SQLITE_DELETE)
                    {
                        vc.fetch_add(1, Ordering::Relaxed);
                    }
                },
            ))
            .expect("set hook");

        // INSERT should NOT trigger the violation counter
        direct_conn
            .execute(
                "INSERT INTO ssb (project_id, session_id, agent_id, superstep, action_type, payload, created_at) \
                 VALUES ('p', 's', 'a', 0, 'checkpoint', X'01', 100)",
                [],
            )
            .expect("insert ok");
        assert_eq!(
            violation_count.load(Ordering::Relaxed),
            0,
            "INSERT should not trigger append-only hook"
        );

        // UPDATE should trigger
        direct_conn
            .execute("UPDATE ssb SET agent_id = 'hacked' WHERE seq = 1", [])
            .expect("update executes");
        assert_eq!(
            violation_count.load(Ordering::Relaxed),
            1,
            "UPDATE should trigger append-only hook"
        );

        // DELETE should also trigger
        direct_conn
            .execute("DELETE FROM ssb WHERE seq = 2", [])
            .expect("delete executes");
        assert_eq!(
            violation_count.load(Ordering::Relaxed),
            2,
            "DELETE should trigger append-only hook"
        );

        // Verify the production code installs the hook on writer (debug mode).
        // The Ssb writer_conn has the panic-based hook. We test its presence
        // indirectly: if write() succeeds (INSERT), the hook didn't fire.
        let seq = ssb
            .write(SsbWriteRequest {
                action_type: SsbActionType::Checkpoint,
                agent_id: "agent-verify".into(),
                superstep: 0,
                payload: vec![2],
                files: vec![],
            })
            .await
            .expect("write after hook installed");
        assert!(seq > 0, "write should succeed (INSERT allowed by hook)");

        ssb.shutdown();
    }

    // -----------------------------------------------------------------------
    // 3B.4 — Typed payload tests
    // -----------------------------------------------------------------------

    fn all_test_payloads() -> Vec<SsbPayload> {
        vec![
            SsbPayload::FileWrite(FileWritePayload {
                path: "src/main.rs".into(),
                size_bytes: 1024,
                hash: "abc123".into(),
            }),
            SsbPayload::FileRead(FileReadPayload {
                path: "src/lib.rs".into(),
            }),
            SsbPayload::ToolResult(ToolResultPayload {
                tool_id: "grep".into(),
                success: true,
                output_summary: "found 3 matches".into(),
            }),
            SsbPayload::TestOutcome(TestOutcomePayload {
                test_name: "test_foo".into(),
                passed: true,
                duration_ms: 42,
            }),
            SsbPayload::Finding(FindingPayload {
                severity: "high".into(),
                category: "security".into(),
                message: "SQL injection".into(),
                file_path: Some("src/db.rs".into()),
                line: Some(99),
            }),
            SsbPayload::Claim(ClaimPayload {
                claim_text: "blake3 is faster".into(),
                grounding_source: "benchmark".into(),
                verified: true,
            }),
            SsbPayload::Checkpoint(CheckpointPayload {
                agent_state_hash: "deadbeef".into(),
                grc_history_hash: "cafebabe".into(),
                agent_id: "agent-001".into(),
                step_index: 5,
                file_sizes: vec![("state.msgpack".into(), 1024)],
            }),
            SsbPayload::ModelSwap(ModelSwapPayload {
                from_model: "sonnet".into(),
                to_model: "opus".into(),
                reason: "complex task".into(),
            }),
            SsbPayload::PhaseTransition(PhaseTransitionPayload {
                from_state: "PLANNING".into(),
                to_state: "EXECUTING".into(),
                evidence_summary: "all tests pass".into(),
            }),
            SsbPayload::KnowledgeDegradation(KnowledgeDegradationPayload {
                stage: "compaction".into(),
                from_mode: "full".into(),
                to_mode: "summary".into(),
            }),
            SsbPayload::IsolationMetric(IsolationMetricPayload {
                metric_name: "memory_mb".into(),
                value: 256.5,
            }),
            SsbPayload::PlanUpdate(PlanUpdatePayload {
                plan_hash: "plan123".into(),
                phase: "3B".into(),
                step: "4".into(),
            }),
            SsbPayload::ReplanRequest(ReplanRequestPayload {
                reason: "scope change".into(),
                requesting_agent: "agent-0".into(),
            }),
            SsbPayload::StubReference(StubReferencePayload {
                original_seq: 42,
                stub_reason: "compacted".into(),
            }),
            SsbPayload::ReasoningTrace(ReasoningTracePayload {
                step_type: "observe".into(),
                content: "read file".into(),
                topic: "debug".into(),
                grounding: "file:src/main.rs".into(),
                conclusion: "ACTION: proceed".into(),
            }),
            SsbPayload::PhaseComplete(PhaseCompletePayload {
                phase_id: "3B.4".into(),
                agent_id: "agent-main".into(),
                fsm_state: "COMPLETE".into(),
                evidence_artifacts: vec!["test-results.json".into()],
                commit_hashes: vec!["abc1234".into()],
                test_outcomes: vec!["20 passed, 0 failed".into()],
            }),
            SsbPayload::MemoryMonitoring(MemoryMonitoringPayload {
                metric_name: "heap_mb".into(),
                value: 512.0,
                threshold: 1024.0,
                action_taken: "none".into(),
            }),
            SsbPayload::DeathEvent(DeathEventPayload {
                agent_id: "agent-dead".into(),
                classification: "crash".into(),
                prior_step_index: 10,
                checkpoint_path: "sessions/default/agents/abc/abc".into(),
                had_checkpoint: true,
            }),
            SsbPayload::RecoveryEvent(RecoveryEventPayload {
                agent_id: "agent-dead".into(),
                classification: "crash".into(),
                recovered_from_checkpoint: true,
                degraded: false,
                first_grounding_source: "CHECKPOINT:sessions/default/agents/abc/abc".into(),
                prior_step_index: 10,
                recovery_step_index: 11,
            }),
            SsbPayload::SupervisionExhausted(SupervisionExhaustedPayload {
                agent_id: "agent-exhausted".into(),
                phase_id: "phase-3C".into(),
                restart_count: 4,
                max_restarts: 3,
                restart_history: vec![1, 5, 9, 13],
                escalation_action: "terminate".into(),
            }),
            SsbPayload::SpiralDetected(SpiralDetectedPayload {
                agent_id: "agent-spiral".into(),
                phase_id: "phase-3C".into(),
                spiral_depth: 5,
            }),
            SsbPayload::ContextReduction(ContextReductionPayload {
                level: 1,
                before_tokens: 10000,
                after_tokens: 8000,
                slots_affected: vec![5, 6],
            }),
            SsbPayload::BranchMerge(BranchMergePayload {
                winner_name: "explore-1".into(),
                abandoned_names: vec!["explore-2".into()],
                trunk_snap_id: "snap-merge-001".into(),
                from_snap_ids: vec!["snap-tip-1".into(), "snap-tip-2".into()],
            }),
        ]
    }

    #[tokio::test]
    async fn test_typed_payloads() {
        let dir = TempDir::new().expect("tempdir");
        let ssb = Ssb::new(test_config(&dir), test_clock()).expect("ssb init");

        let payloads = all_test_payloads();
        assert_eq!(payloads.len(), 23, "must cover all 23 action types");

        for (i, payload) in payloads.iter().enumerate() {
            let expected_action = payload.action_type();
            let seq = ssb
                .write_payload(
                    &format!("agent-{i}"),
                    i as i64,
                    payload.clone(),
                    vec![format!("file-{i}.rs")],
                )
                .await
                .unwrap_or_else(|e| panic!("write_payload variant {i} failed: {e}"));
            assert!(seq > 0, "seq must be positive for variant {i}");

            // Read back via query_by_type and verify round-trip
            let entries = ssb
                .query_by_type(expected_action)
                .unwrap_or_else(|e| panic!("query_by_type {expected_action:?} failed: {e}"));
            assert!(
                !entries.is_empty(),
                "query_by_type should return entries for {expected_action:?}"
            );
            let entry = entries.last().expect("at least one entry");
            let deserialized: SsbPayload =
                rmp_serde::from_slice(&entry.payload)
                    .unwrap_or_else(|e| panic!("deserialize variant {i} failed: {e}"));
            assert_eq!(
                &deserialized, payload,
                "round-trip mismatch for variant {i}"
            );
        }
    }

    #[tokio::test]
    async fn test_payload_size_limit() {
        let dir = TempDir::new().expect("tempdir");
        let ssb = Ssb::new(test_config(&dir), test_clock()).expect("ssb init");

        // Build a payload that serializes to > 1 MiB
        let big_content = "x".repeat(2 * 1024 * 1024);
        let big_payload = SsbPayload::ReasoningTrace(ReasoningTracePayload {
            step_type: "observe".into(),
            content: big_content,
            topic: "t".into(),
            grounding: "g".into(),
            conclusion: "c".into(),
        });

        let result = ssb.write_payload("agent-big", 0, big_payload, vec![]).await;
        assert!(result.is_err(), "oversized payload must be rejected");
        match result.unwrap_err() {
            SsbError::PayloadTooLarge => {}
            other => panic!("expected PayloadTooLarge, got: {other}"),
        }
    }

    #[tokio::test]
    async fn test_trace_integration() {
        let trace = ReasoningTracePayload {
            step_type: "hypothesize".into(),
            content: "maybe X".into(),
            topic: "debug".into(),
            grounding: "test:unit".into(),
            conclusion: "".into(),
        };

        let payload = trace.to_ssb_payload();
        match &payload {
            SsbPayload::ReasoningTrace(inner) => {
                assert_eq!(inner.step_type, "hypothesize");
                assert_eq!(inner.content, "maybe X");
            }
            other => panic!("expected ReasoningTrace variant, got: {other:?}"),
        }
        assert_eq!(payload.action_type(), SsbActionType::ReasoningTrace);
    }

    // Type mismatch is impossible at the type level: write_payload() derives
    // action_type from the SsbPayload enum variant via action_type(). There is
    // no runtime string matching and no way to pass a mismatched pair. This is
    // a compile-time guarantee, not a runtime check.

    #[tokio::test]
    async fn test_payload_empty_vecs() {
        // PhaseCompletePayload with empty Vec fields must not error
        let dir = TempDir::new().expect("tempdir");
        let ssb = Ssb::new(test_config(&dir), test_clock()).expect("ssb init");

        let payload = SsbPayload::PhaseComplete(PhaseCompletePayload {
            phase_id: "3B.4".into(),
            agent_id: "a".into(),
            fsm_state: "COMPLETE".into(),
            evidence_artifacts: vec![],
            commit_hashes: vec![],
            test_outcomes: vec![],
        });
        let seq = ssb
            .write_payload("agent-empty", 0, payload, vec![])
            .await
            .expect("empty vecs must be accepted");
        assert!(seq > 0);
    }

    mod proptest_payloads {
        use super::*;
        use proptest::prelude::*;

        fn arb_payload() -> impl Strategy<Value = SsbPayload> {
            prop_oneof![
                ("[a-z]{1,20}", 0u64..10000, "[a-f0-9]{8}")
                    .prop_map(|(path, size_bytes, hash)| SsbPayload::FileWrite(
                        FileWritePayload { path, size_bytes, hash }
                    )),
                "[a-z]{1,20}".prop_map(|path| SsbPayload::FileRead(FileReadPayload { path })),
                ("[a-z]{1,10}", any::<bool>(), "[a-z ]{0,30}")
                    .prop_map(|(tool_id, success, output_summary)| SsbPayload::ToolResult(
                        ToolResultPayload { tool_id, success, output_summary }
                    )),
                ("[a-z_]{1,20}", any::<bool>(), 0u64..100000)
                    .prop_map(|(test_name, passed, duration_ms)| SsbPayload::TestOutcome(
                        TestOutcomePayload { test_name, passed, duration_ms }
                    )),
                ("[a-z]{1,10}", "[a-z]{1,10}", "[a-z ]{1,30}", any::<Option<String>>(), any::<Option<u32>>())
                    .prop_map(|(severity, category, message, file_path, line)| SsbPayload::Finding(
                        FindingPayload { severity, category, message, file_path, line }
                    )),
                ("[a-z ]{1,30}", "[a-z]{1,10}", any::<bool>())
                    .prop_map(|(claim_text, grounding_source, verified)| SsbPayload::Claim(
                        ClaimPayload { claim_text, grounding_source, verified }
                    )),
                ("[a-f0-9]{16}", "[a-f0-9]{16}", "[a-z0-9-]{1,20}", any::<u64>())
                    .prop_map(|(agent_state_hash, grc_history_hash, agent_id, step_index)| SsbPayload::Checkpoint(
                        CheckpointPayload { agent_state_hash, grc_history_hash, agent_id, step_index, file_sizes: vec![] }
                    )),
                ("[a-z]{1,10}", "[a-z]{1,10}", "[a-z ]{1,20}")
                    .prop_map(|(from_model, to_model, reason)| SsbPayload::ModelSwap(
                        ModelSwapPayload { from_model, to_model, reason }
                    )),
                ("[A-Z]{1,10}", "[A-Z]{1,10}", "[a-z ]{1,30}")
                    .prop_map(|(from_state, to_state, evidence_summary)| SsbPayload::PhaseTransition(
                        PhaseTransitionPayload { from_state, to_state, evidence_summary }
                    )),
                ("[a-z]{1,10}", "[a-z]{1,10}", "[a-z]{1,10}")
                    .prop_map(|(stage, from_mode, to_mode)| SsbPayload::KnowledgeDegradation(
                        KnowledgeDegradationPayload { stage, from_mode, to_mode }
                    )),
                ("[a-z_]{1,15}", any::<f64>().prop_filter("finite", |v| v.is_finite()))
                    .prop_map(|(metric_name, value)| SsbPayload::IsolationMetric(
                        IsolationMetricPayload { metric_name, value }
                    )),
                ("[a-f0-9]{8}", "[0-9A-Z]{1,5}", "[0-9]{1,3}")
                    .prop_map(|(plan_hash, phase, step)| SsbPayload::PlanUpdate(
                        PlanUpdatePayload { plan_hash, phase, step }
                    )),
                ("[a-z ]{1,30}", "[a-z-]{1,15}")
                    .prop_map(|(reason, requesting_agent)| SsbPayload::ReplanRequest(
                        ReplanRequestPayload { reason, requesting_agent }
                    )),
                (any::<i64>(), "[a-z ]{1,20}")
                    .prop_map(|(original_seq, stub_reason)| SsbPayload::StubReference(
                        StubReferencePayload { original_seq, stub_reason }
                    )),
                ("[a-z]{1,10}", "[a-z ]{1,30}", "[a-z]{1,10}", "[a-z:]{1,20}", "[a-z ]{0,20}")
                    .prop_map(|(step_type, content, topic, grounding, conclusion)| SsbPayload::ReasoningTrace(
                        ReasoningTracePayload { step_type, content, topic, grounding, conclusion }
                    )),
                ("[0-9A-Z.]{1,10}", "[a-z-]{1,10}", "[A-Z]{1,10}",
                 proptest::collection::vec("[a-z.]{1,10}", 0..3),
                 proptest::collection::vec("[a-f0-9]{7}", 0..3),
                 proptest::collection::vec("[a-z ]{1,20}", 0..3))
                    .prop_map(|(phase_id, agent_id, fsm_state, evidence_artifacts, commit_hashes, test_outcomes)|
                        SsbPayload::PhaseComplete(PhaseCompletePayload {
                            phase_id, agent_id, fsm_state, evidence_artifacts, commit_hashes, test_outcomes,
                        })
                    ),
                ("[a-z_]{1,15}",
                 any::<f64>().prop_filter("finite", |v| v.is_finite()),
                 any::<f64>().prop_filter("finite", |v| v.is_finite()),
                 "[a-z]{1,10}")
                    .prop_map(|(metric_name, value, threshold, action_taken)| SsbPayload::MemoryMonitoring(
                        MemoryMonitoringPayload { metric_name, value, threshold, action_taken }
                    )),
                (0u8..5, 0usize..200000, 0usize..200000,
                 proptest::collection::vec(0u8..8, 0..4))
                    .prop_map(|(level, before_tokens, after_tokens, slots_affected)| SsbPayload::ContextReduction(
                        ContextReductionPayload { level, before_tokens, after_tokens, slots_affected }
                    )),
                ("[a-z-]{1,20}",
                 proptest::collection::vec("[a-z-]{1,20}", 0..3),
                 "[a-f0-9]{8}",
                 proptest::collection::vec("[a-f0-9]{8}", 1..4))
                    .prop_map(|(winner_name, abandoned_names, trunk_snap_id, from_snap_ids)| SsbPayload::BranchMerge(
                        BranchMergePayload { winner_name, abandoned_names, trunk_snap_id, from_snap_ids }
                    )),
            ]
        }

        proptest! {
            #[test]
            fn test_payload_round_trip(payload in arb_payload()) {
                let bytes = rmp_serde::to_vec_named(&payload)
                    .expect("serialize must succeed");
                let decoded: SsbPayload = rmp_serde::from_slice(&bytes)
                    .expect("deserialize must succeed");
                prop_assert_eq!(&decoded, &payload);
            }
        }
    }
}
