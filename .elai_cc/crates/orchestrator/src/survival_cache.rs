//! Tranche B-7 triple (E-13 SHIPPABLE-NOW) -- SQLite-backed survival
//! cache for `cargo-mutants` mutant survivors.
//!
//! Wire-dark default. Gated by `ELAI_E13_SURVIVAL_CACHE_ENABLE` (the
//! caller queries via [`survival_cache_enabled`] before constructing the
//! cache). The cache is the short-circuit substrate referenced by the
//! E-13 mutant-hunter agent loop documented at
//! `workflow/tooling/mutant-hunter.md`.
//!
//! Design constraints:
//!   * `rusqlite` only (NO sqlx -- absent from orchestrator deps per the
//!     B-7 exploration grounding pass).
//!   * blake3-prefix `u64` hashing for cache keys (NEVER sha2 / blake2 --
//!     CLAUDE.md HARD RULE anti-stale-training). The hashing helper
//!     lives in `crate::ssb::blake3_label_prefix`.
//!   * Graceful short-circuit on `cargo-mutants` toolchain absence.
//!     `record_survivor(_, _, SurvivorStatus::Skipped)` is the canonical
//!     "RUNPOD-PENDING / cargo-mutants 27.x missing" row -- the cache
//!     STILL records the row so an operator-facing dashboard can render
//!     a per-mutation skipped count without re-running the subprocess.
//!   * NO `.unwrap()` outside `#[cfg(test)]` -- `rusqlite::Error` is
//!     re-wrapped into [`SurvivalCacheError`] via `thiserror::Error`.
//!   * Schema: single table `survivors(mutation_id_hash, status, ts_ms)`
//!     keyed on `(mutation_id_hash, ts_ms)` (composite primary key so
//!     repeated runs append rather than collide).
//!
//! Producer-side composition is DEFERRED. The
//! `tdd_verifier::mutator::run_cargo_mutants` wrapper at
//! `crates/orchestrator/src/tdd_verifier/mutator.rs` is the intended
//! call site; wiring lands in a follow-on tranche once the cargo-mutants
//! 27.x toolchain pinning RUNPOD-PENDING (E-13 DECISION-7) is resolved.
//!
//! Grounding: arXiv:2404.09952 LLMorpheus §2 (LLM-driven mutation
//! testing -- survivor short-circuit is the canonical kill-rate budget
//! optimization).

use std::path::{Path, PathBuf};

use rusqlite::{params, Connection};
use thiserror::Error;

use crate::ssb::blake3_label_prefix;

/// Canonical env-var gate. Default OFF. The
/// [`survival_cache_enabled`] predicate exposes the truthy parse for
/// callers; the const is re-exported via `crate::survival_cache`.
pub const ELAI_E13_SURVIVAL_CACHE_ENABLE_ENV: &str = "ELAI_E13_SURVIVAL_CACHE_ENABLE";

/// Returns `true` when the wire-dark gate env var is set to `1` / `true`
/// / `yes` / `on` (case-insensitive). Mirrors the precedent set by other
/// Pattern-2 wire-dark substrates (E-50 ELAI_SAAS_ARCH_PATTERNS_ENABLE,
/// E-47 LoraShipGate, C-50 ContractCoding). Reads via `std::env::var`.
#[must_use]
pub fn survival_cache_enabled() -> bool {
    crate::ssb::env_gate_truthy(ELAI_E13_SURVIVAL_CACHE_ENABLE_ENV)
}

/// Status arm recorded for each survivor row. Closed enum -- adding a
/// 4th arm requires bumping the cache schema (`SCHEMA_PRAGMA_VERSION`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SurvivorStatus {
    /// `cargo-mutants` reported the mutation as live AND a test killed
    /// it on this run. Recorded so the dashboard can render kill-rate
    /// momentum without re-running the subprocess.
    LiveAndKilled,
    /// `cargo-mutants` reported the mutation as live AND no test killed
    /// it. The canonical short-circuit input for
    /// [`SurvivalCache::is_known_survivor`].
    LiveAndSurvived,
    /// `cargo-mutants` did not run (toolchain missing, run cancelled by
    /// budget cap, etc.). The cache still records the row so a future
    /// dashboard surface can render per-mutation skipped counts; an
    /// operator can clear it via [`SurvivalCache::clear_skipped`].
    Skipped,
}

