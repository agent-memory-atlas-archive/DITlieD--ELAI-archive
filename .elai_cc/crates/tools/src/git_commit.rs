use std::process::Command;

use serde::{Deserialize, Serialize};

/// Maximum number of files that can be staged in a single commit.
const MAX_FILES_PER_COMMIT: usize = 50;

/// G-8 -- the digest-only provenance trailer key appended to a commit message
/// when the run sealed an in-toto attestation. It carries ONLY the blake3-Merkle
/// subject digest (the value `elai verify-attestation` recomputes + compares),
/// never any AI-attribution content. It is an ALLOWED trailer: its key does NOT
/// match the strip-filter's banned substrings (co-authored-by / generated-with /
/// noreply@anthropic), so it survives `format_commit_message`'s filter
/// (commit-cadence.md bans Co-Authored-By, NOT this provenance line).
const PROVENANCE_TRAILER_KEY: &str = "ELAI-Provenance-Digest";

/// G-8 -- the sidecar file name `emit_attestation` writes per session under
/// `<project>/.elai_cc/sessions/<sid>/`. The trailer reader picks the
/// most-recently-modified one (the run that just sealed an attestation).
const ATTESTATION_SIDECAR_NAME: &str = "attestation.intoto.json";

/// Patterns in file paths that indicate sensitive files.
const SENSITIVE_PATTERNS: &[&str] = &[
    ".env",
    "credentials",
    "secret",
    "private_key",
    "id_rsa",
    ".pem",
    "token.json",
];

#[derive(Debug, Deserialize)]
pub struct GitCommitInput {
    /// Files to stage. Must be explicit paths — never "." or "-A".
    pub files: Vec<String>,
    /// Commit message summary (1-2 sentences from the model).
    pub message: String,
    /// If true, append test pass/fail counts to the commit message body.
    #[serde(default)]
    pub include_test_results: Option<bool>,
}

#[derive(Debug, Serialize)]
pub struct GitCommitOutput {
    pub success: bool,
    pub commit_hash: String,
    pub files_committed: Vec<String>,
    pub summary: String,
}

/// Run a git command and return stdout on success or an error message.
fn git_cmd(args: &[&str]) -> Result<String, String> {
    let output = Command::new("git")
        .args(args)
        .output()
        .map_err(|e| format!("failed to run git {}: {e}", args.first().unwrap_or(&"")))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        let stdout = String::from_utf8_lossy(&output.stdout);
        return Err(format!(
            "git {} failed (exit {}): {}{}",
            args.first().unwrap_or(&""),
            output.status.code().unwrap_or(-1),
            stderr.trim(),
            if stdout.trim().is_empty() {
                String::new()
            } else {
                format!("\nstdout: {}", stdout.trim())
            }
        ));
    }

    Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
}

/// Validate the input before any git operations.
fn validate_input(input: &GitCommitInput) -> Result<(), String> {
    if input.files.is_empty() {
        return Err("files list is empty — specify at least one file to commit".to_string());
    }

    if input.files.len() > MAX_FILES_PER_COMMIT {
        return Err(format!(
            "too many files ({}) — maximum is {MAX_FILES_PER_COMMIT}",
            input.files.len()
        ));
    }

    for file in &input.files {
        let normalized = file.replace('\\', "/");

        // Block wildcard / blanket staging
        if normalized == "." || normalized == "-A" || normalized == "--all" || normalized == "-a" {
            return Err(format!(
                "rejected '{file}': blanket staging is not allowed — list files explicitly"
            ));
        }

        // Block sensitive file patterns
        let lower = normalized.to_ascii_lowercase();
        for pattern in SENSITIVE_PATTERNS {
            if lower.contains(pattern) {
                return Err(format!(
                    "rejected '{file}': path matches sensitive pattern '{pattern}' — \
                     review manually before committing"
                ));
            }
        }
    }

    if input.message.trim().is_empty() {
        return Err("commit message is empty".to_string());
    }

    Ok(())
}

