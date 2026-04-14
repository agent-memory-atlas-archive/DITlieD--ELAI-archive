//! Freshness scoring for RAG retrieval (A-13: anti-staleness enforcement).
//!
//! Maintains a sidecar SQLite database (`freshness_meta.db`) that tracks
//! per-file modification timestamps and access counts. These are combined
//! into a freshness multiplier applied during RRF reranking in `pipeline.rs`.
//!
//! Design: docs/specs/guide/anti-staleness-enforcement.txt section 1.1
//! Audit resolutions: BLOCK-2 (access_boost cap), BLOCK-6 (separate SQLite),
//! WARN-2 (exponential decay, not quadratic).

use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};

use rusqlite::Connection;

use crate::error::KnowledgeError;

// ---------------------------------------------------------------------------
// Named constants (no magic numbers)
// ---------------------------------------------------------------------------

/// Default half-life for code files (days). At this age a document retains 50% weight.
const DEFAULT_HALF_LIFE_DAYS: f64 = 180.0;

/// Default half-life for docs/specs/changelogs (days). More aggressive decay.
const DEFAULT_DOC_HALF_LIFE_DAYS: f64 = 90.0;

/// Default normalization divisor for the logarithmic access boost.
const DEFAULT_BOOST_NORMALIZATION: f64 = 15.0;

/// Floor for recency_decay — even ancient files retain this minimum weight.
const DEFAULT_MIN_DECAY: f64 = 0.05;

/// Hard cap on access_boost to prevent frequently-used stale code from
/// dominating fresh code (BLOCK-2 resolution).
const DEFAULT_MAX_BOOST: f64 = 2.0;

/// File extensions treated as "documentation" (use doc_half_life_days).
const DOC_EXTENSIONS: &[&str] = &[
    "md", "txt", "rst", "adoc", "log", "changelog", "toml", "yaml", "yml",
    "json", "spec",
];

// ---------------------------------------------------------------------------
// FreshnessConfig
// ---------------------------------------------------------------------------

/// Configuration for freshness scoring. All five fields are spec-mandated.
#[derive(Debug, Clone)]
pub struct FreshnessConfig {
    /// Half-life in days for code files. Default: 180.
    pub half_life_days: f64,
    /// Half-life in days for docs/specs/changelogs. Default: 90.
    pub doc_half_life_days: f64,
    /// Normalization divisor for access_boost logarithm. Default: 15.
    pub boost_normalization: f64,
    /// Floor for recency_decay. Default: 0.05.
    pub min_decay: f64,
    /// Hard cap for access_boost. Default: 2.0.
    pub max_boost: f64,
}

impl Default for FreshnessConfig {
    fn default() -> Self {
        Self {
            half_life_days: DEFAULT_HALF_LIFE_DAYS,
            doc_half_life_days: DEFAULT_DOC_HALF_LIFE_DAYS,
            boost_normalization: DEFAULT_BOOST_NORMALIZATION,
            min_decay: DEFAULT_MIN_DECAY,
            max_boost: DEFAULT_MAX_BOOST,
        }
    }
}

// ---------------------------------------------------------------------------
// FreshnessRecord
// ---------------------------------------------------------------------------

/// A single row from `freshness_meta.db`.
#[derive(Debug, Clone)]
pub struct FreshnessRecord {
    pub file_path: String,
    pub last_modified_epoch: i64,
    pub access_count: i64,
}

// ---------------------------------------------------------------------------
// FreshnessDb
// ---------------------------------------------------------------------------

/// Handle to the sidecar `freshness_meta.db` SQLite database.
///
/// This is a **separate** SQLite file — not part of tantivy or vector_cache
/// schemas (BLOCK-6 resolution). Schema changes here never affect index
/// fingerprints.
pub struct FreshnessDb {
    conn: Connection,
    config: FreshnessConfig,
    /// If set, overrides `now_epoch()` for all multiplier calculations.
    /// Used in tests to control time without real clock dependency.
    now_override: Option<i64>,
}

impl FreshnessDb {
    /// Open (or create) the freshness sidecar database at the given directory.
    /// The file is always named `freshness_meta.db`.
    pub fn open(dir: &Path) -> Result<Self, KnowledgeError> {
        Self::open_with_config(dir, FreshnessConfig::default())
    }

