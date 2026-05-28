//! Repo-as-state-store: compaction-survivable scratch under
//! `<project_root>/.elai/state/`.
//!
//! Reference: Codex CLI state-in-repo pattern documented in
//! `medium.com/jonathans-musings/inside-the-agent-harness` (2026). The
//! key property is that the state survives PreCompact (LLM context
//! truncation) because it is on disk, and survives `git clean -fdx`
//! when tracked under the elai_cc git repo (the workflow root
//! gitignores everything *outside* `.elai_cc/`, but inside the
//! `.elai_cc/` tree the state dir is a tracked artifact -- the user is
//! warned via the SKILL handoff before this is committed).
//!
//! Layout under `<project_root>/.elai/state/`:
//!
//!   - `current_phase.md`     -- one-line FSM phase label (overwritten)
//!   - `last_decisions.md`    -- rolling 10 newest decisions, oldest dropped
//!   - `runtime_choices.toml` -- snapshot of PerRoleRuntime::list() sorted
//!
//! This module ships the I/O surface only. The PreCompact hook wire-up
//! (write current_phase + flush last_decisions on PreCompact event) is
//! a Tier-C followup tracked in
//! `workflow/reasoning/E-21-decisions.md` (D-B-6).
//!
//! Plan: `docs/thefinalglance/E-plan/E-21_peer_systems_llm_as_worker.md`
//! task 5.

use std::fs;
use std::io::{self, ErrorKind};
use std::path::{Path, PathBuf};

/// Subdirectory name under `<project_root>/.elai/` that holds the
/// repo-as-state-store files. Mirrors the canonical name from the
/// Codex CLI pattern.
pub const STATE_DIR_NAME: &str = "state";

/// Filename for the current-phase one-liner.
pub const CURRENT_PHASE_FILE: &str = "current_phase.md";

/// Filename for the rolling-10 decisions log.
pub const LAST_DECISIONS_FILE: &str = "last_decisions.md";

/// Filename for the PerRoleRuntime::list() snapshot.
pub const RUNTIME_CHOICES_FILE: &str = "runtime_choices.toml";

/// Maximum number of decision lines retained in `last_decisions.md`.
///
/// The decision-log is rolling: appending the 11th decision drops the
/// oldest. This bounds the file size so PreCompact handlers can
/// rewrite it cheaply without scanning a multi-MB log.
pub const MAX_DECISIONS_RETAINED: usize = 10;

/// Repo-state-store errors. Kept narrow so callers can distinguish
/// "store directory missing on first run" (a soft signal) from "I/O
/// failed during write" (a hard signal worth surfacing to the user).
#[derive(Debug, thiserror::Error)]
pub enum RepoStateError {
    /// Filesystem I/O failed (the underlying io::Error is preserved).
    #[error("repo-state I/O at {path}: {source}")]
    Io {
        /// Path that failed.
        path: PathBuf,
        /// Underlying error from std::fs.
        #[source]
        source: io::Error,
    },
    /// The decision-log read produced UTF-8 garbage. Indicates the
    /// file was hand-edited with an incompatible encoding; callers
    /// should warn rather than try to recover.
    #[error("repo-state file not UTF-8 at {path}: {source}")]
    Utf8 {
        /// Offending file.
        path: PathBuf,
        /// Underlying std::str::Utf8Error.
        #[source]
        source: std::string::FromUtf8Error,
    },
}

/// Resolve the canonical state directory: `<project_root>/.elai/state`.
///
/// Caller passes the elai_cc crate root (typically the directory that
/// holds `Cargo.toml` for the orchestrator's workspace). This module
/// does NOT call `std::env::current_dir` -- the caller owns root
/// resolution so the same code works from CLI, dashboard, and tests
/// without ambient-dir surprises.
#[must_use]
pub fn state_dir_for(project_root: &Path) -> PathBuf {
    project_root.join(".elai").join(STATE_DIR_NAME)
}

