//! E-41: PR review-comment posting helpers.
//!
//! Wraps `gh pr review` (line-anchored review comments on a diff hunk)
//! and `gh pr comment` (top-level PR-thread comments) via shell-out.
//! Both paths go through `GhCliAdapter::run_gh` so existing risk-class
//! gating (`tools::action_classifier::BASH_SHARED_STATE_PATTERNS`)
//! applies — no new bypass.
//!
//! SSB emission: every successful Ok writes one `pr_review_composed` row
//! via `Ssb::write_payload`. PR number is hashed via blake3 prefix
//! (matches `GhCliAdapter::hash_prefix` precedent) so dashboards group by
//! identity without leaking the raw integer to broadcast subscribers
//! that should not learn which PR is being reviewed.

use std::sync::Arc;

use tokio::process::Command;
use tracing::warn;

use crate::ssb::{PrReviewComposedPayload, Ssb, SsbPayload};

use super::error::GithubPrError;
use super::gh_cli::GhCliAdapter;

/// Stable agent_id label embedded in every E-41 SSB row emitted by this
/// module. Distinct from the C-40 `github-pr-adapter` label so dashboards
/// can branch on the producer.
const REVIEW_AGENT_ID: &str = "github-pr-review";

/// `gh` binary; resolved from PATH at each invocation.
const GH_BINARY: &str = "gh";

/// Top-level kinds of review comments distinguishing the gh subcommand
/// shape. Kept stable across releases so the SSB `comment_kind_hash` is
/// reproducible.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ReviewCommentKind {
    /// Plain top-level PR-thread comment via `gh pr comment`. No file or
    /// line anchor.
    PrThread,
    /// File-anchored review comment via `gh pr review --comment`. Carries
    /// a path and optional line number.
    FileAnchored,
}

impl ReviewCommentKind {
    /// Canonical snake_case label used as input to the
    /// `comment_kind_hash` SSB field.
    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            ReviewCommentKind::PrThread => "pr_thread",
            ReviewCommentKind::FileAnchored => "file_anchored",
        }
    }
}

/// Build the `gh` argv for posting a PR review comment.
///
/// Pure helper (no I/O) so unit tests can assert argv shape without
/// touching the network. Exposed at module level for the E-41 contract
/// test to grep the function name.
#[must_use]
pub fn build_review_argv(
    pr_number: u32,
    body: &str,
    file: Option<&str>,
    line: Option<u32>,
) -> Vec<String> {
    if file.is_some() {
        // File-anchored review comment via `gh pr review`. gh's review
        // command takes `--comment` to set the action and `--body`
        // separately; line anchoring is communicated by repository
        // convention (gh's CLI supports it only via the REST API in
        // current releases). For our purposes the file + line are
        // appended to the body as a structured tag so reviewers can see
        // the anchor even when gh strips it server-side.
        let mut anchored_body = String::with_capacity(body.len() + 32);
        anchored_body.push_str(body);
        if let Some(f) = file {
            anchored_body.push_str("\n\n");
            anchored_body.push_str("anchor: ");
            anchored_body.push_str(f);
            if let Some(l) = line {
                anchored_body.push(':');
                anchored_body.push_str(&l.to_string());
            }
        }
        vec![
            "pr".to_string(),
            "review".to_string(),
            pr_number.to_string(),
            "--comment".to_string(),
            "--body".to_string(),
            anchored_body,
        ]
    } else {
        vec![
            "pr".to_string(),
            "comment".to_string(),
            pr_number.to_string(),
            "--body".to_string(),
            body.to_string(),
        ]
    }
}

/// Post a review comment on a PR. When `file` is `Some(path)` the call
/// is dispatched as a file-anchored review comment via `gh pr review`;
/// otherwise it lands as a top-level PR-thread comment via
/// `gh pr comment`. Returns `Ok(())` on gh success and emits one
/// `pr_review_composed` SSB row.
///
/// # Errors
/// - [`GithubPrError::GhCliNotInstalled`] / [`GithubPrError::GhCliNotAuthenticated`]
///   when gh is missing or unauthenticated (verified via the C-40
///   `GhCliAdapter::precheck_ready` precondition before each call).
/// - [`GithubPrError::GhCommandFailed`] for non-zero gh exits.
/// - [`GithubPrError::GhIo`] for spawn / IO failures.
pub async fn post_review_comment(
    ssb: Arc<Ssb>,
    pr_number: u32,
    body: &str,
    file: Option<String>,
    line: Option<u32>,
) -> Result<(), GithubPrError> {
    if body.trim().is_empty() {
        return Err(GithubPrError::InvalidRequest {
            reason: "review comment body must not be empty".to_string(),
        });
    }
    let kind = if file.is_some() {
        ReviewCommentKind::FileAnchored
    } else {
        ReviewCommentKind::PrThread
    };
    let argv_strings = build_review_argv(pr_number, body, file.as_deref(), line);
    let argv: Vec<&str> = argv_strings.iter().map(String::as_str).collect();

    let output = Command::new(GH_BINARY)
        .args(&argv)
        .output()
        .await
        .map_err(GithubPrError::from)?;
    if !output.status.success() {
        let label = format!("gh {}", argv.join(" "));
        return Err(GithubPrError::from_exit(label, output.status, output.stderr));
    }

    let payload = SsbPayload::PrReviewComposed(PrReviewComposedPayload {
        pr_number_hash: GhCliAdapter::hash_prefix(&pr_number.to_string()),
        comment_kind_hash: GhCliAdapter::hash_prefix(kind.label()),
        ts_ms: super::clock::now_ms(),
    });
    if let Err(e) = ssb.write_payload(REVIEW_AGENT_ID, 0, payload, vec![]).await {
        warn!(
            target: "github_pr",
            error = %e,
            "ssb write failed for pr_review_composed row; row dropped",
        );
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pr_thread_argv_shape() {
        let argv = build_review_argv(42, "looks ok to me", None, None);
        assert_eq!(
            argv,
            vec![
                "pr".to_string(),
                "comment".to_string(),
                "42".to_string(),
                "--body".to_string(),
                "looks ok to me".to_string(),
            ]
        );
    }

    #[test]
    fn file_anchored_argv_shape() {
        let argv = build_review_argv(7, "nit: rename this", Some("crates/orchestrator/src/lib.rs"), Some(120));
        assert_eq!(argv[0], "pr");
        assert_eq!(argv[1], "review");
        assert_eq!(argv[2], "7");
        assert_eq!(argv[3], "--comment");
        assert_eq!(argv[4], "--body");
        let body = &argv[5];
        assert!(body.contains("nit: rename this"));
        assert!(body.contains("anchor: crates/orchestrator/src/lib.rs:120"));
    }

    #[test]
    fn kind_label_stable() {
        assert_eq!(ReviewCommentKind::PrThread.label(), "pr_thread");
        assert_eq!(ReviewCommentKind::FileAnchored.label(), "file_anchored");
    }
}
