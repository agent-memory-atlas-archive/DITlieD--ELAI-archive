/// Errors produced by the rules-engine storage layer.
#[derive(Debug, thiserror::Error)]
pub enum RulesDbError {
    #[error("sqlite error: {0}")]
    Sqlite(#[from] rusqlite::Error),

    #[error("unexpected schema version: expected {expected}, found {found}")]
    SchemaVersion { expected: i64, found: i64 },
}
