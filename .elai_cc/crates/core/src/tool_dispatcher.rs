//! Batch 2B — `ToolDispatcher` trait and value types.
//!
//! Lives in `elai-core` so every downstream crate (orchestrator, dashboard,
//! contracts) can program against a single canonical trait shape without
//! depending on `elai-tools` directly. The concrete `InProcessToolDispatcher`
//! impl lives in `elai-orchestrator` (see `in_process_tool_dispatcher.rs`);
//! putting the impl in `elai-tools` would force a dep cycle because
//! `elai-tools` already depends on `elai-orchestrator` for facade
//! re-exports (see `crates/tools/Cargo.toml`, recorded as "deferred crate
//! split" in batch-2b decision notes).
//!
//! No `async_trait` crate: we use native async-fn-in-trait (stabilised
//! in Rust 1.75, 2026-04 MSRV is well past that) per
//! CLAUDE.md anti-stale-training. The trait is therefore NOT object-safe
//! via `Box<dyn ToolDispatcher>` out of the box, but IS object-safe because
//! every async fn returns an `impl Future + Send` in the trait sugar and
//! the trait has no generics. Downstream callers store
//! `Arc<dyn ToolDispatcher>` the same way they store
//! `Arc<dyn HookDispatcher>` in Batch 1A.

use std::path::PathBuf;

use serde::{Deserialize, Serialize};
use thiserror::Error;

// ---------------------------------------------------------------------------
// Error
// ---------------------------------------------------------------------------

/// Errors surfaced by every `ToolDispatcher` method. `thiserror` variants
/// are chosen to match the spec §1 surface — downstream code should not
/// construct new variants without a spec bump.
#[derive(Debug, Error)]
pub enum ToolDispatchError {
    /// The dispatcher exists but the requested tool path is unimplemented.
    /// Used by test doubles and the stub fallback on the dashboard path
    /// before the real `InProcessToolDispatcher` has finished constructing.
    #[error("tool dispatch not implemented: {0}")]
    NotImplemented(String),

    /// I/O error bubbled from an underlying handler (filesystem, process
    /// spawn, stdin/stdout plumbing).
    #[error("tool io: {0}")]
    IoError(#[from] std::io::Error),

    /// `git2`, `git` CLI, or equivalent VCS adapter returned an error.
    /// Stored as `String` so the core crate does not need to depend on
    /// `git2`.
    #[error("git error: {0}")]
    GitError(String),

    /// One of the five validation tiers failed under its timeout.
    #[error("validation tier {tier} failed: {reason}")]
    ValidationFailed { tier: String, reason: String },

    /// A per-tier timeout elapsed before the tier produced a verdict.
    #[error("tool timeout after {elapsed_ms} ms")]
    Timeout { elapsed_ms: u32 },

    /// Cargo / PyPI crate could not be reached (incremental compile only).
    #[error("crate unreachable: {0}")]
    CrateUnreachable(String),
}

// ---------------------------------------------------------------------------
// Edit types
// ---------------------------------------------------------------------------

/// Matching tier reported by `dispatch_edit`. The order is load-bearing:
/// the edit handler tries them top-down and the first successful tier wins.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EditMatchTier {
    /// Agent-provided `old_text` is byte-identical to some region of the file.
    Exact,
    /// Agent-provided `old_text` matches after `\s+` normalisation (tabs vs
    /// spaces, trailing whitespace, CRLF vs LF).
    WhitespaceNormalized,
    /// Agent-provided `old_text` contains the `...` literal marker and the
    /// surrounding anchors match.
    Ellipsis,
    /// No tier matched — the edit was rejected.
    NotMatched,
}

impl EditMatchTier {
    /// Stable SSB-string representation. Used in `ToolDispatch` rows so the
    /// dashboard can filter "exact" vs "whitespace_normalized" matches.
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Exact => "exact",
            Self::WhitespaceNormalized => "whitespace_normalized",
            Self::Ellipsis => "ellipsis",
            Self::NotMatched => "not_matched",
        }
    }
}

