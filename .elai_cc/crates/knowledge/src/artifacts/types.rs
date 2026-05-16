//! Core types for the context artifacts pipeline (B-35 Phase A).
//!
//! All structs are serde-derived. blake3 hashes are stored as hex strings
//! (64 chars for full 32-byte digests, 32 chars for chunk IDs).
//! BTreeMap is used everywhere HashMap would appear — determinism requirement.

use std::collections::BTreeMap;
use std::path::PathBuf;

use serde::{Deserialize, Serialize};

// ---------------------------------------------------------------------------
// ArtifactSpec
// ---------------------------------------------------------------------------

/// One resolved entry from the [[knowledge.artifacts]] manifest.
///
/// `path` is already canonicalized and verified to be inside the project root.
/// `is_dir` is set during manifest loading by checking the filesystem;
/// actual file enumeration happens at index time.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArtifactSpec {
    pub name: String,
    pub path: PathBuf,
    pub description: String,
    pub is_dir: bool,
}

// ---------------------------------------------------------------------------
// ArtifactChunk
// ---------------------------------------------------------------------------

/// One chunk produced by the artifact indexer.
///
/// `id` is the first 32 hex chars of blake3(artifact_name:relative_path:start_line).
/// `content_hash` is the full 64-hex blake3 of the source file's raw bytes.
/// `embedding` is None when no VectorModule was available at index time.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArtifactChunk {
    /// blake3-derived chunk identifier (32 hex chars).
    pub id: String,
    pub artifact_name: String,
    pub artifact_description: String,
    /// Relative path within the artifact (file path inside a dir artifact,
    /// or the artifact's own path for single-file artifacts).
    pub relative_path: String,
    pub content: String,
    pub start_line: u32,
    pub end_line: u32,
    /// Full-file blake3 hex digest (64 chars).
    pub content_hash: String,
    /// Dense embedding vector; None when embedding was unavailable.
    pub embedding: Option<Vec<f32>>,
}

// ---------------------------------------------------------------------------
// ArtifactStatus
// ---------------------------------------------------------------------------

/// Lifecycle state of a single artifact entry.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum ArtifactStatus {
    NotIndexed,
    Indexed,
    Stale { changed_files: Vec<String> },
    Indexing,
}

impl ArtifactStatus {
    pub fn as_sql_str(&self) -> &'static str {
        match self {
            Self::NotIndexed => "not_indexed",
            Self::Indexed => "indexed",
            Self::Stale { .. } => "stale",
            Self::Indexing => "indexing",
        }
    }
}

// ---------------------------------------------------------------------------
// ArtifactIndexState
// ---------------------------------------------------------------------------

/// Persisted state for one artifact after indexing.
///
/// Stored in metadata.db `artifact_index_state` table. `file_hashes` maps
/// each relative path to its blake3 hex digest; used for staleness detection.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArtifactIndexState {
    pub name: String,
    pub last_indexed_at: i64,
    /// Maps relative file path → blake3 hex digest.
    pub file_hashes: BTreeMap<String, String>,
    pub chunk_count: usize,
    pub status: ArtifactStatus,
}

// ---------------------------------------------------------------------------
// ArtifactManifest
// ---------------------------------------------------------------------------

/// The loaded, validated manifest for a project (collection of ArtifactSpec).
#[derive(Debug, Clone, Default)]
pub struct ArtifactManifest {
    pub items: Vec<ArtifactSpec>,
}

// ---------------------------------------------------------------------------
// ArtifactSearchResult
// ---------------------------------------------------------------------------

/// One result returned by `artifact_search`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArtifactSearchResult {
    pub chunk: ArtifactChunk,
    /// Cosine similarity score in [0.0, 1.0].
    pub score: f32,
}

// ---------------------------------------------------------------------------
// ArtifactIndexAllResult
// ---------------------------------------------------------------------------

/// Summary returned by `ArtifactIndexer::index_all_artifacts`.
#[derive(Debug, Clone, Default)]
pub struct ArtifactIndexAllResult {
    pub indexed: Vec<String>,
    pub errors: Vec<(String, String)>,
}
