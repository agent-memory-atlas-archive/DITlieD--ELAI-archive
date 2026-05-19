// C-13 — WIP commit squash helper.
//
// Detects consecutive leading commits whose first message line matches
// `/^[Ww][Ii][Pp]/` (the conventional "WIP:" / "wip" prefix used by the
// agent's intra-step bookkeeping) and, when the count crosses a 5-commit
// threshold, squashes them into a single commit via:
//
//     git reset --soft HEAD~{N}
//     git commit --no-verify -m "phase: squash {N} WIP commits"
//
// The trigger surface is the post-AUDITING -> MEMORIZING boundary; the Rust
// task_runner audit-loop and the `elai wip-squash` CLI subcommand both call
// [`squash_wip_commits_if_any`]. The Python hook chain shells into the CLI
// surface so the same code path runs irrespective of which side fires the
// FSM transition.
//
// Plan reference: C-13 task 6 / DECISION D4 (option A: Rust-side helper +
// CLI shim) — see workflow/reasoning/C-13-decisions.md.
//
// Hard rules honoured:
//   * No `.unwrap()` / `.expect()` / `panic!` on any production path.
//   * No `libgit2` / `gitoxide` — uses `std::process::Command` so the
//     working tree's `.gitconfig` / hooks behave identically to a human
//     operator's `git` invocation.
//   * Captures stderr; maps non-zero exit codes into `anyhow::Error` so the
//     caller can decide whether to abort the FSM transition or proceed.
//   * Emits exactly one `SsbActionType::WipSquashComplete` row per call,
//     regardless of the no-op vs. squashed outcome, so the dashboard's
//     activity feed surfaces both branches.

use std::path::Path;
use std::process::Command;
use std::sync::Arc;

use crate::ssb::{Ssb, SsbPayload, WipSquashCompletePayload};

/// Minimum count of leading WIP commits that triggers a real squash.
/// 5 or fewer is treated as a no-op (the agent often emits a few WIP
/// commits per baby-step and squashing every one of those would obscure
/// the per-step boundary in the git log).
pub const WIP_SQUASH_THRESHOLD: u32 = 5;

/// Agent ID used when emitting the SSB row from this helper.
const WIP_SQUASH_AGENT_ID: &str = "wip-squash";

/// Outcome of [`squash_wip_commits_if_any`]. Callers inspect `squashed` to
/// decide whether to log the post-transition state. `wip_count` is the
/// number of leading WIP commits OBSERVED, not the number squashed (the
/// counts coincide on success; on a no-op `squashed` is false and
/// `wip_count` is the count below the threshold).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WipSquashOutcome {
    /// Count of leading commits whose first message line matched the WIP
    /// regex. `0` when HEAD has no commits or the most recent commit is
    /// not a WIP commit.
    pub wip_count: u32,
    /// `true` when a `git reset --soft` + `git commit --no-verify` actually
    /// fired; `false` for the no-op branch.
    pub squashed: bool,
}

/// Run the squash heuristic against the working tree rooted at
/// `project_root`. Always emits exactly one `WipSquashComplete` SSB row
/// before returning so the dashboard can render both branches.
///
/// `step_id` is the baby-step identifier that triggered the call; empty
/// string is acceptable (used by the CLI surface where no step boundary
/// is in flight).
///
/// Errors:
///   * `project_root` is not a git working tree (any `git` subprocess
///     returns a non-zero exit code).
///   * `git log` / `git reset` / `git commit` fail for any other reason
///     (captured stderr is included in the returned `anyhow::Error`).
///
/// On error the function does NOT emit an SSB row (a failure to count
/// commits is not a "squash complete" event; the caller should surface
/// the error via tracing).
pub async fn squash_wip_commits_if_any(
    project_root: &Path,
    ssb: Arc<Ssb>,
    step_id: &str,
) -> Result<WipSquashOutcome, anyhow::Error> {
    let wip_count = count_leading_wip_commits(project_root)?;

    let squashed = if wip_count > WIP_SQUASH_THRESHOLD {
        squash_n_commits(project_root, wip_count)?;
        true
    } else {
        false
    };

    let payload = SsbPayload::WipSquashComplete(WipSquashCompletePayload {
        wip_count,
        squashed,
        step_id: step_id.to_owned(),
    });
    if let Err(e) = ssb
        .write_payload(WIP_SQUASH_AGENT_ID, 0, payload, Vec::new())
        .await
    {
        // SSB write failure is best-effort: log + continue. The squash
        // itself already succeeded (or was a no-op) by this point.
        tracing::warn!(
            error = %e,
            wip_count,
            squashed,
            "wip_squash: SSB write failed; squash itself succeeded"
        );
    }

    Ok(WipSquashOutcome { wip_count, squashed })
}

