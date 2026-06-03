//! C-47b: rusqlite-backed persistence layer for the C-47 case base.
//!
//! Layered on top of the in-memory [`super::registry::ToolCalibrationRegistry`]
//! LRU. The registry holds a hot working set per tool (CASE_BASE_LRU_CAP=100);
//! [`CasePersistStore`] is the cold tier that survives process restarts and
//! lets the registry rehydrate the hot set on bootstrap.
//!
//! Concurrency: [`tokio::sync::Mutex<Connection>`] (NOT std::sync::Mutex)
//! because persist methods are awaited from inside the tokio-runtime
//! `record_call` path on the registry. The single-writer sqlite model means
//! concurrent `persist_case` calls serialize on the inner mutex; for the
//! expected case-base traffic (1 persist per tool dispatch) that is fine.
//! For sustained high-throughput hot loops, the recommended future-rework
//! is `tokio::task::spawn_blocking` + a deadpool/r2d2 pool — see W3 in
//! workflow/sessions/s-22600/exploration/trancheD-integration.md and
//! D-trancheD-decisions.md D4 for the tradeoff rationale.
//!
//! Fail-closed posture (D3): when `persist_case` returns Err, the registry
//! caller MUST skip the LRU admit. This INVERTS the usual best-effort write
//! semantic; do NOT "simplify" by swallowing persist errors and admitting
//! anyway — that would silently lose the durability invariant.
//!
//! Wire-dark: persist methods are pure helpers; the env-gate
//! `ELAI_C47B_CASE_BASE_PERSIST_ENABLE` is read by the registry caller
//! (per `ToolCalibrationRegistry::record_call` post-D.1) so this module
//! has no env coupling and is unit-testable via `open_in_memory`.
//!
//! Schema: one idempotent `CREATE TABLE IF NOT EXISTS` block via
//! `execute_batch`. Schema is keyed on `(tool_name, call_shape_hash)` so
//! repeat observations of the same shape upsert into the same row.

use std::path::Path;
use std::sync::Arc;

use rusqlite::{params, Connection, OptionalExtension};
use serde::{Deserialize, Serialize};
use tokio::sync::Mutex;

use super::case_base::AggregatedCase;
use super::failure_profile::FailureProfile;
use super::registry::ToolCalibrationError;

/// Embedded `CREATE TABLE` + index statements. Idempotent
/// (`IF NOT EXISTS`). Indexed for the two hot retrieval paths:
///   * `idx_tool_calibration_cases_tool` — `WHERE tool_name=?` (rehydrate)
///   * `idx_tool_calibration_cases_blake3` — `WHERE blake3_prefix=?`
///     (cross-tool lookup; reserved for future plan)
const CREATE_TABLE_SQL: &str = "\
CREATE TABLE IF NOT EXISTS tool_calibration_cases (
    tool_name TEXT NOT NULL,
    call_shape_hash INTEGER NOT NULL,
    total_calls INTEGER NOT NULL,
    error_count INTEGER NOT NULL,
    last_response_kind TEXT NOT NULL,
    last_ts_ms INTEGER NOT NULL,
    complexity_score REAL NOT NULL,
    failure_profile_json TEXT NOT NULL,
    blake3_prefix INTEGER NOT NULL,
    created_at_ms INTEGER NOT NULL,
    updated_at_ms INTEGER NOT NULL,
    PRIMARY KEY (tool_name, call_shape_hash)
);
CREATE INDEX IF NOT EXISTS idx_tool_calibration_cases_tool
    ON tool_calibration_cases(tool_name);
CREATE INDEX IF NOT EXISTS idx_tool_calibration_cases_blake3
    ON tool_calibration_cases(blake3_prefix);
";

