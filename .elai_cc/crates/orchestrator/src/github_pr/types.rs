//! C-40: wire-format DTOs for the GitHub PR adapter.
//!
//! Field shapes match the `gh pr ... --json` flat schema for the small
//! subset of fields we actually consume. Anything we don't read is
//! intentionally absent so the parser fails closed on schema drift.

use serde::{Deserialize, Serialize};

/// Coarse PR state. Mirrors the `state` field of `gh pr view --json
/// state` which is one of: `OPEN`, `CLOSED`, `MERGED`. Any other value
/// is folded into [`PrState::Unknown`] so a future gh release can't
/// silently flip our control flow.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum PrState {
    /// PR is open and not yet merged or closed.
    Open,
    /// PR was closed without merging.
    Closed,
    /// PR was merged into its base branch.
    Merged,
    /// gh returned a state string we did not recognise.
    #[serde(other)]
    Unknown,
}

/// Request payload for [`crate::github_pr::GithubPrAdapter::open`].
/// Constructed by the CLI / dashboard layer; the adapter validates each
/// field at the trait boundary (non-empty title + head branch are
/// required).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PrOpenRequest {
    /// Repo in `owner/name` form. When `None`, gh infers from the
    /// current working directory's git remote.
    #[serde(default)]
    pub repo: Option<String>,
    /// PR title (gh `--title`).
    pub title: String,
    /// PR body (gh `--body`); may be empty for draft PRs but never
    /// missing — pass an empty string to be explicit.
    #[serde(default)]
    pub body: String,
    /// Source branch (gh `--head`). Required.
    pub head: String,
    /// Target branch (gh `--base`). Defaults to `main` at the CLI
    /// layer when None.
    #[serde(default)]
    pub base: Option<String>,
    /// Open as draft PR (gh `--draft`).
    #[serde(default)]
    pub draft: bool,
}

/// Wire-format response shape for [`crate::github_pr::GithubPrAdapter::open`]
/// and for each row of [`crate::github_pr::GithubPrAdapter::list`].
///
/// Matches the `gh pr list --json number,title,state,url,headRefName,
/// baseRefName,isDraft` schema for the fields we consume.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PrSummary {
    /// gh json field: `number`.
    pub number: u32,
    /// gh json field: `title`.
    pub title: String,
    /// gh json field: `state` (UPPERCASE).
    pub state: PrState,
    /// gh json field: `url` (full https URL to the PR).
    pub url: String,
    /// gh json field: `headRefName`.
    #[serde(rename = "headRefName", default)]
    pub head_ref_name: String,
    /// gh json field: `baseRefName`.
    #[serde(rename = "baseRefName", default)]
    pub base_ref_name: String,
    /// gh json field: `isDraft`. Defaults false when gh omits.
    #[serde(rename = "isDraft", default)]
    pub is_draft: bool,
}

/// Single CI check run row from `gh pr view --json statusCheckRollup`.
///
/// gh emits a heterogeneous array; we deliberately keep only the fields
/// we render (`name`, `state`, `conclusion`) so a schema addition can't
/// crash the parser.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PrCheckRun {
    /// gh json field: `name` (check name; falls back to empty string).
    #[serde(default)]
    pub name: String,
    /// gh json field: `status` (e.g. `IN_PROGRESS`, `COMPLETED`).
    #[serde(default)]
    pub status: String,
    /// gh json field: `conclusion` (e.g. `SUCCESS`, `FAILURE`, `null`).
    #[serde(default)]
    pub conclusion: String,
}

/// Result of [`crate::github_pr::GithubPrAdapter::status`].
///
/// Aggregates the subset of `gh pr view --json state,reviewDecision,
/// statusCheckRollup` we need for the dashboard's status drawer.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PrStatus {
    /// gh json field: `state`.
    pub state: PrState,
    /// gh json field: `reviewDecision` (e.g. `APPROVED`,
    /// `CHANGES_REQUESTED`, `REVIEW_REQUIRED`). Empty when no review yet.
    #[serde(rename = "reviewDecision", default)]
    pub review_decision: String,
    /// Flattened CI check rows from `statusCheckRollup`.
    #[serde(rename = "statusCheckRollup", default)]
    pub check_runs: Vec<PrCheckRun>,
}

impl PrStatus {
    /// Canonical snake_case wire label for the CI rollup: either
    /// `success` (every check has conclusion=SUCCESS), `pending`
    /// (any check status != COMPLETED), or `failing` (any conclusion
    /// in FAILURE/TIMED_OUT/CANCELLED/ERROR). Used as the input to the
    /// `ci_status_hash` field of the C-40 SSB row so the dashboard can
    /// render status-transition rows without leaking the raw rollup.
    #[must_use]
    pub fn ci_rollup_label(&self) -> &'static str {
        if self.check_runs.is_empty() {
            return "none";
        }
        let mut any_failing = false;
        let mut any_pending = false;
        for c in &self.check_runs {
            if c.status != "COMPLETED" {
                any_pending = true;
            }
            match c.conclusion.as_str() {
                "FAILURE" | "TIMED_OUT" | "CANCELLED" | "ERROR" => {
                    any_failing = true;
                }
                _ => {}
            }
        }
        if any_failing {
            "failing"
        } else if any_pending {
            "pending"
        } else {
            "success"
        }
    }

    /// Snake_case wire label for the review decision; `none` when gh
    /// returns an empty string. Stable input to the `review_state_hash`
    /// SSB field.
    #[must_use]
    pub fn review_state_label(&self) -> String {
        if self.review_decision.is_empty() {
            "none".to_string()
        } else {
            self.review_decision.to_ascii_lowercase()
        }
    }
}
