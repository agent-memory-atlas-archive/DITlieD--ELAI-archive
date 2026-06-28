#![allow(unused_imports)]
// SSB store: Ssb handle, pool, migration, writer, reader, ReflectEventSink.
// Tier-B Move 1 split from ssb.rs (behavior-preserving code movement, df18539).

use std::collections::BTreeMap;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::{Duration, UNIX_EPOCH};

use rusqlite::types::Value;
use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};
use std::rc::Rc;
use tokio::sync::{broadcast, mpsc, oneshot};

use elai_core::clock::ClockProvider;
use elai_reasoning::chat_reflect::{ReflectEvent, ReflectEventSink};

// E-25 (SCHEMA_VERSION 172) -- claude-code-architecture catalog payload
// type used in `SsbPayload::ClaudeCodeArchitectureEvent`. The catalog enum
// + kind discriminator are imported inside the `tests` module where the
// sample-payload constructors live.
use crate::architecture::claude_code_arch::ClaudeCodeArchitectureEventPayload;

// E-29 Tier-B (SCHEMA_VERSION 173) -- agentic-observability event payload
// type used in `SsbPayload::AgenticObservabilityEvent`. The payload + 4-
// arm kind discriminator live inline below (`AgenticObservabilityEventPayload`
// + `AgenticObservabilityEventKind`) since the observability subtree
// (crate::observability) consumes the payload to build the row but does
// not own the wire-format definition. Reuses crate::mast::MastMode so
// the MastFailureClassified arm carries the canonical 14-variant tag
// without duplicating the taxonomy.
use crate::mast::MastMode;

use super::*;
use crate::ssb::{schema::*, action_type::*, payloads::*};