/// Canonical request shape for `dispatch_edit`. Mirrors the existing
/// `elai-ipc-types::EditRequest` but without a postcard-only dependency.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EditRequest {
    pub file_path: PathBuf,
    pub old_text: String,
    pub new_text: String,
}

/// Outcome of a `dispatch_edit` call.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EditOutcome {
    /// Which tier actually matched. `NotMatched` means the edit was refused.
    pub tier: EditMatchTier,
    /// Number of bytes replaced (length of `new_text` on success; 0 on
    /// `NotMatched`).
    pub bytes_written: u32,
    /// File path after edit (same as request).
    pub file_path: PathBuf,
}

// ---------------------------------------------------------------------------
// Validation types
// ---------------------------------------------------------------------------

/// 5-tier validation pipeline per phase-4B.txt § validation.
///
/// Order is load-bearing: `run_all` iterates in declaration order and
/// short-circuits on the first failure.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ValidationTier {
    /// Re-run a fast deterministic command twice; compare blake3 hashes.
    /// Target budget: < 5 s.
    Determinism,
    /// Per-language syntax / AST check (`cargo check`, `tsc --noEmit`,
    /// `go vet`, `py_compile`/`ruff check`). Target budget: < 10 s.
    QualityAst,
    /// Module-scoped tests (Step level in phase-4E.txt). Target: < 30 s.
    Unit,
    /// Crate-scoped tests (Phase level in phase-4E.txt). Target: < 60 s.
    Integration,
    /// Workspace tests + repeat-N flakiness probe on the Unit tier.
    /// Target: < 120 s.
    Stress,
}

impl ValidationTier {
    /// Stable SSB-string representation. Used in `ValidationTier` rows so
    /// the dashboard can render "determinism ✓ 1.2 s".
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Determinism => "determinism",
            Self::QualityAst => "quality_ast",
            Self::Unit => "unit",
            Self::Integration => "integration",
            Self::Stress => "stress",
        }
    }

    /// Ordered list of tiers in short-circuit order. Used by both
    /// `DefaultTieredValidator::run_all` and the contract tests.
    pub fn ordered() -> [ValidationTier; 5] {
        [
            Self::Determinism,
            Self::QualityAst,
            Self::Unit,
            Self::Integration,
            Self::Stress,
        ]
    }

    /// Per-tier timeout in milliseconds. Matches the spec budget.
    pub fn timeout_ms(&self) -> u32 {
        match self {
            Self::Determinism => 5_000,
            Self::QualityAst => 10_000,
            Self::Unit => 30_000,
            Self::Integration => 60_000,
            Self::Stress => 120_000,
        }
    }
}

/// Request payload for `dispatch_validate`. Keeps `project_dir` separate
/// from `changed_files` so a tier can decide whether to run workspace-scoped
/// (`project_dir`) or file-scoped commands.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ValidateRequest {
    pub project_dir: PathBuf,
    pub language: String,
    pub changed_files: Vec<PathBuf>,
    /// Baby-step identifier used for the emitted SSB row's `step_id`.
    pub step_id: String,
    /// Optional subset of tiers to run. `None` ⇒ run all 5 in short-circuit
    /// order. `Some(list)` ⇒ run only the listed tiers, still in declared
    /// order.
    pub tiers: Option<Vec<ValidationTier>>,
}

/// Per-tier verdict. `passed=false` on any tier aborts the pipeline.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ValidationTierResult {
    pub tier: ValidationTier,
    pub passed: bool,
    pub elapsed_ms: u32,
    pub rules_fired: u32,
    /// First 256 bytes of stderr when the tier failed. `None` on pass.
    pub stderr_snippet: Option<String>,
}

// ---------------------------------------------------------------------------
// Failure classification
// ---------------------------------------------------------------------------

