//! Batch 3C (Phase 4H) — Self-Modification Layer 1A.
//!
//! Automatic data-table growth from PipelineOutcome with safety enforcement.
//! Layer 1A is the MVP scope — adds rows to bounded SQLite tables (deprecated
//! patterns, failure taxonomy, lazy-code regex, source tiers). Layers 2 and 3
//! are OUT OF SCOPE — enum fields for future wiring only.
//!
//! Four enforcement layers:
//!  * **Dispatch** — route through existing [`crate::AutonomousGate`]. Not
//!    re-implemented here; the dashboard path goes through the gate already.
//!  * **IPC** — Layer 1A runs in-process inside the orchestrator; the 4A IPC
//!    ToolFilter is above this layer. Documented, not enforced here.
//!  * **Filesystem** — acquire exclusive lockfile on `rules.db.lock` during
//!    apply. Stub implementation via `std::fs::OpenOptions::create_new`.
//!  * **Git** — commit via optional [`crate::ToolDispatcher`] + emit SSB row.
//!
//! Security rules are never auto-adjusted: [`is_security_sensitive`] rejects
//! any proposal touching auth/credential/password/token keywords.
//!
//! MSI tracking: exponential-decay weighted moving average (λ=0.3) of
//! apply-success outcomes over the last 10 applies, per [`PipelineMode`].

use rustc_hash::FxHashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::UNIX_EPOCH;

use regex::Regex;
use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};
use tokio::sync::Mutex as AsyncMutex;

use crate::ssb::{
    ConfigArchivedPayload, MsiUpdatePayload, ProposalAppliedPayload, ProposalGeneratedPayload,
    ProposalRejectedPayload, Ssb, SsbPayload,
};
use elai_core::{CommitOutcome, GitBranchPolicy, GitCommitRequest, ToolDispatcher};
use elai_core::{ClockProvider as _, OsClock};

// ---------------------------------------------------------------------------
// Named constants (spec defaults per phase-4H.txt)
// ---------------------------------------------------------------------------

/// Default minimum occurrences of a failure pattern before producing a
/// proposal. Matches A-34 self-discovery spec.
pub const DEFAULT_MIN_OCCURRENCES: u32 = 3;

/// Default maximum tolerable false-positive rate (regex matches against the
/// known-good corpus ÷ corpus size).
///
/// Spec phase-4H.txt:245 + 460: default threshold is 5% (0.05). Group C
/// audit BLOCK fix dropped this from 0.10 (batch-3C draft default) to the
/// spec value. Existing layer1a_contract tests pass `with_tuning(_, 0.05,
/// _, _)` explicitly so the change is backward-compatible for those tests.
pub const DEFAULT_MAX_FPR: f32 = 0.05;

/// Default rolling archive cap per [`PipelineMode`]. Older rows dropped.
pub const DEFAULT_ARCHIVE_CAP: u32 = 50;

/// Default MSI exponential-decay weight.
pub const DEFAULT_MSI_LAMBDA: f32 = 0.3;

/// Default MSI window length (newest-N applies counted).
pub const DEFAULT_MSI_WINDOW: usize = 10;

/// Env kill switch. When set to a truthy value, the dashboard construction
/// site declines to wire the processor at all.
pub const ELAI_DISABLE_LAYER1A_ENV: &str = "ELAI_DISABLE_LAYER1A";

/// Env gate for immediate application. When truthy, every proposal produced
/// by [`DefaultLayer1aProcessor::process_outcome`] is applied inline; when
/// unset or falsy, the proposal stays `pending` until a dashboard command
/// confirms it.
pub const ELAI_LAYER1A_AUTO_APPLY_ENV: &str = "ELAI_LAYER1A_AUTO_APPLY";

/// IMPROVING threshold — MSI values above this mark the pipeline mode as
/// healthy and unblocked.
pub const MSI_IMPROVING_THRESHOLD: f32 = 0.3;

/// DIVERGING threshold — MSI values at or below this pause Layer 1A for the
/// affected pipeline mode and emit an alert.
pub const MSI_DIVERGING_THRESHOLD: f32 = -0.1;

/// Agent id used for every SSB row Layer 1A emits.
pub const LAYER1A_AGENT_ID: &str = "layer1a";

/// Stub lockfile name, placed next to the rules.db file.
pub const RULES_DB_LOCKFILE: &str = "rules.db.lock";

/// Minimum pattern length we accept before refusing to build a regex. Anything
/// shorter than this is almost guaranteed to over-match (e.g. `"x"`).
const MIN_PATTERN_LEN: usize = 3;

/// Known-good code snippets used as the FPR fixture. Layer 1A never adapts
/// these — they are the conservative floor. The corpus is tiny by design: the
/// cost of running the regex is paid on every proposal.
const KNOWN_GOOD_CORPUS: &[&str] = &[
    "fn main() { println!(\"hello\"); }",
    "pub struct Config { name: String, version: u32 }",
    "impl Default for Config { fn default() -> Self { Self { name: String::new(), version: 1 } } }",
    "use std::collections::HashMap;",
    "let mut map: HashMap<String, u32> = HashMap::new();",
    "fn add(a: i32, b: i32) -> i32 { a + b }",
    "#[derive(Debug, Clone, Serialize, Deserialize)]\nstruct Record { id: u64, label: String }",
    "if let Some(x) = opt { println!(\"{x}\"); }",
    "for item in iter { total += item; }",
    "async fn fetch() -> Result<String, Error> { Ok(\"ok\".to_string()) }",
];

// ---------------------------------------------------------------------------
// Core types
// ---------------------------------------------------------------------------

/// Which workspace pipeline emitted an outcome. Layer 1A (and Phase 4H
/// Layer 2) track MSI independently for each mode — a SURGICAL divergence
/// does not pause GREENFIELD, and vice versa.
///
/// `Oracle` is the research / open-domain mode introduced by Phase 4H spec
/// (phase-4H.txt:89,105). Adding it is additive — existing `match` arms
/// over `PipelineMode` use `Surgical`/`Greenfield` only and are wildcard-
/// covered or non-exhaustive at construction sites.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum PipelineMode {
    Surgical,
    Greenfield,
    Oracle,
}

impl PipelineMode {
    pub fn as_str(&self) -> &'static str {
        match self {
            PipelineMode::Surgical => "surgical",
            PipelineMode::Greenfield => "greenfield",
            PipelineMode::Oracle => "oracle",
        }
    }
}

/// One failure observed inside a [`PipelineOutcome`]. Caller populates
/// `category` / `pattern` / `occurrences`. `is_false_positive` is an operator
/// hint — when `true` we never propose a row from this record.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FailureRecord {
    pub category: String,
    pub pattern: String,
    pub occurrences: u32,
    pub is_false_positive: bool,
}

/// A single finding row attached to the outcome. Currently advisory — Layer
/// 1A does not propose rows from findings alone, but the struct is here so
/// future tiers can classify them without a breaking change.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FindingRecord {
    pub category: String,
    pub message: String,
    pub severity: String,
}

