/// Errors produced by the rules-engine storage layer.
#[derive(Debug, thiserror::Error)]
pub enum RulesDbError {
    #[error("sqlite error: {0}")]
    Sqlite(#[from] rusqlite::Error),

    #[error("unexpected schema version: expected {expected}, found {found}")]
    SchemaVersion { expected: i64, found: i64 },

    #[error("interrogation tree deserialization error: {0}")]
    TreeDeserialize(String),

    #[error("interrogation tree validation error: {0}")]
    TreeValidation(String),
}

/// Errors produced by rule lifecycle transitions.
#[derive(Debug, thiserror::Error)]
pub enum LifecycleError {
    #[error("sqlite error: {0}")]
    Sqlite(#[from] rusqlite::Error),

    #[error("rule not found: {rule_id}")]
    RuleNotFound { rule_id: String },

    #[error("invalid transition from {from} to {to} for rule {rule_id}")]
    InvalidTransition {
        rule_id: String,
        from: String,
        to: String,
    },

    #[error("insufficient contract tests: have {have}, need {need}")]
    InsufficientContractTests { have: u32, need: u32 },

    #[error("contract tests are failing")]
    ContractTestsFailing,

    #[error("circular trust: finding-source auditor cannot approve its own rule")]
    CircularTrust,

    #[error("insufficient shadow data for promotion")]
    InsufficientShadowData,

    #[error("threshold not met: {metric} = {value:.4}, required {required:.4}")]
    ThresholdNotMet {
        metric: String,
        value: f64,
        required: f64,
    },

    #[error("safety regression detected during shadow period")]
    SafetyRegression,

    #[error("immutable rule cannot be auto-retired (use force=true for manual retirement)")]
    ImmutableRuleCannotAutoRetire,

    #[error("unknown lifecycle status in database: {0}")]
    UnknownStatus(String),

    #[error("unknown rule source in database: {0}")]
    UnknownSource(String),

    #[error("rule already exists: {rule_id}")]
    RuleAlreadyExists { rule_id: String },

    #[error("invalid predicate: {0}")]
    InvalidPredicate(String),

    #[error("rule refused by admission scan: {0}")]
    AdmissionRefused(String),
}