impl Ssb {
    /// Current wall-clock time in milliseconds since UNIX_EPOCH, sourced from
    /// the injected [`ClockProvider`]. Used by callers (e.g. the dashboard
    /// process) that must avoid raw `std::time::SystemTime` per the entropy
    /// quarantine rule.
    ///
    /// Saturation semantics (intentional, not silent error swallowing):
    /// * `duration_since(UNIX_EPOCH)` only fails on a system clock set before
    ///   1970-01-01; we treat that as t=0 since SSB timestamps are advisory
    ///   and we do not want a panic at the IPC boundary.
    /// * `i64::try_from(u128)` only fails past year 292278994; we saturate
    ///   to `i64::MAX` because no caller in 2026 should ever see overflow,
    ///   and a saturated max is harmless for monotonic comparisons.
    pub fn now_ms_via_clock(clock: &Arc<dyn ClockProvider>) -> i64 {
        let ms_u128 = clock
            .system_now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or(Duration::ZERO)
            .as_millis();
        i64::try_from(ms_u128).unwrap_or(i64::MAX)
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
                .map(|n| u32::try_from(n.get()).unwrap_or(u32::MAX))
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
    Pool(String),

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

    #[error("action_type '{declared}' does not match deserialized payload type '{actual}'")]
    ActionTypeMismatch { declared: String, actual: String },

    /// E-46: payload-level validation failure (e.g. empty required-non-empty
    /// field). Distinct from `ActionTypeMismatch` (which catches enum-vs-payload
    /// drift) — this variant covers producer-side stub_ban predicates that
    /// run BEFORE serialization. Carries a free-form reason for diagnostics.
    #[error("payload validation failed: {0}")]
    PayloadValidation(String),
}

// ---------------------------------------------------------------------------
// Async connection pool (deadpool managed, rusqlite 0.39)
// ---------------------------------------------------------------------------

/// Applies per-connection pragmas on pool connection creation.
/// `foreign_keys` is per-connection and must be set on every connection.
///
/// NOTE: This intentionally includes `load_module(conn)` for rarray support,
/// which the writer-side `apply_pragmas` does NOT include (rarray is only
/// needed for read-path queries like query_by_file and query_conflict).
fn apply_pool_pragmas(conn: &Connection) -> Result<(), rusqlite::Error> {
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

/// Deadpool manager for rusqlite connections.
/// Opens connections in a blocking task and applies pragmas on creation.
pub(crate) struct SqliteManager {
    db_path: String,
}

impl deadpool::managed::Manager for SqliteManager {
    type Type = Connection;
    type Error = SsbError;

    async fn create(&self) -> Result<Connection, SsbError> {
        let path = self.db_path.clone();
        tokio::task::spawn_blocking(move || {
            let conn = Connection::open(&path)?;
            apply_pool_pragmas(&conn)?;
            Ok(conn)
        })
        .await
        .map_err(|e| SsbError::Pool(format!("spawn_blocking join error: {e}")))?
    }

    async fn recycle(
        &self,
        conn: &mut Connection,
        _metrics: &deadpool::managed::Metrics,
    ) -> deadpool::managed::RecycleResult<SsbError> {
        // Clear any stale implicit transaction from an interrupted read path.
        conn.execute_batch("ROLLBACK").ok();
        // Validate the connection is still usable with a cheap query.
        conn.execute_batch("SELECT 1")
            .map_err(|e| deadpool::managed::RecycleError::Backend(SsbError::Sqlite(e)))?;
        Ok(())
    }
}

/// Type alias for the deadpool-managed rusqlite pool.
pub(crate) type SsbPool = deadpool::managed::Pool<SqliteManager>;

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

    // Query patterns for new action types (3J.12)
    /// Query reasoning_trace entries for a specific agent, ordered by seq.
    fn query_reasoning_trace(&self, agent_id: &str) -> Result<Vec<SsbEntry>, SsbError>;
    /// Query phase_complete entries for a specific phase.
    fn query_phase_complete(&self, phase_id: &str) -> Result<Vec<SsbEntry>, SsbError>;
    /// Query isolation_metric entries grouped by phase.
    fn query_isolation_metrics(&self) -> Result<Vec<SsbEntry>, SsbError>;
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
    pub(crate) read_pool: SsbPool,
    project_id: String,
    session_id: String,
    /// C-5: canonical DB path captured at construction so callers (e.g.
    /// task_runner cloud-mode hook) can open a sibling read-only
    /// `SsbViewer` without re-plumbing the SsbConfig through every
    /// hand-off site.
    db_path: std::path::PathBuf,
    shutdown: Arc<AtomicBool>,
    _write_task: tokio::task::JoinHandle<()>,
    /// I-18 STEP-1: rolling blake3 chain hash maintained by the single-writer.
    /// After each insert, this holds `Some(blake3_16hex(last_row_payload_bytes))`.
    /// `None` on GENESIS (no rows inserted yet). Exposed via
    /// [`Ssb::chain_prev_hash`] so the production CLI seal path
    /// (`elai run --emit-rra`) can stamp the chain link without re-deriving
    /// the full stream. Deterministic in-process: the single-writer holds the
    /// rolling state so no out-of-order insert can spoof the chain (IS-1).
    chain_rolling: Arc<std::sync::Mutex<Option<String>>>,
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
        // F-13: run_migration is widened to receive db_path + session_id +
        // project_id so it can cold-archive a stale-version ssb.db BEFORE the
        // destructive DROP (gated wire-dark) and emit a session_archived row via
        // a direct synchronous INSERT (the async writer below does not exist
        // yet). The DROP semantics are unchanged when the gate is disabled.
        run_migration(
            &writer_conn,
            &config.db_path,
            &config.session_id,
            &config.project_id,
        )?;

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

        // --- Read pool (deadpool managed, async-native) ---
        let db_path_str = config.db_path.to_string_lossy().to_string();
        let manager = SqliteManager { db_path: db_path_str };
        let read_pool = SsbPool::builder(manager)
            // NARROWING-OK: u32 pool size always fits usize on supported 32/64-bit targets.
            .max_size(config.effective_pool_size() as usize)
            .runtime(deadpool::Runtime::Tokio1)
            .build()
            .map_err(|e| SsbError::Pool(format!("pool build error: {e}")))?;

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

        // I-18 STEP-1: rolling blake3 chain hash shared between the single-writer
        // task and the Ssb handle. None on GENESIS; updated to blake3_16hex of
        // each inserted row's payload bytes after every successful commit.
        let chain_rolling = Arc::new(std::sync::Mutex::new(None::<String>));
        let chain_for_loop = chain_rolling.clone();

        let write_task = tokio::spawn(write_loop(
            writer_conn, write_rx, bc_tx, clock, pid, sid, sd, chain_for_loop,
        ));

        Ok(Self {
            write_tx: Some(write_tx),
            broadcast_tx,
            read_pool,
            project_id,
            session_id,
            db_path: config.db_path.clone(),
            shutdown,
            _write_task: write_task,
            chain_rolling,
        })
    }

    /// C-5 — canonical filesystem path of the underlying SQLite DB.
    /// Captured at construction; consumers (e.g. task_runner cloud-mode
    /// hook) open a sibling read-only [`crate::ssb_viewer::SsbViewer`]
    /// against this path without having to thread the original
    /// [`SsbConfig`] through every wiring hand-off.
    #[must_use]
    pub fn db_path(&self) -> &std::path::Path {
        &self.db_path
    }