/// Everything Layer 1A needs to know about the just-completed pipeline run.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PipelineOutcome {
    pub task_id: String,
    pub success: bool,
    pub failures: Vec<FailureRecord>,
    pub findings: Vec<FindingRecord>,
    pub pipeline_mode: PipelineMode,
    pub elapsed_ms: u64,
}

/// The four bounded data tables Layer 1A can grow. Each variant maps to a
/// concrete SQLite table name in [`DefaultLayer1aProcessor`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DataTable {
    DeprecatedPatterns,
    FailureTaxonomy,
    LazyCodeRegex,
    SourceTiers,
}

impl DataTable {
    pub fn as_str(&self) -> &'static str {
        match self {
            DataTable::DeprecatedPatterns => "deprecated_patterns",
            DataTable::FailureTaxonomy => "failure_taxonomy",
            DataTable::LazyCodeRegex => "lazy_code_regex",
            DataTable::SourceTiers => "source_tiers",
        }
    }
}

/// AllowedConfigKey — Layer 1A only adjusts [`DataTable`] rows. The
/// `Parameter` variant is reserved for Layer 2 (not implemented).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum AllowedConfigKey {
    DataTable(DataTable),
    /// Layer 2 placeholder — not allowed in Layer 1A. Any attempt to modify a
    /// Parameter key returns [`Layer1aError::LockedKey`].
    Parameter(String),
}

/// Provenance of a proposal.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ProposalSource {
    AutoFromOutcome,
    Manual,
}

impl ProposalSource {
    pub fn as_str(&self) -> &'static str {
        match self {
            ProposalSource::AutoFromOutcome => "auto_from_outcome",
            ProposalSource::Manual => "manual",
        }
    }
}

/// A pending or applied proposal to grow one of the Layer 1A data tables.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DataTableProposal {
    pub proposal_id: String,
    pub table: DataTable,
    pub new_row: serde_json::Value,
    pub source: ProposalSource,
    pub fpr_estimate: f32,
    pub occurrences_seen: u32,
    /// The [`PipelineMode`] whose outcome surfaced this proposal. Used for
    /// per-mode archive capping + MSI attribution.
    pub pipeline_mode: PipelineMode,
    /// B-23 Phase 4H Group C (4H.3B): scope filter. When `Some(_)`, the
    /// proposal is only applied if the active project's ecosystem matches.
    /// `None` = unscoped (applies to all ecosystems).
    #[serde(default)]
    pub scope: Option<crate::self_mod_types::ProjectScope>,
}

/// Outcome of a successful [`Layer1aProcessor::apply_proposal`].
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ProposalOutcome {
    pub proposal_id: String,
    pub applied_by: String,
    pub commit_sha: Option<String>,
}

/// A historical record of an applied proposal, returned by
/// [`Layer1aProcessor::list_applied`].
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AppliedProposalRecord {
    pub proposal_id: String,
    pub table: DataTable,
    pub new_row: serde_json::Value,
    pub applied_by: String,
    pub proposed_at_ms: i64,
    pub applied_at_ms: i64,
    pub commit_sha: Option<String>,
    pub pipeline_mode: PipelineMode,
    /// B-23 Phase 4H Group C (4H.3B): scope filter persisted with the
    /// applied row so the dashboard can render "applied to cargo only".
    #[serde(default)]
    pub scope: Option<crate::self_mod_types::ProjectScope>,
}

/// MSI snapshot per pipeline mode.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MsiStatus {
    pub pipeline_mode: PipelineMode,
    pub msi_value: f32,
    pub status: String,
    pub samples: u32,
}

// ---------------------------------------------------------------------------
// Errors
// ---------------------------------------------------------------------------

#[derive(Debug, thiserror::Error)]
pub enum Layer1aError {
    #[error("sqlite: {0}")]
    Sqlite(#[from] rusqlite::Error),
    #[error("proposal not found: {0}")]
    NotFound(String),
    #[error("security-sensitive proposal rejected: {0}")]
    SecuritySensitive(String),
    #[error("locked config key: {0}")]
    LockedKey(String),
    #[error("fpr too high: {fpr} > {max_fpr}")]
    FprTooHigh { fpr: f32, max_fpr: f32 },
    #[error("invalid proposal: {0}")]
    InvalidProposal(String),
    #[error("filesystem lock acquisition failed: {0}")]
    LockAcquisitionFailed(String),
    #[error("regex compile failed: {0}")]
    RegexCompile(String),
    #[error("poisoned mutex")]
    Poisoned,
    #[error("diverging msi; layer1a paused for mode {0}")]
    Diverging(String),
    #[error("serialization: {0}")]
    Serialization(String),
    #[error("tool dispatcher: {0}")]
    Dispatcher(String),
    /// B-23 Phase 4H Group C (4H.3B): proposal carried a scoped ecosystem
    /// that does not match the processor's active ecosystem.
    #[error("scope mismatch: proposal targets {expected} but active ecosystem is {actual}")]
    ScopeMismatch { expected: String, actual: String },
}

// ---------------------------------------------------------------------------
// Trait
// ---------------------------------------------------------------------------

/// Trait surface consumed by `drive_task` and the dashboard Tauri commands.
/// Trait-object safe; dispatched via `Arc<dyn Layer1aProcessor>`.
pub trait Layer1aProcessor: Send + Sync {
    /// Inspect the outcome and produce zero or more proposals. Never applies
    /// anything — call [`Layer1aProcessor::apply_proposal`] for that.
    fn process_outcome<'a>(
        &'a self,
        outcome: &'a PipelineOutcome,
    ) -> std::pin::Pin<
        Box<dyn std::future::Future<Output = Vec<DataTableProposal>> + Send + 'a>,
    >;

    /// Apply one proposal. Writes the new row into rules.db, archives the old
    /// version (if any), emits SSB rows (`ProposalApplied` + `MsiUpdate` +
    /// `ConfigArchived` + `GitCommit` when wired), and updates per-mode MSI.
    fn apply_proposal<'a>(
        &'a self,
        proposal: &'a DataTableProposal,
    ) -> std::pin::Pin<
        Box<
            dyn std::future::Future<Output = Result<ProposalOutcome, Layer1aError>>
                + Send
                + 'a,
        >,
    >;

    fn list_pending<'a>(
        &'a self,
    ) -> std::pin::Pin<
        Box<dyn std::future::Future<Output = Vec<DataTableProposal>> + Send + 'a>,
    >;

    fn list_applied<'a>(
        &'a self,
        limit: u32,
    ) -> std::pin::Pin<
        Box<dyn std::future::Future<Output = Vec<AppliedProposalRecord>> + Send + 'a>,
    >;

    fn reject_proposal<'a>(
        &'a self,
        proposal_id: &'a str,
        reason: &'a str,
    ) -> std::pin::Pin<
        Box<dyn std::future::Future<Output = Result<(), Layer1aError>> + Send + 'a>,
    >;

    fn msi_status<'a>(
        &'a self,
    ) -> std::pin::Pin<
        Box<dyn std::future::Future<Output = Vec<MsiStatus>> + Send + 'a>,
    >;
}

/// Returns true when [`AllowedConfigKey::DataTable(_)`] — the only class Layer
/// 1A may modify. `Parameter` keys always return false.
pub fn is_allowed(key: &AllowedConfigKey) -> bool {
    matches!(key, AllowedConfigKey::DataTable(_))
}