/// 13-variant failure taxonomy (superset of the 9 categories in phase-4B.txt
/// §failure-taxonomy). Extra variants (`LazyCode`, `EmptyBody`,
/// `FileSizeExceeded`, `ScriptQualityDependency`) are ours from a prior
/// wiring pass; keeping them in-place preserves backward compatibility with
/// every downstream caller. Re-exported from `elai_tools::failure_taxonomy`
/// via a thin conversion in `in_process_tool_dispatcher.rs`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ToolFailureCategory {
    CompilationError,
    TestFailure,
    RuntimeError,
    Timeout,
    ResourceExhausted,
    LazyCode,
    EmptyBody,
    FileSizeExceeded,
    ContextExplosion,
    RawLogOverload,
    PhaseGateFailure,
    ScriptQualityDependency,
    SpeculativeRepair,
}

impl ToolFailureCategory {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::CompilationError => "COMPILATION_ERROR",
            Self::TestFailure => "TEST_FAILURE",
            Self::RuntimeError => "RUNTIME_ERROR",
            Self::Timeout => "TIMEOUT",
            Self::ResourceExhausted => "RESOURCE_EXHAUSTED",
            Self::LazyCode => "LAZY_CODE",
            Self::EmptyBody => "EMPTY_BODY",
            Self::FileSizeExceeded => "FILE_SIZE_EXCEEDED",
            Self::ContextExplosion => "CONTEXT_EXPLOSION",
            Self::RawLogOverload => "RAW_LOG_OVERLOAD",
            Self::PhaseGateFailure => "PHASE_GATE_FAILURE",
            Self::ScriptQualityDependency => "SCRIPT_QUALITY_DEPENDENCY",
            Self::SpeculativeRepair => "SPECULATIVE_REPAIR",
        }
    }
}

/// Input to `dispatch_classify_failure`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FailureInput {
    /// Compiler / test-runner output. Truncated to the last 16 KiB before
    /// being handed to the classifier.
    pub raw_output: String,
    /// Baby-step identifier for SSB correlation.
    pub step_id: String,
    /// Which of (`agent`, `validator`, `deterministic_audit`) raised the
    /// failure. Used only for SSB `FailureClassified.source`.
    pub source: String,
}

// ---------------------------------------------------------------------------
// Git commit
// ---------------------------------------------------------------------------

/// Branch-policy config for `dispatch_git_commit`. Default: `BranchPerPlan`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum GitBranchPolicy {
    /// Commit to the currently-checked-out branch. Appropriate for
    /// single-agent flows and the existing CLI path.
    Main,
    /// Create-or-checkout `elai/plan-{plan_id}` before committing. Keeps
    /// concurrent plans isolated on dedicated branches.
    BranchPerPlan,
}

impl Default for GitBranchPolicy {
    fn default() -> Self {
        Self::BranchPerPlan
    }
}

/// Request for `dispatch_git_commit`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GitCommitRequest {
    /// Path inside the working tree. The dispatcher walks up to the repo
    /// root from here.
    pub anchor_path: PathBuf,
    /// Files to stage — explicit list, never "." or "-A".
    pub files: Vec<PathBuf>,
    pub message: String,
    pub branch_policy: GitBranchPolicy,
    /// Identifier used to name the per-plan branch when
    /// `branch_policy = BranchPerPlan`. Empty string ⇒ fall back to
    /// `"default"`.
    pub plan_id: String,
    pub step_id: String,
}

/// Outcome of a successful commit.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CommitOutcome {
    pub sha: String,
    pub branch: String,
    pub files_count: u32,
    pub message: String,
}

// ---------------------------------------------------------------------------
// File read outline
// ---------------------------------------------------------------------------

/// Response for `dispatch_file_read_outline`. When the target file is
/// above the threshold, the dispatcher returns a signature/import-only
/// view; otherwise it returns the full content.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FileView {
    pub outline_only: bool,
    pub signatures: Vec<String>,
    pub imports: Vec<String>,
    pub full_content: Option<String>,
    pub total_lines: u32,
}

// ---------------------------------------------------------------------------
// Incremental compile
// ---------------------------------------------------------------------------

/// Request for `dispatch_incremental_compile`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct IncrementalRequest {
    pub project_dir: PathBuf,
    pub language: String,
    pub retry_round: u8,
    pub step_id: String,
}

/// Outcome of an incremental compile attempt.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct IncrementalOutcome {
    pub success: bool,
    pub attempts: u8,
    pub first_error_category: Option<ToolFailureCategory>,
    pub elapsed_ms: u32,
}