/// Count consecutive leading commits whose first message line matches
/// the WIP regex (case-insensitive `wip` prefix). Implementation runs
/// `git log --pretty=%s` (subjects only, one per line) and inspects the
/// leading entries.
///
/// Returns 0 when:
///   * HEAD has no commits (fresh repo).
///   * The most recent commit is not a WIP commit.
fn count_leading_wip_commits(project_root: &Path) -> Result<u32, anyhow::Error> {
    let output = Command::new("git")
        .arg("-C")
        .arg(project_root)
        .arg("log")
        .arg("--pretty=%s")
        .arg("-100") // bound the scan; agent never accrues >100 leading WIPs
        .output()
        .map_err(|e| {
            anyhow::anyhow!(
                "wip_squash: failed to spawn `git log` at {}: {e}",
                project_root.display()
            )
        })?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr).into_owned();
        let code = output.status.code().unwrap_or(-1);
        return Err(anyhow::anyhow!(
            "wip_squash: `git log` exited with code {code} at {}: {stderr}",
            project_root.display()
        ));
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let mut count: u32 = 0;
    for line in stdout.lines() {
        if is_wip_subject(line) {
            count = count.saturating_add(1);
        } else {
            break;
        }
    }
    Ok(count)
}

/// True when the first non-whitespace token of `subject` is "wip" (any
/// case) optionally followed by a colon, dash, or whitespace.
fn is_wip_subject(subject: &str) -> bool {
    let trimmed = subject.trim_start();
    if trimmed.len() < 3 {
        return false;
    }
    let prefix: String = trimmed.chars().take(3).flat_map(char::to_lowercase).collect();
    if prefix != "wip" {
        return false;
    }
    // After the "wip" prefix the next char (if any) must be a delimiter or
    // EOL; this prevents false positives on commit messages that start
    // with "wipe" / "wiper" / etc.
    match trimmed.chars().nth(3) {
        None => true,
        Some(c) => matches!(c, ':' | '-' | ' ' | '\t'),
    }
}

/// Run `git reset --soft HEAD~N` then `git commit --no-verify -m "phase:
/// squash N WIP commits"`. Both subprocesses capture stderr; any non-zero
/// exit aborts with an `anyhow::Error`.
fn squash_n_commits(project_root: &Path, n: u32) -> Result<(), anyhow::Error> {
    // 1. git reset --soft HEAD~N
    let reset = Command::new("git")
        .arg("-C")
        .arg(project_root)
        .arg("reset")
        .arg("--soft")
        .arg(format!("HEAD~{n}"))
        .output()
        .map_err(|e| {
            anyhow::anyhow!(
                "wip_squash: failed to spawn `git reset --soft HEAD~{n}` at {}: {e}",
                project_root.display()
            )
        })?;
    if !reset.status.success() {
        let stderr = String::from_utf8_lossy(&reset.stderr).into_owned();
        let code = reset.status.code().unwrap_or(-1);
        return Err(anyhow::anyhow!(
            "wip_squash: `git reset --soft HEAD~{n}` exited with code {code}: {stderr}"
        ));
    }

    // 2. git commit --no-verify -m "phase: squash N WIP commits"
    //    --no-verify is intentional: this commit fires from inside the FSM
    //    hook chain; running the pre-commit hooks again here would
    //    re-trigger the very protocol that just produced the squash.
    let message = format!("phase: squash {n} WIP commits");
    let commit = Command::new("git")
        .arg("-C")
        .arg(project_root)
        .arg("commit")
        .arg("--no-verify")
        .arg("-m")
        .arg(&message)
        .output()
        .map_err(|e| {
            anyhow::anyhow!(
                "wip_squash: failed to spawn `git commit --no-verify` at {}: {e}",
                project_root.display()
            )
        })?;
    if !commit.status.success() {
        let stderr = String::from_utf8_lossy(&commit.stderr).into_owned();
        let code = commit.status.code().unwrap_or(-1);
        return Err(anyhow::anyhow!(
            "wip_squash: `git commit --no-verify` exited with code {code}: {stderr}"
        ));
    }

    Ok(())
}

/// Public-facing hook the task_runner calls AFTER the AUDITING -> MEMORIZING
/// transition succeeds. Distinct entry point so the call site reads as
/// `post_transition_auditing_to_memorizing(...)` at the FSM boundary
/// instead of the generic `squash_wip_commits_if_any`. Implementation is a
/// thin wrapper that forwards to [`squash_wip_commits_if_any`].
///
/// Returns `Ok(WipSquashOutcome)` even when the squash itself was a no-op;
/// only IO / subprocess failures bubble up as `Err`.
pub async fn post_transition_auditing_to_memorizing(
    project_root: &Path,
    ssb: Arc<Ssb>,
    step_id: &str,
) -> Result<WipSquashOutcome, anyhow::Error> {
    squash_wip_commits_if_any(project_root, ssb, step_id).await
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn is_wip_subject_matches_canonical_prefixes() {
        assert!(is_wip_subject("WIP: foo"));
        assert!(is_wip_subject("wip: bar"));
        assert!(is_wip_subject("Wip - baz"));
        assert!(is_wip_subject("WIP qux"));
        assert!(is_wip_subject("wip"));
    }

    #[test]
    fn is_wip_subject_rejects_substring_matches() {
        assert!(!is_wip_subject("wiper"));
        assert!(!is_wip_subject("wipe out"));
        assert!(!is_wip_subject("feat: wip thing"));
        assert!(!is_wip_subject(""));
    }
}