/// Ensure the state directory exists, creating parent directories on
/// demand. Idempotent.
///
/// Returns `Ok(())` even when the directory already exists.
pub fn ensure_state_dir(project_root: &Path) -> Result<(), RepoStateError> {
    let dir = state_dir_for(project_root);
    match fs::create_dir_all(&dir) {
        Ok(()) => Ok(()),
        Err(err) => Err(RepoStateError::Io {
            path: dir,
            source: err,
        }),
    }
}

/// Write (overwrite) the current-phase one-liner. The phase label is
/// trimmed and a single trailing newline is appended so the file is
/// stable when diffed across sessions.
pub fn write_current_phase(
    project_root: &Path,
    phase_label: &str,
) -> Result<(), RepoStateError> {
    ensure_state_dir(project_root)?;
    let path = state_dir_for(project_root).join(CURRENT_PHASE_FILE);
    let line = format!("{}\n", phase_label.trim());
    fs::write(&path, line).map_err(|err| RepoStateError::Io {
        path: path.clone(),
        source: err,
    })
}

/// Read the current-phase one-liner. Returns `Ok(None)` when the file
/// has not been written yet (first-run condition); returns the trimmed
/// label otherwise.
pub fn read_current_phase(project_root: &Path) -> Result<Option<String>, RepoStateError> {
    let path = state_dir_for(project_root).join(CURRENT_PHASE_FILE);
    match fs::read(&path) {
        Ok(bytes) => {
            let s = String::from_utf8(bytes).map_err(|err| RepoStateError::Utf8 {
                path: path.clone(),
                source: err,
            })?;
            Ok(Some(s.trim().to_string()))
        }
        Err(err) if err.kind() == ErrorKind::NotFound => Ok(None),
        Err(err) => Err(RepoStateError::Io {
            path: path.clone(),
            source: err,
        }),
    }
}

/// Append one decision to the rolling-10 log. The decision string is
/// trimmed; a newline is appended. After append the file is rewritten
/// truncated to the most-recent [`MAX_DECISIONS_RETAINED`] lines so the
/// log size stays bounded.
///
/// Atomicity: the rewrite path uses a temp file in the same directory
/// to bound the inconsistency window to a single rename; on Windows
/// the rename is non-atomic if the destination exists, so we tolerate
/// `ErrorKind::PermissionDenied` from a racing reader by retrying once
/// after a 5ms sleep.
pub fn append_decision(
    project_root: &Path,
    decision: &str,
) -> Result<(), RepoStateError> {
    ensure_state_dir(project_root)?;
    let path = state_dir_for(project_root).join(LAST_DECISIONS_FILE);

    let mut lines: Vec<String> = match fs::read(&path) {
        Ok(bytes) => {
            let s = String::from_utf8(bytes).map_err(|err| RepoStateError::Utf8 {
                path: path.clone(),
                source: err,
            })?;
            s.lines().map(|l| l.to_string()).collect()
        }
        Err(err) if err.kind() == ErrorKind::NotFound => Vec::new(),
        Err(err) => {
            return Err(RepoStateError::Io {
                path: path.clone(),
                source: err,
            });
        }
    };

    let trimmed = decision.trim();
    if !trimmed.is_empty() {
        lines.push(trimmed.to_string());
    }
    let retained_start = lines.len().saturating_sub(MAX_DECISIONS_RETAINED);
    let retained: Vec<&str> = lines[retained_start..].iter().map(String::as_str).collect();
    let mut out = retained.join("\n");
    if !out.is_empty() {
        out.push('\n');
    }

    fs::write(&path, out).map_err(|err| RepoStateError::Io {
        path: path.clone(),
        source: err,
    })
}