    /// Open with explicit configuration.
    pub fn open_with_config(dir: &Path, config: FreshnessConfig) -> Result<Self, KnowledgeError> {
        let db_path = dir.join("freshness_meta.db");
        let conn = Connection::open(&db_path)?;
        conn.execute_batch(
            "PRAGMA journal_mode = WAL;
             PRAGMA synchronous = NORMAL;",
        )?;
        let db = Self { conn, config, now_override: None };
        db.ensure_schema()?;
        Ok(db)
    }

    /// Open an in-memory database (for tests).
    #[cfg(test)]
    pub fn open_in_memory(config: FreshnessConfig) -> Result<Self, KnowledgeError> {
        let conn = Connection::open_in_memory()?;
        let db = Self { conn, config, now_override: None };
        db.ensure_schema()?;
        Ok(db)
    }

    /// Set a fixed "now" epoch for deterministic testing.
    #[cfg(test)]
    pub fn set_now_override(&mut self, epoch: i64) {
        self.now_override = Some(epoch);
    }

    fn ensure_schema(&self) -> Result<(), KnowledgeError> {
        self.conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS file_freshness (
                 file_path TEXT PRIMARY KEY,
                 last_modified_epoch INTEGER NOT NULL,
                 access_count INTEGER NOT NULL DEFAULT 0
             );",
        )?;
        Ok(())
    }

    /// Upsert a file's modification timestamp (epoch seconds).
    /// Called during DKG `build_graph` and BM25 `index_file`.
    pub fn upsert_mtime(
        &self,
        file_path: &str,
        last_modified_epoch: i64,
    ) -> Result<(), KnowledgeError> {
        self.conn.execute(
            "INSERT INTO file_freshness (file_path, last_modified_epoch, access_count)
             VALUES (?1, ?2, 0)
             ON CONFLICT(file_path) DO UPDATE SET last_modified_epoch = ?2",
            rusqlite::params![file_path, last_modified_epoch],
        )?;
        Ok(())
    }

    /// Batch-upsert from filesystem mtimes. Wraps in a single transaction.
    pub fn upsert_mtimes_from_fs(
        &self,
        file_paths: &[&Path],
    ) -> Result<usize, KnowledgeError> {
        let tx = self.conn.unchecked_transaction()?;
        let mut count = 0usize;
        {
            let mut stmt = tx.prepare_cached(
                "INSERT INTO file_freshness (file_path, last_modified_epoch, access_count)
                 VALUES (?1, ?2, 0)
                 ON CONFLICT(file_path) DO UPDATE SET last_modified_epoch = ?2",
            )?;
            for path in file_paths {
                let epoch = fs_mtime_epoch(path);
                let path_str = path.to_string_lossy();
                stmt.execute(rusqlite::params![path_str.as_ref(), epoch])?;
                count += 1;
            }
        }
        tx.commit()?;
        Ok(count)
    }

    /// Increment access_count for a file (called when a retrieval result is served).
    pub fn record_access(&self, file_path: &str) -> Result<(), KnowledgeError> {
        self.conn.execute(
            "UPDATE file_freshness SET access_count = access_count + 1
             WHERE file_path = ?1",
            rusqlite::params![file_path],
        )?;
        Ok(())
    }

    /// Batch-increment access counts. Wraps in a single transaction.
    pub fn record_accesses(&self, file_paths: &[&str]) -> Result<(), KnowledgeError> {
        let tx = self.conn.unchecked_transaction()?;
        {
            let mut stmt = tx.prepare_cached(
                "UPDATE file_freshness SET access_count = access_count + 1
                 WHERE file_path = ?1",
            )?;
            for path in file_paths {
                stmt.execute(rusqlite::params![path])?;
            }
        }
        tx.commit()?;
        Ok(())
    }

    /// Look up a single file's freshness record. Returns `None` if not tracked.
    pub fn get_record(&self, file_path: &str) -> Result<Option<FreshnessRecord>, KnowledgeError> {
        let mut stmt = self.conn.prepare_cached(
            "SELECT file_path, last_modified_epoch, access_count
             FROM file_freshness WHERE file_path = ?1",
        )?;
        let mut rows = stmt.query(rusqlite::params![file_path])?;
        match rows.next()? {
            Some(row) => Ok(Some(FreshnessRecord {
                file_path: row.get(0)?,
                last_modified_epoch: row.get(1)?,
                access_count: row.get(2)?,
            })),
            None => Ok(None),
        }
    }

    /// Compute the freshness multiplier for a single chunk.
    ///
    /// `chunk_file_path`: the file path extracted from a chunk_id.
    /// `now_epoch`: current time as epoch seconds (caller provides for testability).
    ///
    /// Returns `recency_decay * access_boost`. Missing entries return 1.0
    /// (no penalty for untracked files, per spec).
    pub fn freshness_multiplier(
        &self,
        chunk_file_path: &str,
        now_epoch: i64,
    ) -> Result<f64, KnowledgeError> {
        let record = match self.get_record(chunk_file_path)? {
            Some(r) => r,
            None => return Ok(1.0),
        };
        let half_life = select_half_life(chunk_file_path, &self.config);
        let decay = recency_decay(
            now_epoch,
            record.last_modified_epoch,
            half_life,
            self.config.min_decay,
        );
        let boost = access_boost(
            record.access_count,
            self.config.boost_normalization,
            self.config.max_boost,
        );
        Ok(decay * boost)
    }

    /// Batch-compute freshness multipliers for a set of chunk file paths.
    /// Returns a Vec in the same order as input. Missing entries get 1.0.
    pub fn freshness_multipliers(
        &self,
        chunk_file_paths: &[&str],
        now_epoch: i64,
    ) -> Result<Vec<f64>, KnowledgeError> {
        let mut result = Vec::with_capacity(chunk_file_paths.len());
        for path in chunk_file_paths {
            result.push(self.freshness_multiplier(path, now_epoch)?);
        }
        Ok(result)
    }

    /// Return the effective "now" epoch (override or real clock).
    pub fn effective_now(&self) -> i64 {
        self.now_override.unwrap_or_else(now_epoch)
    }

    /// Return the current config (useful for diagnostics).
    pub fn config(&self) -> &FreshnessConfig {
        &self.config
    }

    /// Total number of tracked files.
    pub fn file_count(&self) -> Result<usize, KnowledgeError> {
        let count: i64 = self.conn.query_row(
            "SELECT COUNT(*) FROM file_freshness",
            [],
            |row| row.get(0),
        )?;
        Ok(usize::try_from(count).unwrap_or(0))
    }
}