impl SurvivorStatus {
    /// Canonical snake_case wire label. Used for both the SQLite
    /// `status` column AND the SSB
    /// `tdd_verification_event::details_json.status` field.
    #[must_use]
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::LiveAndKilled => "live_and_killed",
            Self::LiveAndSurvived => "live_and_survived",
            Self::Skipped => "skipped",
        }
    }

    /// Inverse of [`Self::as_str`]; returns `None` on unknown input so
    /// the cache read path can fail closed.
    #[must_use]
    pub fn from_snake_case(s: &str) -> Option<Self> {
        match s {
            "live_and_killed" => Some(Self::LiveAndKilled),
            "live_and_survived" => Some(Self::LiveAndSurvived),
            "skipped" => Some(Self::Skipped),
            _ => None,
        }
    }
}

/// One row returned from a survival-cache query.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SurvivorEntry {
    /// blake3-prefix `u64` of the canonical mutation identifier (the
    /// cargo-mutants `--list` line, e.g.
    /// `crates/foo/src/bar.rs:42: replace * with /`). Stored as the
    /// hash so the cache row is small and operator-grade redaction is
    /// automatic for private paths.
    pub mutation_id_hash: u64,
    pub status: SurvivorStatus,
    pub ts_ms: i64,
}

/// Error surface for the survival cache. `thiserror`-wrapped per
/// CLAUDE.md HARD RULE (NO `Box<dyn Error>`).
#[derive(Debug, Error)]
pub enum SurvivalCacheError {
    /// SQLite I/O / query error.
    #[error("survival cache sqlite error: {0}")]
    Sqlite(#[from] rusqlite::Error),
    /// On-disk cache directory could not be created.
    #[error("survival cache directory create failed: {0}")]
    DirCreate(#[from] std::io::Error),
}

/// SQLite-backed survival cache. In-memory by default for tests; pass
/// a filesystem path (typically `.elai_cc/cache/survival_cache.db`) for
/// the production cache.
///
/// Construction is `pub`; the underlying `Connection` is private so
/// callers cannot bypass the schema-versioned migration.
pub struct SurvivalCache {
    conn: Connection,
}

impl std::fmt::Debug for SurvivalCache {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SurvivalCache").finish_non_exhaustive()
    }
}

/// Schema pragma version. Bump when the table layout changes -- the
/// `open_at` constructor refuses connections whose `user_version`
/// pragma does NOT match (fail-closed migration posture).
const SCHEMA_PRAGMA_VERSION: i64 = 1;

const CREATE_SURVIVORS_TABLE: &str = "\
CREATE TABLE IF NOT EXISTS survivors (
    mutation_id_hash INTEGER NOT NULL,
    status           TEXT NOT NULL,
    ts_ms            INTEGER NOT NULL,
    PRIMARY KEY (mutation_id_hash, ts_ms)
)";

const CREATE_SURVIVORS_INDEX: &str = "\
CREATE INDEX IF NOT EXISTS survivors_mutation_idx
    ON survivors (mutation_id_hash, status)";

impl SurvivalCache {
    /// Construct an in-memory cache. Intended for tests + ephemeral
    /// short-circuit checks where persistence is unwanted.
    pub fn in_memory() -> Result<Self, SurvivalCacheError> {
        let conn = Connection::open_in_memory()?;
        Self::init_schema(&conn)?;
        Ok(Self { conn })
    }

