use std::io;
use std::path::PathBuf;

/// Single typed error for the knowledge crate.
///
/// B-16A: replaces 14 `Box<dyn std::error::Error>` return sites across
/// `bm25.rs`, `vector/indexer.rs`, and `pagerank/mod.rs`. New variants
/// (`Tantivy`, `Json`, `SchemaMismatch`, `Index`) are added with `#[from]`
/// where the source type is unambiguous.
#[derive(Debug, thiserror::Error)]
pub enum KnowledgeError {
    #[error("I/O error: {0}")]
    Io(#[from] io::Error),

    #[error("SQLite error: {0}")]
    Sqlite(#[from] rusqlite::Error),

    #[error("query validation failed: {0}")]
    QueryValidation(#[from] crate::query_validator::QueryValidationError),

    #[error("vector error: {0}")]
    Vector(String),

    #[error("BM25 error: {0}")]
    Bm25(String),

    #[error("graph error: {0}")]
    Graph(String),

    #[error("pipeline error: {0}")]
    Pipeline(String),

    #[error("tantivy error: {0}")]
    Tantivy(#[from] tantivy::TantivyError),

    #[error("JSON serialization error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("BM25 index schema mismatch at {path}: missing field '{field}' — delete and rebuild")]
    SchemaMismatch { path: PathBuf, field: String },

    #[error("vector index error: {0}")]
    Index(String),
}

// Bridge BruteForceIndex's IndexError into KnowledgeError without forcing a
// public re-export of IndexError on KnowledgeError's surface (the From impl
// keeps `?` ergonomic at call sites).
impl From<crate::vector::hnsw_index::IndexError> for KnowledgeError {
    fn from(e: crate::vector::hnsw_index::IndexError) -> Self {
        KnowledgeError::Index(e.to_string())
    }
}

// Tantivy's QueryParserError is a separate type from TantivyError in 0.26.
impl From<tantivy::query::QueryParserError> for KnowledgeError {
    fn from(e: tantivy::query::QueryParserError) -> Self {
        KnowledgeError::Tantivy(tantivy::TantivyError::InvalidArgument(e.to_string()))
    }
}