// ---------------------------------------------------------------------------
// Pure functions (no DB, fully testable)
// ---------------------------------------------------------------------------

/// Exponential recency decay (WARN-2: NOT quadratic).
///
/// `recency_decay = max(min_decay, exp(-t * ln2 / half_life_days))`
///
/// At `t = half_life_days`, decay = 0.5 exactly.
/// At `t = 0`, decay = 1.0.
/// At `t = 2 * half_life_days`, decay = 0.25.
pub fn recency_decay(now_epoch: i64, modified_epoch: i64, half_life_days: f64, min_decay: f64) -> f64 {
    let age_secs = (now_epoch - modified_epoch).max(0) as f64;
    const SECONDS_PER_DAY: f64 = 86_400.0;
    let age_days = age_secs / SECONDS_PER_DAY;
    let raw = (-age_days * std::f64::consts::LN_2 / half_life_days).exp();
    raw.max(min_decay)
}

/// Logarithmic access boost with hard cap (BLOCK-2 resolution).
///
/// `access_boost = min(max_boost, 1.0 + ln(1.0 + access_count / boost_norm))`
///
/// At 0 accesses: 1.0.
/// At 100 accesses (norm=15): min(2.0, 1.0 + ln(7.67)) = min(2.0, 3.04) = 2.0.
pub fn access_boost(access_count: i64, boost_normalization: f64, max_boost: f64) -> f64 {
    let count = access_count.max(0) as f64;
    let raw = 1.0 + (1.0 + count / boost_normalization).ln();
    raw.min(max_boost)
}

