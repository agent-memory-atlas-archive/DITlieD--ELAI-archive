//! E-54 storage layout: per-family directory shards + `_index/{family}.jsonl`
//! summary append.
//!
//! Layout:
//!   - `.elai_cc/held-out/{family_snake}/{YYYY-MM-DD}/{id}.toml`
//!     (per-family, per-date shard; daily rotation caps per-dir file count
//!     well below the NTFS perf cliff documented in arXiv:2505.23419 +
//!     Microsoft NTFS docs).
//!   - `.elai_cc/held-out/_index/{family_snake}.jsonl`
//!     (one row per task: `{id, path, created_at, cutoffs_keys, embedding_hash, ngram_hash}`).
//!   - `.elai_cc/held-out/_quarantine/` (failed-validation quarantine).
//!   - `.elai_cc/held-out/_audit/knowledge-qa-{date}.jsonl`
//!     (dynamic knowledge-QA audit log; see [`crate::dynamic_knowledge_qa`]).
//!
//! `family_snake` is [`crate::TaskFamily::as_str`] (e.g. `code_fix`,
//! `math_proof`, `prose_reason`, `planning_decomposition`, `tool_use`) so the
//! on-disk directory names match the canary surface. Per E-54-decisions.md
//! DECISION-1, the plan's hyphenated names (`code-fix` etc.) are REJECTED
//! in favour of the CanaryFamily snake-case wire labels.

use std::io::Write;
use std::path::{Path, PathBuf};

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::TaskFamily;
use crate::task_schema::Task;

/// Per-(family, date) shard maximum file count. Above this, daily rotation
/// kicks in (a new sub-shard for the next day). Microsoft NTFS shows fragment
/// / lookup latency jumps above ~5K files in a single directory; ELAI caps at
/// 500 per (family, date) so 10 distinct dates cover the 5K threshold.
pub const SHARD_MAX_FILES_PER_DATE: usize = 500;

/// Subdirectory name for the per-family JSONL index files.
pub const INDEX_SUBDIR: &str = "_index";

/// Subdirectory name for tasks that fail validation during migrate.
pub const QUARANTINE_SUBDIR: &str = "_quarantine";

/// Subdirectory name for the audit JSONL files (dynamic knowledge-QA + prod
/// traffic probes).
pub const AUDIT_SUBDIR: &str = "_audit";

/// Subdirectory name for prod-traffic-derived tasks (OPT-IN only; see
/// [`crate::prod_traffic_grade`]).
pub const FROM_PROD_SUBDIR: &str = "from-prod";

/// Storage-level errors.
#[derive(Debug, Error)]
pub enum StorageError {
    #[error("eval-corpus storage IO at {0}: {1}")]
    Io(String, String),
    #[error("eval-corpus storage JSON at {0}: {1}")]
    Json(String, String),
}

/// One row in the `_index/{family}.jsonl` summary file.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct IndexRow {
    pub id: String,
    pub path: String,
    pub created_at: DateTime<Utc>,
    /// Sorted list of model_id strings present in the task's `cutoffs` map.
    /// Avoids dumping the full map into every index row.
    pub cutoffs_keys: Vec<String>,
    pub embedding_hash: String,
    pub ngram_hash: String,
}

/// Returns the canonical shard path for a task:
/// `{root}/held-out/{family_snake}/{YYYY-MM-DD}/{id}.toml`.
#[must_use]
pub fn shard_path(root: &Path, family: TaskFamily, created_at: DateTime<Utc>, id: &str) -> PathBuf {
    let date = created_at.format("%Y-%m-%d").to_string();
    root.join("held-out")
        .join(family.as_str())
        .join(date)
        .join(format!("{id}.toml"))
}

/// Returns the per-family index path:
/// `{root}/held-out/_index/{family_snake}.jsonl`.
#[must_use]
pub fn index_path(root: &Path, family: TaskFamily) -> PathBuf {
    root.join("held-out")
        .join(INDEX_SUBDIR)
        .join(format!("{}.jsonl", family.as_str()))
}

/// Returns the per-family quarantine directory.
#[must_use]
pub fn quarantine_dir(root: &Path, family: TaskFamily) -> PathBuf {
    root.join("held-out")
        .join(QUARANTINE_SUBDIR)
        .join(family.as_str())
}

/// Returns the audit directory for ad-hoc per-day logs.
#[must_use]
pub fn audit_dir(root: &Path) -> PathBuf {
    root.join("held-out").join(AUDIT_SUBDIR)
}

/// Returns the prod-traffic shard for opt-in silent probes.
#[must_use]
pub fn from_prod_dir(root: &Path, date: DateTime<Utc>) -> PathBuf {
    root.join("held-out")
        .join(FROM_PROD_SUBDIR)
        .join(date.format("%Y-%m-%d").to_string())
}