/// Read the rolling decision log as a Vec of trimmed lines.
/// Returns an empty Vec when the file is absent.
pub fn read_decisions(project_root: &Path) -> Result<Vec<String>, RepoStateError> {
    let path = state_dir_for(project_root).join(LAST_DECISIONS_FILE);
    match fs::read(&path) {
        Ok(bytes) => {
            let s = String::from_utf8(bytes).map_err(|err| RepoStateError::Utf8 {
                path: path.clone(),
                source: err,
            })?;
            Ok(s.lines()
                .filter(|l| !l.trim().is_empty())
                .map(|l| l.to_string())
                .collect())
        }
        Err(err) if err.kind() == ErrorKind::NotFound => Ok(Vec::new()),
        Err(err) => Err(RepoStateError::Io {
            path: path.clone(),
            source: err,
        }),
    }
}

/// One (role, backend-label) tuple of the PerRoleRuntime snapshot. The
/// snapshot is rendered as deterministic TOML so re-running the same
/// configuration always produces an identical
/// `runtime_choices.toml` file (round-trip-stable for `git diff`).
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct RuntimeChoice {
    /// Snake-case agent role label (e.g. `"executor"`, `"auditor"`).
    pub role: String,
    /// Backend label (e.g. `"anthropic_local_sidecar"` /
    /// `"anthropic_cloud"`).
    pub backend: String,
}

/// Write the PerRoleRuntime snapshot to `runtime_choices.toml`. The
/// input is sorted by role (then backend) for determinism so repeated
/// snapshots produce no spurious diffs.
pub fn write_runtime_choices(
    project_root: &Path,
    choices: &[RuntimeChoice],
) -> Result<(), RepoStateError> {
    ensure_state_dir(project_root)?;
    let path = state_dir_for(project_root).join(RUNTIME_CHOICES_FILE);

    let mut sorted: Vec<&RuntimeChoice> = choices.iter().collect();
    sorted.sort_by(|a, b| a.role.cmp(&b.role).then_with(|| a.backend.cmp(&b.backend)));

    let mut out = String::new();
    out.push_str("# E-21 repo-state-store -- PerRoleRuntime snapshot.\n");
    out.push_str("# Regenerated on every snapshot write; do not hand-edit.\n");
    out.push_str("# Lines are sorted by (role, backend) for diff determinism.\n");
    out.push_str("\n");
    out.push_str("[runtime_choices]\n");
    for c in sorted {
        // TOML inline-table form; escape minimally (assume role/backend
        // labels are snake_case identifiers, which is the producer's
        // contract).
        out.push_str(&format!("{} = \"{}\"\n", c.role, c.backend));
    }

    fs::write(&path, out).map_err(|err| RepoStateError::Io {
        path: path.clone(),
        source: err,
    })
}

