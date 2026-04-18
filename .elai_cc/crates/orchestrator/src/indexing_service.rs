//! A-44: IndexingService — coordinates DKG + BM25 index building and
//! incremental updates for the project knowledge pipeline.

use std::path::{Path, PathBuf};

use elai_knowledge::bm25::Bm25Index;
use elai_knowledge::index_version::{self, UpgradeAction};
use elai_knowledge::{DkgEngine, KnowledgeError};

// ---------------------------------------------------------------------------
// Named constants (no magic numbers)
// ---------------------------------------------------------------------------

/// Subdirectory name under project root where indexes are stored.
pub const INDEXES_DIR_NAME: &str = "indexes";

/// Parent directory name for ELAI project data (e.g. ".elai_cc").
pub const ELAI_DATA_DIR_NAME: &str = ".elai_cc";

/// Number of bytes to read from a file's head to detect binary content.
/// Must stay in sync with elai_knowledge::bm25::BINARY_DETECT_BYTES (both 8192).
/// This is a performance pre-filter to avoid BM25 delete+re-add for obviously
/// binary files; BM25's chunk_file also detects binary internally.
pub const BINARY_DETECT_BYTES: usize = 8192;

/// DKG SQLite database filename within the indexes directory.
const DKG_DB_FILENAME: &str = "dkg.db";

/// BM25 project ID used when creating the BM25 index inside indexes_dir.
const BM25_PROJECT_ID: &str = "default";

/// Directories excluded from indexing by default.
const DEFAULT_EXCLUDED_DIRS: &[&str] = &[
    "node_modules",
    "target",
    ".git",
    "__pycache__",
    "dist",
    "build",
];

/// File patterns excluded from indexing by default.
const DEFAULT_EXCLUDED_FILES: &[&str] = &[
    ".env",
];

/// File extensions excluded from indexing by default.
const DEFAULT_EXCLUDED_EXTENSIONS: &[&str] = &[
    "key",
    "pem",
];

/// File name prefixes excluded from indexing by default.
const DEFAULT_EXCLUDED_PREFIXES: &[&str] = &[
    "credentials.",
    "secrets.",
];

/// Batch 1C (A-44): emergency shutoff for the per-project first-task
/// indexing call site. When set to a non-zero truthy value (`1` / `true` /
/// `yes` / `on`), `indexing_disabled_via_env` returns `true` and
/// `task_runner::maybe_build_project_index` short-circuits before acquiring
/// the service mutex. Mirrors the `ELAI_HOOKS_DISABLED` / `ELAI_DISABLE_
/// PROVIDER_SIGNALS` pattern from Batch 1A/1B.
pub const ELAI_DISABLE_INDEXING_ENV: &str = "ELAI_DISABLE_INDEXING";

/// Batch 1C (A-44): freshness window for the per-project indexing marker.
/// When a project's cached `indexed_at` is older than this many seconds, the
/// runner re-runs `run_full_index` (or skips into the on-disk-already path).
/// 24 hours is a conservative default that matches the "daily developer
/// cadence" assumption baked into `mvp-wiring-audit-2026-04-18.md` section
/// 3 A-13 (RAG freshness).
pub const INDEX_FRESHNESS_SECS: i64 = 86_400;

/// Returns `true` when the operator has disabled indexing via the
/// [`ELAI_DISABLE_INDEXING_ENV`] env var. Truthy values: `"1"`, `"true"`,
/// `"yes"`, `"on"` (case-insensitive).
pub fn indexing_disabled_via_env() -> bool {
    std::env::var(ELAI_DISABLE_INDEXING_ENV)
        .map(|v| {
            let v = v.trim().to_ascii_lowercase();
            matches!(v.as_str(), "1" | "true" | "yes" | "on")
        })
        .unwrap_or(false)
}

// ---------------------------------------------------------------------------
// Error type
// ---------------------------------------------------------------------------

