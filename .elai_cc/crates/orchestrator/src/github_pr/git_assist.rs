//! E-41: git-side helpers for PR review workflows.
//!
//! Two pure-helper surfaces:
//!
//! - [`suggest_squash_message`]: composes a maintainer-voice commit subject
//!   from a PR title + commit log. No git mutation — caller decides
//!   whether to apply via `git commit --amend -m <message>` or feed back
//!   to `gh pr edit --title`.
//! - [`detect_force_push`]: queries `gh api repos/{owner}/{repo}/pulls/
//!   {pr_number}/events` for a `head_ref_force_pushed` event signature.
//!   Returns `true` when at least one such event is present.
//!
//! Both helpers are sync-callable via the `pr review` CLI subverbs;
//! [`detect_force_push`] is async because it shells gh. We deliberately
//! do NOT spawn the rewrite — git mutations are reserved for the user
//! per commit-cadence.md HARD RULE ("no automatic squash/amend in
//! ELAI"), and the E-41 surface is advisory.

use std::sync::Arc;

use tokio::process::Command;
use tracing::warn;

use crate::ssb::{PrReviewComposedPayload, Ssb, SsbPayload};

use super::error::GithubPrError;
use super::gh_cli::GhCliAdapter;

const GIT_ASSIST_AGENT_ID: &str = "github-pr-git-assist";
const GH_BINARY: &str = "gh";

/// Compose a maintainer-voice squash message for the supplied PR. The output
/// is a multi-line string the caller can hand to git or to `gh pr edit
/// --title`. Pure function — no I/O.
///
/// Conventions applied (from `.claude/rules/commit-cadence.md`):
/// - lowercase first character on the subject line
/// - no em-dashes (rewrites `—` -> `,`)
/// - no robotic prefix (strips `[Phase N]`, `feat:`, `fix:`, `chore:`)
/// - no Co-Authored-By / Generated-By in the body
/// - max 72 chars on the subject line
#[must_use]
pub fn suggest_squash_message(pr_title: &str, body_hint: &str) -> String {
    let mut subject = pr_title.trim().to_string();
    // Strip robotic prefixes.
    for prefix in [
        "[Phase",
        "Phase ",
        "feat:",
        "fix:",
        "chore:",
        "refactor:",
        "docs:",
        "test:",
        "build:",
        "ci:",
    ] {
        if subject.to_ascii_lowercase().starts_with(&prefix.to_ascii_lowercase()) {
            // Drop the prefix up to the first space or ']'.
            let cut = subject
                .find(|c: char| c == ']' || c == ' ')
                .map(|i| i + 1)
                .unwrap_or(0);
            subject = subject[cut..].trim().to_string();
        }
    }
    // Lowercase first character.
    if let Some(first) = subject.chars().next() {
        let lowered = first.to_lowercase().collect::<String>();
        let rest: String = subject.chars().skip(1).collect();
        subject = format!("{lowered}{rest}");
    }
    // Replace em-dashes with commas.
    subject = subject.replace('\u{2014}', ",");
    // Cap subject length to 72 chars.
    if subject.chars().count() > 72 {
        subject = subject.chars().take(72).collect();
    }
    // Body strips banned trailers if the caller accidentally included
    // them. We do NOT auto-append any trailer — the user controls that.
    let mut body_lines: Vec<&str> = Vec::new();
    for line in body_hint.lines() {
        let lower = line.to_ascii_lowercase();
        if lower.starts_with("co-authored-by:")
            || lower.starts_with("generated-by:")
            || lower.starts_with("reviewed-by-bot:")
        {
            continue;
        }
        body_lines.push(line);
    }
    let body = body_lines.join("\n").trim().to_string();
    if body.is_empty() {
        subject
    } else {
        format!("{subject}\n\n{body}")
    }
}

/// Query gh for the PR's event log and return `true` when at least one
/// `head_ref_force_pushed` event is present. Emits one
/// `pr_review_composed` summary SSB row tagged with kind `force_push_check`
/// so the dashboard can render the force-push indicator without a
/// separate variant.
///
/// # Errors
/// Standard `GithubPrError` shape.
pub async fn detect_force_push(
    ssb: Arc<Ssb>,
    pr_number: u32,
    repo: Option<&str>,
) -> Result<bool, GithubPrError> {
    // gh api path: `/repos/{owner}/{repo}/issues/{pr_number}/events`
    // when we have a `-R owner/repo` flag we can let gh fill the repo;
    // otherwise rely on gh's git remote inference.
    let path = format!("repos/{{owner}}/{{repo}}/issues/{pr_number}/events");
    let mut argv: Vec<String> = Vec::new();
    if let Some(r) = repo {
        argv.push("-R".to_string());
        argv.push(r.to_string());
    }
    argv.push("api".to_string());
    argv.push(path);
    let argv_refs: Vec<&str> = argv.iter().map(String::as_str).collect();
    let output = Command::new(GH_BINARY)
        .args(&argv_refs)
        .output()
        .await
        .map_err(GithubPrError::from)?;
    if !output.status.success() {
        let label = format!("gh {}", argv_refs.join(" "));
        return Err(GithubPrError::from_exit(label, output.status, output.stderr));
    }
    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    // Cheap, stable detection: look for the canonical event name. gh's
    // schema may evolve; the substring check is robust to field-order
    // changes that would break a full deserialize.
    let found = stdout.contains("head_ref_force_pushed");

    let payload = SsbPayload::PrReviewComposed(PrReviewComposedPayload {
        pr_number_hash: GhCliAdapter::hash_prefix(&pr_number.to_string()),
        comment_kind_hash: GhCliAdapter::hash_prefix("force_push_check"),
        ts_ms: super::clock::now_ms(),
    });
    if let Err(e) = ssb.write_payload(GIT_ASSIST_AGENT_ID, 0, payload, vec![]).await {
        warn!(
            target: "github_pr",
            error = %e,
            "ssb write failed for force-push check row; row dropped",
        );
    }
    Ok(found)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn strips_robotic_prefix_and_lowercases() {
        let out = suggest_squash_message("[Phase 4] Add Foo Bar feature", "");
        assert!(out.starts_with("add Foo Bar feature"));
    }

    #[test]
    fn replaces_em_dash() {
        let out = suggest_squash_message("Refactor \u{2014} cleanup imports", "");
        assert!(out.contains(','));
        assert!(!out.contains('\u{2014}'));
    }

    #[test]
    fn strips_banned_trailers_from_body() {
        let out = suggest_squash_message(
            "fix: tighten validation",
            "tighten the path validator.\n\nCo-Authored-By: Bot <bot@example.com>\nGenerated-By: elai/executor",
        );
        assert!(!out.to_ascii_lowercase().contains("co-authored-by"));
        assert!(!out.to_ascii_lowercase().contains("generated-by:"));
        assert!(out.contains("tighten the path validator"));
    }

    #[test]
    fn caps_subject_to_72_chars() {
        let long = "a".repeat(200);
        let out = suggest_squash_message(&long, "");
        let subject = out.lines().next().unwrap_or_default();
        assert!(subject.chars().count() <= 72);
    }
}