/// Check that each listed file actually exists in git status as modified/untracked.
fn verify_files_in_status(files: &[String]) -> Result<String, String> {
    let status_output = git_cmd(&["status", "--porcelain"])?;

    let mut missing = Vec::new();
    for file in files {
        let normalized = file.replace('\\', "/");
        let found = status_output.lines().any(|line| {
            // porcelain format: XY <path> or XY <orig> -> <path>
            let path_part = if let Some(arrow_pos) = line.find(" -> ") {
                &line[arrow_pos + 4..]
            } else if line.len() > 3 {
                &line[3..]
            } else {
                ""
            };
            let norm_path = path_part.trim().replace('\\', "/");
            norm_path == normalized || norm_path.ends_with(&format!("/{normalized}"))
        });
        if !found {
            missing.push(file.clone());
        }
    }

    if !missing.is_empty() {
        return Err(format!(
            "files not found in git status (unmodified or not tracked): {}",
            missing.join(", ")
        ));
    }

    Ok(status_output)
}

/// Gather diff output for the listed files (both staged and unstaged).
fn gather_diff(files: &[String]) -> Result<String, String> {
    let file_refs: Vec<&str> = files.iter().map(String::as_str).collect();

    let mut diff_args = vec!["diff", "--"];
    diff_args.extend_from_slice(&file_refs);
    let unstaged = git_cmd(&diff_args).unwrap_or_default();

    let mut staged_args = vec!["diff", "--staged", "--"];
    staged_args.extend_from_slice(&file_refs);
    let staged = git_cmd(&staged_args).unwrap_or_default();

    let mut combined = String::new();
    if !staged.is_empty() {
        combined.push_str("=== Staged ===\n");
        combined.push_str(&staged);
    }
    if !unstaged.is_empty() {
        if !combined.is_empty() {
            combined.push('\n');
        }
        combined.push_str("=== Unstaged ===\n");
        combined.push_str(&unstaged);
    }

    Ok(combined)
}

/// Get recent commit messages for style reference.
fn recent_commits() -> Result<String, String> {
    git_cmd(&["log", "--oneline", "-5"])
}

/// Stage only the specified files.
fn stage_files(files: &[String]) -> Result<(), String> {
    let file_refs: Vec<&str> = files.iter().map(String::as_str).collect();
    let mut args = vec!["add"];
    args.extend_from_slice(&file_refs);
    git_cmd(&args)?;
    Ok(())
}

/// Format the final commit message.
/// NEVER adds Co-Authored-By or any AI attribution trailer.
fn format_commit_message(
    summary: &str,
    include_test_results: bool,
) -> Result<String, String> {
    let mut msg = summary.trim().to_string();

    if include_test_results {
        // Run cargo test in check mode to get pass/fail counts
        let test_output = Command::new("cargo")
            .args(["test", "--workspace", "--no-fail-fast", "--", "--format=terse"])
            .output();

        match test_output {
            Ok(output) => {
                let stdout = String::from_utf8_lossy(&output.stdout);
                let stderr = String::from_utf8_lossy(&output.stderr);

                // Parse test summary line: "test result: ok. X passed; Y failed; Z ignored"
                let summary_line = stdout
                    .lines()
                    .chain(stderr.lines())
                    .find(|line| line.starts_with("test result:"))
                    .unwrap_or("test result: unavailable");

                msg.push_str("\n\n");
                msg.push_str(summary_line);
            }
            Err(e) => {
                msg.push_str(&format!("\n\ntest results: cargo test failed to run ({e})"));
            }
        }
    }

    // Safety: strip any Co-Authored-By or AI attribution that might have leaked in
    let filtered: Vec<&str> = msg
        .lines()
        .filter(|line| {
            let lower = line.to_ascii_lowercase();
            !lower.contains("co-authored-by")
                && !lower.contains("generated with")
                && !lower.contains("noreply@anthropic")
        })
        .collect();

    let mut out = filtered.join("\n").trim().to_string();

    // G-8: append the digest-only provenance trailer (the git-binding leg, DD4c)
    // AFTER the strip-filter, so it is appended to the cleaned message and the
    // filter can never drop it. The trailer is added ONLY when the run sealed an
    // attestation sidecar (best-effort: a missing sidecar simply omits the
    // trailer — the SSB row + sidecar are the PRIMARY emit, the trailer is the
    // optional git-binding when a commit is made). The digest is recomputed +
    // compared offline by `elai verify-attestation`.
    if let Some(digest) = recent_attestation_digest() {
        out.push_str(&format!("\n\n{PROVENANCE_TRAILER_KEY}: {digest}"));
    }

    Ok(out)
}