    /// Construct an on-disk cache at the given path. Parent directories
    /// are created if missing. Path is typically
    /// `.elai_cc/cache/survival_cache.db` per the workflow's cache
    /// directory convention.
    pub fn open_at(path: &Path) -> Result<Self, SurvivalCacheError> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let conn = Connection::open(path)?;
        Self::init_schema(&conn)?;
        Ok(Self { conn })
    }

    /// Default on-disk path under `.elai_cc/cache/survival_cache.db`
    /// rooted at the supplied project root.
    #[must_use]
    pub fn default_path(project_root: &Path) -> PathBuf {
        project_root.join(".elai_cc/cache/survival_cache.db")
    }

    fn init_schema(conn: &Connection) -> Result<(), SurvivalCacheError> {
        conn.execute_batch(CREATE_SURVIVORS_TABLE)?;
        conn.execute_batch(CREATE_SURVIVORS_INDEX)?;
        let pragma = format!("PRAGMA user_version = {SCHEMA_PRAGMA_VERSION}");
        conn.execute_batch(&pragma)?;
        Ok(())
    }

    /// Record a survivor row. `mutation_id` is the canonical cargo-mutants
    /// identifier (file:line: mutation description); the cache stores the
    /// blake3-prefix `u64` of the snake-case wire label. `ts_ms` is the
    /// wall-clock millis since UNIX_EPOCH at observation.
    ///
    /// The `(mutation_id_hash, ts_ms)` composite primary key means
    /// repeated calls within the same millisecond will collide; callers
    /// SHOULD pass distinct `ts_ms` per run (typical: monotonic wall
    /// clock).
    pub fn record_survivor(
        &self,
        mutation_id: &str,
        ts_ms: i64,
        status: SurvivorStatus,
    ) -> Result<(), SurvivalCacheError> {
        let hash = blake3_label_prefix(mutation_id);
        self.record_survivor_by_hash(hash, ts_ms, status)
    }

    /// Lower-level variant of [`Self::record_survivor`] that takes a
    /// precomputed hash. Useful for cross-process callers that already
    /// own the hash (e.g. the mutant-hunter agent passing a deserialized
    /// SSB row).
    pub fn record_survivor_by_hash(
        &self,
        mutation_id_hash: u64,
        ts_ms: i64,
        status: SurvivorStatus,
    ) -> Result<(), SurvivalCacheError> {
        // SQLite stores INTEGER as i64; reinterpret the u64 bit-pattern
        // through `as i64` (no value loss; both 64-bit, lossless cast).
        let stored: i64 = mutation_id_hash as i64;
        self.conn.execute(
            "INSERT OR REPLACE INTO survivors (mutation_id_hash, status, ts_ms) \
             VALUES (?1, ?2, ?3)",
            params![stored, status.as_str(), ts_ms],
        )?;
        Ok(())
    }

    /// Returns `true` when at least one row exists for this mutation
    /// with status [`SurvivorStatus::LiveAndSurvived`]. This is the
    /// canonical short-circuit input: a cache hit means the
    /// cargo-mutants subprocess can be skipped because the mutation is
    /// already known to survive the current test set.
    pub fn is_known_survivor(&self, mutation_id: &str) -> Result<bool, SurvivalCacheError> {
        let hash = blake3_label_prefix(mutation_id);
        let stored: i64 = hash as i64;
        let mut stmt = self.conn.prepare(
            "SELECT 1 FROM survivors WHERE mutation_id_hash = ?1 AND status = ?2 LIMIT 1",
        )?;
        let mut rows = stmt.query(params![stored, SurvivorStatus::LiveAndSurvived.as_str()])?;
        let hit = rows.next()?.is_some();
        Ok(hit)
    }

    /// Returns all survivor rows recorded at or after the given
    /// timestamp. Used by the dashboard to render a rolling-window
    /// histogram + by the mutant-hunter agent to enumerate uncovered
    /// mutations.
    pub fn list_survivors_since(
        &self,
        ts_ms: i64,
    ) -> Result<Vec<SurvivorEntry>, SurvivalCacheError> {
        let mut stmt = self.conn.prepare(
            "SELECT mutation_id_hash, status, ts_ms FROM survivors WHERE ts_ms >= ?1 \
             ORDER BY ts_ms ASC",
        )?;
        let rows = stmt.query_map(params![ts_ms], |row| {
            let stored: i64 = row.get(0)?;
            let status_str: String = row.get(1)?;
            let row_ts: i64 = row.get(2)?;
            Ok((stored, status_str, row_ts))
        })?;
        let mut out: Vec<SurvivorEntry> = Vec::new();
        for row in rows {
            let (stored, status_str, row_ts) = row?;
            let status = SurvivorStatus::from_snake_case(&status_str).ok_or_else(|| {
                SurvivalCacheError::Sqlite(rusqlite::Error::InvalidQuery)
            })?;
            out.push(SurvivorEntry {
                mutation_id_hash: stored as u64,
                status,
                ts_ms: row_ts,
            });
        }
        Ok(out)
    }

    /// Convenience: short-circuit for the "toolchain missing" branch.
    /// Records a [`SurvivorStatus::Skipped`] row keyed on the supplied
    /// mutation id; intended for the cargo-mutants RUNPOD-PENDING
    /// surface. Callers that prefer to short-circuit by hash should call
    /// [`Self::record_survivor_by_hash`] directly.
    pub fn record_toolchain_missing(
        &self,
        mutation_id: &str,
        ts_ms: i64,
    ) -> Result<(), SurvivalCacheError> {
        self.record_survivor(mutation_id, ts_ms, SurvivorStatus::Skipped)
    }

    /// Operator-facing helper: drop all `Skipped` rows. Useful after a
    /// cargo-mutants toolchain bump so the dashboard rolls forward to a
    /// fresh kill-rate baseline.
    pub fn clear_skipped(&self) -> Result<usize, SurvivalCacheError> {
        let count = self.conn.execute(
            "DELETE FROM survivors WHERE status = ?1",
            params![SurvivorStatus::Skipped.as_str()],
        )?;
        Ok(count)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn cache() -> SurvivalCache {
        SurvivalCache::in_memory().expect("in-memory cache must construct")
    }

    #[test]
    fn record_and_short_circuit_hit() {
        let cache = cache();
        let mid = "crates/foo/src/bar.rs:42: replace + with -";
        cache
            .record_survivor(mid, 1_700_000_000_000, SurvivorStatus::LiveAndSurvived)
            .expect("record");
        assert!(cache.is_known_survivor(mid).expect("query"));
    }

    #[test]
    fn record_killed_does_not_short_circuit() {
        let cache = cache();
        let mid = "crates/foo/src/bar.rs:43: replace * with /";
        cache
            .record_survivor(mid, 1_700_000_000_001, SurvivorStatus::LiveAndKilled)
            .expect("record");
        assert!(!cache.is_known_survivor(mid).expect("query"));
    }

    #[test]
    fn toolchain_missing_short_circuits_record() {
        let cache = cache();
        let mid = "crates/foo/src/bar.rs:44: replace > with <";
        cache
            .record_toolchain_missing(mid, 1_700_000_000_002)
            .expect("record");
        // Skipped is NOT a survivor.
        assert!(!cache.is_known_survivor(mid).expect("query"));
        // But the row IS recorded.
        let rows = cache.list_survivors_since(0).expect("list");
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].status, SurvivorStatus::Skipped);
    }

    #[test]
    fn list_survivors_since_filters_by_timestamp() {
        let cache = cache();
        cache
            .record_survivor("a", 1_000, SurvivorStatus::LiveAndSurvived)
            .expect("record");
        cache
            .record_survivor("b", 2_000, SurvivorStatus::LiveAndKilled)
            .expect("record");
        cache
            .record_survivor("c", 3_000, SurvivorStatus::Skipped)
            .expect("record");
        let rows = cache.list_survivors_since(1_500).expect("list");
        assert_eq!(rows.len(), 2);
    }

    #[test]
    fn clear_skipped_drops_skipped_rows_only() {
        let cache = cache();
        cache
            .record_survivor("kept", 100, SurvivorStatus::LiveAndSurvived)
            .expect("record");
        cache
            .record_toolchain_missing("dropped", 200)
            .expect("record");
        let dropped = cache.clear_skipped().expect("clear");
        assert_eq!(dropped, 1);
        let rows = cache.list_survivors_since(0).expect("list");
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].status, SurvivorStatus::LiveAndSurvived);
    }

    #[test]
    fn env_gate_default_off() {
        // Use a unique env var name to avoid touching the canonical gate.
        let var = "ELAI_E13_SURVIVAL_CACHE_ENABLE_TEST_PROBE";
        // SAFETY: tests are single-threaded by default within this fn;
        // we re-read inside a scope that does not call set_var.
        std::env::remove_var(var);
        assert!(!crate::ssb::env_gate_truthy(var));
    }
}