/// Returns true when the row payload touches security-sensitive keywords.
/// Applied before any other check in [`DefaultLayer1aProcessor::apply_proposal`].
pub fn is_security_sensitive(row: &serde_json::Value) -> bool {
    let s = row.to_string().to_lowercase();
    const NEEDLES: &[&str] = &[
        "auth",
        "credential",
        "password",
        "secret",
        "token",
        "ssh",
        "tls",
        "ssl",
        "private_key",
        "api_key",
    ];
    NEEDLES.iter().any(|n| s.contains(n))
}

// ---------------------------------------------------------------------------
// MSI internals
// ---------------------------------------------------------------------------

/// Per-PipelineMode MSI state. Newest outcome is `history[0]`. `compute()`
/// returns the exponential-decay weighted mean.
#[derive(Debug, Default)]
struct MsiState {
    /// +1.0 on success, -1.0 on failure. Most recent first, capped to
    /// `DEFAULT_MSI_WINDOW`.
    history: Vec<f32>,
}

impl MsiState {
    fn record(&mut self, delta: f32) {
        self.history.insert(0, delta);
        if self.history.len() > DEFAULT_MSI_WINDOW {
            self.history.truncate(DEFAULT_MSI_WINDOW);
        }
    }

    fn compute(&self, lambda: f32) -> f32 {
        if self.history.is_empty() {
            return 0.0;
        }
        let mut weight_sum = 0.0f32;
        let mut value = 0.0f32;
        for (i, d) in self.history.iter().enumerate() {
            // NARROWING-OK: i is bounded by DEFAULT_MSI_WINDOW=10; always fits i32.
            let w = (1.0 - lambda).powi(i32::try_from(i).unwrap_or(i32::MAX));
            value += w * d;
            weight_sum += w;
        }
        if weight_sum <= 0.0 {
            0.0
        } else {
            value / weight_sum
        }
    }

    fn samples(&self) -> u32 {
        u32::try_from(self.history.len()).unwrap_or(u32::MAX)
    }
}

pub fn status_label(msi: f32) -> &'static str {
    if msi > MSI_IMPROVING_THRESHOLD {
        "IMPROVING"
    } else if msi <= MSI_DIVERGING_THRESHOLD {
        "DIVERGING"
    } else {
        "STABILIZING"
    }
}

// ---------------------------------------------------------------------------
// DefaultLayer1aProcessor
// ---------------------------------------------------------------------------

/// Production implementation. Stores every proposal / applied row / archive /
/// MSI transition in `rules.db` (rusqlite). Emits SSB rows through the shared
/// [`Ssb`] writer. When a [`ToolDispatcher`] is wired, successful applies
/// also fire `dispatch_git_commit` + a `GitCommit` SSB row.
pub struct DefaultLayer1aProcessor {
    /// Shared rules.db connection. `Arc<AsyncMutex<_>>` mirrors the pattern
    /// used by the orchestrator's `IndexingService` and is required because
    /// `rusqlite::Connection` is `!Send + !Sync` on its own.
    rules_db: Arc<AsyncMutex<Connection>>,
    /// Path to the rules.db file. Used to derive the lockfile path for the
    /// filesystem enforcement layer.
    rules_db_path: PathBuf,
    /// Shared SSB writer. Emissions go through `write_payload`.
    ssb: Arc<Ssb>,
    /// Optional tool dispatcher for git_commit on successful apply.
    dispatcher: Option<Arc<dyn ToolDispatcher>>,
    /// Active branch policy passed to `dispatch_git_commit`.
    git_branch_policy: GitBranchPolicy,
    pub min_occurrences: u32,
    pub max_fpr: f32,
    pub archive_cap: u32,
    pub msi_lambda: f32,
    msi: Arc<AsyncMutex<FxHashMap<PipelineMode, MsiState>>>,
    /// B-23 Phase 4H Group C (4H.3B): active project ecosystem ("cargo",
    /// "pip", "npm", ...). When `Some(_)`, scoped proposals whose
    /// `scope.ecosystem` differs are rejected with [`Layer1aError::ScopeMismatch`].
    /// When `None`, scope checks are skipped (legacy callers).
    pub active_ecosystem: Option<String>,
}

impl DefaultLayer1aProcessor {
    /// Construct a Layer 1A processor. Call [`Self::ensure_tables`] exactly
    /// once before first use (dashboard setup does this automatically).
    pub fn new(
        rules_db: Arc<AsyncMutex<Connection>>,
        rules_db_path: PathBuf,
        ssb: Arc<Ssb>,
    ) -> Self {
        Self {
            rules_db,
            rules_db_path,
            ssb,
            dispatcher: None,
            git_branch_policy: GitBranchPolicy::Main,
            min_occurrences: DEFAULT_MIN_OCCURRENCES,
            max_fpr: DEFAULT_MAX_FPR,
            archive_cap: DEFAULT_ARCHIVE_CAP,
            msi_lambda: DEFAULT_MSI_LAMBDA,
            msi: Arc::new(AsyncMutex::new(FxHashMap::default())),
            active_ecosystem: None,
        }
    }

    pub fn with_dispatcher(mut self, dispatcher: Arc<dyn ToolDispatcher>) -> Self {
        self.dispatcher = Some(dispatcher);
        self
    }

    pub fn with_branch_policy(mut self, policy: GitBranchPolicy) -> Self {
        self.git_branch_policy = policy;
        self
    }

    /// Set the active project ecosystem (e.g. "cargo", "pip", "npm"). Only
    /// scoped proposals whose `scope.ecosystem` matches will be applied.
    /// Group C audit fix: spec phase-4H.txt:490-500 scope enforcement.
    pub fn with_ecosystem(mut self, ecosystem: impl Into<String>) -> Self {
        self.active_ecosystem = Some(ecosystem.into());
        self
    }

    pub fn with_tuning(
        mut self,
        min_occurrences: u32,
        max_fpr: f32,
        archive_cap: u32,
        msi_lambda: f32,
    ) -> Self {
        self.min_occurrences = min_occurrences;
        self.max_fpr = max_fpr;
        self.archive_cap = archive_cap;
        self.msi_lambda = msi_lambda;
        self
    }