/// G-8 -- read the blake3-Merkle subject digest from the most-recently-modified
/// in-toto attestation sidecar under `<cwd>/.elai_cc/sessions/*/`. Returns `None`
/// when no sidecar exists (no run sealed an attestation), when the directory is
/// unreadable, or when the newest sidecar cannot be parsed — the trailer is then
/// simply omitted (the SSB row + sidecar are the PRIMARY emit). This keeps
/// `git_commit` DECOUPLED from the run path (DD4c seam): it reads the run's
/// just-written attestation digest rather than recomputing it at commit time.
fn recent_attestation_digest() -> Option<String> {
    let sessions_dir = std::path::Path::new(elai_core::paths::ELAI_DATA_DIR_NAME).join("sessions");
    let mut newest: Option<(std::time::SystemTime, std::path::PathBuf)> = None;
    for entry in std::fs::read_dir(&sessions_dir).ok()?.flatten() {
        let sidecar = entry.path().join(ATTESTATION_SIDECAR_NAME);
        let Ok(meta) = std::fs::metadata(&sidecar) else {
            continue;
        };
        let Ok(mtime) = meta.modified() else {
            continue;
        };
        let replace = match &newest {
            Some((best, _)) => mtime > *best,
            None => true,
        };
        if replace {
            newest = Some((mtime, sidecar));
        }
    }
    let (_, path) = newest?;
    let statement = elai_orchestrator::attestation::read_sidecar(&path).ok()?;
    statement.subject_digest().map(str::to_owned)
}

/// Create the commit.
fn create_commit(message: &str) -> Result<String, String> {
    git_cmd(&["commit", "-m", message])
}

/// Get the hash of the latest commit.
fn latest_commit_hash() -> Result<String, String> {
    git_cmd(&["rev-parse", "--short", "HEAD"])
}

/// Verify the working tree is clean for the committed files.
fn verify_commit_success(files: &[String]) -> Result<(), String> {
    let status = git_cmd(&["status", "--porcelain"])?;
    let still_dirty: Vec<&String> = files
        .iter()
        .filter(|f| {
            let normalized = f.replace('\\', "/");
            status.lines().any(|line| {
                let path_part = if line.len() > 3 { &line[3..] } else { "" };
                let norm_path = path_part.trim().replace('\\', "/");
                norm_path == normalized
            })
        })
        .collect();

    if !still_dirty.is_empty() {
        return Err(format!(
            "commit may have failed — files still dirty: {}",
            still_dirty.iter().map(|s| s.as_str()).collect::<Vec<_>>().join(", ")
        ));
    }

    Ok(())
}

