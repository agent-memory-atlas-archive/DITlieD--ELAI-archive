//! [`FactStore`] — rusqlite-backed bi-temporal CRUD for `memory_facts`.
//!
//! The table layout collapses the Zep/Graphiti bi-temporal model (Zep §2.2.3)
//! into one relational table with `(valid_from, valid_to, ingested_at)`
//! per row + a separate `last_observed_at` mutable column for the decay
//! curve. The schema matches the SQL block in
//! `docs/thefinalglance/E-plan/E-5_user_modeling_and_memory.md` Task 1.

use std::path::Path;

use rusqlite::{params, Connection, OptionalExtension};

use crate::error::MemoryError;
use crate::fact::{FactType, MemoryFact, Scope};
use crate::trust::{Provenance, TrustTier};

/// Embedded `CREATE TABLE` statement. Idempotent (`IF NOT EXISTS`).
/// Indexed for the two hot retrieval paths:
///   * `idx_user_scope` — `WHERE user_id=? AND scope=? AND fact_type=?`
///   * `idx_valid_now` — partial index `WHERE valid_to IS NULL` (the
///     currently-valid hot set).
const CREATE_TABLE_SQL: &str = "\
CREATE TABLE IF NOT EXISTS memory_facts (
    fact_id TEXT PRIMARY KEY,
    user_id TEXT NOT NULL,
    seat_id TEXT NOT NULL DEFAULT 'local',
    project_id TEXT,
    scope TEXT NOT NULL,
    fact_type TEXT NOT NULL,
    key TEXT NOT NULL,
    value TEXT NOT NULL,
    confidence REAL NOT NULL,
    trust_score REAL NOT NULL,
    ingested_at_ms INTEGER NOT NULL,
    valid_from_ms INTEGER NOT NULL,
    valid_to_ms INTEGER,
    decay_rate_per_day REAL NOT NULL,
    last_observed_at_ms INTEGER NOT NULL,
    evidence_pointers TEXT NOT NULL,
    superseded_by TEXT,
    trust_tier TEXT NOT NULL DEFAULT 'T3',
    provenance TEXT NOT NULL DEFAULT '',
    last_validated_at_ms INTEGER
);
CREATE INDEX IF NOT EXISTS idx_memory_facts_user_scope
    ON memory_facts(user_id, scope, fact_type);
CREATE INDEX IF NOT EXISTS idx_memory_facts_valid_now
    ON memory_facts(user_id) WHERE valid_to_ms IS NULL;
CREATE INDEX IF NOT EXISTS idx_memory_facts_seat_user
    ON memory_facts(seat_id, user_id) WHERE valid_to_ms IS NULL;
";

/// Bi-temporal CRUD facade over the `memory_facts` SQLite table.
///
/// Multiple `FactStore`s pointed at the same path are safe (sqlite handles
/// the file-level locking) but should share an outer mutex if writes can
/// race; the E-5 producer path emits one fact at a time so a per-store
/// connection suffices.
pub struct FactStore {
    conn: Connection,
}

impl FactStore {
    /// Open (or create) the sqlite file at `path` and migrate the
    /// `memory_facts` table + indexes.
    pub fn open(path: impl AsRef<Path>) -> Result<Self, MemoryError> {
        if let Some(parent) = path.as_ref().parent() {
            // Best-effort dir create. If it already exists or cannot be
            // created we let the subsequent Connection::open surface the
            // typed sqlite error.
            let _ = std::fs::create_dir_all(parent);
        }
        let conn = Connection::open(path.as_ref())?;
        conn.execute_batch(CREATE_TABLE_SQL)?;
        // H-32 migration: add last_validated_at_ms column to existing DBs.
        // Idempotent: SQLite returns an error if the column already exists,
        // which we silently ignore (the column will be present after the
        // CREATE_TABLE_SQL on fresh DBs via the ALTER-guarded path below).
        let _ = conn.execute_batch(
            "ALTER TABLE memory_facts ADD COLUMN last_validated_at_ms INTEGER;"
        );
        // J-59 migration: add seat_id column to existing DBs. Idempotent:
        // SQLite returns an error if the column already exists, which we
        // silently ignore (fresh DBs have it via CREATE_TABLE_SQL).
        let _ = conn.execute_batch(
            "ALTER TABLE memory_facts ADD COLUMN seat_id TEXT NOT NULL DEFAULT 'local';"
        );
        conn.execute_batch("CREATE TABLE IF NOT EXISTS memory_fact_g1_taint (fact_id TEXT PRIMARY KEY, g1_trust TEXT NOT NULL);")?;
        Ok(Self { conn })
    }