// ---------------------------------------------------------------------------
// Trait
// ---------------------------------------------------------------------------

/// Central dispatcher for the 4B tool suite. All six methods are native
/// async fn (no `async_trait` crate), returning `impl Future + Send` sugar.
///
/// Downstream crates store `Arc<dyn ToolDispatcher>` in config structs.
/// Absence (`None`) is the documented graceful fallback — `drive_task`
/// skips every dispatch site when the field is empty.
pub trait ToolDispatcher: Send + Sync {
    /// Apply a 3-tier edit (Exact → WhitespaceNormalized → Ellipsis).
    fn dispatch_edit(
        &self,
        req: EditRequest,
    ) -> std::pin::Pin<
        Box<dyn std::future::Future<Output = Result<EditOutcome, ToolDispatchError>> + Send + '_>,
    >;

    /// Run the 5-tier validation pipeline. Short-circuits on first fail.
    fn dispatch_validate(
        &self,
        req: ValidateRequest,
    ) -> std::pin::Pin<
        Box<
            dyn std::future::Future<
                    Output = Result<Vec<ValidationTierResult>, ToolDispatchError>,
                > + Send
                + '_,
        >,
    >;

    /// Classify a compile/test/runtime failure into one of 13 categories.
    /// Returns `None` when the classifier can't find a confident match.
    fn dispatch_classify_failure(
        &self,
        input: FailureInput,
    ) -> std::pin::Pin<
        Box<dyn std::future::Future<Output = Option<ClassifiedFailure>> + Send + '_>,
    >;

    /// Stage + commit the listed files via `git2`. Honours `branch_policy`.
    fn dispatch_git_commit(
        &self,
        req: GitCommitRequest,
    ) -> std::pin::Pin<
        Box<
            dyn std::future::Future<Output = Result<CommitOutcome, ToolDispatchError>>
                + Send
                + '_,
        >,
    >;

    /// Return outline-only for big files, full content for small ones.
    fn dispatch_file_read_outline(
        &self,
        path: PathBuf,
        line_threshold: usize,
    ) -> std::pin::Pin<
        Box<dyn std::future::Future<Output = Result<FileView, ToolDispatchError>> + Send + '_>,
    >;

    /// Incremental compile with retry (≤ 3 rounds).
    fn dispatch_incremental_compile(
        &self,
        req: IncrementalRequest,
    ) -> std::pin::Pin<
        Box<
            dyn std::future::Future<Output = Result<IncrementalOutcome, ToolDispatchError>>
                + Send
                + '_,
        >,
    >;
}

/// Wrapper returned by `dispatch_classify_failure` so downstream callers
/// can access confidence alongside the category.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ClassifiedFailure {
    pub category: ToolFailureCategory,
    /// Confidence in [0.0, 1.0]. 1.0 on exact regex match; 0.7 on partial.
    pub confidence: f32,
    pub source: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn validation_tier_ordered_has_five_elements() {
        let ordered = ValidationTier::ordered();
        assert_eq!(ordered.len(), 5);
        assert_eq!(ordered[0], ValidationTier::Determinism);
        assert_eq!(ordered[4], ValidationTier::Stress);
    }

    #[test]
    fn validation_tier_timeout_is_monotonic_non_decreasing() {
        let tiers = ValidationTier::ordered();
        for pair in tiers.windows(2) {
            assert!(
                pair[0].timeout_ms() <= pair[1].timeout_ms(),
                "tier budget must be non-decreasing: {pair:?}",
            );
        }
    }

    #[test]
    fn edit_match_tier_as_str_is_snake_case() {
        assert_eq!(EditMatchTier::WhitespaceNormalized.as_str(), "whitespace_normalized");
        assert_eq!(EditMatchTier::Ellipsis.as_str(), "ellipsis");
    }

    #[test]
    fn git_branch_policy_default_is_branch_per_plan() {
        assert_eq!(GitBranchPolicy::default(), GitBranchPolicy::BranchPerPlan);
    }
}