/// blake3-prefix `u64` over `(tool_name || \0 || call_shape_hash_le_bytes)`.
/// Mirrors the orchestrator-side `blake3_label_prefix` discipline; kept
/// crate-local so the persist module has no orchestrator dep.
fn case_blake3_prefix(tool_name: &str, call_shape_hash: u64) -> u64 {
    let mut hasher = blake3::Hasher::new();
    hasher.update(tool_name.as_bytes());
    hasher.update(b"\0");
    hasher.update(&call_shape_hash.to_le_bytes());
    let bytes = hasher.finalize();
    let prefix: [u8; 8] = bytes.as_bytes()[..8]
        .try_into()
        .unwrap_or([0u8; 8]);
    u64::from_le_bytes(prefix)
}

/// Persisted row shape — collapses the AggregatedCase plus
/// persistence-bookkeeping columns. Internal to this module.
#[derive(Debug, Clone, Serialize, Deserialize)]
struct PersistedCase {
    tool_name: String,
    call_shape_hash: u64,
    total_calls: u32,
    error_count: u32,
    last_response_kind: String,
    last_ts_ms: i64,
    complexity_score: f32,
    failure_profile: FailureProfile,
    blake3_prefix: u64,
    created_at_ms: i64,
    updated_at_ms: i64,
}

impl From<PersistedCase> for AggregatedCase {
    fn from(p: PersistedCase) -> Self {
        Self {
            tool_name: p.tool_name,
            call_shape_hash: p.call_shape_hash,
            total_calls: p.total_calls,
            error_count: p.error_count,
            failure_profile: p.failure_profile,
            last_response_kind: p.last_response_kind,
            last_ts_ms: p.last_ts_ms,
            complexity_score: p.complexity_score,
        }
    }
}

/// rusqlite-backed CRUD facade over the `tool_calibration_cases` table.
///
/// Wraps a single [`Connection`] behind a [`tokio::sync::Mutex`] (D4 —
/// SEPARATE from the SSB pool to avoid same-thread contention on
/// sqlite's single-writer model). Constructed once at bootstrap and
/// handed as `Arc<CasePersistStore>` to the [`super::registry::ToolCalibrationRegistry`]
/// via its `with_persist` builder.
pub struct CasePersistStore {
    conn: Arc<Mutex<Connection>>,
}

impl CasePersistStore {
    /// Open (or create) the sqlite file at `path` and migrate the
    /// `tool_calibration_cases` table + indexes. Idempotent.
    pub fn open(path: impl AsRef<Path>) -> Result<Self, ToolCalibrationError> {
        if let Some(parent) = path.as_ref().parent() {
            // Best-effort dir create. If it already exists or cannot be
            // created we let the subsequent Connection::open surface the
            // typed sqlite error.
            let _ = std::fs::create_dir_all(parent);
        }
        let conn = Connection::open(path.as_ref())
            .map_err(|e| ToolCalibrationError::Persist(e.to_string()))?;
        // C-47b CLI-arm: the case_base.sqlite3 file is opened by BOTH the
        // dashboard arm (AppState) and the CLI arm (McpClientDispatch). The
        // default rollback journal takes a whole-DB lock on write and the
        // default busy_timeout is 0, so a cross-process write contending
        // with the other process's lock would surface SQLITE_BUSY
        // immediately (record_call then fails-closed per D3 and silently
        // drops the case -- durability loss). WAL lets a reader and a
        // writer proceed concurrently and busy_timeout makes transient
        // contention retry for up to 5s instead of failing instantly.
        // Best-effort: a PRAGMA failure is non-fatal (the store still
        // works with default journal mode), so we log + continue rather
        // than refusing to open.
        if let Err(e) = conn.pragma_update(None, "journal_mode", "WAL") {
            tracing::warn!(
                error = %e,
                "C-47b: PRAGMA journal_mode=WAL failed; continuing with default journal mode"
            );
        }
        if let Err(e) = conn.busy_timeout(std::time::Duration::from_millis(5_000)) {
            tracing::warn!(
                error = %e,
                "C-47b: busy_timeout set failed; continuing with default (0ms)"
            );
        }
        conn.execute_batch(CREATE_TABLE_SQL)
            .map_err(|e| ToolCalibrationError::Persist(e.to_string()))?;
        Ok(Self {
            conn: Arc::new(Mutex::new(conn)),
        })
    }