/// Execute the full git commit workflow.
///
/// Steps:
/// 1. Validate input (no blanket staging, no sensitive files)
/// 2. Verify files appear in git status
/// 3. Gather diffs for context
/// 4. Check recent commits for style
/// 5. Stage only the listed files
/// 6. Format commit message (with optional test results, never AI attribution)
/// 7. Create commit
/// 8. Verify success and return result
pub fn run_git_commit(input: GitCommitInput) -> Result<String, String> {
    // Step 1: validate
    validate_input(&input)?;

    // Step 2: verify files exist in git status
    let _status = verify_files_in_status(&input.files)?;

    // Step 3: gather diffs (informational — included in output for audit trail)
    let diff_summary = gather_diff(&input.files).unwrap_or_else(|_| "(no diff available)".into());

    // Step 4: recent commits (informational)
    let recent = recent_commits().unwrap_or_else(|_| "(no recent commits)".into());

    // Step 5: stage files
    stage_files(&input.files)?;

    // Step 6: format commit message
    let include_tests = input.include_test_results.unwrap_or(false);
    let final_message = format_commit_message(&input.message, include_tests)?;

    // Step 7: create commit
    let commit_output = create_commit(&final_message)?;

    // Step 8: verify and get hash
    verify_commit_success(&input.files)?;
    let commit_hash = latest_commit_hash().unwrap_or_else(|_| "unknown".into());

    let output = GitCommitOutput {
        success: true,
        commit_hash,
        files_committed: input.files,
        summary: format!(
            "{commit_output}\n\nRecent commits:\n{recent}\n\nDiff summary length: {} bytes",
            diff_summary.len()
        ),
    };

    serde_json::to_string_pretty(&output).map_err(|e| e.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn validate_rejects_empty_files() {
        let input = GitCommitInput {
            files: vec![],
            message: "test commit".to_string(),
            include_test_results: None,
        };
        let err = validate_input(&input).unwrap_err();
        assert!(err.contains("empty"), "expected empty error, got: {err}");
    }

    #[test]
    fn validate_rejects_dot_staging() {
        let input = GitCommitInput {
            files: vec![".".to_string()],
            message: "test commit".to_string(),
            include_test_results: None,
        };
        let err = validate_input(&input).unwrap_err();
        assert!(
            err.contains("blanket staging"),
            "expected blanket staging error, got: {err}"
        );
    }

    #[test]
    fn validate_rejects_dash_a() {
        let input = GitCommitInput {
            files: vec!["-A".to_string()],
            message: "test commit".to_string(),
            include_test_results: None,
        };
        let err = validate_input(&input).unwrap_err();
        assert!(err.contains("blanket staging"), "got: {err}");
    }

    #[test]
    fn validate_rejects_sensitive_files() {
        let input = GitCommitInput {
            files: vec![".env.production".to_string()],
            message: "test commit".to_string(),
            include_test_results: None,
        };
        let err = validate_input(&input).unwrap_err();
        assert!(err.contains("sensitive"), "got: {err}");
    }

    #[test]
    fn validate_rejects_credentials_file() {
        let input = GitCommitInput {
            files: vec!["config/credentials.json".to_string()],
            message: "test commit".to_string(),
            include_test_results: None,
        };
        let err = validate_input(&input).unwrap_err();
        assert!(err.contains("sensitive"), "got: {err}");
    }

    #[test]
    fn validate_rejects_empty_message() {
        let input = GitCommitInput {
            files: vec!["src/main.rs".to_string()],
            message: "   ".to_string(),
            include_test_results: None,
        };
        let err = validate_input(&input).unwrap_err();
        assert!(err.contains("empty"), "got: {err}");
    }

    #[test]
    fn validate_accepts_normal_input() {
        let input = GitCommitInput {
            files: vec!["src/lib.rs".to_string(), "tests/test_foo.rs".to_string()],
            message: "A-28: add git_commit tool".to_string(),
            include_test_results: Some(false),
        };
        assert!(validate_input(&input).is_ok());
    }

    #[test]
    fn validate_rejects_too_many_files() {
        let files: Vec<String> = (0..=MAX_FILES_PER_COMMIT)
            .map(|i| format!("file_{i}.rs"))
            .collect();
        let input = GitCommitInput {
            files,
            message: "too many".to_string(),
            include_test_results: None,
        };
        let err = validate_input(&input).unwrap_err();
        assert!(err.contains("too many"), "got: {err}");
    }

    #[test]
    fn format_message_strips_co_authored_by() {
        let msg = format_commit_message(
            "A-28: test\n\nCo-Authored-By: Bot <bot@example.com>",
            false,
        )
        .unwrap();
        assert!(
            !msg.to_ascii_lowercase().contains("co-authored-by"),
            "AI attribution not stripped: {msg}"
        );
    }

    #[test]
    fn format_message_strips_generated_with() {
        let msg = format_commit_message(
            "fix: thing\n\nGenerated with Claude Code",
            false,
        )
        .unwrap();
        assert!(
            !msg.to_ascii_lowercase().contains("generated with"),
            "generated-with not stripped: {msg}"
        );
    }

    #[test]
    fn format_message_preserves_normal_content() {
        let msg = format_commit_message("A-28: git_commit tool implementation", false).unwrap();
        assert_eq!(msg, "A-28: git_commit tool implementation");
    }
}
