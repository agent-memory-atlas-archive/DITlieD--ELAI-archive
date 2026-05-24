//! E-41: auto-merge safety guard.
//!
//! Two surfaces:
//!
//! - [`assess_merge_safety`] — pure function: given a PR's diff size +
//!   touched-file count + the configured [`MergePolicy`], return a
//!   deterministic [`MergeVerdict`]. Emits a `pr_auto_merge_blocked` SSB
//!   row when the verdict is [`MergeVerdict::Block`].
//! - [`apply_branch_protection`] — one-shot configurator that calls
//!   `gh api PUT /repos/{owner}/{repo}/branches/{branch}/protection`
//!   to install the server-side guardrails (required reviewers, linear
//!   history, dismissal of stale reviews, etc). Async, gated behind the
//!   `pr config --apply-branch-protection` CLI subverb.
//!
//! Both surfaces are advisory: ELAI does NOT auto-merge anywhere — the
//! guard is the wire shape for downstream automation to consult.

use std::sync::Arc;

use tokio::process::Command;
use tracing::warn;

use crate::ssb::{PrAutoMergeBlockedPayload, Ssb, SsbPayload};

use super::error::GithubPrError;
use super::gh_cli::GhCliAdapter;

const GUARD_AGENT_ID: &str = "github-pr-auto-merge-guard";
const GH_BINARY: &str = "gh";

/// Policy configuration for the auto-merge guard. Defaults are the
/// industry-recommended values for AI-co-authored PR risk (Faros 2026,
/// CodeRabbit state-of-AI):
/// - max 100 added/removed LOC delta
/// - max 1 file touched
/// - any owned path -> require human approval
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MergePolicy {
    /// Hard ceiling on LOC delta (added + removed). PRs above this
    /// cannot auto-merge regardless of file count.
    pub max_loc_delta: u32,
    /// Hard ceiling on touched-file count.
    pub max_files: u32,
    /// When true, ANY CODEOWNERS-mapped touched path blocks auto-merge.
    pub block_on_owned_paths: bool,
}

impl Default for MergePolicy {
    fn default() -> Self {
        Self {
            max_loc_delta: 100,
            max_files: 1,
            block_on_owned_paths: true,
        }
    }
}

/// Observed PR shape used as input to [`assess_merge_safety`]. The caller
/// (CLI / dashboard / `gh pr diff` parser) populates these fields from
/// the gh JSON view.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PrShape {
    pub loc_added: u32,
    pub loc_removed: u32,
    pub files_changed: u32,
    pub touches_owned_paths: bool,
    /// True when the head ref's most recent commit is from a known
    /// machine-generator (Dependabot, Renovate, etc). Reversible bots
    /// have a separate exception ladder.
    pub is_machine_generated_reversible: bool,
}

/// Verdict shape — observable wire output of the guard.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MergeVerdict {
    /// Auto-merge is permitted under the policy.
    Allow,
    /// Auto-merge is blocked. `reason` is a canonical snake_case label
    /// fed into the SSB `block_reason_hash` field.
    Block { reason_label: &'static str },
}

/// Assess whether a PR is safe to auto-merge under `policy`. Emits a
/// `pr_auto_merge_blocked` row for any Block verdict. Pure aside from
/// the SSB write; the verdict itself is fully derived from inputs.
pub async fn assess_merge_safety(
    ssb: Arc<Ssb>,
    pr_number: u32,
    shape: &PrShape,
    policy: &MergePolicy,
) -> MergeVerdict {
    let verdict = classify(shape, policy);
    if let MergeVerdict::Block { reason_label } = &verdict {
        let payload = SsbPayload::PrAutoMergeBlocked(PrAutoMergeBlockedPayload {
            pr_number_hash: GhCliAdapter::hash_prefix(&pr_number.to_string()),
            block_reason_hash: GhCliAdapter::hash_prefix(reason_label),
            ts_ms: super::clock::now_ms(),
        });
        if let Err(e) = ssb.write_payload(GUARD_AGENT_ID, 0, payload, vec![]).await {
            warn!(
                target: "github_pr",
                error = %e,
                "ssb write failed for pr_auto_merge_blocked row; row dropped",
            );
        }
    }
    verdict
}