    /// Open an in-memory store. Used by contract / unit tests and
    /// transient pipelines. Schema is migrated identically to [`Self::open`].
    pub fn open_in_memory() -> Result<Self, ToolCalibrationError> {
        let conn = Connection::open_in_memory()
            .map_err(|e| ToolCalibrationError::Persist(e.to_string()))?;
        conn.execute_batch(CREATE_TABLE_SQL)
            .map_err(|e| ToolCalibrationError::Persist(e.to_string()))?;
        Ok(Self {
            conn: Arc::new(Mutex::new(conn)),
        })
    }

    /// Insert-or-replace one case. `now_ms` is used for both
    /// `updated_at_ms` (always) and `created_at_ms` (only when the row
    /// is new; preserved via SELECT-then-INSERT on update). Returns
    /// Ok(()) on success; Err otherwise — the caller (registry) MUST
    /// fail-closed (skip LRU admit) on Err per D3.
    pub async fn persist_case(
        &self,
        case: &AggregatedCase,
        now_ms: i64,
    ) -> Result<(), ToolCalibrationError> {
        let conn = self.conn.lock().await;
        let blake3_prefix = case_blake3_prefix(&case.tool_name, case.call_shape_hash);
        let failure_json = serde_json::to_string(&case.failure_profile)
            .map_err(|e| ToolCalibrationError::Persist(format!("serialize failure_profile: {e}")))?;
        // Preserve created_at_ms across updates: COALESCE the existing
        // value if any, else use now_ms. Use a sub-SELECT in the INSERT
        // values list (sqlite supports this without a WITH clause).
        conn.execute(
            "INSERT OR REPLACE INTO tool_calibration_cases (
                tool_name, call_shape_hash, total_calls, error_count,
                last_response_kind, last_ts_ms, complexity_score,
                failure_profile_json, blake3_prefix, created_at_ms,
                updated_at_ms
            ) VALUES (
                ?1, ?2, ?3, ?4,
                ?5, ?6, ?7,
                ?8, ?9,
                COALESCE(
                    (SELECT created_at_ms FROM tool_calibration_cases
                       WHERE tool_name = ?1 AND call_shape_hash = ?2),
                    ?10
                ),
                ?11
            )",
            params![
                case.tool_name,
                case.call_shape_hash as i64,
                case.total_calls as i64,
                case.error_count as i64,
                case.last_response_kind,
                case.last_ts_ms,
                f64::from(case.complexity_score),
                failure_json,
                blake3_prefix as i64,
                now_ms,
                now_ms,
            ],
        )
        .map_err(|e| ToolCalibrationError::Persist(e.to_string()))?;
        Ok(())
    }

    /// Load one case by primary key. Returns Ok(None) on miss.
    pub async fn load_case(
        &self,
        tool_name: &str,
        call_shape_hash: u64,
    ) -> Result<Option<AggregatedCase>, ToolCalibrationError> {
        let conn = self.conn.lock().await;
        let mut stmt = conn
            .prepare(
                "SELECT tool_name, call_shape_hash, total_calls, error_count, \
                    last_response_kind, last_ts_ms, complexity_score, \
                    failure_profile_json, blake3_prefix, created_at_ms, \
                    updated_at_ms \
                 FROM tool_calibration_cases \
                 WHERE tool_name = ?1 AND call_shape_hash = ?2",
            )
            .map_err(|e| ToolCalibrationError::Persist(e.to_string()))?;
        let row = stmt
            .query_row(
                params![tool_name, call_shape_hash as i64],
                row_to_persisted_case,
            )
            .optional()
            .map_err(|e| ToolCalibrationError::Persist(e.to_string()))?;
        Ok(row.map(AggregatedCase::from))
    }

    /// Load up to `cap` cases for `tool_name`, ordered by `updated_at_ms`
    /// DESC. Used at bootstrap to rehydrate the LRU. ORDER BY DESC + LIMIT
    /// matches D5 — the most-recently-touched rows land at the LRU head
    /// when callers reinsert in reverse-iteration order.
    pub async fn load_all_for_tool(
        &self,
        tool_name: &str,
        cap: usize,
    ) -> Result<Vec<AggregatedCase>, ToolCalibrationError> {
        let conn = self.conn.lock().await;
        let mut stmt = conn
            .prepare(
                "SELECT tool_name, call_shape_hash, total_calls, error_count, \
                    last_response_kind, last_ts_ms, complexity_score, \
                    failure_profile_json, blake3_prefix, created_at_ms, \
                    updated_at_ms \
                 FROM tool_calibration_cases \
                 WHERE tool_name = ?1 \
                 ORDER BY updated_at_ms DESC \
                 LIMIT ?2",
            )
            .map_err(|e| ToolCalibrationError::Persist(e.to_string()))?;
        let cap_i64 = i64::try_from(cap).unwrap_or(i64::MAX);
        let rows = stmt
            .query_map(params![tool_name, cap_i64], row_to_persisted_case)
            .map_err(|e| ToolCalibrationError::Persist(e.to_string()))?;
        let mut out = Vec::with_capacity(cap.min(1024));
        for row in rows {
            let row = row.map_err(|e| ToolCalibrationError::Persist(e.to_string()))?;
            out.push(AggregatedCase::from(row));
        }
        Ok(out)
    }

    /// Enumerate distinct tool_names currently persisted. Used at
    /// bootstrap so the rehydrator does not need an a-priori tool list.
    pub async fn tool_names_from_disk(&self) -> Result<Vec<String>, ToolCalibrationError> {
        let conn = self.conn.lock().await;
        let mut stmt = conn
            .prepare("SELECT DISTINCT tool_name FROM tool_calibration_cases")
            .map_err(|e| ToolCalibrationError::Persist(e.to_string()))?;
        let rows = stmt
            .query_map([], |row| row.get::<_, String>(0))
            .map_err(|e| ToolCalibrationError::Persist(e.to_string()))?;
        let mut out: Vec<String> = Vec::new();
        for row in rows {
            out.push(row.map_err(|e| ToolCalibrationError::Persist(e.to_string()))?);
        }
        out.sort();
        Ok(out)
    }
}