    /// Open an in-memory store (used by tests + transient extractor
    /// pipelines).
    pub fn open_in_memory() -> Result<Self, MemoryError> {
        let conn = Connection::open_in_memory()?;
        conn.execute_batch(CREATE_TABLE_SQL)?;
        // In-memory stores always use the fresh schema (column included), so
        // the ALTER is a no-op but we run it anyway for consistency.
        let _ = conn.execute_batch(
            "ALTER TABLE memory_facts ADD COLUMN last_validated_at_ms INTEGER;"
        );
        let _ = conn.execute_batch(
            "ALTER TABLE memory_facts ADD COLUMN seat_id TEXT NOT NULL DEFAULT 'local';"
        );
        conn.execute_batch("CREATE TABLE IF NOT EXISTS memory_fact_g1_taint (fact_id TEXT PRIMARY KEY, g1_trust TEXT NOT NULL);")?;
        Ok(Self { conn })
    }

    /// Persist a G-1 memory taint label in the side table.
    pub fn persist_g1_taint(&self, fact_id: &str, label: &str) -> Result<(), MemoryError> {
        self.conn.execute(
            "INSERT INTO memory_fact_g1_taint (fact_id, g1_trust) VALUES (?1, ?2) ON CONFLICT(fact_id) DO UPDATE SET g1_trust=excluded.g1_trust",
            params![fact_id, label],
        )?;
        Ok(())
    }

    /// Read a persisted G-1 memory taint label, if present.
    pub fn g1_taint(&self, fact_id: &str) -> Result<Option<String>, MemoryError> {
        self.conn
            .query_row("SELECT g1_trust FROM memory_fact_g1_taint WHERE fact_id = ?1", params![fact_id], |row| row.get(0))
            .optional()
            .map_err(MemoryError::from)
    }