/// Write a task to its shard TOML path. Creates parent directories as needed.
/// Returns the absolute path written.
pub fn write_task_to_shard(root: &Path, task: &Task) -> Result<PathBuf, StorageError> {
    let p = shard_path(root, task.family, task.created_at, &task.id);
    if let Some(parent) = p.parent() {
        std::fs::create_dir_all(parent).map_err(|e| {
            StorageError::Io(parent.display().to_string(), e.to_string())
        })?;
    }
    let toml_string = task.to_toml().map_err(|e| {
        StorageError::Io(p.display().to_string(), e.to_string())
    })?;
    std::fs::write(&p, toml_string).map_err(|e| {
        StorageError::Io(p.display().to_string(), e.to_string())
    })?;
    Ok(p)
}

/// Append an index row to `_index/{family}.jsonl`. Creates parent directories
/// and the file as needed. Append-only via `OpenOptions::append(true)` --
/// safe across concurrent writers on Linux + Windows for `write()` calls
/// shorter than PIPE_BUF (4 KiB).
pub fn append_index(root: &Path, family: TaskFamily, row: &IndexRow) -> Result<(), StorageError> {
    let p = index_path(root, family);
    if let Some(parent) = p.parent() {
        std::fs::create_dir_all(parent).map_err(|e| {
            StorageError::Io(parent.display().to_string(), e.to_string())
        })?;
    }
    let mut line = serde_json::to_string(row).map_err(|e| {
        StorageError::Json(p.display().to_string(), e.to_string())
    })?;
    line.push('\n');
    let mut file = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(&p)
        .map_err(|e| StorageError::Io(p.display().to_string(), e.to_string()))?;
    file.write_all(line.as_bytes()).map_err(|e| {
        StorageError::Io(p.display().to_string(), e.to_string())
    })?;
    Ok(())
}

/// Count files in a (family, date) shard directory. Returns 0 if the
/// directory doesn't exist. Used to detect when daily rotation is needed
/// (above [`SHARD_MAX_FILES_PER_DATE`]).
pub fn shard_file_count(root: &Path, family: TaskFamily, created_at: DateTime<Utc>) -> usize {
    let shard = root
        .join("held-out")
        .join(family.as_str())
        .join(created_at.format("%Y-%m-%d").to_string());
    let Ok(read) = std::fs::read_dir(&shard) else {
        return 0;
    };
    read.filter_map(|e| e.ok())
        .filter(|e| {
            e.path()
                .extension()
                .and_then(|s| s.to_str())
                .map(|s| s.eq_ignore_ascii_case("toml"))
                .unwrap_or(false)
        })
        .count()
}

/// Quarantine a malformed / failed-validation task. Writes the raw TOML body
/// + a `.reason` text sibling. Idempotent (overwrites).
pub fn quarantine_raw(
    root: &Path,
    family: TaskFamily,
    id: &str,
    raw_toml: &str,
    reason: &str,
) -> Result<PathBuf, StorageError> {
    let dir = quarantine_dir(root, family);
    std::fs::create_dir_all(&dir).map_err(|e| {
        StorageError::Io(dir.display().to_string(), e.to_string())
    })?;
    let p = dir.join(format!("{id}.toml"));
    std::fs::write(&p, raw_toml).map_err(|e| {
        StorageError::Io(p.display().to_string(), e.to_string())
    })?;
    let reason_path = dir.join(format!("{id}.reason"));
    std::fs::write(&reason_path, reason).map_err(|e| {
        StorageError::Io(reason_path.display().to_string(), e.to_string())
    })?;
    Ok(p)
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;
    use tempfile::tempdir;

    #[test]
    fn shard_path_uses_canonical_snake_family() {
        let root = Path::new("/tmp/elai");
        let dt = Utc.with_ymd_and_hms(2026, 5, 27, 0, 0, 0).unwrap();
        let p = shard_path(root, TaskFamily::CodeFix, dt, "abc");
        assert!(p.to_string_lossy().contains("/code_fix/2026-05-27/abc.toml")
            || p.to_string_lossy().contains("\\code_fix\\2026-05-27\\abc.toml"));
    }

    #[test]
    fn index_path_per_family() {
        let root = Path::new("/tmp/elai");
        let p = index_path(root, TaskFamily::PlanningDecomposition);
        assert!(p.to_string_lossy().contains("planning_decomposition.jsonl"));
    }

    #[test]
    fn append_index_creates_jsonl() {
        let tmp = tempdir().unwrap();
        let row = IndexRow {
            id: "x".to_string(),
            path: "p".to_string(),
            created_at: Utc.with_ymd_and_hms(2026, 5, 27, 0, 0, 0).unwrap(),
            cutoffs_keys: vec!["m".to_string()],
            embedding_hash: "h".to_string(),
            ngram_hash: "n".to_string(),
        };
        append_index(tmp.path(), TaskFamily::CodeFix, &row).unwrap();
        let p = index_path(tmp.path(), TaskFamily::CodeFix);
        let body = std::fs::read_to_string(p).unwrap();
        assert!(body.contains("\"id\":\"x\""));
    }

    #[test]
    fn shard_file_count_returns_zero_for_missing_dir() {
        let tmp = tempdir().unwrap();
        let dt = Utc.with_ymd_and_hms(2026, 5, 27, 0, 0, 0).unwrap();
        assert_eq!(shard_file_count(tmp.path(), TaskFamily::CodeFix, dt), 0);
    }
}