/// Decode one `tool_calibration_cases` row into a [`PersistedCase`].
/// Centralised so both `load_case` and `load_all_for_tool` share the
/// column ordering invariant.
fn row_to_persisted_case(row: &rusqlite::Row<'_>) -> Result<PersistedCase, rusqlite::Error> {
    let call_shape_hash_i64: i64 = row.get(1)?;
    let total_calls_i64: i64 = row.get(2)?;
    let error_count_i64: i64 = row.get(3)?;
    let complexity_score_f64: f64 = row.get(6)?;
    let failure_json: String = row.get(7)?;
    let blake3_prefix_i64: i64 = row.get(8)?;
    let failure_profile: FailureProfile = serde_json::from_str(&failure_json)
        .map_err(|e| rusqlite::Error::FromSqlConversionFailure(
            7,
            rusqlite::types::Type::Text,
            Box::new(e),
        ))?;
    // We keep the snake_case string form on AggregatedCase (matches
    // the in-memory field type) so no ResponseKind enum round-trip is
    // needed at the persistence boundary.
    let last_response_kind: String = row.get(4)?;
    Ok(PersistedCase {
        tool_name: row.get(0)?,
        call_shape_hash: call_shape_hash_i64 as u64,
        total_calls: u32::try_from(total_calls_i64).unwrap_or(u32::MAX),
        error_count: u32::try_from(error_count_i64).unwrap_or(u32::MAX),
        last_response_kind,
        last_ts_ms: row.get(5)?,
        complexity_score: complexity_score_f64 as f32,
        failure_profile,
        blake3_prefix: blake3_prefix_i64 as u64,
        created_at_ms: row.get(9)?,
        updated_at_ms: row.get(10)?,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tool_calibration::case_base::ResponseKind;

    #[tokio::test]
    async fn persist_and_load_round_trip() {
        let store = CasePersistStore::open_in_memory().expect("open");
        let mut case = AggregatedCase::new("fs.read".into(), 0xDEAD_BEEFu64);
        case.observe(ResponseKind::Ok, r#"{"path":"x"}"#, 1_000);
        case.observe(ResponseKind::ErrInvalidArgs, r#"{"path":"x"}"#, 2_000);
        store.persist_case(&case, 2_000).await.expect("persist");
        let loaded = store
            .load_case("fs.read", 0xDEAD_BEEFu64)
            .await
            .expect("load");
        let loaded = loaded.expect("hit");
        assert_eq!(loaded.tool_name, "fs.read");
        assert_eq!(loaded.call_shape_hash, 0xDEAD_BEEFu64);
        assert_eq!(loaded.total_calls, 2);
        assert_eq!(loaded.error_count, 1);
        assert_eq!(loaded.last_response_kind, "err_invalid_args");
        assert_eq!(loaded.failure_profile.key_err, 1);
    }

    #[tokio::test]
    async fn load_case_miss_returns_none() {
        let store = CasePersistStore::open_in_memory().expect("open");
        let loaded = store
            .load_case("never.persisted", 42)
            .await
            .expect("load");
        assert!(loaded.is_none());
    }

    #[tokio::test]
    async fn load_all_for_tool_orders_by_updated_at_desc() {
        let store = CasePersistStore::open_in_memory().expect("open");
        for i in 0..5u64 {
            let mut case = AggregatedCase::new("fs.read".into(), i);
            case.observe(ResponseKind::Ok, r#"{}"#, i as i64);
            // Each persist call has a strictly increasing now_ms so the
            // updated_at_ms reflects insertion order; DESC must yield 4,3,2,1,0.
            store
                .persist_case(&case, 1_000 + i as i64)
                .await
                .expect("persist");
        }
        let all = store.load_all_for_tool("fs.read", 3).await.expect("load");
        assert_eq!(all.len(), 3);
        assert_eq!(all[0].call_shape_hash, 4);
        assert_eq!(all[1].call_shape_hash, 3);
        assert_eq!(all[2].call_shape_hash, 2);
    }

    #[tokio::test]
    async fn persist_updates_existing_row() {
        let store = CasePersistStore::open_in_memory().expect("open");
        let mut case = AggregatedCase::new("fs.read".into(), 42);
        case.observe(ResponseKind::Ok, r#"{}"#, 1_000);
        store.persist_case(&case, 1_000).await.expect("persist 1");
        case.observe(ResponseKind::ErrServer, r#"{}"#, 2_000);
        store.persist_case(&case, 2_000).await.expect("persist 2");
        let loaded = store
            .load_case("fs.read", 42)
            .await
            .expect("load")
            .expect("hit");
        assert_eq!(loaded.total_calls, 2);
        assert_eq!(loaded.error_count, 1);
        assert_eq!(loaded.last_response_kind, "err_server");
    }

    #[tokio::test]
    async fn tool_names_from_disk_returns_distinct_sorted() {
        let store = CasePersistStore::open_in_memory().expect("open");
        let a = AggregatedCase::new("git.commit".into(), 1);
        let b = AggregatedCase::new("fs.read".into(), 2);
        let c = AggregatedCase::new("fs.read".into(), 3);
        store.persist_case(&a, 1).await.expect("persist a");
        store.persist_case(&b, 2).await.expect("persist b");
        store.persist_case(&c, 3).await.expect("persist c");
        let names = store.tool_names_from_disk().await.expect("names");
        assert_eq!(names, vec!["fs.read".to_string(), "git.commit".to_string()]);
    }
}