    /// Insert a new `MemoryFact` row. Validates invariants first; the SQL
    /// PRIMARY KEY enforces uniqueness on `fact_id`.
    pub fn insert(&self, fact: &MemoryFact) -> Result<(), MemoryError> {
        fact.validate()?;
        let evidence_json = serde_json::to_string(&fact.evidence_pointers)?;
        let value_json = serde_json::to_string(&fact.value)?;
        let provenance_json = serde_json::to_string(&fact.provenance)?;
        self.conn.execute(
            "INSERT INTO memory_facts (
                fact_id, user_id, seat_id, project_id, scope, fact_type, key, value,
                confidence, trust_score, ingested_at_ms, valid_from_ms,
                valid_to_ms, decay_rate_per_day, last_observed_at_ms,
                evidence_pointers, superseded_by,
                trust_tier, provenance, last_validated_at_ms
            ) VALUES (
                ?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8,
                ?9, ?10, ?11, ?12,
                ?13, ?14, ?15,
                ?16, ?17,
                ?18, ?19, ?20
            )",
            params![
                fact.fact_id,
                fact.user_id,
                fact.seat_id,
                fact.project_id,
                fact.scope.as_str(),
                fact.fact_type.as_str(),
                fact.key,
                value_json,
                fact.confidence,
                fact.trust_score,
                fact.ingested_at_ms as i64,
                fact.valid_from_ms as i64,
                fact.valid_to_ms.map(|v| v as i64),
                fact.decay_rate_per_day,
                fact.last_observed_at_ms as i64,
                evidence_json,
                fact.superseded_by,
                fact.trust_tier.as_str(),
                provenance_json,
                fact.last_validated_at_ms.map(|v| v as i64),
            ],
        )?;
        Ok(())
    }

    /// Fetch a single fact by `fact_id`.
    pub fn get(&self, fact_id: &str) -> Result<Option<MemoryFact>, MemoryError> {
        let mut stmt = self.conn.prepare(
            "SELECT fact_id, user_id, seat_id, project_id, scope, fact_type, key, value,
                    confidence, trust_score, ingested_at_ms, valid_from_ms,
                    valid_to_ms, decay_rate_per_day, last_observed_at_ms,
                    evidence_pointers, superseded_by,
                    trust_tier, provenance, last_validated_at_ms
             FROM memory_facts
             WHERE fact_id = ?1",
        )?;
        let row = stmt
            .query_row(params![fact_id], row_to_fact)
            .optional()?;
        row.transpose().map_err(Into::into)
    }

    /// List currently-valid facts (`valid_to_ms IS NULL`) for a user +
    /// optional scope / fact_type filter. Used by `get_memory_facts`
    /// Tauri command.
    pub fn list_current(
        &self,
        user_id: &str,
        scope: Option<Scope>,
        fact_type: Option<FactType>,
        limit: usize,
    ) -> Result<Vec<MemoryFact>, MemoryError> {
        let mut sql = String::from(
            "SELECT fact_id, user_id, seat_id, project_id, scope, fact_type, key, value,
                    confidence, trust_score, ingested_at_ms, valid_from_ms,
                    valid_to_ms, decay_rate_per_day, last_observed_at_ms,
                    evidence_pointers, superseded_by,
                    trust_tier, provenance, last_validated_at_ms
             FROM memory_facts
             WHERE user_id = ?1 AND valid_to_ms IS NULL",
        );
        // Parameter index already 1; rusqlite borrows via params! so we
        // build the predicate string then collect parameters.
        let scope_tag = scope.map(|s| s.as_str().to_owned());
        let type_tag = fact_type.map(|t| t.as_str().to_owned());
        if scope_tag.is_some() {
            sql.push_str(" AND scope = ?2");
        }
        if type_tag.is_some() {
            // ?3 always when scope present; ?2 otherwise.
            sql.push_str(if scope_tag.is_some() {
                " AND fact_type = ?3"
            } else {
                " AND fact_type = ?2"
            });
        }
        sql.push_str(" ORDER BY last_observed_at_ms DESC LIMIT ");
        sql.push_str(&limit.to_string());

        let mut stmt = self.conn.prepare(&sql)?;
        let rows: Vec<Result<MemoryFact, MemoryError>> =
            match (scope_tag.as_deref(), type_tag.as_deref()) {
                (Some(s), Some(t)) => stmt
                    .query_map(params![user_id, s, t], row_to_fact)?
                    .map(|r| r.map_err(Into::into).and_then(|x| x))
                    .collect(),
                (Some(s), None) => stmt
                    .query_map(params![user_id, s], row_to_fact)?
                    .map(|r| r.map_err(Into::into).and_then(|x| x))
                    .collect(),
                (None, Some(t)) => stmt
                    .query_map(params![user_id, t], row_to_fact)?
                    .map(|r| r.map_err(Into::into).and_then(|x| x))
                    .collect(),
                (None, None) => stmt
                    .query_map(params![user_id], row_to_fact)?
                    .map(|r| r.map_err(Into::into).and_then(|x| x))
                    .collect(),
            };
        rows.into_iter().collect()
    }

    /// List ALL facts for `user_id` including CLOSED ones (valid_to_ms IS NOT
    /// NULL). Used by the staleness benchmark broken-retrieve negative control
    /// (H-17 staleness_runner.rs) to model the EXACT same retrieval pipeline
    /// but WITHOUT the valid_to_ms live-only filter — the ONLY difference from
    /// the real retriever.
    ///
    /// Optional `fact_type` filter mirrors `list_current`. `limit` is a
    /// CANDIDATE cap (caller applies further filters after).
    ///
    /// NEVER used in production retrieval; only in the benchmark negative arm.
    pub fn list_including_closed(
        &self,
        user_id: &str,
        fact_type: Option<FactType>,
        limit: usize,
    ) -> Result<Vec<MemoryFact>, MemoryError> {
        let type_tag = fact_type.map(|t| t.as_str().to_owned());
        let sql = if type_tag.is_some() {
            format!(
                "SELECT fact_id, user_id, seat_id, project_id, scope, fact_type, key, value,
                        confidence, trust_score, ingested_at_ms, valid_from_ms,
                        valid_to_ms, decay_rate_per_day, last_observed_at_ms,
                        evidence_pointers, superseded_by, trust_tier, provenance, last_validated_at_ms
                 FROM memory_facts
                 WHERE user_id = ?1 AND fact_type = ?2
                 ORDER BY last_observed_at_ms DESC LIMIT {limit}"
            )
        } else {
            format!(
                "SELECT fact_id, user_id, seat_id, project_id, scope, fact_type, key, value,
                        confidence, trust_score, ingested_at_ms, valid_from_ms,
                        valid_to_ms, decay_rate_per_day, last_observed_at_ms,
                        evidence_pointers, superseded_by, trust_tier, provenance, last_validated_at_ms
                 FROM memory_facts
                 WHERE user_id = ?1
                 ORDER BY last_observed_at_ms DESC LIMIT {limit}"
            )
        };
        let mut stmt = self.conn.prepare(&sql)?;
        let rows: Vec<Result<MemoryFact, MemoryError>> = if let Some(ref t) = type_tag {
            stmt.query_map(params![user_id, t], row_to_fact)?
                .map(|r| r.map_err(Into::into).and_then(|x| x))
                .collect()
        } else {
            stmt.query_map(params![user_id], row_to_fact)?
                .map(|r| r.map_err(Into::into).and_then(|x| x))
                .collect()
        };
        rows.into_iter().collect()
    }

    /// Supersede `old_fact_id` with `new_fact`. Atomic in a single
    /// transaction: closes the old row (`valid_to_ms = now_ms`) and links
    /// it to the new row via `superseded_by`, then inserts the new row.
    /// Per Zep §2.2.3 + the E-5 plan Tests::bi_temporal_round_trip.
    pub fn supersede(
        &mut self,
        old_fact_id: &str,
        new_fact: &MemoryFact,
        now_ms: u128,
    ) -> Result<(), MemoryError> {
        new_fact.validate()?;
        let tx = self.conn.transaction()?;
        let updated = tx.execute(
            "UPDATE memory_facts
             SET valid_to_ms = ?2, superseded_by = ?3
             WHERE fact_id = ?1 AND valid_to_ms IS NULL",
            params![old_fact_id, now_ms as i64, new_fact.fact_id],
        )?;
        if updated == 0 {
            return Err(MemoryError::NotFound(old_fact_id.to_owned()));
        }
        let evidence_json = serde_json::to_string(&new_fact.evidence_pointers)?;
        let value_json = serde_json::to_string(&new_fact.value)?;
        let provenance_json = serde_json::to_string(&new_fact.provenance)?;
        tx.execute(
            "INSERT INTO memory_facts (
                fact_id, user_id, seat_id, project_id, scope, fact_type, key, value,
                confidence, trust_score, ingested_at_ms, valid_from_ms,
                valid_to_ms, decay_rate_per_day, last_observed_at_ms,
                evidence_pointers, superseded_by,
                trust_tier, provenance, last_validated_at_ms
            ) VALUES (
                ?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8,
                ?9, ?10, ?11, ?12,
                ?13, ?14, ?15,
                ?16, ?17,
                ?18, ?19, ?20
            )",
            params![
                new_fact.fact_id,
                new_fact.user_id,
                new_fact.seat_id,
                new_fact.project_id,
                new_fact.scope.as_str(),
                new_fact.fact_type.as_str(),
                new_fact.key,
                value_json,
                new_fact.confidence,
                new_fact.trust_score,
                new_fact.ingested_at_ms as i64,
                new_fact.valid_from_ms as i64,
                new_fact.valid_to_ms.map(|v| v as i64),
                new_fact.decay_rate_per_day,
                new_fact.last_observed_at_ms as i64,
                evidence_json,
                new_fact.superseded_by,
                new_fact.trust_tier.as_str(),
                provenance_json,
                new_fact.last_validated_at_ms.map(|v| v as i64),
            ],
        )?;
        tx.commit()?;
        Ok(())
    }

    /// Apply a (fact_id, new_trust_score, new_last_observed_at) update —
    /// used by [`crate::decay_tick`] and on read-update from
    /// [`crate::get_relevant_facts`].
    pub fn update_trust(
        &self,
        fact_id: &str,
        new_trust: f32,
        new_last_observed_ms: u128,
    ) -> Result<(), MemoryError> {
        if !(0.0..=1.0).contains(&new_trust) {
            return Err(MemoryError::Invalid(format!(
                "new_trust out of [0,1]: {new_trust}"
            )));
        }
        let n = self.conn.execute(
            "UPDATE memory_facts
             SET trust_score = ?2, last_observed_at_ms = ?3
             WHERE fact_id = ?1",
            params![fact_id, new_trust, new_last_observed_ms as i64],
        )?;
        if n == 0 {
            return Err(MemoryError::NotFound(fact_id.to_owned()));
        }
        Ok(())
    }

    /// Soft-close a currently-valid row by setting `valid_to_ms` to
    /// `now_ms` without inserting a replacement. Used by the Mem0
    /// dispatcher's DELETE path (explicit `{"tombstone": true}`
    /// candidates) and by [`crate::contradiction`] supersession of
    /// older conflicting rows.
    ///
    /// Returns `MemoryError::NotFound` if the row does not exist OR is
    /// already closed (`valid_to_ms IS NOT NULL`). Idempotent semantics
    /// must be implemented by the caller — a second soft_close on the
    /// same id is a NotFound, not a success.
    pub fn soft_close(&mut self, fact_id: &str, now_ms: u128) -> Result<(), MemoryError> {
        let updated = self.conn.execute(
            "UPDATE memory_facts
             SET valid_to_ms = ?2
             WHERE fact_id = ?1 AND valid_to_ms IS NULL",
            params![fact_id, now_ms as i64],
        )?;
        if updated == 0 {
            return Err(MemoryError::NotFound(fact_id.to_owned()));
        }
        Ok(())
    }

    /// H-32: stamp `last_validated_at_ms` on a row after a successful
    /// citation re-check. Non-fatal caller pattern: the caller logs + continues
    /// on `NotFound` (fact may have been soft-closed by a concurrent tick).
    pub fn update_validated_at(&self, fact_id: &str, now_ms: u128) -> Result<(), MemoryError> {
        let n = self.conn.execute(
            "UPDATE memory_facts
             SET last_validated_at_ms = ?2
             WHERE fact_id = ?1",
            params![fact_id, now_ms as i64],
        )?;
        if n == 0 {
            return Err(MemoryError::NotFound(fact_id.to_owned()));
        }
        Ok(())
    }

    /// Mutable iteration helper for the decay scheduler — yields every
    /// currently-valid row's (`fact_id`, `fact_type`, `trust_score`,
    /// `decay_rate_per_day`, `last_observed_at_ms`).
    pub fn iter_current(
        &self,
    ) -> Result<Vec<DecayInputRow>, MemoryError> {
        let mut stmt = self.conn.prepare(
            "SELECT fact_id, fact_type, trust_score, decay_rate_per_day,
                    last_observed_at_ms
             FROM memory_facts
             WHERE valid_to_ms IS NULL",
        )?;
        let rows = stmt.query_map([], |r| {
            Ok(DecayInputRow {
                fact_id: r.get::<_, String>(0)?,
                fact_type_tag: r.get::<_, String>(1)?,
                trust_score: r.get::<_, f64>(2)? as f32,
                decay_rate_per_day: r.get::<_, f64>(3)? as f32,
                last_observed_at_ms: r.get::<_, i64>(4)? as u128,
            })
        })?;
        let mut out = Vec::new();
        for r in rows {
            out.push(r?);
        }
        Ok(out)
    }
}