    /// Submit a write request to the single-writer loop.
    ///
    /// # IPC sync contract (WARN-6)
    ///
    /// Callers **must** `.await` this method and receive the `seq` confirmation
    /// **before** dispatching any `ToolResult` to the agent. This guarantees
    /// the SSB entry is durable before the agent sees the result.
    ///
    /// # Type validation (3J.12)
    ///
    /// The payload bytes are deserialized and validated against the declared
    /// `action_type`. If they don't match, the write is rejected with
    /// `SsbError::ActionTypeMismatch`. This prevents silent corruption.
    pub async fn write(&self, request: SsbWriteRequest) -> Result<i64, SsbError> {
        // Validate that payload matches declared action_type (write-time validation)
        validate_payload_matches_action_type(&request.payload, request.action_type)?;

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

    /// I-18 STEP-1: the rolling blake3 chain_prev_hash maintained by the
    /// single-writer. Returns `Some(blake3_16hex)` of the most recently
    /// committed row's payload bytes, or `None` on GENESIS (no rows inserted
    /// yet). This is the chain link that the NEXT insert will carry as its
    /// `chain_prev_hash`.
    ///
    /// Deterministic in-process: the single-writer is the sole updater
    /// (IS-1), so no out-of-order insert can spoof the chain. The hash is
    /// blake3 (CLAUDE.md Anti-Stale-Training hard rule — never sha256/blake2).
    ///
    /// Production caller: `elai run --emit-rra` via
    /// `cli::run_subcommand::emit_run_artifact` stamps this value into the
    /// sealing log alongside the G-2 RRA bundle root.
    #[must_use]
    pub fn chain_prev_hash(&self) -> Option<String> {
        self.chain_rolling
            .lock()
            .ok()
            .and_then(|guard| guard.clone())
    }

    /// Return all entries for a session ordered by seq ASC (oldest first).
    ///
    /// Used by G-2/G-10 run_subcommand to derive the identity tuple for
    /// `build_rra` / model-drift detection from the full session stream.
    pub fn query_session_stream_asc(
        &self,
        session_id: &str,
    ) -> Result<Vec<SsbEntry>, SsbError> {
        let sid = session_id.to_owned();
        self.with_read_conn(move |conn| {
            let mut stmt = conn.prepare(
                "SELECT seq, project_id, session_id, agent_id, superstep, action_type, payload, created_at \
                 FROM ssb WHERE session_id = ?1 ORDER BY seq ASC",
            )?;
            let rows = stmt.query_map(rusqlite::params![sid], row_to_entry)?;
            rows.collect::<Result<Vec<_>, _>>().map_err(SsbError::Sqlite)
        })
    }

    /// Return all `ssb_files` rows for a session grouped by `seq`.
    ///
    /// Used by the B-10 context-budget compute_overlap path in `task_runner`
    /// to build per-seq file-affinity candidates from the REAL ssb_files
    /// junction table without owning open file handles.
    ///
    /// Returns `Ok(HashMap<seq, Vec<file_path>>)`. Returns an empty map on
    /// any read error (non-fatal; compute_overlap degrades gracefully).
    pub fn session_files_by_seq(
        &self,
        session_id: &str,
    ) -> Result<std::collections::BTreeMap<i64, Vec<String>>, SsbError> {
        let session_id = session_id.to_owned();
        self.with_read_conn(move |conn| {
            let mut stmt = conn.prepare(
                "SELECT ssb.seq, ssb_files.file_path \
                 FROM ssb_files \
                 JOIN ssb ON ssb_files.seq = ssb.seq \
                 WHERE ssb.session_id = ?1 \
                 ORDER BY ssb.seq",
            )?;
            let rows = stmt.query_map(rusqlite::params![session_id], |row| {
                Ok((row.get::<_, i64>(0)?, row.get::<_, String>(1)?))
            })?;
            let mut map: std::collections::BTreeMap<i64, Vec<String>> = std::collections::BTreeMap::new();
            for row in rows {
                let (seq, path) = row?;
                map.entry(seq).or_default().push(path);
            }
            Ok(map)
        })
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

/// Run the SSB schema migration on a freshly-opened writer connection.
///
/// F-13: when a STALE on-disk `user_version` is detected and the wire-dark
/// `ELAI_SSB_ARCHIVE_ON_BUMP_ENABLE` gate is set, the stale `ssb.db` is
/// cold-archived to `workflow/sessions/{session_id}/archive/ssb-v{N}.sqlite`
/// BEFORE the destructive DROP (the archive is ADDITIVE — the DROP still runs
/// unconditionally). After the recreate, a `session_archived` SSB row is emitted
/// via a DIRECT synchronous INSERT on this same `conn` (the async single-writer
/// task does not exist yet at migration time). Archive failures are fail-OPEN
/// (logged, the DROP continues) because blocking startup would brick the
/// session for data the code itself marks "acceptable to lose".
///
/// The signature is widened (vs the historical `(conn)`) to receive `db_path`
/// (the archive copy source) + `session_id` + `project_id` (for the synchronous
/// row INSERT). The ONLY caller is [`Ssb::new`], where all three live on
/// [`SsbConfig`]; the blast radius is contained to that one call site.
fn run_migration(
    conn: &Connection,
    db_path: &std::path::Path,
    session_id: &str,
    project_id: &str,
) -> Result<(), rusqlite::Error> {
    let version: i64 = conn.pragma_query_value(None, "user_version", |row| row.get(0))?;
    if version == SCHEMA_VERSION {
        return Ok(());
    }
    // SSB is session-scoped. Stale schema → drop and recreate (data loss
    // is acceptable for old sessions).
    //
    // F-13: best-effort forensic archive BEFORE the DROP. Captured here so the
    // synchronous INSERT after recreate can record a truthful archive_path +
    // manifest_blake3. None unless the gate is enabled AND the copy succeeded.
    // Carries (relative_archive_path, manifest_blake3) when a copy succeeded.
    let mut archived: Option<(String, String)> = None;
    if version > 0 {
        if crate::session_archive::archive_on_bump_enabled() {
            // WAL byte-faithfulness: apply_pragmas put the writer connection in
            // WAL mode, so un-checkpointed pages live in the -wal sidecar. Fold
            // them into the main db file BEFORE the copy so the archive's blake3
            // prefix matches a re-read of the archived file (acceptance gate).
            // Fail-open: a checkpoint error must not block the DROP.
            if let Err(e) = conn.pragma_update(None, "wal_checkpoint", "TRUNCATE") {
                tracing::warn!(
                    error = %e,
                    session_id,
                    "F-13: wal_checkpoint(TRUNCATE) before archive failed; archive may miss WAL pages"
                );
            }
            match crate::session_archive::archive_stale_ssb(db_path, session_id, version) {
                Ok(Some(dest)) => {
                    match crate::session_archive::archive_manifest_digest(&dest) {
                        Ok(digest) => {
                            let rel = dest.to_string_lossy().replace('\\', "/");
                            archived = Some((rel, digest));
                        }
                        Err(e) => tracing::warn!(
                            error = %e,
                            session_id,
                            "F-13: archive manifest digest failed; archive kept, row skipped"
                        ),
                    }
                }
                Ok(None) => {} // gate raced to disabled — treat as no-op.
                Err(e) => tracing::warn!(
                    error = %e,
                    session_id,
                    "F-13: stale-ssb archive failed; continuing the DROP (fail-open)"
                ),
            }
        }
        conn.execute_batch("DROP TABLE IF EXISTS ssb_files; DROP TABLE IF EXISTS ssb;")?;
    }
    conn.execute_batch(CREATE_SSB_TABLE)?;
    conn.execute_batch(CREATE_SSB_FILES_TABLE)?;
    conn.pragma_update(None, "user_version", SCHEMA_VERSION)?;

    // F-13: emit the session_archived beacon AFTER the recreate (the ssb table
    // exists by now) via a DIRECT synchronous INSERT. The async single-writer
    // task is not yet spawned and the #[cfg(debug_assertions)] append-only
    // update_hook is attached by Ssb::new AFTER this returns, so this insert is
    // safe. Fail-open: an emit error must not block startup.
    if let Some((rel_path, manifest_blake3)) = archived {
        if let Err(e) = insert_session_archived_row(
            conn,
            project_id,
            session_id,
            version,
            &rel_path,
            &manifest_blake3,
        ) {
            tracing::warn!(
                error = %e,
                session_id,
                "F-13: session_archived row insert failed (archive file kept on disk)"
            );
        }
    }
    Ok(())
}

/// Direct synchronous INSERT of one `session_archived` row on the writer
/// connection during migration. Mirrors the column set of `execute_write`
/// (project_id, session_id, agent_id, superstep, action_type, payload,
/// created_at). The payload is rmp_serde-encoded exactly like the async path.
fn insert_session_archived_row(
    conn: &Connection,
    project_id: &str,
    session_id: &str,
    stale_version: i64,
    rel_path: &str,
    manifest_blake3: &str,
) -> Result<(), SsbError> {
    let payload = SsbPayload::SessionArchived(SessionArchivedPayload {
        session_id: session_id.to_owned(),
        stale_version,
        archive_path: rel_path.to_owned(),
        manifest_blake3: manifest_blake3.to_owned(),
        store_kind: "ssb".to_owned(),
    });
    let bytes = rmp_serde::to_vec_named(&payload)?;
    let created_at = i64::try_from(
        std::time::SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or(Duration::ZERO)
            .as_secs(),
    )
    .unwrap_or(i64::MAX);
    conn.execute(
        "INSERT INTO ssb (project_id, session_id, agent_id, superstep, action_type, payload, created_at) \
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
        params![
            project_id,
            session_id,
            "ssb-migration",
            0_i64,
            SsbActionType::SessionArchived.as_sql_str(),
            bytes,
            created_at,
        ],
    )?;
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
    // I-18 STEP-1: rolling blake3 chain hash. Updated to blake3_16hex of
    // each committed row's payload bytes. None until the first successful
    // insert (GENESIS). The single-writer is the ONLY updater, so no
    // out-of-order insert can spoof the chain (IS-1 single-writer invariant).
    chain_rolling: Arc<std::sync::Mutex<Option<String>>>,
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
                // I-18 STEP-1: advance the rolling blake3 chain hash.
                // The chain_prev_hash for the NEXT row = blake3 of THIS
                // row's payload bytes (16-hex prefix, same convention as
                // rra.rs::truncate_hex / RRA_HEX_PREFIX_LEN). Computed
                // AFTER the commit so a failed write never advances the
                // chain. blake3 only (CLAUDE.md Anti-Stale-Training).
                let new_chain_hash: String = {
                    let full = blake3::hash(&msg.request.payload).to_hex();
                    full.chars().take(crate::rra::RRA_HEX_PREFIX_LEN).collect()
                };
                if let Ok(mut guard) = chain_rolling.lock() {
                    *guard = Some(new_chain_hash);
                } else {
                    tracing::warn!(
                        "I-18: chain_rolling mutex poisoned; rolling hash not advanced for seq {seq}"
                    );
                }

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
    let created_at = i64::try_from(
        clock
            .system_now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or(Duration::ZERO)
            .as_secs(),
    )
    .unwrap_or(i64::MAX);

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
// Sync→async bridge for deadpool reads
// ---------------------------------------------------------------------------

impl Ssb {
    /// Run a closure on a read-pool connection, bridging deadpool's async API
    /// to the sync SsbReader trait. Uses `block_in_place` to avoid blocking the
    /// tokio runtime when called from an async context.
    fn with_read_conn<F, T>(&self, f: F) -> Result<T, SsbError>
    where
        F: FnOnce(&Connection) -> Result<T, SsbError> + Send + 'static,
        T: Send + 'static,
    {
        let pool = self.read_pool.clone();
        match tokio::runtime::Handle::try_current() {
            Ok(handle) => match handle.runtime_flavor() {
                tokio::runtime::RuntimeFlavor::MultiThread => {
                    // Multi-thread runtime: block_in_place is safe.
                    tokio::task::block_in_place(|| {
                        handle.block_on(async move {
                            let obj = pool
                                .get()
                                .await
                                .map_err(|e| SsbError::Pool(format!("{e}")))?;
                            f(&obj)
                        })
                    })
                }
                _ => {
                    // CurrentThread runtime context (e.g. rt.block_on in
                    // try_save_snapshot): block_in_place panics here, and
                    // re-entering the same runtime deadlocks. Spawn an OS
                    // thread with its own transient runtime to drive the work.
                    std::thread::spawn(move || {
                        let rt = tokio::runtime::Builder::new_current_thread()
                            .enable_all()
                            .build()
                            .map_err(|e| SsbError::Pool(format!("rt build: {e}")))?;
                        rt.block_on(async move {
                            let obj = pool
                                .get()
                                .await
                                .map_err(|e| SsbError::Pool(format!("{e}")))?;
                            f(&obj)
                        })
                    })
                    .join()
                    .map_err(|_| SsbError::Pool("read worker panicked".into()))?
                }
            },
            Err(_) => {
                // No async context: spin transient current-thread runtime.
                let rt = tokio::runtime::Builder::new_current_thread()
                    .enable_all()
                    .build()
                    .map_err(|e| SsbError::Pool(format!("rt build: {e}")))?;
                rt.block_on(async move {
                    let obj = pool
                        .get()
                        .await
                        .map_err(|e| SsbError::Pool(format!("{e}")))?;
                    f(&obj)
                })
            }
        }
    }
}

// ---------------------------------------------------------------------------
// SsbReader implementation
// ---------------------------------------------------------------------------

impl SsbReader for Ssb {
    fn query_by_session(&self, session_id: &str) -> Result<Vec<SsbEntry>, SsbError> {
        let sid = session_id.to_owned();
        let pid = self.project_id.clone();
        self.with_read_conn(move |conn| {
            let mut stmt = conn.prepare(
                "SELECT seq, project_id, session_id, agent_id, superstep, action_type, payload, created_at \
                 FROM ssb WHERE session_id = ?1 AND project_id = ?2",
            )?;
            let rows = stmt.query_map(params![sid, pid], row_to_entry)?;
            rows.collect::<Result<Vec<_>, _>>().map_err(SsbError::Sqlite)
        })
    }

    fn query_by_file(&self, file_paths: &[&str]) -> Result<Vec<SsbEntry>, SsbError> {
        if file_paths.is_empty() {
            return Ok(vec![]);
        }
        let paths: Vec<String> = file_paths.iter().map(|p| (*p).to_owned()).collect();
        let pid = self.project_id.clone();
        self.with_read_conn(move |conn| {
            let values: Vec<Value> = paths.iter().map(|p| Value::Text(p.clone())).collect();
            let ptr = Rc::new(values);
            let mut stmt = conn.prepare(
                "SELECT DISTINCT ssb.seq, ssb.project_id, ssb.session_id, ssb.agent_id, \
                 ssb.superstep, ssb.action_type, ssb.payload, ssb.created_at \
                 FROM ssb JOIN ssb_files ON ssb_files.seq = ssb.seq \
                 WHERE ssb_files.file_path IN (SELECT value FROM rarray(?1)) \
                   AND ssb.project_id = ?2",
            )?;
            let rows = stmt.query_map(params![ptr, pid], row_to_entry)?;
            rows.collect::<Result<Vec<_>, _>>().map_err(SsbError::Sqlite)
        })
    }

    fn query_by_type(&self, action_type: SsbActionType) -> Result<Vec<SsbEntry>, SsbError> {
        let pid = self.project_id.clone();
        self.with_read_conn(move |conn| {
            let mut stmt = conn.prepare(
                "SELECT seq, project_id, session_id, agent_id, superstep, action_type, payload, created_at \
                 FROM ssb WHERE action_type = ?1 AND project_id = ?2",
            )?;
            let rows = stmt.query_map(params![action_type.as_sql_str(), pid], row_to_entry)?;
            rows.collect::<Result<Vec<_>, _>>().map_err(SsbError::Sqlite)
        })
    }

    fn query_by_superstep(&self, superstep: i64) -> Result<Vec<SsbEntry>, SsbError> {
        let pid = self.project_id.clone();
        self.with_read_conn(move |conn| {
            let mut stmt = conn.prepare(
                "SELECT seq, project_id, session_id, agent_id, superstep, action_type, payload, created_at \
                 FROM ssb WHERE superstep = ?1 AND project_id = ?2",
            )?;
            let rows = stmt.query_map(params![superstep, pid], row_to_entry)?;
            rows.collect::<Result<Vec<_>, _>>().map_err(SsbError::Sqlite)
        })
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
        let paths: Vec<String> = file_paths.iter().map(|p| (*p).to_owned()).collect();
        let sid = session_id.to_owned();
        let pid = self.project_id.clone();
        self.with_read_conn(move |conn| {
            let values: Vec<Value> = paths.iter().map(|p| Value::Text(p.clone())).collect();
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
                params![ptr, pid, sid, superstep],
                |row| row.get::<_, String>(0),
            )?;
            rows.collect::<Result<Vec<_>, _>>().map_err(SsbError::Sqlite)
        })
    }

    fn last_seq(&self) -> Result<Option<i64>, SsbError> {
        let pid = self.project_id.clone();
        self.with_read_conn(move |conn| {
            let mut stmt = conn.prepare("SELECT MAX(seq) FROM ssb WHERE project_id = ?1")?;
            let result: Option<i64> = stmt.query_row(params![pid], |row| row.get(0))?;
            Ok(result)
        })
    }

    fn query_by_seq(&self, seq: i64) -> Result<Option<SsbEntry>, SsbError> {
        let pid = self.project_id.clone();
        self.with_read_conn(move |conn| {
            let mut stmt = conn.prepare(
                "SELECT seq, project_id, session_id, agent_id, superstep, action_type, payload, created_at \
                 FROM ssb WHERE seq = ?1 AND project_id = ?2 LIMIT 1",
            )?;
            let mut rows = stmt.query_map(params![seq, pid], row_to_entry)?;
            match rows.next() {
                Some(row) => Ok(Some(row.map_err(SsbError::Sqlite)?)),
                None => Ok(None),
            }
        })
    }

    fn query_reasoning_trace(&self, agent_id: &str) -> Result<Vec<SsbEntry>, SsbError> {
        let aid = agent_id.to_owned();
        let pid = self.project_id.clone();
        self.with_read_conn(move |conn| {
            let mut stmt = conn.prepare(
                "SELECT seq, project_id, session_id, agent_id, superstep, action_type, payload, created_at \
                 FROM ssb WHERE action_type = ?1 AND agent_id = ?2 AND project_id = ?3 \
                 ORDER BY seq ASC",
            )?;
            let rows = stmt.query_map(
                params![SsbActionType::ReasoningTrace.as_sql_str(), aid, pid],
                row_to_entry,
            )?;
            rows.collect::<Result<Vec<_>, _>>().map_err(SsbError::Sqlite)
        })
    }

    fn query_phase_complete(&self, phase_id: &str) -> Result<Vec<SsbEntry>, SsbError> {
        let phase = phase_id.to_owned();
        let pid = self.project_id.clone();
        self.with_read_conn(move |conn| {
            let mut stmt = conn.prepare(
                "SELECT seq, project_id, session_id, agent_id, superstep, action_type, payload, created_at \
                 FROM ssb WHERE action_type = ?1 AND project_id = ?2",
            )?;
            let rows = stmt.query_map(
                params![SsbActionType::PhaseComplete.as_sql_str(), pid],
                row_to_entry,
            )?;
            let all_entries: Vec<SsbEntry> = rows.collect::<Result<Vec<_>, _>>().map_err(SsbError::Sqlite)?;

            // Filter by phase_id from payload (since phase_id is in the payload, not a separate column)
            let filtered = all_entries
                .into_iter()
                .filter(|entry| {
                    if let Ok(payload) = rmp_serde::from_slice::<PhaseCompletePayload>(&entry.payload) {
                        payload.phase_id == phase
                    } else {
                        false
                    }
                })
                .collect();
            Ok(filtered)
        })
    }

    fn query_isolation_metrics(&self) -> Result<Vec<SsbEntry>, SsbError> {
        let pid = self.project_id.clone();
        self.with_read_conn(move |conn| {
            let mut stmt = conn.prepare(
                "SELECT seq, project_id, session_id, agent_id, superstep, action_type, payload, created_at \
                 FROM ssb WHERE action_type = ?1 AND project_id = ?2",
            )?;
            let rows = stmt.query_map(
                params![SsbActionType::IsolationMetric.as_sql_str(), pid],
                row_to_entry,
            )?;
            rows.collect::<Result<Vec<_>, _>>().map_err(SsbError::Sqlite)
        })
    }
}

// ---------------------------------------------------------------------------
// Canonical SSB path helpers
// ---------------------------------------------------------------------------

/// Subdirectory (relative to a project/base directory) that holds the
/// managed sessions state, including the single SSB SQLite database.
// B-1: canonical SSB sessions subdir now under .elai_cc/sessions.
// SSB_SESSIONS_SUBDIR kept for source-compat (external callers); value updated.
pub const SSB_SESSIONS_SUBDIR: &str = ".elai_cc/sessions";

/// File name of the canonical SSB SQLite database inside
/// [`SSB_SESSIONS_SUBDIR`].
pub const SSB_DB_FILE_NAME: &str = "ssb.db";

/// Canonical SSB database path for the given project/base directory.
///
/// The CLI's sidecar bootstrap uses `<session-persistence-dir>/ssb.db`, and
/// managed session persistence files live under `<base>/.elai_cc/sessions/`
/// (see `session_control::managed_sessions_dir_for`). The dashboard must
/// open the SSB at the **same** path so it and a running CLI REPL share
/// telemetry instead of silently creating a second, empty database.
///
/// Returns `<base_dir>/.elai_cc/sessions/ssb.db` as a pure path computation
/// (no filesystem side effects — callers choose whether to create the
/// parent directory or let `Ssb::new`/`SsbViewer::open` do it).
pub fn default_ssb_path_for_base(base_dir: impl AsRef<std::path::Path>) -> std::path::PathBuf {
    base_dir
        .as_ref()
        .join(SSB_SESSIONS_SUBDIR)
        .join(SSB_DB_FILE_NAME)
}

/// Canonical SSB database path rooted at the current working directory.
///
/// Convenience wrapper for callers (dashboard binary, CLI) that treat CWD
/// as the project root. Returns `Err` only if `std::env::current_dir` fails.
pub fn default_ssb_path() -> std::io::Result<std::path::PathBuf> {
    Ok(default_ssb_path_for_base(std::env::current_dir()?))
}

// ---------------------------------------------------------------------------
// B-39: ReflectEventSink bridge — translates reasoning crate events to SSB rows.
// ---------------------------------------------------------------------------

impl ReflectEventSink for Ssb {
    /// Translates each [`ReflectEvent`] variant into the corresponding
    /// [`SsbActionType`] + [`SsbPayload`] and appends to the SSB.
    ///
    /// Write is fire-and-forget via `tokio::spawn`: the pipeline never awaits
    /// SSB persistence. Write failures are logged at WARN level and silently
    /// discarded — the pipeline must never propagate SSB errors to the caller.
    fn emit(&self, event: ReflectEvent) {
        let (action_type, payload) = match event {
            ReflectEvent::Triaged { turn_id, class } => (
                SsbActionType::ReflectTriaged,
                SsbPayload::ReflectTriaged(ReflectTriagedPayload {
                    turn_id,
                    class: class.as_wire_str().to_owned(),
                }),
            ),
            ReflectEvent::Candidate {
                turn_id,
                candidate_index,
                content,
            } => (
                SsbActionType::ReflectCandidate,
                SsbPayload::ReflectCandidate(ReflectCandidatePayload {
                    turn_id,
                    candidate_index,
                    content,
                }),
            ),
            ReflectEvent::AuditVerdict {
                turn_id,
                best_index,
                rationale,
            } => (
                SsbActionType::ReflectAuditVerdict,
                SsbPayload::ReflectAuditVerdict(ReflectAuditVerdictPayload {
                    turn_id,
                    best_index,
                    rationale,
                }),
            ),
            ReflectEvent::Refined {
                turn_id,
                total_tokens_used,
                synthesis,
            } => (
                SsbActionType::ReflectRefined,
                SsbPayload::ReflectRefined(ReflectRefinedPayload {
                    turn_id,
                    total_tokens_used,
                    synthesis,
                }),
            ),
            // Tranche B-6 quad B-39: 85% soft-warn beacon. Wire-dark
            // unless ELAI_B39_SOFTWARN_ENABLE=1. The env-gate is read at
            // emit-time (cheap: ASCII compare against a 4-char env var).
            ReflectEvent::BudgetSoftWarn {
                turn_id,
                used,
                soft_cap,
                hard_cap,
                pct,
            } => {
                if !env_gate_truthy(ELAI_B39_SOFTWARN_ENABLE_ENV) {
                    return;
                }
                (
                    SsbActionType::ReflectBudgetSoftWarn,
                    SsbPayload::ReflectBudgetSoftWarn(ReflectBudgetSoftWarnPayload {
                        turn_id_hash: blake3_label_prefix(&turn_id),
                        used,
                        soft_cap,
                        hard_cap,
                        pct,
                        ts_ms: system_time_ms_i64(),
                    }),
                )
            }
        };

        // Serialize payload eagerly on the caller's thread; spawn write async.
        let bytes = match rmp_serde::to_vec_named(&payload) {
            Ok(b) => b,
            Err(e) => {
                tracing::warn!(
                    target: "orchestrator::ssb",
                    error = %e,
                    action_type = action_type.as_sql_str(),
                    "ReflectEventSink: payload serialization failed — dropping event"
                );
                return;
            }
        };

        if bytes.len() > PAYLOAD_MAX_BYTES {
            tracing::warn!(
                target: "orchestrator::ssb",
                action_type = action_type.as_sql_str(),
                size = bytes.len(),
                max = PAYLOAD_MAX_BYTES,
                "ReflectEventSink: payload exceeds size limit — dropping event"
            );
            return;
        }

        let write_tx = match &self.write_tx {
            Some(tx) => tx.clone(),
            None => {
                tracing::warn!(
                    target: "orchestrator::ssb",
                    action_type = action_type.as_sql_str(),
                    "ReflectEventSink: SSB is shut down — dropping event"
                );
                return;
            }
        };

        let request = SsbWriteRequest {
            action_type,
            // Reflect events are emitted by the CLI chat loop; agent_id is
            // not meaningful here, so we use a stable sentinel.
            agent_id: "reflect_pipeline".to_owned(),
            superstep: 0,
            payload: bytes,
            files: vec![],
        };

        let (respond, _rx) = tokio::sync::oneshot::channel();
        if let Err(e) = write_tx.send(WriteMessage { request, respond }) {
            tracing::warn!(
                target: "orchestrator::ssb",
                error = %e,
                action_type = action_type.as_sql_str(),
                "ReflectEventSink: write channel send failed — dropping event"
            );
        }
        // _rx dropped intentionally: fire-and-forget; no await on the response.
    }
}

// ---------------------------------------------------------------------------
// B-38 (SCHEMA_VERSION 43) payload structs — appended at end of payloads region.
// Additive only. Existing payload structs are not renumbered or modified.
// ---------------------------------------------------------------------------