/// Select the appropriate half-life based on file extension.
/// Docs/specs/changelogs use `doc_half_life_days`; everything else uses `half_life_days`.
pub fn select_half_life(file_path: &str, config: &FreshnessConfig) -> f64 {
    let ext = Path::new(file_path)
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("")
        .to_ascii_lowercase();
    if DOC_EXTENSIONS.contains(&ext.as_str()) {
        config.doc_half_life_days
    } else {
        config.half_life_days
    }
}

/// Extract the file path portion from a chunk_id.
///
/// chunk_id formats in pipeline.rs:
///   - `"{path}:{line_start}-{line_end}"` (BM25, vector)
///   - `"{file_path}:{symbol_name}"` (pagerank)
///
/// We split on the last `:` that precedes a digit (for line ranges) or
/// simply on the first `:` as a fallback. Either way, the file path is the
/// prefix before the first `:`.
pub fn extract_file_path(chunk_id: &str) -> &str {
    // On Windows, paths may contain drive letters like "C:". Handle by
    // skipping a leading drive-letter colon.
    let search_start = if chunk_id.len() >= 2
        && chunk_id.as_bytes()[1] == b':'
        && chunk_id.as_bytes()[0].is_ascii_alphabetic()
    {
        2
    } else {
        0
    };
    match chunk_id[search_start..].find(':') {
        Some(pos) => &chunk_id[..search_start + pos],
        None => chunk_id,
    }
}

/// Get filesystem mtime as epoch seconds. Returns 0 on error (safe default:
/// recency_decay(0) approaches min_decay, which is conservative).
fn fs_mtime_epoch(path: &Path) -> i64 {
    std::fs::metadata(path)
        .and_then(|m| m.modified())
        .ok()
        .and_then(|t| t.duration_since(UNIX_EPOCH).ok())
        .map(|d| i64::try_from(d.as_secs()).unwrap_or(0))
        .unwrap_or(0)
}

