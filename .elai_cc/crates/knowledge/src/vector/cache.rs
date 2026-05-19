use blake3;
use rusqlite::Connection;
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::Arc;

use crate::knowledge_ssb_sink::{
    noop_sink, KnowledgeSsbSink, VectorCacheHitRateEvent,
};

use super::DualVector;
#[cfg(test)]
use super::{CODE_VEC_DIM, TEXT_VEC_DIM};

// ---------------------------------------------------------------------------
// Schema — M3 migration (after DKG M1 + BM25 M2)
// ---------------------------------------------------------------------------

/// C-1: scale factor used to convert hits / total ratio into a
/// percentage for `VectorCacheHitRateEvent.hit_rate_pct`. Named to keep
/// magic numbers out of the code (codeRules baseline). Multiplying by
/// 100.0 yields a percentage in the closed interval [0.0, 100.0].
const HIT_RATE_PCT_SCALE: f64 = 100.0;

const SCHEMA_SQL: &str = "
DROP TABLE IF EXISTS vector_cache;
CREATE TABLE IF NOT EXISTS vector_cache (
    content_hash BLOB PRIMARY KEY,
    code_vec BLOB NOT NULL,
    text_vec BLOB NOT NULL,
    cached_at INTEGER NOT NULL
);
";

// ---------------------------------------------------------------------------
// Error type
// ---------------------------------------------------------------------------

#[derive(Debug)]
pub enum CacheError {
    Migration(rusqlite::Error),
    Query(rusqlite::Error),
    DeserError(String),
    NotSupported(String),
}

impl std::fmt::Display for CacheError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Migration(e) => write!(f, "migration error: {e}"),
            Self::Query(e) => write!(f, "query error: {e}"),
            Self::DeserError(e) => write!(f, "deserialization error: {e}"),
            Self::NotSupported(e) => write!(f, "not supported: {e}"),
        }
    }
}

impl std::error::Error for CacheError {}

// ---------------------------------------------------------------------------
// VectorCache
// ---------------------------------------------------------------------------

pub struct VectorCache {
    conn: Connection,
    /// C-1: monotonic hits counter incremented per [`Self::lookup`]
    /// returning Some. Reset by [`Self::flush_hit_rate_ssb`].
    hits: AtomicU32,
    /// C-1: monotonic misses counter incremented per [`Self::lookup`]
    /// returning None. Reset by [`Self::flush_hit_rate_ssb`].
    misses: AtomicU32,
    /// C-1: typed SSB sink for `vector_cache_hit_rate` beacons.
    /// Production wiring threads `OrchestratorKnowledgeSsbSink`;
    /// tests / inert paths use `NoopKnowledgeSsbSink`.
    ssb_sink: Arc<dyn KnowledgeSsbSink>,
}

impl VectorCache {
    pub fn new(conn: Connection) -> Result<Self, CacheError> {
        Self::with_ssb(conn, noop_sink())
    }

    /// C-1: construct a `VectorCache` with a typed knowledge SSB sink.
    /// Production callers (cli::live_cli) wire the orchestrator adapter
    /// here; tests use `noop_sink()`.
    pub fn with_ssb(
        conn: Connection,
        ssb_sink: Arc<dyn KnowledgeSsbSink>,
    ) -> Result<Self, CacheError> {
        conn.execute_batch(SCHEMA_SQL).map_err(CacheError::Migration)?;
        Ok(Self {
            conn,
            hits: AtomicU32::new(0),
            misses: AtomicU32::new(0),
            ssb_sink,
        })
    }

    pub fn content_hash(content: &[u8]) -> Vec<u8> {
        blake3::hash(content).as_bytes().to_vec()
    }

    pub fn insert(
        &self,
        hash: &[u8],
        dual: &DualVector,
        cached_at: i64,
    ) -> Result<(), CacheError> {
        let code_bytes = f32_slice_to_le_bytes(&dual.code_vec);
        let text_bytes = f32_slice_to_le_bytes(&dual.text_vec);
        self.conn
            .execute(
                "INSERT OR REPLACE INTO vector_cache (content_hash, code_vec, text_vec, cached_at) \
                 VALUES (?1, ?2, ?3, ?4)",
                rusqlite::params![hash, code_bytes, text_bytes, cached_at],
            )
            .map_err(CacheError::Query)?;
        Ok(())
    }