#[derive(Debug, thiserror::Error)]
pub enum IndexingServiceError {
    #[error("knowledge error: {0}")]
    Knowledge(#[from] KnowledgeError),

    #[error("index version error: {0}")]
    IndexVersion(#[from] index_version::IndexVersionError),

    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[error("BM25 error: {0}")]
    Bm25(String),

    #[error("service not initialized — run run_full_index first")]
    NotInitialized,
}

// ---------------------------------------------------------------------------
// IndexingService
// ---------------------------------------------------------------------------

/// Coordinates DKG graph building and BM25 indexing for a project.
///
/// Created in CLI startup, lives for the duration of the session.
/// On first run (no indexes directory), builds everything from scratch.
/// On subsequent runs, checks schema version and rebuilds if needed.
/// Incremental updates are triggered by SSB file_write events.
pub struct IndexingService {
    project_root: PathBuf,
    indexes_dir: PathBuf,
    bm25: Option<Bm25Index>,
    dkg_db_path: PathBuf,
    initialized: bool,
}

impl IndexingService {
    /// Create a new IndexingService. Does NOT build indexes — call
    /// `run_full_index` to populate them.
    pub fn new(project_root: PathBuf, indexes_dir: PathBuf) -> Self {
        let dkg_db_path = indexes_dir.join(DKG_DB_FILENAME);
        Self {
            project_root,
            indexes_dir,
            bm25: None,
            dkg_db_path,
            initialized: false,
        }
    }

    /// Whether any index artifacts exist on disk.
    pub fn indexes_exist(&self) -> bool {
        self.dkg_db_path.exists()
    }

    /// Build DKG graph and BM25 index from scratch.
    ///
    /// Creates the indexes directory if it does not exist.
    /// On error, returns `Err` — the caller (CLI startup) is responsible for
    /// degrading gracefully (logging + continuing without indexes).
    pub fn run_full_index(&mut self) -> Result<IndexingSummary, IndexingServiceError> {
        std::fs::create_dir_all(&self.indexes_dir)?;

        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| i64::try_from(d.as_secs()).unwrap_or(i64::MAX))
            .unwrap_or(0);

        // Build DKG graph.
        let index_version_path = self.indexes_dir.join("index_version.toml");
        let mut dkg = DkgEngine::new(
            &self.dkg_db_path,
            Some(index_version_path.as_path()),
            now,
        )?;
        let graph = dkg.build_graph(&self.project_root)?;
        tracing::info!(
            files_parsed = graph.files_parsed,
            nodes = graph.node_count,
            edges = graph.edge_count,
            "DKG graph built"
        );
        // DkgEngine is dropped here — its Connection closes. Re-opened for queries separately.
        drop(dkg);

        // Build BM25 index.
        let mut bm25 = Bm25Index::new(&self.indexes_dir, BM25_PROJECT_ID)
            .map_err(|e| IndexingServiceError::Bm25(e.to_string()))?;
        let stats = bm25
            .index_directory(&self.project_root)
            .map_err(|e| IndexingServiceError::Bm25(e.to_string()))?;
        tracing::info!(
            files = stats.files_indexed,
            chunks = stats.chunks_indexed,
            skipped_binary = stats.files_skipped_binary,
            "BM25 index built"
        );

        self.bm25 = Some(bm25);
        self.initialized = true;

        Ok(IndexingSummary {
            dkg_files_parsed: graph.files_parsed,
            dkg_nodes: graph.node_count,
            dkg_edges: graph.edge_count,
            bm25_files: stats.files_indexed,
            bm25_chunks: stats.chunks_indexed,
            bm25_skipped_binary: stats.files_skipped_binary,
        })
    }

    /// Incrementally re-index a single changed file.
    ///
    /// Returns `Err(NotInitialized)` if `run_full_index` has not been called.
    pub fn reindex_file(&mut self, changed_path: &Path) -> Result<(), IndexingServiceError> {
        if !self.initialized {
            return Err(IndexingServiceError::NotInitialized);
        }

        // Skip binary files.
        if is_binary_file(changed_path) {
            tracing::debug!(path = %changed_path.display(), "skipping binary file in reindex");
            return Ok(());
        }

        // Skip excluded paths.
        if is_excluded_path(changed_path) {
            tracing::debug!(path = %changed_path.display(), "skipping excluded path in reindex");
            return Ok(());
        }

        // Update BM25 index for the changed file.
        if let Some(ref mut bm25) = self.bm25 {
            bm25.update_file(changed_path, None)
                .map_err(|e| IndexingServiceError::Bm25(e.to_string()))?;
        }

        Ok(())
    }

    /// Check whether indexes need rebuilding due to schema version changes.
    pub fn needs_rebuild(
        &self,
        notifier: &dyn index_version::RebuildNotifier,
    ) -> Result<UpgradeAction, IndexingServiceError> {
        if !self.indexes_dir.exists() {
            return Ok(UpgradeAction::FullRebuild);
        }
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| i64::try_from(d.as_secs()).unwrap_or(i64::MAX))
            .unwrap_or(0);
        let action = index_version::check_and_upgrade(&self.indexes_dir, now, notifier)?;
        Ok(action)
    }

