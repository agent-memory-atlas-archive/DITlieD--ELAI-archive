//! C-40: [`GithubPrAdapter`] trait — 5-method async surface for
//! orchestrating PR lifecycle from /session merge + the dashboard.
//!
//! Per CLAUDE.md anti-stale-training rule: **native async fn in
//! traits** (no `async-trait` crate, no boxed-future tax). The trait is
//! consumed at concrete-type call sites only (`GhCliAdapter` is the
//! sole impl shipped in this slice); no `Arc<dyn GithubPrAdapter>`
//! dispatch is required.

use super::error::GithubPrError;
use super::types::{PrOpenRequest, PrStatus, PrSummary};

/// Async-fn-in-trait surface for opening, listing, inspecting, closing,
/// and merging GitHub pull requests.
///
/// Implementations MUST satisfy these wiring-contract invariants:
///
/// 1. Every Ok return writes exactly one corresponding C-40 SSB row
///    (`github_pr_opened` / `github_pr_status_fetched` / `github_pr_closed`
///    / `github_pr_merged`). `list` is read-only and does NOT emit SSB.
/// 2. Errors are surfaced via [`GithubPrError`]; no panics, no
///    `Box<dyn Error>`, no anyhow at the trait boundary.
/// 3. Sensitive fields (PR URL, merge SHA, CI/review labels) are
///    hashed via the implementation before SSB emission — the trait
///    contract is that no raw URL/SHA reaches the SSB broadcast.
pub trait GithubPrAdapter: Send + Sync {
    /// Open a new pull request via `gh pr create`. Emits one
    /// `github_pr_opened` SSB row on success. Returns the freshly
    /// created PR's summary (number + URL + draft/title fields).
    fn open(
        &self,
        request: PrOpenRequest,
    ) -> impl std::future::Future<Output = Result<PrSummary, GithubPrError>> + Send;

    /// List open + closed PRs for the given repo (or current working
    /// directory's git remote when `repo` is None). Limit is the gh
    /// `--limit` flag; defaults to 20 when 0.
    ///
    /// Read-only — does NOT emit an SSB row (per WIRING_CONTRACT: only
    /// state-changing or non-trivial fetches emit; `list` is treated as
    /// pure observation, mirroring the wip_squash::count_leading_wip_commits
    /// non-emission rule).
    fn list(
        &self,
        repo: Option<String>,
        state_filter: Option<String>,
        limit: u32,
    ) -> impl std::future::Future<Output = Result<Vec<PrSummary>, GithubPrError>> + Send;

    /// Fetch the status (state + review decision + CI rollup) for a
    /// single PR via `gh pr view --json state,reviewDecision,statusCheckRollup`.
    /// Emits one `github_pr_status_fetched` SSB row on success.
    fn status(
        &self,
        repo: Option<String>,
        pr_number: u32,
    ) -> impl std::future::Future<Output = Result<PrStatus, GithubPrError>> + Send;

    /// Close a PR via `gh pr close <pr_number>` WITHOUT merging.
    /// Emits one `github_pr_closed` SSB row on success.
    fn close(
        &self,
        repo: Option<String>,
        pr_number: u32,
    ) -> impl std::future::Future<Output = Result<(), GithubPrError>> + Send;

    /// Merge a PR via `gh pr merge <pr_number> --squash` (default merge
    /// strategy; callers that need rebase/merge-commit can extend the
    /// API in a follow-up). Emits one `github_pr_merged` SSB row on
    /// success. Returns the merge commit SHA when gh reports it (gh
    /// emits it on the `--json mergeCommit` view after a successful
    /// merge); returns empty string when gh omits the field (e.g. when
    /// the merge is queued).
    fn merge(
        &self,
        repo: Option<String>,
        pr_number: u32,
    ) -> impl std::future::Future<Output = Result<String, GithubPrError>> + Send;
}