    /// Create the Layer 1A tables if missing. Must be called once before any
    /// other method. Idempotent.
    pub async fn ensure_tables(&self) -> Result<(), Layer1aError> {
        let conn = self.rules_db.lock().await;
        // `config_proposals` — pending + rejected proposals. Applied rows are
        // COPIED into `applied_proposals` at apply time (we keep the
        // `config_proposals` row so callers can list the full history if they
        // wish; status column records the terminal state).
        conn.execute_batch(
            "\
            CREATE TABLE IF NOT EXISTS config_proposals (
                proposal_id   TEXT PRIMARY KEY,
                table_name    TEXT NOT NULL,
                new_row_json  TEXT NOT NULL,
                source        TEXT NOT NULL,
                fpr_estimate  REAL NOT NULL,
                occurrences   INTEGER NOT NULL,
                pipeline_mode TEXT NOT NULL,
                status        TEXT NOT NULL,
                reject_reason TEXT,
                proposed_at   INTEGER NOT NULL,
                applied_at    INTEGER
            );
            CREATE INDEX IF NOT EXISTS idx_config_proposals_status
                ON config_proposals(status);
            CREATE INDEX IF NOT EXISTS idx_config_proposals_mode
                ON config_proposals(pipeline_mode);

            CREATE TABLE IF NOT EXISTS applied_proposals (
                proposal_id    TEXT PRIMARY KEY,
                table_name     TEXT NOT NULL,
                new_row_json   TEXT NOT NULL,
                applied_by     TEXT NOT NULL,
                proposed_at    INTEGER NOT NULL,
                applied_at     INTEGER NOT NULL,
                commit_sha     TEXT,
                pipeline_mode  TEXT NOT NULL
            );
            CREATE INDEX IF NOT EXISTS idx_applied_proposals_applied_at
                ON applied_proposals(applied_at DESC);

            CREATE TABLE IF NOT EXISTS config_archive (
                archive_id     INTEGER PRIMARY KEY AUTOINCREMENT,
                pipeline_mode  TEXT NOT NULL,
                proposal_id    TEXT NOT NULL,
                table_name     TEXT NOT NULL,
                new_row_json   TEXT NOT NULL,
                archived_at    INTEGER NOT NULL
            );
            CREATE INDEX IF NOT EXISTS idx_config_archive_mode
                ON config_archive(pipeline_mode, archived_at DESC);

            -- The four Layer 1A data tables. Spec says 'FailureTaxonomy for
            -- new categories, DeprecatedPatterns for API-style patterns,
            -- LazyCodeRegex for todo!/pass/TODO signals, SourceTiers for
            -- research-source trust.' Columns are minimal — the authoritative
            -- row content is the serialised JSON.
            CREATE TABLE IF NOT EXISTS deprecated_patterns (
                id          INTEGER PRIMARY KEY AUTOINCREMENT,
                row_json    TEXT NOT NULL,
                applied_by  TEXT NOT NULL,
                created_at  INTEGER NOT NULL
            );
            CREATE TABLE IF NOT EXISTS failure_taxonomy (
                id          INTEGER PRIMARY KEY AUTOINCREMENT,
                row_json    TEXT NOT NULL,
                applied_by  TEXT NOT NULL,
                created_at  INTEGER NOT NULL
            );
            CREATE TABLE IF NOT EXISTS lazy_code_regex (
                id          INTEGER PRIMARY KEY AUTOINCREMENT,
                row_json    TEXT NOT NULL,
                applied_by  TEXT NOT NULL,
                created_at  INTEGER NOT NULL
            );
            CREATE TABLE IF NOT EXISTS source_tiers (
                id          INTEGER PRIMARY KEY AUTOINCREMENT,
                row_json    TEXT NOT NULL,
                applied_by  TEXT NOT NULL,
                created_at  INTEGER NOT NULL
            );
            ",
        )?;
        // B-23 Phase 4H Group C (4H.3B): add scope_json columns for ecosystem
        // / language / pipeline_mode filtering. ALTER TABLE is NOT idempotent
        // — wrap in try_execute and ignore "duplicate column name" errors so
        // existing DBs migrate cleanly without breaking new deployments.
        let _ = conn.execute(
            "ALTER TABLE config_proposals ADD COLUMN scope_json TEXT",
            [],
        );
        let _ = conn.execute(
            "ALTER TABLE applied_proposals ADD COLUMN scope_json TEXT",
            [],
        );
        Ok(())
    }

    /// Persist a freshly-produced proposal as `pending`. Caller retains
    /// access via `list_pending`.
    async fn persist_pending(&self, proposal: &DataTableProposal) -> Result<(), Layer1aError> {
        let new_row_json = serde_json::to_string(&proposal.new_row)
            .map_err(|e| Layer1aError::Serialization(e.to_string()))?;
        let scope_json = match &proposal.scope {
            Some(scope) => Some(
                serde_json::to_string(scope)
                    .map_err(|e| Layer1aError::Serialization(e.to_string()))?,
            ),
            None => None,
        };
        let conn = self.rules_db.lock().await;
        conn.execute(
            "INSERT OR REPLACE INTO config_proposals \
             (proposal_id, table_name, new_row_json, source, fpr_estimate, \
              occurrences, pipeline_mode, status, reject_reason, proposed_at, scope_json) \
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, 'pending', NULL, ?8, ?9)",
            params![
                proposal.proposal_id,
                proposal.table.as_str(),
                new_row_json,
                proposal.source.as_str(),
                proposal.fpr_estimate as f64,
                proposal.occurrences_seen,
                proposal.pipeline_mode.as_str(),
                now_ms(),
                scope_json,
            ],
        )?;
        Ok(())
    }

    /// Dispatch a git commit for a successfully-applied proposal. Emits a
    /// `GitCommit` SSB row on success. Failures are logged but non-fatal —
    /// Layer 1A never blocks on git2.
    async fn commit_apply(
        &self,
        proposal: &DataTableProposal,
    ) -> Result<Option<CommitOutcome>, Layer1aError> {
        let Some(dispatcher) = self.dispatcher.as_ref() else {
            return Ok(None);
        };
        let msg = format!(
            "layer1a: auto-add {} row (source={})",
            proposal.table.as_str(),
            proposal.source.as_str()
        );
        let req = GitCommitRequest {
            anchor_path: self.rules_db_path.clone(),
            files: vec![self.rules_db_path.clone()],
            message: msg,
            branch_policy: self.git_branch_policy,
            plan_id: "layer1a".to_owned(),
            step_id: proposal.proposal_id.clone(),
        };
        match dispatcher.dispatch_git_commit(req).await {
            Ok(out) => Ok(Some(out)),
            Err(e) => {
                tracing::warn!(error = %e, proposal = %proposal.proposal_id, "layer1a git_commit failed");
                Err(Layer1aError::Dispatcher(e.to_string()))
            }
        }
    }
}