    /// Take ownership of the BM25 index, consuming it from this service.
    /// Used by CLI to move BM25 into the QueryPipeline.
    pub fn take_bm25(&mut self) -> Option<Bm25Index> {
        self.bm25.take()
    }

    /// Reference to the project root.
    pub fn project_root(&self) -> &Path {
        &self.project_root
    }

    /// Reference to the indexes directory.
    pub fn indexes_dir(&self) -> &Path {
        &self.indexes_dir
    }

    /// Path to the DKG database.
    pub fn dkg_db_path(&self) -> &Path {
        &self.dkg_db_path
    }
}

// ---------------------------------------------------------------------------
// Summary
// ---------------------------------------------------------------------------

/// Summary of a full indexing run.
#[derive(Debug, Clone)]
pub struct IndexingSummary {
    pub dkg_files_parsed: usize,
    pub dkg_nodes: usize,
    pub dkg_edges: usize,
    pub bm25_files: usize,
    pub bm25_chunks: usize,
    pub bm25_skipped_binary: usize,
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Detect whether a file is binary by checking its first BINARY_DETECT_BYTES
/// bytes for null bytes.
fn is_binary_file(path: &Path) -> bool {
    let Ok(file) = std::fs::File::open(path) else {
        return false;
    };
    let mut buf = vec![0u8; BINARY_DETECT_BYTES];
    let Ok(n) = std::io::Read::read(&mut &file, &mut buf) else {
        return false;
    };
    buf[..n].contains(&0)
}

/// Check whether a path matches the default exclusion list.
fn is_excluded_path(path: &Path) -> bool {
    let path_str = path.to_string_lossy();

    // Check directory exclusions.
    for excluded in DEFAULT_EXCLUDED_DIRS {
        if path_str.contains(&format!("/{excluded}/"))
            || path_str.contains(&format!("\\{excluded}\\"))
        {
            return true;
        }
    }

    // Check file name exclusions.
    if let Some(file_name) = path.file_name().and_then(|n| n.to_str()) {
        for excluded in DEFAULT_EXCLUDED_FILES {
            if file_name == *excluded {
                return true;
            }
        }
        for prefix in DEFAULT_EXCLUDED_PREFIXES {
            if file_name.starts_with(prefix) {
                return true;
            }
        }
        if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
            for excluded_ext in DEFAULT_EXCLUDED_EXTENSIONS {
                if ext == *excluded_ext {
                    return true;
                }
            }
        }
    }

    false
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn is_binary_detects_null_bytes() {
        let tmp = tempfile::NamedTempFile::new().unwrap();
        let mut content = vec![b'a'; 100];
        content[50] = 0;
        std::fs::write(tmp.path(), &content).unwrap();
        assert!(is_binary_file(tmp.path()));
    }

    #[test]
    fn is_binary_false_for_text() {
        let tmp = tempfile::NamedTempFile::new().unwrap();
        std::fs::write(tmp.path(), "fn hello() {}").unwrap();
        assert!(!is_binary_file(tmp.path()));
    }

    #[test]
    fn excluded_path_matches_dirs() {
        assert!(is_excluded_path(Path::new("/project/node_modules/foo.js")));
        assert!(is_excluded_path(Path::new("/project/target/debug/main")));
        assert!(!is_excluded_path(Path::new("/project/src/main.rs")));
    }

    #[test]
    fn excluded_path_matches_files() {
        assert!(is_excluded_path(Path::new("/project/.env")));
        assert!(is_excluded_path(Path::new("/project/secrets.toml")));
        assert!(is_excluded_path(Path::new("/project/credentials.json")));
        assert!(is_excluded_path(Path::new("/project/server.key")));
        assert!(is_excluded_path(Path::new("/project/cert.pem")));
    }
}