/// Pure classifier — exposed for unit tests. Block reasons are
/// canonical snake_case labels so the SSB `block_reason_hash` value is
/// stable across runs.
#[must_use]
pub fn classify(shape: &PrShape, policy: &MergePolicy) -> MergeVerdict {
    if shape.is_machine_generated_reversible
        && shape.loc_added.saturating_add(shape.loc_removed) <= policy.max_loc_delta
    {
        return MergeVerdict::Allow;
    }
    if policy.block_on_owned_paths && shape.touches_owned_paths {
        return MergeVerdict::Block {
            reason_label: "owned_paths",
        };
    }
    if shape.files_changed > policy.max_files {
        return MergeVerdict::Block {
            reason_label: "multi_file",
        };
    }
    let total = shape.loc_added.saturating_add(shape.loc_removed);
    if total > policy.max_loc_delta {
        return MergeVerdict::Block {
            reason_label: "loc_over_cap",
        };
    }
    MergeVerdict::Allow
}

/// One-shot branch-protection configurator. Calls `gh api PUT
/// /repos/{owner}/{repo}/branches/{branch}/protection` with the canonical
/// ELAI-recommended payload (required reviewers >= 1, linear history,
/// dismiss stale reviews, no force pushes, require status checks pass).
///
/// Returns `Ok(())` on gh exit 0. Does not parse the response body — the
/// caller can re-query gh for the applied configuration via a separate
/// `gh api` call.
pub async fn apply_branch_protection(
    repo: &str,
    branch: &str,
) -> Result<(), GithubPrError> {
    if repo.trim().is_empty() {
        return Err(GithubPrError::InvalidRequest {
            reason: "repo must be in owner/name form".to_string(),
        });
    }
    if branch.trim().is_empty() {
        return Err(GithubPrError::InvalidRequest {
            reason: "branch name must not be empty".to_string(),
        });
    }
    // We use multi --field flags so gh handles JSON encoding. The
    // ELAI-recommended payload mirrors the table at
    // https://docs.github.com/en/rest/branches/branch-protection.
    let path = format!("repos/{repo}/branches/{branch}/protection");
    let argv: Vec<&str> = vec![
        "api",
        "--method",
        "PUT",
        path.as_str(),
        "-f",
        "required_status_checks[strict]=true",
        "-f",
        "enforce_admins=true",
        "-f",
        "required_pull_request_reviews[required_approving_review_count]=1",
        "-f",
        "required_pull_request_reviews[dismiss_stale_reviews]=true",
        "-f",
        "required_linear_history=true",
        "-f",
        "allow_force_pushes=false",
        "-f",
        "restrictions=null",
    ];
    let output = Command::new(GH_BINARY)
        .args(&argv)
        .output()
        .await
        .map_err(GithubPrError::from)?;
    if !output.status.success() {
        let label = format!("gh {}", argv.join(" "));
        return Err(GithubPrError::from_exit(label, output.status, output.stderr));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn policy() -> MergePolicy {
        MergePolicy::default()
    }

    #[test]
    fn small_single_file_unowned_allows() {
        let shape = PrShape {
            loc_added: 30,
            loc_removed: 20,
            files_changed: 1,
            touches_owned_paths: false,
            is_machine_generated_reversible: false,
        };
        assert!(matches!(classify(&shape, &policy()), MergeVerdict::Allow));
    }

    #[test]
    fn loc_over_cap_blocks() {
        let shape = PrShape {
            loc_added: 80,
            loc_removed: 40,
            files_changed: 1,
            touches_owned_paths: false,
            is_machine_generated_reversible: false,
        };
        match classify(&shape, &policy()) {
            MergeVerdict::Block { reason_label } => assert_eq!(reason_label, "loc_over_cap"),
            other => panic!("expected Block, got {other:?}"),
        }
    }

    #[test]
    fn multi_file_blocks() {
        let shape = PrShape {
            loc_added: 10,
            loc_removed: 10,
            files_changed: 5,
            touches_owned_paths: false,
            is_machine_generated_reversible: false,
        };
        match classify(&shape, &policy()) {
            MergeVerdict::Block { reason_label } => assert_eq!(reason_label, "multi_file"),
            other => panic!("expected Block, got {other:?}"),
        }
    }

    #[test]
    fn owned_paths_block() {
        let shape = PrShape {
            loc_added: 10,
            loc_removed: 10,
            files_changed: 1,
            touches_owned_paths: true,
            is_machine_generated_reversible: false,
        };
        match classify(&shape, &policy()) {
            MergeVerdict::Block { reason_label } => assert_eq!(reason_label, "owned_paths"),
            other => panic!("expected Block, got {other:?}"),
        }
    }

    #[test]
    fn dependabot_under_cap_allows() {
        let shape = PrShape {
            loc_added: 5,
            loc_removed: 2,
            files_changed: 3, // would normally block on multi_file
            touches_owned_paths: false,
            is_machine_generated_reversible: true,
        };
        assert!(matches!(classify(&shape, &policy()), MergeVerdict::Allow));
    }
}
