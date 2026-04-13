use std::io;

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
}