    pub fn lookup(&self, hash: &[u8]) -> Result<Option<DualVector>, CacheError> {
        let mut stmt = self
            .conn
            .prepare("SELECT code_vec, text_vec FROM vector_cache WHERE content_hash = ?1")
            .map_err(CacheError::Query)?;

        let mut rows = stmt
            .query(rusqlite::params![hash])
            .map_err(CacheError::Query)?;

        match rows.next().map_err(CacheError::Query)? {
            None => {
                // C-1: count miss for the periodic hit-rate flush.
                self.misses.fetch_add(1, Ordering::Relaxed);
                Ok(None)
            }
            Some(row) => {
                let code_bytes: Vec<u8> = row.get(0).map_err(CacheError::Query)?;
                let text_bytes: Vec<u8> = row.get(1).map_err(CacheError::Query)?;

                let code_vec = le_bytes_to_f32_vec(&code_bytes).map_err(|e| {
                    CacheError::DeserError(format!("code_vec: {e}"))
                })?;
                let text_vec = le_bytes_to_f32_vec(&text_bytes).map_err(|e| {
                    CacheError::DeserError(format!("text_vec: {e}"))
                })?;

                let dv = DualVector::new(code_vec, text_vec).map_err(|e| {
                    CacheError::DeserError(format!("dimension check: {e}"))
                })?;
                // C-1: count hit for the periodic hit-rate flush.
                self.hits.fetch_add(1, Ordering::Relaxed);
                Ok(Some(dv))
            }
        }
    }

    /// C-1: hit-rate accessor used by callers that want a peek without
    /// flushing (e.g. dashboard debug). Returns `(hits, misses)`. Relaxed
    /// ordering is sufficient since the counters are monotonic and
    /// readers tolerate stale-by-one snapshots.
    pub fn hit_miss_snapshot(&self) -> (u32, u32) {
        (
            self.hits.load(Ordering::Relaxed),
            self.misses.load(Ordering::Relaxed),
        )
    }

    /// C-1: emit a `VectorCacheHitRate` SSB row through the configured
    /// [`KnowledgeSsbSink`] and atomically reset the hits/misses
    /// counters. Skips emission when `hits + misses == 0` so empty
    /// flushes do not flood the dashboard.
    ///
    /// Returns `true` if a row was emitted, `false` if the call was a
    /// no-op (zero traffic since last flush).
    pub fn flush_hit_rate_ssb(&self, session_id: &str) -> bool {
        let hits = self.hits.swap(0, Ordering::Relaxed);
        let misses = self.misses.swap(0, Ordering::Relaxed);
        let total = hits.saturating_add(misses);
        if total == 0 {
            return false;
        }
        let hit_rate_pct = (f64::from(hits) / f64::from(total)) * HIT_RATE_PCT_SCALE;
        self.ssb_sink
            .emit_vector_cache_hit_rate(VectorCacheHitRateEvent {
                session_id: session_id.to_owned(),
                hits,
                misses,
                hit_rate_pct,
            });
        true
    }

    /// Invalidate cache entries for a given file path.
    /// NOTE: vector_cache schema has no file_path column. This requires either
    /// a schema extension or caller pre-computing which content_hash keys
    /// map to the file. Returning NotSupported until schema clarification.
    pub fn invalidate_file(&self, _file_path: &str) -> Result<usize, CacheError> {
        Err(CacheError::NotSupported(
            "vector_cache has no file_path column; invalidation requires schema extension or \
             caller-provided content_hash keys"
                .into(),
        ))
    }

    pub fn invalidate_keys(&self, hash_keys: &[&[u8]]) -> Result<usize, CacheError> {
        let mut deleted = 0usize;
        for key in hash_keys {
            let n = self
                .conn
                .execute(
                    "DELETE FROM vector_cache WHERE content_hash = ?1",
                    rusqlite::params![key],
                )
                .map_err(CacheError::Query)?;
            deleted += n;
        }
        Ok(deleted)
    }
}

// ---------------------------------------------------------------------------
// Serialization helpers
// ---------------------------------------------------------------------------

fn f32_slice_to_le_bytes(v: &[f32]) -> Vec<u8> {
    let mut buf = Vec::with_capacity(v.len() * 4);
    for &val in v {
        buf.extend_from_slice(&val.to_le_bytes());
    }
    buf
}