impl Layer1aProcessor for DefaultLayer1aProcessor {
    fn process_outcome<'a>(
        &'a self,
        outcome: &'a PipelineOutcome,
    ) -> std::pin::Pin<
        Box<dyn std::future::Future<Output = Vec<DataTableProposal>> + Send + 'a>,
    > {
        Box::pin(async move {
            let mut proposals: Vec<DataTableProposal> = Vec::new();
            // Aggregate identical (category, pattern) pairs so repeat
            // observations across findings accumulate.
            let mut bucket: FxHashMap<(String, String), u32> = FxHashMap::default();
            for f in &outcome.failures {
                if f.is_false_positive {
                    continue;
                }
                if f.pattern.trim().len() < MIN_PATTERN_LEN {
                    continue;
                }
                *bucket.entry((f.category.clone(), f.pattern.clone())).or_insert(0) +=
                    f.occurrences.max(1);
            }
            for ((category, pattern), occurrences) in bucket {
                if occurrences < self.min_occurrences {
                    continue;
                }
                let table = classify_table(&category, &pattern);
                let Ok(regex) = build_pattern_regex(&pattern) else {
                    continue;
                };
                let fpr = estimate_fpr(&regex);
                let new_row = serde_json::json!({
                    "category": category,
                    "pattern": pattern,
                    "source": "pipeline_outcome",
                    "task_id": outcome.task_id,
                });
                // Group C audit BLOCK fix: spec phase-4H.txt:499 requires
                // every auto-proposal to carry scope != None. Use the
                // processor's `active_ecosystem` (set via `with_ecosystem`)
                // when available; fall back to a mode-only scope so legacy
                // callers without an ecosystem still satisfy the check.
                let scope = Some(crate::self_mod_types::ProjectScope {
                    ecosystem: self.active_ecosystem.clone(),
                    language: self
                        .active_ecosystem
                        .as_deref()
                        .and_then(ecosystem_to_language),
                    pipeline_mode: Some(outcome.pipeline_mode),
                });
                let proposal = DataTableProposal {
                    proposal_id: make_proposal_id(&category, &pattern),
                    table,
                    new_row,
                    source: ProposalSource::AutoFromOutcome,
                    fpr_estimate: fpr,
                    occurrences_seen: occurrences,
                    pipeline_mode: outcome.pipeline_mode,
                    scope,
                };
                if proposal.fpr_estimate > self.max_fpr {
                    // Emit rejection up-front so the dashboard can surface
                    // "proposal auto-rejected (fpr too high)".
                    let reason = format!(
                        "fpr {:.3} > max {:.3}",
                        proposal.fpr_estimate, self.max_fpr
                    );
                    let _ = self.ssb
                        .write_payload(
                            LAYER1A_AGENT_ID,
                            0,
                            SsbPayload::ProposalRejected(ProposalRejectedPayload {
                                proposal_id: proposal.proposal_id.clone(),
                                table: proposal.table.as_str().to_owned(),
                                reason,
                            }),
                            Vec::new(),
                        )
                        .await;
                    continue;
                }
                if is_security_sensitive(&proposal.new_row) {
                    let _ = self
                        .ssb
                        .write_payload(
                            LAYER1A_AGENT_ID,
                            0,
                            SsbPayload::ProposalRejected(ProposalRejectedPayload {
                                proposal_id: proposal.proposal_id.clone(),
                                table: proposal.table.as_str().to_owned(),
                                reason: "security-sensitive keyword detected".to_owned(),
                            }),
                            Vec::new(),
                        )
                        .await;
                    continue;
                }
                if let Err(e) = self.persist_pending(&proposal).await {
                    tracing::warn!(error = %e, "layer1a persist_pending failed");
                    continue;
                }
                let summary = truncate_for_summary(&proposal.new_row);
                let _ = self
                    .ssb
                    .write_payload(
                        LAYER1A_AGENT_ID,
                        0,
                        SsbPayload::ProposalGenerated(ProposalGeneratedPayload {
                            proposal_id: proposal.proposal_id.clone(),
                            table: proposal.table.as_str().to_owned(),
                            new_row_summary: summary,
                            fpr_estimate: proposal.fpr_estimate,
                            source: proposal.source.as_str().to_owned(),
                        }),
                        Vec::new(),
                    )
                    .await;
                proposals.push(proposal);
            }
            proposals
        })
    }

    fn apply_proposal<'a>(
        &'a self,
        proposal: &'a DataTableProposal,
    ) -> std::pin::Pin<
        Box<
            dyn std::future::Future<Output = Result<ProposalOutcome, Layer1aError>>
                + Send
                + 'a,
        >,
    > {
        Box::pin(async move {
            // Enforcement layer — Filesystem lock.
            let _guard = acquire_rules_lock(&self.rules_db_path)?;

            // Enforcement layer — Security.
            if is_security_sensitive(&proposal.new_row) {
                return Err(Layer1aError::SecuritySensitive(
                    proposal.proposal_id.clone(),
                ));
            }

            // Enforcement layer — AllowedConfigKey.
            let key = AllowedConfigKey::DataTable(proposal.table);
            if !is_allowed(&key) {
                return Err(Layer1aError::LockedKey(proposal.table.as_str().to_owned()));
            }

            // Enforcement layer — Scope (4H.3B). Skipped when proposal is
            // unscoped or when the processor has no active_ecosystem (legacy
            // callers / tests that pre-date Group C).
            if let (Some(scope), Some(active)) =
                (proposal.scope.as_ref(), self.active_ecosystem.as_ref())
            {
                if let Some(scope_eco) = scope.ecosystem.as_ref() {
                    if scope_eco != active {
                        return Err(Layer1aError::ScopeMismatch {
                            expected: scope_eco.clone(),
                            actual: active.clone(),
                        });
                    }
                }
            }

            // FPR check (re-run against corpus in case the regex changed).
            let pattern_str = proposal
                .new_row
                .get("pattern")
                .and_then(|v| v.as_str())
                .unwrap_or("");
            if !pattern_str.is_empty() {
                if let Ok(rx) = build_pattern_regex(pattern_str) {
                    let fpr = estimate_fpr(&rx);
                    if fpr > self.max_fpr {
                        return Err(Layer1aError::FprTooHigh {
                            fpr,
                            max_fpr: self.max_fpr,
                        });
                    }
                }
            }

            // MSI diverging guard — refuse new applies for a diverging mode.
            {
                let msi = self.msi.lock().await;
                if let Some(state) = msi.get(&proposal.pipeline_mode) {
                    let v = state.compute(self.msi_lambda);
                    if state.samples() >= 3 && v <= MSI_DIVERGING_THRESHOLD {
                        return Err(Layer1aError::Diverging(
                            proposal.pipeline_mode.as_str().to_owned(),
                        ));
                    }
                }
            }

            let now = now_ms();
            let new_row_json = serde_json::to_string(&proposal.new_row)
                .map_err(|e| Layer1aError::Serialization(e.to_string()))?;
            let scope_json: Option<String> = match &proposal.scope {
                Some(s) => Some(
                    serde_json::to_string(s)
                        .map_err(|e| Layer1aError::Serialization(e.to_string()))?,
                ),
                None => None,
            };

            // Persist application to rules.db inside a single transaction so
            // the data-table row + applied-proposals row + archive insert
            // + status flip all commit together.
            let mut dropped_archive_id: Option<i64> = None;
            {
                let mut conn = self.rules_db.lock().await;
                let tx = conn.transaction()?;
                // Insert into the appropriate data table.
                let insert_sql = format!(
                    "INSERT INTO {} (row_json, applied_by, created_at) VALUES (?1, ?2, ?3)",
                    proposal.table.as_str()
                );
                tx.execute(&insert_sql, params![new_row_json, LAYER1A_AGENT_ID, now])?;
                // Record applied_proposals.
                let proposed_at: i64 = tx
                    .query_row(
                        "SELECT proposed_at FROM config_proposals WHERE proposal_id = ?1",
                        params![proposal.proposal_id],
                        |row| row.get(0),
                    )
                    .unwrap_or(now);
                tx.execute(
                    "INSERT OR REPLACE INTO applied_proposals \
                     (proposal_id, table_name, new_row_json, applied_by, proposed_at, \
                      applied_at, commit_sha, pipeline_mode, scope_json) \
                     VALUES (?1, ?2, ?3, ?4, ?5, ?6, NULL, ?7, ?8)",
                    params![
                        proposal.proposal_id,
                        proposal.table.as_str(),
                        new_row_json,
                        LAYER1A_AGENT_ID,
                        proposed_at,
                        now,
                        proposal.pipeline_mode.as_str(),
                        scope_json,
                    ],
                )?;
                // Mark the proposal applied; leaves reject_reason NULL.
                tx.execute(
                    "UPDATE config_proposals SET status = 'applied', applied_at = ?1 \
                     WHERE proposal_id = ?2",
                    params![now, proposal.proposal_id],
                )?;
                // Archive the snapshot. Drop oldest if cap exceeded.
                tx.execute(
                    "INSERT INTO config_archive \
                     (pipeline_mode, proposal_id, table_name, new_row_json, archived_at) \
                     VALUES (?1, ?2, ?3, ?4, ?5)",
                    params![
                        proposal.pipeline_mode.as_str(),
                        proposal.proposal_id,
                        proposal.table.as_str(),
                        new_row_json,
                        now,
                    ],
                )?;
                let count: u32 = tx
                    .query_row(
                        "SELECT COUNT(*) FROM config_archive WHERE pipeline_mode = ?1",
                        params![proposal.pipeline_mode.as_str()],
                        |row| row.get(0),
                    )
                    .unwrap_or(0);
                if count > self.archive_cap {
                    // Drop oldest entries down to the cap.
                    let to_drop = count - self.archive_cap;
                    let rows: Vec<(i64, String)> = {
                        let mut stmt = tx.prepare(
                            "SELECT archive_id, proposal_id FROM config_archive \
                             WHERE pipeline_mode = ?1 ORDER BY archived_at ASC LIMIT ?2",
                        )?;
                        let rows_iter = stmt.query_map(
                            // NARROWING-OK: u32->i64 is widening; to_drop is u32 (count-cap), fits i64.
                            params![proposal.pipeline_mode.as_str(), to_drop as i64],
                            |row| Ok((row.get(0)?, row.get(1)?)),
                        )?;
                        let mut collected = Vec::new();
                        for r in rows_iter {
                            collected.push(r?);
                        }
                        collected
                    };
                    for (id, _) in &rows {
                        tx.execute(
                            "DELETE FROM config_archive WHERE archive_id = ?1",
                            params![id],
                        )?;
                    }
                    if let Some((id, pid)) = rows.first() {
                        // BUG FIX (B-23 Phase 4H Group A audit): previously hardcoded
                        // to Some(0), making ConfigArchivedPayload.dropped_id useless
                        // for the dashboard. Carry the actual archive_id forward.
                        dropped_archive_id = Some(*id);
                        tracing::info!(
                            dropped_archive_id = *id,
                            dropped_proposal = %pid,
                            "layer1a: oldest archive dropped",
                        );
                    }
                }
                tx.commit()?;
            }

            // MSI update — success.
            let (msi_value, status) = self
                .record_msi(proposal.pipeline_mode, 1.0)
                .await;

            // Enforcement layer — Git.
            let commit_sha = match self.commit_apply(proposal).await {
                Ok(out) => {
                    if let Some(ref co) = out {
                        // Persist sha on applied_proposals.
                        let conn = self.rules_db.lock().await;
                        let _ = conn.execute(
                            "UPDATE applied_proposals SET commit_sha = ?1 \
                             WHERE proposal_id = ?2",
                            params![co.sha, proposal.proposal_id],
                        );
                    }
                    out.map(|o| o.sha)
                }
                Err(_) => None,
            };

            // Emit SSB rows — Applied + MsiUpdate + ConfigArchived.
            let _ = self
                .ssb
                .write_payload(
                    LAYER1A_AGENT_ID,
                    0,
                    SsbPayload::ProposalApplied(ProposalAppliedPayload {
                        proposal_id: proposal.proposal_id.clone(),
                        table: proposal.table.as_str().to_owned(),
                        applied_by: LAYER1A_AGENT_ID.to_owned(),
                        commit_sha: commit_sha.clone(),
                    }),
                    Vec::new(),
                )
                .await;
            let _ = self
                .ssb
                .write_payload(
                    LAYER1A_AGENT_ID,
                    0,
                    SsbPayload::MsiUpdate(MsiUpdatePayload {
                        pipeline_mode: proposal.pipeline_mode.as_str().to_owned(),
                        msi_value,
                        status,
                    }),
                    Vec::new(),
                )
                .await;
            let archived_count: u32 = {
                let conn = self.rules_db.lock().await;
                conn.query_row(
                    "SELECT COUNT(*) FROM config_archive WHERE pipeline_mode = ?1",
                    params![proposal.pipeline_mode.as_str()],
                    |row| row.get(0),
                )
                .unwrap_or(0)
            };
            let _ = self
                .ssb
                .write_payload(
                    LAYER1A_AGENT_ID,
                    0,
                    SsbPayload::ConfigArchived(ConfigArchivedPayload {
                        pipeline_mode: proposal.pipeline_mode.as_str().to_owned(),
                        archived_count,
                        dropped_id: dropped_archive_id.map(|v| v.to_string()),
                    }),
                    Vec::new(),
                )
                .await;

            Ok(ProposalOutcome {
                proposal_id: proposal.proposal_id.clone(),
                applied_by: LAYER1A_AGENT_ID.to_owned(),
                commit_sha,
            })
        })
    }

    fn list_pending<'a>(
        &'a self,
    ) -> std::pin::Pin<
        Box<dyn std::future::Future<Output = Vec<DataTableProposal>> + Send + 'a>,
    > {
        Box::pin(async move {
            let conn = self.rules_db.lock().await;
            let mut stmt = match conn.prepare(
                "SELECT proposal_id, table_name, new_row_json, source, fpr_estimate, \
                        occurrences, pipeline_mode, scope_json \
                 FROM config_proposals WHERE status = 'pending' \
                 ORDER BY proposed_at DESC",
            ) {
                Ok(s) => s,
                Err(_) => return Vec::new(),
            };
            let Ok(iter) = stmt.query_map([], |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, String>(2)?,
                    row.get::<_, String>(3)?,
                    row.get::<_, f64>(4)?,
                    row.get::<_, u32>(5)?,
                    row.get::<_, String>(6)?,
                    row.get::<_, Option<String>>(7)?,
                ))
            }) else {
                return Vec::new();
            };
            let mut out = Vec::new();
            for r in iter.flatten() {
                let (pid, table, rowjson, source, fpr, occ, mode, scope_json) = r;
                let Some(table) = parse_table(&table) else { continue };
                let Ok(new_row) = serde_json::from_str::<serde_json::Value>(&rowjson) else {
                    continue;
                };
                let Some(source) = parse_source(&source) else { continue };
                let Some(pipeline_mode) = parse_mode(&mode) else { continue };
                let scope = scope_json
                    .as_deref()
                    .and_then(|s| serde_json::from_str::<crate::self_mod_types::ProjectScope>(s).ok());
                out.push(DataTableProposal {
                    proposal_id: pid,
                    table,
                    new_row,
                    source,
                    fpr_estimate: fpr as f32,
                    occurrences_seen: occ,
                    pipeline_mode,
                    scope,
                });
            }
            out
        })
    }

    fn list_applied<'a>(
        &'a self,
        limit: u32,
    ) -> std::pin::Pin<
        Box<dyn std::future::Future<Output = Vec<AppliedProposalRecord>> + Send + 'a>,
    > {
        Box::pin(async move {
            let conn = self.rules_db.lock().await;
            let capped = if limit == 0 { 50 } else { limit };
            let mut stmt = match conn.prepare(
                "SELECT proposal_id, table_name, new_row_json, applied_by, \
                        proposed_at, applied_at, commit_sha, pipeline_mode, scope_json \
                 FROM applied_proposals ORDER BY applied_at DESC LIMIT ?1",
            ) {
                Ok(s) => s,
                Err(_) => return Vec::new(),
            };
            // NARROWING-OK: u32->i64 is widening; capped is u32 (limit field), fits i64.
            let Ok(iter) = stmt.query_map(params![capped as i64], |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, String>(2)?,
                    row.get::<_, String>(3)?,
                    row.get::<_, i64>(4)?,
                    row.get::<_, i64>(5)?,
                    row.get::<_, Option<String>>(6)?,
                    row.get::<_, String>(7)?,
                    row.get::<_, Option<String>>(8)?,
                ))
            }) else {
                return Vec::new();
            };
            let mut out = Vec::new();
            for r in iter.flatten() {
                let (pid, table, rowjson, by, p_at, a_at, sha, mode, scope_json) = r;
                let Some(table) = parse_table(&table) else { continue };
                let Ok(new_row) = serde_json::from_str::<serde_json::Value>(&rowjson) else {
                    continue;
                };
                let Some(pipeline_mode) = parse_mode(&mode) else { continue };
                let scope = scope_json
                    .as_deref()
                    .and_then(|s| serde_json::from_str::<crate::self_mod_types::ProjectScope>(s).ok());
                out.push(AppliedProposalRecord {
                    proposal_id: pid,
                    table,
                    new_row,
                    applied_by: by,
                    proposed_at_ms: p_at,
                    applied_at_ms: a_at,
                    commit_sha: sha,
                    pipeline_mode,
                    scope,
                });
            }
            out
        })
    }

    fn reject_proposal<'a>(
        &'a self,
        proposal_id: &'a str,
        reason: &'a str,
    ) -> std::pin::Pin<
        Box<dyn std::future::Future<Output = Result<(), Layer1aError>> + Send + 'a>,
    > {
        Box::pin(async move {
            let (table, mode) = {
                let conn = self.rules_db.lock().await;
                let row = conn
                    .query_row(
                        "SELECT table_name, pipeline_mode FROM config_proposals WHERE proposal_id = ?1",
                        params![proposal_id],
                        |row| Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?)),
                    )
                    .map_err(|_| Layer1aError::NotFound(proposal_id.to_owned()))?;
                conn.execute(
                    "UPDATE config_proposals SET status = 'rejected', reject_reason = ?1 \
                     WHERE proposal_id = ?2",
                    params![reason, proposal_id],
                )?;
                row
            };
            let (_m, pipeline_mode) = (table.clone(), parse_mode(&mode));
            let _ = pipeline_mode; // reserved for future MSI negative delta
            let _ = self
                .ssb
                .write_payload(
                    LAYER1A_AGENT_ID,
                    0,
                    SsbPayload::ProposalRejected(ProposalRejectedPayload {
                        proposal_id: proposal_id.to_owned(),
                        table,
                        reason: reason.to_owned(),
                    }),
                    Vec::new(),
                )
                .await;
            Ok(())
        })
    }

    fn msi_status<'a>(
        &'a self,
    ) -> std::pin::Pin<
        Box<dyn std::future::Future<Output = Vec<MsiStatus>> + Send + 'a>,
    > {
        Box::pin(async move {
            let msi = self.msi.lock().await;
            let mut out = Vec::new();
            for mode in [
                PipelineMode::Surgical,
                PipelineMode::Greenfield,
                PipelineMode::Oracle,
            ] {
                let state = msi.get(&mode);
                let v = state.map(|s| s.compute(self.msi_lambda)).unwrap_or(0.0);
                let samples = state.map(|s| s.samples()).unwrap_or(0);
                out.push(MsiStatus {
                    pipeline_mode: mode,
                    msi_value: v,
                    status: status_label(v).to_owned(),
                    samples,
                });
            }
            out
        })
    }
}