/// Lightweight projection used by the decay scheduler — avoids
/// reconstructing the full [`MemoryFact`] when only the trust-related
/// columns matter.
#[derive(Debug, Clone)]
pub struct DecayInputRow {
    /// Row primary key.
    pub fact_id: String,
    /// `fact_type` wire tag.
    pub fact_type_tag: String,
    /// Current persisted trust score.
    pub trust_score: f32,
    /// Persisted decay rate (per day).
    pub decay_rate_per_day: f32,
    /// Persisted last-observed-at (ms).
    pub last_observed_at_ms: u128,
}

/// rusqlite row -> [`MemoryFact`] conversion. Returns `Result<MemoryFact,
/// MemoryError>` wrapped in `Ok` so query_map can collect; serde / enum
/// errors propagate via the inner Result.
fn row_to_fact(row: &rusqlite::Row<'_>) -> rusqlite::Result<Result<MemoryFact, MemoryError>> {
    let fact_id: String = row.get(0)?;
    let user_id: String = row.get(1)?;
    let seat_id: String = row.get(2)?;
    let project_id: Option<String> = row.get(3)?;
    let scope_tag: String = row.get(4)?;
    let fact_type_tag: String = row.get(5)?;
    let key: String = row.get(6)?;
    let value_text: String = row.get(7)?;
    let confidence: f64 = row.get(8)?;
    let trust_score: f64 = row.get(9)?;
    let ingested_at: i64 = row.get(10)?;
    let valid_from: i64 = row.get(11)?;
    let valid_to: Option<i64> = row.get(12)?;
    let decay_rate: f64 = row.get(13)?;
    let last_observed: i64 = row.get(14)?;
    let evidence_text: String = row.get(15)?;
    let superseded_by: Option<String> = row.get(16)?;
    let trust_tier_tag: String = row.get(17)?;
    let provenance_text: String = row.get(18)?;
    let last_validated_at: Option<i64> = row.get(19)?;

    let value: serde_json::Value = match serde_json::from_str(&value_text) {
        Ok(v) => v,
        Err(e) => return Ok(Err(MemoryError::from(e))),
    };
    let evidence_pointers: Vec<String> = match serde_json::from_str(&evidence_text) {
        Ok(v) => v,
        Err(e) => return Ok(Err(MemoryError::from(e))),
    };
    let scope = match Scope::from_str(&scope_tag) {
        Ok(v) => v,
        Err(e) => return Ok(Err(e)),
    };
    let fact_type = match FactType::from_str(&fact_type_tag) {
        Ok(v) => v,
        Err(e) => return Ok(Err(e)),
    };

    let trust_tier = match TrustTier::from_str(&trust_tier_tag) {
        Ok(v) => v,
        Err(e) => return Ok(Err(e)),
    };
    let provenance = match serde_json::from_str::<Provenance>(&provenance_text) {
        Ok(v) => v,
        Err(e) => return Ok(Err(MemoryError::from(e))),
    };

    Ok(Ok(MemoryFact {
        fact_id,
        user_id,
        seat_id,
        project_id,
        scope,
        fact_type,
        key,
        value,
        confidence: confidence as f32,
        trust_score: trust_score as f32,
        ingested_at_ms: ingested_at as u128,
        valid_from_ms: valid_from as u128,
        valid_to_ms: valid_to.map(|v| v as u128),
        decay_rate_per_day: decay_rate as f32,
        last_observed_at_ms: last_observed as u128,
        evidence_pointers,
        superseded_by,
        trust_tier,
        provenance,
        last_validated_at_ms: last_validated_at.map(|v| v as u128),
    }))
}