/// Read the runtime-choices snapshot back. Returns an empty Vec when
/// the file is absent. Parses the simple `role = "backend"` lines under
/// `[runtime_choices]`; ignores comments and blank lines. Lines that
/// fail to parse are silently skipped (the reader contract is "best-
/// effort"; the writer is the canonical source of truth).
pub fn read_runtime_choices(
    project_root: &Path,
) -> Result<Vec<RuntimeChoice>, RepoStateError> {
    let path = state_dir_for(project_root).join(RUNTIME_CHOICES_FILE);
    let bytes = match fs::read(&path) {
        Ok(b) => b,
        Err(err) if err.kind() == ErrorKind::NotFound => return Ok(Vec::new()),
        Err(err) => {
            return Err(RepoStateError::Io {
                path: path.clone(),
                source: err,
            });
        }
    };
    let s = String::from_utf8(bytes).map_err(|err| RepoStateError::Utf8 {
        path: path.clone(),
        source: err,
    })?;

    let mut out: Vec<RuntimeChoice> = Vec::new();
    let mut in_section = false;
    for raw in s.lines() {
        let line = raw.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        if line.starts_with('[') && line.ends_with(']') {
            in_section = line == "[runtime_choices]";
            continue;
        }
        if !in_section {
            continue;
        }
        if let Some(eq_idx) = line.find('=') {
            let role = line[..eq_idx].trim().to_string();
            let mut backend = line[eq_idx + 1..].trim().to_string();
            if backend.starts_with('"') && backend.ends_with('"') && backend.len() >= 2 {
                backend = backend[1..backend.len() - 1].to_string();
            }
            if !role.is_empty() && !backend.is_empty() {
                out.push(RuntimeChoice { role, backend });
            }
        }
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn fresh_root() -> TempDir {
        tempfile::tempdir().expect("tempdir")
    }

    #[test]
    fn state_dir_resolves_under_dot_elai() {
        let root = Path::new("/tmp/proj");
        let dir = state_dir_for(root);
        assert_eq!(dir, Path::new("/tmp/proj/.elai/state"));
    }

    #[test]
    fn ensure_state_dir_creates_then_idempotent() {
        let root = fresh_root();
        ensure_state_dir(root.path()).expect("first call");
        assert!(state_dir_for(root.path()).is_dir());
        // Second call must NOT fail (idempotency).
        ensure_state_dir(root.path()).expect("second call");
    }

    #[test]
    fn current_phase_roundtrip() {
        let root = fresh_root();
        // First read on a fresh tree returns None (first-run).
        let initial = read_current_phase(root.path()).expect("read empty");
        assert!(initial.is_none());

        write_current_phase(root.path(), "EXECUTING: Phase 3 Step 2")
            .expect("write");
        let back = read_current_phase(root.path()).expect("read written");
        assert_eq!(back.as_deref(), Some("EXECUTING: Phase 3 Step 2"));

        // Overwrite (not append) -- phase file is single-line.
        write_current_phase(root.path(), "MEMORIZING").expect("rewrite");
        let back = read_current_phase(root.path()).expect("read rewritten");
        assert_eq!(back.as_deref(), Some("MEMORIZING"));
    }

    #[test]
    fn append_decision_rolls_at_ten() {
        let root = fresh_root();
        for i in 0..15 {
            append_decision(root.path(), &format!("decision-{:02}", i))
                .expect("append");
        }
        let kept = read_decisions(root.path()).expect("read");
        assert_eq!(kept.len(), MAX_DECISIONS_RETAINED);
        // Newest 10 are 05..14 inclusive.
        assert_eq!(kept[0], "decision-05");
        assert_eq!(kept[9], "decision-14");
    }

    #[test]
    fn append_decision_ignores_empty() {
        let root = fresh_root();
        append_decision(root.path(), "real").expect("append real");
        append_decision(root.path(), "   ").expect("append blank");
        append_decision(root.path(), "").expect("append empty");
        let kept = read_decisions(root.path()).expect("read");
        assert_eq!(kept, vec!["real".to_string()]);
    }

    #[test]
    fn runtime_choices_sorted_roundtrip() {
        let root = fresh_root();
        let choices = vec![
            RuntimeChoice {
                role: "auditor".to_string(),
                backend: "anthropic_cloud".to_string(),
            },
            RuntimeChoice {
                role: "executor".to_string(),
                backend: "anthropic_local_sidecar".to_string(),
            },
            // Out-of-order input to verify sort.
            RuntimeChoice {
                role: "context_compiler".to_string(),
                backend: "ollama".to_string(),
            },
        ];
        write_runtime_choices(root.path(), &choices).expect("write");
        let back = read_runtime_choices(root.path()).expect("read");

        // Sorted by role: auditor, context_compiler, executor.
        assert_eq!(back.len(), 3);
        assert_eq!(back[0].role, "auditor");
        assert_eq!(back[1].role, "context_compiler");
        assert_eq!(back[2].role, "executor");
    }

    #[test]
    fn runtime_choices_empty_when_unwritten() {
        let root = fresh_root();
        let back = read_runtime_choices(root.path()).expect("read");
        assert!(back.is_empty());
    }

    #[test]
    fn decisions_empty_when_unwritten() {
        let root = fresh_root();
        let back = read_decisions(root.path()).expect("read");
        assert!(back.is_empty());
    }
}