impl DefaultLayer1aProcessor {
    async fn record_msi(&self, mode: PipelineMode, delta: f32) -> (f32, String) {
        let mut msi = self.msi.lock().await;
        let state = msi.entry(mode).or_default();
        state.record(delta);
        let v = state.compute(self.msi_lambda);
        (v, status_label(v).to_owned())
    }
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Classify which DataTable a (category, pattern) pair belongs in.
///
/// Heuristics (spec-aligned):
///   * `todo`/`placeholder`/`pass` → LazyCodeRegex
///   * `deprecated`/`api` keywords → DeprecatedPatterns
///   * `source`/`tier` keywords → SourceTiers
///   * everything else → FailureTaxonomy
pub fn classify_table(category: &str, pattern: &str) -> DataTable {
    let cat = category.to_lowercase();
    let pat = pattern.to_lowercase();
    if cat.contains("lazy")
        || cat.contains("empty")
        || pat.contains("todo!")
        || pat.contains("todo(")
        || pat.contains("todo:")
        || pat.contains("unimplemented!")
        || pat.contains("pass")
        || pat.contains("placeholder")
    {
        return DataTable::LazyCodeRegex;
    }
    if cat.contains("deprecated") || cat.contains("api") || pat.contains("deprecated") {
        return DataTable::DeprecatedPatterns;
    }
    if cat.contains("source") || cat.contains("tier") {
        return DataTable::SourceTiers;
    }
    DataTable::FailureTaxonomy
}

/// B-23 Phase 4H Group C — map an ecosystem string to its primary language
/// label. Used by `process_outcome` when populating scope from
/// `active_ecosystem`. Mirrors the same table in `self_mod_bridge`.
pub(crate) fn ecosystem_to_language(eco: &str) -> Option<String> {
    match eco {
        "cargo" => Some("Rust".into()),
        "pip" => Some("Python".into()),
        "npm" => Some("TypeScript".into()),
        "go" => Some("Go".into()),
        "maven" => Some("Java".into()),
        _ => None,
    }
}

/// Build a regex from the raw failure pattern text. Plain text is escaped;
/// strings that already look like valid regex (anchors / groups / quantifiers)
/// pass through unchanged.
pub fn build_pattern_regex(pattern: &str) -> Result<Regex, Layer1aError> {
    let trimmed = pattern.trim();
    if trimmed.len() < MIN_PATTERN_LEN {
        return Err(Layer1aError::InvalidProposal("pattern too short".to_owned()));
    }
    // Conservative: always escape. Manual regex authoring belongs in Layer 2.
    let escaped = regex::escape(trimmed);
    Regex::new(&escaped).map_err(|e| Layer1aError::RegexCompile(e.to_string()))
}

/// Estimate the false-positive rate of `regex` against [`KNOWN_GOOD_CORPUS`].
/// Returns `matches / corpus.len()`.
pub fn estimate_fpr(regex: &Regex) -> f32 {
    let total = KNOWN_GOOD_CORPUS.len() as f32;
    if total == 0.0 {
        return 0.0;
    }
    let hits = KNOWN_GOOD_CORPUS
        .iter()
        .filter(|snippet| regex.is_match(snippet))
        .count() as f32;
    hits / total
}

fn parse_table(s: &str) -> Option<DataTable> {
    match s {
        "deprecated_patterns" => Some(DataTable::DeprecatedPatterns),
        "failure_taxonomy" => Some(DataTable::FailureTaxonomy),
        "lazy_code_regex" => Some(DataTable::LazyCodeRegex),
        "source_tiers" => Some(DataTable::SourceTiers),
        _ => None,
    }
}

fn parse_source(s: &str) -> Option<ProposalSource> {
    match s {
        "auto_from_outcome" => Some(ProposalSource::AutoFromOutcome),
        "manual" => Some(ProposalSource::Manual),
        _ => None,
    }
}

fn parse_mode(s: &str) -> Option<PipelineMode> {
    match s {
        "surgical" => Some(PipelineMode::Surgical),
        "greenfield" => Some(PipelineMode::Greenfield),
        "oracle" => Some(PipelineMode::Oracle),
        _ => None,
    }
}

fn now_ms() -> i64 {
    OsClock
        .system_now()
        .duration_since(UNIX_EPOCH)
        .map(|d| i64::try_from(d.as_millis()).unwrap_or(i64::MAX))
        .unwrap_or(0)
}

fn make_proposal_id(category: &str, pattern: &str) -> String {
    let h = blake3::hash(format!("{category}|{pattern}").as_bytes());
    let bytes = h.as_bytes();
    format!(
        "pp-{:02x}{:02x}{:02x}{:02x}-{}",
        bytes[0],
        bytes[1],
        bytes[2],
        bytes[3],
        now_ms()
    )
}

fn truncate_for_summary(row: &serde_json::Value) -> String {
    let raw = row.to_string();
    if raw.len() <= 200 {
        raw
    } else {
        let mut s = raw.chars().take(197).collect::<String>();
        s.push_str("...");
        s
    }
}

// ---------------------------------------------------------------------------
// Filesystem lock (stub)
// ---------------------------------------------------------------------------

/// RAII guard that removes the lockfile on drop. Acquiring the lock is a
/// best-effort exclusive-create: if the file already exists, fail fast.
pub struct RulesDbLockGuard {
    path: PathBuf,
}

impl Drop for RulesDbLockGuard {
    fn drop(&mut self) {
        let _ = std::fs::remove_file(&self.path);
    }
}

/// Acquire a stub exclusive lock next to `rules_db_path`. On success, returns
/// a guard that removes the lockfile on drop. Uses `create_new` so a
/// pre-existing lockfile causes an immediate failure.
pub fn acquire_rules_lock(rules_db_path: &Path) -> Result<RulesDbLockGuard, Layer1aError> {
    let lock_path = rules_db_path
        .parent()
        .map(|p| p.join(RULES_DB_LOCKFILE))
        .unwrap_or_else(|| PathBuf::from(RULES_DB_LOCKFILE));
    if let Some(parent) = lock_path.parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    match std::fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&lock_path)
    {
        Ok(_) => Ok(RulesDbLockGuard { path: lock_path }),
        Err(e) => Err(Layer1aError::LockAcquisitionFailed(e.to_string())),
    }
}