/// Current time as epoch seconds.
pub fn now_epoch() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| i64::try_from(d.as_secs()).unwrap_or(0))
        .unwrap_or(0)
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    const ONE_DAY: i64 = 86_400;

    fn default_config() -> FreshnessConfig {
        FreshnessConfig::default()
    }

    // -- Pure function tests ------------------------------------------------

    #[test]
    fn recency_decay_at_zero_age_is_one() {
        let d = recency_decay(1000, 1000, 180.0, 0.05);
        assert!((d - 1.0).abs() < 1e-9, "decay at age 0 should be 1.0, got {d}");
    }

    #[test]
    fn recency_decay_at_half_life_is_half() {
        let now = 180 * ONE_DAY;
        let d = recency_decay(now, 0, 180.0, 0.05);
        assert!((d - 0.5).abs() < 1e-6, "decay at half-life should be 0.5, got {d}");
    }

    #[test]
    fn recency_decay_at_double_half_life_is_quarter() {
        let now = 360 * ONE_DAY;
        let d = recency_decay(now, 0, 180.0, 0.05);
        assert!((d - 0.25).abs() < 1e-6, "decay at 2x half-life should be 0.25, got {d}");
    }

    #[test]
    fn recency_decay_floors_at_min_decay() {
        // 10 years old — should hit floor
        let now = 3650 * ONE_DAY;
        let d = recency_decay(now, 0, 180.0, 0.05);
        assert!((d - 0.05).abs() < 1e-6, "ancient file should floor at min_decay, got {d}");
    }

    #[test]
    fn recency_decay_future_modified_clamps_to_one() {
        // modified_epoch > now_epoch (clock skew) -> age clamped to 0 -> decay = 1.0
        let d = recency_decay(100, 200, 180.0, 0.05);
        assert!((d - 1.0).abs() < 1e-9, "future mtime should clamp to 1.0, got {d}");
    }

    #[test]
    fn recency_decay_30_days_matches_spec() {
        // Spec: at 30 days, retains 89%
        let now = 30 * ONE_DAY;
        let d = recency_decay(now, 0, 180.0, 0.05);
        assert!((d - 0.891).abs() < 0.01, "30-day decay should be ~89%, got {d}");
    }

    #[test]
    fn recency_decay_365_days_matches_spec() {
        // Spec: at 365 days, retains 25%
        let now = 365 * ONE_DAY;
        let d = recency_decay(now, 0, 180.0, 0.05);
        assert!((d - 0.25).abs() < 0.02, "365-day decay should be ~25%, got {d}");
    }

    #[test]
    fn access_boost_zero_accesses_is_one() {
        let b = access_boost(0, 15.0, 2.0);
        assert!((b - 1.0).abs() < 1e-9, "zero accesses should give boost=1.0, got {b}");
    }

    #[test]
    fn access_boost_capped_at_max() {
        // At 100 accesses: raw = 1.0 + ln(1 + 100/15) = ~3.04, capped to 2.0
        let b = access_boost(100, 15.0, 2.0);
        assert!((b - 2.0).abs() < 1e-9, "100 accesses should cap at 2.0, got {b}");
    }

    #[test]
    fn access_boost_moderate_accesses() {
        // At 15 accesses: raw = 1.0 + ln(1 + 1) = 1.0 + ln(2) = 1.693
        let b = access_boost(15, 15.0, 2.0);
        assert!((b - 1.693).abs() < 0.01, "15 accesses should give ~1.693, got {b}");
    }

    #[test]
    fn access_boost_negative_count_clamps() {
        let b = access_boost(-5, 15.0, 2.0);
        assert!((b - 1.0).abs() < 1e-9, "negative count should clamp to 0 accesses, got {b}");
    }

    #[test]
    fn select_half_life_code_file() {
        let config = default_config();
        assert!((select_half_life("src/main.rs", &config) - 180.0).abs() < 1e-9);
    }

    #[test]
    fn select_half_life_markdown_file() {
        let config = default_config();
        assert!((select_half_life("docs/CHANGELOG.md", &config) - 90.0).abs() < 1e-9);
    }

    #[test]
    fn select_half_life_txt_file() {
        let config = default_config();
        assert!((select_half_life("specs/design.txt", &config) - 90.0).abs() < 1e-9);
    }

    #[test]
    fn select_half_life_no_extension() {
        let config = default_config();
        assert!((select_half_life("Makefile", &config) - 180.0).abs() < 1e-9);
    }

    #[test]
    fn extract_file_path_bm25_format() {
        assert_eq!(extract_file_path("src/main.rs:10-20"), "src/main.rs");
    }

    #[test]
    fn extract_file_path_pagerank_format() {
        assert_eq!(extract_file_path("src/lib.rs:my_function"), "src/lib.rs");
    }

    #[test]
    fn extract_file_path_windows_drive() {
        assert_eq!(
            extract_file_path("C:\\src\\main.rs:10-20"),
            "C:\\src\\main.rs"
        );
    }

    #[test]
    fn extract_file_path_no_colon() {
        assert_eq!(extract_file_path("src/main.rs"), "src/main.rs");
    }

    // -- Database tests -----------------------------------------------------

    #[test]
    fn db_upsert_and_get_record() {
        let db = FreshnessDb::open_in_memory(default_config()).unwrap();
        db.upsert_mtime("src/main.rs", 1000).unwrap();
        let rec = db.get_record("src/main.rs").unwrap().unwrap();
        assert_eq!(rec.last_modified_epoch, 1000);
        assert_eq!(rec.access_count, 0);
    }

    #[test]
    fn db_upsert_updates_mtime() {
        let db = FreshnessDb::open_in_memory(default_config()).unwrap();
        db.upsert_mtime("src/main.rs", 1000).unwrap();
        db.upsert_mtime("src/main.rs", 2000).unwrap();
        let rec = db.get_record("src/main.rs").unwrap().unwrap();
        assert_eq!(rec.last_modified_epoch, 2000);
    }

    #[test]
    fn db_record_access_increments() {
        let db = FreshnessDb::open_in_memory(default_config()).unwrap();
        db.upsert_mtime("src/main.rs", 1000).unwrap();
        db.record_access("src/main.rs").unwrap();
        db.record_access("src/main.rs").unwrap();
        let rec = db.get_record("src/main.rs").unwrap().unwrap();
        assert_eq!(rec.access_count, 2);
    }

    #[test]
    fn db_missing_record_returns_none() {
        let db = FreshnessDb::open_in_memory(default_config()).unwrap();
        assert!(db.get_record("nonexistent.rs").unwrap().is_none());
    }

    #[test]
    fn db_freshness_multiplier_missing_file_returns_one() {
        let db = FreshnessDb::open_in_memory(default_config()).unwrap();
        let m = db.freshness_multiplier("nonexistent.rs", 1000).unwrap();
        assert!((m - 1.0).abs() < 1e-9);
    }

    #[test]
    fn db_freshness_multiplier_fresh_file() {
        let db = FreshnessDb::open_in_memory(default_config()).unwrap();
        let now = 100 * ONE_DAY;
        db.upsert_mtime("src/main.rs", now).unwrap();
        let m = db.freshness_multiplier("src/main.rs", now).unwrap();
        // age=0, decay=1.0, access=0, boost=1.0 -> multiplier=1.0
        assert!((m - 1.0).abs() < 1e-9);
    }

    #[test]
    fn db_freshness_multiplier_stale_code_with_accesses() {
        let db = FreshnessDb::open_in_memory(default_config()).unwrap();
        let modified = 0i64;
        let now = 180 * ONE_DAY; // exactly half-life
        db.upsert_mtime("src/old.rs", modified).unwrap();
        // Simulate 100 accesses
        for _ in 0..100 {
            db.record_access("src/old.rs").unwrap();
        }
        let m = db.freshness_multiplier("src/old.rs", now).unwrap();
        // decay=0.5, boost=min(2.0, 1+ln(1+100/15))=2.0 -> multiplier=1.0
        assert!((m - 1.0).abs() < 0.01, "half-life decay * capped boost should be ~1.0, got {m}");
    }

    #[test]
    fn db_freshness_multiplier_doc_decays_faster() {
        let db = FreshnessDb::open_in_memory(default_config()).unwrap();
        let modified = 0i64;
        let now = 90 * ONE_DAY; // doc half-life

        db.upsert_mtime("docs/README.md", modified).unwrap();
        db.upsert_mtime("src/main.rs", modified).unwrap();

        let doc_m = db.freshness_multiplier("docs/README.md", now).unwrap();
        let code_m = db.freshness_multiplier("src/main.rs", now).unwrap();

        // doc at 90 days with doc_half_life=90: decay=0.5, boost=1.0 -> 0.5
        assert!((doc_m - 0.5).abs() < 0.01, "doc at doc-half-life should be ~0.5, got {doc_m}");
        // code at 90 days with half_life=180: decay=0.707, boost=1.0 -> 0.707
        assert!(code_m > doc_m, "code should decay slower than docs at same age");
    }

    #[test]
    fn db_file_count() {
        let db = FreshnessDb::open_in_memory(default_config()).unwrap();
        assert_eq!(db.file_count().unwrap(), 0);
        db.upsert_mtime("a.rs", 1000).unwrap();
        db.upsert_mtime("b.rs", 2000).unwrap();
        assert_eq!(db.file_count().unwrap(), 2);
    }

    #[test]
    fn db_batch_record_accesses() {
        let db = FreshnessDb::open_in_memory(default_config()).unwrap();
        db.upsert_mtime("a.rs", 1000).unwrap();
        db.upsert_mtime("b.rs", 2000).unwrap();
        db.record_accesses(&["a.rs", "b.rs", "a.rs"]).unwrap();
        assert_eq!(db.get_record("a.rs").unwrap().unwrap().access_count, 2);
        assert_eq!(db.get_record("b.rs").unwrap().unwrap().access_count, 1);
    }

    #[test]
    fn db_freshness_multipliers_batch() {
        let db = FreshnessDb::open_in_memory(default_config()).unwrap();
        let now = 180 * ONE_DAY;
        db.upsert_mtime("a.rs", 0).unwrap();
        // a.rs: age=half_life -> decay=0.5, boost=1.0 -> 0.5
        // b.rs: missing -> 1.0
        let mults = db.freshness_multipliers(&["a.rs", "b.rs"], now).unwrap();
        assert!((mults[0] - 0.5).abs() < 0.01);
        assert!((mults[1] - 1.0).abs() < 1e-9);
    }
}