fn le_bytes_to_f32_vec(bytes: &[u8]) -> Result<Vec<f32>, String> {
    if !bytes.len().is_multiple_of(4) {
        return Err(format!(
            "byte length {} is not a multiple of 4",
            bytes.len()
        ));
    }
    let mut out = Vec::with_capacity(bytes.len() / 4);
    for chunk in bytes.chunks_exact(4) {
        out.push(f32::from_le_bytes([chunk[0], chunk[1], chunk[2], chunk[3]]));
    }
    Ok(out)
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    fn in_memory_cache() -> VectorCache {
        let conn = Connection::open_in_memory().unwrap();
        VectorCache::new(conn).unwrap()
    }

    #[test]
    fn vector_cache_migration_idempotent() {
        let conn = Connection::open_in_memory().unwrap();
        conn.execute_batch(SCHEMA_SQL).unwrap();
        conn.execute_batch(SCHEMA_SQL).unwrap();
        let _cache = VectorCache::new(conn).unwrap();
    }

    #[test]
    fn vector_cache_roundtrip() {
        let cache = in_memory_cache();
        let code = vec![1.0f32; CODE_VEC_DIM];
        let text = vec![2.0f32; TEXT_VEC_DIM];
        let dv = DualVector::new(code.clone(), text.clone()).unwrap();

        let hash = VectorCache::content_hash(b"fn main() {}");
        cache.insert(&hash, &dv, 1700000000).unwrap();

        let found = cache.lookup(&hash).unwrap();
        assert!(found.is_some());
        let found = found.unwrap();
        assert_eq!(found.code_vec.len(), CODE_VEC_DIM);
        assert_eq!(found.text_vec.len(), TEXT_VEC_DIM);
        assert_eq!(found.code_vec[0], 1.0);
        assert_eq!(found.text_vec[0], 2.0);
    }

    #[test]
    fn vector_cache_lookup_missing_returns_none() {
        let cache = in_memory_cache();
        let hash = VectorCache::content_hash(b"nonexistent");
        let result = cache.lookup(&hash).unwrap();
        assert!(result.is_none());
    }

    #[test]
    fn vector_cache_invalidate_file_returns_not_supported() {
        let cache = in_memory_cache();
        let result = cache.invalidate_file("src/main.rs");
        assert!(result.is_err());
        match result.unwrap_err() {
            CacheError::NotSupported(_) => {}
            other => panic!("expected NotSupported, got {other:?}"),
        }
    }

    #[test]
    fn vector_cache_invalidate_keys() {
        let cache = in_memory_cache();
        let code = vec![0.5f32; CODE_VEC_DIM];
        let text = vec![0.5f32; TEXT_VEC_DIM];
        let dv = DualVector::new(code, text).unwrap();

        let hash = VectorCache::content_hash(b"delete me");
        cache.insert(&hash, &dv, 1700000000).unwrap();
        assert!(cache.lookup(&hash).unwrap().is_some());

        let deleted = cache.invalidate_keys(&[&hash]).unwrap();
        assert_eq!(deleted, 1);
        assert!(cache.lookup(&hash).unwrap().is_none());
    }

    #[test]
    fn f32_roundtrip_serialization() {
        let original = vec![1.5f32, -2.3, 0.0, f32::MAX, f32::MIN];
        let bytes = f32_slice_to_le_bytes(&original);
        let recovered = le_bytes_to_f32_vec(&bytes).unwrap();
        assert_eq!(original, recovered);
    }

    #[test]
    fn le_bytes_bad_length_returns_err() {
        let result = le_bytes_to_f32_vec(&[0u8, 1, 2]);
        assert!(result.is_err());
    }

    // --- Step 3 tests ---

    #[test]
    fn test_vector_cache_insert_lookup() {
        let cache = in_memory_cache();
        let code = vec![0.42f32; CODE_VEC_DIM];
        let text = vec![0.84f32; TEXT_VEC_DIM];
        let dv = DualVector::new(code.clone(), text.clone()).unwrap();

        let content = b"pub fn example() -> bool { true }";
        let hash = VectorCache::content_hash(content);
        cache.insert(&hash, &dv, 1700000000).unwrap();

        let found = cache.lookup(&hash).unwrap().expect("should find cached entry");
        assert_eq!(found.code_vec.len(), CODE_VEC_DIM);
        assert_eq!(found.text_vec.len(), TEXT_VEC_DIM);
        assert_eq!(found.code_vec, code);
        assert_eq!(found.text_vec, text);
    }

    #[test]
    fn test_vector_cache_invalidation() {
        let cache = in_memory_cache();
        let code = vec![1.1f32; CODE_VEC_DIM];
        let text = vec![2.2f32; TEXT_VEC_DIM];
        let dv = DualVector::new(code, text).unwrap();

        let content = b"fn invalidate_me() {}";
        let hash = VectorCache::content_hash(content);
        cache.insert(&hash, &dv, 1700000000).unwrap();
        assert!(cache.lookup(&hash).unwrap().is_some());

        let deleted = cache.invalidate_keys(&[&hash]).unwrap();
        assert_eq!(deleted, 1);
        assert!(cache.lookup(&hash).unwrap().is_none());
    }
}