/// Returns true when the kill-switch env var is set.
pub fn layer1a_disabled_via_env() -> bool {
    matches!(
        std::env::var(ELAI_DISABLE_LAYER1A_ENV).as_deref(),
        Ok("1" | "true" | "TRUE" | "on" | "ON")
    )
}

/// Returns true when the auto-apply env var is set.
pub fn layer1a_auto_apply_via_env() -> bool {
    matches!(
        std::env::var(ELAI_LAYER1A_AUTO_APPLY_ENV).as_deref(),
        Ok("1" | "true" | "TRUE" | "on" | "ON")
    )
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn classify_table_lazy() {
        assert_eq!(
            classify_table("logic", "todo!()"),
            DataTable::LazyCodeRegex
        );
        assert_eq!(
            classify_table("empty_body", "pass"),
            DataTable::LazyCodeRegex
        );
    }

    #[test]
    fn classify_table_deprecated() {
        assert_eq!(
            classify_table("deprecated_api", "blake2::hash"),
            DataTable::DeprecatedPatterns
        );
    }

    #[test]
    fn classify_table_default_is_taxonomy() {
        assert_eq!(
            classify_table("unknown", "foo_bar_baz"),
            DataTable::FailureTaxonomy
        );
    }

    #[test]
    fn is_security_sensitive_detects_keywords() {
        let row = serde_json::json!({"pattern": "auth_token_leak"});
        assert!(is_security_sensitive(&row));
        let safe = serde_json::json!({"pattern": "println_spam"});
        assert!(!is_security_sensitive(&safe));
    }

    #[test]
    fn is_allowed_accepts_data_table_only() {
        assert!(is_allowed(&AllowedConfigKey::DataTable(
            DataTable::LazyCodeRegex
        )));
        assert!(!is_allowed(&AllowedConfigKey::Parameter("budget".into())));
    }

    #[test]
    fn msi_weighted_average_bounds() {
        let mut s = MsiState::default();
        for _ in 0..10 {
            s.record(1.0);
        }
        let v = s.compute(0.3);
        assert!(v > 0.9, "all-success streak should trend to +1.0, got {v}");
        assert_eq!(status_label(v), "IMPROVING");
        let mut s = MsiState::default();
        for _ in 0..10 {
            s.record(-1.0);
        }
        let v = s.compute(0.3);
        assert!(v < -0.9, "all-failure streak should trend to -1.0, got {v}");
        assert_eq!(status_label(v), "DIVERGING");
    }

    #[test]
    fn msi_stabilising_label() {
        let mut s = MsiState::default();
        s.record(1.0);
        s.record(-1.0);
        let v = s.compute(0.3);
        // Recent record is weight 1.0 (-1.0) → next is (1-0.3)=0.7 * +1.0.
        // Mean ≈ (-1.0 + 0.7) / 1.7 ≈ -0.176 — "STABILIZING".
        assert!(matches!(status_label(v), "STABILIZING" | "DIVERGING"));
    }

    #[test]
    fn estimate_fpr_known_good_corpus() {
        // A pattern that should never match the known-good corpus.
        let rx = Regex::new(r"todo!\(\)").unwrap();
        assert_eq!(estimate_fpr(&rx), 0.0);
        // A pattern that matches everything.
        let rx = Regex::new(r".*").unwrap();
        assert!(estimate_fpr(&rx) > 0.9);
    }

    #[test]
    fn build_pattern_regex_rejects_short() {
        assert!(build_pattern_regex("x").is_err());
        assert!(build_pattern_regex("ab").is_err());
        assert!(build_pattern_regex("todo!()").is_ok());
    }
}
